use crate::mmc::misc::AllocAndLoadString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_77065B4E456D: [u16; 10] = [0x004Du16, 0x0044, 0x0049, 0x0043, 0x004C, 0x0049, 0x0045, 0x004E, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_9FB14923561E: [u16; 4] = [0x006Du16, 0x0073, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_AC9237AEBB82: [u16; 17] = [
    0x004Du16, 0x0053, 0x0043, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000,
    0x002A, 0x002E, 0x006D, 0x0073, 0x0063, 0x0000, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szMMCMainFrame: [TCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [TCHAR; 13]>(*b"M\0M\0C\0M\0a\0i\0n\0F\0r\0a\0m\0e\0\0\0")
};

static mut szMMCChildFrm: [TCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [TCHAR; 12]>(*b"M\0M\0C\0C\0h\0i\0l\0d\0F\0r\0m\0\0\0")
};

static mut NewConsoleCount: ULONG = 0 as ULONG;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CONSOLE_MAINFRAME_WND {
    pub hwnd: HWND,
    pub hwndMDIClient: HWND,
    pub hMenuConsoleSmall: HMENU,
    pub hMenuConsoleLarge: HMENU,
    pub nConsoleCount: INT,
    pub __bindgen_anon_1: CONSOLE_MAINFRAME_WND__bindgen_ty_1,
}
pub type PCONSOLE_MAINFRAME_WND = *mut CONSOLE_MAINFRAME_WND;

#[derive(Copy, Clone)]
#[repr(C)]
pub union CONSOLE_MAINFRAME_WND__bindgen_ty_1 {
    pub Flags: DWORD,
    pub __bindgen_anon_1: CONSOLE_MAINFRAME_WND__bindgen_ty_1__bindgen_ty_1,
}

// bit 0: AppAuthorMode (1-bit bitfield)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CONSOLE_MAINFRAME_WND__bindgen_ty_1__bindgen_ty_1 {
    pub _bitfield_1: DWORD,
}

#[repr(C)]
pub struct CONSOLE_CHILDFRM_WND {
    pub hwnd: HWND,
    pub MainFrame: PCONSOLE_MAINFRAME_WND,
    pub pFileName: PTSTR,
}
pub type PCONSOLE_CHILDFRM_WND = *mut CONSOLE_CHILDFRM_WND;
#[no_mangle]
pub unsafe extern "C" fn GetActiveChildInfo() -> PCONSOLE_CHILDFRM_WND {
    let mut hWndMDIChild: HWND;
    hWndMDIChild = SendMessageW(
        hwndMDIClient,
        WM_MDIGETACTIVE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HWND;
    if hWndMDIChild.is_null() {
        return core::ptr::null_mut();
    }
    return GetWindowLongPtr(hWndMDIChild, 0 as ::core::ffi::c_int) as PCONSOLE_CHILDFRM_WND;
}

unsafe extern "C" fn CreateNewConsoleTitle() -> LPTSTR {
    let mut lpTitle: LPTSTR = core::ptr::null_mut();
    NewConsoleCount = NewConsoleCount.wrapping_add(1);
    if LoadAndFormatString(
        hAppInstance,
        IDS_CONSOLETITLE,
        core::ptr::addr_of_mut!(lpTitle),
        NewConsoleCount,
    ) == 0
    {
        lpTitle = core::ptr::null_mut();
    }

    return lpTitle;
}
#[no_mangle]
pub unsafe extern "C" fn CreateNewMDIChild(
    Info: PCONSOLE_MAINFRAME_WND,
    hwnd_mdi_client: HWND,
) -> HWND {
    let mut mcs: MDICREATESTRUCT = core::mem::zeroed();
    let mut hChild: HWND;

    mcs.szTitle = (CreateNewConsoleTitle() as LPCTSTR).cast();
    mcs.szClass = (core::ptr::addr_of!(szMMCChildFrm) as LPCTSTR).cast();
    mcs.hOwner = GetModuleHandle(core::ptr::null_mut()) as HANDLE;
    mcs.x = CW_USEDEFAULT;
    mcs.cx = mcs.x;
    mcs.y = CW_USEDEFAULT;
    mcs.cy = mcs.y;
    mcs.style = MDIS_ALLCHILDSTYLES;

    hChild = SendMessage(
        hwnd_mdi_client,
        WM_MDICREATE,
        0,
        core::ptr::addr_of_mut!(mcs) as LPARAM,
    ) as HWND;
    if !hChild.is_null() {
        (*Info).nConsoleCount += 1;
    }

    return hChild;
}

unsafe extern "C" fn FrameOnCreate(hwnd: HWND, lParam: LPARAM) -> LRESULT {
    let mut Info: PCONSOLE_MAINFRAME_WND;
    let mut ccs: CLIENTCREATESTRUCT = core::mem::zeroed();
    let lpFileName: LPCTSTR = (*(lParam as LPCREATESTRUCT)).lpCreateParams as LPCTSTR;

    Info = HeapAlloc(
        hAppHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<CONSOLE_MAINFRAME_WND>()) as SIZE_T,
    ) as PCONSOLE_MAINFRAME_WND;
    if Info.is_null() {
        return -(1) as LRESULT;
    }

    (*Info).hwnd = hwnd;

    SetWindowLongPtr(hwnd, 0, Info as LONG_PTR);

    (*Info).hMenuConsoleSmall = LoadMenu(
        hAppInstance,
        MAKEINTRESOURCE!(IDM_CONSOLE_SMALL) as LPCTSTR,
    );

    (*Info).hMenuConsoleLarge = LoadMenu(
        hAppInstance,
        MAKEINTRESOURCE!(IDM_CONSOLE_LARGE) as LPCTSTR,
    );

    if lpFileName.is_null() {
        /* FIXME */
        (*Info).__bindgen_anon_1.Flags |= 1; // AppAuthorMode = TRUE: bit 0 of the union's Flags
//        Info->lpConsoleTitle = (&[0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004f, 0x0053, 0x0020, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x006d, 0x0065, 0x006e, 0x0074, 0x0020, 0x0043, 0x006f, 0x006e, 0x0073, 0x006f, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "ReactOS Management Console" */;
    } else {
        (*Info).__bindgen_anon_1.Flags |= 1; // AppAuthorMode = TRUE: bit 0 of the union's Flags
//        Info->lpConsoleTitle = CreateNewConsoleTitle();
    }

    SetMenu((*Info).hwnd, (*Info).hMenuConsoleSmall);

    SetWindowText((*Info).hwnd, (&[0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004f, 0x0053, 0x0020, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x006d, 0x0065, 0x006e, 0x0074, 0x0020, 0x0043, 0x006f, 0x006e, 0x0073, 0x006f, 0x006c, 0x0065, 0x0000u16]).as_ptr() /* TEXT "ReactOS Management Console" */);

    ccs.hWindowMenu = GetSubMenu((*Info).hMenuConsoleLarge, 1) as _;
    ccs.idFirstChild = IDM_MDI_FIRSTCHILD;

    /* Create the MDI client window */
    hwndMDIClient = CreateWindowEx(
        WS_EX_CLIENTEDGE,
        RUSTCONV_WSTR_77065B4E456D.as_ptr(),
        core::ptr::null_mut(),
        (WS_CHILD | WS_CLIPCHILDREN | WS_VSCROLL | WS_HSCROLL | WS_VISIBLE | WS_BORDER) as DWORD,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hwnd,
        0xCAC as HMENU,
        hAppInstance,
        core::ptr::addr_of_mut!(ccs) as LPVOID,
    );

    ShowWindow((*Info).hwndMDIClient, (SW_SHOW) as i32);

    return 0;
}

unsafe extern "C" fn SetFileName(Info: PCONSOLE_CHILDFRM_WND, pFileName: PWSTR) {
    DPRINT1!(b"SetFileName(&p '%S')\n\0".as_ptr() as *const i8, Info, pFileName);

    if !(*Info).pFileName.is_null() {
        HeapFree(GetProcessHeap(), 0, ((*Info).pFileName).cast());
        (*Info).pFileName = core::ptr::null_mut();
    }

    if !pFileName.is_null() {
        (*Info).pFileName = HeapAlloc(
            GetProcessHeap(),
            0,
            ((_tcslen(pFileName) + 1) * core::mem::size_of::<TCHAR>()) as SIZE_T,
        ) as PTSTR;
        if !(*Info).pFileName.is_null() {
            _tcscpy((*Info).pFileName, pFileName);
        }
    }
}

unsafe extern "C" fn DoSaveFile(hWnd: HWND, pChildInfo: PCONSOLE_CHILDFRM_WND) -> BOOL {
    if DbgPrint(
        b"(%s:%d) pChildInfo %p\n\0".as_ptr() as PCSTR,
        b"console.c\0".as_ptr() as *const i8,
        212 as i32,
        pChildInfo,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"console.c\0".as_ptr() as *const i8,
            212 as i32,
        );
    }
    if DbgPrint(
        b"(%s:%d) FileName %S\n\0".as_ptr() as PCSTR,
        b"console.c\0".as_ptr() as *const i8,
        214 as i32,
        (*pChildInfo).pFileName,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"console.c\0".as_ptr() as *const i8,
            214 as i32,
        );
    }
    if (*pChildInfo).pFileName.is_null() {
        return DoSaveFileAs(hWnd, pChildInfo);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn DoSaveFileAs(hWnd: HWND, pChildInfo: PCONSOLE_CHILDFRM_WND) -> BOOL {
    let mut saveas: OPENFILENAME = core::mem::zeroed();
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if DbgPrint(
        b"(%s:%d) pChildInfo %p\n\0".as_ptr() as PCSTR,
        b"base/applications/mmc/console.c\0"
            .as_ptr() as *const core::ffi::c_char,
        232 as core::ffi::c_int,
        pChildInfo,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"base/applications/mmc/console.c\0"
                .as_ptr() as *const core::ffi::c_char,
            232 as core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) FileName %S\n\0".as_ptr() as PCSTR,
        b"base/applications/mmc/console.c\0"
            .as_ptr() as *const core::ffi::c_char,
        233 as core::ffi::c_int,
        (*pChildInfo).pFileName,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"base/applications/mmc/console.c\0"
                .as_ptr() as *const core::ffi::c_char,
            233 as core::ffi::c_int,
        );
    }

    core::ptr::write_bytes(core::ptr::addr_of_mut!(saveas), 0, 1);

    if !(*pChildInfo).pFileName.is_null() {
        StringCbCopy(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>(),
            (*pChildInfo).pFileName,
        );
    } else {
        GetWindowText(
            (*pChildInfo).hwnd,
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            (core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>()
                / core::mem::size_of::<TCHAR>()) as i32,
        );
        let DotMsc: [u16; 5] = ['.' as u16, 'm' as u16, 's' as u16, 'c' as u16, 0];
        StringCbCat(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>(),
            core::ptr::addr_of!(DotMsc) as *const TCHAR,
        );
    }

    saveas.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    saveas.hwndOwner = hWnd;
    saveas.hInstance = hAppInstance;
    saveas.lpstrFilter = RUSTCONV_WSTR_AC9237AEBB82.as_ptr();
    saveas.lpstrFile = core::ptr::addr_of_mut!(szPath) as *mut TCHAR;
    saveas.nMaxFile = MAX_PATH as DWORD;
    saveas.Flags = (OFN_EXPLORER as DWORD)
        | (OFN_HIDEREADONLY as DWORD)
        | (OFN_PATHMUSTEXIST as DWORD)
        | (OFN_OVERWRITEPROMPT as DWORD);
    saveas.lpstrDefExt = RUSTCONV_WSTR_9FB14923561E.as_ptr();

    if GetSaveFileName(core::ptr::addr_of_mut!(saveas)) != 0 {
        /* HACK: Because in ROS, Save-As boxes don't check the validity
         * of file names and thus, here, szPath can be invalid !! We only
         * see its validity when we call DoSaveFile()... */
        SetFileName(pChildInfo, core::ptr::addr_of_mut!(szPath) as PWSTR);

        if DoSaveFile(hWnd, pChildInfo) != 0 {
            // UpdateWindowCaption();
            return (TRUE) as BOOL;
        } else {
            SetFileName(pChildInfo, core::ptr::null_mut());
            return (FALSE) as BOOL;
        }
    } else {
        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn FrameOnSave(hWnd: HWND) -> BOOL {
    let mut pChildInfo: PCONSOLE_CHILDFRM_WND = core::ptr::null_mut();
    pChildInfo = GetActiveChildInfo();
    if pChildInfo.is_null() {
        return (FALSE) as BOOL;
    }

    return DoSaveFile(hWnd, pChildInfo);
}

unsafe extern "C" fn FrameOnSaveAs(hWnd: HWND) -> BOOL {
    let mut pChildInfo: PCONSOLE_CHILDFRM_WND;

    pChildInfo = GetActiveChildInfo();
    if pChildInfo.is_null() {
        return (FALSE) as BOOL;
    }

    return DoSaveFileAs(hWnd, pChildInfo);
}

unsafe extern "C" fn FrameOnCommand(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) {
    let mut Info: PCONSOLE_MAINFRAME_WND = core::ptr::null_mut();
    let mut hChild: HWND = core::ptr::null_mut();
    let mut lpTitle: LPTSTR = core::ptr::null_mut();

    Info = GetWindowLongPtr(hwnd, 0) as PCONSOLE_MAINFRAME_WND;

    match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
        IDM_FILE_NEW => {
            CreateNewMDIChild(Info, hwndMDIClient);
            SetMenu((*Info).hwnd, (*Info).hMenuConsoleLarge);
        }
        IDM_FILE_SAVE => {
            FrameOnSave(hwnd);
        }
        IDM_FILE_SAVEAS => {
            FrameOnSaveAs(hwnd);
        }
        IDM_FILE_EXIT => {
            PostMessage(hwnd, WM_CLOSE, 0, 0);
        }
        IDM_HELP_ABOUT => {
            if AllocAndLoadString(core::ptr::addr_of_mut!(lpTitle), hAppInstance, IDS_APPTITLE) != 0 {
                ShellAbout(
                    hwnd,
                    lpTitle,
                    core::ptr::null_mut(),
                    LoadIcon(hAppInstance, MAKEINTRESOURCE!(IDI_MAINAPP)),
                );
                LocalFree((lpTitle).cast());
            }
        }
        _ => {
            if LOWORD!(wParam) as ::core::ffi::c_int >= (IDM_MDI_FIRSTCHILD as i32) {
                DefFrameProc(hwnd, hwndMDIClient, uMsg, wParam, lParam);
            } else {
                hChild = SendMessage(
                    hwndMDIClient,
                    WM_MDIGETACTIVE,
                    0,
                    0,
                ) as HWND;
                if !hChild.is_null() {
                    SendMessage(hChild, WM_COMMAND, wParam, lParam);
                }
            }
        }
    }
}

unsafe extern "C" fn FrameOnSize(hMainWnd: HWND, cx: WORD, cy: WORD) {
    let mut rcClient: RECT = core::mem::zeroed();
    GetClientRect(hMainWnd, core::ptr::addr_of_mut!(rcClient));
    SetWindowPos(
        hwndMDIClient,
        core::ptr::null_mut(),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        rcClient.right as ::core::ffi::c_int,
        rcClient.bottom as ::core::ffi::c_int,
        SWP_NOZORDER as UINT,
    );
}
unsafe extern "system" fn ConsoleMainFrameWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Info: PCONSOLE_MAINFRAME_WND;

    Info = GetWindowLongPtr(hwnd, 0) as PCONSOLE_MAINFRAME_WND;

    if uMsg == WM_CREATE {
        return FrameOnCreate(hwnd, lParam);
    } else if uMsg == WM_COMMAND {
        FrameOnCommand(hwnd, uMsg, wParam, lParam);
    } else if uMsg == WM_SIZE {
        FrameOnSize(hwnd, LOWORD!(lParam), HIWORD!(lParam));
    } else if uMsg == WM_CLOSE {
        DestroyWindow(hwnd);
    } else if uMsg == WM_DESTROY {
        if !Info.is_null() {
            SetMenu((*Info).hwnd, core::ptr::null_mut());

            if !(*Info).hMenuConsoleSmall.is_null() {
                DestroyMenu((*Info).hMenuConsoleSmall);
                (*Info).hMenuConsoleSmall = core::ptr::null_mut();
            }

            if !(*Info).hMenuConsoleLarge.is_null() {
                DestroyMenu((*Info).hMenuConsoleLarge);
                (*Info).hMenuConsoleLarge = core::ptr::null_mut();
            }

            HeapFree(hAppHeap, 0, Info as LPVOID);
        }

        PostQuitMessage(0);
    } else if uMsg == WM_USER_CLOSE_CHILD {
        (*Info).nConsoleCount -= 1;
        if (*Info).nConsoleCount == 0 {
            SetMenu((*Info).hwnd, (*Info).hMenuConsoleSmall);
        }
    } else {
        return DefFrameProc(hwnd, hwndMDIClient, uMsg, wParam, lParam);
    }

    return 0;
}

