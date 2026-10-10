/* rust_cohab_stub.c — Cohabitation shim for fastfat-rust.
 *
 * ALL 34 fastfat .c bodies have been translated to Rust (libfastfat_rust.a).
 * The Rust code contains extern "C" calls to FAT-specific macros/inline
 * functions that have no real linker symbol. This TU provides them.
 *
 * Strategy: DO NOT include fatprocs.h (too many conflicting macros).
 * Use ntifs.h for kernel types, then forward-declare FAT structs we touch.
 */

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmissing-prototypes"
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wincompatible-pointer-types"
#pragma clang diagnostic ignored "-Wredundant-decls"
#pragma clang diagnostic ignored "-Wimplicit-function-declaration"
#pragma clang diagnostic ignored "-Wint-conversion"
#pragma clang diagnostic ignored "-Wpointer-integer-compare"
#pragma clang diagnostic ignored "-Wcast-function-type"

#include <ntifs.h>

/* Undef macros that we want to replace with real linker symbols.
 * NOTE: ExFreeToNPagedLookasideList, ExInterlockedPop/Push, ExInitializeFastMutex,
 *       KeInitializeSpinLock, IoXxx stack helpers, FsRtlEnter/ExitFileSystem,
 *       ObReferenceObject are in rust_cohab_forceinline.c (no DDK includes). */
#undef FsRtlIsAnsiCharacterLegalFat
#undef FsRtlIsLeadDbcsCharacter
#undef FsRtlSetupAdvancedHeader
#undef FsRtlCheckOplockEx
#undef FsRtlOplockBreakH
#undef FsRtlAreThereWaitingFileLocks
#undef FsRtlCheckLockForOplockRequest
#undef FsRtlFastLock
#undef FsRtlCurrentOplockH
#undef FsRtlOplockIsSharedRequest
#undef FsRtlGetSectorSizeInformation
#undef FlagOn
#undef SetFlag
#undef ClearFlag
#undef BooleanFlagOn
#undef Add2Ptr
#undef QuadAlign
#undef BlockAlignTruncate
#undef NodeType
#undef PAGED_CODE
#undef ASSERT
#undef NT_ASSERT
#undef UNREFERENCED_PARAMETER
#undef ARGUMENT_PRESENT
#undef _Analysis_assume_
#undef KdPrintEx
#undef RtlRaiseStatus
#undef RtlCheckBit
#undef RtlConvertLongToLuid
#undef RtlEqualMemory
#undef CanFsdWait
#undef Int32x32To64
#undef UInt32x32To64
#undef CopyUchar4
#undef CopyU4char
#undef RtlOemStringToCountedUnicodeSize
#undef FILE_ACTION_ADDED
#undef FILE_ACTION_REMOVED
#undef FILE_ACTION_MODIFIED
#undef FILE_ACTION_RENAMED_OLD_NAME
#undef FILE_ACTION_RENAMED_NEW_NAME
#undef FO_FILE_MODIFIED
#undef OPLOCK_FLAG_PARENT_OBJECT
#undef STATUS_FILE_CORRUPT_ERROR
#undef IRP_PAGING_IO
#undef SizeOfFullEa
#undef SizeOfPackedEa
#undef ObDereferenceObject
#undef ObfDereferenceObject
#undef ObfReferenceObject
#undef SharedSystemTime
#undef ExGetCurrentResourceThread
#undef CcIsFileCached
#undef IoInitializePriorityInfo
#undef IoRetrievePriorityInfo
#undef _SEH2_TRY
#undef _SEH2_EXCEPT
#undef _SEH2_FINALLY
#undef _SEH2_END
#undef _SEH2_GetExceptionCode
#undef _SEH2_GetExceptionInformation
#undef _SEH2_AbnormalTermination
#undef SEH_TRY_EXCEPT
#undef try_return

/* ======================================================================
 * Forward declarations for FAT structs we need to touch field offsets.
 * We only reference PUBLIC fields here; avoid including fatstruc.h which
 * pulls in the whole macro jungle through fatprocs.h.
 * ====================================================================== */

/* Minimal FAT type aliases */
typedef ULONG  LBO;
typedef ULONG  VBO;
typedef ULONG  LBN;
typedef ULONG  CLUSTER_TYPE;
typedef PVOID  PBCB;  /* BCB = cache manager internal, opaque PVOID in fatstruc.h */

