# Legacy BIOS boot (amd64)

amd64 boots from legacy BIOS as well as UEFI, from the same hybrid ISO. It exists
for machines whose firmware has no UEFI, first of all the homelab OptiPlex 9020 running
Libreboot with a SeaBIOS payload.

BIOS boot was pruned on 2026-05-29 ("BIOS prune 1–5/N", plus the boot-sector prune
`4c45deeac12`) and restored on 2026-10-05. While it was gone, nothing booted it, so
nothing noticed when the code it depended on moved on. It is now a gate lane
(`smoke:amd64-bios`), so it cannot go stale silently again.

## The chain

```
SeaBIOS ─▶ [USB stick: isombr.bin ─▶] isoboot.bin ─▶ freeldr.sys ─▶ rosload.exe ─▶ ntoskrnl
            [CD: El Torito, no-emulation]            │
                                                     ├─ frldr16.bin  16-bit stub at 0xF800 (arch/realmode/amd64.S)
                                                     └─ freeldr_pe   long-mode loader body at 0x10000
```

- **The ISO is hybrid.** It has a BIOS El Torito entry (primary) and a UEFI one
  (`efisys.bin`, alternate), plus an isohybrid MBR (`isombr.bin`), so the same file
  boots on either firmware from CD, or `dd`'d to a USB stick
  (`boot/boot_images.cmake`). arm64 stays UEFI-only.
- **From a USB stick** SeaBIOS runs the isohybrid MBR, which reads isoboot from the
  stick and enters it at its hybrid entry point. freeldr then sees a hard disk and
  boots `multi(0)disk(0)rdisk(0)partition(1)`: the MBR's one partition (type `0x96`)
  starts at sector 0 and covers the whole ISO. In the kernel, `usbstor.sys` is a boot
  driver and cdfs mounts the ISO9660 volume on `\Device\Harddisk0\Partition1`.
  Nothing in that chain was written for it: it is upstream's isohybrid design meeting
  the record/replay lane's disk-held-ISO kernel fixes (`qemu-record-replay.md`). UEFI
  boots the same stick through `cdrom(0)` instead.
- **`freeldr_pe` is all Rust:** `freeldr-rust` built *without* the `uefi`
  feature, which selects the PC-AT machine (`machpc`, `pcmem`, `pcdisk`,
  `pcvideo`, `pcvesa`, `hwacpi`…) and `BootMain`. Like `uefildr.efi` and
  `rosload.exe`, its final link is driven by cargo
  (`sdk/lib/rust/freeldr-cargo`, `ModuleKind::BiosLoader`). The 16-bit stub
  and the entry/BIOS-call asm are assembled by cmake (`boot/freeldr/freeldr/pcat.cmake`).
- **The BIOS loader exports `freeldr.spec`.** `rosload.exe` imports its loader
  services from `freeldr.sys` by name. `uefildr.efi` has no exports because it
  calls the boot manager directly and never loads rosload.

## Things that bite

- **More than ~1 GB of RAM.** The BIOS loader identity-maps 1 GiB (mm.h's x64
  `MM_MAX_PAGE_LOADER` is `0x3FFFF` for that reason); the UEFI loader maps 4 GiB. The Rust
  kept 4 GiB for both, so the page lookup table went to the top of RAM below 4 GiB and the
  BIOS loader triple-faulted at its first write into it, debugcon ending `C1234`. QEMU with
  `-m 1G` booted and `-m 1536M` did not; every smoke used 512 MB. Fixed 2026-10-08 by keying
  the constant on the `uefi` feature. Real BIOS machines have far more than 1 GB.
- **The size ceiling.** The whole loader image, `.bss` included, must fit below
  `MEMORY_MARGIN` (`x86common.h`: `MAX_FREELDR_PE_SIZE` = 0x77000, about 476 KB). Above
  it sits the INT 13h bounce buffer, which must stay in conventional memory under the
  EBDA. On restoration the image was 0x94800. Two fixes brought it to 0x6FE00:
  - cnt-rust's environment table was 111 KB of `.bss`. It is now a token table under
    `cnt-rust-lib`'s `freeldr` feature (`small-env`).
  - The body is built `opt-level=s`.

  The loader verifies its own header at startup and hangs right after the `fh[...]`
  debugcon breadcrumb if it is too big. If that comes back, measure `SizeOfImage`
  first.
