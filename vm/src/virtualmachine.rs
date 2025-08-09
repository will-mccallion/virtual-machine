const MEMORY_SIZE: usize = 1024 * 1024 * 128; // 128MB
const CSR_SIZE: usize = 4096;

// --- RISC-V Instruction Constants ---
// Opcodes
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

const FUNCT12_ECALL: u32 = 0x0;

// --- CSR Addresses for Printing ---
// Machine-Level
const MSTATUS: usize = 0x300;
const MISA: usize = 0x301;
const MIE: usize = 0x304;
const MTVEC: usize = 0x305;
const MSCRATCH: usize = 0x340;
const MEPC: usize = 0x341;
const MCAUSE: usize = 0x342;
const MTVAL: usize = 0x343;
const MIP: usize = 0x344;
// Supervisor-Level
const SSTATUS: usize = 0x100;
const SIE: usize = 0x104;
const STVEC: usize = 0x105;
const SSCRATCH: usize = 0x140;
const SEPC: usize = 0x141;
const SCAUSE: usize = 0x142;
const STVAL: usize = 0x143;
const SIP: usize = 0x144;
const SATP: usize = 0x180;

pub struct VM {
    registers: [u64; 32],
    pc: u64,
    memory: Vec<u8>,
    csr: [u64; CSR_SIZE],
}

impl VM {
    pub fn new() -> VM {
        let mut vm = VM {
            registers: [0; 32],
            pc: 0,
            memory: vec![0; MEMORY_SIZE],
            csr: [0; CSR_SIZE],
        };
        vm.registers[2] = MEMORY_SIZE as u64;
        vm
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory[..program.len()].copy_from_slice(program);
    }

    fn fetch(&self) -> u32 {
        let pc = self.pc as usize;
        u32::from_le_bytes(self.memory[pc..pc + 4].try_into().unwrap())
    }

    fn decode_i_imm(&self, inst: u32) -> i64 {
        (inst as i32 >> 20) as i64
    }

    fn decode_s_imm(&self, inst: u32) -> i64 {
        let imm4_0 = (inst >> 7) & 0x1F;
        let imm11_5 = (inst >> 25) & 0x7F;
        let imm = (imm11_5 << 5) | imm4_0;
        ((imm as i32) << 20 >> 20) as i64
    }

    fn decode_sb_imm(&self, inst: u32) -> i64 {
        let imm12 = (inst >> 31) & 1;
        let imm11 = (inst >> 7) & 1;
        let imm10_5 = (inst >> 25) & 0x3F;
        let imm4_1 = (inst >> 8) & 0xF;
        let offset = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
        ((offset as i32) << 19 >> 19) as i64
    }

    fn decode_u_imm(&self, inst: u32) -> i64 {
        (inst >> 12) as i64
    }

    fn decode_uj_imm(&self, inst: u32) -> i64 {
        let imm20 = (inst >> 31) & 1;
        let imm10_1 = (inst >> 21) & 0x3FF;
        let imm11 = (inst >> 20) & 1;
        let imm19_12 = (inst >> 12) & 0xFF;
        let offset = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
        ((offset as i32) << 11 >> 11) as i64
    }

    pub fn run(&mut self) {
        loop {
            if self.pc as usize >= MEMORY_SIZE {
                println!("PC exceeding memory.");
                break;
            }
            let instruction = self.fetch();
            if instruction == 0x00000000 {
                break;
            }
            if !self.execute(instruction) {
                break;
            }
            self.registers[0] = 0;
        }
    }

