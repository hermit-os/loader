use log::{error, info};
use uefi::boot::{SearchType, open_protocol_exclusive};
use uefi::runtime::VariableVendor;
use uefi::{CStr16, Identify, Status, boot};

use crate::os::uefi::secure_boot::security_lib::Security2ArchProtocol;

pub fn is_secure_boot_enabled() -> bool {
	let mut string_backing = [0u16; 22];
	let mut output_buffer = [0u8; 1];

	let result = uefi::runtime::get_variable(
		CStr16::from_str_with_buf("SecureBoot", &mut string_backing)
			.expect("failed to convert string"),
		&VariableVendor::GLOBAL_VARIABLE,
		&mut output_buffer,
	);

	match result {
		Ok(_) => output_buffer[0] == 1,
		Err(err) => {
			if err.status() == Status::NOT_FOUND {
				false
			} else {
				panic!("Could not load SecureBoot variable: {:?}", err.status())
			}
		}
	}
}

pub fn verify_image(image_contents: &[u8]) -> uefi::Result<Status> {
	let protocol_handle =
		boot::locate_handle_buffer(SearchType::ByProtocol(&Security2ArchProtocol::GUID))?;

	let protocol_handle = protocol_handle
		.first()
		.expect("Security2Arch protocol is missing");

	let protocol = open_protocol_exclusive::<Security2ArchProtocol>(*protocol_handle)?;

	Ok(protocol.authenticate_file(image_contents))
}

pub fn verify_image_or_panic(image_contents: &[u8]) {
	if !is_secure_boot_enabled() {
		if cfg!(feature = "require-secure-boot") {
			error!(
				"The loader was compiled with the `require-secure-boot` flag and requires secure boot to be enabled."
			);
			panic!("This loader requires secure boot to be enabled");
		} else {
			info!("Secure boot is not enabled on this machine");
		}
		return;
	}

	let status = verify_image(image_contents).expect("error encountered while verifying image!");

	if status != Status::SUCCESS {
		error!(
			"Secure boot image verification failed with status {:?}",
			status
		);
		panic!("Secure boot image verification failed!");
	} else {
		info!("Secure boot image verification passed!");
	}
}

mod security_lib {
	use core::ffi::c_void;

	use uefi::Status;
	use uefi::proto::unsafe_protocol;

	use crate::os::uefi::secure_boot::security_lib_raw;

	#[repr(transparent)]
	#[derive(Debug)]
	#[unsafe_protocol(security_lib_raw::Security2ArchProtocolRaw::GUID)]
	pub struct Security2ArchProtocol(security_lib_raw::Security2ArchProtocolRaw);

	impl Security2ArchProtocol {
		pub fn authenticate_file(&self, file_buffer: &[u8]) -> Status {
			unsafe {
				(self.0.authenticate_file)(
					&self.0,
					core::ptr::null(),
					file_buffer.as_ptr() as *const c_void,
					file_buffer.len(),
					false,
				)
			}
		}
	}
}

mod security_lib_raw {
	use core::ffi::c_void;

	use uefi::{Guid, Status, guid};
	use uefi_raw::protocol::device_path::DevicePathProtocol;

	#[derive(Debug)]
	#[repr(C)]
	pub struct Security2ArchProtocolRaw {
		pub authenticate_file: unsafe extern "efiapi" fn(
			this: *const Self,
			device_path: *const DevicePathProtocol,
			file_buffer: *const c_void,
			file_size: usize,
			boot_policy: bool,
		) -> Status,
	}

	impl Security2ArchProtocolRaw {
		pub const GUID: Guid = guid!("94AB2F58-1438-4EF1-9152-18941A3A0E68");
	}
}
