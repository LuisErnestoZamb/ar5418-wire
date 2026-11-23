//! Register offsets, bit fields and SREV decoding for the AR5418.
//!
//! Every constant cites Linux v6.6 ath9k (`reg.h` / `phy.h` / `ath/reg.h`
//! line numbers). AR5418 = AR5416 family; both SREV variants (`0xC` PCIe,
//! `0xD` PCI) are accepted, other revisions are rejected.

use core::fmt;

/// `AR_SREV` (`reg.h:753`) — chip revision register.
pub const AR_SREV: u32 = 0x4020;
/// `AR_SREV_ID` mask (`reg.h:757`) — low byte holds version/revision.
pub const AR_SREV_ID: u32 = 0x0000_00FF;
/// `AR_SREV_VERSION` mask (`reg.h:759`).
pub const AR_SREV_VERSION: u32 = 0x0000_00F0;
/// `AR_SREV_VERSION_S` (`reg.h:760`).
pub const AR_SREV_VERSION_SHIFT: u32 = 4;
/// `AR_SREV_REVISION` mask (`reg.h:761`).
pub const AR_SREV_REVISION: u32 = 0x0000_0007;

/// `AR_SREV_VERSION2` (`reg.h:764`) — new-format version field.
pub const AR_SREV_VERSION2: u32 = 0xFFFC_0000;
/// `AR_SREV_VERSION2_S` (`reg.h:765`).
pub const AR_SREV_VERSION2_SHIFT: u32 = 18;
/// `AR_SREV_TYPE2_HOST_MODE` (`reg.h:769`) — 1 means non-PCIe.
pub const AR_SREV_TYPE2_HOST_MODE: u32 = 0x0000_2000;
/// `AR_SREV_REVISION2` (`reg.h:770`).
pub const AR_SREV_REVISION2: u32 = 0x0000_0F00;
/// `AR_SREV_REVISION2_S` (`reg.h:771`).
pub const AR_SREV_REVISION2_SHIFT: u32 = 8;

/// `AR_SREV_VERSION_5416_PCIE` (`reg.h:774`) — AR5418 (PCIe).
pub const SREV_VERSION_5416_PCIE: u16 = 0xC;
/// `AR_SREV_VERSION_5416_PCI` (`reg.h:776`) — AR5416 PCI sibling.
pub const SREV_VERSION_5416_PCI: u16 = 0xD;
/// `AR_SREV_REVISION_5416_10` (`reg.h:777`).
pub const SREV_REVISION_5416_10: u8 = 0;
/// `AR_SREV_REVISION_5416_20` (`reg.h:778`).
pub const SREV_REVISION_5416_20: u8 = 1;
/// `AR_SREV_REVISION_5416_22` (`reg.h:779`).
pub const SREV_REVISION_5416_22: u8 = 2;

/// `AR_RC` (`reg.h:697`) — host-bridge control.
pub const AR_RC: u32 = 0x4000;
/// `AR_RC_AHB` (`reg.h:698`).
pub const AR_RC_AHB: u32 = 0x0000_0001;
/// `AR_RC_HOSTIF` (`reg.h:700`).
pub const AR_RC_HOSTIF: u32 = 0x0000_0100;

/// `AR_RTC_RC` (`reg.h:1342`) — reset-control state.
pub const AR_RTC_RC: u32 = 0x7000;
/// `AR_RTC_RC_M` (`reg.h:1343`) — reset bits being applied.
pub const AR_RTC_RC_M: u32 = 0x0000_0003;
/// `AR_RTC_RC_MAC_WARM` (`reg.h:1344`).
pub const AR_RTC_RC_MAC_WARM: u32 = 0x0000_0001;
/// `AR_RTC_RC_MAC_COLD` (`reg.h:1345`).
pub const AR_RTC_RC_MAC_COLD: u32 = 0x0000_0002;
/// `AR_RTC_RC_COLD_RESET` (`reg.h:1346`).
pub const AR_RTC_RC_COLD_RESET: u32 = 0x0000_0004;
/// `AR_RTC_RC_WARM_RESET` (`reg.h:1347`).
pub const AR_RTC_RC_WARM_RESET: u32 = 0x0000_0008;

