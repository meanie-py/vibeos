use crate::apphelp::sdbapi::SdbpCloseMemMappedFile;
use crate::apphelp::sdbapi::SdbpOpenMemMappedFile;
use crate::apphelp::sdbapi::SdbpStrDup;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_C554B73F8939: [u16; 5] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NUM_ATTRIBUTES: usize = 28;

pub const MODTYPE_UNKNOWN: u32 = 0;
pub const MODTYPE_DOS: u32 = 1;
pub const MODTYPE_NE: u32 = 2;
pub const MODTYPE_PE: u32 = 3;

unsafe extern "system" fn SdbpSetDWORDAttr(mut attr: PATTRINFO, mut tag: TAG, mut value: DWORD) {
    (*attr).type_ = tag;
    (*attr).flags = ATTRIBUTE_AVAILABLE as DWORD;
    (*attr).__bindgen_anon_1.dwattr = value;
}
unsafe extern "system" fn SdbpSetQWORDAttr(mut attr: PATTRINFO, mut tag: TAG, mut value: QWORD) {
    (*attr).type_ = tag;
    (*attr).flags = ATTRIBUTE_AVAILABLE as DWORD;
    (*attr).__bindgen_anon_1.qwattr = value;
}
unsafe extern "system" fn SdbpSetStringAttr(mut attr: PATTRINFO, mut tag: TAG, mut string: *mut WCHAR) {
    if string.is_null() {
        (*attr).flags = ATTRIBUTE_FAILED as DWORD;
        return;
    }
    (*attr).type_ = tag;
    (*attr).flags = ATTRIBUTE_AVAILABLE as DWORD;
    (*attr).__bindgen_anon_1.lpattr = SdbpStrDup(string as LPCWSTR) as *mut WCHAR;
}

unsafe extern "system" fn SdbpSetAttrFail(attr: PATTRINFO) {
    (*attr).flags = ATTRIBUTE_FAILED as DWORD;
}
unsafe extern "system" fn SdbpGetStringAttr(translation: LPWSTR, attr: LPCWSTR, file_info: PVOID) -> *mut WCHAR {
    let mut size: UINT = 0;
    let mut buffer: PVOID = core::ptr::null_mut();
    let mut value: [WCHAR; 128] = [0; 128];

    if file_info.is_null() {
        return core::ptr::null_mut();
    }

    StringCchPrintfW!(
        core::ptr::addr_of_mut!(value) as *mut WCHAR,
        ARRAYSIZE!(value),
        translation,
        attr,
    );
    if VerQueryValueW(
        file_info,
        core::ptr::addr_of_mut!(value) as LPCWSTR,
        core::ptr::addr_of_mut!(buffer),
        core::ptr::addr_of_mut!(size),
    ) != 0
        && size != 0
    {
        return buffer as *mut WCHAR;
    }

    core::ptr::null_mut()
}

