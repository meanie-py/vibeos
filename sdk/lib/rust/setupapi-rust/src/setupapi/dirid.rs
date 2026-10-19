use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut nb_user_dirids: ::core::ffi::c_int = 0;

static mut alloc_user_dirids: ::core::ffi::c_int = 0;

static mut user_dirids: *mut user_dirid = ::core::ptr::null::<user_dirid>() as *mut user_dirid;

static mut system_dirids: [*const WCHAR; 56] = [::core::ptr::null::<WCHAR>(); 56];

static mut csidl_dirids: [*const WCHAR; 64] = [::core::ptr::null::<WCHAR>(); 64];

// DIRID_PRINTPROCESSOR is not defined in this preamble; the value below is a placeholder and must be verified against the DIRID enum in the original headers.

pub const MAX_SYSTEM_DIRID: usize = DIRID_PRINTPROCESSOR as usize;

pub const MIN_CSIDL_DIRID: u32 = 0x4000;

pub const MAX_CSIDL_DIRID: u32 = 0x403f;

#[repr(C)]
pub struct user_dirid {
    pub id: i32,
    pub str: *mut WCHAR,
}

unsafe extern "C" fn get_unknown_dirid() -> *const WCHAR {
    static mut unknown_dirid: *mut WCHAR = core::ptr::null_mut();
    static mut unknown_str: [u16; 9] = [0x005Cu16, 0x0075, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];

    if unknown_dirid.is_null() {
        let mut len: UINT = (GetSystemDirectoryW(core::ptr::null_mut(), 0) as UINT)
            .wrapping_add(strlenW(core::ptr::addr_of!(unknown_str) as *const WCHAR) as UINT);
        unknown_dirid = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if unknown_dirid.is_null() {
            return core::ptr::null();
        }
        GetSystemDirectoryW(unknown_dirid, len);
        strcatW(unknown_dirid, core::ptr::addr_of!(unknown_str) as *const WCHAR);
    }
    return unknown_dirid as *const WCHAR;
}

