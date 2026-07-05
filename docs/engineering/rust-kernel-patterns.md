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

### Phase 4: Complex Control Flow — DONE (what's tractable)
- `cc/view.rs` — 5 complex functions migrated (FlushDirtyPages, TrimCache, etc.)
- `mm/section.rs` — grab_segment + grab_image_section_object complex loops migrated
- `ARM3/session.rs` — expansion lock loops + release-wait-reacquire patterns migrated
- `ARM3/mminit.rs` — PfnLock in init functions migrated
- Many small files across ARM3/, ke/, iomgr/, ps/, po/ migrated

Remaining calls are in the intractable categories below — conditional acquires, cancel IRQL escapes, and scheduler patterns. These represent the migration ceiling without API redesign.

### Phase 5: `opaque_ptr` Barriers — IN PROGRESS
Automated tool to apply `opaque_ptr` to all `#[no_mangle]` pointer parameters. This unblocks:
- Re-enabling write_volatile sweep (Phase 5a)
- Re-enabling RAII guards (Phase 5b)
- Re-enabling ABI unification (Phase 5c)
- Lowering CGU count (Phase 5d)
- Enabling iofunc.rs, arcname.rs holdouts (Phase 5e)

### Phase 6: Output Pointers → Result — BLOCKED on Phase 5
Priority targets for `captures(none)` protection:
1. **ps/thread.rs** — PspCreateThread output pointers → Result
2. **ke/procobj.rs** — KeAttachProcess/KeDetachProcess IRQL management

### Phase 7: Remove Shims — NOT STARTED
Depends on Phase 6 completion per subsystem.

## Current State (as of 2026-03-28)

**Guard infrastructure reverted.** The write_volatile sweep, RAII guards, Result patterns, and ABI unification were all reverted because they destabilized CGU partitioning and caused boot-blocking ud2s. Zero translations lost — only the defensive hardening layer was removed. The guard types and migration tooling still exist in git history and can be re-applied after the `opaque_ptr` tool is built.

**Current defense:** CGU=256 (near-function-isolation, prevents interprocedural cascade). i386 + amd64 6/6 smoke tests pass, 0 unexpected ud2.

**Next step:** Build automated `opaque_ptr` tool → apply to all `#[no_mangle]` functions → re-enable guards/write_volatile/ABI fixes → lower CGU count.

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

## Other Intractable Patterns

Beyond the scheduler split-release, three more categories resist guard migration. Each has a different root cause.

### Conditional Acquire (~40 sites)

```rust
// Pattern: lock acquired inside a condition, released unconditionally
if OldIrql == MM_NOIRQL {
    OldIrql = MiAcquirePfnLock();
    HaveLock = true;
}
// ... work (may or may not hold lock) ...
if HaveLock {
    MiReleasePfnLock(OldIrql);
}
```

**Why it resists:** A guard must be created unconditionally (Rust ownership is compile-time, not runtime). You can use `Option<PfnLockGuard>` — acquire returns `Some(guard)`, release does `if let Some(g) = guard.take() { drop(g) }` — but this adds runtime overhead (Option discriminant + branch on every drop) and obscures the lock state in code review.

**Files affected:**
- `ARM3/pagfault.rs` (15 calls) — `MM_NOIRQL` sentinel throughout `MmArmAccessFault`
- `ARM3/mdlsup.rs` (9 calls) — `UsePfnLock` flag in `MmProbeAndLockPages`
- `cc/view.rs` (conditional `lock_views`) — `CcRosUnmarkDirtyVacb`
- `cc/copy.rs` (conditional `RetryMasterLocked`) — `CcCanIWrite`
- `mm/section.rs` (conditional `PageOut`) — `MmpPageOutPhysicalAddress`

**Possible approach:** `Option<Guard>` pattern, but only worth it for hot paths where LLVM visibility matters. For cold paths the manual pattern is fine.

### Cancel IRQL Escape (~30 sites)

```rust
IoAcquireCancelSpinLock(&mut OldIrql);
(*Irp).CancelIrql = OldIrql;  // IRQL value escapes into IRP struct
// ... set cancel routine ...
// Cancel routine will later call IoReleaseCancelSpinLock(Irp->CancelIrql)
```

