/*
 * ke_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust KE subsystem calls via extern declarations.
 *
 * Common shims (CONTAINING_RECORD, InsertTailList, etc.)
 * are provided by ob/ob_rust_shims.c and tier1_rust_shims.c.
 *
 * KE dispatcher/thread lock shims are shared with EX subsystem —
 * they live in ex_rust_shims.c (KiAcquireDispatcherLock etc.) and are
 * NOT duplicated here.
 */

/* ========================================================================
 * Phase 1: Rename FORCEINLINE functions BEFORE including headers
 *
 * By #defining the function name to _saved_Name before headers are parsed,
 * the FORCEINLINE definition creates a _saved_Name static inline. After
 * headers, we #undef and define real non-inline functions with original names.
 * ======================================================================== */

/* ke_x.h FORCEINLINE functions */
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
/* ke.h FORCEINLINE functions */
#define KiGetLinkedTrapFrame            _saved_KiGetLinkedTrapFrame
#define KiExceptionExit                 _saved_KiExceptionExit
#define Ki386AdjustEsp0                 _saved_Ki386AdjustEsp0
#endif

#if defined(_M_IX86) || defined(_M_AMD64)
#define KiCpuId                         _saved_KiCpuId
#define KiCpuIdEx                       _saved_KiCpuIdEx
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
#undef KiGetLinkedTrapFrame
#undef KiExceptionExit
#undef Ki386AdjustEsp0
#endif
#if defined(_M_IX86) || defined(_M_AMD64)
#undef KiCpuId
#undef KiCpuIdEx
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

/* ========================================================================
 * Phase 4: Real (non-static, non-inline) function implementations
 *
 * Rust calls these via extern "C" (cdecl). No NTAPI unless noted.
 * ======================================================================== */

/* --- Dispatcher object lock (ke_x.h) --- */

VOID KiAcquireDispatcherObject(IN DISPATCHER_HEADER *Object)
{
    _saved_KiAcquireDispatcherObject(Object);
}

VOID KiReleaseDispatcherObject(IN DISPATCHER_HEADER *Object)
{
    _saved_KiReleaseDispatcherObject(Object);
}

VOID KiAcquireDispatcherLockAtSynchLevel(VOID)
{
    _saved_KiAcquireDispatcherLockAtSynchLevel();
}

VOID KiReleaseDispatcherLockFromSynchLevel(VOID)
{
    _saved_KiReleaseDispatcherLockFromSynchLevel();
}

/* --- Deferred ready list (ke_x.h) --- */

VOID KiInsertDeferredReadyList(IN PKTHREAD Thread)
{
    _saved_KiInsertDeferredReadyList(Thread);
}

VOID KiRescheduleThread(IN BOOLEAN NewThread, IN ULONG Cpu)
{
    _saved_KiRescheduleThread(NewThread, Cpu);
}

/* --- Thread swap (ke_x.h) --- */

VOID KiSetThreadSwapBusy(IN PKTHREAD Thread)
{
    _saved_KiSetThreadSwapBusy(Thread);
}

/* --- PRCB lock (ke_x.h) --- */

VOID KiAcquirePrcbLock(IN PKPRCB Prcb)
{
    _saved_KiAcquirePrcbLock(Prcb);
}

VOID KiReleasePrcbLock(IN PKPRCB Prcb)
{
    _saved_KiReleasePrcbLock(Prcb);
}

/* --- Thread lock (ke_x.h) --- */

BOOLEAN KiTryThreadLock(IN PKTHREAD Thread)
{
    return _saved_KiTryThreadLock(Thread);
}

/* --- Deferred ready list check (ke_x.h) --- */

VOID KiCheckDeferredReadyList(IN PKPRCB Prcb)
{
    _saved_KiCheckDeferredReadyList(Prcb);
}

/* --- APC interrupt (ke_x.h) --- */

VOID KiRequestApcInterrupt(IN BOOLEAN NeedApc, IN UCHAR Processor)
{
    _saved_KiRequestApcInterrupt(NeedApc, Processor);
}

/* --- Timer lock (ke_x.h) --- */

PKSPIN_LOCK_QUEUE KiAcquireTimerLock(IN ULONG Hand)
{
    return _saved_KiAcquireTimerLock(Hand);
}

