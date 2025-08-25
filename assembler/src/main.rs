use assembler::parse_program;
use bincode;
use riscv_core::SimpleElfHeader;
use std::env;
use std::fs;
use std::io::Write;

const MAGIC_NUMBER: [u8; 4] = *b"RBF\n";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input_path = None;
    let mut output_path = None;
    let mut format = "rbf";

    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-o" => {
                output_path = args_iter.next();
            }
            "--format" => {
                format = args_iter.next().map_or("rbf", |s| s.as_str());
            }
            _ => {
                if !arg.starts_with('-') {
                    input_path = Some(arg);
                }
            }
        }
    }

    let (input_path, output_path) = match (input_path, output_path) {
        (Some(i), Some(o)) => (i, o),
        _ => {
            eprintln!(
                "Usage: {} <input.s> -o <output_file> [--format <raw|rbf>]",
                args[0]
            );
            return;
        }
    };

    let source_code = match fs::read_to_string(input_path) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", input_path, e);
            return;
        }
    };

    match parse_program(&source_code) {
        Ok(executable) => {
            let mut output_bytes = Vec::new();

            if format == "raw" {
                println!("Assembling to raw binary format...");
                output_bytes.extend(&executable.text);
                output_bytes.extend(&executable.data);
            } else {
                println!("Assembling to RBF format...");
                let config = bincode::config::standard();
                let dummy_header = SimpleElfHeader {
                    magic: [0; 4],
                    entry_point: 0,
                    text_offset: 0,
                    text_size: 0,
                    data_offset: 0,
                    data_size: 0,
                    bss_size: 0,
                };
                let header_size =
                    bincode::encode_to_vec(&dummy_header, config).unwrap().len() as u64;

                let header = SimpleElfHeader {
                    magic: MAGIC_NUMBER,
                    entry_point: executable.entry_point,
                    text_offset: header_size,
                    text_size: executable.text.len() as u64,
                    data_offset: header_size + executable.text.len() as u64,
                    data_size: executable.data.len() as u64,
                    bss_size: executable.bss_size,
                };

                let header_bytes =
                    bincode::encode_to_vec(&header, config).expect("Failed to serialize header");

                output_bytes.extend(&header_bytes);
                output_bytes.extend(&executable.text);
                output_bytes.extend(&executable.data);
            }

            match fs::File::create(output_path) {
                Ok(mut f) => {
                    if let Err(e) = f.write_all(&output_bytes) {
                        eprintln!(
                            "Error: Failed to write to output file '{}': {}",
                            output_path, e
                        );
                    } else {
                        println!("Assembly successful. Output written to '{}'.", output_path);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error: Failed to create output file '{}': {}",
                        output_path, e
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Assembly failed on line {}: {:?}", e.line, e.kind);
        }
    }
}
