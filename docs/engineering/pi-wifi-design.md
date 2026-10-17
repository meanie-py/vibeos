# Raspberry Pi Wi-Fi: our own CYW43455 driver

Status (2026-10-17): protocol, SDIO, chip, firmware-load and frame layers host-tested
against a simulated controller and firmware; on the Pi 5, bwfm.sys loads the firmware and
the firmware answers `ver`, takes the CLM blob, scans, and joins the bench's hidden
WPA2-PSK network with the host's own 4-way handshake (run `20261017-074933-gupdzsjz`, see 6).
bwfm.sys is an NDIS 6 miniport and carries netstack's traffic: DHCP lease, gateway ping
and DNS over Wi-Fi, and the host pings the board back (run `20261017-105240-4fzlm0y6`, see 7).

## Why our own

Windows ships no driver for the Pis' onboard Wi-Fi, and the USB stick's Realtek driver
(`rtwlanu.sys`, which the OptiPlex runs) is amd64-only: the Windows 11 ARM64 image (26300)
carries no Realtek Wi-Fi driver at all; its only USB network drivers are ASIX Ethernet
(`netax88179_178a`, `netax88772`) and RNDIS. A driver we write is one we can ship.

## The hardware, both boards

| | Pi 4 | Pi 5 |
|---|---|---|
| Chip | Cypress/Infineon CYW43455 (BCM4345/6), SDIO, FullMAC | same |
| SDIO host | Arasan SDHCI, ACPI `SDC1` `_HID BCM2847` | BCM2712 `sdio2`, ACPI `SDC1` `_HID BRCM5D12` (child `WLAN`, `_CID 80860F16`: generic SDHCI) |
| SD card slot (not this) | EMMC2 `BRCME88C`, sdhc.sys | `SDC0`, also BRCM5D12 (child `SDMM`), sdhc.sys |
| Power (WL_REG_ON) | firmware GPIO expander, via the mailbox (`RPIQ`) | a GPIO line |
| NVRAM file | `brcmfmac43455-sdio.txt` (the `raspberrypi,*` names link to it) | same |

The Pi 4's `_HID`s are from a boot's `[ACPI] DSDT` lines (`serial-20261008-194631`). The
Pi 5's are from its DSDT (`docs/engineering/pi5-acpi/dsdt.dsl`): SDC1 is at 0x10_0110_0000
(0x260 bytes), INTID 306, level, with no power method. Its boot log line reads
`SDC0 _HID 80860F16` because SDC0's `_HID` is a method whose first string is that one; it
returns BRCM5D12 by default (`SDCM`). The HAL publishes SDC1 as `VIBEOS\CYW43455`, so
sdhc.sys, which takes BRCM5D12, does not bind it.

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

1. **SDIO host**: `sdhci-rust`, the SDHCI controller layer that sdhc.sys also runs on now
   (taken out of `sdhc-rust/src/host.rs`), and `bwfm-rust/src/sdio.rs`: CMD5 (I/O OCR),
   CMD3/CMD7, CMD52, CMD53 (whole blocks in block mode, the tail in byte mode), CCCR/FBR,
   the CIS's MANFID, 4-bit bus. Still to come: high speed and the card interrupt.
2. **Chip bring-up** (`chip.rs`, so far up to the chip id): the SDIO backplane window
   (function 1, `SBADDR`), chip and core ids,
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
   the firmware files (done, above); SDC1's resources match `0x1001100000` (done).
