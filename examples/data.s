.data
my_message:
    .string "Hello from a .bin file!"
my_number:
    .word 1337

.text
main:
    la a0, my_message
    la a1, my_number
    lw a2, 0(a1)
    addi a0, zero, 0
    addi a7, zero, 93
    ecall
