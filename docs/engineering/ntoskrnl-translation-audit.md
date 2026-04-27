# ntoskrnl C→Rust Translation Audit (updated 2026-04-27)

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
| KD (low-level debugger) | 9 | 3,914 | All arches (this session) |
| LPC (local proc) | 8 | 3,876 | All arches |
| INBV (boot video) | 3 | 3,611 | All arches |
| WMI | 4 | 3,588 | All arches |
| RTL (runtime) | 2 | 2,157 | All arches |
| VDM (virtual DOS) | 2 | 1,221 | All arches |
| VF (verifier) | 1 | 140 | All arches |
| MM section I/O+page-table | 2 | 533 | All arches (replaces cache/section/{io,sptab}.c) |
| **TOTAL** | **281** | **~360,500** | |

## NOT YET TRANSLATED

### 1. CONFIG — Registry/Configuration Manager (CM)
**24 files, 18,827 lines — PRIORITY: HIGH**

The Windows Registry implementation. Used on every boot and throughout runtime. Zero Rust translations exist.

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

### 2. FSTUB — File System Stubs
**4 files, 5,817 lines — PRIORITY: MEDIUM**

Disk partition support and HAL stubs. Used during boot for disk enumeration.

| File | Lines | Description |
|------|-------|-------------|
| disksup.c | 2,596 | Disk partitioning, MBR/GPT support |
| fstubex.c | 2,136 | Extended partition support |
| translate.c | 652 | ARC name translation |
| halstub.c | 433 | HAL dispatch stubs |

**Dependencies**: Disk drivers, HAL, IO manager
**Complexity**: MEDIUM — straightforward partition parsing, some arch-specific

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

### 4. KDBG — Interactive Kernel Debugger (KDB)
**9 files, 12,328 lines — PRIORITY: LOW**

ReactOS-specific interactive debugger (not WinDbg protocol). Development tool only.

| File | Lines | Description |
|------|-------|-------------|
| kdb_cli.c | 4,826 | Command-line interface |
| kdb.c | 2,718 | Main debugger logic |
| kdb_expr.c | 1,693 | Expression evaluator |
| i386/i386-dis.c | 1,173 | i386 disassembler |
| kdbg.c | 644 | KDB glue |
| kdb_symbols.c | 587 | Symbol lookup |
| kdb_print.c | 281 | Print support |
| kdb_cmdhist.c | 226 | Command history |
| i386/kdmemsup.c | 180 | Memory support |

**Complexity**: LOW-MEDIUM — mostly string parsing and display, but large
**Note**: Not critical for boot or runtime. Only active when debugging.

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

### 6. CACHE — Legacy NewCC Cache Manager — PARTIALLY HANDLED
**13 files, 5,986 lines — PRIORITY: VERY LOW**

Alternative cache manager implementation, only compiled when `NEWCC=ON`. The active cache manager (CC subsystem) is already translated to Rust.

**Status as of 2026-04-27:**
- `cache/section/io.c` + `cache/section/sptab.c` — the two unconditionally-compiled files — translated to Rust as `mm/section_io.rs` + `mm/section_table.rs` (533 LOC).
- The 11 NEWCC-only files (cachesub.c, copysup.c, fssup.c, lazyrite.c, logsup.c, mdlsup.c, pinsup.c, section/{data,fault,reqtools,swapout}.c) have been line-comment-out as dead code, retained as reference. To delete in bulk when ready.

| File | Lines | Status |
|------|-------|--------|
| section/io.c | 167 | → mm/section_io.rs |
| section/sptab.c | 94 | → mm/section_table.rs |
| copysup.c | 1,085 | NEWCC dead — line-commented |
| pinsup.c | 1,019 | NEWCC dead — line-commented |
| fssup.c | 886 | NEWCC dead — line-commented |
| section/fault.c | 750 | NEWCC dead — line-commented |
| section/data.c | 496 | NEWCC dead — line-commented |
| cachesub.c | 418 | NEWCC dead — line-commented |
| section/swapout.c | 367 | NEWCC dead — line-commented |
| mdlsup.c | 285 | NEWCC dead — line-commented |
| lazyrite.c | 198 | NEWCC dead — line-commented |
| section/reqtools.c | 155 | NEWCC dead — line-commented |
| logsup.c | 66 | NEWCC dead — line-commented |

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

## Summary (updated 2026-04-27)

| Subsystem | Files | Lines | Priority | Status |
|-----------|-------|-------|----------|--------|
| **config/** | 24 | 18,827 | HIGH | ~$0.50; first attempt translated all 22 .c files but i386 BSODs early in boot — stashed, awaiting Codex token reset for fresh-eyes debugging |
| ~~**fstub/**~~ | 4 | 5,817 | DONE | All 4 files Rust on all 3 arches |
| **kd64/** | 11 | 6,846 | DEFERRED | Already translated, needs debug (recursive exception flood on amd64) |
| **kdbg/** | 9 | 12,328 | LOW | ~$0.35; i386-dis.c (binutils) stays in C |
| ~~**kd/**~~ | 9 | 2,242 | DONE | All 9 files Rust on all 3 arches (this session) |
| **cache/** | 13 | 5,986 | PARTIALLY DONE | section/{io,sptab}.c → Rust; rest is NEWCC dead code, line-commented |
| **TOTAL remaining** | ~33 | ~37,001 | | ~$0.85 |

## Translation Priority Recommendation

1. **CONFIG bisection** — translation done but boot-broken; needs root-cause debugging (Codex / fresh eyes when tokens available). 21 files (~30K Rust) sit in `git stash@{0}`.
2. **kd64/ debug** — already translated, needs recursive exception flood on amd64 debugged. No API cost.
3. **KDBG (12K lines)** — Low priority, debugger-only code. The i386-dis.c (binutils-derived disassembler) stays in C; everything else is straightforward.
4. **Misc root files** — guid.c, kernel_c_helpers.c, tier1_rust_shims.c, KrnlFun.c — small (~1.5K lines).

After config bisection lands and kdbg follows, the kernel would be **~98% Rust** by line count.

## What's Still C (and intentionally so)

- **cmdata.c** (config) — pure globals, no functions. Tree-sitter can't extract anything; either keep or hand-translate as `pub static mut` block.
- **i386-dis.c** (kdbg) — binutils-derived i386 disassembler, ~4453 lines. Not our code.
- **kdb_help.S** (kdbg) — assembly trampolines, two arch variants (~30 lines each).
- **guid.c** — auto-generated GUID definitions.
- **kernel_c_helpers.c** + various `_rust_shims.c` files — outline trampolines for FORCEINLINE/macro/dllimport bridging. Stays in C by design.
- **tests/** — kernel test harness. Not for translation.
