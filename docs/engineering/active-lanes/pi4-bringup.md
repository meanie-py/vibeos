# Pi 4B physical bring-up — graviton-spike

> **2026-09-25: VibeOS boots to the desktop on a physical Raspberry Pi 4B.**
> Serial shows `[WIN32K] shell ready` and `[EXPLORER] _tWinMain entered` (the same
> marker the QEMU smoke gates on), with zero bugchecks. HDMI shows the taskbar,
> Start button and Notepad. Built from `cf73a8d8ba3`. Evidence: bench
> `first-desktop-20260925/` (serial.bin, manifest.json, hdmi-desktop.png). See
> "Session 3" at the end for the three bugs between the Phase-1 stop and this.

The owner changed this lane from a Graviton/KVM experiment to booting VibeOS
**directly on an existing Raspberry Pi 4B**. No AWS resources were created.
The immediate hardware milestone is FreeLDR entry and its menu; the eventual
success criterion remains the graphical desktop, then real SMP workloads.

## Measured firmware baseline (2026-09-20)

- pftf/RPi4 UEFI v1.53 boots from a 4 GiB FAT32 partition, MBR, on the owner's
  approximately 128 GB SD card. Firmware archive SHA256:
  `ca9973e2a7a546b3df871cfb7382e656829114b6dfa424f40dc67cc90a217d88`.
- Firmware serial and serial input work at 115200 8N1, no flow control. The
  FT232 adapter enumerates as COM5, USB VID 0403 / PID 6001, serial BG02D9GD.
  Its VCCIO jumper connects to 3V3. With the current cable orientation:
  black GND -> physical pin 6; yellow RXD -> pin 8 (Pi TX); orange TXD ->
  pin 10 (Pi RX). Those are consecutive outside-row positions 3, 4, 5,
  counting from the end away from Ethernet. Adapter VCCIO is unconnected.
- UEFI Boot Manager -> UEFI Shell works through serial. `map -r` finds FS0,
  and `ls` reads the installed firmware files from it.
- `acpiview -s SPCR`: table absent. `acpiview -s DBG2`: PL011 subtype 3,
  memory address **0xFE201000**, size 0x1000, 32-bit access, valid checksum.
- `acpiview -s APIC`: four enabled GICC entries, GICv2,
  distributor **0xFF841000**, CPU interface **0xFF842000**.
- `memmap`: 2991 MiB total reported, with the firmware's default 3 GiB cap.
- HDMI capture confirms the completed memory-map output and FS0 shell prompt.
  A splash screen alone did not prove a hang.

Evidence lives at `D:\home\vibeos-rpi4-setup` (WSL:
`/mnt/d/home/vibeos-rpi4-setup`): `serial-20260920-190237/serial.bin` is the
first readable firmware boot; `serial-20260920-190550/serial.bin` contains
interactive shell and ACPI evidence; `firmware-inventory.txt` is the decoded
inventory; `hdmi-current.png` is a direct capture-card frame.
The parser test fixture in `arm64_serial_acpi.rs` is the exact 97-byte DBG2
captured from this board, not a reconstructed idealized table.

## Loader work and outstanding hardware work

FreeLDR's live Rust serial/debug/panic paths wrote to QEMU's fixed 0x09000000.
The working change discovers a supported PL011/SBSA UART through ACPI SPCR or
DBG2 before normal loader initialization, preserves firmware UART configuration,
and reports failure through UEFI ConOut rather than guessing an MMIO address.
The loader's special UART mapping uses the discovered address too.

**This does not yet port the kernel to Pi hardware.** Remaining observed source
assumptions include kernel `Arm64UartKernelBase` initialized to 0x09000000,
AP bootstrap PL011 accesses, QEMU fw_cfg access, and fixed GIC identity mappings
in `winldr_arm64.rs`. Carry the platform description through the loader/kernel
handoff, audit early MM mappings and RAM assumptions, then implement/test the
physical interrupt/timer and device paths. Do not mistake loader success for
kernel success or replace the MP kernel with UP to make the result green.

The Pi-specific copy of freeldr.ini will use TimeOut=-1 for the first loader
inspection; QEMU media and its smoke criteria retain their normal settings.
Physical FreeLDR entry and menu are now verified. No kernel boot has been verified.

## Bench tools