typedef struct _FAT_DATA FAT_DATA, *PFAT_DATA;
typedef struct _VCB VCB, *PVCB;
typedef struct _FCB FCB, *PFCB;
typedef struct _DCB DCB, *PDCB;
typedef struct _CCB CCB, *PCCB;
typedef struct _IRP_CONTEXT IRP_CONTEXT, *PIRP_CONTEXT;
typedef struct _NON_PAGED_FCB NON_PAGED_FCB, *PNON_PAGED_FCB;
typedef struct _DIRENT DIRENT, *PDIRENT;
typedef struct _OEM_STRING OEM_STRING_FAT;
typedef struct _PACKED_BOOT_SECTOR PACKED_BOOT_SECTOR, *PPACKED_BOOT_SECTOR;
typedef struct _PACKED_BIOS_PARAMETER_BLOCK PACKED_BIOS_PARAMETER_BLOCK, *PPACKED_BIOS_PARAMETER_BLOCK;
typedef enum _VCB_CONDITION { VcbGood = 1, VcbNotMounted = 2, VcbBad = 3 } VCB_CONDITION;
typedef enum _FCB_CONDITION { FcbGood = 1, FcbBad = 2 } FCB_CONDITION;

/* FAT_DATA only has Resource as value (ERESOURCE, not pointer).
 * The full struct is opaque here; we access via the exported FatData symbol. */
struct _FAT_DATA {
    /* ... lots of fields ... */
    /* Resource is at offset 68 on amd64 (ERESOURCE = 56 bytes). */
    /* We can't compute the offset without fatstruc.h. Use the macro-expanded
     * accessors FatAcquireExclusiveGlobal_Impl etc. instead.             */
    UCHAR opaque[2048];   /* placeholder; real code must use FatData.Resource */
};

/* We do NOT use any FAT struct fields in this file.
 * ALL resource operations are delegated to FatXxx_Real helpers that live
 * in the Rust code or are the real functions exported from fatprocs.c. */

/* ======================================================================
 * Extern declarations for symbols that ARE real linker exports:
 *  - FatData global (from fatinit.rs / Rust)
 *  - Resource/mutex helpers from Rust (FatAcquireExclusiveVcb_Real etc.)
 * ====================================================================== */

extern FAT_DATA FatData;
extern FAST_MUTEX FatCloseQueueMutex;
extern SLIST_HEADER FatCloseContextSList;
extern NPAGED_LOOKASIDE_LIST FatIrpContextLookasideList;
extern NPAGED_LOOKASIDE_LIST FatNonPagedFcbLookasideList;
extern NPAGED_LOOKASIDE_LIST FatEResourceLookasideList;

/* Real functions from Rust (not macros) */
extern BOOLEAN FatAcquireExclusiveFcb(PIRP_CONTEXT, PFCB);
/* BOOLEAN, not VOID. fatprocs.h:1455 declares it BOOLEAN and resrcsup.rs returns u8
 * ("did I get the resource?"). THIS wrong prototype is the root of the discarded
 * return value below: because the compiler was told _Real returns nothing, the
 * FatAcquireExclusiveVcb shim could not forward an answer even in principle. */
extern BOOLEAN FatAcquireExclusiveVcb_Real(PIRP_CONTEXT, PVCB, BOOLEAN);
extern VOID    FatCompleteRequest_Real(PIRP_CONTEXT, PIRP, NTSTATUS);
extern VOID    FatDeleteIrpContext_Real(PIRP_CONTEXT);

/* IRP_CONTEXT_FLAG_WAIT, taken from the real fatstruc.h in rust_cohab_fat.c
 * like the offsets below. It was a hard-coded 0x1 here, which is
 * IRP_CONTEXT_FLAG_DISABLE_DIRTY: every acquire below ran non-blocking. */
extern const unsigned int FatIrpContextFlagWait;
#define FAT_IRP_CTX_FLAG_WAIT FatIrpContextFlagWait

/* Real struct field byte offsets. These are NOT hand-counted (the old
 * hard-coded values 397/404/478/68 were wrong & unaligned → arm64 mount AV in
 * ExAcquireFastMutexUnsafe). They are computed with FIELD_OFFSET against the
 * REAL fatstruc.h layout over in rust_cohab_fat.c (which includes fatprocs.h)
 * and exported as const globals — arch-correct and self-maintaining. */
extern const unsigned int FatVcbFreeClusterBitMapMutexOffset;
extern const unsigned int FatVcbResourceOffset;
extern const unsigned int FatVcbDirCreationMutexOffset;
extern const unsigned int FatDataResourceOffset;
extern const unsigned int FatFcbHeaderResourceOffset;
extern const unsigned int FatIrpContextFlagsOffset;

/* FCB Header.Resource is a PERESOURCE (pointer) field — read the pointer. */
#define FCB_RESOURCE(fcb)       (*(PERESOURCE *)((PUCHAR)(fcb) + FatFcbHeaderResourceOffset))
#define VCB_RESOURCE(vcb)       ((PERESOURCE)((PUCHAR)(vcb) + FatVcbResourceOffset))

