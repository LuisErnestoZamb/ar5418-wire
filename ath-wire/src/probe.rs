//! Bounded F1 probe: PCI identity → SREV → power-on reset → EEPROM/MAC →
//! bands/channels, all through injected [`Mmio`]/[`PciConfig`]/[`Delay`].
//!
//! Sequences follow `ath9k_hw_read_revisions` (`hw.c:254`), the AR5416
//! power-on/warm reset (`hw.c:1455-1445`) and `ath9k_hw_eeprom_init`
//! (`eeprom.c:666-684`). Every wait carries a deadline; there is no unbounded
//! loop anywhere in boot.

use core::fmt;

use crate::channel::Bands;
use crate::eeprom::{EepromInfo, read_def_eeprom};
use crate::error::{Error, ResetStage};
use crate::pci::{PciId, enable_memory_and_bus_master, is_target, read_id};
use crate::platform::{Delay, Mmio, PciConfig};
use crate::regs::{
    AH_TIME_QUANTUM_US, AH_WAIT_TIMEOUT_US, AR_INTR_SYNC_CAUSE, AR_INTR_SYNC_ENABLE,
    AR_INTR_SYNC_LOCAL_TIMEOUT, AR_INTR_SYNC_RADM_CPL_TIMEOUT, AR_PHY_CHIP_ID, AR_RC, AR_RC_AHB,
    AR_RC_HOSTIF, AR_RTC_FORCE_WAKE, AR_RTC_FORCE_WAKE_EN, AR_RTC_FORCE_WAKE_ON_INT, AR_RTC_RC,
    AR_RTC_RC_M, AR_RTC_RC_MAC_WARM, AR_RTC_RESET, AR_RTC_RESET_EN, AR_RTC_STATUS, AR_RTC_STATUS_M,
    AR_RTC_STATUS_ON, AR_SREV, AR_STA_ID1, AR_STA_ID1_PWR_SAV, MacRevision, POWER_UP_TIME_US,
    decode_srev,
};

/// Bounded-wait deadlines for [`probe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeConfig {
    /// Deadline for each RTC/MAC reset wait (µs).
    pub reset_timeout_us: u32,
    /// Deadline for each EEPROM access wait (µs).
    pub eeprom_timeout_us: u32,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        ProbeConfig {
            reset_timeout_us: AH_WAIT_TIMEOUT_US,
            eeprom_timeout_us: AH_WAIT_TIMEOUT_US,
        }
    }
}

/// Everything F1 proves about the card, in one [`fmt::Display`] line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceInfo {
    /// PCI identity (`168c:0024`, revision).
    pub pci: PciId,
    /// Decoded SREV (`0xc.2` for AR5418).
    pub revision: MacRevision,
    /// `AR_PHY_CHIP_ID` value (diagnostic).
    pub phy_rev: u32,
    /// Validated EEPROM base header and metadata.
    pub eeprom: EepromInfo,
    /// Bands decoded from `opCapFlags`.
    pub bands: Bands,
}

impl DeviceInfo {
    /// 2 GHz channels when supported.
    pub fn channels_2ghz(&self) -> &'static [crate::channel::Channel] {
        self.bands.channels_2ghz()
    }

    /// 5 GHz channels when supported.
    pub fn channels_5ghz(&self) -> &'static [crate::channel::Channel] {
        self.bands.channels_5ghz()
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mac = self.eeprom.base.mac;
        write!(
            f,
            "pci={} rev={} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} \
             srev={} pcie={} phy={:#x} eeprom=def len={} magic={:#06x} \
             swap={} ver={:#x}.{:x} op={:#04x} regd={:#06x},{:#06x} \
             txmask={:#04x} rxmask={:#04x} bands={} ch={}+{}",
            self.pci,
            self.pci.revision,
            mac[0],
            mac[1],
            mac[2],
            mac[3],
            mac[4],
            mac[5],
            self.revision,
            self.revision.pcie,
            self.phy_rev,
            self.eeprom.base.length_bytes,
            self.eeprom.magic,
            self.eeprom.byte_swapped,
            self.eeprom.base.version_major(),
            self.eeprom.base.version_minor(),
            self.eeprom.base.op_cap_flags,
            self.eeprom.base.reg_domain[0],
            self.eeprom.base.reg_domain[1],
            self.eeprom.base.tx_mask,
            self.eeprom.base.rx_mask,
            self.bands,
            self.channels_2ghz().len(),
            self.channels_5ghz().len(),
        )
    }
}

