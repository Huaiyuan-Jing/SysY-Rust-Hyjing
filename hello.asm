.text
.globl main
main:
addi sp, sp, -48
li t0, 1
sw t0, 0(sp)
li t0, 2
sw t0, 0(sp)
li t0, 3
sw t0, 4(sp)
lw t0, 0(sp)
sw t0, 8(sp)
lw a0, 8(sp)
addi sp, sp, 48
ret
