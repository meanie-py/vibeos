/*
 * nls_stub.c — NLS globals and runtime stubs for host-side testing
 */
#include "include/nt_host.h"
#include <stdio.h>

/*
 * RtlStringCbPrintfA — safe string formatting (ntstrsafe.h substitute).
 * Used by RtlRangeList test for file:line formatting.
 */
NTSTATUS RtlStringCbPrintfA(char *pszDest, SIZE_T cbDest, const char *pszFormat, ...) {
    if (!pszDest || cbDest == 0) return STATUS_INVALID_PARAMETER;
    va_list ap;
    va_start(ap, pszFormat);
    int ret = vsnprintf(pszDest, cbDest, pszFormat, ap);
    va_end(ap);
    if (ret < 0 || (SIZE_T)ret >= cbDest) {
        pszDest[cbDest - 1] = '\0';
        return STATUS_BUFFER_OVERFLOW;
    }
    return STATUS_SUCCESS;
}

/* No DBCS code pages on host */
BOOLEAN NlsMbOemCodePageTag = FALSE;

/* KmtIsMultiProcessorBuild — FALSE on single-threaded host */
BOOLEAN KmtIsMultiProcessorBuild = FALSE;

/*
 * IRQL tracking — simple state variable for host-side testing.
 * Functions match Windows kernel signatures (KIRQL = unsigned char).
 */
static KIRQL _current_irql = PASSIVE_LEVEL;

void KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql) {
    *OldIrql = _current_irql;
    _current_irql = NewIrql;
}

void KeLowerIrql(KIRQL NewIrql) {
    _current_irql = NewIrql;
}

KIRQL KeGetCurrentIrql(void) {
    return _current_irql;
}

KIRQL KeRaiseIrqlToDpcLevel(void) {
    KIRQL old = _current_irql;
    _current_irql = DISPATCH_LEVEL;
    return old;
}

KIRQL KeRaiseIrqlToSynchLevel(void) {
    /* UP build: SynchLevel = DISPATCH_LEVEL */
    KIRQL old = _current_irql;
    _current_irql = DISPATCH_LEVEL;
    return old;
}

/*
 * Actual function symbols for RtlFillMemory / RtlMoveMemory.
 * Normally these are macros (memset/memmove), but the RtlMemory test
 * does #undef to test the raw function exports. We provide real
 * functions so the linker can resolve them after #undef.
 */
#undef RtlFillMemory
#undef RtlMoveMemory

void RtlFillMemory(void *Dest, SIZE_T Length, UCHAR Fill) {
    memset(Dest, Fill, Length);
}

void RtlMoveMemory(void *Dest, const void *Src, SIZE_T Length) {
    memmove(Dest, Src, Length);
}

/*
 * Actual function symbols for ExInterlocked* list operations.
 * ExDoubleList/ExSingleList tests #undef the macros to test function exports.
 */
#undef ExInterlockedInsertHeadList
#undef ExInterlockedInsertTailList
#undef ExInterlockedRemoveHeadList
#undef ExInterlockedPushEntryList
#undef ExInterlockedPopEntryList

PLIST_ENTRY ExInterlockedInsertHeadList(
    PLIST_ENTRY ListHead, PLIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PLIST_ENTRY OldFirst = IsListEmpty(ListHead) ? NULL : ListHead->Flink;
    InsertHeadList(ListHead, Entry);
    return OldFirst;
}

PLIST_ENTRY ExInterlockedInsertTailList(
    PLIST_ENTRY ListHead, PLIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PLIST_ENTRY OldLast = IsListEmpty(ListHead) ? NULL : ListHead->Blink;
    InsertTailList(ListHead, Entry);
    return OldLast;
}

PLIST_ENTRY ExInterlockedRemoveHeadList(
    PLIST_ENTRY ListHead, PKSPIN_LOCK Lock)
{
    (void)Lock;
    if (IsListEmpty(ListHead)) return NULL;
    return RemoveHeadList(ListHead);
}

PSINGLE_LIST_ENTRY ExInterlockedPushEntryList(
    PSINGLE_LIST_ENTRY ListHead, PSINGLE_LIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PSINGLE_LIST_ENTRY OldFirst = ListHead->Next;
    PushEntryList(ListHead, Entry);
    return OldFirst;
}

PSINGLE_LIST_ENTRY ExInterlockedPopEntryList(
    PSINGLE_LIST_ENTRY ListHead, PKSPIN_LOCK Lock)
{
    (void)Lock;
    return PopEntryList(ListHead);
}

/* ====================================================================
 * KTHREAD — fake thread for APC/guarded region testing
 * ==================================================================== */

KTHREAD _host_current_thread = { 0, 0 };

void _host_init_thread(void) {
    InitializeListHead(&_host_current_thread.MutantListHead);
    _host_current_thread.WaitNext = FALSE;
    _host_current_thread.WaitIrql = PASSIVE_LEVEL;
    _host_current_thread.Win32Thread = NULL;
}

BOOLEAN NTAPI KeAreApcsDisabled(VOID) {
    return (_host_current_thread.KernelApcDisable != 0) ||
           (_host_current_thread.SpecialApcDisable != 0);
}

void NTAPI KeEnterCriticalRegion(VOID) {
    _host_current_thread.KernelApcDisable--;
}

void NTAPI KeLeaveCriticalRegion(VOID) {
    _host_current_thread.KernelApcDisable++;
}

BOOLEAN NTAPI KeAreAllApcsDisabled(VOID) {
    return (_host_current_thread.SpecialApcDisable != 0) ||
           (KeGetCurrentIrql() >= APC_LEVEL);
}