**Why it resists:** The raw IRQL value is stored into the IRP's `CancelIrql` field and consumed later by the cancel routine — potentially in a different function, different thread, or different driver entirely. The lock acquire and release are fundamentally decoupled across an asynchronous boundary. A guard can't span this because the "release" happens in a callback that the guard has no knowledge of.

**Files affected:**
- `fsrtl/oplock.rs` (29 calls) — all oplock IRP handling
- `iomgr/irp.rs` — `IoCancelIrp`
- `iomgr/device.rs` — `IoStartPacket`
- `fsrtl/notify.rs` — notify change IRP handling

**Possible approach:** Not guardable without redesigning the IRP cancel protocol. The cancel spinlock is a Windows API contract — drivers expect to call `IoReleaseCancelSpinLock(Irp->CancelIrql)` in their cancel routines.

### Scheduler Timer/DPC Core (~15 sites in ke/dpc.rs)

```rust
// KiTimerExpiration: complex multi-phase lock dance
OldIrql = KeRaiseIrqlToSynchLevel();
KiAcquireDispatcherLockAtSynchLevel();
// ... expire timers, call DPC routines ...
KiReleaseDispatcherLock(DISPATCH_LEVEL);  // release to DPC, not to OldIrql!
// ... process DPC queue at DISPATCH_LEVEL ...
KeLowerIrql(OldIrql);  // finally lower to original
```

**Why it resists:** The IRQL is lowered in two stages — first from SYNCH to DISPATCH (to process DPCs), then from DISPATCH to the original level. A single guard can only restore to one level. The intermediate DISPATCH_LEVEL plateau is intentional: DPC processing must happen above APC_LEVEL but below SYNCH_LEVEL.

**Also:** `KeRaiseIrqlToSynchLevel()` is a different primitive than `KeRaiseIrql(SYNCH_LEVEL, &mut OldIrql)` — it's a HAL function with its own calling convention (returns IRQL directly). Would need a `SynchLevelGuard` or similar.

### IRQL Stored in Globals (~10 sites)

```rust
// Pattern: IRQL saved in a static/global, not a local
static mut InbvOldIrql: KIRQL = 0;
KeRaiseIrql(HIGH_LEVEL, &mut InbvOldIrql);
// ... much later, possibly different function ...
KeLowerIrql(InbvOldIrql);
```

**Files affected:** `inbv/inbv.rs`, `ke/freeze.rs`, `kd64/kdprint.rs`

**Why it resists:** The acquire and release are in different functions, with the IRQL stored in a global. A guard would need to be stored in the same global, which means `static mut Option<IrqlGuard>` — possible but adds complexity and doesn't prevent the global from being accessed incorrectly.

## Summary: Migration Ceiling

| Category | Sites | Guardable? | Effort to fix |
|---|---|---|---|
| Scheduler split-release | ~24 | No | DispatcherSession redesign |
| WaitIrql + context switch | ~6 | No | Semantically impossible |
| Conditional acquire | ~40 | Partially (`Option<Guard>`) | Medium per-site |
| Cancel IRQL escape | ~30 | No | Windows API contract |
| Timer/DPC multi-phase | ~15 | No | Multi-level guard design |
| IRQL in globals | ~10 | Partially (`static Option<Guard>`) | Medium per-site |
| **Total at ceiling** | **~125** | | |

Against ~300+ migrated guard instances, this means we've converted **~70%** of all lock sites. The remaining 30% are genuinely different patterns that would each need targeted design work, not mechanical transformation.

## The Three LLVM Defenses

LLVM can break kernel code in three distinct ways. Each requires a different defense:

### 1. Guards — Make state transitions visible

LLVM eliminates function calls it proves have no effect. Guards encode lock/IRQL state as live Rust values with `Drop` — LLVM cannot eliminate a value that's used (dropped) later.

**Protects against:** `captures(none)` eliminating output pointer writes, dead store elimination of IRQL changes, spinlock acquire/release pairs being optimized away.

**Implemented:** 9 guard types, ~300+ instances, ~70% of lock sites.

### 2. Asm barriers — Make function calls visible

