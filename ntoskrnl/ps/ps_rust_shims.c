/*
 * ps_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust crate calls via extern declarations,
 * plus global variable definitions that were in the original C PS files.
 *
 * PS-specific FORCEINLINE shims. Common functions (KeEnterCriticalRegion,
 * KeLeaveCriticalRegion, PsGetCurrentThread, push locks, ObpValidateAttributes,
 * PspClearCrossThreadFlag, etc.) are provided by ob/ob_rust_shims.c and
 * se/se_rust_shims.c.
 */

/* === Phase 1: Rename FORCEINLINE functions before headers === */
/* NOTE: Only FORCEINLINE function definitions can be renamed. C macros use
   the trampoline pattern instead (Phase 3). */

#define PspRunCreateThreadNotifyRoutines          _fi_PspRunCreateThreadNotifyRoutines
#define PspRunCreateProcessNotifyRoutines         _fi_PspRunCreateProcessNotifyRoutines
#define PspRunLoadImageNotifyRoutines             _fi_PspRunLoadImageNotifyRoutines
#define PspRunLegoRoutine                         _fi_PspRunLegoRoutine
#define PspLockProcessSecurityShared              _fi_PspLockProcessSecurityShared
#define PspUnlockProcessSecurityShared            _fi_PspUnlockProcessSecurityShared
#define PspLockProcessSecurityExclusive           _fi_PspLockProcessSecurityExclusive
#define PspUnlockProcessSecurityExclusive         _fi_PspUnlockProcessSecurityExclusive
#define PspLockThreadSecurityShared               _fi_PspLockThreadSecurityShared
#define PspUnlockThreadSecurityShared             _fi_PspUnlockThreadSecurityShared
#define PspLockThreadSecurityExclusive            _fi_PspLockThreadSecurityExclusive
#define PspUnlockThreadSecurityExclusive          _fi_PspUnlockThreadSecurityExclusive
#define NtCurrentTeb                              _fi_NtCurrentTeb
#define KeInitializeSpinLock                      _fi_KeInitializeSpinLock
#define ExReleasePushLockShared                   _fi_ExReleasePushLockShared
#define ExInitializeFastMutex                     _fi_ExInitializeFastMutex
/* ExInitializeRundownProtection is a macro (#define → _ExInitializeRundownProtection)
   in ex.h. Phase 1 rename gets overwritten by the header's macro. The trampoline
   approach (Phase 3) handles it correctly. Suppress the redefinition warning. */
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmacro-redefined"

/* These are FORCEINLINE on some arches but macros on others.
   Phase 1 rename prevents calling-convention conflicts on i386. */
#define KeGetContextPc                            _fi_KeGetContextPc
#define KeGetContextReturnRegister                _fi_KeGetContextReturnRegister
#define ExInitializeRundownProtection             _fi_ExInitializeRundownProtection
#define Ki386PerfEnd                              _fi_Ki386PerfEnd
/* IRQL functions are FORCEINLINE on amd64 (read/write CR8) but real HAL
   functions on i386/arm64.  Rename only on amd64 to avoid conflict. */
#ifdef _M_AMD64
#define KeGetCurrentIrql                          _fi_KeGetCurrentIrql
#define KeLowerIrql                               _fi_KeLowerIrql
#define KfRaiseIrql                               _fi_KfRaiseIrql
#endif

/* Rename static const arrays from ps_i.h so we can provide external-linkage copies */
#define PsProcessInfoClass                        _static_PsProcessInfoClass
#define PsThreadInfoClass                         _static_PsThreadInfoClass

/* === Phase 2: Include headers === */

#include <ntoskrnl.h>
#pragma clang diagnostic pop

/* === Phase 3: Trampolines for macro-defined functions === */

static __inline void _tramp_PspSetCrossThreadFlag(PETHREAD Thread, ULONG Flag)
{
    PspSetCrossThreadFlag(Thread, Flag);
}

static __inline void _tramp_PspSetProcessFlag(PEPROCESS Process, ULONG Flag)
{
    PspSetProcessFlag(Process, Flag);
}

static __inline void _tramp_PspClearProcessFlag(PEPROCESS Process, ULONG Flag)
{
    PspClearProcessFlag(Process, Flag);
}

