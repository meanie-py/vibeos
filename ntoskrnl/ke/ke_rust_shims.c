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

/* ke_x.h FORCEINLINE functions */
#define KiAcquireDispatcherLock         _saved_KiAcquireDispatcherLock
#define KiReleaseDispatcherLock         _saved_KiReleaseDispatcherLock
#define KiAcquireThreadLock             _saved_KiAcquireThreadLock
#define KiReleaseThreadLock             _saved_KiReleaseThreadLock
#define KiAcquireDispatcherObject       _saved_KiAcquireDispatcherObject
#define KiReleaseDispatcherObject       _saved_KiReleaseDispatcherObject
#define KiAcquireDispatcherLockAtSynchLevel _saved_KiAcquireDispatcherLockAtSynchLevel
#define KiReleaseDispatcherLockFromSynchLevel _saved_KiReleaseDispatcherLockFromSynchLevel
#define KiInsertDeferredReadyList       _saved_KiInsertDeferredReadyList
#define KiRescheduleThread              _saved_KiRescheduleThread
#define KiSetThreadSwapBusy             _saved_KiSetThreadSwapBusy
#define KiAcquirePrcbLock               _saved_KiAcquirePrcbLock
#define KiReleasePrcbLock               _saved_KiReleasePrcbLock
#define KiTryThreadLock                 _saved_KiTryThreadLock
#define KiCheckDeferredReadyList        _saved_KiCheckDeferredReadyList
#define KiRequestApcInterrupt           _saved_KiRequestApcInterrupt
#define KiAcquireTimerLock              _saved_KiAcquireTimerLock
#define KiReleaseTimerLock              _saved_KiReleaseTimerLock
#define KiAcquireApcLockRaiseToSynch    _saved_KiAcquireApcLockRaiseToSynch
#define KiAcquireApcLockAtSynchLevel    _saved_KiAcquireApcLockAtSynchLevel
#define KiAcquireApcLockRaiseToDpc      _saved_KiAcquireApcLockRaiseToDpc
#define KiReleaseApcLock                _saved_KiReleaseApcLock
#define KiReleaseApcLockFromSynchLevel  _saved_KiReleaseApcLockFromSynchLevel
#define KiAcquireProcessLockRaiseToSynch _saved_KiAcquireProcessLockRaiseToSynch
#define KiReleaseProcessLock            _saved_KiReleaseProcessLock
#define KiReleaseProcessLockFromSynchLevel _saved_KiReleaseProcessLockFromSynchLevel
#define KiAcquireDeviceQueueLock        _saved_KiAcquireDeviceQueueLock
#define KiReleaseDeviceQueueLock        _saved_KiReleaseDeviceQueueLock
#define KiRecalculateDueTime            _saved_KiRecalculateDueTime
#define KiCheckThreadStackSwap          _saved_KiCheckThreadStackSwap
#define KiCheckAlertability             _saved_KiCheckAlertability
#define KiComputeTimerTableIndex        _saved_KiComputeTimerTableIndex
#define KiRemoveEntryTimer              _saved_KiRemoveEntryTimer
#define KxInsertTimer                   _saved_KxInsertTimer
#define KiComputeDueTime                _saved_KiComputeDueTime
#define KxRemoveTreeTimer               _saved_KxRemoveTreeTimer
#define KxSetTimerForThreadWait         _saved_KxSetTimerForThreadWait
#define KxUnwaitThread                  _saved_KxUnwaitThread
#define KxUnwaitThreadForEvent          _saved_KxUnwaitThreadForEvent
#define KxQueueReadyThread              _saved_KxQueueReadyThread
#define KiSelectReadyThread             _saved_KiSelectReadyThread
#define KiComputeNewPriority            _saved_KiComputeNewPriority
#define _KeInitializeGuardedMutex       _saved__KeInitializeGuardedMutex
#define _KeAcquireGuardedMutexUnsafe    _saved__KeAcquireGuardedMutexUnsafe
#define _KeReleaseGuardedMutexUnsafe    _saved__KeReleaseGuardedMutexUnsafe
#define _KeAcquireGuardedMutex          _saved__KeAcquireGuardedMutex
#define _KeReleaseGuardedMutex          _saved__KeReleaseGuardedMutex
#define _KeTryToAcquireGuardedMutex     _saved__KeTryToAcquireGuardedMutex
#define KiAcquireNmiListLock            _saved_KiAcquireNmiListLock
#define KiReleaseNmiListLock            _saved_KiReleaseNmiListLock

/* spinlock.h FORCEINLINE functions */
#define KxAcquireSpinLock               _saved_KxAcquireSpinLock
#define KxReleaseSpinLock               _saved_KxReleaseSpinLock

/* ke.h / arch-specific FORCEINLINE functions */
/* KeQuerySystemTime is a function-like macro, not FORCEINLINE.
 * Phase 1 rename doesn't work — we implement it directly below. */
/* KeRaiseIrqlToSynchLevel is NTHALAPI (dllimport) on i386 — Phase 1 rename
 * doesn't work because _saved_ makes the compiler look for a nonexistent import.
 * Use trampoline pattern instead — capture before undef. */

/* intrinsics */
/* KfRaiseIrql/KfLowerIrql are NOT FORCEINLINE on all arches — do NOT rename.
 * On amd64 they are plain extern HAL imports.  On ARM64 they are HAL dllimport.
 * On i386 they may be FORCEINLINE or extern depending on DDK version. */

/* i386 FORCEINLINE functions from internal/i386/ke.h and intrin_i.h */
#ifdef _M_IX86
#define Ke386GetTr                      _saved_Ke386GetTr
#define Ke386SanitizeSeg                _saved_Ke386SanitizeSeg
#define Ke386SanitizeFlags              _saved_Ke386SanitizeFlags
/* KiFlushNPXState is a real function (not FORCEINLINE) — now in Rust cpu.rs.
 * Do NOT Phase 1 rename it. */
#define KiUserTrap                      _saved_KiUserTrap
#define KeQueryInterruptHandler         _saved_KeQueryInterruptHandler
#define KeRegisterInterruptHandler      _saved_KeRegisterInterruptHandler
/* Note: Ke386FnInit, Ke386SaveFpuState, Ke386RestoreFpuState,
 * Ke386ClearFpExceptions are function-like macros (#define Name(...))
 * in intrin_i.h — cannot be Phase 1 renamed. Will use inline asm shims instead. */