/// `AR_RTC_RESET` (`reg.h:1381`).
pub const AR_RTC_RESET: u32 = 0x7040;
/// `AR_RTC_RESET_EN` (`reg.h:1383`).
pub const AR_RTC_RESET_EN: u32 = 0x0000_0001;
/// `AR_RTC_STATUS` (`reg.h:1385`).
pub const AR_RTC_STATUS: u32 = 0x7044;
/// `AR_RTC_STATUS_M` (`reg.h:1388`).
pub const AR_RTC_STATUS_M: u32 = 0x0000_000F;
/// `AR_RTC_STATUS_SHUTDOWN` (`reg.h:1393`).
pub const AR_RTC_STATUS_SHUTDOWN: u32 = 0x0000_0001;
/// `AR_RTC_STATUS_ON` (`reg.h:1394`).
pub const AR_RTC_STATUS_ON: u32 = 0x0000_0002;
/// `AR_RTC_STATUS_SLEEP` (`reg.h:1395`).
pub const AR_RTC_STATUS_SLEEP: u32 = 0x0000_0004;
/// `AR_RTC_STATUS_WAKEUP` (`reg.h:1396`).
pub const AR_RTC_STATUS_WAKEUP: u32 = 0x0000_0008;
/// `AR_RTC_FORCE_WAKE` (`reg.h:1403`).
pub const AR_RTC_FORCE_WAKE: u32 = 0x704C;
/// `AR_RTC_FORCE_WAKE_EN` (`reg.h:1405`).
pub const AR_RTC_FORCE_WAKE_EN: u32 = 0x0000_0001;
/// `AR_RTC_FORCE_WAKE_ON_INT` (`reg.h:1406`).
pub const AR_RTC_FORCE_WAKE_ON_INT: u32 = 0x0000_0002;

/// `AR_INTR_SYNC_CAUSE` (`reg.h:1041`).
pub const AR_INTR_SYNC_CAUSE: u32 = 0x4028;
/// `AR_INTR_SYNC_ENABLE` (`reg.h:1045`).
pub const AR_INTR_SYNC_ENABLE: u32 = 0x402C;
/// `AR_INTR_SYNC_LOCAL_TIMEOUT` (`reg.h:1063`).
pub const AR_INTR_SYNC_LOCAL_TIMEOUT: u32 = 0x0000_2000;
/// `AR_INTR_SYNC_RADM_CPL_TIMEOUT` (`reg.h:1062`).
pub const AR_INTR_SYNC_RADM_CPL_TIMEOUT: u32 = 0x0000_1000;

/// `AR_STA_ID1` (`ath/reg.h:27` — shared MAC register, not ath9k/reg.h).
pub const AR_STA_ID1: u32 = 0x8004;
/// `AR_STA_ID1_PWR_SAV` (`reg.h:1621`).
pub const AR_STA_ID1_PWR_SAV: u32 = 0x0004_0000;

/// `AR_PHY_CHIP_ID` (`phy.h:43`) — PHY revision register read after wake.
pub const AR_PHY_CHIP_ID: u32 = 0x9818;

/// `AR5416_EEPROM_OFFSET` (`eeprom.h:68`) — EEPROM MMIO window base.
pub const AR_EEPROM_OFFSET: u32 = 0x2000;
/// `AR5416_EEPROM_S` (`eeprom.h:67`) — word-offset shift.
pub const AR_EEPROM_S: u32 = 2;
/// `AR_EEPROM_STATUS_DATA` (`reg.h:1250`, non-9300 value).
pub const AR_EEPROM_STATUS_DATA: u32 = 0x407C;
/// `AR_EEPROM_STATUS_DATA_VAL` (`reg.h:1252`).
pub const AR_EEPROM_STATUS_DATA_VAL: u32 = 0x0000_FFFF;
/// `AR_EEPROM_STATUS_DATA_BUSY` (`reg.h:1254`).
pub const AR_EEPROM_STATUS_DATA_BUSY: u32 = 0x0001_0000;
/// `AR_EEPROM_STATUS_DATA_PROT_ACCESS` (`reg.h:1256`).
pub const AR_EEPROM_STATUS_DATA_PROT_ACCESS: u32 = 0x0004_0000;
/// `AR_EEPROM_STATUS_DATA_ABSENT_ACCESS` (`reg.h:1257`).
pub const AR_EEPROM_STATUS_DATA_ABSENT_ACCESS: u32 = 0x0008_0000;
/// `AR5416_EEPROM_MAGIC` (`eeprom.h:40`).
pub const AR_EEPROM_MAGIC: u16 = 0xA55A;
/// Byte-swapped magic form, seen when the EEPROM stores words big-endian
/// (`eeprom.c:179`).
pub const AR_EEPROM_MAGIC_SWAPPED: u16 = 0x5AA5;
/// `AR5416_EEPROM_MAGIC_OFFSET` (`eeprom.h:66`).
pub const AR_EEPROM_MAGIC_OFFSET: u16 = 0x0;
/// Def-EEPROM start word (`eeprom_def.c:101`).
pub const AR_EEPROM_DEF_START: u16 = 0x100;
/// `AR5416_EEPROM_MAX` (`eeprom.h:69`).
pub const AR_EEPROM_MAX: u16 = 0xAE0;
/// `SIZE_EEPROM_DEF` = `sizeof(ar5416_eeprom_def)/2` (`eeprom_def.c:95`),
/// 3261 packed bytes / 2 = 1630 words. Cap for checksum length clamping
/// (`eeprom_def.c:281`).
pub const DEF_EEPROM_WORDS_MAX: u16 = 1630;
/// `AH_TIME_QUANTUM` (`hw.h:179`) — 10 µs poll step.
pub const AH_TIME_QUANTUM_US: u32 = 10;
/// `AH_WAIT_TIMEOUT` (`hw.h:177`) — 100 ms default deadline.
pub const AH_WAIT_TIMEOUT_US: u32 = 100_000;
/// `POWER_UP_TIME` (`hw.h:181`) — RTC wake deadline, 10 ms.
pub const POWER_UP_TIME_US: u32 = 10_000;

