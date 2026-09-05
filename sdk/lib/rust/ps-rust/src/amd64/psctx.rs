//! Translated from `ntoskrnl/ps/amd64/psctx.c` — Process Manager: Set/Get Context for amd64.

use ntkrnl_sys::ddk::*;

extern "C" {
    fn KiSetTrapContext(TrapFrame: PKTRAP_FRAME, Context: PCONTEXT, RequestorMode: KPROCESSOR_MODE);
    fn KiGetTrapContext(TrapFrame: PKTRAP_FRAME, Context: PCONTEXT);
}

extern "system" {
    fn KeSetEvent(Event: PKEVENT, Increment: KPRIORITY, Wait: BOOLEAN) -> LONG;
}

const KernelMode: KPROCESSOR_MODE = 0;
const IO_NO_INCREMENT: KPRIORITY = 0;

/// APC kernel routine for NtGetContextThread / NtSetContextThread.
#[no_mangle]
pub unsafe extern "system" fn PspGetOrSetContextKernelRoutine(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    _SystemArgument1: *mut PVOID,
    _SystemArgument2: *mut PVOID,
) {
    // GET_SET_CTX_CONTEXT { Apc, Event, Mode, padding, Context } — Apc is the first field,
    // so CONTAINING_RECORD here is just a typed cast (offset 0).
    let GetSetContext: PGET_SET_CTX_CONTEXT = Apc as PGET_SET_CTX_CONTEXT;
    let Thread: PKTHREAD = (*Apc).SystemArgument2 as PKTHREAD;
    ntkrnl_sys::ASSERT!(ntkrnl_sys::inline::tier1::KeGetCurrentThread() == Thread);

    // Kernel-mode requests use the saved trap frame stashed on the KTHREAD.
    let mut TrapFrame: PKTRAP_FRAME = core::ptr::null_mut();
    if (*GetSetContext).Mode == KernelMode {
        TrapFrame = (*Thread).TrapFrame;
    }

    // Otherwise compute the static stack location.
    if TrapFrame.is_null() {
        TrapFrame = ntkrnl_sys::inline::ke::KeGetTrapFrame(
            ntkrnl_sys::inline::tier1::KeGetCurrentThread(),
        );
    }

    let Context: PCONTEXT = core::ptr::addr_of_mut!((*GetSetContext).Context);
    let Event: PKEVENT = core::ptr::addr_of_mut!((*GetSetContext).Event);

    if !(*Apc).SystemArgument1.is_null() {
        // Set: write nonvolatiles back through the trap frame.
        KiSetTrapContext(TrapFrame, Context, (*GetSetContext).Mode);
    } else {
        // Get: read nonvolatiles out of the trap frame.
        KiGetTrapContext(TrapFrame, Context);
    }

    KeSetEvent(Event, IO_NO_INCREMENT, 0);
}
