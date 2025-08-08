# Rusteze Virtual Machine Architecture Specification

## 1. Overview

The Rusteze VM is a simple **64-bit** RISC-inspired virtual machine. It features a fixed-size base instruction set, a general-purpose register file, and a small, byte-addressable memory space.

-   **Architecture:** 64-bit (64-bit data path and registers).
-   **Instruction Size:** Base instructions are fixed at 4 bytes (32 bits). Special instructions for loading large constants exist.
-   **Endianness:** Little-endian.
-   **Memory:** 10240 bytes, byte-addressable.
-   **Registers:** 32 general-purpose **64-bit** registers.

## 2. Registers

The VM has 32 general-purpose 64-bit registers, `x0` through `x31`. The standard RISC-V integer calling convention gives them the following names and roles.

| Register | ABI Name | Description                                         | Saved By |
| :------- | :------- | :-------------------------------------------------- | :------- |
| `x0`     | `zero`   | Hardwired to `0`.                                   | -        |
| `x1`     | `ra`     | **Return Address** for function calls.              | Caller   |
| `x2`     | `sp`     | **Stack Pointer**.                                  | Callee   |
| `x3`     | `gp`     | **Global Pointer**.                                 | -        |
| `x4`     | `tp`     | **Thread Pointer**.                                 | -        |
| `x5-x7`  | `t0-t2`  | Temporary / scratch registers.                      | Caller   |
| `x8-x9`  | `s0-s1`  | Saved ("callee-saved") registers.                   | Callee   |
| `x10-x17`| `a0-a7`  | Function **Arguments** and **Return Values**.       | Caller   |
| `x18-x27`| `s2-s11` | More Saved registers.                               | Callee   |
| `x28-x31`| `t3-t6`  | More Temporary registers.                           | Caller   |


-   **Caller-Saved:** If the calling function needs the value in this register after the call returns, it must save it to the stack before the call. (`ra`, `t0-t6`, `a0-a7`)
-   **Callee-Saved:** The called function (the "callee") must guarantee that these registers have the same value when it returns as they did when it was called. (`sp`, `s0-s11`)

## 3. Memory Layout## 3. Memory Layout

The 1024-byte memory is a single contiguous block. The bootloader initializes the stack pointer (`sp`) to the end of memory.

-   `0x000` (Bottom): Program code (`.text` segment)
-   ...
-   (Middle): Heap data (not yet implemented)
-   ...
-   `0x400` (Top, address 1024): The Stack. The stack grows downwards from high addresses to low addresses. The `sp` register points to the top of the stack.

## 4. Instruction Set

Base instructions are 4 bytes. The first byte is the opcode, followed by three bytes for operands.

---

### `add` - Add

-   **Opcode:** `0x01`
-   **Syntax:** `add rd, rs1, rs2`
-   **Description:** Adds the 64-bit value in `rs1` to the 64-bit value in `rs2` and stores the result in `rd`.
-   **Encoding:** `[0x01, rd, rs1, rs2]`

---

### `sub` - Subtract

-   **Opcode:** `0x02`
-   **Syntax:** `sub rd, rs1, rs2`
-   **Description:** Subtracts the 64-bit value in `rs2` from `rs1` and stores the result in `rd`.
-   **Encoding:** `[0x02, rd, rs1, rs2]`

---

### `addi` - Add Immediate

-   **Opcode:** `0x03`
-   **Syntax:** `addi rd, rs1, imm`
-   **Description:** Sign-extends the 8-bit immediate (`imm`, from -128 to 127) to 64 bits, adds it to the value in `rs1`, and stores the result in `rd`.
-   **Encoding:** `[0x03, rd, rs1, imm]`

---

### `beq` - Branch if Equal

-   **Opcode:** `0x04`
-   **Syntax:** `beq rs1, rs2, label`
-   **Description:** If the value in `rs1` equals the value in `rs2`, adds the sign-extended 8-bit offset to the program counter (`pc`).
-   **Encoding:** `[0x04, rs1, rs2, offset]`

---

### `jal` - Jump and Link

-   **Opcode:** `0x05`
-   **Syntax:** `jal rd, label`
-   **Description:** Stores the address of the next instruction (`pc + 4`) in `rd` (typically `ra`) and jumps to the target `label`. The jump offset is a signed 16-bit immediate.
-   **Encoding:** `[0x05, rd, offset_low, offset_high]`

