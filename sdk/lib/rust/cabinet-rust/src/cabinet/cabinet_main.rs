use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};
use crate::types::*;

// ---- consts / statics / type aliases (deduped) ----
const E_INVALIDARG: i32 = 0x80070057u32 as i32;
const EXTRACT_FILLFILELIST: u32 = 0x00000001;
const EXTRACT_EXTRACTFILES: u32 = 0x00000002;
const S_OK: i32 = 0;
const _O_RDONLY: i32 = 0;
const _O_WRONLY: i32 = 1;
const _O_RDWR: i32 = 2;
const _O_ACCMODE: i32 = 3;
const _O_CREAT: i32 = 0x0100;
const _O_EXCL: i32 = 0x0400;
const _O_TRUNC: i32 = 0x0200;
const INVALID_FILE_ATTRIBUTES: DWORD = (-1i32) as DWORD;
const INVALID_HANDLE_VALUE: HANDLE = (-1isize) as HANDLE;

extern "C" {
}

extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn CreateFileA( lpFileName: LPCSTR, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut SECURITY_ATTRIBUTES, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: HANDLE, ) -> HANDLE;
    fn ReadFile( hFile: *mut core::ffi::c_void, lpBuffer: *mut core::ffi::c_void, nNumberOfBytesToRead: u32, lpNumberOfBytesRead: *mut u32, lpOverlapped: *mut core::ffi::c_void, ) -> i32;
    fn WriteFile(hFile: *mut core::ffi::c_void, lpBuffer: *const core::ffi::c_void, nNumberOfBytesToWrite: u32, lpNumberOfBytesWritten: *mut u32, lpOverlapped: *mut core::ffi::c_void) -> i32;
    fn SetFilePointer( hFile: *mut core::ffi::c_void, lDistanceToMove: i32, lpDistanceToMoveHigh: *mut i32, dwMoveMethod: u32, ) -> u32;
    fn lstrcpyA(lpString1: *mut i8, lpString2: *const i8) -> *mut i8;
    fn lstrcmpiA(lpStr1: *const i8, lpStr2: *const i8) -> i32;
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn CreateDirectoryA( lpPathName: LPCSTR, lpSecurityAttributes: *mut core::ffi::c_void, ) -> BOOL;
    fn DosDateTimeToFileTime( wFatDate: WORD, wFatTime: WORD, lpFileTime: *mut FILETIME, ) -> BOOL;
    fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD;
    fn LocalFileTimeToFileTime( lpLocalFileTime: *const FILETIME, lpFileTime: *mut FILETIME, ) -> BOOL;
    fn SetFileTime( hFile: HANDLE, lpCreationTime: *const FILETIME, lpLastAccessTime: *const FILETIME, lpLastWriteTime: *const FILETIME, ) -> BOOL;
    fn lstrcatA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR;
    fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: i32) -> LPSTR;
    fn lstrlenA(lpString: LPCSTR) -> i32;
}
unsafe extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn strrchr(s: *const i8, c: i32) -> *mut i8;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetVersion(pdvi: *mut DLLVERSIONINFO) -> i32 {
    winedebug_rust::WARN!("hmmm... not right version number \"5.1.1106.1\"?\n");
    if (*pdvi).cbSize as usize != core::mem::size_of::<DLLVERSIONINFO>() {
        return E_INVALIDARG;
    }
    (*pdvi).dwMajorVersion = 5;
    (*pdvi).dwMinorVersion = 1;
    (*pdvi).dwBuildNumber = 1106;
    (*pdvi).dwPlatformID = 1;
    return S_OK;
}

unsafe extern "C" fn mem_alloc(cb: u32) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0, cb as usize)
}

unsafe extern "C" fn mem_free(memory: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, memory);
}

