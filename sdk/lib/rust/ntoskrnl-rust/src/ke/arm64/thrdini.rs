use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _KTHREAD, _KPROCESS, _CONTEXT,
    PKTHREAD, PKPROCESS, PCONTEXT, PKTRAP_FRAME, PKSTART_ROUTINE, PKSYSTEM_ROUTINE,
    KTRAP_FRAME, PKIPCR,
    KeInitializeThread, KeContextToTrapFrame,
    KiThreadStartup,
    KiSwapContext, KiSwapProcess, KiRetireDpcList, KiSelectReadyThread,
    KiInitializeContextThread as _KiInitializeContextThread,
    HalRequestSoftwareInterrupt, HalClearSoftwareInterrupt,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn KiInvalidSystemThreadStartupExit();
    fn KiUserThreadStartupExit();
}

#[repr(C)]
struct KSWITCH_FRAME {
    // Layout must match ctxswitch.S: `stp x0, x30, [sp, #-16]!`
    // x0 (ApcBypass) at [sp+0], x30 (Return) at [sp+8]
    ApcBypass: BOOLEAN,
    _padding: [u8; 7],
    Return: u64,
}

#[repr(C)]
struct KSTART_FRAME {
    P1Home: u64,
    P2Home: u64,
    P3Home: u64,
    Return: u64,
}

#[repr(C)]
struct KKINIT_FRAME {
    CtxSwitchFrame: KSWITCH_FRAME,
    StartFrame: KSTART_FRAME,
}

#[repr(C)]
struct KUINIT_FRAME {
    CtxSwitchFrame: KSWITCH_FRAME,
    StartFrame: KSTART_FRAME,
    TrapFrame: KTRAP_FRAME,
}

const _: () = {
    assert!(core::mem::offset_of!(_KTHREAD, KernelStack) == 0x38);
    assert!(core::mem::offset_of!(_KTHREAD, SwapBusy) == 0x95);
    assert!(core::mem::offset_of!(_KTHREAD, Teb) == 0xB0);
};

#[no_mangle]
pub unsafe extern "system" fn KiInitializeContextThread(
    Thread: PKTHREAD,
    SystemRoutine: PKSYSTEM_ROUTINE,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
    ContextRecord: PCONTEXT,
) {
    let Thread = opaque_ptr(Thread);
    let StartContext = opaque_ptr(StartContext);
    let mut InitialStack = ((*Thread).InitialStack as *mut u8).sub(512); // nolint: magic-offset (stack reservation size, mirrors C thrdini)
    InitialStack = (InitialStack as usize & !(15usize)) as *mut u8;
    (*Thread).InitialStack = InitialStack as PVOID;

    if !ContextRecord.is_null() {
        let UInitFrame = (InitialStack as *mut KUINIT_FRAME).sub(1);
        let TrapFrame = core::ptr::addr_of_mut!((*UInitFrame).TrapFrame);
        let StartFrame = core::ptr::addr_of_mut!((*UInitFrame).StartFrame);
        let CtxSwitchFrame = core::ptr::addr_of_mut!((*UInitFrame).CtxSwitchFrame);

        (*Thread).KernelStack = UInitFrame as PVOID;

        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 1i8;

        RtlZeroMemory(TrapFrame as PVOID, core::mem::size_of::<KTRAP_FRAME>());
        KeContextToTrapFrame(
            ContextRecord,
            core::ptr::null_mut(),
            TrapFrame,
            (*ContextRecord).ContextFlags,
            1i8,
        );

        (*TrapFrame).Spsr = 0;
        (*TrapFrame).__bindgen_anon_1.__bindgen_anon_1.PreviousMode = 1i8;

        (*TrapFrame).__bindgen_anon_3.__bindgen_anon_1.X18 = (*Thread).Teb as u64;

        (*CtxSwitchFrame).Return = KiThreadStartup as u64;
        (*CtxSwitchFrame).ApcBypass = 1u8;

        (*StartFrame).P1Home = StartRoutine.map(|f| f as u64).unwrap_or(0);
        (*StartFrame).P2Home = StartContext as u64;
        (*StartFrame).P3Home = SystemRoutine.map(|f| f as u64).unwrap_or(0);
        (*StartFrame).Return = KiUserThreadStartupExit as u64;
    } else {
        let InitFrame = (InitialStack as *mut KKINIT_FRAME).sub(1);
        let StartFrame = core::ptr::addr_of_mut!((*InitFrame).StartFrame);
        let CtxSwitchFrame = core::ptr::addr_of_mut!((*InitFrame).CtxSwitchFrame);

        (*Thread).KernelStack = InitFrame as PVOID;

        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 0i8;

        (*CtxSwitchFrame).Return = KiThreadStartup as u64;
        (*CtxSwitchFrame).ApcBypass = 1u8;

        (*StartFrame).P1Home = StartRoutine.map(|f| f as u64).unwrap_or(0);
        (*StartFrame).P2Home = StartContext as u64;
        (*StartFrame).P3Home = SystemRoutine.map(|f| f as u64).unwrap_or(0);
        (*StartFrame).Return = KiInvalidSystemThreadStartupExit as u64;
    }

    // cswitchdep birth brand: pool-reused KTHREAD addresses must never
    // inherit a dead thread's shadow (see amd64/thrdini.rs).
    #[cfg(feature = "cswitchdep")]
    crate::ke::cswitchdep::cswitch_thread_init(Thread as usize);
}

