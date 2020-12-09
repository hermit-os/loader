#[repr(C)]
// TODO determine what goes in this struct
pub struct BootInfo {
	pub base: u64,
	pub image_size: u64,
	pub tls_start: u64,
	pub tls_filesz: u64,
	pub tls_memsz: u64,
}

impl BootInfo {
	pub const fn new() -> Self {
		BootInfo {
			base: 0,
			image_size: 0,
			tls_start: 0,
			tls_filesz: 0,
			tls_memsz: 0,
		}
	}
}