The setup directory holds `Prepare-Rpi4Card.ps1` (one-time guarded formatting),
`Deploy-Vibeos.ps1` (guarded payload copy/readback, preserves UEFI), and
`Capture-RpiSerial.ps1`. `serial-current.json` identifies the capture process
and directory. A USB disconnect terminates the current capture loudly; reopen
it before power-on. Put bytes in `serial-send.bin` to send through the existing
capture process; wait for a shell prompt between commands to avoid firmware
input-buffer loss. Do not open a second reader on COM5.

The capture card enumerates as DirectShow **USB Video**. Camera holds it
exclusively; a gdigrab of its window returned black and was NOT HDMI evidence.
A direct FFmpeg dshow frame worked after gracefully closing Camera; Camera was
then reopened. FFmpeg is installed under the Windows user's WinGet packages.

Validation and the prepared payload manifest are recorded alongside the images
when complete. Hardware work resumes by copying the verified payload onto the
same card, reconnecting serial capture, and booting to the first loader marker.

## Resume here — physical loader menu reached (2026-09-20)

**The Pi is ON and sitting at the FreeLDR operating-system menu.** Direct HDMI
capture `D:\home\vibeos-rpi4-setup\hdmi-loader.png` shows two entries:
ReactOS ARM64 and **ReactOS ARM64 SMP kernel** (selected). No kernel entry has
been launched. Leave the Pi at this menu until the remaining hardware-address
work below is ready. This achieves the first physical-loader milestone.

Serial confirms `[freeldr] ACPI PL011 base=0x00000000FE201000`, MachInit,
UI, memory manager, filesystem, module list and boot devices initialized.
Last serial line is `Switching stack...`; **that is not a hang**: HDMI proves
the loader subsequently reached its menu. Snapshot of this power-on's serial
is `first-loader-boot-serial.bin` in the bench directory. Camera was reopened.
Firmware attempted network boot twice (`Could not detect network connection`)
before launching FreeLDR; the intervening splash delay was not a loader fault.

The final VibeOS loader payload was deployed and all three files passed SHA256
readback at 19:34. Evidence: bench `deployment-result.json`. Card is back in Pi;
H: is no longer an available deployment target. COM5 listener is still the
PowerShell process recorded below; verify it before use.

