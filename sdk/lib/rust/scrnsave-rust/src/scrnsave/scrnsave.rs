use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hMainWindow: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut fChildPreview: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut hMainInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut szName: [TCHAR; 40] = [0; 40];

#[no_mangle]
pub static mut szAppName: [TCHAR; 40] = [0; 40];

#[no_mangle]
pub static mut szIniFile: [TCHAR; 13] = [0; 13];

#[no_mangle]
pub static mut szScreenSaver: [TCHAR; 22] = [0; 22];

#[no_mangle]
pub static mut szHelpFile: [TCHAR; 13] = [0; 13];

#[no_mangle]
pub static mut szNoHelpMemory: [TCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut MyHelpMessage: UINT = 0;

static mut pt_orig: POINT = tagPOINT { x: 0, y: 0 };

static mut pt_init: BOOL = (FALSE as i32);

// TITLEBARNAMELEN / APPNAMEBUFFERLEN / MAXFILELEN / BUFFLEN are not defined in this preamble;
// values taken from the standard scrnsave.h.

// #define CLASS_SCRNSAVE TEXT("WindowsScreenSaverClass")
pub static CLASS_SCRNSAVE: [u16; 24] = [
    b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16,
    b's' as u16, b'S' as u16, b'c' as u16, b'r' as u16, b'e' as u16, b'e' as u16,
    b'n' as u16, b'S' as u16, b'a' as u16, b'v' as u16, b'e' as u16, b'r' as u16,
    b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, 0u16,
];

// #define ISNUM(c) ((c) >= '0' && (c) <= '9')  -- function-like macro, skipped (inlined by functions)

unsafe extern "C" fn ISSPACE(c: TCHAR) -> i32 {
    return (c as i32 == ' ' as i32 || c as i32 == '\t' as i32) as i32;
}

unsafe extern "C" fn _toulptr(s: *const TCHAR) -> ULONG_PTR {
    let mut res: ULONG_PTR;
    let mut n: ULONG_PTR;
    let mut p: *const TCHAR;
    p = s;
    while *p != 0 {
        if !ISNUM!(*p as ::core::ffi::c_int) {
            break;
        }
        p = p.offset(1);
    }
    p = p.offset(-1);
    res = 0;
    n = 1;
    while p >= s {
        res = res.wrapping_add(
            ((*p as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as ULONG_PTR).wrapping_mul(n),
        );
        p = p.offset(-1);
        n = n.wrapping_mul(10);
    }
    return res;
}

unsafe extern "system" fn SysScreenSaverProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_DESTROY => {
            PostQuitMessage(0 as i32);
        }
        WM_SYSCOMMAND => {
            if fChildPreview == 0 {
                match wParam {
                    __c if __c == SC_CLOSE as _ || __c == SC_NEXTWINDOW as _ || __c == SC_PREVWINDOW as _ || __c == SC_SCREENSAVE as _ => {
                        return FALSE as LRESULT;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    return ScreenSaverProc(hWnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn DefScreenSaverProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    // Don't do any special processing when in preview mode
    if fChildPreview != 0 {
        return DefWindowProc(hWnd, uMsg, wParam, lParam);
    }

    match uMsg {
        WM_NCACTIVATE | WM_ACTIVATE | WM_ACTIVATEAPP => {
            if wParam == 0 {
                // wParam is FALSE, so the screen saver is losing the focus.
                PostMessage(hWnd, WM_CLOSE, 0, 0);
            }
        }

        WM_MOUSEMOVE => {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            GetCursorPos(core::ptr::addr_of_mut!(pt));

            if pt_init == 0 {
                pt_orig = pt;
                pt_init = (TRUE as i32);
            } else if abs(pt.x - pt_orig.x) > 10
                || abs(pt.y - pt_orig.y) > 10
            {
                PostMessage(hWnd, WM_CLOSE, 0, 0);
            }
        }

        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN
        | WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN => {
            // Send a WM_CLOSE to close the screen saver (allows
            // the screen saver to perform clean-up tasks)
            PostMessage(hWnd, WM_CLOSE, 0, 0);
        }

        WM_SETCURSOR => {
            SetCursor(core::ptr::null_mut());
            return (TRUE) as LRESULT;
        }

        _ => {}
    }

    DefWindowProc(hWnd, uMsg, wParam, lParam)
}

unsafe extern "C" fn RegisterScreenSaverClass() -> BOOL {
    let mut cls: WNDCLASS = core::mem::zeroed();

    cls.hCursor = core::ptr::null_mut();
    cls.hIcon = LoadIcon(hMainInstance, MAKEINTATOM!(ID_APP));
    cls.lpszMenuName = core::ptr::null();
    cls.lpszClassName = (CLASS_SCRNSAVE).as_ptr() as LPCWSTR;
    cls.hbrBackground = GetStockObject((BLACK_BRUSH as i32)) as HBRUSH;
    cls.hInstance = hMainInstance;
    cls.style = (CS_VREDRAW | CS_HREDRAW | CS_DBLCLKS | CS_SAVEBITS | CS_PARENTDC) as UINT;
    cls.lpfnWndProc = Some(SysScreenSaverProc);
    cls.cbWndExtra = 0;
    cls.cbClsExtra = 0;

    return (RegisterClass(core::ptr::addr_of_mut!(cls)) != 0) as BOOL;
}

unsafe extern "C" fn LaunchConfig(hParent: HWND) -> i32 {
    // Only show the dialog if the RegisterDialogClasses function succeeded.
    // This is the same behaviour as MS' scrnsave.lib.
    if RegisterDialogClasses((hMainInstance).cast()) == 0 {
        return -1;
    }

    return DialogBox!(
        hMainInstance,
        MAKEINTRESOURCE!(DLG_SCRNSAVECONFIGURE),
        hParent,
        core::mem::transmute::<
            Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> BOOL>,
            DLGPROC,
        >(Some(
            ScreenSaverConfigureDialog,
        )),
    ) as i32;
}

unsafe extern "C" fn LaunchScreenSaver(hParent: HWND) -> ::core::ffi::c_int {
    let mut lpWindowName: LPCTSTR = ::core::ptr::null();
    let mut style: UINT = 0;
    let mut exstyle: UINT = 0;
    let mut rc: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut msg: MSG = tagMSG {
        hwnd: ::core::ptr::null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: tagPOINT { x: 0, y: 0 },
    };
    // TEXT("RegisterClass() failed")
    let fail_text: [u16; 23] = [
        'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'e' as u16,
        'r' as u16, 'C' as u16, 'l' as u16, 'a' as u16, 's' as u16, 's' as u16, '(' as u16,
        ')' as u16, ' ' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16,
        'd' as u16, 0,
    ];
    // TEXT("Preview")
    let preview_name: [u16; 8] = [
        'P' as u16, 'r' as u16, 'e' as u16, 'v' as u16, 'i' as u16, 'e' as u16, 'w' as u16, 0,
    ];
    // TEXT("Screen Saver")
    let screen_name: [u16; 13] = [
        'S' as u16, 'c' as u16, 'r' as u16, 'e' as u16, 'e' as u16, 'n' as u16, ' ' as u16,
        'S' as u16, 'a' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 0,
    ];

    if RegisterScreenSaverClass() == 0 {
        MessageBox(
            core::ptr::null_mut(),
            fail_text.as_ptr(),
            core::ptr::null_mut(),
            MB_ICONHAND,
        );
        return -(1);
    }

    // A slightly different approach needs to be used when displaying in a preview window
    if !hParent.is_null() {
        fChildPreview = (TRUE as i32);
        lpWindowName = preview_name.as_ptr();

        style = (WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN) as UINT;
        exstyle = 0 as UINT;

        GetClientRect(hParent, core::ptr::addr_of_mut!(rc));
        rc.left = 0 as LONG;
        rc.top = 0 as LONG;
    } else {
        fChildPreview = (FALSE as i32);
        lpWindowName = screen_name.as_ptr();

        style = (WS_POPUP
            | WS_VISIBLE
            | WS_CLIPSIBLINGS
            | WS_CLIPCHILDREN) as UINT;
        exstyle = (WS_EX_TOOLWINDOW | WS_EX_TOPMOST) as UINT;

        // Get the left & top side coordinates of the virtual screen
        rc.left = GetSystemMetrics((SM_XVIRTUALSCREEN as i32)) as LONG;
        rc.top = GetSystemMetrics((SM_YVIRTUALSCREEN as i32)) as LONG;
        // Get the width and height of the virtual screen
        rc.right = GetSystemMetrics((SM_CXVIRTUALSCREEN as i32)) as LONG;
        rc.bottom = GetSystemMetrics((SM_CYVIRTUALSCREEN as i32)) as LONG;
    }

    // Create the main screen saver window
    hMainWindow = CreateWindowEx(
        exstyle as DWORD,
        (CLASS_SCRNSAVE).as_ptr(),
        lpWindowName,
        style as DWORD,
        rc.left as ::core::ffi::c_int,
        rc.top as ::core::ffi::c_int,
        rc.right as ::core::ffi::c_int,
        rc.bottom as ::core::ffi::c_int,
        hParent,
        core::ptr::null_mut(),
        hMainInstance,
        core::ptr::null_mut(),
    );
    if hMainWindow.is_null() {
        return -(1);
    }

    // Display window and start pumping messages
    ShowWindow(hMainWindow, (SW_SHOW as i32));
    if hParent.is_null() {
        SetCursor(core::ptr::null_mut());
    }

    while GetMessage(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessage(core::ptr::addr_of_mut!(msg));
    }

    return msg.wParam as ::core::ffi::c_int;
}

#[no_mangle]
// `_tWinMain` under this library's -DUNICODE (tchar.h) is `wWinMain`, the entry the
// UNICODE screensavers' startup calls; the model wrote the ANSI `WinMain`.
pub unsafe extern "system" fn wWinMain(hInst: HINSTANCE, hPrevInst: HINSTANCE, CmdLine: LPTSTR, nCmdShow: i32) -> i32 {
    let mut p: LPTSTR = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(nCmdShow);
    UNREFERENCED_PARAMETER!(hPrevInst);

    hMainInstance = hInst;

    p = CmdLine;
    while *p != 0 {
        match *p as i32 {
            83 | 115 => {
                // Start the screen saver
                return LaunchScreenSaver(core::ptr::null_mut());
            }
            80 | 112 => {
                let mut hParent: HWND = core::ptr::null_mut();

                loop {
                    p = p.offset(1);
                    if !(ISSPACE(*p) != 0) {
                        break;
                    }
                }
                hParent = _toulptr(p as *const TCHAR) as HWND;

                // Start the screen saver in preview mode
                if !hParent.is_null() && IsWindow(hParent) != 0 {
                    return LaunchScreenSaver(hParent);
                } else {
                    return -1;
                }
            }
            67 | 99 => {
                let mut hParent: HWND = core::ptr::null_mut();

                if *p.offset(1 as isize) as i32 == ':' as i32 {
                    hParent = _toulptr(p.offset(2 as isize) as *const TCHAR) as HWND;
                } else {
                    hParent = GetForegroundWindow();
                }

                // Display the configuration dialog
                if !hParent.is_null() && IsWindow(hParent) != 0 {
                    return LaunchConfig(hParent);
                } else {
                    return -1;
                }
            }
            45 | 47 | 32 => {
                // '-', '/', ' '
            }
            _ => {
                // default
            }
        }
        p = p.offset(1);
    }

    return LaunchConfig(core::ptr::null_mut());
}
