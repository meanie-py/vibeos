/*
 * kernel_c_helpers.c — Residual C helper functions that cannot be implemented
 * in Rust (variadic, LTO workarounds, macro-generated tables, globals).
 *
 * These functions were previously scattered across *_rust_shims.c files.
 */

/* Phase 1: Rename FORCEINLINE/macro functions BEFORE including headers.
 * This causes the header to create _saved_* inline versions that we can
 * call from our Phase 5 outline function definitions below. */
#define RtlStringCbPrintfA _saved_RtlStringCbPrintfA

/* Rename static const arrays from ps_i.h so we can re-include with
 * external linkage later. The first inclusion (via ntoskrnl.h) creates
 * _static_* copies; the second creates the real names. */
#define PsProcessInfoClass _static_PsProcessInfoClass
#define PsThreadInfoClass  _static_PsThreadInfoClass

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Undo the ntstrsafe.h "poison" macros */
#undef RtlStringVPrintfWorkerA

/* Phase 3: Undef the rename so we can define the real function */
#undef RtlStringCbPrintfA
#undef PsProcessInfoClass
#undef PsThreadInfoClass
#undef SharedUserData
#undef ARM64_UART_TRACE
#undef ARM64_UART_TRACE_HEX


/*
 * SepInitTokensShim — C helper to work around LLVM i386 LTO bug.
 *
 * During fat LTO on i386, LLVM incorrectly inserts unreachable/ud2 after the
 * ExInitializeFastReference call that sets up the system process token. This
 * drops all subsequent code (anonymous logon token creation, return TRUE).
 * The bug does not affect amd64 or arm64.
 *
 * By performing the token initialization in C code (which is not part of the
 * Rust LTO link), we prevent LLVM from applying the misoptimization.
 */
BOOLEAN
SepInitTokensShim(VOID)
{
    PTOKEN tok;

    /* Initialize process token fast reference to NULL */
    ExInitializeFastReference(&PsGetCurrentProcess()->Token, NULL);

    /* Create the system process token */
    tok = SepCreateSystemProcessToken();
    ObInitializeFastReference(&PsGetCurrentProcess()->Token, tok);

    /* Create anonymous logon tokens */
    SeAnonymousLogonToken = SepCreateSystemAnonymousLogonToken();
    if (!SeAnonymousLogonToken) return FALSE;

    SeAnonymousLogonTokenNoEveryone = SepCreateSystemAnonymousLogonTokenNoEveryone();
    if (!SeAnonymousLogonTokenNoEveryone) return FALSE;

    return TRUE;
}

/*
 * KdpDprintf — variadic debug printf for KD64 subsystem.
 *
 * The va_arg pull stays in C; format-string parsing + emission live in the
 * Rust ntprintf crate via ntprintf_va_format_a.
 * Only needed when KD64 is Rust (C kdprint.c provides its own when KD64_USE_C).
 */
extern size_t ntprintf_va_format_a(char *dst, size_t cap,
                                   const char *fmt, va_list ap);

#ifndef KD64_USE_C
VOID __cdecl
KdpDprintf(_In_ PCSTR Format, ...)
{
    STRING String;
    SIZE_T Length;
    va_list ap;
    CHAR Buffer[512];
    va_start(ap, Format);
    Length = ntprintf_va_format_a(Buffer, sizeof(Buffer), Format, ap);
    va_end(ap);
    if (Length > MAXUSHORT)
        Length = MAXUSHORT;
    String.Buffer = Buffer;
    String.Length = String.MaximumLength = (USHORT)Length;
    KdpPrintString(&String);
}
#endif

/*
 * RtlStringCbPrintfA — variadic safe string printf.
 *
 * va_arg pull stays in C; format-string parsing + emission via Rust ntprintf.
 * Returns STATUS_BUFFER_OVERFLOW on truncation, STATUS_SUCCESS otherwise,
 * matching the contract of the original RtlStringVPrintfWorkerA helper.
 */
NTSTATUS __cdecl RtlStringCbPrintfA(
    char *pszDest,
    size_t cbDest,
    const char *pszFormat,
    ...)
{
    va_list argList;
    SIZE_T written;

    if (cbDest > NTSTRSAFE_MAX_CCH)
    {
        if (cbDest > 0)
            *pszDest = '\0';
        return STATUS_INVALID_PARAMETER;
    }
    if (cbDest == 0)
        return STATUS_SUCCESS;

    va_start(argList, pszFormat);
    written = ntprintf_va_format_a(pszDest, cbDest, pszFormat, argList);
    va_end(argList);

    /* ntprintf_va_format_a returns the byte count that would have been
     * written excluding NUL. Truncation when written + 1 > cbDest. */
    return (written + 1 > cbDest) ? STATUS_BUFFER_OVERFLOW : STATUS_SUCCESS;
}

