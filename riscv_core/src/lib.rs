pub mod opcodes {
    // Base Opcodes (I-extension and RV64I)
    pub const OP_LOAD: u32 = 0b0000011; // Load instructions (lb, lh, lw, ld, etc.)
    pub const OP_MISC_MEM: u32 = 0b0001111; // Memory synchronization (fence)
    pub const OP_IMM: u32 = 0b0010011; // Immediate arithmetic/logic (addi, slti, etc.)
    pub const OP_AUIPC: u32 = 0b0010111; // Add Upper Immediate to PC
    pub const OP_IMM_32: u32 = 0b0011011; // Immediate arithmetic/logic for 32-bit ops in RV64 (addiw, etc.)
    pub const OP_STORE: u32 = 0b0100011; // Store instructions (sb, sh, sw, sd)
    pub const OP_REG: u32 = 0b0110011; // Register-register arithmetic/logic (add, sub, etc.)
    pub const OP_LUI: u32 = 0b0110111; // Load Upper Immediate
    pub const OP_REG_32: u32 = 0b0111011; // Register-register 32-bit ops in RV64 (addw, subw, etc.)
    pub const OP_BRANCH: u32 = 0b1100011; // Branch instructions (beq, bne, etc.)
    pub const OP_JALR: u32 = 0b1100111; // Jump and Link Register
    pub const OP_JAL: u32 = 0b1101111; // Jump and Link
    pub const OP_SYSTEM: u32 = 0b1110011; // System instructions (ecall, ebreak, csr)

    // Atomic Extension (A-extension) Opcodes
    pub const OP_AMO: u32 = 0b0101111; // Atomic Memory Operations (amoadd, amoswap, etc.)

    // Floating-Point Extensions (F/D/Q-extensions) Opcodes
    pub const OP_LOAD_FP: u32 = 0b0000111; // Floating-point load (flw, fld)
    pub const OP_STORE_FP: u32 = 0b0100111; // Floating-point store (fsw, fsd)
    pub const OP_FP: u32 = 0b1010011; // Floating-point operations (fadd, fsub, etc.)
    pub const OP_MADD: u32 = 0b1000011; // Fused Multiply-Add (fmadd)
    pub const OP_MSUB: u32 = 0b1000111; // Fused Multiply-Subtract (fmsub)
    pub const OP_NMSUB: u32 = 0b1001011; // Fused Negative Multiply-Subtract (fnmsub)
    pub const OP_NMADD: u32 = 0b1001111; // Fused Negative Multiply-Add (fnmadd)
}

pub mod funct3 {
    // funct3 for OP_LOAD and OP_STORE
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

    // funct3 for OP_BRANCH
    pub const BEQ: u32 = 0b000;
    pub const BNE: u32 = 0b001;
    pub const BLT: u32 = 0b100;
    pub const BGE: u32 = 0b101;
    pub const BLTU: u32 = 0b110;
    pub const BGEU: u32 = 0b111;

    // funct3 for OP_IMM, OP_IMM_32, OP_REG, and OP_REG_32
    pub const ADD_SUB: u32 = 0b000; // add, addi, sub, addw, addiw, subw
    pub const SLL: u32 = 0b001; // sll, slli, sllw, slliw
    pub const SLT: u32 = 0b010; // slt, slti
    pub const SLTU: u32 = 0b011; // sltu, sltiu
    pub const XOR: u32 = 0b100; // xor, xori
    pub const SRL_SRA: u32 = 0b101; // srl, sra, srli, srai, srlw, sraiw
    pub const OR: u32 = 0b110; // or, ori
    pub const AND: u32 = 0b111; // and, andi

    // funct3 for M-extension (OP_REG)
    pub const MUL: u32 = 0b000;
    pub const MULH: u32 = 0b001;
    pub const MULHSU: u32 = 0b010;
    pub const MULHU: u32 = 0b011;
    pub const DIV: u32 = 0b100;
    pub const DIVU: u32 = 0b101;
    pub const REM: u32 = 0b110;
    pub const REMU: u32 = 0b111;

    // funct3 for OP_SYSTEM (CSR instructions)
    pub const CSRRW: u32 = 0b001;
    pub const CSRRS: u32 = 0b010;
    pub const CSRRC: u32 = 0b011;
    pub const CSRRWI: u32 = 0b101;
    pub const CSRRSI: u32 = 0b110;
    pub const CSRRCI: u32 = 0b111;

    // funct3 for A-extension (OP_AMO)
    pub const AMO_W: u32 = 0b010; // For 32-bit atomics
    pub const AMO_D: u32 = 0b011; // For 64-bit atomics

    // funct3 for F/D-extensions
    pub const FADD: u32 = 0b000;
    pub const FSUB: u32 = 0b001;
    pub const FMUL: u32 = 0b010;
    pub const FDIV: u32 = 0b011;
}

pub mod funct7 {
    // funct7 for standard R-type instructions
    pub const DEFAULT: u32 = 0b0000000; // For ADD, SLL, SLT, SLTU, XOR, SRL, OR, AND
    pub const SUB: u32 = 0b0100000; // For SUB
    pub const SRA: u32 = 0b0100000; // For SRA

