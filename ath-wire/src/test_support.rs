//! Host-test mocks for the platform traits and a synthetic def-EEPROM image.

use std::vec;
use std::vec::Vec;

use crate::eeprom::BASE_HEADER_WORDS;
use crate::platform::{Delay, Mmio, PciConfig};
use crate::regs::{
    AR_EEPROM_DEF_START, AR_EEPROM_MAGIC, AR_EEPROM_OFFSET, AR_EEPROM_S, AR_EEPROM_STATUS_DATA,
    AR_EEPROM_STATUS_DATA_ABSENT_ACCESS, AR_EEPROM_STATUS_DATA_BUSY, AR_INTR_SYNC_CAUSE,
    AR_PHY_CHIP_ID, AR_RC, AR_RTC_RC, AR_RTC_RESET, AR_RTC_STATUS, AR_SREV, AR_STA_ID1,
};

/// Delay sink that only records elapsed microseconds.
#[derive(Debug, Default, Clone, Copy)]
pub struct MockDelay {
    pub total_us: u32,
}

impl Delay for MockDelay {
    fn delay_us(&mut self, micros: u32) {
        self.total_us = self.total_us.saturating_add(micros);
    }
}

/// Minimal PCI config space for a single function.
#[derive(Debug, Clone, Copy)]
pub struct MockPci {
    /// Dword at config offset 0: `vendor | device << 16`.
    pub id: u32,
    /// Dword at config offset 8: class code in bits 31:8, revision in 7:0.
    pub reg08: u32,
    /// Command register (offset 4).
    pub command: u16,
}

impl MockPci {
    /// An AR5418 function: `168c:0024`, class `0280`, revision 3.
    pub fn ar5418() -> Self {
        MockPci {
            id: 0x0024_168C,
            reg08: 0x0280_0003,
            command: 0,
        }
    }
}

impl PciConfig for MockPci {
    fn read_u16(&mut self, offset: u8) -> u16 {
        match offset {
            0x04 => self.command,
            _ => 0,
        }
    }

    fn read_u32(&mut self, offset: u8) -> u32 {
        match offset {
            0x00 => self.id,
            0x08 => self.reg08,
            _ => 0,
        }
    }

    fn write_u16(&mut self, offset: u8, value: u16) {
        if offset == 0x04 {
            self.command = value;
        }
    }
}

/// MMIO mock: SREV, RTC reset state, and a synthetic EEPROM window.
#[derive(Debug, Clone)]
pub struct MockMmio {
    pub srev: u32,
    pub phy_rev: u32,
    pub rtc_reset: bool,
    /// When false, `AR_RTC_STATUS` never reports ON (reset-timeout test).
    pub rtc_wake_ok: bool,
    pub rtc_rc: u32,
    pub eeprom: Vec<u16>,
    pub status: u32,
    pub absent: bool,
    pub busy_forever: bool,
}

impl MockMmio {
    pub fn with_eeprom(eeprom: Vec<u16>) -> Self {
        MockMmio {
            srev: 0x0000_00C2, // AR5418: version 0xC, revision 2
            phy_rev: 0x80,
            rtc_reset: false,
            rtc_wake_ok: true,
            rtc_rc: 0,
            eeprom,
            status: 0,
            absent: false,
            busy_forever: false,
        }
    }
}

impl Mmio for MockMmio {
    fn read32(&mut self, offset: u32) -> u32 {
        match offset {
            AR_SREV => self.srev,
            AR_PHY_CHIP_ID => self.phy_rev,
            AR_RTC_STATUS => {
                if self.rtc_reset && self.rtc_wake_ok {
                    0x2 // AR_RTC_STATUS_ON
                } else {
                    0
                }
            }
            AR_RTC_RC => self.rtc_rc,
            AR_INTR_SYNC_CAUSE => 0,
            AR_STA_ID1 => 0,
            AR_RC => 0,
            AR_EEPROM_STATUS_DATA => self.status,
            _ if offset >= AR_EEPROM_OFFSET => {
                if self.busy_forever {
                    self.status = AR_EEPROM_STATUS_DATA_BUSY;
                    return self.status;
                }
                if self.absent {
                    self.status = AR_EEPROM_STATUS_DATA_ABSENT_ACCESS;
                    return self.status;
                }
                let word = (offset - AR_EEPROM_OFFSET) >> AR_EEPROM_S;
                let value = self.eeprom.get(word as usize).copied().unwrap_or(0xFFFF);
                self.status = value as u32;
                self.status
            }
            _ => 0,
        }
    }

    fn write32(&mut self, offset: u32, value: u32) {
        match offset {
            AR_RTC_RESET => self.rtc_reset = value & 1 != 0,
            AR_RTC_RC => self.rtc_rc = value,
            _ => {}
        }
    }
}

/// Build a valid def-EEPROM image: magic at word 0, base header at `0x100`
/// with length 64 bytes (32 checksummed words), a valid MAC and a checksum
/// that XORs to `0xffff`.
pub fn sample_eeprom() -> Vec<u16> {
    let mut words = vec![0u16; 0xAE0 + 1];
    words[0] = AR_EEPROM_MAGIC;

    let base = AR_EEPROM_DEF_START as usize;
    words[base] = 64; // length: base header only
    words[base + 2] = 0xE013; // version 0xE.19
    words[base + 3] = 0x0003; // opCapFlags: 11A | 11G
    words[base + 4] = 0x0010; // regDmn[0]
    words[base + 5] = 0x0000; // regDmn[1]
    words[base + 6] = u16::from_le_bytes([0x00, 0x19]);
    words[base + 7] = u16::from_le_bytes([0xE3, 0x39]);
    words[base + 8] = u16::from_le_bytes([0xF4, 0xA2]);
    words[base + 9] = u16::from_le_bytes([0x07, 0x07]); // rx/tx masks

    let mut sum = 0u16;
    for word in words[base..base + BASE_HEADER_WORDS].iter() {
        sum ^= *word;
    }
    words[base + 1] = sum ^ 0xFFFF;
    words
}