unsafe extern "C" fn fdi_open(pszFile: *mut i8, oflag: i32, pmode: i32) -> isize {
    let mut handle: HANDLE;
    let mut dwAccess: u32 = 0;
    let mut dwShareMode: u32 = 0;
    let mut dwCreateDisposition: u32;

    match oflag & _O_ACCMODE {
        _O_RDONLY => {
            dwAccess = GENERIC_READ;
            dwShareMode = FILE_SHARE_READ | FILE_SHARE_DELETE;
        }
        _O_WRONLY => {
            dwAccess = GENERIC_WRITE;
            dwShareMode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
        }
        _O_RDWR => {
            dwAccess = GENERIC_READ | GENERIC_WRITE;
            dwShareMode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
        }
        _ => {}
    }

    if (oflag & _O_CREAT) != 0 {
        dwCreateDisposition = OPEN_ALWAYS;
        if (oflag & _O_EXCL) != 0 {
            dwCreateDisposition = CREATE_NEW;
        } else if (oflag & _O_TRUNC) != 0 {
            dwCreateDisposition = CREATE_ALWAYS;
        }
    } else {
        dwCreateDisposition = OPEN_EXISTING;
        if (oflag & _O_TRUNC) != 0 {
            dwCreateDisposition = TRUNCATE_EXISTING;
        }
    }

    handle = CreateFileA(
        pszFile,
        dwAccess,
        dwShareMode,
        core::ptr::null_mut(),
        dwCreateDisposition,
        0,
        core::ptr::null_mut(),
    );

    return handle as isize;
}

unsafe extern "C" fn fdi_read(hf: isize, pv: *mut core::ffi::c_void, cb: u32) -> u32 {
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    let mut dwRead: u32 = 0;
    if ReadFile(handle, pv, cb, &raw mut dwRead, core::ptr::null_mut()) != 0 {
        return dwRead;
    }
    return 0;
}

unsafe extern "C" fn fdi_write(hf: isize, pv: *mut core::ffi::c_void, cb: u32) -> u32 {
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    let mut dwWritten: u32 = 0;

    if WriteFile(handle, pv as *const core::ffi::c_void, cb, &raw mut dwWritten, core::ptr::null_mut()) != 0 {
        return dwWritten;
    }

    0
}

unsafe extern "C" fn fdi_close(hf: isize) -> i32 {
    extern "system" {
        fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
    }
    let handle: *mut core::ffi::c_void = hf as *mut core::ffi::c_void;
    if CloseHandle(handle) != 0 { 0 } else { -1 }
}

unsafe extern "C" fn fdi_seek(hf: isize, dist: i32, seektype: i32) -> i32 {
    let handle = hf as *mut core::ffi::c_void;
    return SetFilePointer(handle, dist, core::ptr::null_mut(), seektype as u32) as i32;
}

unsafe fn fill_file_node(pNode: *mut FILELIST, szFilename: *const i8) {
    (*pNode).next = core::ptr::null_mut();
    (*pNode).DoExtract = 0;
    let len = strlen(szFilename);
    let alloc_size = len + 1;
    let heap = GetProcessHeap();
    let alloc = HeapAlloc(heap, 0, alloc_size);
    (*pNode).FileName = alloc as *mut i8;
    lstrcpyA((*pNode).FileName, szFilename);
}

