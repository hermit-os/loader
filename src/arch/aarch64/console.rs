use core::num::NonZeroU32;

use hermit_dtb::Dtb;

use crate::arch::aarch64::drivers::SerialDriver;
use crate::arch::drivers::qemu_serial::QemuSerial;
use crate::arch::drivers::xlnx_serial::XlnxSerial;
use crate::arch::drivers::{get_device, SerialPort};

pub struct Console {
	stdout: SerialPort,
}

fn parse_uart_node(path: &str, dtb: Dtb<'_>) -> ([&str], u64) {
	let unaliased_path = dtb
		.get_property("/aliases", path)
		.and_then(|found_alias| core::str::from_utf8(found_alias).ok())
		.unwrap_or(path);
	let compatible = dtb
		.get_property(unaliased_path, "compatible")
		.map(|slice| {
			slice
				.split(|val| val == 0)
				.map(|part| core::str::from_utf8(part).ok())
				.collect()
		})
		.unwrap();
	let address = dtb
		.get_property(unaliased_path, "reg")
		.map(|slice| u64::from_be_bytes(slice[0..8].try_into().unwrap()))
		.unwrap();
	(compatible, address)
}

fn read_from_chosen(property: &[u8], dtb: Dtb<'_>) -> ([&str], u64) {
	core::str::from_utf8(property)
		.ok()
		.map(|stdout| stdout.trim_matches(char::from(0)))
		.and_then(|stdout| stdout.split_once(':'))
		.map(|path| parse_uart_node(path.0, dtb))
		.unwrap()
}
pub fn stdout() -> SerialPort {
	/// Physical address of UART0 at Qemu's virt emulation
	const SERIAL_PORT_ADDRESS: u32 = 0x09000000;

	let dtb = unsafe {
		Dtb::from_raw(sptr::from_exposed_addr(super::DEVICE_TREE as usize))
			.expect(".dtb file has invalid header")
	};

	let property = dtb.get_property("/chosen", "stdout-path");
	let (spec, addr) = property
		.map(|prop| read_from_chosen(prop, dtb))
		.unwrap_or((["arm,pl011"], SERIAL_PORT_ADDRESS));
	let mut serial = get_device(spec, addr).unwrap_or(SerialPort::Qemu(QemuSerial::from_addr(
		NonZeroU32::new(SERIAL_PORT_ADDRESS).unwrap(),
	)));
	//let mut serial = QemuSerial::from_addr(NonZeroU32::new(uart_address).unwrap());
	serial.init();
	serial
}

impl Console {
	pub fn write_bytes(&mut self, bytes: &[u8]) {
		self.stdout.putstr(bytes);
	}

	pub(super) fn get_stdout(&self) -> u32 {
		self.stdout.get_addr()
	}

	pub(crate) fn set_stdout(&mut self, stdout: u32) {
		match self.stdout {
			SerialPort::Qemu(_) => {
				self.stdout =
					SerialPort::Qemu(QemuSerial::from_addr(NonZeroU32::new(stdout).unwrap()))
			}
			SerialPort::Xlnx(_) => {
				self.stdout =
					SerialPort::Xlnx(XlnxSerial::from_addr(NonZeroU32::new(stdout).unwrap()))
			}
		}
		self.stdout.init();
	}

	pub(crate) fn wait_empty(&mut self) {
		self.stdout.wait_empty();
	}
}

impl Default for Console {
	fn default() -> Self {
		let stdout = stdout();
		Self { stdout }
	}
}

unsafe impl Send for Console {}
