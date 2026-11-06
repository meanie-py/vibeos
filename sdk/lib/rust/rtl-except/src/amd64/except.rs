//! Translation of sdk/lib/rtl/amd64/except.c — user-mode exception support for
//! AMD64 (RtlDispatchException entry).
//!
//! The C guards its two VEH blocks (RtlCallVectoredExceptionHandlers /
//! RtlCallVectoredContinueHandlers) with `#ifndef _NTOSKRNL_`, but librtl is
//! only ever built with `-D_NTOSKRNL_` (sdk/lib/rtl/CMakeLists.txt), so the
//! compiled C never ran a vectored handler even in ntdll. That is a C defect:
//! the guard means "not in the kernel". This diverges and decides at run time
//! with `RtlpGetMode() == UserMode`, as arm64 does; ntoskrnl's RtlpGetMode is
//! KernelMode, so the kernel still never touches the TEB-based VEH lock.

use crate::types::*;
use ntkrnl_sys::ddk::*;
#[allow(unused_imports)]
use crate::types::EXCEPTION_DISPOSITION;

/// @unimplemented
#[no_mangle]
pub unsafe extern "system" fn RtlpGetExceptionAddress() -> PVOID {
    /* UNIMPLEMENTED (NDEBUG no-op) */
    core::ptr::null_mut()
}

/// @unimplemented (upstream tag)
#[no_mangle]
pub unsafe extern "system" fn RtlDispatchException(
    ExceptionRecord: PEXCEPTION_RECORD,
    ContextRecord: PCONTEXT,
) -> BOOLEAN {
    let UserModeDispatch = RtlpGetMode() == UserMode;

    /* First-chance vectored handlers (user mode only, see module docs) */
    if UserModeDispatch && RtlCallVectoredExceptionHandlers(ExceptionRecord, ContextRecord) != 0 {
        /* Exception handled, now call vectored continue handlers */
        RtlCallVectoredContinueHandlers(ExceptionRecord, ContextRecord);

        /* Continue execution */
        return 1;
    }

    /* Call the internal unwind routine */
    let Handled = crate::amd64::unwind::RtlpUnwindInternal(
        core::ptr::null_mut(), // TargetFrame
        core::ptr::null_mut(), // TargetIp
        ExceptionRecord,
        core::ptr::null_mut(), // ReturnValue
        ContextRecord,
        core::ptr::null_mut(), // HistoryTable
        UNW_FLAG_EHANDLER,
    );

    /* In user mode, call any registered vectored continue handlers */
    if UserModeDispatch {
        RtlCallVectoredContinueHandlers(ExceptionRecord, ContextRecord);
    }

    Handled
}

/* rtl-diag vectoreh.rs, linked into the same librtl aggregate */
extern "system" {
    fn RtlCallVectoredExceptionHandlers(
        ExceptionRecord: PEXCEPTION_RECORD,
        Context: PCONTEXT,
    ) -> BOOLEAN;
    fn RtlCallVectoredContinueHandlers(ExceptionRecord: PEXCEPTION_RECORD, Context: PCONTEXT);
}
