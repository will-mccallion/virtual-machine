# kernel.s (Basic Test Kernel)

# This kernel assumes it is loaded by the BIOS at address 0x80100000
# and is running in Supervisor Mode.

.section .data
# Define the memory-mapped address for the UART (serial port) transmitter.
# Writing a byte to this address will print it to the console.
# This is a common address used in emulators like QEMU.
UART_TX_ADDR: .quad 0x10000000

# The message to be printed. .asciz adds a null terminator automatically.
kernel_msg: .asciz "--- Kernel Started! ---"

.section .text
.global _start

_start:
    # Load the address of our UART device. We use the la/ld pattern
    # to handle the 64-bit address, just like in the BIOS.
    la a0, UART_TX_ADDR
    ld a0, 0(a0)

    # Load the address of the message we want to print.
    la a1, kernel_msg

    # Call our simple print routine.
    jal ra, print_string

    # To exit cleanly, we use the standard RISC-V exit system call.
    # The VM is set up to recognize this ecall and terminate.
    li a0, 0      # a0 holds the exit code (0 for success)
    li a7, 93     # a7 holds the syscall number for "exit"
    ecall         # Trigger the system call

# A simple infinite loop in case the ecall fails for some reason.
hang:
    jal zero, hang


# -----------------------------------------------------------------------------
# print_string:
# A simple subroutine to print a null-terminated string to the UART.
#
# Arguments:
#   a0: The base address of the UART transmit register.
#   a1: The address of the null-terminated string to print.
# -----------------------------------------------------------------------------
print_string:
print_loop:
    # Load one byte from the string
    lb t0, 0(a1)
    # If the byte is zero (null terminator), we are done.
    beq t0, zero, print_done
    # Write the byte to the UART's transmit register.
    sb t0, 0(a0)
    # Move to the next character in the string.
    addi a1, a1, 1
    # Loop back.
    jal zero, print_loop
print_done:
    # Return from the subroutine.
    ret
