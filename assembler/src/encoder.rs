use crate::types::{AssemblerErrorKind, BASE_ADDRESS};
use riscv_core::{funct3, funct7, opcodes, system};
use std::collections::HashMap;

fn parse_immediate(imm_str: &str) -> Result<i64, AssemblerErrorKind> {
    let s = imm_str.trim_end_matches(',');
    if s.starts_with("0x") {
        i64::from_str_radix(&s[2..], 16)
            .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(s.to_string()))
    } else if s.starts_with("0b") {
        i64::from_str_radix(&s[2..], 2)
            .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(s.to_string()))
    } else {
        s.parse::<i64>()
            .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(s.to_string()))
    }
}

pub(crate) fn encode_instruction(
    instruction: &str,
    operands: &[&str],
    current_address: u64,
    text_labels: &HashMap<String, u64>,
    data_labels: &HashMap<String, u64>,
    bss_labels: &HashMap<String, u64>,
    text_size: u64,
    data_size: u64,
) -> Result<Vec<u32>, AssemblerErrorKind> {
    let single_instr = match instruction {
        "nop" => Ok(encode_i_type(0, 0, funct3::ADD_SUB, 0, opcodes::OP_IMM)),
        // R-type
        "add" | "sub" | "sll" | "slt" | "sltu" | "xor" | "srl" | "sra" | "or" | "and" | "addw"
        | "subw" | "sllw" | "srlw" | "sraw" | "mul" | "mulh" | "mulhsu" | "mulhu" | "mulw"
        | "div" | "divu" | "divw" | "divuw" | "rem" | "remu" | "remw" | "remuw" => {
            let rd = parse_register(operands[0])?;
            let rs1 = parse_register(operands[1])?;
            let rs2 = parse_register(operands[2])?;
            let (funct7, funct3, opcode) = match instruction {
                "add" => (funct7::DEFAULT, funct3::ADD_SUB, opcodes::OP_REG),
                "sub" => (funct7::SUB, funct3::ADD_SUB, opcodes::OP_REG),
                "sll" => (funct7::DEFAULT, funct3::SLL, opcodes::OP_REG),
                "slt" => (funct7::DEFAULT, funct3::SLT, opcodes::OP_REG),
                "sltu" => (funct7::DEFAULT, funct3::SLTU, opcodes::OP_REG),
                "xor" => (funct7::DEFAULT, funct3::XOR, opcodes::OP_REG),
                "srl" => (funct7::DEFAULT, funct3::SRL_SRA, opcodes::OP_REG),
                "sra" => (funct7::SRA, funct3::SRL_SRA, opcodes::OP_REG),
                "or" => (funct7::DEFAULT, funct3::OR, opcodes::OP_REG),
                "and" => (funct7::DEFAULT, funct3::AND, opcodes::OP_REG),

                "addw" => (funct7::DEFAULT, funct3::ADD_SUB, opcodes::OP_REG_32),
                "subw" => (funct7::SUB, funct3::ADD_SUB, opcodes::OP_REG_32),
                "sllw" => (funct7::DEFAULT, funct3::SLL, opcodes::OP_REG_32),
                "srlw" => (funct7::DEFAULT, funct3::SRL_SRA, opcodes::OP_REG_32),
                "sraw" => (funct7::SRA, funct3::SRL_SRA, opcodes::OP_REG_32),

                "mul" => (funct7::MULDIV, funct3::MUL, opcodes::OP_REG),
                "mulh" => (funct7::MULDIV, funct3::MULH, opcodes::OP_REG),
                "mulhsu" => (funct7::MULDIV, funct3::MULHSU, opcodes::OP_REG),
                "mulhu" => (funct7::MULDIV, funct3::MULHU, opcodes::OP_REG),

                "mulw" => (funct7::MULDIV, funct3::MUL, opcodes::OP_REG_32),
                "divw" => (funct7::MULDIV, funct3::DIV, opcodes::OP_REG_32),
                "divuw" => (funct7::MULDIV, funct3::DIVU, opcodes::OP_REG_32),
                "remw" => (funct7::MULDIV, funct3::REM, opcodes::OP_REG_32),
                "remuw" => (funct7::MULDIV, funct3::REMU, opcodes::OP_REG_32),

                "div" => (funct7::MULDIV, funct3::DIV, opcodes::OP_REG),
                "divu" => (funct7::MULDIV, funct3::DIVU, opcodes::OP_REG),
                "rem" => (funct7::MULDIV, funct3::REM, opcodes::OP_REG),
                "remu" => (funct7::MULDIV, funct3::REMU, opcodes::OP_REG),

                _ => unreachable!(),
            };
            Ok(encode_r_type(funct7, rs2, rs1, funct3, rd, opcode))
        }
        // I-type
        "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi" | "slli" | "srli" | "srai"
        | "addiw" | "slliw" | "srliw" | "sraiw" | "lb" | "lh" | "lw" | "ld" | "lbu" | "lhu"
        | "lwu" | "jalr" => {
            let rd = parse_register(operands[0])?;
            let (rs1, imm, funct3, opcode) = match instruction {
                "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi" | "addiw" => {
                    let rs1 = parse_register(operands[1])?;
                    let imm = parse_immediate(operands[2])? as i32;
                    let (funct3, opcode) = match instruction {
                        "addi" => (funct3::ADD_SUB, opcodes::OP_IMM),
                        "slti" => (funct3::SLT, opcodes::OP_IMM),
                        "sltiu" => (funct3::SLTU, opcodes::OP_IMM),
                        "xori" => (funct3::XOR, opcodes::OP_IMM),
                        "ori" => (funct3::OR, opcodes::OP_IMM),
                        "andi" => (funct3::AND, opcodes::OP_IMM),
                        "addiw" => (funct3::ADD_SUB, opcodes::OP_IMM_32),
                        _ => unreachable!(),
                    };
                    (rs1, imm as u32, funct3, opcode)
                }
                "slli" | "srli" | "srai" => {
                    let rs1 = parse_register(operands[1])?;
                    let shamt = parse_immediate(operands[2])? as u32 & 0x3F;
                    let funct6 = match instruction {
                        "slli" => 0b000000,
                        "srli" => 0b000000,
                        "srai" => 0b010000,
                        _ => unreachable!(),
                    };
                    let imm = (funct6 << 6) | shamt;
                    let funct3 = match instruction {
                        "slli" => funct3::SLL,
                        "srli" | "srai" => funct3::SRL_SRA,
                        _ => unreachable!(),
                    };
                    (rs1, imm, funct3, opcodes::OP_IMM)
                }
                "slliw" | "srliw" | "sraiw" => {
                    let rs1 = parse_register(operands[1])?;
                    let shamt = parse_immediate(operands[2])? as u32 & 0x1F;
                    let funct7 = if instruction == "sraiw" {
                        funct7::SRA
                    } else {
                        funct7::DEFAULT
                    };
                    let imm = (funct7 << 5) | shamt;
                    let funct3 = match instruction {
                        "slliw" => funct3::SLL,
                        "srliw" | "sraiw" => funct3::SRL_SRA,
                        _ => unreachable!(),
                    };
                    (rs1, imm, funct3, opcodes::OP_IMM_32)
                }
                "lb" | "lh" | "lw" | "ld" | "lbu" | "lhu" | "lwu" => {
                    let (offset, base) = parse_memory_operand(operands[1])?;
                    let funct3 = match instruction {
                        "lb" => funct3::LB,
                        "lh" => funct3::LH,
                        "lw" => funct3::LW,
                        "ld" => funct3::LD,
                        "lbu" => funct3::LBU,
                        "lhu" => funct3::LHU,
                        "lwu" => funct3::LWU,
                        _ => unreachable!(),
                    };
                    (base, offset as u32, funct3, opcodes::OP_LOAD)
                }
                "jalr" => {
                    let (rs1, imm) = if operands.len() == 1 {
                        (parse_register(operands[0])?, 0)
                    } else if operands[1].contains('(') {
                        let (offset, base) = parse_memory_operand(operands[1])?;
                        (base, offset as u32)
                    } else {
                        (parse_register(operands[1])?, 0)
                    };
                    (rs1, imm, funct3::ADD_SUB, opcodes::OP_JALR)
                }
                _ => unreachable!(),
            };
            Ok(encode_i_type(imm, rs1, funct3, rd, opcode))
        }
        "ret" => Ok(encode_i_type(
            0,
            riscv_core::abi::RA,
            funct3::ADD_SUB,
            riscv_core::abi::ZERO,
            opcodes::OP_JALR,
        )),
        // S-type
        "sb" | "sh" | "sw" | "sd" => {
            let rs2 = parse_register(operands[0])?;
            let (offset, base) = parse_memory_operand(operands[1])?;
            let funct3 = match instruction {
                "sb" => funct3::SB,
                "sh" => funct3::SH,
                "sw" => funct3::SW,
                "sd" => funct3::SD,
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
        // B-type
        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" => {
            let rs1 = parse_register(operands[0])?;
            let rs2 = parse_register(operands[1])?;
            let target_address = text_labels
                .get(operands[2])
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(operands[2].to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            let funct3 = match instruction {
                "beq" => funct3::BEQ,
                "bne" => funct3::BNE,
                "blt" => funct3::BLT,
                "bge" => funct3::BGE,
                "bltu" => funct3::BLTU,
                "bgeu" => funct3::BGEU,
                _ => unreachable!(),
            };
            Ok(encode_b_type(offset, rs2, rs1, funct3, opcodes::OP_BRANCH))
        }
        // U-type
        "lui" | "auipc" => {
            let rd = parse_register(operands[0])?;
            let imm = parse_immediate(operands[1])? as u32;
            let opcode = if instruction == "lui" {
                opcodes::OP_LUI
            } else {
                opcodes::OP_AUIPC
            };
            Ok(encode_u_type(imm << 12, rd, opcode))
        }
        // J-type
        "jal" => {
            let rd = parse_register(operands[0])?;
            let target_label = operands[1];
            let target_address = text_labels
                .get(target_label)
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(target_label.to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            Ok(encode_j_type(offset, rd, opcodes::OP_JAL))
        }
        "j" => {
            let target_label = operands[0];
            let target_address = text_labels
                .get(target_label)
                .ok_or_else(|| AssemblerErrorKind::UndefinedLabel(target_label.to_string()))?;
            let offset = (*target_address as i64 - current_address as i64) as u32;
            Ok(encode_j_type(offset, 0, opcodes::OP_JAL))
        }

        "li" => {
            let rd = parse_register(operands[0])?;
            let imm = parse_immediate(operands[1])?;

            if imm >= -2048 && imm <= 2047 {
                let addi = encode_i_type(imm as u32, 0, funct3::ADD_SUB, rd, opcodes::OP_IMM);
                return Ok(vec![addi]);
            }

            return Err(AssemblerErrorKind::ValueOutOfRange(imm.to_string()));
        }
        "la" => {
            let rd = parse_register(operands[0])?;
            let label = operands[1];

            let target_address = if let Some(addr_offset) = text_labels.get(label) {
                BASE_ADDRESS + addr_offset
            } else if let Some(addr_offset) = data_labels.get(label) {
                BASE_ADDRESS + text_size + addr_offset
            } else if let Some(addr_offset) = bss_labels.get(label) {
                BASE_ADDRESS + text_size + data_size + addr_offset
            } else {
                return Err(AssemblerErrorKind::UndefinedLabel(label.to_string()));
            };

            let current_pc = BASE_ADDRESS + current_address;
            let offset = target_address as i64 - current_pc as i64;

            let upper = (offset + 0x800) as u32 & 0xFFFFF000;
            let lower = (offset - upper as i64) as u32;

            let auipc = encode_u_type(upper, rd, opcodes::OP_AUIPC);
            let addi = encode_i_type(lower, rd, funct3::ADD_SUB, rd, opcodes::OP_IMM);

            return Ok(vec![auipc, addi]);
        }
        // System
        "ecall" => Ok(encode_i_type(
            system::FUNCT12_ECALL,
            0,
            0,
            0,
            opcodes::OP_SYSTEM,
        )),
        "ebreak" => Ok(encode_i_type(
            system::FUNCT12_EBREAK,
            0,
            0,
            0,
            opcodes::OP_SYSTEM,
        )),
        "mret" => Ok(encode_i_type(
            system::FUNCT12_MRET,
            0,
            0,
            0,
            opcodes::OP_SYSTEM,
        )),
        "sret" => Ok(encode_i_type(
            system::FUNCT12_SRET,
            0,
            0,
            0,
            opcodes::OP_SYSTEM,
        )),
        "fence" => {
            let pred = 0b1000;
            let succ = 0b1000;
            let imm = (pred << 4) | succ;
            Ok(encode_i_type(
                imm,
                0,
                funct3::FENCE,
                0,
                opcodes::OP_MISC_MEM,
            ))
        }
        "fence.i" => Ok(encode_i_type(
            0,
            0,
            funct3::FENCE_I,
            0,
            opcodes::OP_MISC_MEM,
        )),
        "csrrw" | "csrrs" | "csrrc" | "csrrwi" | "csrrsi" | "csrrci" => {
            let rd = parse_register(operands[0])?;
            let csr = parse_csr(operands[1])?;
            let is_imm = instruction.ends_with('i');

            let rs1_or_zimm = if is_imm {
                parse_immediate(operands[2])? as u32
            } else {
                parse_register(operands[2])?
            };

            let funct3 = match instruction {
                "csrrw" => funct3::CSRRW,
                "csrrs" => funct3::CSRRS,
                "csrrc" => funct3::CSRRC,
                "csrrwi" => funct3::CSRRWI,
                "csrrsi" => funct3::CSRRSI,
                "csrrci" => funct3::CSRRCI,
                _ => unreachable!(),
            };

            let rs1_field = if is_imm {
                rs1_or_zimm & 0x1F
            } else {
                rs1_or_zimm
            };

            Ok((csr << 20) | (rs1_field << 15) | (funct3 << 12) | (rd << 7) | opcodes::OP_SYSTEM)
        }
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

fn encode_b_type(imm: u32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> u32 {
    let imm12 = (imm >> 12) & 1;
    let imm11 = (imm >> 11) & 1;
    let imm10_5 = (imm >> 5) & 0x3f;
    let imm4_1 = (imm >> 1) & 0xf;
    let imm_hi = (imm12 << 6) | imm10_5;
    let imm_lo = (imm4_1 << 1) | imm11;
    (imm_hi << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm_lo << 7) | opcode
}

fn encode_u_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    (imm & 0xFFFFF000) | (rd << 7) | opcode
}

fn encode_j_type(imm: u32, rd: u32, opcode: u32) -> u32 {
    let imm20 = (imm >> 20) & 1;
    let imm10_1 = (imm >> 1) & 0x3ff;
    let imm11 = (imm >> 11) & 1;
    let imm19_12 = (imm >> 12) & 0xff;
    let encoded_imm = (imm20 << 19) | (imm19_12) | (imm11 << 8) | (imm10_1 << 9);
    (encoded_imm << 12) | (rd << 7) | opcode
}

fn parse_csr(csr_str: &str) -> Result<u32, AssemblerErrorKind> {
    let s = csr_str.trim_end_matches(',');

    if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16)
            .map_err(|_| AssemblerErrorKind::InvalidImmediateValue(s.to_string()))
    } else {
        match s {
            // User Trap Setup
            "ustatus" => Ok(0x000),
            "uie" => Ok(0x004),
            "utvec" => Ok(0x005),
            // User Trap Handling
            "uscratch" => Ok(0x040),
            "uepc" => Ok(0x041),
            "ucause" => Ok(0x042),
            "utval" => Ok(0x043),
            "uip" => Ok(0x044),
            // Supervisor Trap Setup
            "sstatus" => Ok(0x100),
            "sedeleg" => Ok(0x102),
            "sideleg" => Ok(0x103),
            "sie" => Ok(0x104),
            "stvec" => Ok(0x105),
            "scounteren" => Ok(0x106),
            // Supervisor Trap Handling
            "sscratch" => Ok(0x140),
            "sepc" => Ok(0x141),
            "scause" => Ok(0x142),
            "stval" => Ok(0x143),
            "sip" => Ok(0x144),
            // Supervisor Address Translation and Protection
            "satp" => Ok(0x180),
            // Machine Information Registers
            "mvendorid" => Ok(0xF11),
            "marchid" => Ok(0xF12),
            "mimpid" => Ok(0xF13),
            "mhartid" => Ok(0xF14),
            // Machine Trap Setup
            "mstatus" => Ok(riscv_core::csr::MSTATUS),
            "misa" => Ok(0x301),
            "medeleg" => Ok(0x302),
            "mideleg" => Ok(0x303),
            "mie" => Ok(riscv_core::csr::MIE),
            "mtvec" => Ok(riscv_core::csr::MTVEC),
            "mcounteren" => Ok(0x306),
            // Machine Trap Handling
            "mscratch" => Ok(riscv_core::csr::MSCRATCH),
            "mepc" => Ok(riscv_core::csr::MEPC),
            "mcause" => Ok(riscv_core::csr::MCAUSE),
            "mtval" => Ok(riscv_core::csr::MTVAL),
            "mip" => Ok(riscv_core::csr::MIP),
            _ => Err(AssemblerErrorKind::InvalidImmediateValue(s.to_string())),
        }
    }
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
        0
    } else {
        parse_immediate(offset_str)? as i32
    };
    let base_reg = parse_register(parts[1])?;
    Ok((offset, base_reg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use riscv_core::abi;

    fn empty_labels() -> (
        HashMap<String, u64>,
        HashMap<String, u64>,
        HashMap<String, u64>,
    ) {
        (HashMap::new(), HashMap::new(), HashMap::new())
    }

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("zero").unwrap(), abi::ZERO);
        assert_eq!(parse_register("ra,").unwrap(), abi::RA);
        assert!(parse_register("x32").is_err());
    }

    #[test]
    fn test_parse_memory_operand() {
        assert_eq!(parse_memory_operand("16(sp)").unwrap(), (16, abi::SP));
        assert_eq!(parse_memory_operand("-4(s0)").unwrap(), (-4, abi::S0));
        assert_eq!(parse_memory_operand("(ra)").unwrap(), (0, abi::RA));
    }

    #[test]
    fn test_encode_r_type_instructions() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["a0", "a1", "a2"];
        let result = encode_instruction("add", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0x00c58533]);
    }

    #[test]
    fn test_encode_i_type_instructions() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["a0", "a1", "-10"];
        let result = encode_instruction("addi", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0xff658513]);
    }

    #[test]
    fn test_encode_s_type_instructions() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["a1", "32(s0)"];
        let result = encode_instruction("sw", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0x02b42023]);
    }

    #[test]
    fn test_encode_b_type_with_labels() {
        let mut text_labels = HashMap::new();
        text_labels.insert("loop".to_string(), 12);
        let operands = vec!["a0", "a1", "loop"];
        let result = encode_instruction(
            "beq",
            &operands,
            4,
            &text_labels,
            &HashMap::new(),
            &HashMap::new(),
            0,
            0,
        )
        .unwrap();
        assert_eq!(result, vec![0x00b50463]);
    }

    #[test]
    fn test_encode_j_type_with_labels() {
        let mut text_labels = HashMap::new();
        text_labels.insert("target".to_string(), 100);
        let operands = vec!["ra", "target"];
        let result = encode_instruction(
            "jal",
            &operands,
            20,
            &text_labels,
            &HashMap::new(),
            &HashMap::new(),
            0,
            0,
        )
        .unwrap();
        assert_eq!(result, vec![0x050000ef]);
    }

    #[test]
    fn test_encode_u_type_instructions() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["a0", "0xABCDE"];
        let result = encode_instruction("lui", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0xabcde537]);
    }

    #[test]
    fn test_encode_la_pseudo_instruction() {
        let text_labels = HashMap::new();
        let mut data_labels = HashMap::new();
        let bss_labels = HashMap::new();
        let text_size = 128;
        let data_size = 32;
        data_labels.insert("my_data".to_string(), 16);

        let operands = vec!["a0", "my_data"];
        let result = encode_instruction(
            "la",
            &operands,
            8,
            &text_labels,
            &data_labels,
            &bss_labels,
            text_size,
            data_size,
        )
        .unwrap();
        assert_eq!(result, vec![0x00000517, 0x08850513]);
    }

    #[test]
    fn test_system_call_instructions() {
        let (tl, dl, bl) = empty_labels();
        let result = encode_instruction("ecall", &[], 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0x00000073]);
    }

    #[test]
    fn test_csr_instructions() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["zero", "mepc", "a0"];
        let result = encode_instruction("csrrw", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap();
        assert_eq!(result, vec![0x34151073]);
    }

    #[test]
    fn test_unknown_instruction_error() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["a0"];
        let error = encode_instruction("fly", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap_err();
        assert_eq!(
            error,
            AssemblerErrorKind::UnknownInstruction("fly".to_string())
        );
    }

    #[test]
    fn test_undefined_label_error() {
        let (tl, dl, bl) = empty_labels();
        let operands = vec!["ra", "nonexistent_label"];
        let error = encode_instruction("jal", &operands, 0, &tl, &dl, &bl, 0, 0).unwrap_err();
        assert_eq!(
            error,
            AssemblerErrorKind::UndefinedLabel("nonexistent_label".to_string())
        );
    }
}
