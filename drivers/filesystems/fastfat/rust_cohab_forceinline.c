/* rust_cohab_forceinline.c — Kernel FORCEINLINE wrappers for fastfat-rust.
 *
 * These functions are defined as FORCEINLINE (or static __inline) in the DDK
 * headers. We cannot redefine them in a TU that includes ntifs.h. Instead,
 * this file includes NO DDK/XDK headers and provides manual struct layouts.
 *
 * Struct offsets are verified against the ReactOS headers. Minimal and safe.
 */

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmissing-prototypes"
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wincompatible-pointer-types"
#pragma clang diagnostic ignored "-Wredundant-decls"
#pragma clang diagnostic ignored "-Wimplicit-function-declaration"
#pragma clang diagnostic ignored "-Wint-conversion"
#pragma clang diagnostic ignored "-Wpadded"

/* NO #include <ntifs.h> here — that's the whole point. */

/* Basic Windows type definitions (from ntdef.h/windef.h level, no inline fns) */
#ifndef __NTDEF__
typedef unsigned char  UCHAR;
typedef unsigned short USHORT;
typedef unsigned long  ULONG;
typedef long           LONG;
typedef void*          PVOID;
typedef int            BOOL;
typedef unsigned char  BOOLEAN;
typedef short          SHORT;
typedef unsigned short WORD;
typedef ULONG         *PULONG;
typedef UCHAR         *PUCHAR;
#ifdef _WIN64
typedef unsigned long long ULONG_PTR;
typedef long long          LONG_PTR;
typedef unsigned long long SIZE_T;
#else
typedef unsigned long      ULONG_PTR;
typedef long               LONG_PTR;
typedef unsigned long      SIZE_T;
#endif
typedef void  VOID;
typedef char  CHAR, *PCHAR;
typedef ULONG NTSTATUS;
typedef unsigned long long  ULONGLONG;
typedef long long           LONGLONG;
#endif /* __NTDEF__ */

#define TRUE  1
#define FALSE 0

typedef void*  KIRQL;
#define DISPATCH_LEVEL 2

/* ====================================================================
 * KSPIN_LOCK: just a ULONG_PTR
 * ==================================================================== */
typedef ULONG_PTR KSPIN_LOCK, *PKSPIN_LOCK;

VOID KeInitializeSpinLock(PKSPIN_LOCK Lock) { *Lock = 0; }

/* ====================================================================
 * SLIST_HEADER and InterlockedPopEntrySList / InterlockedPushEntrySList
 * We just forward to the real kernel exports.
 * ==================================================================== */

/* SLIST_ENTRY = SINGLE_LIST_ENTRY = struct with one PVOID Flink */
typedef struct { PVOID Flink; } MY_SLIST_ENTRY, *PMY_SLIST_ENTRY;

/* SLIST_HEADER (opaque; we only pass its address to the intrinsics) */
#ifdef _WIN64
typedef union {
    struct { ULONGLONG Alignment; ULONGLONG Region; } s;
    struct {
        ULONGLONG Depth:16;
        ULONGLONG Sequence:48;
        ULONGLONG Reserved:4;
        ULONGLONG NextEntry:60;
    } Header8;
} MY_SLIST_HEADER, *PMY_SLIST_HEADER;
#else
typedef union {
    ULONGLONG Alignment;
    struct { MY_SLIST_ENTRY Next; USHORT Depth; USHORT Sequence; } s;
} MY_SLIST_HEADER, *PMY_SLIST_HEADER;
#endif

/* Declare the real kernel exports we delegate to */
extern PMY_SLIST_ENTRY __fastcall InterlockedPopEntrySList(PMY_SLIST_HEADER Head);
extern PMY_SLIST_ENTRY __fastcall InterlockedPushEntrySList(PMY_SLIST_HEADER Head, PMY_SLIST_ENTRY Entry);
extern void ExFreePool(PVOID Ptr);
extern void KeEnterCriticalRegion(void);
extern void KeLeaveCriticalRegion(void);

