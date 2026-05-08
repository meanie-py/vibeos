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
 *   3. **i386 trap-exit FORCEINLINE outlines** — `KiExitV86Trap`,
 *      `KiExitTrapDebugChecks`, `KiExitSystemCallDebugChecks` capture the
 *      header bodies via the `_saved_*` rename trick. Translating them
 *      to Rust is multi-hour PSEH-intertwined work; deferred.
 *   4. **i386 init / startup** — `Ki386InitializeTss`, `KiThreadStartup`,
 *      `KiAttachProcess` are real implementations that haven't been
 *      ported to Rust yet. `KeWin32Pre/PostServiceHook` globals.
 *   5. **amd64 KVM workaround** — `KiSaveXState`/`KiRestoreXState` force
 *      `fxsave64`/`fxrstor64` to dodge an `xrstors64` page fault that
 *      KVM exposes via mis-reported `KF_XSAVES`.
 *   6. **Tiny dispatch wrappers** — `KiEndInterrupt` (i386 cli + HAL EOI
 *      + KiEoiHelper tail), `KeGdiFlushUserBatch_call` (function-pointer
 *      dispatch).
 *   7. **ARM64 KDBG variadic stub** — `KdbpPrint` (variadic, must stay C).
 *   8. **SSDT generation include** — `<internal/napi.h>` materializes
 *      `MainSSDT[]` / `MainSSPT[]` via the `SVC_` macro for Rust to
 *      reference as extern symbols.
 */

/* === Phase 1: capture FORCEINLINE bodies as `_saved_<Name>` so the
 * non-inline definitions later in this file can reuse the original
 * symbol names. */
#define KiReleaseDispatcherLockFromSynchLevel _saved_KiReleaseDispatcherLockFromSynchLevel
#define KiReleaseApcLockFromSynchLevel        _saved_KiReleaseApcLockFromSynchLevel

/* All 3 i386 trap-exit FORCEINLINE outlines (KiExitV86Trap +
 * KiExitTrapDebugChecks + KiExitSystemCallDebugChecks) and the
 * KiDumpTrapFrame debug helper moved to ke-macros rlib (2026-05-08).
 * The `_saved_*` rename trick is no longer needed. */

/* === Phase 2: include headers (PCH-skipped, so DPRINT must be stubbed). */
#ifndef DPRINT
#define DPRINT(...) do {} while(0)
#endif
#ifndef DPRINT1
#define DPRINT1(...) do {} while(0)
#endif

#include <ntoskrnl.h>

/* === Phase 3: undef the renames so we can define real outlines below. */
#undef KiReleaseDispatcherLockFromSynchLevel
#undef KiReleaseApcLockFromSynchLevel

/* No remaining KiExit / KiEndInterrupt #undefs — all body-rename
 * trampolines retired (2026-05-08). */

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

/* ============================================================
 * KiAttachProcess — originally in procobj.c (now procobj.rs).
 * Calls 2 FORCEINLINE dispatcher/APC release helpers via `_saved_*`.
 * ============================================================ */
VOID
NTAPI
KiAttachProcess(IN PKTHREAD Thread,
                IN PKPROCESS Process,
                IN PKLOCK_QUEUE_HANDLE ApcLock,
                IN PRKAPC_STATE SavedApcState)
{
    ASSERT(Process != Thread->ApcState.Process);

    /* Increase Stack Count */
    ASSERT(Process->StackCount != MAXULONG_PTR);
    Process->StackCount++;

    /* Swap the APC Environment */
    KiMoveApcState(&Thread->ApcState, SavedApcState);

    /* Reinitialize Apc State */
    InitializeListHead(&Thread->ApcState.ApcListHead[KernelMode]);
    InitializeListHead(&Thread->ApcState.ApcListHead[UserMode]);
    Thread->ApcState.Process = Process;
    Thread->ApcState.KernelApcInProgress = FALSE;
    Thread->ApcState.KernelApcPending = FALSE;
    Thread->ApcState.UserApcPending = FALSE;

    /* Update Environment Pointers if needed */
    if (SavedApcState == &Thread->SavedApcState)
    {
        Thread->ApcStatePointer[OriginalApcEnvironment] = &Thread->SavedApcState;
        Thread->ApcStatePointer[AttachedApcEnvironment] = &Thread->ApcState;
        Thread->ApcStateIndex = AttachedApcEnvironment;
    }

    /* Check if the process is paged in */
    if (Process->State == ProcessInMemory)
    {
        _saved_KiReleaseDispatcherLockFromSynchLevel();
        _saved_KiReleaseApcLockFromSynchLevel(ApcLock);
        KiSwapProcess(Process, SavedApcState->Process);
        KiExitDispatcher(ApcLock->OldIrql);
    }
    else
    {
        DPRINT1("Errr. ReactOS doesn't support paging out processes yet...\n");
        ASSERT(FALSE);
    }
}

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

