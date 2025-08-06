use bincode;
use riscv_core::{Executable, SimpleElfHeader};
use std::env;
use std::fs;
use vm::VM;

const MAGIC_NUMBER: [u8; 4] = *b"RBF\n";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.o>", args[0]);
        return;
    }

    let file_path = &args[1];

    let file_bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", file_path, e);
            return;
        }
    };

    let config = bincode::config::standard();
    let (header, header_len): (SimpleElfHeader, usize) =
        match bincode::decode_from_slice(&file_bytes, config) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Error: Not a valid Rusteze executable file: {}", e);
                return;
            }
        };

    if header.magic != MAGIC_NUMBER {
        eprintln!("Error: Invalid file format. Magic number does not match.");
        return;
    }

    let text_start = header_len;
    let text_end = text_start + header.text_size as usize;
    let text_section = file_bytes[text_start..text_end].to_vec();

    let data_start = text_end;
    let data_end = data_start + header.data_size as usize;
    let data_section = file_bytes[data_start..data_end].to_vec();

    let executable = Executable {
        text: text_section,
        data: data_section,
        bss_size: header.bss_size,
        entry_point: header.entry_point,
    };

    let mut vm = VM::new();

    if let Err(e) = vm.load_executable(&executable) {
        eprintln!("Error: Failed to load executable into VM: {}", e);
        return;
    }

    if let Err(e) = vm.run() {
        eprintln!("VM Runtime Error: {}", e);
    }
}
