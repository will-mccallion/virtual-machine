use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod opcodes {
    pub const OP_LOAD: u32 = 0b0000011;
    pub const OP_MISC_MEM: u32 = 0b0001111;
    pub const OP_IMM: u32 = 0b0010011;
    pub const OP_AUIPC: u32 = 0b0010111;
    pub const OP_IMM_32: u32 = 0b0011011;
    pub const OP_STORE: u32 = 0b0100011;
    pub const OP_REG: u32 = 0b0110011;
    pub const OP_LUI: u32 = 0b0110111;
    pub const OP_REG_32: u32 = 0b0111011;
    pub const OP_BRANCH: u32 = 0b1100011;
    pub const OP_JALR: u32 = 0b1100111;
    pub const OP_JAL: u32 = 0b1101111;
    pub const OP_SYSTEM: u32 = 0b1110011;
    pub const OP_AMO: u32 = 0b0101111;
    pub const OP_LOAD_FP: u32 = 0b0000111;
    pub const OP_STORE_FP: u32 = 0b0100111;
    pub const OP_FP: u32 = 0b1010011;
    pub const OP_MADD: u32 = 0b1000011;
    pub const OP_MSUB: u32 = 0b1000111;
    pub const OP_NMSUB: u32 = 0b1001011;
    pub const OP_NMADD: u32 = 0b1001111;
}

pub mod funct3 {
    pub const LB: u32 = 0b000;
    pub const LH: u32 = 0b001;
    pub const LW: u32 = 0b010;
    pub const LD: u32 = 0b011;
    pub const LBU: u32 = 0b100;
    pub const LHU: u32 = 0b101;
    pub const LWU: u32 = 0b110;

    pub const SB: u32 = 0b000;
    pub const SH: u32 = 0b001;
    pub const SW: u32 = 0b010;
    pub const SD: u32 = 0b011;

    pub const BEQ: u32 = 0b000;
    pub const BNE: u32 = 0b001;
    pub const BLT: u32 = 0b100;
    pub const BGE: u32 = 0b101;
    pub const BLTU: u32 = 0b110;
    pub const BGEU: u32 = 0b111;

    pub const ADD_SUB: u32 = 0b000;
    pub const SLL: u32 = 0b001;
    pub const SLT: u32 = 0b010;
    pub const SLTU: u32 = 0b011;
    pub const XOR: u32 = 0b100;
    pub const SRL_SRA: u32 = 0b101;
    pub const OR: u32 = 0b110;
    pub const AND: u32 = 0b111;

    pub const MUL: u32 = 0b000;
    pub const MULH: u32 = 0b001;
    pub const MULHSU: u32 = 0b010;
    pub const MULHU: u32 = 0b011;
    pub const DIV: u32 = 0b100;
    pub const DIVU: u32 = 0b101;
    pub const REM: u32 = 0b110;
    pub const REMU: u32 = 0b111;

    pub const CSRRW: u32 = 0b001;
    pub const CSRRS: u32 = 0b010;
    pub const CSRRC: u32 = 0b011;
    pub const CSRRWI: u32 = 0b101;
    pub const CSRRSI: u32 = 0b110;
    pub const CSRRCI: u32 = 0b111;

    pub const FENCE: u32 = 0b000;
    pub const FENCE_I: u32 = 0b001;

    pub const AMO_W: u32 = 0b010;
    pub const AMO_D: u32 = 0b011;

    pub const FP_OPS: u32 = 0b000;
}

pub mod funct7 {
    pub const DEFAULT: u32 = 0b0000000;
    pub const SUB: u32 = 0b0100000;
    pub const SRA: u32 = 0b0100000;

    pub const MULDIV: u32 = 0b0000001;

    pub const ADDW: u32 = 0b0000000;
    pub const SUBW: u32 = 0b0100000;
    pub const SLLW: u32 = 0b0000000;
    pub const SRLW: u32 = 0b0000000;
    pub const SRAW: u32 = 0b0100000;

    pub const LR: u32 = 0b00010;
    pub const SC: u32 = 0b00011;
    pub const AMOSWAP: u32 = 0b00001;
    pub const AMOADD: u32 = 0b00000;
    pub const AMOXOR: u32 = 0b00100;
    pub const AMOAND: u32 = 0b01100;
    pub const AMOOR: u32 = 0b01000;
    pub const AMOMIN: u32 = 0b10000;
    pub const AMOMAX: u32 = 0b10100;
    pub const AMOMINU: u32 = 0b11000;
    pub const AMOMAXU: u32 = 0b11100;

    pub const SFENCE_VMA: u32 = 0b0001001;
}

pub mod system {
    pub const FUNCT12_ECALL: u32 = 0x000;
    pub const FUNCT12_EBREAK: u32 = 0x001;

    pub const FUNCT12_WFI: u32 = 0x105;

    pub const FUNCT12_URET: u32 = 0x002;
    pub const FUNCT12_SRET: u32 = 0x102;
    pub const FUNCT12_MRET: u32 = 0x302;
}

pub mod cause {
    pub const INTERRUPT_BIT: u64 = 1 << 63;
    pub const INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0;
    pub const INSTRUCTION_ACCESS_FAULT: u64 = 1;
    pub const ILLEGAL_INSTRUCTION: u64 = 2;
    pub const BREAKPOINT: u64 = 3;
    pub const LOAD_ADDRESS_MISALIGNED: u64 = 4;
    pub const LOAD_ACCESS_FAULT: u64 = 5;
    pub const STORE_AMO_ADDRESS_MISALIGNED: u64 = 6;
    pub const STORE_AMO_ACCESS_FAULT: u64 = 7;
    pub const ECALL_FROM_U_MODE: u64 = 8;
    pub const ECALL_FROM_S_MODE: u64 = 9;
    pub const ECALL_FROM_M_MODE: u64 = 11;
    pub const INSTRUCTION_PAGE_FAULT: u64 = 12;
    pub const LOAD_PAGE_FAULT: u64 = 13;
    pub const STORE_AMO_PAGE_FAULT: u64 = 15;

