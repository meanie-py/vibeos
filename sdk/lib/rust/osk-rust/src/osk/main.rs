use crate::osk::settings::LoadSettings;
use crate::osk::settings::SaveSettings;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_01AE8BCA3FED: [u16; 2] = [0x0026u16, 0x0000];
pub static RUSTCONV_WSTR_03C602A9F6CE: [u16; 3] = [0x0026u16, 0x0026, 0x0000];
pub static RUSTCONV_WSTR_10AAF8759FB4: [u16; 11] = [
    0x004Fu16, 0x0053, 0x004B, 0x0052, 0x0075, 0x006E, 0x006E, 0x0069, 0x006E, 0x0067,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const OSK_CLASS: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"O\0S\0K\0M\0a\0i\0n\0W\0i\0n\0d\0o\0w\0\0\0",
    )
};

#[no_mangle]
pub static mut Globals: OSK_GLOBALS = OSK_GLOBALS {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hMainWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hBrushGreenLed: ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__,
    iTimer: 0,
    Keyboard: ::core::ptr::null::<_KEYBOARD_STRUCT>() as *mut _KEYBOARD_STRUCT,
    hKeys: ::core::ptr::null::<HWND>() as *mut HWND,
    hFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
    szTitle: [0; 260],
    bShowWarning: 0,
    bIsEnhancedKeyboard: 0,
    bSoundClick: 0,
    bAlwaysOnTop: 0,
    PosX: 0,
    PosY: 0,
    FontFaceName: [0; 32],
    FontHeight: 0,
};

#[no_mangle]
pub static mut LedKey: [OSK_KEYLEDINDICATOR; 3] = [
    OSK_KEYLEDINDICATOR {
        vKey: (VK_NUMLOCK as i32),
        DlgResource: (IDC_LED_NUM as i32),
        wScanCode: 0x145 as WORD,
        bWasKeyPressed: (FALSE) as i32,
    },
    OSK_KEYLEDINDICATOR {
        vKey: (VK_CAPITAL as i32),
        DlgResource: (IDC_LED_CAPS as i32),
        wScanCode: 0x13a as WORD,
        bWasKeyPressed: (FALSE) as i32,
    },
    OSK_KEYLEDINDICATOR {
        vKey: (VK_SCROLL as i32),
        DlgResource: (IDC_LED_SCROLL as i32),
        wScanCode: 0x146 as WORD,
        bWasKeyPressed: (FALSE) as i32,
    },
];

pub const VK_NUMLOCK: u32 = 0x90; // VK_NUMLOCK
pub const VK_CAPITAL: u32 = 0x14; // VK_CAPITAL
pub const VK_SCROLL: u32 = 0x91; // VK_SCROLL
// IDC_LED_* are resource IDs from the OSK resource header; values unverified here.

