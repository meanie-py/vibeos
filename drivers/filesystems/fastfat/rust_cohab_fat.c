/* rust_cohab_fat.c — REAL-header cohab providers for fastfat-rust.
 *
 * Unlike rust_cohab_stub.c (which deliberately avoids fatprocs.h and hand-rolls
 * struct offsets — fragile), this TU *includes the real fatprocs.h* and provides
 * out-of-line copies of the kernel intrinsics / FsRtl macros / FAT helper macros
 * that fastfat-rust references via extern "C" but that have no linker symbol
 * (they are FORCEINLINE functions or macros in the DDK/FAT headers).
 *
 * Two tricks, chosen per-symbol by whether the header form is a macro or an
 * inline function (classified empirically with a #ifdef probe):
 *
 *  (a) FORCEINLINE *function*  ->  PRE-include rename. We #define the name to a
 *      private alias BEFORE including the header, so the header's inline body is
 *      emitted under the alias; we then provide the real-named out-of-line symbol
 *      that calls the alias. Correct struct offsets come from the real header.
 *
 *  (b) function-like *macro*   ->  POST-include capture. The macro survives the
 *      include; we wrap it in a private static that expands the macro, #undef the
 *      macro, then provide the real-named out-of-line symbol calling the wrapper.
 *
 * Net: every symbol's logic/layout is the real header's, with zero hand offsets.
 */

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmacro-redefined"
#pragma clang diagnostic ignored "-Wmissing-prototypes"
#pragma clang diagnostic ignored "-Wunused-parameter"

/* ---- (a) pre-include renames: FORCEINLINE functions ----
 * NOTE: symbol class is PER-ARCH. KeGetCurrentIrql is FORCEINLINE only on amd64
 * (reads CR8); on i386/arm64 it is a real ntoskrnl import — provide nothing there.
 */
#if defined(_M_AMD64)
#define KeGetCurrentIrql                __ff_inl_KeGetCurrentIrql
#endif
#define KeGetCurrentProcessorNumber     __ff_inl_KeGetCurrentProcessorNumber
#define IoGetRemainingStackSize         __ff_inl_IoGetRemainingStackSize
#define FsRtlSetupAdvancedHeader        __ff_inl_FsRtlSetupAdvancedHeader
#define FatIsIoRangeValid               __ff_inl_FatIsIoRangeValid

#include "fatprocs.h"

/* ======================================================================
 * (a0) Real struct field offsets, exported for rust_cohab_stub.c.
 *
 * rust_cohab_stub.c deliberately does NOT include fatprocs.h, so it used to
 * hand-count these byte offsets — and got them WRONG and unaligned
 * (FreeClusterBitMapMutex 397/Resource 404/DirCreationMutex 478, plus
 * FatData.Resource 68). On arm64 `Vcb + 478` is unaligned garbage → the FAT
 * mount AV'd inside ExAcquireFastMutexUnsafe's `ldaxr` (FatCheckFreeDirentBitmap
 * → FatAcquireDirectoryFileMutex). This TU has the REAL VCB/FCB/FAT_DATA/
 * IRP_CONTEXT layout, so let the compiler compute the offsets — arch-correct,
 * self-maintaining. (Verified 2026-06-14: arm64/amd64 = 384/440/744/56,
 * i386 = 296/328/488/…)
 * ====================================================================== */
__attribute__((used)) const unsigned int FatVcbFreeClusterBitMapMutexOffset = FIELD_OFFSET(VCB, FreeClusterBitMapMutex);
__attribute__((used)) const unsigned int FatVcbResourceOffset               = FIELD_OFFSET(VCB, Resource);
__attribute__((used)) const unsigned int FatVcbDirCreationMutexOffset        = FIELD_OFFSET(VCB, DirectoryFileCreationMutex);
__attribute__((used)) const unsigned int FatDataResourceOffset               = FIELD_OFFSET(FAT_DATA, Resource);
__attribute__((used)) const unsigned int FatFcbHeaderResourceOffset          = FIELD_OFFSET(FCB, Header.Resource);
__attribute__((used)) const unsigned int FatIrpContextFlagsOffset            = FIELD_OFFSET(IRP_CONTEXT, Flags);
/* The flag VALUE goes the same way. rust_cohab_stub.c hard-coded the wait bit
 * as 0x1, which is IRP_CONTEXT_FLAG_DISABLE_DIRTY; the real WAIT bit is 0x2.
 * Every global/VCB/FCB acquire through that stub therefore ran NON-blocking.
 * FatCommonCleanup ignores the result of its FCB acquire (it relies on
 * blocking), so when another thread held the FCB it went on to release THAT
 * thread's lock: the arm64 0xE3 RESOURCE_NOT_OWNED (#43), found when a page
 * fault's FsRtlReleaseFile hit the FCB resource already idle. */
