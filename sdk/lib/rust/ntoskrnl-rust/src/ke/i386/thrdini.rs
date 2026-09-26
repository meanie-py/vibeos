use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _CONTEXT, _FX_SAVE_AREA, _KTRAP_FRAME, _KTHREAD, _KPROCESS, _KPRCB, _KIPCR, _KPCR,
    CONTEXT, PCONTEXT, PKTHREAD, PKPROCESS, PKPRCB, PKIPCR, PKPCR, PKTRAP_FRAME,
    PKSYSTEM_ROUTINE, PKSTART_ROUTINE, PULONGLONG,
    HalClearSoftwareInterrupt, HalRequestSoftwareInterrupt,
    KeContextToTrapFrame, KeSetGdtSelector,
    KiAcquirePrcbLock, KiReleasePrcbLock, KiQuantumEnd, KiRetireDpcList, KiSwapContext, KeBugCheck,
    _KWAIT_REASON_WrDispatchInt,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[repr(C)]
struct KSWITCHFRAME {
    ExceptionList: PVOID,
    ApcBypassDisable: BOOLEAN,
    RetAddr: PVOID,
}

#[repr(C)]
struct KSTART_FRAME {
    SystemRoutine: PKSYSTEM_ROUTINE,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
    UserThread: BOOLEAN,
}

#[repr(C)]
struct KUINIT_FRAME {
    CtxSwitchFrame: KSWITCHFRAME,
    StartFrame: KSTART_FRAME,
    TrapFrame: _KTRAP_FRAME,
    FxSaveArea: _FX_SAVE_AREA,
}

#[repr(C)]
struct KKINIT_FRAME {
    CtxSwitchFrame: KSWITCHFRAME,
    StartFrame: KSTART_FRAME,
    FxSaveArea: _FX_SAVE_AREA,
}

#[repr(C)]
struct FXSAVE_FORMAT {
    ControlWord: u16,
    StatusWord: u16,
    TagWord: u16,
    ErrorOpcode: u16,
    ErrorOffset: u32,
    ErrorSelector: u32,
    DataOffset: u32,
    DataSelector: u32,
    MXCsr: u32,
    Reserved2: u32,
    RegisterArea: [u8; 128],
    Reserved3: [u8; 128],
    Reserved4: [u8; 224],
}

extern "system" {
    static mut KeI386FxsrPresent: BOOLEAN;
    // This file imports `_FX_SAVE_AREA` but not the PFX_SAVE_AREA alias.
    fn Ke386SaveFpuState(SaveArea: *mut _FX_SAVE_AREA);
    fn _enable();
    fn _disable();
    fn Ke386SetLocalDescriptorTable(selector: u16);
    fn Ke386SetGs(selector: u16);
    fn KiSetTebBase(pcr: PKPCR, teb: PVOID);
    fn KiGetThreadNpxArea(thread: PKTHREAD) -> *mut _FX_SAVE_AREA;
}
extern "C" {
    fn KeGetTrapFrame(thread: PKTHREAD) -> PKTRAP_FRAME;
}
extern "C" {
    fn YieldProcessor();
    fn __readcr0() -> ULONG_PTR;
    fn __writecr0(value: ULONG_PTR);
    fn __readcr3() -> ULONG_PTR;
    fn __writecr3(value: ULONG_PTR);
    fn KxQueueReadyThread(thread: PKTHREAD, prcb: PKPRCB);
}
extern "fastcall" {
    fn KiServiceExit2(TrapFrame: PKTRAP_FRAME);
}

/// Context-switch return target installed by `KiInitializeContextThread`.
/// The thread's switch frame at `KernelStack` puts the thread on a
/// `KUINIT_FRAME` whose `StartFrame.SystemRoutine` is invoked once the
/// trampoline lowers IRQL to APC_LEVEL. `KiServiceExit2` is the stdcall
/// asm tail at `i386/trap.s` that returns to user-mode if the thread is
/// a user thread; non-user threads bugcheck.
#[no_mangle]
pub unsafe extern "system" fn KiThreadStartup() {
    let init_frame =
        (*(ntkrnl_sys::mm::KeGetCurrentThread() as PKTHREAD)).KernelStack as *mut KUINIT_FRAME;
    let start_frame = core::ptr::addr_of_mut!((*init_frame).StartFrame);
    let trap_frame = core::ptr::addr_of_mut!((*init_frame).TrapFrame);

    KeLowerIrql(APC_LEVEL);

    let routine = (*start_frame).SystemRoutine.unwrap_unchecked();
    routine((*start_frame).StartRoutine, (*start_frame).StartContext);

    if (*start_frame).UserThread == 0 {
        KeBugCheck(NO_USER_MODE_CONTEXT);
    }

    KiServiceExit2(trap_frame);
}

extern "fastcall" {
    fn KiSwitchThreads(
        OldThread: PKTHREAD,
        NewThread: PKTHREAD,
    );
    fn KiRetireDpcListInDpcStack(
        Prcb: PKPRCB,
        DpcStack: PVOID,
    );
}

const KTRAP_FRAME_LENGTH: usize = core::mem::size_of::<_KTRAP_FRAME>();
// NPX_FRAME_LENGTH == sizeof(FX_SAVE_AREA) per C_ASSERT in ke.h
const NPX_FRAME_LENGTH: usize = core::mem::size_of::<_FX_SAVE_AREA>();
const RPL_MASK: u32 = 3;
const EXCEPTION_CHAIN_END: PVOID = (-1isize) as PVOID;
const USER_MODE: i8 = 1;
const NPX_STATE_NOT_LOADED: u8 = 0xA; // ndk/i386/asm.h
const NPX_STATE_LOADED: u8 = 0x0; // ndk/i386/asm.h
const KGDT_LDT: u32 = 0x48;
const CR0_MP: ULONG_PTR = 0x00000002;
const CR0_EM: ULONG_PTR = 0x00000004;
const CR0_TS: ULONG_PTR = 0x00000008;
const NO_USER_MODE_CONTEXT: u32 = 0x0000000E;
const ATTEMPTED_SWITCH_FROM_DPC: u32 = 0x000000B8;
const Running: u8 = 2;
const WrDispatchInt: u8 = 0x0000001F;
const EFLAGS_V86_MASK: u32 = 0x00020000;