/*
 * ExpKdbgExtFileCache / ExpKdbgExtDefWrites — KDBG debug extensions.
 *
 * These call variadic KdbpPrint which cannot be done from Rust.
 * Only needed when CC subsystem is in Rust (C view.c provides the real versions
 * when CC_USE_C is defined).
 */
#if DBG && defined(KDBG) && !defined(CC_USE_C)
#include <kdbg/kdb.h>

extern ULONG CcTotalDirtyPages;
extern ULONG CcDirtyPageThreshold;

BOOLEAN ExpKdbgExtFileCache(ULONG Argc, PCHAR Argv[])
{
    KdbpPrint("  File cache display not available (CC subsystem is in Rust)\n");
    return TRUE;
}

BOOLEAN ExpKdbgExtDefWrites(ULONG Argc, PCHAR Argv[])
{
    KdbpPrint("CcTotalDirtyPages:\t%lu (%lu Kb)\n", CcTotalDirtyPages,
              (CcTotalDirtyPages * PAGE_SIZE) / 1024);
    KdbpPrint("CcDirtyPageThreshold:\t%lu (%lu Kb)\n", CcDirtyPageThreshold,
              (CcDirtyPageThreshold * PAGE_SIZE) / 1024);
    KdbpPrint("MmAvailablePages:\t%lu (%lu Kb)\n", (ULONG)MmAvailablePages,
              (ULONG)((MmAvailablePages * PAGE_SIZE) / 1024));
    KdbpPrint("MmThrottleTop:\t\t%lu (%lu Kb)\n", MmThrottleTop,
              (MmThrottleTop * PAGE_SIZE) / 1024);
    KdbpPrint("MmThrottleBottom:\t%lu (%lu Kb)\n", MmThrottleBottom,
              (MmThrottleBottom * PAGE_SIZE) / 1024);
    KdbpPrint("MmModifiedPageListHead.Total:\t%lu (%lu Kb)\n",
              (ULONG)MmModifiedPageListHead.Total,
              (ULONG)((MmModifiedPageListHead.Total * PAGE_SIZE) / 1024));

    if (CcTotalDirtyPages >= CcDirtyPageThreshold)
        KdbpPrint("CcTotalDirtyPages above the threshold, writes should be throttled\n");
    else if (CcTotalDirtyPages + 64 >= CcDirtyPageThreshold)
        KdbpPrint("CcTotalDirtyPages within 64 pages of threshold, writes may be throttled\n");
    else
        KdbpPrint("CcTotalDirtyPages below the threshold, writes should not be throttled\n");

    return TRUE;
}
#endif

/*
 * ExpKdbgExtHandle — KDBG debug extension for handle table display.
 *
 * Uses variadic KdbpPrint and C-only string functions (isdigit, strtoul).
 * Previously in ex_rust_shims.c.
 */
#if DBG && defined(KDBG)
#include <kdbg/kdb.h>
#include <ctype.h>

/* strtoulptr — pointer-width strtoul */
#ifdef _WIN64
#define strtoulptr strtoull
#else
#define strtoulptr strtoul
#endif

#define SizeOfHandle(x) (sizeof(HANDLE) * (x))

/* HandleTableListHead and HandleTableListLock are globals from Rust handle.rs */
extern LIST_ENTRY HandleTableListHead;
extern EX_PUSH_LOCK HandleTableListLock;

