/*
 * ke_rust_shims.c — surviving C-side helpers for the Rust KE subsystem.
 *
 * Most of what used to live here (dispatcher/thread/spin/timer/APC/process/
 * device-queue/NMI locks, deferred-ready, sched, wait setup, satisfy-wait,
 * Yield/Rundown/GetLinkedTrapFrame, FORCEINLINE intrinsic wrappers,
 * MSR/CR/DR/EOI/cpuid/xsetbv/InterlockedBitTest, KiCpuId, KiGetGdtEntry,
 * KiCheckForApcDelivery, KiEnterTrap, KiHandleDebugRegisters*, KiAttachProcess,
 * KiThreadStartup, Ki386InitializeTss, the trap-exit FORCEINLINE chain, and
 * the SSDT generation tables) has been promoted to bare-name `#[no_mangle]`
 * exports in `sdk/lib/rust/ntoskrnl-rust/src/shim_exports.rs`, sibling
 * `ke-inlines` / `ke-macros` rlibs, or direct Rust translations. See
 * `docs/engineering/ntoskrnl-translation-audit.md` for the audit trail;
 * `git log --oneline -- ntoskrnl/ke/ke_rust_shims.c` has the per-trampoline
 * removal commits.
 *
 * What survives in this file falls into a few buckets:
 *   1. **HAL-IAT / CR8 IRQL plumbing** — `KeRaiseIrqlToSynchLevel` (IAT
 *      redirect on i386, CR8 inline-asm on amd64) and the i386 dllimport
 *      override decls for `KeAcquire/ReleaseInStackQueuedSpinLock` (the
 *      HAL UP version vs. our Rust UP version diverges on
 *      `(KeGetCurrentThread()|1)` storage; Rust wins via local re-decl).
 *   2. **i386 stdcall ABI bridges** — `Ke386Get{Global,Local}DescriptorTable`
 *      (dual-decoration constraint), `KiComputeIopmOffset` (macro capture).
 *   3. **i386 DBG-only globals** — `KeWin32Pre/PostServiceHook` (kd64/kdapi.c
 *      Win32k service hooks).
 *   4. **amd64 KVM workaround** — `KiSaveXState`/`KiRestoreXState` force
 *      `fxsave64`/`fxrstor64` to dodge an `xrstors64` page fault that KVM
 *      exposes via mis-reported `KF_XSAVES`.
 *   5. **Tiny dispatch wrappers** — `KeGdiFlushUserBatch_call`
 *      (function-pointer dispatch).
 *   6. **ARM64 KDBG variadic stub** — `KdbpPrint` (variadic, must stay C).
 *
 * As of 2026-05-08 this file pulls *no* kernel headers — every type and
 * decoration it needs is hand-rolled below so the file compiles cleanly
 * under cc-rs in the cargo-driven build PoC. If you need to add a new
 * trampoline that uses a kernel type or macro, prefer translating it to
 * Rust over reintroducing `<ntoskrnl.h>`.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* Minimal Windows kernel typedefs — keep this block in sync with the
 * usage below. Anything new needs to come from this block, not from
 * `<ntdef.h>` / `<ntoskrnl.h>`. */
typedef void                 VOID;
typedef unsigned char        BOOLEAN;
typedef unsigned char        UCHAR;
typedef unsigned short       USHORT;
typedef unsigned long        ULONG;
typedef unsigned long long   ULONG64;
typedef char *               PSTR;
typedef VOID *               PVOID;
typedef USHORT *             PUSHORT;
typedef UCHAR                KIRQL;
#ifdef _M_IX86
typedef unsigned int         ULONG_PTR;
#else
typedef unsigned long long   ULONG_PTR;
#endif

/* Calling conventions — clang silently ignores `__stdcall` / `__fastcall`
 * on amd64/arm64, so these expand to the right thing on every arch. */
#define NTAPI                __stdcall
#define FASTCALL             __fastcall

/* SAL annotations — empty in the kernel build. */
#define IN
#define OUT
#define _In_

/* Opaque kernel types we only ever pass by pointer. */
typedef VOID *               PKSPIN_LOCK;
typedef VOID *               PKLOCK_QUEUE_HANDLE;
typedef VOID *               PKDBG_PRESERVICEHOOK;
typedef VOID *               PKDBG_POSTSERVICEHOOK;

