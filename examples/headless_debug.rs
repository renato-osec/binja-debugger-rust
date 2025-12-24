// cargo run --example headless_debug -- /path/to/binary

use binaryninja::binary_view::BinaryViewBase;
use binaryninja::binary_view::BinaryViewExt;
use binaryninja::headless::Session;
use binja_debugger::{
    available_debug_adapters, stop_reason_string, BNDebugStopReason, BNFunctionGraphType,
    DebuggerController,
};
use trace_time::PerfTimer;
use std::env;

fn main() {
    //for perf tracing
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
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

    // Load the binary
    let bv = match session.load(binary_path) {
        Some(bv) => bv,
        None => {
            eprintln!("cant load {}", binary_path);
            std::process::exit(1);
        }
    };

    println!("ep: 0x{:x}", bv.entry_point());

    //get debuggers
    let adapters = available_debug_adapters(&bv);
    println!("\ndebuggers:");
    for adapter in &adapters {
        println!("{}", adapter);
    }

    let dbg = match DebuggerController::new(&bv) {
        Some(d) => d,
        None => {
            eprintln!("cant create dbg");
            std::process::exit(1);
        }
    };

    dbg.set_executable_path(binary_path);

    //args if needed
    if !binary_args.is_empty() {
        dbg.set_command_line_arguments(&binary_args);
    }

    //event handling
    /*
    let callback_id = dbg.register_event_callback("example_callback", |event| {
        println!("event {:?}", event.event_type);
        if let Some(reason) = event.stop_reason {
            println!("stop: {:?}", reason);
        }
        if let Some(msg) = &event.message {
            println!("msg: {}", msg);
        }
    });
    */

    //bp
    let entry = bv.entry_point();
    dbg.add_breakpoint(entry);

    // launch the process
    let stop_reason = dbg.launch_and_wait();
    println!(
        "stopping: {} ({:?})",
        stop_reason_string(stop_reason),
        stop_reason
    );

    if stop_reason == BNDebugStopReason::ProcessExited {
        println!("died =( code: {}", dbg.exit_code());
        return;
    }


    // Step a few times
    {
        let t = PerfTimer::new("step");
        for _ in 0..100 {
            let reason = dbg.step_into_and_wait(BNFunctionGraphType::NormalFunctionGraph);

            if reason == BNDebugStopReason::ProcessExited {
                println!("[*] Process exited");
                break;
            }
        }
    }

    // Print state after stepping
    print_debugger_state(&dbg);

    // Read some memory at the current IP
    let ip = dbg.ip();
    if let Some(data) = dbg.read_memory(ip, 16) {
        print!("    ");
        for (i, byte) in data.iter().enumerate() {
            print!("{:02x} ", byte);
            if (i + 1) % 8 == 0 {
                print!(" ");
            }
        }
        println!();
    } else {
        println!("    [-] Failed to read memory");
    }

    // Continue execution
    let reason = dbg.go_and_wait();
    println!(
        "stopped: {} ({:?})",
        stop_reason_string(reason),
        reason
    );

    if reason == BNDebugStopReason::ProcessExited {
        println!("exited with code: {}", dbg.exit_code());
    }

    // Cleanup
    //dbg.remove_event_callback(callback_id);
    dbg.quit_and_wait();
}

fn print_debugger_state(dbg: &DebuggerController) {
    println!("IP: 0x{:x}", dbg.ip());
    println!("SP: 0x{:x}", dbg.stack_pointer());
    println!("Running: {}", dbg.is_running());
    println!("Target status: {:?}", dbg.target_status());

    let registers = dbg.registers();
    println!("regs:");
    for reg in &registers {
        let name = reg.name.to_lowercase();
        if name == "rip"
            || name == "rsp"
            || name == "rbp"
            || name == "rax"
            || name == "rbx"
            || name == "rcx"
            || name == "rdx"
            || name == "rdi"
            || name == "rsi"
        {
            println!("{} = 0x{:016x}", reg.name, reg.value_u64());
        }
    }

    // memory modules
    let modules = dbg.modules();
    println!("\n  Loaded Modules ({}):", modules.len());
    for module in modules.iter() {
        println!("{}", module);
    }
}
