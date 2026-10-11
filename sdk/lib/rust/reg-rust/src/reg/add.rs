use crate::reg::reg::ask_confirm;
use crate::reg::reg::parse_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05645F4502BA: [u16; 3] = [0x0076u16, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn wchar_get_type(type_name: *const WCHAR) -> DWORD {
    let mut i: DWORD;
    if type_name.is_null() {
        return REG_SZ;
    }
    i = 0;
    while (i as usize) < ARRAY_SIZE!(type_rels) as usize {
        if wcsicmp(
            type_rels[i as usize].name as *const WCHAR,
            type_name,
        ) == 0
        {
            return type_rels[i as usize].type_;
        }
        i = i.wrapping_add(1);
    }
    return !(0 as DWORD);
}

unsafe extern "C" fn hexchar_to_byte(mut ch: WCHAR) -> BYTE {
    if ch as ::core::ffi::c_int >= '0' as i32 && ch as ::core::ffi::c_int <= '9' as i32 {
        return (ch as ::core::ffi::c_int - '0' as i32) as BYTE;
    } else if ch as ::core::ffi::c_int >= 'a' as i32 && ch as ::core::ffi::c_int <= 'f' as i32 {
        return (ch as ::core::ffi::c_int - 'a' as i32 + 10 as ::core::ffi::c_int) as BYTE;
    } else if ch as ::core::ffi::c_int >= 'A' as i32 && ch as ::core::ffi::c_int <= 'F' as i32 {
        return (ch as ::core::ffi::c_int - 'A' as i32 + 10 as ::core::ffi::c_int) as BYTE;
    } else {
        return (-(1 as ::core::ffi::c_int)) as BYTE;
    };
}

unsafe extern "C" fn get_regdata(
    mut data: *const WCHAR,
    mut reg_type: DWORD,
    mut separator: WCHAR,
    mut data_bytes: *mut *mut BYTE,
    mut size_bytes: *mut DWORD,
) -> BOOL {
    static mut empty: WCHAR = 0;

    *size_bytes = 0;

    if data.is_null() {
        data = &raw const empty;
    }

    match reg_type {
        REG_NONE | REG_SZ | REG_EXPAND_SZ => {
            *size_bytes = ((lstrlenW(data as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
            *data_bytes = malloc(*size_bytes as usize) as *mut BYTE;
            lstrcpyW(*data_bytes as LPWSTR, data as LPCWSTR);
        }
        REG_DWORD | REG_DWORD_BIG_ENDIAN => {
            let mut rest: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut val: core::ffi::c_ulong = 0;
            val = wcstoul(
                data as *const WCHAR,
                &raw mut rest,
                if (towlower(*data.offset(1 as isize)) as i32
                    == 'x' as i32)
                {
                    16
                } else {
                    10
                },
            );
            if *rest as i32 != 0
                || *data.offset(0 as isize) as i32 == '-' as i32
                || (val == !(0) && (*_errno()) as u32 == ERANGE)
            {
                output_message(STRING_MISSING_NUMBER);
                return (FALSE) as BOOL;
            }
            *size_bytes = core::mem::size_of::<DWORD>() as DWORD;
            *data_bytes = malloc(*size_bytes as usize) as *mut BYTE;
            *(*data_bytes as *mut DWORD) = val as DWORD;
        }
        REG_BINARY => {
            let mut hex0: BYTE = 0;
            let mut hex1: BYTE = 0;
            let mut ptr: *mut BYTE = core::ptr::null_mut::<BYTE>();
            let mut i: i32 = 0;
            let mut destByteIndex: i32 = 0;
            let mut datalen: i32 = lstrlenW(data as LPCWSTR);

            if datalen == 0 {
                return (TRUE) as BOOL;
            }

            *size_bytes = ((((datalen + datalen % 2) / 2) as usize)
                .wrapping_mul(core::mem::size_of::<BYTE>())) as DWORD;
            *data_bytes = malloc(*size_bytes as usize) as *mut BYTE;

            'sw3: {
                'no_hex: {
                    if datalen % 2 != 0 {
                        let fresh15 = i;
                        i += 1;
                        hex1 = hexchar_to_byte(*data.offset(fresh15 as isize));
                        if hex1 == 0xFF {
                            break 'no_hex;
                        }
                        let fresh16 = destByteIndex;
                        destByteIndex += 1;
                        **data_bytes.offset(fresh16 as isize) = hex1;
                    }

                    ptr = *data_bytes;

                    loop {
                        if !(i + 1 < datalen) {
                            break;
                        }
                        hex0 = hexchar_to_byte(*data.offset(i as isize));
                        hex1 = hexchar_to_byte(*data.offset((i + 1) as isize));
                        if hex0 == 0xFF || hex1 == 0xFF {
                            break 'no_hex;
                        }
                        let fresh17 = destByteIndex;
                        destByteIndex += 1;
                        *ptr.offset(fresh17 as isize) =
                            (((hex0 as i32) << 4) | hex1 as i32) as BYTE;
                        i += 2;
                    }

                    break 'sw3;
                }

                free(*data_bytes as *mut core::ffi::c_void);
                *data_bytes = core::ptr::null_mut::<BYTE>();
                output_message(STRING_MISSING_HEXDATA);
                return (FALSE) as BOOL;
            }
        }
        REG_MULTI_SZ => {
            let mut i_0: i32 = 0;
            let mut destindex: i32 = 0;
            let mut len: i32 = lstrlenW(data as LPCWSTR);
            let mut buffer: *mut WCHAR = malloc(
                ((len + 2) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;

            while i_0 < len {
                if separator == 0
                    && (*data.offset(i_0 as isize) as i32 == '\\' as i32)
                    && (*data.offset((i_0 + 1) as isize) as i32 == '0' as i32)
                {
                    *buffer.offset(destindex as isize) = 0;
                    i_0 += 1;
                } else if *data.offset(i_0 as isize) == separator {
                    *buffer.offset(destindex as isize) = 0;
                } else {
                    *buffer.offset(destindex as isize) = *data.offset(i_0 as isize);
                }

                if destindex != 0
                    && *buffer.offset((destindex - 1) as isize) == 0
                    && (*buffer.offset(destindex as isize) == 0 || destindex == 1)
                {
                    free(buffer as *mut core::ffi::c_void);
                    output_message(STRING_INVALID_STRING);
                    return (FALSE) as BOOL;
                }

                i_0 += 1;
                destindex += 1;
            }

            *buffer.offset(destindex as isize) = 0;
            if destindex != 0 && *buffer.offset((destindex - 1) as isize) != 0 {
                destindex += 1;
                *buffer.offset(destindex as isize) = 0;
            }
            *size_bytes = (((destindex + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD;
            *data_bytes = buffer as *mut BYTE;
        }
        _ => {
            output_message(STRING_UNHANDLED_TYPE, reg_type, data);
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn run_add(root: HKEY, path: *mut WCHAR, sam: REGSAM, value_name: *mut WCHAR, value_empty: BOOL, type_0: *mut WCHAR, separator: WCHAR, data: *mut WCHAR, force: BOOL) -> i32 {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut dispos: DWORD = 0;
    let mut data_type: DWORD = 0;
    let mut data_size: DWORD = 0;
    let mut reg_data: *mut BYTE = core::ptr::null_mut();
    let mut rc: LONG = 0;

    if RegCreateKeyExW(root, path as LPCWSTR, 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE,
                        (KEY_READ | KEY_WRITE) as REGSAM | sam, core::ptr::null_mut(), &raw mut hkey, &raw mut dispos) != 0
    {
        output_message(STRING_ACCESS_DENIED as ::core::ffi::c_uint);
        return 1 as i32;
    }

    if force == 0 && dispos == REG_OPENED_EXISTING_KEY as DWORD
    {
        if RegQueryValueExW(hkey, value_name as LPCWSTR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) == ERROR_SUCCESS as LSTATUS
        {
            if ask_confirm(STRING_OVERWRITE_VALUE as ::core::ffi::c_uint, value_name) == 0
            {
                RegCloseKey(hkey);
                output_message(STRING_CANCELLED as ::core::ffi::c_uint);
                return 0 as i32;
            }
        }
    }

    data_type = wchar_get_type(type_0);

    if data_type == !0 as DWORD
    {
        RegCloseKey(hkey);
        output_message(STRING_UNSUPPORTED_TYPE as ::core::ffi::c_uint, type_0);
        return 1 as i32;
    }

    if (data_type == REG_DWORD as DWORD || data_type == REG_DWORD_BIG_ENDIAN as DWORD) && data.is_null()
    {
        RegCloseKey(hkey);
        output_message(STRING_INVALID_CMDLINE as ::core::ffi::c_uint);
        return 1 as i32;
    }

    if get_regdata(data, data_type, separator, &raw mut reg_data, &raw mut data_size) == 0
    {
        RegCloseKey(hkey);
        return 1 as i32;
    }

    rc = RegSetValueExW(hkey, value_name as LPCWSTR, 0 as DWORD, data_type, reg_data, data_size) as LONG;

    free(reg_data as *mut core::ffi::c_void);
    RegCloseKey(hkey);

    if rc != 0
    {
        output_message(STRING_ACCESS_DENIED as ::core::ffi::c_uint);
        return 1 as i32;
    }

    output_message(STRING_SUCCESS as ::core::ffi::c_uint);

    return 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn reg_add(argc: ::core::ffi::c_int, argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut root: HKEY = core::ptr::null_mut();
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut value_name: *mut WCHAR = core::ptr::null_mut();
    let mut type_0: *mut WCHAR = core::ptr::null_mut();
    let mut data: *mut WCHAR = core::ptr::null_mut();
    let mut separator: WCHAR = 0;
    let mut value_empty: BOOL = (FALSE) as i32;
    let mut force: BOOL = (FALSE) as i32;
    let mut sam: REGSAM = 0 as REGSAM;
    let mut i: ::core::ffi::c_int;

    if parse_registry_key(
        *argvW.offset(2 as isize),
        core::ptr::addr_of_mut!(root),
        core::ptr::addr_of_mut!(path),
    ) == 0
    {
        return 1;
    }

    'invalid: {
        i = 3;
        while i < argc {
            'body: {
                let mut str: *mut WCHAR;

                if *(*argvW.offset(i as isize)).offset(0 as isize) as ::core::ffi::c_int != '/' as ::core::ffi::c_int
                    && *(*argvW.offset(i as isize)).offset(0 as isize) as ::core::ffi::c_int != '-' as ::core::ffi::c_int
                {
                    break 'invalid;
                }

                str = (*argvW.offset(i as isize)).offset(1 as isize);

                if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_05645F4502BA.as_ptr()) == 0 {
                    if value_empty != 0 {
                        break 'invalid;
                    }
                    value_empty = (TRUE) as i32;
                    break 'body;
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
                } else if *str.offset(0 as isize) == 0
                    || *str.offset(1 as isize) as ::core::ffi::c_int != 0
                {
                    break 'invalid;
                }

                match towlower(*str) as ::core::ffi::c_int {
                    118 => {
                        if !value_name.is_null() || {
                            i += 1;
                            value_name = *argvW.offset(i as isize);
                            value_name.is_null()
                        } {
                            break 'invalid;
                        }
                    }
                    116 => {
                        if !type_0.is_null() || {
                            i += 1;
                            type_0 = *argvW.offset(i as isize);
                            type_0.is_null()
                        } {
                            break 'invalid;
                        }
                    }
                    100 => {
                        if !data.is_null() || {
                            i += 1;
                            data = *argvW.offset(i as isize);
                            data.is_null()
                        } {
                            break 'invalid;
                        }
                    }
                    115 => {
                        i += 1;
                        str = *argvW.offset(i as isize);
                        if separator as ::core::ffi::c_int != 0
                            || str.is_null()
                            || lstrlenW(str as LPCWSTR) != 1
                        {
                            break 'invalid;
                        }
                        separator = *str.offset(0 as isize);
                    }
                    102 => {
                        if force != 0 {
                            break 'invalid;
                        }
                        force = (TRUE) as i32;
                    }
                    _ => {
                        break 'invalid;
                    }
                }
            }
            i += 1;
        }

        if !value_name.is_null() && value_empty != 0 {
            break 'invalid;
        }

        if sam == (KEY_WOW64_32KEY | KEY_WOW64_64KEY) as REGSAM {
            break 'invalid;
        }

        return run_add(
            root,
            path,
            sam,
            value_name,
            value_empty,
            type_0,
            separator,
            data,
            force,
        );
    }

    output_message(STRING_INVALID_SYNTAX);
    output_message(
        STRING_FUNC_HELP,
        _wcsupr(*argvW.offset(1 as isize)),
    );
    return 1;
}
