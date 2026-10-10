use crate::reg::reg::ask_confirm;
use crate::reg::reg::get_long_key;
use crate::reg::reg::parse_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05645F4502BA: [u16; 3] = [0x0076u16, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_6ADB75E2E54C: [u16; 3] = [0x0076u16, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut op_delete_key: BOOL = (TRUE) as i32;

unsafe extern "C" fn output_error(rc: LONG) {
    if rc == ERROR_FILE_NOT_FOUND as LONG {
        if op_delete_key != 0 {
            output_message(STRING_KEY_NONEXIST as ::core::ffi::c_uint);
        } else {
            output_message(STRING_VALUE_NONEXIST as ::core::ffi::c_uint);
        }
    } else {
        output_message(STRING_ACCESS_DENIED as ::core::ffi::c_uint);
    }
}

unsafe extern "C" fn run_delete(root: HKEY, path: *mut WCHAR, sam: REGSAM, key_name: *mut WCHAR, value_name: *mut WCHAR, value_empty: BOOL, value_all: BOOL, force: BOOL) -> core::ffi::c_int {
    let mut rc: LONG = 0;
    let mut hkey: HKEY = core::ptr::null_mut();

    if force == 0 {
        let mut ret: BOOL = 0;

        if !value_name.is_null() || value_empty != 0 {
            ret = ask_confirm(STRING_DELETE_VALUE, value_name);
        } else if value_all != 0 {
            ret = ask_confirm(STRING_DELETE_VALUEALL, key_name);
        } else {
            ret = ask_confirm(STRING_DELETE_SUBKEY, key_name);
        }

        if ret == 0 {
            output_message(STRING_CANCELLED);
            return 0;
        }
    }

    rc = RegOpenKeyExW(root, path, 0, (KEY_READ | KEY_SET_VALUE | sam), core::ptr::addr_of_mut!(hkey)) as LONG;
    if rc != 0 {
        output_error(rc);
        return 1;
    }

    /* Delete registry key if no /v* option is given */
    if value_name.is_null() && value_empty == 0 && value_all == 0 {
        rc = RegDeleteTreeW(hkey, core::ptr::null()) as LONG;
        if rc != 0 {
            RegCloseKey(hkey);
            output_error(rc);
            return 1;
        }

        RegDeleteKeyW(hkey, RUSTCONV_WSTR_96A296D224F2.as_ptr());
        RegCloseKey(hkey);

        output_message(STRING_SUCCESS);
        return 0;
    }

    op_delete_key = (FALSE) as i32;

    if value_all != 0 {
        let mut max_value_len: DWORD = 256;
        let mut value_len: DWORD = 0;
        let mut value_name: *mut WCHAR = core::ptr::null_mut();

        value_name = malloc((max_value_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as *mut WCHAR;

        while 1 != 0 {
            value_len = max_value_len;
            rc = RegEnumValueW(hkey, 0, value_name, core::ptr::addr_of_mut!(value_len), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) as LONG;
            if (rc) as u32 == ERROR_SUCCESS {
                rc = RegDeleteValueW(hkey, value_name) as LONG;
                if (rc) as u32 != ERROR_SUCCESS {
                    free((value_name).cast());
                    RegCloseKey(hkey);
                    output_message(STRING_VALUEALL_FAILED, key_name);
                    output_error(rc);
                    return 1;
                }
            } else if (rc) as u32 == ERROR_MORE_DATA {
                max_value_len = max_value_len.wrapping_mul(2);
                value_name = realloc(value_name as *mut core::ffi::c_void, (max_value_len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as *mut WCHAR;
            } else {
                break;
            }
        }
        free(value_name as *mut core::ffi::c_void);
    } else if !value_name.is_null() || value_empty != 0 {
        rc = RegDeleteValueW(hkey, value_name) as LONG;
        if rc != 0 {
            RegCloseKey(hkey);
            output_error(rc);
            return 1;
        }
    }

    RegCloseKey(hkey);
    output_message(STRING_SUCCESS);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn reg_delete(argc: ::core::ffi::c_int, argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut root: HKEY = ::core::ptr::null_mut();
    let mut path: *mut WCHAR = ::core::ptr::null_mut();
    let mut key_name: *mut WCHAR = ::core::ptr::null_mut();
    let mut value_name: *mut WCHAR = ::core::ptr::null_mut();
    let mut value_all: BOOL = FALSE as BOOL;
    let mut value_empty: BOOL = FALSE as BOOL;
    let mut force: BOOL = FALSE as BOOL;
    let mut sam: REGSAM = 0 as REGSAM;
    let mut i: ::core::ffi::c_int = 0;

    if parse_registry_key(
        *argvW.offset(2 as isize),
        core::ptr::addr_of_mut!(root),
        core::ptr::addr_of_mut!(path),
    ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }

    'invalid: {
        i = 3 as ::core::ffi::c_int;
        while i < argc {
            'body: {
                let mut str: *mut WCHAR = ::core::ptr::null_mut();

                if *(*argvW.offset(i as isize)).offset(0 as isize) != '/' as WCHAR
                    && *(*argvW.offset(i as isize)).offset(0 as isize) != '-' as WCHAR
                {
                    break 'invalid;
                }

                str = (*argvW.offset(i as isize)).offset(1 as isize);

                if lstrcmpiW(str, RUSTCONV_WSTR_6ADB75E2E54C.as_ptr()) == 0 {
                    if value_all != 0 {
                        break 'invalid;
                    }
                    value_all = TRUE as BOOL;
                    break 'body;
                } else if lstrcmpiW(str, RUSTCONV_WSTR_05645F4502BA.as_ptr()) == 0 {
                    if value_empty != 0 {
                        break 'invalid;
                    }
                    value_empty = TRUE as BOOL;
                    break 'body;
                } else if lstrcmpiW(str, RUSTCONV_WSTR_744906669607.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_32KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_32KEY as REGSAM;
                    break 'body;
                } else if lstrcmpiW(str, RUSTCONV_WSTR_E3997F49E5CE.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_64KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_64KEY as REGSAM;
                    break 'body;
                } else if *str.offset(0 as isize) == 0
                    || *str.offset(1 as isize) != 0
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
                    102 => {
                        if force != 0 {
                            break 'invalid;
                        }
                        force = TRUE as BOOL;
                    }
                    _ => {
                        break 'invalid;
                    }
                }
            }
            i += 1;
        }

        if (!value_name.is_null() && value_empty != 0)
            || (!value_name.is_null() && value_all != 0)
            || (value_empty != 0 && value_all != 0)
        {
            break 'invalid;
        }

        if sam == (KEY_WOW64_32KEY | KEY_WOW64_64KEY) as REGSAM {
            break 'invalid;
        }

        key_name = get_long_key(root, path);

        return run_delete(
            root,
            path,
            sam,
            key_name,
            value_name,
            value_empty,
            value_all,
            force,
        );
    }

    output_message(STRING_INVALID_SYNTAX as ::core::ffi::c_uint);
    output_message(
        STRING_FUNC_HELP as ::core::ffi::c_uint,
        _wcsupr(*argvW.offset(1 as isize)),
    );
    return 1 as ::core::ffi::c_int;
}
