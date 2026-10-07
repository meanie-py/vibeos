/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Win10+-model WOW64 host layer — Phase A/B skeleton
 *              (docs/engineering/wow64-plan.md). This is OUR layer of the
 *              three-layer BTCpu architecture: the CPU layer below it is
 *              libwow64fex.dll (vendored FEX, contract-only), the kernel/
 *              loader layer above it is Phase C.
 *
 *              Phase A scope: exist, export the two symbols wow64fex imports,
 *              and make Wow64SystemServiceEx loudly report any syscall it is
 *              asked to thunk. The real 32→64 thunk table is Phase B.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <stdint.h>
#include "wow64_btcpu.h"

ULONG
WINAPI
DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

LONG
WINAPI
NtTerminateProcess(_In_ HANDLE ProcessHandle, _In_ LONG ExitStatus);

LONG
WINAPI
NtQueryInformationProcess(_In_ HANDLE ProcessHandle,
                          _In_ ULONG ProcessInformationClass,
                          _Out_ PVOID ProcessInformation,
                          _In_ ULONG ProcessInformationLength,
                          _Out_opt_ PULONG ReturnLength);

/* ntdll-only DLL: no kernel32 import (a WoW64 process must not pull 64-bit
 * kernel32 through us — Win10 keeps it out of wow64 processes too). */
#define WOW64_CURRENT_PROCESS ((HANDLE)(LONG_PTR)-1)
#define WOW64_CURRENT_THREAD  ((HANDLE)(LONG_PTR)-2)

/* Host program counter in the native CONTEXT (diagnostics only). */
#ifdef _M_ARM64
#define WOW64_HOST_PC(Ctx) ((PVOID)(Ctx)->Pc)
#else /* _M_AMD64 */
#define WOW64_HOST_PC(Ctx) ((PVOID)(Ctx)->Rip)
#endif

/* ---- ntdll surface we use (this DLL links ntdll only) ------------------- */

typedef struct _WOW64_UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR Buffer;
} WOW64_UNICODE_STRING;

typedef struct _WOW64_ANSI_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PSTR Buffer;
} WOW64_ANSI_STRING;

typedef struct _WOW64_OBJECT_ATTRIBUTES {
    ULONG Length;
    HANDLE RootDirectory;
    WOW64_UNICODE_STRING *ObjectName;
    ULONG Attributes;
    PVOID SecurityDescriptor;
    PVOID SecurityQualityOfService;
} WOW64_OBJECT_ATTRIBUTES;

typedef struct _WOW64_IO_STATUS_BLOCK {
    union { LONG Status; PVOID Pointer; };
    ULONG_PTR Information;
} WOW64_IO_STATUS_BLOCK;

LONG WINAPI LdrLoadDll(PWSTR SearchPath, PULONG Flags,
                       WOW64_UNICODE_STRING *Name, PVOID *BaseAddress);
LONG WINAPI LdrGetDllHandle(PWSTR Path, PULONG Unused,
                            WOW64_UNICODE_STRING *Name, PVOID *BaseAddress);
LONG WINAPI LdrGetProcedureAddress(PVOID BaseAddress, WOW64_ANSI_STRING *Name,
                                   ULONG Ordinal, PVOID *Address);
VOID WINAPI RtlInitUnicodeString(WOW64_UNICODE_STRING *Dest, PCWSTR Src);
VOID WINAPI RtlInitAnsiString(WOW64_ANSI_STRING *Dest, PCSTR Src);
LONG WINAPI RtlSetEnvironmentVariable(PWSTR *Environment,
                                      WOW64_UNICODE_STRING *Name,
                                      WOW64_UNICODE_STRING *Value);
PVOID WINAPI RtlAddVectoredExceptionHandler(ULONG First,
                                            PVECTORED_EXCEPTION_HANDLER Handler);
LONG WINAPI NtFreeVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                PSIZE_T RegionSize, ULONG FreeType);
LONG WINAPI NtReadVirtualMemory(HANDLE Process, PVOID BaseAddress,
                                PVOID Buffer, SIZE_T Size, PSIZE_T Done);
LONG WINAPI NtWriteVirtualMemory(HANDLE Process, PVOID BaseAddress,
                                 PVOID Buffer, SIZE_T Size, PSIZE_T Done);
/* Our own load address, captured in DllMain. Needed to translate a code
 * address in THIS wow64.dll into the same address in ANOTHER process's copy
 * (Wow64CreateForeignGuestThread). */
static PVOID Wow64pSelfBase;

LONG WINAPI NtAllocateVirtualMemory(HANDLE Process, PVOID *BaseAddress,
                                    ULONG_PTR ZeroBits, SIZE_T *RegionSize,
                                    ULONG AllocationType, ULONG Protect);
LONG WINAPI NtOpenFile(HANDLE *FileHandle, ACCESS_MASK DesiredAccess,
                       WOW64_OBJECT_ATTRIBUTES *ObjectAttributes,
                       WOW64_IO_STATUS_BLOCK *IoStatusBlock,
                       ULONG ShareAccess, ULONG OpenOptions);
LONG WINAPI NtCreateSection(HANDLE *SectionHandle, ACCESS_MASK DesiredAccess,
                            WOW64_OBJECT_ATTRIBUTES *ObjectAttributes,
                            PLARGE_INTEGER MaximumSize, ULONG PageProtection,
                            ULONG AllocationAttributes, HANDLE FileHandle);
LONG WINAPI NtMapViewOfSection(HANDLE SectionHandle, HANDLE ProcessHandle,
                               PVOID *BaseAddress, ULONG_PTR ZeroBits,
                               SIZE_T CommitSize, PLARGE_INTEGER SectionOffset,
                               SIZE_T *ViewSize, ULONG InheritDisposition,
                               ULONG AllocationType, ULONG Win32Protect);
LONG WINAPI NtClose(HANDLE Handle);
LONG WINAPI NtQueryInformationProcess(HANDLE, ULONG, PVOID, ULONG, PULONG);
LONG WINAPI NtProtectVirtualMemory(HANDLE ProcessHandle, PVOID *BaseAddress,
                                   SIZE_T *RegionSize, ULONG NewProtect,
                                   PULONG OldProtect);
LONG WINAPI NtFlushInstructionCache(HANDLE ProcessHandle, PVOID BaseAddress,
                                    SIZE_T Length);

typedef struct _WOW64_CLIENT_ID {
    HANDLE UniqueProcess;
    HANDLE UniqueThread;
} WOW64_CLIENT_ID;

/* #64: real 32-bit guest thread creation. RtlCreateUserThread builds the
 * NATIVE thread (native stack + native start context); the guest's own
 * 32-bit stack came from the guest's BaseCreateStack and arrives in the
 * INITIAL_TEB32 the marshaller hands us. */
LONG WINAPI RtlCreateUserThread(HANDLE ProcessHandle, PVOID SecurityDescriptor,
                                BOOLEAN CreateSuspended, ULONG StackZeroBits,
                                SIZE_T StackReserve, SIZE_T StackCommit,
                                PVOID StartAddress, PVOID Parameter,
                                PHANDLE ThreadHandle, WOW64_CLIENT_ID *ClientId);
PVOID WINAPI RtlAllocateHeap(PVOID HeapHandle, ULONG Flags, SIZE_T Size);
BOOLEAN WINAPI RtlFreeHeap(PVOID HeapHandle, ULONG Flags, PVOID BaseAddress);
VOID WINAPI RtlExitUserThread(LONG Status);

/* wow64_thunks.c — the wow64gen-generated 32->64 syscall thunk table. */
LONG Wow64DispatchSystemService(ULONG ServiceNumber, PULONG Arguments);

/* ---- WoW64/FEX contract constants (same as wow64probe / wow64shim.rs) --- */

#define WOW64_TEB_TLSSLOTS_OFFSET  0x1480  /* TEB64->TlsSlots[]              */
#define WOW64_TLS_WOW64INFO        10
#define WOW64_TLS_CPURESERVED      1
#define WOW64_INITBLOCK_NTDLL_OFF  72      /* SYSTEM_DLL_INIT_BLOCK.ntdll_handle */
#define WOW64_ZEROBITS_BELOW_2G    ((ULONG_PTR)0x7FFFFFFF) /* mask form */
#define WOW64_TEB_WOWTEBOFFSET     0x180c  /* raw Win10 slot (see MmCreateTeb) */
#define WOW64_MACHINE_I386         0x014C
#define WOW64_MACHINE_ARM64        0xAA64
#define WOW64_MACHINE_AMD64        0x8664

/* The CPU layer differs by host arch — a JIT on arm64 (FEX), a native
 * heaven's-gate mode switch on amd64 (wow64cpu) — but both export the identical
 * BTCpu* contract this file resolves by name, so only the DLL name and the
 * WOW64INFO NativeMachineType change. */
#ifdef _M_ARM64
#define WOW64_CPU_DLL          L"libwow64fex.dll"
#define WOW64_NATIVE_MACHINE   WOW64_MACHINE_ARM64
#else /* _M_AMD64 */
#define WOW64_CPU_DLL          L"wow64cpu.dll"
#define WOW64_NATIVE_MACHINE   WOW64_MACHINE_AMD64
#endif

/* FEX winternl.h WOW64INFO — TlsSlots[10] points here; BTCpuProcessInit
 * writes CpuFlags. */
typedef struct _WOW64INFO_FEX {
    ULONG NativeSystemPageSize;
    ULONG CpuFlags;
    ULONG Wow64ExecuteFlags;
    ULONG Unknown;
    ULONGLONG SectionHandle;
    ULONGLONG CrossProcessWorkList;
    USHORT NativeMachineType;
    USHORT EmulatedMachineType;
} WOW64INFO_FEX;

static WOW64INFO_FEX Wow64pWow64Info;

static PVOID *Wow64pTlsSlots(VOID)
{
    return (PVOID *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET);
}

/* This thread's TEB32. MmCreateTeb allocates the TEB64 and the TEB32 as one
 * region and records the delta in TEB64's WowTebOffset slot; zero means the
 * thread has no 32-bit side at all, which is a bug in every caller here. */
static PBYTE Wow64pCurrentTeb32(VOID)
{
    PBYTE Teb = (PBYTE)NtCurrentTeb();
    LONG WowTebOffset = *(LONG *)(Teb + WOW64_TEB_WOWTEBOFFSET);

    return WowTebOffset ? Teb + WowTebOffset : NULL;
}

/*
 * ---- the win32k last-error channel --------------------------------------
 *
 * win32k reports failure by setting last-error and returning FALSE/0/NULL,
 * and it sets it with EngSetLastError, which writes
 * NtCurrentTeb()->LastErrorValue — in kernel mode, the thread's *64-bit* TEB.
 * A 32-bit guest's GetLastError() reads TEB32->LastErrorValue, a different
 * structure at WowTebOffset. So every error code win32k produced used to land
 * in a TEB the guest never reads: the app got the FALSE and never the reason,
 * across the whole USER/GDI surface. (The ntdll surface is unaffected — those
 * syscalls return an NTSTATUS and the guest's own ntdll32 converts it into its
 * own TEB32.)
 *
 * The fix is a mirror at the layer boundaries rather than WoW64 awareness
 * inside win32k, which would mean auditing every Eng* caller in the kernel.
 * One rule, four sites: **crossing into the kernel copies the guest's value
 * UP; crossing into the guest copies it DOWN.** That makes TEB32 and TEB64
 * behave as the single variable a native 64-bit process has, which is the only
 * definition of "correct" here that does not need a case-by-case ruling on
 * whose value wins.
 *
 *   win32k syscall entry   TEB32 -> TEB64        wow64_win32k.c
 *   win32k syscall exit    TEB64 -> TEB32 (*)    wow64_win32k.c
 *   callback into guest    TEB64 -> TEB32        wow64_callback.c
 *   callback return        TEB32 -> TEB64        wow64_callback.c
 *
 * Copying UP is not optional even though the bug is about codes coming DOWN.
 * win32k's own save-and-restore idiom (`Error = EngGetLastError(); ...;
 * EngSetLastError(Error);` in window.rs, menu.rs) exists to leave the CALLER's
 * last-error untouched; without the copy-up it faithfully preserves a stale
 * value from some earlier syscall instead of the guest's.
 *
 * The callback pair closes a hole the syscall pair CREATES: once entry copies
 * up, a guest window proc that makes any win32k syscall overwrites TEB64 in
 * the middle of the outer call. co_IntSendMessageTimeout sets ERROR_TIMEOUT
 * and BroadcastSystemMessage reads it back with EngGetLastError() AFTER the
 * send (message.rs) — and `co_` is exactly the prefix for functions that leave
 * the critical section and call into user mode.
 *
 * (*) The exit copy is conditional on the host value having CHANGED. With both
 * boundaries in place an unconditional copy-down is provably a no-op when
 * win32k did not write, so the test is really insurance against a boundary not
 * listed above — guest exception dispatch re-enters guest code without passing
 * through the callback path, and an unconditional copy-down would there clobber
 * a value the guest set and the kernel never saw. It costs one compare.
 *
 * Offsets are the ones the generated offset checks assert
 * (sdk/lib/rust/ntkrnl-sys/src/offset_check_{amd64,arm64}.rs: TEB
 * .LastErrorValue == 104; offset_check_i386.rs: == 52).
 */
#define WOW64_TEB64_LASTERROR 0x68
#define WOW64_TEB32_LASTERROR 0x34

/* TEB32 -> TEB64. Returns the value now held by both. */
ULONG Wow64LastErrorToHost(VOID)
{
    PBYTE Teb32 = Wow64pCurrentTeb32();
    ULONG Guest;

    if (!Teb32)
        return 0;

    Guest = *(ULONG *)(Teb32 + WOW64_TEB32_LASTERROR);
    *(ULONG *)((PBYTE)NtCurrentTeb() + WOW64_TEB64_LASTERROR) = Guest;
    return Guest;
}

/* TEB64 -> TEB32. */
VOID Wow64LastErrorToGuest(VOID)
{
    PBYTE Teb32 = Wow64pCurrentTeb32();

    if (!Teb32)
        return;

    *(ULONG *)(Teb32 + WOW64_TEB32_LASTERROR) =
        *(ULONG *)((PBYTE)NtCurrentTeb() + WOW64_TEB64_LASTERROR);
}

/* TEB64 -> TEB32, but only if the kernel actually wrote something. */
VOID Wow64LastErrorToGuestIfChanged(ULONG WasOnEntry)
{
    PBYTE Teb32 = Wow64pCurrentTeb32();
    ULONG Host;

    if (!Teb32)
        return;

    Host = *(ULONG *)((PBYTE)NtCurrentTeb() + WOW64_TEB64_LASTERROR);
    if (Host != WasOnEntry)
        *(ULONG *)(Teb32 + WOW64_TEB32_LASTERROR) = Host;
}

/* The CPU layer's exception entry: services LookupCache overcommit faults /
 * SMC / suspends, NtContinue-ing itself when handled. Every exception routes
 * here first (real wow64 wires this through Wow64PrepareForException). */
static LONG (WINAPI *Wow64pBTCpuResetToConsistentState)(EXCEPTION_POINTERS *);
/* Map notification for GUEST-initiated NtMapViewOfSection (called from the
 * wow64_manual.c marshaller): FEX must learn about every mapped image or its
 * dispatcher won't translate code from the new range. Non-static on purpose. */
LONG (WINAPI *Wow64pBTCpuNotifyMap)(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG);
/* NtContinue interception (wow64_manual.c): a guest NtContinue must load the
 * 32-bit CONTEXT into the CPU layer (FEX reloads it on simulate reentry) —
 * passing it to the host kernel's NtContinue rebuilds a garbage arm64 context
 * and resumes the host thread at a wild PC. WOW64_CONTEXT == i386 CONTEXT. */
LONG (WINAPI *Wow64pBTCpuSetContext)(HANDLE, HANDLE, PVOID, PVOID);
/* M8 callback bridge (wow64_callback.c): guest-context snapshot/restore +
 * nested Simulate re-entry for kernel->user callbacks. */
LONG (WINAPI *Wow64pBTCpuGetContext)(HANDLE, HANDLE, PVOID, PVOID);
VOID (WINAPI *Wow64pBTCpuSimulate)(VOID);
PVOID Wow64pKiUserCallbackDispatcher32;
/* E5: guest ntdll32's KiUserExceptionDispatcher — entry point for dispatching
 * exceptions INTO the guest's own 32-bit SEH (hardware faults via the VEH
 * redirect below; software raises via the NtRaiseException marshaller). */
