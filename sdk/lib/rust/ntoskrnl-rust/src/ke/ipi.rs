//
// Fills in the DbgBreakPoint stubs for the non-amd64 (i386 + arm64) IPI
// primitives with real bodies, gated behind `#[cfg(feature = "smp")]`.
// UP behavior (feature "smp" NOT enabled) is preserved exactly as it was
// before this change (DbgBreakPoint()), per AGENTS.md "make UP behavior
// degrade to the current no-op semantics."
//
// See NOTES.md (staged alongside this file) for the design rationale,
// in particular:
//  - Why KiIpiServiceRoutine's IPI_SYNCH_REQUEST branch had to be corrected
//    (it was written for a stale 4-arg "Synchronize BOOLEAN" KiIpiSendPacket
//    that no longer matches the real 5-arg PKIPI_WORKER-based signature
//    used by the only two real callers in the tree today: KeIpiGenericCall
//    and i386/cpu.rs's KiFlushTargetEntireTb).
//  - Why the outer cfg for KiIpiServiceRoutine was widened from
//    `any(x86, arm)` to `not(x86_64)` (it silently didn't exist at all for
//    aarch64 before, which is wrong: ipi.c's `#if defined(_M_ARM) ||
//    defined(_M_AMD64)` guards 32-bit ARM and amd64 only, NOT aarch64 --
//    aarch64 takes the full (non-DbgBreakPoint) path in the real C, same
//    as i386).

use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    KAFFINITY, PKIPI_CONTEXT, PKIPI_WORKER, PKIPI_BROADCAST_WORKER,
    PKTRAP_FRAME, PKEXCEPTION_FRAME, PKPRCB,
    HalRequestSoftwareInterrupt, HalRequestIpi, KeGetCurrentProcessorNumber,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Use actual definitions, not extern declarations (lto=false splits them)
use crate::ke::processor::{KeNumberProcessors, KeActiveProcessors};
extern "C" {
    static mut KiReverseStallIpiLock: KSPIN_LOCK;
}

// ============================================================================
// Small helpers shared by the bodies below
// ============================================================================

/// Read `KiProcessorBlock[i]` through a raw pointer, matching the idiom
/// already used elsewhere in the tree (dpc.rs, thrdschd.rs) to avoid
/// depending on `[PKPRCB; MAXIMUM_PROCESSORS]` being directly indexable
/// through the extern static.
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
unsafe fn ki_processor_block(i: usize) -> PKPRCB {
    *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i)
}

#[cfg(not(target_arch = "x86_64"))]
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn ki_ipi_frozen_set_bit(Prcb: PKPRCB, BitPosition: ULONG) {
    core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32,
    )
    .fetch_or(1i32 << (BitPosition & 31), core::sync::atomic::Ordering::SeqCst);
}

// ============================================================================
// M3/M5c SMP visibility: prove each AP schedules real (non-idle) threads.
// ============================================================================
// Arch-neutral (hoisted from amd64/ipi.rs for the arm64 leg — the counters
// are pure atomics). The COUNT is taken in each arch's KiSwapContextResume
// (Prcb->Number != 0, incoming thread not the idle thread) — a bare atomic
// increment, because mid-context-switch (before OldThread->SwapBusy is
// released) is NO place to call DbgPrint: a contended print there wedges the
// CPU mid-swap holding the dead thread, and everything queued to this CPU
// starves (creeping SamSs/TDI wedge face, 2026-07-11). The PRINT happens
// from idle context (amd64: lazy-idle exit; arm64: idle-loop top) — APs
// idle constantly, so the marker lands moments after the threshold.
// One-shot markers at the 1st and 1000th non-idle switch; the -smp smoke
// milestone check greps "CPU<n> scheduling live".
//
// Both taps take the CPU number as a parameter: the caller derives it from
// its own invariant PCR source (arm64's linked KeGetPcr is the x18 copy,
// which is NOT reloaded by the switch frame — see arm64/thrdini.rs).

#[cfg(feature = "smp")]
static KiSmpApSwitchCount: [core::sync::atomic::AtomicU32; 64] = {
    #[allow(clippy::declare_interior_mutable_const)]
    const ZERO: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    [ZERO; 64]
};
/// bit0 = "first switch" marker printed, bit1 = "1000 switches" marker printed.
#[cfg(feature = "smp")]
static KiSmpApMarkerState: [core::sync::atomic::AtomicU32; 64] = {
    #[allow(clippy::declare_interior_mutable_const)]
    const ZERO: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    [ZERO; 64]
};