/* `KiThreadStartup` — context-switch return target. The kernel stack
 * holds a `KUINIT_FRAME` at `KernelStack` on entry; we cast through
 * shim structs because the canonical `KUINIT_FRAME` layout is internal
 * to thrdini.c (now thrdini.rs). */
DECLSPEC_NORETURN VOID FASTCALL KiServiceExit2(IN PKTRAP_FRAME TrapFrame);

typedef struct _KSWITCHFRAME_SHIM {
    PVOID ExceptionList;
    BOOLEAN ApcBypassDisable;
    PVOID RetAddr;
} KSWITCHFRAME_SHIM;

typedef struct _KSTART_FRAME_SHIM {
    PKSYSTEM_ROUTINE SystemRoutine;
    PKSTART_ROUTINE StartRoutine;
    PVOID StartContext;
    BOOLEAN UserThread;
} KSTART_FRAME_SHIM;

typedef struct _KUINIT_FRAME_SHIM {
    KSWITCHFRAME_SHIM CtxSwitchFrame;
    KSTART_FRAME_SHIM StartFrame;
    KTRAP_FRAME TrapFrame;
    FX_SAVE_AREA FxSaveArea;
} KUINIT_FRAME_SHIM;

VOID
NTAPI
KiThreadStartup(VOID)
{
    PKTRAP_FRAME TrapFrame;
    KSTART_FRAME_SHIM *StartFrame;
    KUINIT_FRAME_SHIM *InitFrame;

    InitFrame = (KUINIT_FRAME_SHIM *)KeGetCurrentThread()->KernelStack;
    StartFrame = &InitFrame->StartFrame;
    TrapFrame = &InitFrame->TrapFrame;

    KeLowerIrql(APC_LEVEL);

    StartFrame->SystemRoutine(StartFrame->StartRoutine, StartFrame->StartContext);

    if (!StartFrame->UserThread)
    {
        KeBugCheck(NO_USER_MODE_CONTEXT);
    }

    KiServiceExit2(TrapFrame);
}

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

/* `Ki386InitializeTss` + 2 helpers — INIT-time TSS setup. Calls many
 * FORCEINLINE/macros (KiInitializeTSS, Ke386SetTr, Ke386GetSs etc.)
 * so kept in C. KiInitializeTSS/TSS2 originally lived in cpu.c
 * (now cpu.rs). */
extern ULONG_PTR KiDoubleFaultTSS;
extern ULONG_PTR KiNMITSS;
extern ULONG KiDoubleFaultStack;

CODE_SEG("INIT")
static VOID
NTAPI
Shim_KiInitializeTSS2(IN PKTSS Tss, IN PKGDTENTRY TssEntry OPTIONAL)
{
    PUCHAR p;

    if (TssEntry) {
        TssEntry->LimitLow = sizeof(KTSS) - 1;
        TssEntry->HighWord.Bits.LimitHi = 0;
    }

    ASSERT(IOPM_COUNT == 1);
    RtlFillMemory(Tss->IoMaps[0].IoMap, IOPM_FULL_SIZE, 0xFF);

    p = (PUCHAR)(Tss->IoMaps[0].DirectionMap);
    RtlZeroMemory(p, IOPM_DIRECTION_MAP_SIZE);
    p[0] = 4; p[3] = 0x18; p[4] = 0x18;

    p = Tss->IntDirectionMap;
    RtlZeroMemory(Tss->IntDirectionMap, IOPM_DIRECTION_MAP_SIZE);
    p[0] = 4; p[3] = 0x18; p[4] = 0x18;
}

