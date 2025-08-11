use std::env;
use std::fs;

use assembler::{parse_program, Executable};

const MAGIC_NUMBER: &[u8; 4] = b"RZEB";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: assembler <input.s> <output.bin>");
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("[ASM] Reading source from '{}'", input_path);
    let source_code = fs::read_to_string(input_path)?;

    println!("[ASM] Assembling program...");
    match parse_program(&source_code) {
        Ok(executable) => {
            println!(
                "[ASM] Assembly successful. .text size: {} bytes, .data size: {} bytes",
                executable.text.len(),
                executable.data.len()
            );
            write_executable(output_path, &executable)?;
            println!("[ASM] Wrote binary to '{}'", output_path);
        }
        Err(e) => {
            eprintln!("Assembly Error: {}", e);
        }
    }

    Ok(())
}

fn write_executable(path: &str, executable: &Executable) -> Result<(), std::io::Error> {
    let mut file_contents = Vec::new();

    file_contents.extend_from_slice(MAGIC_NUMBER);

    file_contents.extend_from_slice(&(executable.text.len() as u64).to_le_bytes());

    file_contents.extend_from_slice(&(executable.data.len() as u64).to_le_bytes());

    file_contents.extend_from_slice(&executable.text);

    file_contents.extend_from_slice(&executable.data);

    fs::write(path, file_contents)
}
