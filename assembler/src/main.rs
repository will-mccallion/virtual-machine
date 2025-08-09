use std::env;
use std::fs;
use std::process;

mod assembler;
use assembler::parse_program;

fn parse_command(args: &[String]) -> Result<(String, String), String> {
    let mut input_file = None;
    let mut output_file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err("-o flag requires an output file path.".to_string());
                }
            }
            arg if !arg.starts_with('-') => {
                if input_file.is_none() {
                    input_file = Some(arg.to_string());
                } else {
                    return Err(
                        "Multiple input files specified, but only one is allowed.".to_string()
                    );
                }
            }
            _ => { /* Ignore unknown flags */ }
        }
        i += 1;
    }

    let input = input_file.ok_or_else(|| "No input file provided.".to_string())?;
    let output =
        output_file.ok_or_else(|| "No output file provided. Use the -o flag.".to_string())?;
    Ok((input, output))
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (input_path, output_path) = match parse_command(&args[1..]) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Argument Error: {}", e);
            eprintln!("Usage: {} <input_file> -o <output_file>", args[0]);
            process::exit(1);
        }
    };

    let file_content = match fs::read_to_string(&input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    let program_binary = match parse_program(&file_content) {
        Ok(binary) => binary,
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
            process::exit(1);
        }
    };

    match fs::write(&output_path, &program_binary) {
        Ok(_) => {
            println!("Successfully assembled {} to {}", input_path, output_path);
        }
        Err(e) => {
            eprintln!("Failed to write to file '{}': {}", output_path, e);
            process::exit(1);
        }
    }
}
