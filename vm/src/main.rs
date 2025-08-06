mod virtualmachine;

use std::env;
use std::fs;
use std::process::exit;
use virtualmachine::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-bytecode-file>", args[0]);
        exit(1);
    }

    let file_path = &args[1];
    println!("Loading program from: {}", file_path);

    let program = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", file_path, e);
            exit(1);
        }
    };

    let mut vm = VM::new();

    vm.load_program(&program);

    vm.run();

    vm.print_state();
}