    // funct7 for M-extension instructions (all share the same funct7)
    pub const MULDIV: u32 = 0b0000001;

    // funct7 for RV64I Word instructions
    pub const ADDW: u32 = 0b0000000;
    pub const SUBW: u32 = 0b0100000;
    pub const SLLW: u32 = 0b0000000;
    pub const SRLW: u32 = 0b0000000;
    pub const SRAW: u32 = 0b0100000;

    // funct5 (bits 31-27) for A-extension instructions. The lower two bits (26-25) are for aq/rl flags.
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

    // funct7 for privileged instructions
    pub const SFENCE_VMA: u32 = 0b0001001;
}

pub mod system {
    // funct12 for OP_SYSTEM instructions
    pub const FUNCT12_ECALL: u32 = 0x000;
    pub const FUNCT12_EBREAK: u32 = 0x001;
    pub const FUNCT12_WFI: u32 = 0x105; // Wait For Interrupt

    // funct12 for trap-return instructions
    pub const FUNCT12_URET: u32 = 0x002;
    pub const FUNCT12_SRET: u32 = 0x102;
    pub const FUNCT12_MRET: u32 = 0x302;
}

pub mod cause {
    //! Contains the cause codes for exceptions and interrupts as defined by the RISC-V specification.
    //! The `mcause` CSR holds one of these values when a trap occurs.

    // The most significant bit of `mcause` is 1 for interrupts and 0 for exceptions.
    pub const INTERRUPT_BIT: u64 = 1 << 63;

    // --- Synchronous Exceptions ---

    /// Instruction address misaligned
    pub const INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0;
    /// Instruction access fault
    pub const INSTRUCTION_ACCESS_FAULT: u64 = 1;
    /// Illegal instruction
    pub const ILLEGAL_INSTRUCTION: u64 = 2;
    /// Breakpoint
    pub const BREAKPOINT: u64 = 3;
    /// Load address misaligned
    pub const LOAD_ADDRESS_MISALIGNED: u64 = 4;
    /// Load access fault
    pub const LOAD_ACCESS_FAULT: u64 = 5;
    /// Store/AMO address misaligned
    pub const STORE_AMO_ADDRESS_MISALIGNED: u64 = 6;
    /// Store/AMO access fault
    pub const STORE_AMO_ACCESS_FAULT: u64 = 7;
    /// Environment call from User mode
    pub const ECALL_FROM_U_MODE: u64 = 8;
    /// Environment call from Supervisor mode
    pub const ECALL_FROM_S_MODE: u64 = 9;
    /// Environment call from Machine mode
    pub const ECALL_FROM_M_MODE: u64 = 11;
    /// Instruction page fault
    pub const INSTRUCTION_PAGE_FAULT: u64 = 12;
    /// Load page fault
    pub const LOAD_PAGE_FAULT: u64 = 13;
    /// Store/AMO page fault
    pub const STORE_AMO_PAGE_FAULT: u64 = 15;

    // --- Asynchronous Interrupts ---

    /// User software interrupt
    pub const USER_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 0;
    /// Supervisor software interrupt
    pub const SUPERVISOR_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 1;
    /// Machine software interrupt
    pub const MACHINE_SOFTWARE_INTERRUPT: u64 = INTERRUPT_BIT | 3;
    /// User timer interrupt
    pub const USER_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 4;
    /// Supervisor timer interrupt
    pub const SUPERVISOR_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 5;
    /// Machine timer interrupt
    pub const MACHINE_TIMER_INTERRUPT: u64 = INTERRUPT_BIT | 7;
    /// User external interrupt
    pub const USER_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 8;
    /// Supervisor external interrupt
    pub const SUPERVISOR_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 9;
    /// Machine external interrupt
    pub const MACHINE_EXTERNAL_INTERRUPT: u64 = INTERRUPT_BIT | 11;
}

pub mod csr {
    //! Contains the addresses of the Control and Status Registers (CSRs) as defined by the RISC-V specification.

    // --- User-Level CSRs (read-only) ---
    pub const CYCLE: u32 = 0xC00; // Cycle counter for RDCYCLE instruction
    pub const TIME: u32 = 0xC01; // Timer for RDTIME instruction
    pub const INSTRET: u32 = 0xC02; // Instructions-retired counter for RDINSTRET
    pub const CYCLEH: u32 = 0xC80; // Upper 32 bits of cycle, for RV32
    pub const TIMEH: u32 = 0xC81; // Upper 32 bits of time, for RV32
    pub const INSTRETH: u32 = 0xC82; // Upper 32 bits of instret, for RV32

    // --- Supervisor-Level CSRs ---

    // Supervisor Trap Setup
    pub const SSTATUS: u32 = 0x100; // Supervisor status register
    pub const SIE: u32 = 0x104; // Supervisor interrupt-enable register
    pub const STVEC: u32 = 0x105; // Supervisor trap handler base address
    pub const SCOUNTEREN: u32 = 0x106; // Supervisor counter enable