/// Run the full F1 probe. See module docs for the exact sequence and the
/// fail-closed deviations from Linux.
pub fn probe<P: PciConfig, M: Mmio, D: Delay>(
    pci: &mut P,
    mmio: &mut M,
    delay: &mut D,
    config: &ProbeConfig,
) -> Result<DeviceInfo, Error> {
    let pci_id = read_id(pci);
    if !is_target(pci_id.vendor, pci_id.device) {
        return Err(Error::NotAth9k {
            vendor: pci_id.vendor,
            device: pci_id.device,
        });
    }
    enable_memory_and_bus_master(pci);

    let raw_srev = mmio.read32(AR_SREV);
    let revision = decode_srev(raw_srev).ok_or(Error::SrevReadFailed)?;
    if !revision.is_5416() {
        return Err(Error::UnsupportedMac {
            version: revision.version,
            revision: revision.revision,
        });
    }

    power_on_reset(mmio, delay, config.reset_timeout_us)?;
    wake_and_clear_power_save(mmio, delay, config.reset_timeout_us)?;

    let phy_rev = mmio.read32(AR_PHY_CHIP_ID);

    let eeprom = read_def_eeprom(mmio, delay, config.eeprom_timeout_us)?;
    if !eeprom.base.mac_is_valid() {
        return Err(Error::BadMac {
            mac: eeprom.base.mac,
        });
    }
    let bands = Bands::from_op_cap_flags(eeprom.base.op_cap_flags);
    if !bands.any() {
        return Err(Error::NoBands {
            op_cap_flags: eeprom.base.op_cap_flags,
        });
    }

    Ok(DeviceInfo {
        pci: pci_id,
        revision,
        phy_rev,
        eeprom,
        bands,
    })
}

/// AR5416 power-on reset: force-wake, release `AR_RTC_RESET`, wait for
/// `AR_RTC_STATUS_ON`, then a warm MAC reset (`hw.c:1455-1445` for the
/// non-9100/non-9300 path).
fn power_on_reset<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    timeout_us: u32,
) -> Result<(), Error> {
    mmio.write32(
        AR_RTC_FORCE_WAKE,
        AR_RTC_FORCE_WAKE_EN | AR_RTC_FORCE_WAKE_ON_INT,
    );
    mmio.write32(AR_RC, AR_RC_AHB);
    mmio.write32(AR_RTC_RESET, 0);
    delay.delay_us(2);
    mmio.write32(AR_RC, 0);
    mmio.write32(AR_RTC_RESET, AR_RTC_RESET_EN);
    if !wait_reg(
        mmio,
        delay,
        AR_RTC_STATUS,
        AR_RTC_STATUS_M,
        AR_RTC_STATUS_ON,
        timeout_us,
    ) {
        return Err(Error::ResetTimeout {
            stage: ResetStage::RtcWake,
        });
    }

    // Warm MAC reset (`ath9k_hw_set_reset`, `hw.c:1361-1443`).
    mmio.write32(
        AR_RTC_FORCE_WAKE,
        AR_RTC_FORCE_WAKE_EN | AR_RTC_FORCE_WAKE_ON_INT,
    );
    let intr = mmio.read32(AR_INTR_SYNC_CAUSE)
        & (AR_INTR_SYNC_LOCAL_TIMEOUT | AR_INTR_SYNC_RADM_CPL_TIMEOUT);
    if intr != 0 {
        mmio.write32(AR_INTR_SYNC_ENABLE, 0);
        mmio.write32(AR_RC, AR_RC_HOSTIF | AR_RC_AHB);
    } else {
        mmio.write32(AR_RC, AR_RC_AHB);
    }
    mmio.write32(AR_RTC_RC, AR_RTC_RC_MAC_WARM);
    delay.delay_us(100);
    mmio.write32(AR_RTC_RC, 0);
    if !wait_reg(mmio, delay, AR_RTC_RC, AR_RTC_RC_M, 0, timeout_us) {
        return Err(Error::ResetTimeout {
            stage: ResetStage::MacReset,
        });
    }
    mmio.write32(AR_RC, 0);
    Ok(())
}