#define KiGetThreadNpxArea              _saved_KiGetThreadNpxArea
#define KiSwitchToBootStack              _saved_KiSwitchToBootStack
#define KeInvalidateTlbEntry            _saved_KeInvalidateTlbEntry
#define KiCheckForApcDelivery           _saved_KiCheckForApcDelivery
#define KiGetGdtEntry                   _saved_KiGetGdtEntry
#define KiSetGdtDescriptorBase          _saved_KiSetGdtDescriptorBase
#define InterlockedBitTestAndSet        _saved_InterlockedBitTestAndSet
#define InterlockedBitTestAndReset      _saved_InterlockedBitTestAndReset
/* trap_x.h FORCEINLINE functions */
#define KiEnterTrap                     _saved_KiEnterTrap
#define KiEnterInterruptTrap            _saved_KiEnterInterruptTrap
#define KiEndInterrupt                  _saved_KiEndInterrupt
/* ke.h FORCEINLINE KiDispatchException helpers */
#define KiDispatchException0Args        _saved_KiDispatchException0Args
#define KiDispatchException1Args        _saved_KiDispatchException1Args
#define KiDispatchException2Args        _saved_KiDispatchException2Args
/* intrin_i.h FORCEINLINE / macros for segment/CR registers */
#define Ke386FnSave                     _saved_Ke386FnSave
#define Ke386FxSave                     _saved_Ke386FxSave
#define Ke386FxStore                    _saved_Ke386FxStore
#define Ke386GetFs                      _saved_Ke386GetFs
#define Ke386SetCr2                     _saved_Ke386SetCr2
#define Ke386SetDs                      _saved_Ke386SetDs
#define Ke386SetEs                      _saved_Ke386SetEs
#define Ke386SetFs                      _saved_Ke386SetFs
#define Ke386SetGlobalDescriptorTable   _saved_Ke386SetGlobalDescriptorTable
#define Ke386SetLocalDescriptorTable    _saved_Ke386SetLocalDescriptorTable
#define Ke386SetTr                      _saved_Ke386SetTr
/* trap_x.h additional FORCEINLINE functions */
#define KiEnterV86Trap                  _saved_KiEnterV86Trap
#define KiExitV86Trap                   _saved_KiExitV86Trap
#define KiFillTrapFrameDebug            _saved_KiFillTrapFrameDebug
#define KiHandleDebugRegistersOnTrapEntry _saved_KiHandleDebugRegistersOnTrapEntry
#define KiHandleDebugRegistersOnTrapExit  _saved_KiHandleDebugRegistersOnTrapExit
#define KiExitTrapDebugChecks           _saved_KiExitTrapDebugChecks
#define KiExitSystemCallDebugChecks     _saved_KiExitSystemCallDebugChecks
/* ke.h FORCEINLINE functions */
#define KiGetLinkedTrapFrame            _saved_KiGetLinkedTrapFrame
#define KiExceptionExit                 _saved_KiExceptionExit
#define Ki386AdjustEsp0                 _saved_Ki386AdjustEsp0
/* intrin_i.h — Ke386SetGs is a macro, handled below */
/* internal/i386/ke.h — KiSetTebBase is FORCEINLINE */
#define KiSetTebBase                    _saved_KiSetTebBase
#define Ke386SetGdtEntryBase            _saved_Ke386SetGdtEntryBase
#endif

#if defined(_M_IX86) || defined(_M_AMD64)
#define KiCpuId                         _saved_KiCpuId
#endif

/* amd64 intrinsics + FORCEINLINE functions used by Rust cpu.rs / interrupt.rs */
#ifdef _M_AMD64
#define __readcr0                       _saved___readcr0
#define __readcr2                       _saved___readcr2
#define __readcr3                       _saved___readcr3
#define __readcr4                       _saved___readcr4
#define __readcr8                       _saved___readcr8
#define __writecr0                      _saved___writecr0
#define __writecr3                      _saved___writecr3
#define __writecr4                      _saved___writecr4
#define __writecr8                      _saved___writecr8
#define __readdr                        _saved___readdr
#define __writedr                       _saved___writedr
#define __readmsr                       _saved___readmsr
#define __writemsr                      _saved___writemsr
#define __lidt                          _saved___lidt
#define __lgdt                          _saved___lgdt
#define __sidt                          _saved___sidt
#define __sgdt                          _saved___sgdt
#define __sldt                          _saved___sldt
#define __str                           _saved___str
#define __wbinvd                        _saved___wbinvd
#define __swapgs                        _saved___swapgs
#define KeQueryInterruptHandler         _saved_KeQueryInterruptHandler
#define KeRegisterInterruptHandler      _saved_KeRegisterInterruptHandler
#define KiSendEOI                       _saved_KiSendEOI
#define KiGetGdtEntry                   _saved_KiGetGdtEntry
#define KiInitGdtEntry                  _saved_KiInitGdtEntry
#define KiGetGdtDescriptorBase          _saved_KiGetGdtDescriptorBase
#define KiSetGdtDescriptorBase          _saved_KiSetGdtDescriptorBase
#define KiSetGdtDescriptorLimit         _saved_KiSetGdtDescriptorLimit
#define KiSaveXState                    _saved_KiSaveXState
#define KiRestoreXState                 _saved_KiRestoreXState
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

#undef KiAcquireDispatcherLock
#undef KiReleaseDispatcherLock
#undef KiAcquireThreadLock
#undef KiReleaseThreadLock
#undef KiAcquireDispatcherObject
#undef KiReleaseDispatcherObject
#undef KiAcquireDispatcherLockAtSynchLevel
#undef KiReleaseDispatcherLockFromSynchLevel
#undef KiInsertDeferredReadyList
#undef KiRescheduleThread
#undef KiSetThreadSwapBusy
#undef KiAcquirePrcbLock
#undef KiReleasePrcbLock
#undef KiTryThreadLock
#undef KiCheckDeferredReadyList
#undef KiRequestApcInterrupt
#undef KiAcquireTimerLock
#undef KiReleaseTimerLock
#undef KiAcquireApcLockRaiseToSynch
#undef KiAcquireApcLockAtSynchLevel
#undef KiAcquireApcLockRaiseToDpc
#undef KiReleaseApcLock
#undef KiReleaseApcLockFromSynchLevel
#undef KiAcquireProcessLockRaiseToSynch
#undef KiReleaseProcessLock
#undef KiReleaseProcessLockFromSynchLevel
#undef KiAcquireDeviceQueueLock
#undef KiReleaseDeviceQueueLock
#undef KiRecalculateDueTime
#undef KiCheckThreadStackSwap
#undef KiCheckAlertability
#undef KiComputeTimerTableIndex
#undef KiRemoveEntryTimer
#undef KxInsertTimer
#undef KiComputeDueTime
#undef KxRemoveTreeTimer
#undef KxSetTimerForThreadWait
#undef KxUnwaitThread
#undef KxUnwaitThreadForEvent
#undef KxQueueReadyThread
#undef KiSelectReadyThread
#undef KiComputeNewPriority
#undef _KeInitializeGuardedMutex
#undef _KeAcquireGuardedMutexUnsafe
#undef _KeReleaseGuardedMutexUnsafe
#undef _KeAcquireGuardedMutex
#undef _KeReleaseGuardedMutex
#undef _KeTryToAcquireGuardedMutex
#undef KiAcquireNmiListLock
#undef KiReleaseNmiListLock
#undef KxAcquireSpinLock
#undef KxReleaseSpinLock
/* KeQuerySystemTime #undef moved to its function definition below */
/* KeRaiseIrqlToSynchLevel — not Phase 1 renamed, uses trampoline pattern */
/* KfRaiseIrql/KfLowerIrql were not renamed — no undef needed */
#ifdef _M_IX86
#undef Ke386GetTr
#undef Ke386SanitizeSeg
#undef Ke386SanitizeFlags
/* KiFlushNPXState — not Phase 1 renamed, no undef needed */
#undef KiUserTrap
#undef KeQueryInterruptHandler
#undef KeRegisterInterruptHandler
#undef KiGetThreadNpxArea
#undef KiSwitchToBootStack
#undef KeInvalidateTlbEntry
#undef KiCheckForApcDelivery
#undef KiGetGdtEntry
#undef KiSetGdtDescriptorBase
#undef InterlockedBitTestAndSet
#undef InterlockedBitTestAndReset
#undef KiEnterTrap
#undef KiEnterInterruptTrap
#undef KiEndInterrupt
#undef KiDispatchException0Args
#undef KiDispatchException1Args
#undef KiDispatchException2Args
#undef Ke386FnSave
#undef Ke386FxSave
#undef Ke386FxStore
#undef Ke386GetFs
#undef Ke386SetCr2
#undef Ke386SetDs
#undef Ke386SetEs
#undef Ke386SetFs
#undef Ke386SetGlobalDescriptorTable
#undef Ke386SetLocalDescriptorTable
#undef Ke386SetTr
#undef KiEnterV86Trap
#undef KiExitV86Trap
#undef KiFillTrapFrameDebug
#undef KiHandleDebugRegistersOnTrapEntry
#undef KiHandleDebugRegistersOnTrapExit
#undef KiExitTrapDebugChecks
#undef KiExitSystemCallDebugChecks
#undef KiGetLinkedTrapFrame
#undef KiExceptionExit
#undef Ki386AdjustEsp0
#undef KiSetTebBase
#undef Ke386SetGdtEntryBase
#endif
#if defined(_M_IX86) || defined(_M_AMD64)
#undef KiCpuId
#endif