#define VCB_FREECLUSTER_MUTEX(vcb)  ((PFAST_MUTEX)((PUCHAR)(vcb) + FatVcbFreeClusterBitMapMutexOffset))
#define VCB_DIRCREATION_MUTEX(vcb)  ((PFAST_MUTEX)((PUCHAR)(vcb) + FatVcbDirCreationMutexOffset))

#define FATDATA_RESOURCE()  ((PERESOURCE)((PUCHAR)&FatData + FatDataResourceOffset))

#define IRP_CTX_FLAGS(irpctx) (*((PULONG)((PUCHAR)(irpctx) + FatIrpContextFlagsOffset)))

/* ======================================================================
 * 1. FAT macro → real-symbol wrappers
 * ====================================================================== */

VOID FatBugCheck(ULONG_PTR A, ULONG_PTR B, ULONG_PTR C)
{
    /* FAT_FILE_SYSTEM = 0x23, FAT_BUG_CHECK_FATDATA = 0x00010000. Every Rust caller comes
     * through here, so P1 names THIS line, not the caller's file and line (the C macro
     * encodes the caller's): print the return address, the one thing that does. */
    DbgPrint("FatBugCheck called from %p (%p, %p, %p)\n", __builtin_return_address(0),
             (PVOID)A, (PVOID)B, (PVOID)C);
    KeBugCheckEx(0x23, 0x00010000 | (ULONG)(__LINE__), A, B, C);
    for(;;);
}

/* Print which stub was hit (to the debug/serial log) before halting, so an arm64
 * boot reveals exactly which untranslated function blocks the FAT read/mount path. */
#define STUB_BUGCHECK() do { DbgPrint("FASTFAT_STUB_HIT: %s\n", __func__); KeBugCheckEx(0x23, 0xDEAD0000, 0, 0, 0); } while(0)

BOOLEAN FatAcquireExclusiveGlobal(PIRP_CONTEXT IrpContext)
{
    return (BOOLEAN)ExAcquireResourceExclusiveLite(
        FATDATA_RESOURCE(),
        !!(IRP_CTX_FLAGS(IrpContext) & FAT_IRP_CTX_FLAG_WAIT));
}

BOOLEAN FatAcquireSharedGlobal(PIRP_CONTEXT IrpContext)
{
    return (BOOLEAN)ExAcquireResourceSharedLite(
        FATDATA_RESOURCE(),
        !!(IRP_CTX_FLAGS(IrpContext) & FAT_IRP_CTX_FLAG_WAIT));
}

VOID FatReleaseGlobal(PIRP_CONTEXT IrpContext)
{
    (void)IrpContext;
    ExReleaseResourceLite(FATDATA_RESOURCE());
}

/* Returns BOOLEAN, and it MUST: fatprocs.h:1461 defines
 *     #define FatAcquireExclusiveVcb(IC, V)  FatAcquireExclusiveVcb_Real(IC, V, FALSE)
 * and _Real returns BOOLEAN ("did I get the resource?"). Callers branch on it —
 * `if (!FatAcquireExclusiveVcb(...)) { post the request and bail }`.
 *
 * This shim used to be declared VOID while still calling _Real, so the answer was
 * DISCARDED. Every Rust caller that declared a return type was reading whatever
 * happened to survive in the return register across the shim's epilogue: true only
 * by accident of codegen, and silently wrong the moment that register is touched.
 * Live readers at the time of the fix: flush.rs:319/347, fileinfo.rs:2798/3068,
 * volinfo.rs:866 — i.e. flush, set-file-info and volume-info, not obscure paths.
 * The Rust extern decls disagreed six ways over this one symbol (void, BOOLEAN,
 * NTSTATUS, i32); they are normalized to BOOLEAN alongside this. */
BOOLEAN FatAcquireExclusiveVcb(PIRP_CONTEXT IrpContext, PVCB Vcb)
{
    return FatAcquireExclusiveVcb_Real(IrpContext, Vcb, FALSE);
}

/* FatAcquireExclusiveFcb lives in resrcsup.rs (2026-09-25), with the
 * OutstandingAsyncWrites drain this shim never had. */

VOID FatReleaseVcb(PIRP_CONTEXT IrpContext, PVCB Vcb)
{
    (void)IrpContext;
    ExReleaseResourceLite(VCB_RESOURCE(Vcb));
}

VOID FatReleaseFcb(PIRP_CONTEXT IrpContext, PFCB Fcb)
{
    (void)IrpContext;
    ExReleaseResourceLite(FCB_RESOURCE(Fcb));
}

VOID FatAcquireExclusiveVolume(PIRP_CONTEXT IrpContext, PVCB Vcb)
{
    /* Full macro iterates FCBs — simplified: just lock VCB (mount/dismount only) */
    FatAcquireExclusiveVcb_Real(IrpContext, Vcb, FALSE);
}

