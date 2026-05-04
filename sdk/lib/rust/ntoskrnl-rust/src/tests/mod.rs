//! In-kernel test harness — formerly `ntoskrnl/tests/`, now Rust.
//!
//! Two parts:
//!  - `dispatch` — full translation of `dispatch.c`. Serial command loop
//!    invoked from `Phase1InitializationDiscard` when the `/TESTKERNEL` boot
//!    option is set (and `REACTOS_TEST_KERNEL` was defined at compile time
//!    on the C side).
//!  - `stubs` — the 28 `NtTestX` test entry points. **All currently stubbed
//!    out**: each prints `[TEST STUB] X — needs translation` and returns
//!    `TRUE`. The original C bodies (~5K LOC across 28 files) were lights-out
//!    cleanup of the tests/ directory; real translations are deferred.
//!
//! ## What's broken
//!
//! 1. **Bodies don't actually exercise anything.** The harness will report
//!    every test as PASS without touching the kernel. To restore real
//!    coverage, port each test_*.c body to its `NtTestX` stub in `stubs.rs`
//!    (see the per-stub TODO comments).
//! 2. **The hook in Phase1Init is dead.** `Phase1InitializationDiscard` was
//!    ported to Rust without the `#ifdef REACTOS_TEST_KERNEL` block that
//!    used to call `NtTestKernelDispatchLoop()` from the C path. Even with
//!    `-DREACTOS_TEST_KERNEL=ON` and `/TESTKERNEL` on the command line, the
//!    dispatch loop is currently never invoked. Restoring the hook is a
//!    ~5-line patch in `ex/init.rs::Phase1InitializationDiscard`.
//!
//! Together (1) and (2) mean the QEMU `test_kernel.py` harness is currently
//! a build-only artifact. The point of this translation pass was to remove
//! the C, not to restore behavior. See `docs/engineering/ntoskrnl-translation-audit.md`.

pub mod dispatch;
pub mod stubs;
