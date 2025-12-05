//! A one-shot allocator.
//!
//! This is a simple allocator design which can only allocate once.

use core::cell::Cell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

use allocator_api2::alloc::{AllocError, Allocator, Layout};

/// A simple, `!Sync` implementation of a one-shot allocator.
///
/// This allocator manages the provided memory.
pub struct OneshotAllocator {
	mem: Cell<*mut u8>,
}

unsafe impl Allocator for OneshotAllocator {
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		assert!(layout.align() <= 8);
		// `mem` is already aligned.

		match NonNull::new(self.mem.take()) {
			None => return Err(AllocError),
			Some(mem) => {
				let mid = layout.size();
				if mid >= (usize::MAX / 2) {
					self.mem.set(mem);
					Err(AllocError)
				} else {
					Ok(mem)
				}
			}
		}
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		if self.mem.get().is_null() {
			self.mem.set(ptr);
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
		assert!(new_layout.align() <= 8);
		Ok(ptr)
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

		Ok(ptr)
	}
}

impl OneshotAllocator {
	pub fn new(ptr: NonNull<u8>) -> Self {
		let align_offset = ptr.align_offset(8);
		Self {
			mem: Cell::new(ptr.add(align_offset)),
		}
	}
}
