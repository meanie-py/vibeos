use crate::reg::reg::build_subkey_path;
use crate::reg::reg::get_long_key;
use crate::reg::reg::parse_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05645F4502BA: [u16; 3] = [0x0076u16, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_1B2D5DD81FC6: [u16; 7] = [0x0025u16, 0x0031, 0x005C, 0x0025, 0x0032, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_1E8D48188E3D: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_39B57A5A9F48: [u16; 4] = [0x0025u16, 0x0031, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_E2644765DD37: [u16; 5] = [0x0030u16, 0x0078, 0x0025, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_EEFB72B6348E: [u16; 7] = [0x0020u16, 0x0020, 0x0020, 0x0020, 0x0025, 0x0031, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];

static mut newlineW: *const WCHAR = unsafe {
    RUSTCONV_C2R_WSTR_075DE2B906DB.as_ptr()
        as *const WCHAR
};

static mut num_values_found: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

static mut sam: REGSAM = 0 as REGSAM;

unsafe extern "C" fn reg_type_to_wchar(mut type_0: DWORD) -> *const WCHAR {
    let mut i: ::core::ffi::c_int = 0;
    let mut array_size: ::core::ffi::c_int = ARRAY_SIZE!(type_rels) as ::core::ffi::c_int;
    i = 0;
    while i < array_size {
        if type_0 == type_rels[i as usize].type_ {
            return type_rels[i as usize].name;
        }
        i += 1;
    }
    return ::core::ptr::null::<WCHAR>();
}

unsafe extern "C" fn reg_data_to_wchar(
    mut type_0: DWORD,
    mut src: *const BYTE,
    mut size_bytes: DWORD,
) -> *mut WCHAR {
    let mut buffer: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut i: ::core::ffi::c_int = 0;
    match type_0 {
        REG_SZ | REG_EXPAND_SZ => {
            buffer = malloc(size_bytes as size_t) as *mut WCHAR;
            lstrcpyW(buffer as LPWSTR, src as *mut WCHAR as LPCWSTR);
        }
        REG_NONE | REG_BINARY => {
            let mut ptr: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
            buffer = malloc(
                (size_bytes.wrapping_mul(2 as DWORD).wrapping_add(1 as DWORD) as size_t)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
            ) as *mut WCHAR;
            if size_bytes == 0 as DWORD {
                *buffer = 0 as WCHAR;
            } else {
                ptr = buffer;
                i = 0 as ::core::ffi::c_int;
                while (i as DWORD) < size_bytes {
                    ptr = ptr.offset(swprintf(
                        ptr as *mut wchar_t,
                        RUSTCONV_WSTR_1E8D48188E3D.as_ptr(),
                        *src.offset(i as isize) as ::core::ffi::c_int,
                    ) as isize);
                    i += 1;
                }
            }
        }
        REG_DWORD | REG_DWORD_BIG_ENDIAN => {
            let zero_x_dword: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
            buffer = malloc(
                ((zero_x_dword + 1 as ::core::ffi::c_int) as size_t)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
            ) as *mut WCHAR;
            swprintf(
                buffer as *mut wchar_t,
                RUSTCONV_WSTR_E2644765DD37.as_ptr(),
                *(src as *mut DWORD),
            );
        }
        REG_MULTI_SZ => {
            let two_wchars: ::core::ffi::c_int = (2 as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_int;
            let mut tmp_size: DWORD = 0;
            let mut tmp: *const WCHAR = src as *const WCHAR;
            let mut len: ::core::ffi::c_int = 0;
            let mut destindex: ::core::ffi::c_int = 0;
            if size_bytes <= two_wchars as DWORD {
                buffer = malloc(::core::mem::size_of::<WCHAR>() as size_t) as *mut WCHAR;
                *buffer = 0 as WCHAR;
                return buffer;
            }
            tmp_size = size_bytes.wrapping_sub(two_wchars as DWORD);
            buffer = malloc(
                (tmp_size.wrapping_mul(2 as DWORD) as size_t)
                    .wrapping_add(::core::mem::size_of::<WCHAR>() as size_t),
            ) as *mut WCHAR;
            len = (tmp_size as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_int;
            i = 0 as ::core::ffi::c_int;
            destindex = 0 as ::core::ffi::c_int;
            while i < len {
                if *tmp.offset(i as isize) != 0 {
                    *buffer.offset(destindex as isize) = *tmp.offset(i as isize);
                } else {
                    let fresh15 = destindex;
                    destindex = destindex + 1;
                    *buffer.offset(fresh15 as isize) = '\\' as i32 as WCHAR;
                    *buffer.offset(destindex as isize) = '0' as i32 as WCHAR;
                }
                i += 1;
                destindex += 1;
            }
            *buffer.offset(destindex as isize) = 0 as WCHAR;
        }
        _ => {}
    }
    return buffer;
}

unsafe extern "C" fn output_value(value_name: *const WCHAR, type_0: DWORD, data: *mut BYTE, data_size: DWORD) {
    static mut fmt: *const WCHAR = RUSTCONV_WSTR_EEFB72B6348E.as_ptr();
    let mut defval: [WCHAR; 32] = [0 as WCHAR; 32];
    let mut reg_data: *mut WCHAR = core::ptr::null_mut();

    if !value_name.is_null() && *value_name.offset(0 as isize) != 0 {
        output_string(fmt, value_name);
    } else {
        LoadStringW(
            GetModuleHandleW(core::ptr::null()) as HINSTANCE,
            STRING_DEFAULT_VALUE as UINT,
            &raw mut defval as LPWSTR,
            ARRAY_SIZE!(defval) as ::core::ffi::c_int,
        );
        output_string(fmt, &raw mut defval as *mut WCHAR);
    }
    output_string(fmt, reg_type_to_wchar(type_0));

    if !data.is_null() {
        reg_data = reg_data_to_wchar(type_0, data, data_size);
        output_string(fmt, reg_data);
        free(reg_data as *mut ::core::ffi::c_void);
    } else {
        LoadStringW(
            GetModuleHandleW(core::ptr::null()) as HINSTANCE,
            STRING_VALUE_NOT_SET as UINT,
            &raw mut defval as LPWSTR,
            ARRAY_SIZE!(defval) as ::core::ffi::c_int,
        );
        output_string(fmt, &raw mut defval as *mut WCHAR);
    }
    output_string(newlineW);
}

unsafe extern "C" fn query_value(
    mut hkey: HKEY,
    mut value_name: *mut WCHAR,
    mut path: *mut WCHAR,
    mut recurse: BOOL,
) -> ::core::ffi::c_int {
    let mut rc: LONG = 0;
    let mut max_data_bytes: DWORD = 2048 as DWORD;
    let mut data_size: DWORD = 0;
    let mut subkey_len: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut path_len: DWORD = 0;
    let mut i: DWORD = 0;
    let mut data: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    static mut fmt: *const WCHAR = RUSTCONV_WSTR_39B57A5A9F48.as_ptr();
    let mut subkey_name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut subkey_path: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut subkey: HKEY = ::core::ptr::null_mut::<HKEY__>();

    data = malloc(max_data_bytes as size_t) as *mut BYTE;

    loop {
        data_size = max_data_bytes;
        rc = RegQueryValueExW(
            hkey,
            value_name as LPCWSTR,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut type_0,
            data as LPBYTE,
            &raw mut data_size,
        ) as LONG;
        if !(rc == ERROR_MORE_DATA as LONG) {
            break;
        }
        max_data_bytes = data_size;
        data = realloc(data as *mut ::core::ffi::c_void, max_data_bytes as size_t) as *mut BYTE;
    }

    if rc == ERROR_SUCCESS as LONG {
        output_string(fmt, path);
        output_value(value_name, type_0, data, data_size);
        output_string(newlineW);
        num_values_found = num_values_found.wrapping_add(1);
    }

    free(data as *mut ::core::ffi::c_void);

    if recurse == 0 {
        if rc == ERROR_FILE_NOT_FOUND as LONG {
            if !value_name.is_null() && *value_name as ::core::ffi::c_int != 0 {
                output_message(STRING_VALUE_NONEXIST as ::core::ffi::c_uint);
                return 1 as ::core::ffi::c_int;
            }
            output_string(fmt, path);
            output_value(
                ::core::ptr::null::<WCHAR>(),
                REG_SZ as DWORD,
                ::core::ptr::null_mut::<BYTE>(),
                0 as DWORD,
            );
        }
        return 0 as ::core::ffi::c_int;
    }

    subkey_name = malloc(
        (MAX_SUBKEY_LEN as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    path_len = lstrlenW(path as LPCWSTR) as DWORD;

    i = 0 as DWORD;
    loop {
        subkey_len = MAX_SUBKEY_LEN as DWORD;
        rc = RegEnumKeyExW(
            hkey,
            i,
            subkey_name as LPWSTR,
            &raw mut subkey_len,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) as LONG;
        if !(rc == ERROR_SUCCESS as LONG) {
            break;
        }
        subkey_path = build_subkey_path(path, path_len, subkey_name, subkey_len);
        if RegOpenKeyExW(
            hkey,
            subkey_name as LPCWSTR,
            0 as DWORD,
            KEY_READ as REGSAM | sam,
            &raw mut subkey,
        ) == 0
        {
            query_value(subkey, value_name, subkey_path, recurse);
            RegCloseKey(subkey);
        }
        free(subkey_path as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }

    free(subkey_name as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn query_all(
    mut hkey: HKEY,
    mut path: *mut WCHAR,
    mut recurse: BOOL,
    mut recursing: BOOL,
) -> ::core::ffi::c_int {
    let mut rc: LONG = 0;
    let mut num_subkeys: DWORD = 0;
    let mut num_values: DWORD = 0;
    let mut max_value_len: DWORD = 256;
    let mut value_len: DWORD = 0;
    let mut max_data_bytes: DWORD = 2048;
    let mut data_size: DWORD = 0;
    let mut subkey_len: DWORD = 0;
    let mut i: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut path_len: DWORD = 0;
    let mut value_name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut subkey_name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut subkey_path: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut data: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut subkey: HKEY = ::core::ptr::null_mut::<HKEY__>();

    rc = RegQueryInfoKeyW(
        hkey,
        ::core::ptr::null_mut::<WCHAR>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        &raw mut num_subkeys,
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        &raw mut num_values,
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<_FILETIME>(),
    ) as LONG;
    if rc != 0 {
        return 1 as ::core::ffi::c_int;
    }

    if num_values != 0 || recursing != 0 {
        output_string(RUSTCONV_WSTR_39B57A5A9F48.as_ptr(), path);
    }

    value_name = malloc(
        (max_value_len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    data = malloc(max_data_bytes as size_t) as *mut BYTE;

    i = 0;
    loop {
        value_len = max_value_len;
        data_size = max_data_bytes;
        rc = RegEnumValueW(
            hkey,
            i,
            value_name as LPWSTR,
            &raw mut value_len,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut type_0,
            data as LPBYTE,
            &raw mut data_size,
        ) as LONG;
        if rc == ERROR_SUCCESS as LONG {
            output_value(value_name, type_0, data, data_size);
            i = i.wrapping_add(1);
        } else if rc == ERROR_MORE_DATA as LONG {
            if data_size > max_data_bytes {
                max_data_bytes = data_size;
                data = realloc(data as *mut ::core::ffi::c_void, max_data_bytes as size_t)
                    as *mut BYTE;
            } else {
                max_value_len = max_value_len.wrapping_mul(2);
                value_name = realloc(
                    value_name as *mut ::core::ffi::c_void,
                    (max_value_len as size_t)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
                ) as *mut WCHAR;
            }
        } else {
            break;
        }
    }

    free(data as *mut ::core::ffi::c_void);
    free(value_name as *mut ::core::ffi::c_void);

    if i != 0 || recursing != 0 {
        output_string(RUSTCONV_WSTR_1B2D5DD81FC6.as_ptr());
    }

    if num_subkeys == 0 {
        return 0 as ::core::ffi::c_int;
    }

    subkey_name = malloc(
        (MAX_SUBKEY_LEN as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    path_len = lstrlenW(path as LPCWSTR) as DWORD;

    i = 0;
    loop {
        subkey_len = MAX_SUBKEY_LEN;
        rc = RegEnumKeyExW(
            hkey,
            i,
            subkey_name as LPWSTR,
            &raw mut subkey_len,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) as LONG;
        if rc == ERROR_SUCCESS as LONG {
            if recurse != 0 {
                subkey_path = build_subkey_path(path, path_len, subkey_name, subkey_len);
                if RegOpenKeyExW(
                    hkey,
                    subkey_name as LPCWSTR,
                    0 as DWORD,
                    KEY_READ as REGSAM | sam,
                    &raw mut subkey,
                ) == 0
                {
                    query_all(subkey, subkey_path, recurse, (TRUE) as BOOL);
                    RegCloseKey(subkey);
                }
                free(subkey_path as *mut ::core::ffi::c_void);
            } else {
                output_string(RUSTCONV_WSTR_1B2D5DD81FC6.as_ptr(), path, subkey_name);
            }
            i = i.wrapping_add(1);
        } else {
            break;
        }
    }

    free(subkey_name as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn run_query(
    mut root: HKEY,
    mut path: *mut WCHAR,
    mut key_name: *mut WCHAR,
    mut value_name: *mut WCHAR,
    mut value_empty: BOOL,
    mut recurse: BOOL,
) -> ::core::ffi::c_int {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut ret: ::core::ffi::c_int;
    if RegOpenKeyExW(
        root,
        path as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM | sam,
        core::ptr::addr_of_mut!(hkey),
    ) != 0
    {
        output_message(STRING_KEY_NONEXIST as ::core::ffi::c_uint);
        return 1 as ::core::ffi::c_int;
    }

    output_string(newlineW);

    if !value_name.is_null() || value_empty != 0 {
        ret = query_value(hkey, value_name, key_name, recurse);
        if recurse != 0 {
            output_message(
                STRING_MATCHES_FOUND as ::core::ffi::c_uint,
                num_values_found,
            );
        }
    } else {
        ret = query_all(hkey, key_name, recurse, FALSE as BOOL);
    }

    RegCloseKey(hkey);

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn reg_query(mut argc: ::core::ffi::c_int, mut argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut root: HKEY = core::ptr::null_mut();
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut key_name: *mut WCHAR = core::ptr::null_mut();
    let mut value_name: *mut WCHAR = core::ptr::null_mut();
    let mut value_empty: BOOL = (FALSE) as i32;
    let mut recurse: BOOL = (FALSE) as i32;
    let mut i: ::core::ffi::c_int = 0;

    if parse_registry_key(
        *argvW.offset(2 as isize),
        core::ptr::addr_of_mut!(root),
        core::ptr::addr_of_mut!(path),
    ) == 0
    {
        return 1;
    }

    'body: {
        i = 3;
        while i < argc {
            'next: {
                if *(*argvW.offset(i as isize)).offset(0 as isize) as i32 != '/' as i32
                    && *(*argvW.offset(i as isize)).offset(0 as isize) as i32 != '-' as i32
                {
                    break 'body;
                }

                let str: *mut WCHAR = (*argvW.offset(i as isize)).offset(1 as isize);

                if lstrcmpiW(str, RUSTCONV_WSTR_05645F4502BA.as_ptr()) == 0 {
                    if value_empty != 0 {
                        break 'body;
                    }
                    value_empty = (TRUE) as i32;
                    break 'next;
                } else if lstrcmpiW(str, RUSTCONV_WSTR_744906669607.as_ptr()) == 0 {
                    if (sam & KEY_WOW64_32KEY) != 0 {
                        break 'body;
                    }
                    sam |= KEY_WOW64_32KEY;
                    break 'next;
                } else if lstrcmpiW(str, RUSTCONV_WSTR_E3997F49E5CE.as_ptr()) == 0 {
                    if (sam & KEY_WOW64_64KEY) != 0 {
                        break 'body;
                    }
                    sam |= KEY_WOW64_64KEY;
                    break 'next;
                } else if *str.offset(0 as isize) == 0
                    || *str.offset(1 as isize) as i32 != 0
                {
                    break 'body;
                }

                match towlower(*str) as i32 {
                    118 => {
                        // 'v'
                        if !value_name.is_null() {
                            break 'body;
                        }
                        i += 1;
                        value_name = *argvW.offset(i as isize);
                        if value_name.is_null() {
                            break 'body;
                        }
                    }
                    115 => {
                        // 's'
                        if recurse != 0 {
                            break 'body;
                        }
                        recurse = (TRUE) as i32;
                    }
                    _ => {
                        break 'body;
                    }
                }
            }
            i += 1;
        }

        if !value_name.is_null() && value_empty != 0 {
            break 'body;
        }

        if sam == (KEY_WOW64_32KEY | KEY_WOW64_64KEY) {
            break 'body;
        }

        key_name = get_long_key(root, path);

        return run_query(root, path, key_name, value_name, value_empty, recurse);
    }

    output_message(STRING_INVALID_SYNTAX);
    output_message(
        STRING_FUNC_HELP,
        _wcsupr(*argvW.offset(1 as isize)),
    );
    return 1;
}
