use crate::reg::reg::parse_registry_key;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_744906669607: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_E3997F49E5CE: [u16; 7] = [0x0072u16, 0x0065, 0x0067, 0x003A, 0x0036, 0x0034, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct key {
    pub root: HKEY,
    pub subkey: *mut WCHAR,
    pub hkey: HKEY,
    pub path: *mut WCHAR,
}

pub const COPY_NO: i32 = 0;
pub const COPY_YES: i32 = 1;
pub const COPY_ALL: i32 = 2;

unsafe extern "C" fn output_error(rc: LONG) {
    if (rc) as u32 == ERROR_FILE_NOT_FOUND {
        output_message(STRING_KEY_NONEXIST);
    } else {
        output_message(STRING_ACCESS_DENIED);
    }
}

unsafe extern "C" fn ask_overwrite_value(mut path: *mut WCHAR, mut value: *mut WCHAR) -> operation {
    let mut hmod: HMODULE;
    static mut Ybuffer: [WCHAR; 4] = [0; 4];
    static mut Nbuffer: [WCHAR; 4] = [0; 4];
    static mut Abuffer: [WCHAR; 4] = [0; 4];
    static mut defval: [WCHAR; 32] = [0; 32];
    let mut answer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut str: *mut WCHAR;
    let mut count: DWORD = 0;

    hmod = GetModuleHandleW(core::ptr::null::<WCHAR>());
    LoadStringW(
        hmod,
        STRING_YES as UINT,
        &raw mut Ybuffer as *mut WCHAR,
        ARRAY_SIZE!(Ybuffer) as i32,
    );
    LoadStringW(
        hmod,
        STRING_NO as UINT,
        &raw mut Nbuffer as *mut WCHAR,
        ARRAY_SIZE!(Nbuffer) as i32,
    );
    LoadStringW(
        hmod,
        STRING_ALL as UINT,
        &raw mut Abuffer as *mut WCHAR,
        ARRAY_SIZE!(Abuffer) as i32,
    );
    LoadStringW(
        hmod,
        STRING_DEFAULT_VALUE as UINT,
        &raw mut defval as *mut WCHAR,
        ARRAY_SIZE!(defval) as i32,
    );

    str = if !value.is_null() && *value != 0 {
        value
    } else {
        &raw mut defval as *mut WCHAR
    };

    while true {
        output_message(STRING_COPY_CONFIRM as ::core::ffi::c_uint, path, str);
        output_message(STRING_YESNOALL as ::core::ffi::c_uint);

        ReadConsoleW(
            GetStdHandle(STD_INPUT_HANDLE),
            &raw mut answer as *mut WCHAR as LPVOID,
            ARRAY_SIZE!(answer) as DWORD,
            &raw mut count,
            core::ptr::null_mut(),
        );

        *(&raw mut answer as *mut WCHAR) =
            towupper((*(&raw mut answer as *mut WCHAR)) as _) as WCHAR;

        if *(&raw mut answer as *mut WCHAR) == *(&raw mut Ybuffer as *mut WCHAR) {
            return (COPY_YES) as operation;
        }
        if *(&raw mut answer as *mut WCHAR) == *(&raw mut Nbuffer as *mut WCHAR) {
            return (COPY_NO) as operation;
        }
        if *(&raw mut answer as *mut WCHAR) == *(&raw mut Abuffer as *mut WCHAR) {
            return (COPY_ALL) as operation;
        }
    }
    unreachable!()
}

unsafe extern "C" fn run_copy(
    mut src: *mut key,
    mut dest: *mut key,
    mut sam: REGSAM,
    mut recurse: BOOL,
    mut force: BOOL,
) -> ::core::ffi::c_int {
    let mut rc: LONG = 0;
    let mut max_subkey_len: DWORD = 0;
    let mut max_name_len: DWORD = 0;
    let mut name_len: DWORD = 0;
    let mut max_data_size: DWORD = 0;
    let mut data_size: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut dispos: DWORD = 0;
    let mut i: DWORD = 0;
    let mut name: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut data: *mut BYTE = ::core::ptr::null_mut::<BYTE>();

    rc = RegOpenKeyExW(
        (*src).root,
        (*src).subkey as LPCWSTR,
        0 as DWORD,
        (KEY_READ as REGSAM) | sam,
        &raw mut (*src).hkey,
    ) as LONG;
    if rc != 0 {
        output_error(rc);
        return 1 as ::core::ffi::c_int;
    }

    rc = RegCreateKeyExW(
        (*dest).root,
        (*dest).subkey as LPCWSTR,
        0 as DWORD,
        ::core::ptr::null_mut::<WCHAR>(),
        REG_OPTION_NON_VOLATILE as DWORD,
        (KEY_READ | KEY_WRITE) as REGSAM | sam,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        &raw mut (*dest).hkey,
        &raw mut dispos,
    ) as LONG;
    if rc != 0 {
        RegCloseKey((*src).hkey);
        output_error(rc);
        return 1 as ::core::ffi::c_int;
    }

    'cleanup: {
        rc = RegQueryInfoKeyW(
            (*src).hkey,
            ::core::ptr::null_mut::<WCHAR>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut max_subkey_len,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut max_name_len,
            &raw mut max_data_size,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<_FILETIME>(),
        ) as LONG;
        if rc != 0 {
            break 'cleanup;
        }

        max_name_len = max!(max_subkey_len, max_name_len).wrapping_add(1 as DWORD);

        name = malloc(
            (max_name_len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        ) as *mut WCHAR;
        if name.is_null() {
            rc = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'cleanup;
        }

        data = malloc(max_data_size as size_t) as *mut BYTE;
        if data.is_null() {
            rc = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'cleanup;
        }

        i = 0 as DWORD;
        'copy: loop {
            name_len = max_name_len;
            data_size = max_data_size;

            rc = RegEnumValueW(
                (*src).hkey,
                i,
                name as LPWSTR,
                &raw mut name_len,
                ::core::ptr::null_mut::<DWORD>(),
                &raw mut type_0,
                data as LPBYTE,
                &raw mut data_size,
            ) as LONG;
            if rc == ERROR_NO_MORE_ITEMS as LONG {
                break 'copy;
            }
            if rc != 0 {
                break 'cleanup;
            }

            'body: {
                if force == 0 && dispos == REG_OPENED_EXISTING_KEY as DWORD {
                    if RegQueryValueExW(
                        (*dest).hkey,
                        name as LPCWSTR,
                        ::core::ptr::null_mut::<DWORD>(),
                        ::core::ptr::null_mut::<DWORD>(),
                        ::core::ptr::null_mut::<BYTE>(),
                        ::core::ptr::null_mut::<DWORD>(),
                    ) == 0
                    {
                        let mut op: operation = (COPY_NO as u32);
                        op = ask_overwrite_value((*src).path, name);
                        if op as ::core::ffi::c_uint
                            == COPY_NO as ::core::ffi::c_uint
                        {
                            break 'body;
                        }
                        if op as ::core::ffi::c_uint
                            == COPY_ALL as ::core::ffi::c_uint
                        {
                            force = TRUE as BOOL;
                        }
                    }
                }

                rc = RegSetValueExW(
                    (*dest).hkey,
                    name as LPCWSTR,
                    0 as DWORD,
                    type_0,
                    data,
                    data_size,
                ) as LONG;
                if rc != 0 {
                    output_error(rc);
                    break 'cleanup;
                }
            }
            i = i.wrapping_add(1);
        }

        i = 0 as DWORD;
        while recurse != 0 {
            let mut subkey_src: key = key {
                root: ::core::ptr::null_mut::<HKEY__>(),
                subkey: ::core::ptr::null_mut::<WCHAR>(),
                hkey: ::core::ptr::null_mut::<HKEY__>(),
                path: ::core::ptr::null_mut::<WCHAR>(),
            };
            let mut subkey_dest: key = key {
                root: ::core::ptr::null_mut::<HKEY__>(),
                subkey: ::core::ptr::null_mut::<WCHAR>(),
                hkey: ::core::ptr::null_mut::<HKEY__>(),
                path: ::core::ptr::null_mut::<WCHAR>(),
            };
            let mut path_len: size_t = 0;

            name_len = max_name_len;

            rc = RegEnumKeyExW(
                (*src).hkey,
                i,
                name as LPWSTR,
                &raw mut name_len,
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<WCHAR>(),
                ::core::ptr::null_mut::<DWORD>(),
                ::core::ptr::null_mut::<_FILETIME>(),
            ) as LONG;
            if rc != 0 {
                break;
            }

            subkey_src.root = (*src).hkey;
            subkey_src.subkey = name;

            subkey_dest.root = (*dest).hkey;
            subkey_dest.subkey = name;

            path_len = (lstrlenW((*src).path as LPCWSTR) as DWORD)
                .wrapping_add(name_len)
                .wrapping_add(2 as DWORD) as size_t;

            subkey_src.path = malloc(
                path_len.wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
            ) as *mut WCHAR;
            if subkey_src.path.is_null() {
                rc = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'cleanup;
            }

            swprintf(
                subkey_src.path,
                RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
                (*src).path,
                name,
            );

            rc = run_copy(
                &raw mut subkey_src,
                &raw mut subkey_dest,
                sam,
                TRUE as BOOL,
                force,
            ) as LONG;

            free(subkey_src.path as *mut ::core::ffi::c_void);

            if rc != 0 {
                break 'cleanup;
            }
            i = i.wrapping_add(1);
        }
    }

    free(name as *mut ::core::ffi::c_void);
    free(data as *mut ::core::ffi::c_void);

    RegCloseKey((*src).hkey);
    RegCloseKey((*dest).hkey);

    return (rc != ERROR_NO_MORE_ITEMS as LONG) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn reg_copy(mut argc: ::core::ffi::c_int, mut argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut src: key = key {
        root: ::core::ptr::null_mut::<HKEY__>(),
        subkey: ::core::ptr::null_mut::<WCHAR>(),
        hkey: ::core::ptr::null_mut::<HKEY__>(),
        path: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut dest: key = key {
        root: ::core::ptr::null_mut::<HKEY__>(),
        subkey: ::core::ptr::null_mut::<WCHAR>(),
        hkey: ::core::ptr::null_mut::<HKEY__>(),
        path: ::core::ptr::null_mut::<WCHAR>(),
    };
    let mut recurse: BOOL = FALSE as BOOL;
    let mut force: BOOL = FALSE as BOOL;
    let mut sam: REGSAM = 0 as REGSAM;
    let mut i: ::core::ffi::c_int = 0;

    'invalid: {
        if argc == 3 as ::core::ffi::c_int {
            break 'invalid;
        }

        if parse_registry_key(
            *argvW.offset(2 as isize),
            core::ptr::addr_of_mut!(src.root),
            core::ptr::addr_of_mut!(src.subkey),
        ) == 0
        {
            return 1 as ::core::ffi::c_int;
        }

        if parse_registry_key(
            *argvW.offset(3 as isize),
            core::ptr::addr_of_mut!(dest.root),
            core::ptr::addr_of_mut!(dest.subkey),
        ) == 0
        {
            return 1 as ::core::ffi::c_int;
        }

        i = 4 as ::core::ffi::c_int;
        while i < argc {
            'next: {
                let mut str: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

                if (*(*argvW.offset(i as isize)).offset(0 as isize)) as ::core::ffi::c_int != '/' as ::core::ffi::c_int
                    && (*(*argvW.offset(i as isize)).offset(0 as isize)) as ::core::ffi::c_int != '-' as ::core::ffi::c_int
                {
                    break 'invalid;
                }

                str = (*argvW.offset(i as isize)).offset(1 as isize) as *mut WCHAR;

                if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_744906669607.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_32KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_32KEY as REGSAM;
                    break 'next;
                } else if lstrcmpiW(str as LPCWSTR, RUSTCONV_WSTR_E3997F49E5CE.as_ptr()) == 0 {
                    if (sam & (KEY_WOW64_64KEY as REGSAM)) != 0 {
                        break 'invalid;
                    }
                    sam |= KEY_WOW64_64KEY as REGSAM;
                    break 'next;
                } else if (*str.offset(0 as isize)) == 0
                    || (*str.offset(1 as isize)) as ::core::ffi::c_int != 0
                {
                    break 'invalid;
                }

                match towlower(*str) as ::core::ffi::c_int {
                    115 => {
                        if recurse != 0 {
                            break 'invalid;
                        }
                        recurse = TRUE as BOOL;
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

        if sam == ((KEY_WOW64_32KEY | KEY_WOW64_64KEY) as REGSAM) {
            break 'invalid;
        }

        if src.root == dest.root
            && lstrcmpiW(src.subkey as LPCWSTR, dest.subkey as LPCWSTR) == 0
        {
            output_message(STRING_COPY_SRC_DEST_SAME as ::core::ffi::c_uint);
            return 1 as ::core::ffi::c_int;
        }

        src.path = src.subkey;

        return run_copy(
            core::ptr::addr_of_mut!(src),
            core::ptr::addr_of_mut!(dest),
            sam,
            recurse,
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
