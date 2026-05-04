/*
 * cm_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions in the Configuration Manager (cm_x.h, cmlib.h, exfuncs.h) that the
 * Rust ntoskrnl-rust::config crate calls via extern declarations.
 *
 * Mirrors the pattern from se_rust_shims.c / ke_rust_shims.c.
 *
 * For i386 the Rust callers are split: bindgen-generated code uses
 * extern "C" (=> bare `_Name` cdecl symbol) while hand-written extern
 * "system" blocks expect `_Name@N` stdcall symbols. We emit BOTH variants
 * by using __asm__() symbol renaming on a duplicate NTAPI definition.
 *
 * Note: cmdata.c is still in the C build, so globals it owns (CmFirstTime,
 * CmpVolatileBoot, etc.) are NOT defined here. Globals owned by excluded
 * files (cmsysini.c, cmlazy.c, cmkcbncb.c, cmdelay.c) are defined in their
 * corresponding .rs files.
 */

/* === Phase 1: shadow FORCEINLINE/macro identifiers before include === */

/* cm_x.h FORCEINLINEs */
#define CmpAcquireKcbLockExclusive          _fi_CmpAcquireKcbLockExclusive
#define CmpAcquireKcbLockExclusiveByIndex   _fi_CmpAcquireKcbLockExclusiveByIndex
#define CmpAcquireKcbLockExclusiveByKey     _fi_CmpAcquireKcbLockExclusiveByKey
#define CmpReleaseKcbLockByIndex            _fi_CmpReleaseKcbLockByIndex
#define CmpReleaseKcbLock                   _fi_CmpReleaseKcbLock
#define CmpReleaseKcbLockByKey              _fi_CmpReleaseKcbLockByKey
#define CmpTryToConvertKcbSharedToExclusive _fi_CmpTryToConvertKcbSharedToExclusive
#define CmpConvertKcbSharedToExclusive      _fi_CmpConvertKcbSharedToExclusive

/* ntstrsafe.h static inlines (need to provide outline RtlStringCbPrintfW) */
#define RtlStringCbPrintfW                  _saved_RtlStringCbPrintfW
#define RtlStringCbVPrintfW                 _saved_RtlStringCbVPrintfW

/* basetsd.h `static inline unsigned long PtrToUlong(const void*)` on amd64.
 * On i386 this is a macro and does NOT define a static-inline. Renaming
 * here is harmless on i386 (no-op). */
#define PtrToUlong                          _saved_PtrToUlong

/* cmlib.h `static inline BOOLEAN CmpIsKeyValueSmall(...)` */
#define CmpIsKeyValueSmall                  _saved_CmpIsKeyValueSmall

/* === Phase 2: include headers === */

#include "ntoskrnl.h"
#define NDEBUG
#include "debug.h"

/* === Phase 3: undef the Phase 1 renames so we can declare outlines under
 *      the original names (the renamed `_fi_*` versions remain available). === */

#undef CmpAcquireKcbLockExclusive
#undef CmpAcquireKcbLockExclusiveByIndex
#undef CmpAcquireKcbLockExclusiveByKey
#undef CmpReleaseKcbLockByIndex
#undef CmpReleaseKcbLock
#undef CmpReleaseKcbLockByKey
#undef CmpTryToConvertKcbSharedToExclusive
#undef CmpConvertKcbSharedToExclusive

/* === Phase 4: outline wrappers for renamed FORCEINLINEs === */

VOID CmpAcquireKcbLockExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    _fi_CmpAcquireKcbLockExclusive(Kcb);
}
VOID CmpAcquireKcbLockExclusiveByIndex(IN ULONG Index)
{
    _fi_CmpAcquireKcbLockExclusiveByIndex(Index);
}
VOID CmpAcquireKcbLockExclusiveByKey(IN ULONG ConvKey)
{
    _fi_CmpAcquireKcbLockExclusiveByKey(ConvKey);
}
VOID CmpReleaseKcbLockByIndex(IN ULONG Index)
{
    _fi_CmpReleaseKcbLockByIndex(Index);
}
VOID CmpReleaseKcbLock(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    _fi_CmpReleaseKcbLock(Kcb);
}
VOID CmpReleaseKcbLockByKey(IN ULONG ConvKey)
{
    _fi_CmpReleaseKcbLockByKey(ConvKey);
}
BOOLEAN CmpTryToConvertKcbSharedToExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    return _fi_CmpTryToConvertKcbSharedToExclusive(Kcb);
}
VOID CmpConvertKcbSharedToExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    _fi_CmpConvertKcbSharedToExclusive(Kcb);
}

