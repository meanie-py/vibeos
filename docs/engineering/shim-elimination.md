# Shim Elimination Plan

## Overview

There are ~340 C shim functions across 10 `*_rust_shims.c` files in the kernel. These exist because Rust can't directly call C FORCEINLINE functions, macros, or certain ABI-incompatible functions. The north star is pure Rust — every shim is a C dependency we want to eliminate.

## Current Shim Files

| File | ~Functions | Primary purpose |
|------|-----------|-----------------|
| `ntoskrnl/ke/ke_rust_shims.c` | 97 | FORCEINLINE functions from ke_x.h |
| `ntoskrnl/io/iomgr/io_rust_shims.c` | 62 | IO macros, Ob* shims, devaction shims, RTL shims |
| `ntoskrnl/ps/ps_rust_shims.c` | 48 | PS FORCEINLINE functions |
| `ntoskrnl/mm/mm_rust_shims.c` | 42 | MM FORCEINLINE functions |
| `ntoskrnl/kd64/kd64_rust_shims.c` | 26 | KD64 FORCEINLINE functions |
| `ntoskrnl/ex/ex_rust_shims.c` | 18 | EX FORCEINLINE/macro functions |
| `ntoskrnl/tier1_rust_shims.c` | 18 | Tier 1 subsystem macros (LPC, HAL dispatch, etc.) |
| `ntoskrnl/ob/ob_rust_shims.c` | 13 | OB FORCEINLINE functions |
| `ntoskrnl/se/se_rust_shims.c` | 10 | SE FORCEINLINE functions |
| `ntoskrnl/mm/mm_sysldr_shims.c` | 6 | System loader helpers |

## Shim Categories

### Category 1: FORCEINLINE Wrappers (~150 shims)

**What:** Functions defined as `FORCEINLINE` (or `static __inline`) in C headers like `ke_x.h`, `io_x.h`, `ob_x.h`, `ps_x.h`. They have no exported linker symbol — the C compiler inlines them at each call site. Rust can't include C headers, so it calls through non-inline C wrapper functions.

**Examples:**
- `KiAcquireDispatcherLock`, `KiReleaseDispatcherLock` (dispatcher locking)
- `KxUnwaitThread`, `KxQueueReadyThread` (scheduler internals)
- `KiComputeTimerTableIndex`, `KxInsertTimer` (timer management)
- `IopLockFileObject`, `IopUnlockFileObject` (IO file object locking)
- `IopUpdateOperationCount`, `IopUpdateTransferCount` (IO statistics)

**Shim pattern:** The C shim file uses a "Phase 1 rename" trick:
```c
// Before including headers: rename the FORCEINLINE so it becomes _saved_Name
#define KiAcquireDispatcherLock _saved_KiAcquireDispatcherLock
#include <ntoskrnl.h>
#undef KiAcquireDispatcherLock

// After headers: define a real non-inline function that calls the saved version
KIRQL KiAcquireDispatcherLock(VOID) {
    return _saved_KiAcquireDispatcherLock();
}
```

**Elimination path:** Translate the inline function bodies from C to Rust. These are self-contained — they only call other kernel functions and do register/memory manipulation. A tool could:
1. Extract FORCEINLINE function bodies from C headers (tree-sitter parse `ke_x.h` etc.)
2. Translate to Rust (LLM or manual)
3. Add as `#[inline(always)]` Rust functions in a shared module

**Effort:** Large but mechanical. ~150 functions averaging ~10-20 lines each. Good candidate for LLM batch translation via rustconv or Codex. The ke_x.h functions are the most complex (scheduler/dispatcher internals with spinlock manipulation).

**Risk:** Some FORCEINLINE functions use compiler intrinsics (`_InterlockedCompareExchange`, `__readcr3`, `_BitScanForward`) that would need `core::arch::asm!` equivalents.

---

### Category 2: C Macro Wrappers (~40 shims)

**What:** C preprocessor macros that expand to expressions or statements. No function exists at all — the preprocessor substitutes the body inline. Rust can't use the C preprocessor.

