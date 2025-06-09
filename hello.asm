.text
.globl main
main:
addi sp, sp, -64
li t0, 0
sw t0, 0(sp)
j while_entry0
while_entry0:
lw t0, 0(sp)
sw t0, 4(sp)
lw t0, 4(sp)
li t1, 10
slt t0, t0, t1
sw t0, 8(sp)
lw t0, 4(sp)
li t1, 10
slt t0, t0, t1
sw t0, 12(sp)
bnez t0, while_body0
j while_end0
while_body0:
lw t0, 0(sp)
sw t0, 16(sp)
lw t0, 16(sp)
li t1, 1
add t0, t0, t1
sw t0, 20(sp)
lw t0, 20(sp)
sw t0, 0(sp)
j while_entry0
while_end0:
lw t0, 0(sp)
sw t0, 24(sp)
lw a0, 24(sp)
addi sp, sp, 64
ret
