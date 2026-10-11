# OptiPlex 9020 bench

The x64 hardware bench: a Dell OptiPlex 9020 SFF (Haswell, Lynx Point PCH, Intel
I217-LM) running Libreboot. It took over the Pi 4 bench's plug, Pico and capture
card on 2026-10-09. `tools/optiplex/` drives it: `env.sh` points the `tools/pi4`
machinery at this board (its own lease, `VIBEOS_BENCH_BOARD=optiplex`).

| Part | What | How to reach it |
|---|---|---|
| Power | The Pi 4's Shelly plug | `tools/optiplex/power.sh status / cycle` |
| Keyboard, mouse, disk | Pico `picousb5` | `tools/optiplex/pico.sh ...` |
| Screen | Capture card `vid_345f` behind a VGA-to-HDMI converter (the 9020 has VGA and DisplayPort, no HDMI; the converter needs its USB power lead) | `tools/optiplex/hdmi.sh out.png` |
| Serial | The 9020's COM1, RS-232 to a Prolific PL2303GT on the host: Windows **COM7**, 115200 8N1. coreboot logs every boot here (`debug_level = Debug`), and VibeOS's loader and kernel debug output follow (`/DEBUGPORT=COM1 /BAUDRATE=115200`) | `tools/optiplex/serial.sh up / status`; the live capture is named in `<bench>/serial-current.json` |

The Pi 5 keeps the other card (`vid_534d`, `tools/pi5/env.sh`).

## Firmware

Libreboot 25.06 (coreboot 25.06, `dell9020sff_nri_12mb`), SeaBIOS payload by
default, 12 MiB of flash over two chips (flashrom: "Opaque flash chip", hardware
sequencing). Internal flashing from Linux works: `flashrom -p internal --ifd -i bios`.

**It powers on when the plug restores power.** Stock Libreboot builds this board
with `CONFIG_STATIC_OPTION_TABLE=y`, which reloads CMOS from the ROM's
`cmos.default` at every boot, so `nvramtool -w power_on_after_fail=Enable` is
undone before coreboot reads it (the log keeps saying `Set power off after power
failure.`). The flash now carries a `cmos.default` with that one option set: two
bytes differ from the stock image (the option bit and the CMOS checksum).

- Stock dump: `D:\home\vibeos-optiplex\fw\rom-backup-20261009.bin`, SHA-256
  `e68111432eaa1674399021cc2e230385dea27c09654182ed5587a67bcbb6ad7f`.
- Flashed: `rom-poweron-20261009.bin`, SHA-256
  `ea3b0ceed86aff9f75f0fb5fbddc0fc088d368f1a6e2ed6e6143399f997fb439`.
- Made with coreboot 25.06's own `cbfstool` and `nvramtool` (Ubuntu's 4.15 ones are
  older than the image): `extract -n cmos.default`, `nvramtool -C <rom> -D <file>
  -w power_on_after_fail=Enable`, `remove` + `add -t cmos_default`, then `cmp -l`
  against the dump. Any other CMOS option is changed the same way.
- Verified: coreboot logs `Set power on after power failure.`, and after
  `power.sh cycle 10` the box draws ~54 W within 5 s of the plug coming back.

## Pico notes

- **The Pico drives SeaBIOS and Ventoy since picousb `0f43f4f`** (2026-10-09). A
  BIOS polls only the boot keyboard; the old firmware queued a mouse release per
  control session that never completed and then blocked its single HID FIFO, so
  no key arrived and `pico.sh key` exited `host_not_polling`. Now each interface
  is scheduled on its own and only interfaces the host reads get a release. Both
  bench Picos run it (the Pi 5's `picousb` was updated the same way from its
  Linux stager).
- **Updating a Pico in place:** put a wait-and-copy loop on the target first (its
  keyboard leaves with BOOTSEL), then `pico.sh bootsel`: from Linux,
  `while [ ! -e /dev/disk/by-label/RP2350 ]; do sleep 1; done`, mount it, copy
  picousb's stripped `build/firmware/picousb.uf2`, `sync`. The board flashes
  itself and rejoins Wi-Fi in ~15 s with its configuration intact. Stage the UF2
  somewhere root-owned (`/run`), not `/dev/shm`: logind's `RemoveIPC` deletes a
  user's `/dev/shm` files when the SSH session that wrote them ends.

