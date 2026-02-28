/*
 * tier1_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust crate calls via extern declarations.
 *
 * Covers Tier 1 subsystems: lpc, dbgk, po, inbv, wmi, vdm, vf, rtl.
 * Common functions (KeEnterCriticalRegion, PsGetCurrentThread, push locks, etc.)
 * are provided by ob/ob_rust_shims.c.
 */

/* === Phase 1: Rename FORCEINLINE static inline functions before headers ===
 * NOTE: KeInitializeGuardedMutex etc. are macros in ntoskrnl.h that redirect
 * to _KeInitializeGuardedMutex (FORCEINLINE). We can't do Phase 1 rename on
 * macros — use trampoline pattern in Phase 3 instead.
 *
 * LpcpAllocateFromPortZone etc. are __inline in lpc_x.h — Phase 1 rename. */
#define LpcpAllocateFromPortZone _orig_LpcpAllocateFromPortZone
#define LpcpGetMessageFromThread _orig_LpcpGetMessageFromThread
#define LpcpGetPortFromThread _orig_LpcpGetPortFromThread
#define LpcpSetPortToThread _orig_LpcpSetPortToThread
#define LpcpGetDataInfoFromMessage _orig_LpcpGetDataInfoFromMessage
#define ExFreeToPagedLookasideList _orig_ExFreeToPagedLookasideList
#define InsertTailList _orig_InsertTailList
#define RtlInitEmptyAnsiString _orig_RtlInitEmptyAnsiString
#define IoGetNextIrpStackLocation _orig_IoGetNextIrpStackLocation
#define IoGetCurrentIrpStackLocation _orig_IoGetCurrentIrpStackLocation
#define IoSetNextIrpStackLocation _orig_IoSetNextIrpStackLocation
#define IoSkipCurrentIrpStackLocation _orig_IoSkipCurrentIrpStackLocation
#define IoSetCancelRoutine _orig_IoSetCancelRoutine
#define IoMarkIrpPending _orig_IoMarkIrpPending
#define IoSetCompletionRoutine _orig_IoSetCompletionRoutine

/* === Phase 2: Include headers === */

/* Suppress DPRINT (used by LPCTRACE in lpc_x.h wait macros) since this
 * file is PCH-skipped and doesn't have debug.h macros. */
#ifndef DPRINT
#define DPRINT(...) do {} while(0)
#endif

#include <ntoskrnl.h>

/* === Phase 3: Trampolines for macro-defined functions === */

/* HalHaltSystem: macro in halfuncs.h (HALPRIVATEDISPATCH->HalHaltSystem) */
static __inline void _tramp_HalHaltSystem(void)
{
    HalHaltSystem();
}

/* HalSetWakeEnable: macro in halfuncs.h (HALPRIVATEDISPATCH->HalSetWakeEnable) */
static __inline void _tramp_HalSetWakeEnable(BOOLEAN NewState)
{
    HalSetWakeEnable(NewState);
}

/* HalEndOfBoot: macro in haltypes.h (HALDISPATCH->HalEndOfBoot) */
static __inline void _tramp_HalEndOfBoot(void)
{
    HalEndOfBoot();
}

/* ProbeForWriteUlong: macro in probe.h */
static __inline void _tramp_ProbeForWriteUlong(PULONG Address)
{
    ProbeForWriteUlong(Address);
}

/* ProbeForWritePointer: macro in probe.h */
static __inline void _tramp_ProbeForWritePointer(PVOID *Address)
{
    ProbeForWritePointer(Address);
}

/* IoGetDevObjExtension: macro in io.h */
static __inline PEXTENDED_DEVOBJ_EXTENSION _tramp_IoGetDevObjExtension(PDEVICE_OBJECT DevObj)
{
    return IoGetDevObjExtension(DevObj);
}

/* SeDeleteClientSecurity: macro in sefuncs.h */
static __inline void _tramp_SeDeleteClientSecurity(PSECURITY_CLIENT_CONTEXT ClientContext)
{
    SeDeleteClientSecurity(ClientContext);
}

/* ObReferenceObject: macro wrapping ObfReferenceObject */
static __inline LONG_PTR _tramp_ObReferenceObject(PVOID Object)
{
    return ObReferenceObject(Object);
}

/* NtCurrentProcess: macro ((HANDLE)(LONG_PTR)-1) */
static __inline HANDLE _tramp_NtCurrentProcess(void)
{
    return NtCurrentProcess();
}

/* PsGetCurrentProcess: macro wrapping IoGetCurrentProcess */
static __inline PEPROCESS _tramp_PsGetCurrentProcess(void)
{
    return PsGetCurrentProcess();
}

