use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::PSTR;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.

#[no_mangle]
pub unsafe extern "system" fn RtlAssert(
    FailedAssertion: PVOID,
    FileName: PVOID,
    LineNumber: ULONG,
    Message: PCHAR,
) {
    static mut AssertCount: LONG = 0;
    let Count = InterlockedIncrement(core::ptr::addr_of_mut!(AssertCount));

    if Count <= 20 {
        DbgPrint(
            b"\n*** ASSERT FAILED: %s at %s:%lu\n\0".as_ptr() as PSTR,
            FailedAssertion as PSTR,
            FileName as PSTR,
            LineNumber,
        );
    }
}
