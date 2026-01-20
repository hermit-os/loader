cfg_if::cfg_if! {
	if #[cfg(target_os = "none")] {
		mod none;
		pub use self::none::*;
	} else if #[cfg(target_os = "uefi")] {
		mod uefi;
		pub use self::uefi::*;
	}
}

#[cfg_attr(not(any(target_os = "none", feature = "fc")), allow(dead_code))]
#[derive(Clone, Default)]
pub struct ExtraBootInfo {
	pub(crate) image: Option<&'static [u8]>,
}
