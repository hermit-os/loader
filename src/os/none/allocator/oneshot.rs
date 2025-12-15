//! A one-shot allocator.
//!
//! This is a simple allocator design which can only allocate once.

use core::cell::Cell;
use core::ptr::NonNull;

use allocator_api2::alloc::{AllocError, Allocator, Layout};

/// A simple, `!Sync` implementation of a one-shot allocator.
///
/// This allocator manages the provided memory.
pub struct OneshotAllocator {
	mem: Cell<*mut u8>,
	// TODO: perhaps provide maximum suitable address
}

unsafe impl Allocator for OneshotAllocator {
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		assert!(layout.align() <= 8);
		// `mem` is already aligned.

		let mid = layout.size();
		if mid >= (usize::MAX / 2) {
			return Err(AllocError);
		}

		match NonNull::new(self.mem.take()) {
			None => Err(AllocError),
			Some(mem) => Ok(NonNull::slice_from_raw_parts(mem, layout.size())),
		}
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
		if self.mem.get().is_null() {
			self.mem.set(ptr.as_ptr());
		} else {
			#[cfg(debug_assertions)]
			panic!("Tried to deallocate pointer that was allocated from a different allocator");
		}
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		assert!(
			new_layout.size() >= old_layout.size(),
			"`new_layout.size()` must be greater than or equal to `old_layout.size()`"
		);
		assert!(new_layout.align() <= 8);

		Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		assert!(
			new_layout.size() >= old_layout.size(),
			"`new_layout.size()` must be greater than or equal to `old_layout.size()`"
		);
		assert!(new_layout.align() <= 8);

		unsafe {
			ptr.add(old_layout.size())
				.as_ptr()
				.write_bytes(0, new_layout.size() - old_layout.size())
		};

		Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
	}
}

impl OneshotAllocator {
	pub fn new(ptr: NonNull<u8>) -> Self {
		let align_offset = ptr.align_offset(8);
		Self {
			mem: Cell::new(unsafe { ptr.add(align_offset) }.as_ptr()),
		}
	}
}
