.text
.globl main
main:
li t0, 6
add t1, x0, t0
xor t2, x0, t1
seqz t2, t2
sub t3, x0, t2
sub t4, x0, t3
mv a0, t4
ret
