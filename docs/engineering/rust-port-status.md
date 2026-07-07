# ReactOS → Rust Port — Current As-Built Status

**A multi-architecture port of ReactOS (Windows-compatible OS) translated from C to
Rust, booting to a graphical desktop on i386, amd64, and ARM64.**

*Live status as of 2026-07-03, re-measured against `master` (the single mainline) by a
fresh 5-surface tree audit (kernel, win32k, userland DLLs, boot-userland exes, static
libs/RTL, drivers). This is THE single source of truth for Rust-vs-C coverage — the seven
per-surface `*-translation-audit.md` files were superseded by this pass and moved to
[`archive/`](archive/) (their per-file counts were ~12 days stale). Historical snapshots +
project timeline live in the [old-details rollup](rust-port-status-rollup.md). Re-measure
before trusting later.*

> **Cohabitation caveat.** Convention is *never delete a translated C file*: a flipped
> `.c` is commented out in CMake (`# replaced by Rust`) and kept on disk as dead source;
> the Rust body (`#[no_mangle]` symbol or cargo-driven link) takes over. So
> `find -name '*.c' | wc -l` counts **on-disk** C, not **compiled** C. Every "C remaining"
> figure below is compiled/active C unless marked *on-disk*.

## TL;DR

**The entire boot-to-desktop path is Rust.** Kernel, win32k, the usermode boot spine, the
core client DLLs, and the driver fleet are all ~90–100% translated with the replaced C
*removed from the compile* (not overridden-in-place). The OS boots to a graphical desktop
on all three arches with an essentially all-Rust runtime.

The remaining C is five well-bounded categories, **none on the boot-critical path**: (1)
the desktop **shell `explorer.exe`** (still 100% C++), (2) the bulk userland **CRT**
(msvcrt/ucrt leaf library), (3) **vendored/by-policy** C (ACPICA, FreeType), (4) **legacy
dead** C superseded by Rust (videoprt, old NDIS/tcpip), and (5) **irreducible glue**
(variadic trampolines, arch `.S`, generated RPC/syscall stubs).

Tree inventory (2026-07-03): **~2,422 `.rs` files · 228 crates** under `sdk/lib/rust/`.

## Headline component ledger

| Surface | Status | Coverage |
|---|---|---|
| **ntoskrnl.exe** (kernel) | 344 subsystem `.c` removed from build; 8-file Rust-glue floor | **~100%** |
| **win32k.sys** (GUI) | 166 `.rs`/256K LOC; 159 `.c`/`.cpp` flipped; glue = shims/floatobj/gendib | **~100%** |
| **ntdll.dll** | body Rust; cargo-driven link; per-arch `dispatch` + gen stubs | **~97%** |
| **Boot spine** smss/csrss/csrsrv/winlogon/services/lsass/userinit | bodies 100% Rust, cargo-driven, zero C in link | **~100%** |
| **user32 / gdi32 / secur32 / msafd** | 0 active body C; cohab-flipped | **~100%** |
| **kernel32.dll** | body Rust; only NLS `locale.c` + resources left | **~95%** |
| **advapi32.dll** | only `wine/security.c` + generated RPC stubs left | **~90%** |
| **msgina.dll** | 7 `.c` flipped; one C++ leaf (`dimmedwindow.cpp`) | **~88%** |
| **hal** (x86 + arm64 stub) | 42+ cohab markers; `.S` NT-ABI + setjmp glue *(not re-verified this pass)* | **~97%** |
| **rtl** static lib | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~100%** |
| Cohab helper libs (cmlib, rossym, evtlib, inflib, …) | real `.c` flipped for kernel/cross consumers | **~100%** |
| **freeldr + boot DLLs** | modules + cmlib/cport/cmboot; 16-bit `.S` floor *(not re-verified this pass)* | **~99%** |
| **Driver fleet** (storage, FS, input, bus, netstack) | see below — netstack is a from-scratch Rust rewrite; rest cohab-flipped | **~90–100%** |
| **LSA/SAM server** (lsasrv / samsrv) | active frontier — samsrv ~90%, lsasrv ~50%+ | **partial** |
| **explorer.exe** (desktop shell) | 100% C++, **no Rust crate** | **0%** |
| **CRT** (msvcrt/ucrt body) | 581 `.c` still compiled; only entry-glue is Rust | **~0–5%** |

