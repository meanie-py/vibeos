use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_280DCAD69C43: [WCHAR; 10] = [0x0049, 0x0074, 0x0065, 0x006D, 0x0053, 0x0074, 0x0061, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_672463267A33: [WCHAR; 6] = [0x0052, 0x0045, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_7289CE32E451: [WCHAR; 8] = [0x0050, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x0000];
pub static CPP_COM_WSTR_91CDBB74B095: [WCHAR; 8] = [0x0054, 0x004F, 0x004F, 0x004C, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A5A4F2227C74: [WCHAR; 41] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043, 0x0054, 0x0046, 0x005C, 0x004C, 0x0061, 0x006E, 0x0067, 0x0042, 0x0061, 0x0072, 0x005C, 0x0049, 0x0074, 0x0065, 0x006D, 0x0053, 0x0074, 0x0061, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_A83D1779AB13: [WCHAR; 15] = [0x0054, 0x0061, 0x0073, 0x006B, 0x0062, 0x0061, 0x0072, 0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_B728528EE8A8: [WCHAR; 8] = [0x004D, 0x0061, 0x0072, 0x006C, 0x0065, 0x0074, 0x0074, 0x0000];
pub static CPP_COM_WSTR_BB39C012AB62: [WCHAR; 14] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x005F, 0x0054, 0x0072, 0x0061, 0x0079, 0x0057, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_C8D26F04DF9D: [WCHAR; 12] = [0x0044, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x004C, 0x0065, 0x0076, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_D3128F43EECC: [WCHAR; 8] = [0x0054, 0x0041, 0x0053, 0x004B, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_DAD1FE337325: [WCHAR; 16] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0044, 0x0065, 0x006D, 0x006F, 0x0074, 0x0069, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_DDBF4DF411E1: [WCHAR; 7] = [0x0057, 0x0049, 0x004E, 0x0044, 0x004F, 0x0057, 0x0000];
pub static CPP_COM_WSTR_E8561835E59F: [WCHAR; 31] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043, 0x0054, 0x0046, 0x005C, 0x004C, 0x0061, 0x006E, 0x0067, 0x0042, 0x0061, 0x0072, 0x0000];
pub static CPP_COM_WSTR_FB3ED1CCBECC: [WCHAR; 11] = [0x004F, 0x004C, 0x0045, 0x0041, 0x0043, 0x0043, 0x002E, 0x0044, 0x004C, 0x004C, 0x0000];

use core::ffi::c_void;

extern "system" {
    // `GetObject` is a UNICODE macro for GetObjectW; gdi32 exports no bare GetObject.
    #[link_name = "GetObjectW"]
    fn GetObject(h: HGDIOBJ, c: i32, pv: *mut core::ffi::c_void) -> i32;
}

// ---- file-scope preamble (translated) ----
pub type PLANGBARITEMSTATE = *mut LANGBARITEMSTATE;

// struct __wine_debug_channel from Wine's debug.h: { unsigned char flags; char name[15]; }
// WINE_DEFAULT_DEBUG_CHANNEL(msutb) expansion (flags/level byte kept at 0; name = "msutb")
pub static mut __wine_dbch_msutb: __wine_debug_channel = __wine_debug_channel {
    flags: 0,
    name: [b'm' as i8, b's' as i8, b'u' as i8, b't' as i8, b'b' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut g_hInst: HINSTANCE = core::ptr::null_mut();
pub static mut g_wmTaskbarCreated: UINT = 0;
pub static mut g_uACP: UINT = 0; // CP_ACP
pub static mut g_dwOSInfo: DWORD = 0;
pub static mut g_cs: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut g_DllRefCount: LONG = 0;
pub static mut g_bWinLogon: BOOL = 0;
pub static mut g_fInClosePopupTipbar: BOOL = 0;
pub static mut g_hwndParent: HWND = core::ptr::null_mut();
// CIC_LIBTHREAD g_libTLS = { NULL, NULL }; -- all fields are pointers, so zero-init matches
pub static mut g_libTLS: CIC_LIBTHREAD = unsafe { core::mem::zeroed() };
pub static mut g_bEnableDeskBand: BOOL = 0;
pub static mut g_bShowTipbar: BOOL = 1;
pub static mut g_bShowDebugMenu: BOOL = 0;
pub static mut g_bNewLook: BOOL = 1;
pub static mut g_bIntelliSense: BOOL = 0;
pub static mut g_bShowCloseMenu: BOOL = 0;
pub static mut g_uTimeOutNonIntentional: UINT = 60 * 1000;
pub static mut g_uTimeOutIntentional: UINT = 10 * 60 * 1000;
pub static mut g_uTimeOutMax: UINT = 60 * 60 * 1000;
pub static mut g_bShowMinimizedBalloon: BOOL = 1;
pub static mut g_ptTipbar: POINT = POINT { x: -1, y: -1 };
pub static mut g_bExcludeCaptionButtons: BOOL = 1;
pub static mut g_bShowShadow: BOOL = 0;
pub static mut g_fTaskbarTheme: BOOL = 1;
pub static mut g_fVertical: BOOL = 0;
pub static mut g_uTimerElapseSTUBSTART: UINT = 100;
pub static mut g_uTimerElapseSTUBEND: UINT = 2 * 1000;
pub static mut g_uTimerElapseBACKTOALPHA: UINT = 3 * 1000;
pub static mut g_uTimerElapseONTHREADITEMCHANGE: UINT = 200;
pub static mut g_uTimerElapseSETWINDOWPOS: UINT = 100;
pub static mut g_uTimerElapseONUPDATECALLED: UINT = 50;
pub static mut g_uTimerElapseSYSCOLORCHANGED: UINT = 20;
pub static mut g_uTimerElapseDISPLAYCHANGE: UINT = 20;
pub static mut g_uTimerElapseUPDATEUI: UINT = 70;
pub static mut g_uTimerElapseSHOWWINDOW: UINT = 50;
pub static mut g_uTimerElapseMOVETOTRAY: UINT = 50;
pub static mut g_uTimerElapseTRAYWNDONDELAYMSG: UINT = 50;
pub static mut g_uTimerElapseDOACCDEFAULTACTION: UINT = 200;
pub static mut g_uTimerElapseENSUREFOCUS: UINT = 50;
pub static mut g_bShowDeskBand: BOOL = 0;
pub static mut g_uTimerElapseSHOWDESKBAND: UINT = 3 * 1000;
pub static mut g_fPolicyDisableCloseButton: BOOL = 0;
pub static mut g_fPolicyEnableLanguagebarInFullscreen: BOOL = 0;
pub static mut g_dwWndStyle: DWORD = 0;
pub static mut g_dwMenuStyle: DWORD = 0;
pub static mut g_dwChildWndStyle: DWORD = 0;
pub static mut g_fRTL: BOOL = 0;
pub static mut g_pTipbarWnd: *mut CTipbarWnd = core::ptr::null_mut();
pub static mut g_pTrayIconWnd: *mut CTrayIconWnd = core::ptr::null_mut();
// CicArray<HKL> is a C++ template; opaque instantiation name
pub static mut g_prghklSkipRedrawing: *mut CicArray_HKL = core::ptr::null_mut();
pub static mut CUTBCloseLangBarDlg__s_bIsDlgShown: BOOL = 0;

// static _ATL_OBJMAP_ENTRY ObjectMap[1] = BEGIN_OBJECT_MAP(ObjectMap) END_OBJECT_MAP();
// The map is empty: a single terminating (all-NULL) entry.
// END_OBJECT_MAP's terminator only (ENABLE_DESKBAND is off): a null pclsid ends the walk.
pub static mut ObjectMap: [_ATL_OBJMAP_ENTRY; 1] = unsafe { core::mem::zeroed() };


























































































































































































































































































































































unsafe fn _countof<T, const N: usize>(_a: *const [T; N]) -> usize {
    N
}










































































#[no_mangle]
pub unsafe extern "C" fn IsSkipRedrawHKL(hSkipKL: HKL) -> BOOL {
    if LOWORD!(hSkipKL) == (LANGID_JAPANESE as u16) {
        return (FALSE) as BOOL; // Japanese HKL will be skipped
    }
    if g_prghklSkipRedrawing.is_null() {
        return (FALSE) as BOOL;
    }

    let mut iItem: usize = 0 as usize;
    while iItem < (*g_prghklSkipRedrawing).size() {
        if (*g_prghklSkipRedrawing)[iItem as size_t] == hSkipKL {
            return (TRUE) as BOOL; // To be skipped
        }
        iItem += 1;
    }

    return (FALSE) as BOOL; // To be not skipped
}
#[no_mangle]
pub unsafe extern "C" fn IsBiDiLocalizedSystem() -> BOOL {
    let mut Sig: LOCALESIGNATURE = core::mem::zeroed();
    let LangID: LANGID = GetUserDefaultUILanguage();
    if LangID == 0 {
        return (FALSE) as BOOL;
    }

    let size: INT = (core::mem::size_of::<LOCALESIGNATURE>() / core::mem::size_of::<WCHAR>()) as INT;
    if GetLocaleInfoW((LangID) as LCID, LOCALE_FONTSIGNATURE, core::ptr::addr_of_mut!(Sig) as LPWSTR, size) == 0 {
        return (FALSE) as BOOL;
    }
    ((Sig.lsUsb[3 as usize] & 0x8000000) != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn GetFontSig(hWnd: HWND, hKL: HKL) -> BOOL {
    let mut Sig: LOCALESIGNATURE = core::mem::zeroed();
    let mut size: INT = (core::mem::size_of::<LOCALESIGNATURE>() / core::mem::size_of::<WCHAR>()) as INT;
    if !(GetLocaleInfoW((LOWORD!(hKL as usize as DWORD)) as LCID, LOCALE_FONTSIGNATURE, core::ptr::addr_of_mut!(Sig) as LPWSTR, size) != 0) {
        return (FALSE) as BOOL;
    }

    let hDC: HDC = GetDC(hWnd);
    let CharSet: DWORD = (GetTextCharsetInfo(hDC, core::ptr::null_mut(), 0)) as u32;
    let mut CharSetInfo: CHARSETINFO = core::mem::zeroed();
    TranslateCharsetInfo(CharSet as DWORD_PTR as *mut DWORD, core::ptr::addr_of_mut!(CharSetInfo), TCI_SRCCHARSET);
    ReleaseDC(hWnd, hDC);

    (((CharSetInfo.fs.fsCsb[0] & Sig.lsCsbSupported[0]) != 0) as BOOL)
}
#[no_mangle]
pub unsafe extern "C" fn InitSkipRedrawHKLArray() {
    // `new(cicNoThrow) CicArray<HKL>()`: allocate, construct only if that worked.
    g_prghklSkipRedrawing = cpp_alloc_CicNoThrow::<CicArray<HKL>>();
    if !g_prghklSkipRedrawing.is_null() {
        (*g_prghklSkipRedrawing).__ctor_0();
    }
    if g_prghklSkipRedrawing.is_null() {
        return;
    }

    if g_bEnableDeskBand != 0 && (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) != 0 {
        // Japanese IME will be skipped
        (*g_prghklSkipRedrawing).Add(UlongToHandle!(0xE0010411) as HKL);
    }

    let mut regKey: CicRegKey = core::mem::zeroed();
    let mut error: LSTATUS = regKey.Open(
        HKEY_LOCAL_MACHINE,
        (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0053, 0x006b, 0x0069, 0x0070, 0x0052, 0x0065, 0x0064, 0x0072, 0x0061, 0x0077, 0x0048, 0x004b, 0x004c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\SkipRedrawHKL" */,
        KEY_READ,
    );
    if (error) as u32 != ERROR_SUCCESS {
        return;
    }

    let mut szValueName: [TCHAR; 256] = [0; 256];
    let mut dwIndex: DWORD = 0;
    loop {
        error = regKey.EnumValue(dwIndex, szValueName.as_mut_ptr(), _countof!(szValueName));
        if (error) as u32 != ERROR_SUCCESS {
            break;
        }

        if szValueName[0] == ('0' as u16)
            && (szValueName[1] == ('x' as u16)
                || szValueName[1] == ('X' as u16))
        {
            let hKL: HKL = UlongToHandle!(_tcstoul(
                szValueName.as_mut_ptr(),
                core::ptr::null_mut(),
                16,
            )) as HKL;
            (*g_prghklSkipRedrawing).Add(hKL); // This hKL will be skipped
        }
        dwIndex += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn UninitSkipRedrawHKLArray() {
    if !g_prghklSkipRedrawing.is_null() {
        cpp_delete(g_prghklSkipRedrawing);
        g_prghklSkipRedrawing = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetGlobalCompartment(rguid: REFGUID, ppComp: *mut *mut ITfCompartment) -> HRESULT {
    let mut pCompMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();
    let mut hr: HRESULT = TF_GetGlobalCompartment(core::ptr::addr_of_mut!(pCompMgr));
    if (hr) < 0 {
        return hr;
    }

    if pCompMgr.is_null() {
        return E_FAIL;
    }

    hr = (*pCompMgr).GetCompartment(rguid, ppComp);
    (*pCompMgr).Release();
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn GetGlobalCompartmentDWORD(rguid: REFGUID, pdwValue: LPDWORD) -> HRESULT {
    *pdwValue = 0;
    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();
    let mut hr: HRESULT = GetGlobalCompartment(rguid, core::ptr::addr_of_mut!(pComp));
    if hr >= 0 {
        let mut vari: VARIANT = core::mem::zeroed();
        hr = (*pComp).GetValue(core::ptr::addr_of_mut!(vari));
        if hr == S_OK {
            *pdwValue = V_I4!(core::ptr::addr_of_mut!(vari)) as DWORD;
        }
        (*pComp).Release();
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SetGlobalCompartmentDWORD(rguid: REFGUID, dwValue: DWORD) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();
    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();
    let mut hr: HRESULT = GetGlobalCompartment(rguid, core::ptr::addr_of_mut!(pComp));
    if hr >= 0 {
        V_VT!(core::ptr::addr_of_mut!(vari)) = (VT_I4 as u16);
        V_I4!(core::ptr::addr_of_mut!(vari)) = (dwValue) as i32;
        hr = (*pComp).SetValue(0, core::ptr::addr_of_mut!(vari));
        (*pComp).Release();
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn TurnOffSpeechIfItsOn() {
    let mut dwValue: DWORD = 0;
    let hr: HRESULT = GetGlobalCompartmentDWORD(&GUID_COMPARTMENT_SPEECH_OPENCLOSE, core::ptr::addr_of_mut!(dwValue));
    if (hr >= 0) && dwValue != 0 {
        SetGlobalCompartmentDWORD(&GUID_COMPARTMENT_SPEECH_OPENCLOSE, 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DoCloseLangbar() {
    let mut pLangBarMgr: *mut ITfLangBarMgr = core::ptr::null_mut();
    let mut hr: HRESULT = TF_CreateLangBarMgr(core::ptr::addr_of_mut!(pLangBarMgr));
    if hr < 0 {
        return;
    }

    if !pLangBarMgr.is_null() {
        hr = (*pLangBarMgr).ShowFloating(TF_SFT_HIDDEN);
        (*pLangBarMgr).Release();
    }

    if hr >= 0 {
        TurnOffSpeechIfItsOn();
    }

    let mut regKey: CicRegKey = CicRegKey__zeroed();
    let error: LSTATUS = regKey.Open(
        HKEY_CURRENT_USER,
        (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0052, 0x0075, 0x006e, 0x0000u16]).as_ptr() /* TEXT "Software\\Microsoft\\Windows\\CurrentVersion\\Run" */,
        KEY_ALL_ACCESS,
    );
    if (error) as u32 == ERROR_SUCCESS {
        RegDeleteValue(regKey.m_hKey, (&[0x0063, 0x0074, 0x0066, 0x006d, 0x006f, 0x006e, 0x002e, 0x0065, 0x0078, 0x0065, 0x0000u16]).as_ptr() /* TEXT "ctfmon.exe" */);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetIconIndexFromhKL(hKL: HKL) -> INT {
    let mut hGotKL: HKL = core::mem::zeroed();

    let mut iKL: INT = 0;
    let cKLs: INT = TF_MlngInfoCount();
    iKL = 0;
    while iKL < cKLs {
        if TF_GetMlngHKL(iKL, core::ptr::addr_of_mut!(hGotKL), core::ptr::null_mut(), 0) != 0 && hKL == hGotKL {
            return TF_GetMlngIconIndex(iKL);
        }
        iKL += 1;
    }

    if TF_GetMlngHKL(0, core::ptr::addr_of_mut!(hGotKL), core::ptr::null_mut(), 0) == 0 {
        return -1;
    }

    return TF_GetMlngIconIndex(0);
}

#[no_mangle]
pub unsafe extern "C" fn GethKLDesc(hKL: HKL, pszDesc: LPWSTR, cchDesc: UINT) -> BOOL {
    let mut hGotKL: HKL = core::mem::zeroed();

    let mut iKL: INT;
    let cKLs: INT = TF_MlngInfoCount();
    iKL = 0;
    while iKL < cKLs {
        if TF_GetMlngHKL(iKL, core::ptr::addr_of_mut!(hGotKL), pszDesc, (cchDesc) as INT) != 0 && hKL == hGotKL {
            return (TRUE) as BOOL;
        }
        iKL += 1;
    }

    return TF_GetMlngHKL(0, core::ptr::addr_of_mut!(hGotKL), pszDesc, (cchDesc) as INT);
}
#[no_mangle]
pub unsafe extern "C" fn LangBarInsertMenu(pMenu: *mut ITfMenu, uId: UINT, pszText: LPCWSTR, bChecked: BOOL, hIcon: HICON) -> HRESULT {
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut hbmpMask: HBITMAP = core::ptr::null_mut();
    if !hIcon.is_null() {
        let hIconNew: HICON = CopyImage(hIcon as HANDLE, IMAGE_ICON, 16, 16, LR_COPYFROMRESOURCE) as HICON;
        let mut iconSize: SIZE = SIZE { cx: 16, cy: 16 };
        let mut hIconNew = hIconNew;
        if hIconNew.is_null() {
            hIconNew = hIcon;
        }
        if cicGetIconBitmaps(hIconNew, core::ptr::addr_of_mut!(hbmp), core::ptr::addr_of_mut!(hbmpMask), core::ptr::addr_of_mut!(iconSize)) == 0 {
            return E_FAIL;
        }
        if !hIconNew.is_null() {
            DestroyIcon(hIconNew);
        }
        DestroyIcon(hIcon);
    }

    let cchText: INT = lstrlenW(pszText) as INT;
    let dwFlags: DWORD = if bChecked != 0 { TF_LBMENUF_CHECKED } else { 0 };
    return (*pMenu).AddMenuItem(uId, dwFlags, hbmp, hbmpMask, pszText, (cchText) as ULONG, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn LangBarInsertSeparator(pMenu: *mut ITfMenu) -> HRESULT {
    return (((*(*pMenu).lpVtbl).AddMenuItem).unwrap_unchecked())(pMenu, (1 as u32).wrapping_neg(), TF_LBMENUF_SEPARATOR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 0, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn IsFELangId(LangID: LANGID) -> BOOL {
    match (LangID) as i32 {
        LANGID_CHINESE_SIMPLIFIED | // Chinese (Simplified)
        LANGID_CHINESE_TRADITIONAL | // Chinese (Traditional)
        LANGID_JAPANESE | // Japanese
        LANGID_KOREAN => { // Korean
            return (TRUE) as BOOL;
        }
        _ => {
            return (FALSE) as BOOL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CheckCloseMenuAvailable() -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pProfiles: *mut ITfInputProcessorProfiles = core::ptr::null_mut();
    let mut pLangIds: *mut LANGID = core::ptr::null_mut();
    let mut iItem: ULONG;
    let mut cItems: ULONG = 0;

    if g_fPolicyDisableCloseButton != 0 {
        return (FALSE) as BOOL;
    }

    if g_bShowCloseMenu != 0 {
        return (TRUE) as BOOL;
    }

    if (TF_CreateInputProcessorProfiles(core::ptr::addr_of_mut!(pProfiles)) >= 0)
        && ((*pProfiles).GetLanguageList(
            core::ptr::addr_of_mut!(pLangIds),
            core::ptr::addr_of_mut!(cItems),
        ) >= 0)
    {
        iItem = 0;
        while iItem < cItems {
            if IsFELangId(*pLangIds.add(iItem as usize)) != 0 {
                break;
            }
            iItem += 1;
        }

        ret = (iItem == cItems) as BOOL;
    }

    if !pLangIds.is_null() {
        CoTaskMemFree(pLangIds as *mut core::ffi::c_void);
    }
    if !pProfiles.is_null() {
        (*pProfiles).Release();
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn IsTransparecyAvailable() -> BOOL {
    (FALSE) as i32
}

unsafe extern "system" fn FindEAEnumFontProc(pLF: *mut ENUMLOGFONT, pTM: *mut NEWTEXTMETRIC, nFontType: INT, lParam: LPARAM) -> INT {
    if (nFontType != (TRUETYPE_FONTTYPE as i32)) || ((*pLF).elfLogFont.lfFaceName[0 as usize] != '@' as u16) {
        return (TRUE) as INT;
    }
    *(lParam as *mut BOOL) = (TRUE) as i32;
    return (FALSE) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn CheckEAFonts() -> BOOL {
    let mut bHasVertical: BOOL = (FALSE) as i32;
    let hDC: HDC = GetDC(core::ptr::null_mut());
    // `(FONTENUMPROC)FindEAEnumFontProc`: the C casts the callback to the SDK's prototype.
    EnumFonts(hDC, core::ptr::null(), core::mem::transmute::<unsafe extern "system" fn(*mut ENUMLOGFONT, *mut NEWTEXTMETRIC, INT, LPARAM) -> INT, FONTENUMPROC>(FindEAEnumFontProc), core::ptr::addr_of_mut!(bHasVertical) as LPARAM);
    ReleaseDC(core::ptr::null_mut(), hDC);
    return bHasVertical;
}
#[no_mangle]
pub unsafe extern "C" fn IsDeskBandFromReg() -> BOOL {
    if g_bEnableDeskBand == 0 || (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) == 0 { // Desk band is for XP+
        return (FALSE) as BOOL;
    }

    let mut regKey: CicRegKey = core::mem::zeroed(); // closes on scope exit (Drop), as in C++
    // The C++ tests `if (regKey.Open(..))`, true when the open FAILED (LSTATUS 0 is success),
    // and then reads the value through the unopened key. Read it when the open succeeded.
    if regKey.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\" */, KEY_READ) == ERROR_SUCCESS as LSTATUS {
        let mut dwValue: DWORD = 0;
        regKey.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0044, 0x0065, 0x0073, 0x006b, 0x0042, 0x0061, 0x006e, 0x0064, 0x0000u16]).as_ptr() /* TEXT "ShowDeskBand" */, core::ptr::addr_of_mut!(dwValue));
        let ret: BOOL = (dwValue != 0) as BOOL;
        return ret;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn SetDeskBandToReg(bShow: BOOL) {
    let mut regKey: CicRegKey = core::mem::zeroed();
    // As in IsDeskBandFromReg, the C++ `if (regKey.Open(..))` is true when the open FAILED;
    // write the value when it succeeded.
    if regKey.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\" */, KEY_ALL_ACCESS) == ERROR_SUCCESS as LSTATUS {
        regKey.SetDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0044, 0x0065, 0x0073, 0x006b, 0x0042, 0x0061, 0x006e, 0x0064, 0x0000u16]).as_ptr() /* TEXT "ShowDeskBand" */, (bShow) as u32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RegisterComCat(rclsid: REFCLSID, rcatid: REFCATID, bRegister: BOOL) -> BOOL {
    if (CoInitialize(core::ptr::null_mut()) < 0) {
        return (FALSE) as BOOL;
    }

    let mut pCat: *mut ICatRegister = core::ptr::null_mut();
    let mut hr: HRESULT = CoCreateInstance(
        core::ptr::addr_of!(CLSID_StdComponentCategoriesMgr),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_ICatRegister),
        core::ptr::addr_of_mut!(pCat) as *mut LPVOID,
    );
    if (hr >= 0) {
        if bRegister != 0 {
            hr = (*pCat).RegisterClassImplCategories(
                rclsid,
                1,
                rcatid as *mut CATID,
            );
        } else {
            hr = (*pCat).UnRegisterClassImplCategories(
                rclsid,
                1,
                rcatid as *mut CATID,
            );
        }

        (*pCat).Release();
    }

    CoUninitialize();

    //if (IsIE5())
    //    RegDeleteKey(HKEY_CLASSES_ROOT, (&[0x0043, 0x006f, 0x006d, 0x0070, 0x006f, 0x006e, 0x0065, 0x006e, 0x0074, 0x0020, 0x0043, 0x0061, 0x0074, 0x0065, 0x0067, 0x006f, 0x0072, 0x0069, 0x0065, 0x0073, 0x005c, 0x007b, 0x0030, 0x0030, 0x0030, 0x0032, 0x0031, 0x0034, 0x0039, 0x0032, 0x002d, 0x0030, 0x0030, 0x0030, 0x0030, 0x002d, 0x0030, 0x0030, 0x0030, 0x0030, 0x002d, 0x0043, 0x0030, 0x0030, 0x0030, 0x002d, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0034, 0x0036, 0x007d, 0x005c, 0x0045, 0x006e, 0x0075, 0x006d, 0x0000u16]).as_ptr() /* TEXT "Component Categories\\{00021492-0000-0000-C000-000000000046}" */);

    return (hr >= 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitFromReg() -> BOOL {
    let mut dwValue: DWORD = 0;
    let mut error: LSTATUS = 0;

    let mut regKey1: CicRegKey = core::mem::zeroed();
    error = regKey1.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey1.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0054, 0x0069, 0x0070, 0x0062, 0x0061, 0x0072, 0x0000u16]).as_ptr() /* TEXT "ShowTipbar" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bShowTipbar = (dwValue != 0) as BOOL;
        }
    }

    let mut regKey2: CicRegKey = core::mem::zeroed();
    error = regKey2.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey2.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0044, 0x0065, 0x0062, 0x0075, 0x0067, 0x004d, 0x0065, 0x006e, 0x0075, 0x0000u16]).as_ptr() /* TEXT "ShowDebugMenu" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bShowDebugMenu = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x004e, 0x0065, 0x0077, 0x004c, 0x006f, 0x006f, 0x006b, 0x0000u16]).as_ptr() /* TEXT "NewLook" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bNewLook = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0049, 0x006e, 0x0074, 0x0065, 0x006c, 0x006c, 0x0069, 0x0053, 0x0065, 0x006e, 0x0073, 0x0065, 0x0000u16]).as_ptr() /* TEXT "IntelliSense" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bIntelliSense = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0043, 0x006c, 0x006f, 0x0073, 0x0065, 0x004d, 0x0065, 0x006e, 0x0075, 0x0000u16]).as_ptr() /* TEXT "ShowCloseMenu" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bShowCloseMenu = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x004f, 0x0075, 0x0074, 0x004e, 0x006f, 0x006e, 0x0049, 0x006e, 0x0074, 0x0065, 0x006e, 0x0074, 0x0069, 0x006f, 0x006e, 0x0061, 0x006c, 0x0000u16]).as_ptr() /* TEXT "TimeOutNonIntentional" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimeOutNonIntentional = 1000 * dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x004f, 0x0075, 0x0074, 0x0049, 0x006e, 0x0074, 0x0065, 0x006e, 0x0074, 0x0069, 0x006f, 0x006e, 0x0061, 0x006c, 0x0000u16]).as_ptr() /* TEXT "TimeOutIntentional" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimeOutIntentional = 1000 * dwValue;
            g_uTimeOutMax = 6000 * dwValue;
        }
        error = regKey2.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x004d, 0x0069, 0x006e, 0x0069, 0x006d, 0x0069, 0x007a, 0x0065, 0x0064, 0x0042, 0x0061, 0x006c, 0x006c, 0x006f, 0x006f, 0x006e, 0x0000u16]).as_ptr() /* TEXT "ShowMinimizedBalloon" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bShowMinimizedBalloon = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x004c, 0x0065, 0x0066, 0x0074, 0x0000u16]).as_ptr() /* TEXT "Left" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_ptTipbar.x = dwValue as LONG;
        }
        error = regKey2.QueryDword((&[0x0054, 0x006f, 0x0070, 0x0000u16]).as_ptr() /* TEXT "Top" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_ptTipbar.y = dwValue as LONG;
        }
        error = regKey2.QueryDword((&[0x0045, 0x0078, 0x0063, 0x006c, 0x0075, 0x0064, 0x0065, 0x0043, 0x0061, 0x0070, 0x0074, 0x0069, 0x006f, 0x006e, 0x0042, 0x0075, 0x0074, 0x0074, 0x006f, 0x006e, 0x0073, 0x0000u16]).as_ptr() /* TEXT "ExcludeCaptionButtons" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bExcludeCaptionButtons = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0053, 0x0068, 0x0061, 0x0064, 0x006f, 0x0077, 0x0000u16]).as_ptr() /* TEXT "ShowShadow" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_bShowShadow = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0061, 0x0073, 0x006b, 0x0062, 0x0061, 0x0072, 0x0054, 0x0068, 0x0065, 0x006d, 0x0065, 0x0000u16]).as_ptr() /* TEXT "TaskbarTheme" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_fTaskbarTheme = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0056, 0x0065, 0x0072, 0x0074, 0x0069, 0x0063, 0x0061, 0x006c, 0x0000u16]).as_ptr() /* TEXT "Vertical" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_fVertical = (dwValue != 0) as BOOL;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0054, 0x0055, 0x0042, 0x0053, 0x0054, 0x0041, 0x0052, 0x0054, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSTUBSTART" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseSTUBSTART = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0054, 0x0055, 0x0042, 0x0045, 0x004e, 0x0044, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSTUBEND" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseSTUBEND = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0042, 0x0041, 0x0043, 0x004b, 0x0054, 0x004f, 0x0041, 0x004c, 0x0050, 0x0048, 0x0041, 0x0000u16]).as_ptr() /* TEXT "TimerElapseBACKTOALPHA" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseBACKTOALPHA = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x004f, 0x004e, 0x0054, 0x0048, 0x0052, 0x0045, 0x0041, 0x0044, 0x0049, 0x0054, 0x0045, 0x004d, 0x0043, 0x0048, 0x0041, 0x004e, 0x0047, 0x0045, 0x0000u16]).as_ptr() /* TEXT "TimerElapseONTHREADITEMCHANGE" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseONTHREADITEMCHANGE = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0045, 0x0054, 0x0057, 0x0049, 0x004e, 0x0044, 0x004f, 0x0057, 0x0050, 0x004f, 0x0053, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSETWINDOWPOS" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseSETWINDOWPOS = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x004f, 0x004e, 0x0055, 0x0050, 0x0044, 0x0041, 0x0054, 0x0045, 0x0043, 0x0041, 0x004c, 0x004c, 0x0045, 0x0044, 0x0000u16]).as_ptr() /* TEXT "TimerElapseONUPDATECALLED" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseONUPDATECALLED = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0059, 0x0053, 0x0043, 0x004f, 0x004c, 0x004f, 0x0052, 0x0043, 0x0048, 0x0041, 0x004e, 0x0047, 0x0045, 0x0044, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSYSCOLORCHANGED" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseSYSCOLORCHANGED = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0044, 0x0049, 0x0053, 0x0050, 0x004c, 0x0041, 0x0059, 0x0043, 0x0048, 0x0041, 0x004e, 0x0047, 0x0045, 0x0000u16]).as_ptr() /* TEXT "TimerElapseDISPLAYCHANGE" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseDISPLAYCHANGE = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0055, 0x0050, 0x0044, 0x0041, 0x0054, 0x0045, 0x0055, 0x0049, 0x0000u16]).as_ptr() /* TEXT "TimerElapseUPDATEUI" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseUPDATEUI = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0048, 0x004f, 0x0057, 0x0057, 0x0049, 0x004e, 0x0044, 0x004f, 0x0057, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSHOWWINDOW" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseSHOWWINDOW = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x004d, 0x004f, 0x0056, 0x0045, 0x0054, 0x004f, 0x0054, 0x0052, 0x0041, 0x0059, 0x0000u16]).as_ptr() /* TEXT "TimerElapseMOVETOTRAY" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseMOVETOTRAY = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0054, 0x0052, 0x0041, 0x0059, 0x0057, 0x004e, 0x0044, 0x004f, 0x004e, 0x0044, 0x0045, 0x004c, 0x0041, 0x0059, 0x004d, 0x0053, 0x0047, 0x0000u16]).as_ptr() /* TEXT "TimerElapseTRAYWNDONDELAYMSG" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseTRAYWNDONDELAYMSG = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0044, 0x004f, 0x0041, 0x0043, 0x0043, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004c, 0x0054, 0x0041, 0x0043, 0x0054, 0x0049, 0x004f, 0x004e, 0x0000u16]).as_ptr() /* TEXT "TimerElapseDOACCDEFAULTACTION" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseDOACCDEFAULTACTION = dwValue;
        }
        error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0045, 0x004e, 0x0053, 0x0055, 0x0052, 0x0045, 0x0046, 0x004f, 0x0043, 0x0055, 0x0053, 0x0000u16]).as_ptr() /* TEXT "TimerElapseENSUREFOCUS" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_uTimerElapseENSUREFOCUS = dwValue;
        }
        if g_bEnableDeskBand != 0 && (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) != 0 {
            error = regKey2.QueryDword((&[0x0053, 0x0068, 0x006f, 0x0077, 0x0044, 0x0065, 0x0073, 0x006b, 0x0042, 0x0061, 0x006e, 0x0064, 0x0000u16]).as_ptr() /* TEXT "ShowDeskBand" */, core::ptr::addr_of_mut!(dwValue));
            if (error) as u32 == ERROR_SUCCESS {
                g_bShowDeskBand = (dwValue != 0) as BOOL;
            }
            error = regKey2.QueryDword((&[0x0054, 0x0069, 0x006d, 0x0065, 0x0072, 0x0045, 0x006c, 0x0061, 0x0070, 0x0073, 0x0065, 0x0053, 0x0048, 0x004f, 0x0057, 0x0057, 0x0044, 0x0045, 0x0053, 0x004b, 0x0042, 0x0041, 0x004e, 0x0044, 0x0000u16]).as_ptr() /* TEXT "TimerElapseSHOWWDESKBAND" */, core::ptr::addr_of_mut!(dwValue));
            if (error) as u32 == ERROR_SUCCESS {
                g_uTimerElapseSHOWDESKBAND = dwValue;
            }
        }
    }

    let mut regKey3: CicRegKey = core::mem::zeroed();
    error = regKey3.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x0050, 0x006f, 0x006c, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x004d, 0x0053, 0x0043, 0x0054, 0x0046, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Policies\\Microsoft\\MSCTF" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey3.QueryDword((&[0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006c, 0x0065, 0x0043, 0x006c, 0x006f, 0x0073, 0x0065, 0x0042, 0x0075, 0x0074, 0x0074, 0x006f, 0x006e, 0x0000u16]).as_ptr() /* TEXT "DisableCloseButton" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_fPolicyDisableCloseButton = (dwValue != 0) as BOOL;
        }
    }

    let mut regKey4: CicRegKey = core::mem::zeroed();
    error = regKey4.Open(HKEY_LOCAL_MACHINE, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x0050, 0x006f, 0x006c, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x004d, 0x0053, 0x0043, 0x0054, 0x0046, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Policies\\Microsoft\\MSCTF" */, KEY_READ);
    if (error) as u32 == ERROR_SUCCESS {
        error = regKey4.QueryDword((&[0x0045, 0x006e, 0x0061, 0x0062, 0x006c, 0x0065, 0x004c, 0x0061, 0x006e, 0x0067, 0x0075, 0x0061, 0x0067, 0x0065, 0x0062, 0x0061, 0x0072, 0x0049, 0x006e, 0x0046, 0x0075, 0x006c, 0x006c, 0x0073, 0x0063, 0x0072, 0x0065, 0x0065, 0x006e, 0x0000u16]).as_ptr() /* TEXT "EnableLanguagebarInFullscreen" */, core::ptr::addr_of_mut!(dwValue));
        if (error) as u32 == ERROR_SUCCESS {
            g_fPolicyEnableLanguagebarInFullscreen = (dwValue != 0) as BOOL;
        }
    }

    InitSkipRedrawHKLArray();

    if g_bNewLook != 0 {
        g_dwWndStyle = UIF_WINDOW_ENABLETHEMED | UIF_WINDOW_WORKAREA | UIF_WINDOW_TOOLTIP
            | UIF_WINDOW_TOOLWINDOW | UIF_WINDOW_TOPMOST;
        if g_bShowShadow != 0 {
            g_dwWndStyle |= UIF_WINDOW_SHADOW;
        }
        g_dwMenuStyle = 0x10000000 | UIF_WINDOW_MONITOR | UIF_WINDOW_SHADOW
            | UIF_WINDOW_TOOLWINDOW | UIF_WINDOW_TOPMOST;
    } else {
        g_dwWndStyle = UIF_WINDOW_WORKAREA | UIF_WINDOW_TOOLTIP | UIF_WINDOW_DLGFRAME
            | UIF_WINDOW_TOPMOST;
        g_dwMenuStyle = UIF_WINDOW_MONITOR | UIF_WINDOW_DLGFRAME | UIF_WINDOW_TOPMOST;
    }

    g_dwChildWndStyle =
        UIF_WINDOW_ENABLETHEMED | UIF_WINDOW_NOMOUSEMSG | UIF_WINDOW_TOOLTIP | UIF_WINDOW_CHILD;

    if IsBiDiLocalizedSystem() != 0 {
        g_dwWndStyle |= UIF_WINDOW_LAYOUTRTL;
        g_dwChildWndStyle |= UIF_WINDOW_LAYOUTRTL;
        g_dwMenuStyle |= UIF_WINDOW_LAYOUTRTL;
        g_fRTL = (TRUE) as i32;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn MyWaitForInputIdle(dwThreadId: DWORD, dwMilliseconds: DWORD) -> LONG {
    if !g_pTipbarWnd.is_null() && ((*g_pTipbarWnd).m_dwShowType & TF_SFT_DESKBAND as DWORD) != 0 {
        return 0 as LONG;
    }

    if TF_IsInMarshaling(dwThreadId) != 0 {
        return STATUS_TIMEOUT as LONG;
    }

    let mut dwFlags1: DWORD = 0 as DWORD;
    let mut dwFlags2: DWORD = 0 as DWORD;
    if TF_GetThreadFlags(
        dwThreadId,
        core::ptr::addr_of_mut!(dwFlags1),
        core::ptr::addr_of_mut!(dwFlags2),
        core::ptr::null_mut(),
    ) == 0
        && dwFlags2 != 0
    {
        return (0u32.wrapping_sub(1 as u32)) as LONG;
    }

    TF_CheckThreadInputIdle(dwThreadId, dwMilliseconds)
}
#[no_mangle]
pub unsafe extern "C" fn GetTipbarInternal(hWnd: HWND, dwFlags: DWORD, pDeskBand: *mut CDeskBand) -> BOOL {
    let bParent: BOOL = ((dwFlags & 0x80000000) != 0) as BOOL;
    g_bWinLogon = ((dwFlags & 0x1) != 0) as BOOL;

    InitFromReg();

    if g_bShowTipbar == 0 {
        return (FALSE) as BOOL;
    }

    if bParent != 0 {
        g_pTrayIconWnd = CTrayIconWnd__new_in_0(cpp_alloc_CicNoThrow::<CTrayIconWnd>());
        if g_pTrayIconWnd.is_null() {
            return (FALSE) as BOOL;
        }
        (*g_pTrayIconWnd).CreateWnd();
    }

    g_pTipbarWnd = CTipbarWnd__new_in_0(
        cpp_alloc_CicNoThrow::<CTipbarWnd>(),
        if bParent != 0 { g_dwWndStyle } else { g_dwChildWndStyle },
    );
    if g_pTipbarWnd.is_null() || (*g_pTipbarWnd).__base3.Initialize() == 0 {
        return (FALSE) as BOOL;
    }

    (*g_pTipbarWnd).Init((bParent == 0) as BOOL, pDeskBand);
    (*g_pTipbarWnd).__base3.CreateWnd(hWnd);

    SetWindowText((*(*g_pTipbarWnd).GetWindow()).m_hWnd, (&[0x0054, 0x0046, 0x005f, 0x0046, 0x006c, 0x006f, 0x0061, 0x0074, 0x0069, 0x006e, 0x0067, 0x004c, 0x0061, 0x006e, 0x0067, 0x0042, 0x0061, 0x0072, 0x005f, 0x0057, 0x006e, 0x0064, 0x0054, 0x0069, 0x0074, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "TF_FloatingLangBar_WndTitle" */);

    let mut dwOldStatus: DWORD = 0;
    if bParent == 0 {
        (*(*g_pTipbarWnd).m_pLangBarMgr).GetPrevShowFloatingStatus(core::ptr::addr_of_mut!(dwOldStatus));
        (*(*g_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_DESKBAND);
    }

    let mut dwStatus: DWORD = 0;
    (*(*g_pTipbarWnd).m_pLangBarMgr).GetShowFloatingStatus(core::ptr::addr_of_mut!(dwStatus));
    (*g_pTipbarWnd).ShowFloating(dwStatus);

    if bParent == 0 && (dwOldStatus & TF_SFT_DESKBAND) != 0 {
        (*g_pTipbarWnd).m_dwTipbarWndFlags |= TIPBAR_NODESKBAND;
    }

    g_hwndParent = hWnd;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetLibTls() -> *mut CIC_LIBTHREAD {
    core::ptr::addr_of_mut!(g_libTLS)
}

#[no_mangle]
pub unsafe extern "system" fn GetPopupTipbar(hWnd: HWND, fWinLogon: BOOL) -> BOOL {
    if fWinLogon == 0 {
        TurnOffSpeechIfItsOn();
    }

    return GetTipbarInternal(hWnd, (fWinLogon as DWORD) | 0x80000000, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetRegisterLangBand(bRegister: BOOL) -> HRESULT {
    if g_bEnableDeskBand == 0 || (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) == 0 {
        return E_FAIL;
    }

    let bDeskBand: BOOL = IsDeskBandFromReg();
    if bDeskBand == bRegister {
        return S_OK;
    }

    SetDeskBandToReg(bRegister);

    if RegisterComCat(
        core::ptr::addr_of!(CLSID_MSUTBDeskBand),
        core::ptr::addr_of!(CATID_DeskBand),
        bRegister,
    ) == 0
    {
        return TF_E_NOLOCK;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn ClosePopupTipbar() {
    if g_fInClosePopupTipbar != 0 {
        return;
    }

    g_fInClosePopupTipbar = (TRUE) as i32;

    if !g_pTipbarWnd.is_null() {
        (*g_pTipbarWnd).m_pDeskBand = core::ptr::null_mut();
        (*g_pTipbarWnd).DestroyWnd();
        (*g_pTipbarWnd).Release();
        g_pTipbarWnd = core::ptr::null_mut();
    }

    if !g_pTrayIconWnd.is_null() {
        (*g_pTrayIconWnd).DestroyWnd();
        CTrayIconWnd__delete(g_pTrayIconWnd);
        g_pTrayIconWnd = core::ptr::null_mut();
    }

    UninitSkipRedrawHKLArray();

    g_fInClosePopupTipbar = (FALSE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return gModule.DllRegisterServer((FALSE) as i32);
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return gModule.DllUnregisterServer((FALSE) as i32);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    // The C++ returns `gModule.DllCanUnloadNow() && (g_DllRefCount == 0)`: a bool, so S_OK (0)
    // -- "unload me" -- whenever ATL answers S_OK, even with g_DllRefCount objects alive.
    // Unloadable only when ATL holds no locks AND no msutb object is outstanding.
    if (*core::ptr::addr_of!(gModule)).DllCanUnloadNow() == S_OK && g_DllRefCount == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const CLSID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    (*core::ptr::addr_of_mut!(gModule)).DllGetClassObject(rclsid, riid, ppv)
}
#[no_mangle]
pub unsafe extern "system" fn MsUtbCoCreateInstance(rclsid: REFCLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    if IsEqualCLSID!(rclsid, core::ptr::addr_of!(CLSID_TF_CategoryMgr)) {
        return TF_CreateCategoryMgr(ppv as *mut *mut ITfCategoryMgr);
    }
    if IsEqualCLSID!(rclsid, core::ptr::addr_of!(CLSID_TF_DisplayAttributeMgr)) {
        return TF_CreateDisplayAttributeMgr(ppv as *mut *mut ITfDisplayAttributeMgr);
    }
    return cicRealCoCreateInstance(rclsid, (pUnkOuter).cast(), dwClsContext, iid, ppv);
}

#[no_mangle]
pub unsafe extern "C" fn TFUninitLib() {}
#[no_mangle]
pub unsafe extern "C" fn ProcessAttach(hinstDLL: HINSTANCE) -> BOOL {
    InitializeCriticalSectionAndSpinCount(core::ptr::addr_of_mut!(g_cs), 0);

    g_hInst = hinstDLL;

    cicGetOSInfo(core::ptr::addr_of_mut!(g_uACP), core::ptr::addr_of_mut!(g_dwOSInfo));

    // cicero's island spells LPUNKNOWN with its own (layout-identical) IUnknown: same ABI.
    TFInitLib(core::mem::transmute::<unsafe extern "system" fn(REFCLSID, LPUNKNOWN, DWORD, REFIID, *mut LPVOID) -> HRESULT, cicero_rust::prelude::FN_CoCreateInstance>(MsUtbCoCreateInstance));
    cicInitUIFLib();

    CTrayIconWnd::RegisterClassW(); // CTrayIconWnd::RegisterClass (UNICODE build)

    g_wmTaskbarCreated = RegisterWindowMessageW(CPP_COM_WSTR_A83D1779AB13.as_ptr());

    gModule.Init(core::ptr::addr_of_mut!(ObjectMap) as *mut _, hinstDLL, core::ptr::null_mut());
    DisableThreadLibraryCalls(hinstDLL);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessDetach(hinstDLL: HINSTANCE) {
    cicDoneUIFLib();
    TFUninitLib();
    DeleteCriticalSection(core::ptr::addr_of_mut!(g_cs));
    gModule.Term();
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    __cxx_global_init();
    if dwReason == DLL_PROCESS_ATTACH {
        {
            if !(ProcessAttach(hinstDLL) != 0) {
                ProcessDetach(hinstDLL);
                return (FALSE) as BOOL;
            }
        }
    } else if dwReason == DLL_PROCESS_DETACH {
        {
            ProcessDetach(hinstDLL);
        }
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    // gModule.__ctor_0(): C++ constructor call on the file-scope global `gModule`.
    (*core::ptr::addr_of_mut!(gModule)).__ctor_0();
}

impl LANGBARITEMSTATE {
    pub unsafe fn IsShown(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_dwDemoteLevel < 2) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CMsUtbModule {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}

impl CShellWndThread {
    pub unsafe fn GetWndTray(&mut self) -> HWND {
        let this_: *mut Self = self;
        if (*this_).m_hTrayWnd.is_null() || ((IsWindow((*this_).m_hTrayWnd)) == 0) {
            (*this_).m_hTrayWnd = FindWindowW(CPP_COM_WSTR_BB39C012AB62.as_ptr(), core::ptr::null_mut());
        }
        return (*this_).m_hTrayWnd;
    }
    pub unsafe fn GetWndProgman(&mut self) -> HWND {
        let this_: *mut Self = self;
        if (*this_).m_hProgmanWnd.is_null() || IsWindow((*this_).m_hProgmanWnd) == 0 {
            (*this_).m_hProgmanWnd = FindWindowW(CPP_COM_WSTR_7289CE32E451.as_ptr(), core::ptr::null_mut());
        }
        return (*this_).m_hProgmanWnd;
    }
    pub unsafe fn clear(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hProgmanWnd = core::ptr::null_mut();
        (*this_).m_hTrayWnd = core::ptr::null_mut();
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hTrayWnd = core::ptr::null_mut();
        (*this_).m_hProgmanWnd = core::ptr::null_mut();
    }
}

impl CUTBLangBarDlg {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn GetThis(hDlg: HWND) -> *mut CUTBLangBarDlg {
        return GetWindowLongPtr(hDlg, DWLP_USER) as *mut CUTBLangBarDlg;
    }
    pub unsafe fn SetThis(hDlg: HWND, pThis: *mut CUTBLangBarDlg) {
        SetWindowLongPtr(hDlg, DWLP_USER, (pThis as isize) as _);
    }
    pub unsafe extern "system" fn s_ThreadProc(pParam: LPVOID) -> DWORD {
        return ((*(pParam as *mut CUTBLangBarDlg)).ThreadProc()) as DWORD;
    }
    pub unsafe extern "system" fn DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
        if uMsg == WM_INITDIALOG {
            CUTBLangBarDlg::SetThis(hDlg, lParam as *mut CUTBLangBarDlg);
            ShowWindow(hDlg, (SW_RESTORE as i32));
            UpdateWindow(hDlg);
            return (TRUE) as INT_PTR;
        }
    
        if uMsg == WM_COMMAND {
            let pThis: *mut CUTBLangBarDlg = CUTBLangBarDlg::GetThis(hDlg);
            (*pThis).OnCommand(hDlg, wParam, lParam);
            return (TRUE) as INT_PTR;
        }
    
        return (FALSE) as INT_PTR;
    }
    pub unsafe fn StartThread(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).IsDlgShown() != 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).SetDlgShown((TRUE) as BOOL);
    
        let mut dwThreadId: DWORD = 0;
        let hThread: HANDLE = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(CUTBLangBarDlg::s_ThreadProc),
            this_ as LPVOID,
            0,
            core::ptr::addr_of_mut!(dwThreadId),
        );
        if hThread.is_null() {
            (*this_).SetDlgShown((FALSE) as BOOL);
            return (TRUE) as BOOL;
        }
    
        (*this_).m_cRefs += 1;
        CloseHandle(hThread);
        return (TRUE) as BOOL;
    }
    pub unsafe fn _Release(&mut self) -> LONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            CUTBLangBarDlg__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs;
    }
    pub unsafe fn ThreadProc__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        DialogBoxParam(g_hInst, (*this_).m_pszDialogName, core::ptr::null_mut(), Some(CUTBLangBarDlg::DlgProc), this_ as LPARAM);
        (*this_).SetDlgShown((FALSE) as BOOL);
        (*this_)._Release();
        return (TRUE) as BOOL;
    }
}

impl CUTBCloseLangBarDlg {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).__base0.m_cRefs = 1;
    
            if !((g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) != 0) {
                (*this_).__base0.m_pszDialogName = MAKEINTRESOURCE!(IDD_CLOSELANGBARNOBAND);
            } else {
                (*this_).__base0.m_pszDialogName = MAKEINTRESOURCE!(IDD_CLOSELANGBAR);
            }
        }
    }
    pub unsafe fn DoModal__body(&mut self, hDlg: HWND) -> BOOL {
        let this_: *mut Self = self;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        let mut error: LSTATUS;
        let mut dwValue: DWORD = FALSE;
        error = regKey.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\" */, KEY_READ);
        if (error) as u32 == ERROR_SUCCESS {
            regKey.QueryDword((&[0x0044, 0x006f, 0x006e, 0x0074, 0x0053, 0x0068, 0x006f, 0x0077, 0x0043, 0x006c, 0x006f, 0x0073, 0x0065, 0x004c, 0x0061, 0x006e, 0x0067, 0x0042, 0x0061, 0x0072, 0x0044, 0x006c, 0x0067, 0x0000u16]).as_ptr() /* TEXT "DontShowCloseLangBarDlg" */, core::ptr::addr_of_mut!(dwValue));
        }
    
        if dwValue != 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).__base0.StartThread();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnCommand__body(&mut self, hDlg: HWND, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        match (LOWORD!(wParam)) as u32 {
            IDOK => {
                DoCloseLangbar();
                if IsDlgButtonChecked(hDlg, (IDC_CLOSELANGBAR_CHECK as i32)) != 0 {
                    let mut regKey: CicRegKey = CicRegKey__zeroed();
                    regKey.__ctor_0();
                    let mut error: LSTATUS;
                    error = regKey.Create(
                        HKEY_CURRENT_USER,
                        (['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'M' as u16, 'S' as u16, 'U' as u16, 'T' as u16, 'B' as u16, '\\' as u16, 0u16].as_ptr()),
                    );
                    if (error) as u32 == ERROR_SUCCESS {
                        regKey.SetDword(
                            (['D' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'S' as u16, 'h' as u16, 'o' as u16, 'w' as u16, 'C' as u16, 'l' as u16, 'o' as u16, 's' as u16, 'e' as u16, 'L' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 'D' as u16, 'l' as u16, 'g' as u16, 0u16].as_ptr()),
                            TRUE,
                        );
                    }
                }
                EndDialog(hDlg, (TRUE) as INT_PTR);
            }
    
            IDCANCEL => {
                EndDialog(hDlg, (FALSE) as INT_PTR);
            }
    
            _ => {
                return (FALSE) as BOOL;
            }
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn IsDlgShown__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return CUTBCloseLangBarDlg__s_bIsDlgShown;
    }
    pub unsafe fn SetDlgShown__body(&mut self, bShown: BOOL) {
        let this_: *mut Self = self;
        CUTBCloseLangBarDlg__s_bIsDlgShown = bShown;
    }
}

impl CUTBMinimizeLangBarDlg {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).__base0.m_cRefs = 1;
            if (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) == 0 {
                (*this_).__base0.m_pszDialogName = MAKEINTRESOURCE!(IDD_MINIMIZELANGBARNOBAND);
            } else {
                (*this_).__base0.m_pszDialogName = MAKEINTRESOURCE!(IDD_MINIMIZELANGBAR);
            }
        }
    }
    pub unsafe fn DoModal__body(&mut self, hDlg: HWND) -> BOOL {
        let this_: *mut Self = self;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        let mut error: LSTATUS;
    
        let mut dwValue: DWORD = FALSE;
        error = regKey.Open(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x004d, 0x0053, 0x0055, 0x0054, 0x0042, 0x005c, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\MSUTB\\" */, KEY_READ);
        if (error) as u32 == ERROR_SUCCESS {
            regKey.QueryDword((&[0x0044, 0x006f, 0x006e, 0x0074, 0x0053, 0x0068, 0x006f, 0x0077, 0x004d, 0x0069, 0x006e, 0x0069, 0x006d, 0x0069, 0x007a, 0x0065, 0x004c, 0x0061, 0x006e, 0x0067, 0x0042, 0x0061, 0x0072, 0x0044, 0x006c, 0x0067, 0x0000u16]).as_ptr() /* TEXT "DontShowMinimizeLangBarDlg" */, core::ptr::addr_of_mut!(dwValue));
        }
    
        if dwValue != 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).__base0.StartThread();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnCommand__body(&mut self, hDlg: HWND, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        static SOFTWARE_CTF_MSUTB: [u16; 30] = ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'M' as u16, 'S' as u16, 'U' as u16, 'T' as u16, 'B' as u16, '\\' as u16, 0];
        static DONT_SHOW_VALUE: [u16; 27] = ['D' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'S' as u16, 'h' as u16, 'o' as u16, 'w' as u16, 'M' as u16, 'i' as u16, 'n' as u16, 'i' as u16, 'm' as u16, 'i' as u16, 'z' as u16, 'e' as u16, 'L' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 'D' as u16, 'l' as u16, 'g' as u16, 0];
        match (LOWORD!(wParam)) as u32 {
            IDOK => {
                if IsDlgButtonChecked(hDlg, (IDC_MINIMIZELANGBAR_CHECK as i32)) != 0 {
                    let mut error: LSTATUS;
                    let mut regKey: CicRegKey = CicRegKey__zeroed();
                    regKey.__ctor_0();
                    error = regKey.Create(HKEY_CURRENT_USER, (SOFTWARE_CTF_MSUTB.as_ptr()));
                    if (error) as u32 == ERROR_SUCCESS {
                        regKey.SetDword((DONT_SHOW_VALUE.as_ptr()), TRUE);
                    }
                }
                EndDialog(hDlg, (TRUE) as INT_PTR);
            }
            IDCANCEL => {
                EndDialog(hDlg, (FALSE) as INT_PTR);
            }
            _ => {
                return (FALSE) as BOOL;
            }
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn IsDlgShown__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return CUTBCloseLangBarDlg__s_bIsDlgShown;
    }
    pub unsafe fn SetDlgShown__body(&mut self, bShown: BOOL) {
        let this_: *mut Self = self;
        CUTBCloseLangBarDlg__s_bIsDlgShown = bShown;
    }
    pub unsafe fn ThreadProc__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        Sleep(700);
        return (*this_).__base0.ThreadProc();
    }
}

impl CCicLibMenu {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_MenuItems.size() {
            let _ = CCicLibMenuItem__delete((*this_).m_MenuItems[iItem]);
            (*this_).m_MenuItems[iItem] = core::ptr::null_mut();
            iItem += 1;
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 2] = [
            QITAB {
                piid: &IID_ITfMenu as *const IID,
                dwOffset: core::mem::offset_of!(CCicLibMenu, __base0) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs += 1;
        (*this_).m_cRefs as ULONG
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            CCicLibMenu__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn AddMenuItem(&mut self, uId: UINT, dwFlags: DWORD, hbmp: HBITMAP, hbmpMask: HBITMAP, pch: *const WCHAR, cch: ULONG, ppSubMenu: *mut *mut ITfMenu) -> HRESULT {
        let this_: *mut Self = self;
        if !ppSubMenu.is_null() {
            *ppSubMenu = core::ptr::null_mut();
        }
    
        let mut pSubMenu: *mut CCicLibMenu = core::ptr::null_mut();
        if dwFlags & TF_LBMENUF_SUBMENU != 0 {
            if ppSubMenu.is_null() {
                return E_INVALIDARG;
            }
            pSubMenu = (*this_).CreateSubMenu();
        }
    
        let pMenuItem: *mut CCicLibMenuItem = (*this_).CreateMenuItem();
        if pMenuItem.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if (*pMenuItem).Init(uId, dwFlags, hbmp, hbmpMask, pch, cch, (&mut (*pSubMenu).__base0) as *mut ITfMenu) == 0 {
            return E_FAIL;
        }
    
        if !ppSubMenu.is_null() && !pSubMenu.is_null() {
            *ppSubMenu = (&mut (*pSubMenu).__base0) as *mut ITfMenu;
            (*pSubMenu).AddRef();
        }
    
        (*this_).m_MenuItems.Add(pMenuItem);
        return S_OK;
    }
    pub unsafe fn CreateSubMenu__body(&mut self) -> *mut CCicLibMenu {
        let this_: *mut Self = self;
        return CCicLibMenu__new_in_0(cpp_alloc_CicNoThrow::<CCicLibMenu>());
    }
    pub unsafe fn CreateMenuItem__body(&mut self) -> *mut CCicLibMenuItem {
        let this_: *mut Self = self;
        CCicLibMenuItem__new_in_0(cpp_alloc_CicNoThrow::<CCicLibMenuItem>())
    }
}

impl CCicLibMenuItem {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_uId = 0;
            (*this_).m_dwFlags = 0;
            (*this_).m_hbmp = core::ptr::null_mut();
            (*this_).m_hbmpMask = core::ptr::null_mut();
            (*this_).m_bstrText = core::ptr::null_mut();
            (*this_).m_pMenu = core::ptr::null_mut();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pMenu.is_null() {
            (*(*this_).m_pMenu).Release();
            (*this_).m_pMenu = core::ptr::null_mut();
        }
    
        if !(*this_).m_hbmp.is_null() {
            DeleteObject(((*this_).m_hbmp).cast());
            (*this_).m_hbmp = core::ptr::null_mut();
        }
    
        if !(*this_).m_hbmpMask.is_null() {
            DeleteObject(((*this_).m_hbmpMask).cast());
            (*this_).m_hbmpMask = core::ptr::null_mut();
        }
    
        SysFreeString((*this_).m_bstrText);
        (*this_).m_bstrText = core::ptr::null_mut();
    }
    pub unsafe fn Init(&mut self, uId: UINT, dwFlags: DWORD, hbmp: HBITMAP, hbmpMask: HBITMAP, pch: *const WCHAR, cch: ULONG, pMenu: *mut ITfMenu) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_uId = uId;
        (*this_).m_dwFlags = dwFlags;
        (*this_).m_bstrText = SysAllocStringLen(pch, cch);
        if (*this_).m_bstrText.is_null() && cch != 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).m_pMenu = pMenu;
        (*this_).m_hbmp = (*this_).CreateBitmap((hbmp).cast());
        (*this_).m_hbmpMask = (*this_).CreateBitmap((hbmpMask).cast());
        if !hbmp.is_null() {
            DeleteObject((hbmp).cast());
        }
        if !hbmpMask.is_null() {
            DeleteObject((hbmpMask).cast());
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn CreateBitmap(&mut self, hBitmap: HANDLE) -> HBITMAP {
        let this_: *mut Self = self;
        if hBitmap.is_null() {
            return core::ptr::null_mut();
        }
    
        let hDC: HDC = CreateDC((['D' as u16, 'I' as u16, 'S' as u16, 'P' as u16, 'L' as u16, 'A' as u16, 'Y' as u16, 0u16].as_ptr()), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if hDC.is_null() {
            return core::ptr::null_mut();
        }
    
        let mut hbmMem: HBITMAP = core::ptr::null_mut();
    
        let mut bm: BITMAP = core::mem::zeroed();
        GetObject(hBitmap, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);
    
        let mut hbmOld1: HGDIOBJ = core::ptr::null_mut();
        let hdcMem1: HDC = CreateCompatibleDC(hDC);
        if !hdcMem1.is_null() {
            hbmOld1 = SelectObject(hdcMem1, hBitmap);
        }
    
        let mut hbmOld2: HGDIOBJ = core::ptr::null_mut();
        let hdcMem2: HDC = CreateCompatibleDC(hDC);
        if !hdcMem2.is_null() {
            hbmMem = CreateCompatibleBitmap(hDC, bm.bmWidth, bm.bmHeight);
            hbmOld2 = SelectObject(hdcMem2, hbmMem as HGDIOBJ);
        }
    
        BitBlt(hdcMem2, 0, 0, bm.bmWidth, bm.bmHeight, hdcMem1, 0, 0, SRCCOPY);
    
        if !hbmOld1.is_null() {
            SelectObject(hdcMem1, hbmOld1);
        }
        if !hbmOld2.is_null() {
            SelectObject(hdcMem2, hbmOld2);
        }
    
        DeleteDC(hDC);
        if !hdcMem1.is_null() {
            DeleteDC(hdcMem1);
        }
        if !hdcMem2.is_null() {
            DeleteDC(hdcMem2);
        }
    
        hbmMem
    }
}

impl CTipbarAccessible {
    pub unsafe fn __ctor_0(&mut self, pItem: *mut CTipbarAccItem) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_cRefs = 1;
            (*this_).m_hWnd = core::ptr::null_mut();
            (*this_).m_pTypeInfo = core::ptr::null_mut();
            (*this_).m_pStdAccessible = core::ptr::null_mut();
            (*this_).m_bInitialized = (FALSE) as i32;
            (*this_).m_cSelection = 1;
            (*this_).m_AccItems.Add(pItem);
            g_DllRefCount += 1;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_pTypeInfo = (*this_).m_pTypeInfo;
        if !(*this_).m_pTypeInfo.is_null() {
            (*(*this_).m_pTypeInfo).Release();
            (*this_).m_pTypeInfo = core::ptr::null_mut();
        }
        if !(*this_).m_pStdAccessible.is_null() {
            (*(*this_).m_pStdAccessible).Release();
            (*this_).m_pStdAccessible = core::ptr::null_mut();
        }
        InterlockedDecrement(core::ptr::addr_of_mut!(g_DllRefCount));
    }
    pub unsafe fn Initialize(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_bInitialized = (TRUE) as i32;
    
        let mut hr: HRESULT = CreateStdAccessibleObject((*this_).m_hWnd, (OBJID_CLIENT) as LONG, &IID_IAccessible,
                                                 core::ptr::addr_of_mut!((*this_).m_pStdAccessible) as *mut *mut core::ffi::c_void);
        if hr < 0 {
            return hr;
        }
    
        let mut pTypeLib: *mut ITypeLib = core::ptr::null_mut();
        hr = LoadRegTypeLib(&LIBID_Accessibility, 1, 0, 0, core::ptr::addr_of_mut!(pTypeLib));
        if hr < 0 {
            hr = LoadTypeLib(CPP_COM_WSTR_FB3ED1CCBECC.as_ptr(), core::ptr::addr_of_mut!(pTypeLib));
        }
    
        if hr >= 0 {
            hr = (*pTypeLib).GetTypeInfoOfGuid(&IID_IAccessible, core::ptr::addr_of_mut!((*this_).m_pTypeInfo));
            (*pTypeLib).Release();
        }
    
        hr
    }
    pub unsafe fn AddAccItem(&mut self, pItem: *mut CTipbarAccItem) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).m_AccItems.Add(pItem);
    }
    pub unsafe fn RemoveAccItem(&mut self, pItem: *mut CTipbarAccItem) -> HRESULT {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_AccItems.size() {
            if (*this_).m_AccItems[iItem] == pItem {
                (*this_).m_AccItems.Remove(iItem, 1);
                break;
            }
            iItem += 1;
        }
        return S_OK;
    }
    pub unsafe fn ClearAccItems(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_AccItems.clear();
    }
    pub unsafe fn AccItemFromID(&mut self, iItem: INT) -> *mut CTipbarAccItem {
        let this_: *mut Self = self;
        if iItem < 0 || ((*this_).m_AccItems.size() as INT) <= iItem {
            return core::ptr::null_mut();
        }
        return (*this_).m_AccItems[iItem as usize];
    }
    pub unsafe fn GetIDOfItem(&mut self, pTarget: *mut CTipbarAccItem) -> INT {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_AccItems.size() {
            if pTarget == (*this_).m_AccItems[iItem] {
                return iItem as INT;
            }
            iItem += 1;
        }
        return -(1);
    }
    pub unsafe fn CreateRefToAccObj(&mut self, wParam: WPARAM) -> LONG_PTR {
        let this_: *mut Self = self;
        return LresultFromObject(&IID_IAccessible, wParam, (this_ as LPVOID).cast()) as LONG_PTR;
    }
    pub unsafe fn DoDefaultActionReal(&mut self, nID: INT) -> BOOL {
        let this_: *mut Self = self;
        let mut pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(nID);
        if pItem.is_null() {
            return (FALSE) as BOOL;
        }
        return (*pItem).DoAccDefaultActionReal();
    }
    pub unsafe fn NotifyWinEvent(&mut self, event: DWORD, pItem: *mut CTipbarAccItem) {
        let this_: *mut Self = self;
        let nID: INT = (*this_).GetIDOfItem(pItem);
        if nID < 0 {
            return;
        }
    
        NotifyWinEvent(event, (*this_).m_hWnd, -4, nID);
    }
    pub unsafe fn SetWindow(&mut self, hWnd: HWND,
    ) {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWnd;
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 3] = [
            QITAB { piid: &IID_IDispatch as *const IID, dwOffset: core::mem::offset_of!(CTipbarAccessible, __base0) as DWORD },
            QITAB { piid: &IID_IAccessible as *const IID, dwOffset: core::mem::offset_of!(CTipbarAccessible, __base0) as DWORD },
            QITAB { piid: core::ptr::null(), dwOffset: 0 },
        ];
        return QISearch(this_ as *mut core::ffi::c_void, c_tab.as_ptr(), riid as *const IID, ppvObject);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return (InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs))) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let count: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs));
        if count == 0 {
            CTipbarAccessible__delete(this_);
            return 0;
        }
        return count as ULONG;
    }
    pub unsafe fn GetTypeInfoCount(&mut self, pctinfo: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        if pctinfo.is_null() {
            return E_INVALIDARG;
        }
        *pctinfo = ((*this_).m_pTypeInfo.is_null()) as UINT;
        return S_OK;
    }
    pub unsafe fn GetTypeInfo(&mut self, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
        let this_: *mut Self = self;
        if ppTInfo.is_null() {
            return E_INVALIDARG;
        }
        *ppTInfo = core::ptr::null_mut();
        if iTInfo != 0 {
            return TYPE_E_ELEMENTNOTFOUND;
        }
        if (*this_).m_pTypeInfo.is_null() {
            return E_NOTIMPL;
        }
        *ppTInfo = (*this_).m_pTypeInfo;
        (*(*this_).m_pTypeInfo).AddRef();
        return S_OK;
    }
    pub unsafe fn GetIDsOfNames(&mut self, riid: &IID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pTypeInfo.is_null() {
            return E_NOTIMPL;
        }
        return (*(*this_).m_pTypeInfo).GetIDsOfNames(rgszNames, cNames, rgDispId);
    }
    pub unsafe fn Invoke(&mut self, dispIdMember: DISPID, riid: &IID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pTypeInfo.is_null() {
            return E_NOTIMPL;
        }
        return (*(*this_).m_pTypeInfo).Invoke(this_ as *mut core::ffi::c_void,
                                   dispIdMember,
                                   wFlags,
                                   pDispParams,
                                   pVarResult,
                                   pExcepInfo,
                                   puArgErr);
    }
    pub unsafe fn get_accParent(&mut self, ppdispParent: *mut *mut IDispatch) -> HRESULT {
        let this_: *mut Self = self;
        return (*(*this_).m_pStdAccessible).get_accParent(ppdispParent);
    }
    pub unsafe fn get_accChildCount(&mut self, pcountChildren: *mut LONG) -> HRESULT {
        let this_: *mut Self = self;
        if pcountChildren.is_null() {
            return E_INVALIDARG;
        }
        let cItems: INT = (*this_).m_AccItems.size() as INT;
        if cItems == 0 {
            return E_FAIL;
        }
        *pcountChildren = cItems - 1;
        return S_OK;
    }
    pub unsafe fn get_accChild(&mut self, varChildID: VARIANT, ppdispChild: *mut *mut IDispatch) -> HRESULT {
        let this_: *mut Self = self;
        if ppdispChild.is_null() {
            return E_INVALIDARG;
        }
        *ppdispChild = core::ptr::null_mut();
        return S_FALSE;
    }
    pub unsafe fn get_accName(&mut self, varID: VARIANT, pszName: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if pszName.is_null() {
            return E_INVALIDARG;
        }
        let mut varID = varID;
        let pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        if pItem.is_null() {
            return E_INVALIDARG;
        }
        *pszName = (*pItem).GetAccName();
        if (*pszName).is_null() {
            return DISP_E_MEMBERNOTFOUND;
        }
        return S_OK;
    }
    pub unsafe fn get_accValue(&mut self, mut varID: VARIANT, pszValue: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if pszValue.is_null() {
            return E_INVALIDARG;
        }
        let pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        if pItem.is_null() {
            return E_INVALIDARG;
        }
        *pszValue = (*pItem).GetAccValue();
        if (*pszValue).is_null() {
            return DISP_E_MEMBERNOTFOUND;
        }
        return S_OK;
    }
    pub unsafe fn get_accDescription(&mut self, varID: VARIANT, description: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if description.is_null() {
            return E_INVALIDARG;
        }
        return (*(*this_).m_pStdAccessible).get_accDescription(varID, description);
    }
    pub unsafe fn get_accRole(&mut self, mut varID: VARIANT, role: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if role.is_null() {
            return E_INVALIDARG;
        }
        let pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        if pItem.is_null() {
            return E_INVALIDARG;
        }
        V_VT!(role) = (VT_I4 as u16);
        V_I4!(role) = (*pItem).GetAccRole();
        return S_OK;
    }
    pub unsafe fn get_accState(&mut self, varID: VARIANT, state: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if state.is_null() {
            return E_INVALIDARG;
        }
        let mut pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(&varID));
        if pItem.is_null() {
            return E_INVALIDARG;
        }
        V_VT!(state) = (VT_I4 as u16);
        V_I4!(state) = (*pItem).GetAccState();
        return S_OK;
    }
    pub unsafe fn get_accHelp(&mut self, varID: VARIANT,
        help: *mut BSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        DISP_E_MEMBERNOTFOUND
    }
    pub unsafe fn get_accHelpTopic(&mut self, helpfile: *mut BSTR,
        varID: VARIANT,
        pidTopic: *mut LONG,
    ) -> HRESULT {
        let this_: *mut Self = self;
        DISP_E_MEMBERNOTFOUND
    }
    pub unsafe fn get_accKeyboardShortcut(&mut self, varID: VARIANT,
        shortcut: *mut BSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        DISP_E_MEMBERNOTFOUND
    }
    pub unsafe fn get_accFocus(&mut self, pvarID: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pvarID.is_null() {
            return E_INVALIDARG;
        }
        V_VT!(pvarID) = VT_EMPTY as VARTYPE;
        return S_FALSE;
    }
    pub unsafe fn get_accSelection(&mut self, pvarID: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pvarID.is_null() {
            return E_INVALIDARG;
        }
    
        V_VT!(pvarID) = VT_EMPTY;
    
        let cItems: INT = (*this_).m_AccItems.size() as INT;
        if cItems < (*this_).m_cSelection {
            return S_FALSE;
        }
    
        if cItems > (*this_).m_cSelection {
            V_VT!(pvarID) = (VT_I4 as u16);
            V_I4!(pvarID) = (*this_).m_cSelection;
        }
    
        return S_OK;
    }
    pub unsafe fn get_accDefaultAction(&mut self, mut varID: VARIANT, action: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if action.is_null() {
            return E_INVALIDARG;
        }
        *action = core::ptr::null_mut();
    
        if V_VT!(core::ptr::addr_of_mut!(varID)) != (VT_I4 as u16) {
            return E_INVALIDARG;
        }
    
        let mut pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        if pItem.is_null() {
            return DISP_E_MEMBERNOTFOUND;
        }
        *action = (*pItem).GetAccDefaultAction();
        if (*action).is_null() {
            return S_FALSE;
        }
        return S_OK;
    }
    pub unsafe fn accSelect(&mut self, flagsSelect: LONG, mut varID: VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if (flagsSelect & (SELFLAG_ADDSELECTION as i32)) != 0 && (flagsSelect & (SELFLAG_REMOVESELECTION as i32)) != 0 {
            return E_INVALIDARG;
        }
        if (flagsSelect & (SELFLAG_TAKEFOCUS | SELFLAG_ADDSELECTION | SELFLAG_EXTENDSELECTION) as LONG) != 0 {
            return S_FALSE;
        }
        if (flagsSelect & (SELFLAG_REMOVESELECTION as i32)) != 0 {
            return S_OK;
        }
        if V_VT!(core::ptr::addr_of_mut!(varID)) != (VT_I4 as u16) {
            return E_INVALIDARG;
        }
        if (flagsSelect & (SELFLAG_TAKESELECTION as i32)) != 0 {
            (*this_).m_cSelection = V_I4!(core::ptr::addr_of_mut!(varID));
            return S_OK;
        }
        return S_FALSE;
    }
    pub unsafe fn accLocation(&mut self, left: *mut LONG, top: *mut LONG, width: *mut LONG, height: *mut LONG, mut varID: VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if left.is_null() || top.is_null() || width.is_null() || height.is_null() {
            return E_INVALIDARG;
        }
    
        if !(V_I4!(core::ptr::addr_of_mut!(varID)) != 0) {
            return (*(*this_).m_pStdAccessible).accLocation(left, top, width, height, varID);
        }
    
        let mut rc: RECT = core::mem::zeroed();
        let pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        (*pItem).GetAccLocation(core::ptr::addr_of_mut!(rc));
    
        *left = rc.left;
        *top = rc.top;
        *width = rc.right - rc.left;
        *height = rc.bottom - rc.top;
        return S_OK;
    }
    pub unsafe fn accNavigate(&mut self, dir: LONG, mut varStart: VARIANT, pvarEnd: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_AccItems.size() <= 1 {
            V_VT!(pvarEnd) = VT_EMPTY;
            return S_OK;
        }
    
        match (dir) as u32 {
            _ if (dir) as u32 == NAVDIR_UP || (dir) as u32 == NAVDIR_LEFT || (dir) as u32 == NAVDIR_PREVIOUS => {
                V_VT!(pvarEnd) = (VT_I4 as u16);
                V_I4!(pvarEnd) = V_I4!(core::ptr::addr_of_mut!(varStart)) - 1;
                if V_I4!(core::ptr::addr_of_mut!(varStart)) - 1 <= 0 {
                    V_I4!(pvarEnd) = ((*this_).m_AccItems.size() - 1) as INT;
                }
                return S_OK;
            }
    
            _ if (dir) as u32 == NAVDIR_DOWN || (dir) as u32 == NAVDIR_RIGHT || (dir) as u32 == NAVDIR_NEXT => {
                V_VT!(pvarEnd) = (VT_I4 as u16);
                V_I4!(pvarEnd) = V_I4!(core::ptr::addr_of_mut!(varStart)) + 1;
                if ((*this_).m_AccItems.size() as INT) <= V_I4!(core::ptr::addr_of_mut!(varStart)) + 1 {
                    V_I4!(pvarEnd) = 1;
                }
                return S_OK;
            }
    
            _ if (dir) as u32 == NAVDIR_FIRSTCHILD => {
                V_VT!(pvarEnd) = (VT_I4 as u16);
                V_I4!(pvarEnd) = 1;
                return S_OK;
            }
    
            _ if (dir) as u32 == NAVDIR_LASTCHILD => {
                V_VT!(pvarEnd) = (VT_I4 as u16);
                V_I4!(pvarEnd) = ((*this_).m_AccItems.size() - 1) as INT;
                return S_OK;
            }
    
            _ => {}
        }
    
        V_VT!(pvarEnd) = VT_EMPTY;
        return S_OK;
    }
    pub unsafe fn accHitTest(&mut self, left: LONG, top: LONG, pvarID: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pvarID.is_null() {
            return E_INVALIDARG;
        }
        let mut Point: POINT = POINT { x: left, y: top };
        let mut Rect: RECT = core::mem::zeroed();
        ScreenToClient((*this_).m_hWnd, core::ptr::addr_of_mut!(Point));
        GetClientRect((*this_).m_hWnd, core::ptr::addr_of_mut!(Rect));
    
        if PtInRect(core::ptr::addr_of_mut!(Rect), Point) == 0 {
            V_VT!(pvarID) = VT_EMPTY;
            return S_OK;
        }
    
        V_VT!(pvarID) = (VT_I4 as u16);
        V_I4!(pvarID) = 0;
    
        let mut iItem: usize = 1;
        while iItem < (*this_).m_AccItems.size() {
            let pItem: *mut CTipbarAccItem = (*this_).m_AccItems[iItem as size_t];
            if !pItem.is_null() {
                (*pItem).GetAccLocation(core::ptr::addr_of_mut!(Rect));
                if PtInRect(core::ptr::addr_of_mut!(Rect), Point) != 0 {
                    V_I4!(pvarID) = iItem as LONG;
                    break;
                }
            }
            iItem += 1;
        }
    
        S_OK
    }
    pub unsafe fn accDoDefaultAction(&mut self, mut varID: VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if V_VT!(core::ptr::addr_of_mut!(varID)) != (VT_I4 as u16) {
            return E_INVALIDARG;
        }
        let pItem: *mut CTipbarAccItem = (*this_).AccItemFromID(V_I4!(core::ptr::addr_of_mut!(varID)));
        if pItem.is_null() {
            return DISP_E_MEMBERNOTFOUND;
        }
        if (*pItem).DoAccDefaultAction() != 0 {
            return S_OK;
        } else {
            return S_FALSE;
        }
    }
    pub unsafe fn put_accName(&mut self, varID: VARIANT,
        name: BSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_FALSE
    }
    pub unsafe fn put_accValue(&mut self, varID: VARIANT,
        value: BSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_FALSE
    }
}

impl CTipbarAccItem {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn GetAccName__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        return SysAllocString(CPP_COM_WSTR_96A296D224F2.as_ptr());
    }
    pub unsafe fn GetAccValue__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        (NULL).cast()
    }
    pub unsafe fn GetAccRole__body(&mut self) -> INT {
        let this_: *mut Self = self;
        10
    }
    pub unsafe fn GetAccState__body(&mut self) -> INT {
        let this_: *mut Self = self;
        256
    }
    pub unsafe fn GetAccLocation__body(&mut self, lprc: LPRECT) {
        let this_: *mut Self = self;
        let _ = this_;
        *lprc = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
    }
    pub unsafe fn GetAccDefaultAction__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        (NULL).cast()
    }
    pub unsafe fn DoAccDefaultAction__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn DoAccDefaultActionReal__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
}

impl CTipbarCoInitialize {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_bCoInit = (FALSE) as i32;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).CoUninit();
    }
    pub unsafe fn EnsureCoInit(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_bCoInit != 0 {
            return S_OK;
        }
        let mut hr: HRESULT = CoInitialize(core::ptr::null_mut());
        if hr < 0 {
            return hr;
        }
        (*this_).m_bCoInit = (TRUE) as i32;
        return S_OK;
    }
    pub unsafe fn CoUninit(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bCoInit != 0 {
            CoUninitialize();
            (*this_).m_bCoInit = (FALSE) as i32;
        }
    }
}

impl CUTBMenuWnd {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE, style: DWORD, dwUnknown14: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn StartDoAccDefaultActionTimer(&mut self, pTarget: *mut CUTBMenuItem) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pAccessible.is_null() {
            return (FALSE) as BOOL;
        }
    
        (*this_).m_nMenuWndID = ((*(*this_).m_pAccessible).GetIDOfItem(core::ptr::addr_of_mut!((*pTarget).__base0))) as u32;
        if (*this_).m_nMenuWndID == 0 || (*this_).m_nMenuWndID == (1 as UINT).wrapping_neg() {
            return (FALSE) as BOOL;
        }
    
        if IsWindow((*this_).__base1.m_hWnd) != 0 {
            KillTimer((*this_).__base1.m_hWnd, TIMER_ID_DOACCDEFAULTACTION as UINT_PTR);
            SetTimer((*this_).__base1.m_hWnd, TIMER_ID_DOACCDEFAULTACTION as UINT_PTR, g_uTimerElapseDOACCDEFAULTACTION, None);
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn GetAccItem(&mut self) -> *mut CTipbarAccItem {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base0);
    }
    pub unsafe fn GetMenu(&mut self) -> *mut CUIFMenu {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1);
    }
    pub unsafe fn GetAccName__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        let mut szText: [WCHAR; 64] = [0; 64];
        LoadStringW(g_hInst, IDS_MENUWND, szText.as_mut_ptr(), _countof!(szText) as i32);
        return SysAllocString(szText.as_ptr());
    }
    pub unsafe fn GetAccRole__body(&mut self) -> INT {
        let this_: *mut Self = self;
        9
    }
    pub unsafe fn Initialize__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let pAccessible: *mut CTipbarAccessible = CTipbarAccessible__new_in_0(cpp_alloc_CicNoThrow::<CTipbarAccessible>(), (*this_).GetAccItem());
        if !pAccessible.is_null() {
            (*this_).m_pAccessible = pAccessible;
        }
    
        return (*this_).__base1.Initialize();
    }
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        if !(*this_).m_pAccessible.is_null() {
            (*(*this_).m_pAccessible).SetWindow(hWnd);
        }
    }
    pub unsafe fn OnDestroy__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        if !(*this_).m_pAccessible.is_null() {
            (*(*this_).m_pAccessible).NotifyWinEvent(EVENT_OBJECT_DESTROY, (*this_).GetAccItem());
            (*(*this_).m_pAccessible).ClearAccItems();
            (*(*this_).m_pAccessible).Release();
            (*this_).m_pAccessible = core::ptr::null_mut();
        }
        (*this_).m_coInit.CoUninit();
    }
    pub unsafe fn OnGetObject__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        if lParam != -4 {
            return S_OK;
        }
    
        if (*this_).m_pAccessible.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if (*(*this_).m_pAccessible).m_bInitialized != 0 {
            return ((*(*this_).m_pAccessible).CreateRefToAccObj(wParam)) as HRESULT;
        }
    
        if (*this_).m_coInit.EnsureCoInit() >= 0 {
            let mut hr: HRESULT = (*(*this_).m_pAccessible).Initialize();
            if hr < 0 {
                (*(*this_).m_pAccessible).Release();
                (*this_).m_pAccessible = core::ptr::null_mut();
                return hr;
            }
    
            (*(*this_).m_pAccessible).NotifyWinEvent(EVENT_OBJECT_CREATE, (*this_).GetAccItem());
            return ((*(*this_).m_pAccessible).CreateRefToAccObj(wParam)) as HRESULT;
        }
    
        return S_OK;
    }
    pub unsafe fn OnShowWindow__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pAccessible.is_null() {
            if wParam != 0 {
                (*(*this_).m_pAccessible).NotifyWinEvent(EVENT_OBJECT_SHOW, (*this_).GetAccItem());
                (*(*this_).m_pAccessible).NotifyWinEvent(EVENT_OBJECT_FOCUS, (*this_).GetAccItem());
            } else {
                (*(*this_).m_pAccessible).NotifyWinEvent(EVENT_OBJECT_HIDE, (*this_).GetAccItem());
            }
        }
    
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn OnTimer__body(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;
        if wParam == TIMER_ID_DOACCDEFAULTACTION as WPARAM {
            KillTimer((*this_).__base1.m_hWnd, TIMER_ID_DOACCDEFAULTACTION as UINT_PTR);
            if !(*this_).m_pAccessible.is_null() && (*this_).m_nMenuWndID != 0 {
                (*(*this_).m_pAccessible).DoDefaultActionReal(((*this_).m_nMenuWndID) as INT);
                (*this_).m_nMenuWndID = 0;
            }
        }
    }
}

impl CUTBMenuItem {
    pub unsafe fn __ctor_0(&mut self, pMenuUI: *mut CUTBMenuWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_pMenuUI = pMenuUI;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).__base1.m_hbmColor.is_null() {
            DeleteObject(((*this_).__base1.m_hbmColor).cast());
            (*this_).__base1.m_hbmColor = core::ptr::null_mut();
        }
        if !(*this_).__base1.m_hbmMask.is_null() {
            DeleteObject(((*this_).__base1.m_hbmMask).cast());
            (*this_).__base1.m_hbmMask = core::ptr::null_mut();
        }
    }
    pub unsafe fn GetMenuItem(&mut self) -> *mut CUIFMenuItem {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base1) as *mut CUIFMenuItem;
    }
    pub unsafe fn DoAccDefaultAction__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pMenuUI.is_null() {
            return (FALSE) as BOOL;
        }
    
        (*(*this_).m_pMenuUI).StartDoAccDefaultActionTimer(this_);
        return (TRUE) as BOOL;
    }
    pub unsafe fn DoAccDefaultActionReal__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).__base1.m_pSubMenu.is_null() {
            (*this_).__base1.OnLButtonUp(0, 0);
        } else {
            (*this_).__base1.ShowSubPopup();
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn GetAccDefaultAction__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        let mut szText: [WCHAR; 64] = [0; 64];
        LoadStringW(g_hInst, IDS_LEFTCLICK, szText.as_mut_ptr(), _countof!(szText) as i32);
        return SysAllocString(szText.as_ptr());
    }
    pub unsafe fn GetAccLocation__body(&mut self, lprc: LPRECT) {
        let this_: *mut Self = self;
        (*this_).__base1.GetRect(lprc);
        ClientToScreen((*(*this_).m_pMenuUI).__base1.__base0.m_hWnd, lprc as LPPOINT);
        ClientToScreen((*(*this_).m_pMenuUI).__base1.__base0.m_hWnd, core::ptr::addr_of_mut!((*lprc).right) as LPPOINT);
    }
    pub unsafe fn GetAccName__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        return SysAllocString((*this_).__base1.m_pszMenuItemLeft);
    }
    pub unsafe fn GetAccRole__body(&mut self) -> INT {
        let this_: *mut Self = self;
        if 0 != 0 { //FIXME
            return 21;
        }
        return 12;
    }
}

impl CModalMenu {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        { }
    }
    pub unsafe fn InsertItem(&mut self, pMenuUI: *mut CUTBMenuWnd, nCommandId: INT, nStringID: INT) -> *mut CUTBMenuItem {
        let this_: *mut Self = self;
        let mut pMenuItem: *mut CUTBMenuItem = CUTBMenuItem__new_in_0(cpp_alloc_CicNoThrow::<CUTBMenuItem>(), pMenuUI);
        if pMenuItem.is_null() {
            return core::ptr::null_mut();
        }
    
        let mut szText: [u16; 256] = [0u16; 256];
        LoadStringW(g_hInst, nStringID as UINT, szText.as_mut_ptr(), _countof!(szText) as INT);
    
        if (*pMenuItem).__base1.Initialize() != 0
            && (*pMenuItem).__base1.Init(nCommandId as UINT, szText.as_mut_ptr()) != 0
            && (*pMenuUI).__base1.InsertItem(core::ptr::addr_of_mut!((*pMenuItem).__base1)) != 0
        {
            return pMenuItem;
        }
    
        CUTBMenuItem__delete(pMenuItem);
        core::ptr::null_mut()
    }
    pub unsafe fn PostKey(&mut self, bUp: BOOL, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        (*(*this_).m_pMenuUI).__base1.PostKey(bUp, wParam, lParam);
    }
    pub unsafe fn CancelMenu(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pMenuUI.is_null() {
            (*(*this_).m_pMenuUI).__base1.CancelMenu();
        }
    }
}

impl CUTBContextMenu {
    pub unsafe fn __ctor_0(&mut self, pTipbarWnd: *mut CTipbarWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_pTipbarWnd = pTipbarWnd;
        }
    }
    pub unsafe fn Init(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_pTipbarThread = (*(*this_).m_pTipbarWnd).m_pFocusThread;
        return (!((*this_).m_pTipbarThread).is_null()) as BOOL;
    }
    pub unsafe fn CreateMenuUI(&mut self, bFlag: BOOL) -> *mut CUTBMenuWnd {
        let this_: *mut Self = self;
        let mut dwStatus: DWORD = 0;
    
        if ((*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).GetShowFloatingStatus(core::ptr::addr_of_mut!(dwStatus))) < 0 {
            return core::ptr::null_mut();
        }
    
        let pMenuUI: *mut CUTBMenuWnd = CUTBMenuWnd__new_in_0(cpp_alloc_CicNoThrow::<CUTBMenuWnd>(), g_hInst, g_dwMenuStyle, 0);
        if pMenuUI.is_null() {
            return core::ptr::null_mut();
        }
    
        (*pMenuUI).Initialize();
    
        if (dwStatus & (TF_SFT_DESKBAND | TF_SFT_MINIMIZED)) != 0 {
            let pRestoreLangBar: *mut CUTBMenuItem = (*this_).__base0.InsertItem(pMenuUI, (ID_RESTORELANGBAR) as INT, (IDS_RESTORELANGBAR2) as INT);
            if !pRestoreLangBar.is_null() && (*(*this_).m_pTipbarWnd).m_dwUnknown20 == 0 {
                (*pRestoreLangBar).__base1.Gray((TRUE) as i32);
            }
        } else {
            (*this_).__base0.InsertItem(pMenuUI, (ID_DESKBAND) as INT, (IDS_MINIMIZE) as INT);
    
            if bFlag != 0 {
                if IsTransparecyAvailable() != 0 {
                    if (dwStatus & TF_LBI_BALLOON) != 0 {
                        (*this_).__base0.InsertItem(pMenuUI, (ID_TRANS) as INT, (IDS_TRANSPARENCY) as INT);
                    } else {
                        let pTransparency: *mut CUTBMenuItem = (*this_).__base0.InsertItem(pMenuUI, (ID_NOTRANS) as INT, (IDS_TRANSPARENCY) as INT);
                        if !pTransparency.is_null() {
                            (*pTransparency).__base1.Check((TRUE) as i32);
                        }
                    }
                }
    
                if (dwStatus & TF_SFT_LABELS) == 0 {
                    (*this_).__base0.InsertItem(pMenuUI, (ID_LABELS) as INT, (IDS_TEXTLABELS) as INT);
                } else {
                    let pTextLabels: *mut CUTBMenuItem = (*this_).__base0.InsertItem(pMenuUI, (ID_NOLABELS) as INT, (IDS_TEXTLABELS) as INT);
                    if !pTextLabels.is_null() {
                        (*pTextLabels).__base1.Check((TRUE) as i32);
                    }
                }
    
                let pVertical: *mut CUTBMenuItem = (*this_).__base0.InsertItem(pMenuUI, (ID_VERTICAL) as INT, (IDS_VERTICAL) as INT);
                if !pVertical.is_null() {
                    (*pVertical).__base1.Check((((*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as BOOL);
                }
            }
        }
    
        if bFlag != 0 {
            let mut pExtraIcons: *mut CUTBMenuItem = core::ptr::null_mut();
    
            if (dwStatus & TF_SFT_EXTRAICONSONMINIMIZED) != 0 {
                pExtraIcons = (*this_).__base0.InsertItem(pMenuUI, (ID_NOEXTRAICONS) as INT, (IDS_EXTRAICONS) as INT);
                if !pExtraIcons.is_null() {
                    (*pExtraIcons).__base1.Check((TRUE) as i32);
                }
            } else {
                pExtraIcons = (*this_).__base0.InsertItem(pMenuUI, (ID_EXTRAICONS) as INT, (IDS_EXTRAICONS) as INT);
            }
    
            if !pExtraIcons.is_null() {
                if GetKeyboardLayoutList(0, core::ptr::null_mut()) == 1 {
                    (*pExtraIcons).__base1.Check((TRUE) as i32);
                    (*pExtraIcons).__base1.Gray((TRUE) as i32);
                } else {
                    (*pExtraIcons).__base1.Gray((FALSE) as i32);
                }
            }
    
            if (dwStatus & TF_SFT_DESKBAND) != 0 {
                (*this_).__base0.InsertItem(pMenuUI, (ID_ADJUSTDESKBAND) as INT, (IDS_ADJUSTLANGBAND) as INT);
            }
    
            (*this_).__base0.InsertItem(pMenuUI, (ID_SETTINGS) as INT, (IDS_SETTINGS) as INT);
    
            if CheckCloseMenuAvailable() != 0 {
                (*this_).__base0.InsertItem(pMenuUI, (ID_CLOSELANGBAR) as INT, (IDS_CLOSELANGBAR) as INT);
            }
        }
    
        pMenuUI
    }
    pub unsafe fn ShowPopup(&mut self, pWindow: *mut CUIFWindow, pt: POINT, prc: LPCRECT, bFlag: BOOL) -> UINT {
        let this_: *mut Self = self;
        if g_bWinLogon != 0 {
            return 0;
        }
    
        if !(*this_).__base0.m_pMenuUI.is_null() {
            return -(1 as i32) as UINT;
        }
    
        (*this_).__base0.m_pMenuUI = (*this_).CreateMenuUI(bFlag);
        if (*this_).__base0.m_pMenuUI.is_null() {
            return 0;
        }
    
        let nCommandId: UINT = (*(*this_).__base0.m_pMenuUI).__base1.ShowModalPopup(pWindow, prc, (TRUE) as i32);
    
        if !(*this_).__base0.m_pMenuUI.is_null() {
            CUTBMenuWnd__delete((*this_).__base0.m_pMenuUI);
            (*this_).__base0.m_pMenuUI = core::ptr::null_mut();
        }
    
        return nCommandId;
    }
    pub unsafe fn SelectMenuItem(&mut self, nCommandId: UINT) -> BOOL {
        let this_: *mut Self = self;
        'sw: {
            match nCommandId {
                ID_TRANS => {
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_LOWTRANSPARENCY);
                    break 'sw;
                }
    
                ID_NOTRANS => {
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_NOTRANSPARENCY);
                    break 'sw;
                }
    
                ID_LABELS => {
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_LABELS);
                    break 'sw;
                }
    
                ID_NOLABELS => {
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_NOLABELS);
                    break 'sw;
                }
    
                ID_DESKBAND => {
                    if g_bEnableDeskBand == 0 || (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) == 0 {
                        (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_MINIMIZED);
                    } else {
                        let mut dwStatus: DWORD = 0;
                        (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr)
                            .GetShowFloatingStatus(core::ptr::addr_of_mut!(dwStatus));
    
                        if (dwStatus & TF_SFT_DESKBAND) != 0 {
                            break 'sw;
                        }
    
                        (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_DESKBAND);
                    }
    
                    let pDialog: *mut CUTBMinimizeLangBarDlg =
                        CUTBMinimizeLangBarDlg__new_in_0(cpp_alloc_CicNoThrow::<CUTBMinimizeLangBarDlg>());
                    if !pDialog.is_null() {
                        (*pDialog).DoModal((*(*(*this_).m_pTipbarWnd).GetWindow()).m_hWnd);
                        (*pDialog).__base0._Release();
                    }
                    break 'sw;
                }
    
                ID_CLOSELANGBAR => {
                    let pDialog: *mut CUTBCloseLangBarDlg =
                        CUTBCloseLangBarDlg__new_in_0(cpp_alloc_CicNoThrow::<CUTBCloseLangBarDlg>());
                    if !pDialog.is_null() {
                        let bOK: BOOL = (*pDialog).DoModal((*(*(*this_).m_pTipbarWnd).GetWindow()).m_hWnd);
                        (*pDialog).__base0._Release();
                        if bOK == 0 {
                            DoCloseLangbar();
                        }
                    }
                    break 'sw;
                }
    
                ID_EXTRAICONS => {
                    (*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags &= !TIPBAR_NODESKBAND;
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_EXTRAICONSONMINIMIZED);
                    break 'sw;
                }
    
                ID_NOEXTRAICONS => {
                    (*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags &= !TIPBAR_NODESKBAND;
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_NOEXTRAICONSONMINIMIZED);
                    break 'sw;
                }
    
                ID_RESTORELANGBAR => {
                    (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_LBI_ICON);
                    break 'sw;
                }
    
                ID_VERTICAL => {
                    (*(*this_).m_pTipbarWnd).SetVertical(
                        ((((*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as BOOL),
                    );
                    break 'sw;
                }
    
                ID_ADJUSTDESKBAND => {
                    (*(*this_).m_pTipbarWnd).AdjustDeskBandSize((TRUE) as BOOL);
                    break 'sw;
                }
    
                ID_SETTINGS => {
                    TF_RunInputCPL();
                    break 'sw;
                }
    
                _ => {
                    break 'sw;
                }
            }
        }
    
        return (TRUE) as BOOL;
    }
}

impl CUTBLBarMenu {
    pub unsafe fn __ctor_0(&mut self, hInst: HINSTANCE) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_hInst = hInst;
        }
    }
    pub unsafe fn CreateMenuUI(&mut self) -> *mut CUTBMenuWnd {
        let this_: *mut Self = self;
        let mut pMenuUI: *mut CUTBMenuWnd = CUTBMenuWnd__new_in_0(cpp_alloc_CicNoThrow::<CUTBMenuWnd>(), (*this_).m_hInst, g_dwMenuStyle, 0);
        let _ = 0;
        if pMenuUI.is_null() {
            return core::ptr::null_mut();
        }
    
        (*pMenuUI).Initialize();
        let mut iItem: usize = 0 as usize;
        while iItem < (*this_).__base0.m_MenuItems.size() {
            let pItem: *mut CUTBLBarMenuItem = (*this_).__base0.m_MenuItems[iItem] as *mut CUTBLBarMenuItem;
            (*pItem).InsertToUI(pMenuUI);
            iItem += 1;
        }
    
        return pMenuUI;
    }
    pub unsafe fn ShowPopup(&mut self, pWindow: *mut CUIFWindow, pt: POINT, prcExclude: LPCRECT) -> INT {
        let this_: *mut Self = self;
        if !(*this_).m_pMenuUI.is_null() {
            return 0;
        }
    
        (*this_).m_pMenuUI = (*this_).CreateMenuUI();
        if (*this_).m_pMenuUI.is_null() {
            return -1;
        }
    
        let mut nCommandId: INT = (*(*this_).m_pMenuUI).__base1.ShowModalPopup(pWindow, prcExclude, TRUE as BOOL) as INT;
    
        if !(*this_).m_pMenuUI.is_null() {
            CUTBMenuWnd__delete((*this_).m_pMenuUI);
            (*this_).m_pMenuUI = core::ptr::null_mut();
        }
    
        return nCommandId;
    }
    pub unsafe fn CreateMenuItem__body(&mut self) -> *mut CCicLibMenuItem {
        let this_: *mut Self = self;
        let pItem: *mut CUTBLBarMenuItem = CUTBLBarMenuItem__new_in_0(cpp_alloc_CicNoThrow::<CUTBLBarMenuItem>());
        if pItem.is_null() {
            return core::ptr::null_mut();
        }
        (*pItem).m_pLBarMenu = this_;
        return core::ptr::addr_of_mut!((*pItem).__base0);
    }
    pub unsafe fn CreateSubMenu__body(&mut self) -> *mut CCicLibMenu {
        let this_: *mut Self = self;
        let p = CUTBLBarMenu__new_in_0(cpp_alloc_CicNoThrow::<CUTBLBarMenu>(), (*this_).m_hInst);
        return core::ptr::addr_of_mut!((*p).__base0);
    }
}

impl CUTBLBarMenuItem {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_pLBarMenu = core::ptr::null_mut();
        }
    }
    pub unsafe fn InsertToUI(&mut self, pMenuUI: *mut CUTBMenuWnd) -> BOOL {
        let this_: *mut Self = self;
        if ((*this_).__base0.m_dwFlags & 4) != 0 {
            (*pMenuUI).__base1.InsertSeparator();
            return (TRUE) as BOOL;
        }
        if ((*this_).__base0.m_dwFlags & 2) != 0 {
            //FIXME
        } else {
            //FIXME
        }
        return (FALSE) as BOOL;
    }
}

