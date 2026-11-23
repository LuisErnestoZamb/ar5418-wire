//! Pure, host-testable bring-up logic for the Atheros AR5418 Wi-Fi chip
//! (PCI `168c:0024`, class `0280`) in the from-scratch `reemplazo` kernel.
//!
//! This crate is the **F1 probe** foundation: it identifies the chip, resets
//! it, reads the on-chip EEPROM, validates the base header, extracts the MAC
//! address and reports the supported bands/channels. It contains no `unsafe`
//! code and no dependencies; the MMIO/PCI/DMA glue lives in the kernel, behind
//! the [`Mmio`], [`PciConfig`] and [`Delay`] traits.
//!
//! # Sources — no register here is remembered or invented
//!
//! Register and sequence truth comes from Linux v6.6 `ath9k`, fetched from
//! the pinned URLs below. Line numbers in `file.c:NNNN` comments refer to
//! exactly these files (SHA-256 prefixes shown):
//!
//! | file | SHA-256 | used for |
//! |---|---|---|
//! | `ath9k/reg.h` | `629a6df177d63b2a99202ab97a3641c488b5f1ac738f6f15612c7b4ebacebade` | SREV, RTC reset, EEPROM status, PCIe IDs |
//! | `ath9k/hw.c` | `8c3930144d5b8390e73d4b10855447143fb75daa36a14fdb84597f6328593817` | revision decode, reset/wake sequences |
//! | `ath9k/eeprom.h` | `b6bc5f1bffd56f5b8828d14635eac4846487ff6f2ef06344d474ac4c1c11b7b0` | magic, offsets, def base header |
//! | `ath9k/eeprom_def.c` | `b30a55af8531c9e3dd8c13280b1e8ebef08d650aa17f0a92478fbb2efd20d097` | def EEPROM read/checksum/MAC |
//! | `ath9k/eeprom.c` | `097a787babd59c38317fce0f4ad8462d3d62bf6cbdf05e23a271c61e261004ed` | magic swap, checksum, version |
//! | `ath9k/pci.c` | `c4213a730e09c01d0eefa7eab5533fc35c54cb1c879b5448c34f90843bc5763e` | PCI probe, EEPROM access |
//! | `ath9k/common-init.c` | `087a0d90d0918d1a3b012292958492e2dc33a1ad843eb20808489b584e90d888` | 2 GHz / 5 GHz channel tables |
//! | `ath9k/phy.h` | `edab45a51cfa37c1226a90d59d1f7474a65758802518b0d00f6e34fd93ddddf2` | `AR_PHY_CHIP_ID` |
//! | `ath/reg.h` | `e719ccdd6f97dd727f158573a8a1ee20ca232acdf4564448514a2494a2a38464` | `AR_STA_ID1` (shared MAC register) |
//!
//! Pinned URLs:
//! - `https://raw.githubusercontent.com/torvalds/linux/v6.6/drivers/net/wireless/ath/ath9k/reg.h`
//! - `https://raw.githubusercontent.com/torvalds/linux/v6.6/drivers/net/wireless/ath/ath9k/hw.c`
//! - `https://raw.githubusercontent.com/torvalds/linux/v6.6/drivers/net/wireless/ath/ath9k/eeprom_def.c`
//! - `https://raw.githubusercontent.com/torvalds/linux/v6.6/drivers/net/wireless/ath/ath9k/pci.c`
//! - `https://raw.githubusercontent.com/torvalds/linux/v6.6/drivers/net/wireless/ath/ath9k/common-init.c`
//!
//! # Hardware facts assumed by the probe
//!
//! - AR5418 is the PCIe member of the AR5416 family:
//!   `AR5416_DEVID_PCIE = 0x0024` (`pci.c:28`), SREV version `0xC`
//!   (`reg.h:774`). The PCI (non-PCIe) sibling is `0x0023` / version `0xD`.
//! - SREV lives at `0x4020` (`reg.h:753`); all-ones means the BAR is not
//!   decoding.
//! - The AR5416 family uses the **def** EEPROM layout (`eeprom.c:676-680`);
//!   its base header starts at EEPROM word `0x100` (`eeprom_def.c:101`).
//! - EEPROM accesses are triggered by a *read* of
//!   `0x2000 + (word << 2)` (`eeprom.h:68`, `pci.c:805`) and complete when
//!   the busy/protect bits in `0x407c` clear (`reg.h:1250-1257`).
//!
//! # Deliberate deviations from Linux
//!
//! - **Fail closed.** Linux replaces an invalid EEPROM MAC with a random one
//!   (`hw.c:486-511`). This crate returns [`Error::BadMac`] instead: F1 must
//!   prove the real MAC is readable, not paper over it.
//! - **Surface `ABSENT_ACCESS`.** Linux ignores the status bit and later fails
//!   on the magic. We report [`Error::EepromAbsent`] so a missing EEPROM/OTP
//!   variant is diagnosable.
//! - **No flash/OTP/platform-data path yet.** F1 reads the chip EEPROM only;
//!   `AH_USE_EEPROM` / flash variants are a later phase.
//!
//! # Status
//!
//! F0 (scaffold) and F1 (probe/EEPROM/MAC/channels) are implemented and
//! host-tested. RX rings (F2), TX (F3), association (F4), WPA2 (F5) and the
//! `net::Nic` bridge (F6) are not. QEMU does not emulate this chip, so F1's
//! acceptance is one [`DeviceInfo`] log line from the real MacBook2,1.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod test_support;

pub mod channel;
pub mod eeprom;
pub mod error;
pub mod pci;
pub mod platform;
pub mod probe;
pub mod regs;

pub use channel::{Band, Bands, CHANNELS_2GHZ, CHANNELS_5GHZ, Channel};
pub use eeprom::{EepromBaseHeader, EepromInfo};
pub use error::Error;
pub use pci::{PciId, is_target, mem_bar_address};
pub use platform::{Delay, Mmio, PciConfig};
pub use probe::{DeviceInfo, ProbeConfig, probe};
pub use regs::{MacRevision, decode_srev};
