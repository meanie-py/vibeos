use crate::magnify::settings::LoadSettings;
use crate::magnify::settings::SaveSettings;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub uTimeout: UINT,
    pub uVersion: UINT,
}

#[no_mangle]
pub static mut szWindowClass: [TCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [TCHAR; 10]>(*b"MAGNIFIER\0") };

#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hMainWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut szTitle: [TCHAR; 100] = [0; 100];

#[no_mangle]
pub static mut lastTicks: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut hDesktopWindow: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut nid: NOTIFYICONDATA = _NOTIFYICONDATAA {
    cbSize: 0,
    hWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    uID: 0,
    uFlags: 0,
    uCallbackMessage: 0,
    hIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    szTip: [0; 128],
    dwState: 0,
    dwStateMask: 0,
    szInfo: [0; 256],
    __bindgen_anon_1: _NOTIFYICONDATAA__bindgen_ty_1 { uTimeout: 0 },
    szInfoTitle: [0; 64],
    dwInfoFlags: 0,
    guidItem: _GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    },
};

#[no_mangle]
pub static mut notifyIcon: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

#[no_mangle]
pub static mut notifyMenu: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut hOptionsDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut bOptionsDialog: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bRecreateOffscreenDC: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut sourceWidth: LONG = 0 as LONG;

#[no_mangle]
pub static mut sourceHeight: LONG = 0 as LONG;

#[no_mangle]
pub static mut hdcOffscreen: HDC = ::core::ptr::null::<HDC__>() as *mut HDC__;

#[no_mangle]
pub static mut hbmpOffscreen: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

#[no_mangle]
pub static mut hbmpOld: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut ptDragOffset: POINT = tagPOINT { x: 0, y: 0 };

#[no_mangle]
pub static mut nearEdge: INT = 0;

#[no_mangle]
pub static mut cp: POINT = tagPOINT { x: 0, y: 0 };

#[no_mangle]
pub static mut pMouse: POINT = tagPOINT { x: 0, y: 0 };

#[no_mangle]
pub static mut pCaret: POINT = tagPOINT { x: 0, y: 0 };

#[no_mangle]
pub static mut pFocus: POINT = tagPOINT { x: 0, y: 0 };

#[no_mangle]
pub static mut pCaretWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut pFocusWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

pub const APPMSG_NOTIFYICON: u32 = 0x8001; // WM_APP+1
pub const APPMSG_APPBAR: u32 = 0x8002; // WM_APP+2

pub const MAX_LOADSTRING: usize = 100;

pub const TIMER_SPEED: usize = 1;