impl CTipbarGripper {
    pub unsafe fn __ctor_0(&mut self, pTipbarWnd: *mut CTipbarWnd, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_bInDebugMenu = (FALSE) as i32;
            (*this_).m_pTipbarWnd = pTipbarWnd;
        }
    }
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*(*this_).m_pTipbarWnd).RestoreFromStub();
    
        if g_bEnableDeskBand != 0 && (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) != 0
        {
            let mut AppBar: APPBARDATA = core::mem::zeroed();
            AppBar.cbSize = core::mem::size_of::<APPBARDATA>() as DWORD;
            AppBar.hWnd = FindWindowW(CPP_COM_WSTR_BB39C012AB62.as_ptr(), core::ptr::null());
            if SHAppBarMessage(ABM_GETTASKBARPOS, core::ptr::addr_of_mut!(AppBar)) != 0
            {
                let rc: RECT = AppBar.rc;
                let mut pt: POINT = core::mem::zeroed();
                GetCursorPos(core::ptr::addr_of_mut!(pt));
                if !g_pTipbarWnd.is_null() && PtInRect(core::ptr::addr_of!(rc), pt) != 0
                {
                    (*(*g_pTipbarWnd).m_pLangBarMgr).ShowFloating(
                        TF_SFT_DESKBAND | TF_SFT_EXTRAICONSONMINIMIZED);
                }
            }
        }
    
        (*this_).__base0.OnLButtonUp(x, y);
        (*(*this_).m_pTipbarWnd).UpdatePosFlags();
    }
    pub unsafe fn OnRButtonUp__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let _ = this_;
        let _ = x;
        let _ = y;
        if g_bShowDebugMenu != 0 {
            // FIXME: Debugging feature
        }
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT, x: LONG, y: LONG) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_bInDebugMenu != 0 {
            return (FALSE) as BOOL;
        }
    
        return (*this_).__base0.OnSetCursor(uMsg, x, y);
    }
}

