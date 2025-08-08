# Rusteze RISC-V Virtual Machine

## 1. Overview

Rusteze is a 64-bit emulator that implements a subset of the standard RISC-V Instruction Set Architecture (ISA). It consists of a command-line assembler and a virtual machine capable of executing RISC-V binaries.

The project adheres to the official open standard to ensure compatibility and demonstrate real-world CPU architecture concepts. For more detailed information on the RISC-V design, see the **[official RISC-V specifications page](https://riscv.org/specifications/)**.

-   **Architecture:** 64-bit (RV64IM subset).
    -   **I:** Base Integer Instruction Set
    -   **M:** Standard Extension for Integer Multiplication and Division
-   **Instruction Size:** All implemented instructions are a fixed 32-bits (4 bytes) wide.
-   **Endianness:** Little-endian.
-   **Memory:** 128 MB, byte-addressable.
-   **Components:**
    -   **Assembler:** Translates standard RISC-V assembly mnemonics into compliant 32-bit machine code.
    -   **Virtual Machine:** Fetches, decodes, and executes the 32-bit instructions.
    -   **OS:** (Not implemented)

## 2. Registers

The VM implements the standard 32 general-purpose 64-bit integer registers, `x0` through `x31`. The standard RISC-V integer calling convention gives them the following Application Binary Interface (ABI) names and roles.

| Register | ABI Name | Description                                         | Saved By |
| :------- | :------- | :-------------------------------------------------- | :------- |
| `x0`     | `zero`   | Hardwired to `0`.                                   | -        |
| `x1`     | `ra`     | **Return Address** for function calls.              | Caller   |
| `x2`     | `sp`     | **Stack Pointer**.                                  | Callee   |
| `x3`     | `gp`     | **Global Pointer**.                                 | -        |
| `x4`     | `tp`     | **Thread Pointer**.                                 | -        |
| `x5-x7`  | `t0-t2`  | Temporary / scratch registers.                      | Caller   |
| `x8-x9`  | `s0-s1`  | Saved registers.                                    | Callee   |
| `x10-x17`| `a0-a7`  | Function **Arguments** and **Return Values**.       | Caller   |
| `x18-x27`| `s2-s11` | More Saved registers.                               | Callee   |
| `x28-x31`| `t3-t6`  | More Temporary registers.                           | Caller   |


-   **Caller-Saved:** If the calling function needs the value in this register after the call returns, it must save it to the stack before the call. (`ra`, `t0-t6`, `a0-a7`)
-   **Callee-Saved:** The called function (the "callee") must guarantee that these registers have the same value when it returns as they did when it was called. (`sp`, `s0-s11`)

## 3. Memory Layout

The 128 MB memory space is a single contiguous block. On startup, the VM initializes the stack pointer (`sp`) to the highest address of memory.

-   **Low Addresses:** Program code (`.text` segment) is loaded at the bottom.
-   ...
-   **(Middle):** Heap data (Not implemented).
-   ...
-   **High Addresses:** The Stack. The stack grows downwards from high addresses to low addresses.

## 4. Instruction Set Architecture

The VM uses the standard 32-bit RISC-V instruction formats. Unlike a simpler design where the first byte is always the opcode, RISC-V uses different formats (R, I, S, SB, UJ, U) to efficiently encode operands. The assembler and VM correctly handle the bit-level encoding and decoding of these formats.

### Implemented Instructions (RV64IM Subset)

| Mnemonic | Description                                                               | Format Type  |
| :------- | :------------------------------------------------------------------------ | :----------  |
| `add`    | Adds the values in two source registers.                                  | R-Type       |
| `sub`    | Subtracts the values in two source registers.                             | R-Type       |
| `mul`    | Multiplies the values in two source registers.                            | R-Type       |
| `div`    | Divides the value in `rs1` by `rs2`.                                      | R-Type       |
| `addi`   | Adds a 12-bit sign-extended immediate to a register.                      | I-Type       |
| `lw`     | Loads a 32-bit word from memory and sign-extends it to 64 bits.           | I-Type       |
| `sw`     | Stores a 32-bit word from a register into memory.                         | S-Type       |
| `beq`    | Branches to a new address if two registers are equal.                     | SB-Type      |
| `blt`    | Branches to a new address if the left register is less than the right.    | SB-Type      |
| `bne`    | Branches to a new address if two registers are not equal.                 | SB-Type      |
| `jal`    | Jumps to a new address, storing the return address in `rd`.               | UJ-Type      |
| `jalr`   | Jumps to an address in a register, storing the return address in `rd`.    | I-Type       |
| `ecall`  | Triggers an environment call (system call) to the host OS/kernel.         | I-Type       |

### Pseudo-Instructions

The assembler also supports common pseudo-instructions that make programming easier:

-   **`ret`**: (Return) Expands to the real instruction **`jalr zero, ra, 0`**, which jumps to the address stored in the `ra` register without saving a new return address.

## 5. Calling Convention

To allow functions to call each other without interfering, the standard RISC-V calling convention must be followed.

### Function Arguments & Return

-   The first eight arguments are passed in registers `a0` through `a7`.
-   The return value is placed in `a0`.

### Function Prologue (At the start of a function)

1.  **Allocate Stack Frame:** Make space on the stack for any data you need to save by decrementing the stack pointer (`addi sp, sp, -size`). The size should be a multiple of **4** to maintain stack alignment for word-sized data.
2.  **Save Return Address:** If the function calls another function (is not a "leaf" function), it MUST save the `ra` register to the stack (e.g., `sw ra, 8(sp)`).
3.  **Save Callee-Saved Registers:** If the function intends to modify any `s` registers (`s0-s11`), it must first save their original values to the stack (e.g., `sw s0, 4(sp)`).

### Function Epilogue (At the end of a function)

1.  **Place Return Value:** Ensure the final return value is in `a0`.
2.  **Restore Callee-Saved Registers:** Restore the original values of any `s` registers from the stack.
3.  **Restore Return Address:** Restore the `ra` register from the stack.
4.  **Deallocate Stack Frame:** Add the size back to the stack pointer (`addi sp, sp, size`).
5.  **Return:** Execute a `ret` instruction.
