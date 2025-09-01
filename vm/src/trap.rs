use crate::VM;
use assembler::disassemble;
use riscv_core::{abi, cause, csr};
use std::io::{self, Write};

impl VM {
    pub(crate) fn handle_trap(&mut self, cause: u64, tval: u64) -> bool {
        let is_interrupt = (cause >> 63) & 1 == 1;
        let return_pc = if is_interrupt { self.pc } else { self.pc + 4 };

        match self.privilege_level {
            1 | 0 => {
                self.csrs.write(csr::SEPC, return_pc, self.privilege_level);
                self.csrs.write(csr::SCAUSE, cause, self.privilege_level);
                self.csrs.write(csr::STVAL, tval, self.privilege_level);
            }
            3 => {
                self.csrs.write(csr::MEPC, return_pc, self.privilege_level);
                self.csrs.write(csr::MCAUSE, cause, self.privilege_level);
                self.csrs.write(csr::MTVAL, tval, self.privilege_level);
            }
            _ => unreachable!(),
        }

        if is_interrupt {
            self.handle_interrupt(cause)
        } else {
            self.handle_exception(cause, tval)
        }
    }

    fn handle_exception(&mut self, exception_code: u64, tval: u64) -> bool {
        match exception_code {
            cause::ECALL_FROM_U_MODE | cause::ECALL_FROM_S_MODE | cause::ECALL_FROM_M_MODE => {
                let syscall_num = self.registers[abi::A7 as usize];
                match syscall_num {
                    93 => {
                        let exit_code = self.registers[abi::A0 as usize];
                        println!("\n\n--- ECALL: Exit with code {} --- \n", exit_code as i32);
                        self.print_state();
                        println!("");
                        return false;
                    }
                    _ => {
                        println!("--- Unimplemented Syscall: a7={} ---", syscall_num);
                        return self.handle_exception(cause::ILLEGAL_INSTRUCTION, syscall_num);
                    }
                }
            }

            cause::ILLEGAL_INSTRUCTION => {
                println!("\n--- FATAL EXCEPTION: Illegal Instruction ---");
                let instruction_word = tval as u32;
                let disassembled_text = disassemble(instruction_word, self.pc);
                println!("Failing Instruction: '{}'", disassembled_text);
                println!("Instruction Word (mtval): {:#010x}", instruction_word);
                return false;
            }

            cause::INSTRUCTION_ADDRESS_MISALIGNED => {
                println!("\n--- FATAL EXCEPTION: Instruction Address Misaligned ---");
                println!("PC (mepc): {:#x}", self.pc);
                println!("Misaligned Address (mtval): {:#x}", tval);
                return false;
            }

            cause::INSTRUCTION_ACCESS_FAULT
            | cause::LOAD_ACCESS_FAULT
            | cause::STORE_AMO_ACCESS_FAULT => {
                println!("\n--- FATAL EXCEPTION: Access Fault ---");
                println!(
                    "Cause: {} ({})",
                    exception_code,
                    self.cause_to_string(exception_code)
                );
                println!("Faulting Address (mtval): {:#x}", tval);
                println!("Instruction PC (mepc): {:#x}", self.pc);
                return false;
            }

            cause::LOAD_ADDRESS_MISALIGNED | cause::STORE_AMO_ADDRESS_MISALIGNED => {
                println!("\n--- FATAL EXCEPTION: Address Misaligned ---");
                println!(
                    "Cause: {} ({})",
                    exception_code,
                    self.cause_to_string(exception_code)
                );
                println!("Misaligned Address (mtval): {:#x}", tval);
                println!("Instruction PC (mepc): {:#x}", self.pc);
                return false;
            }

            cause::BREAKPOINT => {
                println!("\n--- BREAKPOINT ---");
                println!("Breakpoint at PC: {:#x}", self.pc);
                self.print_state();
                self.pc += 4;

                print!("Press Enter to continue...");
                io::stdout().flush().unwrap();
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).unwrap();

                return true;
            }

            cause::INSTRUCTION_PAGE_FAULT
            | cause::LOAD_PAGE_FAULT
            | cause::STORE_AMO_PAGE_FAULT => {
                println!("\n--- PAGE FAULT ---");
                println!(
                    "Cause: {} ({})",
                    exception_code,
                    self.cause_to_string(exception_code)
                );
                println!("Faulting Virtual Address (mtval): {:#x}", tval);
                println!("Instruction PC (mepc): {:#x}", self.pc);
                return false;
            }

            _ => {
                println!("\n--- UNHANDLED EXCEPTION ---");
                println!(
                    "Cause: {} ({})",
                    exception_code,
                    self.cause_to_string(exception_code)
                );
                println!("Trap Value (mtval): {:#x}", tval);
                return false;
            }
        }
    }

    fn handle_interrupt(&mut self, cause: u64) -> bool {
        let interrupt_type = cause & 0xfff;

        match interrupt_type {
            cause::MACHINE_TIMER_INTERRUPT => {
                let mut mip = self.csrs.read(csr::MIP, self.privilege_level).unwrap();
                mip |= 1 << (cause::SUPERVISOR_SOFTWARE_INTERRUPT & 0xfff);
                self.csrs.write(csr::MIP, mip, self.privilege_level);

                mip &= !(1 << (cause::MACHINE_TIMER_INTERRUPT & 0xfff));
                self.csrs.write(csr::MIP, mip, self.privilege_level);
            }

            cause::MACHINE_EXTERNAL_INTERRUPT => {
                let mut mip = self.csrs.read(csr::MIP, self.privilege_level).unwrap();
                mip |= 1 << (cause::SUPERVISOR_EXTERNAL_INTERRUPT & 0xfff);
                self.csrs.write(csr::MIP, mip, self.privilege_level);
            }

            cause::MACHINE_SOFTWARE_INTERRUPT => {
                let mut mip = self.csrs.read(csr::MIP, self.privilege_level).unwrap();
                mip &= !(1 << (cause::MACHINE_SOFTWARE_INTERRUPT & 0xfff));
                mip |= 1 << (cause::SUPERVISOR_SOFTWARE_INTERRUPT & 0xfff);
                self.csrs.write(csr::MIP, mip, self.privilege_level);
            }

            cause::SUPERVISOR_TIMER_INTERRUPT => {
                let mut sip = self.csrs.read(csr::SIP, self.privilege_level).unwrap();
                sip |= 1 << interrupt_type;
                self.csrs.write(csr::SIP, sip, self.privilege_level);
            }
            cause::SUPERVISOR_SOFTWARE_INTERRUPT => {
                let mut sip = self.csrs.read(csr::SIP, self.privilege_level).unwrap();
                sip |= 1 << interrupt_type;
                self.csrs.write(csr::SIP, sip, self.privilege_level);
            }
            cause::SUPERVISOR_EXTERNAL_INTERRUPT => {
                let mut sip = self.csrs.read(csr::SIP, self.privilege_level).unwrap();
                sip |= 1 << interrupt_type;
                self.csrs.write(csr::SIP, sip, self.privilege_level);
            }

            cause::USER_TIMER_INTERRUPT
            | cause::USER_SOFTWARE_INTERRUPT
            | cause::USER_EXTERNAL_INTERRUPT => {
                println!(
                    "\n--- Ignoring User-level Interrupt: {} ---",
                    self.cause_to_string(cause)
                );
            }

            _ => {
                println!("\n--- Unhandled Interrupt ---");
                println!("Cause: {} ({})", cause, self.cause_to_string(cause));
            }
        }
        true
    }

    pub(crate) fn cause_to_string(&self, cause: u64) -> &str {
        let is_interrupt = (cause >> 63) & 1 == 1;
        let code = cause & 0xfff;

        if is_interrupt {
            match code {
                cause::USER_SOFTWARE_INTERRUPT => "User Software Interrupt",
                cause::SUPERVISOR_SOFTWARE_INTERRUPT => "Supervisor Software Interrupt",
                cause::MACHINE_SOFTWARE_INTERRUPT => "Machine Software Interrupt",
                cause::USER_TIMER_INTERRUPT => "User Timer Interrupt",
                cause::SUPERVISOR_TIMER_INTERRUPT => "Supervisor Timer Interrupt",
                cause::MACHINE_TIMER_INTERRUPT => "Machine Timer Interrupt",
                cause::USER_EXTERNAL_INTERRUPT => "User External Interrupt",
                cause::SUPERVISOR_EXTERNAL_INTERRUPT => "Supervisor External Interrupt",
                cause::MACHINE_EXTERNAL_INTERRUPT => "Machine External Interrupt",
                _ => "Unknown Interrupt",
            }
        } else {
            match code {
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
                cause::INSTRUCTION_PAGE_FAULT => "Instruction Page Fault",
                cause::LOAD_PAGE_FAULT => "Load Page Fault",
                cause::STORE_AMO_PAGE_FAULT => "Store/AMO Page Fault",
                _ => "Unknown Exception",
            }
        }
    }
}
