use riscv_core::{funct3, funct7, opcodes, system};

fn abi_to_string(reg: u32) -> String {
    match reg {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => "unknown",
    }
    .to_string()
}

fn csr_to_string(csr: u32) -> String {
    let csr_str = match csr {
        0x000 => "ustatus",
        0x004 => "uie",
        0x005 => "utvec",
        0x040 => "uscratch",
        0x041 => "uepc",
        0x042 => "ucause",
        0x043 => "utval",
        0x044 => "uip",
        0x100 => "sstatus",
        0x102 => "sedeleg",
        0x103 => "sideleg",
        0x104 => "sie",
        0x105 => "stvec",
        0x106 => "scounteren",
        0x140 => "sscratch",
        0x141 => "sepc",
        0x142 => "scause",
        0x143 => "stval",
        0x144 => "sip",
        0x180 => "satp",
        0xF11 => "mvendorid",
        0xF12 => "marchid",
        0xF13 => "mimpid",
        0xF14 => "mhartid",
        0x300 => "mstatus",
        0x301 => "misa",
        0x302 => "medeleg",
        0x303 => "mideleg",
        0x304 => "mie",
        0x305 => "mtvec",
        0x306 => "mcounteren",
        0x340 => "mscratch",
        0x341 => "mepc",
        0x342 => "mcause",
        0x343 => "mtval",
        0x344 => "mip",
        _ => "extra",
    }
    .to_string();

    csr_str
}

