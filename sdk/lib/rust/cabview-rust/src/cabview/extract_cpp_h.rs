use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_28449638299A: [WCHAR; 3] = [0x003F, 0x003A, 0x0000];

use core::ffi::c_void;



unsafe extern "C" fn BuildPath(Dir: LPCWSTR, File: LPCSTR, Attr: UINT) -> LPWSTR {
    let cp: UINT = if (Attr & _A_NAME_IS_UTF) != 0 { CP_UTF8 } else { CP_ACP };
    let cchfile: UINT = MultiByteToWideChar(cp, 0, File, -1, core::ptr::null_mut(), 0) as UINT;
    let mut lendir: SIZE_T = lstrlenW(Dir) as SIZE_T;
    let cch: SIZE_T = lendir + 1 + cchfile as SIZE_T;
    let path: LPWSTR = SHAlloc(cch * (core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
    if !path.is_null() {
        lstrcpyW(path, Dir);
        if lendir != 0 && IsPathSep__T0(*path.add((lendir - 1) as _)) == false {
            *path.add((lendir) as _) = 0x005Cu16;
            lendir += 1;
        }

        let dst: LPWSTR = path.add((lendir) as _);
        MultiByteToWideChar(cp, 0, File.add(IsPathSep__T1(*File) as usize), -1, dst, cchfile as i32);
        let mut i: SIZE_T = 0;
        while *dst.add((i) as _) != 0 {
            if *dst.add((i) as _) == 0x003Au16 && lendir != 0 {
                *dst.add((i) as _) = 0x005Fu16;
            }
            if *dst.add((i) as _) == 0x002Fu16 {
                *dst.add((i) as _) = 0x005Cu16;
            }
            i += 1;
        }
    }
    path
}

unsafe extern "C" fn HResultFrom(erf: *const ERF) -> HRESULT {
    match ((if (*erf).fError != 0 { (*erf).erfOper } else { (FDIERROR_NONE) as i32 })) as u32 {
        FDIERROR_NONE => {
            return if (*erf).fError != 0 { HRESULT_FROM_WIN32!((*erf).erfType) } else { S_OK };
        }
        FDIERROR_CABINET_NOT_FOUND => {
            return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
        }
        FDIERROR_ALLOC_FAIL => {
            return E_OUTOFMEMORY;
        }
        FDIERROR_USER_ABORT => {
            return S_FALSE;
        }
        _ => {
            return if (*erf).erfType != 0 { HRESULT_FROM_WIN32!((*erf).erfType) } else { E_FAIL };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CabMemFree(pv: *mut core::ffi::c_void) {
    SHFree(pv);
}

#[no_mangle]
pub unsafe extern "C" fn CabMemAlloc(cb: ULONG) -> *mut core::ffi::c_void {
    SHAlloc((cb) as SIZE_T)
}
#[no_mangle]
pub unsafe extern "C" fn CabClose(hf: INT_PTR) -> i32 {
    if CloseHandle(hf as HANDLE) != 0 { 0 } else { -1 }
}

unsafe extern "C" fn CabOpenEx(path: LPCWSTR, access: u32, share: u32, disp: u32, attr: u32) -> INT_PTR {
    return CreateFileW(path, access, share, core::ptr::null_mut(), disp, attr, core::ptr::null_mut()) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CabOpen(pszFile: *mut i8, oflag: i32, pmode: i32) -> INT_PTR {
    let disp: UINT = if (oflag & (_O_CREAT as i32)) != 0 { CREATE_ALWAYS } else { OPEN_EXISTING };
    let mut access: UINT = GENERIC_READ;
    if (oflag & (_O_RDWR as i32)) != 0 {
        access = GENERIC_READ | GENERIC_WRITE;
    } else if (oflag & (_O_WRONLY as i32)) != 0 {
        access = GENERIC_WRITE;
    }
    UNREFERENCED_PARAMETER!(pmode);
    let mut buf = [0u16; (MAX_PATH * 2) as _];
    MultiByteToWideChar(CP_UTF8, 0, pszFile, -1, buf.as_mut_ptr(), _countof!(buf) as i32);
    return CabOpenEx(buf.as_ptr(), access, FILE_SHARE_READ, disp, FILE_ATTRIBUTE_NORMAL);
}

#[no_mangle]
pub unsafe extern "C" fn CabRead(hf: INT_PTR, pv: LPVOID, cb: UINT) -> UINT {
    let mut dwBytesRead: DWORD = 0;
    return if ReadFile(hf as HANDLE, pv, cb, core::ptr::addr_of_mut!(dwBytesRead), core::ptr::null_mut()) != 0 {
        dwBytesRead
    } else {
        0u32.wrapping_sub(1 as UINT)
    };
}
#[no_mangle]
pub unsafe extern "C" fn CabWrite(hf: INT_PTR, pv: *mut core::ffi::c_void, cb: UINT) -> UINT {
    let mut dwBytesWritten: DWORD = 0;
    return if WriteFile(hf as HANDLE, pv, cb, core::ptr::addr_of_mut!(dwBytesWritten), core::ptr::null_mut()) != 0 { dwBytesWritten } else { (-(1 as i32)) as UINT };
}
#[no_mangle]
pub unsafe extern "C" fn CabSeek(hf: INT_PTR, dist: LONG, seektype: i32) -> LONG {
    return (SetFilePointer(hf as HANDLE, dist, core::ptr::null_mut(), (seektype) as DWORD)) as LONG;
}

unsafe extern "C" fn Init(hfdi: *mut HFDI, erf: *mut ERF) -> HRESULT {
    let cpu: i32 = cpuUNKNOWN;
    *hfdi = FDICreate(Some(CabMemAlloc), Some(CabMemFree), Some(CabOpen), Some(CabRead), Some(CabWrite), Some(CabClose), Some(CabSeek), cpu, core::ptr::addr_of_mut!(*erf));
    return if !(*hfdi).is_null() { S_OK } else { HResultFrom(erf) };
}
#[no_mangle]
pub unsafe extern "C" fn ExtractCabinetCallback(fdint: FDINOTIFICATIONTYPE, pfdin: PFDINOTIFICATION) -> INT_PTR {
    let attrmask: UINT = FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_ARCHIVE;
    let ecd: &mut EXTRACTCABINETINTERNALDATA = &mut *((*pfdin).pv as *mut EXTRACTCABINETINTERNALDATA);
    let mut noti: EXTRACTCALLBACKDATA = EXTRACTCALLBACKDATA__make_0();
    let mut hr: HRESULT;
    let mut ft: FILETIME = core::mem::zeroed();

    noti.pfdin = pfdin;
    match fdint {
        fdintCOPY_FILE => {
            hr = ((ecd.callback).unwrap_unchecked())(ECM_FILE, &noti, ecd.cookie);
            if hr == S_OK {
                hr = E_OUTOFMEMORY;
                let path: LPWSTR = BuildPath(ecd.destination, (*pfdin).psz1, ((*pfdin).attribs) as UINT);
                if !path.is_null() {
                    // Callee is using SHPPFW_IGNOREFILENAME so we don't need to remove the name.
                    /*LPWSTR file = PathFindFileNameW(path);
                    if (file > path)
                    {
                        file[-1] = L'\0';*/
                        noti.Path = path as LPCWSTR;
                        ((ecd.callback).unwrap_unchecked())(ECM_PREPAREPATH, &noti, ecd.cookie);
                    /*  file[-1] = L'\\';
                    }*/
                    let attr: UINT = (*pfdin).attribs as UINT & attrmask;
                    let access: UINT = GENERIC_READ | GENERIC_WRITE;
                    let share: UINT = FILE_SHARE_DELETE;
                    let handle: INT_PTR = CabOpenEx(path as LPCWSTR, access, share, CREATE_NEW, attr | FILE_FLAG_SEQUENTIAL_SCAN);
                    noti.hr = HResultFromWin32(GetLastError());
                    SHFree(path as LPVOID);
                    if handle != (-(1 as INT_PTR)) {
                        return handle;
                    }
                    if ((ecd.callback).unwrap_unchecked())(ECM_ERROR, &noti, ecd.cookie) != E_NOTIMPL {
                        hr = noti.hr;
                    }
                }
            }
            return if hr == S_FALSE {
                0 as INT_PTR
            } else {
                -(1 as INT_PTR)
            };
        }

        fdintCLOSE_FILE_INFO => {
            if DosDateTimeToFileTime((*pfdin).date, (*pfdin).time, core::ptr::addr_of_mut!(ft)) != 0 {
                SetFileTime((*pfdin).hf as HANDLE, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of!(ft));
            }
            return (CabClose((*pfdin).hf) == 0) as INT_PTR;
        }

        fdintNEXT_CABINET => {
            if (*pfdin).fdie != 0 && (*pfdin).fdie != FDIERROR_USER_ABORT {
                if (*pfdin).fdie == FDIERROR_CABINET_NOT_FOUND {
                    noti.hr = HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
                } else {
                    noti.hr = HRESULT_FROM_WIN32!(ERROR_INVALID_DATA);
                }
                ((ecd.callback).unwrap_unchecked())(ECM_ERROR, &noti, ecd.cookie);
            }
            return if (*pfdin).fdie != 0 {
                -(1 as INT_PTR)
            } else {
                0 as INT_PTR
            };
        }

        fdintPARTIAL_FILE => {
            return 0 as INT_PTR;
        }

        fdintCABINET_INFO => {
            return 0 as INT_PTR;
        }

        fdintENUMERATE => {
            return 0 as INT_PTR;
        }

        _ => {}
    }
    return -(1 as INT_PTR);
}
#[no_mangle]
pub unsafe extern "C" fn ExtractCabinet(cab: LPCWSTR, mut destination: LPCWSTR, callback: EXTRACTCALLBACK, cookie: LPVOID) -> HRESULT {
    let quick: BOOL = if destination.is_null() { 1 } else { 0 };
    if destination.is_null() {
        destination = CPP_COM_WSTR_28449638299A.as_ptr(); // Dummy path for callers that enumerate without extracting
    }
    let mut data: EXTRACTCABINETINTERNALDATA = EXTRACTCABINETINTERNALDATA { destination: destination, callback: callback, cookie: cookie };
    let mut noti: EXTRACTCALLBACKDATA = EXTRACTCALLBACKDATA__make_0();
    let mut erf: ERF = core::mem::zeroed();
    let mut hfdi: HFDI = core::mem::zeroed();
    let mut total: UINT = 0;
    let mut files: UINT = 0;
    let mut hr: HRESULT = Init(&mut hfdi, &mut erf);
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr;
    }

    let share: UINT = FILE_SHARE_READ | FILE_SHARE_DELETE;
    let mut hf: INT_PTR = if quick != 0 { -1 } else { CabOpenEx(cab, GENERIC_READ, share, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL) };
    if hf != -1 {
        let mut ci: FDICABINETINFO = core::mem::zeroed();
        if FDIIsCabinet(hfdi, hf, core::ptr::addr_of_mut!(ci)) != 0 {
            total = (ci.cbCabinet) as u32;
            files = (ci.cFiles) as _;
        }
        CabClose(hf);
    }

    hr = HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    let mut buf: [i8; (MAX_PATH * 2) as _] = [0i8; (MAX_PATH * 2) as _];
    let mut name: *mut i8 = core::ptr::null_mut();
    if WideCharToMultiByte(CP_UTF8, 0, cab, -1, buf.as_mut_ptr(), (_countof!(buf)) as i32, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        buf[0] = 0;
        hr = E_INVALIDARG;
    }
    {
        let mut i: UINT = 0;
        while buf[i as usize] != 0 {
            if buf[i as usize] == '\\' as i8 || buf[i as usize] == '/' as i8 {
                name = buf.as_mut_ptr().add((i + 1) as usize);
            }
            i += 1;
        }
    }
    if name > buf.as_mut_ptr() && *name != 0 {
        // Format the name the way FDI likes it
        *name.offset(-(1 as isize)) = ANSI_NULL as i8;
        let mut namebuf: [i8; (MAX_PATH) as _] = [0i8; (MAX_PATH) as _];
        namebuf[0 as usize] = '\\' as i8;
        lstrcpyA(namebuf.as_mut_ptr().add(1 as usize), name);
        name = namebuf.as_mut_ptr();

        let mut fdin: FDINOTIFICATION = core::mem::zeroed();
        fdin.cb = total as _;
        fdin.hf = files as _;
        noti.Path = cab;
        noti.pfdin = core::ptr::addr_of!(fdin);
        (callback.unwrap_unchecked())(ECM_BEGIN, &noti, cookie);

        hr = if FDICopy(hfdi, name, buf.as_mut_ptr(), 0, Some(ExtractCabinetCallback), None, core::ptr::addr_of_mut!(data) as *mut c_void) != 0 { S_OK } else { HResultFrom(&erf) };
    }
    FDIDestroy(hfdi);
    return hr;
}

impl EXTRACTCABINETINTERNALDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