static __inline ULONG _tramp_PspPrioritySeparationFromMask(ULONG Mask)
{
    return PspPrioritySeparationFromMask(Mask);
}

static __inline ULONG _tramp_PspQuantumTypeFromMask(ULONG Mask)
{
    return PspQuantumTypeFromMask(Mask);
}

static __inline ULONG _tramp_PspQuantumLengthFromMask(ULONG Mask)
{
    return PspQuantumLengthFromMask(Mask);
}

static __inline ULONG_PTR _tramp_KeGetContextPc(PCONTEXT Context)
{
    return KeGetContextPc(Context);
}

static __inline ULONG_PTR _tramp_KeGetContextReturnRegister(PCONTEXT Context)
{
    return KeGetContextReturnRegister(Context);
}

static __inline ULONG _tramp_InterlockedDecrementUL(PULONG Addend)
{
    return InterlockedDecrementUL(Addend);
}

static __inline void _tramp_KeQuerySystemTime(PLARGE_INTEGER CurrentTime)
{
    KeQuerySystemTime(CurrentTime);
}

static __inline void _tramp_ExInitializeRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    ExInitializeRundownProtection(RunRef);
}

static __inline void _tramp_Ki386PerfEnd(void)
{
    Ki386PerfEnd();
}

static __inline void _tramp_ARM64_UART_TRACE(PCSTR msg)
{
    ARM64_UART_TRACE(msg);
}

static __inline void _tramp_ARM64_UART_TRACE_HEX(PCSTR prefix, ULONG_PTR val)
{
    ARM64_UART_TRACE_HEX(prefix, val);
}

static __inline void _tramp_KeEnterGuardedRegion(void)
{
    KeEnterGuardedRegion();
}

static __inline void _tramp_KeLeaveGuardedRegion(void)
{
    KeLeaveGuardedRegion();
}

static __inline void _tramp_KeEnterGuardedRegionThread(PKTHREAD Thread)
{
    KeEnterGuardedRegionThread(Thread);
}

static __inline void _tramp_KeLeaveGuardedRegionThread(PKTHREAD Thread)
{
    KeLeaveGuardedRegionThread(Thread);
}

static __inline DECLSPEC_NORETURN void _tramp_ExRaiseStatus(NTSTATUS Status)
{
    ExRaiseStatus(Status);
}

static __inline PVOID _tramp_InterlockedCompareExchangePointer(
    PVOID volatile *Destination, PVOID Exchange, PVOID Comparand)
{
    return InterlockedCompareExchangePointer(Destination, Exchange, Comparand);
}

static __inline LONG _tramp_InterlockedCompareExchange(
    PLONG Destination, LONG Exchange, LONG Comparand)
{
    return InterlockedCompareExchange(Destination, Exchange, Comparand);
}

#ifdef _M_AMD64
static __inline void _tramp_KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql)
{
    KeRaiseIrql(NewIrql, OldIrql);
}
#endif

static __inline PKTRAP_FRAME _tramp_KeGetTrapFrame(PKTHREAD Thread)
{
    return KeGetTrapFrame(Thread);
}

static __inline PKEXCEPTION_FRAME _tramp_KeGetExceptionFrame(PKTHREAD Thread)
{
    return KeGetExceptionFrame(Thread);
}

/* === Phase 4: Undef all macros and Phase 1 renames === */

