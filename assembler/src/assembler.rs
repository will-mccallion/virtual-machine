use riscv_core::{funct3, funct7, opcodes};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblerErrorKind {
    InvalidRegister(String),
    InvalidMemoryOperand(String),
    InvalidImmediateValue(String),
    UndefinedLabel(String),
    UnknownInstruction(String),
}

impl fmt::Display for AssemblerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidRegister(reg) => write!(f, "Invalid register name: '{}'", reg),
            Self::InvalidMemoryOperand(op) => write!(f, "Invalid memory operand format: '{}'", op),
            Self::InvalidImmediateValue(val) => {
                write!(f, "Cannot parse immediate value: '{}'", val)
            }
            Self::UndefinedLabel(label) => write!(f, "Use of undefined label: '{}'", label),
            Self::UnknownInstruction(inst) => write!(f, "Unknown instruction: '{}'", inst),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblerError {
    pub line: usize,
    pub kind: AssemblerErrorKind,
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.line, self.kind)
    }
}

impl Error for AssemblerError {}

pub fn parse_program(program: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut symbol_table = HashMap::new();
    let mut address_counter: u64 = 0;

    for line in program.lines() {
        let clean_line = line.split('#').next().unwrap_or("").trim();
        if clean_line.is_empty() {
            continue;
        }

        if let Some(label) = clean_line.strip_suffix(':') {
            symbol_table.insert(label.to_string(), address_counter);
        } else {
            address_counter += 4;
        }
    }

    let mut bin = Vec::new();
    let mut current_address: u64 = 0;

    for (i, line) in program.lines().enumerate() {
        let line_number = i + 1;
        let clean_line = line.split('#').next().unwrap_or("").trim();

        if clean_line.is_empty() || clean_line.ends_with(':') {
            continue;
        }

        let tokens: Vec<&str> = clean_line.split_whitespace().collect();
        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];

        let encoded_inst =
            encode_instruction(&instruction, operands, current_address, &symbol_table).map_err(
                |kind| AssemblerError {
                    line: line_number,
                    kind,
                },
            )?;

        bin.extend_from_slice(&encoded_inst.to_le_bytes());
        current_address += 4;
    }

    Ok(bin)
}