void NTAPI KeEnterGuardedRegion(VOID) {
    _host_current_thread.SpecialApcDisable--;
}

void NTAPI KeLeaveGuardedRegion(VOID) {
    _host_current_thread.SpecialApcDisable++;
}

/* ====================================================================
 * KmtGetSystemRoutineAddress — wide string function lookup table
 * ==================================================================== */

PVOID KmtGetSystemRoutineAddress(PCWSTR Name) {
    /* Compare 16-bit wide strings (WCHAR with -fshort-wchar) */
    static const WCHAR n1[] = L"KeAreAllApcsDisabled";
    static const WCHAR n2[] = L"KeEnterGuardedRegion";
    static const WCHAR n3[] = L"KeLeaveGuardedRegion";

    if (wcscmp(Name, n1) == 0) return (PVOID)KeAreAllApcsDisabled;
    if (wcscmp(Name, n2) == 0) return (PVOID)KeEnterGuardedRegion;
    if (wcscmp(Name, n3) == 0) return (PVOID)KeLeaveGuardedRegion;
    return NULL;
}

/* ====================================================================
 * RtlUpcaseUnicodeChar / RtlIsValidOemCharacter — simplified for host
 *
 * Real implementation uses NLS tables. Our simplified version:
 * - RtlUpcaseUnicodeChar: a-z → A-Z, everything else unchanged
 * - RtlIsValidOemCharacter: ASCII ≤ 0x7F → valid, else invalid
 *   Also sets *Char to the upcased value.
 * ==================================================================== */

WCHAR NTAPI RtlUpcaseUnicodeChar(WCHAR Source) {
    if (Source >= L'a' && Source <= L'z')
        return Source - L'a' + L'A';
    return Source;
}

BOOLEAN NTAPI RtlIsValidOemCharacter(PWCHAR Char) {
    /* Access via volatile to ensure NULL dereference triggers SIGSEGV for SEH */
    volatile PWCHAR vChar = Char;
    WCHAR c = *vChar;
    WCHAR upcased = RtlUpcaseUnicodeChar(c);
    *vChar = upcased;
    /* '?' (0x3F) is the OEM default/substitution character — always invalid */
    if (upcased == L'?')
        return FALSE;
    /* ASCII range (0x00-0x7F) maps to valid OEM characters */
    return (upcased <= 0x7F) ? TRUE : FALSE;
}

/*
 * RtlFindCharInUnicodeString — NOT inline so that NULL pointer dereferences
 * actually trigger SIGSEGV (caught by our SEH signal handler). If inlined,
 * the compiler may optimize away the NULL access as undefined behavior.
 */
#define RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END        1
#define RTL_FIND_CHAR_IN_UNICODE_STRING_COMPLEMENT_CHAR_SET 2
#define RTL_FIND_CHAR_IN_UNICODE_STRING_CASE_INSENSITIVE    4

static WCHAR _UpcaseChar(WCHAR c) {
    return (c >= L'a' && c <= L'z') ? (c - L'a' + L'A') : c;
}

static BOOLEAN _IsCharInStr(WCHAR Char, volatile PCUNICODE_STRING Match, BOOLEAN CaseInsensitive) {
    if (CaseInsensitive) Char = _UpcaseChar(Char);
    for (USHORT i = 0; i < Match->Length / sizeof(WCHAR); i++) {
        WCHAR Other = Match->Buffer[i];
        if (CaseInsensitive) Other = _UpcaseChar(Other);
        if (Char == Other) return TRUE;
    }
    return FALSE;
}

NTSTATUS RtlFindCharInUnicodeString(
    ULONG Flags, PCUNICODE_STRING SearchString,
    PCUNICODE_STRING MatchString, PUSHORT Position)
{
    if (Position == NULL)
        return STATUS_INVALID_PARAMETER;

    *Position = 0;

    if (Flags & ~(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END |
                  RTL_FIND_CHAR_IN_UNICODE_STRING_COMPLEMENT_CHAR_SET |
                  RTL_FIND_CHAR_IN_UNICODE_STRING_CASE_INSENSITIVE))
        return STATUS_INVALID_PARAMETER;

    /* Access through volatile to prevent compiler from optimizing away
       NULL dereferences — tests rely on SEH catching these */
    volatile PCUNICODE_STRING vSearch = SearchString;
    volatile PCUNICODE_STRING vMatch = MatchString;

    BOOLEAN WantToFind = (Flags & RTL_FIND_CHAR_IN_UNICODE_STRING_COMPLEMENT_CHAR_SET) == 0;
    BOOLEAN CaseInsensitive = (Flags & RTL_FIND_CHAR_IN_UNICODE_STRING_CASE_INSENSITIVE) != 0;
    USHORT Length = vSearch->Length / sizeof(WCHAR);

    if (Flags & RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END) {
        for (SHORT i = (SHORT)(Length - 1); i >= 0; i--) {
            if (_IsCharInStr(vSearch->Buffer[i], vMatch, CaseInsensitive) == WantToFind) {
                *Position = i * sizeof(WCHAR);
                return STATUS_SUCCESS;
            }
        }
    } else {
        for (USHORT i = 0; i < Length; i++) {
            if (_IsCharInStr(vSearch->Buffer[i], vMatch, CaseInsensitive) == WantToFind) {
                *Position = (i + 1) * sizeof(WCHAR);
                return STATUS_SUCCESS;
            }
        }
    }
    return STATUS_NOT_FOUND;
}
