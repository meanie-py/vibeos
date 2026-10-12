use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_56054C132291: [WCHAR; 12] = [0x004D, 0x0053, 0x0043, 0x0054, 0x0046, 0x0049, 0x004D, 0x0045, 0x0020, 0x0055, 0x0049, 0x0000];
pub static CPP_COM_WSTR_62586A0891BF: [WCHAR; 21] = [0x004D, 0x0053, 0x0043, 0x0054, 0x0046, 0x0049, 0x004D, 0x0045, 0x0020, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x0073, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_91CDBB74B095: [WCHAR; 8] = [0x0054, 0x004F, 0x004F, 0x004C, 0x0042, 0x0041, 0x0052, 0x0000];
pub static CPP_COM_WSTR_D3128F43EECC: [WCHAR; 8] = [0x0054, 0x0041, 0x0053, 0x004B, 0x0042, 0x0041, 0x0052, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut WM_MSIME_UIREADY: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_RECONVERTREQUEST: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_RECONVERT: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_DOCUMENTFEED: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_QUERYPOSITION: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_MODEBIAS: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_SHOWIMEPAD: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_MOUSE: u32 = 0;
#[no_mangle]
pub static mut WM_MSIME_KEYMAP: u32 = 0;





































































#[no_mangle]
pub unsafe extern "C" fn IsMsImeMessage(uMsg: UINT) -> BOOL {
    if uMsg == WM_MSIME_SERVICE ||
            uMsg == WM_MSIME_UIREADY ||
            uMsg == WM_MSIME_RECONVERTREQUEST ||
            uMsg == WM_MSIME_RECONVERT ||
            uMsg == WM_MSIME_DOCUMENTFEED ||
            uMsg == WM_MSIME_QUERYPOSITION ||
            uMsg == WM_MSIME_MODEBIAS ||
            uMsg == WM_MSIME_SHOWIMEPAD ||
            uMsg == WM_MSIME_MOUSE ||
            uMsg == WM_MSIME_KEYMAP {
        return 1;
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterMSIMEMessage() -> i32 {
    // Using ANSI (A) version here can reduce binary size.
    WM_MSIME_SERVICE = RegisterWindowMessageA(b"MSIMEService\0".as_ptr() as *const i8);
    WM_MSIME_UIREADY = RegisterWindowMessageA(b"MSIMEUIReady\0".as_ptr() as *const i8);
    WM_MSIME_RECONVERTREQUEST = RegisterWindowMessageA(b"MSIMEReconvertRequest\0".as_ptr() as *const i8);
    WM_MSIME_RECONVERT = RegisterWindowMessageA(b"MSIMEReconvert\0".as_ptr() as *const i8);
    WM_MSIME_DOCUMENTFEED = RegisterWindowMessageA(b"MSIMEDocumentFeed\0".as_ptr() as *const i8);
    WM_MSIME_QUERYPOSITION = RegisterWindowMessageA(b"MSIMEQueryPosition\0".as_ptr() as *const i8);
    WM_MSIME_MODEBIAS = RegisterWindowMessageA(b"MSIMEModeBias\0".as_ptr() as *const i8);
    WM_MSIME_SHOWIMEPAD = RegisterWindowMessageA(b"MSIMEShowImePad\0".as_ptr() as *const i8);
    WM_MSIME_MOUSE = RegisterWindowMessageA(b"MSIMEMouseOperation\0".as_ptr() as *const i8);
    WM_MSIME_KEYMAP = RegisterWindowMessageA(b"MSIMEKeyMap\0".as_ptr() as *const i8);
    return ((WM_MSIME_SERVICE != 0)
            && (WM_MSIME_UIREADY != 0)
            && (WM_MSIME_RECONVERTREQUEST != 0)
            && (WM_MSIME_RECONVERT != 0)
            && (WM_MSIME_DOCUMENTFEED != 0)
            && (WM_MSIME_QUERYPOSITION != 0)
            && (WM_MSIME_MODEBIAS != 0)
            && (WM_MSIME_SHOWIMEPAD != 0)
            && (WM_MSIME_MOUSE != 0)
            && (WM_MSIME_KEYMAP != 0)) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn UIWndProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    return CIMEUIWindowHandler::ImeUIWndProcWorker(hWnd, uMsg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn RegisterImeClass() -> BOOL {
    let mut wcx: WNDCLASSEXW = core::mem::zeroed();

    if GetClassInfoExW(g_hInst, CPP_COM_WSTR_56054C132291.as_ptr(), core::ptr::addr_of_mut!(wcx)) == 0 {
        ZeroMemory!(core::ptr::addr_of_mut!(wcx) as *mut _, core::mem::size_of::<WNDCLASSEXW>());
        wcx.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
        wcx.cbWndExtra = UI_GWLP_SIZE;
        wcx.hIcon = LoadIconW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
        wcx.hInstance = g_hInst;
        wcx.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
        wcx.hbrBackground = GetStockObject((NULL_BRUSH as i32)) as HBRUSH;
        wcx.style = CS_IME | CS_GLOBALCLASS;
        wcx.lpfnWndProc = Some(UIWndProc);
        wcx.lpszClassName = CPP_COM_WSTR_56054C132291.as_ptr();
        if RegisterClassExW(core::ptr::addr_of_mut!(wcx)) == 0 {
            return (FALSE) as BOOL;
        }
    }

    if GetClassInfoExW(g_hInst, CPP_COM_WSTR_62586A0891BF.as_ptr(), core::ptr::addr_of_mut!(wcx)) == 0 {
        ZeroMemory!(core::ptr::addr_of_mut!(wcx) as *mut _, core::mem::size_of::<WNDCLASSEXW>());
        wcx.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
        wcx.cbWndExtra = (UICOMP_GWLP_SIZE as i32);
        wcx.hIcon = core::ptr::null_mut();
        wcx.hInstance = g_hInst;
        wcx.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_IBEAM as LPCWSTR);
        wcx.hbrBackground = GetStockObject((NULL_BRUSH as i32)) as HBRUSH;
        wcx.style = CS_IME | CS_HREDRAW | CS_VREDRAW;
        wcx.lpfnWndProc = Some(UIComposition::CompWndProc);
        wcx.lpszClassName = CPP_COM_WSTR_62586A0891BF.as_ptr();
        if RegisterClassExW(core::ptr::addr_of_mut!(wcx)) == 0 {
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterImeClass() {
    let mut wcx: WNDCLASSEXW = core::mem::zeroed();

    GetClassInfoExW(g_hInst, CPP_COM_WSTR_56054C132291.as_ptr(), core::ptr::addr_of_mut!(wcx));
    UnregisterClassW(CPP_COM_WSTR_56054C132291.as_ptr(), g_hInst);
    DestroyIcon(wcx.hIcon);
    DestroyIcon(wcx.hIconSm);

    GetClassInfoExW(g_hInst, CPP_COM_WSTR_62586A0891BF.as_ptr(), core::ptr::addr_of_mut!(wcx));
    UnregisterClassW(CPP_COM_WSTR_62586A0891BF.as_ptr(), g_hInst);
    DestroyIcon(wcx.hIcon);
    DestroyIcon(wcx.hIconSm);
}

impl CDefCompFrameGripper {
    pub unsafe fn __ctor_0(&mut self, pDefCompFrameWindow: *mut CDefCompFrameWindow, prc: LPCRECT, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_pDefCompFrameWindow = pDefCompFrameWindow;
        }
    }
}

impl CCompFinalizeButton {
    pub unsafe fn __ctor_0(&mut self, pParent: *mut CCompFrameWindow, nObjectID: DWORD, prc: LPCRECT, style: DWORD, dwButtonFlags: DWORD, pszText: LPCWSTR) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_pCompFrameWindow = pParent;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        let hIcon: HICON = (*this_).__base0.GetIcon();
        if !hIcon.is_null() {
            DestroyIcon(hIcon);
            (*this_).__base0.SetIcon(core::ptr::null_mut());
        }
    }
    pub unsafe fn OnLeftClick__body(&mut self) {
        let this_: *mut Self = self;
        let hIMC = (*(*this_).m_pCompFrameWindow).m_hIMC;
        if !hIMC.is_null() {
            ImmNotifyIME(hIMC, NI_COMPOSITIONSTR, CPS_COMPLETE, 0);
        }
    }
}

impl CCompFrameWindow {
    pub unsafe fn __ctor_0(&mut self, hIMC: HIMC, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_hIMC = hIMC;
        }
    }
}

impl CCompButtonFrameWindow {
    pub unsafe fn __ctor_0(&mut self, hIMC: HIMC, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hIMC, style);
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn Init(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pFinalizeButton.is_null() {
            return;
        }
    
        let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        (*this_).m_pFinalizeButton = CCompFinalizeButton__new_in_0(cpp_alloc_CicNoThrow::<CCompFinalizeButton>(), (this_).cast(), 0, core::ptr::addr_of!(rc), 0, 0x10000, core::ptr::null_mut()) as *mut CCompFinalizeButton;
    
        (*(*this_).m_pFinalizeButton).Initialize();
        (*(*this_).m_pFinalizeButton).Init();
    
        let hIcon: HICON = LoadImageW(g_hInst, MAKEINTRESOURCEW!(IDI_DOWN), IMAGE_ICON, 16, 16, 0) as HICON;
        (*(*this_).m_pFinalizeButton).SetIcon(hIcon);
    
        let mut szText: [u16; 256] = [0; 256];
        LoadStringW(g_hInst, IDS_FINALIZE_STRING, szText.as_mut_ptr(), (_countof!(szText)) as i32);
        (*(*this_).m_pFinalizeButton).SetToolTip(szText.as_mut_ptr());
    
        (*this_).AddUIObj((core::ptr::addr_of_mut!((*(*this_).m_pFinalizeButton).__base0)).cast());
    }
    pub unsafe fn MoveShow(&mut self, x: LONG, y: LONG, bShow: BOOL) {
        let this_: *mut Self = self;
        let mut nWidth: INT = (*this_).m_Margins.cxRightWidth + (*this_).m_Margins.cxLeftWidth + 18;
        let mut nHeight: INT = (*this_).m_Margins.cyBottomHeight + (*this_).m_Margins.cyTopHeight + 18;
        (*this_).Move(x, y, nWidth + 4, nHeight + 4);
    
        if !(*this_).m_pFinalizeButton.is_null() {
            let mut rc: RECT = RECT {
                left: 1,
                top: 1,
                right: nWidth + 1,
                bottom: nHeight + 1,
            };
            (*(*this_).m_pFinalizeButton).SetRect(core::ptr::addr_of_mut!(rc));
        }
    
        (*this_).Show(bShow);
    }
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        SetWindowTheme(hWnd, CPP_COM_WSTR_91CDBB74B095.as_ptr(), core::ptr::null_mut());
    
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).m_Margins) as *mut core::ffi::c_void, core::mem::size_of::<MARGINS>());
    
        let mut theme: CUIFTheme = CUIFTheme__zeroed();
        theme.m_hTheme = core::ptr::null_mut();
        theme.m_iPartId = 1;
        theme.m_iStateId = 0;
        theme.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
        if (theme.InternalOpenThemeData(hWnd)) >= 0 {
            theme.GetThemeMargins(
                core::ptr::null_mut(),
                1,
                3602,
                core::ptr::null_mut(),
                (core::ptr::addr_of_mut!((*this_).m_Margins)).cast(),
            );
        }
    }
}

impl CDefCompFrameWindow {
    pub unsafe fn __ctor_0(&mut self, hIMC: HIMC, style: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(hIMC, style);
        (*this_).__init_vtables();
        {
            (*this_).LoadPosition();
            (*this_).m_iPartId = 1;
            (*this_).m_iStateId = 1;
            (*this_).m_pszClassList = CPP_COM_WSTR_D3128F43EECC.as_ptr();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).SavePosition();
    }
    pub unsafe fn Init(&mut self) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
    
        if (*this_).m_pGripper.is_null() {
            ZeroMemory!(core::ptr::addr_of_mut!(rc) as *mut _, core::mem::size_of::<RECT>());
            (*this_).m_pGripper = CDefCompFrameGripper__new_in_0(cpp_alloc_CicNoThrow::<CDefCompFrameGripper>(), this_, core::ptr::addr_of_mut!(rc), 0);
            (*(*this_).m_pGripper).Initialize();
            (*this_).AddUIObj((core::ptr::addr_of_mut!((*(*this_).m_pGripper).__base0)).cast());
        }
    
        if (*this_).m_pFinalizeButton.is_null() {
            ZeroMemory!(core::ptr::addr_of_mut!(rc) as *mut _, core::mem::size_of::<RECT>());
            (*this_).m_pFinalizeButton = CCompFinalizeButton__new_in_0(cpp_alloc_CicNoThrow::<CCompFinalizeButton>(), this_ as *mut CCompFrameWindow, 0, core::ptr::addr_of_mut!(rc), 0, 0x10000, core::ptr::null_mut());
            (*(*this_).m_pFinalizeButton).Initialize();
            (*(*this_).m_pFinalizeButton).Init();
    
            let hIcon: HICON = LoadImageW(g_hInst, MAKEINTRESOURCEW!(IDI_DOWN), IMAGE_ICON, 16, 16, 0) as HICON;
            (*(*this_).m_pFinalizeButton).SetIcon(hIcon);
    
            let mut szText: [u16; 256] = [0u16; 256];
            LoadStringW(g_hInst, IDS_FINALIZE_STRING, szText.as_mut_ptr(), (_countof!(szText)) as i32);
            (*this_).SetToolTip(szText.as_ptr());
    
            (*this_).AddUIObj((core::ptr::addr_of_mut!((*(*this_).m_pFinalizeButton).__base0)).cast());
        }
    }
    pub unsafe fn GetGripperWidth(&mut self) -> INT {
        let this_: *mut Self = self;
        if (*this_).m_pGripper.is_null() || (((*(*this_).m_pGripper).EnsureThemeData((*this_).m_hWnd)) < 0) {
            return 5;
        }
    
        let mut ret: INT = -1;
        let hDC: HDC = GetDC((*this_).m_hWnd);
        let mut partSize: SIZE = core::mem::zeroed();
        if ((*(*this_).m_pGripper).GetThemePartSize(hDC, 1, core::ptr::null_mut(), TS_TRUE, core::ptr::addr_of_mut!(partSize)) >= 0) {
            ret = partSize.cx + 4;
        }
    
        ReleaseDC((*this_).m_hWnd, hDC);
    
        return if ret < 0 { 5 } else { ret };
    }
    pub unsafe fn MyScreenToClient(&mut self, ppt: LPPOINT, prc: LPRECT) {
        let this_: *mut Self = self;
        if !ppt.is_null() {
            ScreenToClient((*this_).__base0.m_hWnd, ppt);
        }
    
        if !prc.is_null() {
            ScreenToClient((*this_).__base0.m_hWnd, prc as LPPOINT);
            ScreenToClient((*this_).__base0.m_hWnd, core::ptr::addr_of_mut!((*prc).right) as LPPOINT);
        }
    }
    pub unsafe fn SetCompStrRect(&mut self, nWidth: INT, nHeight: INT, bShow: BOOL) {
        let this_: *mut Self = self;
        let GripperWidth: INT = (*this_).GetGripperWidth();
    
        let mut rc: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base0.m_hWnd, core::ptr::addr_of_mut!(rc));
    
        (*this_).Move(rc.left, rc.top, GripperWidth + nWidth + 24, nHeight + 10);
    
        if !(*this_).m_pGripper.is_null() {
            rc = RECT {
                left: 2,
                top: 3,
                right: GripperWidth + 2,
                bottom: nHeight + 7,
            };
            (*(*this_).m_pGripper).SetRect(core::ptr::addr_of_mut!(rc));
        }
    
        if !(*this_).m_pFinalizeButton.is_null() {
            rc = RECT {
                left: GripperWidth + nWidth + 4,
                top: 3,
                right: (*this_).m_Margins.cxLeftWidth + (*this_).m_Margins.cxRightWidth + GripperWidth + nWidth + 22,
                bottom: (*this_).m_Margins.cyBottomHeight + (*this_).m_Margins.cyTopHeight + 21,
            };
            (*(*this_).m_pFinalizeButton).SetRect(core::ptr::addr_of_mut!(rc));
        }
    
        (*this_).Show(bShow);
    
        MoveWindow((*this_).m_hwndCompStr, GripperWidth + 2, 7, nWidth, nHeight, (TRUE) as BOOL);
        ShowWindow((*this_).m_hwndCompStr, if bShow != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
    }
    pub unsafe fn LoadPosition(&mut self) {
        let this_: *mut Self = self;
        let mut x: DWORD = 0;
        let mut y: DWORD = 0;
    
        let error: LSTATUS;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        error = regKey.Open(HKEY_CURRENT_USER,
                            (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0043, 0x0055, 0x0041, 0x0053, 0x005c, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006c, 0x0074, 0x0043, 0x006f, 0x006d, 0x0070, 0x006f, 0x0073, 0x0069, 0x0074, 0x0069, 0x006f, 0x006e, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\CUAS\\DefaultCompositionWindow" */, KEY_READ);
        if (error) as u32 == ERROR_SUCCESS
        {
            regKey.QueryDword((&[0x004c, 0x0065, 0x0066, 0x0074, 0x0000u16]).as_ptr() /* TEXT "Left" */, core::ptr::addr_of_mut!(x));
            regKey.QueryDword((&[0x0054, 0x006f, 0x0070, 0x0000u16]).as_ptr() /* TEXT "Top" */, core::ptr::addr_of_mut!(y));
        }
    
        (*this_).Move((x) as i32, (y) as i32, 0, 0);
    }
    pub unsafe fn SavePosition(&mut self) {
        let this_: *mut Self = self;
        let mut error: LSTATUS;
        let mut regKey: CicRegKey = CicRegKey__zeroed();
        error = regKey.Create(HKEY_CURRENT_USER, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0043, 0x0055, 0x0041, 0x0053, 0x005c, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006c, 0x0074, 0x0043, 0x006f, 0x006d, 0x0070, 0x006f, 0x0073, 0x0069, 0x0074, 0x0069, 0x006f, 0x006e, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\CUAS\\DefaultCompositionWindow" */);
        if (error) as u32 == ERROR_SUCCESS {
            regKey.SetDword((&[0x004c, 0x0065, 0x0066, 0x0074, 0x0000u16]).as_ptr() /* TEXT "Left" */, ((*this_).m_nLeft) as u32);
            regKey.SetDword((&[0x0054, 0x006f, 0x0070, 0x0000u16]).as_ptr() /* TEXT "Top" */, ((*this_).m_nTop) as u32);
        }
    }
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        SetWindowTheme(hWnd, CPP_COM_WSTR_D3128F43EECC.as_ptr(), core::ptr::null_mut());
    
        ZeroMemory!(
            core::ptr::addr_of_mut!((*this_).m_Margins) as *mut core::ffi::c_void,
            core::mem::size_of::<MARGINS>(),
        );
    
        let mut theme: CUIFTheme = CUIFTheme__zeroed();
        theme.m_hTheme = core::ptr::null_mut();
        theme.m_iPartId = 1;
        theme.m_iStateId = 0;
        theme.m_pszClassList = CPP_COM_WSTR_91CDBB74B095.as_ptr();
        if (theme.InternalOpenThemeData(hWnd)) >= 0 {
            (*this_).GetThemeMargins(
                core::ptr::null_mut(),
                1,
                3602,
                core::ptr::null_mut(),
                (core::ptr::addr_of_mut!((*this_).m_Margins)).cast(),
            );
        }
    }
    pub unsafe fn OnSetCursor__body(&mut self, uMsg: UINT, x: LONG, y: LONG) -> BOOL {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hwndCompStr) == 0 {
            return (FALSE) as BOOL;
        }
    
        let mut rc: RECT = core::mem::zeroed();
        GetWindowRect((*this_).m_hwndCompStr, core::ptr::addr_of_mut!(rc));
        (*this_).MyScreenToClient(core::ptr::null_mut(), core::ptr::addr_of_mut!(rc));
        let mut pt: POINT = POINT { x: x, y: y };
        return PtInRect(core::ptr::addr_of!(rc), pt);
    }
    pub unsafe fn OnWindowPosChanged__body(&mut self, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        let mut imcLock = CicIMCLock__make_0(((*this_).__base0.m_hIMC).cast());
        if (imcLock.m_hr) >= 0 {
            SendMessage(imcLock.get().hWnd, WM_IME_NOTIFY, 0xF, (*this_).__base0.m_hIMC as LPARAM);
        }
        // imcLock stays held across DefWindowProc and unlocks at return, as in the C++.
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }
    pub unsafe fn HandleMouseMsg__body(&mut self, uMsg: UINT, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        if IsWindow((*this_).m_hwndCompStr) != 0 {
            let mut rc: RECT = core::mem::zeroed();
            GetWindowRect((*this_).m_hwndCompStr, core::ptr::addr_of_mut!(rc));
            (*this_).MyScreenToClient(core::ptr::null_mut(), core::ptr::addr_of_mut!(rc));
    
            let pt: POINT = POINT { x: x, y: y };
            if PtInRect(core::ptr::addr_of!(rc), pt) != 0 {
                SendMessage((*this_).m_hwndCompStr, 0x7E8, 0, 0);
            }
        }
    
        // C: CUIFWindow::this_->HandleMouseMsg(uMsg, x, y) — explicit base-class call, non-virtual
        (*this_).__base0.__base0.HandleMouseMsg(uMsg, x, y);
    }
}

impl CPolyText {
    pub unsafe fn ShiftPolyText(&mut self, xDelta: INT, yDelta: INT) -> HRESULT {
        let this_: *mut Self = self;
        let mut iItem: usize = 0;
        while iItem < (*this_).m_PolyTextArray.size() {
            let pPolyText: *mut POLYTEXTW = (*this_).GetPolyAt(iItem as INT);
            (*pPolyText).x += xDelta;
            (*pPolyText).y += yDelta;
            OffsetRect(core::ptr::addr_of_mut!((*pPolyText).rcl) as LPRECT, xDelta, yDelta);
            iItem += 1;
        }
        return S_OK;
    }
    pub unsafe fn GetPolyAt(&mut self, iItem: INT) -> *mut POLYTEXTW {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).m_PolyTextArray[iItem as usize]);
    }
    pub unsafe fn RemoveLastLine(&mut self, bHorizontal: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        let mut iItem: usize;
        let cItems: usize = (*this_).m_PolyTextArray.size();
        if cItems == 0 {
            return E_FAIL;
        }
    
        let pData1: *mut POLYTEXTW = core::ptr::addr_of_mut!((*this_).m_PolyTextArray[cItems - 1]);
    
        iItem = 0;
        while iItem < cItems {
            let pData2: *mut POLYTEXTW = core::ptr::addr_of_mut!((*this_).m_PolyTextArray[iItem]);
            if bHorizontal != 0 {
                if (*pData1).x == (*pData2).x {
                    break;
                }
            } else {
                if (*pData1).y == (*pData2).y {
                    break;
                }
            }
            iItem += 1;
        }
    
        if iItem >= cItems {
            return E_FAIL;
        }
    
        (*this_).m_PolyTextArray.Remove(iItem, cItems - iItem);
        (*this_).m_ValueArray.Remove(iItem, cItems - iItem);
        return S_OK;
    }
    pub unsafe fn RemoveAll(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_PolyTextArray.clear();
        (*this_).m_ValueArray.clear();
    }
}

