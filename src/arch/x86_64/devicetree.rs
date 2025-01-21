use alloc::format;
use alloc::vec::Vec;

use log::{debug, info};
use multiboot::information::{MemoryType, Multiboot};
use pci_types::{Bar, EndpointHeader, PciAddress, PciHeader, MAX_BARS};
use vm_fdt::{FdtWriter, FdtWriterResult};

use super::multiboot::{mb_info, Mem};
use super::pci::{PciConfigRegion, PCI_MAX_BUS_NUMBER, PCI_MAX_DEVICE_NUMBER};
use crate::fdt::Fdt;

pub struct DeviceTree;

impl DeviceTree {
	pub fn create() -> FdtWriterResult<&'static [u8]> {
		let mut mem = Mem;
		let multiboot = unsafe { Multiboot::from_ptr(mb_info as u64, &mut mem).unwrap() };

		let memory_regions = multiboot
			.memory_regions()
			.expect("Could not find a memory map in the Multiboot information");

		let mut fdt = Fdt::new("multiboot")?.memory_regions(memory_regions)?;

		if let Some(cmdline) = multiboot.command_line() {
			fdt = fdt.bootargs(cmdline)?;
		}

		fdt = fdt.pci()?;

		let fdt = fdt.finish()?;

		Ok(fdt.leak())
	}
}