VOID FatReleaseVolume(PIRP_CONTEXT IrpContext, PVCB Vcb)
{
    (void)IrpContext;
    ExReleaseResourceLite(VCB_RESOURCE(Vcb));
}

VOID FatConvertToSharedFcb(PIRP_CONTEXT IrpContext, PFCB Fcb)
{
    (void)IrpContext;
    ExConvertExclusiveToSharedLite(FCB_RESOURCE(Fcb));
}

BOOLEAN FatVcbAcquiredExclusive(PIRP_CONTEXT IrpContext, PVCB Vcb)
{
    (void)IrpContext;
    return (BOOLEAN)(ExIsResourceAcquiredExclusiveLite(VCB_RESOURCE(Vcb)) ||
                     ExIsResourceAcquiredExclusiveLite(FATDATA_RESOURCE()));
}

VOID FatAcquireCloseMutex(VOID)
{
    ExAcquireFastMutexUnsafe(&FatCloseQueueMutex);
}

VOID FatReleaseCloseMutex(VOID)
{
    ExReleaseFastMutexUnsafe(&FatCloseQueueMutex);
}

VOID FatAcquireDirectoryFileMutex(PVCB Vcb)
{
    ExAcquireFastMutexUnsafe(VCB_DIRCREATION_MUTEX(Vcb));
}

VOID FatReleaseDirectoryFileMutex(PVCB Vcb)
{
    ExReleaseFastMutexUnsafe(VCB_DIRCREATION_MUTEX(Vcb));
}

VOID FatLockFreeClusterBitMap(PVCB Vcb)
{
    ExAcquireFastMutex(VCB_FREECLUSTER_MUTEX(Vcb));
}

VOID FatUnlockFreeClusterBitMap(PVCB Vcb)
{
    ExReleaseFastMutex(VCB_FREECLUSTER_MUTEX(Vcb));
}

VOID FatCompleteRequest(PIRP_CONTEXT IrpContext, PIRP Irp, NTSTATUS Status)
{
    FatCompleteRequest_Real(IrpContext, Irp, Status);
}

VOID FatDeleteIrpContext(PIRP_CONTEXT IrpContext)
{
    FatDeleteIrpContext_Real(IrpContext);
}

VOID FatDeleteResource(PERESOURCE Resource)
{
    ExDeleteResourceLite(Resource);
}

/* FatSetVcbCondition: now implemented in Rust (verfysup.rs) with the typed
 * bindgen struct. The old stub here wrote at a GUESSED offset (+344) and
 * left the real VcbCondition untouched. */

/* FatRaiseStatus / FatNormalizeAndRaiseStatus / FatResetExceptionState live in
 * fastfat-rust/src/seh.rs: the fatprocs.h macros record the status in
 * IrpContext->ExceptionStatus, which FatExceptionFilter and FatProcessException
 * read, and these stubs could not reach the field. */
VOID __cdecl FatDisableParentCheck(PIRP_CONTEXT IrpContext)  { (void)IrpContext; }

/* NOTE: IO/EX/KE/Ob/FsRtl FORCEINLINE wrappers are in rust_cohab_forceinline.c */

/* ======================================================================
 * 2. FAT helper macros → real symbols
 * ====================================================================== */

ULONG_PTR FlagOn(ULONG_PTR x, ULONG_PTR f)        { return x & f; }
VOID      SetFlag(PULONG_PTR x, ULONG_PTR f)       { *x |= f; }
VOID      ClearFlag(PULONG_PTR x, ULONG_PTR f)     { *x &= ~f; }
BOOLEAN   BooleanFlagOn(ULONG_PTR x, ULONG_PTR f)  { return !!(x & f); }
PVOID     Add2Ptr(PVOID p, ULONG_PTR n)             { return (PUCHAR)p + n; }
ULONG_PTR QuadAlign(ULONG_PTR v)                    { return (v + 7) & ~(ULONG_PTR)7; }
ULONG_PTR BlockAlignTruncate(ULONG_PTR v, ULONG_PTR a) { return v & ~(a - 1); }
LONGLONG  Int32x32To64(LONG a, LONG b)              { return (LONGLONG)a * b; }
ULONGLONG UInt32x32To64(ULONG a, ULONG b)           { return (ULONGLONG)a * b; }

VOID CopyUchar4(PULONG dst, PULONG src)  { *dst = *src; }
VOID CopyU4char(PULONG dst, PULONG src)  { *dst = *src; }

/* Rust's 15 declarations of this read a u32. As CSHORT the upper 16 bits of the return
 * register were never written on the 64-bit arches; widened here, once, for all of them. */