    // Supervisor Trap Handling
    pub const SSCRATCH: u32 = 0x140; // Scratch register for supervisor trap handlers
    pub const SEPC: u32 = 0x141; // Supervisor exception program counter
    pub const SCAUSE: u32 = 0x142; // Supervisor trap cause
    pub const STVAL: u32 = 0x143; // Supervisor bad address or instruction
    pub const SIP: u32 = 0x144; // Supervisor interrupt pending

    // Supervisor Address Translation and Protection
    pub const SATP: u32 = 0x180; // Supervisor address translation and protection (for MMU)

    // --- Machine-Level CSRs ---

    // Machine Information Registers
    pub const MVENDORID: u32 = 0xF11; // Vendor ID
    pub const MARCHID: u32 = 0xF12; // Architecture ID
    pub const MIMPID: u32 = 0xF13; // Implementation ID
    pub const MHARTID: u32 = 0xF14; // Hardware thread ID

    // Machine Trap Setup
    pub const MSTATUS: u32 = 0x300; // Machine status register
    pub const MISA: u32 = 0x301; // ISA and extensions
    pub const MEDELEG: u32 = 0x302; // Machine exception delegation register
    pub const MIDELEG: u32 = 0x303; // Machine interrupt delegation register
    pub const MIE: u32 = 0x304; // Machine interrupt-enable register
    pub const MTVEC: u32 = 0x305; // Machine trap-handler base address
    pub const MCOUNTEREN: u32 = 0x306; // Machine counter enable

    // Machine Trap Handling
    pub const MSCRATCH: u32 = 0x340; // Scratch register for machine trap handlers
    pub const MEPC: u32 = 0x341; // Machine exception program counter
    pub const MCAUSE: u32 = 0x342; // Machine trap cause
    pub const MTVAL: u32 = 0x343; // Machine bad address or instruction
    pub const MIP: u32 = 0x344; // Machine interrupt pending

    // Machine Memory Protection (PMP)
    pub const PMPCFG0: u32 = 0x3A0;
    pub const PMPCFG1: u32 = 0x3A1;
    pub const PMPCFG2: u32 = 0x3A2;
    pub const PMPCFG3: u32 = 0x3A3;
    // ... up to PMPCFG15 for RV64
    pub const PMPADDR0: u32 = 0x3B0;
    pub const PMPADDR1: u32 = 0x3B1;
    // ... up to PMPADDR63

    // Machine Counters and Timers
    pub const MCYCLE: u32 = 0xB00; // Machine cycle counter
    pub const MINSTRET: u32 = 0xB02; // Machine instructions-retired counter
    pub const MCYCLEH: u32 = 0xB80; // Upper 32 bits of mcycle, for RV32
    pub const MINSTRETH: u32 = 0xB82; // Upper 32 bits of minstret, for RV32
                                      // MTIME and MTIMECMP are typically memory-mapped, not standard CSRs,
                                      // but are fundamental to the timer interrupt mechanism.

    // Debug/Trace Registers (optional)
    pub const TSELECT: u32 = 0x7A0; // Trigger select register
    pub const TDATA1: u32 = 0x7A1; // Trigger data register 1
    pub const TDATA2: u32 = 0x7A2; // Trigger data register 2
}

pub mod abi {
    pub const ZERO: u32 = 0; // x0: Hard-wired zero
    pub const RA: u32 = 1; // x1: Return address
    pub const SP: u32 = 2; // x2: Stack pointer
    pub const GP: u32 = 3; // x3: Global pointer
    pub const TP: u32 = 4; // x4: Thread pointer

    // Temporary/alternate link registers
    pub const T0: u32 = 5; // x5
    pub const T1: u32 = 6; // x6
    pub const T2: u32 = 7; // x7

    // Saved registers/frame pointer
    pub const S0: u32 = 8; // x8: Also known as FP (Frame Pointer)
    pub const FP: u32 = 8; // x8: Alias for S0
    pub const S1: u32 = 9; // x9

    // Function arguments/return values
    pub const A0: u32 = 10; // x10: Also for return value
    pub const A1: u32 = 11; // x11: Also for return value
    pub const A2: u32 = 12; // x12
    pub const A3: u32 = 13; // x13
    pub const A4: u32 = 14; // x14
    pub const A5: u32 = 15; // x15
    pub const A6: u32 = 16; // x16
    pub const A7: u32 = 17; // x17: Also for syscall number

    // Saved registers
    pub const S2: u32 = 18; // x18
    pub const S3: u32 = 19; // x19
    pub const S4: u32 = 20; // x20
    pub const S5: u32 = 21; // x21
    pub const S6: u32 = 22; // x22
    pub const S7: u32 = 23; // x23
    pub const S8: u32 = 24; // x24
    pub const S9: u32 = 25; // x25
    pub const S10: u32 = 26; // x26
    pub const S11: u32 = 27; // x27

    // Temporary registers
    pub const T3: u32 = 28; // x28
    pub const T4: u32 = 29; // x29
    pub const T5: u32 = 30; // x30
    pub const T6: u32 = 31; // x31
}