LLVM can prove extern function calls are dead if it can see (or infer) they have no side effects. Empty `asm!("", options(nostack))` statements without `nomem` act as memory barriers — LLVM must assume arbitrary memory was read/written, so it can't prove the adjacent extern call is dead.

**Protects against:** LLVM eliminating calls to `KfRaiseIrql`, `KfLowerIrql`, `KeAcquireSpinLock` when it infers they're `readnone`/`readonly`.

**Implemented:** In `guards.rs` Layer 1 sacred primitives (ARM64 + i386 paths). Verified by `test_sacred_primitives` smoke test.

### 3. `Volatile<T>` — Make data stores visible

LLVM can eliminate stores to `static mut` variables it proves are never read, or hoist/sink loads across code that doesn't touch the variable. `Volatile<T>` wraps a value in `UnsafeCell<T>` and forces all access through `read_volatile`/`write_volatile`, which LLVM must preserve.

```rust
#[repr(transparent)]
pub struct Volatile<T>(UnsafeCell<T>);

impl<T: Copy> Volatile<T> {
    pub const fn new(val: T) -> Self { Self(UnsafeCell::new(val)) }
    pub fn read(&self) -> T { unsafe { core::ptr::read_volatile(self.0.get()) } }
    pub fn write(&self, val: T) { unsafe { core::ptr::write_volatile(self.0.get(), val) } }
}
```

**Protects against:** LLVM eliminating stores to global IRQL state, reordering loads of hardware-modified memory, dead store elimination of state that's read asynchronously (by interrupt handlers, other CPUs, etc.).

**Status:** `Volatile<T>` type implemented in `ntkrnl-sys/src/volatile.rs`. However, it CANNOT be used on `#[no_mangle] pub static` globals — Rust places `static` (vs `static mut`) in read-only sections regardless of `UnsafeCell`. See "Lessons Learned" below.

For `#[no_mangle]` globals, use `read_volatile`/`write_volatile` directly on the `static mut`:
```rust
static mut InbvOldIrql: KIRQL = 0;
// Write: core::ptr::write_volatile(&raw mut InbvOldIrql, old_irql);
// Read:  core::ptr::read_volatile(&raw const InbvOldIrql);
```

Applicable sites:
- `inbv/inbv.rs` — `InbvOldIrql` global
- `ke/freeze.rs` — `KiOldIrql` global
- `ke/profobj.rs` — `KiProfileIrql` global
- `kd64/kdprint.rs` — KD lock state

**Relationship to guards:** Complementary, not alternative. Guards protect *state transitions* (acquire→release). `Volatile<T>` protects *data at rest* (stores that must survive). Asm barriers protect *function calls* (extern calls that must not be eliminated). A fully defended kernel function might use all three:

```rust
fn example() {
    let guard = SpinLockGuard::acquire(lock);  // guard: state transition visible
    GLOBAL_STATE.write(new_value);              // volatile: store visible
    extern_hal_function();                      // asm barrier: call visible (inside guard impl)
    drop(guard);                                // guard: release visible
}
```

## Automated Verification Tooling

The defenses above are only useful if we can verify they're applied. These tools catch miscompilation and coverage gaps automatically:

### `tools/scan_ud2.py` — Binary miscompilation scanner
Scans `ntoskrnl.exe` with objdump for `ud2` (illegal instruction) in named functions. LLVM replaces function bodies with ud2 when it proves undefined behavior exists — almost always from `captures(none)` or ABI mismatch. Expected ud2 in `unwrap_unchecked` panic stubs are excluded. Runs in ~30s, no boot needed.

Integrated into smoke tests as `test_no_unexpected_ud2`. Also runs automatically as a post-build check before QEMU boot (via conftest `_run_post_build_checks`).

### `tools/lint_kernel.py` — Source-level output pointer linter
Scans Rust source for `*OutputParam = value` writes in `#[no_mangle]` functions that don't use `write_volatile`. Every such write is vulnerable to `captures(none)` elimination. Catches the bug before compilation — no binary analysis needed.

### `tools/verify_sacred_primitives.sh` — IRQL/spinlock call verification
Checks that `KfRaiseIrql`, `KfLowerIrql`, `KeAcquireSpinLock` calls survive in the final binary. Integrated into smoke tests as `test_sacred_primitives`.

