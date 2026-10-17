use crate::reg::copy::key;
use crate::reg::add::reg_add;
use crate::reg::copy::reg_copy;
use crate::reg::delete::reg_delete;
use crate::reg::export::reg_export;
use crate::reg::import::reg_import;
use crate::reg::query::reg_query;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_08AFF42C5A62: [u16; 6] = [0x0071u16, 0x0075, 0x0065, 0x0072, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_08F79C40983C: [u16; 7] = [0x0064u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_09C488271038: [u16; 4] = [0x0061u16, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_377C04EA0393: [u16; 5] = [0x0063u16, 0x006F, 0x0070, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_E92602BF937C: [u16; 7] = [0x0065u16, 0x0078, 0x0070, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_ED9215882D4E: [u16; 7] = [0x0069u16, 0x006D, 0x0070, 0x006F, 0x0072, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_010340BC5B61: [u16; 11] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0042, 0x0049, 0x004E, 0x0041, 0x0052, 0x0059,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_111FD57AB4CB: [u16; 20] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0043, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_22559A0A7652: [u16; 13] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x004D, 0x0055, 0x004C, 0x0054, 0x0049, 0x005F,
    0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_22FF61FCDE09: [u16; 4] = [0x0048u16, 0x004B, 0x0055, 0x0000];
pub static RUSTCONV_C2R_WSTR_259A60C8EC82: [u16; 24] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x005F,
    0x004C, 0x0049, 0x0054, 0x0054, 0x004C, 0x0045, 0x005F, 0x0045, 0x004E, 0x0044,
    0x0049, 0x0041, 0x004E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_30BC907EFB89: [u16; 7] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0053, 0x005A, 0x0000];
pub static RUSTCONV_C2R_WSTR_3B3A24BC138B: [u16; 10] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x0000];
pub static RUSTCONV_C2R_WSTR_4D71FACE03C3: [u16; 14] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0045, 0x0058, 0x0050, 0x0041, 0x004E, 0x0044,
    0x005F, 0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_50C930882486: [u16; 19] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x004C, 0x004F, 0x0043, 0x0041, 0x004C,
    0x005F, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_87D16031626E: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_89F38F0B7FC9: [u16; 5] = [0x0048u16, 0x004B, 0x004C, 0x004D, 0x0000];
pub static RUSTCONV_C2R_WSTR_8E490F9323D5: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0052, 0x0000];
pub static RUSTCONV_C2R_WSTR_9C98935C198E: [u16; 11] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0053,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_BFE074CE3641: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0043, 0x0000];
pub static RUSTCONV_C2R_WSTR_D02D0BCFE5D4: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0055, 0x0000];
pub static RUSTCONV_C2R_WSTR_D0EE0F231465: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D8DCD81B58B7: [u16; 21] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x005F,
    0x0042, 0x0049, 0x0047, 0x005F, 0x0045, 0x004E, 0x0044, 0x0049, 0x0041, 0x004E,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_F43A50ADA987: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x004E, 0x004F, 0x004E, 0x0045, 0x0000];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub key: HKEY,
    pub short_name: *const WCHAR,
    pub long_name: *const WCHAR,
}

static mut root_rels: [C2RustUnnamed_11; 5] = unsafe {
    [
        C2RustUnnamed_11 {
            key: HKEY_LOCAL_MACHINE,
            short_name: RUSTCONV_C2R_WSTR_89F38F0B7FC9.as_ptr() as *const WCHAR,
            long_name: RUSTCONV_C2R_WSTR_50C930882486.as_ptr() as *const WCHAR,
        },
        C2RustUnnamed_11 {
            key: HKEY_CURRENT_USER,
            short_name: RUSTCONV_C2R_WSTR_D02D0BCFE5D4.as_ptr() as *const WCHAR,
            long_name: RUSTCONV_C2R_WSTR_D0EE0F231465.as_ptr() as *const WCHAR,
        },
        C2RustUnnamed_11 {
            key: HKEY_CLASSES_ROOT,
            short_name: RUSTCONV_C2R_WSTR_8E490F9323D5.as_ptr() as *const WCHAR,
            long_name: RUSTCONV_C2R_WSTR_87D16031626E.as_ptr() as *const WCHAR,
        },
        C2RustUnnamed_11 {
            key: HKEY_USERS,
            short_name: RUSTCONV_C2R_WSTR_22FF61FCDE09.as_ptr() as *const WCHAR,
            long_name: RUSTCONV_C2R_WSTR_9C98935C198E.as_ptr() as *const WCHAR,
        },
        C2RustUnnamed_11 {
            key: HKEY_CURRENT_CONFIG,
            short_name: RUSTCONV_C2R_WSTR_BFE074CE3641.as_ptr() as *const WCHAR,
            long_name: RUSTCONV_C2R_WSTR_111FD57AB4CB.as_ptr() as *const WCHAR,
        },
    ]
};

