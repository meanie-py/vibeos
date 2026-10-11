use crate::prelude::*;
use crate::msctfime::misc_cpp_h::GetCharsetFromLangId;
use crate::msctfime::misc_cpp_h::IsInteractiveUserLogon;
use crate::msctfime::ui_cpp_h::IsMsImeMessage;
use crate::msctfime::misc_cpp_h::IsVKDBEKey;
use crate::msctfime::misc_cpp_h::MsimtfIsGuidMapEnable;
use crate::msctfime::ui_cpp_h::RegisterImeClass;
use crate::msctfime::ui_cpp_h::RegisterMSIMEMessage;
use crate::msctfime::ui_cpp_h::UnregisterImeClass;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_56054C132291: [WCHAR; 12] = [0x004D, 0x0053, 0x0043, 0x0054, 0x0046, 0x0049, 0x004D, 0x0045, 0x0020, 0x0055, 0x0049, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut g_hInst: HINSTANCE = core::ptr::null_mut();
pub static mut g_bWinLogon: i32 = 0;
pub static mut g_uACP: u32 = 0; // CP_ACP
pub static mut g_dwOSInfo: DWORD = 0;
pub static mut gfTFInitLib: i32 = 0;
#[no_mangle]
pub static mut g_csLock: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut g_pPropCache: *mut CDispAttrPropCache = core::ptr::null_mut();

unsafe extern "C" fn InternalSelectEx(hIMC: HIMC, fSelect: BOOL, LangID: LANGID) -> HRESULT {
    let mut imcLock = CicIMCLock__make_0((hIMC).cast());
    if imcLock.m_hr < 0 {
        return imcLock.m_hr;
    }

    if (PRIMARYLANGID!(LangID)) as u32 == LANG_CHINESE {
        imcLock.get().cfCandForm[0 as usize].dwStyle = 0;
        imcLock.get().cfCandForm[0 as usize].dwIndex = (1 as DWORD).wrapping_neg();
    }

    if fSelect == 0 {
        imcLock.get().fdwInit &= !(INIT_GUIDMAP);
        return imcLock.m_hr;
    }

    if imcLock.ClearCand() == 0 {
        return imcLock.m_hr;
    }

    // Populate conversion mode
    if (imcLock.get().fdwInit & INIT_CONVERSION) == 0 {
        let mut dwConv: DWORD = imcLock.get().fdwConversion & IME_CMODE_SOFTKBD;
        if LangID != 0 {
            if (PRIMARYLANGID!(LangID)) as u32 == LANG_JAPANESE {
                dwConv |= IME_CMODE_ROMAN | IME_CMODE_FULLSHAPE | IME_CMODE_NATIVE;
            } else if (PRIMARYLANGID!(LangID)) as u32 != LANG_KOREAN {
                dwConv |= IME_CMODE_NATIVE;
            }
        }
        imcLock.get().fdwConversion |= dwConv;
        imcLock.get().fdwInit |= INIT_CONVERSION;
    }

    // Populate sentence mode
    imcLock.get().fdwSentence |= IME_SMODE_PHRASEPREDICT;

    // Populate LOGFONT
    if (imcLock.get().fdwInit & INIT_LOGFONT) == 0 {
        // Get logical font
        let mut lf: LOGFONTW = core::mem::zeroed();
        let hDC: HDC = GetDC(imcLock.get().hWnd);
        let hFont: HGDIOBJ = GetCurrentObject(hDC, OBJ_FONT);
        GetObjectW(
            hFont,
            (core::mem::size_of::<LOGFONTW>()) as i32,
            core::ptr::addr_of_mut!(lf) as LPVOID,
        );
        ReleaseDC(imcLock.get().hWnd, hDC);

        // LOGFONTW from this island into cicero's: same layout, a plain struct copy in the C++.
        core::ptr::write(core::ptr::addr_of_mut!(imcLock.get().lfFont.W) as *mut LOGFONTW, lf);
        imcLock.get().fdwInit |= INIT_LOGFONT;
    }
    imcLock.get().lfFont.W.lfCharSet = GetCharsetFromLangId((LangID) as DWORD);

    imcLock.InitContext();

    imcLock.m_hr
}
#[no_mangle]
pub unsafe extern "C" fn Inquire(
    lpIMEInfo: LPIMEINFO,
    lpszWndClass: LPWSTR,
    dwSystemInfoFlags: DWORD,
    hKL: HKL,
) -> HRESULT {
    if lpIMEInfo.is_null() {
        return E_OUTOFMEMORY;
    }

    StringCchCopyW(lpszWndClass, 64, CPP_COM_WSTR_56054C132291.as_ptr());
    (*lpIMEInfo).dwPrivateDataSize = 0;

    match (LOWORD!(hKL as usize as u32)) as i32 { // Language ID
        LANGID_JAPANESE => { // Japanese
            (*lpIMEInfo).fdwProperty = IME_PROP_COMPLETE_ON_UNSELECT | IME_PROP_SPECIAL_UI |
                                       IME_PROP_AT_CARET | IME_PROP_NEED_ALTKEY |
                                       IME_PROP_KBD_CHAR_FIRST;
            (*lpIMEInfo).fdwConversionCaps = IME_CMODE_FULLSHAPE | IME_CMODE_KATAKANA |
                                             IME_CMODE_NATIVE;
            (*lpIMEInfo).fdwSentenceCaps = IME_SMODE_CONVERSATION | IME_SMODE_PLAURALCLAUSE;
            (*lpIMEInfo).fdwSelectCaps = SELECT_CAP_SENTENCE | SELECT_CAP_CONVERSION;
            (*lpIMEInfo).fdwSCSCaps = SCS_CAP_SETRECONVERTSTRING | SCS_CAP_MAKEREAD |
                                      SCS_CAP_COMPSTR;
            (*lpIMEInfo).fdwUICaps = UI_CAP_ROT90;
        }
        LANGID_KOREAN => { // Korean
            (*lpIMEInfo).fdwProperty = IME_PROP_COMPLETE_ON_UNSELECT | IME_PROP_SPECIAL_UI |
                                       IME_PROP_AT_CARET | IME_PROP_NEED_ALTKEY |
                                       IME_PROP_KBD_CHAR_FIRST;
            (*lpIMEInfo).fdwConversionCaps = IME_CMODE_FULLSHAPE | IME_CMODE_NATIVE;
            (*lpIMEInfo).fdwSentenceCaps = 0;
            (*lpIMEInfo).fdwSCSCaps = SCS_CAP_SETRECONVERTSTRING | SCS_CAP_COMPSTR;
            (*lpIMEInfo).fdwSelectCaps = SELECT_CAP_CONVERSION;
            (*lpIMEInfo).fdwUICaps = UI_CAP_ROT90;
        }
        LANGID_CHINESE_SIMPLIFIED | LANGID_CHINESE_TRADITIONAL => { // Simplified / Traditional Chinese
            (*lpIMEInfo).fdwProperty = IME_PROP_SPECIAL_UI | IME_PROP_AT_CARET |
                                       IME_PROP_NEED_ALTKEY | IME_PROP_KBD_CHAR_FIRST;
            (*lpIMEInfo).fdwConversionCaps = IME_CMODE_FULLSHAPE | IME_CMODE_NATIVE;
            (*lpIMEInfo).fdwSentenceCaps = SELECT_CAP_CONVERSION;
            (*lpIMEInfo).fdwSelectCaps = 0;
            (*lpIMEInfo).fdwSCSCaps = SCS_CAP_SETRECONVERTSTRING | SCS_CAP_MAKEREAD |
                                      SCS_CAP_COMPSTR;
            (*lpIMEInfo).fdwUICaps = UI_CAP_ROT90;
        }
        _ => { // Otherwise
            (*lpIMEInfo).fdwProperty = IME_PROP_UNICODE | IME_PROP_AT_CARET;
            (*lpIMEInfo).fdwConversionCaps = 0;
            (*lpIMEInfo).fdwSentenceCaps = 0;
            (*lpIMEInfo).fdwSCSCaps = 0;
            (*lpIMEInfo).fdwUICaps = 0;
            (*lpIMEInfo).fdwSelectCaps = 0;
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn ImeInquire(
    lpIMEInfo: LPIMEINFO,
    lpszWndClass: LPWSTR,
    dwSystemInfoFlags: DWORD,
) -> BOOL {
    // TRACE("(%p, %p, 0x%lX)\n", lpIMEInfo, lpszWndClass, dwSystemInfoFlags);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ImeConversionList(
    hIMC: HIMC,
    lpSrc: LPCWSTR,
    lpDst: LPCANDIDATELIST,
    dwBufLen: DWORD,
    uFlag: UINT,
) -> DWORD {
    // TRACE("(%p, %s, %p, 0x%lX, %u)\n", hIMC, debugstr_w(lpSrc), lpDst, dwBufLen, uFlag);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ImeRegisterWord(
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszString: LPCWSTR,
) -> BOOL {
    // TRACE("(%s, 0x%lX, %s)\n", debugstr_w(lpszReading), dwStyle, debugstr_w(lpszString));
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ImeUnregisterWord(
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszString: LPCWSTR,
) -> BOOL {
    // TRACE("(%s, 0x%lX, %s)\n", debugstr_w(lpszReading), dwStyle, debugstr_w(lpszString));
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ImeGetRegisterWordStyle(
    nItem: UINT,
    lpStyleBuf: LPSTYLEBUFW,
) -> UINT {
    // TRACE("(%u, %p)\n", nItem, lpStyleBuf);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ImeEnumRegisterWord(
    lpfnEnumProc: REGISTERWORDENUMPROCW,
    lpszReading: LPCWSTR,
    dwStyle: DWORD,
    lpszString: LPCWSTR,
    lpData: LPVOID,
) -> UINT {
    // TRACE("(%p, %s, %lu, %s, %p)\n", lpfnEnumProc, debugstr_w(lpszReading), dwStyle, debugstr_w(lpszString), lpData);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ImeConfigure(hKL: HKL, hWnd: HWND, dwMode: DWORD, lpData: LPVOID) -> BOOL {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() || (*pTLS).m_pBridge.is_null() || (*pTLS).m_pThreadMgr.is_null() {
        return (FALSE) as BOOL;
    }

    let pBridge = (*pTLS).m_pBridge;
    let pThreadMgr = (*pTLS).m_pThreadMgr;

    if dwMode & 0x1 != 0 {
        return ((*pBridge).ConfigureGeneral(pTLS, (pThreadMgr).cast(), hKL, hWnd) == S_OK) as i32;
    }

    if dwMode & 0x2 != 0 {
        return ((*pBridge).ConfigureRegisterWord(pTLS, (pThreadMgr).cast(), hKL, hWnd, lpData) == S_OK) as i32;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImeDestroy(uReserved: u32) -> BOOL {
    let pTLS: *mut TLS = TLS::PeekTLS();
    if !pTLS.is_null() {
        return (FALSE) as BOOL;
    }

    if (*pTLS).m_pBridge.is_null() || (*pTLS).m_pThreadMgr.is_null() {
        return (FALSE) as BOOL;
    }

    if ((*pTLS).m_dwSystemInfoFlags & IME_SYSINFO_WINLOGON) != 0 {
        return (TRUE) as BOOL;
    }

    if (*(*pTLS).m_pBridge).DeactivateIMMX(pTLS, (*pTLS).m_pThreadMgr) != S_OK {
        return (FALSE) as BOOL;
    }

    (*(*pTLS).m_pBridge).UnInitIMMX(pTLS)
}

#[no_mangle]
pub unsafe extern "system" fn ImeEscape(hIMC: HIMC, uEscape: UINT, lpData: LPVOID) -> LRESULT {
    // TRACE("(%p, %u, %p)\n", hIMC, uEscape, lpData);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ImeProcessKey(
    hIMC: HIMC,
    uVirtKey: UINT,
    lParam: LPARAM,
    lpbKeyState: LPBYTE,
) -> BOOL {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return (FALSE) as BOOL;
    }

    let pBridge: *mut CicBridge = (*pTLS).m_pBridge;
    let pThreadMgr: *mut ITfThreadMgr_P = (*pTLS).m_pThreadMgr;
    if pBridge.is_null() || pThreadMgr.is_null() {
        return (FALSE) as BOOL;
    }

    if ((*pTLS).m_dwFlags1 & 0x1) != 0 {
        let mut pDocMgr: *mut ITfDocumentMgr = core::ptr::null_mut();
        (*pThreadMgr).GetFocus(core::ptr::addr_of_mut!(pDocMgr));
        if !pDocMgr.is_null() && CicBridge::IsOwnDim(pDocMgr) == 0 {
            (*pDocMgr).Release();
            return (FALSE) as BOOL;
        }

        if !pDocMgr.is_null() {
            (*pDocMgr).Release();
        }
    }

    let LangID: LANGID = LOWORD!(GetKeyboardLayout(0));
    if (((*pTLS).m_dwFlags2 & 1) != 0
        && MsimtfIsGuidMapEnable(hIMC, core::ptr::null_mut()) != 0)
        || ((lParam & ((KF_ALTDOWN << 16) as LPARAM)) != 0
            && (LangID == MAKELANGID!(LANG_JAPANESE, SUBLANG_DEFAULT))
            && IsVKDBEKey(uVirtKey) != 0)
    {
        return (FALSE) as BOOL;
    }

    let mut nUnknown60: INT = 0;
    return (*pBridge).ProcessKey(
        pTLS,
        pThreadMgr,
        hIMC,
        uVirtKey as WPARAM,
        lParam,
        lpbKeyState,
        core::ptr::addr_of_mut!(nUnknown60),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ImeSelect(hIMC: HIMC, fSelect: BOOL) -> BOOL {
    // TRACE("(%p, %u)\n", hIMC, fSelect);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ImeSetActiveContext(hIMC: HIMC, fFlag: BOOL) -> BOOL {
    // TRACE("(%p, %u)\n", hIMC, fFlag);
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn ImeToAsciiEx(
    uVirtKey: u32,
    uScanCode: u32,
    lpbKeyState: *const u8,
    lpTransMsgList: LPTRANSMSGLIST,
    fuState: u32,
    hIMC: HIMC,
) -> u32 {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return 0;
    }

    let pBridge = (*pTLS).m_pBridge;
    let pThreadMgr = (*pTLS).m_pThreadMgr;
    if pBridge.is_null() || pThreadMgr.is_null() {
        return 0;
    }

    let mut ret: u32 = 0;
    let hr: HRESULT = (*pBridge).ToAsciiEx(
        pTLS,
        pThreadMgr,
        uVirtKey,
        uScanCode,
        lpbKeyState as LPBYTE, // CONST LPBYTE: the pointer is const, not the bytes
        lpTransMsgList,
        fuState,
        hIMC,
        core::ptr::addr_of_mut!(ret),
    );
    return if hr == S_OK { ret } else { 0 };
}

#[no_mangle]
pub unsafe extern "system" fn NotifyIME(
    hIMC: HIMC,
    dwAction: DWORD,
    dwIndex: DWORD,
    dwValue: DWORD_PTR,
) -> BOOL {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return (FALSE) as BOOL;
    }

    let pBridge: *mut CicBridge = (*pTLS).m_pBridge;
    let pThreadMgr: *mut ITfThreadMgr_P = (*pTLS).m_pThreadMgr;
    if pBridge.is_null() || pThreadMgr.is_null() {
        return (FALSE) as BOOL;
    }

    let hr: HRESULT = (*pBridge).Notify(pTLS, pThreadMgr, hIMC, dwAction, dwIndex, dwValue);
    return (hr == S_OK) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImeSetCompositionString(hIMC: HIMC, dwIndex: DWORD, lpComp: LPCVOID, dwCompLen: DWORD, lpRead: LPCVOID, dwReadLen: DWORD) -> BOOL {
    let mut pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return (FALSE) as BOOL;
    }

    let mut pBridge: *mut CicBridge = (*pTLS).m_pBridge;
    let mut pThreadMgr: *mut ITfThreadMgr_P = (*pTLS).m_pThreadMgr;
    if pBridge.is_null() || pThreadMgr.is_null() {
        return (FALSE) as BOOL;
    }

    return (*pBridge).SetCompositionString(pTLS, pThreadMgr, hIMC, dwIndex, lpComp, dwCompLen, lpRead, dwReadLen);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeInquireExW(
    lpIMEInfo: LPIMEINFO,
    lpszWndClass: LPWSTR,
    mut dwSystemInfoFlags: DWORD,
    hKL: HKL,
) -> HRESULT {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return E_OUTOFMEMORY;
    }

    if IsInteractiveUserLogon() == 0 {
        dwSystemInfoFlags |= IME_SYSINFO_WINLOGON;
        g_bWinLogon = (TRUE) as i32;
    }

    (*pTLS).m_dwSystemInfoFlags = dwSystemInfoFlags;

    return Inquire(lpIMEInfo, lpszWndClass, dwSystemInfoFlags, hKL);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeSelectEx(hIMC: HIMC, fSelect: BOOL, hKL: HKL) -> BOOL {
    let pTLS: *mut TLS = TLS::PeekTLS();
    if pTLS.is_null() {
        return E_OUTOFMEMORY;
    }

    InternalSelectEx(hIMC, fSelect, LOWORD!(hKL));

    if (*pTLS).m_pBridge.is_null() || (*pTLS).m_pThreadMgr.is_null() {
        return E_OUTOFMEMORY;
    }

    return (*(*pTLS).m_pBridge).SelectEx(pTLS, (*pTLS).m_pThreadMgr, hIMC, fSelect, hKL);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeEscapeEx(hIMC: HIMC, uSubFunc: UINT, lpData: LPVOID, hKL: HKL) -> LRESULT {
    if LOWORD!(hKL) != MAKELANGID!(LANG_KOREAN, SUBLANG_DEFAULT) {
        return 0;
    }

    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() || (*pTLS).m_pBridge.is_null() {
        return 0;
    }

    return (*(*pTLS).m_pBridge).EscapeKorean(pTLS, hIMC, uSubFunc, lpData);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeGetGuidAtom(hIMC: HIMC, dwUnknown: DWORD, pdwGuidAtom: LPDWORD) -> HRESULT {
    let mut imcLock = CicIMCLock__make_0((hIMC).cast());
    if ((imcLock.m_hr) < 0) {
        return imcLock.m_hr;
    }

    let mut imccLock = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCtfImeContext);
    if ((imccLock.m_hr) < 0) {
        return imccLock.m_hr;
    }

    if (imccLock.get().m_pCicIC).is_null() {
        return E_OUTOFMEMORY;
    }

    return (*imccLock.get().m_pCicIC).GetGuidAtom(&mut imcLock, dwUnknown as BYTE, pdwGuidAtom);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeIsGuidMapEnable(hIMC: HIMC) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut imcLock = CicIMCLock__make_0((hIMC).cast());
    if imcLock.m_hr >= 0 {
        ret = ((imcLock.get().fdwInit & INIT_GUIDMAP) != 0) as BOOL;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeCreateThreadMgr() -> HRESULT {
    let mut pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return E_OUTOFMEMORY;
    }

    if (*pTLS).m_pBridge.is_null() {
        (*pTLS).m_pBridge = CicBridge__new_in_0(cpp_alloc_CicNoThrow::<CicBridge>());
        if (*pTLS).m_pBridge.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    let mut hr: HRESULT = S_OK;
    if g_bWinLogon == 0 && ((*pTLS).m_dwSystemInfoFlags & IME_SYSINFO_WINLOGON) == 0 {
        hr = (*(*pTLS).m_pBridge).InitIMMX(pTLS);
        if hr >= 0 {
            if (*pTLS).m_pThreadMgr.is_null() {
                return E_OUTOFMEMORY;
            }

            hr = (*(*pTLS).m_pBridge).ActivateIMMX(pTLS, (*pTLS).m_pThreadMgr);
            if hr < 0 {
                (*(*pTLS).m_pBridge).UnInitIMMX(pTLS);
            }
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeDestroyThreadMgr() -> HRESULT {
    let mut pTLS: *mut TLS = TLS::PeekTLS();
    if pTLS.is_null() {
        return E_OUTOFMEMORY;
    }

    if !(*pTLS).m_pBridge.is_null() {
        (*pTLS).m_pBridge = CicBridge__new_in_0(cpp_alloc_CicNoThrow::<CicBridge>());
        if (*pTLS).m_pBridge.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    if (*pTLS).m_pThreadMgr.is_null() {
        return E_OUTOFMEMORY;
    }

    if ((*pTLS).m_dwSystemInfoFlags & IME_SYSINFO_WINLOGON) != 0 {
        return S_OK;
    }

    let mut hr: HRESULT = (*(*pTLS).m_pBridge).DeactivateIMMX(pTLS, (*pTLS).m_pThreadMgr);
    if hr == S_OK {
        (*(*pTLS).m_pBridge).UnInitIMMX(pTLS);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeCreateInputContext(hIMC: HIMC) -> HRESULT {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() || (*pTLS).m_pBridge.is_null() {
        return E_OUTOFMEMORY;
    }

    return (*(*pTLS).m_pBridge).CreateInputContext(pTLS, hIMC);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeDestroyInputContext(hIMC: HIMC) -> HRESULT {
    let pTLS: *mut TLS = TLS::PeekTLS();
    if pTLS.is_null() || (*pTLS).m_pBridge.is_null() {
        return E_OUTOFMEMORY;
    }

    return (*(*pTLS).m_pBridge).DestroyInputContext(pTLS, hIMC);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeSetActiveContextAlways(hIMC: HIMC, fActive: BOOL, hWnd: HWND, hKL: HKL) -> HRESULT {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() || (*pTLS).m_pBridge.is_null() {
        return E_OUTOFMEMORY;
    }
    return (*(*pTLS).m_pBridge).SetActiveContextAlways(pTLS, hIMC, fActive, hWnd, hKL);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeProcessCicHotkey(hIMC: HIMC, vKey: UINT, lParam: LPARAM) -> HRESULT {
    let pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() {
        return S_OK;
    }

    let mut hr: HRESULT = S_OK;
    let mut pThreadMgr: *mut ITfThreadMgr = core::ptr::null_mut();
    let mut pThreadMgr_P: *mut ITfThreadMgr_P = core::ptr::null_mut();
    if (TF_GetThreadMgr(core::ptr::addr_of_mut!(pThreadMgr)) == S_OK)
        && ((*pThreadMgr).QueryInterface(
            &IID_ITfThreadMgr_P,
            core::ptr::addr_of_mut!(pThreadMgr_P) as *mut _,
        ) == S_OK)
        && ((CtfImmIsCiceroStartedInThread()) != 0)
    {
        let mut hr2: HRESULT;
        hr2 = core::mem::zeroed();
        if (*pThreadMgr_P).CallImm32HotkeyHandler(vKey, lParam, core::ptr::addr_of_mut!(hr2)) >= 0 {
            hr = hr2;
        }
    }

    if !pThreadMgr.is_null() {
        (*pThreadMgr).Release();
    }
    if !pThreadMgr_P.is_null() {
        (*pThreadMgr_P).Release();
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeDispatchDefImeMessage(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pTLS: *mut TLS = TLS::GetTLS();
    if !pTLS.is_null() {
        if uMsg == WM_CREATE {
            (*pTLS).m_cWnds += 1;
        } else if uMsg == WM_DESTROY {
            (*pTLS).m_cWnds -= 1;
        }
    }

    if IsMsImeMessage(uMsg) == 0 {
        return 0;
    }

    let hKL: HKL = GetKeyboardLayout(0);
    if IS_IME_HKL!(hKL) != 0 {
        return 0;
    }

    let hImeWnd: HWND = SendMessageW(hWnd, WM_IME_NOTIFY, 0x17, 0) as HWND;
    if IsWindow(hImeWnd) == 0 {
        return 0;
    }

    return SendMessageW(hImeWnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeIsIME(hKL: HKL) -> BOOL {
    if IS_IME_HKL!(hKL) != 0 {
        return (TRUE) as BOOL;
    }

    let mut pTLS: *mut TLS = TLS::GetTLS();
    if pTLS.is_null() || (*pTLS).m_pProfile.is_null() {
        return (FALSE) as BOOL;
    }

    // The return value of CicProfile::IsIME is brain-damaged
    return ((*(*pTLS).m_pProfile).IsIME(hKL) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImeThreadDetach() -> HRESULT {
    ImeDestroy(0);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn AttachIME() -> i32 {
    (RegisterImeClass() != 0 && RegisterMSIMEMessage() != 0) as i32
}

#[no_mangle]
pub unsafe extern "C" fn DetachIME() {
    UnregisterImeClass();
}
#[no_mangle]
pub unsafe extern "C" fn TFUninitLib() {
    if !g_pPropCache.is_null() {
        cpp_delete(g_pPropCache);
        g_pPropCache = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ProcessAttach(hinstDLL: HINSTANCE) -> BOOL {
    g_hInst = hinstDLL;

    InitializeCriticalSectionAndSpinCount(core::ptr::addr_of_mut!(g_csLock), 0);

    if (TLS::Initialize()) == 0 {
        return (FALSE) as BOOL;
    }

    cicGetOSInfo(core::ptr::addr_of_mut!(g_uACP), core::ptr::addr_of_mut!(g_dwOSInfo));

    cicInitUIFLib();

    if TFInitLib(None) == 0 { // the C++ default argument: no user CoCreateInstance
        return (FALSE) as BOOL;
    }

    gfTFInitLib = (TRUE) as i32;
    return AttachIME();
}

#[no_mangle]
pub unsafe extern "C" fn ProcessDetach(hinstDLL: HINSTANCE) {
    TF_DllDetachInOther();

    if gfTFInitLib != 0 {
        DetachIME();
        TFUninitLib();
    }

    DeleteCriticalSection(core::ptr::addr_of_mut!(g_csLock));
    TLS::InternalDestroyTLS();
    TLS::Uninitialize();
    cicDoneUIFLib();
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            if ProcessAttach(hinstDLL) == 0 {
                ProcessDetach(hinstDLL);
                return (FALSE) as BOOL;
            }
        }
        DLL_PROCESS_DETACH => {
            ProcessDetach(hinstDLL);
        }
        DLL_THREAD_DETACH => {
            TF_DllDetachInOther();
            CtfImeThreadDetach();
            TLS::InternalDestroyTLS();
        }
        _ => {}
    }
    (TRUE) as i32
}
