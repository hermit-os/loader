cfg_if::cfg_if! {
	if #[cfg(all(target_os = "none", not(feature = "require-secure-boot")))] {
		mod none;
		pub use self::none::*;
	} else if #[cfg(target_os = "uefi")] {
		mod uefi;
		pub use self::uefi::*;
	}
}
