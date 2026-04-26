/*
 * ex_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust EX subsystem calls via extern declarations.
 *
 * Common shims (CONTAINING_RECORD, InsertTailList, ProbeForWriteHandle, etc.)
 * are provided by ob/ob_rust_shims.c and tier1_rust_shims.c.
 *
 * NOTE: KeGetPreviousMode is already provided by ke/thrdobj.c.
 *       RtlMoveMemory is already provided by librtl (rtlmem.s / Rust).
 *       RtlEqualLuid is already provided by libreactos_luid.a.
 */

/* ========================================================================
 * Phase 1: Rename FORCEINLINE functions BEFORE including headers
 *
 * By #defining the function name to _saved_Name before headers are parsed,
 * the FORCEINLINE definition creates a _saved_Name static inline. After
 * headers, we #undef and define real non-inline functions with original names.
 * ======================================================================== */

/* ke_x.h FORCEINLINE functions (pulled in via ke.h → ntoskrnl.h) */
#define KiAcquireDispatcherLock          _saved_KiAcquireDispatcherLock
#define KiReleaseDispatcherLock          _saved_KiReleaseDispatcherLock
#define KiAcquireThreadLock              _saved_KiAcquireThreadLock
#define KiReleaseThreadLock              _saved_KiReleaseThreadLock

/* spinlock.h FORCEINLINE functions */
#define KxAcquireSpinLock                _saved_KxAcquireSpinLock
#define KxReleaseSpinLock                _saved_KxReleaseSpinLock

/* ex.h FORCEINLINE functions */
#define _ExAcquireFastMutex              _saved__ExAcquireFastMutex
#define _ExReleaseFastMutex              _saved__ExReleaseFastMutex
#define _ExAcquireFastMutexUnsafe        _saved__ExAcquireFastMutexUnsafe
#define _ExReleaseFastMutexUnsafe        _saved__ExReleaseFastMutexUnsafe
#define _ExEnterCriticalRegionAndAcquireFastMutexUnsafe \
                                         _saved__ExEnterCriticalRegionAndAcquireFastMutexUnsafe
#define _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion \
                                         _saved__ExReleaseFastMutexUnsafeAndLeaveCriticalRegion
#define _ExTryToAcquireFastMutex         _saved__ExTryToAcquireFastMutex
#define ExGetRunRefForGivenProcessor     _saved_ExGetRunRefForGivenProcessor
#define ExWaitOnPushLock                 _saved_ExWaitOnPushLock

/* rtlfuncs.h FORCEINLINE functions */
#define PopEntryList                     _saved_PopEntryList

/* ntstrsafe.h static inline functions — rename before include */
#define RtlStringCbPrintfA               _saved_RtlStringCbPrintfA
#define RtlStringCbCopyA                 _saved_RtlStringCbCopyA

/* ========================================================================
 * Phase 2: Include headers
 * ======================================================================== */

/* Suppress DPRINT (this file is PCH-skipped) */
#ifndef DPRINT
#define DPRINT(...) do {} while(0)
#endif

#include <ntoskrnl.h>

/* ntstrsafe.h for RtlStringCbPrintfA/CopyA — NOT included from ntoskrnl.h.
 * These are static inline (NTSTRSAFEAPI/NTSTRSAFEVAPI) so no external symbol.
 * We renamed them in Phase 1 so they define as _saved_* names. */
#include <ntstrsafe.h>

/* Undo the ntstrsafe.h "poison" macros that prevent use of worker functions */
#undef RtlStringVPrintfWorkerA
#undef RtlStringCopyWorkerA

/* ========================================================================
 * Phase 2.5: Capture macros in static inline trampolines BEFORE undef
 * ======================================================================== */

/* KeGetContextSwitches: macro in internal/ke.h */
static __inline ULONG _saved_KeGetContextSwitches(PKPRCB Prcb)
{
    return KeGetContextSwitches(Prcb);
}

/* MmGetSystemAddressForMdlSafe: macro in mmfuncs.h */
static __inline PVOID _saved_MmGetSystemAddressForMdlSafe(PMDL Mdl, ULONG Priority)
{
    return MmGetSystemAddressForMdlSafe(Mdl, Priority);
}

/* ========================================================================
 * Phase 3: Undef macros and Phase 1 renames
 * ======================================================================== */

/* Original Phase 2 undefs */
#undef KeGetContextSwitches
#undef MmGetSystemAddressForMdlSafe
#undef ObpGetHandleObject
#undef ProbeForWriteBoolean
#undef ProbeForWriteLangId
#undef InterlockedExchangeAdd64

/* Phase 1 renames */
#undef KiAcquireDispatcherLock
#undef KiReleaseDispatcherLock
#undef KiAcquireThreadLock
#undef KiReleaseThreadLock
#undef KxAcquireSpinLock
#undef KxReleaseSpinLock
#undef _ExAcquireFastMutex
#undef _ExReleaseFastMutex
#undef _ExAcquireFastMutexUnsafe
#undef _ExReleaseFastMutexUnsafe
#undef _ExEnterCriticalRegionAndAcquireFastMutexUnsafe
#undef _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion
#undef _ExTryToAcquireFastMutex
#undef ExGetRunRefForGivenProcessor
#undef ExWaitOnPushLock
#undef PopEntryList
#undef RtlStringCbPrintfA
#undef RtlStringCbCopyA

