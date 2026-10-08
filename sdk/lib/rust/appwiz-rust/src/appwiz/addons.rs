use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const GECKO_VERSION: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"2.40\0") };

// addons.c picks these by `#ifdef __i386__`. c2rust laid the table out on ONE arch and froze
// that arch's branch (`""` / `"???"`), which would have disabled the i386 Gecko install.
#[cfg(target_arch = "x86")]
pub const ARCH_STRING: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"x86\0") };
#[cfg(not(target_arch = "x86"))]
pub const ARCH_STRING: [::core::ffi::c_char; 1] =
    unsafe { ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0") };

#[cfg(target_arch = "x86")]
pub const GECKO_SHA: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"8a3adedf3707973d1ed4ac3b2e791486abf814bd\0",
    )
};
#[cfg(not(target_arch = "x86"))]
pub const GECKO_SHA: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"???\0") };

// "wine_gecko-" GECKO_VERSION "-" ARCH_STRING ".msi"
#[cfg(target_arch = "x86")]
const GECKO_FILE_NAME: &[u8] = b"wine_gecko-2.40-x86.msi\0";
#[cfg(not(target_arch = "x86"))]
const GECKO_FILE_NAME: &[u8] = b"wine_gecko-2.40-.msi\0";

static mut addons_info: [addon_info_t; 1] = unsafe {
    [addon_info_t {
        version: GECKO_VERSION.as_ptr(),
        file_name: GECKO_FILE_NAME.as_ptr() as *const ::core::ffi::c_char,
        sha: GECKO_SHA.as_ptr(),
        config_key: b"MSHTML\0" as *const u8 as *const ::core::ffi::c_char,
        dir_config_key: b"GeckoCabDir\0" as *const u8 as *const ::core::ffi::c_char,
        dialog_template: (1200 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
            as ULONG_PTR as LPWSTR as LPCWSTR,
    }]
};

static mut addon: *const addon_info_t = ::core::ptr::null::<addon_info_t>();

static mut install_dialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut csLock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

static mut download_binding: *mut IBinding = ::core::ptr::null::<IBinding>() as *mut IBinding;

static mut GeckoUrl: [WCHAR; 54] = unsafe {
    ::core::mem::transmute::<
        [u8; 108],
        [WCHAR; 54],
    >(
        *b"h\0t\0t\0p\0s\0:\0/\0/\0s\0v\0n\0.\0r\0e\0a\0c\0t\0o\0s\0.\0o\0r\0g\0/\0a\0m\0i\0n\0e\0/\0w\0i\0n\0e\0_\0g\0e\0c\0k\0o\0-\x002\0.\x004\x000\0-\0x\08\x006\0.\0m\0s\0i\0\0\0",
    )
};

static mut mshtml_keyA: [CHAR; 21] = [
    'S' as i32 as CHAR,
    'o' as i32 as CHAR,
    'f' as i32 as CHAR,
    't' as i32 as CHAR,
    'w' as i32 as CHAR,
    'a' as i32 as CHAR,
    'r' as i32 as CHAR,
    'e' as i32 as CHAR,
    '\\' as i32 as CHAR,
    'W' as i32 as CHAR,
    'i' as i32 as CHAR,
    'n' as i32 as CHAR,
    'e' as i32 as CHAR,
    '\\' as i32 as CHAR,
    'M' as i32 as CHAR,
    'S' as i32 as CHAR,
    'H' as i32 as CHAR,
    'T' as i32 as CHAR,
    'M' as i32 as CHAR,
    'L' as i32 as CHAR,
    0 as ::core::ffi::c_int as CHAR,
];

static mut InstallCallback: IBindStatusCallback = unsafe {
    IBindStatusCallback {
        lpVtbl: &raw const InstallCallbackVtbl,
    }
};

#[repr(C)]
pub struct addon_info_t {
    pub version: *const i8,
    pub file_name: *const i8,
    pub sha: *const i8,
    pub config_key: *const i8,
    pub dir_config_key: *const i8,
    pub dialog_template: LPCWSTR,
}
pub type Paddon_info_t = *mut addon_info_t;

unsafe impl Sync for addon_info_t {}

// Value of ID_DWL_GECKO_DIALOG is defined in the resource header, which was not provided.
// Placeholder value 0; must be replaced with the real resource id.
pub const ID_DWL_GECKO_DIALOG: u32 = 0;

pub type PSHA_CTX = *mut SHA_CTX;

// void WINAPI A_SHAInit(PSHA_CTX);
// void WINAPI A_SHAUpdate(PSHA_CTX,const unsigned char*,UINT);
// void WINAPI A_SHAFinal(PSHA_CTX,PULONG);
// (prototypes only; resolved as imports by the caller)

pub const INSTALL_OK: i32 = 0;
pub const INSTALL_FAILED: i32 = 1;
pub const INSTALL_NEXT: i32 = 2;

unsafe extern "C" fn sha_check(mut file_name: *const WCHAR) -> BOOL {
    let mut file_map: *const ::core::ffi::c_uchar = ::core::ptr::null::<::core::ffi::c_uchar>();
    let mut file: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut map: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut sha: [ULONG; 5] = [0; 5];
    let mut buf: [::core::ffi::c_char; 2 * ::core::mem::size_of::<[ULONG; 5]>() + 1] =
        [0; 2 * ::core::mem::size_of::<[ULONG; 5]>() + 1];
    let mut ctx: SHA_CTX = SHA_CTX {
        Unknown: [0; 6],
        State: [0; 5],
        Count: [0; 2],
        Buffer: [0; 64],
    };
    let mut size: DWORD = 0;
    let mut i: DWORD = 0;

    file = CreateFileW(
        file_name as LPCWSTR,
        GENERIC_READ as DWORD,
        0 as DWORD,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_READONLY as DWORD,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    );
    if file == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    size = GetFileSize(file, ::core::ptr::null_mut::<DWORD>());

    map = CreateFileMappingW(
        file,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        PAGE_READONLY as DWORD,
        0 as DWORD,
        0 as DWORD,
        ::core::ptr::null::<WCHAR>(),
    );
    CloseHandle(file);
    if map.is_null() {
        return (FALSE) as BOOL;
    }

    file_map = MapViewOfFile(
        map,
        FILE_MAP_READ as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as SIZE_T,
    ) as *const ::core::ffi::c_uchar;
    CloseHandle(map);
    if file_map.is_null() {
        return (FALSE) as BOOL;
    }

    A_SHAInit(::core::ptr::addr_of_mut!(ctx));
    A_SHAUpdate(::core::ptr::addr_of_mut!(ctx), file_map, size as UINT);
    A_SHAFinal(::core::ptr::addr_of_mut!(ctx), ::core::ptr::addr_of_mut!(sha) as PULONG);

    UnmapViewOfFile(file_map as LPCVOID);

    i = 0 as DWORD;
    while (i as usize) < ::core::mem::size_of::<[ULONG; 5]>() {
        sprintf(
            (::core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char)
                .offset(i.wrapping_mul(2 as DWORD) as isize),
            b"%02x\0" as *const u8 as *const ::core::ffi::c_char,
            *(::core::ptr::addr_of_mut!(sha) as *mut ULONG as *mut ::core::ffi::c_uchar)
                .offset(i as isize) as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }

    if strcmp(
        ::core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
        (*addon).sha,
    ) != 0
    {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int)))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/appwiz/addons.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"sha_check\0" as *const u8 as *const ::core::ffi::c_char,
                115 as ::core::ffi::c_int,
                b"Got %s, expected %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::ptr::addr_of_mut!(buf) as *mut ::core::ffi::c_char,
                (*addon).sha,
            );
        }
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn set_status(id: DWORD) {
    let status: HWND = GetDlgItem(install_dialog, ID_DWL_STATUS as c_int);
    let mut buf: [WCHAR; 64] = [0; 64];

    LoadStringW(
        hApplet,
        id as UINT,
        core::ptr::addr_of_mut!(buf) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 64]>()
            / core::mem::size_of::<WCHAR>()) as c_int,
    );
    SendMessageW(
        status,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn install_file(file_name: *const WCHAR) -> install_res {
    let mut res: ULONG;

    res = MsiInstallProductW(file_name as LPCWSTR, core::ptr::null::<WCHAR>()) as ULONG;
    if res != ERROR_SUCCESS as ULONG {
        return (INSTALL_FAILED) as install_res;
    }

    return (INSTALL_OK) as install_res;
}

unsafe extern "C" fn install_from_unix_file(dir: *const c_char, subdir: *const c_char, file_name: *const c_char) -> install_res {
    let mut dos_file_name: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut file_path: *mut c_char = core::ptr::null_mut::<c_char>();
    let mut fd: c_int = 0;
    let mut len: c_int = 0;
    let mut ret: install_res = (INSTALL_OK) as u32;
    let mut res: UINT = 0;

    len = strlen(dir) as c_int;
    file_path = heap_alloc(
        (len as size_t)
            .wrapping_add(strlen(subdir))
            .wrapping_add(strlen(file_name))
            .wrapping_add(3 as size_t),
    ) as *mut c_char;
    if file_path.is_null() {
        return (INSTALL_FAILED) as install_res;
    }

    memcpy(file_path as *mut c_void, dir as *const c_void, len as size_t);
    if len != 0
        && *file_path.offset((len - 1) as isize) as c_int != '/' as i32
        && *file_path.offset((len - 1) as isize) as c_int != '\\' as i32
    {
        let fresh15 = len;
        len = len + 1;
        *file_path.offset(fresh15 as isize) = '/' as i32 as c_char;
    }
    if *subdir != 0 {
        strcpy(file_path.offset(len as isize), subdir);
        len = (len as size_t).wrapping_add(strlen(subdir)) as c_int;
        let fresh16 = len;
        len = len + 1;
        *file_path.offset(fresh16 as isize) = '/' as i32 as c_char;
    }
    strcpy(file_path.offset(len as isize), file_name);

    fd = _open(file_path, O_RDONLY);
    if fd == -(1) {
        if (*__wine_dbch___default).flags as c_int
            & (1 as c_int) << __WINE_DBCL_TRACE as c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/cpl/appwiz/addons.c\0"
                    as *const u8 as *const c_char,
                b"install_from_unix_file\0" as *const u8 as *const c_char,
                175 as c_int,
                b"%s not found\n\0" as *const u8 as *const c_char,
                debugstr_a(file_path),
            );
        }
        heap_free(file_path as *mut c_void);
        return (INSTALL_NEXT) as install_res;
    }

    _close(fd);

    if (*__wine_dbch___default).flags as c_int
        & (1 as c_int) << __WINE_DBCL_WARN as c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/cpl/appwiz/addons.c\0"
                as *const u8 as *const c_char,
            b"install_from_unix_file\0" as *const u8 as *const c_char,
            182 as c_int,
            b"Could not get wine_get_dos_file_name function, calling install_cab directly.\n\0"
                as *const u8 as *const c_char,
        );
    }

    res = MultiByteToWideChar(
        CP_ACP,
        0,
        file_path as LPCSTR,
        -(1),
        0 as LPWSTR,
        0,
    ) as UINT;
    dos_file_name = heap_alloc((res as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t)) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        file_path as LPCSTR,
        -(1),
        dos_file_name,
        res as c_int,
    );

    heap_free(file_path as *mut c_void);

    ret = install_file(dos_file_name as *const WCHAR);

    heap_free(dos_file_name as *mut c_void);
    return ret;
}

