# Rust Kernel Patterns — LLVM-Safe Internal APIs

## Problem

LLVM optimizes Rust code under assumptions that break kernel invariants: pointer writes are eliminated via `captures(none)`, IRQL stores are treated as dead code, function calls with "invisible" side effects are removed. These assumptions are valid for userspace but catastrophic for kernel code.

The root cause is that kernel state (IRQL, spinlocks, hardware registers) is managed through side effects invisible to the optimizer. The fix: make invisible state into visible values that flow through the program.

## Architecture: Three Layers

```
┌─────────────────────────────────────────────────────┐
│  Layer 3: Internal Kernel Code                       │
│  Uses guards, Result types, scoped state             │
│  LLVM optimizes freely — all state is visible        │
├─────────────────────────────────────────────────────┤
│  Layer 2: Rust Guard Primitives                      │
│  IrqlGuard, SpinLockGuard, PoolGuard                │
│  Drop impls use Layer 1 primitives                   │
├─────────────────────────────────────────────────────┤
│  Layer 1: Sacred Primitives (ASM / volatile)         │
│  IRQL raise/lower, spinlock acquire/release          │
│  Trap entry/exit, context switch                     │
│  LLVM cannot touch — asm! or separate C compilation  │
└─────────────────────────────────────────────────────┘
         ↕ C ABI boundary (unchanged)
┌─────────────────────────────────────────────────────┐
│  Exported Functions (KeRaiseIrql, KeAcquireSpinLock) │
│  Thin wrappers — C calling convention                │
│  win32k, drivers, user-mode consume these            │
└─────────────────────────────────────────────────────┘
```

## Layer 1: Sacred Primitives

These are the ~10 operations where hardware state changes must survive optimization. They live in assembly (.S files), `global_asm!`, or C shims with volatile/asm barriers.

### IRQL Raise/Lower

Implemented in `ntkrnl-sys/src/guards.rs` as `irql_raise`/`irql_lower`/`irql_current`.

- **amd64**: Direct `mov cr8` via inline asm — LLVM cannot optimize away real asm instructions
- **ARM64/i386**: Extern call to HAL (`KfRaiseIrql`/`KfLowerIrql`) with asm barriers before and after

### Asm Barrier Design

The ARM64/i386 pattern uses empty asm statements as fences around extern calls:

```rust
core::arch::asm!("", options(nostack));  // barrier
let old = KfRaiseIrql(new_irql);
core::arch::asm!("", options(nostack));  // barrier
```

**Why this works:** Rust's `asm!` without the `nomem` option (our case) defaults to "may read and write memory." This prevents LLVM from reordering the extern call past the barriers or proving it has no observable effects. The `nostack` option just says we don't push/pop the stack — it does NOT imply `nomem`.

**Verification:** The emitted LLVM IR should show the asm barriers as `call void asm sideeffect ""` with memory clobbers. Check with `--emit=llvm-ir` on each architecture. A post-build verification script (`tools/verify_sacred_primitives.sh`) greps `objdump -d` output to confirm `KfRaiseIrql`/`KfLowerIrql`/`KeAcquireSpinLock` calls survive in the final binary.

### Spinlock Acquire/Release

SpinLockGuard delegates to the kernel's `KeAcquireSpinLock`/`KeReleaseSpinLock` — NOT a custom atomic spin loop. See "Lessons Learned" below.

### Trap Entry/Exit (ARM64)

Already in assembly (`ntoskrnl/ke/arm64/trap.S`, `zwdispatch.S`). No change needed — this stays ASM permanently.

## Layer 2: Guard Primitives

All guards are `#[must_use]` — dropping without binding produces a compiler warning.

### Core guards (`ntkrnl-sys/src/guards.rs`)

| Guard | Acquire | Release | Use case |
|-------|---------|---------|----------|
| `IrqlGuard` | `irql_raise(level)` | `irql_lower(prev)` | Bare IRQL raise/lower |
| `SpinLockGuard` | `KeAcquireSpinLock` | `KeReleaseSpinLock` | Standard kernel spinlocks |
| `SpinLockAtDpcGuard` | `KeAcquireSpinLockAtDpcLevel` | `KeReleaseSpinLockFromDpcLevel` | Nested spinlocks at DISPATCH |
| `DispatcherLockGuard` | `KiAcquireDispatcherLock` | `KiReleaseDispatcherLock` | Scheduler dispatcher lock |
| `QueuedSpinLockGuard` | `KeAcquireQueuedSpinLock(N)` | `KeReleaseQueuedSpinLock(N)` | Queued spinlocks (e.g., IO database) |
| `CancelSpinLockGuard` | `IoAcquireCancelSpinLock` | `IoReleaseCancelSpinLock` | IRP cancel spinlock |
| `VpbSpinLockGuard` | `IoAcquireVpbSpinLock` | `IoReleaseVpbSpinLock` | Volume parameter block lock |
| `ExpansionLockGuard` | `MiAcquireExpansionLock` | `MiReleaseExpansionLock` | MM working set expansion |

