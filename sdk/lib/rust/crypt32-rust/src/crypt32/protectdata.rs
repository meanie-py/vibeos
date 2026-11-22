use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static MS_ENHANCED_PROV_W: [::core::ffi::c_ushort; 47] = unsafe {
    ::core::mem::transmute::<
        [u8; 94],
        [::core::ffi::c_ushort; 47],
    >(
        *b"M\0i\0c\0r\0o\0s\0o\0f\0t\0 \0E\0n\0h\0a\0n\0c\0e\0d\0 \0C\0r\0y\0p\0t\0o\0g\0r\0a\0p\0h\0i\0c\0 \0P\0r\0o\0v\0i\0d\0e\0r\0 \0v\x001\0.\x000\0\0\0",
    )
};

static mut crypt32_protectdata_secret: [BYTE; 20] = [
    'I' as i32 as BYTE,
    '\'' as i32 as BYTE,
    'm' as i32 as BYTE,
    ' ' as i32 as BYTE,
    'h' as i32 as BYTE,
    'u' as i32 as BYTE,
    'n' as i32 as BYTE,
    't' as i32 as BYTE,
    'i' as i32 as BYTE,
    'n' as i32 as BYTE,
    'g' as i32 as BYTE,
    ' ' as i32 as BYTE,
    'w' as i32 as BYTE,
    'a' as i32 as BYTE,
    'b' as i32 as BYTE,
    'b' as i32 as BYTE,
    'i' as i32 as BYTE,
    't' as i32 as BYTE,
    's' as i32 as BYTE,
    0 as ::core::ffi::c_int as BYTE,
];

static mut crypt_magic_str: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"Wine Crypt32 ok\0") };

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel name for the TRACE macros
pub const WINE_DEBUG_CHANNEL_NAME: &str = "crypt";

pub const CRYPT32_PROTECTDATA_PROV: u32 = 1; // PROV_RSA_FULL
pub const CRYPT32_PROTECTDATA_HASH_CALG: u32 = 0x0000_8004; // CALG_SHA1
pub const CRYPT32_PROTECTDATA_HASH_LEN: u32 = 160;
pub const CRYPT32_PROTECTDATA_KEY_CALG: u32 = 0x0000_6603; // CALG_3DES
pub const CRYPT32_PROTECTDATA_KEY_LEN: u32 = 168;
pub const CRYPT32_PROTECTDATA_SALT_LEN: u32 = 16;

#[repr(C)]
pub struct protect_data_t {
    pub count0: DWORD,
    pub info0: DATA_BLOB, /* using this to hold crypt_magic_str */
    pub count1: DWORD,
    pub info1: DATA_BLOB,
    pub null0: DWORD,
    pub szDataDescr: *mut WCHAR, /* serialized differently than the DATA_BLOBs */
    pub cipher_alg: ALG_ID,
    pub cipher_key_len: DWORD,
    pub data0: DATA_BLOB,
    pub null1: DWORD,
    pub hash_alg: ALG_ID,
    pub hash_len: DWORD,
    pub salt: DATA_BLOB,
    pub cipher: DATA_BLOB,
    pub fingerprint: DATA_BLOB,
}

unsafe extern "C" fn hex_str(mut p: *const c_uchar, mut n: c_int) -> *const c_char {
    let mut ptr: *const c_char = core::ptr::null::<c_char>();
    let mut report: [c_char; 80] = [0; 80];
    let mut r: c_int = -(1 as c_int);
    report[0 as usize] = 0 as c_char;
    ptr = wine_dbg_sprintf(
        b"%s\0".as_ptr() as *const c_char,
        b"\0".as_ptr() as *const c_char,
    );
    loop {
        n -= 1;
        if !(n >= 0 as c_int) {
            break;
        }
        let fresh15 = r;
        r = r + 1;
        if fresh15 % (20 as c_int) == (19 as c_int) {
            ptr = wine_dbg_sprintf(
                b"%s%s\0".as_ptr() as *const c_char,
                ptr,
                core::ptr::addr_of_mut!(report) as *mut c_char,
            );
            report[0 as usize] = 0 as c_char;
        }
        let fresh16 = p;
        p = p.offset(1);
        sprintf(
            (core::ptr::addr_of_mut!(report) as *mut c_char)
                .offset(strlen(core::ptr::addr_of_mut!(report) as *mut c_char) as isize),
            b"%s%02x\0".as_ptr() as *const c_char,
            if r != 0 {
                b",\0".as_ptr() as *const c_char
            } else {
                b"\0".as_ptr() as *const c_char
            },
            *fresh16 as c_int,
        );
    }
    return wine_dbg_sprintf(
        b"%s%s\0".as_ptr() as *const c_char,
        ptr,
        core::ptr::addr_of_mut!(report) as *mut c_char,
    );
}

unsafe extern "C" fn serialize_dword(mut value: DWORD, mut ptr: *mut *mut BYTE) {
    memcpy(
        *ptr as *mut core::ffi::c_void,
        &raw mut value as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as usize,
    );
    *ptr = (*ptr).offset(core::mem::size_of::<DWORD>() as usize as isize);
}

unsafe extern "C" fn serialize_string(str: *const BYTE, ptr: *mut *mut BYTE, len: DWORD, width: DWORD, prepend_len: BOOL) {
    if prepend_len != 0 {
        serialize_dword(len, ptr);
    }
    memcpy(*ptr as *mut core::ffi::c_void, str as *const core::ffi::c_void, len.wrapping_mul(width) as usize);
    *ptr = (*ptr).add(len.wrapping_mul(width) as usize);
}

