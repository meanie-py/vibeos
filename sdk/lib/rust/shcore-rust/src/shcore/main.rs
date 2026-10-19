// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8888EF3C3D8B: [u16; 60] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072,
    0x005C, 0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_F6C54912C188: [u16; 14] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0041, 0x0064, 0x006D, 0x0069,
    0x006E, 0x0055, 0x0049, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const core::ffi::c_char, func: *const core::ffi::c_char, line: core::ffi::c_int, fmt: *const core::ffi::c_char, ... );
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
    pub handle: HANDLE,
    pub mode: DWORD,
    pub path: *mut WCHAR,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub mem: C2RustUnnamed_17,
    pub file: C2RustUnnamed_16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub buffer: *mut BYTE,
    pub length: DWORD,
    pub position: DWORD,
    pub hkey: HKEY,
    pub valuename: *mut WCHAR,
}

#[no_mangle]
pub static mut IID_IUnknown: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_ISequentialStream: GUID = _GUID {
    Data1: 0xc733a30 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x2a1c as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IStream: GUID = _GUID {
    Data1: 0xc as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IServiceProvider: GUID = _GUID {
    Data1: 0x6d5140c1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7436 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IInternetSecurityManager: GUID = _GUID {
    Data1: 0x79eac9ee as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbaf9 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IObjectWithSite: GUID = _GUID {
    Data1: 0xfc4801a3 as ::core::ffi::c_uint,
    Data2: 0x2ba9 as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut shcore_tls: DWORD = 0;

static mut process_ref: *mut IUnknown = ::core::ptr::null::<IUnknown>() as *mut IUnknown;

static mut dummyregstream: shstream = unsafe {
    shstream {
        IStream_iface: IStream {
            lpVtbl: &raw const dummystreamvtbl,
        },
        refcount: 0,
        u: shstream_u {
            mem: shstream_u_mem {
                buffer: ::core::ptr::null::<BYTE>() as *mut BYTE,
                length: 0,
                position: 0,
                hkey: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
                valuename: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            },
        },
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(shcore);  (debug channel macro, no Rust declaration emitted)
// COBJMACROS and WINSHLWAPI are empty feature defines, so they are not emitted.

#[derive(Copy, Clone)]
#[repr(C)]
pub struct shstream_u_mem {
    pub buffer: *mut BYTE,
    pub length: DWORD,
    pub position: DWORD,
    pub hkey: HKEY,
    pub valuename: *mut WCHAR,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct shstream_u_file {
    pub handle: HANDLE,
    pub mode: DWORD,
    pub path: *mut WCHAR,
}

#[repr(C)]
pub union shstream_u {
    // Plain C data, so Copy fields and no ManuallyDrop: reading one through the stream's raw
    // pointer then makes no implicit reference (dangerous_implicit_autorefs, 49 sites).
    pub mem: shstream_u_mem,
    pub file: shstream_u_file,
}

#[repr(C)]
pub struct shstream {
    pub IStream_iface: IStream,
    pub refcount: LONG,
    pub u: shstream_u,
}

#[repr(C)]
pub struct threadref {
    pub IUnknown_iface: IUnknown,
    pub refcount: *mut LONG,
}

#[repr(C)]
pub struct thread_data {
    pub thread_proc: LPTHREAD_START_ROUTINE,
    pub callback: LPTHREAD_START_ROUTINE,
    pub data: *mut core::ffi::c_void,
    pub flags: DWORD,
    pub hEvent: HANDLE,
    pub thread_ref: *mut IUnknown,
    pub process_ref: *mut IUnknown,
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(instance: HINSTANCE, reason: DWORD, reserved: *mut core::ffi::c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(instance as HMODULE);
            shcore_tls = TlsAlloc();
        }
        DLL_PROCESS_DETACH => {
            if !reserved.is_null() {
            } else if shcore_tls != TLS_OUT_OF_INDEXES {
                TlsFree(shcore_tls);
            }
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetScaleFactorForMonitor(monitor: HMONITOR, scale: *mut DEVICE_SCALE_FACTOR) -> HRESULT {
    let _ = monitor;
    *scale = (SCALE_100_PERCENT as u32);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn GetScaleFactorForDevice(
    device_type: DISPLAY_DEVICE_TYPE,
) -> DEVICE_SCALE_FACTOR {
    // FIXME("%d\n", device_type);
    (SCALE_100_PERCENT as u32)
}

#[no_mangle]
pub unsafe extern "system" fn _IStream_Read(stream: *mut IStream, dest: *mut core::ffi::c_void, size: ULONG) -> HRESULT {
    let mut read: ULONG = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = IStream_Read!(stream, dest, size, core::ptr::addr_of_mut!(read));
    if ((hr) >= 0) && read != size {
        hr = E_FAIL;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Reset(stream: *mut IStream) -> HRESULT {
    let zero: LARGE_INTEGER = core::mem::zeroed();

    return IStream_Seek!(stream, zero, 0, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Size(stream: *mut IStream, size: *mut ULARGE_INTEGER) -> HRESULT {
    let mut statstg: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IStream_Size\0" as *const u8 as *const ::core::ffi::c_char,
            125 as ::core::ffi::c_int,
            b"(%p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            stream,
            size,
        );
    }

    memset(
        core::ptr::addr_of_mut!(statstg) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<STATSTG>() as usize,
    );

    hr = IStream_Stat!(
        stream,
        core::ptr::addr_of_mut!(statstg),
        STATFLAG_NONAME as DWORD,
    );

    if (hr >= 0 as HRESULT) && !size.is_null() {
        *size = statstg.cbSize;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn _IStream_Write(stream: *mut IStream, src: *const core::ffi::c_void, size: ULONG) -> HRESULT {
    let mut written: ULONG = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = IStream_Write!(stream, src, size, core::ptr::addr_of_mut!(written));
    if ((hr) >= 0) && written != size {
        hr = E_FAIL;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_AtomicRelease(obj: *mut *mut IUnknown) {
    // WINE_TRACE("(%p)\n", obj);

    if obj.is_null() || (*obj).is_null() {
        return;
    }

    IUnknown_Release(*obj);
    *obj = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_GetSite(unk: *mut IUnknown, iid: REFIID, site: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut obj: *mut IObjectWithSite = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if !unk.is_null() && !iid.is_null() && !site.is_null() {
        hr = IUnknown_QueryInterface(
            unk,
            core::ptr::addr_of!(IID_IObjectWithSite),
            core::ptr::addr_of_mut!(obj) as *mut *mut core::ffi::c_void,
        );
        if hr >= 0 && !obj.is_null() {
            hr = IObjectWithSite_GetSite(obj, iid, site);
            IObjectWithSite_Release(obj);
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryService(obj: *mut IUnknown, sid: REFGUID, iid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut provider: *mut IServiceProvider = core::ptr::null_mut();
    let mut hr: HRESULT;

    if out.is_null() {
        return E_FAIL;
    }

    *out = core::ptr::null_mut();

    if obj.is_null() {
        return E_FAIL;
    }

    hr = IUnknown_QueryInterface(
        obj,
        core::ptr::addr_of!(IID_IServiceProvider),
        core::ptr::addr_of_mut!(provider) as *mut *mut core::ffi::c_void,
    );
    if hr == S_OK && !provider.is_null() {

        hr = IServiceProvider_QueryService(provider, sid, iid, out);

        IServiceProvider_Release(provider);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_Set(dest: *mut *mut IUnknown, src: *mut IUnknown) {
    IUnknown_AtomicRelease(dest);

    if !src.is_null() {
        IUnknown_AddRef(src);
        *dest = src;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_SetSite(obj: *mut IUnknown, site: *mut IUnknown) -> HRESULT {
    let mut sec_manager: *mut IInternetSecurityManager =
        ::core::ptr::null_mut::<IInternetSecurityManager>();
    let mut objwithsite: *mut IObjectWithSite = ::core::ptr::null_mut::<IObjectWithSite>();
    let mut hr: HRESULT = 0;

    if obj.is_null() {
        return E_FAIL as HRESULT;
    }

    hr = (*(*obj).lpVtbl)
        .QueryInterface
        .unwrap_unchecked()(
        obj,
        &raw const IID_IObjectWithSite,
        &raw mut objwithsite as *mut *mut ::core::ffi::c_void,
    );

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IUnknown_SetSite\0" as *const u8 as *const ::core::ffi::c_char,
            232 as ::core::ffi::c_int,
            b"ObjectWithSite %p, hr %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
            objwithsite,
            hr,
        );
    }

    if hr >= 0 as HRESULT {
        hr = (*(*objwithsite).lpVtbl)
            .SetSite
            .unwrap_unchecked()(objwithsite, site);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/shcore/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"IUnknown_SetSite\0" as *const u8 as *const ::core::ffi::c_char,
                236 as ::core::ffi::c_int,
                b"SetSite() hr %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
                hr,
            );
        }

        (*(*objwithsite).lpVtbl)
            .Release
            .unwrap_unchecked()(objwithsite);
    } else {
        hr = (*(*obj).lpVtbl)
            .QueryInterface
            .unwrap_unchecked()(
            obj,
            &raw const IID_IInternetSecurityManager,
            &raw mut sec_manager as *mut *mut ::core::ffi::c_void,
        );

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/shcore/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"IUnknown_SetSite\0" as *const u8 as *const ::core::ffi::c_char,
                242 as ::core::ffi::c_int,
                b"InternetSecurityManager %p, hr %#lx.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                sec_manager,
                hr,
            );
        }

        if hr < 0 as HRESULT {
            return hr;
        }

        hr = (*(*sec_manager).lpVtbl)
            .SetSecuritySite
            .unwrap_unchecked()(
            sec_manager,
            site as *mut IInternetSecurityMgrSite,
        );

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/shcore/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"IUnknown_SetSite\0" as *const u8 as *const ::core::ffi::c_char,
                247 as ::core::ffi::c_int,
                b"SetSecuritySite() hr %#lx.\n\0" as *const u8 as *const ::core::ffi::c_char,
                hr,
            );
        }

        (*(*sec_manager).lpVtbl)
            .Release
            .unwrap_unchecked()(sec_manager);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SetCurrentProcessExplicitAppUserModelID(
    appid: *const WCHAR,
) -> HRESULT {
    // FIXME("%s: stub\n", debugstr_w(appid));
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentProcessExplicitAppUserModelID(appid: *mut *const u16) -> HRESULT {
    DPRINT1!("%p: stub\n", appid);
    *appid = core::ptr::null::<u16>();
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn CommandLineToArgvW(
    cmdline: *const WCHAR,
    numargs: *mut ::core::ffi::c_int,
) -> *mut *mut WCHAR {
    let mut qcount: ::core::ffi::c_int = 0;
    let mut bcount: ::core::ffi::c_int = 0;
    let mut s: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut argv: *mut *mut WCHAR = ::core::ptr::null_mut::<*mut WCHAR>();
    let mut argc: DWORD = 0;
    let mut d: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

    if numargs.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return ::core::ptr::null_mut::<*mut WCHAR>();
    }

    if *cmdline as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        /* Return the path to the executable */
        let mut len: DWORD = 0;
        let mut deslen: DWORD = MAX_PATH as DWORD;
        let mut size: DWORD = 0;

        size = (::core::mem::size_of::<*mut WCHAR>() as usize)
            .wrapping_mul(2 as usize)
            .wrapping_add((deslen as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize))
            as DWORD;
        loop {
            argv = LocalAlloc(LMEM_FIXED as UINT, size as SIZE_T)
                as *mut *mut WCHAR;
            if argv.is_null() {
                return ::core::ptr::null_mut::<*mut WCHAR>();
            }
            len = GetModuleFileNameW(
                0 as usize as HINSTANCE,
                argv.offset(2 as isize) as LPWSTR,
                deslen,
            );
            if len == 0 {
                LocalFree(argv as HLOCAL);
                return ::core::ptr::null_mut::<*mut WCHAR>();
            }
            if len < deslen {
                break;
            }
            deslen = deslen.wrapping_mul(2 as DWORD);
            size = (::core::mem::size_of::<*mut WCHAR>() as usize)
                .wrapping_mul(2 as usize)
                .wrapping_add(
                    (deslen as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
                ) as DWORD;
            LocalFree(argv as HLOCAL);
        }
        let ref mut fresh15 = *argv.offset(0 as isize);
        *fresh15 = argv.offset(2 as isize) as *mut WCHAR;
        let ref mut fresh16 = *argv.offset(1 as isize);
        *fresh16 = ::core::ptr::null_mut::<WCHAR>();
        *numargs = 1 as ::core::ffi::c_int;

        return argv as *mut *mut WCHAR;
    }

    /* --- First count the arguments */
    argc = 1 as DWORD;
    s = cmdline;
    /* The first argument, the executable path, follows special rules */
    if *s as ::core::ffi::c_int == '"' as i32 {
        /* The executable path ends at the next quote, no matter what */
        s = s.offset(1);
        while *s != 0 {
            let fresh17 = s;
            s = s.offset(1);
            if *fresh17 as ::core::ffi::c_int == '"' as i32 {
                break;
            }
        }
    } else {
        /* The executable path ends at the next space, no matter what */
        while *s as ::core::ffi::c_int != 0
            && *s as ::core::ffi::c_int != ' ' as i32
            && *s as ::core::ffi::c_int != '\t' as i32
        {
            s = s.offset(1);
        }
    }
    /* skip to the first argument, if any */
    while *s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32 {
        s = s.offset(1);
    }
    if *s != 0 {
        argc = argc.wrapping_add(1);
    }

    /* Analyze the remaining arguments */
    bcount = 0 as ::core::ffi::c_int;
    qcount = bcount;
    while *s != 0 {
        if (*s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32)
            && qcount == 0 as ::core::ffi::c_int
        {
            /* skip to the next argument and count it if any */
            while *s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32
            {
                s = s.offset(1);
            }
            if *s != 0 {
                argc = argc.wrapping_add(1);
            }
            bcount = 0 as ::core::ffi::c_int;
        } else if *s as ::core::ffi::c_int == '\\' as i32 {
            /* '\', count them */
            bcount += 1;
            s = s.offset(1);
        } else if *s as ::core::ffi::c_int == '"' as i32 {
            /* '"' */
            if (bcount & 1 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
            {
                qcount += 1; /* unescaped '"' */
            }
            s = s.offset(1);
            bcount = 0 as ::core::ffi::c_int;
            /* consecutive quotes, see comment in copying code below */
            while *s as ::core::ffi::c_int == '"' as i32 {
                qcount += 1;
                s = s.offset(1);
            }
            qcount = qcount % 3 as ::core::ffi::c_int;
            if qcount == 2 as ::core::ffi::c_int {
                qcount = 0 as ::core::ffi::c_int;
            }
        } else {
            /* a regular character */
            bcount = 0 as ::core::ffi::c_int;
            s = s.offset(1);
        }
    }

    /* Allocate in a single lump, the string array, and the strings that go
     * with it. This way the caller can make a single LocalFree() call to free
     * both, as per MSDN.
     */
    argv = LocalAlloc(
        LMEM_FIXED as UINT,
        (argc.wrapping_add(1 as DWORD) as SIZE_T)
            .wrapping_mul(::core::mem::size_of::<*mut WCHAR>() as SIZE_T)
            .wrapping_add(
                ((lstrlenW(cmdline as LPCWSTR)
                    + 1 as ::core::ffi::c_int) as SIZE_T)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ),
    ) as *mut *mut WCHAR;
    if argv.is_null() {
        return ::core::ptr::null_mut::<*mut WCHAR>();
    }

    /* --- Then split and copy the arguments */
    let ref mut fresh18 = *argv.offset(0 as isize);
    d = lstrcpyW(
        argv.offset(argc as isize)
            .offset(1 as isize) as LPWSTR,
        cmdline as LPCWSTR,
    ) as *mut WCHAR;
    *fresh18 = d;
    argc = 1 as DWORD;
    /* The first argument, the executable path, follows special rules */
    if *d as ::core::ffi::c_int == '"' as i32 {
        /* The executable path ends at the next quote, no matter what */
        s = d.offset(1 as isize);
        while *s != 0 {
            if *s as ::core::ffi::c_int == '"' as i32 {
                s = s.offset(1);
                break;
            } else {
                let fresh19 = s;
                s = s.offset(1);
                let fresh20 = d;
                d = d.offset(1);
                *fresh20 = *fresh19;
            }
        }
    } else {
        /* The executable path ends at the next space, no matter what */
        while *d as ::core::ffi::c_int != 0
            && *d as ::core::ffi::c_int != ' ' as i32
            && *d as ::core::ffi::c_int != '\t' as i32
        {
            d = d.offset(1);
        }
        s = d;
        if *s != 0 {
            s = s.offset(1);
        }
    }
    /* close the executable path */
    let fresh21 = d;
    d = d.offset(1);
    *fresh21 = 0 as WCHAR;
    /* skip to the first argument and initialize it if any */
    while *s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32 {
        s = s.offset(1);
    }
    if *s == 0 {
        /* There are no parameters so we are all done */
        let ref mut fresh22 = *argv.offset(argc as isize);
        *fresh22 = ::core::ptr::null_mut::<WCHAR>();
        *numargs = argc as ::core::ffi::c_int;
        return argv as *mut *mut WCHAR;
    }

    /* Split and copy the remaining arguments */
    let fresh23 = argc;
    argc = argc.wrapping_add(1);
    let ref mut fresh24 = *argv.offset(fresh23 as isize);
    *fresh24 = d;
    bcount = 0 as ::core::ffi::c_int;
    qcount = bcount;
    while *s != 0 {
        if (*s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32)
            && qcount == 0 as ::core::ffi::c_int
        {
            /* close the argument */
            let fresh25 = d;
            d = d.offset(1);
            *fresh25 = 0 as WCHAR;
            bcount = 0 as ::core::ffi::c_int;

            /* skip to the next one and initialize it if any */
            loop {
                s = s.offset(1);
                if !(*s as ::core::ffi::c_int == ' ' as i32
                    || *s as ::core::ffi::c_int == '\t' as i32)
                {
                    break;
                }
            }
            if *s != 0 {
                let fresh26 = argc;
                argc = argc.wrapping_add(1);
                let ref mut fresh27 = *argv.offset(fresh26 as isize);
                *fresh27 = d;
            }
        } else if *s as ::core::ffi::c_int == '\\' as i32 {
            let fresh28 = s;
            s = s.offset(1);
            let fresh29 = d;
            d = d.offset(1);
            *fresh29 = *fresh28;
            bcount += 1;
        } else if *s as ::core::ffi::c_int == '"' as i32 {
            if (bcount & 1 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
            {
                /* Preceded by an even number of '\', this is half that
                 * number of '\', plus a quote which we erase.
                 */
                d = d.offset(-((bcount / 2 as ::core::ffi::c_int) as isize));
                qcount += 1;
            } else {
                /* Preceded by an odd number of '\', this is half that
                 * number of '\' followed by a '"'
                 */
                d = d
                    .offset(-((bcount / 2 as ::core::ffi::c_int) as isize))
                    .offset(-(1 as isize));
                let fresh30 = d;
                d = d.offset(1);
                *fresh30 = '"' as i32 as WCHAR;
            }
            s = s.offset(1);
            bcount = 0 as ::core::ffi::c_int;
            /* Now count the number of consecutive quotes. Note that qcount
             * already takes into account the opening quote if any, as well as
             * the quote that lead us here.
             */
            while *s as ::core::ffi::c_int == '"' as i32 {
                qcount += 1;
                if qcount == 3 as ::core::ffi::c_int {
                    let fresh31 = d;
                    d = d.offset(1);
                    *fresh31 = '"' as i32 as WCHAR;
                    qcount = 0 as ::core::ffi::c_int;
                }
                s = s.offset(1);
            }
            if qcount == 2 as ::core::ffi::c_int {
                qcount = 0 as ::core::ffi::c_int;
            }
        } else {
            /* a regular character */
            let fresh32 = s;
            s = s.offset(1);
            let fresh33 = d;
            d = d.offset(1);
            *fresh33 = *fresh32;
            bcount = 0 as ::core::ffi::c_int;
        }
    }
    *d = '\0' as i32 as WCHAR;
    let ref mut fresh34 = *argv.offset(argc as isize);
    *fresh34 = ::core::ptr::null_mut::<WCHAR>();
    *numargs = argc as ::core::ffi::c_int;

    return argv as *mut *mut WCHAR;
}

unsafe extern "C" fn impl_from_IStream(iface: *mut IStream) -> *mut shstream {
    return (iface as *mut u8).sub(core::mem::offset_of!(shstream, IStream_iface)) as *mut shstream;
}
unsafe extern "system" fn shstream_QueryInterface(iface: *mut IStream, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IStream))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_ISequentialStream))) != 0)
    {
        *out = iface as *mut core::ffi::c_void;
        IStream_AddRef!(iface);
        return S_OK;
    }

    *out = core::ptr::null_mut();
    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}
unsafe extern "system" fn shstream_AddRef(mut iface: *mut IStream) -> ULONG {
    let mut stream: *mut shstream = impl_from_IStream(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*stream).refcount) as *mut LONG,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"shstream_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            579 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn memstream_Release(iface: *mut IStream) -> ULONG {
    let stream: *mut shstream = impl_from_IStream(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*stream).refcount) as *mut LONG) as ULONG;

    if refcount == 0 {
        free((*stream).u.mem.buffer as *mut core::ffi::c_void);
        free(stream as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn memstream_Read(
    iface: *mut IStream,
    buff: *mut core::ffi::c_void,
    buff_size: ULONG,
    read_len: *mut ULONG,
) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut length: DWORD = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"memstream_Read\0" as *const u8 as *const core::ffi::c_char,
            605 as core::ffi::c_int,
            b"%p, %p, %lu, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            buff,
            buff_size,
            read_len,
        );
    }

    if (*stream).u.mem.position >= (*stream).u.mem.length {
        if !read_len.is_null() {
            *read_len = 0;
        }
        return S_FALSE;
    }

    length = (*stream)
        .u
        .mem
        .length
        .wrapping_sub((*stream).u.mem.position);
    if buff_size < length {
        length = buff_size as DWORD;
    }

    memmove(
        buff,
        (*stream)
            .u
            .mem
            .buffer
            .offset((*stream).u.mem.position as isize) as *const core::ffi::c_void,
        length as size_t,
    );
    (*stream).u.mem.position = (*stream).u.mem.position.wrapping_add(length);

    if !read_len.is_null() {
        *read_len = length as ULONG;
    }

    return S_OK;
}

unsafe extern "system" fn memstream_Write(iface: *mut IStream, buff: *const core::ffi::c_void, buff_size: ULONG, written: *mut ULONG) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);
    let mut length: DWORD = (*stream).u.mem.position.wrapping_add(buff_size as DWORD);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"memstream_Write\0" as *const u8 as *const core::ffi::c_char,
            632 as core::ffi::c_int,
            b"%p, %p, %lu, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            buff,
            buff_size,
            written,
        );
    }
    if length < (*stream).u.mem.position {
        return STG_E_INSUFFICIENTMEMORY;
    }
    if length > (*stream).u.mem.length {
        let mut buffer: *mut BYTE = _recalloc(
            (*stream).u.mem.buffer as *mut core::ffi::c_void,
            1 as size_t,
            length as size_t,
        ) as *mut BYTE;
        if buffer.is_null() {
            return STG_E_INSUFFICIENTMEMORY;
        }
        (*stream).u.mem.length = length;
        (*stream).u.mem.buffer = buffer;
    }
    memmove(
        (*stream).u.mem.buffer.offset((*stream).u.mem.position as isize) as *mut core::ffi::c_void,
        buff,
        buff_size as size_t,
    );
    (*stream).u.mem.position = (*stream).u.mem.position.wrapping_add(buff_size as DWORD);
    if !written.is_null() {
        *written = buff_size;
    }
    return S_OK;
}

unsafe extern "system" fn memstream_Seek(
    iface: *mut IStream,
    move_0: LARGE_INTEGER,
    origin: DWORD,
    new_pos: *mut ULARGE_INTEGER,
) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut tmp: LARGE_INTEGER = core::mem::zeroed();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"memstream_Seek\0" as *const u8 as *const ::core::ffi::c_char,
            660 as ::core::ffi::c_int,
            b"%p, %s, %ld, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            wine_dbgstr_longlong(move_0.QuadPart as ULONGLONG),
            origin,
            new_pos,
        );
    }

    if origin == STREAM_SEEK_SET {
        tmp = move_0;
    } else if origin == STREAM_SEEK_CUR {
        tmp.QuadPart = (*stream).u.mem.position as LONGLONG + move_0.QuadPart;
    } else if origin == STREAM_SEEK_END {
        tmp.QuadPart = (*stream).u.mem.length as LONGLONG + move_0.QuadPart;
    } else {
        return STG_E_INVALIDPARAMETER;
    }

    if tmp.QuadPart < 0 {
        return STG_E_INVALIDFUNCTION;
    }

    /* we cut off the high part here */
    (*stream).u.mem.position = tmp.u.LowPart;

    if !new_pos.is_null() {
        (*new_pos).QuadPart = (*stream).u.mem.position as ULONGLONG;
    }
    return S_OK;
}

unsafe extern "system" fn memstream_SetSize(iface: *mut IStream, new_size: ULARGE_INTEGER) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut length: DWORD;
    let mut buffer: *mut BYTE;

    /* we cut off the high part here */
    length = new_size.u.LowPart;
    buffer = _recalloc((*stream).u.mem.buffer as *mut core::ffi::c_void, 1 as size_t, length as size_t) as *mut BYTE;
    if buffer.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    (*stream).u.mem.buffer = buffer;
    (*stream).u.mem.length = length;

    return S_OK;
}

unsafe extern "system" fn shstream_CopyTo(
    iface: *mut IStream,
    dest: *mut IStream,
    mut size: ULARGE_INTEGER,
    read_len: *mut ULARGE_INTEGER,
    written: *mut ULARGE_INTEGER,
) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut total_read: ULARGE_INTEGER = core::mem::zeroed();
    let mut total_written: ULARGE_INTEGER = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;
    let mut buffer: [BYTE; 0x400] = [0; 0x400];

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"shstream_CopyTo\0" as *const u8 as *const ::core::ffi::c_char,
            710 as ::core::ffi::c_int,
            b"(%p, %p, %s, %p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            stream,
            dest,
            wine_dbgstr_longlong(size.QuadPart),
            read_len,
            written,
        );
    }

    if dest.is_null() {
        return E_POINTER;
    }

    total_read.QuadPart = 0;
    total_written.QuadPart = 0;

    while size.QuadPart > 0 {
        let mut chunk_size: ULONG = if size.QuadPart
            >= (core::mem::size_of_val(&buffer) as ULONGLONG)
        {
            core::mem::size_of_val(&buffer) as ULONG
        } else {
            size.u.LowPart
        };
        let mut chunk_read: ULONG = 0;
        let mut chunk_written: ULONG = 0;

        hr = IStream_Read!(
            iface,
            core::ptr::addr_of_mut!(buffer) as *mut BYTE as *mut ::core::ffi::c_void,
            chunk_size,
            core::ptr::addr_of_mut!(chunk_read),
        );
        if hr < 0 {
            break;
        }

        total_read.QuadPart += chunk_read as ULONGLONG;

        if chunk_read != 0 {
            hr = IStream_Write!(
                dest,
                core::ptr::addr_of_mut!(buffer) as *mut BYTE as *const ::core::ffi::c_void,
                chunk_read,
                core::ptr::addr_of_mut!(chunk_written),
            );
            if hr < 0 {
                break;
            }

            total_written.QuadPart += chunk_written as ULONGLONG;
        }

        if chunk_read != chunk_size {
            size.QuadPart = 0;
        } else {
            size.QuadPart -= chunk_read as ULONGLONG;
        }
    }

    if !read_len.is_null() {
        (*read_len).QuadPart = total_read.QuadPart;
    }
    if !written.is_null() {
        (*written).QuadPart = total_written.QuadPart;
    }

    return hr;
}
unsafe extern "system" fn shstream_Commit(iface: *mut IStream, flags: DWORD) -> HRESULT {
    // TRACE("%p, %#lx.\n", iface, flags);
    E_NOTIMPL
}
unsafe extern "system" fn shstream_Revert(iface: *mut IStream) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);
    let _ = stream;

    /* revert not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "system" fn shstream_LockRegion(iface: *mut IStream, offset: ULARGE_INTEGER, size: ULARGE_INTEGER, lock_type: DWORD) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);

    TRACE!("(%p)\n", stream);

    /* lock/unlock not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "system" fn shstream_UnlockRegion(iface: *mut IStream, offset: ULARGE_INTEGER, size: ULARGE_INTEGER, lock_type: DWORD) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let _ = (stream, offset, size, lock_type);

    /* lock/unlock not supported by this stream */
    return E_NOTIMPL;
}
unsafe extern "system" fn memstream_Stat(iface: *mut IStream, statstg: *mut STATSTG, flags: DWORD) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);

    memset(statstg as *mut core::ffi::c_void, 0, core::mem::size_of::<STATSTG>());
    (*statstg).type_ = STGTY_STREAM;
    (*statstg).cbSize.QuadPart = (*stream).u.mem.length as u64;
    (*statstg).grfMode = STGM_READWRITE;

    return S_OK;
}

