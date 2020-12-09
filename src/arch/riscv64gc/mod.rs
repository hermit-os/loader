use goblin::elf;
pub mod bootinfo;
pub mod paging;
pub mod physicalmem;
pub mod uart;

use crate::arch::bootinfo::*;
use crate::arch::uart::*;

use crate::arch::paging::GigaPageSize;
use crate::arch::riscv64gc::paging::PageSize;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("boot.S"));

pub static mut BOOT_INFO: BootInfo = BootInfo::new();
pub const ELF_ARCH: u16 = elf::header::EM_RISCV;
const UART_ADDRESS: *mut u8 = 0x1000_0000 as *mut u8;

const UART: Uart = Uart::new(UART_ADDRESS);

extern "C" {
	static kernel_end: u8;
}

pub fn message_output_init() {
	println!("in riscv");
}

pub unsafe fn get_memory(_memory_size: u64) -> u64 {
	align_up!(&kernel_end as *const u8 as u64, GigaPageSize::SIZE as u64)
}

pub fn output_message_byte(byte: u8) {
	UART.write_byte(byte);
}

pub fn find_kernel() -> &'static [u8] {
	// HERMIT_APP is the absolute path of the RustyHermit kernel
	include_bytes!(env!("HERMIT_APP"))
}

pub unsafe fn boot_kernel(virtual_address: u64, mem_size: u64, entry_point: u64) -> ! {
	loaderlog!(
		"Jumping to HermitCore Application Entry Point at 0x{:x}",
		entry_point
	);

	BOOT_INFO.base = virtual_address;
	BOOT_INFO.image_size = mem_size;

	let kernel_entry: extern "C" fn(boot_info: &'static mut BootInfo) -> ! =
		core::mem::transmute(entry_point);
	kernel_entry(&mut BOOT_INFO);
}

#[no_mangle]
pub fn cpu_ipi() -> ! {
	panic!();
}