ULONG NodeType(PVOID Node)              { return (ULONG)*((PUSHORT)Node); }
VOID PAGED_CODE(VOID)                  { }
VOID paged_code(VOID)                  { }
VOID PagedCode(VOID)                   { }
VOID ASSERT(BOOLEAN e)                 { (void)e; }
VOID NT_ASSERT(BOOLEAN e)              { (void)e; }
VOID UNREFERENCED_PARAMETER(PVOID p)   { (void)p; }
VOID _Analysis_assume_(BOOLEAN b)      { (void)b; }
BOOLEAN ARGUMENT_PRESENT(PVOID p)      { return p != NULL; }

VOID KdPrintEx(ULONG a, ULONG b, PCHAR c, ...) { (void)a;(void)b;(void)c; }

VOID RtlRaiseStatus(NTSTATUS s)    { ExRaiseStatus(s); }

BOOLEAN RtlCheckBit(PRTL_BITMAP h, ULONG pos)
{
    return !!(((PULONG)h->Buffer)[pos >> 5] >> (pos & 31));
}

LUID RtlConvertLongToLuid(LONG v)
{
    LARGE_INTEGER li; LUID l;
    li.QuadPart = v;
    l.LowPart = li.LowPart;
    l.HighPart = li.HighPart;
    return l;
}

BOOLEAN RtlEqualMemory(const VOID *a, const VOID *b, SIZE_T n)
{
    return (RtlCompareMemory(a, b, n) == n);
}

ULONG RtlOemStringToCountedUnicodeSize(POEM_STRING s) { return s->Length * sizeof(WCHAR); }

/* ======================================================================
 * 4. Constants that Rust calls as functions
 * ====================================================================== */

/* FAT_NTC_FCB -> rust_cohab_fat.c DATA */
/* FAT_NTC_DCB -> rust_cohab_fat.c DATA */
/* moved to rust_cohab_fat.c as header-correct DATA */

/* CCB_FLAG_SKIP_SHORT_NAME_COMPARE -> rust_cohab_fat.c DATA */
/* CCB_FLAG_USER_SET_LAST_WRITE -> rust_cohab_fat.c DATA */
/* VCB_STATE_FLAG_DEFERRED_FLUSH -> rust_cohab_fat.c DATA */
/* VCB_STATE_FLAG_REMOVABLE_MEDIA -> rust_cohab_fat.c DATA */
/* VCB_STATE_FLAG_CREATE_IN_PROGRESS -> rust_cohab_fat.c DATA */
/* FCB_STATE_PAGING_FILE -> rust_cohab_fat.c DATA */
/* moved to rust_cohab_fat.c as header-correct DATA */
/* moved to rust_cohab_fat.c as header-correct DATA */
/* FILE_ACTION_ADDED -> rust_cohab_fat.c DATA */
/* FILE_ACTION_REMOVED -> rust_cohab_fat.c DATA */
/* FILE_ACTION_MODIFIED -> rust_cohab_fat.c DATA */
/* FILE_ACTION_RENAMED_OLD_NAME -> rust_cohab_fat.c DATA */
/* FILE_ACTION_RENAMED_NEW_NAME -> rust_cohab_fat.c DATA */
/* IRP_CONTEXT_FLAG_WAIT -> rust_cohab_fat.c DATA */
const ULONG IRP_PAGING_IO = 0x00000002;  /* xdk/iotypes.h (stub had fabricated 0x8) */
/* FAT_DIRENT_ATTR_READ_ONLY -> rust_cohab_fat.c DATA */
/* FAT_DIRENT_ATTR_DIRECTORY -> rust_cohab_fat.c DATA */
/* OPLOCK_FLAG_PARENT_OBJECT -> rust_cohab_fat.c DATA */
/* FO_FILE_MODIFIED -> rust_cohab_fat.c DATA */
/* MAX_LFN_CHARACTERS -> rust_cohab_fat.c DATA */
/* USER_DIRECTORY_OPEN -> rust_cohab_fat.c DATA */
/* UserVolumeOpen -> rust_cohab_fat.c DATA */
/* moved to rust_cohab_fat.c as header-correct DATA */

/* TAG_DIRENT -> rust_cohab_fat.c DATA */
/* moved to rust_cohab_fat.c as header-correct DATA */
/* TAG_FILENAME_BUFFER -> rust_cohab_fat.c DATA */
ULONG FAT_LFN_DIRENTS_NEEDED(PUNICODE_STRING Name)
{
    return (Name->Length / sizeof(WCHAR) + 12) / 13;
}

/* ======================================================================
 * 5. Utility helpers
 * ====================================================================== */

/* SharedSystemTime: fastfat reads this as a 64-bit DATA value (the shared system
 * time snapshot), not a function. Provide a real data symbol so the Rust
 * `extern static mut SharedSystemTime: u64` reference resolves to data, not code.
 * NOTE (owed translation fix): the C macro aliases SharedUserData->SystemTime; a
 * static-zero here yields epoch file timestamps. Tracked for a timesup.rs fix. */
LARGE_INTEGER SharedSystemTime = { 0 };

