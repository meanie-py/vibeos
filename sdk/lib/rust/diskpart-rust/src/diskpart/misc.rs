// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_399C8C2C7CAA: [u16; 50] = [
    0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078,
    0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078,
    0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025,
    0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032,
    0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn IsDecString(pszDecString: PWSTR) -> BOOL {
    let mut ptr: PWSTR;

    if pszDecString.is_null() || *pszDecString == UNICODE_NULL {
        return FALSE;
    }

    ptr = pszDecString;
    while *ptr != UNICODE_NULL {
        if iswdigit(*ptr) == 0 {
            return FALSE;
        }

        ptr = ptr.add(1);
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn IsHexString(pszHexString: PWSTR) -> BOOL {
    let mut ptr: PWSTR;

    if pszHexString.is_null() || *pszHexString == UNICODE_NULL {
        return FALSE;
    }

    ptr = pszHexString;
    while *ptr != UNICODE_NULL {
        if ((iswxdigit(*ptr)) == 0) {
            return FALSE;
        }

        ptr = ptr.add(1);
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn HasPrefix(
    pszString: PWSTR,
    pszPrefix: PWSTR,
    ppszSuffix: *mut PWSTR,
) -> BOOL {
    let mut nPrefixLength: i32;
    let mut ret: i32;

    nPrefixLength = (wcslen(pszPrefix)) as i32;
    ret = _wcsnicmp(pszString, pszPrefix, (nPrefixLength) as usize);
    if (ret == 0) && (!ppszSuffix.is_null()) {
        *ppszSuffix = pszString.add(nPrefixLength as usize);
    }

    return (ret == 0) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn RoundingDivide(
    Dividend: ULONGLONG,
    Divisor: ULONGLONG,
) -> ULONGLONG {
    return Dividend
        .wrapping_add(Divisor.wrapping_div(2))
        .wrapping_div(Divisor);
}
#[no_mangle]
pub unsafe extern "C" fn DuplicateQuotedString(pszInString: PWSTR) -> PWSTR {
    let mut pszOutString: PWSTR = core::ptr::null_mut();
    let mut pStart: PWSTR;
    let mut pEnd: PWSTR;
    let mut nLength: i32;

    if pszInString.is_null() || *pszInString.offset(0 as isize) as i32 == (UNICODE_NULL) as _ {
        return core::ptr::null_mut();
    }

    if *pszInString.offset(0 as isize) as u16 == 0x0022u16 {
        if *pszInString.offset(1 as isize) as i32 == (UNICODE_NULL) as _ {
            return core::ptr::null_mut();
        }

        pStart = pszInString.offset(1 as isize);
        pEnd = wcschr(pStart, '"' as u16);
        if pEnd.is_null() {
            nLength = (wcslen(pStart)) as i32;
        } else {
            nLength = (pEnd.offset_from(pStart)) as i32;
        }
    } else {
        pStart = pszInString;
        nLength = (wcslen(pStart)) as i32;
    }

    pszOutString = (RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        ((nLength + 1) * (core::mem::size_of::<u16>()) as i32) as SIZE_T,
    )).cast();
    if pszOutString.is_null() {
        return core::ptr::null_mut();
    }

    wcsncpy(pszOutString, pStart, (nLength) as usize);

    pszOutString
}
#[no_mangle]
pub unsafe extern "C" fn DuplicateString(pszInString: PWSTR) -> PWSTR {
    let mut pszOutString: PWSTR = core::ptr::null_mut();
    let mut nLength: i32;

    if pszInString.is_null() || *pszInString.offset(0 as isize) as i32 == (UNICODE_NULL) as _ {
        return core::ptr::null_mut();
    }

    nLength = (wcslen(pszInString)) as i32;
    pszOutString = (RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        ((nLength + 1) * (core::mem::size_of::<u16>()) as i32) as SIZE_T,
    )).cast();
    if pszOutString.is_null() {
        return core::ptr::null_mut();
    }

    wcscpy(pszOutString, pszInString);

    pszOutString
}
#[no_mangle]
pub unsafe extern "C" fn CreateGUID(pGuid: *mut GUID) {
    RtlGenRandom(pGuid as *mut core::ffi::c_void, core::mem::size_of::<GUID>() as u32);
    (*pGuid).Data3 = ((*pGuid).Data3 as i32 & 0x0fff) as u16;
    (*pGuid).Data3 = ((*pGuid).Data3 as i32 | (4 << 12)) as u16;
    (*pGuid).Data4[0 as usize] = ((*pGuid).Data4[0 as usize] as i32 & 0x3f) as u8;
    (*pGuid).Data4[0 as usize] = ((*pGuid).Data4[0 as usize] as i32 | 0x80) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSignature(pSignature: PDWORD) {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut Buffer: *mut u8;

    NtQuerySystemTime(&mut SystemTime);
    RtlTimeToTimeFields(&mut SystemTime, &mut TimeFields);

    Buffer = pSignature as *mut u8;
    *Buffer.add(0) = (TimeFields.Year & 0xFF) as u8
        + (TimeFields.Hour & 0xFF) as u8;
    *Buffer.add(1) = (TimeFields.Year >> 8) as u8
        + (TimeFields.Minute & 0xFF) as u8;
    *Buffer.add(2) = (TimeFields.Month & 0xFF) as u8
        + (TimeFields.Second & 0xFF) as u8;
    *Buffer.add(3) = (TimeFields.Day & 0xFF) as u8
        + (TimeFields.Milliseconds & 0xFF) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn PrintGUID(
    pszBuffer: PWSTR,
    pGuid: *mut GUID,
) {
    _swprintf(
        pszBuffer,
        RUSTCONV_WSTR_399C8C2C7CAA.as_ptr(),
        (*pGuid).Data1,
        (*pGuid).Data2 as core::ffi::c_uint,
        (*pGuid).Data3 as core::ffi::c_uint,
        (*pGuid).Data4[0] as core::ffi::c_uint,
        (*pGuid).Data4[1] as core::ffi::c_uint,
        (*pGuid).Data4[2] as core::ffi::c_uint,
        (*pGuid).Data4[3] as core::ffi::c_uint,
        (*pGuid).Data4[4] as core::ffi::c_uint,
        (*pGuid).Data4[5] as core::ffi::c_uint,
        (*pGuid).Data4[6] as core::ffi::c_uint,
        (*pGuid).Data4[7] as core::ffi::c_uint,
    );
}

unsafe extern "C" fn CharToByte(Char: u16) -> u8 {
    if iswdigit(Char) != 0 {
        return (Char as i32 - '0' as i32) as u8;
    } else if iswalpha(Char) != 0 {
        if iswupper(Char) != 0 {
            return (Char as i32 - ('A' as i32 - 10)) as u8;
        } else {
            return (Char as i32 - ('a' as i32 - 10)) as u8;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn StringToGUID(
    pGuid: *mut GUID,
    pszString: PWSTR,
) -> BOOL {
    let mut i: i32;

    if pszString.is_null() {
        return FALSE;
    }

    (*pGuid).Data1 = 0;
    i = 0;
    while i < 8 {
        if ((iswxdigit(*pszString.offset(i as isize))) == 0) {
            return FALSE;
        }
        (*pGuid).Data1 = ((*pGuid).Data1 << 4)
            | CharToByte(*pszString.offset(i as isize)) as u32;
        i += 1;
    }

    if *pszString.offset(8 as isize) != 0x002Du16 {
        return FALSE;
    }

    (*pGuid).Data2 = 0;
    i = 9;
    while i < 13 {
        if ((iswxdigit(*pszString.offset(i as isize))) == 0) {
            return FALSE;
        }
        (*pGuid).Data2 = (((*pGuid).Data2 as i32) << 4
            | CharToByte(*pszString.offset(i as isize)) as i32) as u16;
        i += 1;
    }

    if *pszString.offset(13 as isize) != 0x002Du16 {
        return FALSE;
    }

    (*pGuid).Data3 = 0;
    i = 14;
    while i < 18 {
        if ((iswxdigit(*pszString.offset(i as isize))) == 0) {
            return FALSE;
        }
        (*pGuid).Data3 = (((*pGuid).Data3 as i32) << 4
            | CharToByte(*pszString.offset(i as isize)) as i32) as u16;
        i += 1;
    }

    if *pszString.offset(18 as isize) != 0x002Du16 {
        return FALSE;
    }

    i = 19;
    while i < 36 {
        if i == 23 {
            if *pszString.offset(i as isize) != 0x002Du16 {
                return FALSE;
            }
            i += 1;
        }

        if ((iswxdigit(*pszString.offset(i as isize))) == 0)
            || ((iswxdigit(*pszString.offset((i + 1) as isize))) == 0)
        {
            return FALSE;
        }

        (*pGuid).Data4[((i - 19) / 2) as usize] =
            ((CharToByte(*pszString.offset(i as isize)) as i32)
                << 4
                | CharToByte(
                    *pszString.offset((i + 1) as isize),
                ) as i32) as u8;

        i += 2;
    }

    if *pszString.offset(36 as isize) == 0x0000u16 {
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn PrintBusType(
    pszBuffer: PWSTR,
    cchBufferMax: i32,
    BusType: STORAGE_BUS_TYPE,
) {
    if BusType as u32 <= BusTypeMax {
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            IDS_BUSTYPE_UNKNOWN + BusType as u32,
            pszBuffer,
            cchBufferMax,
        );
    } else {
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            IDS_BUSTYPE_OTHER,
            pszBuffer,
            cchBufferMax,
        );
    }
}
