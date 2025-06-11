.text
.globl half
half:
addi sp, sp, -32
sw ra 28(sp)
lw t0, 32(sp)
sw t0, 0(sp)
lw t0, 0(sp)
sw t0, 4(sp)
lw t0, 4(sp)
li t1, 2
div t0, t0, t1
sw t0, 8(sp)
lw a0, 8(sp)
lw ra 28(sp)
addi sp, sp, 32
ret
.text
.globl f
f:
addi sp, sp, -16
sw ra 12(sp)
lw ra 12(sp)
addi sp, sp, 16
ret
.text
.globl main
main:
addi sp, sp, -32
sw ra 28(sp)
call f
sw a0, 4(sp)
lw t0, 4(sp)
li t0, 10
sw t0, 0(sp)
call half
sw a0, 8(sp)
lw t0, 8(sp)
lw a0, 8(sp)
lw ra 28(sp)
addi sp, sp, 32
ret