/* i386 cdecl wrappers need RtlInitEmptyUnicodeString without NTAPI — undef if needed */
#ifdef _M_IX86
/* ob_rust_shims.c provides NTAPI (stdcall) version — we need cdecl version too.
 * On i386, bindgen extern "C" = cdecl = _Name, vs NTAPI = stdcall = _Name@12.
 * They're different symbols, so no conflict. */
#endif

/* ========================================================================
 * Phase 4: Real (non-static, non-inline) function implementations
 * ======================================================================== */

/* --- KeGetContextSwitches --- */
ULONG KeGetContextSwitches(PKPRCB Prcb)
{
    return _saved_KeGetContextSwitches(Prcb);
}

/* --- MmGetSystemAddressForMdlSafe --- */
PVOID MmGetSystemAddressForMdlSafe(PMDL Mdl, ULONG Priority)
{
    return _saved_MmGetSystemAddressForMdlSafe(Mdl, Priority);
}

/* --- ObpGetHandleObject: macro in internal/ob.h ---
 * #define ObpGetHandleObject(x) ((POBJECT_HEADER)((ULONG_PTR)(x)->Object & ~OBJ_HANDLE_ATTRIBUTES)) */
POBJECT_HEADER ObpGetHandleObject(PHANDLE_TABLE_ENTRY Entry)
{
    return (POBJECT_HEADER)((ULONG_PTR)Entry->Object & ~OBJ_HANDLE_ATTRIBUTES);
}

/* --- ProbeForWriteBoolean: probe.h macro wrapping ProbeForWriteGenericType --- */
void ProbeForWriteBoolean(PBOOLEAN Address)
{
    volatile BOOLEAN *Vol = (volatile BOOLEAN *)Address;
    BOOLEAN Temp = *Vol;
    *Vol = Temp;
}

/* --- ProbeForWriteLangId: probe.h macro wrapping ProbeForWriteGenericType --- */
void ProbeForWriteLangId(LANGID *Address)
{
    volatile LANGID *Vol = (volatile LANGID *)Address;
    LANGID Temp = *Vol;
    *Vol = Temp;
}

/* --- InterlockedExchangeAdd64: intrinsic _InterlockedExchangeAdd64 --- */
LONGLONG NTAPI InterlockedExchangeAdd64(volatile LONGLONG *Addend, LONGLONG Value)
{
    return _InterlockedExchangeAdd64(Addend, Value);
}

/* ========================================================================
 * KE dispatcher/thread lock functions moved to ke/ke_rust_shims.c
 * (canonical location). Do not duplicate here.
 * ======================================================================== */

/* ========================================================================
 * PopEntryList (rtlfuncs.h FORCEINLINE)
 *
 * Bindgen declares as extern "C" — no NTAPI.
 * ======================================================================== */

PSINGLE_LIST_ENTRY PopEntryList(IN PSINGLE_LIST_ENTRY ListHead)
{
    return _saved_PopEntryList(ListHead);
}

/* ========================================================================
 * Fast Mutex functions (ex.h FORCEINLINE)
 *
 * Rust fmutex.rs declares these as extern "C" (cdecl on all arches).
 * On i386: cdecl produces __ExAcquireFastMutex (_ prefix + _ in name).
 * No NTAPI — these must be plain cdecl.
 * ======================================================================== */

VOID _ExAcquireFastMutex(IN PFAST_MUTEX FastMutex)
{
    _saved__ExAcquireFastMutex(FastMutex);
}

VOID _ExReleaseFastMutex(IN OUT PFAST_MUTEX FastMutex)
{
    _saved__ExReleaseFastMutex(FastMutex);
}

VOID _ExAcquireFastMutexUnsafe(IN PFAST_MUTEX FastMutex)
{
    _saved__ExAcquireFastMutexUnsafe(FastMutex);
}

VOID _ExReleaseFastMutexUnsafe(IN OUT PFAST_MUTEX FastMutex)
{
    _saved__ExReleaseFastMutexUnsafe(FastMutex);
}

VOID _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(IN OUT PFAST_MUTEX FastMutex)
{
    _saved__ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex);
}

VOID _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(IN OUT PFAST_MUTEX FastMutex)
{
    _saved__ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex);
}

BOOLEAN _ExTryToAcquireFastMutex(IN OUT PFAST_MUTEX FastMutex)
{
    return _saved__ExTryToAcquireFastMutex(FastMutex);
}

/* ========================================================================
 * ExGetRunRefForGivenProcessor (ex.h FORCEINLINE)
 *
 * Bindgen declares as extern "C" — no NTAPI.
 * ======================================================================== */

PEX_RUNDOWN_REF ExGetRunRefForGivenProcessor(
    IN PEX_RUNDOWN_REF_CACHE_AWARE RunRefCacheAware,
    IN ULONG ProcNumber)
{
    return _saved_ExGetRunRefForGivenProcessor(RunRefCacheAware, ProcNumber);
}