BOOLEAN ExpKdbgExtHandle(ULONG Argc, PCHAR Argv[])
{
    USHORT i;
    char *endptr;
    HANDLE ProcessId;
    EXHANDLE ExHandle;
    PLIST_ENTRY Entry;
    PEPROCESS Process;
    WCHAR KeyPath[256];
    PFILE_OBJECT FileObject;
    PHANDLE_TABLE HandleTable;
    POBJECT_HEADER ObjectHeader;
    PHANDLE_TABLE_ENTRY TableEntry;
    ULONG NeededLength = 0;
    ULONG NameLength;
    PCM_KEY_CONTROL_BLOCK Kcb, CurrentKcb;
    POBJECT_HEADER_NAME_INFO ObjectNameInfo;

    if (Argc > 1)
    {
        i = 0;
        while (Argv[1][i])
        {
            if (!isdigit(Argv[1][i]))
            {
                i = 0;
                break;
            }
            ++i;
        }

        if (i == 0)
        {
            if (!KdbpGetHexNumber(Argv[1], (PVOID)&Process))
            {
                KdbpPrint("Invalid parameter: %s\n", Argv[1]);
                return TRUE;
            }
            ProcessId = PsGetProcessId(Process);
        }
        else
        {
            ProcessId = (HANDLE)strtoulptr(Argv[1], &endptr, 10);
            if (*endptr != '\0')
            {
                KdbpPrint("Invalid parameter: %s\n", Argv[1]);
                return TRUE;
            }
        }
    }
    else
    {
        ProcessId = PsGetCurrentProcessId();
    }

    for (Entry = HandleTableListHead.Flink;
         Entry != &HandleTableListHead;
         Entry = Entry->Flink)
    {
        HandleTable = CONTAINING_RECORD(Entry, HANDLE_TABLE, HandleTableList);
        if (ProcessId != 0 && HandleTable->UniqueProcessId != ProcessId)
            continue;

        KdbpPrint("\n");
        KdbpPrint("Handle table at %p with %d entries in use\n",
                  HandleTable, HandleTable->HandleCount);

        ExHandle.Value = 0;
        while ((TableEntry = ExpLookupHandleTableEntry(HandleTable, ExHandle)))
        {
            if ((TableEntry->Object) &&
                (TableEntry->NextFreeTableEntry != -2))
            {
                ObjectHeader = (POBJECT_HEADER)((ULONG_PTR)TableEntry->Object & ~OBJ_HANDLE_ATTRIBUTES);

                KdbpPrint("%p: Object: %p GrantedAccess: %x Entry: %p\n",
                          ExHandle.Value, &ObjectHeader->Body,
                          TableEntry->GrantedAccess, TableEntry);
                KdbpPrint("Object: %p Type: (%x) %wZ\n",
                          &ObjectHeader->Body, ObjectHeader->Type,
                          &ObjectHeader->Type->Name);
                KdbpPrint("\tObjectHeader: %p\n", ObjectHeader);
                KdbpPrint("\t\tHandleCount: %u PointerCount: %u\n",
                          ObjectHeader->HandleCount, ObjectHeader->PointerCount);

                if (ObjectHeader->Type == IoFileObjectType)
                {
                    FileObject = (PFILE_OBJECT)&ObjectHeader->Body;
                    KdbpPrint("\t\tName: %wZ\n", &FileObject->FileName);
                }
                else if (ObjectHeader->Type == CmpKeyObjectType)
                {
                    Kcb = ((PCM_KEY_BODY)&ObjectHeader->Body)->KeyControlBlock;
                    NeededLength = 0;
                    CurrentKcb = Kcb;
                    while (CurrentKcb)
                    {
                        if (CurrentKcb->NameBlock->Compressed)
                            NeededLength += CurrentKcb->NameBlock->NameLength + 1;
                        else
                            NeededLength += CurrentKcb->NameBlock->NameLength / sizeof(WCHAR) + 1;
                        CurrentKcb = CurrentKcb->ParentKcb;
                    }

                    if (NeededLength < 256)
                    {
                        KeyPath[NeededLength] = UNICODE_NULL;
                        CurrentKcb = Kcb;
                        while (CurrentKcb)
                        {
                            if (CurrentKcb->NameBlock->Compressed)
                            {
                                NameLength = CurrentKcb->NameBlock->NameLength;
                                NeededLength -= NameLength + 1;
                                for (i = 0; i < NameLength; ++i)
                                    KeyPath[NeededLength + 1 + i] =
                                        ((PUCHAR)CurrentKcb->NameBlock->Name)[i];
                            }
                            else
                            {
                                NameLength = CurrentKcb->NameBlock->NameLength / sizeof(WCHAR);
                                NeededLength -= NameLength + 1;
                                for (i = 0; i < NameLength; ++i)
                                    KeyPath[NeededLength + 1 + i] =
                                        CurrentKcb->NameBlock->Name[i];
                            }
                            KeyPath[NeededLength] = OBJ_NAME_PATH_SEPARATOR;
                            CurrentKcb = CurrentKcb->ParentKcb;
                        }
                        KdbpPrint("\t\tName: %S\n", KeyPath);
                    }
                }

                ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
                if (ObjectNameInfo && ObjectNameInfo->Name.Buffer)
                    KdbpPrint("\t\tName: %wZ\n", &ObjectNameInfo->Name);
            }
            ExHandle.Value += SizeOfHandle(1);
        }
    }

    return TRUE;
}
#endif /* DBG && KDBG */

/* ===================================================================
 * PsProcessInfoClass / PsThreadInfoClass — re-include ps_i.h with
 * external linkage (override 'static' to nothing).
 * The first inclusion (via ntoskrnl.h) created _static_* copies.
 * This second inclusion creates the real names with external linkage.
 * Previously in ps_rust_shims.c.
 * =================================================================== */
#pragma push_macro("static")
#define static
#include "internal/ps_i.h"
#pragma pop_macro("static")

