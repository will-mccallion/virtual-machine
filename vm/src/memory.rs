use crate::VM;
use riscv_core::cause;

pub const MEMORY_SIZE: usize = 1024 * 1024 * 128; // 128MB of physical RAM
pub const CSR_SIZE: usize = 4096;
pub const BASE_ADDRESS: u64 = 0x80000000;

impl VM {
    pub(crate) fn read_csr(&self, addr: u32) -> u64 {
        self.csr[addr as usize]
    }

    pub(crate) fn write_csr(&mut self, addr: u32, value: u64) {
        self.csr[addr as usize] = value;
    }

    pub(crate) fn translate_addr(&self, vaddr: u64) -> Result<usize, u64> {
        // PRIO 6: # TODO: This is direct physical address mapping. A compliant machine requires a proper Memory Management Unit (MMU) that handles virtual-to-physical address translation and enforces memory protection.
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
