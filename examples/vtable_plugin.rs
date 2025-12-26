// Binary Ninja plugin that registers the vtable call targets render layer
//
// Build: cargo build --example vtable_plugin --release
// Install: Copy target/release/examples/libvtable_plugin.so to ~/.binaryninja/plugins/

use binja_debugger::vtable_render_layer;

#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
    vtable_render_layer::register();
    true
}