- **BIOS calls may clobber GDTR.** QEMU's SeaVGABIOS drops into 32-bit mode to clear
  video memory on a VBE mode set, builds that GDT on our 16-bit stack, and returns
  with GDTR still aimed at it. `ExitToLongMode` now reloads our GDT; without that, the
  far jump into long mode triple-faulted at `0xFCC1`.
- **The display is a VBE linear framebuffer.** Since May the kernel's display path is
  framebuffer-only (lfb bootvid plus the framebuffer miniport), and it needs a
  `DisplayController` node carrying `CM_FRAMEBUF_DEVICE_DATA`. Without one,
  win32k bugchecks `0xB4` (no `DEVICEMAP\VIDEO`).
  - During hardware detection the BIOS loader picks a 32-bpp direct-colour VBE 2.0+
    mode with a linear framebuffer. It prefers 1024×768, then 800×600, then the
    largest mode up to 1920×1200 (`pcvideo.rs` `PcVideoSelectVbeFramebuffer`).
  - It describes that mode exactly as `uefihw.rs` describes GOP (`machpc.rs`
    `CreateFramebufferDisplayKey`).
  - It switches into the mode only in `PcVideoPrepareForReactOS`, so the loader's
    text UI keeps working until the kernel takes over.
- **isoboot prefers an installed OS, when booted as a CD.** (From a stick, the stick
  is drive 80h and the hybrid entry boots it directly.) If the first hard disk (drive
  80h) has a valid MBR, isoboot prints "Press any key to boot from the ReactOS medium" and, after five
  seconds with no key, boots the hard disk. That is upstream's intended behaviour for
  a person, and it is what the 9020 will do if its disk is bootable: press a key. The
  BIOS smoke therefore attaches no MBR temp disk (`vm.py` `temp_disk_args`). The
  disk path is covered by `amd64-uefi`, which boots the same media with that disk.
- **`freeldr.sys` is not a PE.** It is the stub with the PE appended, so the
  post-link scanners judge `freeldr_pe.exe` instead (`tools/binary_corpus.py`
  `CONTAINERS`).
- **amd64 only.** i386 would also need `drvmap.S`, which is 16-bit code only GNU
  as assembles, and i386 is on the deprecation track (roadmap §9).

## Running it

```bash
./vibeos-cli build --arch amd64
./vibeos-cli smoke --arch amd64-bios     # SeaBIOS, same media as amd64, on a USB stick
VIBEOS_QEMU_BOOT_TRANSPORT=usb-ehci ./vibeos-cli smoke --arch amd64-bios   # stick behind EHCI
VIBEOS_QEMU_BOOT_TRANSPORT=ide-cd   ./vibeos-cli smoke --arch amd64-bios   # El Torito CD
```

All three reach the desktop (2026-10-05). The gate lane boots the xHCI stick because
that is how the OptiPlex gets the ISO, and it covers more than the CD: the hybrid MBR,
freeldr's hard-disk path, and usbstor as a boot driver. The CD entry is not gated.

Every gate that includes amd64 also boots the media under SeaBIOS
(`gate.py` `BIOS_LANE`). The QEMU config is `Amd64BiosConfig` in `tests/qemu/vm.py`.
It uses `-vga std` (Bochs VBE), not cirrus.

The lane also has an NVMe drive in the box, as the OptiPlex does (`smoke` turns
`--nvme` on for `amd64-bios`; `VIBEOS_QEMU_NVME=0` opts out). SeaBIOS then sees two
hard disks, the stick stays BIOS drive 80h (`rdisk(0)`) because it boots first, freeldr
loads `nvme.sys` as a boot driver, and the boot fails unless the namespace is published.

## Real hardware

QEMU's SeaBIOS is the oracle here. A real machine adds its own VGA option ROM, its
own EBDA size, and USB or SATA boot media. Expected first problems on the 9020:

- whether its Intel VBE ROM offers a 32-bpp linear mode in the preferred list
- which controller serves the stick. Intel 8-series chipsets route their ports to EHCI
  until the OS sets `XUSB2PR`/`USB3_PSSEN` (Linux's `usb_enable_intel_xhci_ports`), and
  `usbxhci` does not, so expect the stick on EHCI unless coreboot already routed the
  ports. Both controllers boot the stick in QEMU.

`docs/engineering/modern-windows-compat-roadmap.md` describes the bench setup
(remote power, serial console, network boot) that makes the box agent-friendly.