The initial deployment
refused before writing because its full-firmware hash check did not account for
UEFI saving variables in RPI_EFI.fd. This is resolved in Deploy-Vibeos.ps1:
validate the release-reference hash and compare every byte outside the documented
NV region `[0x3b0000, 0x3d0000)`, backing up the live firmware before deployment.
Observed changed bytes were only `0x3b0064..0x3b4539`, 17,462 bytes, with UEFI
variable records. The entire executable region matched the release byte-for-byte.
Source: [RPi4 firmware layout](https://github.com/tianocore/edk2-platforms/blob/master/Platform/RaspberryPi/RPi4/RPi4.fdf).

Card identity guard: Windows disk serial `121220160204`, USB, size 125671833600,
nonboot/nonsystem, H: FAT32 label `VIBEOS_RPI4`, partition size 4 GiB.
**Do not run Prepare-Rpi4Card.ps1 again.** Deploy only copies three payload files.

Current serial listener: Windows PowerShell PID 16280, COM5, directory
`serial-20260920-190550`. Recheck `serial-current.json` AND `Get-Process` before
assuming it is alive. Existing listener owns COM5; do not open a second reader.
PowerShell executable from WSL:
`/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe`.

### Validation and pending payload refresh

- Clean ARM64 build: `/tmp/vibeos-cli/20260920-191723-3jajq20u-build-arm64/summary.json`.
- ARM64 desktop smoke: `/tmp/vibeos-cli/20260920-191923-1mveg3_2-smoke-arm64/summary.json`,
  verdict OK. Durable copy, serial and screenshot in bench `arm64-smoke/`.
- Full gate: `/tmp/vibeos-cli/20260920-192056-vfzvd4g2-gate-i386+amd64+arm64/summary.json`.
  All three builds, desktop smokes, conformance, network smoke, tooling and difftest
  passed. Overall **FAIL LINT_FAIL**, solely a newly added OutputString.unwrap()
  in the UART-discovery error path. Fixed to `if let Some(output)`; source check
  `/tmp/vibeos-cli/20260920-193001-ku09bcir-check-all/summary.json` now OK.
  Do not call that earlier full gate green.
- Four host ACPI parser tests passed, including the exact Pi DBG2 captured bytes.
- Final clean ARM64 build after the error-path fix passed:
  `/tmp/vibeos-cli/20260920-192959-1gtrai2u-build-arm64/summary.json`.
  Final ARM64 desktop smoke **OK**:
  `/tmp/vibeos-cli/20260920-193137-bgg2biay-smoke-arm64/summary.json`.
  Screenshot inspected; durable summary/serial/screenshot in bench `arm64-smoke-final/`.
- The bench `payload/manifest.json` now describes the final verified payload,
  status VERIFIED_FOR_LOADER_TEST. Its BOOTAA64.EFI comes from
  `build-arm64/boot/freeldr/freeldr/uefildr.efi`, ramdisk.img from `qemu/ramdisk.img`.
  Pi freeldr.ini retains TimeOut=-1. Manifest records file lengths/hashes and provenance.
- Deployment completed with Deploy-Vibeos.ps1; deployment-result.json records
  the final hashes. Firmware and its saved variables were preserved. No reformat.
  The copy helper also now avoids attempting New-Item on an existing drive root.
- First target achieved: real UART discovery and physical loader menu.
  Next: kernel hardware-address audit; no kernel boot claimed.

### Working-tree changes and scope

No commit or push. New `arm64_serial_acpi.rs` and modified FreeLDR Rust files
implement ACPI UART discovery, early logging/panic routing, and loader UART mapping.
Two Solitaire resource files (`ro-RO.rc`, `sv-SE.rc`) had literal line breaks inside
quoted strings that broke both x86 builds; replaced those with escaped newline
characters, preserving the text. There are no kernel changes or paid translations.
Inspect `git status` and diff before continuing; never discard uncommitted work.
The bench has `loader-working-tree.patch` and a separate copy of the new parser;
refresh these when source changes. HEAD alone does not identify this payload.

### Direct capture-card recipe

Windows device is `video=USB Video`, VID534D/PID2109. FFmpeg executable:
`C:\Users\Lambda\AppData\Local\Microsoft\WinGet\Packages\Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe\ffmpeg-9.0.1-full_build\bin\ffmpeg.exe`.
Gracefully close only the ApplicationFrameHost window titled Camera, run FFmpeg
`-f dshow -i "video=USB Video" -frames:v 1 -update 1 -y
D:\home\vibeos-rpi4-setup\hdmi-current.png`, view that PNG, then reopen with
`Start-Process microsoft.windows.camera:`. Do not interpret a black gdigrab as
HDMI output. The user authorized inspecting this locally attached capture card.

### Physical work still owed

After the loader menu works, carry real platform addresses through the loader/
kernel handoff; replace early UART/GIC/QEMU fw_cfg assumptions; validate memory
maps, interrupts, timer and SMP on this board. Graphical desktop remains the goal.
Weak-ordering litmus work is still unmeasured and deferred until the physical
execution path is usable. Do not claim TCG cannot exhibit store buffering or
that this is the first native ARM64 execution merely from the original spike plan.

## Session 2 (2026-09-20 evening) — platform description, and the first real bug

### Firmware discovery now covers the GIC, and it is confirmed on the board

`arm64_serial_acpi.rs` gained MADT ("APIC") parsing for the GICv2 distributor
and CPU-interface bases, alongside the existing DBG2/SPCR UART discovery. On the
Pi the loader printed

    [FLDR] arm64 platform: UART=0x00000000FE201000 GICD=0x00000000FF841000 GICC=0x00000000FF842000

which matches acpiview's decode of this board exactly (distributor 0xFF841000,
CPU interface 0xFF842000). On QEMU the same code independently derives QEMU
virt's own 0x09000000 / 0x08000000 / 0x08010000 — i.e. it reproduces the values
that used to be hardcoded, which is the useful cross-check.

### The addresses are carried to the kernel instead of being constants

