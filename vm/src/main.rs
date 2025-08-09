use std::env;
use std::error::Error;
use std::fs;
use std::process;

// Declare the virtualmachine module, which corresponds to virtualmachine.rs
mod virtualmachine;

// Bring the VM and its error type into the current scope
use virtualmachine::{VMError, VM};

fn main() {
    // We use a separate run function to allow for the '?' operator
    // for easy error handling.
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // Return a proper error instead of exiting directly
        return Err(format!("Usage: {} <path-to-bytecode-file>", args[0]).into());
    }

    let file_path = &args[1];
    println!("Loading program from: {}", file_path);

    // The '?' operator will propagate any I/O errors
    let program = fs::read(file_path)?;

    // Create and set up the VM
    let mut vm = VM::new();
    vm.load_program(&program);

    // Run the VM and handle its result
    match vm.run() {
        Ok(_) => println!("Program finished with HALT instruction."),
        Err(VMError::Ecall) => println!("Program finished with ECALL."),
        // Any other VM error will be propagated up to main
        Err(e) => return Err(e.into()),
    }

    // Print the final state of the registers
    vm.print_state();

    Ok(())
}