/* Constants */
#define DISPATCH_LEVEL       2

/* Inline asm helpers — `_fxsave64` / `_fxrstor64` would normally come
 * from `<intrin.h>`, but inlining the asm directly avoids the header
 * dependency entirely. */
#ifdef _M_AMD64
static inline void _shim_fxsave64(void *Buffer)
{
    __asm__ __volatile__("fxsave64 (%0)" : : "r"(Buffer) : "memory");
}

static inline void _shim_fxrstor64(void *Buffer)
{
    __asm__ __volatile__("fxrstor64 (%0)" : : "r"(Buffer) : "memory");
}
#endif

/*
 * On i386, `KeReleaseInStackQueuedSpinLock` / `KeAcquireInStackQueuedSpinLock`
 * carry `_DECL_HAL_KE_IMPORT` (= `DECLSPEC_IMPORT`) in the SDK headers, so
 * the compiler generates indirect calls through the HAL's IAT. Rust
 * `shim_exports.rs` provides local UP implementations that don't store
 * `(KeGetCurrentThread()|1)` into the spin lock (KxAcquireSpinLock is a
 * no-op on UP), and the HAL version bugchecks on mismatch. Re-declare
 * without `dllimport` so the linker resolves to the Rust-provided
 * version with matching UP semantics. (We don't include the headers
 * any more, so there's nothing left to override — these decls just
 * publish the FASTCALL signature for the linker.)
 */
#ifdef _M_IX86
VOID FASTCALL KeReleaseInStackQueuedSpinLock(IN PKLOCK_QUEUE_HANDLE LockHandle);
VOID FASTCALL KeAcquireInStackQueuedSpinLock(IN PKSPIN_LOCK SpinLock, IN PKLOCK_QUEUE_HANDLE LockHandle);
#endif

/* ============================================================
 * KeRaiseIrqlToSynchLevel
 *
 * i386: NTHALAPI dllimport — call directly through the HAL IAT. Trampoline /
 *   function pointer / local definition all collide with the dllimport in
 *   the same TU, so the inline asm is the only way that resolves.
 * amd64: out-of-line CR8 implementation, and since the xdk header's
 *   FORCEINLINE was retired (drivers import this export now — real NT amd64
 *   ntoskrnl exports it), this IS the amd64 export. The target level comes
 *   from the Rust kernel's KiSynchIrql (ke_defs.rs): DISPATCH_LEVEL == 2 on
 *   the UP kernel, IPI_LEVEL-2 == 12 on the SMP kernel — one binary per
 *   flavor, each raising to its own SYNCH_LEVEL.
 * ============================================================ */
#if defined(_M_IX86)
KIRQL NTAPI KeRaiseIrqlToSynchLevel(VOID)
{
    KIRQL result;
    __asm__ __volatile__(
        "call *__imp__KeRaiseIrqlToSynchLevel@0"
        : "=a"(result)
        :
        : "ecx", "edx", "memory");
    return result;
}
#elif defined(_M_AMD64)
extern const KIRQL KiSynchIrql; /* ke_defs.rs: 2 on UP, 12 on the SMP kernel */
KIRQL KeRaiseIrqlToSynchLevel(VOID)
{
    unsigned long long cr8;
    __asm__ __volatile__("mov %%cr8, %0" : "=r"(cr8));
    {
        unsigned long long new_irql = KiSynchIrql;
        __asm__ __volatile__("mov %0, %%cr8" : : "r"(new_irql));
    }
    return (KIRQL)cr8;
}
#endif

/* ============================================================
 * ARM64 KDBG variadic stub.
 * ============================================================ */
#ifdef _M_ARM64
#ifdef KDBG
VOID KdbpPrint(_In_ PSTR Format, ...)
{
    /* Nothing to do */
    (void)Format;
}
#endif
#endif

/* ============================================================
 * amd64 KVM XState workaround.
 * Force fxsave64/fxrstor64 — KVM reports KF_XSAVES via CPUID, but
 * xrstors64 faults on StateSaveArea pages that look valid via PTE
 * self-referencing. Likely IA32_XSS MSR mis-config or KVM-side bug.
 * fxsave64/fxrstor64 covers x87+SSE which is sufficient.
 * ============================================================ */