/* ========================================================================
 * ExWaitOnPushLock (ex.h FORCEINLINE)
 *
 * Bindgen declares as extern "C" — no NTAPI.
 * ======================================================================== */

VOID ExWaitOnPushLock(PEX_PUSH_LOCK PushLock)
{
    _saved_ExWaitOnPushLock(PushLock);
}

/* ========================================================================
 * InterlockedCompareExchange — cdecl version for amd64/arm64.
 *
 * On amd64/arm64, Rust work.rs declares InterlockedCompareExchange as
 * extern "system" (= cdecl on 64-bit). The Rust exintrin.rs provides this.
 * On i386, mm_rust_shims.c provides cdecl wrappers separately.
 *
 * NOT needed here — provided by Rust exintrin.rs on all arches.
 * ======================================================================== */

/* ========================================================================
 * RtlStringCbPrintfA / RtlStringCbCopyA (ntstrsafe.h static inline)
 *
 * These are static __inline in ntstrsafe.h — no linker symbol.
 * Rust declares them as extern "C" (cdecl). We provide real functions.
 *
 * For the variadic RtlStringCbPrintfA, we call the static inline worker
 * RtlStringVPrintfWorkerA directly (poison undefs already applied in Phase 2).
 *
 * RtlStringCbPrintfA is __cdecl (NTSTRSAFEVAPI).
 * RtlStringCbCopyA is NTAPI (NTSTRSAFEAPI).
 * ======================================================================== */

NTSTATUS __cdecl RtlStringCbPrintfA(
    char *pszDest,
    size_t cbDest,
    const char *pszFormat,
    ...)
{
    NTSTATUS Status;
    va_list argList;

    if (cbDest > NTSTRSAFE_MAX_CCH)
    {
        if (cbDest > 0)
            *pszDest = '\0';
        return STATUS_INVALID_PARAMETER;
    }

    va_start(argList, pszFormat);
    Status = RtlStringVPrintfWorkerA(pszDest, cbDest, pszFormat, argList);
    va_end(argList);

    return Status;
}

NTSTATUS NTAPI RtlStringCbCopyA(
    char *pszDest,
    size_t cbDest,
    const char *pszSrc)
{
    size_t cchDest = cbDest / sizeof(char);

    if (cchDest > NTSTRSAFE_MAX_CCH)
    {
        if (cchDest > 0)
            *pszDest = '\0';
        return STATUS_INVALID_PARAMETER;
    }

    return RtlStringCopyWorkerA(pszDest, cchDest, pszSrc);
}

/* ========================================================================
 * i386-only shims
 * ======================================================================== */
#ifdef _M_IX86

/* KeDisableInterrupts / KeRestoreInterrupts — FORCEINLINE per-arch in ke.h.
 * On amd64/arm64 they exist in ke.h and have symbols from other code.
 * On i386 they're FORCEINLINE in i386/ke.h — no symbol. Bindgen
 * declares as extern "C" (cdecl). */
BOOLEAN KeDisableInterrupts(VOID)
{
    ULONG Flags;
    BOOLEAN Return;

    Flags = __readeflags();
    Return = (Flags & EFLAGS_INTERRUPT_MASK) ? TRUE : FALSE;
    _disable();
    return Return;
}

VOID KeRestoreInterrupts(BOOLEAN WereEnabled)
{
    if (WereEnabled) _enable();
}

/* RtlInitEmptyUnicodeString — cdecl version for i386.
 * ob_rust_shims.c provides NTAPI (stdcall = _Name@12).
 * Bindgen declares extern "C" (cdecl = _Name).
 * These are different symbols on i386, so no conflict. */
VOID RtlInitEmptyUnicodeString(PUNICODE_STRING UnicodeString,
                                PWCHAR Buffer,
                                USHORT BufferSize)
{
    UnicodeString->Length = 0;
    UnicodeString->MaximumLength = BufferSize;
    UnicodeString->Buffer = Buffer;
}

#endif /* _M_IX86 */

/* ========================================================================
 * ExpKdbgExtHandle — KDBG debug extension (handle.c, #if DBG && defined(KDBG))
 *
 * This function was in handle.c (now replaced by Rust handle.rs) but the
 * Rust translation omitted it. kdb_cli.c references it. Only compiled for
 * non-ARM64 arches with KDBG enabled.
 * ======================================================================== */
#if DBG && defined(KDBG)
#include <kdbg/kdb.h>
#include <ctype.h>

/* strtoulptr — pointer-width strtoul (from original handle.c) */
#ifdef _WIN64
#define strtoulptr strtoull
#else
#define strtoulptr strtoul
#endif

/* SizeOfHandle — handle size multiplier (from original handle.c) */
#define SizeOfHandle(x) (sizeof(HANDLE) * (x))

/* HandleTableListHead and HandleTableListLock are globals from the
 * original handle.c, now provided by Rust handle.rs */
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
        /* Get EPROCESS address or PID */
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

                /* Specific objects debug prints */
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
