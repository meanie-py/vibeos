use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const HAS_EXTENDED: u32 = 0x0001;
pub const HAS_NUMERIC: u32 = 0x0002;
pub const HAS_NON_NUMERIC: u32 = 0x0004;
pub const HAS_DOT: u32 = 0x0008;
pub const HAS_DOT_DOT: u32 = 0x0010;
pub const HAS_SPACE: u32 = 0x0020;
pub const HAS_INVALID: u32 = 0x0040;
pub const HAS_ASTERISK: u32 = 0x0080;
pub const HAS_UNDERSCORE: u32 = 0x0100;
pub const HAS_LONG_LABEL: u32 = 0x0200;

// ---- DnsNameCompare_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCompare_W(mut name1: PCWSTR, mut name2: PCWSTR) -> i32 {
    if name1.is_null() && name2.is_null() {
        return 1;
    }
    if name1.is_null() || name2.is_null() {
        return 0;
    }

    let len1 = lstrlenW(name1);
    let len2 = lstrlenW(name2);

    let mut p = name1.wrapping_offset(len1 as isize - 1);
    let mut q = name2.wrapping_offset(len2 as isize - 1);

    while (p as usize) >= (name1 as usize) && *p == b'.' as u16 {
        p = p.wrapping_offset(-1);
    }
    while (q as usize) >= (name2 as usize) && *q == b'.' as u16 {
        q = q.wrapping_offset(-1);
    }

    let diff1 = (p as isize - name1 as isize) / 2;
    let diff2 = (q as isize - name2 as isize) / 2;
    if diff1 != diff2 {
        return 0;
    }

    while (name1 as usize) <= (p as usize) {
        if towupper(*name1 as wint_t) != towupper(*name2 as wint_t) {
            return 0;
        }
        name1 = name1.offset(1);
        name2 = name2.offset(1);
    }

    1
}