`ARM_LOADER_BLOCK` (the `_M_ARM64` branch only, so no other arch's layout moves)
gained `Arm64UartBase`, `Arm64GicDistributorBase`, `Arm64GicCpuInterfaceBase` at
offsets 56/64/72, size 56 -> 80. Regenerating with `tools/verify_offsets.py` and
`ntkrnl-sys/verify_sizes.py` reproduced those offsets independently of the
hand-edited `bindgen_arm64.rs`, which is what validates the hand edit.

Consumers that previously hardcoded QEMU virt:

- `ke/arm64/stubs.rs` — `Arm64UartKernelBase` now starts at **0** rather than
  0x09000000, and `KiSystemStartup` fills it from the loader block before its
  first debug write. The old default is not a harmless fallback: 0x09000000 is
  ordinary DRAM on this board, so every debug write would have silently
  corrupted memory instead of failing.
- `ex-rust/init.rs` — the Phase-1 `MmMapIoSpace` remap of the UART, and the GIC
  mapping feeding `HalInitializeClockArm64`, both use the loader-block values.
- `hal-rust/arm64/clock.rs` — `gic_ensure_early_bases` takes the loader block
  (`HalInitializeProcessor` already receives it) instead of `GICD_PA`/`GICC_PA`.
  Note the HAL cannot simply `extern` an ntoskrnl global: it does not link
  against one unless it is in the `.spec`, which is what LINK_FAIL reported.
- `tripwire.rs` — the arm64 fw_cfg probe is now gated. It **writes** a selector,
  and the x86 comment "real hardware floats the port" does not carry over to
  ARM MMIO; ungated it would corrupt DRAM or fault. fw_cfg has no ACPI
  description, so it is inferred from the discovered UART being QEMU virt's, in
  the loader and the kernel under the same rule.

### Open bug: the loader faults right after the page-table switch

Reached only because everything before it works: FreeLDR read the 140 MB ramdisk
off the SD card and loaded the entire boot-driver set (ntoskrnl, hal, wdf01000,
pci, acpi, the USB stack, fastfat, ndis, mup, ...) on real hardware.

    [FLDR] load ramdisk(0)\reactos\AppPatch\drvmain.sdb
    Synchronous Exception at 0x0000000037F7131C

Symbolicated against `__ImageBase` (printed as `Mmck[ib=...]`; it moves between
builds, so always re-derive) that is `winldr.rs`'s

    (*(*LoaderBlock).Extension).LoaderPagesSpanned = MmGetLoaderPagesSpanned();

the first store after `WinLdrSetProcessorContext()` switches TTBR0/TTBR1. The
Extension is reached through a `PaToVa` KSEG0 VA, and that VA is not mapped on
this board. QEMU's layout happens to get it mapped, so the bug has been latent.

**A hypothesis that the measurement killed:** that
`MempSetupPagingForRegion`'s `BasePage + PageCount > MmGetLoaderPagesSpanned()`
bail was skipping the region. The board says otherwise —
`LB=0x0B13C000 Ext=0xFFFFF8000B13C108 spanned=0x37FF4` — page 0xB13C is well
inside the span. `LoaderSystemBlock` is also in the kernel-mapped type list. So
neither the span gate nor the memory type explains it.

**Why this was invisible:** `ERR()`/`TRACE()` are `#if DBG` and compile to
nothing in the release build that ships to hardware, so a failed
`MempMapSinglePage` produces no output at all. Only raw `DbgPrint`/`dbgcon`
reaches the board — anything meant to be legible on hardware must use those.

### Next diagnostic, built and QEMU-verified, waiting on hardware

`Arm64ReportMapping` walks the live tables for one VA and prints every level, and
`Arm64MapFailures` counts silent `MempMapSinglePage` failures. Both are reported
just before the faulting store. On QEMU:

    [FLDR] pre-spanned: LB=0x...42138000 Ext=0xFFFFF80042138108 spanned=0x7CB44 mapfail=0x00000000
    [FLDR] ext-ident va=0x...42138000 L00[00000000]=0x...703 L01[1] L02[10] L03[138] MAPPED
    [FLDR] ext-kseg0 va=0xFFFFF80042138108 L00[000001F0]=0x...703 L01[1] L02[10] L03[138] MAPPED
    [FLDR] spanned stored

Run this on the Pi and the level that reports INVALID is the answer.

### Bench changes

- `Serve-Tftp.py` — read-only TFTP server, run with **Windows** python
  (`C:\Users\Lambda\AppData\Local\Programs\Python\Python313\python.exe`), not
  WSL's: WSL2 is NAT'd here (172.26.x.x) and the Pi cannot reach it. The host is
  192.168.1.183 on "Ethernet", the same subnet the Pi DHCPs into.
- `Add-TftpFirewallRule.ps1` — one narrow inbound rule (UDP, program-scoped to
  that python.exe, remote address 192.168.1.0/24). Needs elevation; the owner
  approved it. Remove with `Get-NetFirewallRule -DisplayName 'VibeOS Pi4 TFTP
  (bring-up)' | Remove-NetFirewallRule`.
- Ethernet is now connected. The firmware tries PXE v4/v6 then HTTP v4/v6,
  ~2 minutes of timeouts, before falling through to FreeLDR. Press ESC early to
  skip it. PXE-booting the loader itself does NOT help: FreeLDR resolves
  `freeldr.ini` and `RDPATH=\ramdisk.img` on its own boot device, which a network
  boot does not provide. The intended loop is the UEFI Shell's `tftp` writing
  onto FS0 so the card never leaves the board — **not yet verified**; whether
  this firmware's Shell ships `tftp`/`ifconfig` is still unknown.
- The Pi's `freeldr.ini` now defaults to the **UP** entry. Deliberate, and only
  on this card: early SMP programs the GIC through the boot identity map before
  `MmInitSystem(1)`, so it would fail first on a board whose GIC we have only
  just started discovering, masking whatever the UP kernel hits. SMP here is
  still the goal; QEMU media keeps MP as its default.
- Loader-only iterations need just the 300 KB `BOOTAA64.EFI`; the 140 MB
  ramdisk only changes when kernel-side sources do.

### Validation

Full `vibeos-cli gate` green on i386+amd64+arm64 (builds, all three desktop
smokes, net smoke, conformance, difftest, tooling, lint) at
`/tmp/vibeos-cli/20260920-200513-5vt4_h74-gate-i386+amd64+arm64/`. The six host
ACPI parser tests pass, including the captured Pi DBG2 bytes and the new MADT
cases. The page-walk diagnostic added after that gate passed its own arm64
build + desktop smoke (`20260920-202711-wkispj1l-smoke-arm64`, marker reached).

## Session 2, part 2 — the kernel runs on the Pi and paints the screen

Four bugs, each one only reachable after the previous was fixed. All four were
invisible on QEMU for the same underlying reason: QEMU virt's firmware, memory
layout and UART model differ from this board in ways our code had silently
baked in as assumptions.

### 1. The loader programmed a translation regime it was not executing in

**Symptom.** A store through a PaToVa KSEG0 VA faulted while a walk of that exact
VA reported every level MAPPED and `MempMapSinglePage` had never failed.

**Cause.** `WinLdrSetProcessorContext` programs `MAIR_EL1`/`TCR_EL1`/`TTBR0_EL1`/
`TTBR1_EL1` and `tlbi vmalle1`, and never enables the MMU - correct when firmware
hands off at EL1 with the MMU already on, which is what QEMU does. Measured on
this board:

    [FLDR] ctx-pre EL02 SCTLR_EL1=0x30D00800 HCR_EL2=0x08000000 E2H=0   (Pi)
    [FLDR] ctx-pre EL01 SCTLR_EL1=0x30D0198D                            (QEMU)

EL2 with `HCR_EL2.E2H == 0`: no VHE, so those `_EL1` writes are inert for the
regime we execute in, and firmware's EL2 identity map stayed live. Tables that
are correct but unused - identity worked, KSEG0 never could.

Note `TTBR1_EL1` read back equal to our PGD the whole time. **That is not
evidence**: at EL2 without VHE the register holds what you wrote, it just is not
what translates. The exception level is the thing to measure.

**Fix.** `eret` to EL1h first, then program EL1 and enable `SCTLR_EL1.M|C|I`.
EL1 is where the kernel must run regardless. Both steps are conditional on the
measured state, so the QEMU path is untouched.

### 2. Dropping to EL1 made every later fault silent

Firmware's exception handler is EL2's, so after the drop a fault landed on
whatever `VBAR_EL1` held and the board simply stopped - the
`Synchronous Exception at 0x...` line that had located every bug disappeared
exactly when it started mattering most. The loader now installs minimal EL1
vectors reporting ESR/FAR/ELR/SPSR/EC until the kernel installs its own.

### 3. The kernel's UART dropped bytes on a real PL011

First kernel output came back as `MiIniti`, `MiBuild` - traces shredded
mid-word, which reads like a crash at that point rather than a serial fault.
The kernel wrote the PL011 data register without ever checking `UARTFR.TXFF`;
QEMU's model drains instantly so nothing was ever lost there, a physical PL011
at 115200 overruns immediately. `arm64_dbg`, `trap.rs`'s `TrapPutChar` (the
fault reporter - a shredded fault dump is worse than useless), `kiinit` and
`mminit` now share one FIFO-aware `uart_putc` with a bounded spin. Fixing this
was what made the next bug readable at all.