/* RtlUnicodeStringToOemSize: macro */
static __inline ULONG _tramp_RtlUnicodeStringToOemSize(PCUNICODE_STRING UnicodeString)
{
    return RtlUnicodeStringToOemSize(UnicodeString);
}

/* LPC wait macros from lpc_x.h:
 * These are block macros that set a local 'Status' variable. We wrap them
 * as functions that return the status. */
static __inline NTSTATUS _tramp_LpcpReceiveWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    NTSTATUS Status;
    LpcpReceiveWait(Semaphore, WaitMode);
    return Status;
}

static __inline NTSTATUS _tramp_LpcpReplyWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    NTSTATUS Status;
    LpcpReplyWait(Semaphore, WaitMode);
    return Status;
}

static __inline NTSTATUS _tramp_LpcpConnectWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    NTSTATUS Status;
    LpcpConnectWait(Semaphore, WaitMode);
    return Status;
}

static __inline void _tramp_LpcpCompleteWait(PVOID Semaphore)
{
    LpcpCompleteWait(Semaphore);
}

#ifdef _M_IX86
/* KeRaiseIrql/KeLowerIrql: macros wrapping KfRaiseIrql/KfLowerIrql on i386. */
static __inline void _tramp_KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql)
{
    KeRaiseIrql(NewIrql, OldIrql);
}

static __inline void _tramp_KeLowerIrql(KIRQL NewIrql)
{
    KeLowerIrql(NewIrql);
}
#endif

/* === Phase 4: Undef macros === */

#undef CONTAINING_RECORD
#undef ProbeForWriteUlong
#undef ProbeForWritePointer
#undef IoGetDevObjExtension
#undef SeDeleteClientSecurity
#undef ObReferenceObject
#undef NtCurrentProcess
#undef PsGetCurrentProcess
#undef RtlUnicodeStringToOemSize
#undef LpcpReceiveWait
#undef LpcpReplyWait
#undef LpcpConnectWait
#undef LpcpCompleteWait
#undef LpcpAllocateFromPortZone
#undef LpcpGetMessageFromThread
#undef LpcpGetPortFromThread
#undef LpcpSetPortToThread
#undef LpcpGetDataInfoFromMessage
#undef ExFreeToPagedLookasideList
#undef InsertTailList
#undef RtlInitEmptyAnsiString
#undef HalHaltSystem
#undef HalSetWakeEnable
#undef HalEndOfBoot
#undef IoGetNextIrpStackLocation
#undef IoGetCurrentIrpStackLocation
#undef IoSetNextIrpStackLocation
#undef IoSkipCurrentIrpStackLocation
#undef IoSetCancelRoutine
#undef IoMarkIrpPending
#undef IoSetCompletionRoutine

#ifdef _M_IX86
#undef KeRaiseIrql
#undef KeLowerIrql
#endif

/* === Phase 5: Real (non-static, non-inline) functions === */

/* --- CONTAINING_RECORD: C macro → extern function for Rust ---
 * Rust code calls CONTAINING_RECORD(address, type_size_or_null, field_offset_or_null).
 * The implementation subtracts field_offset from address (type_size is unused). */
PVOID NTAPI CONTAINING_RECORD(PVOID Address, SIZE_T TypeSize, SIZE_T FieldOffset)
{
    (void)TypeSize;
    return (PCHAR)Address - FieldOffset;
}

/* --- ProbeForWriteUlong: probe.h macro --- */
void NTAPI ProbeForWriteUlong(PULONG Address)
{
    _tramp_ProbeForWriteUlong(Address);
}

/* --- ProbeForWritePointer: probe.h macro --- */
void NTAPI ProbeForWritePointer(PVOID *Address)
{
    _tramp_ProbeForWritePointer(Address);
}

/* --- IoGetDevObjExtension: macro in io.h --- */
PEXTENDED_DEVOBJ_EXTENSION NTAPI IoGetDevObjExtension(PDEVICE_OBJECT DeviceObject)
{
    return _tramp_IoGetDevObjExtension(DeviceObject);
}

/* --- SeDeleteClientSecurity: macro in sefuncs.h --- */
void NTAPI SeDeleteClientSecurity(PSECURITY_CLIENT_CONTEXT ClientContext)
{
    _tramp_SeDeleteClientSecurity(ClientContext);
}

/* --- ObReferenceObject: macro wrapping ObfReferenceObject --- */
LONG_PTR NTAPI ObReferenceObject(PVOID Object)
{
    return _tramp_ObReferenceObject(Object);
}

/* --- NtCurrentProcess: macro ((HANDLE)(LONG_PTR)-1) --- */
HANDLE NTAPI NtCurrentProcess(void)
{
    return _tramp_NtCurrentProcess();
}

/* --- PsGetCurrentProcess: macro wrapping IoGetCurrentProcess --- */
PEPROCESS NTAPI PsGetCurrentProcess(void)
{
    return _tramp_PsGetCurrentProcess();
}

