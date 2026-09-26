# Raspberry Pi 4B bench

How to wire up and drive a physical Pi 4B for VibeOS arm64 bring-up, remotely
and without card swaps. Read this before touching the board. What has been
found *on* the board, and where the port stands, lives in the lane doc
`docs/engineering/active-lanes/pi4-bringup.md`.

Everything the bench runs is in `tools/pi4/`. The Windows-side paths below
(COM port, bench directory, python) are this bench's defaults, and each can be
overridden by the environment variable named beside it.

## Hardware

| Piece | What | Notes |
|---|---|---|
| Board | Raspberry Pi 4B, 4 GB+ | Firmware exposes 3 GiB by default (the firmware's own cap) |
| Firmware | [pftf/RPi4 UEFI](https://github.com/pftf/RPi4) v1.53 on a FAT32 SD card | ACPI mode. VibeOS boots as `EFI\BOOT\BOOTAA64.EFI` |
| Serial | 3.3 V USB-UART (FT232, VID 0403 / PID 6001) on the GPIO header | The only kernel debug channel |
| Video | HDMI → USB capture card (DirectShow "USB Video") | `tools/pi4/hdmi.sh` |
| Network | Ethernet on the same LAN as the host | TFTP netboot |
| Power | Shelly Plug (Gen2+ local RPC) | `tools/pi4/power.sh`, no one at the bench needed |

### Serial wiring

Three wires, **never four**:

| Adapter | Pi header pin | Pi signal |
|---|---|---|
| GND | 6 | GND |
| RXD | 8 | GPIO14 / UART TX |
| TXD | 10 | GPIO15 / UART RX |

Leave the adapter's VCC unconnected. Wired to the Pi, it back-powers the board
through the header, so a "power cycle" becomes a partial reset. Use a **3.3 V**
adapter (or set its VCCIO jumper to 3V3); 5 V signalling damages the SoC.

Line settings: **115200 8N1, no flow control.** Firmware describes this UART to
the OS in ACPI **DBG2** (PL011 at `0xFE201000`); there is no SPCR on this
firmware. The loader discovers it from there, so there is nothing to configure
in VibeOS.

## Host services

```bash
tools/pi4/bench.sh up       # serial capture listener + TFTP server
tools/pi4/bench.sh status   # listener, TFTP server, plug state and watts
tools/pi4/bench.sh stage    # copy the current arm64 build into the TFTP payload
tools/pi4/bench.sh down
```

Both services are **Windows** processes, started from WSL:

- **Serial listener** (`Capture-RpiSerial.ps1`, port `PI4_COM`, default COM5).
  Writes every byte to `<bench>/serial-<timestamp>/serial.bin`, and records the
  live capture in `<bench>/serial-current.json`. It is the **only** program that
  may open the port. A second reader steals bytes from the first, and Windows
  refuses the second open, which is what `"Access to the port is denied"` in
  `serial-current.json` means.
- **TFTP server** (`serve_tftp.py`, Windows python at `PI4_WINPY`). Read-only,
  serves `<bench>/payload`. It has to run on Windows because WSL2 in NAT mode
  (172.x) is not reachable from the LAN. It needs one inbound firewall rule: UDP
  69, Private profile, remote 192.168.1.0/24, scoped to that python.exe.

The bench directory is `PI4_BENCH` (default `/mnt/d/home/vibeos-rpi4-setup`).
`bench.sh up` copies the repo's scripts into it before starting them, so the
repo stays the source of truth.

### Talking to the board

Send input with `tools/pi4/serial-send.sh "command"` (appends CR) or `--raw`.
Do not write the port any other way:

- **Pace input.** The firmware drops characters at 115200 with no flow control.
  An unpaced `tftp ... fs0:\tftp-test.tmp` arrived as `freelt.tmp` **and ran**,
  against the wrong filename. `serial-send.sh` sends 4 bytes per listener poll.
- **Write, then rename.** The listener reads `serial-send.bin` while the sender
  may still be writing it. That sharing violation once killed the capture
  mid-session. The sender now renames a finished file into place, and the
  listener retries instead of dying.

Read output from `serial.bin` (it is raw bytes with ANSI colour codes, so pipe
it through `strings`).

### Power

```bash
tools/pi4/power.sh status        # "on  2.6 W"
tools/pi4/power.sh cycle [secs]  # off, then the plug turns itself back on
```

`cycle` uses the plug's own `toggle_after` timer, so if the host dies
mid-cycle the board still comes back on. Keep the gap at 5 s or more; the PSU's
capacitors ride through a shorter one. Set `PI4_PLUG` (hostname) and, if the
plug has a password, `PI4_PLUG_PASS`. **Never cycle while the Shell is writing
the SD card**: a power cut mid-write can corrupt the FAT volume the firmware
itself boots from.

## The loop

```bash
./vibeos-cli build --arch arm64
./vibeos-cli smoke --arch arm64        # QEMU first, always
tools/pi4/bench.sh stage
tools/pi4/netboot.sh [--ramdisk]       # power-cycle -> Shell -> TFTP -> boot
tools/pi4/hdmi.sh                      # what the screen shows
```

`netboot.sh` cycles power, catches the firmware splash and sends F1 for the UEFI
Shell. The splash is a window of about 5 s. Missing it costs about 90 s of
PXE/HTTP timeouts, and then the firmware boots whatever stale loader is on the
card. The script then brings up DHCP, TFTPs `BOOTAA64.EFI` (and, with
`--ramdisk`, the whole-system `ramdisk.img`) onto the card, launches it, and
picks the loader's default menu entry. The Pi's `freeldr.ini` is
`tools/pi4/freeldr.ini`: `TimeOut=-1`, so the menu waits for that keypress.

The loader is ~300 KB and lands in about a second. The ramdisk is ~140 MB and
takes **~13.5 minutes**, then freeldr spends another **~3.5 minutes** reading
it back off the card, so only pass `--ramdisk` when kernel-side code changed.

The limit is **the SD card as this firmware drives it, not the network**.
Measured 2026-09-26 on the same 146,800,640-byte file:

| Path | Time |
|---|---|
| Firmware Shell `tftp` → card | 805 s (825 s lockstep; windowing and 8K blocks changed nothing) |
| iPXE `imgfetch` (TFTP, lockstep, 1432-byte blocks) → RAM | **18.9 s** (7.8 MB/s) |
| Shell `cp` from iPXE's RAM volume → card | ~810 s |
| freeldr reading it from the card | ~3.5 min |

So card writes run at ~0.18 MB/s and every write-to-card route costs the same;
an earlier reading of the TFTP number as "one frame per poll" was wrong. The
fix is to not touch the card: iPXE `sanboot` of the whole boot image (iPXE
installs it as a UEFI block device, and freeldr reads block devices with its
own FAT driver), or freeldr fetching over the network itself (roadmap 10e).
Both need TCP (HTTP or iSCSI), so a second inbound firewall rule.

iPXE notes: use `ipxe.efi`, not `snponly.efi`. snponly only drives the NIC it
was loaded from, and loaded from the card it sees none. Chaining a UEFI Shell
from iPXE exposes iPXE's downloaded images as a volume, and in that shell the
mapping shifts: the iPXE volume becomes `FS0:` and the card `FS1:`. Find the
card by its `HD(1,MBR,...)` device path, never by number.

Every staged payload carries a `manifest.json` with the commit it came from and
whether the tree was dirty. Tie each board result to that, never to "what was on
the card".

## Firmware facts worth knowing

- `acpiview -s DBG2` / `-s APIC` in the UEFI Shell show what the loader will
  discover: the PL011 at `0xFE201000`, and GICv2 distributor `0xFF841000` / CPU
  interface `0xFF842000`.
- The firmware hands over at **EL2 without VHE** (`HCR_EL2.E2H=0`). QEMU hands
  over at EL1. The loader drops to EL1 itself before programming translation.
- UEFI saves its variables inside `RPI_EFI.fd` (NV region `0x3b0000..0x3d0000`),
  so that file's hash changes after boot-order edits. That is expected; any
  change outside that region is not.
- The HDMI capture card's first MJPEG frames are often garbage. `hdmi.sh`
  decodes 30 and keeps the last.