#undef PspSetCrossThreadFlag
#undef PspSetProcessFlag
#undef PspClearProcessFlag
#undef PspPrioritySeparationFromMask
#undef PspQuantumTypeFromMask
#undef PspQuantumLengthFromMask
#undef PspRunCreateThreadNotifyRoutines
#undef PspRunCreateProcessNotifyRoutines
#undef PspRunLoadImageNotifyRoutines
#undef PspRunLegoRoutine
#undef PspLockProcessSecurityShared
#undef PspUnlockProcessSecurityShared
#undef PspLockProcessSecurityExclusive
#undef PspUnlockProcessSecurityExclusive
#undef PspLockThreadSecurityShared
#undef PspUnlockThreadSecurityShared
#undef PspLockThreadSecurityExclusive
#undef PspUnlockThreadSecurityExclusive
#undef NtCurrentTeb
#undef KeInitializeSpinLock
#undef KeGetContextPc
#undef KeGetContextReturnRegister
#undef InterlockedDecrementUL
#undef KeQuerySystemTime
#undef ExInitializeRundownProtection
#undef Ki386PerfEnd
#undef ARM64_UART_TRACE
#undef ARM64_UART_TRACE_HEX
#undef KeEnterGuardedRegion
#undef KeLeaveGuardedRegion
#undef KeEnterGuardedRegionThread
#undef KeLeaveGuardedRegionThread
#undef ExRaiseStatus
#undef InterlockedCompareExchangePointer
#undef InterlockedCompareExchange
#undef KeGetTrapFrame
#undef KeGetExceptionFrame
#undef ExReleasePushLockShared
#undef ExInitializeFastMutex
#ifdef _M_AMD64
#undef KeGetCurrentIrql
#undef KeLowerIrql
#undef KfRaiseIrql
#undef KeRaiseIrql
#endif
#undef PsProcessInfoClass
#undef PsThreadInfoClass
#undef SharedUserData

/* ===================================================================
 * PsProcessInfoClass / PsThreadInfoClass — re-include ps_i.h with
 * external linkage (override 'static' to nothing).
 * The first inclusion (via ntoskrnl.h) created _static_* copies.
 * This second inclusion creates the real names with external linkage.
 * =================================================================== */
#pragma push_macro("static")
#define static
#include "internal/ps_i.h"
#pragma pop_macro("static")

/* ===================================================================
 * Global variables — previously defined in ps/ C files, now needed
 * because those C files are replaced by Rust (ntoskrnl-rust/src/ps/).
 *
 * Globals that are defined in Rust with #[no_mangle] are NOT here.
 * Only globals that Rust imports via extern "C" { static mut ... }
 * and are not provided by any other compiled C file.
 * =================================================================== */

/* --- From psmgr.c --- */
PVOID PspSystemDllBase;
PVOID PspSystemDllSection;
PVOID PspSystemDllEntryPoint;

UNICODE_STRING PsNtDllPathName =
    RTL_CONSTANT_STRING(L"\\SystemRoot\\System32\\ntdll.dll");

PHANDLE_TABLE PspCidTable;

PEPROCESS PsInitialSystemProcess = NULL;
PEPROCESS PsIdleProcess = NULL;
HANDLE PspInitialSystemProcessHandle = NULL;

ULONG PsMinimumWorkingSet, PsMaximumWorkingSet;
ULONG PspDefaultPagedLimit, PspDefaultNonPagedLimit, PspDefaultPagefileLimit;
BOOLEAN PspDoingGiveBacks;

PVOID KeUserPopEntrySListEnd;
PVOID KeUserPopEntrySListFault;
PVOID KeUserPopEntrySListResume;

GENERIC_MAPPING PspProcessMapping =
{
    STANDARD_RIGHTS_READ    | PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
    STANDARD_RIGHTS_WRITE   | PROCESS_CREATE_PROCESS    | PROCESS_CREATE_THREAD   |
    PROCESS_VM_OPERATION    | PROCESS_VM_WRITE          | PROCESS_DUP_HANDLE      |
    PROCESS_TERMINATE       | PROCESS_SET_QUOTA         | PROCESS_SET_INFORMATION |
    PROCESS_SUSPEND_RESUME,
    STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    PROCESS_ALL_ACCESS
};

GENERIC_MAPPING PspThreadMapping =
{
    STANDARD_RIGHTS_READ    | THREAD_GET_CONTEXT      | THREAD_QUERY_INFORMATION,
    STANDARD_RIGHTS_WRITE   | THREAD_TERMINATE        | THREAD_SUSPEND_RESUME    |
    THREAD_ALERT            | THREAD_SET_INFORMATION  | THREAD_SET_CONTEXT,
    STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    THREAD_ALL_ACCESS
};

/* --- From security.c --- */
PTOKEN PspBootAccessToken;

/* SharedUserData is a C macro: (KUSER_SHARED_DATA * const)KI_USER_SHARED_DATA.
 * Rust code declares it as extern static. Provide a real global. */
