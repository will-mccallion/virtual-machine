const MEMORY_SIZE: usize = 1024;

// Opcodes for RISC-V-like instruction set
pub const OP_HALT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUB: u8 = 0x02;
pub const OP_ADDI: u8 = 0x03;

pub struct VM {
    pub registers: [u8; 32],
    pub pc: usize,
    pub memory: [u8; MEMORY_SIZE],
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
            if self.pc >= MEMORY_SIZE {
                println!("Program Counter exceeded memory bounds. Halting.");
                break;
            }

            let instruction = self.memory[self.pc];
            self.pc += 1;

            if !self.execute_instruction(instruction) {
                break;
            }
        }
    }

    fn execute_instruction(&mut self, instruction: u8) -> bool {
        match instruction {
            OP_HALT => {
                println!("HALT instruction encountered. VM is stopping.");
                return false;
            }

            OP_ADD => {
                // Syntax: add rd, rs1, rs2
                let rd = self.memory[self.pc] as usize;
                let rs1 = self.memory[self.pc + 1] as usize;
                let rs2 = self.memory[self.pc + 2] as usize;
                self.pc += 3;

                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];

                // Don't write to register 0
                if rd > 0 {
                    self.registers[rd] = val1.wrapping_add(val2);
                }
            }

            OP_SUB => {
                // Syntax: sub rd, rs1, rs2
                let rd = self.memory[self.pc] as usize;
                let rs1 = self.memory[self.pc + 1] as usize;
                let rs2 = self.memory[self.pc + 2] as usize;
                self.pc += 3;

                let val1 = self.registers[rs1];
                let val2 = self.registers[rs2];

                // Don't write to register 0
                if rd > 0 {
                    self.registers[rd] = val1.wrapping_sub(val2);
                }
            }

            OP_ADDI => {
                // Syntax: addi rd, rs1, imm
                let rd = self.memory[self.pc] as usize;
                let rs1 = self.memory[self.pc + 1] as usize;
                let imm = self.memory[self.pc + 2];
                self.pc += 3;

                let val1 = self.registers[rs1];

                // Don't write to register 0
                if rd > 0 {
                    self.registers[rd] = val1.wrapping_add(imm);
                }
            }

            _ => {
                println!(
                    "Unknown instruction: {:#04x} at PC {}. Halting.",
                    instruction,
                    self.pc - 1
                );
                return false;
            }
        }
        true
    }
}
