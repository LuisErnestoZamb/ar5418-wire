# DESIGN — F1 probe (ath-wire)

Scope: PCI identification → SREV decode → power-on/warm reset → def-EEPROM
read/validate → MAC + bands/channels, all host-tested with mock MMIO.
Reference: Linux v6.6 ath9k (pinned URLs + SHA-256 in `src/lib.rs`).

## What F1 does

1. **PCI** (`pci.rs`): read `168c:0024` + revision from config space, enable
   memory space + bus master (RMW so other command bits survive).
   `mem_bar_address` decodes 32/64-bit memory BARs for the kernel to map.
2. **SREV** (`regs.rs`): read `0x4020`. All-ones → `SrevReadFailed` (BAR not
   decoding). Old nibble layout (version/revision) or new type2 layout
   (14-bit version); only AR5416 versions `0xC`/`0xD` accepted.
3. **Reset** (`probe.rs::power_on_reset`): force-wake, release `AR_RTC_RESET`,
   wait `AR_RTC_STATUS_ON`, then warm MAC reset via `AR_RTC_RC` (mirrors
   `hw.c:1455-1445` for non-9100/non-9300). Every wait has a deadline
   (`ProbeConfig`, default 100 ms, 10 µs steps).
4. **Wake** (`wake_and_clear_power_save`): bounded `POWER_UP_TIME` loop, then
   clear `AR_STA_ID1_PWR_SAV` (`hw.c:2187-2226`).
5. **EEPROM** (`eeprom.rs`): magic at word 0 (`0xa55a` / `0x5aa5` swapped),
   base header 32 words at `0x100`, XOR checksum over `length/2` words
   clamped to 1630, version `0xE.1+`. Accesses are *reads* of
   `0x2000 + (word << 2)`; status at `0x407c`.
6. **Result** (`DeviceInfo`): one `Display` line, e.g.
   `pci=168c:0024 rev=3 mac=00:19:e3:39:f4:a2 srev=0xc.2 pcie=true phy=0x80
   eeprom=def len=64 magic=0xa55a swap=false ver=0xe.13 op=0x03
   regd=0x0010,0x0000 txmask=0x07 rxmask=0x07 bands=2g+5g ch=14+24`

## Deviations from Linux (deliberate, fail-closed)

- Invalid MAC → `Error::BadMac`, never a random MAC (`hw.c:486-511` fallback).
- `ABSENT_ACCESS` status bit → `Error::EepromAbsent` instead of a later
  confusing bad-magic error.
- EEPROM flash/OTP/platform-data paths are not implemented (chip EEPROM only).

## Risks / open items for F2+

- **EEPROM variant**: AR5418 boards may ship OTP/no-EEPROM; F1 must report
  the exact failure (`EepromAbsent`/magic/checksum) on hardware before F2.
- **Calibration**: no RF cal data validated yet; TX can fail silently — F3 has
  its own acceptance (probe response received).
- **PHY revision** (`AR_PHY_CHIP_ID`) is reported, not validated.
- **Regulatory filtering** is not applied: `regDmn` words are logged raw;
  channel restriction by domain is deferred until scan/association.
- **DMA/IRQ**: F1 does no DMA and uses no IRQ; F2 adds the RX ring + `Dma`
  trait with barriers, then bounded poll fallback.

## Verification status

Host: 36 unit tests (`cargo test`), `fmt --check`, `clippy -D warnings`,
`cargo check --target x86_64-unknown-none`. Hardware: **pending** — QEMU does
not emulate AR5418; acceptance is the F1 log line on the MacBook2,1.
