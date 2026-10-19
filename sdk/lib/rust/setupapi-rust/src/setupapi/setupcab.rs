use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut OsVersionInfo: OSVERSIONINFOEXW = _OSVERSIONINFOEXW {
    dwOSVersionInfoSize: 0,
    dwMajorVersion: 0,
    dwMinorVersion: 0,
    dwBuildNumber: 0,
    dwPlatformId: 0,
    szCSDVersion: [0; 128],
    wServicePackMajor: 0,
    wServicePackMinor: 0,
    wSuiteMask: 0,
    wProductType: 0,
    wReserved: 0,
};

static mut CABINET_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut sc_FDICreate: Option<
    unsafe extern "C" fn(
        PFNALLOC,
        PFNFREE,
        PFNOPEN,
        PFNREAD,
        PFNWRITE,
        PFNCLOSE,
        PFNSEEK,
        ::core::ffi::c_int,
        PERF,
    ) -> HFDI,
> = None;

static mut sc_FDICopy: Option<
    unsafe extern "C" fn(
        HFDI,
        *mut ::core::ffi::c_char,
        *mut ::core::ffi::c_char,
        ::core::ffi::c_int,
        PFNFDINOTIFY,
        PFNFDIDECRYPT,
        *mut ::core::ffi::c_void,
    ) -> BOOL,
> = None;

static mut sc_FDIDestroy: Option<unsafe extern "C" fn(HFDI) -> BOOL> = None;

pub const SC_HSC_A_MAGIC: u32 = 0xACABFEED;

#[repr(C)]
pub struct SC_HSC_A {
    pub magic: UINT,
    pub hfdi: HFDI,
    pub msghandler: PSP_FILE_CALLBACK_A,
    pub context: PVOID,
    pub most_recent_cabinet_name: [CHAR; MAX_PATH as usize],
    pub most_recent_target: [CHAR; MAX_PATH as usize],
}
pub type PSC_HSC_A = *mut SC_HSC_A;

pub const SC_HSC_W_MAGIC: u32 = 0x0CABFEED;

#[repr(C)]
pub struct SC_HSC_W {
    pub magic: UINT,
    pub hfdi: HFDI,
    pub msghandler: PSP_FILE_CALLBACK_W,
    pub context: PVOID,
    pub most_recent_cabinet_name: [WCHAR; MAX_PATH as usize],
    pub most_recent_target: [WCHAR; MAX_PATH as usize],
}
pub type PSC_HSC_W = *mut SC_HSC_W;

pub const SIZEOF_MYSTERIO: usize = (MAX_PATH as usize) * 3;

unsafe extern "C" fn LoadCABINETDll() -> BOOL {
    if CABINET_hInstance.is_null() {
        CABINET_hInstance = LoadLibraryA(b"cabinet.dll\0".as_ptr() as PCSTR);
        if !CABINET_hInstance.is_null() {
            sc_FDICreate = ::core::mem::transmute::<
                *mut ::core::ffi::c_void,
                Option<
                    unsafe extern "C" fn(
                        PFNALLOC,
                        PFNFREE,
                        PFNOPEN,
                        PFNREAD,
                        PFNWRITE,
                        PFNCLOSE,
                        PFNSEEK,
                        ::core::ffi::c_int,
                        PERF,
                    ) -> HFDI,
                >,
            >(::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(
                GetProcAddress(CABINET_hInstance, b"FDICreate\0".as_ptr() as PCSTR),
            ));
            sc_FDICopy = ::core::mem::transmute::<
                *mut ::core::ffi::c_void,
                Option<
                    unsafe extern "C" fn(
                        HFDI,
                        *mut ::core::ffi::c_char,
                        *mut ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        PFNFDINOTIFY,
                        PFNFDIDECRYPT,
                        *mut ::core::ffi::c_void,
                    ) -> BOOL,
                >,
            >(::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(
                GetProcAddress(CABINET_hInstance, b"FDICopy\0".as_ptr() as PCSTR),
            ));
            sc_FDIDestroy = ::core::mem::transmute::<
                *mut ::core::ffi::c_void,
                Option<unsafe extern "C" fn(HFDI) -> BOOL>,
            >(::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(
                GetProcAddress(CABINET_hInstance, b"FDIDestroy\0".as_ptr() as PCSTR),
            ));
            return (TRUE) as BOOL;
        } else {
            ERR!("load cabinet dll failed.\n");
            return (FALSE) as BOOL;
        }
    } else {
        return (TRUE) as BOOL;
    }
}

