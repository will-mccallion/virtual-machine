use crate::VM;
use riscv_core::{cause, funct3, funct7, opcodes, system};

impl VM {
    pub(crate) fn execute(&mut self, inst: u32) {
        let opcode = inst & 0x7F;
        let rd = ((inst >> 7) & 0x1F) as usize;
        let rs1 = ((inst >> 15) & 0x1F) as usize;
        let rs2 = ((inst >> 20) & 0x1F) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7F;

        // PRIO 7: # TODO: Add support for the 'C' (Compressed) extension. This requires checking the lower 2 bits of the instruction to determine if it's 16-bit or 32-bit, and adjusting the PC increment accordingly.
        let mut next_pc = self.pc.wrapping_add(4);

        match opcode {
            opcodes::OP_REG => {
                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];
                if rd > 0 {
                    match (funct3, funct7) {
                        // PRIO 3: # TODO: Implement remaining RV64I R-type instructions: SLL, SLTU, ADDW, SUBW, SLLW, SRLW, SRAW.
                        // PRIO 5: # TODO: Implement remaining 'M' extension instructions: MULH, MULHSU, MULHU, DIVU, REM, REMU, DIVW, REMW.
                        (funct3::ADD_SUB, funct7::DEFAULT) => {
                            self.registers[rd] = val1.wrapping_add(val2)
                        }
                        (funct3::ADD_SUB, funct7::SUB) => {
                            self.registers[rd] = val1.wrapping_sub(val2)
                        }
                        (funct3::MUL, funct7::MULDIV) => {
                            self.registers[rd] = val1.wrapping_mul(val2)
                        }
                        (funct3::DIV, funct7::MULDIV) => {
                            self.registers[rd] = if val2 == 0 {
                                u64::MAX // PRIO 2: # FIX: Division by zero should set rd to all 1s. For REM, the result should be the dividend (val1). This logic is incomplete.
                            } else {
                                (val1 as i64).wrapping_div(val2 as i64) as u64
                            }
                        }
                        (funct3::OR, funct7::DEFAULT) => self.registers[rd] = val1 | val2,
                        (funct3::AND, funct7::DEFAULT) => self.registers[rd] = val1 & val2,
                        (funct3::XOR, funct7::DEFAULT) => self.registers[rd] = val1 ^ val2,
                        (funct3::SLT, funct7::DEFAULT) => {
                            self.registers[rd] = if (val1 as i64) < (val2 as i64) { 1 } else { 0 }
                        }
                        (funct3::SRL_SRA, funct7::DEFAULT) => {
                            self.registers[rd] = val1 >> (val2 & 0x3F)
                        }
                        (funct3::SRL_SRA, funct7::SRA) => {
                            self.registers[rd] = ((val1 as i64) >> (val2 & 0x3F)) as u64
                        }
                        _ => {
                            self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                            return;
                        }
                    }
                }
            }
            opcodes::OP_AUIPC => {
                if rd > 0 {
                    let imm = (inst & 0xFFFFF000) as i32 as i64 as u64;
                    self.registers[rd] = self.pc.wrapping_add(imm);
                }
            }
            opcodes::OP_IMM => {
                if rd > 0 {
                    // PRIO 3: # TODO: Implement other OP_IMM instructions: SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI, and their 'W' variants for RV64.
                    // PRIO 2: # FIX: The immediate decoding `(inst as i32 >> 20)` is only correct for some I-types. Shift instructions use a different format for the immediate (shamt).
                    let imm = (inst as i32 >> 20) as i64 as u64;
                    self.registers[rd] = self.registers[rs1].wrapping_add(imm);
                }
            }
            opcodes::OP_LOAD => {
                if rd > 0 {
                    let imm = (inst as i32 >> 20) as i64 as u64;
                    let vaddr = self.registers[rs1].wrapping_add(imm);

                    // PRIO 2: # FIX: Load operations must check for memory alignment.
                    let alignment = match funct3 {
                        funct3::LW | funct3::LWU => 4,
                        funct3::LD => 8,
                        funct3::LH | funct3::LHU => 2,
                        _ => 1,
                    };
                    if alignment > 1 && vaddr % alignment != 0 {
                        self.handle_trap(cause::LOAD_ADDRESS_MISALIGNED, vaddr);
                        return;
                    }

                    let paddr = match self.translate_addr(vaddr) {
                        Ok(addr) => addr,
                        Err(fault_addr) => {
                            self.handle_trap(cause::LOAD_ACCESS_FAULT, fault_addr);
                            return;
                        }
                    };

                    match funct3 {
                        funct3::LW => {
                            let bytes: [u8; 4] = self.memory[paddr..paddr + 4].try_into().unwrap();
                            self.registers[rd] = i32::from_le_bytes(bytes) as i64 as u64;
                        }
                        // PRIO 3: # TODO: Implement remaining standard load instructions: LH, LHU, and LWU.
                        funct3::LD => {
                            let bytes: [u8; 8] = self.memory[paddr..paddr + 8].try_into().unwrap();
                            self.registers[rd] = u64::from_le_bytes(bytes);
                        }
                        funct3::LB => self.registers[rd] = self.memory[paddr] as i8 as i64 as u64,
                        funct3::LBU => self.registers[rd] = self.memory[paddr] as u64,
                        _ => {
                            self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                            return;
                        }
                    }
                }
            }
            opcodes::OP_STORE => {
                // PRIO 2: # FIX: The immediate for S-type instructions should be correctly sign-extended from its 12 bits.
                let imm4_0 = (inst >> 7) & 0x1F;
                let imm11_5 = (inst >> 25) & 0x7F;
                let imm = (((imm11_5 << 5) | imm4_0) as i32) << 20 >> 20;
                let vaddr = self.registers[rs1].wrapping_add(imm as i64 as u64);
                let data = self.registers[rs2];

                // PRIO 2: # FIX: Store operations must check for memory alignment.
                let alignment = match funct3 {
                    funct3::SW => 4,
                    funct3::SD => 8,
                    funct3::SH => 2,
                    _ => 1,
                };
                if alignment > 1 && vaddr % alignment != 0 {
                    self.handle_trap(cause::STORE_AMO_ADDRESS_MISALIGNED, vaddr);
                    return;
                }

                let paddr = match self.translate_addr(vaddr) {
                    Ok(addr) => addr,
                    Err(fault_addr) => {
                        self.handle_trap(cause::STORE_AMO_ACCESS_FAULT, fault_addr);
                        return;
                    }
                };

                match funct3 {
                    // PRIO 3: # TODO: Implement the remaining standard store instructions: SH.
                    funct3::SW => {
                        self.memory[paddr..paddr + 4].copy_from_slice(&(data as u32).to_le_bytes())
                    }
                    funct3::SD => {
                        self.memory[paddr..paddr + 8].copy_from_slice(&data.to_le_bytes())
                    }
                    funct3::SB => self.memory[paddr] = data as u8,
                    _ => {
                        self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        return;
                    }
                }
            }
            opcodes::OP_BRANCH => {
                let imm12 = (inst >> 31) & 1;
                let imm11 = (inst >> 7) & 1;
                let imm10_5 = (inst >> 25) & 0x3F;
                let imm4_1 = (inst >> 8) & 0xF;
                let offset = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
                // PRIO 2: # FIX: The sign extension for the branch offset must correctly sign-extend the 13-bit B-immediate.
                let offset = (((offset as i32) << 19) >> 19) as i64 as u64;

                let condition_met = match funct3 {
                    // PRIO 3: # TODO: Implement remaining standard branch instructions: BGE, BGEU, and BLTU.
                    funct3::BEQ => self.registers[rs1] == self.registers[rs2],
                    funct3::BNE => self.registers[rs1] != self.registers[rs2],
                    funct3::BLT => (self.registers[rs1] as i64) < (self.registers[rs2] as i64),
                    _ => {
                        self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        return;
                    }
                };
                if condition_met {
                    next_pc = self.pc.wrapping_add(offset);
                }
            }
            opcodes::OP_JAL => {
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm20 = (inst >> 31) & 1;
                let imm10_1 = (inst >> 21) & 0x3FF;
                let imm11 = (inst >> 20) & 1;
                let imm19_12 = (inst >> 12) & 0xFF;
                let offset = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
                // PRIO 2: # FIX: The sign extension for the JAL offset must correctly sign-extend the 21-bit J-immediate.
                let offset = (((offset as i32) << 11) >> 11) as i64 as u64;
                next_pc = self.pc.wrapping_add(offset);
            }
            opcodes::OP_JALR => {
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm = (inst as i32 >> 20) as i64 as u64;
                // PRIO 2: # FIX: The target address for JALR must have its least-significant bit cleared to zero before jumping.
                next_pc = self.registers[rs1].wrapping_add(imm) & !1;
            }
            opcodes::OP_SYSTEM => {
                let funct12 = (inst >> 20) & 0xFFF;
                // PRIO 4: # TODO: Implement CSR instructions (CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI) which read and write to the control and status registers.
                // PRIO 4: # TODO: Implement trap-return instructions (MRET, SRET) and `ebreak`.
                match funct12 {
                    system::FUNCT12_ECALL => {
                        self.handle_trap(cause::ECALL_FROM_U_MODE + self.privilege_level as u64, 0);
                        return;
                    }
                    _ => {
                        self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        return;
                    }
                }
            }
            _ => {
                self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                return;
            }
        }

        self.pc = next_pc;
    }
}
