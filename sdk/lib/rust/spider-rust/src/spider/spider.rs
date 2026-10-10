use crate::prelude::*;
macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut szHelpPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
pub static mut dwAppStartTime: DWORD = 0;
#[no_mangle]
pub static mut hwndMain: HWND = core::ptr::null_mut();
pub static mut hInstance: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut szAppName: [TCHAR; 128] = [0; 128];
pub static mut MsgQuit: [TCHAR; 128] = [0; 128];
pub static mut MsgAbout: [TCHAR; 128] = [0; 128];
#[no_mangle]
pub static mut MsgWin: [TCHAR; 128] = [0; 128];
#[no_mangle]
pub static mut MsgDeal: [TCHAR; 128] = [0; 128];
#[no_mangle]
pub static mut dwDifficulty: DWORD = 0;
#[no_mangle]
pub static mut SpiderWnd: *mut CARDWINDOW = core::ptr::null_mut();

#[repr(C)]
pub struct _CardBack {
    pub hSelf: HWND,
    pub hOldProc: WNDPROC,
    pub hdcNum: INT,
    pub imgNum: INT,
    pub bSelected: BOOL,
}
pub type CARDBACK = _CardBack;
pub type PCARDBACK = *mut CARDBACK;

// WndProc: forward prototype (LRESULT __stdcall WndProc(HWND, UINT, WPARAM, LPARAM)); provided by the function pass.
#[no_mangle]
pub unsafe extern "C" fn MakePath(szDest: *mut TCHAR, nDestLen: UINT, szExt: *const TCHAR) {
    let mut ptr: *mut TCHAR;

    ptr = szDest.offset(GetModuleFileName(GetModuleHandle(core::ptr::null()), szDest, nDestLen) as isize - 1 as isize);
    loop {
        let c = *ptr;
        ptr = ptr.offset(-1);
        if c == '.' as TCHAR {
            break;
        }
    }
    lstrcpy(ptr.offset(1 as isize), szExt);
}
#[no_mangle]
pub unsafe extern "system" fn DifficultyDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            CheckRadioButton(hDlg, (IDC_DIF_ONECOLOR as i32), (IDC_DIF_FOURCOLORS as i32), (IDC_DIF_ONECOLOR as i32));
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDOK => {
                    if IsDlgButtonChecked(hDlg, (IDC_DIF_ONECOLOR as i32)) == BST_CHECKED {
                        dwDifficulty = IDC_DIF_ONECOLOR;
                    } else if IsDlgButtonChecked(hDlg, (IDC_DIF_TWOCOLORS as i32)) == BST_CHECKED {
                        dwDifficulty = IDC_DIF_TWOCOLORS;
                    } else if IsDlgButtonChecked(hDlg, (IDC_DIF_FOURCOLORS as i32)) == BST_CHECKED {
                        dwDifficulty = IDC_DIF_FOURCOLORS;
                    }

                    NewGame();
                    EndDialog(hDlg, (TRUE) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }
                IDCANCEL => {
                    EndDialog(hDlg, (FALSE) as INT_PTR);
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
pub unsafe extern "system" fn wWinMain(hInst: HINSTANCE, hPrev: HINSTANCE, szCmdLine: LPTSTR, iCmdShow: i32) -> i32 {
    let mut hwnd: HWND = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut wndclass: WNDCLASS = core::mem::zeroed();
    let mut ice: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut hAccelTable: HACCEL = core::mem::zeroed();

    hInstance = hInst;

    /* Load application title */
    LoadString(hInst, IDS_SPI_NAME, addr_of_mut!(szAppName) as LPTSTR, (core::mem::size_of_val(&szAppName) / core::mem::size_of_val(&szAppName[0])) as i32);
    /* Load MsgBox() texts here to avoid loading them many times later */
    LoadString(hInst, IDS_SPI_ABOUT, addr_of_mut!(MsgAbout) as LPTSTR, (core::mem::size_of_val(&MsgAbout) / core::mem::size_of_val(&MsgAbout[0])) as i32);
    LoadString(hInst, IDS_SPI_QUIT, addr_of_mut!(MsgQuit) as LPTSTR, (core::mem::size_of_val(&MsgQuit) / core::mem::size_of_val(&MsgQuit[0])) as i32);
    LoadString(hInst, IDS_SPI_WIN, addr_of_mut!(MsgWin) as LPTSTR, (core::mem::size_of_val(&MsgWin) / core::mem::size_of_val(&MsgWin[0])) as i32);
    LoadString(hInst, IDS_SPI_DEAL, addr_of_mut!(MsgDeal) as LPTSTR, (core::mem::size_of_val(&MsgDeal) / core::mem::size_of_val(&MsgDeal[0])) as i32);

    /* Window class for the main application parent window */
    wndclass.style             = 0;
    wndclass.lpfnWndProc       = Some(WndProc);
    wndclass.cbClsExtra        = 0;
    wndclass.cbWndExtra        = 0;
    wndclass.hInstance         = hInst;
    wndclass.hIcon             = LoadIcon (hInst, MAKEINTRESOURCE!(IDI_SPIDER));
    wndclass.hCursor           = LoadCursor (core::ptr::null_mut(), IDC_ARROW);
    wndclass.hbrBackground     = core::ptr::null_mut() as HBRUSH;
    wndclass.lpszMenuName      = MAKEINTRESOURCE!(IDR_MENU1) as _;
    wndclass.lpszClassName     = addr_of_mut!(szAppName) as LPTSTR;

    RegisterClass(core::ptr::addr_of_mut!(wndclass));

    ice.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as u32;
    ice.dwICC = ICC_BAR_CLASSES;
    InitCommonControlsEx(core::ptr::addr_of_mut!(ice));

    srand(GetTickCount() as u32);

    /* Load the card artwork out of cards.dll. Nothing below here can draw
       without it, so a failure is fatal rather than a blank table. */
    if CardLib_Init() == 0
    {
        static FAIL_TEXT: [u16; 47] = [
            'F' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
            't' as u16, 'o' as u16, ' ' as u16,
            'l' as u16, 'o' as u16, 'a' as u16, 'd' as u16, ' ' as u16,
            't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
            'c' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16,
            'i' as u16, 'm' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 's' as u16, ' ' as u16,
            'f' as u16, 'r' as u16, 'o' as u16, 'm' as u16, ' ' as u16,
            'c' as u16, 'a' as u16, 'r' as u16, 'd' as u16, 's' as u16, '.' as u16,
            'd' as u16, 'l' as u16, 'l' as u16, '.' as u16,
            0,
        ];
        MessageBox(core::ptr::null_mut(), FAIL_TEXT.as_ptr() as *const TCHAR,
                   addr_of_mut!(szAppName) as LPTSTR, MB_OK | MB_ICONERROR);
        return 1;
    }

    /* Construct the path to our help file */
    static HLP_EXT: [u16; 5] = ['.' as u16, 'h' as u16, 'l' as u16, 'p' as u16, 0];
    MakePath(addr_of_mut!(szHelpPath) as *mut TCHAR, MAX_PATH, HLP_EXT.as_ptr() as *const TCHAR);

    hwnd = CreateWindow!(addr_of_mut!(szAppName) as LPTSTR,
                        addr_of_mut!(szAppName) as LPTSTR,
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        0,  /*The real size will be computed in WndProc through WM_GETMINMAXINFO */
                        0,  /* The real size will be computed in WndProc through WM_GETMINMAXINFO */
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        hInst,
                        core::ptr::null_mut());

    hwndMain = hwnd;

    ShowWindow(hwnd, iCmdShow);
    UpdateWindow(hwnd);

    hAccelTable = LoadAccelerators(hInstance, MAKEINTRESOURCE!(IDR_ACCELERATOR1));

    DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_DIFFICULTY), hwnd, DifficultyDlgProc);

    while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0
    {
        if TranslateAccelerator(hwnd, hAccelTable, core::ptr::addr_of_mut!(msg)) == 0
        {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }
    return msg.wParam as i32;
}

#[no_mangle]
pub unsafe extern "system" fn CardImageWndProc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pCardBack: PCARDBACK = GetWindowLongPtr(hwnd, GWLP_USERDATA) as PCARDBACK;
    static mut hOldProc: WNDPROC = None;

    if pCardBack.is_null() {
        return (FALSE) as LRESULT;
    }

    if hOldProc.is_none() {
        hOldProc = (*pCardBack).hOldProc;
    }

    match msg {
        WM_PAINT => {
            let mut hdc: HDC = core::mem::zeroed();
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hPen: HPEN = core::mem::zeroed();
            let mut hOldPen: HPEN = core::mem::zeroed();
            let mut hBrush: HBRUSH = core::mem::zeroed();
            let mut hOldBrush: HBRUSH = core::mem::zeroed();
            let mut rc: RECT = core::mem::zeroed();

            hdc = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps)) as HDC;

            if (*pCardBack).bSelected != 0 {
                hPen = CreatePen((PS_SOLID as i32), 2, RGB!(0, 0, 0)) as HPEN;
            } else {
                let Face: DWORD = GetSysColor((COLOR_3DFACE) as i32);
                hPen = CreatePen((PS_SOLID as i32), 2, Face) as HPEN;
            }

            GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
            hBrush = GetStockObject((NULL_BRUSH as i32)) as HBRUSH;
            hOldPen = SelectObject(hdc, (hPen).cast()) as HPEN;
            hOldBrush = SelectObject(hdc, (hBrush).cast()) as HBRUSH;

            Rectangle(hdc, rc.left + 1, rc.top + 1, rc.right, rc.bottom);

            StretchBlt(
                hdc,
                2,
                2,
                (CARDBACK_OPTIONS_WIDTH as i32),
                (CARDBACK_OPTIONS_HEIGHT as i32),
                CardLib_CardBitmapsDC(),
                (*pCardBack).hdcNum * CardLib_CardWidth(),
                0,
                CardLib_CardWidth(),
                CardLib_CardHeight(),
                SRCCOPY,
            );

            SelectObject(hdc, (hOldPen).cast());
            SelectObject(hdc, (hOldBrush).cast());

            EndPaint(hwnd, core::ptr::addr_of_mut!(ps));

            // break;
        }

        WM_LBUTTONDOWN => {
            (*pCardBack).bSelected = if (*pCardBack).bSelected != 0 {
                (FALSE) as i32
            } else {
                (TRUE) as i32
            };
            // break;
        }

        _ => {}
    }

    return CallWindowProc(hOldProc, hwnd, msg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "system" fn CardBackDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut pCardBacks: PCARDBACK = core::ptr::null_mut();

    'sw: {
        if uMsg == WM_INITDIALOG {
            let mut i: INT;
            let mut c: INT;
            let size: SIZE_T = (core::mem::size_of::<CARDBACK>() * (NUM_CARDBACKS as usize)) as SIZE_T;

            pCardBacks = HeapAlloc(GetProcessHeap(), 0, size) as PCARDBACK;

            if pCardBacks.is_null() {
                return (FALSE) as INT_PTR;
            }

            i = 0;
            c = (CARDBACK_START as i32);
            while c <= (CARDBACK_END as i32) {
                (*pCardBacks.offset(i as isize)).hSelf = GetDlgItem(hDlg, c);
                (*pCardBacks.offset(i as isize)).bSelected = (FALSE) as i32;
                (*pCardBacks.offset(i as isize)).hdcNum = (CARDBACK_RES_START + (i) as u32) as i32;
                (*pCardBacks.offset(i as isize)).imgNum = i + 1;
                (*pCardBacks.offset(i as isize)).hOldProc = core::mem::transmute(SetWindowLongPtr(
                    (*pCardBacks.offset(i as isize)).hSelf,
                    GWLP_WNDPROC,
                    CardImageWndProc as usize as LONG_PTR,
                ));

                SetWindowLongPtr(
                    (*pCardBacks.offset(i as isize)).hSelf,
                    GWLP_USERDATA,
                    pCardBacks.offset(i as isize) as LONG_PTR,
                );

                i += 1;
                c += 1;
            }

            return (TRUE) as INT_PTR;
        } else if uMsg == WM_COMMAND {
            if LOWORD!(wParam) == IDOK || LOWORD!(wParam) == IDCANCEL {
                let mut i: INT;
                let mut num: INT = 0;
                i = 0;
                while i < (NUM_CARDBACKS as i32) {
                    if (*pCardBacks.offset(i as isize)).bSelected != 0 {
                        num = (*pCardBacks.offset(i as isize)).imgNum;
                    }
                    i += 1;
                }

                EndDialog(
                    hDlg,
                    if LOWORD!(wParam) == IDOK { (num) as _ } else { (FALSE) as _ },
                );
                HeapFree(GetProcessHeap(), 0, pCardBacks as LPVOID);
                return (TRUE) as INT_PTR;
            }

            if (HIWORD!(wParam)) as u32 == STN_CLICKED {
                let mut i: INT;
                let mut rc: RECT = core::mem::zeroed();
                i = 0;
                while i < (NUM_CARDBACKS as i32) {
                    (*pCardBacks.offset(i as isize)).bSelected =
                        ((*pCardBacks.offset(i as isize)).hSelf == lParam as HWND) as BOOL;

                    GetClientRect((*pCardBacks.offset(i as isize)).hSelf, core::ptr::addr_of_mut!(rc));
                    InvalidateRect(
                        (*pCardBacks.offset(i as isize)).hSelf,
                        core::ptr::addr_of_mut!(rc),
                        (TRUE) as BOOL,
                    );
                    i += 1;
                }

                break 'sw;
            }
        }
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn ShowDeckOptionsDlg(hwnd: HWND) {
    let mut cardBack: i32;

    cardBack = DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_CARDBACK), hwnd, CardBackDlgProc) as i32;
    if cardBack != 0 {
        CardWindow_SetBackCardIdx(SpiderWnd, CARDBACK_RES_START + ((cardBack - 1)) as u32);
        CardWindow_Redraw(SpiderWnd);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WndProc(hwnd: HWND, iMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    static mut nWidth: i32 = 0;
    static mut nHeight: i32 = 0;

    match iMsg {
        WM_CREATE => {
            // For now, the Help dialog is disabled because of lacking of HTML Help support
            EnableMenuItem(GetMenu(hwnd), IDM_HELP_CONTENTS, MF_BYCOMMAND | MF_GRAYED);

            SpiderWnd = CardWindow_Create(hwnd, 0, WS_CHILD | WS_VISIBLE, 0, 0, 100, 100);
            dwDifficulty = IDC_DIF_ONECOLOR;
            CreateSpider();

            SetWindowPos(hwnd, core::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOZORDER);

            dwAppStartTime = GetTickCount();

            return 0;
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        }

        WM_SIZE => {
            nWidth = LOWORD!(lParam) as i32;
            nHeight = HIWORD!(lParam) as i32;

            MoveWindow(CardWindow_GetHwnd(SpiderWnd), 0, 0, nWidth, nHeight, (TRUE) as BOOL);
            return 0;
        }

        WM_GETMINMAXINFO => {
            let mmi: *mut MINMAXINFO;

            mmi = lParam as *mut MINMAXINFO;
            (*mmi).ptMinTrackSize.x = (NUM_STACKS * (CardLib_CardWidth()) as u32 + (NUM_STACKS + 3) * X_BORDER + 12) as i32; // Border left and right of 6px
            (*mmi).ptMinTrackSize.y = GetSystemMetrics((SM_CYCAPTION) as i32) +
                                    GetSystemMetrics((SM_CYMENU) as i32) +
                                    (2 * Y_BORDER) as i32 +
                                    3 * CardLib_CardHeight() +
                                    6 * yRowStackCardOffset;
            return 0;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDM_GAME_NEW => {
                    NewGame();
                    return 0;
                }

                IDM_GAME_DECK => {
                    ShowDeckOptionsDlg(hwnd);
                    return 0;
                }

                IDM_HELP_CONTENTS => {
                    WinHelp(hwnd, core::ptr::addr_of_mut!(szHelpPath) as LPTSTR, HELP_CONTENTS, 0);//HELP_KEY, (DWORD)"How to play");
                    return 0;
                }

                IDM_HELP_ABOUT => {
                    ShellAbout(hwnd, core::ptr::addr_of_mut!(szAppName) as LPTSTR, core::ptr::addr_of_mut!(MsgAbout) as LPTSTR,
                               LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SPIDER)));
                    return 0;
                }

                IDM_GAME_EXIT => {
                    PostMessage(hwnd, WM_CLOSE, 0, 0);
                    return 0;
                }

                _ => {}
            }

            return 0;
        }

        WM_CLOSE => {
            if fGameStarted == false {
                DestroyWindow(hwnd);
                return 0;
            } else {
                let ret: i32;

                ret = MessageBox(hwnd, core::ptr::addr_of_mut!(MsgQuit) as LPTSTR, core::ptr::addr_of_mut!(szAppName) as LPTSTR, MB_YESNO | MB_ICONQUESTION);
                if (ret) as u32 == IDYES {
                    WinHelp(hwnd, core::ptr::addr_of_mut!(szHelpPath) as LPTSTR, HELP_QUIT, 0);
                    DestroyWindow(hwnd);
                }
            }
            return 0;
        }

        _ => {}
    }
    return DefWindowProc(hwnd, iMsg, wParam, lParam);
}
