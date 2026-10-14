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
| Serial | The 9020's COM1, RS-232 to a Prolific PL2303GT on the host: Windows **COM7**, 115200 8N1. coreboot logs every boot here (`debug_level = Debug`), and VibeOS's loader and kernel debug output follow (`/DEBUGPORT=COM1 /BAUDRATE=115200`) | `tools/optiplex/serial.sh up / status`; the live capture is named in `<bench>/serial-current.json`. `python3 tools/pi4/serial_times.py <capture dir> --gaps 10` (or `--lines`) puts arrival times on the latest boot |

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

`OPTIPLEX_TRIPWIRE='SPECIALPOOL=*' tools/optiplex/netboot.sh` arms the kernel's tripwire for
the staged image (a `/TRIPWIRE=` boot option; commas between tokens), as `smoke --tripwire`
does in QEMU. The copy into the payload dir is `dd` in 16 KB blocks: under host memory
pressure drvfs fails larger writes, and reads, with ENOMEM.

Timing (2026-10-13, from `serial.times`): coreboot to the kernel handoff in ~50 s, of which
freeldr's read of the image is a ~23 s silence; `shell ready` at ~2 min. (2026-10-10 measured
~3.5 min for the pull alone; nothing in the loader changed since, so suspect the host side
before relying on either number.) freeldr reads it 62 KB at a time (its INT 13h buffer sits below 1 MB), so a `sanboot` is ~6,500
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
| Lynx Point xHCI, 2x EHCI | Started. On the xHCI: the Pico, a Razer 1532:0099 and a 413d:2114 HID device, all bound to HID class drivers. Its HCSPARAMS1 says MaxPorts 85 against 21 ports in its protocol capabilities; usbxhci uses the 21 (2026-10-13) |
| Renesas uPD720201 USB 3 card (1912:0014) | usbxhci starts it (8 ports). Since 2026-10-11 a USB-C hub sits on it with a Wi-Fi/Bluetooth stick: its USB 2 side enumerates (`214b:7250` hub, VIA `2109:2815` hub, the Realtek RTL8821CU `0bda:c820`, no driver, and the hub's RTL8153B gigabit Ethernet `0bda:8153`, which Realtek's stock XP x64 driver 5.23 now drives: see "Vendor drivers" below). Its SuperSpeed side (`2109:0815`) fails START: usbhub has no USB 3 hub support (it asks for a 0x29 hub descriptor, which a SuperSpeed hub stalls). A cold boot that once never saw the hub was the card missing usbxhci's 20 ms halt deadline, a failure usbport then reported as success; fixed 2026-10-11 (roadmap bug registry) |
| Crucial P310 NVMe | nvme.sys, disk and three partitions |
| SATA AHCI, TSSTcorp DVD | UniATA; the DVD is `cdrom.sys` |
| I217-LM Ethernet (8086:153A) | **Works** (2026-10-11): netstack-rust's e1000 backend drives it (`pch.rs`: PCH reset, PHY wake, MAC from RAR0), DHCP leases an address from the router and the stack's ICMP self-test pings the gateway. Interrupt-driven since 2026-10-13: the BIOS's Interrupt Line (128) is an 8259 number, so pci.sys now writes the `_PRT`-routed GSI (23) back into the register, as Windows' pci.sys does, and netstack connects that (ping RTT 48 ms polled, 2.7 ms on the interrupt). Link autonegotiation takes over 5 s after reset. The MAC (`a6:ee:e0:c0:50:12`) is locally administered, as Libreboot's GbE region ships |
| HD Audio (8C20) | **Starts** (2026-10-11): the controller needed more than hdaudbus's 1 ms to leave reset (now 100 ms, as Linux). Codec 0 is a Realtek ALC280 with an audio function group; its `HDAUDIO\FUNC_01&VEN_10EC&DEV_0280` device has **no function driver** (none in the tree). The Haswell HDMI/DP audio controller (0C0C) starts, but its codec verbs time out |
| PCI INTx | All routed by ACPI `_PRT` (GSI 16-23, level, active-low) |
| PCI bus driver | `pci.sys` by default. `pcix.sys` (the `hivesys.inf` Pci `ImagePath`) also boots it to the desktop since 2026-10-12, with every device, the Renesas card behind root port 00:1c.0 and its hub included |

## Vendor drivers

A stock Windows driver package goes onto the netboot image with the `vendordrv` payload,
from a directory outside the repo (vendor binaries are not ours to commit):

```bash
VIBEOS_VENDOR_DRIVERS=<dir with the .inf and its files> \
VIBEOS_PAYLOAD=ndisprobe,vendordrv ./vibeos-cli build --arch amd64
tools/optiplex/netboot.sh stage && tools/optiplex/power.sh cycle 10
```

PnP's installer reaches the device about four minutes into boot, after every other
unmatched device. An `ndisprobe.txt` in that directory (`arp 192.168.1.1 0.0.0.0`) points
ndisprobe at the LAN's router.

**Realtek RTL8153B, XP x64 driver 5.23 (`rtux64xp.sys`, 2026-10-11):** installs from its own
INF, starts, binds to ndisuio, and answers OID queries with the chip's MAC
(`00:e0:4c:68:03:51`). Getting there took a growable NDIS packet pool (the driver asks for a
pool of 196,512 descriptors), pre-placing the `.sys` (the live image is read-only), and the
serial fixes that made the log readable. With a cable in the hub's port the link is up
4 s after the driver starts, and the ARP round trip to the router passes
(`[NDISPROBE] done VERDICT=PASS`): a vendor's binary miniport sending and receiving real
frames through our ndis.sys, usbport and xHCI. Since 2026-10-13 netstack can run the OS's
traffic over it: netstack binds an NDIS miniport when it finds no NIC of its own, and
`OPTIPLEX_BOOT_OPTIONS=/NETSTACK=NDIS tools/optiplex/netboot.sh` makes it leave the I217
alone so the Realtek is the one it takes (`[NETSTACK] NDIS: bound adapter`, then a DHCP
lease). Proven in QEMU on all three arches (smoke `--payload ndisprobe`); on this board it
needs the cable in the hub's port.

## Known gaps

- Audio: an HD Audio codec function driver for the ALC280, and the HDMI audio verb timeouts.
- USB: no SuperSpeed hub support in usbhub.
- The netboot pull: freeldr reads 62 KB per INT 13h call; bigger reads would cut it.
