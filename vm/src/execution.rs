use crate::{memory::VIRTUAL_DISK_ADDRESS, VM};
use riscv_core::{cause, csr, funct3, funct7, opcodes, system};

impl VM {
    pub(crate) fn execute(&mut self, inst: u32) -> bool {
        let opcode = inst & 0x7F;

        let mut next_pc = self.pc.wrapping_add(4);

        match opcode {
            opcodes::OP_LUI => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                if rd > 0 {
                    let imm = (inst & 0xFFFFF000) as i32;
                    self.registers[rd] = imm as i64 as u64;
                }
            }
            opcodes::OP_AUIPC => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                if rd > 0 {
                    let imm = (inst as i32 as i64) as u64 & 0xFFFFF000;
                    self.registers[rd] = self.pc.wrapping_add(imm);
                }
            }
            opcodes::OP_JAL => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm20 = (inst >> 31) & 1;
                let imm10_1 = (inst >> 21) & 0x3FF;
                let imm11 = (inst >> 20) & 1;
                let imm19_12 = (inst >> 12) & 0xFF;
                let offset = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
                let offset = (((offset as i32) << 11) >> 11) as i64 as u64;
                next_pc = self.pc.wrapping_add(offset);
            }
            opcodes::OP_JALR => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm = (inst as i32 >> 20) as i64 as u64;
                next_pc = (self.registers[rs1].wrapping_add(imm)) & !1;
            }
            opcodes::OP_BRANCH => {
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                let rs2 = ((inst >> 20) & 0x1F) as usize;
                let imm12 = (inst >> 31) & 1;
                let imm11 = (inst >> 7) & 1;
                let imm10_5 = (inst >> 25) & 0x3F;
                let imm4_1 = (inst >> 8) & 0xF;
                let offset = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
                let offset = (((offset as i32) << 19) >> 19) as i64 as u64;

                let condition_met = match funct3 {
                    funct3::BEQ => self.registers[rs1] == self.registers[rs2],
                    funct3::BNE => self.registers[rs1] != self.registers[rs2],
                    funct3::BLT => (self.registers[rs1] as i64) < (self.registers[rs2] as i64),
                    funct3::BGE => (self.registers[rs1] as i64) >= (self.registers[rs2] as i64),
                    funct3::BLTU => self.registers[rs1] < self.registers[rs2],
                    funct3::BGEU => self.registers[rs1] >= self.registers[rs2],
                    _ => {
                        return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                    }
                };
                if condition_met {
                    next_pc = self.pc.wrapping_add(offset);
                }
            }
            opcodes::OP_LOAD => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                if rd > 0 {
                    let imm = (inst as i32 >> 20) as i64 as u64;
                    let vaddr = self.registers[rs1].wrapping_add(imm);

                    if vaddr >= VIRTUAL_DISK_ADDRESS
                        && vaddr < VIRTUAL_DISK_ADDRESS + self.virtual_disk.len() as u64
                    {
                        let disk_offset = (vaddr - VIRTUAL_DISK_ADDRESS) as usize;
                        match funct3 {
                            funct3::LB => {
                                self.registers[rd] = self.virtual_disk[disk_offset] as i8 as u64
                            }
                            funct3::LH => {
                                let bytes: [u8; 2] = self.virtual_disk
                                    [disk_offset..disk_offset + 2]
                                    .try_into()
                                    .unwrap();
                                self.registers[rd] = i16::from_le_bytes(bytes) as i64 as u64;
                            }
                            funct3::LW => {
                                let bytes: [u8; 4] = self.virtual_disk
                                    [disk_offset..disk_offset + 4]
                                    .try_into()
                                    .unwrap();
                                self.registers[rd] = i32::from_le_bytes(bytes) as i64 as u64;
                            }
                            funct3::LD => {
                                let bytes: [u8; 8] = self.virtual_disk
                                    [disk_offset..disk_offset + 8]
                                    .try_into()
                                    .unwrap();
                                self.registers[rd] = u64::from_le_bytes(bytes);
                            }
                            funct3::LBU => {
                                self.registers[rd] = self.virtual_disk[disk_offset] as u64
                            }
                            funct3::LHU => {
                                let bytes: [u8; 2] = self.virtual_disk
                                    [disk_offset..disk_offset + 2]
                                    .try_into()
                                    .unwrap();
                                self.registers[rd] = u16::from_le_bytes(bytes) as u64;
                            }
                            funct3::LWU => {
                                let bytes: [u8; 4] = self.virtual_disk
                                    [disk_offset..disk_offset + 4]
                                    .try_into()
                                    .unwrap();
                                self.registers[rd] = u32::from_le_bytes(bytes) as u64;
                            }
                            _ => return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64),
                        }
                    } else {
                        // It's a normal memory access
                        let alignment = match funct3 {
                            funct3::LW | funct3::LWU => 4,
                            funct3::LD => 8,
                            funct3::LH | funct3::LHU => 2,
                            _ => 1,
                        };

                        if alignment > 1 && vaddr % alignment != 0 {
                            return self.handle_trap(cause::LOAD_ADDRESS_MISALIGNED, vaddr);
                        }

                        let paddr = match self.translate_addr(vaddr) {
                            Ok(addr) => addr,
                            Err(fault_addr) => {
                                return self.handle_trap(cause::LOAD_ACCESS_FAULT, fault_addr);
                            }
                        };

                        match funct3 {
                            funct3::LB => {
                                self.registers[rd] = self.memory[paddr] as i8 as i64 as u64
                            }
                            funct3::LH => {
                                let bytes: [u8; 2] =
                                    self.memory[paddr..paddr + 2].try_into().unwrap();
                                self.registers[rd] = i16::from_le_bytes(bytes) as i64 as u64;
                            }
                            funct3::LW => {
                                let bytes: [u8; 4] =
                                    self.memory[paddr..paddr + 4].try_into().unwrap();
                                self.registers[rd] = i32::from_le_bytes(bytes) as i64 as u64;
                            }
                            funct3::LD => {
                                let bytes: [u8; 8] =
                                    self.memory[paddr..paddr + 8].try_into().unwrap();
                                self.registers[rd] = u64::from_le_bytes(bytes);
                            }
                            funct3::LBU => self.registers[rd] = self.memory[paddr] as u64,
                            funct3::LHU => {
                                let bytes: [u8; 2] =
                                    self.memory[paddr..paddr + 2].try_into().unwrap();
                                self.registers[rd] = u16::from_le_bytes(bytes) as u64;
                            }
                            funct3::LWU => {
                                let bytes: [u8; 4] =
                                    self.memory[paddr..paddr + 4].try_into().unwrap();
                                self.registers[rd] = u32::from_le_bytes(bytes) as u64;
                            }
                            _ => {
                                return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                            }
                        }
                    }
                }
            }
            opcodes::OP_STORE => {
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                let rs2 = ((inst >> 20) & 0x1F) as usize;

                let imm4_0 = (inst >> 7) & 0x1F;
                let imm11_5 = (inst >> 25) & 0x7F;
                let imm = (((imm11_5 << 5) | imm4_0) as i32) << 20 >> 20;
                let vaddr = self.registers[rs1].wrapping_add(imm as i64 as u64);
                let data = self.registers[rs2];

                if vaddr >= VIRTUAL_DISK_ADDRESS
                    && vaddr < VIRTUAL_DISK_ADDRESS + self.virtual_disk.len() as u64
                {
                    // Our disk is read-only, so we do nothing on a store.
                    // A more complex VM could simulate writing here.
                } else {
                    let alignment = match funct3 {
                        funct3::SW => 4,
                        funct3::SD => 8,
                        funct3::SH => 2,
                        _ => 1,
                    };

                    if alignment > 1 && vaddr % alignment != 0 {
                        return self.handle_trap(cause::STORE_AMO_ADDRESS_MISALIGNED, vaddr);
                    }

                    let paddr = match self.translate_addr(vaddr) {
                        Ok(addr) => addr,
                        Err(fault_addr) => {
                            return self.handle_trap(cause::STORE_AMO_ACCESS_FAULT, fault_addr);
                        }
                    };

                    match funct3 {
                        funct3::SB => self.memory[paddr] = data as u8,
                        funct3::SH => self.memory[paddr..paddr + 2]
                            .copy_from_slice(&(data as u16).to_le_bytes()),
                        funct3::SW => self.memory[paddr..paddr + 4]
                            .copy_from_slice(&(data as u32).to_le_bytes()),
                        funct3::SD => {
                            self.memory[paddr..paddr + 8].copy_from_slice(&data.to_le_bytes())
                        }
                        _ => {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }
                    }
                }
            }
            opcodes::OP_IMM => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;

                if rd > 0 {
                    let imm = (inst as i32 >> 20) as i64 as u64;

                    match funct3 {
                        funct3::ADD_SUB => {
                            self.registers[rd] =
                                self.registers[rs1].wrapping_add(imm) as i64 as u64;
                        }
                        funct3::SLT => {
                            self.registers[rd] = if (self.registers[rs1] as i64) < (imm as i64) {
                                1
                            } else {
                                0
                            }
                        }
                        funct3::SLTU => {
                            self.registers[rd] = if self.registers[rs1] < imm { 1 } else { 0 }
                        }
                        funct3::XOR => self.registers[rd] = self.registers[rs1] ^ imm,
                        funct3::OR => self.registers[rd] = self.registers[rs1] | imm,
                        funct3::AND => self.registers[rd] = self.registers[rs1] & imm,
                        funct3::SLL => {
                            let shamt = ((inst >> 20) & 0x3F) as u32;
                            self.registers[rd] = self.registers[rs1].wrapping_shl(shamt);
                        }
                        funct3::SRL_SRA => {
                            let shamt = ((inst >> 20) & 0x3F) as u32;
                            if (inst >> 30) & 1 == 1 {
                                self.registers[rd] =
                                    (self.registers[rs1] as i64).wrapping_shr(shamt) as u64;
                            } else {
                                self.registers[rd] = self.registers[rs1].wrapping_shr(shamt);
                            }
                        }
                        _ => {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }
                    }
                }
            }
            opcodes::OP_IMM_32 => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;

                if rd > 0 {
                    let imm = (inst as i32 >> 20) as i32;
                    let val1 = self.registers[rs1] as i32;

                    match funct3 {
                        funct3::ADD_SUB => {
                            let val_rs1_32 = (self.registers[rs1] as i32) as i64;
                            let result_32 = val_rs1_32.wrapping_add(imm as i64);
                            self.registers[rd] = (result_32 as i32) as i64 as u64;
                        }
                        funct3::SLL => {
                            let shamt = ((inst >> 20) & 0x1F) as u32;
                            self.registers[rd] = val1.wrapping_shl(shamt) as i64 as u64;
                        }
                        funct3::SRL_SRA => {
                            let shamt = ((inst >> 20) & 0x1F) as u32;
                            if (inst >> 30) & 1 == 1 {
                                self.registers[rd] = val1.wrapping_shr(shamt) as i64 as u64;
                            } else {
                                self.registers[rd] =
                                    (val1 as u32).wrapping_shr(shamt) as i32 as i64 as u64;
                            }
                        }
                        _ => {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }
                    }
                }
            }
            opcodes::OP_REG => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                let rs2 = ((inst >> 20) & 0x1F) as usize;
                let funct7 = (inst >> 25) & 0x7F;

                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];
                if rd > 0 {
                    match (funct3, funct7) {
                        (funct3::ADD_SUB, funct7::DEFAULT) => {
                            self.registers[rd] = val1.wrapping_add(val2)
                        }
                        (funct3::ADD_SUB, funct7::SUB) => {
                            self.registers[rd] = val1.wrapping_sub(val2)
                        }
                        (funct3::SLL, funct7::DEFAULT) => {
                            self.registers[rd] = val1.wrapping_shl(val2 as u32)
                        }
                        (funct3::SLT, funct7::DEFAULT) => {
                            self.registers[rd] = if (val1 as i64) < (val2 as i64) { 1 } else { 0 }
                        }
                        (funct3::SLTU, funct7::DEFAULT) => {
                            self.registers[rd] = if val1 < val2 { 1 } else { 0 }
                        }
                        (funct3::XOR, funct7::DEFAULT) => self.registers[rd] = val1 ^ val2,
                        (funct3::SRL_SRA, funct7::DEFAULT) => {
                            self.registers[rd] = val1.wrapping_shr(val2 as u32)
                        }
                        (funct3::SRL_SRA, funct7::SRA) => {
                            self.registers[rd] = (val1 as i64).wrapping_shr(val2 as u32) as u64
                        }
                        (funct3::OR, funct7::DEFAULT) => self.registers[rd] = val1 | val2,
                        (funct3::AND, funct7::DEFAULT) => self.registers[rd] = val1 & val2,
                        // M Extension
                        (funct3::MUL, funct7::MULDIV) => {
                            self.registers[rd] = val1.wrapping_mul(val2)
                        }
                        (funct3::MULH, funct7::MULDIV) => {
                            let result = (val1 as i64 as i128).wrapping_mul(val2 as i64 as i128);
                            self.registers[rd] = (result >> 64) as u64;
                        }
                        (funct3::MULHSU, funct7::MULDIV) => {
                            let result = (val1 as i64 as i128).wrapping_mul(val2 as u128 as i128);
                            self.registers[rd] = (result >> 64) as u64;
                        }
                        (funct3::MULHU, funct7::MULDIV) => {
                            let result = (val1 as u128).wrapping_mul(val2 as u128);
                            self.registers[rd] = (result >> 64) as u64;
                        }
                        (funct3::DIV, funct7::MULDIV) => {
                            if val2 == 0 {
                                self.registers[rd] = u64::MAX;
                            } else {
                                self.registers[rd] = (val1 as i64).wrapping_div(val2 as i64) as u64;
                            }
                        }
                        (funct3::DIVU, funct7::MULDIV) => {
                            if val2 == 0 {
                                self.registers[rd] = u64::MAX;
                            } else {
                                self.registers[rd] = val1.wrapping_div(val2);
                            }
                        }
                        (funct3::REM, funct7::MULDIV) => {
                            if val2 == 0 {
                                self.registers[rd] = val1;
                            } else {
                                self.registers[rd] = (val1 as i64).wrapping_rem(val2 as i64) as u64;
                            }
                        }
                        (funct3::REMU, funct7::MULDIV) => {
                            if val2 == 0 {
                                self.registers[rd] = val1;
                            } else {
                                self.registers[rd] = val1.wrapping_rem(val2);
                            }
                        }
                        _ => {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }
                    }
                }
            }
            opcodes::OP_REG_32 => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                let rs2 = ((inst >> 20) & 0x1F) as usize;
                let funct7 = (inst >> 25) & 0x7F;

                let val1 = self.registers[rs1] as i32;
                let val2 = self.registers[rs2] as i32;
                if rd > 0 {
                    match (funct3, funct7) {
                        (funct3::ADD_SUB, funct7::DEFAULT) => {
                            self.registers[rd] = val1.wrapping_add(val2) as i64 as u64;
                        }
                        (funct3::ADD_SUB, funct7::SUB) => {
                            self.registers[rd] = val1.wrapping_sub(val2) as i64 as u64;
                        }
                        (funct3::SLL, funct7::DEFAULT) => {
                            let shamt = (val2 & 0x1F) as u32;
                            self.registers[rd] = (val1.wrapping_shl(shamt)) as i64 as u64;
                        }
                        (funct3::SRL_SRA, funct7::DEFAULT) => {
                            let shamt = (val2 & 0x1F) as u32;
                            self.registers[rd] =
                                ((val1 as u32).wrapping_shr(shamt)) as i32 as i64 as u64;
                        }
                        (funct3::SRL_SRA, funct7::SRA) => {
                            let shamt = (val2 & 0x1F) as u32;
                            self.registers[rd] = (val1.wrapping_shr(shamt)) as i64 as u64;
                        }
                        (funct3::MUL, funct7::MULDIV) => {
                            let result = (val1 as i64).wrapping_mul(val2 as i64) as i32;
                            self.registers[rd] = result as i64 as u64;
                        }
                        (funct3::DIV, funct7::MULDIV) => {
                            self.registers[rd] = if val2 == 0 {
                                (-1i32) as i64 as u64
                            } else if val1 == i32::MIN && val2 == -1 {
                                (i32::MIN as i64) as u64
                            } else {
                                (val1.wrapping_div(val2)) as i64 as u64
                            };
                        }
                        (funct3::DIVU, funct7::MULDIV) => {
                            let lhs = self.registers[rs1] as u32;
                            let rhs = self.registers[rs2] as u32;
                            let result = if rhs == 0 {
                                0xFFFF_FFFFu32
                            } else {
                                lhs.wrapping_div(rhs)
                            };
                            self.registers[rd] = (result as i32 as i64) as u64;
                        }
                        (funct3::REM, funct7::MULDIV) => {
                            self.registers[rd] = if val2 == 0 {
                                val1 as i64 as u64
                            } else if val1 == i32::MIN && val2 == -1 {
                                0
                            } else {
                                (val1.wrapping_rem(val2)) as i64 as u64
                            };
                        }
                        (funct3::REMU, funct7::MULDIV) => {
                            let lhs = self.registers[rs1] as u32;
                            let rhs = self.registers[rs2] as u32;
                            let result = if rhs == 0 { lhs } else { lhs.wrapping_rem(rhs) };
                            self.registers[rd] = (result as i32 as i64) as u64;
                        }
                        _ => {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }
                    }
                }
            }

            opcodes::OP_MISC_MEM => {
                let funct3 = (inst >> 12) & 0x7;
                match funct3 {
                    funct3::FENCE | funct3::FENCE_I => {
                        // In a simple single-core VM, FENCE can be treated as a NOP.
                        // A more complex implementation would handle memory ordering here.
                    }
                    _ => {
                        return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                    }
                }
            }
            opcodes::OP_SYSTEM => {
                let rd = ((inst >> 7) & 0x1F) as usize;
                let funct3 = (inst >> 12) & 0x7;
                let rs1 = ((inst >> 15) & 0x1F) as usize;
                match funct3 {
                    0b000 => {
                        let funct12 = (inst >> 20) & 0xFFF;
                        match funct12 {
                            system::FUNCT12_ECALL => {
                                return self.handle_trap(
                                    cause::ECALL_FROM_U_MODE + self.privilege_level as u64,
                                    0,
                                );
                            }
                            system::FUNCT12_EBREAK => {
                                return self.handle_trap(cause::BREAKPOINT, 0);
                            }
                            system::FUNCT12_MRET => {
                                let mstatus =
                                    self.csrs.read(csr::MSTATUS, self.privilege_level).unwrap();
                                self.privilege_level = ((mstatus >> 11) & 0b11) as u8;
                                let mpie = (mstatus >> 7) & 1;
                                self.csrs.mstatus = (self.csrs.mstatus & !(1 << 3)) | (mpie << 3);
                                self.csrs.mstatus |= 1 << 7;
                                self.csrs.mstatus &= !(0b11 << 11);
                                next_pc = self.csrs.read(csr::MEPC, self.privilege_level).unwrap();
                            }
                            system::FUNCT12_SRET => {
                                let sstatus =
                                    self.csrs.read(csr::SSTATUS, self.privilege_level).unwrap();

                                self.privilege_level = ((sstatus >> 8) & 0b1) as u8;

                                let spie = (sstatus >> 5) & 1;

                                let mut mstatus =
                                    self.csrs.read(csr::MSTATUS, self.privilege_level).unwrap();

                                mstatus = (mstatus & !(1 << 1)) | (spie << 1);
                                mstatus |= 1 << 5;
                                mstatus &= !(1 << 8);

                                self.csrs.write(csr::MSTATUS, mstatus, self.privilege_level);

                                next_pc = self.csrs.read(csr::SEPC, self.privilege_level).unwrap();
                            }
                            _ => {
                                return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                            }
                        }
                    }
                    funct3::CSRRW
                    | funct3::CSRRS
                    | funct3::CSRRC
                    | funct3::CSRRWI
                    | funct3::CSRRSI
                    | funct3::CSRRCI => {
                        let csr_addr = (inst >> 20) as u32;
                        let old_val = match self.csrs.read(csr_addr, self.privilege_level) {
                            Some(val) => val,
                            None => {
                                return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                            }
                        };

                        let write_val = if funct3 & 0b100 == 0b100 {
                            ((inst >> 15) & 0x1F) as u64
                        } else {
                            self.registers[rs1]
                        };

                        let new_val = match funct3 & 0b011 {
                            funct3::CSRRW => write_val,
                            funct3::CSRRS => old_val | write_val,
                            funct3::CSRRC => old_val & !write_val,
                            _ => unreachable!(),
                        };

                        if !self.csrs.write(csr_addr, new_val, self.privilege_level) {
                            return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                        }

                        if rd > 0 {
                            self.registers[rd] = old_val;
                        }
                    }
                    _ => {
                        return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
                    }
                }
            }
            _ => {
                return self.handle_trap(cause::ILLEGAL_INSTRUCTION, inst as u64);
            }
        }

        self.pc = next_pc;
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::{memory::BASE_ADDRESS, VM};
    use riscv_core::{abi, csr};

    fn setup_vm() -> VM {
        let mut vm = VM::new();
        vm.pc = BASE_ADDRESS;
        vm
    }

    #[test]
    fn test_op_lui() {
        let mut vm = setup_vm();
        vm.execute(0xabcde537); // lui a0, 0xABCDE
        assert_eq!(vm.registers[abi::A0 as usize], 0xffffffffabcde000);
        assert_eq!(vm.pc, BASE_ADDRESS + 4);
    }

    #[test]
    fn test_op_auipc() {
        let mut vm = setup_vm();
        vm.execute(0x00001517); // auipc a0, 0x1
        assert_eq!(vm.registers[abi::A0 as usize], BASE_ADDRESS + 0x1000);
    }

    #[test]
    fn test_op_jal() {
        let mut vm = setup_vm();
        vm.execute(0x014000ef); // jal ra, 20
        assert_eq!(vm.registers[abi::RA as usize], BASE_ADDRESS + 4);
        assert_eq!(vm.pc, BASE_ADDRESS + 20);
    }

    #[test]
    fn test_op_jalr() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = BASE_ADDRESS + 0x100;
        vm.execute(0x020500e7); // jalr ra, 32(a0)
        assert_eq!(vm.registers[abi::RA as usize], BASE_ADDRESS + 4);
        assert_eq!(vm.pc, BASE_ADDRESS + 0x100 + 32);
    }

    #[test]
    fn test_op_branch() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 5;
        vm.registers[abi::A1 as usize] = 5;
        vm.execute(0x00b50863); // beq a0, a1, 16 (taken)
        assert_eq!(vm.pc, BASE_ADDRESS + 16);
    }

    #[test]
    fn test_op_loads() {
        let mut vm = setup_vm();
        let data_addr = BASE_ADDRESS + 0x200;
        let data_val = 0x8899AABBCCDDEEFF_u64;
        let paddr = vm.translate_addr(data_addr).unwrap();
        vm.memory[paddr..paddr + 8].copy_from_slice(&data_val.to_le_bytes());
        vm.registers[abi::A0 as usize] = data_addr;

        vm.execute(0x00053583); // ld a1, 0(a0)
        assert_eq!(vm.registers[abi::A1 as usize], data_val);
        vm.execute(0x00052603); // lw a2, 0(a0)
        assert_eq!(vm.registers[abi::A2 as usize], 0xffffffffccddeeff);
    }

    #[test]
    fn test_op_stores() {
        let mut vm = setup_vm();
        let store_addr = BASE_ADDRESS + 0x200;
        vm.registers[abi::A0 as usize] = store_addr;
        vm.registers[abi::A1 as usize] = 0x11223344AABBCCDD;
        vm.execute(0x00b53023); // sd a1, 0(a0)
        let paddr = vm.translate_addr(store_addr).unwrap();
        assert_eq!(
            u64::from_le_bytes(vm.memory[paddr..paddr + 8].try_into().unwrap()),
            0x11223344AABBCCDD
        );
    }

    #[test]
    fn test_op_imm() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 100;
        vm.execute(0xff650593); // addi a1, a0, -10
        assert_eq!(vm.registers[abi::A1 as usize], 90);
    }

    #[test]
    fn test_op_imm_32() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 0xFFFFFFFF_80000000;
        vm.execute(0x0015059B); // addiw a1, a0, 1
        assert_eq!(vm.registers[abi::A1 as usize], -2147483647i64 as u64);
    }

    #[test]
    fn test_op_imm_shifts_rv64() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 0x00000000_FFFFFFFF;
        vm.execute(0x02051593); // slli a1, a0, 32
        assert_eq!(vm.registers[abi::A1 as usize], 0xFFFFFFFF_00000000);
    }

    #[test]
    fn test_op_reg() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 100;
        vm.registers[abi::A1 as usize] = 50;
        vm.execute(0x40b50633); // sub a2, a0, a1
        assert_eq!(vm.registers[abi::A2 as usize], 50);
    }

    #[test]
    fn test_op_m_extension() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = -100i64 as u64;
        vm.registers[abi::A1 as usize] = 10;
        vm.execute(0x02b50633); // mul a2, a0, a1
        assert_eq!(vm.registers[abi::A2 as usize], -1000i64 as u64);
    }

    #[test]
    fn test_op_reg_32() {
        let mut vm = setup_vm();
        vm.registers[abi::A0 as usize] = 10;
        vm.registers[abi::A1 as usize] = 20;
        vm.execute(0x40b505bb); // subw a1, a0, a1 -> -10
        assert_eq!(vm.registers[abi::A1 as usize], -10i64 as u64);
    }

    #[test]
    fn test_op_system_csr() {
        let mut vm = setup_vm();
        vm.csrs.write(csr::MSTATUS, 0xABCD, vm.privilege_level);
        vm.registers[abi::A0 as usize] = 0x1234;

        // csrrw a1, mstatus, a0
        vm.execute(0x300515f3);

        assert_eq!(vm.registers[abi::A1 as usize], 0xABCD);

        let new_mstatus = vm.csrs.read(csr::MSTATUS, vm.privilege_level).unwrap();
        assert_eq!(new_mstatus, 0x1234);
    }
}
