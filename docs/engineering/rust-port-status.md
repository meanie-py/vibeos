# ReactOS → Rust Port — Current As-Built Status

**A multi-architecture port of ReactOS (Windows-compatible OS) translated from C to
Rust, booting to a graphical desktop on i386, amd64, and ARM64.**

*Live status as of 2026-07-03, measured against `master` (the single mainline). This is
the headline as-built; the verbose per-area detail, historical snapshots, and project
timeline live in the [old-details rollup](rust-port-status-rollup.md), and the detailed
per-file reference in the `*-translation-audit.md` set. Re-measure before trusting later.*

> **Cohabitation caveat.** Convention is *never delete a translated C file*: a flipped
> `.c` is commented out in CMake and kept on disk as dead source; the Rust `#[no_mangle]`
> symbol takes over the link. So `find -name '*.c' | wc -l` counts **on-disk** C, not
> **compiled** C. Every "C remaining" figure below is compiled/active C unless marked
> *on-disk*.

## TL;DR

**The entire boot-critical OS — kernel, boot loader, drivers, and the usermode boot
chain — is Rust (glue-only, ~95–100% per area) and boots to a graphical desktop on all
three arches.** The remaining frontier is the **non-boot usermode DLL tail**
(gdi32/advapi32/userenv/ws2_32 usermode winsock, the CRT variadic floor, samsrv, the
shell32 COM half) — large, but none of it on the path to a desktop.

Tree inventory (2026-07-03): **~2,390 `.rs` files · 225 crates · 739 cohab flip markers.**

## Headline component ledger

| Tier | Status | % (area methodology) |
|---|---|---|
| **ntoskrnl.exe** | body 100% Rust at link; 8-file glue floor | **~100% body** |
| **ntdll.dll** | body Rust; cargo-driven link; per-arch `dispatch.S` + gen stubs | **~97%** |
| **win32k.sys** | body Rust; cargo-driven; glue = shims/floatobj/gendib | **~99%** |
| **hal** (x86 + arm64 stub) | 42+ cohab markers; `.S` NT-ABI + setjmp glue | **~97%** |
| **rtl** | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~99%** |
| **freeldr + boot DLLs** | 89 modules + cmlib/cport/cmboot; 16-bit `.S` floor | **~99%** |
| **drivers (in-scope)** | 50/52 done; only buslogic + ks remain C | **~96%** |
| **usermode boot chain** (smss/csrss/csrsrv/services/lsass/winlogon/userinit) | bodies 100% Rust, cargo-driven | **~95%** |
| **kernel32.dll** | body 100% Rust (heavy-FFI files translated) | **~100% body** |
| **user32.dll** | Wave 4 landed; DDE + spy.c deferred | **~95%** |
| **advapi32 / gdi32 / secur32 / shell32(wine `.c`)** | 100% of their translated-C surface flipped | **done (lane scope)** |
| **static-libs / cargo cmake-off** | kernel + 13-spine + drivers + win32k byte-equivalent | **~100%** |
| **usermode DLL tail** (userenv, ws2_32 usermode, CRT floor, samsrv, shell32 COM, wine libs) | **the remaining frontier** | **early-stage** |

## Remaining C translation backlog (the real work)

- **Filesystems — CLOSED.** msfs · npfs · cdfs (boots) · fastfat (boots arm64 desktop) ·
  fs_rec · mup. nfs/udfs/vfatfs dropped from kernel scope; btrfs/ext2/ntfs deleted.
- **Non-FS drivers — only 2 C-only left (both non-boot):** **buslogic** (legacy SCSI HBA,
  off all QEMU paths) and **ks** (kernel streaming, audio-gated).
- **Usermode / lib tail — the frontier:**
  - **kernel32/user32:** bodies Rust; user32 DDE (~5.4K) + spy.c (~2.7K) deferred.
  - **not yet started:** gdi32 further work, advapi32 tail, **userenv**, **ws2_32/msafd**
    (the *usermode* winsock DLLs — netstack reached the *kernel* TDI layer, not these),
    setupapi, samsrv (0%, no crate yet), dnslib (partial).
  - **CRT floor:** ~30–40K LOC C (printf/scanf/string/math/memory) — largely irreducible
    variadic.
  - **shell32 COM classes (81 `.cpp`, ~64K LOC) — not started.** The ATL/COM vtable
    implementations (CDefView / CShellLink / CDefaultContextMenu / folders / droptargets)
    are a separate, much heavier C++→Rust effort, out of scope for the C→Rust pipeline
    that finished shell32's 14 wine-derived `.c` files; likely needs a `shell32-sys`
    substrate and a COM-vtable translation approach.

## Open items / flagged regressions

- **arm64 kernel stack-size bump still owed.** `sdk/include/xdk/arm64/ke.h` reads the old
  `KERNEL_STACK_SIZE 12288` / `KERNEL_LARGE_STACK_SIZE 61440`; the intended bump to
  `0xC000`/`0x12000` (as amd64 carries) regressed out via a merge and has **not** been
  re-applied. Re-apply owed.
- ✅ **amd64 taskbar BUGCHECK 0x50 — FIXED** (2026-07-02, `d74739332d9`): the i386-width
  hand-computed offsets in win32k `ptCursor`/SERVERINFO reads now use typed bindgen reads.
- **arm64-net flaky 0xF4** (smss EL0 sysreg-write under net load) — non-blocking;
  plain-boot green.

## Scale

~2.9M LOC of translated Rust across kernel + drivers + usermode chain (ntoskrnl 451K,
win32k 256K, + the driver wave: cdrom 211K, disk 170K, classpnp 166K, kernel32 161K,
uniata 125K, scsiport/atapi/storahci ~120K each, fastfat 113K, freeldr 75K, cdfs 65K,
storport 65K, + ~150 smaller crates). Generated FFI crates (ntkrnl-sys, ntdll-sys,
kmdf-sys bindgen output) are large but are declarations, not translation.
