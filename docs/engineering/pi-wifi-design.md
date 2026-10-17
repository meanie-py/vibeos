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
| NVRAM file | `brcmfmac43455-sdio.txt` (the `raspberrypi,*` names link to it) | same |

The `_HID`s are read from our own boots' `[ACPI] DSDT` lines (Pi 5: run
`20261016-210210-5ptzil5g`; Pi 4: `serial-20261008-194631`).

**Pi 5, checked on the board's Linux (2026-10-17):** the chip is `mmc1`, an SDHCI at
`0x1001100000` (DDR50, 4-bit), SDIO vendor `0x02d0` device `0x4345`, three functions, chip id
`0x15264345` at `0x18000000` (BCM4345 rev 6), firmware `7.45.265`. WL_REG_ON is line 28
(`WL_ON`) of the `brcm,brcmstb-gpio` block at `0x107d508500` (lines 30-35 are the SDIO pins),
active high, with a 150 ms start-up delay (`wl-on-reg`'s `startup-delay-us`). Whether the UEFI
firmware leaves it high at handoff is still to be read from that register under VibeOS. The
Pi 4's power line (the firmware's GPIO expander, line 1 in its Linux tree) is unchecked.

Every Pi links the same NVRAM file, `brcmfmac43455-sdio.txt`, so one set of three files
serves both boards.

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

`cyfmac43455-sdio.bin` (609,309 bytes), `cyfmac43455-sdio.clm_blob` (2,676) and
`brcmfmac43455-sdio.txt` (2,074): the chip's own program, its regulatory table, and the board's
calibration. All Pis use these three.

**VibeOS does not ship them** (the user's decision, 2026-10-17). Cypress/Infineon license the
program for redistribution only in object form, for use with their chips, and Raspberry Pi
distributes it under its own agreement with them. So it is never committed or hosted here; the
published images go without Pi Wi-Fi, and a user who wants it builds their own image, which
downloads the files at build time:

    python3 tools/fetch_pi_wifi_firmware.py      # into $VIBEOS_PI_WIFI_FIRMWARE or ~/.cache/vibeos/cyw43455

It takes them from Raspberry Pi's `RPi-Distro/firmware-nonfree` at a pinned commit and checks
each against a pinned SHA-256: those of the files Raspberry Pi OS (`firmware-brcm80211`
1:20241210-1+rpt4) runs on our Pi 5, which the pinned commit's are byte for byte. A mismatch
fails the fetch. The build will stage the directory into the image when the driver exists, and
only when asked to (`VIBEOS_PI_WIFI_FIRMWARE`), so an image built without it is the default.

## Order of work

1. Protocol layer, host-tested (done: `proto.rs`, `tools/test_bwfm_proto.py`).
2. On the Pi 5 stager's Linux: confirm the WL_REG_ON line and the `sdio2` registers, and copy
   the firmware files (done, above). Left: the ACPI `SDC0` resources against `0x1001100000`.
3. SDIO host: CMD5 and CMD52 against the chip, read its CIS and chip id. First board milestone.
4. Firmware download and the ready mailbox; `ver` over BCDC.
5. Scan, join, WPA2, link up, netstack traffic. Then the Pi 4 (host and power differ).
