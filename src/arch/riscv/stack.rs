//! Abstractions for Stacks

use crate::arch::riscv::KERNEL_STACK_SIZE;
use core::fmt::{self, Debug};

/// A stack of [`STACK_SIZE`], which grows downwards.
#[derive(Copy, Clone)]
#[repr(align(0x100000))]
#[repr(C)]
pub struct Stack {
	buffer: [u8; KERNEL_STACK_SIZE],
}

impl Stack {
	pub const fn new() -> Stack {
		Stack {
			buffer: [0; KERNEL_STACK_SIZE],
		}
	}

	pub fn top(&self) -> usize {
		(&(self.buffer[KERNEL_STACK_SIZE - 16]) as *const _) as usize
	}

	pub fn bottom(&self) -> usize {
		(&(self.buffer[0]) as *const _) as usize
	}
}

impl Debug for Stack {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Stack")
			.field("top", &self.top())
			.field("bottom", &self.bottom())
			.finish()
	}
}

/// A statically allocated boot stack, which we can safely switch to directly
/// after boot.
pub static mut BOOT_STACK: Stack = Stack::new();