unsafe extern "C" fn sc_cb_alloc(cb: ULONG) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), 0, (cb) as SIZE_T) as *mut core::ffi::c_void;
}

unsafe extern "C" fn sc_cb_free(pv: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 as DWORD, pv as LPVOID);
}

unsafe extern "C" fn sc_cb_open(pszFile: *mut i8, oflag: i32, pmode: i32) -> INT_PTR {
    let mut creation: DWORD = 0;
    let mut sharing: DWORD = 0;
    let mut ioflag: i32 = 0;
    let mut ret: INT_PTR = 0;
    let mut sa: SECURITY_ATTRIBUTES = core::mem::zeroed();

    /* TRACE("(pszFile == %s, oflag == %d, pmode == %d)\n", debugstr_a(pszFile), oflag, pmode); */

    let m1: i32 = oflag & (_O_RDONLY | _O_WRONLY | _O_RDWR);
    if m1 == _O_RDONLY {
        ioflag |= (GENERIC_READ) as i32;
    } else if m1 == _O_WRONLY {
        ioflag |= (GENERIC_WRITE as i32);
    } else if m1 == _O_RDWR {
        ioflag |= (GENERIC_READ | GENERIC_WRITE) as i32;
    } else if m1 == (_O_WRONLY | _O_RDWR) {
        /* hmmm.. */
        ERR!("_O_WRONLY & _O_RDWR in oflag?\n");
        return -(1) as INT_PTR;
    }

    if (oflag & (_O_CREAT as i32)) != 0 {
        if (oflag & (_O_EXCL as i32)) != 0 {
            creation = CREATE_NEW;
        } else if (oflag & (_O_TRUNC as i32)) != 0 {
            creation = CREATE_ALWAYS;
        } else {
            creation = OPEN_ALWAYS;
        }
    } else {
        /* no _O_CREAT */
        if (oflag & (_O_TRUNC as i32)) != 0 {
            creation = TRUNCATE_EXISTING;
        } else {
            creation = OPEN_EXISTING;
        }
    }

    let m2: i32 = pmode & 0x70;
    if m2 == _SH_DENYRW {
        sharing = 0;
    } else if m2 == _SH_DENYWR {
        sharing = FILE_SHARE_READ;
    } else if m2 == _SH_DENYRD {
        sharing = FILE_SHARE_WRITE;
    } else if m2 == _SH_COMPAT || m2 == _SH_DENYNO {
        sharing = FILE_SHARE_READ | FILE_SHARE_WRITE;
    } else {
        ERR!("<-- -1 (Unhandled pmode 0x%x)\n", pmode);
        return -(1) as INT_PTR;
    }

    if ((oflag as u32) & !(_O_BINARY | _O_TRUNC | _O_EXCL | _O_CREAT | (_O_RDWR as u32) | (_O_WRONLY as u32) | _O_NOINHERIT)) != 0 {
        WARN!("unsupported oflag 0x%04x\n", oflag);
    }

    sa.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    sa.lpSecurityDescriptor = core::ptr::null_mut();
    // Divergence: the C tests `ioflag`, which by now holds GENERIC_READ/WRITE bits, never
    // _O_NOINHERIT, so every handle came out inheritable. The CRT flag is in oflag.
    sa.bInheritHandle = ((oflag & (_O_NOINHERIT as i32)) == 0) as i32;

    ret = CreateFileA(
        pszFile as PCSTR,
        ioflag as DWORD,
        sharing,
        core::ptr::addr_of_mut!(sa),
        creation,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    ) as INT_PTR;

    /* TRACE("<-- %d\n", ret); */

    return ret;
}

unsafe extern "C" fn sc_cb_read(hf: INT_PTR, pv: *mut ::core::ffi::c_void, cb: UINT) -> UINT {
    let mut num_read: DWORD = 0;
    let mut rslt: BOOL;

    /* TRACE("(hf == %d, pv == ^%p, cb == %u)\n", hf, pv, cb); */

    rslt = ReadFile(
        hf as HANDLE,
        pv,
        cb,
        core::ptr::addr_of_mut!(num_read),
        core::ptr::null_mut(),
    );

    /* eof and failure both give "-1" return */
    if rslt == 0 || (cb > 0 && num_read == 0) {
        /* TRACE("<-- -1\n"); */
        return (-(1 as i32)) as UINT;
    }

    /* TRACE("<-- %lu\n", num_read); */
    return num_read as UINT;
}

