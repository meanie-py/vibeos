use crate::reg::reg::ask_confirm;
use crate::reg::reg::build_subkey_path;
use crate::reg::reg::get_long_key;
use crate::reg::reg::is_char;
use crate::reg::reg::output_writeconsole;
use crate::reg::reg::parse_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0561B80DCBE9: [u16; 6] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_097381A2F37D: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_10A8D224346F: [u16; 9] = [0x000Du16, 0x000A, 0x005B, 0x0025, 0x0073, 0x005D, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_644DC1C64048: [u16; 3] = [0x0040u16, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_7443311FB636: [u16; 9] = [0x0068u16, 0x0065, 0x0078, 0x0028, 0x0025, 0x0078, 0x0029, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_AA2A982F0618: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_B23F78685766: [u16; 11] = [
    0x0064u16, 0x0077, 0x006F, 0x0072, 0x0064, 0x003A, 0x0025, 0x0030, 0x0038, 0x0078,
    0x0000,
];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_FFFF89D9175A: [u16; 6] = [0x005Cu16, 0x000D, 0x000A, 0x0020, 0x0020, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_HEX_CHARS: usize = 77;

unsafe extern "C" fn write_file(hFile: HANDLE, str: *const WCHAR) {
    let mut written: DWORD = 0;
    WriteFile(
        hFile,
        str as LPCVOID,
        (lstrlenW(str as LPCWSTR) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::addr_of_mut!(written),
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn escape_string(
    mut str: *mut WCHAR,
    mut str_len: size_t,
    mut line_len: *mut size_t,
) -> *mut WCHAR {
    let mut i: size_t = 0;
    let mut escape_count: size_t = 0;
    let mut pos: size_t = 0;
    let mut buf: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    i = 0;
    escape_count = 0;
    while i < str_len {
        let mut c: WCHAR = *str.offset(i as isize);
        if c == 0 {
            break;
        }
        if c as ::core::ffi::c_int == '\r' as i32
            || c as ::core::ffi::c_int == '\n' as i32
            || c as ::core::ffi::c_int == '\\' as i32
            || c as ::core::ffi::c_int == '"' as i32
        {
            escape_count = escape_count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    buf = malloc(
        str_len
            .wrapping_add(escape_count)
            .wrapping_add(1 as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    i = 0;
    pos = 0;
    while i < str_len {
        let mut c_0: WCHAR = *str.offset(i as isize);
        if c_0 == 0 {
            break;
        }
        match c_0 as ::core::ffi::c_int {
            13 => {
                let fresh15 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh15 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = 'r' as i32 as WCHAR;
            }
            10 => {
                let fresh16 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh16 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = 'n' as i32 as WCHAR;
            }
            92 => {
                let fresh17 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh17 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = '\\' as i32 as WCHAR;
            }
            34 => {
                let fresh18 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh18 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = '"' as i32 as WCHAR;
            }
            _ => {
                *buf.offset(pos as isize) = c_0;
            }
        }
        i = i.wrapping_add(1);
        pos = pos.wrapping_add(1);
    }
    *buf.offset(pos as isize) = 0 as WCHAR;
    *line_len = pos;
    return buf;
}

unsafe extern "C" fn export_value_name(hFile: HANDLE, name: *mut WCHAR, len: size_t) -> size_t {
    static mut default_name: *const WCHAR = RUSTCONV_WSTR_644DC1C64048.as_ptr();
    let mut line_len: size_t = 0;
    if !name.is_null() && *name != 0 {
        let str: *mut WCHAR = escape_string(name, len, core::ptr::addr_of_mut!(line_len));
        let buf: *mut WCHAR = malloc(
            (line_len + 4) * core::mem::size_of::<WCHAR>() as size_t,
        ) as *mut WCHAR;
        line_len = swprintf(buf, RUSTCONV_WSTR_0561B80DCBE9.as_ptr(), str) as size_t;
        write_file(hFile, buf);
        free(buf as *mut core::ffi::c_void);
        free(str as *mut core::ffi::c_void);
    } else {
        line_len = lstrlenW(default_name as LPCWSTR) as size_t;
        write_file(hFile, default_name);
    }

    return line_len;
}

unsafe extern "C" fn export_string_data(buf: *mut *mut WCHAR, data: *mut WCHAR, size: size_t) {
    let mut len: size_t = 0;
    let mut line_len: size_t = 0;
    let mut str: *mut WCHAR = core::ptr::null_mut();
    if size != 0 {
        len = (size / core::mem::size_of::<WCHAR>() as size_t) - 1;
    }
    str = escape_string(data, len, core::ptr::addr_of_mut!(line_len));
    *buf = malloc(
        (line_len + 3) * core::mem::size_of::<WCHAR>() as size_t,
    ) as *mut WCHAR;
    swprintf(*buf, RUSTCONV_WSTR_9896841D0A6A.as_ptr(), str);
    free(str as *mut core::ffi::c_void);
}

unsafe extern "C" fn export_dword_data(buf: *mut *mut WCHAR, data: *mut DWORD) {
    *buf = malloc(15 * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    swprintf(*buf, RUSTCONV_WSTR_B23F78685766.as_ptr(), *data);
}

unsafe extern "C" fn export_hex_data_type(hFile: HANDLE, type_0: DWORD) -> size_t {
    let hex: *const WCHAR = RUSTCONV_WSTR_097381A2F37D.as_ptr();
    let mut line_len: size_t = 0;

    if type_0 == REG_BINARY {
        line_len = lstrlenW(hex) as size_t;
        write_file(hFile, hex);
    } else {
        let buf: *mut WCHAR = malloc(
            (15 as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        line_len = swprintf(buf, RUSTCONV_WSTR_7443311FB636.as_ptr(), type_0) as size_t;
        write_file(hFile, buf);
        free(buf as *mut core::ffi::c_void);
    }

    return line_len;
}

unsafe extern "C" fn export_hex_data(
    mut hFile: HANDLE,
    mut buf: *mut *mut WCHAR,
    mut type_0: DWORD,
    mut line_len: DWORD,
    mut data: *mut ::core::ffi::c_void,
    mut size: DWORD,
) {
    let mut num_commas: size_t = 0;
    let mut i: size_t = 0;
    let mut pos: size_t = 0;

    line_len = (line_len as size_t).wrapping_add(export_hex_data_type(hFile, type_0)) as DWORD;

    if size == 0 {
        return;
    }

    num_commas = (size - 1) as size_t;
    *buf = malloc(
        (size as size_t).wrapping_mul(3 as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    i = 0 as size_t;
    pos = 0 as size_t;
    while i < size as size_t {
        pos = pos.wrapping_add(swprintf(
            (*buf).add(pos),
            RUSTCONV_WSTR_AA2A982F0618.as_ptr(),
            *(data as *mut BYTE).add(i) as ::core::ffi::c_int,
        ) as size_t);
        if i == num_commas {
            break;
        }
        let fresh_comma = pos;
        pos = pos.wrapping_add(1);
        *(*buf).add(fresh_comma) = ',' as WCHAR;
        *(*buf).add(pos) = 0 as WCHAR;
        line_len = line_len.wrapping_add(3 as DWORD);

        if line_len >= MAX_HEX_CHARS as DWORD {
            write_file(hFile, *buf);
            write_file(hFile, RUSTCONV_WSTR_FFFF89D9175A.as_ptr());
            line_len = 2 as DWORD;
            pos = 0 as size_t;
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn export_newline(hFile: HANDLE) {
    static mut newline: *const u16 = RUSTCONV_WSTR_80E09B2F645F.as_ptr();

    write_file(hFile, newline);
}

unsafe extern "C" fn export_data(
    mut hFile: HANDLE,
    mut value_name: *mut WCHAR,
    mut value_len: DWORD,
    mut type_: DWORD,
    mut data: *mut ::core::ffi::c_void,
    mut size: size_t,
) {
    let mut buf: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut line_len: size_t = export_value_name(hFile, value_name, value_len as size_t);

    match type_ {
        REG_SZ => {
            export_string_data(&raw mut buf, data as *mut WCHAR, size);
        }
        REG_DWORD => {
            if size != 0 {
                export_dword_data(&raw mut buf, data as *mut DWORD);
            } else {
                export_hex_data(
                    hFile,
                    &raw mut buf,
                    type_,
                    line_len as DWORD,
                    data,
                    size as DWORD,
                );
            }
        }
        REG_NONE | REG_EXPAND_SZ | REG_BINARY | REG_MULTI_SZ | _ => {
            export_hex_data(
                hFile,
                &raw mut buf,
                type_,
                line_len as DWORD,
                data,
                size as DWORD,
            );
        }
    }

    if size != 0 || type_ == REG_SZ {
        write_file(hFile, buf);
        free(buf as *mut ::core::ffi::c_void);
    }

    export_newline(hFile);
}

unsafe extern "C" fn export_key_name(hFile: HANDLE, name: *mut WCHAR) {
    let mut buf: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    buf = malloc(
        ((lstrlenW(name as LPCWSTR) + 7) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    swprintf(buf, RUSTCONV_WSTR_10A8D224346F.as_ptr(), name);
    write_file(hFile, buf);
    free(buf as *mut core::ffi::c_void);
}

unsafe extern "C" fn export_registry_data(mut hFile: HANDLE, mut hkey: HKEY, mut path: *mut WCHAR, mut sam: REGSAM) -> ::core::ffi::c_int {
    let mut rc: LONG = 0;
    let mut max_value_len: DWORD = 256 as DWORD;
    let mut value_len: DWORD = 0;
    let mut max_data_bytes: DWORD = 2048 as DWORD;
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

    export_key_name(hFile, path);

    value_name = malloc(
        (max_value_len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    data = malloc(max_data_bytes as size_t) as *mut BYTE;

    i = 0 as DWORD;
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
            export_data(
                hFile,
                value_name,
                value_len,
                type_0,
                data as *mut ::core::ffi::c_void,
                data_size as size_t,
            );
            i = i.wrapping_add(1);
        } else if rc == ERROR_MORE_DATA as LONG {
            if data_size > max_data_bytes {
                max_data_bytes = data_size;
                data = realloc(data as *mut ::core::ffi::c_void, max_data_bytes as size_t) as *mut BYTE;
            } else {
                max_value_len = max_value_len.wrapping_mul(2 as DWORD);
                value_name = realloc(
                    value_name as *mut ::core::ffi::c_void,
                    (max_value_len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
                ) as *mut WCHAR;
            }
        } else {
            break;
        }
    }

    free(data as *mut ::core::ffi::c_void);
    free(value_name as *mut ::core::ffi::c_void);

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
        if rc == ERROR_SUCCESS as LONG {
            subkey_path = build_subkey_path(path, path_len, subkey_name, subkey_len);
            if RegOpenKeyExW(
                hkey,
                subkey_name as LPCWSTR,
                0 as DWORD,
                KEY_READ as REGSAM | sam,
                &raw mut subkey,
            ) == 0
            {
                export_registry_data(hFile, subkey, subkey_path, sam);
                RegCloseKey(subkey);
            }
            free(subkey_path as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        } else {
            break;
        }
    }

    free(subkey_name as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn export_file_header(hFile: HANDLE) {
    static header: [u16; 40] = [
    0xFEFFu16, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0052,
    0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0045, 0x0064,
    0x0069, 0x0074, 0x006F, 0x0072, 0x0020, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069,
    0x006F, 0x006E, 0x0020, 0x0035, 0x002E, 0x0030, 0x0030, 0x000D, 0x000A, 0x0000,
];

    write_file(hFile, header.as_ptr());
}

unsafe extern "C" fn create_file(filename: *const WCHAR, action: DWORD) -> HANDLE {
    return CreateFileW(
        filename as LPCWSTR,
        GENERIC_WRITE as DWORD,
        0 as DWORD,
        core::ptr::null_mut(),
        action,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn get_file_handle(filename: *mut WCHAR, overwrite_file: BOOL) -> HANDLE {
    let mut hFile: HANDLE = create_file(
        filename,
        if overwrite_file != 0 { CREATE_ALWAYS } else { CREATE_NEW },
    );

    if hFile == INVALID_HANDLE_VALUE {
        let mut error: DWORD = GetLastError();

        if error == ERROR_FILE_EXISTS {
            if ask_confirm(STRING_OVERWRITE_FILE, filename) == 0 {
                output_message(STRING_CANCELLED);
                exit(0);
            }

            hFile = create_file(filename, CREATE_ALWAYS);
        } else {
            let mut str: *mut WCHAR = core::ptr::null_mut();

            FormatMessageW(
                FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                core::ptr::null(),
                error,
                0,
                core::ptr::addr_of_mut!(str) as *mut WCHAR,
                0,
                core::ptr::null_mut(),
            );
            output_writeconsole(str, (lstrlenW(str)) as DWORD);
            LocalFree((str).cast());
            exit(1);
        }
    }

    return hFile;
}
#[no_mangle]
pub unsafe extern "C" fn reg_export(argc: ::core::ffi::c_int, argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut root: HKEY = ::core::ptr::null_mut();
    let mut hkey: HKEY = ::core::ptr::null_mut();
    let mut path: *mut WCHAR = ::core::ptr::null_mut();
    let mut key_name: *mut WCHAR = ::core::ptr::null_mut();
    let mut overwrite_file: BOOL = (FALSE) as i32;
    let mut sam: REGSAM = 0;
    let mut hFile: HANDLE = ::core::ptr::null_mut();
    let mut i: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = 0;

    'invalid: {
        if argc < 4 {
            break 'invalid;
        }

        if parse_registry_key(
            *argvW.offset(2 as isize),
            core::ptr::addr_of_mut!(root),
            core::ptr::addr_of_mut!(path),
        ) == 0
        {
            return 1;
        }

        i = 4;
        while i < argc {
            'body: {
                let mut str: *mut WCHAR = ::core::ptr::null_mut();

                if *(*argvW.offset(i as isize)).offset(0 as isize) != '/' as WCHAR
                    && *(*argvW.offset(i as isize)).offset(0 as isize) != '-' as WCHAR
                {
                    break 'invalid;
                }

                str = (*argvW.offset(i as isize)).offset(1 as isize);

                if is_char(*str, 'y' as WCHAR) != 0
                    && *str.offset(1 as isize) == 0
                {
                    overwrite_file = (TRUE) as i32;
                } else if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_744906669607.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_32KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_32KEY as REGSAM;
                    break 'body;
                } else if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_E3997F49E5CE.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_64KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_64KEY as REGSAM;
                    break 'body;
                } else {
                    break 'invalid;
                }
            }
            i += 1;
        }

        if sam == ((KEY_WOW64_32KEY | KEY_WOW64_64KEY) as REGSAM) {
            break 'invalid;
        }

        if RegOpenKeyExW(
            root,
            path as LPCWSTR,
            0,
            (KEY_READ as REGSAM) | sam,
            core::ptr::addr_of_mut!(hkey),
        ) != 0
        {
            output_message(STRING_KEY_NONEXIST);
            return 1;
        }

        key_name = get_long_key(root, path);

        hFile = get_file_handle(
            *argvW.offset(3 as isize),
            overwrite_file,
        );
        export_file_header(hFile);
        ret = export_registry_data(hFile, hkey, key_name, sam);
        export_newline(hFile);
        CloseHandle(hFile);

        RegCloseKey(hkey);

        return ret;
    }

    output_message(STRING_INVALID_SYNTAX);
    output_message(
        STRING_FUNC_HELP,
        _wcsupr(*argvW.offset(1 as isize)),
    );
    return 1;
}