PKUSER_SHARED_DATA SharedUserData = (PKUSER_SHARED_DATA)KI_USER_SHARED_DATA;

/* ===================================================================
 * Phase 5: Real (non-static, non-inline) functions
 * All use NTAPI (__stdcall on i386) to match Rust extern "system"
 * =================================================================== */

/* --- Macro trampolines --- */

void PspSetCrossThreadFlag(PETHREAD Thread, ULONG Flag)
{
    _tramp_PspSetCrossThreadFlag(Thread, Flag);
}

void PspSetProcessFlag(PEPROCESS Process, ULONG Flag)
{
    _tramp_PspSetProcessFlag(Process, Flag);
}

void PspClearProcessFlag(PEPROCESS Process, ULONG Flag)
{
    _tramp_PspClearProcessFlag(Process, Flag);
}

ULONG PspPrioritySeparationFromMask(ULONG Mask)
{
    return _tramp_PspPrioritySeparationFromMask(Mask);
}

ULONG PspQuantumTypeFromMask(ULONG Mask)
{
    return _tramp_PspQuantumTypeFromMask(Mask);
}

ULONG PspQuantumLengthFromMask(ULONG Mask)
{
    return _tramp_PspQuantumLengthFromMask(Mask);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
ULONG_PTR KeGetContextPc(PCONTEXT Context)
{
    return _tramp_KeGetContextPc(Context);
}

ULONG_PTR KeGetContextReturnRegister(PCONTEXT Context)
{
    return _tramp_KeGetContextReturnRegister(Context);
}

ULONG InterlockedDecrementUL(PULONG Addend)
{
    return _tramp_InterlockedDecrementUL(Addend);
}

/* KeQuerySystemTime is a real function on i386 (ex/time.c) but a macro
   on amd64/arm64, so only provide the shim where it's a macro. */
#ifndef _M_IX86
VOID KeQuerySystemTime(PLARGE_INTEGER CurrentTime)
{
    _tramp_KeQuerySystemTime(CurrentTime);
}
#endif

/* FASTCALL — matches bindgen extern "fastcall" declaration on i386 */
VOID FASTCALL ExInitializeRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    _tramp_ExInitializeRundownProtection(RunRef);
}

/* IRQL functions — only needed on amd64 where they are FORCEINLINE.
   On i386/arm64, HAL provides the real symbols. */
#ifdef _M_AMD64
KIRQL KeGetCurrentIrql(VOID)
{
    return _fi_KeGetCurrentIrql();
}

VOID KeLowerIrql(KIRQL NewIrql)
{
    _fi_KeLowerIrql(NewIrql);
}

VOID KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql)
{
    _tramp_KeRaiseIrql(NewIrql, OldIrql);
}
#endif

VOID KeEnterGuardedRegion(void)
{
    _tramp_KeEnterGuardedRegion();
}

VOID KeLeaveGuardedRegion(void)
{
    _tramp_KeLeaveGuardedRegion();
}

VOID KeEnterGuardedRegionThread(PKTHREAD Thread)
{
    _tramp_KeEnterGuardedRegionThread(Thread);
}

VOID KeLeaveGuardedRegionThread(PKTHREAD Thread)
{
    _tramp_KeLeaveGuardedRegionThread(Thread);
}

DECLSPEC_NORETURN VOID ExRaiseStatus(NTSTATUS Status)
{
    _tramp_ExRaiseStatus(Status);
}

PVOID InterlockedCompareExchangePointer(
    PVOID volatile *Destination, PVOID Exchange, PVOID Comparand)
{
    return _tramp_InterlockedCompareExchangePointer(Destination, Exchange, Comparand);
}

/* Only needed on i386 — amd64/arm64 have it in ex/exintrin.c or as intrinsic */
#ifdef _M_IX86
LONG InterlockedCompareExchange(PLONG Destination, LONG Exchange, LONG Comparand)
{
    return _tramp_InterlockedCompareExchange(Destination, Exchange, Comparand);
}
#endif

PKTRAP_FRAME __cdecl KeGetTrapFrame(PKTHREAD Thread)
{
    return _tramp_KeGetTrapFrame(Thread);
}

PKEXCEPTION_FRAME KeGetExceptionFrame(PKTHREAD Thread)
{
    return _tramp_KeGetExceptionFrame(Thread);
}

