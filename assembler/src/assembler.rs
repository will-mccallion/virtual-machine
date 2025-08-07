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
pub const OP_JAL: u8 = 0x05;
pub const OP_LW: u8 = 0x06;
pub const OP_SW: u8 = 0x07;
pub const OP_RET: u8 = 0x08;

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

fn parse_memory_operand(operand: &str) -> Result<(i8, u8), &'static str> {
    let open_paren = operand.find('(').ok_or("Missing '(' in memory operand")?;
    let close_paren = operand.find(')').ok_or("Missing ')' in memory operand")?;

    let offset_str = &operand[..open_paren];
    let offset = offset_str
        .parse::<i8>()
        .map_err(|_| "Invalid memory offset")?;

    let base_reg_str = &operand[open_paren + 1..close_paren];
    let base_reg = parse_register(base_reg_str)?;

    Ok((offset, base_reg))
}

fn parse_register(reg_str: &str) -> Result<u8, &'static str> {
    let cleaned_reg = reg_str.trim_end_matches(',');

    match cleaned_reg {
        "zero" => return Ok(0),
        "ra" => return Ok(1),
        "sp" => return Ok(2),
        "t0" => return Ok(5),
        "t1" => return Ok(6),
        "t2" => return Ok(7),
        "s0" => return Ok(8),
        "s1" => return Ok(9),
        "a0" => return Ok(10),
        "a1" => return Ok(11),
        _ => {}
    }

    if cleaned_reg.starts_with('x') {
        match cleaned_reg[1..].parse::<u8>() {
            Ok(num) if num < 32 => return Ok(num),
            _ => return Err("Invalid register number (must be x0-x31)"),
        }
    }

    Err("Invalid register format")
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
                bin.push(if instruction == "add" { OP_ADD } else { OP_SUB });
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
                if offset > 127 || offset < -128 {
                    panic!("beq offset too large");
                }
                bin.extend_from_slice(&[rs1, rs2, offset as u8]);
            }
            "jal" => {
                bin.push(OP_JAL);
                let rd = parse_register(operands[0]).unwrap();
                let label = operands[1];
                let target_address = *symbol_table.get(label).expect("Label not found");
                let offset = target_address as i32 - current_address as i32;

                if offset < -32768 || offset > 32767 {
                    panic!("jal offset is too large for 16 bits");
                }

                let offset_bytes = (offset as i16).to_le_bytes();

                bin.extend_from_slice(&[rd, offset_bytes[0], offset_bytes[1]]);
            }
            "sw" => {
                bin.push(OP_SW);
                let rs = parse_register(operands[0]).expect("Invalid source register for sw");
                let (offset, base) =
                    parse_memory_operand(operands[1]).expect("Invalid memory operand for sw");
                bin.extend_from_slice(&[rs, base, offset as u8]);
            }
            "lw" => {
                bin.push(OP_LW);
                let rd = parse_register(operands[0]).expect("Invalid destination register for lw");
                let (offset, base) =
                    parse_memory_operand(operands[1]).expect("Invalid memory operand for lw");
                bin.extend_from_slice(&[rd, base, offset as u8]);
            }
            "ret" => {
                bin.push(OP_RET);
                bin.extend_from_slice(&[0, 0, 0]);
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
