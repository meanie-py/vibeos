use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_3E31FFE6FE35: [WCHAR; 5] = [0x002E, 0x0070, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_461B74EEBD9E: [WCHAR; 30] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x005C, 0x0048, 0x0065, 0x006C, 0x0070, 0x005C, 0x006D, 0x0073, 0x0070, 0x0061, 0x0069, 0x006E, 0x0074, 0x002E, 0x0063, 0x0068, 0x006D, 0x0000];
pub static CPP_COM_WSTR_67DB42DC8E1B: [WCHAR; 11] = [0x006D, 0x0061, 0x0070, 0x0069, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_773566D1D102: [WCHAR; 7] = [0x0026, 0x0025, 0x0075, 0x0020, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_85D11F0B94E7: [WCHAR; 30] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x006D, 0x0061, 0x0069, 0x006E, 0x0020, 0x0077, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x002E, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_C3F0B98328E3: [WCHAR; 12] = [0x005C, 0x0068, 0x0068, 0x0063, 0x0074, 0x0072, 0x006C, 0x002E, 0x006F, 0x0063, 0x0078, 0x0000];
pub static CPP_COM_WSTR_F7E2D0733EDD: [WCHAR; 4] = [0x0061, 0x0066, 0x0078, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut g_askBeforeEnlarging: BOOL = 0;
#[no_mangle]
pub static mut g_hinstExe: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut g_szFileName: [WCHAR; 512] = [0; 512];
pub static mut g_szMailTempFile: [WCHAR; 512] = [0; 512];
#[no_mangle]
pub static mut g_isAFile: BOOL = 0;
#[no_mangle]
pub static mut g_imageSaved: BOOL = 0;
#[no_mangle]
pub static mut g_showGrid: BOOL = 0;
#[no_mangle]
pub static mut g_hStatusBar: HWND = core::ptr::null_mut();
pub static mut s_hHHCTRL_OCX: HINSTANCE = core::ptr::null_mut();
// FN_HtmlHelpW is a nullable function pointer typedef; NULL initializer maps to None.
pub static mut s_pHtmlHelpW: FN_HtmlHelpW = None;

static mut s_hHHCTRL_OCX_UNUSED_MARKER: u8 = 0;

static OFNHOOKPROC_FILE_TYPE_PLACEHOLDER: () = ();





















#[no_mangle]
pub unsafe extern "C" fn ShowOutOfMemory() {
    let mut szText = [0u16; 256];
    FormatMessageW(FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM,
                     core::ptr::null_mut(),
                     ERROR_OUTOFMEMORY,
                     0,
                     szText.as_mut_ptr(), _countof!(szText) as u32,
                     core::ptr::null_mut());
    mainWindow.__base0.MessageBoxW(szText.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);
}

unsafe extern "C" fn FileExtFromFilter(pExt: LPWSTR, pOFN: *mut OPENFILENAME) -> BOOL {
    let mut pchExt: LPWSTR = pExt;
    *pchExt = 0;

    let mut nIndex: DWORD = 1;
    let mut pch: LPCWSTR = (*pOFN).lpstrFilter;
    while *pch != 0 {
        pch = pch.add((lstrlen(pch) + 1) as usize);
        if (*pOFN).nFilterIndex == nIndex {
            pch = pch.add(1);
            while *pch != 0 && *pch != 0x003Bu16 {
                *pchExt = *pch;
                pchExt = pchExt.add(1);
                pch = pch.add(1);
            }
            *pchExt = 0;
            CharLower(pExt);
            return (TRUE) as BOOL;
        }
        pch = pch.add((wcslen(pch) + 1) as usize);
        nIndex += 1;
    }
    return (FALSE) as BOOL;
}
unsafe extern "system" fn OFNHookProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT_PTR {
    let mut hParent: HWND;
    let mut pon: *mut OFNOTIFYW;
    let mut Path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    match uMsg {
        WM_NOTIFY => {
            pon = lParam as *mut OFNOTIFYW;
            if (*pon).hdr.code == CDN_TYPECHANGE {
                hParent = GetParent(hwnd);
                SendMessageW(hParent, CDM_GETFILEPATH, _countof!(Path) as WPARAM, core::ptr::addr_of_mut!(Path) as *mut WCHAR as LPARAM);
                FileExtFromFilter(PathFindExtensionW(core::ptr::addr_of_mut!(Path) as *mut WCHAR), (*pon).lpOFN);
                SendMessageW(hParent, CDM_SETCONTROLTEXT, (cmb13) as WPARAM, PathFindFileNameW(core::ptr::addr_of_mut!(Path) as *mut WCHAR) as LPARAM);
                StringCchCopyW((*(*pon).lpOFN).lpstrFile, (*(*pon).lpOFN).nMaxFile as usize, core::ptr::addr_of_mut!(Path) as *const WCHAR);
            }
        }
        _ => {}
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn OpenMailer(hWnd: HWND, mut pszPathName: LPCWSTR) -> BOOL {
    // Delete the temporary file if any
    if g_szMailTempFile[0] != 0 {
        DeleteFileW(g_szMailTempFile.as_ptr());
        g_szMailTempFile[0] = UNICODE_NULL;
    }

    let mut strFileTitle = CStringW::new();
    if PathFileExistsW(pszPathName) != 0 && ((imageModel.IsImageSaved()) != 0) {
        strFileTitle = CStringW::from_ptr(PathFindFileNameW(pszPathName));
    } else {
        // Not existing or not saved
        // Get the name of a temporary file
        let mut szTempDir = [0u16; (MAX_PATH) as _];
        GetTempPathW(_countof!(szTempDir), szTempDir.as_mut_ptr());
        if GetTempFileNameW(
            szTempDir.as_ptr(),
            CPP_COM_WSTR_F7E2D0733EDD.as_ptr(),
            0,
            g_szMailTempFile.as_mut_ptr(),
        ) == 0
        {
            return (FALSE) as BOOL; // Failure
        }

        if PathFileExistsW(g_szFileName.as_ptr()) != 0 {
            // Set file title
            strFileTitle = CStringW::from_ptr(PathFindFileNameW(g_szFileName.as_ptr()));

            // Copy to the temporary file
            if CopyFileW(
                g_szFileName.as_ptr(),
                g_szMailTempFile.as_ptr(),
                (FALSE) as BOOL,
            ) == 0
            {
                g_szMailTempFile[0] = UNICODE_NULL;
                return (FALSE) as BOOL; // Failure
            }
        } else {
            // Set file title
            strFileTitle.LoadStringW(IDS_DEFAULTFILENAME);
            strFileTitle += CPP_COM_WSTR_3E31FFE6FE35.as_ptr();

            // Save it to the temporary file
            let hbmLocked: HBITMAP = imageModel.LockBitmap();
            let ret: BOOL = SaveDIBToFile(
                hbmLocked,
                g_szMailTempFile.as_ptr(),
                (FALSE) as BOOL,
                &Gdiplus_ImageFormatPNG,
            );
            imageModel.UnlockBitmap(hbmLocked);
            if ret == 0 {
                g_szMailTempFile[0] = UNICODE_NULL;
                return (FALSE) as BOOL; // Failure
            }
        }

        // Use the temporary file
        pszPathName = g_szMailTempFile.as_ptr();
    }

    // Load "mapi32.dll"
    let hMAPI: HINSTANCE = LoadLibraryW(CPP_COM_WSTR_67DB42DC8E1B.as_ptr());
    if hMAPI.is_null() {
        return (FALSE) as BOOL; // Failure
    }

    // Attachment
    let mut attachmentW: MapiFileDescW = core::mem::zeroed();
    attachmentW.nPosition = (-(1)) as ULONG;
    attachmentW.lpszPathName = pszPathName as LPWSTR;
    attachmentW.lpszFileName = strFileTitle.GetString() as LPWSTR;

    // Message with attachment
    let mut messageW: MapiMessageW = core::mem::zeroed();
    messageW.lpszSubject = core::ptr::null_mut();
    messageW.nFileCount = 1;
    messageW.lpFiles = core::ptr::addr_of_mut!(attachmentW);

    // First, try to open the mailer by the function of Unicode version
    let pMAPISendMailW: FN_MAPISendMailW = core::mem::transmute(
        GetProcAddress(hMAPI, b"MAPISendMailW\0".as_ptr() as *const i8),
    );
    if pMAPISendMailW as usize != 0 {
        pMAPISendMailW(
            0,
            hWnd as ULONG_PTR,
            core::ptr::addr_of_mut!(messageW),
            MAPI_DIALOG | MAPI_LOGON_UI,
            0,
        );
        FreeLibrary(hMAPI);
        return (TRUE) as BOOL; // MAPISendMailW will show an error message on failure
    }

    // Convert to ANSI strings
    let mut szPathNameA = CStringA::from_wide(pszPathName);
    let mut szFileTitleA = CStringA::from_wide(strFileTitle.GetString());

    let mut attachment: MapiFileDesc = core::mem::zeroed();
    attachment.nPosition = (-(1)) as ULONG;
    attachment.lpszPathName = szPathNameA.GetString() as LPSTR;
    attachment.lpszFileName = szFileTitleA.GetString() as LPSTR;

    let mut message: MapiMessage = core::mem::zeroed();
    message.lpszSubject = core::ptr::null_mut();
    message.nFileCount = 1;
    message.lpFiles = core::ptr::addr_of_mut!(attachment);

    // Try again but in ANSI version
    let pMAPISendMail: FN_MAPISendMail = core::mem::transmute(
        GetProcAddress(hMAPI, b"MAPISendMail\0".as_ptr() as *const i8),
    );
    if pMAPISendMail as usize != 0 {
        pMAPISendMail(
            0,
            hWnd as ULONG_PTR,
            core::ptr::addr_of_mut!(message),
            MAPI_DIALOG | MAPI_LOGON_UI,
            0,
        );
        FreeLibrary(hMAPI);
        return (TRUE) as BOOL; // MAPISendMail will show an error message on failure
    }

    FreeLibrary(hMAPI);
    return (FALSE) as BOOL; // Failure
}

unsafe extern "C" fn DoHtmlHelpW(hwndCaller: HWND, pszFile: LPCWSTR, uCommand: UINT, dwData: DWORD_PTR) -> HWND {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if s_hHHCTRL_OCX.is_null() && (uCommand != HH_CLOSE_ALL) {
        // The function loads the system library, not local
        GetSystemDirectoryW(szPath.as_mut_ptr(), _countof!(szPath));
        StringCchCatW(szPath.as_mut_ptr(), (_countof!(szPath)) as _, CPP_COM_WSTR_C3F0B98328E3.as_ptr());
        s_hHHCTRL_OCX = LoadLibraryW(szPath.as_ptr());
        if !s_hHHCTRL_OCX.is_null() {
            s_pHtmlHelpW = core::mem::transmute(GetProcAddress(s_hHHCTRL_OCX, b"HtmlHelpW\0".as_ptr() as *const i8));
        }
    }

    if s_pHtmlHelpW.is_null() {
        return core::ptr::null_mut();
    }

    return s_pHtmlHelpW(hwndCaller, pszFile, uCommand, dwData);
}
#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPWSTR, nCmdShow: INT) -> INT {
    __cxx_global_init();
    g_hinstExe = hInstance;

    // Initialize common controls library
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();
    iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccx.dwICC = ICC_STANDARD_CLASSES | ICC_USEREX_CLASSES | ICC_BAR_CLASSES;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

    // Load settings from registry
    registrySettings.Load(nCmdShow);

    // Create the main window
    if !mainWindow.DoCreate() {
        MessageBox(core::ptr::null_mut(), CPP_COM_WSTR_85D11F0B94E7.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);
        return 1;
    }

    // Initialize imageModel
    if __argc < 2 || !DoLoadImageFile((mainWindow).op_to_HWND(), __targv[1], (TRUE) as BOOL) {
        InitializeImage(core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL);
    }

    // Make the window visible on the screen
    mainWindow.__base0.ShowWindow((registrySettings.WindowPlacement.showCmd) as i32);

    // Load the access keys
    let hAccel: HACCEL = ::LoadAcceleratorsW(hInstance, MAKEINTRESOURCEW!(800));

    // The message loop
    let mut msg: MSG = core::mem::zeroed();
    while ::GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
        if fontsDialog.__base0.IsWindow() != 0 && fontsDialog.__base0.IsDialogMessageW(core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }

        if ::TranslateAcceleratorW((mainWindow).op_to_HWND(), hAccel, core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }

        ::TranslateMessage(core::ptr::addr_of_mut!(msg));
        ::DispatchMessage(core::ptr::addr_of_mut!(msg));
    }

    // Unload the access keys
    ::DestroyAcceleratorTable(hAccel);

    // Write back settings to registry
    registrySettings.Store();

    if g_szMailTempFile[0] != 0 {
        ::DeleteFileW(core::ptr::addr_of_mut!(g_szMailTempFile) as *mut u16);
    }

    // Return the value that PostQuitMessage() gave
    return msg.wParam as INT;
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    (*core::ptr::addr_of_mut!(canvasWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(mirrorRotateDialog)).__ctor_0();
    (*core::ptr::addr_of_mut!(attributesDialog)).__ctor_0();
    (*core::ptr::addr_of_mut!(stretchSkewDialog)).__ctor_0();
    (*core::ptr::addr_of_mut!(fontsDialog)).__ctor_0();
    (*core::ptr::addr_of_mut!(fullscreenWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(imageModel)).__ctor_0();
    (*core::ptr::addr_of_mut!(mainWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(miniature)).__ctor_0();
    (*core::ptr::addr_of_mut!(paletteWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(paletteModel)).__ctor_0();
    (*core::ptr::addr_of_mut!(selectionModel)).__ctor_0();
    (*core::ptr::addr_of_mut!(textEditWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(toolBoxContainer)).__ctor_0();
    (*core::ptr::addr_of_mut!(toolSettingsWindow)).__ctor_0();
    (*core::ptr::addr_of_mut!(toolsModel)).__ctor_0();
}

impl CMainWindow {
    pub unsafe fn DoCreate(&mut self) -> HWND {
        let this_: *mut Self = self;
        LoadStringW(g_hinstExe, IDS_DEFAULTFILENAME, core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR, _countof!(g_szFileName) as i32);
    
        let mut strTitle = CStringW::new();
        atl_format!(strTitle, CStringW::from_resource(IDS_WINDOWTITLE).GetString(), PathFindFileName(core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR));
    
        let rc: &mut RECT = &mut registrySettings.WindowPlacement.rcNormalPosition;
        return (*this_).Create(HWND_DESKTOP, rc, strTitle.GetString(), WS_OVERLAPPEDWINDOW, WS_EX_ACCEPTFILES);
    }
    pub unsafe fn TrackPopupMenu(&mut self, ptScreen: POINT, iSubMenu: INT) {
        let this_: *mut Self = self;
        let hMenu: HMENU = LoadMenuW(g_hinstExe, MAKEINTRESOURCEW!(ID_POPUPMENU));
        let hSubMenu: HMENU = GetSubMenu(hMenu, iSubMenu);
    
        SetForegroundWindow((*this_).__base0.m_hWnd);
        let id: INT_PTR = (TrackPopupMenu(hSubMenu, TPM_RIGHTBUTTON | TPM_RETURNCMD,
                                      ptScreen.x, ptScreen.y, 0, (*this_).__base0.m_hWnd, core::ptr::null_mut())) as _;
        (*this_).__base0.PostMessageW(WM_NULL, 0, 0);
        if id != 0 {
            (*this_).__base0.PostMessageW(WM_COMMAND, id, 0);
        }
    
        DestroyMenu(hMenu);
    }
    pub unsafe fn CanUndo(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible(textEditWindow.op_to_HWND()) != 0 {
            return textEditWindow.__base0.SendMessageW__0(EM_CANUNDO, 0, 0) as BOOL;
        }
        if ((selectionModel.m_bShow) != 0) && ((toolsModel.IsSelection()) != 0) {
            return (TRUE) as BOOL;
        }
        return imageModel.CanUndo();
    }
    pub unsafe fn CanRedo(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible(textEditWindow.op_to_HWND()) != 0 {
            return (FALSE) as BOOL; // There is no "WM_REDO" in EDIT control
        }
        return imageModel.CanRedo();
    }
    pub unsafe fn CanPaste(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible((textEditWindow).op_to_HWND()) != 0 {
            return IsClipboardFormatAvailable(CF_UNICODETEXT);
        }
    
        return (IsClipboardFormatAvailable(CF_ENHMETAFILE) != 0 ||
                IsClipboardFormatAvailable(CF_DIB) != 0 ||
                IsClipboardFormatAvailable(CF_BITMAP) != 0) as BOOL;
    }
    pub unsafe fn OnDropFiles(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut droppedfile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    
        let hDrop: HDROP = wParam as HDROP;
        DragQueryFile(hDrop, 0, droppedfile.as_mut_ptr(), _countof!(droppedfile));
        DragFinish(hDrop);
    
        if (*this_).ConfirmSave() != 0 {
            DoLoadImageFile((*this_).__base0.m_hWnd, droppedfile.as_mut_ptr(), (TRUE) as BOOL);
        }
    
        return 0;
    }
    pub unsafe fn OnCreate(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Loading and setting the window menu from resource
        (*this_).m_hMenu = LoadMenuW(g_hinstExe, MAKEINTRESOURCEW!(ID_MENU));
        (*this_).__base0.SetMenu((*this_).m_hMenu);
    
        // Create the status bar
        let mut style: DWORD = SBARS_SIZEGRIP | WS_CHILD | (if registrySettings.ShowStatusBar != 0 { WS_VISIBLE } else { 0 });
        g_hStatusBar = CreateWindowExW(0, STATUSCLASSNAME, core::ptr::null_mut(), style, 0, 0, 0, 0, (*this_).__base0.m_hWnd,
                                         core::ptr::null_mut(), g_hinstExe, core::ptr::null_mut());
        SendMessageW(g_hStatusBar, SB_SETMINHEIGHT, 21, 0);
    
        // Create the tool box
        toolBoxContainer.DoCreate((*this_).__base0.m_hWnd);
    
        // Create the palette window
        let rcEmpty: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 }; // Rely on WM_SIZE
        style = WS_CHILD | (if registrySettings.ShowPalette != 0 { WS_VISIBLE } else { 0 });
        paletteWindow.Create((*this_).__base0.m_hWnd, &mut rcEmpty, core::ptr::null_mut(), style, WS_EX_STATICEDGE);
    
        // Create the canvas
        style = WS_CHILD | WS_GROUP | WS_HSCROLL | WS_VSCROLL | WS_VISIBLE;
        canvasWindow.Create((*this_).__base0.m_hWnd, &mut rcEmpty, core::ptr::null_mut(), style, WS_EX_CLIENTEDGE);
    
        // Create and show the miniature if necessary
        if registrySettings.ShowThumbnail != 0 {
            miniature.DoCreate((*this_).__base0.m_hWnd);
            miniature.__base0.ShowWindow((SW_SHOWNOACTIVATE) as i32);
        }
    
        // Set icon
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_BIG) as WPARAM, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_SMALL) as WPARAM, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
    
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        registrySettings.WindowPlacement.length = core::mem::size_of::<WINDOWPLACEMENT>() as _;
        (*this_).__base0.GetWindowPlacement(core::ptr::addr_of_mut!(registrySettings.WindowPlacement));
    
        DoHtmlHelpW(core::ptr::null_mut(), core::ptr::null_mut(), HH_CLOSE_ALL, 0);
    
        if !s_hHHCTRL_OCX.is_null() {
            FreeLibrary(s_hHHCTRL_OCX);
            s_hHHCTRL_OCX = core::ptr::null_mut();
            s_pHtmlHelpW = core::ptr::null_mut();
        }
    
        (*this_).__base0.SetMenu(core::ptr::null_mut());
        if !(*this_).m_hMenu.is_null() {
            DestroyMenu((*this_).m_hMenu);
            (*this_).m_hMenu = core::ptr::null_mut();
        }
    
        PostQuitMessage(0); /* send a WM_QUIT to the message queue */
        return 0;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).ConfirmSave() != 0 {
            (*this_).__base0.DestroyWindow();
        }
        return 0;
    }
    pub unsafe fn OnInitMenuPopup(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let menu: HMENU = wParam as HMENU;
        let trueSelection: BOOL = ((selectionModel.m_bShow != 0) && (toolsModel.IsSelection() != 0)) as BOOL;
        let textShown: BOOL = (((toolsModel.GetActiveTool() == TOOL_TEXT) as BOOL != 0) && (IsWindowVisible((textEditWindow).op_to_HWND()) != 0)) as BOOL;
        let mut dwStart: DWORD = 0;
        let mut dwEnd: DWORD = 0;
        if textShown != 0 {
            (textEditWindow).__base0.SendMessageW__0(EM_GETSEL, core::ptr::addr_of_mut!(dwStart) as WPARAM, core::ptr::addr_of_mut!(dwEnd) as LPARAM);
        }
        let hasTextSel: BOOL = (dwStart < dwEnd) as BOOL;
    
        //
        // File menu
        //
        if GetSubMenu((*this_).__base0.GetMenu(), 0) == menu {
            (*this_).ProcessFileMenu(menu);
        }
    
        //
        // Edit menu
        //
        EnableMenuItem(menu, IDM_EDITUNDO, ENABLED_IF!((((*this_).CanUndo()) != 0)));
        EnableMenuItem(menu, IDM_EDITREDO, ENABLED_IF!((((*this_).CanRedo()) != 0)));
        EnableMenuItem(menu, IDM_EDITCUT, ENABLED_IF!(if textShown != 0 { ((hasTextSel) != 0) } else { ((trueSelection) != 0) }));
        EnableMenuItem(menu, IDM_EDITCOPY, ENABLED_IF!(if textShown != 0 { ((hasTextSel) != 0) } else { ((trueSelection) != 0) }));
        EnableMenuItem(menu, IDM_EDITDELETESELECTION,
                       ENABLED_IF!(if textShown != 0 { ((hasTextSel) != 0) } else { ((trueSelection) != 0) }));
        EnableMenuItem(menu, IDM_EDITINVERTSELECTION, ENABLED_IF!(((trueSelection) != 0)));
        EnableMenuItem(menu, IDM_EDITCOPYTO, ENABLED_IF!(((trueSelection) != 0)));
        EnableMenuItem(menu, IDM_EDITPASTE, ENABLED_IF!((((*this_).CanPaste()) != 0)));
        EnableMenuItem(menu, IDM_CROPSELECTION, ENABLED_IF!(((trueSelection) != 0)));
    
        //
        // View menu
        //
        CheckMenuItem(menu, IDM_VIEWTOOLBOX, CHECKED_IF!(((IsWindowVisible((toolBoxContainer).op_to_HWND())) != 0)));
        CheckMenuItem(menu, IDM_VIEWCOLORPALETTE, CHECKED_IF!(((IsWindowVisible((paletteWindow).op_to_HWND())) != 0)));
        CheckMenuItem(menu, IDM_VIEWSTATUSBAR, CHECKED_IF!(((IsWindowVisible(g_hStatusBar)) != 0)));
        CheckMenuItem(menu, IDM_FORMATICONBAR, CHECKED_IF!(((IsWindowVisible((fontsDialog).op_to_HWND())) != 0)));
        EnableMenuItem(menu, IDM_FORMATICONBAR, ENABLED_IF!((((toolsModel.GetActiveTool() == TOOL_TEXT) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM125, CHECKED_IF!((((toolsModel.GetZoom() == 125) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM25, CHECKED_IF!((((toolsModel.GetZoom() == 250) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM50, CHECKED_IF!((((toolsModel.GetZoom() == 500) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM100, CHECKED_IF!((((toolsModel.GetZoom() == 1000) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM200, CHECKED_IF!((((toolsModel.GetZoom() == 2000) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM400, CHECKED_IF!((((toolsModel.GetZoom() == 4000) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWZOOM800, CHECKED_IF!((((toolsModel.GetZoom() == 8000) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_VIEWSHOWGRID, CHECKED_IF!(((g_showGrid) != 0)));
        CheckMenuItem(menu, IDM_VIEWSHOWMINIATURE, CHECKED_IF!(((registrySettings.ShowThumbnail) != 0)));
    
        //
        // Image menu
        //
        EnableMenuItem(menu, IDM_IMAGECROP, ENABLED_IF!(((selectionModel.m_bShow) != 0)));
        EnableMenuItem(menu, IDM_IMAGEDELETEIMAGE, ENABLED_IF!((((selectionModel.m_bShow == 0) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_IMAGEDRAWOPAQUE, CHECKED_IF!((((toolsModel.IsBackgroundTransparent() == 0) as BOOL) != 0)));
    
        //
        // Palette menu
        //
        CheckMenuItem(menu, IDM_COLORSMODERNPALETTE, CHECKED_IF!((((paletteModel.SelectedPalette() == PAL_MODERN) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_COLORSOLDPALETTE, CHECKED_IF!((((paletteModel.SelectedPalette() == PAL_OLDTYPE) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_COLORSGRAYSCALE, CHECKED_IF!((((paletteModel.SelectedPalette() == PAL_GRAYSCALE) as BOOL) != 0)));
        CheckMenuItem(menu, IDM_COLORSMONOCHROME, CHECKED_IF!((((paletteModel.SelectedPalette() == PAL_MONOCHROME) as BOOL) != 0)));
        return 0;
    }
    pub unsafe fn OnSize(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut test: [i32; 3] = [(LOWORD!(lParam) as i32) - 260, (LOWORD!(lParam) as i32) - 140, (LOWORD!(lParam) as i32) - 20];
        if IsWindow(g_hStatusBar) != 0 {
            SendMessageW(g_hStatusBar, WM_SIZE, 0, 0);
            SendMessageW(g_hStatusBar, SB_SETPARTS, 3, core::ptr::addr_of_mut!(test) as LPARAM);
        }
        (*this_).alignChildrenToMainWindow();
        return 0;
    }
    pub unsafe fn OnGetMinMaxInfo(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mm: *mut MINMAXINFO = lParam as *mut MINMAXINFO;
        (*mm).ptMinTrackSize = POINT { x: 330, y: 360 };
        return 0;
    }
    pub unsafe fn OnKeyDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match wParam {
            VK_ESCAPE => {
                canvasWindow.__base0.PostMessageW(nMsg, wParam, lParam);
            }
            VK_LEFT => {
                selectionModel.moveSelection(-1, 0);
            }
            VK_RIGHT => {
                selectionModel.moveSelection(1, 0);
            }
            VK_UP => {
                selectionModel.moveSelection(0, -1);
            }
            VK_DOWN => {
                selectionModel.moveSelection(0, 1);
            }
            _ => {}
        }
        return 0;
    }
    pub unsafe fn OnSysColorChange(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        /* Redirect message to common controls */
        let hToolbar: HWND = FindWindowEx(toolBoxContainer.__base0.m_hWnd, core::ptr::null_mut(), TOOLBARCLASSNAME, core::ptr::null_mut());
        SendMessageW(hToolbar, WM_SYSCOLORCHANGE, 0, 0);
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Disable commands while dragging mouse
        if canvasWindow.m_drawing != 0 && GetCapture() != core::ptr::null_mut()
        {
            ATLTRACE!(b"locking!\n\0".as_ptr() as *const i8);
            return 0;
        }
    
        let textShown: BOOL = (toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible((textEditWindow).op_to_HWND()) != 0) as BOOL;
        'sw: {
            match LOWORD!(wParam) as UINT {
                IDM_HELPINFO => {
                    let mut infotitle = [0u16; 100 as usize];
                    let mut infotext = [0u16; 200 as usize];
                    LoadStringW(g_hinstExe, IDS_INFOTITLE, infotitle.as_mut_ptr(), (_countof!(infotitle)) as i32);
                    LoadStringW(g_hinstExe, IDS_INFOTEXT, infotext.as_mut_ptr(), (_countof!(infotext)) as i32);
                    ShellAboutW((*this_).__base0.m_hWnd, infotitle.as_ptr(), infotext.as_ptr(),
                                  LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)));
                    break 'sw;
                }
                IDM_HELPHELPTOPICS => {
                    DoHtmlHelpW((*this_).__base0.m_hWnd, CPP_COM_WSTR_461B74EEBD9E.as_ptr(), HH_DISPLAY_TOPIC, 0);
                    break 'sw;
                }
                IDM_FILEEXIT => {
                    (*this_).__base0.SendMessageW__0(WM_CLOSE, wParam, lParam);
                    break 'sw;
                }
                IDM_FILENEW => {
                    if (*this_).ConfirmSave() != 0
                    {
                        InitializeImage(core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL);
                    }
                    break 'sw;
                }
                IDM_FILEOPEN => {
                    {
                        let mut szFileName = [0u16; MAX_LONG_PATH as usize];
                        core::ptr::copy_nonoverlapping(CPP_COM_WSTR_96A296D224F2.as_ptr(), szFileName.as_mut_ptr(), szFileName.len());
                        if (*this_).ConfirmSave() != 0 && (*this_).GetOpenFileName(szFileName.as_mut_ptr(), _countof!(szFileName)) != 0
                        {
                            DoLoadImageFile((*this_).__base0.m_hWnd, szFileName.as_ptr(), (TRUE) as BOOL);
                        }
                        break 'sw;
                    }
                }
                IDM_FILESAVE => {
                    (*this_).saveImage((TRUE) as BOOL);
                    break 'sw;
                }
                IDM_FILESAVEAS => {
                    (*this_).saveImage((FALSE) as BOOL);
                    break 'sw;
                }
                IDM_FILEPAGESETUP => {
                    // DUMMY: Shows the dialog only, no functionality
                    let mut psd: PAGESETUPDLG = core::mem::zeroed();
                    ZeroMemory!(core::ptr::addr_of_mut!(psd) as *mut core::ffi::c_void, core::mem::size_of::<PAGESETUPDLG>());
                    psd.lStructSize = core::mem::size_of::<PAGESETUPDLG>() as _;
                    psd.hwndOwner = (*this_).__base0.m_hWnd;
                    PageSetupDlg(core::ptr::addr_of_mut!(psd));
                    break 'sw;
                }
                IDM_FILEPRINT => {
                    // TODO: Test whether it actually works
                    let mut pd: PRINTDLG = core::mem::zeroed();
                    ZeroMemory!(core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void, core::mem::size_of::<PRINTDLG>());
                    pd.lStructSize = core::mem::size_of::<PRINTDLG>() as _;
                    pd.hwndOwner = (*this_).__base0.m_hWnd;
                    pd.hDevMode = core::ptr::null_mut();  // freed by user
                    pd.hDevNames = core::ptr::null_mut();  // freed by user
                    pd.Flags = PD_USEDEVMODECOPIESANDCOLLATE | PD_RETURNDC;
                    pd.nCopies = 1;
                    pd.nFromPage = 0xffff;
                    pd.nToPage = 0xffff;
                    pd.nMinPage = 1;
                    pd.nMaxPage = 0xffff;
                    if (PrintDlg(core::ptr::addr_of_mut!(pd))) as u32 == TRUE
                    {
                        BitBlt(pd.hDC, 0, 0, imageModel.GetWidth(), imageModel.GetHeight(), imageModel.GetDC(), 0, 0, SRCCOPY);
                        DeleteDC(pd.hDC);
                    }
                    if !pd.hDevMode.is_null()
                    {
                        GlobalFree(pd.hDevMode);
                    }
                    if !pd.hDevNames.is_null()
                    {
                        GlobalFree(pd.hDevNames);
                    }
                    break 'sw;
                }
                IDM_FILESEND => {
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    if OpenMailer((*this_).__base0.m_hWnd, core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR) == 0
                    {
                        ShowError((IDS_CANTSENDMAIL) as INT);
                    }
                    break 'sw;
                }
                IDM_FILEASWALLPAPERPLANE => {
                    RegistrySettings::SetWallpaper(core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR, RegistrySettings_WallpaperStyle_TILED);
                    break 'sw;
                }
                IDM_FILEASWALLPAPERCENTERED => {
                    RegistrySettings::SetWallpaper(core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR, RegistrySettings_WallpaperStyle_CENTERED);
                    break 'sw;
                }
                IDM_FILEASWALLPAPERSTRETCHED => {
                    RegistrySettings::SetWallpaper(core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR, RegistrySettings_WallpaperStyle_STRETCHED);
                    break 'sw;
                }
                IDM_FILE1 | IDM_FILE2 | IDM_FILE3 | IDM_FILE4 => {
                    {
                        let iFile: INT = (LOWORD!(wParam) as INT) - (IDM_FILE1 as i32);
                        if (*this_).ConfirmSave() != 0
                        {
                            DoLoadImageFile((*this_).__base0.m_hWnd, registrySettings.strFiles[iFile as usize].GetString(), (TRUE) as BOOL);
                        }
                        break 'sw;
                    }
                }
                IDM_EDITUNDO => {
                    if textShown != 0
                    {
                        textEditWindow.__base0.PostMessageW(WM_UNDO, 0, 0);
                        break 'sw;
                    }
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    imageModel.Undo((FALSE) as BOOL);
                    break 'sw;
                }
                IDM_EDITREDO => {
                    if textShown != 0
                    {
                        // There is no "WM_REDO" in EDIT control
                        break 'sw;
                    }
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    imageModel.Redo();
                    break 'sw;
                }
                IDM_EDITCOPY => {
                    if textShown != 0
                    {
                        textEditWindow.__base0.SendMessageW__0(WM_COPY, 0, 0);
                        break 'sw;
                    }
                    if selectionModel.m_bShow == 0 || (*this_).__base0.OpenClipboard() == 0
                    {
                        break 'sw;
                    }
    
                    EmptyClipboard();
    
                    selectionModel.TakeOff();
    
                    {
                        let hbmCopy = selectionModel.GetSelectionContents();
                        let hGlobal = BitmapToClipboardDIB(hbmCopy);
                        if !hGlobal.is_null()
                        {
                            SetClipboardData(CF_DIB, hGlobal);
                        }
                        else
                        {
                            ShowOutOfMemory();
                            imageModel.ClearHistory();
                        }
                        DeleteObject((hbmCopy).cast());
                    }
    
                    CloseClipboard();
                    break 'sw;
                }
                IDM_EDITCUT => {
                    if textShown != 0
                    {
                        textEditWindow.__base0.SendMessageW__0(WM_CUT, 0, 0);
                        break 'sw;
                    }
                    /* Copy */
                    (*this_).__base0.SendMessageW__0(WM_COMMAND, (IDM_EDITCOPY) as WPARAM, 0);
                    /* Delete selection */
                    (*this_).__base0.SendMessageW__0(WM_COMMAND, (IDM_EDITDELETESELECTION) as WPARAM, 0);
                    break 'sw;
                }
                IDM_EDITPASTE => {
                    if textShown != 0
                    {
                        textEditWindow.__base0.SendMessageW__0(WM_PASTE, 0, 0);
                        break 'sw;
                    }
    
                    if (*this_).__base0.OpenClipboard() == 0
                    {
                        break 'sw;
                    }
    
                    // In many cases, CF_ENHMETAFILE provides a better image than CF_DIB
                    if IsClipboardFormatAvailable(CF_ENHMETAFILE) != 0
                    {
                        let hEMF: HENHMETAFILE = GetClipboardData(CF_ENHMETAFILE) as HENHMETAFILE;
                        if !hEMF.is_null()
                        {
                            let hbm = BitmapFromHEMF(hEMF);
                            DeleteEnhMetaFile(hEMF);
                            if !hbm.is_null()
                            {
                                (*this_).InsertSelectionFromHBITMAP(hbm, (*this_).__base0.m_hWnd);
                                CloseClipboard();
                                break 'sw;
                            }
                        }
                    }
    
                    // In many cases, CF_DIB provides a better image than CF_BITMAP
                    if IsClipboardFormatAvailable(CF_DIB) != 0
                    {
                        let hbm = BitmapFromClipboardDIB(GetClipboardData(CF_DIB));
                        if !hbm.is_null()
                        {
                            (*this_).InsertSelectionFromHBITMAP(hbm, (*this_).__base0.m_hWnd);
                            CloseClipboard();
                            break 'sw;
                        }
                    }
    
                    // The last resort
                    if IsClipboardFormatAvailable(CF_BITMAP) != 0
                    {
                        let hbm = GetClipboardData(CF_BITMAP) as HBITMAP;
                        if !hbm.is_null()
                        {
                            (*this_).InsertSelectionFromHBITMAP(hbm, (*this_).__base0.m_hWnd);
                            CloseClipboard();
                            break 'sw;
                        }
                    }
    
                    // Failed to paste
                    {
                        let mut strText = CStringW::new();
                        let mut strTitle = CStringW::new();
                        strText.LoadStringW(IDS_CANTPASTE);
                        strTitle.LoadStringW(IDS_PROGRAMNAME);
                        (*this_).__base0.MessageBoxW(strText.GetString(), strTitle.GetString(), MB_ICONINFORMATION);
                    }
    
                    CloseClipboard();
                    break 'sw;
                }
                IDM_CROPSELECTION => {
                    {
                        let hbmSelection = selectionModel.GetSelectionContents();
                        if !hbmSelection.is_null()
                        {
                            imageModel.PushImageForUndo__1(hbmSelection);
                            selectionModel.HideSelection();
                            imageModel.NotifyImageChanged();
                        }
                        break 'sw;
                    }
                }
                IDM_EDITDELETESELECTION => {
                    {
                        if textShown != 0
                        {
                            textEditWindow.__base0.SendMessageW__0(WM_CLEAR, 0, 0);
                            break 'sw;
                        }
                        match toolsModel.GetActiveTool() {
                            TOOL_FREESEL | TOOL_RECTSEL => {
                                selectionModel.DeleteSelection();
                            }
    
                            TOOL_TEXT => {
                                canvasWindow.OnEndDraw((TRUE) as BOOL);
                            }
                            _ => {}
                        }
                        break 'sw;
                    }
                }
                IDM_EDITSELECTALL => {
                    {
                        if textShown != 0
                        {
                            textEditWindow.__base0.SendMessageW__0(EM_SETSEL, 0, -1);
                            break 'sw;
                        }
                        let hToolbar = FindWindowEx(toolBoxContainer.__base0.m_hWnd, core::ptr::null_mut(), TOOLBARCLASSNAME, core::ptr::null_mut());
                        SendMessageW(hToolbar, TB_CHECKBUTTON, (ID_RECTSEL) as WPARAM, MAKELPARAM!(TRUE, 0));
                        toolsModel.selectAll();
                        canvasWindow.__base0.Invalidate((TRUE) as BOOL);
                        break 'sw;
                    }
                }
                IDM_EDITCOPYTO => {
                    {
                        let mut szFileName = [0u16; MAX_LONG_PATH as usize];
                        LoadStringW(g_hinstExe, IDS_DEFAULTFILENAME, szFileName.as_mut_ptr(), (_countof!(szFileName)) as i32);
                        if (*this_).GetSaveFileName(szFileName.as_mut_ptr(), _countof!(szFileName)) != 0
                        {
                            let hbmSelection = selectionModel.GetSelectionContents();
                            if hbmSelection.is_null()
                            {
                                ShowOutOfMemory();
                                imageModel.ClearHistory();
                                break 'sw;
                            }
                            SaveDIBToFile(hbmSelection, szFileName.as_ptr(), (FALSE) as BOOL, &GUID_NULL);
                            DeleteObject((hbmSelection).cast());
                        }
                        break 'sw;
                    }
                }
                IDM_EDITPASTEFROM => {
                    {
                        let mut szFileName = [0u16; MAX_LONG_PATH as usize];
                        core::ptr::copy_nonoverlapping(CPP_COM_WSTR_96A296D224F2.as_ptr(), szFileName.as_mut_ptr(), szFileName.len());
                        if (*this_).GetOpenFileName(szFileName.as_mut_ptr(), _countof!(szFileName)) != 0
                        {
                            let hbmNew = DoLoadImageFile((*this_).__base0.m_hWnd, szFileName.as_ptr(), (FALSE) as BOOL);
                            if !hbmNew.is_null()
                            {
                                (*this_).InsertSelectionFromHBITMAP(hbmNew, (*this_).__base0.m_hWnd);
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_COLORSEDITPALETTE => {
                    {
                        let mut rgbColor: COLORREF = paletteModel.GetFgColor();
                        if (*this_).ChooseColor(core::ptr::addr_of_mut!(rgbColor)) != 0
                        {
                            paletteModel.SetFgColor(rgbColor);
                        }
                        break 'sw;
                    }
                }
                IDM_COLORSMODERNPALETTE => {
                    paletteModel.SelectPalette(PAL_MODERN);
                    break 'sw;
                }
                IDM_COLORSOLDPALETTE => {
                    paletteModel.SelectPalette(PAL_OLDTYPE);
                    break 'sw;
                }
                IDM_COLORSGRAYSCALE => {
                    paletteModel.SelectPalette(PAL_GRAYSCALE);
                    break 'sw;
                }
                IDM_COLORSMONOCHROME => {
                    paletteModel.SelectPalette(PAL_MONOCHROME);
                    break 'sw;
                }
                IDM_IMAGEINVERTCOLORS => {
                    {
                        if selectionModel.m_bShow != 0
                        {
                            selectionModel.InvertSelection();
                        }
                        else
                        {
                            imageModel.InvertColors();
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGEDELETEIMAGE => {
                    imageModel.PushImageForUndo__0();
                    Rect(imageModel.GetDC(), 0, 0, imageModel.GetWidth(), imageModel.GetHeight(), paletteModel.GetBgColor(), paletteModel.GetBgColor(), 0, (TRUE) as i32);
                    imageModel.NotifyImageChanged();
                    break 'sw;
                }
                IDM_IMAGEROTATEMIRROR => {
                    {
                        let mut waitCursor = CWaitCursor__make_0();
                        canvasWindow.updateScrollPos(0, 0);
                        match mirrorRotateDialog.DoModal(mainWindow.__base0.m_hWnd) {
                            1 => { /* flip horizontally */
                                if selectionModel.m_bShow != 0
                                {
                                    selectionModel.FlipHorizontally();
                                }
                                else
                                {
                                    imageModel.FlipHorizontally();
                                }
                            }
                            2 => { /* flip vertically */
                                if selectionModel.m_bShow != 0
                                {
                                    selectionModel.FlipVertically();
                                }
                                else
                                {
                                    imageModel.FlipVertically();
                                }
                            }
                            3 => { /* rotate 90 degrees */
                                if selectionModel.m_bShow != 0
                                {
                                    selectionModel.RotateNTimes90Degrees(1);
                                }
                                else
                                {
                                    imageModel.RotateNTimes90Degrees(1);
                                }
                            }
                            4 => { /* rotate 180 degrees */
                                if selectionModel.m_bShow != 0
                                {
                                    selectionModel.RotateNTimes90Degrees(2);
                                }
                                else
                                {
                                    imageModel.RotateNTimes90Degrees(2);
                                }
                            }
                            5 => { /* rotate 270 degrees */
                                if selectionModel.m_bShow != 0
                                {
                                    selectionModel.RotateNTimes90Degrees(3);
                                }
                                else
                                {
                                    imageModel.RotateNTimes90Degrees(3);
                                }
                            }
                            _ => {}
                        }
                    }
                    break 'sw;
                }
                IDM_IMAGEATTRIBUTES => {
                    {
                        if attributesDialog.DoModal(mainWindow.__base0.m_hWnd) != 0
                        {
                            let mut waitCursor = CWaitCursor__make_0();
                            if attributesDialog.m_bBlackAndWhite != 0 && imageModel.IsBlackAndWhite() == 0
                            {
                                let mut strText = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_LOSECOLOR));
                                let mut strTitle = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_PROGRAMNAME));
                                let id: INT = (*this_).__base0.MessageBoxW(strText.GetString(), strTitle.GetString(), MB_ICONINFORMATION | MB_YESNOCANCEL);
                                if (id) as u32 != IDYES
                                {
                                    break 'sw;
                                }
    
                                imageModel.PushBlackAndWhite();
                            }
    
                            if imageModel.GetWidth() != attributesDialog.newWidth ||
                                imageModel.GetHeight() != attributesDialog.newHeight
                            {
                                imageModel.Crop(attributesDialog.newWidth, attributesDialog.newHeight, 0, 0);
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGESTRETCHSKEW => {
                    {
                        if stretchSkewDialog.DoModal(mainWindow.__base0.m_hWnd) != 0
                        {
                            let mut waitCursor = CWaitCursor__make_0();
                            if selectionModel.m_bShow != 0
                            {
                                selectionModel.StretchSkew(stretchSkewDialog.percentage.x, stretchSkewDialog.percentage.y,
                                                           stretchSkewDialog.angle.x, stretchSkewDialog.angle.y);
                            }
                            else
                            {
                                imageModel.StretchSkew(stretchSkewDialog.percentage.x, stretchSkewDialog.percentage.y,
                                                       stretchSkewDialog.angle.x, stretchSkewDialog.angle.y);
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGEDRAWOPAQUE => {
                    toolsModel.SetBackgroundTransparent((toolsModel.IsBackgroundTransparent() == 0) as BOOL);
                    break 'sw;
                }
                IDM_IMAGECROP => {
                    {
                        let hbmCopy = selectionModel.GetSelectionContents();
                        imageModel.PushImageForUndo__1(hbmCopy);
                        selectionModel.HideSelection();
                        break 'sw;
                    }
                }
                IDM_VIEWTOOLBOX => {
                    registrySettings.ShowToolBox = (toolBoxContainer.__base0.IsWindowVisible() == 0) as DWORD;
                    toolBoxContainer.__base0.ShowWindow(if registrySettings.ShowToolBox != 0 { (SW_SHOWNOACTIVATE) as i32 } else { (SW_HIDE) as i32 });
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_VIEWCOLORPALETTE => {
                    registrySettings.ShowPalette = (paletteWindow.__base0.IsWindowVisible() == 0) as DWORD;
                    paletteWindow.__base0.ShowWindow(if registrySettings.ShowPalette != 0 { (SW_SHOWNOACTIVATE) as i32 } else { (SW_HIDE) as i32 });
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_VIEWSTATUSBAR => {
                    registrySettings.ShowStatusBar = (IsWindowVisible(g_hStatusBar) == 0) as DWORD;
                    ShowWindow(g_hStatusBar, (if registrySettings.ShowStatusBar != 0 { (SW_SHOWNOACTIVATE) as i32 } else { (SW_HIDE) as i32 }));
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_FORMATICONBAR => {
                    if toolsModel.GetActiveTool() == TOOL_TEXT
                    {
                        if fontsDialog.__base0.IsWindow() == 0
                        {
                            fontsDialog.Create((mainWindow).op_to_HWND());
                        }
                        registrySettings.ShowTextTool = (IsWindowVisible((fontsDialog).op_to_HWND()) == 0) as DWORD;
                        fontsDialog.__base0.ShowWindow(if registrySettings.ShowTextTool != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
                        fontsDialog.__base0.SendMessageW__0(DM_REPOSITION, 0, 0);
                    }
                    break 'sw;
                }
                IDM_VIEWSHOWGRID => {
                    g_showGrid = (g_showGrid == 0) as BOOL;
                    canvasWindow.__base0.Invalidate((FALSE) as BOOL);
                    break 'sw;
                }
                IDM_VIEWSHOWMINIATURE => {
                    registrySettings.ShowThumbnail = (IsWindowVisible((miniature).op_to_HWND()) == 0) as DWORD;
                    miniature.DoCreate((*this_).__base0.m_hWnd);
                    miniature.__base0.ShowWindow(if registrySettings.ShowThumbnail != 0 { (SW_SHOWNOACTIVATE) as i32 } else { (SW_HIDE) as i32 });
                    break 'sw;
                }
    
                IDM_VIEWZOOM125 => {
                    canvasWindow.zoomTo(125, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM25 => {
                    canvasWindow.zoomTo(250, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM50 => {
                    canvasWindow.zoomTo(500, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM100 => {
                    canvasWindow.zoomTo(1000, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM200 => {
                    canvasWindow.zoomTo(2000, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM400 => {
                    canvasWindow.zoomTo(4000, 0, 0);
                    break 'sw;
                }
                IDM_VIEWZOOM800 => {
                    canvasWindow.zoomTo(8000, 0, 0);
                    break 'sw;
                }
    
                IDM_VIEWFULLSCREEN => {
                    // Create and show the fullscreen window
                    fullscreenWindow.DoCreate();
                    fullscreenWindow.__base0.ShowWindow((SW_SHOWMAXIMIZED) as i32);
                    break 'sw;
                }
    
                IDM_CTRL_PLUS => {
                    toolsModel.SpecialTweak((FALSE) as BOOL);
                    break 'sw;
                }
                IDM_CTRL_MINUS => {
                    toolsModel.SpecialTweak((TRUE) as BOOL);
                    break 'sw;
                }
                _ => {}
            }
        }
        return 0;
    }
    pub unsafe fn OnMouseWheel(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut zDelta: INT = (HIWORD!(wParam) as SHORT) as INT;
    
        if GetKeyState(0x11 /* VK_CONTROL */) < 0 {
            // Ctrl+Wheel
            if zDelta < 0 {
                if toolsModel.GetZoom() > (MIN_ZOOM as i32) {
                    canvasWindow.zoomTo(toolsModel.GetZoom() / 2, 0, 0);
                }
            } else if zDelta > 0 {
                if toolsModel.GetZoom() < (MAX_ZOOM as i32) {
                    canvasWindow.zoomTo(toolsModel.GetZoom() * 2, 0, 0);
                }
            }
        } else {
            // Wheel only
            let mut nCount: UINT = 3;
            if GetAsyncKeyState(0x10 /* VK_SHIFT */) < 0 {
                // #define SPI_GETWHEELSCROLLCHARS 0x006C  // Needed for pre-NT6 PSDK
                SystemParametersInfoW(SPI_GETWHEELSCROLLCHARS, 0, core::ptr::addr_of_mut!(nCount) as *mut _, 0);
                let mut i: UINT = 0;
                while i < nCount {
                    if zDelta < 0 {
                        PostMessageW(canvasWindow.op_to_HWND(), WM_HSCROLL, MAKEWPARAM!(SB_LINERIGHT, 0), 0);
                    } else if zDelta > 0 {
                        PostMessageW(canvasWindow.op_to_HWND(), WM_HSCROLL, MAKEWPARAM!(SB_LINELEFT, 0), 0);
                    }
                    i += 1;
                }
            } else {
                SystemParametersInfoW(SPI_GETWHEELSCROLLLINES, 0, core::ptr::addr_of_mut!(nCount) as *mut _, 0);
                let mut i: UINT = 0;
                while i < nCount {
                    if zDelta < 0 {
                        PostMessageW(canvasWindow.op_to_HWND(), WM_VSCROLL, MAKEWPARAM!(SB_LINEDOWN, 0), 0);
                    } else if zDelta > 0 {
                        PostMessageW(canvasWindow.op_to_HWND(), WM_VSCROLL, MAKEWPARAM!(SB_LINEUP, 0), 0);
                    }
                    i += 1;
                }
            }
        }
    
        return 0;
    }
    pub unsafe fn alignChildrenToMainWindow(&mut self) {
        let this_: *mut Self = self;
        let mut clientRect: RECT = core::mem::zeroed();
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(clientRect));
        let mut rcSpace: RECT = clientRect;
        let uFlags: UINT = (SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREPOSITION | SWP_NOCOPYBITS);
    
        if IsWindowVisible(g_hStatusBar) != 0 {
            GetWindowRect(g_hStatusBar, core::ptr::addr_of_mut!(rc));
            rcSpace.bottom -= rc.bottom - rc.top;
        }
    
        let mut hDWP: HDWP = BeginDeferWindowPos(3);
    
        if IsWindowVisible(toolBoxContainer.op_to_HWND()) != 0 {
            if registrySettings.Bar2ID == BAR2ID_RIGHT {
                hDWP = DeferWindowPos(hDWP, toolBoxContainer.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.right - (CX_TOOLBAR as i32), rcSpace.top,
                                        (CX_TOOLBAR as i32), rcSpace.bottom - rcSpace.top,
                                        uFlags);
                rcSpace.right -= (CX_TOOLBAR as i32);
            } else {
                hDWP = DeferWindowPos(hDWP, toolBoxContainer.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.top,
                                        (CX_TOOLBAR as i32), rcSpace.bottom - rcSpace.top,
                                        uFlags);
                rcSpace.left += (CX_TOOLBAR as i32);
            }
        }
    
        if IsWindowVisible(paletteWindow.op_to_HWND()) != 0 {
            if registrySettings.Bar1ID == BAR1ID_BOTTOM {
                hDWP = DeferWindowPos(hDWP, paletteWindow.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.bottom - (CY_PALETTE as i32),
                                        rcSpace.right - rcSpace.left, (CY_PALETTE as i32),
                                        uFlags);
                rcSpace.bottom -= (CY_PALETTE as i32);
            } else {
                hDWP = DeferWindowPos(hDWP, paletteWindow.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.top,
                                        rcSpace.right - rcSpace.left, (CY_PALETTE as i32),
                                        uFlags);
                rcSpace.top += (CY_PALETTE as i32);
            }
        }
    
        if canvasWindow.__base0.IsWindow() != 0 {
            hDWP = DeferWindowPos(hDWP, canvasWindow.op_to_HWND(), core::ptr::null_mut(),
                                    rcSpace.left, rcSpace.top,
                                    rcSpace.right - rcSpace.left, rcSpace.bottom - rcSpace.top,
                                    uFlags);
        }
    
        EndDeferWindowPos(hDWP);
    }
    pub unsafe fn saveImage(&mut self, overwrite: BOOL) {
        let this_: *mut Self = self;
        let mut overwrite = overwrite;
        canvasWindow.OnEndDraw((FALSE) as BOOL);
    
        // Is the extension not supported?
        let pchDotExt: PWCHAR = PathFindExtensionW(core::ptr::addr_of_mut!(g_szFileName) as PWCHAR);
        if !pchDotExt.is_null() && *pchDotExt != 0 && CImageDx::IsExtensionSupported(pchDotExt) == 0 {
            // Remove the extension
            PathRemoveExtensionW(core::ptr::addr_of_mut!(g_szFileName) as PWCHAR);
            // No overwrite
            overwrite = (FALSE) as i32;
        }
    
        if g_isAFile != 0 && overwrite != 0 {
            imageModel.SaveImage(core::ptr::addr_of_mut!(g_szFileName) as PWCHAR);
        } else if (*this_).GetSaveFileName(core::ptr::addr_of_mut!(g_szFileName) as PWCHAR, _countof!(g_szFileName)) != 0 {
            imageModel.SaveImage(core::ptr::addr_of_mut!(g_szFileName) as PWCHAR);
        }
    }
    pub unsafe fn InsertSelectionFromHBITMAP(&mut self, bitmap: HBITMAP, window: HWND) {
        let this_: *mut Self = self;
        let width: i32 = GetDIBWidth(bitmap);
        let height: i32 = GetDIBHeight(bitmap);
        let mut curWidth: i32 = imageModel.GetWidth();
        let mut curHeight: i32 = imageModel.GetHeight();
    
        if width > curWidth || height > curHeight {
            let mut shouldEnlarge: BOOL = (TRUE) as i32;
    
            if ((g_askBeforeEnlarging) != 0) {
                let mut programname = [0u16; 20];
                let mut shouldEnlargePromptText = [0u16; 100];
    
                LoadStringW(g_hinstExe, IDS_PROGRAMNAME, programname.as_mut_ptr(), (_countof!(programname)) as i32);
                LoadStringW(g_hinstExe, IDS_ENLARGEPROMPTTEXT, shouldEnlargePromptText.as_mut_ptr(), (_countof!(shouldEnlargePromptText)) as i32);
    
                match ((*this_).__base0.MessageBoxW(shouldEnlargePromptText.as_mut_ptr(), programname.as_mut_ptr(), MB_YESNOCANCEL | MB_ICONQUESTION)) as u32 {
                    IDYES => {
                    }
                    IDNO => {
                        shouldEnlarge = (FALSE) as i32;
                    }
                    IDCANCEL => {
                        return;
                    }
                    _ => {}
                }
            }
    
            if shouldEnlarge != 0 {
                if width > curWidth {
                    curWidth = width;
                }
    
                if height > curHeight {
                    curHeight = height;
                }
    
                imageModel.Crop(curWidth, curHeight, 0, 0);
            }
        }
    
        toolsModel.SetActiveTool(TOOL_RECTSEL);
    
        selectionModel.InsertFromHBITMAP(bitmap, 0, 0, core::ptr::null_mut());
        selectionModel.m_bShow = (TRUE) as i32;
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn ConfirmSave(&mut self) -> BOOL {
        let this_: *mut Self = self;
        canvasWindow.OnEndDraw((FALSE) as BOOL);
    
        if ((imageModel.IsImageSaved()) != 0) {
            return (TRUE) as BOOL;
        }
    
        let mut strProgramName = CStringW::new();
        strProgramName.LoadStringW(IDS_PROGRAMNAME);
    
        let mut strSavePromptText = CStringW::new();
        atl_format!(
            strSavePromptText,
            IDS_SAVEPROMPTTEXT,
            PathFindFileName((g_szFileName).as_ptr())
        );
    
        match ((*this_).__base0.MessageBoxW(
            strSavePromptText.GetString(),
            strProgramName.GetString(),
            (MB_YESNOCANCEL | MB_ICONQUESTION),
        )) as u32 {
            IDYES => {
                (*this_).saveImage((TRUE) as BOOL);
                return imageModel.IsImageSaved();
            }
            IDNO => {
                return (TRUE) as BOOL;
            }
            IDCANCEL => {
                return (FALSE) as BOOL;
            }
            _ => {}
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn ProcessFileMenu(&mut self, hPopupMenu: HMENU) {
        let this_: *mut Self = self;
        let mut iItem: INT = 0;
        while iItem < (MAX_RECENT_FILES as i32) {
            RemoveMenu(hPopupMenu, IDM_FILE1 + (iItem) as u32, MF_BYCOMMAND);
            iItem += 1;
        }
    
        if registrySettings.strFiles[0 as usize].IsEmpty() {
            return;
        }
    
        RemoveMenu(hPopupMenu, IDM_FILEMOSTRECENTLYUSEDFILE, MF_BYCOMMAND);
    
        let mut cMenuItems: INT = GetMenuItemCount(hPopupMenu);
    
        let mut iItem: INT = 0;
        while iItem < (MAX_RECENT_FILES as i32) {
            let strFile: &mut CStringW = &mut registrySettings.strFiles[iItem as usize];
            if strFile.IsEmpty() {
                break;
            }
    
            // Condense the lengthy pathname by using '...'
            // #define MAX_RECENT_PATHNAME_DISPLAY 30
            let mut pathFile: CPath = CPath__make_0(strFile.GetString());
            pathFile.CompactPathEx(MAX_RECENT_PATHNAME_DISPLAY);
            assert!(wcslen(pathFile.GetString()) <= MAX_RECENT_PATHNAME_DISPLAY as usize);
    
            // Add an accelerator (by '&') to the item number for quick access
            let mut szText: [WCHAR; (4 + MAX_RECENT_PATHNAME_DISPLAY + 1) as usize] =
                [0 as WCHAR; (4 + MAX_RECENT_PATHNAME_DISPLAY + 1) as usize];
            StringCchPrintfW!(
                szText.as_mut_ptr(),
                _countof!(szText),
                CPP_COM_WSTR_773566D1D102.as_ptr(),
                iItem + 1,
                pathFile.GetString(),
            );
    
            let iMenuItem: INT = (cMenuItems - 2) + iItem;
            InsertMenu(
                hPopupMenu,
                iMenuItem,
                MF_BYPOSITION | MF_STRING,
                IDM_FILE1 + (iItem) as u32,
                szText.as_ptr(),
            );
    
            iItem += 1;
        }
    }
}