__attribute__((used)) const unsigned int FatIrpContextFlagWait              = IRP_CONTEXT_FLAG_WAIT;

/* ======================================================================
 * (b) MACRO captures — wrap-then-undef-then-provide
 * ====================================================================== */

/* KeQuerySystemTime / InterlockedPop|PushEntrySList are REAL exports on i386
 * (KeQuerySystemTime@4 stdcall; Interlocked*@N fastcall) — providing them there
 * duplicates the import. amd64/arm64: macros → provide. */
#if !defined(_M_IX86)
static VOID __cap_KeQuerySystemTime(PLARGE_INTEGER t) { KeQuerySystemTime(t); }
#undef KeQuerySystemTime
VOID KeQuerySystemTime(PLARGE_INTEGER CurrentTime) { __cap_KeQuerySystemTime(CurrentTime); }
#endif

static VOID __cap_KeAcquireSpinLock(PKSPIN_LOCK l, PKIRQL i) { KeAcquireSpinLock(l, i); }
#undef KeAcquireSpinLock
VOID KeAcquireSpinLock(PKSPIN_LOCK SpinLock, PKIRQL OldIrql) { __cap_KeAcquireSpinLock(SpinLock, OldIrql); }

static VOID __cap_KeFlushIoBuffers(PMDL m, BOOLEAN r, BOOLEAN d) { KeFlushIoBuffers(m, r, d); }
#undef KeFlushIoBuffers
VOID KeFlushIoBuffers(PMDL Mdl, BOOLEAN ReadOperation, BOOLEAN DmaOperation) { __cap_KeFlushIoBuffers(Mdl, ReadOperation, DmaOperation); }

#if !defined(_M_IX86)
static PSLIST_ENTRY __cap_InterlockedPopEntrySList(PSLIST_HEADER h) { return InterlockedPopEntrySList(h); }
#undef InterlockedPopEntrySList
PSLIST_ENTRY InterlockedPopEntrySList(PSLIST_HEADER ListHead) { return __cap_InterlockedPopEntrySList(ListHead); }

static PSLIST_ENTRY __cap_InterlockedPushEntrySList(PSLIST_HEADER h, PSLIST_ENTRY e) { return InterlockedPushEntrySList(h, e); }
#undef InterlockedPushEntrySList
PSLIST_ENTRY InterlockedPushEntrySList(PSLIST_HEADER ListHead, PSLIST_ENTRY ListEntry) { return __cap_InterlockedPushEntrySList(ListHead, ListEntry); }
#endif

static BOOLEAN __cap_IoIsErrorUserInduced(NTSTATUS s) { return IoIsErrorUserInduced(s); }
#undef IoIsErrorUserInduced
BOOLEAN IoIsErrorUserInduced(NTSTATUS Status) { return __cap_IoIsErrorUserInduced(Status); }

static BOOLEAN __cap_FsRtlIsAnsiCharacterLegalFat(UCHAR c, BOOLEAN w) { return FsRtlIsAnsiCharacterLegalFat(c, w); }
#undef FsRtlIsAnsiCharacterLegalFat
BOOLEAN FsRtlIsAnsiCharacterLegalFat(UCHAR Char, BOOLEAN WildCardsPermissible) { return __cap_FsRtlIsAnsiCharacterLegalFat(Char, WildCardsPermissible); }

static BOOLEAN __cap_FsRtlIsLeadDbcsCharacter(UCHAR c) { return FsRtlIsLeadDbcsCharacter(c); }
#undef FsRtlIsLeadDbcsCharacter
BOOLEAN FsRtlIsLeadDbcsCharacter(UCHAR DbcsChar) { return __cap_FsRtlIsLeadDbcsCharacter(DbcsChar); }

/* ======================================================================
 * (a) FORCEINLINE out-of-line providers — call the pre-renamed inline body
 * ====================================================================== */

#if defined(_M_AMD64)
#undef KeGetCurrentIrql
KIRQL KeGetCurrentIrql(VOID) { return __ff_inl_KeGetCurrentIrql(); }
#endif

#undef KeGetCurrentProcessorNumber
ULONG KeGetCurrentProcessorNumber(VOID) { return __ff_inl_KeGetCurrentProcessorNumber(); }

#undef IoGetRemainingStackSize
ULONG_PTR IoGetRemainingStackSize(VOID) { return __ff_inl_IoGetRemainingStackSize(); }