/* KEVENT (minimal for ExInitializeFastMutex) */
typedef struct {
    /* DISPATCHER_HEADER */
    ULONG Type;
    ULONG SignalState;
    struct { PVOID Flink, Blink; } WaitListHead;
} MY_KEVENT;

extern void KeInitializeEvent(MY_KEVENT *Event, int Type, BOOLEAN InitialState);
#define SynchronizationEvent 1

/* ====================================================================
 * FAST_MUTEX layout:
 *   LONG    Count;       // +0
 *   PVOID   Owner;       // +8 (amd64), +4 (i386)
 *   ULONG   Contention;  // +16 (amd64), +8 (i386)
 *   MY_KEVENT Event;     // +24 (amd64), +12 (i386)
 *   ULONG   OldIrql;     // +56 (amd64), +28 (i386)
 * ==================================================================== */
typedef struct {
    LONG     Count;
    PVOID    Owner;
    ULONG    Contention;
    MY_KEVENT Event;
    ULONG    OldIrql;
} MY_FAST_MUTEX, *PMY_FAST_MUTEX;

#define FM_LOCK_BIT 1

VOID ExInitializeFastMutex(PMY_FAST_MUTEX FastMutex)
{
    FastMutex->Count = FM_LOCK_BIT;
    FastMutex->Owner = (PVOID)0;
    FastMutex->Contention = 0;
    KeInitializeEvent(&FastMutex->Event, SynchronizationEvent, FALSE);
    FastMutex->OldIrql = 0;
}

/* ====================================================================
 * NPAGED_LOOKASIDE_LIST (opaque — we just need to free the entry)
 * ==================================================================== */
typedef struct { UCHAR opaque[256]; } MY_NPAGED_LOOKASIDE_LIST, *PMY_NPAGED_LOOKASIDE_LIST;

VOID ExFreeToNPagedLookasideList(PMY_NPAGED_LOOKASIDE_LIST Lookaside, PVOID Entry)
{
    (void)Lookaside;
    ExFreePool(Entry);
}

PMY_SLIST_ENTRY ExInterlockedPopEntrySList(PMY_SLIST_HEADER Head, PKSPIN_LOCK Lock)
{
    (void)Lock;
    return InterlockedPopEntrySList(Head);
}

PMY_SLIST_ENTRY ExInterlockedPushEntrySList(
    PMY_SLIST_HEADER Head, PMY_SLIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    return InterlockedPushEntrySList(Head, Entry);
}

PMY_SLIST_ENTRY RtlInterlockedPopEntrySList(PMY_SLIST_HEADER Head)
{
    return InterlockedPopEntrySList(Head);
}

/* ====================================================================
 * IRP / IO_STACK_LOCATION layout (amd64)
 *
 * Only the fields we touch are listed here. The key one is
 * IRP.Tail.Overlay.CurrentStackLocation at offset 176 (amd64).
 * The CurrentLocation char field is at offset 67 (amd64).
 * ==================================================================== */

/* IO_COMPLETION_ROUTINE */
typedef NTSTATUS (*PIO_COMPLETION_ROUTINE)(PVOID DevObj, PVOID Irp, PVOID Context);

/* IO_STACK_LOCATION (minimal) */
#define SL_PENDING_RETURNED  0x01
/* Real SL_INVOKE_ON_* values from wdm.h — were wrongly 0x01/0x02/0x04 (placeholders),
 * which made IoSetCompletionRoutine write Control=0x07; the kernel checks Control&0x40
 * (real SL_INVOKE_ON_SUCCESS) so the completion routine was never invoked → async FSD
 * IRPs auto-completed+freed → FatMarkVolume double-freed them → BUGCHECK 0xC2. */
#define SL_INVOKE_ON_CANCEL  0x20
#define SL_INVOKE_ON_SUCCESS 0x40
#define SL_INVOKE_ON_ERROR   0x80

/* FIELD_OFFSET for CompletionRoutine in IO_STACK_LOCATION:
 * The IO_STACK_LOCATION.CompletionRoutine is at a large offset. We use
 * the real value from wdm.h: on amd64 it is at offset 72 within the
 * IO_STACK_LOCATION union.
 * For IoCopyCurrentIrpStackLocationToNext we copy up to that offset. */
