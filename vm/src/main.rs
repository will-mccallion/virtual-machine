use std::env;
use vm::{VmConfig, VM};

const BIOS_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bios.bin"));
const KERNEL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kernel.bin"));

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut trace_enabled = false;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--trace" => trace_enabled = true,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_usage(&args[0]);
                return;
            }
        }
    }

    println!("VM: Initializing...");
    let vm_config = VmConfig {
        trace: trace_enabled,
    };
    let mut vm = VM::new_config(vm_config);

    println!("VM: Loading embedded BIOS...");
    vm.load_bios(BIOS_BYTES);

    println!("VM: Loading embedded kernel into virtual disk...");
    vm.load_virtual_disk(KERNEL_BYTES.to_vec());

    println!("VM: Starting execution at reset vector...");
    println!("");
    if let Err(e) = vm.run() {
        eprintln!("\n--- VM Runtime Error ---");
        eprintln!("{}", e);
        vm.print_state();
    } else {
        println!("\n--- VM Halted ---");
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [--trace]", program_name);
}
