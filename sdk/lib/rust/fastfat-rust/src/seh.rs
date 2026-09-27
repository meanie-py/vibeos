//! fastfat's structured exception handling, on the real trampolines in
//! ntkrnl_sys::seh (ntoskrnl/sehtry.c).
//!
//! Under USE_DUMMY_PSEH every `_SEH2_EXCEPT` in this crate was dead and every
//! `_SEH2_FINALLY` ran only on the fall-through path, so a raise-by-design
//! (FatRaiseStatus, STATUS_CANT_WAIT, STATUS_VERIFY_REQUIRED, a Cc in-page error)
//! found no handler and bugchecked 0x1E. The idioms here replace that:
//!
//! * `_SEH2_TRY { B } _SEH2_FINALLY { F }` -> `try_finally(&mut (), |_| B, |_, abnormal| F)`;
//!   `_SEH2_AbnormalTermination()` is the `abnormal` argument.
//! * `_SEH2_TRY { B } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { H }` ->
//!   `if let Err(code) = fat_try_except(addr_of!(IrpContext), || B) { H }`.
//! * any other filter -> `try_except` with the filter written out.
//!
//! A `try_return(S)` inside a body is `S; return;` out of the closure (the finally
//! still runs). A real `return X` from the enclosing function inside a body must
//! become "store X, return from the closure, then return X after the call".
#![allow(non_snake_case)]

use ntkrnl_sys::ddk::{ExRaiseStatus, EXCEPTION_POINTERS, NTSTATUS};
pub use ntkrnl_sys::seh::{
    exception_code, try_except, try_finally, EXCEPTION_CONTINUE_EXECUTION,
    EXCEPTION_CONTINUE_SEARCH, EXCEPTION_EXECUTE_HANDLER,
};

use crate::types::IRP_CONTEXT;

extern "system" {
    fn FsRtlNormalizeNtstatus(Exception: NTSTATUS, GenericException: NTSTATUS) -> NTSTATUS;
}

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNEXPECTED_IO_ERROR: NTSTATUS = 0xC00000E9u32 as i32;

/// `_SEH2_TRY { body } _SEH2_EXCEPT( FatExceptionFilter( IrpContext,
/// _SEH2_GetExceptionInformation() ) )`. `irp_context` points at the caller's
/// IrpContext LOCAL, not its value: the FSD entry points assign it inside the
/// body, and the filter must see the value current at raise time (NULL when the
/// raise was FatCreateIrpContext's own allocation failure). Returns the exception
/// code when the handler is to run. `T` is whatever pointee the caller's local
/// was translated with (IRP_CONTEXT or c_void).
pub unsafe fn fat_try_except<T, F: FnOnce()>(
    irp_context: *const *mut T,
    body: F,
) -> Result<(), NTSTATUS> {
    try_except(
        &mut (),
        |_| body(),
        |_, pointers| {
            crate::fatdata::FatExceptionFilter(*irp_context as *mut IRP_CONTEXT, pointers as *mut EXCEPTION_POINTERS)
                as i32
        },
    )
}

/// fatprocs.h `FatRaiseStatus`: record the status in the IrpContext (which marks
/// it as expected for FatExceptionFilter / FatProcessException), then raise.
#[no_mangle]
pub unsafe extern "system" fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS) -> ! {
    if !IrpContext.is_null() {
        (*IrpContext).ExceptionStatus = Status;
    }
    ExRaiseStatus(Status)
}

/// fatprocs.h `FatNormalizeAndRaiseStatus`: the ORIGINAL status goes in the
/// IrpContext and the normalized one is raised.
#[no_mangle]
pub unsafe extern "system" fn FatNormalizeAndRaiseStatus(
    IrpContext: *mut IRP_CONTEXT,
    Status: NTSTATUS,
) -> ! {
    if !IrpContext.is_null() {
        (*IrpContext).ExceptionStatus = Status;
    }
    ExRaiseStatus(FsRtlNormalizeNtstatus(Status, STATUS_UNEXPECTED_IO_ERROR))
}

/// fatprocs.h `FatResetExceptionState`.
#[no_mangle]
pub unsafe extern "system" fn FatResetExceptionState(IrpContext: *mut IRP_CONTEXT) {
    if !IrpContext.is_null() {
        (*IrpContext).ExceptionStatus = STATUS_SUCCESS;
    }
}