PVOID Wow64pKiUserExceptionDispatcher32;
void Wow64pInstallUserCallbackTable(void);
/* The BOP bridge address (BTCpuGetBopCode), kept for map-time Wow64Transition
 * patching of guest DLLs that carry their own slot (gdi32/user32 embed the
 * win32ksys stubs; ntdll32 is patched at bootstrap). Non-static: the
 * NtMapViewOfSection marshaller (wow64_manual.c) patches each image as it
 * maps via Wow64pPatchTransitionSlot. */
PVOID Wow64pBopCode;

/* Extent of the ntdll32 image view, so the export walk over it is bounded by
 * the same rule as every other module's rather than trusted for being ours. */
static SIZE_T Wow64pNtdll32Size;
static ULONG_PTR Wow64pPeb32;  /* set in Wow64LdrpInitialize, for VEH diagnostics */

/* ---- the rest of the CPU layer's memory-notification contract ------------ *
 * See wow64_btcpu.h for why these matter and why the Before/After pairing is
 * load-bearing. All optional: a CPU layer that emulates nothing (amd64's native
 * wow64cpu) leaves them NULL and every wrapper below becomes a no-op. */
static VOID (WINAPI *Wow64pBTCpuNotifyMemoryAlloc)(PVOID, SIZE_T, ULONG, ULONG, BOOL, ULONG);
static VOID (WINAPI *Wow64pBTCpuNotifyMemoryProtect)(PVOID, SIZE_T, ULONG, BOOL, ULONG);
static VOID (WINAPI *Wow64pBTCpuNotifyMemoryFree)(PVOID, SIZE_T, ULONG, BOOL, ULONG);
static VOID (WINAPI *Wow64pBTCpuNotifyMemoryDirty)(PVOID, SIZE_T);
static VOID (WINAPI *Wow64pBTCpuNotifyUnmapViewOfSection)(PVOID, BOOL, ULONG);
static VOID (WINAPI *Wow64pBTCpuNotifyReadFile)(HANDLE, PVOID, SIZE_T, BOOL, LONG);
static VOID (WINAPI *Wow64pBTCpuNotifyProcessExecuteFlagsChange)(ULONG);
static VOID (WINAPI *Wow64pBTCpuFlushInstructionCache2)(const void *, SIZE_T);

/* ProcessBasicInformation, enough of it to read UniqueProcessId. */
typedef struct _WOW64_PROCESS_BASIC_INFORMATION {
    LONG ExitStatus;
    PVOID PebBaseAddress;
    ULONG_PTR AffinityMask;
    LONG BasePriority;
    ULONG_PTR UniqueProcessId;
    ULONG_PTR InheritedFromUniqueProcessId;
} WOW64_PROCESS_BASIC_INFORMATION;

BOOLEAN Wow64IsCurrentProcess(HANDLE Process)
{
    static ULONG_PTR SelfPid;   /* 0 until the first non-pseudo-handle query */
    WOW64_PROCESS_BASIC_INFORMATION Pbi;

    if (Process == WOW64_CURRENT_PROCESS)
        return TRUE;
    if (!Process)
        return FALSE;

    if (!SelfPid)
    {
        if (NtQueryInformationProcess(WOW64_CURRENT_PROCESS, 0 /* Basic */,
                                      &Pbi, sizeof(Pbi), NULL) < 0)
            return FALSE;
        SelfPid = Pbi.UniqueProcessId;
    }
    if (NtQueryInformationProcess(Process, 0 /* Basic */, &Pbi, sizeof(Pbi),
                                  NULL) < 0)
        return FALSE;
    return (BOOLEAN)(Pbi.UniqueProcessId == SelfPid);
}

VOID Wow64NotifyAllocBefore(PVOID Base, SIZE_T Size, ULONG Type, ULONG Protect)
{
    if (Wow64pBTCpuNotifyMemoryAlloc)
        Wow64pBTCpuNotifyMemoryAlloc(Base, Size, Type, Protect, FALSE, 0);
}

VOID Wow64NotifyAllocAfter(PVOID Base, SIZE_T Size, ULONG Type, ULONG Protect,
                           LONG Status)
{
    if (Wow64pBTCpuNotifyMemoryAlloc)
        Wow64pBTCpuNotifyMemoryAlloc(Base, Size, Type, Protect, TRUE, Status);
}

VOID Wow64NotifyProtectBefore(PVOID Base, SIZE_T Size, ULONG NewProtect)
{
    if (Wow64pBTCpuNotifyMemoryProtect)
        Wow64pBTCpuNotifyMemoryProtect(Base, Size, NewProtect, FALSE, 0);
}

VOID Wow64NotifyProtectAfter(PVOID Base, SIZE_T Size, ULONG NewProtect,
                             LONG Status)
{
    if (Wow64pBTCpuNotifyMemoryProtect)
        Wow64pBTCpuNotifyMemoryProtect(Base, Size, NewProtect, TRUE, Status);
}

VOID Wow64NotifyFreeBefore(PVOID Base, SIZE_T Size, ULONG FreeType)
{
    if (Wow64pBTCpuNotifyMemoryFree)
        Wow64pBTCpuNotifyMemoryFree(Base, Size, FreeType, FALSE, 0);
}

VOID Wow64NotifyFreeAfter(PVOID Base, SIZE_T Size, ULONG FreeType,
                          LONG Status)
{
    if (Wow64pBTCpuNotifyMemoryFree)
        Wow64pBTCpuNotifyMemoryFree(Base, Size, FreeType, TRUE, Status);
}

VOID Wow64NotifyUnmapBefore(PVOID Base)
{
    if (Wow64pBTCpuNotifyUnmapViewOfSection)
        Wow64pBTCpuNotifyUnmapViewOfSection(Base, FALSE, 0);
}

VOID Wow64NotifyUnmapAfter(PVOID Base, LONG Status)
{
    if (Wow64pBTCpuNotifyUnmapViewOfSection)
        Wow64pBTCpuNotifyUnmapViewOfSection(Base, TRUE, Status);
}

VOID Wow64NotifyReadFileBefore(HANDLE File, PVOID Buffer, SIZE_T Length)
{
    if (Wow64pBTCpuNotifyReadFile)
        Wow64pBTCpuNotifyReadFile(File, Buffer, Length, FALSE, 0);
}

VOID Wow64NotifyReadFileAfter(HANDLE File, PVOID Buffer, SIZE_T Length,
                              LONG Status)
{
    if (Wow64pBTCpuNotifyReadFile)
        Wow64pBTCpuNotifyReadFile(File, Buffer, Length, TRUE, Status);
}

VOID Wow64NotifyMemoryDirty(PVOID Base, SIZE_T Size)
{
    if (Wow64pBTCpuNotifyMemoryDirty)
        Wow64pBTCpuNotifyMemoryDirty(Base, Size);
}

VOID Wow64NotifyFlushInstructionCache(PVOID Base, SIZE_T Size)
{
    if (Wow64pBTCpuFlushInstructionCache2)
        Wow64pBTCpuFlushInstructionCache2(Base, Size);
}

VOID Wow64SetExecuteFlags(ULONG Flags)
{
    Wow64pWow64Info.Wow64ExecuteFlags = Flags;
    if (Wow64pBTCpuNotifyProcessExecuteFlagsChange)
        Wow64pBTCpuNotifyProcessExecuteFlagsChange(Flags);
}

VOID Wow64pResolveNotifyContract(PVOID CpuDll)
{
    WOW64_ANSI_STRING ProcName;
    ULONG Wired = 0;

#define WOW64_RESOLVE_OPT(var, name)                                          \
    do {                                                                      \
        PVOID Addr = NULL;                                                    \
        RtlInitAnsiString(&ProcName, name);                                   \
        if (LdrGetProcedureAddress(CpuDll, &ProcName, 0, &Addr) >= 0 && Addr) \
        {                                                                     \
            *(PVOID *)&(var) = Addr;                                          \
            Wired++;                                                          \
        }                                                                     \
    } while (0)

    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyMemoryAlloc, "BTCpuNotifyMemoryAlloc");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyMemoryProtect, "BTCpuNotifyMemoryProtect");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyMemoryFree, "BTCpuNotifyMemoryFree");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyMemoryDirty, "BTCpuNotifyMemoryDirty");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyUnmapViewOfSection, "BTCpuNotifyUnmapViewOfSection");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyReadFile, "BTCpuNotifyReadFile");
    WOW64_RESOLVE_OPT(Wow64pBTCpuNotifyProcessExecuteFlagsChange,
                      "BTCpuNotifyProcessExecuteFlagsChange");
    WOW64_RESOLVE_OPT(Wow64pBTCpuFlushInstructionCache2, "BTCpuFlushInstructionCache2");
#undef WOW64_RESOLVE_OPT

    /* 0 is a legitimate answer, not a defect: a native CPU layer executes the
     * guest directly and has no translation cache to keep honest. Report the
     * count either way so a JIT layer that silently stopped exporting one of
     * these is visible on serial instead of showing up as stale guest code. */
    DbgPrint("[FEX-WOW64C] memory-notification contract: %lu/8 entry points wired "
             "(0 = native CPU layer, nothing to invalidate)\n", Wired);
}

/* ---- #64: process-once state that the PER-THREAD entry also needs -------
 * Wow64LdrpInitialize used to be process-init and first-thread-init fused
 * together, and it kept the genuinely per-thread pieces (the CPU area, the
 * run CONTEXT) in function-local `static`s — exactly one thread's worth of
 * storage. A second guest thread would have run on the first one's CPU area,
 * which is the register file the CPU layer reads and rewrites on every mode
 * switch. These are the values that really are process-wide, hoisted so the
 * split-out thread entry can reach them. */
static PVOID Wow64pKiUserApcDispatcher32;
static PVOID Wow64pLdrInitThunk32;
static VOID (WINAPI *Wow64pBTCpuThreadInit)(VOID);

/* PEB64->ProcessHeap, for the per-thread CPU area. TEB64
 * ProcessEnvironmentBlock is at +0x60 and PEB64.ProcessHeap at +0x30 on both
 * 64-bit targets; this DLL deliberately imports ntdll only, so there is no
 * RtlGetProcessHeap to call. */
PVOID Wow64pProcessHeap(VOID)
{
    PBYTE Peb = *(PBYTE *)((PBYTE)NtCurrentTeb() + 0x60);
    return Peb ? *(PVOID *)(Peb + 0x30) : NULL;
}

/* M3b heap diagnostic: dump the guest process heap state at an unhandled fault.
 * FEX shares the host address space, so we read the 32-bit heap directly. The
 * loader near-null-writes when RtlAllocateHeap returns NULL, so this fires with
 * the heap in its failed state. Offsets are the ReactOS/i386 HEAP layout the
 * guest RtlAllocateHeap disasm uses (Flags +0x44, VMThreshold +0x60,
 * FreeListsHead +0x1cc). */
static void Wow64pDumpGuestHeap(void)
{
    ULONG Heap;
    PULONG H;
    if (!Wow64pPeb32)
        return;
    Heap = *(ULONG *)(Wow64pPeb32 + 0x18); /* PEB32->ProcessHeap */
    DbgPrint("[FEX-WOW64C] PEB32=%p ImageBase[+0x08]=%08lx Ldr[+0x0c]=%08lx ProcParams[+0x10]=%08lx ProcessHeap[+0x18]=%08lx NumHeaps[+0x88]=%08lx\n",
             (PVOID)Wow64pPeb32,
             *(ULONG *)(Wow64pPeb32 + 0x08), *(ULONG *)(Wow64pPeb32 + 0x0c),
             *(ULONG *)(Wow64pPeb32 + 0x10), Heap,
             *(ULONG *)(Wow64pPeb32 + 0x88));
    if (!Heap || Heap >= 0x80000000u)
        return;
    H = (PULONG)(ULONG_PTR)Heap;
    DbgPrint("[FEX-WOW64C] HEAP hdr[0..7]: %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx\n",
             H[0], H[1], H[2], H[3], H[4], H[5], H[6], H[7]);
    DbgPrint("[FEX-WOW64C] HEAP Flags[0x44]=%08lx VMThresh[0x60]=%08lx TotalFreeSize[0x54]=%08lx Committed[0x58]=%08lx\n",
             H[0x44/4], H[0x60/4], H[0x54/4], H[0x58/4]);
    DbgPrint("[FEX-WOW64C] HEAP FreeListHead[0x1cc]: Flink=%08lx Blink=%08lx (self=%08lx -> EMPTY if Flink==self)\n",
             H[0x1cc/4], H[0x1d0/4], Heap + 0x1cc);

    /* Guest x86 registers at the fault, from the CPU-area WOW64_CONTEXT (FEX
     * flushes it on the trap). Reveals which near-null store faulted and why. */
    {
        PBYTE CpuArea = (PBYTE)Wow64pTlsSlots()[WOW64_TLS_CPURESERVED];
        WOW64_CONTEXT *C = (WOW64_CONTEXT *)(CpuArea + 16);
        DbgPrint("[FEX-WOW64C] guest regs: eip=%08lx eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx esp=%08lx\n",
                 C->Eip, C->Eax, C->Ebx, C->Ecx, C->Edx, C->Esi, C->Edi, C->Ebp, C->Esp);
        if (C->Ebp && C->Ebp < 0x80000000u)
            DbgPrint("[FEX-WOW64C] guest [ebp+0x0c]=%08lx (RtlImageNtHeader arg / image base?)\n",
                     *(ULONG *)(ULONG_PTR)(C->Ebp + 0x0c));
    }
}

/* E5 — the i386 EXCEPTION_RECORD (0x50 bytes; every pointer field is 32-bit). */
typedef struct _WOW64_EXCEPTION_RECORD32 {
    ULONG ExceptionCode;
    ULONG ExceptionFlags;
    ULONG ExceptionRecord;          /* guest EXCEPTION_RECORD32* (chain) */
    ULONG ExceptionAddress;         /* guest PVOID */
    ULONG NumberParameters;
    ULONG ExceptionInformation[15];
} WOW64_EXCEPTION_RECORD32;

/*
 * E5 — build an i386 KiUserExceptionDispatcher frame on the guest stack:
 *   [Esp+0] = EXCEPTION_RECORD32*   [Esp+4] = CONTEXT32*
 * with both structs copied immediately above, everything below `Below`.
 * Shared by the VEH hardware-fault redirect (amd64) and the NtRaiseException
 * marshaller (both arches). Returns the new guest Esp, 0 on a bogus stack.
 */
ULONG Wow64pBuildGuestExceptionFrame(const WOW64_EXCEPTION_RECORD32 *Rec32,
                                     const WOW64_CONTEXT *Ctx32, ULONG Below)
{
    ULONG CtxA, RecA, Esp, i;

    Below &= ~15u;
    if (Below < 0x10000 || Below >= 0x80000000u)
        return 0;
    CtxA = Below - ((sizeof(WOW64_CONTEXT) + 15) & ~15u);
    RecA = CtxA - sizeof(WOW64_EXCEPTION_RECORD32);
    Esp  = RecA - 8;
    if (Esp < 0x10000)
        return 0;
    for (i = 0; i < sizeof(WOW64_CONTEXT); i++)
        *(PBYTE)(ULONG_PTR)(CtxA + i) = ((const BYTE *)Ctx32)[i];
    for (i = 0; i < sizeof(WOW64_EXCEPTION_RECORD32); i++)
        *(PBYTE)(ULONG_PTR)(RecA + i) = ((const BYTE *)Rec32)[i];
    *(ULONG *)(ULONG_PTR)(Esp + 0) = RecA;
    *(ULONG *)(ULONG_PTR)(Esp + 4) = CtxA;
    return Esp;
}

#ifndef _M_ARM64
extern LONG WINAPI NtContinue(CONTEXT *Context, BOOLEAN TestAlert);

