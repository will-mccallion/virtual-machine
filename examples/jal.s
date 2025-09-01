.text
main:
    jal ra, foo
    li a7, 93
    ecall

foo:
    addi sp, sp, -8
    sd ra, 0(sp)

    jal ra, bar

    ld ra, 0(sp)
    addi sp, sp, 8

    ret

bar:
    addi s0, zero, 9
    ret