#undef FsRtlSetupAdvancedHeader
VOID FsRtlSetupAdvancedHeader(PVOID AdvancedHeader, PFAST_MUTEX FastMutex)
{ __ff_inl_FsRtlSetupAdvancedHeader(AdvancedHeader, FastMutex); }

/* FatIsIoRangeValid — static INLINE in fatprocs.h (AllocSup helper). Rust
 * callers (write/create/fileinfo) reference it via extern "C". Out-of-line it
 * with the real header's body: a FAT object's size must fit in 32 bits. */
#undef FatIsIoRangeValid
BOOLEAN FatIsIoRangeValid(PVCB Vcb, LARGE_INTEGER Start, ULONG Length)
{ return __ff_inl_FatIsIoRangeValid(Vcb, Start, Length); }

/* ======================================================================
 * Vista+ (0x600) APIs that fastfat references but are #if'd out at 0x502:
 * provide them directly on top of the 0x502 exports.
 * ====================================================================== */

/* ObReferenceObjectWithTag — tag is a leak-tracking hint; the reference itself
 * is just ObfReferenceObject (the 0x502 fast-ref export). */
LONG_PTR ObReferenceObjectWithTag(PVOID Object, ULONG Tag) { (void)Tag; return ObfReferenceObject(Object); }

/* KeExpandKernelStackAndCalloutEx — runs Callout(Parameter) on a guaranteed-large
 * stack. On our QEMU targets the kernel stack is already ample; call through
 * directly (the documented degenerate behavior when expansion isn't needed). */
NTSTATUS KeExpandKernelStackAndCalloutEx(PEXPAND_STACK_CALLOUT Callout, PVOID Parameter,
                                         SIZE_T Size, BOOLEAN Wait, PVOID Context)
{ (void)Size; (void)Wait; (void)Context; Callout(Parameter); return STATUS_SUCCESS; }

/* FsRtlCheckOplockEx — the Ex flags variant; fold to the 0x502 FsRtlCheckOplock
 * (oplocks are advisory and not on the boot/mount/read path). */
NTSTATUS FsRtlCheckOplockEx(POPLOCK Oplock, PIRP Irp, ULONG Flags,
                            PVOID Context, POPLOCK_WAIT_COMPLETE_ROUTINE CompletionRoutine,
                            POPLOCK_FS_PREPOST_IRP PostIrpRoutine)
{
    (void)Flags;
    return FsRtlCheckOplock(Oplock, Irp, Context, CompletionRoutine, PostIrpRoutine);
}

/* ======================================================================
 * fastfat helpers genuinely absent in this build config (not macros here)
 * ====================================================================== */

/* SafeZeroMemory: fastfat's checked-zero helper — RtlZeroMemory equivalent. */
VOID SafeZeroMemory(PUCHAR At, ULONG ByteCount) { RtlZeroMemory(At, ByteCount); }

/* CollectReadStats: FAT IO statistics accounting — no-op when stats disabled. */
VOID CollectReadStats(PVCB Vcb, ULONG OpenType, ULONG BytesRead) { (void)Vcb; (void)OpenType; (void)BytesRead; }

/* FatLowLevelReadWrite: the read/write primitive — the fatprocs.h macro is simply
 * IoCallDriver(DeviceObject, Irp). (Hand-author wrongly bugcheck-stubbed it.) */
#undef FatLowLevelReadWrite
NTSTATUS FatLowLevelReadWrite(PIRP_CONTEXT IrpContext, PDEVICE_OBJECT DeviceObject, PIRP Irp, PVCB Vcb)
{ (void)IrpContext; (void)Vcb; return IoCallDriver(DeviceObject, Irp); }

/* CompareNames: directory-search name compare — capture the real fatprocs.h macro. */
static COMPARISON __cap_CompareNames(PSTRING a, PSTRING b) { return CompareNames(a, b); }
#undef CompareNames
COMPARISON CompareNames(PSTRING NameA, PSTRING NameB) { return __cap_CompareNames(NameA, NameB); }

/* ======================================================================
 * Read/alloc-path FAT macros (fat.h / fatprocs.h). fastfat-rust references
 * these via extern "C"; provide out-of-line copies using the REAL macros +
 * REAL structs (capture-then-undef-then-define). NOT bugcheck stubs.
 * ====================================================================== */

static ULONG __c_FatBytesPerCluster(PBIOS_PARAMETER_BLOCK B){ return FatBytesPerCluster(B); }
#undef FatBytesPerCluster
ULONG FatBytesPerCluster(PBIOS_PARAMETER_BLOCK B){ return __c_FatBytesPerCluster(B); }

static ULONG __c_FatBytesPerFat(PBIOS_PARAMETER_BLOCK B){ return FatBytesPerFat(B); }
#undef FatBytesPerFat
ULONG FatBytesPerFat(PBIOS_PARAMETER_BLOCK B){ return __c_FatBytesPerFat(B); }