### Kernel-internal guard (`ntoskrnl-rust/src/ARM3/helpers.rs`)

| Guard | Acquire | Release | Use case |
|-------|---------|---------|----------|
| `PfnLockGuard` | `MiAcquirePfnLock` | `MiReleasePfnLock` | PFN database lock |

### Guard Interop API

For bridging between guard-based Rust code and raw-KIRQL C interfaces:

```rust
// Wrap a raw KIRQL from an existing acquire into a guard
let guard = PfnLockGuard::from_raw(old_irql);        // caller already holds lock
let guard = QueuedSpinLockGuard::from_raw(17, irql);  // queued spinlock variant

// Extract raw IRQL without releasing (for scheduler patterns)
let irql = guard.previous_irql();  // borrow — guard still owns the lock

// Consume guard WITHOUT releasing (for split-release patterns)
let irql = guard.into_raw();  // guard is consumed, lock NOT released
// caller must manually release: KiReleaseDispatcherLock(irql)

// mem::forget is also used for split-release, equivalent to into_raw()
let irql = guard.previous_irql();
core::mem::forget(guard);  // prevent Drop from releasing
```

### SpinLockGuard: Why Not AtomicUsize

SpinLockGuard delegates to `KeAcquireSpinLock`/`KeReleaseSpinLock`, not a custom atomic spin loop. An earlier version used `AtomicUsize::compare_exchange_weak` and caused **deadlocks on i386** because:

1. The kernel's spinlock protocol involves queued spinlocks, not simple test-and-set
2. C code and Rust code racing on the same `KSPIN_LOCK` with different atomic implementations is a data race
3. Debugging hooks (`KD_SPIN_LOCK_DEBUG`) in the C acquire path were bypassed

Lesson: never reimplement kernel lock primitives — call the kernel's own functions.

## Layer 3: Internal Patterns

### Pattern: Output Pointers → Return Values

**Before (C-style, captures(none) vulnerable):**
```rust
#[no_mangle]
pub unsafe extern "system" fn PspCreateThread(
    ThreadHandle: *mut HANDLE,  // output pointer — LLVM can eliminate write
    ...
) -> NTSTATUS {
    // ...
    *ThreadHandle = handle;  // LLVM may prove this is dead
    STATUS_SUCCESS
}
```

**After (Rust-native, optimizer-safe):**
```rust
// Internal implementation — returns Result
unsafe fn psp_create_thread_inner(...) -> Result<HANDLE, NTSTATUS> {
    // ...
    Ok(handle)  // return value — LLVM CANNOT eliminate
}

// C-facing export — thin wrapper
#[no_mangle]
pub unsafe extern "system" fn PspCreateThread(
    ThreadHandle: *mut HANDLE,
    ...
) -> NTSTATUS {
    match psp_create_thread_inner(...) {
        Ok(h) => {
            core::ptr::write_volatile(ThreadHandle, h);
            STATUS_SUCCESS
        }
        Err(s) => s,
    }
}
```

### Pattern: IRQL Scoping

**Before:**
```rust
let mut old_irql: KIRQL = 0;
KeRaiseIrql(DISPATCH_LEVEL, &mut old_irql);
// ... protected code ...
if error {
    KeLowerIrql(old_irql);  // easy to forget on error paths!
    return STATUS_UNSUCCESSFUL;
}
// ... more code ...
KeLowerIrql(old_irql);
```

**After:**
```rust
{
    let _irql = unsafe { IrqlGuard::raise_to_dpc() };
    // ... protected code ...
    if error {
        return STATUS_UNSUCCESSFUL;  // guard drops automatically!
    }
    // ... more code ...
}  // guard drops, IRQL restored
```

### Pattern: Spinlock Scoping

**Before:**
```rust
let mut old_irql: KIRQL = 0;
KeAcquireSpinLock(&mut lock, &mut old_irql);
// ... critical section ...
KeReleaseSpinLock(&mut lock, old_irql);
```

**After:**
```rust
{
    let _lock = unsafe { SpinLockGuard::acquire(&mut lock) };
    // ... critical section ...
}  // lock released, IRQL restored
```

## Migration Strategy