unsafe extern "C" fn unserialize_dword(ptr: *const BYTE, index: *mut DWORD, size: DWORD, value: *mut DWORD) -> BOOL {
    if ptr.is_null() || index.is_null() || value.is_null() {
        return (FALSE) as BOOL;
    }

    if (*index).wrapping_add(core::mem::size_of::<DWORD>() as DWORD) > size {
        return (FALSE) as BOOL;
    }

    memcpy(
        value as *mut core::ffi::c_void,
        ptr.add(*index as usize) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    );
    *index = (*index).wrapping_add(core::mem::size_of::<DWORD>() as DWORD);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn unserialize_string(
    ptr: *const BYTE,
    index: *mut DWORD,
    size: DWORD,
    mut len: DWORD,
    width: DWORD,
    inline_len: BOOL,
    data: *mut *mut BYTE,
    stored: *mut DWORD,
) -> BOOL {
    if ptr.is_null() || data.is_null() {
        return (FALSE) as BOOL;
    }

    if inline_len != 0 {
        if unserialize_dword(ptr, index, size, core::ptr::addr_of_mut!(len)) == 0 {
            return (FALSE) as BOOL;
        }
    }

    if (*index).wrapping_add(len.wrapping_mul(width)) > size {
        return (FALSE) as BOOL;
    }

    *data = CryptMemAlloc(len.wrapping_mul(width) as ULONG) as *mut BYTE;
    if (*data).is_null() {
        return (FALSE) as BOOL;
    }

    memcpy(
        *data as *mut ::core::ffi::c_void,
        ptr.offset(*index as isize) as *const BYTE as *const ::core::ffi::c_void,
        len.wrapping_mul(width) as size_t,
    );
    if !stored.is_null() {
        *stored = len;
    }
    *index = (*index).wrapping_add(len.wrapping_mul(width));

    return (TRUE) as BOOL;
}

unsafe extern "C" fn serialize(pInfo: *const protect_data_t, pSerial: *mut DATA_BLOB) -> BOOL {
    let mut ptr: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut dwStrLen: DWORD = 0;
    let mut dwStruct: DWORD = 0;

    if pInfo.is_null()
        || (*pInfo).szDataDescr.is_null()
        || pSerial.is_null()
        || (*pInfo).info0.pbData.is_null()
        || (*pInfo).info1.pbData.is_null()
        || (*pInfo).data0.pbData.is_null()
        || (*pInfo).salt.pbData.is_null()
        || (*pInfo).cipher.pbData.is_null()
        || (*pInfo).fingerprint.pbData.is_null()
    {
        return (FALSE) as BOOL;
    }

    if (*pInfo).info0.cbData != 16 as DWORD {
        ERR!("protect_data_t info0 not 16 bytes long\n");
    }

    if (*pInfo).info1.cbData != 16 as DWORD {
        ERR!("protect_data_t info1 not 16 bytes long\n");
    }

    dwStrLen = lstrlenW((*pInfo).szDataDescr as LPCWSTR) as DWORD;

    (*pSerial).cbData = 0 as DWORD;
    (*pSerial).cbData = ((*pSerial).cbData as core::ffi::c_ulonglong).wrapping_add(
        (core::mem::size_of::<DWORD>() as usize).wrapping_mul(8 as usize)
            as core::ffi::c_ulonglong,
    ) as DWORD;
    (*pSerial).cbData = ((*pSerial).cbData as core::ffi::c_ulonglong).wrapping_add(
        (core::mem::size_of::<DWORD>() as usize).wrapping_mul(4 as usize)
            as core::ffi::c_ulonglong,
    ) as DWORD;
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).info0.cbData);
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).info1.cbData);
    (*pSerial).cbData = ((*pSerial).cbData as core::ffi::c_ulonglong).wrapping_add(
        (dwStrLen.wrapping_add(1 as DWORD) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_add(4 as usize) as core::ffi::c_ulonglong,
    ) as DWORD;
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).data0.cbData);
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).salt.cbData);
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).cipher.cbData);
    (*pSerial).cbData = (*pSerial).cbData.wrapping_add((*pInfo).fingerprint.cbData);

    /* save the actual structure size */
    dwStruct = (*pSerial).cbData;
    /* There may be a 256 byte minimum, but I can't prove it. */
    /*if (pSerial->cbData<256) pSerial->cbData=256;*/

    (*pSerial).pbData = LocalAlloc(
        LPTR as UINT,
        (*pSerial).cbData as SIZE_T,
    ) as *mut BYTE;
    if (*pSerial).pbData.is_null() {
        return (FALSE) as BOOL;
    }

    ptr = (*pSerial).pbData;

    /* count0 */
    serialize_dword((*pInfo).count0, core::ptr::addr_of_mut!(ptr));

    /* info0 */
    serialize_string(
        (*pInfo).info0.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).info0.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (FALSE) as BOOL,
    );

    /* count1 */
    serialize_dword((*pInfo).count1, core::ptr::addr_of_mut!(ptr));

    /* info1 */
    serialize_string(
        (*pInfo).info1.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).info1.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (FALSE) as BOOL,
    );

    /* null0 */
    serialize_dword((*pInfo).null0, core::ptr::addr_of_mut!(ptr));

    /* szDataDescr */
    serialize_string(
        (*pInfo).szDataDescr as *mut BYTE,
        core::ptr::addr_of_mut!(ptr),
        (dwStrLen.wrapping_add(1 as DWORD) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
    );

    /* cipher_alg */
    serialize_dword((*pInfo).cipher_alg as DWORD, core::ptr::addr_of_mut!(ptr));
    /* cipher_key_len */
    serialize_dword((*pInfo).cipher_key_len, core::ptr::addr_of_mut!(ptr));

    /* data0 */
    serialize_string(
        (*pInfo).data0.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).data0.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
    );

    /* null1 */
    serialize_dword((*pInfo).null1, core::ptr::addr_of_mut!(ptr));

    /* hash_alg */
    serialize_dword((*pInfo).hash_alg as DWORD, core::ptr::addr_of_mut!(ptr));
    /* hash_len */
    serialize_dword((*pInfo).hash_len, core::ptr::addr_of_mut!(ptr));

    /* salt */
    serialize_string(
        (*pInfo).salt.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).salt.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
    );

    /* cipher */
    serialize_string(
        (*pInfo).cipher.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).cipher.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
    );

    /* fingerprint */
    serialize_string(
        (*pInfo).fingerprint.pbData,
        core::ptr::addr_of_mut!(ptr),
        (*pInfo).fingerprint.cbData,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
    );

    if ptr.offset_from((*pSerial).pbData) as core::ffi::c_longlong
        != dwStruct as core::ffi::c_longlong
    {
        ERR!("struct size changed!? expected %lu\n", dwStruct);
        LocalFree((*pSerial).pbData as HLOCAL);
        (*pSerial).pbData = core::ptr::null_mut::<BYTE>();
        (*pSerial).cbData = 0 as DWORD;
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn unserialize(pSerial: *const DATA_BLOB, pInfo: *mut protect_data_t) -> BOOL {
    let mut ptr: *mut BYTE = core::ptr::null_mut();
    let mut index: DWORD;
    let mut size: DWORD;
    let mut status: BOOL = (TRUE as i32);

    TRACE!("called\n");

    if pInfo.is_null() || pSerial.is_null() || (*pSerial).pbData.is_null() {
        return (FALSE) as BOOL;
    }

    index = 0 as DWORD;
    ptr = (*pSerial).pbData;
    size = (*pSerial).cbData;

    /* count0 */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).count0)) == 0 {
        ERR!("reading count0 failed!\n");
        return (FALSE) as BOOL;
    }

    /* info0 */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        16 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (FALSE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).info0.pbData),
        core::ptr::addr_of_mut!((*pInfo).info0.cbData),
    ) == 0
    {
        ERR!("reading info0 failed!\n");
        return (FALSE) as BOOL;
    }

    /* count1 */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).count1)) == 0 {
        ERR!("reading count1 failed!\n");
        return (FALSE) as BOOL;
    }

    /* info1 */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        16 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (FALSE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).info1.pbData),
        core::ptr::addr_of_mut!((*pInfo).info1.cbData),
    ) == 0
    {
        ERR!("reading info1 failed!\n");
        return (FALSE) as BOOL;
    }

    /* null0 */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).null0)) == 0 {
        ERR!("reading null0 failed!\n");
        return (FALSE) as BOOL;
    }

    /* szDataDescr */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        0 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).szDataDescr) as *mut *mut BYTE,
        core::ptr::null_mut::<DWORD>(),
    ) == 0
    {
        ERR!("reading szDataDescr failed!\n");
        return (FALSE) as BOOL;
    }

    /* cipher_alg */
    if unserialize_dword(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        core::ptr::addr_of_mut!((*pInfo).cipher_alg) as *mut DWORD,
    ) == 0
    {
        ERR!("reading cipher_alg failed!\n");
        return (FALSE) as BOOL;
    }

    /* cipher_key_len */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).cipher_key_len)) == 0 {
        ERR!("reading cipher_key_len failed!\n");
        return (FALSE) as BOOL;
    }

    /* data0 */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        0 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).data0.pbData),
        core::ptr::addr_of_mut!((*pInfo).data0.cbData),
    ) == 0
    {
        ERR!("reading data0 failed!\n");
        return (FALSE) as BOOL;
    }

    /* null1 */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).null1)) == 0 {
        ERR!("reading null1 failed!\n");
        return (FALSE) as BOOL;
    }

    /* hash_alg */
    if unserialize_dword(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        core::ptr::addr_of_mut!((*pInfo).hash_alg) as *mut DWORD,
    ) == 0
    {
        ERR!("reading hash_alg failed!\n");
        return (FALSE) as BOOL;
    }

    /* hash_len */
    if unserialize_dword(ptr, core::ptr::addr_of_mut!(index), size, core::ptr::addr_of_mut!((*pInfo).hash_len)) == 0 {
        ERR!("reading hash_len failed!\n");
        return (FALSE) as BOOL;
    }

    /* salt */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        0 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).salt.pbData),
        core::ptr::addr_of_mut!((*pInfo).salt.cbData),
    ) == 0
    {
        ERR!("reading salt failed!\n");
        return (FALSE) as BOOL;
    }

    /* cipher */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        0 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).cipher.pbData),
        core::ptr::addr_of_mut!((*pInfo).cipher.cbData),
    ) == 0
    {
        ERR!("reading cipher failed!\n");
        return (FALSE) as BOOL;
    }

    /* fingerprint */
    if unserialize_string(
        ptr,
        core::ptr::addr_of_mut!(index),
        size,
        0 as DWORD,
        core::mem::size_of::<BYTE>() as DWORD,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!((*pInfo).fingerprint.pbData),
        core::ptr::addr_of_mut!((*pInfo).fingerprint.cbData),
    ) == 0
    {
        ERR!("reading fingerprint failed!\n");
        return (FALSE) as BOOL;
    }

    /* allow structure size to be too big (since some applications
     * will pad this up to 256 bytes, it seems) */
    if index > size {
        /* this is an impossible-to-reach test, but if the padding
         * issue is ever understood, this may become more useful */
        ERR!("loaded corrupt structure! (used %lu expected %lu)\n", index, size);
        status = (FALSE as i32);
    }

    return status;
}

