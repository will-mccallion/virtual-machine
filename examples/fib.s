main:
    addi a0, zero, 8
    jal ra, fib             # Calculate fib(8) and put it in a0
    ecall

#
# fib
#
# Args:
#   a0: fibonacci value to calculate (n)
#
# Returns:
#   a0: fibonacci value calculated
#
fib:
    addi sp, sp, -12        # Make room on stack

    sw s0, 0(sp)            # Store saved registers
    sw s1, 4(sp)
    sw ra, 8(sp)

    addi t0, zero, 2
    blt a0, t0, end_fib     # If n < 2 goto end_fib

    add s0, a0, zero        # s0 (n) <- a0

    addi a0, s0, -1         # a0 <- n - 1
    jal ra, fib             # a0 <- fib(n-1)

    add s1, a0, zero        # s1 <- fib(n-1)

    addi a0, s0, -2         # a0 <- n - 2
    jal ra, fib             # a0 <- fib(n-2)

    add a0, s1, a0          # a0 <- fib(n-1) + fib(n-2)

end_fib:
    lw s0, 0(sp)            # Restore registers from stack
    lw s1, 4(sp)
    lw ra, 8(sp)

    addi sp, sp, 12         # Restore stack pointer
    ret