unsafe extern "system" fn shstream_Clone(iface: *mut IStream, dest: *mut *mut IStream) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);

    TRACE!("(%p, %p)\n", stream, dest);
    *dest = core::ptr::null_mut();

    /* clone not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "C" fn shstream_create(vtbl: *const IStreamVtbl, data: *const BYTE, data_len: UINT) -> *mut shstream {
    let mut data_len = data_len;
    let mut stream: *mut shstream = core::ptr::null_mut::<shstream>();
    if data.is_null() {
        data_len = 0;
    }
    stream = malloc(core::mem::size_of::<shstream>() as size_t) as *mut shstream;
    (*stream).IStream_iface.lpVtbl = vtbl;
    (*stream).refcount = 1;
    (*stream).u.mem.buffer = malloc(data_len as size_t) as *mut BYTE;
    if (*stream).u.mem.buffer.is_null() {
        free(stream as *mut core::ffi::c_void);
        return core::ptr::null_mut::<shstream>();
    }
    memcpy(
        (*stream).u.mem.buffer as *mut core::ffi::c_void,
        data as *const core::ffi::c_void,
        data_len as size_t,
    );
    (*stream).u.mem.length = data_len as DWORD;
    (*stream).u.mem.position = 0;

    return stream;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateMemStream(data: *const BYTE, data_len: UINT) -> *mut IStream {
    let mut stream: *mut shstream;

    stream = shstream_create(core::ptr::addr_of!(memstreamvtbl), data, data_len);
    return if !stream.is_null() {
        core::ptr::addr_of_mut!((*stream).IStream_iface)
    } else {
        core::ptr::null_mut()
    };
}

unsafe extern "system" fn filestream_Release(iface: *mut IStream) -> ULONG {
    let stream: *mut shstream = impl_from_IStream(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*stream).refcount)) as ULONG;

    if refcount == 0 {
        CloseHandle((*stream).u.file.handle);
        free((*stream).u.file.path as *mut core::ffi::c_void);
        free(stream as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn filestream_Read(iface: *mut IStream, buff: *mut core::ffi::c_void, size: ULONG, read_len: *mut ULONG) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut read: DWORD = 0;

    if ReadFile((*stream).u.file.handle, buff, size, core::ptr::addr_of_mut!(read), core::ptr::null_mut()) == 0 {
        WARN!("error %ld reading file\n", GetLastError());
        return S_FALSE;
    }

    if !read_len.is_null() {
        *read_len = read;
    }

    return if read == size { S_OK } else { S_FALSE };
}

unsafe extern "system" fn filestream_Write(
    iface: *mut IStream,
    buff: *const core::ffi::c_void,
    size: ULONG,
    written: *mut ULONG,
) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut written_len: DWORD = 0;

    match (*stream).u.file.mode & 0xf {
        STGM_WRITE | STGM_READWRITE => {}
        _ => return STG_E_ACCESSDENIED,
    }

    if WriteFile(
        (*stream).u.file.handle,
        buff,
        size,
        core::ptr::addr_of_mut!(written_len),
        core::ptr::null_mut(),
    ) == 0
    {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    if !written.is_null() {
        *written = written_len;
    }

    return S_OK;
}

unsafe extern "system" fn filestream_Seek(
    iface: *mut IStream,
    move_: LARGE_INTEGER,
    origin: DWORD,
    new_pos: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);
    let mut position: DWORD = 0;

    TRACE!(
        "%p, %s, %ld, %p.\n",
        iface,
        wine_dbgstr_longlong(move_.QuadPart as ULONGLONG),
        origin,
        new_pos
    );

    position = SetFilePointer(
        (*stream).u.file.handle,
        move_.u.LowPart as LONG,
        core::ptr::null_mut::<LONG>(),
        origin,
    );
    if position == INVALID_SET_FILE_POINTER {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    if !new_pos.is_null() {
        (*new_pos).u.HighPart = 0;
        (*new_pos).u.LowPart = position;
    }

    return S_OK;
}

unsafe extern "system" fn filestream_SetSize(iface: *mut IStream, size: ULARGE_INTEGER) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut origin: LARGE_INTEGER = core::mem::zeroed();
    let mut move_0: LARGE_INTEGER = core::mem::zeroed();

    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0".as_ptr() as *const i8,
            b"filestream_SetSize\0".as_ptr() as *const i8,
            969i32,
            b"(%p, %s)\n\0".as_ptr() as *const i8,
            stream,
            wine_dbgstr_longlong(size.QuadPart),
        );
    }

    move_0.QuadPart = 0;
    if SetFilePointerEx((*stream).u.file.handle, move_0, &mut origin, FILE_CURRENT) == 0 {
        return E_FAIL;
    }

    move_0.QuadPart = (size.QuadPart) as i64;
    if SetFilePointerEx((*stream).u.file.handle, move_0, core::ptr::null_mut(), FILE_BEGIN) == 0 {
        return E_FAIL;
    }

    if (*stream).u.file.mode != STGM_READ {
        if SetEndOfFile((*stream).u.file.handle) == 0 {
            return E_FAIL;
        }
        if SetFilePointerEx((*stream).u.file.handle, origin, core::ptr::null_mut(), FILE_BEGIN) == 0 {
            return E_FAIL;
        }
    }

    return S_OK;
}
unsafe extern "system" fn filestream_CopyTo(
    mut iface: *mut IStream,
    mut dest: *mut IStream,
    mut size: ULARGE_INTEGER,
    mut read_len: *mut ULARGE_INTEGER,
    mut written: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut stream: *mut shstream = impl_from_IStream(iface);
    let mut hr: HRESULT = S_OK;
    let mut buff: [::core::ffi::c_char; 1024] = [0; 1024];
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"filestream_CopyTo\0" as *const u8 as *const ::core::ffi::c_char,
            997 as ::core::ffi::c_int,
            b"(%p, %p, %s, %p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            stream,
            dest,
            wine_dbgstr_longlong(size.QuadPart),
            read_len,
            written,
        );
    }
    if !read_len.is_null() {
        (*read_len).QuadPart = 0 as ULONGLONG;
    }
    if !written.is_null() {
        (*written).QuadPart = 0 as ULONGLONG;
    }
    if dest.is_null() {
        return S_OK;
    }
    while size.QuadPart != 0 {
        let mut left: ULONG = 0;
        let mut read_chunk: ULONG = 0;
        let mut written_chunk: ULONG = 0;
        left = (if size.QuadPart as usize
            > ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize
        {
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize
        } else {
            size.QuadPart as usize
        }) as ULONG;
        hr = IStream_Read!(
            iface,
            &raw mut buff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            left,
            &raw mut read_chunk,
        );
        if hr < 0 || read_chunk == 0 as ULONG {
            break;
        }
        if !read_len.is_null() {
            (*read_len).QuadPart = (*read_len).QuadPart.wrapping_add(read_chunk as ULONGLONG);
        }
        hr = IStream_Write!(
            dest,
            &raw mut buff as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            read_chunk,
            &raw mut written_chunk,
        );
        if !written.is_null() {
            (*written).QuadPart = (*written).QuadPart.wrapping_add(written_chunk as ULONGLONG);
        }
        if hr < 0 || written_chunk != left {
            break;
        }
        size.QuadPart = size.QuadPart.wrapping_sub(left as ULONGLONG);
    }
    return hr;
}
unsafe extern "system" fn filestream_Commit(iface: *mut IStream, flags: DWORD) -> HRESULT {
    // TRACE("%p, %#lx.\n", iface, flags);
    S_OK
}

unsafe extern "system" fn filestream_Stat(iface: *mut IStream, statstg: *mut STATSTG, flags: DWORD) -> HRESULT {
    let stream: *mut shstream = impl_from_IStream(iface);
    let mut fi: BY_HANDLE_FILE_INFORMATION = core::mem::zeroed();

    if statstg.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    memset(core::ptr::addr_of_mut!(fi) as *mut core::ffi::c_void, 0, core::mem::size_of::<BY_HANDLE_FILE_INFORMATION>() as usize);
    GetFileInformationByHandle((*stream).u.file.handle, core::ptr::addr_of_mut!(fi));

    if (flags & STATFLAG_NONAME) != 0 {
        (*statstg).pwcsName = core::ptr::null_mut();
    } else {
        let len: i32 = lstrlenW((*stream).u.file.path as LPCWSTR);
        (*statstg).pwcsName = CoTaskMemAlloc(
            ((len + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPOLESTR;
        if !(*statstg).pwcsName.is_null() {
            memcpy(
                (*statstg).pwcsName as *mut core::ffi::c_void,
                (*stream).u.file.path as *const core::ffi::c_void,
                ((len + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            );
        }
    }
    (*statstg).type_ = 0;
    (*statstg).cbSize.u.LowPart = fi.nFileSizeLow;
    (*statstg).cbSize.u.HighPart = fi.nFileSizeHigh;
    (*statstg).mtime = fi.ftLastWriteTime;
    (*statstg).ctime = fi.ftCreationTime;
    (*statstg).atime = fi.ftLastAccessTime;
    (*statstg).grfMode = (*stream).u.file.mode;
    (*statstg).grfLocksSupported = 0;
    memcpy(
        core::ptr::addr_of_mut!((*statstg).clsid) as *mut core::ffi::c_void,
        core::ptr::addr_of!(IID_IStream) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>() as usize,
    );
    (*statstg).grfStateBits = 0;
    (*statstg).reserved = 0;

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileEx(
    mut path: *const WCHAR,
    mut mode: DWORD,
    mut attributes: DWORD,
    mut create: BOOL,
    mut template: *mut IStream,
    mut ret: *mut *mut IStream,
) -> HRESULT {
    let mut access: DWORD = 0;
    let mut share: DWORD = 0;
    let mut creation_disposition: DWORD = 0;
    let mut len: DWORD = 0;
    let mut stream: *mut shstream = ::core::ptr::null_mut::<shstream>();
    let mut hFile: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SHCreateStreamOnFileEx\0" as *const u8 as *const ::core::ffi::c_char,
            1105 as ::core::ffi::c_int,
            b"%s, %ld, %#lx, %d, %p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(path),
            mode,
            attributes,
            create,
            template,
            ret,
        );
    }
    if path.is_null() || ret.is_null() || !template.is_null() {
        return E_INVALIDARG;
    }
    *ret = ::core::ptr::null_mut::<IStream>();
    match mode & 0xf {
        STGM_WRITE | STGM_READWRITE => {
            access = (GENERIC_READ | GENERIC_WRITE as ::core::ffi::c_ulong) as DWORD;
        }
        STGM_READ => {
            access = GENERIC_READ as DWORD;
        }
        _ => return E_INVALIDARG,
    }
    match mode & 0xf0 {
        0 | STGM_SHARE_DENY_NONE => {
            share = (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD;
        }
        STGM_SHARE_DENY_READ => {
            share = FILE_SHARE_WRITE as DWORD;
        }
        STGM_SHARE_DENY_WRITE => {
            share = FILE_SHARE_READ as DWORD;
        }
        STGM_SHARE_EXCLUSIVE => {
            share = 0 as DWORD;
        }
        _ => return E_INVALIDARG,
    }
    match mode & 0xf000 {
        STGM_FAILIFTHERE => {
            creation_disposition = (if create != 0 {
                CREATE_NEW
            } else {
                OPEN_EXISTING
            }) as DWORD;
        }
        STGM_CREATE => {
            creation_disposition = CREATE_ALWAYS as DWORD;
        }
        _ => return E_INVALIDARG,
    }
    hFile = CreateFileW(
        path as LPCWSTR,
        access,
        share,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        creation_disposition,
        attributes,
        0 as HANDLE,
    );
    if hFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32!(GetLastError());
    }
    stream = malloc(::core::mem::size_of::<shstream>() as size_t) as *mut shstream;
    (*stream).IStream_iface.lpVtbl = &raw const filestreamvtbl;
    (*stream).refcount = 1 as LONG;
    (*stream).u.file.handle = hFile;
    (*stream).u.file.mode = mode;
    len = lstrlenW(path as LPCWSTR) as DWORD;
    (*stream).u.file.path = malloc(
        (len.wrapping_add(1 as DWORD) as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    memcpy(
        (*stream).u.file.path as *mut ::core::ffi::c_void,
        path as *const ::core::ffi::c_void,
        (len.wrapping_add(1 as DWORD) as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    *ret = &raw mut (*stream).IStream_iface;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileW(path: *const WCHAR, mode: DWORD, stream: *mut *mut IStream) -> HRESULT {
    if path.is_null() || stream.is_null() {
        return E_INVALIDARG;
    }

    if (mode & (STGM_CONVERT | STGM_DELETEONRELEASE | STGM_TRANSACTED)) != 0 {
        return E_INVALIDARG;
    }

    return SHCreateStreamOnFileEx(path, mode, 0, (FALSE) as BOOL, core::ptr::null_mut(), stream);
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileA(
    path: *const ::core::ffi::c_char,
    mode: DWORD,
    stream: *mut *mut IStream,
) -> HRESULT {
    let mut pathW: *mut WCHAR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut len: DWORD = 0;

    TRACE!("%s, %#lx, %p.\n", debugstr_a(path), mode, stream);

    if path.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_PATH_NOT_FOUND);
    }

    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        path as LPCSTR,
        (-(1)) as ::core::ffi::c_int,
        core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    ) as DWORD;
    pathW = malloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if pathW.is_null() {
        return E_OUTOFMEMORY;
    }

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        path as LPCSTR,
        (-(1)) as ::core::ffi::c_int,
        pathW as LPWSTR,
        len as ::core::ffi::c_int,
    );
    hr = SHCreateStreamOnFileW(pathW as *const WCHAR, mode, stream);
    free(pathW as *mut ::core::ffi::c_void);

    return hr;
}

unsafe extern "system" fn regstream_Release(iface: *mut IStream) -> ULONG {
    let stream: *mut shstream = impl_from_IStream(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*stream).refcount)) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*stream).u.mem.hkey.is_null() {
            if (*stream).u.mem.length != 0 {
                RegSetValueExW(
                    (*stream).u.mem.hkey,
                    (*stream).u.mem.valuename as LPCWSTR,
                    0 as DWORD,
                    REG_BINARY as DWORD,
                    (*stream).u.mem.buffer as *const BYTE,
                    (*stream).u.mem.length,
                );
            } else {
                RegDeleteValueW((*stream).u.mem.hkey, (*stream).u.mem.valuename as LPCWSTR);
            }
            RegCloseKey((*stream).u.mem.hkey);
        }
        CoTaskMemFree((*stream).u.mem.valuename as LPVOID);
        free((*stream).u.mem.buffer as *mut ::core::ffi::c_void);
        free(stream as *mut ::core::ffi::c_void);
    }

    return refcount;
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStream2W(
    mut hKey: HKEY,
    mut subkey: *const WCHAR,
    mut value: *const WCHAR,
    mut mode: DWORD,
) -> *mut IStream {
    let mut stream: *mut shstream = core::ptr::null_mut();
    let mut hStrKey: HKEY = core::ptr::null_mut();
    let mut buff: *mut BYTE = core::ptr::null_mut();
    let mut length: DWORD = 0;
    let mut ret: LONG;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SHOpenRegStream2W\0" as *const u8 as *const core::ffi::c_char,
            1274 as core::ffi::c_int,
            b"%p, %s, %s, %#lx.\n\0" as *const u8 as *const core::ffi::c_char,
            hKey,
            debugstr_w(subkey),
            debugstr_w(value),
            mode,
        );
    }

    if mode == STGM_READ {
        ret = RegOpenKeyExW(
            hKey,
            subkey,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hStrKey),
        ) as LONG;
    } else {
        /* in write mode we make sure the subkey exits */
        ret = RegCreateKeyExW(
            hKey,
            subkey,
            0,
            core::ptr::null_mut(),
            0,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hStrKey),
            core::ptr::null_mut(),
        ) as LONG;
    }

    if ret == ERROR_SUCCESS as LONG {
        if mode == STGM_READ || mode == STGM_READWRITE {
            /* read initial data */
            ret = RegQueryValueExW(
                hStrKey,
                value,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(length),
            ) as LONG;
            if ret == ERROR_SUCCESS as LONG && length != 0 {
                buff = malloc(length as usize) as *mut BYTE;
                RegQueryValueExW(
                    hStrKey,
                    value,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    buff,
                    core::ptr::addr_of_mut!(length),
                );
            }
        }

        if length == 0 {
            buff = malloc(length as usize) as *mut BYTE;
        }

        stream = shstream_create(
            core::ptr::addr_of!(regstreamvtbl),
            buff,
            length as UINT,
        );
        free(buff as *mut core::ffi::c_void);
        if !stream.is_null() {
            (*stream).u.mem.hkey = hStrKey;
            SHStrDupW(value, core::ptr::addr_of_mut!((*stream).u.mem.valuename));
            return core::ptr::addr_of_mut!((*stream).IStream_iface);
        }
    }

    if !hStrKey.is_null() {
        RegCloseKey(hStrKey);
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStream2A(
    hKey: HKEY,
    subkey: *const core::ffi::c_char,
    value: *const core::ffi::c_char,
    mode: DWORD,
) -> *mut IStream {
    let mut subkeyW: *mut WCHAR = core::ptr::null_mut();
    let mut valueW: *mut WCHAR = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SHOpenRegStream2A\0" as *const u8 as *const core::ffi::c_char,
            1321 as core::ffi::c_int,
            b"%p, %s, %s, %#lx.\n\0" as *const u8 as *const core::ffi::c_char,
            hKey,
            debugstr_a(subkey),
            debugstr_a(value),
            mode,
        );
    }
    if !subkey.is_null()
        && (SHStrDupA(subkey, core::ptr::addr_of_mut!(subkeyW)))
            < 0
    {
        return core::ptr::null_mut();
    }
    if !value.is_null()
        && (SHStrDupA(value, core::ptr::addr_of_mut!(valueW)))
            < 0
    {
        CoTaskMemFree(subkeyW as LPVOID);
        return core::ptr::null_mut();
    }

    stream = SHOpenRegStream2W(
        hKey,
        subkeyW as *const WCHAR,
        valueW as *const WCHAR,
        mode,
    ) as *mut IStream;
    CoTaskMemFree(subkeyW as LPVOID);
    CoTaskMemFree(valueW as LPVOID);
    return stream as *mut IStream;
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStreamA(hkey: HKEY, subkey: *const i8, value: *const i8, mode: DWORD) -> *mut IStream {
    let mut subkeyW: *mut WCHAR = core::ptr::null_mut();
    let mut valueW: *mut WCHAR = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();

    if !subkey.is_null() && (SHStrDupA(subkey, core::ptr::addr_of_mut!(subkeyW)) < 0) {
        return core::ptr::null_mut();
    }
    if !value.is_null() && (SHStrDupA(value, core::ptr::addr_of_mut!(valueW)) < 0) {
        CoTaskMemFree(subkeyW as LPVOID);
        return core::ptr::null_mut();
    }

    stream = SHOpenRegStreamW(hkey, subkeyW as *const WCHAR, valueW as *const WCHAR, mode) as *mut IStream;
    CoTaskMemFree(subkeyW as LPVOID);
    CoTaskMemFree(valueW as LPVOID);
    return stream as *mut IStream;
}
unsafe extern "system" fn dummystream_AddRef(iface: *mut IStream) -> ULONG {
    // TRACE("()\n");
    2
}
unsafe extern "system" fn dummystream_Release(iface: *mut IStream) -> ULONG {
    // TRACE("()\n");
    1
}

