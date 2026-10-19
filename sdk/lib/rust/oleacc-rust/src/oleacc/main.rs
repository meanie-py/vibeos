use crate::oleacc::client::create_client_object;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C6AD5C21A11: [u16; 15] = [
    0x0025u16, 0x0030, 0x0038, 0x0078, 0x003A, 0x0025, 0x0030, 0x0038, 0x0078, 0x003A,
    0x0025, 0x0030, 0x0038, 0x0078, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut CLSID_PSFactoryBuffer: GUID = _GUID {
    Data1: 0x3022430 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xabc4 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut lresult_atom_prefix: [WCHAR; 12] = [
    'w' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    ':' as i32 as WCHAR,
];

#[no_mangle]
pub static mut SID_AccFromDAWrapper: GUID = _GUID {
    Data1: 0x33f139ee as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe509 as ::core::ffi::c_ushort,
    Data3: 0x47f7 as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut oleacc_handle: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// COBJMACROS: feature-enable define with no value; no Rust declaration needed.

// WINE_DEFAULT_DEBUG_CHANNEL(oleacc): debug-channel declaration macro, not translated.

pub const NAVDIR_INTERNAL_HWND: i32 = 10;

// extern prototypes for OLEACC_DllGetClassObject, OLEACC_DllMain,
// OLEACC_DllRegisterServer, OLEACC_DllUnregisterServer: declared in extern blocks elsewhere, not emitted here.
#[no_mangle]
pub unsafe extern "C" fn convert_child_id(v: *mut VARIANT) -> i32 {
    match (V_VT!(v)) as u32 {
        VT_I4 => return V_I4!(v),
        _ => {
            FIXME!("unhandled child ID variant type: %d\n", V_VT!(v) as c_uint);
            return -1;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn find_class_data(hwnd: HWND, classes: *const win_class_data) -> *const win_class_data {
    let mut class_name: [WCHAR; 64] = [0; 64];
    let mut i: ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int;

    if RealGetWindowClassW(hwnd, core::ptr::addr_of_mut!(class_name) as LPWSTR, ARRAY_SIZE!(class_name) as UINT) == 0 {
        return core::ptr::null();
    }

    i = 0 as ::core::ffi::c_int;
    while !(*classes.offset(i as isize)).name.is_null() {
        if wcsicmp(core::ptr::addr_of_mut!(class_name) as *mut WCHAR, (*classes.offset(i as isize)).name as *const wchar_t) == 0 {
            if (*classes.offset(i as isize)).stub != 0 {
                FIXME!("unhandled window class: %s\n", debugstr_w(core::ptr::addr_of_mut!(class_name) as *mut WCHAR));
            }
            return classes.offset(i as isize);
        }
        i += 1;
    }

    idx = SendMessageW(hwnd, WM_GETOBJECT as UINT, 0 as WPARAM, OBJID_QUERYCLASSNAMEIDX as LPARAM) as ::core::ffi::c_int;
    if idx != 0 {
        i = 0 as ::core::ffi::c_int;
        while !(*classes.offset(i as isize)).name.is_null() {
            if idx as DWORD == (*classes.offset(i as isize)).idx {
                if (*classes.offset(i as isize)).stub != 0 {
                    FIXME!("unhandled window class: %s\n", debugstr_w(core::ptr::addr_of_mut!(class_name) as *mut WCHAR));
                }
                return classes.offset(i as isize);
            }
            i += 1;
        }

        WARN!("unhandled class name idx: %x\n", idx);
    }

    return core::ptr::null();
}

#[no_mangle]
pub unsafe extern "system" fn CreateStdAccessibleObject(
    hwnd: HWND,
    idObject: LONG,
    riidInterface: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    match idObject {
        OBJID_CLIENT => {
            return create_client_object(hwnd, riidInterface, ppvObject);
        }
        OBJID_WINDOW => {
            return create_window_object(hwnd, riidInterface, ppvObject);
        }
        _ => {
            FIXME!("unhandled object id: %ld\n", idObject);
            return E_NOTIMPL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObjectFromLresult(
    result: LRESULT,
    riid: *const IID,
    wParam: WPARAM,
    ppObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // C: WCHAR atom_str[ARRAY_SIZE(lresult_atom_prefix)+3*8+3]; the prefix is [WCHAR; 12].
    let mut atom_str: [WCHAR; 12 + 3 * 8 + 3] = [0 as WCHAR; 12 + 3 * 8 + 3];
    let mut server_proc: HANDLE = core::ptr::null_mut();
    let mut server_mapping: HANDLE = core::ptr::null_mut();
    let mut mapping: HANDLE = core::ptr::null_mut();
    let mut proc_id: DWORD = 0;
    let mut size: DWORD = 0;
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut data: HGLOBAL = core::ptr::null_mut();
    let mut view: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut p: *mut WCHAR = core::ptr::null_mut();

    if wParam != 0 {
        FIXME!("unsupported wParam = %Ix\n", wParam);
    }

    if ppObject.is_null() {
        return E_INVALIDARG;
    }
    *ppObject = core::ptr::null_mut();

    if result != result as ATOM as LRESULT {
        return E_FAIL;
    }

    if GlobalGetAtomNameW(result as ATOM, core::ptr::addr_of_mut!(atom_str) as LPWSTR, ARRAY_SIZE!(atom_str) as ::core::ffi::c_int) == 0 {
        return E_FAIL;
    }
    if memcmp(
        core::ptr::addr_of_mut!(atom_str) as *const core::ffi::c_void,
        core::ptr::addr_of!(lresult_atom_prefix) as *const core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 12]>() as usize,
    ) != 0
    {
        return E_FAIL;
    }
    p = (core::ptr::addr_of_mut!(atom_str) as *mut WCHAR).offset(ARRAY_SIZE!(lresult_atom_prefix) as isize);
    proc_id = wcstoul(p, core::ptr::addr_of_mut!(p), 16) as DWORD;
    if *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
        return E_FAIL;
    }
    server_mapping = ULongToHandle!(
        wcstoul(
            p.offset(1 as isize),
            core::ptr::addr_of_mut!(p),
            16,
        ) as ::core::ffi::c_ulong,
    ) as HANDLE;
    if *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
        return E_FAIL;
    }
    size = wcstoul(
        p.offset(1 as isize),
        core::ptr::addr_of_mut!(p),
        16,
    ) as DWORD;
    if *p as ::core::ffi::c_int != 0 {
        return E_FAIL;
    }

    server_proc = OpenProcess(PROCESS_DUP_HANDLE, (FALSE) as BOOL, proc_id);
    if server_proc.is_null() {
        return E_FAIL;
    }

    if DuplicateHandle(
        server_proc,
        server_mapping,
        GetCurrentProcess(),
        core::ptr::addr_of_mut!(mapping),
        0,
        (FALSE) as BOOL,
        DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS,
    ) == 0
    {
        return E_FAIL;
    }
    CloseHandle(server_proc);
    GlobalDeleteAtom(result as ATOM);

    view = MapViewOfFile(
        mapping,
        FILE_MAP_READ,
        0,
        0,
        0,
    ) as *mut core::ffi::c_void;
    CloseHandle(mapping);
    if view.is_null() {
        return E_FAIL;
    }

    data = GlobalAlloc(GMEM_FIXED, size as SIZE_T);
    if data.is_null() {
        UnmapViewOfFile(view);
        return E_OUTOFMEMORY;
    }
    memcpy(data as *mut core::ffi::c_void, view, size as usize);
    UnmapViewOfFile(view);

    hr = CreateStreamOnHGlobal(data, (TRUE) as BOOL, core::ptr::addr_of_mut!(stream));
    if hr < 0 {
        GlobalFree(data);
        return hr;
    }

    hr = CoUnmarshalInterface(stream as LPSTREAM, riid, ppObject as *mut LPVOID);
    IStream_Release(stream);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn LresultFromObject(
    riid: *const IID,
    wParam: WPARAM,
    pAcc: LPUNKNOWN,
) -> LRESULT {
    static seek_zero: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let mut atom_str: [WCHAR; 12 + 3 * 8 + 3] = core::mem::zeroed(); // ARRAY_SIZE(lresult_atom_prefix) is 12
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut mapping: HANDLE = core::ptr::null_mut();
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut atom: ATOM = 0;
    let mut view: *mut core::ffi::c_void = core::ptr::null_mut();

    if wParam != 0 {
        FIXME!("unsupported wParam = %Ix\n", wParam);
    }

    if pAcc.is_null() {
        return (E_INVALIDARG) as LRESULT;
    }

    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), (TRUE) as BOOL, core::ptr::addr_of_mut!(stream));
    if hr < 0 {
        return hr as LRESULT;
    }

    hr = CoMarshalInterface(stream as LPSTREAM, riid, pAcc, MSHCTX_LOCAL, core::ptr::null_mut(), MSHLFLAGS_NORMAL);
    if hr < 0 {
        IStream_Release(stream);
        return hr as LRESULT;
    }

    hr = IStream_Seek(stream, seek_zero, STREAM_SEEK_SET, core::ptr::null_mut::<ULARGE_INTEGER>());
    if hr < 0 {
        IStream_Release(stream);
        return hr as LRESULT;
    }

    hr = IStream_Stat(stream, core::ptr::addr_of_mut!(stat), STATFLAG_NONAME);
    if hr < 0 {
        CoReleaseMarshalData(stream as LPSTREAM);
        IStream_Release(stream);
        return hr as LRESULT;
    } else if stat.cbSize.u.HighPart != 0 {
        FIXME!("stream size to big\n");
        CoReleaseMarshalData(stream);
        IStream_Release(stream as LPSTREAM);
        return (E_NOTIMPL) as LRESULT;
    }

    mapping = CreateFileMappingW(
        INVALID_HANDLE_VALUE,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        PAGE_READWRITE,
        stat.cbSize.u.HighPart,
        stat.cbSize.u.LowPart,
        core::ptr::null::<WCHAR>(),
    );
    if mapping.is_null() {
        CoReleaseMarshalData(stream);
        IStream_Release(stream as LPSTREAM);
        return hr as LRESULT;
    }

    view = MapViewOfFile(
        mapping,
        FILE_MAP_WRITE,
        0,
        0,
        0,
    ) as *mut core::ffi::c_void;
    if view.is_null() {
        CloseHandle(mapping);
        CoReleaseMarshalData(stream as LPSTREAM);
        IStream_Release(stream);
        return (E_FAIL) as LRESULT;
    }

    hr = IStream_Read(stream, view, stat.cbSize.u.LowPart as ULONG, core::ptr::null_mut::<ULONG>());
    UnmapViewOfFile(view as LPCVOID);
    if hr < 0 {
        CloseHandle(mapping);
        hr = IStream_Seek(stream, seek_zero, STREAM_SEEK_SET, core::ptr::null_mut::<ULARGE_INTEGER>());
        if hr >= 0 {
            CoReleaseMarshalData(stream as LPSTREAM);
        }
        IStream_Release(stream);
        return hr as LRESULT;
    }

    memcpy(
        core::ptr::addr_of_mut!(atom_str) as *mut WCHAR as *mut core::ffi::c_void,
        core::ptr::addr_of!(lresult_atom_prefix) as *const WCHAR as *const core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 12]>() as size_t,
    );
    swprintf(
        (core::ptr::addr_of_mut!(atom_str) as *mut WCHAR).add((ARRAY_SIZE!(lresult_atom_prefix)) as _),
        (3 * 8 + 3) as usize,
        RUSTCONV_WSTR_0C6AD5C21A11.as_ptr(),
        GetCurrentProcessId(),
        HandleToUlong!(mapping),
        stat.cbSize.u.LowPart,
    );
    atom = GlobalAddAtomW(core::ptr::addr_of_mut!(atom_str) as *mut WCHAR as LPCWSTR);
    if atom == 0 {
        CloseHandle(mapping);
        hr = IStream_Seek(stream, seek_zero, STREAM_SEEK_SET, core::ptr::null_mut::<ULARGE_INTEGER>());
        if hr >= 0 {
            CoReleaseMarshalData(stream as LPSTREAM);
        }
        IStream_Release(stream);
        return (E_FAIL) as LRESULT;
    }

    IStream_Release(stream);
    return atom as LRESULT;
}

unsafe extern "C" fn variant_init_i4(v: *mut VARIANT, val: ::core::ffi::c_int) {
    (*v).n1.n2.vt = VT_I4 as ::core::ffi::c_int as VARTYPE;
    (*v).n1.n2.n3.lVal = val as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn AccessibleObjectFromPoint(
    mut point: POINT,
    mut acc: *mut *mut IAccessible,
    mut child_id: *mut VARIANT,
) -> HRESULT {
    let mut cur: *mut IAccessible = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut v: VARIANT = core::mem::zeroed();
    let mut hwnd: HWND = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        TRACE!("{%ld,%ld} %p %p\n", point.x, point.y, acc, child_id);
    }

    if acc.is_null() || child_id.is_null() {
        return E_INVALIDARG;
    }

    *acc = core::ptr::null_mut();
    (*child_id).n1.n2.vt = VT_EMPTY as VARTYPE;

    hwnd = WindowFromPoint(point);
    if hwnd.is_null() {
        return E_FAIL;
    }
    hwnd = GetAncestor(hwnd, GA_ROOT);

    hr = AccessibleObjectFromWindow(
        hwnd,
        (OBJID_WINDOW) as DWORD,
        &raw const IID_IAccessible,
        &raw mut cur as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }
    if cur.is_null() {
        return E_FAIL;
    }

    v.n1.n2.vt = VT_EMPTY as VARTYPE;
    while 1 != 0 {
        hr = IAccessible_accHitTest(cur, point.x, point.y, &raw mut v);
        if hr < 0 {
            IAccessible_Release(cur);
            return hr;
        }

        if V_VT!(&raw mut v) == VT_I4 as VARTYPE {
            *acc = cur;
            variant_init_i4(child_id, V_I4!(&raw mut v));
            return S_OK;
        } else if V_VT!(&raw mut v) == VT_DISPATCH as VARTYPE {
            IAccessible_Release(cur);

            hr = IDispatch_QueryInterface(
                V_DISPATCH!(&raw mut v),
                &raw const IID_IAccessible,
                &raw mut cur as *mut *mut core::ffi::c_void,
            );
            VariantClear(&raw mut v);
            if hr < 0 {
                return hr;
            }
            if cur.is_null() {
                return E_FAIL;
            }
        } else {
            VariantClear(&raw mut v);
            IAccessible_Release(cur);
            FIXME!("unhandled variant type: %d\n", V_VT!(&raw mut v) as i32); // C promotes VARTYPE to int
            return E_NOTIMPL;
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn AccessibleObjectFromEvent(
    hwnd: HWND,
    object_id: DWORD,
    child_id: DWORD,
    acc_out: *mut *mut IAccessible,
    child_id_out: *mut VARIANT,
) -> HRESULT {
    let mut child_id_variant: VARIANT = core::mem::zeroed();
    let mut acc: *mut IAccessible = core::ptr::null_mut();
    let mut child: *mut IDispatch = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if acc_out.is_null() {
        return E_INVALIDARG;
    }
    *acc_out = core::ptr::null_mut();
    VariantInit(child_id_out);

    hr = AccessibleObjectFromWindow(
        hwnd,
        object_id,
        core::ptr::addr_of!(IID_IAccessible),
        core::ptr::addr_of_mut!(acc) as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }

    variant_init_i4(core::ptr::addr_of_mut!(child_id_variant), child_id as core::ffi::c_int);
    hr = IAccessible_get_accChild(acc, child_id_variant, core::ptr::addr_of_mut!(child));
    if hr < 0 {
        TRACE!("get_accChild failed with %#lx!\n", hr);
    }

    if hr >= 0 && !child.is_null() {
        IAccessible_Release(acc);
        hr = IDispatch_QueryInterface(
            child,
            core::ptr::addr_of!(IID_IAccessible),
            core::ptr::addr_of_mut!(acc) as *mut *mut core::ffi::c_void,
        );
        IDispatch_Release(child);
        if hr < 0 {
            return hr;
        }

        variant_init_i4(core::ptr::addr_of_mut!(child_id_variant), (CHILDID_SELF as i32));
    }

    *acc_out = acc;
    *child_id_out = child_id_variant;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn AccessibleObjectFromWindow(
    hwnd: HWND,
    dwObjectID: DWORD,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }
    *ppvObject = core::ptr::null_mut();

    if IsWindow(hwnd) != 0 {
        let mut lres: LRESULT = 0;

        lres = SendMessageW(
            hwnd,
            WM_GETOBJECT as UINT,
            0xffffffff as WPARAM,
            dwObjectID as LPARAM,
        );
        if (lres as HRESULT) < (0 as HRESULT) {
            return lres as HRESULT;
        } else if lres != 0 {
            return ObjectFromLresult(
                lres,
                riid,
                0 as WPARAM,
                ppvObject,
            );
        }
    }

    return CreateStdAccessibleObject(hwnd, dwObjectID as LONG, riid, ppvObject);
}

#[no_mangle]
pub unsafe extern "system" fn WindowFromAccessibleObject(mut acc: *mut IAccessible, phwnd: *mut HWND) -> HRESULT {
    let mut sp: *mut IServiceProvider = core::ptr::null_mut();
    let mut acc2: *mut IAccessible = core::ptr::null_mut();
    let mut parent: *mut IDispatch = core::ptr::null_mut();
    let mut ow: *mut IOleWindow = core::ptr::null_mut();
    let mut v: VARIANT = core::mem::zeroed();
    let mut cid: VARIANT = core::mem::zeroed();
    let mut hres: HRESULT;

    hres = IAccessible_QueryInterface(acc, &raw const IID_IOleWindow, &raw mut ow as *mut *mut core::ffi::c_void);
    if hres >= 0 {
        hres = IOleWindow_GetWindow(ow, phwnd);
        IOleWindow_Release(ow);
        if hres >= 0 && !(*phwnd).is_null() {
            return S_OK;
        }
    }

    VariantInit(&raw mut v);
    variant_init_i4(&raw mut cid, (CHILDID_SELF as i32));
    hres = IAccessible_accNavigate(acc, NAVDIR_INTERNAL_HWND, cid, &raw mut v);
    if hres >= 0 {
        if hres == S_OK && (V_VT!(&raw mut v)) as u32 == VT_I4 && V_I4!(&raw mut v) != 0 {
            *phwnd = LongToHandle!(V_I4!(&raw mut v)) as HWND;
            return S_OK;
        }
        /* native leaks v here */
        VariantClear(&raw mut v);
    }

    hres = IAccessible_QueryInterface(acc, &raw const IID_IServiceProvider, &raw mut sp as *mut *mut core::ffi::c_void);
    if hres >= 0 {
        hres = IServiceProvider_QueryService(
            sp,
            &raw const SID_AccFromDAWrapper,
            &raw const IID_IAccessible,
            &raw mut acc2 as *mut *mut core::ffi::c_void,
        );
        IServiceProvider_Release(sp);
    }
    if hres < 0 {
        acc2 = acc;
        IAccessible_AddRef(acc2);
    }

    loop {
        hres = IAccessible_QueryInterface(acc2, &raw const IID_IOleWindow, &raw mut ow as *mut *mut core::ffi::c_void);
        if hres >= 0 {
            hres = IOleWindow_GetWindow(ow, phwnd);
            IOleWindow_Release(ow);
            if hres >= 0 {
                IAccessible_Release(acc2);
                return hres;
            }
        }

        hres = IAccessible_get_accParent(acc, &raw mut parent);
        IAccessible_Release(acc2);
        if hres < 0 {
            return hres;
        }
        if parent.is_null() {
            *phwnd = core::ptr::null_mut();
            return hres;
        }

        hres = IDispatch_QueryInterface(parent, &raw const IID_IAccessible, &raw mut acc2 as *mut *mut core::ffi::c_void);
        IDispatch_Release(parent);
        if hres < 0 {
            return hres;
        }
        acc = acc2;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/main.c\0"
                as *const u8 as *const i8,
            b"DllMain\0" as *const u8 as *const i8,
            482 as i32,
            b"%p, %ld, %p\n\0" as *const u8 as *const i8,
            hinstDLL,
            fdwReason,
            lpvReserved,
        );
    }

    match fdwReason {
        DLL_PROCESS_ATTACH => {
            oleacc_handle = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL);
        }
        _ => {}
    }

    return OLEACC_DllMain(hinstDLL, fdwReason, lpvReserved);
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    OLEACC_DllRegisterServer()
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    OLEACC_DllUnregisterServer()
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: REFCLSID,
    iid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of!(CLSID_CAccPropServices), rclsid) != false {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/oleacc/main.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"DllGetClassObject\0" as *const u8 as *const core::ffi::c_char,
                508 as core::ffi::c_int,
                b"(CLSID_CAccPropServices %s %p)\n\0" as *const u8 as *const core::ffi::c_char,
                debugstr_guid(iid as *const _GUID),
                ppv,
            );
        }
        return get_accpropservices_factory(iid, ppv);
    }

    if IsEqualGUID!(core::ptr::addr_of!(CLSID_PSFactoryBuffer), rclsid) != false {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/oleacc/main.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"DllGetClassObject\0" as *const u8 as *const core::ffi::c_char,
                513 as core::ffi::c_int,
                b"(CLSID_PSFactoryBuffer %s %p)\n\0" as *const u8 as *const core::ffi::c_char,
                debugstr_guid(iid as *const _GUID),
                ppv,
            );
        }
        return OLEACC_DllGetClassObject(rclsid, iid, ppv);
    }

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/oleacc/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DllGetClassObject\0" as *const u8 as *const core::ffi::c_char,
            517 as core::ffi::c_int,
            b"%s %s %p: stub\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_guid(rclsid as *const _GUID),
            debugstr_guid(iid as *const _GUID),
            ppv,
        );
    }
    return E_NOTIMPL;
}

