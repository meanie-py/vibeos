//! User-mode exception support for IA-32 — translated from
//! `sdk/lib/rtl/i386/except.c` (RtlGetCallersAddress, RtlDispatchException,
//! RtlUnwind).
//!
//! `NDEBUG` is defined in the C, so `DPRINT`/`ASSERT` are no-ops and are
//! omitted here with comments at their original locations.

use ntkrnl_sys::ddk::*;
use crate::types::*;
/* Disambiguate names present in both globs (equivalent declarations;
 * prefer the curated types.rs ones). */
use crate::types::{EXCEPTION_DISPOSITION, RtlRaiseException};

/* Asm trampolines (i386/except_asm.s) + per-consumer libsupp helpers +
 * rtl-diag vectoreh.rs — all NTAPI (stdcall on i386). */
extern "system" {
    fn RtlpGetExceptionList() -> PEXCEPTION_REGISTRATION_RECORD;
    fn RtlpSetExceptionList(NewExceptionList: PEXCEPTION_REGISTRATION_RECORD);
    fn RtlpExecuteHandlerForException(
        ExceptionRecord: PEXCEPTION_RECORD,
        RegistrationFrame: PEXCEPTION_REGISTRATION_RECORD,
        Context: PCONTEXT,
        DispatcherContext: PVOID,
        ExceptionHandler: PEXCEPTION_ROUTINE,
    ) -> EXCEPTION_DISPOSITION;
    fn RtlpExecuteHandlerForUnwind(
        ExceptionRecord: PEXCEPTION_RECORD,
        RegistrationFrame: PEXCEPTION_REGISTRATION_RECORD,
        Context: PCONTEXT,
        DispatcherContext: PVOID,
        ExceptionHandler: PEXCEPTION_ROUTINE,
    ) -> EXCEPTION_DISPOSITION;
    fn RtlpHandleDpcStackException(
        RegistrationFrame: PEXCEPTION_REGISTRATION_RECORD,
        RegistrationFrameEnd: ULONG_PTR,
        StackLow: *mut ULONG_PTR,
        StackHigh: *mut ULONG_PTR,
    ) -> BOOLEAN;
    /// asm (i386) — note the `p`; this is NOT RtlCaptureContext.
    fn RtlpCaptureContext(ContextRecord: PCONTEXT);
    /// rtl-diag vectoreh.rs
    fn RtlCallVectoredContinueHandlers(ExceptionRecord: PEXCEPTION_RECORD, Context: PCONTEXT);
    /// Tiny asm helper standing in for `_ReturnAddress()` (see global_asm! below).
    fn RtlpGetReturnAddress() -> PVOID;
}

/* C's `_ReturnAddress()` intrinsic has no stable Rust equivalent; this helper
 * returns its own return address — i.e. the call site inside RtlUnwind, one
 * frame off from what the C intrinsic would yield. Acceptable: the value only
 * feeds the diagnostic-only ExceptionRecord3.ExceptionAddress field. */
