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

```rust
// In ntkrnl-sys/src/inline/irql.rs (new module)

/// Raise IRQL — returns old IRQL. Uses architecture-specific implementation
/// that LLVM cannot optimize away.
#[inline(always)]
pub unsafe fn irql_raise(new_irql: KIRQL) -> KIRQL {
    let old: KIRQL;
    #[cfg(target_arch = "x86_64")]
    {
        // Direct CR8 access — asm is opaque to LLVM
        let old64: u64;
        core::arch::asm!(
            "mov {old}, cr8",
            "mov cr8, {new}",
            old = out(reg) old64,
            new = in(reg) new_irql as u64,
        );
        old = old64 as KIRQL;
    }
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 — call HAL via opaque extern (separate compilation unit)
        extern "C" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        // asm barrier prevents LLVM from proving the call is dead
        core::arch::asm!("", options(nostack));
        old = KfRaiseIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        core::arch::asm!("", options(nostack));
        old = KfRaiseIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
    old
}

/// Lower IRQL — restores previous IRQL.
#[inline(always)]
pub unsafe fn irql_lower(new_irql: KIRQL) {
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("mov cr8, {}", in(reg) new_irql as u64);
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn KfLowerIrql(NewIrql: KIRQL); }
        core::arch::asm!("", options(nostack));
        KfLowerIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfLowerIrql(NewIrql: KIRQL); }
        core::arch::asm!("", options(nostack));
        KfLowerIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
}

/// Read current IRQL without changing it.
#[inline(always)]
pub unsafe fn irql_current() -> KIRQL {
    #[cfg(target_arch = "x86_64")]
    {
        let val: u64;
        core::arch::asm!("mov {}, cr8", out(reg) val);
        val as KIRQL
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn KeGetCurrentIrql() -> KIRQL; }
        core::arch::asm!("", options(nostack));
        let r = KeGetCurrentIrql();
        core::arch::asm!("", options(nostack));
        r
    }
    #[cfg(target_arch = "x86")]
    {
        extern "system" { fn KeGetCurrentIrql() -> KIRQL; }
        core::arch::asm!("", options(nostack));
        let r = KeGetCurrentIrql();
        core::arch::asm!("", options(nostack));
        r
    }
}
```

### Spinlock Acquire/Release

Similar pattern — the actual lock acquire (atomic exchange) is inline asm, and the IRQL transition uses the sacred IRQL primitives above.

### Trap Entry/Exit (ARM64)

Already in assembly (`ntoskrnl/ke/arm64/trap.S`, `zwdispatch.S`). No change needed — this stays ASM permanently.

## Layer 2: Guard Primitives

All implemented in `ntkrnl-sys/src/guards.rs` and `ntoskrnl-rust/src/ARM3/helpers.rs`.

### IrqlGuard

```rust
// ntkrnl-sys/src/guards.rs
pub struct IrqlGuard { previous: KIRQL }
impl IrqlGuard {
    pub unsafe fn raise(new_irql: KIRQL) -> Self { ... }
    pub unsafe fn raise_to_dpc() -> Self { ... }
    pub unsafe fn raise_to_apc() -> Self { ... }
    pub fn previous(&self) -> KIRQL { ... }
}
impl Drop for IrqlGuard { fn drop(&mut self) { irql_lower(self.previous) } }
```

### SpinLockGuard

**IMPORTANT:** Uses kernel's `KeAcquireSpinLock`/`KeReleaseSpinLock`, NOT a custom atomic spin loop. A custom `AtomicUsize` spin loop was tried and caused deadlocks on i386 because the kernel's spinlock protocol involves more than just an atomic exchange (queued spinlocks, debugging hooks, etc.).

```rust
// ntkrnl-sys/src/guards.rs
pub struct SpinLockGuard { lock: PKSPIN_LOCK, previous_irql: KIRQL }
impl SpinLockGuard {
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        let mut previous_irql: KIRQL = 0;
        KeAcquireSpinLock(lock, &mut previous_irql);
        Self { lock, previous_irql }
    }
}
impl Drop for SpinLockGuard {
    fn drop(&mut self) { KeReleaseSpinLock(self.lock, self.previous_irql) }
}
```

### SpinLockAtDpcGuard

For nested spinlock acquisition when already at DISPATCH_LEVEL.

```rust
// ntkrnl-sys/src/guards.rs
pub struct SpinLockAtDpcGuard { lock: PKSPIN_LOCK }
impl SpinLockAtDpcGuard {
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self { ... }
}
impl Drop for SpinLockAtDpcGuard {
    fn drop(&mut self) { KeReleaseSpinLockFromDpcLevel(self.lock) }
}
```

### PfnLockGuard

Lives in `ntoskrnl-rust` (not `ntkrnl-sys`) because `MiAcquirePfnLock`/`MiReleasePfnLock` are kernel-internal.

