use core::arch::asm;

pub struct SerialPort {}

impl SerialPort {
	pub const fn new() -> Self {
		Self {}
	}

	fn sbi_putchar(byte: u8) {
		unsafe {
			asm!(
				"li a7, 0x01",
				"ecall",
				in("a0") byte,
				lateout("a7") _
			);
		}
	}

	pub fn write_byte(&self, byte: u8) {
		// LF newline characters need to be extended to CRLF over a real serial port.
		if byte == b'\n' {
			SerialPort::sbi_putchar(b'\r');
		}

		SerialPort::sbi_putchar(byte);
	}

	pub fn init(&self) {}
}