/*
 * E5 — the KiUserExceptionDispatcher hook (amd64 heaven's gate). This is the
 * real-Windows dispatch point: ntdll's 64-bit KiUserExceptionDispatcher calls
 * through the `Wow64PrepareForException` slot (which we install below) with
 * (ExceptionRecord, Context) BEFORE RtlDispatchException — and unlike VEH it
 * runs regardless of this build's _NTOSKRNL_ RtlDispatchException eliding the
 * vectored-handler block.
 *
 * `Ctx` is the CAPTURED faulting context: for a compat-mode guest fault its
 * SegCs is 0x23 and its low-32 Rip/Rsp/GPRs are the guest registers (the actual
 * dispatcher runs in 64-bit mode at CS 0x33 — that's TrapFrame, not this
 * captured copy). We flush the live guest state into the CPU area
 * (BTCpuResetToConsistentState), convert the record 64->32, build the i386
 * KiUserExceptionDispatcher frame on the GUEST stack (below the guest's own
 * faulting Esp), redirect the context there, and NtContinue — which never
 * returns: the kernel's context sanitizer keeps CS 0x23 (any non-0x33 user CS
 * -> CMCODE|3) and resumes the guest's 32-bit dispatcher, so
 * RtlDispatchException never runs on this host frame. The guest's own
 * RtlDispatchException then walks its real fs:[0] SEH chain. If no handler is
 * found the guest dispatcher second-chances via NtRaiseException(...,FALSE) —
 * the manual marshaller terminates there.
 */
static VOID WINAPI Wow64pPrepareForExceptionHook(EXCEPTION_RECORD *R, CONTEXT *Ctx)
{
    PBYTE CpuArea;
    WOW64_CONTEXT *C;
    WOW64_EXCEPTION_RECORD32 Rec32;
    EXCEPTION_POINTERS Info;
    ULONG Esp, i, N;

    if (Ctx->SegCs != 0x23 || !Wow64pKiUserExceptionDispatcher32)
        return; /* not a compat-mode guest fault — let the host path handle it */

    Info.ExceptionRecord = R;
    Info.ContextRecord = Ctx;
    if (Wow64pBTCpuResetToConsistentState)
        Wow64pBTCpuResetToConsistentState(&Info); /* flush live guest -> CPU area */

    CpuArea = (PBYTE)Wow64pTlsSlots()[WOW64_TLS_CPURESERVED];
    if (!CpuArea)
    {
        /* Same hazard as Wow64pVeh: no guest state means we cannot build a
         * 32-bit exception frame, and reading C-> would fault inside the
         * handler. Let the host death path report the real fault. */
        DbgPrint("[FEX-WOW64C] guest fault with no CPU area on this thread — host path\n");
        return;
    }
    C = (WOW64_CONTEXT *)(CpuArea + 16);

    /* DBG_PRINTEXCEPTION(_WIDE)_C from a guest OutputDebugString: consume it
     * like the kernel's KD does (guest kernel32 __except is DUMMY_PSEH), print
     * and let the host dispatcher's DUMMY_PSEH swallow-and-continue path resume
     * at the faulting Eip — do NOT redirect into guest SEH. */
    if ((ULONG)R->ExceptionCode == 0x40010006 || (ULONG)R->ExceptionCode == 0x4001000A)
        return;

    Rec32.ExceptionCode    = (ULONG)R->ExceptionCode;
    Rec32.ExceptionFlags   = R->ExceptionFlags;
    Rec32.ExceptionRecord  = 0; /* no chain conversion */
    Rec32.ExceptionAddress = C->Eip;
    N = R->NumberParameters > 15 ? 15 : R->NumberParameters;
    Rec32.NumberParameters = N;
    for (i = 0; i < 15; i++)
        Rec32.ExceptionInformation[i] =
            i < N ? (ULONG)R->ExceptionInformation[i] : 0;

    /* Frame goes on the GUEST stack, below the guest's faulting Esp. */
    Esp = Wow64pBuildGuestExceptionFrame(&Rec32, C, C->Esp - 0x10);
    if (!Esp)
    {
        DbgPrint("[FEX-WOW64C] guest fault %08lx eip=%08lx: guest-stack frame build FAILED (esp=%08lx) — host death path\n",
                 Rec32.ExceptionCode, C->Eip, C->Esp);
        return;
    }

    DbgPrint("[FEX-WOW64C] guest fault %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx)\n",
             Rec32.ExceptionCode, C->Eip, Esp);
    if (Rec32.ExceptionCode == 0xc0000005)
        DbgPrint("[FEX-WOW64C]   AV rw=%lx addr=%08lx | eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx\n",
                 (ULONG)Rec32.ExceptionInformation[0], (ULONG)Rec32.ExceptionInformation[1],
                 C->Eax, C->Ebx, C->Ecx, C->Edx, C->Esi, C->Edi, C->Ebp);
    Ctx->Rip = (ULONG_PTR)Wow64pKiUserExceptionDispatcher32;
    Ctx->Rsp = Esp;
    Ctx->EFlags &= ~0x400u; /* cld — fault may have been mid-rep-movs */
    NtContinue(Ctx, FALSE); /* resume in the guest dispatcher; never returns */
}
#endif /* !_M_ARM64 */

#ifdef _M_ARM64
/*
 * E18 — the arm64 guest-SEH redirect, the arm64 half of what E5 built for
 * amd64. There is no compat-mode fault to catch here: a guest fault arrives as
 * a HOST arm64 fault inside FEX's JIT, and the CPU layer gets first refusal
 * through BTCpuResetToConsistentState. When it *can* deal with the fault (SMC,
 * overcommit, unaligned atomic, suspend) it NtContinues and never comes back.
 * When it cannot, it hands the fault over in guest terms and returns:
 *
 *   - the guest register file is reconstructed into the CPU area
 *     (FEX calls BTCpuSetContext itself),
 *   - the EXCEPTION_RECORD is rewritten to describe the GUEST fault, with
 *     ExceptionAddress set to the guest Eip — which is what lets us tell a
 *     handed-over guest fault from a genuine host-side one,
 *   - and the host CONTEXT is replaced with the one captured before JIT entry.
 *
 * So the piece that was missing is small and entirely on our side: build the
 * i386 KiUserExceptionDispatcher frame on the GUEST stack, point the guest
 * context at the dispatcher, and let the host resume. Returning
 * EXCEPTION_CONTINUE_EXECUTION resumes at that pre-JIT-entry context, i.e.
 * back inside BTCpuSimulate, which re-enters the guest from the CPU area —
 * now at the guest's own dispatcher, so its real fs:[0] SEH chain runs.
 *
 * Without this the fault has nowhere to go: the host dispatcher's
 * swallow-and-continue path resumes the simulate loop with the guest context
 * still pointing at the faulting instruction, which re-faults forever.
 */
static BOOLEAN Wow64pRedirectGuestFault(EXCEPTION_POINTERS *Info)
{
    PBYTE CpuArea = (PBYTE)Wow64pTlsSlots()[WOW64_TLS_CPURESERVED];
    EXCEPTION_RECORD *R = Info->ExceptionRecord;
    ULONG_PTR ExAddr = (ULONG_PTR)R->ExceptionAddress;
    WOW64_CONTEXT *C;
    WOW64_EXCEPTION_RECORD32 Rec32;
    ULONG Esp, i, N;

    /* Same hazard as the breakpoint path below: a VEH runs on ANY thread, and
     * a thread with no guest state has no CPU area to read. */
    if (!CpuArea || !Wow64pKiUserExceptionDispatcher32)
        return FALSE;
    C = (WOW64_CONTEXT *)(CpuArea + 16);

    /* Ours only if the CPU layer has already restated the fault in guest
     * terms. A host-side fault it declined to touch still carries a host
     * ExceptionAddress, which cannot be both below 2GB and equal to the
     * guest's Eip. */
    if (ExAddr >= 0x80000000u || ExAddr != (ULONG_PTR)C->Eip)
        return FALSE;

    /* DBG_PRINTEXCEPTION_C from a guest OutputDebugString is a message, not a
     * fault — consume it exactly like the amd64 hook does. */
    if ((ULONG)R->ExceptionCode == 0x40010006 || (ULONG)R->ExceptionCode == 0x4001000A)
        return FALSE;

    Rec32.ExceptionCode    = (ULONG)R->ExceptionCode;
    Rec32.ExceptionFlags   = R->ExceptionFlags;
    Rec32.ExceptionRecord  = 0; /* no chain conversion */
    Rec32.ExceptionAddress = C->Eip;
    N = R->NumberParameters > 15 ? 15 : R->NumberParameters;
    Rec32.NumberParameters = N;
    for (i = 0; i < 15; i++)
        Rec32.ExceptionInformation[i] =
            i < N ? (ULONG)R->ExceptionInformation[i] : 0;

    /* Frame goes on the GUEST stack, below the guest's faulting Esp. */
    Esp = Wow64pBuildGuestExceptionFrame(&Rec32, C, C->Esp - 0x10);
    if (!Esp)
    {
        DbgPrint("[FEX-WOW64C] guest fault %08lx eip=%08lx: guest-stack frame build FAILED (esp=%08lx) — host death path\n",
                 Rec32.ExceptionCode, C->Eip, C->Esp);
        return FALSE;
    }

    DbgPrint("[FEX-WOW64C] guest fault %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx)\n",
             Rec32.ExceptionCode, C->Eip, Esp);
    if (Rec32.ExceptionCode == 0xc0000005)
        DbgPrint("[FEX-WOW64C]   AV rw=%lx addr=%08lx | eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx\n",
                 Rec32.ExceptionInformation[0], Rec32.ExceptionInformation[1],
                 C->Eax, C->Ebx, C->Ecx, C->Edx, C->Esi, C->Edi, C->Ebp);

    C->Eip = (ULONG)(ULONG_PTR)Wow64pKiUserExceptionDispatcher32;
    C->Esp = Esp;
    C->EFlags &= ~0x400u; /* cld — the fault may have been mid-rep-movs */

    /* Publish through the CPU layer rather than trusting the raw CPU-area
     * write: the JIT keeps its own copy of the register file and reloads it
     * from here on simulate re-entry (the same reason the NtContinue
     * marshaller goes through BTCpuSetContext). Passing the CPU area itself is
     * explicitly supported — FEX copies the input before flushing. */
    if (Wow64pBTCpuSetContext)
        Wow64pBTCpuSetContext(WOW64_CURRENT_THREAD, WOW64_CURRENT_PROCESS, NULL, C);
    return TRUE;
}
#endif /* _M_ARM64 */

static LONG WINAPI Wow64pVeh(EXCEPTION_POINTERS *Info)
{
    if (Wow64pBTCpuResetToConsistentState)
        Wow64pBTCpuResetToConsistentState(Info); /* NtContinues if handled */

    /* Guest DbgPrint forwarding: ntdll32's DebugService does `int 2d` with
     * eax=1 (BREAKPOINT_PRINT), ecx=buffer, edx=length; FEX surfaces it as a
     * breakpoint exception. Print the guest's message to serial — this is the
     * guest loader's own debug channel (LDR: snaps, failure text). */
    if (Info->ExceptionRecord->ExceptionCode == (LONG)0x80000003L)
    {
        PBYTE CpuArea = (PBYTE)Wow64pTlsSlots()[WOW64_TLS_CPURESERVED];
        WOW64_CONTEXT *C = (WOW64_CONTEXT *)(CpuArea + 16);
        /* A VEH runs on ANY thread, including one with no guest state: a thread
         * that never entered the guest, or one past Wow64FreeThreadCpuArea on
         * its way out. Reading C-> then faults at C's own field offsets (Eax is
         * CpuArea+0xC0) *inside the exception handler*, which re-enters the VEH
         * and buries whatever the original exception was. Decline instead. */
        if (!CpuArea)
        {
            DbgPrint("[FEX-WOW64C] VEH: breakpoint with no CPU area on this thread — declining\n");
            return EXCEPTION_CONTINUE_SEARCH;
        }
        if (C->Eax == 1 && C->Ecx && C->Ecx < 0x80000000u && C->Edx < 512)
            DbgPrint("[GUEST32] %.*s", (int)C->Edx, (const char *)(ULONG_PTR)C->Ecx);
        /* Breakpoints keep the pre-E18 behaviour deliberately. FEX reports a
         * DebugService `int 2d` with ExceptionAddress == the (already
         * advanced) guest Eip but a genuine `int3` with Eip - 1, so the
         * redirect's discriminator would match exactly the one that must NOT
         * be dispatched — the loader's own debug channel — and miss the one
         * that should be. Guest int3 dispatch is its own change. */
        return EXCEPTION_CONTINUE_SEARCH;
    }

#ifdef _M_ARM64
    /* E18: hand a guest fault to the guest's 32-bit SEH. */
    if (Wow64pRedirectGuestFault(Info))
        return EXCEPTION_CONTINUE_EXECUTION;
#endif

    DbgPrint("[FEX-WOW64C] VEH: unhandled code=%08lx addr=%p pc=%p\n",
             Info->ExceptionRecord->ExceptionCode,
             Info->ExceptionRecord->ExceptionAddress,
             WOW64_HOST_PC(Info->ContextRecord));
    Wow64pDumpGuestHeap();
    return EXCEPTION_CONTINUE_SEARCH;
}

static PVOID Wow64pAllocBelow2G(SIZE_T Size, ULONG Protect, const char *What)
{
    PVOID Base = NULL;
    LONG Status = NtAllocateVirtualMemory(WOW64_CURRENT_PROCESS, &Base,
                                          WOW64_ZEROBITS_BELOW_2G, &Size,
                                          MEM_RESERVE | MEM_COMMIT, Protect);
    if (Status < 0 || (ULONG_PTR)Base >= 0x80000000u)
    {
        DbgPrint("[FEX-WOW64C] %s: below-2GB alloc FAILED (status=%08lx base=%p)\n",
                 What, Status, Base);
        return NULL;
    }
    return Base;
}

/* Map the staged 32-bit ntdll (SEC_IMAGE) for LdrSystemDllInitBlock.
 * Path: \??\<SystemRoot>\syswow64\ntdll.dll from KUSER_SHARED_DATA. */
static PVOID Wow64pMapNtdll32(VOID)
{
    static WCHAR PathBuf[280];
    const WCHAR *SystemRoot = (const WCHAR *)((PBYTE)0x7FFE0000 + 0x30);
    WCHAR *P = PathBuf;
    const WCHAR *S;
    WOW64_UNICODE_STRING Path;
    WOW64_OBJECT_ATTRIBUTES Oa;
    WOW64_IO_STATUS_BLOCK Iosb;
    HANDLE File = NULL, Section = NULL;
    PVOID Base = NULL;
    SIZE_T ViewSize = 0;
    LONG Status;

    for (S = L"\\??\\"; *S;) *P++ = *S++;
    for (S = SystemRoot; *S && (P - PathBuf) < 260;) *P++ = *S++;
    for (S = L"\\syswow64\\ntdll.dll"; *S;) *P++ = *S++;
    *P = 0;
    RtlInitUnicodeString(&Path, PathBuf);

    Oa.Length = sizeof(Oa);
    Oa.RootDirectory = NULL;
    Oa.ObjectName = &Path;
    Oa.Attributes = 0x40; /* OBJ_CASE_INSENSITIVE */
    Oa.SecurityDescriptor = NULL;
    Oa.SecurityQualityOfService = NULL;

    Status = NtOpenFile(&File, GENERIC_READ | GENERIC_EXECUTE | SYNCHRONIZE,
                        &Oa, &Iosb, FILE_SHARE_READ,
                        0x60 /* FILE_NON_DIRECTORY_FILE|FILE_SYNCHRONOUS_IO_NONALERT */);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtOpenFile(%S) FAILED %08lx\n", PathBuf, Status);
        return NULL;
    }

    Status = NtCreateSection(&Section,
                             SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_QUERY,
                             NULL, NULL, PAGE_EXECUTE, SEC_IMAGE, File);
    NtClose(File);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtCreateSection(ntdll32) FAILED %08lx\n", Status);
        return NULL;
    }

    Status = NtMapViewOfSection(Section, WOW64_CURRENT_PROCESS, &Base, 0, 0,
                                NULL, &ViewSize, 1 /* ViewShare */, 0,
                                PAGE_EXECUTE);
    NtClose(Section);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] NtMapViewOfSection(ntdll32) FAILED %08lx\n", Status);
        return NULL;
    }
    Wow64pNtdll32Size = ViewSize;
    DbgPrint("[FEX-WOW64C] ntdll32 mapped at %p size=%zx\n", Base, ViewSize);
    return Base;
}

/* Resolve an export from the mapped 32-bit ntdll by hand (PE32 export
 * directory walk — arch-independent format; we avoid LdrGetProcedureAddress
 * because the i386 image is a raw section mapping, not a loader entry). */
