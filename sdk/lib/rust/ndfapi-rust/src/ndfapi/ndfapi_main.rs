use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NdfCloseIncident(handle: NDFHANDLE) -> HRESULT {
    // FIXME("%p\n", handle);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn NdfCreateConnectivityIncident(handle: *mut NDFHANDLE) -> HRESULT {
    // FIXME("%p\n", handle);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn NdfCreateWebIncident(
    url: LPCWSTR,
    handle: *mut NDFHANDLE,
) -> HRESULT {
    // FIXME("%s, %p\n", debugstr_w(url), handle);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn NdfCreateWinSockIncident(
    sock: SOCKET,
    host: LPCWSTR,
    port: USHORT,
    appId: LPCWSTR,
    userId: *mut SID,
    handle: *mut NDFHANDLE,
) -> HRESULT {
    // FIXME("%Ix, %s, %s, %p, %p\n", sock, debugstr_w(host), debugstr_w(appId), userId, handle);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn NdfDiagnoseIncident(
    handle: NDFHANDLE,
    count: *mut ULONG,
    causes: *mut *mut RootCauseInfo,
    wait: DWORD,
    flags: DWORD,
) -> HRESULT {
    // FIXME("%p, %p, %p, %ld, %ld\n", handle, count, causes, wait, flags);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn NdfExecuteDiagnosis(handle: NDFHANDLE, hwnd: HWND) -> HRESULT {
    // FIXME("%p, %p\n", handle, hwnd);
    E_NOTIMPL
}
