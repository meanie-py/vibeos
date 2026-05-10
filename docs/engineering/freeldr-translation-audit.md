# FreeLDR + Boot DLLs C→Rust Translation Audit

Last updated 2026-05-08. **Status: pre-translation.** No Rust code
exists yet for any of these targets; this audit defines the keep-set,
the prune-set, and the forward path.

**Scope:** the three pre-kernel components that FreeLDR loads
alongside ntoskrnl + HAL before transferring control to
`KiSystemStartup`:

1. **FreeLDR** itself (`uefildr.efi`) — the bootloader binary
2. **`kdcom.dll`** — kernel debugger COM transport (~933 LOC)
3. **`bootvid.dll`** — boot-time video / framebuffer (~1,400 LOC keep-set)

Bundled into one audit because all three are pre-kernel siblings with
small ABI surfaces, similar build shapes, and the same UEFI-only +
prune-arch-variants strategy applies to each.

Companion to:
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — the kernel translation, ~99.7% done
- [hal-translation-audit.md](hal-translation-audit.md) — the HAL translation, pre-translation

FreeLDR is the bootloader that runs *before* ntoskrnl. It loads the
kernel + HAL + kdcom + bootvid + boot drivers, sets up early memory,
parses the registry hive, and jumps into `KiSystemStartup`. The Rust
translation of these components is independent of the HAL/kernel work
— different crates, different target specs, different runtime — but
follows the same methodology.

## TL;DR

- **Go UEFI-only.** Drop the PC-AT/BIOS path entirely. arm64 already
  has only this option; switching i386/amd64 smoke to OVMF aligns all
  three arches on one boot path. **~9,500 LOC dropped.**
- **Drop NTFS, EXT, btrfs, PXE filesystems.** We boot from ISO9660
  (livecd) and FAT32 (UEFI ESP / arm64 ramdisk). Everything else is
  installed-OS scenarios we don't smoke. **~4,400 LOC dropped.**
- **Drop dead arch trees and the SARCH variants.** `arch/arm/` (314),
  `arch/i386/xbox/` (1,572), `arch/i386/pc98/` (2,886). **~4,800 LOC
  dropped.**
- **Drop bootsect/.** With BIOS path gone, the FAT/NTFS/btrfs/ext
  bootsectors and ISO MBRs all go. **~1,400 LOC asm dropped.**
- **Drop alternate KD transports.** Keep `kdcom.dll` only; drop
  `kdgdb`, `kdvm`, `kd1394`, `kdusb2`, `kdrosdbg` sibling DLLs that
  our `/DEBUGPORT=COM1` boot config never selects.
- **Drop bootvid arch variants.** Keep `framebuf/` (UEFI GOP) only;
  drop `arm/`, `i386/{pc,xbox,pc98}/` variants. UEFI-only means all
  three arches share one framebuffer path. **~3,000 LOC dropped.**
- **After pruning, total source under translation: ~30K LOC** —
  ~28K FreeLDR + ~933 kdcom + ~1,400 bootvid. Three Rust crates
  (`freeldr-rust` bin, `kdcom-rust` cdylib, `bootvid-rust` cdylib).
- **Crown jewel: `lib/peloader.c`.** Rust-implemented PE loader is
  what loads ntoskrnl. After this lands, the entire kernel-mode
  load path is Rust end-to-end.
- **Translation cost via rustconv: ~$3–5 in API spend.** kdcom is
  ~$0.10; bootvid is ~$0.20; the rest is FreeLDR. UEFI types come
  for free via bindgen against the existing `sdk/include/reactos/edk2/`
  headers.
- **Smoke-test risk: high (FreeLDR), low (kdcom + bootvid).** Break
  the bootloader and nothing boots. kdcom and bootvid are loaded
  alongside the kernel but their failures are recoverable (debug
  output silently lost, boot logo missing). Translation order must
  keep at least one C-built FreeLDR binary working at every step;
  kdcom and bootvid can flip atomically.

## Today's FreeLDR

Total **~58K LOC C/asm** under `boot/freeldr/freeldr/` plus ~1,400 LOC
asm under `boot/freeldr/bootsect/`. Two binaries built per arch:

| Binary | i386 | amd64 | arm64 | Path |
|---|---|---|---|---|
| `freeldr.sys` (PC-AT/BIOS) | ✅ 242 KB | ✅ 248 KB | ❌ | El Torito boot from ISO; starts in real mode, transitions to 32-bit |
| `uefildr.efi` (UEFI) | ✅ 338 KB | ✅ 354 KB | ✅ 340 KB | EFI application; loaded by UEFI firmware |
| `frldr16.bin` (real-mode init) | ✅ | ✅ | — | Embedded inside `freeldr.sys` |

**What our smoke tests use today:**
- i386, amd64 → `freeldr.sys` (QEMU SeaBIOS → ISO El Torito)
- arm64 → `uefildr.efi` (QEMU EDK2 firmware → ramdisk)

Both arches' livecd.iso *also* contains `uefildr.efi` at
`EFI/BOOT/boot{IA32,X64}.efi`; we just don't currently boot via UEFI
on those arches. The switch is a one-flag QEMU change.

## LOC by area

```
boot/freeldr/freeldr/
├── freeldr.c             209
├── bootmgr.c             467
├── arcname.c             242
├── settings.c
├── arch/              21,447
│   ├── i386/          13,587   ← BIOS path: pc/, xbox/, pc98/, real-mode bridges
│   ├── uefi/           3,072   ← UEFI implementations of console, disk, mem, hw, vid
│   ├── drivers/        1,736   ← shared (hwide.c IDE driver — BIOS-only consumer)
│   ├── amd64/            738
│   ├── realmode/         598   ← 16-bit asm bridges, .inc helpers
│   ├── arm/              314   ← DEAD (no `arm` arch in our matrix)
│   └── arm64/             63   ← UEFI debug.c + tiny asm
├── lib/               12,719
│   ├── fs/             7,499   ← btrfs, ext, fat, iso, ntfs, pxe
│   ├── mm/             1,683   ← boot heap + memory map
│   ├── inifile/        1,026   ← INI parser
│   ├── cache/            558
│   ├── comm/             149   ← rs232 serial
│   ├── peloader.c      1,107   ← PE loader for ntoskrnl + hal + drivers
│   ├── arcsupp.c
│   ├── debug.c           561
│   └── rtl/               86
├── ntldr/              8,860   ← Windows-style boot path
│   ├── winldr.c        1,386
│   ├── inffile.c       1,552
│   ├── registry.c        938
│   ├── wlregistry.c      857
│   ├── setupldr.c        835
│   ├── wlmemory.c        423
│   ├── headless.c        373
│   ├── ntldropts.c       213
│   └── conversion.c      111
├── ui/                 3,440   ← TUI/directui boot menu
├── disk/               2,483
└── include/            6,157

boot/freeldr/bootsect/  ~1,400  (separate dir — pure asm)
boot/freeldr/{install,fdebug,tools}/  (host-side tooling)
```

## Prune-set

### 1. Dead arch trees (no Rust work needed — just delete)

| Path | LOC | Why drop |
|---|---:|---|
| `arch/arm/` | 314 | 32-bit ARM never built (matches `hal/halarm` decision) |
| `arch/i386/xbox/` | 1,572 | Original Xbox; `SARCH=xbox` we don't set |
| `arch/i386/pc98/` | 2,886 | NEC PC-98; `SARCH=pc98` we don't set |

**Subtotal: ~4,800 LOC.** Reversible; no smoke risk.

### 2. PC-AT path (UEFI-only decision)

| Path | LOC | Notes |
|---|---:|---|
| `freeldr.sys` build target | — | Drop the binary entirely |
| `frldr16.bin` real-mode init blob | — | Embedded in `freeldr.sys` |
| `arch/realmode/` | 598 | 16-bit asm bridges + .inc files |
| `arch/i386/i386trap.S` | 182 | BIOS-side trap entry |
| `arch/i386/{entry,multiboot,int386,pnpbios,drvmap,irqsup,linux}.S` | ~975 | BIOS-side asm |
| `arch/i386/{drivemap,hwacpi,hwapm,hwdisk,hwpci,i386bug,i386idt,halstub,ntoskrnl}.c` | ~1,800 | BIOS hardware detection |
| `arch/i386/pc/` | ~3,800 | Standard PC machine layer (`pcvideo`, `pcdisk`, `pcmem`, `machpc`, `pchw`, `pcrtc`, `pcvesa`, `pcbeep`, `pccons`) |
| `arch/amd64/` (PCAT bits) | ~400 | `entry.S`, `int386.S`, `pnpbios.S`, `linux.S` for amd64 PC-AT |
| `arch/drivers/hwide.c` | 1,736 | BIOS-only IDE driver (UEFI uses firmware block-IO protocol) |
| `boot/freeldr/bootsect/` | ~1,400 | All FAT/NTFS/btrfs/ext/ISO bootsectors |
| `pcat.cmake` | — | Build config for `freeldr.sys` |

