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

/* ---- (a) pre-include renames: FORCEINLINE functions ---- */
#define KeGetCurrentIrql                __ff_inl_KeGetCurrentIrql
#define KeGetCurrentProcessorNumber     __ff_inl_KeGetCurrentProcessorNumber
#define IoGetRemainingStackSize         __ff_inl_IoGetRemainingStackSize
#define FsRtlSetupAdvancedHeader        __ff_inl_FsRtlSetupAdvancedHeader

#include "fatprocs.h"

/* ======================================================================
 * (b) MACRO captures — wrap-then-undef-then-provide
 * ====================================================================== */

static VOID __cap_KeQuerySystemTime(PLARGE_INTEGER t) { KeQuerySystemTime(t); }
#undef KeQuerySystemTime
VOID KeQuerySystemTime(PLARGE_INTEGER CurrentTime) { __cap_KeQuerySystemTime(CurrentTime); }

static VOID __cap_KeAcquireSpinLock(PKSPIN_LOCK l, PKIRQL i) { KeAcquireSpinLock(l, i); }
#undef KeAcquireSpinLock
VOID KeAcquireSpinLock(PKSPIN_LOCK SpinLock, PKIRQL OldIrql) { __cap_KeAcquireSpinLock(SpinLock, OldIrql); }

static VOID __cap_KeFlushIoBuffers(PMDL m, BOOLEAN r, BOOLEAN d) { KeFlushIoBuffers(m, r, d); }
#undef KeFlushIoBuffers
VOID KeFlushIoBuffers(PMDL Mdl, BOOLEAN ReadOperation, BOOLEAN DmaOperation) { __cap_KeFlushIoBuffers(Mdl, ReadOperation, DmaOperation); }

static PSLIST_ENTRY __cap_InterlockedPopEntrySList(PSLIST_HEADER h) { return InterlockedPopEntrySList(h); }
#undef InterlockedPopEntrySList
PSLIST_ENTRY InterlockedPopEntrySList(PSLIST_HEADER ListHead) { return __cap_InterlockedPopEntrySList(ListHead); }

static PSLIST_ENTRY __cap_InterlockedPushEntrySList(PSLIST_HEADER h, PSLIST_ENTRY e) { return InterlockedPushEntrySList(h, e); }
#undef InterlockedPushEntrySList
PSLIST_ENTRY InterlockedPushEntrySList(PSLIST_HEADER ListHead, PSLIST_ENTRY ListEntry) { return __cap_InterlockedPushEntrySList(ListHead, ListEntry); }

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

#undef KeGetCurrentIrql
KIRQL KeGetCurrentIrql(VOID) { return __ff_inl_KeGetCurrentIrql(); }

#undef KeGetCurrentProcessorNumber
ULONG KeGetCurrentProcessorNumber(VOID) { return __ff_inl_KeGetCurrentProcessorNumber(); }

#undef IoGetRemainingStackSize
ULONG_PTR IoGetRemainingStackSize(VOID) { return __ff_inl_IoGetRemainingStackSize(); }

#undef FsRtlSetupAdvancedHeader
VOID FsRtlSetupAdvancedHeader(PVOID AdvancedHeader, PFAST_MUTEX FastMutex)
{ __ff_inl_FsRtlSetupAdvancedHeader(AdvancedHeader, FastMutex); }

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

/* FatTestRaisedStatus: fault-injection debug global referenced by the FatRaiseStatus
 * debug path (pulled in by FatVerifyIndexIsValid). Defaults to 0 (disabled). */
BOOLEAN FatTestRaisedStatus = 0;

#pragma clang diagnostic pop
