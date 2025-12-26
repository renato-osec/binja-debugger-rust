#![allow(dead_code)]

use binaryninja::architecture::Register;
use binaryninja::binary_view::{BinaryView, BinaryViewBase, BinaryViewExt};
use binaryninja::low_level_il::{
    expression::ExpressionHandler,
    instruction::{InstructionHandler, LowLevelILInstructionKind},
    LowLevelILRegisterKind, LowLevelILRegularExpressionKind,
};
use binaryninja::section::Semantics;
use binja_debugger::DebuggerController;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const MAX_VTABLE_SIZE: u64 = 0x18 + 1024 * 8;

#[derive(Clone)]
pub struct RaxDerefCall {
    pub addr: u64,
    pub offset: i64,
}

#[derive(Clone, Debug)]
pub struct VTableObservation {
    pub vtable_ptr: u64,
    pub method_count: u64,
}

#[derive(Clone, Debug)]
pub struct CallSiteInfo {
    pub addr: u64,
    pub offset: i64,
    pub observations: Vec<VTableObservation>,
}

pub fn read_u64(bv: &BinaryView, addr: u64) -> Option<u64> {
    let mut buf = [0u8; 8];
    if bv.read(&mut buf, addr) == 8 {
        Some(u64::from_le_bytes(buf))
    } else {
        None
    }
}

pub fn is_code_ptr(bv: &BinaryView, ptr: u64) -> bool {
    if ptr == 0 {
        return false;
    }
    for section in bv.sections().iter() {
        let start = section.start();
        let end = start + section.len() as u64;
        if ptr >= start && ptr < end {
            return section.semantics() == Semantics::ReadOnlyCode;
        }
    }
    false
}

pub fn find_rax_deref_calls(bv: &BinaryView) -> Vec<RaxDerefCall> {
    let binding = bv.functions();
    let functions: Vec<_> = binding.iter().collect();
    let total = functions.len();
    let processed = AtomicUsize::new(0);
    let last_report = AtomicUsize::new(0);

    println!("  scanning {} functions...", total);

    let calls: Vec<RaxDerefCall> = functions
        .par_iter()
        .flat_map(|f| {
            let current = processed.fetch_add(1, Ordering::Relaxed);
            // Report progress every 10000 functions
            if current % 10000 == 0 {
                let last = last_report.swap(current, Ordering::Relaxed);
                if current > last {
                    eprintln!("  {}/{} functions scanned...", current, total);
                }
            }

            let Ok(llil) = f.low_level_il() else { return vec![] };
            let mut func_calls = Vec::new();
            for b in llil.basic_blocks().iter() {
                for i in b.iter() {
                    if let LowLevelILInstructionKind::Call(call_op) = i.kind() {
                        if let LowLevelILRegularExpressionKind::Load(load_op) = call_op.target().kind() {
                            if let Some(offset) = check_rax_expr(load_op.source_expr().kind()) {
                                func_calls.push(RaxDerefCall {
                                    addr: i.address(),
                                    offset,
                                });
                            }
                        }
                    }
                }
            }
            func_calls
        })
        .collect();

    calls
}