unsafe extern "system" fn SdbpSetStringAttrFromAnsiString(attr: PATTRINFO, tag: TAG, string: PBYTE, len: size_t) {
    let mut len = len;
    let mut string = string;
    let mut dest: *mut WCHAR;
    if string.is_null() {
        (*attr).flags = ATTRIBUTE_FAILED as DWORD;
        return;
    }

    (*attr).type_ = tag;
    (*attr).flags = ATTRIBUTE_AVAILABLE as DWORD;
    (*attr).__bindgen_anon_1.lpattr = SdbAlloc!(
        (len as size_t)
            .wrapping_add(1 as size_t)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    dest = (*attr).__bindgen_anon_1.lpattr;
    while len != 0 {
        len = len.wrapping_sub(1);
        *dest = *string as WCHAR;
        dest = dest.offset(1);
        string = string.offset(1);
    }
    *dest = 0 as WCHAR;
}
unsafe extern "system" fn SdbpSetStringAttrFromPascalString(attr: PATTRINFO, tag: TAG, string: PBYTE) {
    if string.is_null() {
        (*attr).flags = ATTRIBUTE_FAILED;
        return;
    }
    SdbpSetStringAttrFromAnsiString(attr, tag, string.offset(1 as isize), *string as size_t);
}

unsafe extern "C" fn SdbpReadFileVersion(attr_info: PATTRINFO, file_info: PVOID) {
    static str_root: [u16; 2] = [0x005Cu16, 0x0000];

    let mut fixed_info: *mut VS_FIXEDFILEINFO = core::ptr::null_mut();
    let mut size: UINT = 0;
    if !file_info.is_null()
        && VerQueryValueW(
            file_info as LPCVOID,
            core::ptr::addr_of!(str_root) as LPCWSTR,
            core::ptr::addr_of_mut!(fixed_info) as *mut LPVOID,
            core::ptr::addr_of_mut!(size),
        ) != 0
        && size != 0
    {
        if (*fixed_info).dwSignature == VS_FFI_SIGNATURE {
            let mut version: LARGE_INTEGER = core::mem::zeroed();
            version.__bindgen_anon_1.HighPart = (*fixed_info).dwFileVersionMS as LONG;
            version.__bindgen_anon_1.LowPart = (*fixed_info).dwFileVersionLS;
            SdbpSetQWORDAttr(
                attr_info.offset(2 as isize) as PATTRINFO,
                TAG_BIN_FILE_VERSION as TAG,
                version.QuadPart as QWORD,
            );
            SdbpSetQWORDAttr(
                attr_info.offset(21 as isize) as PATTRINFO,
                TAG_UPTO_BIN_FILE_VERSION as TAG,
                version.QuadPart as QWORD,
            );
            version.__bindgen_anon_1.HighPart = (*fixed_info).dwProductVersionMS as LONG;
            version.__bindgen_anon_1.LowPart = (*fixed_info).dwProductVersionLS;
            SdbpSetQWORDAttr(
                attr_info.offset(3 as isize) as PATTRINFO,
                TAG_BIN_PRODUCT_VERSION as TAG,
                version.QuadPart as QWORD,
            );
            SdbpSetQWORDAttr(
                attr_info.offset(22 as isize) as PATTRINFO,
                TAG_UPTO_BIN_PRODUCT_VERSION as TAG,
                version.QuadPart as QWORD,
            );

            SdbpSetDWORDAttr(
                attr_info.offset(12 as isize) as PATTRINFO,
                TAG_VERDATEHI as TAG,
                (*fixed_info).dwFileDateMS,
            );
            SdbpSetDWORDAttr(
                attr_info.offset(13 as isize) as PATTRINFO,
                TAG_VERDATELO as TAG,
                (*fixed_info).dwFileDateLS,
            );
            SdbpSetDWORDAttr(
                attr_info.offset(14 as isize) as PATTRINFO,
                TAG_VERFILEOS as TAG,
                (*fixed_info).dwFileOS,
            ); /* 0x000, 0x4, 0x40004, 0x40000, 0x10004, 0x10001*/
            SdbpSetDWORDAttr(
                attr_info.offset(15 as isize) as PATTRINFO,
                TAG_VERFILETYPE as TAG,
                (*fixed_info).dwFileType,
            ); /* VFT_APP, VFT_DLL, .... */
            return;
        }
    }

    SdbpSetAttrFail(attr_info.offset(2 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(3 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(12 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(13 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(14 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(15 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(21 as isize) as PATTRINFO);
    SdbpSetAttrFail(attr_info.offset(22 as isize) as PATTRINFO);
}
unsafe extern "system" fn SdbpCalculateFileChecksum(mut mapping: PMEMMAPPED) -> DWORD {
    let mut n: size_t = 0;
    let mut size: size_t = 0;
    let mut data: PDWORD = ::core::ptr::null_mut::<DWORD>();
    let mut checks: DWORD = 0 as DWORD;
    let mut carry: DWORD = 0 as DWORD;
    if (*mapping).size < 4 as SIZE_T {
        return 0 as DWORD;
    }
    if (*mapping).size >= 0x1000 as SIZE_T {
        size = 0x1000 as size_t;
        if (*mapping).size < 0x1200 as SIZE_T {
            data = (*mapping)
                .view
                .offset((*mapping).size as isize)
                .offset(-(size as isize)) as PDWORD;
        } else {
            data = ((*mapping).view as PDWORD)
                .offset((0x200 as ::core::ffi::c_int / 4 as ::core::ffi::c_int) as isize);
        }
    } else {
        data = (*mapping).view as PDWORD;
        size = (*mapping).size as size_t;
    }
    n = 0 as size_t;
    while n < size.wrapping_div(4 as size_t) {
        checks = checks.wrapping_add(*data);
        carry = (if checks & 1 as DWORD != 0 {
            0x80000000 as ::core::ffi::c_uint
        } else {
            0 as ::core::ffi::c_uint
        }) as DWORD;
        checks >>= 1 as ::core::ffi::c_int;
        checks |= carry;
        data = data.offset(1);
        n = n.wrapping_add(1);
    }
    return checks;
}
unsafe extern "system" fn SdbpGetModuleType(mut mapping: PMEMMAPPED, mut nt_headers: *mut PIMAGE_NT_HEADERS) -> DWORD {
    let mut dos: PIMAGE_DOS_HEADER = (*mapping).view as PIMAGE_DOS_HEADER;
    let mut os2: PIMAGE_OS2_HEADER;

    *nt_headers = core::ptr::null_mut();

    if (*mapping).size < 2 as SIZE_T || (*dos).e_magic as i32 != IMAGE_DOS_SIGNATURE as i32 {
        return MODTYPE_UNKNOWN as DWORD;
    }

    if (*mapping).size < core::mem::size_of::<IMAGE_DOS_HEADER>() as SIZE_T
        || (*mapping).size < ((*dos).e_lfanew + 2 as LONG) as SIZE_T
    {
        return MODTYPE_DOS as DWORD;
    }

    os2 = (dos as PBYTE).offset((*dos).e_lfanew as isize) as PIMAGE_OS2_HEADER;
    if (*os2).ne_magic as i32 == IMAGE_OS2_SIGNATURE as i32
        || (*os2).ne_magic as i32 == IMAGE_OS2_SIGNATURE_LE as i32
    {
        *nt_headers = os2 as PIMAGE_NT_HEADERS;
        return MODTYPE_NE as DWORD;
    }

    if (*mapping).size >= ((*dos).e_lfanew + 4 as LONG) as SIZE_T
        && (*(os2 as PIMAGE_NT_HEADERS)).Signature == IMAGE_NT_SIGNATURE as DWORD
    {
        *nt_headers = os2 as PIMAGE_NT_HEADERS;
        return MODTYPE_PE as DWORD;
    }

    return MODTYPE_DOS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SdbFreeFileAttributes(mut attr_info: PATTRINFO) -> BOOL {
    let mut i: WORD;

    if attr_info.is_null() {
        return FALSE as BOOL;
    }

    i = 0 as WORD;
    while (i as usize) < NUM_ATTRIBUTES {
        if ((*attr_info.offset(i as isize)).type_ & (TAG_TYPE_MASK as u16)) == (TAG_TYPE_STRINGREF as u16) {
            SdbFree!((*attr_info.offset(i as isize)).__bindgen_anon_1.lpattr as LPVOID);
        }
        i = i.wrapping_add(1);
    }
    SdbFree!(attr_info as LPVOID);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetFileAttributes(path: LPCWSTR, attr_info_ret: *mut PATTRINFO, attr_count: LPDWORD) -> BOOL {
    static str_tinfo: [WCHAR; 25] = [
    0x005Cu16, 0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E,
    0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static str_trans: [WCHAR; 29] = [
    0x005Cu16, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078,
    0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0025, 0x0025, 0x0073, 0x0000,
];
    static str_CompanyName: [WCHAR; 12] = [
    0x0043u16, 0x006F, 0x006D, 0x0070, 0x0061, 0x006E, 0x0079, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0000,
];
    static str_FileDescription: [WCHAR; 16] = [
    0x0046u16, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069,
    0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static str_FileVersion: [WCHAR; 12] = [
    0x0046u16, 0x0069, 0x006C, 0x0065, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0000,
];
    static str_InternalName: [WCHAR; 13] = [
    0x0049u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0061, 0x006C, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];
    static str_LegalCopyright: [WCHAR; 15] = [
    0x004Cu16, 0x0065, 0x0067, 0x0061, 0x006C, 0x0043, 0x006F, 0x0070, 0x0079, 0x0072,
    0x0069, 0x0067, 0x0068, 0x0074, 0x0000,
];
    static str_OriginalFilename: [WCHAR; 17] = [
    0x004Fu16, 0x0072, 0x0069, 0x0067, 0x0069, 0x006E, 0x0061, 0x006C, 0x0046, 0x0069,
    0x006C, 0x0065, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000,
];
    static str_ProductName: [WCHAR; 12] = [
    0x0050u16, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0000,
];
    static str_ProductVersion: [WCHAR; 15] = [
    0x0050u16, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];

    let mut headers: PIMAGE_NT_HEADERS = core::ptr::null_mut();
    let mut mapped: MEMMAPPED = core::mem::zeroed::<MEMMAPPED>();
    let mut mapping_end: PBYTE = core::ptr::null_mut();
    let mut file_info: PVOID = core::ptr::null_mut();
    let mut module_type: DWORD = 0;
    let mut translation: [WCHAR; 128] = [0; 128];
    let mut attr_info: PATTRINFO = core::ptr::null_mut();

    let mut lang_page: *mut LANGANDCODEPAGE = core::ptr::null_mut();

    if SdbpOpenMemMappedFile(path, &raw mut mapped) == 0 {
        SHIM_ERR!(b"Error retrieving FILEINFO structure\n\0".as_ptr() as PCSTR);
        return FALSE as BOOL;
    }
    mapping_end = mapped.view.add(mapped.size as usize);

    attr_info = SdbAlloc!((NUM_ATTRIBUTES as usize) * core::mem::size_of::<ATTRINFO>()) as PATTRINFO;

    SdbpSetDWORDAttr(attr_info.offset(0 as isize), TAG_SIZE as TAG, mapped.size as DWORD);
    if mapped.size != 0 {
        SdbpSetDWORDAttr(attr_info.offset(1 as isize), TAG_CHECKSUM as TAG, SdbpCalculateFileChecksum(&raw mut mapped));
    } else {
        SdbpSetAttrFail(attr_info.offset(1 as isize));
    }
    module_type = SdbpGetModuleType(&raw mut mapped, &raw mut headers);

    if module_type != MODTYPE_UNKNOWN {
        SdbpSetDWORDAttr(attr_info.offset(16 as isize), TAG_MODULE_TYPE as TAG, module_type);
    } else {
        SdbpSetAttrFail(attr_info.offset(16 as isize)); /* TAG_MODULE_TYPE */
    }

    if !headers.is_null() && module_type == MODTYPE_PE && ((headers.offset(1 as isize) as PBYTE) <= mapping_end) {
        let mut info_size: DWORD = 0;
        let mut export_dir_size: ULONG = 0;
        let mut export_dir: PIMAGE_EXPORT_DIRECTORY = core::ptr::null_mut();
        let mut dos_path: LPCWSTR = core::ptr::null();

        /* Check if the caller passed an NT style path (full path, prefixed wih "\\??\\") */
        if wcsncmp(path, RUSTCONV_WSTR_C554B73F8939.as_ptr(), 4) == 0
            && (*path.offset(4 as isize) != UNICODE_NULL)
            && (*path.offset(5 as isize) == 0x003Au16)
        {
            /* Skip NT prefix, if it is present (GetFileVersionInfoSizeW doesn't handle NT paths) */
            dos_path = path.offset(4 as isize);
        } else {
            dos_path = path;
        }

        info_size = GetFileVersionInfoSizeW(dos_path, core::ptr::null_mut());
        if info_size != 0 {
            let mut page_size: UINT = 0;
            file_info = SdbAlloc!(info_size);
            GetFileVersionInfoW(dos_path, 0, info_size, file_info);
            VerQueryValueW(file_info, core::ptr::addr_of!(str_tinfo) as LPCWSTR, &raw mut lang_page as *mut LPVOID, &raw mut page_size);
            StringCchPrintfW!(
                &raw mut translation as LPWSTR,
                ARRAYSIZE!(translation),
                core::ptr::addr_of!(str_trans) as LPCWSTR,
                (*lang_page).language as i32,
                (*lang_page).code_page as i32,
            );
        }

        /* Handles 2, 3, 12, 13, 14, 15, 21, 22 */
        SdbpReadFileVersion(attr_info, file_info);

        SdbpSetStringAttr(
            attr_info.offset(4 as isize),
            TAG_PRODUCT_VERSION as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_ProductVersion) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(5 as isize),
            TAG_FILE_DESCRIPTION as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_FileDescription) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(6 as isize),
            TAG_COMPANY_NAME as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_CompanyName) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(7 as isize),
            TAG_PRODUCT_NAME as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_ProductName) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(8 as isize),
            TAG_FILE_VERSION as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_FileVersion) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(9 as isize),
            TAG_ORIGINAL_FILENAME as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_OriginalFilename) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(10 as isize),
            TAG_INTERNAL_NAME as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_InternalName) as LPCWSTR, file_info),
        );
        SdbpSetStringAttr(
            attr_info.offset(11 as isize),
            TAG_LEGAL_COPYRIGHT as TAG,
            SdbpGetStringAttr(&raw mut translation as LPWSTR, core::ptr::addr_of!(str_LegalCopyright) as LPCWSTR, file_info),
        );

        /* https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header32 */

        SdbpSetDWORDAttr(attr_info.offset(17 as isize), TAG_PE_CHECKSUM as TAG, (*headers).OptionalHeader.CheckSum);

        SdbpSetDWORDAttr(
            attr_info.offset(18 as isize),
            TAG_LINKER_VERSION as TAG, /* mislabeled! */
            ((*headers).OptionalHeader.MajorImageVersion as DWORD) << 16 | (*headers).OptionalHeader.MinorImageVersion as DWORD,
        );
        SdbpSetAttrFail(attr_info.offset(19 as isize)); /* TAG_16BIT_DESCRIPTION */
        SdbpSetAttrFail(attr_info.offset(20 as isize)); /* TAG_16BIT_MODULE_NAME */

        SdbpSetDWORDAttr(attr_info.offset(23 as isize), TAG_LINK_DATE as TAG, (*headers).FileHeader.TimeDateStamp);
        SdbpSetDWORDAttr(attr_info.offset(24 as isize), TAG_UPTO_LINK_DATE as TAG, (*headers).FileHeader.TimeDateStamp);

        export_dir = RtlImageDirectoryEntryToData(
            mapped.view as PVOID,
            FALSE as u8,
            IMAGE_DIRECTORY_ENTRY_EXPORT as u16,
            &raw mut export_dir_size,
        ) as PIMAGE_EXPORT_DIRECTORY;
        if !export_dir.is_null() && ((export_dir.offset(1 as isize) as PBYTE) <= mapping_end) {
            let mut section: PIMAGE_SECTION_HEADER = core::ptr::null_mut();
            let mut export_name: PBYTE = RtlImageRvaToVa(
                headers,
                mapped.view as PVOID,
                (*export_dir).Name as ULONG,
                &raw mut section,
            ) as PBYTE;
            if !export_name.is_null() {
                SdbpSetStringAttrFromAnsiString(
                    attr_info.offset(25 as isize),
                    TAG_EXPORT_NAME as TAG,
                    export_name,
                    strlen(export_name as *mut core::ffi::c_char),
                );
            } else {
                SdbpSetAttrFail(attr_info.offset(25 as isize)); /* TAG_EXPORT_NAME */
            }
        } else {
            SdbpSetAttrFail(attr_info.offset(25 as isize)); /* TAG_EXPORT_NAME */
        }

        if info_size != 0 {
            SdbpSetDWORDAttr(attr_info.offset(26 as isize), TAG_VER_LANGUAGE as TAG, (*lang_page).language as DWORD);
        }

        SdbpSetDWORDAttr(attr_info.offset(27 as isize), TAG_EXE_WRAPPER as TAG, 0); /* boolean */
    } else {
        let mut n: i32 = 0;
        n = 2;
        while n < NUM_ATTRIBUTES as i32 {
            if n != 16 && n != 26 {
                SdbpSetAttrFail(attr_info.offset(n as isize));
            }
            n += 1;
        }
        if module_type == MODTYPE_NE {
            let mut ptr: PBYTE = core::ptr::null_mut();
            let mut os2: PIMAGE_OS2_HEADER = headers as PIMAGE_OS2_HEADER;
            if ((os2.offset(1 as isize) as PBYTE) <= mapping_end) {
                ptr = mapped.view.offset((*os2).ne_nrestab as isize);
                if ptr <= mapping_end && ((ptr.offset(1 as isize).offset(*ptr as isize)) <= mapping_end) {
                    SdbpSetStringAttrFromPascalString(attr_info.offset(19 as isize), TAG_16BIT_DESCRIPTION as TAG, ptr);
                }
                ptr = (os2 as PBYTE).offset((*os2).ne_restab as isize);
                if ptr <= mapping_end && ((ptr.offset(1 as isize).offset(*ptr as isize)) <= mapping_end) {
                    SdbpSetStringAttrFromPascalString(attr_info.offset(20 as isize), TAG_16BIT_MODULE_NAME as TAG, ptr);
                }
            }
        }
    }

    *attr_info_ret = attr_info;
    *attr_count = NUM_ATTRIBUTES as DWORD; /* As far as I know, this one is always 28 */

    SdbFree!(file_info);
    SdbpCloseMemMappedFile(&raw mut mapped);
    return (TRUE) as BOOL;
}
