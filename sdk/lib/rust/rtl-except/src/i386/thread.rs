//! Rtl user thread functions (i386) — translated from
//! `sdk/lib/rtl/i386/thread.c` (RtlInitializeContext, RtlQueueApcWow64Thread).
//!
//! `NDEBUG` is defined in the C, so `DPRINT` is a no-op and is omitted.

use ntkrnl_sys::ddk::*;
use crate::types::ke32::*;
use crate::types::*;

/* PRIVATE FUNCTIONS *******************************************************/

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RtlInitializeContext(
    ProcessHandle: HANDLE,
    ThreadContext: PCONTEXT,
    ThreadStartParam: PVOID,
    ThreadStartAddress: PTHREAD_START_ROUTINE,
    InitialTeb: PINITIAL_TEB,
) {
    /* DPRINT("RtlInitializeContext: ...") — no-op under NDEBUG */

    /*
     * Set the Initial Registers
     * This is based on NT's default values -- crazy apps might expect this...
     */
    (*ThreadContext).Ebp = 0;
    (*ThreadContext).Eax = 0;
    (*ThreadContext).Ebx = 1;
    (*ThreadContext).Ecx = 2;
    (*ThreadContext).Edx = 3;
    (*ThreadContext).Esi = 4;
    (*ThreadContext).Edi = 5;

    /* Set the Selectors */
    (*ThreadContext).SegGs = 0;
    (*ThreadContext).SegFs = KGDT_R3_TEB;
    (*ThreadContext).SegEs = KGDT_R3_DATA;
    (*ThreadContext).SegDs = KGDT_R3_DATA;
    (*ThreadContext).SegSs = KGDT_R3_DATA;
    (*ThreadContext).SegCs = KGDT_R3_CODE;

    /* Enable Interrupts */
    (*ThreadContext).EFlags = EFLAGS_INTERRUPT_MASK;

    /* Settings passed — (ULONG)ThreadStartAddress: Option<fn> is
     * pointer-sized (niche optimization), transmute matches the C cast */
    (*ThreadContext).Eip = core::mem::transmute::<PTHREAD_START_ROUTINE, ULONG>(ThreadStartAddress);
    (*ThreadContext).Esp = InitialTeb as ULONG;

    /* Only the basic Context is initialized */
    (*ThreadContext).ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_SEGMENTS;

    /* Set up ESP to the right value */
    (*ThreadContext).Esp -= core::mem::size_of::<PVOID>() as ULONG;
    ZwWriteVirtualMemory(
        ProcessHandle,
        (*ThreadContext).Esp as PVOID,
        &ThreadStartParam as *const PVOID as PVOID, /* address of the parameter local, as in C */
        core::mem::size_of::<PVOID>() as SIZE_T,
        core::ptr::null_mut(),
    );

    /* Push it down one more notch for RETEIP */
    (*ThreadContext).Esp -= core::mem::size_of::<PVOID>() as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueueApcWow64Thread(
    ThreadHandle: HANDLE,
    ApcRoutine: PKNORMAL_ROUTINE,
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) -> NTSTATUS {
    NtQueueApcThread(
        ThreadHandle,
        ApcRoutine,
        NormalContext,
        SystemArgument1,
        SystemArgument2,
    )
}

/* EOF */
