use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::exit;

// --- RISC-V Instruction Constants ---
const OP_LOAD: u32 = 0b0000011;
const OP_IMM: u32 = 0b0010011;
const OP_STORE: u32 = 0b0100011;
const OP_REG: u32 = 0b0110011;
const OP_BRANCH: u32 = 0b1100011;
const OP_JALR: u32 = 0b1100111;
const OP_JAL: u32 = 0b1101111;
const OP_SYSTEM: u32 = 0b1110011;

// Funct3/Funct7
const FUNCT3_ADD_SUB: u32 = 0b000;
const FUNCT3_MUL: u32 = 0b000;
const FUNCT3_DIV: u32 = 0b100;
const FUNCT3_LW: u32 = 0b010;
const FUNCT3_ADDI: u32 = 0b000;
const FUNCT3_AND: u32 = 0b111;
const FUNCT3_SW: u32 = 0b010;
const FUNCT3_BEQ: u32 = 0b000;
const FUNCT3_BLT: u32 = 0b100;
const FUNCT3_BNE: u32 = 0b001;
const FUNCT3_LD: u32 = 0b011;
const FUNCT3_SD: u32 = 0b011;
const FUNCT3_LB: u32 = 0b000;
const FUNCT3_SB: u32 = 0b000;
const FUNCT3_OR: u32 = 0b110;
const FUNCT3_SLT: u32 = 0b010;
const FUNCT3_SRA: u32 = 0b101;
const FUNCT3_SRL: u32 = 0b101;
const FUNCT3_XOR: u32 = 0b100;

const FUNCT7_MULDIV: u32 = 0b0000001;
const FUNCT7_ADD: u32 = 0b0000000;
const FUNCT7_SUB: u32 = 0b0100000;
const FUNCT7_AND: u32 = 0b0000000;
const FUNCT7_OR: u32 = 0b0000000;
const FUNCT7_SLT: u32 = 0b0000000;
const FUNCT7_SRA: u32 = 0b0100000;
const FUNCT7_SRL: u32 = 0b0000000;
const FUNCT7_XOR: u32 = 0b0000000;

// Custom HALT instruction
pub const OP_HALT: u32 = 0x00000000;

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

fn parse_register(reg_str: &str) -> Result<u32, &'static str> {
    let cleaned_reg = reg_str.trim_end_matches(',');
    match cleaned_reg {
        "zero" | "x0" => Ok(0),
        "ra" | "x1" => Ok(1),
        "sp" | "x2" => Ok(2),
        "gp" | "x3" => Ok(3),
        "tp" | "x4" => Ok(4),
        "t0" | "x5" => Ok(5),
        "t1" | "x6" => Ok(6),
        "t2" | "x7" => Ok(7),
        "s0" | "fp" | "x8" => Ok(8),
        "s1" | "x9" => Ok(9),
        "a0" | "x10" => Ok(10),
        "a1" | "x11" => Ok(11),
        "a2" | "x12" => Ok(12),
        "a3" | "x13" => Ok(13),
        "a4" | "x14" => Ok(14),
        "a5" | "x15" => Ok(15),
        "a6" | "x16" => Ok(16),
        "a7" | "x17" => Ok(17),
        "s2" | "x18" => Ok(18),
        "s3" | "x19" => Ok(19),
        "s4" | "x20" => Ok(20),
        "s5" | "x21" => Ok(21),
        "s6" | "x22" => Ok(22),
        "s7" | "x23" => Ok(23),
        "s8" | "x24" => Ok(24),
        "s9" | "x25" => Ok(25),
        "s10" | "x26" => Ok(26),
        "s11" | "x27" => Ok(27),
        "t3" | "x28" => Ok(28),
        "t4" | "x29" => Ok(29),
        "t5" | "x30" => Ok(30),
        "t6" | "x31" => Ok(31),
        _ => Err("Invalid register name"),
    }
}

fn parse_memory_operand(operand: &str) -> Result<(i32, u32), &'static str> {
    let open_paren = operand.find('(').ok_or("Missing '(' in memory operand")?;
    let close_paren = operand.find(')').ok_or("Missing ')' in memory operand")?;
    let offset_str = &operand[..open_paren];
    let offset = offset_str
        .parse::<i32>()
        .map_err(|_| "Invalid memory offset")?;
    let base_reg_str = &operand[open_paren + 1..close_paren];
    let base_reg = parse_register(base_reg_str)?;
    Ok((offset, base_reg))
}

