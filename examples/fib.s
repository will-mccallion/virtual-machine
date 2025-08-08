main:
    addi a0, zero, 8
    jal ra, fib
    ecall

fib:
    addi sp, sp, -12

    sw s0, 0(sp)
    sw s1, 4(sp)
    sw ra, 8(sp)

    addi t0, zero, 1
    beq a0, t0, end_fib
    addi t0, zero, 0
    beq a0, t0, end_fib

    add s0, a0, zero

    addi a0, s0, -1
    jal ra, fib

    add s1, a0, zero

    addi a0, s0, -2
    jal ra, fib

    add a0, s1, a0

end_fib:
    lw s0, 0(sp)
    lw s1, 4(sp)
    lw ra, 8(sp)

    addi sp, sp, 12
    ret