static PVOID Wow64pFindExport32(PVOID Base, SIZE_T ImgSize, const char *Name)
{
    PBYTE B = (PBYTE)Base;
    PIMAGE_NT_HEADERS32 Nt = (PIMAGE_NT_HEADERS32)(B + ((PIMAGE_DOS_HEADER)B)->e_lfanew);
    ULONG ExpRva = Nt->OptionalHeader.DataDirectory[0].VirtualAddress;
    PIMAGE_EXPORT_DIRECTORY Exp;
    PULONG NameRvas, FuncRvas;
    PUSHORT Ordinals;
    ULONG i;

    /* Every RVA below is attacker-or-accident controlled data read out of a
     * freshly mapped file, and this runs on EVERY map — so each one is range
     * checked before it is dereferenced rather than trusted. An unchecked walk
     * here faulted the whole 32-bit process (a 0xc0000005 inside wow64.dll with
     * no hint that the WoW64 layer, rather than the guest, was the one reading
     * out of bounds). */
    if (!ExpRva || ExpRva > ImgSize || ImgSize < sizeof(*Exp) ||
        ExpRva > ImgSize - sizeof(*Exp))
        return NULL;
    Exp = (PIMAGE_EXPORT_DIRECTORY)(B + ExpRva);

    if (Exp->NumberOfNames > ImgSize / sizeof(ULONG))
        return NULL;
    if (Exp->AddressOfNames > ImgSize ||
        Exp->AddressOfNameOrdinals > ImgSize ||
        Exp->AddressOfFunctions > ImgSize)
        return NULL;
    if ((SIZE_T)Exp->NumberOfNames * sizeof(ULONG) > ImgSize - Exp->AddressOfNames ||
        (SIZE_T)Exp->NumberOfNames * sizeof(USHORT) > ImgSize - Exp->AddressOfNameOrdinals)
        return NULL;

    NameRvas = (PULONG)(B + Exp->AddressOfNames);
    Ordinals = (PUSHORT)(B + Exp->AddressOfNameOrdinals);
    FuncRvas = (PULONG)(B + Exp->AddressOfFunctions);
    for (i = 0; i < Exp->NumberOfNames; i++)
    {
        const char *N, *M = Name;
        SIZE_T Left;

        if (NameRvas[i] >= ImgSize)
            return NULL;
        N = (const char *)(B + NameRvas[i]);
        Left = ImgSize - NameRvas[i];
        /* Bounded: an unterminated name at the end of the view must stop at the
         * view, not run past it. */
        while (Left && *N && *N == *M) { N++; M++; Left--; }
        if (Left && *N == *M)
        {
            ULONG Ord = Ordinals[i];
            if (Ord >= Exp->NumberOfFunctions ||
                (SIZE_T)Exp->AddressOfFunctions + ((SIZE_T)Ord + 1) * sizeof(ULONG) > ImgSize)
                return NULL;
            if (FuncRvas[Ord] >= ImgSize)
                return NULL;
            return B + FuncRvas[Ord];
        }
    }
    return NULL;
}

/* Patch a freshly-mapped guest image's Wow64Transition slot (if it exports
 * one) to the BOP bridge. gdi32/user32 embed their own copy of the win32ksys
 * syscall stubs, each dispatching through a per-module slot that statically
 * points at SharedUserData->SystemCall — under WoW64 every one of them must
 * be redirected, exactly like ntdll32's at bootstrap. Safe on any map: header
 * magics are checked before the export walk. */
VOID Wow64pPatchTransitionSlot(PVOID Base, SIZE_T Size)
{
    PBYTE B = (PBYTE)Base;
    PIMAGE_NT_HEADERS32 Nt;
    PULONG Slot;
    SIZE_T ImgSize;

    if (!Wow64pBopCode || !Base || Size < 0x1000)
        return;
    if (((PIMAGE_DOS_HEADER)B)->e_magic != IMAGE_DOS_SIGNATURE)
        return;
    Nt = (PIMAGE_NT_HEADERS32)(B + ((PIMAGE_DOS_HEADER)B)->e_lfanew);
    if ((ULONG_PTR)Nt - (ULONG_PTR)B > Size - sizeof(*Nt) ||
        Nt->Signature != IMAGE_NT_SIGNATURE)
        return;

    /* Everything below is RVA arithmetic, which is only meaningful for a SECTION
     * mapped as an IMAGE. A PE mapped as a raw DATA file has its sections at
     * FILE offsets instead, so the same RVAs point at unrelated bytes — or off
     * the end of the view entirely. Both kinds of mapping arrive here, because
     * this runs on every successful NtMapViewOfSection and both start with a
     * valid MZ and PE signature at offset 0. Nothing before this line could tell
     * them apart.
     *
     * The view size does. An image view is exactly the image: ViewSize ==
     * SizeOfImage (ntdll32 maps at 0x142000, its SizeOfImage). A data view is
     * the FILE, page-rounded — which is how a 32-bit socket() died: msafd.dll
     * was mapped as data at ViewSize 0x56000 (file 0x55400 rounded) while its
     * SizeOfImage is 0x49000, the export walk read RVAs into raw file bytes, and
     * wow64.dll took the access violation on the guest's behalf.
     *
     * A data mapping has no transition slot to patch, so declining is the
     * correct outcome, not a degraded one. */
    ImgSize = Nt->OptionalHeader.SizeOfImage;
    if (ImgSize < 0x1000 || ((ImgSize + 0xFFF) & ~(SIZE_T)0xFFF) != ((Size + 0xFFF) & ~(SIZE_T)0xFFF))
        return;
    if (ImgSize > Size)
        return;

    Slot = (PULONG)Wow64pFindExport32(Base, ImgSize, "Wow64Transition");
    if (Slot)
    {
        *Slot = (ULONG)(ULONG_PTR)Wow64pBopCode;
        DbgPrint("[FEX-WOW64C] Wow64Transition slot %p (module %p) patched -> BOP %p\n",
                 Slot, Base, Wow64pBopCode);
    }

#ifdef _M_AMD64
    /* The win32k client stubs in gdi32/user32 (the ZwGdi/NtUser stub families)
     * do NOT use an exported Wow64Transition slot — they dispatch through a
     * per-module _KiIndirectSystemCall:
     *     BA 00 03 FE 7F   mov edx, 0x7FFE0300   ; &SharedUserData->SystemCall
     *     FF 22            jmp dword ptr [edx]
     * On arm64/FEX the guest's syscall instruction at [SystemCall] is caught by
     * the JIT no matter what that pointer is; on amd64 the guest LITERALLY jumps
     * to [0x7FFE0300], which is 0 (KUSER_SHARED_DATA is a single global page and
     * cannot carry the per-process below-2GB BOP) -> the guest jumps to NULL
     * (rip=0, the gui32/gdi32-DllMain fault). Rewrite the dispatcher in place to
     * jump straight to this process's heaven's-gate BOP:
     *     BA <bop32>       mov edx, BOP
     *     FF E2            jmp edx
     * eax (the win32k service number, 0x1000+) is preserved and the stub's
     * `call [_KiSlot]` return address is still parked on the guest stack for the
     * normal dispatch fixup. One 7-byte patch per module covers every win32k
     * stub. amd64-only: the arm64 path is FEX-intercepted and must stay as-is. */
    if (Wow64pBopCode)
    {
        static const UCHAR Pat[7] = {0xBA,0x00,0x03,0xFE,0x7F,0xFF,0x22};
        /* ImgSize, not Size: the authoritative extent is the PE header's, and
         * scanning to a larger view size walks off the end of the image. */
        SIZE_T Scan = (ImgSize > 0x400000) ? 0x400000 : ImgSize;
        PBYTE End = B + Scan - sizeof(Pat);
        PBYTE P;
        for (P = B; P <= End; P++)
        {
            ULONG k = 0;
            while (k < sizeof(Pat) && P[k] == Pat[k]) k++;
            if (k != sizeof(Pat))
                continue;
            {
                PVOID Pg = P;
                SIZE_T Sz = sizeof(Pat);
                ULONG Old = 0;
                NtProtectVirtualMemory(WOW64_CURRENT_PROCESS, &Pg, &Sz,
                                       PAGE_EXECUTE_READWRITE, &Old);
                *(ULONG *)(P + 1) = (ULONG)(ULONG_PTR)Wow64pBopCode; /* mov edx, BOP */
                P[5] = 0xFF; P[6] = 0xE2;                            /* jmp edx      */
                NtProtectVirtualMemory(WOW64_CURRENT_PROCESS, &Pg, &Sz, Old, &Old);
                NtFlushInstructionCache(WOW64_CURRENT_PROCESS, P, sizeof(Pat));
                DbgPrint("[FEX-WOW64C] win32k KiIndirectSystemCall @ %p (module %p) -> BOP %p\n",
                         P, Base, Wow64pBopCode);
            }
            break; /* one dispatcher per module */
        }
    }
#endif
}

/* ---- 32-bit process-parameters down-conversion (M3b) -------------------- *
 * The kernel builds PEB32 by mirroring scalars only; ProcessParameters stays
 * NULL. The 32-bit loader (LdrpInitializeProcess) derefs it immediately, so
 * we down-convert the process's real 64-bit RTL_USER_PROCESS_PARAMETERS
 * (PEB64->ProcessParameters, already built by the native loader) into a
 * 32-bit one below 2GB — width conversion of the UNICODE_STRINGs, not content
 * synthesis. Win10's wow64.dll does the same PEB32 fixup at process init. */

typedef struct _WOW64_UNICODE_STRING32 {
    USHORT Length;
    USHORT MaximumLength;
    ULONG Buffer; /* 32-bit guest pointer */
} WOW64_UNICODE_STRING32;

/* 32-bit RTL_USER_PROCESS_PARAMETERS (offsets are the canonical ntdll ones;
 * only the fields the loader reads are converted, the rest stay zero). */
typedef struct _WOW64_RTL_USER_PROCESS_PARAMETERS32 {
    ULONG MaximumLength;                  /* +0x00 */
    ULONG Length;                         /* +0x04 */
    ULONG Flags;                          /* +0x08 */
    ULONG DebugFlags;                     /* +0x0c */
    ULONG ConsoleHandle;                  /* +0x10 */
    ULONG ConsoleFlags;                   /* +0x14 */
    ULONG StandardInput;                  /* +0x18 */
    ULONG StandardOutput;                 /* +0x1c */
    ULONG StandardError;                  /* +0x20 */
    WOW64_UNICODE_STRING32 CurrentDirectoryPath; /* +0x24 CURDIR.DosPath */
    ULONG CurrentDirectoryHandle;         /* +0x2c CURDIR.Handle */
    WOW64_UNICODE_STRING32 DllPath;       /* +0x30 */
    WOW64_UNICODE_STRING32 ImagePathName; /* +0x38 */
    WOW64_UNICODE_STRING32 CommandLine;   /* +0x40 */
    ULONG Environment;                    /* +0x48 */
    ULONG StartingX, StartingY, CountX, CountY, CountCharsX, CountCharsY;
    ULONG FillAttribute, WindowFlags, ShowWindowFlags;
    WOW64_UNICODE_STRING32 WindowTitle;   /* +0x70 */
    WOW64_UNICODE_STRING32 DesktopInfo;   /* +0x78 */
    WOW64_UNICODE_STRING32 ShellInfo;     /* +0x80 */
    WOW64_UNICODE_STRING32 RuntimeData;   /* +0x88 */
} WOW64_RTL_USER_PROCESS_PARAMETERS32;

/* PEB64 field offsets (ntdll native layout). */
#define PEB64_PROCESS_PARAMETERS_OFF  0x20
#define PEB32_PROCESS_PARAMETERS_OFF  0x10
/* 64-bit RTL_USER_PROCESS_PARAMETERS UNICODE_STRING member offsets. */
#define UPP64_CURDIR_OFF         0x38 /* CURDIR.DosPath (UNICODE_STRING) */
#define UPP64_CURDIRHANDLE_OFF   0x48 /* CURDIR.Handle */
#define UPP64_DLLPATH_OFF        0x50
#define UPP64_IMAGEPATHNAME_OFF  0x60
#define UPP64_COMMANDLINE_OFF    0x70
#define UPP64_ENVIRONMENT_OFF    0x80
#define UPP64_WINDOWTITLE_OFF    0xB0
#define UPP64_DESKTOPINFO_OFF    0xC0
/* The scalar block ahead of CurrentDirectory: console + the three std handles.
 * HANDLEs are 8 bytes here and 4 in the guest; a kernel handle value always
 * fits in 32 bits, which is what makes a WoW64 process able to share one
 * handle table across both views at all. */
#define UPP64_CONSOLEHANDLE_OFF  0x10
#define UPP64_CONSOLEFLAGS_OFF   0x18
#define UPP64_STDINPUT_OFF       0x20
#define UPP64_STDOUTPUT_OFF      0x28
#define UPP64_STDERROR_OFF       0x30
/* The window/show block between Environment and WindowTitle. */
#define UPP64_STARTINGX_OFF      0x88
#define UPP64_SHOWWINDOWFLAGS_OFF 0xA8

/* A corrupt/unterminated Environment would otherwise make the scan below run
 * off the end of the 64-bit block. Real environments are a few KB. */
#define WOW64_MAX_ENVIRONMENT_BYTES 0x20000

/* Copy one UNICODE_STRING64 (at Src) into Dst32, appending its buffer bytes to
 * *Arena (a below-2GB bump pointer). NULL/empty stays zero. */
static VOID Wow64pCopyUstr32(WOW64_UNICODE_STRING32 *Dst32, PBYTE Src, PBYTE *Arena)
{
    USHORT Length = *(USHORT *)(Src + 0);
    PWSTR Buffer = *(PWSTR *)(Src + 8); /* +8: after Length/Max/pad in UNICODE_STRING64 */
    Dst32->Length = Length;
    if (!Length || !Buffer)
    {
        Dst32->MaximumLength = Length ? (USHORT)(Length + sizeof(WCHAR)) : 0;
        Dst32->Buffer = 0;
        return;
    }
    Dst32->MaximumLength = (USHORT)(Length + sizeof(WCHAR));
    Dst32->Buffer = (ULONG)(ULONG_PTR)*Arena;
    for (USHORT i = 0; i < Length; i++)
        (*Arena)[i] = ((PBYTE)Buffer)[i];
    (*Arena)[Length] = 0;
    (*Arena)[Length + 1] = 0;
    *Arena += (Length + sizeof(WCHAR) + 3) & ~3u;
}

/* Marshal the process environment block down for the guest.
 *
 * The env block itself is width-agnostic — a double-NUL-terminated run of
 * WCHARs, "NAME=VALUE\0NAME=VALUE\0\0" — so it copies byte-for-byte. What is
 * NOT width-agnostic is the POINTER to it in RTL_USER_PROCESS_PARAMETERS, and
 * we simply never filled the 32-bit one in. The guest therefore saw
 * Environment == NULL and any scan of it read from address 0 (kernel32's
 * GetEnvironmentStrings dies at `cmpw $0, 0x2(%esi,%eax,2)` with esi=0 — the
 * exact shape the CurrentDirectory comment below warns about).
 *
 * This stayed latent through M3-M10 because every guest we wrote was no-CRT
 * and never touched the environment. A stock CRT-linked binary reads it during
 * msvcrt's DLL_PROCESS_ATTACH, before main.
 *
 * The env gets its OWN below-2GB allocation, not a slice of the params arena:
 * the guest's RtlSetEnvironmentVariable grows the block by allocating a new one
 * and NtFreeVirtualMemory'ing the old, which requires a real allocation base. */
static VOID Wow64pCopyEnvironment32(WOW64_RTL_USER_PROCESS_PARAMETERS32 *Upp32,
                                    PBYTE Upp64)
{
    PWSTR Env64 = *(PWSTR *)(Upp64 + UPP64_ENVIRONMENT_OFF);
    SIZE_T EnvBytes, i;
    PBYTE Env32;

    if (!Env64)
    {
        DbgPrint("[FEX-WOW64C] WARN: PEB64 params carry no Environment — "
                 "guest gets an empty (but terminated) block\n");
        EnvBytes = 2 * sizeof(WCHAR);
    }
    else
    {
        /* Find the double-NUL that ends the block (inclusive). */
        SIZE_T MaxChars = WOW64_MAX_ENVIRONMENT_BYTES / sizeof(WCHAR);
        SIZE_T n = 0;
        while (n + 1 < MaxChars && !(Env64[n] == 0 && Env64[n + 1] == 0))
            n++;
        if (n + 1 >= MaxChars)
        {
            DbgPrint("[FEX-WOW64C] WARN: Environment not double-NUL terminated within "
                     "%u bytes — guest gets an empty block\n",
                     WOW64_MAX_ENVIRONMENT_BYTES);
            Env64 = NULL;
            EnvBytes = 2 * sizeof(WCHAR);
        }
        else
        {
            EnvBytes = (n + 2) * sizeof(WCHAR);
        }
    }

    Env32 = (PBYTE)Wow64pAllocBelow2G(EnvBytes, PAGE_READWRITE, "environment32");
    if (!Env32)
    {
        /* Leave Environment NULL rather than pointing at nothing: the guest's
         * first env read would AV either way, but a loud line beats a mystery. */
        DbgPrint("[FEX-WOW64C] WARN: environment32 alloc failed — guest env is NULL\n");
        return;
    }
    if (Env64)
    {
        for (i = 0; i < EnvBytes; i++)
            Env32[i] = ((PBYTE)Env64)[i];
    }
    /* else: the alloc is already zero-filled — an empty, terminated block. */

    Upp32->Environment = (ULONG)(ULONG_PTR)Env32;
    DbgPrint("[FEX-WOW64C] environment32 at %p (%u bytes)\n", Env32, (ULONG)EnvBytes);
}

