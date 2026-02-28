/*
 * kd64_rust_shims.c - C shims for KD64/CC/FSRTL Rust subsystems
 *
 * Provides linker symbols for FORCEINLINE/macro functions that Rust
 * code calls via extern declarations. Also provides KdpDprintf (variadic).
 *
 * This file MUST be in PCH_SKIP_SOURCE for both ntoskrnl and ntkrnlmp
 * so that Phase 1 renames take effect before header inclusion.
 */

/* === Phase 1: Rename FORCEINLINE functions before headers === */

/* Arch-specific FORCEINLINE functions (amd64/i386 ke.h) */
#if !defined(_M_ARM64) && !defined(_M_ARM)
#define KeSetContextPc              _fi_KeSetContextPc
#define KeSetContextReturnRegister  _fi_KeSetContextReturnRegister
#define KeGetContextPc              _fi_KeGetContextPc
#define KeDisableInterrupts         _fi_KeDisableInterrupts
#define KeRestoreInterrupts         _fi_KeRestoreInterrupts
#define KeSweepICache               _fi_KeSweepICache
#endif

/* All-arch FORCEINLINE functions */
#define KeGetPcr                             _fi_KeGetPcr
#define KeGetCurrentProcessorNumber          _fi_KeGetCurrentProcessorNumber
#define MiIsMemoryTypeInvisible              _fi_MiIsMemoryTypeInvisible
#define ExAllocateFromNPagedLookasideList    _fi_ExAllocateFromNPagedLookasideList
#define ExFreeToNPagedLookasideList          _fi_ExFreeToNPagedLookasideList
#define ExAllocateFromPagedLookasideList     _fi_ExAllocateFromPagedLookasideList

/* windbgkd.h static __inline */
#define ExceptionRecord32To64       _fi_ExceptionRecord32To64

/* intsafe.h __forceinline (via INTSAFE_NAME macro) */
#define RtlLongLongAdd              _fi_RtlLongLongAdd

/* === Phase 2: Include headers === */
#define NDEBUG
#include <ntoskrnl.h>
#include <debug.h>
#include <mm/ARM3/miarm.h>

/* === Phase 3: Trampolines for MACROS (not FORCEINLINE) === */

static __inline BOOLEAN _tramp_KeGetTrapFrameInterruptState(PKTRAP_FRAME tf)
{ return KeGetTrapFrameInterruptState(tf); }
#undef KeGetTrapFrameInterruptState

static __inline void _tramp_ExInitializeWorkItem(PWORK_QUEUE_ITEM Item,
    PWORKER_THREAD_ROUTINE Routine, PVOID Context)
{ ExInitializeWorkItem(Item, Routine, Context); }
#undef ExInitializeWorkItem

static __inline void _tramp_FsRtlExitFileSystem(void)
{ FsRtlExitFileSystem; }
#undef FsRtlExitFileSystem

static __inline void _tramp_FsRtlEnterFileSystem(void)
{ FsRtlEnterFileSystem; }
#undef FsRtlEnterFileSystem

static __inline PPFN_NUMBER _tramp_MmGetMdlPfnArray(PMDL Mdl)
{ return MmGetMdlPfnArray(Mdl); }
#undef MmGetMdlPfnArray

static __inline PVOID _tramp_MmGetSystemAddressForMdl(PMDL Mdl)
{ return MmGetSystemAddressForMdl(Mdl); }
#undef MmGetSystemAddressForMdl

static __inline void _tramp_RtlInitializeSplayLinks(PRTL_SPLAY_LINKS Links)
{ RtlInitializeSplayLinks(Links); }
#undef RtlInitializeSplayLinks

static __inline void _tramp_RtlInsertAsLeftChild(PRTL_SPLAY_LINKS P, PRTL_SPLAY_LINKS C)
{ RtlInsertAsLeftChild(P, C); }
#undef RtlInsertAsLeftChild

static __inline void _tramp_RtlInsertAsRightChild(PRTL_SPLAY_LINKS P, PRTL_SPLAY_LINKS C)
{ RtlInsertAsRightChild(P, C); }
#undef RtlInsertAsRightChild

static __inline BOOLEAN _tramp_RtlIsLeftChild(PRTL_SPLAY_LINKS Links)
{ return RtlIsLeftChild(Links); }
#undef RtlIsLeftChild

