use std::env;
use std::error::Error;
use std::fs;

use assembler::Executable;
use vm::VM;

const MAGIC_NUMBER: &[u8; 4] = b"RZEB";
const HEADER_SIZE: usize = 20; // 4 (magic) + 8 (text_size) + 8 (data_size)

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: vm <input.bin>");
        return Ok(());
    }

    let input_path = &args[1];

    println!("[VM] Reading binary from '{}'", input_path);
    let file_bytes = fs::read(input_path)?;

    println!("[VM] Parsing executable...");
    match parse_executable(&file_bytes) {
        Ok(executable) => {
            println!(
                "[VM] Executable loaded. .text size: {} bytes, .data size: {} bytes",
                executable.text.len(),
                executable.data.len()
            );
            let mut vm = VM::new();
            if let Err(e) = vm.load_executable(&executable) {
                eprintln!("[VM] Error loading executable into VM: {}", e);
                return Ok(());
            }

            println!("\n--- Running VM ---");
            match vm.run() {
                Ok(()) => println!("--- Execution Finished ---"),
                Err(e) => eprintln!("\n--- VM Execution Error: {} ---", e),
            }

            vm.print_state();
        }
        Err(e) => {
            eprintln!("[VM] Error parsing binary file: {}", e);
        }
    }

    Ok(())
}

fn parse_executable(bytes: &[u8]) -> Result<Executable, String> {
    if bytes.len() < HEADER_SIZE {
        return Err("Invalid file: too short to be a valid executable.".to_string());
    }

    if &bytes[0..4] != MAGIC_NUMBER {
        return Err("Invalid file: incorrect magic number.".to_string());
    }

    let text_size = u64::from_le_bytes(bytes[4..12].try_into().unwrap()) as usize;

    let data_size = u64::from_le_bytes(bytes[12..20].try_into().unwrap()) as usize;

    if bytes.len() != HEADER_SIZE + text_size + data_size {
        return Err("Invalid file: file size does not match header values.".to_string());
    }

    let text = bytes[HEADER_SIZE..HEADER_SIZE + text_size].to_vec();

    let data = bytes[HEADER_SIZE + text_size..HEADER_SIZE + text_size + data_size].to_vec();

    Ok(Executable { text, data })
}
