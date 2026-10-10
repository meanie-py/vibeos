use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut cDateSeparator: TCHAR = 0;

#[no_mangle]
pub static mut cTimeSeparator: TCHAR = 0;

#[no_mangle]
pub static mut cThousandSeparator: TCHAR = 0;

#[no_mangle]
pub static mut cDecimalSeparator: TCHAR = 0;

#[no_mangle]
pub static mut nDateFormat: INT = 0;

#[no_mangle]
pub static mut nTimeFormat: INT = 0;

#[no_mangle]
pub static mut nNumberGroups: INT = 0;
#[no_mangle]
pub unsafe extern "C" fn InitLocale() {
    let mut szBuffer: [TCHAR; 256] = [0; 256];

    /* date settings */
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_SDATE, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer));
    cDateSeparator = szBuffer[0];
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_IDATE | LOCALE_RETURN_NUMBER, core::ptr::addr_of_mut!(nDateFormat) as LPTSTR, core::mem::size_of::<INT>() / (core::mem::size_of::<TCHAR>() as usize));

    /* time settings */
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_STIME, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer));
    cTimeSeparator = szBuffer[0];
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_ITIME | LOCALE_RETURN_NUMBER, core::ptr::addr_of_mut!(nTimeFormat) as LPTSTR, core::mem::size_of::<INT>() / (core::mem::size_of::<TCHAR>() as usize));

    /* number settings */
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_STHOUSAND, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer));
    cThousandSeparator = szBuffer[0];
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer));
    cDecimalSeparator = szBuffer[0];
    GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_SGROUPING, szBuffer.as_mut_ptr(), ARRAYSIZE(szBuffer));
    nNumberGroups = _ttoi(szBuffer.as_mut_ptr()) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn GetDateString() -> LPTSTR {
    static mut szDate: [TCHAR; 32] = [0; 32];
    let mut t: SYSTEMTIME = core::mem::zeroed();

    GetLocalTime(&raw mut t);
    GetDateFormat(
        LOCALE_USER_DEFAULT,
        DATE_SHORTDATE as DWORD,
        &raw mut t,
        core::ptr::null(),
        core::ptr::addr_of_mut!(szDate) as LPTSTR,
        ARRAYSIZE(szDate) as i32,
    );
    return core::ptr::addr_of_mut!(szDate) as LPTSTR;
}
#[no_mangle]
pub unsafe extern "C" fn GetTimeString() -> LPTSTR {
    static mut szTime: [TCHAR; 12] = [0 as TCHAR; 12];
    let mut t: SYSTEMTIME = core::mem::zeroed();
    GetLocalTime(core::ptr::addr_of_mut!(t));
    // _T("%2d%c%02d%c%02d%c%02d") as an explicit NUL-terminated wide array (22 units)
    let FMT: [u16; 22] = [
        '%' as u16, '2' as u16, 'd' as u16,
        '%' as u16, 'c' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'd' as u16,
        '%' as u16, 'c' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'd' as u16,
        '%' as u16, 'c' as u16,
        '%' as u16, '0' as u16, '2' as u16, 'd' as u16,
        0,
    ];
    _stprintf(
        core::ptr::addr_of_mut!(szTime) as LPTSTR,
        FMT.as_ptr() as *const TCHAR,
        t.wHour as i32,
        cTimeSeparator as i32,
        t.wMinute as i32,
        cTimeSeparator as i32,
        t.wSecond as i32,
        cDecimalSeparator as i32,
        (t.wMilliseconds as i32) / 10,
    );
    return core::ptr::addr_of_mut!(szTime) as LPTSTR;
}