### Smoke test infrastructure (`tests/qemu/conftest.py`)
- **Clean Rust rebuild**: Nukes `rust-kernel-target/` before each test session to prevent stale cargo cache from masking regressions. Skip with `--skip-clean-build` for fast iteration.
- **Post-build checks**: ud2 scanner runs automatically after build, before QEMU boot.
- **BSOD auto-symbolication**: When a BUGCHECK is detected in serial output, automatically extracts exception code, crash RVA, and function name via objdump.

## Lessons Learned: Approaches That Failed

These are approaches we tried, found to be unworkable, and documented for future reference. Each failure taught us something about how Rust/LLVM interacts with kernel code.

### Failed: Removing `#[inline(never)]` from shim_exports (2026-03-22)

**Hypothesis:** Since `#[no_mangle]` prevents symbol removal, `#[inline(never)]` should be redundant.

**Result:** Boot hang on i386. LLVM still inlines the function *body* into callers within the same CGU, even though the symbol is preserved for the linker. The inlined body is then subject to `captures(none)` and other optimizations. `#[no_mangle]` prevents the *symbol* from being removed but does NOT prevent the *code* from being inlined into call sites.

**Lesson:** `#[no_mangle]` ≠ `#[inline(never)]`. They protect different things. Both are needed for shim functions that wrap FORCEINLINE kernel primitives.

### Failed: `Volatile<T>` wrapper on `#[no_mangle]` statics (2026-03-22)

**Hypothesis:** Wrapping `static mut X: KIRQL` as `static X: Volatile<KIRQL>` (using `UnsafeCell` for interior mutability) would force volatile load/store semantics while keeping the symbol exported.

**Result:** Boot hang on i386. Changing from `static mut` to `static` (even with `UnsafeCell`) causes Rust to place the symbol in a read-only `.rdata` section instead of writable `.data`. The first write to the global faults.

**Root cause:** Rust's `static` vs `static mut` affects linker section placement. `UnsafeCell` provides interior mutability at the *language* level (borrow checker), but the compiler's section placement logic doesn't consider `UnsafeCell` — it sees `static` (not `static mut`) and emits into `.rdata`. This is a Rust compiler behavior, not a language spec guarantee.

**Workaround:** Use `read_volatile`/`write_volatile` directly on `static mut` globals instead of wrapping in `Volatile<T>`. Same LLVM protection (volatile load/store instructions emitted), no linker section issue.

**`Volatile<T>` IS safe for:** Local variables, struct fields, non-`#[no_mangle]` statics — anywhere the linker section placement doesn't matter because the variable lives on the stack or in a heap allocation.

### Failed: Custom `AtomicUsize` SpinLockGuard (2026-03-21)

**Hypothesis:** SpinLockGuard can implement its own spinlock using `AtomicUsize::compare_exchange_weak` instead of calling the kernel's `KeAcquireSpinLock`.

**Result:** Deadlock on i386. The kernel's spinlock protocol is more complex than a simple test-and-set:
1. Queued spinlocks use a different data structure than a plain atomic flag
2. Debugging hooks (`KD_SPIN_LOCK_DEBUG`) in the C acquire path are bypassed
3. C code and Rust code racing on the same `KSPIN_LOCK` with different atomic implementations is technically a data race under the C++ memory model

**Lesson:** Never reimplement kernel lock primitives in Rust atomics. Always call the kernel's own functions (`KeAcquireSpinLock`/`KeReleaseSpinLock`) through the guard.

### Failed: `ObReferenceObjectByName` inner returning `Result<PVOID, NTSTATUS>` (2026-03-22)

**Hypothesis:** The inner function returns `Ok(object)` on success and the wrapper returns `STATUS_SUCCESS`.

**Result:** Black screen. `ObReferenceObjectByName` can return success codes OTHER than `STATUS_SUCCESS` (e.g., `STATUS_REPARSE`). Callers check for specific status values. Returning `STATUS_SUCCESS` when the original returned a different success code broke object lookup for the display driver.