unsafe extern "C" fn install_from_registered_dir() -> install_res {
    let mut package_dir: *mut c_char;
    let mut res: DWORD;
    let mut type_: DWORD = 0;
    let mut size: DWORD = MAX_PATH;
    let mut ret: install_res;

    package_dir = heap_alloc((size as size_t).wrapping_add(core::mem::size_of::<*const c_char>() as size_t)) as *mut c_char;

    res = RegGetValueA(HKEY_CURRENT_USER, &raw const mshtml_keyA as LPCSTR, b"GeckoCabDir\0".as_ptr() as LPCSTR, RRF_RT_ANY, &raw mut type_, package_dir as PBYTE as PVOID, &raw mut size) as DWORD;
    if res == ERROR_MORE_DATA {
        package_dir = heap_realloc(package_dir as *mut c_void, (size as size_t).wrapping_add(core::mem::size_of::<*const c_char>() as size_t)) as *mut c_char;
        res = RegGetValueA(HKEY_CURRENT_USER, &raw const mshtml_keyA as LPCSTR, b"GeckoCabDir\0".as_ptr() as LPCSTR, RRF_RT_ANY, &raw mut type_, package_dir as PBYTE as PVOID, &raw mut size) as DWORD;
    }

    if res != ERROR_SUCCESS || (type_ != REG_SZ && type_ != REG_EXPAND_SZ) {
        heap_free(package_dir as *mut c_void);
        return (INSTALL_FAILED) as install_res;
    }

    if type_ == REG_EXPAND_SZ {
        size = ExpandEnvironmentStringsA(package_dir as LPCSTR, core::ptr::null_mut::<CHAR>(), 0);
        if size != 0 {
            let mut buf: *mut c_char = heap_alloc((size as size_t).wrapping_add(core::mem::size_of::<*const c_char>() as size_t)) as *mut c_char;
            ExpandEnvironmentStringsA(package_dir as LPCSTR, buf as LPSTR, size);
            heap_free(package_dir as *mut c_void);
            package_dir = buf;
        }
    }

    if (*__wine_dbch___default).flags as c_int & (1 as c_int) << __WINE_DBCL_TRACE as c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/cpl/appwiz/addons.c\0".as_ptr() as *const c_char,
            b"install_from_registered_dir\0".as_ptr() as *const c_char,
            231 as c_int,
            b"Trying %s/%s\n\0".as_ptr() as *const c_char,
            debugstr_a(package_dir as LPCSTR),
            debugstr_a((*addon).file_name),
        );
    }

    ret = install_from_unix_file(package_dir, b"\0".as_ptr() as *const c_char, (*addon).file_name);

    heap_free(package_dir as *mut c_void);
    return ret;
}