// --- Encoder Functions ---
fn encode_r_type(funct7: u32, rs2: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> u32 {
    (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
}

fn encode_i_type(imm: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> u32 {
    (imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
}

fn encode_s_type(imm: u32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> u32 {
    let imm11_5 = (imm >> 5) & 0x7F;
    let imm4_0 = imm & 0x1F;
    (imm11_5 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm4_0 << 7) | opcode
}

fn encode_sb_type(imm: u32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> u32 {
    let imm12 = (imm >> 12) & 1;
    let imm11 = (imm >> 11) & 1;
    let imm10_5 = (imm >> 5) & 0x3f;
    let imm4_1 = (imm >> 1) & 0xf;
    let imm_hi = (imm12 << 6) | imm10_5;
    let imm_lo = (imm4_1 << 1) | imm11;
    (imm_hi << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm_lo << 7) | opcode
}

fn encode_u_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    (imm << 12) | (rd << 7) | opcode
}

fn encode_uj_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    let imm20 = (imm >> 20) & 1;
    let imm10_1 = (imm >> 1) & 0x3ff;
    let imm11 = (imm >> 11) & 1;
    let imm19_12 = (imm >> 12) & 0xff;
    let encoded_imm = (imm20 << 31) | (imm19_12 << 12) | (imm11 << 20) | (imm10_1 << 21);
    encoded_imm | (rd << 7) | opcode
}

pub fn parse_program(program: String) -> Vec<u8> {
    let mut symbol_table: HashMap<String, u64> = HashMap::new();
    let mut instructions = Vec::new();
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
            instructions.push(line.to_string());
            current_address += 4;
        }
    }

    let mut bin: Vec<u8> = Vec::new();
    current_address = 0;

    for line in instructions {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];

        let encoded_inst = match instruction.as_str() {
            // R-type Instructions (register-register)
            "add" | "sub" | "mul" | "div" | "and" | "or" | "slt" | "sra" | "srl" | "xor" => {
                let rd = parse_register(operands[0]).unwrap();
                let rs1 = parse_register(operands[1]).unwrap();
                let rs2 = parse_register(operands[2]).unwrap();
                let (funct7, funct3) = match instruction.as_str() {
                    "add" => (FUNCT7_ADD, FUNCT3_ADD_SUB),
                    "sub" => (FUNCT7_SUB, FUNCT3_ADD_SUB),
                    "mul" => (FUNCT7_MULDIV, FUNCT3_MUL),
                    "div" => (FUNCT7_MULDIV, FUNCT3_DIV),
                    "and" => (FUNCT7_AND, FUNCT3_AND),
                    "or" => (FUNCT7_OR, FUNCT3_OR),
                    "slt" => (FUNCT7_SLT, FUNCT3_SLT),
                    "sra" => (FUNCT7_SRA, FUNCT3_SRA),
                    "srl" => (FUNCT7_SRL, FUNCT3_SRL),
                    "xor" => (FUNCT7_XOR, FUNCT3_XOR),
                    _ => unreachable!(),
                };
                encode_r_type(funct7, rs2, rs1, funct3, rd, OP_REG)
            }

            // I-Type instructions
            "addi" | "lw" | "ld" | "lb" | "jalr" | "ret" => {
                if instruction == "ret" {
                    encode_i_type(0, 1, 0b000, 0, OP_JALR)
                } else {
                    let rd = parse_register(operands[0]).unwrap();
                    let rs1;
                    let imm;
                    let funct3;
                    let opcode;

                    match instruction.as_str() {
                        "addi" => {
                            rs1 = parse_register(operands[1]).unwrap();
                            imm = operands[2].parse::<i32>().unwrap() as u32;
                            funct3 = FUNCT3_ADDI;
                            opcode = OP_IMM;
                        }
                        "lw" | "ld" | "lb" => {
                            let (offset, base) = parse_memory_operand(operands[1]).unwrap();
                            rs1 = base;
                            imm = offset as u32;
                            opcode = OP_LOAD;
                            funct3 = match instruction.as_str() {
                                "lw" => FUNCT3_LW,
                                "ld" => FUNCT3_LD,
                                "lb" => FUNCT3_LB,
                                _ => unreachable!(),
                            };
                        }
                        "jalr" => {
                            rs1 = parse_register(operands[1]).unwrap();
                            imm = 0;
                            funct3 = 0b000;
                            opcode = OP_JALR;
                        }
                        _ => unreachable!(),
                    };

                    encode_i_type(imm, rs1, funct3, rd, opcode)
                }
            }

            // S-type Instructions
            "sw" | "sd" | "sb" => {
                let rs2 = parse_register(operands[0]).unwrap();
                let (offset, base) = parse_memory_operand(operands[1]).unwrap();
                let funct3 = match instruction.as_str() {
                    "sw" => FUNCT3_SW,
                    "sd" => FUNCT3_SD,
                    "sb" => FUNCT3_SB,
                    _ => unreachable!(),
                };
                encode_s_type(offset as u32, rs2, base, funct3, OP_STORE)
            }

            // SB-type Instructions
            "beq" | "blt" | "bne" => {
                let rs1 = parse_register(operands[0]).unwrap();
                let rs2 = parse_register(operands[1]).unwrap();
                let target_address = *symbol_table
                    .get(operands[2])
                    .expect("branch label not found");
                let offset = (target_address as i64 - current_address as i64) as u32;
                let funct3 = match instruction.as_str() {
                    "beq" => FUNCT3_BEQ,
                    "blt" => FUNCT3_BLT,
                    "bne" => FUNCT3_BNE,
                    _ => unreachable!(),
                };
                encode_sb_type(offset, rs2, rs1, funct3, OP_BRANCH)
            }

            // UJ-type Instructions
            "jal" => {
                let rd = parse_register(operands[0]).unwrap();
                let target_label = operands.get(1).unwrap_or(&"");
                let target_address = *symbol_table
                    .get(*target_label)
                    .expect("JAL label not found");
                let offset = (target_address as i64 - current_address as i64) as u32;
                encode_uj_type(offset, rd, OP_JAL)
            }

            "ecall" => encode_r_type(0, 0, 0, 0, 0, OP_SYSTEM),
            "halt" => OP_HALT,

            _ => {
                eprintln!("Unknown instruction: {}", instruction);
                exit(2);
            }
        };

        bin.extend_from_slice(&encoded_inst.to_le_bytes());
        current_address += 4;
    }
    bin
}

pub fn save_assembly(output_path: &str, data: &[u8]) -> io::Result<()> {
    fs::write(output_path, data)
}
