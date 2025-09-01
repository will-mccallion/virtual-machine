use crate::{
    csr::{SATP_MODE_SV39, SATP_PPN_MASK},
    VM,
};
use riscv_core::csr;

const PAGE_SIZE: u64 = 4096;
const PTE_SIZE: u64 = 8;
const LEVELS: u64 = 3;

const PTE_VALID: u64 = 1 << 0;
const PTE_READ: u64 = 1 << 1;
const PTE_WRITE: u64 = 1 << 2;
const PTE_EXECUTE: u64 = 1 << 3;
//const PTE_USER: u64 = 1 << 4;

impl VM {
    pub fn translate(&mut self, vaddr: u64, is_write: bool, is_execute: bool) -> Result<u64, u64> {
        let satp = self.csrs.read(csr::SATP, self.privilege_level).unwrap_or(0);
        let mode = satp >> 60;

        if mode != (SATP_MODE_SV39 >> 60) {
            if vaddr < super::memory::BASE_ADDRESS {
                return Err(vaddr);
            }
            let paddr = vaddr - super::memory::BASE_ADDRESS;
            if paddr >= self.memory.len() as u64 {
                return Err(vaddr);
            }
            return Ok(paddr);
        }

        let vpn = vaddr / PAGE_SIZE;
        if let Some(&paddr_base) = self.tlb.get(&vpn) {
            return Ok(paddr_base + (vaddr % PAGE_SIZE));
        }

        let root_ppn = satp & SATP_PPN_MASK;
        let mut table_addr = root_ppn * PAGE_SIZE;

        for level in (0..LEVELS).rev() {
            let vpn_part = (vaddr >> (12 + 9 * level)) & 0x1FF;
            let pte_addr = table_addr + vpn_part * PTE_SIZE;

            if pte_addr < super::memory::BASE_ADDRESS
                || pte_addr >= super::memory::BASE_ADDRESS + self.memory.len() as u64
            {
                return Err(vaddr);
            }
            let pte_offset = (pte_addr - super::memory::BASE_ADDRESS) as usize;

            let pte_bytes: [u8; 8] = self.memory[pte_offset..(pte_offset + PTE_SIZE as usize)]
                .try_into()
                .unwrap();
            let pte = u64::from_le_bytes(pte_bytes);

            if (pte & PTE_VALID) == 0 {
                return Err(vaddr);
            }

            if (pte & (PTE_READ | PTE_WRITE | PTE_EXECUTE)) != 0 {
                if is_write && (pte & PTE_WRITE) == 0 {
                    return Err(vaddr);
                }
                if !is_write && is_execute && (pte & PTE_EXECUTE) == 0 {
                    return Err(vaddr);
                }
                if !is_write && !is_execute && (pte & PTE_READ) == 0 {
                    return Err(vaddr);
                }

                let paddr = match level {
                    // Level 2 -> 1GB Gigapage.
                    // Physical Address = [PTE PPN[2] | vaddr[29:0]]
                    2 => {
                        let pte_ppn2 = (pte >> 28) & 0x3FFFFFF;
                        let vaddr_offset = vaddr & 0x3FFFFFFF;
                        (pte_ppn2 << 30) | vaddr_offset
                    }
                    // Level 1 -> 2MB Megapage.
                    // Physical Address = [PTE PPN[2] | PTE PPN[1] | vaddr[20:0]]
                    1 => {
                        let pte_ppn2 = (pte >> 28) & 0x3FFFFFF;
                        let pte_ppn1 = (pte >> 19) & 0x1FF;
                        let vaddr_offset = vaddr & 0x1FFFFF;
                        (pte_ppn2 << 30) | (pte_ppn1 << 21) | vaddr_offset
                    }
                    // Level 0 -> 4KB Page.
                    // Physical Address = [PTE PPN[2] | PTE PPN[1] | PTE PPN[0] | vaddr[11:0]]
                    _ => {
                        let pte_ppn = (pte >> 10) & SATP_PPN_MASK;
                        let vaddr_offset = vaddr & 0xFFF;
                        (pte_ppn << 12) | vaddr_offset
                    }
                };

                if level == 0 {
                    self.tlb.insert(vpn, paddr - (vaddr % PAGE_SIZE));
                }

                if paddr < super::memory::BASE_ADDRESS {
                    return Err(paddr);
                }
                return Ok(paddr - super::memory::BASE_ADDRESS);
            }

            table_addr = ((pte >> 10) & SATP_PPN_MASK) * PAGE_SIZE;
        }

        Err(vaddr)
    }
}
