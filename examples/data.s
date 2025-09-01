.data
my_message:
    .asciz "Hello from a .bin file!"
my_number:
    .word 1337

.text
main:
    la t0, my_message
    la t1, my_number
    lw t2, 0(t1)
    addi a0, zero, 0
    addi a7, zero, 93
    ecall
