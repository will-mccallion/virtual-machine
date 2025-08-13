use assembler::parse_program;
use std::env;
use std::fs;
use std::io::Write;

// A basic main function to run the assembler as a command-line tool.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 4 {
        eprintln!("Usage: {} <input.s> [-o <output.bin>]", args[0]);
        return;
    }

    let input_path = &args[1];
    let output_path = if args.len() == 4 && args[2] == "-o" {
        &args[3]
    } else {
        "a.out"
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
            // In a real assembler, you'd likely write out an ELF or other object format.
            // For this example, we'll just concatenate .text and .data.
            let mut output_bytes = Vec::new();
            output_bytes.extend(&executable.text);
            output_bytes.extend(&executable.data);

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
            eprintln!("Assembly failed: {}", e);
        }
    }
}
