/*
 * ke_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust KE subsystem calls via extern declarations.
 *
 * Common shims (CONTAINING_RECORD, InsertTailList, etc.)
 * are provided by ob/ob_rust_shims.c and tier1_rust_shims.c.
 *
 * KE dispatcher/thread lock and spinlock shims are shared with EX subsystem —
 * they are the canonical location here (moved from ex_rust_shims.c).
 */

/* ========================================================================
 * Phase 1: Rename FORCEINLINE functions BEFORE including headers
 *
 * By #defining the function name to _saved_Name before headers are parsed,
 * the FORCEINLINE definition creates a _saved_Name static inline. After
 * headers, we #undef and define real non-inline functions with original names.
 * ======================================================================== */

/* Phase 1 rename: capture FORCEINLINE bodies as `_saved_<Name>` so the
 * non-inline definitions later in this file can reuse the original symbol
 * names. Only the functions still consumed below need this — fns whose C
 * body is now a one-line `_rs_*` delegation into a sibling rlib do NOT
 * need the rename and are not listed.
 *
 * KeRaiseIrqlToSynchLevel is NTHALAPI (dllimport) on i386 — uses a direct
 * IAT call below, not the rename pattern.
 * KeQuerySystemTime is a function-like macro (not FORCEINLINE), provided
 * by ps_rust_shims.c.
 * KeGetPcr is a macro (i386 __readfsdword) — a static __inline trampoline
 * captures the value at the function definition site, see below. */

/* Generic ke_x.h FORCEINLINE outlines used by KiAttachProcess +
 * KiRemoveEntryTimer. */
#define KiReleaseDispatcherLockFromSynchLevel _saved_KiReleaseDispatcherLockFromSynchLevel
#define KiReleaseApcLockFromSynchLevel  _saved_KiReleaseApcLockFromSynchLevel
#define KiRemoveEntryTimer              _saved_KiRemoveEntryTimer

/* i386 FORCEINLINE functions from internal/i386/ke.h, intrin_i.h, trap_x.h
 * and ke.h. */
#ifdef _M_IX86
#define KiCheckForApcDelivery           _saved_KiCheckForApcDelivery
#define KiExitV86Trap                   _saved_KiExitV86Trap
#define KiExitTrapDebugChecks           _saved_KiExitTrapDebugChecks
#define KiExitSystemCallDebugChecks     _saved_KiExitSystemCallDebugChecks
#endif

/* ========================================================================
 * Phase 2: Include headers
 * ======================================================================== */

/* Suppress DPRINT/DPRINT1 (this file is PCH-skipped) */
#ifndef DPRINT
#define DPRINT(...) do {} while(0)
#endif
#ifndef DPRINT1
#define DPRINT1(...) do {} while(0)
#endif

#include <ntoskrnl.h>

/* ========================================================================
 * Phase 3: Undef Phase 1 renames
 * ======================================================================== */

#undef KiReleaseDispatcherLockFromSynchLevel
#undef KiReleaseApcLockFromSynchLevel
#undef KiRemoveEntryTimer

#ifdef _M_IX86
#undef KiCheckForApcDelivery
#undef KiEndInterrupt
#undef KiExitV86Trap
#undef KiExitTrapDebugChecks
#undef KiExitSystemCallDebugChecks
#endif

/*
 * On i386, KeReleaseInStackQueuedSpinLock / KeAcquireInStackQueuedSpinLock are
 * declared with _DECL_HAL_KE_IMPORT (= DECLSPEC_IMPORT) in the SDK headers,
 * so the compiler generates indirect calls through the HAL's IAT.  However,
 * Rust shim_exports.rs provides local UP implementations that do NOT store
 * (KeGetCurrentThread()|1) into the spin lock (they use KxAcquireSpinLock which
 * is a no-op on UP).  The HAL's KeReleaseInStackQueuedSpinLock checks for that
 * value and bugchecks on mismatch.  Re-declare without __declspec(dllimport)
 * so the linker resolves to the Rust-provided version (matching UP semantics).
 */
#ifdef _M_IX86
VOID FASTCALL KeReleaseInStackQueuedSpinLock(IN PKLOCK_QUEUE_HANDLE LockHandle);
VOID FASTCALL KeAcquireInStackQueuedSpinLock(IN PKSPIN_LOCK SpinLock, IN PKLOCK_QUEUE_HANDLE LockHandle);
#endif

/* ========================================================================
 * Phase 4: Real (non-static, non-inline) function implementations
 *
 * Rust calls these via extern "C" (cdecl). No NTAPI unless noted.
 *
 * NOTE: Generic ke_x.h functions are now implemented in Rust
 * (ntkrnl-sys/src/inline/ke.rs) with export wrappers in shim_exports.rs.
 * ======================================================================== */

/* --- Dispatcher/thread lock (ke_x.h) — shared with EX subsystem ---
 *
 * Phase 1 batch 3: dispatcher lock family bodies live in
 * `sdk/lib/rust/ke-inlines/src/lib.rs` as `_rs_KiAcquireDispatcherLock` etc.
 * The C trampoline keeps the IRQL-debug instrumentation (was added during
 * boot triage and is too useful to drop yet) before delegating. The
 * `_saved_KiAcquireDispatcherLock` outline is no longer called by any
 * surviving C body — kept in scope as a documented fallback path.
 */

extern KIRQL _rs_KiAcquireDispatcherLock(VOID);
extern void  _rs_KiReleaseDispatcherLock(KIRQL OldIrql);
extern void  _rs_KiAcquireDispatcherObject(DISPATCHER_HEADER *Object);
extern void  _rs_KiReleaseDispatcherObject(DISPATCHER_HEADER *Object);
extern void  _rs_KiAcquireDispatcherLockAtSynchLevel(VOID);
extern void  _rs_KiReleaseDispatcherLockFromSynchLevel(VOID);

KIRQL KiAcquireDispatcherLock(VOID)
{
    KIRQL cur = KeGetCurrentIrql();
    if (cur > DISPATCH_LEVEL) {
        DPRINT1("KiAcquireDispatcherLock at IRQL %d! caller=%p\n",
                cur, _ReturnAddress());
    }
    return _rs_KiAcquireDispatcherLock();
}

VOID KiReleaseDispatcherLock(IN KIRQL OldIrql)
{
    _rs_KiReleaseDispatcherLock(OldIrql);
}

/* --- Thread / PRCB lock (ke_x.h, UP no-ops) ---
 * Phase 1 batch 4 — bodies in ke-inlines.
 */

extern void    _rs_KiAcquireThreadLock(PKTHREAD Thread);
extern void    _rs_KiReleaseThreadLock(PKTHREAD Thread);
extern BOOLEAN _rs_KiTryThreadLock(PKTHREAD Thread);
extern void    _rs_KiAcquirePrcbLock(PKPRCB Prcb);
extern void    _rs_KiReleasePrcbLock(PKPRCB Prcb);

VOID KiAcquireThreadLock(IN PKTHREAD Thread)
{
    _rs_KiAcquireThreadLock(Thread);
}

VOID KiReleaseThreadLock(IN PKTHREAD Thread)
{
    _rs_KiReleaseThreadLock(Thread);
}

