# Raspberry Pi 4B bench

How to wire up and drive a physical Pi 4B for VibeOS arm64 bring-up, remotely
and without card swaps. Read this before touching the board. What has been
found *on* the board, and where the port stands, lives in the lane doc
`docs/engineering/active-lanes/pi4-bringup.md`.

## Booting your change on the Pi (any lane)

```bash
./vibeos-cli build --arch arm64     # pi4 boot refuses stale media, like smoke
./vibeos-cli pi4 boot               # lease, doctor, stage, netboot, desktop verdict
./vibeos-cli pi4 boot --kernel up   # the UP kernel pair instead of MP
```

`pi4 boot` is `smoke` for the real board, and takes about two minutes. It takes
the bench lease, checks every bench service, stages *this* tree's arm64 media
and netboots it. It then waits for the same `shell ready` marker smoke requires,
and classifies a failure with smoke's triage (BUGCHECK, RUST_PANIC, HANG_TIMEOUT,
plus `NETBOOT_FAIL`, `STAGE_FAIL`, `BENCH_BUSY` and `BENCH_UNHEALTHY`). The run
directory holds `summary.json`, this boot's slice of the serial log as
`serial.log`, and `hdmi.png`. The screenshot waits up to 30 s for the HDMI frame
to stop being black, because the display lags the marker by a few seconds.

