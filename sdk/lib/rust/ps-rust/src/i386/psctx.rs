//! Translated from `ntoskrnl/ps/i386/psctx.c` — Process Manager: Set/Get Context for i386.

use ntkrnl_sys::ddk::*;

extern "system" {
    fn KeTrapFrameToContext(TrapFrame: PKTRAP_FRAME, ExceptionFrame: PVOID, Context: PCONTEXT);
    fn KeContextToTrapFrame(
        Context: PCONTEXT,
        ExceptionFrame: PVOID,
        TrapFrame: PKTRAP_FRAME,
        ContextFlags: ULONG,
        PreviousMode: KPROCESSOR_MODE,
    );
    fn KeSetEvent(Event: PKEVENT, Increment: KPRIORITY, Wait: BOOLEAN) -> LONG;
}

const KernelMode: KPROCESSOR_MODE = 0;
const IO_NO_INCREMENT: KPRIORITY = 0;

/// Convert the trap frame to a context.
#[no_mangle]
pub unsafe extern "system" fn PspGetContext(
    TrapFrame: PKTRAP_FRAME,
    _NonVolatileContext: PVOID,
    Context: PCONTEXT,
) {
    KeTrapFrameToContext(TrapFrame, core::ptr::null_mut(), Context);
}

/// Convert the context to a trap frame structure.
#[no_mangle]
pub unsafe extern "system" fn PspSetContext(
    TrapFrame: PKTRAP_FRAME,
    _NonVolatileContext: PVOID,
    Context: PCONTEXT,
    Mode: KPROCESSOR_MODE,
) {
    KeContextToTrapFrame(
        Context,
        core::ptr::null_mut(),
        TrapFrame,
        (*Context).ContextFlags,
        Mode,
    );
}

/// APC kernel routine for NtGetContextThread / NtSetContextThread.
#[no_mangle]
pub unsafe extern "system" fn PspGetOrSetContextKernelRoutine(
    Apc: PKAPC,
    _NormalRoutine: *mut PKNORMAL_ROUTINE,
    _NormalContext: *mut PVOID,
    _SystemArgument1: *mut PVOID,
    _SystemArgument2: *mut PVOID,
) {
    // GET_SET_CTX_CONTEXT layout: { Apc, Event, Mode, Context }; Apc is the first field,
    // so CONTAINING_RECORD here is just a typed cast (offset 0).
    let GetSetContext: PGET_SET_CTX_CONTEXT = Apc as PGET_SET_CTX_CONTEXT;
    let Context: PCONTEXT = core::ptr::addr_of_mut!((*GetSetContext).Context);
    let Event: PKEVENT = core::ptr::addr_of_mut!((*GetSetContext).Event);
    let Mode: KPROCESSOR_MODE = (*GetSetContext).Mode;
    let Thread: PETHREAD = (*Apc).SystemArgument2 as PETHREAD;

    // Kernel-mode requests use the saved trap frame stashed on the KTHREAD.
    let mut TrapFrame: PKTRAP_FRAME = core::ptr::null_mut();
    if Mode == KernelMode {
        TrapFrame = (*Thread).Tcb.TrapFrame;
    }

    // Otherwise compute the static stack location.
    if TrapFrame.is_null() {
        TrapFrame = ntkrnl_sys::inline::ke::KeGetTrapFrame(
            core::ptr::addr_of_mut!((*Thread).Tcb),
        );
    }

    // SystemArgument1 is the get/set selector (non-NULL = set).
    if !(*Apc).SystemArgument1.is_null() {
        PspSetContext(TrapFrame, core::ptr::null_mut(), Context, Mode);
    } else {
        PspGetContext(TrapFrame, core::ptr::null_mut(), Context);
    }

    KeSetEvent(Event, IO_NO_INCREMENT, 0);
}
