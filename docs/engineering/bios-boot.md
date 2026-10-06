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
SeaBIOS ─▶ isoboot.bin (El Torito, no-emulation) ─▶ freeldr.sys ─▶ rosload.exe ─▶ ntoskrnl
            boot/freeldr/bootsect/isoboot.S          │
                                                     ├─ frldr16.bin  16-bit stub at 0xF800 (arch/realmode/amd64.S)
                                                     └─ freeldr_pe   long-mode loader body at 0x10000
```

- **The ISO is hybrid.** It has a BIOS El Torito entry (primary) and a UEFI one
  (`efisys.bin`, alternate), plus an isohybrid MBR (`isombr.bin`), so the same file
  boots on either firmware from CD, or `dd`'d to a USB stick
  (`boot/boot_images.cmake`). arm64 stays UEFI-only.
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
- **`freeldr.sys` is not a PE.** It is the stub with the PE appended, so the
  post-link scanners judge `freeldr_pe.exe` instead (`tools/binary_corpus.py`
  `CONTAINERS`).
- **amd64 only.** i386 would also need `drvmap.S`, which is 16-bit code only GNU
  as assembles, and i386 is on the deprecation track (roadmap §9).

## Running it

```bash
./vibeos-cli build --arch amd64
./vibeos-cli smoke --arch amd64-bios     # SeaBIOS, same media as amd64
```

Every gate that includes amd64 also boots the media under SeaBIOS
(`gate.py` `BIOS_LANE`). The QEMU config is `Amd64BiosConfig` in `tests/qemu/vm.py`.
It uses `-vga std` (Bochs VBE), not cirrus.

## Real hardware

QEMU's SeaBIOS is the oracle here. A real machine adds its own VGA option ROM, its
own EBDA size, and USB or SATA boot media. Expected first problems on the 9020:

- whether its Intel VBE ROM offers a 32-bpp linear mode in the preferred list
- USB-stick boot through the isohybrid MBR, which then needs USB mass storage in the
  kernel (roadmap 10b)

`docs/engineering/modern-windows-compat-roadmap.md` describes the bench setup
(remote power, serial console, network boot) that makes the box agent-friendly.