LONGLONG ExGetCurrentResourceThread(VOID)
{
    return (LONGLONG)(ULONG_PTR)KeGetCurrentThread();
}

BOOLEAN CcIsFileCached(PFILE_OBJECT f)
{
    return (f->SectionObjectPointer &&
            f->SectionObjectPointer->SharedCacheMap) ? TRUE : FALSE;
}

VOID CcSetAdditionalCacheAttributesEx(PFILE_OBJECT f, ULONG fl)
{ (void)f;(void)fl; }

VOID PsUpdateDiskCounters(PEPROCESS a, ULONG64 b, ULONG64 c, ULONG d, ULONG e, ULONG f)
{ (void)a;(void)b;(void)c;(void)d;(void)e;(void)f; }

VOID IoInitializePriorityInfo(PVOID p)
{
    /* IO_PRIORITY_INFO: Size(ULONG)+ThreadPriority(ULONG)+PagePriority(ULONG)+IoPriority(ULONG)=16 */
    if (p) RtlZeroMemory(p, 16);
    if (p) *((PULONG)p) = 16; /* Size = sizeof(IO_PRIORITY_INFO) */
}

NTSTATUS IoRetrievePriorityInfo(PIRP a, PFILE_OBJECT b, PETHREAD c, PVOID d)
{
    (void)a;(void)b;(void)c;
    IoInitializePriorityInfo(d);
    return STATUS_SUCCESS;
}

ULONG SizeOfFullEa(PFILE_FULL_EA_INFORMATION Ea)
{
    return (ULONG)(sizeof(FILE_FULL_EA_INFORMATION) - 1 +
                   Ea->EaNameLength + 1 + Ea->EaValueLength);
}

/* SizeOfPackedEa -> now REAL in fastfat-rust easup.rs (shares ea.rs's inline form).
 * Another MACRO (fat.h:738) that reached here as an unresolved symbol and got
 * "resolved" into a bugcheck — armed on any EA-bearing FAT file. Task #45. */

BOOLEAN GetcbList(PVOID x) { (void)x; return FALSE; }

/* __int2c is in rust_cohab_forceinline.c (clang builtin, can't define here) */

BOOLEAN IsFileObjectReadOnly(PFILE_OBJECT f)
{
    return !f->WriteAccess && !f->DeleteAccess;
}

BOOLEAN IsFileWriteThrough(PFILE_OBJECT f, PDEVICE_OBJECT d)
{
    (void)d;
    return !!(f->Flags & FO_WRITE_THROUGH);
}

/* IsFileDeleted: now implemented in Rust (verfysup.rs) with the typed struct.
 * The old stub here always returned FALSE. */

/* IsBpbFat32: now implemented in Rust (verfysup.rs). The old stub here read
 * a GUESSED offset (+28 = HiddenSectors) instead of RootEntries (+6). */

VOID CanFsdWait(PIRP_CONTEXT IrpContext)
{
    (void)IrpContext; /* no-op; real semantics in strucsup.rs */
}

/* ======================================================================
 * 6. Debug / trace stubs
 * ====================================================================== */

VOID DebugTrace(LONG a, ULONG b, PCHAR c, ...) { (void)a;(void)b;(void)c; }
VOID DebugDump(PCHAR a, ULONG b, PVOID c)               { (void)a;(void)b;(void)c; }
VOID DebugUnwind(PCHAR a)                               { (void)a; }
VOID DebugDoit(PVOID a)                                 { (void)a; }
VOID DbgDoit(PVOID a)                                   { (void)a; }
ULONG Dbg = 0;  /* DebugTrace component id; read as data, no-op when not tracing */
VOID TimerStart(ULONG a)                                { (void)a; }
VOID TimerStop(ULONG a, PCHAR b)                        { (void)a;(void)b; }

/* ======================================================================
 * 7. SEH2 stubs (dead with USE_DUMMY_PSEH)
 * ====================================================================== */

VOID _SEH2_TRY(VOID)                    { }
VOID _SEH2_EXCEPT(PVOID x)              { (void)x; }
VOID _SEH2_FINALLY(VOID)               { }
VOID _SEH2_END(VOID)                   { }
NTSTATUS _SEH2_GetExceptionCode(VOID)  { return STATUS_SUCCESS; }
PVOID _SEH2_GetExceptionInformation(VOID) { return NULL; }
BOOLEAN _SEH2_AbnormalTermination(VOID){ return FALSE; }
/*
 * SEH_TRY_EXCEPT is NOT a dead SEH2 macro — it is the real control-flow wrapper
 * fastfat-rust's FatFsdDirectoryControl uses to run its __try body. The exception
 * path (filter/except) IS dead under USE_DUMMY_PSEH, but the TRY BODY MUST STILL
 * RUN. The previous no-op stub meant FatFsdDirectoryControl returned its init
 * STATUS_SUCCESS without ever creating an IrpContext or calling
 * FatCommonDirectoryControl → every NtQueryDirectoryFile silently returned an
 * empty result → FindFirstFileW failed → the shell's ParseDisplayName of any
 * profile path (e.g. the Start Menu folder) got ERROR_FILE_NOT_FOUND → the arm64
 * start menu never opened. arm64-only in practice because arm64 boots a FAT
 * ramdisk (fastfat); x86 boots the LiveCD (cdfs). The callbacks are Rust
 * `extern "system"` == NTAPI on every arch.
 */
