//! Fail-closed probe errors.

use core::fmt;

/// Which bounded reset wait expired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetStage {
    /// Waiting for `AR_RTC_STATUS` to report `AR_RTC_STATUS_ON` after
    /// releasing `AR_RTC_RESET` (`hw.c:1466-1476`).
    RtcWake,
    /// Waiting for `AR_RTC_RC_M` to clear after the warm MAC reset
    /// (`hw.c:1429-1432`).
    MacReset,
}

impl fmt::Display for ResetStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResetStage::RtcWake => f.write_str("rtc-wake"),
            ResetStage::MacReset => f.write_str("mac-reset"),
        }
    }
}

/// Every failure the F1 probe can report.
///
/// The probe never fabricates a MAC address, never accepts a bad checksum and
/// never loops without a deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The PCI function is not the AR5418 this crate drives.
    NotAth9k { vendor: u16, device: u16 },
    /// `AR_SREV` read as all-ones: the BAR is not decoding.
    SrevReadFailed,
    /// The SREV revision is outside the AR5416 family (versions `0xC`/`0xD`).
    UnsupportedMac { version: u16, revision: u8 },
    /// A bounded RTC/MAC reset wait expired.
    ResetTimeout { stage: ResetStage },
    /// The EEPROM status register never cleared busy/protect access.
    EepromTimeout { offset: u16 },
    /// The EEPROM status register reported an access to an absent EEPROM.
    EepromAbsent { offset: u16 },
    /// Neither `0xa55a` nor its byte-swapped form was at EEPROM word 0.
    BadEepromMagic { magic: u16 },
    /// The base header length is zero, odd, or smaller than the 64-byte header.
    BadEepromLength { length: u16 },
    /// The XOR over the EEPROM words is not `0xffff` (`eeprom.c:220-232`).
    BadEepromChecksum { sum: u16 },
    /// Not def-EEPROM version `0xE`, or minor revision below 1
    /// (`eeprom.h:132-136`, `eeprom.c:234-246`).
    BadEepromVersion { version: u16 },
    /// The EEPROM MAC is zero or multicast (`hw.c:486-511`).
    BadMac { mac: [u8; 6] },
    /// `opCapFlags` advertises neither 2 GHz nor 5 GHz operation.
    NoBands { op_cap_flags: u8 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAth9k { vendor, device } => {
                write!(f, "not an AR5418: pci={vendor:04x}:{device:04x}")
            }
            Error::SrevReadFailed => f.write_str("AR_SREV read all-ones (BAR not decoding)"),
            Error::UnsupportedMac { version, revision } => {
                write!(f, "unsupported MAC revision {version:#x}.{revision}")
            }
            Error::ResetTimeout { stage } => write!(f, "reset timeout at {stage}"),
            Error::EepromTimeout { offset } => {
                write!(f, "EEPROM access timed out at word {offset:#x}")
            }
            Error::EepromAbsent { offset } => {
                write!(f, "EEPROM absent (status bit) at word {offset:#x}")
            }
            Error::BadEepromMagic { magic } => {
                write!(f, "bad EEPROM magic {magic:#06x} (want 0xa55a/0x5aa5)")
            }
            Error::BadEepromLength { length } => {
                write!(f, "bad EEPROM length {length} bytes")
            }
            Error::BadEepromChecksum { sum } => {
                write!(f, "bad EEPROM checksum {sum:#06x} (want 0xffff)")
            }
            Error::BadEepromVersion { version } => {
                write!(f, "bad EEPROM version {version:#06x} (want 0xe.1+)")
            }
            Error::BadMac { mac } => write!(
                f,
                "invalid EEPROM MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            ),
            Error::NoBands { op_cap_flags } => {
                write!(
                    f,
                    "EEPROM advertises no bands (opCapFlags={op_cap_flags:#04x})"
                )
            }
        }
    }
}

impl core::error::Error for Error {}