pub fn disassemble(word: u32, pc: u64) -> String {
    let opcode = word & 0x7f;
    let rd = (word >> 7) & 0x1f;
    let rs1 = (word >> 15) & 0x1f;
    let rs2 = (word >> 20) & 0x1f;
    let funct3 = (word >> 12) & 0x7;
    let funct7 = (word >> 25) & 0x7f;

    let rd_str = abi_to_string(rd);
    let rs1_str = abi_to_string(rs1);
    let rs2_str = abi_to_string(rs2);

    match opcode {
        opcodes::OP_LUI => format!("lui {}, {:#x}", rd_str, (word & 0xfffff000) as i32 >> 12),

        opcodes::OP_AUIPC => {
            let imm = (word & 0xfffff000) as i32 >> 12;
            format!("auipc {}, {:#x}", rd_str, imm)
        }

        opcodes::OP_JAL => {
            let imm20 = (word >> 31) & 1;
            let imm10_1 = (word >> 21) & 0x3ff;
            let imm11 = (word >> 20) & 1;
            let imm19_12 = (word >> 12) & 0xff;

            let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);

            let signed_imm = ((imm as i32) << 11) >> 11;

            let target = pc.wrapping_add(signed_imm as i64 as u64);
            if rd == 0 {
                format!("j {:#x}", target)
            } else {
                format!("jal {}, {:#x}", rd_str, target)
            }
        }

        opcodes::OP_JALR => {
            let imm = (word >> 20) as i32 as i64;
            if rd == 0 && rs1 == 1 && imm == 0 {
                "ret".to_string()
            } else {
                format!("jalr {}, {}({})", rd_str, imm, rs1_str)
            }
        }
        opcodes::OP_BRANCH => {
            let imm12 = (word >> 31) & 1;
            let imm10_5 = (word >> 25) & 0x3f;
            let imm4_1 = (word >> 8) & 0xf;
            let imm11 = (word >> 7) & 1;
            let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
            let signed_imm = ((imm as i32) << 19) >> 19;
            let target = pc.wrapping_add(signed_imm as i64 as u64);
            let mnemonic = match funct3 {
                funct3::BEQ => "beq",
                funct3::BNE => "bne",
                funct3::BLT => "blt",
                funct3::BGE => "bge",
                funct3::BLTU => "bltu",
                funct3::BGEU => "bgeu",
                _ => "unknown_branch",
            };
            format!("{} {}, {}, {:#x}", mnemonic, rs1_str, rs2_str, target)
        }
        opcodes::OP_LOAD => {
            let imm = (word as i32 >> 20) as i64;
            let mnemonic = match funct3 {
                funct3::LB => "lb",
                funct3::LH => "lh",
                funct3::LW => "lw",
                funct3::LD => "ld",
                funct3::LBU => "lbu",
                funct3::LHU => "lhu",
                funct3::LWU => "lwu",
                _ => "unknown_load",
            };
            format!("{} {}, {}({})", mnemonic, rd_str, imm, rs1_str)
        }
        opcodes::OP_STORE => {
            let imm11_5 = (word >> 25) & 0x7f;
            let imm4_0 = (word >> 7) & 0x1f;
            let imm = (imm11_5 << 5) | imm4_0;
            let signed_imm = ((imm as i32) << 20) >> 20;
            let mnemonic = match funct3 {
                funct3::SB => "sb",
                funct3::SH => "sh",
                funct3::SW => "sw",
                funct3::SD => "sd",
                _ => "unknown_store",
            };
            format!("{} {}, {}({})", mnemonic, rs2_str, signed_imm, rs1_str)
        }
        opcodes::OP_IMM => {
            let imm = (word as i32 >> 20) as i64;
            match funct3 {
                funct3::ADD_SUB if word == 0x00000013 => "nop".to_string(),
                funct3::ADD_SUB if imm == 0 => format!("mv {}, {}", rd_str, rs1_str),
                funct3::ADD_SUB => format!("addi {}, {}, {}", rd_str, rs1_str, imm),
                funct3::SLL => format!("slli {}, {}, {}", rd_str, rs1_str, (word >> 20) & 0x3f),
                funct3::SLT => format!("slti {}, {}, {}", rd_str, rs1_str, imm),
                funct3::SLTU => format!("sltiu {}, {}, {}", rd_str, rs1_str, imm),
                funct3::XOR => format!("xori {}, {}, {}", rd_str, rs1_str, imm),
                funct3::SRL_SRA => {
                    let shamt = (word >> 20) & 0x3f;
                    let mnemonic = if (word >> 30) & 1 == 0 {
                        "srli"
                    } else {
                        "srai"
                    };
                    format!("{} {}, {}, {}", mnemonic, rd_str, rs1_str, shamt)
                }
                funct3::OR => format!("ori {}, {}, {}", rd_str, rs1_str, imm),
                funct3::AND => format!("andi {}, {}, {}", rd_str, rs1_str, imm),
                _ => "unknown_op_imm".to_string(),
            }
        }
        opcodes::OP_REG => {
            let mnemonic = match (funct7, funct3) {
                (funct7::DEFAULT, funct3::ADD_SUB) => "add",
                (funct7::SUB, funct3::ADD_SUB) => "sub",
                (funct7::DEFAULT, funct3::SLL) => "sll",
                (funct7::DEFAULT, funct3::SLT) => "slt",
                (funct7::DEFAULT, funct3::SLTU) => "sltu",
                (funct7::DEFAULT, funct3::XOR) => "xor",
                (funct7::DEFAULT, funct3::SRL_SRA) => "srl",
                (funct7::SRA, funct3::SRL_SRA) => "sra",
                (funct7::DEFAULT, funct3::OR) => "or",
                (funct7::DEFAULT, funct3::AND) => "and",
                (funct7::MULDIV, funct3::MUL) => "mul",
                (funct7::MULDIV, funct3::MULH) => "mulh",
                (funct7::MULDIV, funct3::MULHSU) => "mulhsu",
                (funct7::MULDIV, funct3::MULHU) => "mulhu",
                (funct7::MULDIV, funct3::DIV) => "div",
                (funct7::MULDIV, funct3::DIVU) => "divu",
                (funct7::MULDIV, funct3::REM) => "rem",
                (funct7::MULDIV, funct3::REMU) => "remu",
                _ => "unknown_op_reg",
            };
            format!("{} {}, {}, {}", mnemonic, rd_str, rs1_str, rs2_str)
        }
        opcodes::OP_IMM_32 => {
            let imm = (word as i32 >> 20) as i64;
            match funct3 {
                funct3::ADD_SUB => format!("addiw {}, {}, {}", rd_str, rs1_str, imm),
                funct3::SLL => format!("slliw {}, {}, {}", rd_str, rs1_str, (word >> 20) & 0x1f),
                funct3::SRL_SRA => {
                    let shamt = (word >> 20) & 0x1f;
                    let mnemonic = if funct7 == funct7::DEFAULT {
                        "srliw"
                    } else {
                        "sraiw"
                    };
                    format!("{} {}, {}, {}", mnemonic, rd_str, rs1_str, shamt)
                }
                _ => "unknown_op_imm_32".to_string(),
            }
        }
        opcodes::OP_REG_32 => {
            let mnemonic = match (funct7, funct3) {
                (funct7::DEFAULT, funct3::ADD_SUB) => "addw",
                (funct7::SUB, funct3::ADD_SUB) => "subw",
                (funct7::DEFAULT, funct3::SLL) => "sllw",
                (funct7::DEFAULT, funct3::SRL_SRA) => "srlw",
                (funct7::SRA, funct3::SRL_SRA) => "sraw",
                (funct7::MULDIV, funct3::MUL) => "mulw",
                (funct7::MULDIV, funct3::DIV) => "divw",
                (funct7::MULDIV, funct3::DIVU) => "divuw",
                (funct7::MULDIV, funct3::REM) => "remw",
                (funct7::MULDIV, funct3::REMU) => "remuw",
                _ => "unknown_op_reg_32",
            };
            format!("{} {}, {}, {}", mnemonic, rd_str, rs1_str, rs2_str)
        }
        opcodes::OP_MISC_MEM => match funct3 {
            funct3::FENCE => "fence".to_string(),
            funct3::FENCE_I => "fence.i".to_string(),
            _ => "unknown_misc_mem".to_string(),
        },
        opcodes::OP_SYSTEM => {
            let csr = word >> 20;
            match funct3 {
                0 => match csr {
                    system::FUNCT12_ECALL => "ecall".to_string(),
                    system::FUNCT12_EBREAK => "ebreak".to_string(),
                    system::FUNCT12_SRET => "sret".to_string(),
                    system::FUNCT12_MRET => "mret".to_string(),
                    _ => "unknown_system".to_string(),
                },
                funct3::CSRRW => format!("csrrw {}, {}, {}", rd_str, csr_to_string(csr), rs1_str),
                funct3::CSRRS => format!("csrrs {}, {}, {}", rd_str, csr_to_string(csr), rs1_str),
                funct3::CSRRC => format!("csrrc {}, {}, {}", rd_str, csr_to_string(csr), rs1_str),
                funct3::CSRRWI => format!("csrrwi {}, {}, {}", rd_str, csr_to_string(csr), rs1),
                funct3::CSRRSI => format!("csrrsi {}, {}, {}", rd_str, csr_to_string(csr), rs1),
                funct3::CSRRCI => format!("csrrci {}, {}, {}", rd_str, csr_to_string(csr), rs1),
                _ => "unknown_system".to_string(),
            }
        }
        _ => format!("unimplemented {:#010x}", word),
    }
}