**Subtotal: ~9,500 LOC + ~1,400 LOC asm + the bootsect tree.**

### 3. Filesystem readers we don't boot from

We boot from **ISO9660** (livecd) and **FAT32** (arm64 UEFI ramdisk +
amd64/i386 ESP). Other readers exist for installed-disk scenarios we
don't smoke-test.

| File | LOC | Why drop |
|---|---:|---|
| `lib/fs/btrfs.c` | 1,420 | btrfs installed-disk; not booted in smoke |
| `lib/fs/ext.c` | 1,486 | ext2/3/4 installed-disk |
| `lib/fs/ntfs.c` | 1,070 | NTFS installed-disk |
| `lib/fs/pxe.c` | 395 | Network boot via PXE; BIOS-only consumer anyway |

**Subtotal: ~4,371 LOC.**

### 4. Other PC-AT-coupled bits

| Path | LOC | Why drop |
|---|---:|---|
| `linuxboot.c` (top-level) | — | Multiboot Linux loading; PC-AT only, not used |
| `miscboot.c` | — | Misc legacy boot types |
| Most of `disk/` | partial | Some pieces are PC-AT-specific (real-mode disk callouts via INT 13h); UEFI uses firmware Block-IO |

**Total prune: ~20K LOC + ~1,400 LOC asm + the entire `bootsect/` tree
+ multiple `add_executable`/`add_library` cmake targets.** This is
roughly the same magnitude as the prune we projected for ntoskrnl
shims; an order-of-magnitude bigger than the HAL prune.

## Keep-set (after pruning)

```
boot/freeldr/freeldr/
├── freeldr.c           209
├── bootmgr.c           467
├── arcname.c           242
├── settings.c
├── arch/
│   ├── uefi/         3,072   ← UEFI service implementations (console, disk, mem, hw, vid, video)
│   ├── i386/i386idt.c           ← needed for UEFI on i386 (custom IDT setup)
│   ├── i386/i386trap.S          ← needed for UEFI on i386
│   ├── arm64/          63   ← UEFI debug + arm64-asm
│   └── vidfb.c, vgafont.c       ← framebuffer + bitmap font (UEFI uses)
├── lib/
│   ├── peloader.c    1,107   ← CROWN JEWEL: PE loader for ntoskrnl
│   ├── fs/{fat,iso,fs}.c  ~2,900   ← FAT + ISO + dispatcher only
│   ├── mm/           1,683
│   ├── inifile/      1,026
│   ├── cache/          558
│   ├── comm/           149
│   ├── debug.c         561
│   ├── arcsupp.c
│   └── rtl/             86
├── ntldr/            8,860
├── ui/               3,440
├── disk/{disk,partition,ramdisk}.c  ← UEFI-compat subset
└── include/
```

**Estimated total post-prune: ~28K LOC C + small amount of UEFI bridge asm.**

### Per-component landscape

#### `arch/uefi/` — 3,072 LOC

The UEFI service callers. Twelve files:

| File | LOC | Role |
|---|---:|---|
| `uefildr.c` | (entry) | EFI entry, exit-boot-services flow |
| `stubs.c` | (small) | NT-style API stubs that map to EFI |
| `ueficon.c` | (small) | EFI Simple Text Output Protocol → console |
| `uefidisk.c` | (medium) | EFI Block IO Protocol → disk reads |
| `uefihw.c` | (small) | Hardware enumeration via EFI |
| `uefimem.c` | (medium) | EFI memory map → ARC memory descriptor list |
| `uefisetup.c` | (small) | Setup-mode hooks |
| `uefiutil.c` | (small) | Helpers |
| `uefivid.c` | (medium) | EFI Graphics Output Protocol → framebuffer |
| `{i386,amd64,arm64}/uefiasm.S` | (each small) | Per-arch UEFI entry trampoline |

