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

### IrqlGuard

```rust
// In ntkrnl-sys/src/guards.rs (new module)

/// RAII guard that restores IRQL when dropped.
/// The old IRQL is a live value — LLVM cannot eliminate the Drop.
pub struct IrqlGuard {
    previous: KIRQL,
}

impl IrqlGuard {
    /// Raise IRQL and return a guard that restores it on drop.
    #[inline]
    pub unsafe fn raise(new_irql: KIRQL) -> Self {
        let previous = irql_raise(new_irql);
        Self { previous }
    }

    /// Raise to DISPATCH_LEVEL.
    #[inline]
    pub unsafe fn raise_to_dpc() -> Self {
        Self::raise(DISPATCH_LEVEL)
    }

    /// Get the saved (previous) IRQL.
    #[inline]
    pub fn previous(&self) -> KIRQL {
        self.previous
    }
}

impl Drop for IrqlGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { irql_lower(self.previous) }
    }
}
```

### SpinLockGuard

```rust
/// RAII guard for spinlocks. Acquires on creation, releases on drop.
/// Embeds the IRQL guard so IRQL is automatically restored.
pub struct SpinLockGuard {
    lock: *mut KSPIN_LOCK,
    irql: IrqlGuard,
}

impl SpinLockGuard {
    /// Acquire a spinlock, raising IRQL to DISPATCH_LEVEL.
    #[inline]
    pub unsafe fn acquire(lock: *mut KSPIN_LOCK) -> Self {
        let irql = IrqlGuard::raise_to_dpc();
        // Atomic spinlock acquire
        while core::sync::atomic::AtomicUsize::from_ptr(lock as *mut usize)
            .compare_exchange_weak(0, 1,
                core::sync::atomic::Ordering::Acquire,
                core::sync::atomic::Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self { lock, irql }
    }
}

impl Drop for SpinLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Release spinlock
            core::sync::atomic::AtomicUsize::from_ptr(self.lock as *mut usize)
                .store(0, core::sync::atomic::Ordering::Release);
            // IrqlGuard drop restores IRQL automatically
        }
    }
}
```

### PfnLockGuard (Specialized)

```rust
/// Guard for the PFN database lock — a frequently used pattern.
pub struct PfnLockGuard {
    inner: SpinLockGuard,
}

impl PfnLockGuard {
    #[inline]
    pub unsafe fn acquire() -> Self {
        extern "C" { static mut MmPfnLock: KSPIN_LOCK; }
        Self { inner: SpinLockGuard::acquire(core::ptr::addr_of_mut!(MmPfnLock)) }
    }
}
// Drop is automatic via inner SpinLockGuard
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

### Phase 1: Build the Primitives (ntkrnl-sys)
1. Add `ntkrnl-sys/src/inline/irql.rs` — sacred IRQL primitives with asm barriers
2. Add `ntkrnl-sys/src/guards.rs` — IrqlGuard, SpinLockGuard, PfnLockGuard
3. Verify on all 3 architectures with smoke tests

### Phase 2: Migrate Hot Paths First
Priority targets (high call count + known LLVM victims):
1. **ps/thread.rs** — PspCreateThread output pointers → Result (Bug 1 fix)
2. **ke/procobj.rs** — KeAttachProcess/KeDetachProcess IRQL management
3. **ARM3/procsup.rs** — MiAcquirePfnLock/MiReleasePfnLock → PfnLockGuard
4. **ke/wait.rs** — KiExitDispatcher IRQL restoration

### Phase 3: Systematic Migration
Use `verify_ffi.py` to identify remaining C-style patterns. Migrate subsystem by subsystem:
- KE (62 files, 906 spinlock sites)
- MM (30 files, heavy PFN lock usage)
- PS (13 files, output pointers)
- OB, SE, EX, IO (already using some Result patterns via probe API)

### Phase 4: Remove Shims
As guards replace manual lock/unlock patterns, the FORCEINLINE C shims for lock functions become unnecessary. Each migrated subsystem allows removing its corresponding shims.

## Current State (for reference)

| Pattern | Call Sites | Guard Instances | Migration Status |
|---------|-----------|-----------------|------------------|
| IRQL raise/lower | 711 | 0 | Not started |
| Spinlock acquire/release | 906 | 0 | Not started |
| Output pointers (Nt* exports) | ~100 | 0 | Not started |
| PFN lock | ~30 | 0 | Not started |
| Result-based probe API | 97 | N/A | Done (SE+OB) |

## Relationship to Other Docs

- `shim-elimination.md` — Guard migration enables shim removal (Category 1+4)
- `arm64-debug.md` — Bug 1 would be prevented by Result pattern; Bug 2 is fixed by ASM dispatch
- `AGENTS.md` — Translation rules should be updated to use guards for new translations

## Key Principle

> Stop encoding kernel invariants as invisible side effects.
> Make them visible values that the optimizer must preserve.
>
> IRQL → IrqlGuard (scoped lifetime)
> Output pointers → Result (return value)
> Spinlocks → SpinLockGuard (RAII)
> Trap state → Assembly (non-optimizable)
