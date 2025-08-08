const MEMORY_SIZE: usize = 10240;

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

pub struct VM {
    registers: [u64; 32],
    pc: u64,
    memory: [u8; MEMORY_SIZE],
}

impl VM {
    pub fn new() -> VM {
        let mut vm = VM {
            registers: [0; 32],
            pc: 0,
            memory: [0; MEMORY_SIZE],
        };

        vm.registers[2] = MEMORY_SIZE as u64;
        vm
    }

    pub fn load_program(&mut self, program: &[u8]) {
        assert!(
            program.len() < MEMORY_SIZE,
            "Program is too large for VM memory."
        );
        self.memory[..program.len()].copy_from_slice(program);
    }

    pub fn run(&mut self) {
        loop {
            if self.pc as usize >= MEMORY_SIZE {
                println!("PC out of bounds. Halting.");
                break;
            }

            if self.pc >= self.registers[2] {
                println!(
                    "Stack overflow detected! PC ({:#x}) collided with SP ({:#x}). Halting.",
                    self.pc, self.registers[2]
                );
                break;
            }

            let start_of_cycle_pc = self.pc;

            match self.execute_instruction() {
                Ok(pc_increment) => {
                    if self.pc == start_of_cycle_pc {
                        self.pc += pc_increment;
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn execute_instruction(&mut self) -> Result<u64, ()> {
        let pc = self.pc as usize;
        let instruction = self.memory[pc];

        match instruction {
            OP_HALT => {
                println!("HALT instruction encountered. VM is stopping.");
                Err(())
            }

            OP_ADD | OP_SUB | OP_MUL | OP_DIV => {
                let rd = self.memory[pc + 1] as usize;
                let rs1 = self.memory[pc + 2] as usize;
                let rs2 = self.memory[pc + 3] as usize;
                if rd > 0 {
                    let val1 = self.registers[rs1];
                    let val2 = self.registers[rs2];
                    match instruction {
                        OP_ADD => self.registers[rd] = val1.wrapping_add(val2),
                        OP_SUB => self.registers[rd] = val1.wrapping_sub(val2),
                        OP_MUL => self.registers[rd] = val1.wrapping_mul(val2),
                        OP_DIV => self.registers[rd] = val1.wrapping_div(val2),
                        _ => {}
                    };
                }
                Ok(4)
            }

            OP_ADDI => {
                let rd = self.memory[pc + 1] as usize;
                let rs1 = self.memory[pc + 2] as usize;
                let imm = self.memory[pc + 3] as i8;
                if rd > 0 {
                    let val1 = self.registers[rs1];
                    self.registers[rd] = val1.wrapping_add(imm as i64 as u64);
                }
                Ok(4)
            }

            OP_LW => {
                let rd = self.memory[pc + 1] as usize;
                let base_reg = self.memory[pc + 2] as usize;
                let offset = self.memory[pc + 3] as i8 as i64;
                if rd > 0 {
                    let addr = self.registers[base_reg].wrapping_add(offset as u64) as usize;
                    let bytes: [u8; 8] = self.memory[addr..addr + 8].try_into().unwrap();
                    self.registers[rd] = u64::from_le_bytes(bytes);
                }
                Ok(4)
            }
            OP_SW => {
                let rs = self.memory[pc + 1] as usize;
                let base_reg = self.memory[pc + 2] as usize;
                let offset = self.memory[pc + 3] as i8 as i64;
                let addr = self.registers[base_reg].wrapping_add(offset as u64) as usize;
                let bytes = self.registers[rs].to_le_bytes();
                self.memory[addr..addr + 8].copy_from_slice(&bytes);
                Ok(4)
            }

            OP_LDI => {
                let rd = self.memory[pc + 1] as usize;
                if rd > 0 {
                    let value_addr = pc + 4;
                    let bytes: [u8; 8] =
                        self.memory[value_addr..value_addr + 8].try_into().unwrap();
                    self.registers[rd] = u64::from_le_bytes(bytes);
                }
                Ok(12)
            }

            OP_BEQ => {
                let rs1 = self.memory[pc + 1] as usize;
                let rs2 = self.memory[pc + 2] as usize;
                let offset = self.memory[pc + 3] as i8 as i64;
                if self.registers[rs1] == self.registers[rs2] {
                    self.pc = self.pc.wrapping_add(offset as u64);
                }
                Ok(4)
            }

            OP_JAL => {
                let rd = self.memory[pc + 1] as usize;
                let offset = i16::from_le_bytes([self.memory[pc + 2], self.memory[pc + 3]]) as i64;
                if rd > 0 {
                    self.registers[rd] = self.pc + 4;
                }
                self.pc = self.pc.wrapping_add(offset as u64);
                Ok(4)
            }

            OP_RET => {
                let return_address = self.registers[1];
                if return_address == 0 {
                    println!("RET from main context detected. Halting.");
                    return Err(());
                }
                if return_address == self.pc {
                    println!(
                        "Infinite loop detected (RET to self at PC={:#x}). Halting.",
                        self.pc
                    );
                    return Err(());
                }
                self.pc = return_address;
                Ok(4)
            }

            OP_ECALL => {
                todo!()
            }

            _ => {
                println!(
                    "Unknown instruction: {:#04x} at PC {:#x}. Halting.",
                    instruction, self.pc
                );
                Err(())
            }
        }
    }

    pub fn print_state(&self) {
        println!("\n--- VM execution finished ---");
        println!("Final pc value: {:#018x}", self.pc);
        println!("--- Final Register State ---");
        println!("{:<4} {:<5}  {:<18}", "Reg", "(ABI)", "Value");
        println!("{:-<4} {:-<5}  {:-<18}", "", "", "");

        for i in 0..32 {
            let abi_name = match i {
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
                _ => "",
            };

            let reg_name = format!("x{}", i);
            let abi_part = format!("({})", abi_name);

            println!(
                "{:<4} {:<5}  {:#018x}",
                reg_name, abi_part, self.registers[i]
            );
        }
    }
}
