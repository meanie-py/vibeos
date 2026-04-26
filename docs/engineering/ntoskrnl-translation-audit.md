# ntoskrnl C→Rust Translation Audit

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
| MM (memory) | 11 | 11,221 | All arches |
| FSRTL (fs runtime) | 17 | 20,785 | All arches |
| CC (cache) | 7 | 6,112 | All arches |
| DBGK (debug) | 2 | 5,061 | All arches |
| PO (power) | 4 | 4,961 | All arches |
| LPC (local proc) | 8 | 3,876 | All arches |
| INBV (boot video) | 3 | 3,611 | All arches |
| WMI | 4 | 3,588 | All arches |
| RTL (runtime) | 2 | 2,157 | All arches |
| VDM (virtual DOS) | 2 | 1,221 | All arches |
| VF (verifier) | 1 | 140 | All arches |
| **TOTAL** | **266** | **~344,000** | |

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

### 5. KD — Low-Level Debugger Support
**9 files, 2,242 lines — PRIORITY: LOW**

Serial port, PS/2 keyboard, and terminal I/O for the kernel debugger.

| File | Lines | Description |
|------|-------|-------------|
| kdmain.c | 563 | Main KD initialization |
| kdterminal.c | 443 | Terminal emulation |
| kdio.c | 355 | I/O operations |
| kdserial.c | 256 | Serial port (generic) |
| kdprompt.c | 182 | Debugger prompt |
| kdps2kbd.c | 128 | PS/2 keyboard |
| i386/kdserial.c | 115 | i386 serial |
| arm64/kdserial.c | 100 | ARM64 serial |
| arm/kdserial.c | 100 | ARM serial |

### 6. CACHE — Legacy NewCC Cache Manager
**13 files, 5,986 lines — PRIORITY: VERY LOW**

Alternative cache manager implementation, only compiled when `NEWCC=ON`. The active cache manager (CC subsystem) is already translated to Rust.

| File | Lines | Description |
|------|-------|-------------|
| copysup.c | 1,085 | Copy support |
| pinsup.c | 1,019 | Pin support |
| fssup.c | 886 | FS support |
| section/fault.c | 750 | Section fault handling |
| section/data.c | 496 | Section data |
| cachesub.c | 418 | Cache subsystem |
| section/swapout.c | 367 | Swap out |
| mdlsup.c | 285 | MDL support |
| lazyrite.c | 198 | Lazy write |
| section/io.c | 167 | Section I/O |
| section/reqtools.c | 155 | Request tools |
| section/sptab.c | 94 | Section page table |
| logsup.c | 66 | Log support |

**Note**: Two cache/section files (io.c, sptab.c) are compiled unconditionally. The rest are NEWCC-only.

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

## Summary

| Subsystem | Files | Lines | Priority | Effort |
|-----------|-------|-------|----------|--------|
| **config/** | 24 | 18,827 | HIGH | ~$0.50 via rustconv batch |
| **fstub/** | 4 | 5,817 | MEDIUM | ~$0.15 |
| **kd64/** | 11 | 6,846 | DEFERRED | Already translated, needs debug |
| **kdbg/** | 9 | 12,328 | LOW | ~$0.35 |
| **kd/** | 9 | 2,242 | LOW | ~$0.08 |
| **cache/** | 13 | 5,986 | VERY LOW | NEWCC-only, skip |
| **TOTAL** | 70 | 52,046 | | ~$1.10 |

## Translation Priority Recommendation

1. **CONFIG (18.8K lines)** — Highest impact. Registry is core infrastructure used by every subsystem. Would bring the kernel to ~95% Rust.
2. **FSTUB (5.8K lines)** — Medium effort, needed for disk/partition support.
3. **KD64 (6.8K lines)** — Already translated, just needs the recursive exception bug debugged. No API cost.
4. **KD + KDBG (14.6K lines)** — Low priority, debugger-only code. Can defer.
5. **CACHE (6K lines)** — Skip, NEWCC is not the active code path.

After CONFIG + FSTUB, the kernel would be **~98% Rust** by line count (344K Rust + 25K remaining C for debuggers/tests/shims).
