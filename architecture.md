# Rusteze Virtual Machine Architecture Specification

## 1. Overview

The Rusteze VM is a simple 32-bit RISC-inspired virtual machine. It features a fixed-size instruction set, a general-purpose register file, and a small, unified memory space.

-   **Architecture:** 32-bit
-   **Instruction Size:** Fixed at 4 bytes (32 bits)
-   **Endianness:** Little-endian
-   **Memory:** 1024 bytes, byte-addressable
-   **Registers:** 32 general-purpose 8-bit registers

## 2. Registers

The VM has 32 general-purpose registers, `x0` through `x31`. By software convention (ABI), several registers have special roles.

| Register | ABI Name | Description                                         | Saved By |
| :------- | :------- | :-------------------------------------------------- | :------- |
| `x0`     | `zero`   | Hardwired to `0`. Writes are ignored.             | -        |
| `x1`     | `ra`     | **Return Address** for function calls.              | Caller   |
| `x2`     | `sp`     | **Stack Pointer**.                                  | Callee   |
| `x3-x4`  |          | (Reserved)                                        | -        |
| `x5-x7`  | `t0-t2`  | Temporary / scratch registers.                      | Caller   |
| `x8-x9`  | `s0-s1`  | Saved registers.                                  | Callee   |
| `x10-x11`| `a0-a1`  | Function **Arguments** and **Return Values**.       | Caller   |
| `x12-x31`|          | (Available for general use)                         | -        |

-   **Caller-Saved:** If the calling function needs the value in this register after the call returns, it must save it to the stack before making the call.
-   **Callee-Saved:** The called function (the "callee") must guarantee that these registers have the same value when it returns as they did when it was called. It must save them to its own stack frame and restore them before returning.

## 3. Memory Layout

The 1024-byte memory is a single contiguous block. A typical program will lay out its memory as follows:

-   `0x000` (Bottom): Program code (`.text` segment)
-   ...
-   (Middle): Heap data (not yet implemented)
-   ...
-   `0x3FF` (Top): The Stack. The stack grows downwards from high addresses to low addresses. The `sp` register points to the top of the stack.

## 4. Instruction Set

All instructions are 4 bytes. The first byte is the opcode, followed by three bytes for operands.

---

### `halt` - Halt Execution

-   **Opcode:** `0x00`
-   **Syntax:** `halt`
-   **Description:** Stops the virtual machine.
-   **Encoding:** `[0x00, 0x00, 0x00, 0x00]`

---

### `add` - Add

-   **Opcode:** `0x01`
-   **Syntax:** `add rd, rs1, rs2`
-   **Description:** Adds the value in `rs1` to the value in `rs2` and stores the result in `rd`.
-   **Encoding:** `[0x01, rd, rs1, rs2]`

---

### `sub` - Subtract

-   **Opcode:** `0x02`
-   **Syntax:** `sub rd, rs1, rs2`
-   **Description:** Subtracts the value in `rs2` from the value in `rs1` and stores the result in `rd`.
-   **Encoding:** `[0x02, rd, rs1, rs2]`

---

### `addi` - Add Immediate

-   **Opcode:** `0x03`
-   **Syntax:** `addi rd, rs1, imm`
-   **Description:** Adds the signed immediate value (`imm`, from -128 to 127) to the value in `rs1` and stores the result in `rd`.
-   **Encoding:** `[0x03, rd, rs1, imm]`

---

### `beq` - Branch if Equal

-   **Opcode:** `0x04`
-   **Syntax:** `beq rs1, rs2, label`
-   **Description:** If the value in `rs1` is equal to the value in `rs2`, sets the program counter (`pc`) to the address of `label`. The offset is calculated relative to the `beq` instruction's own address.
-   **Encoding:** `[0x04, rs1, rs2, offset]`

---

### `jal` - Jump and Link

-   **Opcode:** `0x05`
-   **Syntax:** `jal rd, label` (unimplemented)
-   **Description:** Used to call functions. Stores the address of the next instruction (`pc + 4`) in `rd` (typically `ra`), then sets the `pc` to the address of `label`.
-   **Encoding:** `[0x05, rd, 0x00, offset]` (Offset calculated like `beq`)

---

### `lw` - Load Word (Byte)

-   **Opcode:** `0x06` (unimplemented)
-   **Syntax:** `lw rd, offset(base)`
-   **Description:** Loads one byte from memory at the address `[register base + offset]` and stores it in `rd`.
-   **Encoding:** `[0x06, rd, base, offset]`

---

### `sw` - Store Word (Byte)

-   **Opcode:** `0x07` (unimplemented)
-   **Syntax:** `sw rs, offset(base)`
-   **Description:** Stores the value from register `rs` into memory at the address `[register base + offset]`.
-   **Encoding:** `[0x07, rs, base, offset]`

---

### `ret` - Return from Function

-   **Pseudo-instruction**
-   **Syntax:** `ret`
-   **Description:** Standard way to return from a function. The assembler translates this to `jal x0, 0(ra)`, effectively jumping to the address stored in the `ra` register.

## 5. Calling Convention

To allow functions to call each other without interfering, the following rules must be followed.

### Function Arguments & Return

-   The first argument is passed in `a0` (`x10`).
-   The second argument is passed in `a1` (`x11`).
-   The return value is placed in `a0` (`x10`).

### Function Prologue (At the start of a function)

1.  **Allocate Stack Frame:** Make space on the stack for any data you need to save by decrementing the stack pointer (`addi sp, sp, -size`).
2.  **Save Return Address:** If the function calls another function (is not a "leaf" function), it MUST save the `ra` register to its stack frame. (`sw ra, offset(sp)`)
3.  **Save Callee-Saved Registers:** If the function intends to modify any `s` registers, it must save their original values to its stack frame first.

### Function Epilogue (At the end of a function)

1.  **Place Return Value:** Put the return value into `a0` (`x10`).
2.  **Restore Callee-Saved Registers:** Restore the original values of any `s` registers from the stack frame.
3.  **Restore Return Address:** Restore the `ra` register from the stack frame.
4.  **Deallocate Stack Frame:** Add the size back to the stack pointer (`addi sp, sp, size`).
5.  **Return:** Execute a `ret` instruction.
