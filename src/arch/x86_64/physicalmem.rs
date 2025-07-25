use core::num::NonZeroUsize;

use log::debug;
use one_shot_mutex::sync::OneShotMutex;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};

static PHYS_ALLOC: OneShotMutex<Option<PhysAllocInner>> = OneShotMutex::new(None);

struct PhysAllocInner {
	next: NonZeroUsize,
}

impl PhysAllocInner {
	pub fn new(addr: NonZeroUsize) -> Self {
		Self { next: addr }
	}

	pub fn allocate(&mut self, size: usize) -> usize {
		assert_ne!(size, 0);
		assert_eq!(size % Size4KiB::SIZE as usize, 0);

		let addr = self.next.get();
		self.next = self.next.checked_add(size).unwrap();
		addr
	}
}
pub struct PhysAlloc;

impl PhysAlloc {
	pub fn init(addr: usize) {
		let mut phys_alloc = PHYS_ALLOC.lock();
		assert!(phys_alloc.is_none());
		phys_alloc.replace(PhysAllocInner::new(addr.try_into().unwrap()));
	}

	pub fn allocate(size: usize) -> usize {
		PHYS_ALLOC.lock().as_mut().unwrap().allocate(size)
	}
}

unsafe impl<S: PageSize> FrameAllocator<S> for PhysAlloc {
	fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
		let addr = Self::allocate(S::SIZE as usize) as u64;
		Some(PhysFrame::from_start_address(x86_64::PhysAddr::new(addr)).unwrap())
	}
}

impl<S: PageSize> FrameDeallocator<S> for PhysAlloc {
	unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>) {
		debug!("Tried to free {frame:?}");
	}
}