unsafe extern "C" fn sc_cb_write(hf: INT_PTR, pv: *mut core::ffi::c_void, cb: UINT) -> UINT {
    let mut num_written: DWORD = 0;
    if WriteFile(
        hf as HANDLE,
        pv as LPCVOID,
        cb as DWORD,
        core::ptr::addr_of_mut!(num_written),
        core::ptr::null_mut(),
    ) != 0
        && num_written == cb as DWORD
    {
        return num_written as UINT;
    } else {
        return (-(1 as i32)) as UINT;
    }
}

unsafe extern "C" fn sc_cb_close(mut hf: INT_PTR) -> i32 {
    if CloseHandle(hf as HANDLE) != 0 {
        return 0;
    } else {
        return -1;
    }
}

unsafe extern "C" fn sc_cb_lseek(hf: INT_PTR, dist: LONG, seektype: i32) -> LONG {
    let mut ret: DWORD = 0;

    if seektype < 0 || seektype > 2 {
        return -(1) as LONG;
    }

    if {
        ret = SetFilePointer(hf as HANDLE, dist, core::ptr::null_mut(), seektype as DWORD);
        ret != INVALID_SET_FILE_POINTER
    } || GetLastError() == 0 {
        return ret as LONG;
    } else {
        return -(1) as LONG;
    }
}