pub fn determine_vtable_size(bv: &BinaryView, ptr: u64) -> Option<u64> {
    for i in (0x18..MAX_VTABLE_SIZE).step_by(8) {
        if let Some(addr) = read_u64(bv, ptr.wrapping_add(i)) {
            if !is_code_ptr(bv, addr) {
                return Some((i - 0x18) / 8);
            }
        }
    }
    None
}

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
            if let LowLevelILRegularExpressionKind::Reg(reg_op) = op.left().kind() {
                if let LowLevelILRegisterKind::Arch(r) = reg_op.source_reg() {
                    if r.name().as_ref() == "rax" {
                        if let LowLevelILRegularExpressionKind::Const(const_op) = op.right().kind() {
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

pub fn read_u64_runtime(dbg: &DebuggerController, addr: u64) -> Option<u64> {
    let bytes = dbg.read_memory(addr, 8)?;
    Some(u64::from_le_bytes(bytes.try_into().ok()?))
}

pub fn is_code_ptr_runtime(dbg_bv: &BinaryView, ptr: u64) -> bool {
    if ptr == 0 {
        return false;
    }
    for section in dbg_bv.sections().iter() {
        let start = section.start();
        let end = start + section.len() as u64;
        if ptr >= start && ptr < end {
            return section.semantics() == Semantics::ReadOnlyCode;
        }
    }
    false
}

pub fn determine_vtable_size_runtime(
    dbg: &DebuggerController,
    dbg_bv: &BinaryView,
    ptr: u64,
) -> Option<u64> {
    for i in (0x18..MAX_VTABLE_SIZE).step_by(8) {
        match read_u64_runtime(dbg, ptr.wrapping_add(i)) {
            Some(addr) if !is_code_ptr_runtime(dbg_bv, addr) => {
                return Some((i - 0x18) / 8);
            }
            None => return Some((i - 0x18) / 8),
            _ => {}
        }
    }
    None
}

pub fn generate_vtable_type_def(call_site_addr: u64, method_count: u64) -> String {
    let mut def = format!("struct VTable_{:x} {{\n", call_site_addr);
    def.push_str("    void* drop;\n");
    def.push_str("    uint64_t size;\n");
    def.push_str("    uint64_t align;\n");
    for i in 0..method_count {
        def.push_str(&format!("    void* method{};\n", i + 1));
    }
    def.push_str("};\n");
    def
}

pub fn define_vtable_type(bv: &BinaryView, call_site_addr: u64, method_count: u64) -> bool {
    let type_def = generate_vtable_type_def(call_site_addr, method_count);

    let Some(platform) = bv.default_platform() else {
        return false;
    };

    match platform.parse_types_from_source(&type_def, "", &[], "") {
        Ok(result) => {
            for p in result.types.iter() {
                bv.define_user_type(p.name.to_string(), &p.ty);
            }
            true
        }
        Err(e) => {
            eprintln!("parse error: {:?}", e);
            false
        }
    }
}

pub fn retype_vtable_variable(
    bv: &BinaryView,
    call_site_addr: u64,
    _method_count: u64,
) -> bool {
    let type_name = format!("VTable{:x}", call_site_addr);
    let ptr_type_str = format!("struct {}*", type_name);

    let funcs = bv.functions_containing(call_site_addr);
    let Some(func) = funcs.iter().next() else {
        return false;
    };

    let Ok(hlil) = func.high_level_il(true) else {
        return false;
    };

    // Parse the pointer type using the type container
    let Ok(parsed) = bv.type_container().parse_type_string(&ptr_type_str, true) else {
        return false;
    };

    for block in hlil.basic_blocks().iter() {
        for instr in block.iter() {
            if instr.address != call_site_addr {
                continue;
            }

            if let Some(var_refs) = extract_call_base_var(&hlil, &instr) {
                for var in var_refs {
                    func.create_user_var(&var, parsed.ty.as_ref(), "", false);
                    return true;
                }
            }
        }
    }
    false
}

fn extract_call_base_var(
    hlil: &binaryninja::high_level_il::HighLevelILFunction,
    instr: &binaryninja::high_level_il::HighLevelILInstruction,
) -> Option<Vec<binaryninja::variable::Variable>> {
    use binaryninja::high_level_il::HighLevelILInstructionKind;

    match &instr.kind {
        HighLevelILInstructionKind::Call(call) => {
            hlil.instruction_from_expr_index(call.dest)
                .and_then(|dest_instr| extract_vars_from_expr(hlil, &dest_instr))
        }
        HighLevelILInstructionKind::Tailcall(call) => {
            hlil.instruction_from_expr_index(call.dest)
                .and_then(|dest_instr| extract_vars_from_expr(hlil, &dest_instr))
        }
        _ => None,
    }
}

fn extract_vars_from_expr(
    hlil: &binaryninja::high_level_il::HighLevelILFunction,
    expr: &binaryninja::high_level_il::HighLevelILInstruction,
) -> Option<Vec<binaryninja::variable::Variable>> {
    use binaryninja::high_level_il::HighLevelILInstructionKind;

    let mut vars = Vec::new();

    match &expr.kind {
        HighLevelILInstructionKind::Deref(deref) => {
            if let Some(src_instr) = hlil.instruction_from_expr_index(deref.src) {
                if let Some(v) = extract_vars_from_expr(hlil, &src_instr) {
                    vars.extend(v);
                }
            }
        }
        HighLevelILInstructionKind::Add(add) => {
            if let Some(left_instr) = hlil.instruction_from_expr_index(add.left) {
                if let Some(v) = extract_vars_from_expr(hlil, &left_instr) {
                    vars.extend(v);
                }
            }
            if let Some(right_instr) = hlil.instruction_from_expr_index(add.right) {
                if let Some(v) = extract_vars_from_expr(hlil, &right_instr) {
                    vars.extend(v);
                }
            }
        }
        HighLevelILInstructionKind::Var(var_op) => {
            vars.push(var_op.var.clone());
        }
        _ => {}
    }

    if vars.is_empty() {
        None
    } else {
        Some(vars)
    }
}
