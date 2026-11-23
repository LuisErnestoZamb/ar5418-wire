# AGENTS.md — ath-wire

Atheros AR5418 (`168c:0024`, class `0280`) bring-up for the from-scratch
`reemplazo` kernel. The full project spec is `ath-wire-prompt.txt` (Spanish;
milestones F0–F6 and non-goals live there). This is a **standalone Cargo
workspace** with one crate, `ath-wire/` — it is *not* part of the `reemplazo`
workspace.

## Hard constraints (violations are bugs)

- **CPU is a 2007 Core2 Duo T7400**: baseline x86-64-v1, SSSE3 at most, no
  SSE4/AVX/POPCNT. **Never `-C target-cpu=native`** (the build host is a modern
  Mac). `.cargo/config.toml` already pins `target-cpu=x86-64` for
  `x86_64-unknown-none`; do not override it.
- Crate is `no_std`, **zero dependencies**, `#![forbid(unsafe_code)]`.
  MMIO/PCI/DMA `unsafe` glue belongs in the kernel, not here.
- Every hardware register constant must cite its source
  (`file.c:NNNN` from Linux v6.6 ath9k). No remembered or invented registers.
- All hardware waits are bounded: each poll has a deadline; never a boot-time
  infinite loop. See `Delay` + timeout args in `probe`.
- Fail closed: bad magic/checksum/version/MAC returns `Error`, never a
  fabricated MAC (deviation from Linux's random-MAC fallback, documented in
  `src/lib.rs`).

## Commands

Toolchain is pinned in `rust-toolchain.toml` (1.98.1, auto-installs
`x86_64-unknown-none`). Run before every commit, in this order:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check -p ath-wire --target x86_64-unknown-none   # proves no_std
```

Single test: `cargo test -p ath-wire <test_name_substring>`.

## Architecture

- `ath-wire/src/` is **pure logic only** (host-tested): register constants,
  SREV decode, EEPROM parse/checksum, channel tables, BAR math, bounded
  `probe` state machine. I/O is injected through `platform::{Mmio, PciConfig,
  Delay}`.
- Status: F0 + F1 (`probe` → `DeviceInfo`) implemented, 36 host tests green.
  F2+ (RX rings, TX, association, WPA2, `net::Nic` bridge) are **not
  implemented yet**. Per spec: abort and document rather than ship half a
  phase. Design notes and risks: `DESIGN.md`.
- The kernel (`reemplazo/kernel/src/…`) will implement those traits over its
  PCI/MMIO/timer code and own all `unsafe`. Integration is a path dependency:
  `ath-wire = { path = "../ar5418-wire/ath-wire" }`, then `probe(...)` and
  log the resulting `DeviceInfo`.
- `DeviceInfo` implements `Display` as one stable log line; F1 acceptance is
  that line from real hardware: `pci=168c:0024 rev=… mac=… srev=0xc.…
  eeprom=… ver=… bands=… ch=…`.

## Register sources

Truth is Linux v6.6 `drivers/net/wireless/ath/ath9k` (pinned URLs and SHA-256
hashes in `src/lib.rs`). Key offsets for AR5418 (AR5416 PCIe, version `0xC`):
SREV `0x4020`, RTC reset `0x7040`/status `0x7044`, EEPROM window
`0x2000 + (word << 2)`, EEPROM status `0x407c`, EEPROM def base header at word
`0x100`, magic `0xa55a` (byte-swapped form `0x5aa5`), def EEPROM size cap
1630 words (`eeprom.h`, `eeprom_def.c`).

## Validation

**QEMU does not emulate the AR5418.** Host tests prove the pure logic; F1+
acceptance is the persistent-log line on the real MacBook2,1
(`REEMPLAZO-LOG-v1` region, `cargo xtask readlog` / `provisionlog` in the
kernel repo). Network credentials/AP details are not available yet; monitor
mode is the planned diagnostic fallback.

## Non-goals (from spec)

No WPA3/SAE, roaming, power save, AMPDU, AP/hostapd, multi-SSID, or full
mac80211 port. Optional second crate if needed: `wlan-mgmt` (802.11 state +
auth/assoc + WPA2).
