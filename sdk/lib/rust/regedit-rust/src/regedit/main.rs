use crate::regedit::childwnd::ChildWndProc;
use crate::regedit::framewnd::FrameWndProc;
use crate::regedit::security::InitializeAclUiDll;
use crate::regedit::settings::LoadSettings;
use crate::regedit::regedit::ProcessCmdLine;
use crate::regedit::hexedit::RegisterHexEditorClass;
use crate::regedit::framewnd::SetupStatusBar;
use crate::regedit::security::UnloadAclUiDll;
use crate::regedit::hexedit::UnregisterHexEditorClass;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_111FD57AB4CB: [u16; 20] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0043, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_50C930882486: [u16; 19] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x004C, 0x004F, 0x0043, 0x0041, 0x004C,
    0x005F, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_5CDE252B65C7: [u16; 14] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0044, 0x0059, 0x004E, 0x005F, 0x0044,
    0x0041, 0x0054, 0x0041, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_87D16031626E: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_9C98935C198E: [u16; 11] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0053,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_9EEBA813F2D9: [u16; 25] = [
    0x0054u16, 0x004F, 0x0044, 0x004F, 0x003A, 0x0020, 0x0053, 0x0045, 0x0054, 0x0020,
    0x0043, 0x004F, 0x0052, 0x0052, 0x0045, 0x0043, 0x0054, 0x0020, 0x0046, 0x004F,
    0x0052, 0x004D, 0x0041, 0x0054, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D0EE0F231465: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0000,
];

#[no_mangle]
pub static mut reg_class_namesW: [*const WCHAR; 6] = unsafe {
    [
        RUSTCONV_C2R_WSTR_50C930882486.as_ptr(),
        RUSTCONV_C2R_WSTR_9C98935C198E.as_ptr(),
        RUSTCONV_C2R_WSTR_87D16031626E.as_ptr(),
        RUSTCONV_C2R_WSTR_111FD57AB4CB.as_ptr(),
        RUSTCONV_C2R_WSTR_D0EE0F231465.as_ptr(),
        RUSTCONV_C2R_WSTR_5CDE252B65C7.as_ptr(),
    ]
};

#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hFrameWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hStatusBar: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hMenuFrame: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut hPopupMenus: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut nClipboardFormat: UINT = 0;

#[no_mangle]
pub static mut strClipboardFormat: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_9EEBA813F2D9.as_ptr() as LPCWSTR
};

#[no_mangle]
pub static mut szTitle: [WCHAR; 100] = [0; 100];

#[no_mangle]
pub static mut szFrameClass: [WCHAR; 100] = [0; 100];

#[no_mangle]
pub static mut szChildClass: [WCHAR; 100] = [0; 100];