#ifdef _M_AMD64
#undef __readcr0
#undef __readcr2
#undef __readcr3
#undef __readcr4
#undef __readcr8
#undef __writecr0
#undef __writecr3
#undef __writecr4
#undef __writecr8
#undef __readdr
#undef __writedr
#undef __readmsr
#undef __writemsr
#undef __lidt
#undef __lgdt
#undef __sidt
#undef __sgdt
#undef __sldt
#undef __str
#undef __wbinvd
#undef __swapgs
#undef KeQueryInterruptHandler
#undef KeRegisterInterruptHandler
#undef KiSendEOI
#undef KiGetGdtEntry
#undef KiInitGdtEntry
#undef KiGetGdtDescriptorBase
#undef KiSetGdtDescriptorBase
#undef KiSetGdtDescriptorLimit
#undef KiSaveXState
#undef KiRestoreXState
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

#if 1 /* Generic Phase 4 — TEMPORARILY RESTORED for debugging */

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

/* --- Timer/wait helpers (ke_x.h) --- */

PLARGE_INTEGER KiRecalculateDueTime(IN PLARGE_INTEGER OriginalDueTime,
                                     IN PLARGE_INTEGER DueTime,
                                     IN OUT PLARGE_INTEGER NewDueTime)
{
    return _saved_KiRecalculateDueTime(OriginalDueTime, DueTime, NewDueTime);
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
    _saved_KxInsertTimer(Timer, Hand);
}

BOOLEAN KiComputeDueTime(IN PKTIMER Timer, IN LARGE_INTEGER DueTime, OUT PULONG Hand)
{
    return _saved_KiComputeDueTime(Timer, DueTime, Hand);
}

VOID KxRemoveTreeTimer(IN PKTIMER Timer)
{
    _saved_KxRemoveTreeTimer(Timer);
}

