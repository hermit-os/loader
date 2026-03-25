mod debug;

use core::ffi::{CStr, c_char};
use core::{ptr, slice};

use super::*;

impl StartInfo {
	pub unsafe fn reader<M>(&self, mem_map: M) -> StartInfoReader<'_, M> {
		StartInfoReader {
			start_info: self,
			mem_map,
		}
	}

	pub unsafe fn identity_reader(&self) -> StartInfoReader<'_, IdentityMap> {
		unsafe { self.reader(IdentityMap) }
	}
}

pub struct StartInfoReader<'a, M> {
	pub(super) start_info: &'a StartInfo,
	pub(super) mem_map: M,
}

impl<'a, M: MemMap> StartInfoReader<'a, M> {
	pub fn modlist_vaddr(&self) -> Option<NonNull<ModlistEntry>> {
		if self.start_info.modlist_paddr == 0 {
			return None;
		}

		let ptr = self
			.mem_map
			.ptr(self.start_info.modlist_paddr)
			.cast::<ModlistEntry>();

		NonNull::new(ptr)
	}

	pub fn modlist(&self) -> Option<&'a [ModlistEntry]> {
		let ptr = self.modlist_vaddr()?.as_ptr();
		let len = usize::try_from(self.start_info.nr_modules).unwrap();
		let slice = unsafe { slice::from_raw_parts(ptr, len) };
		Some(slice)
	}

	pub fn modlist_readers(&self) -> Option<impl Iterator<Item = ModlistEntryReader<'a, &M>>> {
		let mem_map = &self.mem_map;
		let iter = self
			.modlist()?
			.iter()
			.map(move |module| unsafe { module.reader(mem_map) });
		Some(iter)
	}

	pub fn cmdline_vaddr(&self) -> Option<NonNull<c_char>> {
		if self.start_info.cmdline_paddr == 0 {
			return None;
		}

		let ptr = self
			.mem_map
			.ptr(self.start_info.cmdline_paddr)
			.cast::<c_char>();

		NonNull::new(ptr)
	}

	pub fn cmdline(&self) -> Option<&'a CStr> {
		let ptr = self.cmdline_vaddr()?.as_ptr();

		let cmdline = unsafe { CStr::from_ptr(ptr) };
		Some(cmdline)
	}

	pub fn memmap(&self) -> Option<&'a [MemmapTableEntry]> {
		if self.start_info.version < 1 {
			return None;
		}

		let ptr = self
			.mem_map
			.ptr(self.start_info.memmap_paddr)
			.cast::<MemmapTableEntry>();
		let len = usize::try_from(self.start_info.memmap_entries).unwrap();
		let slice = unsafe { slice::from_raw_parts(ptr, len) };
		Some(slice)
	}
}

impl ModlistEntry {
	pub unsafe fn reader<M>(&self, mem_map: M) -> ModlistEntryReader<'_, M> {
		ModlistEntryReader {
			modlist_entry: self,
			mem_map,
		}
	}

	pub unsafe fn identity_reader(&self) -> ModlistEntryReader<'_, IdentityMap> {
		unsafe { self.reader(IdentityMap) }
	}
}

pub struct ModlistEntryReader<'a, M> {
	pub(super) modlist_entry: &'a ModlistEntry,
	pub(super) mem_map: M,
}

impl<'a, M: MemMap> ModlistEntryReader<'a, M> {
	pub fn vaddr(&self) -> *const u8 {
		self.mem_map.ptr(self.modlist_entry.paddr).cast::<u8>()
	}

	pub fn cmdline_vaddr(&self) -> *const c_char {
		self.mem_map
			.ptr(self.modlist_entry.cmdline_paddr)
			.cast::<c_char>()
	}

	pub fn as_slice(&self) -> &'a [u8] {
		let ptr = self.vaddr();
		let len = usize::try_from(self.modlist_entry.size).unwrap();
		unsafe { slice::from_raw_parts(ptr, len) }
	}

	pub fn cmdline(&self) -> Option<&CStr> {
		let ptr = self.cmdline_vaddr();

		if ptr.is_null() {
			return None;
		}

		let cmdline = unsafe { CStr::from_ptr(ptr) };
		Some(cmdline)
	}
}

pub trait MemMap {
	fn ptr(&self, paddr: u64) -> *mut ();
}

impl<M: MemMap + ?Sized> MemMap for &M {
	#[inline]
	fn ptr(&self, paddr: u64) -> *mut () {
		(**self).ptr(paddr)
	}
}

impl<M: MemMap + ?Sized> MemMap for &mut M {
	#[inline]
	fn ptr(&self, paddr: u64) -> *mut () {
		(**self).ptr(paddr)
	}
}

pub struct IdentityMap;

impl MemMap for IdentityMap {
	fn ptr(&self, paddr: u64) -> *mut () {
		let addr = usize::try_from(paddr).unwrap();
		ptr::with_exposed_provenance_mut(addr)
	}
}
