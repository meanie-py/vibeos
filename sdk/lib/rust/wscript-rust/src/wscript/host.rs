use crate::wscript::main::schedule_timeout;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5BD1A4ED912F: [u16; 20] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0053, 0x0063,
    0x0072, 0x0069, 0x0070, 0x0074, 0x0020, 0x0048, 0x006F, 0x0073, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_F078EF1160A2: [u16; 5] = [0x006Eu16, 0x0075, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut wshVersionW: [WCHAR; 4] =
    unsafe { ::core::mem::transmute::<[u8; 8], [WCHAR; 4]>(*b"5\0.\08\0\0\0") };

#[no_mangle]
pub static mut wshInteractive: VARIANT_BOOL = VARIANT_FALSE;

#[no_mangle]
pub static mut wshTimeout: LONG = 0 as LONG;

#[no_mangle]
pub static mut host_obj: IHost = unsafe {
    IHost {
        lpVtbl: &raw const HostVtbl,
    }
};

pub const BUILDVERSION: i32 = 16535;

// WINE_DEFAULT_DEBUG_CHANNEL(wscript): debug channel declaration, not emitted here.
// COBJMACROS and CONST_VTABLE are flag-only defines with no Rust counterpart.

unsafe extern "C" fn to_string(src: *mut VARIANT, dst: *mut BSTR) -> HRESULT {
    let mut v: VARIANT = core::mem::zeroed();
    let mut hres: HRESULT = 0;

    if V_VT!(src) == VT_NULL as VARTYPE {
        *dst = SysAllocString(RUSTCONV_WSTR_F078EF1160A2.as_ptr());
        return if !(*dst).is_null() {
            S_OK
        } else {
            E_OUTOFMEMORY
        };
    }

    (*core::ptr::addr_of_mut!(v)).n1.n2.vt = VT_EMPTY as VARTYPE;
    hres = VariantChangeType(
        core::ptr::addr_of_mut!(v),
        src,
        0 as USHORT,
        VT_BSTR as VARTYPE,
    );
    if hres < 0 as HRESULT {
        WARN!(
            "Could not convert argument %s to string\n",
            debugstr_variant(src)
        );
        return hres;
    }

    *dst = V_BSTR!(core::ptr::addr_of!(v));
    return S_OK;
}

unsafe extern "C" fn print_string(string: *const WCHAR) {
    let mut count: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut len: DWORD = 0;
    let mut lena: DWORD = 0;
    let mut buf: *mut core::ffi::c_char = core::ptr::null_mut();
    if wshInteractive != 0 {
        MessageBoxW(
            core::ptr::null_mut(),
            string,
            RUSTCONV_WSTR_5BD1A4ED912F.as_ptr(),
            MB_OK,
        );
        return;
    }
    len = lstrlenW(string) as DWORD;
    ret = WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        string as *const core::ffi::c_void,
        len,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) as DWORD;
    if ret != 0 {
        WriteConsoleW(
            GetStdHandle(STD_OUTPUT_HANDLE),
            RUSTCONV_WSTR_80E09B2F645F.as_ptr() as *const core::ffi::c_void,
            lstrlenW(RUSTCONV_WSTR_80E09B2F645F.as_ptr()) as DWORD,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        );
        return;
    }
    lena = WideCharToMultiByte(
        GetOEMCP(),
        0,
        string,
        len as core::ffi::c_int,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    buf = malloc(len as usize) as *mut core::ffi::c_char;
    if buf.is_null() {
        return;
    }
    WideCharToMultiByte(
        GetOEMCP(),
        0,
        string,
        len as core::ffi::c_int,
        buf,
        lena as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    WriteFile(
        GetStdHandle(STD_OUTPUT_HANDLE),
        buf as *const core::ffi::c_void,
        lena,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );
    free(buf as *mut core::ffi::c_void);
    WriteFile(
        GetStdHandle(STD_OUTPUT_HANDLE),
        b"\r\n\0".as_ptr() as *const core::ffi::c_void,
        2,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );
}

unsafe extern "system" fn Host_QueryInterface(iface: *mut IHost, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    WINE_TRACE!(b"(%s %p)\n\0".as_ptr() as *const i8, wine_dbgstr_guid(riid), ppv);

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false
       || IsEqualGUID!(core::ptr::addr_of!(IID_IDispatch), riid) != false
       || IsEqualGUID!(core::ptr::addr_of!(IID_IHost), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    *ppv = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn Host_AddRef(iface: *mut IHost) -> ULONG {
    2
}
unsafe extern "system" fn Host_Release(iface: *mut IHost) -> ULONG {
    1
}
unsafe extern "system" fn Host_GetTypeInfoCount(iface: *mut IHost, pctinfo: *mut UINT) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, pctinfo);
    *pctinfo = 1;
    return S_OK;
}

unsafe extern "system" fn Host_GetTypeInfo(
    iface: *mut IHost,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    WINE_TRACE!(b"(%x %lx %p\n\0".as_ptr() as *const i8, iTInfo, lcid, ppTInfo);

    ITypeInfo_AddRef(host_ti);
    *ppTInfo = host_ti;
    return S_OK;
}

unsafe extern "system" fn Host_GetIDsOfNames(
    iface: *mut IHost,
    riid: REFIID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    WINE_TRACE!(
        b"(%s %p %d %lx %p)\n\0".as_ptr() as *const i8,
        wine_dbgstr_guid(riid),
        rgszNames,
        cNames,
        lcid,
        rgDispId,
    );

    return ITypeInfo_GetIDsOfNames(host_ti, rgszNames, cNames, rgDispId as *mut MEMBERID);
}

unsafe extern "system" fn Host_Invoke(
    iface: *mut IHost,
    dispIdMember: DISPID,
    riid: REFIID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    WINE_TRACE!(b"(%ld %p %p)\n\0".as_ptr() as *const i8, dispIdMember, pDispParams, pVarResult);

    return ITypeInfo_Invoke(
        host_ti,
        iface as PVOID,
        dispIdMember as MEMBERID,
        wFlags,
        pDispParams,
        pVarResult,
        pExcepInfo,
        puArgErr,
    );
}

unsafe extern "system" fn Host_get_Name(iface: *mut IHost, out_Name: *mut BSTR) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, out_Name);

    if (*out_Name).is_null() {
        *out_Name = SysAllocString(RUSTCONV_WSTR_5BD1A4ED912F.as_ptr());
    }
    if (*out_Name).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

unsafe extern "system" fn Host_get_Application(iface: *mut IHost, out_Dispatch: *mut *mut IDispatch) -> HRESULT {
    let _ = iface;
    WINE_TRACE!("(%p)\n", out_Dispatch);

    *out_Dispatch = core::ptr::addr_of_mut!(host_obj) as *mut IDispatch;
    return S_OK;
}

unsafe extern "system" fn Host_get_FullName(iface: *mut IHost, out_Path: *mut BSTR) -> HRESULT {
    let mut fullPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, out_Path);

    if GetModuleFileNameW(core::ptr::null_mut(), core::ptr::addr_of_mut!(fullPath) as LPWSTR, ARRAY_SIZE!(fullPath) as DWORD) == 0 {
        return E_FAIL;
    }
    *out_Path = SysAllocString(core::ptr::addr_of_mut!(fullPath) as *mut WCHAR);
    if (*out_Path).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}
