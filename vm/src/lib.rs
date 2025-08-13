pub mod execution;
pub mod memory;
pub mod trap;

use assembler::Executable;
use riscv_core::abi;

use crate::memory::{BASE_ADDRESS, CSR_SIZE, MEMORY_SIZE};

pub struct VM {
    pub registers: [u64; 32],
    pub pc: u64,
    pub memory: Vec<u8>,
    pub csr: [u64; CSR_SIZE],
    pub privilege_level: u8, // 0: User, 1: Supervisor, 3: Machine
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            registers: [0; 32],
            pc: BASE_ADDRESS,
            memory: vec![0; MEMORY_SIZE],
            csr: [0; CSR_SIZE],
            privilege_level: 3, // Start in Machine mode
        };
        // PRIO 5: # TODO: The stack pointer should be initialized based on information from the executable (e.g., ELF headers) or command-line arguments, not just to the end of physical memory.
        vm.registers[abi::SP as usize] = BASE_ADDRESS + MEMORY_SIZE as u64;
        vm
    }

    pub fn load_executable(&mut self, executable: &Executable) -> Result<(), String> {
        // PRIO 5: # TODO: Implement loading of other common ELF sections, especially `.bss` (which should be allocated and zero-initialized in memory).
        let text_size = executable.text.len();
        let data_size = executable.data.len();

        let text_start = (BASE_ADDRESS - BASE_ADDRESS) as usize;
        let data_start = text_start + text_size;

        if data_start + data_size > self.memory.len() {
            return Err(format!(
                "Executable too large: text_size={}, data_size={}",
                text_size, data_size
            ));
        }

        self.memory[text_start..text_start + text_size].copy_from_slice(&executable.text);
        self.memory[data_start..data_start + data_size].copy_from_slice(&executable.data);

        self.pc = BASE_ADDRESS;
        Ok(())
    }

    pub fn run(&mut self) {
        // PRIO 8: # TODO: Add interrupt checking to the main loop. Before fetching, check if `(MIP & MIE) != 0`. If so, trigger the appropriate interrupt trap.
        loop {
            if let Some(instruction) = self.fetch() {
                self.execute(instruction);
            }
        }
    }

    pub fn print_state(&self) {
        let abi = [
            "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3",
            "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
            "t3", "t4", "t5", "t6",
        ];
        println!();
        println!("---------------------------------");
        println!("\n--- VM State ---");
        println!("---------------------------------");
        println!("PC: {:#018x}", self.pc);
        println!("Privilege Level: {}", self.privilege_level_to_string());
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

    fn privilege_level_to_string(&self) -> &str {
        match self.privilege_level {
            0 => "User",
            1 => "Supervisor",
            3 => "Machine",
            _ => "Unknown",
        }
    }
}