unsafe extern "C" fn valid_protect_data(pInfo: *const protect_data_t) -> BOOL {
    let mut status: BOOL = (TRUE as i32);

    if (*pInfo).count0 != 0x0001 {
        ERR!("count0 != 0x0001 !\n");
        status = (FALSE as i32);
    }
    if (*pInfo).count1 != 0x0001 {
        ERR!("count0 != 0x0001 !\n");
        status = (FALSE as i32);
    }
    if (*pInfo).null0 != 0x0000 {
        ERR!("null0 != 0x0000 !\n");
        status = (FALSE as i32);
    }
    if (*pInfo).null1 != 0x0000 {
        ERR!("null1 != 0x0000 !\n");
        status = (FALSE as i32);
    }
    /* since we have no idea what info0 is used for, and it seems
     * rather constant, we can test for a Wine-specific magic string
     * there to be reasonably sure we're using data created by the Wine
     * implementation of CryptProtectData.
     */
    if (*pInfo).info0.cbData as usize
        != strlen(core::ptr::addr_of!(crypt_magic_str) as *const i8) + 1
        || strcmp(
            (*pInfo).info0.pbData as LPCSTR,
            core::ptr::addr_of!(crypt_magic_str) as *const i8,
        ) != 0
    {
        ERR!("info0 magic value not matched !\n");
        status = (FALSE as i32);
    }

    if status == 0 {
        ERR!("unrecognized CryptProtectData block\n");
    }

    return status;
}