unsafe extern "system" fn Host_get_Path(iface: *mut IHost, out_Path: *mut BSTR) -> HRESULT {
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut howMany: ::core::ffi::c_int = 0;
    let mut pos: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const ::core::ffi::c_char, out_Path);
    if GetModuleFileNameW(
        ::core::ptr::null_mut::<HINSTANCE__>(),
        ::core::ptr::addr_of_mut!(path) as LPWSTR,
        ARRAY_SIZE!(path) as DWORD,
    ) == 0 as DWORD
    {
        return E_FAIL;
    }
    pos = wcsrchr(
        ::core::ptr::addr_of_mut!(path) as *mut WCHAR,
        '\\' as WCHAR,
    ) as *mut WCHAR;
    howMany = pos.offset_from(::core::ptr::addr_of_mut!(path) as *mut WCHAR) as ::core::ffi::c_longlong
        as ::core::ffi::c_int;
    *out_Path = SysAllocStringLen(
        ::core::ptr::addr_of_mut!(path) as *mut WCHAR,
        howMany as UINT,
    );
    if (*out_Path).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

unsafe extern "system" fn Host_get_Interactive(iface: *mut IHost, out_Interactive: *mut VARIANT_BOOL) -> HRESULT {
    WINE_TRACE!("(%p)\n", out_Interactive);

    *out_Interactive = wshInteractive;
    return S_OK;
}

