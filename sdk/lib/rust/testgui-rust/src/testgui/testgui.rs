// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1A0BF4BFAB61: [u16; 40] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0041, 0x0052,
    0x004D, 0x0036, 0x0034, 0x0020, 0x002D, 0x0020, 0x0048, 0x0065, 0x006C, 0x006C,
    0x006F, 0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x0020, 0x0063, 0x006C, 0x0069,
    0x0065, 0x006E, 0x0074, 0x0020, 0x0061, 0x0072, 0x0065, 0x0061, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_2F95DEE4D2B5: [u16; 30] = [
    0x0028u16, 0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0074, 0x006F, 0x0020, 0x0074,
    0x0065, 0x0073, 0x0074, 0x0020, 0x006B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061,
    0x0072, 0x0064, 0x0020, 0x0069, 0x006E, 0x0070, 0x0075, 0x0074, 0x0029, 0x0000,
];
pub static RUSTCONV_WSTR_4E5265E6E7F2: [u16; 15] = [
    0x0054u16, 0x0065, 0x0078, 0x0074, 0x004F, 0x0075, 0x0074, 0x0020, 0x0077, 0x006F,
    0x0072, 0x006B, 0x0073, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_4F77B957397F: [u16; 19] = [
    0x0052u16, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0020, 0x0041, 0x0052,
    0x004D, 0x0036, 0x0034, 0x0020, 0x0054, 0x0065, 0x0073, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_9D4B732162FA: [u16; 8] = [0x0054u16, 0x0079, 0x0070, 0x0065, 0x0064, 0x003A, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_CE0EF80010D6: [u16; 23] = [
    0x0045u16, 0x0064, 0x0069, 0x0074, 0x0020, 0x0063, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x0020, 0x0074, 0x0065, 0x0073, 0x0074, 0x0020, 0x0074, 0x0065,
    0x0078, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_CF7179D37A80: [u16; 5] = [0x0045u16, 0x0044, 0x0049, 0x0054, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static TESTGUI_CLASS: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"T\0e\0s\0t\0G\0u\0i\0C\0l\0a\0s\0s\0\0\0",
    )
};

static mut g_TypedText: [WCHAR; 256] = [0; 256];

static mut g_TypedLen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

static mut g_hEdit: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

// WINE_DEFAULT_DEBUG_CHANNEL(testgui)

// #define TESTGUI_CLASS L"TestGuiClass" (NUL-terminated UTF-16)

unsafe extern "system" fn TestWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            DbgPrint(b"[TESTGUI] WM_CREATE received\n\0".as_ptr() as *const i8);
            /* Create child EDIT control to test edit control rendering */
            g_hEdit = CreateWindowExW(
                0,
                RUSTCONV_WSTR_CF7179D37A80.as_ptr(),
                RUSTCONV_WSTR_CE0EF80010D6.as_ptr(),
                (WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL) as DWORD,
                10,
                120,
                380,
                24,
                hWnd,
                (100) as HMENU,
                (*(lParam as LPCREATESTRUCT)).hInstance,
                core::ptr::null_mut(),
            );
            if !g_hEdit.is_null() {
                DbgPrint(
                    b"[TESTGUI] EDIT control created (hWnd=%p)\n\0".as_ptr() as *const i8,
                    g_hEdit,
                );
            } else {
                DbgPrint(
                    b"[TESTGUI] EDIT control creation FAILED (error %lu)\n\0".as_ptr() as *const i8,
                    GetLastError(),
                );
            }
            return 0 as LRESULT;
        }

        WM_KEYDOWN => {
            DbgPrint(
                b"[TESTGUI] WM_KEYDOWN vk=0x%02X scan=0x%02X\n\0".as_ptr() as *const i8,
                wParam as UINT,
                (((lParam >> 16) & 0xFF) as UINT),
            );
            /* F1 = toggle focus to EDIT control */
            if wParam == (VK_F1 as WPARAM) && !g_hEdit.is_null() {
                SetFocus(g_hEdit);
                DbgPrint(b"[TESTGUI] Focus switched to EDIT control\n\0".as_ptr() as *const i8);
            }
            return 0 as LRESULT;
        }

        WM_CHAR => {
            let mut ch: WCHAR = wParam as WCHAR;
            DbgPrint(
                b"[TESTGUI] WM_CHAR char=0x%04X '%c'\n\0".as_ptr() as *const i8,
                ch as UINT,
                if ((ch as i32) >= 0x20) && ((ch as i32) < 0x7F) {
                    (ch as i8) as i32
                } else {
                    '?' as i32
                },
            );
            if ((ch as i32) == 8) && (g_TypedLen > 0) {
                /* Backspace */
                g_TypedLen -= 1;
            } else if ((ch as i32) >= 0x20) && (g_TypedLen < 255) {
                let idx = g_TypedLen;
                g_TypedLen += 1;
                g_TypedText[idx as usize] = ch;
            }
            g_TypedText[g_TypedLen as usize] = 0 as WCHAR;
            InvalidateRect(hWnd, core::ptr::null(), TRUE as BOOL);
            return 0 as LRESULT;
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hdc: HDC = core::ptr::null_mut();
            let mut rc: RECT = core::mem::zeroed();
            let mut hBrush: HBRUSH = core::ptr::null_mut();

            DbgPrint(b"[TESTGUI] WM_PAINT received\n\0".as_ptr() as *const i8);

            hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            if !hdc.is_null() {
                GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));

                /* Paint a bright blue rectangle as window background */
                hBrush = CreateSolidBrush(RGB!(
                    0,
                    100,
                    255
                ));
                if !hBrush.is_null() {
                    FillRect(hdc, core::ptr::addr_of!(rc), hBrush);
                    DeleteObject(hBrush as HGDIOBJ);
                }

                /* Paint a smaller green rectangle in the center */
                {
                    let mut inner: RECT = core::mem::zeroed();
                    inner.left = rc.right / 4;
                    inner.top = rc.bottom / 4;
                    inner.right = rc.right * 3 / 4;
                    inner.bottom = rc.bottom * 3 / 4;
                    hBrush = CreateSolidBrush(RGB!(
                        0,
                        200,
                        0
                    ));
                    if !hBrush.is_null() {
                        FillRect(hdc, core::ptr::addr_of!(inner), hBrush);
                        DeleteObject(hBrush as HGDIOBJ);
                    }
                }

                /* Draw text to verify font rendering in client area */
                SetBkMode(hdc, (TRANSPARENT as i32));
                SetTextColor(hdc, RGB!(
                    255,
                    255,
                    255
                ));
                {
                    let mut textRect: RECT = core::mem::zeroed();
                    textRect.left = 20;
                    textRect.top = 20;
                    textRect.right = rc.right - 20;
                    textRect.bottom = 60;
                    DrawTextW(
                        hdc,
                        RUSTCONV_WSTR_1A0BF4BFAB61.as_ptr(),
                        -1,
                        core::ptr::addr_of_mut!(textRect),
                        (DT_LEFT | DT_SINGLELINE | DT_VCENTER) as UINT,
                    );
                }

                SetTextColor(hdc, RGB!(
                    255,
                    255,
                    0
                ));
                TextOutW(
                    hdc,
                    rc.right / 4 + 10,
                    rc.bottom / 4 + 10,
                    RUSTCONV_WSTR_4E5265E6E7F2.as_ptr(),
                    14,
                );

                /* Display typed text (keyboard input test) */
                if g_TypedLen > 0 {
                    let mut typedRect: RECT = core::mem::zeroed();
                    typedRect.left = 20;
                    typedRect.top = 70;
                    typedRect.right = rc.right - 20;
                    typedRect.bottom = 110;
                    SetTextColor(hdc, RGB!(
                        255,
                        200,
                        0
                    ));
                    DrawTextW(
                        hdc,
                        RUSTCONV_WSTR_9D4B732162FA.as_ptr(),
                        -1,
                        core::ptr::addr_of_mut!(typedRect),
                        (DT_LEFT | DT_SINGLELINE | DT_VCENTER) as UINT,
                    );
                    typedRect.left = 100;
                    DrawTextW(
                        hdc,
                        core::ptr::addr_of_mut!(g_TypedText) as *mut WCHAR as LPCWSTR,
                        g_TypedLen,
                        core::ptr::addr_of_mut!(typedRect),
                        (DT_LEFT | DT_SINGLELINE | DT_VCENTER) as UINT,
                    );
                } else {
                    let mut typedRect: RECT = core::mem::zeroed();
                    typedRect.left = 20;
                    typedRect.top = 70;
                    typedRect.right = rc.right - 20;
                    typedRect.bottom = 110;
                    SetTextColor(hdc, RGB!(
                        180,
                        180,
                        180
                    ));
                    DrawTextW(
                        hdc,
                        RUSTCONV_WSTR_2F95DEE4D2B5.as_ptr(),
                        -1,
                        core::ptr::addr_of_mut!(typedRect),
                        (DT_LEFT | DT_SINGLELINE | DT_VCENTER) as UINT,
                    );
                }

                EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
            }
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            DbgPrint(b"[TESTGUI] WM_DESTROY received\n\0".as_ptr() as *const i8);
            PostQuitMessage(0 as i32);
            return 0 as LRESULT;
        }

        _ => {}
    }

    return DefWindowProcW(hWnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nShowCmd: i32,
) -> i32 {
    let mut wcex: WNDCLASSEXW = core::mem::zeroed();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut msg: MSG = core::mem::zeroed();

    let _ = hPrevInstance;
    let _ = lpCmdLine;

    DbgPrint(
        b"[TESTGUI] wWinMain entered (hInstance=%p)\n\0".as_ptr() as *const core::ffi::c_char,
        hInstance,
    );

    /* Register window class */
    ZeroMemory!(
        core::ptr::addr_of_mut!(wcex) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSEXW>(),
    );
    wcex.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
    wcex.style = (CS_HREDRAW | CS_VREDRAW) as u32;
    wcex.lpfnWndProc = Some(TestWndProc);
    wcex.hInstance = hInstance;
    wcex.hCursor = core::ptr::null_mut(); /* No cursor resources on ARM64 yet */
    wcex.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wcex.lpszClassName = (TESTGUI_CLASS).as_ptr();

    if !(RegisterClassExW(core::ptr::addr_of_mut!(wcex)) != 0) {
        DbgPrint(
            b"[TESTGUI] RegisterClassExW FAILED (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            GetLastError(),
        );
        return 1;
    }
    DbgPrint(
        b"[TESTGUI] RegisterClassExW succeeded\n\0".as_ptr() as *const core::ffi::c_char,
    );

    /* Create the window */
    hWnd = CreateWindowExW(
        0,
        (TESTGUI_CLASS).as_ptr(),
        RUSTCONV_WSTR_4F77B957397F.as_ptr(),
        (WS_OVERLAPPEDWINDOW | WS_VISIBLE) as DWORD,
        0,
        0, /* Position: top-left */
        400,
        300, /* Size */
        core::ptr::null_mut(),  /* No parent */
        core::ptr::null_mut(),  /* No menu */
        hInstance,
        core::ptr::null_mut(),
    );

    if hWnd.is_null() {
        DbgPrint(
            b"[TESTGUI] CreateWindowExW FAILED (error %lu)\n\0".as_ptr() as *const core::ffi::c_char,
            GetLastError(),
        );
        return 1;
    }
    DbgPrint(
        b"[TESTGUI] CreateWindowExW succeeded (hWnd=%p)\n\0".as_ptr() as *const core::ffi::c_char,
        hWnd,
    );

    ShowWindow(hWnd, nShowCmd);
    SetForegroundWindow(hWnd);
    SetFocus(hWnd);
    InvalidateRect(hWnd, core::ptr::null(), (TRUE) as BOOL);
    UpdateWindow(hWnd);
    DbgPrint(
        b"[TESTGUI] Window shown, focus set, entering message loop\n\0".as_ptr()
            as *const core::ffi::c_char,
    );

    /* Message loop */
    while GetMessageW(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) > 0
    {
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessageW(core::ptr::addr_of_mut!(msg));
    }

    DbgPrint(
        b"[TESTGUI] Message loop exited (wParam=%lu)\n\0".as_ptr() as *const core::ffi::c_char,
        msg.wParam as u32,
    );
    return msg.wParam as i32;
}
