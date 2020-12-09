pub struct Uart {
	base_address: *mut u8,
}

impl Uart {
	pub const fn new(base_address: *mut u8) -> Self {
		Uart { base_address }
	}

	pub fn write_byte(&self, byte: u8) {
		unsafe {
			self.base_address.write_volatile(byte);
		}
	}
}
