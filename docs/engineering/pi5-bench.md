# Raspberry Pi 5 bench

> **No bench since 2026-10-18:** its plug, Pico, capture card and Ethernet moved to the
> Pi 4. `tools/pi5/env.sh` points at `none.invalid` until `PI5_PLUG` and
> `PI5_PICO_HOST` name new ones.

The Pi 4 bench's twin (`docs/engineering/pi4-bench.md`): same lease, plug, Pico and
capture-card machinery, pointed at the other board by `tools/pi5/env.sh`. What is
different is how a build reaches the board.

```bash
./vibeos-cli build --arch arm64
./vibeos-cli pi5 boot          # lease, Linux stager, stage over SSH, tryboot, verdict
./vibeos-cli pi5 doctor        # plug, Pico, serial, stager, capture card
./vibeos-cli pi5 status
./vibeos-cli pi5 lease take --minutes 60 --why "..."   # separate from the Pi 4's lease
```

**Opt-in boot diagnostics** (`pi4-bench.md` lists them) go on the command line, not into
`freeldr.ini`: `pi5 boot --options /PROCTRACE` appends to every entry's `Options=` for that
staging only (`VIBEOS_LOADER_OPTIONS` to `stage.sh`). `pi5 boot --stortest` is the USB
storage end-to-end check: it serves a fresh FAT image labelled `VIBEUSB` (holding
`MARKER.TXT`) as the Pico's writable remote disk, boots with `/STORTEST`, grades
`stortest.exe`'s `[STORTEST] ALL OK` on serial, and then reads `WRITETST.TXT` back off the
image on the host, so the write is proven by the medium rather than by the guest that made
it. It needs the serial listener (`tools/pi5/serial.sh up`). The card is labelled
`VIBEOS_RPI5`, which stortest does not match, so it never writes to the card.

VibeOS reached the desktop on the Pi 5 on 2026-10-04. The first hands-free
`pi5 boot` took 2m48s from SSH stage to desktop. Since 2026-10-06/07 both of RP1's
xHCI controllers (XHC0, XHC1) run under `usbxhci.sys`, so every USB port works and the
Pico's keyboard and mouse with them, and the card slot (SDC0) runs under the Pi 4's
`sdhc.sys`, at ~11 MB/s. Since 2026-10-08 the Ethernet works too (`gem.rs`, polled: 1 Gb/s
link, DHCP, ping, TCP; interrupt-driven since that evening). The Pico's remote disk reads as a drive too, once the host
serves it an image (`tools/pi5/pico.sh serve-image`; see `pi4-bench.md`). The two controllers share one level-triggered SPI (261),
which needed shared-vector chaining in the arm64 kernel (`ke/arm64/stubs.rs`).

## How a build gets there: the Linux stager and tryboot

The Pi 5's UEFI is worproject's rpi5-uefi v0.3. It is archived, and that release is
its last. Its network stack has no driver for the board's Ethernet, because the
Ethernet sits behind the RP1 I/O chip. So there is no netboot, and the SD card
carries two systems in one FAT partition (`tools/pi5/card/`):

- **A normal boot starts Raspberry Pi OS: the stager.** It owns the Ethernet.
  cloud-init sets it up as `bench@vibeos-pi5`, with key-only SSH using the host's
  `~/.ssh/id_ed25519`, and masks `rpi-eeprom-update`.
- **`pi5 boot` copies this tree's files and reboots once into VibeOS.**
  `tools/pi5/stage.sh` copies `BOOTAA64.EFI`, `freeldr.ini`, the ramdisk and
  both config files over SSH and checks each by SHA-256 on the card. It then runs
  `reboot '0 tryboot'`. For that one boot the firmware reads `tryboot.txt`
  **instead of** `config.txt`: UEFI as the armstub with its own device tree, then
  VibeOS. A `[tryboot]` section in `config.txt` is never consulted; that filter is
  for `autoboot.txt`. With one, the first `pi5 boot` left the bootloader cycling
  SD → USB → restart.
- **`tryboot.txt` is the UEFI release's own `config.txt`, unmodified, and the
  partition root is a UEFI-only card.** The stager's kernel, initramfs, device tree
  and `cmdline.txt` live in `linux/` (`config.txt`: `os_prefix=linux/`), and the
  root's `bcm2712-rpi-5-b.dtb` is the UEFI release's. Variations booted to a live,
  all-black HDMI frame with no UEFI logo, twice: a renamed device tree with the
  stager's kernel in the root, then the same with `kernel=` pointed away from it.
  `stage.sh` moves an older card's stager into `linux/`. It installs the
  `config.txt` that depends on that only once all four files are there.
- **Any later reset is Linux again.** `pi5 boot` power-cycles a board that is still
  in VibeOS or hung through its plug, then waits for the stager's SSH.

The EEPROM stays on the version the board shipped with: worproject reports that
newer EEPROMs break the UEFI's display on D0 boards. Both bench boards report
"Raspberry Pi 5 Model B Rev 1.0".

## Building the card (one time)

```bash
tools/pi5/make-card.sh /tmp/pi5card 8b744144     # the card's MBR id = cmdline.txt's PARTUUID
powershell.exe -File tools/pi5/Deploy-Card.ps1 -Src <the tree, Windows path> -Drive F -PartUuid 8b744144
```

The card must already hold a Raspberry Pi OS root filesystem in partition 2.
`make-card.sh` takes the kernel from Raspberry Pi OS Lite 2025-12-04, whose kernel
matches that root's modules. Inputs are pinned by SHA-256 and cached in
`<bench>/cache`. `Deploy-Card.ps1` refuses any disk that is not plainly this card,
and reads every file back. The stager's host key is in `<bench>/known_hosts`;
delete that file after rebuilding a card.

