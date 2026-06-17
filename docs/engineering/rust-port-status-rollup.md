# Rust Port Status — Consolidated Rollup (ground-truth)

**Date: 2026-06-17.** Branch `continue-rust-port` (== `master`, FF-synced).

This is a **ground-truth rollup** measured from the tree (CMake source lists + cohab
`# replaced by Rust` markers + Rust crate inventory), produced after a ~100-commit swarm
wave left the per-subsystem audit docs stale. It supersedes the scattered status claims
in the per-area `*-translation-audit.md` set for *headline* numbers; those docs remain the
detailed per-file reference. Where they disagree with this rollup, **this rollup is the
measured truth as of the date above** (re-measure before trusting later).

Methodology: for each module, count active (uncommented) `.c` still added to the build
target vs `# replaced by Rust` markers vs the Rust crate's `.rs`/LOC. "Glue" = irreducible
C that can't/shouldn't be Rust (C-variadic ABI bridges, `.S`/`.s` asm, LLVM LTO
workarounds, `FORCEINLINE`/macro providers, `__FILE__/__LINE__` trampolines, syscall
dispatch tables, spec2def-generated stubs).

## TL;DR

- **The kernel proper is translated.** ntoskrnl / ntdll / win32k / hal / rtl / freeldr have
  **zero untranslated subsystem-body C** — only the irreducible glue floor remains.
- **Remaining C backlog is in peripheral drivers + a usermode tail**, none of it boot-critical:
  filesystems (~110K LOC, 5 drivers), non-FS leaf drivers (~20K LOC, 9 drivers), the crt
  variadic floor (~30–40K LOC, largely irreducible), kernel32's deliberately-deferred
  heavy-FFI subsystems, and dnslib (partial).