unsafe extern "C" fn free_protect_data(pInfo: *mut protect_data_t) {
    if pInfo.is_null() {
        return;
    }
    CryptMemFree((*pInfo).info0.pbData as LPVOID);
    CryptMemFree((*pInfo).info1.pbData as LPVOID);
    CryptMemFree((*pInfo).szDataDescr as LPVOID);
    CryptMemFree((*pInfo).data0.pbData as LPVOID);
    CryptMemFree((*pInfo).salt.pbData as LPVOID);
    CryptMemFree((*pInfo).cipher.pbData as LPVOID);
    CryptMemFree((*pInfo).fingerprint.pbData as LPVOID);
}

unsafe extern "C" fn convert_str_to_blob(str: LPCSTR, blob: *mut DATA_BLOB) -> *mut BYTE {
    if str.is_null() || blob.is_null() {
        return core::ptr::null_mut();
    }
    (*blob).cbData = strlen(str as *const ::core::ffi::c_char).wrapping_add(1 as size_t) as DWORD;
    (*blob).pbData = CryptMemAlloc((*blob).cbData as ULONG) as *mut BYTE;
    if (*blob).pbData.is_null() {
        (*blob).cbData = 0 as DWORD;
    } else {
        strcpy(
            (*blob).pbData as *mut ::core::ffi::c_char,
            str as *const ::core::ffi::c_char,
        );
    }
    return (*blob).pbData;
}