3. SDIO host: CMD5 and CMD52 against the chip, read its CIS and chip id. First board milestone.
   Done on the Pi 5 (run `20261017-012021-t5v81pv4`), first try:

       [BWFM] SDIO host 0x1001100000+0x260: version 0x1002, caps 0x55eec832, ...
       [BWFM] WL_ON (GPIO 28) as the firmware left it: output high (DATA 0x9f100002 IODIR 0xefffffff)
       [BWFM] SDIO card 02d0:4345, RCA 0x0001, 3 functions, OCR 0xffff00, CCCR rev 0x43, 25000 kHz 4-bit
       [BWFM] chip 0x4345 rev 6 package 2 cores 5 (chip id 0x15264345, CYW43455)
       [BWFM]   core 0x800 rev 51 base 0x18000000 wrap 0x18100000    chip-common
       [BWFM]   core 0x812 rev 54 base 0x18001000 wrap 0x18101000    802.11
       [BWFM]   core 0x83e rev  9 base 0x18002000 wrap 0x18102000    ARM CR4
       [BWFM]   core 0x83c rev 14 base 0x18003000 wrap 0x18103000    PCIe2
       [BWFM]   core 0x829 rev 21 base 0x18004000 wrap 0x18104000    SDIO device

   So the UEFI firmware leaves WL_ON high and the SDIO pins muxed: the driver still drops
   the line for 20 ms and raises it, so the chip starts from its power-on reset. Two more
   components (0x135, 0x240) have no register base; they are the interconnect's own.
   SDHCI version 0x1002 is spec 3.00, vendor 0x10.