impl CLangBarItemList {
    pub unsafe fn IsStartedIntentionally(&mut self, rclsid: &CLSID) -> BOOL {
        let this_: *mut Self = self;
        let pItem = (*this_).FindItem(rclsid);
        if pItem.is_null() {
            return (FALSE) as BOOL;
        }
        return (*pItem).m_bStartedIntentionally;
    }
    pub unsafe fn AddItem(&mut self, rclsid: &CLSID) -> *mut LANGBARITEMSTATE {
        let this_: *mut Self = self;
        let mut pItem: *mut LANGBARITEMSTATE = (*this_).FindItem(rclsid);
        if !pItem.is_null() {
            return pItem;
        }
    
        pItem = (*this_).__base0.Append(1);
        if pItem.is_null() {
            return core::ptr::null_mut();
        }
    
        ZeroMemory!(pItem as LPVOID, core::mem::size_of::<LANGBARITEMSTATE>());
        (*pItem).m_clsid = *rclsid;
        (*pItem).m_dwDemoteLevel = 0;
        return pItem;
    }
    pub unsafe fn Clear(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.clear();
    
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        let mut error: LSTATUS;
        error = regKey.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_E8561835E59F.as_ptr(), KEY_ALL_ACCESS);
        if (error) as u32 == ERROR_SUCCESS {
            regKey.RecurseDeleteKey(CPP_COM_WSTR_280DCAD69C43.as_ptr());
        }
    }
    pub unsafe fn SetDemoteLevel(&mut self, rclsid: &CLSID, dwDemoteLevel: DWORD) -> BOOL {
        let this_: *mut Self = self;
        let pItem: *mut LANGBARITEMSTATE = (*this_).AddItem(rclsid);
        if pItem.is_null() {
            return (TRUE) as BOOL;
        }
    
        (*pItem).m_dwDemoteLevel = dwDemoteLevel;
        if (((*pItem).IsShown()) == 0) {
            if (*pItem).m_nTimerID != 0 {
                if !g_pTipbarWnd.is_null() {
                    (*g_pTipbarWnd).KillTimer((*pItem).m_nTimerID);
                }
                (*pItem).m_nTimerID = 0;
                (*pItem).m_uTimeOut = 0;
            }
            (*pItem).m_bDisableDemoting = (FALSE) as i32;
        }
    
        (*this_).SaveItem(core::ptr::null_mut(), pItem);
        return (TRUE) as BOOL;
    }
    pub unsafe fn FindItem(&mut self, rclsid: &CLSID) -> *mut LANGBARITEMSTATE {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).__base0.size() {
            let item: *mut LANGBARITEMSTATE = core::ptr::addr_of_mut!((*this_).__base0[iItem]);
            if IsEqualCLSID!(&(*item).m_clsid, rclsid) {
                return item;
            }
            iItem += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn GetItemStateFromTimerId(&mut self, nTimerID: UINT_PTR) -> *mut LANGBARITEMSTATE {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).__base0.size() {
            let item = &mut (*this_).__base0[iItem];
            if item.m_nTimerID == nTimerID {
                return item as *mut LANGBARITEMSTATE;
            }
            iItem += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn Load(&mut self) {
        let this_: *mut Self = self;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        let mut error: LSTATUS;
        error = regKey.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_A5A4F2227C74.as_ptr(), KEY_READ);
        if (error) as u32 != ERROR_SUCCESS {
            return;
        }
    
        let mut szKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut dwIndex: DWORD = 0;
        'outer: loop {
            'body: {
                error = RegEnumKeyW(regKey.m_hKey, dwIndex, szKeyName.as_mut_ptr(), _countof!(szKeyName) as DWORD);
                if (error) as u32 != ERROR_SUCCESS {
                    break 'outer; // the C `break` leaves the for loop
                }
    
                let mut clsid: CLSID = core::mem::zeroed();
                if CLSIDFromString(szKeyName.as_ptr(), core::ptr::addr_of_mut!(clsid)) != S_OK {
                    break 'body;
                }
    
                let mut regKey2: CicRegKey = CicRegKey__zeroed();
                error = regKey2.Open(regKey.m_hKey, szKeyName.as_ptr(), KEY_READ);
                if (error) as u32 != ERROR_SUCCESS {
                    break 'body;
                }
    
                let pItem: *mut LANGBARITEMSTATE = (*this_).AddItem(&clsid);
                if pItem.is_null() {
                    break 'body;
                }
    
                let mut Data: DWORD = 0;
                regKey2.QueryDword(CPP_COM_WSTR_C8D26F04DF9D.as_ptr(), core::ptr::addr_of_mut!(Data));
                (*pItem).m_dwDemoteLevel = Data;
                regKey2.QueryDword(CPP_COM_WSTR_DAD1FE337325.as_ptr(), core::ptr::addr_of_mut!(Data));
                (*pItem).m_bDisableDemoting = (Data != 0) as BOOL;
            }
            dwIndex += 1;
        }
    }
    pub unsafe fn SaveItem(&mut self, mut pRegKey: *mut CicRegKey, pState: *const LANGBARITEMSTATE) {
        let this_: *mut Self = self;
        let mut error: LSTATUS = 0;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
    
        if pRegKey.is_null() {
            error = regKey.Create(HKEY_CURRENT_USER, CPP_COM_WSTR_A5A4F2227C74.as_ptr());
            if (error) as u32 != ERROR_SUCCESS {
                return;
            }
    
            pRegKey = core::ptr::addr_of_mut!(regKey);
        }
    
        let mut szSubKey: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
        StringFromGUID2(&(*pState).m_clsid, szSubKey.as_mut_ptr(), (_countof!(szSubKey)) as INT);
    
        if (*pState).m_dwDemoteLevel != 0 || (*pState).m_bDisableDemoting != 0 {
            let mut regKey2: CicRegKey = CicRegKey__zeroed();
            error = regKey2.Create((&raw mut *pRegKey).cast(), szSubKey.as_ptr());
            if (error) as u32 == ERROR_SUCCESS {
                let mut dwDemoteLevel: DWORD = (*pState).m_dwDemoteLevel;
                if dwDemoteLevel != 0 {
                    regKey2.SetDword(CPP_COM_WSTR_C8D26F04DF9D.as_ptr(), dwDemoteLevel);
                } else {
                    regKey2.DeleteValue(CPP_COM_WSTR_C8D26F04DF9D.as_ptr());
                }
    
                regKey2.SetDword(CPP_COM_WSTR_DAD1FE337325.as_ptr(), ((*pState).m_bDisableDemoting) as u32);
            }
        } else {
            (*pRegKey).RecurseDeleteKey(szSubKey.as_ptr());
        }
    }
    pub unsafe fn StartDemotingTimer(&mut self, rclsid: &CLSID, bIntentional: BOOL) {
        let this_: *mut Self = self;
        if g_bIntelliSense == 0 {
            return;
        }
    
        let pItem: *mut LANGBARITEMSTATE = (*this_).AddItem(rclsid);
        if pItem.is_null() || (*pItem).m_bDisableDemoting != 0 {
            return;
        }
    
        if (*pItem).m_nTimerID != 0 {
            if bIntentional == 0 {
                return;
            }
    
            if !g_pTipbarWnd.is_null() {
                (*g_pTipbarWnd).KillTimer((*pItem).m_nTimerID);
            }
    
            (*pItem).m_nTimerID = 0;
        }
    
        (*pItem).m_bStartedIntentionally |= bIntentional;
    
        let uTimeOut: UINT = if bIntentional != 0 {
            g_uTimeOutIntentional
        } else {
            g_uTimeOutNonIntentional
        };
        (*pItem).m_uTimeOut += uTimeOut;
    
        if (*pItem).m_uTimeOut < g_uTimeOutMax {
            let uDemotingTimerId: UINT_PTR = (*this_).FindDemotingTimerId();
            (*pItem).m_nTimerID = uDemotingTimerId;
            if uDemotingTimerId != 0 {
                if !g_pTipbarWnd.is_null() {
                    (*g_pTipbarWnd).SetTimer(uDemotingTimerId, uTimeOut);
                }
            }
        } else {
            (*pItem).m_bDisableDemoting = (TRUE) as i32;
        }
    }
    pub unsafe fn FindDemotingTimerId(&mut self) -> UINT_PTR {
        let this_: *mut Self = self;
        let mut nTimerID: UINT_PTR = 10000;
    
        if (*this_).__base0.empty() {
            return nTimerID;
        }
    
        loop {
            let mut iItem: usize = 0;
    
            while (*this_).__base0[iItem].m_nTimerID != nTimerID {
                iItem += 1;
                if iItem >= (*this_).__base0.size() {
                    return nTimerID;
                }
            }
    
            nTimerID += 1;
            if nTimerID >= 10050 {
                return 0;
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}

impl CTrayIconWnd {
    pub unsafe extern "system" fn EnumChildWndProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
        let pWnd: *mut CTrayIconWnd = lParam as *mut CTrayIconWnd;
    
        let mut ClassName: [u16; 60] = [0u16; 60];
        GetClassName(hWnd, ClassName.as_mut_ptr(), _countof!(ClassName) as i32);
        let TrayNotifyWnd: [u16; 14] = [
            'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'o' as u16, 't' as u16,
            'i' as u16, 'f' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0,
        ];
        if lstrcmp(ClassName.as_ptr(), TrayNotifyWnd.as_ptr()) != 0 {
            return (TRUE) as BOOL;
        }
    
        (*pWnd).m_hNotifyWnd = hWnd;
        return (FALSE) as BOOL;
    }
    pub unsafe extern "system" fn _WndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let mut pThis: *mut CTrayIconWnd;
        match uMsg {
            WM_CREATE => {
                CTrayIconWnd::SetThis(hWnd, lParam as LPCREATESTRUCT);
            }
            WM_DESTROY => {
                SetWindowLongPtr(hWnd, GWL_USERDATA, 0);
            }
            WM_TIMER => {
                if wParam == 100 {
                    KillTimer(hWnd, 100);
                    pThis = CTrayIconWnd::GetThis(hWnd);
                    if !pThis.is_null() {
                        (*pThis).CallOnDelayMsg();
                    }
                }
            }
            _ => {
                if uMsg < WM_USER {
                    return DefWindowProc(hWnd, uMsg, wParam, lParam);
                }
                pThis = CTrayIconWnd::GetThis(hWnd);
                if !pThis.is_null() && (*pThis).OnIconMessage(uMsg, wParam, lParam) != 0 {
                    // break out of the switch
                } else {
                    return DefWindowProc(hWnd, uMsg, wParam, lParam);
                }
            }
        }
        return 0;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).m_uCallbackMsg = WM_USER + 0x1000;
            (*this_).m_uNotifyIconID = 0x1000;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Items.size() {
            let pItem: *mut *mut CButtonIconItem = core::ptr::addr_of_mut!((*this_).m_Items[iItem]);
            if !(*pItem).is_null() {
                CButtonIconItem__delete(*pItem);
                *pItem = core::ptr::null_mut();
            }
            iItem += 1;
        }
    }
    pub unsafe fn RegisterClassW() -> BOOL {
        let mut wc: WNDCLASSEX = core::mem::zeroed();
        wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.hInstance = g_hInst;
        wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW as usize as LPCTSTR);
        wc.lpfnWndProc = Some(CTrayIconWnd::_WndProc);
        let class_name: [u16; 18] = [
            'C' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16,
            'I' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'W' as u16,
            'n' as u16, 'd' as u16, 'C' as u16, 'l' as u16, 'a' as u16,
            's' as u16, 's' as u16, 0,
        ];
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassEx(core::ptr::addr_of_mut!(wc));
        return (TRUE) as BOOL;
    }
    pub unsafe fn GetThis(hWnd: HWND) -> *mut CTrayIconWnd {
        return GetWindowLongPtr(hWnd, GWL_USERDATA) as *mut CTrayIconWnd;
    }
    pub unsafe fn SetThis(hWnd: HWND, pCS: LPCREATESTRUCT) {
        if !pCS.is_null() {
            SetWindowLongPtr(hWnd, GWL_USERDATA, (*pCS).lpCreateParams as LONG_PTR);
        } else {
            SetWindowLongPtr(hWnd, GWL_USERDATA, 0);
        }
    }
    pub unsafe fn CreateWnd(&mut self) -> HWND {
        let this_: *mut Self = self;
        let class_name: [u16; 18] = ['C' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'I' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 'C' as u16, 'l' as u16, 'a' as u16, 's' as u16, 's' as u16, 0];
        (*this_).m_hWnd = CreateWindowEx(0, (class_name.as_ptr()), core::ptr::null_mut(), WS_DISABLED,
                                  0, 0, 0, 0, core::ptr::null_mut(), core::ptr::null_mut(), g_hInst, this_ as LPVOID);
        (*this_).FindTrayEtc();
    
        (*this_).m_pMainIconItem = CMainIconItem__new_in_0(cpp_alloc_CicNoThrow::<CMainIconItem>(), this_);
        if !(*this_).m_pMainIconItem.is_null()
        {
            (*(*this_).m_pMainIconItem).Init((*this_).m_hWnd);
            (*this_).m_Items.Add(core::ptr::addr_of_mut!((*(*this_).m_pMainIconItem).__base0));
        }
    
        return (*this_).m_hWnd;
    }
    pub unsafe fn DestroyWnd(&mut self) {
        let this_: *mut Self = self;
        DestroyWindow((*this_).m_hWnd);
        (*this_).m_hWnd = core::ptr::null_mut();
    }
    pub unsafe fn SetMainIcon(&mut self, hKL: HKL) -> BOOL {
        let this_: *mut Self = self;
        if hKL.is_null() {
            (*(*this_).m_pMainIconItem).__base0.__base0.RemoveIcon();
            (*(*this_).m_pMainIconItem).__base0.m_hKL = core::ptr::null_mut();
            return (TRUE) as BOOL;
        }
    
        if hKL != (*(*this_).m_pMainIconItem).__base0.m_hKL {
            let mut szText: [u16; 64 as usize] = [0u16; 64 as usize];
            let mut hIcon: HICON = TF_GetLangIcon(LOWORD!(hKL), szText.as_mut_ptr(), (_countof!(szText)) as INT);
            if !hIcon.is_null() {
                (*(*this_).m_pMainIconItem).__base0.__base0.SetIcon(hIcon, szText.as_ptr());
                DestroyIcon(hIcon);
            } else {
                LoadStringW(g_hInst, IDS_RESTORELANGBAR, szText.as_mut_ptr(), (_countof!(szText)) as i32);
                hIcon = LoadIconW(g_hInst, MAKEINTRESOURCEW!(IDI_MAINICON));
                (*(*this_).m_pMainIconItem).__base0.__base0.SetIcon(hIcon, szText.as_ptr());
            }
    
            (*(*this_).m_pMainIconItem).__base0.m_hKL = hKL;
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetIcon(&mut self, rguid: &GUID, dwUnknown24: DWORD, hIcon: HICON, psz: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        let mut pItem: *mut CButtonIconItem = (*this_).FindIconItem(rguid);
        if pItem.is_null() {
            if hIcon.is_null() {
                return (FALSE) as BOOL;
            }
            pItem = CButtonIconItem__new_in_0(cpp_alloc_CicNoThrow::<CButtonIconItem>(), this_, dwUnknown24);
            if pItem.is_null() {
                return (FALSE) as BOOL;
            }
    
            (*pItem).__base0._Init((*this_).m_hWnd, (*this_).m_uCallbackMsg, (*this_).m_uNotifyIconID, rguid);
            (*this_).m_uCallbackMsg += 2;
            (*this_).m_uNotifyIconID += 1;
            (*this_).m_Items.Add(pItem);
        }
    
        if hIcon.is_null() {
            return (*pItem).__base0.RemoveIcon();
        }
    
        return (*pItem).__base0.SetIcon(hIcon, psz);
    }
    pub unsafe fn RemoveAllIcon(&mut self, dwFlags: DWORD) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0 as usize;
        while iItem < (*this_).m_Items.size() {
            'body: {
                let pItem: *mut CButtonIconItem = (*this_).m_Items[iItem];
                if (dwFlags & 0x1) != 0 {
                    if IsEqualGUID!(core::ptr::addr_of!((*pItem).__base0.m_guid), &GUID_LBI_INATITEM)
                        || IsEqualGUID!(core::ptr::addr_of!((*pItem).__base0.m_guid), &GUID_LBI_CTRL)
                    {
                        break 'body;
                    }
                }
    
                if (dwFlags & 0x2) != 0 {
                    if IsEqualGUID!(core::ptr::addr_of!((*pItem).__base0.m_guid), &GUID_TFCAT_TIP_KEYBOARD) {
                        break 'body;
                    }
                }
    
                if (*pItem).__base0.m_uNotifyIconID < 0x1000 {
                    break 'body;
                }
    
                (*pItem).__base0.RemoveIcon();
            }
            iItem += 1;
        }
    }
    pub unsafe fn RemoveUnusedIcons(&mut self, unknown: i32,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn FindIconItem(&mut self, rguid: &GUID) -> *mut CButtonIconItem {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Items.size() {
            let pItem: *mut CButtonIconItem = (*this_).m_Items[iItem];
            if IsEqualGUID!(rguid, &(*pItem).__base0.m_guid) != false {
                return pItem;
            }
            iItem += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn FindTrayEtc(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let ShellTrayWnd: [u16; 14] = ['S' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, '_' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 'W' as u16, 'n' as u16, 'd' as u16, 0];
        let Progman: [u16; 8] = ['P' as u16, 'r' as u16, 'o' as u16, 'g' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 0];
        (*this_).m_hTrayWnd = FindWindow(ShellTrayWnd.as_ptr(), core::ptr::null_mut());
        if (*this_).m_hTrayWnd.is_null() {
            return (FALSE) as BOOL;
        }
    
        EnumChildWindows((*this_).m_hTrayWnd, Some(CTrayIconWnd::EnumChildWndProc), this_ as LPARAM);
        if (*this_).m_hNotifyWnd.is_null() {
            return (FALSE) as BOOL;
        }
        (*this_).m_dwTrayWndThreadId = GetWindowThreadProcessId((*this_).m_hTrayWnd, core::ptr::null_mut());
        (*this_).m_hwndProgman = FindWindow(Progman.as_ptr(), core::ptr::null_mut());
        (*this_).m_dwProgmanThreadId = GetWindowThreadProcessId((*this_).m_hwndProgman, core::ptr::null_mut());
        return (TRUE) as BOOL;
    }
    pub unsafe fn GetNotifyWnd(&mut self) -> HWND {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hNotifyWnd) == 0 {
            (*this_).FindTrayEtc();
        }
        return (*this_).m_hNotifyWnd;
    }
    pub unsafe fn OnIconMessage(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        if !g_pTipbarWnd.is_null() {
            (*g_pTipbarWnd).AttachFocusThread();
        }
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Items.size() {
            let pItem: *mut CButtonIconItem = (*this_).m_Items[iItem];
            if !pItem.is_null() {
                if uMsg == (*pItem).__base0.m_uCallbackMessage {
                    (*pItem).OnMsg(wParam, lParam);
                    return (TRUE) as BOOL;
                }
            }
            iItem += 1;
        }
        return (FALSE) as BOOL;
    }
    pub unsafe fn CallOnDelayMsg(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Items.size() {
            let pItem: *mut CButtonIconItem = (*this_).m_Items[iItem];
            if !pItem.is_null() && (*this_).m_uCallbackMessage == (*pItem).__base0.m_uCallbackMessage {
                (*pItem).OnDelayMsg((*this_).m_uMsg);
                break;
            }
            iItem += 1;
        }
    }
}

impl CTrayIconItem {
    pub unsafe fn __ctor_0(&mut self, pTrayIconWnd: *mut CTrayIconWnd) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_dwIconAddOrModify = NIM_ADD;
            (*this_).m_pTrayIconWnd = pTrayIconWnd;
        }
    }
    pub unsafe fn _Init(&mut self, hWnd: HWND, uCallbackMessage: UINT, uNotifyIconID: UINT, rguid: &GUID) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWnd;
        (*this_).m_uCallbackMessage = uCallbackMessage;
        (*this_).m_uNotifyIconID = uNotifyIconID;
        (*this_).m_guid = *rguid;
        return (TRUE) as BOOL;
    }
    pub unsafe fn UpdateMenuRectPoint(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let hNotifyWnd: HWND = (*(*this_).m_pTrayIconWnd).GetNotifyWnd();
        GetClientRect(hNotifyWnd, core::ptr::addr_of_mut!((*this_).m_rcMenu));
        ClientToScreen(hNotifyWnd, core::ptr::addr_of_mut!((*this_).m_rcMenu) as LPPOINT);
        ClientToScreen(hNotifyWnd, core::ptr::addr_of_mut!((*this_).m_rcMenu.right) as LPPOINT);
        GetCursorPos(core::ptr::addr_of_mut!((*this_).m_ptCursor));
        return (TRUE) as BOOL;
    }
    pub unsafe fn RemoveIcon(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_dwIconAddOrModify == NIM_MODIFY {
            let mut NotifyIcon: NOTIFYICONDATAW = core::mem::zeroed();
            NotifyIcon.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
            NotifyIcon.hWnd = (*this_).m_hWnd;
            NotifyIcon.uID = (*this_).m_uNotifyIconID;
            NotifyIcon.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            NotifyIcon.uCallbackMessage = (*this_).m_uCallbackMessage;
            Shell_NotifyIconW(NIM_DELETE, core::ptr::addr_of_mut!(NotifyIcon));
        }
    
        (*this_).m_dwIconAddOrModify = NIM_ADD;
        (*this_).m_bIconAdded = (TRUE) as i32;
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetIcon__body(&mut self, hIcon: HICON, pszTip: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        if hIcon.is_null() {
            return (FALSE) as BOOL;
        }
    
        let mut NotifyIcon: NOTIFYICONDATAW = core::mem::zeroed();
        NotifyIcon.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
        NotifyIcon.hWnd = (*this_).m_hWnd;
        NotifyIcon.uID = (*this_).m_uNotifyIconID;
        NotifyIcon.uFlags = NIF_ICON | NIF_MESSAGE;
        NotifyIcon.uCallbackMessage = (*this_).m_uCallbackMessage;
        NotifyIcon.hIcon = hIcon;
        if !pszTip.is_null() {
            NotifyIcon.uFlags |= NIF_TIP;
            // szTip is a field of a packed struct: its length comes from the array TYPE, never a reference.
        const fn __len<T, const N: usize>(_: *const [T; N]) -> usize { N }
        StringCchCopyW(core::ptr::addr_of_mut!(NotifyIcon.szTip).cast(), __len(core::ptr::addr_of!(NotifyIcon.szTip)), pszTip);
        }
    
        Shell_NotifyIconW((*this_).m_dwIconAddOrModify, core::ptr::addr_of_mut!(NotifyIcon));
    
        (*this_).m_dwIconAddOrModify = NIM_MODIFY;
        (*this_).m_bIconAdded = (NIM_MODIFY as i32);
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnMsg__body(&mut self, wParam: WPARAM,
        lParam: LPARAM,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn OnDelayMsg__body(&mut self, uMsg: UINT,
    ) -> BOOL {
        let this_: *mut Self = self;
        0
    }
}

impl CButtonIconItem {
    pub unsafe fn __ctor_0(&mut self, pWnd: *mut CTrayIconWnd, dwUnknown24: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pWnd);
        (*this_).__init_vtables();
        {
            (*this_).m_dwUnknown24 = dwUnknown24;
        }
    }
    pub unsafe fn OnMsg__body(&mut self, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        let _ = (this_, wParam);
        match (lParam) as u32 {
            v if (v) as u32 == WM_LBUTTONDOWN
                || (v) as u32 == WM_RBUTTONDOWN
                || (v) as u32 == WM_LBUTTONDBLCLK
                || (v) as u32 == WM_RBUTTONDBLCLK => {}
            _ => {
                return (TRUE) as BOOL;
            }
        }
    
        //FIXME
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnDelayMsg__body(&mut self, uMsg: UINT,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
}

impl CMainIconItem {
    pub unsafe fn __ctor_0(&mut self, pWnd: *mut CTrayIconWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(pWnd, 1);
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn Init(&mut self, hWnd: HWND) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.__base0._Init(hWnd, WM_USER, 0, &GUID_LBI_TRAYMAIN);
    }
    pub unsafe fn OnDelayMsg__body(&mut self, uMsg: UINT) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).__base0.OnDelayMsg(uMsg) == 0 {
            return 0;
        }
    
        if uMsg == WM_LBUTTONDBLCLK {
            if (*g_pTipbarWnd).m_dwUnknown20 != 0 {
                (*(*g_pTipbarWnd).m_pLangBarMgr).ShowFloating(TF_SFT_SHOWNORMAL);
            }
        } else if uMsg == WM_LBUTTONDOWN || uMsg == WM_RBUTTONDOWN {
            (*g_pTipbarWnd).ShowContextMenu(
                (*this_).__base0.__base0.m_ptCursor,
                core::ptr::addr_of_mut!((*this_).__base0.__base0.m_rcMenu),
                (uMsg == WM_RBUTTONDOWN) as BOOL,
            );
        }
        return (TRUE) as BOOL;
    }
}

impl CLBarItemBase {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_dwItemStatus = 0;
            (*this_).m_szToolTipText[0 as usize] = 0;
            (*this_).m_cRefs = 1;
            (*this_).m_pLangBarItemSink = core::ptr::null_mut();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pLangBarItemSink.is_null() {
            (*(*this_).m_pLangBarItemSink).Release();
        }
    }
    pub unsafe fn ShowInternal(&mut self, bShow: BOOL, bUpdate: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let dwOldStatus: DWORD = (*this_).m_dwItemStatus;
    
        if bShow != 0 {
            (*this_).m_dwItemStatus &= !TF_LBI_STATUS_HIDDEN;
        } else {
            (*this_).m_dwItemStatus |= TF_LBI_STATUS_HIDDEN;
        }
    
        if bUpdate != 0 && (dwOldStatus != (*this_).m_dwItemStatus) {
            if !(*this_).m_pLangBarItemSink.is_null() {
                (*(*this_).m_pLangBarItemSink).OnUpdate(TF_LBI_STATUS);
            }
        }
    
        return S_OK;
    }
    pub unsafe fn InitNuiInfo(&mut self, clsidService: &IID, guidItem: &GUID, dwStyle: DWORD, ulSort: DWORD, Source: LPCWSTR) {
        let this_: *mut Self = self;
        (*this_).m_NewUIInfo.clsidService = *clsidService;
        (*this_).m_NewUIInfo.guidItem = *guidItem;
        (*this_).m_NewUIInfo.dwStyle = dwStyle;
        (*this_).m_NewUIInfo.ulSort = ulSort;
        StringCchCopyW((*this_).m_NewUIInfo.szDescription.as_mut_ptr(), (_countof!((*this_).m_NewUIInfo.szDescription)) as _, Source);
    }
    pub unsafe fn GetInfo(&mut self, pInfo: *mut TF_LANGBARITEMINFO) -> HRESULT {
        let this_: *mut Self = self;
        CopyMemory!(pInfo as *mut core::ffi::c_void, core::ptr::addr_of_mut!((*this_).m_NewUIInfo) as *const core::ffi::c_void, core::mem::size_of::<TF_LANGBARITEMINFO>());
        return S_OK;
    }
    pub unsafe fn GetStatus(&mut self, pdwStatus: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwStatus = (*this_).m_dwItemStatus;
        return S_OK;
    }
    pub unsafe fn Show(&mut self, fShow: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).ShowInternal(fShow, (TRUE) as BOOL);
    }
    pub unsafe fn GetTooltipString(&mut self, pbstrToolTip: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if pbstrToolTip.is_null() {
            return E_INVALIDARG;
        }
        let bstr: BSTR = SysAllocString((*this_).m_szToolTipText.as_ptr());
        *pbstrToolTip = bstr;
        return if !bstr.is_null() { S_OK } else { E_OUTOFMEMORY };
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualIID!(riid, &IID_ITfLangBarItemSink) != false || !(*this_).m_pLangBarItemSink.is_null() {
            return TF_E_NOOBJECT;
        }
    
        let mut hr: HRESULT = (*punk).QueryInterface(&IID_ITfLangBarItemSink, core::ptr::addr_of_mut!((*this_).m_pLangBarItemSink) as *mut *mut core::ffi::c_void);
        if ((hr) >= 0) {
            *pdwCookie = 0x80000001;
        }
        return hr;
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if dwCookie != 0x80000001 {
            return E_FAIL;
        }
    
        if (*this_).m_pLangBarItemSink.is_null() {
            return E_UNEXPECTED;
        }
    
        (*(*this_).m_pLangBarItemSink).Release();
        (*this_).m_pLangBarItemSink = core::ptr::null_mut();
        return S_OK;
    }
}

