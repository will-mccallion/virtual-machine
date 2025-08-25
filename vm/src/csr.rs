use riscv_core::csr;
use std::collections::HashMap;

// For now, this is a simplified mask. A full implementation would be more granular.
const MSTATUS_MASK: u64 = 0x00000003_00001888;
pub const SATP_MODE_SV39: u64 = 8 << 60;
pub const SATP_ASID_MASK: u64 = 0xFFFF << 44;
pub const SATP_PPN_MASK: u64 = (1u64 << 44) - 1;

pub struct CsrFile {
    pub mstatus: u64,
    pub mie: u64,
    pub mip: u64,
    pub mepc: u64,
    pub mcause: u64,
    pub mtval: u64,
    pub mscratch: u64,
    pub mtvec: u64,
    pub satp: u64,
    other_csrs: HashMap<u32, u64>,
}

impl CsrFile {
    pub fn new() -> Self {
        let mut other_csrs = HashMap::new();

        other_csrs.insert(csr::MEDELEG, 0);
        other_csrs.insert(csr::MIDELEG, 0);

        other_csrs.insert(csr::SEDELEG, 0);
        other_csrs.insert(csr::SIDELEG, 0);

        Self {
            mstatus: 0,
            mie: 0,
            mip: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            mscratch: 0,
            mtvec: 0,
            satp: 0,
            other_csrs,
        }
    }

    pub fn read(&self, addr: u32, privilege_level: u8) -> Option<u64> {
        let required_priv = (addr >> 8) & 0x3;
        if privilege_level < required_priv as u8 {
            return None;
        }

        match addr {
            csr::MSTATUS => Some(self.mstatus),
            csr::MIE => Some(self.mie),
            csr::MIP => Some(self.mip),
            csr::MEPC => Some(self.mepc),
            csr::MCAUSE => Some(self.mcause),
            csr::MTVAL => Some(self.mtval),
            csr::MSCRATCH => Some(self.mscratch),
            csr::MTVEC => Some(self.mtvec),
            csr::SATP => Some(self.satp),

            csr::SSTATUS => Some(self.mstatus & MSTATUS_MASK),
            csr::SIE => Some(self.mie & self.read(csr::MIDELEG, 3).unwrap_or(0)),
            csr::SIP => Some(self.mip & self.read(csr::MIDELEG, 3).unwrap_or(0)),

            csr::MHARTID => Some(0),

            _ => self.other_csrs.get(&addr).copied(),
        }
    }

    pub fn write(&mut self, addr: u32, value: u64, privilege_level: u8) -> bool {
        let required_priv = (addr >> 8) & 0x3;
        if privilege_level < required_priv as u8 {
            return false;
        }

        match addr {
            csr::MSTATUS => self.mstatus = value,
            csr::MIE => self.mie = value,
            csr::MIP => self.mip = value,
            csr::MEPC => self.mepc = value,
            csr::MCAUSE => self.mcause = value,
            csr::MTVAL => self.mtval = value,
            csr::MSCRATCH => self.mscratch = value,
            csr::MTVEC => self.mtvec = value,
            csr::SATP => self.satp = value,

            csr::SSTATUS => {
                let new_mstatus = (self.mstatus & !MSTATUS_MASK) | (value & MSTATUS_MASK);
                self.mstatus = new_mstatus;
            }
            csr::SIE => {
                let mideleg = self.read(csr::MIDELEG, 3).unwrap_or(0);
                self.mie = (self.mie & !mideleg) | (value & mideleg);
            }
            csr::SIP => {
                let mideleg = self.read(csr::MIDELEG, 3).unwrap_or(0);
                self.mip = (self.mip & !mideleg) | (value & mideleg);
            }

            _ => {
                self.other_csrs.insert(addr, value);
            }
        }
        true
    }
}