#[no_mangle]
pub unsafe extern "system" fn KiSwapContextResume(
    ApcBypass: BOOLEAN,
    OldThread: PKTHREAD,
    NewThread: PKTHREAD,
) -> BOOLEAN {
    let OldThread = opaque_ptr(OldThread);
    let NewThread = opaque_ptr(NewThread);
    // NOT ddk::KeGetPcr: the linked KeGetPcr symbol is the x18-convention C
    // copy (`mov x0, x18`), and x18 is NOT reloaded by the switch frame — in
    // this fresh-off-the-switch context it read 0 (first-light CSWDIAG,
    // 2026-07-07), so the KeContextSwitches bump, the DpcRoutineActive
    // BugCheck probe, and tap 2 were all dereferencing NULL+offset silently.
    // TPIDR_EL1 is the invariant PCR source (ctxswitch.S uses it too).
    let Pcr = ntkrnl_sys::inline::ke::KeGetPcr() as PKIPCR;
    let OldProcess = (*OldThread).__bindgen_anon_1.ApcState.Process;
    let NewProcess = (*NewThread).__bindgen_anon_1.ApcState.Process;

    // Interrupt-quest tripwire. Context switches entered with IRQs masked are
    // ROUTINE on this port (SVC exceptions run DAIF-masked, so every syscall
    // wait parks a masked context that self-heals at ERET — a known
    // characteristic, see the arm64-interrupts quest doc). The FATAL shape is
    // a masked swap performed while the IRQ trap-exit path is retiring DPCs:
    // its wake chain can hand a brand-new thread a masked DAIF (mitigated by
    // KiThreadStartup's daifclr, ctxswitch.S) — report it loudly so any
    // recurrence is diagnosable from serial instead of a silent wedge.
    {
        let daif: u64;
        core::arch::asm!("mrs {0}, daif", out(reg) daif, options(nomem, nostack, preserves_flags));
        if daif & 0x80 != 0
            && crate::ke::arm64::trap::KI_IN_IRQ_EXCEPTION_RETIRE
                .load(core::sync::atomic::Ordering::SeqCst)
        {
            static SWAP_MASKED: core::sync::atomic::AtomicU32 =
                core::sync::atomic::AtomicU32::new(0);
            if SWAP_MASKED.fetch_add(1, core::sync::atomic::Ordering::Relaxed) < 8 {
                crate::dbg_print!(
                    b"[SWAP-TRIPWIRE] masked KiSwapContext inside IRQ-exit DPC retire! daif=%p old=%p new=%p\n\0",
                    daif as usize,
                    OldThread,
                    NewThread
                );
            }
        }
    }

    if OldProcess != NewProcess {
        KiSwapProcess(NewProcess, OldProcess);
    }

    (*Pcr).Prcb.KeContextSwitches += 1;
    (*NewThread).__bindgen_anon_5.__bindgen_anon_5.ContextSwitches += 1;

    // M5c SMP visibility: one-shot "[SMP] CPUn scheduling live" markers when
    // an AP switches onto a real (non-idle) thread — count-only tap (NEVER
    // DbgPrint mid-swap: SwapBusy is still set); the print is flushed from
    // the idle loop. Shared machinery: ke/ipi.rs.
    #[cfg(feature = "smp")]
    {
        if (*Pcr).Prcb.Number != 0
            && NewThread as usize != (*Pcr).Prcb.IdleThread as usize
        {
            crate::ke::ipi::KiSmpNoteApSwitch((*Pcr).Prcb.Number as u32);
        }
    }

    // cswitchdep tap 2 (post-swap, on the NEW thread's stack) — arm64 parity
    // with amd64/thrdini.rs; runs BEFORE the ATTEMPTED_SWITCH_FROM_DPC
    // BugCheck below so the record reaches the table + serial either way.
    #[cfg(feature = "cswitchdep")]
    {
        let sp_probe = 0u8;
        crate::ke::cswitchdep::cswitch_swap_resume(
            OldThread as usize,
            NewThread as usize,
            (*Pcr).Prcb.CurrentThread as usize,
            core::ptr::addr_of!(sp_probe) as usize,
            (*NewThread).StackLimit as usize,
            (*NewThread).InitialStack as usize,
            (*Pcr).Prcb.DpcRoutineActive != 0,
            NewThread as usize == (*Pcr).Prcb.IdleThread as usize,
            crate::code_addr!(),
        );
    }

    if (*Pcr).Prcb.DpcRoutineActive != 0 {
        KeBugCheckEx(
            ATTEMPTED_SWITCH_FROM_DPC,
            OldThread as ULONG_PTR,
            NewThread as ULONG_PTR,
            (*OldThread).InitialStack as ULONG_PTR,
            0,
        );
    }

    (*OldThread).SwapBusy = 0u8;

    if (*NewThread).__bindgen_anon_1.ApcState.KernelApcPending != 0 {
        if (*NewThread).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable == 0 && ApcBypass == 0 {
            return 1u8;
        }
        HalRequestSoftwareInterrupt(APC_LEVEL);
    }

    0u8
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn KiIdleLoop() -> ! {
    let Prcb = KeGetCurrentPrcb();

    // The idle loop runs at DISPATCH_LEVEL (as on x86/NT): it retires DPCs and
    // calls KiSwapContext with IRQs enabled, and the IRQ trap-exit DPC-retire
    // path (KiIrqHandler) keys off IRQL >= DISPATCH to know it must NOT retire
    // from exception context here — pinning the level closes that race. The
    // idle thread's WaitIrql is DISPATCH, so swaps back into idle restore it.
    {
        let mut OldIrql: KIRQL = 0;
        KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
    }

    loop {
        core::arch::asm!("msr daifclr, #2");
        core::hint::spin_loop();
        core::hint::spin_loop();
        core::arch::asm!("msr daifset, #2");

        // M5c SMP visibility: flush any earned "[SMP] CPUn scheduling live"
        // marker — idle-thread context, safe to DbgPrint (the count tap in
        // KiSwapContextResume must not). Two relaxed loads when idle.
        #[cfg(feature = "smp")]
        crate::ke::ipi::KiSmpApMarkerFlush((*Prcb).Number as u32);

        if (*Prcb).DpcData[0].DpcQueueDepth != 0
            || (*Prcb).TimerRequest != 0
            || !(*Prcb).DeferredReadyListHead.Next.is_null()
        {
            HalClearSoftwareInterrupt(DISPATCH_LEVEL);
            KiRetireDpcList(Prcb);
        }

        if !(*Prcb).NextThread.is_null() {
            // UP keeps the historical shape: unmask, then a lock-free consume
            // (single CPU — nothing can race it).
            #[cfg(not(feature = "smp"))]
            {
                core::arch::asm!("msr daifclr, #2");

                let OldThread = (*Prcb).CurrentThread;
                let NewThread = (*Prcb).NextThread;

                (*Prcb).NextThread = core::ptr::null_mut();
                (*Prcb).CurrentThread = NewThread;

                (*NewThread).__bindgen_anon_5.__bindgen_anon_6.State = Running as UCHAR;

                KiSwapContext(APC_LEVEL, OldThread);
            }

            // SMP (§9.56, mirrors amd64/stubs.rs KiIdleLoop): consume
            // NextThread UNDER THE PRCB LOCK with IRQs still masked. The
            // lock-free consume races KiDeferredReadyThread's standby
            // DISPLACEMENT (which runs under this lock and re-defers a
            // NextThread it reads) — idle would promote T to Running while
            // the displacement sets T DeferredReady → double-link tear.
            // Re-check under the lock, promote + in-flight unmark, release,
            // THEN unmask, and do the swap at SYNCH_LEVEL like amd64 (C:
            // KfRaiseIrql(SYNCH_LEVEL) around KiSwapContext). arm64 SMP
            // SYNCH_LEVEL is 12 as of 2026-07-25; while it read DISPATCH the
            // raise was a no-op and this was skipped.
            #[cfg(feature = "smp")]
            {
                ntkrnl_sys::inline::ke::KiAcquirePrcbLock(Prcb);
                if (*Prcb).NextThread.is_null() {
                    ntkrnl_sys::inline::ke::KiReleasePrcbLock(Prcb);
                    continue;
                }

                let OldThread = (*Prcb).CurrentThread;
                let NewThread = (*Prcb).NextThread;

                (*Prcb).NextThread = core::ptr::null_mut();
                (*Prcb).CurrentThread = NewThread;

                (*NewThread).__bindgen_anon_5.__bindgen_anon_6.State = Running as UCHAR;

                // §9.56 in-flight oracle: promotion-to-Running unmark.
                // Without this every idle-loop promote leaks a stale mark
                // and the thread's next wake false-positives a [RQDEFER2]
                // DOUBLE-DEFER (seen spamming the -smp 2 settle log).
                ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread);

                ntkrnl_sys::inline::ke::KiReleasePrcbLock(Prcb);

                core::arch::asm!("msr daifclr, #2");

                // C: KfRaiseIrql(SYNCH_LEVEL) — do the swap at SYNCH_LEVEL.
                crate::ke::ke_defs::KfRaiseIrql(crate::ke::ke_defs::SYNCH_LEVEL);
                KiSwapContext(APC_LEVEL, OldThread);
                // C: KeLowerIrql(DISPATCH_LEVEL) — the idle loop is pinned there.
                crate::ke::ke_defs::KeLowerIrql(DISPATCH_LEVEL);
            }
        } else if (*Prcb).ReadySummary != 0 {
            #[cfg(not(feature = "smp"))]
            {
                core::arch::asm!("msr daifclr, #2");

                let OldThread = (*Prcb).CurrentThread;
                let NewThread = KiSelectReadyThread(0, Prcb);
                if !NewThread.is_null() {
                    (*Prcb).CurrentThread = NewThread;
                    (*NewThread).__bindgen_anon_5.__bindgen_anon_6.State = Running as UCHAR;
                    KiSwapContext(APC_LEVEL, OldThread);
                }
            }

            // SMP: the per-PRCB ready lists are PRCB-lock serialized
            // (KiSwapThread selects under the lock) — select + promote under
            // it here too, IRQs masked until release.
            #[cfg(feature = "smp")]
            {
                ntkrnl_sys::inline::ke::KiAcquirePrcbLock(Prcb);

                let OldThread = (*Prcb).CurrentThread;
                let NewThread = KiSelectReadyThread(0, Prcb);
                if NewThread.is_null() {
                    ntkrnl_sys::inline::ke::KiReleasePrcbLock(Prcb);
                    continue;
                }

                (*Prcb).CurrentThread = NewThread;
                (*NewThread).__bindgen_anon_5.__bindgen_anon_6.State = Running as UCHAR;
                ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread);
                ntkrnl_sys::inline::ke::KiReleasePrcbLock(Prcb);

                core::arch::asm!("msr daifclr, #2");

                // C: KfRaiseIrql(SYNCH_LEVEL) — do the swap at SYNCH_LEVEL.
                crate::ke::ke_defs::KfRaiseIrql(crate::ke::ke_defs::SYNCH_LEVEL);
                KiSwapContext(APC_LEVEL, OldThread);
                // C: KeLowerIrql(DISPATCH_LEVEL) — the idle loop is pinned there.
                crate::ke::ke_defs::KeLowerIrql(DISPATCH_LEVEL);
            }
        } else {
            // WFI, never WFE: IRQs are masked here (daifset above). A pending
            // interrupt wakes WFI even while PSTATE.I masks it, and the loop
            // top then unmasks and takes it. A MASKED IRQ is not a WFE wake-up
            // event, and nothing in this kernel issues SEV or enables the
            // timer event stream, so on real hardware WFE here slept forever:
            // the first time a Raspberry Pi 4 went truly idle (~70 s in, when
            // the zero-page thread ran out of pages) the tick stopped and the
            // desktop froze. QEMU's TCG treats WFE as a yield, so no smoke
            // run could ever see it.
            core::arch::asm!("wfi");
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "C" fn KiIdleLoop() -> ! {
    loop {} // nolint: silent-failure (cfg(not(aarch64)) compile-stub — this module only builds on arm64; body never compiled)
}

#[no_mangle]
pub unsafe extern "system" fn KiInitializeHandBuiltThread(
    Thread: PKTHREAD,
    Process: PKPROCESS,
    Stack: PVOID,
) {
    let Thread = opaque_ptr(Thread);
    let Process = opaque_ptr(Process);
    let Stack = opaque_ptr(Stack);
    let Prcb = KeGetCurrentPrcb();

    KeInitializeThread(
        Process,
        Thread,
        None,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Stack,
    );

    (*Thread).__bindgen_anon_1.__bindgen_anon_1.NextProcessor = (*Prcb).Number as UCHAR;
    (*Thread).IdealProcessor = (*Prcb).Number as UCHAR;
    (*Thread).Priority = HIGH_PRIORITY as i8;
    (*Thread).__bindgen_anon_5.__bindgen_anon_6.State = Running as UCHAR;
    (*Thread).Affinity = 1u64 << (*Prcb).Number;
    (*Thread).__bindgen_anon_5.__bindgen_anon_6.WaitIrql = DISPATCH_LEVEL;
    (*(Process as *mut _KPROCESS)).ActiveProcessors |= 1u64 << (*Prcb).Number;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL-2.0-or-later
//  * FILE:            ntoskrnl/ke/arm64/thrdini.c
//  * PURPOSE:         ARM64 Thread Context Setup, Context Switch Resume, and Idle Loop
//  * PROGRAMMERS:     ReactOS ARM64 Port
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FORWARD DECLARATIONS *******************************************************/
//
// extern void KiThreadStartup(void);
// extern void KiInvalidSystemThreadStartupExit(void);
// extern void KiUserThreadStartupExit(void);
//
// /* TYPES **********************************************************************/
//
// /*
//  * Kernel thread initialization frame layout:
//  *   KSWITCH_FRAME   (ApcBypass + Return=KiThreadStartup)
//  *   KSTART_FRAME    (StartRoutine, StartContext, SystemRoutine, Return)
//  *
//  * No KEXCEPTION_FRAME: when a new thread is first switched to,
//  * KiSwapContextInternal pops the KSWITCH_FRAME and returns directly
//  * to KiThreadStartup (via the Return field), bypassing KiSwapContext's
//  * callee-saved register restore.  Same design as amd64.
//  */
// typedef struct _KKINIT_FRAME
// {
//     KSWITCH_FRAME CtxSwitchFrame;
//     KSTART_FRAME StartFrame;
// } KKINIT_FRAME, *PKKINIT_FRAME;
//
// /*
//  * User thread initialization frame layout:
//  *   KSWITCH_FRAME   (ApcBypass + Return=KiThreadStartup)
//  *   KSTART_FRAME    (StartRoutine, StartContext, SystemRoutine, Return=KiUserThreadStartupExit)
//  *   KTRAP_FRAME     (user-mode context populated from CONTEXT via KeContextToTrapFrame)
//  *
//  * KTRAP_FRAME is last (highest address) so KeGetTrapFrame(Thread)
//  * = InitialStack - sizeof(KTRAP_FRAME) points to it correctly.
//  *
//  * When KiThreadStartup returns after SystemRoutine, it pops KSTART_FRAME,
//  * leaving SP at the KTRAP_FRAME, then rets to KiUserThreadStartupExit
//  * which does KiTrapReturn -> eret to EL0.
//  */
// typedef struct _KUINIT_FRAME
// {
//     KSWITCH_FRAME CtxSwitchFrame;   /* 16 bytes */
//     KSTART_FRAME StartFrame;        /* 32 bytes */
//     KTRAP_FRAME TrapFrame;          /* 0x160 bytes */
// } KUINIT_FRAME, *PKUINIT_FRAME;
//
// /* Verify offsets used in ctxswitch.S */
// _Static_assert(__builtin_offsetof(KTHREAD, KernelStack) == 0x38, "ThKernelStack");
// _Static_assert(__builtin_offsetof(KTHREAD, SwapBusy) == 0x95, "ThSwapBusy");
// _Static_assert(__builtin_offsetof(KTHREAD, Teb) == 0xB0,
//     "ThTeb mismatch with ctxswitch.S");
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// KiInitializeContextThread(
//     _In_ PKTHREAD Thread,
//     _In_ PKSYSTEM_ROUTINE SystemRoutine,
//     _In_opt_ PKSTART_ROUTINE StartRoutine,
//     _In_opt_ PVOID StartContext,
//     _In_ PCONTEXT ContextRecord)
// {
//     PVOID InitialStack;
//
//     /*
//      * Reserve space at the top of the stack. On amd64, this holds the
//      * XSAVE area. On ARM64 we don't have XSAVE, but we still need to
//      * reserve space so the init frame doesn't overlap with the caller's
//      * saved registers when initializing the currently-running idle thread.
//      * Use 512 bytes (matching amd64's minimum FXSAVE area size).
//      */
//     InitialStack = (PUCHAR)Thread->InitialStack - 512;
//     InitialStack = (PVOID)((ULONG_PTR)InitialStack & ~(ULONG_PTR)15);
//     Thread->InitialStack = InitialStack;
//
//     if (ContextRecord)
//     {
//         /*
//          * User thread: set up KUINIT_FRAME on the kernel stack.
//          *
//          * Stack layout (low → high address):
//          *   KSWITCH_FRAME  (ApcBypass + Return=KiThreadStartup)
//          *   KSTART_FRAME   (StartRoutine, StartContext, SystemRoutine,
//          *                   Return=KiUserThreadStartupExit)
//          *   KTRAP_FRAME    (user-mode context → eret to EL0)
//          *
//          * KTRAP_FRAME is at the highest address so
//          * KeGetTrapFrame(Thread) = InitialStack - sizeof(KTRAP_FRAME)
//          * points to it correctly.
//          */
//         PKUINIT_FRAME UInitFrame;
//         PKTRAP_FRAME TrapFrame;
//         PKSTART_FRAME StartFrame;
//         PKSWITCH_FRAME CtxSwitchFrame;
//
//         UInitFrame = ((PKUINIT_FRAME)InitialStack) - 1;
//         TrapFrame = &UInitFrame->TrapFrame;
//         StartFrame = &UInitFrame->StartFrame;
//         CtxSwitchFrame = &UInitFrame->CtxSwitchFrame;
//
//         /* Save back the new value of the kernel stack */
//         Thread->KernelStack = (PVOID)UInitFrame;
//
//         /* User thread runs in user mode */
//         Thread->PreviousMode = UserMode;
//
//         /* Zero the trap frame, then populate from context */
//         RtlZeroMemory(TrapFrame, sizeof(KTRAP_FRAME));
//         KeContextToTrapFrame(ContextRecord,
//                              NULL,  /* No exception frame */
//                              TrapFrame,
//                              ContextRecord->ContextFlags,
//                              UserMode);
//
//         /* Force EL0t return (SPSR M[3:0] = 0) */
//         TrapFrame->Spsr = 0;
//         TrapFrame->PreviousMode = UserMode;
//
//         /* Set X18 = TEB so user-mode code has NtCurrentTeb() via x18 */
//         TrapFrame->X18 = (ULONG64)Thread->Teb;
//
//         /* Set up the context switch frame */
//         CtxSwitchFrame->Return = (ULONG64)KiThreadStartup;
//         CtxSwitchFrame->ApcBypass = TRUE;
//
//         /* Set up the start frame with thread parameters */
//         StartFrame->P1Home = (ULONG64)StartRoutine;
//         StartFrame->P2Home = (ULONG64)StartContext;
//         StartFrame->P3Home = (ULONG64)SystemRoutine;
//         StartFrame->Return = (ULONG64)KiUserThreadStartupExit;
//     }
//     else
//     {
//         /* Kernel thread: set up KKINIT_FRAME on the kernel stack */
//         PKKINIT_FRAME InitFrame;
//         PKSTART_FRAME StartFrame;
//         PKSWITCH_FRAME CtxSwitchFrame;
//
//         InitFrame = ((PKKINIT_FRAME)InitialStack) - 1;
//         StartFrame = &InitFrame->StartFrame;
//         CtxSwitchFrame = &InitFrame->CtxSwitchFrame;
//
//         /* Save back the new value of the kernel stack */
//         Thread->KernelStack = (PVOID)InitFrame;
//
//         /* Kernel thread runs in kernel mode */
//         Thread->PreviousMode = KernelMode;
//
//         /*
//          * Set up the context switch frame.
//          * Return = KiThreadStartup: when KiSwapContextInternal pops this
//          * frame and KiSwapContextResume returns, it goes to KiThreadStartup.
//          * ApcBypass = TRUE: skip APC delivery on the first switch.
//          */
//         CtxSwitchFrame->Return = (ULONG64)KiThreadStartup;
//         CtxSwitchFrame->ApcBypass = TRUE;
//
//         /*
//          * Set up the start frame with thread parameters.
//          * KiThreadStartup will read these from SP and call:
//          *   SystemRoutine(StartRoutine, StartContext)
//          */
//         StartFrame->P1Home = (ULONG64)StartRoutine;
//         StartFrame->P2Home = (ULONG64)StartContext;
//         StartFrame->P3Home = (ULONG64)SystemRoutine;
//         StartFrame->Return = (ULONG64)KiInvalidSystemThreadStartupExit;
//     }
// }
//
// BOOLEAN
// KiSwapContextResume(
//     _In_ BOOLEAN ApcBypass,
//     _In_ PKTHREAD OldThread,
//     _In_ PKTHREAD NewThread)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     PKPROCESS OldProcess, NewProcess;
//
//     /* Now we are on the new thread's stack. Check for process switch */
//     OldProcess = OldThread->ApcState.Process;
//     NewProcess = NewThread->ApcState.Process;
//     if (OldProcess != NewProcess)
//     {
//         /* Switch address space (no-op during early boot - all kernel threads
//          * share the same address space) */
//         KiSwapProcess(NewProcess, OldProcess);
//     }
//
//     /* Increase context switch counts */
//     Pcr->Prcb.KeContextSwitches++;
//     NewThread->ContextSwitches++;
//
//     /* DPCs shouldn't be active */
//     if (Pcr->Prcb.DpcRoutineActive)
//     {
//         KeBugCheckEx(ATTEMPTED_SWITCH_FROM_DPC,
//                      (ULONG_PTR)OldThread,
//                      (ULONG_PTR)NewThread,
//                      (ULONG_PTR)OldThread->InitialStack,
//                      0);
//     }
//
//     /* Old thread is no longer busy */
//     OldThread->SwapBusy = FALSE;
//
//     /* Check for pending kernel APCs */
//     if (NewThread->ApcState.KernelApcPending)
//     {
//         if ((NewThread->SpecialApcDisable == 0) && (ApcBypass == 0))
//         {
//             /* Return TRUE to indicate that we want APCs to be delivered */
//             return TRUE;
//         }
//
//         /* Request an APC interrupt to be delivered later */
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
//
//     /* No kernel APCs pending */
//     return FALSE;
// }
//
// DECLSPEC_NORETURN
// VOID
// FASTCALL
// KiIdleLoop(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD OldThread, NewThread;
//
//     /* Now loop forever */
//     while (TRUE)
//     {
//         /* Start of the idle loop: enable then disable interrupts */
//         _enable();
//         YieldProcessor();
//         YieldProcessor();
//         _disable();
//
//         /* Check for pending timers, pending DPCs, or pending ready threads */
//         if ((Prcb->DpcData[0].DpcQueueDepth) ||
//             (Prcb->TimerRequest) ||
//             (Prcb->DeferredReadyListHead.Next))
//         {
//             /* Quiesce the DPC software interrupt */
//             HalClearSoftwareInterrupt(DISPATCH_LEVEL);
//
//             /* Handle it */
//             KiRetireDpcList(Prcb);
//         }
//
//         /* Check if a new thread is scheduled for execution */
//         if (Prcb->NextThread)
//         {
//             /* Enable interrupts */
//             _enable();
//
//             /* Capture current thread data */
//             OldThread = Prcb->CurrentThread;
//             NewThread = Prcb->NextThread;
//
//             /* Set new thread data */
//             Prcb->NextThread = NULL;
//             Prcb->CurrentThread = NewThread;
//
//             /* The thread is now running */
//             NewThread->State = Running;
//
//             /* Switch away from the idle thread */
//             KiSwapContext(APC_LEVEL, OldThread);
//         }
//         else if (Prcb->ReadySummary)
//         {
//             /* ARM64: No timer/software interrupts to preempt idle.
//              * Threads in the ready list would never be picked up because
//              * nothing sets Prcb->NextThread when HalRequestSoftwareInterrupt
//              * is a no-op.  Check the ready list directly. */
//             _enable();
//
//             OldThread = Prcb->CurrentThread;
//             NewThread = KiSelectReadyThread(0, Prcb);
//             if (NewThread)
//             {
//                 Prcb->CurrentThread = NewThread;
//                 NewThread->State = Running;
//                 KiSwapContext(APC_LEVEL, OldThread);
//             }
//         }
//         else
//         {
//             /* ARM64 Wait For Event - low power idle */
//             __asm__ __volatile__("wfe");
//         }
//     }
// }
//
// VOID
// NTAPI
// KiInitializeHandBuiltThread(
//     _In_ PKTHREAD Thread,
//     _In_ PKPROCESS Process,
//     _In_ PVOID Stack)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Setup the Thread */
//     KeInitializeThread(Process, Thread, NULL, NULL, NULL, NULL, NULL, Stack);
//
//     Thread->NextProcessor = Prcb->Number;
//     Thread->IdealProcessor = Prcb->Number;
//     Thread->Priority = HIGH_PRIORITY;
//     Thread->State = Running;
//     Thread->Affinity = (ULONG_PTR)1 << Prcb->Number;
//     Thread->WaitIrql = DISPATCH_LEVEL;
//     Process->ActiveProcessors |= (ULONG_PTR)1 << Prcb->Number;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
