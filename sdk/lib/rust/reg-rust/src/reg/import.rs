use crate::reg::reg::path_get_rootkey;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_097381A2F37D: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_1B9334FEECE6: [u16; 2] = [0x0022u16, 0x0000];
pub static RUSTCONV_WSTR_43425498687F: [u16; 4] = [0x0030u16, 0x0030, 0x002C, 0x0000];
pub static RUSTCONV_WSTR_5461380CE9C5: [u16; 8] = [0x0052u16, 0x0045, 0x0047, 0x0045, 0x0044, 0x0049, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_76CFE90FF2FC: [u16; 3] = [0x0072u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_87D16031626E: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_994915F97EDD: [u16; 37] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0052, 0x0065,
    0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x0020, 0x0045, 0x0064, 0x0069,
    0x0074, 0x006F, 0x0072, 0x0020, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0020, 0x0035, 0x002E, 0x0030, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_A7125E3ED5E3: [u16; 7] = [0x0064u16, 0x0077, 0x006F, 0x0072, 0x0064, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_C6E98D2C3B58: [u16; 5] = [0x0068u16, 0x0065, 0x0078, 0x0028, 0x0000];
pub static RUSTCONV_WSTR_D392DA5B86CA: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x0045, 0x0044, 0x0049, 0x0054, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
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

pub const HEADER: parser_state = 0;
pub const PARSE_WIN31_LINE: parser_state = 1;
pub const LINE_START: parser_state = 2;
pub const KEY_NAME: parser_state = 3;
pub const DELETE_KEY: parser_state = 4;
pub const DEFAULT_VALUE_NAME: parser_state = 5;
pub const QUOTED_VALUE_NAME: parser_state = 6;
pub const DATA_START: parser_state = 7;
pub const DELETE_VALUE: parser_state = 8;
pub const DATA_TYPE: parser_state = 9;
pub const STRING_DATA: parser_state = 10;
pub const DWORD_DATA: parser_state = 11;
pub const HEX_DATA: parser_state = 12;
pub const EOL_BACKSLASH: parser_state = 13;
pub const HEX_MULTILINE: parser_state = 14;
pub const UNKNOWN_DATA: parser_state = 15;
pub const SET_VALUE: parser_state = 16;
pub const NB_PARSER_STATES: parser_state = 17;
pub type parser_state = i32;

#[repr(C)]
pub struct parser {
    pub file: *mut FILE,
    pub two_wchars: [WCHAR; 2],
    pub is_unicode: BOOL,
    pub reg_version: i16,
    pub sam: REGSAM,
    pub hkey: HKEY,
    pub key_name: *mut WCHAR,
    pub value_name: *mut WCHAR,
    pub parse_type: DWORD,
    pub data_type: DWORD,
    pub data: *mut core::ffi::c_void,
    pub data_size: DWORD,
    pub backslash: BOOL,
    pub state: parser_state,
}

// C: `typedef WCHAR *(*parser_state_func)(struct parser *parser, WCHAR *pos);` -- a nullable
// function pointer, which the table's `Some(..)` and the dispatch's unwrap already assume.
pub type parser_state_func = Option<unsafe extern "C" fn(*mut parser, *mut WCHAR) -> *mut WCHAR>;

pub const REG_VERSION_31: i32 = 0;
pub const REG_VERSION_40: i32 = 1;
pub const REG_VERSION_50: i32 = 2;
pub const REG_VERSION_FUZZY: i32 = 3;
pub const REG_VERSION_INVALID: i32 = 4;

pub const REG_VAL_BUF_SIZE: usize = 4096;

unsafe extern "C" fn GetWideString(strA: *const ::core::ffi::c_char) -> *mut WCHAR {
    if !strA.is_null() {
        let mut strW: *mut WCHAR = core::ptr::null_mut();
        let len: ::core::ffi::c_int = MultiByteToWideChar(CP_ACP, 0, strA, -1, core::ptr::null_mut(), 0);

        strW = malloc((len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        MultiByteToWideChar(CP_ACP, 0, strA, -1, strW, len);
        return strW;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn GetWideStringN(strA: *const ::core::ffi::c_char, size: ::core::ffi::c_int, len: *mut DWORD) -> *mut WCHAR {
    if !strA.is_null() {
        let mut strW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        *len = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, strA, size, ::core::ptr::null_mut::<WCHAR>(), 0 as ::core::ffi::c_int) as DWORD;

        strW = malloc((*len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>())) as *mut WCHAR;
        MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, strA, size, strW, *len as ::core::ffi::c_int);
        return strW;
    }
    *len = 0 as DWORD;
    return ::core::ptr::null_mut::<WCHAR>();
}

unsafe extern "C" fn set_state(parser: *mut parser, state: parser_state) -> parser_state {
    let mut ret: parser_state = (*parser).state;
    (*parser).state = state;
    return ret;
}

unsafe extern "C" fn convert_hex_to_dword(mut str: *mut WCHAR, mut dw: *mut DWORD) -> BOOL {
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut end: *mut WCHAR = core::ptr::null_mut();
    let mut count: i32 = 0;

    while *str == ' ' as WCHAR || *str == '\t' as WCHAR {
        str = str.add(1);
    }
    'error: {
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

unsafe extern "C" fn convert_hex_csv_to_hex(
    parser: *mut parser,
    str: *mut *mut WCHAR,
) -> BOOL {
    let mut size: size_t;
    let mut d: *mut BYTE;
    let mut s: *mut WCHAR;

    (*parser).backslash = (FALSE) as i32;

    /* The worst case is 1 digit + 1 comma per byte */
    size = (((lstrlenW(*str) + 1) / 2) as DWORD) as _;
    size = size.wrapping_add(((*parser).data_size) as _) as size_t;
    (*parser).data = realloc((*parser).data, size);

    s = *str;
    d = ((*parser).data as *mut BYTE).offset((*parser).data_size as isize);

    while *s != 0 {
        let mut end: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        let mut wc: ::core::ffi::c_ulong = 0;

        wc = wcstoul(s, ::core::ptr::addr_of_mut!(end), 16);
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

        let fresh15 = d;
        d = d.offset(1);
        *fresh15 = wc as BYTE;
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
    static mut data_types: [data_type; 5] = unsafe {
        [
            data_type {
                tag: RUSTCONV_WSTR_1B9334FEECE6.as_ptr(),
                len: 1,
                type_: (REG_SZ) as i32,
                parse_type: (REG_SZ) as i32,
            },
            data_type {
                tag: RUSTCONV_WSTR_097381A2F37D.as_ptr(),
                len: 4,
                type_: (REG_BINARY) as i32,
                parse_type: (REG_BINARY) as i32,
            },
            data_type {
                tag: RUSTCONV_WSTR_A7125E3ED5E3.as_ptr(),
                len: 6,
                type_: (REG_DWORD) as i32,
                parse_type: (REG_DWORD) as i32,
            },
            data_type {
                tag: RUSTCONV_WSTR_C6E98D2C3B58.as_ptr(),
                len: 4,
                type_: -1,
                parse_type: (REG_BINARY) as i32,
            },
            data_type {
                tag: core::ptr::null::<WCHAR>(),
                len: 0,
                type_: 0,
                parse_type: 0,
            },
        ]
    };
    let mut ptr: *const data_type = core::ptr::null::<data_type>();
    ptr = core::ptr::addr_of!(data_types) as *const data_type;
    while !(*ptr).tag.is_null() {
        if wcsncmp((*ptr).tag as *const wchar_t, *line, (*ptr).len as size_t) != 0 {
            ptr = ptr.offset(1);
        } else {
            (*parser).parse_type = (*ptr).parse_type as DWORD;
            (*parser).data_type = (*ptr).parse_type as DWORD;
            *line = (*line).offset((*ptr).len as isize);
            if (*ptr).type_ == -1 {
                let mut end: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
                let mut val: DWORD = 0;
                if **line == 0
                    || towlower(*(*line).offset(1 as isize) as wint_t)
                        as ::core::ffi::c_int
                        == 'x' as i32
                {
                    return (FALSE) as BOOL;
                }
                /* "hex(xx):" is special */
                val = wcstoul(*line, core::ptr::addr_of_mut!(end), 16 as ::core::ffi::c_int) as DWORD;
                /* Up to 8 hex digits, "hex(000000002)" is invalid */
                if *end as ::core::ffi::c_int != ')' as i32
                    || *end.offset(1 as isize) as ::core::ffi::c_int
                        != ':' as i32
                    || val == !(0 as DWORD) && (*_errno()) as u32 == ERANGE
                    || end.offset_from(*line) as ::core::ffi::c_longlong
                        > 8 as ::core::ffi::c_longlong
                {
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

unsafe extern "C" fn unescape_string(mut str: *mut WCHAR, mut unparsed: *mut *mut WCHAR) -> BOOL {
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
                    *str.offset(val_idx as isize) = '\n' as i32 as WCHAR;
                }
                114 => {
                    *str.offset(val_idx as isize) = '\r' as i32 as WCHAR;
                }
                48 => return (FALSE) as i32,
                92 | 34 => {
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
        } else {
            if *str.offset(str_idx as isize) as ::core::ffi::c_int == '"' as i32 {
                break;
            }
            *str.offset(val_idx as isize) = *str.offset(str_idx as isize);
        }
        str_idx += 1;
        val_idx += 1;
    }
    ret = (*str.offset(str_idx as isize) as ::core::ffi::c_int == '"' as i32) as ::core::ffi::c_int
        as BOOL;
    *unparsed = str
        .offset(str_idx as isize)
        .offset(1 as isize);
    *str.offset(val_idx as isize) = '\0' as i32 as WCHAR;
    return ret;
}

unsafe extern "C" fn parse_key_name(mut key_name: *mut WCHAR, mut key_path: *mut *mut WCHAR) -> HKEY {
    if key_name.is_null() {
        return core::ptr::null_mut();
    }
    *key_path = wcschr(key_name, '\\' as u16) as *mut WCHAR;
    if !(*key_path).is_null() {
        *key_path = (*key_path).offset(1);
    }
    return path_get_rootkey(key_name);
}

unsafe extern "C" fn close_key(parser: *mut parser) {
    if !(*parser).hkey.is_null() {
        free((*parser).key_name as *mut ::core::ffi::c_void);
        (*parser).key_name = core::ptr::null_mut();

        RegCloseKey((*parser).hkey);
        (*parser).hkey = core::ptr::null_mut();
    }
}

unsafe extern "C" fn open_key(parser: *mut parser, path: *mut WCHAR) -> LONG {
    let mut key_class: HKEY = core::ptr::null_mut();
    let mut key_path: *mut WCHAR = core::ptr::null_mut();
    let mut res: LONG;

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
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE as DWORD,
        (KEY_ALL_ACCESS as REGSAM) | (*parser).sam,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*parser).hkey),
        core::ptr::null_mut(),
    ) as LONG;

    if res == ERROR_SUCCESS as LONG {
        (*parser).key_name = malloc(
            ((lstrlenW(path as LPCWSTR) + 1) as size_t)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        lstrcpyW((*parser).key_name as LPWSTR, path as LPCWSTR);
    } else {
        (*parser).hkey = core::ptr::null_mut();
    }

    return res;
}

unsafe extern "C" fn free_parser_data(parser: *mut parser) {
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
        if (*parser).is_unicode != 0 {
            let mut data: *mut WCHAR = (*parser).data as *mut WCHAR;
            let mut len: DWORD = ((*parser).data_size as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as DWORD;
            if *data.offset(len.wrapping_sub(1 as DWORD) as isize)
                as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
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
                != 0 as ::core::ffi::c_int
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
    static mut header_31: *const WCHAR = RUSTCONV_WSTR_5461380CE9C5.as_ptr();

    while *s as ::core::ffi::c_int == ' ' as i32 || *s as ::core::ffi::c_int == '\t' as i32 {
        s = s.offset(1);
    }

    if lstrcmpW(s as LPCWSTR, header_31 as LPCWSTR) == 0 {
        return (REG_VERSION_31) as reg_versions;
    }

    if lstrcmpW(s as LPCWSTR, RUSTCONV_WSTR_D392DA5B86CA.as_ptr() as LPCWSTR) == 0 {
        return (REG_VERSION_40) as reg_versions;
    }

    if lstrcmpW(s as LPCWSTR, RUSTCONV_WSTR_994915F97EDD.as_ptr() as LPCWSTR) == 0 {
        return (REG_VERSION_50) as reg_versions;
    }

    /* The Windows version accepts registry file headers beginning with "REGEDIT" and ending
     * with other characters, as long as "REGEDIT" appears at the start of the line. For example,
     * "REGEDIT 4", "REGEDIT9" and "REGEDIT4FOO" are all treated as valid file headers.
     * In all such cases, however, the contents of the registry file are not imported.
     */
    if wcsncmp(
        s as *const wchar_t,
        header_31 as *const wchar_t,
        7 as size_t,
    ) == 0
    {
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
            ((lstrlenW(line as LPCWSTR) + 3 as i32) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        *header.offset(0 as isize) =
            (*parser).two_wchars[0 as usize];
        *header.offset(1 as isize) =
            (*parser).two_wchars[1 as usize];
        lstrcpyW(
            header.offset(2 as isize),
            line as LPCWSTR,
        );
        (*parser).reg_version = parse_file_header(header as *const WCHAR) as i16;
        free(header as *mut core::ffi::c_void);
    } else {
        (*parser).reg_version = parse_file_header(line as *const WCHAR) as i16;
    }

    match (*parser).reg_version as i32 {
        REG_VERSION_31 => {
            set_state(parser, PARSE_WIN31_LINE);
        }
        REG_VERSION_40 | REG_VERSION_50 => {
            set_state(parser, LINE_START);
        }
        _ => {
            (get_line.unwrap_unchecked())(core::ptr::null_mut::<FILE>());
            return core::ptr::null_mut();
        }
    }

    return line;
}

unsafe extern "C" fn parse_win31_line_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR;
    let mut value: *mut WCHAR;
    let mut key_end: u32 = 0;

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        return core::ptr::null_mut();
    }

    if wcsncmp(line, RUSTCONV_WSTR_87D16031626E.as_ptr(), 17) != 0 {
        /* "HKEY_CLASSES_ROOT" without NUL */
        return line;
    }

    /* get key name */
    while *line.add(key_end as usize) != 0 && iswspace(*line.add(key_end as usize)) == 0 {
        key_end += 1;
    }

    value = line.add(key_end as usize);
    while *value == ' ' as WCHAR || *value == '\t' as WCHAR {
        value = value.add(1);
    }

    if *value == '=' as WCHAR {
        value = value.add(1);
    }
    if *value == ' ' as WCHAR {
        value = value.add(1); /* at most one space is skipped */
    }

    *line.add(key_end as usize) = 0;

    if (open_key(parser, line)) as u32 != ERROR_SUCCESS {
        output_message(STRING_KEY_IMPORT_FAILED, line);
        return line;
    }

    (*parser).value_name = core::ptr::null_mut();
    (*parser).data_type = REG_SZ;
    (*parser).data = value as *mut core::ffi::c_void;
    (*parser).data_size = ((lstrlenW(value as LPCWSTR) + 1) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;

    set_state(parser, SET_VALUE);
    return value;
}

unsafe extern "C" fn line_start_state(mut parser: *mut parser, mut pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut p: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    p = line;
    while *p != 0 {
        match *p as ::core::ffi::c_int {
            91 => {
                set_state(parser, KEY_NAME);
                return p.offset(1 as isize);
            }
            64 => {
                set_state(parser, DEFAULT_VALUE_NAME);
                return p;
            }
            34 => {
                set_state(parser, QUOTED_VALUE_NAME);
                return p.offset(1 as isize);
            }
            32 | 9 => {}
            _ => return p,
        }
        p = p.offset(1);
    }
    return p;
}

unsafe extern "C" fn key_name_state(mut parser: *mut parser, mut pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    let mut key_end: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    'done: {
        if *p as core::ffi::c_int == ' ' as i32 || *p as core::ffi::c_int == '\t' as i32 {
            break 'done;
        }
        key_end = wcsrchr(p, ']' as WCHAR) as *mut WCHAR;
        if key_end.is_null() {
            break 'done;
        }

        *key_end = 0;

        if *p as core::ffi::c_int == '-' as i32 {
            set_state(parser, DELETE_KEY);
            return p.offset(1 as isize);
        } else if (open_key(parser, p)) as u32 != ERROR_SUCCESS {
            output_message(STRING_KEY_IMPORT_FAILED, p);
        }
    }

    set_state(parser, LINE_START);
    return p;
}

unsafe extern "C" fn delete_key_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;

    close_key(parser);

    if *p == 'H' as u16 || *p == 'h' as u16 {
        let mut root: HKEY = core::ptr::null_mut();
        let mut path: *mut WCHAR = core::ptr::null_mut();

        root = parse_key_name(p, core::ptr::addr_of_mut!(path));

        if !root.is_null() && !path.is_null() && *path != 0 {
            RegDeleteTreeW(root, path as LPCWSTR);
        }
    }

    set_state(parser, LINE_START);
    return p;
}

unsafe extern "C" fn default_value_name_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    free((*parser).value_name as *mut ::core::ffi::c_void);
    (*parser).value_name = core::ptr::null_mut();

    set_state(parser, DATA_START);
    return pos.offset(1 as isize);
}

unsafe extern "C" fn quoted_value_name_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut val_name: *mut WCHAR = pos;
    let mut p: *mut WCHAR = core::ptr::null_mut();

    free((*parser).value_name as *mut core::ffi::c_void);
    (*parser).value_name = core::ptr::null_mut();

    if unescape_string(val_name, core::ptr::addr_of_mut!(p)) == 0 {
        set_state(parser, LINE_START);
        return val_name;
    } else {
        /* copy the value name in case we need to parse multiple lines and the buffer is overwritten */
        (*parser).value_name = malloc(
            ((lstrlenW(val_name as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        lstrcpyW((*parser).value_name as LPWSTR, val_name as LPCWSTR);

        set_state(parser, DATA_START);
        return p;
    }
}

unsafe extern "C" fn data_start_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;
    let mut len: u32;
    'invalid: {
        while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
            p = p.add(1);
        }
        if *p != '=' as WCHAR {
            break 'invalid;
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
            set_state(parser, DELETE_VALUE);
        } else {
            set_state(parser, DATA_TYPE);
        }
        return p;
    }

    set_state(parser, LINE_START);
    return p;
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

    set_state(parser, LINE_START);
    return p;
}

unsafe extern "C" fn data_type_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = pos;

    if parse_data_type(parser, core::ptr::addr_of_mut!(line)) == 0 {
        set_state(parser, LINE_START);
        return line;
    }

    match (*parser).parse_type {
        REG_SZ => {
            set_state(parser, STRING_DATA);
        }
        REG_DWORD => {
            set_state(parser, DWORD_DATA);
        }
        REG_BINARY => {
            /* all hex data types, including undefined */
            set_state(parser, HEX_DATA);
        }
        _ => {
            set_state(parser, UNKNOWN_DATA);
        }
    }

    return line;
}

unsafe extern "C" fn string_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut();

    (*parser).data = pos as *mut core::ffi::c_void;

    'invalid: {
        if unescape_string((*parser).data as *mut WCHAR, core::ptr::addr_of_mut!(line)) == 0 {
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
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;

        set_state(parser, SET_VALUE);
        return line;
    }

    free_parser_data(parser);
    set_state(parser, LINE_START);
    return line;
}

unsafe extern "C" fn dword_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = pos;

    (*parser).data = malloc(core::mem::size_of::<DWORD>());

    if !(convert_hex_to_dword(line, (*parser).data as *mut DWORD) != 0) {
        free_parser_data(parser);
        set_state(parser, LINE_START);
        return line;
    }

    (*parser).data_size = core::mem::size_of::<DWORD>() as DWORD;

    set_state(parser, SET_VALUE);
    return line;
}

unsafe extern "C" fn hex_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = pos;
    let mut Buffer: [WCHAR; 10] = [0; 10];
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut unicode_in_ascii: BOOL = (FALSE) as i32;
    let mut result: BOOL = 0;

    'set_value: {
        if !(*line == 0) {
            // no-op: the C `if (!*line) goto set_value;` is handled below
        }
        if *line == 0 {
            break 'set_value;
        }

        if ((*parser).is_unicode == 0)
            && ((*parser).data_type == REG_EXPAND_SZ)
            && ((*parser).parse_type == REG_BINARY)
        {
            memcpy(
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as *mut core::ffi::c_void,
                pos as *const core::ffi::c_void,
                18 as usize,
            );
            Buffer[(_countof!(Buffer) - 1) as usize] = UNICODE_NULL;
            ret = wcsstr(
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
                RUSTCONV_WSTR_43425498687F.as_ptr() as *const WCHAR,
            ) as *mut WCHAR; // Any UNICODE characters?
            unicode_in_ascii = (!ret.is_null()) as BOOL;
        }

        if unicode_in_ascii != 0 {
            (*parser).is_unicode = (TRUE) as i32;
            result = convert_hex_csv_to_hex(parser, core::ptr::addr_of_mut!(line));
            (*parser).is_unicode = (FALSE) as i32;
        } else {
            result = convert_hex_csv_to_hex(parser, core::ptr::addr_of_mut!(line));
        }
        if false {
            // #else branch (not compiled under __REACTOS__)
            let _ = convert_hex_csv_to_hex(parser, core::ptr::addr_of_mut!(line));
        }

        'invalid: {
            if result == 0 {
                break 'invalid;
            }

            if (*parser).backslash != 0 {
                set_state(parser, EOL_BACKSLASH);
                return line;
            }

            if unicode_in_ascii != 0 {
                (*parser).is_unicode = (TRUE) as i32;
                prepare_hex_string_data(parser);
                (*parser).is_unicode = (FALSE) as i32;
            } else {
                prepare_hex_string_data(parser);
            }
            if false {
                // #else branch (not compiled under __REACTOS__)
                prepare_hex_string_data(parser);
            }

            break 'set_value;
        }

        // invalid:
        free_parser_data(parser);
        set_state(parser, LINE_START);
        return line;
    }

    // set_value:
    set_state(parser, SET_VALUE);
    return line;
}

unsafe extern "C" fn eol_backslash_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut p: *mut WCHAR = pos;

    while *p == ' ' as WCHAR || *p == '\t' as WCHAR {
        p = p.add(1);
    }
    'invalid: {
        if *p != 0 && *p != ';' as WCHAR {
            break 'invalid;
        }

        set_state(parser, HEX_MULTILINE);
        return pos;
    }

    free_parser_data(parser);
    set_state(parser, LINE_START);
    p
}

