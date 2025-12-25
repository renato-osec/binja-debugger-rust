// cargo run --example headless_debug -- /path/to/binary

use binaryninja::architecture::Register;
use binaryninja::binary_view::{BinaryView, BinaryViewBase, BinaryViewExt};
use binaryninja::headless::Session;
use binaryninja::low_level_il::{
    expression::ExpressionHandler,
    instruction::{InstructionHandler, LowLevelILInstructionKind},
    LowLevelILRegisterKind, LowLevelILRegularExpressionKind,
};
use binja_debugger::{stop_reason_string, BNDebugStopReason, DebuggerController};
use binaryninja::section::Semantics;
use std::env;

const MAX_VTABLE_SIZE : u64 = 0x18 +  1024 * 8;

/// Info about a call that dereferences rax: call [rax] or call [rax + offset]
#[derive(Clone)]
struct RaxDerefCall {
    addr: u64,
    offset: i64,
}

/// Read a u64 from the binary view at the given address
fn read_u64(bv: &BinaryView, addr: u64) -> Option<u64> {
    let mut buf = [0u8; 8];
    if bv.read(&mut buf, addr) == 8 {
        Some(u64::from_le_bytes(buf))
    } else {
        None
    }
}

/// Check if an address points to executable code
fn is_code_ptr(bv: &BinaryView, ptr: u64) -> bool {
    if ptr == 0 {
        return false;
    }

    // Check if pointer is within a code section
    for section in bv.sections().iter() {
        let start = section.start();
        let end = start + section.len() as u64;
        if ptr >= start && ptr < end {
            return section.semantics() == Semantics::ReadOnlyCode;
        }
    }
    false
}

/// Find calls that dereference rax: call [rax] or call [rax + offset]
fn find_rax_deref_calls(bv: &BinaryView) -> Vec<RaxDerefCall> {
    let mut calls = Vec::new();
    for f in bv.functions().iter() {
        let Ok(llil) = f.low_level_il() else { continue };
        for b in llil.basic_blocks().iter() {
            for i in b.iter() {
                if let LowLevelILInstructionKind::Call(call_op) = i.kind() {
                    if let LowLevelILRegularExpressionKind::Load(load_op) = call_op.target().kind()
                    {
                        if let Some(offset) = check_rax_expr(load_op.source_expr().kind()) {
                            calls.push(RaxDerefCall {
                                addr: i.address(),
                                offset,
                            });
                        }
                    }
                }
            }
        }
    }
    calls
}

fn determine_vtable_size(bv: &BinaryView, ptr: u64) -> Option<u64> {
    for i in (0x18..MAX_VTABLE_SIZE).step_by(8) {
        if let Some(addr) = read_u64(bv, ptr.wrapping_add(i)) {
            if !is_code_ptr(bv, addr) {
                return Some((i - 0x18) / 8);
            }
        }
    }
    None
}

/// Check if expression is rax or rax+const, return offset if so
fn check_rax_expr(kind: LowLevelILRegularExpressionKind) -> Option<i64> {
    match kind {
        LowLevelILRegularExpressionKind::Reg(op) => {
            if let LowLevelILRegisterKind::Arch(r) = op.source_reg() {
                if r.name().as_ref() == "rax" {
                    return Some(0);
                }
            }
            None
        }
        LowLevelILRegularExpressionKind::Add(op) => {
            // Check left=rax, right=const
            if let LowLevelILRegularExpressionKind::Reg(reg_op) = op.left().kind() {
                if let LowLevelILRegisterKind::Arch(r) = reg_op.source_reg() {
                    if r.name().as_ref() == "rax" {
                        if let LowLevelILRegularExpressionKind::Const(const_op) = op.right().kind()
                        {
                            return Some(const_op.value() as i64);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <binary> [args...]", args[0]);
        std::process::exit(1);
    }

    let binary_path = &args[1];
    let binary_args = if args.len() > 2 {
        args[2..].join(" ")
    } else {
        String::new()
    };

    let session = match Session::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cant init {:?}", e);
            std::process::exit(1);
        }
    };

    let bv = match session.load(binary_path) {
        Some(bv) => bv,
        None => {
            eprintln!("cant load {}", binary_path);
            std::process::exit(1);
        }
    };

    println!("loaded: {} ep=0x{:x}", binary_path, bv.entry_point());

    // Find call [rax...] patterns
    let rax_calls = find_rax_deref_calls(&bv);
    println!("found {} call [rax...] sites:", rax_calls.len());
    for c in &rax_calls {
        if c.offset == 0 {
            println!("  0x{:x}: call [rax]", c.addr);
        } else {
            println!("  0x{:x}: call [rax + 0x{:x}]", c.addr, c.offset);
        }

    }

    if rax_calls.is_empty() {
        println!("no rax deref calls found, exiting");
        return;
    }

    // Setup debugger
    let dbg = match DebuggerController::new(&bv) {
        Some(d) => d,
        None => {
            eprintln!("cant create debugger");
            std::process::exit(1);
        }
    };

    dbg.set_executable_path(binary_path);
    if !binary_args.is_empty() {
        dbg.set_command_line_arguments(&binary_args);
    }

    //bv.default_platform().unwrap().parse_types_from_source(src, filename, include_dirs, auto_type_source)

    // Launch
    let reason = dbg.launch_and_wait();
    println!("launch: {}", stop_reason_string(reason));
    if reason == BNDebugStopReason::ProcessExited {
        println!("process exited early, code={}", dbg.exit_code());
        return;
    }

    // Set breakpoints
    for c in &rax_calls {
        dbg.add_breakpoint(c.addr);
    }

    // Run and log load addresses
    println!("\n--- tracing call [rax...] ---");
    loop {
        let reason = dbg.go_and_wait();
        if reason == BNDebugStopReason::ProcessExited {
            println!("process exited, code={}", dbg.exit_code());
            break;
        }

        let ip = dbg.ip();
        if let Some(call) = rax_calls.iter().find(|c| c.addr == ip) {
            let rax_bytes = dbg.get_register_value("rax");
            let rax = u64::from_le_bytes(rax_bytes[..8].try_into().unwrap_or([0; 8]));
            let load_addr = rax.wrapping_add(call.offset as u64);

            
            println!("vtable {:?}", determine_vtable_size(&bv, rax));

            // Read the pointer at load_addr
            let target = if let Some(bytes) = dbg.read_memory(load_addr, 8) {
                u64::from_le_bytes(bytes.try_into().unwrap_or([0; 8]))
            } else {
                0
            };

            println!(
                "0x{:x}: rax=0x{:x} load_addr=0x{:x} -> target=0x{:x}",
                ip, rax, load_addr, target
            );
        }
    }

    dbg.quit_and_wait();
}
