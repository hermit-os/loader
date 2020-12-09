use core::marker::PhantomData;

use crate::arch::physicalmem;

/// Number of Offset bits of a virtual address for a 4 KiB page, which are shifted away to get its Page Frame Number (PFN).
const PAGE_BITS: usize = 12;

/// Number of bits of the index in each table (PML4, PDPT, PDT, PGT).
const PAGE_MAP_BITS: usize = 9;

/// A mask where PAGE_MAP_BITS are set to calculate a table index.
const PAGE_MAP_MASK: usize = 0x1FF;

bitflags! {
	/// Possible flags for an entry in a RISC-V paging structure
	///
	/// See RISC-V Privileged Spec, Section 4.3
	pub struct PageTableEntryFlags: usize {
		/// Set if the page table is valid and points to a page or table
		const VALID = 1 << 0;

		/// If READ | WRITE | EXECUTE = 0, this PTE points to another table
		/// Otherwise, the PTE is a leaf.

		/// Set if the current PTE is a leaf and the frame is readable
		const READ = 1 << 1;

		/// Set if the current PTE is a leaf and the frame is writeable
		const WRITE = 1 << 2;

		/// Set if the current PTE is a leaf and the frame is executable
		const EXECUTE = 1 << 3;

		/// Set if this is a U-mode mapping
		const USER = 1 << 4;

		/// Set if this mapping is present in all address spaces
		const GLOBAL = 1 << 5;

		/// Set if this page has been accessed since the last time the A bit was cleared
		const ACCESSED = 1 << 6;

		/// Set if this page has been written since the last time the D bit was cleared
		const DIRTY = 1 << 7;


	}
}

impl PageTableEntryFlags {
	/// An empty set of flags for unused/zeroed table entries.
	/// Needed as long as empty() is no const function.
	const BLANK: PageTableEntryFlags = PageTableEntryFlags { bits: 0 };
}

// An entry in a page table (any level)
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
	/// Physical memory address this entry refers, combined with flags from PageTableEntryFlags.
	physical_address_and_flags: usize,
}

impl PageTableEntry {
	/// Returns whether this entry is valid (present).
	fn is_present(&self) -> bool {
		(self.physical_address_and_flags & PageTableEntryFlags::VALID.bits()) != 0
	}

	/// Mark this as a valid (present) entry and set address translation and flags.
	///
	/// # Arguments
	///
	/// * `physical_address` - The physical memory address this entry shall translate to
	/// * `flags` - Flags from PageTableEntryFlags (note that the VALID flag is set automatically)
	fn set(&mut self, physical_address: usize, flags: PageTableEntryFlags) {
		self.physical_address_and_flags =
			physical_address | (PageTableEntryFlags::VALID | flags).bits();
	}
}

/// A generic interface to support all possible page sizes.
///
/// This is defined as a subtrait of Copy to enable #[derive(Clone, Copy)] for Page.
/// Currently, deriving implementations for these traits only works if all dependent types implement it as well.
pub trait PageSize: Copy {
	/// The page size in bytes.
	const SIZE: usize;

	/// The page table level at which a page of this size is mapped (from 0 for PGT through 3 for PML4).
	/// Implemented as a numeric value to enable numeric comparisons.
	const MAP_LEVEL: usize;

	/// Any extra flag that needs to be set to map a page of this size.
	/// For example: PageTableEntryFlags::HUGE_PAGE
	const MAP_EXTRA_FLAG: PageTableEntryFlags;
}

/// A 4 KiB page mapped in the level 0 PT.
#[derive(Clone, Copy)]
pub enum BasePageSize {}
impl PageSize for BasePageSize {
	const SIZE: usize = 4096;
	const MAP_LEVEL: usize = 0;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 2 MiB page mapped in the level 1 PT
#[derive(Clone, Copy)]
pub enum MegaPageSize {}
impl PageSize for MegaPageSize {
	const SIZE: usize = 2 * 1024 * 1024;
	const MAP_LEVEL: usize = 1;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 1 GiB page mapped in the level 2 PT
#[derive(Clone, Copy)]
pub enum GigaPageSize {}
impl PageSize for GigaPageSize {
	const SIZE: usize = 1024 * 1024 * 1024;
	const MAP_LEVEL: usize = 2;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A 512 GiB page mapped in the level 3 PT
#[derive(Clone, Copy)]
pub enum TeraPageSize {}
impl PageSize for TeraPageSize {
	const SIZE: usize = 512 * 1024 * 1024 * 1024;
	const MAP_LEVEL: usize = 3;
	const MAP_EXTRA_FLAG: PageTableEntryFlags = PageTableEntryFlags::BLANK;
}

/// A memory page of the size given by S.
#[derive(Clone, Copy)]
struct Page<S: PageSize> {
	/// Virtual memory address of this page.
	/// This is rounded to a page size boundary on creation.
	virtual_address: usize,