/// Context-switch tap: count only. NEVER print from here (see block comment).
#[cfg(feature = "smp")]
#[inline(always)]
pub(crate) unsafe fn KiSmpNoteApSwitch(Number: u32) {
    use core::sync::atomic::Ordering;
    KiSmpApSwitchCount[(Number & 63) as usize].fetch_add(1, Ordering::Relaxed);
}

/// Idle tap: emit any marker the counters have earned. Runs on the AP itself
/// in idle-thread context — safe to DbgPrint.
#[cfg(feature = "smp")]
#[inline(always)]
pub(crate) unsafe fn KiSmpApMarkerFlush(Number: u32) {
    use core::sync::atomic::Ordering;
    if Number == 0 {
        return;
    }
    let i = (Number & 63) as usize;
    let n = KiSmpApSwitchCount[i].load(Ordering::Relaxed);
    if n >= 1 && KiSmpApMarkerState[i].fetch_or(1, Ordering::Relaxed) & 1 == 0 {
        crate::dbg_print!(b"[SMP] CPU%u scheduling live (first non-idle context switch)\n\0", Number);
    }
    if n >= 1000 && KiSmpApMarkerState[i].fetch_or(2, Ordering::Relaxed) & 2 == 0 {
        crate::dbg_print!(b"[SMP] CPU%u scheduling live (1000 non-idle context switches)\n\0", Number);
    }
}

// ============================================================================
// KiIpiGenericCallTarget
// ============================================================================
//
// ipi.c's own body is literally `ASSERTMSG("Not yet implemented\n", FALSE)`
// -- it was never implemented upstream. We implement the standard NT
// "generic call" rendezvous protocol here so KeIpiGenericCall (below) and
// KiIpiSendPacket actually work end to end:
//
//   1. Decrement *Count (a shared arrival counter living in the sender's
//      stack frame, passed down through CurrentPacket[2]) to signal arrival.
//   2. Spin until the sender publishes the "go" signal by writing 0 to
//      *Count (KeIpiGenericCall does this once every CPU has arrived).
//   3. Invoke the real broadcast worker with its argument.
//   4. Signal completion via KiIpiSignalPacketDone(PacketContext), which
//      clears this CPU's bit in the sender's TargetSet.
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiGenericCallTarget(
    PacketContext: PKIPI_CONTEXT,
    BroadcastFunction: PVOID,
    Argument: PVOID,
    Count: PVOID,
) {
    #[cfg(feature = "smp")]
    {
        let CountPtr = Count as *mut u32;

        // Signal arrival.
        core::sync::atomic::AtomicU32::from_ptr(CountPtr)
            .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);

        // Wait for the sender's "go" signal (Count == 0).
        while core::sync::atomic::AtomicU32::from_ptr(CountPtr)
            .load(core::sync::atomic::Ordering::SeqCst)
            != 0
        {
            core::hint::spin_loop();
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        }

        // Call the real per-CPU broadcast worker.
        let Function: PKIPI_BROADCAST_WORKER = core::mem::transmute(BroadcastFunction);
        if let Some(f) = Function {
            // `as _`: bindgen's ULONG_PTR is u32 on i386 / u64 elsewhere.
            f(Argument as _);
        }

        // Tell the sender we're done.
        KiIpiSignalPacketDone(PacketContext);
    }
    #[cfg(not(feature = "smp"))]
    {
        let BroadcastFunction = opaque_ptr(BroadcastFunction);
        let Argument = opaque_ptr(Argument);
        let Count = opaque_ptr(Count);
        DbgBreakPoint();
    }
}