**Translation difficulty:** mostly mechanical. The challenge is wrapping
EFI types/protocols. UEFI types live in `sdk/include/reactos/edk2/` —
**bindgen-able**, same pattern as `ntkrnl-sys` for kernel headers and
`hal-sys` for ACPI tables.

#### `lib/peloader.c` — 1,107 LOC

PE loader. Maps `ntoskrnl.exe`, `hal.dll`, and boot drivers from disk
into memory; resolves imports; handles relocations; calls
`MmFixupForwardHandlers`-style forwarders. **Most strategically
important file in FreeLDR** — once Rust, the entire kernel load chain
is Rust-implemented.

Key pieces:
- `PeLdrLoadImage` — read PE off disk into a heap buffer
- `PeLdrAllocateDataTableEntry` — allocate `LDR_DATA_TABLE_ENTRY` + add to module list
- `PeLdrScanImportDescriptorTable` — recursively load imports
- `PeLdrLoadBootImage` — top-level kernel/hal/driver load

Translation difficulty: medium. Self-contained, well-defined PE format,
lots of bindgen targets in `winnt.h` (`IMAGE_NT_HEADERS`,
`IMAGE_IMPORT_DESCRIPTOR`, `IMAGE_DATA_DIRECTORY`, etc. — already in
`ntkrnl-sys`).

