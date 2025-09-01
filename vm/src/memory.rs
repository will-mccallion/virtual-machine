use crate::VM;
use riscv_core::cause;

pub const MEMORY_SIZE: usize = 1024 * 1024 * 128; // 128MB of physical RAM
pub const BASE_ADDRESS: u64 = 0x80000000;
pub const VIRTUAL_DISK_ADDRESS: u64 = 0x90000000;
pub const VIRTUAL_DISK_SIZE_ADDRESS: u64 = 0x90001000;

impl VM {
    pub(crate) fn fetch(&mut self) -> Option<u32> {
        let satp = self
            .csrs
            .read(riscv_core::csr::SATP, self.privilege_level)
            .unwrap_or(0);
        let mmu_is_on = (satp >> 60) == (crate::csr::SATP_MODE_SV39 >> 60);

        let paddr = match self.translate(self.pc, false, true) {
            Ok(addr) => addr,
            Err(fault_addr) => {
                if mmu_is_on {
                    self.handle_trap(cause::INSTRUCTION_PAGE_FAULT, fault_addr);
                } else {
                    self.handle_trap(cause::INSTRUCTION_ACCESS_FAULT, fault_addr);
                }
                return None;
            }
        };

        if paddr.checked_add(3).is_none() || paddr + 3 >= self.memory.len() as u64 {
            self.handle_trap(cause::INSTRUCTION_ACCESS_FAULT, self.pc);
            return None;
        }

        let inst_bytes: [u8; 4] = self.memory[paddr as usize..(paddr + 4) as usize]
            .try_into()
            .unwrap();
        Some(u32::from_le_bytes(inst_bytes))
    }
}
