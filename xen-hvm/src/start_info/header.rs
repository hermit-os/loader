//! HVM start info.
//!
//! For details, see [xen/include/public/arch-x86/hvm/start_info.h].
//!
//! [xen/include/public/arch-x86/hvm/start_info.h]: https://xenbits.xen.org/gitweb/?p=xen.git;a=blob;f=xen/include/public/arch-x86/hvm/start_info.h;h=e33557c0b4e98c6db3d3521710daa3838586733c;hb=06af9ef22996cecc2024a2e6523cec77a655581e

use crate::start_info::Sif;

/// Start of day structure passed to PVH guests and to HVM guests in %ebx.
///
/// NOTE: nothing will be loaded at physical address 0, so a 0 value in any
/// of the address fields should be treated as not present.
///
///  0 +----------------+
///    | magic          | Contains the magic value XEN_HVM_START_MAGIC_VALUE
///    |                | ("xEn3" with the 0x80 bit of the "E" set).
///  4 +----------------+
///    | version        | Version of this structure. Current version is 1. New
///    |                | versions are guaranteed to be backwards-compatible.
///  8 +----------------+
///    | flags          | SIF_xxx flags.
/// 12 +----------------+
///    | nr_modules     | Number of modules passed to the kernel.
/// 16 +----------------+
///    | modlist_paddr  | Physical address of an array of modules
///    |                | (layout of the structure below).
/// 24 +----------------+
///    | cmdline_paddr  | Physical address of the command line,
///    |                | a zero-terminated ASCII string.
/// 32 +----------------+
///    | rsdp_paddr     | Physical address of the RSDP ACPI data structure.
/// 40 +----------------+
///    | memmap_paddr   | Physical address of the (optional) memory map. Only
///    |                | present in version 1 and newer of the structure.
/// 48 +----------------+
///    | memmap_entries | Number of entries in the memory map table. Zero
///    |                | if there is no memory map being provided. Only
///    |                | present in version 1 and newer of the structure.
/// 52 +----------------+
///    | reserved       | Version 1 and newer only.
/// 56 +----------------+
///
/// The layout of each entry in the module structure is the following:
///
///  0 +----------------+
///    | paddr          | Physical address of the module.
///  8 +----------------+
///    | size           | Size of the module in bytes.
/// 16 +----------------+
///    | cmdline_paddr  | Physical address of the command line,
///    |                | a zero-terminated ASCII string.
/// 24 +----------------+
///    | reserved       |
/// 32 +----------------+
///
/// The layout of each entry in the memory map table is as follows:
///
///  0 +----------------+
///    | addr           | Base address
///  8 +----------------+
///    | size           | Size of mapping in bytes
/// 16 +----------------+
///    | type           | Type of mapping as defined between the hypervisor
///    |                | and guest. See XEN_HVM_MEMMAP_TYPE_* values below.
/// 20 +----------------|
///    | reserved       |
/// 24 +----------------+
///
/// The address and sizes are always a 64bit little endian unsigned integer.
///
/// NB: Xen on x86 will always try to place all the data below the 4GiB
/// boundary.
///
/// Version numbers of the hvm_start_info structure have evolved like this:
///
/// Version 0:  Initial implementation.
///
/// Version 1:  Added the memmap_paddr/memmap_entries fields (plus 4 bytes of
///             padding) to the end of the hvm_start_info struct. These new
///             fields can be used to pass a memory map to the guest. The
///             memory map is optional and so guests that understand version 1
///             of the structure must check that memmap_entries is non-zero
///             before trying to read the memory map.
#[doc(alias = "XEN_HVM_START_MAGIC_VALUE")]
pub const START_MAGIC_VALUE: u32 = 0x336ec578;

/// The values used in the type field of the memory map table entries are
/// defined below and match the Address Range Types as defined in the "System
/// Address Map Interfaces" section of the ACPI Specification. Please refer to
/// section 15 in version 6.2 of the ACPI spec: <http://uefi.org/specifications>
#[doc(alias = "XEN_HVM_MEMMAP_TYPE")]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum MemmapType {
	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_RAM")]
	Ram = 1,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_RESERVED")]
	Reserved = 2,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_ACPI")]
	Acpi = 3,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_NVS")]
	Nvs = 4,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_UNUSABLE")]
	Unusable = 5,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_DISABLED")]
	Disabled = 6,

	#[doc(alias = "XEN_HVM_MEMMAP_TYPE_PMEM")]
	Pmem = 7,
}

/// C representation of the x86/HVM start info layout.
///
/// The canonical definition of this layout is above, this is just a way to
/// represent the layout described there using C types.
#[doc(alias = "hvm_start_info")]
#[derive(Debug)]
#[repr(C)]
pub struct StartInfo {
	/// Contains the magic value 0x336ec578 "xEn3" with the 0x80 bit of the "E"
	/// set).
	///
	/// See [`START_MAGIC_VALUE`] for a definition of the magic value.
	pub magic: u32,

	/// Version of this structure.
	pub version: u32,

	/// SIF_xxx flags.
	pub flags: Sif,

	/// Number of modules passed to the kernel.
	pub nr_modules: u32,

	/// Physical address of an array of hvm_modlist_entry.
	pub modlist_paddr: u64,

	/// Physical address of the command line.
	pub cmdline_paddr: u64,

	/// Physical address of the RSDP ACPI data structure.
	pub rsdp_paddr: u64,

	// All following fields only present in version 1 and newer
	/// Physical address of an array of hvm_memmap_table_entry.
	pub memmap_paddr: u64,

	/// Number of entries in the memmap table.
	///
	/// Value will be zero if there is no memory map being provided.
	pub memmap_entries: u32,

	/// Must be zero.
	pub reserved: u32,
}

#[doc(alias = "hvm_modlist_entry")]
#[derive(Debug)]
#[repr(C)]
pub struct ModlistEntry {
	/// Physical address of the module.
	pub paddr: u64,

	/// Size of the module in bytes.
	pub size: u64,

	/// Physical address of the command line.
	pub cmdline_paddr: u64,

	pub reserved: u64,
}

#[doc(alias = "hvm_memmap_table_entry")]
#[derive(Debug)]
#[repr(C)]
pub struct MemmapTableEntry {
	/// Base address of the memory region
	pub addr: u64,

	/// Size of the memory region in bytes
	pub size: u64,

	/// Mapping type
	#[doc(alias = "type")]
	pub ty: MemmapType,

	/// Must be zero for Version 1.
	pub reserved: u32,
}
