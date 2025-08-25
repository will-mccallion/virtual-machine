pub mod csr;
pub mod execution;
pub mod memory;
pub mod trap;

use crate::csr::CsrFile;
use crate::memory::{BASE_ADDRESS, MEMORY_SIZE};
use assembler::disassemble;
use riscv_core::csr as rv_csrs;

#[derive(Default)]
pub struct VmConfig {
    pub trace: bool,
}

pub struct VM {
    pub registers: [u64; 32],
    pub pc: u64,
    pub memory: Vec<u8>,
    pub csrs: CsrFile,
    pub privilege_level: u8,
    pub config: VmConfig,
    pub virtual_disk: Vec<u8>,
}

impl VM {
    pub fn new_config(config: VmConfig) -> Self {
        Self {
            registers: [0; 32],
            pc: BASE_ADDRESS,
            memory: vec![0; MEMORY_SIZE],
            csrs: CsrFile::new(),
            privilege_level: 3,
            config,
            virtual_disk: Vec::new(),
        }
    }

    pub fn new() -> Self {
        VM::new_config(VmConfig::default())
    }

    pub fn load_bios(&mut self, bios_bytes: &[u8]) {
        self.memory[0..bios_bytes.len()].copy_from_slice(bios_bytes);
    }

    pub fn load_virtual_disk(&mut self, disk_bytes: Vec<u8>) {
        self.virtual_disk = disk_bytes;
    }

    pub fn run(&mut self) -> Result<(), String> {
        const INSTRUCTION_LIMIT: u64 = 5_000_000;

        for _ in 0..INSTRUCTION_LIMIT {
            let pc_before_fetch = self.pc;

            let instruction = match self.fetch() {
                Some(inst) => inst,
                None => {
                    let cause = self
                        .csrs
                        .read(riscv_core::csr::MCAUSE, self.privilege_level)
                        .unwrap_or(0);
                    if self.is_exit_ecall(cause) {
                        return Ok(());
                    } else {
                        return Err(format!(
                            "Execution halted by trap: {}",
                            self.cause_to_string(cause)
                        ));
                    }
                }
            };

            if self.config.trace {
                let disassembled_text = disassemble(instruction, pc_before_fetch);
                eprintln!("TRACE: 0x{:016x}: {}", pc_before_fetch, disassembled_text);
            }

            if !self.execute(instruction) {
                let cause = self
                    .csrs
                    .read(riscv_core::csr::MCAUSE, self.privilege_level)
                    .unwrap_or(0);
                if self.is_exit_ecall(cause) {
                    return Ok(());
                } else {
                    return Err(format!(
                        "Execution halted by trap: {}",
                        self.cause_to_string(cause)
                    ));
                }
            }
        }

        Err("Instruction limit reached. Program may be in an infinite loop.".to_string())
    }

    fn is_exit_ecall(&self, cause: u64) -> bool {
        if cause == riscv_core::cause::ECALL_FROM_M_MODE {
            if self.registers[riscv_core::abi::A7 as usize] == 93 {
                return true;
            }
        }
        false
    }

    pub fn print_state(&self) {
        let key_csrs_to_print = [
            (rv_csrs::MSTATUS, "mstatus"),
            (rv_csrs::MISA, "misa"),
            (rv_csrs::MIE, "mie"),
            (rv_csrs::MTVEC, "mtvec"),
            (rv_csrs::MSCRATCH, "mscratch"),
            (rv_csrs::MEPC, "mepc"),
            (rv_csrs::MCAUSE, "mcause"),
            (rv_csrs::MTVAL, "mtval"),
            (rv_csrs::MIP, "mip"),
            (rv_csrs::SSTATUS, "sstatus"),
            (rv_csrs::SIE, "sie"),
            (rv_csrs::STVEC, "stvec"),
            (rv_csrs::SSCRATCH, "sscratch"),
            (rv_csrs::SEPC, "sepc"),
            (rv_csrs::SCAUSE, "scause"),
            (rv_csrs::STVAL, "stval"),
            (rv_csrs::SIP, "sip"),
            (rv_csrs::SATP, "satp"),
        ];
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
        let gpr_header = format!("{:<5} {:<7} {:<18}", "Reg", "ABI", "Value");
        let csr_header = format!("{:<8} {:<10} {:<18}", "Address", "Name", "Value");
        let seperator = " | ";
        println!(
            "{} {} {}",
            " --- General Purpose Registers --- ",
            seperator,
            " --- Control & Status Registers --- "
        );
        println!("------------------------------------------------------------------");
        println!("{}{}{}", gpr_header, seperator, csr_header);
        println!("------------------------------------------------------------------");
        for i in 0..32 {
            let reg_name = format!("x{}", i);
            let abi_name = format!("{}", abi[i]);
            let gpr_line = format!(
                "{:<5} {:<7} {:#018x}",
                reg_name, abi_name, self.registers[i]
            );

            if i < key_csrs_to_print.len() {
                let (addr, name) = key_csrs_to_print[i];
                let csr_val = self.csrs.read(addr, 3).unwrap_or(0);
                let csr_line = format!("{:<#8x} {:<10} {:#018x}", addr, name, csr_val);
                println!("{}{}{}", gpr_line, seperator, csr_line);
            } else {
                println!("{}{}", gpr_line, seperator);
            }
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
