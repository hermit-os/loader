mod allocator;
mod console;
mod fdt;

use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::slice;

use align_address::Align;
use hermit_entry::boot_info::{
	BootInfo, DeviceTreeAddress, HardwareInfo, PlatformInfo, SerialPortBase,
};
use hermit_entry::elf::{KernelObject, LoadedKernel};
use log::info;
use sptr::Strict;
use uefi::fs::{FileSystem, Path};
use uefi::prelude::*;
use uefi::table::boot::{AllocateType, BootServices, MemoryType, PAGE_SIZE};
use uefi::table::cfg;

pub use self::console::CONSOLE;
use self::fdt::Fdt;
use crate::{arch, BootInfoExt};

// Entry Point of the Uefi Loader
#[entry]
fn loader_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
	uefi::helpers::init(&mut system_table).unwrap();
	unsafe {
		uefi::allocator::init(&mut system_table);
	}
	crate::log::init();
	let bs = system_table.boot_services();

	let kernel_image = read_app(bs);
	let kernel = KernelObject::parse(&kernel_image).unwrap();

	let kernel_memory = bs.alloc_page_slice(kernel.mem_size()).unwrap();
	let kernel_memory = &mut kernel_memory[..kernel.mem_size()];

	let kernel_info = kernel.load_kernel(kernel_memory, kernel_memory.as_ptr() as u64);

	let rsdp = system_table.rsdp();

	drop(kernel_image);

	let fdt = Fdt::new()
		.unwrap()
		.rsdp(u64::try_from(rsdp.expose_addr()).unwrap())
		.unwrap();

	allocator::exit_boot_services();
	let (_runtime_system_table, mut memory_map) =
		system_table.exit_boot_services(MemoryType::LOADER_DATA);

	let fdt = fdt.memory_map(&mut memory_map).unwrap().finish().unwrap();

	unsafe { boot_kernel(kernel_info, fdt) }
}

fn read_app(bt: &BootServices) -> Vec<u8> {
	let fs = bt
		.get_image_file_system(bt.image_handle())
		.expect("should open file system");

	let path = Path::new(cstr16!(r"\efi\boot\hermit-app"));

	let data = FileSystem::new(fs)
		.read(path)
		.expect("should read file content");

	let len = data.len();
	info!("Read Hermit application from \"{path}\" (size = {len} B)");

	data
}

pub unsafe fn boot_kernel(kernel_info: LoadedKernel, fdt: Vec<u8>) -> ! {
	let LoadedKernel {
		load_info,
		entry_point,
	} = kernel_info;

	let device_tree = DeviceTreeAddress::new(u64::try_from(fdt.leak().as_ptr().addr()).unwrap());

	let boot_info = BootInfo {
		hardware_info: HardwareInfo {
			phys_addr_range: 0..0,
			serial_port_base: SerialPortBase::new(arch::SERIAL_IO_PORT),
			device_tree,
		},
		load_info,
		platform_info: PlatformInfo::Fdt,
	};

	let stack = usize::try_from(boot_info.load_info.kernel_image_addr_range.end)
		.unwrap()
		.align_down(PAGE_SIZE);
	let entry = sptr::from_exposed_addr(entry_point.try_into().unwrap());
	let stack = sptr::from_exposed_addr_mut(stack);
	let raw_boot_info = boot_info.write();

	unsafe { arch::enter_kernel(stack, entry, raw_boot_info) }
}

trait BootServicesExt {
	fn alloc_page_slice(&self, size: usize) -> uefi::Result<&'static mut [MaybeUninit<u8>]>;
}

impl BootServicesExt for BootServices {
	fn alloc_page_slice(&self, size: usize) -> uefi::Result<&'static mut [MaybeUninit<u8>]> {
		let size = size.align_up(PAGE_SIZE);
		let phys_addr = self.allocate_pages(
			AllocateType::AnyPages,
			MemoryType::LOADER_DATA,
			size / PAGE_SIZE,
		)?;
		let ptr = sptr::from_exposed_addr_mut(usize::try_from(phys_addr).unwrap());
		Ok(unsafe { slice::from_raw_parts_mut(ptr, size) })
	}
}

trait SystemTableBootExt {
	/// Returns the RSDP.
	///
	/// This must be called before exiting boot services.
	/// See [5.2.5.2. Finding the RSDP on UEFI Enabled Systems — ACPI Specification 6.5 documentation](https://uefi.org/specs/ACPI/6.5/05_ACPI_Software_Programming_Model.html#finding-the-rsdp-on-uefi-enabled-systems) for details.
	fn rsdp(&self) -> *const c_void;
}

impl SystemTableBootExt for SystemTable<Boot> {
	fn rsdp(&self) -> *const c_void {
		let config_table = self.config_table();
		let (rsdp, version) = if let Some(entry) = config_table
			.iter()
			.find(|entry| entry.guid == cfg::ACPI2_GUID)
		{
			(entry.address, 2)
		} else {
			let entry = config_table
				.iter()
				.find(|entry| entry.guid == cfg::ACPI_GUID)
				.unwrap();
			(entry.address, 1)
		};
		info!("Found ACPI {version} RSDP at {rsdp:p}");
		rsdp
	}
}
