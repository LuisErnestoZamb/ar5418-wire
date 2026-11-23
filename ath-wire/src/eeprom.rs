//! AR5416 **def** EEPROM reading, validation and base-header parsing.
//!
//! The AR5418 uses the def layout selected by `ath9k_hw_eeprom_init`
//! (`eeprom.c:676-680`): magic at word 0, struct starting at word `0x100`
//! (`eeprom_def.c:101`), XOR checksum over `length/2` words clamped to 1630
//! (`eeprom_def.c:281`). The base header is the first 64 bytes = 32 words
//! (`eeprom.h:283-313`).

use crate::error::Error;
use crate::platform::{Delay, Mmio};
use crate::regs::{
    AH_TIME_QUANTUM_US, AR_EEPROM_DEF_START, AR_EEPROM_MAGIC, AR_EEPROM_MAGIC_OFFSET,
    AR_EEPROM_MAGIC_SWAPPED, AR_EEPROM_OFFSET, AR_EEPROM_S, AR_EEPROM_STATUS_DATA,
    AR_EEPROM_STATUS_DATA_ABSENT_ACCESS, AR_EEPROM_STATUS_DATA_BUSY,
    AR_EEPROM_STATUS_DATA_PROT_ACCESS, AR_EEPROM_STATUS_DATA_VAL, DEF_EEPROM_WORDS_MAX,
};

/// `AR5416_EEP_VER` (`eeprom.h:134`) — required def-EEPROM major version.
pub const AR5416_EEP_VER: u16 = 0xE;
/// `AR5416_EEP_VER_MAJOR_SHIFT` (`eeprom.h:135`).
pub const AR5416_EEP_VER_MAJOR_SHIFT: u16 = 12;
/// `AR5416_EEP_VER_MINOR_MASK` (`eeprom.h:137`).
pub const AR5416_EEP_VER_MINOR_MASK: u16 = 0x0FFF;
/// `AR5416_EEP_NO_BACK_VER` (`eeprom.h:132`) — minimum minor revision.
pub const AR5416_EEP_NO_BACK_VER: u16 = 0x1;
/// `AR5416_OPFLAGS_11A` (`eeprom.h:125`) — 5 GHz capable.
pub const OPFLAGS_11A: u8 = 0x01;
/// `AR5416_OPFLAGS_11G` (`eeprom.h:126`) — 2 GHz capable.
pub const OPFLAGS_11G: u8 = 0x02;
/// `AR5416_OPFLAGS_N_5G_HT20` (`eeprom.h:129`).
pub const OPFLAGS_N_5G_HT20: u8 = 0x10;
/// `AR5416_OPFLAGS_N_2G_HT20` (`eeprom.h:130`).
pub const OPFLAGS_N_2G_HT20: u8 = 0x20;
/// `AR5416_EEPMISC_BIG_ENDIAN` (`eeprom.h:177`) — base-header misc bit.
pub const EEPMISC_BIG_ENDIAN: u8 = 0x01;

/// Base header word count (`sizeof(base_eep_header) == 64`, `eeprom.h:283`).
pub const BASE_HEADER_WORDS: usize = 32;
/// Base header size in bytes.
pub const BASE_HEADER_BYTES: u16 = BASE_HEADER_WORDS as u16 * 2;

/// Parsed AR5416 def-EEPROM base header (64 bytes, `eeprom.h:283-313`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EepromBaseHeader {
    /// Header length in bytes (`length`).
    pub length_bytes: u16,
    /// Stored XOR checksum (must make the full XOR `0xffff`).
    pub checksum: u16,
    /// Packed version: major in bits 15:12, minor in bits 11:0.
    pub version: u16,
    /// `opCapFlags` — band capability bits ([`OPFLAGS_11A`], [`OPFLAGS_11G`]).
    pub op_cap_flags: u8,
    /// `eepMisc` ([`EEPMISC_BIG_ENDIAN`] is bit 0).
    pub eep_misc: u8,
    /// `regDmn[2]` — regulatory domain words.
    pub reg_domain: [u16; 2],
    /// Station MAC address (6 bytes, EEPROM byte order).
    pub mac: [u8; 6],
    /// `rxMask` — receive chain mask.
    pub rx_mask: u8,
    /// `txMask` — transmit chain mask.
    pub tx_mask: u8,
    /// `rfSilent` — RF silent GPIO config.
    pub rf_silent: u16,
    /// `blueToothOptions` — Bluetooth coexistence options.
    pub bluetooth_options: u16,
    /// `deviceCap` — device capability flags.
    pub device_cap: u16,
    /// `binBuildNumber` — calibration binary build.
    pub bin_build_number: u32,
    /// `deviceType`.
    pub device_type: u8,
    /// `pwr_table_offset` — power-table offset.
    pub pwr_table_offset: u8,
}

