//! Xen x86/HVM definitions.
//!
//! This crate allows guest kernels to be started via HVM.
//! [`phys32_entry!`] allows creating a `XEN_ELFNOTE_PHYS32_ENTRY`.
//! [`start_info`] provides the definitions for the start info that is passed by
//! the hypervisor.

#![no_std]

#[cfg(feature = "elfnote")]
#[doc(hidden)]
pub mod elfnote;
mod start_info;

pub use self::start_info::*;
