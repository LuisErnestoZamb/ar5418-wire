//! Channel tables and band capability decoding.
//!
//! The static tables mirror Linux's `ath9k_2ghz_chantable` /
//! `ath9k_5ghz_chantable` (`common-init.c:39-107`): the `hw_index` values are
//! the driver's channel indices (`hw_value`), `ieee` the 802.11 channel number
//! and `freq_mhz` the centre frequency. Band capability comes from the
//! EEPROM `opCapFlags` bits ([`crate::eeprom::OPFLAGS_11G`],
//! [`crate::eeprom::OPFLAGS_11A`], `hw.c:2507-2523`).

use core::fmt;

use crate::eeprom::{OPFLAGS_11A, OPFLAGS_11G};

/// Radio band of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// 2.4 GHz (802.11b/g).
    Ghz2,
    /// 5 GHz (802.11a).
    Ghz5,
}

/// One supported channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel {
    pub band: Band,
    /// 802.11 channel number (1..=14, 36..=165).
    pub ieee: u8,
    /// Centre frequency in MHz.
    pub freq_mhz: u16,
    /// Driver channel index (`hw_value` in `common-init.c`).
    pub hw_index: u8,
}

/// 2 GHz channels 1..=14 (`common-init.c:39-57`).
pub static CHANNELS_2GHZ: [Channel; 14] = [
    Channel {
        band: Band::Ghz2,
        ieee: 1,
        freq_mhz: 2412,
        hw_index: 0,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 2,
        freq_mhz: 2417,
        hw_index: 1,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 3,
        freq_mhz: 2422,
        hw_index: 2,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 4,
        freq_mhz: 2427,
        hw_index: 3,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 5,
        freq_mhz: 2432,
        hw_index: 4,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 6,
        freq_mhz: 2437,
        hw_index: 5,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 7,
        freq_mhz: 2442,
        hw_index: 6,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 8,
        freq_mhz: 2447,
        hw_index: 7,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 9,
        freq_mhz: 2452,
        hw_index: 8,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 10,
        freq_mhz: 2457,
        hw_index: 9,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 11,
        freq_mhz: 2462,
        hw_index: 10,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 12,
        freq_mhz: 2467,
        hw_index: 11,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 13,
        freq_mhz: 2472,
        hw_index: 12,
    },
    Channel {
        band: Band::Ghz2,
        ieee: 14,
        freq_mhz: 2484,
        hw_index: 13,
    },
];

/// 5 GHz channels 36..=165 (`common-init.c:60-107`).
pub static CHANNELS_5GHZ: [Channel; 24] = [
    Channel {
        band: Band::Ghz5,
        ieee: 36,
        freq_mhz: 5180,
        hw_index: 14,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 40,
        freq_mhz: 5200,
        hw_index: 15,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 44,
        freq_mhz: 5220,
        hw_index: 16,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 48,
        freq_mhz: 5240,
        hw_index: 17,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 52,
        freq_mhz: 5260,
        hw_index: 18,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 56,
        freq_mhz: 5280,
        hw_index: 19,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 60,
        freq_mhz: 5300,
        hw_index: 20,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 64,
        freq_mhz: 5320,
        hw_index: 21,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 100,
        freq_mhz: 5500,
        hw_index: 22,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 104,
        freq_mhz: 5520,
        hw_index: 23,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 108,
        freq_mhz: 5540,
        hw_index: 24,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 112,
        freq_mhz: 5560,
        hw_index: 25,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 116,
        freq_mhz: 5580,
        hw_index: 26,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 120,
        freq_mhz: 5600,
        hw_index: 27,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 124,
        freq_mhz: 5620,
        hw_index: 28,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 128,
        freq_mhz: 5640,
        hw_index: 29,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 132,
        freq_mhz: 5660,
        hw_index: 30,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 136,
        freq_mhz: 5680,
        hw_index: 31,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 140,
        freq_mhz: 5700,
        hw_index: 32,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 149,
        freq_mhz: 5745,
        hw_index: 33,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 153,
        freq_mhz: 5765,
        hw_index: 34,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 157,
        freq_mhz: 5785,
        hw_index: 35,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 161,
        freq_mhz: 5805,
        hw_index: 36,
    },
    Channel {
        band: Band::Ghz5,
        ieee: 165,
        freq_mhz: 5825,
        hw_index: 37,
    },
];