unsafe extern "system" fn InstallCallback_QueryInterface(iface: *mut IBindStatusCallback, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false || IsEqualGUID!(core::ptr::addr_of!(IID_IBindStatusCallback), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    return E_INVALIDARG;
}
unsafe extern "system" fn InstallCallback_AddRef(iface: *mut IBindStatusCallback) -> ULONG {
    2
}
unsafe extern "system" fn InstallCallback_Release(iface: *mut IBindStatusCallback) -> ULONG {
    1
}
unsafe extern "system" fn InstallCallback_OnStartBinding(iface: *mut IBindStatusCallback, dwReserved: DWORD, pib: *mut IBinding) -> HRESULT {
    set_status(IDS_DOWNLOADING as DWORD);

    IBinding_AddRef(pib);

    EnterCriticalSection(core::ptr::addr_of_mut!(csLock));
    download_binding = pib;
    LeaveCriticalSection(core::ptr::addr_of_mut!(csLock));

    return S_OK as HRESULT;
}
unsafe extern "system" fn InstallCallback_GetPriority(
    iface: *mut IBindStatusCallback,
    pnPriority: *mut LONG,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn InstallCallback_OnLowResource(
    iface: *mut IBindStatusCallback,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn InstallCallback_OnProgress(
    mut iface: *mut IBindStatusCallback,
    mut ulProgress: ULONG,
    mut ulProgressMax: ULONG,
    mut ulStatusCode: ULONG,
    mut szStatusText: LPCWSTR,
) -> HRESULT {
    let mut progress: HWND = GetDlgItem(install_dialog, ID_DWL_PROGRESS as _);
    if ulProgressMax != 0 {
        SendMessageW(
            progress,
            PBM_SETRANGE32 as UINT,
            0 as WPARAM,
            ulProgressMax as LPARAM,
        );
    }
    if ulProgress != 0 {
        SendMessageW(
            progress,
            PBM_SETPOS as UINT,
            ulProgress as WPARAM,
            0 as LPARAM,
        );
    }
    return S_OK;
}
unsafe extern "system" fn InstallCallback_OnStopBinding(
    mut iface: *mut IBindStatusCallback,
    mut hresult: HRESULT,
    mut szError: LPCWSTR,
) -> HRESULT {
    EnterCriticalSection(&raw mut csLock);
    if !download_binding.is_null() {
        IBinding_Release(download_binding);
        download_binding = ::core::ptr::null_mut::<IBinding>();
    }
    LeaveCriticalSection(&raw mut csLock);

    if hresult < 0 {
        if hresult == E_ABORT {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/cpl/appwiz/addons.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"InstallCallback_OnStopBinding\0" as *const u8
                        as *const ::core::ffi::c_char,
                    311 as ::core::ffi::c_int,
                    b"Binding aborted\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/cpl/appwiz/addons.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"InstallCallback_OnStopBinding\0" as *const u8 as *const ::core::ffi::c_char,
                313 as ::core::ffi::c_int,
                b"Binding failed %08x\n\0" as *const u8 as *const ::core::ffi::c_char,
                hresult,
            );
        }
        return S_OK;
    }

    set_status(IDS_INSTALLING as DWORD);
    return S_OK;
}
unsafe extern "system" fn InstallCallback_GetBindInfo(
    iface: *mut IBindStatusCallback,
    grfBINDF: *mut DWORD,
    pbindinfo: *mut BINDINFO,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn InstallCallback_OnDataAvailable(
    iface: *mut IBindStatusCallback,
    grfBSCF: DWORD,
    dwSize: DWORD,
    pformatetc: *mut FORMATETC,
    pstgmed: *mut STGMEDIUM,
) -> HRESULT {
    // ERR("\n");
    E_NOTIMPL
}
unsafe extern "system" fn InstallCallback_OnObjectAvailable(
    iface: *mut IBindStatusCallback,
    riid: REFIID,
    punk: *mut IUnknown,
) -> HRESULT {
    // ERR("\n");
    E_NOTIMPL
}

unsafe extern "system" fn download_proc(arg: PVOID) -> DWORD {
    let mut message: [WCHAR; 256] = [0; 256];
    let mut tmp_dir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut tmp_file: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hres: HRESULT;
    let mut hrCoInit: HRESULT;

    hrCoInit = CoInitializeEx(core::ptr::null_mut(), COINIT_MULTITHREADED);

    GetTempPathW(
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        tmp_dir.as_mut_ptr(),
    );
    GetTempFileNameW(
        tmp_dir.as_mut_ptr() as LPCWSTR,
        core::ptr::null(),
        0,
        tmp_file.as_mut_ptr(),
    );

    hres = URLDownloadToFileW(
        core::ptr::null_mut(),
        addr_of_mut!(GeckoUrl) as LPCWSTR,
        tmp_file.as_mut_ptr() as LPCWSTR,
        0,
        core::ptr::addr_of_mut!(InstallCallback),
    );
    if hres < 0 {
        if LoadStringW(
            hApplet,
            IDS_DWL_FAILED,
            message.as_mut_ptr(),
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as i32,
        ) != 0
        {
            /* If the user aborted the download, DO NOT display the message box */
            if hres == E_ABORT {
            } else {
                MessageBoxW(
                    core::ptr::null_mut(),
                    message.as_mut_ptr() as LPCWSTR,
                    core::ptr::null(),
                    MB_ICONERROR,
                );
            }
        }
    } else {
        if sha_check(tmp_file.as_mut_ptr()) != 0 {
            install_file(tmp_file.as_mut_ptr());
        } else {
            if LoadStringW(
                hApplet,
                IDS_INVALID_SHA,
                message.as_mut_ptr(),
                (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as i32,
            ) != 0
            {
                MessageBoxW(
                    core::ptr::null_mut(),
                    message.as_mut_ptr() as LPCWSTR,
                    core::ptr::null(),
                    MB_ICONERROR,
                );
            }
        }
    }

    DeleteFileW(tmp_file.as_mut_ptr() as LPCWSTR);
    PostMessageW(
        install_dialog,
        WM_COMMAND,
        (IDCANCEL) as WPARAM,
        0,
    );

    if hrCoInit >= 0 {
        CoUninitialize();
    }

    return 0;
}

unsafe extern "system" fn installer_proc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hwndProgress: HWND = core::ptr::null_mut();
    let mut hwndInstallButton: HWND = core::ptr::null_mut();
    match msg {
        WM_INITDIALOG => {
            hwndProgress = GetDlgItem(hwnd, (ID_DWL_PROGRESS) as i32);

            /* CORE-5737: Move focus before SW_HIDE */
            if hwndProgress == GetFocus() {
                SendMessageW(hwnd, WM_NEXTDLGCTL, 0 as WPARAM, FALSE as LPARAM);
            }

            ShowWindow(hwndProgress, (SW_HIDE) as i32);
            install_dialog = hwnd;
            return TRUE as INT_PTR;
        }

        WM_NOTIFY => {}

        WM_COMMAND => {
            match wParam {
                x if x == IDCANCEL as WPARAM => {
                    EnterCriticalSection(core::ptr::addr_of_mut!(csLock));
                    if !download_binding.is_null() {
                        IBinding_Abort(download_binding);
                    } else {
                        EndDialog(hwnd, 0 as INT_PTR);
                    }
                    LeaveCriticalSection(core::ptr::addr_of_mut!(csLock));
                    return FALSE as INT_PTR;
                }

                x if x == ID_DWL_INSTALL as WPARAM => {
                    ShowWindow(GetDlgItem(hwnd, (ID_DWL_PROGRESS) as i32), (SW_SHOW) as i32);

                    /* CORE-17550: Never leave focus on a disabled control (Old/New/Thing p.228) */
                    hwndInstallButton = GetDlgItem(hwnd, (ID_DWL_INSTALL) as i32);
                    if hwndInstallButton == GetFocus() {
                        SendMessageW(hwnd, WM_NEXTDLGCTL, 0 as WPARAM, FALSE as LPARAM);
                    }
                    EnableWindow(hwndInstallButton, (FALSE) as BOOL);

                    CloseHandle(CreateThread(
                        core::ptr::null_mut(),
                        0 as DWORD,
                        Some(download_proc),
                        core::ptr::null_mut(),
                        0 as DWORD,
                        core::ptr::null_mut(),
                    ));
                    return FALSE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "C" fn install_addon(addon_type: addon_t, hwnd_parent: HWND) -> BOOL {
    if ARCH_STRING[0] == 0 {
        return (FALSE) as BOOL;
    }
    addon = (&raw const addons_info as *const addon_info_t).offset(addon_type as isize);
    InitializeCriticalSection(core::ptr::addr_of_mut!(csLock));
    /*
     * Try to find addon .msi file in following order:
     * - directory stored in $dir_config_key value of HKCU/Wine/Software/$config_key key
     * - download the package
     */
    if install_from_registered_dir() as ::core::ffi::c_uint == INSTALL_NEXT as ::core::ffi::c_uint {
        DialogBoxW!(hApplet, (*addon).dialog_template, hwnd_parent, Some(installer_proc));
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!(csLock));
    return (TRUE) as BOOL;
}

// COM vtable tables — generated from addons.c by vtbl_gen.py
/// addons.c:341 `static const IBindStatusCallbackVtbl InstallCallbackVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static InstallCallbackVtbl: IBindStatusCallbackVtbl = IBindStatusCallbackVtbl {
    QueryInterface: Some(InstallCallback_QueryInterface),
    AddRef: Some(InstallCallback_AddRef),
    Release: Some(InstallCallback_Release),
    OnStartBinding: Some(InstallCallback_OnStartBinding),
    GetPriority: Some(InstallCallback_GetPriority),
    OnLowResource: Some(InstallCallback_OnLowResource),
    OnProgress: Some(InstallCallback_OnProgress),
    OnStopBinding: Some(InstallCallback_OnStopBinding),
    GetBindInfo: Some(InstallCallback_GetBindInfo),
    OnDataAvailable: Some(InstallCallback_OnDataAvailable),
    OnObjectAvailable: Some(InstallCallback_OnObjectAvailable),
};
