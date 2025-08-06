# A program that loops 3 times decrementing register x1 from 5 to 2
main:
    addi t1, zero, 5
    addi t2, zero, 2
start:
    addi t1, t1, -1
    beq t1, t2, done
    beq x0, x0, start
done:
    ecall