CODE_SEG("INIT")
static VOID
NTAPI
Shim_KiInitializeTSS(IN PKTSS Tss)
{
    Tss->IoMapBase = _tramp_KiComputeIopmOffset(IO_ACCESS_MAP_NONE);
    Tss->Flags = 0;
    Tss->LDT = 0;
    Tss->Ss0 = KGDT_R0_DATA;
}

CODE_SEG("INIT")
VOID
FASTCALL
Ki386InitializeTss(IN PKTSS Tss,
                   IN PKIDTENTRY Idt,
                   IN PKGDTENTRY Gdt)
{
    PKGDTENTRY TssEntry, TaskGateEntry;

    TssEntry = &Gdt[KGDT_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;

    Shim_KiInitializeTSS2(Tss, TssEntry);

    Shim_KiInitializeTSS(Tss);

    __asm__ __volatile__("ltr %%ax" : :"a" ((USHORT)KGDT_TSS));

    TaskGateEntry = (PKGDTENTRY)&Idt[8];
    TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
    TaskGateEntry->HighWord.Bits.Pres = 1;
    TaskGateEntry->HighWord.Bits.Dpl = 0;
    ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_DF_TSS;

    Tss = (PKTSS)KiDoubleFaultTSS;
    Shim_KiInitializeTSS(Tss);
    Tss->CR3 = __readcr3();
    Tss->Esp0 = KiDoubleFaultStack;
    Tss->Esp = KiDoubleFaultStack;
    Tss->Eip = PtrToUlong(KiTrap08);
    Tss->Cs = KGDT_R0_CODE;
    Tss->Fs = KGDT_R0_PCR;
    { unsigned int __ss; __asm__("movl %%ss,%0" :"=r"(__ss)); Tss->Ss = (USHORT)__ss; }
    Tss->Es = KGDT_R3_DATA | RPL_MASK;
    Tss->Ds = KGDT_R3_DATA | RPL_MASK;

    TssEntry = &Gdt[KGDT_DF_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;
    TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
    TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
    TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
    TssEntry->LimitLow = KTSS_IO_MAPS;

    TaskGateEntry = (PKGDTENTRY)&Idt[2];
    TaskGateEntry->HighWord.Bits.Type = I386_TASK_GATE;
    TaskGateEntry->HighWord.Bits.Pres = 1;
    TaskGateEntry->HighWord.Bits.Dpl = 0;
    ((PKIDTENTRY)TaskGateEntry)->Selector = KGDT_NMI_TSS;

    Tss = (PKTSS)KiNMITSS;
    Shim_KiInitializeTSS(Tss);
    Tss->CR3 = __readcr3();
    Tss->Esp0 = KiDoubleFaultStack;
    Tss->Esp = KiDoubleFaultStack;
    Tss->Eip = PtrToUlong(KiTrap02);
    Tss->Cs = KGDT_R0_CODE;
    Tss->Fs = KGDT_R0_PCR;
    { unsigned int __ss; __asm__("movl %%ss,%0" :"=r"(__ss)); Tss->Ss = (USHORT)__ss; }
    Tss->Es = KGDT_R3_DATA | RPL_MASK;
    Tss->Ds = KGDT_R3_DATA | RPL_MASK;

    TssEntry = &Gdt[KGDT_NMI_TSS / sizeof(KGDTENTRY)];
    TssEntry->HighWord.Bits.Type = I386_TSS;
    TssEntry->HighWord.Bits.Pres = 1;
    TssEntry->HighWord.Bits.Dpl = 0;
    TssEntry->BaseLow = (USHORT)((ULONG_PTR)Tss & 0xFFFF);
    TssEntry->HighWord.Bytes.BaseMid = (UCHAR)((ULONG_PTR)Tss >> 16);
    TssEntry->HighWord.Bytes.BaseHi = (UCHAR)((ULONG_PTR)Tss >> 24);
    TssEntry->LimitLow = KTSS_IO_MAPS;

}

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
