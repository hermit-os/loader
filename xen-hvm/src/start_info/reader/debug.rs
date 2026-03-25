use core::{fmt, ptr};

use super::*;

impl<'a, M: MemMap> fmt::Debug for StartInfoReader<'a, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("StartInfo")
			.field("version", &self.start_info.version)
			.field("flags", &self.start_info.flags)
			.field("modlist", &self.modlist_debug())
			.field(
				"cmdline_paddr",
				&DebugAsPointer(self.start_info.cmdline_paddr),
			)
			.field("cmdline_vaddr", &self.cmdline_vaddr())
			.field("cmdline", &self.cmdline())
			.field("rsdp_paddr", &DebugAsPointer(self.start_info.rsdp_paddr))
			.field("memmap", &self.memmap().map(MemmapTableDebug))
			.finish()
	}
}

impl<'a, M: MemMap> StartInfoReader<'a, M> {
	pub fn modlist_debug(&self) -> Option<ModlistDebug<'_, &M>> {
		let modlist = self.modlist()?;
		let mem_map = &self.mem_map;
		let modlist_debug = ModlistDebug { modlist, mem_map };
		Some(modlist_debug)
	}
}

impl<'a, M: MemMap> fmt::Debug for ModlistEntryReader<'a, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ModlistEntry")
			.field("paddr", &DebugAsPointer(self.modlist_entry.paddr))
			.field("vaddr", &self.vaddr())
			.field("size", &DebugAsPointer(self.modlist_entry.size))
			.field(
				"cmdline_paddr",
				&DebugAsPointer(self.modlist_entry.cmdline_paddr),
			)
			.field("cmdline_vaddr", &self.cmdline_vaddr())
			.field("cmdline", &self.cmdline())
			.finish()
	}
}

struct MemmapTableEntryDebug<'a>(&'a MemmapTableEntry);

impl fmt::Debug for MemmapTableEntryDebug<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("MemmapTableEntry")
			.field("addr", &DebugAsPointer(self.0.addr))
			.field("size", &DebugAsPointer(self.0.size))
			.field("type", &self.0.ty)
			.finish()
	}
}

pub struct MemmapTableDebug<'a>(&'a [MemmapTableEntry]);

impl<'a> fmt::Debug for MemmapTableDebug<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let entries = self.0.iter().map(MemmapTableEntryDebug);
		f.debug_list().entries(entries).finish()
	}
}

pub struct ModlistDebug<'a, M> {
	pub(super) modlist: &'a [ModlistEntry],
	pub(super) mem_map: M,
}

impl<'a, M: MemMap> fmt::Debug for ModlistDebug<'a, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mem_map = &self.mem_map;
		let entries = self
			.modlist
			.iter()
			.map(move |modlist_entry| unsafe { modlist_entry.reader(mem_map) });
		f.debug_list().entries(entries).finish()
	}
}

struct DebugAsPointer<T>(T);

impl<T> fmt::Debug for DebugAsPointer<T>
where
	T: TryInto<usize> + Copy,
	T::Error: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let addr = self.0.try_into().unwrap();
		let ptr = ptr::without_provenance::<()>(addr);
		fmt::Pointer::fmt(&ptr, f)
	}
}
