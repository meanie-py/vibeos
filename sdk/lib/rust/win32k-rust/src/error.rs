use win32k_sys::win32k::*;
// NtCurrentTeb: use the `inline::ke` impl (real per-arch TIB read intrinsic),
// not the bindgen extern decl in ddk — the latter has no link symbol on i386.
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::ddk::RtlNtStatusToDosError;
#[no_mangle]
pub unsafe extern "system" fn EngGetLastError() -> ULONG {
    let pTeb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;
    if !pTeb.is_null() {
        (*pTeb).LastErrorValue
    } else {
        ERROR_SUCCESS
    }
}
#[no_mangle]
pub unsafe extern "system" fn EngSetLastError(iError: ULONG) {
    let pTeb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;
    if !pTeb.is_null() {
        (*pTeb).LastErrorValue = iError;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn SetLastNtError(Status: NTSTATUS) {
    EngSetLastError(RtlNtStatusToDosError(Status));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn SetLastNtError(Status: NTSTATUS) {
    EngSetLastError(RtlNtStatusToDosError(Status));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include <win32k.h>
//
// /*
//  * @implemented
//  * https://learn.microsoft.com/en-us/windows/win32/api/winddi/nf-winddi-enggetlasterror
//  */
// ULONG
// APIENTRY
// EngGetLastError(VOID)
// {
//     PTEB pTeb = NtCurrentTeb();
//     return (pTeb ? pTeb->LastErrorValue : ERROR_SUCCESS);
// }
//
// /*
//  * @implemented
//  * https://learn.microsoft.com/en-us/windows/win32/api/winddi/nf-winddi-engsetlasterror
//  * Win: UserSetLastError
//  */
// VOID
// APIENTRY
// EngSetLastError(_In_ ULONG iError)
// {
//     PTEB pTeb = NtCurrentTeb();
//     if (pTeb)
//         pTeb->LastErrorValue = iError;
// }
//
// VOID
// FASTCALL
// SetLastNtError(_In_ NTSTATUS Status)
// {
//     EngSetLastError(RtlNtStatusToDosError(Status));
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