    fn execute(&mut self, inst: u32) -> bool {
        let opcode = inst & 0x7F;
        let rd = ((inst >> 7) & 0x1F) as usize;
        let rs1 = ((inst >> 15) & 0x1F) as usize;
        let rs2 = ((inst >> 20) & 0x1F) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7F;

        let mut next_pc = self.pc.wrapping_add(4);

        match opcode {
            OP_REG => {
                if rd > 0 {
                    let val1 = self.registers[rs1];
                    let val2 = self.registers[rs2];
                    match (funct3, funct7) {
                        (FUNCT3_ADD_SUB, FUNCT7_ADD) => {
                            self.registers[rd] = val1.wrapping_add(val2)
                        }

                        (FUNCT3_ADD_SUB, FUNCT7_SUB) => {
                            self.registers[rd] = val1.wrapping_sub(val2)
                        }

                        (FUNCT3_MUL, FUNCT7_MULDIV) => self.registers[rd] = val1.wrapping_mul(val2),
                        (FUNCT3_DIV, FUNCT7_MULDIV) => self.registers[rd] = val1.wrapping_div(val2),
                        (FUNCT3_OR, FUNCT7_OR) => self.registers[rd] = val1 | val2,
                        (FUNCT3_AND, FUNCT7_AND) => self.registers[rd] = val1 & val2,
                        (FUNCT3_XOR, FUNCT7_XOR) => self.registers[rd] = val1 ^ val2,
                        (FUNCT3_SLT, FUNCT7_SLT) => {
                            self.registers[rd] = if val1 < val2 { 1 } else { 0 }
                        }
                        (FUNCT3_SRL, FUNCT7_SRL) => self.registers[rd] = val1 >> val2,
                        (FUNCT3_SRA, FUNCT7_SRA) => {
                            self.registers[rd] = ((val1 as i64) >> (val2 as i64)) as u64
                        }
                        _ => println!(
                            "Unsupported R-type: funct3={:#b}, funct7={:#b}",
                            funct3, funct7
                        ),
                    }
                    const FUNCT3_SLT: u32 = 0b010;
                    const FUNCT3_SRA: u32 = 0b101;
                    const FUNCT3_SRL: u32 = 0b101;
                    const FUNCT3_XOR: u32 = 0b100;
                }
            }
            OP_IMM => {
                if rd > 0 && funct3 == FUNCT3_ADDI {
                    let imm = self.decode_i_imm(inst);
                    self.registers[rd] = self.registers[rs1].wrapping_add(imm as u64);
                }
            }
            OP_LOAD => {
                if rd > 0 {
                    let imm = self.decode_i_imm(inst);
                    let addr = self.registers[rs1].wrapping_add(imm as u64) as usize;
                    match funct3 {
                        FUNCT3_LW => {
                            let bytes: [u8; 4] = self.memory[addr..addr + 4].try_into().unwrap();
                            self.registers[rd] = i32::from_le_bytes(bytes) as i64 as u64;
                        }
                        FUNCT3_LD => {
                            let bytes: [u8; 8] = self.memory[addr..addr + 8].try_into().unwrap();
                            self.registers[rd] = i64::from_le_bytes(bytes) as i64 as u64;
                        }
                        FUNCT3_LB => {
                            self.registers[rd] = self.memory[addr] as u64;
                        }
                        _ => {}
                    }
                }
            }
            OP_STORE => {
                let imm = self.decode_s_imm(inst);
                let addr = self.registers[rs1].wrapping_add(imm as u64) as usize;
                match funct3 {
                    FUNCT3_SW => {
                        let data = self.registers[rs2] as u32;
                        self.memory[addr..addr + 4].copy_from_slice(&data.to_le_bytes());
                    }
                    FUNCT3_SD => {
                        let data = self.registers[rs2];
                        self.memory[addr..addr + 8].copy_from_slice(&data.to_le_bytes());
                    }
                    FUNCT3_SB => {
                        let data = self.registers[rs2];
                        self.memory[addr..addr + 1].copy_from_slice(&data.to_le_bytes());
                    }
                    _ => {}
                }
            }
            OP_BRANCH => match funct3 {
                FUNCT3_BEQ => {
                    let offset = self.decode_sb_imm(inst);
                    if self.registers[rs1] == self.registers[rs2] {
                        next_pc = self.pc.wrapping_add(offset as u64);
                    }
                }
                FUNCT3_BLT => {
                    let offset = self.decode_sb_imm(inst);
                    if self.registers[rs1] < self.registers[rs2] {
                        next_pc = self.pc.wrapping_add(offset as u64);
                    }
                }
                FUNCT3_BNE => {
                    let offset = self.decode_sb_imm(inst);
                    if self.registers[rs1] != self.registers[rs2] {
                        next_pc = self.pc.wrapping_add(offset as u64);
                    }
                }
                _ => {}
            },
            OP_JAL => {
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let offset = self.decode_uj_imm(inst);
                next_pc = self.pc.wrapping_add(offset as u64);
            }
            OP_JALR => {
                if rd > 0 {
                    self.registers[rd] = next_pc;
                }
                let imm = self.decode_i_imm(inst);
                next_pc = self.registers[rs1].wrapping_add(imm as u64) & !1;
            }
            OP_SYSTEM => {
                let funct12 = (inst >> 20) & 0xFFF;
                if funct12 == FUNCT12_ECALL {
                    println!("ECALL: System call initiated. Halting.");
                    return false;
                }
            }
            _ => println!("Unknown opcode: {:#09b} at PC {:#x}", opcode, self.pc),
        }

        self.pc = next_pc;
        true
    }

    pub fn print_state(&self) {
        let key_csrs_to_print = [
            (MSTATUS, "mstatus"),
            (MISA, "misa"),
            (MIE, "mie"),
            (MTVEC, "mtvec"),
            (MSCRATCH, "mscratch"),
            (MEPC, "mepc"),
            (MCAUSE, "mcause"),
            (MTVAL, "mtval"),
            (MIP, "mip"),
            (SSTATUS, "sstatus"),
            (SIE, "sie"),
            (STVEC, "stvec"),
            (SSCRATCH, "sscratch"),
            (SEPC, "sepc"),
            (SCAUSE, "scause"),
            (STVAL, "stval"),
            (SIP, "sip"),
            (SATP, "satp"),
        ];

        let abi = [
            "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3",
            "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
            "t3", "t4", "t5", "t6",
        ];

        println!("\n--- VM execution finished ---");
        println!("Final pc value: {:#018x}", self.pc);

        let gpr_header = format!("{:<5} {:<7} {:<18}", "Reg", "(ABI)", "Value");
        let csr_header = format!("{:<8} {:<10} {:<18}", "Address", "Name", "Value");
        let separator = " | ";

        println!(
            "{}{}{}",
            "--- General Purpose Registers ---", separator, "--- Control & Status Registers ---"
        );
        println!(
            "{}{}{}",
            "-".repeat(gpr_header.len()),
            separator,
            "-".repeat(csr_header.len())
        );
        println!("{}{}{}", gpr_header, separator, csr_header);
        println!(
            "{}{}{}",
            "-".repeat(gpr_header.len()),
            separator,
            "-".repeat(csr_header.len())
        );

        for i in 0..32 {
            let reg_name = format!("x{}", i);
            let abi_name = format!("({})", abi[i]);
            let gpr_line = format!(
                "{:<5} {:<7} {:#018x}",
                reg_name, abi_name, self.registers[i]
            );

            if i < key_csrs_to_print.len() {
                let (addr, name) = key_csrs_to_print[i];
                let csr_val = self.csr[addr];
                let csr_line = format!("{:<#8x} {:<10} {:#018x}", addr, name, csr_val);
                println!("{}{}{}", gpr_line, separator, csr_line);
            } else {
                println!("{}", gpr_line);
            }
        }
    }
}
