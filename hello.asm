.text
.globl main
main:
li t0, 1
li t1, 2
slt t2, t1, t0
xori t2, t2, 1
li t3, 1
sub t4, x0, t3
li t5, 3
add t6, t5, t4
li t7, 4
slt t8, t7, t6
slt t9, x0, t2
slt t10, x0, t8
and t11, t9, t10
mv a0, t11
ret
