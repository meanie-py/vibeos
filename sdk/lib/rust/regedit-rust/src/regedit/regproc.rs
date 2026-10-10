// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0561B80DCBE9: [u16; 6] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_097381A2F37D: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_10A8D224346F: [u16; 9] = [0x000Du16, 0x000A, 0x005B, 0x0025, 0x0073, 0x005D, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_1B9334FEECE6: [u16; 2] = [0x0022u16, 0x0000];
pub static RUSTCONV_WSTR_1EA9E69A696A: [u16; 3] = [0x0077u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_28276425D458: [u16; 2] = [0x002Du16, 0x0000];
pub static RUSTCONV_WSTR_43425498687F: [u16; 4] = [0x0030u16, 0x0030, 0x002C, 0x0000];
pub static RUSTCONV_WSTR_7443311FB636: [u16; 9] = [0x0068u16, 0x0065, 0x0078, 0x0028, 0x0025, 0x0078, 0x0029, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_994915F97EDD: [u16; 37] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0052, 0x0065,
    0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0045, 0x0064, 0x0069,
    0x0074, 0x006F, 0x0072, 0x0020, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0020, 0x0035, 0x002E, 0x0030, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_A7125E3ED5E3: [u16; 7] = [0x0064u16, 0x0077, 0x006F, 0x0072, 0x0064, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_AA2A982F0618: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_B23F78685766: [u16; 11] = [
    0x0064u16, 0x0077, 0x006F, 0x0072, 0x0064, 0x003A, 0x0025, 0x0030, 0x0038, 0x0078,
    0x0000,
];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_C6E98D2C3B58: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x0028, 0x0000];
pub static RUSTCONV_WSTR_D392DA5B86CA: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x0045, 0x0044, 0x0049, 0x0054, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_E89B9A83645C: [u16; 8] = [0x0072u16, 0x0065, 0x0067, 0x0065, 0x0064, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_FFFF89D9175A: [u16; 6] = [0x005Cu16, 0x000D, 0x000A, 0x0020, 0x0020, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut reg_class_keys: [HKEY; 6] = unsafe {
    [
        HKEY_LOCAL_MACHINE,
        HKEY_USERS,
        HKEY_CLASSES_ROOT,
        HKEY_CURRENT_CONFIG,
        HKEY_CURRENT_USER,
        HKEY_DYN_DATA,
    ]
};

static mut get_line: Option<unsafe extern "C" fn(*mut FILE) -> *mut WCHAR> = None;

static mut parser_funcs: [parser_state_func; 17] = unsafe {
    [
        Some(header_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(parse_win31_line_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(line_start_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(key_name_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(delete_key_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(
            default_value_name_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR,
        ),
        Some(
            quoted_value_name_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR,
        ),
        Some(data_start_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(delete_value_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(data_type_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(string_data_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(dword_data_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(hex_data_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(eol_backslash_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(hex_multiline_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(unknown_data_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
        Some(set_value_state as unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR),
    ]
};

pub const REG_VAL_BUF_SIZE: usize = 4096;


pub const HEADER: i32 = 0;
pub const PARSE_WIN31_LINE: i32 = 1;
pub const LINE_START: i32 = 2;
pub const KEY_NAME: i32 = 3;
pub const DELETE_KEY: i32 = 4;
pub const DEFAULT_VALUE_NAME: i32 = 5;
pub const QUOTED_VALUE_NAME: i32 = 6;
pub const DATA_START: i32 = 7;
pub const DELETE_VALUE: i32 = 8;
pub const DATA_TYPE: i32 = 9;
pub const STRING_DATA: i32 = 10;
pub const DWORD_DATA: i32 = 11;
pub const HEX_DATA: i32 = 12;
pub const EOL_BACKSLASH: i32 = 13;
pub const HEX_MULTILINE: i32 = 14;
pub const UNKNOWN_DATA: i32 = 15;
pub const SET_VALUE: i32 = 16;
pub const NB_PARSER_STATES: i32 = 17;

#[repr(C)]
pub struct parser {
    pub file: *mut FILE,
    pub two_wchars: [WCHAR; 2],
    pub is_unicode: BOOL,
    pub reg_version: i16,
    pub hkey: HKEY,
    pub key_name: *mut WCHAR,
    pub value_name: *mut WCHAR,
    pub parse_type: DWORD,
    pub data_type: DWORD,
    pub data: *mut core::ffi::c_void,
    pub data_size: DWORD,
    pub backslash: BOOL,
    pub unicode_in_asc: BOOL,
    pub state: i32,
}
pub type PPARSER = *mut parser;

// A C function pointer is nullable: the table is `Some(..)` per state.
pub type parser_state_func = Option<unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR>;

pub const REG_VERSION_31: i32 = 0;
pub const REG_VERSION_40: i32 = 1;
pub const REG_VERSION_50: i32 = 2;
pub const REG_VERSION_FUZZY: i32 = 3;
pub const REG_VERSION_INVALID: i32 = 4;

pub const MAX_HEX_CHARS: usize = 77;

pub const MAX_SUBKEY_LEN: usize = 257;

unsafe extern "C" fn GetWideString(strA: *const ::core::ffi::c_char) -> *mut WCHAR {
    if !strA.is_null() {
        let mut strW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        let len: ::core::ffi::c_int = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            strA as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        );

        strW = malloc(
            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()) as size_t,
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            strA as LPCSTR,
            -(1 as ::core::ffi::c_int),
            strW as LPWSTR,
            len,
        );
        return strW;
    }
    return ::core::ptr::null_mut::<WCHAR>();
}

unsafe extern "C" fn GetWideStringN(
    strA: *const ::core::ffi::c_char,
    chars: ::core::ffi::c_int,
    len: *mut DWORD,
) -> *mut WCHAR {
    if !strA.is_null() {
        let mut strW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        *len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            strA as LPCSTR,
            chars,
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as DWORD;
        strW = malloc(
            (*len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            strA as LPCSTR,
            chars,
            strW as LPWSTR,
            *len as ::core::ffi::c_int,
        );
        return strW;
    }
    *len = 0 as DWORD;
    return ::core::ptr::null_mut::<WCHAR>();
}
#[no_mangle]
pub unsafe extern "C" fn GetMultiByteString(strW: *const WCHAR) -> *mut ::core::ffi::c_char {
    if !strW.is_null() {
        let mut strA: *mut ::core::ffi::c_char = core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            strW as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            core::ptr::null_mut::<CHAR>(),
            0 as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );

        strA = malloc(len as size_t) as *mut ::core::ffi::c_char;
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            strW as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            strA as LPSTR,
            len,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        return strA;
    }
    return core::ptr::null_mut::<::core::ffi::c_char>();
}

unsafe extern "C" fn GetMultiByteStringN(strW: *const WCHAR, chars: ::core::ffi::c_int, len: *mut DWORD) -> *mut ::core::ffi::c_char {
    if !strW.is_null() {
        let mut strA: *mut ::core::ffi::c_char = core::ptr::null_mut();
        *len = WideCharToMultiByte(CP_ACP, 0, strW, chars, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;

        strA = malloc(*len as size_t) as *mut ::core::ffi::c_char;
        WideCharToMultiByte(CP_ACP, 0, strW, chars, strA, *len as ::core::ffi::c_int, core::ptr::null_mut(), core::ptr::null_mut());
        return strA;
    }
    *len = 0 as DWORD;
    return core::ptr::null_mut();
}

unsafe extern "C" fn set_state(parser: *mut parser, state: parser_state) -> parser_state {
    let mut ret: parser_state = ((*parser).state) as u32;
    (*parser).state = (state) as i32;
    return ret;
}

unsafe extern "C" fn convert_hex_to_dword(str: *mut WCHAR, dw: *mut DWORD) -> BOOL {
    let mut str = str;
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut end: *mut WCHAR = core::ptr::null_mut();
    let mut count: core::ffi::c_int = 0;

    'error: {
        while *str == ' ' as WCHAR || *str == '\t' as WCHAR {
            str = str.add(1);
        }
        if *str == 0 {
            break 'error;
        }

        p = str;
        while iswxdigit(*p) != 0 {
            count += 1;
            p = p.add(1);
        }
        if count > 8 {
            break 'error;
        }

        end = p;
        while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
            p = p.add(1);
        }
        if *p != 0 && *p != ';' as WCHAR {
            break 'error;
        }

        *end = 0;
        *dw = wcstoul(str, core::ptr::addr_of_mut!(end), 16) as DWORD;
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn convert_hex_csv_to_hex(parser: *mut parser, str: *mut *mut WCHAR) -> BOOL {
    let mut size: size_t = 0;
    let mut d: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut s: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    (*parser).backslash = (FALSE) as i32;

    /* The worst case is 1 digit + 1 comma per byte */
    size = (((lstrlenW(*str as LPCWSTR) + 1) / 2)
        as DWORD)
        .wrapping_add((*parser).data_size) as size_t;
    (*parser).data = realloc((*parser).data, size);

    s = *str;
    d = ((*parser).data as *mut BYTE).offset((*parser).data_size as isize);

    while *s != 0 {
        let mut end: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
        let mut wc: core::ffi::c_ulong = 0;

        wc = wcstoul(s, core::ptr::addr_of_mut!(end), 16 as core::ffi::c_int);
        if wc > 0xff {
            return (FALSE) as BOOL;
        }

        if s == end && wc == 0 {
            while *end == ' ' as WCHAR || *end == '\t' as WCHAR {
                end = end.offset(1);
            }
            if *end == '\\' as WCHAR {
                (*parser).backslash = (TRUE) as i32;
                *str = end.offset(1 as isize);
                return (TRUE) as BOOL;
            } else if *end == ';' as WCHAR {
                return (TRUE) as BOOL;
            }
            return (FALSE) as BOOL;
        }

        *d = wc as BYTE;
        d = d.offset(1);
        (*parser).data_size = (*parser).data_size.wrapping_add(1);

        if *end != 0 && *end != ',' as WCHAR {
            while *end == ' ' as WCHAR || *end == '\t' as WCHAR {
                end = end.offset(1);
            }
            if *end != 0 && *end != ';' as WCHAR {
                return (FALSE) as BOOL;
            }
            return (TRUE) as BOOL;
        }

        if *end != 0 {
            end = end.offset(1);
        }
        s = end;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn parse_data_type(mut parser: *mut parser, mut line: *mut *mut WCHAR) -> BOOL {
    #[allow(non_camel_case_types)]
    struct data_type {
        tag: *const WCHAR,
        len: i32,
        type_0: i32,
        parse_type: i32,
    }
    static mut data_types: [data_type; 5] = [
        data_type {
            tag: RUSTCONV_WSTR_1B9334FEECE6.as_ptr(),
            len: 1,
            type_0: (REG_SZ) as i32,
            parse_type: (REG_SZ) as i32,
        },
        data_type {
            tag: RUSTCONV_WSTR_097381A2F37D.as_ptr(),
            len: 4,
            type_0: (REG_BINARY) as i32,
            parse_type: (REG_BINARY) as i32,
        },
        data_type {
            tag: RUSTCONV_WSTR_A7125E3ED5E3.as_ptr(),
            len: 6,
            type_0: (REG_DWORD) as i32,
            parse_type: (REG_DWORD) as i32,
        },
        data_type {
            tag: RUSTCONV_WSTR_C6E98D2C3B58.as_ptr(),
            len: 4,
            type_0: -1,
            parse_type: (REG_BINARY) as i32,
        },
        data_type {
            tag: core::ptr::null(),
            len: 0,
            type_0: 0,
            parse_type: 0,
        },
    ];

    let mut ptr: *const data_type = core::ptr::null();

    // Selects the __REACTOS__ preprocessor branch of the C source.
    let use_reactos: bool = true;
    if use_reactos {
        let len: DWORD = wcslen(*line) as DWORD;

        (*parser).unicode_in_asc = (FALSE) as i32;

        if (*parser).is_unicode == 0 && len >= 15 {
            let mut Buffer: [WCHAR; 16] = [0; 16];
            let mut ret: *mut WCHAR = core::ptr::null_mut();

            memcpy(
                core::ptr::addr_of_mut!(Buffer) as *mut core::ffi::c_void,
                *line as *const core::ffi::c_void,
                30 as usize,
            );
            Buffer[(_countof!(Buffer) - 1) as usize] = UNICODE_NULL;

            ret = wcsstr(Buffer.as_mut_ptr(), RUSTCONV_WSTR_43425498687F.as_ptr()) as *mut WCHAR; // Any UNICODE characters?
            (*parser).unicode_in_asc = (ret != core::ptr::null_mut()) as BOOL;
        }
    }

    ptr = core::ptr::addr_of!(data_types) as *const data_type;
    while !(*ptr).tag.is_null() {
        if wcsncmp((*ptr).tag, *line, (*ptr).len as usize) != 0 {
            ptr = ptr.offset(1);
        } else {
            (*parser).parse_type = (*ptr).parse_type as DWORD;
            (*parser).data_type = (*ptr).parse_type as DWORD;
            *line = (*line).offset((*ptr).len as isize);

            if (*ptr).type_0 == -1 {
                let mut end: *mut WCHAR = core::ptr::null_mut();
                let mut val: DWORD = 0;

                if **line == 0
                    || towlower(*(*line).offset(1 as isize)) as i32
                        == 'x' as i32
                {
                    return (FALSE) as BOOL;
                }

                /* "hex(xx):" is special */
                val = wcstoul(*line, core::ptr::addr_of_mut!(end), 16) as DWORD;

                let bad: bool = if use_reactos {
                    /* Up to 8 hex digits, "hex(000000002)" is invalid */
                    (*end as i32 != ')' as i32)
                        || (*end.offset(1 as isize) as i32 != ':' as i32)
                        || (val == !(0 as DWORD) && (*_errno()) as u32 == ERANGE)
                        || ((end.offset_from(*line) as i64) > 8 as i64)
                } else {
                    (*end as i32 != ')' as i32)
                        || (*end.offset(1 as isize) as i32 != ':' as i32)
                        || (val == !(0 as DWORD) && (*_errno()) as u32 == ERANGE)
                };
                if bad {
                    return (FALSE) as BOOL;
                }

                (*parser).data_type = val;
                *line = end.offset(2 as isize);
            }
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn REGPROC_unescape_string(mut str: *mut WCHAR, mut unparsed: *mut *mut WCHAR) -> BOOL {
    let mut str_idx: ::core::ffi::c_int = 0;
    let mut val_idx: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = lstrlenW(str as LPCWSTR);
    let mut ret: BOOL = 0;
    str_idx = 0;
    while str_idx < len {
        if *str.offset(str_idx as isize) as ::core::ffi::c_int == '\\' as i32 {
            str_idx += 1;
            match *str.offset(str_idx as isize) as ::core::ffi::c_int {
                110 => {
                    // 'n'
                    *str.offset(val_idx as isize) = '\n' as i32 as WCHAR;
                }
                114 => {
                    // 'r'
                    *str.offset(val_idx as isize) = '\r' as i32 as WCHAR;
                }
                48 => {
                    // '0'
                    return (FALSE) as BOOL;
                }
                92 | 34 => {
                    // '\\' and '"'
                    *str.offset(val_idx as isize) = *str.offset(str_idx as isize);
                }
                _ => {
                    if *str.offset(str_idx as isize) == 0 {
                        return (FALSE) as BOOL;
                    }
                    output_message(
                        STRING_ESCAPE_SEQUENCE as ::core::ffi::c_uint,
                        *str.offset(str_idx as isize) as ::core::ffi::c_int,
                    );
                    *str.offset(val_idx as isize) = *str.offset(str_idx as isize);
                }
            }
        } else if *str.offset(str_idx as isize) as ::core::ffi::c_int == '"' as i32 {
            break;
        } else {
            *str.offset(val_idx as isize) = *str.offset(str_idx as isize);
        }
        str_idx += 1;
        val_idx += 1;
    }
    ret = (*str.offset(str_idx as isize) as ::core::ffi::c_int == '"' as i32) as ::core::ffi::c_int as BOOL;
    *unparsed = str
        .offset(str_idx as isize)
        .offset(1 as isize);
    *str.offset(val_idx as isize) = '\0' as i32 as WCHAR;
    return ret;
}

unsafe extern "C" fn parse_key_name(key_name: *mut WCHAR, key_path: *mut *mut WCHAR) -> HKEY {
    let mut i: core::ffi::c_uint = 0;

    if key_name.is_null() {
        return core::ptr::null_mut();
    }

    *key_path = wcschr(key_name, '\\' as u16) as *mut WCHAR;
    if !(*key_path).is_null() {
        *key_path = (*key_path).offset(1);
    }

    while (i as usize) < ARRAY_SIZE!(reg_class_keys) as usize {
        let len: core::ffi::c_int = lstrlenW(
            *(&raw mut reg_class_namesW as *mut *const WCHAR).add(i as usize) as LPCWSTR,
        );
        // regproc.c:427, the `#ifdef __REACTOS__` arm (the translation had it compiled out).
        let cmp: core::ffi::c_int = _wcsnicmp(
            key_name,
            *(&raw mut reg_class_namesW as *mut *const WCHAR).add(i as usize),
            len as usize,
        );
        if cmp == 0
            && (*key_name.offset(len as isize) == 0
                || *key_name.offset(len as isize) == '\\' as u16)
        {
            return reg_class_keys[i as usize];
        }
        i = i.wrapping_add(1);
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn close_key(parser: *mut parser) {
    if !(*parser).hkey.is_null() {
        free((*parser).key_name as *mut ::core::ffi::c_void);
        (*parser).key_name = ::core::ptr::null_mut::<WCHAR>();

        RegCloseKey((*parser).hkey);
        (*parser).hkey = ::core::ptr::null_mut::<HKEY__>();
    }
}

unsafe extern "C" fn open_key(parser: *mut parser, path: *mut WCHAR) -> LONG {
    let mut key_class: HKEY = core::ptr::null_mut();
    let mut key_path: *mut WCHAR = core::ptr::null_mut();
    let mut res: LONG = 0;

    close_key(parser);

    /* Get the registry class */
    if path.is_null() || {
        key_class = parse_key_name(path, core::ptr::addr_of_mut!(key_path));
        key_class.is_null()
    } {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    res = RegCreateKeyExW(
        key_class,
        key_path as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut::<WCHAR>(),
        REG_OPTION_NON_VOLATILE as DWORD,
        KEY_ALL_ACCESS as REGSAM,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        core::ptr::addr_of_mut!((*parser).hkey),
        core::ptr::null_mut::<DWORD>(),
    ) as LONG;

    if res == ERROR_SUCCESS as LONG {
        (*parser).key_name = malloc(
            ((lstrlenW(path as LPCWSTR) + 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        lstrcpyW((*parser).key_name as LPWSTR, path as LPCWSTR);
    } else {
        (*parser).hkey = core::ptr::null_mut::<HKEY__>();
    }

    return res;
}

unsafe extern "C" fn free_parser_data(mut parser: *mut parser) {
    if (*parser).parse_type == REG_DWORD as DWORD || (*parser).parse_type == REG_BINARY as DWORD {
        free((*parser).data);
    }
    (*parser).data = core::ptr::null_mut();
    (*parser).data_size = 0 as DWORD;
}

unsafe extern "C" fn prepare_hex_string_data(mut parser: *mut parser) {
    if (*parser).data_type == REG_EXPAND_SZ as DWORD
        || (*parser).data_type == REG_MULTI_SZ as DWORD
        || (*parser).data_type == REG_SZ as DWORD
    {
        if (*parser).is_unicode != 0 || ((*parser).is_unicode == 0 && (*parser).unicode_in_asc != 0) {
            let mut data: *mut WCHAR = (*parser).data as *mut WCHAR;
            let mut len: DWORD = ((*parser).data_size as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as DWORD;
            if *data.offset(len.wrapping_sub(1 as DWORD) as isize) as ::core::ffi::c_int
                != 0 as WCHAR as ::core::ffi::c_int
            {
                *data.offset(len as isize) = 0 as WCHAR;
                (*parser).data_size = ((*parser).data_size as ::core::ffi::c_ulonglong)
                    .wrapping_add(
                        ::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong
                    ) as DWORD as DWORD;
            }
        } else {
            let mut data_0: *mut BYTE = (*parser).data as *mut BYTE;
            if *data_0.offset((*parser).data_size.wrapping_sub(1 as DWORD) as isize)
                as ::core::ffi::c_int
                != 0 as BYTE as ::core::ffi::c_int
            {
                *data_0.offset((*parser).data_size as isize) = 0 as BYTE;
                (*parser).data_size = (*parser).data_size.wrapping_add(1);
            }
            (*parser).data = GetWideStringN(
                (*parser).data as *const ::core::ffi::c_char,
                (*parser).data_size as ::core::ffi::c_int,
                &raw mut (*parser).data_size,
            ) as *mut ::core::ffi::c_void;
            (*parser).data_size = ((*parser).data_size as ::core::ffi::c_ulonglong)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong)
                as DWORD as DWORD;
            free(data_0 as *mut ::core::ffi::c_void);
        }
    }
}

unsafe extern "C" fn parse_file_header(mut s: *const WCHAR) -> reg_versions {
    static header_31: [u16; 8] = [0x0052u16, 0x0045, 0x0047, 0x0045, 0x0044, 0x0049, 0x0054, 0x0000];

    while *s == ' ' as u16 || *s == '\t' as u16 {
        s = s.offset(1);
    }

    if lstrcmpW(s, header_31.as_ptr()) == 0 {
        return (REG_VERSION_31) as reg_versions;
    }

    if lstrcmpW(s, RUSTCONV_WSTR_D392DA5B86CA.as_ptr()) == 0 {
        return (REG_VERSION_40) as reg_versions;
    }

    if lstrcmpW(s, RUSTCONV_WSTR_994915F97EDD.as_ptr()) == 0 {
        return (REG_VERSION_50) as reg_versions;
    }

    /* The Windows version accepts registry file headers beginning with "REGEDIT" and ending
     * with other characters, as long as "REGEDIT" appears at the start of the line. For example,
     * "REGEDIT 4", "REGEDIT9" and "REGEDIT4FOO" are all treated as valid file headers.
     * In all such cases, however, the contents of the registry file are not imported.
     */
    if wcsncmp(s, header_31.as_ptr(), 7 as usize) == 0 { /* "REGEDIT" without NUL */
        return (REG_VERSION_FUZZY) as reg_versions;
    }

    return (REG_VERSION_INVALID) as reg_versions;
}

unsafe extern "C" fn header_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR;
    let mut header: *mut WCHAR;

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        return core::ptr::null_mut();
    }

    if (*parser).is_unicode == 0 {
        header = malloc(
            ((lstrlenW(line as LPCWSTR) + 3) as usize)
                * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        *header.offset(0) = (*parser).two_wchars[0];
        *header.offset(1) = (*parser).two_wchars[1];
        lstrcpyW(header.offset(2), line as LPCWSTR);
        (*parser).reg_version = parse_file_header(header) as i16;
        free(header as *mut core::ffi::c_void);
    } else {
        (*parser).reg_version = parse_file_header(line) as i16;
    }

    match (*parser).reg_version as i32 {
        REG_VERSION_31 => {
            set_state(parser, (PARSE_WIN31_LINE) as parser_state);
        }
        REG_VERSION_40 | REG_VERSION_50 => {
            set_state(parser, (LINE_START) as parser_state);
        }
        _ => {
            (get_line.unwrap_unchecked())(core::ptr::null_mut());
            return core::ptr::null_mut();
        }
    }

    return line;
}

unsafe extern "C" fn parse_win31_line_state(parser: *mut parser, mut pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut();
    let mut value: *mut WCHAR = core::ptr::null_mut();
    static mut hkcr: [WCHAR; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
    let mut key_end: u32 = 0;

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        return core::ptr::null_mut();
    }

    if wcsncmp(
        line,
        core::ptr::addr_of_mut!(hkcr) as *mut WCHAR,
        lstrlenW(core::ptr::addr_of_mut!(hkcr) as *mut WCHAR) as usize,
    ) != 0
    {
        return line;
    }

    /* get key name */
    while *line.offset(key_end as isize) != 0
        && iswspace(*line.offset(key_end as isize)) == 0
    {
        key_end = key_end.wrapping_add(1);
    }

    value = line.offset(key_end as isize);
    while *value == ' ' as WCHAR || *value == '\t' as WCHAR {
        value = value.offset(1);
    }

    if *value == '=' as WCHAR {
        value = value.offset(1);
    }
    if *value == ' ' as WCHAR {
        value = value.offset(1); /* at most one space is skipped */
    }

    *line.offset(key_end as isize) = 0;

    if (open_key(parser, line)) as u32 != ERROR_SUCCESS {
        output_message(STRING_OPEN_KEY_FAILED, line);
        return line;
    }

    (*parser).value_name = core::ptr::null_mut();
    (*parser).data_type = REG_SZ;
    (*parser).data = value as *mut core::ffi::c_void;
    (*parser).data_size = ((lstrlenW(value) + 1) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;

    set_state(parser, (SET_VALUE) as parser_state);
    return value;
}

unsafe extern "C" fn line_start_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut();
    let mut p: *mut WCHAR = core::ptr::null_mut();

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        return core::ptr::null_mut();
    }

    p = line;
    while *p != 0 {
        match *p as i32 {
            // '['
            91 => {
                set_state(parser, (KEY_NAME) as parser_state);
                return p.offset(1 as isize);
            }
            // '@'
            64 => {
                set_state(parser, (DEFAULT_VALUE_NAME) as parser_state);
                return p;
            }
            // '"'
            34 => {
                set_state(parser, (QUOTED_VALUE_NAME) as parser_state);
                return p.offset(1 as isize);
            }
            // ' ' and '\t'
            32 | 9 => {}
            _ => return p,
        }
        p = p.offset(1);
    }

    p
}

unsafe extern "C" fn key_name_state(mut parser: *mut parser, mut pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    let mut key_end: *mut WCHAR = core::ptr::null_mut();
    'done: {
        if *p == ' ' as WCHAR || *p == '\t' as WCHAR {
            break 'done;
        }
        key_end = wcsrchr(p, ']' as WCHAR) as *mut WCHAR;
        if key_end.is_null() {
            break 'done;
        }

        *key_end = 0 as WCHAR;

        if *p == '-' as WCHAR {
            set_state(parser, (DELETE_KEY) as parser_state);
            return p.offset(1 as isize);
        } else if open_key(parser, p) != ERROR_SUCCESS as LONG {
            output_message(STRING_OPEN_KEY_FAILED as ::core::ffi::c_uint, p);
        }
    }
    set_state(parser, (LINE_START) as parser_state);
    return p;
}

unsafe extern "C" fn delete_key_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    close_key(parser);
    if *p == 'H' as u16 || *p == 'h' as u16 {
        delete_registry_key(p);
    }
    set_state(parser, (LINE_START) as parser_state);
    return p;
}

unsafe extern "C" fn default_value_name_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    free((*parser).value_name as *mut ::core::ffi::c_void);
    (*parser).value_name = ::core::ptr::null_mut::<WCHAR>();

    set_state(parser, (DATA_START) as parser_state);
    return pos.offset(1 as isize);
}

unsafe extern "C" fn quoted_value_name_state(
    mut parser: *mut parser,
    mut pos: *mut WCHAR,
) -> *mut WCHAR {
    let mut val_name: *mut WCHAR = pos;
    let mut p: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    free((*parser).value_name as *mut core::ffi::c_void);
    (*parser).value_name = core::ptr::null_mut::<WCHAR>();
    if REGPROC_unescape_string(val_name, core::ptr::addr_of_mut!(p)) == 0 {
        set_state(parser, (LINE_START) as parser_state);
        return val_name;
    } else {
        (*parser).value_name = malloc(
            ((lstrlenW(val_name as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        lstrcpyW(
            (*parser).value_name as LPWSTR,
            val_name as LPCWSTR,
        );
        set_state(parser, (DATA_START) as parser_state);
        return p;
    };
}

unsafe extern "C" fn data_start_state(mut parser: *mut parser, mut pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    let mut len: u32 = 0;
    'done: {
        while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
            p = p.add(1);
        }
        if *p != '=' as WCHAR {
            break 'done;
        }
        p = p.add(1);
        while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
            p = p.add(1);
        }

        /* trim trailing whitespace */
        len = lstrlenW(p as LPCWSTR) as u32;
        while len > 0
            && (*p.add((len - 1) as usize) == ' ' as WCHAR
                || *p.add((len - 1) as usize) == '\t' as WCHAR)
        {
            len -= 1;
        }
        *p.add(len as usize) = 0 as WCHAR;

        if *p == '-' as WCHAR {
            set_state(parser, (DELETE_VALUE) as parser_state);
        } else {
            set_state(parser, (DATA_TYPE) as parser_state);
        }
        return p;
    }
    set_state(parser, (LINE_START) as parser_state);
    p
}

unsafe extern "C" fn delete_value_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos.offset(1 as isize);

    while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
        p = p.offset(1);
    }
    'done: {
        if *p != 0 && *p != ';' as WCHAR {
            break 'done;
        }

        RegDeleteValueW((*parser).hkey, (*parser).value_name as LPCWSTR);
    }

    set_state(parser, (LINE_START) as parser_state);
    return p;
}

unsafe extern "C" fn data_type_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = pos;

    if parse_data_type(parser, core::ptr::addr_of_mut!(line)) == 0 {
        set_state(parser, (LINE_START) as parser_state);
        return line;
    }

    match (*parser).parse_type {
        REG_SZ => {
            set_state(parser, (STRING_DATA) as parser_state);
        }
        REG_DWORD => {
            set_state(parser, (DWORD_DATA) as parser_state);
        }
        REG_BINARY => {
            /* all hex data types, including undefined */
            set_state(parser, (HEX_DATA) as parser_state);
        }
        _ => {
            set_state(parser, (UNKNOWN_DATA) as parser_state);
        }
    }

    return line;
}

unsafe extern "C" fn string_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    (*parser).data = pos as *mut core::ffi::c_void;

    'invalid: {
        if REGPROC_unescape_string((*parser).data as *mut WCHAR, core::ptr::addr_of_mut!(line)) == 0 {
            break 'invalid;
        }

        while *line == ' ' as WCHAR || *line == '\t' as WCHAR {
            line = line.offset(1);
        }
        if *line != 0 && *line != ';' as WCHAR {
            break 'invalid;
        }

        (*parser).data_size = ((lstrlenW((*parser).data as LPCWSTR)
            + 1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            as DWORD;

        set_state(parser, (SET_VALUE) as parser_state);
        return line;
    }

    free_parser_data(parser);
    set_state(parser, (LINE_START) as parser_state);
    return line;
}

unsafe extern "C" fn dword_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let line: *mut WCHAR = pos;

    (*parser).data = malloc(core::mem::size_of::<DWORD>() as usize);

    if convert_hex_to_dword(line, (*parser).data as *mut DWORD) == 0 {
        // goto invalid
        free_parser_data(parser);
        set_state(parser, (LINE_START) as parser_state);
        return line;
    }

    (*parser).data_size = core::mem::size_of::<DWORD>() as DWORD;

    set_state(parser, (SET_VALUE) as parser_state);
    return line;
}

unsafe extern "C" fn hex_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = pos;
    'quit: {
        'invalid: {
            'set_value: {
                if *line == 0 {
                    break 'set_value;
                }
                if convert_hex_csv_to_hex(parser, core::ptr::addr_of_mut!(line)) == 0 {
                    break 'invalid;
                }
                if (*parser).backslash != 0 {
                    set_state(parser, (EOL_BACKSLASH) as parser_state);
                    break 'quit;
                }
                prepare_hex_string_data(parser);
            }
            // set_value:
            set_state(parser, (SET_VALUE) as parser_state);
            break 'quit;
        }
        // invalid:
        free_parser_data(parser);
        set_state(parser, (LINE_START) as parser_state);
    }
    return line;
}

unsafe extern "C" fn eol_backslash_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
        p = p.offset(1);
    }
    'invalid: {
        if *p != 0 && *p != ';' as WCHAR {
            break 'invalid;
        }
        set_state(parser, (HEX_MULTILINE) as parser_state);
        return pos;
    }
    free_parser_data(parser);
    set_state(parser, (LINE_START) as parser_state);
    return p;
}

unsafe extern "C" fn hex_multiline_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut();

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        prepare_hex_string_data(parser);
        set_state(parser, (SET_VALUE) as parser_state);
        return pos;
    }

    while *line == ' ' as u16 || *line == '\t' as u16 {
        line = line.offset(1);
    }
    if *line == 0 || *line == ';' as u16 {
        return line;
    }

    if iswxdigit(*line) == 0 {
        // invalid:
        free_parser_data(parser);
        set_state(parser, (LINE_START) as parser_state);
        return line;
    }

    set_state(parser, (HEX_DATA) as parser_state);
    return line;
}

unsafe extern "C" fn unknown_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    output_message(STRING_UNKNOWN_DATA_FORMAT, (*parser).data_type);

    set_state(parser, (LINE_START) as parser_state);
    return pos;
}

unsafe extern "C" fn set_value_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    RegSetValueExW(
        (*parser).hkey,
        (*parser).value_name as LPCWSTR,
        0 as DWORD,
        (*parser).data_type,
        (*parser).data as *const BYTE,
        (*parser).data_size,
    );

    free_parser_data(parser);

    if (*parser).reg_version as ::core::ffi::c_int == REG_VERSION_31 as ::core::ffi::c_int {
        set_state(parser, (PARSE_WIN31_LINE) as parser_state);
    } else {
        set_state(parser, (LINE_START) as parser_state);
    }

    return pos;
}

unsafe extern "C" fn get_lineA(fp: *mut FILE) -> *mut WCHAR {
    static mut lineW: *mut WCHAR = core::ptr::null_mut();
    static mut size: usize = 0;
    static mut buf: *mut core::ffi::c_char = core::ptr::null_mut();
    static mut next: *mut core::ffi::c_char = core::ptr::null_mut();
    let mut line: *mut core::ffi::c_char = core::ptr::null_mut();

    free(lineW as *mut core::ffi::c_void);

    'cleanup: {
        if fp.is_null() {
            break 'cleanup;
        }

        if size == 0 {
            size = REG_VAL_BUF_SIZE as usize;
            buf = malloc(size) as *mut core::ffi::c_char;
            *buf = 0 as core::ffi::c_char;
            next = buf;
        }
        line = next;

        while !next.is_null() {
            let mut p: *mut core::ffi::c_char =
                (strpbrk(line as *const u8, (b"\r\n\0" as *const u8 as *const core::ffi::c_char).cast())).cast();
            if p.is_null() {
                let mut len: usize = 0;
                let mut count: usize = 0;
                len = strlen(next) as usize;
                memmove(
                    buf as *mut core::ffi::c_void,
                    next as *const core::ffi::c_void,
                    len.wrapping_add(1 as usize),
                );
                if size.wrapping_sub(len) < (3 as usize) {
                    size = size.wrapping_mul(2 as usize);
                    buf = realloc(buf as *mut core::ffi::c_void, size)
                        as *mut core::ffi::c_char;
                }
                count = fread(
                    buf.offset(len as isize) as *mut core::ffi::c_void,
                    1 as usize,
                    size
                        .wrapping_sub(len)
                        .wrapping_sub(1 as usize),
                    fp,
                ) as usize;
                if count == 0 {
                    next = core::ptr::null_mut();
                    lineW = GetWideString(buf);
                    return lineW;
                }
                *buf.offset(len.wrapping_add(count) as isize) =
                    0 as core::ffi::c_char;
                next = buf;
                line = buf;
                continue;
            }
            next = p.offset(1 as isize);
            if *p as core::ffi::c_int == '\r' as core::ffi::c_int
                && *p.offset(1 as isize) as core::ffi::c_int
                    == '\n' as core::ffi::c_int
            {
                next = next.offset(1);
            }
            *p = 0 as core::ffi::c_char;
            lineW = GetWideString(line);
            return lineW;
        }
    }

    lineW = core::ptr::null_mut();
    if size != 0 {
        free(buf as *mut core::ffi::c_void);
    }
    size = 0 as usize;
    return core::ptr::null_mut();
}

unsafe extern "C" fn get_lineW(mut fp: *mut FILE) -> *mut WCHAR {
    static mut size: size_t = 0;
    static mut buf: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    static mut next: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut line: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    'cleanup: {
        if fp.is_null() {
            break 'cleanup;
        }

        if size == 0 {
            size = REG_VAL_BUF_SIZE as size_t;
            buf = malloc(size.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t))
                as *mut WCHAR;
            *buf = 0 as WCHAR;
            next = buf;
        }
        line = next;

        while !next.is_null() {
            let mut p: *mut WCHAR = wcspbrk(line, RUSTCONV_WSTR_80E09B2F645F.as_ptr()) as *mut WCHAR;
            if p.is_null() {
                let mut len: size_t = 0;
                let mut count: size_t = 0;
                len = lstrlenW(next as *const WCHAR) as size_t;
                memmove(
                    buf as *mut core::ffi::c_void,
                    next as *const core::ffi::c_void,
                    len.wrapping_add(1 as size_t)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                );
                if size.wrapping_sub(len) < 3 as size_t {
                    size = size.wrapping_mul(2 as size_t);
                    buf = realloc(
                        buf as *mut core::ffi::c_void,
                        size.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                    ) as *mut WCHAR;
                }
                count = fread(
                    buf.offset(len as isize) as *mut core::ffi::c_void,
                    core::mem::size_of::<WCHAR>() as size_t,
                    size
                        .wrapping_sub(len)
                        .wrapping_sub(1 as size_t),
                    fp,
                ) as size_t;
                if count == 0 {
                    next = core::ptr::null_mut::<WCHAR>();
                    return buf;
                }
                *buf.offset(len.wrapping_add(count) as isize) = 0 as WCHAR;
                next = buf;
                line = buf;
                continue;
            }
            next = p.offset(1 as isize);
            if *p == '\r' as WCHAR && *p.offset(1 as isize) == '\n' as WCHAR {
                next = next.offset(1);
            }
            *p = 0 as WCHAR;
            return line;
        }
    }

    // cleanup:
    if size != 0 {
        free(buf as *mut core::ffi::c_void);
    }
    size = 0 as size_t;
    return core::ptr::null_mut::<WCHAR>();
}
#[no_mangle]
pub unsafe extern "C" fn import_registry_file(mut reg_file: *mut FILE) -> BOOL {
    let mut s: [BYTE; 2] = [0; 2];
    let mut parser: parser = parser {
        file: ::core::ptr::null_mut::<FILE>(),
        two_wchars: [0; 2],
        is_unicode: 0,
        reg_version: 0,
        hkey: ::core::ptr::null_mut::<HKEY__>(),
        key_name: ::core::ptr::null_mut::<WCHAR>(),
        value_name: ::core::ptr::null_mut::<WCHAR>(),
        parse_type: 0,
        data_type: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        data_size: 0,
        backslash: 0,
        unicode_in_asc: 0,
        state: HEADER,
    };
    let mut pos: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

    if reg_file.is_null()
        || fread(
            s.as_mut_ptr() as *mut ::core::ffi::c_void,
            2,
            1,
            reg_file,
        ) != 1
    {
        return (FALSE) as BOOL;
    }

    parser.is_unicode = (s[0] as ::core::ffi::c_int
        == 0xff as ::core::ffi::c_int
        && s[1] as ::core::ffi::c_int
            == 0xfe as ::core::ffi::c_int)
        as ::core::ffi::c_int as BOOL;
    get_line = if parser.is_unicode != 0 {
        Some(get_lineW as unsafe extern "C" fn(*mut FILE) -> *mut WCHAR)
    } else {
        Some(get_lineA as unsafe extern "C" fn(*mut FILE) -> *mut WCHAR)
    };

    parser.file = reg_file;
    parser.two_wchars[0] = s[0] as WCHAR;
    parser.two_wchars[1] = s[1] as WCHAR;
    parser.reg_version = -(1 as ::core::ffi::c_int) as ::core::ffi::c_short;
    parser.hkey = ::core::ptr::null_mut::<HKEY__>();
    parser.key_name = ::core::ptr::null_mut::<WCHAR>();
    parser.value_name = ::core::ptr::null_mut::<WCHAR>();
    parser.parse_type = 0 as DWORD;
    parser.data_type = 0 as DWORD;
    parser.data = NULL;
    parser.data_size = 0 as DWORD;
    parser.backslash = FALSE as BOOL;
    parser.state = HEADER;

    pos = &raw mut parser.two_wchars as *mut WCHAR;

    /* parser main loop */
    while !pos.is_null() {
        pos = parser_funcs[parser.state as usize].unwrap_unchecked()(
            &raw mut parser,
            pos,
        );
    }

    if parser.reg_version as ::core::ffi::c_int == REG_VERSION_FUZZY as ::core::ffi::c_int
        || parser.reg_version as ::core::ffi::c_int == REG_VERSION_INVALID as ::core::ffi::c_int
    {
        return (parser.reg_version as ::core::ffi::c_int
            == REG_VERSION_FUZZY as ::core::ffi::c_int) as ::core::ffi::c_int;
    }

    free(parser.value_name as *mut ::core::ffi::c_void);
    close_key(&raw mut parser);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn delete_registry_key(mut reg_key_name: *mut WCHAR) {
    let mut key_name: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut key_class: HKEY = core::ptr::null_mut::<HKEY__>();
    if reg_key_name.is_null() || *reg_key_name.offset(0 as isize) == 0 {
        return;
    }
    key_class = parse_key_name(reg_key_name, core::ptr::addr_of_mut!(key_name));
    if key_class.is_null() {
        if !key_name.is_null() {
            *key_name.offset(-(1 as isize)) = 0 as WCHAR;
        }
        // regproc.c:1125-1147, the `#ifdef __REACTOS__` arms: report and return (Wine's
        // error_exit ends the process) and SHDeleteKey (Wine's RegDeleteTreeW).
        output_message(STRING_INVALID_SYSTEM_KEY as ::core::ffi::c_uint, reg_key_name);
        return;
    }
    if key_name.is_null() || *key_name == 0 {
        output_message(STRING_DELETE_FAILED as ::core::ffi::c_uint, reg_key_name);
        return;
    }
    SHDeleteKeyW(key_class, key_name as LPCWSTR);
}

unsafe extern "C" fn REGPROC_write_line(fp: *mut FILE, str: *const WCHAR, unicode: BOOL) {
    if unicode != 0 {
        fwrite(str as *const core::ffi::c_void, core::mem::size_of::<WCHAR>(), lstrlenW(str as LPCWSTR) as usize, (fp).cast());
    } else {
        let strA: *mut core::ffi::c_char = GetMultiByteString(str);
        fputs(strA, fp);
        free(strA as *mut core::ffi::c_void);
    }
}

unsafe extern "C" fn REGPROC_escape_string(
    mut str: *mut WCHAR,
    mut str_len: size_t,
    mut line_len: *mut size_t,
) -> *mut WCHAR {
    let mut i: size_t;
    let mut escape_count: size_t;
    let mut pos: size_t;
    let mut buf: *mut WCHAR;
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
            .wrapping_add(1)
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
                let fresh32 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh32 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = 'r' as i32 as WCHAR;
            }
            10 => {
                let fresh33 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh33 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = 'n' as i32 as WCHAR;
            }
            92 => {
                let fresh34 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh34 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = '\\' as i32 as WCHAR;
            }
            34 => {
                let fresh35 = pos;
                pos = pos.wrapping_add(1);
                *buf.offset(fresh35 as isize) = '\\' as i32 as WCHAR;
                *buf.offset(pos as isize) = '"' as i32 as WCHAR;
            }
            _ => {
                *buf.offset(pos as isize) = c_0;
            }
        }
        i = i.wrapping_add(1);
        pos = pos.wrapping_add(1);
    }
    *buf.offset(pos as isize) = 0;
    *line_len = pos;
    return buf;
}

unsafe extern "C" fn export_value_name(fp: *mut FILE, name: *mut WCHAR, len: size_t, unicode: BOOL) -> size_t {
    static default_name: [WCHAR; 3] = [0x0040u16, 0x003D, 0x0000];
    let mut line_len: size_t = 0;

    if !name.is_null() && *name != 0 {
        let mut str: *mut WCHAR = REGPROC_escape_string(name, len, &raw mut line_len);
        let mut buf: *mut WCHAR = malloc(
            (line_len + 4) * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        {
            StringCchPrintfW!(buf, line_len + 4, RUSTCONV_WSTR_0561B80DCBE9.as_ptr(), str);
            line_len = wcslen(buf) as size_t;
        }
        REGPROC_write_line(fp, buf, unicode);
        free(buf as *mut core::ffi::c_void);
        free(str as *mut core::ffi::c_void);
    } else {
        line_len = lstrlenW(core::ptr::addr_of!(default_name) as *const WCHAR) as size_t;
        REGPROC_write_line(fp, core::ptr::addr_of!(default_name) as *const WCHAR, unicode);
    }

    return line_len;
}

unsafe extern "C" fn export_string_data(buf: *mut *mut WCHAR, data: *mut WCHAR, size: size_t) {
    let mut len: size_t = 0 as size_t;
    let mut line_len: size_t = 0 as size_t;
    let mut str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if size != 0 as size_t {
        len = (size as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1 as usize) as size_t;
    }
    str = REGPROC_escape_string(data, len, core::ptr::addr_of_mut!(line_len));
    *buf = malloc(
        line_len
            .wrapping_add(3 as size_t)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    {
        StringCchPrintfW!(
            *buf,
            line_len.wrapping_add(3 as size_t),
            RUSTCONV_WSTR_9896841D0A6A.as_ptr(),
            str,
        );
    }
    free(str as *mut core::ffi::c_void);
}

unsafe extern "C" fn export_dword_data(buf: *mut *mut WCHAR, data: *mut DWORD) {
    *buf = malloc(15 as usize * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    StringCchPrintfW!(*buf, 15 as usize, RUSTCONV_WSTR_B23F78685766.as_ptr(), *data);
    #[cfg(any())]
    swprintf(*buf, 15 as usize, RUSTCONV_WSTR_B23F78685766.as_ptr(), *data);
}

unsafe extern "C" fn export_hex_data_type(
    mut fp: *mut FILE,
    mut type_0: DWORD,
    mut unicode: BOOL,
) -> size_t {
    static mut hex: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x003A, 0x0000];
    let mut line_len: size_t = 0;
    if type_0 == REG_BINARY {
        line_len = lstrlenW(core::ptr::addr_of_mut!(hex) as *const u16) as size_t;
        REGPROC_write_line(fp, core::ptr::addr_of_mut!(hex) as *const u16, unicode);
    } else {
        let mut buf: *mut WCHAR =
            malloc(15 * core::mem::size_of::<WCHAR>() as size_t)
                as *mut WCHAR;
        {
            StringCchPrintfW!(buf, 15, RUSTCONV_WSTR_7443311FB636.as_ptr(), type_0);
            line_len = wcslen(buf) as size_t;
        }
        REGPROC_write_line(fp, buf, unicode);
        free(buf as *mut core::ffi::c_void);
    }

    return line_len;
}

unsafe extern "C" fn export_hex_data(
    fp: *mut FILE,
    buf: *mut *mut WCHAR,
    type_: DWORD,
    mut line_len: DWORD,
    mut data: *mut ::core::ffi::c_void,
    mut size: DWORD,
    unicode: BOOL,
) {
    let mut num_commas: size_t = 0;
    let mut i: size_t = 0;
    let mut pos: size_t = 0;

    line_len = (line_len as size_t).wrapping_add(export_hex_data_type(fp, type_, unicode)) as DWORD;

    if size == 0 {
        return;
    }

    if unicode == 0 && (type_ == REG_EXPAND_SZ || type_ == REG_MULTI_SZ) {
        data = GetMultiByteStringN(
            data as *const WCHAR,
            (size as usize).wrapping_div(::core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
            &raw mut size,
        ) as *mut ::core::ffi::c_void;
    }

    num_commas = size.wrapping_sub(1 as DWORD) as size_t;
    *buf = malloc(
        (size.wrapping_mul(3 as DWORD) as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    i = 0 as size_t;
    pos = 0 as size_t;

    while i < size as size_t {
        {
            StringCchPrintfW!(
                (*buf).add(pos),
                3 as size_t,
                RUSTCONV_WSTR_AA2A982F0618.as_ptr(),
                *(data as *mut BYTE).add(i) as ::core::ffi::c_int,
            );
            pos = (pos as ::core::ffi::c_ulonglong)
                .wrapping_add((wcslen((*buf).add(pos))) as _)
                as size_t;
        }
        #[cfg(any())]
        {
            pos = pos.wrapping_add(swprintf(
                (*buf).add(pos),
                3 as size_t,
                RUSTCONV_WSTR_AA2A982F0618.as_ptr(),
                *(data as *mut BYTE).add(i) as ::core::ffi::c_int,
            ) as size_t);
        }

        if i == num_commas {
            break;
        }

        *(*buf).add(pos) = ',' as WCHAR;
        pos += 1;
        *(*buf).add(pos) = 0 as WCHAR;
        line_len = line_len.wrapping_add(3 as DWORD);

        if line_len >= MAX_HEX_CHARS as DWORD {
            REGPROC_write_line(fp, *buf, unicode);
            REGPROC_write_line(fp, RUSTCONV_WSTR_FFFF89D9175A.as_ptr(), unicode);
            line_len = 2 as DWORD;
            pos = 0 as size_t;
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn export_newline(fp: *mut FILE, unicode: BOOL) {
    REGPROC_write_line(fp, RUSTCONV_WSTR_80E09B2F645F.as_ptr() as *const u16, unicode);
}

unsafe extern "C" fn export_data(mut fp: *mut FILE, mut value_name: *mut WCHAR, mut value_len: DWORD, mut type_0: DWORD, mut data: *mut ::core::ffi::c_void, mut size: size_t, mut unicode: BOOL) {
    let mut buf: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut line_len: size_t = export_value_name(fp, value_name, value_len as size_t, unicode);

    match type_0 {
        REG_SZ => {
            export_string_data(&raw mut buf, (data).cast(), size);
        }
        REG_DWORD => {
            if size != 0 {
                export_dword_data(&raw mut buf, data as *mut DWORD);
            } else {
                export_hex_data(fp, &raw mut buf, type_0, line_len as DWORD, data, size as DWORD, unicode);
            }
        }
        REG_NONE | REG_EXPAND_SZ | REG_BINARY | REG_MULTI_SZ | _ => {
            export_hex_data(fp, &raw mut buf, type_0, line_len as DWORD, data, size as DWORD, unicode);
        }
    }

    if size != 0 || type_0 == REG_SZ as DWORD {
        REGPROC_write_line(fp, buf, unicode);
        free(buf as *mut ::core::ffi::c_void);
    }

    export_newline(fp, unicode);
}

unsafe extern "C" fn build_subkey_path(
    mut path: *mut WCHAR,
    mut path_len: DWORD,
    mut subkey_name: *mut WCHAR,
    mut subkey_len: DWORD,
) -> *mut WCHAR {
    let mut subkey_path: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    subkey_path = malloc(
        (path_len.wrapping_add(subkey_len).wrapping_add(2 as DWORD) as size_t)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    {
        StringCchPrintfW!(
            subkey_path,
            path_len.wrapping_add(subkey_len).wrapping_add(2 as DWORD) as size_t,
            RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
            path,
            subkey_name,
        );
    }
    #[cfg(any())]
    {
        swprintf(
            subkey_path,
            path_len.wrapping_add(subkey_len).wrapping_add(2 as DWORD) as size_t,
            RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
            path,
            subkey_name,
        );
    }

    return subkey_path;
}

unsafe extern "C" fn export_key_name(mut fp: *mut FILE, mut name: *mut WCHAR, mut unicode: BOOL) {
    const REACTOS: bool = true;
    let mut buf: *mut WCHAR;
    buf = malloc(
        ((lstrlenW(name) + 7) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    {
        StringCchPrintfW!(
            buf,
            (lstrlenW(name) + 7) as usize,
            RUSTCONV_WSTR_10A8D224346F.as_ptr(),
            name,
        );
    }
    REGPROC_write_line(fp, buf, unicode);
    free(buf as *mut core::ffi::c_void);
}

unsafe extern "C" fn export_registry_data(
    mut fp: *mut FILE,
    mut key: HKEY,
    mut path: *mut WCHAR,
    mut unicode: BOOL,
) {
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

    export_key_name(fp, path, unicode);

    value_name = malloc(
        (max_value_len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    data = malloc(max_data_bytes as size_t) as *mut BYTE;

    i = 0 as DWORD;
    loop {
        value_len = max_value_len;
        data_size = max_data_bytes;
        rc = RegEnumValueW(
            key,
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
                fp,
                value_name,
                value_len,
                type_0,
                data as *mut ::core::ffi::c_void,
                data_size as size_t,
                unicode,
            );
            i = i.wrapping_add(1);
        } else if rc == ERROR_MORE_DATA as LONG {
            if data_size > max_data_bytes {
                max_data_bytes = data_size;
                data = realloc(data as *mut ::core::ffi::c_void, max_data_bytes as size_t)
                    as *mut BYTE;
            } else {
                max_value_len = max_value_len.wrapping_mul(2 as DWORD);
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

    subkey_name = malloc(
        (MAX_SUBKEY_LEN as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    path_len = lstrlenW(path as LPCWSTR) as DWORD;

    i = 0 as DWORD;
    loop {
        subkey_len = MAX_SUBKEY_LEN as DWORD;
        rc = RegEnumKeyExW(
            key,
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
                key,
                subkey_name as LPCWSTR,
                0 as DWORD,
                KEY_READ as REGSAM,
                &raw mut subkey,
            ) == 0
            {
                export_registry_data(fp, subkey, subkey_path, unicode);
                RegCloseKey(subkey);
            }
            free(subkey_path as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        } else {
            break;
        }
    }

    free(subkey_name as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn REGPROC_open_export_file(
    mut file_name: *mut WCHAR,
    mut unicode: BOOL,
) -> *mut FILE {
    const REACTOS: bool = true;
    let mut file: *mut FILE = core::ptr::null_mut::<FILE>();
    if lstrcmpW(file_name as LPCWSTR, RUSTCONV_WSTR_28276425D458.as_ptr()) == 0 {
        file = (__iob_func() as *mut FILE).offset(1) as *mut FILE;
        _setmode(_fileno((file).cast()), (_O_BINARY as i32));
    } else {
        file = (_wfopen(file_name, RUSTCONV_WSTR_1EA9E69A696A.as_ptr())).cast();
        if file.is_null() {
            _wperror(RUSTCONV_WSTR_E89B9A83645C.as_ptr());
            {
                output_message(STRING_CANNOT_OPEN_FILE, file_name);
                return core::ptr::null_mut::<FILE>();
            }
        }
    }

    if unicode != 0 {
        static bom: [BYTE; 2] = [0xff, 0xfe];
        static header: [WCHAR; 39] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0052, 0x0065,
    0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0045, 0x0064, 0x0069,
    0x0074, 0x006F, 0x0072, 0x0020, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0020, 0x0035, 0x002E, 0x0030, 0x0030, 0x000D, 0x000A, 0x0000,
];

        fwrite(
            &raw const bom as *const BYTE as *const core::ffi::c_void,
            core::mem::size_of::<BYTE>() as size_t,
            ARRAY_SIZE!(bom) as size_t,
            (file).cast(),
        );
        fwrite(
            &raw const header as *const WCHAR as *const core::ffi::c_void,
            core::mem::size_of::<WCHAR>() as size_t,
            lstrlenW(&raw const header as *const WCHAR) as size_t,
            (file).cast(),
        );
    } else {
        fputs(
            b"REGEDIT4\r\n\0" as *const u8 as *const core::ffi::c_char,
            file,
        );
    }

    return file;
}

unsafe extern "C" fn open_export_key(key_class: HKEY, subkey: *mut WCHAR, path: *mut WCHAR) -> HKEY {
    let mut key: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        key_class,
        subkey as LPCWSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(key),
    ) == 0
    {
        return key;
    }

    output_message(STRING_OPEN_KEY_FAILED as ::core::ffi::c_uint, path);
    return core::ptr::null_mut();
}

unsafe extern "C" fn export_key(
    mut file_name: *mut WCHAR,
    mut path: *mut WCHAR,
    mut unicode: BOOL,
) -> BOOL {
    let mut key_class: HKEY = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut subkey: *mut WCHAR = core::ptr::null_mut();
    let mut fp: *mut FILE = core::ptr::null_mut();

    key_class = parse_key_name(path, core::ptr::addr_of_mut!(subkey));
    if key_class.is_null() {
        if !subkey.is_null() {
            *subkey.offset(-(1 as isize)) = 0;
        }
        output_message(STRING_INVALID_SYSTEM_KEY, path);
        return (FALSE) as BOOL;
    }

    key = open_export_key(key_class, subkey, path);
    if key.is_null() {
        return (FALSE) as BOOL;
    }

    fp = REGPROC_open_export_file(file_name, unicode);
    if fp.is_null() {
        return (TRUE) as BOOL;
    }
    export_registry_data(fp, key, path, unicode);
    export_newline(fp, unicode);
    fclose(fp);

    RegCloseKey(key);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn export_all(mut file_name: *mut WCHAR, mut path: *mut WCHAR, mut unicode: BOOL) -> BOOL {
    let mut fp: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut i: ::core::ffi::c_int = 0;
    let mut classes: [HKEY; 2] = [HKEY_LOCAL_MACHINE, HKEY_USERS];
    let mut key: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut class_name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    fp = REGPROC_open_export_file(file_name, unicode);
    if fp.is_null() {
        return (TRUE) as BOOL;
    }
    i = 0 as ::core::ffi::c_int;
    while (i as usize) < ARRAY_SIZE!(classes) as usize {
        key = open_export_key(classes[i as usize], ::core::ptr::null_mut::<WCHAR>(), path);
        if key.is_null() {
            fclose(fp);
            return (FALSE) as BOOL;
        }
        class_name = malloc(
            ((lstrlenW(
                *(&raw mut reg_class_namesW as *mut *const WCHAR).offset(i as isize) as LPCWSTR,
            ) + 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        lstrcpyW(
            class_name as LPWSTR,
            *(&raw mut reg_class_namesW as *mut *const WCHAR).offset(i as isize) as LPCWSTR,
        );
        export_registry_data(fp, classes[i as usize], class_name, unicode);
        free(class_name as *mut ::core::ffi::c_void);
        RegCloseKey(key);
        i += 1;
    }
    export_newline(fp, unicode);
    fclose(fp);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn export_registry_key(file_name: *mut WCHAR, path: *mut WCHAR, format: DWORD) -> BOOL {
    let unicode: BOOL = (format == REG_FORMAT_5) as BOOL;

    if !path.is_null() && *path != 0 {
        return export_key(file_name, path, unicode);
    } else {
        return export_all(file_name, path, unicode);
    }
}
