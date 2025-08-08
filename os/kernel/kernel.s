# RustezeOS Kernel v0.2
# A monolithic kernel source file using assembler directives.

# =============================================================================
# Section 1: The Trap Handler
# This code MUST live at the hardware-defined trap vector address.
# =============================================================================
.org 0x0100
ecall_handler:
    # Syscall 99: Exit
    li t0, 99
    beq a0, t0, handle_exit
    halt # Unknown syscall

handle_exit:
    halt

# =============================================================================
# Section 2: The Main Kernel Code
# This is the first code executed on boot.
# =============================================================================
.org 0xC000
_start:
    # --- 1. Load the size of the user program from its variable ---
    li s3, _user_program_size_addr # s3 = address of the size variable
    lw s2, 0(s3)                   # s2 = The actual size value

    # --- 2. Copy the user program to its correct memory location ---
    li s0, _user_program_start      # s0 = Source address (in our data section)
    li s1, 0x1000                   # s1 = Destination address (user space)

copy_loop:
    beq s2, zero, copy_done     # If bytes to copy is 0, we're done.
    lb t0, 0(s0)                # Load a byte from the source
    sb t0, 0(s1)                # Store it at the destination
    addi s0, s0, 1              # Move to the next source byte
    addi s1, s1, 1              # Move to the next destination byte
    addi s2, s2, -1             # Decrement the count
    jal zero, copy_loop

copy_done:
    # --- 3. Set up the user program's environment ---
    li sp, 0xC000               # User stack starts just below kernel code

    # --- 4. Run the user program ---
    li t0, 0x1000
    jal zero, t0                # Jump to 0x1000

# =============================================================================
# Section 3: Kernel Data
# =============================================================================
.org 0xD000
_user_program_start:
    .incbin "os/programs/bin/helloworld.bin"
_user_program_end:

.org 0xD800
_user_program_size_addr:
    .dword _user_program_end - _user_program_start