static VOID Wow64pBuildProcessParams32(ULONG_PTR Peb32)
{
    PBYTE Peb64 = (PBYTE)NtCurrentTeb() + 0; /* set below */
    PBYTE Upp64;
    WOW64_RTL_USER_PROCESS_PARAMETERS32 *Upp32;
    PBYTE Arena, Block;
    SIZE_T Size = 0x2000; /* params + string arena */

    Peb64 = *(PBYTE *)((PBYTE)NtCurrentTeb() + 0x60); /* TEB64->ProcessEnvironmentBlock */
    Upp64 = *(PBYTE *)(Peb64 + PEB64_PROCESS_PARAMETERS_OFF);
    if (!Upp64)
    {
        DbgPrint("[FEX-WOW64C] WARN: PEB64->ProcessParameters NULL — 32-bit params left empty\n");
        return;
    }

    Block = (PBYTE)Wow64pAllocBelow2G(Size, PAGE_READWRITE, "proc params32");
    if (!Block)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    Upp32 = (WOW64_RTL_USER_PROCESS_PARAMETERS32 *)Block;
    Arena = Block + sizeof(*Upp32);
    Arena = (PBYTE)(((ULONG_PTR)Arena + 3) & ~(ULONG_PTR)3);

    Upp32->MaximumLength = (ULONG)Size;
    Upp32->Length = (ULONG)Size;
    Upp32->Flags = 0x6001; /* RTL_USER_PROC_PARAMS_NORMALIZED | ProfileUser */
    /* CurrentDirectory MUST be down-converted: RtlGetFullPathName_Ustr reads
     * CurDir.DosPath.Buffer to resolve relative/DLL paths, and a NULL buffer
     * faults at `cmpw $':',0x2(esi)` (esi=0). hello32 imported nothing so never
     * resolved a path; gui32's gdi32/user32 search hits it immediately. */
    /* The standard handles. Nothing filled these, so every WoW64 guest saw
     * StandardOutput == NULL and msvcrt built a stdio with no file behind it:
     * a redirected console guest ran to completion and wrote NOTHING, silently
     * -- measured with a Wine conformance subtest whose only job is to print
     * "Fatal: test does not exist" and exit 1. It exited 1 and printed 0 bytes.
     * Same class as the Environment and CurrentDirectory fields below: the
     * guest reads a field the marshaller never wrote, and the damage is silence
     * rather than a fault, so it survives every guest that does not print.
     *
     * ConsoleHandle/ConsoleFlags ride along for the same reason: a guest that
     * inherits a console is entitled to find it. */
    Upp32->ConsoleHandle = (ULONG)*(ULONGLONG *)(Upp64 + UPP64_CONSOLEHANDLE_OFF);
    Upp32->ConsoleFlags = *(ULONG *)(Upp64 + UPP64_CONSOLEFLAGS_OFF);
    Upp32->StandardInput = (ULONG)*(ULONGLONG *)(Upp64 + UPP64_STDINPUT_OFF);
    Upp32->StandardOutput = (ULONG)*(ULONGLONG *)(Upp64 + UPP64_STDOUTPUT_OFF);
    Upp32->StandardError = (ULONG)*(ULONGLONG *)(Upp64 + UPP64_STDERROR_OFF);
    /* StartingX..ShowWindowFlags: nine consecutive ULONGs in BOTH layouts, so
     * one run copies them. ShowWindowFlags is the one that bites -- it carries
     * CreateProcess's STARTUPINFO.wShowWindow, and a guest reading zero opens
     * its first window SW_HIDE. */
    {
        ULONG i;
        const ULONG *Src = (const ULONG *)(Upp64 + UPP64_STARTINGX_OFF);
        ULONG *Dst = &Upp32->StartingX;
        for (i = 0; i <= (UPP64_SHOWWINDOWFLAGS_OFF - UPP64_STARTINGX_OFF) / 4; i++)
            Dst[i] = Src[i];
    }
    Wow64pCopyUstr32(&Upp32->CurrentDirectoryPath, Upp64 + UPP64_CURDIR_OFF, &Arena);
    Upp32->CurrentDirectoryHandle =
        (ULONG)*(ULONGLONG *)(Upp64 + UPP64_CURDIRHANDLE_OFF);
    Wow64pCopyUstr32(&Upp32->DllPath, Upp64 + UPP64_DLLPATH_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->ImagePathName, Upp64 + UPP64_IMAGEPATHNAME_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->CommandLine, Upp64 + UPP64_COMMANDLINE_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->WindowTitle, Upp64 + UPP64_WINDOWTITLE_OFF, &Arena);
    Wow64pCopyUstr32(&Upp32->DesktopInfo, Upp64 + UPP64_DESKTOPINFO_OFF, &Arena);
    Wow64pCopyEnvironment32(Upp32, Upp64);

    /* The guest loader treats an empty DllPath as fatal (LdrpInitFailure) and a
     * NULL LdrpDefaultPath crashes RtlDosSearchPath_U — synthesize the syswow64
     * search path when the 64-bit params carry none. system32 entries are fine
     * here too: the wow64 file marshallers redirect them to syswow64. */
    if (Upp32->DllPath.Length == 0)
    {
        static const WCHAR DefPath[] = L"X:\\reactos\\syswow64;X:\\reactos\\system32";
        USHORT Len = sizeof(DefPath) - sizeof(WCHAR);
        Upp32->DllPath.Length = Len;
        Upp32->DllPath.MaximumLength = (USHORT)(Len + sizeof(WCHAR));
        Upp32->DllPath.Buffer = (ULONG)(ULONG_PTR)Arena;
        for (USHORT i = 0; i < sizeof(DefPath); i++)
            Arena[i] = ((const BYTE *)DefPath)[i];
        Arena += (sizeof(DefPath) + 3) & ~3u;
    }

    *(ULONG *)((PBYTE)Peb32 + PEB32_PROCESS_PARAMETERS_OFF) = (ULONG)(ULONG_PTR)Upp32;
    DbgPrint("[FEX-WOW64C] PEB32->ProcessParameters built at %p (image='%.*S')\n",
             Upp32, Upp32->ImagePathName.Length / 2,
             (PWSTR)(ULONG_PTR)Upp32->ImagePathName.Buffer);
}

/*
 * Phase-B probe services (wow64probe.exe / docs/engineering/wow64-plan.md).
 * Magic service numbers far above any real syscall id: ECHO proves 32-bit
 * guest-stack arguments marshal through the CPU layer and a 64-bit result
 * flows back into guest EAX; VERDICT receives that result back FROM the
 * guest, proving the full round-trip, then ends the guest process (the only
 * way out of BTCpuSimulate). Real Nt* thunks are Phase C/D.
 */
#define WOW64B_SVC_ECHO      0xF00Du
#define WOW64B_SVC_VERDICT   0xD00Eu
#define WOW64B_ECHO_MAGIC    0x5CA11EDu

/* Phase C guest exit: the exit stub (emitted below the image entry hand-off)
 * lands here when the guest's entry point returns. M3a shape: the stub first
 * calls the REAL ntdll32 NtClose(NULL) stub (mov eax,N; call
 * [Wow64Transition]) so Arguments[0] = that call's NTSTATUS (want
 * STATUS_INVALID_HANDLE from the generated thunk table) and Arguments[1] =
 * the guest entry's EAX (hello32.exe returns the 0x51DE sentinel). */
#define WOW64C_SVC_EXIT      0xE0D1u
#define WOW64C_HELLO32_SENTINEL 0x51DEu
#define WOW64C_STATUS_INVALID_HANDLE 0xC0000008u

/*
 * Wow64SystemServiceEx — the syscall thunk dispatcher. The CPU layer traps a
 * guest `syscall`/BOP, stores the guest context, and calls here with the
 * 32-bit service number and a pointer to the 32-bit argument array (guest
 * stack). Unknown services report and fail loud (STATUS_INVALID_SYSTEM_
 * SERVICE) so a premature guest syscall is visible on serial instead of
 * silently returning success.
 */