unsafe extern "C" fn create_system_dirid(dirid: ::core::ffi::c_int) -> *const WCHAR {
    static Null: [WCHAR; 1] = [0];
    static C_Root: [WCHAR; 4] = [0x0043u16, 0x003A, 0x005C, 0x0000];
    static Drivers: [WCHAR; 9] = [0x005Cu16, 0x0064, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0073, 0x0000];
    static Inf: [WCHAR; 5] = [0x005Cu16, 0x0069, 0x006E, 0x0066, 0x0000];
    static Help: [WCHAR; 6] = [0x005Cu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
    static Fonts: [WCHAR; 7] = [0x005Cu16, 0x0066, 0x006F, 0x006E, 0x0074, 0x0073, 0x0000];
    static Viewers: [WCHAR; 9] = [0x005Cu16, 0x0076, 0x0069, 0x0065, 0x0077, 0x0065, 0x0072, 0x0073, 0x0000];
    static System: [WCHAR; 8] = [0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];
    static Spool: [WCHAR; 7] = [0x005Cu16, 0x0073, 0x0070, 0x006F, 0x006F, 0x006C, 0x0000];
    static UserProfile: [WCHAR; 12] = [
    0x0055u16, 0x0053, 0x0045, 0x0052, 0x0050, 0x0052, 0x004F, 0x0046, 0x0049, 0x004C,
    0x0045, 0x0000,
];

    let mut buffer: [WCHAR; (MAX_PATH + 32) as _] =
        [0; (MAX_PATH + 32) as _];
    let mut str: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: ::core::ffi::c_int = 0;
    let mut needed: DWORD = 0;

    'sw: {
        match (dirid) as u32 {
            DIRID_NULL => {
                return core::ptr::addr_of!(Null) as *const WCHAR;
            }
            DIRID_WINDOWS => {
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                break 'sw;
            }
            DIRID_SYSTEM => {
                GetSystemDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                break 'sw;
            }
            DIRID_DRIVERS => {
                GetSystemDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Drivers) as *const WCHAR);
                break 'sw;
            }
            DIRID_INF | 13 => {
                /* DIRID_DRIVER_STORE (Windows 10 1709+): the package's driver store folder. */
                /* VibeOS keeps no per-package store; %windir%\inf, where the package's INF and
                 * files are staged, is the closest equivalent. Modern INFs install
                 * `ServiceBinary = %13%\foo.sys` and load the driver from there. */
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Inf) as *const WCHAR);
                break 'sw;
            }
            DIRID_HELP => {
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Help) as *const WCHAR);
                break 'sw;
            }
            DIRID_FONTS => {
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Fonts) as *const WCHAR);
                break 'sw;
            }
            DIRID_VIEWERS => {
                GetSystemDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Viewers) as *const WCHAR);
                break 'sw;
            }
            DIRID_APPS => {
                return core::ptr::addr_of!(C_Root) as *const WCHAR; /* FIXME */
            }
            DIRID_SHARED => {
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                break 'sw;
            }
            DIRID_BOOT => {
                return core::ptr::addr_of!(C_Root) as *const WCHAR; /* FIXME */
            }
            DIRID_SYSTEM16 => {
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(System) as *const WCHAR);
                break 'sw;
            }
            DIRID_SPOOL | DIRID_SPOOLDRIVERS => {
                /* FIXME */
                GetWindowsDirectoryW(buffer.as_mut_ptr(), MAX_PATH);
                strcatW(buffer.as_mut_ptr(), core::ptr::addr_of!(Spool) as *const WCHAR);
                break 'sw;
            }
            DIRID_USERPROFILE => {
                if GetEnvironmentVariableW(
                    core::ptr::addr_of!(UserProfile) as *const WCHAR,
                    buffer.as_mut_ptr(),
                    MAX_PATH,
                ) != 0
                {
                    break 'sw;
                }
                return get_csidl_dir(CSIDL_PROFILE);
            }
            DIRID_LOADER => {
                return core::ptr::addr_of!(C_Root) as *const WCHAR; /* FIXME */
            }
            DIRID_PRINTPROCESSOR => {
                if GetPrintProcessorDirectoryW(
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    1,
                    buffer.as_mut_ptr() as LPBYTE,
                    core::mem::size_of::<[WCHAR; (MAX_PATH + 32) as _]>() as DWORD,
                    core::ptr::addr_of_mut!(needed),
                ) == 0
                {
                    WARN!("cannot retrieve print processor directory\n");
                    return get_unknown_dirid();
                }
                break 'sw;
            }
            DIRID_COLOR | _ => {
                /* FIXME */
                FIXME!("unknown dirid %d\n", dirid);
                return get_unknown_dirid();
            }
        }
    }
    len = ((strlenW(buffer.as_ptr()) + 1) as usize
        * core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int;
    str = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len) as SIZE_T,
    ) as *mut WCHAR;
    if !str.is_null() {
        memcpy(
            str as *mut ::core::ffi::c_void,
            buffer.as_ptr() as *const ::core::ffi::c_void,
            len as usize,
        );
    }
    return str;
}

