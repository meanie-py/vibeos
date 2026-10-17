use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;

// WideCharToMultiByte + CP_THREAD_ACP come from crate::winbase (gdi32/kernel32/CRT glue).
use crate::winbase::{WideCharToMultiByte, CP_THREAD_ACP};
// GetUserObjectInformationW is the NtUserx* spec forwarder in crate::ntwrapper.
use crate::ntwrapper::GetUserObjectInformationW;

// ── No-op debug macro (TRACE compiles to nothing; consume args so they type-check).

// kernel32 heap + error glue (extern "system" = stdcall on i386).
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
}

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn GetUserObjectInformationA(
    hObj: HANDLE,
    nIndex: core::ffi::c_int,
    pvInfo: PVOID,
    nLength: DWORD,
    lpnLengthNeeded: LPDWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let LengthNeeded: DWORD;

    TRACE!(
        b"GetUserObjectInformationA(%x %d %x %d %x)\n\0".as_ptr() as PCHAR,
        hObj,
        nIndex,
        pvInfo,
        nLength,
        lpnLengthNeeded,
    );

    if nIndex != UOI_NAME as core::ffi::c_int && nIndex != UOI_TYPE as core::ffi::c_int {
        return GetUserObjectInformationW(hObj, nIndex, pvInfo, nLength, lpnLengthNeeded);
    }

    let buffer = HeapAlloc(GetProcessHeap(), 0, (nLength * 2) as SIZE_T) as LPWSTR;
    if buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if GetUserObjectInformationW(hObj, nIndex, buffer as PVOID, nLength * 2, lpnLengthNeeded) != 0 {
        LengthNeeded = WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            buffer,
            -1,
            pvInfo as LPSTR,
            nLength as core::ffi::c_int,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;

        if LengthNeeded != 0 {
            if !lpnLengthNeeded.is_null() {
                *lpnLengthNeeded = LengthNeeded;
            }
            ret = 1;
        }
    }

    HeapFree(GetProcessHeap(), 0, buffer as PVOID);
    ret
}
