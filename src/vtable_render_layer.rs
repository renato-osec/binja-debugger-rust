// RenderLayer plugin to display vtable call targets inline
//
// Shows possible call targets for indirect calls like:
//   call qword [rax+0x18]  ; {Dog::speak, Cat::speak, Rakaraka::speak}

use binaryninja::basic_block::BasicBlock;
use binaryninja::binary_view::BinaryViewExt;
use binaryninja::disassembly::{
    DisassemblyTextLine, InstructionTextToken, InstructionTextTokenContext,
    InstructionTextTokenKind,
};
use binaryninja::function::NativeBlock;
use binaryninja::render_layer::{register_render_layer, RenderLayer, RenderLayerDefaultState};

pub struct VtableCallTargetsLayer;

impl RenderLayer for VtableCallTargetsLayer {
    fn apply_to_disassembly_block(
        &self,
        block: &BasicBlock<NativeBlock>,
        lines: Vec<DisassemblyTextLine>,
    ) -> Vec<DisassemblyTextLine> {
        let func = block.function();
        let bv = func.view();
        let mut result = Vec::new();

        for line in lines {
            let addr = line.address;
            result.push(line);

            // Check for indirect branch targets at this address
            let branches = func.indirect_branches_at(addr, None);
            if branches.is_empty() {
                continue;
            }

            // Build target names
            let mut targets: Vec<String> = Vec::new();
            for branch in branches.iter() {
                let dest = branch.dest.addr;
                // Try to get function name at target
                let funcs = bv.functions_containing(dest);
                if let Some(target_func) = funcs.iter().next() {
                    targets.push(target_func.symbol().full_name().to_string_lossy().into_owned());
                } else {
                    targets.push(format!("0x{:x}", dest));
                }
            }

            if targets.is_empty() {
                continue;
            }

            // Create annotation line
            let annotation_text = format!("    ; possible targets: {{{}}}", targets.join(", "));
            let token = InstructionTextToken {
                address: addr,
                text: annotation_text,
                confidence: 255,
                context: InstructionTextTokenContext::Normal,
                expr_index: None,
                kind: InstructionTextTokenKind::Text,
            };

            let annotation_line = DisassemblyTextLine {
                address: addr,
                instruction_index: 0,
                tokens: vec![token],
                highlight: Default::default(),
                tags: vec![],
                type_info: Default::default(),
            };

            result.push(annotation_line);
        }

        result
    }
}

/// Register the vtable call targets render layer
pub fn register() -> &'static mut VtableCallTargetsLayer {
    let (layer, _core) = register_render_layer(
        "Vtable Call Targets",
        VtableCallTargetsLayer,
        RenderLayerDefaultState::Enabled,
    );
    layer
}
