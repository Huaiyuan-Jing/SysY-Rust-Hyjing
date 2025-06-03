.text
.globl main
main:
li t0, 3
xor t1, x0, t0
seqz t1, t1
xor t2, x0, t1
snez t2, t2
li t3, 4
xor t4, x0, t3
snez t4, t4
or t5, t2, t4
li t6, 2
mul t7, t6, t5
li t8, 1
add t9, t8, t7
li t10, 5
slt t11, t9, t10
li t12, 6
xor t13, t11, t12
snez t13, t13
li t14, 7
sub t15, x0, t14
xor t16, x0, t13
snez t16, t16
xor t17, x0, t15
snez t17, t17
and t18, t16, t17
mv a0, t18
ret