#if defined(_M_ARM64) || defined(__aarch64__)
/* arm64 IO_STACK_LOCATION — NATURAL (unpacked) NT layout, size 0x48, IDENTICAL
 * to amd64. (2026-07-09: was PACKED size 0x44 / CompletionRoutine@0x34 — that
 * matched the OLD arm64 header bug where iotypes.h's pshpack4 guard was missing
 * _ARM64_, so arm64 C compiled the i386-packed IO_STACK_LOCATION. That guard is
 * now fixed [_ARM64_ added], so arm64 has the same 72-byte layout as amd64 and
 * this stale packed shim made IoGetNextIrpStackLocation subtract 0x44 instead of
 * 0x48 → completion routines written 4 bytes high → IofCompleteRequest unwrap on
 * a None routine → ramdisk-mount boot panic. Verified vs bindgen offset_of:
 * Control@0x03; CompletionRoutine@0x38 (56); Context@0x40; size 0x48.) */
#define IO_STACK_COMPLETION_ROUTINE_OFFSET 56
typedef struct {
    UCHAR  pre0[3];                             /* 0x00 Major,Minor,Flags */
    UCHAR  Control;                             /* 0x03 */
    UCHAR  pre1[IO_STACK_COMPLETION_ROUTINE_OFFSET - 4]; /* 0x04..0x37 */
    PIO_COMPLETION_ROUTINE CompletionRoutine;   /* 0x38 */
    PVOID  Context;                             /* 0x40 */
} MY_IO_STACK_LOCATION;                          /* total 0x48 */
#elif defined(_WIN64)
/* amd64 IO_STACK_LOCATION (verified vs ntoskrnl amd64 disasm, size 0x48):
 * Major@0 Minor@1 Flags@2 Control@3; Parameters@0x08; DeviceObject@0x28;
 * FileObject@0x30; CompletionRoutine@0x38 (56); Context@0x40.
 * The previous branch was FABRICATED (CompletionRoutine at 72 = END of the
 * struct; Control at 88 = inside the NEXT frame) — never executed before the
 * first x86 FAT mount (USB-disk e2e, 2026-07-06). */
#define IO_STACK_COMPLETION_ROUTINE_OFFSET 56
typedef struct {
    UCHAR  pre0[3];                             /* 0x00 Major,Minor,Flags */
    UCHAR  Control;                             /* 0x03 */
    UCHAR  pre1[IO_STACK_COMPLETION_ROUTINE_OFFSET - 4]; /* 0x04..0x37 */
    PIO_COMPLETION_ROUTINE CompletionRoutine;   /* 0x38 */
    PVOID  Context;                             /* 0x40 */
} MY_IO_STACK_LOCATION;                          /* total 0x48 */
#else
/* i386 IO_STACK_LOCATION (verified vs ntoskrnl i386 disasm, size 0x24):
 * Control@3; CompletionRoutine@0x1C (28); Context@0x20. */
#define IO_STACK_COMPLETION_ROUTINE_OFFSET 28
typedef struct {
    UCHAR  pre0[3];                             /* 0x00 Major,Minor,Flags */
    UCHAR  Control;                             /* 0x03 */
    UCHAR  pre1[IO_STACK_COMPLETION_ROUTINE_OFFSET - 4]; /* 0x04..0x1B */
    PIO_COMPLETION_ROUTINE CompletionRoutine;   /* 0x1C */
    PVOID  Context;                             /* 0x20 */
} MY_IO_STACK_LOCATION;                          /* total 0x24 */
#endif

/* IRP layout — only the fields we care about.
 * We use raw byte offsets verified from wdm.h.
 */
#if defined(_M_ARM64) || defined(__aarch64__)
/* arm64 IRP layout (verified vs bindgen + ntoskrnl/ramdisk disasm):
 * Tail.Overlay.CurrentStackLocation at 0xb8 (184), CurrentLocation char at 0x43 (67). */