### Driver fleet detail
Essentially the entire boot/smoke driver fleet is Rust. **`netstack.sys` is the only
from-scratch Rust rewrite (100%, zero C, replacing the legacy NDIS/tcpip stack).**
Everything else is a cohab flip (body in a `*-rust` crate, thin C residue —
`guid.c`/data-TU/stub — so ~90–100% each): all storage class/port/ide
(uniata, scsiport, classpnp, disk, cdrom, ramdisk, storport, storahci, **buslogic**,
atapi, pciide(x), mountmgr, partmgr), all boot-path filesystems (fastfat, cdfs, fs_rec,
mup, msfs, npfs), input (i8042prt, kbdclass, mouclass, arm64input), bus (pci, pcix, acpi
glue), display (framebuf, arm64vid, bootvid), **afd**, **ks**, ksecdd, fltmgr, and the USB
HC/hub/function stack. **Still pure C (not targets):** `videoprt.sys` (legacy INT10, dark
under UEFI), the vendored **ACPICA** core (155 `.c` inside acpi.sys, kept by policy), and
the legacy **NDIS/tcpip/tdi** stack (dead, superseded by netstack).

## Remaining C translation frontiers (the real work)

- **`explorer.exe` — 0%.** The desktop shell, 22 `.cpp`, no crate. The one boot-path
  component still fully untranslated. A C++→Rust effort (like shell32's COM half).
- **CRT floor — ~0–5%.** `sdk/lib/crt/` = 581 `.c` (printf/scanf/string/math/locale/
  mbstring/wine) still compiled; only `crt-startup/dllmain/crt0` entry-glue + the
  `libcntpr` kernel subset (`cnt-rust`) are Rust. Largely irreducible variadic.
- **LSA/SAM server lane — in flight.** samsrv (~90%, `samsrv-rust` 16 `.rs`) and lsasrv
  (~50%+, `lsasrv-rust` 20 `.rs`) are the active server-side frontier.
- **kernel32/advapi32 tails.** kernel32 NLS `locale.c` + resources; advapi32 `wine/
  security.c` + generated RPC.
- **shell32 COM classes (81 `.cpp`, ~64K LOC) — not started.** ATL/COM vtable impls
  (CDefView / CShellLink / folders / droptargets) — a heavy C++→Rust effort separate from
  the C→Rust pipeline that finished shell32's 14 wine-derived `.c` files.
- **Vendored, by policy (NOT targets):** ACPICA (~155 `.c`), FreeType (~208K LOC).

## Open items / flagged regressions

- **arm64 kernel stack-size bump still owed.** `sdk/include/xdk/arm64/ke.h` reads the old
  `KERNEL_STACK_SIZE 12288` / `KERNEL_LARGE_STACK_SIZE 61440`; the intended bump to
  `0xC000`/`0x12000` (as amd64 carries) regressed out via a merge and has **not** been
  re-applied. Re-apply owed.
- **Documented boot flakes** (catalogued in `tests/qemu/cli/triage.py`, root fixes in
  flight on `flake-*` branches): `nettest_explorer_race` (winlogon nettest spawn races
  the explorer handoff — i386 + arm64), `arm64_lpcexitthread_race`, `tWinMain_0xF4_race`,
  and the amd64 EventLog/storage stall.
- ✅ **amd64 taskbar BUGCHECK 0x50 — FIXED** (2026-07-02, `d74739332d9`).

## Scale

~3.58M LOC of translated Rust across kernel + drivers + usermode chain, + ~1.43M LOC of
generated FFI (`-sys` bindgen: ntkrnl-sys 444K, win32k-sys 379K, winsrv-sys 192K, ntdll-sys
128K, … — declarations, not translation). The two *hot* monoliths dwarf everything for
edit-churn: **win32k 257K** and **the kernel** (see crate-split note). The driver wave is
big but cold: cdrom 211K, disk 170K, classpnp 166K, kernel32 161K, buslogic 138K, uniata
125K, scsiport/atapi/storahci ~120K each, fastfat 113K, freeldr 75K, cdfs 65K, storport 65K,
+ ~130 smaller crates.

## Kernel crate topology (build-architecture, not coverage)

`ntoskrnl-rust` is being split from one ~456K-LOC rlib monolith into per-subsystem crates to
cut the edit→rebuild loop (rustc frontend is whole-crate). **As of 2026-07-07 it is ~63%
extracted:** the monolith is down to **170K LOC**, with **~286K across 21 crates** (iomgr, se,
config, pnpmgr, ps, ob, fsrtl, kdbg, fstub, kd64, rtlk, cc, po, dbgk, lpc, kd, wmi, inbv, vdm,
vf, dbgprint) severed and talking over the FFI symbol boundary. Remaining inside the monolith:
the dense center **ke (58K) + mm/ARM3 (59K)**, plus **ex (41K) extracted-but-parked** on an
x86 boot hang. win32k (257K, flat single crate) is the planned follow-on. Full as-built +
recipe + forward plan: [`ntoskrnl-crate-split-plan.md`](ntoskrnl-crate-split-plan.md). This
is orthogonal to C-vs-Rust coverage — the linked `.exe` is functionally identical.
