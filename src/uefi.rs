use qemu_exit::QEMUExit;
use crate::arch;
use core::{cmp, fmt::Write, mem, slice};
#[allow(unused_imports)]
use hermit_entry::elf::KernelObject;
use log::info;
use uefi::{
	Identify,
	prelude::*,
	proto::console::gop::GraphicsOutput,
	table::{boot::*, cfg},
};

/// Entry Point of the UEFI Loader
/// This function gets a so-called "EFI System Table" (see UEFI Specification, Section 4: EFI System Table) from the Firmware Interface.
/// Here, the RSDP (for BOOT_INFO) and the kernel are located and the kernel is parsed and loaded into memory.
/// After that, free physical memory for the kernel to use is looked for and saved for BOOT_INFO as well.
/// After that, the architecture specific boot function is called.
#[entry]
unsafe fn loader_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
	// initialize Hermits Log functionality
	uefi_services::init(&mut system_table).unwrap();

	log::info!("Hello, UEFI!");

	// let custom_exit_success = 3;
	// let qemu_exit_handle = qemu_exit::X86::new(0xf4, custom_exit_success);
	// qemu_exit_handle.exit_success()
	
	info!("Hello World from UEFI boot!");
	// let stdout = system_table.stdout();
	// stdout.clear().unwrap();
	// writeln!(stdout, "Hello World! This is the bootloader").unwrap();

	
	let bs = system_table.boot_services();
	let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
	let mut gop = bs.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
	// // for g in gop.modes(){
	// // 	info!("gop_handle modes: {:#?}", g.info());
	// // }
	// let gop_mode = gop.query_mode(0).unwrap();
	// //gop.set_mode(&gop_mode).unwrap();
	let mut framebuffer = gop.frame_buffer();
	let mut framebuf_ptr = framebuffer.as_mut_ptr();
	let framebuf_size = framebuffer.size();
	info!("framebuf_ptr: {framebuf_ptr:?}");
	for i in 0..100000 {
	unsafe {framebuffer.write_byte(i, 110)};
	
	}

	drop(gop);
	// look for the rsdp in the EFI system table before calling exit boot services (see UEFI specification for more)
	let rsdp_addr = {
		// returns an iterator to the config table entries which in turn point to other system-specific tables
		let mut cfg_entries = system_table.config_table().iter();
		// look for ACPI2 RSDP first
		let acpi2_addr = cfg_entries.find(|entry| entry.guid == cfg::ACPI2_GUID);
		// takes the value of either acpi2_addr or (if it does not exist) ACPI1 address
		let rsdp = acpi2_addr.or(cfg_entries.find(|entry| entry.guid == cfg::ACPI_GUID));
		rsdp.map(|entry| entry.address as u64)
			.expect("no RSDP address found")
	};
	info!("RSDP found at {rsdp_addr:#x}");
	info!("Locating kernel");

	let kernel = KernelObject::parse(arch::find_kernel()).unwrap();
	info!("Kernel parsed!");
	let filesize = kernel.mem_size();
	info!("Kernelsize: {:#?}", filesize);
	let kernel_addr = system_table
		.boot_services()
		.allocate_pages(
			AllocateType::AnyPages,
			MemoryType::LOADER_DATA,
			(filesize / 4096) + 1,
		)
		.unwrap();
	let kernel_addr = kernel.start_addr().unwrap_or(kernel_addr);
	info!("Kernel located at {:#x}", kernel_addr);
	let memory = slice::from_raw_parts_mut(kernel_addr as *mut mem::MaybeUninit<u8>, filesize);

	let kernel_info = kernel.load_kernel(memory, memory.as_ptr() as u64);
	info!("Kernel loaded into memory");

	// exit boot services for getting a runtime view of the system table and an iterator to the UEFI memory map
	let (runtime_system_table, mut memory_map) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

	for i in 100000..200000 {
		unsafe { framebuf_ptr.add(i).write_volatile(50) }
	}

	memory_map.sort();
	let mut entries = memory_map.entries();
	let mut clone = entries.clone();
	let mut size = 0;
	let mut max_size = 0;
	for index in entries {
		if index.ty.eq(&uefi::table::boot::MemoryType(7)) {
			size = index.page_count;			
			if size > max_size {
				max_size = size;
			}
		}
	}

	let start_address = clone
		.find(|&&x| x.page_count == max_size)
		.unwrap()
		.phys_start as usize;
	let end_address = (start_address + (max_size as usize * 0x1000 as usize) - 1) as usize;

	info!("Kernelmemory: start: {start_address:#x?}, end: {end_address:#x?}");

	// Jump into actual booting routine
	
	arch::boot_kernel(
		rsdp_addr,
		kernel_addr,
		filesize,
		kernel_info,
		runtime_system_table,
		start_address,
		end_address,

	)
}

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
// 	// We can't use `println!` or related macros, because `_print` unwraps a result and might panic again
// 	writeln!(unsafe { &mut console::CONSOLE }, "[LOADER] {info}").ok();

// 	loop {}
// }

