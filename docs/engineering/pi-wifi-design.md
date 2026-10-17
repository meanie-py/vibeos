# Raspberry Pi Wi-Fi: our own CYW43455 driver

Status: design, plus the host-tested protocol layer (2026-10-16). No board run yet.

## Why our own

Windows ships no driver for the Pis' onboard Wi-Fi, and the USB stick's Realtek driver
(`rtwlanu.sys`, which the OptiPlex runs) is amd64-only: the Windows 11 ARM64 image (26300)
carries no Realtek Wi-Fi driver at all; its only USB network drivers are ASIX Ethernet
(`netax88179_178a`, `netax88772`) and RNDIS. A driver we write is one we can ship.

## The hardware, both boards

| | Pi 4 | Pi 5 |
|---|---|---|
| Chip | Cypress/Infineon CYW43455 (BCM4345/6), SDIO, FullMAC | same |
| SDIO host | Arasan SDHCI, ACPI `SDC1` `_HID BCM2847` | BCM2712 `sdio2`, ACPI `SDC0` `_HID 80860F16` (generic SDHCI) |
| SD card slot (not this) | EMMC2 `BRCME88C`, sdhc.sys | `SDC1` `BRCM5D12`, sdhc.sys |
| Power (WL_REG_ON) | firmware GPIO expander, via the mailbox (`RPIQ`) | a GPIO line |
| NVRAM file | `brcmfmac43455-sdio.raspberrypi,4-model-b.txt` | `...,5-model-b.txt` |

The `_HID`s are read from our own boots' `[ACPI] DSDT` lines (Pi 5: run
`20261016-210210-5ptzil5g`; Pi 4: `serial-20261008-194631`). The power lines and whether the
UEFI firmware already raises WL_REG_ON are from the Linux device trees and are **to be
checked on the boards** before anything depends on them.

Everything above the SDIO host is shared: same chip, same firmware image, same protocol.

## Layers

1. **SDIO host** (per board, small): SDHCI registers as `sdhc-rust/src/host.rs` drives them,
   plus SDIO: CMD5 (I/O OCR), CMD3/CMD7, CMD52 (one register), CMD53 (byte and block mode),
   4-bit bus, card interrupt. `host.rs` is SD-memory only; its register layer should become a
   shared module rather than a second copy.
2. **Chip bring-up**: the SDIO backplane window (function 1, `SBADDR`), chip and core ids,
   ALP/HT clock requests, halt the ARM core, write firmware and NVRAM into RAM, release the
   core, wait for the firmware's ready mailbox; then load the CLM blob (`clmload`).
   `if_bwfm_sdio.c` `bwfm_sdio_preinit` / `load_microcode` / `clkctl` / `htclk`.
3. **SDPCM framing** (`proto.rs`, host-tested): hardware header (length, ~length), software
   header (sequence, channel, next length, data offset, flow control, max sequence), the
   credit window, glom (several frames in one read).
4. **BCDC** (`proto.rs`, host-tested): the control channel's commands (ioctl number, length,
   GET/SET, id, status) and iovars (`name\0value`); the 4-byte data header in front of every
   Ethernet frame; events (`0x886C` frames carrying `bwfm_event_msg`).
5. **NDIS 6 miniport**: plain 802.3 to netstack over ndis.sys's NDIS 6 core, as `rtux64w10`
   is. No WDI: the chip is FullMAC, so scan, association and roaming are firmware commands.
   Connect: `escan`, then `wsec`/`wpa_auth`/`join`. WPA2 runs on the host with `sup_wpa 0`,
   as OpenBSD does: EAPOL frames arrive as data and `ndis-rust/src/ndis/wpa.rs`'s
   `Supplicant`, which already runs the Realtek's handshake, answers them; keys go down with
   `WLC_SET_KEY`. Configuration is the same `\SystemRoot\bin\wifi.txt`.

## Reference

OpenBSD's `bwfm` (`sys/dev/ic/bwfm.c`, `sys/dev/sdmmc/if_bwfm_sdio.c`, `bwfmreg.h`), ISC
licensed (Broadcom 2010-2016, Patrick Wildt 2016-2017): ported code keeps that notice.
Linux's `brcmfmac` is GPL: read it to answer a question, never copy from it.

## Firmware files

`cyfmac43455-sdio.bin`, `cyfmac43455-sdio.clm_blob` and the board's NVRAM text, from the
Raspberry Pi's `firmware-brcm80211` package (on the Pi 5 stager: `/lib/firmware/cypress`,
`/lib/firmware/brcm`). Staged like the vendor drivers, from `~/vibeos-vendor-drivers/`, never
committed. Whether VibeOS may ship them (Cypress's redistribution licence) is the user's
decision, as with any vendor licence.

## Order of work

1. Protocol layer, host-tested (done: `proto.rs`, `tools/test_bwfm_proto.py`).
2. On the Pi 5 stager's Linux: confirm the WL_REG_ON line and the `sdio2` registers against
   the ACPI resources, and copy the firmware files.
3. SDIO host: CMD5 and CMD52 against the chip, read its CIS and chip id. First board milestone.
4. Firmware download and the ready mailbox; `ver` over BCDC.
5. Scan, join, WPA2, link up, netstack traffic. Then the Pi 4 (host and power differ).