impl COMPWND {
    pub unsafe fn _ClientToScreen(&mut self, prc: LPRECT) {
        let this_: *mut Self = self;
        ClientToScreen((*this_).m_hWnd, prc as LPPOINT);
        ClientToScreen((*this_).m_hWnd, core::ptr::addr_of_mut!((*prc).right) as LPPOINT);
    }
}

impl UIComposition {
    pub unsafe fn __ctor_0(&mut self, hwndParent: HWND) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).DestroyCompositionWindow();
    
        if !(*this_).m_hFont1.is_null() {
            DeleteObject((*this_).m_hFont1 as _);
            (*this_).m_hFont1 = core::ptr::null_mut();
        }
    
        if !(*this_).m_hFont2.is_null() {
            DeleteObject((*this_).m_hFont2 as _);
            (*this_).m_hFont2 = core::ptr::null_mut();
        }
    
        if !(*this_).m_strCompStr.is_null() {
            cicMemFree((*this_).m_strCompStr as _);
            (*this_).m_strCompStr = core::ptr::null_mut();
        }
    
        (*this_).m_cchCompStr = 0;
    }
    pub unsafe fn CreateDefFrameWnd(&mut self, hwndParent: HWND, hIMC: HIMC) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_pDefCompFrameWindow.is_null() {
            (*this_).m_pDefCompFrameWindow = CDefCompFrameWindow__new_in_0(cpp_alloc_CicNoThrow::<CDefCompFrameWindow>(), hIMC, 0x800000A4);
            if (*this_).m_pDefCompFrameWindow.is_null() {
                return E_OUTOFMEMORY;
            }
            if (*(*this_).m_pDefCompFrameWindow).Initialize() == 0 {
                CDefCompFrameWindow__delete((*this_).m_pDefCompFrameWindow);
                (*this_).m_pDefCompFrameWindow = core::ptr::null_mut();
                return E_FAIL;
            }
    
            (*(*this_).m_pDefCompFrameWindow).Init();
        }
    
        (*(*this_).m_pDefCompFrameWindow).CreateWnd(hwndParent);
        return S_OK;
    }
    pub unsafe fn CreateCompButtonWnd(&mut self, hwndParent: HWND, hIMC: HIMC) -> HRESULT {
        let this_: *mut Self = self;
        let pTLS: *mut TLS = TLS::GetTLS() as *mut TLS;
        if pTLS.is_null() || (*pTLS).NonEACompositionEnabled() == 0 {
            return S_OK;
        }
    
        if IsEALang(0) != 0 {
            if !(*this_).m_pCompButtonFrameWindow.is_null() {
                CCompButtonFrameWindow__delete((*this_).m_pCompButtonFrameWindow);
                (*this_).m_pCompButtonFrameWindow = core::ptr::null_mut();
            }
            return S_OK;
        }
    
        if (*this_).m_pCompButtonFrameWindow.is_null() {
            (*this_).m_pCompButtonFrameWindow = CCompButtonFrameWindow__new_in_0(
                cpp_alloc_CicNoThrow::<CCompButtonFrameWindow>(),
                hIMC,
                0x800000B4,
            );
            if (*this_).m_pCompButtonFrameWindow.is_null() {
                return E_OUTOFMEMORY;
            }
    
            if (*(*this_).m_pCompButtonFrameWindow).Initialize() == 0 {
                if !(*this_).m_pCompButtonFrameWindow.is_null() {
                    CCompButtonFrameWindow__delete((*this_).m_pCompButtonFrameWindow);
                    (*this_).m_pCompButtonFrameWindow = core::ptr::null_mut();
                }
                return E_FAIL;
            }
    
            (*(*this_).m_pCompButtonFrameWindow).Init();
        }
    
        (*(*this_).m_pCompButtonFrameWindow).CreateWnd(hwndParent);
        return S_OK;
    }
    pub unsafe fn CreateCompositionWindow(&mut self, imcLock: &mut CicIMCLock, hwndParent: HWND) -> HRESULT {
        let this_: *mut Self = self;
        if (imcLock.m_hr) < 0 {
            return imcLock.m_hr;
        }
    
        if (IsWindow(hwndParent) == 0) || (*this_).m_bHasCompWnd != 0 {
            return E_FAIL;
        }
    
        let mut iCompStr: INT = 0;
        while iCompStr < 3 {
            let style: DWORD = WS_POPUP | WS_DISABLED;
            let hwndCompStr: HWND = CreateWindowExW(0, CPP_COM_WSTR_62586A0891BF.as_ptr(), core::ptr::null_mut(), style,
                0, 0, 0, 0, hwndParent, core::ptr::null_mut(), g_hInst, core::ptr::null_mut());
            (*this_).m_CompStrs[iCompStr as usize].m_hWnd = hwndCompStr;
            SetWindowLongPtrW(hwndCompStr, GWLP_USERDATA, this_ as LONG_PTR);
            SetWindowLongPtrW(hwndCompStr, UICOMP_GWLP_INDEX, (iCompStr) as _);
            (*this_).m_CompStrs[iCompStr as usize].m_Caret.CreateCaret(hwndCompStr, (*this_).m_CaretSize);
            iCompStr += 1;
        }
    
        let mut hr: HRESULT = (*this_).CreateCompButtonWnd(hwndParent, (imcLock.m_hIMC).cast());
        if hr < 0 {
            (*this_).DestroyCompositionWindow();
            return E_OUTOFMEMORY;
        }
    
        hr = (*this_).CreateDefFrameWnd(hwndParent, (imcLock.m_hIMC).cast());
        if hr < 0 {
            (*this_).DestroyCompositionWindow();
            return E_OUTOFMEMORY;
        }
    
        let style: DWORD = WS_CHILD | WS_DISABLED;
        let hwndCompStr: HWND = CreateWindowExW(WS_EX_CLIENTEDGE, CPP_COM_WSTR_62586A0891BF.as_ptr(), core::ptr::null_mut(), style,
            0, 0, 0, 0, (*(*this_).m_pDefCompFrameWindow).m_hWnd, core::ptr::null_mut(), g_hInst, core::ptr::null_mut());
        if hwndCompStr.is_null() {
            (*this_).DestroyCompositionWindow();
            return E_OUTOFMEMORY;
        }
    
        (*this_).m_CompStrs[3 as usize].m_hWnd = hwndCompStr;
        (*(*this_).m_pDefCompFrameWindow).m_hwndCompStr = hwndCompStr;
        SetWindowLongPtrW(hwndCompStr, GWLP_USERDATA, this_ as LONG_PTR);
        SetWindowLongPtrW(hwndCompStr, UICOMP_GWLP_INDEX, -1);
        (*this_).m_CompStrs[3 as usize].m_Caret.CreateCaret(hwndCompStr, (*this_).m_CaretSize);
        (*this_).m_bHasCompWnd = (TRUE) as i32;
    
        S_OK
    }
    pub unsafe fn DestroyCompositionWindow(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut i: INT = 0;
        while i < 4 {
            let pCompStr: *mut COMPWND = core::ptr::addr_of_mut!((*this_).m_CompStrs[i as usize]);
            (*pCompStr).m_Caret.DestroyCaret();
            if IsWindow((*pCompStr).m_hWnd) != 0 {
                DestroyWindow((*pCompStr).m_hWnd);
                (*pCompStr).m_PolyText.RemoveAll();
            }
            (*pCompStr).m_hWnd = core::ptr::null_mut();
            i += 1;
        }
    
        if !(*this_).m_pCompButtonFrameWindow.is_null() {
            DestroyWindow((*(*this_).m_pCompButtonFrameWindow).m_hWnd);
            CCompButtonFrameWindow__delete((*this_).m_pCompButtonFrameWindow);
            (*this_).m_pCompButtonFrameWindow = core::ptr::null_mut();
        }
    
        if !(*this_).m_pDefCompFrameWindow.is_null() {
            DestroyWindow((*(*this_).m_pDefCompFrameWindow).m_hWnd);
            CDefCompFrameWindow__delete((*this_).m_pDefCompFrameWindow);
            (*this_).m_pDefCompFrameWindow = core::ptr::null_mut();
        }
    
        return 0;
    }
    pub unsafe fn UpdateShowCompWndFlag(&mut self, imcLock: &mut CicIMCLock, pdwCompStrLen: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
        if IsWindow(imcLock.get().hWnd) == 0 {
            return E_FAIL;
        }
    
        let mut compStr = CicIMCCLock__zeroed::<COMPOSITIONSTRING>();
        compStr.__ctor_0(imcLock.get().hCompStr);
        if compStr.m_hr < 0 {
            return compStr.m_hr;
        }
    
        if ((*this_).m_dwUnknown56[0 as usize] & 0x80000000) != 0 && compStr.get().dwCompStrLen != 0 {
            (*this_).m_bHasCompStr = (TRUE) as i32;
        } else {
            (*this_).m_bHasCompStr = (FALSE) as i32;
        }
    
        if !pdwCompStrLen.is_null() {
            *pdwCompStrLen = compStr.get().dwCompStrLen;
        }
    
        return S_OK;
    }
    pub unsafe fn UpdateFont(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        if ((imcLock.m_hr) < 0) {
            return imcLock.m_hr;
        }
    
        if (*this_).m_hFont1 != core::ptr::null_mut() {
            DeleteObject(((*this_).m_hFont1).cast());
        }
        if (*this_).m_hFont2 != core::ptr::null_mut() {
            DeleteObject(((*this_).m_hFont2).cast());
        }
    
        // cicero's island and this one both lay out LOGFONTW; the C++ is a plain struct copy.
        let mut lf: LOGFONTW = core::ptr::read(core::ptr::addr_of!(imcLock.get().lfFont.W) as *const LOGFONTW);
        (*this_).m_hFont2 = CreateFontIndirectW(core::ptr::addr_of!(lf));
    
        lf.lfEscapement = 0;
        lf.lfOrientation = 0;
        let bVertical: BOOL = (lf.lfFaceName[0] == 0x0040u16) as BOOL;
        if bVertical != 0 {
            MoveMemory!(
                lf.lfFaceName.as_mut_ptr() as *mut core::ffi::c_void,
                lf.lfFaceName.as_ptr().add(1) as *const core::ffi::c_void,
                core::mem::size_of::<[WCHAR; 32]>() - core::mem::size_of::<WCHAR>(),
            );
            lf.lfFaceName[(_countof!(lf.lfFaceName) - 1) as usize] = UNICODE_NULL;
        }
        (*this_).m_hFont1 = CreateFontIndirectW(core::ptr::addr_of!(lf));
    
        return S_OK;
    }
    pub unsafe fn UpdateCompositionRect(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return E_NOTIMPL;
    }
    pub unsafe fn GetCompStrBuffer(&mut self, cchStr: INT) -> LPWSTR {
        let this_: *mut Self = self;
        if (*this_).m_strCompStr.is_null() {
            (*this_).m_strCompStr = cicMemAllocClear(((cchStr + 1) as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
            (*this_).m_cchCompStr = cchStr;
        }
        if (*this_).m_cchCompStr < cchStr {
            (*this_).m_strCompStr = cicMemReAlloc(((*this_).m_strCompStr).cast(), (cchStr + 1) as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;
            (*this_).m_cchCompStr = cchStr;
        }
        (*this_).m_strCompStr
    }
    pub unsafe fn GetLevelFromIMC(&mut self, imcLock: &mut CicIMCLock) -> INT {
        let this_: *mut Self = self;
        let dwStyle: DWORD = (*imcLock.get()).cfCompForm.dwStyle;
        if dwStyle == CFS_DEFAULT {
            return 1;
        }
        if (dwStyle & (CFS_FORCE_POSITION | CFS_POINT | CFS_RECT)) == 0 {
            return 0;
        }
    
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect((*imcLock.get()).hWnd, core::ptr::addr_of_mut!(rc));
        if PtInRect(core::ptr::addr_of_mut!(rc), (*imcLock.get()).cfCompForm.ptCurrentPos) == 0 {
            return 1;
        }
    
        let pIC: *mut INPUTCONTEXT = core::ptr::addr_of_mut!(*imcLock.get()) as *mut INPUTCONTEXT; // INPUTCONTEXTDX : INPUTCONTEXT
        if ((*pIC).cfCompForm.dwStyle & CFS_RECT) != 0 &&
            ((*pIC).cfCompForm.rcArea.top == (*pIC).cfCompForm.rcArea.bottom) &&
            ((*pIC).cfCompForm.rcArea.left == (*pIC).cfCompForm.rcArea.right)
        {
            return 1;
        }
    
        return 2;
    }
    pub unsafe fn OnImeStartComposition(&mut self, imcLock: &mut CicIMCLock, hUIWnd: HWND) {
        let this_: *mut Self = self;
        // FIXME
        let _ = this_;
        let _ = imcLock;
        let _ = hUIWnd;
    }
    pub unsafe fn OnImeCompositionUpdate(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_dwUnknown56[0 as usize] |= 0x8000;
        return (*this_).UpdateShowCompWndFlag(imcLock, core::ptr::null_mut());
    }
    pub unsafe fn OnImeEndComposition(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_dwUnknown56[0] = 0;
        return (*this_).DestroyCompositionWindow();
    }
    pub unsafe fn OnImeNotifySetCompositionWindow(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).UpdateCompositionRect(imcLock);
    }
    pub unsafe fn OnImeSetContextAfter(&mut self, imcLock: &mut CicIMCLock) -> HRESULT {
        let this_: *mut Self = self;
        if (((*this_).m_pDefCompFrameWindow.is_null() || IsWindow((*(*this_).m_pDefCompFrameWindow).m_hWnd) == 0) && IsWindow((*this_).m_CompStrs[0 as usize].m_hWnd) == 0)
        {
            (*this_).m_dwUnknown56[0 as usize] &= !0x8000;
            return S_OK;
        }
    
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let mut Level: INT = (*this_).GetLevelFromIMC(imcLock);
        if (Level == 1 || Level == 2)
            && ((*this_).m_dwUnknown56[0 as usize] & 0x80000000) != 0
            && (*this_).m_dwUnknown56[1 as usize] != 0
        {
            let mut dwCompStrLen: DWORD = 0;
            (*this_).UpdateShowCompWndFlag(imcLock, core::ptr::addr_of_mut!(dwCompStrLen));
    
            if Level == 1 {
                ShowWindow(
                    (*(*this_).m_pDefCompFrameWindow).m_hWnd,
                    if (*this_).m_bHasCompStr != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) },
                );
            } else if Level == 2 && (*this_).m_bHasCompStr == 0 && dwCompStrLen != 0 {
                let mut iCompStr: INT = 0;
                while iCompStr < 3 {
                    (*this_).m_CompStrs[iCompStr as usize].m_Caret.HideCaret();
                    ShowWindow((*this_).m_CompStrs[iCompStr as usize].m_Caret.m_hWnd, (SW_HIDE as i32));
                    iCompStr += 1;
                }
            }
        } else {
            ShowWindow((*(*this_).m_pDefCompFrameWindow).m_hWnd, (SW_HIDE as i32));
    
            let mut iCompStr: INT = 0;
            while iCompStr < 3 {
                (*this_).m_CompStrs[iCompStr as usize].m_Caret.HideCaret();
                ShowWindow((*this_).m_CompStrs[iCompStr as usize].m_Caret.m_hWnd, (SW_HIDE as i32));
                iCompStr += 1;
            }
        }
    
        return S_OK;
    }
    pub unsafe fn OnImeSetContext(&mut self, imcLock: &mut CicIMCLock, hUIWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        let _ = (this_, imcLock, hUIWnd, wParam, lParam);
    }
    pub unsafe fn OnPaintTheme(&mut self, wParam: WPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnTimer(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        let iCompStr: INT = GetWindowLongPtrW(hWnd, UICOMP_GWLP_INDEX) as INT;
        if iCompStr == -1 {
            (*this_).m_CompStrs[3 as usize].m_Caret.OnTimer();
        } else {
            (*this_).m_CompStrs[iCompStr as usize].m_Caret.OnTimer();
        }
    }
    pub unsafe fn OnDestroy(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        return (*this_).DestroyCompositionWindow();
    }
    pub unsafe fn SendMessageToUI(imcLock: &mut CicIMCLock, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let hImeWnd: HWND = ImmGetDefaultIMEWnd(core::ptr::null_mut());
        if IsWindow(hImeWnd) == 0 {
            return (TRUE) as BOOL;
        }
        let pTLS: *mut TLS = TLS::GetTLS();
        let ret: LRESULT;
        if !pTLS.is_null() && (*pTLS).m_cWnds > 1 {
            ret = SendMessage(imcLock.get().hWnd, WM_IME_NOTIFY, wParam, lParam);
        } else {
            ret = SendMessage(hImeWnd, WM_IME_NOTIFY, wParam, lParam);
        }
        return (ret == 0) as BOOL;
    }
    pub unsafe fn InquireImeUIWndState(imcLock: &mut CicIMCLock) -> BOOL {
        let mut bValue: BOOL = (FALSE) as i32;
        UIComposition::SendMessageToUI(imcLock, 0x11, core::ptr::addr_of_mut!(bValue) as LPARAM);
        return bValue;
    }
    pub unsafe fn GetImeUIWndTextExtent(imcLock: &mut CicIMCLock, lParam: LPARAM) -> BOOL {
        return (UIComposition::SendMessageToUI(imcLock, 0x14, lParam) == 0) as BOOL;
    }
    pub unsafe extern "system" fn CompWndProc(hWnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if uMsg == WM_CREATE {
            return -1; // FIXME
        }
        return 0;
    }
}