static BOOLEAN __c_FatDeviceIsFatFsdo(PDEVICE_OBJECT D){ return FatDeviceIsFatFsdo(D); }
#undef FatDeviceIsFatFsdo
BOOLEAN FatDeviceIsFatFsdo(PDEVICE_OBJECT D){ return __c_FatDeviceIsFatFsdo(D); }

static ULONGLONG __c_FatDirectoryKey(PFCB F){ return FatDirectoryKey(F); }
#undef FatDirectoryKey
ULONGLONG FatDirectoryKey(PFCB FcbOrDcb){ return __c_FatDirectoryKey(FcbOrDcb); }

static LBO __c_FatFileAreaLbo(PBIOS_PARAMETER_BLOCK B){ return FatFileAreaLbo(B); }
#undef FatFileAreaLbo
LBO FatFileAreaLbo(PBIOS_PARAMETER_BLOCK B){ return __c_FatFileAreaLbo(B); }

static POPLOCK __c_FatGetFcbOplock(PFCB F){ return FatGetFcbOplock(F); }
#undef FatGetFcbOplock
POPLOCK FatGetFcbOplock(PFCB F){ return __c_FatGetFcbOplock(F); }

static PFCB __c_FatGetFirstChild(PDCB D){ return FatGetFirstChild(D); }
#undef FatGetFirstChild
PFCB FatGetFirstChild(PDCB Dir){ return __c_FatGetFirstChild(Dir); }

static ULONG __c_FatGetIndexFromLbo(PVCB V, LBO L){ return FatGetIndexFromLbo(V, L); }
#undef FatGetIndexFromLbo
ULONG FatGetIndexFromLbo(PVCB Vcb, LBO Lbo){ return __c_FatGetIndexFromLbo(Vcb, Lbo); }

static LBO __c_FatGetLboFromIndex(PVCB V, FAT_ENTRY I){ return FatGetLboFromIndex(V, I); }
#undef FatGetLboFromIndex
LBO FatGetLboFromIndex(PVCB Vcb, FAT_ENTRY FatIndex){ return __c_FatGetLboFromIndex(Vcb, FatIndex); }

static PFCB __c_FatGetNextSibling(PFCB F){ return FatGetNextSibling(F); }
#undef FatGetNextSibling
PFCB FatGetNextSibling(PFCB Fcb){ return __c_FatGetNextSibling(Fcb); }

static UCHAR __c_FatIndexBitSize(PBIOS_PARAMETER_BLOCK B){ return FatIndexBitSize(B); }
#undef FatIndexBitSize
UCHAR FatIndexBitSize(PBIOS_PARAMETER_BLOCK B){ return __c_FatIndexBitSize(B); }

static BOOLEAN __c_FatIsFastIoPossible(PFCB F){ return FatIsFastIoPossible(F); }
#undef FatIsFastIoPossible
BOOLEAN FatIsFastIoPossible(PFCB Fcb){ return __c_FatIsFastIoPossible(Fcb); }

static BOOLEAN __c_FatIsFat12(PVCB V){ return FatIsFat12(V); }
#undef FatIsFat12
BOOLEAN FatIsFat12(PVCB Vcb){ return __c_FatIsFat12(Vcb); }

static BOOLEAN __c_FatIsFat32(PVCB V){ return FatIsFat32(V); }
#undef FatIsFat32
BOOLEAN FatIsFat32(PVCB Vcb){ return __c_FatIsFat32(Vcb); }

static BOOLEAN __c_FatIsFileOplockable(PFCB F){ return FatIsFileOplockable(F); }
#undef FatIsFileOplockable
BOOLEAN FatIsFileOplockable(PFCB Fcb){ return __c_FatIsFileOplockable(Fcb); }

static VOID __c_FatLookup12BitEntry(PVOID FAT, FAT_ENTRY I, FAT_ENTRY *E){ FatLookup12BitEntry(FAT, I, E); }
#undef FatLookup12BitEntry
VOID FatLookup12BitEntry(PVOID Fat, FAT_ENTRY Index, FAT_ENTRY *Entry){ __c_FatLookup12BitEntry(Fat, Index, Entry); }

static ULONG __c_FatNumberOfClusters(PBIOS_PARAMETER_BLOCK B){ return FatNumberOfClusters(B); }
#undef FatNumberOfClusters
ULONG FatNumberOfClusters(PBIOS_PARAMETER_BLOCK B){ return __c_FatNumberOfClusters(B); }

