use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::exit;

// Opcodes for RISC-V-like instruction set
pub const OP_HALT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUB: u8 = 0x02;
pub const OP_ADDI: u8 = 0x03;
pub const OP_BEQ: u8 = 0x04;

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
    let cleaned_reg = reg_str.trim_start_matches('x').trim_end_matches(',');
    match cleaned_reg.parse::<u8>() {
        Ok(num) => Ok(num),
        Err(_) => Err("Invalid register format"),
    }
}

pub fn parse_program(program: String) -> Vec<u8> {
    let mut symbol_table: HashMap<String, u32> = HashMap::new();
    let mut current_address: u32 = 0;
    for line in program.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.ends_with(':') {
            let label = line.trim_end_matches(':').to_string();
            symbol_table.insert(label, current_address);
        } else {
            current_address += 4;
        }
    }

    let mut bin: Vec<u8> = Vec::new();
    current_address = 0;
    for line in program.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.ends_with(':') {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];
        match instruction.as_str() {
            "add" | "sub" => {
                if instruction == "add" {
                    bin.push(OP_ADD);
                } else {
                    bin.push(OP_SUB);
                }
                let rd = parse_register(operands[0]).unwrap();
                let rs1 = parse_register(operands[1]).unwrap();
                let rs2 = parse_register(operands[2]).unwrap();
                bin.extend_from_slice(&[rd, rs1, rs2]);
            }
            "addi" => {
                bin.push(OP_ADDI);
                let rd = parse_register(operands[0]).unwrap();
                let rs1 = parse_register(operands[1]).unwrap();
                let immediate = operands[2].parse::<i8>().unwrap();
                bin.extend_from_slice(&[rd, rs1, immediate as u8]);
            }
            "beq" => {
                bin.push(OP_BEQ);
                let rs1 = parse_register(operands[0]).unwrap();
                let rs2 = parse_register(operands[1]).unwrap();
                let label = operands[2];
                let target_address = *symbol_table.get(label).expect("Label not found");
                let offset = target_address as i32 - current_address as i32;
                if offset < -128 || offset > 127 {
                    panic!("Branch offset too large!");
                }
                bin.push(rs1);
                bin.push(rs2);
                bin.push(offset as i8 as u8);
            }
            "halt" => {
                bin.extend_from_slice(&[OP_HALT, 0, 0, 0]);
            }
            _ => {
                eprintln!("Unknown instruction: {}", instruction);
                exit(2);
            }
        }
        current_address += 4;
    }
    bin
}

pub fn save_assembly(output_path: &str, data: &[u8]) -> io::Result<()> {
    fs::write(output_path, data)
}