/* === Phase 4: outline wrappers for cm_x.h macros (KCB/NCB shared lock helpers,
 *      CmpIsKcbLockedExclusive). Body re-implements the macro inline. ===
 * GET_HASH_INDEX is still a macro at this point (undef happens later). */

#undef CmpAcquireKcbLockShared
#undef CmpAcquireKcbLockSharedByIndex
#undef CmpAcquireNcbLockExclusive
#undef CmpAcquireNcbLockExclusiveByKey
#undef CmpReleaseNcbLock
#undef CmpReleaseNcbLockByKey
#undef CmpIsKcbLockedExclusive

VOID CmpAcquireKcbLockShared(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    ExAcquirePushLockShared(&CmpCacheTable[GET_HASH_INDEX(Kcb->ConvKey)].Lock);
}
VOID CmpAcquireKcbLockSharedByIndex(IN ULONG Index)
{
    ExAcquirePushLockShared(&CmpCacheTable[Index].Lock);
}
VOID CmpAcquireNcbLockExclusiveByKey(IN ULONG ConvKey)
{
    ExAcquirePushLockExclusive(&CmpNameCacheTable[GET_HASH_INDEX(ConvKey)].Lock);
}
VOID CmpReleaseNcbLockByKey(IN ULONG ConvKey)
{
    ExReleasePushLock(&CmpNameCacheTable[GET_HASH_INDEX(ConvKey)].Lock);
}
BOOLEAN CmpIsKcbLockedExclusive_outline(IN PCM_KEY_CONTROL_BLOCK Kcb)
{
    return (CmpCacheTable[GET_HASH_INDEX(Kcb->ConvKey)].Owner ==
            KeGetCurrentThread());
}

/* Outline wrappers for stub-callable macros: bodies use the macros, then we
 * alias them under the macro names below (after #undef). */

ULONG _outline_GET_HASH_INDEX(IN ULONG k) { return GET_HASH_INDEX(k); }

/* HvGetCell / HvReleaseCell — macros in cmlib.h that call function pointers
 * inside HHIVE. The Rust translations call them as plain C functions.
 * Must undef the macros first. */
#undef HvGetCell
#undef HvReleaseCell

PVOID HvGetCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
{
    return Hive->GetCellRoutine(Hive, Cell);
}
VOID HvReleaseCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
{
    if (Hive->ReleaseCellRoutine)
        Hive->ReleaseCellRoutine(Hive, Cell);
}

/* === Phase 5: undef the assertion / helper macros so we can declare
 *      same-named outline functions for them === */

#undef CMP_ASSERT_REGISTRY_LOCK
#undef CMP_ASSERT_FLUSH_LOCK
#undef CMP_ASSERT_KCB_LOCK
#undef CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK
#undef CMP_ASSERT_REGISTRY_LOCK_OR_LOADING
#undef CMP_IS_CELL_CACHED
#undef GET_HASH_INDEX
#undef ASSERT_VALUE_BIG
#undef IS_NULL_TERMINATED
#undef LANGIDFROMLCID
#undef PtrToUlong
#undef CMTRACE
#undef RtlStringCbPrintfW
#undef RtlStringCbVPrintfW
#undef PtrToUlong
#undef CmpIsKeyValueSmall
#undef CM_CHECK_REGISTRY_SUCCESS
#undef HvGetCellType

/* === Phase 6: cdecl exports for the LLM-as-function macros and the
 *      CmpIsKcbLockedExclusive name. === */

VOID CMP_ASSERT_REGISTRY_LOCK(VOID) { }
VOID CMP_ASSERT_FLUSH_LOCK(IN PVOID h) { (void)h; }
VOID CMP_ASSERT_KCB_LOCK(IN PVOID k) { (void)k; }
VOID CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK(VOID) { }
VOID CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(IN PVOID h) { (void)h; }
ULONG CMP_IS_CELL_CACHED(IN ULONG c) { return ((c & 1) != 0); }
VOID CMTRACE(IN ULONG f, IN PCSTR s, IN PCSTR a1, IN PVOID a2)
{ (void)f; (void)s; (void)a1; (void)a2; }
VOID ASSERT_VALUE_BIG(IN PVOID h, IN ULONG l) { (void)h; (void)l; }
BOOLEAN IS_NULL_TERMINATED(IN PWSTR s, IN ULONG l)
{ (void)s; (void)l; return TRUE; }

