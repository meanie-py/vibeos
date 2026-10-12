// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4D0E0B244A33: [u16; 11] = [
    0x005Cu16, 0x005C, 0x002E, 0x005C, 0x0066, 0x006C, 0x0074, 0x006D, 0x0067, 0x0072,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define WIN32_NO_STATUS (no value; compile-time flag only, no Rust item emitted)
// #define NTOS_MODE_USER (no value; compile-time flag only, no Rust item emitted)
// static HRESULT FilterLoadUnload(LPCWSTR lpFilterName, BOOL Load); (forward prototype; provided by the function definition)
// static DWORD SendIoctl(HANDLE Handle, ULONG IoControlCode, LPVOID lpInBuffer, DWORD BufferSize); (forward prototype; provided by the function definition)

#[no_mangle]
pub unsafe extern "system" fn FilterLoad(lpFilterName: LPCWSTR) -> HRESULT {
    return FilterLoadUnload(lpFilterName, (TRUE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn FilterUnload(lpFilterName: LPCWSTR) -> HRESULT {
    return FilterLoadUnload(lpFilterName, (FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn NtStatusToHResult(Status: NTSTATUS) -> HRESULT {
    let mut hr: HRESULT = 0;
    hr = RtlNtStatusToDosError(Status) as HRESULT;
    if hr != ERROR_SUCCESS as HRESULT {
        hr = ((hr as ULONG) | (0x80070000 as ULONG)) as HRESULT;
    }
    return hr;
}

unsafe extern "C" fn FilterLoadUnload(lpFilterName: LPCWSTR, Load: BOOL) -> HRESULT {
    let mut FilterName: PFILTER_NAME = core::ptr::null_mut();
    let mut hFltMgr: HANDLE = core::ptr::null_mut();
    let mut StringLength: SIZE_T = 0;
    let mut BufferLength: SIZE_T = 0;
    let mut dwError: DWORD = 0;

    /* Get a handle to the filter manager */
    hFltMgr = CreateFileW(RUSTCONV_WSTR_4D0E0B244A33.as_ptr(),
                          GENERIC_WRITE,
                          FILE_SHARE_WRITE,
                          core::ptr::null_mut(),
                          OPEN_EXISTING,
                          FILE_ATTRIBUTE_NORMAL,
                          (core::ptr::addr_of_mut!(hFltMgr)).cast());
    if hFltMgr == INVALID_HANDLE_VALUE
    {
        dwError = GetLastError();
        return HRESULT_FROM_WIN32!(dwError);
    }

    /* Calc and allocate a buffer to hold our filter name */
    StringLength = (wcslen(lpFilterName) as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T);
    BufferLength = StringLength + (core::mem::size_of::<FILTER_NAME>()) as SIZE_T;
    FilterName = RtlAllocateHeap(GetProcessHeap() as PVOID,
                                 0,
                                 BufferLength) as PFILTER_NAME;
    if FilterName.is_null()
    {
        CloseHandle(hFltMgr);
        return HRESULT_FROM_WIN32!(ERROR_OUTOFMEMORY);
    }

    /* Build up the filter name */
    (*FilterName).Length = StringLength as USHORT;
    CopyMemory!(core::ptr::addr_of_mut!((*FilterName).FilterName) as *mut WCHAR as *mut core::ffi::c_void,
               lpFilterName as *const core::ffi::c_void,
               StringLength);

    /* Tell the filter manager to load the filter for us */
    dwError = SendIoctl(hFltMgr,
                        if Load != 0 { (IOCTL_FILTER_LOAD as u32) } else { (IOCTL_FILTER_UNLOAD as u32) },
                        (FilterName).cast(),
                        (BufferLength) as DWORD);

    /* Cleanup and bail*/
    RtlFreeHeap(GetProcessHeap(), 0, FilterName as PVOID);
    CloseHandle(hFltMgr);

    return HRESULT_FROM_WIN32!(dwError);
}

unsafe extern "C" fn SendIoctl(
    mut Handle: HANDLE,
    mut IoControlCode: ULONG,
    mut lpInBuffer: LPVOID,
    mut BufferSize: DWORD,
) -> DWORD {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtDeviceIoControlFile(
        Handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        IoControlCode,
        lpInBuffer as PVOID,
        BufferSize as ULONG,
        core::ptr::null_mut(),
        0,
    );
    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(
            Handle,
            (FALSE) as BOOLEAN,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS!(Status) {
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
    }

    return RtlNtStatusToDosError(Status) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