ULONG
WINAPI
Wow64SystemServiceEx(ULONG ServiceNumber, PULONG Arguments)
{
    switch (ServiceNumber)
    {
        case WOW64B_SVC_ECHO:
            DbgPrint("[FEX-WOW64B] ECHO service: args %p = %08lx %08lx %08lx\n",
                     Arguments, Arguments[0], Arguments[1], Arguments[2]);
            return WOW64B_ECHO_MAGIC;

        case WOW64B_SVC_VERDICT:
            if (Arguments[0] == WOW64B_ECHO_MAGIC)
            {
                DbgPrint("[FEX-WOW64B] guest syscall round-trip PASS (echo magic returned by guest)\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            DbgPrint("[FEX-WOW64B] guest syscall round-trip FAIL (guest passed %08lx, want %08x)\n",
                     Arguments[0], WOW64B_ECHO_MAGIC);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        case WOW64C_SVC_EXIT:
            DbgPrint("[FEX-WOW64C] guest entry returned %08lx (want %08x), NtClose(NULL) via ntdll32 stub => %08lx (want %08x)\n",
                     Arguments[1], WOW64C_HELLO32_SENTINEL,
                     Arguments[0], WOW64C_STATUS_INVALID_HANDLE);
            if (Arguments[1] == WOW64C_HELLO32_SENTINEL &&
                Arguments[0] == WOW64C_STATUS_INVALID_HANDLE)
            {
                DbgPrint("[FEX-WOW64C] M2 PASS: real i386 image code executed under FEX in a WoW64 process\n");
                DbgPrint("[FEX-WOW64C] M3a PASS: real ntdll32 syscall stub round-tripped Wow64Transition -> BOP -> thunk table\n");
                DbgPrint("[FEX-WOW64C] M3b PASS: guest ntdll32 LdrInitializeThunk ran the 32-bit loader, then handed off to the image entry\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        default:
            /* Phase D: generated thunk table (wow64_thunks.c), indexed by the
             * real service number the guest's ntdll32 stub put in EAX. */
            return (ULONG)Wow64DispatchSystemService(ServiceNumber, Arguments);
    }
}

/*
 * ---- #64: the PER-THREAD WoW64 entry ------------------------------------
 *
 * The kernel already does its half for every thread, not just the first:
 * MmCreateTeb keys the combined TEB64+TEB32 low region off
 * EPROCESS.Wow64Process, and on amd64 KiSwapContextInternal re-bases the
 * KGDT64_R3_CMTEB descriptor from the incoming thread's TEB on every context
 * switch. The gap was entirely on this side — wow64.dll had no per-thread
 * entry at all, so a guest thread would have had no CPU area, no TEB32 stack
 * bounds and no BTCpuThreadInit.
 *
 * Note the deliberate asymmetry with the initial thread: that one enters
 * through the guest loader's APC frame (LdrInitializeThunk, which runs the
 * 32-bit LdrpInitializeProcess), whereas a later thread's 32-bit process is
 * already initialized and enters directly at the CONTEXT the guest asked for.
 */

#define WOW64_HEAP_ZERO_MEMORY 0x00000008

/*
 * Install this thread's CPU area: the WOW64_CPURESERVED header plus a
 * 16-aligned WOW64_CONTEXT. That CONTEXT is the register file the CPU layer
 * enters the guest with and rewrites on every mode switch, so sharing one
 * between threads corrupts both. It used to be a function-local `static`,
 * i.e. exactly one thread's worth of storage.
 *
 * The heap base is stashed in the alignment slack immediately below the
 * returned area so the teardown can free it without a second TLS slot.
 */
static WOW64_CONTEXT *Wow64pInstallCpuArea(VOID)
{
    PBYTE Buf, CpuArea;

    Buf = (PBYTE)RtlAllocateHeap(Wow64pProcessHeap(), WOW64_HEAP_ZERO_MEMORY,
                                 sizeof(PVOID) + 16 + 16 + sizeof(WOW64_CONTEXT));
    if (!Buf)
    {
        DbgPrint("[FEX-WOW64C] FAIL: per-thread CPU area allocation failed\n");
        return NULL;
    }
    CpuArea = (PBYTE)(((ULONG_PTR)Buf + sizeof(PVOID) + 15) & ~(ULONG_PTR)15);
    ((PVOID *)CpuArea)[-1] = Buf;
    *(USHORT *)(CpuArea + 2) = WOW64_MACHINE_I386; /* WOW64_CPURESERVED.Machine */
    Wow64pTlsSlots()[WOW64_TLS_CPURESERVED] = CpuArea;
    return (WOW64_CONTEXT *)(CpuArea + 16);
}

/*
 * Wow64FreeThreadCpuArea — release this thread's CPU area.
 * Called from the NtTerminateThread marshaller (wow64_manual.c) when a guest
 * thread terminates itself, which is the only moment we still have control:
 * the kernel call below it never returns, and heap memory is not reclaimed by
 * thread exit. Safe there because the CPU layer is not running — we are
 * host-side inside a syscall the guest made.
 */
VOID Wow64FreeThreadCpuArea(VOID)
{
    PBYTE CpuArea = (PBYTE)Wow64pTlsSlots()[WOW64_TLS_CPURESERVED];
    if (!CpuArea)
        return;
    Wow64pTlsSlots()[WOW64_TLS_CPURESERVED] = NULL;
    RtlFreeHeap(Wow64pProcessHeap(), 0, ((PVOID *)CpuArea)[-1]);
}

/*
 * Build the i386 CONTEXT a WoW64 thread starts from.
 *
 * The guest supplies Eip/Esp (and, for a thread, Eax/Ebx) but its segment
 * selectors must NOT be used: kernel32's BaseInitializeContext fills in the
 * native i386 GDT values (CS 0x18, DS/ES/SS 0x20, FS 0x38) because on a real
 * i386 that is what they are. Under WoW64 the guest runs in compat mode
 * against the 64-bit GDT, where the ring-3 32-bit selectors are CS 0x23,
 * SS/DS/ES 0x2B and FS 0x53 (KGDT64_R3_CMTEB). Copying the guest's values
 * verbatim would load descriptors that mean something else entirely.
 * EFlags is refused for the same reason: BaseInitializeContext asks for
 * IOPL 3 (0x3000), which user mode cannot be granted.
 */
static VOID Wow64pInitStartContext(WOW64_CONTEXT *Ctx, ULONG Eip, ULONG Esp)
{
    PBYTE XSave;
    ULONG i;

    for (i = 0; i < sizeof(*Ctx); i++)
        ((PBYTE)Ctx)[i] = 0;

    Ctx->ContextFlags = WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS;
    Ctx->Eip = Eip;
    Ctx->Esp = Esp;
    Ctx->EFlags = 0x202;
    Ctx->SegCs = 0x23;
    Ctx->SegSs = 0x2B;
    Ctx->SegDs = 0x2B;
    Ctx->SegEs = 0x2B;
    Ctx->SegFs = 0x53;
    Ctx->SegGs = 0x2B;
    XSave = (PBYTE)Ctx->ExtendedRegisters;
    *(USHORT *)(XSave + 0x00) = 0x027F;    /* FCW   */
    *(ULONG *)(XSave + 0x18) = 0x00001F80; /* MXCSR */
}

/*
 * Wow64pThreadInit — install per-thread state and enter the guest.
 * Returns only on failure; on success BTCpuSimulate owns the thread.
 *
 * StackBase/StackLimit are the GUEST's 32-bit stack bounds. i386
 * RtlDispatchException validates every fs:[0] SEH record against TEB32's
 * NtTib bounds, and MmCreateTeb deliberately leaves them zero ("set by
 * wow64.dll at thread init"), so without this every guest SEH handler on the
 * new thread is rejected and faults die second-chance.
 */
static VOID Wow64pThreadInit(const WOW64_CONTEXT *Start,
                             ULONG StackBase, ULONG StackLimit,
                             ULONG DeallocationStack,
                             const char *What)
{
    PBYTE Teb32 = Wow64pCurrentTeb32();
    WOW64_CONTEXT *Ctx;

    if (!Teb32)
    {
        DbgPrint("[FEX-WOW64C] FAIL: %s has no TEB32 (WowTebOffset=0)\n", What);
        return;
    }

    /* The WOW64INFO *data* is process-wide, but the TLS slot pointing at it
     * is per-thread — the CPU layer reads it on whichever thread it runs. */
    Wow64pTlsSlots()[WOW64_TLS_WOW64INFO] = &Wow64pWow64Info;

    Ctx = Wow64pInstallCpuArea();
    if (!Ctx)
        return;
    *Ctx = *Start;

    *(ULONG *)(Teb32 + 0x04) = StackBase;  /* NtTib.StackBase (high) */
    *(ULONG *)(Teb32 + 0x08) = StackLimit; /* NtTib.StackLimit (low) */

    /* And the BOTTOM of the reservation, which is a different fact from
     * StackLimit: StackLimit is only the COMMITTED floor and walks down as the
     * stack grows, so it cannot say how far this stack is allowed to grow.
     * MiCheckForUserStackOverflow needs that bound to extend a guest stack at
     * all -- with it zero the kernel has no 32-bit descriptor to match a guest
     * fault against, rejects every guest stack-growth fault, and the guest dies
     * on its first page of growth (measured: eventvwr's worker needed the page
     * at 0x1fae000 while TEB64 covered [0x1fb0000, 0x20b0000)). Displacement
     * asserted against the bindgen _TEB32 in ntkrnl-sys::wow64. */
    *(ULONG *)(Teb32 + 0xE0C) = DeallocationStack; /* TEB32.DeallocationStack */

    Wow64pBTCpuThreadInit();

    DbgPrint("[FEX-WOW64C] %s: entering guest eip=%08lx esp=%08lx stack=%08lx..%08lx dealloc=%08lx\n",
             What, Ctx->Eip, Ctx->Esp, StackLimit, StackBase, DeallocationStack);
    Wow64pBTCpuSimulate();
}

/* What the NtCreateThread marshaller hands the new native thread: allocated
 * by the creating thread, consumed and freed by the created one. */
typedef struct _WOW64_THREAD_SEED {
    ULONG Eip;        /* guest thread entry — kernel32's BaseThreadStartupThunk */
    ULONG Esp;
    ULONG Eax;        /* guest thread routine, per BaseInitializeContext */
    ULONG Ebx;        /* guest thread parameter */
    ULONG StackBase;
    ULONG StackLimit;
    ULONG DeallocationStack; /* INITIAL_TEB32.AllocatedStackBase */
    /* How to release this block. An in-process seed is heap memory; a seed
     * planted by ANOTHER process (CreateRemoteThread) is its own VirtualAlloc,
     * because a foreign process cannot reach into this one's heap. Freeing the
     * second with RtlFreeHeap would corrupt the heap, so the seed says which
     * it is rather than the consumer guessing. */
    ULONG Remote;
} WOW64_THREAD_SEED;

/*
 * Build the guest loader's APC frame (KiUserApcDispatcher ABI) below FrameTop,
 * carrying RunCtx as the context to resume:
 *   [E+0]=LdrInitializeThunk [E+4]=&CONTEXT [E+8,+12]=SysArgs
 *   [E+16..]=CONTEXT [E+16+CAS..]=SEH scratch
 * Entry is Eip=KiUserApcDispatcher32, Esp=E; the loader runs on the stack
 * below E and NtContinues to RunCtx when it is done.
 *
 * SystemArgument1 is the ntdll32 base for the INITIAL thread only — that is
 * the contract LdrpInitializeProcess reads it under
 * (LdrpAllocateDataTableEntry(SystemArgument1)), and 0 there made the loader
 * call RtlImageNtHeader(NULL) and near-null-write. A later thread takes the
 * LdrpInitializeThread branch instead, which never looks at it.
 */
static ULONG Wow64pBuildLoaderApcFrame(PBYTE FrameTop,
                                       const WOW64_CONTEXT *RunCtx,
                                       PVOID SystemArgument1)
{
    ULONG CAS = (ULONG)((sizeof(WOW64_CONTEXT) + 3) & ~3u);
    ULONG E = (ULONG)(ULONG_PTR)(FrameTop - (16 + CAS + 8));
    ULONG i;

    E &= ~15u;
    *(ULONG *)(ULONG_PTR)(E + 0)  = (ULONG)(ULONG_PTR)Wow64pLdrInitThunk32;
    *(ULONG *)(ULONG_PTR)(E + 4)  = E + 16; /* NormalContext = &CONTEXT */
    *(ULONG *)(ULONG_PTR)(E + 8)  = (ULONG)(ULONG_PTR)SystemArgument1;
    *(ULONG *)(ULONG_PTR)(E + 12) = 0;
    for (i = 0; i < sizeof(WOW64_CONTEXT); i++)
        *(PBYTE)(ULONG_PTR)(E + 16 + i) = ((const BYTE *)RunCtx)[i];
    return E;
}

static ULONG WINAPI Wow64pGuestThreadStart(PVOID Parameter)
{
    WOW64_THREAD_SEED Seed = *(WOW64_THREAD_SEED *)Parameter;
    WOW64_CONTEXT RunCtx, StartCtx;
    ULONG ApcEsp;

    if (Seed.Remote)
    {
        PVOID Base = Parameter;
        SIZE_T Zero = 0;
        NtFreeVirtualMemory(WOW64_CURRENT_PROCESS, &Base, &Zero, MEM_RELEASE);
    }
    else
    {
        RtlFreeHeap(Wow64pProcessHeap(), 0, Parameter);
    }

    /* What the thread runs once the 32-bit loader has initialized it: the
     * entry kernel32 asked for, on the stack kernel32 allocated for it. */
    Wow64pInitStartContext(&RunCtx, Seed.Eip, Seed.Esp);
    RunCtx.Eax = Seed.Eax;
    RunCtx.Ebx = Seed.Ebx;

    /* Route through the guest loader exactly like the initial thread does, so
     * the new thread gets LdrpInitializeThread — TLS slot allocation and
     * DLL_THREAD_ATTACH for every loaded 32-bit DLL. Entering RunCtx directly
     * would skip both, and a guest that uses TLS would read a slot array that
     * was never allocated. The frame goes at the top of the guest's OWN stack
     * (kernel32's BaseCreateStack allocated it before the syscall); the loader
     * unwinds off it before NtContinue-ing to RunCtx. */
    ApcEsp = Wow64pBuildLoaderApcFrame((PBYTE)(ULONG_PTR)Seed.Esp, &RunCtx, NULL);
    Wow64pInitStartContext(&StartCtx,
                           (ULONG)(ULONG_PTR)Wow64pKiUserApcDispatcher32, ApcEsp);

    Wow64pThreadInit(&StartCtx, Seed.StackBase, Seed.StackLimit,
                     Seed.DeallocationStack, "guest thread");

    /* Only reached if the entry failed outright — kill this thread, not the
     * process: the rest of the guest is still healthy. */
    DbgPrint("[FEX-WOW64C] guest thread: entry failed, exiting thread\n");
    RtlExitUserThread((LONG)0xC0000001L);
    return 0;
}

/*
 * Wow64CreateGuestThread — create a native thread that runs 32-bit guest code.
 * Called by the NtCreateThread marshaller with the fields it pulled out of the
 * guest's CONTEXT32/INITIAL_TEB32. The guest's own 32-bit stack was already
 * allocated by the guest (kernel32's BaseCreateStack), so we allocate only the
 * NATIVE stack here — RtlCreateUserThread's default.
 */
LONG Wow64CreateGuestThread(ULONG Eip, ULONG Esp, ULONG Eax, ULONG Ebx,
                            ULONG StackBase, ULONG StackLimit,
                            ULONG DeallocationStack,
                            BOOLEAN CreateSuspended,
                            HANDLE *ThreadHandle, WOW64_CLIENT_ID *ClientId)
{
    WOW64_THREAD_SEED *Seed;
    LONG Status;

    if (!Wow64pBTCpuThreadInit || !Wow64pBTCpuSimulate)
    {
        DbgPrint("[FEX-WOW64C] Wow64CreateGuestThread: no CPU layer\n");
        return (LONG)0xC0000001L; /* STATUS_UNSUCCESSFUL */
    }

    Seed = (WOW64_THREAD_SEED *)RtlAllocateHeap(Wow64pProcessHeap(),
                                                WOW64_HEAP_ZERO_MEMORY,
                                                sizeof(*Seed));
    if (!Seed)
        return (LONG)0xC0000017L; /* STATUS_NO_MEMORY */

    Seed->Eip = Eip;
    Seed->Esp = Esp;
    Seed->Eax = Eax;
    Seed->Ebx = Ebx;
    Seed->StackBase = StackBase;
    Seed->StackLimit = StackLimit;
    Seed->DeallocationStack = DeallocationStack;

    Status = RtlCreateUserThread(WOW64_CURRENT_PROCESS, NULL, CreateSuspended,
                                 0, 0, 0, (PVOID)Wow64pGuestThreadStart, Seed,
                                 ThreadHandle, ClientId);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] Wow64CreateGuestThread: RtlCreateUserThread %08lx\n",
                 Status);
        RtlFreeHeap(Wow64pProcessHeap(), 0, Seed);
    }
    return Status;
}

/* ---- a thread a 64-bit creator started with a 32-bit start ---------------
 *
 * CreateRemoteThread into this WoW64 process from a 64-bit one -- a native
 * process, or an x86-64 guest under wowx64 -- names a 32-bit routine, but the
 * thread the kernel makes is a native one: it arrives in the native loader
 * with a native CONTEXT, for the Win32 shape starting the native kernel32's
 * BaseThreadStartup(Start, Param), for the RtlCreateUserThread shape starting
 * Start(Param) directly. Run as native code, either one calls 32-bit code as
 * native and takes the process down. Windows' WoW64 converts such a startup
 * context to a 32-bit one for every thread it initializes
 * (ThunkStartupContext64TO32); here the native loader asks first
 * (ldrinit.rs LdrpDivertWow64Thread) and hands only those threads over.
 *
 * The thread keeps its native stack (the creator allocated it) and gets a
 * 32-bit one of its own below 2GB, fully committed so it never needs guard-page
 * growth. It then enters the guest through the 32-bit loader exactly as
 * Wow64pGuestThreadStart does: TLS and DLL_THREAD_ATTACH for the 32-bit DLLs
 * first, then the start.
 *
 * Win32 shape: the 32-bit kernel32's BaseThreadStart(Start, Param) (its export
 * of BaseThreadStartup), so the thread exits through ExitThread with the
 * routine's return value and an unhandled exception goes where a 32-bit
 * CreateThread's would. Rtl shape: Start(Param) returning into the 32-bit
 * ntdll's RtlExitUserThread. */
#define WOW64_GUEST_FROM64_STACK  (1024 * 1024)
#define LDR32_PEB_LDR_OFF         0x0C /* PEB32.Ldr                          */
#define LDR32_INLOADORDER_OFF     0x0C /* PEB_LDR_DATA32.InLoadOrderModuleList */
#define LDR32_ENTRY_DLLBASE       0x18 /* LDR_DATA_TABLE_ENTRY32.DllBase     */
#define LDR32_ENTRY_SIZEOFIMAGE   0x20 /* .SizeOfImage                       */
#define LDR32_ENTRY_BASENAME      0x2C /* .BaseDllName.Length                */
#define LDR32_ENTRY_BASENAMEBUF   0x30 /* .BaseDllName.Buffer                */
#define TEB32_PEB_OFF             0x30 /* TEB32.ProcessEnvironmentBlock      */

static WCHAR Wow64pLowerW(WCHAR c);

/* A 32-bit module of THIS process, by base name, from the 32-bit loader's own
 * list (offsets per ntkrnl-sys offset_check_i386.rs). NULL if not loaded. */
static PVOID Wow64pGuestModule32(const WCHAR *Name, SIZE_T *ImgSize)
{
    PBYTE Teb32 = Wow64pCurrentTeb32();
    ULONG Peb32, Ldr, Head, Cur;
    unsigned Guard, NameLen = 0, i;

    while (Name[NameLen])
        NameLen++;
    if (!Teb32)
        return NULL;
    Peb32 = *(ULONG *)(Teb32 + TEB32_PEB_OFF);
    if (!Peb32)
        return NULL;
    Ldr = *(ULONG *)(ULONG_PTR)(Peb32 + LDR32_PEB_LDR_OFF);
    if (!Ldr)
        return NULL;
    Head = Ldr + LDR32_INLOADORDER_OFF;
    for (Cur = *(ULONG *)(ULONG_PTR)Head, Guard = 0;
         Cur && Cur != Head && Guard < 512;
         Cur = *(ULONG *)(ULONG_PTR)Cur, Guard++)
    {
        USHORT ByteLen = *(USHORT *)(ULONG_PTR)(Cur + LDR32_ENTRY_BASENAME);
        const WCHAR *Buf = (const WCHAR *)(ULONG_PTR)
            *(ULONG *)(ULONG_PTR)(Cur + LDR32_ENTRY_BASENAMEBUF);
        if (!Buf || ByteLen / sizeof(WCHAR) != NameLen)
            continue;
        for (i = 0; i < NameLen; i++)
            if (Wow64pLowerW(Buf[i]) != Wow64pLowerW(Name[i]))
                break;
        if (i == NameLen)
        {
            *ImgSize = *(ULONG *)(ULONG_PTR)(Cur + LDR32_ENTRY_SIZEOFIMAGE);
            return (PVOID)(ULONG_PTR)*(ULONG *)(ULONG_PTR)(Cur + LDR32_ENTRY_DLLBASE);
        }
    }
    return NULL;
}

VOID WINAPI Wow64StartGuestThreadFrom64(ULONG_PTR Start, ULONG_PTR Param,
                                        BOOLEAN Win32Thread)
{
    WOW64_CONTEXT RunCtx, StartCtx;
    PVOID Module, Entry;
    SIZE_T ImgSize = 0;
    PBYTE Stack;
    ULONG Top, Esp, ApcEsp;

    if (Start > 0xFFFFFFFFu || Param > 0xFFFFFFFFu)
    {
        /* LdrpDivertWow64Thread only asks for a start that is not native
         * code; one that is not 32-bit either is nobody's. */
        DbgPrint("[FEX-WOW64C] thread from 64-bit: start %p param %p not 32-bit\n",
                 (PVOID)Start, (PVOID)Param);
        RtlExitUserThread((LONG)0xC000007BL); /* STATUS_INVALID_IMAGE_FORMAT */
    }
    if (!Wow64pBTCpuThreadInit || !Wow64pBTCpuSimulate || !Wow64pLdrInitThunk32)
    {
        DbgPrint("[FEX-WOW64C] thread from 64-bit: no CPU layer\n");
        RtlExitUserThread((LONG)0xC0000001L);
    }

    Module = Wow64pGuestModule32(Win32Thread ? L"kernel32.dll" : L"ntdll.dll", &ImgSize);
    Entry = Module ? Wow64pFindExport32(Module, ImgSize,
                                        Win32Thread ? "BaseThreadStart" : "RtlExitUserThread")
                   : NULL;
    if (!Entry)
    {
        DbgPrint("[FEX-WOW64C] thread from 64-bit: 32-bit %s has no %s\n",
                 Win32Thread ? "kernel32" : "ntdll",
                 Win32Thread ? "BaseThreadStart" : "RtlExitUserThread");
        RtlExitUserThread((LONG)0xC0000139L); /* STATUS_ENTRYPOINT_NOT_FOUND */
    }

    Stack = (PBYTE)Wow64pAllocBelow2G(WOW64_GUEST_FROM64_STACK, PAGE_READWRITE,
                                      "thread from 64-bit: stack");
    if (!Stack)
        RtlExitUserThread((LONG)0xC0000017L); /* STATUS_NO_MEMORY */
    Top = (ULONG)(ULONG_PTR)Stack + WOW64_GUEST_FROM64_STACK;

    /* The routine's frame. stdcall: [esp] return address, then the args. The
     * loader APC frame goes below it (Wow64pBuildLoaderApcFrame). */
    if (Win32Thread)
    {
        /* BaseThreadStartup(Start, Param) never returns: ExitThread. */
        Esp = (Top - 0x100) & ~15u;
        ((ULONG *)(ULONG_PTR)Esp)[0] = 0;
        ((ULONG *)(ULONG_PTR)Esp)[1] = (ULONG)Start;
        ((ULONG *)(ULONG_PTR)Esp)[2] = (ULONG)Param;
        Wow64pInitStartContext(&RunCtx, (ULONG)(ULONG_PTR)Entry, Esp);
    }
    else
    {
        /* Start(Param), returning its exit code into RtlExitUserThread. */
        Esp = (Top - 0x100) & ~15u;
        ((ULONG *)(ULONG_PTR)Esp)[0] = (ULONG)(ULONG_PTR)Entry;
        ((ULONG *)(ULONG_PTR)Esp)[1] = (ULONG)Param;
        Wow64pInitStartContext(&RunCtx, (ULONG)Start, Esp);
    }

    ApcEsp = Wow64pBuildLoaderApcFrame((PBYTE)(ULONG_PTR)(Esp - 16), &RunCtx, NULL);
    Wow64pInitStartContext(&StartCtx,
                           (ULONG)(ULONG_PTR)Wow64pKiUserApcDispatcher32, ApcEsp);

    DbgPrint("[FEX-WOW64C] thread from 64-bit: %s start=%08lx param=%08lx stack=%p\n",
             Win32Thread ? "Win32" : "Rtl", (ULONG)Start, (ULONG)Param, Stack);
    Wow64pThreadInit(&StartCtx, Top, (ULONG)(ULONG_PTR)Stack,
                     (ULONG)(ULONG_PTR)Stack, "thread from 64-bit");

    DbgPrint("[FEX-WOW64C] thread from 64-bit: entry failed, exiting thread\n");
    RtlExitUserThread((LONG)0xC0000001L);
}