#[no_mangle]
pub unsafe extern "system" fn GetOleaccVersionInfo(pVersion: *mut DWORD, pBuild: *mut DWORD) {
    *pVersion = (MAKELONG!(0, 7)) as u32; /* Windows 7 version of oleacc: 7.0.0.0 */
    *pBuild = (MAKELONG!(0, 0)) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessHandleFromHwnd(hwnd: HWND) -> HANDLE {
    let mut proc_id: DWORD = 0;

    if GetWindowThreadProcessId(hwnd, core::ptr::addr_of_mut!(proc_id)) == 0 {
        return core::ptr::null_mut();
    }
    return OpenProcess(
        PROCESS_DUP_HANDLE | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE | SYNCHRONIZE,
        (TRUE) as BOOL,
        proc_id,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetRoleTextW(role: DWORD, lpRole: LPWSTR, rolemax: UINT) -> UINT {
    let mut ret: INT;
    let mut resptr: *mut WCHAR = core::ptr::null_mut();

    /* return role text length */
    if lpRole.is_null() {
        return LoadStringW(
            oleacc_handle,
            role as UINT,
            core::ptr::addr_of_mut!(resptr) as LPWSTR,
            0,
        ) as UINT;
    }

    ret = LoadStringW(
        oleacc_handle,
        role as UINT,
        lpRole,
        rolemax as core::ffi::c_int,
    ) as INT;
    if !(ret > 0) {
        if rolemax > 0 {
            *lpRole.offset(0 as isize) = '\0' as WCHAR;
        }
        return 0 as UINT;
    }

    return ret as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn GetRoleTextA(role: DWORD, lpRole: LPSTR, rolemax: UINT) -> UINT {
    let mut length: UINT = 0;
    let mut roletextW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

    if !lpRole.is_null() && rolemax == 0 {
        return 0 as UINT;
    }

    length = GetRoleTextW(role, ::core::ptr::null_mut::<WCHAR>(), 0 as UINT);
    if length == 0 {
        if !lpRole.is_null() && rolemax != 0 {
            *lpRole.offset(0 as isize) = 0 as CHAR;
        }
        return 0 as UINT;
    }

    roletextW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (length.wrapping_add(1 as UINT) as SIZE_T)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if roletextW.is_null() {
        return 0 as UINT;
    }

    GetRoleTextW(role, roletextW, length.wrapping_add(1 as UINT));

    length = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        roletextW as LPCWSTR,
        -(1) as i32,
        ::core::ptr::null_mut::<CHAR>(),
        0 as i32,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as UINT;

    if lpRole.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, roletextW as LPVOID);
        return length.wrapping_sub(1 as UINT);
    }

    if rolemax < length {
        HeapFree(GetProcessHeap(), 0 as DWORD, roletextW as LPVOID);
        *lpRole.offset(0 as isize) = 0 as CHAR;
        return 0 as UINT;
    }

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        roletextW as LPCWSTR,
        -(1) as i32,
        lpRole,
        rolemax as i32,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );

    if rolemax < length {
        *lpRole.offset(rolemax.wrapping_sub(1 as UINT) as isize) = 0 as CHAR;
        length = rolemax;
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, roletextW as LPVOID);

    return length.wrapping_sub(1 as UINT);
}

#[no_mangle]
pub unsafe extern "system" fn GetStateTextW(
    mut state_bit: DWORD,
    mut state_str: *mut WCHAR,
    mut state_str_len: UINT,
) -> UINT {
    let mut state_id: DWORD;

    if state_bit & !(STATE_SYSTEM_VALID | STATE_SYSTEM_HASPOPUP) as DWORD != 0 {
        if !state_str.is_null() && state_str_len != 0 {
            *state_str.offset(0 as isize) = 0;
        }
        return 0;
    }

    state_id = IDS_STATE_NORMAL as DWORD;
    while state_bit != 0 {
        state_id = state_id.wrapping_add(1);
        state_bit /= 2 as DWORD;
    }

    if !state_str.is_null() {
        let mut ret: UINT = LoadStringW(
            oleacc_handle,
            state_id as UINT,
            state_str as LPWSTR,
            state_str_len as ::core::ffi::c_int,
        ) as UINT;
        if ret == 0 && state_str_len != 0 {
            *state_str.offset(0 as isize) = 0;
        }
        return ret;
    } else {
        let mut tmp: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
        return LoadStringW(
            oleacc_handle,
            state_id as UINT,
            core::ptr::addr_of_mut!(tmp) as LPWSTR,
            0 as ::core::ffi::c_int,
        ) as UINT;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetStateTextA(state_bit: DWORD, state_str: *mut CHAR, state_str_len: UINT) -> UINT {
    let mut state_bit = state_bit;
    let mut state_str = state_str;
    let mut state_str_len = state_str_len;
    let mut state_id: DWORD;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetStateTextA\0" as *const u8 as *const ::core::ffi::c_char,
            641 as ::core::ffi::c_int,
            b"%lx %p %u\n\0" as *const u8 as *const ::core::ffi::c_char,
            state_bit,
            state_str,
            state_str_len,
        );
    }

    if !state_str.is_null() && state_str_len == 0 {
        return (0) as UINT;
    }

    if state_bit & !(STATE_SYSTEM_VALID | STATE_SYSTEM_HASPOPUP) as DWORD != 0 {
        if !state_str.is_null() && state_str_len != 0 {
            *state_str.offset((0) as isize) = (0) as CHAR;
        }
        return (0) as UINT;
    }

    state_id = (IDS_STATE_NORMAL) as DWORD;
    while state_bit != 0 {
        state_id = state_id.wrapping_add(1);
        state_bit = state_bit.wrapping_div((2) as DWORD);
    }

    if !state_str.is_null() {
        let mut ret: UINT = LoadStringA(
            oleacc_handle,
            state_id as UINT,
            state_str as LPSTR,
            state_str_len as ::core::ffi::c_int,
        ) as UINT;
        if ret == 0 && state_str_len != 0 {
            *state_str.offset((0) as isize) = (0) as CHAR;
        }
        return ret;
    } else {
        let mut tmp: [CHAR; 256] = [0; 256];
        return LoadStringA(
            oleacc_handle,
            state_id as UINT,
            core::ptr::addr_of_mut!(tmp) as LPSTR,
            core::mem::size_of::<[CHAR; 256]>() as ::core::ffi::c_int,
        ) as UINT;
    }
}

#[no_mangle]
pub unsafe extern "system" fn AccessibleChildren(
    container: *mut IAccessible,
    start: LONG,
    count: LONG,
    children: *mut VARIANT,
    children_cnt: *mut LONG,
) -> HRESULT {
    let mut ev: *mut IEnumVARIANT = core::ptr::null_mut::<IEnumVARIANT>();
    let mut i: LONG;
    let mut child_no: LONG = 0;
    let mut hr: HRESULT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleacc/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"AccessibleChildren\0" as *const u8 as *const ::core::ffi::c_char,
            676 as ::core::ffi::c_int,
            b"%p %ld %ld %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            container,
            start,
            count,
            children,
            children_cnt,
        );
    }

    if container.is_null() || children.is_null() || children_cnt.is_null() {
        return E_INVALIDARG;
    }

    i = 0;
    while i < count {
        VariantInit(children.offset(i as isize));
        i += 1;
    }

    hr = IAccessible_QueryInterface(
        container,
        core::ptr::addr_of!(IID_IEnumVARIANT),
        core::ptr::addr_of_mut!(ev) as *mut *mut ::core::ffi::c_void,
    );
    if hr >= 0 {
        hr = IEnumVARIANT_Reset(ev);
        if hr >= 0 {
            hr = IEnumVARIANT_Skip(ev, (start) as ULONG);
        }
        if hr >= 0 {
            hr = IEnumVARIANT_Next(ev, count as ULONG, children, children_cnt as *mut ULONG);
        }
        IEnumVARIANT_Release(ev);
        return hr;
    }

    hr = IAccessible_get_accChildCount(container, core::ptr::addr_of_mut!(child_no));
    if hr < 0 {
        return hr;
    }

    i = 0;
    while i < count && start + i + 1 <= child_no {
        let mut disp: *mut IDispatch = core::ptr::null_mut::<IDispatch>();

        (*children.offset(i as isize)).n1.n2.vt = VT_I4 as VARTYPE;
        (*children.offset(i as isize)).n1.n2.n3.lVal = start + i + 1;

        hr = IAccessible_get_accChild(
            container,
            *children.offset(i as isize),
            core::ptr::addr_of_mut!(disp),
        );
        if hr >= 0 && !disp.is_null() {
            (*children.offset(i as isize)).n1.n2.vt = VT_DISPATCH as VARTYPE;
            (*children.offset(i as isize)).n1.n2.n3.pdispVal = disp;
        }
        i += 1;
    }

    *children_cnt = i;
    return if i == count { S_OK } else { S_FALSE };
}