VOID KiReleaseTimerLock(IN PKSPIN_LOCK_QUEUE LockQueue)
{
    _saved_KiReleaseTimerLock(LockQueue);
}

/* --- APC lock (ke_x.h) --- */

VOID KiAcquireApcLockRaiseToSynch(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiAcquireApcLockRaiseToSynch(Thread, Handle);
}

VOID KiAcquireApcLockAtSynchLevel(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiAcquireApcLockAtSynchLevel(Thread, Handle);
}

VOID KiAcquireApcLockRaiseToDpc(IN PKTHREAD Thread, IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiAcquireApcLockRaiseToDpc(Thread, Handle);
}

VOID KiReleaseApcLock(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiReleaseApcLock(Handle);
}

VOID KiReleaseApcLockFromSynchLevel(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiReleaseApcLockFromSynchLevel(Handle);
}

/* --- Process lock (ke_x.h) --- */
/* KiAcquireProcessLockRaiseToSynch and KiReleaseProcessLock are already
 * provided by mm_rust_shims.c — only provide KiReleaseProcessLockFromSynchLevel here. */

VOID KiReleaseProcessLockFromSynchLevel(IN PKLOCK_QUEUE_HANDLE Handle)
{
    _saved_KiReleaseProcessLockFromSynchLevel(Handle);
}

/* --- Device queue lock (ke_x.h) --- */

VOID KiAcquireDeviceQueueLock(IN PKDEVICE_QUEUE DeviceQueue, IN PKLOCK_QUEUE_HANDLE DeviceLock)
{
    _saved_KiAcquireDeviceQueueLock(DeviceQueue, DeviceLock);
}

VOID KiReleaseDeviceQueueLock(IN PKLOCK_QUEUE_HANDLE DeviceLock)
{
    _saved_KiReleaseDeviceQueueLock(DeviceLock);
}

/* --- Timer/wait helpers (ke_x.h) --- */

PLARGE_INTEGER KiRecalculateDueTime(IN PLARGE_INTEGER OriginalDueTime,
                                     IN PLARGE_INTEGER DueTime,
                                     IN OUT PLARGE_INTEGER NewDueTime)
{
    return _saved_KiRecalculateDueTime(OriginalDueTime, DueTime, NewDueTime);
}

BOOLEAN KiCheckThreadStackSwap(IN PKTHREAD Thread, IN KPROCESSOR_MODE WaitMode)
{
    return _saved_KiCheckThreadStackSwap(Thread, WaitMode);
}

NTSTATUS KiCheckAlertability(IN PKTHREAD Thread, IN BOOLEAN Alertable, IN KPROCESSOR_MODE WaitMode)
{
    return _saved_KiCheckAlertability(Thread, Alertable, WaitMode);
}

ULONG KiComputeTimerTableIndex(IN ULONGLONG DueTime)
{
    return _saved_KiComputeTimerTableIndex(DueTime);
}

VOID KiRemoveEntryTimer(IN PKTIMER Timer)
{
    _saved_KiRemoveEntryTimer(Timer);
}

VOID NTAPI KxInsertTimer(IN PKTIMER Timer, IN ULONG Hand)
{
    _saved_KxInsertTimer(Timer, Hand);
}

BOOLEAN KiComputeDueTime(IN PKTIMER Timer, IN LARGE_INTEGER DueTime, OUT PULONG Hand)
{
    return _saved_KiComputeDueTime(Timer, DueTime, Hand);
}

VOID NTAPI KxRemoveTreeTimer(IN PKTIMER Timer)
{
    _saved_KxRemoveTreeTimer(Timer);
}

VOID KxSetTimerForThreadWait(IN PKTIMER Timer, IN LARGE_INTEGER Interval, OUT PULONG Hand)
{
    _saved_KxSetTimerForThreadWait(Timer, Interval, Hand);
}

/* --- Unwait (ke_x.h) --- */

VOID NTAPI KxUnwaitThread(IN DISPATCHER_HEADER *Object, IN KPRIORITY Increment)
{
    _saved_KxUnwaitThread(Object, Increment);
}

VOID NTAPI KxUnwaitThreadForEvent(IN PKEVENT Event, IN KPRIORITY Increment)
{
    _saved_KxUnwaitThreadForEvent(Event, Increment);
}

/* --- Ready queue (ke_x.h) --- */

