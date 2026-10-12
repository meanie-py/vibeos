use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FilterCreate(lpFilterName: LPCWSTR, hFilter: *mut HFILTER) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(hFilter);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterClose(hFilter: HFILTER) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilter);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceCreate(
    lpFilterName: LPCWSTR,
    lpVolumeName: LPCWSTR,
    lpInstanceName: LPCWSTR,
    hInstance: *mut HFILTER_INSTANCE,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(lpInstanceName);
    UNREFERENCED_PARAMETER!(hInstance);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceClose(hInstance: HFILTER_INSTANCE) -> HRESULT {
    let _ = hInstance;
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterAttach(
    lpFilterName: LPCWSTR,
    lpVolumeName: LPCWSTR,
    lpInstanceName: LPCWSTR,
    dwCreatedInstanceNameLength: DWORD,
    lpCreatedInstanceName: LPWSTR,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(lpInstanceName);
    UNREFERENCED_PARAMETER!(dwCreatedInstanceNameLength);
    UNREFERENCED_PARAMETER!(lpCreatedInstanceName);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterAttachAtAltitude(
    lpFilterName: LPCWSTR,
    lpVolumeName: LPCWSTR,
    lpAltitude: LPCWSTR,
    lpInstanceName: LPCWSTR,
    dwCreatedInstanceNameLength: DWORD,
    lpCreatedInstanceName: LPWSTR,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(lpAltitude);
    UNREFERENCED_PARAMETER!(lpInstanceName);
    UNREFERENCED_PARAMETER!(dwCreatedInstanceNameLength);
    UNREFERENCED_PARAMETER!(lpCreatedInstanceName);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterDetach(
    lpFilterName: LPCWSTR,
    lpVolumeName: LPCWSTR,
    lpInstanceName: LPCWSTR,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(lpInstanceName);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterFindFirst(
    dwInformationClass: FILTER_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
    lpFilterFind: LPHANDLE,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    UNREFERENCED_PARAMETER!(lpFilterFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterFindNext(
    hFilterFind: HANDLE,
    dwInformationClass: FILTER_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilterFind);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterFindClose(hFilterFind: HANDLE) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilterFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeFindFirst(
    dwInformationClass: FILTER_VOLUME_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
    lpVolumeFind: PHANDLE,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    UNREFERENCED_PARAMETER!(lpVolumeFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeFindNext(
    hVolumeFind: HANDLE,
    dwInformationClass: FILTER_VOLUME_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hVolumeFind);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeFindClose(hVolumeFind: HANDLE) -> HRESULT {
    UNREFERENCED_PARAMETER!(hVolumeFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceFindFirst(
    lpFilterName: LPCWSTR,
    dwInformationClass: INSTANCE_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
    lpFilterInstanceFind: LPHANDLE,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpFilterName);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    UNREFERENCED_PARAMETER!(lpFilterInstanceFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceFindNext(
    hFilterInstanceFind: HANDLE,
    dwInformationClass: INSTANCE_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilterInstanceFind);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceFindClose(hFilterInstanceFind: HANDLE) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilterInstanceFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeInstanceFindFirst(
    lpVolumeName: LPCWSTR,
    dwInformationClass: INSTANCE_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
    lpVolumeInstanceFind: LPHANDLE,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    UNREFERENCED_PARAMETER!(lpVolumeInstanceFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeInstanceFindNext(
    hVolumeInstanceFind: HANDLE,
    dwInformationClass: INSTANCE_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hVolumeInstanceFind);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterVolumeInstanceFindClose(hVolumeInstanceFind: HANDLE) -> HRESULT {
    UNREFERENCED_PARAMETER!(hVolumeInstanceFind);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterGetInformation(
    hFilter: HFILTER,
    dwInformationClass: FILTER_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hFilter);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterInstanceGetInformation(
    hInstance: HFILTER_INSTANCE,
    dwInformationClass: INSTANCE_INFORMATION_CLASS,
    lpBuffer: LPVOID,
    dwBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hInstance);
    UNREFERENCED_PARAMETER!(dwInformationClass);
    UNREFERENCED_PARAMETER!(lpBuffer);
    UNREFERENCED_PARAMETER!(dwBufferSize);
    UNREFERENCED_PARAMETER!(lpBytesReturned);
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn FilterGetDosName(
    lpVolumeName: LPCWSTR,
    lpDosName: LPWSTR,
    dwDosNameBufferSize: DWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(lpVolumeName);
    UNREFERENCED_PARAMETER!(lpDosName);
    UNREFERENCED_PARAMETER!(dwDosNameBufferSize);
    return E_NOTIMPL;
}
