use win32k_sys::win32k::*;
use crate::types::*;

#[no_mangle]
pub unsafe extern "system" fn DxDxgGenericThunk(
    ulIndex: ULONG_PTR,
    ulHandle: ULONG_PTR,
    pdwSizeOfPtr1: *mut SIZE_T,
    pvPtr1: PVOID,
    pdwSizeOfPtr2: *mut SIZE_T,
    pvPtr2: PVOID,
) -> DWORD {
    DDHAL_DRIVER_NOTHANDLED
}

#[no_mangle]
pub unsafe extern "system" fn DxDdIoctl(
    ulIoctl: ULONG,
    pBuffer: PVOID,
    ulBufferSize: ULONG,
) -> DWORD {
    return DDERR_UNSUPPORTED;
}