**Lesson:** NT functions' NTSTATUS return values carry information even on success. Inner functions must return `Result<(OutputValue, NTSTATUS), NTSTATUS>` to preserve the exact status code, not `Result<OutputValue, NTSTATUS>` with `STATUS_SUCCESS` hardcoded in the wrapper.

### Failed: Global write barrier for CGU=1 (2026-03-22)

**Hypothesis:** LLVM constant-folds `pub static mut` globals initialized to 0 because it can see (with CGU=1) that no Rust code writes to them. A `#[no_mangle]` function that `write_volatile`s every global would prevent constant-folding because LLVM can't prove the function is never called.

**Result:** The barrier compiled and linked cleanly, but the 45 ud2s remained unchanged. The global write barrier did NOT reduce the ud2 count.

**Root cause was wrong:** The ud2s are NOT from constant-folding globals. Analysis of the disassembly revealed three distinct mechanisms:

1. **`noreturn` propagation** (5 sites): `PspCatchCriticalBreak` calls `KeBugCheckEx` (which is `-> !` / noreturn). LLVM with CGU=1 can inline `PspCatchCriticalBreak`, see the `KeBugCheckEx` call, and prove the code after the call is unreachable — inserting ud2. The global `KdDebuggerEnabled` check was a red herring; the real issue is LLVM proving the `handled == 0` path always leads to `KeBugCheckEx`.

2. **Interprocedural call graph analysis** (10+ sites): Functions that call `KeAttachProcess`, `PsGetCurrentThread`, or `KiExitDispatcher` get ud2 AFTER the call. LLVM with full CGU=1 visibility inlines these and finds UB in the inlined body — likely from `unsafe` pointer arithmetic that's technically UB in Rust but was safe in C.

3. **Entire body replacement** (30+ sites): Functions like `DbgkCreateThread`, `NtTerminateProcess`, `PsConvertToGuiThread` have their ENTIRE body replaced with `push ebp; mov ebp,esp; ud2`. LLVM proves the function body is unreachable from the first instruction — possibly through interprocedural alias analysis or calling convention verification.