// ============================================================================
// KiIpiSend
// ============================================================================
//
// Real body per ipi.c's dead `KiIpiSendRequest` (the intended body of
// KiIpiSend -- ipi.c keeps it under `#if 0` but names it differently; the
// live declaration is `KiIpiSend(TargetProcessors, IpiRequest)`). For each
// target CPU, atomically set the request bit (IpiRequest is a BIT POSITION,
// e.g. IPI_APC/IPI_DPC/IPI_SYNCH_REQUEST, matching the convention already
// established by KiIpiServiceRoutine's `1 << IPI_x` bit tests below) in
// that CPU's Prcb->IpiFrozen, then request the actual interrupt once with
// the full target set (HalRequestIpi takes a KAFFINITY, not a processor
// index -- the `HalRequestIpi(i)` in ipi.c's dead code predates the current
// HAL prototype in ndk/halfuncs.h).

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSend(
    TargetProcessors: KAFFINITY,
    IpiRequest: ULONG,
) {
    #[cfg(feature = "smp")]
    {
        // IPI_FREEZE is NOT a request bit: the freeze protocol (i386/freeze.rs
        // KxFreezeExecution) stages Prcb->IpiFrozen = IPI_FROZEN_STATE_TARGET_
        // FREEZE (5) on every target BEFORE this call, and KiIpiServiceRoutine
        // dispatches on that exact STATE. OR-ing bit 4 here turned the staged
        // 5 into 0x15, the handler's exact-state compare rejected its own
        // freeze, and the owner spun forever — the KdSystemDebugControl
        // -smp 2 wedge (2026-07-18). Just ring the targets.
        if IpiRequest == IPI_FREEZE {
            HalRequestIpi(TargetProcessors);
            return;
        }
        let mut Current: KAFFINITY = 1;
        for i in 0..(KeNumberProcessors as usize) {
            if TargetProcessors & Current != 0 {
                let Prcb = ki_processor_block(i);
                if !Prcb.is_null() {
                    ki_ipi_frozen_set_bit(Prcb, IpiRequest);
                }
            }
            Current <<= 1;
        }
        HalRequestIpi(TargetProcessors);
    }
    #[cfg(not(feature = "smp"))]
    {
        DbgBreakPoint();
    }
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSend(
    TargetProcessors: KAFFINITY,
    IpiRequest: ULONG,
) {
    #[cfg(feature = "smp")]
    {
        // Same freeze-is-a-state-not-a-bit rule as the x86 arm above. arm64's
        // freeze handler isn't wired into its IPI receive path yet, but do not
        // corrupt the staged state either way.
        if IpiRequest == IPI_FREEZE {
            HalRequestIpi(TargetProcessors);
            return;
        }
        let mut Current: KAFFINITY = 1;
        for i in 0..(KeNumberProcessors as usize) {
            if TargetProcessors & Current != 0 {
                let Prcb = ki_processor_block(i);
                if !Prcb.is_null() {
                    ki_ipi_frozen_set_bit(Prcb, IpiRequest);
                }
            }
            Current <<= 1;
        }
        HalRequestIpi(TargetProcessors);
    }
    #[cfg(not(feature = "smp"))]
    {
        DbgBreakPoint();
    }
}

// ============================================================================
// KiIpiSendPacket
// ============================================================================
//
// Real body. Follows ipi.c's dead `KiIpiSendPacket` structurally (publish
// the packet fields on the CURRENT Prcb, claim each target's SignalDone
// slot, set the IPI_SYNCH_REQUEST bit, HalRequestIpi, then run the packet
// inline if we're one of our own targets) -- adapted from the historical
// 4-arg/"Synchronize BOOLEAN" shape to the real 5-arg PKIPI_WORKER-based
// signature that's actually declared today and actually used by
// KeIpiGenericCall and i386/cpu.rs's KiFlushTargetEntireTb:
//
//   CurrentPrcb->WorkerRoutine  = WorkerFunction   (the PKIPI_WORKER meta-fn,
//                                                    e.g. KiIpiGenericCallTarget
//                                                    or KiFlushTargetEntireTb)
//   CurrentPrcb->CurrentPacket[0] = BroadcastFunction (opaque PVOID, WorkerFunction's
//                                                    2nd parameter)
//   CurrentPrcb->CurrentPacket[1] = Context             (WorkerFunction's 3rd parameter)
//   CurrentPrcb->CurrentPacket[2] = Count                (WorkerFunction's 4th parameter)
//   CurrentPrcb->TargetSet         = TargetProcessors (drained bit-by-bit by
//                                                    KiIpiSignalPacketDone as
//                                                    each target finishes)
//
// KiIpiServiceRoutine (see below) is the receiving side: on the IPI_SYNCH_REQUEST
// bit, it reads `Prcb->SignalDone` (== the sender's PKPRCB, claimed below) and
// invokes `Sender->WorkerRoutine(Sender_as_PacketContext, Sender->CurrentPacket[0..2])`
// -- the real 4-arg PKIPI_WORKER call, matching KiFlushTargetEntireTb's actual
// declared shape (PacketContext, Parameter1, Parameter2, Parameter3).

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiSendPacket(
    TargetProcessors: KAFFINITY,
    WorkerFunction: PKIPI_WORKER,
    BroadcastFunction: PKIPI_BROADCAST_WORKER,
    Context: ULONG_PTR,
    Count: *mut ULONG,
) {
    #[cfg(feature = "smp")]
    {
        ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == SYNCH_LEVEL);

        let CurrentPrcb = KeGetCurrentPrcb();

        // Publish the packet on our own Prcb -- InterlockedExchange* in the
        // C, plain atomic stores here (single writer: the sending CPU).
        core::sync::atomic::AtomicUsize::from_ptr(
            core::ptr::addr_of_mut!((*CurrentPrcb).TargetSet) as *mut usize,
        )
        .store(TargetProcessors as usize, core::sync::atomic::Ordering::SeqCst);

        core::sync::atomic::AtomicPtr::from_ptr(
            core::ptr::addr_of_mut!((*CurrentPrcb).WorkerRoutine) as *mut PVOID,
        )
        .store(
            core::mem::transmute::<PKIPI_WORKER, PVOID>(WorkerFunction),
            core::sync::atomic::Ordering::SeqCst,
        );

        core::sync::atomic::AtomicPtr::from_ptr(
            core::ptr::addr_of_mut!((*CurrentPrcb).CurrentPacket[0]),
        )
        .store(
            core::mem::transmute::<PKIPI_BROADCAST_WORKER, PVOID>(BroadcastFunction),
            core::sync::atomic::Ordering::SeqCst,
        );

        core::sync::atomic::AtomicPtr::from_ptr(
            core::ptr::addr_of_mut!((*CurrentPrcb).CurrentPacket[1]),
        )
        .store(Context as PVOID, core::sync::atomic::Ordering::SeqCst);

        core::sync::atomic::AtomicPtr::from_ptr(
            core::ptr::addr_of_mut!((*CurrentPrcb).CurrentPacket[2]),
        )
        .store(Count as PVOID, core::sync::atomic::Ordering::SeqCst);

        let mut Processor: KAFFINITY = 1;
        for i in 0..(KeNumberProcessors as usize) {
            if TargetProcessors & Processor != 0 {
                let Prcb = ki_processor_block(i);
                if !Prcb.is_null() {
                    // while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone, CurrentPrcb, 0));
                    let SignalDoneSlot = core::sync::atomic::AtomicPtr::from_ptr(
                        core::ptr::addr_of_mut!((*Prcb).SignalDone) as *mut PKPRCB,
                    );
                    while SignalDoneSlot
                        .compare_exchange(
                            core::ptr::null_mut(),
                            CurrentPrcb,
                            core::sync::atomic::Ordering::SeqCst,
                            core::sync::atomic::Ordering::SeqCst,
                        )
                        .is_err()
                    {
                        core::hint::spin_loop();
                    }

                    ki_ipi_frozen_set_bit(Prcb, IPI_SYNCH_REQUEST);

                    if Processor != (*CurrentPrcb).SetMember as KAFFINITY {
                        HalRequestIpi(Processor);
                    }
                }
            }
            Processor <<= 1;
        }

        if TargetProcessors & (*CurrentPrcb).SetMember as KAFFINITY != 0 {
            let mut OldIrql: KIRQL = 0;
            KeRaiseIrql(IPI_LEVEL, core::ptr::addr_of_mut!(OldIrql));
            KiIpiServiceRoutine(core::ptr::null_mut(), core::ptr::null_mut()); // nolint: irql-balance (self-IPI shortcut runs the service routine inline at IPI_LEVEL, then lowers — mirrors C ke/ipi.c)
            KeLowerIrql(OldIrql);
        }
    }
    #[cfg(not(feature = "smp"))]
    {
        let Count = opaque_ptr(Count);
        DbgBreakPoint();
    }
}