- **`./vibeos-cli pi4 doctor`** checks the plug, the Pico (including that it is on
  the Pi's USB and not on another host), the serial listener, TFTP, HTTP, the
  capture card and the staged payload. Each failure prints the command that fixes
  it.
- **`tools/pi4/bench.sh serial-restart`** replaces a serial listener whose capture has
  stalled. pico_serial.py can keep reporting `LISTENING, connected` for hours after its stream
  stops, and the doctor passes it; the symptom is a boot that fails `NETBOOT_FAIL` with "serial
  capture did not grow at all" while HDMI shows the firmware splash timing out to PXE.
- **`./vibeos-cli pi4 status`** is the one-screen view: lease, power, Pico, serial
  and payload.
- **`./vibeos-cli pi4 lease take --minutes 60 --why "..."`** holds the board
  across several manual commands; release it with `pi4 lease drop`. There is one
  board.
  - Every bench script that changes its state (`power.sh` on/off/cycle,
    `netboot.sh`, `serial-send.sh`, `bench.sh stage|down`, and `pico.sh` input,
    media and `bootsel`) takes or renews the lease for your worktree.
  - While another lane holds it, those scripts exit **75** and name the holder.
  - Reading state (`power.sh status`, `pico.sh usb`, `hdmi.sh`, `serial.bin`) is
    always allowed.
  - `pi4 lease drop --force` is for a lane that died holding the lease.

Everything the bench runs is in `tools/pi4/`. The Windows-side paths below
(COM port, bench directory, python) are this bench's defaults, and each can be
overridden by the environment variable named beside it.

## Hardware

| Piece | What | Notes |
|---|---|---|
| Board | Raspberry Pi 4B, 4 GB+ | Firmware exposes 3 GiB by default (the firmware's own cap) |
| Firmware | [pftf/RPi4 UEFI](https://github.com/pftf/RPi4) v1.53 on a FAT32 SD card | ACPI mode. VibeOS boots as `EFI\BOOT\BOOTAA64.EFI` |
| Serial | Pico 2 W UART bridge over Wi-Fi (default), or a 3.3 V USB-UART (FT232) | The only kernel debug channel; see "Host services" |
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
tools/pi4/bench.sh up       # serial listener (PI4_SERIAL=pico|com) + TFTP + sanboot HTTP servers
tools/pi4/bench.sh status   # listener, TFTP server, plug state and watts
tools/pi4/bench.sh stage    # copy the current arm64 build into the TFTP payload
tools/pi4/bench.sh down
```

- **Serial listener.** It writes every byte to `<bench>/serial-<timestamp>/serial.bin`
  and records the live capture in `<bench>/serial-current.json`. There are two
  kinds, selected by `PI4_SERIAL`. Both keep the same files, so `netboot.sh` and
  `serial-send.sh` work with either.
  - **`pico`, the default since 2026-09-27.** `tools/pi4/pico_serial.py` runs in
    WSL under the picousb client venv. It reads the Pi's UART through the Pico 2 W
    over Wi-Fi, on TCP 8082 with the device token.
    - Wiring: Pi pin 8 (TXD) → Pico GP1, Pi pin 10 (RXD) ← Pico GP0, GND ↔ GND.
    - A console that stays silent while the Pi boots usually means the two signal
      wires are swapped. The Pico's own diagnostic log (UART1, GP4) then shows
      `console_rx` with `bytes` stuck at 0.
    - The Pico keeps the newest 128 KiB of output while no listener is attached,
      and marks any gap with `[picousb: N console bytes lost]`.
    - It reconnects on its own. Its log is `<bench>/pico-serial.log`.
    - Protocol and pin handling: picousb `docs/protocol.md`, "Serial console".
  - **`com`.** `Capture-RpiSerial.ps1` on a Windows USB-serial adapter, port
    `PI4_COM`, default COM5. It is the **only** program that may open the port. A
    second reader steals bytes from the first, and Windows refuses the second open,
    which is what `"Access to the port is denied"` in `serial-current.json` means.

The TFTP and HTTP services are **Windows** processes, started from WSL:

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
tools/pi4/netboot.sh                   # power-cycle -> Shell -> iPXE sanboot: ~100 s to desktop
tools/pi4/hdmi.sh                      # what the screen shows
```

The default is sanboot (below): nothing but iPXE touches the SD card, and a
kernel change reaches the desktop ~100 s after power-on (measured 2026-09-26:
80 s firmware + Shell + iPXE, 20 s loader to desktop). `netboot.sh --card
[--ramdisk]` is the old path, TFTP onto the card, kept for when HTTP is down.

`netboot.sh` cycles power, catches the firmware splash and sends F1 for the UEFI
Shell. The splash is a window of about 5 s. Missing it costs about 90 s of
PXE/HTTP timeouts, and then the firmware boots whatever stale loader is on the
card. The script then brings up DHCP, TFTPs `BOOTAA64.EFI` (and, with
`--ramdisk`, the whole-system `ramdisk.img`) onto the card, launches it, and
picks the loader's default menu entry. The Pi's `freeldr.ini` is
`tools/pi4/freeldr.ini`: `TimeOut=-1`, so the menu waits for that keypress.

On the `--card` path the loader is ~300 KB and lands in about a second. The
ramdisk is ~140 MB and takes **~13.5 minutes**, then freeldr spends another
**~3.5 minutes** reading it back off the card.

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

### Sanboot (no card writes)

`tools/pi4/netboot.sh` (the default) TFTPs `ipxe.efi` and `autoexec.ipxe` onto
the card (a second each) and launches iPXE, which runs
`sanboot http://<host>:8069/boot.img`. `boot.img` is `qemu/boot.img` with the
Pi's `freeldr.ini`, built by `bench.sh stage`; `bench.sh up` (or `http`) serves
it with `serve_http.py` (HEAD and byte ranges, which iPXE needs). Proven in
QEMU and then on the Pi on 2026-09-26. It needs a second inbound firewall rule,
created once by the bench owner in an elevated PowerShell (this bench has it):

```powershell
New-NetFirewallRule -DisplayName "VibeOS Pi4 sanboot HTTP (bring-up)" -Direction Inbound `
  -Protocol TCP -LocalPort 8069 -RemoteAddress 192.168.1.0/24 -Profile Private `
  -Program "C:\Users\Lambda\AppData\Local\Programs\Python\Python313\python.exe" -Action Allow
```

iPXE notes: use `ipxe.efi`, not `snponly.efi`. snponly only drives the NIC it
was loaded from, and loaded from the card it sees none. Chaining a UEFI Shell
from iPXE exposes iPXE's downloaded images as a volume, and in that shell the
mapping shifts: the iPXE volume becomes `FS0:` and the card `FS1:`. Find the
card by its `HD(1,MBR,...)` device path, never by number. A USB stick with a FAT
partition shifts it too, in the firmware's own Shell: a Ventoy stick's `VTOYEFI`
mapped as `FS0:` and `netboot.sh`, then hardcoding `fs0:`, wrote the loader onto
the stick. It now takes the one filesystem whose device path has no `USB(` from
the Shell's startup mapping table (the card's is `VenHw(100C2CFA-…)`), and
refuses to write when it cannot find exactly one.
`netboot.sh --shell` power-cycles into the Shell and stops there, for reading
bench media with a second FAT implementation (`tools/pi4/serial-send.sh
'type fs0:\FILE'`); that is how a file VibeOS wrote to a stick was checked
after a power cycle.

Every staged payload carries a `manifest.json` with the commit it came from and
whether the tree was dirty. Tie each board result to that, never to "what was on
the card".

## USB input: the Pico

A Pico 2 W running the picousb firmware (`D:\home\Projects\picousb`) sits in a
Pi USB port. It presents a boot keyboard, boot mouse, mass storage and an
absolute pointer, is controlled over Wi-Fi, and reports what the Pi's USB host
did to it. `tools/pi4/pico.sh` wraps its CLI (address, token file, venv; setup
in the script header):

```bash
tools/pi4/pico.sh usb                       # mounted / resets / per-endpoint completions
tools/pi4/pico.sh type 'hello' --delay 30
tools/pi4/pico.sh pointer click 16384 16384 # absolute, 0..32767 per axis
```

The token file is created by the bench owner; agents pass its path and never
read it. Since 2026-09-27 the Pico is also fed 5 V on VSYS through a rectifier
diode, besides the Pi's USB. It stays up and on Wi-Fi across a `power.sh cycle`:
`uptime` keeps counting, `usb` shows the suspend, and UEFI re-enumerates it
~10 s in. Before that it rebooted with the Pi and joined Wi-Fi too late for the
UEFI F1 window. Driving F1 from the Pico is untested; serial works.

**Pico firmware updates.** The safe path is the host: plug the Pico's USB into
the PC and run `pico-devbox bootsel` (picousb; no button needed). Then copy
picousb's stripped `build/firmware/picousb.uf2` onto the RP2350 drive. Never use
picotool's raw UF2: it wraps onto the config sector. `tools/pi4/pico-flash.sh`
does the same through the Pi's UEFI Shell with the Pico left on the Pi. It is
experimental and refuses to run without `PI4_PICO_FLASH_ATTENDED=1`, because a
failed copy leaves the Pico in BOOTSEL until its power is pulled.

`pico.sh usb` also reports the console bridge: `target_powered` is the Pi's USB
VBUS as the Pico sees it (`pi4 doctor` checks that it follows the plug), plus
`console_rx`, `console_errors` and `console_dropped`. When a boot leaves the
console silent, `console_rx` stuck at 0 means the TX/RX wires are swapped or
open, and a climbing `console_errors` means a baud or noise problem.

The Pico is VibeOS's keyboard and mouse on the Pi (usbxhci M4, 2026-09-26):
- **Configuration:** VibeOS configures it, and `usb` then shows `mounted=True`,
  `set_config=2` and growing completions.
- **Typing:** `type` lands in the focused window.
- **Pointer:** `pointer click x y` goes where aimed. The pointer spans the whole
  1920x1080 screen, so pixel = value × 1920 / 32768 (and × 1080 / 32768 for y).
  Notepad's text area is around `pointer click 8000 12000`.
- **Chords:** modifiers come first, e.g. `key CTRL ESC`.

What proves each layer, on serial:

| Layer | Serial line |
|---|---|
| Controller | `[XHCI] controller running` |
| Hub | `slot 1: hub, 4 ports`, the board's VIA hub |
| Pico | `port 1 route 3: new device ... tt slot 1 port 3` |
| Desktop | `[WIN32K] RIT first keyboard input` |

Before this (build `f5abd8cee6a4`), UEFI configured the Pico and a bus reset at
hand-over unmounted it for good.

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
