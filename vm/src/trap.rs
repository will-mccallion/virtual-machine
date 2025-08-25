use crate::VM;
use assembler::disassemble;
use riscv_core::{abi, cause, csr};

impl VM {
    pub(crate) fn handle_trap(&mut self, cause: u64, tval: u64) -> bool {
        self.csrs.write(csr::MEPC, self.pc, self.privilege_level);
        self.csrs.write(csr::MCAUSE, cause, self.privilege_level);
        self.csrs.write(csr::MTVAL, tval, self.privilege_level);

        match cause {
            cause::ECALL_FROM_U_MODE | cause::ECALL_FROM_S_MODE | cause::ECALL_FROM_M_MODE => {
                let syscall_num = self.registers[abi::A7 as usize];
                match syscall_num {
                    93 => {
                        let exit_code = self.registers[abi::A0 as usize];
                        println!("\n--- ECALL: Exit with code {} ---", exit_code as i32);
                        self.print_state();
                        return false;
                    }
                    _ => {
                        println!("--- Unimplemented Syscall: a7={} ---", syscall_num);
                        return self.handle_trap(cause::ILLEGAL_INSTRUCTION, syscall_num);
                    }
                }
            }

            cause::ILLEGAL_INSTRUCTION => {
                println!("\n--- Unhandled Trap ---");
                println!("Cause: {} (Illegal Instruction)", cause);

                let instruction_word = tval as u32;
                let pc = self.pc;
                let disassembled_text = disassemble(instruction_word, pc);

                println!("Failing Instruction: '{}'", disassembled_text);
                println!("Instruction Word (mtval): {:#010x}", instruction_word);

                self.print_state();
                return false;
            }

            _ => {
                println!("\n--- Unhandled Trap ---");
                println!("Cause: {} ({})", cause, self.cause_to_string(cause));
                println!("Trap Value (mtval): {:#x}", tval);
                self.print_state();
                return false;
            }
        }
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