VOID SEH_TRY_EXCEPT(PVOID ctx,
                    VOID (NTAPI *try_fn)(PVOID),
                    PVOID filter_fn,
                    PVOID except_fn)
{
    (void)filter_fn;
    (void)except_fn;
    try_fn(ctx);
}
VOID try_return(PVOID x)               { (void)x; }

/* ======================================================================
 * 8. FsRtl wrappers (FORCEINLINE or new APIs not in older ntoskrnl.lib)
 * ====================================================================== */

BOOLEAN FsRtlAreThereWaitingFileLocks(PFILE_LOCK a)   { (void)a; return FALSE; }
BOOLEAN FsRtlCurrentOplockH(POPLOCK a)                { (void)a; return FALSE; }
BOOLEAN FsRtlOplockIsSharedRequest(PIRP a)            { (void)a; return FALSE; }
BOOLEAN FsRtlCheckLockForOplockRequest(PFILE_LOCK a, PLARGE_INTEGER b) { (void)a;(void)b; return TRUE; }

NTSTATUS FsRtlGetSectorSizeInformation(PDEVICE_OBJECT a, PVOID b)
{
    /* PFILE_FS_SECTOR_SIZE_INFORMATION is Vista+ — stub returns defaults */
    (void)a;
    if (b) {
        /* Zero the struct; the first 4 ULONGs are the common fields.
         * Set LogicalBytesPerSector = 512 (ULONG at offset 0). */
        ((PULONG)b)[0] = 512; /* LogicalBytesPerSector */
        ((PULONG)b)[1] = 512; /* PhysicalBytesPerSectorForAtomicity */
        ((PULONG)b)[2] = 512; /* PhysicalBytesPerSectorForPerformance */
        ((PULONG)b)[3] = 512; /* Effective */
    }
    return STATUS_SUCCESS;
}

NTSTATUS FsRtlOplockBreakH(
    POPLOCK a, PIRP b, ULONG c, PVOID d,
    POPLOCK_WAIT_COMPLETE_ROUTINE e, POPLOCK_FS_PREPOST_IRP f)
{
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    return STATUS_SUCCESS;
}

BOOLEAN FsRtlFastLock(
    PFILE_LOCK a, PFILE_OBJECT b, PLARGE_INTEGER c, PLARGE_INTEGER d,
    PEPROCESS e, ULONG f, BOOLEAN g, BOOLEAN h, PIO_STATUS_BLOCK i,
    PIRP j, BOOLEAN k)
{
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;(void)i;(void)j;(void)k;
    return FALSE;
}

/* ======================================================================
 * 9. Statistics / accounting stubs
 * ====================================================================== */

VOID CollectCreateStatistics(PVCB a, NTSTATUS b)   { (void)a;(void)b; }
VOID CollectCreateHitStatistics(PVCB a)             { (void)a; }
VOID CollectDiskIoStats(PVCB a, PIRP b, ULONG c, ULONG d) { (void)a;(void)b;(void)c;(void)d; }
/* CompareNames -> rust_cohab_fat.c (real macro) */
VOID FatUpdateIOCountersPCW(PVCB a, BOOLEAN b, ULONG_PTR c) { (void)a;(void)b;(void)c; }

/* ======================================================================
 * 10. BugCheck stubs for FAT functions not yet in Rust
 * ====================================================================== */

/* FatUnlockVolume / FatUnlockVolumeInternal / FatDirtyVolume / FatIsVolumeMounted /
 * FatQueryBpb -> now REAL in fastfat-rust fsctrl.rs. Stubs removed so the Rust
 * #[no_mangle] definitions are the sole symbols — same first-match link rule as the
 * FatSetEndOfFileInfo note below: this C object links BEFORE libfastfat_rust.a, so a
 * stub left here would silently SHADOW the real implementation and keep bugchecking.
 * (FatIsPathnameValid's stub lived further down and is removed there.) */
/* FatIsVolumeDirty + FatGetStatistics -> now REAL in fastfat-rust fsctrl.rs; stubs
 * removed so the Rust #[no_mangle] definitions win the first-match link. */