**Examples:**
- `InitializeObjectAttributes` — struct initialization macro
- `CONTAINING_RECORD` — pointer arithmetic macro for linked lists
- `InsertTailList`, `RemoveHeadList`, `IsListEmpty`, `InitializeListHead` — linked list macros
- `ObReferenceObject` → expands to `ObfReferenceObject(Object)` (fastcall wrapper)
- `ObDereferenceObject` → expands to `ObfDereferenceObject(Object)`
- `ExRaiseStatus` → expands to `RtlRaiseStatus(Status)`
- `HalInitPnpDriver` → expands to `HALDISPATCH->HalInitPnpDriver()`
- `ExInitializeWorkItem` — struct initialization macro
- `IoGetCurrentIrpStackLocation` — field access macro

**Detection:** `verify_ffi.py` flags these as `[C_MACRO]` — currently finds 62 instances across the codebase.

**Elimination path:** Inline as Rust helper functions. Most are trivial:
```rust
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING,
    a: ULONG, r: HANDLE, s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}
```

**Status:** Already being done file-by-file. `iomgr.rs` and `arcname.rs` already have inline `InitializeObjectAttributes`. Several files still declare these as `extern "C" fn` — verify_ffi.py tracks them.

**Effort:** Small. Most macros are 1-5 lines. Can be done incrementally as each file is touched.

---

### Category 3: LLVM `captures(none)` Workarounds (~15 shims)

**What:** Functions that write through output pointers (`OUT PVOID *Object`). When called cross-CGU from Rust, LLVM propagates `captures(none)` to the declaration and then incorrectly concludes the output pointer is never written — eliminating the entire success path as dead code.

**Root cause confirmed in LLVM IR:**
```llvm
; Cross-CGU declaration gets captures(none) on output pointer:
declare x86_stdcallcc i32 @ObCreateObject(
    ..., ptr noundef captures(none))  ; ← LLVM thinks this ptr isn't written

; Caller sees captures(none) → concludes write is dead → eliminates success path → ud2
```

**Examples:**
- `ObCreateObject_shim` / `ObCreateObjectAddress_shim` — object creation
- `ObInsertObject_shim` — object handle insertion
- `ObReferenceObjectByHandle_shim` — handle-to-object lookup
- `IopGetRegistryValue_shim` — registry value read
- `IopOpenRegistryKeyEx_shim` — registry key open
- `IopCreateRegistryKeyEx_shim` — registry key create
- `IopLoadDriver_shim` — driver loading
- `IopInitializePlugPlayServices_shim` — PnP init
- `RtlAnsiStringToUnicodeString_shim` — string conversion
- `RtlUnicodeStringToAnsiString_shim` — string conversion
- `RtlInitAnsiString_shim` — string init
- `RtlFreeUnicodeString_shim` — string free

**Elimination path: `codegen-units = 1` in Cargo.toml.**

This is a **one-line fix** that completely eliminates the issue:
```toml
[profile.release]
codegen-units = 1   # was 16
```

**Why it works:** With CGU=1, all Rust code is in a single LLVM module. `ObCreateObject` is a *definition* (not a cross-CGU declaration), so LLVM sees the full function body and knows the output pointer IS written. No `captures(none)` on cross-CGU declarations because there are no cross-CGU declarations.

**Verified:** LLVM IR with CGU=1 shows `ObCreateObject` as `define` (0 declares) vs CGU=16 which shows it as `declare ... captures(none)`.

**Build time impact:** 6s → 10s wall time (CGU=16 parallelizes across cores; CGU=1 is sequential but does less total work). Negligible for a kernel build.

**Additional benefit:** Would also eliminate the `#[inline(never)]` helper splits that Codex added to device.rs (`IopFinishCreatedDeviceObject`) and iomgr.rs (`IopFinishIoInitSystem`) to work around the same optimizer issue. Those structural workarounds could be reverted, simplifying the code.

**May also fix ARM64 baseline:** The ARM64 BUGCHECK 0x6B regression could be the same class of bug manifesting differently. If CGU=1 fixes ARM64 too, that's a major bonus. (Codex is currently investigating ARM64 and found it's "codegen-sensitive" — consistent with this theory.)

---

### Category 4: Calling Convention Bridges (~19 shims)

**What:** Functions where the real kernel export uses `fastcall` (first two params in ECX/EDX registers on i386) but Rust declares them as `extern "C"` (all params on stack). The shim converts between calling conventions.

**The 7 unique fastcall functions (19 total instances):**