/* --- FORCEINLINE function wrappers --- */

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID PspRunCreateThreadNotifyRoutines(PETHREAD CurrentThread, BOOLEAN Create)
{
    _fi_PspRunCreateThreadNotifyRoutines(CurrentThread, Create);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID PspRunCreateProcessNotifyRoutines(PEPROCESS CurrentProcess, BOOLEAN Create)
{
    _fi_PspRunCreateProcessNotifyRoutines(CurrentProcess, Create);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID PspRunLoadImageNotifyRoutines(PUNICODE_STRING FullImageName,
                                   HANDLE ProcessId,
                                   PIMAGE_INFO ImageInfo)
{
    _fi_PspRunLoadImageNotifyRoutines(FullImageName, ProcessId, ImageInfo);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID PspRunLegoRoutine(PKTHREAD Thread)
{
    _fi_PspRunLegoRoutine(Thread);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID PspLockProcessSecurityShared(PEPROCESS Process)
{
    _fi_PspLockProcessSecurityShared(Process);
}

VOID PspUnlockProcessSecurityShared(PEPROCESS Process)
{
    _fi_PspUnlockProcessSecurityShared(Process);
}

VOID PspLockProcessSecurityExclusive(PEPROCESS Process)
{
    _fi_PspLockProcessSecurityExclusive(Process);
}

VOID PspUnlockProcessSecurityExclusive(PEPROCESS Process)
{
    _fi_PspUnlockProcessSecurityExclusive(Process);
}

VOID PspLockThreadSecurityShared(PETHREAD Thread)
{
    _fi_PspLockThreadSecurityShared(Thread);
}

VOID PspUnlockThreadSecurityShared(PETHREAD Thread)
{
    _fi_PspUnlockThreadSecurityShared(Thread);
}

VOID PspLockThreadSecurityExclusive(PETHREAD Thread)
{
    _fi_PspLockThreadSecurityExclusive(Thread);
}

VOID PspUnlockThreadSecurityExclusive(PETHREAD Thread)
{
    _fi_PspUnlockThreadSecurityExclusive(Thread);
}

PTEB NtCurrentTeb(void)
{
    return _fi_NtCurrentTeb();
}

VOID KeInitializeSpinLock(PKSPIN_LOCK SpinLock)
{
    _fi_KeInitializeSpinLock(SpinLock);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID ExReleasePushLockShared(PEX_PUSH_LOCK PushLock)
{
    _fi_ExReleasePushLockShared(PushLock);
}

/* __cdecl (no NTAPI) — matches bindgen extern "C" declaration */
VOID ExInitializeFastMutex(PFAST_MUTEX FastMutex)
{
    _fi_ExInitializeFastMutex(FastMutex);
}

/* --- Debug / perf stubs --- */

/* NTAPI — called from C code (kdapi.c), not from Rust */
VOID PspDumpThreads(BOOLEAN IncludeSystem)
{
    UNREFERENCED_PARAMETER(IncludeSystem);
}

/* __cdecl — matches bindgen extern "C" declaration */
VOID Ki386PerfEnd(void)
{
    _tramp_Ki386PerfEnd();
}

VOID NTAPI ARM64_UART_TRACE(PCSTR msg)
{
#ifdef _M_ARM64
    extern volatile PVOID Arm64UartKernelBase;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) { while (*msg) *u = (ULONG)*msg++; }
#else
    _tramp_ARM64_UART_TRACE(msg);
#endif
}

VOID NTAPI ARM64_UART_TRACE_HEX(PCSTR prefix, ULONG_PTR val)
{
#ifdef _M_ARM64
    char buf[32];
    static const char hex[] = "0123456789ABCDEF";
    int i;
    extern volatile PVOID Arm64UartKernelBase;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (!u) return;
    /* Print prefix */
    while (*prefix) *u = (ULONG)*prefix++;
    /* Print hex value */
    *u = '0'; *u = 'x';
    for (i = (sizeof(val)*2 - 1); i >= 0; i--)
        *u = hex[(val >> (i*4)) & 0xF];
#else
    _tramp_ARM64_UART_TRACE_HEX(prefix, val);
#endif
}