#[no_mangle]
pub static mut type_rels: [reg_type_rels; 8] = unsafe {
    [
        reg_type_rels {
            type_: REG_NONE as DWORD,
            name: RUSTCONV_C2R_WSTR_F43A50ADA987.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_SZ as DWORD,
            name: RUSTCONV_C2R_WSTR_30BC907EFB89.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_EXPAND_SZ as DWORD,
            name: RUSTCONV_C2R_WSTR_4D71FACE03C3.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_BINARY as DWORD,
            name: RUSTCONV_C2R_WSTR_010340BC5B61.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_DWORD as DWORD,
            name: RUSTCONV_C2R_WSTR_3B3A24BC138B.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_DWORD_LITTLE_ENDIAN as DWORD,
            name: RUSTCONV_C2R_WSTR_259A60C8EC82.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_DWORD_BIG_ENDIAN as DWORD,
            name: RUSTCONV_C2R_WSTR_D8DCD81B58B7.as_ptr() as *const WCHAR,
        },
        reg_type_rels {
            type_: REG_MULTI_SZ as DWORD,
            name: RUSTCONV_C2R_WSTR_22559A0A7652.as_ptr() as *const WCHAR,
        },
    ]
};

// WINE_DEFAULT_DEBUG_CHANNEL(reg): debug channel declaration, handled by the DPRINT1!/WARN!/ERR! macros.

#[repr(C)]
pub struct reg_root_rel {
    pub key: HKEY,
    pub short_name: *const u16,
    pub long_name: *const u16,
}

pub const REG_ADD: i32 = 0;
pub const REG_COPY: i32 = 1;
pub const REG_DELETE: i32 = 2;
pub const REG_EXPORT: i32 = 3;
pub const REG_IMPORT: i32 = 4;
pub const REG_QUERY: i32 = 5;
pub const REG_INVALID: i32 = 6;

// HKEY root constants (Win32 values)
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_CONFIG: HKEY = 0x8000_0005u32 as i32 as isize as HKEY;
#[no_mangle]
pub unsafe extern "C" fn output_writeconsole(str: *const WCHAR, wlen: DWORD) {
    let mut count: DWORD = 0;
    let mut ret: DWORD = 0;
    ret = WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        str as *const ::core::ffi::c_void,
        wlen,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) as DWORD;
    if ret == 0 {
        let mut len: DWORD = 0;
        let mut msgA: *mut ::core::ffi::c_char = core::ptr::null_mut();
        len = WideCharToMultiByte(
            GetConsoleOutputCP(),
            0,
            str as LPCWSTR,
            wlen as ::core::ffi::c_int,
            core::ptr::null_mut(),
            0 as ::core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as DWORD;
        msgA = malloc(len as usize) as *mut ::core::ffi::c_char;

        WideCharToMultiByte(
            GetConsoleOutputCP(),
            0,
            str as LPCWSTR,
            wlen as ::core::ffi::c_int,
            msgA as LPSTR,
            len as ::core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            msgA as LPCVOID,
            len,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(), // C passes FALSE for the LPOVERLAPPED
        );
        free(msgA as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C" fn output_formatstring(mut fmt: *const WCHAR, mut va_args: *mut ::core::ffi::c_void) {
    let mut str: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: DWORD = 0;
    len = FormatMessageW(
        (FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ALLOCATE_BUFFER) as DWORD,
        fmt as LPCVOID,
        0 as DWORD,
        0 as DWORD,
        &raw mut str as LPWSTR,
        0 as DWORD,
        (&raw mut va_args).cast(),
    );
    if len == 0 as DWORD
        && GetLastError() != NO_ERROR as DWORD
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            WINE_FIXME!(
                b"Could not format string: le=%u, fmt=%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                GetLastError(),
                wine_dbgstr_w(fmt),
            );
        }
        return;
    }
    output_writeconsole(str, len);
    LocalFree(str as HLOCAL);
}

