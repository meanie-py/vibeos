# Rust Port Status — Detailed Rollup (historical detail dump)

> **This is the old-details dump, not the live headline.** For the current as-built
> status see **[rust-port-status.md](rust-port-status.md)**. This doc keeps the verbose
> per-area detail, the measured snapshots, and superseded numbers we want on record but
> don't want cluttering the live status — a graveyard of detail, kept on purpose. The
> per-file reference lives in the `*-translation-audit.md` set.

**Snapshot date: 2026-06-21** (measured against mainline, then `continue-rust-port == master`;
`continue-rust-port` has since been retired and `master` is the single mainline).

This was a **ground-truth rollup** measured from the tree (CMake source lists + cohab
`# replaced by Rust` markers + Rust crate inventory) as of the date above. Numbers here are a
point-in-time snapshot — the live headline in `rust-port-status.md` is the current truth.

Methodology: per module, count active (uncommented) `.c` still added to the build target vs
`# replaced by Rust` markers vs the Rust crate's `.rs`/LOC. "Glue" = irreducible C that
can't/shouldn't be Rust (C-variadic ABI bridges, `.S`/`.s` asm, LLVM LTO workarounds,
`FORCEINLINE`/macro providers, `__FILE__/__LINE__` trampolines, syscall dispatch tables,
spec2def-generated stubs).

## Headline progress

**The entire boot-critical OS — kernel, boot loader, drivers, and the usermode boot chain —
is translated (glue-only, ~95–100% per area).** The remaining frontier is the **non-boot
usermode DLL tail** (gdi32/advapi32/userenv/ws2_32, the CRT variadic floor, samsrv) — large,
but none of it on the path to a graphical desktop.

Fresh tree inventory (2026-06-21): **1,992 `.rs` files · ~2.87M LOC · 207 crates · 594 cohab
flip markers** (was ~1.4M LOC at the 2026-06-17 rollup — the cdrom/disk/classpnp/uniata/
scsiport/atapi/storahci/storport driver wave, the kernel32 wave, and netstack all landed since).

| Tier | Status | % (area methodology) |
|---|---|---|
| ntoskrnl.exe | body 100% Rust at link; 8-file glue floor | **~100% body** |
| ntdll.dll | body Rust; per-arch `dispatch.S` + gen stubs | **~97%** |
| win32k.sys | body Rust; cargo-driven; glue = shims/floatobj/gendib | **~99%** |
| hal (x86) + arm64 stub HAL | 42 cohab markers; .S NT-ABI + setjmp glue | **~97%** |
| rtl | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~99%** |
| freeldr + boot DLLs | 89 modules + cmlib/cport/cmboot; 16-bit `.S` floor | **~99%** |
| drivers (in-scope) | 50/52 done; only buslogic + ks remain C | **~96%** |
| usermode boot chain (smss/csrss/csrsrv/services/lsass/winlogon/userinit) | bodies 100% Rust, cargo-driven | **~95%** |
| user32.dll | Wave 4 landed; DDE + spy.c deferred | **~95%** |
| kernel32.dll | body 100% Rust (heavy-FFI files translated) | **~100% body** |
| static-libs / cargo cmake-off | kernel + 13-spine + 25/25 drivers + win32k byte-equivalent | **~100%** |
| **usermode DLL tail** (gdi32, advapi32, userenv, ws2_32, CRT floor, samsrv, wine libs) | **the remaining frontier** | **early-stage** |

## Kernel proper — DONE (glue-only)

| Binary | Rust (files / LOC) | Active C remaining | Verdict |
|---|---|---|---|
| ntoskrnl.exe | 406 / 451K | 8 glue: io/ke/mm/kdbg `_rust_shims.c`, `variadic_shims.c`, `lto_workaround.c`, `mm_lock_outlines.c`, `ntprintf_trampoline.c` | **100% body** |
| ntdll.dll | 24 / 20K | 0 (per-arch `dispatch.S` asm + setjmp + generated stubs); cargo-driven link, 0 cmake `.a` | **~97%** |
| win32k.sys | 166 / 256K | ~5 `*_rust_shims.c` + i386 `floatobj.c`/asm + gendib tables; cargo-driven (`win32k_residue.a`) | **~99%** |
| hal (x86 i386+amd64) | 46 / 20K | `.S` NT-ABI offsets + i386 setjmp/longjmp V86 trampolines; arm64 = cargo-driven Rust stub HAL (~1.1K) | **~97%** |
| rtl | 20 algorithm rlibs + umbrella (94 `.rs` / ~57K) | `dbgprint_variadic.c` + per-arch asm | **~99%** |
| freeldr | 89 / 75K (+ rosload/uefildr cargo-driven, cmlib-rust, cmboot-rust, cport-rust) | `strsafe_shim.c` + 16-bit/boot `.S` (`drvmap.S`, `fat.S`, `fat32.S`) | **~99%** |