unsafe extern "system" fn dummystream_Read(iface: *mut IStream, buff: *mut core::ffi::c_void, buff_size: ULONG, read_len: *mut ULONG) -> HRESULT {
    if !read_len.is_null() {
        *read_len = 0;
    }

    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStreamW(hkey: HKEY, subkey: *const WCHAR, value: *const WCHAR, mode: DWORD) -> *mut IStream {
    let mut stream: *mut IStream = core::ptr::null_mut();
    stream = SHOpenRegStream2W(hkey, subkey, value, mode);
    return if !stream.is_null() {
        stream
    } else {
        core::ptr::addr_of_mut!(dummyregstream.IStream_iface)
    };
}

unsafe extern "C" fn threadref_impl_from_IUnknown(iface: *mut IUnknown) -> *mut threadref {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(threadref, IUnknown_iface)) as *mut threadref;
}

unsafe extern "system" fn threadref_QueryInterface(iface: *mut IUnknown, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let threadref: *mut threadref = threadref_impl_from_IUnknown(iface);

    if out.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false {
        *out = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    *out = core::ptr::null_mut();
    WARN!("Interface %s not supported.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "system" fn threadref_AddRef(mut iface: *mut IUnknown) -> ULONG {
    let mut threadref: *mut threadref = threadref_impl_from_IUnknown(iface);
    let mut refcount: LONG = InterlockedIncrement((*threadref).refcount);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0".as_ptr() as *const i8,
            b"threadref_AddRef\0".as_ptr() as *const i8,
            1450i32,
            b"%p, refcount %ld.\n\0".as_ptr() as *const i8,
            threadref,
            refcount,
        );
    }
    return refcount as ULONG;
}

