/*
 * ke_rust_shims.c — surviving C-side helpers for the Rust KE subsystem.
 *
 * Most of what used to live here (dispatcher/thread/spin/timer/APC/process/
 * device-queue/NMI locks, deferred-ready, sched, wait setup, satisfy-wait,
 * Yield/Rundown/GetLinkedTrapFrame, FORCEINLINE intrinsic wrappers,
 * MSR/CR/DR/EOI/cpuid/xsetbv/InterlockedBitTest, KiCpuId, KiGetGdtEntry,
 * KiCheckForApcDelivery, KiEnterTrap, KiHandleDebugRegisters*, etc.) has
 * been promoted to bare-name `#[no_mangle] extern "C" fn` exports in
 * `sdk/lib/rust/ntoskrnl-rust/src/shim_exports.rs` or to sibling
 * `ke-inlines` / `ke-macros` rlibs. See `docs/engineering/shim-elimination.md`
 * for the audit trail; `git log --oneline -- ntoskrnl/ke/ke_rust_shims.c`
 * has the per-trampoline removal commits.
 *
 * What survives in this file falls into a few buckets:
 *   1. **HAL-IAT / CR8 IRQL plumbing** — `KeRaiseIrqlToSynchLevel` (IAT
 *      redirect on i386, CR8 inline-asm on amd64) and the i386 dllimport
 *      override decls for `KeAcquire/ReleaseInStackQueuedSpinLock` (the
 *      HAL UP version vs. our Rust UP version diverges on
 *      `(KeGetCurrentThread()|1)` storage; Rust wins via local re-decl).
 *   2. **Stdcall-on-i386 ABI bridges** — bindgen on i386 emits these as
 *      `extern "system"`, and `shim_exports.rs` only provides the cdecl
 *      counterpart, so the stdcall variant has to come from a C
 *      trampoline. Affects HalQuery/SetSystemInformation,
 *      HalVectorToIDTEntry, Ke386Get{Global,Local}DescriptorTable,
 *      KiComputeIopmOffset, KeGetTrapFramePc.
 *   3. **i386 DBG-only globals** — `KeWin32Pre/PostServiceHook` globals
 *      (kd64/kdapi.c Win32k service hooks).
 *   4. **amd64 KVM workaround** — `KiSaveXState`/`KiRestoreXState` force
 *      `fxsave64`/`fxrstor64` to dodge an `xrstors64` page fault that
 *      KVM exposes via mis-reported `KF_XSAVES`.
 *   5. **Tiny dispatch wrappers** — `KeGdiFlushUserBatch_call`
 *      (function-pointer dispatch).
 *   6. **ARM64 KDBG variadic stub** — `KdbpPrint` (variadic, must stay C).
 *   7. **SSDT generation include** — `<internal/napi.h>` materializes
 *      `MainSSDT[]` / `MainSSPT[]` via the `SVC_` macro for Rust to
 *      reference as extern symbols.
 */

/* All 3 i386 trap-exit FORCEINLINE outlines (KiExitV86Trap +
 * KiExitTrapDebugChecks + KiExitSystemCallDebugChecks), the
 * KiDumpTrapFrame debug helper, and KiAttachProcess all moved to Rust
 * (2026-05-08). The `_saved_*` rename trick that captured FORCEINLINE
 * bodies is no longer needed. */

#ifndef DPRINT
#define DPRINT(...) do {} while(0)
#endif
#ifndef DPRINT1
#define DPRINT1(...) do {} while(0)
#endif

#include <ntoskrnl.h>

/*
 * On i386, `KeReleaseInStackQueuedSpinLock` / `KeAcquireInStackQueuedSpinLock`
 * carry `_DECL_HAL_KE_IMPORT` (= `DECLSPEC_IMPORT`) in the SDK headers, so
 * the compiler generates indirect calls through the HAL's IAT. Rust
 * `shim_exports.rs` provides local UP implementations that don't store
 * `(KeGetCurrentThread()|1)` into the spin lock (KxAcquireSpinLock is a
 * no-op on UP), and the HAL version bugchecks on mismatch. Re-declare
 * without `dllimport` so the linker resolves to the Rust-provided
 * version with matching UP semantics.
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
 * amd64: FORCEINLINE over CR8. Inlined-trampoline pattern can fail (no
 *   inline → infinite recursion → `jmp $`), so reimplement with CR8
 *   intrinsics. Without CONFIG_SMP, SYNCH_LEVEL == DISPATCH_LEVEL == 2.
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
#undef KeRaiseIrqlToSynchLevel
KIRQL KeRaiseIrqlToSynchLevel(VOID)
{
    unsigned long long cr8;
    __asm__ __volatile__("mov %%cr8, %0" : "=r"(cr8));
#ifdef CONFIG_SMP
    { unsigned long long new_irql = 12; /* SYNCH_LEVEL = IPI_LEVEL - 2 */
    __asm__ __volatile__("mov %0, %%cr8" : : "r"(new_irql)); }
#else
    { unsigned long long new_irql = DISPATCH_LEVEL;
    __asm__ __volatile__("mov %0, %%cr8" : : "r"(new_irql)); }
#endif
    return (KIRQL)cr8;
}
#endif

/* ============================================================
 * Stdcall-on-i386 ABI bridges (cdecl shim_exports + stdcall here).
 * Each `_rs_*` impl lives in `ke-macros` rlib; the trampoline supplies
 * the NTAPI/stdcall decoration that bindgen i386 callers want.
 * ============================================================ */
