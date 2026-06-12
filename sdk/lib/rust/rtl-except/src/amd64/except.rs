//! Translation of sdk/lib/rtl/amd64/except.c — user-mode exception support for
//! AMD64 (RtlDispatchException entry).
//!
//! librtl always builds with `-D_NTOSKRNL_` (see sdk/lib/rtl/CMakeLists.txt
//! add_definitions), so the `#ifndef _NTOSKRNL_` VEH blocks
//! (RtlCallVectoredExceptionHandlers / RtlCallVectoredContinueHandlers) are
//! dead code in the only variant ever built and are omitted here, matching
//! the compiled C exactly.

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
    /* (VEH dispatch elided — kernel variant, see module docs.) */

    /* Call the internal unwind routine */
    crate::amd64::unwind::RtlpUnwindInternal(
        core::ptr::null_mut(), // TargetFrame
        core::ptr::null_mut(), // TargetIp
        ExceptionRecord,
        core::ptr::null_mut(), // ReturnValue
        ContextRecord,
        core::ptr::null_mut(), // HistoryTable
        UNW_FLAG_EHANDLER,
    )
}