#ifdef _M_AMD64
VOID KiSaveXState(PVOID Buffer, ULONG64 ComponentMask)
{
    (void)ComponentMask;
    _shim_fxsave64(Buffer);
}

VOID KiRestoreXState(PVOID Buffer, ULONG64 ComponentMask)
{
    (void)ComponentMask;
    _shim_fxrstor64(Buffer);
}
#endif /* _M_AMD64 */

/* ============================================================
 * i386-specific shims
 * ============================================================ */
#ifdef _M_IX86

/* `KeWin32Pre/PostServiceHook` — global function pointers used by
 * kd64/kdapi.c for Win32k service hooks. Originally i386/traphdlr.c. */
#if DBG && !defined(_WINKD_)
PKDBG_PRESERVICEHOOK KeWin32PreServiceHook = NULL;
PKDBG_POSTSERVICEHOOK KeWin32PostServiceHook = NULL;
#endif

/* `Ke386GetGlobalDescriptorTable` / `Ke386GetLocalDescriptorTable` —
 * dual-decoration constraint: i386 has both extern "C" (cdecl) and
 * extern "system" (stdcall) Rust callers referencing the same name.
 * Rust forbids two source-level fn defs with the same name. The cdecl
 * variant lives bare-name in ke-macros (`extern "C"` impl); this
 * trampoline supplies the stdcall `_X@4` symbol via a one-line delegate
 * to the cdecl `_rs_*` variant. Without `<ntoskrnl.h>` we don't have
 * the `__sgdt`/`__sldt` macros from `intrin_i.h`, so no `#undef`
 * needed. */
extern void _rs_Ke386GetGlobalDescriptorTable(PVOID Limit);
extern void _rs_Ke386GetLocalDescriptorTable(PUSHORT Selector);

VOID NTAPI Ke386GetGlobalDescriptorTable(PVOID Limit)
{
    _rs_Ke386GetGlobalDescriptorTable(Limit);
}

VOID NTAPI Ke386GetLocalDescriptorTable(PUSHORT Selector)
{
    _rs_Ke386GetLocalDescriptorTable(Selector);
}

/* `KiComputeIopmOffset` — originally a macro in `ndk/i386/ketypes.h`.
 * We reproduce it inline so the file doesn't need ketypes.h:
 *   (MapNumber == IO_ACCESS_MAP_NONE)
 *      ? sizeof(KTSS)
 *      : offsetof(KTSS, IoMaps[MapNumber-1].IoMap)
 * Constants (verified by `ntkrnl-sys/size_check_i386.rs`):
 *   sizeof(KTSS)           = 8364
 *   offsetof(KTSS, IoMaps) = 0x68 (104)
 *   sizeof(KIIO_ACCESS_MAP)= IOPM_DIRECTION_MAP_SIZE (32) +
 *                            IOPM_FULL_SIZE (8196) = 8228
 *   offsetof(KIIO_ACCESS_MAP, IoMap) = 32
 * In practice every active caller passes IO_ACCESS_MAP_NONE (=0), so
 * the second branch is dead today — we still implement it generally
 * so behaviour matches the macro bit-for-bit. */
USHORT NTAPI KiComputeIopmOffset(USHORT MapNumber)
{
    if (MapNumber == 0 /* IO_ACCESS_MAP_NONE */) {
        return (USHORT)8364; /* sizeof(KTSS) */
    }
    return (USHORT)(0x68 + (MapNumber - 1) * 8228 + 32);
}

#endif /* _M_IX86 */

/* `KeGdiFlushUserBatch_call` — translated to ke-macros rlib
 * (`_rs_KeGdiFlushUserBatch_call`, 2026-05-07). ARM64 doesn't have win32k
 * GDI batching so the symbol is x86/amd64 only. */
#if defined(_M_IX86) || defined(_M_AMD64)
extern void _rs_KeGdiFlushUserBatch_call(void);
void NTAPI KeGdiFlushUserBatch_call(void)
{
    _rs_KeGdiFlushUserBatch_call();
}
#endif