unsafe fn file_in_list(mut pNode: *mut FILELIST, szFilename: *const i8, pOut: *mut *mut FILELIST) -> i32 {
    while !pNode.is_null() {
        if lstrcmpiA((*pNode).FileName, szFilename) == 0 {
            if !pOut.is_null() {
                *pOut = pNode;
            }
            return 1;
        }
        pNode = (*pNode).next;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn Extract(dest: *mut SESSION, szCabName: *const i8) -> i32 {
    // external declarations
    extern "system" {
        fn GetFileAttributesA(lpFileName: *const i8) -> u32;
        fn GetLastError() -> u32;
        fn GetProcessHeap() -> *mut core::ffi::c_void;
        fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: u32) -> *mut core::ffi::c_void;
        fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
        fn lstrcpyA(lpString1: *mut i8, lpString2: *const i8) -> *mut i8;
        fn lstrlenA(lpString: *const i8) -> i32;
    }
    extern "C" {
        fn FDICreate(
            pfnalloc: unsafe extern "C" fn(u32) -> *mut core::ffi::c_void,
            pfnfree: unsafe extern "C" fn(*mut core::ffi::c_void),
            pfnopen: unsafe extern "C" fn(*mut i8, i32, i32) -> isize,
            pfnread: unsafe extern "C" fn(isize, *mut core::ffi::c_void, u32) -> u32,
            pfnwrite: unsafe extern "C" fn(isize, *mut core::ffi::c_void, u32) -> u32,
            pfnclose: unsafe extern "C" fn(isize) -> i32,
            pfnseek: unsafe extern "C" fn(isize, i32, i32) -> i32,
            cpuType: u32,
            perf: *mut ERF,
        ) -> *mut core::ffi::c_void;
        fn FDICopy(
            hfdi: *mut core::ffi::c_void,
            pszCabinet: *const i8,
            pszCabPath: *const i8,
            flags: i32,
            pfnNotify: *const core::ffi::c_void,
            pfnDecrypt: *const core::ffi::c_void,
            pvUser: *mut core::ffi::c_void,
        ) -> i32;
        fn FDIDestroy(hfdi: *mut core::ffi::c_void) -> i32;
        fn debugstr_a(s: *const i8) -> *const i8;
        fn strrchr(s: *const i8, c: i32) -> *mut i8;
        fn strlen(s: *const i8) -> u32;
        fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
        fn HRESULT_FROM_WIN32(error: u32) -> i32;
        // mem_alloc/mem_free/fdi_* and fdi_notify_extract are defined locally in
        // this file; re-declaring them extern here shadowed the local defs and
        // produced undefined _mem_alloc / _fdi_read / ... symbols. Removed so the
        // FDICreate(...) call binds to the real local fns.
    }

    const S_OK: i32 = 0x00000000;
    const E_FAIL: i32 = 0x80004005u32 as i32;
    const E_OUTOFMEMORY: i32 = 0x8007000Eu32 as i32;
    const INVALID_FILE_ATTRIBUTES: u32 = 0xFFFFFFFF;
    const cpuUNKNOWN: u32 = 0;

    let mut res: i32 = S_OK;
    let mut hfdi: *mut core::ffi::c_void;
    let mut str: *mut i8;
    let mut end: *mut i8;
    let mut path: *mut i8 = core::ptr::null_mut();
    let mut name: *mut i8 = core::ptr::null_mut();

    winedebug_rust::TRACE!("(%p, %s)\n", dest, debugstr_a(szCabName));

    hfdi = FDICreate(
        mem_alloc,
        mem_free,
        fdi_open,
        fdi_read,
        fdi_write,
        fdi_close,
        fdi_seek,
        cpuUNKNOWN,
        core::ptr::addr_of_mut!((*dest).Error),
    );

    if hfdi.is_null() {
        return E_FAIL;
    }

    'end: loop {
        if GetFileAttributesA((*dest).Destination.as_ptr()) == INVALID_FILE_ATTRIBUTES {
            res = S_OK;
            break 'end;
        }

        // split the cabinet name into path + name
        str = HeapAlloc(GetProcessHeap(), 0, (lstrlenA(szCabName) + 1) as u32) as *mut i8;
        if str.is_null() {
            res = E_OUTOFMEMORY;
            break 'end;
        }
        lstrcpyA(str, szCabName);

        end = strrchr(str, '\\' as i32);
        if !end.is_null() {
            path = str;
            end = end.add(1); // end++
            let len = strlen(end as *const i8) + 1;
            name = HeapAlloc(GetProcessHeap(), 0, len) as *mut i8;
            if name.is_null() {
                res = E_OUTOFMEMORY;
                break 'end;
            }
            strcpy(name, end);
            *end = 0;
        } else {
            name = str;
            path = core::ptr::null_mut();
        }

        (*dest).FileSize = 0;

        if FDICopy(
            hfdi,
            name,
            path,
            0,
            fdi_notify_extract as *const core::ffi::c_void,
            core::ptr::null(),
            dest as *mut core::ffi::c_void,
        ) == 0
        {
            res = HRESULT_FROM_WIN32(GetLastError());
        }

        break 'end; // fall through to cleanup
    }

    HeapFree(GetProcessHeap(), 0, path as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, name as *mut core::ffi::c_void);
    FDIDestroy(hfdi);
    return res;
}