/* --- RtlUnicodeStringToOemSize: macro wrapping RtlxUnicodeStringToOemSize --- */
ULONG NTAPI RtlUnicodeStringToOemSize(PCUNICODE_STRING UnicodeString)
{
    return _tramp_RtlUnicodeStringToOemSize(UnicodeString);
}

/* --- RtlInitEmptyAnsiString: FORCEINLINE in rtlfuncs.h --- */
void RtlInitEmptyAnsiString(PANSI_STRING AnsiString, PCHAR Buffer, USHORT MaximumLength)
{
    _orig_RtlInitEmptyAnsiString(AnsiString, Buffer, MaximumLength);
}

/* --- InsertTailList: FORCEINLINE in wdm.h --- */
void NTAPI InsertTailList(PLIST_ENTRY ListHead, PLIST_ENTRY Entry)
{
    _orig_InsertTailList(ListHead, Entry);
}

/* --- ExFreeToPagedLookasideList: FORCEINLINE in exfuncs.h --- */
void NTAPI ExFreeToPagedLookasideList(PPAGED_LOOKASIDE_LIST Lookaside, PVOID Entry)
{
    _orig_ExFreeToPagedLookasideList(Lookaside, Entry);
}

/* --- LPC wait macros from lpc_x.h ---
 * C macros are block statements that set a local Status variable.
 * Rust declares these as functions. */
NTSTATUS NTAPI LpcpReceiveWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    return _tramp_LpcpReceiveWait(Semaphore, WaitMode);
}

void NTAPI LpcpReplyWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    _tramp_LpcpReplyWait(Semaphore, WaitMode);
}

void NTAPI LpcpConnectWait(PVOID Semaphore, KPROCESSOR_MODE WaitMode)
{
    _tramp_LpcpConnectWait(Semaphore, WaitMode);
}

void NTAPI LpcpCompleteWait(PVOID Semaphore)
{
    _tramp_LpcpCompleteWait(Semaphore);
}

/* --- LPC FORCEINLINE functions from lpc_x.h --- */
PLPCP_MESSAGE NTAPI LpcpAllocateFromPortZone(void)
{
    return _orig_LpcpAllocateFromPortZone();
}

/* LpcpGetMessageFromThread / LpcpSetPortToThread / LpcpGetPortFromThread:
 * These are FORCEINLINE in lpc_x.h with NO calling convention specifier,
 * so they default to cdecl. Rust DDK (bindgen) declares them as extern "C".
 * Do NOT use NTAPI here — it would produce stdcall @N symbols on i386. */
PLPCP_MESSAGE LpcpGetMessageFromThread(PETHREAD Thread)
{
    return _orig_LpcpGetMessageFromThread(Thread);
}

PLPCP_PORT_OBJECT LpcpGetPortFromThread(PETHREAD Thread)
{
    return _orig_LpcpGetPortFromThread(Thread);
}

void LpcpSetPortToThread(PETHREAD Thread, PLPCP_PORT_OBJECT Port)
{
    _orig_LpcpSetPortToThread(Thread, Port);
}

PLPCP_DATA_INFO LpcpGetDataInfoFromMessage(PPORT_MESSAGE Message)
{
    return _orig_LpcpGetDataInfoFromMessage(Message);
}

/* --- HAL dispatch table macros --- */
VOID NTAPI HalHaltSystem(VOID)
{
    _tramp_HalHaltSystem();
}

VOID NTAPI HalSetWakeEnable(BOOLEAN NewState)
{
    _tramp_HalSetWakeEnable(NewState);
}

VOID NTAPI HalEndOfBoot(VOID)
{
    _tramp_HalEndOfBoot();
}

/* --- IO IRP stack macros from iofuncs.h --- */
PIO_STACK_LOCATION IoGetNextIrpStackLocation(PIRP Irp)
{
    return _orig_IoGetNextIrpStackLocation(Irp);
}

PIO_STACK_LOCATION IoGetCurrentIrpStackLocation(PIRP Irp)
{
    return _orig_IoGetCurrentIrpStackLocation(Irp);
}

void IoSetNextIrpStackLocation(PIRP Irp)
{
    _orig_IoSetNextIrpStackLocation(Irp);
}

void IoSkipCurrentIrpStackLocation(PIRP Irp)
{
    _orig_IoSkipCurrentIrpStackLocation(Irp);
}

PDRIVER_CANCEL IoSetCancelRoutine(PIRP Irp, PDRIVER_CANCEL CancelRoutine)
{
    return _orig_IoSetCancelRoutine(Irp, CancelRoutine);
}

void IoMarkIrpPending(PIRP Irp)
{
    _orig_IoMarkIrpPending(Irp);
}

