use crate::VM;
use riscv_core::cause;

pub const MEMORY_SIZE: usize = 1024 * 1024 * 128; // 128MB of physical RAM
pub const BASE_ADDRESS: u64 = 0x80000000;
pub const VIRTUAL_DISK_ADDRESS: u64 = 0x90000000;
pub const VIRTUAL_DISK_SIZE_ADDRESS: u64 = 0x90001000;

impl VM {
    pub fn translate_addr(&self, vaddr: u64) -> Result<usize, u64> {
        if vaddr < BASE_ADDRESS {
            return Err(vaddr);
        }
        let paddr = (vaddr - BASE_ADDRESS) as usize;
        if paddr >= self.memory.len() {
            Err(vaddr)
        } else {
            Ok(paddr)
        }
    }

    pub(crate) fn fetch(&mut self) -> Option<u32> {
        if self.pc % 4 != 0 {
            self.handle_trap(cause::INSTRUCTION_ADDRESS_MISALIGNED, self.pc);
            return None;
        }

        let pc_phys = match self.translate_addr(self.pc) {
            Ok(addr) => addr,
            Err(fault_addr) => {
                self.handle_trap(cause::INSTRUCTION_ACCESS_FAULT, fault_addr);
                return None;
            }
        };

        if pc_phys.checked_add(3).is_none() || pc_phys + 3 >= self.memory.len() {
            self.handle_trap(cause::INSTRUCTION_ACCESS_FAULT, self.pc);
            return None;
        }

        let inst_bytes: [u8; 4] = self.memory[pc_phys..pc_phys + 4].try_into().unwrap();
        Some(u32::from_le_bytes(inst_bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::{memory::BASE_ADDRESS, VM};
    use riscv_core::{cause, csr};

    #[test]
    fn test_translate_addr_success() {
        let vm = VM::new();
        assert_eq!(vm.translate_addr(BASE_ADDRESS).unwrap(), 0);
        let vaddr = BASE_ADDRESS + 1024;
        assert_eq!(vm.translate_addr(vaddr).unwrap(), 1024);
        let vaddr_end = BASE_ADDRESS + (super::MEMORY_SIZE as u64) - 1;
        assert_eq!(
            vm.translate_addr(vaddr_end).unwrap(),
            super::MEMORY_SIZE - 1
        );
    }

    #[test]
    fn test_translate_addr_failure() {
        let vm = VM::new();
        let below_base = BASE_ADDRESS - 1;
        assert_eq!(vm.translate_addr(below_base), Err(below_base));
        let boundary = BASE_ADDRESS + super::MEMORY_SIZE as u64;
        assert_eq!(vm.translate_addr(boundary), Err(boundary));
    }

    #[test]
    fn test_fetch_success() {
        let mut vm = VM::new();
        vm.pc = BASE_ADDRESS;
        let instruction: u32 = 0xFEEDC0DE;
        let paddr = vm.translate_addr(vm.pc).unwrap();
        vm.memory[paddr..paddr + 4].copy_from_slice(&instruction.to_le_bytes());

        let fetched_inst = vm.fetch().expect("Fetch should succeed");
        assert_eq!(fetched_inst, instruction);
    }

    #[test]
    fn test_fetch_misaligned() {
        let mut vm = VM::new();
        vm.pc = BASE_ADDRESS + 2;

        let result = vm.fetch();
        assert!(result.is_none(), "Fetch should fail for misaligned PC");

        assert_eq!(
            vm.csrs.read(csr::MCAUSE, vm.privilege_level).unwrap(),
            cause::INSTRUCTION_ADDRESS_MISALIGNED
        );
        assert_eq!(vm.csrs.read(csr::MTVAL, vm.privilege_level).unwrap(), vm.pc);
    }

    #[test]
    fn test_fetch_access_fault() {
        let mut vm = VM::new();
        vm.pc = BASE_ADDRESS - 4;

        let result = vm.fetch();
        assert!(result.is_none(), "Fetch should fail for out-of-bounds PC");

        assert_eq!(
            vm.csrs.read(csr::MCAUSE, vm.privilege_level).unwrap(),
            cause::INSTRUCTION_ACCESS_FAULT
        );
        assert_eq!(vm.csrs.read(csr::MTVAL, vm.privilege_level).unwrap(), vm.pc);
    }

    #[test]
    fn test_fetch_at_memory_boundary() {
        let mut vm = VM::new();
        vm.pc = BASE_ADDRESS + (super::MEMORY_SIZE as u64) - 4;
        let instruction: u32 = 0x12345678;
        let paddr = vm.translate_addr(vm.pc).unwrap();
        vm.memory[paddr..paddr + 4].copy_from_slice(&instruction.to_le_bytes());

        let result = vm.fetch();
        assert!(result.is_some(), "Fetch at memory boundary should succeed");
        assert_eq!(result.unwrap(), instruction);

        vm.pc = BASE_ADDRESS + (super::MEMORY_SIZE as u64) - 2;
        let result_fail = vm.fetch();
        assert!(
            result_fail.is_none(),
            "Fetch crossing memory boundary should fail"
        );
        assert_eq!(
            vm.csrs.read(csr::MCAUSE, vm.privilege_level).unwrap(),
            cause::INSTRUCTION_ADDRESS_MISALIGNED
        );
    }
}