static ULONG __c_FatReservedBytes(PBIOS_PARAMETER_BLOCK B){ return FatReservedBytes(B); }
#undef FatReservedBytes
ULONG FatReservedBytes(PBIOS_PARAMETER_BLOCK B){ return __c_FatReservedBytes(B); }

static LBO __c_FatRootDirectoryLbo(PBIOS_PARAMETER_BLOCK B){ return FatRootDirectoryLbo(B); }
#undef FatRootDirectoryLbo
LBO FatRootDirectoryLbo(PBIOS_PARAMETER_BLOCK B){ return __c_FatRootDirectoryLbo(B); }

static ULONG __c_FatRootDirectorySize(PBIOS_PARAMETER_BLOCK B){ return FatRootDirectorySize(B); }
#undef FatRootDirectorySize
ULONG FatRootDirectorySize(PBIOS_PARAMETER_BLOCK B){ return __c_FatRootDirectorySize(B); }

static VOID __c_FatSet12BitEntry(PVOID FAT, FAT_ENTRY I, FAT_ENTRY E){ FatSet12BitEntry(FAT, I, E); }
#undef FatSet12BitEntry
VOID FatSet12BitEntry(PVOID Fat, FAT_ENTRY Index, FAT_ENTRY Entry){ __c_FatSet12BitEntry(Fat, Index, Entry); }

static VOID __c_FatUnpackBios(PBIOS_PARAMETER_BLOCK Bi, PPACKED_BIOS_PARAMETER_BLOCK P){ FatUnpackBios(Bi, P); }
#undef FatUnpackBios
VOID FatUnpackBios(PBIOS_PARAMETER_BLOCK Bios, PPACKED_BIOS_PARAMETER_BLOCK Pbios){ __c_FatUnpackBios(Bios, Pbios); }

static VOID __c_FatUnpinBcb(PIRP_CONTEXT IC, PBCB *B){ FatUnpinBcb(IC, *B); }
#undef FatUnpinBcb
VOID FatUnpinBcb(PIRP_CONTEXT IrpContext, PBCB *Bcb){ __c_FatUnpinBcb(IrpContext, Bcb); }

static VOID __c_FatVerifyIndexIsValid(PIRP_CONTEXT IC, PVCB V, ULONG I){ FatVerifyIndexIsValid(IC, V, I); }
#undef FatVerifyIndexIsValid
VOID FatVerifyIndexIsValid(PIRP_CONTEXT IrpContext, PVCB Vcb, ULONG Index){ __c_FatVerifyIndexIsValid(IrpContext, Vcb, Index); }

static VOID __c_FatUpcaseEaName(PIRP_CONTEXT IC, POEM_STRING N, POEM_STRING U){ FatUpcaseEaName(IC, N, U); }
#undef FatUpcaseEaName
VOID FatUpcaseEaName(PIRP_CONTEXT IrpContext, POEM_STRING Name, POEM_STRING UpcasedName){ __c_FatUpcaseEaName(IrpContext, Name, UpcasedName); }

/* ======================================================================
 * Directory-path macro wrappers — these fatprocs.h/dirsup.c macros were
 * lazily STUB_BUGCHECK'd by the batch translation (rust_cohab_stub.c) and
 * fired during the arm64 FAT mount/dirent path. They are genuine C macros,
 * so reuse the real implementation here (wrap-then-undef-then-provide), the
 * same pattern as FatUnpinBcb above. Stubs removed from rust_cohab_stub.c.
 * 2026-06-14.
 * ====================================================================== */
/* FatReadDirent / FatConstructDot / FatConstructDotDot are LOCAL macros in
 * dirsup.c (not a header), so they aren't visible here — inline their bodies
 * directly. FatUnpinBcb below is the cohab FUNCTION (PBCB*) from above. */
VOID FatReadDirent(PIRP_CONTEXT IrpContext, PDCB Dcb, ULONG Vbo, PBCB *Bcb, PDIRENT *Dirent, PNTSTATUS Status){
    if (Vbo >= Dcb->Header.AllocationSize.LowPart) {
        *Status = STATUS_END_OF_FILE;
        FatUnpinBcb(IrpContext, Bcb);
    } else if ((Vbo % PAGE_SIZE == 0) || (*Bcb == NULL)) {
        FatUnpinBcb(IrpContext, Bcb);
        FatReadDirectoryFile(IrpContext, Dcb, Vbo & ~(PAGE_SIZE - 1), PAGE_SIZE,
                             FALSE, Bcb, (PVOID *)Dirent, Status);
        *Dirent = (PDIRENT)((PUCHAR)*Dirent + (Vbo % PAGE_SIZE));
    }
}