### 4. PFN database init faulted reading a PDE in unbacked address space

    [FAULTLOOP] VA=0xFFFFF6FB7E002FD8 PC=0xFFFFF8000B1A5264
                ESR=0x96000007 DFSC=0x07   (translation fault, level 3)

`MiInitializePfnDatabase` (the PC decodes to an `lsr #18` + mask + `orr` -
`MiAddressToPde`, not Pte) reading the PDE for kernel VA `0xFFFFF800BF700000`,
i.e. physical `0xBF700000` = 3063 MiB.

Reading a PDE needs only the TABLE containing it to exist; an absent entry reads
as zero, which the kernel copes with. QEMU's RAM fits inside a single 1GB L1
entry whose L2 table the loader's own allocations had already created, so every
PDE read landed somewhere valid. The Pi's 3GB spans three L1 entries and the
loader only allocates in the first (`LoaderPagesSpanned` = 895 MiB against a
3072 MiB RAM top), so two of the three L2 tables did not exist.

**Fix:** pre-create the L0/L1/L2 chain across `KSEG0 + [0, MmHighestPhysicalPage)`.
Three pages per GB, rather than the ~2MB per GB that mapping every leaf costs,
and it does not change which pages the kernel inherits as mapped.

**Worth recording as a reasoning error:** this span gate was the FIRST hypothesis
for bug 1, and the board's own numbers correctly killed it there (page 0xB13C is
well inside the span). It was the right mechanism attached to the wrong victim.
The measurement was sound both times; the conclusion drawn from it was too broad.