/* ---- cross-process guest thread creation (task #64, second half) --------
 *
 * CreateRemoteThread from a 32-bit guest into ANOTHER running 32-bit process.
 * The in-process path above cannot simply be pointed at a foreign handle, and
 * the two reasons are worth separating because only one of them is hard:
 *
 *   The SEED is a pointer into this process's heap. Meaningless over there.
 *     Fixed by allocating the seed in the TARGET instead, which is why
 *     WOW64_THREAD_SEED carries a Remote flag telling the consumer how to free
 *     it -- RtlFreeHeap on a foreign VirtualAlloc would corrupt the heap.
 *
 *   The START ROUTINE is a code address in THIS copy of wow64.dll. The target
 *     has its own copy, and assuming the two share a base is exactly the kind
 *     of "it is a system DLL, it is always at the same address" reasoning that
 *     holds right up until ASLR or a relocation makes it false -- silently, by
 *     starting a thread at whatever lives at that address over there. So it is
 *     not assumed: walk the TARGET's loader list, find its wow64.dll, and
 *     translate Wow64pGuestThreadStart through the measured delta. Equal bases
 *     make the delta zero and cost nothing but the walk.
 *
 * What this deliberately does NOT build is Win10's cross-process work list
 * (Wow64ProcessPendingCrossProcessItems, still empty here). That mechanism
 * exists so the TARGET can do the per-thread setup on its own schedule. We do
 * not need it: the new thread's start routine IS the target's own
 * Wow64pGuestThreadStart, so the setup happens in the target, on the new
 * thread, by the target's own code -- which is the property the work list
 * exists to provide.
 *
 * Loader-walk offsets MEASURED against the bindgen structs this tree builds
 * against, with sizeof as the control (PEB_LDR_DATA 72, LDR_DATA_TABLE_ENTRY
 * 152 on amd64, per ntdll-sys/src/size_check_amd64.rs).
 */
#define WOW64_SELF_DLL_NAME      L"wow64.dll"
#define LDR64_PEB_LDR_OFF        0x18 /* PEB64.Ldr                          */
#define LDR64_INLOADORDER_OFF    0x10 /* PEB_LDR_DATA.InLoadOrderModuleList */
#define LDR64_ENTRY_DLLBASE      0x30 /* LDR_DATA_TABLE_ENTRY.DllBase       */
#define LDR64_ENTRY_BASENAME     0x58 /* .BaseDllName.Length                */
#define LDR64_ENTRY_BASENAMEBUF  0x60 /* .BaseDllName.Buffer                */
/* A corrupt list must not spin forever; no process has this many modules. */
#define LDR64_MAX_MODULES        512

static WCHAR Wow64pLowerW(WCHAR c)
{
    return (c >= L'A' && c <= L'Z') ? (WCHAR)(c - L'A' + L'a') : c;
}

/* DllBase of `Name` in another process, or 0 if it is not loaded there. */
static ULONGLONG Wow64pRemoteModuleBase(HANDLE Process, ULONGLONG Peb64,
                                        const WCHAR *Name)
{
    ULONGLONG Ldr = 0, Head, Cur = 0;
    SIZE_T Done = 0;
    unsigned Guard, NameLen = 0;

    while (Name[NameLen])
        NameLen++;

    if (NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)(Peb64 + LDR64_PEB_LDR_OFF),
                            &Ldr, sizeof(Ldr), &Done) < 0 || !Ldr)
        return 0;

    Head = Ldr + LDR64_INLOADORDER_OFF;
    if (NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)Head, &Cur,
                            sizeof(Cur), &Done) < 0)
        return 0;

    for (Guard = 0; Cur && Cur != Head && Guard < LDR64_MAX_MODULES; Guard++)
    {
        /* InLoadOrderLinks sits at offset 0, so the link IS the entry. */
        ULONGLONG DllBase = 0, NameBuf = 0;
        USHORT ByteLen = 0;
        WCHAR Buf[64];
        unsigned Chars, i;
        int Match;

        if (NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)(Cur + LDR64_ENTRY_DLLBASE),
                                &DllBase, sizeof(DllBase), &Done) >= 0 &&
            NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)(Cur + LDR64_ENTRY_BASENAME),
                                &ByteLen, sizeof(ByteLen), &Done) >= 0 &&
            NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)(Cur + LDR64_ENTRY_BASENAMEBUF),
                                &NameBuf, sizeof(NameBuf), &Done) >= 0 &&
            NameBuf && ByteLen && (ByteLen / sizeof(WCHAR)) == NameLen &&
            NameLen < (sizeof(Buf) / sizeof(Buf[0])) &&
            NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)NameBuf, Buf,
                                ByteLen, &Done) >= 0)
        {
            Chars = ByteLen / sizeof(WCHAR);
            Match = 1;
            for (i = 0; i < Chars; i++)
                if (Wow64pLowerW(Buf[i]) != Wow64pLowerW(Name[i]))
                {
                    Match = 0;
                    break;
                }
            if (Match)
                return DllBase;
        }

        if (NtReadVirtualMemory(Process, (PVOID)(ULONG_PTR)Cur, &Cur,
                                sizeof(Cur), &Done) < 0)
            break;
    }
    return 0;
}

LONG Wow64CreateForeignGuestThread(HANDLE Process, ULONGLONG Peb64,
                                   ULONG Eip, ULONG Esp, ULONG Eax, ULONG Ebx,
                                   ULONG StackBase, ULONG StackLimit,
                                   ULONG DeallocationStack,
                                   BOOLEAN CreateSuspended,
                                   HANDLE *ThreadHandle, WOW64_CLIENT_ID *ClientId)
{
    WOW64_THREAD_SEED Seed;
    ULONGLONG TheirBase;
    LONGLONG Delta;
    PVOID Remote = NULL, RemoteStart;
    SIZE_T Size = sizeof(Seed), Done = 0, Zero = 0;
    LONG Status;

    if (!Wow64pSelfBase)
        return (LONG)0xC0000001L; /* STATUS_UNSUCCESSFUL */

    TheirBase = Wow64pRemoteModuleBase(Process, Peb64, WOW64_SELF_DLL_NAME);
    if (!TheirBase)
    {
        /* Not a WoW64 process, or one whose wow64.dll has not loaded yet --
         * either way there is nothing over there to run the guest side, and
         * refusing beats starting a thread that cannot be initialised. */
        DbgPrint("[FEX-WOW64C] foreign thread: target has no wow64.dll loaded\n");
        return (LONG)0xC0000002L; /* STATUS_NOT_IMPLEMENTED */
    }
    Delta = (LONGLONG)TheirBase - (LONGLONG)(ULONG_PTR)Wow64pSelfBase;
    RemoteStart = (PVOID)(ULONG_PTR)
        ((LONGLONG)(ULONG_PTR)Wow64pGuestThreadStart + Delta);

    Seed.Eip = Eip;
    Seed.Esp = Esp;
    Seed.Eax = Eax;
    Seed.Ebx = Ebx;
    Seed.StackBase = StackBase;
    Seed.StackLimit = StackLimit;
    Seed.DeallocationStack = DeallocationStack;
    Seed.Remote = 1;

    Status = NtAllocateVirtualMemory(Process, &Remote, 0, &Size,
                                     MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (Status < 0)
    {
        DbgPrint("[FEX-WOW64C] foreign thread: seed allocate %08lx\n", Status);
        return Status;
    }
    Status = NtWriteVirtualMemory(Process, Remote, &Seed, sizeof(Seed), &Done);
    if (Status < 0 || Done != sizeof(Seed))
    {
        NtFreeVirtualMemory(Process, &Remote, &Zero, MEM_RELEASE);
        return Status < 0 ? Status : (LONG)0x8000000DL;
    }

    Status = RtlCreateUserThread(Process, NULL, CreateSuspended, 0, 0, 0,
                                 RemoteStart, Remote, ThreadHandle, ClientId);
    if (Status < 0)
    {
        NtFreeVirtualMemory(Process, &Remote, &Zero, MEM_RELEASE);
        DbgPrint("[FEX-WOW64C] foreign thread: RtlCreateUserThread %08lx\n", Status);
        return Status;
    }

    DbgPrint("[FEX-WOW64C] foreign guest thread tid=%lx eip=%08lx esp=%08lx "
             "seed=%p start=%p (wow64.dll delta %s)\n",
             (ULONG)(ULONG_PTR)ClientId->UniqueThread, Eip, Esp, Remote,
             RemoteStart, Delta ? "NONZERO - relocated" : "0");
    return Status;
}

/*
 * Wow64LdrpInitialize — the thread handoff from the 64-bit ntdll loader.
 * For a WoW64 process, LdrpInitializeProcess (ntdll-rust ldrinit.rs) skips the
 * i386 image entirely, loads this DLL, and calls here with the initial thread
 * CONTEXT. This function owns the thread from here on and never returns: it
 * will bootstrap the 32-bit side (map syswow64 ntdll, init the CPU dll, build
 * the guest context, BTCpuSimulate).
 *
 * Phase C milestone 1 scope: prove the kernel+loader chain end-to-end —
 * EPROCESS marked WoW64 (PEB32 via ProcessWow64Information), TEB32 reachable
 * through the raw WowTebOffset slot (TEB+0x180c, kernel MmCreateTeb wrote it),
 * then exit 0x600D as the harness verdict. The CPU-layer bring-up (proven
 * standalone by wow64probe in Phase B) moves in here as milestone 2.
 */
VOID
WINAPI
Wow64LdrpInitialize(PVOID Context)
{
    ULONG_PTR Peb32 = 0;
    PBYTE Teb32 = Wow64pCurrentTeb32();
    ULONG ImageBase32;
    PBYTE ImageBase;
    PIMAGE_NT_HEADERS32 Nt32;
    ULONG EntryRva;
    ULONG GuestStackSize;
    PVOID Ntdll32, InitBlock, Fex, Bop;
    PBYTE ExitStub, GuestStack, ApcStack;
    PVOID KiUserApcDispatcher32, LdrInitThunk32;
    ULONG GuestEsp, ApcEsp;
    WOW64_UNICODE_STRING DllName;
    WOW64_ANSI_STRING ProcName;
    VOID (WINAPI *BTCpuProcessInit)(VOID);
    VOID (WINAPI *BTCpuThreadInit)(VOID);
    PVOID (WINAPI *BTCpuGetBopCode)(VOID);
    VOID (WINAPI *BTCpuSimulate)(VOID);
    LONG (WINAPI *BTCpuNotifyMapViewOfSection)(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG);
    LONG Status;

    DbgPrint("[FEX-WOW64C] Wow64LdrpInitialize: 64-bit loader handed the thread over (Context=%p)\n",
             Context);

    /* Register the crash oracle FIRST: libwow64fex.dll (and its kernel32
     * dependency) load below, before any BTCpu call — a fault there must be
     * dumped, not swallowed into a silent process exit. */
    RtlAddVectoredExceptionHandler(1, Wow64pVeh);

    /* -- M1 chain verification (kernel bootstrap sanity) ------------------ */
    Status = NtQueryInformationProcess(WOW64_CURRENT_PROCESS,
                                       26 /* ProcessWow64Information */,
                                       &Peb32, sizeof(Peb32), NULL);
    if (Status < 0 || !Peb32 || !Teb32 ||
        *(ULONG *)(Teb32 + 0x30 /* TEB32.ProcessEnvironmentBlock */) != (ULONG)Peb32)
    {
        DbgPrint("[FEX-WOW64C] FAIL: PEB32/TEB32 chain inconsistent (PEB32=%p TEB32=%p st=%08lx)\n",
                 (PVOID)Peb32, Teb32, Status);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    Wow64pPeb32 = Peb32; /* for the VEH heap diagnostic */
    ImageBase32 = *(ULONG *)((PBYTE)Peb32 + 0x08 /* PEB32.ImageBaseAddress */);
    ImageBase = (PBYTE)(ULONG_PTR)ImageBase32;
    Nt32 = (PIMAGE_NT_HEADERS32)(ImageBase + ((PIMAGE_DOS_HEADER)ImageBase)->e_lfanew);
    EntryRva = Nt32->OptionalHeader.AddressOfEntryPoint;
    DbgPrint("[FEX-WOW64C] PEB32=%p TEB32=%p image=%08lx entry=+%lx\n",
             (PVOID)Peb32, Teb32, ImageBase32, EntryRva);

    /* -- 32-bit ntdll + LdrSystemDllInitBlock ----------------------------- */
    Ntdll32 = Wow64pMapNtdll32();
    if (!Ntdll32)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);

    {
        WOW64_UNICODE_STRING NtdllName;
        PVOID NtdllBase = NULL;
        RtlInitUnicodeString(&NtdllName, L"ntdll.dll");
        LdrGetDllHandle(NULL, NULL, &NtdllName, &NtdllBase);
        RtlInitAnsiString(&ProcName, "LdrSystemDllInitBlock");
        InitBlock = NULL;
        if (!NtdllBase ||
            LdrGetProcedureAddress(NtdllBase, &ProcName, 0, &InitBlock) < 0 ||
            !InitBlock)
        {
            DbgPrint("[FEX-WOW64C] FAIL: LdrSystemDllInitBlock unresolved\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *(ULONGLONG *)((PBYTE)InitBlock + WOW64_INITBLOCK_NTDLL_OFF) =
            (ULONGLONG)(ULONG_PTR)Ntdll32;
    }

    /* -- WOW64INFO + environment landmine (Logging::Init formats
     *    getenv("LOCALAPPDATA") unchecked; default it before the CPU dll's
     *    mini-CRT snapshots the PEB environment) --------------------------- */
    Wow64pWow64Info.NativeSystemPageSize = 0x1000;
    Wow64pWow64Info.NativeMachineType = WOW64_NATIVE_MACHINE;
    Wow64pWow64Info.EmulatedMachineType = WOW64_MACHINE_I386;
    Wow64pTlsSlots()[WOW64_TLS_WOW64INFO] = &Wow64pWow64Info;

    {
        WOW64_UNICODE_STRING EnvName, EnvValue;
        RtlInitUnicodeString(&EnvName, L"LOCALAPPDATA");
        RtlInitUnicodeString(&EnvValue, L"C:\\ReactOS\\Temp");
        RtlSetEnvironmentVariable(NULL, &EnvName, &EnvValue);
    }

    /* -- Load the CPU layer ------------------------------------------------ */
    DbgPrint("[FEX-WOW64C] InitBlock.ntdll_handle set; loading CPU layer...\n");
    RtlInitUnicodeString(&DllName, WOW64_CPU_DLL);
    Fex = NULL;
    Status = LdrLoadDll(NULL, NULL, &DllName, &Fex);
    if (Status < 0 || !Fex)
    {
        DbgPrint("[FEX-WOW64C] FAIL: CPU-layer DLL load %08lx\n", Status);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

#define WOW64_RESOLVE(fn)                                                     \
    do {                                                                      \
        PVOID Addr = NULL;                                                    \
        RtlInitAnsiString(&ProcName, #fn);                                    \
        if (LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr) < 0 || !Addr)    \
        {                                                                     \
            DbgPrint("[FEX-WOW64C] FAIL: " #fn " unresolved\n");              \
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);                 \
        }                                                                     \
        *(PVOID *)&fn = Addr;                                                 \
    } while (0)

    WOW64_RESOLVE(BTCpuProcessInit);
    WOW64_RESOLVE(BTCpuThreadInit);
    WOW64_RESOLVE(BTCpuGetBopCode);
    WOW64_RESOLVE(BTCpuSimulate);
    WOW64_RESOLVE(BTCpuNotifyMapViewOfSection);
    Wow64pBTCpuNotifyMap = BTCpuNotifyMapViewOfSection;
    {
        PVOID Addr = NULL;
        RtlInitAnsiString(&ProcName, "BTCpuSetContext");
        LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr);
        Wow64pBTCpuSetContext = (LONG (WINAPI *)(HANDLE, HANDLE, PVOID, PVOID))Addr;
    }
    {
        PVOID Addr = NULL;
        RtlInitAnsiString(&ProcName, "BTCpuGetContext");
        LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr);
        Wow64pBTCpuGetContext = (LONG (WINAPI *)(HANDLE, HANDLE, PVOID, PVOID))Addr;
    }
    Wow64pBTCpuSimulate = BTCpuSimulate;
    Wow64pBTCpuThreadInit = BTCpuThreadInit; /* #64: the per-thread entry needs it */
    {
        PVOID Addr = NULL;
        RtlInitAnsiString(&ProcName, "BTCpuResetToConsistentState");
        LdrGetProcedureAddress(Fex, &ProcName, 0, &Addr);
        Wow64pBTCpuResetToConsistentState = (LONG (WINAPI *)(EXCEPTION_POINTERS *))Addr;
    }
#undef WOW64_RESOLVE

    DbgPrint("[FEX-WOW64C] CPU layer %ws loaded @ %p; calling BTCpuProcessInit...\n",
             WOW64_CPU_DLL, Fex);
    /* FEX discovers the process address-space layout during ProcessInit.
     * Allocate every raw guest page first; pages created afterward are not
     * visible until the full memory-notification contract is wired. */
    ExitStub = Wow64pAllocBelow2G(0x1000, PAGE_EXECUTE_READWRITE, "exit stub");

    /*
     * The guest's stack must be the size its IMAGE asked for, not a fixed 64K.
     *
     * This was `0x10000` hardcoded, while every stock applet declares
     * SizeOfStackReserve = 0x100000. A guest with frames deeper than 64K then
     * walks off the bottom of its own stack, and the compiler's stack-probe
     * helper is what finds it first: charmap died at `__alloca+0x12`
     * (`test %ecx,(%ecx)`, the page probe) reading 001efd84 with the stack
     * mapped 001f0000..00200000 -- one page BELOW the allocation.
     *
     * It also explains the corrupt SEH chain seen from the same guest.
     * Wow64pThreadInit publishes these same bounds as TEB32
     * NtTib.StackBase/StackLimit, and i386 RtlDispatchException validates
     * fs:[0] registration records against them -- so once esp leaves the
     * region, the chain is out of bounds and reads as garbage. One root
     * cause, two symptoms.
     *
     * NOTE this COMMITS the whole reserve (Wow64pAllocBelow2G is
     * MEM_RESERVE|MEM_COMMIT), where Windows reserves it and commits one page
     * plus a PAGE_GUARD that grows on demand. Committing outright is a
     * deliberate simplification: it costs ~1MB per guest and needs no
     * guard-page growth path, and it cannot fault where the real thing would
     * have grown. Growing on demand is the refinement, not the fix.
     */
    GuestStackSize = Nt32->OptionalHeader.SizeOfStackReserve;
    if (GuestStackSize < 0x10000)
        GuestStackSize = 0x10000;          /* never smaller than the old fixed size */
    if (GuestStackSize > 0x1000000)
        GuestStackSize = 0x1000000;        /* sanity cap: 16MB */
    GuestStackSize = (GuestStackSize + 0xFFFF) & ~0xFFFFu;
    GuestStack = Wow64pAllocBelow2G(GuestStackSize, PAGE_READWRITE, "guest run stack");
    ApcStack = Wow64pAllocBelow2G(0x40000, PAGE_READWRITE, "guest loader stack");
    if (!ExitStub || !GuestStack || !ApcStack)
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);

    /* Build PEB32->ProcessParameters (down-converted from PEB64) so the 32-bit
     * loader has valid ImagePathName/CommandLine to run LdrpInitializeProcess. */
    Wow64pBuildProcessParams32(Peb32);

    BTCpuProcessInit();
    /* Only now: the CPU layer built its address-space model inside ProcessInit,
     * so a notification delivered before this point has nothing to land in. */
    Wow64pResolveNotifyContract(Fex);
    Bop = BTCpuGetBopCode();
    Wow64pBopCode = Bop;
    DbgPrint("[FEX-WOW64C] BTCpuProcessInit OK — BOP=%p CpuFlags=%lx\n",
             Bop, Wow64pWow64Info.CpuFlags);
    if ((ULONG_PTR)Bop >= 0x80000000u)
    {
        DbgPrint("[FEX-WOW64C] FAIL: BOP above 2GB\n");
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Publish the BOP into the guest ntdll32's Wow64Transition slot ------
     * (Win10-parity: every i386 user syscall stub is `mov eax, N; call
     * [Wow64Transition]`; statically the slot holds the native fallback, we
     * repoint it at the CPU layer's bridge in OUR process's COW data page). */
    {
        PULONG Slot = (PULONG)Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "Wow64Transition");
        PVOID Page;
        SIZE_T Size = sizeof(ULONG);
        ULONG OldProt = 0;
        if (!Slot)
        {
            DbgPrint("[FEX-WOW64C] FAIL: ntdll32 has no Wow64Transition export (stale syswow64 stage?)\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        Page = Slot;
        NtProtectVirtualMemory(WOW64_CURRENT_PROCESS, &Page, &Size,
                               PAGE_READWRITE, &OldProt); /* COW data — best effort */
        *Slot = (ULONG)(ULONG_PTR)Bop;
        DbgPrint("[FEX-WOW64C] Wow64Transition slot %p patched -> BOP %p\n", Slot, Bop);
    }

    /* -- Register the guest images with FEX -------------------------------
     * The kernel mapped hello32 (image base) and we mapped ntdll32 BEFORE
     * BTCpuProcessInit ran its address-space discovery — so FEX's image
     * tracker doesn't know they're images and the guest ntdll's
     * RtlImageNtHeader read of the PE header at the image base returns NULL
     * (loader then near-null-writes). Notify FEX now (real WoW64 calls this
     * from the map-view hook). */
    BTCpuNotifyMapViewOfSection(NULL, (PVOID)(ULONG_PTR)ImageBase32, NULL, 0, 0, PAGE_EXECUTE_READ);
    BTCpuNotifyMapViewOfSection(NULL, Ntdll32, NULL, 0, 0, PAGE_EXECUTE_READ);
    DbgPrint("[FEX-WOW64C] BTCpuNotifyMapViewOfSection: image=%08lx ntdll32=%p registered\n",
             ImageBase32, Ntdll32);

    /* -- Resolve the guest ntdll32 loader entrypoints (M3b) ---------------- */
    KiUserApcDispatcher32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "KiUserApcDispatcher");
    LdrInitThunk32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "LdrInitializeThunk");
    /* #64: a guest thread routes through the same pair, so hoist them out of
     * this frame — Wow64pGuestThreadStart runs long after it is gone. */
    Wow64pKiUserApcDispatcher32 = KiUserApcDispatcher32;
    Wow64pLdrInitThunk32 = LdrInitThunk32;
    /* M8: kernel->user callbacks re-enter the guest through its own 32-bit
     * callback dispatcher; the 64-bit PEB table points at our bridge. */
    Wow64pKiUserCallbackDispatcher32 =
        Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "KiUserCallbackDispatcher");
    /* E5: guest SEH entry — non-fatal if absent (redirect stays disabled and
     * guest faults fall back to the loud host-side diagnostics + death). */
    Wow64pKiUserExceptionDispatcher32 =
        Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "KiUserExceptionDispatcher");
    if (!Wow64pKiUserExceptionDispatcher32)
        DbgPrint("[FEX-WOW64C] WARN: ntdll32 has no KiUserExceptionDispatcher export — guest SEH dispatch disabled\n");
