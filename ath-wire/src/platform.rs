//! Platform-injection traits.
//!
//! The crate never touches real I/O. A host kernel supplies these
//! implementations; host tests supply mocks. Implementations must:
//!
//! - use **volatile** accesses for [`Mmio`] (the compiler must not cache or
//!   reorder register reads/writes),
//! - serialise PCI config accesses as required by the host bridge,
//! - make [`Delay::delay_us`] actually wait (the probe's bounded loops rely on
//!   it), and
//! - provide the barriers/ownership rules needed before DMA is enabled. F1
//!   performs no DMA, so no DMA trait is defined yet; F2 adds one with the
//!   RX-ring code rather than guessing its shape now.

/// 32-bit memory-mapped register access.
///
/// Offsets are byte offsets into the device's BAR, exactly as written in
/// `ath9k/reg.h`. The implementation must use volatile reads/writes.
pub trait Mmio {
    fn read32(&mut self, offset: u32) -> u32;
    fn write32(&mut self, offset: u32, value: u32);
}

/// Microsecond delay source.
///
/// The probe uses it for the RTC reset settle time and for bounded polling.
/// A kernel implementation can busy-wait on the TSC; a test implementation
/// just records the requested delay.
pub trait Delay {
    fn delay_us(&mut self, micros: u32);
}

/// PCI configuration-space access (mechanism #1 or ECAM — the crate does not
/// care). Only the handful of registers F1 needs are exposed.
pub trait PciConfig {
    fn read_u16(&mut self, offset: u8) -> u16;
    fn read_u32(&mut self, offset: u8) -> u32;
    fn write_u16(&mut self, offset: u8, value: u16);
}
