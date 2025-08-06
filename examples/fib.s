.data
fib_n:
    .word 8

.text
main:
    la a0, fib_n
    lw a0, 0(a0)
    jal ra, fib
    addi a0, zero, 0
    addi a7, zero, 93
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
    addi sp, sp, -24        # Make room on stack

    sd s0, 0(sp)            # Store saved registers
    sd s1, 8(sp)
    sd ra, 16(sp)

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
    add t0, a0, zero
    ld s0, 0(sp)            # Restore registers from stack
    ld s1, 8(sp)
    ld ra, 16(sp)

    addi sp, sp, 24         # Restore stack pointer
    ret
