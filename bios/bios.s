# bios.s (Final, Corrected Version)

.section .data
STACK_POINTER_ADDR: .quad 0x88000000
VIRTUAL_DISK_ADDR:  .quad 0x90000000
KERNEL_LOAD_ADDR:   .quad 0x80100000
DISK_SIZE_REG_ADDR: .quad 0x90001000

.section .text
.global _start

_start:
    # 1. Initialize the stack pointer.
    la sp, STACK_POINTER_ADDR
    ld sp, 0(sp)

    # 2. Get kernel size from the virtual device's MMIO register.
    #    This requires two loads:
    #    - First, load the ADDRESS of the MMIO register from our .data section.
    #    - Second, perform the MMIO read FROM that address.
    la t1, DISK_SIZE_REG_ADDR   # t1 = address of the constant (e.g., 0x80000078)
    ld t1, 0(t1)                # t1 = value of the constant (0x90001000)
    ld a2, 0(t1)                # a2 = value FROM MMIO addr 0x90001000 (this should be 8!)

    # 3. Load the kernel source and destination addresses.
    la a0, VIRTUAL_DISK_ADDR
    ld a0, 0(a0)

    la a1, KERNEL_LOAD_ADDR
    ld a1, 0(a1)

    # 4. Check if there's anything to load. If size is 0, skip to jump.
    beq a2, zero, jump_to_kernel

load_loop:
    lb t0, 0(a0)
    sb t0, 0(a1)
    addi a0, a0, 1
    addi a1, a1, 1
    addi a2, a2, -1
    bne a2, zero, load_loop

jump_to_kernel:
    # 5. Jump to the Operating System's entry point.
    la t0, KERNEL_LOAD_ADDR
    ld t0, 0(t0)
    jalr zero, 0(t0)

hang:
    j hang