#[no_mangle]
pub unsafe extern "C" fn output_message_v(mut id: ::core::ffi::c_uint, ap_list: *mut ::core::ffi::c_void) {
    let mut fmt: *mut WCHAR = core::ptr::null_mut();
    let mut len: ::core::ffi::c_int;
    let mut va_args: va_list;

    len = LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HMODULE,
        id as UINT,
        core::ptr::addr_of_mut!(fmt) as *mut WCHAR,
        0,
    );
    if len == 0 {
        WINE_FIXME!(
            b"LoadString failed with %d\n\0".as_ptr() as *const ::core::ffi::c_char,
            GetLastError(),
        );
        return;
    }

    len += 1;
    fmt = malloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as *mut WCHAR;
    if fmt.is_null() {
        return;
    }

    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HMODULE,
        id as UINT,
        fmt,
        len,
    );

    // va_start(va_args, id);
    va_args = *(ap_list as *mut va_list);
    output_formatstring(fmt, (va_args).cast());
    // va_end(va_args);

    free(fmt as *mut ::core::ffi::c_void);
}


#[no_mangle]
pub unsafe extern "C" fn output_string_v(fmt: *const WCHAR, ap_list: *mut ::core::ffi::c_void) {
    let mut va_args: va_list = *(ap_list as *mut va_list);
    output_formatstring(fmt, (va_args).cast());
}


#[no_mangle]
pub unsafe extern "C" fn ask_confirm(msgid: ::core::ffi::c_uint, reg_info: *mut WCHAR) -> BOOL {
    let mut hmod: HMODULE = core::ptr::null_mut();
    let mut Ybuffer: [WCHAR; 4] = [0; 4];
    let mut Nbuffer: [WCHAR; 4] = [0; 4];
    let mut defval: [WCHAR; 32] = [0; 32];
    let mut answer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut str: *mut WCHAR = core::ptr::null_mut();
    let mut count: DWORD = 0;

    hmod = GetModuleHandleW(core::ptr::null());
    LoadStringW(
        hmod as HINSTANCE,
        STRING_YES as UINT,
        core::ptr::addr_of_mut!(Ybuffer) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(Ybuffer) as ::core::ffi::c_int,
    );
    LoadStringW(
        hmod as HINSTANCE,
        STRING_NO as UINT,
        core::ptr::addr_of_mut!(Nbuffer) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(Nbuffer) as ::core::ffi::c_int,
    );
    LoadStringW(
        hmod as HINSTANCE,
        STRING_DEFAULT_VALUE as UINT,
        core::ptr::addr_of_mut!(defval) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(defval) as ::core::ffi::c_int,
    );

    str = if !reg_info.is_null() && *reg_info != 0 {
        reg_info
    } else {
        core::ptr::addr_of_mut!(defval) as *mut WCHAR
    };

    while 1 != 0 {
        output_message(msgid, str);
        output_message(STRING_YESNO as ::core::ffi::c_uint);
        ReadConsoleW(
            GetStdHandle(STD_INPUT_HANDLE),
            core::ptr::addr_of_mut!(answer) as *mut WCHAR as LPVOID,
            ARRAY_SIZE!(answer) as DWORD,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        );
        answer[0] =
            towupper(answer[0] as _) as WCHAR;
        if answer[0] == Ybuffer[0] {
            return (TRUE) as BOOL;
        }
        if answer[0] == Nbuffer[0] {
            return (FALSE) as BOOL;
        }
    }
    0
}

