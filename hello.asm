.text
.globl main
main:
addi sp, sp, -96
li t0, 2
sw t0, 0(sp)
lw t0, 0(sp)
sw t0, 4(sp)
lw t0, 4(sp)
li t1, 2
sub t0, t0, t1
sw t0, 8(sp)
lw t0, 8(sp)
sw t0, 12(sp)
lw t0, 0(sp)
sw t0, 16(sp)
lw t0, 16(sp)
bnez t0, then_0
j else_0
then_0:
lw t0, 12(sp)
sw t0, 20(sp)
lw t0, 20(sp)
bnez t0, then_1
j else_1
else_0:
j end_0
then_1:
lw t0, 0(sp)
sw t0, 24(sp)
lw t0, 24(sp)
li t1, 1
add t0, t0, t1
sw t0, 28(sp)
lw t0, 28(sp)
sw t0, 0(sp)
j end_1
else_1:
li t0, 0
sw t0, 0(sp)
j end_1
end_0:
lw t0, 0(sp)
sw t0, 32(sp)
lw a0, 32(sp)
addi sp, sp, 96
ret
end_1:
j end_0
