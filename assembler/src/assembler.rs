use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::exit;

// Opcodes for our 64-bit RISC-V-like instruction set
pub const OP_HALT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUB: u8 = 0x02;
pub const OP_ADDI: u8 = 0x03;
pub const OP_BEQ: u8 = 0x04;
pub const OP_JAL: u8 = 0x05;
pub const OP_LW: u8 = 0x06;
pub const OP_SW: u8 = 0x07;
pub const OP_RET: u8 = 0x08;
pub const OP_LDI: u8 = 0x09;
pub const OP_MUL: u8 = 0x0a;
pub const OP_DIV: u8 = 0x0b;
pub const OP_ECALL: u8 = 0xFF;

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
            arg if !arg.starts_with('-') => input_file = arg.to_string(),
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
        "zero" => Ok(0),
        "ra" => Ok(1),
        "sp" => Ok(2),
        "gp" => Ok(3),
        "tp" => Ok(4),

        "t0" => Ok(5),
        "t1" => Ok(6),
        "t2" => Ok(7),

        "s0" => Ok(8),
        "s1" => Ok(9),

        "a0" => Ok(10),
        "a1" => Ok(11),
        "a2" => Ok(12),
        "a3" => Ok(13),
        "a4" => Ok(14),
        "a5" => Ok(15),
        "a6" => Ok(16),
        "a7" => Ok(17),

        "s2" => Ok(18),
        "s3" => Ok(19),
        "s4" => Ok(20),
        "s5" => Ok(21),
        "s6" => Ok(22),
        "s7" => Ok(23),
        "s8" => Ok(24),
        "s9" => Ok(25),
        "s10" => Ok(26),
        "s11" => Ok(27),

        "t3" => Ok(28),
        "t4" => Ok(29),
        "t5" => Ok(30),
        "t6" => Ok(31),

        _ => {
            if cleaned_reg.starts_with('x') {
                match cleaned_reg[1..].parse::<u8>() {
                    Ok(num) if num < 32 => Ok(num),
                    _ => Err("Invalid register number (must be x0-x31)"),
                }
            } else {
                Err("Invalid register format")
            }
        }
    }
}

pub fn parse_program(program: String) -> Vec<u8> {
    let mut symbol_table: HashMap<String, u64> = HashMap::new();
    let mut current_address: u64 = 0;

    for line in program.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.ends_with(':') {
            let label = line.trim_end_matches(':').to_string();
            symbol_table.insert(label, current_address);
        } else {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let instruction = tokens[0].to_lowercase();
            current_address += if instruction == "li" { 12 } else { 4 };
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
            "add" | "sub" | "mul" | "div" => {
                match instruction.as_str() {
                    "add" => bin.push(OP_ADD),
                    "sub" => bin.push(OP_SUB),
                    "mul" => bin.push(OP_MUL),
                    "div" => bin.push(OP_DIV),
                    _ => {}
                }
                bin.extend_from_slice(&[
                    parse_register(operands[0]).unwrap(),
                    parse_register(operands[1]).unwrap(),
                    parse_register(operands[2]).unwrap(),
                ]);
            }
            "addi" => {
                bin.push(OP_ADDI);
                let immediate = operands[2].parse::<i8>().unwrap();
                bin.extend_from_slice(&[
                    parse_register(operands[0]).unwrap(),
                    parse_register(operands[1]).unwrap(),
                    immediate as u8,
                ]);
            }
            "li" => {
                bin.push(OP_LDI);
                let rd = parse_register(operands[0]).unwrap();
                let immediate = operands[1].parse::<u64>().unwrap();
                bin.push(rd);
                bin.extend_from_slice(&[0, 0]);
                bin.extend_from_slice(&immediate.to_le_bytes());
            }
            "beq" => {
                bin.push(OP_BEQ);
                let target_address = *symbol_table.get(operands[2]).expect("Label not found");
                let offset = target_address as i64 - current_address as i64;
                if offset > 127 || offset < -128 {
                    panic!("beq offset too large");
                }
                bin.extend_from_slice(&[
                    parse_register(operands[0]).unwrap(),
                    parse_register(operands[1]).unwrap(),
                    offset as u8,
                ]);
            }
            "jal" => {
                bin.push(OP_JAL);
                let target_address = *symbol_table.get(operands[1]).expect("Label not found");
                let offset = target_address as i64 - current_address as i64;
                if offset > 32767 || offset < -32768 {
                    panic!("jal offset too large");
                }
                bin.push(parse_register(operands[0]).unwrap());
                bin.extend_from_slice(&(offset as i16).to_le_bytes());
            }
            "sw" | "lw" => {
                bin.push(if instruction == "sw" { OP_SW } else { OP_LW });
                let (offset, base) = parse_memory_operand(operands[1]).unwrap();
                bin.extend_from_slice(&[parse_register(operands[0]).unwrap(), base, offset as u8]);
            }
            "ret" => {
                bin.push(OP_RET);
                bin.extend_from_slice(&[0, 0, 0]);
            }
            "ecall" => {
                bin.extend_from_slice(&[OP_ECALL, 0, 0, 0]);
            }
            "halt" => {
                bin.extend_from_slice(&[OP_HALT, 0, 0, 0]);
            }
            _ => {
                eprintln!("Unknown instruction: {}", instruction);
                exit(2);
            }
        }
        current_address += if instruction == "li" { 12 } else { 4 };
    }
    bin
}

pub fn save_assembly(output_path: &str, data: &[u8]) -> io::Result<()> {
    fs::write(output_path, data)
}
