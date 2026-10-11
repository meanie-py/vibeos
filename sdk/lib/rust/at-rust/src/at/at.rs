// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0E18B58FA673: [u16; 3] = [0x0061u16, 0x006D, 0x0000];
pub static RUSTCONV_WSTR_4437D105C3C0: [u16; 4] = [0x0020u16, 0x0025, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_59A543594151: [u16; 4] = [0x002Eu16, 0x002E, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_7E7219CF4FF7: [u16; 8] = [0x002Fu16, 0x0064, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_A2D398922901: [u16; 2] = [0x0061u16, 0x0000];
pub static RUSTCONV_WSTR_B437F67EB717: [u16; 13] = [
    0x002Fu16, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0061, 0x0063, 0x0074, 0x0069,
    0x0076, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_BACA8E3CF044: [u16; 8] = [0x002Fu16, 0x0065, 0x0076, 0x0065, 0x0072, 0x0079, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_C0A3A6D4344E: [u16; 3] = [0x0070u16, 0x006D, 0x0000];
pub static RUSTCONV_WSTR_D3FE97979D0F: [u16; 2] = [0x0070u16, 0x0000];
pub static RUSTCONV_WSTR_D7ABED73C2D5: [u16; 30] = [
    0x0020u16, 0x0020, 0x0020, 0x0025, 0x0036, 0x006C, 0x0075, 0x0020, 0x0020, 0x0020,
    0x0025, 0x002D, 0x0032, 0x0031, 0x0073, 0x0020, 0x0020, 0x0020, 0x0025, 0x002D,
    0x0031, 0x0031, 0x0073, 0x0020, 0x0020, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_DEEFE3BC3DD3: [u16; 4] = [0x0020u16, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_E29263B3FB5F: [u16; 7] = [0x002Fu16, 0x006E, 0x0065, 0x0078, 0x0074, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_E61F882486BE: [u16; 5] = [0x002Fu16, 0x0079, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut pszDaysOfWeekArray: [PWSTR; 7] = [
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    ::core::ptr::null::<WCHAR>() as *mut WCHAR,
];
unsafe extern "C" fn FreeDaysOfWeekArray() {
    let mut i: WORD = 0 as WORD;
    while (i as ::core::ffi::c_int) < 7 {
        if !pszDaysOfWeekArray[i as usize].is_null() {
            HeapFree(GetProcessHeap(), 0 as DWORD, pszDaysOfWeekArray[i as usize] as PVOID);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn InitDaysOfWeekArray() -> BOOL {
    let mut i: WORD = 0;
    i = 0 as WORD;
    while (i as ::core::ffi::c_int) < 7 as ::core::ffi::c_int {
        let mut nLength: INT = 0;
        nLength = GetLocaleInfo(
            LOCALE_USER_DEFAULT as LCID,
            (LOCALE_SABBREVDAYNAME1 as ::core::ffi::c_int + i as ::core::ffi::c_int) as LCTYPE,
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        pszDaysOfWeekArray[i as usize] = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (nLength as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if pszDaysOfWeekArray[i as usize].is_null() {
            FreeDaysOfWeekArray();
            return FALSE as BOOL;
        }
        GetLocaleInfo(
            LOCALE_USER_DEFAULT as LCID,
            (LOCALE_SABBREVDAYNAME1 as ::core::ffi::c_int + i as ::core::ffi::c_int) as LCTYPE,
            pszDaysOfWeekArray[i as usize],
            nLength as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn ParseTime(
    mut pszTime: PWSTR,
    mut pulJobHour: PULONG,
    mut pulJobMinute: PULONG,
) -> BOOL {
    let mut szHour: [WCHAR; 3] = [0; 3];
    let mut szMinute: [WCHAR; 3] = [0; 3];
    let mut szAmPm: [WCHAR; 5] = [0; 5];
    let mut startPtr: PWSTR = core::ptr::null_mut();
    let mut endPtr: PWSTR = core::ptr::null_mut();
    let mut ulHour: ULONG = 0;
    let mut ulMinute: ULONG = 0;
    let mut nLength: UINT;

    if pszTime.is_null() {
        return (FALSE) as BOOL;
    }

    startPtr = pszTime;

    /* Extract the hour string */
    nLength = 0;
    while *startPtr != 0x0000u16 && iswdigit(*startPtr) != 0 {
        if nLength >= 2 {
            return (FALSE) as BOOL;
        }

        szHour[nLength as usize] = *startPtr;
        nLength += 1;

        startPtr = startPtr.offset(1);
    }
    szHour[nLength as usize] = 0x0000u16;

    /* Check for a valid time separator */
    if *startPtr != 0x003Au16 {
        return (FALSE) as BOOL;
    }

    /* Skip the time separator */
    startPtr = startPtr.offset(1);

    /* Extract the minute string */
    nLength = 0;
    while *startPtr != 0x0000u16 && iswdigit(*startPtr) != 0 {
        if nLength >= 2 {
            return (FALSE) as BOOL;
        }

        szMinute[nLength as usize] = *startPtr;
        nLength += 1;

        startPtr = startPtr.offset(1);
    }
    szMinute[nLength as usize] = 0x0000u16;

    /* Extract the optional AM/PM indicator string */
    nLength = 0;
    while *startPtr != 0x0000u16 {
        if nLength >= 4 {
            return (FALSE) as BOOL;
        }

        if iswspace(*startPtr) == 0 {
            szAmPm[nLength as usize] = *startPtr;
            nLength += 1;
        }

        startPtr = startPtr.offset(1);
    }
    szAmPm[nLength as usize] = 0x0000u16;

    /* Convert the hour string */
    ulHour = wcstoul(
        szHour.as_ptr(),
        core::ptr::addr_of_mut!(endPtr),
        10,
    ) as ULONG;
    if ulHour == 0 && *endPtr != UNICODE_NULL {
        return (FALSE) as BOOL;
    }

    /* Convert the minute string */
    ulMinute = wcstoul(
        szMinute.as_ptr(),
        core::ptr::addr_of_mut!(endPtr),
        10,
    ) as ULONG;
    if ulMinute == 0 && *endPtr != UNICODE_NULL {
        return (FALSE) as BOOL;
    }

    /* Check for valid AM/PM indicator */
    if wcslen(szAmPm.as_ptr()) > 0
        && _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_A2D398922901.as_ptr()) != 0
        && _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_0E18B58FA673.as_ptr()) != 0
        && _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_D3FE97979D0F.as_ptr()) != 0
        && _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_C0A3A6D4344E.as_ptr()) != 0
    {
        return (FALSE) as BOOL;
    }

    /* Check for the valid minute range [0-59] */
    if ulMinute > 59 {
        return (FALSE) as BOOL;
    }

    if wcslen(szAmPm.as_ptr()) > 0 {
        /* 12 hour time format */

        /* Check for the valid hour range [1-12] */
        if ulHour == 0 || ulHour > 12 {
            return (FALSE) as BOOL;
        }

        /* Convert 12 hour format to 24 hour format */
        if _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_A2D398922901.as_ptr()) == 0
            || _wcsicmp(szAmPm.as_ptr(), RUSTCONV_WSTR_0E18B58FA673.as_ptr()) == 0
        {
            if ulHour == 12 {
                ulHour = 0;
            }
        } else {
            if ulHour >= 1 && ulHour <= 11 {
                ulHour += 12;
            }
        }
    } else {
        /* 24 hour time format */

        /* Check for the valid hour range [0-23] */
        if ulHour > 23 {
            return (FALSE) as BOOL;
        }
    }

    if !pulJobHour.is_null() {
        *pulJobHour = ulHour;
    }

    if !pulJobMinute.is_null() {
        *pulJobMinute = ulMinute;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ParseId(pszId: PWSTR, pulId: PULONG) -> BOOL {
    let mut startPtr: PWSTR = core::ptr::null_mut();
    let mut endPtr: PWSTR = core::ptr::null_mut();
    let mut ulId: ULONG = 0 as ULONG;
    let mut bResult: BOOL = FALSE as BOOL;

    startPtr = pszId;
    endPtr = core::ptr::null_mut();
    ulId = wcstoul(startPtr as *const u16, core::ptr::addr_of_mut!(endPtr), 10 as core::ffi::c_int) as ULONG;
    if !endPtr.is_null() && *endPtr == UNICODE_NULL as WCHAR {
        bResult = TRUE as BOOL;

        if !pulId.is_null() {
            *pulId = ulId;
        }
    }

    return bResult;
}

unsafe extern "C" fn ParseDaysOfMonth(mut pszBuffer: PWSTR, mut pulDaysOfMonth: PULONG) -> BOOL {
    let mut startPtr: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut endPtr: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ulValue: ULONG = 0;
    if wcslen(pszBuffer as *const wchar_t) == 0 as _ {
        return (FALSE) as BOOL;
    }
    startPtr = pszBuffer;
    endPtr = ::core::ptr::null_mut::<WCHAR>();
    loop {
        ulValue = wcstoul(
            startPtr as *const wchar_t,
            &raw mut endPtr,
            10 as ::core::ffi::c_int,
        ) as ULONG;
        if ulValue == 0 as ULONG {
            return (FALSE) as BOOL;
        }
        if ulValue > 0 as ULONG && ulValue <= 31 as ULONG {
            *pulDaysOfMonth |=
                ((1 as ::core::ffi::c_int) << (ulValue - 1 as ULONG)) as ULONG;
        }
        if !endPtr.is_null() && *endPtr as ::core::ffi::c_int == UNICODE_NULL as ::core::ffi::c_int {
            return (TRUE) as BOOL;
        }
        startPtr = endPtr.offset(1 as isize);
        endPtr = ::core::ptr::null_mut::<WCHAR>();
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn ParseDaysOfWeek(mut pszBuffer: PWSTR, mut pucDaysOfWeek: PUCHAR) -> BOOL {
    let mut startPtr: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut endPtr: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut nLength: size_t = 0;
    let mut i: WORD = 0;
    if wcslen(pszBuffer as *const WCHAR) == 0 {
        return (FALSE) as BOOL;
    }
    startPtr = pszBuffer;
    endPtr = ::core::ptr::null_mut::<WCHAR>();
    loop {
        endPtr = wcschr(startPtr as *const WCHAR, 0x002Cu16) as PWSTR;
        if endPtr.is_null() {
            nLength = wcslen(startPtr as *const WCHAR) as size_t;
        } else {
            nLength = ((endPtr as usize).wrapping_sub(startPtr as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>())) as size_t;
        }
        i = 0;
        while i < 7 {
            if nLength == wcslen(pszDaysOfWeekArray[i as usize] as *const WCHAR) as size_t
                && _wcsnicmp(
                    startPtr as *const WCHAR,
                    pszDaysOfWeekArray[i as usize] as *const WCHAR,
                    nLength,
                ) == 0
            {
                *pucDaysOfWeek |= ((1 << i) as UCHAR);
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if endPtr.is_null() {
            return (TRUE) as BOOL;
        }
        startPtr = endPtr.offset(1);
        endPtr = ::core::ptr::null_mut::<WCHAR>();
    }
    #[allow(unreachable_code)]
    {
        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn PrintErrorMessage(dwError: DWORD) {
    ConMsgPuts(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut(),
        dwError,
        0 as DWORD,
    );
}

unsafe extern "C" fn PrintHorizontalLine() {
    let mut szBuffer: [WCHAR; 80] = [0; 80];
    let mut i: WORD = 0;
    i = 0 as WORD;
    while (i as usize) < (79 as usize) {
        szBuffer[i as usize] = 0x002Du16;
        i = i.wrapping_add(1);
    }
    szBuffer[79 as usize] = UNICODE_NULL;

    ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR);
}

unsafe extern "C" fn Confirm() -> BOOL {
    let mut szYesBuffer: [WCHAR; 8] = [0; 8];
    let mut szNoBuffer: [WCHAR; 8] = [0; 8];
    let mut szInput: [WCHAR; 80] = [0; 80];
    let mut dwOldMode: DWORD = 0;
    let mut dwRead: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);
    let mut hFile: HANDLE = core::ptr::null_mut();

    LoadStringW(
        core::ptr::null_mut(),
        IDS_CONFIRM_YES,
        szYesBuffer.as_mut_ptr(),
        (_countof!(szYesBuffer)) as i32,
    );
    LoadStringW(
        core::ptr::null_mut(),
        IDS_CONFIRM_NO,
        szNoBuffer.as_mut_ptr(),
        (_countof!(szNoBuffer)) as i32,
    );

    ZeroMemory!(
        szInput.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 80]>(),
    );

    hFile = GetStdHandle(STD_INPUT_HANDLE);
    GetConsoleMode(hFile, core::ptr::addr_of_mut!(dwOldMode));

    SetConsoleMode(hFile, ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);

    loop {
        ConResPrintf(StdOut, IDS_CONFIRM_QUESTION);

        ReadConsoleW(
            hFile,
            (szInput.as_mut_ptr()).cast(),
            _countof!(szInput),
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::null_mut(),
        );

        szInput[0 as usize] =
            towupper((szInput[0 as usize]) as _) as WCHAR;
        if szInput[0 as usize] == szYesBuffer[0 as usize]
        {
            ret = (TRUE as i32);
            break;
        } else if szInput[0 as usize] == 13
            || szInput[0 as usize] == szNoBuffer[0 as usize]
        {
            ret = (FALSE as i32);
            break;
        }

        ConResPrintf(StdOut, IDS_CONFIRM_INVALID);
    }

    SetConsoleMode(hFile, dwOldMode);

    return ret;
}

unsafe extern "C" fn GetTimeAsJobTime() -> DWORD_PTR {
    let mut Time: SYSTEMTIME = core::mem::zeroed();
    let mut JobTime: DWORD_PTR = 0;
    GetLocalTime(core::ptr::addr_of_mut!(Time));
    JobTime = (Time.wHour as DWORD_PTR).wrapping_mul(3600000).wrapping_add((Time.wMinute as DWORD_PTR).wrapping_mul(60000));
    return JobTime;
}

unsafe extern "C" fn GetCurrentDayOfMonth() -> ULONG {
    let mut Time: SYSTEMTIME = core::mem::zeroed();

    GetLocalTime(core::ptr::addr_of_mut!(Time));

    return 1 << (Time.wDay - 1);
}

unsafe extern "C" fn JobTimeToTimeString(
    pszBuffer: PWSTR,
    cchBuffer: INT,
    wHour: WORD,
    wMinute: WORD,
) {
    let mut Time: SYSTEMTIME = SYSTEMTIME {
        wYear: 0 as WORD,
        wMonth: 0 as WORD,
        wDayOfWeek: 0 as WORD,
        wDay: 0 as WORD,
        wHour: 0 as WORD,
        wMinute: 0 as WORD,
        wSecond: 0 as WORD,
        wMilliseconds: 0 as WORD,
    };

    Time.wHour = wHour;
    Time.wMinute = wMinute;

    GetTimeFormat(
        LOCALE_USER_DEFAULT as DWORD,
        TIME_NOSECONDS as DWORD,
        core::ptr::addr_of_mut!(Time),
        core::ptr::null(),
        pszBuffer,
        cchBuffer,
    );
}

unsafe extern "C" fn PrintJobDetails(pszComputerName: PWSTR, ulJobId: ULONG) -> INT {
    let mut pBuffer: PAT_INFO = core::ptr::null_mut();
    let mut CurrentTime: DWORD_PTR = 0;
    let mut szStatusBuffer: [WCHAR; 16] = [0; 16];
    let mut szScheduleBuffer: [WCHAR; 60] = [0; 60];
    let mut szTimeBuffer: [WCHAR; 16] = [0; 16];
    let mut szInteractiveBuffer: [WCHAR; 16] = [0; 16];
    let mut szDateBuffer: [WCHAR; 8] = [0; 8];
    let mut nDateLength: size_t = 0;
    let mut nScheduleLength: size_t = 0;
    let mut i: WORD = 0;
    let mut Status: NET_API_STATUS;

    Status = NetScheduleJobGetInfo(
        pszComputerName,
        ulJobId,
        core::ptr::addr_of_mut!(pBuffer) as *mut PBYTE,
    );
    if Status != NERR_Success {
        PrintErrorMessage(Status);
        return 1;
    }

    if ((*pBuffer).Flags & (JOB_EXEC_ERROR as u8)) != 0 {
        LoadStringW(
            core::ptr::null_mut(),
            IDS_ERROR,
            core::ptr::addr_of_mut!(szStatusBuffer) as LPWSTR,
            _countof!(szStatusBuffer) as INT,
        );
    } else {
        LoadStringW(
            core::ptr::null_mut(),
            IDS_OK,
            core::ptr::addr_of_mut!(szStatusBuffer) as LPWSTR,
            _countof!(szStatusBuffer) as INT,
        );
    }

    if (*pBuffer).DaysOfMonth != 0 {
        if ((*pBuffer).Flags & (JOB_RUN_PERIODICALLY as u8)) != 0 {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_EVERY,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        } else {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_NEXT,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        }

        nScheduleLength = wcslen(core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR) as size_t;
        i = 0;
        while i < 31 {
            if ((*pBuffer).DaysOfMonth & ((1 as DWORD) << i)) != 0 {
                _swprintf(
                    core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR,
                    RUSTCONV_WSTR_4437D105C3C0.as_ptr(),
                    (i as ::core::ffi::c_int + 1) as ::core::ffi::c_int,
                );
                nDateLength = wcslen(core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR) as size_t;
                if nScheduleLength + nDateLength <= 55 {
                    wcscat(
                        core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
                        core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR,
                    );
                    nScheduleLength += nDateLength;
                } else {
                    wcscat(
                        core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
                        RUSTCONV_WSTR_59A543594151.as_ptr(),
                    );
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    } else if (*pBuffer).DaysOfWeek != 0 {
        if ((*pBuffer).Flags & (JOB_RUN_PERIODICALLY as u8)) != 0 {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_EVERY,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        } else {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_NEXT,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        }

        nScheduleLength = wcslen(core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR) as size_t;
        i = 0;
        while i < 7 {
            if (((*pBuffer).DaysOfWeek as DWORD) & ((1 as DWORD) << i)) != 0 {
                _swprintf(
                    core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR,
                    RUSTCONV_WSTR_DEEFE3BC3DD3.as_ptr(),
                    pszDaysOfWeekArray[i as usize],
                );
                nDateLength = wcslen(core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR) as size_t;
                if nScheduleLength + nDateLength <= 55 {
                    wcscat(
                        core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
                        core::ptr::addr_of_mut!(szDateBuffer) as *mut WCHAR,
                    );
                    nScheduleLength += nDateLength;
                } else {
                    wcscat(
                        core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
                        RUSTCONV_WSTR_59A543594151.as_ptr(),
                    );
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    } else {
        CurrentTime = GetTimeAsJobTime();
        if CurrentTime > (*pBuffer).JobTime as DWORD_PTR {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_TOMORROW,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        } else {
            LoadStringW(
                core::ptr::null_mut(),
                IDS_TODAY,
                core::ptr::addr_of_mut!(szScheduleBuffer) as LPWSTR,
                _countof!(szScheduleBuffer) as INT,
            );
        }
    }

    JobTimeToTimeString(
        core::ptr::addr_of_mut!(szTimeBuffer) as PWSTR,
        _countof!(szTimeBuffer) as INT,
        ((*pBuffer).JobTime / 3600000) as WORD,
        (((*pBuffer).JobTime % 3600000) / 60000) as WORD,
    );

    if ((*pBuffer).Flags & (JOB_NONINTERACTIVE as u8)) != 0 {
        LoadStringW(
            core::ptr::null_mut(),
            IDS_NO,
            core::ptr::addr_of_mut!(szInteractiveBuffer) as LPWSTR,
            _countof!(szInteractiveBuffer) as INT,
        );
    } else {
        LoadStringW(
            core::ptr::null_mut(),
            IDS_YES,
            core::ptr::addr_of_mut!(szInteractiveBuffer) as LPWSTR,
            _countof!(szInteractiveBuffer) as INT,
        );
    }

    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_TASKID,
        ulJobId,
    );
    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_STATUS,
        core::ptr::addr_of_mut!(szStatusBuffer) as *mut WCHAR,
    );
    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_SCHEDULE,
        core::ptr::addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
    );
    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_TIME,
        core::ptr::addr_of_mut!(szTimeBuffer) as *mut WCHAR,
    );
    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_INTERACTIVE,
        core::ptr::addr_of_mut!(szInteractiveBuffer) as *mut WCHAR,
    );
    ConResPrintf(
        core::ptr::addr_of_mut!(csStdOut),
        IDS_COMMAND,
        (*pBuffer).Command,
    );

    NetApiBufferFree(pBuffer as PVOID);

    return 0;
}

unsafe extern "C" fn PrintAllJobs(pszComputerName: PWSTR) -> INT {
    let mut pBuffer: PAT_ENUM = core::ptr::null_mut();
    let mut dwRead: DWORD = 0 as DWORD;
    let mut dwTotal: DWORD = 0 as DWORD;
    let mut dwResume: DWORD = 0 as DWORD;
    let mut i: DWORD;
    let mut CurrentTime: DWORD_PTR;
    let mut Status: NET_API_STATUS;
    let mut szScheduleBuffer: [WCHAR; 32 as usize] = [0; 32 as usize];
    let mut szTimeBuffer: [WCHAR; 16 as usize] = [0; 16 as usize];
    let mut szDateBuffer: [WCHAR; 8 as usize] = [0; 8 as usize];
    let mut nDateLength: size_t;
    let mut nScheduleLength: size_t;
    let mut j: WORD;

    Status = NetScheduleJobEnum(
        pszComputerName,
        &raw mut pBuffer as *mut PBYTE,
        MAX_PREFERRED_LENGTH as DWORD,
        &raw mut dwRead,
        &raw mut dwTotal,
        &raw mut dwResume,
    );
    if Status != NERR_Success as NET_API_STATUS {
        PrintErrorMessage(Status);
        return 1 as INT;
    }

    if dwTotal == 0 as DWORD {
        ConResPrintf(&raw mut csStdOut, IDS_NO_ENTRIES as UINT);
        return 0 as INT;
    }

    ConResPrintf(&raw mut csStdOut, IDS_JOBS_LIST as UINT);
    PrintHorizontalLine();

    i = 0 as DWORD;
    while i < dwRead {
        if (*pBuffer.add(i as usize)).DaysOfMonth != 0 as DWORD {
            if ((*pBuffer.add(i as usize)).Flags as INT & JOB_RUN_PERIODICALLY as INT) != 0 {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_EVERY as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            } else {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_NEXT as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            }

            nScheduleLength = wcslen(addr_of_mut!(szScheduleBuffer) as *mut WCHAR) as size_t;
            j = 0 as WORD;
            while (j as INT) < 31 as INT {
                if ((*pBuffer.add(i as usize)).DaysOfMonth
                    & ((1 as INT) << (j as INT)) as DWORD)
                    != 0
                {
                    _swprintf(
                        addr_of_mut!(szDateBuffer) as *mut wchar_t,
                        RUSTCONV_WSTR_4437D105C3C0.as_ptr(),
                        j as INT + 1 as INT,
                    );
                    nDateLength = wcslen(addr_of_mut!(szDateBuffer) as *mut WCHAR) as size_t;
                    if nScheduleLength + nDateLength <= 19 as size_t {
                        wcscat(
                            addr_of_mut!(szScheduleBuffer) as *mut wchar_t,
                            addr_of_mut!(szDateBuffer) as *mut WCHAR,
                        );
                        nScheduleLength += nDateLength;
                    } else {
                        wcscat(
                            addr_of_mut!(szScheduleBuffer) as *mut wchar_t,
                            RUSTCONV_WSTR_59A543594151.as_ptr(),
                        );
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
        } else if (*pBuffer.add(i as usize)).DaysOfWeek as INT != 0 as INT {
            if ((*pBuffer.add(i as usize)).Flags as INT & JOB_RUN_PERIODICALLY as INT) != 0 {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_EVERY as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            } else {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_NEXT as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            }

            nScheduleLength = wcslen(addr_of_mut!(szScheduleBuffer) as *mut WCHAR) as size_t;
            j = 0 as WORD;
            while (j as INT) < 7 as INT {
                if ((*pBuffer.add(i as usize)).DaysOfWeek as INT
                    & ((1 as INT) << (j as INT)))
                    != 0
                {
                    _swprintf(
                        addr_of_mut!(szDateBuffer) as *mut wchar_t,
                        RUSTCONV_WSTR_DEEFE3BC3DD3.as_ptr(),
                        pszDaysOfWeekArray[j as usize],
                    );
                    nDateLength = wcslen(addr_of_mut!(szDateBuffer) as *mut WCHAR) as size_t;
                    if nScheduleLength + nDateLength <= 55 as size_t {
                        wcscat(
                            addr_of_mut!(szScheduleBuffer) as *mut wchar_t,
                            addr_of_mut!(szDateBuffer) as *mut WCHAR,
                        );
                        nScheduleLength += nDateLength;
                    } else {
                        wcscat(
                            addr_of_mut!(szScheduleBuffer) as *mut wchar_t,
                            RUSTCONV_WSTR_59A543594151.as_ptr(),
                        );
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
        } else {
            CurrentTime = GetTimeAsJobTime();
            if CurrentTime > (*pBuffer.add(i as usize)).JobTime as DWORD_PTR {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_TOMORROW as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            } else {
                LoadStringW(
                    core::ptr::null_mut(),
                    IDS_TODAY as UINT,
                    addr_of_mut!(szScheduleBuffer) as LPWSTR,
                    (_countof!(szScheduleBuffer)) as i32,
                );
            }
        }

        JobTimeToTimeString(
            addr_of_mut!(szTimeBuffer) as PWSTR,
            (_countof!(szTimeBuffer)) as INT,
            ((*pBuffer.add(i as usize)).JobTime / (3600000 as DWORD)) as WORD,
            (((*pBuffer.add(i as usize)).JobTime % (3600000 as DWORD))
                / (60000 as DWORD)) as WORD,
        );

        ConPrintf(
            &raw mut csStdOut,
            RUSTCONV_WSTR_D7ABED73C2D5.as_ptr(),
            (*pBuffer.add(i as usize)).JobId,
            addr_of_mut!(szScheduleBuffer) as *mut WCHAR,
            addr_of_mut!(szTimeBuffer) as *mut WCHAR,
            (*pBuffer.add(i as usize)).Command,
        );

        i = i.wrapping_add(1);
    }

    NetApiBufferFree(pBuffer as PVOID);

    return 0 as INT;
}

unsafe extern "C" fn AddJob(
    mut pszComputerName: PWSTR,
    mut ulJobHour: ULONG,
    mut ulJobMinute: ULONG,
    mut ulDaysOfMonth: ULONG,
    mut ucDaysOfWeek: UCHAR,
    mut bInteractiveJob: BOOL,
    mut bPeriodicJob: BOOL,
    mut pszCommand: PWSTR,
) -> INT {
    let mut InfoBuffer: AT_INFO = core::mem::zeroed();
    let mut ulJobId: ULONG = 0 as ULONG;
    let mut Status: DWORD;

    InfoBuffer.JobTime = (ulJobHour as DWORD_PTR)
        .wrapping_mul(3600000 as DWORD_PTR)
        .wrapping_add((ulJobMinute as DWORD_PTR).wrapping_mul(60000 as DWORD_PTR))
        as DWORD;
    InfoBuffer.DaysOfMonth = ulDaysOfMonth as DWORD;
    InfoBuffer.DaysOfWeek = ucDaysOfWeek;
    InfoBuffer.Flags = ((if bInteractiveJob != 0 {
        0
    } else {
        JOB_NONINTERACTIVE
    }) | (if bPeriodicJob != 0 {
        JOB_RUN_PERIODICALLY
    } else {
        0
    })) as UCHAR;
    InfoBuffer.Command = pszCommand as LPWSTR;

    Status = NetScheduleJobAdd(
        pszComputerName as LPWSTR,
        &raw mut InfoBuffer as PBYTE,
        &raw mut ulJobId,
    );
    if Status != NERR_Success as DWORD {
        PrintErrorMessage(Status);
        return 1 as INT;
    }

    ConResPrintf(StdOut, IDS_NEW_JOB as UINT, ulJobId);

    return 0 as INT;
}

unsafe extern "C" fn DeleteJob(
    mut pszComputerName: PWSTR,
    mut ulJobId: ULONG,
    mut bForceDelete: BOOL,
) -> INT {
    let mut Status: NET_API_STATUS;

    if ulJobId == (1 as u32).wrapping_neg() as ULONG && bForceDelete == (FALSE as i32) {
        ConResPrintf(StdOut, IDS_DELETE_ALL);
        if Confirm() == 0 {
            return 0;
        }
    }

    Status = NetScheduleJobDel(
        pszComputerName,
        if ulJobId == (1 as u32).wrapping_neg() as ULONG {
            0
        } else {
            ulJobId
        },
        if ulJobId == (1 as u32).wrapping_neg() as ULONG {
            (1 as u32).wrapping_neg() as ULONG
        } else {
            ulJobId
        },
    );
    if Status != NERR_Success {
        PrintErrorMessage(Status);
        return 1;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut pszComputerName: PWSTR = core::ptr::null_mut();
    let mut pszCommand: PWSTR = core::ptr::null_mut();
    let mut ulJobId: ULONG = (-(1 as i32)) as ULONG;
    let mut ulJobHour: ULONG = (-(1 as i32)) as ULONG;
    let mut ulJobMinute: ULONG = (-(1 as i32)) as ULONG;
    let mut bDeleteJob: BOOL = (FALSE as i32);
    let mut bForceDelete: BOOL = (FALSE as i32);
    let mut bInteractiveJob: BOOL = (FALSE as i32);
    let mut bPeriodicJob: BOOL = (FALSE as i32);
    let mut bPrintUsage: BOOL = (FALSE as i32);
    let mut ulDaysOfMonth: ULONG = 0;
    let mut ucDaysOfWeek: UCHAR = 0;
    let mut nResult: INT = 0;
    let mut i: INT = 0;
    let mut minIdx: INT = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if InitDaysOfWeekArray() == 0 {
        return 1;
    }

    /* Parse the computer name */
    i = 1;
    minIdx = 1;
    if i < argc
        && *(*argv.offset(i as isize)).offset(0 as isize) == 0x005Cu16
        && *(*argv.offset(i as isize)).offset(1 as isize) == 0x005Cu16
    {
        pszComputerName = *argv.offset(i as isize);
        i += 1;
        minIdx += 1;
    }

    /* Parse the time or job id */
    if i < argc && *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16
    {
        if ParseTime(
            *argv.offset(i as isize),
            core::ptr::addr_of_mut!(ulJobHour),
            core::ptr::addr_of_mut!(ulJobMinute),
        ) != 0
        {
            i += 1;
            minIdx += 1;
        } else if ParseId(*argv.offset(i as isize), core::ptr::addr_of_mut!(ulJobId)) != 0 {
            i += 1;
            minIdx += 1;
        }
    }

    'done: {
        /* Parse the options */
        while i < argc {
            if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16 {
                if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0 {
                    bPrintUsage = (TRUE as i32);
                    break 'done;
                } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_7E7219CF4FF7.as_ptr()) == 0 {
                    bDeleteJob = (TRUE as i32);
                } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_E61F882486BE.as_ptr()) == 0 {
                    bForceDelete = (TRUE as i32);
                } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_B437F67EB717.as_ptr()) == 0 {
                    bInteractiveJob = (TRUE as i32);
                } else if _wcsnicmp(
                    *argv.offset(i as isize),
                    RUSTCONV_WSTR_BACA8E3CF044.as_ptr(),
                    7,
                ) == 0
                {
                    bPeriodicJob = (TRUE as i32);
                    if ParseDaysOfMonth(
                        (*argv.offset(i as isize)).offset(7 as isize),
                        core::ptr::addr_of_mut!(ulDaysOfMonth),
                    ) == (FALSE as i32)
                    {
                        if ParseDaysOfWeek(
                            (*argv.offset(i as isize)).offset(7 as isize),
                            core::ptr::addr_of_mut!(ucDaysOfWeek),
                        ) == (FALSE as i32)
                        {
                            ulDaysOfMonth = GetCurrentDayOfMonth();
                        }
                    }
                } else if _wcsnicmp(
                    *argv.offset(i as isize),
                    RUSTCONV_WSTR_E29263B3FB5F.as_ptr(),
                    6,
                ) == 0
                {
                    bPeriodicJob = (FALSE as i32);
                    if ParseDaysOfMonth(
                        (*argv.offset(i as isize)).offset(6 as isize),
                        core::ptr::addr_of_mut!(ulDaysOfMonth),
                    ) == (FALSE as i32)
                    {
                        if ParseDaysOfWeek(
                            (*argv.offset(i as isize)).offset(6 as isize),
                            core::ptr::addr_of_mut!(ucDaysOfWeek),
                        ) == (FALSE as i32)
                        {
                            ulDaysOfMonth = GetCurrentDayOfMonth();
                        }
                    }
                } else {
                    bPrintUsage = (TRUE as i32);
                    nResult = 1;
                    break 'done;
                }
            }
            i += 1;
        }

        /* Parse the command */
        if argc > minIdx
            && *(*argv.offset((argc - 1) as isize)).offset(0 as isize)
                != 0x002Fu16
        {
            pszCommand = *argv.offset((argc - 1) as isize);
        }

        if bDeleteJob == (TRUE as i32) {
            /* Check for invalid options or arguments */
            if bInteractiveJob == (TRUE as i32)
                || ulJobHour != (-(1 as i32)) as ULONG
                || ulJobMinute != (-(1 as i32)) as ULONG
                || ulDaysOfMonth != 0
                || ucDaysOfWeek != 0
                || !pszCommand.is_null()
            {
                bPrintUsage = (TRUE as i32);
                nResult = 1;
                break 'done;
            }

            nResult = DeleteJob(pszComputerName, ulJobId, bForceDelete);
        } else {
            if ulJobHour != (-(1 as i32)) as ULONG
                && ulJobMinute != (-(1 as i32)) as ULONG
            {
                /* Check for invalid options or arguments */
                if bForceDelete == (TRUE as i32) || pszCommand.is_null() {
                    bPrintUsage = (TRUE as i32);
                    nResult = 1;
                    break 'done;
                }

                nResult = AddJob(
                    pszComputerName,
                    ulJobHour,
                    ulJobMinute,
                    ulDaysOfMonth,
                    ucDaysOfWeek,
                    bInteractiveJob,
                    bPeriodicJob,
                    pszCommand,
                );
            } else {
                /* Check for invalid options or arguments */
                if bForceDelete == (TRUE as i32)
                    || bInteractiveJob == (TRUE as i32)
                    || ulDaysOfMonth != 0
                    || ucDaysOfWeek != 0
                    || !pszCommand.is_null()
                {
                    bPrintUsage = (TRUE as i32);
                    nResult = 1;
                    break 'done;
                }

                if ulJobId == (-(1 as i32)) as ULONG {
                    nResult = PrintAllJobs(pszComputerName);
                } else {
                    nResult = PrintJobDetails(pszComputerName, ulJobId);
                }
            }
        }
    }

    FreeDaysOfWeekArray();

    if bPrintUsage == (TRUE as i32) {
        ConResPuts(StdOut, IDS_USAGE);
    }

    return nResult;
}