unsafe extern "system" fn ConsoleChildFrmProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Info: PCONSOLE_CHILDFRM_WND = GetWindowLongPtr(hwnd, 0) as PCONSOLE_CHILDFRM_WND;

    match uMsg {
        WM_CREATE => {
            Info = HeapAlloc(
                hAppHeap,
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<CONSOLE_CHILDFRM_WND>() as SIZE_T,
            ) as PCONSOLE_CHILDFRM_WND;
            if !Info.is_null() {
                (*Info).hwnd = hwnd;

                SetWindowLongPtr(hwnd, 0, Info as LONG_PTR);
            }
        }

        WM_DESTROY => {
            if !Info.is_null() {
                if !(*Info).pFileName.is_null() {
                    HeapFree(hAppHeap, 0, (*Info).pFileName as PVOID);
                }

                HeapFree(hAppHeap, 0, Info as PVOID);
            }

            PostMessage(hwndMainConsole, WM_USER_CLOSE_CHILD, 0, 0);
        }

        _ => return DefMDIChildProc(hwnd, uMsg, wParam, lParam),
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterMMCWndClasses() -> BOOL {
    let mut wc: WNDCLASSEX = core::mem::zeroed();
    let mut Ret: BOOL;

    /* Register the MMCMainFrame window class */
    wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wc.style = 0 as UINT;
    wc.lpfnWndProc = Some(ConsoleMainFrameWndProc);
    wc.cbClsExtra = 0 as i32;
    wc.cbWndExtra = core::mem::size_of::<PCONSOLE_MAINFRAME_WND>() as i32;
    wc.hInstance = hAppInstance;
    wc.hIcon = LoadIcon(
        hAppInstance,
        MAKEINTRESOURCE!(IDI_MAINAPP),
    );
    wc.hCursor = LoadCursor(
        core::ptr::null_mut(),
        MAKEINTRESOURCE!(IDC_ARROW),
    );
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszMenuName = core::ptr::null_mut();
    wc.lpszClassName = (core::ptr::addr_of!(szMMCMainFrame) as *const TCHAR as LPCTSTR).cast();

    Ret = ((RegisterClassEx(core::ptr::addr_of_mut!(wc) as *const _)
        != 0 as ATOM) as i32) as BOOL;

    if Ret != 0 {
        /* Register the MMCChildFrm window class */
        wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
        wc.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
        wc.lpfnWndProc = Some(ConsoleChildFrmProc);
        wc.cbClsExtra = 0 as i32;
        wc.cbWndExtra = core::mem::size_of::<PCONSOLE_CHILDFRM_WND>() as i32;
        wc.hInstance = hAppInstance;
        wc.hIcon = LoadIcon(core::ptr::null_mut(), IDI_APPLICATION);
        wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
        wc.hbrBackground = (COLOR_3DFACE + 1) as HBRUSH;
        wc.lpszMenuName = core::ptr::null_mut();
        wc.lpszClassName = (core::ptr::addr_of!(szMMCChildFrm) as *const TCHAR as LPCTSTR).cast();
        wc.hIconSm = LoadIcon(core::ptr::null_mut(), IDI_APPLICATION);

        Ret = ((RegisterClassEx(core::ptr::addr_of_mut!(wc) as *const _)
            != 0 as ATOM) as i32) as BOOL;
        if Ret == 0 {
            UnregisterClass(
                core::ptr::addr_of!(szMMCMainFrame) as *const TCHAR as LPCTSTR,
                hAppInstance,
            );
        }
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterMMCWndClasses() {
    UnregisterClass(core::ptr::addr_of!(szMMCChildFrm) as LPCTSTR, hAppInstance);
    UnregisterClass(core::ptr::addr_of!(szMMCMainFrame) as LPCTSTR, hAppInstance);
}
#[no_mangle]
pub unsafe extern "C" fn CreateConsoleWindow(lpFileName: LPCTSTR, nCmdShow: ::core::ffi::c_int) -> HWND {
    let mut hWndConsole: HWND = core::ptr::null_mut();
    let mut FileName: LONG_PTR = lpFileName as LONG_PTR;

    hWndConsole = CreateWindowEx(
        (WS_EX_WINDOWEDGE) as DWORD,
        core::ptr::addr_of!(szMMCMainFrame) as LPCTSTR,
        core::ptr::null(),
        ((WS_OVERLAPPEDWINDOW) | (WS_CLIPSIBLINGS) | (WS_CLIPCHILDREN)) as DWORD,
        (CW_USEDEFAULT) as core::ffi::c_int,
        (CW_USEDEFAULT) as core::ffi::c_int,
        (CW_USEDEFAULT) as core::ffi::c_int,
        (CW_USEDEFAULT) as core::ffi::c_int,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hAppInstance,
        FileName as LPVOID,
    );

    if !hWndConsole.is_null() {
        ShowWindow(hWndConsole, nCmdShow);
    }

    return hWndConsole;
}