VOID FatConstructDot(PIRP_CONTEXT IrpContext, PDCB Dcb, PDIRENT ParentDirent, PDIRENT Dirent){
    UNREFERENCED_PARAMETER(IrpContext);
    RtlCopyMemory((PUCHAR)Dirent, ".          ", 11);
    Dirent->Attributes = FAT_DIRENT_ATTR_DIRECTORY;
    Dirent->LastWriteTime = ParentDirent->LastWriteTime;
    if (FatData.ChicagoMode) {
        Dirent->CreationTime = ParentDirent->CreationTime;
        Dirent->CreationMSec = ParentDirent->CreationMSec;
        Dirent->LastAccessDate = ParentDirent->LastAccessDate;
    }
    Dirent->FirstClusterOfFile = (USHORT)Dcb->FirstClusterOfFile;
    Dirent->FirstClusterOfFileHi = (USHORT)(Dcb->FirstClusterOfFile / 0x10000);
}

VOID FatConstructDotDot(PIRP_CONTEXT IrpContext, PDCB Dcb, PDIRENT ParentDirent, PDIRENT Dirent){
    UNREFERENCED_PARAMETER(IrpContext);
    RtlCopyMemory((PUCHAR)Dirent, "..         ", 11);
    Dirent->Attributes = FAT_DIRENT_ATTR_DIRECTORY;
    Dirent->LastWriteTime = ParentDirent->LastWriteTime;
    if (FatData.ChicagoMode) {
        Dirent->CreationTime = ParentDirent->CreationTime;
        Dirent->CreationMSec = ParentDirent->CreationMSec;
        Dirent->LastAccessDate = ParentDirent->LastAccessDate;
    }
    if (NodeType(Dcb->ParentDcb) == FAT_NTC_ROOT_DCB) {
        Dirent->FirstClusterOfFile = 0;
        Dirent->FirstClusterOfFileHi = 0;
    } else {
        Dirent->FirstClusterOfFile = (USHORT)(Dcb->ParentDcb->FirstClusterOfFile);
        Dirent->FirstClusterOfFileHi = (USHORT)(Dcb->ParentDcb->FirstClusterOfFile / 0x10000);
    }
}

static VOID __c_FatNotifyReportChange(PIRP_CONTEXT IC, PVCB V, PFCB F, ULONG FL, ULONG A){ FatNotifyReportChange(IC, V, F, FL, A); }
#undef FatNotifyReportChange
VOID FatNotifyReportChange(PIRP_CONTEXT IrpContext, PVCB Vcb, PFCB Fcb, ULONG Filter, ULONG Action){ __c_FatNotifyReportChange(IrpContext, Vcb, Fcb, Filter, Action); }

static BOOLEAN __c_FatAreNamesEqual(PIRP_CONTEXT IC, OEM_STRING A, OEM_STRING B){ return FatAreNamesEqual(IC, A, B); }
#undef FatAreNamesEqual
BOOLEAN FatAreNamesEqual(PIRP_CONTEXT IrpContext, OEM_STRING Name1, OEM_STRING Name2){ return __c_FatAreNamesEqual(IrpContext, Name1, Name2); }

static BOOLEAN __c_FatIsNameShortOemValid(PIRP_CONTEXT IC, OEM_STRING N, BOOLEAN W, BOOLEAN P, BOOLEAN L){ return FatIsNameShortOemValid(IC, N, W, P, L); }
#undef FatIsNameShortOemValid
BOOLEAN FatIsNameShortOemValid(PIRP_CONTEXT IrpContext, OEM_STRING OemName, BOOLEAN CanContainWildCards, BOOLEAN PathNameOk, BOOLEAN LeadingBackslashOk){ return __c_FatIsNameShortOemValid(IrpContext, OemName, CanContainWildCards, PathNameOk, LeadingBackslashOk); }

/* FatTestRaisedStatus: fault-injection debug global referenced by the FatRaiseStatus
 * debug path (pulled in by FatVerifyIndexIsValid). Defaults to 0 (disabled). */
BOOLEAN FatTestRaisedStatus = 0;

/* ---- header-defined macro constants the Rust translation references as DATA ----
 * These C macros (DCB_UNION_SLACK_SPACE / FCB_LOOKUP_ALLOCATIONSIZE_HINT / etc.)
 * were translated as `extern static` data refs. Provide them as real DATA symbols
 * (bare, -mrtd-immune) with HEADER-CORRECT values computed here where fatprocs.h /
 * nodetype.h / ntstatus.h are in scope. (The earlier hand-stub fabricated several
 * of these — e.g. FCB_LOOKUP_ALLOCATIONSIZE_HINT was 0x40000000, really -1.) */
