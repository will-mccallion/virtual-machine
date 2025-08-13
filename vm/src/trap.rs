use crate::VM;
use riscv_core::{abi, cause, csr};
use std::process::exit;

impl VM {
    pub(crate) fn handle_trap(&mut self, cause: u64, tval: u64) {
        self.write_csr(csr::MEPC, self.pc);

        self.write_csr(csr::MCAUSE, cause);
        self.write_csr(csr::MTVAL, tval);

        match cause {
            cause::ECALL_FROM_U_MODE | cause::ECALL_FROM_S_MODE | cause::ECALL_FROM_M_MODE => {
                let syscall_num = self.registers[abi::A7 as usize];
                match syscall_num {
                    93 => {
                        let exit_code = self.registers[abi::A0 as usize];
                        println!("\n--- ECALL: Exit = {} ---", exit_code as i32);
                        self.print_state();
                        exit(exit_code as i32);
                    }
                    // PRIO 6: # TODO: Add other syscall numbers here (e.g., for read, write).
                    _ => {
                        println!("--- Unimplemented Syscall: a7={} ---", syscall_num);
                        self.handle_trap(cause::ILLEGAL_INSTRUCTION, syscall_num);
                    }
                }
                let return_pc = self.read_csr(csr::MEPC).wrapping_add(4);
                self.write_csr(csr::MEPC, return_pc);
            }
            _ => {
                println!("\n--- Unhandled Trap ---");
                println!("Cause: {} ({})", cause, self.cause_to_string(cause));
                println!("Trap Value (mtval): {:#x}", tval);
                self.print_state();
                exit(1);
            }
        }

        // PRIO 3: # TODO: In a real machine, we would now jump to the trap handler address
        // specified in the `mtvec` CSR. For this simple implementation, we either exit or halt.
        // self.pc = self.read_csr(csr::MTVEC);
    }

    pub(crate) fn cause_to_string(&self, cause: u64) -> &str {
        match cause {
            cause::INSTRUCTION_ADDRESS_MISALIGNED => "Instruction Address Misaligned",
            cause::INSTRUCTION_ACCESS_FAULT => "Instruction Access Fault",
            cause::ILLEGAL_INSTRUCTION => "Illegal Instruction",
            cause::BREAKPOINT => "Breakpoint",
            cause::LOAD_ADDRESS_MISALIGNED => "Load Address Misaligned",
            cause::LOAD_ACCESS_FAULT => "Load Access Fault",
            cause::STORE_AMO_ADDRESS_MISALIGNED => "Store/AMO Address Misaligned",
            cause::STORE_AMO_ACCESS_FAULT => "Store/AMO Access Fault",
            cause::ECALL_FROM_U_MODE => "Environment Call from U-mode",
            cause::ECALL_FROM_S_MODE => "Environment Call from S-mode",
            cause::ECALL_FROM_M_MODE => "Environment Call from M-mode",
            _ => "Unknown",
        }
    }
}