**✅ The "one real architectural blocker" from the prior rollup is GONE.** The x86
`captures(none)` layout-class wall (#35) that kept cportlib/csq/arbiter/lsalib **arm64-only**
is **fully dissolved** — all five (cport/csq/arbiter/lsalib/rossym) now link cargo-native Rust
**uniformly on i386/amd64/arm64** (root cause was an i386 stdcall/cdecl ABI mismatch in cmlib,
not a layout miscompile; commit `e8765d21008`). cmlib is 100% Rust end-to-end, kernel + loader.

## Usermode boot-chain — DONE (cargo-driven, zero cmake `.a`)

smss.exe · csrss.exe · csrsrv.dll · services.exe · lsass.exe · userinit.exe · winlogon.exe —
all 100% Rust bodies, cargo-driven links, zero reused C `.a`. winsrv.dll (consrv + usersrv +
concfg): 46 / 29K Rust; remaining C is shim/stub glue only. lsasrv: 5/16 units flipped (M0–M2);
msgina done bar a `dimmedwindow.cpp` C++ leaf. smlib/csrlib client-side flip is now in-tree.

## Remaining C translation backlog (the real work)

### Filesystems — CLOSED
msfs ✅ · npfs ✅ · cdfs ✅ (boots) · fastfat ✅ (boots arm64 desktop) · fs_rec ✅ · mup ✅.
nfs/udfs/vfatfs(FATX) **dropped from kernel scope** (3rd-party/niche/non-boot); btrfs/ext2/ntfs
were deleted from the tree (`14a0563b291`).

### Non-FS drivers — only 2 C-only left (both non-boot)
**buslogic** (legacy SCSI HBA, off all QEMU paths) · **ks** (kernel streaming, audio-gated).
Everything else done & landed (50/52 in-scope): cdrom (mainline `c2839ebaf5b`), disk, classpnp,
cdfs, fastfat, scsiport, atapi, uniata, pciide(x), partmgr, mountmgr, acpi (ACPICA FFI), pcix,
kdcom/kdgdb/kdvm, pl011, i8042prt, kbdclass, mouclass, hidparse(r), hidclass, hidusb, kbdhid,
mouhid, wmilib, bootvid, framebuf, beep, ramdisk, serial, swenum, usbd, condrv, storport,
storahci, arm64vid, dxg, nmidebug, null, + netstack (from-scratch smoltcp, not a C-translate).

### Usermode / lib tail — the remaining frontier
- **kernel32**: body 100% Rust (the previously-deferred console/comm/lzexpand/profile/tape/
  toolhelp/res are translated; only `kernel32.rc` + generated stubs are active C).
- **user32**: ~95% (DDE ~5.4K + spy.c ~2.7K deferred).
- **big DLLs not yet started**: gdi32, advapi32, userenv, ws2_32/msafd (still C — netstack
  reached them at the *kernel* TDI layer, the usermode winsock DLLs are unported), setupapi, etc.
- **crt**: ~30–40K LOC C floor (printf/scanf/string/math/memory) — largely irreducible variadic.
- **samsrv**: 0% (no crate yet); **dnslib**: partial.

## ⚠️ Flagged regressions / open items (from the 2026-06-21 audit sweep)
- **arm64 kernel stack-size bump regressed out of the tree.** `26408b0f33a` set 0xC000/0x12000
  but a later merge silently dropped the `ke.h` hunk — both build-cache and c-r-p read the old
  12288/61440. The section_table.rs half survived; the stack half did not. **Re-apply owed.**
- **amd64 taskbar BUGCHECK 0x50** still open (non-deterministic ~1/3, win32k child-control
  creation post-`shell ready`; runtime/logic, not a coverage gap).
- **arm64-net flaky 0xF4** (smss EL0 sysreg-write under net load) — non-blocking, plain-boot green.

## Rough scale
~2.87M LOC of translated Rust across kernel + drivers + usermode chain (ntoskrnl 451K, win32k
256K, + the driver wave: cdrom 211K, disk 170K, classpnp 166K, kernel32 161K, uniata 125K,
scsiport/atapi/storahci ~120K each, fastfat 113K, freeldr 75K, cdfs 65K, storport 65K, + ~150
smaller crates). Generated FFI crates (ntkrnl-sys, ntdll-sys, kmdf-sys bindgen output) are large
but are declarations, not translation, and excluded where noted.