#if defined(_M_IX86)
#define FF_SYM(s) "_" s          /* i386 mingw prepends a leading underscore */
#else
#define FF_SYM(s) s              /* amd64 / arm64: undecorated */
#endif
const ULONG    __c_DCB_UNION_SLACK_SPACE __asm__(FF_SYM("DCB_UNION_SLACK_SPACE"))          = DCB_UNION_SLACK_SPACE;
const LONGLONG __c_FCB_LOOKUP_AS_HINT    __asm__(FF_SYM("FCB_LOOKUP_ALLOCATIONSIZE_HINT")) = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
const ULONG    __c_TAG_DIRENT_BITMAP     __asm__(FF_SYM("TAG_DIRENT_BITMAP"))              = TAG_DIRENT_BITMAP;
const ULONG    __c_FAT_NTC_ROOT_DCB      __asm__(FF_SYM("FAT_NTC_ROOT_DCB"))               = (ULONG)FAT_NTC_ROOT_DCB;
const LONG     __c_STATUS_FILE_CORRUPT   __asm__(FF_SYM("STATUS_FILE_CORRUPT_ERROR"))      = (LONG)STATUS_FILE_CORRUPT_ERROR;

/* ---- const-class symbols the Rust translation references as DATA (header-correct) ----
 * These are #define / enum constants in fatprocs.h / nodetype.h / ntifs.h. Provide them
 * as real DATA symbols with header values (replaces fabricated stub accessors). */
const ULONG __c_CCB_FLAG_SKIP_SHORT_NAME_COMPARE __asm__(FF_SYM("CCB_FLAG_SKIP_SHORT_NAME_COMPARE")) = (ULONG)(CCB_FLAG_SKIP_SHORT_NAME_COMPARE);
const ULONG __c_CCB_FLAG_USER_SET_LAST_WRITE __asm__(FF_SYM("CCB_FLAG_USER_SET_LAST_WRITE")) = (ULONG)(CCB_FLAG_USER_SET_LAST_WRITE);
const ULONG __c_VCB_STATE_FLAG_DEFERRED_FLUSH __asm__(FF_SYM("VCB_STATE_FLAG_DEFERRED_FLUSH")) = (ULONG)(VCB_STATE_FLAG_DEFERRED_FLUSH);
const ULONG __c_VCB_STATE_FLAG_REMOVABLE_MEDIA __asm__(FF_SYM("VCB_STATE_FLAG_REMOVABLE_MEDIA")) = (ULONG)(VCB_STATE_FLAG_REMOVABLE_MEDIA);
const ULONG __c_VCB_STATE_FLAG_CREATE_IN_PROGRESS __asm__(FF_SYM("VCB_STATE_FLAG_CREATE_IN_PROGRESS")) = (ULONG)(VCB_STATE_FLAG_CREATE_IN_PROGRESS);
const ULONG __c_FCB_STATE_PAGING_FILE __asm__(FF_SYM("FCB_STATE_PAGING_FILE")) = (ULONG)(FCB_STATE_PAGING_FILE);
const ULONG __c_FILE_ACTION_ADDED __asm__(FF_SYM("FILE_ACTION_ADDED")) = (ULONG)(FILE_ACTION_ADDED);
const ULONG __c_FILE_ACTION_REMOVED __asm__(FF_SYM("FILE_ACTION_REMOVED")) = (ULONG)(FILE_ACTION_REMOVED);
const ULONG __c_FILE_ACTION_MODIFIED __asm__(FF_SYM("FILE_ACTION_MODIFIED")) = (ULONG)(FILE_ACTION_MODIFIED);
const ULONG __c_FILE_ACTION_RENAMED_OLD_NAME __asm__(FF_SYM("FILE_ACTION_RENAMED_OLD_NAME")) = (ULONG)(FILE_ACTION_RENAMED_OLD_NAME);
const ULONG __c_FILE_ACTION_RENAMED_NEW_NAME __asm__(FF_SYM("FILE_ACTION_RENAMED_NEW_NAME")) = (ULONG)(FILE_ACTION_RENAMED_NEW_NAME);
const ULONG __c_IRP_CONTEXT_FLAG_WAIT __asm__(FF_SYM("IRP_CONTEXT_FLAG_WAIT")) = (ULONG)(IRP_CONTEXT_FLAG_WAIT);
const ULONG __c_FAT_DIRENT_ATTR_READ_ONLY __asm__(FF_SYM("FAT_DIRENT_ATTR_READ_ONLY")) = (ULONG)(FAT_DIRENT_ATTR_READ_ONLY);
const ULONG __c_FAT_DIRENT_ATTR_DIRECTORY __asm__(FF_SYM("FAT_DIRENT_ATTR_DIRECTORY")) = (ULONG)(FAT_DIRENT_ATTR_DIRECTORY);
const ULONG __c_OPLOCK_FLAG_PARENT_OBJECT __asm__(FF_SYM("OPLOCK_FLAG_PARENT_OBJECT")) = 0x00000002UL; /* not in this header set; Windows-documented value */
const ULONG __c_FO_FILE_MODIFIED __asm__(FF_SYM("FO_FILE_MODIFIED")) = (ULONG)(FO_FILE_MODIFIED);
const ULONG __c_MAX_LFN_CHARACTERS __asm__(FF_SYM("MAX_LFN_CHARACTERS")) = (ULONG)(MAX_LFN_CHARACTERS);
const ULONG __c_TAG_DIRENT __asm__(FF_SYM("TAG_DIRENT")) = (ULONG)(TAG_DIRENT);
const ULONG __c_TAG_FILENAME_BUFFER __asm__(FF_SYM("TAG_FILENAME_BUFFER")) = (ULONG)(TAG_FILENAME_BUFFER);
const ULONG __c_FAT_NTC_FCB __asm__(FF_SYM("FAT_NTC_FCB")) = (ULONG)(FAT_NTC_FCB);
const ULONG __c_FAT_NTC_DCB __asm__(FF_SYM("FAT_NTC_DCB")) = (ULONG)(FAT_NTC_DCB);
const LONG  __c_USER_DIRECTORY_OPEN __asm__(FF_SYM("USER_DIRECTORY_OPEN")) = (LONG)(UserDirectoryOpen);
const LONG  __c_UserVolumeOpen __asm__(FF_SYM("UserVolumeOpen")) = (LONG)(UserVolumeOpen);