impl UI {
    pub unsafe fn __ctor_0(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hWnd = hWnd;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        UIComposition__delete((*this_).m_pComp);
        (*this_).m_pComp;
    }
    pub unsafe fn _Create(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pComp = UIComposition__new_in_0(cpp_alloc_CicNoThrow::<UIComposition>(), (*this_).m_hWnd);
        if (*this_).m_pComp.is_null() {
            return E_OUTOFMEMORY;
        }
    
        SetWindowLongPtrW((*this_).m_hWnd, UI_GWLP_UI, (this_) as LONG_PTR);
        return S_OK;
    }
    pub unsafe fn _Destroy(&mut self) {
        let this_: *mut Self = self;
        (*(*this_).m_pComp).OnDestroy();
        SetWindowLongPtrW((*this_).m_hWnd, UI_GWLP_UI, 0);
    }
    pub unsafe fn OnCreate(hWnd: HWND) {
        let mut pUI: *mut UI = GetWindowLongPtrW(hWnd, UI_GWLP_UI) as *mut UI;
        if !pUI.is_null() {
            return;
        }
        pUI = UI__new_in_0(cpp_alloc_CicNoThrow::<UI>(), hWnd);
        if !pUI.is_null() {
            (*pUI)._Create();
        }
    }
    pub unsafe fn OnDestroy(hWnd: HWND) {
        let pUI: *mut UI = GetWindowLongPtrW(hWnd, UI_GWLP_UI) as *mut UI;
        if pUI.is_null() {
            return;
        }
    
        (*pUI)._Destroy();
        UI__delete(pUI);
    }
    pub unsafe fn OnImeSetContext(&mut self, imcLock: &mut CicIMCLock, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        (*(*this_).m_pComp).OnImeSetContext(imcLock, (*this_).m_hWnd, wParam, lParam);
    }
}