#[no_mangle]
pub unsafe extern "system" fn KiInitializeContextThread(
    Thread: PKTHREAD,
    SystemRoutine: PKSYSTEM_ROUTINE,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
    ContextPointer: PCONTEXT,
) {
    let Thread = opaque_ptr(Thread);
    let StartContext = opaque_ptr(StartContext);
    let mut FxSaveArea: *mut _FX_SAVE_AREA;
    let mut StartFrame: *mut KSTART_FRAME;
    let mut CtxSwitchFrame: *mut KSWITCHFRAME;
    let mut TrapFrame: PKTRAP_FRAME;
    let mut LocalContext: _CONTEXT = core::mem::zeroed();
    let mut Context: PCONTEXT = core::ptr::null_mut();
    let mut ContextFlags: ULONG;

    if !ContextPointer.is_null() {
        let InitFrame = (((*Thread).InitialStack as ULONG_PTR) - core::mem::size_of::<KUINIT_FRAME>()) as *mut KUINIT_FRAME;

        core::ptr::copy_nonoverlapping(
            ContextPointer as *const u8,
            core::ptr::addr_of_mut!(LocalContext) as *mut u8,
            core::mem::size_of::<_CONTEXT>(),
        );
        Context = core::ptr::addr_of_mut!(LocalContext);
        ContextFlags = 0x00010001u32;

        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*InitFrame).TrapFrame) as *mut u8,
            0,
            KTRAP_FRAME_LENGTH + core::mem::size_of::<_FX_SAVE_AREA>(),
        );

        FxSaveArea = core::ptr::addr_of_mut!((*InitFrame).FxSaveArea);

        if KeI386FxsrPresent != 0 {
            let FxSaveFormat = (*Context).ExtendedRegisters.as_mut_ptr() as *mut FXSAVE_FORMAT;

            (*FxSaveFormat).ControlWord = 0x27F;
            (*FxSaveFormat).StatusWord = 0;
            (*FxSaveFormat).TagWord = 0;
            (*FxSaveFormat).ErrorOffset = 0;
            (*FxSaveFormat).ErrorSelector = 0;
            (*FxSaveFormat).DataOffset = 0;
            (*FxSaveFormat).DataSelector = 0;
            (*FxSaveFormat).MXCsr = 0x1F80;
        } else {
            (*Context).FloatSave.ControlWord = 0x27F;
            (*Context).FloatSave.StatusWord = 0;
            (*Context).FloatSave.TagWord = 0xFFFFFFFFu32;
            (*Context).FloatSave.ErrorOffset = 0;
            (*Context).FloatSave.ErrorSelector = 0;
            (*Context).FloatSave.DataOffset = 0;
            (*Context).FloatSave.DataSelector = 0;
        }

        (*Context).FloatSave.Cr0NpxState = 0;
        (*FxSaveArea).Cr0NpxState = 0;
        (*FxSaveArea).NpxSavedCpu = 0;

        // CONTEXT_EXTENDED_REGISTERS (0x20) when FXSR, else CONTEXT_FLOATING_POINT (0x08)
        ContextFlags |= if KeI386FxsrPresent != 0 { 0x00000020u32 } else { 0x00000008u32 };

        (*Thread).NpxState = NPX_STATE_NOT_LOADED;
        (*Thread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql = PASSIVE_LEVEL;

        // C: `ContextFlags &= ~CONTEXT_DEBUG_REGISTERS`, i.e. ~(CONTEXT_i386|0x10)
        // = ~0x10010 — it also strips the ARCH bit. Deliberately not copied: the
        // only consumer is the `Context->ContextFlags | ContextFlags` argument
        // below, and the local `ContextFlags` re-adds CONTEXT_i386 either way, so
        // the two differ solely in what the caller's CONTEXT record retains, which
        // KeInitThread never reads. Clearing 0x10 alone leaves the record valid.
        (*Context).ContextFlags &= !0x00000010u32; // nolint: mask-drift (audited; see above)

        TrapFrame = core::ptr::addr_of_mut!((*InitFrame).TrapFrame);

        KeContextToTrapFrame(
            Context,
            core::ptr::null_mut(),
            TrapFrame,
            (*Context).ContextFlags | ContextFlags,
            1i8,
        );

        (*TrapFrame).HardwareSegSs |= RPL_MASK;
        (*TrapFrame).SegDs |= RPL_MASK;
        (*TrapFrame).SegEs |= RPL_MASK;
        (*TrapFrame).Dr7 = 0;

        (*TrapFrame).DbgArgMark = 0xBADB0D00;

        (*TrapFrame).PreviousPreviousMode = 1i32 as ULONG;

        (*TrapFrame).ExceptionList = EXCEPTION_CHAIN_END as *mut ntkrnl_sys::ddk::_EXCEPTION_REGISTRATION_RECORD;

        StartFrame = core::ptr::addr_of_mut!((*InitFrame).StartFrame);
        CtxSwitchFrame = core::ptr::addr_of_mut!((*InitFrame).CtxSwitchFrame);

        (*Thread).__bindgen_anon_7.__bindgen_anon_2.PreviousMode = 1i32 as CCHAR;

        (*StartFrame).UserThread = 1u8;
    } else {
        let InitFrame = (((*Thread).InitialStack as ULONG_PTR) - core::mem::size_of::<KKINIT_FRAME>()) as *mut KKINIT_FRAME;

        FxSaveArea = core::ptr::addr_of_mut!((*InitFrame).FxSaveArea);
        core::ptr::write_bytes(FxSaveArea as *mut u8, 0, core::mem::size_of::<_FX_SAVE_AREA>());

        if KeI386FxsrPresent != 0 {
            (*FxSaveArea).U.FxArea.ControlWord = 0x27F;
            (*FxSaveArea).U.FxArea.MXCsr = 0x1F80;
        } else {
            (*FxSaveArea).U.FnArea.ControlWord = 0x27F;
            (*FxSaveArea).U.FnArea.TagWord = 0xFFFFFFFFu32;
        }

        (*Thread).NpxState = NPX_STATE_NOT_LOADED;

        StartFrame = core::ptr::addr_of_mut!((*InitFrame).StartFrame);
        CtxSwitchFrame = core::ptr::addr_of_mut!((*InitFrame).CtxSwitchFrame);

        (*Thread).__bindgen_anon_7.__bindgen_anon_2.PreviousMode = 0i32 as CCHAR;

        (*StartFrame).UserThread = 0u8;
    }

    (*StartFrame).StartContext = StartContext;
    (*StartFrame).StartRoutine = StartRoutine;
    (*StartFrame).SystemRoutine = SystemRoutine;

    (*CtxSwitchFrame).RetAddr = KiThreadStartup as PVOID;
    (*CtxSwitchFrame).ApcBypassDisable = 1u8;
    (*CtxSwitchFrame).ExceptionList = EXCEPTION_CHAIN_END;

    (*Thread).KernelStack = CtxSwitchFrame as PVOID;

    // cswitchdep birth brand: pool-reused KTHREAD addresses must never
    // inherit a dead thread's shadow (see amd64/thrdini.rs).
    #[cfg(feature = "cswitchdep")]
    crate::ke::cswitchdep::cswitch_thread_init(Thread as usize);
}

