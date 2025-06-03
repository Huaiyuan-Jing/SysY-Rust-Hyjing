.text
.globl main
main:
li t0, 3
sub t1, x0, t0
li t2, 2
mul t3, t2, t1
li t4, 1
add t5, t4, t3
li t6, 12
mul t7, t5, t6
mv a0, t7
ret