| Function | Instances | What it does |
|----------|-----------|-------------|
| `InterlockedCompareExchange` | 8 | Atomic compare-and-swap |
| `InterlockedExchange` | 3 | Atomic exchange |
| `KiAcquireSpinLock` | 2 | Spinlock acquire (KE internal) |
| `KiReleaseSpinLock` | 2 | Spinlock release (KE internal) |
| `KiEoiHelper` | 2 | End-of-interrupt (KE internal) |
| `ObfDereferenceObject` | 1 | Object dereference (the real function behind ObDereferenceObject macro) |
| `ExfInterlockedInsertTailList` | 1 | Interlocked list insert |

**Elimination path: `extern "fastcall"` works on stable Rust for i686-pc-windows-gnu!**

Verified:
```rust
extern "fastcall" { fn TestFastcall(x: i32, y: i32) -> i32; }
// Generates correct symbol: @TestFastcall@8
```

The fix is arch-gated extern blocks:
```rust
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(Target: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
    fn ObfDereferenceObject(Object: PVOID) -> LONG;
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedCompareExchange(Target: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
    // On amd64/arm64, these are stdcall/cdecl (same thing), not fastcall
}
```

**Note:** `KeAcquireSpinLock` / `KeReleaseSpinLock` are actually C macros wrapping `KeAcquireSpinLockRaiseToDpc` (stdcall) — they fall into Category 2, not Category 4. Only the `Ki*` internal spinlock functions are truly fastcall.

**Effort:** Small. 7 unique functions, mechanical cfg-gated declarations.

---

### Category 5: Complex Delegations (~10 shims)

**What:** Functions where the C shim does significant work that's hard to replicate in Rust, or where the delegation pattern is intentional.

**Examples:**
- `PiQueueDeviceAction_c` — spinlock + list insertion + work queue dispatch (devaction.rs)
- `IoRequestDeviceEject_c` — complex eject logic with many internal calls (devaction.rs)
- `PiPerformSyncDeviceAction_c` — event wait wrapper (devaction.rs)
- `PiSetDevNodeText_c` — registry string operations (devaction.rs)

**Elimination path:** Case-by-case. Some can be translated to Rust (especially with CGU=1 fixing the output-pointer issue). Others may stay as C for pragmatic reasons (complex callback patterns, HAL dispatch table interaction).

**Effort:** Medium. Requires careful per-function analysis.

---

## Priority Order for Elimination

| Priority | Category | Shims | Fix | Effort |
|----------|----------|-------|-----|--------|
| **1** | captures(none) | ~15 | `codegen-units = 1` | One line in Cargo.toml |
| **2** | Calling convention | ~19 | `extern "fastcall"` | Mechanical, per-file |
| **3** | C macros | ~40 | Inline Rust helpers | Incremental, per-file |
| **4** | FORCEINLINE | ~150 | Translate C → Rust | LLM batch (rustconv/Codex) |
| **5** | Complex delegations | ~10 | Case-by-case | Manual analysis |

## Verification Tools

| Tool | What it checks | Location |
|------|---------------|----------|
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `verify_abi.py` | extern "C" vs extern "system" ABI | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | Combined: ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |

Run all checks:
```bash
cd reactos
.venv/bin/python tools/rustconv/rustconv/verify_constants.py   # 0 mismatches (all fixed)
.venv/bin/python tools/rustconv/rustconv/verify_abi.py          # 61 ABI mismatches
.venv/bin/python tools/rustconv/rustconv/verify_ffi.py          # 152 total issues
```

## Reference Files

| File | Description |
|------|-------------|
| `/home/curi/Projects/os-playground/reactos/CLAUDE.md` | Project instructions and architecture |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/MEMORY.md` | Memory index |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/feedback_extern_c_abi.md` | extern "C" vs "system" learnings |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/feedback_llvm_workarounds.md` | LLVM i386 workaround patterns |
| `/home/curi/.claude/projects/-home-curi-Projects-os-playground/memory/feedback_flexible_array.md` | Flexible array bug pattern |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/Cargo.toml` | Workspace config (codegen-units setting) |
| `/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src/` | Rust kernel source |
| `/home/curi/Projects/os-playground/reactos/ntoskrnl/ke/ke_rust_shims.c` | Largest shim file (97 functions) |
| `/home/curi/Projects/os-playground/reactos/sdk/include/ndk/ketypes.h` | FORCEINLINE function definitions |
| `/home/curi/Projects/os-playground/reactos/ntoskrnl/include/internal/ke_x.h` | KE FORCEINLINE functions |
