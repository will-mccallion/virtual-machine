# bios.s (Final Version for Limited Toolchains)

.section .data
# --- 64-bit Constants for Boot Process ---
STACK_POINTER_ADDR:   .quad 0x88000000
ROOT_PAGE_TABLE:      .quad 0x80010000
PAGE_TABLE_CLEAR_SIZE:.quad 8192
SATP_PPN_VALUE:       .quad 0x80010
SATP_MODE_SV39_VALUE: .quad 0x8000000000000000
VIRTUAL_DISK_ADDR:    .quad 0x90000000
KERNEL_LOAD_ADDR:     .quad 0x80100000
DISK_SIZE_REG_ADDR:   .quad 0x90001000
CORRECT_PTE_VALUE:    .quad 0x2000000F

# --- Constants for Privilege Drop (avoids large `li` and `~`) ---
# A mask of all 1s, used to delegate all exceptions and interrupts.
DELEGATION_MASK:      .quad -1

# A mask for the mstatus.MPP bits (bits 11 and 12).
# Used with `csrrc` to clear the previous privilege level setting.
MSTATUS_MPP_MASK:     .quad 0x1800

# The value to set mstatus.MPP to for Supervisor Mode (0b01 << 11).
# Used with `csrrs` to set the new privilege level.
MSTATUS_MPP_S_MODE:   .quad 0x800

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
    addi t0, t0, 16

    # Load the pre-calculated correct PTE and store it.
    la t1, CORRECT_PTE_VALUE
    ld t1, 0(t1)
    sd t1, 0(t0)

    # 4. Enable Paging!
    la t0, SATP_PPN_VALUE
    ld t0, 0(t0)
    la t1, SATP_MODE_SV39_VALUE
    ld t1, 0(t1)
    or t0, t0, t1
    csrrw zero, satp, t0

    # --- Stage 2: Running in Virtual Memory (MMU is ON) ---

    # 5. Load the kernel from the virtual disk.
    la t1, DISK_SIZE_REG_ADDR
    ld t1, 0(t1)
    ld a2, 0(t1)
    la a0, VIRTUAL_DISK_ADDR
    ld a0, 0(a0)
    la a1, KERNEL_LOAD_ADDR
    ld a1, 0(a1)
    beq a2, zero, prepare_s_mode

load_loop:
    lb t0, 0(a0)
    sb t0, 0(a1)
    addi a0, a0, 1
    addi a1, a1, 1
    addi a2, a2, -1
    bne a2, zero, load_loop

prepare_s_mode:
    # --- Stage 3: Prepare Handoff to Supervisor-Mode Kernel ---

    # 6. Delegate all exceptions and interrupts to S-mode.
    la t0, DELEGATION_MASK
    ld t0, 0(t0)
    # Use `csrrw` to write the value from t0 into the CSR.
    # `csrrw rd, csr, rs1` writes rs1 to csr, and reads the old value into rd.
    # Using `zero` for rd means we discard the old value.
    csrrw zero, medeleg, t0
    csrrw zero, mideleg, t0

    # 7. Set mstatus.MPP to Supervisor Mode (0b01).
    # This is a two-step process using base instructions.
    # First, clear the MPP bits using the mask and `csrrc`.
    la t1, MSTATUS_MPP_MASK
    ld t1, 0(t1)         # t1 = 0x1800
    # `csrrc zero, csr, rs1` clears bits in csr specified by rs1.
    csrrc zero, mstatus, t1

    # Second, set the MPP bits to S-mode using the value and `csrrs`.
    la t2, MSTATUS_MPP_S_MODE
    ld t2, 0(t2)         # t2 = 0x800
    # `csrrs zero, csr, rs1` sets bits in csr specified by rs1.
    csrrs zero, mstatus, t2

    # 8. Set mepc to the kernel's entry point.
    la t0, KERNEL_LOAD_ADDR
    ld t0, 0(t0)
    csrrw zero, mepc, t0

    # 9. Drop privilege and jump to the kernel.
    mret

hang:
    # This should never be reached.
    jal zero, hang