pub const MAX_LOADSTRING: usize = 100;
#[no_mangle]
pub unsafe extern "C" fn InitInstance(hInstance: HINSTANCE, nCmdShow: core::ffi::c_int) -> BOOL {
    let mut AclUiAvailable: BOOL = 0;
    let mut hEditMenu: HMENU = core::ptr::null_mut();
    let mut icce: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut wcFrame: WNDCLASSEXW = core::mem::zeroed();
    let mut wcChild: WNDCLASSEXW = core::mem::zeroed();
    let mut hFrameWndClass: ATOM = 0;

    ZeroMemory!(core::ptr::addr_of_mut!(wcFrame) as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSEXW>() as usize);
    wcFrame.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wcFrame.lpfnWndProc =
        Some(FrameWndProc);
    wcFrame.hInstance = hInstance;
    wcFrame.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_REGEDIT) as LPCWSTR);
    wcFrame.hIconSm = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW!(IDI_REGEDIT) as LPCWSTR,
        IMAGE_ICON as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        LR_SHARED as UINT,
    ) as HICON;
    wcFrame.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
    wcFrame.hbrBackground = (COLOR_3DFACE + 1) as HBRUSH;
    wcFrame.lpszClassName = core::ptr::addr_of_mut!(szFrameClass) as LPCWSTR;

    hFrameWndClass = RegisterClassExW(core::ptr::addr_of_mut!(wcFrame)); /* register frame window class */

    ZeroMemory!(core::ptr::addr_of_mut!(wcChild) as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSEXW>() as usize);
    wcChild.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wcChild.lpfnWndProc =
        Some(ChildWndProc);
    wcChild.cbWndExtra = core::mem::size_of::<HANDLE>() as core::ffi::c_int;
    wcChild.hInstance = hInstance;
    wcChild.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_REGEDIT) as LPCWSTR);
    wcChild.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
    wcChild.hbrBackground = (COLOR_3DFACE + 1) as HBRUSH;
    wcChild.lpszClassName = core::ptr::addr_of_mut!(szChildClass) as LPCWSTR;
    wcChild.hIconSm = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW!(IDI_REGEDIT) as LPCWSTR,
        IMAGE_ICON as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        LR_SHARED as UINT,
    ) as HICON;

    RegisterClassExW(core::ptr::addr_of_mut!(wcChild)); /* register child windows class */

    RegisterHexEditorClass(hInstance);

    hMenuFrame = LoadMenuW(hInstance, MAKEINTRESOURCEW!(IDR_REGEDIT_MENU) as LPCWSTR);
    hPopupMenus = LoadMenuW(hInstance, MAKEINTRESOURCEW!(IDR_POPUP_MENUS) as LPCWSTR);

    /* Initialize the Windows Common Controls DLL */
    /* NOTE: Windows sets 0xFFFF to icce.dwICC but we use better value. */
    icce.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    icce.dwICC = (ICC_WIN95_CLASSES | ICC_STANDARD_CLASSES | ICC_USEREX_CLASSES) as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(icce));

    hEditMenu = GetSubMenu(hMenuFrame, 1 as core::ffi::c_int);

    AclUiAvailable = InitializeAclUiDll();
    if AclUiAvailable == 0 {
        /* hide the Edit/Permissions... menu entry */
        if !hEditMenu.is_null() {
            RemoveMenu(hEditMenu, ID_EDIT_PERMISSIONS as UINT, MF_BYCOMMAND as UINT);
            /* remove the separator after the menu item */
            RemoveMenu(hEditMenu, 4 as UINT, MF_BYPOSITION as UINT);
        }
    }

    if !hEditMenu.is_null() {
        SetMenuDefaultItem(hEditMenu, ID_EDIT_MODIFY as UINT, MF_BYCOMMAND as UINT);
    }

    nClipboardFormat = RegisterClipboardFormatW(strClipboardFormat);
    /* if (nClipboardFormat == 0) {
        DWORD dwError = GetLastError();
    } */

    hFrameWnd = CreateWindowExW(
        WS_EX_WINDOWEDGE as DWORD,
        UlongToPtr!(hFrameWndClass as core::ffi::c_ulong) as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_CLIPCHILDREN) as DWORD,
        CW_USEDEFAULT as core::ffi::c_int,
        CW_USEDEFAULT as core::ffi::c_int,
        CW_USEDEFAULT as core::ffi::c_int,
        CW_USEDEFAULT as core::ffi::c_int,
        core::ptr::null_mut(),
        hMenuFrame,
        hInstance,
        core::ptr::null_mut(), /*lpParam*/
    );
    if hFrameWnd.is_null() {
        return (FALSE) as BOOL;
    }

    /* Create the status bar */
    hStatusBar = CreateStatusWindowW(
        (WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | SBT_NOBORDERS) as LONG,
        RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPCWSTR,
        hFrameWnd,
        STATUS_WINDOW as UINT,
    );
    if !hStatusBar.is_null() {
        /* Create the status bar panes */
        SetupStatusBar(hFrameWnd, FALSE as BOOL);
        CheckMenuItem(
            GetSubMenu(hMenuFrame, ID_VIEW_MENU as core::ffi::c_int),
            ID_VIEW_STATUSBAR as UINT,
            (MF_BYCOMMAND | MF_CHECKED) as UINT,
        );
    }

    LoadSettings();
    UpdateWindow(hFrameWnd);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyMainMenu() {
    DestroyMenu(hMenuFrame);
}
#[no_mangle]
pub unsafe extern "C" fn ExitInstance(hInstance: HINSTANCE) {
    UnregisterHexEditorClass(hInstance);

    DestroyMenu(hPopupMenus);
    UnloadAclUiDll();
}