unsafe extern "C" fn hex_multiline_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    let mut line: *mut WCHAR = core::ptr::null_mut();

    line = (get_line.unwrap_unchecked())((*parser).file);
    if line.is_null() {
        prepare_hex_string_data(parser);
        set_state(parser, SET_VALUE);
        return pos;
    }

    while *line == ' ' as u16 || *line == '\t' as u16 {
        line = line.offset(1);
    }
    if *line == 0 || *line == ';' as u16 {
        return line;
    }

    if iswxdigit(*line) == 0 {
        free_parser_data(parser);
        set_state(parser, LINE_START);
        return line;
    }

    set_state(parser, HEX_DATA);
    return line;
}

unsafe extern "C" fn unknown_data_state(parser: *mut parser, pos: *mut WCHAR) -> *mut WCHAR {
    set_state(parser, LINE_START);
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
        set_state(parser, PARSE_WIN31_LINE);
    } else {
        set_state(parser, LINE_START);
    }

    return pos;
}

unsafe extern "C" fn get_lineA(mut fp: *mut FILE) -> *mut WCHAR {
    static mut lineW: *mut WCHAR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;
    static mut size: size_t = 0;
    static mut buf: *mut ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
    static mut next: *mut ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();

    free(lineW as *mut ::core::ffi::c_void);

    'cleanup: {
        if fp.is_null() {
            break 'cleanup;
        }

        if size == 0 {
            size = REG_VAL_BUF_SIZE as size_t;
            buf = malloc(size as size_t) as *mut ::core::ffi::c_char;
            *buf = 0 as ::core::ffi::c_char;
            next = buf;
        }
        line = next;

        while !next.is_null() {
            let mut p: *mut ::core::ffi::c_char = (strpbrk(
                line,
                (b"\r\n\0".as_ptr() as *const ::core::ffi::c_char).cast(),
            )).cast();
            if p.is_null() {
                let mut len: size_t = 0;
                let mut count: size_t = 0;
                len = strlen(next) as size_t;
                memmove(
                    buf as *mut ::core::ffi::c_void,
                    next as *const ::core::ffi::c_void,
                    len.wrapping_add(1 as size_t),
                );
                if size.wrapping_sub(len) < (3 as size_t) {
                    size = size.wrapping_mul(2 as size_t);
                    buf = realloc(buf as *mut ::core::ffi::c_void, size as size_t)
                        as *mut ::core::ffi::c_char;
                }
                count = fread(
                    buf.offset(len as isize) as *mut ::core::ffi::c_void,
                    1 as size_t,
                    size
                        .wrapping_sub(len)
                        .wrapping_sub(1 as size_t),
                    fp,
                ) as size_t;
                if count == 0 {
                    next = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    lineW = GetWideString(buf);
                    return lineW;
                }
                *buf.offset(len.wrapping_add(count) as isize) =
                    0 as ::core::ffi::c_char;
                next = buf;
                line = buf;
                continue;
            }
            next = p.offset(1 as isize);
            if *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                && *p.offset(1 as isize) as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int
            {
                next = next.offset(1);
            }
            *p = 0 as ::core::ffi::c_char;
            lineW = GetWideString(line);
            return lineW;
        }
    }

    lineW = ::core::ptr::null_mut::<WCHAR>();
    free(buf as *mut ::core::ffi::c_void);
    size = 0 as size_t;
    return ::core::ptr::null_mut::<WCHAR>();
}