unsafe extern "C" fn sc_FNNOTIFY_A(mut fdint: FDINOTIFICATIONTYPE, mut pfdin: PFDINOTIFICATION) -> INT_PTR {
    let mut fici: FILE_IN_CABINET_INFO_A = core::mem::zeroed();
    let mut phsc: PSC_HSC_A = core::ptr::null_mut();
    let mut ci: CABINET_INFO_A = core::mem::zeroed();
    let mut fp: FILEPATHS_A = core::mem::zeroed();
    let mut err: UINT = 0;

    let mut mysterio: [CHAR; SIZEOF_MYSTERIO] = [0; SIZEOF_MYSTERIO];

    memset(
        core::ptr::addr_of_mut!(mysterio) as *mut CHAR as *mut ::core::ffi::c_void,
        (0) as ::core::ffi::c_int,
        (SIZEOF_MYSTERIO) as size_t,
    );

    if !pfdin.is_null()
        && !(*pfdin).pv.is_null()
        && (*((*pfdin).pv as PSC_HSC_A)).magic == (SC_HSC_A_MAGIC) as UINT
    {
        phsc = (*pfdin).pv as PSC_HSC_A;
    } else {
        ERR!(
            "pv %p is not an SC_HSC_A.\n",
            if !pfdin.is_null() {
                (*pfdin).pv
            } else {
                core::ptr::null_mut::<::core::ffi::c_void>()
            }
        );
        return -((1) as INT_PTR);
    }

    match fdint as ::core::ffi::c_uint {
        0 => {
            TRACE!("Cabinet info notification\n");
            WARN!("SPFILENOTIFY_CABINETINFO undocumented: guess implementation.\n");
            ci.CabinetFile = core::ptr::addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut CHAR as PCSTR;
            ci.CabinetPath = (*pfdin).psz3 as PCSTR;
            ci.DiskName = (*pfdin).psz2 as PCSTR;
            ci.SetId = (*pfdin).setID;
            ci.CabinetNumber = (*pfdin).iCabinet;
            (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                (SPFILENOTIFY_CABINETINFO) as UINT,
                core::ptr::addr_of_mut!(ci) as UINT_PTR,
                (0) as UINT_PTR,
            );
            return (0) as INT_PTR;
        }
        1 => {
            TRACE!("Partial file notification\n");
            return (0) as INT_PTR;
        }
        2 => {
            TRACE!("Copy file notification\n");
            fici.NameInCabinet = (*pfdin).psz1 as PCSTR;
            fici.FileSize = (*pfdin).cb as DWORD;
            fici.Win32Error = (0) as DWORD;
            fici.DosDate = (*pfdin).date as WORD;
            fici.DosTime = (*pfdin).time as WORD;
            fici.DosAttribs = (*pfdin).attribs as WORD;
            memset(
                core::ptr::addr_of_mut!(fici.FullTargetName) as *mut CHAR as *mut ::core::ffi::c_void,
                (0) as ::core::ffi::c_int,
                (MAX_PATH) as size_t,
            );
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                (SPFILENOTIFY_FILEINCABINET) as UINT,
                core::ptr::addr_of_mut!(fici) as UINT_PTR,
                (*pfdin).psz1 as UINT_PTR,
            );
            if err == (FILEOP_DOIT) as UINT {
                if fici.FullTargetName[(0) as usize] == 0 {
                    WARN!("  Empty return string causing abort.\n");
                    SetLastError((ERROR_PATH_NOT_FOUND) as DWORD);
                    return -((1) as INT_PTR);
                }
                strcpy(
                    core::ptr::addr_of_mut!((*phsc).most_recent_target) as *mut CHAR,
                    core::ptr::addr_of_mut!(fici.FullTargetName) as *mut CHAR,
                );
                return sc_cb_open(
                    core::ptr::addr_of_mut!(fici.FullTargetName) as *mut CHAR as *mut ::core::ffi::c_char,
                    ((_O_BINARY) | (_O_CREAT) | ((_O_WRONLY)) as u32) as ::core::ffi::c_int,
                    ((_S_IREAD) | (_S_IWRITE)) as ::core::ffi::c_int,
                );
            } else {
                return (0) as INT_PTR;
            }
        }
        3 => {
            TRACE!("Close file notification\n");
            fp.Source = core::ptr::addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut CHAR as PCSTR;
            fp.Target = core::ptr::addr_of_mut!((*phsc).most_recent_target) as *mut CHAR as PCSTR;
            fp.Win32Error = (0) as UINT;
            fp.Flags = (0) as DWORD;
            WARN!("Should set file date/time/attribs (and execute files?)\n");
            if sc_cb_close((*pfdin).hf) != 0 {
                WARN!("_close failed.\n");
            }
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                (SPFILENOTIFY_FILEEXTRACTED) as UINT,
                core::ptr::addr_of_mut!(fp) as UINT_PTR,
                (0) as UINT_PTR,
            );
            if err != 0 {
                SetLastError(err as DWORD);
                return (FALSE) as INT_PTR;
            } else {
                return (TRUE) as INT_PTR;
            }
        }
        4 => {
            TRACE!("Next cabinet notification\n");
            ci.CabinetFile = (*pfdin).psz1 as PCSTR;
            ci.CabinetPath = (*pfdin).psz3 as PCSTR;
            ci.DiskName = (*pfdin).psz2 as PCSTR;
            ci.SetId = (*pfdin).setID;
            ci.CabinetNumber = (*pfdin).iCabinet;
            strcpy(
                core::ptr::addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut CHAR,
                (*pfdin).psz1 as PCSTR,
            );
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                (SPFILENOTIFY_NEEDNEWCABINET) as UINT,
                core::ptr::addr_of_mut!(ci) as UINT_PTR,
                core::ptr::addr_of_mut!(mysterio) as *mut CHAR as UINT_PTR,
            );
            if err != 0 {
                SetLastError(err as DWORD);
                return -((1) as INT_PTR);
            } else {
                if mysterio[(0) as usize] != 0 {
                    lstrcpynA(
                        (*pfdin).psz3 as LPSTR,
                        core::ptr::addr_of_mut!(mysterio) as *mut CHAR as LPCSTR,
                        (SIZEOF_MYSTERIO) as ::core::ffi::c_int,
                    );
                }
                return (0) as INT_PTR;
            }
        }
        _ => {
            FIXME!("Unknown notification type %d.\n", fdint as ::core::ffi::c_uint);
            return (0) as INT_PTR;
        }
    }
}

