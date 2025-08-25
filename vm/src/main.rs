use std::env;
use std::fs;
use vm::{VmConfig, VM};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut trace_enabled = false;
    let mut bios_path = None;
    let mut kernel_path = None;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--trace" => trace_enabled = true,
            "--bios" => bios_path = iter.next(),
            "--kernel" => kernel_path = iter.next(),
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_usage(&args[0]);
                return;
            }
        }
    }

    let bios_path = match bios_path {
        Some(path) => path,
        None => {
            eprintln!("Error: BIOS file not provided.");
            print_usage(&args[0]);
            return;
        }
    };

    let kernel_path = match kernel_path {
        Some(path) => path,
        None => {
            eprintln!("Error: Kernel file not provided.");
            print_usage(&args[0]);
            return;
        }
    };

    let bios_bytes = match fs::read(bios_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read BIOS file '{}': {}", bios_path, e);
            return;
        }
    };

    let kernel_bytes = match fs::read(kernel_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read kernel file '{}': {}", kernel_path, e);
            return;
        }
    };

    println!("VM: Initializing...");
    let vm_config = VmConfig {
        trace: trace_enabled,
    };
    let mut vm = VM::new_config(vm_config);

    println!("VM: Loading BIOS from '{}'...", bios_path);
    vm.load_bios(&bios_bytes);

    println!(
        "VM: Loading kernel from '{}' into virtual disk...",
        kernel_path
    );
    vm.load_virtual_disk(kernel_bytes);

    println!("VM: Starting execution at reset vector...");
    if let Err(e) = vm.run() {
        eprintln!("\n--- VM Runtime Error ---");
        eprintln!("{}", e);
        vm.print_state();
    } else {
        println!("\n--- VM Halted ---");
    }
}

fn print_usage(program_name: &str) {
    eprintln!(
        "Usage: {} --bios <bios.bin> --kernel <kernel.bin> [--trace]",
        program_name
    );
}
