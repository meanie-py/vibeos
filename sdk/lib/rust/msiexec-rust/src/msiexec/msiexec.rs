use crate::msiexec::service::DoService;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5D89CFDA588E: [u16; 17] = [
    0x0052u16, 0x0045, 0x0042, 0x004F, 0x004F, 0x0054, 0x0050, 0x0052, 0x004F, 0x004D,
    0x0050, 0x0054, 0x003D, 0x0022, 0x0053, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_6E220D871E8B: [u16; 11] = [
    0x0052u16, 0x0045, 0x004D, 0x004F, 0x0056, 0x0045, 0x003D, 0x0041, 0x004C, 0x004C,
    0x0000,
];
pub static RUSTCONV_WSTR_808A599CC396: [u16; 12] = [
    0x005Cu16, 0x006D, 0x0073, 0x0069, 0x0065, 0x0078, 0x0065, 0x0063, 0x0020, 0x002F,
    0x0056, 0x0000,
];
pub static RUSTCONV_WSTR_98F59DB8D472: [u16; 21] = [
    0x005Cu16, 0x005C, 0x002E, 0x005C, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x006D,
    0x0073, 0x0069, 0x0063, 0x0061, 0x005F, 0x0025, 0x0078, 0x005F, 0x0025, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_9C9A54CDDB92: [u16; 67] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0065,
    0x0072, 0x005C, 0x0052, 0x0075, 0x006E, 0x004F, 0x006E, 0x0063, 0x0065, 0x0045,
    0x006E, 0x0074, 0x0072, 0x0069, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_A5468EA1B040: [u16; 10] = [0x004Du16, 0x0053, 0x0049, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_C06870C8E03C: [u16; 13] = [
    0x0041u16, 0x0043, 0x0054, 0x0049, 0x004F, 0x004E, 0x003D, 0x0041, 0x0044, 0x004D,
    0x0049, 0x004E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SERVICES_ACTIVE_DATABASEW: [::core::ffi::c_ushort; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_ushort; 15]>(
        *b"S\0e\0r\0v\0i\0c\0e\0s\0A\0c\0t\0i\0v\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut GUID_NULL: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: unsafe { ::core::mem::zeroed() },
};

static mut silent: BOOL = 0;

static mut client_pid: DWORD = 0;

// #define WIN32_LEAN_AND_MEAN: no value, nothing to emit.

// WINE_DEFAULT_DEBUG_CHANNEL(msiexec): debug channel descriptor for this file.
pub static mut __wine_dbch_msiexec: __wine_debug_channel = __wine_debug_channel {
    flags: 0xff,
    name: unsafe { core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"msiexec\0\0\0\0\0\0\0\0") },
};

pub type DLLREGISTERSERVER = unsafe extern "system" fn() -> HRESULT;
pub type DLLUNREGISTERSERVER = unsafe extern "system" fn() -> HRESULT;

// DWORD DoService(void); is a prototype for a function defined in this file; emitted with its definition.

#[repr(C)]
pub struct string_list {
    pub next: *mut string_list,
    pub str_0: [WCHAR; 1],
}

// extern UINT CDECL __wine_msi_call_dll_function(DWORD client_pid, const GUID *guid);
// is an external prototype; resolved by the import/extern pass, not emitted here.

pub const CS_WHITESPACE: i32 = 0;
pub const CS_TOKEN: i32 = 1;
pub const CS_QUOTE: i32 = 2;
#[no_mangle]
pub unsafe extern "C" fn report_error_v(msg: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    let mut buffer: [::core::ffi::c_char; 2048] = [0; 2048];
    let mut va_args: va_list = *(ap_list as *mut va_list);

    vsnprintf(
        core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
        core::mem::size_of::<[::core::ffi::c_char; 2048]>() as usize,
        msg,
        va_args,
    );
    if silent != 0 {
        wine_dbg_printf(
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
        );
    } else {
        MsiMessageBoxA(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char as LPCSTR,
            b"MsiExec\0".as_ptr() as LPCSTR,
            0 as UINT,
            GetUserDefaultLangID() as WORD,
            0 as DWORD,
        );
    }
}



unsafe extern "C" fn ShowUsage(ExitCode: ::core::ffi::c_int) {
    let mut msiexec_version: [WCHAR; 40] = [0; 40];
    let mut filename: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut msi_res: LPWSTR = core::ptr::null_mut();
    let mut msiexec_help: LPWSTR = core::ptr::null_mut();
    let mut hmsi: HMODULE = GetModuleHandleA(b"msi.dll\0".as_ptr() as LPCSTR);
    let mut len: DWORD = 0;
    let mut res: DWORD = 0;

    /* MsiGetFileVersion need the full path */
    *(core::ptr::addr_of_mut!(filename) as *mut WCHAR) = 0;
    res = GetModuleFileNameW(
        hmsi as HINSTANCE,
        core::ptr::addr_of_mut!(filename) as LPWSTR,
        ARRAY_SIZE!(filename) as DWORD,
    ) as DWORD;
    if res == 0 {
        WINE_ERR!(
            b"GetModuleFileName failed: %ld\n\0".as_ptr() as LPCSTR,
            GetLastError(),
        );
    }

    len = ARRAY_SIZE!(msiexec_version) as DWORD;
    *(core::ptr::addr_of_mut!(msiexec_version) as *mut WCHAR) = 0;
    res = MsiGetFileVersionW(
        core::ptr::addr_of_mut!(filename) as LPCWSTR,
        core::ptr::addr_of_mut!(msiexec_version) as LPWSTR,
        core::ptr::addr_of_mut!(len),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as DWORD;
    if res != 0 {
        WINE_ERR!(
            b"MsiGetFileVersion failed with %ld\n\0".as_ptr() as LPCSTR,
            res,
        );
    }

    /* Return the length of the resource.
       No typo: The LPWSTR parameter must be a LPWSTR * for this mode */
    len = LoadStringW(
        hmsi as HINSTANCE,
        10,
        core::ptr::addr_of_mut!(msi_res) as LPWSTR,
        0,
    ) as DWORD;

    msi_res = malloc(
        ((len + 1) as usize) * core::mem::size_of::<WCHAR>(),
    ) as LPWSTR;
    msiexec_help = malloc(
        ((len + 1) as usize) * core::mem::size_of::<WCHAR>()
            + core::mem::size_of::<[WCHAR; 40]>(),
    ) as LPWSTR;
    if !msi_res.is_null() && !msiexec_help.is_null() {
        *msi_res = 0;
        LoadStringW(
            hmsi as HINSTANCE,
            10,
            msi_res,
            (len + 1) as ::core::ffi::c_int,
        );

        swprintf(
            msiexec_help as *mut WCHAR,
            (len + 1 + ARRAY_SIZE!(msiexec_version) as DWORD) as usize,
            msi_res as *const WCHAR,
            core::ptr::addr_of_mut!(msiexec_version) as *mut WCHAR,
        );
        MsiMessageBoxW(
            core::ptr::null_mut(),
            msiexec_help,
            core::ptr::null(),
            0,
            GetUserDefaultLangID() as WORD,
            0,
        );
    }
    free(msi_res as *mut ::core::ffi::c_void);
    free(msiexec_help as *mut ::core::ffi::c_void);
    ExitProcess(ExitCode as UINT);
}

unsafe extern "C" fn IsProductCode(mut str: LPWSTR) -> BOOL {
    let mut ProductCode: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    if lstrlenW(str as LPCWSTR) != 38 {
        return (FALSE) as BOOL;
    }
    return (CLSIDFromString(str as LPCOLESTR, core::ptr::addr_of_mut!(ProductCode)) == NOERROR) as BOOL;
}

unsafe extern "C" fn StringListAppend(mut list: *mut *mut string_list, mut str: LPCWSTR) {
    let mut entry: *mut string_list = core::ptr::null_mut::<string_list>();
    entry = malloc((FIELD_OFFSET!(string_list, str_0) + ((wcslen(str) + 1 as usize) * core::mem::size_of::<u16>()) as i32) as usize) as *mut string_list;
    if entry.is_null() {
        WINE_ERR!(b"Out of memory!\n\0".as_ptr() as *const i8);
        ExitProcess(1 as UINT);
    }
    lstrcpyW(core::ptr::addr_of_mut!((*entry).str_0) as LPWSTR, str);
    (*entry).next = core::ptr::null_mut::<string_list>();
    while !(*list).is_null() {
        list = core::ptr::addr_of_mut!((**list).next);
    }
    *list = entry;
}

unsafe extern "C" fn build_properties(property_list: *mut string_list) -> LPWSTR {
    let mut list: *mut string_list;
    let mut ret: LPWSTR;
    let mut p: LPWSTR;
    let mut value: LPWSTR;
    let mut len: DWORD;
    let mut needs_quote: BOOL;

    if property_list.is_null() {
        return core::ptr::null_mut();
    }

    /* count the space we need */
    len = 1 as DWORD;
    list = property_list;
    while !list.is_null() {
        len = len.wrapping_add(
            (lstrlenW(core::ptr::addr_of_mut!((*list).str_0) as LPCWSTR) + 3) as DWORD,
        );
        list = (*list).next;
    }

    ret = malloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;

    /* add a space before each string, and quote the value */
    p = ret;
    list = property_list;
    while !list.is_null() {
        value = wcschr(core::ptr::addr_of_mut!((*list).str_0) as *mut WCHAR, '=' as WCHAR) as LPWSTR;
        if value.is_null() {
            list = (*list).next;
            continue;
        }
        len = value.offset_from(core::ptr::addr_of_mut!((*list).str_0) as *mut WCHAR) as DWORD;
        *p = ' ' as WCHAR;
        p = p.add(1);
        memcpy(
            p as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*list).str_0) as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.add(len as usize);
        *p = '=' as WCHAR;
        p = p.add(1);

        /* check if the value contains spaces and maybe quote it */
        value = value.add(1);
        needs_quote = (*value != '"' as WCHAR
            && !wcschr(value as *const WCHAR, ' ' as WCHAR).is_null()) as BOOL;
        if needs_quote != 0 {
            *p = '"' as WCHAR;
            p = p.add(1);
        }
        len = lstrlenW(value as LPCWSTR) as DWORD;
        memcpy(
            p as *mut core::ffi::c_void,
            value as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.add(len as usize);
        if needs_quote != 0 {
            *p = '"' as WCHAR;
            p = p.add(1);
        }
        list = (*list).next;
    }
    *p = 0 as WCHAR;

    WINE_TRACE!("properties -> %s\n", wine_dbgstr_w(ret));

    return ret;
}

unsafe extern "C" fn build_transforms(transform_list: *mut string_list) -> LPWSTR {
    let mut list: *mut string_list = core::ptr::null_mut();
    let mut ret: LPWSTR = core::ptr::null_mut();
    let mut p: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 1;

    /* count the space we need */
    list = transform_list;
    while !list.is_null() {
        len = len.wrapping_add(
            (lstrlenW(&raw mut (*list).str_0 as *mut WCHAR as LPCWSTR) + 1) as DWORD,
        );
        list = (*list).next;
    }

    ret = malloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;

    /* add all the transforms with a semicolon between each one */
    p = ret;
    list = transform_list;
    while !list.is_null() {
        len = lstrlenW(&raw mut (*list).str_0 as *mut WCHAR as LPCWSTR) as DWORD;
        lstrcpynW(p, &raw mut (*list).str_0 as *mut WCHAR as LPCWSTR, len as core::ffi::c_int);
        p = p.offset(len as isize);
        if !(*list).next.is_null() {
            *p = ';' as WCHAR;
            p = p.offset(1);
        }
        list = (*list).next;
    }
    *p = 0 as WCHAR;

    return ret;
}

unsafe extern "C" fn msi_atou(mut str: LPCWSTR) -> DWORD {
    let mut ret: DWORD = 0;
    while *str >= '0' as u16 && *str <= '9' as u16 {
        ret = ret.wrapping_mul(10);
        ret = ret.wrapping_add((*str as DWORD).wrapping_sub('0' as DWORD));
        str = str.offset(1);
    }
    return ret;
}

unsafe extern "C" fn msi_strequal(str1: LPCWSTR, str2: LPCSTR) -> BOOL {
    let mut len: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut strW: LPWSTR = core::ptr::null_mut::<WCHAR>();
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str2,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as core::ffi::c_int,
    ) as DWORD;
    if len == 0 {
        return FALSE as BOOL;
    }
    if lstrlenW(str1) as DWORD != len.wrapping_sub(1 as DWORD) {
        return FALSE as BOOL;
    }
    strW = malloc(
        core::mem::size_of::<WCHAR>().wrapping_mul(len as usize),
    ) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str2,
        -(1 as core::ffi::c_int),
        strW,
        len as core::ffi::c_int,
    );
    ret = CompareStringW(
        GetThreadLocale(),
        NORM_IGNORECASE as DWORD,
        str1,
        len as core::ffi::c_int,
        strW as LPCWSTR,
        len as core::ffi::c_int,
    ) as DWORD;
    free(strW as *mut core::ffi::c_void);
    return (ret == CSTR_EQUAL as DWORD) as BOOL;
}

unsafe extern "C" fn msi_option_equal(str1: LPCWSTR, str2: LPCSTR) -> BOOL {
    if *str1.offset(0 as isize) != '/' as u16
        && *str1.offset(0 as isize) != '-' as u16
    {
        return (FALSE) as BOOL;
    }

    /* skip over the hyphen or slash */
    return msi_strequal(str1.offset(1 as isize), str2);
}

unsafe extern "C" fn msi_strprefix(str1: LPCWSTR, str2: LPCSTR) -> BOOL {
    let mut len: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut strW: LPWSTR = core::ptr::null_mut();
    len = MultiByteToWideChar(CP_ACP, 0, str2, -1, core::ptr::null_mut(), 0) as DWORD;
    if len == 0 {
        return (FALSE) as BOOL;
    }
    if (lstrlenW(str1) as DWORD) < (len - 1) {
        return (FALSE) as BOOL;
    }
    strW = malloc((core::mem::size_of::<WCHAR>() as usize).wrapping_mul(len as usize)) as LPWSTR;
    MultiByteToWideChar(CP_ACP, 0, str2, -1, strW, len as core::ffi::c_int);
    ret = CompareStringW(GetThreadLocale(), NORM_IGNORECASE, str1, (len - 1) as core::ffi::c_int, strW as LPCWSTR, (len - 1) as core::ffi::c_int) as DWORD;
    free(strW as *mut core::ffi::c_void);
    return (ret == CSTR_EQUAL) as BOOL;
}

unsafe extern "C" fn msi_option_prefix(str1: LPCWSTR, str2: LPCSTR) -> BOOL {
    if *str1.offset(0 as isize) != '/' as u16 && *str1.offset(0 as isize) != '-' as u16 {
        return (FALSE) as BOOL;
    }

    // skip over the hyphen or slash
    return msi_strprefix(str1.offset(1 as isize), str2);
}

unsafe extern "C" fn LoadProc(
    DllName: LPCWSTR,
    ProcName: LPCSTR,
    DllHandle: *mut HMODULE,
) -> *mut c_void {
    let mut proc_: Option<unsafe extern "C" fn() -> *mut c_void>;

    *DllHandle = LoadLibraryExW(DllName, core::ptr::null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH as DWORD) as HMODULE;
    if (*DllHandle).is_null() {
        report_error(
            b"Unable to load dll %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_w(DllName as *const WCHAR),
        );
        ExitProcess(1 as UINT);
    }
    proc_ = core::mem::transmute::<FARPROC, Option<unsafe extern "C" fn() -> *mut c_void>>(
        GetProcAddress(*DllHandle, ProcName),
    );
    if proc_.is_none() {
        report_error(
            b"Dll %s does not implement function %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_w(DllName as *const WCHAR),
            ProcName,
        );
        FreeLibrary(*DllHandle);
        ExitProcess(1 as UINT);
    }

    return core::mem::transmute::<
        Option<unsafe extern "C" fn() -> *mut c_void>,
        *mut c_void,
    >(proc_);
}

unsafe extern "C" fn DoDllRegisterServer(mut DllName: LPCWSTR) -> DWORD {
    let mut hr: HRESULT = 0;
    // LoadProc exits the process rather than return NULL, so this is never None.
    let mut pfDllRegisterServer: DLLREGISTERSERVER;
    let mut DllHandle: HMODULE = core::ptr::null_mut();

    pfDllRegisterServer = core::mem::transmute::<*mut core::ffi::c_void, DLLREGISTERSERVER>(
        LoadProc(
            DllName,
            b"DllRegisterServer\0".as_ptr() as LPCSTR,
            core::ptr::addr_of_mut!(DllHandle),
        ),
    );

    hr = pfDllRegisterServer();
    if hr < 0 {
        report_error(
            b"Failed to register dll %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_w(DllName as *const WCHAR),
        );
        return 1;
    }

    wine_dbg_printf(
        b"Successfully registered dll %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        wine_dbgstr_w(DllName as *const WCHAR),
    );
    if !DllHandle.is_null() {
        FreeLibrary(DllHandle);
    }
    return 0;
}

unsafe extern "C" fn DoDllUnregisterServer(DllName: LPCWSTR) -> DWORD {
    let mut hr: HRESULT;
    let mut pfDllUnregisterServer: DLLUNREGISTERSERVER;
    let mut DllHandle: HMODULE = core::ptr::null_mut();

    pfDllUnregisterServer = core::mem::transmute::<*mut core::ffi::c_void, DLLUNREGISTERSERVER>(
        LoadProc(
            DllName,
            b"DllUnregisterServer\0" as *const u8 as LPCSTR,
            core::ptr::addr_of_mut!(DllHandle),
        ),
    );

    hr = pfDllUnregisterServer();
    if hr < 0 {
        report_error(
            b"Failed to unregister dll %s\n\0" as *const u8 as *const core::ffi::c_char,
            wine_dbgstr_w(DllName as *const WCHAR),
        );
        return 1;
    }

    wine_dbg_printf(
        b"Successfully unregistered dll %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        wine_dbgstr_w(DllName as *const WCHAR),
    );
    if !DllHandle.is_null() {
        FreeLibrary(DllHandle);
    }
    return 0;
}

unsafe extern "C" fn DoRegServer() -> DWORD {
    let mut scm: SC_HANDLE = core::ptr::null_mut();
    let mut service: SC_HANDLE = core::ptr::null_mut();
    let mut path: [WCHAR; (MAX_PATH+12) as _] = [0; (MAX_PATH+12) as _];
    let mut len: DWORD;
    let mut ret: DWORD = 0;

    scm = OpenSCManagerW(core::ptr::null_mut(), (SERVICES_ACTIVE_DATABASEW).as_ptr(), SC_MANAGER_CREATE_SERVICE);
    if scm.is_null() {
        report_error(b"Failed to open the service control manager.\n\0".as_ptr() as *const ::core::ffi::c_char);
        return 1;
    }
    len = GetSystemDirectoryW(path.as_mut_ptr(), MAX_PATH) as DWORD;
    lstrcpyW(path.as_mut_ptr().add(len as usize), RUSTCONV_WSTR_808A599CC396.as_ptr());
    service = CreateServiceW(scm, RUSTCONV_WSTR_A5468EA1B040.as_ptr(), RUSTCONV_WSTR_A5468EA1B040.as_ptr(), GENERIC_ALL,
                                  SERVICE_WIN32_SHARE_PROCESS, SERVICE_DEMAND_START,
                                  SERVICE_ERROR_NORMAL, path.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    if !service.is_null() {
        CloseServiceHandle(service);
    } else if GetLastError() != ERROR_SERVICE_EXISTS {
        report_error(b"Failed to create MSI service\n\0".as_ptr() as *const ::core::ffi::c_char);
        ret = 1;
    }
    CloseServiceHandle(scm);
    return ret;
}

unsafe extern "C" fn DoUnregServer() -> DWORD {
    let mut scm: SC_HANDLE = core::ptr::null_mut();
    let mut service: SC_HANDLE = core::ptr::null_mut();
    let mut ret: DWORD = 0 as DWORD;

    scm = OpenSCManagerW(
        core::ptr::null_mut(),
        (SERVICES_ACTIVE_DATABASEW).as_ptr() as LPCWSTR,
        SC_MANAGER_CONNECT as DWORD,
    );
    if scm.is_null() {
        report_error(
            b"Failed to open service control manager\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 1 as DWORD;
    }
    service = OpenServiceW(
        scm,
        RUSTCONV_WSTR_A5468EA1B040.as_ptr(),
        DELETE as DWORD,
    );
    if !service.is_null() {
        if DeleteService(service) == 0 {
            report_error(
                b"Failed to delete MSI service\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            ret = 1 as DWORD;
        }
        CloseServiceHandle(service);
    } else if GetLastError() != ERROR_SERVICE_DOES_NOT_EXIST as DWORD {
        report_error(
            b"Failed to open MSI service\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ret = 1 as DWORD;
    }
    CloseServiceHandle(scm);
    return ret;
}

unsafe extern "system" fn custom_action_thread(arg: *mut core::ffi::c_void) -> DWORD {
    let mut guid: GUID = *(arg as *mut GUID);
    free(arg);
    return __wine_msi_call_dll_function(client_pid, core::ptr::addr_of_mut!(guid)) as DWORD;
}

unsafe extern "C" fn custom_action_server(arg: *const WCHAR) -> i32 {
    let mut guid: GUID = core::mem::zeroed();
    let mut thread_guid: *mut GUID = core::ptr::null_mut();
    let mut thread64: DWORD64 = 0;
    let mut buffer: [WCHAR; 24] = [0; 24];
    let mut thread: HANDLE = core::ptr::null_mut();
    let mut pipe: HANDLE = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let reactos_build: bool = false;

    client_pid = wcstol(arg, core::ptr::null_mut(), 10) as DWORD;
    if client_pid == 0 {
        ERR!("Invalid parameter %s\n", debugstr_w(arg));
        return 1;
    }

    if reactos_build {
        swprintf(
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
            (ARRAY_SIZE!(buffer)) as _,
            RUSTCONV_WSTR_98F59DB8D472.as_ptr(),
            client_pid,
            (core::mem::size_of::<*mut core::ffi::c_void>() * 8) as i32,
        );
    } else {
        swprintf(
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
            (ARRAY_SIZE!(buffer)) as _,
            RUSTCONV_WSTR_98F59DB8D472.as_ptr(),
            client_pid,
            core::mem::size_of::<*mut core::ffi::c_void>() * 8,
        );
    }
    pipe = CreateFileW(
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        GENERIC_READ | GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if pipe == INVALID_HANDLE_VALUE {
        ERR!("Failed to create custom action server pipe: %lu\n", GetLastError());
        return GetLastError() as i32;
    }

    /* We need this to unmarshal streams, and some apps expect it to be present. */
    CoInitializeEx(core::ptr::null_mut(), COINIT_MULTITHREADED);

    while ReadFile(
        pipe,
        core::ptr::addr_of_mut!(guid) as PVOID,
        core::mem::size_of::<GUID>() as DWORD,
        core::ptr::addr_of_mut!(size),
        core::ptr::null_mut(),
    ) != 0
        && size == core::mem::size_of::<GUID>() as DWORD
    {
        if IsEqualGUID!(
            core::ptr::addr_of!(guid),
            core::ptr::addr_of!(GUID_NULL),
        ) != false
        {
            /* package closed; time to shut down */
            CoUninitialize();
            return 0;
        }

        thread_guid = malloc(core::mem::size_of::<GUID>()) as *mut GUID;
        memcpy(
            thread_guid as *mut core::ffi::c_void,
            core::ptr::addr_of!(guid) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        thread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(custom_action_thread),
            thread_guid as PVOID,
            0,
            core::ptr::null_mut(),
        );

        /* give the thread handle to the client to wait on, since we might have
         * to run a nested action and can't block during this one */
        thread64 = thread as DWORD_PTR as DWORD64;
        if WriteFile(
            pipe,
            core::ptr::addr_of_mut!(thread64) as LPCVOID,
            core::mem::size_of::<DWORD64>() as DWORD,
            core::ptr::addr_of_mut!(size),
            core::ptr::null_mut(),
        ) == 0
            || size != core::mem::size_of::<DWORD64>() as DWORD
        {
            ERR!("Failed to write to custom action server pipe: %lu\n", GetLastError());
            CoUninitialize();
            return GetLastError() as i32;
        }
    }
    ERR!("Failed to read from custom action server pipe: %lu\n", GetLastError());
    CoUninitialize();
    return GetLastError() as i32;
}

unsafe extern "C" fn chomp(in_0: *const WCHAR, mut out: *mut WCHAR) -> ::core::ffi::c_int {
    let mut state: chomp_state = CS_TOKEN as chomp_state;
    let mut p: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut ignore: BOOL = 0;
    p = in_0;
    while *p != 0 {
        ignore = TRUE as BOOL;
        match (state as ::core::ffi::c_uint) as i32 {
            CS_WHITESPACE => match *p as ::core::ffi::c_int {
                32 => {}
                34 => {
                    state = CS_QUOTE as chomp_state;
                    count += 1;
                    ignore = FALSE as BOOL;
                }
                _ => {
                    count += 1;
                    ignore = FALSE as BOOL;
                    state = CS_TOKEN as chomp_state;
                }
            },
            CS_TOKEN => match *p as ::core::ffi::c_int {
                34 => {
                    state = CS_QUOTE as chomp_state;
                    ignore = FALSE as BOOL;
                }
                32 => {
                    state = CS_WHITESPACE as chomp_state;
                    if !out.is_null() {
                        let fresh23 = out;
                        out = out.offset(1);
                        *fresh23 = 0 as WCHAR;
                    }
                }
                _ => {
                    if p > in_0
                        && *p.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                            == '"' as i32
                    {
                        if !out.is_null() {
                            let fresh24 = out;
                            out = out.offset(1);
                            *fresh24 = 0 as WCHAR;
                        }
                        count += 1;
                    }
                    ignore = FALSE as BOOL;
                }
            },
            CS_QUOTE => match *p as ::core::ffi::c_int {
                34 => {
                    state = CS_TOKEN as chomp_state;
                    ignore = FALSE as BOOL;
                }
                _ => {
                    ignore = FALSE as BOOL;
                }
            },
            _ => {}
        }
        if ignore == 0 && !out.is_null() {
            let fresh25 = out;
            out = out.offset(1);
            *fresh25 = *p;
        }
        p = p.offset(1);
    }
    if !out.is_null() {
        *out = 0 as WCHAR;
    }
    return count;
}

unsafe extern "C" fn process_args(cmdline: *mut WCHAR, pargc: *mut ::core::ffi::c_int, pargv: *mut *mut *mut WCHAR) {
    let mut argv: *mut *mut WCHAR = core::ptr::null_mut();
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut i: ::core::ffi::c_int;
    let mut count: ::core::ffi::c_int;

    *pargc = 0 as ::core::ffi::c_int;
    *pargv = core::ptr::null_mut();

    count = chomp(cmdline, core::ptr::null_mut());
    p = malloc(
        ((wcslen(cmdline) as usize)
            .wrapping_add(count as usize)
            .wrapping_add(1 as usize))
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if p.is_null() {
        return;
    }

    count = chomp(cmdline, p);
    argv = malloc(
        ((count + 1 as ::core::ffi::c_int) as usize)
            .wrapping_mul(core::mem::size_of::<*mut WCHAR>()),
    ) as *mut *mut WCHAR;
    if argv.is_null() {
        free(p as *mut ::core::ffi::c_void);
        return;
    }

    i = 0 as ::core::ffi::c_int;
    while i < count {
        *argv.offset(i as isize) = p;
        p = p.offset(
            (lstrlenW(p as LPCWSTR) + 1 as ::core::ffi::c_int)
                as isize,
        );
        i += 1;
    }
    *argv.offset(i as isize) = core::ptr::null_mut();

    *pargc = count;
    *pargv = argv;
}

unsafe extern "C" fn process_args_from_reg(
    ident: *const WCHAR,
    pargc: *mut ::core::ffi::c_int,
    pargv: *mut *mut *mut WCHAR,
) -> BOOL {
    let mut r: LONG = 0;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut sz: DWORD = 0 as DWORD;
    let mut type_: DWORD = 0 as DWORD;
    let mut buf: *mut WCHAR;
    let mut ret: BOOL = FALSE as BOOL;

    r = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_9C9A54CDDB92.as_ptr(),
        core::ptr::addr_of_mut!(hkey),
    ) as LONG;
    if r != ERROR_SUCCESS as LONG {
        return (FALSE) as BOOL;
    }
    r = RegQueryValueExW(
        hkey,
        ident as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(type_),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(sz),
    ) as LONG;
    if r == ERROR_SUCCESS as LONG && type_ == REG_SZ as DWORD {
        let mut len: ::core::ffi::c_int =
            lstrlenW(**pargv.offset(0 as isize) as LPCWSTR);
        // Exceeds the C: it allocates only argv[0] + ' ' and then has RegQueryValueExW write the
        // `sz`-byte value after it, a heap overflow on every RunOnceEntries run. Upstream Wine
        // sizes it `sz + (len + 1) * sizeof(WCHAR)`; so does this.
        buf = malloc(
            (sz as usize).wrapping_add(
                ((len + 1 as ::core::ffi::c_int) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
            ),
        ) as *mut WCHAR;
        if buf.is_null() {
            RegCloseKey(hkey);
            return FALSE as BOOL;
        }
        memcpy(
            buf as *mut ::core::ffi::c_void,
            (**pargv.offset(0 as isize)) as *const ::core::ffi::c_void,
            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
        );
        let fresh20 = len;
        len = len + 1;
        *buf.offset(fresh20 as isize) = ' ' as i32 as WCHAR;
        r = RegQueryValueExW(
            hkey,
            ident as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(type_),
            buf.offset(len as isize) as LPBYTE,
            core::ptr::addr_of_mut!(sz),
        ) as LONG;
        if r == ERROR_SUCCESS as LONG {
            process_args(buf, pargc, pargv);
            ret = TRUE as BOOL;
        }
        free(buf as *mut ::core::ffi::c_void);
    }
    RegCloseKey(hkey);
    return ret;
}

unsafe extern "C" fn get_path_with_extension(package_name: *const WCHAR) -> *mut WCHAR {
    static mut ext: [WCHAR; 5] = [0x002Eu16, 0x006D, 0x0073, 0x0069, 0x0000];
    let mut p: u32;
    let mut path: *mut WCHAR;

    path = malloc(
        (wcslen(package_name) as usize) * core::mem::size_of::<WCHAR>()
            + core::mem::size_of::<[WCHAR; 5]>(),
    ) as *mut WCHAR;
    if path.is_null() {
        WINE_ERR!(b"No memory.\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    lstrcpyW(path, package_name);
    p = lstrlenW(path) as u32;
    while p != 0
        && *path.offset(p as isize) != '.' as WCHAR
        && *path.offset(p as isize) != 0x005Cu16
        && *path.offset(p as isize) != '/' as WCHAR
    {
        p = p.wrapping_sub(1);
    }
    if *path.offset(p as isize) == '.' as WCHAR {
        free(path as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    lstrcatW(path, core::ptr::addr_of!(ext) as *const WCHAR);
    return path;
}

unsafe extern "C" fn remove_quotes(filename: *const WCHAR) -> *mut WCHAR {
    let mut ptr: *const WCHAR = filename;
    let mut len: i32 = wcslen(filename as *const wchar_t) as i32;
    let mut ret: *mut WCHAR;

    ret = malloc(
        ((len + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    if *ptr as i32 == '"' as i32 {
        ptr = ptr.offset(1);
        len -= 1;
    }
    wcscpy(ret as *mut wchar_t, ptr as *const wchar_t);
    if len != 0 && *ret.offset((len - 1) as isize) as i32 == '"' as i32 {
        *ret.offset((len - 1) as isize) = 0 as WCHAR;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPSTR,
    nCmdShow: i32,
) -> i32 {
    let mut i: i32;
    let mut FunctionInstall: BOOL = (FALSE as i32);
    let mut FunctionInstallAdmin: BOOL = (FALSE as i32);
    let mut FunctionRepair: BOOL = (FALSE as i32);
    let mut FunctionAdvertise: BOOL = (FALSE as i32);
    let mut FunctionPatch: BOOL = (FALSE as i32);
    let mut FunctionDllRegisterServer: BOOL = (FALSE as i32);
    let mut FunctionDllUnregisterServer: BOOL = (FALSE as i32);
    let mut FunctionRegServer: BOOL = (FALSE as i32);
    let mut FunctionUnregServer: BOOL = (FALSE as i32);
    let mut FunctionServer: BOOL = (FALSE as i32);
    let mut FunctionUnknown: BOOL = (FALSE as i32);

    let mut PackageName: LPWSTR = core::ptr::null_mut();
    let mut Properties: LPWSTR = core::ptr::null_mut();
    let mut property_list: *mut string_list = core::ptr::null_mut();

    let mut RepairMode: DWORD = 0;

    let mut AdvertiseMode: DWORD_PTR = 0;
    let mut transform_list: *mut string_list = core::ptr::null_mut();
    let mut Language: LANGID = 0;

    let mut LogMode: DWORD = 0;
    let mut LogAttributes: DWORD = 0;

    let mut PatchFileName: LPWSTR = core::ptr::null_mut();
    let mut InstallType: INSTALLTYPE = INSTALLTYPE_DEFAULT;

    let mut InstallUILevel: INSTALLUILEVEL = INSTALLUILEVEL_FULL;

    let mut DllName: LPWSTR = core::ptr::null_mut();
    let mut ReturnCode: DWORD;
    let mut argc: i32 = 0;
    let mut argvW: *mut LPWSTR = core::ptr::null_mut();
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut package_unquoted: *mut WCHAR = core::ptr::null_mut();
    let mut dll_unquoted: *mut WCHAR = core::ptr::null_mut();
    let mut patch_unquoted: *mut WCHAR = core::ptr::null_mut();
    let mut log_unquoted: *mut WCHAR = core::ptr::null_mut();

    InitCommonControls();

    /* parse the command line */
    process_args(GetCommandLineW(), core::ptr::addr_of_mut!(argc), core::ptr::addr_of_mut!(argvW));

    /*
     * If the args begin with /@ IDENT then we need to load the real
     * command line out of the RunOnceEntries key in the registry.
     *  We do that before starting to process the real commandline,
     * then overwrite the commandline again.
     */
    if argc > 1
        && msi_option_equal(*argvW.offset(1 as isize), b"@\0".as_ptr() as LPCSTR) != 0
    {
        if process_args_from_reg(
            *argvW.offset(2 as isize),
            core::ptr::addr_of_mut!(argc),
            core::ptr::addr_of_mut!(argvW),
        ) == 0
        {
            return 1;
        }
    }

    if argc == 3
        && msi_option_equal(*argvW.offset(1 as isize), b"Embedding\0".as_ptr() as LPCSTR) != 0
    {
        return custom_action_server(*argvW.offset(2 as isize));
    }

    i = 1;
    while i < argc {
        WINE_TRACE!(
            b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
            i,
            wine_dbgstr_w(*argvW.offset(i as isize)),
        );

        if msi_option_equal(*argvW.offset(i as isize), b"regserver\0".as_ptr() as LPCSTR) != 0 {
            FunctionRegServer = (TRUE as i32);
        } else if msi_option_equal(*argvW.offset(i as isize), b"unregserver\0".as_ptr() as LPCSTR) != 0
            || msi_option_equal(*argvW.offset(i as isize), b"unregister\0".as_ptr() as LPCSTR) != 0
            || msi_option_equal(*argvW.offset(i as isize), b"unreg\0".as_ptr() as LPCSTR) != 0
        {
            FunctionUnregServer = (TRUE as i32);
        } else if msi_option_prefix(*argvW.offset(i as isize), b"i\0".as_ptr() as LPCSTR) != 0
            || msi_option_prefix(*argvW.offset(i as isize), b"package\0".as_ptr() as LPCSTR) != 0
        {
            let mut argvWi: LPWSTR = *argvW.offset(i as isize);
            let mut argLen: i32 = if msi_option_prefix(*argvW.offset(i as isize), b"i\0".as_ptr() as LPCSTR) != 0 {
                2
            } else {
                8
            };
            FunctionInstall = (TRUE as i32);
            if lstrlenW(*argvW.offset(i as isize)) > argLen {
                argvWi = argvWi.offset(argLen as isize);
            } else {
                i += 1;
                if i >= argc {
                    ShowUsage(1);
                }
                WINE_TRACE!(
                    b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                    i,
                    wine_dbgstr_w(*argvW.offset(i as isize)),
                );
                argvWi = *argvW.offset(i as isize);
            }
            PackageName = argvWi;
        } else if msi_option_equal(*argvW.offset(i as isize), b"a\0".as_ptr() as LPCSTR) != 0 {
            FunctionInstall = (TRUE as i32);
            FunctionInstallAdmin = (TRUE as i32);
            InstallType = INSTALLTYPE_NETWORK_IMAGE;
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PackageName = *argvW.offset(i as isize);
            StringListAppend(core::ptr::addr_of_mut!(property_list), RUSTCONV_WSTR_C06870C8E03C.as_ptr());
        } else if msi_option_prefix(*argvW.offset(i as isize), b"f\0".as_ptr() as LPCSTR) != 0 {
            let mut j: i32;
            let len: i32 = lstrlenW(*argvW.offset(i as isize));
            FunctionRepair = (TRUE as i32);
            j = 2;
            while j < len {
                match *(*argvW.offset(i as isize)).offset(j as isize) as i32 {
                    80 | 112 => {
                        RepairMode |= REINSTALLMODE_FILEMISSING;
                    }
                    79 | 111 => {
                        RepairMode |= REINSTALLMODE_FILEOLDERVERSION;
                    }
                    69 | 101 => {
                        RepairMode |= REINSTALLMODE_FILEEQUALVERSION;
                    }
                    68 | 100 => {
                        RepairMode |= REINSTALLMODE_FILEEXACT;
                    }
                    67 | 99 => {
                        RepairMode |= REINSTALLMODE_FILEVERIFY;
                    }
                    65 | 97 => {
                        RepairMode |= REINSTALLMODE_FILEREPLACE;
                    }
                    85 | 117 => {
                        RepairMode |= REINSTALLMODE_USERDATA;
                    }
                    77 | 109 => {
                        RepairMode |= REINSTALLMODE_MACHINEDATA;
                    }
                    83 | 115 => {
                        RepairMode |= REINSTALLMODE_SHORTCUT;
                    }
                    86 | 118 => {
                        RepairMode |= REINSTALLMODE_PACKAGE;
                    }
                    _ => {
                        report_error(
                            b"Unknown option \"%c\" in Repair mode\n\0".as_ptr() as *const i8,
                            *(*argvW.offset(i as isize)).offset(j as isize) as i32,
                        );
                    }
                }
                j += 1;
            }
            if len == 2 {
                RepairMode = REINSTALLMODE_FILEMISSING
                    | REINSTALLMODE_FILEEQUALVERSION
                    | REINSTALLMODE_FILEVERIFY
                    | REINSTALLMODE_MACHINEDATA
                    | REINSTALLMODE_SHORTCUT;
            }
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PackageName = *argvW.offset(i as isize);
        } else if msi_option_prefix(*argvW.offset(i as isize), b"x\0".as_ptr() as LPCSTR) != 0
            || msi_option_equal(*argvW.offset(i as isize), b"uninstall\0".as_ptr() as LPCSTR) != 0
        {
            FunctionInstall = (TRUE as i32);
            if msi_option_prefix(*argvW.offset(i as isize), b"x\0".as_ptr() as LPCSTR) != 0 {
                PackageName = (*argvW.offset(i as isize)).offset(2 as isize);
            }
            if PackageName.is_null() || *PackageName.offset(0 as isize) == 0 {
                i += 1;
                if i >= argc {
                    ShowUsage(1);
                }
                PackageName = *argvW.offset(i as isize);
            }
            WINE_TRACE!(
                b"PackageName = %s\n\0".as_ptr() as *const i8,
                wine_dbgstr_w(PackageName),
            );
            StringListAppend(core::ptr::addr_of_mut!(property_list), RUSTCONV_WSTR_6E220D871E8B.as_ptr());
        } else if msi_option_prefix(*argvW.offset(i as isize), b"j\0".as_ptr() as LPCSTR) != 0 {
            let mut j: i32;
            let len: i32 = lstrlenW(*argvW.offset(i as isize));
            FunctionAdvertise = (TRUE as i32);
            j = 2;
            while j < len {
                match *(*argvW.offset(i as isize)).offset(j as isize) as i32 {
                    85 | 117 => {
                        AdvertiseMode = (ADVERTISEFLAGS_USERASSIGN) as _;
                    }
                    77 | 109 => {
                        AdvertiseMode = (ADVERTISEFLAGS_MACHINEASSIGN) as _;
                    }
                    _ => {
                        report_error(
                            b"Unknown option \"%c\" in Advertise mode\n\0".as_ptr() as *const i8,
                            *(*argvW.offset(i as isize)).offset(j as isize) as i32,
                        );
                    }
                }
                j += 1;
            }
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PackageName = *argvW.offset(i as isize);
        } else if msi_strequal(*argvW.offset(i as isize), b"u\0".as_ptr() as LPCSTR) != 0 {
            FunctionAdvertise = (TRUE as i32);
            AdvertiseMode = (ADVERTISEFLAGS_USERASSIGN) as _;
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PackageName = *argvW.offset(i as isize);
        } else if msi_strequal(*argvW.offset(i as isize), b"m\0".as_ptr() as LPCSTR) != 0 {
            FunctionAdvertise = (TRUE as i32);
            AdvertiseMode = (ADVERTISEFLAGS_MACHINEASSIGN) as _;
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PackageName = *argvW.offset(i as isize);
        } else if msi_option_equal(*argvW.offset(i as isize), b"t\0".as_ptr() as LPCSTR) != 0 {
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            StringListAppend(core::ptr::addr_of_mut!(transform_list), *argvW.offset(i as isize));
        } else if msi_option_equal(*argvW.offset(i as isize), b"g\0".as_ptr() as LPCSTR) != 0 {
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            Language = msi_atou(*argvW.offset(i as isize)) as LANGID;
        } else if msi_option_prefix(*argvW.offset(i as isize), b"l\0".as_ptr() as LPCSTR) != 0 {
            let mut j: i32;
            let len: i32 = lstrlenW(*argvW.offset(i as isize));
            j = 2;
            while j < len {
                match *(*argvW.offset(i as isize)).offset(j as isize) as i32 {
                    73 | 105 => {
                        LogMode |= INSTALLLOGMODE_INFO;
                    }
                    87 | 119 => {
                        LogMode |= INSTALLLOGMODE_WARNING;
                    }
                    69 | 101 => {
                        LogMode |= INSTALLLOGMODE_ERROR;
                    }
                    65 | 97 => {
                        LogMode |= INSTALLLOGMODE_ACTIONSTART;
                    }
                    82 | 114 => {
                        LogMode |= INSTALLLOGMODE_ACTIONDATA;
                    }
                    85 | 117 => {
                        LogMode |= INSTALLLOGMODE_USER;
                    }
                    67 | 99 => {
                        LogMode |= INSTALLLOGMODE_COMMONDATA;
                    }
                    77 | 109 => {
                        LogMode |= INSTALLLOGMODE_FATALEXIT;
                    }
                    79 | 111 => {
                        LogMode |= INSTALLLOGMODE_OUTOFDISKSPACE;
                    }
                    80 | 112 => {
                        LogMode |= INSTALLLOGMODE_PROPERTYDUMP;
                    }
                    86 | 118 => {
                        LogMode |= INSTALLLOGMODE_VERBOSE;
                    }
                    42 => {
                        LogMode = INSTALLLOGMODE_FATALEXIT
                            | INSTALLLOGMODE_ERROR
                            | INSTALLLOGMODE_WARNING
                            | INSTALLLOGMODE_USER
                            | INSTALLLOGMODE_INFO
                            | INSTALLLOGMODE_RESOLVESOURCE
                            | INSTALLLOGMODE_OUTOFDISKSPACE
                            | INSTALLLOGMODE_ACTIONSTART
                            | INSTALLLOGMODE_ACTIONDATA
                            | INSTALLLOGMODE_COMMONDATA
                            | INSTALLLOGMODE_PROPERTYDUMP
                            | INSTALLLOGMODE_PROGRESS
                            | INSTALLLOGMODE_INITIALIZE
                            | INSTALLLOGMODE_TERMINATE
                            | INSTALLLOGMODE_SHOWDIALOG;
                    }
                    43 => {
                        LogAttributes |= INSTALLLOGATTRIBUTES_APPEND;
                    }
                    33 => {
                        LogAttributes |= INSTALLLOGATTRIBUTES_FLUSHEACHLINE;
                    }
                    _ => {}
                }
                j += 1;
            }
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            log_unquoted = remove_quotes(*argvW.offset(i as isize));
            if log_unquoted.is_null() {
                return 1;
            }
            if MsiEnableLogW(LogMode, log_unquoted as LPCWSTR, LogAttributes) != ERROR_SUCCESS {
                report_error(
                    b"Logging in %s (0x%08lx, %lu) failed\n\0".as_ptr() as *const i8,
                    wine_dbgstr_w(log_unquoted),
                    LogMode,
                    LogAttributes,
                );
                ExitProcess(1);
            }
        } else if msi_option_equal(*argvW.offset(i as isize), b"p\0".as_ptr() as LPCSTR) != 0
            || msi_option_equal(*argvW.offset(i as isize), b"update\0".as_ptr() as LPCSTR) != 0
        {
            FunctionPatch = (TRUE as i32);
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            PatchFileName = *argvW.offset(i as isize);
        } else if msi_option_prefix(*argvW.offset(i as isize), b"q\0".as_ptr() as LPCSTR) != 0 {
            if lstrlenW(*argvW.offset(i as isize)) == 2
                || msi_strequal(
                    (*argvW.offset(i as isize)).offset(2 as isize),
                    b"n\0".as_ptr() as LPCSTR,
                ) != 0
                || msi_strequal(
                    (*argvW.offset(i as isize)).offset(2 as isize),
                    b"uiet\0".as_ptr() as LPCSTR,
                ) != 0
            {
                silent = (TRUE as i32);
                InstallUILevel = INSTALLUILEVEL_NONE;
            } else if msi_strequal(
                (*argvW.offset(i as isize)).offset(2 as isize),
                b"r\0".as_ptr() as LPCSTR,
            ) != 0
            {
                InstallUILevel = INSTALLUILEVEL_REDUCED;
            } else if msi_strequal(
                (*argvW.offset(i as isize)).offset(2 as isize),
                b"f\0".as_ptr() as LPCSTR,
            ) != 0
            {
                InstallUILevel = ((INSTALLUILEVEL_FULL as u32) | (INSTALLUILEVEL_ENDDIALOG as u32)) as INSTALLUILEVEL;
            } else if msi_strequal(
                (*argvW.offset(i as isize)).offset(2 as isize),
                b"n+\0".as_ptr() as LPCSTR,
            ) != 0
            {
                InstallUILevel = ((INSTALLUILEVEL_NONE as u32) | (INSTALLUILEVEL_ENDDIALOG as u32)) as INSTALLUILEVEL;
            } else if msi_strprefix(
                (*argvW.offset(i as isize)).offset(2 as isize),
                b"b\0".as_ptr() as LPCSTR,
            ) != 0
            {
                let mut ptr: *const WCHAR = (*argvW.offset(i as isize)).offset(3 as isize);

                InstallUILevel = INSTALLUILEVEL_BASIC;

                while *ptr != 0 {
                    if msi_strprefix(ptr, b"+\0".as_ptr() as LPCSTR) != 0 {
                        InstallUILevel = ((InstallUILevel as u32) | (INSTALLUILEVEL_ENDDIALOG as u32)) as INSTALLUILEVEL;
                    }
                    if msi_strprefix(ptr, b"-\0".as_ptr() as LPCSTR) != 0 {
                        InstallUILevel = ((InstallUILevel as u32) | (INSTALLUILEVEL_PROGRESSONLY as u32)) as INSTALLUILEVEL;
                    }
                    if msi_strprefix(ptr, b"!\0".as_ptr() as LPCSTR) != 0 {
                        WINE_FIXME!(b"Unhandled modifier: !\n\0".as_ptr() as *const i8);
                        InstallUILevel = ((InstallUILevel as u32) | (INSTALLUILEVEL_HIDECANCEL as u32)) as INSTALLUILEVEL;
                    }
                    ptr = ptr.offset(1);
                }
            } else {
                report_error(
                    b"Unknown option \"%s\" for UI level\n\0".as_ptr() as *const i8,
                    wine_dbgstr_w((*argvW.offset(i as isize)).offset(2 as isize)),
                );
            }
        } else if msi_option_equal(*argvW.offset(i as isize), b"passive\0".as_ptr() as LPCSTR) != 0 {
            InstallUILevel = ((INSTALLUILEVEL_BASIC as u32)
                | (INSTALLUILEVEL_PROGRESSONLY as u32)
                | (INSTALLUILEVEL_HIDECANCEL as u32)) as INSTALLUILEVEL;
            StringListAppend(core::ptr::addr_of_mut!(property_list), RUSTCONV_WSTR_5D89CFDA588E.as_ptr());
        } else if msi_option_equal(*argvW.offset(i as isize), b"y\0".as_ptr() as LPCSTR) != 0 {
            FunctionDllRegisterServer = (TRUE as i32);
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            DllName = *argvW.offset(i as isize);
        } else if msi_option_equal(*argvW.offset(i as isize), b"z\0".as_ptr() as LPCSTR) != 0 {
            FunctionDllUnregisterServer = (TRUE as i32);
            i += 1;
            if i >= argc {
                ShowUsage(1);
            }
            WINE_TRACE!(
                b"argvW[%d] = %s\n\0".as_ptr() as *const i8,
                i,
                wine_dbgstr_w(*argvW.offset(i as isize)),
            );
            DllName = *argvW.offset(i as isize);
        } else if msi_option_equal(*argvW.offset(i as isize), b"help\0".as_ptr() as LPCSTR) != 0
            || msi_option_equal(*argvW.offset(i as isize), b"?\0".as_ptr() as LPCSTR) != 0
        {
            ShowUsage(0);
        } else if msi_option_equal(*argvW.offset(i as isize), b"m\0".as_ptr() as LPCSTR) != 0 {
            FunctionUnknown = (TRUE as i32);
            WINE_FIXME!(b"Unknown parameter /m\n\0".as_ptr() as *const i8);
        } else if msi_option_equal(*argvW.offset(i as isize), b"D\0".as_ptr() as LPCSTR) != 0 {
            FunctionUnknown = (TRUE as i32);
            WINE_FIXME!(b"Unknown parameter /D\n\0".as_ptr() as *const i8);
        } else if msi_option_equal(*argvW.offset(i as isize), b"V\0".as_ptr() as LPCSTR) != 0 {
            FunctionServer = (TRUE as i32);
        } else {
            StringListAppend(core::ptr::addr_of_mut!(property_list), *argvW.offset(i as isize));
        }
        i += 1;
    }

    /* start the GUI */
    MsiSetInternalUI(InstallUILevel, core::ptr::null_mut());

    Properties = build_properties(property_list);

    if FunctionInstallAdmin != 0 && FunctionPatch != 0 {
        FunctionInstall = (FALSE as i32);
    }

    if !PackageName.is_null() && {
        package_unquoted = remove_quotes(PackageName);
        package_unquoted.is_null()
    } {
        return 1;
    }
    if !PatchFileName.is_null() && {
        patch_unquoted = remove_quotes(PatchFileName);
        patch_unquoted.is_null()
    } {
        return 1;
    }
    if !DllName.is_null() && {
        dll_unquoted = remove_quotes(DllName);
        dll_unquoted.is_null()
    } {
        return 1;
    }

    ReturnCode = 1;
    if FunctionInstall != 0 {
        if IsProductCode(package_unquoted) != 0 {
            ReturnCode = MsiConfigureProductExW(
                package_unquoted,
                0,
                INSTALLSTATE_DEFAULT,
                Properties,
            ) as DWORD;
        } else {
            ReturnCode = MsiInstallProductW(package_unquoted, Properties) as DWORD;
            if ReturnCode == ERROR_FILE_NOT_FOUND
                && {
                    path = get_path_with_extension(package_unquoted);
                    !path.is_null()
                }
            {
                ReturnCode = MsiInstallProductW(path, Properties) as DWORD;
                free((path).cast());
            }
        }
    } else if FunctionRepair != 0 {
        if IsProductCode(package_unquoted) != 0 {
            WINE_FIXME!(b"Product code treatment not implemented yet\n\0".as_ptr() as *const i8);
        } else {
            ReturnCode = MsiReinstallProductW(package_unquoted, RepairMode) as DWORD;
            if ReturnCode == ERROR_FILE_NOT_FOUND
                && {
                    path = get_path_with_extension(package_unquoted);
                    !path.is_null()
                }
            {
                ReturnCode = MsiReinstallProductW(path, RepairMode) as DWORD;
                free((path).cast());
            }
        }
    } else if FunctionAdvertise != 0 {
        let mut Transforms: LPWSTR = build_transforms(property_list);
        ReturnCode = MsiAdvertiseProductW(
            package_unquoted,
            AdvertiseMode as LPWSTR,
            Transforms,
            Language,
        ) as DWORD;
    } else if FunctionPatch != 0 {
        ReturnCode = MsiApplyPatchW(
            patch_unquoted,
            package_unquoted,
            InstallType,
            Properties,
        ) as DWORD;
    } else if FunctionDllRegisterServer != 0 {
        ReturnCode = DoDllRegisterServer(dll_unquoted) as DWORD;
    } else if FunctionDllUnregisterServer != 0 {
        ReturnCode = DoDllUnregisterServer(dll_unquoted) as DWORD;
    } else if FunctionRegServer != 0 {
        ReturnCode = DoRegServer() as DWORD;
    } else if FunctionUnregServer != 0 {
        ReturnCode = DoUnregServer() as DWORD;
    } else if FunctionServer != 0 {
        ReturnCode = DoService() as DWORD;
    } else if FunctionUnknown != 0 {
        WINE_FIXME!(b"Unknown function, ignoring\n\0".as_ptr() as *const i8);
    } else {
        ShowUsage(1);
    }

    return ReturnCode as i32;
}
