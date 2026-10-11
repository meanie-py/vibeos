use crate::prelude::*;
use crate::msctf::utils_cpp_h::g_cs;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_464EEE2E93B7: [WCHAR; 10] = [0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0049, 0x0064, 0x0000];
pub static CPP_COM_WSTR_6F4B49BAF187: [WCHAR; 12] = [0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_BFDA085919A5: [WCHAR; 28] = [0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x005C, 0x0053, 0x0075, 0x0062, 0x0073, 0x0074, 0x0069, 0x0074, 0x0075, 0x0074, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_D7727E89129A: [WCHAR; 50] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_ED80A4D01B91: [WCHAR; 56] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0020, 0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0073, 0x005C, 0x0025, 0x0030, 0x0038, 0x006C, 0x0058, 0x0000];
pub static CPP_COM_WSTR_FCC5F82B6F58: [WCHAR; 20] = [0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0020, 0x0044, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_pMlngInfo: *mut CicArray<MLNGINFO> = core::ptr::null_mut();
pub static mut CStaticIconList__s_cx: i32 = 0;
pub static mut CStaticIconList__s_cy: i32 = 0;
static mut s_hCacheKL: HKL = core::ptr::null_mut();
static mut s_dwCacheKLID: u32 = 0;











#[no_mangle]
pub unsafe extern "C" fn GetSpecialKLID(hKL: HKL) -> DWORD {
    assert!(IS_SPECIAL_HKL!(hKL));

    if s_hCacheKL == hKL && s_dwCacheKLID != 0 {
        return s_dwCacheKLID;
    }

    s_dwCacheKLID = 0;

    let mut regKey1: CicRegKey = core::mem::zeroed();
    let mut error: LSTATUS = regKey1.Open(HKEY_LOCAL_MACHINE, CPP_COM_WSTR_D7727E89129A.as_ptr(), KEY_READ);
    if (error) as u32 != ERROR_SUCCESS {
        return 0;
    }

    let mut szName = [0u16; 16];
    let mut szLayoutId = [0u16; 16];
    let dwSpecialId: DWORD = SPECIALIDFROMHKL!(hKL) as DWORD;
    let mut dwIndex: DWORD = 0;
    'outer: loop {
        'body: {
            error = RegEnumKeyW(regKey1.m_hKey, dwIndex, szName.as_mut_ptr(), _countof!(szName) as DWORD);
            szName[(_countof!(szName) - 1) as usize] = UNICODE_NULL; // Avoid buffer overrun
            if (error) as u32 != ERROR_SUCCESS {
                break 'outer;
            }

            let mut regKey2: CicRegKey = core::mem::zeroed();
            error = regKey2.Open(regKey1.m_hKey, szName.as_ptr(), KEY_READ);
            if (error) as u32 != ERROR_SUCCESS {
                break 'outer;
            }

            error = regKey2.QuerySz(CPP_COM_WSTR_464EEE2E93B7.as_ptr(), szLayoutId.as_mut_ptr(), _countof!(szLayoutId));
            szLayoutId[(_countof!(szLayoutId) - 1) as usize] = UNICODE_NULL; // Avoid buffer overrun
            // The C++ says `if (error == ERROR_SUCCESS) continue;`: it skips exactly the keys whose
            // "Layout Id" it read, and compares the unread buffer for the rest, so no special
            // HKL is ever found. Skip the keys WITHOUT a readable Layout Id.
            if (error) as u32 != ERROR_SUCCESS {
                break 'body;
            }

            let dwLayoutId: DWORD = wcstoul(szLayoutId.as_ptr(), core::ptr::null_mut(), 16);
            if dwLayoutId == dwSpecialId {
                s_hCacheKL = hKL;
                s_dwCacheKLID = wcstoul(szName.as_ptr(), core::ptr::null_mut(), 16);
                break 'outer;
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    return s_dwCacheKLID;
}
#[no_mangle]
pub unsafe extern "C" fn GetHKLSubstitute(hKL: HKL) -> DWORD {
    if IS_IME_HKL!(hKL) {
        return HandleToUlong!(hKL);
    }

    let mut dwKLID: DWORD;
    if HIWORD!(hKL) == LOWORD!(hKL) {
        dwKLID = (LOWORD!(hKL)) as u32;
    } else if IS_SPECIAL_HKL!(hKL) {
        dwKLID = GetSpecialKLID(hKL);
    } else {
        dwKLID = HandleToUlong!(hKL);
    }

    if dwKLID == 0 {
        return HandleToUlong!(hKL);
    }

    let mut regKey: CicRegKey = core::mem::zeroed();
    let mut error: LSTATUS = regKey.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_BFDA085919A5.as_ptr(), KEY_READ);
    if error == ERROR_SUCCESS as LSTATUS {
        let mut szName = [0u16; (MAX_PATH) as _];
        let mut szValue = [0u16; (MAX_PATH) as _];
        let mut dwIndex: DWORD;
        let mut dwValue: DWORD;
        dwIndex = 0;
        loop {
            error = regKey.EnumValue(dwIndex, szName.as_mut_ptr(), _countof!(szName));
            szName[(_countof!(szName) - 1) as usize] = UNICODE_NULL; // Avoid buffer overrun
            if error != ERROR_SUCCESS as LSTATUS {
                break;
            }

            error = regKey.QuerySz(szName.as_ptr(), szValue.as_mut_ptr(), _countof!(szValue));
            szValue[(_countof!(szValue) - 1) as usize] = UNICODE_NULL; // Avoid buffer overrun
            if error != ERROR_SUCCESS as LSTATUS {
                break;
            }

            dwValue = wcstoul(szValue.as_mut_ptr(), core::ptr::null_mut(), 16);
            if (dwKLID & !SPECIAL_MASK) == dwValue {
                dwKLID = wcstoul(szName.as_mut_ptr(), core::ptr::null_mut(), 16);
                break;
            }
            dwIndex = dwIndex.wrapping_add(1);
        }
    }

    dwKLID
}

unsafe extern "C" fn GetKbdLayoutNameFromReg(hKL: HKL, pszDesc: LPWSTR, cchDesc: UINT) -> BOOL {
    let dwKLID: DWORD = GetHKLSubstitute(hKL);

    let mut szSubKey: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    StringCchPrintfW!(szSubKey.as_mut_ptr(), _countof!(szSubKey),
                     CPP_COM_WSTR_ED80A4D01B91.as_ptr(),
                     dwKLID);

    let mut regKey: CicRegKey = core::mem::zeroed();
    let mut error: LSTATUS = regKey.Open(HKEY_LOCAL_MACHINE, szSubKey.as_ptr(), KEY_READ);
    if (error) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    if SHLoadRegUIStringW(regKey.m_hKey, CPP_COM_WSTR_FCC5F82B6F58.as_ptr(), pszDesc, cchDesc) == S_OK
    {
        *pszDesc.add((cchDesc - 1) as usize) = UNICODE_NULL; // Avoid buffer overrun
        return (TRUE) as BOOL;
    }

    error = regKey.QuerySz(CPP_COM_WSTR_6F4B49BAF187.as_ptr(), pszDesc, cchDesc);
    *pszDesc.add((cchDesc - 1) as usize) = UNICODE_NULL; // Avoid buffer overrun
    ((error) as u32 == ERROR_SUCCESS) as BOOL
}

unsafe extern "C" fn GetHKLName(hKL: HKL, pszDesc: LPWSTR, cchDesc: UINT) -> BOOL {
    if GetLocaleInfoW(LOWORD!(hKL) as LCID, LOCALE_SLANGUAGE, pszDesc, cchDesc as INT) != 0 {
        return (TRUE) as BOOL;
    }

    *pszDesc = UNICODE_NULL;

    if LOWORD!(hKL) == HIWORD!(hKL) {
        return (FALSE) as BOOL;
    }

    return GetKbdLayoutNameFromReg(hKL, pszDesc, cchDesc);
}

unsafe extern "C" fn GetHKLDesctription(hKL: HKL, pszDesc: LPWSTR, cchDesc: UINT, pszImeFileName: LPWSTR, cchImeFileName: UINT) -> BOOL {
    *pszImeFileName.add(0) = UNICODE_NULL;
    *pszDesc.add(0) = UNICODE_NULL;

    if IS_IME_HKL!(hKL) == (((false) as i32) != 0) {
        return GetHKLName(hKL, pszDesc, cchDesc);
    }

    if GetKbdLayoutNameFromReg(hKL, pszDesc, cchDesc) != 0 {
        return (TRUE) as BOOL;
    }

    if ImmGetDescriptionW(hKL, pszDesc, cchDesc) == 0 {
        *pszDesc = UNICODE_NULL;
        return GetHKLName(hKL, pszDesc, cchDesc);
    }

    if ImmGetIMEFileNameW(hKL, pszImeFileName, cchImeFileName) == 0 {
        *pszImeFileName = UNICODE_NULL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetIconFromFile(cx: INT, cy: INT, pszFileName: LPCWSTR, iIcon: INT) -> HICON {
    let mut hIcon: HICON = core::mem::zeroed();

    if cx <= GetSystemMetrics((SM_CXSMICON as i32)) {
        ExtractIconExW(pszFileName, iIcon, core::ptr::null_mut(), core::ptr::addr_of_mut!(hIcon), 1);
    } else {
        ExtractIconExW(pszFileName, iIcon, core::ptr::addr_of_mut!(hIcon), core::ptr::null_mut(), 1);
    }

    return hIcon;
}

unsafe extern "C" fn EnsureIconImageList() -> BOOL {
    if (CStaticIconList__s_cx == 0) {
        (*core::ptr::addr_of_mut!(g_IconList)).Init(GetSystemMetrics((SM_CYSMICON as i32)), GetSystemMetrics((SM_CXSMICON as i32)));
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn GetPhysicalFontHeight(plf: *mut LOGFONTW) -> INT {
    let hDC: HDC = GetDC(core::ptr::null_mut());
    let hFont: HFONT = CreateFontIndirectW(plf);
    let hFontOld: HGDIOBJ = SelectObject(hDC, (hFont).cast());
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    GetTextMetricsW(hDC, &mut tm);
    let ret: INT = tm.tmExternalLeading + tm.tmHeight;
    SelectObject(hDC, hFontOld);
    DeleteObject((hFont).cast());
    ReleaseDC(core::ptr::null_mut(), hDC);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn InatAddIcon(hIcon: HICON) -> INT {
    if EnsureIconImageList() == 0 {
        return -1;
    }
    return g_IconList.AddIcon(hIcon);
}
#[no_mangle]
pub unsafe extern "C" fn InatCreateIconBySize(LangID: LANGID, nWidth: i32, nHeight: i32, plf: *const LOGFONTW) -> HICON {
    let mut szText: [u16; 64] = [0; 64];
    let ret: i32 = GetLocaleInfoW(LangID as LCID, LOCALE_NOUSEROVERRIDE | LOCALE_SABBREVLANGNAME,
                                szText.as_mut_ptr(), _countof!(szText) as INT) as i32;
    if ret == 0 {
        szText[1] = 0x003Fu16;
        szText[0] = 0x003Fu16;
    }

    szText[2] = UNICODE_NULL;
    CharUpperW(szText.as_mut_ptr());

    let hFont = CreateFontIndirectW(plf);
    if hFont.is_null() {
        return core::ptr::null_mut();
    }

    let hDC = GetDC(core::ptr::null_mut());
    let hMemDC = CreateCompatibleDC(hDC);
    let hbmColor = CreateCompatibleBitmap(hDC, nWidth, nHeight);
    let hbmMask = CreateBitmap(nWidth, nHeight, 1, 1, core::ptr::null_mut());
    ReleaseDC(core::ptr::null_mut(), hDC);

    let mut hIcon = core::ptr::null_mut();
    let hbmOld = SelectObject(hMemDC, (hbmColor).cast());
    let hFontOld = SelectObject(hMemDC, (hFont).cast());
    if !hMemDC.is_null() && !hbmColor.is_null() && !hbmMask.is_null()
    {
        SetBkColor(hMemDC, GetSysColor((COLOR_HIGHLIGHT as i32)));
        SetTextColor(hMemDC, GetSysColor((COLOR_HIGHLIGHTTEXT as i32)));

        let mut rc = RECT { left: 0, top: 0, right: nWidth, bottom: nHeight };
        ExtTextOutW(hMemDC, 0, 0, ETO_OPAQUE, core::ptr::addr_of_mut!(rc), CPP_COM_WSTR_96A296D224F2.as_ptr(), 0, core::ptr::null_mut());

        DrawTextW(hMemDC, szText.as_mut_ptr(), 2, core::ptr::addr_of_mut!(rc), DT_SINGLELINE | DT_CENTER | DT_VCENTER);
        SelectObject(hMemDC, (hbmMask).cast());

        PatBlt(hMemDC, 0, 0, nWidth, nHeight, BLACKNESS);

        let mut IconInfo = ICONINFO { fIcon: (TRUE) as i32, xHotspot: 0, yHotspot: 0, hbmMask: hbmMask, hbmColor: hbmColor };
        hIcon = CreateIconIndirect(core::ptr::addr_of_mut!(IconInfo));
    }
    SelectObject(hMemDC, hFontOld);
    SelectObject(hMemDC, hbmOld);

    DeleteObject((hbmMask).cast());
    DeleteObject((hbmColor).cast());
    DeleteDC(hMemDC);
    DeleteObject((hFont).cast());
    return hIcon;
}
#[no_mangle]
pub unsafe extern "C" fn InatCreateIcon(LangID: LANGID) -> HICON {
    let cxSmIcon: INT = GetSystemMetrics((SM_CXSMICON as i32));
    let cySmIcon: INT = GetSystemMetrics((SM_CYSMICON as i32));

    let mut lf: LOGFONTW = core::mem::zeroed();
    if SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT,
        (core::mem::size_of::<LOGFONTW>()) as u32,
        core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
        0,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    if cySmIcon < GetPhysicalFontHeight(core::ptr::addr_of_mut!(lf)) {
        lf.lfWidth = 0;
        lf.lfHeight = -(7 * cySmIcon) / 10;
    }

    return InatCreateIconBySize(LangID, cxSmIcon, cySmIcon, core::ptr::addr_of_mut!(lf));
}
#[no_mangle]
pub unsafe extern "C" fn InatGetIconSize(pcx: *mut INT, pcy: *mut INT) -> BOOL {
    (*core::ptr::addr_of_mut!(g_IconList)).GetIconSize(pcx, pcy);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InatGetImageCount() -> i32 {
    (*core::ptr::addr_of_mut!(g_IconList)).GetImageCount()
}
#[no_mangle]
pub unsafe extern "C" fn InatRemoveAll() {
    if CStaticIconList__s_cx != 0 {
        g_IconList.RemoveAll((FALSE) as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn UninitINAT() {
    g_IconList.RemoveAll((TRUE) as BOOL);

    if !g_pMlngInfo.is_null() {
        cpp_delete(g_pMlngInfo);
        g_pMlngInfo = core::ptr::null_mut();
    }
}

unsafe extern "C" fn CheckMlngInfo() -> i32 {
    if g_pMlngInfo.is_null() {
        return (TRUE) as i32; // Needs creation
    }

    let cKLs: i32 = (GetKeyboardLayoutList(0, core::ptr::null_mut())) as i32;
    if cKLs != TF_MlngInfoCount() {
        return (TRUE) as i32; // Needs refresh
    }

    if cKLs == 0 {
        return (FALSE) as i32;
    }

    let phKLs: *mut HKL = cicMemAlloc((cKLs as usize) * core::mem::size_of::<HKL>()) as *mut HKL;
    if phKLs.is_null() {
        return (FALSE) as i32;
    }

    GetKeyboardLayoutList(cKLs, phKLs);

    assert!(!g_pMlngInfo.is_null());

    let mut ret: i32 = (FALSE) as i32;
    let mut iKL: i32 = 0;
    while iKL < cKLs {
        if (*g_pMlngInfo)[iKL as usize].m_hKL != *phKLs.offset(iKL as isize) {
            ret = (TRUE) as i32; // Needs refresh
            break;
        }
        iKL += 1;
    }

    cicMemFree(phKLs as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn DestroyMlngInfo() {
    if g_pMlngInfo.is_null() {
        return;
    }

    cpp_delete(g_pMlngInfo);
    g_pMlngInfo = core::ptr::null_mut();
}

unsafe extern "C" fn CreateMlngInfo() {
    if g_pMlngInfo.is_null() {
        // `new(cicNoThrow) CicArray<MLNGINFO>()`: allocate, construct only if that worked.
        g_pMlngInfo = cpp_alloc_CicNoThrow::<CicArray<MLNGINFO>>();
        if !g_pMlngInfo.is_null() {
            (*g_pMlngInfo).__ctor_0();
        }
        if g_pMlngInfo.is_null() {
            return;
        }
    }

    if EnsureIconImageList() == 0 {
        return;
    }

    let cKLs: INT = (GetKeyboardLayoutList(0, core::ptr::null_mut())) as i32;
    let phKLs: *mut HKL = cicMemAllocClear(
        ((cKLs as usize) * core::mem::size_of::<HKL>()) as _,
    ) as *mut HKL;
    if phKLs.is_null() {
        return;
    }

    GetKeyboardLayoutList(cKLs, phKLs);

    let mut iKL: INT = 0;
    while iKL < cKLs {
        let info: &mut MLNGINFO = &mut (*g_pMlngInfo)[iKL as usize];
        info.m_hKL = *phKLs.offset(iKL as isize);
        info.m_bInitDesc = (FALSE) as i32;
        info.m_bInitIcon = (FALSE) as i32;
        iKL += 1;
    }

    cicMemFree((phKLs).cast());
}

#[no_mangle]
pub unsafe extern "system" fn TF_InitMlngInfo() {
    EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));

    if CheckMlngInfo() != 0 {
        DestroyMlngInfo();
        CreateMlngInfo();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
}

#[no_mangle]
pub unsafe extern "system" fn TF_MlngInfoCount() -> i32 {
    if g_pMlngInfo.is_null() {
        return 0;
    }

    return (*g_pMlngInfo).size() as i32;
}

#[no_mangle]
pub unsafe extern "system" fn TF_InatExtractIcon(iKL: i32) -> HICON {
    return (*core::ptr::addr_of_mut!(g_IconList)).ExtractIconW(iKL);
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetMlngIconIndex(iKL: INT) -> INT {
    let mut iIcon: INT = -1;

    EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));

    assert!(!g_pMlngInfo.is_null());

    if iKL < (*g_pMlngInfo).size() as INT {
        iIcon = (*g_pMlngInfo)[iKL as usize].GetIconIndex();
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));

    return iIcon;
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetMlngHKL(iKL: INT, phKL: *mut HKL, pszDesc: LPWSTR, cchDesc: INT) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;

    EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));

    assert!(!g_pMlngInfo.is_null());

    if iKL < (*g_pMlngInfo).size() as INT {
        let info: &mut MLNGINFO = &mut (*g_pMlngInfo)[iKL as usize];

        if !phKL.is_null() {
            *phKL = info.m_hKL;
        }

        if !pszDesc.is_null() {
            StringCchCopyW(pszDesc, cchDesc as usize, info.GetDesc());
        }

        ret = (TRUE) as i32;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));

    return ret;
}

impl MLNGINFO {
    pub unsafe fn InitDesc(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bInitDesc != 0 {
            return;
        }
    
        let mut szDesc = [0 as WCHAR; (MAX_PATH) as _];
        let mut szImeFileName = [0 as WCHAR; (MAX_PATH) as _];
        GetHKLDesctription((*this_).m_hKL, szDesc.as_mut_ptr(), _countof!(szDesc) as UINT,
                           szImeFileName.as_mut_ptr(), _countof!(szImeFileName) as UINT);
        (*this_).SetDesc(szDesc.as_ptr());
        (*this_).m_bInitDesc = (TRUE) as i32;
    }
    pub unsafe fn InitIcon(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bInitIcon != 0 {
            return;
        }
    
        let mut szDesc: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut szImeFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        GetHKLDesctription(
            (*this_).m_hKL,
            szDesc.as_mut_ptr(),
            _countof!(szDesc) as UINT,
            szImeFileName.as_mut_ptr(),
            _countof!(szImeFileName) as UINT,
        );
        (*this_).SetDesc(szDesc.as_ptr());
        (*this_).m_bInitDesc = (TRUE) as i32;
    
        let mut cxIcon: INT = 0;
        let mut cyIcon: INT = 0;
        InatGetIconSize(&mut cxIcon, &mut cyIcon);
    
        let mut hIcon: HICON = core::ptr::null_mut();
        if szImeFileName[0 as usize] != 0 {
            hIcon = GetIconFromFile(cxIcon, cyIcon, szImeFileName.as_ptr(), 0);
        }
    
        if hIcon.is_null() {
            hIcon = InatCreateIcon(LOWORD!((*this_).m_hKL as usize));
        }
    
        if !hIcon.is_null() {
            (*this_).m_iIconIndex = InatAddIcon(hIcon);
            DestroyIcon(hIcon);
        }
    
        (*this_).m_bInitIcon = (TRUE) as i32;
    }
    pub unsafe fn GetIconIndex(&mut self) -> INT {
        let this_: *mut Self = self;
        if (*this_).m_bInitIcon == 0 {
            (*this_).InitIcon();
        }
    
        return (*this_).m_iIconIndex;
    }
    pub unsafe fn GetDesc(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        if (*this_).m_bInitDesc == 0 {
            (*this_).InitDesc();
        }
    
        return (*this_).m_szDesc.as_ptr();
    }
    pub unsafe fn SetDesc(&mut self, pszDesc: LPCWSTR) {
        let this_: *mut Self = self;
        StringCchCopyW((*this_).m_szDesc.as_mut_ptr(), (core::mem::size_of_val(&(*this_).m_szDesc) / core::mem::size_of::<WCHAR>()) as usize, pszDesc);
    }
}

impl CStaticIconList {
    pub unsafe fn Init(&mut self, cxIcon: INT, cyIcon: INT) {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
        CStaticIconList__s_cx = cxIcon;
        CStaticIconList__s_cy = cyIcon;
        LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
    }
    pub unsafe fn AddIcon(&mut self, hIcon: HICON) -> INT {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
    
        let mut iItem: INT = -1;
        let hCopyIcon: HICON = CopyIcon(hIcon);
        if !hCopyIcon.is_null() {
            if g_IconList.Add(hIcon) != 0 {
                iItem = (g_IconList.size() - 1) as INT;
            }
        }
    
        LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
        return iItem;
    }
    pub unsafe fn ExtractIconW(&mut self, iIcon: INT) -> HICON {
        let this_: *mut Self = self;
        let mut hCopyIcon: HICON = core::ptr::null_mut();
        EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
        if iIcon <= (g_IconList.size() as INT) {
            hCopyIcon = CopyIcon(
                *(core::ptr::addr_of_mut!(g_IconList.__base0) as *mut HICON).add(iIcon as usize),
            );
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
        return hCopyIcon;
    }
    pub unsafe fn GetIconSize(&mut self, pcx: *mut INT, pcy: *mut INT) {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
        *pcx = CStaticIconList__s_cx;
        *pcy = CStaticIconList__s_cy;
        LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
    }
    pub unsafe fn GetImageCount(&mut self) -> INT {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
        let cItems: INT = (*core::ptr::addr_of_mut!(g_IconList)).size() as INT;
        LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
        return cItems;
    }
    pub unsafe fn RemoveAll(&mut self, bNoLock: BOOL) {
        let this_: *mut Self = self;
        if bNoLock == 0 {
            EnterCriticalSection(core::ptr::addr_of_mut!(g_cs));
        }
    
        let mut iItem: usize = 0;
        while iItem < g_IconList.size() {
            DestroyIcon(g_IconList.__base0[iItem]);
            iItem += 1;
        }
    
        (*this_).__base0.clear();
    
        if bNoLock == 0 {
            LeaveCriticalSection(core::ptr::addr_of_mut!(g_cs));
        }
    }
}