---

### `lw` - Load Word (64-bit)

-   **Opcode:** `0x06`
-   **Syntax:** `lw rd, offset(base)`
-   **Description:** Loads an 8-byte (64-bit) word from memory at the address `[register base + offset]` and stores it in `rd`.
-   **Encoding:** `[0x06, rd, base, offset]`

---

### `sw` - Store Word (64-bit)

-   **Opcode:** `0x07`
-   **Syntax:** `sw rs, offset(base)`
-   **Description:** Stores the 8-byte (64-bit) value from register `rs` into memory at the address `[register base + offset]`.
-   **Encoding:** `[0x07, rs, base, offset]`

---

### `ret` - Return from Function

-   **Opcode:** `0x08`
-   **Syntax:** `ret`
-   **Description:** Real instruction that jumps to the address stored in the `ra` (`x1`) register. If `ra` is 0, the VM halts.
-   **Encoding:** `[0x08, 0x00, 0x00, 0x00]`

---

### `ldi` - Load Immediate

-   **Opcode:** `0x09`
-   **Syntax:** `ldi rd, imm64` (Used by the assembler, not directly by the programmer).
-   **Description:** Loads a full 64-bit immediate value into register `rd`. This is a 12-byte instruction.
-   **Encoding:** `[0x09, rd, 0x00, 0x00, imm_byte0, imm_byte1, ..., imm_byte7]`

### `li` - Load Immediate (Pseudo-instruction)

-   **Syntax:** `li rd, immediate`
-   **Description:** The user-friendly way to load any 64-bit constant into a register. The assembler translates this into an `ldi` instruction.

### `mul` - Multiply

-   **Opcode:** `0x0A`
-   **Syntax:** `mul rd, rs1, rs2`
-   **Description:** Multiplies the 64-bit value in `rs1` by the value in `rs2` and stores the lower 64 bits of the result in `rd`.
-   **Encoding:** `[0x0A, rd, rs1, rs2]`

---

### `div` - Divide

-   **Opcode:** `0x0B`
-   **Syntax:** `div rd, rs1, rs2`
-   **Description:** Divides the 64-bit value in `rs1` by the value in `rs2` and stores the integer result (quotient) in `rd`.
-   **Note:** Behavior for division by zero is undefined in the current VM. A future version should trigger an exception.
-   **Encoding:** `[0x0B, rd, rs1, rs2]`

---

### `ecall` - Environment Call (System Call)

-   **Opcode:** `0xFF`
-   **Syntax:** `ecall`
-   **Description:** Used to request a service from the operating system (the 'environment'). When executed, the VM halts normal execution and jumps to a predefined system trap handler address (e.g., `0x100`). The specific service requested is determined by software convention, typically by a value passed in register `a0` or `a7`. This is the primary mechanism for a program to interact with the kernel.
-   **Encoding:** `[0xFF, 0x00, 0x00, 0x00]`

## 5. Calling Convention

To allow functions to call each other without interfering, the following rules must be followed.

### Function Arguments & Return

-   The first argument is passed in `a0` (`x10`).
-   The second argument is passed in `a1` (`x11`).
-   The return value is placed in `a0` (`x10`).

### Function Prologue (At the start of a function)

1.  **Allocate Stack Frame:** Make space on the stack for any data you need to save by decrementing the stack pointer (`addi sp, sp, -size`). The size should be a multiple of 8 to maintain stack alignment.
2.  **Save Return Address:** If the function calls another function (is not a "leaf" function), it MUST save the `ra` register to the stack (e.g., `sw ra, 16(sp)`).
3.  **Save Callee-Saved Registers:** If the function intends to modify any `s` registers (`s0`, `s1`), it must save their original values to the stack (e.g., `sw s0, 0(sp)`).

### Function Epilogue (At the end of a function)

1.  **Place Return Value:** Ensure the final return value is in `a0` (`x10`).
2.  **Restore Callee-Saved Registers:** Restore the original values of any `s` registers from the stack frame (e.g., `lw s0, 0(sp)`).
3.  **Restore Return Address:** Restore the `ra` register from the stack frame (e.g., `lw ra, 16(sp)`).
4.  **Deallocate Stack Frame:** Add the size back to the stack pointer (`addi sp, sp, size`).
5.  **Return:** Execute a `ret` instruction.