**Lesson:** CGU=1's ud2 problem is NOT solvable by making globals volatile. It's caused by LLVM's interprocedural analysis finding (real or false) UB through the full call graph. The translated C→Rust code contains patterns that are valid C but technically UB in Rust (null pointer offsets, union access patterns, etc.). Fixing this requires either:
- Auditing every ud2 function for actual UB and fixing it
- Using `#[inline(never)]` on every function that LLVM misanalyzes (doesn't fully work — LLVM still does interprocedural analysis without inlining)
- Keeping CGU=16 where CGU boundaries accidentally prevent LLVM from seeing the full picture

**Current approach:** Stay on CGU=16 for now. CGU=1 enabled experimentally — down from 45 to 21 ud2s after ABI fixes. Remaining 21 traced to rustc codegen bug (see below).

### Root Cause Found: Rustc Emits Contradictory LLVM Attributes (2026-03-22)

**Discovery:** LLVM IR inspection revealed the REAL root cause of the CGU=1 ud2s. For 15 of 21 affected functions, **rustc itself** emits `unreachable` as the function body before LLVM even runs. The LLVM IR shows:

```llvm
define x86_stdcallcc i8 @DbgkpSuspendProcess() #10 {
start:
  unreachable
}

attributes #10 = { mustprogress noreturn willreturn memory(none) }
```

The attribute set includes `noreturn` AND `willreturn` — a logical contradiction. A function can't both "never return" and "will eventually return." LLVM resolves this by treating the function body as unreachable → ud2.

**15 functions with this bug:** DbgkCreateThread, DbgkExitProcess, DbgkExitThread, DbgkForwardException, DbgkMapViewOfSection, DbgkUnMapViewOfSection, DbgkpSendApiMessage, DbgkpSendApiMessageLpc, DbgkpSuspendProcess, FsRtlAllocateFileLock, IoQueryFileInformation, IoQueryVolumeInformation, IoSetInformation, NtTerminateProcess, PipAllocateDeviceNode.

**Remaining 6 functions** (IoOpenDeviceRegistryKey, IoSetHardErrorOrVerifyDevice, IopCloseFile, LpcpFreeToPortZone, MmCreateArm3Section, MmCreateImageSection) have normal attributes but ud2 inserted by LLVM optimization — a separate issue.

**Why rustc thinks they're `noreturn`:** Under investigation. Likely causes:
- `_SEH2_YIELD` is declared as `-> !` (noreturn) in dummy PSEH. If rustc traces through the call graph and finds a path that inevitably calls `_SEH2_YIELD`, it may mark the caller as `noreturn`.
- MIR-level analysis may prove a code path always reaches a `!`-returning function (like `KeBugCheckEx`) and propagate `noreturn` upward.
- With CGU=1, rustc has full visibility to trace these paths; with CGU=16, the analysis stops at CGU boundaries.

**Key insight:** This is NOT an LLVM bug — it's **rustc codegen** emitting contradictory attributes. The fix must be at the source level (prevent rustc from inferring `noreturn`) or by filing a rustc bug report if the inference is incorrect.

**Distinction from CGU=16:** With CGU=16, rustc can't see across CGU boundaries, so it can't trace call paths to `-> !` functions. The CGU boundaries accidentally prevent the incorrect `noreturn` inference.

### Breakthrough: ABI Mismatches Are the Primary CGU=1 Root Cause (2026-03-22)

**Discovery:** After the global barrier failed, deeper disassembly analysis revealed **ud2 contagion** — one function gets ud2, then every caller of that function also gets ud2, propagating up the call graph. Tracing the contagion chains to their roots revealed that `extern "C"` blocks (cdecl) were declaring stdcall/fastcall kernel functions. With CGU=16, LLVM can't see across the boundary to detect the mismatch. With CGU=1, it can.

**`verify_abi.py` already existed** and catches these mismatches perfectly (reports function name, declared convention, correct convention per ntoskrnl.spec/bindgen). It found **61 ABI mismatches** — but it wasn't integrated into the smoke tests and hadn't been run since it was created. New C→Rust translations kept adding `extern "C"` blocks because the LLM translation prompt defaults to `extern "C"`.

**Fix:** Changed 61 `extern "C"` blocks to `extern "system"` (stdcall) or `extern "fastcall"` as appropriate. Purely mechanical — the tool tells you exactly what each function should be.

**Result: CGU=1 ud2 count dropped from 45 to 21.** 24 ud2s were caused by ABI mismatches alone. The remaining 21 still need investigation — they may be further ABI mismatches that verify_abi.py doesn't cover (functions not in ntoskrnl.spec), or genuine UB in translated code.

**ud2 contagion pattern:** When function A has ud2, LLVM knows calling A is unreachable. So any function B that calls A without a guaranteed-taken early return also gets ud2. This propagates transitively — the Dbgk subsystem had 10 ud2s all traceable to a single root cause (`DbgkpSuspendProcess` calling `PsGetCurrentProcess` with wrong ABI → ud2 → every Dbgk function that calls `DbgkpSuspendProcess` also gets ud2).

**Action items:**
1. Integrate verify_abi.py into smoke tests (like scan_ud2 and lint_kernel)
2. Update LLM translation prompts to use `extern "system"` instead of `extern "C"` for NTAPI functions
3. Investigate remaining 21 ud2s — likely more ABI mismatches or actual UB in unsafe code
4. CGU=1 becomes viable once ud2 count reaches 0

### Root Cause Confirmed: LLVM Ignores Volatile Stores in Attribute Deduction (2026-03-28)

**Discovery:** Emitting LLVM IR for `PspCreateProcess` revealed the complete mechanism:

1. **Pre-optimization IR** has `store volatile ptr %src, ptr %dst` inside `write_volatile` — the volatile store exists
2. **Post-optimization IR** has `define ... @PspCreateProcess(ptr readnone captures(none) %process_handle)` — LLVM deduced `readnone captures(none)` despite the volatile store
3. **Callers** pass `ptr poison` for the parameter (since LLVM thinks it's unused)
4. **Function body** collapses: all stores through `poison` are UB → `unreachable` → `ud2`
5. **Cascade**: function becomes `noreturn` → every caller gets `unreachable` after calling it → 66+ functions affected

**LLVM's reasoning (technically correct per its semantics):**
- `captures(none)`: storing *through* a pointer isn't "capturing" it (the pointer value doesn't escape)
- `readnone`: LLVM's attribute deduction pass ignores volatile stores when computing memory effects on parameters
- Combined: `readnone captures(none)` = parameter is completely unused → callers can pass `poison`

This is arguably an LLVM bug — volatile stores should count as memory effects during attribute deduction. But it's consistent with LLVM's internal model where "capture" specifically means the pointer VALUE escaping, not effects through the pointer.

### Failed: write_volatile Sweep (2026-03-22 → reverted 2026-03-28)

**Hypothesis:** Converting all `*output_ptr = value` to `core::ptr::write_volatile(output_ptr, value)` in `#[no_mangle]` functions would prevent LLVM from eliminating output pointer writes via `captures(none)`.

**Result:** Paradoxically made things worse. The write_volatile sweep (247 conversions across 66 files) changed enough code to reshuffle LLVM's CGU=16 partitioning, exposing captures(none) bugs in DIFFERENT functions (PspCreateProcess, PspCreateThread, ExInitSystem) that were previously hidden by favorable CGU assignments.

**Why write_volatile alone can't work:** LLVM ignores volatile stores when deducing parameter attributes (see above). Even with `write_volatile`, LLVM still deduces `readnone captures(none)` → callers pass `poison` → volatile store through poison is UB → eliminated anyway.

**Lesson:** write_volatile protects against dead store elimination WITHIN a function, but NOT against interprocedural attribute deduction that poisons the parameter at call sites.

### Failed: ABI Unification (extern "stdcall" → extern "system") (2026-03-22 → reverted 2026-03-28)

**Hypothesis:** Unifying 2835 `extern "stdcall"` to `extern "system"` in bindgen_i386.rs (both are stdcall on i386, but rustc treats them as distinct types) would prevent future ABI mismatches.

**Result:** The type-level change to function pointer signatures in structs cascaded through the entire codebase. This reshuffled CGU partitioning enough to cause ud2 count to jump from 6 to 19 — introducing boot-blocking ud2s in PspCreateProcess and ExInitSystem.

**Lesson:** ANY code change — even nominally equivalent type aliasing — can destabilize CGU partitioning. ABI unification is correct in principle but must be done AFTER the captures(none) defense is in place.

### Validated: `opaque_ptr` Inline Asm Barrier (2026-03-28)

**Mechanism:** A zero-cost inline asm barrier that makes pointer parameters opaque to LLVM:

```rust
#[inline(always)]
pub unsafe fn opaque_ptr<T>(ptr: *mut T) -> *mut T {
    let p: *mut T;
    core::arch::asm!("mov {0}, {1}", out(reg) p, in(reg) ptr,
                     options(nostack, preserves_flags));
    p
}
```

The `out(reg) + in(reg)` forces the pointer through an opaque register transformation. LLVM must assume the output is an arbitrary address → can't deduce `captures(none)` or `readnone` → callers pass the real pointer → no poison → no ud2.

**Verified via LLVM IR:** Applied to PspCreateProcess's `process_handle` parameter. Before: `ptr readnone captures(none)`, callers pass `ptr poison`. After: `ptr noundef`, callers pass `ptr noundef %process_handle`. The attribute deduction was completely blocked.

**Limitation:** PspCreateProcess had a SECOND ud2 cause beyond captures(none) — LLVM proved a different code path unreachable through interprocedural analysis (the `llvm.assume` + security check path). opaque_ptr fixed the attribute deduction but not the path elimination.

**Also validated on iofunc.rs:** Applied to all PIO_STATUS_BLOCK parameters across 21 functions. But IoPageRead's ud2 came from a DIFFERENT parameter interaction — `MmIsFileObjectAPagingFile` is correctly deduced as `readnone` (it only compares the pointer address, doesn't dereference it), and this propagates to collapse the caller's code paths.

