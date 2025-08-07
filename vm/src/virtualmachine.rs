const MEMORY_SIZE: usize = 1024;

// Opcodes for RISC-V-like instruction set
pub const OP_HALT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUB: u8 = 0x02;
pub const OP_ADDI: u8 = 0x03;
pub const OP_BEQ: u8 = 0x04;
pub const OP_JAL: u8 = 0x05;
pub const OP_LW: u8 = 0x06;
pub const OP_SW: u8 = 0x07;
pub const OP_RET: u8 = 0x08;

pub struct VM {
    registers: [u8; 32],
    pc: usize,
    memory: [u8; MEMORY_SIZE],
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            pc: 0,
            memory: [0; MEMORY_SIZE],
        }
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
            if self.pc >= MEMORY_SIZE - 4 {
                println!("Program Counter near memory bounds. Halting.");
                break;
            }

            let start_of_cycle_pc = self.pc;

            if !self.execute_instruction() {
                break;
            }

            if self.pc == start_of_cycle_pc {
                self.pc += 4;
            }
        }
    }

    fn execute_instruction(&mut self) -> bool {
        let instruction = self.memory[self.pc];

        match instruction {
            OP_HALT => {
                println!("HALT instruction encountered. VM is stopping.");
                return false;
            }

            OP_ADD | OP_SUB => {
                let rd = self.memory[self.pc + 1] as usize;
                let rs1 = self.memory[self.pc + 2] as usize;
                let rs2 = self.memory[self.pc + 3] as usize;

                if rd > 0 {
                    let val1 = self.registers[rs1];
                    let val2 = self.registers[rs2];
                    if instruction == OP_ADD {
                        self.registers[rd] = val1.wrapping_add(val2);
                    } else {
                        self.registers[rd] = val1.wrapping_sub(val2);
                    }
                }
            }

            OP_ADDI => {
                let rd = self.memory[self.pc + 1] as usize;
                let rs1 = self.memory[self.pc + 2] as usize;
                let imm = self.memory[self.pc + 3] as i8;

                if rd > 0 {
                    let val1 = self.registers[rs1];
                    self.registers[rd] = val1.wrapping_add_signed(imm);
                }
            }

            OP_BEQ => {
                let rs1 = self.memory[self.pc + 1] as usize;
                let rs2 = self.memory[self.pc + 2] as usize;
                let offset_byte = self.memory[self.pc + 3] as i8;

                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];

                if val1 == val2 {
                    self.pc = self.pc.wrapping_add_signed(offset_byte as isize);
                }
            }

            OP_JAL => {
                let rd = self.memory[self.pc + 1] as usize;
                let offset_low = self.memory[self.pc + 2];
                let offset_high = self.memory[self.pc + 3];

                let offset = i16::from_le_bytes([offset_low, offset_high]);

                if rd > 0 {
                    self.registers[rd] = (self.pc + 4) as u8;
                }

                self.pc = self.pc.wrapping_add_signed(offset as isize);
            }

            OP_LW => {
                let rd = self.memory[self.pc + 1] as usize;
                let base = self.memory[self.pc + 2] as usize;
                let offset = self.memory[self.pc + 3] as i8;
                if rd > 0 {
                    let addr = self.registers[base].wrapping_add_signed(offset as i8);
                    self.registers[rd] = self.memory[addr as usize];
                }
            }
            OP_SW => {
                let rs = self.memory[self.pc + 1] as usize;
                let base = self.memory[self.pc + 2] as usize;
                let offset = self.memory[self.pc + 3] as i8;
                let addr = self.registers[base].wrapping_add_signed(offset as i8);
                self.memory[addr as usize] = self.registers[rs];
            }
            OP_RET => {
                self.pc = self.registers[1] as usize;
            }

            _ => {
                println!(
                    "Unknown instruction: {:#04x} at PC {:#04x}. Halting.",
                    instruction, self.pc
                );
                return false;
            }
        }
        true
    }

    pub fn print_state(&self) {
        println!("\n--- VM execution finished ---");
        println!("Final pc value: {:#04x}", self.pc);
        println!("--- Final Register State ---");
        println!("{:<4} {:<5}  {:<6}", "Reg", "(ABI)", "Value");
        println!("{:-<4} {:-<5}  {:-<6}", "", "", ""); // Underline for the header

        for i in 0..32 {
            let abi_name = match i {
                0 => "zero",
                1 => "ra",
                2 => "sp",
                5 => "t0",
                6 => "t1",
                7 => "t2",
                8 => "s0",
                9 => "s1",
                10 => "a0",
                11 => "a1",
                _ => "",
            };

            let reg_name = format!("x{}", i);

            let abi_part = if !abi_name.is_empty() {
                format!("({})", abi_name)
            } else {
                String::new()
            };

            println!(
                "{:<4} {:<5}  {:#04x}",
                reg_name, abi_part, self.registers[i]
            );
        }
    }
}
