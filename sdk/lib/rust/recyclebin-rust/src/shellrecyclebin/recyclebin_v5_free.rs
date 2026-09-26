use crate::prelude::*;
use crate::shellrecyclebin::RecycleBin5::RecycleBin5_new;
use crate::shellrecyclebin::RecycleBin5::RecycleBin5_Init_body;
use crate::shellrecyclebin::RecycleBin5::RecycleBin5_delete;
use crate::shellrecyclebin::RecycleBin5::WIN32_FIND_DATAW;
// recyclebin_private.h:26-28's #define's, as real UTF-16 constants -- the glob-imported
// `types_arch::RECYCLE_BIN_FILE_NAME` is a bindgen-decoded `&[u8; 6] = b"INFO2\0"` (an ASCII
// byte string, wrong unit entirely for `wcslen`/`wcscpy`'s LPCWSTR). This explicit import
// shadows that glob hit with the real wide-string pointer recyclebin_support.rs provides.
use crate::shellrecyclebin::recyclebin_support::RECYCLE_BIN_FILE_NAME;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

// NOTE: no local `RecycleBin5` preamble struct/`PRecycleBin5` alias here -- RecycleBin5_new()
// below returns the REAL `RecycleBin5` from RecycleBin5.rs (imported below); a same-named
// local duplicate would be a distinct nominal type and fail exactly like RecycleBin5Enum's
// `pThis` did (see RecycleBin5Enum.rs / recyclebin_v5_enumerator_free.rs for the same bug).
use crate::shellrecyclebin::RecycleBin5::RecycleBin5;

