pub(crate) mod allocator;
mod console;

use core::fmt::Write;
use core::mem::MaybeUninit;
use core::{ptr, slice};

use hermit_entry::elf::KernelObject;
use log::info;

pub use self::console::CONSOLE;
use crate::arch;
use crate::os::ExtraBootInfo;

unsafe extern "C" {
	static loader_end: u8;
	static loader_start: u8;
}

/// Entry Point of the BIOS Loader
/// (called from entry.asm or entry.rs)
pub(crate) unsafe extern "C" fn loader_main() -> ! {
	crate::log::init();
	crate::log_built_info();

	unsafe {
		info!("Loader: [{:p} - {:p}]", &loader_start, &loader_end);
	}

	let kernel = arch::find_kernel();
	let mut buf = None;
	// TODO: handle config
	let (kernel, _) = crate::resolve_kernel(kernel, &mut buf);

	let kernel = KernelObject::parse(kernel).unwrap();

	let mem_size = kernel.mem_size();
	let kernel_addr = unsafe { arch::get_memory(mem_size as u64) };
	let kernel_addr = kernel.start_addr().unwrap_or(kernel_addr);
	let memory = unsafe {
		slice::from_raw_parts_mut(
			ptr::with_exposed_provenance_mut::<MaybeUninit<u8>>(kernel_addr as usize),
			mem_size,
		)
	};

	let kernel_info = kernel.load_kernel(memory, memory.as_ptr() as u64);

	let mut extra_info = ExtraBootInfo::default();
	if let Some(tar_image) = buf {
		let tar_image = alloc::boxed::Box::leak(tar_image);
		extra_info.image = Some(&*tar_image);
	}

	unsafe { arch::boot_kernel(kernel_info, extra_info) }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
	// We can't use `println!` or related macros, because `_print` unwraps a result and might panic again
	writeln!(crate::os::CONSOLE.lock(), "[LOADER] {info}").ok();

	loop {}
}