static __inline PRTL_SPLAY_LINKS _tramp_RtlParent(PRTL_SPLAY_LINKS Links)
{ return RtlParent(Links); }
#undef RtlParent

static __inline void _tramp_CcBugCheck(ULONG A, ULONG B, ULONG C)
{ KeBugCheckEx(CACHE_MANAGER, 0, A, B, C); }

/* ARM64/ARM: KeSetContextPc etc. are MACROS, need trampoline */
#if defined(_M_ARM64) || defined(_M_ARM)
static __inline void _tramp_KeSetContextPc(PCONTEXT ctx, ULONG_PTR pc)
{ KeSetContextPc(ctx, pc); }
static __inline void _tramp_KeSetContextReturnRegister(PCONTEXT ctx, ULONG_PTR val)
{ KeSetContextReturnRegister(ctx, val); }
static __inline BOOLEAN _tramp_KeDisableInterrupts(void)
{ return KeDisableInterrupts(); }
static __inline void _tramp_KeRestoreInterrupts(BOOLEAN e)
{ KeRestoreInterrupts(e); }
static __inline void _tramp_KeSweepICache(PVOID Base, SIZE_T Size)
{ KeSweepICache(Base, Size); }
#undef KeSetContextPc
#undef KeSetContextReturnRegister
#undef KeDisableInterrupts
#undef KeRestoreInterrupts
#undef KeSweepICache
#endif

/* === Phase 4: Undef Phase 1 renames === */
#if !defined(_M_ARM64) && !defined(_M_ARM)
#undef KeSetContextPc
#undef KeSetContextReturnRegister
#undef KeGetContextPc
#undef KeDisableInterrupts
#undef KeRestoreInterrupts
#undef KeSweepICache
#endif

#undef KeGetPcr
#undef KeGetCurrentProcessorNumber
#undef MiIsMemoryTypeInvisible
#undef ExAllocateFromNPagedLookasideList
#undef ExFreeToNPagedLookasideList
#undef ExAllocateFromPagedLookasideList
#undef ExceptionRecord32To64
#undef RtlLongLongAdd
#undef CcBugCheck

/* === Phase 5: Real function implementations === */

/* KdpDprintf - variadic debug printf. Cannot be in Rust (va_list).
 * Only needed when KD64 is Rust (C kdprint.c provides its own). */
#ifndef KD64_USE_C
VOID __cdecl
KdpDprintf(_In_ PCSTR Format, ...)
{
    STRING String;
    USHORT Length;
    va_list ap;
    CHAR Buffer[512];
    va_start(ap, Format);
    Length = (USHORT)_vsnprintf(Buffer, sizeof(Buffer), Format, ap);
    va_end(ap);
    String.Buffer = Buffer;
    String.Length = String.MaximumLength = Length;
    KdpPrintString(&String);
}
#endif

VOID NTAPI KeSetContextPc(PCONTEXT Context, ULONG_PTR ProgramCounter)
{
#if defined(_M_ARM64) || defined(_M_ARM)
    _tramp_KeSetContextPc(Context, ProgramCounter);
#else
    _fi_KeSetContextPc(Context, ProgramCounter);
#endif
}

VOID NTAPI KeSetContextReturnRegister(PCONTEXT Context, ULONG_PTR ReturnValue)
{
#if defined(_M_ARM64) || defined(_M_ARM)
    _tramp_KeSetContextReturnRegister(Context, ReturnValue);
#else
    _fi_KeSetContextReturnRegister(Context, ReturnValue);
#endif
}

#if defined(_M_IX86)
ULONG_PTR NTAPI KeGetContextPc(PCONTEXT Context)
{ return _fi_KeGetContextPc(Context); }
#endif

#ifdef _M_AMD64
PKPCR NTAPI KeGetPcr(VOID)
{ return _fi_KeGetPcr(); }
#endif

ULONG KeGetCurrentProcessorNumber(VOID)
{ return _fi_KeGetCurrentProcessorNumber(); }

BOOLEAN NTAPI KeDisableInterrupts(VOID)
{
#if defined(_M_ARM64) || defined(_M_ARM)
    return _tramp_KeDisableInterrupts();
#else
    return _fi_KeDisableInterrupts();
#endif
}

