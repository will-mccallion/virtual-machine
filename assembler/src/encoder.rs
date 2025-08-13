use crate::types::{AssemblerErrorKind, BASE_ADDRESS};
use riscv_core::{funct3, funct7, opcodes, system};
use std::collections::HashMap;

pub(crate) fn encode_instruction(
    instruction: &str,
    operands: &[&str],
    current_address: u64,
    text_labels: &HashMap<String, u64>,
    data_labels: &HashMap<String, u64>,
    text_size: u64,
) -> Result<Vec<u32>, AssemblerErrorKind> {
    // PRIO 3: # TODO: Implement more common pseudo-instructions: `li` (load immediate), `mv` (move register), `j` (unconditional jump), `call`, and `nop`.
    let single_instr = match instruction {
        // R-type
        // PRIO 2: # TODO: Add assembler support for remaining RV64I R-type instructions: SLL, SLTU, ADDW, SUBW, SLLW, SRLW, SRAW.
        // PRIO 4: # TODO: Add support for the 'M' extension R-type instructions: MULH, MULHSU, MULHU, DIVU, REM, REMU, DIVW, REMW.
        "add" | "sub" | "mul" | "div" | "and" | "or" | "slt" | "sra" | "srl" | "xor" => {
            let rd = parse_register(operands[0])?;
            let rs1 = parse_register(operands[1])?;
            let rs2 = parse_register(operands[2])?;
            let (funct7, funct3) = match instruction {
                "add" => (funct7::DEFAULT, funct3::ADD_SUB),
                "sub" => (funct7::SUB, funct3::ADD_SUB),
                "mul" => (funct7::MULDIV, funct3::MUL),
                "div" => (funct7::MULDIV, funct3::DIV),
                "and" => (funct7::DEFAULT, funct3::AND),
                "or" => (funct7::DEFAULT, funct3::OR),
                "slt" => (funct7::DEFAULT, funct3::SLT),
                "sra" => (funct7::SRA, funct3::SRL_SRA),
                "srl" => (funct7::DEFAULT, funct3::SRL_SRA),
                "xor" => (funct7::DEFAULT, funct3::XOR),
                _ => unreachable!(),
            };
            Ok(encode_r_type(funct7, rs2, rs1, funct3, rd, opcodes::OP_REG))
        }
        // I-type
        // PRIO 2: # TODO: Add assembler support for other I-type instructions: SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI, and the 64-bit variants (ADDIW, SLLIW, SRLIW, SRAIW).
        "addi" | "lw" | "ld" | "lb" | "lbu" | "jalr" => {
            let rd = parse_register(operands[0])?;
            let (rs1, imm, funct3, opcode) = match instruction {
                "addi" => {
                    let rs1 = parse_register(operands[1])?;
                    let imm = operands[2].parse::<i32>().map_err(|_| {
                        AssemblerErrorKind::InvalidImmediateValue(operands[2].to_string())
                    })?;
                    (rs1, imm as u32, funct3::ADD_SUB, opcodes::OP_IMM)
                }
                "lw" | "ld" | "lb" | "lbu" => {
                    // PRIO 2: # TODO: Add support for LH and LHU load instructions.
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
                    0, // PRIO 3: # TODO: JALR can have an immediate offset. This should be parsed from the memory operand format, e.g., `jalr ra, 16(sp)`.
                    funct3::ADD_SUB,
                    opcodes::OP_JALR,
                ),
                _ => unreachable!(),
            };
            Ok(encode_i_type(imm, rs1, funct3, rd, opcode))
        }
        "ret" => Ok(encode_i_type(0, 1, funct3::ADD_SUB, 0, opcodes::OP_JALR)),
        // S-type
        // PRIO 2: # TODO: Add assembler support for `sh` (store halfword).
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
        // PRIO 2: # TODO: Add assembler support for other standard branch instructions: BGE, BGEU, and BLTU.
        "beq" | "blt" | "bne" => {
            let rs1 = parse_register(operands[0])?;
            let rs2 = parse_register(operands[1])?;
            let target_address = text_labels
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
            let target_address = text_labels
                .get(target_label)
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(target_label.to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            Ok(encode_uj_type(offset, rd, opcodes::OP_JAL))
        }
        "la" => {
            let rd = parse_register(operands[0])?;
            let label = operands[1];

            let target_address = if let Some(addr_offset) = data_labels.get(label) {
                BASE_ADDRESS + text_size + addr_offset
            } else if let Some(addr_offset) = text_labels.get(label) {
                BASE_ADDRESS + addr_offset
            } else {
                return Err(AssemblerErrorKind::UndefinedLabel(label.to_string()));
            };

            let current_pc = BASE_ADDRESS + current_address;
            let offset = target_address as i64 - current_pc as i64;

            // PRIO 5: # TODO: The calculation for the AUIPC+ADDI pair needs to be precise. The standard algorithm involves adding 0x800 to handle rounding correctly before extracting the upper 20 bits.
            let upper = (offset + 0x800) as u32 & 0xFFFFF000;
            let lower = (offset - upper as i64) as u32;

            let auipc = encode_u_type(upper, rd, opcodes::OP_AUIPC);
            let addi = encode_i_type(lower, rd, funct3::ADD_SUB, rd, opcodes::OP_IMM);

            return Ok(vec![auipc, addi]);
        }
        // PRIO 4: # TODO: Add support for `ebreak` for debuggers.
        "ecall" => Ok(encode_i_type(
            system::FUNCT12_ECALL,
            0,
            0,
            0,
            opcodes::OP_SYSTEM,
        )),
        _ => Err(AssemblerErrorKind::UnknownInstruction(
            instruction.to_string(),
        )),
    }?;
    Ok(vec![single_instr])
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

fn encode_u_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    imm | (rd << 7) | opcode
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
    if !operand.ends_with(')') {
        return Err(AssemblerErrorKind::InvalidMemoryOperand(
            operand.to_string(),
        ));
    }
    let parts: Vec<&str> = operand[..operand.len() - 1].split('(').collect();
    if parts.len() != 2 {
        return Err(AssemblerErrorKind::InvalidMemoryOperand(
            operand.to_string(),
        ));
    }
    let offset_str = parts[0];
    let offset = if offset_str.is_empty() {
        // PRIO 2: # FIX: This parsing for `offset(base)` is not robust. It should handle optional whitespace and correctly parse negative offsets.
        0
    } else {
        offset_str
            .parse::<i32>()
            .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(offset_str.to_string()))?
    };
    let base_reg = parse_register(parts[1])?;
    Ok((offset, base_reg))
}