#define IRP_CURRENT_LOCATION_OFFSET     67
#define IRP_CURRENT_STACK_PTR_OFFSET    184
#elif defined(_WIN64)
/* amd64 (verified vs ntoskrnl amd64 IofCallDriver disasm):
 * CurrentLocation@0x43 (67); Tail.Overlay.CurrentStackLocation@0xB8 (184).
 * Was 176 (0xB0) — fabricated 8 low; every fastfat IRP access read the wrong
 * slot and the FIRST x86 FAT mount AV'd in FatFsdFileSystemControl. */
#define IRP_CURRENT_LOCATION_OFFSET     67
#define IRP_CURRENT_STACK_PTR_OFFSET    184
#else
/* i386 (verified vs ntoskrnl i386 IofCallDriver disasm):
 * CurrentLocation@0x23 (35, was fabricated 30); CurrentStackLocation@0x60 (96). */
#define IRP_CURRENT_LOCATION_OFFSET     35
#define IRP_CURRENT_STACK_PTR_OFFSET    96
#endif

typedef struct { UCHAR opaque[512]; } MY_IRP;

static MY_IO_STACK_LOCATION* _IrpGetCurrentSP(MY_IRP* Irp)
{
    return *(MY_IO_STACK_LOCATION**)((PUCHAR)Irp + IRP_CURRENT_STACK_PTR_OFFSET);
}

static void _IrpSetCurrentSP(MY_IRP* Irp, MY_IO_STACK_LOCATION* sp)
{
    *(MY_IO_STACK_LOCATION**)((PUCHAR)Irp + IRP_CURRENT_STACK_PTR_OFFSET) = sp;
}

static char* _IrpGetCurrentLocation(MY_IRP* Irp)
{
    return (char*)Irp + IRP_CURRENT_LOCATION_OFFSET;
}

MY_IO_STACK_LOCATION* IoGetCurrentIrpStackLocation(MY_IRP* Irp)
{
    return _IrpGetCurrentSP(Irp);
}

MY_IO_STACK_LOCATION* IoGetNextIrpStackLocation(MY_IRP* Irp)
{
    return _IrpGetCurrentSP(Irp) - 1;
}

VOID IoCopyCurrentIrpStackLocationToNext(MY_IRP* Irp)
{
    MY_IO_STACK_LOCATION* cur  = _IrpGetCurrentSP(Irp);
    MY_IO_STACK_LOCATION* next = cur - 1;
    /* Copy everything up to CompletionRoutine */
    unsigned i;
    for (i = 0; i < IO_STACK_COMPLETION_ROUTINE_OFFSET; i++)
        ((PUCHAR)next)[i] = ((PUCHAR)cur)[i];
    next->Control = 0;
}

VOID IoMarkIrpPending(MY_IRP* Irp)
{
    _IrpGetCurrentSP(Irp)->Control |= SL_PENDING_RETURNED;
}

VOID IoSetNextIrpStackLocation(MY_IRP* Irp)
{
    (*_IrpGetCurrentLocation(Irp))--;
    _IrpSetCurrentSP(Irp, _IrpGetCurrentSP(Irp) - 1);
}

VOID IoSkipCurrentIrpStackLocation(MY_IRP* Irp)
{
    (*_IrpGetCurrentLocation(Irp))++;
    _IrpSetCurrentSP(Irp, _IrpGetCurrentSP(Irp) + 1);
}

VOID IoSetCompletionRoutine(
    MY_IRP* Irp,
    PIO_COMPLETION_ROUTINE Routine,
    PVOID Context,
    BOOLEAN OnSuccess,
    BOOLEAN OnError,
    BOOLEAN OnCancel)
{
    MY_IO_STACK_LOCATION* sp = _IrpGetCurrentSP(Irp) - 1;
    sp->CompletionRoutine = Routine;
    sp->Context = Context;
    sp->Control = 0;
    if (OnSuccess) sp->Control |= SL_INVOKE_ON_SUCCESS;
    if (OnError)   sp->Control |= SL_INVOKE_ON_ERROR;
    if (OnCancel)  sp->Control |= SL_INVOKE_ON_CANCEL;
}

