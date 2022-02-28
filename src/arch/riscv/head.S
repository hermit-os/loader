//Adapted from Linux (/arch/riscv/kernel/head.S)

.section .text._start

.global _start
_start:
    j _rust_start
    .word 0
    .balign 8
    .dword 0x200000 //Image load offset(2MB) from start of RAM
    .dword kernel_end - 0x80200000 //Effective size of kernel image
	.dword 1 << 0 // header flags
	.word (0 << 16 | 2) //version
	.word 0
	.dword 0
	.ascii "RISCV\0\0\0" // magic1
	.balign 4
	.ascii "RSC\x05" // magic2
    .word 0