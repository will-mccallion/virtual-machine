# =============================================================================
# fib.s
#
# A recursive Fibonacci calculator to test RISC-V assembly and the .align
# directive.
# =============================================================================

.data
    # This byte is here to intentionally misalign the following data.
    # Without .align, fib_n would start at an odd memory address.
    .byte 0 

    # .align 2 ensures that the address is a multiple of 2^2 = 4 bytes.
    # The assembler will insert 3 padding bytes to move the current
    # address from 1 to 4 before defining fib_n.
    # Comment out the line below to see the program fail.
    .align 2

fib_n:
    .word 8 # Calculate the 8th Fibonacci number (which is 21)

.text
.global main
main:
    # Load the address of the data we want to read
    la a0, fib_n
    
    # Load the word value from that address. This will trap if the address
    # of 'fib_n' is not a multiple of 4.
    lw a0, 0(a0)

    # Call the fibonacci function with the loaded value (8)
    jal ra, fib

    # When fib returns, the result (21) will be in a0.
    # We will use this as the exit code.
    addi a7, zero, 93   # ecall service number 93 is 'exit'
    ecall               # Exit the program. The exit code is in a0.

# =============================================================================
# fib: Recursive Fibonacci function
#
# Args:
#   a0: The number (n) to calculate the Fibonacci value for.
#
# Returns:
#   a0: The calculated Fibonacci value, fib(n).
#
# Note: fib(0)=0, fib(1)=1, fib(2)=1, fib(3)=2, ...
# =============================================================================
fib:
    ebreak
    addi sp, sp, -24        # Make room on stack for 3 registers

    sd s0, 0(sp)            # Store saved register s0 (our n)
    sd s1, 8(sp)            # Store saved register s1 (fib(n-1))
    sd ra, 16(sp)           # Store the return address

    # Base case: if n < 2, return n
    addi t0, zero, 2
    blt a0, t0, end_fib     # If a0 < 2, jump to the end

    # Recursive step
    add s0, a0, zero        # Save n in s0: s0 = n

    addi a0, s0, -1         # Set up arg for first recursive call: a0 = n - 1
    jal ra, fib             # a0 = fib(n-1)

    add s1, a0, zero        # Save result: s1 = fib(n-1)

    addi a0, s0, -2         # Set up arg for second call: a0 = n - 2
    jal ra, fib             # a0 = fib(n-2)

    add a0, s1, a0          # a0 = fib(n-1) + fib(n-2)

end_fib:
    # Restore registers from stack and return
    ld s0, 0(sp)
    ld s1, 8(sp)
    ld ra, 16(sp)

    addi sp, sp, 24         # Restore stack pointer
    ret                     # Return to caller