/* ====================================================================
 * FsRtl helpers that are macros (already in stub, but in case of
 * duplicate-definition, provide them here as weak aliases)
 * ==================================================================== */

VOID FsRtlEnterFileSystem(VOID) { KeEnterCriticalRegion(); }
VOID FsRtlExitFileSystem(VOID)  { KeLeaveCriticalRegion(); }

/* ====================================================================
 * ObReferenceObject — maps to ObReferenceObjectWithTag in ntoskrnl
 * ==================================================================== */
extern void ObReferenceObjectWithTag(PVOID Object, ULONG Tag);

VOID ObReferenceObject(PVOID Object)
{
    ObReferenceObjectWithTag(Object, 'IrpC');
}

/* ====================================================================
 * __int2c — clang builtin, cannot define normally. Use global_asm.
 * On x86/x64: emits INT 0x2C (assertion failure interrupt).
 * On ARM64: BRK #0x2C equivalent.
 * ==================================================================== */
#if defined(__x86_64__) || defined(__i386__)
__asm__(".globl __int2c\n"
        "__int2c:\n"
        "    int $0x2c\n"
        "    ret\n");
#elif defined(__aarch64__)
__asm__(".globl __int2c\n"
        "__int2c:\n"
        "    brk #0\n"
        "    ret\n");
#endif

/* i386 dual-decoration bridge: the bindgen ddk substrate models these FORCEINLINE
 * helpers as `extern "C"` (bare, caller pops), but other fastfat modules
 * hand-declare them `extern "system"` (@N, callee pops), and -mrtd makes our
 * definitions @N. The bare name therefore needs a real cdecl THUNK, not a symbol
 * alias of the bare name onto `_X@4` resolves the link but hands a cdecl caller a body that
 * pops its argument a second time. fastfat's DriverEntry calls two of these, so on
 * i386 it returned with ESP 8 bytes high, popped its own return address into EBX
 * and `ret`-ed into its RegistryPath argument on the caller's stack -- BUGCHECK
 * 0x1E with EIP on the stack, the first time a FAT volume made fs_rec load it.
 * An alias is only sound for @0, where the two conventions are the same code.
 *
 * The thunk re-pushes the N bytes of arguments (each `pushl N(%esp)` fetches the
 * next-lower argument, since the push itself moved ESP by 4), calls the stdcall
 * body, which pops the copies, and returns leaving the caller's own to the caller.
 * (__int2c: bridge the underscore/stdcall-arity gap between the asm symbol and the
 * Rust @0 reference.) */
#if defined(__i386__)
__asm__(
    ".text\n"
    ".macro CDECL_TO_STDCALL name, bytes\n"
    "    .globl _\\name\n"
    "_\\name:\n"
    "    .rept \\bytes / 4\n"
    "    pushl \\bytes(%esp)\n"
    "    .endr\n"
    "    call _\\name\\()@\\bytes\n"
    "    ret\n"
    ".endm\n"
    "CDECL_TO_STDCALL ExInitializeFastMutex, 4\n"
    "CDECL_TO_STDCALL IoGetCurrentIrpStackLocation, 4\n"
    "CDECL_TO_STDCALL IoGetNextIrpStackLocation, 4\n"
    "CDECL_TO_STDCALL IoMarkIrpPending, 4\n"
    "CDECL_TO_STDCALL IoSetCompletionRoutine, 24\n"
    "CDECL_TO_STDCALL IoSetNextIrpStackLocation, 4\n"
    "CDECL_TO_STDCALL IoCopyCurrentIrpStackLocationToNext, 4\n"
    "CDECL_TO_STDCALL IoSkipCurrentIrpStackLocation, 4\n"
    "CDECL_TO_STDCALL KeInitializeSpinLock, 4\n"
    ".globl ___int2c@0\n                    .set ___int2c@0, __int2c\n");
#endif

#pragma clang diagnostic pop