/// Decoded SREV identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacRevision {
    /// MAC version nibble (`AR_SREV_VERSION`) or 14-bit type2 version.
    pub version: u16,
    /// MAC revision bits (`AR_SREV_REVISION` or `AR_SREV_REVISION2`).
    pub revision: u8,
    /// True when the chip reports PCIe (type2 host-mode inverse / version `0xC`).
    pub pcie: bool,
    /// Raw SREV dword, kept for logging.
    pub raw: u32,
}

impl MacRevision {
    /// True for the AR5416 family (`AR_SREV_5416`, `reg.h:831`).
    pub const fn is_5416(self) -> bool {
        matches!(self.version, SREV_VERSION_5416_PCIE | SREV_VERSION_5416_PCI)
    }
}

impl fmt::Display for MacRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}.{}", self.version, self.revision)
    }
}

/// Decode the SREV dword the way `ath9k_hw_read_revisions` does
/// (`hw.c:254-307`): all-ones means the BAR is not decoding; a low byte of
/// `0xff` selects the new type2 layout, otherwise the old nibble layout.
pub const fn decode_srev(raw: u32) -> Option<MacRevision> {
    if raw == u32::MAX {
        return None;
    }
    let val = raw & AR_SREV_ID;
    if val == 0xFF {
        let version = ((raw & AR_SREV_VERSION2) >> AR_SREV_VERSION2_SHIFT) as u16;
        let revision = ((raw & AR_SREV_REVISION2) >> AR_SREV_REVISION2_SHIFT) as u8;
        let pcie = raw & AR_SREV_TYPE2_HOST_MODE == 0;
        Some(MacRevision {
            version,
            revision,
            pcie,
            raw,
        })
    } else {
        let version = ((val & AR_SREV_VERSION) >> AR_SREV_VERSION_SHIFT) as u16;
        let revision = (val & AR_SREV_REVISION) as u8;
        let pcie = version == SREV_VERSION_5416_PCIE;
        Some(MacRevision {
            version,
            revision,
            pcie,
            raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_5416_pcie_old_format() {
        // AR5418: version 0xC, revision 2 (0xC2).
        let rev = decode_srev(0x0000_00C2).unwrap();
        assert_eq!(rev.version, SREV_VERSION_5416_PCIE);
        assert_eq!(rev.revision, SREV_REVISION_5416_22);
        assert!(rev.pcie);
        assert!(rev.is_5416());
        assert_eq!(std::format!("{rev}"), "0xc.2");
    }

    #[test]
    fn decodes_5416_pci_old_format() {
        let rev = decode_srev(0x0000_00D1).unwrap();
        assert_eq!(rev.version, SREV_VERSION_5416_PCI);
        assert_eq!(rev.revision, SREV_REVISION_5416_20);
        assert!(!rev.pcie);
        assert!(rev.is_5416());
    }

    #[test]
    fn decodes_type2_format() {
        // AR9280-style: version2 0x80, host-mode 0 (PCIe), revision2 1.
        let raw = (0x80u32 << AR_SREV_VERSION2_SHIFT) | (1u32 << AR_SREV_REVISION2_SHIFT) | 0xFF;
        let rev = decode_srev(raw).unwrap();
        assert_eq!(rev.version, 0x80);
        assert_eq!(rev.revision, 1);
        assert!(rev.pcie);
        assert!(!rev.is_5416());

        // Host-mode set -> not PCIe.
        let raw = raw | AR_SREV_TYPE2_HOST_MODE;
        let rev = decode_srev(raw).unwrap();
        assert!(!rev.pcie);
    }

    #[test]
    fn type2_version_keeps_high_bits() {
        // AR9300 family versions exceed 8 bits; must not truncate.
        let raw = (0x1C0u32 << AR_SREV_VERSION2_SHIFT) | 0xFF;
        let rev = decode_srev(raw).unwrap();
        assert_eq!(rev.version, 0x1C0);
        assert!(!rev.is_5416());
    }

    #[test]
    fn all_ones_is_failure() {
        assert!(decode_srev(u32::MAX).is_none());
    }
}