### Phase 1: Build the Primitives — DONE
Sacred IRQL primitives (`irql_raise`/`irql_lower`/`irql_current`) in `ntkrnl-sys/src/guards.rs` with asm barriers. Guard types: `IrqlGuard`, `SpinLockGuard`, `SpinLockAtDpcGuard`, `PfnLockGuard`. Verified on all 3 architectures with smoke tests.

### Phase 2: Automated Migration of Simple Patterns — DONE (~148 sites)
Tools: `migrate_guards.py` (regex scanner), `migrate_guards_ts.py` (tree-sitter transform). Subagent-driven parallel migration for remaining simple patterns. Handles single, early_return, sequential, and loop-reacquire patterns.

**Pattern examples migrated:**
```rust
// Simple: acquire → ... → release
OldIrql = MiAcquirePfnLock();          →  let pfn_guard = PfnLockGuard::acquire(); // [guard]
MiReleasePfnLock(OldIrql);             →  drop(pfn_guard); // [guard-drop]

// Early return: acquire → if { release; return } → release
KeAcquireSpinLock(&mut Lock, &mut OldIrql);  →  let guard = SpinLockGuard::acquire(&mut Lock);
if err { KeReleaseSpinLock(&mut Lock, OldIrql); return; }  →  if err { drop(guard); return; }
KeReleaseSpinLock(&mut Lock, OldIrql);       →  drop(guard);

// Loop reacquire: acquire → while { release; delay; reacquire } → release
let mut pfn_guard = PfnLockGuard::acquire();
while cond { drop(pfn_guard); delay(); pfn_guard = PfnLockGuard::acquire(); }
drop(pfn_guard);
```

### Phase 3: Cross-Function Patterns — DONE

All tractable cross-function patterns have been migrated using three techniques:

**Tier 1 — Guard-passing (callee always releases once):**
- `MiCheckControlArea` → accepts `PfnLockGuard` by value (5 callers)

**Tier 2 — Inner+wrapper (exported functions with C ABI):**
- `MmDereferenceSegmentWithLock` → `Inner(Segment, Option<PfnLockGuard>)`
- `MiCompleteProtoPteFault` / `MiResolveProtoPteFault` → `Inner(..., PfnLockGuard)`
- `CcRosDeleteFileCache` → `Inner(..., QueuedSpinLockGuard) -> QueuedSpinLockGuard`
- Guard interop: `from_raw(irql)` / `into_raw()` / `previous_irql()` for bridging guards ↔ raw KIRQL

**Tier 3 — Consume+return (release-work-reacquire):**
- `MiMakeSystemAddressValidPfn` → `Inner(PVOID, PfnLockGuard) -> (ULONG, PfnLockGuard)`
- `KiTimerListExpire` → `Inner(PLIST_ENTRY, DispatcherLockGuard)` with `previous_irql()` + `mem::forget()` for the split-release DPC branch

### Phase 4: Complex Control Flow — PARTIALLY DONE
- `cc/view.rs` — 5 complex functions migrated
- Still remaining:
  - `ex/timer.rs` (38 calls) — timer cancellation/insertion with multiple lock types
  - `pnpmgr/devaction.rs` — remaining complex patterns after simple ones migrated

### Phase 5: Output Pointers → Result — NOT STARTED
Priority targets for `captures(none)` protection:
1. **ps/thread.rs** — PspCreateThread output pointers → Result
2. **ke/procobj.rs** — KeAttachProcess/KeDetachProcess IRQL management

### Phase 6: Remove Shims — NOT STARTED
Depends on Phase 4-5 completion per subsystem.

## Current State (as of 2026-03-22)

~260+ guard instances across the kernel. 9 guard types:
- `IrqlGuard`, `SpinLockGuard`, `SpinLockAtDpcGuard`, `PfnLockGuard`
- `DispatcherLockGuard`, `QueuedSpinLockGuard`, `CancelSpinLockGuard`, `VpbSpinLockGuard`, `ExpansionLockGuard`

All simple same-function patterns, cross-function guard-passing, inner+wrapper, and consume+return patterns are migrated. What remains is the scheduler's split-release pattern (see below) and some high-count complex files.

## Scheduler Split-Release — The Hard Problem

~24 call sites in the KE subsystem use a pattern that fundamentally resists single-guard RAII:

### The Pattern