impl CIMEUIWindowHandler {
    pub unsafe extern "system" fn ImeUIMsImeHandler(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if uMsg == WM_MSIME_MOUSE {
            return (CIMEUIWindowHandler::ImeUIMsImeMouseHandler(hWnd, wParam, lParam)) as LRESULT;
        }
        if uMsg == WM_MSIME_MODEBIAS {
            return CIMEUIWindowHandler::ImeUIMsImeModeBiasHandler(hWnd, wParam, lParam);
        }
        if uMsg == WM_MSIME_RECONVERTREQUEST {
            return CIMEUIWindowHandler::ImeUIMsImeReconvertRequest(hWnd, wParam, lParam);
        }
        if uMsg == WM_MSIME_SERVICE {
            let pTLS: *mut TLS = TLS::GetTLS();
            if !pTLS.is_null() && !(*pTLS).m_pProfile.is_null() {
                let mut LangID: LANGID = 0;
                (*(*pTLS).m_pProfile).GetLangId(core::ptr::addr_of_mut!(LangID));
                if (PRIMARYLANGID!(LangID)) as u32 == LANG_KOREAN {
                    return (FALSE) as LRESULT;
                }
            }
            return (TRUE) as LRESULT;
        }
        return 0;
    }
    pub unsafe extern "system" fn ImeUIMsImeMouseHandler(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        if (wParam as BYTE) == 0xFF {
            return (TRUE) as HRESULT;
        }
    
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((lParam as HIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return (FALSE) as HRESULT;
        }
    
        let mut hr: HRESULT = E_FAIL;
        let mut pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() {
            let mut keys: UINT = 0;
            if (wParam & (MK_LBUTTON as WPARAM)) != 0 {
                keys |= 0x1;
            }
            if (wParam & (MK_SHIFT as WPARAM)) != 0 {
                keys |= 0x10;
            }
            if (wParam & (MK_RBUTTON as WPARAM)) != 0 {
                keys |= 0x2;
            }
            if (wParam & (MK_MBUTTON as WPARAM)) != 0 {
                keys |= 0x780000;
            } else if (wParam & (MK_ALT as WPARAM)) != 0 {
                keys |= 0xFF880000;
            }
            hr = (*pCicIC).MsImeMouseHandler(
                HIWORD!(wParam as DWORD) as DWORD,
                HIBYTE!(LOWORD!(wParam as DWORD) as WORD) as DWORD,
                keys,
                &mut imcLock,
            );
        }
    
        return hr;
    }
    pub unsafe extern "system" fn ImeUIMsImeModeBiasHandler(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if wParam == 0 {
            return (TRUE) as LRESULT;
        }
    
        let mut imcLock: CicIMCLock = CicIMCLock__make_0((lParam as HIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr as LRESULT;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return (FALSE) as LRESULT;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if pCicIC.is_null() {
            return (FALSE) as LRESULT;
        }
    
        if wParam == 1 {
            if lParam == 0 || lParam == 1 || lParam == 4 || lParam == 0x10000 {
                let guid: GUID = (*pCicIC).m_ModeBias.ConvertModeBias__0((lParam as DWORD) as LONG);
                (*pCicIC).m_ModeBias.SetModeBias(&guid);
    
                (*pCicIC).m_dwUnknown7[2 as usize] |= 1;
                (*(*pCicIC).m_pContextOwnerServices).AddRef();
                (*(*pCicIC).m_pContextOwnerServices).OnAttributeChange(&GUID_PROP_MODEBIAS);
                (*(*pCicIC).m_pContextOwnerServices).Release();
                return (TRUE) as LRESULT;
            }
        } else if wParam == 2 {
            return ((*pCicIC).m_ModeBias.ConvertModeBias__1(&(*pCicIC).m_ModeBias.m_guid)) as LRESULT;
        }
    
        return (FALSE) as LRESULT;
    }
    pub unsafe extern "system" fn ImeUIMsImeReconvertRequest(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if wParam == 0x10000000 {
            return (TRUE) as LRESULT;
        }
    
        let hIMC: HIMC = GetWindowLongPtrW(hWnd, UI_GWLP_HIMC) as HIMC;
        let mut imcLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return (FALSE) as LRESULT;
        }
    
        let mut imeContext = CicIMCCLock__zeroed::<CTFIMECONTEXT>();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return (FALSE) as LRESULT;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        let pTLS: *mut TLS = TLS::GetTLS();
        if pCicIC.is_null() || pTLS.is_null() {
            return (FALSE) as LRESULT;
        }
    
        let pThreadMgr = (*pTLS).m_pThreadMgr;
        let pProfile = (*pTLS).m_pProfile;
        if pThreadMgr.is_null() || pProfile.is_null() {
            return (FALSE) as LRESULT;
        }
    
        let mut uCodePage: UINT = 0;
        (*pProfile).GetCodePageA(core::ptr::addr_of_mut!(uCodePage));
        (*pCicIC).SetupReconvertString(&mut imcLock, pThreadMgr, uCodePage, WM_MSIME_RECONVERT, (FALSE) as BOOL);
        (*pCicIC).EndReconvertString(&mut imcLock);
        return (TRUE) as LRESULT;
    }
    pub unsafe extern "system" fn ImeUIWndProcWorker(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let pTLS: *mut TLS = TLS::GetTLS();
        if !pTLS.is_null() && ((*pTLS).m_dwSystemInfoFlags & IME_SYSINFO_WINLOGON) != 0 {
            if uMsg == WM_CREATE {
                return -1;
            }
            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
    
        match uMsg {
            WM_CREATE => {
                UI::OnCreate(hWnd);
            }
            WM_DESTROY | WM_ENDSESSION => {
                UI::OnDestroy(hWnd);
            }
            WM_IME_STARTCOMPOSITION | WM_IME_COMPOSITION | WM_IME_ENDCOMPOSITION | WM_IME_SETCONTEXT | WM_IME_NOTIFY | WM_IME_SELECT | WM_TIMER => {
                let hIMC: HIMC = GetWindowLongPtrW(hWnd, UI_GWLP_HIMC) as HIMC;
                let pUI: *mut UI = GetWindowLongPtrW(hWnd, UI_GWLP_UI) as *mut UI;
                let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
                imcLock.__ctor_0((hIMC).cast());
                match uMsg {
                    WM_IME_STARTCOMPOSITION => {
                        (*(*pUI).m_pComp).OnImeStartComposition(&mut imcLock, (*pUI).m_hWnd);
                    }
                    WM_IME_COMPOSITION => {
                        if (lParam & GCS_COMPSTR as LPARAM) != 0 {
                            (*(*pUI).m_pComp).OnImeCompositionUpdate(&mut imcLock);
                            SetTimer(hWnd, 0, 10, None);
                            (*(*pUI).m_pComp).m_bInComposition = (TRUE) as i32;
                        }
                    }
                    WM_IME_ENDCOMPOSITION => {
                        KillTimer(hWnd, 0);
                        (*(*pUI).m_pComp).OnImeEndComposition();
                    }
                    WM_IME_SETCONTEXT => {
                        (*pUI).OnImeSetContext(&mut imcLock, wParam, lParam);
                        KillTimer(hWnd, 1);
                        SetTimer(hWnd, 1, 300, None);
                    }
                    WM_TIMER => {
                        match wParam {
                            0 => {
                                KillTimer(hWnd, wParam);
                                (*(*pUI).m_pComp).m_bInComposition = (FALSE) as i32;
                                (*(*pUI).m_pComp).OnImeNotifySetCompositionWindow(&mut imcLock);
                            }
                            1 => {
                                KillTimer(hWnd, wParam);
                                (*(*pUI).m_pComp).OnImeSetContextAfter(&mut imcLock);
                            }
                            2 => {
                                KillTimer(hWnd, wParam);
                                CIMEUIWindowHandler::ImeUIDelayedReconvertFuncCall(hWnd);
                            }
                            _ => {}
                        }
                    }
                    WM_IME_NOTIFY | WM_IME_SELECT | _ => {
                        (*(*pUI).m_pComp).OnPaintTheme(wParam);
                    }
                }
            }
            _ => {
                if IsMsImeMessage(uMsg) != 0 {
                    return CIMEUIWindowHandler::ImeUIMsImeHandler(hWnd, uMsg, wParam, lParam);
                }
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
    
        return 0;
    }
    pub unsafe extern "system" fn ImeUIDelayedReconvertFuncCall(hWnd: HWND) -> HRESULT {
        let hIMC: HIMC = GetWindowLongPtrW(hWnd, UI_GWLP_HIMC) as HIMC;
        let mut imcLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if (imcLock.m_hr) < 0 {
            return imcLock.m_hr;
        }
        let mut imeContext = CicIMCCLock__zeroed::<CTFIMECONTEXT>();
        imeContext.__ctor_0((imcLock.get()).hCtfImeContext);
        if (imeContext.m_hr) < 0 {
            return imeContext.m_hr;
        }
        let pCicIC: *mut CicInputContext = (imeContext.get()).m_pCicIC;
        if !pCicIC.is_null() {
            (*pCicIC).DelayedReconvertFuncCall(&mut imcLock);
        }
        return S_OK;
    }
    pub unsafe extern "system" fn ImeUIOnLayoutChange(lParam: LPARAM) -> HRESULT {
        let mut imcLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((lParam as HIMC).cast());
        if (imcLock.m_hr < 0) {
            return S_OK;
        }
    
        let mut imeContext: CicIMCCLock<CTFIMECONTEXT> = CicIMCCLock__zeroed();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if (imeContext.m_hr < 0) {
            return S_OK;
        }
    
        let mut pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() {
            let mut pContextOwnerServices: *mut ITfContextOwnerServices = (*pCicIC).m_pContextOwnerServices;
            (*pContextOwnerServices).AddRef();
            (*pContextOwnerServices).OnLayoutChange();
            (*pContextOwnerServices).Release();
        }
    
        return S_OK;
    }
    pub unsafe extern "system" fn ImeUIPrivateHandler(uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let _ = uMsg;
        let mut imcLock = CicIMCLock__make_0((lParam as HIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr;
        }
    
        let mut imeContext = CicIMCCLock__make_0::<CTFIMECONTEXT>(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr;
        }
    
        let mut pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() && wParam == 0x10 {
            (*pCicIC).EscbClearDocFeedBuffer(&mut imcLock, (TRUE) as BOOL);
        }
    
        return S_OK;
    }
    pub unsafe extern "system" fn ImeUINotifyHandler(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let hIMC: HIMC = GetWindowLongPtrW(hWnd, UI_GWLP_HIMC) as HIMC;
        let mut imcLock: CicIMCLock = CicIMCLock__zeroed();
        imcLock.__ctor_0((hIMC).cast());
        if imcLock.m_hr < 0 {
            return imcLock.m_hr as LRESULT;
        }
        let mut imeContext = CicIMCCLock__zeroed::<CTFIMECONTEXT>();
        imeContext.__ctor_0(imcLock.get().hCtfImeContext);
        if imeContext.m_hr < 0 {
            return imeContext.m_hr as LRESULT;
        }
    
        let pCicIC: *mut CicInputContext = imeContext.get().m_pCicIC;
        if !pCicIC.is_null() {
            // C's switch compares the whole WPARAM; the IMN_* constants widen to it.
            const IMN_CLOSECANDIDATE_W: WPARAM = IMN_CLOSECANDIDATE as WPARAM;
            const IMN_OPENCANDIDATE_W: WPARAM = IMN_OPENCANDIDATE as WPARAM;
            const IMN_SETCANDIDATEPOS_W: WPARAM = IMN_SETCANDIDATEPOS as WPARAM;
            const IMN_SETCOMPOSITIONFONT_W: WPARAM = IMN_SETCOMPOSITIONFONT as WPARAM;
            const IMN_SETCOMPOSITIONWINDOW_W: WPARAM = IMN_SETCOMPOSITIONWINDOW as WPARAM;
            match wParam {
                IMN_CLOSECANDIDATE_W => {
                    (*pCicIC).m_bCandidateOpen = (FALSE) as i32;
                    let hImeWnd: HWND = ImmGetDefaultIMEWnd(core::ptr::null_mut());
                    if IsWindow(hImeWnd) != 0 {
                        PostMessage(hImeWnd, WM_IME_NOTIFY, 0x10, hIMC as LPARAM);
                    }
                }
                IMN_OPENCANDIDATE_W => {
                    (*pCicIC).m_bCandidateOpen = (TRUE) as i32;
                    (*pCicIC).ClearPrevCandidatePos();
                }
                IMN_SETCANDIDATEPOS_W => {
                    // FIXME
                }
                IMN_SETCOMPOSITIONFONT_W => {
                    // FIXME
                }
                IMN_SETCOMPOSITIONWINDOW_W => {
                    // FIXME
                }
                0xF => {
                    CIMEUIWindowHandler::ImeUIOnLayoutChange(lParam);
                }
                0x10 => {
                    CIMEUIWindowHandler::ImeUIPrivateHandler(uMsg, 0x10, lParam);
                }
                0x13 => {
                    CIMEUIWindowHandler::ImeUIOnLayoutChange(lParam);
                }
                0x14 => {
                    // FIXME
                }
                0x16 => {
                    SetTimer(hWnd, 2, 100, None);
                }
                0x17 => {
                    return hWnd as LRESULT;
                }
                0x10D => {
                    // FIXME
                }
                0x10E => {
                    (*pCicIC).m_dwQueryPos = 0;
                }
                _ => {}
            }
        }
    
        return 0 as LRESULT;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
