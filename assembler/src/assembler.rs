use std::fs;
use std::io;
use std::process::exit;

// Opcodes for RISC-V-like instruction set
pub const OP_HALT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUB: u8 = 0x02;
pub const OP_ADDI: u8 = 0x03;
//pub const OP_LOAD: u8 = 0x04;
//pub const OP_STORE: u8 = 0x05;

pub fn parse_command(program_args: &[String]) -> (String, String) {
    let mut input_file = String::new();
    let mut output_file = String::new();

    let mut i = 0;
    while i < program_args.len() {
        match program_args[i].as_str() {
            "-o" => {
                i += 1;
                if i < program_args.len() {
                    output_file = program_args[i].clone();
                } else {
                    eprintln!("Error: -o flag requires an output file path.");
                    exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                input_file = arg.to_string();
            }
            _ => { /* Ignore unknown flags */ }
        }
        i += 1;
    }

    if input_file.is_empty() {
        eprintln!("Error: No input file provided.");
        exit(1);
    }
    if output_file.is_empty() {
        eprintln!("Error: No output file provided. Use the -o flag.");
        exit(1);
    }

    (input_file, output_file)
}

pub fn read_file(input_path: &str) -> io::Result<String> {
    fs::read_to_string(input_path)
}

fn parse_register(reg_str: &str) -> Result<u8, &'static str> {
    // Remove leading 'x' and comma if present.
    let cleaned_reg = reg_str.trim_start_matches('x').trim_end_matches(',');

    match cleaned_reg.parse::<u8>() {
        Ok(num) => Ok(num),
        Err(_) => Err("Invalid register format"),
    }
}

pub fn parse_program(program: String) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();

    for line in program.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];

        match instruction.as_str() {
            "add" | "sub" => {
                // Example format: add rd, rs1, rs2
                if operands.len() != 3 {
                    eprintln!(
                        "Error: '{}' expects 3 operands, but got {}",
                        instruction,
                        operands.len()
                    );
                    exit(2);
                }

                if instruction == "add" {
                    bin.push(OP_ADD);
                } else {
                    bin.push(OP_SUB);
                }

                let rd = parse_register(operands[0]).expect("Invalid destination register");
                let rs1 = parse_register(operands[1]).expect("Invalid source register 1");
                let rs2 = parse_register(operands[2]).expect("Invalid source register 2");

                bin.push(rd);
                bin.push(rs1);
                bin.push(rs2);
            }
            "addi" => {
                // Example format: addi rd, rs1, immediate
                if operands.len() != 3 {
                    eprintln!(
                        "Error: 'addi' expects 3 operands, but got {}",
                        operands.len()
                    );
                    exit(2);
                }

                bin.push(OP_ADDI);

                let rd = parse_register(operands[0]).expect("Invalid destination register");
                let rs1 = parse_register(operands[1]).expect("Invalid source register");

                let immediate = operands[2].parse::<u8>().expect("Invalid immediate value");

                bin.push(rd);
                bin.push(rs1);
                bin.push(immediate);
            }
            "halt" => {
                bin.push(OP_HALT);
            }
            _ => {
                eprintln!("Error: Unknown instruction '{}'", instruction);
                exit(2);
            }
        }
    }

    bin
}

pub fn save_assembly(output_path: &str, data: &[u8]) -> io::Result<()> {
    fs::write(output_path, data)
}