impl CLBarItemButtonBase {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_hIcon = core::ptr::null_mut();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hIcon.is_null() {
            DestroyIcon((*this_).m_hIcon);
            (*this_).m_hIcon = core::ptr::null_mut();
        }
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 4] = [
            QITAB {
                piid: core::ptr::addr_of!(IID_ITfLangBarItem),
                dwOffset: core::mem::offset_of!(CLBarItemButtonBase, __base1) as DWORD,
            },
            QITAB {
                piid: core::ptr::addr_of!(IID_ITfLangBarItemButton),
                dwOffset: core::mem::offset_of!(CLBarItemButtonBase, __base2) as DWORD,
            },
            QITAB {
                piid: core::ptr::addr_of!(IID_ITfSource),
                dwOffset: core::mem::offset_of!(CLBarItemButtonBase, __base3) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObject);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).__base0.m_cRefs += 1;
        return ((*this_).__base0.m_cRefs) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).__base0.m_cRefs -= 1;
        if (*this_).__base0.m_cRefs == 0 {
            CLBarItemButtonBase__delete(this_);
            return 0;
        }
        return ((*this_).__base0.m_cRefs) as ULONG;
    }
    pub unsafe fn GetInfo(&mut self, pInfo: *mut TF_LANGBARITEMINFO) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.GetInfo(pInfo)
    }
    pub unsafe fn GetStatus(&mut self, pdwStatus: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.GetStatus(pdwStatus)
    }
    pub unsafe fn Show(&mut self, fShow: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.Show(fShow)
    }
    pub unsafe fn GetTooltipString(&mut self, pbstrToolTip: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).__base0.GetTooltipString(pbstrToolTip);
    }
    pub unsafe fn OnClick(&mut self, click: TfLBIClick, pt: POINT, prc: LPCRECT) -> HRESULT {
        let this_: *mut Self = self;
        if click == TF_LBI_CLK_RIGHT {
            return E_NOTIMPL; //FIXME
        }
        if click == TF_LBI_CLK_LEFT {
            return E_NOTIMPL; //FIXME
        }
        return E_NOTIMPL;
    }
    pub unsafe fn InitMenu(&mut self, pMenu: *mut ITfMenu,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn OnMenuSelect(&mut self, wID: UINT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetIcon(&mut self, phIcon: *mut HICON,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetText(&mut self, pbstr: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if pbstr.is_null() {
            return E_INVALIDARG;
        }
        *pbstr = SysAllocString((*this_).__base0.m_NewUIInfo.szDescription.as_ptr());
        return if !(*pbstr).is_null() { S_OK } else { E_OUTOFMEMORY };
    }
    pub unsafe fn AdviseSink(&mut self, riid: &IID, punk: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).__base0.AdviseSink(riid, punk, pdwCookie);
    }
    pub unsafe fn UnadviseSink(&mut self, dwCookie: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.UnadviseSink(dwCookie)
    }
}

impl CLBarInatItem {
    pub unsafe fn __ctor_0(&mut self, dwThreadId: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            let mut szText: [WCHAR; 256] = [0; 256];
            LoadStringW(g_hInst, IDS_LANGUAGE, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            (*this_).__base0.__base0.InitNuiInfo(&CLSID_SYSTEMLANGBARITEM, &GUID_LBI_INATITEM, 0x20001, 0, szText.as_mut_ptr());
    
            LoadStringW(g_hInst, IDS_LANGUAGEBUTTON, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            StringCchCopyW((*this_).__base0.__base0.m_szToolTipText.as_mut_ptr(), (_countof!((*this_).__base0.__base0.m_szToolTipText)) as _, szText.as_ptr());
            (*this_).m_dwThreadId = dwThreadId;
            (*this_).m_hKL = GetKeyboardLayout((*this_).m_dwThreadId);
    
            TF_InitMlngInfo();
            (*this_).__base0.__base0.ShowInternal((TF_MlngInfoCount() > 1) as i32, 0);
        }
    }
    pub unsafe fn InitMenu(&mut self, pMenu: *mut ITfMenu) -> HRESULT {
        let this_: *mut Self = self;
        TF_InitMlngInfo();
    
        let mut iKL: INT = 0;
        let cKLs: INT = TF_MlngInfoCount();
        iKL = 0;
        while iKL < cKLs {
            let mut hKL: HKL = core::mem::zeroed();
            let mut szDesc: [WCHAR; 128] = [0 as WCHAR; 128];
            if TF_GetMlngHKL(iKL, core::ptr::addr_of_mut!(hKL), szDesc.as_mut_ptr(), (_countof!(szDesc)) as INT) != 0 {
                let mut hIcon: HICON = core::ptr::null_mut();
                let iIndex: INT = GetIconIndexFromhKL(hKL);
                if iIndex != -1 {
                    hIcon = TF_InatExtractIcon(iIndex);
                }
    
                LangBarInsertMenu(pMenu, iKL as UINT, szDesc.as_ptr(), ((hKL == (*this_).m_hKL) as BOOL), hIcon);
            }
            iKL += 1;
        }
    
        let mut dwStatus: DWORD = core::mem::zeroed();
        if !g_pTipbarWnd.is_null()
            && !(*g_pTipbarWnd).m_pLangBarMgr.is_null()
            && ((*(*g_pTipbarWnd).m_pLangBarMgr).GetShowFloatingStatus(core::ptr::addr_of_mut!(dwStatus)) >= 0)
            && (dwStatus & (TF_SFT_DESKBAND | TF_SFT_MINIMIZED)) != 0
        {
            LangBarInsertSeparator(pMenu);
    
            let mut szText: [WCHAR; 256] = [0 as WCHAR; 256];
            LoadStringW(g_hInst, IDS_RESTORELANGBAR2 as UINT, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            LangBarInsertMenu(pMenu, 2000 as UINT, szText.as_ptr(), (FALSE) as BOOL, core::ptr::null_mut());
        }
    
        return S_OK;
    }
    pub unsafe fn OnMenuSelect(&mut self, nCommandId: INT) -> HRESULT {
        let this_: *mut Self = self;
        let mut hKL: HKL = core::mem::zeroed();
    
        if nCommandId == 2000 {
            if !g_pTipbarWnd.is_null() {
                let pLangBarMgr: *mut ITfLangBarMgr = ((*g_pTipbarWnd).m_pLangBarMgr).cast();
                if !pLangBarMgr.is_null() {
                    (*pLangBarMgr).ShowFloating(TF_SFT_SHOWNORMAL);
                }
            }
        } else if TF_GetMlngHKL((nCommandId as UINT) as INT, &mut hKL, core::ptr::null_mut(), 0) != 0 {
            (*g_pTipbarWnd).RestoreLastFocus(core::ptr::null_mut(), (((*g_pTipbarWnd).m_dwTipbarWndFlags & TIPBAR_CHILD) != 0) as BOOL);
            let hwndFore: HWND = GetForegroundWindow();
            if (*this_).m_dwThreadId == GetWindowThreadProcessId(hwndFore, core::ptr::null_mut()) {
                let FontSig: BOOL = GetFontSig(hwndFore, hKL);
                PostMessage(hwndFore, WM_INPUTLANGCHANGEREQUEST, FontSig as WPARAM, hKL as LPARAM);
            }
        }
    
        return S_OK;
    }
    pub unsafe fn GetIcon(&mut self, phIcon: *mut HICON) -> HRESULT {
        let this_: *mut Self = self;
        let mut hIcon: HICON = core::ptr::null_mut();
        let mut iIndex: INT = GetIconIndexFromhKL((*this_).m_hKL);
        if iIndex != -1 {
            hIcon = TF_InatExtractIcon(iIndex);
        }
        *phIcon = hIcon;
        return S_OK;
    }
    pub unsafe fn GetText(&mut self, pbstr: *mut BSTR) -> HRESULT {
        let this_: *mut Self = self;
        if pbstr.is_null() {
            return E_INVALIDARG;
        }
    
        let mut szText = [0u16; 256];
        if GethKLDesc((*this_).m_hKL, szText.as_mut_ptr(), _countof!(szText) as UINT) == 0 {
            return (*this_).GetText(pbstr);
        }
    
        *pbstr = SysAllocString(szText.as_ptr());
        return S_OK;
    }
}

impl CTipbarThread {
    pub unsafe fn __ctor_0(&mut self, pTipbarWnd: *mut CTipbarWnd) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_pTipbarWnd = pTipbarWnd;
            (*this_).m_dwThreadId = 0;
            (*this_).m_pLangBarItemMgr = core::ptr::null_mut();
            (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pTipbarWnd.is_null() {
            (*this_).RemoveUIObjs();
            (*(*this_).m_pTipbarWnd).CleanUpThreadPointer(this_, 1);
        }
    
        (*this_)._UninitItemList(1);
    
        if !(*this_).m_pLangBarItemMgr.is_null() {
            (*(*this_).m_pLangBarItemMgr).Release();
            (*this_).m_pLangBarItemMgr = core::ptr::null_mut();
        }
    }
    pub unsafe fn Init(&mut self, mut dwThreadId: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_dwThreadId = dwThreadId;
        if TF_GetThreadFlags(
            dwThreadId,
            core::ptr::addr_of_mut!((*this_).m_dwFlags1),
            core::ptr::addr_of_mut!((*this_).m_dwFlags2),
            core::ptr::addr_of_mut!((*this_).m_dwFlags3),
        ) == 0
        {
            return E_FAIL;
        }
        if ((*this_).m_dwFlags1 & 0x8) != 0 {
            return S_OK;
        }
        return (*(*(*this_).m_pTipbarWnd).m_pLangBarMgr).GetThreadLangBarItemMgr(
            (*this_).m_dwThreadId,
            core::ptr::addr_of_mut!((*this_).m_pLangBarItemMgr),
            core::ptr::addr_of_mut!(dwThreadId),
        );
    }
    pub unsafe fn InitItemList(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn _UninitItemList(&mut self, bUnAdvise: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() {
                (*pItem).m_dwItemFlags |= 0x10;
            }
            iItem += 1;
        }
    
        let mut hr: HRESULT = S_OK;
        if bUnAdvise != 0 {
            if (*this_).m_dwThreadId == GetCurrentThreadId() || !(MyWaitForInputIdle((*this_).m_dwThreadId, 2000) != 0) {
                hr = (*this_)._UnadviseItemsSink();
            }
        }
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() {
                if !(*this_).m_pTipbarWnd.is_null() {
                    (*pItem).OnUnknown47((*(*this_).m_pTipbarWnd).GetWindow());
                }
    
                (*pItem).ClearConnections();
    
                if !(*this_).m_pTipbarWnd.is_null() {
                    (*pItem).OnUnknown50();
                } else {
                    (*pItem).OnUnknown51();
                }
    
                (*pItem).OnUnknown59();
                (*pItem).OnUnknown42();
            }
            iItem += 1;
        }
    
        (*this_).m_UIObjects.clear();
    
        (*this_).RemoveAllSeparators();
    
        return hr;
    }
    pub unsafe fn IsDirtyItem(&mut self) -> DWORD {
        let this_: *mut Self = self;
        let mut dwDirty: DWORD = 0;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() {
                dwDirty |= (*pItem).m_dwDirty;
            }
            iItem += 1;
        }
        return dwDirty;
    }
    pub unsafe fn IsFocusThread(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarWnd.is_null() {
            return (FALSE) as BOOL;
        }
        return (this_ == (*(*this_).m_pTipbarWnd).m_pFocusThread) as BOOL;
    }
    pub unsafe fn IsVertical(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarWnd.is_null() {
            return (FALSE) as BOOL;
        }
        return (((*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as BOOL;
    }
    pub unsafe fn AddAllSeparators(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Separators.size() {
            let pItem: *mut CUIFObject = (*this_).m_Separators[iItem];
            if !pItem.is_null() {
                (*(*this_).m_pTipbarWnd).__base3.AddUIObj(pItem);
            }
            iItem += 1;
        }
    }
    pub unsafe fn RemoveAllSeparators(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Separators.size() {
            let pItem: *mut CUIFObject = (*this_).m_Separators[iItem];
            if !pItem.is_null() {
                if !(*this_).m_pTipbarWnd.is_null() {
                    (*(*this_).m_pTipbarWnd).__base3.RemoveUIObj(pItem);
                }
                cpp_delete(pItem);
            }
            iItem += 1;
        }
        (*this_).m_Separators.clear();
    }
    pub unsafe fn AddUIObjs(&mut self) {
        let this_: *mut Self = self;
        (*this_)._AddRef();
    
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() && ((*pItem).m_dwItemFlags & 0x8) != 0 {
                let hwnd = if !(*this_).m_pTipbarWnd.is_null() {
                    (*(*this_).m_pTipbarWnd).GetWindow()
                } else {
                    core::ptr::null_mut()
                };
                (*pItem).OnUnknown46(hwnd);
            }
            iItem += 1;
        }
    
        (*this_).AddAllSeparators();
        (*this_).MyMoveWnd(0, 0);
    
        (*this_)._Release();
    }
    pub unsafe fn RemoveUIObjs(&mut self) {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() {
                (*pItem).OnUnknown47(if !(*this_).m_pTipbarWnd.is_null() {
                    (*(*this_).m_pTipbarWnd).GetWindow()
                } else {
                    core::ptr::null_mut()
                });
            }
            iItem += 1;
        }
        (*this_).RemoveAllSeparators();
    }
    pub unsafe fn GetItem(&mut self, rclsid: &CLSID) -> *mut CTipbarItem {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() && IsEqualCLSID!(&(*pItem).m_ItemInfo.guidItem, rclsid) {
                return pItem;
            }
            iItem += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn GetTextSize(&mut self, bstr: BSTR, pSize: LPSIZE) {
        let this_: *mut Self = self;
        let hWnd: HWND = (*(*(*this_).m_pTipbarWnd).GetWindow()).m_hWnd;
    
        let mut hFontOld: HGDIOBJ = core::ptr::null_mut();
    
        let hDC: HDC = GetDC(hWnd);
        if (*(*this_).m_pTipbarWnd).__base3.EnsureThemeData((*(*(*this_).m_pTipbarWnd).GetWindow()).m_hWnd) < 0 {
            let hFont: HFONT = (*(*this_).m_pTipbarWnd).__base3.__base0.m_hFont;
            if !hFont.is_null() {
                hFontOld = SelectObject(hDC, (hFont).cast());
            }
            let cch: INT = (SysStringLen(bstr)) as i32;
            GetTextExtentPoint32W(hDC, bstr, cch, pSize);
            if !hFontOld.is_null() {
                SelectObject(hDC, hFontOld);
            }
        } else {
            let mut theme: CUIFTheme = CUIFTheme__zeroed();
            theme.m_iPartId = 1;
            theme.m_iStateId = 0;
            theme.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
    
            let mut hFont: HFONT = core::ptr::null_mut();
    
            if theme.InternalOpenThemeData(hWnd) >= 0 {
                let mut lf: LOGFONTW = core::mem::zeroed::<LOGFONTW>();
                if GetThemeFont(theme.m_hTheme, core::ptr::null_mut(), theme.m_iPartId, 0, 210, core::ptr::addr_of_mut!(lf)) >= 0 {
                    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
                    if !hFont.is_null() {
                        hFontOld = SelectObject(hDC, (hFont).cast());
                    }
                }
    
                let mut rc: RECT = core::mem::zeroed::<RECT>();
                let cch: INT = (SysStringLen(bstr)) as i32;
                GetThemeTextExtent(theme.m_hTheme, hDC, theme.m_iPartId, 0, bstr, cch, 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(rc));
    
                (*pSize).cx = rc.right;
                (*pSize).cy = rc.bottom;
            }
    
            if !hFontOld.is_null() {
                SelectObject(hDC, hFontOld);
            }
            if !hFont.is_null() {
                DeleteObject((hFont).cast());
            }
        }
    
        ReleaseDC(hWnd, hDC);
    }
    pub unsafe fn LocateItems(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn MyMoveWnd(&mut self, xDelta: LONG, yDelta: LONG) {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarWnd.is_null() || (*(*this_).m_pTipbarWnd).m_pFocusThread != this_ {
            return;
        }
    
        let mut Rect: RECT = core::mem::zeroed();
        let mut rcWorkArea: RECT = core::mem::zeroed();
        (*(*this_).m_pTipbarWnd).__base3.GetRect(core::ptr::addr_of_mut!(Rect));
        let pt: POINT = POINT { x: Rect.left, y: Rect.top };
        cicGetWorkAreaRect(pt, core::ptr::addr_of_mut!(rcWorkArea));
    
        GetWindowRect((*(*(*this_).m_pTipbarWnd).GetWindow()).m_hWnd, core::ptr::addr_of_mut!(Rect));
    
        let mut X0: LONG = Rect.left + xDelta;
        let mut Y0: LONG = Rect.top + yDelta;
        if (*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags & 0x1000 != 0 {
            if (*(*this_).m_pTipbarWnd).CheckExcludeCaptionButtonMode(
                core::ptr::addr_of_mut!(Rect),
                core::ptr::addr_of_mut!(rcWorkArea),
            ) != 0
            {
                X0 = rcWorkArea.right
                    - (3 * (*(*this_).m_pTipbarWnd).m_ButtonWidth
                        + (*(*this_).m_pTipbarWnd).m_cxDlgFrameX2
                        + (*this_).m_cxGrip);
                Y0 = 0;
            } else {
                (*(*this_).m_pTipbarWnd).m_dwTipbarWndFlags &= !(0x1000);
            }
        }
    
        if (*this_).IsVertical() != 0 {
            let width: LONG = (*(*this_).m_pTipbarWnd).m_cxDlgFrameX2
                + (*(*this_).m_pTipbarWnd).GetTipbarHeight();
            let height: LONG = (*this_).m_cyGrip + (*(*this_).m_pTipbarWnd).m_cyDlgFrameX2;
            (*(*this_).m_pTipbarWnd).SetMoveRect(X0, Y0, width, height);
        } else {
            let width: LONG = (*this_).m_cxGrip + (*(*this_).m_pTipbarWnd).m_cxDlgFrameX2;
            let height: LONG = (*(*this_).m_pTipbarWnd).m_cyDlgFrameX2
                + (*(*this_).m_pTipbarWnd).GetTipbarHeight();
            (*(*this_).m_pTipbarWnd).SetMoveRect(X0, Y0, width, height);
        }
    
        let mut frameSize: SIZE = SIZE { cx: 0, cy: 0 };
        if !(*(*this_).m_pTipbarWnd).m_pWndFrame.is_null() {
            (*(*(*this_).m_pTipbarWnd).m_pWndFrame).GetFrameSize(core::ptr::addr_of_mut!(frameSize));
        }
    
        (*(*this_).m_pTipbarWnd).LocateCtrlButtons();
        (*(*this_).m_pTipbarWnd).AutoAdjustDeskBandSize();
    }
    pub unsafe fn _UnadviseItemsSink(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pLangBarItemMgr.is_null() {
            return E_FAIL;
        }
    
        let pdwCoolkies: *mut DWORD = cpp_new_array_CicNoThrow::<DWORD>((*this_).m_UIObjects.size());
        if pdwCoolkies.is_null() {
            return E_OUTOFMEMORY;
        }
    
        let cItems: usize = (*this_).m_UIObjects.size();
        let mut iItem: usize = 0;
        while iItem < cItems {
            let pItem: *mut CTipbarItem = (*this_).m_UIObjects[iItem];
            if !pItem.is_null() {
                *pdwCoolkies.add(iItem) = (*pItem).m_dwCookie;
            }
            iItem += 1;
        }
    
        let hr: HRESULT = (*(*this_).m_pLangBarItemMgr).UnadviseItemsSink((cItems as LONG) as ULONG, pdwCoolkies);
    
        cpp_delete_array(pdwCoolkies);
    
        return hr;
    }
    pub unsafe fn _AddRef(&mut self) -> LONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs += 1;
        (*this_).m_cRefs
    }
    pub unsafe fn _Release(&mut self) -> LONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            CTipbarThread__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs;
    }
    pub unsafe fn SetFocus(&mut self, pTarget: *mut CTipbarBalloonItem,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE) as i32
    }
    pub unsafe fn CallOnUpdateHandler(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}

impl CTipbarItem {
    pub unsafe fn __ctor_0(&mut self, pThread: *mut CTipbarThread, pLangBarItem: *mut ITfLangBarItem, pItemInfo: *mut TF_LANGBARITEMINFO, dwUnknown16: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).m_dwUnknown19[1 as usize] = 0;
            (*this_).m_dwUnknown19[2 as usize] = 0;
            (*this_).m_dwUnknown19[3 as usize] = 0;
            (*this_).m_pTipbarThread = pThread;
            (*this_).m_ItemInfo = *pItemInfo;
            (*this_).m_pLangBarItem = pLangBarItem;
            (*(*this_).m_pLangBarItem).AddRef();
            (*this_).m_dwItemFlags = 0;
            (*this_).m_dwUnknown16 = dwUnknown16;
            (*this_).m_dwDirty = 0x1001F;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !g_pTipbarWnd.is_null() {
            if !(*g_pTipbarWnd).m_pTipbarAccessible.is_null() {
                (*(*g_pTipbarWnd).m_pTipbarAccessible).RemoveAccItem((this_).cast());
            }
        }
    
        if !(*this_).m_pLangBarItem.is_null() {
            (*(*this_).m_pLangBarItem).Release();
        }
    }
    pub unsafe fn _AddedToUI(&mut self) {
        let this_: *mut Self = self;
        if (*this_).IsConnected() == 0 {
            return;
        }
    
        (*this_).OnUnknown41();
    
        (*this_).m_dwItemFlags |= 0x2;
    
        let mut dwStatus: DWORD = 0;
        if (*this_).m_dwDirty != 0 {
            if ((*this_).m_dwDirty & 0x10000) != 0 {
                (*(*this_).m_pLangBarItem).GetStatus(core::ptr::addr_of_mut!(dwStatus));
            } else {
                dwStatus = 0;
            }
            (*this_).OnUnknown45((*this_).m_dwDirty, dwStatus);
            (*this_).m_dwDirty = 0;
        }
    
        if !(*this_).m_pTipbarThread.is_null() {
            let mut pTipbarWnd: *mut CTipbarWnd = (*(*this_).m_pTipbarThread).m_pTipbarWnd;
            if !pTipbarWnd.is_null() {
                let mut pTipbarAccessible: *mut CTipbarAccessible = (*pTipbarWnd).m_pTipbarAccessible;
                if !pTipbarAccessible.is_null() {
                    (*pTipbarAccessible).AddAccItem((this_).cast());
                }
            }
        }
    
        (*this_).OnUnknown42();
    }
    pub unsafe fn _RemovedToUI(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwItemFlags &= !0x2;
    
        if !g_pTipbarWnd.is_null() {
            let pAccessible: *mut CTipbarAccessible = (*g_pTipbarWnd).m_pTipbarAccessible;
            if !pAccessible.is_null() {
                (*pAccessible).RemoveAccItem(this_ as *mut CTipbarAccItem);
            }
        }
    }
    pub unsafe fn AddRemoveMeToUI(&mut self, bFlag: BOOL) {
        let this_: *mut Self = self;
        if (*this_).IsConnected() == 0 {
            return;
        }
    
        (*(*this_).m_pTipbarThread).LocateItems();
    
        if (*this_).IsConnected() == 0 {
            return;
        }
    
        (*(*this_).m_pTipbarThread).AddAllSeparators();
    
        let pTipbarWnd: *mut CTipbarWnd = (*(*this_).m_pTipbarThread).m_pTipbarWnd;
        if bFlag != 0 {
            (*this_).OnUnknown46(if !pTipbarWnd.is_null() {
                (*pTipbarWnd).GetWindow()
            } else {
                core::ptr::null_mut()
            });
        } else {
            (*this_).OnUnknown47(if !pTipbarWnd.is_null() {
                (*pTipbarWnd).GetWindow()
            } else {
                core::ptr::null_mut()
            });
        }
    }
    pub unsafe fn IsConnected(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return (((*this_).m_dwItemFlags & 0x10) == 0
            && !(*this_).m_pTipbarThread.is_null()
            && !(*(*this_).m_pTipbarThread).m_pTipbarWnd.is_null()
            && !(*this_).m_pLangBarItem.is_null()) as BOOL;
    }
    pub unsafe fn ClearConnections(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_pTipbarThread = core::ptr::null_mut();
        if !(*this_).m_pLangBarItem.is_null() {
            (*(*this_).m_pLangBarItem).Release();
            (*this_).m_pLangBarItem = core::ptr::null_mut();
        }
    }
    pub unsafe fn StartDemotingTimer(&mut self, bStarted: BOOL) {
        let this_: *mut Self = self;
        if g_bIntelliSense == 0 {
            return;
        }
    
        if (*this_).m_pTipbarThread.is_null() {
            return;
        }
    
        let pTipbarWnd: *mut CTipbarWnd = (*(*this_).m_pTipbarThread).m_pTipbarWnd;
        if pTipbarWnd.is_null() {
            return;
        }
    
        //FIXME
    }
    pub unsafe fn MyClientToScreen__0(&mut self, ppt: LPPOINT, prc: LPRECT) {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarThread.is_null() {
            return;
        }
        if !(*(*this_).m_pTipbarThread).m_pTipbarWnd.is_null() {
            (*(*(*this_).m_pTipbarThread).m_pTipbarWnd).MyClientToScreen(ppt, prc);
        }
    }
    pub unsafe fn MyClientToScreen__1(&mut self, prc: LPRECT) {
        let this_: *mut Self = self;
        return (*this_).MyClientToScreen__0(core::ptr::null_mut(), prc);
    }
    pub unsafe fn GetAccName__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        return SysAllocString((*this_).m_ItemInfo.szDescription.as_ptr() as *const u16);
    }
    pub unsafe fn GetAccLocation__body(&mut self, prc: LPRECT) {
        let this_: *mut Self = self;
        let _ = (*this_).OnUnknown57(prc);
    }
    pub unsafe fn DoAccDefaultAction__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarThread.is_null() {
            return (FALSE) as BOOL;
        }
        let pTipbarWnd: *mut CTipbarWnd = (*(*this_).m_pTipbarThread).m_pTipbarWnd;
        if pTipbarWnd.is_null() {
            return (FALSE) as BOOL;
        }
        (*pTipbarWnd).StartDoAccDefaultActionTimer(this_);
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnUnknown40__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown41__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown42__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown43__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUpdate__body(&mut self, dwDirty: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).IsConnected() == 0 {
            return S_OK;
        }
    
        (*this_).m_dwDirty |= dwDirty;
        (*this_).m_dwItemFlags |= 0x20;
    
        if (dwDirty & 0x10000) != 0 || ((*this_).m_dwItemFlags & 0x6) != 0 {
            if !(*this_).m_pTipbarThread.is_null() {
                let pTipBarWnd: *mut CTipbarWnd = (*(*this_).m_pTipbarThread).m_pTipbarWnd;
                // C: `pTipBarWnd && *pTipBarWnd` -- `*pTipBarWnd` is the class's operator BOOL,
                // which is not a generated method here; the pointer test stands in for it.
                if !pTipBarWnd.is_null() {
                    (*pTipBarWnd).KillTimer(6);
                    (*pTipBarWnd).SetTimer(6, g_uTimerElapseONUPDATECALLED);
                }
            }
        }
    
        return S_OK;
    }
    pub unsafe fn OnUnknown44__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown45__body(&mut self, dwDirty: DWORD,
        dwStatus: DWORD,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUpdateHandler__body(&mut self, arg0: ULONG,
        arg1: ULONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUnknown46__body(&mut self, pWindow: *mut CUIFWindow,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown47__body(&mut self, pWindow: *mut CUIFWindow,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown48__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown49__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown50__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown51__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown52__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown53__body(&mut self, bstr: BSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown55__body(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        core::ptr::null()
    }
    pub unsafe fn OnUnknown56__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn GetToolTip__body(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        (*this_).OnUnknown41();
    
        if ((*this_).m_dwItemFlags & 0x1) == 0 {
            (*this_).m_dwItemFlags |= 0x1;
    
            let mut bstrString: BSTR = core::ptr::null_mut();
            let hr: HRESULT = (*(*this_).m_pLangBarItem).GetTooltipString(core::ptr::addr_of_mut!(bstrString));
            if hr < 0 {
                return core::ptr::null_mut();
            }
    
            if !bstrString.is_null() {
                (*this_).OnUnknown53(bstrString);
                SysFreeString(bstrString);
            }
        }
    
        let pszToolTip: LPCWSTR = (*this_).OnUnknown55();
    
        (*this_).OnUnknown42();
    
        pszToolTip
    }
    pub unsafe fn OnUnknown57__body(&mut self, prc: LPRECT,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown58__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnUnknown59__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUnknown60__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUnknown61__body(&mut self, arg0: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUnknown62__body(&mut self, arg0: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUnknown63__body(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
}

impl CTipbarWnd {
    pub unsafe fn __ctor_0(&mut self, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base2.__ctor_0();
        (*this_).__init_vtables();
        {
        (*this_).m_dwUnknown23_1[4] = 0;
        (*this_).m_dwUnknown23_1[5] = 0;
        (*this_).m_dwUnknown23_1[6] = 0;
        (*this_).m_dwUnknown23_1[7] = 0;
    
        let mut rc: RECT = core::mem::zeroed();
        cicGetScreenRect(g_ptTipbar, core::ptr::addr_of_mut!(rc));
    
        //FIXME: Fix g_ptTipbar
    
        (*this_).Move(g_ptTipbar.x, g_ptTipbar.y, 100, 24);
        (*this_).UpdatePosFlags();
    
        (*this_).m_hMarlettFont = CreateFontW(8, 8, 0, 0, (FW_NORMAL as i32), 0, 0, 0,
                                       SYMBOL_CHARSET, 0, 0, 0, 0, CPP_COM_WSTR_B728528EE8A8.as_ptr());
    
        let mut pLangBarMgr: *mut ITfLangBarMgr = core::ptr::null_mut();
        if ((TF_CreateLangBarMgr(core::ptr::addr_of_mut!(pLangBarMgr))) >= 0) && !pLangBarMgr.is_null()
        {
            (*pLangBarMgr).QueryInterface(&IID_ITfLangBarMgr_P, core::ptr::addr_of_mut!((*this_).m_pLangBarMgr) as *mut *mut core::ffi::c_void);
            (*pLangBarMgr).Release();
        }
    
        if style & UIF_WINDOW_ENABLETHEMED != 0
        {
            if g_fTaskbarTheme != 0
            {
                (*this_).__base3.__base0.__base0.m_iPartId = 1;
                (*this_).__base3.__base0.__base0.m_iStateId = 1;
                (*this_).__base3.__base0.__base0.m_pszClassList = CPP_COM_WSTR_D3128F43EECC.as_ptr();
            }
            else
            {
                (*this_).__base3.__base0.__base0.m_iPartId = 0;
                (*this_).__base3.__base0.__base0.m_iStateId = 1;
                (*this_).__base3.__base0.__base0.m_pszClassList = CPP_COM_WSTR_672463267A33.as_ptr();
            }
        }
    
        (*this_).SetVertical(g_fVertical);
    
        (*this_).m_cRefs = 1;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).UnInit();
    
        if !(*this_).m_hMarlettFont.is_null() {
            DeleteObject(((*this_).m_hMarlettFont).cast());
        }
        if !(*this_).m_hTextFont.is_null() {
            DeleteObject(((*this_).m_hTextFont).cast());
        }
    
        TFUninitLib_Thread(core::ptr::addr_of_mut!(g_libTLS));
    }
    pub unsafe fn GetWindow(&mut self) -> *mut CUIFWindow {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).__base3) as *mut CUIFWindow;
    }
    pub unsafe fn GetAccItem(&mut self) -> *mut CTipbarAccItem {
        let this_: *mut Self = self;
        return (this_ as *mut u8).add(core::mem::offset_of!(CTipbarWnd, __base2)) as *mut CTipbarAccItem;
    }
    pub unsafe fn Init(&mut self, bChild: BOOL, pDeskBand: *mut CDeskBand) {
        let this_: *mut Self = self;
        if bChild != 0 {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_CHILD;
        } else {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_CHILD;
        }
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) != 0 {
            (*this_).m_dwTipbarWndFlags &= TIPBAR_HIGHCONTRAST;
        }
    
        (*this_).m_pDeskBand = pDeskBand;
    
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
    
        if g_bNewLook != 0 && (*this_).m_pWndFrame.is_null() && ((*this_).__base3.__base0.m_style & 0x20000000) != 0 {
            let pWndFrame: *mut CUIFWndFrame = cicero_rust::cicero::cpp_classes::CUIFWndFrame__new_in_0(
                cpp_alloc_CicNoThrow::<CUIFWndFrame>(),
                (*this_).GetWindow() as *mut CUIFObject,
                core::ptr::addr_of_mut!(rc),
                0,
            );
            if !pWndFrame.is_null() {
                (*pWndFrame).Initialize();
                (*this_).__base3.AddUIObj(((*this_).m_pWndFrame).cast());
            }
        }
    
        if (*this_).m_pTipbarGripper.is_null() && ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) == 0 {
            (*this_).m_pTipbarGripper = CTipbarGripper__new_in_0(
                cpp_alloc_CicNoThrow::<CTipbarGripper>(),
                this_,
                core::ptr::addr_of_mut!(rc),
                ((((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as BOOL) as DWORD,
            );
            if !(*this_).m_pTipbarGripper.is_null() {
                (*(*this_).m_pTipbarGripper).Initialize();
                (*this_).__base3.AddUIObj((core::ptr::addr_of_mut!((*(*this_).m_pTipbarGripper).__base0)).cast());
            }
        }
    
        // FIXME: CTipbarCtrlButtonHolder
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0 {
            (*this_).Move(
                (*this_).__base3.m_nLeft,
                (*this_).__base3.m_nTop,
                (*this_).GetTipbarHeight(),
                0,
            );
        } else {
            (*this_).Move(
                (*this_).__base3.m_nLeft,
                (*this_).__base3.m_nTop,
                0,
                (*this_).GetTipbarHeight(),
            );
        }
    }
    pub unsafe fn InitHighContrast(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwTipbarWndFlags &= !TIPBAR_HIGHCONTRAST;
    
        let mut HiCon: HIGHCONTRAST = core::mem::zeroed();
        HiCon.cbSize = core::mem::size_of::<HIGHCONTRAST>() as UINT;
        if SystemParametersInfo(SPI_GETHIGHCONTRAST, core::mem::size_of::<HIGHCONTRAST>() as UINT, core::ptr::addr_of_mut!(HiCon) as LPVOID, 0) != 0
        {
            if (HiCon.dwFlags & HCF_HIGHCONTRASTON) != 0 {
                (*this_).m_dwTipbarWndFlags |= TIPBAR_HIGHCONTRAST;
            }
        }
    }
    pub unsafe fn InitMetrics(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_cxSmallIcon = GetSystemMetrics((SM_CXSMICON as i32));
        (*this_).m_cySmallIcon = GetSystemMetrics((SM_CYSMICON as i32));
    
        let style: DWORD_PTR = GetWindowLongPtr((*this_).__base3.m_hWnd, GWL_STYLE) as DWORD_PTR;
        if (style & (WS_DLGFRAME as DWORD_PTR)) != 0
        {
            (*this_).m_cxDlgFrameX2 = 2 * GetSystemMetrics((SM_CXDLGFRAME as i32));
            (*this_).m_cyDlgFrameX2 = 2 * GetSystemMetrics((SM_CYDLGFRAME as i32));
        }
        else if (style & (WS_BORDER as DWORD_PTR)) != 0
        {
            (*this_).m_cxDlgFrameX2 = 2 * GetSystemMetrics((SM_CXBORDER as i32));
            (*this_).m_cyDlgFrameX2 = 2 * GetSystemMetrics((SM_CYBORDER as i32));
        }
        else
        {
            (*this_).m_cyDlgFrameX2 = 0;
            (*this_).m_cxDlgFrameX2 = (*this_).m_cyDlgFrameX2;
        }
    }
    pub unsafe fn InitThemeMargins(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).m_Margins) as *mut core::ffi::c_void, core::mem::size_of::<MARGINS>());
    
        let mut theme: CUIFTheme = CUIFTheme__zeroed();
        (*this_).m_dwUnknown23_5[0 as usize] = 6;
        (*this_).m_dwUnknown23_5[1 as usize] = 6;
        (*this_).m_dwUnknown23_5[2 as usize] = 0;
        (*this_).m_ButtonWidth = GetSystemMetrics((SM_CXSIZE as i32));
    
        theme.m_iPartId = 1;
        theme.m_iStateId = 0;
        theme.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
        if (theme.InternalOpenThemeData((*this_).__base3.m_hWnd)) >= 0
        {
            GetThemeMargins(theme.m_hTheme, core::ptr::null_mut(), theme.m_iPartId, 1, 3602, core::ptr::null_mut(), core::ptr::addr_of_mut!((*this_).m_Margins));
            (*this_).m_dwUnknown23_5[0 as usize] = 4;
            (*this_).m_dwUnknown23_5[1 as usize] = 2;
            (*this_).m_dwUnknown23_5[2 as usize] = 1;
        }
        theme.CloseThemeData();
    
        theme.m_iPartId = 18;
        theme.m_iStateId = 0;
        theme.m_pszClassList = CPP_COM_WSTR_DDBF4DF411E1.as_ptr();
        if (theme.InternalOpenThemeData((*this_).__base3.m_hWnd)) >= 0
        {
            let mut partSize: SIZE = core::mem::zeroed();
            GetThemePartSize(theme.m_hTheme, core::ptr::null_mut(), theme.m_iPartId, 1, core::ptr::null_mut(), TS_TRUE, core::ptr::addr_of_mut!(partSize));
            let mut size: INT = GetThemeSysSize(theme.m_hTheme, 31);
            (*this_).m_ButtonWidth = MulDiv(size, partSize.cx, partSize.cy);
        }
        theme.CloseThemeData();
    }
    pub unsafe fn UnInit(&mut self) {
        let this_: *mut Self = self;
        (*this_).SetFocusThread(core::ptr::null_mut());
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Threads.size() {
            let pThread: *mut CTipbarThread = (*this_).m_Threads[iItem as size_t];
            if !pThread.is_null() {
                (*pThread)._UninitItemList((TRUE) as BOOL);
                (*pThread).m_pTipbarWnd = core::ptr::null_mut();
                (*pThread)._Release();
            }
            iItem += 1;
        }
        (*this_).m_Threads.clear();
    
        if !(*this_).m_pLangBarMgr.is_null() {
            (*(*this_).m_pLangBarMgr).UnAdviseEventSink((*this_).m_dwSinkCookie);
        }
    
        if !(*this_).m_pLangBarMgr.is_null() {
            (*(*this_).m_pLangBarMgr).Release();
            (*this_).m_pLangBarMgr = core::ptr::null_mut();
        }
    }
    pub unsafe fn IsFullScreenWindow(&mut self, hWnd: HWND) -> BOOL {
        let this_: *mut Self = self;
        if g_fPolicyEnableLanguagebarInFullscreen != 0 {
            return (FALSE) as BOOL;
        }
    
        let style: DWORD_PTR = GetWindowLongPtr(hWnd, GWL_STYLE) as DWORD_PTR;
        if ((style & (WS_VISIBLE as DWORD_PTR)) == 0) || ((style & (WS_CAPTION as DWORD_PTR)) != 0) {
            return (FALSE) as BOOL;
        }
    
        let exstyle: DWORD_PTR = GetWindowLongPtr(hWnd, GWL_EXSTYLE) as DWORD_PTR;
        if (exstyle & (WS_EX_LAYERED as DWORD_PTR)) != 0 {
            return (FALSE) as BOOL;
        }
    
        if ((exstyle & (WS_EX_TOOLWINDOW as DWORD_PTR)) != 0) && (hWnd == (*this_).m_ShellWndThread.GetWndProgman()) {
            return (FALSE) as BOOL;
        }
    
        ((cicIsFullScreenSize(hWnd) != 0) as BOOL)
    }
    pub unsafe fn IsHKLToSkipRedrawOnNoItem(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let mut hKL: HKL = (*this_).GetFocusKeyboardLayout();
        return IsSkipRedrawHKL(hKL);
    }
    pub unsafe fn IsInItemChangeOrDirty(&mut self, pTarget: *mut CTipbarThread) -> BOOL {
        let this_: *mut Self = self;
        if (*pTarget).m_dwThreadId == (*this_).m_dwChangingThreadId {
            return (TRUE) as BOOL;
        }
        return (*pTarget).IsDirtyItem() as BOOL;
    }
    pub unsafe fn AddThreadToThreadCreatingList(&mut self, pThread: *mut CTipbarThread) {
        let this_: *mut Self = self;
        (*this_).m_ThreadCreatingList.Add(pThread);
    }
    pub unsafe fn RemoveThredFromThreadCreatingList(&mut self, pTarget: *mut CTipbarThread) {
        let this_: *mut Self = self;
        let iItem: isize = (*this_).m_ThreadCreatingList.Find(pTarget);
        if iItem >= 0 {
            (*this_).m_ThreadCreatingList.Remove((iItem) as usize, 1);
        }
    }
    pub unsafe fn MoveToStub(&mut self, bFlag: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_dwTipbarWndFlags |= 0x40;
    
        let mut rcWorkArea: RECT = core::mem::zeroed();
        SystemParametersInfo(SPI_GETWORKAREA, 0, (core::ptr::addr_of_mut!(rcWorkArea)).cast(), 0);
    
        if bFlag != 0 {
            (*this_).__base3.m_nLeft = rcWorkArea.right - 38;
            (*this_).m_dwTipbarWndFlags &= !0x80;
        } else {
            let mut Rect: RECT = core::mem::zeroed();
            GetWindowRect((*this_).__base3.m_hWnd, core::ptr::addr_of_mut!(Rect));
            (*this_).__base3.m_nLeft = rcWorkArea.right + Rect.left - Rect.right;
            (*this_).m_dwTipbarWndFlags |= 0x80;
        }
    
        (*this_).__base3.m_nTop = rcWorkArea.bottom - (*this_).m_cyDlgFrameX2 - (*this_).GetTipbarHeight();
    
        if !(*this_).m_pFocusThread.is_null() {
            (*(*this_).m_pFocusThread).MyMoveWnd(0, 0);
        }
    }
    pub unsafe fn RestoreFromStub(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwTipbarWndFlags &= 0x3F;
        (*this_).KillTimer(1);
        (*this_).KillTimer(2);
    }
    pub unsafe fn GetCtrlButtonWidth(&mut self) -> INT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn GetGripperWidth(&mut self) -> INT {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & 2) != 0 {
            return 0;
        }
    
        if (*this_).m_pTipbarGripper.is_null() || (((*(*this_).m_pTipbarGripper).EnsureThemeData((*this_).__base3.m_hWnd)) < 0) {
            return 5;
        }
    
        let mut width: INT = -1;
        let mut partSize: SIZE = core::mem::zeroed();
        let hDC: HDC = GetDC((*this_).__base3.m_hWnd);
        if ((*(*this_).m_pTipbarGripper).GetThemePartSize(hDC, 1, core::ptr::null_mut(), TS_TRUE, core::ptr::addr_of_mut!(partSize)) >= 0)
        {
            let mut cx: INT = partSize.cx;
            if ((*this_).m_dwTipbarWndFlags & 4) != 0 {
                cx = partSize.cy;
            }
            width = cx + 4;
        }
        ReleaseDC((*this_).__base3.m_hWnd, hDC);
    
        return if width < 0 { 5 } else { width };
    }
    pub unsafe fn GetTipbarHeight(&mut self) -> INT {
        let this_: *mut Self = self;
        let mut size: SIZE = SIZE { cx: 0, cy: 0 };
        if !(*this_).m_pWndFrame.is_null() {
            (*(*this_).m_pWndFrame).GetFrameSize(core::ptr::addr_of_mut!(size));
        }
        let mut cy: INT = (*this_).m_Margins.cyBottomHeight + (*this_).m_Margins.cyTopHeight + 2;
        if cy < 6 {
            cy = 6;
        }
        return (*this_).m_cySmallIcon + cy + (2 * size.cy);
    }
    pub unsafe fn AutoAdjustDeskBandSize(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_NODESKBAND) != 0
            || (*this_).m_pFocusThread.is_null()
            || (((*(*this_).m_pFocusThread).m_dwFlags1) & 0x800) != 0
        {
            return (FALSE) as BOOL;
        }
    
        let dwOldWndFlags: DWORD = (*this_).m_dwTipbarWndFlags;
        (*this_).m_dwTipbarWndFlags &= !0x8000;
    
        if (*this_).AdjustDeskBandSize(((dwOldWndFlags & 0x8000) == 0) as BOOL) == 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).m_dwTipbarWndFlags |= TIPBAR_NODESKBAND;
        return (TRUE) as BOOL;
    }
    pub unsafe fn AdjustDeskBandSize(&mut self, bFlag: BOOL,
    ) -> INT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn LocateCtrlButtons(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn AdjustPosOnDisplayChange(&mut self) {
        let this_: *mut Self = self;
        let mut rcWorkArea: RECT = core::mem::zeroed();
        let mut rc: RECT = RECT {
            left: (*this_).__base3.m_nLeft,
            top: (*this_).__base3.m_nTop,
            right: (*this_).__base3.m_nLeft + (*this_).__base3.m_nWidth,
            bottom: (*this_).__base3.m_nTop + (*this_).__base3.m_nHeight,
        };
        if (*this_).__base3.GetWorkArea(core::ptr::addr_of_mut!(rc), core::ptr::addr_of_mut!(rcWorkArea)) == 0 {
            return;
        }
    
        let mut x: INT = (*this_).__base3.m_nLeft;
        let mut y: INT = (*this_).__base3.m_nTop;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_LEFTFIT) != 0 {
            x = rcWorkArea.left;
        }
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_TOPFIT) != 0 {
            y = rcWorkArea.top;
        }
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_RIGHTFIT) != 0 {
            x = rcWorkArea.right - (*this_).__base3.m_nWidth;
        }
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_BOTTOMFIT) != 0 {
            y = rcWorkArea.bottom - (*this_).__base3.m_nHeight;
        }
        if x != (*this_).__base3.m_nLeft || y != (*this_).__base3.m_nTop {
            (*this_).Move(x, y, (*this_).__base3.m_nWidth, (*this_).__base3.m_nHeight);
        }
    }
    pub unsafe fn SetVertical(&mut self, bVertical: BOOL) {
        let this_: *mut Self = self;
        if bVertical != 0 {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_VERTICAL;
        } else {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_VERTICAL;
        }
    
        if !(*this_).m_pTipbarGripper.is_null() {
            let mut style: DWORD = (*(*this_).m_pTipbarGripper).m_style;
            if bVertical != 0 {
                style |= 0x1;
            } else {
                style &= 0x1;
            }
            (*(*this_).m_pTipbarGripper).SetStyle(style);
        }
    
        if g_fTaskbarTheme != 0 {
            (*this_).__base3.SetActiveTheme(
                CPP_COM_WSTR_D3128F43EECC.as_ptr(),
                (((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as BOOL,
                1,
            );
        }
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) == 0 {
            if ((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0 {
                (*this_).Move(
                    (*this_).__base3.m_nLeft,
                    (*this_).__base3.m_nTop,
                    (*this_).GetTipbarHeight(),
                    0,
                );
            } else {
                (*this_).Move(
                    (*this_).__base3.m_nLeft,
                    (*this_).__base3.m_nTop,
                    0,
                    (*this_).GetTipbarHeight(),
                );
            }
        }
    
        if !(*this_).__base3.m_hWnd.is_null() {
            (*this_).KillTimer(7);
            (*this_).SetTimer(7, g_uTimerElapseSYSCOLORCHANGED);
        }
    }
    pub unsafe fn UpdatePosFlags(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_dwTipbarWndFlags & TIPBAR_CHILD != 0 {
            return;
        }
    
        let mut rc: RECT = RECT {
            left: (*this_).__base3.m_nLeft,
            top: (*this_).__base3.m_nTop,
            right: (*this_).__base3.m_nLeft + (*this_).__base3.m_nWidth,
            bottom: (*this_).__base3.m_nTop + (*this_).__base3.m_nHeight,
        };
        let mut rcWorkArea: RECT = core::mem::zeroed();
        if (*this_).__base3.GetWorkArea(core::ptr::addr_of_mut!(rc), core::ptr::addr_of_mut!(rcWorkArea)) == 0 {
            return;
        }
    
        if rcWorkArea.left + 2 < (*this_).__base3.m_nLeft {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_LEFTFIT;
        } else {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_LEFTFIT;
        }
    
        if rcWorkArea.top + 2 < (*this_).__base3.m_nTop {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_TOPFIT;
        } else {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_TOPFIT;
        }
    
        if (*this_).__base3.m_nLeft + (*this_).__base3.m_nWidth < rcWorkArea.right - 2 {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_RIGHTFIT;
        } else {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_RIGHTFIT;
        }
    
        if (*this_).__base3.m_nTop + (*this_).__base3.m_nHeight < rcWorkArea.bottom - 2 {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_BOTTOMFIT;
        } else {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_BOTTOMFIT;
        }
    }
    pub unsafe fn CancelMenu(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_pThread.is_null() {
            return;
        }
    
        let mut pTipbarWnd: *mut CTipbarWnd = (*(*this_).m_pThread).m_pTipbarWnd;
        if !pTipbarWnd.is_null() {
            if !(*pTipbarWnd).m_pLangBarMgr.is_null() {
                (*pTipbarWnd).StartModalInput(core::ptr::null_mut(), (*(*this_).m_pThread).m_dwThreadId);
            }
        }
    
        (*(*this_).m_pModalMenu).CancelMenu();
        (*this_).StartBackToAlphaTimer();
    }
    pub unsafe fn CheckExcludeCaptionButtonMode(&mut self, prc1: LPRECT, prc2: LPCRECT) -> BOOL {
        let this_: *mut Self = self;
        return if ((*prc1).top < (*prc2).top + 5) && ((*prc2).right <= (*prc1).right + (5 * (*this_).m_ButtonWidth)) { 1 } else { 0 };
    }
    pub unsafe fn ClearLBItemList(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_LangBarItemList.Clear();
        if !(*this_).m_pFocusThread.is_null() {
            let _ = (*this_).OnThreadItemChange((*(*this_).m_pFocusThread).m_dwThreadId);
        }
    }
    pub unsafe fn CreateVerticalFont(&mut self) -> HFONT {
        let this_: *mut Self = self;
        if (*this_).__base3.m_hWnd.is_null() {
            return core::ptr::null_mut();
        }
    
        let mut theme: CUIFTheme = core::mem::zeroed();
        theme.m_iPartId = 1;
        theme.m_iStateId = 0;
        theme.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
    
        let mut lf: LOGFONTW = core::mem::zeroed();
        if (theme.InternalOpenThemeData((*this_).__base3.m_hWnd) < 0)
            || ((GetThemeFont(
                theme.m_hTheme,
                core::ptr::null_mut(),
                theme.m_iPartId,
                0,
                210,
                core::ptr::addr_of_mut!(lf),
            )) < 0)
        {
            GetObject(
                GetStockObject((DEFAULT_GUI_FONT as i32)),
                core::mem::size_of::<LOGFONTW>() as i32,
                core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
            );
        }
    
        lf.lfEscapement = 2700;
        lf.lfOrientation = 2700;
        lf.lfOutPrecision = (OUT_TT_ONLY_PRECIS as u8);
    
        if CheckEAFonts() != 0 {
            let mut szText: [WCHAR; LF_FACESIZE as usize] =
                [0u16; LF_FACESIZE as usize];
            szText[0 as usize] = 0x0040u16;
            StringCchCopyW(
                szText.as_mut_ptr().add(1 as usize),
                (_countof!(szText) - 1) as size_t,
                lf.lfFaceName.as_ptr(),
            );
            StringCchCopyW(
                lf.lfFaceName.as_mut_ptr(),
                (_countof!(lf.lfFaceName)) as _,
                szText.as_ptr(),
            );
        }
    
        return CreateFontIndirectW(core::ptr::addr_of!(lf));
    }
    pub unsafe fn UpdateVerticalFont(&mut self) {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0 {
            if !(*this_).m_hTextFont.is_null() {
                DeleteObject(((*this_).m_hTextFont).cast());
                (*this_).__base3.SetFontToThis(core::ptr::null_mut());
                (*this_).m_hTextFont = core::ptr::null_mut();
            }
            (*this_).m_hTextFont = (*this_).CreateVerticalFont();
            (*this_).__base3.SetFontToThis((*this_).m_hTextFont);
        } else {
            (*this_).__base3.SetFontToThis(core::ptr::null_mut());
        }
    }
    pub unsafe fn ShowOverScreenSizeBalloon(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn DestroyOverScreenSizeBalloon(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pBalloon.is_null() {
            if IsWindow((*(*this_).m_pBalloon).m_hWnd) != 0 {
                DestroyWindow((*(*this_).m_pBalloon).m_hWnd);
            }
            cpp_delete((*this_).m_pBalloon);
            (*this_).m_pBalloon = core::ptr::null_mut();
        }
    }
    pub unsafe fn DestroyWnd(&mut self) {
        let this_: *mut Self = self;
        if IsWindow((*this_).__base3.m_hWnd) != 0 {
            DestroyWindow((*this_).__base3.m_hWnd);
        }
    }
    pub unsafe fn GetFocusKeyboardLayout(&mut self) -> HKL {
        let this_: *mut Self = self;
        let mut dwThreadId: DWORD = 0;
        if !(*this_).m_pFocusThread.is_null() {
            dwThreadId = (*(*this_).m_pFocusThread).m_dwThreadId;
        }
        return GetKeyboardLayout(dwThreadId);
    }
    pub unsafe fn KillOnTheadItemChangeTimer(&mut self) {
        let this_: *mut Self = self;
        let dwChangingThreadId: DWORD = (*this_).m_dwChangingThreadId;
        (*this_).m_dwChangingThreadId = 0;
        (*this_).KillTimer(4);
    
        if dwChangingThreadId != 0 {
            let pThread: *mut CTipbarThread = (*this_)._FindThread(dwChangingThreadId);
            if !pThread.is_null() {
                (*pThread).m_dwUnknown34 |= 0x1;
            }
        }
    }
    pub unsafe fn SetTimer(&mut self, nIDEvent: UINT_PTR, uElapse: UINT) -> UINT_PTR {
        let this_: *mut Self = self;
        if IsWindow((*this_).__base3.m_hWnd) != 0 {
            return SetTimer((*this_).__base3.m_hWnd, nIDEvent, uElapse, None);
        }
        return 0;
    }
    pub unsafe fn KillTimer(&mut self, uIDEvent: UINT_PTR) -> BOOL {
        let this_: *mut Self = self;
        if IsWindow((*this_).__base3.m_hWnd) != 0 {
            return KillTimer((*this_).__base3.m_hWnd, uIDEvent);
        }
        return (FALSE) as BOOL;
    }
    pub unsafe fn MoveToTray(&mut self) {
        let this_: *mut Self = self;
        if g_bEnableDeskBand == 0 || (g_dwOSInfo & (CIC_OSINFO_XPPLUS as u32)) == 0 {
            //FIXME
        }
    }
    pub unsafe fn MyClientToScreen(&mut self, lpPoint: LPPOINT, prc: LPRECT) {
        let this_: *mut Self = self;
        if !lpPoint.is_null() {
            ClientToScreen((*this_).__base3.m_hWnd, lpPoint);
        }
    
        if !prc.is_null() {
            ClientToScreen((*this_).__base3.m_hWnd, prc as LPPOINT);
            ClientToScreen((*this_).__base3.m_hWnd, core::ptr::addr_of_mut!((*prc).right) as LPPOINT);
        }
    }
    pub unsafe fn SavePosition(&mut self) {
        let this_: *mut Self = self;
        let mut regKey = CicRegKey__zeroed();
        static SUBKEY: [u16; 30] = ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'M' as u16, 'S' as u16, 'U' as u16, 'T' as u16, 'B' as u16, '\\' as u16, 0];
        static LEFT_NAME: [u16; 5] = ['L' as u16, 'e' as u16, 'f' as u16, 't' as u16, 0];
        static TOP_NAME: [u16; 4] = ['T' as u16, 'o' as u16, 'p' as u16, 0];
        static VERTICAL_NAME: [u16; 9] = ['V' as u16, 'e' as u16, 'r' as u16, 't' as u16, 'i' as u16, 'c' as u16, 'a' as u16, 'l' as u16, 0];
        if regKey.Create(HKEY_CURRENT_USER, (SUBKEY.as_ptr())) != 0 {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            ClientToScreen((*this_).__base3.m_hWnd, core::ptr::addr_of_mut!(pt));
            regKey.SetDword((LEFT_NAME.as_ptr()), pt.x as DWORD);
            regKey.SetDword((TOP_NAME.as_ptr()), pt.y as DWORD);
            regKey.SetDword((VERTICAL_NAME.as_ptr()), (((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0) as DWORD);
        }
    }
    pub unsafe fn SetAlpha(&mut self, bAlpha: BYTE,
        bFlag: BOOL,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn SetLangBand(&mut self, bDeskBand: BOOL, bFlag2: BOOL) -> BOOL {
        let this_: *mut Self = self;
        if bDeskBand == ((((*this_).m_dwShowType & TF_SFT_DESKBAND) != 0) as BOOL) {
            return (TRUE) as BOOL;
        }
    
        let mut ret: BOOL = (TRUE) as i32;
        let hwndTray: HWND = (*this_).m_ShellWndThread.GetWndTray();
        if bFlag2 != 0 && !hwndTray.is_null() {
            let mut dwResult: DWORD_PTR = 0;
            let hImeWnd: HWND = ImmGetDefaultIMEWnd(hwndTray);
            if !hImeWnd.is_null() {
                SendMessageTimeout(
                    hImeWnd,
                    WM_IME_SYSTEM,
                    (0x24 - bDeskBand) as WPARAM,
                    hwndTray as LPARAM,
                    (SMTO_BLOCK | SMTO_ABORTIFHUNG),
                    5000,
                    core::ptr::addr_of_mut!(dwResult),
                );
            } else {
                SendMessageTimeout(
                    hwndTray,
                    0x505,
                    0,
                    bDeskBand as LPARAM,
                    (SMTO_BLOCK | SMTO_ABORTIFHUNG),
                    5000,
                    core::ptr::addr_of_mut!(dwResult),
                );
            }
        } else {
            ret = (FALSE) as i32;
        }
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) == 0 && bDeskBand != 0 {
            (*this_).KillTimer(7);
            (*this_).SetTimer(7, g_uTimerElapseSYSCOLORCHANGED);
        }
    
        return ret;
    }
    pub unsafe fn SetMoveRect(&mut self, X: INT, Y: INT, nWidth: INT, nHeight: INT) {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) != 0 {
            (*this_).__base3.m_nWidth = nWidth;
            (*this_).__base3.m_nHeight = nHeight;
            return;
        }
    
        (*this_).m_bInCallOn += 1;
    
        (*this_).m_dwTipbarWndFlags |= TIPBAR_UPDATING;
    
        (*this_).m_X = X;
        (*this_).m_Y = Y;
        (*this_).m_CX = nWidth;
        (*this_).m_CY = nHeight;
    
        let mut rc: RECT = core::mem::zeroed();
        let mut size: SIZE = SIZE { cx: 0, cy: 0 };
        if !(*this_).m_pWndFrame.is_null() {
            SetRect(
                core::ptr::addr_of_mut!(rc),
                0,
                0,
                nWidth - (*this_).m_cxDlgFrameX2,
                nHeight - (*this_).m_cyDlgFrameX2,
            );
            (*(*this_).m_pWndFrame).SetRect(core::ptr::addr_of_mut!(rc));
            (*(*this_).m_pWndFrame).GetFrameSize(core::ptr::addr_of_mut!(size));
        }
    
        if !(*this_).m_pTipbarGripper.is_null() {
            if ((*this_).m_dwTipbarWndFlags & TIPBAR_VERTICAL) != 0 {
                let GripperWidth: INT = (*this_).GetGripperWidth();
                SetRect(
                    core::ptr::addr_of_mut!(rc),
                    size.cx,
                    size.cy,
                    nWidth - (*this_).m_cxDlgFrameX2 - size.cx,
                    size.cy + GripperWidth,
                );
            } else {
                let GripperWidth: INT = (*this_).GetGripperWidth();
                let y1: INT = nHeight - (*this_).m_cyDlgFrameX2 - size.cy;
                SetRect(
                    core::ptr::addr_of_mut!(rc),
                    size.cx,
                    size.cy,
                    size.cx + GripperWidth,
                    y1,
                );
            }
            (*(*this_).m_pTipbarGripper).SetRect(core::ptr::addr_of_mut!(rc));
        }
    
        (*this_).m_bInCallOn -= 1;
    }
    pub unsafe fn SetShowText(&mut self, bShow: BOOL) {
        let this_: *mut Self = self;
        if bShow != 0 {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_HIGHCONTRAST;
        } else {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_HIGHCONTRAST;
        }
    
        if !(*this_).m_pFocusThread.is_null() {
            (*this_).OnThreadItemChange((*(*this_).m_pFocusThread).m_dwThreadId);
        }
    
        (*this_).TerminateAllThreads((FALSE) as BOOL);
    }
    pub unsafe fn SetShowTrayIcon(&mut self, bShow: BOOL) {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_TRAYICON) != 0 {
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_TRAYICON;
        } else {
            (*this_).m_dwTipbarWndFlags |= TIPBAR_TRAYICON;
        }
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_TRAYICON) != 0 && !(*this_).m_pFocusThread.is_null() {
            (*this_).KillTimer(10);
            (*this_).SetTimer(10, g_uTimerElapseMOVETOTRAY);
        } else if !g_pTrayIconWnd.is_null() {
            (*g_pTrayIconWnd).SetMainIcon(core::ptr::null_mut());
            (*g_pTrayIconWnd).RemoveAllIcon(0);
        }
    }
    pub unsafe fn ShowContextMenu(&mut self, pt: POINT, mut prc: LPCRECT, bFlag: BOOL) {
        let this_: *mut Self = self;
        (*this_).AddRef();
    
        let rc: RECT = RECT { left: pt.x, top: pt.y, right: pt.x, bottom: pt.y };
        if prc.is_null() {
            prc = core::ptr::addr_of!(rc);
        }
    
        if !(*this_).m_pFocusThread.is_null() {
            let pContextMenu: *mut CUTBContextMenu = CUTBContextMenu__new_in_0(cpp_alloc_CicNoThrow::<CUTBContextMenu>(), this_);
            if !pContextMenu.is_null() {
                if (*pContextMenu).Init() != 0 {
                    (*this_).m_pThread = (*this_).m_pFocusThread;
                    (*this_).StartModalInput(
                        core::ptr::addr_of_mut!((*this_).__base0),
                        (*(*this_).m_pFocusThread).m_dwThreadId,
                    );
    
                    (*this_).m_pModalMenu = core::ptr::addr_of_mut!((*pContextMenu).__base0);
                    let dwCommandId: DWORD = (*pContextMenu).ShowPopup((*this_).GetWindow(), pt, prc, bFlag);
                    (*this_).m_pModalMenu = core::ptr::null_mut();
    
                    if !(*this_).m_pThread.is_null() {
                        (*this_).StopModalInput((*(*this_).m_pThread).m_dwThreadId);
                    }
    
                    (*this_).m_pThread = core::ptr::null_mut();
    
                    if dwCommandId != (0 as DWORD).wrapping_sub(1 as DWORD) {
                        (*pContextMenu).SelectMenuItem(dwCommandId);
                    }
                }
    
                CUTBContextMenu__delete(pContextMenu);
            }
        }
    
        (*this_).Release();
    }
    pub unsafe fn StartBackToAlphaTimer(&mut self) {
        let this_: *mut Self = self;
        let uTime: UINT = GetDoubleClickTime();
        SetTimer((*this_).__base3.m_hWnd, 3, 3 * uTime, None);
    }
    pub unsafe fn StartDoAccDefaultActionTimer(&mut self, pTarget: *mut CTipbarItem) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pTipbarAccessible.is_null() {
            return (FALSE) as BOOL;
        }
        let IDOfItem: INT = (*(*this_).m_pTipbarAccessible).GetIDOfItem(core::ptr::addr_of_mut!((*pTarget).__base0));
        (*this_).m_nID = IDOfItem;
        if IDOfItem == 0 || IDOfItem == -1 {
            return (FALSE) as BOOL;
        }
        (*this_).KillTimer(11);
        (*this_).SetTimer(11, g_uTimerElapseDOACCDEFAULTACTION);
        return (TRUE) as BOOL;
    }
    pub unsafe fn StartModalInput(&mut self, pSink: *mut ITfLangBarEventSink, dwThreadId: DWORD) {
        let this_: *mut Self = self;
        if (*this_).m_pLangBarMgr.is_null() {
            return;
        }
    
        (*(*this_).m_pLangBarMgr).SetModalInput(pSink, dwThreadId, 0);
        if !g_pTrayIconWnd.is_null() {
            (*(*this_).m_pLangBarMgr).SetModalInput(pSink, (*g_pTrayIconWnd).m_dwTrayWndThreadId, 0);
        }
    
        let dwCurThreadId: DWORD = GetCurrentThreadId();
        (*(*this_).m_pLangBarMgr).SetModalInput(pSink, dwCurThreadId, 1);
    }
    pub unsafe fn StopModalInput(&mut self, dwThreadId: DWORD) {
        let this_: *mut Self = self;
        if (*this_).m_pLangBarMgr.is_null() {
            return;
        }
    
        (*(*this_).m_pLangBarMgr).SetModalInput(core::ptr::null_mut(), dwThreadId, 0);
        if !g_pTrayIconWnd.is_null() {
            (*(*this_).m_pLangBarMgr).SetModalInput(core::ptr::null_mut(), (*g_pTrayIconWnd).m_dwTrayWndThreadId, 0);
        }
    
        let dwCurThreadId: DWORD = GetCurrentThreadId();
        (*(*this_).m_pLangBarMgr).SetModalInput(core::ptr::null_mut(), dwCurThreadId, 0);
    }
    pub unsafe fn _CreateThread(&mut self, dwThreadId: DWORD) -> *mut CTipbarThread {
        let this_: *mut Self = self;
        let mut pTarget: *mut CTipbarThread = (*this_)._FindThread(dwThreadId);
        if !pTarget.is_null() {
            return pTarget;
        }
    
        MyWaitForInputIdle(dwThreadId, 2000);
    
        pTarget = CTipbarThread__new_in_0(cpp_alloc_CicNoThrow::<CTipbarThread>(), this_);
        if pTarget.is_null() {
            return core::ptr::null_mut();
        }
    
        (*this_).AddThreadToThreadCreatingList(pTarget);
    
        let hr: HRESULT = (*pTarget).Init(dwThreadId);
    
        (*this_).RemoveThredFromThreadCreatingList(pTarget);
    
        if (hr >= 0) && (*this_).m_Threads.Add(pTarget) == 0 {
            (*pTarget)._UninitItemList((TRUE) as BOOL);
            (*pTarget).m_pTipbarWnd = core::ptr::null_mut();
            (*pTarget)._Release();
            return core::ptr::null_mut();
        }
    
        pTarget
    }
    pub unsafe fn _FindThread(&mut self, dwThreadId: DWORD) -> *mut CTipbarThread {
        let this_: *mut Self = self;
        if g_bWinLogon != 0 {
            return core::ptr::null_mut();
        }
    
        let mut pTarget: *mut CTipbarThread = core::ptr::null_mut();
        let mut iItem: usize = 0;
        while iItem < (*this_).m_Threads.size() {
            let pThread: *mut CTipbarThread = (*this_).m_Threads[iItem];
            if !pThread.is_null() && (*pThread).m_dwThreadId == dwThreadId {
                pTarget = pThread;
                break;
            }
            iItem += 1;
        }
    
        if pTarget.is_null() {
            return core::ptr::null_mut();
        }
    
        let mut dwFlags1: DWORD = 0;
        let mut dwFlags2: DWORD = 0;
        let mut dwFlags3: DWORD = 0;
        TF_GetThreadFlags(
            dwThreadId,
            core::ptr::addr_of_mut!(dwFlags1),
            core::ptr::addr_of_mut!(dwFlags2),
            core::ptr::addr_of_mut!(dwFlags3),
        );
    
        if dwFlags2 == 0 || (dwFlags2 != (*pTarget).m_dwFlags2) || (dwFlags3 != (*pTarget).m_dwFlags3) {
            (*this_).OnThreadTerminateInternal(dwThreadId);
            return core::ptr::null_mut();
        }
    
        pTarget
    }
    pub unsafe fn EnsureFocusThread(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pFocusThread.is_null() || ((*this_).m_dwTipbarWndFlags & (TIPBAR_TOOLBARENDED | TIPBAR_ENSURING)) != 0 {
            return;
        }
    
        (*this_).m_dwTipbarWndFlags |= TIPBAR_ENSURING;
    
        let hwndFore: HWND = GetForegroundWindow();
        if hwndFore as usize == 0 {
            return;
        }
    
        let dwThreadId: DWORD = GetWindowThreadProcessId(hwndFore, core::ptr::null_mut());
        if dwThreadId != 0 {
            (*this_).OnSetFocus(dwThreadId);
        }
    
        (*this_).m_dwTipbarWndFlags &= !TIPBAR_ENSURING;
    }
    pub unsafe fn SetFocusThread(&mut self, pFocusThread: *mut CTipbarThread) -> HRESULT {
        let this_: *mut Self = self;
        if pFocusThread == (*this_).m_pFocusThread {
            return S_OK;
        }
    
        let dwThreadId: DWORD = GetCurrentThreadId();
        (*this_).DestroyOverScreenSizeBalloon();
    
        if !(*this_).m_pFocusThread.is_null() {
            (*(*this_).m_pFocusThread).SetFocus(core::ptr::null_mut());
            AttachThreadInput(dwThreadId, (*(*this_).m_pFocusThread).m_dwThreadId, (FALSE) as BOOL);
        }
    
        (*this_).m_dwTipbarWndFlags &= !TIPBAR_ATTACHED;
        (*this_).m_pFocusThread = pFocusThread;
        return S_OK;
    }
    pub unsafe fn AttachFocusThread(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_ATTACHED) != 0 {
            return S_FALSE;
        }
    
        if !(*this_).m_pFocusThread.is_null() {
            let dwThreadId: DWORD = GetCurrentThreadId();
            AttachThreadInput(dwThreadId, (*(*this_).m_pFocusThread).m_dwThreadId, (TRUE) as BOOL);
            (*this_).m_dwTipbarWndFlags |= TIPBAR_ATTACHED;
        }
    
        return S_OK;
    }
    pub unsafe fn RestoreLastFocus(&mut self, pdwThreadId: *mut DWORD, fPrev: BOOL) {
        let this_: *mut Self = self;
        if !(*this_).m_pLangBarMgr.is_null() {
            (*(*this_).m_pLangBarMgr).RestoreLastFocus(pdwThreadId, fPrev);
        }
    }
    pub unsafe fn CleanUpThreadPointer(&mut self, pThread: *mut CTipbarThread, bRemove: BOOL) {
        let this_: *mut Self = self;
        if bRemove != 0 {
            let iItem: isize = (*this_).m_Threads.Find(pThread);
            if iItem >= 0 {
                (*this_).m_Threads.Remove((iItem) as usize, 1);
            }
        }
    
        if pThread == (*this_).m_pFocusThread {
            (*this_).SetFocusThread(core::ptr::null_mut());
        }
    
        if pThread == (*this_).m_pThread {
            (*this_).m_pThread = core::ptr::null_mut();
        }
    
        if pThread == (*this_).m_pUnknownThread {
            (*this_).m_pUnknownThread = core::ptr::null_mut();
        }
    }
    pub unsafe fn TerminateAllThreads(&mut self, bFlag: BOOL) {
        let this_: *mut Self = self;
        let cItems: usize = (*this_).m_Threads.size() as usize;
    
        let pdwThreadIds: *mut DWORD = cpp_new_array_CicNoThrow::<DWORD>(cItems);
        if pdwThreadIds.is_null() {
            return;
        }
    
        let mut iItem: usize = 0;
        while iItem < cItems {
            *pdwThreadIds.add(iItem) = 0;
            let pThread: *mut CTipbarThread = (*this_).m_Threads[iItem];
            if !pThread.is_null() && ((bFlag != 0) || (pThread != (*this_).m_pFocusThread)) {
                *pdwThreadIds.add(iItem) = (*pThread).m_dwThreadId;
            }
            iItem += 1;
        }
    
        let mut iItem: usize = 0;
        while iItem < cItems {
            if *pdwThreadIds.add(iItem) != 0 {
                (*this_).OnThreadTerminateInternal(*pdwThreadIds.add(iItem));
            }
            iItem += 1;
        }
    
        cpp_delete_array(pdwThreadIds);
    }
    pub unsafe fn OnTerminateToolbar(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_dwTipbarWndFlags |= TIPBAR_TOOLBARENDED;
        (*this_).DestroyOverScreenSizeBalloon();
        (*this_).TerminateAllThreads((TRUE) as BOOL);
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) == 0 {
            (*this_).SavePosition();
        }
    }
    pub unsafe fn OnThreadTerminateInternal(&mut self, dwThreadId: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut iItem: usize = 0 as usize;
        while iItem < (*this_).m_Threads.size() {
            let pThread: *mut CTipbarThread = (*this_).m_Threads[iItem];
            if !pThread.is_null() && (*pThread).m_dwThreadId == dwThreadId {
                (*this_).m_Threads.Remove(iItem as size_t, 1);
                (*pThread).RemoveUIObjs();
                (*this_).CleanUpThreadPointer(pThread, FALSE as BOOL);
                (*pThread)._UninitItemList(TRUE as BOOL);
                (*pThread).m_pTipbarWnd = core::ptr::null_mut();
                (*pThread)._Release();
                break;
            }
            iItem += 1;
        }
    
        return S_OK as HRESULT;
    }
    pub unsafe fn OnThreadItemChangeInternal(&mut self, dwThreadId: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let c_tab: [QITAB; 3] = [
            QITAB {
                piid: &IID_ITfLangBarEventSink as *const IID,
                dwOffset: core::mem::offset_of!(CTipbarWnd, __base0) as DWORD,
            },
            QITAB {
                piid: &IID_ITfLangBarEventSink_P as *const IID,
                dwOffset: core::mem::offset_of!(CTipbarWnd, __base1) as DWORD,
            },
            QITAB {
                piid: core::ptr::null(),
                dwOffset: 0,
            },
        ];
        return QISearch((this_).cast(), c_tab.as_ptr(), riid, ppvObj);
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs += 1;
        (*this_).m_cRefs as ULONG
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            CTipbarWnd__delete(this_);
            return 0;
        }
        return (*this_).m_cRefs as ULONG;
    }
    pub unsafe fn OnSetFocus(&mut self, dwThreadId: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn OnThreadTerminate(&mut self, dwThreadId: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT;
        (*this_).m_bInCallOn += 1;
        (*this_).AddRef();
        {
            hr = (*this_).OnThreadTerminateInternal(dwThreadId);
            if (*this_).m_pFocusThread.is_null() {
                (*this_).EnsureFocusThread();
            }
        }
        (*this_).m_bInCallOn -= 1;
        (*this_).Release();
        return hr;
    }
    pub unsafe fn OnThreadItemChange(&mut self, dwThreadId: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_dwTipbarWndFlags & TIPBAR_TOOLBARENDED != 0 {
            return S_OK;
        }
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD == 0) && ((*this_).m_dwShowType & TF_SFT_DESKBAND) != 0 {
            return S_OK;
        }
    
        let pThread: *mut CTipbarThread = (*this_)._FindThread(dwThreadId);
        if !pThread.is_null() {
            if ((*this_).m_dwUnknown23 == 0 || (*this_).m_dwUnknown23 == dwThreadId) && pThread == (*this_).m_pFocusThread {
                (*this_).KillOnTheadItemChangeTimer();
                (*this_).m_dwChangingThreadId = dwThreadId;
                (*this_).KillTimer(6);
                (*this_).SetTimer(4, g_uTimerElapseONTHREADITEMCHANGE);
            } else {
                (*pThread).m_dwUnknown34 |= 0x1;
            }
        } else {
            let mut iItem: usize = 0;
            while iItem < (*this_).m_ThreadCreatingList.size() {
                let pItem: *mut CTipbarThread = (*this_).m_ThreadCreatingList[iItem];
                if !pItem.is_null() && (*pItem).m_dwThreadId == dwThreadId {
                    (*pItem).m_dwUnknown34 |= 0x1;
                }
                iItem += 1;
            }
        }
    
        S_OK
    }
    pub unsafe fn OnModalInput(&mut self, dwThreadId: DWORD, mut uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        match uMsg {
            WM_NCLBUTTONDOWN |
            WM_NCRBUTTONDOWN |
            WM_NCMBUTTONDOWN |
            WM_LBUTTONUP |
            WM_RBUTTONUP |
            WM_MBUTTONUP => {
            }
    
            WM_NCLBUTTONUP |
            WM_NCRBUTTONUP |
            WM_NCMBUTTONUP => {
                if !(*this_).m_pThread.is_null() {
                    let pMenuUI: *mut CUTBMenuWnd = (*(*this_).m_pModalMenu).m_pMenuUI;
                    if !pMenuUI.is_null() {
                        let hWnd: HWND = (*pMenuUI).__base1.m_hWnd;
                        if !hWnd.is_null() {
                            let mut pt: POINT = POINT {
                                x: (LOWORD!(lParam) as SHORT) as LONG,
                                y: (HIWORD!(lParam) as SHORT) as LONG,
                            };
                            ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt));
                            uMsg += WM_LBUTTONUP - WM_NCLBUTTONUP;
                            PostMessage((*this_).__base3.m_hWnd, uMsg, wParam, MAKELPARAM!(pt.x as WORD, pt.y as WORD));
                        }
                    }
                }
            }
    
            _ => {
                if uMsg == WM_KEYDOWN || uMsg == WM_KEYUP {
                    if !(*this_).m_pThread.is_null() {
                        (*(*this_).m_pModalMenu).PostKey((uMsg == WM_KEYUP) as BOOL, wParam, lParam);
                    }
                } else {
                    (*this_).CancelMenu();
                }
            }
        }
    
        return 0;
    }
    pub unsafe fn ShowFloating(&mut self, dwFlags: DWORD,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetItemFloatingRect(&mut self, dwThreadId: DWORD, rguid: &GUID, prc: *mut RECT) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_dwTipbarWndFlags & TIPBAR_TRAYICON != 0 {
            return E_UNEXPECTED;
        }
    
        if (*this_).m_pFocusThread.is_null() || ((*(*this_).m_pFocusThread).m_dwThreadId != dwThreadId) {
            return E_FAIL;
        }
    
        let mut iItem: usize = 0;
        while iItem < (*(*this_).m_pFocusThread).m_UIObjects.size() {
            let pItem: *mut CTipbarItem = (*(*this_).m_pFocusThread).m_UIObjects[iItem];
            if !pItem.is_null() {
                if ((*pItem).m_dwItemFlags & 0x8) != 0
                    && IsEqualGUID!(&(*pItem).m_ItemInfo.guidItem, rguid)
                {
                    (*pItem).OnUnknown57(prc);
                    return S_OK;
                }
            }
            iItem += 1;
        }
    
        return E_FAIL;
    }
    pub unsafe fn OnLangBarUpdate__body(&mut self, click: TfLBIClick,
        bFlag: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn GetAccName__body(&mut self) -> BSTR {
        let this_: *mut Self = self;
        let mut szText: [u16; 256] = core::mem::zeroed();
        LoadStringW(g_hInst, IDS_LANGUAGEBAR, szText.as_mut_ptr(), (_countof!(szText)) as i32);
        return SysAllocString(szText.as_ptr());
    }
    pub unsafe fn GetAccLocation__body(&mut self, lprc: LPRECT) {
        let this_: *mut Self = self;
        (*this_).__base3.GetRect(lprc);
    }
    pub unsafe fn PaintObject__body(&mut self, hDC: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_UPDATING) != 0 {
            (*this_).Move((*this_).m_X, (*this_).m_Y, (*this_).m_CX, (*this_).m_CY);
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_UPDATING;
        }
    
        if (*this_).m_pFocusThread.is_null() || (((*(*this_).m_pFocusThread).IsDirtyItem()) == 0) {
            (*(*this_).m_pFocusThread).CallOnUpdateHandler();
            if !g_pTipbarWnd.is_null() {
                (*this_).__base3.PaintObject(hDC, prc);
            }
        }
    }
    pub unsafe fn GetWndStyle__body(&mut self) -> DWORD {
        let this_: *mut Self = self;
        return (*this_).__base3.GetWndStyle() & !WS_BORDER;
    }
    pub unsafe fn Move__body(&mut self, x: INT, y: INT, nWidth: INT, nHeight: INT) {
        let this_: *mut Self = self;
        (*this_).__base3.Move(x, y, nWidth, nHeight);
    }
    pub unsafe fn OnMouseOutFromWindow__body(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        (*this_).StartBackToAlphaTimer();
        if ((*this_).m_dwTipbarWndFlags & 0x40) != 0 && ((*this_).m_dwTipbarWndFlags & 0x80) != 0 {
            (*this_).SetTimer(2, g_uTimerElapseSTUBEND);
        }
    }
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnDestroy__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        let _ = hWnd;
        (*this_).CancelMenu();
    
        if !(*this_).m_pTipbarAccessible.is_null() {
            (*(*this_).m_pTipbarAccessible).NotifyWinEvent(EVENT_OBJECT_DESTROY, (*this_).GetAccItem());
        }
    
        (*this_).OnTerminateToolbar();
        if !(*this_).m_pTipbarAccessible.is_null() {
            (*(*this_).m_pTipbarAccessible).ClearAccItems();
            (*(*this_).m_pTipbarAccessible).Release();
            (*this_).m_pTipbarAccessible = core::ptr::null_mut();
        }
    
        (*this_).m_coInit.CoUninit();
    
        if !(*this_).m_pLangBarMgr.is_null() {
            (*(*this_).m_pLangBarMgr).UnAdviseEventSink((*this_).m_dwSinkCookie);
        }
    }
    pub unsafe fn OnTimer__body(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;
        (*this_).AddRef();
        match wParam {
            1 => {
                (*this_).KillTimer(1);
                (*this_).MoveToStub((FALSE) as BOOL);
            }
            2 => {
                (*this_).KillTimer(2);
                (*this_).MoveToStub((TRUE) as BOOL);
            }
            3 => {
                (*this_).KillTimer(3);
                (*this_).SetAlpha((*this_).m_dwAlphaValue as BYTE, (TRUE) as BOOL);
            }
            4 => {
                let status: LONG = MyWaitForInputIdle((*this_).m_dwChangingThreadId, 2000);
                if status != 0 {
                    if status != STATUS_TIMEOUT {
                        (*this_).KillTimer(4);
                        (*this_).m_dwChangingThreadId = 0;
                    }
                } else if (*this_).m_pThread.is_null() {
                    (*this_).KillTimer(4);
                    let dwOldThreadId: DWORD = (*this_).m_dwChangingThreadId;
                    (*this_).m_dwChangingThreadId = 0;
                    (*this_).OnThreadItemChangeInternal(dwOldThreadId);
                }
            }
            5 => {
                (*this_).KillTimer(5);
                SetWindowPos(
                    (*this_).__base3.m_hWnd,
                    core::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                );
            }
            6 => {
                (*this_).KillTimer(6);
                if !(*this_).m_pFocusThread.is_null() {
                    if (*(*this_).m_pFocusThread).m_dwThreadId != (*this_).m_dwChangingThreadId
                        && (*(*this_).m_pFocusThread).CallOnUpdateHandler() == 0
                    {
                        if !(*this_).m_pFocusThread.is_null() {
                            (*this_).OnThreadItemChange((*(*this_).m_pFocusThread).m_dwThreadId);
                        }
                    }
                }
            }
            7 => {
                let mut dwThreadId: DWORD = 0;
                if (*this_).KillTimer(7) != 0 {
                    if !(*this_).m_pFocusThread.is_null() {
                        dwThreadId = (*(*this_).m_pFocusThread).m_dwThreadId;
                    }
    
                    (*this_).TerminateAllThreads((TRUE) as BOOL);
                    (*this_).UpdateVerticalFont();
    
                    if dwThreadId != 0 {
                        (*this_).OnSetFocus(dwThreadId);
                    }
    
                    (*this_).InitMetrics();
                    // FIXME: CTipbarCtrlButtonHolder
    
                    (*this_).InitHighContrast();
                    (*this_).SetAlpha(0xFF, (TRUE) as BOOL);
                    RedrawWindow(
                        (*this_).__base3.m_hWnd,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        (RDW_FRAME | RDW_UPDATENOW | RDW_INVALIDATE),
                    );
                }
            }
            8 => {
                (*this_).KillTimer(8);
                (*this_).UpdateUI(core::ptr::null());
            }
            9 => {
                (*this_).KillTimer(9);
                //FIXME
                if (*this_).m_pUnknownThread == (*this_).m_pFocusThread {
                    (*this_).__base3.Show((((*this_).m_dwUnknown23_5[3 as usize] & 0x80000000) != 0) as BOOL);
                }
                (*this_).m_pUnknownThread = core::ptr::null_mut();
                if ((*this_).m_dwUnknown23_5[3 as usize] & 0x2) != 0 {
                    (*this_).ShowOverScreenSizeBalloon();
                }
            }
            10 => {
                (*this_).KillTimer(10);
                (*this_).MoveToTray();
            }
            11 => {
                (*this_).KillTimer(11);
                if !(*this_).m_pTipbarAccessible.is_null() {
                    if (*this_).m_nID != 0 {
                        (*(*this_).m_pTipbarAccessible).DoDefaultActionReal((*this_).m_nID);
                        (*this_).m_nID = 0;
                    }
                }
            }
            12 => {
                (*this_).KillTimer(12);
                (*this_).AdjustPosOnDisplayChange();
            }
            13 => {
            }
            14 => {
                if (*this_).SetLangBand((TRUE) as BOOL, (TRUE) as BOOL) != 0 {
                    (*this_).m_dwShowType = TF_SFT_DESKBAND;
                    (*this_).KillTimer(14);
                }
            }
            _ => {
                if 10000 <= wParam && wParam < 10050 {
                    let pItem = (*this_).m_LangBarItemList.GetItemStateFromTimerId(wParam);
                    if !pItem.is_null() {
                        let clsid: *const GUID = core::ptr::addr_of!((*pItem).m_clsid);
                        (*this_).m_LangBarItemList.SetDemoteLevel(&(*pItem).m_clsid, 2);
                        if !(*this_).m_pFocusThread.is_null() {
                            let pThreadItem = (*(*this_).m_pFocusThread).GetItem(&*clsid);
                            if !pThreadItem.is_null() {
                                (*pThreadItem).AddRemoveMeToUI((FALSE) as BOOL);
                            }
                        }
                    }
                }
            }
        }
    
        (*this_).Release();
    }
    pub unsafe fn OnSysColorChange__body(&mut self) {
        let this_: *mut Self = self;
        let _ = (*this_).KillTimer(7);
        let _ = (*this_).SetTimer(7, g_uTimerElapseSYSCOLORCHANGED);
    }
    pub unsafe fn OnEndSession__body(&mut self, hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if g_bWinLogon == 0 {
            (*this_).OnTerminateToolbar();
        }
    
        if wParam != 0 { // End session?
            if lParam & (ENDSESSION_LOGOFF as LPARAM) != 0 {
                (*this_).KillTimer(9);
                (*this_).__base3.Show((FALSE) as i32);
            } else {
                (*this_).OnTerminateToolbar();
    
                (*this_).AddRef();
                DestroyWindow(hWnd);
                (*this_).Release();
            }
        }
    }
    pub unsafe fn OnUser__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if uMsg == WM_USER + 1 {
            let mut pt: POINT = POINT {
                x: (LOWORD!(lParam as DWORD) as SHORT) as LONG,
                y: (HIWORD!(lParam as DWORD) as SHORT) as LONG,
            };
            ClientToScreen((*this_).__base3.m_hWnd, core::ptr::addr_of_mut!(pt));
            (*this_).ShowContextMenu(pt, core::ptr::null_mut(), (TRUE) as BOOL);
        } else if uMsg == g_wmTaskbarCreated {
            (*this_).m_ShellWndThread.clear();
        }
    }
    pub unsafe fn OnWindowPosChanged__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pFocusThread.is_null() {
            let mut iItem: usize = 0;
            while iItem < (*(*this_).m_pFocusThread).m_UIObjects.size() {
                let pItem: *mut CTipbarItem = (*(*this_).m_pFocusThread).m_UIObjects[iItem];
                if !pItem.is_null() {
                    (*pItem).OnUnknown44();
                }
                iItem += 1;
            }
        }
    
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn OnWindowPosChanging__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        let pWP: LPWINDOWPOS = lParam as LPWINDOWPOS;
        if ((*pWP).flags & SWP_NOZORDER) == 0 {
            if (*this_).m_pThread.is_null() && ((*this_).__base3.m_pToolTip.is_null() || (*(*this_).__base3.m_pToolTip).m_bShowToolTip == 0) {
                (*pWP).hwndInsertAfter = core::ptr::null_mut();
            }
        }
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn OnShowWindow__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(*this_).m_pTipbarAccessible.is_null() {
            if wParam != 0 {
                (*(*this_).m_pTipbarAccessible).NotifyWinEvent(EVENT_OBJECT_SHOW, (*this_).GetAccItem());
                (*(*this_).m_pTipbarAccessible).NotifyWinEvent(EVENT_OBJECT_FOCUS, (*this_).GetAccItem());
            } else {
                (*(*this_).m_pTipbarAccessible).NotifyWinEvent(EVENT_OBJECT_HIDE, (*this_).GetAccItem());
            }
        }
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn OnSettingChange__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == 0 || wParam == (SPI_SETNONCLIENTMETRICS as WPARAM) || wParam == (SPI_SETHIGHCONTRAST as WPARAM)
        {
            (*this_).KillTimer(7);
            (*this_).SetTimer(7, g_uTimerElapseSYSCOLORCHANGED);
        }
        return 0;
    }
    pub unsafe fn OnDisplayChange__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        if !(((*this_).m_dwTipbarWndFlags & TIPBAR_CHILD) != 0) {
            (*this_).KillTimer(12);
            (*this_).SetTimer(12, g_uTimerElapseDISPLAYCHANGE);
        }
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn OnGetObject__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        if lParam != -4 {
            return S_OK;
        }
        if (*this_).m_pTipbarAccessible.is_null() {
            return E_OUTOFMEMORY;
        }
    
        if (*(*this_).m_pTipbarAccessible).m_bInitialized != 0 {
            return ((*(*this_).m_pTipbarAccessible).CreateRefToAccObj(wParam)) as HRESULT;
        }
    
        let mut hr: HRESULT = S_OK;
        if (*this_).m_coInit.EnsureCoInit() >= 0 {
            hr = (*(*this_).m_pTipbarAccessible).Initialize();
            if hr < 0 {
                (*(*this_).m_pTipbarAccessible).Release();
                (*this_).m_pTipbarAccessible = core::ptr::null_mut();
                return hr;
            }
    
            (*(*this_).m_pTipbarAccessible).NotifyWinEvent(EVENT_OBJECT_CREATE, (*this_).GetAccItem());
            return ((*(*this_).m_pTipbarAccessible).CreateRefToAccObj(wParam)) as HRESULT;
        }
    
        hr
    }
    pub unsafe fn OnEraseBkGnd__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> BOOL {
        let this_: *mut Self = self;
        (TRUE) as i32
    }
    pub unsafe fn OnThemeChanged__body(&mut self, hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        (*this_).OnThemeChanged(hWnd, wParam, lParam);
    }
    pub unsafe fn UpdateUI__body(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        (*this_).KillTimer(8);
    
        if (*this_).m_dwChangingThreadId != 0
            || (*this_).m_bInCallOn != 0
            || ((*this_).m_pFocusThread != core::ptr::null_mut()
                && (*(*this_).m_pFocusThread).IsDirtyItem() != 0)
        {
            (*this_).SetTimer(8, g_uTimerElapseUPDATEUI);
            return;
        }
    
        if ((*this_).m_dwTipbarWndFlags & TIPBAR_UPDATING) != 0 {
            (*this_).m_bInCallOn += 1;
            (*this_).Move((*this_).m_X, (*this_).m_Y, (*this_).m_CX, (*this_).m_CY);
            (*this_).m_dwTipbarWndFlags &= !TIPBAR_UPDATING;
            (*this_).m_bInCallOn -= 1;
        }
    
        (*this_).__base3.UpdateUI(core::ptr::null());
    }
    pub unsafe fn HandleMouseMsg__body(&mut self, uMsg: UINT,
        x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
}
