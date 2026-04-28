# ntoskrnl C→Rust Translation Audit (updated 2026-04-28)

## Already Translated (Active in Rust)

| Subsystem | Files | Rust Lines | Status |
|-----------|-------|------------|--------|
| KE (kernel) | 63 | 53,461 | All arches, i386/arm64 smoke pass |
| ARM3 (mm internals) | 25 | 43,638 | All arches |
| IOMGR (IO manager) | 26 | 40,508 | All arches (arcname re-enabled, iofunc complete) |
| EX (executive) | 31 | 38,613 | All arches |
| SE (security) | 18 | 34,868 | All arches |
| PNPMGR (PnP) | 18 | 25,755 | amd64/arm64 Rust, i386 C (devaction stays C) |
| OB (object) | 11 | 22,332 | All arches |
| PS (process) | 13 | 22,604 | All arches |
| FSTUB (file stubs) | 4 | 12,060 | All arches (this session) |
| MM (memory) | 11 | 11,221 | All arches |
| FSRTL (fs runtime) | 17 | 20,785 | All arches |
| CC (cache) | 7 | 6,112 | All arches |
| DBGK (debug) | 2 | 5,061 | All arches |
| PO (power) | 4 | 4,961 | All arches |
| KD (low-level debugger) | 9 | 3,914 | All arches |
| KDBG (interactive debugger) | 7 | 9,991 | i386 + amd64 (off on ARM64) |
| LPC (local proc) | 8 | 3,876 | All arches |
| INBV (boot video) | 3 | 3,611 | All arches |
| WMI | 4 | 3,588 | All arches |
| RTL (runtime) | 2 | 2,157 | All arches |
| VDM (virtual DOS) | 2 | 1,221 | All arches |
| VF (verifier) | 1 | 140 | All arches |
| MM section I/O+page-table | 2 | 540 | All arches; wired in mm/mod.rs, replaces cache/section/{io,sptab}.c |
| **TOTAL** | **281** | **~360,500** | |

## NOT YET TRANSLATED

### 1. CONFIG — Registry/Configuration Manager (CM)
**24 files, 18,827 lines — PRIORITY: HIGH**

The Windows Registry implementation. Used on every boot and throughout runtime.

Current status as of 2026-04-28:
- Translation lives on branch `wip-config-rust-port` (tip: `2668b739 WIP: config/ Rust translation — boots-broken, bisected to LLVM CGU regression`). Not merged into `continue-rust-port`; the C config/ files are still active in this branch's build.
- On the WIP branch: 21 .rs files in `sdk/lib/rust/ntoskrnl-rust/src/config`, compile-clean on `x86_64-pc-windows-gnu`, but boot is broken and bisected to the LLVM CGU/captures(none) regression (see `feedback_cgu1.md`).
- Parity fixes already landed there: `ntapi.rs`, `cmsysini.rs`, `cmconfig.rs`, `cmparse.rs`, `cmhvlist.rs`, `cmvalche.rs`, `cminit.rs`, `cmcontrl.rs`. Small files (`cmquota.rs`, `cmsecach.rs`, `cmhook.rs`, `cmmapvw.rs`, `cmnotify.rs`) are aligned/stubbed at C parity.
- Highest remaining audit risk is concentrated in `cmapi.rs`, `cmboot.rs`, and `cmkcbncb.rs`, where translation-era local stubs and large control-flow surfaces still need deeper line-by-line verification.

