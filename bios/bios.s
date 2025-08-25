# bios.s

# The .data section is where we will store our full 64-bit addresses.
# Your assembler will place this data in the binary file after the text section.
.section .data
STACK_POINTER_ADDR: .quad 0x88000000      # Top of 128MB RAM
VIRTUAL_DISK_ADDR:  .quad 0x90000000
KERNEL_LOAD_ADDR:   .quad 0x80100000
KERNEL_SIZE:        .quad 8              # A small, safe size for our tiny kernel

.section .text
.global _start

_start:
    # 1. Initialize the stack pointer.
    #    `la` gets the address of our constant.
    #    `ld` loads the 64-bit value from that address.
    la sp, STACK_POINTER_ADDR
    ld sp, 0(sp)

    # 2. Load the kernel by loading the addresses and size from the .data section.
    la a0, VIRTUAL_DISK_ADDR
    ld a0, 0(a0)            # a0 now correctly holds 0x90000000

    la a1, KERNEL_LOAD_ADDR
    ld a1, 0(a1)            # a1 now correctly holds 0x80100000

    la a2, KERNEL_SIZE
    ld a2, 0(a2)            # a2 now correctly holds 16

load_loop:
    lb t0, 0(a0)           # Load one byte from source
    sb t0, 0(a1)           # Store one byte to destination
    addi a0, a0, 1         # Increment source pointer
    addi a1, a1, 1         # Increment destination pointer
    addi a2, a2, -1        # Decrement byte counter
    bne a2, zero, load_loop     # Loop until counter is zero

    # 3. Jump to the Operating System's entry point.
    #    We already loaded the address into a1, but we'll reload it
    #    into t0 for the jump, which is good practice.
    la t0, KERNEL_LOAD_ADDR
    ld t0, 0(t0)
    jalr zero, 0(t0)       # Jump and give control to the OS

# Should never be reached. If the OS returns, we hang.
hang:
    j hang