/* GET_HASH_INDEX / LANGIDFROMLCID / PtrToUlong — alias to the outline body */
__attribute__((alias("_outline_GET_HASH_INDEX")))
ULONG GET_HASH_INDEX(IN ULONG k);

ULONG LANGIDFROMLCID(IN LCID l) { return (USHORT)l; }
/* PtrToUlong: cdecl outline (macro/static-inline shadowed via Phase 1 rename) */
ULONG PtrToUlong(IN PVOID p) { return (ULONG)(ULONG_PTR)p; }

/* CmpIsKcbLockedExclusive cdecl alias to the outline */
__attribute__((alias("CmpIsKcbLockedExclusive_outline")))
BOOLEAN CmpIsKcbLockedExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb);

/* ============================================================================
 * NTAPI variants — symbol-renamed via __asm__() so each gets the `_Name@N`
 * decoration on i386 mingw without colliding with the cdecl symbols above.
 * Rust extern "system" callers resolve against these.
 * ============================================================================
 */

#if defined(__i386__) || defined(_M_IX86)

VOID NTAPI _shim_CmpAcquireKcbLockShared(IN PCM_KEY_CONTROL_BLOCK Kcb)
    __asm__("_CmpAcquireKcbLockShared@4");
VOID NTAPI _shim_CmpAcquireKcbLockShared(IN PCM_KEY_CONTROL_BLOCK Kcb)
{ CmpAcquireKcbLockShared(Kcb); }

VOID NTAPI _shim_CmpAcquireKcbLockSharedByIndex(IN ULONG Index)
    __asm__("_CmpAcquireKcbLockSharedByIndex@4");
VOID NTAPI _shim_CmpAcquireKcbLockSharedByIndex(IN ULONG Index)
{ CmpAcquireKcbLockSharedByIndex(Index); }

BOOLEAN NTAPI _shim_CmpIsKcbLockedExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb)
    __asm__("_CmpIsKcbLockedExclusive@4");
BOOLEAN NTAPI _shim_CmpIsKcbLockedExclusive(IN PCM_KEY_CONTROL_BLOCK Kcb)
{ return CmpIsKcbLockedExclusive_outline(Kcb); }

PVOID NTAPI _shim_HvGetCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
    __asm__("_HvGetCell@8");
PVOID NTAPI _shim_HvGetCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
{ return Hive->GetCellRoutine(Hive, Cell); }

VOID NTAPI _shim_HvReleaseCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
    __asm__("_HvReleaseCell@8");
VOID NTAPI _shim_HvReleaseCell(IN PHHIVE Hive, IN HCELL_INDEX Cell)
{
    if (Hive->ReleaseCellRoutine)
        Hive->ReleaseCellRoutine(Hive, Cell);
}

ULONG NTAPI _shim_HvGetCellType(IN HCELL_INDEX Cell)
    __asm__("_HvGetCellType@4");
ULONG NTAPI _shim_HvGetCellType(IN HCELL_INDEX Cell)
{
    /* HvGetCellType macro from hivedata.h:
       ((ULONG)(((Cell) & HCELL_TYPE_MASK) >> HCELL_TYPE_SHIFT)) */
    return ((Cell & HCELL_TYPE_MASK) >> HCELL_TYPE_SHIFT);
}

VOID NTAPI _shim_ExInitializeWorkItem(
    PWORK_QUEUE_ITEM Item,
    PWORKER_THREAD_ROUTINE Routine,
    PVOID Context) __asm__("_ExInitializeWorkItem@12");
VOID NTAPI _shim_ExInitializeWorkItem(
    PWORK_QUEUE_ITEM Item,
    PWORKER_THREAD_ROUTINE Routine,
    PVOID Context)
{
    Item->WorkerRoutine = Routine;
    Item->Parameter = Context;
    Item->List.Flink = NULL;
}

/* Stub-as-function NTAPI exports */
VOID NTAPI _shim_CMP_ASSERT_REGISTRY_LOCK(VOID)
    __asm__("_CMP_ASSERT_REGISTRY_LOCK@0");
