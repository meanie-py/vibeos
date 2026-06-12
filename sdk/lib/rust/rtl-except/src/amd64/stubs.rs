//! Translation of sdk/lib/rtl/amd64/stubs.c — AMD64 RtlInitializeContext +
//! RtlQueueApcWow64Thread stub.

use crate::types::ke64::*;
use crate::types::*;
use ntkrnl_sys::ddk::*;

/// @unimplemented (upstream tag — the body below IS the upstream implementation)
#[no_mangle]
pub unsafe extern "system" fn RtlInitializeContext(
    _ProcessHandle: HANDLE,
    ThreadContext: PCONTEXT,
    ThreadStartParam: PVOID,
    ThreadStartAddress: PTHREAD_START_ROUTINE,
    StackBase: PINITIAL_TEB,
) {
    /* Initialize everything to 0 */
    core::ptr::write_bytes(ThreadContext, 0, 1);

    /* Initialize StartAddress and Stack */
    (*ThreadContext).Rip = core::mem::transmute::<_, ULONG64>(ThreadStartAddress);
    (*ThreadContext).Rsp = (StackBase as ULONG64) - 6 * core::mem::size_of::<PVOID>() as ULONG64;

    /* Align stack by 16 and substract 8 (unaligned on function entry) */
    (*ThreadContext).Rsp &= !15u64;
    (*ThreadContext).Rsp -= 8;

    /* Enable Interrupts */
    (*ThreadContext).EFlags = EFLAGS_INTERRUPT_MASK;

    /* Set start parameter */
    (*ThreadContext).Rcx = ThreadStartParam as ULONG64;

    /* Set the Selectors */
    if ((*ThreadContext).Rip as i64) < 0 {
        /* Initialize kernel mode segments */
        (*ThreadContext).SegCs = KGDT64_R0_CODE;
        (*ThreadContext).SegDs = KGDT64_R3_DATA;
        (*ThreadContext).SegEs = KGDT64_R3_DATA;
        (*ThreadContext).SegFs = KGDT64_R3_CMTEB;
        (*ThreadContext).SegGs = KGDT64_R3_DATA;
        (*ThreadContext).SegSs = KGDT64_R0_DATA;
    } else {
        /* Initialize user mode segments */
        (*ThreadContext).SegCs = KGDT64_R3_CODE | RPL_MASK;
        (*ThreadContext).SegDs = KGDT64_R3_DATA | RPL_MASK;
        (*ThreadContext).SegEs = KGDT64_R3_DATA | RPL_MASK;
        (*ThreadContext).SegFs = KGDT64_R3_CMTEB | RPL_MASK;
        (*ThreadContext).SegGs = KGDT64_R3_DATA | RPL_MASK;
        (*ThreadContext).SegSs = KGDT64_R3_DATA | RPL_MASK;
    }

    (*ThreadContext).MxCsr = INITIAL_MXCSR;

    /* Only the basic Context is initialized */
    (*ThreadContext).ContextFlags =
        CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_SEGMENTS | CONTEXT_FLOATING_POINT;
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueueApcWow64Thread(
    _ThreadHandle: HANDLE,
    _ApcRoutine: PKNORMAL_ROUTINE,
    _NormalContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) -> NTSTATUS {
    /* UNIMPLEMENTED (NDEBUG no-op in the C) */
    STATUS_NOT_IMPLEMENTED
}
