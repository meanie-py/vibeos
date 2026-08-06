// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct GENERIC_NAME {
    pub pwName: PCWSTR,
    pub bCaseSensitive: BOOL,
    pub bRemove: BOOL,
}

#[repr(C)]
pub struct VENDOR_LONG_NAME {
    pub pwLongName: PCWSTR,
    pub pwShortName: PCWSTR,
}

#[repr(C)]
pub struct REDUNDANT_WORD {
    pub pwStr: PCWSTR,
    pub bReplaceFirstWord: BOOL,
}
unsafe extern "C" fn IsPunctuation(chr: wchar_t) -> u8 {
    if chr <= ' ' as wchar_t || chr == '.' as wchar_t || chr == ',' as wchar_t {
        1
    } else {
        0
    }
}
unsafe extern "C" fn IsDigitStrA(DmiString: PCHAR) -> BOOL {
    let mut c = DmiString;
    if c.is_null() {
        return FALSE;
    }
    while *c != 0 {
        if *c >= b'0' && *c <= b'9' {
            c = c.offset(1);
        } else {
            return FALSE;
        }
    }
    return TRUE;
}
unsafe extern "C" fn TrimPunctuation(pStr: PWSTR) {
    let mut Length: usize;
    let mut i: u32 = 0;

    if pStr.is_null() {
        return;
    }

    Length = wcslen(pStr);
    if Length == 0 {
        return;
    }

    while i < Length && IsPunctuation(*pStr.offset(i as isize)) != 0 {
        i += 1;
    }

    if i > 0 {
        Length -= i;
        memmove(
            pStr as *mut core::ffi::c_void,
            pStr.offset(i as isize) as *const core::ffi::c_void,
            (Length + 1) * core::mem::size_of::<u16>(),
        );
    }

    while Length != 0 && IsPunctuation(*pStr.offset((Length - 1) as isize)) != 0 {
        *pStr.offset((Length - 1) as isize) = 0u16;
        Length -= 1;
    }
}
unsafe extern "C" fn wcsistr(s: *const wchar_t, b: *const wchar_t) -> *mut wchar_t {
    let mut x: *mut wchar_t;
    let mut y: *mut wchar_t;
    let mut c: *mut wchar_t;
    x = s as *mut wchar_t;
    while *x != 0 {
        if towlower(*x) == towlower(*b) {
            y = x;
            c = b as *mut wchar_t;
            while *y != 0 && *c != 0 && towlower(*y) == towlower(*c) {
                c = c.add(1);
                y = y.add(1);
            }
            if *c == 0 {
                return x;
            }
        }
        x = x.add(1);
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn wcsistr_plus(s: *const wchar_t, b: *mut wchar_t) -> *mut wchar_t {
    let mut result = wcsistr(s, b);
    let len = wcslen(b);

    if result.is_null() && *b.offset((len - 1) as isize) == ' ' as wchar_t && !wcschr(s, ',' as wchar_t).is_null() {
        *b.offset((len - 1) as isize) = ',' as wchar_t;
        result = wcsistr(s, b);
        *b.offset((len - 1) as isize) = ' ' as wchar_t;
        if result.is_null() {
            *b.offset(0 as isize) = ',' as wchar_t;
            result = wcsistr(s, b);
            *b.offset(0 as isize) = ' ' as wchar_t;
        }
    }
    if result.is_null() && *b.offset((len - 1) as isize) == ' ' as wchar_t && !wcschr(s, '(' as wchar_t).is_null() {
        *b.offset((len - 1) as isize) = '(' as wchar_t;
        result = wcsistr(s, b);
        *b.offset((len - 1) as isize) = ' ' as wchar_t;
    }
    if result.is_null() && *b.offset((len - 1) as isize) == ' ' as wchar_t && !wcschr(s, '_' as wchar_t).is_null() {
        *b.offset(0 as isize) = '_' as wchar_t;
        result = wcsistr(s, b);
        *b.offset(0 as isize) = ' ' as wchar_t;
    }
    if result.is_null() && *b.offset(0 as isize) == ' ' as wchar_t && *b.offset((len - 1) as isize) == ' ' as wchar_t && !wcschr(s, ')' as wchar_t).is_null() {
        *b.offset(0 as isize) = ')' as wchar_t;
        result = wcsistr(s, b);
        if result.is_null() && !wcschr(s, '.' as wchar_t).is_null() {
            *b.offset((len - 1) as isize) = '.' as wchar_t;
            result = wcsistr(s, b);
            *b.offset((len - 1) as isize) = ' ' as wchar_t;
        }
        *b.offset(0 as isize) = ' ' as wchar_t;
    }
    result
}
unsafe extern "C" fn wcsrep(
    pwStr: PWSTR,
    pwFind: PCWSTR,
    pwReplace: PCWSTR,
    bReplaceFirstWord: i32,
) {
    let mut pwsStr: PWSTR;
    let mut pwsFind: PWSTR;
    let mut pwsReplace: PWSTR;
    let mut pwsBuf: PWSTR = core::ptr::null_mut();
    let lenStr: usize;
    let lenFind: usize;
    let lenReplace: usize;

    if pwStr.is_null() || pwFind.is_null() || pwReplace.is_null() ||
        wcslen(pwStr) == 0 ||
        wcslen(pwFind) == 0 ||
        wcslen(pwFind) < wcslen(pwReplace)
    {
        return;
    }
    lenStr = wcslen(pwStr) + 2 + 1;
    lenFind = wcslen(pwFind) + 2 + 1;
    lenReplace = wcslen(pwReplace) + 2 + 1;

    pwsStr = HeapAlloc(GetProcessHeap(), 0, lenStr * core::mem::size_of::<u16>()) as PWSTR;
    if pwsStr.is_null()
    {
        return;
    }
    StringCchCopyW(pwsStr, lenStr, b" \0".as_ptr() as *const u16);
    StringCchCatW(pwsStr, lenStr, pwStr);
    StringCchCatW(pwsStr, lenStr, b" \0".as_ptr() as *const u16);

    pwsFind = HeapAlloc(GetProcessHeap(), 0, lenFind * core::mem::size_of::<u16>()) as PWSTR;
    if pwsFind.is_null()
    {
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }
    StringCchCopyW(pwsFind, lenFind, b" \0".as_ptr() as *const u16);
    StringCchCatW(pwsFind, lenFind, pwFind);
    StringCchCatW(pwsFind, lenFind, b" \0".as_ptr() as *const u16);

    pwsBuf = wcsistr_plus(pwsStr, pwsFind);
    if pwsBuf.is_null()
    {
        HeapFree(GetProcessHeap(), 0, pwsFind as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }
    if bReplaceFirstWord == 0 && (pwsBuf as usize - pwsStr as usize) / core::mem::size_of::<u16>() < 2
    {
        HeapFree(GetProcessHeap(), 0, pwsFind as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }

    pwsReplace = HeapAlloc(GetProcessHeap(), 0, lenReplace * core::mem::size_of::<u16>()) as PWSTR;
    if pwsReplace.is_null()
    {
        HeapFree(GetProcessHeap(), 0, pwsFind as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }
    StringCchCopyW(pwsReplace, lenReplace, b" \0".as_ptr() as *const u16);
    StringCchCatW(pwsReplace, lenReplace, pwReplace);
    StringCchCatW(pwsReplace, lenReplace, b" \0".as_ptr() as *const u16);

    loop
    {
        memmove(pwsBuf as *mut core::ffi::c_void, pwsReplace as *const core::ffi::c_void, (lenReplace - 1) * core::mem::size_of::<u16>());
        memmove(
            pwsBuf.offset((lenReplace as isize) - (if wcslen(pwReplace) > 0 { 1 } else { 2 })) as *mut core::ffi::c_void,
            pwsBuf.offset((lenFind as isize) - 1) as *const core::ffi::c_void,
            (lenStr - lenFind - ((pwsBuf as usize - pwsStr as usize) / core::mem::size_of::<u16>()) + 1) * core::mem::size_of::<u16>()
        );
        pwsBuf = wcsistr_plus(pwsStr, pwsFind);
        if pwsBuf.is_null()
        {
            break;
        }
    }

    TrimDmiStringW(pwsStr);
    StringCchCopyW(pwStr, wcslen(pwStr), pwsStr);

    HeapFree(GetProcessHeap(), 0, pwsReplace as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, pwsFind as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
}
unsafe extern "C" fn IsGenericSystemName(ven: PCWSTR, dev: PCWSTR, bRemove: *mut u8) -> u8 {
    struct GENERIC_NAME {
        pwName: PCWSTR,
        bCaseSensitive: u8,
        bRemove: u8,
    }

    static Vendors: [GENERIC_NAME; 20] = [
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
    ];

    static Devices: [GENERIC_NAME; 49] = [
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: FALSE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: FALSE, bRemove: 0 },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: 0, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: FALSE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: FALSE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: TRUE, bRemove: TRUE },
        GENERIC_NAME { pwName: TRUE as PCWSTR, bCaseSensitive: FALSE, bRemove: TRUE },
    ];

    let mut bMatch: u8;
    let mut i: u32;

    if !bRemove.is_null() {
        *bRemove = FALSE;
    }

    i = TRUE;
    while i < _countof(Vendors) {
        if ven.is_null() {
            break;
        }
        if Vendors[i as usize].bCaseSensitive != 0 {
            bMatch = (wcscmp(ven, Vendors[i as usize].pwName) == 0) as u8;
        } else {
            bMatch = (_wcsicmp(ven, Vendors[i as usize].pwName) == 0) as u8;
        }
        if bMatch != 0 {
            if !bRemove.is_null() {
                *bRemove = Vendors[i as usize].bRemove;
            }
            return FALSE;
        }
        i += 1;
    }

    i = TRUE;
    while i < _countof(Devices) {
        if dev.is_null() {
            break;
        }
        if Devices[i as usize].bCaseSensitive != 0 {
            bMatch = (wcscmp(dev, Devices[i as usize].pwName) == 0) as u8;
        } else {
            bMatch = (_wcsicmp(dev, Devices[i as usize].pwName) == 0) as u8;
        }
        if bMatch != 0 {
            if !bRemove.is_null() {
                *bRemove = Devices[i as usize].bRemove;
            }
            return FALSE;
        }
        i += 1;
    }

    TRUE
}
unsafe extern "C" fn AppendSystemFamily(pBuf: PWSTR, cchBuf: usize, DmiStrings: *mut PCSTR, dev: PWSTR) {
    static KnownFamilies: [PCSTR; 4] = [
        b"Eee PC\0".as_ptr() as PCSTR,
        b"ThinkPad\0".as_ptr() as PCSTR,
        b"IdeaPad\0".as_ptr() as PCSTR,
        b"IdeaCentre\0".as_ptr() as PCSTR,
    ];
    static Aliases: [PCWSTR; 4] = [
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        0 as PCWSTR,
    ];
    let mut i: u32;
    let mut wideStr: [wchar_t; 128] = [0; 128];

    i = 0;
    while i < _countof(KnownFamilies) as u32 {
        StringCchPrintfW(wideStr.as_mut_ptr(), _countof(wideStr) as usize, 0 as PCWSTR, *KnownFamilies.get_unchecked(i as usize));

        if wcsistr(dev, wideStr.as_ptr()) == core::ptr::null_mut() &&
            (*Aliases.get_unchecked(i as usize) == core::ptr::null() || wcsistr(dev, *Aliases.get_unchecked(i as usize)) == core::ptr::null_mut()) &&
            *DmiStrings.offset(SYS_FAMILY as isize) != core::ptr::null() &&
            _stricmp(*DmiStrings.offset(SYS_FAMILY as isize), *KnownFamilies.get_unchecked(i as usize)) == 0
        {
            if wcslen(pBuf) > 0 && wcslen(dev) > 0 {
                StringCchCatW(pBuf, cchBuf, b" \0".as_ptr() as *const u16);
            }
            StringCchCatW(pBuf, cchBuf, wideStr.as_ptr());
        }
        i += 1;
    }
}
unsafe extern "C" fn TrimNonPrintable(DmiString: PCHAR) -> BOOL {
    let mut c = DmiString;
    if c.is_null() {
        return FALSE;
    }
    while *c != 0 {
        if *c >= 0x20 && *c <= 0x7e {
            c = c.offset(1);
        } else {
            *c = 0;
            return TRUE;
        }
    }
    return FALSE;
}
unsafe extern "C" fn TrimNonPrintableProd(DmiString: PCHAR) -> BOOL {
    let mut c: PCHAR;

    if DmiString.is_null() {
        return FALSE;
    }

    c = strstr(DmiString, b"(\xFF\xFF".as_ptr() as PCSTR);
    if c > DmiString {
        *c = 0;
    }

    return TrimNonPrintable(DmiString);
}
#[no_mangle]
pub unsafe extern "C" fn GetSystemName(pBuf: PWSTR, cchBuf: usize) -> u8 {
    static LongNames: [VENDOR_LONG_NAME; 25] = [
        VENDOR_LONG_NAME { pwLongName: b"A\0S\0U\0S\0T\0e\0K\0\0\0".as_ptr() as PCWSTR, pwShortName: b"A\0S\0U\0S\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"F\0i\0r\0s\0t\0 \0I\0n\0t\0e\0r\0n\0a\0t\0i\0o\0n\0a\0l\0 \0C\0o\0m\0p\0u\0t\0e\0r\0\0\0".as_ptr() as PCWSTR, pwShortName: b"F\0I\0C\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"H\0e\0w\0l\0e\0t\0t\0-\0P\0a\0c\0k\0a\0r\0d\0\0\0".as_ptr() as PCWSTR, pwShortName: b"H\0P\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"M\0I\0C\0R\0O\0-\0S\0T\0A\0R\0\0\0".as_ptr() as PCWSTR, pwShortName: b"M\0S\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"S\0G\0I\0.\0C\0O\0M\0\0\0".as_ptr() as PCWSTR, pwShortName: b"S\0G\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"S\0i\0l\0i\0c\0o\0n\0 \0G\0r\0a\0p\0h\0i\0c\0s\0 \0I\0n\0t\0e\0r\0n\0a\0t\0i\0o\0n\0a\0l\0\0\0".as_ptr() as PCWSTR, pwShortName: b"S\0G\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"I\0n\0t\0e\0l\0(\0R\0)\0 \0C\0l\0i\0e\0n\0t\0 \0S\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as PCWSTR, pwShortName: b"I\0n\0t\0e\0l\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"I\0n\0f\0o\0r\0m\0a\0t\0i\0o\0n\0C\0o\0m\0p\0u\0t\0e\0r\0S\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as PCWSTR, pwShortName: b"I\0C\0S\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"B\0e\0r\0n\0e\0c\0k\0e\0r\0 \0+\0 \0R\0a\0i\0n\0e\0r\0 \0 \0I\0n\0d\0u\0s\0t\0r\0i\0e\0-\0E\0l\0e\0k\0t\0r\0o\0n\0i\0k\0\0\0".as_ptr() as PCWSTR, pwShortName: b"B\0e\0r\0n\0e\0c\0k\0e\0r\0 \0&\0 \0R\0a\0i\0n\0e\0r\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0H\0U\0W\0I\0 \0I\0N\0N\0O\0V\0A\0T\0I\0O\0N\0 \0A\0N\0D\0 \0T\0E\0C\0H\0N\0O\0L\0O\0G\0Y\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0H\0U\0W\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0H\0U\0W\0I\0 \0I\0N\0N\0O\0V\0A\0T\0I\0O\0N\0 \0L\0I\0M\0I\0T\0E\0D\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0H\0U\0W\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0H\0U\0W\0I\0 \0 \0I\0N\0N\0O\0V\0A\0T\0I\0O\0N\0 \0 \0L\0I\0M\0I\0T\0E\0D\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0H\0U\0W\0I\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"h\0t\0t\0p\0:\0/\0/\0w\0w\0w\0.\0a\0b\0i\0t\0.\0c\0o\0m\0.\0t\0w\0/\0\0\0".as_ptr() as PCWSTR, pwShortName: b"A\0B\0I\0T\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"h\0t\0t\0p\0:\0\\\0\\\0w\0w\0w\0.\0a\0b\0i\0t\0.\0c\0o\0m\0.\0t\0w\0\0\0".as_ptr() as PCWSTR, pwShortName: b"A\0B\0I\0T\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"w\0w\0w\0.\0a\0b\0i\0t\0.\0c\0o\0m\0.\0t\0w\0\0\0".as_ptr() as PCWSTR, pwShortName: b"A\0B\0I\0T\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0A\0S\0P\0E\0R\0 \0B\0I\0L\0G\0I\0S\0A\0Y\0A\0R\0 \0S\0I\0S\0T\0E\0M\0L\0E\0R\0I\0 \0A\0.\0S\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0a\0s\0p\0e\0r\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0o\0l\0o\0r\0f\0u\0l\0 \0T\0e\0c\0h\0n\0o\0l\0o\0g\0y\0 \0A\0n\0d\0 \0D\0e\0v\0e\0l\0o\0p\0m\0e\0n\0t\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0o\0l\0o\0r\0f\0u\0l\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"C\0o\0l\0o\0r\0f\0u\0l\0 \0Y\0u\0 \0G\0o\0n\0g\0 \0T\0e\0c\0h\0n\0o\0l\0o\0g\0y\0 \0A\0n\0d\0 \0D\0e\0v\0e\0l\0o\0p\0m\0e\0n\0t\0\0\0".as_ptr() as PCWSTR, pwShortName: b"C\0o\0l\0o\0r\0f\0u\0l\0 \0Y\0u\0 \0G\0o\0n\0g\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"H\0a\0i\0e\0r\0C\0o\0m\0p\0u\0t\0e\0r\0\0\0".as_ptr() as PCWSTR, pwShortName: b"H\0a\0i\0e\0r\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"H\0a\0i\0e\0r\0 \0I\0n\0f\0o\0r\0m\0a\0t\0i\0o\0n\0 \0T\0e\0c\0h\0n\0o\0l\0o\0g\0y\0 \0(\0S\0h\0e\0n\0 \0Z\0h\0e\0n\0)\0\0\0".as_ptr() as PCWSTR, pwShortName: b"H\0a\0i\0e\0r\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"H\0A\0S\0E\0E\0C\0O\0M\0P\0U\0T\0E\0R\0S\0\0\0".as_ptr() as PCWSTR, pwShortName: b"H\0a\0s\0e\0e\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"H\0E\0L\0I\0O\0S\0 \0B\0U\0S\0I\0N\0E\0S\0S\0 \0C\0O\0M\0P\0U\0T\0E\0R\0\0\0".as_ptr() as PCWSTR, pwShortName: b"H\0E\0L\0I\0O\0S\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"S\0h\0a\0n\0g\0h\0a\0i\0 \0Z\0o\0n\0g\0z\0h\0i\0 \0I\0n\0f\0o\0T\0e\0c\0h\0\0\0".as_ptr() as PCWSTR, pwShortName: b"Z\0o\0n\0g\0z\0h\0i\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"T\0S\0I\0N\0G\0 \0H\0U\0A\0 \0T\0O\0N\0G\0F\0A\0N\0G\0 \0C\0O\0.\0,\0L\0T\0D\0\0\0".as_ptr() as PCWSTR, pwShortName: b"T\0S\0I\0N\0G\0H\0U\0A\0 \0T\0O\0N\0G\0F\0A\0N\0G\0\0\0".as_ptr() as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: b"Y\0e\0s\0t\0o\0n\0 \0D\0i\0g\0i\0t\0a\0l\0 \0T\0e\0c\0h\0n\0o\0l\0o\0g\0y\0 \0C\0o\0.\0,\0L\0T\0D\0\0\0".as_ptr() as PCWSTR, pwShortName: b"Y\0e\0s\0t\0o\0n\0\0\0".as_ptr() as PCWSTR },
    ];
    static RedundantWords: [REDUNDANT_WORD; 52] = [
        REDUNDANT_WORD { pwStr: b"C\0o\0r\0p\0o\0r\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0o\0m\0m\0u\0n\0i\0c\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0o\0m\0p\0u\0t\0e\0r\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0o\0m\0p\0u\0t\0e\0r\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"G\0r\0o\0u\0p\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0l\0o\0u\0d\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0e\0n\0t\0e\0r\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"M\0i\0c\0r\0o\0s\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0f\0o\0s\0y\0s\0t\0e\0m\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"D\0i\0g\0i\0t\0a\0l\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"E\0l\0e\0c\0t\0r\0o\0n\0i\0c\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"E\0l\0e\0c\0t\0r\0i\0c\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"E\0l\0e\0k\0t\0r\0o\0n\0i\0k\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0o\0f\0t\0w\0a\0r\0e\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"F\0o\0u\0n\0d\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0t\0e\0r\0n\0a\0t\0i\0o\0n\0a\0l\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0t\0e\0r\0a\0n\0t\0o\0n\0a\0l\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0N\0T\0E\0R\0A\0N\0T\0I\0O\0N\0A\0L\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0d\0u\0s\0t\0r\0i\0a\0l\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0d\0u\0s\0t\0r\0i\0e\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0f\0o\0r\0m\0a\0t\0i\0o\0n\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0f\0o\0r\0m\0a\0t\0i\0c\0a\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"P\0r\0o\0d\0u\0k\0t\0e\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"T\0e\0c\0h\0n\0o\0l\0o\0g\0y\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"T\0e\0c\0o\0h\0n\0o\0l\0o\0g\0y\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"T\0e\0c\0h\0n\0o\0l\0o\0g\0i\0e\0s\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"T\0e\0c\0n\0o\0l\0o\0g\0i\0a\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"L\0i\0m\0i\0t\0e\0d\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0t\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0n\0c\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0o\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0o\0r\0p\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"C\0r\0o\0p\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"L\0L\0C\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"L\0t\0d\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"L\0T\0D\0A\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"G\0m\0b\0H\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0.\0p\0.\0A\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"A\0.\0S\0.\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0.\0A\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0.\0A\0.\0S\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0/\0A\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0A\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"S\0A\0S\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"B\0V\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"A\0G\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"O\0O\0O\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: TRUE },
        REDUNDANT_WORD { pwStr: b"C\0J\0S\0C\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0N\0T\0'\0L\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"I\0N\0T\0L\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: b"p\0l\0c\0\0\0".as_ptr() as PCWSTR, bReplaceFirstWord: FALSE },
    ];
    let mut SMBiosBuf: *mut core::ffi::c_void;
    let mut DmiStrings: [PCSTR; ID_STRINGS_MAX] = [core::ptr::null(); ID_STRINGS_MAX];
    let mut ven: [u16; 512] = [0; 512];
    let mut dev: [u16; 512] = [0; 512];
    let mut tmpstr: [i8; 512] = [0; 512];
    let mut bTrimProduct: u8;
    let mut bTrimFamily: u8;
    let mut bGenericName: u8;
    let mut bRemove: u8;
    let mut i: u32;
    let mut j: PWSTR;

    SMBiosBuf = LoadSMBiosData(DmiStrings.as_mut_ptr());
    if SMBiosBuf.is_null() {
        return FALSE;
    }

    TrimNonPrintable(DmiStrings[SYS_VENDOR]);
    bTrimProduct = TrimNonPrintableProd(DmiStrings[SYS_PRODUCT]);
    TrimNonPrintable(DmiStrings[SYS_VERSION]);
    bTrimFamily = TrimNonPrintable(DmiStrings[SYS_FAMILY]);
    TrimNonPrintable(DmiStrings[BOARD_VENDOR]);
    TrimNonPrintable(DmiStrings[BOARD_NAME]);
    TrimNonPrintable(DmiStrings[BOARD_VERSION]);

    if bTrimProduct != 0 {
        if !DmiStrings[SYS_FAMILY].is_null() && bTrimFamily == 0 {
            DmiStrings[SYS_PRODUCT] = DmiStrings[SYS_FAMILY];
            bTrimProduct = FALSE;
        }
    }

    GetSMBiosStringW(DmiStrings[SYS_VENDOR], ven.as_mut_ptr(), _countof(ven.len()), TRUE);
    GetSMBiosStringW(DmiStrings[SYS_PRODUCT], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
    bGenericName = IsGenericSystemName(ven.as_ptr(), dev.as_ptr(), core::ptr::null_mut()) | bTrimProduct;

    if wcslen(dev.as_ptr()) == 0 ||
        wcscmp(dev.as_ptr(), ven.as_ptr()) == 0 ||
        bGenericName != 0
    {
        let mut bGenericVen: u8 = FALSE;
        let mut bRemoveVen: u8 = FALSE;
        let mut bGenericDev: u8 = if wcslen(dev.as_ptr()) == 0 || wcscmp(dev.as_ptr(), ven.as_ptr()) == 0 || bTrimProduct != 0 { 1 } else { 0 };

        if bGenericName != 0 && IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 {
            if bRemove != 0 {
                *ven.as_mut_ptr() = 0;
            }
            bGenericVen = TRUE;
            bRemoveVen = bRemove;
        }
        if bGenericName != 0 && IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 {
            if bRemove != 0 {
                *dev.as_mut_ptr() = 0;
            }
            bGenericDev = TRUE;
        }

        if !DmiStrings[BOARD_VENDOR].is_null() || bGenericName == 0 {
            if (!DmiStrings[BOARD_VENDOR].is_null() &&
                strlen(DmiStrings[BOARD_VENDOR]) >= 2 &&
                strstr(DmiStrings[BOARD_VENDOR], b"  \0".as_ptr() as PCSTR) != DmiStrings[BOARD_VENDOR]) ||
                bGenericVen != 0
            {
                GetSMBiosStringW(DmiStrings[BOARD_VENDOR], ven.as_mut_ptr(), _countof(ven.len()), TRUE);
            }
            GetSMBiosStringW(DmiStrings[BOARD_NAME], dev.as_mut_ptr(), _countof(dev.len()), TRUE);

            if IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 && bRemove != 0 {
                *ven.as_mut_ptr() = 0;

                if bGenericVen != 0 && bRemoveVen == 0 {
                    GetSMBiosStringW(DmiStrings[SYS_VENDOR], ven.as_mut_ptr(), _countof(ven.len()), TRUE);
                }
            }
            if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                *dev.as_mut_ptr() = 0;

                if bGenericDev == 0 {
                    GetSMBiosStringW(DmiStrings[SYS_PRODUCT], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
                }
            }
            if wcslen(dev.as_ptr()) == 0 &&
                !DmiStrings[SYS_VERSION].is_null()
            {
                GetSMBiosStringW(DmiStrings[SYS_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);

                if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                    *dev.as_mut_ptr() = 0;
                }
            }
            if wcslen(dev.as_ptr()) == 0 &&
                !DmiStrings[BOARD_VERSION].is_null()
            {
                GetSMBiosStringW(DmiStrings[BOARD_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);

                if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                    *dev.as_mut_ptr() = 0;
                }
            }
        } else if !DmiStrings[BOARD_NAME].is_null() {
            GetSMBiosStringW(DmiStrings[BOARD_NAME], dev.as_mut_ptr(), _countof(dev.len()), TRUE);

            if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                *dev.as_mut_ptr() = 0;
            }
        }

        if wcslen(ven.as_ptr()) == 0 && wcslen(dev.as_ptr()) == 0 {
            GetSMBiosStringW(DmiStrings[BIOS_VENDOR], ven.as_mut_ptr(), _countof(ven.len()), TRUE);
        }
    } else {
        if wcslen(ven.as_ptr()) < 2 {
            GetSMBiosStringW(DmiStrings[BOARD_VENDOR], ven.as_mut_ptr(), _countof(ven.len()), TRUE);

            if IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 && bRemove != 0 {
                *ven.as_mut_ptr() = 0;
            }
        }
    }

    j = wcsstr(ven.as_ptr(), b" \0\0\0".as_ptr() as PCWSTR);
    if !j.is_null() && (j as usize - ven.as_ptr() as usize) / 2 > 2 {
        i = ((j as usize - ven.as_ptr() as usize) / 2) as u32;
        if wcsncmp(ven.as_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize), ven.as_ptr(), i as usize) == 0 {
            *ven.as_mut_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize) = 0;
        }
    }

    for i in 0..25 {
        if wcsstr(dev.as_ptr(), LongNames[i as usize].pwLongName) == dev.as_ptr() {
            StringCchCopyW(pBuf, cchBuf, ven.as_ptr());
            StringCchCopyW(ven.as_mut_ptr(), _countof(ven.len()), dev.as_ptr());
            StringCchCopyW(dev.as_mut_ptr(), _countof(dev.len()), pBuf);
        }
        wcsrep(ven.as_mut_ptr(), LongNames[i as usize].pwLongName, LongNames[i as usize].pwShortName, TRUE);
    }

    for i in 0..52 {
        wcsrep(ven.as_mut_ptr(), RedundantWords[i as usize].pwStr, b"\0\0".as_ptr() as PCWSTR, RedundantWords[i as usize].bReplaceFirstWord as i32);
    }
    for i in 0..52 {
        StringCchCopyW(pBuf, cchBuf, RedundantWords[i as usize].pwStr);
        StringCchCatW(pBuf, cchBuf, b".\0\0\0".as_ptr() as PCWSTR);
        wcsrep(ven.as_mut_ptr(), pBuf, b"\0\0".as_ptr() as PCWSTR, RedundantWords[i as usize].bReplaceFirstWord as i32);
    }

    if _wcsicmp(ven.as_ptr(), b"L\0E\0N\0O\0V\0O\0\0\0".as_ptr() as PCWSTR) == 0 {
        StringCchCopyW(ven.as_mut_ptr(), _countof(ven.len()), b"L\0e\0n\0o\0v\0o\0\0\0".as_ptr() as PCWSTR);

        if !DmiStrings[SYS_VERSION].is_null() {
            if strncmp(DmiStrings[SYS_VERSION], b"ThinkPad   \0".as_ptr() as PCSTR, 11) == 0 {
                *DmiStrings[SYS_VERSION].offset(8 as isize) = 0;
            }
            if wcslen(dev.as_ptr()) > 0 &&
                (strcmp(DmiStrings[SYS_VERSION], b"IdeaCentre\0".as_ptr() as PCSTR) == 0 ||
                 strcmp(DmiStrings[SYS_VERSION], b"ThinkPad\0".as_ptr() as PCSTR) == 0)
            {
                DmiStrings[SYS_FAMILY] = DmiStrings[SYS_VERSION];
                DmiStrings[SYS_VERSION] = core::ptr::null();
            } else {
                StringCchCopyA(tmpstr.as_mut_ptr(), _countof(tmpstr.len()), DmiStrings[SYS_VERSION]);
                _strupr(tmpstr.as_mut_ptr());
            }
        }

        if !DmiStrings[SYS_VERSION].is_null() &&
            !strcmp(tmpstr.as_mut_ptr(), b" \0".as_ptr() as PCSTR).is_null() &&
            !strcmp(tmpstr.as_mut_ptr(), b"LENOVO\0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVO   \0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVO PRODUCT\0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVOPRODUCT\0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"INVALID\0".as_ptr() as PCSTR).is_null() &&
            strncmp(tmpstr.as_mut_ptr(), b"   \0".as_ptr() as PCSTR, 3) == 0 &&
            (strlen(tmpstr.as_mut_ptr()) >= 3 || IsDigitStrA(tmpstr.as_mut_ptr()) == 0) &&
            strstr(DmiStrings[SYS_VERSION], b"Rev \0".as_ptr() as PCSTR).is_null() &&
            strstr(DmiStrings[SYS_VERSION], b"1.\0".as_ptr() as PCSTR).is_null() &&
            wcsistr(dev.as_ptr(), b"S\0y\0s\0t\0e\0m\0 \0\0\0".as_ptr() as PCWSTR).is_null() &&
            wcsistr(dev.as_ptr(), b"I\0d\0e\0a\0P\0a\0d\0 \0\0\0".as_ptr() as PCWSTR).is_null() &&
            wcsistr(dev.as_ptr(), b"T\0h\0i\0n\0k\0S\0e\0r\0v\0e\0r\0 \0\0\0".as_ptr() as PCWSTR).is_null()
        {
            GetSMBiosStringW(DmiStrings[SYS_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
        }

        if wcsstr(dev.as_ptr(), b"L\0e\0n\0o\0v\0o\0-\0\0\0".as_ptr() as PCWSTR) == dev.as_ptr() {
            *dev.as_mut_ptr().offset(6 as isize) = b' ' as u16;
        }

        if wcscmp(dev.as_ptr(), b"L\0e\0n\0o\0v\0o\0\0\0".as_ptr() as PCWSTR) == 0 {
            GetSMBiosStringW(DmiStrings[BOARD_NAME], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
        }
    }
    if wcscmp(ven.as_ptr(), b"I\0B\0M\0\0\0".as_ptr() as PCWSTR) == 0 &&
        !DmiStrings[SYS_VERSION].is_null() &&
        (!strstr(DmiStrings[SYS_VERSION], b"ThinkPad \0".as_ptr() as PCSTR).is_null() ||
         !strstr(DmiStrings[SYS_VERSION], b"ThinkCentre \0".as_ptr() as PCSTR).is_null())
    {
        GetSMBiosStringW(DmiStrings[SYS_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
    }

    if wcscmp(ven.as_ptr(), b"D\0E\0X\0P\0\0\0".as_ptr() as PCWSTR) == 0 {
        if !DmiStrings[SYS_PRODUCT].is_null() &&
            !DmiStrings[SYS_VERSION].is_null() &&
            (_stricmp(DmiStrings[SYS_PRODUCT], b"Tablet PC\0".as_ptr() as PCSTR) == 0 ||
             _stricmp(DmiStrings[SYS_PRODUCT], b"Notebook\0".as_ptr() as PCSTR) == 0 ||
             _stricmp(DmiStrings[SYS_PRODUCT], b"Decktop\0".as_ptr() as PCSTR) == 0)
        {
            GetSMBiosStringW(DmiStrings[SYS_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
        }
    }

    if wcscmp(ven.as_ptr(), b"R\0a\0z\0e\0r\0\0\0".as_ptr() as PCWSTR) == 0 && wcscmp(dev.as_ptr(), b"B\0l\0a\0d\0e\0\0\0".as_ptr() as PCWSTR) == 0 {
        if !DmiStrings[SYS_VERSION].is_null() {
            StringCchCopyW(ven.as_mut_ptr(), _countof(ven.len()), b"R\0a\0z\0e\0r\0 \0B\0l\0a\0d\0e\0\0\0".as_ptr() as PCWSTR);
            GetSMBiosStringW(DmiStrings[SYS_VERSION], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
        }
    }

    if wcscmp(ven.as_ptr(), b"M\0S\0I\0\0\0".as_ptr() as PCWSTR) == 0 &&
        !wcsstr(dev.as_ptr(), b"M\0S\0-\0\0\0".as_ptr() as PCWSTR).is_null() &&
        !DmiStrings[BOARD_NAME].is_null() &&
        !strstr(DmiStrings[BOARD_NAME], b"(MS-\0".as_ptr() as PCSTR).is_null()
    {
        GetSMBiosStringW(DmiStrings[BOARD_NAME], dev.as_mut_ptr(), _countof(dev.len()), TRUE);
    }
    if wcslen(ven.as_ptr()) == 0 &&
        wcsstr(dev.as_ptr(), b"M\0S\0-\0\0\0".as_ptr() as PCWSTR) == dev.as_ptr()
    {
        StringCchCopyW(ven.as_mut_ptr(), _countof(ven.len()), b"M\0S\0I\0\0\0".as_ptr() as PCWSTR);
    }

    TrimPunctuation(ven.as_mut_ptr());
    TrimPunctuation(dev.as_mut_ptr());

    if wcsistr(dev.as_ptr(), ven.as_ptr()) == dev.as_ptr() ||
        (wcscmp(ven.as_ptr(), b"A\0S\0U\0S\0\0\0".as_ptr() as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), b"A\0S\0U\0S\0\0\0".as_ptr() as PCWSTR).is_null()) ||
        (wcscmp(ven.as_ptr(), b"H\0P\0\0\0".as_ptr() as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), b" \0b\0y\0 \0H\0P\0\0\0".as_ptr() as PCWSTR).is_null()) ||
        (wcscmp(ven.as_ptr(), b"I\0N\0T\0E\0L\0\0\0".as_ptr() as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), b" \0I\0N\0T\0E\0L\0\0\0".as_ptr() as PCWSTR).is_null())
    {
        StringCchCopyW(pBuf, cchBuf, dev.as_ptr());
    } else {
        if wcslen(ven.as_ptr()) > 0 && wcslen(dev.as_ptr()) > 0 {
            j = wcschr(dev.as_ptr(), b' ' as u16 as i32);
            if !j.is_null() {
                i = ((j as usize - dev.as_ptr() as usize) / 2) as u32;
                if wcslen(ven.as_ptr()) > i && _wcsnicmp(ven.as_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize), dev.as_ptr(), i as usize) == 0 {
                    *ven.as_mut_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize) = 0;
                    TrimPunctuation(ven.as_mut_ptr());
                }
            }
        }
        StringCchCopyW(pBuf, cchBuf, ven.as_ptr());
        AppendSystemFamily(pBuf, cchBuf, DmiStrings.as_mut_ptr(), dev.as_mut_ptr());
        if wcslen(pBuf) > 0 && wcslen(dev.as_ptr()) > 0 {
            StringCchCatW(pBuf, cchBuf, b" \0\0\0".as_ptr() as PCWSTR);
        }
        StringCchCatW(pBuf, cchBuf, dev.as_ptr());
    }

    FreeSMBiosData(SMBiosBuf);

    if wcslen(pBuf) > 0 { 1 } else { 0 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/smbios.c
//  * PURPOSE:     Retrieve device or motherboard name identifier from DMI/SMBIOS
//  * COPYRIGHT:   Copyright 2018-2020 Stanislav Motylkov <x86corez@gmail.com>
//  *
//  */
//
// #include "precomp.h"
//
// #include <udmihelp.h>
//
// typedef struct GENERIC_NAME
// {
//     PCWSTR pwName;
//     BOOL bCaseSensitive;
//     BOOL bRemove;
// } GENERIC_NAME;
//
// typedef struct VENDOR_LONG_NAME
// {
//     PCWSTR pwLongName;
//     PCWSTR pwShortName;
// } VENDOR_LONG_NAME;
//
// typedef struct REDUNDANT_WORD
// {
//     PCWSTR pwStr;
//     BOOL bReplaceFirstWord;
// } REDUNDANT_WORD;
//
// static
// BOOL
// IsPunctuation(
//      WCHAR chr)
// {
//     return (chr <= L' ' || chr == L'.' || chr == L',');
// }
//
// static
// BOOL IsDigitStrA(PCHAR DmiString)
// {
//     PCHAR c = DmiString;
//     if (!c)
//     {
//         return FALSE;
//     }
//     while (*c)
//     {
//         if (*c >= '0' && *c <= '9')
//         {
//             c++;
//         }
//         else
//         {
//             return FALSE;
//         }
//     }
//     return TRUE;
// }
//
// /*
//  * Trim redundant characters
//  */
// static
// VOID
// TrimPunctuation(
//      PWSTR pStr)
// {
//     SIZE_T Length;
//     UINT i = 0;
//
//     if (!pStr)
//         return;
//
//     Length = wcslen(pStr);
//     if (Length == 0)
//         return;
//
//     /* Trim leading characters */
//     while (i < Length && IsPunctuation(pStr[i]))
//     {
//         i++;
//     }
//
//     if (i > 0)
//     {
//         Length -= i;
//         memmove(pStr, pStr + i, (Length + 1) * sizeof(WCHAR));
//     }
//
//     /* Trim trailing characters */
//     while (Length && IsPunctuation(pStr[Length-1]))
//     {
//         pStr[Length-1] = L'\0';
//         --Length;
//     }
// }
//
// /*
//  * Case insensitive variant of wcsstr
//  */
// static
// wchar_t * wcsistr(const wchar_t *s, const wchar_t *b)
// {
//     wchar_t *x;
//     wchar_t *y;
//     wchar_t *c;
//     x = (wchar_t *)s;
//     while (*x)
//     {
//         if (towlower(*x) == towlower(*b))
//         {
//             y = x;
//             c = (wchar_t *)b;
//             while (*y && *c && towlower(*y) == towlower(*c))
//             {
//                 c++;
//                 y++;
//             }
//             if (!*c)
//                 return x;
//         }
//         x++;
//     }
//     return NULL;
// }
//
// static
// wchar_t * wcsistr_plus(const wchar_t *s, wchar_t *b)
// {
//     wchar_t * result = wcsistr(s, b);
//     UINT len = wcslen(b);
//     // workarounds
//     if (!result && b[len - 1] == L' ' && wcschr(s, L',') != NULL)
//     {
//         b[len - 1] = L',';
//         result = wcsistr(s, b);
//         b[len - 1] = L' ';
//         if (!result)
//         {
//             b[0] = L',';
//             result = wcsistr(s, b);
//             b[0] = L' ';
//         }
//     }
//     if (!result && b[len - 1] == L' ' && wcschr(s, L'(') != NULL)
//     {
//         b[len - 1] = L'(';
//         result = wcsistr(s, b);
//         b[len - 1] = L' ';
//     }
//     if (!result && b[len - 1] == L' ' && wcschr(s, L'_') != NULL)
//     {
//         b[0] = L'_';
//         result = wcsistr(s, b);
//         b[0] = L' ';
//     }
//     if (!result && b[0] == L' ' && b[len - 1] == L' ' && wcschr(s, L')') != NULL)
//     {
//         b[0] = L')';
//         result = wcsistr(s, b);
//         if (!result && wcschr(s, L'.'))
//         {
//             b[len - 1] = L'.';
//             result = wcsistr(s, b);
//             b[len - 1] = L' ';
//         }
//         b[0] = L' ';
//     }
//     return result;
// }
//
// /*
//  * Replaces full word with another shorter word
//  */
// static
// VOID wcsrep(
//      PWSTR pwStr,
//      PCWSTR pwFind,
//      PCWSTR pwReplace,
//      BOOL bReplaceFirstWord)
// {
//     PWSTR pwsStr, pwsFind, pwsReplace, pwsBuf = NULL;
//     SIZE_T lenStr;
//     SIZE_T lenFind;
//     SIZE_T lenReplace;
//
//     if (!pwStr || !pwFind || !pwReplace ||
//         wcslen(pwStr) == 0 ||
//         wcslen(pwFind) == 0 ||
//         wcslen(pwFind) < wcslen(pwReplace))
//     {
//         return;
//     }
//     lenStr = wcslen(pwStr) + 2 + 1;
//     lenFind = wcslen(pwFind) + 2 + 1;
//     lenReplace = wcslen(pwReplace) + 2 + 1;
//
//     pwsStr = HeapAlloc(GetProcessHeap(), 0, lenStr * sizeof(WCHAR));
//     if (!pwsStr)
//     {
//         return;
//     }
//     StringCchCopyW(pwsStr, lenStr, L" ");
//     StringCchCatW(pwsStr, lenStr, pwStr);
//     StringCchCatW(pwsStr, lenStr, L" ");
//
//     pwsFind = HeapAlloc(GetProcessHeap(), 0, lenFind * sizeof(WCHAR));
//     if (!pwsFind)
//     {
//         goto freeStr;
//     }
//     StringCchCopyW(pwsFind, lenFind, L" ");
//     StringCchCatW(pwsFind, lenFind, pwFind);
//     StringCchCatW(pwsFind, lenFind, L" ");
//
//     if (!(pwsBuf = wcsistr_plus(pwsStr, pwsFind)))
//     {
//         goto freeFind;
//     }
//     if (!bReplaceFirstWord && pwsBuf - pwsStr < 2)
//     {
//         goto freeFind;
//     }
//
//     pwsReplace = HeapAlloc(GetProcessHeap(), 0, lenReplace * sizeof(WCHAR));
//     if (!pwsReplace)
//     {
//         goto freeFind;
//     }
//     StringCchCopyW(pwsReplace, lenReplace, L" ");
//     StringCchCatW(pwsReplace, lenReplace, pwReplace);
//     StringCchCatW(pwsReplace, lenReplace, L" ");
//
//     do
//     {
//         // replace substring
//         memmove(pwsBuf, pwsReplace, (lenReplace - 1) * sizeof(WCHAR));
//         // shift characters
//         memmove(pwsBuf + lenReplace - (wcslen(pwReplace) > 0 ? 1 : 2), pwsBuf + lenFind - 1, (lenStr - lenFind - (pwsBuf - pwsStr) + 1) * sizeof(WCHAR));
//     }
//     while ((pwsBuf = wcsistr_plus(pwsStr, pwsFind)) != NULL);
//
//     TrimDmiStringW(pwsStr);
//     StringCchCopyW(pwStr, wcslen(pwStr), pwsStr);
//
//     HeapFree(GetProcessHeap(), 0, pwsReplace);
// freeFind:
//     HeapFree(GetProcessHeap(), 0, pwsFind);
// freeStr:
//     HeapFree(GetProcessHeap(), 0, pwsStr);
// }
//
// static
// BOOL IsGenericSystemName(PCWSTR ven, PCWSTR dev, BOOL * bRemove)
// {
//     static const GENERIC_NAME Vendors[] =
//     {
//         // some ASUS boards
//         { L"To Be Filled By O.E.M.", FALSE, TRUE },
//         { L"To Be Filled By O.E.M", FALSE, TRUE },
//         { L"System manufacturer", FALSE, TRUE },
//         // some Gigabyte boards
//         { L"Default string", TRUE, TRUE },
//         { L"LTD Delovoy Office", TRUE, FALSE },
//         { L"Motherboard by ZOTAC", TRUE, FALSE },
//         // various boards
//         { L"Type2 - Board Manufacturer", TRUE, TRUE },
//         { L"Type2 - Board Vendor Name1", TRUE, TRUE },
//         { L"BASE_BOARD_MANUFACTURER", TRUE, TRUE },
//         { L"$(DEFAULT_STRING)", TRUE, TRUE },
//         { L"DEPO Computers", TRUE, FALSE },
//         { L"-", TRUE, TRUE },
//         { L"N/A", TRUE, TRUE },
//         { L"OEM", TRUE, TRUE },
//         { L"O.E.M", TRUE, TRUE },
//         { L"empty", TRUE, TRUE },
//         { L"insyde", TRUE, FALSE },
//         { L"Unknow", TRUE, TRUE },
//         { L"Not Applicable", TRUE, TRUE },
//         // distinguish between Oracle and older VirtualBox releases (Sun, etc.)
//         { L"innotek GmbH", TRUE, FALSE },
//     };
//     static const GENERIC_NAME Devices[] =
//     {
//         // some ASUS boards
//         { L"To Be Filled By O.E.M.", FALSE, TRUE },
//         { L"To Be Filled By O.E.M", FALSE, TRUE },
//         { L"All Series", TRUE, TRUE },
//         { L"System Product Name", TRUE, TRUE },
//         { L"System Name", TRUE, TRUE },
//         // some Gigabyte boards
//         { L"Default string", TRUE, TRUE },
//         // some MSI boards
//         { L"Please change product name", TRUE, TRUE },
//         // some Intel boards
//         { L"Computer", TRUE, TRUE },
//         { L"ChiefRiver Platform", TRUE, FALSE },
//         { L"OakTrail Platform", TRUE, FALSE },
//         { L"SharkBay Platform", TRUE, FALSE },
//         { L"HuronRiver Platform", TRUE, FALSE },
//         { L"SandyBridge Platform", TRUE, FALSE },
//         { L"Broadwell Platform", TRUE, FALSE },
//         { L"Kabylake Platform", TRUE, FALSE },
//         { L"Sabine Platform", TRUE, FALSE },
//         // various boards
//         { L"Base Board Product Name", TRUE, TRUE },
//         { L"Base Board Version", TRUE, TRUE },
//         { L"Type2 - Board Product Name1", TRUE, TRUE },
//         { L"Type2 - Board Product Name", TRUE, TRUE },
//         { L"Type2 - Board Version", TRUE, TRUE },
//         { L"MODEL_NAME", TRUE, TRUE },
//         { L"$(DEFAULT_STRING)", TRUE, TRUE },
//         { L"*", TRUE, TRUE },
//         { L"T", TRUE, TRUE },
//         { L"GEG", TRUE, TRUE },
//         { L"N/A", TRUE, TRUE },
//         { L"---", TRUE, TRUE },
//         { L"OEM", TRUE, TRUE },
//         { L"INVA", TRUE, TRUE },
//         { L"O.E.M", TRUE, TRUE },
//         { L"empty", TRUE, TRUE },
//         { L"DNSNB", TRUE, FALSE },
//         { L"12345", TRUE, FALSE },
//         { L"``````", TRUE, TRUE },
//         { L"Uknown", TRUE, TRUE },
//         { L"Desktop", FALSE, TRUE },
//         { L"Invalid", FALSE, TRUE },
//         { L"Reserved", TRUE, TRUE },
//         { L"Not Applicable", TRUE, TRUE },
//         { L"HaierComputer", TRUE, FALSE },
//         { L"DEPO Computers", TRUE, FALSE },
//         { L"InsydeH2O EFI BIOS", TRUE, TRUE },
//         { L"HP All-in-One", TRUE, FALSE },
//         { L"MP Server", TRUE, FALSE },
//         { L"0000000000", TRUE, TRUE },
//         // some Foxconn boards
//         { L"Aquarius Pro, Std, Elt Series", TRUE, FALSE },
//         // some ASUS server boards
//         { L"Aquarius Server", TRUE, FALSE },
//         { L"Aquarius Server G2", TRUE, FALSE },
//         // some Supermicro server boards
//         { L"Super Server", TRUE, FALSE },
//         // some Positivo devices
//         { L"POSITIVO MOBILE", FALSE, FALSE },
//     };
//     BOOL bMatch;
//     UINT i;
//
//     if (bRemove)
//     {
//         *bRemove = FALSE;
//     }
//     for (i = 0; i < _countof(Vendors); i++)
//     {
//         if (!ven)
//         {
//             break;
//         }
//         if (Vendors[i].bCaseSensitive)
//         {
//             bMatch = !wcscmp(ven, Vendors[i].pwName);
//         }
//         else
//         {
//             bMatch = !_wcsicmp(ven, Vendors[i].pwName);
//         }
//         if (bMatch)
//         {
//             if (bRemove)
//             {
//                 *bRemove = Vendors[i].bRemove;
//             }
//             return TRUE;
//         }
//     }
//
//     for (i = 0; i < _countof(Devices); i++)
//     {
//         if (!dev)
//         {
//             break;
//         }
//         if (Devices[i].bCaseSensitive)
//         {
//             bMatch = !wcscmp(dev, Devices[i].pwName);
//         }
//         else
//         {
//             bMatch = !_wcsicmp(dev, Devices[i].pwName);
//         }
//         if (bMatch)
//         {
//             if (bRemove)
//             {
//                 *bRemove = Devices[i].bRemove;
//             }
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// static
// void AppendSystemFamily(PWSTR pBuf, SIZE_T cchBuf, PCHAR * DmiStrings, PWSTR dev)
// {
//     static const PCSTR KnownFamilies[] =
//     {
//         "Eee PC",      // ASUS
//         "ThinkPad",    // Lenovo
//         "IdeaPad",     // Lenovo
//         "IdeaCentre",  // Lenovo
//     };
//     static const PCWSTR Aliases[] =
//     {
//         NULL,
//         NULL,
//         NULL,
//         L"IdeaCenter",
//     };
//     UINT i;
//     WCHAR wideStr[128];
//
//     for (i = 0; i < _countof(KnownFamilies); i++)
//     {
//         StringCchPrintfW(wideStr, _countof(wideStr), L"%S", KnownFamilies[i]);
//
//         if (wcsistr(dev, wideStr) == NULL &&
//             (!Aliases[i] || wcsistr(dev, Aliases[i]) == NULL) &&
//             DmiStrings[SYS_FAMILY] != NULL &&
//             !_stricmp(DmiStrings[SYS_FAMILY], KnownFamilies[i]))
//         {
//             if (wcslen(pBuf) > 0 && wcslen(dev) > 0)
//             {
//                 StringCchCatW(pBuf, cchBuf, L" ");
//             }
//             StringCchCatW(pBuf, cchBuf, wideStr);
//         }
//     }
// }
//
// static
// BOOL TrimNonPrintable(PCHAR DmiString)
// {
//     PCHAR c = DmiString;
//     if (!c)
//     {
//         return FALSE;
//     }
//     while (*c)
//     {
//         if (*c >= 0x20 && *c <= 0x7e)
//         {
//             c++;
//         }
//         else
//         {
//             *c = 0;
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// /* TrimNonPrintable function wrapper. It does special preprocessing
//  * so the function returns FALSE in some corner cases, making the parser
//  * use system strings anyway (instead of board strings). */
// static
// BOOL TrimNonPrintableProd(PCHAR DmiString)
// {
//     PCHAR c;
//
//     if (!DmiString)
//     {
//         return FALSE;
//     }
//
//     /* Special handling for HP with broken revision */
//     c = strstr(DmiString, "(\xFF\xFF");
//     if (c > DmiString)
//     {
//         *c = 0;
//     }
//
//     return TrimNonPrintable(DmiString);
// }
//
// BOOL GetSystemName(PWSTR pBuf, SIZE_T cchBuf)
// {
//     static const VENDOR_LONG_NAME LongNames[] =
//     {
//         { L"ASUSTeK", L"ASUS" },
//         { L"First International Computer", L"FIC" },
//         { L"Hewlett-Packard", L"HP" },
//         { L"MICRO-STAR", L"MSI" },
//         { L"SGI.COM", L"SGI" },
//         { L"Silicon Graphics International", L"SGI" },
//         { L"Intel(R) Client Systems", L"Intel" },
//         { L"InformationComputerSystems", L"ICS" },
//         { L"Bernecker + Rainer  Industrie-Elektronik", L"Bernecker & Rainer" },
//         { L"CHUWI INNOVATION AND TECHNOLOGY", L"CHUWI" },
//         { L"CHUWI INNOVATION LIMITED", L"CHUWI" },
//         { L"CHUWI  INNOVATION  LIMITED", L"CHUWI" },
//         { L"http://www.abit.com.tw/", L"ABIT" },
//         { L"http:\\\\www.abit.com.tw", L"ABIT" },
//         { L"www.abit.com.tw", L"ABIT" },
//         { L"CASPER BILGISAYAR SISTEMLERI A.S", L"Casper" },
//         { L"Colorful Technology And Development", L"Colorful" },
//         { L"Colorful Yu Gong Technology And Development", L"Colorful Yu Gong" },
//         { L"HaierComputer", L"Haier" },
//         { L"Haier Information Technology (Shen Zhen)", L"Haier" },
//         { L"HASEECOMPUTERS", L"Hasee" },
//         { L"HELIOS BUSINESS COMPUTER", L"HELIOS" },
//         { L"Shanghai Zongzhi InfoTech", L"Zongzhi" },
//         { L"TSING HUA TONGFANG CO.,LTD", L"TSINGHUA TONGFANG" },
//         { L"Yeston Digital Technology Co.,LTD", L"Yeston" },
//     };
//     static const REDUNDANT_WORD RedundantWords[] =
//     {
//         { L"Corporation", FALSE },
//         { L"Communication", FALSE },
//         { L"Computer", FALSE },
//         { L"Computers", FALSE },
//         { L"Group", FALSE },
//         { L"Cloud", FALSE },
//         { L"Center", FALSE },
//         { L"Systems", FALSE },
//         { L"Microsystems", FALSE },
//         { L"Infosystems", FALSE },
//         { L"Digital", FALSE },
//         { L"Electronics", FALSE },
//         { L"Electric", FALSE },
//         { L"Elektronik", FALSE },
//         { L"Software", FALSE },
//         { L"Foundation", FALSE },
//         { L"International", FALSE },
//         { L"Interantonal", FALSE }, // on purpose (some MSI boards)
//         { L"INTERANTIONAL", FALSE }, // on purpose (some MSI boards)
//         { L"Industrial", FALSE },
//         { L"Industrie", FALSE },
//         { L"Information", FALSE },
//         { L"Informatica", FALSE },
//         { L"Produkte", FALSE },
//         { L"Technology", FALSE },
//         { L"Tecohnology", FALSE }, // on purpose (some Gigabyte boards)
//         { L"Technologies", FALSE },
//         { L"Tecnologia", FALSE },
//         { L"Limited", FALSE },
//         { L"Int", FALSE },
//         { L"Inc", FALSE },
//         { L"Co", FALSE },
//         { L"Corp", FALSE },
//         { L"Crop", FALSE },
//         { L"LLC", FALSE },
//         { L"Ltd", FALSE },
//         { L"LTDA", FALSE },
//         { L"GmbH", FALSE },
//         { L"S.p.A", FALSE },
//         { L"A.S.", FALSE },
//         { L"S.A", FALSE },
//         { L"S.A.S", FALSE },
//         { L"S/A", FALSE },
//         { L"SA", FALSE },
//         { L"SAS", FALSE },
//         { L"BV", FALSE },
//         { L"AG", FALSE },
//         { L"OOO", TRUE },
//         { L"CJSC", FALSE },
//         { L"INT'L", FALSE },
//         { L"INTL", FALSE },
//         { L"plc", FALSE },
//     };
//     PVOID SMBiosBuf;
//     PCHAR DmiStrings[ID_STRINGS_MAX] = { 0 };
//     WCHAR ven[512], dev[512];
//     CHAR tmpstr[512];
//     BOOL bTrimProduct, bTrimFamily, bGenericName, bRemove;
//     UINT i;
//     PWCHAR j;
//
//     SMBiosBuf = LoadSMBiosData(DmiStrings);
//     if (!SMBiosBuf)
//     {
//         return FALSE;
//     }
//
//     TrimNonPrintable(DmiStrings[SYS_VENDOR]);
//     bTrimProduct = TrimNonPrintableProd(DmiStrings[SYS_PRODUCT]);
//     TrimNonPrintable(DmiStrings[SYS_VERSION]);
//     bTrimFamily = TrimNonPrintable(DmiStrings[SYS_FAMILY]);
//     TrimNonPrintable(DmiStrings[BOARD_VENDOR]);
//     TrimNonPrintable(DmiStrings[BOARD_NAME]);
//     TrimNonPrintable(DmiStrings[BOARD_VERSION]);
//
//     if (bTrimProduct)
//     {
//         if (DmiStrings[SYS_FAMILY] && !bTrimFamily)
//         {
//             DmiStrings[SYS_PRODUCT] = DmiStrings[SYS_FAMILY];
//             bTrimProduct = FALSE;
//         }
//     }
//
//     GetSMBiosStringW(DmiStrings[SYS_VENDOR], ven, _countof(ven), TRUE);
//     GetSMBiosStringW(DmiStrings[SYS_PRODUCT], dev, _countof(dev), TRUE);
//     bGenericName = IsGenericSystemName(ven, dev, NULL) || bTrimProduct;
//
//     if (wcslen(dev) == 0 ||
//         !wcscmp(dev, ven) ||
//         bGenericName)
//     {
//         BOOL bGenericVen = FALSE, bRemoveVen = FALSE, bGenericDev = (wcslen(dev) == 0 || !wcscmp(dev, ven) || bTrimProduct);
//
//         if (bGenericName && IsGenericSystemName(ven, NULL, &bRemove))
//         {
//             if (bRemove)
//             {
//                 *ven = 0;
//             }
//             bGenericVen = TRUE;
//             bRemoveVen = bRemove;
//         }
//         if (bGenericName && IsGenericSystemName(NULL, dev, &bRemove))
//         {
//             if (bRemove)
//             {
//                 *dev = 0;
//             }
//             bGenericDev = TRUE;
//         }
//         // system strings are unusable, use board strings
//         if (DmiStrings[BOARD_VENDOR] != NULL || !bGenericName)
//         {
//             if ((DmiStrings[BOARD_VENDOR] &&
//                 strlen(DmiStrings[BOARD_VENDOR]) >= 2 &&
//                 strstr(DmiStrings[BOARD_VENDOR], "  ") != DmiStrings[BOARD_VENDOR]) ||
//                 bGenericVen)
//             {
//                 GetSMBiosStringW(DmiStrings[BOARD_VENDOR], ven, _countof(ven), TRUE);
//             }
//             GetSMBiosStringW(DmiStrings[BOARD_NAME], dev, _countof(dev), TRUE);
//
//             if (IsGenericSystemName(ven, NULL, &bRemove) && bRemove)
//             {
//                 *ven = 0;
//
//                 if (bGenericVen && !bRemoveVen)
//                 {
//                     GetSMBiosStringW(DmiStrings[SYS_VENDOR], ven, _countof(ven), TRUE);
//                 }
//             }
//             if (IsGenericSystemName(NULL, dev, &bRemove) && bRemove)
//             {
//                 *dev = 0;
//
//                 if (!bGenericDev)
//                 {
//                     GetSMBiosStringW(DmiStrings[SYS_PRODUCT], dev, _countof(dev), TRUE);
//                 }
//             }
//             if (wcslen(dev) == 0 &&
//                 DmiStrings[SYS_VERSION] != NULL)
//             {
//                 GetSMBiosStringW(DmiStrings[SYS_VERSION], dev, _countof(dev), TRUE);
//
//                 if (IsGenericSystemName(NULL, dev, &bRemove) && bRemove)
//                 {
//                     *dev = 0;
//                 }
//             }
//             if (wcslen(dev) == 0 &&
//                 DmiStrings[BOARD_VERSION] != NULL)
//             {
//                 GetSMBiosStringW(DmiStrings[BOARD_VERSION], dev, _countof(dev), TRUE);
//
//                 if (IsGenericSystemName(NULL, dev, &bRemove) && bRemove)
//                 {
//                     *dev = 0;
//                 }
//             }
//         }
//         else if (DmiStrings[BOARD_NAME] != NULL)
//         {
//             GetSMBiosStringW(DmiStrings[BOARD_NAME], dev, _countof(dev), TRUE);
//
//             if (IsGenericSystemName(NULL, dev, &bRemove) && bRemove)
//             {
//                 *dev = 0;
//             }
//         }
//
//         if (wcslen(ven) == 0 && wcslen(dev) == 0)
//         {
//             // board strings are empty, use BIOS vendor string
//             GetSMBiosStringW(DmiStrings[BIOS_VENDOR], ven, _countof(ven), TRUE);
//         }
//     }
//     else
//     {
//         if (wcslen(ven) < 2)
//         {
//             GetSMBiosStringW(DmiStrings[BOARD_VENDOR], ven, _countof(ven), TRUE);
//
//             if (IsGenericSystemName(ven, NULL, &bRemove) && bRemove)
//             {
//                 *ven = 0;
//             }
//         }
//     }
//
//     // workaround for LORD ELECTRONICS
//     if (((j = wcsstr(ven, L" ")) != NULL) && (j - ven > 2))
//     {
//         i = j - ven;
//         if (!wcsncmp(ven + wcslen(ven) - i, ven, i))
//         {
//             ven[wcslen(ven) - i] = L'\0';
//         }
//     }
//
//     // make vendor strings shorter
//     for (i = 0; i < _countof(LongNames); i++)
//     {
//         if (wcsstr(dev, LongNames[i].pwLongName) == dev)
//         {
//             // swap ven and dev
//             StringCchCopyW(pBuf, cchBuf, ven);
//             StringCchCopyW(ven, _countof(ven), dev);
//             StringCchCopyW(dev, _countof(dev), pBuf);
//         }
//         wcsrep(ven, LongNames[i].pwLongName, LongNames[i].pwShortName, TRUE);
//     }
//
//     // remove redundant words
//     for (i = 0; i < _countof(RedundantWords); i++)
//     {
//         wcsrep(ven, RedundantWords[i].pwStr, L"", RedundantWords[i].bReplaceFirstWord);
//     }
//     for (i = 0; i < _countof(RedundantWords); i++)
//     {
//         StringCchCopyW(pBuf, cchBuf, RedundantWords[i].pwStr);
//         StringCchCatW(pBuf, cchBuf, L".");
//         wcsrep(ven, pBuf, L"", RedundantWords[i].bReplaceFirstWord);
//     }
//
//     // workaround for LENOVO notebooks
//     if (!_wcsicmp(ven, L"LENOVO"))
//     {
//         StringCchCopyW(ven, _countof(ven), L"Lenovo");
//
//         if (DmiStrings[SYS_VERSION] != NULL)
//         {
//             if (!strncmp(DmiStrings[SYS_VERSION], "ThinkPad   ", 11))
//             {
//                 DmiStrings[SYS_VERSION][8] = L'\0';
//             }
//             if (wcslen(dev) > 0 &&
//                 (!strcmp(DmiStrings[SYS_VERSION], "IdeaCentre") ||
//                 !strcmp(DmiStrings[SYS_VERSION], "ThinkPad")))
//             {
//                 DmiStrings[SYS_FAMILY] = DmiStrings[SYS_VERSION];
//                 DmiStrings[SYS_VERSION] = NULL;
//             }
//             else
//             {
//                 StringCchCopyA(tmpstr, _countof(tmpstr), DmiStrings[SYS_VERSION]);
//                 _strupr(tmpstr);
//             }
//         }
//
//         if (DmiStrings[SYS_VERSION] != NULL &&
//             strcmp(tmpstr, " ") &&
//             strcmp(tmpstr, "LENOVO") &&
//             strstr(tmpstr, "LENOVO   ") == NULL &&
//             strstr(tmpstr, "LENOVO PRODUCT") == NULL &&
//             strstr(tmpstr, "LENOVOPRODUCT") == NULL &&
//             strstr(tmpstr, "INVALID") == NULL &&
//             strncmp(tmpstr, "   ", 3) &&
//             (strlen(tmpstr) >= 3 || !IsDigitStrA(tmpstr)) &&
//             strstr(DmiStrings[SYS_VERSION], "Rev ") == NULL &&
//             strstr(DmiStrings[SYS_VERSION], "1.") == NULL &&
//             wcsistr(dev, L"System ") == NULL && // includes System x and ThinkSystem
//             wcsistr(dev, L"IdeaPad ") == NULL &&
//             wcsistr(dev, L"ThinkServer ") == NULL)
//         {
//             GetSMBiosStringW(DmiStrings[SYS_VERSION], dev, _countof(dev), TRUE);
//         }
//
//         if (wcsstr(dev, L"Lenovo-") == dev)
//         {
//             // replace "-" with space
//             dev[6] = L' ';
//         }
//
//         if (!wcscmp(dev, L"Lenovo"))
//         {
//             GetSMBiosStringW(DmiStrings[BOARD_NAME], dev, _countof(dev), TRUE);
//         }
//     }
//     if (!wcscmp(ven, L"IBM") &&
//         DmiStrings[SYS_VERSION] != NULL &&
//         (strstr(DmiStrings[SYS_VERSION], "ThinkPad ") != NULL ||
//          strstr(DmiStrings[SYS_VERSION], "ThinkCentre ") != NULL))
//     {
//         GetSMBiosStringW(DmiStrings[SYS_VERSION], dev, _countof(dev), TRUE);
//     }
//
//     // workaround for DEXP
//     if (!wcscmp(ven, L"DEXP"))
//     {
//         if (DmiStrings[SYS_PRODUCT] != NULL &&
//             DmiStrings[SYS_VERSION] != NULL &&
//             (!_stricmp(DmiStrings[SYS_PRODUCT], "Tablet PC") ||
//              !_stricmp(DmiStrings[SYS_PRODUCT], "Notebook") ||
//              !_stricmp(DmiStrings[SYS_PRODUCT], "Decktop")))
//         {
//             GetSMBiosStringW(DmiStrings[SYS_VERSION], dev, _countof(dev), TRUE);
//         }
//     }
//
//     // workaround for Razer Blade
//     if (!wcscmp(ven, L"Razer") && !wcscmp(dev, L"Blade"))
//     {
//         if (DmiStrings[SYS_VERSION] != NULL)
//         {
//             StringCchCopyW(ven, _countof(ven), L"Razer Blade");
//             GetSMBiosStringW(DmiStrings[SYS_VERSION], dev, _countof(dev), TRUE);
//         }
//     }
//
//     // workaround for MSI motherboards
//     if (!wcscmp(ven, L"MSI") &&
//         wcsstr(dev, L"MS-") != NULL &&
//         DmiStrings[BOARD_NAME] != NULL &&
//         strstr(DmiStrings[BOARD_NAME], "(MS-") != NULL)
//     {
//         GetSMBiosStringW(DmiStrings[BOARD_NAME], dev, _countof(dev), TRUE);
//     }
//     if (wcslen(ven) == 0 &&
//         wcsstr(dev, L"MS-") == dev)
//     {
//         StringCchCopyW(ven, _countof(ven), L"MSI");
//     }
//
//     // trim redundant characters
//     TrimPunctuation(ven);
//     TrimPunctuation(dev);
//
//     if (wcsistr(dev, ven) == dev ||
//         (!wcscmp(ven, L"ASUS") && wcsstr(dev, L"ASUS") != NULL) ||
//         (!wcscmp(ven, L"HP") && wcsstr(dev, L" by HP") != NULL) ||
//         (!wcscmp(ven, L"INTEL") && wcsstr(dev, L" INTEL") != NULL))
//     {
//         // device string contains vendor string, use second only
//         StringCchCopyW(pBuf, cchBuf, dev);
//     }
//     else
//     {
//         if (wcslen(ven) > 0 && wcslen(dev) > 0 && (j = wcschr(dev, L' ')))
//         {
//             // check if vendor string ends with first word of device string
//             i = j - dev;
//             if (wcslen(ven) > i && !_wcsnicmp(ven + wcslen(ven) - i, dev, i))
//             {
//                 ven[wcslen(ven) - i] = L'\0';
//                 TrimPunctuation(ven);
//             }
//         }
//         StringCchCopyW(pBuf, cchBuf, ven);
//         AppendSystemFamily(pBuf, cchBuf, DmiStrings, dev);
//         if (wcslen(pBuf) > 0 && wcslen(dev) > 0)
//         {
//             StringCchCatW(pBuf, cchBuf, L" ");
//         }
//         StringCchCatW(pBuf, cchBuf, dev);
//     }
//
//     FreeSMBiosData(SMBiosBuf);
//
//     return (wcslen(pBuf) > 0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
