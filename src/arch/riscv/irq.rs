use riscv::register::*;
use trapframe::TrapFrame;

/// Init Interrupts
pub fn install() {
	unsafe {
		trapframe::init();
	}
}

/// Enable Interrupts
#[inline]
pub fn enable() {
	unsafe {
		sstatus::set_sie();
	}
}

/// Disable Interrupts
#[inline]
pub fn disable() {
	unsafe { sstatus::clear_sie() };
}

/// Disable IRQs (nested)
///
/// Disable IRQs when unsure if IRQs were enabled at all.
/// This function together with nested_enable can be used
/// in situations when interrupts shouldn't be activated if they
/// were not activated before calling this function.
#[inline]
pub fn nested_disable() -> bool {
	let was_enabled = sstatus::read().sie();

	disable();
	was_enabled
}

/// Enable IRQs (nested)
///
/// Can be used in conjunction with nested_disable() to only enable
/// interrupts again if they were enabled before.
#[inline]
pub fn nested_enable(was_enabled: bool) {
	if was_enabled {
		enable();
	}
}

//Derived from rCore: https://github.com/rcore-os/rCore
/// Dispatch and handle interrupt.
///
/// This function is called from `trap.S` which is in the trapframe crate.
#[no_mangle]
pub extern "C" fn trap_handler(tf: &mut TrapFrame) {
	let scause = scause::read();
	let stval = stval::read();
	//trace!("Interrupt @ CPU{}: {:?} ", super::cpu::id(), scause.cause());
	loaderlog!("Interrupt: {:?} ", scause.cause());
	match scause.cause() {
		// The trap occurred before the kernel sets stvec => panic!
		_ => panic!(
			"unhandled trap {:?}, stval: {:x} tf {:#x?}",
			scause.cause(),
			stval,
			tf
		),
	}
}