unsafe extern "system" fn Host_put_Interactive(iface: *mut IHost, v: VARIANT_BOOL) -> HRESULT {
    WINE_TRACE!("(%x)\n", v as core::ffi::c_int);

    wshInteractive = v;
    return S_OK;
}

unsafe extern "system" fn Host_Quit(iface: *mut IHost, ExitCode: i32) -> HRESULT {
    FIXME!("(%d) semi-stub: no script engine clean up\n", ExitCode);
    ExitProcess(ExitCode as UINT);
    return S_OK;
}
unsafe extern "system" fn Host_get_ScriptName(iface: *mut IHost, out_ScriptName: *mut BSTR) -> HRESULT {
    let mut scriptName: *mut WCHAR = core::ptr::null_mut();

    WINE_TRACE!("(%p)\n", out_ScriptName);

    scriptName = wcsrchr(core::ptr::addr_of_mut!(scriptFullName) as *mut WCHAR, '\\' as u16) as *mut WCHAR;
    scriptName = scriptName.add(1);
    *out_ScriptName = SysAllocString(scriptName);
    if (*out_ScriptName).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

unsafe extern "system" fn Host_get_ScriptFullName(iface: *mut IHost, out_ScriptFullName: *mut BSTR) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, out_ScriptFullName);

    *out_ScriptFullName = SysAllocString(core::ptr::addr_of_mut!(scriptFullName) as *mut WCHAR);
    if (*out_ScriptFullName).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

