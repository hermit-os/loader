//! HVM start info.
//!
//! For details, see [xen/include/public/arch-x86/hvm/start_info.h].
//!
//! [xen/include/public/arch-x86/hvm/start_info.h]: https://xenbits.xen.org/gitweb/?p=xen.git;a=blob;f=xen/include/public/arch-x86/hvm/start_info.h;h=e33557c0b4e98c6db3d3521710daa3838586733c;hb=06af9ef22996cecc2024a2e6523cec77a655581e

mod header;
#[cfg(feature = "reader")]
pub mod reader;
mod xen;

use core::ptr::NonNull;

pub use self::header::*;
pub use self::xen::*;

impl StartInfo {
	pub unsafe fn from_ptr<'a>(start_info: NonNull<u32>) -> Option<&'a Self> {
		let magic = unsafe { start_info.read() };

		if magic != START_MAGIC_VALUE {
			return None;
		}

		let start_info = unsafe { start_info.cast::<Self>().as_ref() };
		Some(start_info)
	}
}