```rust
// 1. Acquire APC lock — saves IRQL into lock handle struct
KiAcquireApcLockRaiseToSynch(Thread, &mut ApcLock);  // ApcLock.OldIrql = previous IRQL

// 2. Acquire dispatcher lock at already-elevated IRQL (no IRQL change)
KiAcquireDispatcherLockAtSynchLevel();

// 3. Do dispatcher work (signal objects, wake threads, etc.)
KiUnwaitThread(Thread, STATUS_SUCCESS, INCREMENT);

// 4. Release spinlocks — but STAY at elevated IRQL
KiReleaseDispatcherLockFromSynchLevel();    // release dispatcher spinlock
KiReleaseApcLockFromSynchLevel(&mut ApcLock);  // release APC spinlock

// 5. Lower IRQL + check for preemption — SEPARATELY from spinlock release
KiExitDispatcher(ApcLock.OldIrql);  // may context switch!
```

### Why a Single Guard Can't Work

A guard's `Drop` combines "release lock" and "restore IRQL" into one atomic operation. But the scheduler **intentionally separates these**:

1. **Spinlocks are released first** (step 4) — other CPUs can now acquire the dispatcher lock and modify scheduler state.

2. **IRQL stays elevated** — this prevents interrupts on the current CPU from preempting us between "spinlock released" and "preemption check done."

3. **`KiExitDispatcher` then lowers IRQL** (step 5) — but first it checks `Prcb.NextThread` to see if a higher-priority thread became ready while we held the lock. If so, it does an immediate context switch rather than waiting for the next timer interrupt.

If a guard dropped IRQL at the same time as releasing the spinlock, an interrupt could fire between the release and the preemption check. That interrupt's DPC could ready a high-priority thread, but we'd miss the preemption because we already lowered IRQL and returned to the caller.

### IRQL Crosses the Context Switch Boundary

In 6 of these sites, the raw IRQL value is stored into the thread struct before a context switch:

```rust
*kt_WaitIrql(Thread) = OldIrql;       // save IRQL into thread for later
PendingApc = KiSwapContext(OldIrql, Thread);  // context switch (asm)
// ... we return here later, when rescheduled, possibly on a different CPU ...
KeLowerIrql(OldIrql);
```

The IRQL value is consumed by the context switch assembly — it's saved into the thread's state and restored when the thread runs again, potentially much later and on a different processor. A guard's lifetime is meaningless across a context switch because the thread's execution is suspended and resumed in an entirely different context.

### What Would It Take to Guard This?

A hypothetical `DispatcherSession` API would need to model the full lifecycle:

```rust
// Hypothetical — NOT implemented
let session = DispatcherSession::begin(Thread);  // acquire APC + dispatcher locks
session.unwait_thread(Thread, STATUS_SUCCESS);   // work under both locks
session.exit();  // release locks, lower IRQL, check preemption, maybe context switch
```

This would be a significant scheduler API redesign — every KE function that signals an object (`KeSetEvent`, `KeReleaseSemaphore`, `KeReleaseMutant`, `KeInsertQueue`, etc.) would need to use it. The benefit is real (the split-release dance is error-prone and the raw IRQL threading is invisible state), but the scope is large and the scheduler is the most risk-sensitive part of the kernel.

### Files Affected

| File | Sites | Pattern |
|------|-------|---------|
| ke/thrdobj.rs | 7 | ApcLock/LockHandle → KiExitDispatcher |
| ke/procobj.rs | 4 | ProcessLock/ApcLock → KiExitDispatcher |
| ke/eventobj.rs | 2 | WaitIrql store + KiExitDispatcher |
| ke/queue.rs | 3 | WaitIrql + KiExitDispatcher |
| ke/gate.rs | 1 | ApcLock → KiExitDispatcher |
| ke/timerobj.rs | 1 | OldIrql → KiExitDispatcher |
| ke/apc.rs | 1 | ApcLock → KiExitDispatcher |
| ke/mutex.rs | 1 | WaitIrql store + KiExitDispatcher |
| ke/semphobj.rs | 1 | WaitIrql store + KiExitDispatcher |
| ke/wait.rs | 1 | WaitIrql + KiSwapContext + KeLowerIrql |
| ke/thrdschd.rs | 2 | WaitIrql + KiSwapContext |

## Relationship to Other Docs

- `shim-elimination.md` — Guard migration enables shim removal (Category 1+4)
- `arm64-debug.md` — Bug 1 would be prevented by Result pattern; Bug 2 is fixed by ASM dispatch
- `AGENTS.md` — Translation rules updated to use guards for new translations
- `~gemini-guard-migration-prompt.md` — Prompt template for LLM-assisted guard migration

## Key Principle

> Stop encoding kernel invariants as invisible side effects.
> Make them visible values that the optimizer must preserve.
>
> IRQL → IrqlGuard (scoped lifetime)
> Output pointers → Result (return value)
> Spinlocks → SpinLockGuard (RAII)
> Trap state → Assembly (non-optimizable)
