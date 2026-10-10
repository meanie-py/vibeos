// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_010340BC5B61: [u16; 11] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0042, 0x0049, 0x004E, 0x0041, 0x0052, 0x0059,
    0x0000,
];
pub static RUSTCONV_WSTR_1EA9E69A696A: [u16; 3] = [0x0077u16, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_22559A0A7652: [u16; 13] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x004D, 0x0055, 0x004C, 0x0054, 0x0049, 0x005F,
    0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_WSTR_2DD11AA35112: [u16; 4] = [0x0020u16, 0x0020, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_30BC907EFB89: [u16; 7] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0053, 0x005A, 0x0000];
pub static RUSTCONV_WSTR_3B3A24BC138B: [u16; 10] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0044, 0x0057, 0x004F, 0x0052, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_4D71FACE03C3: [u16; 14] = [
    0x0052u16, 0x0045, 0x0047, 0x005F, 0x0045, 0x0058, 0x0050, 0x0041, 0x004E, 0x0044,
    0x005F, 0x0053, 0x005A, 0x0000,
];
pub static RUSTCONV_WSTR_5E0088F414A5: [u16; 10] = [0x0025u16, 0x002D, 0x0031, 0x0039, 0x0073, 0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_64C484C8069A: [u16; 13] = [
    0x0025u16, 0x002D, 0x0031, 0x0039, 0x0073, 0x0030, 0x0078, 0x0025, 0x006C, 0x0078,
    0x000D, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_75C0322241F2: [u16; 10] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x0051, 0x0057, 0x004F, 0x0052, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_7961091B3C25: [u16; 3] = [0x0020u16, 0x002D, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_836DAD312A4A: [u16; 6] = [0x0020u16, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A2FF2DDCAEDE: [u16; 3] = [0x0020u16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_C245AFF91259: [u16; 2] = [0x002Eu16, 0x0000];
pub static RUSTCONV_WSTR_D211154FB261: [u16; 15] = [
    0x0025u16, 0x002D, 0x0031, 0x0039, 0x0073, 0x0025, 0x0073, 0x0020, 0x002D, 0x0020,
    0x0025, 0x0073, 0x000D, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_ED919F2C2586: [u16; 7] = [0x0025u16, 0x0030, 0x0038, 0x0058, 0x0020, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_F43A50ADA987: [u16; 9] = [0x0052u16, 0x0045, 0x0047, 0x005F, 0x004E, 0x004F, 0x004E, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_FF65042600CB: [u16; 3] = [0x0025u16, 0x0063, 0x0000];

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

unsafe extern "C" fn load_str(mut id: INT) -> LPWSTR {
    static mut s_asz: [[WCHAR; (MAX_PATH) as _]; 3] = [[0; (MAX_PATH) as _]; 3];
    static mut s_index: INT = 0 as INT;
    let mut psz: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    LoadStringW(
        hInst,
        id as UINT,
        (core::ptr::addr_of_mut!(s_asz) as *mut [WCHAR; (MAX_PATH) as _]).offset(s_index as isize) as LPWSTR,
        (MAX_PATH) as i32,
    );
    psz = (core::ptr::addr_of_mut!(s_asz) as *mut [WCHAR; (MAX_PATH) as _]).offset(s_index as isize) as *mut WCHAR as LPWSTR;
    s_index = ((s_index + 1) as usize % (_countof!(s_asz) as usize)) as INT;
    return psz;
}

unsafe extern "C" fn txt_fputs(fp: *mut FILE, str: LPCWSTR) {
    fwrite(str as *const core::ffi::c_void, (lstrlenW(str) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()), 1, (fp).cast());
}

unsafe extern "C" fn txt_newline(fp: *mut FILE) {
    txt_fputs(fp, RUSTCONV_WSTR_80E09B2F645F.as_ptr());
}

#[no_mangle]
unsafe extern "C" fn txt_fprintf_v(mut fp: *mut FILE, mut format: LPCWSTR, ap_list: *mut ::core::ffi::c_void) {
    let mut line: [WCHAR; 1024] = [0; 1024];
    let mut va: va_list = *(ap_list as *mut va_list);

    StringCchVPrintfW(line.as_mut_ptr(), (_countof!(line)) as _, format, (va).cast());
    txt_fputs(fp, line.as_ptr());

}



unsafe extern "C" fn txt_parse_key_name(key_name: LPCWSTR, key_path: *mut *mut WCHAR) -> HKEY {
    let mut i: u32;
    if key_name.is_null() {
        return core::ptr::null_mut();
    }
    *key_path = wcschr(key_name, '\\' as u16) as *mut WCHAR;
    if !(*key_path).is_null() {
        *key_path = (*key_path).add(1);
    }
    i = 0;
    while (i as usize) < (_countof!(reg_class_keys) as usize) {
        let len: i32 = lstrlenW(reg_class_namesW[i as usize]);
        if _wcsnicmp(key_name, reg_class_namesW[i as usize], len as usize) == 0
            && (*key_name.offset(len as isize) == 0
                || *key_name.offset(len as isize) == '\\' as u16)
        {
            return reg_class_keys[i as usize];
        }
        i = i.wrapping_add(1);
    }
    core::ptr::null_mut()
}

unsafe extern "C" fn txt_export_binary(fp: *mut FILE, data: *const core::ffi::c_void, size: size_t) {
    let pb: *const BYTE = data as *const BYTE;
    let mut addr: DWORD = 0;
    while (addr as size_t) < size {
        txt_fprintf(fp, RUSTCONV_WSTR_ED919F2C2586.as_ptr(), addr);
        let mut column: size_t = 0;
        while column < 16 {
            if (addr as size_t).wrapping_add(column) >= size {
                if column == 8 {
                    txt_fputs(fp, RUSTCONV_WSTR_A2FF2DDCAEDE.as_ptr());
                }
                txt_fputs(fp, RUSTCONV_WSTR_2DD11AA35112.as_ptr());
            } else {
                if column == 8 {
                    txt_fputs(fp, RUSTCONV_WSTR_7961091B3C25.as_ptr());
                }
                txt_fprintf(
                    fp,
                    RUSTCONV_WSTR_836DAD312A4A.as_ptr(),
                    (*pb.add((addr as size_t).wrapping_add(column))) as core::ffi::c_int
                        & 0xFF,
                );
            }
            column = column.wrapping_add(1);
        }
        txt_fputs(fp, RUSTCONV_WSTR_A2FF2DDCAEDE.as_ptr());
        let mut column_0: size_t = 0;
        while column_0 < 16 {
            if (addr as size_t).wrapping_add(column_0) >= size {
                break;
            } else {
                let b: BYTE = *pb.add((addr as size_t).wrapping_add(column_0));
                if isprint(b as core::ffi::c_int) != 0
                    || IsCharAlphaNumericW(b as WCHAR) != 0
                {
                    txt_fprintf(fp, RUSTCONV_WSTR_FF65042600CB.as_ptr(), b as core::ffi::c_int);
                } else {
                    txt_fputs(fp, RUSTCONV_WSTR_C245AFF91259.as_ptr());
                }
            }
            column_0 = column_0.wrapping_add(1);
        }
        txt_newline(fp);
        addr = addr.wrapping_add(0x10);
    }
}

unsafe extern "C" fn txt_export_field(fp: *mut FILE, label: LPCWSTR, value: LPCWSTR) {
    txt_fprintf(fp, RUSTCONV_WSTR_5E0088F414A5.as_ptr(), label, value);
}

unsafe extern "C" fn txt_export_multi_sz(mut fp: *mut FILE, mut data: *const core::ffi::c_void, mut size: size_t) {
    let mut pch: LPCWSTR;
    pch = data as LPCWSTR;
    while *pch != 0 {
        if pch == data as LPCWSTR {
            txt_export_field(fp, load_str((IDS_FIELD_DATA) as INT) as LPCWSTR, pch);
        } else {
            txt_export_field(fp, RUSTCONV_WSTR_96A296D224F2.as_ptr(), pch);
        }
        pch = pch.offset((lstrlenW(pch) + 1) as isize);
    }
}

unsafe extern "C" fn txt_export_type(fp: *mut FILE, type_0: LPCWSTR) {
    txt_export_field(fp, load_str((IDS_FIELD_TYPE) as INT) as LPCWSTR, type_0);
}

unsafe extern "C" fn txt_export_name(fp: *mut FILE, name: LPCWSTR) {
    txt_export_field(fp, load_str((IDS_FIELD_NAME) as INT) as LPCWSTR, name);
}

unsafe extern "C" fn txt_export_data(mut fp: *mut FILE, mut i: INT, mut value_name: LPCWSTR, mut value_len: DWORD, mut type_0: DWORD, mut data: *const core::ffi::c_void, mut size: size_t) {
    let mut pszType: LPCWSTR = core::ptr::null_mut();

    txt_fprintf(fp, load_str((IDS_VALUE_INDEX) as INT) as LPCWSTR, i);
    txt_newline(fp);
    txt_export_name(fp, value_name);

    match type_0 {
        REG_SZ => {
            txt_export_type(fp, RUSTCONV_WSTR_30BC907EFB89.as_ptr());
            txt_export_field(fp, load_str((IDS_FIELD_DATA) as INT) as LPCWSTR, data as LPCWSTR);
        }
        REG_DWORD => {
            txt_export_type(fp, RUSTCONV_WSTR_3B3A24BC138B.as_ptr());
            txt_fprintf(fp, RUSTCONV_WSTR_64C484C8069A.as_ptr(), load_str((IDS_FIELD_DATA) as INT), *(data as *const DWORD));
        }
        REG_EXPAND_SZ => {
            txt_export_type(fp, RUSTCONV_WSTR_4D71FACE03C3.as_ptr());
            txt_export_field(fp, load_str((IDS_FIELD_DATA) as INT) as LPCWSTR, data as LPCWSTR);
        }
        REG_MULTI_SZ => {
            txt_export_type(fp, RUSTCONV_WSTR_22559A0A7652.as_ptr());
            txt_export_multi_sz(fp, data, size);
        }
        REG_BINARY | REG_QWORD | REG_NONE | _ => {
            if type_0 == REG_BINARY {
                pszType = RUSTCONV_WSTR_010340BC5B61.as_ptr();
            } else if type_0 == REG_QWORD {
                pszType = RUSTCONV_WSTR_75C0322241F2.as_ptr();
            } else if type_0 == REG_NONE {
                pszType = RUSTCONV_WSTR_F43A50ADA987.as_ptr();
            } else {
                pszType = load_str((IDS_UNKNOWN) as INT) as LPCWSTR;
            }

            txt_export_type(fp, pszType);
            txt_export_field(fp, load_str((IDS_FIELD_DATA) as INT) as LPCWSTR, RUSTCONV_WSTR_96A296D224F2.as_ptr());
            txt_export_binary(fp, data, size);
        }
    }

    txt_newline(fp);
}

unsafe extern "C" fn txt_build_subkey_path(
    path: LPCWSTR,
    path_len: DWORD,
    subkey_name: LPCWSTR,
    subkey_len: DWORD,
) -> *mut WCHAR {
    let mut subkey_path: *mut WCHAR;
    let cb_subkey_path: SIZE_T = (path_len as SIZE_T + subkey_len as SIZE_T + 2) * core::mem::size_of::<WCHAR>() as SIZE_T;
    subkey_path = malloc(cb_subkey_path as size_t) as *mut WCHAR;
    StringCbPrintfW!(subkey_path, cb_subkey_path, RUSTCONV_WSTR_BA2342D760A2.as_ptr(), path, subkey_name);
    return subkey_path;
}

unsafe extern "C" fn txt_export_key_name(mut fp: *mut FILE, mut name: LPCWSTR) {
    txt_export_field(fp, load_str((IDS_FIELD_KEY_NAME) as INT) as LPCWSTR, name);
}

unsafe extern "C" fn txt_export_class_and_last_write(fp: *mut FILE, key: HKEY) {
    let mut szClassName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cchClassName: DWORD = _countof!(szClassName);
    let mut ftLastWrite: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut ftLocal: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut ftNull: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut stLastWrite: SYSTEMTIME = SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };
    let mut sz1: [WCHAR; 64] = [0; 64];
    let mut sz2: [WCHAR; 64] = [0; 64];
    let mut error: LONG;

    error = RegQueryInfoKeyW(
        key,
        core::ptr::addr_of_mut!(szClassName) as LPWSTR,
        core::ptr::addr_of_mut!(cchClassName),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ftLastWrite),
    );
    if (error) as u32 != ERROR_SUCCESS {
        cchClassName = 0;
        ftLastWrite = ftNull;
    }

    szClassName[(_countof!(szClassName) - 1) as usize] = UNICODE_NULL;

    if cchClassName > 0 {
        txt_export_field(
            fp,
            load_str((IDS_FIELD_CLASS_NAME) as INT) as LPCWSTR,
            core::ptr::addr_of_mut!(szClassName) as LPCWSTR,
        );
    } else {
        txt_export_field(
            fp,
            load_str((IDS_FIELD_CLASS_NAME) as INT) as LPCWSTR,
            load_str((IDS_NO_CLASS_NAME) as INT) as LPCWSTR,
        );
    }

    if memcmp(
        core::ptr::addr_of_mut!(ftLastWrite) as *const core::ffi::c_void,
        core::ptr::addr_of_mut!(ftNull) as *const core::ffi::c_void,
        core::mem::size_of::<FILETIME>() as usize,
    ) == 0
    {
        txt_export_field(
            fp,
            load_str((IDS_FIELD_LASTWRITE) as INT) as LPCWSTR,
            load_str((IDS_NULL_TIMESTAMP) as INT) as LPCWSTR,
        );
        return;
    }

    FileTimeToLocalFileTime(
        core::ptr::addr_of_mut!(ftLastWrite),
        core::ptr::addr_of_mut!(ftLocal),
    );
    FileTimeToSystemTime(
        core::ptr::addr_of_mut!(ftLocal),
        core::ptr::addr_of_mut!(stLastWrite),
    );
    GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_SHORTDATE,
        core::ptr::addr_of_mut!(stLastWrite),
        core::ptr::null(),
        core::ptr::addr_of_mut!(sz1) as LPWSTR,
        _countof!(sz1) as core::ffi::c_int,
    );
    GetTimeFormatW(
        LOCALE_USER_DEFAULT,
        TIME_NOSECONDS,
        core::ptr::addr_of_mut!(stLastWrite),
        core::ptr::null(),
        core::ptr::addr_of_mut!(sz2) as LPWSTR,
        _countof!(sz2) as core::ffi::c_int,
    );
    txt_fprintf(
        fp,
        RUSTCONV_WSTR_D211154FB261.as_ptr(),
        load_str((IDS_FIELD_LASTWRITE) as INT),
        core::ptr::addr_of_mut!(sz1) as *mut WCHAR,
        core::ptr::addr_of_mut!(sz2) as *mut WCHAR,
    );
}

unsafe extern "C" fn txt_export_registry_data(mut fp: *mut FILE, mut key: HKEY, mut path: LPCWSTR) {
    let mut rc: LONG = 0;
    let mut max_value_len: DWORD = MAX_PATH;
    let mut value_len: DWORD = 0;
    let mut max_data_bytes: DWORD = 2048;
    let mut data_size: DWORD = 0;
    let mut subkey_len: DWORD = 0;
    let mut i: DWORD = 0;
    let mut type_0: DWORD = 0;
    let mut path_len: DWORD = 0;
    let mut value_name: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut subkey_name: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut subkey_path: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut data: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut subkey: HKEY = core::ptr::null_mut::<HKEY__>();

    txt_export_key_name(fp, path);
    txt_export_class_and_last_write(fp, key);

    value_name = malloc(
        (max_value_len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    data = malloc(max_data_bytes as size_t) as *mut BYTE;

    i = 0;
    loop {
        value_len = max_value_len;
        data_size = max_data_bytes;
        rc = RegEnumValueW(
            key,
            i,
            value_name as LPWSTR,
            core::ptr::addr_of_mut!(value_len),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(type_0),
            data as LPBYTE,
            core::ptr::addr_of_mut!(data_size),
        ) as LONG;
        if rc == ERROR_SUCCESS as LONG {
            txt_export_data(
                fp,
                i as INT,
                value_name as LPCWSTR,
                value_len,
                type_0,
                data as *const core::ffi::c_void,
                data_size as size_t,
            );
            i = i.wrapping_add(1);
        } else if rc == ERROR_MORE_DATA as LONG {
            if data_size > max_data_bytes {
                max_data_bytes = data_size;
                data = realloc(data as *mut core::ffi::c_void, max_data_bytes as size_t)
                    as *mut BYTE;
            } else {
                max_value_len = max_value_len.wrapping_mul(2);
                value_name = realloc(
                    value_name as *mut core::ffi::c_void,
                    (max_value_len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                ) as *mut WCHAR;
            }
        } else {
            break;
        }
    }

    free(data as *mut core::ffi::c_void);
    free(value_name as *mut core::ffi::c_void);

    subkey_name = malloc(
        (MAX_PATH as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;

    path_len = lstrlenW(path) as DWORD;

    i = 0;
    loop {
        subkey_len = MAX_PATH;
        rc = RegEnumKeyExW(
            key,
            i,
            subkey_name as LPWSTR,
            core::ptr::addr_of_mut!(subkey_len),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<_FILETIME>(),
        ) as LONG;
        if rc == ERROR_SUCCESS as LONG {
            if i == 0 {
                txt_newline(fp);
            }

            subkey_path = txt_build_subkey_path(path, path_len, subkey_name as LPCWSTR, subkey_len);
            // txtproc.c:307 `if (!RegOpenKeyExW(..))`: recurse when the open SUCCEEDS.
            if RegOpenKeyExW(
                key,
                subkey_name as LPCWSTR,
                0 as DWORD,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(subkey),
            ) == 0
            {
                txt_newline(fp);
                txt_export_registry_data(fp, subkey, subkey_path as LPCWSTR);
                RegCloseKey(subkey);
            }
            free(subkey_path as *mut core::ffi::c_void);
            i = i.wrapping_add(1);
        } else {
            break;
        }
    }

    free(subkey_name as *mut core::ffi::c_void);
}

unsafe extern "C" fn txt_open_export_file(file_name: LPCWSTR) -> *mut FILE {
    let mut file: *mut FILE = (_wfopen(file_name, RUSTCONV_WSTR_1EA9E69A696A.as_ptr())).cast();
    if !file.is_null() {
        fwrite(
            b"\xFF\xFE\0".as_ptr() as *const core::ffi::c_void,
            2,
            1,
            (file).cast(),
        );
    }
    return file;
}

unsafe extern "C" fn txt_open_export_key(key_class: HKEY, subkey: LPCWSTR, path: LPCWSTR) -> HKEY {
    let mut key: HKEY = core::ptr::null_mut();
    if (RegOpenKeyExW(key_class, subkey, 0, KEY_READ, core::ptr::addr_of_mut!(key))) as u32 != ERROR_SUCCESS {
        return core::ptr::null_mut();
    }
    return key;
}

unsafe extern "C" fn txt_export_key(mut file_name: LPCWSTR, mut path: LPCWSTR) -> BOOL {
    let mut key_class: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut key: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut subkey: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut fp: *mut FILE = core::ptr::null_mut::<FILE>();

    key_class = txt_parse_key_name(path, core::ptr::addr_of_mut!(subkey));
    if key_class.is_null() {
        if !subkey.is_null() {
            *subkey.offset(-(1 as isize)) = 0 as WCHAR;
        }
        return FALSE as BOOL;
    }

    key = txt_open_export_key(key_class, subkey as LPCWSTR, path);
    if key.is_null() {
        return FALSE as BOOL;
    }

    fp = txt_open_export_file(file_name);
    if !fp.is_null() {
        txt_export_registry_data(fp, key, path);
        txt_newline(fp);
        fclose(fp);
    }

    RegCloseKey(key);
    return (!fp.is_null()) as ::core::ffi::c_int;
}

unsafe extern "C" fn txt_export_all(file_name: LPCWSTR, path: LPCWSTR) -> BOOL {
    let mut fp: *mut FILE = core::ptr::null_mut::<FILE>();
    let mut i: ::core::ffi::c_int = 0;
    let mut classes: [HKEY; 2] = [HKEY_LOCAL_MACHINE, HKEY_USERS];
    let mut key: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut class_name: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    fp = txt_open_export_file(file_name);
    if fp.is_null() {
        return (FALSE) as BOOL;
    }

    i = 0 as ::core::ffi::c_int;
    while (i as usize) < _countof!(classes) as usize {
        key = txt_open_export_key(classes[i as usize], core::ptr::null::<WCHAR>(), path);
        if key.is_null() {
            fclose(fp);
            return (FALSE) as BOOL;
        }

        class_name = malloc(
            ((lstrlenW(
                *(&raw mut reg_class_namesW as *mut *const WCHAR).offset(i as isize) as LPCWSTR,
            ) + 1 as ::core::ffi::c_int) as usize)
                * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        lstrcpyW(
            class_name as LPWSTR,
            *(&raw mut reg_class_namesW as *mut *const WCHAR).offset(i as isize) as LPCWSTR,
        );

        txt_export_registry_data(fp, classes[i as usize], class_name as LPCWSTR);

        free(class_name as *mut ::core::ffi::c_void);
        RegCloseKey(key);
        i += 1;
    }

    txt_newline(fp);
    fclose(fp);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn txt_export_registry_key(file_name: LPCWSTR, path: LPCWSTR) -> BOOL {
    if !path.is_null() && *path != 0 {
        return txt_export_key(file_name, path);
    } else {
        return txt_export_all(file_name, path);
    }
}
