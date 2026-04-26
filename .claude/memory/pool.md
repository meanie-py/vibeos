# Rust Kernel Pool Allocator

## Overview
Replaced `ntoskrnl/mm/ARM3/expool.c` (3,110 lines) + `pool.c` (1,743 lines) with Rust.
Two crates: `ntkrnl-sys` (FFI declarations) + `pool` (pool allocator implementation).
36 exported functions, all 3 architectures, zero build failures.

## Crate Structure

### ntkrnl-sys (FFI declarations)
```
sdk/lib/rust/ntkrnl-sys/src/
├── lib.rs           # Re-exports
├── base.rs          # NTSTATUS, PVOID, ULONG, LIST_ENTRY, SLIST_HEADER
├── ke.rs            # Spinlocks, GuardedMutex, DPC, KPRCB, lookaside
├── mm.rs            # POOL_TYPE, POOL_HEADER, POOL_DESCRIPTOR, PFN
├── ex.rs            # Pool constants, ExRaiseStatus
├── ps.rs            # PsGetCurrentProcess (→IoGetCurrentProcess)
├── ob.rs            # ObReferenceObject (→ObfReferenceObject, fastcall on i386)
├── rtl.rs           # RtlCopyMemory (inline), RtlZeroMemory
└── interlocked.rs   # All inline atomics + SList Rtl* wrappers
```

### pool (allocator implementation)
```
sdk/lib/rust/pool/src/
├── lib.rs           # FFI exports (36 #[no_mangle] functions)
├── expool.rs        # Executive pool: ExAllocatePoolWithTag, ExFreePoolWithTag, tag tracking
├── pool.rs          # Page-level: MiAllocatePoolPages, MiFreePoolPages, init
├── types.rs         # POOL_DPC_CONTEXT, internal types
└── globals.rs       # Global state (static mut)
```

## Key FFI Calling Convention Patterns (i386)

These caused linker failures and required careful handling:

| Function | Convention | Symbol | Solution |
|----------|-----------|--------|----------|
| KeGetCurrentIrql | inline on amd64 | (reads CR8) | `core::arch::asm!("mov {}, cr8")` |
| KeGetCurrentIrql | stdcall on i386/arm64 | HAL export | `extern "system"` |
| KeAcquireGuardedMutex | fastcall all arches | `@KeAcquireGuardedMutex@4` | `extern "fastcall"` on i386, `extern "C"` on others |
| KeAcquireSpinLockForDpc | fastcall all arches | `@KeAcquireSpinLockForDpc@4` | Direct `pub` extern, no wrappers |
| KeAcquireSpinLock | macro → KfAcquireSpinLock | `@KfAcquireSpinLock@4` | Inline wrapper with cfg |
| KeAcquireQueuedSpinLock | fastcall on i386 | `@KeAcquireQueuedSpinLock@4` (HAL) | `extern "fastcall"` on i386 |
| PsGetCurrentProcess | macro → IoGetCurrentProcess | `_IoGetCurrentProcess@0` | `#[link_name]` |
| ObReferenceObject | macro → ObfReferenceObject | fastcall on i386 | `extern "fastcall"` + wrapper |
| InterlockedIncrement | compiler intrinsic | (no symbol) | Inline `AtomicI32::fetch_add` |
| RtlCopyMemory | macro → memcpy | (no symbol) | Inline `copy_nonoverlapping` |
| ExRaiseStatus | macro → RtlRaiseStatus | `_RtlRaiseStatus@4` | `#[link_name]` |

## Critical Gotcha: link_name + extern "fastcall"

**NEVER use `#[link_name = "@Name@N"]` with `extern "fastcall"`!**
Fastcall adds its own `@` prefix and `@N` suffix, resulting in double decoration like `@@Name@N@N`.
Instead, use the actual function name directly in the `extern "fastcall"` block.

## i386 SList Conflict Resolution

The Rust SList crate (in librtl) and ntoskrnl's `fastinterlck_asm.S` both defined:
- `_ExpInterlockedPopEntrySListResume@0`
- `_ExpInterlockedPopEntrySListFault@0`
- `_ExpInterlockedPopEntrySListEnd@0`

Fix: Replaced the inline SList implementations in fastinterlck_asm.S with
forwarder stubs that call the Rust SList's `Rtl*` versions. Only one set
of fault labels exists now (from the Rust SList crate).

**Push param count**: Rust SList's `RtlInterlockedPushEntrySList` takes **2 params** on i386
(no Lock), so the symbol is `_RtlInterlockedPushEntrySList@8`, not `@12`.

## CMake Integration

- `ntoskrnl/CMakeLists.txt`: Cargo build + link `libreactos_pool.a`
- `ntoskrnl/ntos.cmake`: Commented out `expool.c` and `pool.c`
- `ntkrnlmp/CMakeLists.txt`: Same linking for multiprocessor kernel

## MiDumpPoolConsumers Calling Convention

Declared as `VOID MiDumpPoolConsumers(...)` in kdbg.c (no NTAPI → cdecl).
Rust export must use `extern "C"`, NOT `extern "system"` (which is stdcall on i386).

## amd64 Rust Pool Bugs Fixed

### Bug 1: MMFREE_POOL_ENTRY Owner marking range (pool.rs)
**Symptom**: GPF in `MiAllocatePoolPages` RemoveEntryList — Flink contained pool tag bytes ("MmPt").
**Root cause**: MiFreePoolPages marking loop used `number_of_pages` (just the freed allocation) instead of `free_pages` (total merged size including coalesced next-range). Coalesced next-range pages kept stale Owner pointers into allocated pool data with POOL_HEADER.
**Fix**: Changed marking loop to use `free_pages`, matching C code (pool.c:1299-1309).

### Bug 2: MMPFN struct size mismatch (mm.rs)
**Symptom**: Page fault at IRQL 2 accessing invalid PFN database address, PFN list corruption in MmZeroPageThread → MiRemovePageByColor.
**Root cause**: Hand-written MMPFN was 48 bytes (64-bit) / 24 bytes (32-bit) — actual C struct is 72 / 36 bytes. Every `MmPfnDatabase.add(pfn)` used wrong stride (pfn*48 vs pfn*72), accessing wrong PFN entries.
**Fix**: Changed `_opaque: [u8; 48]` → `[u8; 72]` and `[u8; 24]` → `[u8; 36]`. Verified against bindgen size_check assertions.

### Bug 3: MM_PAGED_POOL_INFO field order (mm.rs, previous session)
**Symptom**: amd64 boot hang in Phase 1 MM init.
**Root cause**: Rust struct had PagedPoolCommit and PagedPoolHint swapped vs C layout.
**Fix**: Reordered fields to match C: `{ PagedPoolCommit, PagedPoolHint, AllocatedPagedPool }`.

### Bug 4: mmfault.c IRQL safety (C, defense-in-depth)
**Issue**: `MiLockWorkingSetShared` asserts `IRQL <= APC_LEVEL`, but kernel page faults can happen at DISPATCH_LEVEL.
**Fix**: Added IRQL check before VAD lookup — skips to MmArmAccessFault at elevated IRQL. Non-ARM3 memory areas don't exist at DISPATCH_LEVEL.

### Bug 5: ExpLargePoolTableLock (expool.rs, previous session)
**Symptom**: WARNING spam flooding serial output from KeAcquireSpinLockForDpc.
**Fix**: Changed to KeAcquireSpinLock/KeReleaseSpinLock (proper IRQL transition).
