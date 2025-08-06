# A program that loops 3 times decrementing register x1 from 5 to 2
main:
    addi x1, x0, 5
    addi x2, x0, 2
start:
    addi x1, x1, -1
    beq x1, x2, done
    beq x0, x0, start
done:
    halt
