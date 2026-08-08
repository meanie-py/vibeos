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

unsafe extern "C" fn IntDeleteRecursive(FullName: *const u16) -> i32 {
    let RemovableAttributes = FILE_ATTRIBUTE_READONLY;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hSearch: isize = INVALID_HANDLE_VALUE;
    let mut FullPath: *mut u16 = core::ptr::null_mut();
    let pFilePart: *mut u16;
    let FileAttributes: u32;
    let dwLength: usize;
    let mut ret: i32 = FALSE;

    FileAttributes = GetFileAttributesW(FullName);
    if FileAttributes == INVALID_FILE_ATTRIBUTES {
        if GetLastError() == ERROR_FILE_NOT_FOUND {
            ret = TRUE;
        }
        // goto cleanup
    } else {
        if FileAttributes & RemovableAttributes != 0 {
            if SetFileAttributesW(FullName, FileAttributes & !RemovableAttributes) == 0 {
                // goto cleanup (via break after block)
            } else {
                // continue to next check
                if FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                    dwLength = wcslen(FullName) as usize;
                    FullPath = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        (dwLength + 1 + MAX_PATH + 1) * core::mem::size_of::<u16>(),
                    ) as *mut u16;
                    if FullPath.is_null() {
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        // goto cleanup
                    } else {
                        wcscpy(FullPath, FullName);
                        if *FullPath.add(dwLength - 1) != '\\' as u16 {
                            *FullPath.add(dwLength) = '\\' as u16;
                            dwLength += 1;
                        }
                        pFilePart = FullPath.add(dwLength);
                        wcscpy(pFilePart, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());

                        hSearch = FindFirstFileW(FullPath, &mut FindData) as isize;
                        if hSearch == INVALID_HANDLE_VALUE {
                            // goto cleanup
                        } else {
                            loop {
                                if !(*FindData.cFileName.as_ptr().add(0) == '.' as u16
                                    && (*FindData.cFileName.as_ptr().add(1) == 0u16
                                        || (*FindData.cFileName.as_ptr().add(1) == '.' as u16
                                            && *FindData.cFileName.as_ptr().add(2) == 0u16)))
                                {
                                    wcscpy(pFilePart, FindData.cFileName.as_ptr());
                                    if IntDeleteRecursive(FullPath) == 0 {
                                        FindClose(hSearch);
                                        // goto cleanup, but need to break outer and cleanup
                                        ret = 0; // ensure ret is FALSE
                                        // break to cleanup after freeing below
                                        break;
                                    }
                                }
                                if FindNextFileW(hSearch, &mut FindData) == 0 {
                                    break;
                                }
                            }
                            FindClose(hSearch);
                            if GetLastError() != ERROR_NO_MORE_FILES {
                                ret = 0; // keep FALSE
                                // goto cleanup
                            } else {
                                if RemoveDirectoryW(FullName) == 0 {
                                    ret = 0;
                                    // goto cleanup
                                } else {
                                    ret = TRUE;
                                }
                            }
                        }
                    }
                } else {
                    if DeleteFileW(FullName) == 0 {
                        ret = 0;
                        // goto cleanup
                    } else {
                        ret = TRUE;
                    }
                }
            }
        } else {
            // RemovableAttributes not set, but if directory fallback else delete file
            if FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                // same directory branch as above, but no SetFileAttributes call
                dwLength = wcslen(FullName) as usize;
                FullPath = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (dwLength + 1 + MAX_PATH + 1) * core::mem::size_of::<u16>(),
                ) as *mut u16;
                if FullPath.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    ret = 0;
                } else {
                    wcscpy(FullPath, FullName);
                    if *FullPath.add(dwLength - 1) != '\\' as u16 {
                        *FullPath.add(dwLength) = '\\' as u16;
                        dwLength += 1;
                    }
                    pFilePart = FullPath.add(dwLength);
                    wcscpy(pFilePart, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());

                    hSearch = FindFirstFileW(FullPath, &mut FindData) as isize;
                    if hSearch == INVALID_HANDLE_VALUE {
                        ret = 0;
                    } else {
                        loop {
                            if !(*FindData.cFileName.as_ptr().add(0) == '.' as u16
                                && (*FindData.cFileName.as_ptr().add(1) == 0u16
                                    || (*FindData.cFileName.as_ptr().add(1) == '.' as u16
                                        && *FindData.cFileName.as_ptr().add(2) == 0u16)))
                            {
                                wcscpy(pFilePart, FindData.cFileName.as_ptr());
                                if IntDeleteRecursive(FullPath) == 0 {
                                    FindClose(hSearch);
                                    ret = 0;
                                    break;
                                }
                            }
                            if FindNextFileW(hSearch, &mut FindData) == 0 {
                                break;
                            }
                        }
                        FindClose(hSearch);
                        if GetLastError() != ERROR_NO_MORE_FILES {
                            ret = 0;
                        } else if RemoveDirectoryW(FullName) == 0 {
                            ret = 0;
                        } else {
                            ret = TRUE;
                        }
                    }
                }
            } else {
                if DeleteFileW(FullName) == 0 {
                    ret = 0;
                } else {
                    ret = TRUE;
                }
            }
        }
    }

    // cleanup
    HeapFree(
        GetProcessHeap(),
        0,
        FullPath as *mut core::ffi::c_void,
    );
    ret
}
unsafe extern "C" fn RecycleBin5_Create(Folder: LPCWSTR, OwnerSid: PSID) -> HRESULT {
    let mut BufferName: LPWSTR = core::ptr::null_mut();
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let DesktopIniContents: *const u8 = b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n\0".as_ptr();
    let Info2Contents = [INFO2_HEADER(5, 0, 0, 0x320, 0); 1];
    let hr = 'body: loop {
        unsafe {
            let Needed = (wcslen(Folder) as usize + 1 as usize
                + max(wcslen(RECYCLE_BIN_FILE_NAME), wcslen(CPP_COM_WSTR_A5DBFDEBD401.as_ptr())) as usize
                + 1 as usize) * core::mem::size_of::<u16>();
            BufferName = HeapAlloc(GetProcessHeap(), 0, Needed as u32) as LPWSTR;
            if BufferName.is_null() {
                break 'body ERROR_NOT_ENOUGH_MEMORY;
            }
            wcscpy(BufferName, Folder);
            let Separator = wcsstr(BufferName.wrapping_add(3 as usize), CPP_COM_WSTR_B7FC02907B1F.as_ptr()) as *mut u16;
            if !Separator.is_null() {
                *Separator = UNICODE_NULL;
            }
            if !CreateDirectoryW(BufferName, core::ptr::null_mut()) && GetLastError() != ERROR_ALREADY_EXISTS {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
            SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);
            if !Separator.is_null() {
                *Separator = 0x005Cu16;
                if !CreateDirectoryW(BufferName, core::ptr::null_mut()) && GetLastError() != ERROR_ALREADY_EXISTS {
                    break 'body HRESULT_FROM_WIN32(GetLastError());
                }
            }
            wcscat(BufferName, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
            let FileName = BufferName.wrapping_add(wcslen(BufferName) as usize);
            // Create desktop.ini
            wcscpy(FileName, CPP_COM_WSTR_A5DBFDEBD401.as_ptr());
            hFile = CreateFileW(BufferName, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN, core::ptr::null_mut());
            if hFile == INVALID_HANDLE_VALUE {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
            let BytesToWrite = strlen(DesktopIniContents) as u32;
            let mut BytesWritten: u32 = 0;
            if WriteFile(hFile, DesktopIniContents as *const c_void, BytesToWrite, &mut BytesWritten, core::ptr::null_mut()) == 0 {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
            if BytesWritten != BytesToWrite {
                break 'body E_FAIL;
            }
            CloseHandle(hFile);
            hFile = INVALID_HANDLE_VALUE;
            // Create INFO2 file
            wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
            hFile = CreateFileW(BufferName, GENERIC_READ | GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_HIDDEN, core::ptr::null_mut());
            if hFile == INVALID_HANDLE_VALUE {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
            let BytesToWrite = core::mem::size_of::<INFO2_HEADER>() as u32;
            if WriteFile(hFile, Info2Contents.as_ptr() as *const c_void, BytesToWrite, &mut BytesWritten, core::ptr::null_mut()) == 0 {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
            if BytesWritten == BytesToWrite {
                break 'body S_OK;
            } else {
                break 'body E_FAIL;
            }
        }
    };
    unsafe {
        HeapFree(GetProcessHeap(), 0, BufferName);
        if hFile != INVALID_HANDLE_VALUE {
            CloseHandle(hFile);
        }
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
