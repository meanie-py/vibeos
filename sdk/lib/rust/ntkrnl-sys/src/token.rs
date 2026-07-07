//! RunToken — the right-to-run capability (execution-context ownership, Layer 1).
//!
//! `docs/engineering/execution-context-ownership-model.md`. A kernel context
//! switch is an `.await` point: everything valid only "while this frame is
//! executing on this CPU" — a raised-IRQL token, a held spinlock, a per-CPU
//! pointer — is the kernel's `!Send` and must not survive a voluntary wait.
//! `RunToken` models the right to run as a borrowable, brand-scoped, typestated
//! capability:
//!
//! - **`!Send + !Sync`** — certifies "*this* CPU, *this* thread, while this
//!   frame executes"; meaningless anywhere else.
//! - **Invariant brand lifetime** (the GhostCell trick): values minted from a
//!   token (`Pcpu`-style accessors, phase 3) carry `'brand` and become lifetime
//!   errors after the wait that consumed the token.
//! - **Typestate level** (`Passive`/`Apc`/`Dispatch`/`Interrupt`): the wait API
//!   is bounded on `Waitable` (Passive + Apc — NT legally waits at APC_LEVEL,
//!   `WaitIrql` exists for exactly that), so "wait from ISR/DPC context" is
//!   unrepresentable rather than a runtime assert.
//! - **Guards borrow it `&mut`** — a live guard means the token cannot be
//!   moved into the (by-value, CPS) wait: "sleep with a spinlock held" is a
//!   borrow-check error.
//!
//! The CPS wait API itself (`wait_for`) lives in `ntoskrnl-rust::ke::wait`
//! next to `KeWaitForSingleObject` (it needs the kernel's own types); this
//! module owns only the capability. While the C remnant exists, linearity is
//! **per-region**: exported `extern "system"` functions re-mint at entry
//! (`with_token` / `remint_at_ffi_entry`), so N live tokens per thread is the
//! steady state and cswitchdep remains the runtime witness across the seams.

use core::marker::PhantomData;

use crate::base::KIRQL;
use crate::guards::{IrqlGuard, SpinLockGuard};
use crate::ke::PKSPIN_LOCK;

/// Invariant lifetime brand: `'brand` in both argument and return position of
/// a `fn` pointer makes it invariant — `RunToken<'a>` never coerces to
/// `RunToken<'b>` (not even `'static`). Zero-sized, zero code.
pub struct InvariantLifetime<'brand>(PhantomData<fn(&'brand ()) -> &'brand ()>);

impl<'brand> InvariantLifetime<'brand> {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        InvariantLifetime(PhantomData)
    }
}

/// Typestate levels. Sealed: the set of execution-context classes is closed.
pub trait Level: sealed::Sealed {}
/// Thread context at PASSIVE_LEVEL.
pub struct Passive;
/// Thread context at APC_LEVEL (guarded region / special-APC-disabled) —
/// waits are still legal here (`KTHREAD.WaitIrql` encodes exactly this).
pub struct Apc;
/// Thread context at DISPATCH_LEVEL — no blocking waits (zero-timeout polls
/// only; not modeled yet).
pub struct Dispatch;
/// ISR/DPC context — no wait API exists for this state, by construction.
pub struct Interrupt;

impl Level for Passive {}
impl Level for Apc {}
impl Level for Dispatch {}
impl Level for Interrupt {}

/// Levels a blocking wait is legal from (NT: IRQL <= APC_LEVEL).
pub trait Waitable: Level {}
impl Waitable for Passive {}
impl Waitable for Apc {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Passive {}
    impl Sealed for super::Apc {}
    impl Sealed for super::Dispatch {}
    impl Sealed for super::Interrupt {}
}

/// The right to run on the current processor while this frame is executing.
/// See the module doc. Linear within a Rust region: a voluntary wait consumes
/// it (by value) and hands the continuation a freshly-branded one.
pub struct RunToken<'brand, L: Level = Passive> {
    _brand: InvariantLifetime<'brand>,
    _not_send: PhantomData<*const ()>, // !Send + !Sync
    _level: PhantomData<L>,
}

impl<'b, L: Level> RunToken<'b, L> {
    /// Internal constructor — every public minting path goes through
    /// `with_token` (callee-bound brand) or `remint_at_ffi_entry` (unsafe FFI
    /// escape hatch with a documented contract).
    #[inline(always)]
    pub(crate) fn conjure() -> Self {
        RunToken { _brand: InvariantLifetime::new(), _not_send: PhantomData, _level: PhantomData }
    }

    /// Raise IRQL, borrowing the token exclusively for the guard's lifetime:
    /// while the guard lives, the token cannot be moved into a wait.
    ///
    /// # Safety
    /// Same contract as [`IrqlGuard::raise`]: `new_irql` must be >= the
    /// current level, and the caller is genuine thread context.
    #[inline]
    pub unsafe fn raise<'g>(&'g mut self, new_irql: KIRQL) -> TokenIrqlGuard<'g> {
        TokenIrqlGuard { inner: IrqlGuard::raise(new_irql), _borrow: PhantomData }
    }

    /// Acquire a spinlock (raises to DISPATCH), borrowing the token
    /// exclusively — "sleep with this lock held" becomes a borrow error.
    ///
    /// # Safety
    /// Same contract as [`SpinLockGuard::acquire`].
    #[inline]
    pub unsafe fn acquire<'g>(&'g mut self, lock: PKSPIN_LOCK) -> TokenSpinLockGuard<'g> {
        TokenSpinLockGuard { inner: SpinLockGuard::acquire(lock), _borrow: PhantomData }
    }
}

