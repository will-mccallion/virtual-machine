main:
    jal ra, foo
    ecall

foo:
    addi sp, sp, -8
    sw ra, 0(sp)

    jal ra, bar

    lw ra, 0(sp)
    addi sp, sp, 8

    ret

bar:
    addi s0, zero, 9
    ret
