use crate::prelude::*;
use crate::shellrecyclebin::RecycleBin5::WIN32_FIND_DATAW;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PRecycleBin5 = *mut RecycleBin5;

#[repr(C)]
pub struct RecycleBin5 {
    pub lpVtbl: *const IRecycleBin5Vtbl,
    pub m_ref: LONG,
    pub m_hInfo: HANDLE,
    pub m_hInfoMapped: HANDLE,
    pub m_EnumeratorCount: DWORD,
    pub m_VolumePath: CStringW,
    pub m_Folder: CStringW,
}

unsafe extern "C" fn IntDeleteRecursive(FullName: LPCWSTR) -> BOOL {
    let RemovableAttributes: DWORD = FILE_ATTRIBUTE_READONLY;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hSearch: HANDLE = INVALID_HANDLE_VALUE;
    let mut FullPath: LPWSTR = core::ptr::null_mut();
    let mut pFilePart: LPWSTR;
    let mut FileAttributes: DWORD;
    let mut dwLength: usize;
    let mut ret: BOOL = FALSE;
    'cleanup: {
        FileAttributes = GetFileAttributesW(FullName);
        if FileAttributes == INVALID_FILE_ATTRIBUTES {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                ret = TRUE;
            }
            break 'cleanup;
        }
        if (FileAttributes & RemovableAttributes) != 0 {
            if SetFileAttributesW(FullName, FileAttributes & !RemovableAttributes) == 0 {
                break 'cleanup;
            }
        }
        if (FileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            dwLength = wcslen(FullName) as usize;
            let alloc_size: usize = (dwLength + 1 + MAX_PATH + 1) * core::mem::size_of::<u16>();
            FullPath = HeapAlloc(GetProcessHeap(), 0, alloc_size) as LPWSTR;
            if FullPath.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            wcscpy(FullPath, FullName);
            if *FullPath.add(dwLength - 1) as u16 != '\\' as u16 {
                *FullPath.add(dwLength) = '\\' as u16;
                dwLength += 1;
            }
            pFilePart = FullPath.add(dwLength);
            wcscpy(pFilePart, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());
            hSearch = FindFirstFileW(FullPath, &mut FindData);
            if hSearch == INVALID_HANDLE_VALUE {
                break 'cleanup;
            }
            loop {
                let skip = FindData.cFileName[0 as usize] as u16 == '.' as u16 &&
                    (FindData.cFileName[1 as usize] as u16 == 0u16 ||
                     (FindData.cFileName[1 as usize] as u16 == '.' as u16 &&
                      FindData.cFileName[2 as usize] as u16 == 0u16));
                if !skip {
                    wcscpy(pFilePart, core::ptr::addr_of!(FindData.cFileName) as *const u16);
                    if IntDeleteRecursive(FullPath) == 0 {
                        FindClose(hSearch);
                        break 'cleanup;
                    }
                }
                if FindNextFileW(hSearch, &mut FindData) == 0 {
                    break;
                }
            }
            FindClose(hSearch);
            if GetLastError() != ERROR_NO_MORE_FILES {
                break 'cleanup;
            }
            if RemoveDirectoryW(FullName) == 0 {
                break 'cleanup;
            }
        } else {
            if DeleteFileW(FullName) == 0 {
                break 'cleanup;
            }
        }
        ret = TRUE;
    }
    HeapFree(GetProcessHeap(), 0, FullPath);
    ret
}
unsafe extern "C" fn RecycleBin5_Create(Folder: LPCWSTR, OwnerSid: PSID) -> HRESULT {
    let mut BufferName: LPWSTR = core::ptr::null_mut();
    let mut Separator: LPWSTR;
    let mut FileName: LPWSTR;
    let desktop_ini_contents: LPCSTR = b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n\0".as_ptr() as LPCSTR;
    let info2_contents: [u32; 5] = [
        5,
        0,
        0,
        0x320,
        0,
    ];
    let mut hr: HRESULT;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;

    'cleanup: loop {
        let needed = (wcslen(Folder) as usize + 1 as usize
            + max(wcslen(RECYCLE_BIN_FILE_NAME), wcslen(CPP_COM_WSTR_A5DBFDEBD401.as_ptr())) as usize
            + 1 as usize) * core::mem::size_of::<u16>();
        BufferName = HeapAlloc(GetProcessHeap(), 0, needed) as LPWSTR;
        if BufferName.is_null() {
            hr = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        wcscpy(BufferName, Folder);
        Separator = wcsstr(BufferName.wrapping_add(3 as usize), CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        if !Separator.is_null() {
            (*Separator) = UNICODE_NULL;
        }
        if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0 && GetLastError() != ERROR_ALREADY_EXISTS {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);
        if !Separator.is_null() {
            (*Separator) = UNICODE_NULL;
            if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0 && GetLastError() != ERROR_ALREADY_EXISTS {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'cleanup;
            }
        }

        wcscat(BufferName, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        FileName = BufferName.wrapping_add(wcslen(BufferName) as usize);

        wcscpy(FileName, CPP_COM_WSTR_A5DBFDEBD401.as_ptr());
        hFile = CreateFileW(BufferName, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN, core::ptr::null_mut());
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        let bytes_to_write_desktop = strlen(desktop_ini_contents) as DWORD;
        let mut bytes_written_desktop: DWORD = 0;
        if WriteFile(hFile, desktop_ini_contents as *const c_void, bytes_to_write_desktop, &mut bytes_written_desktop, core::ptr::null_mut()) == 0 {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        if bytes_written_desktop != bytes_to_write_desktop {
            hr = E_FAIL;
            break 'cleanup;
        }
        CloseHandle(hFile);
        hFile = INVALID_HANDLE_VALUE;

        wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
        hFile = CreateFileW(BufferName, GENERIC_READ | GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_HIDDEN, core::ptr::null_mut());
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        let bytes_to_write_info2 = core::mem::size_of_val(&info2_contents) as DWORD;
        let mut bytes_written_info2: DWORD = 0;
        if WriteFile(hFile, &info2_contents as *const _ as *const c_void, bytes_to_write_info2, &mut bytes_written_info2, core::ptr::null_mut()) == 0 {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        if bytes_written_info2 == bytes_to_write_info2 {
            hr = S_OK;
        } else {
            hr = E_FAIL;
        }
        break 'cleanup;
    }

    HeapFree(GetProcessHeap(), 0, BufferName);
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBin5_Constructor(
    VolumePath: *const u16,
    ppUnknown: *mut *mut core::ffi::c_void,
) -> i32 {
    if ppUnknown.is_null() {
        return E_POINTER;
    }
    *ppUnknown = core::ptr::null_mut();
    let pThis: *mut RecycleBin5 = RecycleBin5();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = unsafe { RecycleBin5::Init(pThis, VolumePath) };
    if hr < 0 {
        unsafe { RecycleBin5::delete(pThis) };
        return hr;
    }
    *ppUnknown = pThis as *mut core::ffi::c_void;
    S_OK
}