/// [`IrqlGuard`] tied to an exclusive borrow of a `RunToken`.
#[must_use = "dropping the guard immediately lowers IRQL"]
pub struct TokenIrqlGuard<'g> {
    inner: IrqlGuard,
    _borrow: PhantomData<&'g mut ()>,
}

impl TokenIrqlGuard<'_> {
    #[inline]
    pub fn previous(&self) -> KIRQL {
        self.inner.previous()
    }
}

// A manual Drop impl is LOAD-BEARING, not style: dropck only requires a
// value's lifetime params to strictly outlive its scope when the type itself
// implements Drop. Without this, the `'g` in `PhantomData<&'g mut ()>` is not
// required live at the implicit drop, NLL ends the token loan immediately,
// and `wait_for(tok, ...)` with a live guard COMPILES — verified by a
// compile-fail probe on 2026-07-07. The body is empty; the inner guard's own
// Drop still lowers the IRQL / releases the lock.
impl Drop for TokenIrqlGuard<'_> {
    #[inline]
    fn drop(&mut self) {}
}

/// [`SpinLockGuard`] tied to an exclusive borrow of a `RunToken`.
#[must_use = "dropping the guard immediately releases the lock"]
pub struct TokenSpinLockGuard<'g> {
    inner: SpinLockGuard,
    _borrow: PhantomData<&'g mut ()>,
}

impl TokenSpinLockGuard<'_> {
    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.inner.previous_irql()
    }
}

// Load-bearing empty Drop — see TokenIrqlGuard above.
impl Drop for TokenSpinLockGuard<'_> {
    #[inline]
    fn drop(&mut self) {}
}

/// CPS wait — the RunToken-consuming form of `KeWaitForSingleObject`
/// (Layer 1, `execution-context-ownership-model.md`). Lives HERE (not in
/// ntoskrnl-rust) so every extracted kernel crate (po-rust, ex-rust, …) can
/// use it; the primitive-width signature matches the export's ABI exactly
/// (`KWAIT_REASON` = u32 enum, `KPROCESSOR_MODE` = i8, `BOOLEAN` = u8,
/// `PLARGE_INTEGER` = *mut i64, `NTSTATUS` = i32).
///
/// Takes the region's token **by value** (a live guard borrowing it makes
/// this call a borrow error — no sleeping with a lock held) and hands the
/// continuation a **freshly-branded** token: the higher-ranked `for<'new>`
/// bound means the caller cannot unify `'new` with the old brand, so
/// anything minted from the old token is a lifetime error after the wait.
/// `L: Waitable` restricts callers to Passive/Apc context.
///
/// # Safety
/// Same contract as `KeWaitForSingleObject` (valid dispatcher object, valid
/// or null timeout), plus the token must honestly describe the calling
/// context (see [`with_token`]).
#[inline]
pub unsafe fn wait_for<L: Waitable, R>(
    tok: RunToken<'_, L>,
    object: crate::base::PVOID,
    wait_reason: u32,
    wait_mode: i8,
    alertable: u8,
    timeout: *mut i64,
    k: impl for<'new> FnOnce(RunToken<'new, L>, i32) -> R,
) -> R {
    extern "system" {
        fn KeWaitForSingleObject(
            Object: crate::base::PVOID,
            WaitReason: u32,
            WaitMode: i8,
            Alertable: u8,
            Timeout: *mut i64,
        ) -> i32;
    }
    // Consume the caller's token: the CPU is being given back to the
    // scheduler; everything branded by `tok` dies here.
    drop(tok);
    let status = KeWaitForSingleObject(object, wait_reason, wait_mode, alertable, timeout);
    // SAFETY: back in the same thread context at the same typestate level
    // (KeWaitForSingleObject restores the entry IRQL); the fresh brand is
    // callee-bound via the for<'new> closure bound.
    with_token::<L, R>(|fresh| k(fresh, status))
}

/// Run `f` with a freshly-minted, callee-bound-brand token. The higher-ranked
/// closure bound means the caller cannot name `'new`, so the brand cannot be
/// unified with any other token's brand — this is the sound generativity shape
/// (a plain `fn() -> RunToken<'new>` would let the caller pick `'new`).
///
/// This is the sanctioned FFI-entry idiom: an exported `extern "system"`
/// function called from C mints its region's token here, once, at the top.
///
/// # Safety
/// Caller must be genuine thread context (not ISR/DPC unless `L = Interrupt`),
/// actually at the IRQL class `L` claims. Every call site is greppable; the
/// planned `irqlck` pass verifies mint sites are true entry points.
#[inline]
pub unsafe fn with_token<L: Level, R>(f: impl for<'new> FnOnce(RunToken<'new, L>) -> R) -> R {
    f(RunToken::conjure())
}

/// Raw FFI escape hatch: mint a token with a caller-chosen brand. Prefer
/// [`with_token`] — with this form the caller can unify two tokens' brands,
/// which weakens the stale-per-CPU-reference guarantee (never soundness of
/// memory, but of the discipline). Exists for signatures where CPS cannot be
/// threaded (deep C callback chains).
///
/// # Safety
/// As [`with_token`], plus: the caller must not use the chosen `'brand` to
/// outlive values across a wait.
#[inline(always)]
pub unsafe fn remint_at_ffi_entry<'b, L: Level>() -> RunToken<'b, L> {
    RunToken::conjure()
}