| File | Lines | Description |
|------|-------|-------------|
| cmapi.c | 2,846 | Registry API (NtCreateKey, NtSetValueKey, etc.) |
| cmsysini.c | 2,418 | System initialization, hive loading |
| cmparse.c | 2,311 | Key path parsing, create/open |
| ntapi.c | 1,953 | Nt* syscall entry points for registry |
| cmkcbncb.c | 1,423 | Key control block / name cache block |
| cmboot.c | 1,260 | Boot hive processing |
| cmdata.c | 985 | Static data, type info |
| i386/cmhardwr.c | 897 | Hardware detection (arch-specific) |
| cmvalche.c | 836 | Value cache |
| cminit.c | 689 | CM initialization |
| cmdelay.c | 474 | Delayed operations |
| cmlazy.c | 452 | Lazy flush |
| cmconfig.c | 430 | Configuration tree |
| cmse.c | 355 | Security descriptors for registry |
| cmalloc.c | 303 | Memory allocation |
| cmhvlist.c | 288 | Hive list management |
| cmcontrl.c | 271 | Control set |
| cmwraprs.c | 217 | Wrapper functions |
| cmhook.c | 198 | Registry callbacks |
| cmmapvw.c | 80 | Mapped views |
| cmsecach.c | 52 | Security cache |
| cmnotify.c | 36 | Change notifications |
| cmquota.c | 26 | Quota |
| arm/cmhardwr.c | 27 | ARM hardware detection |

**Dependencies**: cmlib (sdk/lib/cmlib/), registry hives, SE subsystem
**Complexity**: HIGH — complex data structures (hives, KCBs, bins), extensive locking, lazy flush

### 2. FSTUB — File System Stubs — DONE (commit `d171f628`, 2026-04-26)
**4 files, 5,817 LOC C → 12,060 LOC Rust on all 3 arches.**

Disk partition support and HAL stubs. Used during boot for disk enumeration.
Wired in `lib.rs:44` (`pub mod fstub;`); C versions commented out at
`ntos.cmake:135-138`. All 4 arches green; the amd64 explorer/taskbar
regression that had been chased for weeks turned out to be a translation
bug here (`L"\\Device\\Harddisk%d\\Partition0"` translated as bytes cast
to `*const u16` produced garbage paths every other character) — fixed in
the same commit and caught by the new boot-milestone harness.

| C file | C lines | Rust file | Rust lines |
|--------|---------|-----------|------------|
| disksup.c | 2,596 | fstub/disksup.rs | 6,020 |
| fstubex.c | 2,136 | fstub/fstubex.rs | 4,641 |
| translate.c | 652 | fstub/translate.rs | 430 |
| halstub.c | 433 | fstub/halstub.rs | 969 |

### 3. KD64 — WinDbg Kernel Debugger Protocol
**11 files (7 generic + 3 arch), 6,846 lines — PRIORITY: DEFERRED**

Rust translations exist (8 .rs files) but are **disabled** due to recursive exception flooding on amd64 (INT3 → KdpTrap → KdpReport → KdSendPacket → infinite nesting). Root cause unknown.

| File | Lines | Rust? | Description |
|------|-------|-------|-------------|
| kdapi.c | 2,236 | kdapi.rs exists | Debugger API |
| kddata.c | 1,368 | kddata.rs exists | Static data |
| kdtrap.c | 777 | kdtrap.rs exists | Trap handling |
| kdprint.c | 633 | kdprint.rs exists | Print/output |
| kdinit.c | 576 | kdinit.rs exists | Initialization |
| kdlock.c | 418 | kdlock.rs exists | Locking |
| kdbreak.c | 302 | kdbreak.rs exists | Breakpoints |
| kd64_rust_shims.c | 260 | — | C shims |
| amd64/kdx64.c | 153 | — | amd64-specific |
| i386/kdx86.c | 90 | — | i386-specific |
| arm/kdarm.c | 33 | — | ARM-specific |

**Status**: Translated but disabled. Needs debugging of recursive exception issue, not retranslation.

### 4. KDBG — Interactive Kernel Debugger (KDB) — DONE (2026-04-28)
**7 generic files, 7,653 LOC C → 9,991 LOC Rust on i386 + amd64.**