unsafe extern "C" fn get_lineW(fp: *mut FILE) -> *mut WCHAR {
    static mut size: size_t = 0;
    static mut buf: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    static mut next: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut line: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    'cleanup: {
        if fp.is_null() {
            break 'cleanup;
        }
        if size == 0 {
            size = REG_VAL_BUF_SIZE;
            buf = malloc(size.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t)) as *mut WCHAR;
            *buf = 0;
            next = buf;
        }
        line = next;
        while !next.is_null() {
            let p: *mut WCHAR = wcspbrk(line, RUSTCONV_WSTR_80E09B2F645F.as_ptr()) as *mut WCHAR;
            if p.is_null() {
                let mut len: size_t = 0;
                let mut count: size_t = 0;
                len = lstrlenW(next as *const WCHAR) as size_t;
                memmove(
                    buf as *mut ::core::ffi::c_void,
                    next as *const ::core::ffi::c_void,
                    len.wrapping_add(1).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                );
                if size.wrapping_sub(len) < 3 {
                    size = size.wrapping_mul(2);
                    buf = realloc(
                        buf as *mut ::core::ffi::c_void,
                        size.wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                    ) as *mut WCHAR;
                }
                count = fread(
                    buf.offset(len as isize) as *mut ::core::ffi::c_void,
                    core::mem::size_of::<WCHAR>() as size_t,
                    size.wrapping_sub(len).wrapping_sub(1),
                    fp,
                ) as size_t;
                if count == 0 {
                    next = core::ptr::null_mut::<WCHAR>();
                    return buf;
                }
                *buf.offset(len.wrapping_add(count) as isize) = 0;
                next = buf;
                line = buf;
                continue;
            }
            next = p.offset(1 as isize);
            if *p == '\r' as WCHAR && *p.offset(1 as isize) == '\n' as WCHAR {
                next = next.offset(1);
            }
            *p = 0;
            return line;
        }
    }
    free(buf as *mut ::core::ffi::c_void);
    size = 0;
    return core::ptr::null_mut::<WCHAR>();
}
#[no_mangle]
pub unsafe extern "C" fn reg_import(mut argc: ::core::ffi::c_int, mut argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut filename: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut pos: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut fp: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut s: [BYTE; 2] = [0; 2];
    let mut parser: parser = parser {
        file: ::core::ptr::null_mut::<FILE>(),
        two_wchars: [0; 2],
        is_unicode: 0,
        reg_version: 0,
        sam: 0,
        hkey: ::core::ptr::null_mut::<HKEY__>(),
        key_name: ::core::ptr::null_mut::<WCHAR>(),
        value_name: ::core::ptr::null_mut::<WCHAR>(),
        parse_type: 0,
        data_type: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        data_size: 0,
        backslash: 0,
        state: HEADER,
    };

    'invalid: {
        if argc > 4 {
            break 'invalid;
        }

        parser.sam = 0 as REGSAM;

        if argc == 4 {
            let mut str: *mut WCHAR = *argvW.offset(3 as isize);

            if *str as ::core::ffi::c_int != '/' as i32 && *str as ::core::ffi::c_int != '-' as i32 {
                break 'invalid;
            }

            str = str.offset(1);

            if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_744906669607.as_ptr()) == 0 {
                parser.sam = KEY_WOW64_32KEY as REGSAM;
            } else if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_E3997F49E5CE.as_ptr()) == 0 {
                parser.sam = KEY_WOW64_64KEY as REGSAM;
            } else {
                break 'invalid;
            }
        }

        filename = *argvW.offset(2 as isize);

        fp = (_wfopen(filename, RUSTCONV_WSTR_76CFE90FF2FC.as_ptr())).cast();
        if fp.is_null() {
            output_message(STRING_FILE_NOT_FOUND, filename);
            return 1;
        }

        'error: {
            if fread(
                &raw mut s as *mut BYTE as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<WCHAR>() as size_t,
                1 as size_t,
                fp,
            ) != 1 as size_t
            {
                break 'error;
            }

            parser.is_unicode = (s[0 as usize] as ::core::ffi::c_int
                == 0xff as ::core::ffi::c_int
                && s[1 as usize] as ::core::ffi::c_int
                    == 0xfe as ::core::ffi::c_int)
                as ::core::ffi::c_int as BOOL;
            get_line = (if parser.is_unicode != 0 {
                Some(get_lineW as unsafe extern "C" fn(*mut FILE) -> *mut WCHAR)
            } else {
                Some(get_lineA as unsafe extern "C" fn(*mut FILE) -> *mut WCHAR)
            })
                as Option<unsafe extern "C" fn(*mut FILE) -> *mut WCHAR>;

            parser.file = fp;
            parser.two_wchars[0 as usize] =
                s[0 as usize] as WCHAR;
            parser.two_wchars[1 as usize] =
                s[1 as usize] as WCHAR;
            parser.reg_version = -(1 as ::core::ffi::c_int) as ::core::ffi::c_short;
            parser.hkey = ::core::ptr::null_mut::<HKEY__>();
            parser.key_name = ::core::ptr::null_mut::<WCHAR>();
            parser.value_name = ::core::ptr::null_mut::<WCHAR>();
            parser.parse_type = 0 as DWORD;
            parser.data_type = 0 as DWORD;
            parser.data = ::core::ptr::null_mut::<::core::ffi::c_void>();
            parser.data_size = 0 as DWORD;
            parser.backslash = FALSE as BOOL;
            parser.state = HEADER;

            pos = &raw mut parser.two_wchars as *mut WCHAR;

            /* parser main loop */
            while !pos.is_null() {
                pos = parser_funcs[parser.state as usize]
                    .unwrap_unchecked()(
                    &raw mut parser, pos
                );
            }

            if parser.reg_version as ::core::ffi::c_int
                == REG_VERSION_INVALID as ::core::ffi::c_int
            {
                break 'error;
            }

            free(parser.value_name as *mut ::core::ffi::c_void);
            close_key(&raw mut parser);

            fclose(fp);
            return 0;
        }

        // error:
        fclose(fp);
        return 1;
    }

    // invalid:
    output_message(STRING_INVALID_SYNTAX as ::core::ffi::c_uint);
    output_message(
        STRING_FUNC_HELP as ::core::ffi::c_uint,
        _wcsupr(*argvW.offset(1 as isize) as *mut wchar_t),
    );
    return 1;
}
