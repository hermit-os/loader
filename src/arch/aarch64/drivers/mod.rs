use core::num::NonZeroU32;

use enum_dispatch::enum_dispatch;
use qemu_serial::QemuSerial;
use xlnx_serial::XlnxSerial;

pub mod qemu_serial;
pub mod xlnx_serial;

pub enum SerialSuccess<T> {
	Success(T),
	ERetry,
}

#[enum_dispatch]
pub trait SerialDriver {
	fn init(&mut self);
	fn set_baud(&self, baud_rate: u32);
	fn putc(&mut self, c: u8) -> SerialSuccess<u8>;
	fn getc(&self) -> SerialSuccess<u8>;
	fn putstr(&mut self, s: &[u8]);
	fn get_addr(&self) -> u32;
	fn wait_empty(&mut self);
}

#[enum_dispatch(SerialDriver)]
pub enum SerialPort {
	Qemu(QemuSerial),
	Xlnx(XlnxSerial),
}

pub fn get_device(id: &[&str], uart_addr: u32) -> Option<SerialPort> {
	if id.iter().any(|compat| compat == "arm,pl011") {
		return Some(SerialPort::Qemu(QemuSerial::from_addr(
			NonZeroU32::new(uart_addr).unwrap(),
		)));
	} else if id.iter().any(|compat| compat == "xlnx,xuartlite") {
		return Some(SerialPort::Xlnx(XlnxSerial::from_addr(
			NonZeroU32::new(uart_addr).unwrap(),
		)));
	}
	None
}