pub const REPAINT_SPEED: u32 = 100;

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPSTR,
    nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut msg: MSG = core::mem::zeroed();
    let mut hAccelTable: HACCEL = core::ptr::null_mut();
    let mut iccex: INITCOMMONCONTROLSEX = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(lpCmdLine);

    let switchVal = GetUserDefaultUILanguage() as ::core::ffi::c_int;
    if switchVal == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) as ::core::ffi::c_int {
        SetProcessDefaultLayout(LAYOUT_RTL as DWORD);
    }

    /* Initialize global strings */
    LoadString(
        hInstance,
        IDS_APP_TITLE as UINT,
        core::ptr::addr_of_mut!(szTitle) as LPSTR,
        (MAX_LOADSTRING as i32),
    );
    MyRegisterClass(hInstance);

    /* Perform application initialization */
    if InitInstance(hInstance, nCmdShow) == 0 {
        return (FALSE) as i32;
    }

    hAccelTable = LoadAccelerators(
        hInstance,
        MAKEINTRESOURCE!(IDC_MAGNIFIER) as LPCSTR,
    );

    /* Main message loop */
    while GetMessage(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0 as UINT,
        0 as UINT,
    ) != 0
    {
        if TranslateAccelerator(hMainWnd, hAccelTable, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    /* Load the common controls */
    iccex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccex.dwICC = (ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccex));

    SelectObject(hdcOffscreen, hbmpOld as HGDIOBJ);
    DeleteObject(hbmpOffscreen as HGDIOBJ);
    DeleteDC(hdcOffscreen);

    return msg.wParam as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn MyRegisterClass(hInstance: HINSTANCE) -> ATOM {
    let mut wc: WNDCLASS = core::mem::zeroed();

    wc.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    wc.lpfnWndProc = Some(WndProc);
    wc.cbClsExtra = 0 as i32;
    wc.cbWndExtra = 0 as i32;
    wc.hInstance = hInstance;
    wc.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_ICON) as LPCSTR);
    wc.hCursor = LoadCursor(core::ptr::null_mut(), MAKEINTRESOURCE!(IDC_ARROW) as LPCSTR);
    wc.hbrBackground = (COLOR_WINDOW + 1) as usize as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = core::ptr::addr_of!(szWindowClass) as *const TCHAR as LPCSTR;

    return RegisterClass(core::ptr::addr_of_mut!(wc));
}
#[no_mangle]
pub unsafe extern "C" fn DoAppBarStuff(mode: DWORD) {
    let mut uState: UINT = 0;
    let mut data: APPBARDATA = core::mem::zeroed();
    data.cbSize = core::mem::size_of::<APPBARDATA>() as DWORD;
    data.hWnd = hMainWnd;
    data.uCallbackMessage = APPMSG_APPBAR as UINT;

    if mode == ABM_NEW as DWORD || mode == ABM_SETPOS as DWORD {
        let mut hWndOrder: HWND = HWND_BOTTOM;
        let mut rcw: core::ffi::c_int = 0;
        let mut rch: core::ffi::c_int = 0;
        let mut rcWorkArea: RECT = core::mem::zeroed();
        SystemParametersInfo(
            SPI_GETWORKAREA as UINT,
            0 as UINT,
            core::ptr::addr_of_mut!(rcWorkArea) as PVOID,
            0 as UINT,
        );

        if mode == ABM_NEW as DWORD {
            SHAppBarMessage(ABM_NEW as DWORD, core::ptr::addr_of_mut!(data));

            match (AppBarConfig.uEdge) as u32 {
                ABE_LEFT => {
                    data.rc.top = rcWorkArea.top;
                    data.rc.bottom = rcWorkArea.bottom;
                    data.rc.left = rcWorkArea.left;
                    data.rc.right = data.rc.left + AppBarConfig.appBarSizes.left;
                }
                ABE_TOP => {
                    data.rc.left = rcWorkArea.left;
                    data.rc.right = rcWorkArea.right;
                    data.rc.top = rcWorkArea.top;
                    data.rc.bottom = data.rc.top + AppBarConfig.appBarSizes.top;
                }
                ABE_RIGHT => {
                    data.rc.top = rcWorkArea.top;
                    data.rc.bottom = rcWorkArea.bottom;
                    data.rc.right = rcWorkArea.left;
                    data.rc.left = data.rc.right - AppBarConfig.appBarSizes.right;
                }
                ABE_BOTTOM => {
                    data.rc.left = rcWorkArea.left;
                    data.rc.right = rcWorkArea.right;
                    data.rc.bottom = rcWorkArea.bottom;
                    data.rc.top = data.rc.bottom - AppBarConfig.appBarSizes.bottom;
                }
                _ => {}
            }
        } else {
            GetWindowRect(hMainWnd, core::ptr::addr_of_mut!(data.rc));
        }

        data.uEdge = AppBarConfig.uEdge as UINT;
        uState = SHAppBarMessage(ABM_QUERYPOS as DWORD, core::ptr::addr_of_mut!(data)) as UINT;
        uState = SHAppBarMessage(ABM_SETPOS as DWORD, core::ptr::addr_of_mut!(data)) as UINT;

        rcw = (data.rc.right - data.rc.left) as core::ffi::c_int;
        rch = (data.rc.bottom - data.rc.top) as core::ffi::c_int;

        uState = SHAppBarMessage(ABM_GETSTATE as DWORD, core::ptr::addr_of_mut!(data)) as UINT;
        if (uState & (ABS_ALWAYSONTOP as UINT)) != 0 {
            hWndOrder = HWND_TOPMOST;
        }

        SetWindowPos(
            hMainWnd,
            hWndOrder,
            data.rc.left as core::ffi::c_int,
            data.rc.top as core::ffi::c_int,
            rcw,
            rch,
            (SWP_SHOWWINDOW | SWP_NOCOPYBITS) as UINT,
        );
    } else if mode == ABM_GETSTATE as DWORD {
        let mut hWndOrder_0: HWND = HWND_BOTTOM;
        uState = SHAppBarMessage(ABM_GETSTATE as DWORD, core::ptr::addr_of_mut!(data)) as UINT;
        if (uState & (ABS_ALWAYSONTOP as UINT)) != 0 {
            hWndOrder_0 = HWND_TOPMOST;
        }
        SetWindowPos(
            hMainWnd,
            hWndOrder_0,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            (SWP_NOMOVE | SWP_NOSIZE) as UINT,
        );
    } else if mode == ABM_ACTIVATE as DWORD {
        SHAppBarMessage(ABM_ACTIVATE as DWORD, core::ptr::addr_of_mut!(data));
    } else if mode == ABM_WINDOWPOSCHANGED as DWORD {
        SHAppBarMessage(ABM_WINDOWPOSCHANGED as DWORD, core::ptr::addr_of_mut!(data));
    } else if mode == ABM_REMOVE as DWORD {
        SHAppBarMessage(ABM_REMOVE as DWORD, core::ptr::addr_of_mut!(data));
    }
}
#[no_mangle]
pub unsafe extern "C" fn AttachAppBar(uEdge: INT) {
    if AppBarConfig.uEdge == uEdge {
        return;
    }

    if AppBarConfig.uEdge < 0 && uEdge >= 0 {
        SetWindowLongPtr(
            hMainWnd,
            GWL_STYLE,
            GetWindowLongPtr(hMainWnd, GWL_STYLE) & ((!WS_CAPTION)) as LONG_PTR,
        );
    } else if uEdge < 0 && AppBarConfig.uEdge >= 0 {
        SetWindowLongPtr(
            hMainWnd,
            GWL_STYLE,
            GetWindowLongPtr(hMainWnd, GWL_STYLE) | (WS_CAPTION) as LONG_PTR,
        );
        SetWindowPos(
            hMainWnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
        );
    }

    if AppBarConfig.uEdge >= 0 {
        DoAppBarStuff(ABM_REMOVE);
    }

    if uEdge >= 0 {
        AppBarConfig.uEdge = uEdge;
        DoAppBarStuff(ABM_NEW);
    } else {
        let mut rc: RECT = AppBarConfig.rcFloating;
        SetWindowPos(
            hMainWnd,
            HWND_TOPMOST,
            rc.left,
            rc.top,
            rc.right - rc.left,
            rc.bottom - rc.top,
            0,
        );
    }

    AppBarConfig.uEdge = uEdge;
}
#[no_mangle]
pub unsafe extern "C" fn InitInstance(hInstance: HINSTANCE, nCmdShow: core::ffi::c_int) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();
    let mut exStyles: DWORD = WS_EX_TOOLWINDOW | WS_EX_CONTROLPARENT;
    let mut dwStyles: DWORD = WS_SIZEBOX | WS_SYSMENU | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_POPUP;

    /* Load settings from registry */
    LoadSettings();

    rc = AppBarConfig.rcFloating;

    hInst = hInstance;

    if AppBarConfig.uEdge < 0 {
        dwStyles |= WS_CAPTION;
        exStyles |= WS_EX_TOPMOST;
    }

    /* Create the Window */
    hMainWnd = CreateWindowEx(
        exStyles,
        core::ptr::addr_of!(szWindowClass) as LPCSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut TCHAR as LPCSTR,
        dwStyles,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hMainWnd.is_null() {
        return (FALSE) as BOOL;
    }

    if AppBarConfig.uEdge >= 0 {
        DoAppBarStuff(ABM_NEW);
    } else {
        SetWindowPos(
            hMainWnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
    }

    // In Windows 2003's Magnifier, the "Start Minimized" setting
    // refers exclusively to the options dialog, not the main window itself.
    hOptionsDialog = CreateDialog!(hInstance, MAKEINTRESOURCE!(IDD_DIALOGOPTIONS), hMainWnd, Some(OptionsProc));
    ShowWindow(hOptionsDialog, if bStartMinimized != 0 { (SW_HIDE) as i32 } else { (SW_SHOW) as i32 });

    if bShowWarning != 0 {
        DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_WARNINGDIALOG), hMainWnd, Some(WarningProc));
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Refresh() {
    if IsIconic(hMainWnd) == 0 {
        /* Invalidate the client area forcing a WM_PAINT message */
        InvalidateRgn(hMainWnd, core::ptr::null_mut(), (TRUE) as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetBestOverlapWithMonitors(rect: LPRECT) {
    let mut rcLeft: i32 = 0;
    let mut rcTop: i32 = 0;
    let mut rcWidth: i32 = 0;
    let mut rcHeight: i32 = 0;
    let mut rcMon: RECT = core::mem::zeroed();
    let mut hMon: HMONITOR = MonitorFromRect(rect as LPCRECT, MONITOR_DEFAULTTONEAREST as DWORD);
    let mut info: MONITORINFO = core::mem::zeroed();
    info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;

    GetMonitorInfo(hMon, &raw mut info);

    rcMon = info.rcMonitor;

    rcLeft = (*rect).left as i32;
    rcTop = (*rect).top as i32;
    rcWidth = ((*rect).right - (*rect).left) as i32;
    rcHeight = ((*rect).bottom - (*rect).top) as i32;

    if (rcLeft as LONG) < rcMon.left {
        rcLeft = rcMon.left as i32;
    }

    if (rcTop as LONG) < rcMon.top {
        rcTop = rcMon.top as i32;
    }

    if (rcLeft as LONG) > (rcMon.right - rcWidth as LONG) {
        rcLeft = (rcMon.right - rcWidth as LONG) as i32;
    }

    if (rcTop as LONG) > (rcMon.bottom - rcHeight as LONG) {
        rcTop = (rcMon.bottom - rcHeight as LONG) as i32;
    }

    OffsetRect(
        rect,
        (rcLeft as LONG - (*rect).left) as i32,
        (rcTop as LONG - (*rect).top) as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Draw(aDc: HDC) {
    let mut desktopHdc: HDC = core::ptr::null_mut();

    let mut sourceRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut intersectedRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut targetRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut appRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rop: DWORD = SRCCOPY as DWORD;
    let mut cinfo: CURSORINFO = CURSORINFO {
        cbSize: 0,
        flags: 0,
        hCursor: core::ptr::null_mut(),
        ptScreenPos: POINT { x: 0, y: 0 },
    };
    let mut iinfo: ICONINFO = ICONINFO {
        fIcon: 0,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: core::ptr::null_mut(),
        hbmColor: core::ptr::null_mut(),
    };

    let mut AppWidth: core::ffi::c_int = 0;
    let mut AppHeight: core::ffi::c_int = 0;

    if bInvertColors != 0 {
        rop = NOTSRCCOPY as DWORD;
    }

    desktopHdc = GetDC(core::ptr::null_mut());

    GetClientRect(hMainWnd, core::ptr::addr_of_mut!(appRect));
    AppWidth = (appRect.right - appRect.left) as core::ffi::c_int;
    AppHeight = (appRect.bottom - appRect.top) as core::ffi::c_int;

    ZeroMemory!(
        core::ptr::addr_of_mut!(cinfo) as *mut core::ffi::c_void,
        core::mem::size_of::<CURSORINFO>() as size_t,
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(iinfo) as *mut core::ffi::c_void,
        core::mem::size_of::<ICONINFO>() as size_t,
    );
    cinfo.cbSize = core::mem::size_of::<CURSORINFO>() as DWORD;
    GetCursorInfo(core::ptr::addr_of_mut!(cinfo));
    GetIconInfo(cinfo.hCursor as HICON, core::ptr::addr_of_mut!(iinfo));

    targetRect = appRect;
    ClientToScreen(hMainWnd, core::ptr::addr_of_mut!(targetRect.left) as *mut POINT);
    ClientToScreen(hMainWnd, core::ptr::addr_of_mut!(targetRect.right) as *mut POINT);

    if bRecreateOffscreenDC != 0 || hdcOffscreen.is_null() {
        bRecreateOffscreenDC = FALSE as BOOL;

        if !hdcOffscreen.is_null() {
            SelectObject(hdcOffscreen, hbmpOld as HGDIOBJ);
            DeleteObject(hbmpOffscreen as HGDIOBJ);
            DeleteDC(hdcOffscreen);
        }

        sourceWidth = (AppWidth as UINT).wrapping_div(uiZoom) as LONG;
        sourceHeight = (AppHeight as UINT).wrapping_div(uiZoom) as LONG;

        /* Create a memory DC compatible with client area DC */
        hdcOffscreen = CreateCompatibleDC(desktopHdc);

        /* Create a bitmap compatible with the client area DC */
        hbmpOffscreen = CreateCompatibleBitmap(
            desktopHdc,
            sourceWidth as INT,
            sourceHeight as INT,
        );

        /* Select our bitmap in memory DC and save the old one */
        hbmpOld = SelectObject(hdcOffscreen, hbmpOffscreen as HGDIOBJ) as HANDLE;
    }

    GetWindowRect(hDesktopWindow, core::ptr::addr_of_mut!(sourceRect));
    sourceRect.left = cp.x - (sourceWidth / 2);
    sourceRect.top = cp.y - (sourceHeight / 2);
    sourceRect.right = sourceRect.left + sourceWidth;
    sourceRect.bottom = sourceRect.top + sourceHeight;

    GetBestOverlapWithMonitors(core::ptr::addr_of_mut!(sourceRect));

    /* Paint the screen bitmap to our in memory DC */
    BitBlt(
        hdcOffscreen,
        0,
        0,
        sourceWidth,
        sourceHeight,
        desktopHdc,
        sourceRect.left,
        sourceRect.top,
        rop,
    );

    if IntersectRect(
        core::ptr::addr_of_mut!(intersectedRect),
        core::ptr::addr_of_mut!(sourceRect),
        core::ptr::addr_of_mut!(targetRect),
    ) != 0
    {
        OffsetRect(
            core::ptr::addr_of_mut!(intersectedRect),
            -sourceRect.left,
            -sourceRect.top,
        );
        FillRect(
            hdcOffscreen,
            core::ptr::addr_of_mut!(intersectedRect),
            GetStockObject((DC_BRUSH as i32)) as HBRUSH,
        );
    }

    /* Draw the mouse pointer in the right position */
    DrawIcon(
        hdcOffscreen,
        (pMouse.x as DWORD)
            .wrapping_sub(iinfo.xHotspot)
            .wrapping_sub(sourceRect.left as DWORD) as core::ffi::c_int,
        (pMouse.y as DWORD)
            .wrapping_sub(iinfo.yHotspot)
            .wrapping_sub(sourceRect.top as DWORD) as core::ffi::c_int,
        cinfo.hCursor as HICON,
    );

    /* Blast the stretched image from memory DC to window DC */
    StretchBlt(
        aDc,
        0,
        0,
        AppWidth,
        AppHeight,
        hdcOffscreen,
        0,
        0,
        sourceWidth,
        sourceHeight,
        SRCCOPY | NOMIRRORBITMAP,
    );

    /* Cleanup */
    if !iinfo.hbmMask.is_null() {
        DeleteObject(iinfo.hbmMask as HGDIOBJ);
    }
    if !iinfo.hbmColor.is_null() {
        DeleteObject(iinfo.hbmColor as HGDIOBJ);
    }
    ReleaseDC(hDesktopWindow, desktopHdc);
}
#[no_mangle]
pub unsafe extern "C" fn HandleNotifyIconMessage(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut pt: POINT = core::mem::zeroed();
    if (lParam) as u32 == (WM_LBUTTONUP) as _ {
        PostMessage(hMainWnd, WM_COMMAND, (IDM_OPTIONS as WPARAM), 0);
    } else if (lParam) as u32 == (WM_RBUTTONUP) as _ {
        GetCursorPos(core::ptr::addr_of_mut!(pt));
        TrackPopupMenu(notifyMenu, 0, pt.x, pt.y, 0, hWnd, core::ptr::null::<RECT>());
    }
}
#[no_mangle]
pub unsafe extern "system" fn WndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut wmId: ::core::ffi::c_int = 0;
    let mut tail: bool = false;
    match message {
        WM_TIMER => {
            let mut hasMoved: BOOL = FALSE as BOOL;

            let mut guiInfo: GUITHREADINFO = core::mem::zeroed();
            guiInfo.cbSize = ::core::mem::size_of::<GUITHREADINFO>() as DWORD;

            GetGUIThreadInfo(0 as DWORD, &raw mut guiInfo);

            if bFollowMouse != 0 {
                let mut pNewMouse: POINT = core::mem::zeroed();

                //Get current mouse position
                GetCursorPos(&raw mut pNewMouse);

                //If mouse has moved ...
                if !PointsAreEqual!(pMouse, pNewMouse) {
                    //Update to new position
                    pMouse = pNewMouse;
                    cp = pNewMouse;
                    hasMoved = TRUE as BOOL;
                }
            }

            if guiInfo.hwndActive != hMainWnd {
                if bFollowCaret != 0 {
                    if !guiInfo.hwndCaret.is_null() {
                        let mut ptCaret: POINT = core::mem::zeroed();
                        ptCaret.x = (guiInfo.rcCaret.left + guiInfo.rcCaret.right) / 2 as LONG;
                        ptCaret.y = (guiInfo.rcCaret.top + guiInfo.rcCaret.bottom) / 2 as LONG;

                        if (pCaretWnd != guiInfo.hwndCaret) || !PointsAreEqual!(pCaret, ptCaret) {
                            //Update to new position
                            pCaret = ptCaret;
                            pCaretWnd = guiInfo.hwndCaret;
                            if hasMoved == 0 {
                                ClientToScreen(guiInfo.hwndCaret, &raw mut ptCaret as LPPOINT);
                                cp = ptCaret;
                                hasMoved = TRUE as BOOL;
                            }
                        }
                    } else {
                        pCaretWnd = core::ptr::null_mut();
                    }
                }

                if bFollowFocus != 0 {
                    if !guiInfo.hwndFocus.is_null() && guiInfo.hwndCaret.is_null() {
                        let mut ptFocus: POINT = core::mem::zeroed();
                        let mut activeRect: RECT = core::mem::zeroed();

                        //Get current control focus
                        GetWindowRect(guiInfo.hwndFocus, &raw mut activeRect);
                        ptFocus.x = (activeRect.left + activeRect.right) / 2 as LONG;
                        ptFocus.y = (activeRect.top + activeRect.bottom) / 2 as LONG;

                        if (guiInfo.hwndFocus != pFocusWnd) || !PointsAreEqual!(pFocus, ptFocus) {
                            //Update to new position
                            pFocus = ptFocus;
                            pFocusWnd = guiInfo.hwndFocus;
                            if hasMoved == 0 {
                                cp = ptFocus;
                                hasMoved = TRUE as BOOL;
                            }
                        }
                    } else {
                        pFocusWnd = core::ptr::null_mut();
                    }
                }
            }

            if hasMoved == 0 {
                let mut newTicks: DWORD = GetTickCount();
                let mut elapsed: DWORD = newTicks.wrapping_sub(lastTicks);
                if elapsed > REPAINT_SPEED as DWORD {
                    hasMoved = TRUE as BOOL;
                }
            }

            if hasMoved != 0 {
                lastTicks = GetTickCount();
                Refresh();
            }

            return 0 as LRESULT;
        }

        WM_COMMAND => {
            wmId = LOWORD!(wParam) as ::core::ffi::c_int;
            /* Parse the menu selections */
            match (wmId) as u32 {
                IDM_OPTIONS => {
                    ShowWindow(
                        hOptionsDialog,
                        if bOptionsDialog != 0 {
                            SW_HIDE as ::core::ffi::c_int
                        } else {
                            SW_SHOW as ::core::ffi::c_int
                        },
                    );
                }
                IDM_ABOUT => {
                    DialogBox!(hInst, MAKEINTRESOURCE!(IDD_ABOUTBOX), hWnd, Some(AboutProc));
                }
                IDM_EXIT => {
                    DestroyWindow(hWnd);
                }
                _ => {
                    return DefWindowProc(hWnd, message, wParam, lParam);
                }
            }
            return 0 as LRESULT;
        }

        WM_PAINT => {
            let mut PaintStruct: PAINTSTRUCT = core::mem::zeroed();
            let mut dc: HDC = core::ptr::null_mut();
            dc = BeginPaint(hWnd, &raw mut PaintStruct);
            Draw(dc);
            EndPaint(hWnd, &raw mut PaintStruct);
            return 0 as LRESULT;
        }

        WM_CONTEXTMENU => {
            TrackPopupMenu(
                notifyMenu,
                0 as UINT,
                GET_X_LPARAM!(lParam),
                GET_Y_LPARAM!(lParam),
                0 as ::core::ffi::c_int,
                hWnd,
                core::ptr::null::<RECT>(),
            );
            return 0 as LRESULT;
        }

        WM_LBUTTONDOWN => {
            let mut rc: RECT = core::mem::zeroed();
            let mut pt: POINT = core::mem::zeroed();
            SetCapture(hWnd);

            GetCursorPos(&raw mut pt);
            GetWindowRect(hWnd, &raw mut rc);
            ptDragOffset.x = pt.x - rc.left;
            ptDragOffset.y = pt.y - rc.top;

            nearEdge = AppBarConfig.uEdge;
        }

        WM_MOUSEMOVE => {
            if GetCapture() == hWnd {
                let mut rc: RECT = core::mem::zeroed();
                let mut pt: POINT = core::mem::zeroed();
                let mut rcWorkArea: RECT = core::mem::zeroed();
                SystemParametersInfo(
                    SPI_GETWORKAREA as UINT,
                    0 as UINT,
                    &raw mut rcWorkArea as PVOID,
                    0 as UINT,
                );
                GetCursorPos(&raw mut pt);
                GetWindowRect(hWnd, &raw mut rc);

                if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                    if pt.x >= rcWorkArea.left && pt.x <= rcWorkArea.right
                        && pt.y >= rcWorkArea.top && pt.y <= rcWorkArea.bottom
                    {
                        AttachAppBar(-(2 as INT));

                        // Fixup offset
                        GetWindowRect(hWnd, &raw mut rc);
                        ptDragOffset.x = (rc.right - rc.left) / 2 as LONG;
                        ptDragOffset.y = 2 as LONG;

                        rc.left = pt.x - ptDragOffset.x;
                        rc.top = pt.y - ptDragOffset.y;

                        SetWindowPos(
                            hWnd,
                            HWND_TOPMOST,
                            rc.left as ::core::ffi::c_int,
                            rc.top as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            SWP_NOSIZE as UINT,
                        );
                    }
                } else {
                    if pt.x <= rcWorkArea.left + 8 as LONG
                        && nearEdge != ABE_LEFT as INT
                    {
                        AttachAppBar(ABE_LEFT as INT);
                        nearEdge = ABE_LEFT as INT;
                    } else if pt.y <= rcWorkArea.top + 8 as LONG
                        && nearEdge != ABE_TOP as INT
                    {
                        AttachAppBar(ABE_TOP as INT);
                        nearEdge = ABE_TOP as INT;
                    } else if pt.x >= rcWorkArea.right - 8 as LONG
                        && nearEdge != ABE_RIGHT as INT
                    {
                        AttachAppBar(ABE_RIGHT as INT);
                        nearEdge = ABE_RIGHT as INT;
                    } else if pt.y >= rcWorkArea.bottom - 8 as LONG
                        && nearEdge != ABE_BOTTOM as INT
                    {
                        AttachAppBar(ABE_BOTTOM as INT);
                        nearEdge = ABE_BOTTOM as INT;
                    } else {
                        rc.left = pt.x - ptDragOffset.x;
                        rc.top = pt.y - ptDragOffset.y;

                        SetWindowPos(
                            hWnd,
                            HWND_TOPMOST,
                            rc.left as ::core::ffi::c_int,
                            rc.top as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            SWP_NOSIZE as UINT,
                        );
                        nearEdge = -(1 as ::core::ffi::c_int) as INT;
                    }
                }

                pMouse = pt;
                Refresh();
            }
        }

        WM_LBUTTONUP => {
            if GetCapture() == hWnd {
                if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                    DoAppBarStuff(ABM_GETSTATE as DWORD);
                } else {
                    SetWindowPos(
                        hWnd,
                        HWND_TOP,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        (SWP_NOMOVE | SWP_NOSIZE) as UINT,
                    );
                }
                ReleaseCapture();
            }
        }

        WM_SIZE => {
            if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                DoAppBarStuff(ABM_SETPOS as DWORD);
            }
            /* fallthrough */
            tail = true;
        }

        WM_DISPLAYCHANGE => {
            tail = true;
        }

        WM_ERASEBKGND => {
            // handle WM_ERASEBKGND by simply returning non-zero because we did all the drawing in WM_PAINT.
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                DoAppBarStuff(ABM_REMOVE as DWORD);
            }

            KillTimer(hWnd, 1 as UINT_PTR);

            /* Save settings to registry */
            SaveSettings();

            /* Cleanup notification icon */
            ZeroMemory!(&raw mut nid, ::core::mem::size_of::<NOTIFYICONDATA>());
            nid.cbSize = ::core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
            nid.uFlags = NIF_MESSAGE as UINT;
            nid.hWnd = hWnd;
            nid.uCallbackMessage = APPMSG_NOTIFYICON as UINT;
            Shell_NotifyIcon(NIM_DELETE as DWORD, &raw mut nid);
            DestroyIcon(notifyIcon);

            DestroyWindow(hOptionsDialog);

            PostQuitMessage(0 as ::core::ffi::c_int);
            return 0 as LRESULT;
        }

        WM_CREATE => {
            let mut tempMenu: HMENU = core::ptr::null_mut();

            /* Get the desktop window */
            hDesktopWindow = GetDesktopWindow();

            /* Set the timer */
            SetTimer(
                hWnd,
                1 as UINT_PTR,
                TIMER_SPEED as UINT,
                None,
            );

            /* Notification icon */
            notifyIcon = LoadImage(
                hInst,
                MAKEINTRESOURCE!(IDI_ICON),
                IMAGE_ICON as UINT,
                16 as ::core::ffi::c_int,
                16 as ::core::ffi::c_int,
                0 as UINT,
            ) as HICON;

            ZeroMemory!(&raw mut nid, ::core::mem::size_of::<NOTIFYICONDATA>());
            nid.cbSize = ::core::mem::size_of::<NOTIFYICONDATA>() as DWORD;
            nid.uFlags = (NIF_ICON | NIF_MESSAGE) as UINT;
            nid.hWnd = hWnd;
            nid.uCallbackMessage = APPMSG_NOTIFYICON as UINT;
            nid.hIcon = notifyIcon;
            Shell_NotifyIcon(NIM_ADD as DWORD, &raw mut nid);

            tempMenu = LoadMenu(hInst, MAKEINTRESOURCE!(IDC_MAGNIFIER));
            notifyMenu = GetSubMenu(tempMenu, 0 as ::core::ffi::c_int);
            RemoveMenu(tempMenu, 0 as UINT, MF_BYPOSITION as UINT);
            DestroyMenu(tempMenu);
            return 0 as LRESULT;
        }

        APPMSG_APPBAR => {
            match wParam {
                x if x == ABN_STATECHANGE as WPARAM => {
                    DoAppBarStuff(ABM_GETSTATE as DWORD);
                }
                x if x == ABN_POSCHANGED as WPARAM => {
                    DoAppBarStuff(ABM_SETPOS as DWORD);
                }
                x if x == ABN_FULLSCREENAPP as WPARAM => {
                    if lParam == 0 {
                        DoAppBarStuff(ABM_GETSTATE as DWORD);
                    } else {
                        SetWindowPos(
                            hMainWnd,
                            HWND_BOTTOM,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            (SWP_NOMOVE | SWP_NOSIZE) as UINT,
                        );
                    }
                }
                x if x == ABN_WINDOWARRANGE as WPARAM => {
                    ShowWindow(
                        hMainWnd,
                        if lParam != 0 {
                            SW_HIDE as ::core::ffi::c_int
                        } else {
                            SW_SHOW as ::core::ffi::c_int
                        },
                    );
                }
                _ => {}
            }
            return 0 as LRESULT;
        }

        APPMSG_NOTIFYICON => {
            HandleNotifyIconMessage(hWnd, wParam, lParam);
            return 0 as LRESULT;
        }

        WM_ACTIVATE => {
            if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                DoAppBarStuff(ABM_ACTIVATE as DWORD);
            }
        }

        WM_WINDOWPOSCHANGED => {
            if AppBarConfig.uEdge >= 0 as ::core::ffi::c_int {
                DoAppBarStuff(ABM_WINDOWPOSCHANGED as DWORD);
            }
            Refresh();
        }

        _ => {}
    }

    if tail {
        bRecreateOffscreenDC = TRUE as BOOL;
        Refresh();
    }

    return DefWindowProc(hWnd, message, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn AboutProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    match message {
        WM_INITDIALOG => {
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if (LOWORD!(wParam)) as u32 == IDOK || (LOWORD!(wParam)) as u32 == IDCANCEL {
                EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
                return TRUE as INT_PTR;
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn OptionsProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match message {
        WM_INITDIALOG => {
            /* Add the zoom items */
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"1\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"2\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"3\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"4\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"5\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"6\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"7\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"8\0".as_ptr() as LPARAM));
            SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_ADDSTRING, 0, (b"9\0".as_ptr() as LPARAM));

            if uiZoom >= 1 && uiZoom <= 9 {
                SendDlgItemMessage(hDlg, (IDC_ZOOM as i32), CB_SETCURSEL, (uiZoom - 1) as WPARAM, 0);
            }

            if bFollowMouse != 0 {
                SendDlgItemMessage(hDlg, (IDC_FOLLOWMOUSECHECK as i32), BM_SETCHECK, wParam, 0);
            }

            if bFollowFocus != 0 {
                SendDlgItemMessage(hDlg, (IDC_FOLLOWKEYBOARDCHECK as i32), BM_SETCHECK, wParam, 0);
            }

            if bFollowCaret != 0 {
                SendDlgItemMessage(hDlg, (IDC_FOLLOWTEXTEDITINGCHECK as i32), BM_SETCHECK, wParam, 0);
            }

            if bInvertColors != 0 {
                SendDlgItemMessage(hDlg, (IDC_INVERTCOLORSCHECK as i32), BM_SETCHECK, wParam, 0);
            }

            if bStartMinimized != 0 {
                SendDlgItemMessage(hDlg, (IDC_STARTMINIMIZEDCHECK as i32), BM_SETCHECK, wParam, 0);
            }

            if bShowMagnifier != 0 {
                SendDlgItemMessage(hDlg, (IDC_SHOWMAGNIFIERCHECK as i32), BM_SETCHECK, wParam, 0);
            }

            return TRUE as INT_PTR;
        }

        WM_SHOWWINDOW => {
            bOptionsDialog = wParam as BOOL;
        }

        WM_COMMAND => match (LOWORD!(wParam)) as u32 {
            IDOK | IDCANCEL => {
                ShowWindow(hDlg, (SW_HIDE) as i32);
                return TRUE as INT_PTR;
            }

            IDC_BUTTON_HELP => {
                /* Unimplemented */
                MessageBox(hDlg, b"Magnifier help not available yet!\0".as_ptr() as LPCSTR, b"Help\0".as_ptr() as LPCSTR, MB_OK);
            }

            IDC_ZOOM => {
                if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                    let hCombo: HWND = GetDlgItem(hDlg, (IDC_ZOOM as i32));
                    let mut currentZoomValue: [TCHAR; 2] = [0; 2];

                    /* Get index of current selection and the text of that selection */
                    let currentSelectionIndex: i32 = ComboBox_GetCurSel!(hCombo);
                    ComboBox_GetLBText!(hCombo, currentSelectionIndex, currentZoomValue.as_mut_ptr());
                    uiZoom = _ttoi(currentZoomValue.as_ptr()) as UINT;
                    /* The zoom factor cannot be smaller than 1 (and not zero) or greater than 9 */
                    if uiZoom < 1 || uiZoom > 9 {
                        uiZoom = 1;
                    }

                    /* Trigger the Draw function to rezoom (which will be set false automatically after rezooming) */
                    bRecreateOffscreenDC = (TRUE) as i32;

                    /* Update the magnifier UI */
                    Refresh();
                }
            }

            IDC_INVERTCOLORSCHECK => {
                bInvertColors = IsDlgButtonChecked(hDlg, (IDC_INVERTCOLORSCHECK as i32)) as BOOL;
                Refresh();
            }
            IDC_FOLLOWMOUSECHECK => {
                bFollowMouse = IsDlgButtonChecked(hDlg, (IDC_FOLLOWMOUSECHECK as i32)) as BOOL;
            }
            IDC_FOLLOWKEYBOARDCHECK => {
                bFollowFocus = IsDlgButtonChecked(hDlg, (IDC_FOLLOWKEYBOARDCHECK as i32)) as BOOL;
            }
            IDC_FOLLOWTEXTEDITINGCHECK => {
                bFollowCaret = IsDlgButtonChecked(hDlg, (IDC_FOLLOWTEXTEDITINGCHECK as i32)) as BOOL;
            }
            IDC_STARTMINIMIZEDCHECK => {
                bStartMinimized = IsDlgButtonChecked(hDlg, (IDC_STARTMINIMIZEDCHECK as i32)) as BOOL;
            }
            IDC_SHOWMAGNIFIER => {
                bShowMagnifier = IsDlgButtonChecked(hDlg, (IDC_SHOWMAGNIFIERCHECK as i32)) as BOOL;
                ShowWindow(hMainWnd, (if bShowMagnifier != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 }));
            }
            _ => {}
        },
        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn WarningProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match message {
        WM_INITDIALOG => {
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match ((LOWORD!(wParam) as i32)) as u32 {
                IDC_SHOWWARNINGCHECK => {
                    bShowWarning = (IsDlgButtonChecked(hDlg, (IDC_SHOWWARNINGCHECK as i32)) == 0) as BOOL;
                }
                _ => {}
            }
            if ((LOWORD!(wParam) as i32)) as u32 == IDOK || ((LOWORD!(wParam) as i32)) as u32 == IDCANCEL {
                EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
                return TRUE as INT_PTR;
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
