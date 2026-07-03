# FreeLDR + Boot DLLs C→Rust Translation Audit

Last updated 2026-06-21. **Status: FreeLDR + all boot-DLLs flipped.**

**Translation: ~99% (methodology).** Measured from the tree on 2026-06-21:
`freeldr-rust` is **89 `.rs` modules / ~74.8K LOC**, plus the boot-helper crates
`cmlib-rust` (15 files / ~7.1K), `cmboot-rust` (1 file / ~2.8K), `cport-rust`
(1 file / ~0.3K) — **~85K LOC of boot Rust total** across 106 files. The only
non-Rust glue left in the booted UEFI path is `lib/strsafe_shim.c` (variadic
`RtlStringCb*` shell Rust stable can't host — floor) and `arch/i386/drvmap.S`
(16-bit `.code16`, clang can't assemble — floor, i386 rosload only). Outside the
booted path, `boot/freeldr/bootsect/{fat.S,fat32.S}` (512-byte FAT VBR reused to
stamp the UEFI `efisys.bin` image, floor — the pure-BIOS sectors were git-rm'd).
The four boot binaries (`uefildr.efi` ×3 arch, `rosload.exe` i386+amd64,
`kdcom.dll`, `bootvid.dll`) are all cargo-link-driven and cohab-flipped. "~99%"
is by remaining-glue: 2 small irreducible files (variadic shim + 16-bit asm)
against the whole boot body translated.

> **Ground-truth re-verified 2026-06-21** against `boot/freeldr/` + `sdk/lib/rust/freeldr-rust` + the `-cargo`/boot-helper crates — FreeLDR (89 `.rs` modules / ~74.8K LOC, confirmed by `find … *.rs | wc -l`) + kdcom/bootvid/uefildr/rosload cargo-driven links all present and wired into the main cmake build. `cmboot.c` is **gone from disk** (translated to the `cmboot-rust` crate, linked via freeldr-rust). The only non-Rust left in the booted path is `lib/strsafe_shim.c` (variadic shell, floor) + `arch/i386/drvmap.S` (16-bit `.code16` gcc obj, floor); plus `boot/freeldr/bootsect/{fat.S,fat32.S}` (boot-sector asm reused to stamp the UEFI FAT image, floor — the pure-BIOS sectors were git-rm'd).
This doc started (2026-05-08) as a pre-translation plan; it is now a
*state* audit. The headline change since: **FreeLDR is translated and
integrated** (89 Rust modules, cohabitation, all 3 arches boot green),
**the BIOS path is pruned** (x86 is UEFI-only), **the display drivers
are flipped** (`arm64vid.sys` + `framebuf.dll`), and as of 2026-06-01
the two sibling DLLs **`kdcom.dll` and `bootvid.dll` are flipped to Rust
via cohabitation and pass smoke on all 3 arches**.

**Scope:** the pre-kernel components FreeLDR loads alongside ntoskrnl +
HAL before `KiSystemStartup`:

1. **FreeLDR** itself (`uefildr.efi`) — the bootloader binary — **DONE**
2. **`kdcom.dll`** — kernel debugger COM transport — **DONE** (Rust, 3-arch green)
3. **`bootvid.dll`** — boot-time framebuffer — **DONE** (Rust, 3-arch green)

Plus two display drivers that landed alongside the boot-track work:
4. **`arm64vid.sys`** — early UEFI GOP video driver — **DONE**
5. **`framebuf.dll`** — GDI framebuffer display driver — **DONE**

Companion to:
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — the kernel translation
- [hal-translation-audit.md](hal-translation-audit.md) — the HAL translation

---

## Current state (2026-06-21)

| Component | Crate | Translated | Integrated (default cmake) | Smoke |
|---|---|---|---|---|
| **peloader** (crown jewel) | `freeldr-rust` (`peloader.rs`) | ✅ | ✅ (within freeldr-rust) | ✅ kernel load is Rust→Rust |
| **rosload** 2nd-stage | `freeldr-rust` + `rosload-cargo` | ✅ | ✅ cargo-link (i386+amd64), cmake link retired | ✅ |
| **cmboot / cmlib / cport** | `cmboot-rust`, `cmlib-rust`, `cport-rust` | ✅ | ✅ (linked via freeldr-rust; `cmboot.c` git-rm'd) | ✅ |
| **arm64vid.sys** | `arm64vid-rust` | ✅ 3-arch | ✅ cohabitation | ✅ (UEFI GOP on every boot) |
| **framebuf.dll** (GDI) | `framebuf-rust` | ✅ 3-arch | ✅ cohabitation | ✅ |
| **kdcom.dll** | `kdcom-rust` | ✅ | ✅ cohabitation (kdrosdbg/CMakeLists) | ✅ 3 arches |
| **bootvid.dll** | `bootvid-rust` (framebuf GOP) | ✅ | ✅ cohabitation (add_bootvid RUST) | ✅ 3 arches |
| **BIOS / PC-AT path** | — | — | ✅ **PRUNED** (`pcat.cmake` deleted) | x86 UEFI-only |

**What "DONE" means here:** the C sources stay on disk (never-delete
rule), but Rust `#[no_mangle]` exports take over at link time via the
cohabitation pattern, and the smoke harness boots all three arches
through the Rust path.

**What changed the plan most:** the BIOS-vs-UEFI decision in the
original audit was carried out fully — `pcat.cmake` is *deleted*, the
x86 smoke configs are `i386-uefi` / `amd64-uefi` (OVMF), and all three
arches load `uefildr.efi`. There is no BIOS fallback to maintain, which
is what made flipping the whole of FreeLDR to one Rust crate tractable.

---

## FreeLDR — DONE (`freeldr-rust`, 89 modules / ~74.8K LOC, 3-arch)

`sdk/lib/rust/freeldr-rust/` builds three archives consumed by the
cmake `uefildr` / `rosload` targets via cohabitation:

- `libfreeldr_rust.a` — main loader symbols
- `libfreeldr_rust_uefi.a` — UEFI bridge (`uefi` feature)
- `libfreeldr_rust_rosload.a` — 2nd-stage `rosload.exe` (`rosload` feature)

Module inventory by area (all translated; C kept on disk, commented out
of the freeldr cmake `SOURCE` lists):

| Area | Rust modules | Was |
|---|---|---|
| **Core / entry** | `freeldr`, `frldr`, `uefildr`, `bootmgr`, `arcname`, `settings` | freeldr.c, bootmgr.c, … |
| **peloader (crown jewel)** | `peloader`, `ffi` | lib/peloader.c (1118 LOC) |
| **lib/fs** | `fs`, `fat`, `iso`, `pxe` | lib/fs/*.c (btrfs/ext/ntfs **pruned**) |
| **lib/mm** | `meminit`, `mm`, `heap` | lib/mm/*.c |
| **lib/inifile** | `ini_init`, `inifile`, `parse` | lib/inifile/*.c |
| **lib/cache, comm, rtl** | `blocklist`, `cache`, `rs232`, `libsupp` | lib/cache, comm, rtl |
| **lib/debug, arcsupp** | `debug`, `arcsupp` | lib/debug.c, arcsupp.c |
| **UEFI bridge** | `ueficon`, `uefidisk`, `uefihw`, `uefimem`, `uefivid`, `uefisetup`, `uefi_stubs`, `uefiutil` | arch/uefi/*.c |
| **ntldr** | `winldr`, `wlmemory`, `wlregistry`, `registry`, `inffile`, `setupldr`, `headless`, `conversion`, `winldr_{i386,amd64,arm64}` | ntldr/*.c (all) |
| **ui** | `ui`, `tui`, `tuimenu`, `minitui`, `noui`, `video`, `vidfb`, `vgafont`, `directui` | ui/*.c (gui.c/directui stub **pruned**) |
| **disk / arch hw** | `partition`, `ramdisk`, `drivemap`, `hwacpi`, `hwapm`, `hwdisk`, `hwpci`, `hwide`, `halstub`, `i386bug`, `i386idt`, `arm64_debug`, `arcemul`, `archwsup` | disk/*.c, arch/*/*.c |
| **boot logic** | `custom`, `options`, `oslist`, `miscboot`, `linuxboot` | bootmgr support |
| **shims** | `strsafe_shim` (**C kept** — variadic), `rosload_shims`, `cmboot_shims` | — |
| **cmboot/cmlib (registry hive)** | `cmboot-rust`, `cmlib-rust`, `cport-rust` (sibling crates) | cmboot.c (**git-rm'd**), cmlib, cportlib |

**Notable kept-as-C (the irreducible glue floor):**
- `lib/strsafe_shim.c` — Rust stable cannot define variadic functions, so
  `RtlStringCbPrintfA` & friends forward to the C `ntstrsafe.h` inlines (see
  memory `feedback_degraded_variadic_stubs`). Still the only active `.c` in
  `uefi.cmake`'s SOURCE list.
- `arch/i386/drvmap.S` — 16-bit `.code16` drive-map asm; clang can't assemble
  it, so it stays a gcc-built object (`drvmap_gcc.obj`) consumed by the i386
  `rosload` cargo link. The **sole remaining cmake-built input** to rosload.

---

## Prune — DONE

The UEFI-only decision in the original audit was executed across a
multi-part "BIOS prune" series:

| Item | Status | Commit theme |
|---|---|---|
| `pcat.cmake` / `freeldr.sys` BIOS loader | ✅ **deleted** | "BIOS prune (4/N)" |
| x86 smoke → UEFI only (`i386-uefi`, `amd64-uefi`, OVMF) | ✅ | "BIOS prune (1/N)" |
| UEFI-only livecd ISO (drop El Torito BIOS boot) | ✅ | "BIOS prune (2/N)" |
| bootvid → GOP framebuffer on all arches | ✅ | "BIOS prune (3/N)" |
| dead BIOS-boot source files git-rm'd | ✅ | "BIOS prune (5/N)" |
| `lib/fs/{btrfs,ext,ntfs}.c` | ✅ pruned (no UEFI reader) | — |
| `lib/fs/pxe.c` | ✅ kept + translated (`pxe.rs`) | — |

**Still on disk but unused (cleanup candidates):**
- `boot/freeldr/bootsect/{fat.S,fat32.S}` — kept (reused to stamp the UEFI FAT
  image, see Forward-path §3); the pure-BIOS sectors are already git-rm'd.
- `drivers/base/{kdgdb,kdvm,kdrosdbg}` — alternate KD transports, still C.
- Live filesystem table is `[Iso, Fat, Pxe]` (livecd ISO + ramdisk FAT
  + PXE net boot).

> Note: `arch/i386/xbox/` and `arch/i386/pc98/` never existed in this
> repo's tree — the original audit's projected prune of those was moot.

---

## kdcom.dll — Rust body + CARGO-DRIVEN LINK in the main build (2026-06-04)

Lives at `drivers/base/kdcom/` (under `_WINKD_=TRUE`, the WinDbg
default) or `drivers/base/kdrosdbg/` (under `_WINKD_=FALSE`, the ReactOS
debugger — see memory `feedback_kdcom_is_kdrosdbg`). The active
(`_WINKD_=FALSE`) `drivers/base/kdrosdbg/` path is fully Rust.

Two workspace crates:
- `kdcom-rust` (3-arch staticlib, the kdrosdbg.c translation)
- `kdcom-cargo` (cargo-driven `.dll` link via the `reactos-link` engine,
  `ModuleKind::KernelDll`, 3-arch: i686/x86_64/aarch64-kernel.json)

**The main cmake build now drives the final link from cargo, not
`add_library(MODULE)`.** `drivers/base/kdrosdbg/CMakeLists.txt` keeps only
`spec2def(kdcom.dll kdrosdbg.spec ADD_IMPORTLIB)` (ntoskrnl imports the 8
Kd* exports) and calls `add_cargo_kernel_dll(kdcom kdcom-cargo kdcom.dll
BODY_CRATE kdcom-rust)` (the reusable helper in `sdk/cmake/gcc.cmake`):
cargo builds kdcom-rust + links it with cportlib/memcmp + the ntoskrnl
import lib into the shipping PE, then applies the kernel-dll post-link
(`native-pefixup --kerneldll`, + the i386 forwarder fix). The cmake link
is **retired** — this is the first module whose cmake link is fully
cargo-driven. Cold smoke green ×3 (PE-header/export-identical to the old
cmake link). The cargo link uses nightly + build-std (inherent to the
cargo-drive path).

**ABI surface** (8-line spec): `KdD0Transition`, `KdD3Transition`,
`KdSendPacket`, `KdReceivePacket`, `KdDebuggerInitialize0/1`, `KdSave`,
`KdRestore`. Imports `cportlib` (port-I/O helpers).

---

## bootvid.dll — Rust body + CARGO-DRIVEN LINK in the main build (2026-06-04)

Lives at `drivers/base/bootvid/`. Post-BIOS-prune, **the only backend
that builds is `framebuf/` (UEFI GOP linear framebuffer)** — the legacy
VGA/`i386/pc/` path is gone, and arm/xbox/pc98 never applied. So the
keep-set the original audit projected is exactly what's left.

Two crates: `bootvid-rust` (3-arch staticlib) + `bootvid-cargo` (cargo
`.dll` link, `ModuleKind::KernelDll`, 3-arch). **The main cmake build drives
the final `bootvid.dll` link from cargo** (the `RUST` branch of `add_bootvid`
in `drivers/base/bootvid/CMakeLists.txt` calls `add_cargo_kernel_dll(bootvid
bootvid-cargo bootvid.dll BODY_CRATE bootvid-rust …)`); `spec2def(... 
ADD_IMPORTLIB)` stays for the 11 Vid* exports (ntoskrnl imports them). The
cmake `add_library(MODULE)` link is retired. The sibling `lfbbvid.dll` (a
vestigial alias not loaded by INBV) stays a C `add_library`. Cold smoke
green ×3.

> The `bootvid-rust/Cargo.toml` carries a note that the original audit's
> keep/prune set was "inverted" — historical confusion now moot since
> only the framebuffer backend survives.

**ABI surface** (11-line spec): `VidInitialize`, `VidResetDisplay`,
`VidCleanUp`, `VidDisplayString`, `VidBufferToScreenBlt`,
`VidScreenToBufferBlt`, `VidBitBlt`, `VidSolidColorFill`,
`VidDisplayStringXY`, `VidSetTextColor`, `VidSetScrollRegion`.

---

## Display drivers — DONE (landed alongside the boot track)

These weren't in the original audit's scope but are part of the
"everything UEFI GOP" convergence and are fully flipped:

| Driver | Crate | C cohab | Notes |
|---|---|---|---|
| **arm64vid.sys** | `arm64vid-rust` | `drivers/video/arm64vid/{arm64vid.c→commented, rust_cohab_stub.c}` | early UEFI GOP video, all 3 arches |
| **framebuf.dll** | `framebuf-rust` | `win32ss/drivers/displays/framebuf/{enable,palette,pointer,screen,surface}.c→commented` | GDI display driver, all 3 arches |

Both use the standalone-`.sys`/`.dll` cohabitation pattern (staticlib +
`rust_cohab_stub.c`, per-arch `RUST_TARGET`); see memory
`feedback_standalone_sys_cohab_rust`.

---

## Forward path (remaining work, priority order)

   *(kdcom.dll + bootvid.dll cohabitation flips: **DONE** 2026-06-01,
   3-arch smoke green. Lesson: a freeldr-LOADED DLL must NOT use
   raw-dylib for cross-DLL kernel-data imports — the 2nd ntoskrnl.exe
   import descriptor hangs freeldr's peloader; use the import lib's
   `__imp_` thunk instead. See memory feedback_freeldr_loaded_dll_no_raw_dylib.)*

1. **kdcom-cargo / bootvid-cargo fate: DECIDED — wired into the main build
   (2026-06-04).** No longer off-path PoCs: the cmake build drives both
   `.dll` links from cargo via the reusable `add_cargo_kernel_dll` helper
   (`sdk/cmake/gcc.cmake`); the cmake `add_library(MODULE)` links are retired.
   Cold smoke green ×3. This is the first cmake→cargo link migration in the
   tree — the pattern (`spec2def` importlib stays in cmake; cargo produces the
   shipping PE + post-link) is the template for retiring the cmake links of
   `uefildr`/`rosload`/`ntdll`/`ntoskrnl` next. (The standalone
   `build-and-test.sh` in each crate remains an isolated i386 link+swap-smoke
   debug helper.)
2. **Prune alternate KD transports.** `kdgdb`, `kdvm`, and the inactive
   `_WINKD_` half (`kdcom` or `kdrosdbg`) are dead for our COM1 smoke
   config. `qemu/inspect.sh` GDB tooling covers the kdgdb use case.
3. **Prune `boot/freeldr/bootsect/`.** ✅ **Partially done (2026-06-02).**
   The four pure-BIOS sectors (`dosmbr.S`/`ext.S`/`extldr.S` legacy disk/ext
   boot + experimental `faty.S`) are `git rm`'d — they were `FOR bootcd
   regtest` only, never on the booted livecd, and feed nothing in the UEFI
   path. `fat.S`/`fat32.S` **stay**: `native-fatten -boot fat.bin` reads
   their 512-byte FAT VBR back to stamp the UEFI `efisys.bin` FAT image
   (smoke boots it), and `installfreeldr` bin2c's them. So the dir is not
   fully deletable — the original "pure BIOS asm" claim was inexact.
4. **Retire the cmake C `uefildr` build → cargo `bin`.** 🚧 **PoC started
   (2026-06-02): `sdk/lib/rust/uefildr-cargo`.** New `ModuleKind::
   EfiApplication` in the `reactos-link` engine (subsystem
   `efi_application:5.01`, entry `_EfiEntry`, 0x200 align, `--gc-sections`
   + auto image base — extracted verbatim from `build.ninja`). The crate
   reuses the cmake-built `libfreeldr_rust_uefi.a` + sibling libs +
   `strsafe_shim.c.obj` and drives the final EFI link from cargo (same
   shape as `ntoskrnl-cargo`/`kdcom-cargo`). **Status: LINKS CLEAN** —
   produces a `uefildr.efi` whose PE header is identical to cmake's
   (coff-i386, Subsystem 0x0a EFI application, ImageBase 0x00400000,
   0x200 file/section align, 5.1 subsystem version; entry differs by 96
   bytes — the `main.rs` panic stub — both in the `EfiEntry` region).
   The fix for the initial `_PxeCallApi`/`_DriveMapMapDrivesInSection`
   `<root>: undefined` was to **not** pass the `.def`: cmake's uefildr
   link (rules.ninja `C_EXECUTABLE_LINKER__uefildr`) never feeds
   `uefildr.def` to the linker — the EFI app links `--exclude-all-symbols`
   and exports nothing (the `.def`/`freeldr.spec` is only for the freeldr
   *import library* that rosload consumes). Passing it forced every spec
   export as required, and a few are genuinely absent from the UEFI build.
   **amd64 too (2026-06-02):** `reactos-link` is now arch-aware (`ArchCfg`
   threads build dir / clang triple / clang_rt / spec2def all per-arch), and
   `uefildr-cargo` builds for amd64 via `x86_64-kernel.json` (entry `EfiEntry`
   no-decoration, `--image-base 0x10000`, no mini_hal libs — all from
   build-amd64's build.ninja). The amd64 `uefildr.efi` is likewise
   PE-header-identical to cmake's (PE32+, Subsystem 0x0a, ImageBase 0x10000,
   0x200 align; entry differs 208 bytes for the panic stub).

   **arm64 too (2026-06-02): all 3 arches now cargo-link.** `EfiApplication`
   gained two per-arch knobs — `image_base: Option<u64>` (`None` ⇒ rely on the
   common `--enable-auto-image-base` instead of emitting an explicit base) and
   `alignment: u64` (x86 0x200, arm64 0x1000). arm64 `uefildr-cargo` builds via
   `aarch64-kernel.json` (entry `EfiEntry`, NO `--image-base`, 0x1000
   file/section align, same lib set as amd64 — no mini_hal). The arm64
   `uefildr.efi` is PE-header-identical to cmake's (coff-arm64, Subsystem 0x0a
   EFI application, ImageBase 0x140000000 [auto-hash, both use
   `--enable-auto-image-base`], 0x1000 align, OS/subsystem ver 5.1; entry
   differs 168 bytes for the panic stub). `reactos-link`'s
   asm/c_shims/spec2def/import_lib paths stay x86-leaning (the kernel/HAL
   KernelExe/KernelDll links aren't wired for arm64 yet) — but the
   EfiApplication/static_libs path is fully tri-arch.
   Remaining: full **boot**-validation needs a livecd (x86) / ramdisk swap
   (arm64) under smoke — the link + PE-header parity is proven on all three.
7. **Optional: `setupldr.c`/`inffile.c` are already translated** (they
   ship in `freeldr-rust`), so the original "prune install-mode?" open
   question is moot — they're in Rust and cheap to keep.
8. **`rosload.exe` is now cargo-link-driven too (2026-06-04):
   `sdk/lib/rust/rosload-cargo`.** New `ModuleKind::NativeExe` in the
   `reactos-link` engine — native-subsystem exe WITH exports (`rosload.def`,
   one export `RtlAssert`, so NO `--exclude-all-symbols` unlike `KernelExe`),
   `--gc-sections`, no `-shared`, image base 0x10000, DEFAULT PE alignment
   (cmake's rosload link passes no `--file/-section-alignment`). Plus a new
   `.force_undefined(&[...])` builder for the i386-only
   `-u _HalDispatchTable`/`-u _HalPrivateDispatchTable` pulls (the Rust
   halstub dispatch tables are unreferenced+unexported → GC'd without `-u`).
   Like `uefildr-cargo` it REUSES the cmake-built inputs (run `ninja rosload`
   first): the prebuilt rosload.dir objects (`strsafe_shim.c.obj`,
   `cmboot.c.obj`, per-arch asm), `libfreeldr_rust_rosload.a` (`--features
   rosload`, carries the `RunLoader` entry), the leaf libs + i386 mini-HAL,
   and the freeldr import lib (`libfreeldr.dir/libfreeldr.a`). i386 + amd64
   only (arm64 is UEFI-only, no rosload). Cargo `rosload.exe` is
   PE-header-identical to cmake's (ImageBase 0x10000, NT-native subsystem,
   ver 5.1, exports `RtlAssert` ord 1; entry RVA differs for GC layout) and
   **boots both livecds to desktop under smoke** (`build-and-test.sh
   [i386|amd64]`, `RUN_SMOKE=1`; 7-pass, the lone `test_idle_cpu_not_spinning`
   red is the pre-existing idle-spin gate, not rosload). With this, BOTH
   freeldr link binaries (`uefildr.efi` + `rosload.exe`) are cargo-driven.
9. **uefildr + rosload now wired into the MAIN cmake build (2026-06-04) —
   cmake's loader link RETIRED.** The two `add_executable` links are gone;
   `boot/freeldr/freeldr/{uefi,rosload}.cmake` call
   `add_cargo_module(... KIND efi|native …)` (the generalized
   `add_cargo_kernel_dll` helper in `sdk/cmake/gcc.cmake`). The
   header-dependent objects the cargo link reuses (uefildr: `strsafe_shim`;
   rosload: `strsafe_shim`+`cmboot`+asm) are kept compiled via OBJECT libs
   (`uefildr_shim` / `rosload_objs`) — the `-cargo` build.rs read from
   `<objlib>.dir/`. `boot/boot_images.cmake` was updated to embed uefildr.efi
   into the efisys image (and the efi/boot CD copy) by PATH + a `DEPENDS
   uefildr` (a custom_target has no `$<TARGET_FILE:>`). Cold smoke green ×3
   (i386/amd64: cargo uefildr + cargo rosload together — the first time both
   cargo loader binaries boot in the same image; arm64: cargo uefildr, no
   rosload). With kdcom/bootvid, FOUR modules' cmake links are now retired.

---

## Definition of done (updated)

- ✅ `freeldr-rust` produces working `uefildr.efi` (+ `rosload.exe`) for
  all three arches; all three smoke tests boot through it via UEFI.
- ✅ `arm64vid-rust` / `framebuf-rust` flipped, 3-arch, smoke green.
- ✅ `kdcom-rust` / `bootvid-rust` integrated into the default build (cargo
  link via `add_cargo_kernel_dll`, cmake link retired) and cold-smoke-green ×3.
- ⬜ `tools/scan_ud2.py --check` clean against each boot artifact.
- ⬜ Dead trees deleted: `boot/freeldr/bootsect/`, unused KD transports.
- ✅ (was Stretch) cmake C `uefildr`/`rosload` link retired in favour of the
  cargo-driven link, **wired into the main build** (2026-06-04): cmake's
  `add_executable` for both is gone; `add_cargo_module` drives the link
  (`uefildr-cargo` EfiApplication ×3 arch; `rosload-cargo` NativeExe i386+amd64,
  arm64 has no rosload). Cold smoke green ×3. Remaining cleanup: the
  `boot/freeldr/freeldr/` C source files the Rust bodies replaced are still on
  disk (never-delete rule) — a future `git rm` pass once the cargo path is the
  settled canonical.

## What this audit doesn't measure

- **Boot correctness.** Smoke is the source of truth; this doc tracks
  *translation* + *integration* status, not runtime correctness.
- **The cargo-link PoC vs cohabitation tension — RESOLVED (2026-06-04, still
  true 2026-06-21).** All four boot binaries (`uefildr.efi`, `rosload.exe`,
  `kdcom.dll`, `bootvid.dll`) are now cargo-link-driven and wired into the main
  cmake build via `add_cargo_module`/`add_cargo_kernel_dll`; the cmake
  `add_executable`/`add_library(MODULE)` links are retired. The body crates
  (`*-rust` staticlibs) feed the cargo link, so "translated" == "shipping" for
  the boot track.

---

## i386/x86 UEFI boot — operational recipe & fixes (reusable)

The non-obvious operational bits that make an x86 ReactOS kernel boot under UEFI
(OVMF) instead of legacy BIOS. These are arch-relevant gotchas, not status — they
recur any time the UEFI boot path is touched.

### i386 UEFI PAE → non-PAE paging transition

UEFI/OVMF hands control to the loader with **paging ON and `CR4.PAE=1`**. The i386
ReactOS kernel uses classic **non-PAE 2-level paging**. Doing `writecr3(PDE)` while
`CR4.PAE=1` makes the MMU interpret the non-PAE PDE as a PAE PDPT → instant
triple-fault (instruction-fetch #PF the moment paging re-reads the next insn).

The paging switch must be ordered so PAE is cleared while paging is off:

1. If `CR0.PG` set → **clear `CR0.PG`** (paging off)
2. **clear `CR4.PAE`**
3. **`writecr3`** the non-PAE PDE
4. **set `CR0.PG`** (paging back on)

Steps run with paging off, so they execute identity-mapped — safe because the
loader + stack live in identity-mapped low memory. No-op on legacy BIOS (paging is
already off when control reaches the equivalent point). This bug is **latent in the
original C too** (the `writecr3`/`writecr0` pair has no PAE handling) — it only
manifests once you actually boot via UEFI. (Lives in the i386
`WinLdrSetProcessorContext`.)

### OVMF boot recipe (i440fx + ide-cd)

Boot the LiveCD ISO from OVMF on the **i440fx `-M pc`** machine — *not* q35. Both
i386 and amd64 use the same shape (no AHCI/SATA needed; uniata serves the IDE CD):

- Firmware: i386 = `qemu/OVMF32_CODE.fd` + `OVMF32_VARS.fd`; amd64 =
  `/usr/share/OVMF/OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd`.
- Boot device: `-device ide-cd,drive=cd0,bootindex=0`. A bare `-cdrom` does **not**
  auto-boot — OVMF drops to the UEFI shell without an explicit `bootindex=0`.
- The writable VARS pflash **must be qcow2** (convert the raw OVMF VARS → qcow2)
  for QMP `savevm`/snapshot to work; a raw VARS pflash blocks snapshotting.
- The LiveCD ISO is already UEFI-bootable: the EFI El Torito entry loads
  `EFI/BOOT/boot<plat>.efi` (`EFI_PLATFORM_ID` ia32/x64/aa64), which folds in
  `uefildr` — re-run `ninja livecd` after `ninja uefildr` or you boot a stale ISO.

### Synthetic ACPI 2.0 RSDP (keyboard / PS/2 enumeration)

Under UEFI the RSDP lives in EFI-reclaim memory, so acpi.sys's legacy BIOS-area
RSDP scan (`0xE0000`–`0xFFFFF`) finds nothing → `AcpiInitializeTables` fails → no
ACPI namespace → `PNP0303`/`PNP0F13` (PS/2 keyboard/mouse) are never enumerated →
i8042prt gets no `AddDevice` → **no keyboard input**.

Fix (in `AcpiOsGetRootPointer`, `drivers/bus/acpi/osl.c`): keep the BIOS-area scan
first (legacy path byte-for-byte unchanged); on failure, read the RSDT/XSDT
physical address FreeLDR records in the ARC tree — the `"ACPI BIOS"`
multifunction-adapter Configuration Data (registry
`\HARDWARE\DESCRIPTION\System\MultifunctionAdapter\N`,
`CM_FULL_RESOURCE_DESCRIPTOR` → device-specific descriptor → trailing
`ACPI_BIOS_DATA.RSDTAddress`, which holds the XSDT address on ACPI 2.0+) — and
**synthesize a valid ACPI 2.0 RSDP** (correct 1.0 checksum + extended checksum)
pointing at it. Arch-agnostic: arm64 takes the same fallback safely.

Diagnosis tell: a BIOS boot's serial shows `[[I8K]] AddDevice ... KBD ISR FIRED`;
a broken UEFI boot has **zero** `[[I8K]]` lines and reports `Tables: [XSDT]` (vs
BIOS `[RSDT]`) plus `Unable to AcpiInitializeTables`.
