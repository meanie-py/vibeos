use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub alias: *const WCHAR,
    pub class: *const WCHAR,
}

#[no_mangle]
pub static mut CLSID_WbemLocator: GUID = _GUID {
    Data1: 0x4590f811 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1d3a as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IWbemLocator: GUID = _GUID {
    Data1: 0xdc12a687 as ::core::ffi::c_uint,
    Data2: 0x737f as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut biosW: [WCHAR; 5] = [
    'b' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut computersystemW: [WCHAR; 15] = [
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut cpuW: [WCHAR; 4] = [
    'c' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut logicaldiskW: [WCHAR; 12] = [
    'L' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut nicW: [WCHAR; 4] = [
    'n' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut osW: [WCHAR; 3] = [
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut processW: [WCHAR; 8] = [
    'p' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_biosW: [WCHAR; 11] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'B' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_computersystemW: [WCHAR; 21] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_logicaldiskW: [WCHAR; 18] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_networkadapterW: [WCHAR; 21] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_operatingsystemW: [WCHAR; 22] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_processW: [WCHAR; 14] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut win32_processorW: [WCHAR; 16] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut alias_map: [C2RustUnnamed_18; 7] = unsafe {
    [
        C2RustUnnamed_18 {
            alias: &raw const biosW as *const WCHAR,
            class: &raw const win32_biosW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const computersystemW as *const WCHAR,
            class: &raw const win32_computersystemW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const cpuW as *const WCHAR,
            class: &raw const win32_processorW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const logicaldiskW as *const WCHAR,
            class: &raw const win32_logicaldiskW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const nicW as *const WCHAR,
            class: &raw const win32_networkadapterW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const osW as *const WCHAR,
            class: &raw const win32_operatingsystemW as *const WCHAR,
        },
        C2RustUnnamed_18 {
            alias: &raw const processW as *const WCHAR,
            class: &raw const win32_processW as *const WCHAR,
        },
    ]
};

// #define COBJMACROS: preprocessor flag, no Rust declaration needed.
// WINE_DEFAULT_DEBUG_CHANNEL(wmic): function-like macro that defines the debug channel; omitted.

#[repr(C)]
pub struct AliasMapEntry {
    pub alias: *const WCHAR,
    pub class: *const WCHAR,
}

unsafe impl Sync for AliasMapEntry {}

unsafe extern "C" fn find_class(alias: *const WCHAR) -> *const WCHAR {
    let mut i: ::core::ffi::c_uint;
    i = 0;
    while i < ARRAY_SIZE(alias_map) as ::core::ffi::c_uint {
        if strcmpiW(alias, alias_map[i as usize].alias) == 0 {
            return alias_map[i as usize].class;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<WCHAR>();
}

unsafe extern "C" fn strdupW(src: *const WCHAR) -> *mut WCHAR {
    let mut dst: *mut WCHAR = core::ptr::null_mut();
    if src.is_null() {
        return core::ptr::null_mut();
    }
    dst = HeapAlloc(
        GetProcessHeap(),
        0,
        (strlenW(src) + 1) * core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    strcpyW(dst, src);
    return dst;
}

unsafe extern "C" fn find_prop(class: *mut IWbemClassObject, prop: *const WCHAR) -> *mut WCHAR {
    let mut sa: *mut SAFEARRAY = core::ptr::null_mut();
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut i: LONG;
    let mut last_index: LONG = 0;
    let mut str: BSTR;

    if IWbemClassObject_GetNames(
        class,
        core::ptr::null(),
        WBEM_FLAG_ALWAYS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(sa),
    ) != S_OK
    {
        return core::ptr::null_mut();
    }

    SafeArrayGetUBound(sa, 1, core::ptr::addr_of_mut!(last_index));
    i = 0;
    while i <= last_index {
        SafeArrayGetElement(
            sa,
            core::ptr::addr_of_mut!(i),
            core::ptr::addr_of_mut!(str) as *mut ::core::ffi::c_void,
        );
        if strcmpiW(str, prop) == 0 {
            ret = strdupW(str);
            break;
        }
        i += 1;
    }
    SafeArrayDestroy(sa);
    ret
}

unsafe extern "C" fn output_string(mut handle: HANDLE, mut msg: *const WCHAR, mut args: ...) -> ::core::ffi::c_int {
    let mut va_args: va_list;
    let mut len: ::core::ffi::c_int = 0;
    let mut count: DWORD = 0;
    let mut buffer: [WCHAR; 8192] = [0; 8192];

    va_start(va_args, msg);
    len = vsnprintfW(buffer.as_mut_ptr(), ARRAY_SIZE(buffer), msg, va_args);
    va_end(va_args);

    if WriteConsoleW(
        handle,
        buffer.as_mut_ptr() as *const ::core::ffi::c_void,
        len as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) == 0
    {
        WriteFile(
            handle,
            buffer.as_mut_ptr() as LPCVOID,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            core::ptr::addr_of_mut!(count),
            FALSE,
        );
    }

    return count as ::core::ffi::c_int;
}

unsafe extern "C" fn output_error(mut msg: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static mut fmtW: [WCHAR; 3] = [0x0025u16, 0x0073, 0x0000];
    let mut buffer: [WCHAR; 8192] = [0; 8192];

    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut::<WCHAR>()) as HINSTANCE,
        msg as UINT,
        core::ptr::addr_of_mut!(buffer) as LPWSTR,
        ARRAY_SIZE(buffer) as ::core::ffi::c_int,
    );
    return output_string(
        GetStdHandle(STD_ERROR_HANDLE),
        core::ptr::addr_of!(fmtW) as *const WCHAR,
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
    );
}

unsafe extern "C" fn output_header(prop: *const WCHAR, column_width: ULONG) -> ::core::ffi::c_int {
    static mut bomW: [WCHAR; 1] = [0xFEFFu16];
    static mut fmtW: [WCHAR; 7] = [
        '%' as WCHAR,
        '-' as WCHAR,
        '*' as WCHAR,
        's' as WCHAR,
        '\r' as WCHAR,
        '\n' as WCHAR,
        0,
    ];
    let mut len: ::core::ffi::c_int;
    let mut count: DWORD;
    let mut buffer: [WCHAR; 8192] = [0; 8192];

    count = 0;
    len = snprintfW(
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        ARRAY_SIZE(buffer),
        core::ptr::addr_of!(fmtW) as *const WCHAR,
        column_width,
        prop,
    );

    if WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        core::ptr::addr_of_mut!(buffer) as *const ::core::ffi::c_void,
        len as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) == 0
    {
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            core::ptr::addr_of!(bomW) as *const WCHAR as LPCVOID,
            core::mem::size_of::<[WCHAR; 1]>() as DWORD,
            core::ptr::addr_of_mut!(count),
            FALSE,
        );
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            core::ptr::addr_of_mut!(buffer) as *const WCHAR as LPCVOID,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            core::ptr::addr_of_mut!(count),
            FALSE,
        );
        count = count.wrapping_add(core::mem::size_of::<[WCHAR; 1]>() as DWORD);
    }

    return count as ::core::ffi::c_int;
}

unsafe extern "C" fn output_line(str: *const WCHAR, column_width: ULONG) -> i32 {
    static mut fmtW: [WCHAR; 7] = [0x0025u16, 0x002D, 0x002A, 0x0073, 0x000D, 0x000A, 0x0000];
    return output_string(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of!(fmtW) as *const WCHAR, column_width, str);
}

unsafe extern "C" fn query_prop(class: *const WCHAR, propname: *const WCHAR) -> ::core::ffi::c_int {
    static select_allW: [u16; 15] = [
    0x0053u16, 0x0045, 0x004C, 0x0045, 0x0043, 0x0054, 0x0020, 0x002A, 0x0020, 0x0046,
    0x0052, 0x004F, 0x004D, 0x0020, 0x0000,
];
    static cimv2W: [u16; 11] = [
    0x0052u16, 0x004F, 0x004F, 0x0054, 0x005C, 0x0043, 0x0049, 0x004D, 0x0056, 0x0032,
    0x0000,
];
    static wqlW: [u16; 4] = [0x0057u16, 0x0051, 0x004C, 0x0000];
    let mut hr: HRESULT = 0;
    let mut locator: *mut IWbemLocator = core::ptr::null_mut::<IWbemLocator>();
    let mut services: *mut IWbemServices = core::ptr::null_mut::<IWbemServices>();
    let mut result: *mut IEnumWbemClassObject = core::ptr::null_mut::<IEnumWbemClassObject>();
    let mut flags: LONG = (WBEM_FLAG_RETURN_IMMEDIATELY | WBEM_FLAG_FORWARD_ONLY) as LONG;
    let mut path: BSTR = core::ptr::null_mut::<OLECHAR>();
    let mut wql: BSTR = core::ptr::null_mut::<OLECHAR>();
    let mut query: BSTR = core::ptr::null_mut::<OLECHAR>();
    let mut prop: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut first: BOOL = TRUE as BOOL;
    let mut len: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut obj: *mut IWbemClassObject = core::ptr::null_mut::<IWbemClassObject>();
    let mut count: ULONG = 0;
    let mut width: ULONG = 0 as ULONG;
    let mut v: VARIANT = core::mem::zeroed();

    WINE_TRACE(
        b"%s, %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        debugstr_w(class),
        debugstr_w(propname),
    );

    CoInitialize(core::ptr::null_mut());
    CoInitializeSecurity(
        core::ptr::null_mut(),
        -(1 as LONG),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        RPC_C_AUTHN_LEVEL_DEFAULT as DWORD,
        RPC_C_IMP_LEVEL_IMPERSONATE as DWORD,
        core::ptr::null_mut(),
        EOAC_NONE as DWORD,
        core::ptr::null_mut(),
    );

    'done: {
        hr = CoCreateInstance(
            &raw const CLSID_WbemLocator,
            core::ptr::null_mut::<IUnknown>(),
            (CLSCTX_INPROC_SERVER) as DWORD,
            &raw const IID_IWbemLocator,
            &raw mut locator as *mut LPVOID,
        );
        if hr != S_OK {
            break 'done;
        }

        path = SysAllocString(&raw const cimv2W as *const OLECHAR);
        if path.is_null() {
            break 'done;
        }
        hr = IWbemLocator_ConnectServer(
            locator,
            path,
            core::ptr::null_mut::<OLECHAR>(),
            core::ptr::null_mut::<OLECHAR>(),
            core::ptr::null_mut::<OLECHAR>(),
            0 as LONG,
            core::ptr::null_mut::<OLECHAR>(),
            core::ptr::null_mut::<IWbemContext>(),
            &raw mut services,
        );
        if hr != S_OK {
            break 'done;
        }

        len = (strlenW(class) as usize).wrapping_add(ARRAY_SIZE(select_allW))
            as ::core::ffi::c_int;
        query = SysAllocStringLen(core::ptr::null(), len as UINT);
        if query.is_null() {
            break 'done;
        }
        strcpyW(query, &raw const select_allW as *const WCHAR);
        strcatW(query, class);

        wql = SysAllocString(&raw const wqlW as *const OLECHAR);
        if wql.is_null() {
            break 'done;
        }
        hr = IWbemServices_ExecQuery(
            services,
            wql,
            query,
            flags,
            core::ptr::null_mut::<IWbemContext>(),
            &raw mut result,
        );
        if hr != S_OK {
            break 'done;
        }

        loop {
            // get column width
            IEnumWbemClassObject_Next(
                result,
                WBEM_INFINITE as LONG,
                1 as ULONG,
                &raw mut obj,
                &raw mut count,
            );
            if count == 0 {
                break;
            }

            if prop.is_null() {
                prop = find_prop(obj, propname);
                if prop.is_null() {
                    output_error(STRING_INVALID_QUERY);
                    break 'done;
                }
            }
            if IWbemClassObject_Get(
                obj,
                prop as LPCWSTR,
                0 as LONG,
                &raw mut v,
                core::ptr::null_mut::<CIMTYPE>(),
                core::ptr::null_mut::<LONG>(),
            ) == WBEM_S_NO_ERROR
            {
                VariantChangeType(
                    &raw mut v,
                    &raw mut v,
                    0 as USHORT,
                    VT_BSTR as VARTYPE,
                );
                width = max(strlenW(V_BSTR(&raw mut v)) as ULONG, width);
                VariantClear(&raw mut v);
            }
            IWbemClassObject_Release(obj);
        }
        width = width.wrapping_add(2 as ULONG);

        IEnumWbemClassObject_Reset(result);
        loop {
            IEnumWbemClassObject_Next(
                result,
                WBEM_INFINITE as LONG,
                1 as ULONG,
                &raw mut obj,
                &raw mut count,
            );
            if count == 0 {
                break;
            }

            if first != 0 {
                output_header(prop, width);
                first = FALSE as BOOL;
            }
            if IWbemClassObject_Get(
                obj,
                prop as LPCWSTR,
                0 as LONG,
                &raw mut v,
                core::ptr::null_mut::<CIMTYPE>(),
                core::ptr::null_mut::<LONG>(),
            ) == WBEM_S_NO_ERROR
            {
                VariantChangeType(
                    &raw mut v,
                    &raw mut v,
                    0 as USHORT,
                    VT_BSTR as VARTYPE,
                );
                output_line(V_BSTR(&raw mut v), width);
                VariantClear(&raw mut v);
            }
            IWbemClassObject_Release(obj);
        }
        ret = 0 as ::core::ffi::c_int;
    }

    // done:
    if !result.is_null() {
        IEnumWbemClassObject_Release(result);
    }
    if !services.is_null() {
        IWbemServices_Release(services);
    }
    if !locator.is_null() {
        IWbemLocator_Release(locator);
    }
    SysFreeString(path);
    SysFreeString(query);
    SysFreeString(wql);
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        prop as PVOID,
    );
    CoUninitialize();
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: c_int, argv: *mut *mut WCHAR) -> c_int {
    static getW: [u16; 4] = [0x0067u16, 0x0065, 0x0074, 0x0000];
    static quitW: [u16; 5] = [0x0071u16, 0x0075, 0x0069, 0x0074, 0x0000];
    static exitW: [u16; 5] = [0x0065u16, 0x0078, 0x0069, 0x0074, 0x0000];
    static pathW: [u16; 5] = [0x0070u16, 0x0061, 0x0074, 0x0068, 0x0000];
    static classW: [u16; 6] = [0x0063u16, 0x006C, 0x0061, 0x0073, 0x0073, 0x0000];
    static contextW: [u16; 8] = [0x0063u16, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x0000];
    let mut class: *const WCHAR = core::ptr::null();
    let mut value: *const WCHAR = core::ptr::null();
    let mut i: c_int;
    i = 1;
    while i < argc && *(*argv.offset(i as isize)).offset(0 as isize) == '/' as WCHAR {
        WINE_FIXME("command line switch %s not supported\n", debugstr_w(*argv.offset(i as isize)));
        i += 1;
    }

    'not_supported: {
        if i >= argc {
            break 'not_supported;
        }

        if strcmpiW(*argv.offset(i as isize), quitW.as_ptr()) == 0
            || strcmpiW(*argv.offset(i as isize), exitW.as_ptr()) == 0
        {
            return 0;
        }

        if strcmpiW(*argv.offset(i as isize), classW.as_ptr()) == 0
            || strcmpiW(*argv.offset(i as isize), contextW.as_ptr()) == 0
        {
            WINE_FIXME("command %s not supported\n", debugstr_w(*argv.offset(i as isize)));
            break 'not_supported;
        }

        if strcmpiW(*argv.offset(i as isize), pathW.as_ptr()) == 0 {
            i += 1;
            if i >= argc {
                output_error(STRING_INVALID_PATH);
                return 1;
            }
            class = *argv.offset(i as isize);
        } else {
            class = find_class(*argv.offset(i as isize));
            if class.is_null() {
                output_error(STRING_ALIAS_NOT_FOUND);
                return 1;
            }
        }

        i += 1;
        if i >= argc {
            break 'not_supported;
        }

        if strcmpiW(*argv.offset(i as isize), getW.as_ptr()) == 0 {
            i += 1;
            if i >= argc {
                break 'not_supported;
            }
            value = *argv.offset(i as isize);
            return query_prop(class, value);
        }
    }

    output_error(STRING_CMDLINE_NOT_SUPPORTED);
    return 1;
}