VOID KxSetTimerForThreadWait(IN PKTIMER Timer, IN LARGE_INTEGER Interval, OUT PULONG Hand)
{
    _saved_KxSetTimerForThreadWait(Timer, Interval, Hand);
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

#endif /* Generic Phase 4 — now in Rust */

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

/* --- CPUID (x86/amd64 only) --- */
#if defined(_M_IX86) || defined(_M_AMD64)

VOID NTAPI KiCpuId(PCPU_INFO CpuInfo, ULONG Function)
{
    _saved_KiCpuId(CpuInfo, Function);
}

#endif /* _M_IX86 || _M_AMD64 */

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

#if 0 /* InterlockedBitTestAndSet/Reset cdecl — now in Rust inline/ke.rs */
/* --- InterlockedBitTestAndSet / InterlockedBitTestAndReset ---
 * These are compiler intrinsics mapped to macros in wdm.h.
 * Provide linker symbols for Rust. */
#undef InterlockedBitTestAndSet
#undef InterlockedBitTestAndReset

BOOLEAN InterlockedBitTestAndSet(volatile LONG *Base, LONG Bit)
{
    return _interlockedbittestandset(Base, Bit);
}

BOOLEAN InterlockedBitTestAndReset(volatile LONG *Base, LONG Bit)
{
    return _interlockedbittestandreset(Base, Bit);
}
#endif /* InterlockedBitTestAndSet/Reset cdecl */

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

/* ========================================================================
 * KeGetPcr — macro on i386 (__readfsdword), needs callable wrapper for Rust.
 * On amd64/arm64 it's already a real function in bindgen.
 * ======================================================================== */
#ifdef _M_IX86
/* Save the macro result, then undef and define a real function */
static __inline PKPCR _saved_KeGetPcr(VOID) { return KeGetPcr(); }
#undef KeGetPcr
PKPCR KeGetPcr(VOID)
{
    return _saved_KeGetPcr();
}
#endif

#if 1 /* Macro trampolines — TEMPORARILY RESTORED for debugging */

/* --- Function-like macro shims ---
 * These are #define macros in ke_x.h that need to be captured as trampolines
 * BEFORE the #undef, then provided as real functions. Since they're macros
 * (not FORCEINLINE), Phase 1 rename doesn't work. Instead, we capture them
 * in static inline trampolines above the #undef section. */

/* Capture the macro expansions into static inlines */
static __inline void _tramp_KiSatisfyObjectWait(PKMUTANT Object, PKTHREAD Thread)
{
    KiSatisfyObjectWait(Object, Thread);
}

static __inline void _tramp_KiSatisfyMutantWait(PKMUTANT Object, PKTHREAD Thread)
{
    KiSatisfyMutantWait(Object, Thread);
}

static __inline void _tramp_KiSatisfyNonMutantWait(PKMUTANT Object)
{
    KiSatisfyNonMutantWait(Object);
}

static __inline void _tramp_KiAddThreadToWaitList(PKTHREAD Thread, BOOLEAN Swappable)
{
    KiAddThreadToWaitList(Thread, Swappable);
}

/* Statement macros — these reference caller locals.
 * We provide simplified wrappers that derive Thread, WaitBlock, TimerBlock, Timer
 * from KeGetCurrentThread() and take only the truly external parameters. */

/* Now #undef the macros and define real functions */
#undef KiSatisfyObjectWait
#undef KiSatisfyMutantWait
#undef KiSatisfyNonMutantWait
#undef KiAddThreadToWaitList

void KiSatisfyObjectWait(PKMUTANT Object, PKTHREAD Thread)
{
    _tramp_KiSatisfyObjectWait(Object, Thread);
}

void KiSatisfyMutantWait(PKMUTANT Object, PKTHREAD Thread)
{
    _tramp_KiSatisfyMutantWait(Object, Thread);
}

void KiSatisfyNonMutantWait(PKMUTANT Object)
{
    _tramp_KiSatisfyNonMutantWait(Object);
}

void KiAddThreadToWaitList(PKTHREAD Thread, BOOLEAN Swappable)
{
    _tramp_KiAddThreadToWaitList(Thread, Swappable);
}

/* ProbeForReadLargeInteger is provided by se_rust_shims.c — not duplicated here */

/* Statement macro wrappers — the macros reference caller-local variables, so
 * we expose parameterized C functions. Rust callers must pass all locals.
 * These are #define KxSingleThreadWait() etc. in ke_x.h — must #undef first. */
#undef KxSingleThreadWait
#undef KxDelayThreadWait
#undef KxMultiThreadWait
void KxSingleThreadWait(
    PVOID Object, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    KWAIT_REASON WaitReason, BOOLEAN Alertable,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    PKTHREAD Thread = KeGetCurrentThread();
    PKWAIT_BLOCK WaitBlock = &Thread->WaitBlock[0];
    PKTIMER Timer = &Thread->Timer;
    PKWAIT_BLOCK TimerBlock = &Thread->WaitBlock[TIMER_WAIT_BLOCK];

    Thread->WaitBlockList = WaitBlock;
    WaitBlock->WaitKey = STATUS_SUCCESS;
    WaitBlock->Object = Object;
    WaitBlock->WaitType = WaitAny;
    Thread->WaitStatus = STATUS_SUCCESS;
    if (Timeout)
    {
        _saved_KxSetTimerForThreadWait(Timer, *Timeout, Hand);
        DueTime->QuadPart = Timer->DueTime.QuadPart;
        WaitBlock->NextWaitBlock = TimerBlock;
        TimerBlock->NextWaitBlock = WaitBlock;
        Timer->Header.WaitListHead.Flink = &TimerBlock->WaitListEntry;
        Timer->Header.WaitListHead.Blink = &TimerBlock->WaitListEntry;
    }
    else
    {
        WaitBlock->NextWaitBlock = WaitBlock;
    }
    Thread->Alertable = Alertable;
    Thread->WaitMode = WaitMode;
    Thread->WaitReason = WaitReason;
    Thread->WaitListEntry.Flink = NULL;
    *Swappable = _saved_KiCheckThreadStackSwap(Thread, WaitMode);
    Thread->WaitTime = KeTickCount.LowPart;
}

void KxDelayThreadWait(
    PLARGE_INTEGER Interval, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    PKTHREAD Thread = KeGetCurrentThread();
    PKTIMER Timer = &Thread->Timer;
    PKWAIT_BLOCK TimerBlock = &Thread->WaitBlock[TIMER_WAIT_BLOCK];

    Thread->WaitBlockList = TimerBlock;
    TimerBlock->NextWaitBlock = TimerBlock;
    _saved_KxSetTimerForThreadWait(Timer, *Interval, Hand);
    DueTime->QuadPart = Timer->DueTime.QuadPart;
    Timer->Header.WaitListHead.Flink = &TimerBlock->WaitListEntry;
    Timer->Header.WaitListHead.Blink = &TimerBlock->WaitListEntry;
    Thread->Alertable = (WaitMode == UserMode) ? TRUE : FALSE;
    Thread->WaitMode = WaitMode;
    Thread->WaitReason = DelayExecution;
    Thread->WaitListEntry.Flink = NULL;
    *Swappable = _saved_KiCheckThreadStackSwap(Thread, WaitMode);
    Thread->WaitTime = KeTickCount.LowPart;
}

void KxMultiThreadWait(
    PKWAIT_BLOCK WaitBlockArray, PVOID *Objects, ULONG Count,
    PLARGE_INTEGER Timeout, WAIT_TYPE WaitType,
    KPROCESSOR_MODE WaitMode, KWAIT_REASON WaitReason,
    BOOLEAN Alertable, BOOLEAN *Swappable,
    PLARGE_INTEGER DueTime, PULONG Hand)
{
    PKTHREAD Thread = KeGetCurrentThread();
    PKTIMER Timer = &Thread->Timer;
    PKWAIT_BLOCK TimerBlock = &Thread->WaitBlock[TIMER_WAIT_BLOCK];
    ULONG Index;

    Thread->WaitBlockList = WaitBlockArray;
    for (Index = 0; Index < Count; Index++)
    {
        WaitBlockArray[Index].Object = Objects[Index];
        WaitBlockArray[Index].WaitKey = (USHORT)Index;
        WaitBlockArray[Index].WaitType = WaitType;
        WaitBlockArray[Index].Thread = Thread;
        WaitBlockArray[Index].NextWaitBlock = &WaitBlockArray[Index + 1];
    }
    Thread->WaitStatus = STATUS_SUCCESS;
    if (Timeout)
    {
        _saved_KxSetTimerForThreadWait(Timer, *Timeout, Hand);
        DueTime->QuadPart = Timer->DueTime.QuadPart;
        WaitBlockArray[Count - 1].NextWaitBlock = TimerBlock;
        TimerBlock->NextWaitBlock = WaitBlockArray;
        Timer->Header.WaitListHead.Flink = &TimerBlock->WaitListEntry;
        Timer->Header.WaitListHead.Blink = &TimerBlock->WaitListEntry;
    }
    else
    {
        WaitBlockArray[Count - 1].NextWaitBlock = WaitBlockArray;
    }
    Thread->Alertable = Alertable;
    Thread->WaitMode = WaitMode;
    Thread->WaitReason = WaitReason;
    Thread->WaitListEntry.Flink = NULL;
    *Swappable = _saved_KiCheckThreadStackSwap(Thread, WaitMode);
    Thread->WaitTime = KeTickCount.LowPart;
}

/* --- YieldProcessor ---
 * Compiler intrinsic/macro, architecture-dependent. Provide a callable symbol. */
#undef YieldProcessor
void YieldProcessor(void)
{
#if defined(_M_IX86) || defined(_M_AMD64)
    _mm_pause();
#elif defined(_M_ARM64)
    __yield();
#endif
}

/* --- KeGetTrapFramePc ---
 * Inline function or macro, architecture-dependent.
 * On ARM64 it's a #define macro — must undef first. */
#undef KeGetTrapFramePc
ULONG_PTR KeGetTrapFramePc(IN PKTRAP_FRAME TrapFrame)
{
#if defined(_M_IX86)
    return TrapFrame->Eip;
#elif defined(_M_AMD64)
    return TrapFrame->Rip;
#elif defined(_M_ARM64)
    return TrapFrame->Pc;
#endif
}

/* KiSystemStartup for ARM64 is now provided by Rust (arm64/kiinit.rs) */

/* --- KiRundownThread ---
 * FORCEINLINE in arch-specific ke.h headers. No-op on ARM64, may have
 * FPU state cleanup on other arches. Provide callable wrapper. */
VOID KiRundownThread(IN PKTHREAD Thread)
{
    /* On ARM64: no-op. On x86/amd64: inline body handles FPU state.
     * Use the saved inline from headers. */
#if defined(_M_IX86)
    /* i386 may need NpxThread cleanup */
    PKPRCB Prcb = KeGetCurrentPrcb();
    if (Prcb->NpxThread == Thread)
    {
        Prcb->NpxThread = NULL;
    }
#elif defined(_M_AMD64) || defined(_M_ARM64)
    UNREFERENCED_PARAMETER(Thread);
#endif
}

/* --- KiGetLinkedTrapFrame ---
 * Macro on ARM64, FORCEINLINE on others. Returns previous TrapFrame. */
#undef KiGetLinkedTrapFrame
PKTRAP_FRAME KiGetLinkedTrapFrame(IN PKTRAP_FRAME TrapFrame)
{
#if defined(_M_ARM64)
    return (PKTRAP_FRAME)(TrapFrame->TrapFrame);
#elif defined(_M_AMD64)
    return (PKTRAP_FRAME)(TrapFrame->TrapFrame);
#elif defined(_M_IX86)
    return (PKTRAP_FRAME)(TrapFrame->Edx);
#endif
}

/* --- KiGetUserModeStackAddress ---
 * FORCEINLINE in arch-specific ke.h. Returns pointer to user-mode stack in TrapFrame. */
PULONG_PTR KiGetUserModeStackAddress(void)
{
#if defined(_M_ARM64)
    return &PsGetCurrentThread()->Tcb.TrapFrame->Sp;
#elif defined(_M_AMD64)
    return &PsGetCurrentThread()->Tcb.TrapFrame->Rsp;
#elif defined(_M_IX86)
    return (PULONG_PTR)&PsGetCurrentThread()->Tcb.TrapFrame->HardwareEsp;
#endif
}

/* --- KxQueueThreadWait ---
 * Statement macro in ke_x.h. References caller locals: Thread, WaitBlock,
 * Timer, TimerBlock, Queue, Timeout, WaitMode, DueTime, Hand, Swappable.
 * We provide a parameterized wrapper. */
#undef KxQueueThreadWait
void KxQueueThreadWait(
    PVOID Queue, PLARGE_INTEGER Timeout, KPROCESSOR_MODE WaitMode,
    BOOLEAN *Swappable, PLARGE_INTEGER DueTime, PULONG Hand)
{
    PKTHREAD Thread = KeGetCurrentThread();
    PKWAIT_BLOCK WaitBlock = &Thread->WaitBlock[0];
    PKTIMER Timer = &Thread->Timer;
    PKWAIT_BLOCK TimerBlock = &Thread->WaitBlock[TIMER_WAIT_BLOCK];

    Thread->WaitBlockList = WaitBlock;
    WaitBlock->WaitKey = STATUS_SUCCESS;
    WaitBlock->Object = Queue;
    WaitBlock->WaitType = WaitAny;
    WaitBlock->Thread = Thread;
    Thread->WaitStatus = STATUS_SUCCESS;
    if (Timeout)
    {
        _saved_KxSetTimerForThreadWait(Timer, *Timeout, Hand);
        DueTime->QuadPart = Timer->DueTime.QuadPart;
        WaitBlock->NextWaitBlock = TimerBlock;
        TimerBlock->NextWaitBlock = WaitBlock;
        Timer->Header.WaitListHead.Flink = &TimerBlock->WaitListEntry;
        Timer->Header.WaitListHead.Blink = &TimerBlock->WaitListEntry;
    }
    else
    {
        WaitBlock->NextWaitBlock = WaitBlock;
    }
    Thread->Alertable = FALSE;
    Thread->WaitMode = WaitMode;
    Thread->WaitReason = WrQueue;
    Thread->WaitListEntry.Flink = NULL;
    *Swappable = _saved_KiCheckThreadStackSwap(Thread, WaitMode);
    Thread->WaitTime = KeTickCount.LowPart;
}

/* --- KxUnwaitThread_impl ---
 * Rust thrdobj.rs references this name to avoid collision with the ddk import.
 * Just forward to the existing KxUnwaitThread shim. */
void KxUnwaitThread_impl(DISPATCHER_HEADER *Header, KPRIORITY Increment)
{
    KxUnwaitThread(Header, Increment);
}

#endif /* Macro trampolines + statement macros + utilities — now in Rust */

/* --- HalQuerySystemInformation / HalSetSystemInformation ---
 * These are macros: HALDISPATCH->HalQuerySystemInformation etc.
 * Rust profobj.rs declares them as extern "system" functions.
 * Capture macro expansion in trampolines, then undef and provide real functions. */
static __inline NTSTATUS _tramp_HalQuerySystemInformation(
    IN HAL_QUERY_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN OUT PVOID Buffer,
    OUT PULONG ReturnedLength)
{
    return HalQuerySystemInformation(InformationClass, BufferSize, Buffer, ReturnedLength);
}

static __inline NTSTATUS _tramp_HalSetSystemInformation(
    IN HAL_SET_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN PVOID Buffer)
{
    return HalSetSystemInformation(InformationClass, BufferSize, Buffer);
}

#undef HalQuerySystemInformation
#undef HalSetSystemInformation

NTSTATUS NTAPI HalQuerySystemInformation(
    IN HAL_QUERY_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN OUT PVOID Buffer,
    OUT PULONG ReturnedLength)
{
    return _tramp_HalQuerySystemInformation(InformationClass, BufferSize, Buffer, ReturnedLength);
}

NTSTATUS NTAPI HalSetSystemInformation(
    IN HAL_SET_INFORMATION_CLASS InformationClass,
    IN ULONG BufferSize,
    IN PVOID Buffer)
{
    return _tramp_HalSetSystemInformation(InformationClass, BufferSize, Buffer);
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

#if 0 /* amd64 intrinsics — now in Rust inline/ke_amd64.rs + shim_exports.rs */
/* ========================================================================
 * amd64 compiler intrinsic wrappers
 *
 * These are __INTRIN_INLINE (static inline) in the C headers but Rust
 * needs callable linker symbols. Phase 1 renamed them to _saved_* above,
 * so we can define real functions here that forward to the saved inlines.
 * ======================================================================== */
#ifdef _M_AMD64

ULONG_PTR __readcr0(VOID)    { return _saved___readcr0(); }
ULONG_PTR __readcr2(VOID)    { return _saved___readcr2(); }
ULONG_PTR __readcr3(VOID)    { return _saved___readcr3(); }
ULONG_PTR __readcr4(VOID)    { return _saved___readcr4(); }
ULONG_PTR __readcr8(VOID)    { return _saved___readcr8(); }

void __writecr0(ULONG_PTR v) { _saved___writecr0(v); }
void __writecr3(ULONG_PTR v) { _saved___writecr3(v); }
void __writecr4(ULONG_PTR v) { _saved___writecr4(v); }
void __writecr8(ULONG_PTR v) { _saved___writecr8(v); }

ULONG_PTR __readdr(ULONG reg)              { return _saved___readdr(reg); }
void __writedr(ULONG reg, ULONG_PTR val)   { _saved___writedr(reg, val); }

ULONGLONG __readmsr(ULONG reg)               { return _saved___readmsr(reg); }
void __writemsr(ULONG reg, ULONGLONG val)    { _saved___writemsr(reg, val); }

void __lidt(void *Source) { _saved___lidt(Source); }
void __lgdt(void *Source) { _saved___lgdt(Source); }
void __sidt(void *Dest)   { _saved___sidt(Dest); }
void __sgdt(void *Dest)   { _saved___sgdt(Dest); }
void __sldt(USHORT *Ldtr)  { _saved___sldt(Ldtr); }
void __str(USHORT *Tr)     { _saved___str(Tr); }

void __wbinvd(VOID)
{
    _saved___wbinvd();
}

void KeMemoryBarrier(VOID)
{
    __asm__ __volatile__("mfence" : : : "memory");
}

/* --- KeQueryInterruptHandler / KeRegisterInterruptHandler ---
 * FORCEINLINE in internal/amd64/ke.h. The Rust interrupt.rs declares them
 * as extern "system" imports. Provide callable symbols via Phase 1 rename. */
PVOID NTAPI KeQueryInterruptHandler(IN ULONG Vector)
{
    return _saved_KeQueryInterruptHandler(Vector);
}

VOID NTAPI KeRegisterInterruptHandler(IN ULONG Vector, IN PVOID Handler)
{
    _saved_KeRegisterInterruptHandler(Vector, Handler);
}

/* --- Clang builtin wrappers ---
 * __cpuid, __cpuidex, _mm_getcsr, _mm_setcsr are Clang builtins.
 * Can't redefine as C functions. Use file-scope asm to provide linker symbols. */
/* Microsoft x64 ABI: args in rcx, rdx, r8, r9 */
__asm__(
    ".globl __cpuid\n"
    "__cpuid:\n"
    "  pushq %rbx\n"
    "  movq  %rcx, %r9\n"          /* r9 = CPUInfo ptr (save rcx) */
    "  movl  %edx, %eax\n"         /* eax = InfoType */
    "  xorl  %ecx, %ecx\n"         /* ecx = 0 (default subleaf) */
    "  cpuid\n"
    "  movl  %eax,  (%r9)\n"
    "  movl  %ebx, 4(%r9)\n"
    "  movl  %ecx, 8(%r9)\n"
    "  movl  %edx,12(%r9)\n"
    "  popq  %rbx\n"
    "  retq\n"
    "\n"
    ".globl __cpuidex\n"
    "__cpuidex:\n"
    "  pushq %rbx\n"
    "  movq  %rcx, %r9\n"          /* r9 = CPUInfo ptr */
    "  movl  %edx, %eax\n"         /* eax = InfoType */
    "  movl  %r8d, %ecx\n"         /* ecx = ECXValue */
    "  cpuid\n"
    "  movl  %eax,  (%r9)\n"
    "  movl  %ebx, 4(%r9)\n"
    "  movl  %ecx, 8(%r9)\n"
    "  movl  %edx,12(%r9)\n"
    "  popq  %rbx\n"
    "  retq\n"
    "\n"
    ".globl _mm_getcsr\n"
    "_mm_getcsr:\n"
    "  subq  $4, %rsp\n"
    "  stmxcsr (%rsp)\n"
    "  movl  (%rsp), %eax\n"
    "  addq  $4, %rsp\n"
    "  retq\n"
    "\n"
    ".globl _mm_setcsr\n"
    "_mm_setcsr:\n"
    "  subq  $4, %rsp\n"
    "  movl  %ecx, (%rsp)\n"       /* ecx = first arg (MS ABI) */
    "  ldmxcsr (%rsp)\n"
    "  addq  $4, %rsp\n"
    "  retq\n"
);

/* --- RtlFillMemoryUlong ---
 * ntoskrnl.h does: #define RtlFillMemoryUlong _RtlFillMemoryUlong
 * Rust code declares: fn RtlFillMemoryUlong(...) — no underscore prefix.
 * Provide a symbol with the unmangled name that forwards to the real one. */
#undef RtlFillMemoryUlong
VOID RtlFillMemoryUlong(PVOID Destination, SIZE_T Length, ULONG Pattern)
{
    _RtlFillMemoryUlong(Destination, Length, Pattern);
}

/* --- KiSendEOI ---
 * FORCEINLINE in internal/amd64/ke.h. Writes 0 to APIC EOI register. */
VOID KiSendEOI(VOID)
{
    _saved_KiSendEOI();
}

/* --- __swapgs ---
 * static inline in intrin_i.h. swapgs instruction. */
void __swapgs(void)
{
    _saved___swapgs();
}

/* --- GDT helpers (intrin_i.h FORCEINLINE) --- */
PKGDTENTRY64 KiGetGdtEntry(PVOID pGdt, USHORT Selector)
{
    return _saved_KiGetGdtEntry(pGdt, Selector);
}

PVOID KiGetGdtDescriptorBase(PKGDTENTRY Entry)
{
    return _saved_KiGetGdtDescriptorBase(Entry);
}

VOID KiInitGdtEntry(PKGDTENTRY64 Entry, ULONG64 Base, ULONG Size, UCHAR Type, UCHAR Dpl)
{
    _saved_KiInitGdtEntry(Entry, Base, Size, Type, Dpl);
}

/* --- KiSaveXState / KiRestoreXState (intrin_i.h FORCEINLINE) --- */
VOID KiSaveXState(PVOID Buffer, ULONG64 ComponentMask)
{
    _saved_KiSaveXState(Buffer, ComponentMask);
}

VOID KiRestoreXState(PVOID Buffer, ULONG64 ComponentMask)
{
    _saved_KiRestoreXState(Buffer, ComponentMask);
}

/* --- _enable / _disable ---
 * Compiler intrinsics: sti / cli. Use file-scope asm since they are builtins. */
__asm__(
    ".globl _enable\n"
    "_enable:\n"
    "  sti\n"
    "  retq\n"
    "\n"
    ".globl _disable\n"
    "_disable:\n"
    "  cli\n"
    "  retq\n"
    "\n"
    ".globl __debugbreak\n"
    "__debugbreak:\n"
    "  int $3\n"
    "  retq\n"
    "\n"
    /* __addgsdword(offset, value): add value to gs:[offset]
     * MS x64 ABI: ecx = offset, edx = value */
    ".globl __addgsdword\n"
    "__addgsdword:\n"
    "  addl %edx, %gs:(%ecx)\n"
    "  retq\n"
    "\n"
    /* _xsetbv(xcr, value): MS x64 ABI: ecx = xcr, rdx = value
     * xsetbv takes ecx = xcr, edx:eax = value */
    ".globl _xsetbv\n"
    "_xsetbv:\n"
    "  movq %rdx, %rax\n"           /* rax = value (low 32 bits) */
    "  shrq $32, %rdx\n"            /* edx = value >> 32 (high 32 bits) */
    "  xsetbv\n"
    "  retq\n"
    "\n"
    /* InterlockedBitTestAndSet64(Base, Bit):
     * MS x64 ABI: rcx = Base ptr, rdx = Bit
     * Returns old bit value in al */
    ".globl InterlockedBitTestAndSet64\n"
    "InterlockedBitTestAndSet64:\n"
    "  xorl %eax, %eax\n"
    "  lock btsq %rdx, (%rcx)\n"
    "  setc %al\n"
    "  retq\n"
    "\n"
    /* InterlockedAnd64(Destination, Value):
     * MS x64 ABI: rcx = Destination ptr, rdx = Value
     * Returns original value */
    ".globl InterlockedAnd64\n"
    "InterlockedAnd64:\n"
    "  movq (%rcx), %rax\n"
    "1:\n"
    "  movq %rax, %r8\n"
    "  andq %rdx, %r8\n"
    "  lock cmpxchgq %r8, (%rcx)\n"
    "  jnz 1b\n"
    "  retq\n"
    "\n"
    /* __rdtsc: returns 64-bit TSC in rax (edx:eax combined) */
    ".globl __rdtsc\n"
    "__rdtsc:\n"
    "  rdtsc\n"
    "  shlq $32, %rdx\n"
    "  orq  %rdx, %rax\n"
    "  retq\n"
);

#endif /* _M_AMD64 */
#endif /* amd64 intrinsics — now in Rust */

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

USHORT NTAPI Ke386GetTr(VOID)
{
    return _saved_Ke386GetTr();
}

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

ULONG NTAPI Ke386SanitizeSeg(IN ULONG Cs, IN KPROCESSOR_MODE Mode)
{
    return _saved_Ke386SanitizeSeg(Cs, Mode);
}

ULONG NTAPI Ke386SanitizeFlags(IN ULONG Eflags, IN KPROCESSOR_MODE Mode)
{
    return _saved_Ke386SanitizeFlags(Eflags, Mode);
}

/* KiFlushNPXState is now in Rust cpu.rs — no C shim needed */

BOOLEAN NTAPI KiUserTrap(IN PKTRAP_FRAME TrapFrame)
{
    return _saved_KiUserTrap(TrapFrame);
}

/* --- HalVectorToIDTEntry ---
 * Macro expanding to HALPRIVATEDISPATCH->HalVectorToIDTEntry. */
static __inline UCHAR _tramp_HalVectorToIDTEntry(IN ULONG Vector)
{
    return HalVectorToIDTEntry(Vector);
}
#undef HalVectorToIDTEntry
UCHAR NTAPI HalVectorToIDTEntry(IN ULONG Vector)
{
    return _tramp_HalVectorToIDTEntry(Vector);
}

/* --- KeQueryInterruptHandler / KeRegisterInterruptHandler ---
 * FORCEINLINE in internal/i386/ke.h. */
PVOID NTAPI KeQueryInterruptHandler(IN ULONG Vector)
{
    return _saved_KeQueryInterruptHandler(Vector);
}

VOID NTAPI KeRegisterInterruptHandler(IN ULONG Vector, IN PVOID Handler)
{
    _saved_KeRegisterInterruptHandler(Vector, Handler);
}

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
PFX_SAVE_AREA NTAPI KiGetThreadNpxArea(IN PKTHREAD Thread)
{
    return _saved_KiGetThreadNpxArea(Thread);
}

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

/* --- KeInvalidateTlbEntry --- FORCEINLINE in internal/i386/ke.h */
VOID KeInvalidateTlbEntry(IN PVOID Address)
{
    _saved_KeInvalidateTlbEntry(Address);
}

/* --- KiCheckForApcDelivery --- FORCEINLINE in internal/i386/ke.h */
VOID NTAPI KiCheckForApcDelivery(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiCheckForApcDelivery(TrapFrame);
}

/* --- KiGetGdtEntry --- FORCEINLINE in internal/i386/intrin_i.h */
PKGDTENTRY KiGetGdtEntry(IN PVOID pGdt, IN USHORT Selector)
{
    return _saved_KiGetGdtEntry(pGdt, Selector);
}

/* --- KiSetGdtDescriptorBase --- FORCEINLINE in internal/i386/intrin_i.h */
VOID KiSetGdtDescriptorBase(IN PKGDTENTRY Entry, IN ULONG Base)
{
    _saved_KiSetGdtDescriptorBase(Entry, Base);
}

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

/* --- KiEnterTrap ---
 * FORCEINLINE in trap_x.h. Sets up trap frame debug state.
 * Declared as extern "fastcall" in Rust traphdlr.rs → @KiEnterTrap@4 */
void FASTCALL KiEnterTrap(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiEnterTrap(TrapFrame);
}

/* --- KiEnterInterruptTrap ---
 * FORCEINLINE in trap_x.h. Sets up interrupt trap frame.
 * Declared as extern "C" in Rust irqobj.rs → _KiEnterInterruptTrap */
void __cdecl KiEnterInterruptTrap(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiEnterInterruptTrap(TrapFrame);
}

/* --- KiEndInterrupt ---
 * FORCEINLINE in ke.h. Ends interrupt via HAL and exits trap.
 * Declared as extern "C" in Rust time.rs → _KiEndInterrupt */
void __cdecl KiEndInterrupt(IN KIRQL Irql, IN PKTRAP_FRAME TrapFrame)
{
    _disable();
    HalEndSystemInterrupt(Irql, TrapFrame);
    KiEoiHelper(TrapFrame);
}

/* --- KiDispatchException0Args ---
 * FORCEINLINE DECLSPEC_NORETURN in ke.h. Wraps KiDispatchExceptionFromTrapFrame.
 * Declared as extern "fastcall" in Rust → @KiDispatchException0Args@12 */
DECLSPEC_NORETURN
void FASTCALL KiDispatchException0Args(IN NTSTATUS Code,
                                        IN ULONG_PTR Address,
                                        IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiDispatchException0Args(Code, Address, TrapFrame);
}

/* --- KiDispatchException1Args ---
 * FORCEINLINE DECLSPEC_NORETURN in ke.h.
 * Declared as extern "fastcall" in Rust → @KiDispatchException1Args@16 */
DECLSPEC_NORETURN
void FASTCALL KiDispatchException1Args(IN NTSTATUS Code,
                                        IN ULONG_PTR Address,
                                        IN ULONG P1,
                                        IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiDispatchException1Args(Code, Address, P1, TrapFrame);
}

/* --- KiDispatchException2Args ---
 * FORCEINLINE DECLSPEC_NORETURN in ke.h.
 * Declared as extern "fastcall" in Rust → @KiDispatchException2Args@20 */
DECLSPEC_NORETURN
void FASTCALL KiDispatchException2Args(IN NTSTATUS Code,
                                        IN ULONG_PTR Address,
                                        IN ULONG P1,
                                        IN ULONG P2,
                                        IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiDispatchException2Args(Code, Address, P1, P2, TrapFrame);
}

/* --- KiSwitchToBootStack ---
 * FORCEINLINE DECLSPEC_NORETURN in ke.h. Manipulates ESP and jumps to
 * KiSystemStartupBootStack. Cannot be trivially shimmed as a C call — use asm. */
CODE_SEG("INIT")
DECLSPEC_NORETURN
void NTAPI KiSwitchToBootStack(IN ULONG_PTR InitialStack)
{
    _saved_KiSwitchToBootStack(InitialStack);
}

/* --- Ke386FnSave ---
 * FORCEINLINE with inline asm: fnsave + wait. */
void NTAPI Ke386FnSave(IN PVOID SaveArea)
{
    __asm__ __volatile__("fnsave (%0); wait" : : "r"(SaveArea) : "memory");
}

/* --- Ke386FxSave ---
 * FORCEINLINE with inline asm: fxsave. */
void NTAPI Ke386FxSave(IN PVOID SaveArea)
{
    __asm__ __volatile__("fxsave (%0)" : : "r"(SaveArea) : "memory");
}

/* --- Ke386FxStore ---
 * FORCEINLINE with inline asm: fxrstor (misnamed "Store" = restore).
 * Sanitize MXCSR before FXRSTOR to prevent #GP from reserved bits. */
void NTAPI Ke386FxStore(IN PVOID SaveArea)
{
    ULONG *mxcsr = (ULONG *)((UCHAR *)SaveArea + 24);
    ULONG *mxcsr_mask = (ULONG *)((UCHAR *)SaveArea + 28);
    ULONG mask = *mxcsr_mask ? *mxcsr_mask : 0xFFBF;
    *mxcsr &= mask;
    __asm__ __volatile__("fxrstor (%0)" : : "r"(SaveArea) : "memory");
}

/* --- Ke386GetFs ---
 * Macro with inline asm: movl %fs, reg. Returns USHORT. */
USHORT Ke386GetFs(void)
{
    unsigned int __d;
    __asm__("movl %%fs,%0\n\t" :"=r" (__d));
    return (USHORT)__d;
}

/* --- Ke386SetCr2 ---
 * Macro with inline asm: movl val, %cr2. */
void NTAPI Ke386SetCr2(IN ULONG Value)
{
    __asm__ __volatile__("movl %0,%%cr2" : :"r" (Value));
}

/* --- Ke386SetDs ---
 * Macro with inline asm: movl val, %ds. */
void NTAPI Ke386SetDs(IN ULONG Value)
{
    __asm__ __volatile__("movl %0,%%ds" : :"r" (Value));
}

/* --- Ke386SetEs ---
 * Macro with inline asm: movl val, %es. */
void NTAPI Ke386SetEs(IN ULONG Value)
{
    __asm__ __volatile__("movl %0,%%es" : :"r" (Value));
}

/* --- Ke386SetFs ---
 * Macro with inline asm: movl val, %fs. */
void NTAPI Ke386SetFs(IN ULONG Value)
{
    __asm__ __volatile__("movl %0,%%fs" : :"r" (Value));
}

/* --- Ke386SetGlobalDescriptorTable ---
 * Macro: lgdt. Takes pointer to KDESCRIPTOR. */
void NTAPI Ke386SetGlobalDescriptorTable(IN PVOID Descriptor)
{
    __asm__ __volatile__("lgdt (%0)" : : "r"(Descriptor) : "memory");
}

/* --- Ke386SetLocalDescriptorTable ---
 * Macro: lldt. Takes USHORT selector. */
void NTAPI Ke386SetLocalDescriptorTable(IN USHORT Selector)
{
    __asm__ __volatile__("lldt %w0" : : "q"(Selector));
}

/* --- Ke386SetTr ---
 * Macro: ltr. Takes USHORT selector. */
void NTAPI Ke386SetTr(IN USHORT Selector)
{
    __asm__ __volatile__("ltr %%ax" : :"a" (Selector));
}

/* --- KiEnterV86Trap ---
 * FORCEINLINE in trap_x.h. Saves exception list and sets V86 mode state.
 * Declared as extern "fastcall" in Rust traphdlr.rs → @KiEnterV86Trap@4 */
void FASTCALL KiEnterV86Trap(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiEnterV86Trap(TrapFrame);
}

/* --- KiExitV86Trap ---
 * FORCEINLINE DECLSPEC_NORETURN in trap_x.h. Exits V86 mode trap.
 * Declared as extern "fastcall" in Rust traphdlr.rs → @KiExitV86Trap@4 */
DECLSPEC_NORETURN
void FASTCALL KiExitV86Trap(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiExitV86Trap(TrapFrame);
}

/* --- KiFillTrapFrameDebug ---
 * FORCEINLINE in trap_x.h (DBG only, otherwise empty macro).
 * Declared as extern "system" in Rust v86vdm.rs/traphdlr.rs → _KiFillTrapFrameDebug@4 */
void NTAPI KiFillTrapFrameDebug(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiFillTrapFrameDebug(TrapFrame);
}

/* --- KiExceptionExit ---
 * FORCEINLINE on i386: calls KiServiceExit2(TrapFrame).
 * Declared as extern "C" in DDK bindgen → _KiExceptionExit (cdecl)
 * Note: ExceptionFrame is unused on i386 (i386 has no KEXCEPTION_FRAME). */
DECLSPEC_NORETURN
void __cdecl KiExceptionExit(IN PKTRAP_FRAME TrapFrame,
                              IN PKEXCEPTION_FRAME ExceptionFrame)
{
    _saved_KiExceptionExit(TrapFrame, ExceptionFrame);
}

/* --- KiHandleDebugRegistersOnTrapEntry ---
 * FORCEINLINE in trap_x.h: saves debug registers into TrapFrame and disables DR7.
 * Declared as extern "system" in Rust traphdlr.rs → _KiHandleDebugRegistersOnTrapEntry@4 */
void NTAPI KiHandleDebugRegistersOnTrapEntry(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiHandleDebugRegistersOnTrapEntry(TrapFrame);
}

/* --- KiHandleDebugRegistersOnTrapExit ---
 * FORCEINLINE in trap_x.h: restores debug registers from TrapFrame/Prcb.
 * Declared as extern "system" in Rust traphdlr.rs → _KiHandleDebugRegistersOnTrapExit@4 */
void NTAPI KiHandleDebugRegistersOnTrapExit(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiHandleDebugRegistersOnTrapExit(TrapFrame);
}

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

/* --- Ke386SetGs ---
 * Macro/FORCEINLINE in intrin_i.h: sets GS segment register.
 * Cannot Phase 1 rename (it's a #define macro), so undef and implement directly. */
#undef Ke386SetGs
void NTAPI Ke386SetGs(IN USHORT Value)
{
    __asm__ __volatile__("movw %0, %%gs" :: "r"(Value));
}

/* --- KiSetTebBase ---
 * FORCEINLINE in internal/i386/ke.h: updates PCR NtTib.Self and GDT TEB entry.
 * Declared as extern "system" in Rust thrdini.rs → _KiSetTebBase@8 */
void NTAPI KiSetTebBase(PKPCR Pcr, PNT_TIB TebAddress)
{
    _saved_KiSetTebBase(Pcr, TebAddress);
}

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
