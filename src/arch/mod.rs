#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
pub use crate::arch::aarch64::*;

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::*;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv;