unsafe extern "C" fn sc_FNNOTIFY_W(fdint: FDINOTIFICATIONTYPE, pfdin: PFDINOTIFICATION) -> INT_PTR {
    let mut fici: FILE_IN_CABINET_INFO_W = core::mem::zeroed();
    let mut phsc: PSC_HSC_W = core::ptr::null_mut();
    let mut ci: CABINET_INFO_W = core::mem::zeroed();
    let mut fp: FILEPATHS_W = core::mem::zeroed();
    let mut err: UINT = 0;
    let mut len: core::ffi::c_int = 0;

    let mut mysterio: [WCHAR; SIZEOF_MYSTERIO] = [0; SIZEOF_MYSTERIO];
    let mut buf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut buf2: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut charbuf: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    memset(
        addr_of_mut!(mysterio) as *mut core::ffi::c_void,
        0,
        (MAX_PATH as usize * 3) * core::mem::size_of::<WCHAR>(),
    );
    memset(
        addr_of_mut!(buf) as *mut core::ffi::c_void,
        0,
        (MAX_PATH as usize) * core::mem::size_of::<WCHAR>(),
    );
    memset(
        addr_of_mut!(buf2) as *mut core::ffi::c_void,
        0,
        (MAX_PATH as usize) * core::mem::size_of::<WCHAR>(),
    );
    memset(
        addr_of_mut!(charbuf) as *mut core::ffi::c_void,
        0,
        (MAX_PATH) as _,
    );

    if !pfdin.is_null()
        && !(*pfdin).pv.is_null()
        && (*((*pfdin).pv as PSC_HSC_W)).magic == SC_HSC_W_MAGIC as UINT
    {
        phsc = (*pfdin).pv as PSC_HSC_W;
    } else {
        ERR!(
            "pv %p is not an SC_HSC_W.\n",
            if !pfdin.is_null() {
                (*pfdin).pv
            } else {
                core::ptr::null_mut()
            }
        );
        return -(1);
    }

    match fdint as UINT {
        0 => {
            TRACE!("Cabinet info notification\n");
            WARN!("SPFILENOTIFY_CABINETINFO undocumented: guess implementation.\n");
            ci.CabinetFile = addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut WCHAR as PCWSTR;
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz3 as LPCSTR,
                    -(1),
                    addr_of_mut!(buf) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                buf[0] = 0 as WCHAR;
            }
            ci.CabinetPath = addr_of_mut!(buf) as *mut WCHAR as PCWSTR;
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz2 as LPCSTR,
                    -(1),
                    addr_of_mut!(buf2) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                buf2[0] = 0 as WCHAR;
            }
            ci.DiskName = addr_of_mut!(buf2) as *mut WCHAR as PCWSTR;
            ci.SetId = (*pfdin).setID;
            ci.CabinetNumber = (*pfdin).iCabinet;
            (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                SPFILENOTIFY_CABINETINFO as UINT,
                addr_of_mut!(ci) as UINT_PTR,
                0 as UINT_PTR,
            );
            return 0;
        }
        1 => {
            TRACE!("Partial file notification\n");
            return 0;
        }
        2 => {
            TRACE!("Copy file notification\n");
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz1 as LPCSTR,
                    -(1),
                    addr_of_mut!(buf2) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                buf2[0] = 0 as WCHAR;
            }
            fici.NameInCabinet = addr_of_mut!(buf2) as *mut WCHAR as PCWSTR;
            fici.FileSize = (*pfdin).cb as DWORD;
            fici.Win32Error = 0 as DWORD;
            fici.DosDate = (*pfdin).date as WORD;
            fici.DosTime = (*pfdin).time as WORD;
            fici.DosAttribs = (*pfdin).attribs as WORD;
            memset(
                addr_of_mut!(fici.FullTargetName) as *mut core::ffi::c_void,
                0,
                (MAX_PATH as usize) * core::mem::size_of::<WCHAR>(),
            );
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                SPFILENOTIFY_FILEINCABINET as UINT,
                addr_of_mut!(fici) as UINT_PTR,
                (*pfdin).psz1 as UINT_PTR,
            );
            if err == FILEOP_DOIT as UINT {
                if fici.FullTargetName[0] != 0 {
                    len = strlenW(addr_of_mut!(fici.FullTargetName) as *mut WCHAR) as core::ffi::c_int
                        + 1;
                    if (len > (MAX_PATH as i32)) || (len <= 1) {
                        return 0;
                    }
                    if WideCharToMultiByte(
                        CP_ACP as UINT,
                        0 as UINT,
                        addr_of_mut!(fici.FullTargetName) as LPCWSTR,
                        len,
                        addr_of_mut!(charbuf) as LPSTR,
                        (MAX_PATH as i32),
                        core::ptr::null::<CHAR>(),
                        core::ptr::null_mut::<BOOL>(),
                    ) == 0
                    {
                        return 0;
                    }
                } else {
                    WARN!("Empty buffer string caused abort.\n");
                    SetLastError(ERROR_PATH_NOT_FOUND as DWORD);
                    return -(1);
                }
                strcpyW(
                    addr_of_mut!((*phsc).most_recent_target) as *mut WCHAR,
                    addr_of_mut!(fici.FullTargetName) as *mut WCHAR,
                );
                return sc_cb_open(
                    addr_of_mut!(charbuf) as *mut core::ffi::c_char,
                    (_O_BINARY | _O_CREAT | (_O_WRONLY as u32)) as i32,
                    (_S_IREAD | _S_IWRITE) as i32,
                );
            } else {
                return 0;
            }
        }
        3 => {
            TRACE!("Close file notification\n");
            fp.Source = addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut WCHAR as PCWSTR;
            fp.Target = addr_of_mut!((*phsc).most_recent_target) as *mut WCHAR as PCWSTR;
            fp.Win32Error = 0 as UINT;
            fp.Flags = 0 as DWORD;
            if sc_cb_close((*pfdin).hf) != 0 {
                WARN!("_close failed.\n");
            }
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                SPFILENOTIFY_FILEEXTRACTED as UINT,
                addr_of_mut!(fp) as UINT_PTR,
                0 as UINT_PTR,
            );
            if err != 0 {
                SetLastError(err as DWORD);
                return (FALSE) as INT_PTR;
            } else {
                return (TRUE) as INT_PTR;
            }
        }
        4 => {
            TRACE!("Next cabinet notification\n");
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz1 as LPCSTR,
                    -(1),
                    addr_of_mut!((*phsc).most_recent_cabinet_name) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                (*phsc).most_recent_cabinet_name[0] = 0 as WCHAR;
            }
            ci.CabinetFile = addr_of_mut!((*phsc).most_recent_cabinet_name) as *mut WCHAR as PCWSTR;
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz3 as LPCSTR,
                    -(1),
                    addr_of_mut!(buf) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                buf[0] = 0 as WCHAR;
            }
            ci.CabinetPath = addr_of_mut!(buf) as *mut WCHAR as PCWSTR;
            len = 1
                + MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*pfdin).psz2 as LPCSTR,
                    -(1),
                    addr_of_mut!(buf2) as LPWSTR,
                    (MAX_PATH as i32),
                );
            if (len > (MAX_PATH as i32)) || (len <= 1) {
                buf2[0] = 0 as WCHAR;
            }
            ci.DiskName = addr_of_mut!(buf2) as *mut WCHAR as PCWSTR;
            ci.SetId = (*pfdin).setID;
            ci.CabinetNumber = (*pfdin).iCabinet;
            err = (*phsc).msghandler.unwrap_unchecked()(
                (*phsc).context,
                SPFILENOTIFY_NEEDNEWCABINET as UINT,
                addr_of_mut!(ci) as UINT_PTR,
                addr_of_mut!(mysterio) as *mut WCHAR as UINT_PTR,
            );
            if err != 0 {
                SetLastError(err as DWORD);
                return -(1);
            } else {
                if mysterio[0] != 0 {
                    len = strlenW(addr_of_mut!(mysterio) as *mut WCHAR) as core::ffi::c_int
                        + 1;
                    if (len > 255) || (len <= 1) {
                        return 0;
                    }
                    if WideCharToMultiByte(
                        CP_ACP as UINT,
                        0 as UINT,
                        addr_of_mut!(mysterio) as LPCWSTR,
                        len,
                        (*pfdin).psz3 as LPSTR,
                        255,
                        core::ptr::null::<CHAR>(),
                        core::ptr::null_mut::<BOOL>(),
                    ) == 0
                    {
                        return 0;
                    }
                }
                return 0;
            }
        }
        _ => {
            FIXME!("Unknown notification type %d.\n", fdint as UINT);
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetupIterateCabinetA(
    CabinetFile: PCSTR,
    Reserved: DWORD,
    MsgHandler: PSP_FILE_CALLBACK_A,
    Context: PVOID,
) -> BOOL {
    let mut my_hsc: SC_HSC_A = SC_HSC_A {
        magic: 0,
        hfdi: core::ptr::null_mut(),
        msghandler: None,
        context: core::ptr::null_mut(),
        most_recent_cabinet_name: [0; MAX_PATH as usize],
        most_recent_target: [0; MAX_PATH as usize],
    };
    let mut erf: ERF = ERF {
        erfOper: 0,
        erfType: 0,
        fError: 0,
    };
    let mut pszCabinet: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pszCabPath: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut p: *mut CHAR = core::ptr::null_mut();
    let mut fpnsize: DWORD = 0;
    let mut ret: BOOL = 0;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/setupcab.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupIterateCabinetA\0" as *const u8 as *const core::ffi::c_char,
            521 as core::ffi::c_int,
            b"(CabinetFile == %s, Reserved == %u, MsgHandler == ^%p, Context == ^%p)\n\0"
                as *const u8 as *const core::ffi::c_char,
            debugstr_a(CabinetFile as *const core::ffi::c_char),
            Reserved,
            MsgHandler,
            Context,
        );
    }

    if LoadCABINETDll() == 0 {
        return (FALSE) as BOOL;
    }

    if CabinetFile.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    fpnsize = strlen(CabinetFile as *const core::ffi::c_char) as DWORD;
    if fpnsize >= MAX_PATH as DWORD {
        SetLastError(ERROR_BAD_PATHNAME as DWORD);
        return (FALSE) as BOOL;
    }

    fpnsize = GetFullPathNameA(
        CabinetFile as LPCSTR,
        MAX_PATH as DWORD,
        core::ptr::addr_of_mut!(pszCabPath) as LPSTR,
        core::ptr::addr_of_mut!(p),
    );
    if fpnsize > MAX_PATH as DWORD {
        SetLastError(ERROR_BAD_PATHNAME as DWORD);
        return (FALSE) as BOOL;
    }

    if !p.is_null() {
        strcpy(
            core::ptr::addr_of_mut!(pszCabinet) as *mut core::ffi::c_char,
            p,
        );
        *p = 0 as CHAR;
    } else {
        strcpy(
            core::ptr::addr_of_mut!(pszCabinet) as *mut core::ffi::c_char,
            CabinetFile as *const core::ffi::c_char,
        );
        pszCabPath[0] = 0 as CHAR;
    }

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/setupcab.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupIterateCabinetA\0" as *const u8 as *const core::ffi::c_char,
            552 as core::ffi::c_int,
            b"path: %s, cabfile: %s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_a(core::ptr::addr_of_mut!(pszCabPath) as *mut CHAR),
            debugstr_a(core::ptr::addr_of_mut!(pszCabinet) as *mut CHAR),
        );
    }

    strcpy(
        core::ptr::addr_of_mut!(my_hsc.most_recent_cabinet_name) as *mut core::ffi::c_char,
        core::ptr::addr_of_mut!(pszCabinet) as *mut CHAR,
    );

    my_hsc.magic = SC_HSC_A_MAGIC as UINT;
    my_hsc.msghandler = MsgHandler;
    my_hsc.context = Context;
    my_hsc.hfdi = (sc_FDICreate.unwrap_unchecked())(
        Some(sc_cb_alloc as unsafe extern "C" fn(ULONG) -> *mut core::ffi::c_void),
        Some(sc_cb_free as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        Some(
            sc_cb_open
                as unsafe extern "C" fn(
                    *mut core::ffi::c_char,
                    core::ffi::c_int,
                    core::ffi::c_int,
                ) -> INT_PTR,
        ),
        Some(sc_cb_read as unsafe extern "C" fn(INT_PTR, *mut core::ffi::c_void, UINT) -> UINT),
        Some(sc_cb_write as unsafe extern "C" fn(INT_PTR, *mut core::ffi::c_void, UINT) -> UINT),
        Some(sc_cb_close as unsafe extern "C" fn(INT_PTR) -> core::ffi::c_int),
        Some(sc_cb_lseek as unsafe extern "C" fn(INT_PTR, LONG, core::ffi::c_int) -> LONG),
        cpuUNKNOWN,
        core::ptr::addr_of_mut!(erf),
    );

    if my_hsc.hfdi.is_null() {
        return (FALSE) as BOOL;
    }

    ret = (sc_FDICopy.unwrap_unchecked())(
        my_hsc.hfdi,
        core::ptr::addr_of_mut!(pszCabinet) as *mut core::ffi::c_char,
        core::ptr::addr_of_mut!(pszCabPath) as *mut core::ffi::c_char,
        0 as core::ffi::c_int,
        Some(
            sc_FNNOTIFY_A as unsafe extern "C" fn(FDINOTIFICATIONTYPE, PFDINOTIFICATION) -> INT_PTR,
        ),
        None,
        core::ptr::addr_of_mut!(my_hsc) as *mut core::ffi::c_void,
    );

    (sc_FDIDestroy.unwrap_unchecked())(my_hsc.hfdi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupIterateCabinetW(
    CabinetFile: PCWSTR,
    Reserved: DWORD,
    MsgHandler: PSP_FILE_CALLBACK_W,
    Context: PVOID,
) -> BOOL {
    let mut pszCabinet: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pszCabPath: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut len: UINT = 0;
    let mut my_hsc: SC_HSC_W = core::mem::zeroed();
    let mut erf: ERF = core::mem::zeroed();
    let mut pszCabPathW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut fpnsize: DWORD = 0;
    let mut ret: BOOL = 0;

    if LoadCABINETDll() == 0 {
        return (FALSE) as BOOL;
    }

    if CabinetFile.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    fpnsize = GetFullPathNameW(
        CabinetFile,
        MAX_PATH,
        core::ptr::addr_of_mut!(pszCabPathW) as *mut WCHAR,
        core::ptr::addr_of_mut!(p),
    );
    if fpnsize > MAX_PATH {
        SetLastError(ERROR_BAD_PATHNAME);
        return (FALSE) as BOOL;
    }

    if !p.is_null() {
        strcpyW(core::ptr::addr_of_mut!(my_hsc.most_recent_cabinet_name) as *mut WCHAR, p);
        *p = 0;
        len = WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(pszCabPathW) as *mut WCHAR,
            -1,
            core::ptr::addr_of_mut!(pszCabPath) as *mut CHAR,
            (MAX_PATH as i32),
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as UINT;
        if len == 0 {
            return (FALSE) as BOOL;
        }
    } else {
        strcpyW(
            core::ptr::addr_of_mut!(my_hsc.most_recent_cabinet_name) as *mut WCHAR,
            CabinetFile,
        );
        pszCabPath[0 as usize] = 0 as CHAR;
    }

    len = WideCharToMultiByte(
        CP_ACP,
        0,
        core::ptr::addr_of_mut!(my_hsc.most_recent_cabinet_name) as *mut WCHAR,
        -1,
        core::ptr::addr_of_mut!(pszCabinet) as *mut CHAR,
        (MAX_PATH as i32),
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as UINT;
    if len == 0 {
        return (FALSE) as BOOL;
    }

    my_hsc.magic = SC_HSC_W_MAGIC;
    my_hsc.msghandler = MsgHandler;
    my_hsc.context = Context;
    my_hsc.hfdi = sc_FDICreate.unwrap_unchecked()(
        Some(sc_cb_alloc as unsafe extern "C" fn(ULONG) -> *mut ::core::ffi::c_void),
        Some(sc_cb_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        Some(
            sc_cb_open
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> INT_PTR,
        ),
        Some(sc_cb_read as unsafe extern "C" fn(INT_PTR, *mut ::core::ffi::c_void, UINT) -> UINT),
        Some(sc_cb_write as unsafe extern "C" fn(INT_PTR, *mut ::core::ffi::c_void, UINT) -> UINT),
        Some(sc_cb_close as unsafe extern "C" fn(INT_PTR) -> ::core::ffi::c_int),
        Some(sc_cb_lseek as unsafe extern "C" fn(INT_PTR, LONG, ::core::ffi::c_int) -> LONG),
        cpuUNKNOWN,
        core::ptr::addr_of_mut!(erf),
    );

    if my_hsc.hfdi.is_null() {
        return (FALSE) as BOOL;
    }

    ret = sc_FDICopy.unwrap_unchecked()(
        my_hsc.hfdi,
        core::ptr::addr_of_mut!(pszCabinet) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(pszCabPath) as *mut ::core::ffi::c_char,
        0,
        Some(
            sc_FNNOTIFY_W as unsafe extern "C" fn(FDINOTIFICATIONTYPE, PFDINOTIFICATION) -> INT_PTR,
        ),
        None,
        core::ptr::addr_of_mut!(my_hsc) as *mut ::core::ffi::c_void,
    );

    sc_FDIDestroy.unwrap_unchecked()(my_hsc.hfdi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            OsVersionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD;
            if !NT_SUCCESS!(RtlGetVersion(core::ptr::addr_of_mut!(OsVersionInfo) as PRTL_OSVERSIONINFOW)) {
                return (FALSE) as BOOL;
            }
            hInstance = hinstDLL;
        }
        DLL_PROCESS_DETACH => {
            'brk: {
                if !lpvReserved.is_null() {
                    break 'brk;
                }
                SetupCloseLog();
                if !CABINET_hInstance.is_null() {
                    FreeLibrary(CABINET_hInstance as HMODULE);
                }
            }
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
