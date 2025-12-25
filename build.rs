use std::env;

fn main() {
    //binja commerical path
    let binja_path = env::var("BINJA_PATH").unwrap();

    // Link against debugger core
    println!("cargo:rustc-link-search=native={}/plugins", binja_path);
    println!("cargo:rustc-link-lib=dylib:+verbatim=libdebuggercore.so");

    // Also need binaryninjacore
    println!("cargo:rustc-link-search=native={}", binja_path);
    println!("cargo:rustc-link-lib=dylib:+verbatim=libbinaryninjacore.so.1");

    // Set rpath for runtime
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", binja_path);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}/plugins", binja_path);

    // Rerun if BINJA_PATH changes
    println!("cargo:rerun-if-env-changed=BINJA_PATH");
}