impl EepromBaseHeader {
    /// Major version (bits 15:12), `eeprom_def.c:80-85`.
    pub const fn version_major(&self) -> u16 {
        self.version >> AR5416_EEP_VER_MAJOR_SHIFT
    }

    /// Minor version (bits 11:0), `eeprom_def.c:88-92`.
    pub const fn version_minor(&self) -> u16 {
        self.version & AR5416_EEP_VER_MINOR_MASK
    }

    /// True when the base header advertises 2 GHz operation.
    pub const fn has_2ghz(&self) -> bool {
        self.op_cap_flags & OPFLAGS_11G != 0
    }

    /// True when the base header advertises 5 GHz operation.
    pub const fn has_5ghz(&self) -> bool {
        self.op_cap_flags & OPFLAGS_11A != 0
    }

    /// Same validity rule Linux applies (`is_valid_ether_addr`, `hw.c:494-507`):
    /// not all-zero and not multicast (first bit clear).
    pub const fn mac_is_valid(&self) -> bool {
        let mac = self.mac;
        if mac[0] & 1 != 0 {
            return false;
        }
        !(mac[0] == 0 && mac[1] == 0 && mac[2] == 0 && mac[3] == 0 && mac[4] == 0 && mac[5] == 0)
    }
}

/// Result of a successful def-EEPROM read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EepromInfo {
    /// Raw magic word at EEPROM offset 0 (`0xa55a` or `0x5aa5`).
    pub magic: u16,
    /// True when the magic required byte-swapping every word.
    pub byte_swapped: bool,
    /// Parsed base header.
    pub base: EepromBaseHeader,
    /// Number of words covered by the checksum (`length/2`, clamped).
    pub checksum_words: u16,
}

/// Trigger an EEPROM word access by *reading* the EEPROM window
/// (`pci.c:805`, `eeprom.h:68`), wait for the access to finish and return
/// the 16-bit value.
///
/// Bounded: one deadline of `timeout_us`, 10 µs steps (`AH_TIME_QUANTUM`).
pub fn read_word<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    offset: u16,
    timeout_us: u32,
) -> Result<u16, Error> {
    let addr = AR_EEPROM_OFFSET + (u32::from(offset) << AR_EEPROM_S);
    // A read (not a write) of the EEPROM window starts the access;
    // the returned dword is a scratch/status value and is discarded.
    let _ = mmio.read32(addr);
    wait_ready(mmio, delay, offset, timeout_us)?;
    let status = mmio.read32(AR_EEPROM_STATUS_DATA);
    if status & AR_EEPROM_STATUS_DATA_ABSENT_ACCESS != 0 {
        return Err(Error::EepromAbsent { offset });
    }
    Ok((status & AR_EEPROM_STATUS_DATA_VAL) as u16)
}

/// Wait until busy/protect access bits clear (`reg.h:1254,1256`;
/// `pci.c:806-812`).
fn wait_ready<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    offset: u16,
    timeout_us: u32,
) -> Result<(), Error> {
    let mut waited = 0u32;
    loop {
        let status = mmio.read32(AR_EEPROM_STATUS_DATA);
        if status & (AR_EEPROM_STATUS_DATA_BUSY | AR_EEPROM_STATUS_DATA_PROT_ACCESS) == 0 {
            return Ok(());
        }
        if waited >= timeout_us {
            return Err(Error::EepromTimeout { offset });
        }
        delay.delay_us(AH_TIME_QUANTUM_US);
        waited = waited.saturating_add(AH_TIME_QUANTUM_US);
    }
}