## Netboot

Hands-free from a plug cycle: SeaBIOS boots the NVMe, whose GRUB (`/boot/grub/grub.cfg`,
first entry, 3 s timeout) chainloads `ipxe.lkrn` (iPXE v2.0.0, **i386** build; the
x86_64 one reboot-loops here) with an embedded script that chains
`http://192.168.1.183:8069/optiplex.ipxe`, which `sanboot`s `vibeos-amd64.iso` from the
bench HTTP server's payload directory.

```bash
./vibeos-cli build --arch amd64
tools/optiplex/netboot.sh            # build the RAM-disk image, copy + verify it
tools/optiplex/power.sh cycle 10     # boot it; watch COM7 (serial.sh)
```

`netboot.sh` does not stage the plain livecd. iPXE's SAN disk is an INT 13h device that
freeldr reads but the kernel cannot see, so the livecd loads its kernel and then has no
boot volume. The staged image holds the loader and `livecd/livecd.iso`, and its only
`freeldr.ini` entry loads that into a RAM disk exported as a CD (`/RDPATH ...
/RDEXPORTASCD`, the SMP kernel pair, debug on COM1). `netboot.sh` also writes
`optiplex.ipxe`; it `imgfree`s first, because the board's embedded script re-chains it on
failure and a failed attempt's images stay registered (each retry leaked 400 MB until every
download failed with ENOSPC, hiding the first error).

Timing (2026-10-10): ~3.5 min pulling the image, then ~1 min to `shell ready`. freeldr
reads it 62 KB at a time (its INT 13h buffer sits below 1 MB), so a `sanboot` is ~6,500
HTTP range requests; `serve_http.py` speaks HTTP/1.1 keep-alive for that, which took the
pull from ~1 MB/s (a new TCP connection per request) to ~2 MB/s.

Tried and dropped: iPXE `kernel memdisk iso raw` + `initrd` (one GET, then INT 13h from
RAM; boots in QEMU). On the board, GRUB's iPXE fails the 400 MB `initrd` with ENOSPC
although its heap has 2 GB (a chained second iPXE downloads it fine; not run down), and
memdisk's CD emulation leaves the NVMe at drive 80h, where `isoboot.bin` finds an MBR and
offers "Press any key to boot from the ReactOS medium", then boots the NVMe. `isobtrt.bin`
is no way round that: without the prompt it boots the hard disk at once.

## What VibeOS sees (2026-10-10)

| Hardware | Status |
|---|---|
| CPU, i7-4770 (4C/8T) | 8 processors started (MADT) |
| RAM, 32 GB | All of it: `[MM] physical memory: 32686 MiB` |
| HD 4600 iGPU | VBE linear framebuffer at the panel's native 1920x1080; no native driver |
| Lynx Point xHCI, 2x EHCI | Started. On the xHCI: the Pico, a Razer 1532:0099 and a 413d:2114 HID device, all bound to HID class drivers. It reports 85 ports (should be 21) |
| Renesas uPD720201 USB 3 card (1912:0014) | usbxhci starts it (8 ports); its root hub enumerated no devices |
| Crucial P310 NVMe | nvme.sys, disk and three partitions |
| SATA AHCI, TSSTcorp DVD | UniATA; the DVD is `cdrom.sys` |
| I217-LM Ethernet (8086:153A) | **No driver**: e1000 lists no PCH (e1000e) parts; `NicPresent=0` |
| HD Audio (8C20) | **hdaudbus fails**: "unable to exit reset", START 0xC0000001. The HDMI/DP audio function starts but its codec verbs time out |
| PCI INTx | All routed by ACPI `_PRT` (GSI 16-23, level, active-low) |

## Known gaps

- The NIC and audio (table above).
- The netboot pull is still ~2 MB/s; the next step is bigger INT 13h reads in freeldr.