unsafe extern "C" fn fill_protect_data(
    mut pInfo: *mut protect_data_t,
    mut szDataDescr: LPCWSTR,
    mut hProv: HCRYPTPROV,
) -> BOOL {
    let mut dwStrLen: DWORD = 0;

    if pInfo.is_null() {
        return (FALSE) as BOOL;
    }

    dwStrLen = lstrlenW(szDataDescr) as DWORD;

    memset(
        pInfo as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<protect_data_t>() as size_t,
    );

    (*pInfo).count0 = 0x0001;

    convert_str_to_blob(
        core::ptr::addr_of!(crypt_magic_str) as LPCSTR,
        core::ptr::addr_of_mut!((*pInfo).info0),
    );

    (*pInfo).count1 = 0x0001;

    convert_str_to_blob(
        core::ptr::addr_of!(crypt_magic_str) as LPCSTR,
        core::ptr::addr_of_mut!((*pInfo).info1),
    );

    (*pInfo).null0 = 0x0000;

    (*pInfo).szDataDescr = CryptMemAlloc(
        ((dwStrLen + 1) as usize
            * ::core::mem::size_of::<WCHAR>()) as ULONG,
    ) as *mut WCHAR;
    if !(*pInfo).szDataDescr.is_null() {
        memcpy(
            (*pInfo).szDataDescr as *mut ::core::ffi::c_void,
            szDataDescr as *const ::core::ffi::c_void,
            ((dwStrLen + 1) as usize
                * ::core::mem::size_of::<WCHAR>()) as size_t,
        );
    }

    (*pInfo).cipher_alg = CRYPT32_PROTECTDATA_KEY_CALG;
    (*pInfo).cipher_key_len = CRYPT32_PROTECTDATA_KEY_LEN;

    convert_str_to_blob(
        core::ptr::addr_of!(crypt_magic_str) as LPCSTR,
        core::ptr::addr_of_mut!((*pInfo).data0),
    );

    (*pInfo).null1 = 0x0000;
    (*pInfo).hash_alg = CRYPT32_PROTECTDATA_HASH_CALG;
    (*pInfo).hash_len = CRYPT32_PROTECTDATA_HASH_LEN;

    /* allocate memory to hold a salt */
    (*pInfo).salt.pbData = CryptMemAlloc(CRYPT32_PROTECTDATA_SALT_LEN) as *mut BYTE;
    if !(*pInfo).salt.pbData.is_null() {
        /* generate random salt */
        if CryptGenRandom(hProv, CRYPT32_PROTECTDATA_SALT_LEN, (*pInfo).salt.pbData) == 0 {
            ERR!("CryptGenRandom\n");
            free_protect_data(pInfo);
            return (FALSE) as BOOL;
        }
        (*pInfo).salt.cbData = CRYPT32_PROTECTDATA_SALT_LEN;
        /* debug: show our salt */
        TRACE_DATA_BLOB!(core::ptr::addr_of_mut!((*pInfo).salt));
    }

    (*pInfo).cipher.cbData = 0;
    (*pInfo).cipher.pbData = core::ptr::null_mut();

    (*pInfo).fingerprint.cbData = 0;
    (*pInfo).fingerprint.pbData = core::ptr::null_mut();

    /* check all the allocations at once */
    if (*pInfo).info0.pbData.is_null()
        || (*pInfo).info1.pbData.is_null()
        || (*pInfo).szDataDescr.is_null()
        || (*pInfo).data0.pbData.is_null()
        || (*pInfo).salt.pbData.is_null()
    {
        ERR!("could not allocate protect_data structures\n");
        free_protect_data(pInfo);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn convert_hash_to_blob(hHash: HCRYPTHASH, blob: *mut DATA_BLOB) -> BOOL {
    let mut dwSize: DWORD = 0;

    if blob.is_null() {
        return (FALSE) as BOOL;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    if CryptGetHashParam(
        hHash,
        HP_HASHSIZE as DWORD,
        core::ptr::addr_of_mut!((*blob).cbData) as *mut BYTE,
        core::ptr::addr_of_mut!(dwSize),
        0 as DWORD,
    ) == 0
    {
        ERR!("failed to get hash size\n");
        return (FALSE) as BOOL;
    }

    (*blob).pbData = CryptMemAlloc((*blob).cbData as ULONG) as *mut BYTE;
    if (*blob).pbData.is_null() {
        ERR!("failed to allocate blob memory\n");
        return (FALSE) as BOOL;
    }

    dwSize = (*blob).cbData;
    if CryptGetHashParam(
        hHash,
        HP_HASHVAL as DWORD,
        (*blob).pbData,
        core::ptr::addr_of_mut!(dwSize),
        0 as DWORD,
    ) == 0
    {
        ERR!("failed to get hash value\n");
        CryptMemFree((*blob).pbData as LPVOID);
        (*blob).pbData = core::ptr::null_mut::<BYTE>();
        (*blob).cbData = 0 as DWORD;
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn hash_matches_blob(mut hHash: HCRYPTHASH, mut two: *const DATA_BLOB) -> BOOL {
    let mut rc: BOOL = (FALSE as i32);
    let mut one: DATA_BLOB = DATA_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
    };

    if two.is_null() || (*two).pbData.is_null() {
        return (FALSE) as BOOL;
    }

    if convert_hash_to_blob(hHash, core::ptr::addr_of_mut!(one)) == 0 {
        return (FALSE) as BOOL;
    }

    if one.cbData == (*two).cbData
        && memcmp(
            one.pbData as *const core::ffi::c_void,
            (*two).pbData as *const core::ffi::c_void,
            one.cbData as usize,
        ) == 0
    {
        rc = (TRUE as i32);
    }

    CryptMemFree(one.pbData as LPVOID);
    return rc;
}

unsafe extern "C" fn load_encryption_key(
    hProv: HCRYPTPROV,
    key_len: DWORD,
    salt: *const DATA_BLOB,
    pOptionalEntropy: *const DATA_BLOB,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    let mut rc: BOOL = TRUE as BOOL;
    let mut hSaltHash: HCRYPTHASH = 0;
    let mut szUsername: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut dwUsernameLen: DWORD = 0;
    let mut dwError: DWORD = 0;

    /* create hash for salt */
    if salt.is_null()
        || phKey.is_null()
        || CryptCreateHash(
            hProv,
            CRYPT32_PROTECTDATA_HASH_CALG as ALG_ID,
            0 as HCRYPTKEY,
            0 as DWORD,
            core::ptr::addr_of_mut!(hSaltHash),
        ) == 0
    {
        ERR!("CryptCreateHash\n");
        return FALSE as BOOL;
    }

    /* This should be the "logon credentials" instead of username */
    dwError = GetLastError();
    dwUsernameLen = 0 as DWORD;
    if GetUserNameA(core::ptr::null_mut(), core::ptr::addr_of_mut!(dwUsernameLen)) == 0
        && GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD
        && dwUsernameLen != 0
        && {
            szUsername = CryptMemAlloc(dwUsernameLen as ULONG) as *mut ::core::ffi::c_char;
            !szUsername.is_null()
        }
    {
        *szUsername.offset(0 as isize) = '\0' as i32 as ::core::ffi::c_char;
        GetUserNameA(szUsername as LPSTR, core::ptr::addr_of_mut!(dwUsernameLen));
    }
    SetLastError(dwError);

    /* salt the hash with:
     * - the user id
     * - an "internal secret"
     * - randomness (from the salt)
     * - user-supplied entropy
     */
    if (!szUsername.is_null()
        && CryptHashData(
            hSaltHash,
            szUsername as LPBYTE as *const BYTE,
            dwUsernameLen,
            0 as DWORD,
        ) == 0)
        || CryptHashData(
            hSaltHash,
            core::ptr::addr_of!(crypt32_protectdata_secret) as *const BYTE,
            (core::mem::size_of::<[BYTE; 20]>() as usize).wrapping_sub(1 as usize) as DWORD,
            0 as DWORD,
        ) == 0
        || CryptHashData(
            hSaltHash,
            (*salt).pbData,
            (*salt).cbData,
            0 as DWORD,
        ) == 0
        || (!pOptionalEntropy.is_null()
            && CryptHashData(
                hSaltHash,
                (*pOptionalEntropy).pbData,
                (*pOptionalEntropy).cbData,
                0 as DWORD,
            ) == 0)
    {
        ERR!("CryptHashData\n");
        rc = FALSE as BOOL;
    }

    /* produce a symmetric key */
    if rc != 0
        && CryptDeriveKey(
            hProv,
            CRYPT32_PROTECTDATA_KEY_CALG as ALG_ID,
            hSaltHash,
            (key_len << 16) | CRYPT_EXPORTABLE as DWORD,
            phKey,
        ) == 0
    {
        ERR!("CryptDeriveKey\n");
        rc = FALSE as BOOL;
    }

    /* clean up */
    CryptDestroyHash(hSaltHash);
    CryptMemFree(szUsername as LPVOID);

    return rc;
}

unsafe extern "C" fn report(pDataIn: *const DATA_BLOB, pOptionalEntropy: *const DATA_BLOB, pPromptStruct: *mut CRYPTPROTECT_PROMPTSTRUCT, dwFlags: DWORD) {
    if !pPromptStruct.is_null() {
        WINE_TRACE!("pPromptStruct: %p\n", pPromptStruct);
        WINE_TRACE!("  cbSize: 0x%lx\n", (*pPromptStruct).cbSize);
        WINE_TRACE!("  dwPromptFlags: 0x%lx\n", (*pPromptStruct).dwPromptFlags);
        WINE_TRACE!("  hwndApp: %p\n", (*pPromptStruct).hwndApp);
        WINE_TRACE!("  szPrompt: %p %s\n", (*pPromptStruct).szPrompt, if !(*pPromptStruct).szPrompt.is_null() { debugstr_w((*pPromptStruct).szPrompt as *const WCHAR) } else { b"\0" as *const u8 as *const core::ffi::c_char });
    }
    WINE_TRACE!("dwFlags: 0x%04lx\n", dwFlags);
    WINE_TRACE!("%s cbData: %u\n", b"pDataIn\0" as *const u8 as *const core::ffi::c_char, (*pDataIn).cbData as core::ffi::c_uint);
    WINE_TRACE!("%s pbData @ %p:%s\n", b"pDataIn\0" as *const u8 as *const core::ffi::c_char, (*pDataIn).pbData, hex_str((*pDataIn).pbData, (*pDataIn).cbData as core::ffi::c_int));
    if !pOptionalEntropy.is_null() {
        WINE_TRACE!("%s cbData: %u\n", b"pOptionalEntropy\0" as *const u8 as *const core::ffi::c_char, (*pOptionalEntropy).cbData as core::ffi::c_uint);
        WINE_TRACE!("%s pbData @ %p:%s\n", b"pOptionalEntropy\0" as *const u8 as *const core::ffi::c_char, (*pOptionalEntropy).pbData, hex_str( (*pOptionalEntropy).pbData, (*pOptionalEntropy).cbData as core::ffi::c_int, ));
        WINE_TRACE!("  %s\n", debugstr_an( (*pOptionalEntropy).pbData as *const core::ffi::c_char, (*pOptionalEntropy).cbData as core::ffi::c_int, ));
    }
}

#[no_mangle]
pub unsafe extern "system" fn CryptProtectData(
    pDataIn: *mut DATA_BLOB,
    mut szDataDescr: LPCWSTR,
    pOptionalEntropy: *mut DATA_BLOB,
    pvReserved: PVOID,
    pPromptStruct: *mut CRYPTPROTECT_PROMPTSTRUCT,
    dwFlags: DWORD,
    pDataOut: *mut DATA_BLOB,
) -> BOOL {
    let mut rc: BOOL = (FALSE as i32);
    let mut hProv: HCRYPTPROV = 0;
    let mut protect_data: protect_data_t = core::mem::zeroed();
    let mut hHash: HCRYPTHASH = 0;
    let mut hKey: HCRYPTKEY = 0;
    let mut dwLength: DWORD = 0;

    SetLastError(ERROR_SUCCESS);

    'finished: {
        if pDataIn.is_null() || pDataOut.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'finished;
        }

        /* debug: show our arguments */
        report(pDataIn, pOptionalEntropy, pPromptStruct, dwFlags);

        /* Windows appears to create an empty szDataDescr instead of maintaining
         * a NULL */
        if szDataDescr.is_null() {
            szDataDescr = RUSTCONV_WSTR_96A296D224F2.as_ptr();
        }

        /* get crypt context */
        if CryptAcquireContextW(
            core::ptr::addr_of_mut!(hProv),
            core::ptr::null(),
            (MS_ENHANCED_PROV_W).as_ptr(),
            CRYPT32_PROTECTDATA_PROV,
            CRYPT_VERIFYCONTEXT,
        ) == 0
        {
            ERR!("CryptAcquireContextW failed\n");
            break 'finished;
        }

        'free_context: {
            /* populate our structure */
            if fill_protect_data(core::ptr::addr_of_mut!(protect_data), szDataDescr, hProv) == 0 {
                ERR!("fill_protect_data\n");
                break 'free_context;
            }

            'free_protect_data: {
                /* load key */
                if load_encryption_key(
                    hProv,
                    protect_data.cipher_key_len,
                    core::ptr::addr_of_mut!(protect_data.salt),
                    pOptionalEntropy,
                    core::ptr::addr_of_mut!(hKey),
                ) == 0
                {
                    break 'free_protect_data;
                }

                'free_key: {
                    /* create a hash for the encryption validation */
                    if CryptCreateHash(
                        hProv,
                        CRYPT32_PROTECTDATA_HASH_CALG,
                        0,
                        0,
                        core::ptr::addr_of_mut!(hHash),
                    ) == 0
                    {
                        ERR!("CryptCreateHash\n");
                        break 'free_key;
                    }

                    'free_hash: {
                        /* calculate storage required */
                        dwLength = (*pDataIn).cbData;
                        if CryptEncrypt(
                            hKey,
                            0,
                            (TRUE) as BOOL,
                            0,
                            (*pDataIn).pbData,
                            core::ptr::addr_of_mut!(dwLength),
                            0,
                        ) != 0
                            || GetLastError() != ERROR_MORE_DATA
                        {
                            ERR!("CryptEncrypt\n");
                            break 'free_hash;
                        }

                        /* copy plain text into cipher area for CryptEncrypt call */
                        protect_data.cipher.cbData = dwLength;
                        protect_data.cipher.pbData =
                            CryptMemAlloc(protect_data.cipher.cbData as ULONG) as *mut BYTE;
                        if protect_data.cipher.pbData.is_null() {
                            ERR!("CryptMemAlloc\n");
                            break 'free_hash;
                        }
                        memcpy(
                            protect_data.cipher.pbData as *mut core::ffi::c_void,
                            (*pDataIn).pbData as *const core::ffi::c_void,
                            (*pDataIn).cbData as usize,
                        );

                        /* encrypt! */
                        dwLength = (*pDataIn).cbData;
                        if CryptEncrypt(
                            hKey,
                            hHash,
                            (TRUE) as BOOL,
                            0,
                            protect_data.cipher.pbData,
                            core::ptr::addr_of_mut!(dwLength),
                            protect_data.cipher.cbData,
                        ) == 0
                        {
                            ERR!("CryptEncrypt %lu\n", GetLastError());
                            break 'free_hash;
                        }
                        protect_data.cipher.cbData = dwLength;

                        /* debug: show the cipher */
                        TRACE_DATA_BLOB!(core::ptr::addr_of_mut!(protect_data.cipher));

                        /* attach our fingerprint */
                        if convert_hash_to_blob(
                            hHash,
                            core::ptr::addr_of_mut!(protect_data.fingerprint),
                        ) == 0
                        {
                            ERR!("convert_hash_to_blob\n");
                            break 'free_hash;
                        }

                        /* serialize into an opaque blob */
                        if serialize(core::ptr::addr_of_mut!(protect_data), pDataOut) == 0 {
                            ERR!("serialize\n");
                            break 'free_hash;
                        }

                        /* success! */
                        rc = (TRUE as i32);
                    }
                    // free_hash:
                    CryptDestroyHash(hHash);
                }
                // free_key:
                CryptDestroyKey(hKey);
            }
            // free_protect_data:
            free_protect_data(core::ptr::addr_of_mut!(protect_data));
        }
        // free_context:
        CryptReleaseContext(hProv, 0);
    }
    // finished:
    /* If some error occurred, and no error code was set, force one. */
    if rc == 0 && GetLastError() == ERROR_SUCCESS {
        SetLastError(ERROR_INVALID_DATA);
    }

    if rc != 0 {
        SetLastError(ERROR_SUCCESS);

        TRACE_DATA_BLOB!(pDataOut);
    }

    return rc;
}

