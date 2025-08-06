# A RISC-V CPU and Virtual Machine

## 1. Overview

This is a 64-bit virtual machine that fully implements the standard RISC-V 64-bit integer instruction set. It includes a command-line assembler that translates human-readable assembly code into machine code, and a virtual machine that executes that code according to the official RISC-V specification.

The project is built to be a correct and clear model of a real-world CPU, demonstrating core concepts in computer architecture, including instruction encoding, privilege levels, and system control.

-   **Architecture:** RV64IM
    -   **I:** The complete 32-bit and 64-bit Base Integer Instruction Set.
    -   **M:** The Standard Extension for Integer Multiplication and Division.
-   **Privilege Levels:** Implements Machine, Supervisor, and User modes, forming the foundation for running a future operating system.
-   **System Control:** Models Control and Status Registers (CSRs) for managing system state, traps, and exceptions.
-   **Memory:** 128 MB of byte-addressable RAM with a simple, direct-mapped memory model.
-   **Endianness:** Little-endian.

## 2. CPU Registers

The virtual CPU implements the standard 32 general-purpose 64-bit integer registers (`x0` through `x31`). The register `x0` is special, as it is hardwired to the value zero. The standard RISC-V calling convention gives the other registers the following names and roles.

| Register | ABI Name | Description                                    | Saved By |
| :------- | :------- | :--------------------------------------------- | :------- |
| `x0`     | `zero`   | Hardwired to `0`.                              | -        |
| `x1`     | `ra`     | **Return Address** for function calls.         | Caller   |
| `x2`     | `sp`     | **Stack Pointer**.                             | Callee   |
| `x3`     | `gp`     | **Global Pointer**.                            | -        |
| `x4`     | `tp`     | **Thread Pointer**.                            | -        |
| `x5-x7`  | `t0-t2`  | Temporary / scratch registers.                 | Caller   |
| `x8-x9`  | `s0-s1`  | Saved registers (preserved across calls).      | Callee   |
| `x10-x17`| `a0-a7`  | Function **Arguments** and **Return Values**.  | Caller   |
| `x18-x27`| `s2-s11` | More Saved registers.                          | Callee   |
| `x28-x31`| `t3-t6`  | More Temporary registers.                      | Caller   |

-   **Caller-Saved:** The calling function is responsible for saving these registers if it needs their values after the call returns.
-   **Callee-Saved:** A function must ensure these registers have the same value upon returning as they did when it was called.

## 3. Memory Model

The VM's 128 MB of memory is a single, contiguous block starting at the physical address `0x80000000`.

-   **Program Code (`.text`):** Loaded at the base address (`0x80000000`).
-   **Program Data (`.data`):** Loaded immediately after the program code.
-   **The Stack:** The stack pointer (`sp`) is initialized to the highest address in memory. The stack grows downwards toward lower addresses as data is pushed onto it.

Currently, the VM uses a direct physical addressing model. A future implementation of a Memory Management Unit (MMU) would translate virtual addresses to physical addresses and enforce memory protection.

## 4. System Control and Privilege Levels

To support a future operating system, the VM implements the RISC-V privileged architecture. This system protects the machine's core functions from user programs.

-   **Privilege Modes:** The CPU can be in one of three modes:
    1.  **Machine Mode:** The highest privilege level, used for low-level system setup. The VM starts in this mode.
    2.  **Supervisor Mode:** The level where an operating system kernel would run.
    3.  **User Mode:** The lowest privilege level, where application code runs.

-   **Control and Status Registers (CSRs):** The VM implements a set of special registers that the CPU uses to control its own operation. These are used for managing interrupts, exceptions, and the machine's status. Key implemented CSRs include:
    -   `mstatus` / `sstatus`: Control the CPU's current operating state.
    -   `mepc` / `sepc`: Store the program counter after an exception.
    -   `mcause` / `scause`: Store the reason for an exception or interrupt.
    -   `mtvec` / `stvec`: Hold the address of the code that handles exceptions.

-   **Traps and Exceptions:** The VM correctly handles events that disrupt normal program flow, such as `ecall` (for system calls) or illegal instructions. The CPU traps to a higher privilege level to handle the event.

## 5. Instruction Set (RV64IM)

The assembler and VM correctly encode, decode, and execute the complete RISC-V 64-bit base integer instruction set ("I") and the standard multiplication and division extension ("M").

Instructions are grouped by function:

-   **Integer Computation:** Standard arithmetic (`add`, `sub`, `addi`) and logical (`and`, `or`, `xor`, `slt`) operations.
-   **32-bit Operations:** A key feature of RV64 is a full set of 32-bit operations (`addw`, `subw`, `slliw`, etc.) that produce sign-extended 32-bit results, improving efficiency for 32-bit integer math.
-   **Control Flow:** Conditional branches (`beq`, `bne`, `blt`) and unconditional jumps (`jal`, `jalr`).
-   **Loads and Stores:** Instructions to move data of different sizes (64-bit, 32-bit, 16-bit, 8-bit) between registers and memory (`ld`, `lw`, `lhu`, `lb`, `sd`, `sw`, `sh`, `sb`).
-   **Multiplication & Division (M Extension):** `mul`, `div`, `rem`, and their variants for signed and unsigned arithmetic.
-   **System Instructions:** Instructions for interacting with the system, including `ecall`, `ebreak`, `mret`, `sret`, and the full set of CSR instructions (`csrrw`, `csrrs`, `csrrc`, etc.).

## 6. Assembler and Pseudo-Instructions

The assembler translates standard RISC-V assembly into machine code. To make assembly programming more convenient, it also supports several common pseudo-instructions, which expand into one or more real instructions:

-   `nop`: (No Operation) Does nothing. Expands to `addi zero, zero, 0`.
-   `j <label>`: (Jump) Unconditionally jumps to a label. Expands to `jal zero, <label>`.
-   `la <reg>, <label>`: (Load Address) Loads the address of a label into a register. Expands into an `auipc` and `addi` instruction pair.
-   `ret`: (Return) Returns from a function. Expands to `jalr zero, ra, 0`.

## 7. Calling Convention

To allow functions to call each other safely, the VM's code follows the standard RISC-V calling convention.

-   **Function Arguments & Return:** The first eight arguments are passed in registers `a0` through `a7`. The return value is placed in `a0`.
-   **Stack Management:** Functions create a "stack frame" to save registers before using them and restore them before returning, ensuring that function calls do not have unexpected side effects. This process involves saving the return address (`ra`) and any callee-saved registers (`s0`-`s11`) that will be used.