**Conclusion:** opaque_ptr works for its intended purpose (blocking attribute deduction on specific parameters), but must be applied to ALL pointer parameters of ALL `#[no_mangle]` functions to be effective against the full cascade. A targeted approach is whack-a-mole — the cascade propagates through any unprotected parameter.

### Current Workaround: CGU=256 (2026-03-28)

**Mechanism:** Setting `codegen-units = 256` gives each function near-isolation in its own compilation unit. LLVM's interprocedural analysis can only operate within a CGU, so with ~2-3 functions per CGU, it can't build the cross-function attribute chains that cause the cascade.

**Result:** i386 + amd64 6/6 smoke tests pass, 0 unexpected ud2s.

**Trade-off:** Slightly larger binary (less inlining across functions), slightly slower compilation. No runtime cost for the kernel.

**Fragility:** CGU assignment is not deterministic — any code change CAN reshuffle partitioning. CGU=256 minimizes the risk but doesn't eliminate it. The only durable fix is opaque_ptr barriers.

### RESOLVED AT THE TOOLCHAIN — LLVM 21 respects volatile stores in attribute deduction (2026-07-05)

**The root cause above ("LLVM Ignores Volatile Stores in Attribute Deduction",
2026-03-28) is FIXED in the toolchains we ship on** — stable rustc 1.92
(LLVM 21.1.3) and nightly 1.94 (LLVM 21.1.5). Synthetic repro of the exact
historical shape (a `#[no_mangle]` fn whose body is a lone `write_volatile`
through its pointer param, same-CGU caller, opt-level=1):

