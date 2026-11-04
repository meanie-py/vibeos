//! Translation of sdk/lib/crt/except/amd64/ehandler.c — the amd64 C SEH personality
//! (`__C_specific_handler`, which walks a function's `__try` scope table) and
//! `_local_unwind`.
//!
//! The C was compiled into each amd64 consumer separately: ntoskrnl's and ntdll's cargo
//! cells (both link rtl-rust, so this one definition replaces it), libcntpr for drivers
//! (which now import it from ntoskrnl, whose spec exports both, as Windows drivers do),
//! and the boot loaders, which never referenced it.
//!
//! The arm64 handler (`arm64/except.rs`) is a different algorithm and is NOT shared: this
//! one advances `DispatcherContext->ScopeIndex` before each scope (so a nested exception
//! raised from a filter or `__finally` resumes after it), stops a target unwind at the
//! scope that contains the target or whose `__except` IS the target, and calls a filter
//! for any nonzero `HandlerAddress` other than the literal EXCEPTION_EXECUTE_HANDLER.

use super::unwind::{RtlUnwind, RtlUnwindEx};
use crate::types::*;
use ntkrnl_sys::ddk::*;
#[allow(unused_imports)]
use crate::types::{DISPATCHER_CONTEXT, EXCEPTION_DISPOSITION, PDISPATCHER_CONTEXT};

/// `LONG (*)(struct _EXCEPTION_POINTERS *, PVOID EstablisherFrame)` — a `__except` filter.
type PEXCEPTION_FILTER = unsafe extern "C" fn(*mut EXCEPTION_POINTERS, PVOID) -> LONG;
/// `VOID (*)(BOOLEAN AbnormalTermination, PVOID EstablisherFrame)` — a `__finally` body.
type PTERMINATION_HANDLER = unsafe extern "C" fn(BOOLEAN, PVOID);

#[no_mangle]
pub unsafe extern "C" fn __C_specific_handler(
    ExceptionRecord: PEXCEPTION_RECORD,
    EstablisherFrame: PVOID,
    ContextRecord: PCONTEXT,
    DispatcherContext: PDISPATCHER_CONTEXT,
) -> EXCEPTION_DISPOSITION {
    /* Set up the EXCEPTION_POINTERS */
    let mut ExceptionPointers = EXCEPTION_POINTERS { ExceptionRecord, ContextRecord };

    /* Get the image base */
    let ImageBase: ULONG64 = (*DispatcherContext).ImageBase as ULONG64;

    /* Get the image base relative instruction pointers */
    let IpOffset: ULONG64 = ((*DispatcherContext).ControlPc as ULONG64).wrapping_sub(ImageBase);
    let TargetIpOffset: ULONG64 = ((*DispatcherContext).TargetIp as ULONG64).wrapping_sub(ImageBase);

    /* Get the scope table; ScopeRecord is a flexible array */
    let ScopeTable = (*DispatcherContext).HandlerData as PSCOPE_TABLE;
    let ScopeRecord = (&raw const (*ScopeTable).ScopeRecord) as *const SCOPE_RECORD;

    /* Loop while we have scope table entries */
    while (*DispatcherContext).ScopeIndex < (*ScopeTable).Count {
        /* Use i as index and update the dispatcher context */
        let i = (*DispatcherContext).ScopeIndex;
        (*DispatcherContext).ScopeIndex = i + 1;
        let Scope = ScopeRecord.add(i as usize);

        /* Get the start and end of the scope */
        let BeginAddress = (*Scope).BeginAddress as ULONG64;
        let EndAddress = (*Scope).EndAddress as ULONG64;

        /* Skip this scope if we are not within the bounds */
        if IpOffset < BeginAddress || IpOffset >= EndAddress {
            continue;
        }

        if (*ExceptionRecord).ExceptionFlags & EXCEPTION_UNWIND != 0 {
            /* A target unwind stops at the scope that contains the target */
            if (*ExceptionRecord).ExceptionFlags & EXCEPTION_TARGET_UNWIND != 0
                && TargetIpOffset >= BeginAddress
                && TargetIpOffset < EndAddress
            {
                return ExceptionContinueSearch;
            }

            if (*Scope).JumpTarget == 0 {
                /* A termination handler / finally function: call it */
                let TerminationHandler: PTERMINATION_HANDLER = core::mem::transmute::<
                    usize,
                    PTERMINATION_HANDLER,
                >((ImageBase + (*Scope).HandlerAddress as ULONG64) as usize);
                TerminationHandler(1 /* TRUE */, EstablisherFrame);
            } else if (*Scope).JumpTarget as ULONG64 == TargetIpOffset {
                /* This __except is the unwind target */
                return ExceptionContinueSearch;
            }
        } else {
            /* We are only interested in exception handlers */
            if (*Scope).JumpTarget == 0 {
                continue;
            }

            /* This is an exception filter, get the handler address */
            let Handler = (*Scope).HandlerAddress;

            let FilterResult: LONG = if Handler == EXCEPTION_EXECUTE_HANDLER as ULONG {
                /* Hardcoded EXCEPTION_EXECUTE_HANDLER: this is our result */
                EXCEPTION_EXECUTE_HANDLER as LONG
            } else {
                /* Otherwise we need to call the filter */
                let ExceptionFilter: PEXCEPTION_FILTER = core::mem::transmute::<
                    usize,
                    PEXCEPTION_FILTER,
                >((ImageBase + Handler as ULONG64) as usize);
                ExceptionFilter(&mut ExceptionPointers, EstablisherFrame)
            };

            if FilterResult < 0 {
                /* EXCEPTION_CONTINUE_EXECUTION */
                return ExceptionContinueExecution;
            }

            if FilterResult > 0 {
                /* EXCEPTION_EXECUTE_HANDLER: unwind to the target address (does not return) */
                let JumpTarget = ImageBase + (*Scope).JumpTarget as ULONG64;
                RtlUnwindEx(
                    EstablisherFrame,
                    JumpTarget as usize as PVOID,
                    ExceptionRecord,
                    (*ExceptionRecord).ExceptionCode as u32 as usize as PVOID, /* UlongToPtr */
                    (*DispatcherContext).ContextRecord,
                    (*DispatcherContext).HistoryTable as _,
                );

                /* Should not get here */
                core::arch::asm!("int3", options(nomem, nostack));
            }
        }
    }

    /* Reached the end of the scope table */
    ExceptionContinueSearch
}

#[no_mangle]
pub unsafe extern "C" fn _local_unwind(frame: PVOID, target: PVOID) {
    RtlUnwind(frame, target, core::ptr::null_mut(), core::ptr::null_mut());
}