VOID NTAPI KxQueueReadyThread(IN PKTHREAD Thread, IN PKPRCB Prcb)
{
    _saved_KxQueueReadyThread(Thread, Prcb);
}

PKTHREAD KiSelectReadyThread(IN KPRIORITY Priority, IN PKPRCB Prcb)
{
    return _saved_KiSelectReadyThread(Priority, Prcb);
}

SCHAR KiComputeNewPriority(IN PKTHREAD Thread, IN SCHAR Adjustment)
{
    return _saved_KiComputeNewPriority(Thread, Adjustment);
}

/* --- Guarded mutex (ke_x.h) --- */

VOID _KeInitializeGuardedMutex(OUT PKGUARDED_MUTEX GuardedMutex)
{
    _saved__KeInitializeGuardedMutex(GuardedMutex);
}

VOID _KeAcquireGuardedMutexUnsafe(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _saved__KeAcquireGuardedMutexUnsafe(GuardedMutex);
}

VOID _KeReleaseGuardedMutexUnsafe(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _saved__KeReleaseGuardedMutexUnsafe(GuardedMutex);
}

VOID _KeAcquireGuardedMutex(IN PKGUARDED_MUTEX GuardedMutex)
{
    _saved__KeAcquireGuardedMutex(GuardedMutex);
}

VOID _KeReleaseGuardedMutex(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    _saved__KeReleaseGuardedMutex(GuardedMutex);
}

BOOLEAN _KeTryToAcquireGuardedMutex(IN OUT PKGUARDED_MUTEX GuardedMutex)
{
    return _saved__KeTryToAcquireGuardedMutex(GuardedMutex);
}

/* --- NMI list lock (ke_x.h) --- */

VOID KiAcquireNmiListLock(OUT PKIRQL OldIrql)
{
    _saved_KiAcquireNmiListLock(OldIrql);
}

VOID KiReleaseNmiListLock(IN KIRQL OldIrql)
{
    _saved_KiReleaseNmiListLock(OldIrql);
}

/* --- IRQL functions — KeRaiseIrql/KeLowerIrql are macros that expand to
 * KfRaiseIrql/KfLowerIrql.  Inside ntoskrnl (_NTOSKRNL_ defined), the HAL
 * imports KfRaiseIrql/KfLowerIrql may be suppressed.  Capture the macros
 * in trampolines BEFORE undef, then define real callable functions. */
static __inline VOID _tramp_KeRaiseIrql(IN KIRQL NewIrql, OUT PKIRQL OldIrql)
{
    KeRaiseIrql(NewIrql, OldIrql);
}
static __inline VOID _tramp_KeLowerIrql(IN KIRQL NewIrql)
{
    KeLowerIrql(NewIrql);
}
#undef KeRaiseIrql
#undef KeLowerIrql
VOID NTAPI KeRaiseIrql(IN KIRQL NewIrql, OUT PKIRQL OldIrql)
{
    _tramp_KeRaiseIrql(NewIrql, OldIrql);
}
VOID NTAPI KeLowerIrql(IN KIRQL NewIrql)
{
    _tramp_KeLowerIrql(NewIrql);
}

/* KeRaiseIrqlToSynchLevel — trampoline pattern.
 * On i386/amd64 it's NTHALAPI (dllimport from HAL). Capture via trampoline
 * before undef, then provide a real callable function. */
#if defined(_M_IX86) || defined(_M_AMD64)
static __inline KIRQL _tramp_KeRaiseIrqlToSynchLevel(VOID)
{
    return KeRaiseIrqlToSynchLevel();
}
#undef KeRaiseIrqlToSynchLevel
KIRQL NTAPI KeRaiseIrqlToSynchLevel(VOID)
{
    return _tramp_KeRaiseIrqlToSynchLevel();
}
#endif

/* KeQuerySystemTime is already provided by ps_rust_shims.c — do not duplicate here. */

/* --- CPUID (x86/amd64 only) --- */
#if defined(_M_IX86) || defined(_M_AMD64)

VOID NTAPI KiCpuId(PCPU_INFO CpuInfo, ULONG Function)
{
    _saved_KiCpuId(CpuInfo, Function);
}