## Serial, plug, Pico, capture

- **Serial** comes from the 3-pin JST-SH debug connector, wired to picousb: the Pi's
  TX to GP1 (the Pico's UART0 RX), its RX to GP0 (UART0 TX), and the middle pin to
  GND. Crossed the other way, both directions are silent: no byte arrives and the
  Pico counts no errors. It was, until 2026-10-06, so no boot before then had serial.
  Check it from the stager: `echo hi | sudo tee /dev/ttyAMA10` must raise the Pico's
  `console_rx` (`./vibeos-cli pi5 status`). `pico.sh serial --send` needs `--seconds`. That port is the UART UEFI's ACPI tables name, so the firmware, UEFI, the
  loader and the kernel all print there with no configuration.
  - Start the listener with `tools/pi5/serial.sh up`.
  - **With the listener, `pi5 boot` waits for `shell ready`** like `pi4 boot`.
  - **Without it, the verdict is HDMI-only and passes only as a warning:** the
    desktop background has to fill at least 20% of the frame.
  - GPIO14/15 belong to RP1, which neither UEFI nor VibeOS sets up yet.
- **Plug:** a Shelly Plug US Gen4, `shellyplugusg4-acebe6f18e28` (`tools/pi5/env.sh`;
  `PI5_PLUG` overrides). `tools/pi5/power.sh` refuses to run if none is named,
  because `tools/pi4/power.sh` would otherwise fall back to the Pi 4's plug. The
  meter reads 0 W for the first ~30 s after power-on; a booted stager draws ~5 W.
- **Pico:** `picousb` is the Pi 5's (`picousb5` is the Pi 4's). Use
  `tools/pi5/pico.sh` for input.
- **Capture:** the Pi 5's card has USB vendor id 345f and the Pi 4's has 534d. Both
  are named "USB Video", so `hdmi.sh` picks by id.

## What the firmware describes

`docs/engineering/pi5-acpi/dsdt.dsl` is the worproject v0.3 DSDT, decompiled from a dump
taken on the bench. The HAL prints the same inventory on every boot (`[ACPI] DSDT <name>
_HID <id>`):

- CPU0-3, the debug PL011 (`URT0`, ARMH0011), and three PCIe root complexes (PNP0A08),
  each with an `AMZN0001` resource device: the firmware borrows Graviton's ID for a
  non-standard ECAM.
- `RP1B`: RP1 as an ACPI0004 container. `PBAR` = 0x1F_0000_0000 (RP1's registers) and
  `PINT` = 0x105 (GIC INTID 261). Inside it, `XHC0` and `XHC1` (PNP0D10) at PBAR +
  0x200000 and + 0x300000, both on INTID 261, level, shared.
- `SDC0`, the card slot, at 0x10_00FF_F000, INTID 305: its _HID is a Method returning
  BRCM5D12, and its child `SDMM` is the card. `_INI` clears bit 3 of the GPIO data
  register at 0x10_7D51_7C04, which by its `_DSM` is the 3.3 V/1.8 V signalling switch.
- `SDC1` (BRCM5D12, compatible 80860F16: generic SDHCI) at 0x10_0110_0000, INTID 306:
  the Wi-Fi's SDIO host (child `WLAN`). The HAL tells the two apart by that child.
- No Ethernet: RP1's GEM is not described. The HAL publishes it anyway, as
  `VIBEOS\RP1GEM` at PBAR + 0x100000 on `PINT` (`find_rp1_gem`), and netstack drives it
  (`gem.rs`). What the firmware leaves, logged on every boot (`[NETSTACK] gem: RP1 at`):
  the Ethernet clocks on, GPIO 32 (the PHY's reset) unassigned with its pad's output
  disabled, `MSIX_CFG(6)` off, and no station address in the GEM. So the driver drives the
  reset itself and asks the VideoCore firmware for the board's MAC (mailbox at
  0x10_7C01_3880, property tag GET_BOARD_MAC_ADDRESS), the one Linux uses: VibeOS and the
  stager then share the bench board's DHCP lease (192.168.1.248 on 2026-10-08), and the
  stager still resolves as `vibeos-pi5` after a VibeOS boot. If the mailbox fails, the log
  says so and the driver falls back to `02:56:42:50:35:01`.
- RP1's interrupts reach the GIC as legacy INTA, not MSI-X. The firmware leaves RP1's MSI-X
  capability disabled (message control 0x3c, enable clear) and INTx enabled, and enables
  only sources 31 and 36 (the two xHCIs) in `MSIX_CFG`, without IACK. Every enabled source
  then drives INTA, which the BCM2712 root complex for RP1 (0x10_0012_0000) delivers as
  INTID 261: the level, shared line in the DSDT. So the GEM's interrupt is one write,
  `MSIX_CFG(6).ENABLE` (`gem.rs` `enable_interrupts`), done only once netstack's ISR is
  connected to 261; the ISR acks the GEM's own ISR register (write-1-to-clear: DCFG1
  IRQCOR = 0). Measured 2026-10-08 by dumping `MSIX_CFG`, the root complex's MSI registers
  and RP1's config space through the controller's index/data window (0x9000/0x8000).

The MCFG (OEM `RPIFDN`/`RPI5`) is not generic ECAM. It names bus 0 at 0x10_0010_8000,
inside the Broadcom controller's registers, and the first read there returns 0xDEADDEAD
and leaves an SError pending. The loader keeps that OEM on a quirk list and reports no
MCFG (`arm64_serial_acpi.rs`); before it did, every boot ended in BUGCHECK 0x80 at the
kernel's first `_enable`.
