use assembler::Executable;
use std::env;
use std::fs;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.bin>", args[0]);
        return;
    }

    let file_path = &args[1];

    let program_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", file_path, e);
            return;
        }
    };

    let executable = Executable {
        text: program_bytes,
        data: Vec::new(),
    };

    let mut vm = VM::new();

    if let Err(e) = vm.load_executable(&executable) {
        eprintln!("Error: Failed to load executable into VM: {}", e);
        return;
    }

    vm.run();
}