// ---- DnsValidateName_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsValidateName_W(name: PCWSTR, format: DNS_NAME_FORMAT) -> u32 {
    if name.is_null() {
        return ERROR_INVALID_NAME;
    }

    static INVALID: [u16; 29] = [
        b'{' as u16, b'|' as u16, b'}' as u16, b'~' as u16, b'[' as u16, b'\\' as u16,
        b']' as u16, b'^' as u16, b'\'' as u16, b':' as u16, b';' as u16, b'<' as u16,
        b'=' as u16, b'>' as u16, b'?' as u16, b'@' as u16, b'!' as u16, b'"' as u16,
        b'#' as u16, b'$' as u16, b'%' as u16, b'^' as u16, b'`' as u16, b'(' as u16,
        b')' as u16, b'+' as u16, b'/' as u16, b',' as u16, 0u16
    ];

    let mut p = name;
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut state: u32 = 0;

    while *p != 0 {
        let ch = *p;

        if ch == b'.' as u16 {
            j = 0;
            state |= HAS_DOT;
            if *p.add(1) == b'.' as u16 {
                state |= HAS_DOT_DOT;
            }
        } else if ch < b'0' as u16 || ch > b'9' as u16 {
            state |= HAS_NON_NUMERIC;
        } else {
            state |= HAS_NUMERIC;
        }

        if j > 62 {
            state |= HAS_LONG_LABEL;
        }

        if !wcschr(INVALID.as_ptr(), ch).is_null() {
            state |= HAS_INVALID;
        } else if (ch as u32) > 127 { // unsigned* p > 127
            state |= HAS_EXTENDED;
        } else if ch == b' ' as u16 {
            state |= HAS_SPACE;
        } else if ch == b'_' as u16 {
            state |= HAS_UNDERSCORE;
        } else if ch == b'*' as u16 {
            state |= HAS_ASTERISK;
        }

        i += 1;
        j += 1;
        p = p.add(1);
    }

    if i == 0
        || i > 255
        || (state & HAS_LONG_LABEL) != 0
        || (state & HAS_DOT_DOT) != 0
        || (*name == b'.' as u16 && *name.add(1) != 0)
    {
        return ERROR_INVALID_NAME;
    }

    match format {
        DnsNameDomain | DnsNameHostnameFull => {
            if (state & HAS_NON_NUMERIC) == 0 && (state & HAS_NUMERIC) != 0 {
                return DNS_ERROR_NUMERIC_NAME;
            }
            if (state & HAS_EXTENDED) != 0 || (state & HAS_UNDERSCORE) != 0 {
                return DNS_ERROR_NON_RFC_NAME;
            }
            if (state & HAS_SPACE) != 0
                || (state & HAS_INVALID) != 0
                || (state & HAS_ASTERISK) != 0
            {
                return DNS_ERROR_INVALID_NAME_CHAR;
            }
        }
        DnsNameDomainLabel => {
            if (state & HAS_DOT) != 0 {
                return ERROR_INVALID_NAME;
            }
            if (state & HAS_EXTENDED) != 0 || (state & HAS_UNDERSCORE) != 0 {
                return DNS_ERROR_NON_RFC_NAME;
            }
            if (state & HAS_SPACE) != 0
                || (state & HAS_INVALID) != 0
                || (state & HAS_ASTERISK) != 0
            {
                return DNS_ERROR_INVALID_NAME_CHAR;
            }
        }
        DnsNameHostnameLabel => {
            if (state & HAS_DOT) != 0 {
                return ERROR_INVALID_NAME;
            }
            if (state & HAS_NON_NUMERIC) == 0 && (state & HAS_NUMERIC) != 0 {
                return DNS_ERROR_NUMERIC_NAME;
            }
            if (state & HAS_EXTENDED) != 0 || (state & HAS_UNDERSCORE) != 0 {
                return DNS_ERROR_NON_RFC_NAME;
            }
            if (state & HAS_SPACE) != 0
                || (state & HAS_INVALID) != 0
                || (state & HAS_ASTERISK) != 0
            {
                return DNS_ERROR_INVALID_NAME_CHAR;
            }
        }
        DnsNameWildcard => {
            if (state & HAS_NON_NUMERIC) == 0 && (state & HAS_NUMERIC) != 0 {
                return ERROR_INVALID_NAME;
            }
            if *name != b'*' as u16 {
                return ERROR_INVALID_NAME;
            }
            if *name.add(1) != 0 && *name.add(1) != b'.' as u16 {
                return DNS_ERROR_INVALID_NAME_CHAR;
            }
            if (state & HAS_EXTENDED) != 0
                || (state & HAS_SPACE) != 0
                || (state & HAS_INVALID) != 0
            {
                return ERROR_INVALID_NAME;
            }
        }
        DnsNameSrvRecord => {
            if (state & HAS_NON_NUMERIC) == 0 && (state & HAS_NUMERIC) != 0 {
                return ERROR_INVALID_NAME;
            }
            if *name != b'_' as u16 {
                return ERROR_INVALID_NAME;
            }
            if (state & HAS_UNDERSCORE) != 0 && *name.add(1) == 0 {
                return DNS_ERROR_NON_RFC_NAME;
            }
            if (state & HAS_EXTENDED) != 0
                || (state & HAS_SPACE) != 0
                || (state & HAS_INVALID) != 0
            {
                return ERROR_INVALID_NAME;
            }
        }
        _ => {
            // DPRINT1("unknown format: %d\n", format);
            // Fall through
        }
    }

    ERROR_SUCCESS
}

// ---- DnsNameCompare_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCompare_A(name1: LPCSTR, name2: LPCSTR) -> i32 {
    let name1W: PWSTR = dns_strdup_aw(name1);
    let name2W: PWSTR = dns_strdup_aw(name2);

    let ret = DnsNameCompare_W(name1W, name2W);

    HeapFree(GetProcessHeap(), 0, name1W as _);
    HeapFree(GetProcessHeap(), 0, name2W as _);

    ret
}

// ---- DnsValidateName_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsValidateName_A(name: PCSTR, format: DNS_NAME_FORMAT) -> u32 {
    let nameW: PWSTR = dns_strdup_aw(name);
    let ret: u32 = DnsValidateName_W(nameW, format);
    HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
    ret
}

// ---- DnsValidateName_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsValidateName_UTF8(name: PCSTR, format: DNS_NAME_FORMAT) -> DNS_STATUS {
    let nameW: PWSTR = dns_strdup_uw(name);
    let ret = DnsValidateName_W(nameW, format);
    HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
    ret
}