VOID NTAPI KeRestoreInterrupts(BOOLEAN WereEnabled)
{
#if defined(_M_ARM64) || defined(_M_ARM)
    _tramp_KeRestoreInterrupts(WereEnabled);
#else
    _fi_KeRestoreInterrupts(WereEnabled);
#endif
}

VOID KeSweepICache(IN PVOID BaseAddress, IN SIZE_T FlushSize)
{
#if defined(_M_ARM64) || defined(_M_ARM)
    _tramp_KeSweepICache(BaseAddress, FlushSize);
#else
    _fi_KeSweepICache(BaseAddress, FlushSize);
#endif
}

BOOLEAN NTAPI KeGetTrapFrameInterruptState(PKTRAP_FRAME TrapFrame)
{ return _tramp_KeGetTrapFrameInterruptState(TrapFrame); }

BOOLEAN MiIsMemoryTypeInvisible(TYPE_OF_MEMORY MemoryType)
{ return _fi_MiIsMemoryTypeInvisible(MemoryType); }

PVOID NTAPI ExAllocateFromNPagedLookasideList(PNPAGED_LOOKASIDE_LIST Lookaside)
{ return _fi_ExAllocateFromNPagedLookasideList(Lookaside); }

VOID NTAPI ExFreeToNPagedLookasideList(PNPAGED_LOOKASIDE_LIST Lookaside, PVOID Entry)
{ _fi_ExFreeToNPagedLookasideList(Lookaside, Entry); }

PVOID NTAPI ExAllocateFromPagedLookasideList(PPAGED_LOOKASIDE_LIST Lookaside)
{ return _fi_ExAllocateFromPagedLookasideList(Lookaside); }

VOID ExInitializeWorkItem(PWORK_QUEUE_ITEM Item, PWORKER_THREAD_ROUTINE Routine, PVOID Context)
{ _tramp_ExInitializeWorkItem(Item, Routine, Context); }

VOID NTAPI FsRtlExitFileSystem(VOID)
{ _tramp_FsRtlExitFileSystem(); }

VOID NTAPI FsRtlEnterFileSystem(VOID)
{ _tramp_FsRtlEnterFileSystem(); }

VOID CcBugCheck(ULONG A, ULONG B, ULONG C)
{ _tramp_CcBugCheck(A, B, C); }

PPFN_NUMBER NTAPI MmGetMdlPfnArray(PMDL Mdl)
{ return _tramp_MmGetMdlPfnArray(Mdl); }

PVOID NTAPI MmGetSystemAddressForMdl(PMDL Mdl)
{ return _tramp_MmGetSystemAddressForMdl(Mdl); }

VOID NTAPI RtlInitializeSplayLinks(PRTL_SPLAY_LINKS Links)
{ _tramp_RtlInitializeSplayLinks(Links); }

VOID NTAPI RtlInsertAsLeftChild(PRTL_SPLAY_LINKS ParentLinks, PRTL_SPLAY_LINKS ChildLinks)
{ _tramp_RtlInsertAsLeftChild(ParentLinks, ChildLinks); }

VOID NTAPI RtlInsertAsRightChild(PRTL_SPLAY_LINKS ParentLinks, PRTL_SPLAY_LINKS ChildLinks)
{ _tramp_RtlInsertAsRightChild(ParentLinks, ChildLinks); }

BOOLEAN NTAPI RtlIsLeftChild(PRTL_SPLAY_LINKS Links)
{ return _tramp_RtlIsLeftChild(Links); }

PRTL_SPLAY_LINKS NTAPI RtlParent(PRTL_SPLAY_LINKS Links)
{ return _tramp_RtlParent(Links); }

NTSTATUS RtlLongLongAdd(LONGLONG Augend, LONGLONG Addend, LONGLONG* pResult)
{ return _fi_RtlLongLongAdd(Augend, Addend, pResult); }

VOID NTAPI ExceptionRecord32To64(IN PEXCEPTION_RECORD32 Ex32, OUT PEXCEPTION_RECORD64 Ex64)
{ _fi_ExceptionRecord32To64(Ex32, Ex64); }

/* ExpKdbgExtFileCache / ExpKdbgExtDefWrites - KDBG debug extensions.
 * Only needed when CC subsystem is in Rust (C view.c provides the real versions). */
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

/* EOF */
