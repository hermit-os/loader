//! Xen ELF notes.
//!
//! This module helps with adding Xen ELF notes to guests kernels.
//!
//! For details, see [xen/include/public/elfnote.h].
//!
//! [xen/include/public/elfnote.h]: https://xenbits.xen.org/gitweb/?p=xen.git;a=blob;f=xen/include/public/elfnote.h;h=2fd8f1b770fd34214313a4b811e910958116f0c7;hb=06af9ef22996cecc2024a2e6523cec77a655581e

use core::mem;

use crate::start_info::StartInfo;

/// Creates a `XEN_ELFNOTE_PHYS32_ENTRY`.
///
/// <div class="warning">The entry itself has to be 32-bit code!</div>
///
/// # Safety
///
/// The provided `phys32_entry` needs to conform to the HVM booting
/// requirements.
///
/// # Example
///
/// ```
/// use xen_hvm::start_info::StartInfo;
///
/// unsafe extern "C" fn phys32_entry(start_info: &'static StartInfo) -> ! {
///     loop {}
/// }
///
/// xen_hvm::phys32_entry!(phys32_entry);
/// ```
#[macro_export]
macro_rules! phys32_entry {
    ($phys32_entry:expr) => {
        #[used]
        #[unsafe(link_section = ".note.Xen")]
        static XEN_ELFNOTE: $crate::elfnote::ElfnotePhys32Entry =
            $crate::elfnote::ElfnotePhys32Entry::phys32_entry($phys32_entry);
    };
}

#[repr(C, packed(4))]
pub struct Elfnote<N, D> {
    header: Nhdr32,
    name: N,
    desc: D,
}

impl<N, D> Elfnote<N, D> {
    pub const fn new(n_type: u32, name: N, desc: D) -> Self {
        Self {
            header: Nhdr32 {
                n_namesz: mem::size_of::<N>() as u32,
                n_descsz: mem::size_of::<D>() as u32,
                n_type,
            },
            name,
            desc,
        }
    }
}

pub type Phys32Entry = unsafe extern "C" fn(start_info: &'static StartInfo) -> !;
pub type ElfnotePhys32Entry = Elfnote<[u8; 4], Phys32Entry>;

impl ElfnotePhys32Entry {
    /// Physical entry point into the kernel.
    ///
    /// 32bit entry point into the kernel. When requested to launch the
    /// guest kernel in a HVM container, Xen will use this entry point to
    /// launch the guest in 32bit protected mode with paging disabled.
    /// Ignored otherwise.
    #[doc(alias = "XEN_ELFNOTE_PHYS32_ENTRY")]
    const PHYS32_ENTRY: u32 = 18;

    pub const fn phys32_entry(phys32_entry: Phys32Entry) -> Self {
        Self::new(Self::PHYS32_ENTRY, *b"Xen\0", phys32_entry)
    }
}

#[repr(C, packed(4))]
struct Nhdr32 {
    n_namesz: u32,
    n_descsz: u32,
    n_type: u32,
}
