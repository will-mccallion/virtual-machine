//! A library of shared constants and types for the RISC-V toolkit.

// Each module is made public so it can be used by other crates.
pub mod opcodes {
    pub const OP_LOAD: u32 = 0b0000011;
    pub const OP_IMM: u32 = 0b0010011;
    pub const OP_STORE: u32 = 0b0100011;
    pub const OP_REG: u32 = 0b0110011;
    pub const OP_BRANCH: u32 = 0b1100011;
    pub const OP_JALR: u32 = 0b1100111;
    pub const OP_JAL: u32 = 0b1101111;
    pub const OP_SYSTEM: u32 = 0b1110011;
    pub const OP_HALT: u32 = 0b00000000;
    pub const OP_AUIPC: u32 = 0b0010111;
}

pub mod funct3 {
    pub const ADD_SUB: u32 = 0b000;
    pub const MUL: u32 = 0b000;
    pub const DIV: u32 = 0b100;
    pub const LW: u32 = 0b010;
    pub const ADDI: u32 = 0b000;
    pub const AND: u32 = 0b111;
    pub const SW: u32 = 0b010;
    pub const BEQ: u32 = 0b000;
    pub const BLT: u32 = 0b100;
    pub const BNE: u32 = 0b001;
    pub const LD: u32 = 0b011;
    pub const SD: u32 = 0b011;
    pub const LB: u32 = 0b000;
    pub const SB: u32 = 0b000;
    pub const OR: u32 = 0b110;
    pub const SLT: u32 = 0b010;
    pub const SRA: u32 = 0b101;
    pub const SRL: u32 = 0b101;
    pub const XOR: u32 = 0b100;
    pub const LBU: u32 = 0b100;
}

pub mod funct7 {
    pub const MULDIV: u32 = 0b0000001;
    pub const ADD: u32 = 0b0000000;
    pub const SUB: u32 = 0b0100000;
    pub const SRA: u32 = 0b0100000;
    pub const DEFAULT: u32 = 0b0000000;
}

pub mod system {
    pub const FUNCT12_ECALL: u32 = 0x0;
}