core::arch::global_asm!(
    ".global _RtlpGetReturnAddress@0",
    "_RtlpGetReturnAddress@0:",
    "mov eax, [esp]",
    "ret",
);

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RtlGetCallersAddress(
    CallersAddress: *mut PVOID,
    CallersCaller: *mut PVOID,
) {
    let mut BackTrace: [PVOID; 2] = [core::ptr::null_mut(); 2];
    let BackTraceHash: PULONG = core::ptr::null_mut();

    /* Get the tow back trace address */
    let FrameCount: USHORT =
        RtlCaptureStackBackTrace(2, 2, &mut BackTrace[0], BackTraceHash);

    /* Only if user want it */
    if !CallersAddress.is_null() {
        /* only when first frames exist */
        if FrameCount >= 1 {
            *CallersAddress = BackTrace[0];
        } else {
            *CallersAddress = core::ptr::null_mut();
        }
    }

    /* Only if user want it */
    if !CallersCaller.is_null() {
        /* only when second frames exist */
        if FrameCount >= 2 {
            *CallersCaller = BackTrace[1];
        } else {
            *CallersCaller = core::ptr::null_mut();
        }
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RtlDispatchException(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
) -> BOOLEAN {
    let mut NestedFrame: PEXCEPTION_REGISTRATION_RECORD = core::ptr::null_mut();
    /* Uninitialized stack local in C; the callee fills it before any read. */
    let mut DispatcherContext = DISPATCHER_CONTEXT {
        RegistrationPointer: core::ptr::null_mut(),
    };
    let mut ExceptionRecord2: EXCEPTION_RECORD = core::mem::zeroed();
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;

    /*
     * The C source guards an early RtlCallVectoredExceptionHandlers dispatch
     * with `#ifndef _NTOSKRNL_` — but librtl always builds with -D_NTOSKRNL_
     * (sdk/lib/rtl/CMakeLists.txt add_definitions), so that block is dead
     * code in the only variant ever built and is omitted here.
     */

    /* Get the current stack limits and registration frame */
    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);
    let mut RegistrationFrame = RtlpGetExceptionList();

    /* Now loop every frame */
    while RegistrationFrame != EXCEPTION_CHAIN_END {
        /* Registration chain entries are never NULL */
        /* ASSERT(RegistrationFrame != NULL) — no-op under NDEBUG */

        /* Find out where it ends */
        let RegistrationFrameEnd = RegistrationFrame as ULONG_PTR
            + core::mem::size_of::<EXCEPTION_REGISTRATION_RECORD>() as ULONG_PTR;

        /* Make sure the registration frame is located within the stack */
        if (RegistrationFrameEnd > StackHigh)
            || ((RegistrationFrame as ULONG_PTR) < StackLow)
            || ((RegistrationFrame as ULONG_PTR) & 0x3 != 0)
        {
            /* Check if this happened in the DPC Stack */
            if RtlpHandleDpcStackException(
                RegistrationFrame,
                RegistrationFrameEnd,
                &mut StackLow,
                &mut StackHigh,
            ) != 0
            {
                /* Use DPC Stack Limits and restart */
                continue;
            }

            /* Set invalid stack and bail out */
            (*ExceptionRecord).ExceptionFlags |= EXCEPTION_STACK_INVALID;
            return 0;
        }

        //
        // TODO: Implement and call here RtlIsValidHandler(RegistrationFrame->Handler)
        // for supporting SafeSEH functionality, see the following articles:
        // https://www.optiv.com/blog/old-meets-new-microsoft-windows-safeseh-incompatibility
        // https://msrc-blog.microsoft.com/2012/01/10/more-information-on-the-impact-of-ms12-001/
        //
        // Full SafeSEH needs the image's registered-handler table and is still
        // the TODO above. These two cases need none of it, and leaving them
        // out is what turns ONE guest fault into a process death:
        //
        //   * Handler == 0 (or in the reserved low 64K). Calling it executes
        //     at address 0. The resulting fault re-enters this function, which
        //     re-walks the SAME chain from fs:[0], finds the SAME null handler
        //     and calls it again, one frame deeper -- MEASURED at 56 identical
        //     `eip=00000000 rw=8` faults at descending esp before the process
        //     died. The bounds check above cannot catch it: the RECORD is
        //     within the stack, it is the HANDLER that is junk.
        //   * Handler inside [StackLow, StackHigh]. A handler that points into
        //     the thread's own stack is never code; that is the shape of a
        //     popped/overwritten registration record being walked as if live.
        //     Measured on the first frame of the same cascade: handler was a
        //     stack address and executing it faulted immediately.
        //
        // Measured case that motivated this: a guest whose fs:[0] pointed 0x44
        // bytes BELOW its own esp -- abandoned stack -- at an all-zero record
        // (`next=00000000` where a terminated chain must be 0xffffffff).
        //
        // Bail out the same way an invalid stack does rather than skipping to
        // the next frame: once a record is this corrupt its `Next` is not
        // trustworthy either, and returning FALSE sends the ORIGINAL exception
        // to second chance, which is diagnosable and kills the process once
        // instead of 56 times.
        {
            /* bindgen models the handler as Option<fn>, so a null slot is
             * None rather than 0 — take the raw address either way. */
            let Handler = match (*RegistrationFrame).Handler {
                Some(f) => f as usize as ULONG_PTR,
                None => 0,
            };
            if Handler < 0x10000
                || (Handler >= StackLow && Handler < StackHigh)
            {
                (*ExceptionRecord).ExceptionFlags |= EXCEPTION_STACK_INVALID;
                return 0;
            }
        }

        /* Check if logging is enabled */
        RtlpCheckLogException(
            ExceptionRecord,
            Context,
            RegistrationFrame as PVOID,
            core::mem::size_of::<EXCEPTION_REGISTRATION_RECORD>() as ULONG,
        );

        /* Call the handler */
        let Disposition = RtlpExecuteHandlerForException(
            ExceptionRecord,
            RegistrationFrame,
            Context,
            &mut DispatcherContext as *mut _ as PVOID,
            (*RegistrationFrame).Handler,
        );

        /* Check if this is a nested frame */
        if RegistrationFrame == NestedFrame {
            /* Mask out the flag and the nested frame */
            (*ExceptionRecord).ExceptionFlags &= !EXCEPTION_NESTED_CALL;
            NestedFrame = core::ptr::null_mut();
        }

        /* Handle the dispositions.
         * NB: the C switch FALLS THROUGH from ExceptionContinueExecution into
         * ExceptionContinueSearch when the non-continuable RtlRaiseException
         * returns — replicated below with explicit control flow. */
        if Disposition == ExceptionContinueExecution {
            /* Check if it was non-continuable */
            if (*ExceptionRecord).ExceptionFlags & EXCEPTION_NONCONTINUABLE != 0 {
                /* Set up the exception record */
                ExceptionRecord2.ExceptionRecord = ExceptionRecord;
                ExceptionRecord2.ExceptionCode = STATUS_NONCONTINUABLE_EXCEPTION;
                ExceptionRecord2.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                ExceptionRecord2.NumberParameters = 0;

                /* Raise the exception */
                RtlRaiseException(&mut ExceptionRecord2);

                /* C fall-through into the ExceptionContinueSearch case */
                if (*ExceptionRecord).ExceptionFlags & EXCEPTION_STACK_INVALID != 0 {
                    /* We have an invalid stack, bail out */
                    return 0;
                }
            } else {
                /* In user mode, call any registered vectored continue handlers */
                RtlCallVectoredContinueHandlers(ExceptionRecord, Context);

                /* Execution continues */
                return 1;
            }
        } else if Disposition == ExceptionContinueSearch {
            /* Continue searching */
            if (*ExceptionRecord).ExceptionFlags & EXCEPTION_STACK_INVALID != 0 {
                /* We have an invalid stack, bail out */
                return 0;
            }
        } else if Disposition == ExceptionNestedException {
            /* Nested exception: turn the nested flag on */
            (*ExceptionRecord).ExceptionFlags |= EXCEPTION_NESTED_CALL;

            /* Update the current nested frame */
            if DispatcherContext.RegistrationPointer > NestedFrame {
                /* Get the frame from the dispatcher context */
                NestedFrame = DispatcherContext.RegistrationPointer;
            }
        } else {
            /* Anything else: set up the exception record */
            ExceptionRecord2.ExceptionRecord = ExceptionRecord;
            ExceptionRecord2.ExceptionCode = STATUS_INVALID_DISPOSITION;
            ExceptionRecord2.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
            ExceptionRecord2.NumberParameters = 0;

            /* Raise the exception */
            RtlRaiseException(&mut ExceptionRecord2);
        }

        /* Go to the next frame */
        RegistrationFrame = (*RegistrationFrame).Next;
    }

    /* Unhandled, bail out */
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RtlUnwind(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    mut ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
) {
    let _ = TargetIp; /* unused in the C body too (only sizeof'd) */
    /* Uninitialized stack local in C; the callee fills it before any read. */
    let mut DispatcherContext = DISPATCHER_CONTEXT {
        RegistrationPointer: core::ptr::null_mut(),
    };
    let mut ExceptionRecord2: EXCEPTION_RECORD = core::mem::zeroed();
    let mut ExceptionRecord3: EXCEPTION_RECORD = core::mem::zeroed();
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;
    let mut LocalContext: CONTEXT = core::mem::zeroed();

    /* Get the current stack limits */
    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);

    /* Check if we don't have an exception record */
    if ExceptionRecord.is_null() {
        /* Overwrite the argument */
        ExceptionRecord = &mut ExceptionRecord3;

        /* Setup a local one (written through the pointer: direct stores to
         * the local would be dead under Rust's aliasing rules since all
         * later reads go through ExceptionRecord) */
        (*ExceptionRecord).ExceptionFlags = 0;
        (*ExceptionRecord).ExceptionCode = STATUS_UNWIND;
        (*ExceptionRecord).ExceptionRecord = core::ptr::null_mut();
        /* C uses _ReturnAddress(); the helper returns the call site inside
         * RtlUnwind instead (one frame off) — diagnostic-only field. */
        (*ExceptionRecord).ExceptionAddress = RtlpGetReturnAddress();
        (*ExceptionRecord).NumberParameters = 0;
    }

    /* Check if we have a frame */
    if !TargetFrame.is_null() {
        /* Set it as unwinding */
        (*ExceptionRecord).ExceptionFlags |= EXCEPTION_UNWINDING;
    } else {
        /* Set the Exit Unwind flag as well */
        (*ExceptionRecord).ExceptionFlags |= EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND;
    }

    /* Now capture the context (ContextFlags written through the pointer —
     * same aliasing rationale as ExceptionRecord3 above) */
    let Context: PCONTEXT = &mut LocalContext;
    (*Context).ContextFlags = CONTEXT_INTEGER | CONTEXT_CONTROL | CONTEXT_SEGMENTS;
    RtlpCaptureContext(Context);

    /* Pop the current arguments off (4 stdcall args = 16 bytes on i386) */
    (*Context).Esp += (core::mem::size_of::<PVOID>()      /* TargetFrame */
        + core::mem::size_of::<PVOID>()                   /* TargetIp */
        + core::mem::size_of::<PEXCEPTION_RECORD>()       /* ExceptionRecord */
        + core::mem::size_of::<PVOID>()) as ULONG; /* ReturnValue */

    /* Set the new value for EAX */
    (*Context).Eax = ReturnValue as ULONG;

    /* Get the current frame */
    let mut RegistrationFrame = RtlpGetExceptionList();

    /* Now loop every frame */
    while RegistrationFrame != EXCEPTION_CHAIN_END {
        /* Registration chain entries are never NULL */
        /* ASSERT(RegistrationFrame != NULL) — no-op under NDEBUG */

        /* If this is the target */
        if RegistrationFrame as PVOID == TargetFrame {
            ZwContinue(Context, 0);
        }

        /* Check if the frame is too low */
        if !TargetFrame.is_null()
            && (TargetFrame as ULONG_PTR) < (RegistrationFrame as ULONG_PTR)
        {
            /* Create an invalid unwind exception */
            ExceptionRecord2.ExceptionCode = STATUS_INVALID_UNWIND_TARGET;
            ExceptionRecord2.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
            ExceptionRecord2.ExceptionRecord = ExceptionRecord;
            ExceptionRecord2.NumberParameters = 0;

            /* Raise the exception */
            RtlRaiseException(&mut ExceptionRecord2);
        }

        /* Find out where it ends */
        let RegistrationFrameEnd = RegistrationFrame as ULONG_PTR
            + core::mem::size_of::<EXCEPTION_REGISTRATION_RECORD>() as ULONG_PTR;

        /* Make sure the registration frame is located within the stack */
        if (RegistrationFrameEnd > StackHigh)
            || ((RegistrationFrame as ULONG_PTR) < StackLow)
            || ((RegistrationFrame as ULONG_PTR) & 0x3 != 0)
        {
            /* Check if this happened in the DPC Stack */
            if RtlpHandleDpcStackException(
                RegistrationFrame,
                RegistrationFrameEnd,
                &mut StackLow,
                &mut StackHigh,
            ) != 0
            {
                /* Use DPC Stack Limits and restart */
                continue;
            }

            /* Create an invalid stack exception */
            ExceptionRecord2.ExceptionCode = STATUS_BAD_STACK;
            ExceptionRecord2.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
            ExceptionRecord2.ExceptionRecord = ExceptionRecord;
            ExceptionRecord2.NumberParameters = 0;

            /* Raise the exception */
            RtlRaiseException(&mut ExceptionRecord2);
        } else {
            /* Call the handler */
            let Disposition = RtlpExecuteHandlerForUnwind(
                ExceptionRecord,
                RegistrationFrame,
                Context,
                &mut DispatcherContext as *mut _ as PVOID,
                (*RegistrationFrame).Handler,
            );

            if Disposition == ExceptionContinueSearch {
                /* Continue searching */
            } else if Disposition == ExceptionCollidedUnwind {
                /* Collision: get the original frame */
                RegistrationFrame = DispatcherContext.RegistrationPointer;
            } else {
                /* Anything else: set up the exception record */
                ExceptionRecord2.ExceptionRecord = ExceptionRecord;
                ExceptionRecord2.ExceptionCode = STATUS_INVALID_DISPOSITION;
                ExceptionRecord2.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                ExceptionRecord2.NumberParameters = 0;

                /* Raise the exception */
                RtlRaiseException(&mut ExceptionRecord2);
            }

            /* Go to the next frame */
            let OldFrame = RegistrationFrame;
            RegistrationFrame = (*RegistrationFrame).Next;

            /* Remove this handler */
            RtlpSetExceptionList(OldFrame);
        }
    }

    /* Check if we reached the end */
    if TargetFrame == EXCEPTION_CHAIN_END as PVOID {
        /* Unwind completed, so we don't exit */
        ZwContinue(Context, 0);
    } else {
        /* This is an exit_unwind or the frame wasn't present in the list */
        ZwRaiseException(ExceptionRecord, Context, 0);
    }
}

/* EOF */
