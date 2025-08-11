use assembler::Executable;
use riscv_core::{funct3, funct7, opcodes, system};
use std::error::Error;
use std::fmt;

const MEMORY_SIZE: usize = 1024 * 1024 * 128; // 128MB
const CSR_SIZE: usize = 4096;

#[derive(Debug)]
pub enum VMError {
    PcOutOfBounds(u64),
    MemoryOutOfBounds(u64),
    UnknownOpcode(u32),
    Ecall,
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PcOutOfBounds(pc) => write!(f, "Program Counter out of bounds at {:#x}", pc),
            Self::MemoryOutOfBounds(addr) => {
                write!(f, "Memory access out of bounds at {:#x}", addr)
            }
            Self::UnknownOpcode(opcode) => write!(f, "Unknown opcode encountered: {:#09b}", opcode),
            Self::Ecall => write!(f, "ECALL instruction executed, halting."),
        }
    }
}
impl Error for VMError {}

pub struct VM {
    registers: [u64; 32],
    pc: u64,
    memory: Vec<u8>,
    csr: [u64; CSR_SIZE],
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            registers: [0; 32],
            pc: 0,
            memory: vec![0; MEMORY_SIZE],
            csr: [0; CSR_SIZE],
        };
        vm.registers[2] = MEMORY_SIZE as u64;
        vm
    }

    pub fn load_executable(&mut self, executable: &Executable) -> Result<(), VMError> {
        let text_size = executable.text.len();
        let data_size = executable.data.len();

        if text_size + data_size > MEMORY_SIZE {
            return Err(VMError::MemoryOutOfBounds((text_size + data_size) as u64));
        }

        self.memory[..text_size].copy_from_slice(&executable.text);

        self.memory[text_size..text_size + data_size].copy_from_slice(&executable.data);

        self.pc = 0;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            let instruction = self.fetch()?;
            if instruction == opcodes::OP_HALT {
                break;
            }
            self.execute(instruction)?;
            self.registers[0] = 0;
        }
        Ok(())
    }

    fn fetch(&self) -> Result<u32, VMError> {
        let pc = self.pc as usize;
        if pc + 4 > self.memory.len() {
            return Err(VMError::PcOutOfBounds(self.pc));
        }
        let inst_bytes: [u8; 4] = self.memory[pc..pc + 4].try_into().unwrap();
        Ok(u32::from_le_bytes(inst_bytes))
    }

    fn execute(&mut self, inst: u32) -> Result<(), VMError> {
        let opcode = inst & 0x7F;
        let rd = ((inst >> 7) & 0x1F) as usize;
        let rs1 = ((inst >> 15) & 0x1F) as usize;
        let rs2 = ((inst >> 20) & 0x1F) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7F;

        let mut next_pc = self.pc.wrapping_add(4);

        match opcode {
            opcodes::OP_REG => {
                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];
                if rd > 0 {
                    match (funct3, funct7) {
                        (funct3::ADD_SUB, funct7::ADD) => {
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
                                u64::MAX
                            } else {
                                val1.wrapping_div(val2)
                            }
                        }
                        (funct3::OR, funct7::DEFAULT) => self.registers[rd] = val1 | val2,
                        (funct3::AND, funct7::DEFAULT) => self.registers[rd] = val1 & val2,
                        (funct3::XOR, funct7::DEFAULT) => self.registers[rd] = val1 ^ val2,
                        (funct3::SLT, funct7::DEFAULT) => {
                            self.registers[rd] = if (val1 as i64) < (val2 as i64) { 1 } else { 0 }
                        }
                        (funct3::SRL, funct7::DEFAULT) => self.registers[rd] = val1 >> val2,
                        (funct3::SRA, funct7::SRA) => {
                            self.registers[rd] = ((val1 as i64) >> val2) as u64
                        }
                        _ => { /* Unsupported R-type variants can be added here */ }
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
                    let imm = (inst as i32 >> 20) as i64 as u64;
                    self.registers[rd] = self.registers[rs1].wrapping_add(imm);
                }
            }
            opcodes::OP_LOAD => {
                if rd > 0 {
                    let imm = (inst as i32 >> 20) as i64 as u64;
                    let addr = self.registers[rs1].wrapping_add(imm) as usize;
                    if addr + 8 > MEMORY_SIZE {
                        return Err(VMError::MemoryOutOfBounds(addr as u64));
                    }
                    match funct3 {
                        funct3::LW => {
                            let bytes: [u8; 4] = self.memory[addr..addr + 4].try_into().unwrap();
                            self.registers[rd] = i32::from_le_bytes(bytes) as i64 as u64;
                        }
                        funct3::LD => {
                            let bytes: [u8; 8] = self.memory[addr..addr + 8].try_into().unwrap();
                            self.registers[rd] = u64::from_le_bytes(bytes);
                        }
                        funct3::LB => self.registers[rd] = self.memory[addr] as i8 as i64 as u64,
                        funct3::LBU => self.registers[rd] = self.memory[addr] as u64,
                        _ => { /* Unsupported load variants */ }
                    }
                }
            }
            opcodes::OP_STORE => {
                let imm4_0 = (inst >> 7) & 0x1F;
                let imm11_5 = (inst >> 25) & 0x7F;
                let imm = (((imm11_5 << 5) | imm4_0) as i32) << 20 >> 20;
                let addr = self.registers[rs1].wrapping_add(imm as i64 as u64) as usize;
                let data = self.registers[rs2];
                if addr + 8 > MEMORY_SIZE {
                    return Err(VMError::MemoryOutOfBounds(addr as u64));
                }
                match funct3 {
                    funct3::SW => {
                        self.memory[addr..addr + 4].copy_from_slice(&(data as u32).to_le_bytes())
                    }
                    funct3::SD => self.memory[addr..addr + 8].copy_from_slice(&data.to_le_bytes()),
                    funct3::SB => self.memory[addr] = data as u8,
                    _ => { /* Unsupported store variants */ }
                }
            }
            opcodes::OP_BRANCH => {
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
                    _ => false,
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
                let offset = (((offset as i32) << 11) >> 11) as i64 as u64;
                next_pc = self.pc.wrapping_add(offset);
            }
            opcodes::OP_JALR => {
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm = (inst as i32 >> 20) as i64 as u64;
                next_pc = self.registers[rs1].wrapping_add(imm) & !1;
            }
            opcodes::OP_SYSTEM => {
                let funct12 = (inst >> 20) & 0xFFF;
                if funct12 == system::FUNCT12_ECALL {
                    return Err(VMError::Ecall);
                }
            }
            _ => return Err(VMError::UnknownOpcode(opcode)),
        }

        self.pc = next_pc;
        Ok(())
    }

    pub fn print_state(&self) {
        let abi = [
            "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3",
            "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
            "t3", "t4", "t5", "t6",
        ];
        println!("\n--- VM Execution Halted ---");
        println!("Final PC: {:#018x}", self.pc);
        println!("---------------------------------");
        println!("General Purpose Registers");
        println!("---------------------------------");
        for i in 0..32 {
            println!(
                "x{:<2}  {:<4}  = {:#018x} ({})",
                i, abi[i], self.registers[i], self.registers[i] as i64
            );
        }
        println!("---------------------------------");
    }
}