```rust
// ntoskrnl-rust/src/ARM3/helpers.rs
pub struct PfnLockGuard { previous_irql: KIRQL }
impl PfnLockGuard {
    pub unsafe fn acquire() -> Self { Self { previous_irql: MiAcquirePfnLock() } }
}
impl Drop for PfnLockGuard {
    fn drop(&mut self) { MiReleasePfnLock(self.previous_irql) }
}
```

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

### Phase 3: Cross-Function Patterns — IN PROGRESS
The main remaining blocker. ~300+ call sites where `OldIrql` is passed to another function that releases the lock internally.

**Key cross-function families:**
- `MiCheckControlArea(ControlArea, OldIrql)` — called from 6+ functions in ARM3/section.rs
- `MmDereferenceSegmentWithLock(Segment, OldIrql)` — called from 4+ functions in mm/section.rs
- `MiResolveProtoPteFault(OldIrql, ...)` / `MiCompleteProtoPteFault(OldIrql, ...)` — page fault chain
- `MiMakeSystemAddressValidPfn(Pfn, OldIrql)` — releases lock, maps page, reacquires

See "Cross-Function Pattern Analysis" section below.

### Phase 4: Complex Control Flow — NOT STARTED
High-acquire-count files needing restructuring:
- `ex/timer.rs` (38 calls) — timer cancellation/insertion with multiple lock types
- `pnpmgr/devaction.rs` (50 calls) — PnP device state machine
- `cc/view.rs` (48 calls) — cache manager view management
- `ke/dpc.rs` (21 calls) — DPC queue management

### Phase 5: Output Pointers → Result — NOT STARTED
Priority targets for `captures(none)` protection:
1. **ps/thread.rs** — PspCreateThread output pointers → Result
2. **ke/procobj.rs** — KeAttachProcess/KeDetachProcess IRQL management
3. **ke/wait.rs** — KiExitDispatcher IRQL restoration

### Phase 6: Remove Shims — NOT STARTED
Depends on Phase 3-4 completion per subsystem.

## Current State (as of 2026-03-22)

| Pattern | Remaining Calls | Guard Instances | Status |
|---------|----------------|-----------------|--------|
| PFN lock (MiAcquire/ReleasePfnLock) | ~93 acquire + ~195 release | ~55 guards | Phase 2 done, Phase 3 in progress |
| Spinlock (KeAcquire/ReleaseSpinLock) | ~102 acquire + ~120 release | ~40 guards | Phase 2 done |
| SpinlockAtDpc | ~51 acquire + ~57 release | ~5 guards | Phase 2 done |
| IRQL (KeRaiseIrql/KeLowerIrql) | ~139 raise + ~190 lower | ~48 guards | Phase 2 done |
| Output pointers (Nt* exports) | ~100 | 0 | Not started |
| Result-based probe API | 97 | N/A | Done (SE+OB) |
| **Total guard instances** | | **~148** | |

## Cross-Function Pattern Analysis

The remaining ~300 sites are dominated by a few recurring cross-function patterns. These can't be migrated with simple guard replacement — they require API redesign.

### Pattern: Lock-Passing Chain (MiCheckControlArea)

```rust
// Current: OldIrql threaded through call chain
fn MiDereferenceControlArea(ControlArea: PCONTROL_AREA) {
    OldIrql = MiAcquirePfnLock();          // acquire here
    MiCheckControlArea(ControlArea, OldIrql);  // release inside
}

fn MiCheckControlArea(ControlArea: PCONTROL_AREA, OldIrql: KIRQL) {
    // ... check conditions ...
    MiReleasePfnLock(OldIrql);  // release buried in callee
}
```

**Options:**
1. **Guard-passing**: Change callee to accept `PfnLockGuard` by value (move semantics)
2. **Split acquire/check**: Return a bool, let caller handle lock lifecycle
3. **Callback**: Callee takes a closure that runs under the lock

### Pattern: Release-Work-Reacquire (MiMakeSystemAddressValidPfn)

```rust
fn MiSegmentDelete(Segment: PSEGMENT) {
    old_irql = MiAcquirePfnLock();
    // ... work under lock ...
    MiMakeSystemAddressValidPfn(pfn, old_irql);  // releases, maps page, reacquires
    // ... more work under lock ...
    MiReleasePfnLock(old_irql);
}
```

**Options:**
1. **Guard-passing with return**: Callee consumes guard, returns new guard
2. **Split into explicit phases**: `drop(guard); do_work(); guard = acquire();`

### Pattern: Conditional Lock Ownership (MmDereferenceSegmentWithLock)

```rust
fn MmDereferenceSegmentWithLock(Segment: PMM_SECTION_SEGMENT, OldIrql: KIRQL) {
    if InterlockedDecrement64(&mut (*Segment).RefCount) == 0 {
        // ... cleanup, releases lock somewhere inside ...
    }
    MiReleasePfnLock(OldIrql);  // unconditional release at end
}
```

**Options:**
1. **Accept guard by value**: `fn ...(guard: PfnLockGuard)` — drop handles release
2. **Return remaining IRQL**: If callee needs to release early in some paths

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
