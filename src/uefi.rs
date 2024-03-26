extern crate alloc;
use crate::{arch, framebuffer::*};
//use crate::arch;
use alloc::vec::Vec;
use core::{mem, slice};

#[allow(unused_imports)]
use hermit_entry::elf::KernelObject;
use log::info;
use uefi::{
	fs::{FileSystem, Path},
	prelude::*,
	proto::console::gop::GraphicsOutput,
	table::{boot::*, cfg},
};

/// This function reads the provided kernelbinary
/// req. location: same subdir as the bootloader itself
/// reg. name: hermit_app
fn read_app(bt: &BootServices) -> Vec<u8> {
	let fs = bt
		.get_image_file_system(bt.image_handle())
		.expect("should open file system");

	let path = Path::new(cstr16!(r"\efi\boot\hermit_app"));

	let data = FileSystem::new(fs)
		.read(path)
		.expect("should read file content");

	let len = data.len();
	info!("Read Hermit application from \"{path}\" (size = {len} B)");

	data
}

/// Entry Point of the UEFI Loader
/// This function gets a so-called "EFI System Table" (see UEFI Specification, Section 4: EFI System Table) from the Firmware Interface.
/// Here, the RSDP (for BOOT_INFO) and the kernel are located and the kernel is parsed and loaded into memory.
/// After that, free physical memory for the kernel to use is looked for and saved for BOOT_INFO as well.
/// Finally, the architecture specific boot function is called.
#[entry]
unsafe fn loader_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
	// initialize Hermits Log functionality
	uefi_services::init(&mut system_table).unwrap();
	system_table.stdout().clear().unwrap();
	let bs = system_table.boot_services();

	// get the Graphics Output Protocol (GOP) to extract the raw pointer to the framebuffer
	let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
	let mut gop = bs
		.open_protocol_exclusive::<GraphicsOutput>(gop_handle)
		.unwrap();

	let mut framebuffer = get_framebuffer(&mut gop);
	let mut fbwriter = FramebufWriter::new(framebuffer);
	info!("Hello World from UEFI boot!");
	fbwriter.write("Hello World from UEFI boot!", None);

	info!("GOP Mode info: {:#?}", gop.current_mode_info());
	let mut framebuffer = get_framebuffer(&mut gop);
	let mut fbwriter = FramebufWriter::new(framebuffer);
	info!("Hello World from UEFI boot!");
	fbwriter.write("Hello World from UEFI boot!", None);

	// GOP needs to be dropped in order to exit boot services later and we only need the raw pointer to the framebuffer (which still exists after the GOP has been dropped)
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
	let kernel_bytes = read_app(bs);

	let kernel = KernelObject::parse(&kernel_bytes).unwrap();

	fbwriter.write("Kernel parsed!", None);
	info!("Kernel parsed!");
	let filesize = kernel.mem_size();
	info!("Kernelsize: {:#?}", filesize);
	fbwriter.write("Kernelsize", Some(filesize));

	// allocate space for the kernel
	let kernel_addr = bs
		.allocate_pages(
			AllocateType::AnyPages,
			MemoryType::LOADER_DATA,
			(filesize / 4096) + 1,
		)
		.unwrap();

	let kernel_addr = kernel.start_addr().unwrap_or(kernel_addr);
	fbwriter.write("Kernel located at", Some(kernel_addr.try_into().unwrap()));
	info!("Kernel located at {:#x}", kernel_addr);
	let memory =
		unsafe { slice::from_raw_parts_mut(kernel_addr as *mut mem::MaybeUninit<u8>, filesize) };

	let kernel_info = kernel.load_kernel(memory, memory.as_ptr() as u64);
	fbwriter.write("Kernel loaded into memory", None);
	info!("Kernel loaded into memory");

	// exit boot services for getting a runtime view of the system table and an iterator to the UEFI memory map
	let (runtime_system_table, mut memory_map) =
		system_table.exit_boot_services(MemoryType::LOADER_DATA);

	// find largest continous space of memory for the kernel to use
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
	let end_address = start_address + (max_size as usize * 0x1000_usize) - 1;
	fbwriter.write("Kernelmemory start address", Some(start_address));
	fbwriter.write("Kernelmemory end address", Some(end_address));

	info!("Kernelmemory: start: {start_address:#x?}, end: {end_address:#x?}");

	// Jump into actual booting routine
	unsafe {
		arch::boot_kernel(
			rsdp_addr,
			kernel_addr,
			filesize,
			kernel_info,
			runtime_system_table,
			start_address,
			end_address,
			fbwriter,
		)
	}
}