unsafe extern "system" fn Host_get_Arguments(iface: *mut IHost, out_Arguments: *mut *mut IArguments2) -> HRESULT {
    WINE_TRACE!(b"(%p)\n\0".as_ptr() as *const i8, out_Arguments);

    *out_Arguments = core::ptr::addr_of_mut!(arguments_obj);
    return S_OK;
}
unsafe extern "system" fn Host_get_Version(iface: *mut IHost, out_Version: *mut BSTR) -> HRESULT {
    WINE_TRACE!("(%p)\n", out_Version);

    *out_Version = SysAllocString(core::ptr::addr_of!(wshVersionW) as *const OLECHAR);
    if (*out_Version).is_null() {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

unsafe extern "system" fn Host_get_BuildVersion(iface: *mut IHost, out_Build: *mut i32) -> HRESULT {
    WINE_TRACE!("(%p)\n", out_Build);

    *out_Build = BUILDVERSION;
    return S_OK;
}

unsafe extern "system" fn Host_get_Timeout(iface: *mut IHost, out_Timeout: *mut LONG) -> HRESULT {
    TRACE!("(%p)\n", out_Timeout);
    *out_Timeout = wshTimeout;
    return S_OK;
}

unsafe extern "system" fn Host_put_Timeout(iface: *mut IHost, v: LONG) -> HRESULT {
    if v < 0 {
        return E_INVALIDARG;
    }
    wshTimeout = v;
    schedule_timeout(v);
    return S_OK;
}

unsafe extern "system" fn Host_CreateObject(
    iface: *mut IHost,
    ProgID: BSTR,
    Prefix: BSTR,
    out_Dispatch: *mut *mut IDispatch,
) -> HRESULT {
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let mut guid: GUID = core::mem::zeroed();
    let mut hres: HRESULT;

    if !Prefix.is_null() && *Prefix != 0 {
        return E_NOTIMPL;
    }

    hres = CLSIDFromProgID(ProgID as LPCOLESTR, core::ptr::addr_of_mut!(guid));
    if hres < 0 {
        return hres;
    }

    hres = CoCreateInstance(
        core::ptr::addr_of_mut!(guid),
        core::ptr::null_mut::<IUnknown>(),
        (CLSCTX_INPROC_SERVER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER) as DWORD,
        core::ptr::addr_of!(IID_IUnknown),
        core::ptr::addr_of_mut!(unk) as *mut LPVOID,
    );
    if hres < 0 {
        return hres;
    }

    hres = IUnknown_QueryInterface(
        unk,
        core::ptr::addr_of!(IID_IDispatch),
        out_Dispatch as *mut *mut core::ffi::c_void,
    );
    IUnknown_Release(unk);
    return hres;
}

unsafe extern "system" fn Host_Echo(iface: *mut IHost, args: *mut SAFEARRAY) -> HRESULT {
    let mut output: *mut WCHAR = core::ptr::null_mut();
    let mut ptr: *mut WCHAR = core::ptr::null_mut();
    let mut argc: u32 = 0;
    let mut i: u32 = 0;
    let mut len: u32 = 0;
    let mut ubound: LONG = 0;
    let mut lbound: LONG = 0;
    let mut argv: *mut VARIANT = core::ptr::null_mut();
    let mut strs: *mut BSTR = core::ptr::null_mut();
    let mut hres: HRESULT = 0;

    if SafeArrayGetDim(args) != 1 {
        return E_NOTIMPL;
    }

    SafeArrayGetLBound(args, 1, &mut lbound);
    SafeArrayGetUBound(args, 1, &mut ubound);

    hres = SafeArrayAccessData(args, &mut argv as *mut *mut VARIANT as *mut *mut core::ffi::c_void);
    if hres < 0 {
        return hres;
    }

    argc = (ubound - lbound + 1) as u32;
    strs = calloc((argc as usize) as _, (core::mem::size_of::<OLECHAR>()) as _) as *mut BSTR;
    if strs.is_null() {
        SafeArrayUnaccessData(args);
        return E_OUTOFMEMORY;
    }

    /* Len of spaces between arguments. */
    len = argc - 1;

    i = 0;
    while i < argc {
        hres = to_string(argv.add(i as usize), strs.add(i as usize));
        if hres < 0 {
            break;
        }

        len += SysStringLen(*strs.add(i as usize)) as u32;
        i += 1;
    }

    SafeArrayUnaccessData(args);
    if hres >= 0 {
        output = malloc(
            (len + 1) as usize * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        ptr = output;
        if !output.is_null() {
            i = 0;
            while i < argc {
                if i != 0 {
                    let fresh15 = ptr;
                    ptr = ptr.add(1);
                    *fresh15 = ' ' as u16;
                }
                len = SysStringLen(*strs.add(i as usize)) as u32;
                memcpy(
                    ptr as *mut core::ffi::c_void,
                    *strs.add(i as usize) as *const core::ffi::c_void,
                    len as usize * core::mem::size_of::<WCHAR>(),
                );
                ptr = ptr.add(len as usize);
                i += 1;
            }
            *ptr = 0;
        } else {
            hres = E_OUTOFMEMORY;
        }
    }

    i = 0;
    while i < argc {
        SysFreeString(*strs.add(i as usize));
        i += 1;
    }
    free(strs as *mut core::ffi::c_void);
    if hres < 0 {
        return hres;
    }

    print_string(output);

    free(output as *mut core::ffi::c_void);
    return S_OK;
}
unsafe extern "system" fn Host_GetObject(
    iface: *mut IHost,
    Pathname: BSTR,
    ProgID: BSTR,
    Prefix: BSTR,
    out_Dispatch: *mut *mut IDispatch,
) -> HRESULT {
    // WINE_FIXME("(%s %s %s %p)\n", wine_dbgstr_w(Pathname), wine_dbgstr_w(ProgID), wine_dbgstr_w(Prefix), out_Dispatch);
    E_NOTIMPL
}
unsafe extern "system" fn Host_DisconnectObject(
    iface: *mut IHost,
    Object: *mut IDispatch,
) -> HRESULT {
    // WINE_FIXME("(%p)\n", Object);
    E_NOTIMPL
}

unsafe extern "system" fn Host_Sleep(iface: *mut IHost, Time: LONG) -> HRESULT {
    UNREFERENCED_PARAMETER!(iface);
    TRACE!("(%ld)\n", Time);
    if Time < 0 {
        return E_INVALIDARG;
    }
    Sleep(Time as DWORD);
    return S_OK;
}
unsafe extern "system" fn Host_ConnectObject(
    iface: *mut IHost,
    Object: *mut IDispatch,
    Prefix: BSTR,
) -> HRESULT {
    // WINE_FIXME("(%p %s)\n", Object, wine_dbgstr_w(Prefix));
    E_NOTIMPL
}
unsafe extern "system" fn Host_get_StdIn(
    iface: *mut IHost,
    ppts: *mut *mut ITextStream,
) -> HRESULT {
    // WINE_FIXME("(%p)\n", ppts);
    E_NOTIMPL
}
unsafe extern "system" fn Host_get_StdOut(
    iface: *mut IHost,
    ppts: *mut *mut ITextStream,
) -> HRESULT {
    // WINE_FIXME("(%p)\n", ppts);
    E_NOTIMPL
}
unsafe extern "system" fn Host_get_StdErr(
    iface: *mut IHost,
    ppts: *mut *mut ITextStream,
) -> HRESULT {
    // WINE_FIXME("(%p)\n", ppts);
    E_NOTIMPL
}

// COM vtable tables — generated from host.c by vtbl_gen.py
/// host.c:441 `static const IHostVtbl HostVtbl`  (slot order: rust:types_amd64.rs)
pub static HostVtbl: IHostVtbl = IHostVtbl {
    QueryInterface: Some(Host_QueryInterface),
    AddRef: Some(Host_AddRef),
    Release: Some(Host_Release),
    GetTypeInfoCount: Some(Host_GetTypeInfoCount),
    GetTypeInfo: Some(Host_GetTypeInfo),
    GetIDsOfNames: Some(Host_GetIDsOfNames),
    Invoke: Some(Host_Invoke),
    get_Name: Some(Host_get_Name),
    get_Application: Some(Host_get_Application),
    get_FullName: Some(Host_get_FullName),
    get_Path: Some(Host_get_Path),
    get_Interactive: Some(Host_get_Interactive),
    put_Interactive: Some(Host_put_Interactive),
    Quit: Some(Host_Quit),
    get_ScriptName: Some(Host_get_ScriptName),
    get_ScriptFullName: Some(Host_get_ScriptFullName),
    get_Arguments: Some(Host_get_Arguments),
    get_Version: Some(Host_get_Version),
    get_BuildVersion: Some(Host_get_BuildVersion),
    get_Timeout: Some(Host_get_Timeout),
    put_Timeout: Some(Host_put_Timeout),
    CreateObject: Some(Host_CreateObject),
    Echo: Some(Host_Echo),
    GetObject: Some(Host_GetObject),
    DisconnectObject: Some(Host_DisconnectObject),
    Sleep: Some(Host_Sleep),
    ConnectObject: Some(Host_ConnectObject),
    get_StdIn: Some(Host_get_StdIn),
    get_StdOut: Some(Host_get_StdOut),
    get_StdErr: Some(Host_get_StdErr),
};
