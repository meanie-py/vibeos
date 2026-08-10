// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const SAMPLE_NUMBER: [u16; 10] = [
    0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039,
    0x0000,
];

pub const NUM_SHEETS: u32 = 4;
pub const MAX_FIELD_DIG_SAMPLES: u32 = 3;
pub const MAX_GROUPINGFORMATS: usize = 3;

pub static mut hList: HWND = core::ptr::null_mut();
pub static mut hLocaleList: HWND = core::ptr::null_mut();
pub static mut hGeoList: HWND = core::ptr::null_mut();
pub static mut isSpain: i32 = 0;

#[no_mangle]
pub static mut GroupingFormats: [GROUPINGDATA; MAX_GROUPINGFORMATS] = [
    GROUPINGDATA { nInteger: 0, pszString: unsafe { w!("0;0") as PWSTR } },
    GROUPINGDATA { nInteger: 3, pszString: unsafe { w!("3;0") as PWSTR } },
    GROUPINGDATA { nInteger: 32, pszString: unsafe { w!("3;2;0") as PWSTR } },
];
unsafe extern "system" fn GeneralPropertyPageLocalesEnumProc(lpLocale: LPTSTR) -> i32 {
    let mut lcid: LCID;
    let mut lang: [u16; 255] = [0; 255];
    let mut index: i32;
    let mut bNoShow: i32 = FALSE as BOOL;

    lcid = wcstoul(lpLocale, core::ptr::null_mut(), 16);

    if IsValidLocale(lcid, LCID_INSTALLED) == 0 {
        return TRUE as BOOL;
    }

    if lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
       lcid == MAKELCID!(MAKELANGID!(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT)
    {
        if isSpain == FALSE as BOOL
        {
            LoadStringW(hApplet, IDS_SPAIN, lang.as_mut_ptr(), 255);
            isSpain = TRUE as BOOL;
        }
        else
        {
            bNoShow = TRUE as BOOL;
        }
    }
    else
    {
        GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang.as_mut_ptr(), (core::mem::size_of_val(&lang) / core::mem::size_of::<u16>()) as i32);
    }

    if bNoShow == FALSE as BOOL
    {
        index = SendMessageW(hList,
                             CB_ADDSTRING,
                             0,
                             lang.as_ptr() as LPARAM) as c_int;

        SendMessageW(hList,
                     CB_SETITEMDATA,
                     index as WPARAM,
                     lcid as LPARAM);
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn UpdateLocaleSample(
    hwndDlg: HWND,
    pGlobalData: PGLOBALDATA) {
    let mut OutBuffer: [u16; (MAX_SAMPLES_STR_SIZE) as usize] = [0; (MAX_SAMPLES_STR_SIZE) as usize];
    let mut NumberFormat: NUMBERFMT = core::mem::zeroed();
    let mut CurrencyFormat: CURRENCYFMTW = core::mem::zeroed();

    NumberFormat.NumDigits = ((*pGlobalData).nNumDigits) as u32;
    NumberFormat.LeadingZero = ((*pGlobalData).nNumLeadingZero) as u32;
    NumberFormat.Grouping = GroupingFormats[(*pGlobalData).nNumGrouping as usize].nInteger;
    NumberFormat.lpDecimalSep = (*pGlobalData).szNumDecimalSep.as_mut_ptr();
    NumberFormat.lpThousandSep = (*pGlobalData).szNumThousandSep.as_mut_ptr();
    NumberFormat.NegativeOrder = ((*pGlobalData).nNumNegFormat) as u32;

    CurrencyFormat.NumDigits = ((*pGlobalData).nCurrDigits) as u32;
    CurrencyFormat.LeadingZero = ((*pGlobalData).nNumLeadingZero) as u32;
    CurrencyFormat.Grouping = GroupingFormats[(*pGlobalData).nCurrGrouping as usize].nInteger;
    CurrencyFormat.lpDecimalSep = (*pGlobalData).szCurrDecimalSep.as_mut_ptr();
    CurrencyFormat.lpThousandSep = (*pGlobalData).szCurrThousandSep.as_mut_ptr();
    CurrencyFormat.NegativeOrder = ((*pGlobalData).nCurrNegFormat) as u32;
    CurrencyFormat.PositiveOrder = ((*pGlobalData).nCurrPosFormat) as u32;
    CurrencyFormat.lpCurrencySymbol = (*pGlobalData).szCurrSymbol.as_mut_ptr();

    GetNumberFormatW((*pGlobalData).UserLCID, 0, SAMPLE_NUMBER.as_ptr(),
                     &mut NumberFormat,
                     OutBuffer.as_mut_ptr(), (MAX_SAMPLES_STR_SIZE) as i32);
    SendDlgItemMessageW(hwndDlg, (IDC_NUMSAMPLE_EDIT) as i32,
                        WM_SETTEXT, 0, OutBuffer.as_ptr() as LPARAM);
    ZeroMemory!(OutBuffer.as_mut_ptr() as *mut core::ffi::c_void, MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32);

    GetCurrencyFormatW((*pGlobalData).UserLCID, 0, SAMPLE_NUMBER.as_ptr(),
                       &mut CurrencyFormat,
                       OutBuffer.as_mut_ptr(), (MAX_SAMPLES_STR_SIZE) as i32);
    SendDlgItemMessageW(hwndDlg, (IDC_MONEYSAMPLE_EDIT) as i32,
                        WM_SETTEXT, 0, OutBuffer.as_ptr() as LPARAM);
    ZeroMemory!(OutBuffer.as_mut_ptr() as *mut core::ffi::c_void, MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32);

    GetTimeFormatW((*pGlobalData).UserLCID, 0, core::ptr::null_mut(),
                   (*pGlobalData).szTimeFormat.as_mut_ptr(),
                   OutBuffer.as_mut_ptr(), (MAX_SAMPLES_STR_SIZE) as i32);
    SendDlgItemMessageW(hwndDlg, (IDC_TIMESAMPLE_EDIT) as i32,
                        WM_SETTEXT, 0, OutBuffer.as_ptr() as LPARAM);
    ZeroMemory!(OutBuffer.as_mut_ptr() as *mut core::ffi::c_void, MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32);

    GetDateFormatW((*pGlobalData).UserLCID, 0, core::ptr::null_mut(),
                   (*pGlobalData).szShortDateFormat.as_mut_ptr(),
                   OutBuffer.as_mut_ptr(), (MAX_SAMPLES_STR_SIZE) as i32);
    SendDlgItemMessageW(hwndDlg, (IDC_SHORTTIMESAMPLE_EDIT) as i32,
                        WM_SETTEXT, 0, OutBuffer.as_ptr() as LPARAM);

    GetDateFormatW((*pGlobalData).UserLCID, 0, core::ptr::null_mut(),
                   (*pGlobalData).szLongDateFormat.as_mut_ptr(),
                   OutBuffer.as_mut_ptr(), (MAX_SAMPLES_STR_SIZE) as i32);
    SendDlgItemMessageW(hwndDlg, (IDC_FULLTIMESAMPLE_EDIT) as i32,
                        WM_SETTEXT, 0, OutBuffer.as_ptr() as LPARAM);
}
unsafe extern "C" fn CreateLanguagesList(hwnd: HWND) {
    let mut langSel: [u16; 255] = [0; 255];

    hList = hwnd;
    isSpain = (FALSE) as i32;
    EnumSystemLocalesW(Some(GeneralPropertyPageLocalesEnumProc), LCID_SUPPORTED);

    GetLocaleInfoW(GetUserDefaultLCID(), LOCALE_SLANGUAGE, langSel.as_mut_ptr(), langSel.len() as i32);

    SendMessageW(hList,
                 CB_SELECTSTRING,
                 -1i32 as WPARAM,
                 langSel.as_ptr() as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn LoadCurrentLocale(
    pGlobalData: PGLOBALDATA) -> BOOL {
    let mut szBuffer: [u16; 16] = [0; 16];
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut hLocaleKey: HKEY = core::ptr::null_mut();
    let mut ret: u32;
    let mut dwSize: u32;

    ret = (RegOpenKeyExW(HKEY_CURRENT_USER,
                        w!("Control Panel\\International"),
                        0,
                        KEY_READ,
                        &mut hLocaleKey)) as u32;
    if ret != ERROR_SUCCESS
    {
        PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
        return (FALSE) as i32;
    }

    dwSize = 9 * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("Locale"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     szBuffer.as_mut_ptr() as PBYTE,
                     &mut dwSize);
    (*pGlobalData).UserLCID = wcstoul(szBuffer.as_ptr(), &mut ptr, 16) as LCID;

    dwSize = MAX_NUMDECIMALSEP * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sDecimal"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumDecimalSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_NUMTHOUSANDSEP * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sThousand"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumThousandSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_NUMNEGATIVESIGN * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sNegativeSign"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumNegativeSign.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_NUMPOSITIVESIGN * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sPositiveSign"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumPositiveSign.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_NUMLISTSEP * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sList"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumListSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_NUMNATIVEDIGITS * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sNativeDigits"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szNumNativeDigits.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    (*pGlobalData).nNumNegFormat = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iNegNumber"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumNegFormat = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nNumDigits = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iDigits"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumDigits = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nNumLeadingZero = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iLZero"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumLeadingZero = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nNumMeasure = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iMeasure"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumMeasure = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nNumShape = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("NumShape"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumShape = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nNumGrouping = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("sGrouping"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nNumGrouping = 0;
        if szBuffer[0] == '3' as u16
        {
            if (szBuffer[1] == ';' as u16) &&
                (szBuffer[2] == '2' as u16)
            {
                (*pGlobalData).nNumGrouping = 2;
            }
            else
            {
                (*pGlobalData).nNumGrouping = 1;
            }
        }
    }

    dwSize = MAX_CURRSYMBOL * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sCurrency"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szCurrSymbol.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_CURRDECIMALSEP * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sMonDecimalSep"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szCurrDecimalSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_CURRTHOUSANDSEP * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sMonThousandSep"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szCurrThousandSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    (*pGlobalData).nCurrGrouping = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("sMonGrouping"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nCurrGrouping = 0;
        if szBuffer[0] == '3' as u16
        {
            if (szBuffer[1] == ';' as u16) &&
                (szBuffer[2] == '2' as u16)
            {
                (*pGlobalData).nCurrGrouping = 2;
            }
            else
            {
                (*pGlobalData).nCurrGrouping = 1;
            }
        }
    }

    (*pGlobalData).nCurrPosFormat = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iCurrency"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nCurrPosFormat = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nCurrNegFormat = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iNegCurr"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nCurrNegFormat = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nCurrDigits = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iCurrDigits"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nCurrDigits = _wtoi(szBuffer.as_ptr());
    }

    dwSize = MAX_TIMEFORMAT * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sTimeFormat"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szTimeFormat.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_TIMESEPARATOR * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sTime"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szTimeSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_TIMEAMSYMBOL * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("s1159"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szTimeAM.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_TIMEPMSYMBOL * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("s2359"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szTimePM.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    (*pGlobalData).nTime = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iTime"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nTime = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nTimePrefix = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iTimePrefix"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nTimePrefix = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nTimeLeadingZero = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iTLZero"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nTimeLeadingZero = _wtoi(szBuffer.as_ptr());
    }

    dwSize = MAX_LONGDATEFORMAT * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sLongDate"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szLongDateFormat.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_SHORTDATEFORMAT * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sShortDate"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szShortDateFormat.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_DATESEPARATOR * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sDate"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szDateSep.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    (*pGlobalData).nFirstDayOfWeek = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iFirstDayOfWeek"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nFirstDayOfWeek = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nFirstWeekOfYear = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iFirstWeekOfYear"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nFirstWeekOfYear = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nDate = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iDate"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nDate = _wtoi(szBuffer.as_ptr());
    }

    (*pGlobalData).nCalendarType = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iCalendarType"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nCalendarType = _wtoi(szBuffer.as_ptr());
    }

    dwSize = MAX_MISCCOUNTRY * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sCountry"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szMiscCountry.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    dwSize = MAX_MISCLANGUAGE * core::mem::size_of::<u16>() as u32;
    RegQueryValueExW(hLocaleKey,
                     w!("sLanguage"),
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     (*pGlobalData).szMiscLanguage.as_mut_ptr() as PBYTE,
                     &mut dwSize);

    (*pGlobalData).nMiscCountry = 0;
    dwSize = 16 * core::mem::size_of::<u16>() as u32;
    if RegQueryValueExW(hLocaleKey,
                         w!("iCountry"),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         szBuffer.as_mut_ptr() as PBYTE,
                         &mut dwSize) == (ERROR_SUCCESS) as i32
    {
        (*pGlobalData).nMiscCountry = _wtoi(szBuffer.as_ptr());
    }

    RegCloseKey(hLocaleKey);

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn SetNewLocale(
    pGlobalData: PGLOBALDATA,
    lcid: LCID) {
    let mut szBuffer: [u16; 16] = [0; 16];

    (*pGlobalData).UserLCID = lcid;

    GetLocaleInfo(lcid,
                  LOCALE_SDECIMAL | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumDecimalSep.as_mut_ptr(),
                  (MAX_NUMDECIMALSEP) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_STHOUSAND | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumThousandSep.as_mut_ptr(),
                  (MAX_NUMTHOUSANDSEP) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SNEGATIVESIGN | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumNegativeSign.as_mut_ptr(),
                  (MAX_NUMNEGATIVESIGN) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SPOSITIVESIGN | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumPositiveSign.as_mut_ptr(),
                  (MAX_NUMPOSITIVESIGN) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SLIST | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumListSep.as_mut_ptr(),
                  (MAX_NUMLISTSEP) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SNATIVEDIGITS | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szNumNativeDigits.as_mut_ptr(),
                  (MAX_NUMNATIVEDIGITS) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_INEGNUMBER | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumNegFormat = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_IDIGITS | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumDigits = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_ILZERO | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumLeadingZero = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_IMEASURE | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumMeasure = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_IDIGITSUBSTITUTION | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumShape = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_SGROUPING | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nNumGrouping = 0;
    if szBuffer[0] == 0x0033 {
        if (szBuffer[1] == 0x003B) &&
            (szBuffer[2] == 0x0032) {
            (*pGlobalData).nNumGrouping = 2;
        } else {
            (*pGlobalData).nNumGrouping = 1;
        }
    }

    GetLocaleInfo(lcid,
                  LOCALE_SCURRENCY | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szCurrSymbol.as_mut_ptr(),
                  (MAX_CURRSYMBOL) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SMONDECIMALSEP | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szCurrDecimalSep.as_mut_ptr(),
                  (MAX_CURRDECIMALSEP) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SMONTHOUSANDSEP | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szCurrThousandSep.as_mut_ptr(),
                  (MAX_CURRTHOUSANDSEP) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SMONGROUPING | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nCurrGrouping = 0;
    if szBuffer[0] == 0x0033 {
        if (szBuffer[1] == 0x003B) &&
            (szBuffer[2] == 0x0032) {
            (*pGlobalData).nCurrGrouping = 2;
        } else {
            (*pGlobalData).nCurrGrouping = 1;
        }
    }

    GetLocaleInfo(lcid,
                  LOCALE_ICURRENCY | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nCurrPosFormat = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_INEGCURR | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nCurrNegFormat = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_ICURRDIGITS | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nCurrDigits = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_STIMEFORMAT | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szTimeFormat.as_mut_ptr(),
                  (MAX_TIMEFORMAT) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_STIME | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szTimeSep.as_mut_ptr(),
                  (MAX_TIMESEPARATOR) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_S1159 | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szTimeAM.as_mut_ptr(),
                  (MAX_TIMEAMSYMBOL) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_S2359 | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szTimePM.as_mut_ptr(),
                  (MAX_TIMEPMSYMBOL) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_ITIME | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nTime = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_ITIMEMARKPOSN | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nTimePrefix = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_ITLZERO | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nTimeLeadingZero = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_SLONGDATE | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szLongDateFormat.as_mut_ptr(),
                  (MAX_LONGDATEFORMAT) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SSHORTDATE | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szShortDateFormat.as_mut_ptr(),
                  (MAX_SHORTDATEFORMAT) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SDATE | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szDateSep.as_mut_ptr(),
                  (MAX_DATESEPARATOR) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_IFIRSTDAYOFWEEK | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nFirstDayOfWeek = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_IFIRSTWEEKOFYEAR | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nFirstWeekOfYear = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_IDATE | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nDate = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_ICALENDARTYPE | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nCalendarType = _wtoi(szBuffer.as_ptr());

    GetLocaleInfo(lcid,
                  LOCALE_SCOUNTRY | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szMiscCountry.as_mut_ptr(),
                  (MAX_MISCCOUNTRY) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_SABBREVLANGNAME | LOCALE_NOUSEROVERRIDE,
                  (*pGlobalData).szMiscLanguage.as_mut_ptr(),
                  (MAX_MISCLANGUAGE) as i32);

    GetLocaleInfo(lcid,
                  LOCALE_ICOUNTRY | LOCALE_NOUSEROVERRIDE,
                  szBuffer.as_mut_ptr(),
                  (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);
    (*pGlobalData).nMiscCountry = _wtoi(szBuffer.as_ptr());
}
unsafe extern "C" fn SaveUserLocale(pGlobalData: *mut GLOBALDATA, hLocaleKey: HKEY) {
    let mut szBuffer: [u16; 16] = [0u16; 16];

    wsprintf(szBuffer.as_mut_ptr(), w!("%08lx"), (*pGlobalData).UserLCID as u32);
    RegSetValueExW(
        hLocaleKey,
        w!("Locale"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    /* Number */
    RegSetValueExW(
        hLocaleKey,
        w!("sDecimal"),
        0,
        REG_SZ,
        (*pGlobalData).szNumDecimalSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumDecimalSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sThousand"),
        0,
        REG_SZ,
        (*pGlobalData).szNumThousandSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumThousandSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sNegativeSign"),
        0,
        REG_SZ,
        (*pGlobalData).szNumNegativeSign.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumNegativeSign.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sPositiveSign"),
        0,
        REG_SZ,
        (*pGlobalData).szNumPositiveSign.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumPositiveSign.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sGrouping"),
        0,
        REG_SZ,
        GroupingFormats[(*pGlobalData).nNumGrouping as usize].pszString as *mut u8,
        ((wcslen(GroupingFormats[(*pGlobalData).nNumGrouping as usize].pszString) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sList"),
        0,
        REG_SZ,
        (*pGlobalData).szNumListSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumListSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sNativeDigits"),
        0,
        REG_SZ,
        (*pGlobalData).szNumNativeDigits.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szNumNativeDigits.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nNumNegFormat, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iNegNumber"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nNumDigits, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iDigits"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nNumLeadingZero, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iLZero"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nNumMeasure, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iMeasure"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nNumShape, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("NumShape"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    /* Currency */
    RegSetValueExW(
        hLocaleKey,
        w!("sCurrency"),
        0,
        REG_SZ,
        (*pGlobalData).szCurrSymbol.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szCurrSymbol.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sMonDecimalSep"),
        0,
        REG_SZ,
        (*pGlobalData).szCurrDecimalSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szCurrDecimalSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sMonThousandSep"),
        0,
        REG_SZ,
        (*pGlobalData).szCurrThousandSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szCurrThousandSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sMonGrouping"),
        0,
        REG_SZ,
        GroupingFormats[(*pGlobalData).nCurrGrouping as usize].pszString as *mut u8,
        ((wcslen(GroupingFormats[(*pGlobalData).nCurrGrouping as usize].pszString) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nCurrPosFormat, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iCurrency"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nCurrNegFormat, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iNegCurr"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nCurrDigits, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iCurrDigits"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    /* Time */
    RegSetValueExW(
        hLocaleKey,
        w!("sTimeFormat"),
        0,
        REG_SZ,
        (*pGlobalData).szTimeFormat.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szTimeFormat.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sTime"),
        0,
        REG_SZ,
        (*pGlobalData).szTimeSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szTimeSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("s1159"),
        0,
        REG_SZ,
        (*pGlobalData).szTimeAM.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szTimeAM.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("s2359"),
        0,
        REG_SZ,
        (*pGlobalData).szTimePM.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szTimePM.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nTime, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iTime"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nTimePrefix, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iTimePrefix"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nTimeLeadingZero, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iTLZero"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    /* Date */
    RegSetValueExW(
        hLocaleKey,
        w!("sLongDate"),
        0,
        REG_SZ,
        (*pGlobalData).szLongDateFormat.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szLongDateFormat.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sShortDate"),
        0,
        REG_SZ,
        (*pGlobalData).szShortDateFormat.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szShortDateFormat.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sDate"),
        0,
        REG_SZ,
        (*pGlobalData).szDateSep.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szDateSep.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nFirstDayOfWeek, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iFirstDayOfWeek"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nFirstWeekOfYear, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iFirstWeekOfYear"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nDate, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iDate"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nCalendarType, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iCalendarType"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    /* Misc */
    RegSetValueExW(
        hLocaleKey,
        w!("sCountry"),
        0,
        REG_SZ,
        (*pGlobalData).szMiscCountry.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szMiscCountry.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    RegSetValueExW(
        hLocaleKey,
        w!("sLanguage"),
        0,
        REG_SZ,
        (*pGlobalData).szMiscLanguage.as_mut_ptr() as *mut u8,
        ((wcslen((*pGlobalData).szMiscLanguage.as_ptr()) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );

    _itow((*pGlobalData).nMiscCountry, szBuffer.as_mut_ptr(), (DECIMAL_RADIX) as i32);
    RegSetValueExW(
        hLocaleKey,
        w!("iCountry"),
        0,
        REG_SZ,
        szBuffer.as_mut_ptr() as *mut u8,
        ((wcslen(szBuffer.as_ptr() as *const u16) + 1) * core::mem::size_of::<u16>()) as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SaveCurrentLocale(pGlobalData: *mut GLOBALDATA) {
    let mut hLocaleKey: HKEY = core::ptr::null_mut();
    if (*pGlobalData).bApplyToDefaultUser != 0 {
        let ret = RegOpenKeyExW(
            HKEY_USERS,
            w!(".DEFAULT\\Control Panel\\International"),
            0,
            KEY_WRITE,
            core::ptr::addr_of_mut!(hLocaleKey),
        );
        if ret != (ERROR_SUCCESS) as i32 {
            PrintErrorMsgBox(IDS_ERROR_DEF_INT_KEY_REG);
            return;
        }
        SaveUserLocale(pGlobalData, hLocaleKey);
        RegFlushKey(hLocaleKey);
        RegCloseKey(hLocaleKey);
    }
    let ret = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Control Panel\\International"),
        0,
        KEY_WRITE,
        core::ptr::addr_of_mut!(hLocaleKey),
    );
    if ret != (ERROR_SUCCESS) as i32 {
        PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
        return;
    }
    SaveUserLocale(pGlobalData, hLocaleKey);
    RegFlushKey(hLocaleKey);
    RegCloseKey(hLocaleKey);
    NtSetDefaultLocale((TRUE) as BOOLEAN, (*pGlobalData).UserLCID);
}
#[no_mangle]
pub unsafe extern "system" fn LocationsEnumProc(gId: GEOID) -> BOOL {
    let mut loc: [u16; (MAX_STR_SIZE) as usize] = [0u16; (MAX_STR_SIZE) as usize];
    let index: i32;

    if GetGeoInfoW(gId, GEO_FRIENDLYNAME, loc.as_mut_ptr(), (MAX_STR_SIZE) as i32, (GetThreadLocale()) as LANGID) == 0 {
        return TRUE as BOOL;
    }

    index = SendMessageW(hGeoList, CB_ADDSTRING, 0, loc.as_ptr() as LPARAM) as i32;

    SendMessageW(hGeoList, CB_SETITEMDATA, (index) as WPARAM, gId as LPARAM);

    TRUE as BOOL
}
unsafe extern "C" fn CreateLocationsList(hWnd: HWND) -> GEOID {
    hGeoList = hWnd;
    let mut loc: [u16; (MAX_STR_SIZE) as usize] = [0; (MAX_STR_SIZE) as usize];

    EnumSystemGeoID(
        GEOCLASS_NATION,
        0,
        Some(LocationsEnumProc),
    );

    let userGeoID: GEOID = GetUserGeoID(GEOCLASS_NATION);
    GetGeoInfoW(
        userGeoID,
        GEO_FRIENDLYNAME,
        loc.as_mut_ptr() as PWSTR,
        (MAX_STR_SIZE) as i32,
        (GetThreadLocale()) as LANGID,
    );
    SendMessageW(
        hGeoList,
        CB_SELECTSTRING,
        (-(1 as isize)) as WPARAM,
        loc.as_mut_ptr() as LPARAM,
    );

    userGeoID
}
#[no_mangle]
pub unsafe extern "C" fn SaveGeoID(pGlobalData: PGLOBALDATA) {
    let mut hGeoKey: HKEY;
    let mut value: [u16; 15] = [0u16; 15];
    let len;
    let valuesize: u32;
    let mut ret;

    wsprintf(value.as_mut_ptr(), w!("%lu"), (*pGlobalData).geoid as u32);
    len = wcslen(value.as_ptr());
    valuesize = (len as u32 + 1 as u32) * (core::mem::size_of::<u16>() as u32);

    if (*pGlobalData).bApplyToDefaultUser != 0 {
        hGeoKey = core::ptr::null_mut();
        ret = RegOpenKeyExW(
            HKEY_USERS,
            w!(".DEFAULT\\Control Panel\\International\\Geo"),
            0,
            KEY_WRITE,
            &mut hGeoKey,
        );
        if ret != (ERROR_SUCCESS) as i32 {
            PrintErrorMsgBox(IDS_ERROR_DEF_INT_KEY_REG);
            return;
        }
        ret = RegSetValueExW(
            hGeoKey,
            w!("Nation"),
            0,
            REG_SZ,
            value.as_ptr() as PBYTE,
            valuesize,
        );
        RegFlushKey(hGeoKey);
        RegCloseKey(hGeoKey);
        if ret != (ERROR_SUCCESS) as i32 {
            PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
            return;
        }
    }

    hGeoKey = core::ptr::null_mut();
    ret = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Control Panel\\International\\Geo"),
        0,
        KEY_WRITE,
        &mut hGeoKey,
    );
    if ret != (ERROR_SUCCESS) as i32 {
        PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
        return;
    }
    ret = RegSetValueExW(
        hGeoKey,
        w!("Nation"),
        0,
        REG_SZ,
        value.as_ptr() as PBYTE,
        valuesize,
    );
    RegFlushKey(hGeoKey);
    RegCloseKey(hGeoKey);
    if ret != (ERROR_SUCCESS) as i32 {
        PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn VerifyUnattendLCID(hwndDlg: HWND) -> DWORD {
    let lCount: LRESULT = SendMessage(
        hList,
        CB_GETCOUNT,
        0 as WPARAM,
        0 as LPARAM,
    );
    if lCount == CB_ERR as LRESULT {
        return 0;
    }

    for lIndex in (0 as isize)..(lCount as isize) {
        let lResult: LRESULT = SendMessage(
            hList,
            CB_GETITEMDATA,
            lIndex as WPARAM,
            0 as LPARAM,
        );
        if lResult == CB_ERR as LRESULT {
            continue;
        }

        if lResult == UnattendLCID as LRESULT {
            SendMessage(
                hList,
                CB_SETCURSEL,
                lIndex as WPARAM,
                0 as LPARAM,
            );
            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            return 1;
        }
    }

    0
}
unsafe extern "C" fn InitPropSheetPage(psp: *mut _PROPSHEETPAGEW, idDlg: u16, DlgProc: DLGPROC, pGlobalData: *mut GLOBALDATA) {
    ZeroMemory!(psp, core::mem::size_of::<PROPSHEETPAGEW>());
    (*psp).dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    (*psp).dwFlags = PSP_DEFAULT;
    (*psp).hInstance = hApplet;
    (*psp).__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg);
    (*psp).pfnDlgProc = DlgProc;
    (*psp).lParam = pGlobalData as LPARAM;
}
unsafe extern "system" fn PropSheetProc(
    hwndDlg: HWND,
    uMsg: UINT,
    lParam: LPARAM,
) -> i32 {
    let hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(
                hApplet,
                MAKEINTRESOURCEW!(IDC_CPLICON),
            );
            SendMessageW(
                hwndDlg,
                WM_SETICON,
                ICON_BIG as WPARAM,
                hIcon as LPARAM,
            );
        }
        _ => {}
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CustomizeLocalePropertySheet(hwndDlg: HWND, pGlobalData: *mut GLOBALDATA) -> isize {
    let mut PsPage: [_PROPSHEETPAGEW; (NUM_SHEETS + 1) as usize] = core::mem::zeroed();
    let mut psh: _PROPSHEETHEADERW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADER>());
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwndDlg;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCE!(IDC_CPLICON);
    psh.pszCaption = MAKEINTRESOURCE!(IDS_CUSTOMIZE_TITLE);
    psh.nPages = (core::mem::size_of_val(&PsPage) / core::mem::size_of::<PROPSHEETPAGE>()) as u32 - 1;
    psh.__bindgen_anon_2.nStartPage = 0;
    psh.__bindgen_anon_3.ppsp = PsPage.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    InitPropSheetPage(core::ptr::addr_of_mut!(PsPage[0]), (IDD_NUMBERSPAGE) as u16, Some(NumbersPageProc), pGlobalData);
    InitPropSheetPage(core::ptr::addr_of_mut!(PsPage[1]), (IDD_CURRENCYPAGE) as u16, Some(CurrencyPageProc), pGlobalData);
    InitPropSheetPage(core::ptr::addr_of_mut!(PsPage[2]), (IDD_TIMEPAGE) as u16, Some(TimePageProc), pGlobalData);
    InitPropSheetPage(core::ptr::addr_of_mut!(PsPage[3]), (IDD_DATEPAGE) as u16, Some(DatePageProc), pGlobalData);

    if IsSortPageNeeded((*pGlobalData).UserLCID) != 0 {
        psh.nPages += 1;
        InitPropSheetPage(core::ptr::addr_of_mut!(PsPage[4]), (IDD_SORTPAGE) as u16, Some(SortPageProc), pGlobalData);
    }

    (PropertySheetW(core::ptr::addr_of!(psh))) as isize
}
#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobalData: PGLOBALDATA;
    pGlobalData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PGLOBALDATA;

    'uMsg_block: {
        match uMsg {
            WM_INITDIALOG => {
                pGlobalData = (*(lParam as *mut PROPSHEETPAGE)).lParam as PGLOBALDATA;
                SetWindowLongPtr(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);
                if !pGlobalData.is_null() {
                    LoadCurrentLocale(pGlobalData);
                    CreateLanguagesList(GetDlgItem(hwndDlg, (IDC_LANGUAGELIST) as i32));
                    UpdateLocaleSample(hwndDlg, pGlobalData);
                    (*pGlobalData).geoid = CreateLocationsList(GetDlgItem(hwndDlg, (IDC_LOCATION_COMBO) as i32));
                    if IsUnattendedSetupEnabled != 0 {
                        if VerifyUnattendLCID(hwndDlg) != 0 {
                            SetNewLocale(pGlobalData, UnattendLCID);
                            SaveCurrentLocale(pGlobalData);
                            PostQuitMessage(0);
                        } else {
                            DPRINT1!("VerifyUnattendLCID failed\n");
                        }
                        return TRUE as INT_PTR;
                    }
                }
                break 'uMsg_block;
            }
            WM_COMMAND => {
                let ctrl_id = LOWORD!(wParam);
                match (ctrl_id) as u32 {
                    IDC_LANGUAGELIST => {
                        if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                            let mut NewLcid: LCID;
                            let iCurSel: INT;
                            iCurSel = (SendMessage(hList,
                                                  CB_GETCURSEL,
                                                  0,
                                                  0)) as i32;
                            if iCurSel == CB_ERR {
                                break 'uMsg_block;
                            }
                            NewLcid = (SendMessage(hList,
                                                  CB_GETITEMDATA,
                                                  iCurSel as WPARAM,
                                                  0)) as u32;
                            if NewLcid == CB_ERR as LCID {
                                break 'uMsg_block;
                            }
                            SetNewLocale(pGlobalData, NewLcid);
                            UpdateLocaleSample(hwndDlg, pGlobalData);
                            (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                        break 'uMsg_block;
                    }
                    IDC_LOCATION_COMBO => {
                        if HIWORD!(wParam) == (CBN_SELCHANGE) as u16 {
                            let mut NewGeoID: GEOID;
                            let iCurSel: INT;
                            iCurSel = (SendMessage(GetDlgItem(hwndDlg, (IDC_LOCATION_COMBO) as i32),
                                                  CB_GETCURSEL,
                                                  0,
                                                  0)) as i32;
                            if iCurSel == CB_ERR {
                                break 'uMsg_block;
                            }
                            NewGeoID = (SendMessage(GetDlgItem(hwndDlg, (IDC_LOCATION_COMBO) as i32),
                                                   CB_GETITEMDATA,
                                                   iCurSel as WPARAM,
                                                   0)) as u32;
                            if NewGeoID == CB_ERR as GEOID {
                                break 'uMsg_block;
                            }
                            (*pGlobalData).geoid = NewGeoID;
                            (*pGlobalData).bGeoIdChanged = (TRUE) as i32;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                        break 'uMsg_block;
                    }
                    IDC_SETUP_BUTTON => {
                        if CustomizeLocalePropertySheet(GetParent(hwndDlg), pGlobalData) > 0 {
                            UpdateLocaleSample(hwndDlg, pGlobalData);
                            (*pGlobalData).bUserLocaleChanged = (TRUE) as i32;
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                        }
                        break 'uMsg_block;
                    }
                    _ => {}
                }
                break 'uMsg_block;
            }
            WM_NOTIFY => {
                if (*(lParam as *mut NMHDR)).code == (PSN_APPLY) as u32 {
                    PropSheet_UnChanged!(GetParent(hwndDlg), hwndDlg);
                    if (*pGlobalData).bUserLocaleChanged == (TRUE) as i32 {
                        SaveCurrentLocale(pGlobalData);
                        (*pGlobalData).bUserLocaleChanged = (FALSE) as i32;
                    }
                    if (*pGlobalData).bGeoIdChanged == (TRUE) as i32 {
                        SaveGeoID(pGlobalData);
                        (*pGlobalData).bGeoIdChanged = (FALSE) as i32;
                    }
                    AddNewKbLayoutsByLcid((*pGlobalData).UserLCID);
                    PostMessageW(HWND_BROADCAST, WM_WININICHANGE, 0, w!("intl") as LPARAM);
                }
            }
            _ => {}
        }
    }
    FALSE as INT_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS
//  *  Copyright (C) 2004, 2005 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * PROJECT:         ReactOS International Control Panel
//  * FILE:            dll/cpl/intl/generalp.c
//  * PURPOSE:         General property page
//  * PROGRAMMER:      Eric Kohl
//  *                  Klemens Friedl
//  *                  Aleksey Bragin
//  */
//
// #include "intl.h"
//
// #include <debug.h>
//
// #define SAMPLE_NUMBER   L"123456789"
//
// #define NUM_SHEETS      4
//
// #define MAX_FIELD_DIG_SAMPLES       3
//
//
// HWND hList;
// HWND hLocaleList, hGeoList;
// BOOL isSpain = FALSE;
//
// GROUPINGDATA
// GroupingFormats[MAX_GROUPINGFORMATS] =
// {
//     {0, L"0;0"},
//     {3, L"3;0"},
//     {32, L"3;2;0"}
// };
//
// static BOOL __stdcall
// GeneralPropertyPageLocalesEnumProc(LPTSTR lpLocale)
// {
//     LCID lcid;
//     WCHAR lang[255];
//     INT index;
//     BOOL bNoShow = FALSE;
//
//     lcid = wcstoul(lpLocale, NULL, 16);
//
//     /* Display only languages with installed support */
//     if (!IsValidLocale(lcid, LCID_INSTALLED))
//         return TRUE;
//
//     if (lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH), SORT_DEFAULT) ||
//         lcid == MAKELCID(MAKELANGID(LANG_SPANISH, SUBLANG_SPANISH_MODERN), SORT_DEFAULT))
//     {
//         if (isSpain == FALSE)
//         {
//             LoadStringW(hApplet, IDS_SPAIN, lang, 255);
//             isSpain = TRUE;
//         }
//         else
//         {
//             bNoShow = TRUE;
//         }
//     }
//     else
//     {
//         GetLocaleInfoW(lcid, LOCALE_SLANGUAGE, lang, sizeof(lang)/sizeof(WCHAR));
//     }
//
//     if (bNoShow == FALSE)
//     {
//     index = SendMessageW(hList,
//                          CB_ADDSTRING,
//                          0,
//                          (LPARAM)lang);
//
//     SendMessageW(hList,
//                  CB_SETITEMDATA,
//                  index,
//                  (LPARAM)lcid);
//     }
//
//     return TRUE;
// }
//
//
// /* Update all locale samples */
// static
// VOID
// UpdateLocaleSample(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR OutBuffer[MAX_SAMPLES_STR_SIZE];
//     NUMBERFMT NumberFormat;
//     CURRENCYFMTW CurrencyFormat;
//
//     NumberFormat.NumDigits = pGlobalData->nNumDigits;
//     NumberFormat.LeadingZero = pGlobalData->nNumLeadingZero;
//     NumberFormat.Grouping = GroupingFormats[pGlobalData->nNumGrouping].nInteger;
//     NumberFormat.lpDecimalSep = pGlobalData->szNumDecimalSep;
//     NumberFormat.lpThousandSep = pGlobalData->szNumThousandSep;
//     NumberFormat.NegativeOrder = pGlobalData->nNumNegFormat;
//
//     CurrencyFormat.NumDigits = pGlobalData->nCurrDigits;
//     CurrencyFormat.LeadingZero = pGlobalData->nNumLeadingZero;
//     CurrencyFormat.Grouping = GroupingFormats[pGlobalData->nCurrGrouping].nInteger;
//     CurrencyFormat.lpDecimalSep = pGlobalData->szCurrDecimalSep;
//     CurrencyFormat.lpThousandSep = pGlobalData->szCurrThousandSep;
//     CurrencyFormat.NegativeOrder = pGlobalData->nCurrNegFormat;
//     CurrencyFormat.PositiveOrder = pGlobalData->nCurrPosFormat;
//     CurrencyFormat.lpCurrencySymbol = pGlobalData->szCurrSymbol;
//
//     /* Get number format sample */
//     GetNumberFormatW(pGlobalData->UserLCID, 0, SAMPLE_NUMBER,
//                      &NumberFormat,
//                      OutBuffer, MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_NUMSAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
//     ZeroMemory(OutBuffer, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//
//     /* Get monetary format sample */
//     GetCurrencyFormatW(pGlobalData->UserLCID, 0, SAMPLE_NUMBER,
//                        &CurrencyFormat,
//                        OutBuffer, MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_MONEYSAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
//     ZeroMemory(OutBuffer, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//
//     /* Get time format sample */
//     GetTimeFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szTimeFormat,
//                    OutBuffer, MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_TIMESAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
//     ZeroMemory(OutBuffer, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//
//     /* Get short date format sample */
//     GetDateFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szShortDateFormat,
//                    OutBuffer, MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_SHORTTIMESAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
//
//     /* Get long date sample */
//     GetDateFormatW(pGlobalData->UserLCID, 0, NULL,
//                    pGlobalData->szLongDateFormat,
//                    OutBuffer, MAX_SAMPLES_STR_SIZE);
//     SendDlgItemMessageW(hwndDlg, IDC_FULLTIMESAMPLE_EDIT,
//                         WM_SETTEXT, 0, (LPARAM)OutBuffer);
// }
//
// static VOID
// CreateLanguagesList(HWND hwnd)
// {
//     WCHAR langSel[255];
//
//     hList = hwnd;
//     isSpain = FALSE;
//     EnumSystemLocalesW(GeneralPropertyPageLocalesEnumProc, LCID_SUPPORTED);
//
//     /* Select current locale */
//     /* or should it be System and not user? */
//     GetLocaleInfoW(GetUserDefaultLCID(), LOCALE_SLANGUAGE, langSel, sizeof(langSel)/sizeof(WCHAR));
//
//     SendMessageW(hList,
//                  CB_SELECTSTRING,
//                  -1,
//                  (LPARAM)langSel);
// }
//
//
// BOOL
// LoadCurrentLocale(
//     PGLOBALDATA pGlobalData)
// {
//     WCHAR szBuffer[16];
//     PWSTR ptr;
//     HKEY hLocaleKey;
//     DWORD ret;
//     DWORD dwSize;
//
//     ret = RegOpenKeyExW(HKEY_CURRENT_USER,
//                         L"Control Panel\\International",
//                         0,
//                         KEY_READ,
//                         &hLocaleKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
//         return FALSE;
//     }
//
//     dwSize = 9 * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"Locale",
//                      NULL,
//                      NULL,
//                      (PBYTE)szBuffer,
//                      &dwSize);
//     pGlobalData->UserLCID = (LCID)wcstoul(szBuffer, &ptr, 16);
//
//     /* Number */
//     dwSize = MAX_NUMDECIMALSEP * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sDecimal",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumDecimalSep,
//                      &dwSize);
//
//     dwSize = MAX_NUMTHOUSANDSEP * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sThousand",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumThousandSep,
//                      &dwSize);
//
//     dwSize = MAX_NUMNEGATIVESIGN * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sNegativeSign",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumNegativeSign,
//                      &dwSize);
//
//     dwSize = MAX_NUMPOSITIVESIGN * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sPositiveSign",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumPositiveSign,
//                      &dwSize);
//
//     dwSize = MAX_NUMLISTSEP * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sList",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumListSep,
//                      &dwSize);
//
//     dwSize = MAX_NUMNATIVEDIGITS * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sNativeDigits",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szNumNativeDigits,
//                      &dwSize);
//
//     pGlobalData->nNumNegFormat = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iNegNumber",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nNumNegFormat = _wtoi(szBuffer);
//
//     pGlobalData->nNumDigits = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iDigits",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nNumDigits = _wtoi(szBuffer);
//
//     pGlobalData->nNumLeadingZero = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iLZero",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nNumLeadingZero = _wtoi(szBuffer);
//
//     pGlobalData->nNumMeasure = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iMeasure",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nNumMeasure = _wtoi(szBuffer);
//
//     pGlobalData->nNumShape = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"NumShape",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nNumShape = _wtoi(szBuffer);
//
//     pGlobalData->nNumGrouping = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"sGrouping",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//     {
//         pGlobalData->nNumGrouping = 0;
//         if (szBuffer[0] == L'3')
//         {
//             if ((szBuffer[1] == L';') &&
//                 (szBuffer[2] == L'2'))
//                 pGlobalData->nNumGrouping = 2;
//             else
//                 pGlobalData->nNumGrouping = 1;
//         }
//     }
//
//     /* Currency */
//     dwSize = MAX_CURRSYMBOL * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sCurrency",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szCurrSymbol,
//                      &dwSize);
//
//     dwSize = MAX_CURRDECIMALSEP * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sMonDecimalSep",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szCurrDecimalSep,
//                      &dwSize);
//
//     dwSize = MAX_CURRTHOUSANDSEP * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sMonThousandSep",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szCurrThousandSep,
//                      &dwSize);
//
//     pGlobalData->nCurrGrouping = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"sMonGrouping",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//     {
//         pGlobalData->nCurrGrouping = 0;
//         if (szBuffer[0] == L'3')
//         {
//             if ((szBuffer[1] == L';') &&
//                 (szBuffer[2] == L'2'))
//                 pGlobalData->nCurrGrouping = 2;
//             else
//                 pGlobalData->nCurrGrouping = 1;
//         }
//     }
//
//     pGlobalData->nCurrPosFormat = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iCurrency",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nCurrPosFormat = _wtoi(szBuffer);
//
//     pGlobalData->nCurrNegFormat = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iNegCurr",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nCurrNegFormat = _wtoi(szBuffer);
//
//     pGlobalData->nCurrDigits = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iCurrDigits",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nCurrDigits = _wtoi(szBuffer);
//
//     /* Time */
//     dwSize = MAX_TIMEFORMAT * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sTimeFormat",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szTimeFormat,
//                      &dwSize);
//
//     dwSize = MAX_TIMESEPARATOR * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sTime",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szTimeSep,
//                      &dwSize);
//
//     dwSize = MAX_TIMEAMSYMBOL * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"s1159",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szTimeAM,
//                      &dwSize);
//
//     dwSize = MAX_TIMEPMSYMBOL * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"s2359",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szTimePM,
//                      &dwSize);
//
//     pGlobalData->nTime = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iTime",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nTime = _wtoi(szBuffer);
//
//     pGlobalData->nTimePrefix = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iTimePrefix",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nTimePrefix = _wtoi(szBuffer);
//
//     pGlobalData->nTimeLeadingZero = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iTLZero",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nTimeLeadingZero = _wtoi(szBuffer);
//
//     /* Date */
//     dwSize = MAX_LONGDATEFORMAT * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sLongDate",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szLongDateFormat,
//                      &dwSize);
//
//     dwSize = MAX_SHORTDATEFORMAT * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sShortDate",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szShortDateFormat,
//                      &dwSize);
//
//     dwSize = MAX_DATESEPARATOR * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sDate",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szDateSep,
//                      &dwSize);
//
//     pGlobalData->nFirstDayOfWeek = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iFirstDayOfWeek",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nFirstDayOfWeek = _wtoi(szBuffer);
//
//     pGlobalData->nFirstWeekOfYear = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iFirstWeekOfYear",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nFirstWeekOfYear = _wtoi(szBuffer);
//
//     pGlobalData->nDate = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iDate",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nDate = _wtoi(szBuffer);
//
//     pGlobalData->nCalendarType = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iCalendarType",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nCalendarType = _wtoi(szBuffer);
//
//     /* Misc */
//     dwSize = MAX_MISCCOUNTRY * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sCountry",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szMiscCountry,
//                      &dwSize);
//
//     dwSize = MAX_MISCLANGUAGE * sizeof(WCHAR);
//     RegQueryValueExW(hLocaleKey,
//                      L"sLanguage",
//                      NULL,
//                      NULL,
//                      (PBYTE)pGlobalData->szMiscLanguage,
//                      &dwSize);
//
//     pGlobalData->nMiscCountry = 0;
//     dwSize = 16 * sizeof(WCHAR);
//     if (RegQueryValueExW(hLocaleKey,
//                          L"iCountry",
//                          NULL,
//                          NULL,
//                          (PBYTE)szBuffer,
//                          &dwSize) == ERROR_SUCCESS)
//         pGlobalData->nMiscCountry = _wtoi(szBuffer);
//
//     RegCloseKey(hLocaleKey);
//
//     return TRUE;
// }
//
//
// VOID
// SetNewLocale(
//     PGLOBALDATA pGlobalData,
//     LCID lcid)
// {
//     WCHAR szBuffer[16];
//
//     pGlobalData->UserLCID = lcid;
//
//     /* Number */
//     GetLocaleInfo(lcid,
//                   LOCALE_SDECIMAL | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumDecimalSep,
//                   MAX_NUMDECIMALSEP);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_STHOUSAND | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumThousandSep,
//                   MAX_NUMTHOUSANDSEP);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SNEGATIVESIGN | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumNegativeSign,
//                   MAX_NUMNEGATIVESIGN);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SPOSITIVESIGN | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumPositiveSign,
//                   MAX_NUMPOSITIVESIGN);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SLIST | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumListSep,
//                   MAX_NUMLISTSEP);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SNATIVEDIGITS | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szNumNativeDigits,
//                   MAX_NUMNATIVEDIGITS);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_INEGNUMBER | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumNegFormat = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IDIGITS | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumDigits = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ILZERO | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumLeadingZero = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IMEASURE | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumMeasure = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IDIGITSUBSTITUTION | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumShape = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SGROUPING | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nNumGrouping = 0;
//     if (szBuffer[0] == L'3')
//     {
//         if ((szBuffer[1] == L';') &&
//             (szBuffer[2] == L'2'))
//             pGlobalData->nNumGrouping = 2;
//         else
//             pGlobalData->nNumGrouping = 1;
//     }
//
//     /* Currency */
//     GetLocaleInfo(lcid,
//                   LOCALE_SCURRENCY | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szCurrSymbol,
//                   MAX_CURRSYMBOL);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SMONDECIMALSEP | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szCurrDecimalSep,
//                   MAX_CURRDECIMALSEP);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SMONTHOUSANDSEP | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szCurrThousandSep,
//                   MAX_CURRTHOUSANDSEP);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SMONGROUPING | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nCurrGrouping = 0;
//     if (szBuffer[0] == L'3')
//     {
//         if ((szBuffer[1] == L';') &&
//             (szBuffer[2] == L'2'))
//             pGlobalData->nCurrGrouping = 2;
//         else
//             pGlobalData->nCurrGrouping = 1;
//     }
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ICURRENCY | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nCurrPosFormat = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_INEGCURR | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nCurrNegFormat = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ICURRDIGITS | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nCurrDigits = _wtoi(szBuffer);
//
//     /* Time */
//     GetLocaleInfo(lcid,
//                   LOCALE_STIMEFORMAT | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szTimeFormat,
//                   MAX_TIMEFORMAT);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_STIME | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szTimeSep,
//                   MAX_TIMESEPARATOR);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_S1159 | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szTimeAM,
//                   MAX_TIMEAMSYMBOL);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_S2359 | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szTimePM,
//                   MAX_TIMEPMSYMBOL);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ITIME | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nTime = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ITIMEMARKPOSN | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nTimePrefix = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ITLZERO | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nTimeLeadingZero = _wtoi(szBuffer);
//
//     /* Date */
//     GetLocaleInfo(lcid,
//                   LOCALE_SLONGDATE | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szLongDateFormat,
//                   MAX_LONGDATEFORMAT);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SSHORTDATE | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szShortDateFormat,
//                   MAX_SHORTDATEFORMAT);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SDATE | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szDateSep,
//                   MAX_DATESEPARATOR);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IFIRSTDAYOFWEEK | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nFirstDayOfWeek = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IFIRSTWEEKOFYEAR | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nFirstWeekOfYear = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_IDATE | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nDate = _wtoi(szBuffer);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ICALENDARTYPE | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nCalendarType = _wtoi(szBuffer);
//
//     /* Misc */
//     GetLocaleInfo(lcid,
//                   LOCALE_SCOUNTRY | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szMiscCountry,
//                   MAX_MISCCOUNTRY);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_SABBREVLANGNAME | LOCALE_NOUSEROVERRIDE,
//                   pGlobalData->szMiscLanguage,
//                   MAX_MISCLANGUAGE);
//
//     GetLocaleInfo(lcid,
//                   LOCALE_ICOUNTRY | LOCALE_NOUSEROVERRIDE,
//                   szBuffer,
//                   sizeof(szBuffer) / sizeof(WCHAR));
//     pGlobalData->nMiscCountry = _wtoi(szBuffer);
// }
//
//
// static
// VOID
// SaveUserLocale(
//     PGLOBALDATA pGlobalData,
//     HKEY hLocaleKey)
// {
//     WCHAR szBuffer[16];
//
//     wsprintf(szBuffer, L"%08lx", (DWORD)pGlobalData->UserLCID);
//     RegSetValueExW(hLocaleKey,
//                    L"Locale",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     /* Number */
//     RegSetValueExW(hLocaleKey,
//                    L"sDecimal",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumDecimalSep,
//                    (wcslen(pGlobalData->szNumDecimalSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sThousand",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumThousandSep,
//                    (wcslen(pGlobalData->szNumThousandSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sNegativeSign",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumNegativeSign,
//                    (wcslen(pGlobalData->szNumNegativeSign) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sPositiveSign",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumPositiveSign,
//                    (wcslen(pGlobalData->szNumPositiveSign) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sGrouping",
//                    0,
//                    REG_SZ,
//                    (PBYTE)GroupingFormats[pGlobalData->nNumGrouping].pszString,
//                    (wcslen(GroupingFormats[pGlobalData->nNumGrouping].pszString) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sList",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumListSep,
//                    (wcslen(pGlobalData->szNumListSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sNativeDigits",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szNumNativeDigits,
//                    (wcslen(pGlobalData->szNumNativeDigits) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nNumNegFormat,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iNegNumber",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nNumDigits,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iDigits",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nNumLeadingZero,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iLZero",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nNumMeasure,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iMeasure",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nNumShape,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"NumShape",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     /* Currency */
//     RegSetValueExW(hLocaleKey,
//                    L"sCurrency",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szCurrSymbol,
//                    (wcslen(pGlobalData->szCurrSymbol) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sMonDecimalSep",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szCurrDecimalSep,
//                    (wcslen(pGlobalData->szCurrDecimalSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sMonThousandSep",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szCurrThousandSep,
//                    (wcslen(pGlobalData->szCurrThousandSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sMonGrouping",
//                    0,
//                    REG_SZ,
//                    (PBYTE)GroupingFormats[pGlobalData->nCurrGrouping].pszString,
//                    (wcslen(GroupingFormats[pGlobalData->nCurrGrouping].pszString) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nCurrPosFormat,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iCurrency",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nCurrNegFormat,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iNegCurr",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nCurrDigits,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iCurrDigits",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     /* Time */
//     RegSetValueExW(hLocaleKey,
//                    L"sTimeFormat",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szTimeFormat,
//                    (wcslen(pGlobalData->szTimeFormat) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sTime",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szTimeSep,
//                    (wcslen(pGlobalData->szTimeSep) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"s1159",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szTimeAM,
//                    (wcslen(pGlobalData->szTimeAM) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"s2359",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szTimePM,
//                    (wcslen(pGlobalData->szTimePM) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nTime,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iTime",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nTimePrefix,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iTimePrefix",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nTimeLeadingZero,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iTLZero",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     /* Date */
//     RegSetValueExW(hLocaleKey,
//                    L"sLongDate",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szLongDateFormat,
//                    (wcslen(pGlobalData->szLongDateFormat) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sShortDate",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szShortDateFormat,
//                    (wcslen(pGlobalData->szShortDateFormat) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sDate",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szDateSep,
//                    (wcslen(pGlobalData->szDateSep) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nFirstDayOfWeek,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iFirstDayOfWeek",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nFirstWeekOfYear,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iFirstWeekOfYear",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nDate,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iDate",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nCalendarType,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iCalendarType",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
//
//     /* Misc */
//     RegSetValueExW(hLocaleKey,
//                    L"sCountry",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szMiscCountry,
//                    (wcslen(pGlobalData->szMiscCountry) + 1) * sizeof(WCHAR));
//
//     RegSetValueExW(hLocaleKey,
//                    L"sLanguage",
//                    0,
//                    REG_SZ,
//                    (PBYTE)pGlobalData->szMiscLanguage,
//                    (wcslen(pGlobalData->szMiscLanguage) + 1) * sizeof(WCHAR));
//
//     _itow(pGlobalData->nMiscCountry,
//           szBuffer, DECIMAL_RADIX);
//     RegSetValueExW(hLocaleKey,
//                    L"iCountry",
//                    0,
//                    REG_SZ,
//                    (PBYTE)szBuffer,
//                    (wcslen(szBuffer) + 1) * sizeof(WCHAR));
// }
//
//
// /* Sets new locale */
// VOID
// SaveCurrentLocale(
//     PGLOBALDATA pGlobalData)
// {
//     HKEY hLocaleKey;
//     DWORD ret;
//
//     if (pGlobalData->bApplyToDefaultUser)
//     {
//         ret = RegOpenKeyExW(HKEY_USERS,
//                             L".DEFAULT\\Control Panel\\International",
//                             0,
//                             KEY_WRITE,
//                             &hLocaleKey);
//         if (ret != ERROR_SUCCESS)
//         {
//             PrintErrorMsgBox(IDS_ERROR_DEF_INT_KEY_REG);
//             return;
//         }
//
//         SaveUserLocale(pGlobalData, hLocaleKey);
//
//         /* Flush and close the locale key */
//         RegFlushKey(hLocaleKey);
//         RegCloseKey(hLocaleKey);
//     }
//
//     ret = RegOpenKeyExW(HKEY_CURRENT_USER,
//                         L"Control Panel\\International",
//                         0,
//                         KEY_WRITE,
//                         &hLocaleKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
//         return;
//     }
//
//     SaveUserLocale(pGlobalData, hLocaleKey);
//
//     /* Flush and close the locale key */
//     RegFlushKey(hLocaleKey);
//     RegCloseKey(hLocaleKey);
//
//     /* Set the new locale for the current process */
//     NtSetDefaultLocale(TRUE, pGlobalData->UserLCID);
// }
//
// /* Location enumerate procedure */
// BOOL
// __stdcall
// LocationsEnumProc(GEOID gId)
// {
//     WCHAR loc[MAX_STR_SIZE];
//     INT index;
//
//     if (GetGeoInfoW(gId, GEO_FRIENDLYNAME, loc, MAX_STR_SIZE, GetThreadLocale()) == 0)
//         return TRUE;
//
//     index = (INT)SendMessageW(hGeoList,
//                               CB_ADDSTRING,
//                               0,
//                               (LPARAM)loc);
//
//     SendMessageW(hGeoList,
//                  CB_SETITEMDATA,
//                  index,
//                  (LPARAM)gId);
//
//     return TRUE;
// }
//
// /* Enumerate all system locations identifiers */
// static
// GEOID
// CreateLocationsList(HWND hWnd)
// {
//     GEOID userGeoID;
//     WCHAR loc[MAX_STR_SIZE];
//
//     hGeoList = hWnd;
//
//     EnumSystemGeoID(GEOCLASS_NATION, 0, LocationsEnumProc);
//
//     /* Select current location */
//     userGeoID = GetUserGeoID(GEOCLASS_NATION);
//     GetGeoInfoW(userGeoID,
//                 GEO_FRIENDLYNAME,
//                 loc,
//                 MAX_STR_SIZE,
//                 GetThreadLocale());
//
//     SendMessageW(hGeoList,
//                  CB_SELECTSTRING,
//                  (WPARAM) -1,
//                  (LPARAM)loc);
//
//     return userGeoID;
// }
//
// VOID
// SaveGeoID(
//     PGLOBALDATA pGlobalData)
// {
//     HKEY hGeoKey;
//     WCHAR value[15];
//     DWORD valuesize;
//     DWORD ret;
//
//     wsprintf(value, L"%lu", (DWORD)pGlobalData->geoid);
//     valuesize = (wcslen(value) + 1) * sizeof(WCHAR);
//
//     if (pGlobalData->bApplyToDefaultUser)
//     {
//         ret = RegOpenKeyExW(HKEY_USERS,
//                             L".DEFAULT\\Control Panel\\International\\Geo",
//                             0,
//                             KEY_WRITE,
//                             &hGeoKey);
//         if (ret != ERROR_SUCCESS)
//         {
//             PrintErrorMsgBox(IDS_ERROR_DEF_INT_KEY_REG);
//             return;
//         }
//
//         ret = RegSetValueExW(hGeoKey,
//                              L"Nation",
//                              0,
//                              REG_SZ,
//                              (PBYTE)value,
//                              valuesize);
//
//         RegFlushKey(hGeoKey);
//         RegCloseKey(hGeoKey);
//
//         if (ret != ERROR_SUCCESS)
//         {
//             PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
//             return;
//         }
//     }
//
//     ret = RegOpenKeyExW(HKEY_CURRENT_USER,
//                         L"Control Panel\\International\\Geo",
//                         0,
//                         KEY_WRITE,
//                         &hGeoKey);
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
//         return;
//     }
//
//     ret = RegSetValueExW(hGeoKey,
//                          L"Nation",
//                          0,
//                          REG_SZ,
//                          (PBYTE)value,
//                          valuesize);
//
//     RegFlushKey(hGeoKey);
//     RegCloseKey(hGeoKey);
//
//     if (ret != ERROR_SUCCESS)
//     {
//         PrintErrorMsgBox(IDS_ERROR_INT_KEY_REG);
//         return;
//     }
// }
//
// DWORD
// VerifyUnattendLCID(HWND hwndDlg)
// {
//     LRESULT lCount, lIndex, lResult;
//
//     lCount = SendMessage(hList, CB_GETCOUNT, (WPARAM)0, (LPARAM)0);
//     if (lCount == CB_ERR)
//     {
//         return 0;
//     }
//
//     for (lIndex = 0; lIndex < lCount; lIndex++)
//     {
//         lResult = SendMessage(hList, CB_GETITEMDATA, (WPARAM)lIndex, (LPARAM)0);
//         if (lResult == CB_ERR)
//         {
//             continue;
//         }
//
//         if (lResult == (LRESULT)UnattendLCID)
//         {
//             SendMessage(hList, CB_SETCURSEL, (WPARAM)lIndex, (LPARAM)0);
//             PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//             return 1;
//         }
//     }
//
//     return 0;
// }
//
//
// static
// VOID
// InitPropSheetPage(
//     PROPSHEETPAGEW *psp,
//     WORD idDlg,
//     DLGPROC DlgProc,
//     PGLOBALDATA pGlobalData)
// {
//     ZeroMemory(psp, sizeof(PROPSHEETPAGEW));
//     psp->dwSize = sizeof(PROPSHEETPAGEW);
//     psp->dwFlags = PSP_DEFAULT;
//     psp->hInstance = hApplet;
//     psp->pszTemplate = MAKEINTRESOURCE(idDlg);
//     psp->pfnDlgProc = DlgProc;
//     psp->lParam = (LPARAM)pGlobalData;
// }
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDC_CPLICON));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// INT_PTR
// __stdcall
// CustomizeLocalePropertySheet(
//     HWND hwndDlg,
//     PGLOBALDATA pGlobalData)
// {
//     PROPSHEETPAGEW PsPage[NUM_SHEETS + 1];
//     PROPSHEETHEADERW psh;
//
//     ZeroMemory(&psh, sizeof(PROPSHEETHEADER));
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags =  PSH_PROPSHEETPAGE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwndDlg;
//     psh.hInstance = hApplet;
//     psh.pszIcon = MAKEINTRESOURCE(IDC_CPLICON);
//     psh.pszCaption = MAKEINTRESOURCE(IDS_CUSTOMIZE_TITLE);
//     psh.nPages = (sizeof(PsPage) / sizeof(PROPSHEETPAGE)) - 1;
//     psh.nStartPage = 0;
//     psh.ppsp = PsPage;
//     psh.pfnCallback = PropSheetProc;
//
//     InitPropSheetPage(&PsPage[0], IDD_NUMBERSPAGE, NumbersPageProc, pGlobalData);
//     InitPropSheetPage(&PsPage[1], IDD_CURRENCYPAGE, CurrencyPageProc, pGlobalData);
//     InitPropSheetPage(&PsPage[2], IDD_TIMEPAGE, TimePageProc, pGlobalData);
//     InitPropSheetPage(&PsPage[3], IDD_DATEPAGE, DatePageProc, pGlobalData);
//
//     if (IsSortPageNeeded(pGlobalData->UserLCID))
//     {
//         psh.nPages++;
//         InitPropSheetPage(&PsPage[4], IDD_SORTPAGE, SortPageProc, pGlobalData);
//     }
//
//     return PropertySheetW(&psh);
// }
//
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// GeneralPageProc(HWND hwndDlg,
//                 UINT uMsg,
//                 WPARAM wParam,
//                 LPARAM lParam)
// {
//     PGLOBALDATA pGlobalData;
//
//     pGlobalData = (PGLOBALDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pGlobalData = (PGLOBALDATA)((LPPROPSHEETPAGE)lParam)->lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pGlobalData);
//
//             if (pGlobalData)
//             {
//                 LoadCurrentLocale(pGlobalData);
//
//                 CreateLanguagesList(GetDlgItem(hwndDlg, IDC_LANGUAGELIST));
//                 UpdateLocaleSample(hwndDlg, pGlobalData);
//                 pGlobalData->geoid = CreateLocationsList(GetDlgItem(hwndDlg, IDC_LOCATION_COMBO));
//                 if (IsUnattendedSetupEnabled)
//                 {
//                     if (VerifyUnattendLCID(hwndDlg))
//                     {
//                         SetNewLocale(pGlobalData, UnattendLCID);
//                         SaveCurrentLocale(pGlobalData);
//                         PostQuitMessage(0);
//                     }
//                     else
//                     {
//                         DPRINT1("VerifyUnattendLCID failed\n");
//                     }
//                     return TRUE;
//                 }
//             }
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_LANGUAGELIST:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         LCID NewLcid;
//                         INT iCurSel;
//
//                         iCurSel = SendMessage(hList,
//                                               CB_GETCURSEL,
//                                               0,
//                                               0);
//                         if (iCurSel == CB_ERR)
//                             break;
//
//                         NewLcid = SendMessage(hList,
//                                               CB_GETITEMDATA,
//                                               iCurSel,
//                                               0);
//                         if (NewLcid == (LCID)CB_ERR)
//                             break;
//
//                         SetNewLocale(pGlobalData, NewLcid);
//                         UpdateLocaleSample(hwndDlg, pGlobalData);
//                         pGlobalData->bUserLocaleChanged = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//
//                 case IDC_LOCATION_COMBO:
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                     {
//                         GEOID NewGeoID;
//                         INT iCurSel;
//
//                         iCurSel = SendMessage(GetDlgItem(hwndDlg, IDC_LOCATION_COMBO),
//                                               CB_GETCURSEL,
//                                               0,
//                                               0);
//                         if (iCurSel == CB_ERR)
//                             break;
//
//                         NewGeoID = SendMessage(GetDlgItem(hwndDlg, IDC_LOCATION_COMBO),
//                                                CB_GETITEMDATA,
//                                                iCurSel,
//                                                0);
//                         if (NewGeoID == (GEOID)CB_ERR)
//                             break;
//
//                         pGlobalData->geoid = NewGeoID;
//                         pGlobalData->bGeoIdChanged = TRUE;
//
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//
//                 case IDC_SETUP_BUTTON:
//                     if (CustomizeLocalePropertySheet(GetParent(hwndDlg), pGlobalData) > 0)
//                     {
//                         UpdateLocaleSample(hwndDlg, pGlobalData);
//                         pGlobalData->bUserLocaleChanged = TRUE;
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     }
//                     break;
//             }
//             break;
//
//         case WM_NOTIFY:
//             if (((LPNMHDR)lParam)->code == (UINT)PSN_APPLY)
//             {
//                 /* Apply changes */
//                 PropSheet_UnChanged(GetParent(hwndDlg), hwndDlg);
//
//                 /* Set new locale */
//                 if (pGlobalData->bUserLocaleChanged == TRUE)
//                 {
//                     SaveCurrentLocale(pGlobalData);
//                     pGlobalData->bUserLocaleChanged = FALSE;
//                 }
//
//                 /* Set new GEO ID */
//                 if (pGlobalData->bGeoIdChanged == TRUE)
//                 {
//                     SaveGeoID(pGlobalData);
//                     pGlobalData->bGeoIdChanged = FALSE;
//                 }
//
//                 AddNewKbLayoutsByLcid(pGlobalData->UserLCID);
//
//                 /* Post WM_WININICHANGE messages to system */
//                 PostMessageW(HWND_BROADCAST, WM_WININICHANGE, 0, (LPARAM)L"intl");
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
