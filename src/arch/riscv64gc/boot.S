# boot.S
# bootloader for SoS
# Stephen Marz
# 8 February 2019
.option norvc
.section .data
.global TOTAL_NUM_HARTS
TOTAL_NUM_HARTS: .byte 1

.global HEAP_SIZE
HEAP_SIZE: .dword _heap_size

.global HEAP_START
HEAP_START: .dword _heap_start

.section .text.init
.global _start
_start:
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f
	# SATP should be zero, but let's make sure
	csrw	satp, zero
.option push
.option norelax
	la		gp, _global_pointer
.option pop
	# The BSS section is expected to be zero
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	# Control registers, set the stack, mstatus, mepc,
	# and mtvec to return to the main function.
	# li		t5, 0xffff;
	# csrw	medeleg, t5
	# csrw	mideleg, t5
	la		sp, _stack_end
	# We use mret here so that the mstatus register
	# is properly updated.
	li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
	csrw	mstatus, t0
	la		t1, loader_main
	csrw	mepc, t1
	la		t2, asm_trap_vector
	csrw	mtvec, t2
	li		t3, (1 << 3) | (1 << 7) | (1 << 11)
	csrw	mie, t3
	la		ra, 4f
	mret
3:

	# Parked harts go here. We need to set these
	# to only awaken if it receives a software interrupt,
	# which we're going to call the SIPI (Software Intra-Processor Interrupt).
	# We only use these to run user-space programs, although this may
	# change.
	la		t0, TOTAL_NUM_HARTS
	lb		t1, 0(t0)
	addi	t1, t1, 1
	sb		t1, 0(t0)

	li		t0, 1 << 3
	csrw	mstatus, t0
	li		t1, 1 << 3
	csrw	mie, t1
	la		t2, cpu_ipi
	csrw	mtvec, t2
	# Waiting loop. An IPI will cause the other harts to
	# wake up and start executing.
4:
	wfi
	j		4b
