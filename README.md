## Binja Debugger for Rust

The native Rust Binja API does not support the use of the debugger. 

This repo aims to provide that same functionality in Rust, meant to be used for the development of headless analysis tools and plugins

### repo structure

- `src` contains all the ffi and type definitions
- `examples` contains examples of how to use the debugger

### why?

I wanted to restore the full functionality of the C++ Binja API in Rust, and hope to push some version of this upstream to the Binja API