### Where it stands now

    [KERNEL] KiSystemStartup entered!
    [KERNEL] Calling ExpInitializeExecutive
    [MM] MmArmInitSystem ... returning TRUE
    [KERNEL] ExpInitializeExecutive returned!
    [KERNEL] Entering KiIdleLoop
    [PHASE1] ObInitSystem
    [MM1] MiCreateMemoryEvent ... MiInitializeMemoryEvents done     <- serial stops

and HDMI (`hdmi-phase1.png`) shows **ReactOS's blue bootvid background**, so the
kernel is alive past the point serial goes quiet. A capture 60s later is
byte-identical, so it is wedged, not merely slow. **No desktop. This is not a
boot.**

### Next, in order

(Superseded 2026-09-25: see "Session 3" below. Item 1's premise was wrong.)

1. **Why serial dies at the Phase-1 UART remap.** Output stops within
   `MmInitSystem(1)`/the `MmMapIoSpace` remap of the PL011 in `ex-rust/init.rs`,
   and `[PHASE1] UART remapped to kernel VA` never appears. Cheapest probe: skip
   the remap on arm64 and keep the identity-mapped physical base (temporary - the
   identity map dies at SMSS) and see whether Phase-1 output continues. Suspects:
   `MmNonCached` giving Normal-NC where a PL011 wants Device-nGnRnE, or the new
   FIFO wait spinning on an `FR` read that no longer reaches the device (a
   bounded 1,000,000-iteration spin per byte would look like silence).
2. **bootvid stride.** The blue field is horizontally streaked and occupies a
   sub-rectangle, so bootvid's assumed resolution/pitch does not match this
   board's GOP mode. Carry the real framebuffer geometry through like the UART
   and GIC.
3. **Then the actual wedge**, once 1 restores visibility.

Do not read the blue screen as "almost booting" - it is one early bootvid paint,
not the desktop, and the desktop marker remains the only success criterion.

### Loop mechanics (no card swaps)

`tools/pi4-netboot.sh` drives a whole cycle: catch the firmware splash (a ~5s
window; missing it costs ~90s of PXE/HTTP timeouts and then boots whatever stale
loader is on the card), enter the UEFI Shell over serial, DHCP, TFTP the fresh
loader onto the card, launch it from `fs0:`, and pick the menu entry. Only the
power cycle is manual. `--ramdisk` also pushes the 140MB image, which takes
~10 minutes; skip it unless kernel-side sources changed, since the loader alone
is 300KB and lands in seconds.

`tools/pi4-serial-send.sh` paces input by chunk. Sending a command all at once
silently loses characters - `freeldr.ini fs0:\tftp-test.tmp` arrived as
`freelt.tmp` and RAN, against the wrong filename. It also writes-then-renames,
because a partially written `serial-send.bin` is a sharing violation that killed
the capture listener mid-session; `Capture-RpiSerial.ps1` now retries instead of
dying.

Serving side: `Serve-Tftp.py` under **Windows** python (WSL2 is NAT'd at
172.26.x.x and the Pi cannot reach it; the host is 192.168.1.183 on "Ethernet",
the Pi DHCPs to 192.168.1.216). One narrow firewall rule, program-scoped, remote
address 192.168.1.0/24 - remove with
`Get-NetFirewallRule -DisplayName 'VibeOS Pi4 TFTP (bring-up)' | Remove-NetFirewallRule`.

Still worth doing: make the UEFI Shell boot option #1 via `bcfg` and drop a
`startup.nsh` that does the DHCP/TFTP/launch, so a power cycle alone pulls the
latest build. And the BCM2711 watchdog (`PM_WDOG`/`PM_RSTC` at 0xFE100024 /
0xFE10001C, password 0x5A000000) maxes at ~16s, so it can only be armed right
before a known-risky step and disarmed at kernel entry - long enough to turn
"halts forever" into "reboots", not long enough to cover a 3-minute ramdisk load.

## Session 3 (2026-09-25) — the bench runs itself; the "last line" was not the last line

### Bench

The bench is now driven entirely from the host: a Shelly smart plug switches the
Pi's power (`tools/pi4/power.sh`), so `tools/pi4/netboot.sh` does power-cycle ->
splash -> Shell -> TFTP -> boot with nobody at the bench. All bench tooling moved
into the repo under `tools/pi4/`. The serial listener and TFTP server used to
exist only on the bench machine's D: drive. The standing how-to is
`docs/engineering/pi4-bench.md`; this file keeps the findings.

Current master (1,540 commits past session 2) reproduced the session-2 stop
exactly on the first hands-free boot.

### Serial output was being truncated, not stopping

The raw bytes around the Phase-1 stop read

    [MM] MmArmInitSystem returning TRUE\r\n[E[\n[[[KERNEL] ExpInitializeExecutive returned!
    [PHASE1] ObInitSystem\r\n[PHASE1] ExI[[[[[[[MM1] MiCreateMemoryEvent LowMem

Each `[` is a whole trace line cut to its first byte. `ARM64_UART_TRACE`, which
every ex-rust `uart_trace!` goes through, stored bytes straight into the PL011
data register. Session 2's FIFO fix covered `kiinit`/`trap` only. The same bare
store had 87 private copies in 13 files across ntoskrnl, iomgr, se, ps and lpc.
All of them now go through `Arm64UartPutChar` (exported from `arm64_dbg.rs`),
and `tools/scan_raw_uart_writes.py` gates new ones (baseline empty; it finds
all 87 on the old tree).

So session 2's "serial stops at `MiInitializeMemoryEvents done`" located
nothing. Whole Phase-1 lines were being lost, and the remap-failure theory
(next step 1 above) was built on that. The hang location has to be re-read
from a FIFO-safe boot.

### Other QEMU-virt constants still live (not yet on the boot path)

- `hal-rust/src/arm64/mpboot.rs`: the AP trampoline writes a trace byte to
  physical `0x09000000`, which is RAM on a Pi. That only matters for the MP
  entry; the Pi `freeldr.ini` defaults to UP.
- `pl011-rust/src/pl011.rs`: `PL011_PHYS_ADDR = 0x09000000`.

### The cache bugs: from the Phase-1 stop to the desktop

With FIFO-safe output the real stop was visible: serial died at the first byte
written through the Phase-1 `MmMapIoSpace` remap of the UART.

1. **`MmNonCached` mappings were cached.** freeldr programs `MAIR_EL1 =
   0x004400FF` (slot 0 Normal-WB, 1 Device-nGnRnE, 2 Normal-NC), and
   `MI_PAGE_DISABLE_CACHE` wrote slot 0 under a comment saying "device". So the
   UART and the GIC were mapped as cached RAM, and stores never reached them.
   This was the third copy of the same bit-swap: `PTE_DISABLE_CACHE` and the
   `stubs.rs` protection table had been fixed earlier. Fixing it took the board
   through the registry, I/O init, and the launch of SMSS into user mode.
2. **Image pages were not visible to instruction fetch.** smss died with
   `STATUS_ILLEGAL_INSTRUCTION` (ESR EC=0) at the first `stp` of
   `RtlSetUnhandledExceptionFilter`. That is a zero page decoding as UDF: the
   section page-in wrote ntdll's code through the D-cache and never cleaned it
   to the point of unification. `MmMakeSegmentResident` now sweeps image pages.
   **That fix alone reached the desktop.**
3. Siblings of 2, fixed in the same pass and landed after the desktop boot:
   `KeSweepICache(NULL, 0)` now cleans the D-cache by set/way (the loader's
   post-relocation flush was I-cache-only), and `MmLoadSystemImage` sweeps a
   driver before running it.

All three are invisible to QEMU TCG, which models no caches. Every one of the
seven Pi bugs so far has been a hardware property QEMU does not have (EL2 entry,
FIFO depth, memory above one L1 entry, EL1 vectors, cacheability, I/D
coherence), so treat "passes on QEMU" as necessary and nowhere near sufficient
for arm64.

### Netboot speed, measured

The ramdisk takes ~13.5 minutes whatever the TFTP framing: 825 s lockstep,
805 s with a 32-block window, 804 s with 8 KB blocks. The explanation first
written here (EDK2 polling the NIC once per ~10 ms) was WRONG: session 4
measured iPXE pulling the same file into RAM in 18.9 s, and copying it from
RAM to the card taking as long as the TFTP did. The limit is SD card writes
through this firmware. See `docs/engineering/pi4-bench.md`.

### Next (superseded by session 4 below)

- Re-verify on the board with the item-3 sweeps (not yet run on hardware).
- bootvid geometry: the boot splash is streaked and undersized, but the desktop
  (win32k on the GOP framebuffer) renders correctly.
- Input: USB on the Pi 4 is xHCI behind PCIe, which we have no driver for.
- MP: the AP trampoline in `hal-rust/src/arm64/mpboot.rs` still writes to
  `0x09000000`, which is RAM on a Pi.

## Session 4 (2026-09-26 overnight) — the ~70 s freeze was the idle loop's WFE

**Fixed and verified on the board.** The desktop froze about a minute after it
appeared, on every boot: tray clock stopped, serial silent, no bugcheck. With
the fix the clock advances (12:02 -> 12:05 AM captured over HDMI) and the tick
ran past 35000 (~6 min) with the CPU in the Idle thread.

**Cause.** The idle loop masks IRQs and, with nothing ready, ran `wfe`. A masked
IRQ is not a WFE wake-up event (it is one for WFI), and nothing in the kernel
issues `SEV` or enables the timer event stream, so the core slept forever with
the tick pending. QEMU's TCG treats WFE as a yield, so smoke could never see
it. It struck at ~70 s because the zero-page thread keeps the CPU busy until
all free RAM (~3 GB here) is zeroed: the first truly idle moment was fatal.
Fix: `wfi` (`ke/arm64/thrdini.rs`). Field manual: "Real hardware: what QEMU's
TCG does not model".

**How it was found, since the obvious suspects were all wrong.** Every death
followed an lsass worker's exit, and a switch-narration probe showed that
switch completing fine. A post-swap trap trace found explorer busy in
`RtlDosSearchPath_U` (gdi32 retrying the missing `lpk.dll` on every text draw:
real, fixed separately, not the hang). A raw-UART tick heartbeat in
`KeUpdateSystemTime` stopped at tick 7000 every time. A tick-health check sampled
from IRQ entry, user-mode traps and the idle loop reported nothing: the CPU was
in none of those places, which left a sleeping core. The probe set is kept
outside the tree as a reusable patch:
`<bench>/probes/pi-idle-hang-probes-20260926.diff`.

Wrong turns worth not repeating: a "reproduced in QEMU" that was the reaper
killing a `debug start` VM when the tool call that launched it exited (pass
`VIBEOS_QEMU_OWNER_PID`), and reading a `DbgPrint` heartbeat's silence as
"interrupts died" (DbgPrint can go quiet on its own; use raw UART).

### Also this session

- `vibeos-cli build --up-kernel` failed at mkramdisk since the CRLF
  `freeldr.ini`; fixed. The Pi boots the UP kernel, which nothing else tests.
- `lpk.dll` is now staged in the arm64 ramdisk.
- Netboot: the card write is the limit (above). `netboot.sh --sanboot` (iPXE +
  `serve_http.py`) boots with no card writes; proven in QEMU, needs one TCP
  firewall rule on the bench host (`pi4-bench.md`).

### Next

- Owner: create the sanboot firewall rule, then make `--sanboot` the default loop.
- bootvid geometry (splash streaked and undersized; the desktop is fine).
- Input: xHCI behind PCIe; nothing drives it yet.
- MP on the board: the AP trampoline (`hal-rust/src/arm64/mpboot.rs`) still
  writes `0x09000000`, which is RAM on a Pi.
