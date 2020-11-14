use goblin::elf;
pub mod bootinfo;
pub mod uart;
pub mod paging;

use crate::arch::bootinfo::*;
use crate::arch::uart::*;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("boot.S"));

pub static mut BOOT_INFO: BootInfo = BootInfo::new();
pub const ELF_ARCH: u16 = elf::header::EM_RISCV;
const UART_ADDRESS: *mut u8 = 0x1000_0000 as *mut u8;

const UART: Uart = Uart::new(UART_ADDRESS);

pub fn message_output_init() {
    println!("in riscv");
}

pub unsafe fn get_memory(_memory_size: u64) -> u64 {
    unimplemented!();
}

pub fn output_message_byte(byte: u8) {
    UART.write_byte(byte);
}

pub fn find_kernel() -> &'static [u8] {
    unimplemented!();
}

pub unsafe fn boot_kernel(virtual_address: u64, mem_size: u64, entry_point: u64) -> ! {
    unimplemented!();
}

#[no_mangle]
pub fn cpu_ipi() -> ! {
    panic!();
}
