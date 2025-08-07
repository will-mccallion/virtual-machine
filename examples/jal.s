main:
    jal ra, foo
    halt

foo:
    addi sp, sp, -4
    sw ra, 0(sp)

    jal ra, bar

    lw ra, 0(sp)
    addi sp, sp, 4

    ret

bar:
    addi s0, zero, 9
    ret