VOID NTAPI _shim_CMP_ASSERT_REGISTRY_LOCK(VOID) { }

VOID NTAPI _shim_CMP_ASSERT_FLUSH_LOCK(IN PVOID h)
    __asm__("_CMP_ASSERT_FLUSH_LOCK@4");
VOID NTAPI _shim_CMP_ASSERT_FLUSH_LOCK(IN PVOID h) { (void)h; }

VOID NTAPI _shim_CMP_ASSERT_KCB_LOCK(IN PVOID k)
    __asm__("_CMP_ASSERT_KCB_LOCK@4");
VOID NTAPI _shim_CMP_ASSERT_KCB_LOCK(IN PVOID k) { (void)k; }

VOID NTAPI _shim_CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(IN PVOID h)
    __asm__("_CMP_ASSERT_REGISTRY_LOCK_OR_LOADING@4");
VOID NTAPI _shim_CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(IN PVOID h) { (void)h; }

ULONG NTAPI _shim_GET_HASH_INDEX(IN ULONG k)
    __asm__("_GET_HASH_INDEX@4");
ULONG NTAPI _shim_GET_HASH_INDEX(IN ULONG k) { return _outline_GET_HASH_INDEX(k); }

VOID NTAPI _shim_ASSERT_VALUE_BIG(IN PVOID h, IN ULONG l)
    __asm__("_ASSERT_VALUE_BIG@8");
VOID NTAPI _shim_ASSERT_VALUE_BIG(IN PVOID h, IN ULONG l) { (void)h; (void)l; }

ULONG NTAPI _shim_LANGIDFROMLCID(IN LCID l)
    __asm__("_LANGIDFROMLCID@4");
ULONG NTAPI _shim_LANGIDFROMLCID(IN LCID l) { return (USHORT)l; }

ULONG NTAPI _shim_PtrToUlong(IN PVOID p)
    __asm__("_PtrToUlong@4");
ULONG NTAPI _shim_PtrToUlong(IN PVOID p) { return (ULONG)(ULONG_PTR)p; }

/* CmpIsKeyValueSmall NTAPI shim — calls the renamed static inline directly. */
BOOLEAN NTAPI _shim_CmpIsKeyValueSmall(OUT PULONG RealLength, IN ULONG Length)
    __asm__("_CmpIsKeyValueSmall@8");
BOOLEAN NTAPI _shim_CmpIsKeyValueSmall(OUT PULONG RealLength, IN ULONG Length)
{ return _saved_CmpIsKeyValueSmall(RealLength, Length); }

/* CMP_IS_CELL_CACHED — also called via extern "system" (NTAPI = _Name@4) */
BOOLEAN NTAPI _shim_CMP_IS_CELL_CACHED(IN ULONG c)
    __asm__("_CMP_IS_CELL_CACHED@4");
BOOLEAN NTAPI _shim_CMP_IS_CELL_CACHED(IN ULONG c) { return ((c & 1) != 0); }

#endif /* i386 */

/* === Common cdecl outlines for static-inline shadowing (all arches) === */

/* RtlStringCbPrintfW outline — bypasses the static __inline rename above. */
NTSTATUS __cdecl RtlStringCbPrintfW(WCHAR *dst, size_t cbDest, const WCHAR *fmt, ...)
{
    NTSTATUS s;
    va_list args;
    va_start(args, fmt);
    s = _saved_RtlStringCbVPrintfW(dst, cbDest, fmt, args);
    va_end(args);
    return s;
}

/* CmpIsKeyValueSmall — cdecl outline that calls the renamed inline body. */
BOOLEAN CmpIsKeyValueSmall(OUT PULONG RealLength, IN ULONG Length)
{
    return _saved_CmpIsKeyValueSmall(RealLength, Length);
}

/* HvGetCellType cdecl outline (macro on all arches). */
ULONG HvGetCellType(IN HCELL_INDEX Cell)
{
    return ((Cell & HCELL_TYPE_MASK) >> HCELL_TYPE_SHIFT);
}

/* CM_CHECK_REGISTRY_SUCCESS — function-style macro on cmlib.h. */
BOOLEAN CM_CHECK_REGISTRY_SUCCESS(IN ULONG Status)
{
    /* Returns TRUE iff Status == CM_CHECK_REGISTRY_GOOD (0). */
    return Status == 0;
}