	/// Required by Rust to support the S parameter.
	size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
	/// Flushes this page from the TLB of this CPU.
	fn flush_from_tlb(&self) {
		todo!();
	}

	/// Returns whether the given virtual address is a valid one in the RISC-V Sv48 memory model.
	///
	/// RISC-V Sv48 supports 48-bit for virtual memory addresses.
	/// This is enforced by requiring bits 63 through 48 to replicate bit 47 (cf. RISC-V Privileged Spec Section 4.5).
	/// As a consequence, the address space is divided into the two valid regions 0x8000_0000_0000
	/// and 0xFFFF_8000_0000_0000.
	///
	/// Although we could make this check depend on the actual linear address width from the CPU,
	/// any extension above 48-bit would require a new page table level, which we don't implement.
	fn is_valid_address(virtual_address: usize) -> bool {
		virtual_address < 0x8000_0000_0000 || virtual_address >= 0xFFFF_8000_0000_0000
	}

	/// Returns a Page including the given virtual address.
	/// That means, the address is rounded down to a page size boundary.
	fn including_address(virtual_address: usize) -> Self {
		assert!(Self::is_valid_address(virtual_address));

		Self {
			virtual_address: align_down!(virtual_address, S::SIZE),
			size: PhantomData,
		}
	}

	/// Returns a PageIter to iterate from the given first Page to the given last Page (inclusive).
	fn range(first: Self, last: Self) -> PageIter<S> {
		assert!(first.virtual_address <= last.virtual_address);
		PageIter {
			current: first,
			last: last,
		}
	}

	/// Returns the index of this page in the table given by L.
	fn table_index<L: PageTableLevel>(&self) -> usize {
		assert!(L::LEVEL >= S::MAP_LEVEL);
		self.virtual_address >> PAGE_BITS >> L::LEVEL * PAGE_MAP_BITS & PAGE_MAP_MASK
	}
}

/// An iterator to walk through a range of pages of size S.
struct PageIter<S: PageSize> {
	current: Page<S>,
	last: Page<S>,
}

impl<S: PageSize> Iterator for PageIter<S> {
	type Item = Page<S>;

	fn next(&mut self) -> Option<Page<S>> {
		if self.current.virtual_address <= self.last.virtual_address {
			let p = self.current;
			self.current.virtual_address += S::SIZE;
			Some(p)
		} else {
			None
		}
	}
}

/// An interface to allow for a generic implementation of struct PageTable for all 4 page tables.
/// Must be implemented by all page tables.
trait PageTableLevel {
	/// Numeric page table level (from 0 for PGT through 3 for PML4) to enable numeric comparisons.
	const LEVEL: usize;
}

/// An interface for page tables with sub page tables (all except PGT).
/// Having both PageTableLevel and PageTableLevelWithSubtables leverages Rust's typing system to provide
/// a next_table_for_page method only for those that have sub page tables.
///
/// Kudos to Philipp Oppermann for the trick!
trait PageTableLevelWithSubtables: PageTableLevel {
	type SubtableLevel;
}

macro_rules! define_page_table_level {
	($name:ident, $level:expr) => {
		enum $name {}
		impl PageTableLevel for $name {
			const LEVEL: usize = $level;
		}
	};
	($name:ident, $level:expr, $next_level:ident) => {
		define_page_table_level!($name, $level);
		impl PageTableLevelWithSubtables for $name {
			type SubtableLevel = $next_level;
		}
	};
}

define_page_table_level!(PageTableLevel3, 3, PageTableLevel2);
define_page_table_level!(PageTableLevel2, 2, PageTableLevel1);
define_page_table_level!(PageTableLevel1, 1, PageTableLevel0);
define_page_table_level!(PageTableLevel0, 0);

/// Representation of any page table (PML4, PDPT, PDT, PGT) in memory.
/// Parameter L supplies information for Rust's typing system to distinguish between the different tables.
struct PageTable<L> {
	/// Each page table has 512 entries (can be calculated using PAGE_MAP_BITS).
	entries: [PageTableEntry; 1 << PAGE_MAP_BITS],

