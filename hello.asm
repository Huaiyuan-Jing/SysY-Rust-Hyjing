.text
.globl main
main:
addi sp, sp, -32
li t0, 1
bnez t0, then_0
j else_0
then_0:
li t0, 1
mv a0, t0
addi sp, sp, 32
ret
else_0:
j end_0
end_0:
li t0, 0
mv a0, t0
addi sp, sp, 32
ret