/// Read, validate and parse the def-EEPROM base header.
///
/// Fails closed on absent EEPROM, bad magic, bad length, bad checksum or bad
/// version. The MAC itself is not validated here ([`Error::BadMac`] is raised
/// by the probe so the raw bytes can be reported).
pub fn read_def_eeprom<M: Mmio, D: Delay>(
    mmio: &mut M,
    delay: &mut D,
    timeout_us: u32,
) -> Result<EepromInfo, Error> {
    let magic = read_word(mmio, delay, AR_EEPROM_MAGIC_OFFSET, timeout_us)?;
    let byte_swapped = match magic {
        AR_EEPROM_MAGIC => false,
        AR_EEPROM_MAGIC_SWAPPED => true,
        _ => return Err(Error::BadEepromMagic { magic }),
    };

    let mut words = [0u16; BASE_HEADER_WORDS];
    for (i, word) in words.iter_mut().enumerate() {
        let raw = read_word(mmio, delay, AR_EEPROM_DEF_START + i as u16, timeout_us)?;
        *word = if byte_swapped { raw.swap_bytes() } else { raw };
    }

    let base = parse_def_base_header(&words);
    if base.length_bytes < BASE_HEADER_BYTES || !base.length_bytes.is_multiple_of(2) {
        return Err(Error::BadEepromLength {
            length: base.length_bytes,
        });
    }

    // Checksum covers length/2 words, clamped to SIZE_EEPROM_DEF
    // (eeprom_def.c:278-283).
    let mut count = base.length_bytes / 2;
    if count > DEF_EEPROM_WORDS_MAX {
        count = DEF_EEPROM_WORDS_MAX;
    }

    let mut sum = 0u16;
    for word in words.iter() {
        sum ^= *word;
    }
    for i in BASE_HEADER_WORDS as u16..count {
        let raw = read_word(mmio, delay, AR_EEPROM_DEF_START + i, timeout_us)?;
        sum ^= if byte_swapped { raw.swap_bytes() } else { raw };
    }
    if sum != u16::MAX {
        return Err(Error::BadEepromChecksum { sum });
    }

    if base.version_major() != AR5416_EEP_VER || base.version_minor() < AR5416_EEP_NO_BACK_VER {
        return Err(Error::BadEepromVersion {
            version: base.version,
        });
    }

    Ok(EepromInfo {
        magic,
        byte_swapped,
        base,
        checksum_words: count,
    })
}