    pub const USER_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 0;
    pub const SUPERVISOR_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 1;
    pub const MACHINE_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 3;
    pub const USER_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 4;
    pub const SUPERVISOR_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 5;
    pub const MACHINE_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 7;
    pub const USER_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 8;
    pub const SUPERVISOR_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 9;
    pub const MACHINE_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 11;
}

pub mod csr {
    pub const USTATUS: u32 = 0x000;
    pub const UIE: u32 = 0x004;
    pub const UTVEC: u32 = 0x005;
    pub const USCRATCH: u32 = 0x040;
    pub const UEPC: u32 = 0x041;
    pub const UCAUSE: u32 = 0x042;
    pub const UTVAL: u32 = 0x043;
    pub const UIP: u32 = 0x044;
    pub const FFLAGS: u32 = 0x001;
    pub const FRM: u32 = 0x002;
    pub const FCSR: u32 = 0x003;
    pub const CYCLE: u32 = 0xC00;
    pub const TIME: u32 = 0xC01;
    pub const INSTRET: u32 = 0xC02;
    pub const CYCLEH: u32 = 0xC80;
    pub const TIMEH: u32 = 0xC81;
    pub const INSTRETH: u32 = 0xC82;

    pub const SSTATUS: u32 = 0x100;
    pub const SEDELEG: u32 = 0x102;
    pub const SIDELEG: u32 = 0x103;
    pub const SIE: u32 = 0x104;
    pub const STVEC: u32 = 0x105;
    pub const SCOUNTEREN: u32 = 0x106;
    pub const SSCRATCH: u32 = 0x140;
    pub const SEPC: u32 = 0x141;
    pub const SCAUSE: u32 = 0x142;
    pub const STVAL: u32 = 0x143;
    pub const SIP: u32 = 0x144;
    pub const SATP: u32 = 0x180;

    pub const MVENDORID: u32 = 0xF11;
    pub const MARCHID: u32 = 0xF12;
    pub const MIMPID: u32 = 0xF13;
    pub const MHARTID: u32 = 0xF14;
    pub const MSTATUS: u32 = 0x300;
    pub const MISA: u32 = 0x301;
    pub const MEDELEG: u32 = 0x302;
    pub const MIDELEG: u32 = 0x303;
    pub const MIE: u32 = 0x304;
    pub const MTVEC: u32 = 0x305;
    pub const MCOUNTEREN: u32 = 0x306;
    pub const MSCRATCH: u32 = 0x340;
    pub const MEPC: u32 = 0x341;
    pub const MCAUSE: u32 = 0x342;
    pub const MTVAL: u32 = 0x343;
    pub const MIP: u32 = 0x344;

    pub const PMPCFG0: u32 = 0x3A0;
    pub const PMPCFG1: u32 = 0x3A1;
    pub const PMPCFG2: u32 = 0x3A2;
    pub const PMPCFG3: u32 = 0x3A3;

    pub const PMPADDR0: u32 = 0x3B0;
    pub const PMPADDR1: u32 = 0x3B1;

    pub const MCYCLE: u32 = 0xB00;
    pub const MINSTRET: u32 = 0xB02;
    pub const MCYCLEH: u32 = 0xB80;
    pub const MINSTRETH: u32 = 0xB82;

    pub const TSELECT: u32 = 0x7A0;
    pub const TDATA1: u32 = 0x7A1;
    pub const TDATA2: u32 = 0x7A2;
}

pub mod abi {
    pub const ZERO: u32 = 0;
    pub const RA: u32 = 1;
    pub const SP: u32 = 2;
    pub const GP: u32 = 3;
    pub const TP: u32 = 4;
    pub const T0: u32 = 5;
    pub const T1: u32 = 6;
    pub const T2: u32 = 7;
    pub const S0: u32 = 8;
    pub const FP: u32 = 8;
    pub const S1: u32 = 9;
    pub const A0: u32 = 10;
    pub const A1: u32 = 11;
    pub const A2: u32 = 12;
    pub const A3: u32 = 13;
    pub const A4: u32 = 14;
    pub const A5: u32 = 15;
    pub const A6: u32 = 16;
    pub const A7: u32 = 17;
    pub const S2: u32 = 18;
    pub const S3: u32 = 19;
    pub const S4: u32 = 20;
    pub const S5: u32 = 21;
    pub const S6: u32 = 22;
    pub const S7: u32 = 23;
    pub const S8: u32 = 24;
    pub const S9: u32 = 25;
    pub const S10: u32 = 26;
    pub const S11: u32 = 27;
    pub const T3: u32 = 28;
    pub const T4: u32 = 29;
    pub const T5: u32 = 30;
    pub const T6: u32 = 31;
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct SimpleElfHeader {
    pub magic: [u8; 4],
    pub entry_point: u64,
    pub text_offset: u64,
    pub text_size: u64,
    pub data_offset: u64,
    pub data_size: u64,
    pub bss_size: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Executable {
    pub text: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: u64,
    pub entry_point: u64,
}

pub const BASE_ADDRESS: u64 = 0x80000000;