unsafe extern "system" fn threadref_Release(iface: *mut IUnknown) -> ULONG {
    let threadref: *mut threadref = threadref_impl_from_IUnknown(iface);
    let refcount: LONG = InterlockedDecrement((*threadref).refcount as *mut LONG);

    if refcount == 0 {
        free(threadref as *mut core::ffi::c_void);
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateThreadRef(refcount: *mut LONG, out: *mut *mut IUnknown) -> HRESULT {
    let mut threadref: *mut threadref = core::ptr::null_mut();

    if refcount.is_null() || out.is_null() {
        return E_INVALIDARG;
    }

    *out = core::ptr::null_mut();

    threadref = malloc(core::mem::size_of::<threadref>()) as *mut threadref;
    if threadref.is_null() {
        return E_OUTOFMEMORY;
    }
    (*threadref).IUnknown_iface.lpVtbl = core::ptr::addr_of!(threadrefvtbl);
    (*threadref).refcount = refcount;

    *refcount = 1;
    *out = core::ptr::addr_of_mut!((*threadref).IUnknown_iface);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetThreadRef(out: *mut *mut IUnknown) -> HRESULT {
    if shcore_tls == TLS_OUT_OF_INDEXES {
        return E_NOINTERFACE;
    }
    *out = TlsGetValue(shcore_tls) as *mut IUnknown;
    if (*out).is_null() {
        return E_NOINTERFACE;
    }
    IUnknown_AddRef(*out);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetThreadRef(obj: *mut IUnknown) -> HRESULT {
    if shcore_tls == TLS_OUT_OF_INDEXES {
        return E_NOINTERFACE;
    }

    TlsSetValue(shcore_tls, obj as PVOID);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHReleaseThreadRef() -> HRESULT {
    // FIXME("() - stub!\n");
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessReference(obj: *mut *mut IUnknown) -> HRESULT {
    *obj = process_ref;

    if process_ref.is_null() {
        return E_FAIL;
    }

    if !(*obj).is_null() {
        IUnknown_AddRef(*obj);
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessReference(obj: *mut IUnknown) {
    process_ref = obj;
}

unsafe extern "system" fn shcore_thread_wrapper(data: LPVOID) -> DWORD {
    let mut thread_data: thread_data = thread_data {
        thread_proc: None,
        callback: None,
        data: core::ptr::null_mut(),
        flags: 0,
        hEvent: core::ptr::null_mut(),
        thread_ref: core::ptr::null_mut(),
        process_ref: core::ptr::null_mut(),
    };
    let mut hr: HRESULT = E_FAIL;
    let mut retval: DWORD;

    /* We are now executing in the context of the newly created thread.
     * So we copy the data passed to us (it is on the stack of the function
     * that called us, which is waiting for us to signal an event before
     * returning). */
    thread_data = core::ptr::read(data as *mut thread_data);

    if (thread_data.flags & CTF_COINIT) != 0 {
        hr = CoInitializeEx(
            core::ptr::null_mut(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as DWORD,
        );
        if hr < 0 {
            hr = CoInitializeEx(core::ptr::null_mut(), COINIT_DISABLE_OLE1DDE as DWORD);
        }
    }

    if thread_data.callback.is_some() {
        thread_data.callback.unwrap_unchecked()(thread_data.data as LPVOID);
    }

    /* Signal the thread that created us; it can return now. */
    SetEvent(thread_data.hEvent);

    /* Execute the callers start code. */
    retval = thread_data.thread_proc.unwrap_unchecked()(thread_data.data as LPVOID);

    /* Release thread and process references. */
    if !thread_data.thread_ref.is_null() {
        IUnknown_Release(thread_data.thread_ref);
    }

    if !thread_data.process_ref.is_null() {
        IUnknown_Release(thread_data.process_ref);
    }

    if hr >= 0 {
        CoUninitialize();
    }

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateThread(
    thread_proc: LPTHREAD_START_ROUTINE,
    data: *mut ::core::ffi::c_void,
    flags: DWORD,
    callback: LPTHREAD_START_ROUTINE,
) -> BOOL {
    let mut thread_data: thread_data = core::mem::zeroed();
    let mut called: BOOL = (FALSE as i32);

    thread_data.thread_proc = thread_proc;
    thread_data.callback = callback;
    thread_data.data = data;
    thread_data.flags = flags;
    thread_data.hEvent = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null(),
    );

    if (flags & CTF_THREAD_REF) != 0 {
        SHGetThreadRef(core::ptr::addr_of_mut!(thread_data.thread_ref));
    } else {
        thread_data.thread_ref = core::ptr::null_mut();
    }

    if (flags & CTF_PROCESS_REF) != 0 {
        GetProcessReference(core::ptr::addr_of_mut!(thread_data.process_ref));
    } else {
        thread_data.process_ref = core::ptr::null_mut();
    }

    /* Create the thread */
    if !thread_data.hEvent.is_null() {
        let mut hThread: HANDLE = core::ptr::null_mut();
        let mut retval: DWORD = 0;

        hThread = CreateThread(
            core::ptr::null_mut(),
            0 as DWORD,
            Some(shcore_thread_wrapper),
            core::ptr::addr_of_mut!(thread_data) as PVOID,
            0 as DWORD,
            core::ptr::addr_of_mut!(retval),
        );
        if !hThread.is_null() {
            /* Wait for the thread to signal us to continue */
            WaitForSingleObject(thread_data.hEvent, INFINITE as DWORD);
            CloseHandle(hThread);
            called = (TRUE as i32);
        }
        CloseHandle(thread_data.hEvent);
    }

    if called == 0 {
        if thread_data.callback.is_none() && (flags & CTF_INSIST) != 0 {
            /* Couldn't call, call synchronously */
            thread_data.thread_proc.unwrap_unchecked()(data as LPVOID);
            called = (TRUE as i32);
        } else {
            if !thread_data.thread_ref.is_null() {
                IUnknown_Release(thread_data.thread_ref);
            }

            if !thread_data.process_ref.is_null() {
                IUnknown_Release(thread_data.process_ref);
            }
        }
    }

    return called;
}

#[no_mangle]
pub unsafe extern "system" fn SHStrDupW(src: *const WCHAR, dest: *mut *mut WCHAR) -> HRESULT {
    let mut len: usize = 0;

    *dest = core::ptr::null_mut();

    if src.is_null() {
        return E_INVALIDARG;
    }

    len = ((lstrlenW(src as LPCWSTR) + 1) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as usize;
    *dest = CoTaskMemAlloc(len as SIZE_T) as *mut WCHAR;
    if (*dest).is_null() {
        return E_OUTOFMEMORY;
    }

    memcpy(
        *dest as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        len,
    );

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHStrDupA(src: *const i8, dest: *mut *mut WCHAR) -> HRESULT {
    let mut len: DWORD = 0;

    *dest = core::ptr::null_mut::<WCHAR>();

    if src.is_null() {
        return E_INVALIDARG;
    }

    len = MultiByteToWideChar(
        CP_ACP,
        0,
        src,
        -1,
        core::ptr::null_mut::<WCHAR>(),
        0,
    ) as DWORD;
    *dest = CoTaskMemAlloc((len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T))
        as *mut WCHAR;
    if (*dest).is_null() {
        return E_OUTOFMEMORY;
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        src,
        -1,
        *dest,
        len as i32,
    );

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHAnsiToAnsi(src: *const i8, dest: *mut i8, dest_len: i32) -> DWORD {
    let mut ret: DWORD = 0;

    if src.is_null() || dest.is_null() || dest_len <= 0 {
        return 0;
    }

    lstrcpynA(dest, src, dest_len);
    ret = strlen(dest) as DWORD;

    return if *src.offset(ret as isize) != 0 { 0 } else { ret + 1 };
}

#[no_mangle]
pub unsafe extern "system" fn SHUnicodeToAnsi(src: *const WCHAR, dest: *mut c_char, dest_len: c_int) -> DWORD {
    let mut ret: c_int = 1;

    if dest.is_null() || dest_len == 0 {
        return 0 as DWORD;
    }

    if !src.is_null() {
        ret = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            src as LPCWSTR,
            -(1 as c_int),
            dest as LPSTR,
            dest_len,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        if ret == 0 {
            *dest.offset((dest_len - 1 as c_int) as isize) = 0 as c_char;
            ret = dest_len;
        }
    } else {
        *dest.offset(0 as isize) = 0 as c_char;
    }

    ret as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn SHUnicodeToUnicode(src: *const WCHAR, dest: *mut WCHAR, dest_len: i32) -> DWORD {
    let mut ret: DWORD = 0;
    if src.is_null() || dest.is_null() || dest_len <= 0 {
        return 0;
    }
    lstrcpynW(dest as LPWSTR, src as LPCWSTR, dest_len);
    ret = lstrlenW(dest as LPCWSTR) as DWORD;
    return if *src.offset(ret as isize) != 0 {
        0
    } else {
        ret.wrapping_add(1)
    };
}

#[no_mangle]
pub unsafe extern "system" fn SHAnsiToUnicode(src: *const core::ffi::c_char, dest: *mut WCHAR, dest_len: core::ffi::c_int) -> DWORD {
    let mut ret: core::ffi::c_int = 1;

    if dest.is_null() || dest_len == 0 {
        return 0;
    }

    if !src.is_null() {
        ret = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, src as LPCSTR, -(1 as core::ffi::c_int), dest as LPWSTR, dest_len);
        if ret == 0 {
            *dest.offset((dest_len - 1 as core::ffi::c_int) as isize) = 0;
            ret = dest_len;
        }
    } else {
        *dest.offset(0 as isize) = 0;
    }

    return ret as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegDuplicateHKey(hKey: HKEY) -> HKEY {
    let mut newKey: HKEY = core::ptr::null_mut();

    RegOpenKeyExW(hKey, core::ptr::null(), 0, MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(newKey));

    return newKey;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteEmptyKeyW(hkey: HKEY, subkey: *const WCHAR) -> DWORD {
    let mut ret: DWORD;
    let mut count: DWORD = 0;
    let mut hsubkey: HKEY = core::ptr::null_mut();

    ret = RegOpenKeyExW(hkey, subkey as LPCWSTR, 0, KEY_READ, core::ptr::addr_of_mut!(hsubkey)) as DWORD;
    if ret == 0 {
        ret = RegQueryInfoKeyW(hsubkey, core::ptr::null_mut::<WCHAR>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::addr_of_mut!(count), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<DWORD>(), core::ptr::null_mut::<_FILETIME>()) as DWORD;
        RegCloseKey(hsubkey);
        if ret == 0 {
            if count != 0 {
                ret = ERROR_KEY_HAS_CHILDREN;
            } else {
                ret = RegDeleteKeyW(hkey, subkey as LPCWSTR) as DWORD;
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteEmptyKeyA(hkey: HKEY, subkey: *const i8) -> DWORD {
    let mut subkeyW: *mut WCHAR = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0".as_ptr() as *const i8,
            b"SHDeleteEmptyKeyA\0".as_ptr() as *const i8,
            1873i32,
            b"(%p, %s)\n\0".as_ptr() as *const i8,
            hkey,
            debugstr_a(subkey),
        );
    }
    if !subkey.is_null() && SHStrDupA(subkey, core::ptr::addr_of_mut!(subkeyW)) < 0 {
        return ERROR_OUTOFMEMORY;
    }
    ret = SHDeleteEmptyKeyW(hkey, subkeyW as *const WCHAR);
    CoTaskMemFree(subkeyW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteKeyW(hkey: HKEY, subkey: *const WCHAR) -> DWORD {
    // TRACE("(%p, %s)\n", hkey, debugstr_w(subkey));
    (RegDeleteTreeW(hkey, subkey)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteKeyA(hkey: HKEY, subkey: *const i8) -> DWORD {
    // TRACE("(%p, %s)\n", hkey, debugstr_a(subkey));
    (RegDeleteTreeA(hkey, subkey)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteValueW(hkey: HKEY, subkey: *const WCHAR, value: *const WCHAR) -> DWORD {
    let mut hsubkey: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;

    ret = RegOpenKeyExW(
        hkey,
        subkey as LPCWSTR,
        0 as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hsubkey),
    ) as DWORD;
    if ret == 0 {
        ret = RegDeleteValueW(hsubkey, value as LPCWSTR) as DWORD;
        RegCloseKey(hsubkey);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHDeleteValueA(
    mut hkey: HKEY,
    mut subkey: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
) -> DWORD {
    let mut subkeyW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut valueW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut ret: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SHDeleteValueA\0" as *const u8 as *const ::core::ffi::c_char,
            1931 as ::core::ffi::c_int,
            b"(%p, %s, %s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hkey,
            debugstr_a(subkey),
            debugstr_a(value),
        );
    }
    if !subkey.is_null()
        && (SHStrDupA(subkey, &raw mut subkeyW)) < 0 as HRESULT
    {
        return ERROR_OUTOFMEMORY as DWORD;
    }
    if !value.is_null()
        && (SHStrDupA(value, &raw mut valueW)) < 0 as HRESULT
    {
        CoTaskMemFree(subkeyW as LPVOID);
        return ERROR_OUTOFMEMORY as DWORD;
    }

    ret = SHDeleteValueW(hkey, subkeyW as *const WCHAR, valueW as *const WCHAR);
    CoTaskMemFree(subkeyW as LPVOID);
    CoTaskMemFree(valueW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHCopyKeyA(hkey_src: HKEY, subkey: *const i8, hkey_dst: HKEY, reserved: DWORD) -> DWORD {
    let mut subkeyW: *mut WCHAR = core::ptr::null_mut();
    let mut ret: DWORD;

    if !subkey.is_null() && (SHStrDupA(subkey, core::ptr::addr_of_mut!(subkeyW)) < 0) {
        return 0;
    }

    ret = SHCopyKeyW(hkey_src, subkeyW as *const WCHAR, hkey_dst, reserved);
    CoTaskMemFree(subkeyW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHCopyKeyW(
    hkey_src: HKEY,
    subkey: *const WCHAR,
    hkey_dst: HKEY,
    reserved: DWORD,
) -> DWORD {
    let mut hkey_src = hkey_src;
    let mut key_count: DWORD = 0;
    let mut value_count: DWORD = 0;
    let mut max_key_len: DWORD = 0;
    let mut name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ptr_name: *mut WCHAR = core::ptr::addr_of_mut!(name) as *mut WCHAR;
    let mut buff: [BYTE; 1024] = [0; 1024];
    let mut ptr: *mut BYTE = core::ptr::addr_of_mut!(buff) as *mut BYTE;
    let mut max_data_len: DWORD = 0;
    let mut i: DWORD;
    let mut ret: DWORD = 0;

    if hkey_dst.is_null() || hkey_src.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    if !subkey.is_null() {
        ret = RegOpenKeyExW(
            hkey_src,
            subkey as LPCWSTR,
            0 as DWORD,
            KEY_ALL_ACCESS as REGSAM,
            core::ptr::addr_of_mut!(hkey_src),
        ) as DWORD;
    }

    if ret != 0 {
        hkey_src = core::ptr::null_mut::<HKEY__>();
    } else {
        let mut max_value_len: DWORD = 0;

        ret = RegQueryInfoKeyW(
            hkey_src,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(key_count),
            core::ptr::addr_of_mut!(max_key_len),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(value_count),
            core::ptr::addr_of_mut!(max_value_len),
            core::ptr::addr_of_mut!(max_data_len),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<FILETIME>(),
        ) as DWORD;
        if ret == 0 {
            /* Get max size for key/value names */
            max_key_len = max!(max_key_len, max_value_len);

            let fresh35 = max_key_len;
            max_key_len = max_key_len.wrapping_add(1);
            if fresh35 > (MAX_PATH - 1) as DWORD {
                ptr_name = malloc(
                    (max_key_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                ) as *mut WCHAR;
            }

            if max_data_len as usize > core::mem::size_of::<[BYTE; 1024]>() {
                ptr = malloc(max_data_len as usize) as *mut BYTE;
            }

            if ptr_name.is_null() || ptr.is_null() {
                ret = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            }
        }
    }

    i = 0 as DWORD;
    while i < key_count && ret == 0 {
        let mut hsubkey_src: HKEY = core::ptr::null_mut::<HKEY__>();
        let mut hsubkey_dst: HKEY = core::ptr::null_mut::<HKEY__>();
        let mut length: DWORD = max_key_len;

        ret = RegEnumKeyExW(
            hkey_src,
            i,
            ptr_name as LPWSTR,
            core::ptr::addr_of_mut!(length),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<FILETIME>(),
        ) as DWORD;
        if ret == 0 {
            ret = RegOpenKeyExW(
                hkey_src,
                ptr_name as LPCWSTR,
                0 as DWORD,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(hsubkey_src),
            ) as DWORD;
            if ret == 0 {
                /* Create destination sub key */
                ret = RegCreateKeyW(
                    hkey_dst,
                    ptr_name as LPCWSTR,
                    core::ptr::addr_of_mut!(hsubkey_dst),
                ) as DWORD;
                if ret == 0 {
                    /* Recursively copy keys and values from the sub key */
                    ret = SHCopyKeyW(
                        hsubkey_src,
                        core::ptr::null::<WCHAR>(),
                        hsubkey_dst,
                        0 as DWORD,
                    );
                    RegCloseKey(hsubkey_dst);
                }
            }
            RegCloseKey(hsubkey_src);
        }
        i = i.wrapping_add(1);
    }

    /* Copy all the values in this key */
    i = 0 as DWORD;
    while i < value_count && ret == 0 {
        let mut length_0: DWORD = max_key_len;
        let mut type_0: DWORD = 0;
        let mut data_len: DWORD = max_data_len;

        ret = RegEnumValueW(
            hkey_src,
            i,
            ptr_name as LPWSTR,
            core::ptr::addr_of_mut!(length_0),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(type_0),
            ptr as LPBYTE,
            core::ptr::addr_of_mut!(data_len),
        ) as DWORD;
        if ret == 0 {
            ret = SHSetValueW(
                hkey_dst,
                core::ptr::null::<WCHAR>(),
                ptr_name as *const WCHAR,
                type_0,
                ptr as *const core::ffi::c_void,
                data_len,
            );
        }
        i = i.wrapping_add(1);
    }

    /* Free buffers if allocated */
    if ptr_name != core::ptr::addr_of_mut!(name) as *mut WCHAR {
        free(ptr_name as *mut core::ffi::c_void);
    }
    if ptr != core::ptr::addr_of_mut!(buff) as *mut BYTE {
        free(ptr as *mut core::ffi::c_void);
    }

    if !subkey.is_null() && !hkey_src.is_null() {
        RegCloseKey(hkey_src);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumKeyExA(
    hkey: HKEY,
    index: DWORD,
    subkey: *mut CHAR,
    length: *mut DWORD,
) -> LONG {
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0".as_ptr() as *const i8,
            b"SHEnumKeyExA\0".as_ptr() as *const i8,
            2061,
            b"%p, %ld, %s, %p.\n\0".as_ptr() as *const i8,
            hkey,
            index,
            debugstr_a(subkey),
            length,
        );
    }
    return RegEnumKeyExA(
        hkey,
        index,
        subkey,
        length,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<CHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<_FILETIME>(),
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumKeyExW(
    hkey: HKEY,
    index: DWORD,
    subkey: *mut WCHAR,
    length: *mut DWORD,
) -> LONG {
    if ((*__wine_dbch___default).flags as core::ffi::c_int)
        & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SHEnumKeyExW\0" as *const u8 as *const core::ffi::c_char,
            2071 as core::ffi::c_int,
            b"%p, %ld, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            hkey,
            index,
            debugstr_w(subkey),
            length,
        );
    }
    return RegEnumKeyExW(
        hkey,
        index,
        subkey as LPWSTR,
        length as LPDWORD,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<WCHAR>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<_FILETIME>(),
    ) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumValueA(
    hkey: HKEY,
    index: DWORD,
    value: *mut ::core::ffi::c_char,
    length: *mut DWORD,
    type_: *mut DWORD,
    data: *mut ::core::ffi::c_void,
    data_len: *mut DWORD,
) -> LONG {
    return RegEnumValueA(
        hkey,
        index,
        value,
        length,
        core::ptr::null_mut::<DWORD>(),
        type_,
        (data).cast(),
        data_len,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumValueW(hkey: HKEY, index: DWORD, value: *mut WCHAR, length: *mut DWORD, type_0: *mut DWORD, data: *mut core::ffi::c_void, data_len: *mut DWORD) -> LONG {
    return RegEnumValueW(hkey, index, value, length, core::ptr::null_mut::<DWORD>(), type_0, (data).cast(), data_len);
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryValueExW(
    hkey: HKEY,
    name: *const WCHAR,
    reserved: *mut DWORD,
    type_0: *mut DWORD,
    buff: *mut ::core::ffi::c_void,
    buff_len: *mut DWORD,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut value_type: DWORD = 0;
    let mut data_len: DWORD = 0 as DWORD;

    if !buff_len.is_null() {
        data_len = *buff_len;
    }

    ret = RegQueryValueExW(
        hkey,
        name as LPCWSTR,
        reserved as LPDWORD,
        core::ptr::addr_of_mut!(value_type),
        buff as LPBYTE,
        core::ptr::addr_of_mut!(data_len),
    ) as DWORD;
    if ret != ERROR_SUCCESS as DWORD && ret != ERROR_MORE_DATA as DWORD {
        return ret;
    }

    if !buff_len.is_null() && value_type == REG_EXPAND_SZ as DWORD {
        let mut length: DWORD = 0;
        let mut value: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

        if buff.is_null() || ret == ERROR_MORE_DATA as DWORD {
            length = data_len;
            value = malloc(length as usize) as *mut WCHAR;
            RegQueryValueExW(
                hkey,
                name as LPCWSTR,
                reserved as LPDWORD,
                core::ptr::null_mut::<DWORD>(),
                value as LPBYTE,
                core::ptr::addr_of_mut!(length),
            );
            length = ExpandEnvironmentStringsW(
                value as LPCWSTR,
                core::ptr::null_mut::<WCHAR>(),
                0 as DWORD,
            );
        } else {
            length = ((lstrlenW(buff as LPCWSTR) + 1 as ::core::ffi::c_int) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
            value = malloc(length as usize) as *mut WCHAR;
            memcpy(value as *mut ::core::ffi::c_void, buff, length as usize);
            length = ExpandEnvironmentStringsW(
                value as LPCWSTR,
                buff as LPWSTR,
                *buff_len / core::mem::size_of::<WCHAR>() as DWORD,
            );
            if length > *buff_len {
                ret = ERROR_MORE_DATA as DWORD;
            }
        }
        data_len = max!(data_len, length);
        free(value as *mut ::core::ffi::c_void);
    }

    if !type_0.is_null() {
        *type_0 = if value_type == REG_EXPAND_SZ as DWORD {
            REG_SZ as DWORD
        } else {
            value_type
        };
    }
    if !buff_len.is_null() {
        *buff_len = data_len;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryValueExA(
    hkey: HKEY,
    name: LPCSTR,
    reserved: *mut DWORD,
    type_0: *mut DWORD,
    buff: *mut core::ffi::c_void,
    buff_len: *mut DWORD,
) -> DWORD {
    let mut ret: DWORD;
    let mut value_type: DWORD = core::mem::zeroed();
    let mut data_len: DWORD = 0;

    if !buff_len.is_null() {
        data_len = *buff_len;
    }

    ret = RegQueryValueExA(
        hkey,
        name,
        reserved,
        core::ptr::addr_of_mut!(value_type),
        buff as LPBYTE,
        core::ptr::addr_of_mut!(data_len),
    ) as DWORD;
    if ret != ERROR_SUCCESS && ret != ERROR_MORE_DATA {
        return ret;
    }

    if !buff_len.is_null() && value_type == REG_EXPAND_SZ {
        let mut length: DWORD;
        let mut value: *mut CHAR;

        if buff.is_null() || ret == ERROR_MORE_DATA {
            length = data_len;
            value = malloc(length as usize) as *mut CHAR;
            RegQueryValueExA(
                hkey,
                name,
                reserved,
                core::ptr::null_mut::<DWORD>(),
                value as LPBYTE,
                core::ptr::addr_of_mut!(length),
            );
            length = ExpandEnvironmentStringsA(
                value as LPCSTR,
                core::ptr::null_mut::<CHAR>(),
                0,
            );
        } else {
            length = (strlen(buff as *const CHAR) as DWORD) + 1;
            value = malloc(length as usize) as *mut CHAR;
            memcpy(value as *mut core::ffi::c_void, buff, length as usize);
            length = ExpandEnvironmentStringsA(value as LPCSTR, buff as LPSTR, *buff_len);
            if length > *buff_len {
                ret = ERROR_MORE_DATA;
            }
        }
        data_len = max!(data_len, length);
        free(value as *mut core::ffi::c_void);
    }

    if !type_0.is_null() {
        *type_0 = if value_type == REG_EXPAND_SZ {
            REG_SZ
        } else {
            value_type
        };
    }
    if !buff_len.is_null() {
        *buff_len = data_len;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueA(
    hkey: HKEY,
    subkey: LPCSTR,
    value: LPCSTR,
    type_: *mut DWORD,
    data: LPVOID,
    data_len: *mut DWORD,
) -> DWORD {
    let mut hsubkey: HKEY = core::ptr::null_mut();
    let mut ret: DWORD = 0;

    if !subkey.is_null() {
        ret = RegOpenKeyExA(
            hkey,
            subkey,
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hsubkey),
        ) as DWORD;
    }

    if ret == 0 {
        ret = SHQueryValueExA(
            if !hsubkey.is_null() { hsubkey } else { hkey },
            value,
            core::ptr::null_mut(),
            type_,
            data,
            data_len,
        );
        if !subkey.is_null() {
            RegCloseKey(hsubkey);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetValueW(hkey: HKEY, subkey: *const WCHAR, value: *const WCHAR, type_: *mut DWORD, data: *mut core::ffi::c_void, data_len: *mut DWORD) -> DWORD {
    let mut hsubkey: HKEY = core::ptr::null_mut();
    let mut ret: DWORD = 0;

    if !subkey.is_null() {
        ret = RegOpenKeyExW(hkey, subkey as LPCWSTR, 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hsubkey)) as DWORD;
    }

    if ret == 0 {
        ret = SHQueryValueExW(if !hsubkey.is_null() { hsubkey } else { hkey }, value, core::ptr::null_mut(), type_, data, data_len);
        if !subkey.is_null() {
            RegCloseKey(hsubkey);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetIntW(hkey: HKEY, value: *const WCHAR, default_value: i32) -> i32 {
    let mut buff: [WCHAR; 32] = [0; 32];
    let mut buff_len: DWORD = 0;

    buff[0] = 0;
    buff_len = core::mem::size_of::<[WCHAR; 32]>() as DWORD;
    if SHQueryValueExW(
        hkey,
        value,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(buff) as *mut WCHAR as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(buff_len),
    ) != 0
    {
        return default_value;
    }

    let buff_p: *mut WCHAR = core::ptr::addr_of_mut!(buff) as *mut WCHAR;
    if *buff_p >= '0' as WCHAR && *buff_p <= '9' as WCHAR {
        return wcstol(
            buff_p,
            core::ptr::null_mut(),
            10,
        ) as i32;
    }

    default_value
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetPathA(hkey: HKEY, subkey: *const core::ffi::c_char, value: *const core::ffi::c_char, path: *mut core::ffi::c_char, flags: DWORD) -> DWORD {
    let mut length: DWORD = MAX_PATH as DWORD;

    return SHGetValueA(hkey, subkey, value, core::ptr::null_mut(), path as *mut core::ffi::c_void, core::ptr::addr_of_mut!(length));
}

#[no_mangle]
pub unsafe extern "system" fn SHRegGetPathW(hkey: HKEY, subkey: *const WCHAR, value: *const WCHAR, path: *mut WCHAR, flags: DWORD) -> DWORD {
    let mut length: DWORD = MAX_PATH as DWORD;

    return SHGetValueW(hkey, subkey, value, core::ptr::null_mut(), path as *mut core::ffi::c_void, &raw mut length);
}

#[no_mangle]
pub unsafe extern "system" fn SHSetValueW(
    hkey: HKEY,
    subkey: *const WCHAR,
    value: *const WCHAR,
    type_: DWORD,
    data: *const core::ffi::c_void,
    data_len: DWORD,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS as DWORD;
    let mut dummy: DWORD = 0;
    let mut hsubkey: HKEY = core::ptr::null_mut();

    if !subkey.is_null() && *subkey != 0 {
        ret = RegCreateKeyExW(
            hkey,
            subkey,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hsubkey),
            core::ptr::addr_of_mut!(dummy),
        ) as DWORD;
    } else {
        hsubkey = hkey;
    }

    if ret == 0 {
        ret = RegSetValueExW(
            hsubkey,
            value,
            0 as DWORD,
            type_,
            data as *const BYTE,
            data_len,
        ) as DWORD;
        if hsubkey != hkey {
            RegCloseKey(hsubkey);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetValueA(
    hkey: HKEY,
    subkey: *const i8,
    value: *const i8,
    type_: DWORD,
    data: *const core::ffi::c_void,
    data_len: DWORD,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS as DWORD;
    let mut dummy: DWORD = 0;
    let mut hsubkey: HKEY = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/shcore/main.c\0"
                as *const u8 as *const i8,
            b"SHSetValueA\0" as *const u8 as *const i8,
            2326 as i32,
            b"%p, %s, %s, %ld, %p, %ld.\n\0" as *const u8 as *const i8,
            hkey,
            debugstr_a(subkey),
            debugstr_a(value),
            type_,
            data,
            data_len,
        );
    }
    if !subkey.is_null() && *subkey as i32 != 0 {
        ret = RegCreateKeyExA(
            hkey,
            subkey,
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hsubkey),
            core::ptr::addr_of_mut!(dummy),
        ) as DWORD;
    } else {
        hsubkey = hkey;
    }
    if ret == 0 {
        ret = RegSetValueExA(
            hsubkey,
            value,
            0 as DWORD,
            type_,
            data as *const BYTE,
            data_len,
        ) as DWORD;
        if hsubkey != hkey {
            RegCloseKey(hsubkey);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetPathA(hkey: HKEY, subkey: *const core::ffi::c_char, value: *const core::ffi::c_char, path: *const core::ffi::c_char, flags: DWORD) -> DWORD {
    return SHSetValueA(hkey, subkey, value, REG_SZ as DWORD, path as *const core::ffi::c_void, lstrlenA(path as LPCSTR) as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn SHRegSetPathW(hkey: HKEY, subkey: *const WCHAR, value: *const WCHAR, path: *const WCHAR, flags: DWORD) -> DWORD {
    return SHSetValueW(hkey, subkey, value, REG_SZ as DWORD, path as *const core::ffi::c_void, lstrlenW(path as LPCWSTR) as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryInfoKeyA(hkey: HKEY, subkeys: *mut DWORD, subkey_max: *mut DWORD, values: *mut DWORD, value_max: *mut DWORD) -> LONG {
    return RegQueryInfoKeyA(hkey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), subkeys, subkey_max, core::ptr::null_mut(), values, value_max, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryInfoKeyW(hkey: HKEY, subkeys: *mut DWORD, subkey_max: *mut DWORD, values: *mut DWORD, value_max: *mut DWORD) -> LONG {
    return RegQueryInfoKeyW(hkey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), subkeys, subkey_max, core::ptr::null_mut(), values, value_max, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn IsOS(feature: DWORD) -> BOOL {
    let mut platform: DWORD = 0;
    let mut majorv: DWORD = 0;
    let mut minorv: DWORD = 0;
    let mut osvi: OSVERSIONINFOEXA = core::mem::zeroed();

    osvi.dwOSVersionInfoSize = ::core::mem::size_of::<OSVERSIONINFOEXA>() as DWORD;
    if GetVersionExA(core::ptr::addr_of_mut!(osvi) as *mut OSVERSIONINFOA) == 0 {
        osvi.dwOSVersionInfoSize = ::core::mem::size_of::<OSVERSIONINFOA>() as DWORD;
        if GetVersionExA(core::ptr::addr_of_mut!(osvi) as *mut OSVERSIONINFOA) == 0 {
            ERR!("GetVersionEx failed\n");
            return (FALSE) as BOOL;
        }
        osvi.wProductType = VER_NT_WORKSTATION as BYTE;
        osvi.wSuiteMask = 0 as WORD;
    }

    majorv = osvi.dwMajorVersion;
    minorv = osvi.dwMinorVersion;
    platform = osvi.dwPlatformId;

    match feature {
        OS_WIN32SORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32s || platform == VER_PLATFORM_WIN32_WINDOWS) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_NT => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN95ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_NT4ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && majorv >= 4) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000ORGREATER_ALT | OS_WIN2000ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && majorv >= 5) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN98ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS && minorv >= 10) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN98_GOLD => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS && minorv == 10) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000PRO => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && majorv >= 5) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000SERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000ADVSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000DATACENTER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WIN2000TERMINAL => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_EMBEDDED => {
            FIXME!("(OS_EMBEDDED) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_TERMINALCLIENT => {
            FIXME!("(OS_TERMINALCLIENT) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_TERMINALREMOTEADMIN => {
            FIXME!("(OS_TERMINALREMOTEADMIN) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_WIN95_GOLD => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS && minorv == 0) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_MEORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS && minorv >= 90) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_XPORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && ((majorv >= 5 && minorv >= 1) || majorv >= 6)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_HOME => {
            /* "5.1 or later": the Wine test is FALSE at 10.0 (minor 0) */
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (majorv > 5 || (majorv == 5 && minorv >= 1))) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_PROFESSIONAL => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_DATACENTER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_ADVSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && majorv >= 5) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_SERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_TERMINALSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_PERSONALTERMINALSERVER => {
            /* "5.1 or later": the Wine test is FALSE at 10.0 (minor 0) */
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && (majorv > 5 || (majorv == 5 && minorv >= 1))) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_FASTUSERSWITCHING => {
            FIXME!("(OS_FASTUSERSWITCHING) What should we return here?\n");
            return (TRUE) as BOOL;
        }
        OS_WELCOMELOGONUI => {
            FIXME!("(OS_WELCOMELOGONUI) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_DOMAINMEMBER => {
            FIXME!("(OS_DOMAINMEMBER) What should we return here?\n");
            return (TRUE) as BOOL;
        }
        OS_ANYSERVER => {
            let ret: BOOL = (osvi.wProductType > (VER_NT_WORKSTATION as u8)) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_WOW6432 => {
            let mut is_wow64: BOOL = 0;
            IsWow64Process(GetCurrentProcess(), core::ptr::addr_of_mut!(is_wow64));
            return is_wow64;
        }
        OS_WEBSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_SMALLBUSINESSSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_TABLETPC => {
            FIXME!("(OS_TABLETPC) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_SERVERADMINUI => {
            let mut value: DWORD = FALSE as DWORD;
            let mut size: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;
            let mut hKey: HKEY = core::ptr::null_mut();

            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                RUSTCONV_WSTR_8888EF3C3D8B.as_ptr(),
                0,
                MAXIMUM_ALLOWED,
                core::ptr::addr_of_mut!(hKey),
            );

            if !hKey.is_null() {
                SHQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_F6C54912C188.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(value) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(size),
                );
                RegCloseKey(hKey);
            }
            let ret: BOOL = value as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        OS_MEDIACENTER => {
            FIXME!("(OS_MEDIACENTER) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_APPLIANCE => {
            FIXME!("(OS_APPLIANCE) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        0x25 => {
            /*OS_VISTAORGREATER*/
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT && majorv >= 6) as BOOL;
            TRACE!("(%#lx) ret %d\n", feature, ret);
            return ret;
        }
        _ => {}
    }

    WARN!("(%#lx) unknown parameter\n", feature);

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SubscribeFeatureStateChangeNotification(
    subscription: *mut FEATURE_STATE_CHANGE_SUBSCRIPTION,
    // featurestagingapi.h: `typedef void WINAPI FEATURE_STATE_CHANGE_CALLBACK(void*)` -- a
    // function TYPE, so the C's `FEATURE_STATE_CHANGE_CALLBACK *` is a function pointer.
    callback: Option<unsafe extern "system" fn(*mut core::ffi::c_void)>,
    context: *mut core::ffi::c_void,
) {
    // FIXME("(%p, %p, %p) stub\n", subscription, callback, context);
}

#[no_mangle]
pub unsafe extern "system" fn GetFeatureEnabledState(
    feature: UINT32,
    change_time: FEATURE_CHANGE_TIME,
) -> FEATURE_ENABLED_STATE {
    // FIXME("(%u, %u) stub\n", feature, change_time);
    FEATURE_ENABLED_STATE_DEFAULT
}

#[no_mangle]
pub unsafe extern "system" fn RegisterScaleChangeEvent(
    handle: HANDLE,
    cookie: *mut DWORD_PTR,
) -> HRESULT {
    // FIXME("(%p, %p) stub\n", handle, cookie);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn RegisterScaleChangeNotifications(display_device: DISPLAY_DEVICE_TYPE, hwnd: HWND, msg: UINT, cookie: *mut DWORD) -> HRESULT {
    if !cookie.is_null() {
        *cookie = 0;
    }

    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn CreateRandomAccessStreamOverStream(
    stream: *mut IStream,
    options: BSOS_OPTIONS,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("(%p, %d, %s, %p) stub\n", stream, options, debugstr_guid(riid), ppv);
    E_NOTIMPL
}

// COM vtable tables — generated from main.c by vtbl_gen.py
/// main.c:816 `static const IStreamVtbl memstreamvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static memstreamvtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(shstream_QueryInterface),
    AddRef: Some(shstream_AddRef),
    Release: Some(memstream_Release),
    Read: Some(memstream_Read),
    Write: Some(memstream_Write),
    Seek: Some(memstream_Seek),
    SetSize: Some(memstream_SetSize),
    CopyTo: Some(shstream_CopyTo),
    Commit: Some(shstream_Commit),
    Revert: Some(shstream_Revert),
    LockRegion: Some(shstream_LockRegion),
    UnlockRegion: Some(shstream_UnlockRegion),
    Stat: Some(memstream_Stat),
    Clone: Some(shstream_Clone),
};

/// main.c:1076 `static const IStreamVtbl filestreamvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static filestreamvtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(shstream_QueryInterface),
    AddRef: Some(shstream_AddRef),
    Release: Some(filestream_Release),
    Read: Some(filestream_Read),
    Write: Some(filestream_Write),
    Seek: Some(filestream_Seek),
    SetSize: Some(filestream_SetSize),
    CopyTo: Some(filestream_CopyTo),
    Commit: Some(filestream_Commit),
    Revert: Some(shstream_Revert),
    LockRegion: Some(shstream_LockRegion),
    UnlockRegion: Some(shstream_UnlockRegion),
    Stat: Some(filestream_Stat),
    Clone: Some(shstream_Clone),
};

/// main.c:1245 `static const IStreamVtbl regstreamvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static regstreamvtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(shstream_QueryInterface),
    AddRef: Some(shstream_AddRef),
    Release: Some(regstream_Release),
    Read: Some(memstream_Read),
    Write: Some(memstream_Write),
    Seek: Some(memstream_Seek),
    SetSize: Some(memstream_SetSize),
    CopyTo: Some(shstream_CopyTo),
    Commit: Some(shstream_Commit),
    Revert: Some(shstream_Revert),
    LockRegion: Some(shstream_LockRegion),
    UnlockRegion: Some(shstream_UnlockRegion),
    Stat: Some(memstream_Stat),
    Clone: Some(shstream_Clone),
};

/// main.c:1381 `static const IStreamVtbl dummystreamvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static dummystreamvtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(shstream_QueryInterface),
    AddRef: Some(dummystream_AddRef),
    Release: Some(dummystream_Release),
    Read: Some(dummystream_Read),
    Write: Some(memstream_Write),
    Seek: Some(memstream_Seek),
    SetSize: Some(memstream_SetSize),
    CopyTo: Some(shstream_CopyTo),
    Commit: Some(shstream_Commit),
    Revert: Some(shstream_Revert),
    LockRegion: Some(shstream_LockRegion),
    UnlockRegion: Some(shstream_UnlockRegion),
    Stat: Some(memstream_Stat),
    Clone: Some(shstream_Clone),
};

/// main.c:1468 `static const IUnknownVtbl threadrefvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static threadrefvtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(threadref_QueryInterface),
    AddRef: Some(threadref_AddRef),
    Release: Some(threadref_Release),
};
