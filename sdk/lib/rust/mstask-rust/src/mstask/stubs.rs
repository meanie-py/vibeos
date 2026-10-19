use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type SASEC_HANDLE = *mut u16;

pub const MAX_BUFFER_SIZE: i32 = 273; // nolint: const-contradiction (stubs.c:77's own #define; mode.c's 4096 is a different module's macro)

#[no_mangle]
pub unsafe extern "system" fn ConvertAtJobsToTasks() -> HRESULT {
    // UNIMPLEMENTED;
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn GetNetScheduleAccountInformation(
    pwszServerName: LPCWSTR,
    ccAccount: DWORD,
    wszAccount: *mut WCHAR,
) -> HRESULT {
    UNIMPLEMENTED!();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn NetrJobAdd(
    ServerName: ATSVC_HANDLE,
    pAtInfo: LPAT_INFO,
    pJobId: LPDWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    (E_NOTIMPL) as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetrJobDel(
    ServerName: ATSVC_HANDLE,
    MinJobId: DWORD,
    MaxJobId: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    (E_NOTIMPL) as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetrJobEnum(
    ServerName: ATSVC_HANDLE,
    pEnumContainer: LPAT_ENUM_CONTAINER,
    PreferedMaximumLength: DWORD,
    pTotalEntries: LPDWORD,
    pResumeHandle: LPDWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    (E_NOTIMPL) as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetrJobGetInfo(
    ServerName: ATSVC_HANDLE,
    JobId: DWORD,
    ppAtInfo: *mut LPAT_INFO,
) -> NET_API_STATUS {
    // UNIMPLEMENTED;
    (E_NOTIMPL) as u32
}

#[no_mangle]
pub unsafe extern "system" fn SAGetAccountInformation(
    Handle: SASEC_HANDLE,
    pwszJobName: LPCWSTR,
    ccBufferSize: DWORD,
    wszBuffer: *mut wchar_t,
) -> HRESULT {
    UNIMPLEMENTED!();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn SAGetNSAccountInformation(
    Handle: SASEC_HANDLE,
    ccBufferSize: DWORD,
    wszBuffer: *mut wchar_t,
) -> HRESULT {
    UNIMPLEMENTED!();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn SASetAccountInformation(
    Handle: SASEC_HANDLE,
    pwszJobName: *const wchar_t,
    pwszAccount: *const wchar_t,
    pwszPassword: *const wchar_t,
    dwJobFlags: DWORD,
) -> HRESULT {
    // UNIMPLEMENTED;
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn SASetNSAccountInformation(
    Handle: SASEC_HANDLE,
    pwszAccount: *const wchar_t,
    pwszPassword: *const wchar_t,
) -> HRESULT {
    // UNIMPLEMENTED;
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn SetNetScheduleAccountInformation(
    pwszServerName: LPCWSTR,
    pwszAccount: LPCWSTR,
    pwszPassword: LPCWSTR,
) -> HRESULT {
    // UNIMPLEMENTED;
    E_NOTIMPL
}