- **Then (LLVM 19/20):** `define @VolWriter(ptr readnone captures(none) %out)`,
  callers pass `ptr poison`, store folds, body may collapse to `ud2`.
- **Now (LLVM 21):** `define @VolWriter(ptr noundef %out)` — NO memory
  attributes deduced; the volatile store survives at every callsite. And plain
  (non-volatile) out-param stores now deduce `writeonly captures(none)
  initializes(...)`, which is equally fold-proof — callers must pass real
  pointers to anything the callee stores through, volatile or not.

Empirical confirmation on the real kernel body (`tools/scan_captures.py` over
the 256 post-opt CGU .ll files of ntoskrnl-rust, amd64, shipping profile):
**zero committed folds.** Every `ptr poison` callsite in the build traces to a
param the Rust source genuinely never uses (`let _ = param;` discards or
`_`-prefixed params — MiLoadImageSection's non-session `file_name`/`ldr_entry`,
FsRtlCancelOplockIIIrp's `DeviceObject`, NOT_IMPLEMENTED arbiter stubs), where
poison is legal and harmless. Every remaining `readnone captures(none)` param
(42 across 33 fns) is likewise an unused-in-source param, not an elided write.

Consequences:
- **write_volatile is a sound defense again.** The "Failed: write_volatile
  Sweep" lesson above is obsolete on LLVM ≥ 21 (its CGU-reshuffle caveat about
  mass changes remains true of any mass change).
- **The planned automated `opaque_ptr` tool below is unnecessary.** Existing
  `opaque_ptr` laundering stays (harmless, one reg-move), but new code doesn't
  need it for attribute-deduction protection.
- **`cgu=256` + `lto=false` stays** as belt-and-suspenders (it also firewalls
  the ABI-mismatch `noreturn` cascade, which is toolchain-independent).
- **The tripwire matters more than the fix:** rustc upgrades bump LLVM
  regularly, and a future LLVM could regress this. `tools/scan_captures.py`
  (post-opt IR audit: POISON_ARG / READNONE_OUTPARAM / UNREACHABLE_BODY,
  emitted via `VIBEOS_EMIT_LLVM_IR=1` through reactos-link) is the permanent
  detector — run it after every toolchain bump.

### Superseded: Automated `opaque_ptr` Tool (planned 2026-03, retired 2026-07-05)

Made unnecessary by the LLVM 21 fix above — kept for history. The plan was to
insert `let param = opaque_ptr(param);` for every pointer param of every
`#[no_mangle]` fn, to block the (then-broken) attribute deduction wholesale,
then re-enable the write_volatile sweep, ABI unification, and lower CGU counts.
The toolchain fix delivers the first two for free; CGU stays at 256 by choice.

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
> Global state → read_volatile/write_volatile (load/store visible)
> Trap state → Assembly (non-optimizable)