	/// Required by Rust to support the L parameter.
	level: PhantomData<L>,
}

/// A trait defining methods every page table has to implement.
/// This additional trait is necessary to make use of Rust's specialization feature and provide a default
/// implementation of some methods.
trait PageTableMethods {
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool;
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool;
}

impl<L: PageTableLevel> PageTableMethods for PageTable<L> {
	/// Maps a single page in this table to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// Must only be called if a page of this size is mapped at this page table level!
	fn map_page_in_this_table<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		assert_eq!(L::LEVEL, S::MAP_LEVEL);
		let index = page.table_index::<L>();
		let flush = self.entries[index].is_present();

		self.entries[index].set(
			physical_address,
			PageTableEntryFlags::DIRTY | S::MAP_EXTRA_FLAG | flags,
		);

		if flush {
			page.flush_from_tlb();
		}

		flush
	}

	/// Maps a single page to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// This is the default implementation that just calls the map_page_in_this_table method.
	/// It is overridden by a specialized implementation for all tables with sub tables (all except PGT).
	default fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		self.map_page_in_this_table::<S>(page, physical_address, flags)
	}
}

impl<L: PageTableLevelWithSubtables> PageTableMethods for PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Maps a single page to the given physical address.
	/// Returns whether an existing entry was updated. You can use this return value to flush TLBs.
	///
	/// This is the implementation for all tables with subtables (PML4, PDPT, PDT).
	/// It overrides the default implementation above.
	fn map_page<S: PageSize>(
		&mut self,
		page: Page<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) -> bool {
		assert!(L::LEVEL >= S::MAP_LEVEL);

		if L::LEVEL > S::MAP_LEVEL {
			let index = page.table_index::<L>();

			// Does the table exist yet?
			if !self.entries[index].is_present() {
				// Allocate a single 4 KiB page for the new entry and mark it as a valid, writable subtable.
				let physical_address = physicalmem::allocate(BasePageSize::SIZE);
				self.entries[index].set(physical_address, PageTableEntryFlags::VALID);

				// Mark all entries as unused in the newly created table.
				let subtable = self.subtable::<S>(page);
				for entry in subtable.entries.iter_mut() {
					entry.physical_address_and_flags = 0;
				}
			}

			let subtable = self.subtable::<S>(page);
			subtable.map_page::<S>(page, physical_address, flags)
		} else {
			// Calling the default implementation from a specialized one is not supported (yet),
			// so we have to resort to an extra function.
			self.map_page_in_this_table::<S>(page, physical_address, flags)
		}
	}
}

impl<L: PageTableLevelWithSubtables> PageTable<L>
where
	L::SubtableLevel: PageTableLevel,
{
	/// Returns the next subtable for the given page in the page table hierarchy.
	///
	/// Must only be called if a page of this size is mapped in a subtable!
	fn subtable<S: PageSize>(&self, page: Page<S>) -> &mut PageTable<L::SubtableLevel> {
		assert!(L::LEVEL > S::MAP_LEVEL);

		// Calculate the address of the subtable.
		let index = page.table_index::<L>();
		let table_address = self as *const PageTable<L> as usize;
		let subtable_address = (table_address << PAGE_MAP_BITS) | (index << PAGE_BITS);
		unsafe { &mut *(subtable_address as *mut PageTable<L::SubtableLevel>) }
	}

	/// Maps a continuous range of pages.
	///
	/// # Arguments
	///
	/// * `range` - The range of pages of size S
	/// * `physical_address` - First physical address to map these pages to
	/// * `flags` - Flags from PageTableEntryFlags to set for the page table entry (e.g. WRITABLE or EXECUTE_DISABLE).
	///             The PRESENT, ACCESSED, and DIRTY flags are already set automatically.
	fn map_pages<S: PageSize>(
		&mut self,
		range: PageIter<S>,
		physical_address: usize,
		flags: PageTableEntryFlags,
	) {
		let mut current_physical_address = physical_address;

		for page in range {
			self.map_page::<S>(page, current_physical_address, flags);
			current_physical_address += S::SIZE;
		}
	}
}

#[inline]
fn get_page_range<S: PageSize>(virtual_address: usize, count: usize) -> PageIter<S> {
	let first_page = Page::<S>::including_address(virtual_address);
	let last_page = Page::<S>::including_address(virtual_address + (count - 1) * S::SIZE);
	Page::range(first_page, last_page)
}

/*
pub fn map<S: PageSize>(
	virtual_address: usize,
	physical_address: usize,
	count: usize,
	flags: PageTableEntryFlags,
) {
	let range = get_page_range::<S>(virtual_address, count);
	let root_pagetable = unsafe { &mut *PML4_ADDRESS };
	root_pagetable.map_pages(range, physical_address, flags);
}
*/
