use pci_types::{ConfigRegionAccess, PciAddress};
use x86::io::*;

pub(crate) const PCI_MAX_BUS_NUMBER: u8 = 32;
pub(crate) const PCI_MAX_DEVICE_NUMBER: u8 = 32;

pub(crate) const PCI_CONFIG_ADDRESS_PORT: u16 = 0xCF8;
const PCI_CONFIG_ADDRESS_ENABLE: u32 = 1 << 31;

const PCI_CONFIG_DATA_PORT: u16 = 0xCFC;

#[derive(Debug, Copy, Clone)]
pub(crate) struct PciConfigRegion;

impl PciConfigRegion {
	pub const fn new() -> Self {
		Self {}
	}
}

impl ConfigRegionAccess for PciConfigRegion {
	#[inline]
	fn function_exists(&self, _address: PciAddress) -> bool {
		true
	}

	#[inline]
	unsafe fn read(&self, pci_addr: PciAddress, register: u16) -> u32 {
		let address = PCI_CONFIG_ADDRESS_ENABLE
			| u32::from(pci_addr.bus()) << 16
			| u32::from(pci_addr.device()) << 11
			| u32::from(register);
		unsafe {
			outl(PCI_CONFIG_ADDRESS_PORT, address);
			u32::from_le(inl(PCI_CONFIG_DATA_PORT))
		}
	}

	#[inline]
	unsafe fn write(&self, pci_addr: PciAddress, register: u16, value: u32) {
		let address = PCI_CONFIG_ADDRESS_ENABLE
			| u32::from(pci_addr.bus()) << 16
			| u32::from(pci_addr.device()) << 11
			| u32::from(register);
		unsafe {
			outl(PCI_CONFIG_ADDRESS_PORT, address);
			outl(PCI_CONFIG_DATA_PORT, value.to_le());
		}
	}
}
