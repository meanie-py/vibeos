# ReactOS → Rust Port — Current As-Built Status

**A multi-architecture port of ReactOS (Windows-compatible OS) translated from C to
Rust, booting to a graphical desktop on i386, amd64, and ARM64.**

*Live status as of 2026-07-03, re-measured against `master` (the single mainline) by a
fresh 5-surface tree audit (kernel, win32k, userland DLLs, boot-userland exes, static
libs/RTL, drivers). This is THE single source of truth for Rust-vs-C coverage — the seven
per-surface `*-translation-audit.md` files were superseded by this pass and moved to
[`archive/`](archive/) (their per-file counts were ~12 days stale). Historical snapshots +
project timeline live in the [old-details rollup](archive/rust-port-status-rollup.md). Re-measure
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

The remaining C is four well-bounded categories, **none on the boot-critical path**: (1) the
bulk userland **CRT** (msvcrt/ucrt leaf library — the assemble-from-`cnt-rust`+`rtl-rust`
track), (2) **vendored/by-policy** C (ACPICA, FreeType), (3) **legacy dead** C superseded by
Rust (videoprt, old NDIS/tcpip), and (4) **irreducible glue** (variadic trampolines, arch
`.S`, generated RPC/syscall stubs). *(The desktop shell `explorer.exe` is NO LONGER a
remnant — it's ~99% Rust as of the crate-split era; see the ledger below.)* The broad
still-C userland frontier is the **~140 ROS-native DLLs (~240K LOC)** + the **Wine-derived
bucket (~154 DLLs, ~1.05M LOC)** catalogued in the roadmap's Phase 0 Tier 3.

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
| **ole32.dll** (COM runtime) | `ole32-rust`: the COM core (compobj + marshal/stubmanager/rpc/ftmarshal/git) **plus the whole mechanical ring** — the task allocator + streams + lockbytes (`ifs`/`hglobalstream`/`mem`+`filelockbytes`), the Running Object Table + all 6 monikers, `bindctx`/`errorinfo`/`enumx`/`dictionary`, `ole2`/`ole2impl`/`ole32_main` (OleInitialize/DoDragDrop/OleCreate), `clipboard`, `comcat`, `oleproxy`, and the NDR `usrmarshal`. **29 TUs flipped**; boots ×3. Remaining C: the Tier-D storage/embedding cluster (storage32, stg_prop, stg_stream, datacache, defaulthandler, oleobj) — translated + staged, not yet integrated. `guid.c` stays C permanently (an `initguid.h` TU whose job is emitting the moniker CLSIDs as data) | **core done** |
| **oleaut32.dll** (OLE automation) | `oleaut32-rust`: **the whole automation surface is Rust** — BSTR (`oleaut`), VARIANT (`variant`), the VarXxxFromYyy coercion matrix (`vartype`), SAFEARRAY (`safearray`), the typelib reader/writer + ITypeInfo/ITypeLib (`typelib` + submodules), IDispatch (`dispatch`), user-marshal (`usrmarshal`), `hash`. 8 TUs flipped; boots ×3. Remaining C is off the import surface (olefont/olepicture/varformat/connpt/recinfo/olepropframe) | **core done** |
| **hal** (x86 + arm64 stub) | 42+ cohab markers; `.S` NT-ABI + setjmp glue *(not re-verified this pass)* | **~97%** |
| **rtl** static lib | 80/82 `.c` flipped; `dbgprint_variadic.c` + asm floor | **~100%** |
| Cohab helper libs (cmlib, rossym, evtlib, inflib, …) | real `.c` flipped for kernel/cross consumers | **~100%** |
| **freeldr + boot DLLs** | modules + cmlib/cport/cmboot; 16-bit `.S` floor *(not re-verified this pass)* | **~99%** |
| **Driver fleet** (storage, FS, input, bus, netstack) | see below — netstack is a from-scratch Rust rewrite; rest cohab-flipped | **~90–100%** |
| **LSA/SAM server** (lsasrv / samsrv) | active frontier — samsrv ~90%, lsasrv ~50%+ | **partial** |
| **explorer.exe** (desktop shell) | `explorer-rust` 22 `.rs`/22K LOC; 22 cohab markers; 2 glue `.cpp` left (settings_shim, util_vararg) | **~99%** |
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

## Beyond a port — Rust-native capabilities & correctness (the differentiators)

*Added 2026-07-10. Coverage above is "how much C became Rust." This section is the part
that isn't just a port — capabilities and correctness properties that go past upstream
ReactOS, made tractable because the kernel is now our Rust in one workspace. Landed-vs-in-
flight is marked; in-flight lanes live in [`active-lanes/`](active-lanes/).*

- **x86-on-ARM64 — a homegrown Rosetta (WoW64 + FEX).** 32-bit x86 PE binaries launch as a
  **WoW64 process on the ARM64 NT kernel** — Phase B complete + Phase C M1 landed (real
  i386 PE runs through the full WoW64 stack: `wow64.dll`/`wow64cpu`/`wow64win`, generated
  syscall thunks). M3b (ntdll32 loader → `RtlCreateActivationContext`) is in flight on
  `fex-arm`, currently blocked on a foundational arm64 unaligned-access fault. The x86→arm64
  execution engine is **FEX-emu** (vendored FEX-2607, `wow64fex`/`FEXOfflineCompiler` link
  with zero patches; `fexprobe` executes an x86 guest blob) — the "run real x86 Windows
  binaries on an ARM64 Rust kernel" thesis. Docs: `active-lanes/wow64-plan.md`,
  `active-lanes/fex-bringup-plan.md`.
- **SMP / multi-processor bring-up.** The `CONFIG_SMP` kernel (`ntkrnlmp.exe`) **links on
  all three arches and boots `-smp 1` to desktop (amd64)** — landed. The **first amd64 AP
  starts and runs `KiSystemStartup`** (CPU 1 parks) on the `smp-bringup` lane; M2 is pinned
  on an AP-path `ZwOpenFile(ntdll)` 0x6B. This is the pressure test that makes every Rust
  spinlock/queued-spinlock/pushlock/interlocked path run under *real* concurrency for the
  first time (UP made them ≈ IRQL raises). Doc: `active-lanes/smp-bringup-quest.md`.
- **Compile-time & RAII correctness the C never had.** IRQL/spinlock lifetimes are
  **RAII-guarded** (`ntkrnl-sys::guards` — `IrqlGuard`/`SpinLockGuard`, released on `Drop`,
  the idiomatic replacement for `__finally`); a 504-site audit found **0 leaks**. SEH
  catch-at-a-distance is being replaced by **validate-first `Result<T>` probes** (`probe.rs`;
  SE + OB migrated) and callee-side `try_*` contract conversion, per `dummy-pseh-verdict.md`
  — the honest Rust shape of `__try/__except`. arm64 carries **hardware-backed IRQL** via
  `GICC_PMR` (lazy — no MMIO in the PASSIVE..DISPATCH band, so spinlocks stay register-only).
  A planned **typestate CPU-ownership model** (`archive/execution-context-ownership-model.md`)
  would make context-switch-safety a compile-time property.
- **Runtime correctness tripwires unique to a Rust kernel.** Bespoke always-on / opt-in
  instrumentation no stock NT has, enabled because we own every symbol: **waitdep**
  (dispatcher wait-block chain integrity at `KiUnlinkThread`/`KiWaitTest`, BUGCHECKs 0x109
  with a full chain dump at the corruption point — field manual §9.27/§9.31/§9.36),
  **lockdep** (pushlock self-deadlock + ABBA lock-order graph, `--lockdep`), **cswitchdep**
  (context-switch safety), per-boot **special-pool** tripwires (`--tripwire SPECIALPOOL=*`),
  and a post-opt **captures(none) IR audit** (`scan_captures.py`). See
  `kernel-debugging-infra.md`.
- **ABI-drift elimination (in flight).** The **exported-symbol trampoline refactor** (roadmap
  Phase 0 Tier 2) splits every export into a native snake_case `nt_foo()` (plain Rust ABI,
  holds the logic) + a generated `NtFoo` calling-convention trampoline — so no hand-written
  `extern` decl is left to drift, killing the ABI-mismatch `ud2`/`noreturn` cascade class at
  the source (today caught reactively by `verify_ffi` + `scan_cc_mismatch`).

## Remaining C translation frontiers (the real work)

- **ROS-native userland DLLs — ~140 components, ~240K LOC.** The real resume-translation
  queue (Winsock/net, setup/PnP/device, ~15 control panels, services, security/auth, shell
  exts, audio/ACM). Batched in the roadmap Phase 0 Tier 3. *(`explorer.exe` is DONE — ~99%
  Rust, `explorer-rust`; only 2 glue `.cpp` remain.)*
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

- ✅ **arm64 kernel stack-size bump — FIXED/re-landed** (`7e9d4644e53`, 12KB→48KB;
  `KERNEL_STACK_SIZE 0xC000` / `KERNEL_LARGE_STACK_SIZE 0x12000`, amd64 parity).
- ✅ **amd64 taskbar BUGCHECK 0x50 — FIXED** (2026-07-02, `d74739332d9`).
- **Live open bugs & flakes** are now tracked in one place — the roadmap **Known bugs &
  flakes registry** (`docs/product/vibeos-roadmap.md`) — with field-manual § pointers:
  the arm64 0x1E/`[WAITDEP]` ALL-ONES residual (§9.36 addendum), the winlogon 0x4C
  basesrv-population race, the win32k `DialogBoxParamW` race, and the amd64 start-menu
  render residue. (The old `triage.py` flake catalog was largely retired as its entries
  were root-caused — see the field manual.)

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
