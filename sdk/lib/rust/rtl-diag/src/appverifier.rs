use crate::shim::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{PCSTR, PTEB};

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.
#[no_mangle]
pub unsafe extern "system" fn RtlApplicationVerifierStop(
    Code: ULONG_PTR,
    Message: PCSTR,
    Value1: PVOID,
    Description1: PCSTR,
    Value2: PVOID,
    Description2: PCSTR,
    Value3: PVOID,
    Description3: PCSTR,
    Value4: PVOID,
    Description4: PCSTR,
) {
    let Teb: PTEB = NtCurrentTeb();

    DbgPrint(b"**************************************************\n\0".as_ptr() as PCSTR);
    DbgPrint(
        b"VERIFIER STOP %08Ix: pid %04Ix:  %s\n\0".as_ptr() as PCSTR,
        Code,
        (*Teb).ClientId.UniqueProcess as ULONG_PTR,
        Message,
    );
    DbgPrint(b"    %p : %s\n\0".as_ptr() as PCSTR, Value1, Description1);
    DbgPrint(b"    %p : %s\n\0".as_ptr() as PCSTR, Value2, Description2);
    DbgPrint(b"    %p : %s\n\0".as_ptr() as PCSTR, Value3, Description3);
    DbgPrint(b"    %p : %s\n\0".as_ptr() as PCSTR, Value4, Description4);
    DbgPrint(b"**************************************************\n\0".as_ptr() as PCSTR);
    DbgBreakPoint();
}