/// Band capability decoded from EEPROM `opCapFlags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bands {
    pub has_2ghz: bool,
    pub has_5ghz: bool,
}

impl Bands {
    /// Decode the `opCapFlags` bits (`eeprom.h:125-126`).
    pub const fn from_op_cap_flags(flags: u8) -> Self {
        Bands {
            has_2ghz: flags & OPFLAGS_11G != 0,
            has_5ghz: flags & OPFLAGS_11A != 0,
        }
    }

    /// True when at least one band is usable.
    pub const fn any(self) -> bool {
        self.has_2ghz || self.has_5ghz
    }

    /// 2 GHz channels when the card supports them, else an empty slice.
    pub fn channels_2ghz(self) -> &'static [Channel] {
        if self.has_2ghz { &CHANNELS_2GHZ } else { &[] }
    }

    /// 5 GHz channels when the card supports them, else an empty slice.
    pub fn channels_5ghz(self) -> &'static [Channel] {
        if self.has_5ghz { &CHANNELS_5GHZ } else { &[] }
    }
}

impl fmt::Display for Bands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.has_2ghz, self.has_5ghz) {
            (true, true) => f.write_str("2g+5g"),
            (true, false) => f.write_str("2g"),
            (false, true) => f.write_str("5g"),
            (false, false) => f.write_str("none"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_tables_match_linux() {
        assert_eq!(CHANNELS_2GHZ.len(), 14);
        assert_eq!(CHANNELS_5GHZ.len(), 24);
        assert_eq!(CHANNELS_2GHZ[0].ieee, 1);
        assert_eq!(CHANNELS_2GHZ[0].freq_mhz, 2412);
        assert_eq!(CHANNELS_2GHZ[13].ieee, 14);
        assert_eq!(CHANNELS_2GHZ[13].freq_mhz, 2484);
        assert_eq!(CHANNELS_2GHZ[13].hw_index, 13);
        assert_eq!(CHANNELS_5GHZ[0].ieee, 36);
        assert_eq!(CHANNELS_5GHZ[0].freq_mhz, 5180);
        assert_eq!(CHANNELS_5GHZ[0].hw_index, 14);
        assert_eq!(CHANNELS_5GHZ[23].ieee, 165);
        assert_eq!(CHANNELS_5GHZ[23].freq_mhz, 5825);
        assert_eq!(CHANNELS_5GHZ[23].hw_index, 37);
    }

    #[test]
    fn hw_indices_are_dense() {
        for (i, ch) in CHANNELS_2GHZ.iter().enumerate() {
            assert_eq!(ch.hw_index as usize, i);
        }
        for (i, ch) in CHANNELS_5GHZ.iter().enumerate() {
            assert_eq!(ch.hw_index as usize, i + 14);
        }
    }

    #[test]
    fn bands_decode_from_opcapflags() {
        assert_eq!(
            Bands::from_op_cap_flags(0x03),
            Bands {
                has_2ghz: true,
                has_5ghz: true
            }
        );
        assert_eq!(
            Bands::from_op_cap_flags(0x02),
            Bands {
                has_2ghz: true,
                has_5ghz: false
            }
        );
        assert_eq!(
            Bands::from_op_cap_flags(0x01),
            Bands {
                has_2ghz: false,
                has_5ghz: true
            }
        );
        assert!(!Bands::from_op_cap_flags(0x00).any());
    }

    #[test]
    fn bands_display() {
        assert_eq!(std::format!("{}", Bands::from_op_cap_flags(0x03)), "2g+5g");
        assert_eq!(std::format!("{}", Bands::from_op_cap_flags(0x01)), "5g");
        assert_eq!(std::format!("{}", Bands::default()), "none");
    }

    #[test]
    fn band_slices_gate_on_capability() {
        let both = Bands::from_op_cap_flags(0x03);
        assert_eq!(both.channels_2ghz().len(), 14);
        assert_eq!(both.channels_5ghz().len(), 24);

        let only_5g = Bands::from_op_cap_flags(0x01);
        assert!(only_5g.channels_2ghz().is_empty());
        assert_eq!(only_5g.channels_5ghz().len(), 24);
    }
}
