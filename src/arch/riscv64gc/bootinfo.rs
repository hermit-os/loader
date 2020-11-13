

#[repr(C)]
// TODO determine what goes in this struct
pub struct BootInfo {
    pub tls_start: u64,
	pub tls_filesz: u64,
	pub tls_memsz: u64,
}

impl BootInfo {
    pub const fn new() -> Self {
        BootInfo {
            tls_start: 0,
            tls_filesz: 0,
            tls_memsz: 0,
        }
    }
}
