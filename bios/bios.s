# bios.s (Complete and Correct Version)

.section .data
# --- 64-bit Constants (Originals, all required) ---
STACK_POINTER_ADDR:   .quad 0x88000000
ROOT_PAGE_TABLE:      .quad 0x80010000
PAGE_TABLE_CLEAR_SIZE:.quad 8192
SATP_PPN_VALUE:       .quad 0x80010
SATP_MODE_SV39_VALUE: .quad 0x8000000000000000
VIRTUAL_DISK_ADDR:    .quad 0x90000000
KERNEL_LOAD_ADDR:     .quad 0x80100000
DISK_SIZE_REG_ADDR:   .quad 0x90001000

# --- NEW CONSTANT FOR THE FIX ---
# The final, pre-calculated, correct PTE value for the 1GB gigapage.
# PPN = (0x80000000 >> 30) = 2.
# PTE = (PPN << 28) | Valid(1) | Read(2) | Write(4) | Execute(8)
# PTE = (2 << 28) | 15 = 0x20000000 | 0xF = 0x2000000F
CORRECT_PTE_VALUE:    .quad 0x2000000F

# --- CONSTANTS FOR DEBUGGING ---
DEBUG_MAGIC_ADDR:     .quad 0x80001000
DEBUG_MAGIC_VALUE:    .quad 0xDEADBEEF

.section .text
.global _start

_start:
    # --- Stage 1: Running in Physical Memory (MMU is OFF) ---

    # 1. Setup Stack Pointer
    la sp, STACK_POINTER_ADDR
    ld sp, 0(sp)

    # 2. Clear the page table memory.
    la t0, ROOT_PAGE_TABLE
    ld t0, 0(t0)
    la t1, PAGE_TABLE_CLEAR_SIZE
    ld t1, 0(t1)
    addi t2, zero, 0
clear_loop:
    sd t2, 0(t0)
    addi t0, t0, 8
    addi t1, t1, -8
    bne t1, zero, clear_loop

    # 3. Create the identity map for the first 1GB of memory.
    la t0, ROOT_PAGE_TABLE
    ld t0, 0(t0)
    addi t0, t0, 16 # Go to entry index 2 for VA 0x80000000

    la t1, CORRECT_PTE_VALUE
    ld t1, 0(t1)         # t1 now holds the correct value: 0x2000000F
    sd t1, 0(t0)         # Store this correct PTE into the page table.

    # 4. Enable Paging!
    la t0, SATP_PPN_VALUE
    ld t0, 0(t0)
    la t1, SATP_MODE_SV39_VALUE
    ld t1, 0(t1)
    or t0, t0, t1
    csrrw zero, satp, t0

    # --- Stage 2: Running in Virtual Memory (MMU is ON) ---

    # 5. Load and jump to the kernel.
    la t1, DISK_SIZE_REG_ADDR
    ld t1, 0(t1)
    ld a2, 0(t1)

    la a0, VIRTUAL_DISK_ADDR
    ld a0, 0(a0)

    la a1, KERNEL_LOAD_ADDR
    ld a1, 0(a1)

    beq a2, zero, jump_to_kernel

load_loop:
    lb t0, 0(a0)
    sb t0, 0(a1)
    addi a0, a0, 1
    addi a1, a1, 1
    addi a2, a2, -1
    bne a2, zero, load_loop

jump_to_kernel:
    la t0, KERNEL_LOAD_ADDR
    ld t0, 0(t0)
    jalr zero, 0(t0)

hang:
    jal zero, hang