ReactOS-specific interactive debugger (not WinDbg protocol). Wired in
`lib.rs` (`pub mod kdbg;`), C versions commented out at `ntos.cmake:518-525`.
Smoke 7/7 on both arches. See `docs/engineering/kdbg-port-status.md`.

| C file | C lines | Rust file | Rust lines |
|--------|---------|-----------|------------|
| kdb_cli.c | 3,711 | kdbg/kdb_cli.rs | 3,656 |
| kdb.c | 1,682 | kdbg/kdb.rs | 1,784 |
| kdb_expr.c | 1,216 | kdbg/kdb_expr.rs | 2,283 |
| kdb_symbols.c | 469 | kdbg/kdb_symbols.rs | 1,026 |
| kdbg.c | 212 | kdbg/kdbg.rs | 429 |
| kdb_print.c | 201 | kdbg/kdb_print.rs | 413 |
| kdb_cmdhist.c | 162 | kdbg/kdb_cmdhist.rs | 310 |

The variadic `KdbPrintf` / `KdbpPrint` plus ~25 file-private statics from
the original `kdb_cli.c` live in `ntoskrnl/kdbg/kdbg_rust_shims.c` (539 LOC).
i386-dis.c (binutils, 4,453 LOC) and kdb_help.S (asm trampolines) stay in C.

### 5. KD — Low-Level Debugger Support — DONE (this session)
**~~9 files, 2,242 lines — PRIORITY: LOW~~ COMPLETE**

Serial port, PS/2 keyboard, and terminal I/O for the kernel debugger. All 9 files
translated to Rust this session. ~3,914 LOC of new Rust + 1 shim file
(`kd_rust_shims.c` for HAL `KdComPortInUse` cross-DLL bridge).

| File | C lines | Rust file | Notes |
|------|--------:|-----------|-------|
| kdio.c | 355 | kd/kdio.rs (1,525) | Owns globals KdpDebugMode/KdProviders/DispatchTable/InitRoutines |
| kdmain.c | 563 | kd/kdmain.rs (795) | Defensive `InitializeListHead(&KdProviders)` for null Rust static-init |
| kdterminal.c | 443 | kd/kdterminal.rs (488) | Owns KdbDebugState; cdecl ABI (matches C) |
| kdprompt.c | 182 | kd/kdprompt.rs (243) | |
| kdps2kbd.c | 128 | kd/kdps2kbd.rs (250) | x86 in/out via inline asm |
| kdserial.c | 256 | kd/kdserial.rs (80) | Generic wrapper |
| i386/kdserial.c | 115 | i386/kdserial.rs | Used by BOTH i386 + amd64 (cfg `any(x86, x86_64)`) |
| arm64/kdserial.c | 100 | arm64/kdserial.rs | PL011 |
| arm/kdserial.c | 100 | arm/kdserial.rs | PL011 |

### 6. CACHE — Legacy NewCC Cache Manager — DONE
**13 files, ~7,500 lines — PRIORITY: VERY LOW**

Alternative cache manager implementation, only compiled when `NEWCC=ON`. The active cache manager (CC subsystem) is already translated to Rust.

**Status as of 2026-04-27 — fully resolved:**
- `cache/section/io.c` (338 LOC) + `cache/section/sptab.c` (429 LOC) — the two unconditionally-compiled files — translated to Rust as `mm/section_io.rs` (173 LOC) + `mm/section_table.rs` (367 LOC). **Wired in `mm/mod.rs:16-17`; C versions commented out at `ntos.cmake:50-51`. Active in build, smoke green on all 3 arches.**
- The 11 NEWCC-only files (cachesub.c, copysup.c, fssup.c, lazyrite.c, logsup.c, mdlsup.c, pinsup.c, section/{data,fault,reqtools,swapout}.c) have been line-comment-out as dead code, retained as reference. To delete in bulk when ready.
- `mm_rust_shims.c` gained `MiAcquirePfnLock` / `MiReleasePfnLock` cdecl outline trampolines — these were FORCEINLINE in `mm.h` with no real symbol; the new Rust callers need linkable functions.