/// Finish wake-up: wait for `AR_RTC_STATUS_ON` (bounded by `POWER_UP_TIME`),
/// then clear `AR_STA_ID1_PWR_SAV` (`hw.c:2187-2226`).
fn wake_and_clear_power_save<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    _timeout_us: u32,
) -> Result<(), Error> {
    mmio.write32(AR_RTC_FORCE_WAKE, AR_RTC_FORCE_WAKE_EN);
    delay.delay_us(50);

    let mut waited = 0u32;
    loop {
        let status = mmio.read32(AR_RTC_STATUS) & AR_RTC_STATUS_M;
        if status == AR_RTC_STATUS_ON {
            break;
        }
        if waited >= POWER_UP_TIME_US {
            return Err(Error::ResetTimeout {
                stage: ResetStage::RtcWake,
            });
        }
        delay.delay_us(50);
        waited = waited.saturating_add(50);
        mmio.write32(AR_RTC_FORCE_WAKE, AR_RTC_FORCE_WAKE_EN);
    }

    let sta_id1 = mmio.read32(AR_STA_ID1);
    mmio.write32(AR_STA_ID1, sta_id1 & !AR_STA_ID1_PWR_SAV);
    Ok(())
}

/// Bounded register poll: `value & mask == want` within `timeout_us`, 10 µs
/// steps (`ath9k_hw_wait`, `hw.c:76-94`).
fn wait_reg<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    reg: u32,
    mask: u32,
    want: u32,
    timeout_us: u32,
) -> bool {
    let mut waited = 0u32;
    loop {
        if mmio.read32(reg) & mask == want {
            return true;
        }
        if waited >= timeout_us {
            return false;
        }
        delay.delay_us(AH_TIME_QUANTUM_US);
        waited = waited.saturating_add(AH_TIME_QUANTUM_US);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{MockDelay, MockMmio, MockPci, sample_eeprom};
    use std::format;
    use std::vec::Vec;

    fn run(mmiomock: MockMmio, config: ProbeConfig) -> Result<DeviceInfo, Error> {
        let mut pci = MockPci::ar5418();
        let mut mmio = mmiomock;
        let mut delay = MockDelay::default();
        probe(&mut pci, &mut mmio, &mut delay, &config)
    }

    #[test]
    fn probe_succeeds_and_formats_one_log_line() {
        let info = run(
            MockMmio::with_eeprom(sample_eeprom()),
            ProbeConfig::default(),
        )
        .expect("probe should succeed on the mock AR5418");
        assert_eq!(info.pci.vendor, 0x168C);
        assert_eq!(info.pci.device, 0x0024);
        assert_eq!(info.pci.revision, 3);
        assert_eq!(info.revision.version, 0xC);
        assert_eq!(info.revision.revision, 2);
        assert!(info.revision.pcie);
        assert_eq!(info.phy_rev, 0x80);
        assert_eq!(info.eeprom.base.mac, [0x00, 0x19, 0xE3, 0x39, 0xF4, 0xA2]);
        assert!(info.bands.has_2ghz && info.bands.has_5ghz);
        assert_eq!(info.channels_2ghz().len(), 14);
        assert_eq!(info.channels_5ghz().len(), 24);

        let line = format!("{info}");
        assert_eq!(
            line,
            "pci=168c:0024 rev=3 mac=00:19:e3:39:f4:a2 srev=0xc.2 pcie=true \
             phy=0x80 eeprom=def len=64 magic=0xa55a swap=false ver=0xe.13 \
             op=0x03 regd=0x0010,0x0000 txmask=0x07 rxmask=0x07 bands=2g+5g \
             ch=14+24"
        );
    }

    #[test]
    fn probe_enables_memory_and_bus_master() {
        let mut pci = MockPci::ar5418();
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        let mut delay = MockDelay::default();
        probe(&mut pci, &mut mmio, &mut delay, &ProbeConfig::default()).unwrap();
        assert_eq!(
            pci.command & (0x2 | 0x4),
            0x2 | 0x4,
            "memory space + bus master must be enabled"
        );
    }

    #[test]
    fn rejects_wrong_pci_id() {
        let mut pci = MockPci::ar5418();
        pci.id = 0x0024_8086; // Intel, not Atheros
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        let mut delay = MockDelay::default();
        assert_eq!(
            probe(&mut pci, &mut mmio, &mut delay, &ProbeConfig::default()),
            Err(Error::NotAth9k {
                vendor: 0x8086,
                device: 0x0024
            })
        );
    }

    #[test]
    fn rejects_srev_all_ones() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.srev = u32::MAX;
        assert_eq!(
            run(mmio, ProbeConfig::default()),
            Err(Error::SrevReadFailed)
        );
    }

    #[test]
    fn rejects_non_5416_revision() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.srev = 0x0000_0080; // AR9280-style
        assert_eq!(
            run(mmio, ProbeConfig::default()),
            Err(Error::UnsupportedMac {
                version: 0x8,
                revision: 0
            })
        );
    }

    #[test]
    fn reset_timeout_is_bounded() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.rtc_wake_ok = false;
        let mut delay = MockDelay::default();
        let mut pci = MockPci::ar5418();
        let config = ProbeConfig {
            reset_timeout_us: 50,
            eeprom_timeout_us: 1000,
        };
        let err = probe(&mut pci, &mut mmio, &mut delay, &config).unwrap_err();
        assert_eq!(
            err,
            Error::ResetTimeout {
                stage: ResetStage::RtcWake
            }
        );
        assert!(delay.total_us >= 50);
    }

    #[test]
    fn reports_absent_eeprom() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.absent = true;
        assert_eq!(
            run(mmio, ProbeConfig::default()),
            Err(Error::EepromAbsent { offset: 0 })
        );
    }

    #[test]
    fn rejects_invalid_mac() {
        let mut words = sample_eeprom();
        let base = 0x100usize;
        words[base + 6] = 0;
        words[base + 7] = 0;
        words[base + 8] = 0;
        // Recompute checksum after clearing the MAC words (skip the checksum word).
        let mut sum = 0u16;
        for (i, word) in words[base..base + 32].iter().enumerate() {
            if i != 1 {
                sum ^= *word;
            }
        }
        words[base + 1] = sum ^ 0xFFFF;
        let mmio = MockMmio::with_eeprom(words);
        assert_eq!(
            run(mmio, ProbeConfig::default()),
            Err(Error::BadMac {
                mac: [0, 0, 0, 0, 0, 0]
            })
        );
    }

    #[test]
    fn rejects_no_bands() {
        let mut words = sample_eeprom();
        let base = 0x100usize;
        words[base + 3] = 0; // clear opCapFlags
        let mut sum = 0u16;
        for (i, word) in words[base..base + 32].iter().enumerate() {
            if i != 1 {
                sum ^= *word;
            }
        }
        words[base + 1] = sum ^ 0xFFFF;
        let mmio = MockMmio::with_eeprom(words);
        assert_eq!(
            run(mmio, ProbeConfig::default()),
            Err(Error::NoBands { op_cap_flags: 0 })
        );
    }

    #[test]
    fn byte_swapped_eeprom_still_probes() {
        let words: Vec<u16> = sample_eeprom().iter().map(|w| w.swap_bytes()).collect();
        let info = run(MockMmio::with_eeprom(words), ProbeConfig::default()).unwrap();
        assert!(info.eeprom.byte_swapped);
        assert_eq!(info.eeprom.base.mac, [0x00, 0x19, 0xE3, 0x39, 0xF4, 0xA2]);
    }
}