void IoSetCompletionRoutine(
    PIRP Irp,
    PIO_COMPLETION_ROUTINE CompletionRoutine,
    PVOID Context,
    BOOLEAN InvokeOnSuccess,
    BOOLEAN InvokeOnError,
    BOOLEAN InvokeOnCancel)
{
    _orig_IoSetCompletionRoutine(Irp, CompletionRoutine, Context,
                                 InvokeOnSuccess, InvokeOnError, InvokeOnCancel);
}

/* --- KeRaiseIrqlToDpcLevel: FORCEINLINE on amd64 only ---
 * On amd64 it wraps KfRaiseIrql(DISPATCH_LEVEL) (FORCEINLINE).
 * On i386 it's exported from HAL as a real function.
 * On ARM64 it's a real function in ke/arm/irql.c. */
#ifdef _M_AMD64
KIRQL NTAPI KeRaiseIrqlToDpcLevel(void)
{
    return KfRaiseIrql(DISPATCH_LEVEL);
}
#endif

/* --- KeRaiseIrql / KeLowerIrql on i386 ---
 * On i386, KeRaiseIrql is a macro wrapping KfRaiseIrql (FASTCALL).
 * Rust declares these as extern "C" (cdecl). Provide cdecl wrappers. */
#ifdef _M_IX86
void NTAPI KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql)
{
    _tramp_KeRaiseIrql(NewIrql, OldIrql);
}

void NTAPI KeLowerIrql(KIRQL NewIrql)
{
    _tramp_KeLowerIrql(NewIrql);
}
#endif

/* --- KeGetCurrentThread: FORCEINLINE on amd64 ---
 * On amd64 it reads from KPCR->CurrentThread.
 * On i386 and ARM64 it's exported as a real function.
 * Rust kd64 subsystem needs it on amd64. */
#ifdef _M_AMD64
PKTHREAD NTAPI KeGetCurrentThread(void)
{
    return KeGetCurrentPrcb()->CurrentThread;
}
#endif


/* --- Stubs for sdk/lib/rtl symbols that only exist in user-mode ---
 * sdk/lib/rtl/critical.c references LdrpShutdownInProgress/ThreadId
 * (from ntdll loader). In kernel mode, we provide dummy definitions. */
BOOLEAN LdrpShutdownInProgress = FALSE;
HANDLE LdrpShutdownThreadId = NULL;

/* --- SRW Lock functions for kernel mode ---
 * sdk/lib/rtl/amd64/dynfntbl.c references these. They are defined in
 * sdk/lib/rtl/srw.c (rtl_vista library), but ntoskrnl doesn't link
 * rtl_vista. Provide stubs using spin locks for kernel mode. */
VOID NTAPI RtlAcquireSRWLockExclusive(PRTL_SRWLOCK SRWLock)
{
    /* Simple spin-lock implementation for kernel mode */
    while (_InterlockedCompareExchange((volatile LONG*)&SRWLock->Ptr, 1, 0) != 0)
    {
        YieldProcessor();
    }
}

VOID NTAPI RtlReleaseSRWLockExclusive(PRTL_SRWLOCK SRWLock)
{
    _InterlockedExchange((volatile LONG*)&SRWLock->Ptr, 0);
}

VOID NTAPI RtlAcquireSRWLockShared(PRTL_SRWLOCK SRWLock)
{
    /* Shared == exclusive for this simple stub */
    RtlAcquireSRWLockExclusive(SRWLock);
}

VOID NTAPI RtlReleaseSRWLockShared(PRTL_SRWLOCK SRWLock)
{
    RtlReleaseSRWLockExclusive(SRWLock);
}

/* RtlInitEmptyUnicodeString is provided by ob_rust_shims.c — do not duplicate */

/* --- DbgUnLoadImageSymbols: macro in ntddk.h wrapping DbgkUnLoadImageSymbols ---
 * Rust code calls DbgUnLoadImageSymbols directly. Provide shim wrapping the
 * actual kernel function. */
/* --- DbgUnLoadImageSymbols: declared as NTAPI in ndk/rtlfuncs.h ---
 * Rust code in poshtdwn.rs declares it as extern "system" (matching NTAPI).
 * No shim needed — the kernel already provides this symbol. */

/* --- VdmState: C macro → real global for Rust ---
 * In the kernel headers, VdmState is a macro: (PULONG)FIXED_NTVDMSTATE_LINEAR_PC_AT
 * (address 0x714). The Rust translation declares it as extern static mut *mut ULONG.
 * We #undef the macro and provide a real variable initialized to the fixed address. */
#ifdef _M_IX86
#undef VdmState
PULONG VdmState = (PULONG)FIXED_NTVDMSTATE_LINEAR_PC_AT;
#endif