// Field names of OSK_KEYLEDINDICATOR are assumed; adjust to the bindgen struct.
#[no_mangle]
pub unsafe extern "C" fn OSK_SetImage(IdDlgItem: c_int, IdResource: c_int) -> c_int {
    let mut hIcon: HICON;
    let mut hWndItem: HWND;

    hIcon = LoadImageW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IdResource),
        IMAGE_ICON,
        16,
        16,
        LR_DEFAULTCOLOR,
    ) as HICON;
    if hIcon.is_null() {
        return (FALSE) as i32;
    }

    hWndItem = GetDlgItem(Globals.hMainWnd, IdDlgItem);
    if hWndItem.is_null() {
        DestroyIcon(hIcon);
        return (FALSE) as i32;
    }

    SendMessageW(
        hWndItem,
        BM_SETIMAGE,
        IMAGE_ICON as WPARAM,
        hIcon as LPARAM,
    );

    /* The system automatically deletes these resources when the process that created them terminates (MSDN) */

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_SetText(IdDlgItem: ::core::ffi::c_int, IdResource: ::core::ffi::c_int) {
    let mut szText: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hWndItem: HWND = core::ptr::null_mut();

    hWndItem = GetDlgItem(Globals.hMainWnd, IdDlgItem);

    if hWndItem.is_null() {
        return;
    }

    LoadStringW(
        Globals.hInstance,
        IdResource as UINT,
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    );

    SetWindowTextW(hWndItem, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR);
}
#[no_mangle]
pub unsafe extern "system" fn OSK_WarningProc(hDlg: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match Msg {
        WM_INITDIALOG => {
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SHOWWARNINGCHECK => {
                    Globals.bShowWarning = (IsDlgButtonChecked(hDlg, (IDC_SHOWWARNINGCHECK as i32)) == 0) as BOOL;
                    return (TRUE) as INT_PTR;
                }

                IDOK | IDCANCEL => {
                    EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn OSK_WarningDlgThread(lpParameter: LPVOID) -> DWORD {
    let hInstance: HINSTANCE = lpParameter as HINSTANCE;

    DialogBoxW!(hInstance, MAKEINTRESOURCEW!(IDD_WARNINGDIALOG_OSK), Globals.hMainWnd, Some(OSK_WarningProc));
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn OSK_About() {
    let mut szAuthors: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Load the strings into the "About" dialog */
    LoadStringW(
        Globals.hInstance,
        IDS_AUTHORS,
        core::ptr::addr_of_mut!(szAuthors) as LPWSTR,
        (_countof!(szAuthors)) as INT,
    );

    /* Load the icon */
    /* Finally, execute the "About" dialog by using the Shell routine */
    ShellAboutW(
        Globals.hMainWnd,
        core::ptr::addr_of_mut!(Globals.szTitle) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szAuthors) as *mut WCHAR as LPCWSTR,
        LoadImageW(
            Globals.hInstance,
            MAKEINTRESOURCEW!(IDI_OSK),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        ) as HICON,
    );
}
#[no_mangle]
pub unsafe extern "C" fn OSK_DestroyKeys() {
    let mut i: ::core::ffi::c_int;
    /* Hide before destroying child controls */
    ShowWindow(Globals.hMainWnd, (SW_HIDE) as i32);

    i = 0 as ::core::ffi::c_int;
    while i < (*Globals.Keyboard).KeyCount {
        DestroyWindow(*Globals.hKeys.offset(i as isize));
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < _countof!(LedKey) as ::core::ffi::c_int {
        DestroyWindow(GetDlgItem(
            Globals.hMainWnd,
            LedKey[i as usize].DlgResource as ::core::ffi::c_int,
        ));
        i += 1;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        Globals.hKeys as PVOID,
    );
    Globals.hKeys = ::core::ptr::null_mut::<HWND>();
    Globals.Keyboard = ::core::ptr::null_mut::<_KEYBOARD_STRUCT>();
}
#[no_mangle]
pub unsafe extern "C" fn OSK_SetKeys(mut reason: ::core::ffi::c_int) -> LRESULT {
    let mut wKey: [WCHAR; 2] = [0; 2];
    let mut bKeyStates: [BYTE; 256] = [0; 256];
    let mut szKey: LPCWSTR = core::ptr::null();
    let mut Keys: PKEY = core::ptr::null_mut();
    let mut uVirtKey: UINT = 0;
    let mut LedPos: POINT = core::mem::zeroed();
    let mut LedSize: SIZE = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;
    let mut yPad: ::core::ffi::c_int = 0;

    /* Get key states before doing anything */
    if GetKeyboardState(core::ptr::addr_of_mut!(bKeyStates) as *mut BYTE) == 0 {
        DPRINT!(b"OSK_SetKeys(): GetKeyboardState() call failed.\n\0".as_ptr() as *const i8);
        return -1;
    }

    let mut fall: bool = false;
    match (reason) as u32 {
        SETKEYS_LANG => {
            /* Keyboard language/caps change, just update the button texts */
            Keys = (*Globals.Keyboard).Keys;
            i = 0 as ::core::ffi::c_int;
            while i < (*Globals.Keyboard).KeyCount {
                if !((*Keys.offset(i as isize)).translate == 0) {
                    uVirtKey = MapVirtualKeyW(
                        ((*Keys.offset(i as isize)).scancode & (SCANCODE_MASK as INT_PTR)) as UINT,
                        MAPVK_VSC_TO_VK as UINT,
                    );

                    if ToUnicode(
                        uVirtKey,
                        ((*Keys.offset(i as isize)).scancode & (SCANCODE_MASK as INT_PTR)) as UINT,
                        core::ptr::addr_of_mut!(bKeyStates) as *mut BYTE,
                        core::ptr::addr_of_mut!(wKey) as LPWSTR,
                        _countof!(wKey) as ::core::ffi::c_int,
                        0 as UINT,
                    ) >= 1 as ::core::ffi::c_int
                    {
                        szKey = core::ptr::addr_of_mut!(wKey) as *mut WCHAR as LPCWSTR;
                    } else {
                        szKey = (*Keys.offset(i as isize)).name;
                    }

                    /* Only one & the button will try to underline the next character... */
                    if wcsncmp(
                        szKey,
                        RUSTCONV_WSTR_01AE8BCA3FED.as_ptr(),
                        1 as usize,
                    ) == 0 as ::core::ffi::c_int
                    {
                        szKey = RUSTCONV_WSTR_03C602A9F6CE.as_ptr();
                    }

                    SetWindowTextW(*Globals.hKeys.offset(i as isize), szKey);
                }
                i += 1;
            }
            return 0;
        }
        SETKEYS_LAYOUT => {
            /* Clear up current layout before applying a different one */
            OSK_DestroyKeys();
            fall = true;
        }
        SETKEYS_INIT => {
            fall = true;
        }
        _ => {}
    }

    /* Fallthrough */
    if fall {
        if Globals.bIsEnhancedKeyboard != 0 {
            Globals.Keyboard = core::ptr::addr_of_mut!(EnhancedKeyboard) as PKEYBOARD_STRUCT;
        } else {
            Globals.Keyboard = core::ptr::addr_of_mut!(StandardKeyboard) as PKEYBOARD_STRUCT;
        }

        Globals.hKeys = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (core::mem::size_of::<HWND>() as SIZE_T).wrapping_mul((*Globals.Keyboard).KeyCount as SIZE_T),
        ) as *mut HWND;

        if Globals.hKeys.is_null() {
            DPRINT!(b"OSK_SetKeys(): Failed to allocate memory for button handles.\n\0".as_ptr() as *const i8);
            return -1;
        }

        Keys = (*Globals.Keyboard).Keys;

        /* Create key buttons */
        i = 0 as ::core::ffi::c_int;
        while i < (*Globals.Keyboard).KeyCount {
            uVirtKey = MapVirtualKeyW(
                ((*Keys.offset(i as isize)).scancode & (SCANCODE_MASK as INT_PTR)) as UINT,
                MAPVK_VSC_TO_VK as UINT,
            );

            if (*Keys.offset(i as isize)).translate != 0
                && ToUnicode(
                    uVirtKey,
                    ((*Keys.offset(i as isize)).scancode & (SCANCODE_MASK as INT_PTR)) as UINT,
                    core::ptr::addr_of_mut!(bKeyStates) as *mut BYTE,
                    core::ptr::addr_of_mut!(wKey) as LPWSTR,
                    _countof!(wKey) as ::core::ffi::c_int,
                    0 as UINT,
                ) >= 1 as ::core::ffi::c_int
            {
                szKey = core::ptr::addr_of_mut!(wKey) as *mut WCHAR as LPCWSTR;
            } else {
                szKey = (*Keys.offset(i as isize)).name;
            }

            *Globals.hKeys.offset(i as isize) = CreateWindowW!(
                WC_BUTTONW as LPCWSTR,
                szKey,
                (WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON | ((*Keys.offset(i as isize)).flags) as u32) as DWORD,
                (*Keys.offset(i as isize)).x as ::core::ffi::c_int,
                (*Keys.offset(i as isize)).y as ::core::ffi::c_int,
                (*Keys.offset(i as isize)).cx as ::core::ffi::c_int,
                (*Keys.offset(i as isize)).cy as ::core::ffi::c_int,
                Globals.hMainWnd,
                (*Keys.offset(i as isize)).scancode as HMENU,
                Globals.hInstance,
                core::ptr::null_mut(),
            );
            if !Globals.hFont.is_null() {
                SendMessageW(
                    *Globals.hKeys.offset(i as isize),
                    WM_SETFONT as UINT,
                    Globals.hFont as WPARAM,
                    0 as LPARAM,
                );
            }
            i += 1;
        }

        /* Add additional padding for caption and menu */
        yPad = GetSystemMetrics((SM_CYSIZE) as i32) + GetSystemMetrics(SM_CYMENU as ::core::ffi::c_int);
        /* Size window according to layout */
        SetWindowPos(
            Globals.hMainWnd,
            if Globals.bAlwaysOnTop != 0 {
                HWND_TOPMOST as HWND
            } else {
                HWND_NOTOPMOST as HWND
            },
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*Globals.Keyboard).Size.cx as ::core::ffi::c_int,
            ((*Globals.Keyboard).Size.cy + yPad as LONG) as ::core::ffi::c_int,
            SWP_NOMOVE as UINT,
        );

        /* Create LEDs */
        LedPos = (*Globals.Keyboard).LedStart;
        LedSize = (*Globals.Keyboard).LedSize;

        CreateWindowW!(
            WC_STATICW as LPCWSTR,
            RUSTCONV_WSTR_96A296D224F2.as_ptr(),
            (WS_VISIBLE | WS_CHILD | SS_CENTER | SS_NOTIFY) as DWORD,
            LedPos.x as ::core::ffi::c_int,
            LedPos.y as ::core::ffi::c_int,
            LedSize.cx as ::core::ffi::c_int,
            LedSize.cy as ::core::ffi::c_int,
            Globals.hMainWnd,
            IDC_LED_NUM as HMENU,
            Globals.hInstance,
            core::ptr::null_mut(),
        );

        LedPos.x += (*Globals.Keyboard).LedGap as LONG;

        CreateWindowW!(
            WC_STATICW as LPCWSTR,
            RUSTCONV_WSTR_96A296D224F2.as_ptr(),
            (WS_VISIBLE | WS_CHILD | SS_CENTER | SS_NOTIFY) as DWORD,
            LedPos.x as ::core::ffi::c_int,
            LedPos.y as ::core::ffi::c_int,
            LedSize.cx as ::core::ffi::c_int,
            LedSize.cy as ::core::ffi::c_int,
            Globals.hMainWnd,
            IDC_LED_CAPS as HMENU,
            Globals.hInstance,
            core::ptr::null_mut(),
        );

        LedPos.x += (*Globals.Keyboard).LedGap as LONG;

        CreateWindowW!(
            WC_STATICW as LPCWSTR,
            RUSTCONV_WSTR_96A296D224F2.as_ptr(),
            (WS_VISIBLE | WS_CHILD | SS_CENTER | SS_NOTIFY) as DWORD,
            LedPos.x as ::core::ffi::c_int,
            LedPos.y as ::core::ffi::c_int,
            LedSize.cx as ::core::ffi::c_int,
            LedSize.cy as ::core::ffi::c_int,
            Globals.hMainWnd,
            IDC_LED_SCROLL as HMENU,
            Globals.hInstance,
            core::ptr::null_mut(),
        );

        /* Set system keys text */
        OSK_SetText((SCAN_CODE_110 as i32), (IDS_ESCAPE as i32));
        OSK_SetText((SCAN_CODE_124 as i32), (IDS_PRN as i32));
        OSK_SetText((SCAN_CODE_125 as i32), (IDS_STOP as i32));
        OSK_SetText((SCAN_CODE_126 as i32), (IDS_ATTN as i32));
        OSK_SetText((SCAN_CODE_90 as i32), (IDS_NUMLOCKKEY as i32));
        OSK_SetText((SCAN_CODE_75 as i32), (IDS_INSERT as i32));
        OSK_SetText((SCAN_CODE_76 as i32), (IDS_DELETE as i32));
        OSK_SetText((SCAN_CODE_81 as i32), (IDS_END as i32));
        OSK_SetText((SCAN_CODE_58 as i32), (IDS_CTRL as i32)); /* Left ctrl */
        OSK_SetText((SCAN_CODE_64 as i32), (IDS_CTRL as i32)); /* Right ctrl */
        OSK_SetText((SCAN_CODE_60 as i32), (IDS_LEFTALT as i32));
        OSK_SetText((SCAN_CODE_62 as i32), (IDS_RIGHTALT as i32));

        /* Set icon on visual buttons */
        OSK_SetImage((SCAN_CODE_15 as i32), (IDI_BACK as i32));
        OSK_SetImage((SCAN_CODE_16 as i32), (IDI_TAB as i32));
        OSK_SetImage((SCAN_CODE_30 as i32), (IDI_CAPS_LOCK as i32));
        OSK_SetImage((SCAN_CODE_43 as i32), (IDI_RETURN as i32));
        OSK_SetImage((SCAN_CODE_44 as i32), (IDI_SHIFT as i32));
        OSK_SetImage((SCAN_CODE_57 as i32), (IDI_SHIFT as i32));
        OSK_SetImage((SCAN_CODE_127 as i32), (IDI_REACTOS as i32));
        OSK_SetImage((SCAN_CODE_128 as i32), (IDI_REACTOS as i32));
        OSK_SetImage((SCAN_CODE_129 as i32), (IDI_MENU as i32));
        OSK_SetImage((SCAN_CODE_80 as i32), (IDI_HOME as i32));
        OSK_SetImage((SCAN_CODE_85 as i32), (IDI_PG_UP as i32));
        OSK_SetImage((SCAN_CODE_86 as i32), (IDI_PG_DOWN as i32));
        OSK_SetImage((SCAN_CODE_79 as i32), (IDI_LEFT as i32));
        OSK_SetImage((SCAN_CODE_83 as i32), (IDI_TOP as i32));
        OSK_SetImage((SCAN_CODE_84 as i32), (IDI_BOTTOM as i32));
        OSK_SetImage((SCAN_CODE_89 as i32), (IDI_RIGHT as i32));
    }

    if reason != SETKEYS_INIT as ::core::ffi::c_int {
        ShowWindow(Globals.hMainWnd, SW_SHOW as ::core::ffi::c_int);
        UpdateWindow(Globals.hMainWnd);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_Create(hwnd: HWND) -> LRESULT {
    let mut monitor: HMONITOR = core::ptr::null_mut();
    let mut info: MONITORINFO = core::mem::zeroed();
    let mut Pt: POINT = core::mem::zeroed();
    let mut rcWindow: RECT = core::mem::zeroed();
    let mut rcDlgIntersect: RECT = core::mem::zeroed();
    let mut lf: LOGFONTW = core::mem::zeroed();

    /* Save handle */
    Globals.hMainWnd = hwnd;

    /* Init Font */
    lf.lfHeight = Globals.FontHeight;
    StringCchCopyW(core::ptr::addr_of_mut!(lf.lfFaceName) as *mut WCHAR, (_countof!(Globals.FontFaceName)) as _, core::ptr::addr_of_mut!(Globals.FontFaceName) as *mut WCHAR);
    Globals.hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));

    if OSK_SetKeys(SETKEYS_INIT as core::ffi::c_int) == -1 {
        return -1;
    }

    /* Check the checked menu item before displaying the window */
    if Globals.bIsEnhancedKeyboard != 0 {
        /* Enhanced keyboard dialog chosen, set the respective menu item as checked */
        CheckMenuItem(GetMenu(hwnd), IDM_ENHANCED_KB, MF_BYCOMMAND | MF_CHECKED);
        CheckMenuItem(GetMenu(hwnd), IDM_STANDARD_KB, MF_BYCOMMAND | MF_UNCHECKED);
    } else {
        /* Standard keyboard dialog chosen, set the respective menu item as checked */
        CheckMenuItem(GetMenu(hwnd), IDM_STANDARD_KB, MF_BYCOMMAND | MF_CHECKED);
        CheckMenuItem(GetMenu(hwnd), IDM_ENHANCED_KB, MF_BYCOMMAND | MF_UNCHECKED);
    }

    /* Check if the "Click Sound" option was chosen before (and if so, then tick the menu item) */
    if Globals.bSoundClick != 0 {
        CheckMenuItem(GetMenu(hwnd), IDM_CLICK_SOUND, MF_BYCOMMAND | MF_CHECKED);
    }

    /* Get screen info */
    memset(core::ptr::addr_of_mut!(Pt) as *mut core::ffi::c_void, 0, core::mem::size_of::<POINT>());
    monitor = MonitorFromPoint(Pt, MONITOR_DEFAULTTOPRIMARY);
    info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(monitor, core::ptr::addr_of_mut!(info));
    GetWindowRect(hwnd, core::ptr::addr_of_mut!(rcWindow));

    /*
        If the coordination values are default then re-initialize using the specific formulas
        to move the dialog at the bottom of the screen.
    */
    if Globals.PosX == CW_USEDEFAULT && Globals.PosY == CW_USEDEFAULT {
        Globals.PosX = (info.rcMonitor.left + info.rcMonitor.right - (rcWindow.right - rcWindow.left)) / 2;
        Globals.PosY = info.rcMonitor.bottom - (rcWindow.bottom - rcWindow.top);
    }

    /*
        Calculate the intersection of two rectangle sources (dialog and work desktop area).
        If such sources do not intersect, then the dialog is deemed as "off screen".
    */
    if IntersectRect(core::ptr::addr_of_mut!(rcDlgIntersect), core::ptr::addr_of_mut!(rcWindow), core::ptr::addr_of_mut!(info.rcWork)) == 0 {
        Globals.PosX = (info.rcMonitor.left + info.rcMonitor.right - (rcWindow.right - rcWindow.left)) / 2;
        Globals.PosY = info.rcMonitor.bottom - (rcWindow.bottom - rcWindow.top);
    } else {
        /*
            There's still some intersection but we're not for sure if it is sufficient (the dialog could also be partially hidden).
            Therefore, check the remaining intersection if it's enough.
        */
        if rcWindow.top < info.rcWork.top || rcWindow.left < info.rcWork.left || rcWindow.right > info.rcWork.right || rcWindow.bottom > info.rcWork.bottom {
            Globals.PosX = (info.rcMonitor.left + info.rcMonitor.right - (rcWindow.right - rcWindow.left)) / 2;
            Globals.PosY = info.rcMonitor.bottom - (rcWindow.bottom - rcWindow.top);
        }
    }

    /*
        Place the window (with respective placement coordinates) as topmost, above
        every window which are not on top or are at the bottom of the Z order.
    */
    if Globals.bAlwaysOnTop != 0 {
        CheckMenuItem(GetMenu(hwnd), IDM_ON_TOP, MF_BYCOMMAND | MF_CHECKED);
        SetWindowPos(hwnd, HWND_TOPMOST, Globals.PosX, Globals.PosY, 0, 0, SWP_NOSIZE);
    } else {
        CheckMenuItem(GetMenu(hwnd), IDM_ON_TOP, MF_BYCOMMAND | MF_UNCHECKED);
        SetWindowPos(hwnd, HWND_NOTOPMOST, Globals.PosX, Globals.PosY, 0, 0, SWP_NOSIZE);
    }

    /* Create a green brush for leds */
    Globals.hBrushGreenLed = CreateSolidBrush(RGB!(0, 255, 0));

    /* Set a timer for periodic tasks */
    Globals.iTimer = SetTimer(hwnd, 0, 100, None);

    /* If the member of the struct (bShowWarning) is set then display the dialog box */
    if Globals.bShowWarning != 0 {
        /* If for whatever reason the thread fails to be created then handle the dialog box in main thread... */
        if CreateThread(core::ptr::null_mut(), 0, Some(OSK_WarningDlgThread), Globals.hInstance as PVOID, 0, core::ptr::null_mut()) == core::ptr::null_mut() {
            DialogBoxW!(Globals.hInstance, MAKEINTRESOURCEW!(IDD_WARNINGDIALOG_OSK), Globals.hMainWnd, Some(OSK_WarningProc));
        }
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_Close() -> i32 {
    KillTimer(Globals.hMainWnd, Globals.iTimer);

    /* Release Ctrl, Shift, Alt keys */
    OSK_ReleaseKey((SCAN_CODE_44) as WORD); // Left shift
    OSK_ReleaseKey((SCAN_CODE_57) as WORD); // Right shift
    OSK_ReleaseKey((SCAN_CODE_58) as WORD); // Left ctrl
    OSK_ReleaseKey((SCAN_CODE_60) as WORD); // Left alt
    OSK_ReleaseKey((SCAN_CODE_62) as WORD); // Right alt
    OSK_ReleaseKey((SCAN_CODE_64) as WORD); // Right ctrl

    /* Destroy child controls */
    OSK_DestroyKeys();

    /* delete GDI objects */
    if !Globals.hBrushGreenLed.is_null() {
        DeleteObject(Globals.hBrushGreenLed as HGDIOBJ);
    }
    if !Globals.hFont.is_null() {
        DeleteObject(Globals.hFont as HGDIOBJ);
    }

    /* Save the application's settings on registry */
    SaveSettings();

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_RefreshLEDKeys() {
    let mut i: INT = 0 as INT;
    let mut bKeyIsPressed: BOOL = 0;
    while (i as usize) < _countof!(LedKey) as usize {
        bKeyIsPressed = ((GetAsyncKeyState(LedKey[i as usize].vKey as INT) as INT & 0x8000 as INT) != 0 as INT) as BOOL;
        if LedKey[i as usize].bWasKeyPressed != bKeyIsPressed {
            LedKey[i as usize].bWasKeyPressed = bKeyIsPressed;
            InvalidateRect(
                GetDlgItem(Globals.hMainWnd, LedKey[i as usize].DlgResource as INT),
                core::ptr::null_mut(),
                FALSE as BOOL,
            );
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn OSK_Timer() -> ::core::ffi::c_int {
    let mut hWndActiveWindow: HWND = core::ptr::null_mut();
    let mut dwThread: DWORD = 0;
    let mut hKeyboardLayout: HKL = core::ptr::null_mut();

    hWndActiveWindow = GetForegroundWindow();
    if !hWndActiveWindow.is_null() && hWndActiveWindow != Globals.hMainWnd {
        /* Grab the current keyboard layout from the foreground window */
        dwThread = GetWindowThreadProcessId(hWndActiveWindow, core::ptr::null_mut());
        hKeyboardLayout = GetKeyboardLayout(dwThread);
        /* Activate the layout */
        ActivateKeyboardLayout(hKeyboardLayout, 0 as UINT);
    }

    /*
        Update the LED key indicators accordingly to their state (if one
        of the specific keys is held down).
    */
    OSK_RefreshLEDKeys();
    /* Update the buttons */
    OSK_SetKeys(SETKEYS_LANG as ::core::ffi::c_int);

    return TRUE as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_ChooseFont() {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut cf: CHOOSEFONTW = core::mem::zeroed();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut i: core::ffi::c_int;

    StringCchCopyW(
        core::ptr::addr_of_mut!(lf.lfFaceName) as *mut WCHAR,
        (_countof!(Globals.FontFaceName)) as _,
        core::ptr::addr_of_mut!(Globals.FontFaceName) as *mut WCHAR,
    );
    lf.lfHeight = Globals.FontHeight;

    cf.lStructSize = core::mem::size_of::<CHOOSEFONTW>() as DWORD;
    cf.hwndOwner = Globals.hMainWnd;
    cf.lpLogFont = core::ptr::addr_of_mut!(lf);
    cf.Flags = (CF_INITTOLOGFONTSTRUCT | CF_NOSTYLESEL) as DWORD;

    if ChooseFontW(core::ptr::addr_of_mut!(cf)) == 0 {
        return;
    }

    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));

    if hFont.is_null() {
        return;
    }

    /* Set font information */
    StringCchCopyW(
        core::ptr::addr_of_mut!(Globals.FontFaceName) as *mut WCHAR,
        (_countof!(Globals.FontFaceName)) as _,
        core::ptr::addr_of_mut!(lf.lfFaceName) as *mut WCHAR,
    );
    Globals.FontHeight = lf.lfHeight;

    hOldFont = Globals.hFont;
    Globals.hFont = hFont;

    i = 0 as core::ffi::c_int;
    while i < (*Globals.Keyboard).KeyCount {
        SendMessageW(
            *Globals.hKeys.offset(i as isize),
            WM_SETFONT as UINT,
            Globals.hFont as WPARAM,
            TRUE as LPARAM,
        );
        i += 1;
    }

    DeleteObject(hOldFont as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn OSK_Command(wCommand: WPARAM, hWndControl: HWND) -> BOOL {
    let mut ScanCode: WORD = 0;
    let mut Input: INPUT = core::mem::zeroed();
    let mut bExtendedKey: BOOL = 0;
    let mut bKeyDown: BOOL = 0;
    let mut bKeyUp: BOOL = 0;
    let mut WindowStyle: LONG = 0;
    let mut i: INT = 0;

    /* KeyDown and/or KeyUp ? */
    WindowStyle = GetWindowLongW(hWndControl, GWL_STYLE);
    if ((WindowStyle as u32 & BS_AUTOCHECKBOX)) == BS_AUTOCHECKBOX {
        /* 2-states key like Shift, Alt, Ctrl, ... */
        if (SendMessageW(hWndControl, BM_GETCHECK, 0, 0)) as u32 == (BST_CHECKED) as _ {
            bKeyDown = (TRUE) as i32;
            bKeyUp = (FALSE) as i32;
        } else {
            bKeyDown = (FALSE) as i32;
            bKeyUp = (TRUE) as i32;
        }
    } else {
        /* Other key */
        bKeyDown = (TRUE) as i32;
        bKeyUp = (TRUE) as i32;
    }

    /* Get the key from dialog control key command */
    ScanCode = wCommand as WORD;

    /*
        The user could've pushed one of the key buttons of the dialog that
        can trigger particular function toggling (Caps Lock, Num Lock or Scroll Lock). Update
        (invalidate) the LED icon resources accordingly.
    */
    i = 0;
    while i < _countof!(LedKey) as INT {
        if LedKey[i as usize].wScanCode == ScanCode {
            InvalidateRect(
                GetDlgItem(Globals.hMainWnd, LedKey[i as usize].DlgResource),
                core::ptr::null_mut(),
                (FALSE) as BOOL,
            );
        }
        i += 1;
    }

    /* Extended key ? */
    if (ScanCode as INT & 0x0200) != 0 {
        bExtendedKey = (TRUE) as i32;
    } else {
        bExtendedKey = (FALSE) as i32;
    }
    ScanCode &= (SCANCODE_MASK as u16);

    /* Press and release the key */
    if bKeyDown != 0 {
        Input.type_ = INPUT_KEYBOARD;
        Input.__bindgen_anon_1.ki.wVk = 0;
        Input.__bindgen_anon_1.ki.wScan = ScanCode;
        Input.__bindgen_anon_1.ki.time = GetTickCount();
        Input.__bindgen_anon_1.ki.dwExtraInfo = GetMessageExtraInfo() as _;
        Input.__bindgen_anon_1.ki.dwFlags = KEYEVENTF_SCANCODE;
        if bExtendedKey != 0 {
            Input.__bindgen_anon_1.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }
        SendInput(
            1,
            core::ptr::addr_of_mut!(Input),
            core::mem::size_of::<INPUT>() as INT,
        );
    }

    if bKeyUp != 0 {
        Input.type_ = INPUT_KEYBOARD;
        Input.__bindgen_anon_1.ki.wVk = 0;
        Input.__bindgen_anon_1.ki.wScan = ScanCode;
        Input.__bindgen_anon_1.ki.time = GetTickCount();
        Input.__bindgen_anon_1.ki.dwExtraInfo = GetMessageExtraInfo() as _;
        Input.__bindgen_anon_1.ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if bExtendedKey != 0 {
            Input.__bindgen_anon_1.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }
        SendInput(
            1,
            core::ptr::addr_of_mut!(Input),
            core::mem::size_of::<INPUT>() as INT,
        );
    }

    /* Play the sound during clicking event (only if "Use Click Sound" menu option is ticked) */
    if Globals.bSoundClick != 0 {
        PlaySoundW(
            MAKEINTRESOURCEW!(IDI_SOUNDCLICK),
            (GetModuleHandle(core::ptr::null_mut())).cast(),
            SND_RESOURCE | SND_ASYNC,
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_ReleaseKey(mut ScanCode: WORD) -> BOOL {
    let mut Input: INPUT = core::mem::zeroed();
    let mut bExtendedKey: BOOL = 0;
    let mut WindowStyle: LONG = 0;
    let mut hWndControl: HWND = core::ptr::null_mut();

    /* Is it a 2-states key ? */
    hWndControl = GetDlgItem(Globals.hMainWnd, ScanCode as ::core::ffi::c_int);
    WindowStyle = GetWindowLongW(hWndControl, GWL_STYLE);
    if (WindowStyle & (BS_AUTOCHECKBOX as LONG)) != (BS_AUTOCHECKBOX as LONG) {
        return (FALSE) as BOOL;
    }

    /* Is the key down ? */
    if SendMessageW(
        hWndControl,
        BM_GETCHECK as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) != (BST_CHECKED as LRESULT)
    {
        return (TRUE) as BOOL;
    }

    /* Extended key ? */
    if (ScanCode as ::core::ffi::c_int & 0x0200) != 0 {
        bExtendedKey = (TRUE) as i32;
    } else {
        bExtendedKey = (FALSE) as i32;
    }
    ScanCode = (ScanCode as ::core::ffi::c_int & (SCANCODE_MASK as i32)) as WORD;

    /* Release the key */
    Input.type_ = INPUT_KEYBOARD as DWORD;
    Input.__bindgen_anon_1.ki.wVk = 0 as WORD;
    Input.__bindgen_anon_1.ki.wScan = ScanCode;
    Input.__bindgen_anon_1.ki.time = GetTickCount();
    Input.__bindgen_anon_1.ki.dwExtraInfo = GetMessageExtraInfo() as ULONG_PTR;
    Input.__bindgen_anon_1.ki.dwFlags = (KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP) as DWORD;
    if bExtendedKey != 0 {
        Input.__bindgen_anon_1.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY as DWORD;
    }
    SendInput(
        1 as UINT,
        core::ptr::addr_of_mut!(Input),
        core::mem::size_of::<INPUT>() as ::core::ffi::c_int,
    );

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OSK_Paint(hwnd: HWND) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut rcText: RECT = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut szTemp: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    let hdc: HDC = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));

    if !Globals.hFont.is_null() {
        hOldFont = SelectObject(hdc, Globals.hFont as HGDIOBJ) as HFONT;
    }

    rcText.left = (*Globals.Keyboard).LedTextStart.x;
    rcText.top = (*Globals.Keyboard).LedTextStart.y;
    rcText.right = rcText.left + (*Globals.Keyboard).LedTextSize.cx;
    rcText.bottom = rcText.top + (*Globals.Keyboard).LedTextSize.cy;

    LoadStringW(
        Globals.hInstance,
        IDS_NUMLOCK as UINT,
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        _countof!(szTemp) as core::ffi::c_int,
    );
    DrawTextW(
        hdc,
        core::ptr::addr_of_mut!(szTemp) as *mut WCHAR as LPCWSTR,
        -(1 as core::ffi::c_int),
        core::ptr::addr_of_mut!(rcText),
        DT_NOCLIP as UINT,
    );

    OffsetRect(
        core::ptr::addr_of_mut!(rcText),
        (*Globals.Keyboard).LedTextOffset as core::ffi::c_int,
        0 as core::ffi::c_int,
    );

    LoadStringW(
        Globals.hInstance,
        IDS_CAPSLOCK as UINT,
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        _countof!(szTemp) as core::ffi::c_int,
    );
    DrawTextW(
        hdc,
        core::ptr::addr_of_mut!(szTemp) as *mut WCHAR as LPCWSTR,
        -(1 as core::ffi::c_int),
        core::ptr::addr_of_mut!(rcText),
        DT_NOCLIP as UINT,
    );

    OffsetRect(
        core::ptr::addr_of_mut!(rcText),
        (*Globals.Keyboard).LedTextOffset as core::ffi::c_int,
        0 as core::ffi::c_int,
    );

    LoadStringW(
        Globals.hInstance,
        IDS_SCROLLLOCK as UINT,
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        _countof!(szTemp) as core::ffi::c_int,
    );
    DrawTextW(
        hdc,
        core::ptr::addr_of_mut!(szTemp) as *mut WCHAR as LPCWSTR,
        -(1 as core::ffi::c_int),
        core::ptr::addr_of_mut!(rcText),
        DT_NOCLIP as UINT,
    );

    if !hOldFont.is_null() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }

    EndPaint(hwnd, core::ptr::addr_of_mut!(ps));

    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "system" fn OSK_WndProc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            return OSK_Create(hwnd);
        }

        WM_PAINT => {
            return OSK_Paint(hwnd);
        }

        WM_TIMER => {
            return OSK_Timer() as LRESULT;
        }

        WM_CTLCOLORSTATIC => {
            if (lParam as HWND) == GetDlgItem(hwnd, (IDC_LED_NUM as i32)) {
                if (GetKeyState((VK_NUMLOCK as i32)) as ::core::ffi::c_int & 0x0001) != 0 {
                    return Globals.hBrushGreenLed as LRESULT;
                } else {
                    return GetStockObject((BLACK_BRUSH as i32)) as LRESULT;
                }
            }
            if (lParam as HWND) == GetDlgItem(hwnd, (IDC_LED_CAPS as i32)) {
                if (GetKeyState((VK_CAPITAL as i32)) as ::core::ffi::c_int & 0x0001) != 0 {
                    return Globals.hBrushGreenLed as LRESULT;
                } else {
                    return GetStockObject((BLACK_BRUSH as i32)) as LRESULT;
                }
            }
            if (lParam as HWND) == GetDlgItem(hwnd, (IDC_LED_SCROLL as i32)) {
                if (GetKeyState((VK_SCROLL as i32)) as ::core::ffi::c_int & 0x0001) != 0 {
                    return Globals.hBrushGreenLed as LRESULT;
                } else {
                    return GetStockObject((BLACK_BRUSH as i32)) as LRESULT;
                }
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDM_EXIT => {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }

                IDM_ENHANCED_KB => {
                    if Globals.bIsEnhancedKeyboard == 0 {
                        /*
                            The user attempted to switch to enhanced keyboard dialog type.
                            Set the member value as TRUE, destroy the dialog and save the data configuration into the registry.
                        */
                        Globals.bIsEnhancedKeyboard = (TRUE) as i32;
                        SaveSettings();

                        /* Change the condition of enhanced keyboard item menu to checked */
                        CheckMenuItem(GetMenu(hwnd), IDM_ENHANCED_KB, MF_BYCOMMAND | MF_CHECKED);
                        CheckMenuItem(GetMenu(hwnd), IDM_STANDARD_KB, MF_BYCOMMAND | MF_UNCHECKED);

                        /* Finally, update the key layout */
                        LoadSettings();
                        OSK_SetKeys((SETKEYS_LAYOUT) as i32);
                    }
                }

                IDM_STANDARD_KB => {
                    if Globals.bIsEnhancedKeyboard != 0 {
                        /*
                            The user attempted to switch to standard keyboard dialog type.
                            Set the member value as FALSE, destroy the dialog and save the data configuration into the registry.
                        */
                        Globals.bIsEnhancedKeyboard = (FALSE) as i32;
                        SaveSettings();

                        /* Change the condition of standard keyboard item menu to checked */
                        CheckMenuItem(GetMenu(hwnd), IDM_ENHANCED_KB, MF_BYCOMMAND | MF_UNCHECKED);
                        CheckMenuItem(GetMenu(hwnd), IDM_STANDARD_KB, MF_BYCOMMAND | MF_CHECKED);

                        /* Finally, update the key layout */
                        LoadSettings();
                        OSK_SetKeys((SETKEYS_LAYOUT) as i32);
                    }
                }

                IDM_CLICK_SOUND => {
                    /*
                        This case is triggered when the user attempts to click on the menu item. Before doing anything,
                        we must check the condition state of such menu item so that we can tick/untick the menu item accordingly.
                    */
                    if Globals.bSoundClick == 0 {
                        Globals.bSoundClick = (TRUE) as i32;
                        CheckMenuItem(GetMenu(hwnd), IDM_CLICK_SOUND, MF_BYCOMMAND | MF_CHECKED);
                    } else {
                        Globals.bSoundClick = (FALSE) as i32;
                        CheckMenuItem(GetMenu(hwnd), IDM_CLICK_SOUND, MF_BYCOMMAND | MF_UNCHECKED);
                    }
                }

                IDM_ON_TOP => {
                    /*
                        Check the condition state before disabling/enabling the menu
                        item and change the topmost order.
                    */
                    if Globals.bAlwaysOnTop == 0 {
                        Globals.bAlwaysOnTop = (TRUE) as i32;
                        CheckMenuItem(GetMenu(hwnd), IDM_ON_TOP, MF_BYCOMMAND | MF_CHECKED);
                        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
                    } else {
                        Globals.bAlwaysOnTop = (FALSE) as i32;
                        CheckMenuItem(GetMenu(hwnd), IDM_ON_TOP, MF_BYCOMMAND | MF_UNCHECKED);
                        SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
                    }
                }

                IDM_FONT => {
                    OSK_ChooseFont();
                }

                IDM_ABOUT => {
                    OSK_About();
                }

                _ => {
                    OSK_Command(wParam, lParam as HWND);
                }
            }
            return 0 as LRESULT;
        }

        WM_THEMECHANGED => {
            /* Redraw the dialog (and its control buttons) using the new theme */
            InvalidateRect(hwnd, core::ptr::null::<RECT>(), (FALSE) as BOOL);
            return 0 as LRESULT;
        }

        WM_CLOSE => {
            OSK_Close();
            PostQuitMessage(0);
            return 0 as LRESULT;
        }

        _ => {}
    }
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    prev: HINSTANCE,
    cmdline: LPWSTR,
    show: core::ffi::c_int,
) -> core::ffi::c_int {
    let mut dwError: DWORD;
    let mut hMutex: HANDLE;
    let mut iccex: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut hwnd: HWND;

    UNREFERENCED_PARAMETER!(prev);
    UNREFERENCED_PARAMETER!(cmdline);
    UNREFERENCED_PARAMETER!(show);

    /*
        Obtain a mutex for the program. This will ensure that
        the program is launched only once.
    */
    hMutex = CreateMutexW(core::ptr::null_mut(), (FALSE) as BOOL, RUSTCONV_WSTR_10AAF8759FB4.as_ptr());

    if !hMutex.is_null() {
        /* Check if there's already a mutex for the program */
        dwError = GetLastError();

        if dwError == ERROR_ALREADY_EXISTS {
            /*
                A mutex with the object name has been created previously.
                Therefore, another instance is already running.
            */
            DPRINT!("wWinMain(): Failed to create a mutex! The program instance is already running.\n");
            CloseHandle(hMutex);
            return 0;
        }
    }

    /* Load the common controls */
    iccex.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccex.dwICC = (ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccex));

    core::ptr::write_bytes(core::ptr::addr_of_mut!(Globals) as *mut u8, 0, core::mem::size_of::<OSK_GLOBALS>());
    Globals.hInstance = hInstance;

    /* Load the application's settings from the registry */
    LoadSettings();

    /* Define the window class */
    wc.cbSize        = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.hInstance     = Globals.hInstance;
    wc.lpfnWndProc   = Some(OSK_WndProc);
    wc.lpszMenuName  = MAKEINTRESOURCEW!(IDR_OSK_MENU) as LPCWSTR;
    wc.lpszClassName = (OSK_CLASS).as_ptr();
    wc.style         = (CS_HREDRAW | CS_VREDRAW) as UINT;
    wc.hbrBackground = ((COLOR_WINDOW + 1) as usize) as HBRUSH;
    /* Set the application's icon */
    wc.hIcon = LoadImageW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IDI_OSK) as LPCWSTR,
        IMAGE_ICON,
        0,
        0,
        (LR_SHARED | LR_DEFAULTSIZE) as UINT,
    ) as HICON;
    wc.hIconSm = CopyImage(
        wc.hIcon as HANDLE,
        IMAGE_ICON as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        LR_COPYFROMRESOURCE as UINT,
    ) as HICON;

    'quit: {
        if RegisterClassExW(core::ptr::addr_of_mut!(wc)) == 0 {
            break 'quit;
        }

        /* Load window title */
        LoadStringW(
            Globals.hInstance,
            IDS_OSK as UINT,
            addr_of_mut!(Globals.szTitle) as LPWSTR,
            _countof!(Globals.szTitle) as core::ffi::c_int,
        );

        hwnd = CreateWindowExW(
            (WS_EX_TOPMOST | WS_EX_APPWINDOW | WS_EX_NOACTIVATE) as DWORD,
            (OSK_CLASS).as_ptr(),
            addr_of_mut!(Globals.szTitle) as LPCWSTR,
            (WS_SYSMENU | WS_MINIMIZEBOX) as DWORD,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Globals.hInstance,
            core::ptr::null_mut(),
        );

        if hwnd.is_null() {
            break 'quit;
        }

        ShowWindow(hwnd, (SW_SHOW) as i32);
        UpdateWindow(hwnd);

        while GetMessageW(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0 as UINT,
            0 as UINT,
        ) != 0
        {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    /* Delete the mutex */
    if !hMutex.is_null() {
        CloseHandle(hMutex);
    }

    return 0;
}