/// Parse the 32 base-header words (`eeprom.h:283`). Little-endian words are
/// reassembled into the EEPROM's byte order; the MAC is stored as bytes
/// `macAddr[0..6]` (`eeprom_def.c:346-352`).
pub fn parse_def_base_header(words: &[u16; BASE_HEADER_WORDS]) -> EepromBaseHeader {
    EepromBaseHeader {
        length_bytes: words[0],
        checksum: words[1],
        version: words[2],
        op_cap_flags: (words[3] & 0xFF) as u8,
        eep_misc: (words[3] >> 8) as u8,
        reg_domain: [words[4], words[5]],
        mac: [
            (words[6] & 0xFF) as u8,
            (words[6] >> 8) as u8,
            (words[7] & 0xFF) as u8,
            (words[7] >> 8) as u8,
            (words[8] & 0xFF) as u8,
            (words[8] >> 8) as u8,
        ],
        rx_mask: (words[9] & 0xFF) as u8,
        tx_mask: (words[9] >> 8) as u8,
        rf_silent: words[10],
        bluetooth_options: words[11],
        device_cap: words[12],
        bin_build_number: u32::from(words[13]) | (u32::from(words[14]) << 16),
        device_type: (words[15] & 0xFF) as u8,
        // pwdclkind lives in the high byte of word 15 and is not needed by F1.
        pwr_table_offset: (words[20] >> 8) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{MockDelay, MockMmio, sample_eeprom};
    use std::vec::Vec;

    #[test]
    fn parses_base_header_fields() {
        let mut words = [0u16; BASE_HEADER_WORDS];
        words[0] = 64; // length
        words[2] = (AR5416_EEP_VER << AR5416_EEP_VER_MAJOR_SHIFT) | 0x13;
        words[3] = 0x0003; // opCapFlags 11A|11G
        words[4] = 0x0010; // regDmn[0]
        words[5] = 0x0000; // regDmn[1]
        words[6] = u16::from_le_bytes([0x00, 0x19]);
        words[7] = u16::from_le_bytes([0xE3, 0x39]);
        words[8] = u16::from_le_bytes([0xF4, 0xA2]);
        words[9] = u16::from_le_bytes([0x07, 0x07]); // rx/tx masks
        words[12] = 0x0100; // deviceCap
        words[13] = 0x1234;
        words[14] = 0x5678; // binBuildNumber 0x56781234
        words[20] = u16::from_le_bytes([0x11, 0x25]); // pwrTableOffset 0x25

        let base = parse_def_base_header(&words);
        assert_eq!(base.length_bytes, 64);
        assert_eq!(base.version_major(), AR5416_EEP_VER);
        assert_eq!(base.version_minor(), 0x13);
        assert_eq!(base.op_cap_flags, 0x03);
        assert!(base.has_2ghz());
        assert!(base.has_5ghz());
        assert_eq!(base.reg_domain, [0x0010, 0x0000]);
        assert_eq!(base.mac, [0x00, 0x19, 0xE3, 0x39, 0xF4, 0xA2]);
        assert!(base.mac_is_valid());
        assert_eq!(base.rx_mask, 0x07);
        assert_eq!(base.tx_mask, 0x07);
        assert_eq!(base.device_cap, 0x0100);
        assert_eq!(base.bin_build_number, 0x5678_1234);
        assert_eq!(base.pwr_table_offset, 0x25);
    }

    #[test]
    fn rejects_zero_and_multicast_mac() {
        let mut words = [0u16; BASE_HEADER_WORDS];
        assert!(!parse_def_base_header(&words).mac_is_valid());

        words[6] = u16::from_le_bytes([0x01, 0x00]); // multicast
        let base = parse_def_base_header(&words);
        assert!(!base.mac_is_valid());

        words[6] = u16::from_le_bytes([0x00, 0x19]);
        words[7] = u16::from_le_bytes([0xE3, 0x39]);
        words[8] = u16::from_le_bytes([0xF4, 0xA2]);
        assert!(parse_def_base_header(&words).mac_is_valid());
    }

    #[test]
    fn reads_valid_eeprom() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        let mut delay = MockDelay::default();
        let info = read_def_eeprom(&mut mmio, &mut delay, 1000).unwrap();
        assert_eq!(info.magic, AR_EEPROM_MAGIC);
        assert!(!info.byte_swapped);
        assert_eq!(info.checksum_words, 32);
        assert_eq!(info.base.mac, [0x00, 0x19, 0xE3, 0x39, 0xF4, 0xA2]);
        assert!(info.base.version_major() == AR5416_EEP_VER);
    }

    #[test]
    fn reads_byte_swapped_eeprom() {
        let words: Vec<u16> = sample_eeprom().iter().map(|w| w.swap_bytes()).collect();
        let mut mmio = MockMmio::with_eeprom(words);
        let mut delay = MockDelay::default();
        let info = read_def_eeprom(&mut mmio, &mut delay, 1000).unwrap();
        assert_eq!(info.magic, AR_EEPROM_MAGIC_SWAPPED);
        assert!(info.byte_swapped);
        assert_eq!(info.base.mac, [0x00, 0x19, 0xE3, 0x39, 0xF4, 0xA2]);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut words = sample_eeprom();
        words[0] = 0x1234;
        let mut mmio = MockMmio::with_eeprom(words);
        let mut delay = MockDelay::default();
        assert_eq!(
            read_def_eeprom(&mut mmio, &mut delay, 1000),
            Err(Error::BadEepromMagic { magic: 0x1234 })
        );
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut words = sample_eeprom();
        words[AR_EEPROM_DEF_START as usize + 5] ^= 0x0001;
        let mut mmio = MockMmio::with_eeprom(words);
        let mut delay = MockDelay::default();
        assert!(matches!(
            read_def_eeprom(&mut mmio, &mut delay, 1000),
            Err(Error::BadEepromChecksum { .. })
        ));
    }

    #[test]
    fn rejects_bad_version() {
        let mut words = sample_eeprom();
        let base = AR_EEPROM_DEF_START as usize;
        words[base + 2] = (0xD << AR5416_EEP_VER_MAJOR_SHIFT) | 0x13;
        // Recompute the checksum so only the version is wrong.
        let mut sum = 0u16;
        for (i, word) in words[base..base + BASE_HEADER_WORDS].iter().enumerate() {
            if i != 1 {
                sum ^= *word;
            }
        }
        words[base + 1] = sum ^ 0xFFFF;
        let mut mmio = MockMmio::with_eeprom(words);
        let mut delay = MockDelay::default();
        assert_eq!(
            read_def_eeprom(&mut mmio, &mut delay, 1000),
            Err(Error::BadEepromVersion {
                version: (0xD << AR5416_EEP_VER_MAJOR_SHIFT) | 0x13
            })
        );
    }

    #[test]
    fn rejects_bad_length() {
        let mut words = sample_eeprom();
        words[AR_EEPROM_DEF_START as usize] = 31; // odd and < 64
        let mut mmio = MockMmio::with_eeprom(words);
        let mut delay = MockDelay::default();
        assert_eq!(
            read_def_eeprom(&mut mmio, &mut delay, 1000),
            Err(Error::BadEepromLength { length: 31 })
        );
    }

    #[test]
    fn reports_absent_eeprom() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.absent = true;
        let mut delay = MockDelay::default();
        assert_eq!(
            read_def_eeprom(&mut mmio, &mut delay, 1000),
            Err(Error::EepromAbsent { offset: 0 })
        );
    }

    #[test]
    fn eeprom_wait_is_bounded() {
        let mut mmio = MockMmio::with_eeprom(sample_eeprom());
        mmio.busy_forever = true;
        let mut delay = MockDelay::default();
        assert_eq!(
            read_def_eeprom(&mut mmio, &mut delay, 50),
            Err(Error::EepromTimeout { offset: 0 })
        );
        assert!(delay.total_us >= 50);
    }
}