/* KeGetTrapFramePc — promoted to bare-name `extern "system"` export in
 * ke-macros rlib (2026-05-08); trampoline retired. */

/* HalQuerySystemInformation / HalSetSystemInformation — promoted to
 * bare-name `extern "system"` exports in ke-macros rlib (2026-05-08);
 * trampolines retired. */

/* ============================================================
 * ARM64 KDBG variadic stub.
 * ============================================================ */
#ifdef _M_ARM64
#ifdef KDBG
VOID KdbpPrint(_In_ PSTR Format, ...)
{
    /* Nothing to do */
}
#endif
#endif

/* ============================================================
 * SSDT tables (MainSSDT[] / MainSSPT[]) — generated by the SVC_
 * macro in <internal/napi.h>. Originally in krnlinit.c (now Rust);
 * Rust references the tables as extern symbols.
 * ============================================================ */
#include <internal/napi.h>

/* KiAttachProcess — translated to Rust (`ke/procobj.rs`, 2026-05-08).
 * The original C body lived here because it called the FORCEINLINE pair
 * `KiReleaseDispatcherLockFromSynchLevel` / `KiReleaseApcLockFromSynchLevel`
 * via `_saved_*` capture. Both now exist as `_rs_*` exports in `ke-inlines`
 * with bare-name `extern "C"` trampolines in `shim_exports.rs`, so the Rust
 * impl calls them directly. */

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
    _fxsave64(Buffer);
}

VOID KiRestoreXState(PVOID Buffer, ULONG64 ComponentMask)
{
    _fxrstor64(Buffer);
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
 * variant lives bare-name in ke-macros (`extern "C"` impl at ~L860);
 * this trampoline supplies the stdcall `_X@4` symbol via a one-line
 * delegate to the cdecl `_rs_*` variant. Re-probed 2026-05-08. */
#undef Ke386GetGlobalDescriptorTable
#undef Ke386GetLocalDescriptorTable
extern void _rs_Ke386GetGlobalDescriptorTable(PVOID Limit);
extern void _rs_Ke386GetLocalDescriptorTable(PUSHORT Selector);
VOID NTAPI Ke386GetGlobalDescriptorTable(PVOID Limit)
{ _rs_Ke386GetGlobalDescriptorTable(Limit); }
VOID NTAPI Ke386GetLocalDescriptorTable(PUSHORT Selector)
{ _rs_Ke386GetLocalDescriptorTable(Selector); }

/* HalVectorToIDTEntry — promoted to bare-name `extern "system"` export
 * in ke-macros rlib (2026-05-08); trampoline retired. */

/* `KiThreadStartup` — translated to Rust (`ke/i386/thrdini.rs`,
 * 2026-05-08). Context-switch return target; the KUINIT_FRAME layout
 * lives in thrdini.rs alongside KiInitializeContextThread which builds
 * the frame, so the shim's parallel KUINIT_FRAME_SHIM struct is no
 * longer needed. */

/* `KiComputeIopmOffset` — macro in ndk/i386/ketypes.h. Trampoline
 * captures the macro expansion. */
static __inline USHORT _tramp_KiComputeIopmOffset(USHORT MapNumber)
{
    return (USHORT)KiComputeIopmOffset(MapNumber);
}
#undef KiComputeIopmOffset
USHORT NTAPI KiComputeIopmOffset(USHORT MapNumber)
{
    return _tramp_KiComputeIopmOffset(MapNumber);
}

/* `Ki386InitializeTss` translated to Rust (`ke/i386/cpu.rs`,
 * 2026-05-08). The KiInitializeTSS / KiInitializeTSS2 private helpers
 * already lived alongside it in cpu.rs; the shim's `Shim_*` copies were
 * just trampolines for the FORCEINLINE captures. The Rust impl uses
 * direct inline asm for `ltr ax` and `mov %ss,...` instead. */

/* KiEndInterrupt — promoted to direct Rust impl in ke-macros rlib
 * (2026-05-08). The audit's "rustc 1.92 stdcall/fastcall ABI bug"
 * re-probe came back negative on rustc 1.92.0 — disasm of the
 * KiExitV86Trap pilot showed correct fastcall arg-passing for the same
 * @KfRaiseIrql@4 / @KiEoiHelper@4 calls KiEndInterrupt makes. */

/* All i386 trap-exit FORCEINLINE outlines now live in `ke-macros` rlib
 * (2026-05-08). The block below was the last to land — `KiExitV86Trap`
 * (NORETURN) drains pending user APCs in a loop, optionally restores DR
 * registers, and tail-calls `KiTrapReturnNoSegments` (asm in
 * `i386/trap.s`). */

#endif /* _M_IX86 */

/* `KeGdiFlushUserBatch_call` — translated to ke-macros rlib
 * (`_rs_KeGdiFlushUserBatch_call`, 2026-05-07). ARM64 doesn't have win32k
 * GDI batching so the symbol is x86/amd64 only. */
#if defined(_M_IX86) || defined(_M_AMD64)
extern void _rs_KeGdiFlushUserBatch_call(void);
void NTAPI KeGdiFlushUserBatch_call(void)
{ _rs_KeGdiFlushUserBatch_call(); }
#endif