unsafe extern "C" fn path_rootname_cmp(input_path: *const WCHAR, rootkey_name: *const WCHAR) -> BOOL {
    let mut length: DWORD = lstrlenW(rootkey_name) as DWORD;

    return (!(_wcsnicmp(input_path, rootkey_name, length as usize) != 0)
        && (*input_path.add(length as usize) == 0
            || *input_path.add(length as usize) == '\\' as WCHAR)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn path_get_rootkey(path: *const WCHAR) -> HKEY {
    let mut i: DWORD = 0;
    while (i as usize) < ARRAY_SIZE!(root_rels) as usize {
        if path_rootname_cmp(path, root_rels[i as usize].short_name) != 0
            || path_rootname_cmp(path, root_rels[i as usize].long_name) != 0
        {
            return root_rels[i as usize].key;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn sane_path(key: *const WCHAR) -> BOOL {
    let mut i: ::core::ffi::c_uint = lstrlenW(key as LPCWSTR) as ::core::ffi::c_uint;

    if i < 3 as ::core::ffi::c_uint
        || (*key.offset((i - 1 as ::core::ffi::c_uint) as isize) == '\\' as u16
            && *key.offset((i - 2 as ::core::ffi::c_uint) as isize) == '\\' as u16)
    {
        output_message(STRING_INVALID_KEY as ::core::ffi::c_uint);
        return FALSE as BOOL;
    }

    if *key.offset(0 as isize) == '\\' as u16
        && *key.offset(1 as isize) == '\\' as u16
        && *key.offset(2 as isize) != '\\' as u16
    {
        output_message(STRING_NO_REMOTE as ::core::ffi::c_uint);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn build_subkey_path(path: *mut WCHAR, path_len: DWORD, subkey_name: *mut WCHAR, subkey_len: DWORD) -> *mut WCHAR {
    let mut subkey_path: *mut WCHAR;

    subkey_path = malloc(
        (path_len.wrapping_add(subkey_len).wrapping_add(2) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    swprintf(subkey_path, RUSTCONV_WSTR_BA2342D760A2.as_ptr(), path, subkey_name);

    return subkey_path;
}
#[no_mangle]
pub unsafe extern "C" fn get_long_key(root: HKEY, path: *mut WCHAR) -> *mut WCHAR {
    let mut i: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int;
    let mut long_key: *mut WCHAR;

    while i < (ARRAY_SIZE!(root_rels) as ::core::ffi::c_int) {
        if root == root_rels[i as usize].key {
            break;
        }
        i += 1;
    }

    len = lstrlenW(root_rels[i as usize].long_name as LPCWSTR);

    if path.is_null() {
        long_key = malloc(
            ((len + 1) as usize) * ::core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        lstrcpyW(
            long_key as LPWSTR,
            root_rels[i as usize].long_name as LPCWSTR,
        );
        return long_key;
    }

    len += lstrlenW(path as LPCWSTR) + 1;
    long_key = malloc(
        ((len + 1) as usize) * ::core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    swprintf(
        long_key,
        RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
        root_rels[i as usize].long_name,
        path,
    );
    return long_key;
}
#[no_mangle]
pub unsafe extern "C" fn parse_registry_key(key: *const WCHAR, root: *mut HKEY, path: *mut *mut WCHAR) -> BOOL {
    let mut p: *mut WCHAR;
    if sane_path(key) == 0 {
        return (FALSE) as BOOL;
    }
    *root = path_get_rootkey(key);
    if (*root).is_null() {
        output_message(STRING_INVALID_SYSTEM_KEY as ::core::ffi::c_uint);
        return (FALSE) as BOOL;
    }
    *path = wcschr(key as *const WCHAR, '\\' as i32 as WCHAR) as *mut WCHAR;
    if (*path).is_null() {
        return (TRUE) as BOOL;
    }
    *path = (*path).offset(1);
    if **path == 0 {
        output_message(STRING_INVALID_SYSTEM_KEY as ::core::ffi::c_uint);
        return (FALSE) as BOOL;
    }
    p = (*path)
        .offset(lstrlenW(*path as *const WCHAR) as isize)
        .offset(-(1 as isize));
    if *p as ::core::ffi::c_int == '\\' as i32 {
        *p = 0 as WCHAR;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn is_char(s: WCHAR, c: WCHAR) -> BOOL {
    return (s as ::core::ffi::c_int == c as ::core::ffi::c_int
        || s as ::core::ffi::c_int == towupper((c as wint_t) as _) as ::core::ffi::c_int)
        as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn is_switch(s: *const WCHAR, c: WCHAR) -> BOOL {
    if lstrlenW(s) > 2 {
        return (FALSE) as BOOL;
    }

    return (((*s.offset(0 as isize)) == '/' as WCHAR
        || (*s.offset(0 as isize)) == '-' as WCHAR)
        && is_char(*s.offset(1 as isize), c) != 0) as BOOL;
}

unsafe extern "C" fn is_help_switch(s: *const WCHAR) -> BOOL {
    return (is_switch(s, '?' as u16) != 0 || is_switch(s, 'h' as u16) != 0) as BOOL;
}

unsafe extern "C" fn get_operation(str: *const u16, op_help: *mut i32) -> operations {
    struct op_info {
        op: *const u16,
        id: i32,
        help_id: i32,
    }

    static mut op_array: [op_info; 7] = [
        op_info {
            op: RUSTCONV_WSTR_09C488271038.as_ptr(),
            id: REG_ADD,
            help_id: (STRING_ADD_USAGE as i32),
        },
        op_info {
            op: RUSTCONV_WSTR_377C04EA0393.as_ptr(),
            id: REG_COPY,
            help_id: (STRING_COPY_USAGE as i32),
        },
        op_info {
            op: RUSTCONV_WSTR_08F79C40983C.as_ptr(),
            id: REG_DELETE,
            help_id: (STRING_DELETE_USAGE as i32),
        },
        op_info {
            op: RUSTCONV_WSTR_E92602BF937C.as_ptr(),
            id: REG_EXPORT,
            help_id: (STRING_EXPORT_USAGE as i32),
        },
        op_info {
            op: RUSTCONV_WSTR_ED9215882D4E.as_ptr(),
            id: REG_IMPORT,
            help_id: (STRING_IMPORT_USAGE as i32),
        },
        op_info {
            op: RUSTCONV_WSTR_08AFF42C5A62.as_ptr(),
            id: REG_QUERY,
            help_id: (STRING_QUERY_USAGE as i32),
        },
        op_info {
            op: core::ptr::null::<u16>(),
            id: -1,
            help_id: 0,
        },
    ];

    let mut ptr: *const op_info = core::ptr::null::<op_info>();
    ptr = core::ptr::addr_of!(op_array) as *const op_info;
    while !(*ptr).op.is_null() {
        if lstrcmpiW(str, (*ptr).op) == 0 {
            *op_help = (*ptr).help_id;
            return (*ptr).id as operations;
        }
        ptr = ptr.offset(1);
    }

    return REG_INVALID as operations;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: i32, mut argvW: *mut *mut WCHAR) -> i32 {
    let mut op: i32 = 0;
    let mut op_help: i32 = 0;

    'invalid: {
        if argc == 1 {
            output_message(STRING_INVALID_SYNTAX);
            output_message(STRING_REG_HELP);
            return 1;
        }

        if is_help_switch(*argvW.offset(1 as isize)) != 0 {
            output_message(STRING_USAGE);
            return 0;
        }

        op = get_operation(
            *argvW.offset(1 as isize),
            core::ptr::addr_of_mut!(op_help),
        ) as i32;

        if op == REG_INVALID {
            output_message(
                STRING_INVALID_OPTION,
                *argvW.offset(1 as isize),
            );
            output_message(STRING_REG_HELP);
            return 1;
        } else if argc == 2 {
            /* Valid operation, no arguments supplied */
            break 'invalid;
        }

        if is_help_switch(*argvW.offset(2 as isize)) != 0 {
            if argc > 3 {
                break 'invalid;
            }

            output_message((op_help) as u32);
            output_message(STRING_REG_VIEW_USAGE);
            return 0;
        }

        if op == REG_ADD {
            return reg_add(argc, argvW);
        }

        if op == REG_COPY {
            return reg_copy(argc, argvW);
        }

        if op == REG_DELETE {
            return reg_delete(argc, argvW);
        }

        if op == REG_EXPORT {
            return reg_export(argc, argvW);
        }

        if op == REG_IMPORT {
            return reg_import(argc, argvW);
        }

        return reg_query(argc, argvW);
    }

    /* invalid: */
    output_message(STRING_INVALID_SYNTAX);
    output_message(
        STRING_FUNC_HELP,
        _wcsupr(*argvW.offset(1 as isize)),
    );
    return 1;
}