// ============================================================================
// KiIpiSignalPacketDone / KiIpiSignalPacketDoneAndStall
// ============================================================================
//
// PacketContext (as established by KiIpiSendPacket/KiIpiServiceRoutine
// above) is the SENDING processor's PKPRCB. Signal completion by clearing
// our own bit in the sender's TargetSet -- KeIpiGenericCall's completion
// wait (`while CurrentPrcb->TargetSet != 0`) and any other TargetSet-based
// waiter observes TargetSet reach 0 once every target has called this.

#[cfg(not(target_arch = "x86_64"))]
#[cfg(feature = "smp")]
#[inline(always)]
unsafe fn ki_ipi_signal_packet_done_impl(PacketContext: PKIPI_CONTEXT) {
    let Sender = PacketContext as PKPRCB;
    if !Sender.is_null() {
        core::sync::atomic::AtomicUsize::from_ptr(
            core::ptr::addr_of_mut!((*Sender).TargetSet) as *mut usize,
        )
        .fetch_and(
            !((1usize) << KeGetCurrentProcessorNumber()),
            core::sync::atomic::Ordering::SeqCst,
        );
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSignalPacketDone(PacketContext: PKIPI_CONTEXT) {
    #[cfg(feature = "smp")]
    {
        ki_ipi_signal_packet_done_impl(PacketContext);
    }
    #[cfg(not(feature = "smp"))]
    {
        DbgBreakPoint();
    }
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSignalPacketDone(PacketContext: PKIPI_CONTEXT) {
    #[cfg(feature = "smp")]
    {
        ki_ipi_signal_packet_done_impl(PacketContext);
    }
    #[cfg(not(feature = "smp"))]
    {
        DbgBreakPoint();
    }
}

// KiIpiSignalPacketDoneAndStall has no call sites anywhere in the tree
// (grep confirms) and ipi.c never implemented it either -- there's no
// oracle to translate. Design (documented in NOTES.md): signal completion
// exactly like KiIpiSignalPacketDone, then stall (spin) until the caller's
// ReverseStall flag is released (written to 0), mirroring the
// KiReverseStallIpiLock-guarded "everyone waits for the master to say go"
// pattern used by the freeze/stop-processors family. volatile reads on the
// polled location match the C parameter's `volatile PULONG`.

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiIpiSignalPacketDoneAndStall(
    PacketContext: PKIPI_CONTEXT,
    ReverseStall: *mut ULONG,
) {
    #[cfg(feature = "smp")]
    {
        ki_ipi_signal_packet_done_impl(PacketContext);
        if !ReverseStall.is_null() {
            while core::ptr::read_volatile(ReverseStall) != 0 {
                core::hint::spin_loop();
            }
        }
    }
    #[cfg(not(feature = "smp"))]
    {
        let ReverseStall = opaque_ptr(ReverseStall);
        DbgBreakPoint();
    }
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KiIpiSignalPacketDoneAndStall(
    PacketContext: PKIPI_CONTEXT,
    ReverseStall: *mut ULONG,
) {
    #[cfg(feature = "smp")]
    {
        ki_ipi_signal_packet_done_impl(PacketContext);
        if !ReverseStall.is_null() {
            while core::ptr::read_volatile(ReverseStall) != 0 {
                core::hint::spin_loop();
            }
        }
    }
    #[cfg(not(feature = "smp"))]
    {
        let ReverseStall = opaque_ptr(ReverseStall);
        DbgBreakPoint();
    }
}

// ============================================================================
// KiIpiServiceRoutine
// ============================================================================
//
// Outer cfg WIDENED from `any(target_arch = "x86", target_arch = "arm")` to
// `not(target_arch = "x86_64")` -- ipi.c's `#ifndef _M_AMD64` gate means this
// function IS compiled for aarch64 in the real C (aarch64 isn't 32-bit ARM,
// so `_M_ARM` isn't defined there either); the old cfg silently dropped the
// whole function for aarch64, which would have been a link error / missing
// symbol the moment arm64 SMP needed it.
//
// The IPI_SYNCH_REQUEST branch's WorkerRoutine invocation was CORRECTED from
// a 1-argument call (`WorkerRoutine(CurrentPacket[0])`) to the real 4-argument
// PKIPI_WORKER call (`WorkerRoutine(PacketContext, CurrentPacket[0],
// CurrentPacket[1], CurrentPacket[2])`). The 1-arg version was written to
// match ipi.c's dead/`#if 0` 4-parameter "Synchronize BOOLEAN" KiIpiSendPacket,
// which is NOT the signature actually declared or actually called anywhere in
// this tree -- the real callers (KeIpiGenericCall, i386/cpu.rs's
// KiFlushTargetEntireTb) both use the 5-parameter PKIPI_WORKER-based
// KiIpiSendPacket, whose WorkerFunction parameter IS a real
// `fn(PacketContext, Parameter1, Parameter2, Parameter3)` (see
// KiFlushTargetEntireTb's own declared signature in i386/cpu.rs -- it takes
// exactly these 4 parameters). Calling it with only 1 argument supplied was
// undefined behavior (the callee would read uninitialized register/stack
// slots for its other 3 parameters). This is the "must change" case per
// AGENTS.md: leaving it as 1-arg would make the newly-implemented
// KiIpiSendPacket/KiIpiGenericCallTarget above unable to interoperate with
// the ALREADY-EXISTING, non-stub KiFlushTargetEntireTb caller.
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KiIpiServiceRoutine(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) -> BOOLEAN {
    #[cfg(feature = "smp")]
    {
        let Prcb = KeGetCurrentPrcb();

        // Freeze dispatch — STATE-based, not a request bit (see KiIpiSend's
        // IPI_FREEZE special case). amd64 delivers freeze via NMI; i386 rides
        // this same 0xE1 vector, so check the staged state here, FIRST (the
        // owner is spinning in KxFreezeExecution until we flip to FROZEN).
        // Needs the real TrapFrame for KiSaveProcessorState — the packet
        // self-service path (KiIpiSendPacket self-target) passes NULL and a
        // freeze never self-targets, so skipping there loses nothing: the
        // interrupt stays pending in the LAPIC and the real ISR delivers it.
        // Known gap vs NT (accepted until i386 gets an NMI path): a CPU
        // parked at IPI_LEVEL (e.g. inside a generic-call worker) can't take
        // 0xE1 and freezes only after it returns below IPI_LEVEL.
        #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (i386 IPI freeze handler (SMP), Rust-only)
        {
            if !TrapFrame.is_null()
                && core::ptr::read_volatile(core::ptr::addr_of!((*Prcb).IpiFrozen))
                    == IPI_FROZEN_STATE_TARGET_FREEZE as _
            {
                crate::ke::i386::freeze::KiProcessorFreezeHandler(TrapFrame, ExceptionFrame);
            }
        }

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_APC), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_APC) != 0
        {
            HalRequestSoftwareInterrupt(APC_LEVEL);
        }

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_DPC), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_DPC) != 0
        {
            (*Prcb).DpcInterruptRequested = 1u8;
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }

        if core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*Prcb).IpiFrozen) as *mut i32
        ).fetch_and(!(1 << IPI_SYNCH_REQUEST), core::sync::atomic::Ordering::SeqCst) & (1 << IPI_SYNCH_REQUEST) != 0
        {
            #[cfg(target_arch = "arm")]
            {
                // 32-bit ARM: never one of our 3 build targets
                // (i686/x86_64/aarch64-pc-windows-gnu), kept only for
                // parity with ipi.c's `#if defined(_M_ARM) ...` gate.
                DbgBreakPoint();
            }
            #[cfg(not(target_arch = "arm"))]
            {
                // Sender = the PKPRCB claimed for us by KiIpiSendPacket.
                let Sender = (*Prcb).SignalDone;
                if !Sender.is_null() {
                    let WorkerRoutine: PKIPI_WORKER =
                        core::mem::transmute((*Sender).WorkerRoutine);
                    if let Some(worker) = WorkerRoutine {
                        worker(
                            Sender as PKIPI_CONTEXT,
                            (*Sender).CurrentPacket[0],
                            (*Sender).CurrentPacket[1],
                            (*Sender).CurrentPacket[2],
                        );
                    }

                    // Release our claim on the sender's SignalDone slot so a
                    // future KiIpiSendPacket targeting us can claim it again.
                    // (The worker itself -- via KiIpiSignalPacketDone -- is
                    // responsible for clearing the sender's TargetSet bit;
                    // this only frees the per-target rendezvous slot.)
                    InterlockedExchangePointer(
                        core::ptr::addr_of_mut!((*Prcb).SignalDone) as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                }
            }
        }
    }
    1u8
}