unsafe extern "C" fn InLabelEdit(hWnd: HWND, Msg: UINT) -> BOOL {
    let mut hEdit: HWND = SendMessageW(hWnd, Msg, 0 as WPARAM, 0 as LPARAM) as HWND;
    return (!hEdit.is_null() && IsWindowVisible(hEdit) != 0) as BOOL;
}

unsafe extern "C" fn TranslateChildTabMessage(mut msg: PMSG) -> BOOL {
    if (*msg).message != WM_KEYDOWN as UINT {
        return (FALSE) as BOOL;
    }
    if (*msg).wParam != VK_TAB as WPARAM {
        return (FALSE) as BOOL;
    }
    if GetParent((*msg).hwnd) != (*g_pChildWnd).hWnd {
        return (FALSE) as BOOL;
    }
    PostMessageW(
        hFrameWnd,
        WM_COMMAND as UINT,
        ID_SWITCH_PANELS as WPARAM,
        0 as LPARAM,
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn TranslateRegeditAccelerator(hWnd: HWND, hAccTable: HACCEL, msg: PMSG) -> BOOL {
    if (*msg).message == WM_KEYDOWN {
        if (*msg).wParam == (VK_DELETE) as _ {
            if (*g_pChildWnd).hAddressBarWnd == (*msg).hwnd {
                return (FALSE) as BOOL;
            }
            if InLabelEdit((*g_pChildWnd).hTreeWnd, TVM_GETEDITCONTROL) != 0 || InLabelEdit((*g_pChildWnd).hListWnd, LVM_GETEDITCONTROL) != 0 {
                return (FALSE) as BOOL;
            }
        }
    }
    return TranslateAcceleratorW(hWnd, hAccTable, msg);
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: core::ffi::c_int,
) -> core::ffi::c_int {
    let mut msg: MSG = core::mem::zeroed();
    let mut hAccel: HACCEL = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(hPrevInstance);

    OleInitialize(core::ptr::null_mut());

    /* Initialize global strings */
    LoadStringW(hInstance, IDS_APP_TITLE, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR, ARRAY_SIZE!(szTitle) as core::ffi::c_int);
    LoadStringW(hInstance, IDC_REGEDIT_FRAME, core::ptr::addr_of_mut!(szFrameClass) as *mut WCHAR, ARRAY_SIZE!(szFrameClass) as core::ffi::c_int);
    LoadStringW(hInstance, IDC_REGEDIT, core::ptr::addr_of_mut!(szChildClass) as *mut WCHAR, ARRAY_SIZE!(szChildClass) as core::ffi::c_int);

    if ProcessCmdLine(GetCommandLineW() as *mut WCHAR) != 0 {
        return 0;
    }

    match GetUserDefaultUILanguage() as core::ffi::c_int {
        v if v == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) as core::ffi::c_int => {
            SetProcessDefaultLayout(LAYOUT_RTL);
        }
        _ => {}
    }
    /* Store instance handle in our global variable */
    hInst = hInstance;

    /* Perform application initialization */
    if InitInstance(hInstance, nCmdShow) == 0 {
        return 0;
    }

    hAccel = LoadAcceleratorsW(hInstance, MAKEINTRESOURCEW!(ID_ACCEL));

    /* Main message loop */
    while GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0
    {
        if TranslateRegeditAccelerator(hFrameWnd, hAccel, core::ptr::addr_of_mut!(msg)) == 0 &&
            TranslateChildTabMessage(core::ptr::addr_of_mut!(msg)) == 0
        {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    ExitInstance(hInstance);
    return msg.wParam as core::ffi::c_int;
}