#ifndef _M_ARM64
    /* E5: install the KiUserExceptionDispatcher hook in the HOST ntdll so
     * hardware faults in 32-bit guest code dispatch into the guest's own SEH.
     * The slot is a ntdll data export (dispatch.S / ntdll.spec). */
    {
        WOW64_UNICODE_STRING NtdllName;
        PVOID Ntdll64 = NULL;
        PVOID *Slot = NULL;
        RtlInitUnicodeString(&NtdllName, L"ntdll.dll");
        LdrGetDllHandle(NULL, NULL, &NtdllName, &Ntdll64);
        RtlInitAnsiString(&ProcName, "Wow64PrepareForException");
        if (Ntdll64 &&
            LdrGetProcedureAddress(Ntdll64, &ProcName, 0, (PVOID *)&Slot) >= 0 &&
            Slot)
        {
            *Slot = (PVOID)Wow64pPrepareForExceptionHook;
            DbgPrint("[FEX-WOW64C] Wow64PrepareForException hook installed @ %p (slot %p)\n",
                     (PVOID)Wow64pPrepareForExceptionHook, (PVOID)Slot);
        }
        else
        {
            DbgPrint("[FEX-WOW64C] WARN: Wow64PrepareForException slot unresolved — guest SEH dispatch disabled\n");
        }
    }
#endif
    Wow64pInstallUserCallbackTable();
    if (!KiUserApcDispatcher32 || !LdrInitThunk32)
    {
        DbgPrint("[FEX-WOW64C] FAIL: ntdll32 KiUserApcDispatcher=%p LdrInitializeThunk=%p\n",
                 KiUserApcDispatcher32, LdrInitThunk32);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Guest exit stub + stack ------------------------------------------ */
    /* When the image entry (cdecl, no args) returns, EIP lands on this stub
     * with EAX = the return value. M3a shape: exercise a REAL ntdll32 syscall
     * stub first (NtClose(NULL) -> STATUS_INVALID_HANDLE through
     * Wow64Transition -> BOP -> generated thunk table), then report both
     * results through the EXIT service (HandleSyscallImpl pops the
     * ret-to-bopcall slot, args at ESP+8):
     *   exit:  50               push eax          ; save entry sentinel
     *          6A 00            push 0            ; NtClose arg: handle=NULL
     *          BA <NtClose32>   mov edx, ntdll32!NtClose
     *          FF D2            call edx          ; stub does `ret 4`
     *          50               push eax          ; NtClose status
     *          B8 <EXIT>        mov eax, SVC_EXIT ; args: [status, sentinel]
     *          E8 <rel bopcall> call bopcall      ; (never returns)
     *          F4               hlt
     *   bopcall: BA <BOP>       mov edx, BOP
     *          FF D2            call edx
     *          C3               ret
     */
    {
        PVOID NtClose32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, "NtClose");
        PBYTE P = ExitStub;
        PBYTE BopCall;
        ULONG Rel;
        if (!NtClose32)
        {
            DbgPrint("[FEX-WOW64C] FAIL: ntdll32 has no NtClose export\n");
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *P++ = 0x50;                                             /* push eax */
        *P++ = 0x6A; *P++ = 0x00;                                /* push 0   */
        *P++ = 0xBA; *(ULONG *)P = (ULONG)(ULONG_PTR)NtClose32; P += 4;
        *P++ = 0xFF; *P++ = 0xD2;                                /* call edx */
        *P++ = 0x50;                                             /* push eax */
        *P++ = 0xB8; *(ULONG *)P = WOW64C_SVC_EXIT; P += 4;      /* mov eax  */
        *P++ = 0xE8; BopCall = P; P += 4;                        /* call rel */
        *P++ = 0xF4;                                             /* hlt      */
        Rel = (ULONG)(P - (BopCall + 4));
        *(ULONG *)BopCall = Rel;
        *P++ = 0xBA; *(ULONG *)P = (ULONG)(ULONG_PTR)Bop; P += 4;/* mov edx  */
        *P++ = 0xFF; *P++ = 0xD2;                                /* call edx */
        *P++ = 0xC3;                                             /* ret      */
    }

    GuestEsp = (ULONG)(ULONG_PTR)GuestStack + GuestStackSize - 64;
    GuestEsp -= 4;
    *(ULONG *)(ULONG_PTR)GuestEsp = (ULONG)(ULONG_PTR)ExitStub; /* ret addr */

    /* -- Inner "run" CONTEXT32: what the guest executes AFTER the loader
     *    finishes (LdrInitializeThunk ends by NtContinue-ing to this). It is
     *    the i386 image entry on the run stack, returning to the exit stub. -- */
    {
        WOW64_CONTEXT RunCtx, StartCtx;

        Wow64pInitStartContext(&RunCtx, ImageBase32 + EntryRva, GuestEsp);

        /* The loader APC frame goes on the dedicated loader stack for the
         * INITIAL thread (the image entry's own stack is separate here); a
         * guest thread builds the same frame on the stack kernel32 gave it. */
        ApcEsp = Wow64pBuildLoaderApcFrame((PBYTE)ApcStack + 0x40000 - 0x100,
                                           &RunCtx, Ntdll32);
        Wow64pInitStartContext(&StartCtx,
                               (ULONG)(ULONG_PTR)KiUserApcDispatcher32, ApcEsp);

        DbgPrint("[FEX-WOW64C] entering BTCpuSimulate via loader (apcDisp=%08lx esp=%08lx LdrInit=%p runEip=%08lx exit=%p)\n",
                 StartCtx.Eip, StartCtx.Esp, LdrInitThunk32,
                 ImageBase32 + EntryRva, ExitStub);

        /* The guest image entry runs on GuestStack — those are the bounds
         * i386 RtlDispatchException validates fs:[0] SEH records against. */
        Wow64pThreadInit(&StartCtx,
                         (ULONG)(ULONG_PTR)GuestStack + GuestStackSize,
                         (ULONG)(ULONG_PTR)GuestStack,
                         (ULONG)(ULONG_PTR)GuestStack,
                         "initial thread");
    }

    DbgPrint("[FEX-WOW64C] FAIL: BTCpuSimulate returned\n");
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
}

/*
 * Wow64ProcessPendingCrossProcessItems — in Win10, drains a work list of
 * things other processes want done to THIS process's guest state.
 *
 * Still a no-op here, and deliberately so rather than pending. Cross-process
 * thread creation works (task #64, Wow64CreateForeignGuestThread) and it
 * needs no list: the creator points the new thread at the TARGET's own
 * Wow64pGuestThreadStart, so the per-thread guest setup already happens in
 * the target, on the new thread, by the target's own code. That is the
 * property the work list exists to provide, obtained by construction instead
 * of by deferral -- so there is genuinely nothing queued to drain, and an
 * empty body is the honest implementation rather than a stub.
 *
 * It would start to matter if we ever needed the target to act at a moment it
 * chooses rather than at thread entry -- guest memory invalidation after a
 * foreign WriteProcessMemory is the obvious candidate, since a JIT CPU layer
 * would have to drop cached blocks. The native CPU layer has nothing to
 * invalidate, which is why that has not come up.
 */
VOID
WINAPI
Wow64ProcessPendingCrossProcessItems(VOID)
{
}

BOOL
WINAPI
DllMain(HINSTANCE hDll, DWORD dwReason, LPVOID lpReserved)
{
    UNREFERENCED_PARAMETER(lpReserved);

    if (dwReason == DLL_PROCESS_ATTACH)
    {
        Wow64pSelfBase = (PVOID)hDll;
        /* No DisableThreadLibraryCalls: that's kernel32, and this DLL is
         * deliberately ntdll-only. Thread attach/detach no-op through DllMain. */
        DbgPrint("[WOW64] wow64.dll loaded\n");
    }
    return TRUE;
}
