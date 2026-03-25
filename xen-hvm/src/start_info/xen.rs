//! Items from xen.h.
//! 
//! For details, see [xen/include/public/xen.h#L886].
//! 
//! [xen/include/public/xen.h#L886]: https://xenbits.xen.org/gitweb/?p=xen.git;a=blob;f=xen/include/public/xen.h;h=82b9c05a76b7faedded8778fb8274a0d3d5d31e4;hb=06af9ef22996cecc2024a2e6523cec77a655581e#l886

use bitflags::bitflags;

bitflags! {
	/// These flags are passed in the 'flags' field of start_info_t.
    #[doc(alias = "SIF")]
	#[repr(transparent)]
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct Sif: u32 {
		/// Is the domain privileged?
		#[doc(alias = "SIF_PRIVILEGED")]
		const PRIVILEGED = 1 << 0;

		/// Is this the initial control domain?
		#[doc(alias = "SIF_INITDOMAIN")]
		const INITDOMAIN = 1 << 1;

		/// Is mod_start a multiboot module?
		#[doc(alias = "SIF_MULTIBOOT_MOD")]
		const MULTIBOOT_MOD = 1 << 2;

		/// Is mod_start a PFN?
		#[doc(alias = "SIF_MOD_START_PFN")]
		const MOD_START_PFN = 1 << 3;

		/// Do Xen tools understand a virt. mapped
		/// P->M making the 3 level tree obsolete?
		#[doc(alias = "SIF_VIRT_P2M_4TOOLS")]
		const VIRT_P2M_4TOOLS = 1 << 4;

		/// reserve 1 byte for xen-pm options
		#[doc(alias = "SIF_PM_MASK")]
		const PM_MASK = 0xFF << 8;
	}
}