/* --- Spinlock acquire/release (spinlock.h) ---
 * Phase 1 batch 7 — bodies in ke-inlines (compiler_fence(SeqCst), same
 * semantics as KeMemoryBarrierWithoutFence: no machine-code emission,
 * just a reordering barrier). DBG owner tracking stays disabled — see
 * ke-inlines for the reason.
 */

extern void _rs_KxAcquireSpinLock(PKSPIN_LOCK SpinLock);
extern void _rs_KxReleaseSpinLock(PKSPIN_LOCK SpinLock);

VOID KxAcquireSpinLock(IN PKSPIN_LOCK SpinLock)
{
    _rs_KxAcquireSpinLock(SpinLock);
}

VOID KxReleaseSpinLock(IN PKSPIN_LOCK SpinLock)
{
    _rs_KxReleaseSpinLock(SpinLock);
}

/* --- Dispatcher object lock + dispatcher-lock at-synch pair (ke_x.h) ---
 * Bodies in ke-inlines (Phase 1 batch 3). All four are UP no-ops; the
 * SMP implementations (CONFIG_SMP path of ke_x.h) would matter on real MP.
 */

VOID KiAcquireDispatcherObject(IN DISPATCHER_HEADER *Object)
{
    _rs_KiAcquireDispatcherObject(Object);
}

VOID KiReleaseDispatcherObject(IN DISPATCHER_HEADER *Object)
{
    _rs_KiReleaseDispatcherObject(Object);
}

VOID KiAcquireDispatcherLockAtSynchLevel(VOID)
{
    _rs_KiAcquireDispatcherLockAtSynchLevel();
}

VOID KiReleaseDispatcherLockFromSynchLevel(VOID)
{
    _rs_KiReleaseDispatcherLockFromSynchLevel();
}

/* --- Deferred ready / sched helpers (ke_x.h) ---
 * Phase 1 batch 5 — bodies in ke-inlines.
 * KiInsertDeferredReadyList: KTHREAD State + DeferredProcessor write +
 *                            KiDeferredReadyThread call (only non-no-op).
 * KiRescheduleThread / KiSetThreadSwapBusy / KiCheckDeferredReadyList /
 * KiRequestApcInterrupt: UP no-ops.
 */

extern void _rs_KiInsertDeferredReadyList(PKTHREAD Thread);
extern void _rs_KiRescheduleThread(BOOLEAN NewThread, ULONG Cpu);
extern void _rs_KiSetThreadSwapBusy(PKTHREAD Thread);
extern void _rs_KiCheckDeferredReadyList(PKPRCB Prcb);
extern void _rs_KiRequestApcInterrupt(BOOLEAN NeedApc, UCHAR Processor);

VOID KiInsertDeferredReadyList(IN PKTHREAD Thread)
{
    _rs_KiInsertDeferredReadyList(Thread);
}

VOID KiRescheduleThread(IN BOOLEAN NewThread, IN ULONG Cpu)
{
    _rs_KiRescheduleThread(NewThread, Cpu);
}

VOID KiSetThreadSwapBusy(IN PKTHREAD Thread)
{
    _rs_KiSetThreadSwapBusy(Thread);
}

/* --- PRCB lock + KiTryThreadLock (ke_x.h, UP no-ops) ---
 * Phase 1 batch 4 — bodies in ke-inlines (declared near KiAcquireThreadLock).
 */

VOID KiAcquirePrcbLock(IN PKPRCB Prcb)
{
    _rs_KiAcquirePrcbLock(Prcb);
}

VOID KiReleasePrcbLock(IN PKPRCB Prcb)
{
    _rs_KiReleasePrcbLock(Prcb);
}

BOOLEAN KiTryThreadLock(IN PKTHREAD Thread)
{
    return _rs_KiTryThreadLock(Thread);
}

VOID KiCheckDeferredReadyList(IN PKPRCB Prcb)
{
    _rs_KiCheckDeferredReadyList(Prcb);
}

VOID KiRequestApcInterrupt(IN BOOLEAN NeedApc, IN UCHAR Processor)
{
    _rs_KiRequestApcInterrupt(NeedApc, Processor);
}

/* --- Timer lock + table index (ke_x.h) ---
 * Phase 1 batch 6 — bodies in ke-inlines.
 * UP: lock acquire returns NULL, release no-op. Table-index is
 * `(DueTime / KeMaximumIncrement) & (TIMER_TABLE_SIZE - 1)`.
 */

extern PKSPIN_LOCK_QUEUE _rs_KiAcquireTimerLock(ULONG Hand);
extern void              _rs_KiReleaseTimerLock(PKSPIN_LOCK_QUEUE LockQueue);
extern ULONG             _rs_KiComputeTimerTableIndex(ULONGLONG DueTime);

PKSPIN_LOCK_QUEUE KiAcquireTimerLock(IN ULONG Hand)
{
    return _rs_KiAcquireTimerLock(Hand);
}

VOID KiReleaseTimerLock(IN PKSPIN_LOCK_QUEUE LockQueue)
{
    _rs_KiReleaseTimerLock(LockQueue);
}

/* --- APC / Process / DeviceQueue locks (ke_x.h) ---
 *
 * Phase 1 augment pattern (cm-inlines pilot 2 — see
 * docs/engineering/shim-elimination.md): each C symbol stays as the ABI
 * anchor; body collapses to a one-line delegating call into the matching
 * `_rs_Ki*` Rust impl in the isolated `sdk/lib/rust/ke-inlines/` rlib.
 * Existing Rust callers (in ntoskrnl-rust + ntkrnl-sys bindgen) keep
 * resolving `Ki{Acquire,Release}*` to the C symbol below — no source
 * change in ntoskrnl-rust for this batch. The `_saved_*` FORCEINLINE
 * outlines (Phase 1 rename macros at the top of this file) are kept in
 * scope as a documented fallback path if the ke-inlines crate is ever
 * unwired. Process lock note: `KiAcquireProcessLockRaiseToSynch` and
 * `KiReleaseProcessLock` are PCH-covered and live in mm_rust_shims.c —
 * only the FromSynchLevel half lives here.
 */