unsafe extern "C" fn get_csidl_dir(csidl: DWORD) -> *const WCHAR {
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut str: *mut WCHAR = core::ptr::null_mut();
    let mut len: core::ffi::c_int;

    if SHGetSpecialFolderPathW(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        csidl as core::ffi::c_int,
        (TRUE) as BOOL,
    ) == 0
    {
        return get_unknown_dirid();
    }
    len = ((strlenW(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as usize + 1)
        * core::mem::size_of::<WCHAR>()) as core::ffi::c_int;
    str = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        len as SIZE_T,
    ) as *mut WCHAR;
    if !str.is_null() {
        memcpy(
            str as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR as *const core::ffi::c_void,
            len as usize,
        );
    }
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn DIRID_get_string(mut dirid: ::core::ffi::c_int) -> *const WCHAR {
    let mut i: ::core::ffi::c_int = 0;
    if dirid == DIRID_ABSOLUTE || dirid == (DIRID_ABSOLUTE_16BIT as i32) {
        dirid = (DIRID_NULL as i32);
    }
    if dirid >= (DIRID_USER as i32) {
        i = 0;
        while i < nb_user_dirids {
            if (*user_dirids.offset(i as isize)).id == dirid {
                return (*user_dirids.offset(i as isize)).str;
            }
            i += 1;
        }
        WARN!("user id %d not found\n", dirid);
        return core::ptr::null::<WCHAR>();
    } else if dirid >= (MIN_CSIDL_DIRID as i32) {
        if dirid > (MAX_CSIDL_DIRID as i32) {
            return get_unknown_dirid();
        }
        dirid -= (MIN_CSIDL_DIRID as i32);
        if csidl_dirids[dirid as usize].is_null() {
            csidl_dirids[dirid as usize] = get_csidl_dir(dirid as DWORD) as *const WCHAR;
        }
        return csidl_dirids[dirid as usize];
    } else {
        // Divergence from the C: it indexes system_dirids[dirid] for ANY dirid not above
        // MAX_SYSTEM_DIRID, so a negative one (other than DIRID_ABSOLUTE, mapped above; an
        // INF's `%-5%` reaches here through strtolW) reads and writes out of bounds. That
        // is a panic in Rust, so a negative dirid is unknown, like one past the table.
        if dirid > MAX_SYSTEM_DIRID as i32 || dirid < 0 {
            return get_unknown_dirid();
        }
        if system_dirids[dirid as usize].is_null() {
            system_dirids[dirid as usize] = create_system_dirid(dirid) as *const WCHAR;
        }
        return system_dirids[dirid as usize];
    };
}

unsafe extern "C" fn store_user_dirid(mut hinf: HINF, mut id: ::core::ffi::c_int, mut str: *mut WCHAR) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < nb_user_dirids {
        if (*user_dirids.offset(i as isize)).id == id {
            break;
        }
        i += 1;
    }
    if i < nb_user_dirids {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*user_dirids.offset(i as isize)).str as PVOID,
        );
    } else {
        if nb_user_dirids >= alloc_user_dirids {
            let mut new_size: ::core::ffi::c_int = max(
                32 as ::core::ffi::c_int,
                alloc_user_dirids * 2 as ::core::ffi::c_int,
            );
            let mut new: *mut user_dirid = ::core::ptr::null_mut::<user_dirid>();
            if !user_dirids.is_null() {
                new = HeapReAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    user_dirids as PVOID,
                    (new_size as SIZE_T).wrapping_mul(::core::mem::size_of::<user_dirid>() as SIZE_T),
                ) as *mut user_dirid;
            } else {
                new = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (new_size as SIZE_T).wrapping_mul(::core::mem::size_of::<user_dirid>() as SIZE_T),
                ) as *mut user_dirid;
            }
            if new.is_null() {
                return (FALSE) as BOOL;
            }
            user_dirids = new;
            alloc_user_dirids = new_size;
        }
        nb_user_dirids += 1;
    }
    (*user_dirids.offset(i as isize)).id = id;
    let ref mut fresh15 = (*user_dirids.offset(i as isize)).str;
    *fresh15 = str;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/dirid.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"store_user_dirid\0" as *const u8 as *const ::core::ffi::c_char,
            222 as ::core::ffi::c_int,
            b"id %d -> %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            id,
            debugstr_w(str),
        );
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetDirectoryIdA(
    hinf: HINF,
    id: DWORD,
    dir: PCSTR,
) -> BOOL {
    let mut dirW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut i: core::ffi::c_int = 0;

    if id == 0 {
        /* clear everything */
        i = 0;
        while i < nb_user_dirids {
            HeapFree(
                GetProcessHeap(),
                0,
                (*user_dirids.offset(i as isize)).str as PVOID,
            );
            i += 1;
        }
        nb_user_dirids = 0;
        return (TRUE) as BOOL;
    }
    if id < DIRID_USER {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* duplicate the string */
    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(dirW), dir) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }
    return store_user_dirid(hinf, id as core::ffi::c_int, dirW.Buffer as *mut WCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetDirectoryIdW(hinf: HINF, id: DWORD, dir: PCWSTR) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut str: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

    if id == 0 {
        /* clear everything */
        i = 0;
        while i < nb_user_dirids {
            HeapFree(
                GetProcessHeap(),
                0,
                (*user_dirids.offset(i as isize)).str as PVOID,
            );
            i += 1;
        }
        nb_user_dirids = 0;
        return (TRUE) as BOOL;
    }
    if id < DIRID_USER as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    /* duplicate the string */
    len = ((strlenW(dir) + 1) as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int;
    str = HeapAlloc(
        GetProcessHeap(),
        0,
        len as SIZE_T,
    ) as *mut WCHAR;
    if str.is_null() {
        return (FALSE) as BOOL;
    }
    memcpy(
        str as *mut ::core::ffi::c_void,
        dir as *const ::core::ffi::c_void,
        len as usize,
    );
    return store_user_dirid(hinf, id as ::core::ffi::c_int, str);
}
