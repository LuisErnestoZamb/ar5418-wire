//! PCI identity, command register and BAR decoding for the AR5418.
//!
//! The AR5418 is the PCIe member of the AR5416 family: vendor `0x168c`,
//! device `0x0024` (`pci.c:28`), class `0280` (network / other). Its PCI
//! sibling is `0x0023`. The kernel owns the actual config-space access; this
//! module only defines the offsets, the decode rules and the pure BAR math.

use crate::platform::PciConfig;

/// Atheros Communications vendor ID (`PCI_VENDOR_ID_ATHEROS`).
pub const ATHEROS_VENDOR: u16 = 0x168C;
/// `AR5416_DEVID_PCIE` — the AR5418 this crate drives (`pci.c:28`).
pub const AR5416_DEVID_PCIE: u16 = 0x0024;
/// `AR5416_DEVID_PCI` — the non-PCIe AR5416 sibling (`pci.c:27`).
pub const AR5416_DEVID_PCI: u16 = 0x0023;
/// Alias that names the target card explicitly.
pub const AR5418_DEVICE: u16 = AR5416_DEVID_PCIE;

/// PCI class code for network controllers (class byte `0x02`).
pub const PCI_CLASS_NETWORK: u8 = 0x02;
/// PCI subclass "other network controller" (`0x80`); AR5418 reports `0280`.
pub const PCI_SUBCLASS_OTHER_NETWORK: u8 = 0x80;

/// PCI vendor/device register (offset `0x00`).
pub const PCI_VENDOR_ID: u8 = 0x00;
/// PCI command register (offset `0x04`).
pub const PCI_COMMAND: u8 = 0x04;
/// Command bit 1: decode memory space (needed before touching the BAR).
pub const PCI_COMMAND_MEMORY_SPACE: u16 = 1 << 1;
/// Command bit 2: bus master (needed for DMA in F2+; harmless in F1).
pub const PCI_COMMAND_BUS_MASTER: u16 = 1 << 2;
/// Revision ID / class-code register (offset `0x08`).
pub const PCI_REVISION_ID: u8 = 0x08;
/// BAR0 (offset `0x10`).
pub const PCI_BAR0: u8 = 0x10;
/// BAR1 (offset `0x14`), the upper half of a 64-bit BAR0.
pub const PCI_BAR1: u8 = 0x14;

/// True only for the card this crate targets.
pub const fn is_target(vendor: u16, device: u16) -> bool {
    vendor == ATHEROS_VENDOR && device == AR5418_DEVICE
}

/// Decoded identity of the PCI function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciId {
    pub vendor: u16,
    pub device: u16,
    /// PCI revision ID (config byte `0x08`), distinct from the SREV revision.
    pub revision: u8,
}

impl core::fmt::Display for PciId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vendor, self.device)
    }
}

/// Read and decode the PCI identity. Revision is the low byte of config
/// offset `0x08`; the class code is the upper 24 bits of the same dword.
pub fn read_id<P: PciConfig>(pci: &mut P) -> PciId {
    let id = pci.read_u32(PCI_VENDOR_ID);
    let reg08 = pci.read_u32(PCI_REVISION_ID);
    PciId {
        vendor: id as u16,
        device: (id >> 16) as u16,
        revision: revision_id(reg08),
    }
}

/// Revision ID from the dword at config offset `0x08`.
pub const fn revision_id(reg08: u32) -> u8 {
    reg08 as u8
}

/// 24-bit class code from the dword at config offset `0x08`.
pub const fn class_code(reg08: u32) -> u32 {
    reg08 >> 8
}

/// True for class `02xxxx` (network controller), without requiring the
/// `0280` subclass — some AR5416 variants report a different subclass.
pub const fn is_network_class(class_code: u32) -> bool {
    (class_code >> 16) as u8 == PCI_CLASS_NETWORK
}

/// Enable memory-space decoding and bus mastering.
///
/// Reads the command register first so the host bridge's other bits are
/// preserved. F1 needs memory space; bus master is set now because F2's RX
/// rings will need it and setting it early matches `pci.c:948` (`pci_set_master`).
pub fn enable_memory_and_bus_master<P: PciConfig>(pci: &mut P) {
    let command = pci.read_u16(PCI_COMMAND);
    pci.write_u16(
        PCI_COMMAND,
        command | PCI_COMMAND_MEMORY_SPACE | PCI_COMMAND_BUS_MASTER,
    );
}

/// Decode a *memory* BAR address from BAR0 (+ BAR1 when the BAR is 64-bit).
///
/// PCI BAR low bits are flags: bit 0 = I/O space, bits 2:1 = type (`0b10`
/// means 64-bit), bit 3 = prefetchable. Linux masks them with
/// `PCI_BASE_ADDRESS_MEM_MASK = ~0x0f`. Returns `None` for I/O-space or
/// unassigned BARs.
pub const fn mem_bar_address(bar0: u32, bar1: u32) -> Option<u64> {
    if bar0 & 1 != 0 {
        return None; // I/O space
    }
    let is_64bit = (bar0 >> 1) & 0b11 == 0b10;
    let low = (bar0 & !0xF) as u64;
    let addr = if is_64bit {
        low | ((bar1 as u64) << 32)
    } else {
        low
    };
    if addr == 0 { None } else { Some(addr) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_ids_are_exact() {
        assert!(is_target(0x168C, 0x0024));
        assert!(!is_target(0x168C, 0x0023));
        assert!(!is_target(0x8086, 0x0024));
        assert!(!is_target(0xFFFF, 0xFFFF));
    }

    #[test]
    fn class_and_revision_decode() {
        // class 0280, revision 3, little-endian dword.
        let reg08 = 0x0280_0003;
        assert_eq!(revision_id(reg08), 3);
        assert_eq!(class_code(reg08), 0x0002_8000);
        assert!(is_network_class(class_code(reg08)));

        // class 0200 (Ethernet) is still a network controller.
        assert!(is_network_class(0x0002_0000));
        // class 0600 (bridge) is not.
        assert!(!is_network_class(0x0006_0000));
    }

    #[test]
    fn bar_decoding_32bit() {
        // Memory BAR, 32-bit, non-prefetchable, address d0200000.
        assert_eq!(mem_bar_address(0xD020_0000, 0), Some(0xD020_0000));
        // Type/reserved flag bits are masked off (bit 0 stays clear: memory).
        assert_eq!(mem_bar_address(0xD020_0002, 0), Some(0xD020_0000));
        // Prefetchable (bit 3) is still the same address.
        assert_eq!(mem_bar_address(0xD020_0008, 0), Some(0xD020_0000));
    }

    #[test]
    fn bar_decoding_64bit() {
        // bits 2:1 = 0b10 -> 64-bit; upper half in BAR1.
        assert_eq!(
            mem_bar_address(0x0000_0004, 0x0000_0001),
            Some(0x1_0000_0000)
        );
        assert_eq!(mem_bar_address(0xD020_0004, 0x0000_0000), Some(0xD020_0000));
    }

    #[test]
    fn bar_rejects_io_and_unassigned() {
        assert_eq!(mem_bar_address(0x0000_1001, 0), None); // I/O space
        assert_eq!(mem_bar_address(0x0000_0000, 0), None); // unassigned
        assert_eq!(mem_bar_address(0x0000_0004, 0), None); // 64-bit, zero
    }

    #[test]
    fn id_display_is_pci_form() {
        let id = PciId {
            vendor: 0x168C,
            device: 0x0024,
            revision: 3,
        };
        assert_eq!(std::format!("{id}"), "168c:0024");
    }
}
