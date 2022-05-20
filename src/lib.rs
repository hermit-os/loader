#![feature(alloc_error_handler)]
#![cfg_attr(target_arch = "aarch64", feature(asm_const))]
#![allow(incomplete_features)]
#![feature(specialization)]
#![no_std]
#![warn(rust_2018_idioms)]
#![allow(clippy::missing_safety_doc)]

// EXTERNAL CRATES
#[macro_use]
extern crate alloc;

#[cfg(target_arch = "x86_64")]
#[macro_use]
extern crate bitflags;

// MODULES
#[macro_use]
pub mod macros;

pub mod arch;
pub mod console;
pub mod kernel;
#[cfg(target_os = "none")]
mod runtime_glue;

#[cfg(target_os = "none")]
use core::ptr;

#[cfg(target_os = "none")]
use static_alloc::Bump;

/// A simple bump memory allocator using static storage.
///
/// This allocator is used for bootstrapping.
/// It manages a slice of uninitialized memory and cannot deallocate.
/// The kernel will setup a proper memory allocator later.
#[cfg(target_os = "none")]
#[global_allocator]
static ALLOCATOR: Bump<[u8; 2 * 1024 * 1024]> = Bump::uninit();

// FUNCTIONS
#[cfg(target_os = "none")]
pub unsafe fn sections_init() {
	extern "C" {
		static bss_end: u8;
		static mut bss_start: u8;
	}

	// Initialize .bss section
	ptr::write_bytes(
		&mut bss_start as *mut u8,
		0,
		&bss_end as *const u8 as usize - &bss_start as *const u8 as usize,
	);
}