NTSTATUS FatGetVolumeBitmap(PIRP_CONTEXT a, PIRP b)    { (void)a;(void)b; STUB_BUGCHECK(); return STATUS_SUCCESS; }
NTSTATUS FatGetRetrievalPointers(PIRP_CONTEXT a, PIRP b){ (void)a;(void)b; STUB_BUGCHECK(); return STATUS_SUCCESS; }
NTSTATUS FatQueryRetrievalPointers(PIRP_CONTEXT a, PIRP b){ (void)a;(void)b; STUB_BUGCHECK(); return STATUS_SUCCESS; }
/* FatMarkHandle — now REAL in fastfat-rust fsctrl.rs (task #50). Stub removed so
 * the Rust #[no_mangle] definition is the sole symbol: this C object links BEFORE
 * libfastfat_rust.a and lld --allow-multiple-definition is FIRST-match, so leaving
 * the stub here would silently shadow the translation and keep the bugcheck. */
/* FatSetEndOfFileInfo — recovered to Rust (fileinfo.rs, rustconv 2026-07-16). The
 * bugcheck stub is removed so the Rust #[no_mangle] definition is the sole symbol
 * (lld --allow-multiple-definition is first-match, and this C obj links BEFORE
 * libfastfat_rust.a — leaving the stub here would shadow the real impl). */
/* FatOpenVolume -> now REAL in fastfat-rust create.rs (task #47); stub removed so the
 * Rust #[no_mangle] definition is the sole symbol. It is a genuine ~290-line function
 * (create.c:2292), not a macro like the FatGenerateFileId* pair below. */
/* FatOpenExistingDirectory -> now REAL in fastfat-rust create.rs (field manual §9.45); stub removed so the Rust symbol wins */
/* FatLowLevelReadWrite -> rust_cohab_fat.c (real macro) */
/* FatPerformVerifyDiskRead -> now REAL in fastfat-rust fsctrl.rs. Note the stub
 * declared it NTSTATUS with SEVEN PVOID params; the real function returns BOOLEAN
 * and takes six args. Nothing caught that because no C caller remains -- another
 * reminder that a stub prototype is not evidence of the real signature. */
/* FatGenerateFileIdFromFcb / FatGenerateFileIdFromDirentAndOffset -> now REAL in
 * fastfat-rust fileid.rs. These are MACROS in fatprocs.h, never functions; the
 * translation emitted extern fn declarations, which arrived here as unresolved
 * symbols and got "fixed" into a bugcheck. arm64 (FAT32 ramdisk = system volume)
 * stopped 0x23 0xdead0000 in FatGenerateFileIdFromFcb on any
 * FileInternalInformation query. Rust calls its own helpers now — no symbol. */
/* FatReadDirent -> rust_cohab_fat.c (real dirsup.c macro) */
/* fat_check_system_security_access / fat_supersede_or_overwrite_file: DELETED, and
 * they were never reachable. Both functions are already translated in create.rs as
 * FatCheckSystemSecurityAccess and FatSupersedeOrOverwriteFile (#[no_mangle],
 * PascalCase); the snake_case names existed only as unused `extern` decls, which was
 * by itself enough to bring these two bugcheck bodies into the driver. Nothing called
 * them, so unlike the FSCTL stubs below they could never fire — but they are the
 * cleanest possible statement of the rule: a stray extern decl MINTS a bugcheck. */

/* FatConstructDot / FatConstructDotDot / FatNotifyReportChange
 * -> rust_cohab_fat.c (real dirsup.c / fatprocs.h macros) */
__cdecl VOID Fat8QMdot3QM(PVOID a0, PVOID a1) { (void)a0;(void)a1; STUB_BUGCHECK(); }

/* FatFsdEntryCount: a FASTFATDBG-only debug counter. The Rust definition is
 * cfg'd out (feature off) but strucsup references it unconditionally. Provide as
 * real DATA (bare symbol) so the `extern static mut` reference resolves to a
 * zero counter rather than function-prologue bytes. */
ULONG FatFsdEntryCount = 0;
ULONG FatIoCallDriverCount = 0;  /* FASTFATDBG counter; real DATA, incremented by Rust */


/* FatAreNamesEqual / FatIsNameShortOemValid -> rust_cohab_fat.c (real fatprocs.h macros) */
/* FatIsNameLongUnicodeValid -> namesup.rs (Rust impl of the fatprocs.h INLINE) */
/* FatIsPathnameValid -> now REAL in fastfat-rust fsctrl.rs. Note the stub's return
 * type was BOOLEAN while the C function returns NTSTATUS; the Rust extern decl agreed
 * with the STUB, not with fsctrl.c. It survived only because the function always
 * returns STATUS_SUCCESS (0), which truncates to FALSE either way. */
/* FatIsIoRangeValid -> rust_cohab_fat.c (real fatprocs.h static INLINE) */


#pragma clang diagnostic pop