// ============================================================================
// KeIpiGenericCall
// ============================================================================
//
// Completes the previously-DbgBreakPoint'd "wait for the other processors to
// finish" tail (ipi.c: `ASSERTMSG("Not yet implemented\n", FALSE)`), now that
// KiIpiSendPacket publishes CurrentPrcb->TargetSet == the full affinity we
// sent to, and each target's KiIpiSignalPacketDone (invoked from
// KiIpiGenericCallTarget once it finishes calling Function) clears its own
// bit. Waiting for TargetSet to reach 0 is therefore the correct completion
// signal.
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn KeIpiGenericCall(
    Function: PKIPI_BROADCAST_WORKER,
    Argument: ULONG_PTR,
) -> ULONG_PTR {
    let mut OldIrql: KIRQL = 0;
    let mut OldIrql2: KIRQL = 0;
    #[cfg(feature = "smp")]
    let mut Affinity: KAFFINITY;
    #[cfg(feature = "smp")]
    let mut Count: ULONG;
    #[cfg(feature = "smp")]
    let Prcb: PKPRCB;

    OldIrql = KeGetCurrentIrql();
    if OldIrql < DISPATCH_LEVEL {
        KeRaiseIrql(DISPATCH_LEVEL, core::ptr::addr_of_mut!(OldIrql));
    }

    #[cfg(feature = "smp")]
    {
        Count = KeNumberProcessors as ULONG;
        Affinity = KeActiveProcessors;
        Prcb = KeGetCurrentPrcb();
        Affinity &= !(*Prcb).SetMember;
    }

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KiReverseStallIpiLock));

    #[cfg(feature = "smp")]
    {
        if Affinity != 0 {
            KiIpiSendPacket(
                Affinity,
                Some(core::mem::transmute(KiIpiGenericCallTarget as *const ())),
                Function,
                Argument,
                core::ptr::addr_of_mut!(Count),
            );

            while Count != 1 {
                core::hint::spin_loop();
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    KeRaiseIrql(IPI_LEVEL, core::ptr::addr_of_mut!(OldIrql2) as *mut KIRQL);

    #[cfg(feature = "smp")]
    {
        // Publish the "go" signal -- every target CPU spinning in
        // KiIpiGenericCallTarget on `*Count != 0` proceeds to call Function.
        core::sync::atomic::AtomicU32::from_ptr(core::ptr::addr_of_mut!(Count) as *mut u32)
            .store(0, core::sync::atomic::Ordering::SeqCst);
    }

    let Status = (Function.unwrap())(Argument as _); // nolint: silent-failure (non-null by PKIPI_BROADCAST_WORKER contract; a None here is a caller bug — panic is the fail-loud path)

    #[cfg(feature = "smp")]
    {
        if Affinity != 0 {
            // Sanity check
            ntkrnl_sys::ASSERT!(Prcb == KeGetCurrentPrcb());

            // Wait for every target to signal done via KiIpiSignalPacketDone,
            // which clears its bit in Prcb->TargetSet (published by
            // KiIpiSendPacket above as the full `Affinity` target set).
            while core::sync::atomic::AtomicUsize::from_ptr(
                core::ptr::addr_of_mut!((*Prcb).TargetSet) as *mut usize,
            )
            .load(core::sync::atomic::Ordering::SeqCst)
                != 0
            {
                core::hint::spin_loop();
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(KiReverseStallIpiLock));

    KeLowerIrql(OldIrql);
    Status as ULONG_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/ipi.c
//  * PURPOSE:         Inter-Processor Packet Interface
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern KSPIN_LOCK KiReverseStallIpiLock;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #ifndef _M_AMD64
//
// VOID
// NTAPI
// KiIpiGenericCallTarget(IN PKIPI_CONTEXT PacketContext,
//                        IN PVOID BroadcastFunction,
//                        IN PVOID Argument,
//                        IN PVOID Count)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSend(IN KAFFINITY TargetProcessors,
//           IN ULONG IpiRequest)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// NTAPI
// KiIpiSendPacket(IN KAFFINITY TargetProcessors,
//                 IN PKIPI_WORKER WorkerFunction,
//                 IN PKIPI_BROADCAST_WORKER BroadcastFunction,
//                 IN ULONG_PTR Context,
//                 IN PULONG Count)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSignalPacketDone(IN PKIPI_CONTEXT PacketContext)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// VOID
// FASTCALL
// KiIpiSignalPacketDoneAndStall(IN PKIPI_CONTEXT PacketContext,
//                               IN volatile PULONG ReverseStall)
// {
//     /* FIXME: TODO */
//     ASSERTMSG("Not yet implemented\n", FALSE);
// }
//
// #if 0
// VOID
// NTAPI
// KiIpiSendRequest(IN KAFFINITY TargetSet,
//                  IN ULONG IpiRequest)
// {
// #ifdef CONFIG_SMP
//     LONG i;
//     PKPRCB Prcb;
//     KAFFINITY Current;
//
//     for (i = 0, Current = 1; i < KeNumberProcessors; i++, Current <<= 1)
//     {
//         if (TargetSet & Current)
//         {
//             /* Get the PRCB for this CPU */
//             Prcb = KiProcessorBlock[i];
//
//             InterlockedBitTestAndSet((PLONG)&Prcb->IpiFrozen, IpiRequest);
//             HalRequestIpi(i);
//         }
//     }
// #endif
// }
//
// VOID
// NTAPI
// KiIpiSendPacket(IN KAFFINITY TargetSet,
//                 IN PKIPI_BROADCAST_WORKER WorkerRoutine,
//                 IN ULONG_PTR Argument,
//                 IN ULONG Count,
//                 IN BOOLEAN Synchronize)
// {
// #ifdef CONFIG_SMP
//     KAFFINITY Processor;
//     LONG i;
//     PKPRCB Prcb, CurrentPrcb;
//     KIRQL oldIrql;
//
//     ASSERT(KeGetCurrentIrql() == SYNCH_LEVEL);
//
//     CurrentPrcb = KeGetCurrentPrcb();
//     (void)InterlockedExchangeUL(&CurrentPrcb->TargetSet, TargetSet);
//     (void)InterlockedExchangeUL(&CurrentPrcb->WorkerRoutine, (ULONG_PTR)WorkerRoutine);
//     (void)InterlockedExchangePointer(&CurrentPrcb->CurrentPacket[0], Argument);
//     (void)InterlockedExchangeUL(&CurrentPrcb->CurrentPacket[1], Count);
//     (void)InterlockedExchangeUL(&CurrentPrcb->CurrentPacket[2], Synchronize ? 1 : 0);
//
//     for (i = 0, Processor = 1; i < KeNumberProcessors; i++, Processor <<= 1)
//     {
//         if (TargetSet & Processor)
//         {
//             Prcb = KiProcessorBlock[i];
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone, (LONG)CurrentPrcb, 0));
//             InterlockedBitTestAndSet((PLONG)&Prcb->IpiFrozen, IPI_SYNCH_REQUEST);
//             if (Processor != CurrentPrcb->SetMember)
//             {
//                 HalRequestIpi(i);
//             }
//         }
//     }
//     if (TargetSet & CurrentPrcb->SetMember)
//     {
//         KeRaiseIrql(IPI_LEVEL, &oldIrql);
//         KiIpiServiceRoutine(NULL, NULL);
//         KeLowerIrql(oldIrql);
//     }
// #endif
// }
// #endif
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KiIpiServiceRoutine(IN PKTRAP_FRAME TrapFrame,
//                     IN PKEXCEPTION_FRAME ExceptionFrame)
// {
// #ifdef CONFIG_SMP
//     PKPRCB Prcb;
//     ASSERT(KeGetCurrentIrql() == IPI_LEVEL);
//
//     Prcb = KeGetCurrentPrcb();
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_APC))
//     {
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_DPC))
//     {
//         Prcb->DpcInterruptRequested = TRUE;
//         HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//     }
//
//     if (InterlockedBitTestAndReset((PLONG)&Prcb->IpiFrozen, IPI_SYNCH_REQUEST))
//     {
// #if defined(_M_ARM) || defined(_M_AMD64)
//         DbgBreakPoint();
// #else
//         (void)InterlockedDecrementUL(&Prcb->SignalDone->CurrentPacket[1]);
//         if (InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[2], 0, 0))
//         {
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[1], 0, 0));
//         }
//         ((VOID (NTAPI*)(PVOID))(Prcb->SignalDone->WorkerRoutine))(Prcb->SignalDone->CurrentPacket[0]);
//         InterlockedBitTestAndReset((PLONG)&Prcb->SignalDone->TargetSet, KeGetCurrentProcessorNumber());
//         if (InterlockedCompareExchangeUL(&Prcb->SignalDone->CurrentPacket[2], 0, 0))
//         {
//             while (0 != InterlockedCompareExchangeUL(&Prcb->SignalDone->TargetSet, 0, 0));
//         }
//         (void)InterlockedExchangePointer((PVOID*)&Prcb->SignalDone, NULL);
// #endif // _M_ARM
//     }
// #endif
//    return TRUE;
// }
//
// /*
//  * @implemented
//  */
// ULONG_PTR
// NTAPI
// KeIpiGenericCall(IN PKIPI_BROADCAST_WORKER Function,
//                  IN ULONG_PTR Argument)
// {
//     ULONG_PTR Status;
//     KIRQL OldIrql, OldIrql2;
// #ifdef CONFIG_SMP
//     KAFFINITY Affinity;
//     ULONG Count;
//     PKPRCB Prcb = KeGetCurrentPrcb();
// #endif
//
//     /* Raise to DPC level if required */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < DISPATCH_LEVEL) KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
// #ifdef CONFIG_SMP
//     /* Get current processor count and affinity */
//     Count = KeNumberProcessors;
//     Affinity = KeActiveProcessors;
//
//     /* Exclude ourselves */
//     Affinity &= ~Prcb->SetMember;
// #endif
//
//     /* Acquire the IPI lock */
//     KeAcquireSpinLockAtDpcLevel(&KiReverseStallIpiLock);
//
// #ifdef CONFIG_SMP
//     /* Make sure this is MP */
//     if (Affinity)
//     {
//         /* Send an IPI */
//         KiIpiSendPacket(Affinity,
//                         KiIpiGenericCallTarget,
//                         Function,
//                         Argument,
//                         &Count);
//
//         /* Spin until the other processors are ready */
//         while (Count != 1)
//         {
//             YieldProcessor();
//             KeMemoryBarrierWithoutFence();
//         }
//     }
// #endif
//
//     /* Raise to IPI level */
//     KeRaiseIrql(IPI_LEVEL, &OldIrql2);
//
// #ifdef CONFIG_SMP
//     /* Let the other processors know it is time */
//     Count = 0;
// #endif
//
//     /* Call the function */
//     Status = Function(Argument);
//
// #ifdef CONFIG_SMP
//     /* If this is MP, wait for the other processors to finish */
//     if (Affinity)
//     {
//         /* Sanity check */
//         ASSERT(Prcb == KeGetCurrentPrcb());
//
//         /* FIXME: TODO */
//         ASSERTMSG("Not yet implemented\n", FALSE);
//     }
// #endif
//
//     /* Release the lock */
//     KeReleaseSpinLockFromDpcLevel(&KiReverseStallIpiLock);
//
//     /* Lower IRQL back */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// #endif // !_M_AMD64
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