extern void _rs_KiAcquireApcLockRaiseToSynch(PKTHREAD Thread, PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiAcquireApcLockAtSynchLevel(PKTHREAD Thread, PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiAcquireApcLockRaiseToDpc(PKTHREAD Thread, PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiReleaseApcLock(PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiReleaseApcLockFromSynchLevel(PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiReleaseProcessLockFromSynchLevel(PKLOCK_QUEUE_HANDLE Handle);
extern void _rs_KiAcquireDeviceQueueLock(PKDEVICE_QUEUE DeviceQueue, PKLOCK_QUEUE_HANDLE DeviceLock);
extern void _rs_KiReleaseDeviceQueueLock(PKLOCK_QUEUE_HANDLE DeviceLock);

VOID KiAcquireApcLockRaiseToSynch(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiAcquireApcLockRaiseToSynch(Thread, Handle);
}

VOID KiAcquireApcLockAtSynchLevel(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiAcquireApcLockAtSynchLevel(Thread, Handle);
}

VOID KiAcquireApcLockRaiseToDpc(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiAcquireApcLockRaiseToDpc(Thread, Handle);
}

VOID KiReleaseApcLock(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiReleaseApcLock(Handle);
}

VOID KiReleaseApcLockFromSynchLevel(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiReleaseApcLockFromSynchLevel(Handle);
}

VOID KiReleaseProcessLockFromSynchLevel(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _rs_KiReleaseProcessLockFromSynchLevel(Handle);
}

VOID KiAcquireDeviceQueueLock(IN PKDEVICE_QUEUE DeviceQueue, IN PKLOCK_QUEUE_HANDLE DeviceLock)
{
    _rs_KiAcquireDeviceQueueLock(DeviceQueue, DeviceLock);
}

VOID KiReleaseDeviceQueueLock(IN PKLOCK_QUEUE_HANDLE DeviceLock)
{
    _rs_KiReleaseDeviceQueueLock(DeviceLock);
}

/* --- Timer/wait helpers (ke_x.h) ---
 * Rust impls live in sdk/lib/rust/ke-inlines/src/lib.rs (Phase 1 batch 7). */
extern PLARGE_INTEGER _rs_KiRecalculateDueTime(PLARGE_INTEGER OriginalDueTime,
                                               PLARGE_INTEGER DueTime,
                                               PLARGE_INTEGER NewDueTime);
extern VOID    _rs_KxInsertTimer(PKTIMER Timer, ULONG Hand);
extern BOOLEAN _rs_KiComputeDueTime(PKTIMER Timer, LARGE_INTEGER DueTime, PULONG Hand);
extern VOID    _rs_KxRemoveTreeTimer(PKTIMER Timer);
extern VOID    _rs_KxSetTimerForThreadWait(PKTIMER Timer, LARGE_INTEGER Interval, PULONG Hand);

PLARGE_INTEGER KiRecalculateDueTime(IN PLARGE_INTEGER OriginalDueTime,
                                     IN PLARGE_INTEGER DueTime,
                                     IN OUT PLARGE_INTEGER NewDueTime)
{
    return _rs_KiRecalculateDueTime(OriginalDueTime, DueTime, NewDueTime);
}

/* --- Stack-swap eligibility + alertability dispatch (ke_x.h) ---
 * Phase 1 batch 9 — bodies in ke-inlines.
 */

extern BOOLEAN  _rs_KiCheckThreadStackSwap(PKTHREAD Thread, KPROCESSOR_MODE WaitMode);
extern NTSTATUS _rs_KiCheckAlertability(PKTHREAD Thread, BOOLEAN Alertable, KPROCESSOR_MODE WaitMode);

BOOLEAN KiCheckThreadStackSwap(IN PKTHREAD Thread, IN KPROCESSOR_MODE WaitMode)
{
    return _rs_KiCheckThreadStackSwap(Thread, WaitMode);
}

NTSTATUS KiCheckAlertability(IN PKTHREAD Thread, IN BOOLEAN Alertable, IN KPROCESSOR_MODE WaitMode)
{
    return _rs_KiCheckAlertability(Thread, Alertable, WaitMode);
}

ULONG KiComputeTimerTableIndex(IN ULONGLONG DueTime)
{
    return _rs_KiComputeTimerTableIndex(DueTime);
}

VOID KiRemoveEntryTimer(IN PKTIMER Timer)
{
    _saved_KiRemoveEntryTimer(Timer);
}

VOID KxInsertTimer(IN PKTIMER Timer, IN ULONG Hand)
{
    _rs_KxInsertTimer(Timer, Hand);
}

BOOLEAN KiComputeDueTime(IN PKTIMER Timer, IN LARGE_INTEGER DueTime, OUT PULONG Hand)
{
    return _rs_KiComputeDueTime(Timer, DueTime, Hand);
}

VOID KxRemoveTreeTimer(IN PKTIMER Timer)
{
    _rs_KxRemoveTreeTimer(Timer);
}

VOID KxSetTimerForThreadWait(IN PKTIMER Timer, IN LARGE_INTEGER Interval, OUT PULONG Hand)
{
    _rs_KxSetTimerForThreadWait(Timer, Interval, Hand);
}

/* --- Unwait + queue-ready (ke_x.h) ---
 * Rust impls live in sdk/lib/rust/ke-inlines/src/lib.rs (Phase 1 batch 9).
 * The Rust port preserves the defensive WaitThread null-check that was added
 * to the original FORCEINLINE to handle corrupt WaitBlocks. */
extern VOID _rs_KxUnwaitThread(DISPATCHER_HEADER *Object, KPRIORITY Increment);
extern VOID _rs_KxUnwaitThreadForEvent(PKEVENT Event, KPRIORITY Increment);
extern VOID _rs_KxQueueReadyThread(PKTHREAD Thread, PKPRCB Prcb);

VOID KxUnwaitThread(IN DISPATCHER_HEADER *Object, IN KPRIORITY Increment)
{
    _rs_KxUnwaitThread(Object, Increment);
}

VOID KxUnwaitThreadForEvent(IN PKEVENT Event, IN KPRIORITY Increment)
{
    _rs_KxUnwaitThreadForEvent(Event, Increment);
}

VOID KxQueueReadyThread(IN PKTHREAD Thread, IN PKPRCB Prcb)
{
    _rs_KxQueueReadyThread(Thread, Prcb);
}

/* Rust impls for KiSelectReadyThread + KiComputeNewPriority live in
 * sdk/lib/rust/ke-inlines/src/lib.rs — Phase 1 batch 10. */
extern PKTHREAD _rs_KiSelectReadyThread(KPRIORITY Priority, PKPRCB Prcb);
extern SCHAR _rs_KiComputeNewPriority(PKTHREAD Thread, SCHAR Adjustment);

PKTHREAD KiSelectReadyThread(IN KPRIORITY Priority, IN PKPRCB Prcb)
{
    return _rs_KiSelectReadyThread(Priority, Prcb);
}

SCHAR KiComputeNewPriority(IN PKTHREAD Thread, IN SCHAR Adjustment)
{
    return _rs_KiComputeNewPriority(Thread, Adjustment);
}

/* --- Guarded mutex (ke_x.h) ---
 * Rust impls live in sdk/lib/rust/ke-inlines/src/lib.rs (Phase 1 batch 8). */
extern VOID _rs__KeInitializeGuardedMutex(PKGUARDED_MUTEX GuardedMutex);
extern VOID _rs__KeAcquireGuardedMutexUnsafe(PKGUARDED_MUTEX GuardedMutex);
extern VOID _rs__KeReleaseGuardedMutexUnsafe(PKGUARDED_MUTEX GuardedMutex);
extern VOID _rs__KeAcquireGuardedMutex(PKGUARDED_MUTEX GuardedMutex);
extern VOID _rs__KeReleaseGuardedMutex(PKGUARDED_MUTEX GuardedMutex);
extern BOOLEAN _rs__KeTryToAcquireGuardedMutex(PKGUARDED_MUTEX GuardedMutex);

VOID _KeInitializeGuardedMutex(OUT PKGUARDED_MUTEX GuardedMutex)
{
    _rs__KeInitializeGuardedMutex(GuardedMutex);
}

VOID _KeAcquireGuardedMutexUnsafe(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _rs__KeAcquireGuardedMutexUnsafe(GuardedMutex);
}

VOID _KeReleaseGuardedMutexUnsafe(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _rs__KeReleaseGuardedMutexUnsafe(GuardedMutex);
}

VOID _KeAcquireGuardedMutex(IN PKGUARDED_MUTEX GuardedMutex)
{
    _rs__KeAcquireGuardedMutex(GuardedMutex);
}

VOID _KeReleaseGuardedMutex(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _rs__KeReleaseGuardedMutex(GuardedMutex);
}

BOOLEAN _KeTryToAcquireGuardedMutex(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    return _rs__KeTryToAcquireGuardedMutex(GuardedMutex);
}

/* --- NMI list lock (ke_x.h) ---
 * Phase 1 batch 8 — bodies in ke-inlines (KeAcquire/ReleaseSpinLock arch
 * dispatch over the global KiNmiCallbackListLock).
 */

extern void _rs_KiAcquireNmiListLock(PKIRQL OldIrql);
extern void _rs_KiReleaseNmiListLock(KIRQL OldIrql);

VOID KiAcquireNmiListLock(OUT PKIRQL OldIrql)
{
    _rs_KiAcquireNmiListLock(OldIrql);
}

VOID KiReleaseNmiListLock(IN KIRQL OldIrql)
{
    _rs_KiReleaseNmiListLock(OldIrql);
}

/* KeRaiseIrql/KeLowerIrql — already provided by tier1_rust_shims.c (i386)
 * and ps_rust_shims.c (amd64/arm64). Do not duplicate here. */

/* KeRaiseIrqlToSynchLevel shim.
 * On amd64 it's FORCEINLINE (body = KfRaiseIrql(SYNCH_LEVEL)), so the
 * inline-trampoline pattern works — the body is inlined, no recursion.
 * On i386 it's NTHALAPI (dllimport from HAL), NOT a macro/FORCEINLINE.
 * Cannot use trampoline, function pointer, or any indirect pattern because
 * the linker/compiler always resolves to the local definition over dllimport
 * when both are in the same TU. Instead, directly call through the IAT. */
#if defined(_M_IX86)
KIRQL NTAPI KeRaiseIrqlToSynchLevel(VOID)
{
    KIRQL result;
    __asm__ __volatile__(
        "call *__imp__KeRaiseIrqlToSynchLevel@0"
        : "=a"(result)
        :
        : "ecx", "edx", "memory");
    return result;
}
#elif defined(_M_AMD64)
/* On amd64, KeRaiseIrqlToSynchLevel is FORCEINLINE (reads/writes CR8).
 * The trampoline pattern doesn't work because the compiler may not inline the
 * FORCEINLINE body, creating infinite recursion optimized to jmp $ (eb fe).
 * Directly implement using CR8 intrinsics instead.
 * Without CONFIG_SMP, SYNCH_LEVEL = DISPATCH_LEVEL = 2. */
#undef KeRaiseIrqlToSynchLevel
KIRQL KeRaiseIrqlToSynchLevel(VOID)
{
    unsigned long long cr8;
    __asm__ __volatile__("mov %%cr8, %0" : "=r"(cr8));
#ifdef CONFIG_SMP
    { unsigned long long new_irql = 12; /* SYNCH_LEVEL = IPI_LEVEL - 2 */
    __asm__ __volatile__("mov %0, %%cr8" : : "r"(new_irql)); }
#else
    { unsigned long long new_irql = DISPATCH_LEVEL;
    __asm__ __volatile__("mov %0, %%cr8" : : "r"(new_irql)); }
#endif
    return (KIRQL)cr8;
}
#endif

/* KeQuerySystemTime is already provided by ps_rust_shims.c — do not duplicate here. */

/* KiCpuId lives in Rust now — sdk/lib/rust/ntkrnl-sys/src/inline/ke_i386.rs
 * (i386 NTAPI/stdcall) and ke_amd64.rs (Microsoft x64 ABI). */

/* --- MSR read/write (x86/amd64 only) ---
 * Rust code calls these as extern "C" functions. On x86/amd64, the compiler
 * provides __readmsr/__writemsr as intrinsics, but Rust needs linker symbols.
 * On amd64, Phase 1 renamed __readmsr → _saved___readmsr, etc. */
#if defined(_M_IX86)

ULONGLONG _rdmsr(ULONG Register)
{
    return __readmsr(Register);
}

VOID _wrmsr(ULONG Register, ULONGLONG Value)
{
    __writemsr(Register, Value);
}

#endif /* _M_IX86 */

/* InterlockedBitTestAndSet / InterlockedBitTestAndReset live in Rust
 * inline/ke.rs (with shim_exports providing the linker symbols). */

/* KeEnterCriticalRegion / KeLeaveCriticalRegion are already provided by
 * ob_rust_shims.c — do not duplicate here. */

/* KeEnterGuardedRegion / KeLeaveGuardedRegion are already provided by
 * ps_rust_shims.c — do not duplicate here. */

/* ========================================================================
 * FASTCALL wrappers for i386
 *
 * On i386, Rust can't directly export fastcall symbols. We provide C wrappers
 * that call the Rust cdecl implementations.
 *
 * The naming convention is: Rust exports _KiName_cdecl (cdecl), and this
 * file wraps it with FASTCALL KiName.
 *
 * NOTE: Many KE spinlock functions are FASTCALL on i386. The Rust translations
 * should use the fastcall_export! macro pattern from EX subsystem.
 * ======================================================================== */

/* KeGetPcr (i386) lives in Rust now — global_asm! in ntkrnl-sys/inline/ke_i386.rs
 * (single `mov eax, fs:[0x1C]; ret`). On amd64/arm64 it's a real function
 * already provided by bindgen / shim_exports.rs. */

/* --- Function-like macro shims ---
 * These are #define macros in ke_x.h. Phase 2 (sibling rlib `ke-macros`)
 * provides the bodies; this file keeps the C symbols alive as ABI anchors
 * that delegate to the Rust impls. Same augment shape as the Phase 1
 * `_rs_*` pattern (cm-inlines / ke-inlines etc.). #undef the macros so the
 * function definitions below don't get rewritten by them. */
#undef KiSatisfyObjectWait
#undef KiSatisfyMutantWait
#undef KiSatisfyNonMutantWait
#undef KiAddThreadToWaitList

extern void _rs_KiSatisfyObjectWait(PKMUTANT Object, PKTHREAD Thread);
extern void _rs_KiSatisfyMutantWait(PKMUTANT Object, PKTHREAD Thread);
extern void _rs_KiSatisfyNonMutantWait(PKMUTANT Object);
extern void _rs_KiAddThreadToWaitList(PKTHREAD Thread, BOOLEAN Swappable);

void KiSatisfyObjectWait(PKMUTANT Object, PKTHREAD Thread)
{
    _rs_KiSatisfyObjectWait(Object, Thread);
}

void KiSatisfyMutantWait(PKMUTANT Object, PKTHREAD Thread)
{
    _rs_KiSatisfyMutantWait(Object, Thread);
}

void KiSatisfyNonMutantWait(PKMUTANT Object)
{
    _rs_KiSatisfyNonMutantWait(Object);
}

void KiAddThreadToWaitList(PKTHREAD Thread, BOOLEAN Swappable)
{
    _rs_KiAddThreadToWaitList(Thread, Swappable);
}

/* ProbeForReadLargeInteger is provided by se_rust_shims.c — not duplicated here */

/* `Kx{Single,Delay,Multi}ThreadWait` — Phase 2 wait-setup batch, bodies in
 * sibling rlib `ke-macros`. Trampolines stay as ABI anchors that delegate to
 * the Rust impls. The 2026-05-05 wait-block-corruption fix lives in the
 * Rust side now (KxMultiThreadWait calls InitializeListHead instead of the
 * half-init Flink/Blink pre-set; KxSingle/KxDelay keep the defensive full-
 * init shape). #undef the macros so the function definitions don't get
 * rewritten by them. */
#undef KxSingleThreadWait
#undef KxDelayThreadWait
#undef KxMultiThreadWait

extern void _rs_KxSingleThreadWait(
    PVOID Object, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    UCHAR WaitReason, BOOLEAN Alertable,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand);
extern void _rs_KxDelayThreadWait(
    PLARGE_INTEGER Interval, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand);
extern void _rs_KxMultiThreadWait(
    PKWAIT_BLOCK WaitBlockArray, PVOID *Objects, ULONG Count,
    PLARGE_INTEGER Timeout, UCHAR WaitType,
    KPROCESSOR_MODE WaitMode, UCHAR WaitReason,
    BOOLEAN Alertable, BOOLEAN *Swappable,
    PLARGE_INTEGER DueTime, PULONG Hand);

void KxSingleThreadWait(
    PVOID Object, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    KWAIT_REASON WaitReason, BOOLEAN Alertable,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    _rs_KxSingleThreadWait(Object, Timeout, WaitMode, (UCHAR)WaitReason,
                           Alertable, Swappable, DueTime, Hand);
}

void KxDelayThreadWait(
    PLARGE_INTEGER Interval, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    _rs_KxDelayThreadWait(Interval, WaitMode, Swappable, DueTime, Hand);
}

void KxMultiThreadWait(
    PKWAIT_BLOCK WaitBlockArray, PVOID *Objects, ULONG Count,
    PLARGE_INTEGER Timeout, WAIT_TYPE WaitType,
    KPROCESSOR_MODE WaitMode, KWAIT_REASON WaitReason,
    BOOLEAN Alertable, BOOLEAN *Swappable,
    PLARGE_INTEGER DueTime, PULONG Hand)
{
    _rs_KxMultiThreadWait(WaitBlockArray, Objects, Count, Timeout,
                          (UCHAR)WaitType, WaitMode, (UCHAR)WaitReason,
                          Alertable, Swappable, DueTime, Hand);
}

/* `YieldProcessor`, `KeGetTrapFramePc`, `KiRundownThread`,
 * `KiGetLinkedTrapFrame`, `KiGetUserModeStackAddress` — Phase 2 trap-frame /
 * intrinsic batch, bodies in sibling rlib `ke-macros`. Trampolines stay as
 * ABI anchors that delegate to the Rust impls. KiSystemStartup for ARM64 is
 * provided by Rust (arm64/kiinit.rs). */
#undef YieldProcessor
#undef KeGetTrapFramePc
#undef KiGetLinkedTrapFrame

extern void _rs_YieldProcessor(void);
extern ULONG_PTR _rs_KeGetTrapFramePc(PKTRAP_FRAME TrapFrame);
extern VOID _rs_KiRundownThread(PKTHREAD Thread);
extern PKTRAP_FRAME _rs_KiGetLinkedTrapFrame(PKTRAP_FRAME TrapFrame);
extern PULONG_PTR _rs_KiGetUserModeStackAddress(void);

void YieldProcessor(void)
{
    _rs_YieldProcessor();
}

ULONG_PTR KeGetTrapFramePc(IN PKTRAP_FRAME TrapFrame)
{
    return _rs_KeGetTrapFramePc(TrapFrame);
}

VOID KiRundownThread(IN PKTHREAD Thread)
{
    _rs_KiRundownThread(Thread);
}

PKTRAP_FRAME KiGetLinkedTrapFrame(IN PKTRAP_FRAME TrapFrame)
{
    return _rs_KiGetLinkedTrapFrame(TrapFrame);
}

PULONG_PTR KiGetUserModeStackAddress(void)
{
    return _rs_KiGetUserModeStackAddress();
}

/* --- KxQueueThreadWait ---
 * Phase 2 wait-setup batch — body in `ke-macros::_rs_KxQueueThreadWait`. */
#undef KxQueueThreadWait
extern void _rs_KxQueueThreadWait(
    PVOID Queue, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand);

void KxQueueThreadWait(
    PVOID Queue, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    _rs_KxQueueThreadWait(Queue, Timeout, WaitMode, Swappable, DueTime, Hand);
}

/* --- KxUnwaitThread_impl ---
 * Rust thrdobj.rs references this name to avoid collision with the ddk import.
 * Just forward to the existing KxUnwaitThread shim. */
void KxUnwaitThread_impl(DISPATCHER_HEADER *Header, KPRIORITY Increment)
{
    KxUnwaitThread(Header, Increment);
}

/* --- HalQuerySystemInformation / HalSetSystemInformation ---
 * Phase 2 HAL-dispatch batch — bodies in `ke-macros::_rs_HalQuery/Set*`.
 * These are NTAPI (stdcall on i386), so the trampoline keeps the NTAPI
 * decoration; the Rust impl is `extern "C"` (cdecl) and the trampoline
 * bridges. */
#undef HalQuerySystemInformation
#undef HalSetSystemInformation

extern NTSTATUS _rs_HalQuerySystemInformation(
    HAL_QUERY_INFORMATION_CLASS InformationClass, ULONG BufferSize,
    PVOID Buffer, PULONG ReturnedLength);
extern NTSTATUS _rs_HalSetSystemInformation(
    HAL_SET_INFORMATION_CLASS InformationClass, ULONG BufferSize, PVOID Buffer);

NTSTATUS NTAPI HalQuerySystemInformation(
    IN HAL_QUERY_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN OUT PVOID Buffer,
    OUT PULONG ReturnedLength)
{
    return _rs_HalQuerySystemInformation(InformationClass, BufferSize, Buffer, ReturnedLength);
}

NTSTATUS NTAPI HalSetSystemInformation(
    IN HAL_SET_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN PVOID Buffer)
{
    return _rs_HalSetSystemInformation(InformationClass, BufferSize, Buffer);
}

/* --- KDBG stubs for ARM64 ---
 * KdbpPrint is variadic so it must be a C stub (Rust can't export variadic functions).
 * Other KDBG stubs (KdbInitialize, KdbpGetHexNumber, etc.) are in arm64/stubs.rs. */
#ifdef _M_ARM64
#ifdef KDBG
VOID KdbpPrint(_In_ PSTR Format, ...)
{
    /* Nothing to do */
}
#endif
#endif

/* ========================================================================
 * SSDT tables — MainSSDT[] and MainSSPT[]
 *
 * These are generated by the SVC_ macro in <internal/napi.h> which
 * #include's sysfuncs.h.  Originally lived in krnlinit.c (now Rust).
 * The Rust code references them as extern symbols.
 * ======================================================================== */
#include <internal/napi.h>

/* ========================================================================
 * KiAttachProcess — originally in procobj.c (now Rust)
 *
 * The Rust procobj.rs imports this via FFI.  It calls several FORCEINLINE
 * helpers that are already shimmed elsewhere.
 * ======================================================================== */
VOID
NTAPI
KiAttachProcess(IN PKTHREAD Thread,
                IN PKPROCESS Process,
                IN PKLOCK_QUEUE_HANDLE ApcLock,
                IN PRKAPC_STATE SavedApcState)
{
    ASSERT(Process != Thread->ApcState.Process);

    /* Increase Stack Count */
    ASSERT(Process->StackCount != MAXULONG_PTR);
    Process->StackCount++;

    /* Swap the APC Environment */
    KiMoveApcState(&Thread->ApcState, SavedApcState);

    /* Reinitialize Apc State */
    InitializeListHead(&Thread->ApcState.ApcListHead[KernelMode]);
    InitializeListHead(&Thread->ApcState.ApcListHead[UserMode]);
    Thread->ApcState.Process = Process;
    Thread->ApcState.KernelApcInProgress = FALSE;
    Thread->ApcState.KernelApcPending = FALSE;
    Thread->ApcState.UserApcPending = FALSE;

    /* Update Environment Pointers if needed */
    if (SavedApcState == &Thread->SavedApcState)
    {
        Thread->ApcStatePointer[OriginalApcEnvironment] = &Thread->
                                                          SavedApcState;
        Thread->ApcStatePointer[AttachedApcEnvironment] = &Thread->ApcState;
        Thread->ApcStateIndex = AttachedApcEnvironment;
    }

    /* Check if the process is paged in */
    if (Process->State == ProcessInMemory)
    {
        /* Release dispatcher lock */
        _saved_KiReleaseDispatcherLockFromSynchLevel();

        /* Release lock */
        _saved_KiReleaseApcLockFromSynchLevel(ApcLock);

        /* Swap Processes */
        KiSwapProcess(Process, SavedApcState->Process);

        /* Exit the dispatcher */
        KiExitDispatcher(ApcLock->OldIrql);
    }
    else
    {
        DPRINT1("Errr. ReactOS doesn't support paging out processes yet...\n");
        ASSERT(FALSE);
    }
}

/* amd64 compiler intrinsic wrappers (__readcr*, __writecr*, __readdr,
 * __writedr, __readmsr, __writemsr, __lidt, __lgdt, __sidt, __sgdt, __sldt,
 * __str, __wbinvd, __swapgs, KeMemoryBarrier, __cpuid, __cpuidex,
 * _mm_getcsr/setcsr, RtlFillMemoryUlong, KiSendEOI, KiGetGdtEntry,
 * KiGetGdtDescriptorBase, KiInitGdtEntry, _enable, _disable, __debugbreak,
 * __addgsdword, _xsetbv, InterlockedBitTestAndSet64, InterlockedAnd64,
 * __rdtsc, KeQueryInterruptHandler, KeRegisterInterruptHandler) all live in
 * Rust now (inline/ke_amd64.rs + shim_exports.rs / global_asm!). */

/* KiSaveXState / KiRestoreXState — force fxsave64/fxrstor64 to avoid
 * STATUS_IN_PAGE_ERROR crash with xsaves64/xrstors64 on KVM.
 * KVM reports KF_XSAVES via CPUID, but xrstors64 faults on StateSaveArea
 * pages that appear valid via PTE self-referencing mapping.
 * Root cause is likely IA32_XSS MSR not being configured, or a KVM
 * interaction with supervisor-mode XSAVE.
 * fxsave64/fxrstor64 preserves x87+SSE state which is sufficient. */
#ifdef _M_AMD64
VOID KiSaveXState(PVOID Buffer, ULONG64 ComponentMask)
{
    _fxsave64(Buffer);
}

VOID KiRestoreXState(PVOID Buffer, ULONG64 ComponentMask)
{
    _fxrstor64(Buffer);
}
#endif /* _M_AMD64 */

/* ========================================================================
 * i386-specific shims
 * ======================================================================== */
#ifdef _M_IX86

/* --- KeWin32PreServiceHook / KeWin32PostServiceHook ---
 * Global function pointers used by kd64/kdapi.c for Win32k service hooks.
 * Originally defined in i386/traphdlr.c (now Rust). Types declared in kd.h. */
#if DBG && !defined(_WINKD_)
PKDBG_PRESERVICEHOOK KeWin32PreServiceHook = NULL;
PKDBG_POSTSERVICEHOOK KeWin32PostServiceHook = NULL;
#endif

/* --- i386 FORCEINLINE function shims (Phase 4) --- */

/* Ke386GetTr, Ke386SanitizeSeg, Ke386SanitizeFlags, KeInvalidateTlbEntry
 * live in Rust now (sdk/lib/rust/ntkrnl-sys/src/inline/ke_i386.rs).
 * KiFlushNPXState is in Rust cpu.rs. */

/* Ke386GetGlobalDescriptorTable and Ke386GetLocalDescriptorTable are
 * macros (#define to __sgdt / __sldt). Capture via trampoline. */
static __inline void _tramp_Ke386GetGlobalDescriptorTable(PVOID Limit)
{
    Ke386GetGlobalDescriptorTable(Limit);
}
static __inline void _tramp_Ke386GetLocalDescriptorTable(PUSHORT Selector)
{
    Ke386GetLocalDescriptorTable(Selector);
}
#undef Ke386GetGlobalDescriptorTable
#undef Ke386GetLocalDescriptorTable
VOID NTAPI Ke386GetGlobalDescriptorTable(PVOID Limit)
{
    _tramp_Ke386GetGlobalDescriptorTable(Limit);
}
VOID NTAPI Ke386GetLocalDescriptorTable(PUSHORT Selector)
{
    _tramp_Ke386GetLocalDescriptorTable(Selector);
}

/* --- HalVectorToIDTEntry ---
 * Phase 2 HAL-dispatch batch — body in `ke-macros::_rs_HalVectorToIDTEntry`. */
#undef HalVectorToIDTEntry
extern UCHAR _rs_HalVectorToIDTEntry(ULONG Vector);
UCHAR NTAPI HalVectorToIDTEntry(IN ULONG Vector)
{
    return _rs_HalVectorToIDTEntry(Vector);
}

/* KeQueryInterruptHandler / KeRegisterInterruptHandler live in Rust now —
 * ntkrnl-sys/inline/ke_i386.rs (PCR.IDT entry mux). */

/* --- KfRaiseIrql ---
 * FASTCALL HAL function on i386. Rust declares it as extern "system" (stdcall)
 * in ke_defs.rs, producing _KfRaiseIrql@4. But HAL exports @KfRaiseIrql@4
 * (fastcall). We provide a stdcall wrapper named KfRaiseIrql_stdcall that
 * calls the real fastcall HAL import. Then rename Rust's extern to match. */
/* Actually, simpler: just change the Rust declaration to not use KfRaiseIrql
 * directly but use the KeRaiseIrql wrapper we already provide. */
/* KfRaiseIrql stdcall→fastcall trampoline: now in Rust ke_i386.rs global_asm! */

/* --- KxUnwaitThread ---
 * Already defined above in generic section, but the generic definition
 * lacks NTAPI. The NTAPI version is needed for i386 stdcall. Since the
 * generic version was already fixed to add NTAPI, this is a no-op.
 * If still missing, verify the generic definition above has NTAPI. */

/* --- KiThreadStartup ---
 * Thread startup function on i386. Used as a return address from context switch.
 * Originally in thrdini.c (now Rust), but Rust thrdini.rs only declares it as
 * extern. We provide the C implementation here.
 *
 * This function is entered via context switch "ret" — the context switch
 * frame has RetAddr = KiThreadStartup. On entry, the kernel stack contains
 * KUINIT_FRAME at KernelStack. */
DECLSPEC_NORETURN VOID FASTCALL KiServiceExit2(IN PKTRAP_FRAME TrapFrame);

typedef struct _KSWITCHFRAME_SHIM {
    PVOID ExceptionList;
    BOOLEAN ApcBypassDisable;
    PVOID RetAddr;
} KSWITCHFRAME_SHIM;

typedef struct _KSTART_FRAME_SHIM {
    PKSYSTEM_ROUTINE SystemRoutine;
    PKSTART_ROUTINE StartRoutine;
    PVOID StartContext;
    BOOLEAN UserThread;
} KSTART_FRAME_SHIM;

typedef struct _KUINIT_FRAME_SHIM {
    KSWITCHFRAME_SHIM CtxSwitchFrame;
    KSTART_FRAME_SHIM StartFrame;
    KTRAP_FRAME TrapFrame;
    FX_SAVE_AREA FxSaveArea;
} KUINIT_FRAME_SHIM;

VOID
NTAPI
KiThreadStartup(VOID)
{
    PKTRAP_FRAME TrapFrame;
    KSTART_FRAME_SHIM *StartFrame;
    KUINIT_FRAME_SHIM *InitFrame;

    InitFrame = (KUINIT_FRAME_SHIM *)KeGetCurrentThread()->KernelStack;
    StartFrame = &InitFrame->StartFrame;
    TrapFrame = &InitFrame->TrapFrame;

    KeLowerIrql(APC_LEVEL);

    StartFrame->SystemRoutine(StartFrame->StartRoutine, StartFrame->StartContext);

    if (!StartFrame->UserThread)
    {
        KeBugCheck(NO_USER_MODE_CONTEXT);
    }

    KiServiceExit2(TrapFrame);
}

/* Ke386FnInit, SaveFpuState, RestoreFpuState, ClearFpExceptions:
 * Now in Rust ke_i386.rs global_asm! */

/* --- KiGetThreadNpxArea --- FORCEINLINE in internal/i386/ke.h */
/* KiGetThreadNpxArea lives in Rust now (ntkrnl-sys/inline/ke_i386.rs). */

/* --- KiComputeIopmOffset --- macro in ndk/i386/ketypes.h */
static __inline USHORT _tramp_KiComputeIopmOffset(USHORT MapNumber)
{
    return (USHORT)KiComputeIopmOffset(MapNumber);
}
#undef KiComputeIopmOffset
USHORT NTAPI KiComputeIopmOffset(USHORT MapNumber)
{
    return _tramp_KiComputeIopmOffset(MapNumber);
}

/* KfLowerIrql, __rdtsc, __readeflags, _disable, _enable, __writeeflags,
 * __debugbreak, __readcr0-4, __writecr0: now in Rust ke_i386.rs global_asm! */
/* __writecr3, __sgdt, __sidt, __invlpg, __readfsdword, __writefsdword:
 * Now in Rust ke_i386.rs global_asm! */

/* --- KiCheckForApcDelivery --- FORCEINLINE in internal/i386/ke.h */
VOID NTAPI KiCheckForApcDelivery(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiCheckForApcDelivery(TrapFrame);
}

/* KiGetGdtEntry / KiSetGdtDescriptorBase live in Rust now —
 * sdk/lib/rust/ntkrnl-sys/src/inline/ke_i386.rs (the inline impls), with
 * #[no_mangle] export wrappers in sdk/lib/rust/ntoskrnl-rust/src/shim_exports.rs
 * (mod ke_i386_exports). */

/* InterlockedBitTestAndSet@8, InterlockedBitTestAndReset@8, KeMemoryBarrier:
 * Now in Rust ke_i386.rs global_asm! + shim_exports.rs */

/* --- Ki386InitializeTss and helpers ---
 * Regular FASTCALL function from cpu.c. Calls many FORCEINLINE/macros
 * (KiInitializeTSS, Ke386SetTr, Ke386GetSs etc.) so kept in C.
 * KiInitializeTSS/TSS2 were also in cpu.c (now Rust). */
extern ULONG_PTR KiDoubleFaultTSS;
extern ULONG_PTR KiNMITSS;
extern ULONG KiDoubleFaultStack;

CODE_SEG("INIT")
static VOID
NTAPI
Shim_KiInitializeTSS2(IN PKTSS Tss, IN PKGDTENTRY TssEntry OPTIONAL)
{
    PUCHAR p;

    if (TssEntry) {
        TssEntry->LimitLow = sizeof(KTSS) - 1;
        TssEntry->HighWord.Bits.LimitHi = 0;
    }

    ASSERT(IOPM_COUNT == 1);
    RtlFillMemory(Tss->IoMaps[0].IoMap, IOPM_FULL_SIZE, 0xFF);

    p = (PUCHAR)(Tss->IoMaps[0].DirectionMap);
    RtlZeroMemory(p, IOPM_DIRECTION_MAP_SIZE);
    p[0] = 4; p[3] = 0x18; p[4] = 0x18;

    p = Tss->IntDirectionMap;
    RtlZeroMemory(Tss->IntDirectionMap, IOPM_DIRECTION_MAP_SIZE);
    p[0] = 4; p[3] = 0x18; p[4] = 0x18;
}

CODE_SEG("INIT")
static VOID
NTAPI
Shim_KiInitializeTSS(IN PKTSS Tss)
{
    Tss->IoMapBase = _tramp_KiComputeIopmOffset(IO_ACCESS_MAP_NONE);
    Tss->Flags = 0;
    Tss->LDT = 0;
    Tss->Ss0 = KGDT_R0_DATA;
}

CODE_SEG("INIT")
VOID
FASTCALL
Ki386InitializeTss(IN PKTSS Tss,
                   IN PKIDTENTRY Idt,
                   IN PKGDTENTRY Gdt)
{
    PKGDTENTRY TssEntry, TaskGateEntry;

    TssEntry = &Gdt[KGDT_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;

    Shim_KiInitializeTSS2(Tss, TssEntry);

    Shim_KiInitializeTSS(Tss);

    __asm__ __volatile__("ltr %%ax" : :"a" ((USHORT)KGDT_TSS));

    TaskGateEntry = (PKGDTENTRY)&Idt[8];
    TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
    TaskGateEntry->HighWord.Bits.Pres = 1;
    TaskGateEntry->HighWord.Bits.Dpl = 0;
    ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_DF_TSS;

    Tss = (PKTSS)KiDoubleFaultTSS;
    Shim_KiInitializeTSS(Tss);
    Tss->CR3 = __readcr3();
    Tss->Esp0 = KiDoubleFaultStack;
    Tss->Esp = KiDoubleFaultStack;
    Tss->Eip = PtrToUlong(KiTrap08);
    Tss->Cs = KGDT_R0_CODE;
    Tss->Fs = KGDT_R0_PCR;
    { unsigned int __ss; __asm__("movl %%ss,%0" :"=r"(__ss)); Tss->Ss = (USHORT)__ss; }
    Tss->Es = KGDT_R3_DATA | RPL_MASK;
    Tss->Ds = KGDT_R3_DATA | RPL_MASK;

    TssEntry = &Gdt[KGDT_DF_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;
    TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
    TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
    TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
    TssEntry->LimitLow = KTSS_IO_MAPS;

    TaskGateEntry = (PKGDTENTRY)&Idt[2];
    TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
    TaskGateEntry->HighWord.Bits.Pres = 1;
    TaskGateEntry->HighWord.Bits.Dpl = 0;
    ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_NMI_TSS;

    Tss = (PKTSS)KiNMITSS;
    Shim_KiInitializeTSS(Tss);
    Tss->CR3 = __readcr3();
    Tss->Esp0 = KiDoubleFaultStack;
    Tss->Esp = KiDoubleFaultStack;
    Tss->Eip = PtrToUlong(KiTrap02);
    Tss->Cs = KGDT_R0_CODE;
    Tss->Fs = KGDT_R0_PCR;
    { unsigned int __ss; __asm__("movl %%ss,%0" :"=r"(__ss)); Tss->Ss = (USHORT)__ss; }
    Tss->Es = KGDT_R3_DATA | RPL_MASK;
    Tss->Ds = KGDT_R3_DATA | RPL_MASK;

    TssEntry = &Gdt[KGDT_NMI_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;
    TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
    TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
    TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
    TssEntry->LimitLow = KTSS_IO_MAPS;

}

/* KiEnterTrap lives in Rust now (ntkrnl-sys/inline/ke_i386.rs global_asm!). */

/* KiEnterInterruptTrap lives in Rust now (ntkrnl-sys/inline/ke_i386.rs). */

/* --- KiEndInterrupt ---
 * FORCEINLINE in ke.h. Ends interrupt via HAL and exits trap.
 * Declared as extern "C" in Rust time.rs → _KiEndInterrupt */
void __cdecl KiEndInterrupt(IN KIRQL Irql, IN PKTRAP_FRAME TrapFrame)
{
    _disable();
    HalEndSystemInterrupt(Irql, TrapFrame);
    KiEoiHelper(TrapFrame);
}

/* KiDispatchException{0,1,2}Args + KiSwitchToBootStack live in Rust now
 * (ntkrnl-sys/inline/ke_i386.rs global_asm!). Each is DECLSPEC_NORETURN
 * and just delegates to KiDispatchExceptionFromTrapFrame /
 * KiSystemStartupBootStack. */

/* Ke386FnSave / Ke386FxSave / Ke386FxStore (x87 + SSE state save/restore),
 * Ke386SetCr2, Ke386Set{Ds,Es,Fs,Gs}, Ke386SetGlobalDescriptorTable,
 * Ke386SetLocalDescriptorTable, Ke386SetTr live in Rust now
 * (sdk/lib/rust/ntkrnl-sys/src/inline/ke_i386.rs global_asm! block).
 * Ke386GetFs has no surviving callers — dropped. */

/* KiEnterV86Trap lives in Rust now (ntkrnl-sys/inline/ke_i386.rs). */

/* --- KiExitV86Trap ---
 * FORCEINLINE DECLSPEC_NORETURN in trap_x.h. Exits V86 mode trap.
 * Declared as extern "fastcall" in Rust traphdlr.rs → @KiExitV86Trap@4 */
DECLSPEC_NORETURN
void FASTCALL KiExitV86Trap(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiExitV86Trap(TrapFrame);
}

/* KiFillTrapFrameDebug lives in Rust now (ntkrnl-sys/inline/ke_i386.rs). */

/* KiExceptionExit lives in Rust now (ntkrnl-sys/inline/ke_i386.rs).
 * On i386 ExceptionFrame is unused; body just tail-calls KiServiceExit2. */

/* KiHandleDebugRegistersOnTrap{Entry,Exit} live in Rust now —
 * pure global_asm! in ntkrnl-sys/inline/ke_i386.rs with hardcoded
 * KTRAP_FRAME / PRCB / KSPECIAL_REGISTERS offsets guarded by
 * compile-time offset_of! asserts. */

/* --- KiExitTrapDebugChecks ---
 * FORCEINLINE in trap_x.h: debug checks on trap exit (IRQL, APC state).
 * Declared as extern "system" in Rust traphdlr.rs → _KiExitTrapDebugChecks@8 */
void NTAPI KiExitTrapDebugChecks(IN PKTRAP_FRAME TrapFrame, IN BOOLEAN SkipPreviousMode)
{
    _saved_KiExitTrapDebugChecks(TrapFrame, SkipPreviousMode);
}

/* --- KiExitSystemCallDebugChecks ---
 * FORCEINLINE in trap_x.h: debug checks on syscall exit (IRQL sanity, APC index).
 * Declared as extern "system" in Rust traphdlr.rs → _KiExitSystemCallDebugChecks@8 */
void NTAPI KiExitSystemCallDebugChecks(IN ULONG SystemCall, IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiExitSystemCallDebugChecks(SystemCall, TrapFrame);
}

/* KiSetTebBase lives in Rust now (ntkrnl-sys/inline/ke_i386.rs). */

#endif /* _M_IX86 (i386-specific shims) */

/* --- KeGdiFlushUserBatch_call ---
 * KeGdiFlushUserBatch is a global FUNCTION POINTER (PGDI_BATCHFLUSH_ROUTINE),
 * NOT a real function. Rust imports it as "fn KeGdiFlushUserBatch_call()".
 * We provide a wrapper that calls through the pointer.
 * Needed on all arches (i386 + amd64; ARM64 doesn't have win32k GDI batching yet). */
#if defined(_M_IX86) || defined(_M_AMD64)
void NTAPI KeGdiFlushUserBatch_call(void)
{
    extern PGDI_BATCHFLUSH_ROUTINE KeGdiFlushUserBatch;
    if (KeGdiFlushUserBatch) KeGdiFlushUserBatch();
}
#endif

/* Stub for KiReportCpuFeatures — original is in cpu.c (now cpu.rs, which
 * has it commented out). Only needed when C kiinit.c is used for hybrid
 * testing (Rust kiinit.rs has it behind #[cfg(debug_assertions)]). */
#if DBG
#ifdef _M_AMD64
VOID KiReportCpuFeatures(IN PKPRCB Prcb)
#else
VOID KiReportCpuFeatures(VOID)
#endif
{
    /* stub — the full implementation prints FeatureBits to DbgPrint */
}
#endif
