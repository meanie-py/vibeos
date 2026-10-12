// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_00D5A895B81B: [u16; 12] = [
    0x002Fu16, 0x004D, 0x0041, 0x0043, 0x0052, 0x004F, 0x0046, 0x0049, 0x004C, 0x0045,
    0x003D, 0x0000,
];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_0EB2C50BC926: [u16; 4] = [0x002Fu16, 0x004D, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_1CF1ED85F35F: [u16; 6] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_1F0448BC599C: [u16; 8] = [0x002Fu16, 0x004D, 0x0041, 0x0043, 0x0052, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_5A9AC44B1B5A: [u16; 9] = [0x002Fu16, 0x004D, 0x0041, 0x0043, 0x0052, 0x004F, 0x0053, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_618611CD599B: [u16; 10] = [0x002Fu16, 0x0045, 0x0058, 0x0045, 0x004E, 0x0041, 0x004D, 0x0045, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_6C126795E198: [u16; 6] = [0x005Bu16, 0x0025, 0x0073, 0x005D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_731D7EC48EEF: [u16; 3] = [0x0020u16, 0x0009, 0x0000];
pub static RUSTCONV_WSTR_7EF8D2448D12: [u16; 4] = [0x0041u16, 0x004C, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_8BBC8C8BCD07: [u16; 11] = [
    0x002Fu16, 0x004C, 0x0049, 0x0053, 0x0054, 0x0053, 0x0049, 0x005A, 0x0045, 0x003D,
    0x0000,
];
pub static RUSTCONV_WSTR_90B77BCB8ED8: [u16; 12] = [
    0x002Fu16, 0x004F, 0x0056, 0x0045, 0x0052, 0x0053, 0x0054, 0x0052, 0x0049, 0x004B,
    0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9DF129D38A02: [u16; 3] = [0x002Fu16, 0x0048, 0x0000];
pub static RUSTCONV_WSTR_AFA42E523754: [u16; 9] = [0x002Fu16, 0x0048, 0x0049, 0x0053, 0x0054, 0x004F, 0x0052, 0x0059, 0x0000];
pub static RUSTCONV_WSTR_B85BE71CE3BC: [u16; 3] = [0x0072u16, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_BDDFEBAAEC28: [u16; 8] = [0x002Fu16, 0x0049, 0x004E, 0x0053, 0x0045, 0x0052, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_ED53584BB2A3: [u16; 11] = [
    0x002Fu16, 0x0052, 0x0045, 0x0049, 0x004E, 0x0053, 0x0054, 0x0041, 0x004C, 0x004C,
    0x0000,
];
pub static RUSTCONV_WSTR_EF2E9B46DA39: [u16; 3] = [0x002Fu16, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_F27C0439A02B: [u16; 5] = [0x0020u16, 0x0020, 0x0020, 0x0020, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_68E64E56CA39: [u16; 8] = [0x0063u16, 0x006D, 0x0064, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];

#[no_mangle]
pub static mut szStringBuf: [WCHAR; 2000] = [0; 2000];

#[no_mangle]
pub static mut pszExeName: LPWSTR = unsafe {
    RUSTCONV_C2R_WSTR_68E64E56CA39.as_ptr() as LPWSTR
};

pub const MAX_STRING: usize = 2000;

unsafe extern "C" fn SetInsert(dwFlag: DWORD) {
    /*
     * NOTE: Enabling the ENABLE_INSERT_MODE mode can also be done by calling
     * kernel32:SetConsoleCommandHistoryMode(CONSOLE_OVERSTRIKE) (deprecated).
     */
    let mut dwMode: DWORD = 0;
    let hConsole: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    GetConsoleMode(hConsole, core::ptr::addr_of_mut!(dwMode));
    dwMode |= ENABLE_EXTENDED_FLAGS;
    SetConsoleMode(hConsole, (dwMode & !ENABLE_INSERT_MODE) | dwFlag);
}

unsafe extern "C" fn PrintHistory() {
    let mut Length: DWORD = GetConsoleCommandHistoryLengthW(pszExeName);
    let mut HistBuf: PBYTE;
    let mut Hist: *mut WCHAR;
    let mut HistEnd: *mut WCHAR;

    HistBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        Length as SIZE_T,
    ) as PBYTE;
    if HistBuf.is_null() {
        return;
    }
    Hist = HistBuf as *mut WCHAR;
    HistEnd = HistBuf.offset(Length as isize) as *mut WCHAR;

    if GetConsoleCommandHistoryW(Hist as LPWSTR, Length, pszExeName) != 0 {
        while Hist < HistEnd {
            wprintf(RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), Hist);
            Hist = Hist.offset(
                (wcslen(Hist) + 1) as isize,
            );
        }
    }

    HeapFree(GetProcessHeap(), 0, HistBuf as PVOID);
}

unsafe extern "C" fn SetMacro(definition: LPWSTR) -> INT {
    let mut name: *mut WCHAR = definition;
    let mut nameend: *mut WCHAR;
    let mut text: *mut WCHAR;
    let mut temp: WCHAR;

    while *name == 0x0020u16 {
        name = name.offset(1);
    }

    /* error if no '=' found */
    nameend = wcschr(name, 0x003Du16);
    if !nameend.is_null() {
        text = nameend.offset(1 as isize);
        while *text == 0x0020u16 {
            text = text.offset(1);
        }

        while nameend > name && *nameend.offset(-(1 as isize)) == 0x0020u16 {
            nameend = nameend.offset(-1);
        }

        /* Split rest into name and substitute */
        temp = *nameend;
        *nameend = 0x0000u16;
        /* Don't allow spaces in the name, since such a macro would be unusable */
        if wcschr(name, 0x0020u16).is_null()
            && AddConsoleAliasW(name, text, pszExeName) != 0
        {
            return 0;
        }
        *nameend = temp;
    }

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        IDS_INVALID_MACRO_DEF as UINT,
        core::ptr::addr_of_mut!(szStringBuf) as LPWSTR,
        ARRAYSIZE!(szStringBuf) as INT,
    );
    wprintf(core::ptr::addr_of_mut!(szStringBuf) as *mut WCHAR, definition);
    return 1;
}

unsafe extern "C" fn PrintMacros(pszExeNameParam: LPWSTR, Indent: LPWSTR) {
    let Length: DWORD = GetConsoleAliasesLengthW(pszExeNameParam as LPCWSTR);
    let mut AliasBuf: PBYTE;
    let mut Alias: *mut WCHAR;
    let mut AliasEnd: *mut WCHAR;

    AliasBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (Length as SIZE_T).wrapping_mul(::core::mem::size_of::<BYTE>() as SIZE_T),
    ) as PBYTE;
    if AliasBuf.is_null() {
        return;
    }
    Alias = AliasBuf as *mut WCHAR;
    AliasEnd = AliasBuf.add(Length as usize) as *mut WCHAR;

    if GetConsoleAliasesW(
        Alias as LPWSTR,
        (Length as usize).wrapping_mul(::core::mem::size_of::<BYTE>() as usize) as DWORD,
        pszExeNameParam as LPCWSTR,
    ) != 0
    {
        while Alias < AliasEnd {
            wprintf(RUSTCONV_WSTR_1CF1ED85F35F.as_ptr(), Indent, Alias);
            Alias = Alias.offset(
                (wcslen(Alias) + 1 as usize) as isize,
            );
        }
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        AliasBuf as PVOID,
    );
}

unsafe extern "C" fn PrintAllMacros() {
    let mut Length: DWORD = GetConsoleAliasExesLength();
    let mut ExeNameBuf: PBYTE = core::ptr::null_mut();
    let mut ExeName: *mut WCHAR = core::ptr::null_mut();
    let mut ExeNameEnd: *mut WCHAR = core::ptr::null_mut();

    ExeNameBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (Length as SIZE_T).wrapping_mul(core::mem::size_of::<BYTE>() as SIZE_T),
    ) as PBYTE;
    if ExeNameBuf.is_null() {
        return;
    }
    ExeName = ExeNameBuf as *mut WCHAR;
    ExeNameEnd = ExeNameBuf.offset(Length as isize) as *mut BYTE as *mut WCHAR;

    if GetConsoleAliasExesW(
        ExeName as LPWSTR,
        Length.wrapping_mul(core::mem::size_of::<BYTE>() as DWORD),
    ) != 0
    {
        while ExeName < ExeNameEnd {
            wprintf(RUSTCONV_WSTR_6C126795E198.as_ptr(), ExeName);
            PrintMacros(
                ExeName as LPWSTR,
                RUSTCONV_WSTR_F27C0439A02B.as_ptr() as *const u16 as LPWSTR,
            );
            wprintf(RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            ExeName = ExeName.offset(
                (wcslen(ExeName as *const WCHAR)
                    .wrapping_add(1 as usize)) as isize,
            );
        }
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ExeNameBuf as PVOID,
    );
}

unsafe extern "C" fn RemoveQuotes(mut str: LPWSTR) -> LPWSTR {
    let mut end: *mut WCHAR = core::ptr::null_mut();
    if *str == 0x0022u16 && {
        end = str.offset(wcslen(str as *const u16) as isize - 1 as isize) as *mut WCHAR;
        *end == 0x0022u16
    } {
        str = str.offset(1);
        *end = 0x0000u16;
    }
    return str;
}

unsafe extern "C" fn ReadFromFile(FileName: LPWSTR) {
    let mut fp: *mut FILE = core::ptr::null_mut();
    let mut line: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ExeNameBuffer: [WCHAR; (EXENAME_LENGTH) as _] = [0; (EXENAME_LENGTH) as _];
    let mut pszOrgExeName: LPWSTR = pszExeName;
    let line_p: *mut WCHAR = core::ptr::addr_of_mut!(line) as *mut WCHAR;

    /* Open the file */
    fp = (_wfopen(FileName, RUSTCONV_WSTR_B85BE71CE3BC.as_ptr())).cast();
    if fp.is_null() {
        _wperror(FileName);
        return;
    }

    loop {
        if fgetws(line_p, ARRAYSIZE!(line) as i32, fp).is_null() {
            break;
        }

        let mut end: PWCHAR;

        if *line_p == 0 {
            continue;
        }

        /* Remove trailing newline character */
        end = line_p.offset((wcslen(line_p) - 1) as isize);
        if *end == 0x000Au16 {
            *end = 0x0000u16;
        }

        if *line_p == 0 {
            continue;
        }

        /* Check for any section redefining the current executable name */
        end = core::ptr::null_mut();
        if *line_p == 0x005Bu16 {
            end = wcschr(line_p, 0x005Du16);
        }

        if !end.is_null() {
            /* New section: change the current executable name */

            *end = 0x0000u16; // NULL-terminate it
            pszExeName = RemoveQuotes(line_p.offset(1 as isize));
            if *pszExeName != 0 {
                /* Capture the new executable name and truncate it if needed */
                end = pszExeName.offset(wcslen(pszExeName as *const WCHAR) as isize);
                if end.offset_from(pszExeName) >= EXENAME_LENGTH as isize {
                    end = pszExeName.offset((EXENAME_LENGTH - 1) as isize);
                }
                *end = 0x0000u16; // Truncate it
                wcscpy(
                    core::ptr::addr_of_mut!(ExeNameBuffer) as *mut WCHAR,
                    pszExeName as *const WCHAR,
                );
                pszExeName = core::ptr::addr_of_mut!(ExeNameBuffer) as *mut WCHAR as LPWSTR;
            } else {
                /* Restore the original current executable name */
                pszExeName = pszOrgExeName;
            }
        } else {
            /* Set the new macro for the current executable */
            SetMacro(line_p as LPWSTR);
        }
    }

    /* Restore the original current executable name if it has changed */
    pszExeName = pszOrgExeName;

    /* Close the file and return */
    fclose(fp);
    return;
}

unsafe extern "C" fn GetArg(pStart: *mut *mut WCHAR, pEnd: *mut *mut WCHAR) -> BOOL {
    let mut bInQuotes: BOOL = (FALSE) as i32;
    let mut p: *mut WCHAR = *pEnd;
    p = p.offset(wcsspn(p, RUSTCONV_WSTR_731D7EC48EEF.as_ptr()) as isize);
    if *p == 0 {
        return (FALSE) as BOOL;
    }
    *pStart = p;
    loop {
        if !(bInQuotes != 0) && (*p == 0x0020u16 || *p == 0x0009u16) {
            break;
        }
        let fresh15 = p;
        p = p.add(1);
        bInQuotes ^= (*fresh15 == 0x0022u16) as BOOL;
        if !(*p != 0) {
            break;
        }
    }
    *pEnd = p;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wmain() -> INT {
    let mut pArgStart: LPWSTR = core::ptr::null_mut();
    let mut pArgEnd: LPWSTR = core::ptr::null_mut();

    setlocale((LC_ALL as i32), b"\0".as_ptr() as *const i8);

    pArgEnd = GetCommandLineW();

    GetArg(core::ptr::addr_of_mut!(pArgStart), core::ptr::addr_of_mut!(pArgEnd));

    while GetArg(core::ptr::addr_of_mut!(pArgStart), core::ptr::addr_of_mut!(pArgEnd)) != 0 {
        let tmp: WCHAR = *pArgEnd;
        *pArgEnd = 0x0000u16;

        if wcscmp(pArgStart, RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0 {
            LoadStringW(
                GetModuleHandle(core::ptr::null_mut()),
                IDS_HELP,
                core::ptr::addr_of_mut!(szStringBuf) as LPWSTR,
                (ARRAYSIZE!(szStringBuf)) as i32,
            );
            wprintf(core::ptr::addr_of_mut!(szStringBuf) as *mut WCHAR);
            break;
        } else if _wcsnicmp(pArgStart, RUSTCONV_WSTR_618611CD599B.as_ptr(), 9) == 0 {
            pszExeName = RemoveQuotes(pArgStart.offset(9 as isize));
        } else if _wcsicmp(pArgStart, RUSTCONV_WSTR_9DF129D38A02.as_ptr()) == 0
            || _wcsicmp(pArgStart, RUSTCONV_WSTR_AFA42E523754.as_ptr()) == 0
        {
            PrintHistory();
        } else if _wcsnicmp(pArgStart, RUSTCONV_WSTR_8BBC8C8BCD07.as_ptr(), 10) == 0 {
            SetConsoleNumberOfCommandsW(
                _wtoi(pArgStart.offset(10 as isize)) as DWORD,
                pszExeName as LPCWSTR,
            );
        } else if _wcsicmp(pArgStart, RUSTCONV_WSTR_ED53584BB2A3.as_ptr()) == 0 {
            ExpungeConsoleCommandHistoryW(pszExeName as LPCWSTR);
        } else if _wcsicmp(pArgStart, RUSTCONV_WSTR_BDDFEBAAEC28.as_ptr()) == 0 {
            SetInsert(ENABLE_INSERT_MODE as DWORD);
        } else if _wcsicmp(pArgStart, RUSTCONV_WSTR_90B77BCB8ED8.as_ptr()) == 0 {
            SetInsert(0 as DWORD);
        } else if _wcsicmp(pArgStart, RUSTCONV_WSTR_EF2E9B46DA39.as_ptr()) == 0
            || _wcsicmp(pArgStart, RUSTCONV_WSTR_1F0448BC599C.as_ptr()) == 0
        {
            PrintMacros(pszExeName, RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR);
        } else if _wcsnicmp(pArgStart, RUSTCONV_WSTR_0EB2C50BC926.as_ptr(), 3) == 0
            || _wcsnicmp(pArgStart, RUSTCONV_WSTR_5A9AC44B1B5A.as_ptr(), 8) == 0
        {
            let exe: LPWSTR = RemoveQuotes(
                wcschr(pArgStart, (0x003Au16 as INT) as u16)
                    .offset(1 as isize),
            );
            if _wcsicmp(exe, RUSTCONV_WSTR_7EF8D2448D12.as_ptr()) == 0 {
                PrintAllMacros();
            } else {
                PrintMacros(exe, RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR);
            }
        } else if _wcsnicmp(pArgStart, RUSTCONV_WSTR_00D5A895B81B.as_ptr(), 11) == 0 {
            ReadFromFile(
                RemoveQuotes(pArgStart.offset(11 as isize)),
            );
        } else {
            *pArgEnd = tmp;
            return SetMacro(pArgStart);
        }

        if tmp == 0 {
            break;
        }
        pArgEnd = pArgEnd.offset(1);
    }

    return 0 as INT;
}