unsafe extern "C" fn fdi_notify_extract(
    fdint: FDINOTIFICATIONTYPE,
    pfdin: *mut FDINOTIFICATION,
) -> INT_PTR {
    match fdint {
        fdintCOPY_FILE => {
            let mut fileList: *mut FILELIST;
            let mut node: *mut FILELIST = core::ptr::null_mut();
            let pDestination: *mut SESSION = (*pfdin).pv as *mut SESSION;
            let szFullPath: LPSTR;
            let szDirectory: LPSTR;
            let mut hFile: HANDLE = core::ptr::null_mut();
            let mut dwSize: DWORD;

            dwSize = (lstrlenA((*pDestination).Destination.as_ptr())
                + lstrlenA(b"\\\0".as_ptr() as LPCSTR)
                + lstrlenA((*pfdin).psz1)
                + 1) as u32;
            szFullPath = HeapAlloc(GetProcessHeap(), 0, dwSize as usize) as LPSTR;

            lstrcpyA(szFullPath, (*pDestination).Destination.as_ptr());
            lstrcatA(szFullPath, b"\\\0".as_ptr() as LPCSTR);
            lstrcatA(szFullPath, (*pfdin).psz1);

            /* pull out the destination directory string from the full path */
            let diff = (strrchr(szFullPath, '\\' as i32) as usize)
                .wrapping_sub(szFullPath as usize);
            dwSize = (diff + 1) as DWORD;
            szDirectory = HeapAlloc(GetProcessHeap(), 0, dwSize as usize) as LPSTR;
            lstrcpynA(szDirectory, szFullPath, dwSize as i32);

            (*pDestination).FileSize += (*pfdin).cb;

            if ((*pDestination).Operation & EXTRACT_FILLFILELIST as i32) != 0 {
                fileList = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    core::mem::size_of::<FILELIST>(),
                ) as *mut FILELIST;

                fill_file_node(fileList, (*pfdin).psz1);
                (*fileList).DoExtract = 1;
                (*fileList).next = (*pDestination).FileList;
                (*pDestination).FileList = fileList;
                lstrcpyA((*pDestination).CurrentFile.as_mut_ptr(), szFullPath);
                (*pDestination).FileCount += 1;
            }

            if ((*pDestination).Operation & EXTRACT_EXTRACTFILES as i32) != 0
                || file_in_list(
                    (*pDestination).FilterList,
                    (*pfdin).psz1,
                    core::ptr::null_mut(),
                ) != 0
            {
                /* find the file node */
                file_in_list(
                    (*pDestination).FileList,
                    (*pfdin).psz1,
                    core::ptr::addr_of_mut!(node),
                );

                if !node.is_null() && (*node).DoExtract == 0 {
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        szFullPath as *mut core::ffi::c_void,
                    );
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        szDirectory as *mut core::ffi::c_void,
                    );
                    return 0;
                }

                /* create the destination directory if it doesn't exist */
                if GetFileAttributesA(szDirectory) == INVALID_FILE_ATTRIBUTES {
                    let mut ptr: *mut i8;
                    ptr = szDirectory
                        .wrapping_add(strlen((*pDestination).Destination.as_ptr()) + 1);
                    while *ptr != 0 {
                        if *ptr as u8 == b'\\' {
                            *ptr = 0;
                            CreateDirectoryA(
                                szDirectory,
                                core::ptr::null_mut(),
                            );
                            *ptr = '\\' as i8;
                        }
                        ptr = ptr.wrapping_add(1);
                    }
                    CreateDirectoryA(szDirectory, core::ptr::null_mut());
                }

                hFile = CreateFileA(
                    szFullPath,
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    core::ptr::null_mut(),
                    CREATE_ALWAYS,
                    FILE_ATTRIBUTE_NORMAL,
                    core::ptr::null_mut(),
                );

                if hFile != INVALID_HANDLE_VALUE && !node.is_null() {
                    (*node).DoExtract = 0;
                }
            }

            HeapFree(
                GetProcessHeap(),
                0,
                szFullPath as *mut core::ffi::c_void,
            );
            HeapFree(
                GetProcessHeap(),
                0,
                szDirectory as *mut core::ffi::c_void,
            );

            return hFile as INT_PTR;
        }

        fdintCLOSE_FILE_INFO => {
            let mut ft: FILETIME = core::mem::zeroed();
            let mut ftLocal: FILETIME = core::mem::zeroed();
            let handle: HANDLE = (*pfdin).hf as HANDLE;

            if DosDateTimeToFileTime((*pfdin).date, (*pfdin).time, &mut ft) == 0 {
                return 0;
            }
            if LocalFileTimeToFileTime(&ft, &mut ftLocal) == 0 {
                return 0;
            }
            if SetFileTime(
                handle,
                &ftLocal,
                core::ptr::null_mut(),
                &ftLocal,
            ) == 0
            {
                return 0;
            }

            CloseHandle(handle);
            return 1;
        }

        _ => return 0,
    }
}

