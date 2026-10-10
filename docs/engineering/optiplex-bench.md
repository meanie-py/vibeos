# OptiPlex 9020 bench

The x64 hardware bench: a Dell OptiPlex 9020 SFF (Haswell, Lynx Point PCH, Intel
I217-LM) running Libreboot. It took over the Pi 4 bench's plug, Pico and capture
card on 2026-10-09, so the `tools/pi4` scripts drive it with these overrides
until it has its own wrapper:

| Part | What | How to reach it |
|---|---|---|
| Power | The Pi 4's Shelly plug | `tools/pi4/power.sh status / cycle` |
| Keyboard, mouse, disk | Pico `picousb5` | `tools/pi4/pico.sh ...` (its default) |
| Screen | Capture card `vid_345f` behind a VGA-to-HDMI converter (the 9020 has VGA and DisplayPort, no HDMI; the converter needs its USB power lead) | `PI4_HDMI_VID=345f tools/pi4/hdmi.sh out.png` |
| Serial | The 9020's COM1, RS-232 to a Prolific PL2303GT on the host: Windows **COM7**, 115200 8N1 | coreboot logs every boot here at `debug_level = Debug` |

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

## Known gaps

- **The Pico's keys do not reach SeaBIOS or Ventoy.** Linux reads them; the
  pre-OS menus do not react (SeaBIOS completes one report per key and nothing
  happens). A real USB keyboard works there. Unsolved; needed before driving the
  loader from the bench.
- **`pico.sh key` reports `host_not_polling` under SeaBIOS** even when the key was
  delivered: the client waits for every HID interface to drain, and SeaBIOS never
  polls the mouse. Read the screen, not the exit code, until the client checks
  only the interface it used.
- No netboot yet. The disk is disposable (its Ubuntu died in a power cut); a Ventoy
  stick with Debian and Ubuntu ISOs is plugged in.