- **One real architectural blocker:** the x86 `captures(none)` layout-class wall (staticlibs
  #35) keeps cportlib/csq/arbiter/lsalib Rust bodies linked on **arm64 only**; i386/amd64
  still link the C. Rust exists; it's a link-gate, not a translation gap.

## Kernel proper — DONE (glue-only)

| Binary | Rust (files / LOC) | Active C remaining | Verdict |
|---|---|---|---|
| ntoskrnl.exe | 406 / 451K | 8 glue: io/ke/mm/kdbg `_rust_shims.c`, `variadic_shims.c`, `lto_workaround.c`, `mm_lock_outlines.c`, `ntprintf_trampoline.c` | **100% body** |
| ntdll.dll | 25 / 20K | 0 (per-arch `dispatch.S` asm + generated `ntdll_stubs.c`) | **100% body** |
| win32k.sys | 166 / 256K | ~7 glue: `main_rust_shims.c` (Win32k SSDT/SSPT), GDI FORCEINLINE wrappers, `gdi/eng/i386/floatobj.c` | **~96%, no untranslated logic** |
| hal (x86 i386+amd64) | 57 / 21K | `lib/strsafe_shim.c` variadic + 5 `.S` (NT-ABI offsets); arm64 = separate Rust stub HAL (~0.9K LOC) | **100% substantive** |
| rtl | ~20 algorithm rlibs (umbrella `rtl-rust`) | `dbgprint_variadic.c` + per-arch asm | **~100%** |
| freeldr | 89 / 75K (+ rosload/uefildr cargo-driven, cmlib-rust 15/7K, cport-rust) | `strsafe_shim.c` + 16-bit/boot `.S` (`drvmap.S`, `fat.S`, `fat32.S`) | **~99%** |

## Usermode boot-chain — DONE (cargo-driven, zero cmake `.a`)

smss.exe · csrss.exe · csrsrv.dll · services.exe · lsass.exe · userinit.exe — all 100% Rust
bodies, cargo-driven links, zero reused C `.a`. winlogon.exe is mostly Rust (GINA/SAS cores
flipped) with some C residue compiled in-cargo (`c_from_compile_db`), not linked as external `.a`.

winsrv.dll (consrv + usersrv + concfg): 46 / 29K Rust; remaining C is shim/stub glue only.

**Caveat:** the smlib/csrlib **client-side** flip is on an unmerged branch
(`smlib-csrlib-client-flip`); server-side is Rust, client-side still C in ntdll on c-r-p.

## Remaining C translation backlog (the real work)

### Filesystems — ~110K LOC, 5 drivers, no Rust crate yet
| FS | C/C++ LOC | |
|---|---|---|
| udfs | 52K | UDF (C++) |
| vfatfs | 28K | VFAT/FAT-LFN |
| nfs | 16K | NFS 4.1 client |
| npfs | 12K | named pipes |
| msfs | 2K | mailslots |

Done: **cdfs** (35/130K, near-100% — a couple cohab shims), **fastfat** (40/225K), **fs_rec**
(19/10K), **mup** (6/5K). Note: btrfs/ext2/ntfs were **deleted** from the tree (`14a0563b291`).

### Non-FS drivers — ~20K LOC, 9 C-only (all non-boot-critical)
storport (3.3K) · storahci (2.6K, disabled) · serial (2.4K, no QEMU serial console) ·
ramdisk (2.5K) · buslogic (2.8K, legacy SCSI) · hidclass (2.8K) · hidusb (1.9K) · kbdhid
(1.1K) · mouhid (1.3K). **Done & landed (22 drivers):** cdrom*, disk, classpnp, pciide,
pciidex, atapi, uniata, scsiport, mountmgr, partmgr, acpi (ACPICA vendored/FFI-wrapped),
pcix, kdcom, kdgdb, kdvm, pl011, i8042prt, kbdclass, mouclass, hidparse, wmilib.
(*cdrom landed to mainline `c2839ebaf5b`.)

### Usermode / lib tail
- **crt**: ~30–40K LOC C floor (printf/scanf/string/math/memory) — largely irreducible variadic;
  Rust half is `cnt-rust`/`crt-startup-rust`/`crt0-rust`.
- **kernel32**: 6–7 deliberately-deferred heavy-FFI files (console, comm, lzexpand, profile,
  tape, toolhelp, res) — `.rs` translations exist on disk for console; deferred on FFI-type gaps.
- **dnslib**: 9 of 17 `.c` still C (name/print/record/rrprint/table/debug/dnsutil/ip6/utf8).
- **x86-ungate-blocked** (Rust exists, links arm64-only): cportlib, csq, arbiter, lsalib —
  blocked by staticlibs #35 `captures(none)` layout-class wall on i386/amd64.

## Stale per-area audit-doc claims (flagged for owners — NOT yet patched here)

| Doc | Stale claim | Reality |
|---|---|---|
| `win32k-translation-audit.md` | "4 live C files: safe_user_io.c, user/rtl/{image,text}.c, dib_new.c" | all 4 already `# replaced by Rust` |
| `driver-translation-audit.md` | classpnp IN-FLIGHT / merge-to-crp unconfirmed | LANDED on c-r-p (`f0d69257cf1` et al.) |
| `freeldr-translation-audit.md` | "86 Rust modules" | 89 in tree |
| `smss-csrss-translation-audit.md` | winlogon/services/userinit dated 2026-06-01 | all 100% Rust as of ~2026-06-09 |
| `ntoskrnl / ntdll / ntdll-cargo / hal / rtl / static-libs` | — | **consistent** (static-libs refreshed 2026-06-15) |

## Rough scale

~1.4M+ LOC of translated Rust across the kernel + drivers + usermode chain (ntoskrnl 451K,
win32k 256K, fastfat 225K, kernel32 159K, cdfs 130K, freeldr 75K, winsrv 29K, hal 21K,
ntdll 20K, + ~30 smaller crates). Generated FFI crates (ntkrnl-sys, ntdll-sys bindgen output)
are large but are declarations, not translation, and excluded from that count.