| File | Lines | Status |
|------|-------|--------|
| section/io.c | 338 | → mm/section_io.rs (active) |
| section/sptab.c | 429 | → mm/section_table.rs (active) |
| pinsup.c | 997 | NEWCC dead — line-commented |
| section/fault.c | 968 | NEWCC dead — line-commented |
| section/data.c | 847 | NEWCC dead — line-commented |
| section/swapout.c | 660 | NEWCC dead — line-commented |
| fssup.c | 632 | NEWCC dead — line-commented |
| cachesub.c | 363 | NEWCC dead — line-commented |
| section/reqtools.c | 283 | NEWCC dead — line-commented |
| copysup.c | 235 | NEWCC dead — line-commented |
| mdlsup.c | 127 | NEWCC dead — line-commented |
| logsup.c | 71 | NEWCC dead — line-commented |
| lazyrite.c | 36 | NEWCC dead — line-commented |

### 7. Misc Root-Level Files
**3 files, ~1,500 lines — already accounted for**

| File | Description |
|------|-------------|
| guid.c | GUID definitions (auto-generated, stays in C) |
| kernel_c_helpers.c | Misc C helpers for Rust FFI |
| tier1_rust_shims.c | Shims for tier 1 Rust subsystems |
| KrnlFun.c | Kernel function stubs |

### 8. Tests
**36 files, 5,859 lines — NOT FOR TRANSLATION**

Kernel test harness, stays in C.

## Summary (updated 2026-04-28)

| Subsystem | Files | Lines | Priority | Status |
|-----------|-------|-------|----------|--------|
| **config/** | 24 | 18,827 | HIGH | Rust translations active; compile-clean on x86_64, parity audit/fixup in progress with remaining highest-risk review in cmapi/cmboot/cmkcbncb |
| ~~**fstub/**~~ | 4 | 5,817 | DONE | All 4 files Rust on all 3 arches |
| **kd64/** | 11 | 6,846 | DEFERRED | Already translated, needs debug (recursive exception flood on amd64) |
| ~~**kdbg/**~~ | 7 + shim | 7,653 | DONE | 7 generic files Rust on i386 + amd64 (2026-04-28). i386-dis.c + kdb_help.S stay in C |
| ~~**kd/**~~ | 9 | 2,242 | DONE | All 9 files Rust on all 3 arches |
| ~~**cache/**~~ | 13 | 7,486 | DONE | section/{io,sptab}.c → Rust (wired, active); 11 NEWCC files line-commented dead code |
| **TOTAL remaining** | ~24 | ~24,673 | | (config + kd64 only) |

## Translation Priority Recommendation

1. **CONFIG bisection** — translation done but boot-broken; needs root-cause debugging (Codex / fresh eyes when tokens available). 21 files (~30K Rust) live on branch `wip-config-rust-port`.
2. **kd64/ debug** — already translated, needs recursive exception flood on amd64 debugged. No API cost.
3. **Misc root files** — guid.c, kernel_c_helpers.c, tier1_rust_shims.c, KrnlFun.c — small (~1.5K lines).

After config bisection lands, the kernel would be **~98% Rust** by line count.

## What's Still C (and intentionally so)

- **cmdata.c** (config) — pure globals, no functions. Tree-sitter can't extract anything; either keep or hand-translate as `pub static mut` block.
- **i386-dis.c** (kdbg) — binutils-derived i386 disassembler, ~4453 lines. Not our code.
- **kdb_help.S** (kdbg) — assembly trampolines, two arch variants (~30 lines each).
- **guid.c** — auto-generated GUID definitions.
- **kernel_c_helpers.c** + various `_rust_shims.c` files — outline trampolines for FORCEINLINE/macro/dllimport bridging. Stays in C by design.
- **tests/** — kernel test harness. Not for translation.
 