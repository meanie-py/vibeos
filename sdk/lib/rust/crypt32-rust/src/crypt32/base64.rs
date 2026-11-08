// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CERT_HEADER: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"-----BEGIN CERTIFICATE-----\0")
};

pub static CERT_HEADER_START: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"-----BEGIN \0") };

pub static CERT_DELIMITER: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"-----\0") };

pub static CERT_TRAILER: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"-----END CERTIFICATE-----\0")
};

pub static CERT_TRAILER_START: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"-----END \0") };

pub static CERT_REQUEST_HEADER: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"-----BEGIN NEW CERTIFICATE REQUEST-----\0",
    )
};

pub static CERT_REQUEST_TRAILER: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"-----END NEW CERTIFICATE REQUEST-----\0",
    )
};

pub static X509_HEADER: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"-----BEGIN X509 CRL-----\0")
};

pub static X509_TRAILER: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"-----END X509 CRL-----\0")
};

static mut CERT_HEADER_W: [WCHAR; 28] = unsafe {
    ::core::mem::transmute::<[u8; 56], [WCHAR; 28]>(
        *b"-\0-\0-\0-\0-\0B\0E\0G\0I\0N\0 \0C\0E\0R\0T\0I\0F\0I\0C\0A\0T\0E\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut CERT_HEADER_START_W: [WCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [WCHAR; 12]>(*b"-\0-\0-\0-\0-\0B\0E\0G\0I\0N\0 \0\0\0")
};

static mut CERT_DELIMITER_W: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b"-\0-\0-\0-\0-\0\0\0") };