#[no_mangle]
pub unsafe extern "C" fn KiIdleLoop() -> ! {
    let Prcb = KeGetCurrentPrcb();
    let mut OldThread: PKTHREAD;
    let mut NewThread: PKTHREAD;

    loop {
        _enable();
        YieldProcessor();
        YieldProcessor();
        _disable();

        if ((*Prcb).DpcData[0].DpcQueueDepth != 0)
            || ((*Prcb).TimerRequest != 0)
            || (!(*Prcb).DeferredReadyListHead.Next.is_null())
        {
            HalClearSoftwareInterrupt(DISPATCH_LEVEL);
            KiRetireDpcList(Prcb);
        }

        if !(*Prcb).NextThread.is_null() {
            // UP keeps the C shape: sti before the (lock-free) consume.
            // SMP MUST NOT (§9.56 — mirror amd64 stubs.rs KiIdleLoop): consume
            // NextThread UNDER THE PRCB LOCK, WITH INTERRUPTS DISABLED, re-check,
            // promote, rq_inflight_unmark, release, THEN sti. The idle thread is
            // never queued ready here (no KxQueueReadyThread — that's what
            // releases the lock in the KiDispatchInterrupt path below), so the
            // lock MUST be released explicitly before the swap. A lock-free
            // consume let KiDeferredReadyThread's standby-DISPLACEMENT (which
            // runs under this lock and re-defers a NextThread it reads) race the
            // idle promote → a RUNNING thread set DeferredReady → double-link
            // tear. This path was inert until i386 Phase C (AP now schedules).
            #[cfg(not(feature = "smp"))]
            _enable();

            #[cfg(feature = "smp")]
            KiAcquirePrcbLock(Prcb);
            #[cfg(feature = "smp")]
            if (*Prcb).NextThread.is_null() {
                KiReleasePrcbLock(Prcb);
                continue;
            }

            OldThread = (*Prcb).CurrentThread;
            NewThread = (*Prcb).NextThread;

            (*Prcb).NextThread = core::ptr::null_mut();
            (*Prcb).CurrentThread = NewThread;

            (*NewThread).State = Running;

            // i386 Phase C visibility: one-shot per-CPU "scheduling live" marker,
            // fired when a non-boot CPU promotes its first real thread off idle.
            #[cfg(feature = "smp")]
            {
                use core::sync::atomic::{AtomicBool, Ordering};
                static CPU_SCHED_LIVE: [AtomicBool; 32] = {
                    const F: AtomicBool = AtomicBool::new(false);
                    [F; 32]
                };
                let n = (*Prcb).Number as usize;
                if n != 0 && n < 32 && !CPU_SCHED_LIVE[n].swap(true, Ordering::Relaxed) {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"[SMP] CPU%u scheduling live\n\0".as_ptr() as *const i8,
                        n as u32,
                    );
                }
            }

            #[cfg(feature = "smp")]
            ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread as *mut _KTHREAD);
            #[cfg(feature = "smp")]
            KiReleasePrcbLock(Prcb);
            #[cfg(feature = "smp")]
            _enable();

            #[cfg(feature = "smp")]
            KfRaiseIrql(crate::ke::ke_defs::SYNCH_LEVEL); // C: KfRaiseIrql(SYNCH_LEVEL) — swap at SYNCH (27 on i386 SMP)

            KiSwapContext(APC_LEVEL, OldThread);

            #[cfg(feature = "smp")]
            KeLowerIrql(DISPATCH_LEVEL);
        } else {
            // NOTE: no KiTbLazyIdleEnter/Exit on i386 — lazy-TLB idle coherence
            // is the separate i386 TLB-shootdown parity item, not required for
            // "AP schedules". Remote shootdowns are not yet deferred on i386.
            // None (never in practice — PoInitializePrcb always sets it) falls
            // through to re-poll the queues rather than panicking in the idle loop.
            if let Some(idle) = (*Prcb).PowerState.IdleFunction {
                idle(core::ptr::addr_of_mut!((*Prcb).PowerState));
            }
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSwapContextExit(
    OldThread: PKTHREAD,
    SwitchFrame: *mut KSWITCHFRAME,
) -> BOOLEAN {
    let OldThread = opaque_ptr(OldThread);
    let SwitchFrame = opaque_ptr(SwitchFrame);
    let Pcr = KeGetPcr() as PKIPCR;
    let OldProcess: PKPROCESS;
    let NewProcess: PKPROCESS;
    let NewThread: PKTHREAD;

    NewThread = (*Pcr).PrcbData.CurrentThread;

    OldProcess = (*OldThread).__bindgen_anon_1.ApcState.Process as PKPROCESS;
    NewProcess = (*NewThread).__bindgen_anon_1.ApcState.Process as PKPROCESS;
    if OldProcess != NewProcess {
        if *(core::ptr::addr_of!((*OldProcess).LdtDescriptor) as PULONGLONG)
            != *(core::ptr::addr_of!((*NewProcess).LdtDescriptor) as PULONGLONG)
        {
            if (*NewProcess).LdtDescriptor.LimitLow != 0 {
                KeSetGdtSelector(
                    KGDT_LDT,
                    (core::ptr::addr_of!((*NewProcess).LdtDescriptor) as *const ULONG).read(),
                    (core::ptr::addr_of!((*NewProcess).LdtDescriptor) as *const ULONG).add(1).read(),
                );
                Ke386SetLocalDescriptorTable(KGDT_LDT as u16);
            } else {
                Ke386SetLocalDescriptorTable(0);
            }
        }

        __writecr3((*NewProcess).DirectoryTableBase[0] as ULONG_PTR);
    }

    Ke386SetGs(0);

    KiSetTebBase(
        Pcr as PKPCR,
        core::ptr::addr_of_mut!((*(*NewThread).Teb).NtTib) as PVOID,
    );

    (*(*Pcr).TSS).Esp0 = (*NewThread).InitialStack as ULONG;

    let trap_frame = KeGetTrapFrame(NewThread);
    if ((*trap_frame).EFlags & EFLAGS_V86_MASK) == 0 {
        // C: Esp0 -= sizeof(KTRAP_FRAME) - FIELD_OFFSET(KTRAP_FRAME, V86Es) (= 0x10).
        // Was a hand-written `- 0x50` (V86Es is at 0x7c!), leaving Esp0 0x2c too low so
        // trap frames no longer sat adjacent to the NPX area (TrapFrame+1 != FX base),
        // shifting every KeContextToTrapFrame/KeTrapFrameToContext FP-state access.
        (*(*Pcr).TSS).Esp0 -=
            (KTRAP_FRAME_LENGTH - core::mem::offset_of!(_KTRAP_FRAME, V86Es)) as ULONG;
    }

    (*(*Pcr).TSS).Esp0 -= NPX_FRAME_LENGTH as ULONG;
    (*(*Pcr).TSS).IoMapBase = (*NewProcess).IopmOffset;

    (*NewThread).ContextSwitches += 1;

    (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = (*SwitchFrame).ExceptionList as *mut ntkrnl_sys::ddk::_EXCEPTION_REGISTRATION_RECORD;

    if (*Pcr).PrcbData.DpcRoutineActive != 0 {
        KeBugCheckEx(
            ATTEMPTED_SWITCH_FROM_DPC,
            OldThread as ULONG_PTR,
            NewThread as ULONG_PTR,
            (*OldThread).InitialStack as ULONG_PTR,
            0,
        );
    }

    // We're on the NEW stack with the old context fully saved: release the
    // old thread for other CPUs (pairs with the Acquire spin in
    // KiSwapContextEntry).  Last OldThread access in this function — after
    // this store the thread may run, exit, and be freed on another CPU.
    core::sync::atomic::AtomicU8::from_ptr(core::ptr::addr_of_mut!((*OldThread).SwapBusy))
        .store(0, core::sync::atomic::Ordering::Release);

    if (*NewThread).__bindgen_anon_1.ApcState.KernelApcPending != 0 {
        if (*NewThread).__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable == 0 {
            if (*SwitchFrame).ApcBypassDisable != 0 {
                HalRequestSoftwareInterrupt(APC_LEVEL);
            } else {
                return 1u8;
            }
        }
    }

    0u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSwapContextEntry(
    SwitchFrame: *mut KSWITCHFRAME,
    OldThreadAndApcFlag: ULONG_PTR,
) {
    let SwitchFrame = opaque_ptr(SwitchFrame);
    let Pcr = KeGetPcr() as PKIPCR;
    let OldThread: PKTHREAD;
    let NewThread: PKTHREAD;
    let Cr0: ULONG_PTR;
    let NewCr0: ULONG_PTR;

    (*SwitchFrame).ApcBypassDisable = (OldThreadAndApcFlag & 3) as BOOLEAN;
    (*SwitchFrame).ExceptionList = (*Pcr).__bindgen_anon_1.NtTib.ExceptionList as PVOID;

    (*Pcr).__bindgen_anon_1.__bindgen_anon_1.ContextSwitches += 1;
    if (*Pcr).__bindgen_anon_1.__bindgen_anon_1.PerfGlobalGroupMask != core::ptr::null_mut() {
    }

    OldThread = (OldThreadAndApcFlag & !3) as PKTHREAD;
    NewThread = (*Pcr).PrcbData.CurrentThread;

    (*OldThread).KernelStack = SwitchFrame as PVOID;

    // NB: an earlier translation cleared (*NewThread).SwapBusy here — the
    // INVERSE of the NT MP protocol (2k3 i386 SwapContext): the switching CPU
    // must (a) SPIN until the new thread's SwapBusy clears before adopting its
    // stack (see the spin below), and (b) clear the OLD thread's SwapBusy only
    // after ESP has left the old stack (KiSwapContextExit).  Clearing the new
    // thread's flag here both destroyed the guard for OldThread (never cleared
    // → lazily self-healed here) and let another CPU adopt a still-in-use
    // stack.  That was the -smp 2 0x1E @ KiSwapContextExit+0x20: CPU1 resumed
    // the migrating thread while CPU0 still ran on its stack, CPU1's pushes
    // scribbled CPU0's live locals, and CPU0's spilled OldThread became a
    // stack VA (0xf7a29c40) → IN_PAGE_ERROR on OldThread->ApcState.Process.

    _disable();

    Cr0 = __readcr0();

    // Flush this processor's NPX ownership before the switch. The outgoing thread's
    // FPU state lives in THESE registers, so on SMP it has to be written back to the
    // thread's save area and local ownership dropped, or the thread resumes on another
    // processor with its FPU state still sitting in the one it left -- silent numeric
    // corruption, not a fault, which is why it survived so long.
    //
    // Ported from upstream 091855fc4f9 ("[NTOS:KE] Flush i386 NPX ownership before SMP
    // switches", #9548), which is a BUG ORACLE hit: we translated thrdini.c before that
    // fix existed, so our Rust carried the defect verbatim. Same sequence as the
    // lazy-restore path in traphdlr.rs -- clear MP/EM/TS so the save cannot trap, save,
    // then mark not-loaded.
    #[cfg(feature = "smp")]
    {
        let NpxThread = (*Pcr).PrcbData.NpxThread;
        if !NpxThread.is_null() {
            // Upstream ASSERTs NpxThread == OldThread here. Only the thread that just
            // ran can own this processor's FPU, so a mismatch means the ownership
            // bookkeeping is already wrong and saving would write the wrong thread's
            // registers into the wrong save area.
            if NpxThread == OldThread && (*NpxThread).NpxState == NPX_STATE_LOADED {
                __writecr0(Cr0 & !(CR0_MP | CR0_EM | CR0_TS));

                let NpxSaveArea = KiGetThreadNpxArea(NpxThread);
                Ke386SaveFpuState(NpxSaveArea);
                (*NpxSaveArea).NpxSavedCpu = 0;
                (*NpxThread).NpxState = NPX_STATE_NOT_LOADED;
            }

            (*Pcr).PrcbData.NpxThread = core::ptr::null_mut();
        }
    }

    NewCr0 = ((*NewThread).NpxState as ULONG_PTR)
        | (Cr0 & !(CR0_MP | CR0_EM | CR0_TS))
        | ((*KiGetThreadNpxArea(NewThread)).Cr0NpxState as ULONG_PTR);
    if Cr0 != NewCr0 {
        __writecr0(NewCr0);
    }

    _enable();

    // NT MP protocol (2k3 i386 SwapContext / our amd64 ctxswitch.S:179): spin
    // until the new thread's context is FULLY saved by whichever CPU last ran
    // it, before adopting its stack.  Acquire pairs with the Release clear in
    // KiSwapContextExit; KernelStack is read only after the spin completes.
    // Interrupts are enabled here so IPIs (TLB shootdown) stay serviceable.
    #[cfg(feature = "smp")]
    {
        let busy = core::sync::atomic::AtomicU8::from_ptr(
            core::ptr::addr_of_mut!((*NewThread).SwapBusy),
        );
        while busy.load(core::sync::atomic::Ordering::Acquire) != 0 {
            core::hint::spin_loop();
        }
    }

    KiSwitchThreads(OldThread, (*NewThread).KernelStack as PKTHREAD);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KiDispatchInterrupt() {
    let Pcr = KeGetPcr() as PKIPCR;
    let Prcb = core::ptr::addr_of_mut!((*Pcr).PrcbData);
    let mut OldHandler: PVOID;
    let mut NewThread: PKTHREAD;
    let mut OldThread: PKTHREAD;

    // cswitchdep tap 3: deferred dispatch must be entered at DISPATCH_LEVEL.
    #[cfg(feature = "cswitchdep")]
    crate::ke::cswitchdep::cswitch_dpc_dispatch(crate::code_addr!());

    _disable();

    if ((*Prcb).DpcData[0].DpcQueueDepth != 0)
        || ((*Prcb).TimerRequest != 0)
        || (!(*Prcb).DeferredReadyListHead.Next.is_null())
    {
        OldHandler = (*Pcr).__bindgen_anon_1.NtTib.ExceptionList as PVOID;
        (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = EXCEPTION_CHAIN_END as *mut ntkrnl_sys::ddk::_EXCEPTION_REGISTRATION_RECORD;

        KiRetireDpcListInDpcStack(Prcb, (*Prcb).DpcStack);

        (*Pcr).__bindgen_anon_1.NtTib.ExceptionList = OldHandler as *mut ntkrnl_sys::ddk::_EXCEPTION_REGISTRATION_RECORD;
    }

    _enable();

    if (*Prcb).QuantumEnd != 0 {
        (*Prcb).QuantumEnd = 0u8;
        KiQuantumEnd();
    } else if !(*Prcb).NextThread.is_null() {
        KiAcquirePrcbLock(Prcb);

        OldThread = (*Prcb).CurrentThread;
        NewThread = (*Prcb).NextThread;

        (*Prcb).NextThread = core::ptr::null_mut();
        (*Prcb).CurrentThread = NewThread;

        (*NewThread).State = Running;

        // §9.56 in-flight oracle: promotion-to-Running unmark. This standby
        // promote was the ONE i386 site missing it — every deferred-ready
        // thread promoted here kept its mark, and each later wake spammed
        // [RQDEFER2] DOUBLE-DEFER with a stale_by naming KiReadyThread
        // (the i386 -smp 2 serial-spam face, 2026-07-18). amd64 twin:
        // ke/amd64/stubs.rs idle-loop promote.
        #[cfg(feature = "smp")]
        ntkrnl_sys::inline::ke::rq_inflight_unmark(NewThread as *mut _KTHREAD);

        (*OldThread).WaitReason = _KWAIT_REASON_WrDispatchInt as UCHAR;

        // NT MP (2k3 KiDispatchInterrupt): mark the outgoing thread
        // context-swap-busy BEFORE it becomes visible in a ready queue —
        // another CPU that picks it must spin in KiSwapContextEntry until our
        // KiSwapContextExit clears it (we're still on its stack until then).
        ntkrnl_sys::inline::ke::KiSetThreadSwapBusy(OldThread as *mut _KTHREAD);

        KxQueueReadyThread(OldThread, Prcb);

        KiSwapContext(APC_LEVEL, OldThread);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ke/i386/thrdini.c
//  * PURPOSE:         i386 Thread Context Creation
//  * PROGRAMMER:      Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _KSWITCHFRAME
// {
//     PVOID ExceptionList;
//     BOOLEAN ApcBypassDisable;
//     PVOID RetAddr;
// } KSWITCHFRAME, *PKSWITCHFRAME;
//
// typedef struct _KSTART_FRAME
// {
//     PKSYSTEM_ROUTINE SystemRoutine;
//     PKSTART_ROUTINE StartRoutine;
//     PVOID StartContext;
//     BOOLEAN UserThread;
// } KSTART_FRAME, *PKSTART_FRAME;
//
// typedef struct _KUINIT_FRAME
// {
//     KSWITCHFRAME CtxSwitchFrame;
//     KSTART_FRAME StartFrame;
//     KTRAP_FRAME TrapFrame;
//     FX_SAVE_AREA FxSaveArea;
// } KUINIT_FRAME, *PKUINIT_FRAME;
//
// typedef struct _KKINIT_FRAME
// {
//     KSWITCHFRAME CtxSwitchFrame;
//     KSTART_FRAME StartFrame;
//     FX_SAVE_AREA FxSaveArea;
// } KKINIT_FRAME, *PKKINIT_FRAME;
//
// VOID
// FASTCALL
// KiSwitchThreads(
//     IN PKTHREAD OldThread,
//     IN PKTHREAD NewThread
// );
//
// VOID
// FASTCALL
// KiRetireDpcListInDpcStack(
//     IN PKPRCB Prcb,
//     IN PVOID DpcStack
// );
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KiThreadStartup(VOID)
// {
//     PKTRAP_FRAME TrapFrame;
//     PKSTART_FRAME StartFrame;
//     PKUINIT_FRAME InitFrame;
//
//     /* Get the start and trap frames */
//     InitFrame = KeGetCurrentThread()->KernelStack;
//     StartFrame = &InitFrame->StartFrame;
//     TrapFrame = &InitFrame->TrapFrame;
//
//     /* Lower to APC level */
//     KfLowerIrql(APC_LEVEL);
//
//     /* Call the system routine */
//     StartFrame->SystemRoutine(StartFrame->StartRoutine, StartFrame->StartContext);
//
//     /* If we returned, we better be a user thread */
//     if (!StartFrame->UserThread)
//     {
//         KeBugCheck(NO_USER_MODE_CONTEXT);
//     }
//
//     /* Exit to user-mode */
//     KiServiceExit2(TrapFrame);
// }
//
// VOID
// NTAPI
// KiInitializeContextThread(IN PKTHREAD Thread,
//                           IN PKSYSTEM_ROUTINE SystemRoutine,
//                           IN PKSTART_ROUTINE StartRoutine,
//                           IN PVOID StartContext,
//                           IN PCONTEXT ContextPointer)
// {
//     PFX_SAVE_AREA FxSaveArea;
//     PFXSAVE_FORMAT FxSaveFormat;
//     PKSTART_FRAME StartFrame;
//     PKSWITCHFRAME CtxSwitchFrame;
//     PKTRAP_FRAME TrapFrame;
//     CONTEXT LocalContext;
//     PCONTEXT Context = NULL;
//     ULONG ContextFlags;
//
//     /* Check if this is a With-Context Thread */
//     if (ContextPointer)
//     {
//         /* Set up the Initial Frame */
//         PKUINIT_FRAME InitFrame;
//         InitFrame = (PKUINIT_FRAME)((ULONG_PTR)Thread->InitialStack -
//                                     sizeof(KUINIT_FRAME));
//
//         /* Copy over the context we got */
//         RtlCopyMemory(&LocalContext, ContextPointer, sizeof(CONTEXT));
//         Context = &LocalContext;
//         ContextFlags = CONTEXT_CONTROL;
//
//         /* Zero out the trap frame and save area */
//         RtlZeroMemory(&InitFrame->TrapFrame,
//                       KTRAP_FRAME_LENGTH + sizeof(FX_SAVE_AREA));
//
//         /* Setup the Fx Area */
//         FxSaveArea = &InitFrame->FxSaveArea;
//
//         /* Check if we support FXsr */
//         if (KeI386FxsrPresent)
//         {
//             /* Get the FX Save Format Area */
//             FxSaveFormat = (PFXSAVE_FORMAT)Context->ExtendedRegisters;
//
//             /* Set an initial state */
//             FxSaveFormat->ControlWord = 0x27F;
//             FxSaveFormat->StatusWord = 0;
//             FxSaveFormat->TagWord = 0;
//             FxSaveFormat->ErrorOffset = 0;
//             FxSaveFormat->ErrorSelector = 0;
//             FxSaveFormat->DataOffset = 0;
//             FxSaveFormat->DataSelector = 0;
//             FxSaveFormat->MXCsr = 0x1F80;
//         }
//         else
//         {
//             /* Setup the regular save area */
//             Context->FloatSave.ControlWord = 0x27F;
//             Context->FloatSave.StatusWord = 0;
//             Context->FloatSave.TagWord = -1;
//             Context->FloatSave.ErrorOffset = 0;
//             Context->FloatSave.ErrorSelector = 0;
//             Context->FloatSave.DataOffset =0;
//             Context->FloatSave.DataSelector = 0;
//         }
//
//         /* Set an intial NPX State */
//         Context->FloatSave.Cr0NpxState = 0;
//         FxSaveArea->Cr0NpxState = 0;
//         FxSaveArea->NpxSavedCpu = 0;
//
//         /* Now set the context flags depending on XMM support */
//         ContextFlags |= (KeI386FxsrPresent) ? CONTEXT_EXTENDED_REGISTERS : CONTEXT_FLOATING_POINT;
//
//         /* Set the Thread's NPX State */
//         Thread->NpxState = NPX_STATE_NOT_LOADED;
//         Thread->Header.NpxIrql = PASSIVE_LEVEL;
//
//         /* Disable any debug registers */
//         Context->ContextFlags &= ~CONTEXT_DEBUG_REGISTERS;
//
//         /* Setup the Trap Frame */
//         TrapFrame = &InitFrame->TrapFrame;
//
//         /* Set up a trap frame from the context. */
//         KeContextToTrapFrame(Context,
//                              NULL,
//                              TrapFrame,
//                              Context->ContextFlags | ContextFlags,
//                              UserMode);
//
//         /* Set SS, DS, ES's RPL Mask properly */
//         TrapFrame->HardwareSegSs |= RPL_MASK;
//         TrapFrame->SegDs |= RPL_MASK;
//         TrapFrame->SegEs |= RPL_MASK;
//         TrapFrame->Dr7 = 0;
//
//         /* Set the debug mark */
//         TrapFrame->DbgArgMark = 0xBADB0D00;
//
//         /* Set the previous mode as user */
//         TrapFrame->PreviousPreviousMode = UserMode;
//
//         /* Terminate the Exception Handler List */
//         TrapFrame->ExceptionList = EXCEPTION_CHAIN_END;
//
//         /* Setup the Stack for KiThreadStartup and Context Switching */
//         StartFrame = &InitFrame->StartFrame;
//         CtxSwitchFrame = &InitFrame->CtxSwitchFrame;
//
//         /* Tell the thread it will run in User Mode */
//         Thread->PreviousMode = UserMode;
//
//         /* Tell KiThreadStartup of that too */
//         StartFrame->UserThread = TRUE;
//     }
//     else
//     {
//         /* Set up the Initial Frame for the system thread */
//         PKKINIT_FRAME InitFrame;
//         InitFrame = (PKKINIT_FRAME)((ULONG_PTR)Thread->InitialStack -
//                                     sizeof(KKINIT_FRAME));
//
//         /* Setup the Fx Area */
//         FxSaveArea = &InitFrame->FxSaveArea;
//         RtlZeroMemory(FxSaveArea, sizeof(FX_SAVE_AREA));
//
//         /* Check if we have Fxsr support */
//         if (KeI386FxsrPresent)
//         {
//             /* Set the stub FX area */
//             FxSaveArea->U.FxArea.ControlWord = 0x27F;
//             FxSaveArea->U.FxArea.MXCsr = 0x1F80;
//         }
//         else
//         {
//             /* Set the stub FN area */
//             FxSaveArea->U.FnArea.ControlWord = 0x27F;
//             FxSaveArea->U.FnArea.TagWord = -1;
//         }
//
//         /* No NPX State */
//         Thread->NpxState = NPX_STATE_NOT_LOADED;
//
//         /* Setup the Stack for KiThreadStartup and Context Switching */
//         StartFrame = &InitFrame->StartFrame;
//         CtxSwitchFrame = &InitFrame->CtxSwitchFrame;
//
//         /* Tell the thread it will run in Kernel Mode */
//         Thread->PreviousMode = KernelMode;
//
//         /* Tell KiThreadStartup of that too */
//         StartFrame->UserThread = FALSE;
//     }
//
//     /* Now setup the remaining data for KiThreadStartup */
//     StartFrame->StartContext = StartContext;
//     StartFrame->StartRoutine = StartRoutine;
//     StartFrame->SystemRoutine = SystemRoutine;
//
//     /* And set up the Context Switch Frame */
//     CtxSwitchFrame->RetAddr = KiThreadStartup;
//     CtxSwitchFrame->ApcBypassDisable = TRUE;
//     CtxSwitchFrame->ExceptionList = EXCEPTION_CHAIN_END;
//
//     /* Save back the new value of the kernel stack. */
//     Thread->KernelStack = (PVOID)CtxSwitchFrame;
// }
//
// DECLSPEC_NORETURN
// VOID
// KiIdleLoop(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD OldThread, NewThread;
//
//     /* Now loop forever */
//     while (TRUE)
//     {
//         /* Start of the idle loop: disable interrupts */
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
// #ifdef CONFIG_SMP
//             /* Do the swap at SYNCH_LEVEL */
//             KfRaiseIrql(SYNCH_LEVEL);
// #endif
//
//             /* Switch away from the idle thread */
//             KiSwapContext(APC_LEVEL, OldThread);
//
// #ifdef CONFIG_SMP
//             /* Go back to DISPATCH_LEVEL */
//             KeLowerIrql(DISPATCH_LEVEL);
// #endif
//         }
//         else
//         {
//             /* Continue staying idle. Note the HAL returns with interrupts on */
//             Prcb->PowerState.IdleFunction(&Prcb->PowerState);
//         }
//     }
// }
//
// BOOLEAN
// FASTCALL
// KiSwapContextExit(IN PKTHREAD OldThread,
//                   IN PKSWITCHFRAME SwitchFrame)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     PKPROCESS OldProcess, NewProcess;
//     PKTHREAD NewThread;
//
//     /* We are on the new thread stack now */
//     NewThread = Pcr->PrcbData.CurrentThread;
//
//     /* Now we are the new thread. Check if it's in a new process */
//     OldProcess = OldThread->ApcState.Process;
//     NewProcess = NewThread->ApcState.Process;
//     if (OldProcess != NewProcess)
//     {
//         /* Check if there is a different LDT */
//         if (*(PULONGLONG)&OldProcess->LdtDescriptor != *(PULONGLONG)&NewProcess->LdtDescriptor)
//         {
//             if (NewProcess->LdtDescriptor.LimitLow)
//             {
//                 KeSetGdtSelector(KGDT_LDT,
//                                  ((PULONG)&NewProcess->LdtDescriptor)[0],
//                                  ((PULONG)&NewProcess->LdtDescriptor)[1]);
//                 Ke386SetLocalDescriptorTable(KGDT_LDT);
//             }
//             else
//             {
//                 Ke386SetLocalDescriptorTable(0);
//             }
//         }
//
//         /* Switch address space and flush TLB */
//         __writecr3(NewProcess->DirectoryTableBase[0]);
//     }
//
//     /* Clear GS */
//     Ke386SetGs(0);
//
//     /* Set the TEB */
//     KiSetTebBase((PKPCR)Pcr, &NewThread->Teb->NtTib);
//
//     /* Set new TSS fields */
//     Pcr->TSS->Esp0 = (ULONG_PTR)NewThread->InitialStack;
//     if (!((KeGetTrapFrame(NewThread))->EFlags & EFLAGS_V86_MASK))
//     {
//         Pcr->TSS->Esp0 -= sizeof(KTRAP_FRAME) - FIELD_OFFSET(KTRAP_FRAME, V86Es);
//     }
//     Pcr->TSS->Esp0 -= NPX_FRAME_LENGTH;
//     Pcr->TSS->IoMapBase = NewProcess->IopmOffset;
//
//     /* Increase thread context switches */
//     NewThread->ContextSwitches++;
//
//     /* Load data from switch frame */
//     Pcr->NtTib.ExceptionList = SwitchFrame->ExceptionList;
//
//     /* DPCs shouldn't be active */
//     if (Pcr->PrcbData.DpcRoutineActive)
//     {
//         /* Crash the machine */
//         KeBugCheckEx(ATTEMPTED_SWITCH_FROM_DPC,
//                      (ULONG_PTR)OldThread,
//                      (ULONG_PTR)NewThread,
//                      (ULONG_PTR)OldThread->InitialStack,
//                      0);
//     }
//
//     /* Kernel APCs may be pending */
//     if (NewThread->ApcState.KernelApcPending)
//     {
//         /* Are APCs enabled? */
//         if (!NewThread->SpecialApcDisable)
//         {
//             /* Request APC delivery */
//             if (SwitchFrame->ApcBypassDisable)
//                 HalRequestSoftwareInterrupt(APC_LEVEL);
//             else
//                 return TRUE;
//         }
//     }
//
//     /* Return stating that no kernel APCs are pending*/
//     return FALSE;
// }
//
// VOID
// FASTCALL
// KiSwapContextEntry(IN PKSWITCHFRAME SwitchFrame,
//                    IN ULONG_PTR OldThreadAndApcFlag)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     PKTHREAD OldThread, NewThread;
//     ULONG Cr0, NewCr0;
//
//     /* Save APC bypass disable */
//     SwitchFrame->ApcBypassDisable = OldThreadAndApcFlag & 3;
//     SwitchFrame->ExceptionList = Pcr->NtTib.ExceptionList;
//
//     /* Increase context switch count and check if tracing is enabled */
//     Pcr->ContextSwitches++;
//     if (Pcr->PerfGlobalGroupMask)
//     {
//         /* We don't support this yet on x86 either */
//         DPRINT1("WMI Tracing not supported\n");
//         ASSERT(FALSE);
//     }
//
//     /* Get thread pointers */
//     OldThread = (PKTHREAD)(OldThreadAndApcFlag & ~3);
//     NewThread = Pcr->PrcbData.CurrentThread;
//
//     /* Get the old thread and set its kernel stack */
//     OldThread->KernelStack = SwitchFrame;
//
//     /* Set swapbusy to false for the new thread */
//     NewThread->SwapBusy = FALSE;
//
//     /* ISRs can change FPU state, so disable interrupts while checking */
//     _disable();
//
//     /* Get current and new CR0 and check if they've changed */
//     Cr0 = __readcr0();
//     NewCr0 = NewThread->NpxState |
//              (Cr0 & ~(CR0_MP | CR0_EM | CR0_TS)) |
//              KiGetThreadNpxArea(NewThread)->Cr0NpxState;
//     if (Cr0 != NewCr0)  __writecr0(NewCr0);
//
//     /* Now enable interrupts and do the switch */
//     _enable();
//     KiSwitchThreads(OldThread, NewThread->KernelStack);
// }
//
// VOID
// NTAPI
// KiDispatchInterrupt(VOID)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     PKPRCB Prcb = &Pcr->PrcbData;
//     PVOID OldHandler;
//     PKTHREAD NewThread, OldThread;
//
//     /* Disable interrupts */
//     _disable();
//
//     /* Check for pending timers, pending DPCs, or pending ready threads */
//     if ((Prcb->DpcData[0].DpcQueueDepth) ||
//         (Prcb->TimerRequest) ||
//         (Prcb->DeferredReadyListHead.Next))
//     {
//         /* Switch to safe execution context */
//         OldHandler = Pcr->NtTib.ExceptionList;
//         Pcr->NtTib.ExceptionList = EXCEPTION_CHAIN_END;
//
//         /* Retire DPCs while under the DPC stack */
//         KiRetireDpcListInDpcStack(Prcb, Prcb->DpcStack);
//
//         /* Restore context */
//         Pcr->NtTib.ExceptionList = OldHandler;
//     }
//
//     /* Re-enable interrupts */
//     _enable();
//
//     /* Check for quantum end */
//     if (Prcb->QuantumEnd)
//     {
//         /* Handle quantum end */
//         Prcb->QuantumEnd = FALSE;
//         KiQuantumEnd();
//     }
//     else if (Prcb->NextThread)
//     {
//         /* Acquire the PRCB lock */
//         KiAcquirePrcbLock(Prcb);
//
//         /* Capture current thread data */
//         OldThread = Prcb->CurrentThread;
//         NewThread = Prcb->NextThread;
//
//         /* Set new thread data */
//         Prcb->NextThread = NULL;
//         Prcb->CurrentThread = NewThread;
//
//         /* The thread is now running */
//         NewThread->State = Running;
//         OldThread->WaitReason = WrDispatchInt;
//
//         /* Make the old thread ready */
//         KxQueueReadyThread(OldThread, Prcb);
//
//         /* Swap to the new thread */
//         KiSwapContext(APC_LEVEL, OldThread);
//     }
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
