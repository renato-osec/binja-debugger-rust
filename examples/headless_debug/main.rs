mod macros;
mod utils;

use binaryninja::binary_view::{BinaryViewBase, BinaryViewExt};
use binaryninja::headless::Session;
use binja_debugger::{stop_reason_string, BNDebugStopReason, DebuggerController};
use std::collections::HashMap;
use std::env;

use utils::{define_vtable_type, CallSiteInfo, VTableObservation};

use crate::utils::{RaxDerefCall, determine_vtable_size, find_rax_deref_calls, read_u64};

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

    println!("initializing binja session...");
    let session = match Session::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cant init {:?}", e);
            std::process::exit(1);
        }
    };

    println!("loading {}...", binary_path);
    let bv = match session.load(binary_path) {
        Some(bv) => bv,
        None => {
            eprintln!("cant load {}", binary_path);
            std::process::exit(1);
        }
    };

    let file_base = bv.start();
    let file_entry = bv.entry_point();
    println!("loaded: base=0x{:x} ep=0x{:x}", file_base, file_entry);

    // Find call [rax...] patterns
    let rax_calls = find_rax_deref_calls(&bv);
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
    println!("setting up debugger...");
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

    // Launch
    println!("launching...");
    let reason = dbg.launch_and_wait();
    println!("launch: {}", stop_reason_string(reason));

    if reason == BNDebugStopReason::ProcessExited {
        println!("process exited early, code={}", dbg.exit_code());
        return;
    }

    // Get runtime IP to calculate rebase offset for PIE
    let runtime_ip = dbg.ip();
    println!("stopped at runtime IP: 0x{:x}", runtime_ip);

    let rebase_offset = if runtime_ip != file_entry && runtime_ip != 0 {
        let offset = runtime_ip as i64 - file_entry as i64;
        println!("PIE: rebase_offset=0x{:x}", offset);
        offset
    } else {
        println!("no PIE or same base");
        0
    };

    // Set breakpoints at RUNTIME addresses
    println!("setting {} breakpoints at runtime addresses...", rax_calls.len());
    let mut bp_to_call: HashMap<u64, &RaxDerefCall> = HashMap::new();
    for c in &rax_calls {
        println!("RUNTIME {:#x}", c.addr);
        dbg.add_breakpoint(c.addr);
        bp_to_call.insert(c.addr, c);
    }

    // Track observations
    let mut call_sites: HashMap<u64, CallSiteInfo> = HashMap::new();
    let mut hit_count = 0;

    loop {
        let reason = dbg.go_and_wait();

        if reason == BNDebugStopReason::ProcessExited {
            println!("process exited, code={}", dbg.exit_code());
            break;
        }

        let runtime_ip = dbg.ip();

        if let Some(call) = bp_to_call.get(&runtime_ip) {
            let rax_bytes = dbg.get_register_value("rax");
            let rax = u64::from_le_bytes(rax_bytes[..8].try_into().unwrap_or([0; 8]));

            let target = read_u64(&bv, call.addr + call.offset as u64).unwrap_or(0);
            
            //add indirect xrefs
            curr_func!(&dbg).add_user_code_ref(runtime_ip, target, None);

            if rax == 0 {
                continue;
            }

            let method_count = determine_vtable_size(&bv, rax).unwrap();
            let file_addr = call.addr;

            let entry = call_sites.entry(file_addr).or_insert_with(|| CallSiteInfo {
                addr: file_addr,
                called_methods: Vec::new(),
                observations: Vec::new(),
            });

            let already_seen = entry.observations.iter().any(|o| o.vtable_ptr == rax);
            if !already_seen {
                entry.observations.push(VTableObservation {
                    vtable_ptr: rax,
                    method_count,
                });

                entry.called_methods.push(target);
                hit_count += 1;

                println!(
                    "HIT {}: file=0x{:x} vtable=0x{:x} methods={} offset=0x{:x}",
                    hit_count, file_addr, rax, method_count, call.offset
                );
            }
        }
    }

    for (file_addr, info) in &call_sites {
        bv.set_comment_at(*file_addr, comment!(info.called_methods).as_str());
    }


    if call_sites.is_empty() {
        println!("no vtable hits recorded");
        return;
    }

    // Reload view before defining types
    println!("\nreloading binary view...");
    dbg.quit_and_wait();
    drop(bv);
    let bv = session.load(binary_path).expect("failed to reload");

    // Define types
    for (file_addr, info) in &call_sites {
        let max_methods = info.observations.iter().map(|o| o.method_count).max().unwrap_or(0);

        if max_methods > 0 {
            if define_vtable_type(&bv, *file_addr, max_methods) {
                println!("defined VTable_{:x} with {} methods", file_addr, max_methods);
            }
        }
    }

    for t in bv.types().iter() {
        println!("TYP {:?}", t);
    }

    // Save
    let db_path = format!("{}.vtable.bndb", binary_path);
    println!("\nsaving to {}...", db_path);
    if bv.file().create_database(&db_path) {
        println!("saved");
    }
}