unsafe extern "C" fn IntDeleteRecursive(FullName: LPCWSTR) -> BOOL {
    let RemovableAttributes: DWORD = FILE_ATTRIBUTE_READONLY;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hSearch: HANDLE = INVALID_HANDLE_VALUE;
    let mut FullPath: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    'cleanup: {
        let FileAttributes = GetFileAttributesW(FullName);
        if FileAttributes == INVALID_FILE_ATTRIBUTES {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                ret = TRUE as BOOL;
            }
            break 'cleanup;
        }
        if FileAttributes & RemovableAttributes != 0 {
            if SetFileAttributesW(FullName, FileAttributes & !RemovableAttributes) == 0 {
                break 'cleanup;
            }
        }
        if FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            let dwLength = wcslen(FullName);
            FullPath = HeapAlloc(
                GetProcessHeap(),
                0,
                ((dwLength + 1 + MAX_PATH as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as LPWSTR;
            if FullPath.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            wcscpy(FullPath, FullName);
            let mut dwLength = dwLength;
            if *FullPath.offset((dwLength.wrapping_sub(1)) as isize) != '\\' as u16 {
                *FullPath.offset(dwLength as isize) = '\\' as u16;
                dwLength += 1;
            }
            let pFilePart = FullPath.offset(dwLength as isize);
            wcscpy(pFilePart, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());

            // `WIN32_FIND_DATAW` here is RecycleBin5.rs's own struct (imported above), a
            // distinct nominal type with the same layout as the bindgen `_WIN32_FIND_DATAW`
            // FindFirstFileW/FindNextFileW actually declare -- `.cast()` at the crossing,
            // same pattern RecycleBin5.rs itself uses (its FindFirstFileW/FindNextFileW calls).
            hSearch = FindFirstFileW(FullPath, core::ptr::addr_of_mut!(FindData).cast());
            if hSearch == INVALID_HANDLE_VALUE {
                break 'cleanup;
            }
            loop {
                if !(FindData.cFileName[0] == '.' as u16
                    && (FindData.cFileName[1] == 0
                        || (FindData.cFileName[1] == '.' as u16
                            && FindData.cFileName[2] == 0)))
                {
                    wcscpy(pFilePart, FindData.cFileName.as_ptr());
                    if IntDeleteRecursive(FullPath) == 0 {
                        FindClose(hSearch);
                        break 'cleanup;
                    }
                }
                if FindNextFileW(hSearch, core::ptr::addr_of_mut!(FindData).cast()) == 0 {
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
        ret = TRUE as BOOL;
    }

    HeapFree(GetProcessHeap(), 0, FullPath.cast());
    ret
}
unsafe extern "C" fn RecycleBin5_Create(
    Folder: LPCWSTR,
    OwnerSid: PSID,
) -> HRESULT {
    let mut BufferName: LPWSTR = core::ptr::null_mut();
    let desktop_ini_contents: LPCSTR =
        b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n\0".as_ptr() as LPCSTR;
    // recyclebin_v5.cpp:559 -- `{ 5, 0, 0, 0x320, 0 }` is positional-init in C; the Rust
    // struct literal needs field names (dwRecordSize=0x320 == sizeof(DELETED_FILE_RECORD)).
    let mut info2_contents: [_INFO2_HEADER; 1] = [_INFO2_HEADER {
        dwVersion: 5,
        dwNumberOfEntries: 0,
        dwHighestRecordUniqueId: 0,
        dwRecordSize: 0x320,
        dwTotalLogicalSize: 0,
    }];
    let mut BytesToWrite: u32;
    // C leaves `DWORD BytesWritten;` uninitialized (WriteFile always sets it on both success
    // paths below); zero-init is a safe superset and satisfies Rust's definite-init check for
    // the plain-value reads (`BytesWritten != BytesToWrite`) after each WriteFile out-param use.
    let mut BytesWritten: u32 = 0;
    let Needed: usize;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let hr: HRESULT;

    Needed = (wcslen(Folder)
        + 1
        + max(
            wcslen(RECYCLE_BIN_FILE_NAME),
            wcslen(CPP_COM_WSTR_A5DBFDEBD401.as_ptr()),
        )
        + 1)
        * core::mem::size_of::<u16>();
    BufferName = HeapAlloc(
        GetProcessHeap(),
        0,
        Needed as SIZE_T,
    ) as LPWSTR;

    hr = 'body: {
        if BufferName.is_null() {
            // recyclebin_v5.cpp:571 -- `hr = ERROR_NOT_ENOUGH_MEMORY;` assigns a raw Win32
            // error code straight into the HRESULT `hr`, with no HRESULT_FROM_WIN32() wrap
            // (unlike every other error path in this function). That looks like an upstream
            // bug, but it's the actual C++ behaviour, so it's preserved faithfully, not fixed.
            break 'body ERROR_NOT_ENOUGH_MEMORY as HRESULT;
        }

        let mut Separator: LPWSTR;
        let FileName: LPWSTR;

        wcscpy(BufferName, Folder);
        Separator = wcsstr(
            BufferName.add(3),
            CPP_COM_WSTR_B7FC02907B1F.as_ptr(),
        );
        if !Separator.is_null() {
            *Separator = UNICODE_NULL;
        }
        if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            break 'body HRESULT_FROM_WIN32(GetLastError());
        }
        SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);
        if !Separator.is_null() {
            *Separator = 0x005C_u16;
            if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0
                && GetLastError() != ERROR_ALREADY_EXISTS
            {
                break 'body HRESULT_FROM_WIN32(GetLastError());
            }
        }

        // OwnerSid block is commented out and skipped.

        wcscat(BufferName, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        FileName = BufferName.add(wcslen(BufferName));

        // Create desktop.ini
        wcscpy(FileName, CPP_COM_WSTR_A5DBFDEBD401.as_ptr());
        hFile = CreateFileW(
            BufferName,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            break 'body HRESULT_FROM_WIN32(GetLastError());
        }
        BytesToWrite = strlen(desktop_ini_contents) as u32;
        if WriteFile(
            hFile,
            desktop_ini_contents as *const c_void,
            BytesToWrite,
            core::ptr::addr_of_mut!(BytesWritten),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'body HRESULT_FROM_WIN32(GetLastError());
        }
        if BytesWritten != BytesToWrite {
            break 'body E_FAIL;
        }
        CloseHandle(hFile);
        hFile = INVALID_HANDLE_VALUE;

        // Create empty INFO2 file
        wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
        hFile = CreateFileW(
            BufferName,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            break 'body HRESULT_FROM_WIN32(GetLastError());
        }
        BytesToWrite = core::mem::size_of::<[_INFO2_HEADER; 1]>() as u32;
        if WriteFile(
            hFile,
            core::ptr::addr_of!(info2_contents[0]) as *const c_void,
            BytesToWrite,
            core::ptr::addr_of_mut!(BytesWritten),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'body HRESULT_FROM_WIN32(GetLastError());
        }
        if BytesWritten == BytesToWrite {
            break 'body S_OK;
        } else {
            break 'body E_FAIL;
        }
    };

    // cleanup
    HeapFree(GetProcessHeap(), 0, BufferName.cast());
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
    let pThis: *mut RecycleBin5 = RecycleBin5_new();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = RecycleBin5_Init_body(pThis, VolumePath);
    if hr < 0 {
        RecycleBin5_delete(pThis);
        return hr;
    }
    *ppUnknown = pThis as *mut core::ffi::c_void;
    S_OK
}
