#![no_std]
#![no_main]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]
#![allow(unstable_name_collisions)]
#![allow(clippy::missing_safety_doc)]

use ::log::info;
use hermit_entry::boot_info::{BootInfo, RawBootInfo};

#[macro_use]
mod macros;

mod arch;

mod bump_allocator;

#[cfg(any(target_os = "uefi", target_arch = "x86_64"))]
mod fdt;
mod log;
mod os;

#[cfg(any(target_os = "uefi", all(target_arch = "x86_64", target_os = "none")))]
extern crate alloc;

trait BootInfoExt {
	fn write(self) -> &'static RawBootInfo;
}

impl BootInfoExt for BootInfo {
	fn write(self) -> &'static RawBootInfo {
		info!("boot_info = {self:#x?}");

		take_static::take_static! {
			static RAW_BOOT_INFO: Option<RawBootInfo> = None;
		}

		let raw_boot_info = RAW_BOOT_INFO.take().unwrap();

		raw_boot_info.insert(RawBootInfo::from(self))
	}
}

#[doc(hidden)]
fn _print(args: core::fmt::Arguments<'_>) {
	use core::fmt::Write;

	self::os::CONSOLE.lock().write_fmt(args).unwrap();
}

/// Detects the input format are resolves the kernel
fn resolve_kernel<'a, A: allocator_api2::alloc::Allocator>(
	input_blob: &'a [u8],
	alloc: A,
	buf: &'a mut Option<allocator_api2::boxed::Box<hermit_entry::tar_parser::Bytes, A>>,
) -> (&'a [u8], Option<hermit_entry::config::Config>) {
	use hermit_entry::{Format, decompress_image_with_allocator, detect_format};
	match detect_format(input_blob) {
		Some(Format::Elf) => (input_blob, None),

		Some(Format::Gzip) => {
			*buf = Some(
				decompress_image_with_allocator(input_blob, alloc)
					.expect("Unable to decompress Hermit gzip image"),
			);
			match *buf {
				Some(ref mut tmp) => {
					let (config, kernel) = hermit_entry::config::handle_config(tmp)
						.expect("Unable to find Hermit image configuration + kernel");

					// TODO: do we just let the kernel handle the config

					(kernel, Some(config))
				}
				None => unreachable!(),
			}
		}

		None => {
			panic!("Input BLOB has unknown magic bytes (neither Gzip nor ELF)")
		}
	}
}