fn encode_instruction(
    instruction: &str,
    operands: &[&str],
    current_address: u64,
    symbol_table: &HashMap<String, u64>,
) -> Result<u32, AssemblerErrorKind> {
    match instruction {
        // R-type
        "add" | "sub" | "mul" | "div" | "and" | "or" | "slt" | "sra" | "srl" | "xor" => {
            let rd = parse_register(operands[0])?;
            let rs1 = parse_register(operands[1])?;
            let rs2 = parse_register(operands[2])?;
            let (funct7, funct3) = match instruction {
                "add" => (funct7::ADD, funct3::ADD_SUB),
                "sub" => (funct7::SUB, funct3::ADD_SUB),
                "mul" => (funct7::MULDIV, funct3::MUL),
                "div" => (funct7::MULDIV, funct3::DIV),
                "and" => (funct7::DEFAULT, funct3::AND),
                "or" => (funct7::DEFAULT, funct3::OR),
                "slt" => (funct7::DEFAULT, funct3::SLT),
                "sra" => (funct7::SRA, funct3::SRA),
                "srl" => (funct7::DEFAULT, funct3::SRL),
                "xor" => (funct7::DEFAULT, funct3::XOR),
                _ => unreachable!(),
            };
            Ok(encode_r_type(funct7, rs2, rs1, funct3, rd, opcodes::OP_REG))
        }
        // I-type
        "addi" | "lw" | "ld" | "lb" | "lbu" | "jalr" => {
            let rd = parse_register(operands[0])?;
            let (rs1, imm, funct3, opcode) = match instruction {
                "addi" => {
                    let rs1 = parse_register(operands[1])?;
                    let imm = operands[2].parse::<i32>().map_err(|_| {
                        AssemblerErrorKind::InvalidImmediateValue(operands[2].to_string())
                    })?;
                    (rs1, imm as u32, funct3::ADDI, opcodes::OP_IMM)
                }
                "lw" | "ld" | "lb" | "lbu" => {
                    let (offset, base) = parse_memory_operand(operands[1])?;
                    let funct3 = match instruction {
                        "lw" => funct3::LW,
                        "ld" => funct3::LD,
                        "lb" => funct3::LB,
                        "lbu" => funct3::LBU,
                        _ => unreachable!(),
                    };
                    (base, offset as u32, funct3, opcodes::OP_LOAD)
                }
                "jalr" => (
                    parse_register(operands[1])?,
                    0,
                    funct3::ADD_SUB,
                    opcodes::OP_JALR,
                ),
                _ => unreachable!(),
            };
            Ok(encode_i_type(imm, rs1, funct3, rd, opcode))
        }
        "ret" => Ok(encode_i_type(0, 1, funct3::ADD_SUB, 0, opcodes::OP_JALR)),
        // S-type
        "sw" | "sd" | "sb" => {
            let rs2 = parse_register(operands[0])?;
            let (offset, base) = parse_memory_operand(operands[1])?;
            let funct3 = match instruction {
                "sw" => funct3::SW,
                "sd" => funct3::SD,
                "sb" => funct3::SB,
                _ => unreachable!(),
            };
            Ok(encode_s_type(
                offset as u32,
                rs2,
                base,
                funct3,
                opcodes::OP_STORE,
            ))
        }
        // SB-type
        "beq" | "blt" | "bne" => {
            let rs1 = parse_register(operands[0])?;
            let rs2 = parse_register(operands[1])?;
            let target_address = symbol_table
                .get(operands[2])
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(operands[2].to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            let funct3 = match instruction {
                "beq" => funct3::BEQ,
                "blt" => funct3::BLT,
                "bne" => funct3::BNE,
                _ => unreachable!(),
            };
            Ok(encode_sb_type(offset, rs2, rs1, funct3, opcodes::OP_BRANCH))
        }
        // UJ-type
        "jal" => {
            let rd = parse_register(operands[0])?;
            let target_label = operands[1];
            let target_address = symbol_table
                .get(target_label)
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(target_label.to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            Ok(encode_uj_type(offset, rd, opcodes::OP_JAL))
        }
        // System and pseudo-instructions
        "ecall" => Ok(encode_r_type(0, 0, 0, 0, 0, opcodes::OP_SYSTEM)),
        "halt" => Ok(opcodes::OP_HALT),
        _ => Err(AssemblerErrorKind::UnknownInstruction(
            instruction.to_string(),
        )),
    }
}

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
fn encode_uj_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    let imm20 = (imm >> 20) & 1;
    let imm10_1 = (imm >> 1) & 0x3ff;
    let imm11 = (imm >> 11) & 1;
    let imm19_12 = (imm >> 12) & 0xff;
    let encoded_imm = (imm20 << 31) | (imm19_12 << 12) | (imm11 << 20) | (imm10_1 << 21);
    encoded_imm | (rd << 7) | opcode
}

fn parse_register(reg_str: &str) -> Result<u32, AssemblerErrorKind> {
    match reg_str.trim_end_matches(',') {
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
        _ => Err(AssemblerErrorKind::InvalidRegister(reg_str.to_string())),
    }
}

fn parse_memory_operand(operand: &str) -> Result<(i32, u32), AssemblerErrorKind> {
    let open_paren = operand
        .find('(')
        .ok_or_else(|| AssemblerErrorKind::InvalidMemoryOperand(operand.to_string()))?;
    let close_paren = operand
        .find(')')
        .ok_or_else(|| AssemblerErrorKind::InvalidMemoryOperand(operand.to_string()))?;
    let offset_str = &operand[..open_paren];
    let offset = offset_str
        .parse::<i32>()
        .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(offset_str.to_string()))?;
    let base_reg_str = &operand[open_paren + 1..close_paren];
    let base_reg = parse_register(base_reg_str)?;
    Ok((offset, base_reg))
}
