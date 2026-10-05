# Raspberry Pi 5 bench

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

VibeOS reached the desktop on the Pi 5 on 2026-10-04, booted from this card.

## How a build gets there: the Linux stager and tryboot

The Pi 5's UEFI is worproject's rpi5-uefi v0.3. It is archived, and that release is
its last. Its network stack has no driver for the board's Ethernet, because the
Ethernet sits behind the RP1 I/O chip. So there is no netboot, and the SD card
carries two systems in one FAT partition (`tools/pi5/card/config.txt`):

- **A normal boot starts Raspberry Pi OS: the stager.** It owns the Ethernet.
  cloud-init sets it up as `bench@vibeos-pi5`, with key-only SSH using the host's
  `~/.ssh/id_ed25519`, and masks `rpi-eeprom-update`.
- **`pi5 boot` copies this tree's files and reboots once into VibeOS.**
  `tools/pi5/stage.sh` copies `BOOTAA64.EFI`, `freeldr.ini`, the ramdisk and
  `config.txt` over SSH and checks each by SHA-256 on the card. It then runs
  `reboot '0 tryboot'`. The firmware applies `[tryboot]` for that one boot: UEFI as
  the armstub with its own device tree, then VibeOS.
- **Any later reset is Linux again.** With `PI5_PLUG` set, `pi5 boot` power-cycles a
  board that is still in VibeOS or hung.

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

- **Serial** comes from the 3-pin JST-SH debug connector, wired to picousb's GP0, GP1
  and GND. That port is the UART UEFI's ACPI tables name, so the firmware, UEFI, the
  loader and the kernel all print there with no configuration.
  - Start the listener with `tools/pi5/serial.sh up`.
  - **With the listener, `pi5 boot` waits for `shell ready`** like `pi4 boot`.
  - **Without it, the verdict is HDMI-only and passes only as a warning:** the
    desktop background has to fill at least 20% of the frame.
  - GPIO14/15 belong to RP1, which neither UEFI nor VibeOS sets up yet.
- **Plug:** a Shelly plug named by `PI5_PLUG`. `tools/pi5/power.sh` refuses to run
  without it, because `tools/pi4/power.sh` would otherwise fall back to the Pi 4's
  plug.
- **Pico:** `picousb` is the Pi 5's (`picousb5` is the Pi 4's). Use
  `tools/pi5/pico.sh` for input.
- **Capture:** the Pi 5's card has USB vendor id 345f and the Pi 4's has 534d. Both
  are named "USB Video", so `hdmi.sh` picks by id.
