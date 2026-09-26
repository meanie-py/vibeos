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
        if *c >= (b'0') as i8 && *c <= (b'9') as i8 {
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

    while i < (Length) as u32 && IsPunctuation(*pStr.offset(i as isize)) != 0 {
        i += 1;
    }

    if i > 0 {
        Length -= (i) as usize;
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
        if towlower((*x) as i32) == towlower((*b) as i32) {
            y = x;
            c = b as *mut wchar_t;
            while *y != 0 && *c != 0 && towlower((*y) as i32) == towlower((*c) as i32) {
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

    pwsStr = HeapAlloc(GetProcessHeap(), 0, (lenStr * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
    if pwsStr.is_null()
    {
        return;
    }
    StringCchCopyW(pwsStr, lenStr, w!(" ") as *const u16);
    StringCchCatW(pwsStr, lenStr, pwStr);
    StringCchCatW(pwsStr, lenStr, w!(" ") as *const u16);

    pwsFind = HeapAlloc(GetProcessHeap(), 0, (lenFind * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
    if pwsFind.is_null()
    {
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }
    StringCchCopyW(pwsFind, lenFind, w!(" ") as *const u16);
    StringCchCatW(pwsFind, lenFind, pwFind);
    StringCchCatW(pwsFind, lenFind, w!(" ") as *const u16);

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

    pwsReplace = HeapAlloc(GetProcessHeap(), 0, (lenReplace * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
    if pwsReplace.is_null()
    {
        HeapFree(GetProcessHeap(), 0, pwsFind as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, pwsStr as *mut core::ffi::c_void);
        return;
    }
    StringCchCopyW(pwsReplace, lenReplace, w!(" ") as *const u16);
    StringCchCatW(pwsReplace, lenReplace, pwReplace);
    StringCchCatW(pwsReplace, lenReplace, w!(" ") as *const u16);

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
// Hand-rewritten from dll/cpl/sysdm/smbios.c:257. The paid body replaced every `L"..."` in
// both tables with TRUE or FALSE, dropped two Devices rows, and inverted the control flow --
// `i` started at TRUE, a hit returned FALSE, and the fall-through returned TRUE, so the
// predicate answered the exact opposite of the question. Sealing did not catch it: its
// invariant is over the CONSTANT set, and a table of STRING LITERALS is not in that set, so
// the model filled 71 pointer slots from the only tokens it had been given.
// `BOOL` stays `u8` here as the rest of the file spells it; the function is `static` in the C
// and has no ABI contract with anything outside this crate.
unsafe extern "C" fn IsGenericSystemName(ven: PCWSTR, dev: PCWSTR, bRemove: *mut u8) -> u8 {
    struct GENERIC_NAME {
        pwName: PCWSTR,
        bCaseSensitive: u8,
        bRemove: u8,
    }

    static Vendors: SyncTable<[GENERIC_NAME; 20]> = SyncTable([
        GENERIC_NAME { pwName: w!("To Be Filled By O.E.M."), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("To Be Filled By O.E.M"), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("System manufacturer"), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Default string"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("LTD Delovoy Office"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Motherboard by ZOTAC"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Type2 - Board Manufacturer"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Type2 - Board Vendor Name1"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("BASE_BOARD_MANUFACTURER"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("$(DEFAULT_STRING)"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("DEPO Computers"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("-"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("N/A"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("OEM"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("O.E.M"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("empty"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("insyde"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Unknow"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Not Applicable"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("innotek GmbH"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
    ]);

    static Devices: SyncTable<[GENERIC_NAME; 51]> = SyncTable([
        GENERIC_NAME { pwName: w!("To Be Filled By O.E.M."), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("To Be Filled By O.E.M"), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("All Series"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("System Product Name"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("System Name"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Default string"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Please change product name"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Computer"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("ChiefRiver Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("OakTrail Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("SharkBay Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("HuronRiver Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("SandyBridge Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Broadwell Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Kabylake Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Sabine Platform"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Base Board Product Name"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Base Board Version"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Type2 - Board Product Name1"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Type2 - Board Product Name"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Type2 - Board Version"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("MODEL_NAME"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("$(DEFAULT_STRING)"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("*"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("T"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("GEG"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("N/A"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("---"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("OEM"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("INVA"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("O.E.M"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("empty"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("DNSNB"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("12345"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("``````"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Uknown"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Desktop"), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Invalid"), bCaseSensitive: FALSE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Reserved"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Not Applicable"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("HaierComputer"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("DEPO Computers"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("InsydeH2O EFI BIOS"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("HP All-in-One"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("MP Server"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("0000000000"), bCaseSensitive: TRUE as u8, bRemove: TRUE as u8 },
        GENERIC_NAME { pwName: w!("Aquarius Pro, Std, Elt Series"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Aquarius Server"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Aquarius Server G2"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("Super Server"), bCaseSensitive: TRUE as u8, bRemove: FALSE as u8 },
        GENERIC_NAME { pwName: w!("POSITIVO MOBILE"), bCaseSensitive: FALSE as u8, bRemove: FALSE as u8 },
    ]);

    let mut bMatch: u8;
    let mut i: u32;

    if !bRemove.is_null() {
        *bRemove = (FALSE) as u8;
    }

    i = 0;
    while i < _countof!(Vendors) {
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
            return (TRUE) as u8;
        }
        i += 1;
    }

    i = 0;
    while i < _countof!(Devices) {
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
            return (TRUE) as u8;
        }
        i += 1;
    }

    (FALSE) as u8
}
unsafe extern "C" fn AppendSystemFamily(pBuf: PWSTR, cchBuf: usize, DmiStrings: *mut PCSTR, dev: PWSTR) {
    // Three more literals the paid body dropped: Aliases[3] is `L"IdeaCenter"`, the
    // StringCchPrintfW format below is `L"%S"`, and the separator is `L" "`.
    static KnownFamilies: SyncTable<[PCSTR; 4]> = SyncTable([
        b"Eee PC\0".as_ptr() as PCSTR,
        b"ThinkPad\0".as_ptr() as PCSTR,
        b"IdeaPad\0".as_ptr() as PCSTR,
        b"IdeaCentre\0".as_ptr() as PCSTR,
    ]);
    static Aliases: SyncTable<[PCWSTR; 4]> = SyncTable([
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        w!("IdeaCenter") as PCWSTR,
    ]);
    let mut i: u32;
    let mut wideStr: [wchar_t; 128] = [0; 128];

    i = 0;
    while i < _countof!(KnownFamilies) as u32 {
        StringCchPrintfW!(wideStr.as_mut_ptr(), _countof!(wideStr) as usize, w!("%S"), *KnownFamilies.get_unchecked(i as usize));

        if wcsistr(dev, wideStr.as_ptr()) == core::ptr::null_mut() &&
            (*Aliases.get_unchecked(i as usize) == core::ptr::null() || wcsistr(dev, *Aliases.get_unchecked(i as usize)) == core::ptr::null_mut()) &&
            *DmiStrings.offset(SYS_FAMILY as isize) != core::ptr::null() &&
            _stricmp(*DmiStrings.offset(SYS_FAMILY as isize), *KnownFamilies.get_unchecked(i as usize)) == 0
        {
            if wcslen(pBuf) > 0 && wcslen(dev) > 0 {
                StringCchCatW(pBuf, cchBuf, w!(" "));
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
    static LongNames: SyncTable<[VENDOR_LONG_NAME; 25]> = SyncTable([
        VENDOR_LONG_NAME { pwLongName: w!("ASUSTeK") as PCWSTR, pwShortName: w!("ASUS") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("First International Computer") as PCWSTR, pwShortName: w!("FIC") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Hewlett-Packard") as PCWSTR, pwShortName: w!("HP") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("MICRO-STAR") as PCWSTR, pwShortName: w!("MSI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("SGI.COM") as PCWSTR, pwShortName: w!("SGI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Silicon Graphics International") as PCWSTR, pwShortName: w!("SGI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Intel(R) Client Systems") as PCWSTR, pwShortName: w!("Intel") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("InformationComputerSystems") as PCWSTR, pwShortName: w!("ICS") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Bernecker + Rainer  Industrie-Elektronik") as PCWSTR, pwShortName: w!("Bernecker & Rainer") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("CHUWI INNOVATION AND TECHNOLOGY") as PCWSTR, pwShortName: w!("CHUWI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("CHUWI INNOVATION LIMITED") as PCWSTR, pwShortName: w!("CHUWI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("CHUWI  INNOVATION  LIMITED") as PCWSTR, pwShortName: w!("CHUWI") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("http://www.abit.com.tw/") as PCWSTR, pwShortName: w!("ABIT") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("http:\\\\www.abit.com.tw") as PCWSTR, pwShortName: w!("ABIT") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("www.abit.com.tw") as PCWSTR, pwShortName: w!("ABIT") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("CASPER BILGISAYAR SISTEMLERI A.S") as PCWSTR, pwShortName: w!("Casper") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Colorful Technology And Development") as PCWSTR, pwShortName: w!("Colorful") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Colorful Yu Gong Technology And Development") as PCWSTR, pwShortName: w!("Colorful Yu Gong") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("HaierComputer") as PCWSTR, pwShortName: w!("Haier") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Haier Information Technology (Shen Zhen)") as PCWSTR, pwShortName: w!("Haier") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("HASEECOMPUTERS") as PCWSTR, pwShortName: w!("Hasee") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("HELIOS BUSINESS COMPUTER") as PCWSTR, pwShortName: w!("HELIOS") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Shanghai Zongzhi InfoTech") as PCWSTR, pwShortName: w!("Zongzhi") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("TSING HUA TONGFANG CO.,LTD") as PCWSTR, pwShortName: w!("TSINGHUA TONGFANG") as PCWSTR },
        VENDOR_LONG_NAME { pwLongName: w!("Yeston Digital Technology Co.,LTD") as PCWSTR, pwShortName: w!("Yeston") as PCWSTR },
    ]);
    static RedundantWords: SyncTable<[REDUNDANT_WORD; 52]> = SyncTable([
        REDUNDANT_WORD { pwStr: w!("Corporation") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Communication") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Computer") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Computers") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Group") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Cloud") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Center") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Systems") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Microsystems") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Infosystems") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Digital") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Electronics") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Electric") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Elektronik") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Software") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Foundation") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("International") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Interantonal") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("INTERANTIONAL") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Industrial") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Industrie") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Information") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Informatica") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Produkte") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Technology") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Tecohnology") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Technologies") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Tecnologia") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Limited") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Int") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Inc") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Co") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Corp") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Crop") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("LLC") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("Ltd") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("LTDA") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("GmbH") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("S.p.A") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("A.S.") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("S.A") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("S.A.S") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("S/A") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("SA") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("SAS") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("BV") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("AG") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("OOO") as PCWSTR, bReplaceFirstWord: TRUE },
        REDUNDANT_WORD { pwStr: w!("CJSC") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("INT'L") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("INTL") as PCWSTR, bReplaceFirstWord: FALSE },
        REDUNDANT_WORD { pwStr: w!("plc") as PCWSTR, bReplaceFirstWord: FALSE },
    ]);
    let mut SMBiosBuf: *mut core::ffi::c_void;
    // smbios.c:576 is `PCHAR DmiStrings[]`, not PCSTR: these get trimmed in place.
    let mut DmiStrings: [PCHAR; (ID_STRINGS_MAX) as usize] = [core::ptr::null_mut(); (ID_STRINGS_MAX) as usize];
    let mut ven: [u16; 512] = [0; 512];
    let mut dev: [u16; 512] = [0; 512];
    let mut tmpstr: [i8; 512] = [0; 512];
    let mut bTrimProduct: u8;
    let mut bTrimFamily: u8;
    let mut bGenericName: u8;
    let mut bRemove: u8 = 0;
    let mut i: u32;
    let mut j: PWSTR;

    SMBiosBuf = LoadSMBiosData(DmiStrings.as_mut_ptr());
    if SMBiosBuf.is_null() {
        return (FALSE) as u8;
    }

    TrimNonPrintable(DmiStrings[(SYS_VENDOR) as usize]);
    bTrimProduct = (TrimNonPrintableProd(DmiStrings[(SYS_PRODUCT) as usize])) as u8;
    TrimNonPrintable(DmiStrings[(SYS_VERSION) as usize]);
    bTrimFamily = (TrimNonPrintable(DmiStrings[(SYS_FAMILY) as usize])) as u8;
    TrimNonPrintable(DmiStrings[(BOARD_VENDOR) as usize]);
    TrimNonPrintable(DmiStrings[(BOARD_NAME) as usize]);
    TrimNonPrintable(DmiStrings[(BOARD_VERSION) as usize]);

    if bTrimProduct != 0 {
        if !DmiStrings[(SYS_FAMILY) as usize].is_null() && bTrimFamily == 0 {
            DmiStrings[(SYS_PRODUCT) as usize] = DmiStrings[(SYS_FAMILY) as usize];
            bTrimProduct = (FALSE) as u8;
        }
    }

    GetSMBiosStringW(DmiStrings[(SYS_VENDOR) as usize], ven.as_mut_ptr(), _countof!(ven), TRUE);
    GetSMBiosStringW(DmiStrings[(SYS_PRODUCT) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
    bGenericName = IsGenericSystemName(ven.as_ptr(), dev.as_ptr(), core::ptr::null_mut()) | bTrimProduct;

    if wcslen(dev.as_ptr()) == 0 ||
        wcscmp(dev.as_ptr(), ven.as_ptr()) == 0 ||
        bGenericName != 0
    {
        let mut bGenericVen: u8 = (FALSE) as u8;
        let mut bRemoveVen: u8 = (FALSE) as u8;
        let mut bGenericDev: u8 = if wcslen(dev.as_ptr()) == 0 || wcscmp(dev.as_ptr(), ven.as_ptr()) == 0 || bTrimProduct != 0 { 1 } else { 0 };

        if bGenericName != 0 && IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 {
            if bRemove != 0 {
                *ven.as_mut_ptr() = 0;
            }
            bGenericVen = (TRUE) as u8;
            bRemoveVen = bRemove;
        }
        if bGenericName != 0 && IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 {
            if bRemove != 0 {
                *dev.as_mut_ptr() = 0;
            }
            bGenericDev = (TRUE) as u8;
        }

        if !DmiStrings[(BOARD_VENDOR) as usize].is_null() || bGenericName == 0 {
            if (!DmiStrings[(BOARD_VENDOR) as usize].is_null() &&
                strlen(DmiStrings[(BOARD_VENDOR) as usize]) >= 2 &&
                strstr(DmiStrings[(BOARD_VENDOR) as usize], b"  \0".as_ptr() as PCSTR) != DmiStrings[(BOARD_VENDOR) as usize]) ||
                bGenericVen != 0
            {
                GetSMBiosStringW(DmiStrings[(BOARD_VENDOR) as usize], ven.as_mut_ptr(), _countof!(ven), TRUE);
            }
            GetSMBiosStringW(DmiStrings[(BOARD_NAME) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);

            if IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 && bRemove != 0 {
                *ven.as_mut_ptr() = 0;

                if bGenericVen != 0 && bRemoveVen == 0 {
                    GetSMBiosStringW(DmiStrings[(SYS_VENDOR) as usize], ven.as_mut_ptr(), _countof!(ven), TRUE);
                }
            }
            if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                *dev.as_mut_ptr() = 0;

                if bGenericDev == 0 {
                    GetSMBiosStringW(DmiStrings[(SYS_PRODUCT) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
                }
            }
            if wcslen(dev.as_ptr()) == 0 &&
                !DmiStrings[(SYS_VERSION) as usize].is_null()
            {
                GetSMBiosStringW(DmiStrings[(SYS_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);

                if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                    *dev.as_mut_ptr() = 0;
                }
            }
            if wcslen(dev.as_ptr()) == 0 &&
                !DmiStrings[(BOARD_VERSION) as usize].is_null()
            {
                GetSMBiosStringW(DmiStrings[(BOARD_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);

                if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                    *dev.as_mut_ptr() = 0;
                }
            }
        } else if !DmiStrings[(BOARD_NAME) as usize].is_null() {
            GetSMBiosStringW(DmiStrings[(BOARD_NAME) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);

            if IsGenericSystemName(core::ptr::null(), dev.as_ptr(), &mut bRemove) != 0 && bRemove != 0 {
                *dev.as_mut_ptr() = 0;
            }
        }

        if wcslen(ven.as_ptr()) == 0 && wcslen(dev.as_ptr()) == 0 {
            GetSMBiosStringW(DmiStrings[(BIOS_VENDOR) as usize], ven.as_mut_ptr(), _countof!(ven), TRUE);
        }
    } else {
        if wcslen(ven.as_ptr()) < 2 {
            GetSMBiosStringW(DmiStrings[(BOARD_VENDOR) as usize], ven.as_mut_ptr(), _countof!(ven), TRUE);

            if IsGenericSystemName(ven.as_ptr(), core::ptr::null(), &mut bRemove) != 0 && bRemove != 0 {
                *ven.as_mut_ptr() = 0;
            }
        }
    }

    j = wcsstr(ven.as_ptr(), w!(" ") as PCWSTR);
    if !j.is_null() && (j as usize - ven.as_ptr() as usize) / 2 > 2 {
        i = ((j as usize - ven.as_ptr() as usize) / 2) as u32;
        if wcsncmp(ven.as_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize), ven.as_ptr(), i as usize) == 0 {
            *ven.as_mut_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize) = 0;
        }
    }

    for i in 0..25 {
        if wcsstr(dev.as_mut_ptr(), LongNames[i as usize].pwLongName) == dev.as_mut_ptr() {
            StringCchCopyW(pBuf, cchBuf, ven.as_ptr());
            StringCchCopyW(ven.as_mut_ptr(), (_countof!(ven)) as usize, dev.as_ptr());
            StringCchCopyW(dev.as_mut_ptr(), (_countof!(dev)) as usize, pBuf);
        }
        wcsrep(ven.as_mut_ptr(), LongNames[i as usize].pwLongName, LongNames[i as usize].pwShortName, TRUE);
    }

    for i in 0..52 {
        wcsrep(ven.as_mut_ptr(), RedundantWords[i as usize].pwStr, w!("") as PCWSTR, RedundantWords[i as usize].bReplaceFirstWord as i32);
    }
    for i in 0..52 {
        StringCchCopyW(pBuf, cchBuf, RedundantWords[i as usize].pwStr);
        StringCchCatW(pBuf, cchBuf, w!(".") as PCWSTR);
        wcsrep(ven.as_mut_ptr(), pBuf, w!("") as PCWSTR, RedundantWords[i as usize].bReplaceFirstWord as i32);
    }

    if _wcsicmp(ven.as_ptr(), w!("LENOVO") as PCWSTR) == 0 {
        StringCchCopyW(ven.as_mut_ptr(), (_countof!(ven)) as usize, w!("Lenovo") as PCWSTR);

        if !DmiStrings[(SYS_VERSION) as usize].is_null() {
            if strncmp(DmiStrings[(SYS_VERSION) as usize], b"ThinkPad   \0".as_ptr() as PCSTR, 11) == 0 {
                *DmiStrings[(SYS_VERSION) as usize].offset(8 as isize) = 0;
            }
            if wcslen(dev.as_ptr()) > 0 &&
                (strcmp(DmiStrings[(SYS_VERSION) as usize], b"IdeaCentre\0".as_ptr() as PCSTR) == 0 ||
                 strcmp(DmiStrings[(SYS_VERSION) as usize], b"ThinkPad\0".as_ptr() as PCSTR) == 0)
            {
                DmiStrings[(SYS_FAMILY) as usize] = DmiStrings[(SYS_VERSION) as usize];
                DmiStrings[(SYS_VERSION) as usize] = core::ptr::null_mut();
            } else {
                StringCchCopyA(tmpstr.as_mut_ptr(), (_countof!(tmpstr)) as usize, DmiStrings[(SYS_VERSION) as usize]);
                _strupr(tmpstr.as_mut_ptr());
            }
        }

        if !DmiStrings[(SYS_VERSION) as usize].is_null() &&
            (strcmp(tmpstr.as_ptr(), b" \0".as_ptr() as PCSTR) == 0) &&
            (strcmp(tmpstr.as_ptr(), b"LENOVO\0".as_ptr() as PCSTR) == 0) &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVO   \0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVO PRODUCT\0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"LENOVOPRODUCT\0".as_ptr() as PCSTR).is_null() &&
            strstr(tmpstr.as_mut_ptr(), b"INVALID\0".as_ptr() as PCSTR).is_null() &&
            strncmp(tmpstr.as_mut_ptr(), b"   \0".as_ptr() as PCSTR, 3) == 0 &&
            (strlen(tmpstr.as_mut_ptr()) >= 3 || IsDigitStrA(tmpstr.as_mut_ptr()) == 0) &&
            strstr(DmiStrings[(SYS_VERSION) as usize], b"Rev \0".as_ptr() as PCSTR).is_null() &&
            strstr(DmiStrings[(SYS_VERSION) as usize], b"1.\0".as_ptr() as PCSTR).is_null() &&
            wcsistr(dev.as_ptr(), w!("System ") as PCWSTR).is_null() &&
            wcsistr(dev.as_ptr(), w!("IdeaPad ") as PCWSTR).is_null() &&
            wcsistr(dev.as_ptr(), w!("ThinkServer ") as PCWSTR).is_null()
        {
            GetSMBiosStringW(DmiStrings[(SYS_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
        }

        if wcsstr(dev.as_mut_ptr(), w!("Lenovo-")) == dev.as_mut_ptr() {
            *dev.as_mut_ptr().offset(6 as isize) = b' ' as u16;
        }

        if wcscmp(dev.as_ptr(), w!("Lenovo") as PCWSTR) == 0 {
            GetSMBiosStringW(DmiStrings[(BOARD_NAME) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
        }
    }
    if wcscmp(ven.as_ptr(), w!("IBM") as PCWSTR) == 0 &&
        !DmiStrings[(SYS_VERSION) as usize].is_null() &&
        (!strstr(DmiStrings[(SYS_VERSION) as usize], b"ThinkPad \0".as_ptr() as PCSTR).is_null() ||
         !strstr(DmiStrings[(SYS_VERSION) as usize], b"ThinkCentre \0".as_ptr() as PCSTR).is_null())
    {
        GetSMBiosStringW(DmiStrings[(SYS_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
    }

    if wcscmp(ven.as_ptr(), w!("DEXP") as PCWSTR) == 0 {
        if !DmiStrings[(SYS_PRODUCT) as usize].is_null() &&
            !DmiStrings[(SYS_VERSION) as usize].is_null() &&
            (_stricmp(DmiStrings[(SYS_PRODUCT) as usize], b"Tablet PC\0".as_ptr() as PCSTR) == 0 ||
             _stricmp(DmiStrings[(SYS_PRODUCT) as usize], b"Notebook\0".as_ptr() as PCSTR) == 0 ||
             _stricmp(DmiStrings[(SYS_PRODUCT) as usize], b"Decktop\0".as_ptr() as PCSTR) == 0)
        {
            GetSMBiosStringW(DmiStrings[(SYS_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
        }
    }

    if wcscmp(ven.as_ptr(), w!("Razer") as PCWSTR) == 0 && wcscmp(dev.as_ptr(), w!("Blade") as PCWSTR) == 0 {
        if !DmiStrings[(SYS_VERSION) as usize].is_null() {
            StringCchCopyW(ven.as_mut_ptr(), (_countof!(ven)) as usize, w!("Razer Blade") as PCWSTR);
            GetSMBiosStringW(DmiStrings[(SYS_VERSION) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
        }
    }

    if wcscmp(ven.as_ptr(), w!("MSI") as PCWSTR) == 0 &&
        !wcsstr(dev.as_ptr(), w!("MS-") as PCWSTR).is_null() &&
        !DmiStrings[(BOARD_NAME) as usize].is_null() &&
        !strstr(DmiStrings[(BOARD_NAME) as usize], b"(MS-\0".as_ptr() as PCSTR).is_null()
    {
        GetSMBiosStringW(DmiStrings[(BOARD_NAME) as usize], dev.as_mut_ptr(), _countof!(dev), TRUE);
    }
    if wcslen(ven.as_ptr()) == 0 &&
        wcsstr(dev.as_mut_ptr(), w!("MS-")) == dev.as_mut_ptr()
    {
        StringCchCopyW(ven.as_mut_ptr(), (_countof!(ven)) as usize, w!("MSI") as PCWSTR);
    }

    TrimPunctuation(ven.as_mut_ptr());
    TrimPunctuation(dev.as_mut_ptr());

    if wcsistr(dev.as_mut_ptr(), ven.as_mut_ptr()) == dev.as_mut_ptr() ||
        (wcscmp(ven.as_ptr(), w!("ASUS") as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), w!("ASUS") as PCWSTR).is_null()) ||
        (wcscmp(ven.as_ptr(), w!("HP") as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), w!(" by HP") as PCWSTR).is_null()) ||
        (wcscmp(ven.as_ptr(), w!("INTEL") as PCWSTR) == 0 && !wcsstr(dev.as_ptr(), w!(" INTEL") as PCWSTR).is_null())
    {
        StringCchCopyW(pBuf, cchBuf, dev.as_ptr());
    } else {
        if wcslen(ven.as_ptr()) > 0 && wcslen(dev.as_ptr()) > 0 {
            j = wcschr(dev.as_ptr(), (b' ' as u16) as WCHAR);
            if !j.is_null() {
                i = ((j as usize - dev.as_ptr() as usize) / 2) as u32;
                if wcslen(ven.as_ptr()) > (i) as usize && _wcsnicmp(ven.as_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize), dev.as_ptr(), i as usize) == 0 {
                    *ven.as_mut_ptr().offset(wcslen(ven.as_ptr()) as isize - i as isize) = 0;
                    TrimPunctuation(ven.as_mut_ptr());
                }
            }
        }
        StringCchCopyW(pBuf, cchBuf, ven.as_ptr());
        AppendSystemFamily(pBuf, cchBuf, DmiStrings.as_mut_ptr() as *mut PCSTR, dev.as_mut_ptr());
        if wcslen(pBuf) > 0 && wcslen(dev.as_ptr()) > 0 {
            StringCchCatW(pBuf, cchBuf, w!(" ") as PCWSTR);
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