/* ===================================================================
 * Global variables — previously defined in ps/ps_rust_shims.c.
 * Needed because the PS C files are replaced by Rust (ntoskrnl-rust/src/ps/).
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
 * ARM64_UART_TRACE / ARM64_UART_TRACE_HEX — UART debug output functions.
 * Called from Rust via extern "C" on ARM64 only.
 * Previously in ps_rust_shims.c.
 * =================================================================== */

VOID NTAPI ARM64_UART_TRACE(PCSTR msg)
{
#ifdef _M_ARM64
    extern volatile PVOID Arm64UartKernelBase;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) { while (*msg) *u = (ULONG)*msg++; }
#else
    UNREFERENCED_PARAMETER(msg);
#endif
}

VOID NTAPI ARM64_UART_TRACE_HEX(PCSTR prefix, ULONG_PTR val)
{
#ifdef _M_ARM64
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
    UNREFERENCED_PARAMETER(prefix);
    UNREFERENCED_PARAMETER(val);
#endif
}

/* ===================================================================
 * MM subsystem C-only helpers — previously in mm_rust_shims.c.
 *
 * These functions cannot be implemented in Rust because they:
 * - Use C macros (__FILE__, __LINE__)
 * - Require #include of avlsupp.c for AVL tree algorithms
 * - Bridge calling conventions (i386 fastcall → cdecl)
 * - Define globals with C-specific initializers
 * =================================================================== */

/* --- Globals from section.c (Rust provides no C definition) --- */
#if !defined(MM_SECTION_C_ACTIVE)
KEVENT MmWaitPageEvent;
ULONG_PTR MmSubsectionBase;
#endif

/* MmVerifyImageIsOkForMpUse — CONFIG_SMP only, was in sysldr.c */
#if !defined(MM_SYSLDR_C_ACTIVE) && defined(CONFIG_SMP)
BOOLEAN
NTAPI
MmVerifyImageIsOkForMpUse(
    _In_ PVOID BaseAddress)
{
    PIMAGE_NT_HEADERS NtHeaders;
    PAGED_CODE();

    NtHeaders = RtlImageNtHeader(BaseAddress);
    if (!NtHeaders)
        return TRUE;

    if ((KeNumberProcessors > 1) &&
        (NtHeaders->FileHeader.Characteristics & IMAGE_FILE_UP_SYSTEM_ONLY))
    {
        return FALSE;
    }
    return TRUE;
}
#endif

/* i386 cdecl InterlockedExchange/InterlockedCompareExchange wrappers — removed.
 * All Rust callers now declare these as `extern "fastcall"` (i386) or
 * `extern "C"` (amd64/arm64) and link against the Rust impls in exintrin.rs.
 * No remaining caller asks for the cdecl `_Name` decoration on i386. */

/* --- Section segment lock/unlock — uses __FILE__/__LINE__ C macros --- */
#include <mm/ARM3/miarm.h>

/* Undef the macros from mm.h so we can define outline functions */
#undef MmLockSectionSegment
#undef MmUnlockSectionSegment
#undef MmGetPageEntrySectionSegment

void NTAPI MmLockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmLockSectionSegment(Segment, __FILE__, __LINE__); }

void NTAPI MmUnlockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmUnlockSectionSegment(Segment, __FILE__, __LINE__); }

ULONG_PTR NTAPI MmGetPageEntrySectionSegment(
    IN PMM_SECTION_SEGMENT Segment, IN PLARGE_INTEGER Offset)
{ return _MmGetPageEntrySectionSegment(Segment, Offset, __FILE__, __LINE__); }

/* AVL Tree support — moved to mm-avl rlib (`sdk/lib/rust/mm-avl/`).
 * Trampolines (`RtlRightChildAvl` / `RtlpFindAvlTableNodeOrParent` / etc.)
 * live in mm/mm_rust_shims.c and one-line-delegate to `_rs_*` exports.
 * The historical `#include <avlsupp.c>` path used to live here. */

/* KfRaiseIrql/KfLowerIrql — moved to Rust on amd64.
 * `shim_exports.rs::KfRaiseIrql/KfLowerIrql` (#[cfg(target_arch = "x86_64")])
 * delegate to `ntkrnl_sys::inline::mm::KfRaiseIrql/KfLowerIrql` which uses
 * the inline `mov cr8, ...` asm in `inline/ke.rs::KfRaiseIrql_amd64`.
 * i386: HAL exports as __fastcall — Rust callers declare `extern "fastcall"`.
 * ARM64: HAL cdecl — Rust callers declare `extern "C"`. */

/* RustShimRtlFindSetBits / RustShimKdbpGetHexNumber removed (Phase 3) —
 * Rust callers in `ARM3/kdbg.rs` now declare these as `extern "system"`
 * directly, which generates the right `_Name@N` stdcall decoration on
 * i386 and cdecl on amd64/arm64. */