4. Firmware download (done on the Pi 5, run `20261017-030446-amqa391_`, first try): the
   CR4 halted and the 802.11 core held in reset, RAM sized from the CR4's TCM banks
   (800 KiB at 0x198000), the firmware written at the bottom and the converted NVRAM
   (`nvram.rs`) at the top, both read back and compared, the first word planted at 0 as
   the reset vector, the CR4 released; then HT, the mailbox version, and function 2,
   whose I/O-ready is the firmware up:

       [BWFM] firmware loaded and verified: 609309 bytes at 0x198000, NVRAM 1748 bytes at 0x25f92c, reset vector 0xb83ef198, 130 ms
       [BWFM] firmware running: function 2 ready after 75 ms

   The files come from a build with `VIBEOS_PAYLOAD=piwifi` (`tools/payloads/piwifi`),
   which runs the pinned fetch and stages them in `\SystemRoot\System32\drivers\cyw43455\`.
   Without it bwfm.sys waits 120 s for them and logs that VibeOS does not ship them.
   `ver` over BCDC (done, run `20261017-035410-ouzyie33`): `bus.rs` reads function 2's
   FIFO (a zero header is "nothing queued"; an empty frame only carries credit), sends
   control frames against the credit window, and matches a reply by command and request
   id, handing anything else that arrives meanwhile on:

       [BWFM] firmware says: wl0: Aug 29 2023 01:47:08 version 7.45.265 (28bca26 CY) FWID 01-b677b91b (2 frames, 0 other frames before the reply, credit window 20)

   The firmware's first frame is a credit-only one; the host sends nothing until it has
   read it. Then the MAC (`cur_etheraddr`) and the CLM blob, in `clmload` chunks of at
   most 1400 bytes behind a download header (BEGIN on the first, END on the last), checked
   with `clmload_status` (done, run `20261017-043257-y_61cdjm`):

       [BWFM] MAC b8:27:eb:74:f2:6c
       [BWFM] CLM loaded: 2676 bytes in 2 chunks, clmload_status 0

5. Scan (done, run `20261017-050605-mg4n2t32`, first try): `mpc`, the station event
   mask (`event_msgs`), `C_UP`, `C_SET_INFRA`, then a passive `escan` of every channel
   (version 1 parameters, `scan.rs`). Each network arrives as an `E_ESCAN_RESULT` PARTIAL
   event holding `bwfm_bss_info` records; SUCCESS ends it:

       [BWFM]   10:7c:61:a7:7b:20 ch  11  -38 dBm  Rokkenjima
       [BWFM]   10:7c:61:a7:7b:24 ch  52  -50 dBm  Rokkenjima_5G
       [BWFM] scan Complete in 4045 ms: 12 networks, 15 reports

   A result frame carries each network's IEs and can pass 2 KiB, so the frame buffer is
   8 KiB, and a frame bigger still is read out and dropped rather than left half-read in
   the FIFO. The bus's buffers live in a static (`bus::Buffers`), not in `Bus`: built on
   the worker thread's stack, 18 KiB of them would be a stack overflow waiting to happen.
6. Join and WPA2 (done on the Pi 5, `join.rs`). The network comes from
   `\SystemRoot\bin\wifi.txt` (`VIBEOS_WIFI_CONFIG`, staged by the piwifi payload; the
   passphrase never reaches a log). The bench network is hidden, so a passive scan does not
   name it: a directed escan (active, one SSID) does. Then `wpaie` (our RSN element),
   `wpa_auth` WPA2-PSK, `wsec` AES, `auth` open, and the `join` iovar. The firmware
   associates; EAPOL arrives as data frames and wpa-rust's `Supplicant` (ndis-rust's, moved
   to its own crate) answers; the pairwise key goes down (`wsec_key`) only after message 4
   has gone, then the group key. Three things the board taught:
   - `sup_wpa` is unknown to this firmware (BCME_UNSUPPORTED): it has no supplicant of
     its own, so the handshake is ours either way. Refused, logged, not fatal.
   - The firmware writes its own RSN element into the association request: capabilities
     0x000C where we gave 0x0000. The AP drops a message 2 that does not repeat the
     request's element byte for byte (it resent message 1 three times, run
     `20261017-070017-_740qopb`). So before answering message 1 the driver reads what was
     sent (`assoc_info`, then `assoc_req_ies`) and the supplicant repeats that.
   - `assoc_info` answers BCME_BUFTOOSHORT unless the query buffer holds its whole
     structure (8 bytes is not enough; 480 is).

       [BWFM] 'Piconet' not heard (hidden=1): asking for it by name
       [BWFM] joining 'Piconet' at 10:7c:61:a7:7b:21 (ch 11, -35 dBm)
       [BWFM] association request RSN 30140100000fac040100000fac040100000fac020c00 (not ours: message 2 used it)
       [BWFM] 4-way handshake complete in 2789 ms: link up, keys installed (2 EAPOL in, 2 out)

   `pi5 boot --until TEXT` keeps reading serial past the desktop: the join finishes after it.
7. Data path: an NDIS 6 miniport (`miniport.rs`, `link.rs`). One path to netstack for
   every Wi-Fi adapter: netstack binds an NDIS 802.3 miniport when it drives no NIC itself
   (`/NETSTACK=NDIS` on the Pi 5, whose GEM it would otherwise take), as it binds the
   OptiPlex's Realtek. PnP installs bwfm from `netbwfm.inf` on `VIBEOS\CYW43455` (a Net
   class key is what ndis.sys's AddDevice needs, so the critical-device entry is gone).
   Initialize runs the bring-up to a running firmware and its MAC (the general attributes
   need it) and starts a worker, which owns the bus from then on: scan, join, link up to
   the protocols, then frames both ways; when the link fails or the AP leaves, `leave`,
   link down, and join again (5 s, doubling to a minute after failures). The send handler
   may run at DISPATCH_LEVEL and the SDIO bus waits, so it only copies frames into a ring
   (`net::TxRing`) for the worker. Receives are indicated at DISPATCH_LEVEL with
   RESOURCES (NDIS copies; the one NBL is never returned). The worker polls the chip every
   4 ms when idle: the card interrupt is still owed. The join hands back its `Station`,
   whose supplicant answers the AP's group rekeys for as long as the link lasts. The NDIS 6
   layouts are one crate, `ndis6-sys`, shared with ndis.sys's NDIS 6 shim.
   On the Pi 5 (`pi5 boot --options /NETSTACK=NDIS`, run `20261017-105240-4fzlm0y6`), PnP
   installed it about 1.5 minutes into boot and netstack bound it before the join finished:

       [BWFM] NDIS adapter ready
        [NETSTACK] NDIS: bound adapter MAC b8:27:eb:74:f2:6c
       [BWFM] 4-way handshake complete in 2788 ms: link up, keys installed (2 EAPOL in, 2 out)
       [BWFM] link up at 72 Mb/s: carrying traffic
        [NETSTACK] dhcp lease 192.168.1.150
        [NETSTACK] ping gateway OK
        [NETSTACK] dns OK (example.com resolved)

   The host pinged 192.168.1.150 back, 5 of 5. Still owed: a soak through the AP's group
   rekeys on the board (host-tested only), the card interrupt in place of the 4 ms poll,
   high-speed SDIO, and the Pi 4 (host and power differ).