/* KeAcquireSpinLockRaiseToDpc: not a real i386 export (macro/forceinline). Provide
 * it directly — acquire raising to DISPATCH_LEVEL, returning the old IRQL. Under
 * -mrtd this emits the stdcall _KeAcquireSpinLockRaiseToDpc@4 the Rust crate wants.
 *
 * Do NOT implement this via KeAcquireSpinLock: the DDK macro KeAcquireSpinLock(l,i)
 * expands to `*(i) = KeAcquireSpinLockRaiseToDpc(l)`, so calling it here recursed
 * straight back into this function. The compiler folded the infinite tail-recursion
 * to `b .` (branch-to-self), so the FIRST real fastfat spinlock acquire
 * (FatPrepareWriteDirectoryFile during FatCreateNewFile, reached once smss starts
 * creating files) spun forever at 100% CPU. On this UP build acquiring a spinlock is
 * just a raise to DISPATCH_LEVEL — exactly what ntoskrnl's own arm64
 * KeAcquireSpinLockRaiseToDpc (ke/arm64/stubs.c) does. */
#undef KeAcquireSpinLockRaiseToDpc
KIRQL KeAcquireSpinLockRaiseToDpc(PKSPIN_LOCK SpinLock)
{
    (void)SpinLock;
    return KfRaiseIrql(DISPATCH_LEVEL);
}

/* RtlCopyMemory is a #define for memcpy; the Rust translation referenced it as a
 * stdcall import (@12 on i386). Provide it (-mrtd decorates on i386; bare elsewhere). */
#undef RtlCopyMemory
VOID RtlCopyMemory(PVOID Dest, const void *Src, SIZE_T Length) { __builtin_memcpy(Dest, Src, Length); }

#if defined(_M_IX86)
/* i386: KeGetCurrentProcessorNumber is referenced bare via the ddk substrate but
 * we provide it @0 (-mrtd). Add a bare alias to the same impl. */
__asm__(".globl _KeGetCurrentProcessorNumber\n .set _KeGetCurrentProcessorNumber, _KeGetCurrentProcessorNumber@0");
__asm__(".globl _IoGetRemainingStackSize\n      .set _IoGetRemainingStackSize, _IoGetRemainingStackSize@0");

/* ExReleaseResourceLite / Ex{Acquire,Release}FastMutexUnsafe used to need stdcall
 * trampolines here, aliased in asm, because the Rust declared these FASTCALL exports
 * `extern "system"` and so asked the linker for _Name@4 instead of @Name@4. That was a
 * workaround for a one-word source bug, and it hid the bug from every gate: `cargo check`
 * never links, so nothing but the final PE link could see it. The declarations are fixed
 * at the source now (fastfat-rust close.rs/allocsup.rs, and ntkrnl_sys::ddk for
 * ExReleaseResourceLite), and tools/scan_import_cc.py checks every cross-module extern
 * against the owning .spec so a new one is a lint failure rather than another trampoline. */
#endif

#pragma clang diagnostic pop