static mut CERT_TRAILER_W: [WCHAR; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [WCHAR; 26]>(
        *b"-\0-\0-\0-\0-\0E\0N\0D\0 \0C\0E\0R\0T\0I\0F\0I\0C\0A\0T\0E\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut CERT_TRAILER_START_W: [WCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 20], [WCHAR; 10]>(*b"-\0-\0-\0-\0-\0E\0N\0D\0 \0\0\0") };

static mut CERT_REQUEST_HEADER_W: [WCHAR; 40] = unsafe {
    ::core::mem::transmute::<
        [u8; 80],
        [WCHAR; 40],
    >(
        *b"-\0-\0-\0-\0-\0B\0E\0G\0I\0N\0 \0N\0E\0W\0 \0C\0E\0R\0T\0I\0F\0I\0C\0A\0T\0E\0 \0R\0E\0Q\0U\0E\0S\0T\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut CERT_REQUEST_TRAILER_W: [WCHAR; 38] = unsafe {
    ::core::mem::transmute::<
        [u8; 76],
        [WCHAR; 38],
    >(
        *b"-\0-\0-\0-\0-\0E\0N\0D\0 \0N\0E\0W\0 \0C\0E\0R\0T\0I\0F\0I\0C\0A\0T\0E\0 \0R\0E\0Q\0U\0E\0S\0T\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut X509_HEADER_W: [WCHAR; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [WCHAR; 25]>(
        *b"-\0-\0-\0-\0-\0B\0E\0G\0I\0N\0 \0X\x005\x000\09\0 \0C\0R\0L\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut X509_TRAILER_W: [WCHAR; 23] = unsafe {
    ::core::mem::transmute::<[u8; 46], [WCHAR; 23]>(
        *b"-\0-\0-\0-\0-\0E\0N\0D\0 \0X\x005\x000\09\0 \0C\0R\0L\0-\0-\0-\0-\0-\0\0\0",
    )
};

static mut b64: [::core::ffi::c_char; 65] = unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/\0",
    )
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt);

pub type BinaryToStringAFunc = Option<unsafe extern "C" fn(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPSTR,
    pcchString: *mut DWORD,
) -> BOOL>;
pub type BinaryToStringWFunc = Option<unsafe extern "C" fn(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPWSTR,
    pcchString: *mut DWORD,
) -> BOOL>;

pub const BASE64_DECODE_PADDING: DWORD = 0x100;
pub const BASE64_DECODE_WHITESPACE: DWORD = 0x200;
pub const BASE64_DECODE_INVALID: DWORD = 0x300;

pub type StringToBinaryAFunc = Option<unsafe extern "C" fn(
    pszString: LPCSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG>;
pub type StringToBinaryWFunc = Option<unsafe extern "C" fn(
    pszString: LPCWSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG>;

unsafe extern "C" fn EncodeBinaryToBinaryA(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPSTR,
    pcchString: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if !pszString.is_null() {
        if *pcchString < cbBinary {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            ret = (FALSE as i32);
        } else if cbBinary != 0 {
            memcpy(
                pszString as *mut ::core::ffi::c_void,
                pbBinary as *const ::core::ffi::c_void,
                cbBinary as usize,
            );
        }
    } else {
        *pcchString = cbBinary;
    }

    return ret;
}

unsafe extern "C" fn stradd(ptr: LPSTR, end: LPCSTR, s: LPCSTR, mut slen: DWORD) -> DWORD {
    if ptr.offset(slen as isize) > end as LPSTR {
        slen = end.offset_from(ptr as LPCSTR) as DWORD;
    }
    memcpy(ptr as *mut core::ffi::c_void, s as *const core::ffi::c_void, slen as usize);
    return slen;
}

unsafe extern "C" fn encodeBase64A(
    in_buf: *const BYTE,
    in_len: ::core::ffi::c_int,
    sep: LPCSTR,
    out_buf: *mut ::core::ffi::c_char,
    out_len: *mut DWORD,
) -> DWORD {
    let mut div: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut d: *const BYTE = in_buf;
    let mut bytes: ::core::ffi::c_int = (in_len * 8 as ::core::ffi::c_int
        + 5 as ::core::ffi::c_int)
        / 6 as ::core::ffi::c_int;
    let mut pad_bytes: ::core::ffi::c_int = if bytes % 4 as ::core::ffi::c_int
        != 0
    {
        4 as ::core::ffi::c_int
            - bytes % 4 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut ptr: LPSTR = ::core::ptr::null_mut::<CHAR>();
    let mut end: LPCSTR = ::core::ptr::null::<CHAR>();
    let mut chunk: [::core::ffi::c_char; 4 as usize] =
        [0 as ::core::ffi::c_char; 4 as usize];

    if out_buf.is_null() {
        *out_len = (bytes + pad_bytes) as DWORD;
        let t: DWORD = (*out_len)
            .wrapping_div(64 as DWORD)
            .wrapping_add(
                (if (*out_len).wrapping_rem(64 as DWORD) != 0 {
                    1
                } else {
                    0
                }) as DWORD,
            );
        *out_len = (*out_len).wrapping_add(
            t.wrapping_mul(strlen(sep as *const ::core::ffi::c_char) as DWORD)
                .wrapping_add(1 as DWORD),
        );
        return 0 as DWORD;
    }

    /* Three bytes of input give 4 chars of output */
    div = in_len / 3 as ::core::ffi::c_int;

    ptr = out_buf as LPSTR;
    end = ptr.offset(*out_len as isize) as LPCSTR;
    i = 0 as ::core::ffi::c_int;
    while div > 0 as ::core::ffi::c_int && ptr < (end as LPSTR) {
        /* first char is the first 6 bits of the first byte*/
        chunk[0 as usize] = b64[((*d.offset(0 as isize)
            as ::core::ffi::c_int)
            >> 2 as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            as usize];
        /* second char is the last 2 bits of the first byte and the first 4
         * bits of the second byte */
        chunk[1 as usize] = b64[(((*d.offset(0 as isize)
            as ::core::ffi::c_int)
            << 4 as ::core::ffi::c_int)
            & 0x30 as ::core::ffi::c_int
            | (*d.offset(1 as isize) as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int
                & 0x0f as ::core::ffi::c_int))
            as usize];
        /* third char is the last 4 bits of the second byte and the first 2
         * bits of the third byte */
        chunk[2 as usize] = b64[(((*d.offset(1 as isize)
            as ::core::ffi::c_int)
            << 2 as ::core::ffi::c_int)
            & 0x3c as ::core::ffi::c_int
            | (*d.offset(2 as isize) as ::core::ffi::c_int
                >> 6 as ::core::ffi::c_int
                & 0x03 as ::core::ffi::c_int))
            as usize];
        /* fourth char is the remaining 6 bits of the third byte */
        chunk[3 as usize] = b64[(*d.offset(2 as isize)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            as usize];
        ptr = ptr.offset(stradd(
            ptr,
            end,
            core::ptr::addr_of_mut!(chunk) as *mut ::core::ffi::c_char as LPCSTR,
            4 as DWORD,
        ) as isize);
        i += 4 as ::core::ffi::c_int;
        d = d.offset(3 as isize);
        div -= 1;

        if i != 0
            && i % (64 as ::core::ffi::c_int)
                == (0 as ::core::ffi::c_int)
        {
            ptr = ptr.offset(stradd(
                ptr,
                end,
                sep,
                strlen(sep as *const ::core::ffi::c_char) as DWORD,
            ) as isize);
        }
    }

    match pad_bytes {
        1 => {
            /* first char is the first 6 bits of the first byte*/
            chunk[0 as usize] = b64[((*d
                .offset(0 as isize)
                as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int
                & 0x3f as ::core::ffi::c_int)
                as usize];
            /* second char is the last 2 bits of the first byte and the first 4
             * bits of the second byte */
            chunk[1 as usize] = b64[(((*d
                .offset(0 as isize)
                as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int)
                & 0x30 as ::core::ffi::c_int
                | (*d.offset(1 as isize) as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 0x0f as ::core::ffi::c_int))
                as usize];
            /* third char is the last 4 bits of the second byte padded with
             * two zeroes */
            chunk[2 as usize] = b64[(((*d
                .offset(1 as isize)
                as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int)
                & 0x3c as ::core::ffi::c_int)
                as usize];
            /* fourth char is a = to indicate one byte of padding */
            chunk[3 as usize] = '=' as i32 as ::core::ffi::c_char;
            ptr = ptr.offset(stradd(
                ptr,
                end,
                core::ptr::addr_of_mut!(chunk) as *mut ::core::ffi::c_char as LPCSTR,
                4 as DWORD,
            ) as isize);
        }
        2 => {
            /* first char is the first 6 bits of the first byte*/
            chunk[0 as usize] = b64[((*d
                .offset(0 as isize)
                as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int
                & 0x3f as ::core::ffi::c_int)
                as usize];
            /* second char is the last 2 bits of the first byte padded with
             * four zeroes*/
            chunk[1 as usize] = b64[(((*d
                .offset(0 as isize)
                as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int)
                & 0x30 as ::core::ffi::c_int)
                as usize];
            /* third char is = to indicate padding */
            chunk[2 as usize] = '=' as i32 as ::core::ffi::c_char;
            /* fourth char is = to indicate padding */
            chunk[3 as usize] = '=' as i32 as ::core::ffi::c_char;
            ptr = ptr.offset(stradd(
                ptr,
                end,
                core::ptr::addr_of_mut!(chunk) as *mut ::core::ffi::c_char as LPCSTR,
                4 as DWORD,
            ) as isize);
        }
        _ => {}
    }
    ptr = ptr.offset(stradd(
        ptr,
        end,
        sep,
        strlen(sep as *const ::core::ffi::c_char) as DWORD,
    ) as isize);

    return ptr.offset_from(out_buf) as ::core::ffi::c_longlong as DWORD;
}

unsafe extern "C" fn BinaryToBase64A(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPSTR,
    pcchString: *mut DWORD,
) -> BOOL {
    static crlf: [CHAR; 3] = [b'\r' as CHAR, b'\n' as CHAR, 0];
    static lf: [CHAR; 2] = [b'\n' as CHAR, 0];
    let mut ret: BOOL = (TRUE as i32);
    let mut header: LPCSTR = core::ptr::null();
    let mut trailer: LPCSTR = core::ptr::null();
    let mut sep: LPCSTR;
    let mut charsNeeded: DWORD;

    if (dwFlags & CRYPT_STRING_NOCR) != 0 {
        sep = core::ptr::addr_of!(lf) as LPCSTR;
    } else if (dwFlags & CRYPT_STRING_NOCRLF) != 0 {
        sep = b"\0".as_ptr() as LPCSTR;
    } else {
        sep = core::ptr::addr_of!(crlf) as LPCSTR;
    }
    match dwFlags & 0x0fffffff {
        CRYPT_STRING_BASE64 => {
            /* no header or footer */
        }
        CRYPT_STRING_BASE64HEADER => {
            header = (CERT_HEADER).as_ptr();
            trailer = (CERT_TRAILER).as_ptr();
        }
        CRYPT_STRING_BASE64REQUESTHEADER => {
            header = (CERT_REQUEST_HEADER).as_ptr();
            trailer = (CERT_REQUEST_TRAILER).as_ptr();
        }
        CRYPT_STRING_BASE64X509CRLHEADER => {
            header = (X509_HEADER).as_ptr();
            trailer = (X509_TRAILER).as_ptr();
        }
        _ => {}
    }

    charsNeeded = 0;
    encodeBase64A(pbBinary, cbBinary as i32, sep, core::ptr::null_mut(), &mut charsNeeded);
    if !header.is_null() {
        charsNeeded = charsNeeded.wrapping_add(
            (strlen(header) + strlen(sep)) as DWORD,
        );
    }
    if !trailer.is_null() {
        charsNeeded = charsNeeded.wrapping_add(
            (strlen(trailer) + strlen(sep)) as DWORD,
        );
    }

    if !pszString.is_null() {
        let mut ptr: LPSTR = pszString;
        let mut size: DWORD = *pcchString;
        let mut end: LPSTR = ptr.add(size as usize);

        if !header.is_null() {
            ptr = ptr.add(stradd(
                ptr,
                end as LPCSTR,
                header,
                strlen(header) as DWORD,
            ) as usize);
            ptr = ptr.add(stradd(
                ptr,
                end as LPCSTR,
                sep,
                strlen(sep) as DWORD,
            ) as usize);
            size = end.offset_from(ptr) as DWORD;
        }
        ptr = ptr.add(encodeBase64A(
            pbBinary,
            cbBinary as i32,
            sep,
            ptr,
            &mut size,
        ) as usize);
        if !trailer.is_null() {
            ptr = ptr.add(stradd(
                ptr,
                end as LPCSTR,
                trailer,
                strlen(trailer) as DWORD,
            ) as usize);
            ptr = ptr.add(stradd(
                ptr,
                end as LPCSTR,
                sep,
                strlen(sep) as DWORD,
            ) as usize);
        }

        if ptr < end {
            *ptr = b'\0' as CHAR;
        }

        if charsNeeded <= *pcchString {
            *pcchString = charsNeeded.wrapping_sub(1);
        } else {
            *pcchString = charsNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        }
    } else {
        *pcchString = charsNeeded;
    }

    return ret;
}

unsafe extern "C" fn BinaryToHexRawA(
    mut bin: *const BYTE,
    mut nbin: DWORD,
    flags: DWORD,
    mut str: *mut ::core::ffi::c_char,
    nstr: *mut DWORD,
) -> BOOL {
    static hex: [u8; 17] = *b"0123456789abcdef\0";
    let mut needed: DWORD = 0;

    if (flags & CRYPT_STRING_NOCRLF) != 0 {
        needed = 0;
    } else if (flags & CRYPT_STRING_NOCR) != 0 {
        needed = 1;
    } else {
        needed = 2;
    }

    needed = needed.wrapping_add(
        nbin.wrapping_mul(2).wrapping_add(1),
    );

    if str.is_null() {
        *nstr = needed;
        return (TRUE) as BOOL;
    }

    if needed > *nstr && *nstr < 3 {
        SetLastError(ERROR_MORE_DATA);
        return (FALSE) as BOOL;
    }

    nbin = min!(
        nbin,
        (*nstr).wrapping_sub(1) / 2,
    );

    loop {
        let fresh_count = nbin;
        nbin = nbin.wrapping_sub(1);
        if !(fresh_count != 0) {
            break;
        }
        let p_hi = str;
        str = str.offset(1);
        *p_hi = hex[((*bin >> 4) & 0xf) as usize]
            as ::core::ffi::c_char;
        let p_lo = str;
        str = str.offset(1);
        *p_lo = hex[(*bin & 0xf) as usize] as ::core::ffi::c_char;
        bin = bin.offset(1);
    }

    if needed > *nstr {
        *str = 0;
        SetLastError(ERROR_MORE_DATA);
        return (FALSE) as BOOL;
    }

    if (flags & CRYPT_STRING_NOCR) != 0 {
        let p_nl = str;
        str = str.offset(1);
        *p_nl = '\n' as i32 as ::core::ffi::c_char;
    } else if (flags & CRYPT_STRING_NOCRLF) == 0 {
        let p_cr = str;
        str = str.offset(1);
        *p_cr = '\r' as i32 as ::core::ffi::c_char;
        let p_lf = str;
        str = str.offset(1);
        *p_lf = '\n' as i32 as ::core::ffi::c_char;
    }

    *str = 0;
    *nstr = needed.wrapping_sub(1);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptBinaryToStringA(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPSTR,
    pcchString: *mut DWORD,
) -> BOOL {
    let mut encoder: BinaryToStringAFunc = None;

    if pbBinary.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if pcchString.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    match dwFlags & 0x0fffffff {
        CRYPT_STRING_BINARY => {
            encoder = Some(EncodeBinaryToBinaryA);
        }
        CRYPT_STRING_BASE64 | CRYPT_STRING_BASE64HEADER | CRYPT_STRING_BASE64REQUESTHEADER | CRYPT_STRING_BASE64X509CRLHEADER => {
            encoder = Some(BinaryToBase64A);
        }
        CRYPT_STRING_HEXRAW => {
            encoder = Some(BinaryToHexRawA);
        }
        CRYPT_STRING_HEX | CRYPT_STRING_HEXASCII | CRYPT_STRING_HEXADDR | CRYPT_STRING_HEXASCIIADDR => {
            FIXME!("Unimplemented type %ld\n", dwFlags & 0x0fffffff);
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }
    let encoder = encoder.unwrap_unchecked();
    return encoder(pbBinary, cbBinary, dwFlags, pszString, pcchString);
}

unsafe extern "C" fn EncodeBinaryToBinaryW(in_buf: *const BYTE, in_len: DWORD, flags: DWORD, out_buf: *mut WCHAR, out_len: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if !out_buf.is_null() {
        if *out_len < in_len {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            ret = (FALSE as i32);
        } else if in_len != 0 {
            memcpy(out_buf as *mut core::ffi::c_void, in_buf as *const core::ffi::c_void, in_len as usize);
        }
    } else {
        *out_len = in_len;
    }

    return ret;
}

unsafe extern "C" fn encodeBase64W(
    in_buf: *const BYTE,
    in_len: ::core::ffi::c_int,
    sep: LPCWSTR,
    out_buf: *mut WCHAR,
    out_len: *mut DWORD,
) -> LONG {
    let mut div: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut d: *const BYTE = in_buf;
    let mut bytes: ::core::ffi::c_int =
        (in_len * 8 + 5) / 6;
    let mut pad_bytes: ::core::ffi::c_int = if bytes % 4 != 0 {
        4 - bytes % 4
    } else {
        0
    };
    let mut needed: DWORD = 0;
    let mut ptr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();

    WINE_TRACE!("bytes is %d, pad bytes is %d\n", bytes, pad_bytes);

    needed = (bytes + pad_bytes) as DWORD;
    needed = needed.wrapping_add(
        needed
            .wrapping_div(64 as DWORD)
            .wrapping_add(
                (if needed.wrapping_rem(64 as DWORD) != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as DWORD,
            )
            .wrapping_mul(lstrlenW(sep) as DWORD),
    );
    needed = needed.wrapping_add(1);

    if needed > *out_len {
        *out_len = needed;
        return ERROR_INSUFFICIENT_BUFFER as LONG;
    } else {
        *out_len = needed;
    }

    /* Three bytes of input give 4 chars of output */
    div = in_len / 3;

    ptr = out_buf as LPWSTR;
    i = 0;
    while div > 0 {
        let fresh37 = ptr;
        ptr = ptr.offset(1);
        *fresh37 = b64[(*d.offset(0 as isize) as ::core::ffi::c_int
            >> 2
            & 0x3f as ::core::ffi::c_int) as usize] as WCHAR;
        let fresh38 = ptr;
        ptr = ptr.offset(1);
        *fresh38 = b64[((*d.offset(0 as isize) as ::core::ffi::c_int)
            << 4
            & 0x30 as ::core::ffi::c_int
            | *d.offset(1 as isize) as ::core::ffi::c_int
                >> 4
                & 0x0f as ::core::ffi::c_int) as usize] as WCHAR;
        let fresh39 = ptr;
        ptr = ptr.offset(1);
        *fresh39 = b64[((*d.offset(1 as isize) as ::core::ffi::c_int)
            << 2
            & 0x3c as ::core::ffi::c_int
            | *d.offset(2 as isize) as ::core::ffi::c_int
                >> 6
                & 0x03 as ::core::ffi::c_int) as usize] as WCHAR;
        let fresh40 = ptr;
        ptr = ptr.offset(1);
        *fresh40 = b64[(*d.offset(2 as isize) as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int) as usize] as WCHAR;
        i += 4;
        d = d.offset(3 as isize);
        div -= 1;

        if i != 0 && i % 64 == 0 {
            lstrcpyW(ptr, sep);
            ptr = ptr.offset(lstrlenW(sep) as isize);
        }
    }

    match pad_bytes {
        1 => {
            let fresh41 = ptr;
            ptr = ptr.offset(1);
            *fresh41 = b64[(*d.offset(0 as isize) as ::core::ffi::c_int
                >> 2
                & 0x3f as ::core::ffi::c_int) as usize] as WCHAR;
            let fresh42 = ptr;
            ptr = ptr.offset(1);
            *fresh42 = b64[((*d.offset(0 as isize) as ::core::ffi::c_int)
                << 4
                & 0x30 as ::core::ffi::c_int
                | *d.offset(1 as isize) as ::core::ffi::c_int
                    >> 4
                    & 0x0f as ::core::ffi::c_int) as usize] as WCHAR;
            let fresh43 = ptr;
            ptr = ptr.offset(1);
            *fresh43 = b64[((*d.offset(1 as isize) as ::core::ffi::c_int)
                << 2
                & 0x3c as ::core::ffi::c_int) as usize] as WCHAR;
            let fresh44 = ptr;
            ptr = ptr.offset(1);
            *fresh44 = '=' as i32 as WCHAR;
        }
        2 => {
            let fresh45 = ptr;
            ptr = ptr.offset(1);
            *fresh45 = b64[(*d.offset(0 as isize) as ::core::ffi::c_int
                >> 2
                & 0x3f as ::core::ffi::c_int) as usize] as WCHAR;
            let fresh46 = ptr;
            ptr = ptr.offset(1);
            *fresh46 = b64[((*d.offset(0 as isize) as ::core::ffi::c_int)
                << 4
                & 0x30 as ::core::ffi::c_int) as usize] as WCHAR;
            let fresh47 = ptr;
            ptr = ptr.offset(1);
            *fresh47 = '=' as i32 as WCHAR;
            let fresh48 = ptr;
            ptr = ptr.offset(1);
            *fresh48 = '=' as i32 as WCHAR;
        }
        _ => {}
    }

    lstrcpyW(ptr, sep);

    return ERROR_SUCCESS as LONG;
}

unsafe extern "C" fn BinaryToBase64W(
    mut pbBinary: *const BYTE,
    mut cbBinary: DWORD,
    mut dwFlags: DWORD,
    mut pszString: LPWSTR,
    mut pcchString: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut header: LPCWSTR = core::ptr::null_mut();
    let mut trailer: LPCWSTR = core::ptr::null_mut();
    let mut sep: LPCWSTR;
    let mut charsNeeded: DWORD;

    if (dwFlags & CRYPT_STRING_NOCR as DWORD) != 0 {
        sep = RUSTCONV_WSTR_075DE2B906DB.as_ptr() as LPCWSTR;
    } else if (dwFlags & CRYPT_STRING_NOCRLF as DWORD) != 0 {
        sep = RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPCWSTR;
    } else {
        sep = RUSTCONV_WSTR_80E09B2F645F.as_ptr() as LPCWSTR;
    }
    match dwFlags & 0x0fffffff as DWORD {
        CRYPT_STRING_BASE64 => {
            // no header or footer
        }
        CRYPT_STRING_BASE64HEADER => {
            header = (CERT_HEADER_W).as_ptr() as LPCWSTR;
            trailer = (CERT_TRAILER_W).as_ptr() as LPCWSTR;
        }
        CRYPT_STRING_BASE64REQUESTHEADER => {
            header = (CERT_REQUEST_HEADER_W).as_ptr() as LPCWSTR;
            trailer = (CERT_REQUEST_TRAILER_W).as_ptr() as LPCWSTR;
        }
        CRYPT_STRING_BASE64X509CRLHEADER => {
            header = (X509_HEADER_W).as_ptr() as LPCWSTR;
            trailer = (X509_TRAILER_W).as_ptr() as LPCWSTR;
        }
        _ => {}
    }

    charsNeeded = 0 as DWORD;
    encodeBase64W(
        pbBinary,
        cbBinary as core::ffi::c_int,
        sep,
        core::ptr::null_mut(),
        &raw mut charsNeeded,
    );
    if !header.is_null() {
        charsNeeded = charsNeeded.wrapping_add(
            (lstrlenW(header) + lstrlenW(sep)) as DWORD,
        );
    }
    if !trailer.is_null() {
        charsNeeded = charsNeeded.wrapping_add(
            (lstrlenW(trailer) + lstrlenW(sep)) as DWORD,
        );
    }

    if !pszString.is_null() {
        if charsNeeded <= *pcchString {
            let mut ptr: LPWSTR = pszString;
            let mut size: DWORD = charsNeeded;

            if !header.is_null() {
                lstrcpyW(ptr, header);
                ptr = ptr.offset(lstrlenW(ptr as LPCWSTR) as isize);
                lstrcpyW(ptr, sep);
                ptr = ptr.offset(lstrlenW(sep) as isize);
            }
            encodeBase64W(
                pbBinary,
                cbBinary as core::ffi::c_int,
                sep,
                ptr as *mut WCHAR,
                &raw mut size,
            );
            ptr = ptr.offset(size.wrapping_sub(1 as DWORD) as isize);
            if !trailer.is_null() {
                lstrcpyW(ptr, trailer);
                ptr = ptr.offset(lstrlenW(ptr as LPCWSTR) as isize);
                lstrcpyW(ptr, sep);
            }
            *pcchString = charsNeeded.wrapping_sub(1 as DWORD);
        } else {
            *pcchString = charsNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        }
    } else {
        *pcchString = charsNeeded;
    }

    return ret;
}

unsafe extern "C" fn BinaryToHexRawW(
    mut bin: *const BYTE,
    mut nbin: DWORD,
    flags: DWORD,
    mut str: LPWSTR,
    nstr: *mut DWORD,
) -> BOOL {
    static hex: [u16; 17] = [
    0x0030u16, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039,
    0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0000,
];
    let mut needed: DWORD;

    if flags & CRYPT_STRING_NOCRLF != 0 {
        needed = 0;
    } else if flags & CRYPT_STRING_NOCR != 0 {
        needed = 1;
    } else {
        needed = 2;
    }

    needed = needed.wrapping_add(nbin.wrapping_mul(2).wrapping_add(1));

    if str.is_null() {
        *nstr = needed;
        return (TRUE) as BOOL;
    }

    if needed > *nstr {
        SetLastError(ERROR_MORE_DATA);
        return (FALSE) as BOOL;
    }

    while nbin != 0 {
        nbin = nbin.wrapping_sub(1);
        *str = hex[((*bin >> 4) & 0xf) as usize];
        str = str.add(1);
        *str = hex[(*bin & 0xf) as usize];
        str = str.add(1);
        bin = bin.add(1);
    }

    if flags & CRYPT_STRING_NOCR != 0 {
        *str = '\n' as u16;
        str = str.add(1);
    } else if flags & CRYPT_STRING_NOCRLF == 0 {
        *str = '\r' as u16;
        str = str.add(1);
        *str = '\n' as u16;
        str = str.add(1);
    }

    *str = 0;
    *nstr = needed.wrapping_sub(1);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn binary_to_hexW(
    bin: *const BYTE,
    nbin: DWORD,
    flags: DWORD,
    mut str: LPWSTR,
    nstr: *mut DWORD,
) -> BOOL {
    let hex: [WCHAR; 17] = [
    0x0030u16, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039,
    0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0000,
];
    let mut needed: DWORD;
    let mut i: DWORD;

    needed = nbin.wrapping_mul(3 as DWORD); /* spaces + terminating \0 */

    if (flags & (CRYPT_STRING_NOCR as DWORD)) != 0 {
        needed = needed.wrapping_add(
            (nbin.wrapping_add(7 as DWORD))
                .wrapping_div(16 as DWORD),
        ); /* space every 16 characters */
        needed = needed.wrapping_add(1 as DWORD); /* terminating \n */
    } else if (flags & (CRYPT_STRING_NOCRLF as DWORD)) == 0 {
        needed = needed.wrapping_add(
            (nbin.wrapping_add(7 as DWORD))
                .wrapping_div(16 as DWORD),
        ); /* space every 16 characters */
        needed = needed.wrapping_add(
            nbin.wrapping_div(16 as DWORD)
                .wrapping_add(1 as DWORD),
        ); /* LF every 16 characters + terminating \r */

        if nbin.wrapping_rem(16 as DWORD) != 0 {
            needed = needed.wrapping_add(1 as DWORD); /* terminating \n */
        }
    }

    if str.is_null() {
        *nstr = needed;
        return TRUE as BOOL;
    }

    if needed > *nstr {
        SetLastError(ERROR_MORE_DATA as DWORD);
        return FALSE as BOOL;
    }

    i = 0 as DWORD;
    while i < nbin {
        'body: {
            *str = hex[((*bin.add(i as usize) as DWORD >> (4 as DWORD))
                & (0xf as DWORD)) as usize];
            str = str.add(1);
            *str = hex[((*bin.add(i as usize) as DWORD) & (0xf as DWORD))
                as usize];
            str = str.add(1);

            if i >= nbin.wrapping_sub(1 as DWORD) {
                break;
            }

            if i != 0 && (flags & (CRYPT_STRING_NOCRLF as DWORD)) == 0 {
                if (i.wrapping_add(1 as DWORD))
                    .wrapping_rem(16 as DWORD)
                    == 0
                {
                    if (flags & (CRYPT_STRING_NOCR as DWORD)) != 0 {
                        *str = '\n' as WCHAR;
                        str = str.add(1);
                    } else {
                        *str = '\r' as WCHAR;
                        str = str.add(1);
                        *str = '\n' as WCHAR;
                        str = str.add(1);
                    }
                    break 'body;
                } else if (i.wrapping_add(1 as DWORD))
                    .wrapping_rem(8 as DWORD)
                    == 0
                {
                    *str = ' ' as WCHAR;
                    str = str.add(1);
                }
            }

            *str = ' ' as WCHAR;
            str = str.add(1);
        }
        i = i.wrapping_add(1);
    }

    if (flags & (CRYPT_STRING_NOCR as DWORD)) != 0 {
        *str = '\n' as WCHAR;
        str = str.add(1);
    } else if (flags & (CRYPT_STRING_NOCRLF as DWORD)) == 0 {
        *str = '\r' as WCHAR;
        str = str.add(1);
        *str = '\n' as WCHAR;
        str = str.add(1);
    }

    *str = 0 as WCHAR;
    *nstr = needed.wrapping_sub(1 as DWORD);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptBinaryToStringW(
    pbBinary: *const BYTE,
    cbBinary: DWORD,
    dwFlags: DWORD,
    pszString: LPWSTR,
    pcchString: *mut DWORD,
) -> BOOL {
    let mut encoder: BinaryToStringWFunc = None;

    TRACE!("(%p, %ld, %08lx, %p, %p)\n", pbBinary, cbBinary, dwFlags, pszString, pcchString);

    if pbBinary.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if pcchString.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    match dwFlags & 0x0fffffff {
        CRYPT_STRING_BINARY => {
            encoder = Some(
                EncodeBinaryToBinaryW
                    as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, *mut WCHAR, *mut DWORD) -> BOOL,
            ) as BinaryToStringWFunc;
        }
        CRYPT_STRING_BASE64 | CRYPT_STRING_BASE64HEADER | CRYPT_STRING_BASE64REQUESTHEADER | CRYPT_STRING_BASE64X509CRLHEADER => {
            encoder = Some(
                BinaryToBase64W
                    as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, LPWSTR, *mut DWORD) -> BOOL,
            ) as BinaryToStringWFunc;
        }
        CRYPT_STRING_HEXRAW => {
            encoder = Some(
                BinaryToHexRawW
                    as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, LPWSTR, *mut DWORD) -> BOOL,
            ) as BinaryToStringWFunc;
        }
        CRYPT_STRING_HEX => {
            encoder = Some(
                binary_to_hexW
                    as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, LPWSTR, *mut DWORD) -> BOOL,
            ) as BinaryToStringWFunc;
        }
        CRYPT_STRING_HEXASCII | CRYPT_STRING_HEXADDR | CRYPT_STRING_HEXASCIIADDR => {
            FIXME!("Unimplemented type %ld\n", dwFlags & 0x0fffffff);
            // fall through to default
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }

    return (encoder.unwrap_unchecked())(pbBinary, cbBinary, dwFlags, pszString, pcchString);
}

unsafe extern "C" fn decodeBase64Byte(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = (BASE64_DECODE_INVALID as i32);
    if c >= 'A' as i32 && c <= 'Z' as i32 {
        ret = c - 'A' as i32;
    } else if c >= 'a' as i32 && c <= 'z' as i32 {
        ret = c - 'a' as i32 + 26 as ::core::ffi::c_int;
    } else if c >= '0' as i32 && c <= '9' as i32 {
        ret = c - '0' as i32 + 52 as ::core::ffi::c_int;
    } else if c == '+' as i32 {
        ret = 62 as ::core::ffi::c_int;
    } else if c == '/' as i32 {
        ret = 63 as ::core::ffi::c_int;
    } else if c == '=' as i32 {
        ret = (BASE64_DECODE_PADDING as i32);
    } else if c == ' ' as i32 || c == '\t' as i32 || c == '\r' as i32 || c == '\n' as i32 {
        ret = (BASE64_DECODE_WHITESPACE as i32);
    }
    return ret;
}

unsafe extern "C" fn Base64ToBinary(
    pszString: *const ::core::ffi::c_void,
    wide: BOOL,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG {
    let mut cbIn: DWORD = 0;
    let mut cbValid: DWORD = 0;
    let mut cbOut: DWORD = 0;
    let mut hasPadding: DWORD = 0;
    let mut block: [BYTE; 4] = [0; 4];
    cbIn = 0;
    cbValid = 0;
    cbOut = 0;
    hasPadding = 0;
    'invalid: {
        'overflow: {
            loop {
                if !(cbIn < cchString) {
                    break;
                }
                let c: ::core::ffi::c_int = if wide != 0 {
                    *(pszString as *const WCHAR).offset(cbIn as isize) as ::core::ffi::c_int
                } else {
                    *(pszString as *const ::core::ffi::c_char).offset(cbIn as isize)
                        as ::core::ffi::c_int
                };
                let d: ::core::ffi::c_int = decodeBase64Byte(c);
                if d == (BASE64_DECODE_INVALID as i32) {
                    break 'invalid;
                }
                if !(d == (BASE64_DECODE_WHITESPACE as i32)) {
                    if hasPadding != 0 && d != (BASE64_DECODE_PADDING as i32) {
                        break 'invalid;
                    }
                    if d == (BASE64_DECODE_PADDING as i32) && (cbValid & 3) == 0 {
                        break;
                    }
                    cbValid = cbValid.wrapping_add(1);
                    if d == (BASE64_DECODE_PADDING as i32) {
                        hasPadding = 1;
                        if (cbValid & 3) == 0 {
                            break;
                        }
                    } else if (cbValid & 3) != 2 {
                        cbOut = cbOut.wrapping_add(1);
                    }
                }
                cbIn = cbIn.wrapping_add(1);
            }
            if (cbValid & 3) != 0 && hasPadding != 0 {
                break 'invalid;
            }
            if !pbBinary.is_null() && *pcbBinary != 0 && cbOut > *pcbBinary {
                break 'overflow;
            }
            if !pbBinary.is_null() {
                cbIn = 0;
                cbValid = 0;
                cbOut = 0;
                while cbIn < cchString {
                    let c_0: ::core::ffi::c_int = if wide != 0 {
                        *(pszString as *const WCHAR).offset(cbIn as isize) as ::core::ffi::c_int
                    } else {
                        *(pszString as *const ::core::ffi::c_char).offset(cbIn as isize)
                            as ::core::ffi::c_int
                    };
                    let d_0: ::core::ffi::c_int = decodeBase64Byte(c_0);
                    if !(d_0 == (BASE64_DECODE_WHITESPACE as i32)) {
                        if d_0 == (BASE64_DECODE_PADDING as i32) {
                            break;
                        }
                        block[(cbValid & 3) as usize] = d_0 as BYTE;
                        cbValid = cbValid.wrapping_add(1);
                        match cbValid & 3 {
                            1 => {
                                let fresh_a = cbOut;
                                cbOut = cbOut.wrapping_add(1);
                                *pbBinary.offset(fresh_a as isize) = (((block[0 as usize]
                                    as ::core::ffi::c_int)
                                    << 2)
                                    as BYTE);
                            }
                            2 => {
                                *pbBinary.offset(cbOut.wrapping_sub(1) as isize) =
                                    (((block[0 as usize] as ::core::ffi::c_int)
                                        << 2
                                        | (block[1 as usize] as ::core::ffi::c_int)
                                            >> 4)
                                        as BYTE);
                            }
                            3 => {
                                let fresh_b = cbOut;
                                cbOut = cbOut.wrapping_add(1);
                                *pbBinary.offset(fresh_b as isize) = (((block[1 as usize]
                                    as ::core::ffi::c_int)
                                    << 4
                                    | (block[2 as usize] as ::core::ffi::c_int)
                                        >> 2)
                                    as BYTE);
                            }
                            0 => {
                                let fresh_c = cbOut;
                                cbOut = cbOut.wrapping_add(1);
                                *pbBinary.offset(fresh_c as isize) = (((block[2 as usize]
                                    as ::core::ffi::c_int)
                                    << 6
                                    | (block[3 as usize] as ::core::ffi::c_int)
                                        >> 0)
                                    as BYTE);
                            }
                            _ => {}
                        }
                    }
                    cbIn = cbIn.wrapping_add(1);
                }
            }
            *pcbBinary = cbOut;
            if !pdwSkip.is_null() {
                *pdwSkip = 0;
            }
            if !pdwFlags.is_null() {
                *pdwFlags = CRYPT_STRING_BASE64;
            }
            return (ERROR_SUCCESS) as LONG;
        }
        return (ERROR_INSUFFICIENT_BUFFER) as LONG;
    }
    *pcbBinary = cbOut;
    return (ERROR_INVALID_DATA) as LONG;
}

unsafe extern "C" fn Base64ToBinaryA(pszString: LPCSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    return Base64ToBinary(pszString as *const core::ffi::c_void, (FALSE) as BOOL, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
}

unsafe extern "C" fn Base64WithHeaderAndTrailerToBinaryA(
    pszString: LPCSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
) -> LONG {
    let mut ret: LONG;
    let mut header: LPCSTR = (CERT_HEADER_START).as_ptr();
    let mut trailer: LPCSTR = (CERT_TRAILER_START).as_ptr();

    let mut headerBegins: LPCSTR;
    let mut dataBegins: LPCSTR;
    let mut trailerBegins: LPCSTR;
    let mut dataLength: size_t;

    if (strlen(header) + strlen(trailer)) > cchString as size_t {
        return ERROR_INVALID_DATA as LONG;
    }

    headerBegins = strstr(pszString, header) as LPCSTR;
    if headerBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }

    dataBegins = headerBegins.offset(strlen(header) as isize);
    dataBegins = strstr(dataBegins, (CERT_DELIMITER).as_ptr()) as LPCSTR;
    if dataBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }
    dataBegins = dataBegins.offset(strlen((CERT_DELIMITER).as_ptr()) as isize);
    if *dataBegins as i32 == '\r' as i32 {
        dataBegins = dataBegins.offset(1);
    }
    if *dataBegins as i32 == '\n' as i32 {
        dataBegins = dataBegins.offset(1);
    }

    trailerBegins = strstr(dataBegins, trailer) as LPCSTR;
    if trailerBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }
    if *trailerBegins.offset(-(1 as isize)) as i32 == '\n' as i32 {
        trailerBegins = trailerBegins.offset(-1);
    }
    if *trailerBegins.offset(-(1 as isize)) as i32 == '\r' as i32 {
        trailerBegins = trailerBegins.offset(-1);
    }

    if !pdwSkip.is_null() {
        *pdwSkip = headerBegins.offset_from(pszString) as DWORD;
    }

    dataLength = trailerBegins.offset_from(dataBegins) as size_t;

    ret = Base64ToBinaryA(
        dataBegins,
        dataLength as DWORD,
        pbBinary,
        pcbBinary,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    return ret;
}

unsafe extern "C" fn Base64HeaderToBinaryA(pszString: LPCSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG = Base64WithHeaderAndTrailerToBinaryA(pszString, cchString,
     pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64HEADER;
    }
    return ret;
}

unsafe extern "C" fn Base64RequestHeaderToBinaryA(pszString: LPCSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG = Base64WithHeaderAndTrailerToBinaryA(pszString, cchString, pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64REQUESTHEADER as DWORD;
    }
    return ret;
}

unsafe extern "C" fn Base64X509HeaderToBinaryA(
    mut pszString: LPCSTR,
    mut cchString: DWORD,
    mut pbBinary: *mut BYTE,
    mut pcbBinary: *mut DWORD,
    mut pdwSkip: *mut DWORD,
    mut pdwFlags: *mut DWORD,
) -> LONG {
    let mut ret: LONG = Base64WithHeaderAndTrailerToBinaryA(pszString, cchString, pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64X509CRLHEADER as DWORD;
    }
    return ret;
}

unsafe extern "C" fn Base64AnyToBinaryA(pszString: LPCSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG;

    ret = Base64HeaderToBinaryA(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    if ret == ERROR_INVALID_DATA as LONG {
        ret = Base64ToBinaryA(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    }
    return ret;
}

unsafe extern "C" fn DecodeBinaryToBinaryA(
    pszString: LPCSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG {
    let mut ret: LONG = ERROR_SUCCESS as LONG;
    if *pcbBinary < cchString {
        if pbBinary.is_null() {
            *pcbBinary = cchString;
        } else {
            ret = ERROR_INSUFFICIENT_BUFFER as LONG;
            *pcbBinary = cchString;
        }
    } else {
        if cchString != 0 {
            memcpy(
                pbBinary as *mut ::core::ffi::c_void,
                pszString as *const ::core::ffi::c_void,
                cchString as usize,
            );
        }
        *pcbBinary = cchString;
    }
    return ret;
}

unsafe extern "C" fn DecodeAnyA(
    pszString: LPCSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG {
    let mut ret: LONG;

    ret = Base64HeaderToBinaryA(pszString, cchString, pbBinary, pcbBinary,
     pdwSkip, pdwFlags);
    if ret == ERROR_INVALID_DATA as LONG {
        ret = Base64ToBinaryA(pszString, cchString, pbBinary, pcbBinary,
         pdwSkip, pdwFlags);
    }
    if ret == ERROR_INVALID_DATA as LONG {
        ret = DecodeBinaryToBinaryA(pszString, cchString, pbBinary, pcbBinary,
         pdwSkip, pdwFlags);
    }
    return ret;
}

unsafe extern "C" fn is_hex_string_special_char(mut c: WCHAR) -> BOOL {
    match c {
        0x2D | 0x2C | 0x20 | 0x09 | 0x0D | 0x0A => {
            return (TRUE) as BOOL;
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn wchar_from_str(wide: BOOL, str: *mut *const ::core::ffi::c_void, len: *mut DWORD) -> WCHAR {
    let mut c: WCHAR = 0;

    if *len == 0 {
        return 0;
    }

    *len = (*len).wrapping_sub(1);
    if wide != 0 {
        let p: *const WCHAR = *(str as *mut *const WCHAR);
        *(str as *mut *const WCHAR) = p.add(1);
        c = *p;
    } else {
        let p: *const ::core::ffi::c_char = *(str as *mut *const ::core::ffi::c_char);
        *(str as *mut *const ::core::ffi::c_char) = p.add(1);
        c = *p as WCHAR;
    }

    return if c != 0 { c } else { 0xffff };
}

unsafe extern "C" fn digit_from_char(mut c: WCHAR) -> BYTE {
    if (c as i32) >= ('0' as i32) && (c as i32) <= ('9' as i32) {
        return ((c as i32) - ('0' as i32)) as BYTE;
    }
    c = towlower(c as wint_t) as WCHAR;
    if (c as i32) >= ('a' as i32) && (c as i32) <= ('f' as i32) {
        return ((c as i32) - ('a' as i32) + 0xa) as BYTE;
    }
    return 0xff as BYTE;
}

unsafe extern "C" fn string_to_hex(mut str: *const ::core::ffi::c_void, wide: BOOL, mut len: DWORD, hex: *mut BYTE, hex_len: *mut DWORD, skipped: *mut DWORD, ret_flags: *mut DWORD) -> LONG {
    let mut byte_idx: u32 = 0 as u32;
    let mut d1: BYTE;
    let mut d2: BYTE;
    let mut c: WCHAR;

    if str.is_null() || hex_len.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if len == 0 {
        len = (if wide != 0 {
            wcslen(str as *const u16) as DWORD
        } else {
            strlen(str as *const i8) as DWORD
        }) as DWORD;
    }

    if wide != 0 && len == 0 {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if !skipped.is_null() {
        *skipped = 0 as DWORD;
    }
    if !ret_flags.is_null() {
        *ret_flags = 0 as DWORD;
    }

    loop {
        c = wchar_from_str(wide, core::ptr::addr_of_mut!(str), core::ptr::addr_of_mut!(len));
        if !(c as i32 != 0 && is_hex_string_special_char(c) != 0) {
            break;
        }
    }

    loop {
        d1 = digit_from_char(c);
        if !(d1 as i32 != 0xff as i32) {
            break;
        }

        d2 = digit_from_char(wchar_from_str(wide, core::ptr::addr_of_mut!(str), core::ptr::addr_of_mut!(len)));
        if d2 as i32 == 0xff as i32 {
            if hex.is_null() {
                *hex_len = 0 as DWORD;
            }
            return ERROR_INVALID_DATA as LONG;
        }

        if !hex.is_null() && (byte_idx as DWORD) < *hex_len {
            *hex.offset(byte_idx as isize) = (((d1 as i32) << 4) | d2 as i32) as BYTE;
        }

        byte_idx = byte_idx.wrapping_add(1);

        loop {
            c = wchar_from_str(wide, core::ptr::addr_of_mut!(str), core::ptr::addr_of_mut!(len));
            if !(c == '-' as WCHAR || c == ',' as WCHAR) {
                break;
            }
        }
    }

    while c != 0 {
        if is_hex_string_special_char(c) == 0 {
            if hex.is_null() {
                *hex_len = 0 as DWORD;
            }
            return ERROR_INVALID_DATA as LONG;
        }
        c = wchar_from_str(wide, core::ptr::addr_of_mut!(str), core::ptr::addr_of_mut!(len));
    }

    if !hex.is_null() && (byte_idx as DWORD) > *hex_len {
        return ERROR_MORE_DATA as LONG;
    }

    if !ret_flags.is_null() {
        *ret_flags = CRYPT_STRING_HEX as DWORD;
    }

    *hex_len = byte_idx as DWORD;

    return ERROR_SUCCESS as LONG;
}

unsafe extern "C" fn string_to_hexA(str: *const ::core::ffi::c_char, len: DWORD, hex: *mut BYTE, hex_len: *mut DWORD, skipped: *mut DWORD, ret_flags: *mut DWORD) -> LONG {
    return string_to_hex(str as *const ::core::ffi::c_void, (FALSE) as BOOL, len, hex, hex_len, skipped, ret_flags);
}

#[no_mangle]
pub unsafe extern "system" fn CryptStringToBinaryA(
    pszString: LPCSTR,
    mut cchString: DWORD,
    dwFlags: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> BOOL {
    let mut decoder: StringToBinaryAFunc = None;
    let mut ret: LONG = 0;

    TRACE!(
        "(%s, %ld, %08lx, %p, %p, %p, %p)\n",
        debugstr_an(
            pszString as *const ::core::ffi::c_char,
            (if cchString != 0 {
                cchString
            } else {
                -(1 as ::core::ffi::c_int) as DWORD
            }) as ::core::ffi::c_int,
        ),
        cchString,
        dwFlags,
        pbBinary,
        pcbBinary,
        pdwSkip,
        pdwFlags
    );

    if pszString.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    /* Only the bottom byte contains valid types */
    if (dwFlags & 0xfffffff0) != 0 {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }
    match dwFlags {
        CRYPT_STRING_BASE64_ANY => {
            decoder = Some(
                Base64AnyToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_BASE64 => {
            decoder = Some(
                Base64ToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_BASE64HEADER => {
            decoder = Some(
                Base64HeaderToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_BASE64REQUESTHEADER => {
            decoder = Some(
                Base64RequestHeaderToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_BASE64X509CRLHEADER => {
            decoder = Some(
                Base64X509HeaderToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_BINARY => {
            decoder = Some(
                DecodeBinaryToBinaryA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_ANY => {
            decoder = Some(
                DecodeAnyA
                    as unsafe extern "C" fn(
                        LPCSTR,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_HEX => {
            decoder = Some(
                string_to_hexA
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_char,
                        DWORD,
                        *mut BYTE,
                        *mut DWORD,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> LONG,
            );
        }
        CRYPT_STRING_HEXASCII | CRYPT_STRING_HEXADDR | CRYPT_STRING_HEXASCIIADDR => {
            FIXME!(
                "Unimplemented type %ld\n",
                dwFlags & 0x7fffffff
            );
            /* fall through */
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }
    if cchString == 0 {
        cchString = strlen(pszString as *const ::core::ffi::c_char) as DWORD;
    }
    ret = (decoder.unwrap_unchecked())(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    if ret != 0 {
        SetLastError(ret as DWORD);
    }
    return (ret == ERROR_SUCCESS as LONG) as BOOL;
}

unsafe extern "C" fn Base64ToBinaryW(pszString: LPCWSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    return Base64ToBinary(pszString as *const core::ffi::c_void, (TRUE) as BOOL, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
}

unsafe extern "C" fn Base64WithHeaderAndTrailerToBinaryW(
    mut pszString: LPCWSTR,
    mut cchString: DWORD,
    mut pbBinary: *mut BYTE,
    mut pcbBinary: *mut DWORD,
    mut pdwSkip: *mut DWORD,
) -> LONG {
    let mut ret: LONG;
    let mut header: LPCWSTR = (CERT_HEADER_START_W).as_ptr();
    let mut trailer: LPCWSTR = (CERT_TRAILER_START_W).as_ptr();

    let mut headerBegins: LPCWSTR;
    let mut dataBegins: LPCWSTR;
    let mut trailerBegins: LPCWSTR;
    let mut dataLength: size_t;

    if (lstrlenW(header) + lstrlenW(trailer)) as DWORD > cchString {
        return ERROR_INVALID_DATA as LONG;
    }

    headerBegins = wcsstr(pszString, header) as LPCWSTR;
    if headerBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }

    dataBegins = headerBegins.offset(lstrlenW(header) as isize);
    dataBegins = wcsstr(dataBegins, (CERT_DELIMITER_W).as_ptr()) as LPCWSTR;
    if dataBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }
    dataBegins = dataBegins.offset(lstrlenW((CERT_DELIMITER_W).as_ptr()) as isize);
    if *dataBegins == '\r' as WCHAR {
        dataBegins = dataBegins.offset(1);
    }
    if *dataBegins == '\n' as WCHAR {
        dataBegins = dataBegins.offset(1);
    }

    trailerBegins = wcsstr(dataBegins, trailer) as LPCWSTR;
    if trailerBegins.is_null() {
        return ERROR_INVALID_DATA as LONG;
    }
    if *trailerBegins.offset(-(1 as isize)) == '\n' as WCHAR {
        trailerBegins = trailerBegins.offset(-1);
    }
    if *trailerBegins.offset(-(1 as isize)) == '\r' as WCHAR {
        trailerBegins = trailerBegins.offset(-1);
    }

    if !pdwSkip.is_null() {
        *pdwSkip = headerBegins.offset_from(pszString) as DWORD;
    }

    dataLength = trailerBegins.offset_from(dataBegins) as size_t;

    ret = Base64ToBinaryW(
        dataBegins,
        dataLength as DWORD,
        pbBinary,
        pcbBinary,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
    );

    return ret;
}

unsafe extern "C" fn Base64HeaderToBinaryW(pszString: LPCWSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG = Base64WithHeaderAndTrailerToBinaryW(pszString, cchString, pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64HEADER as DWORD;
    }
    return ret;
}

unsafe extern "C" fn Base64RequestHeaderToBinaryW(pszString: LPCWSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG = Base64WithHeaderAndTrailerToBinaryW(pszString, cchString, pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64REQUESTHEADER as DWORD;
    }
    return ret;
}

unsafe extern "C" fn Base64X509HeaderToBinaryW(
    pszString: LPCWSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG {
    let ret: LONG = Base64WithHeaderAndTrailerToBinaryW(pszString, cchString,
        pbBinary, pcbBinary, pdwSkip);

    if ret == 0 && !pdwFlags.is_null() {
        *pdwFlags = CRYPT_STRING_BASE64X509CRLHEADER;
    }
    return ret;
}

unsafe extern "C" fn Base64AnyToBinaryW(pszString: LPCWSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG;

    ret = Base64HeaderToBinaryW(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    if ret == (ERROR_INVALID_DATA as LONG) {
        ret = Base64ToBinaryW(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    }
    return ret;
}

unsafe extern "C" fn DecodeBinaryToBinaryW(pszString: LPCWSTR, cchString: DWORD, pbBinary: *mut BYTE, pcbBinary: *mut DWORD, pdwSkip: *mut DWORD, pdwFlags: *mut DWORD) -> LONG {
    let mut ret: LONG = ERROR_SUCCESS as LONG;

    if *pcbBinary < cchString {
        if pbBinary.is_null() {
            *pcbBinary = cchString;
        } else {
            ret = ERROR_INSUFFICIENT_BUFFER as LONG;
            *pcbBinary = cchString;
        }
    } else {
        if cchString != 0 {
            memcpy(
                pbBinary as *mut ::core::ffi::c_void,
                pszString as *const ::core::ffi::c_void,
                (cchString as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()),
            );
        }
        *pcbBinary = (cchString as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()) as DWORD;
    }
    return ret;
}

unsafe extern "C" fn DecodeAnyW(
    pszString: LPCWSTR,
    cchString: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> LONG {
    let mut ret: LONG;

    ret = Base64HeaderToBinaryW(pszString, cchString, pbBinary, pcbBinary,
        pdwSkip, pdwFlags);
    if ret == (ERROR_INVALID_DATA) as LONG {
        ret = Base64ToBinaryW(pszString, cchString, pbBinary, pcbBinary,
            pdwSkip, pdwFlags);
    }
    if ret == (ERROR_INVALID_DATA) as LONG {
        ret = DecodeBinaryToBinaryW(pszString, cchString, pbBinary, pcbBinary,
            pdwSkip, pdwFlags);
    }
    return ret;
}

unsafe extern "C" fn string_to_hexW(str: *const WCHAR, len: DWORD, hex: *mut BYTE, hex_len: *mut DWORD, skipped: *mut DWORD, ret_flags: *mut DWORD) -> LONG {
    return string_to_hex(str as *const core::ffi::c_void, (TRUE) as BOOL, len, hex, hex_len, skipped, ret_flags);
}

#[no_mangle]
pub unsafe extern "system" fn CryptStringToBinaryW(
    pszString: LPCWSTR,
    mut cchString: DWORD,
    dwFlags: DWORD,
    pbBinary: *mut BYTE,
    pcbBinary: *mut DWORD,
    pdwSkip: *mut DWORD,
    pdwFlags: *mut DWORD,
) -> BOOL {
    let mut decoder: StringToBinaryWFunc = None;
    let mut ret: LONG = 0;

    TRACE!(
        "(%s, %ld, %08lx, %p, %p, %p, %p)\n",
        debugstr_wn(
            pszString as *const WCHAR,
            (if cchString != 0 { cchString } else { -1i32 as DWORD }) as i32,
        ),
        cchString,
        dwFlags,
        pbBinary,
        pcbBinary,
        pdwSkip,
        pdwFlags
    );

    if pszString.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    /* Only the bottom byte contains valid types */
    if dwFlags & 0xfffffff0 != 0 {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }
    match dwFlags {
        CRYPT_STRING_BASE64_ANY => {
            decoder = Some(Base64AnyToBinaryW);
        }
        CRYPT_STRING_BASE64 => {
            decoder = Some(Base64ToBinaryW);
        }
        CRYPT_STRING_BASE64HEADER => {
            decoder = Some(Base64HeaderToBinaryW);
        }
        CRYPT_STRING_BASE64REQUESTHEADER => {
            decoder = Some(Base64RequestHeaderToBinaryW);
        }
        CRYPT_STRING_BASE64X509CRLHEADER => {
            decoder = Some(Base64X509HeaderToBinaryW);
        }
        CRYPT_STRING_BINARY => {
            decoder = Some(DecodeBinaryToBinaryW);
        }
        CRYPT_STRING_ANY => {
            decoder = Some(DecodeAnyW);
        }
        CRYPT_STRING_HEX => {
            decoder = Some(string_to_hexW);
        }
        CRYPT_STRING_HEXASCII | CRYPT_STRING_HEXADDR | CRYPT_STRING_HEXASCIIADDR => {
            FIXME!("Unimplemented type %ld\n", dwFlags & 0x7fffffff);
            /* fall through */
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }
    if cchString == 0 {
        cchString = lstrlenW(pszString) as DWORD;
    }
    let decoder = decoder.unwrap_unchecked();
    ret = decoder(pszString, cchString, pbBinary, pcbBinary, pdwSkip, pdwFlags);
    if ret != 0 {
        SetLastError(ret as DWORD);
    }
    return (ret == (ERROR_SUCCESS as LONG)) as BOOL;
}