**Latent bug we already know about:** LLD on i386 prepends `_` to PE
export forwarder targets; current C has a workaround in
`MiSnapThunk` (in this file's tree) and in `sysldr.c` (kernel side).
The Rust port should preserve the workaround; verify decoration via
`nm` after translation.

#### `lib/fs/` — keep-set

| File | LOC | Role |
|---|---:|---|
| `fs.c` | 732 | Filesystem dispatch table; opens files via the right reader |
| `fat.c` | 1,734 | FAT12/16/32 reader (read-only) |
| `iso.c` | 662 | ISO9660 reader (livecd boot path) |

**Translation difficulty:** low-medium. These are well-known formats,
read-only, no mutation, no concurrency. Each is one rustconv pass.

#### `lib/mm/` — 1,683 LOC

Boot heap and memory map handling.

| File | LOC | Role |
|---|---:|---|
| `meminit.c` | 731 | ARC memory map construction (consumed from EFI memory map → fed to ntoskrnl) |
| `heap.c` | 642 | Boot-time heap (`FrLdrHeapAlloc`, `FrLdrHeapFree`) |
| `mm.c` | 310 | Memory abstraction |

The heap is bump-allocator-style. ARC memory map is a list of
descriptors with type/start/length. Both straightforward Rust.

#### `ntldr/` — 8,860 LOC

The Windows-style boot path: parse the registry hive, build the
`LOADER_PARAMETER_BLOCK`, set up GDT/IDT/page-tables for kernel entry,
chain-load kernel + HAL + boot drivers, jump into `KiSystemStartup`.

| File | LOC | Role |
|---|---:|---|
| `winldr.c` | 1,386 | Top-level: orchestrates registry, kernel load, GDT/IDT/CR3 setup, jump |
| `inffile.c` | 1,552 | INF parser (used by setup loader) |
| `registry.c` | 938 | Hive registry access |
| `wlregistry.c` | 857 | Boot-driver enumeration from `HKLM\System\CCS\Services` |
| `setupldr.c` | 835 | Setup-mode loader (we don't smoke this; could prune?) |
| `wlmemory.c` | 423 | Page-table setup, kernel-VA mapping |
| `headless.c` | 373 | EMS/SAC headless console |
| `ntldropts.c` | 213 | `/DEBUG` `/HAL=` `/KERNEL=` option parsing |
| `conversion.c` | 111 | Misc conversions |
| `arch/{i386,amd64,arm64}/winldr.c` | varies | Per-arch GDT/IDT/page-table init |

The per-arch `winldr.c` is the boot-protocol bridge — sets up CR3,
loads GDTR/IDTR, sets up the kernel call gate. Different shape per arch
but each is ~150–300 LOC and self-contained.

**Open question:** do we need `setupldr.c` + `inffile.c`? These
support the text-mode setup wizard (running `usetup`), not the livecd
boot path our smoke tests exercise. **2,387 LOC potentially prunable**
if we don't care about installation flows. **Recommendation:** prune
for now; can re-add when we want install-to-disk smoke.

#### `ui/` — 3,440 LOC

The TUI boot menu (color box drawing, keyboard navigation, splash
graphics). Cosmetic. Translation cost matches LOC; reward is low.

**Optimization candidate:** strip down to a minimal serial-only UI
(`ui/noui.c` is already a stub). The full TUI is mostly there to make
F8 / boot-options menus look pretty — irrelevant for our `LiveCD_Debug`
default that doesn't show a menu. **2,000+ LOC potentially prunable.**

## kdcom.dll — KD transport (~933 LOC)

Lives at `drivers/base/kdcom/`. Three substantive C files plus spec.

| File | LOC | Role |
|---|---:|---|
| `kdcom.c` | 356 | `KdPortInitialize`, `KdPortGetByte`, `KdPortPutByte` over 16550 UART |
| `kddll.c` | 427 | `KdSendPacket` / `KdReceivePacket` — the KD wire-protocol framing |
| `kdserial.c` | 150 | Serial framing helpers |
| `kddll.h` | 65 | Local header |
| `kdcom.spec` | 8 | DLL exports |

**ABI surface:** the spec file is **8 lines**. ntoskrnl hard-imports
this set: `KdD0Transition`, `KdD3Transition`, `KdSendPacket`,
`KdReceivePacket`, `KdDebuggerInitialize0`, `KdDebuggerInitialize1`,
`KdSave`, `KdRestore`. Tiny, stable, easy to keep contract through
translation.

**Imports:** `cportlib` (the kernel-mode CRT subset providing port I/O
helpers), plus link-time imports against ntoskrnl + hal.

**Sibling DLLs (drop):** `drivers/base/{kdgdb,kdvm,kd1394,kdusb2,kdrosdbg}`
— alternate KD transports selected via `/DEBUGPORT=`. We use COM1
(routes to `kdcom.dll`). Pruning candidates, same shape as the HAL
DLL prune. `kdgdb` is a GDB stub that's interesting historically but
not load-bearing for our smoke.

**Translation difficulty:** trivial. Three small files, well-defined
wire protocol (KD packet headers + 16550 UART register pokes), no
SEH-protected paths, no concurrency primitives needed. One rustconv
pass. **~$0.10 in API spend.**

**Build artifact:** `kdcom-rust` cdylib crate. Custom kernel-mode
target spec (same shape as the planned HAL crate). Links against the
existing `kdcom.spec` for export contract. Cargo build replaces the
cmake `add_library(kdcom MODULE …)` target; existing `add_cd_file`
points at the cargo artifact.

## bootvid.dll — boot-time framebuffer (~1,400 LOC keep-set)

Lives at `drivers/base/bootvid/`. Multiple per-arch backends; UEFI-only
mode keeps just the framebuffer one.

| File | LOC | Role | Keep? |
|---|---:|---|---|
| `framebuf/bootvid.c` | 538 | Linear framebuffer (UEFI GOP) | ✅ — all 3 arches |
| `common.c` | 391 | Bitmap blit + glyph render shared logic | ✅ |
| `console.c` | 173 | Boot console abstraction | ✅ |
| `fontdata.c` | 274 | 8x16 VGA font bitmap (data blob) | ✅ |
| `precomp.h` | 104 | Precompiled header | ✅ |
| `framebuf/framebuf.h` | 22 | local header | ✅ |
| `bootvid.spec` | 11 | Exports | ✅ |
| `i386/pc/bootvid.c` + `vga.c` + headers | 1,009 | VGA register pokes (BIOS path) | ❌ drop with PC-AT |
| `i386/pc/bootdata.c` | 127 | VGA color tables | ❌ |
| `arm/bootvid.c` + `arm.h` | 350 | ARM-specific framebuffer | ❌ dead |
| `i386/xbox/bootvid.c` + `xbox.h` | 453 | Xbox NV2A | ❌ dead |
| `i386/pc98/bootvid.c` + `pc98.h` | 511 | PC-98 framebuffer | ❌ dead |

**ABI surface:** spec is **11 lines**. ntoskrnl's `INBV` subsystem
imports `VidInitialize`, `VidResetDisplay`, `VidCleanUp`,
`VidDisplayString`, `VidBufferToScreenBlt`, `VidScreenToBufferBlt`,
`VidBitBlt`, `VidSolidColorFill`, `VidDisplayStringXY`,
`VidSetTextColor`, `VidSetScrollRegion`. Stable contract.

**Translation difficulty:** medium. Mostly framebuffer pixel-pushing
and glyph rendering for the boot logo + blue-screen text path.
Color-format edge cases (RGB vs BGR, 16/24/32 bpp) are the main
lift. `fontdata.c` is a 256-glyph bitmap blob → `include_bytes!` in
Rust, no translation needed for the data itself.

UEFI-only is the architectural simplification: all three of our
arches converge on `framebuf/bootvid.c` because UEFI GOP is the
firmware-portable framebuffer abstraction. The per-arch backends
exist in upstream ReactOS for boot paths we don't take.

**Translation cost:** one rustconv pass, ~6 files. **~$0.20.**

**Build artifact:** `bootvid-rust` cdylib crate. Same shape as kdcom.
The existing `add_bootvid` cmake function (which currently parameterizes
on the per-arch backend) collapses to a single Rust crate target.

## Build artifact decision

**FreeLDR: one Rust `bin` crate, three target specs**, mirroring the
cargo PoC shape that ntoskrnl is moving toward:

```
sdk/lib/rust/freeldr-rust/
├── Cargo.toml
├── build.rs                  ← compiles UEFI bridge asm + minimal C shims
├── i386-uefildr.json         ← custom target: PE32, no SSE, kernel-style runtime
├── amd64-uefildr.json
├── arm64-uefildr.json
└── src/
    ├── main.rs               ← #![no_main], EFI entry shim
    ├── arch/
    │   ├── uefi.rs           ← shared UEFI logic
    │   ├── i386.rs           ← i386 EFI → 32-bit transition
    │   ├── amd64.rs
    │   └── arm64.rs
    ├── pe.rs                 ← peloader port
    ├── fs/{mod,fat,iso}.rs
    ├── mm/{mod,heap,meminit}.rs
    ├── inifile.rs
    ├── ntldr/                ← winldr, registry, wlmemory
    └── ui.rs                 ← minimal serial UI
```

UEFI types: extend `ntkrnl-sys` with an `efi` module bindgen'd from
`sdk/include/reactos/edk2/` — or split into a separate `efi-sys` crate.
Recommendation: separate crate (different consumer set; ntoskrnl
doesn't need EFI types).

The Rust binary ships as the `uefildr.efi` replacement. cmake's
existing `add_cd_file(TARGET uefildr ...)` invocation switches to point
at the cargo-built artifact, same pattern as the ntoskrnl cargo PoC.

**kdcom and bootvid: per-DLL cdylib crates** living next to the
existing `sdk/lib/rust/` workspace:

```
sdk/lib/rust/
├── kdcom-rust/                 ← new: KD COM transport
│   ├── Cargo.toml              ← crate-type = ["cdylib"]
│   ├── build.rs                ← uses existing kdcom.spec for export contract
│   └── src/lib.rs
├── bootvid-rust/               ← new: boot framebuffer
│   ├── Cargo.toml              ← crate-type = ["cdylib"]
│   ├── build.rs                ← uses existing bootvid.spec
│   └── src/
│       ├── lib.rs              ← Vid* exports
│       ├── framebuf.rs         ← linear framebuffer
│       ├── common.rs           ← shared blit + glyph
│       ├── console.rs
│       └── font.rs             ← include_bytes!("fontdata.bin")
└── freeldr-rust/               ← FreeLDR bin crate (above)
```

Each crate uses the same custom kernel-mode target spec pattern (PE32
output, no SSE, no userland CRT, etc.) that the kernel/HAL crates use.
Three target JSONs per crate, shared across the workspace.

## Translation strategy

### What we don't pay for

- **UEFI types → bindgen** from `sdk/include/reactos/edk2/`. ~120
  protocol/struct types. $0.
- **PE/COFF types → bindgen** from `winnt.h` (already in `ntkrnl-sys`). $0.
- **ARC types** from `sdk/include/reactos/arc/`. $0.
- **Boot sector code** — doesn't exist post-prune.
- **Real-mode asm** — doesn't exist post-prune.

### What we do pay for (rustconv)

The ~28K LOC of substantive C. At Sonnet-batch rates ~$0.03/file, ~30
files × $0.03 ≈ **~$1 per pass**, plus iteration. Realistic budget
**~$3–5 across all iterations**. Comparable to the HAL.

### Pipeline staging

Mirrors the ntoskrnl/HAL pattern:

1. **Stage 0 — `efi-sys` substrate.** bindgen the EDK2 headers; add to
   workspace.
2. **Stage 1 — leaf libraries first.** Translate `lib/inifile/`,
   `lib/mm/`, `lib/cache/`, `lib/comm/rs232.c`, `lib/debug.c`,
   `lib/arcsupp.c`. These have no internal cross-deps and are exercised
   incrementally. Each ships as a Rust static lib linked into the
   existing C `uefildr` build (same augment pattern as kernel-rust).
3. **Stage 2 — filesystem readers.** `lib/fs/{fs,fat,iso}.c`. Self-contained.
4. **Stage 3 — peloader.** `lib/peloader.c`. The crown jewel. Lands as
   a static lib first; once smoke-green, the C version is dropped.
5. **Stage 4 — UEFI bridge.** `arch/uefi/`. Needs `efi-sys` types.
6. **Stage 5 — ntldr.** `ntldr/{winldr,registry,wlregistry,wlmemory,
   ntldropts,conversion,headless}.c` plus `arch/<arch>/winldr.c`.
   Largest chunk; per-arch boot-protocol bridge is the trickiest.
7. **Stage 6 — top-level orchestration.** `freeldr.c`, `bootmgr.c`,
   `arcname.c`, `settings.c`, plus the minimal UI. Once these flip,
   the cargo `bin` becomes the canonical artifact and cmake's C
   `uefildr` target gets retired.

### Smoke-test risk management

**The bootloader is the single point of failure for boot.** The
augment-then-flip pattern (translate to a Rust static lib, link
alongside C, then drop C) is the only safe way:

- For each translated file, the C source stays in the cmake build.
- Rust translation lands as `extern "C"` symbols with the same names.
- Linker symbol resolution chooses Rust over C (use `--whole-archive`
  on the Rust lib).
- Once smoke is green for N consecutive passes with the Rust symbols
  in play, delete the C file from `pcat.cmake`/`uefi.cmake`.

This was validated for the kernel ob/handle, ke/, ex/, etc.
translations — the same playbook applies.

## Forward path

In rough priority order:

1. **Switch i386/amd64 smoke to UEFI.** Single-flag QEMU change in
   `tests/qemu/vm.py` (add `-bios OVMF.fd` for `I386Config` /
   `Amd64Config`); verify all 3 arches still pass 7/7. Includes
   ensuring OVMF is in CI's package list (`ovmf` + `ovmf-ia32`).
2. **Land the prune in one PR.** Drop arm/, xbox/, pc98/, the entire
   PC-AT build path, the unused FS readers, `bootsect/`, alternate
   KD transports (`kdgdb`/`kdvm`/`kd1394`/`kdusb2`/`kdrosdbg`), and
   bootvid arch variants (`arm/`, `i386/{pc,xbox,pc98}/`). Smoke
   gates the prune; if any of the dead targets turn out to be live,
   smoke catches it. Estimate: 1–2 sessions.
3. **`efi-sys` bindgen substrate.** New crate or ntkrnl-sys extension;
   generate UEFI protocol/struct types. 1 session.
4. **kdcom port.** Smallest scope, lowest risk (failure mode is
   silent — debug output lost). Validates the cdylib + spec-export
   contract pattern for sibling DLLs. **0.5–1 session.**
5. **bootvid port.** Slightly larger but still self-contained.
   Validates framebuffer access via the EFI GOP wrapper. Failure
   mode: missing boot logo / blank console (recoverable). **1–2 sessions.**
6. **FreeLDR Stage 1 + 2 — leaf libraries + filesystems.** Bulk
   rustconv pass over `lib/inifile/`, `lib/mm/`, `lib/cache/`,
   `lib/comm/rs232.c`, `lib/debug.c`, `lib/arcsupp.c`,
   `lib/fs/{fs,fat,iso}.c`. Augment-link as Rust static lib
   alongside C. 2–3 sessions.
7. **FreeLDR Stage 3 — peloader port.** The strategic milestone.
   Once Rust, the kernel load is Rust→Rust. 1–2 sessions.
8. **FreeLDR Stage 4–5 — UEFI bridge + ntldr.** Largest chunk, but
   mechanical once the substrate exists. 4–6 sessions.
9. **FreeLDR Stage 6 — top-level + UI cleanup.** Flip cargo `bin` as
   canonical artifact; retire cmake C `uefildr` build. 2 sessions.

**Why kdcom + bootvid before the FreeLDR meat:** they're cheap
warmups that validate the cdylib + spec-contract pattern in
isolation. If the ABI handoff between Rust-built `*.dll` and
C-built `ntoskrnl.exe` breaks, we want to discover it on a 933-LOC
target, not a 28K-LOC one. They also reuse zero of FreeLDR's
substrate — no peloader, no UEFI bridge, no ntldr — so they can
ship while FreeLDR is still C.

## Definition of done

For each component:

- `cargo build -p freeldr-rust --target <arch>` produces a working
  `uefildr.efi` for all three arches.
- `cargo build -p kdcom-rust --target <arch>` produces a working
  `kdcom.dll` for all three arches.
- `cargo build -p bootvid-rust --target <arch>` produces a working
  `bootvid.dll` for all three arches.

All three smoke tests pass on the cargo-driven binaries.
`tools/scan_ud2.py --check` is clean against each.

The following cmake-driven C trees can be deleted (or kept as a
parallel CI check):

- `boot/freeldr/freeldr/`
- `drivers/base/kdcom/`
- `drivers/base/bootvid/`

## What this audit doesn't measure

- **Boot correctness.** Smoke is the source of truth.
- **Hardware compatibility differences between BIOS and UEFI.**
  Switching to UEFI changes the firmware surface — ACPI tables come
  from the firmware (always have on UEFI), memory map comes from
  EFI's GetMemoryMap rather than int15h E820, framebuffer is GOP not
  VESA. The existing `arch/uefi/` already handles all of this on arm64;
  bringing i386/amd64 in line should be a no-op since the same C code
  paths exist. But if any hardware-specific BIOS-only quirk turns
  out to be load-bearing, smoke will catch it during step 1.
- **Install-to-disk path.** `setupldr.c` + `inffile.c` support text-mode
  setup. We don't smoke this. If/when we want install-to-disk smoke,
  re-translate those files (~2,400 LOC).

## Open questions

1. **Prune `setupldr.c` + `inffile.c`?** They're install-mode-only
   (~2,400 LOC) and we don't smoke installation. **Recommendation:
   prune now**, re-add if needed.
2. **TUI vs serial-only UI?** The full TUI (3,440 LOC) is cosmetic for
   menu-based boot selection; our `LiveCD_Debug` default doesn't show
   a menu. **Recommendation: prune to a minimal serial UI**, save
   ~2,000 LOC.
3. **`efi-sys` standalone vs `ntkrnl-sys` extension?** Different
   consumer sets — the kernel doesn't need EFI types. **Recommendation:
   standalone crate.**
4. **One Rust `bin` crate or per-arch crates?** ntoskrnl cargo PoC
   uses one crate with multiple target specs. **Recommendation: same
   pattern.**
5. **`freeldr.spec` keeps or drops?** With FreeLDR not exporting symbols
   to other DLLs (it's a self-contained executable), the spec file is
   only used internally for `KdComPortInUse` etc. across the
   `freeldr_common` static lib boundary. Likely can be dropped once
   the build is one cargo crate. Verify during step 9.
6. **Keep `kdgdb` for future GDB-stub debugging?** It's a sibling KD
   transport that lets you remote-debug the kernel from a host GDB
   instance — potentially useful for kernel-mode bug triage. Not in
   any current smoke path. **Recommendation: prune now**, the
   `qemu/inspect.sh`-based debugging infrastructure (see CLAUDE.md)
   covers this need without `kdgdb`'s complexity. Re-add if needed.
7. **`bootvid-rust` and `kdcom-rust` as separate crates or fold into
   `kernel-rust`?** They're independent DLLs with separate spec files,
   so the cleaner shape is separate crates. But they share the
   kernel-mode runtime (no userland CRT, no SSE, etc.) so a
   workspace-level shared profile is worth setting up.
   **Recommendation: separate crates, shared workspace profile.**
