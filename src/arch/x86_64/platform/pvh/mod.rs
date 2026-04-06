use alloc::borrow::ToOwned;
use core::ptr;
use core::ptr::write_bytes;
use core::sync::atomic::{AtomicU32, Ordering};
use pvh::start_info::MemmapType;
use pvh::start_info::reader::{IdentityMap, StartInfoReader};

use align_address::Align;
use hermit_entry::boot_info::{
	BootInfo, DeviceTreeAddress, HardwareInfo, PlatformInfo, SerialPortBase,
};
use hermit_entry::elf::LoadedKernel;
use vm_fdt::FdtWriterResult;
use x86_64::structures::paging::{PageSize, Size4KiB};

use crate::BootInfoExt;
use crate::arch::x86_64::physicalmem::PhysAlloc;
use crate::arch::x86_64::{KERNEL_STACK_SIZE, SERIAL_IO_PORT, page_tables};
use crate::fdt::Fdt;
use crate::os::executable_end;

unsafe extern "C" {
	fn _start() -> !;
}

pvh::xen_elfnote_phys32_entry!(_start);

unsafe extern "C" {
	static mut _end: u8;
}

mod entry;

static START_INFO: AtomicU32 = AtomicU32::new(0);

fn start_info() -> StartInfoReader<'static, IdentityMap> {
	let paddr = START_INFO.load(Ordering::Relaxed);
	unsafe { StartInfoReader::from_paddr_identity(paddr).unwrap() }
}

unsafe extern "C" fn rust_start(info: u32) -> ! {
	println!("Hello");
	crate::log::init();
	START_INFO.store(info, Ordering::Relaxed);

	use crate::os::{executable_end, executable_start};
	let loader_start = executable_start();
	let loader_end = executable_end();
	println!("Loader: [{loader_start:p} - {loader_end:p}]");

	dbg!(info);

	let start_info = start_info();
	dbg!(&start_info);

	// panic!();

	// println!("{start_info:p}");
	// dbg!(start_info);
	// dbg!(start_info.cmdline(IdentityMap));

	// for module in start_info.modlist(IdentityMap) {
	// 	println!("{module:#x?}");
	// }

	// for memmap in start_info.memmap(IdentityMap).unwrap() {
	// 	println!("{memmap:#x?}");
	// }

	// let mut mem = Mem;
	// let multiboot = unsafe { Multiboot::from_ref(&mut *mb_info, &mut mem) };
	// let highest_address = multiboot.find_highest_address().align_up(Size2MiB::SIZE) as usize;
	// // Memory after the highest end address is unused and available for the physical memory manager.

	let highest_address = ptr::addr_of!(_end).addr();
	println!("highest_addr = {highest_address:#x}");
	PhysAlloc::init(highest_address.align_up(0x1000));

	// let max_phys_addr = multiboot
	// 	.memory_regions()
	// 	.unwrap()
	// 	.filter(|memory_region| memory_region.memory_type() == MemoryType::Available)
	// 	.map(|memory_region| memory_region.base_address() + memory_region.length())
	// 	.max()
	// 	.unwrap();

	let max_phys_addr = start_info
		.memmap()
		.iter()
		.filter(|memmap| memmap.ty() == MemmapType::Ram)
		.map(|memmap| memmap.addr + memmap.size)
		.max()
		.unwrap();

	unsafe {
		page_tables::init(max_phys_addr.try_into().unwrap());
	}

	unsafe {
		crate::os::loader_main();
	}
}

pub struct DeviceTree;

impl DeviceTree {
	pub fn create() -> FdtWriterResult<&'static [u8]> {
		let start_info = start_info();
		let mut fdt = Fdt::new("multiboot")?.mmap(start_info.memmap())?;

		if let Some(cmdline) = start_info.cmdline() {
			let cmdline = cmdline.to_str().unwrap().to_owned();
			fdt = fdt.bootargs(cmdline)?;
		}

		let fdt = fdt.finish()?;

		Ok(fdt.leak())
	}
}

pub fn find_kernel() -> &'static [u8] {
	let start_info = start_info();

	let mut modlist = start_info.modlist();
	let foo = modlist.next().unwrap();
	unsafe  { core::mem::transmute( foo.as_slice()) }
}

pub unsafe fn boot_kernel(kernel_info: LoadedKernel) -> ! {
	let LoadedKernel {
		load_info,
		entry_point,
	} = kernel_info;

	// let mut mem = Mem;
	// let mb_info = MB_INFO.load(Ordering::Relaxed);
	// let multiboot = unsafe { Multiboot::from_ptr(mb_info as u64, &mut mem).unwrap() };

	// determine boot stack address
	let loader_end = executable_end().as_ptr();
	let new_stack = loader_end.addr().align_up(Size4KiB::SIZE as usize);

	// if new_stack + KERNEL_STACK_SIZE as usize > mb_info.addr() {
	// 	new_stack = (mb_info.addr() + mem::size_of::<Multiboot<'_, '_>>())
	// 		.align_up(Size4KiB::SIZE as usize);
	// }

	// let command_line = multiboot.command_line();
	// if let Some(command_line) = command_line {
	// 	let cmdline = command_line.as_ptr() as usize;
	// 	let cmdsize = command_line.len();
	// 	if new_stack + KERNEL_STACK_SIZE as usize > cmdline {
	// 		new_stack = (cmdline + cmdsize).align_up(Size4KiB::SIZE as usize);
	// 	}
	// }

	let stack = loader_end.with_addr(new_stack).cast::<u8>();

	// clear stack
	unsafe {
		write_bytes(stack, 0, KERNEL_STACK_SIZE.try_into().unwrap());
	}

	let device_tree = DeviceTree::create().expect("Unable to create devicetree!");
	let device_tree =
		DeviceTreeAddress::new(u64::try_from(device_tree.as_ptr().expose_provenance()).unwrap());

	let boot_info = BootInfo {
		hardware_info: HardwareInfo {
			phys_addr_range: 0..0,
			serial_port_base: SerialPortBase::new(SERIAL_IO_PORT),
			device_tree,
		},
		load_info,
		platform_info: PlatformInfo::Multiboot {
			command_line: None,
			multiboot_info_addr: (1).try_into().unwrap(),
		},
	};

	let entry = ptr::with_exposed_provenance(entry_point.try_into().unwrap());
	let raw_boot_info = boot_info.write();

	unsafe { crate::arch::x86_64::enter_kernel(stack, entry, raw_boot_info) }
}