#[no_mangle]
pub unsafe extern "system" fn CryptUnprotectData(
    pDataIn: *mut DATA_BLOB,
    ppszDataDescr: *mut LPWSTR,
    pOptionalEntropy: *mut DATA_BLOB,
    pvReserved: PVOID,
    pPromptStruct: *mut CRYPTPROTECT_PROMPTSTRUCT,
    dwFlags: DWORD,
    pDataOut: *mut DATA_BLOB,
) -> BOOL {
    let mut rc: BOOL = (FALSE as i32);

    let mut hProv: HCRYPTPROV = 0;
    let mut protect_data: protect_data_t = core::mem::zeroed();
    let mut hHash: HCRYPTHASH = 0;
    let mut hKey: HCRYPTKEY = 0;
    let mut dwLength: DWORD = 0;

    let announce_bad_opaque_data: *const i8 = b"CryptUnprotectData received a DATA_BLOB that seems to have NOT been generated by Wine.  Please enable tracing ('export WINEDEBUG=crypt') to see details.\0".as_ptr() as *const i8;

    TRACE!("called\n");

    SetLastError(ERROR_SUCCESS);

    'finished: {
        'free_protect_data: {
            'free_context: {
                'free_key: {
                    'free_hash: {
                        if pDataIn.is_null() || pDataOut.is_null() {
                            SetLastError(ERROR_INVALID_PARAMETER);
                            break 'finished;
                        }
                        if (*pDataIn).cbData == 0 {
                            SetLastError(ERROR_INVALID_DATA);
                            break 'finished;
                        }

                        /* debug: show our arguments */
                        report(pDataIn, pOptionalEntropy, pPromptStruct, dwFlags);

                        /* take apart the opaque blob */
                        if unserialize(pDataIn, core::ptr::addr_of_mut!(protect_data)) == 0 {
                            SetLastError(ERROR_INVALID_DATA);
                            break 'finished;
                        }

                        /* perform basic validation on the resulting structure */
                        if valid_protect_data(core::ptr::addr_of_mut!(protect_data)) == 0 {
                            SetLastError(ERROR_INVALID_DATA);
                            break 'free_protect_data;
                        }

                        /* get a crypt context */
                        if CryptAcquireContextW(
                            core::ptr::addr_of_mut!(hProv),
                            core::ptr::null_mut(),
                            (MS_ENHANCED_PROV_W).as_ptr(),
                            CRYPT32_PROTECTDATA_PROV,
                            CRYPT_VERIFYCONTEXT,
                        ) == 0
                        {
                            ERR!("CryptAcquireContextW failed\n");
                            break 'free_protect_data;
                        }

                        /* load key */
                        if load_encryption_key(
                            hProv,
                            protect_data.cipher_key_len,
                            core::ptr::addr_of_mut!(protect_data.salt),
                            pOptionalEntropy,
                            core::ptr::addr_of_mut!(hKey),
                        ) == 0
                        {
                            break 'free_context;
                        }

                        /* create a hash for the decryption validation */
                        if CryptCreateHash(
                            hProv,
                            CRYPT32_PROTECTDATA_HASH_CALG,
                            0,
                            0,
                            core::ptr::addr_of_mut!(hHash),
                        ) == 0
                        {
                            ERR!("CryptCreateHash\n");
                            break 'free_key;
                        }

                        /* prepare for plaintext */
                        (*pDataOut).cbData = protect_data.cipher.cbData;
                        (*pDataOut).pbData = LocalAlloc(
                            LPTR,
                            (*pDataOut).cbData as SIZE_T,
                        ) as *mut BYTE;
                        if (*pDataOut).pbData.is_null() {
                            ERR!("CryptMemAlloc\n");
                            break 'free_hash;
                        }
                        memcpy(
                            (*pDataOut).pbData as *mut core::ffi::c_void,
                            protect_data.cipher.pbData as *const core::ffi::c_void,
                            protect_data.cipher.cbData as usize,
                        );

                        /* decrypt! */
                        if CryptDecrypt(
                            hKey,
                            hHash,
                            (TRUE) as BOOL,
                            0,
                            (*pDataOut).pbData,
                            core::ptr::addr_of_mut!((*pDataOut).cbData),
                        ) == 0
                            /* check the hash fingerprint */
                            || (*pDataOut).cbData > protect_data.cipher.cbData
                            || hash_matches_blob(
                                hHash,
                                core::ptr::addr_of_mut!(protect_data.fingerprint),
                            ) == 0
                        {
                            SetLastError(ERROR_INVALID_DATA);

                            LocalFree(((*pDataOut).pbData).cast());
                            (*pDataOut).pbData = core::ptr::null_mut();
                            (*pDataOut).cbData = 0;

                            break 'free_hash;
                        }

                        /* Copy out the description */
                        dwLength = ((lstrlenW(protect_data.szDataDescr as LPCWSTR)
                            + 1) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
                        if !ppszDataDescr.is_null() {
                            *ppszDataDescr = LocalAlloc(
                                LPTR,
                                dwLength as SIZE_T,
                            ) as LPWSTR;
                            if (*ppszDataDescr).is_null() {
                                ERR!("LocalAlloc (ppszDataDescr)\n");
                                break 'free_hash;
                            } else {
                                memcpy(
                                    *ppszDataDescr as *mut core::ffi::c_void,
                                    protect_data.szDataDescr as *const core::ffi::c_void,
                                    dwLength as usize,
                                );
                            }
                        }

                        /* success! */
                        rc = (TRUE as i32);
                    }
                    /* free_hash: */
                    CryptDestroyHash(hHash);
                }
                /* free_key: */
                CryptDestroyKey(hKey);
            }
            /* free_context: */
            CryptReleaseContext(hProv, 0);
        }
        /* free_protect_data: */
        free_protect_data(core::ptr::addr_of_mut!(protect_data));
    }
    /* finished: */
    /* If some error occurred, and no error code was set, force one. */
    if rc == 0 && GetLastError() == ERROR_SUCCESS {
        SetLastError(ERROR_INVALID_DATA);
    }

    if rc != 0 {
        SetLastError(ERROR_SUCCESS);

        if !ppszDataDescr.is_null() {
        }
        TRACE_DATA_BLOB!(pDataOut);
    }

    TRACE!("returning %s\n", if rc != 0 { "ok" } else { "FAIL" });

    return rc;
}
