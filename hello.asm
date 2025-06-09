.text
.globl main
main:
addi sp, sp, -32
j while_entry0
while_entry0:
li t0, 1
bnez t0, while_body0
j while_end0
while_body0:
j while_end0
while_end0:
li t0, 0
mv a0, t0
addi sp, sp, 32
ret