VOID NTAPI KiCpuIdEx(PCPU_INFO CpuInfo, ULONG Function, ULONG SubFunction)
{
    _saved_KiCpuIdEx(CpuInfo, Function, SubFunction);
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
PKPCR NTAPI KeGetPcr(VOID)
{
    return _saved_KeGetPcr();
}
#endif

/* FASTCALL wrappers will be added here as needed after cargo check */

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

void NTAPI KiSatisfyObjectWait(PKMUTANT Object, PKTHREAD Thread)
{
    _tramp_KiSatisfyObjectWait(Object, Thread);
}

void NTAPI KiSatisfyMutantWait(PKMUTANT Object, PKTHREAD Thread)
{
    _tramp_KiSatisfyMutantWait(Object, Thread);
}

void NTAPI KiSatisfyNonMutantWait(PKMUTANT Object)
{
    _tramp_KiSatisfyNonMutantWait(Object);
}

void NTAPI KiAddThreadToWaitList(PKTHREAD Thread, BOOLEAN Swappable)
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
void NTAPI KxSingleThreadWait(
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

void NTAPI KxDelayThreadWait(
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

void NTAPI KxMultiThreadWait(
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
void NTAPI YieldProcessor(void)
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

/* --- KiSystemStartup for ARM64 ---
 * ARM64 uses KiSystemStartupBootStack, but some code references KiSystemStartup.
 * Provide a thunk. */
#ifdef _M_ARM64
extern DECLSPEC_NORETURN VOID NTAPI KiSystemStartupBootStack(VOID);
DECLSPEC_NORETURN VOID NTAPI KiSystemStartup(PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    /* ARM64 ignores LoaderBlock here — it's passed via global */
    KiSystemStartupBootStack();
}
#endif

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
PKTRAP_FRAME NTAPI KiGetLinkedTrapFrame(IN PKTRAP_FRAME TrapFrame)
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
void NTAPI KxQueueThreadWait(
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
void NTAPI KxUnwaitThread_impl(DISPATCHER_HEADER *Header, KPRIORITY Increment)
{
    _saved_KxUnwaitThread(Header, Increment);
}

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
        KiReleaseDispatcherLockFromSynchLevel();

        /* Release lock */
        KiReleaseApcLockFromSynchLevel(ApcLock);

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
VOID NTAPI RtlFillMemoryUlong(PVOID Destination, SIZE_T Length, ULONG Pattern)
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
/* For now, provide a stdcall wrapper with a different linker name using asm. */
__asm__(
    ".globl _KfRaiseIrql@4\n"
    "_KfRaiseIrql@4:\n"
    "  movl 4(%esp), %ecx\n"     /* stdcall arg → fastcall ecx */
    "  jmp @KfRaiseIrql@4\n"    /* tail-call the fastcall HAL import */
);

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

/* --- Ke386FnInit, Ke386SaveFpuState, Ke386RestoreFpuState, Ke386ClearFpExceptions ---
 * Function-like macros in intrin_i.h. Cannot Phase 1 rename (macros with parens).
 * Cannot define C functions (conflicts with FORCEINLINE declarations in headers).
 * Use global asm to provide stdcall symbols directly.
 * Ke386SaveFpuState/RestoreFpuState use fxsave/fxrstor (KeI386FxsrPresent is always TRUE
 * on Pentium+ which is our minimum target). */
__asm__(
    ".globl _Ke386FnInit@0\n"
    "_Ke386FnInit@0:\n"
    "  fninit\n"
    "  ret\n"
);

__asm__(
    ".globl _Ke386SaveFpuState@4\n"
    "_Ke386SaveFpuState@4:\n"
    "  movl 4(%esp), %eax\n"
    "  fxsave (%eax)\n"
    "  ret $4\n"
);

__asm__(
    ".globl _Ke386RestoreFpuState@4\n"
    "_Ke386RestoreFpuState@4:\n"
    "  movl 4(%esp), %eax\n"
    "  fxrstor (%eax)\n"
    "  ret $4\n"
);

__asm__(
    ".globl _Ke386ClearFpExceptions@0\n"
    "_Ke386ClearFpExceptions@0:\n"
    "  fnclex\n"
    "  ret\n"
);

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

/* --- KfLowerIrql ---
 * HAL FASTCALL function. Rust calls it as extern "system" (stdcall) producing
 * _KfLowerIrql@4. HAL exports @KfLowerIrql@4 (fastcall). Provide trampoline. */
__asm__(
    ".globl _KfLowerIrql@4\n"
    "_KfLowerIrql@4:\n"
    "  movl 4(%esp), %ecx\n"     /* stdcall arg → fastcall ecx */
    "  jmp @KfLowerIrql@4\n"    /* tail-call the fastcall HAL import */
);

/* --- __rdtsc ---
 * MSVC compiler intrinsic. Provide callable symbol for i386. */
__asm__(
    ".globl ___rdtsc@0\n"
    "___rdtsc@0:\n"
    "  rdtsc\n"
    "  ret\n"
);

/* --- __readeflags ---
 * MSVC compiler intrinsic. Provide callable __readeflags_shim symbol.
 * Cannot redefine __readeflags directly (conflicts with intrin.h declaration),
 * so we use a global asm alias. */
__asm__(
    ".globl ___readeflags@0\n"
    "___readeflags@0:\n"
    "  pushfl\n"
    "  popl %eax\n"
    "  ret\n"
);

/* --- _disable ---
 * MSVC compiler intrinsic (cli instruction). Provide callable symbol.
 * extern "system" on i386 = stdcall = __disable@0 */
__asm__(
    ".globl __disable@0\n"
    "__disable@0:\n"
    "  cli\n"
    "  ret\n"
);

/* --- _enable ---
 * MSVC compiler intrinsic (sti instruction). Provide callable symbol.
 * extern "system" on i386 = stdcall = __enable@0 */
__asm__(
    ".globl __enable@0\n"
    "__enable@0:\n"
    "  sti\n"
    "  ret\n"
);

/* --- __writeeflags ---
 * MSVC compiler intrinsic. Provide callable symbol.
 * extern "system" on i386 = stdcall = ___writeeflags@4 */
__asm__(
    ".globl ___writeeflags@4\n"
    "___writeeflags@4:\n"
    "  movl 4(%esp), %eax\n"
    "  pushl %eax\n"
    "  popfl\n"
    "  ret $4\n"
);

/* --- __debugbreak ---
 * MSVC compiler intrinsic (int3 instruction).
 * extern "system" on i386 = stdcall = ___debugbreak@0 */
__asm__(
    ".globl ___debugbreak@0\n"
    "___debugbreak@0:\n"
    "  int $3\n"
    "  ret\n"
);

/* --- __readcr0, __readcr3, __readcr4 ---
 * MSVC compiler intrinsics. Provide callable stdcall symbols for i386.
 * extern "system" → ___readcr0@0 (triple underscore = stdcall prefix on __readcr0) */
__asm__(
    ".globl ___readcr0@0\n"
    "___readcr0@0:\n"
    "  movl %cr0, %eax\n"
    "  ret\n"
);
__asm__(
    ".globl ___writecr0@4\n"
    "___writecr0@4:\n"
    "  movl 4(%esp), %eax\n"
    "  movl %eax, %cr0\n"
    "  ret $4\n"
);
__asm__(
    ".globl ___readcr2@0\n"
    "___readcr2@0:\n"
    "  movl %cr2, %eax\n"
    "  ret\n"
);
__asm__(
    ".globl ___readcr3@0\n"
    "___readcr3@0:\n"
    "  movl %cr3, %eax\n"
    "  ret\n"
);
__asm__(
    ".globl ___readcr4@0\n"
    "___readcr4@0:\n"
    "  movl %cr4, %eax\n"
    "  ret\n"
);

/* --- __sgdt, __sidt ---
 * MSVC compiler intrinsics. Provide callable stdcall symbols.
 * __sgdt(Limit) / __sidt(Limit) — take pointer arg.
 * extern "system" → ___sgdt@4 / ___sidt@4 */
__asm__(
    ".globl ___sgdt@4\n"
    "___sgdt@4:\n"
    "  movl 4(%esp), %eax\n"
    "  sgdt (%eax)\n"
    "  ret $4\n"
);
__asm__(
    ".globl ___sidt@4\n"
    "___sidt@4:\n"
    "  movl 4(%esp), %eax\n"
    "  sidt (%eax)\n"
    "  ret $4\n"
);

/* --- __invlpg ---
 * MSVC compiler intrinsic. Provide callable stdcall symbol.
 * __invlpg(Address) — takes pointer arg. */
__asm__(
    ".globl ___invlpg@4\n"
    "___invlpg@4:\n"
    "  movl 4(%esp), %eax\n"
    "  invlpg (%eax)\n"
    "  ret $4\n"
);

/* --- __readfsdword ---
 * MSVC compiler intrinsic. Read dword from FS segment at offset.
 * __readfsdword(Offset) → ULONG */
__asm__(
    ".globl ___readfsdword@4\n"
    "___readfsdword@4:\n"
    "  movl 4(%esp), %ecx\n"
    "  movl %fs:(%ecx), %eax\n"
    "  ret $4\n"
);

/* --- __writefsdword ---
 * MSVC compiler intrinsic. Write dword to FS segment at offset.
 * __writefsdword(Offset, Value) */
__asm__(
    ".globl ___writefsdword@8\n"
    "___writefsdword@8:\n"
    "  movl 4(%esp), %ecx\n"
    "  movl 8(%esp), %eax\n"
    "  movl %eax, %fs:(%ecx)\n"
    "  ret $8\n"
);

/* --- KeInvalidateTlbEntry --- FORCEINLINE in internal/i386/ke.h */
VOID NTAPI KeInvalidateTlbEntry(IN PVOID Address)
{
    _saved_KeInvalidateTlbEntry(Address);
}

/* --- KiCheckForApcDelivery --- FORCEINLINE in internal/i386/ke.h */
VOID NTAPI KiCheckForApcDelivery(IN PKTRAP_FRAME TrapFrame)
{
    _saved_KiCheckForApcDelivery(TrapFrame);
}

/* --- KiGetGdtEntry --- FORCEINLINE in internal/i386/intrin_i.h */
PKGDTENTRY NTAPI KiGetGdtEntry(IN PVOID pGdt, IN USHORT Selector)
{
    return _saved_KiGetGdtEntry(pGdt, Selector);
}

/* --- KiSetGdtDescriptorBase --- FORCEINLINE in internal/i386/intrin_i.h */
VOID NTAPI KiSetGdtDescriptorBase(IN PKGDTENTRY Entry, IN ULONG Base)
{
    _saved_KiSetGdtDescriptorBase(Entry, Base);
}

/* --- InterlockedBitTestAndSet / InterlockedBitTestAndReset ---
 * Compiler intrinsic wrappers. Use bts/btr inline asm.
 * extern "system" → _InterlockedBitTestAndSet@8 */
__asm__(
    ".globl _InterlockedBitTestAndSet@8\n"
    "_InterlockedBitTestAndSet@8:\n"
    "  movl 4(%esp), %ecx\n"   /* Base pointer */
    "  movl 8(%esp), %edx\n"   /* Bit number */
    "  xorl %eax, %eax\n"
    "  lock btsl %edx, (%ecx)\n"
    "  setc %al\n"
    "  ret $8\n"
);
__asm__(
    ".globl _InterlockedBitTestAndReset@8\n"
    "_InterlockedBitTestAndReset@8:\n"
    "  movl 4(%esp), %ecx\n"   /* Base pointer */
    "  movl 8(%esp), %edx\n"   /* Bit number */
    "  xorl %eax, %eax\n"
    "  lock btrl %edx, (%ecx)\n"
    "  setc %al\n"
    "  ret $8\n"
);

/* --- KeMemoryBarrier ---
 * FORCEINLINE in xdk/x86/ke.h. Uses xchg [barrier], eax (full fence).
 * Bindgen declares it as extern "C" → _KeMemoryBarrier (cdecl, no @0). */
static LONG _KeMemoryBarrier_dummy;
void __cdecl KeMemoryBarrier(void)
{
    long dummy;
    __asm__ __volatile__("xchgl %0, %1"
        : "=r"(dummy), "=m"(_KeMemoryBarrier_dummy)
        : "0"(0), "m"(_KeMemoryBarrier_dummy)
        : "memory");
}

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
    _saved_KiEndInterrupt(Irql, TrapFrame);
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
 * FORCEINLINE with inline asm: fxrstor (misnamed "Store" = restore). */
void NTAPI Ke386FxStore(IN PVOID SaveArea)
{
    __asm__ __volatile__("fxrstor (%0)" : : "r"(SaveArea) : "memory");
}

/* --- Ke386GetFs ---
 * Macro with inline asm: movl %fs, reg. Returns USHORT. */
USHORT NTAPI Ke386GetFs(void)
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

#endif /* _M_IX86 (i386-specific shims) */
