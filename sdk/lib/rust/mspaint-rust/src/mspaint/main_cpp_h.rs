use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_368075081DBB: [WCHAR; 4] = [0x0070, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_3E31FFE6FE35: [WCHAR; 5] = [0x002E, 0x0070, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_461B74EEBD9E: [WCHAR; 30] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x005C, 0x0048, 0x0065, 0x006C, 0x0070, 0x005C, 0x006D, 0x0073, 0x0070, 0x0061, 0x0069, 0x006E, 0x0074, 0x002E, 0x0063, 0x0068, 0x006D, 0x0000];
pub static CPP_COM_WSTR_67DB42DC8E1B: [WCHAR; 11] = [0x006D, 0x0061, 0x0070, 0x0069, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_773566D1D102: [WCHAR; 7] = [0x0026, 0x0025, 0x0075, 0x0020, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_85D11F0B94E7: [WCHAR; 30] = [0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x006D, 0x0061, 0x0069, 0x006E, 0x0020, 0x0077, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x002E, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_C3F0B98328E3: [WCHAR; 12] = [0x005C, 0x0068, 0x0068, 0x0063, 0x0074, 0x0072, 0x006C, 0x002E, 0x006F, 0x0063, 0x0078, 0x0000];
pub static CPP_COM_WSTR_F7E2D0733EDD: [WCHAR; 4] = [0x0061, 0x0066, 0x0078, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn lstrcpynW(lpString: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
    fn PathCompactPathExW(pszOut: LPWSTR, pszSrc: LPCWSTR, cchMax: UINT, dwFlags: DWORD) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub type FN_HtmlHelpW = Option<unsafe extern "system" fn(HWND, LPCWSTR, UINT, DWORD_PTR) -> HWND>;
pub type FN_MAPISendMail =
    Option<unsafe extern "system" fn(LHANDLE, ULONG_PTR, lpMapiMessage, FLAGS, ULONG) -> ULONG>;
pub type FN_MAPISendMailW =
    Option<unsafe extern "system" fn(LHANDLE, ULONG_PTR, lpMapiMessageW, FLAGS, ULONG) -> ULONG>;

#[no_mangle]
pub static mut g_askBeforeEnlarging: BOOL = (FALSE as i32);
#[no_mangle]
pub static mut g_hinstExe: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut g_szFileName: [WCHAR; 512] = [0; 512];
pub static mut g_szMailTempFile: [WCHAR; 512] = [0; 512];
#[no_mangle]
pub static mut g_isAFile: BOOL = (FALSE as i32);
#[no_mangle]
pub static mut g_imageSaved: BOOL = (FALSE as i32);
#[no_mangle]
pub static mut g_showGrid: BOOL = (FALSE as i32);
#[no_mangle]
pub static mut g_hStatusBar: HWND = core::ptr::null_mut();
static mut s_hHHCTRL_OCX: HINSTANCE = core::ptr::null_mut(); // HtmlHelpW needs "hhctrl.ocx"
static mut s_pHtmlHelpW: FN_HtmlHelpW = None;

#[allow(non_upper_case_globals)]
const Gdiplus_ImageFormatPNG: GUID = GUID {
    Data1: 0xb96b3caf,
    Data2: 0x0728,
    Data3: 0x11d3,
    Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e],
};

static mut s_hHHCTRL_OCX_UNUSED_MARKER: u8 = 0;
























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
    let mut Path: [WCHAR; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
    if uMsg == WM_NOTIFY {
        pon = lParam as *mut OFNOTIFYW;
        if (*pon).hdr.code == CDN_TYPECHANGE {
            hParent = GetParent(hwnd);
            SendMessageW(hParent, CDM_GETFILEPATH, (_countof!(Path)) as WPARAM, Path.as_mut_ptr() as LPARAM);
            FileExtFromFilter(PathFindExtensionW(Path.as_ptr()), (*pon).lpOFN);
            SendMessageW(hParent, CDM_SETCONTROLTEXT, (cmb13) as WPARAM, PathFindFileNameW(Path.as_ptr()) as LPARAM);
            StringCchCopyW((*(*pon).lpOFN).lpstrFile, (*(*pon).lpOFN).nMaxFile as usize, Path.as_ptr());
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn OpenMailer(hWnd: HWND, mut pszPathName: LPCWSTR) -> BOOL {
    // Delete the temporary file if any
    if g_szMailTempFile[0 as usize] != 0 {
        DeleteFileW(addr_of_mut!(g_szMailTempFile) as *mut WCHAR);
        g_szMailTempFile[0 as usize] = UNICODE_NULL;
    }

    let mut strFileTitle = CStringW::new();
    if PathFileExistsW(pszPathName) != 0 && ((imageModel.IsImageSaved()) != 0) {
        strFileTitle = CStringW::from_ptr(PathFindFileNameW(pszPathName as *mut WCHAR));
    } else {
        // Not existing or not saved
        // Get the name of a temporary file
        let mut szTempDir = [0 as WCHAR; MAX_PATH as usize];
        GetTempPathW(_countof!(szTempDir) as DWORD, szTempDir.as_mut_ptr());
        if GetTempFileNameW(
            szTempDir.as_ptr(),
            CPP_COM_WSTR_F7E2D0733EDD.as_ptr(),
            0,
            addr_of_mut!(g_szMailTempFile) as *mut WCHAR,
        ) == 0
        {
            return (FALSE) as BOOL; // Failure
        }

        if PathFileExistsW(addr_of_mut!(g_szFileName) as *mut WCHAR) != 0 {
            // Set file title
            strFileTitle = CStringW::from_ptr(PathFindFileNameW(
                addr_of_mut!(g_szFileName) as *mut WCHAR,
            ));

            // Copy to the temporary file
            if CopyFileW(
                addr_of_mut!(g_szFileName) as *const WCHAR,
                addr_of_mut!(g_szMailTempFile) as *const WCHAR,
                (FALSE) as BOOL,
            ) == 0
            {
                g_szMailTempFile[0 as usize] = UNICODE_NULL;
                return (FALSE) as BOOL; // Failure
            }
        } else {
            // Set file title
            strFileTitle.LoadStringW(IDS_DEFAULTFILENAME);
            strFileTitle += CPP_COM_WSTR_3E31FFE6FE35.as_ptr();

            // Save it to the temporary file
            let hbmLocked = imageModel.LockBitmap();
            let ret = SaveDIBToFile(
                hbmLocked,
                addr_of_mut!(g_szMailTempFile) as *const WCHAR,
                (FALSE) as BOOL,
                &Gdiplus_ImageFormatPNG,
            );
            imageModel.UnlockBitmap(hbmLocked);
            if ret == 0 {
                g_szMailTempFile[0 as usize] = UNICODE_NULL;
                return (FALSE) as BOOL; // Failure
            }
        }

        // Use the temporary file
        pszPathName = addr_of_mut!(g_szMailTempFile) as *mut WCHAR as LPCWSTR;
    }

    // Load "mapi32.dll"
    let hMAPI = LoadLibraryW(CPP_COM_WSTR_67DB42DC8E1B.as_ptr());
    if hMAPI.is_null() {
        return (FALSE) as BOOL; // Failure
    }

    // Attachment
    let mut attachmentW: MapiFileDescW = core::mem::zeroed();
    attachmentW.nPosition = (1 as u32).wrapping_neg() as ULONG;
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
    if (pMAPISendMailW).map_or(0, |f| f as usize) as usize != 0 {
        (pMAPISendMailW.unwrap_unchecked())(
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
    attachment.nPosition = (1 as u32).wrapping_neg() as ULONG;
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
    if (pMAPISendMail).map_or(0, |f| f as usize) as usize != 0 {
        (pMAPISendMail.unwrap_unchecked())(
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

    if s_pHtmlHelpW.is_none() {
        return core::ptr::null_mut();
    }

    return (s_pHtmlHelpW.unwrap_unchecked())(hwndCaller, pszFile, uCommand, dwData);
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
    if mainWindow.DoCreate().is_null() {
        MessageBox(core::ptr::null_mut(), CPP_COM_WSTR_85D11F0B94E7.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);
        return 1;
    }

    // Initialize imageModel
    // `__argc`/`__targv` are msvcrt DATA exports, read through their import slots (prelude).
    if *__imp___argc < 2 || DoLoadImageFile((mainWindow).op_to_HWND(), *(*__imp___wargv).add(1), (TRUE) as BOOL).is_null() {
        InitializeImage(core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL);
    }

    // Make the window visible on the screen
    mainWindow.__base0.ShowWindow((registrySettings.WindowPlacement.showCmd) as i32);

    // Load the access keys
    let hAccel: HACCEL = LoadAcceleratorsW(hInstance, MAKEINTRESOURCEW!(800));

    // The message loop
    let mut msg: MSG = core::mem::zeroed();
    while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
        if fontsDialog.__base0.IsWindow() != 0 && fontsDialog.__base0.IsDialogMessageW(core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }

        if TranslateAcceleratorW((mainWindow).op_to_HWND(), hAccel, core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }

        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessage(core::ptr::addr_of_mut!(msg));
    }

    // Unload the access keys
    DestroyAcceleratorTable(hAccel);

    // Write back settings to registry
    registrySettings.Store();

    if g_szMailTempFile[0] != 0 {
        DeleteFileW(core::ptr::addr_of_mut!(g_szMailTempFile) as *mut u16);
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
        LoadStringW(g_hinstExe, IDS_DEFAULTFILENAME, addr_of_mut!(g_szFileName) as *mut WCHAR, (_countof!(g_szFileName)) as i32);
    
        let mut strTitle = CStringW::new();
        atl_format_id!(strTitle, IDS_WINDOWTITLE, PathFindFileName(addr_of!(g_szFileName) as *const WCHAR));
    
        let rc: &mut RECT = &mut *addr_of_mut!(registrySettings.WindowPlacement.rcNormalPosition);
        return (*this_).Create(HWND_DESKTOP, rc, strTitle.GetString(), WS_OVERLAPPEDWINDOW, WS_EX_ACCEPTFILES, 0);
    }
    pub unsafe fn GetOpenFileNameW(&mut self, pszFile: LPWSTR, cchMaxFile: INT) -> BOOL {
        let this_: *mut Self = self;
        static mut ofn: OPENFILENAMEW = unsafe { core::mem::zeroed() };
        static mut strFilter: CStringW = CStringW::new();
    
        if ofn.lStructSize == 0
        {
            // The "All Files" item text
            let mut strAllPictureFiles = CStringW::new();
            strAllPictureFiles.LoadStringW(IDS_ALLPICTUREFILES);
    
            // Get the import filter
            let mut aguidFileTypesI = CSimpleArray::<GUID>::new();
            CImage::GetImporterFilterString(&mut *core::ptr::addr_of_mut!(strFilter), &mut aguidFileTypesI, (&mut strAllPictureFiles).GetString(),
                                            ATL_CImage_ExcludeFlags_excludeDefaultLoad, 0x007Cu16);
            (*core::ptr::addr_of_mut!(strFilter)).Replace(0x007Cu16, UNICODE_NULL);
    
            // Initializing the OPENFILENAME structure for GetOpenFileName
            ZeroMemory!(core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAMEW>());
            ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
            ofn.hwndOwner   = (*this_).__base0.m_hWnd;
            ofn.hInstance   = g_hinstExe;
            ofn.lpstrFilter = (*core::ptr::addr_of_mut!(strFilter)).GetString();
            ofn.Flags       = OFN_EXPLORER | OFN_HIDEREADONLY;
            ofn.lpstrDefExt = CPP_COM_WSTR_368075081DBB.as_ptr();
        }
    
        ofn.lpstrFile = pszFile;
        ofn.nMaxFile  = cchMaxFile as DWORD;
        return GetOpenFileNameW(core::ptr::addr_of_mut!(ofn));
    }
    pub unsafe fn GetSaveFileNameW(&mut self, pszFile: LPWSTR, cchMaxFile: INT) -> BOOL {
        let this_: *mut Self = self;
        #[allow(non_upper_case_globals)]
        static mut sfn: OPENFILENAMEW = unsafe { core::mem::zeroed() };
        #[allow(non_upper_case_globals)]
        static mut strFilter: CStringW = CStringW::new();
    
        if (*core::ptr::addr_of_mut!(sfn)).lStructSize == 0 {
            // Get the export filter
            let mut aguidFileTypesE = CSimpleArray::<GUID>::new();
            CImage::GetExporterFilterString(
                &mut *core::ptr::addr_of_mut!(strFilter),
                &mut aguidFileTypesE,
                core::ptr::null_mut(),
                ATL_CImage_ExcludeFlags_excludeDefaultSave,
                0x007Cu16,
            );
            (*core::ptr::addr_of_mut!(strFilter)).Replace(0x007Cu16, UNICODE_NULL);
    
            // Initializing the OPENFILENAME structure for GetSaveFileName
            ZeroMemory!(
                core::ptr::addr_of_mut!(sfn) as *mut core::ffi::c_void,
                core::mem::size_of::<OPENFILENAMEW>(),
            );
            (*core::ptr::addr_of_mut!(sfn)).lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
            (*core::ptr::addr_of_mut!(sfn)).hwndOwner = (*this_).__base0.m_hWnd;
            (*core::ptr::addr_of_mut!(sfn)).hInstance = g_hinstExe;
            (*core::ptr::addr_of_mut!(sfn)).lpstrFilter = (*core::ptr::addr_of_mut!(strFilter)).GetString();
            (*core::ptr::addr_of_mut!(sfn)).Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_ENABLEHOOK;
            (*core::ptr::addr_of_mut!(sfn)).lpfnHook = Some(OFNHookProc);
            (*core::ptr::addr_of_mut!(sfn)).lpstrDefExt = CPP_COM_WSTR_368075081DBB.as_ptr();
    
            let pchDotExt: LPWSTR = PathFindExtensionW(pszFile);
            if *pchDotExt == UNICODE_NULL {
                // Choose PNG
                StringCchCatW(pszFile, (cchMaxFile) as usize, CPP_COM_WSTR_3E31FFE6FE35.as_ptr());
                let mut i: INT = 0;
                while i < aguidFileTypesE.GetSize() {
                    if IsEqualGUID!(aguidFileTypesE.GetAt(i), &Gdiplus_ImageFormatPNG as *const GUID) != false {
                        (*core::ptr::addr_of_mut!(sfn)).nFilterIndex = (i + 1) as DWORD;
                        break;
                    }
                    i += 1;
                }
            }
        }
    
        (*core::ptr::addr_of_mut!(sfn)).lpstrFile = pszFile;
        (*core::ptr::addr_of_mut!(sfn)).nMaxFile = cchMaxFile as DWORD;
        return GetSaveFileNameW(core::ptr::addr_of_mut!(sfn));
    }
    pub unsafe fn ChooseColorW(&mut self, prgbColor: *mut COLORREF) -> BOOL {
        let this_: *mut Self = self;
        static mut choosecolor: CHOOSECOLOR = unsafe { core::mem::zeroed() };
        static mut custColors: [COLORREF; 16] = [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff
        ];
    
        let cc: *mut CHOOSECOLOR = core::ptr::addr_of_mut!(choosecolor);
        let cust: *mut COLORREF = core::ptr::addr_of_mut!(custColors) as *mut COLORREF;
    
        if (*cc).lStructSize == 0 {
            // Initializing the CHOOSECOLOR structure for ChooseColor
            ZeroMemory!(cc as *mut core::ffi::c_void, core::mem::size_of::<CHOOSECOLOR>());
            (*cc).lStructSize = core::mem::size_of::<CHOOSECOLOR>() as DWORD;
            (*cc).hwndOwner = (*this_).__base0.m_hWnd;
            (*cc).lpCustColors = cust;
        }
    
        (*cc).Flags = CC_RGBINIT;
        (*cc).rgbResult = *prgbColor;
        if ChooseColor(cc) == 0 {
            return (FALSE) as BOOL;
        }
    
        *prgbColor = (*cc).rgbResult;
        return (TRUE) as BOOL;
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
            (*this_).__base0.PostMessageW(WM_COMMAND, id as WPARAM, 0);
        }
    
        DestroyMenu(hMenu);
    }
    pub unsafe fn CanUndo(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible((textEditWindow).op_to_HWND()) != 0 {
            return textEditWindow.__base0.SendMessageW__0(EM_CANUNDO, 0, 0) as BOOL;
        }
        if selectionModel.m_bShow != 0 && toolsModel.IsSelection() != 0 {
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
        if toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible(textEditWindow.op_to_HWND()) != 0 {
            return IsClipboardFormatAvailable(CF_UNICODETEXT);
        }
    
        return ((IsClipboardFormatAvailable(CF_ENHMETAFILE) != 0)
            || (IsClipboardFormatAvailable(CF_DIB) != 0)
            || (IsClipboardFormatAvailable(CF_BITMAP) != 0)) as BOOL;
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
        let mut style: DWORD = SBARS_SIZEGRIP | WS_CHILD | (if ((registrySettings.ShowStatusBar) != 0) { WS_VISIBLE } else { 0 });
        g_hStatusBar = CreateWindowExW(0, STATUSCLASSNAME, core::ptr::null_mut(), style, 0, 0, 0, 0, (*this_).__base0.m_hWnd,
                                         core::ptr::null_mut(), g_hinstExe, core::ptr::null_mut());
        SendMessageW(g_hStatusBar, SB_SETMINHEIGHT, 21, 0);
    
        // Create the tool box
        toolBoxContainer.DoCreate((*this_).__base0.m_hWnd);
    
        // Create the palette window
        let mut rcEmpty: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 }; // Rely on WM_SIZE
        style = WS_CHILD | (if ((registrySettings.ShowPalette) != 0) { WS_VISIBLE } else { 0 });
        paletteWindow.Create((*this_).__base0.m_hWnd, &mut rcEmpty, core::ptr::null_mut(), style, WS_EX_STATICEDGE, 0);
    
        // Create the canvas
        style = WS_CHILD | WS_GROUP | WS_HSCROLL | WS_VSCROLL | WS_VISIBLE;
        canvasWindow.Create((*this_).__base0.m_hWnd, &mut rcEmpty, core::ptr::null_mut(), style, WS_EX_CLIENTEDGE, 0);
    
        // Create and show the miniature if necessary
        if ((registrySettings.ShowThumbnail) != 0) {
            miniature.DoCreate((*this_).__base0.m_hWnd);
            miniature.__base0.ShowWindow((SW_SHOWNOACTIVATE as i32));
        }
    
        // Set icon
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_BIG) as WPARAM, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_SMALL) as WPARAM, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
    
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        registrySettings.WindowPlacement.length = (core::mem::size_of::<WINDOWPLACEMENT>()) as _;
        (*this_).__base0.GetWindowPlacement(core::ptr::addr_of_mut!(registrySettings.WindowPlacement));
    
        DoHtmlHelpW(core::ptr::null_mut(), core::ptr::null_mut(), HH_CLOSE_ALL, 0);
    
        if !s_hHHCTRL_OCX.is_null()
        {
            FreeLibrary(s_hHHCTRL_OCX);
            s_hHHCTRL_OCX = core::ptr::null_mut();
            s_pHtmlHelpW = None;
        }
    
        (*this_).__base0.SetMenu(core::ptr::null_mut());
        if !(*this_).m_hMenu.is_null()
        {
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
        let textShown: BOOL = ((toolsModel.GetActiveTool() == TOOL_TEXT) && (IsWindowVisible((textEditWindow).op_to_HWND()) as usize != 0)) as BOOL;
        let mut dwStart: DWORD = 0;
        let mut dwEnd: DWORD = 0;
        if textShown != 0 {
            textEditWindow.__base0.SendMessageW__0(EM_GETSEL, core::ptr::addr_of_mut!(dwStart) as WPARAM, core::ptr::addr_of_mut!(dwEnd) as LPARAM);
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
        let mut test: [i32; 3] = [
            LOWORD!(lParam) as i32 - 260,
            LOWORD!(lParam) as i32 - 140,
            LOWORD!(lParam) as i32 - 20,
        ];
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
            __c if __c == VK_ESCAPE as _ => {
                canvasWindow.__base0.PostMessageW(nMsg, wParam, lParam);
            }
            __c if __c == VK_LEFT as _ => {
                selectionModel.moveSelection(-1, 0);
            }
            __c if __c == VK_RIGHT as _ => {
                selectionModel.moveSelection(1, 0);
            }
            __c if __c == VK_UP as _ => {
                selectionModel.moveSelection(0, -1);
            }
            __c if __c == VK_DOWN as _ => {
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
        if canvasWindow.m_drawing != 0 && !GetCapture().is_null() {
            ATLTRACE!(b"locking!\n\0".as_ptr() as *const i8);
            return 0;
        }
    
        let mut textShown: BOOL = (toolsModel.GetActiveTool() == TOOL_TEXT && IsWindowVisible(textEditWindow.op_to_HWND()) != 0) as BOOL;
        'sw: {
            match (LOWORD!(wParam)) as u32 {
                IDM_HELPINFO => {
                    let mut infotitle: [WCHAR; 100] = [0 as WCHAR; 100];
                    let mut infotext: [WCHAR; 200] = [0 as WCHAR; 200];
                    LoadStringW(g_hinstExe, IDS_INFOTITLE, infotitle.as_mut_ptr(), _countof!(infotitle) as INT);
                    LoadStringW(g_hinstExe, IDS_INFOTEXT, infotext.as_mut_ptr(), _countof!(infotext) as INT);
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
                    if (*this_).ConfirmSave() != 0 {
                        InitializeImage(core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as BOOL);
                    }
                    break 'sw;
                }
                IDM_FILEOPEN => {
                    {
                        let mut szFileName: [WCHAR; (MAX_LONG_PATH) as _] = core::ptr::read(CPP_COM_WSTR_96A296D224F2.as_ptr() as *const [WCHAR; (MAX_LONG_PATH) as _]);
                        if (*this_).ConfirmSave() != 0 && (*this_).GetOpenFileNameW(szFileName.as_mut_ptr(), _countof!(szFileName) as INT) != 0 {
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
                    psd.lStructSize = core::mem::size_of::<PAGESETUPDLG>() as DWORD;
                    psd.hwndOwner = (*this_).__base0.m_hWnd;
                    PageSetupDlg(core::ptr::addr_of_mut!(psd));
                    break 'sw;
                }
                IDM_FILEPRINT => {
                    // TODO: Test whether it actually works
                    let mut pd: PRINTDLG = core::mem::zeroed();
                    ZeroMemory!(core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void, core::mem::size_of::<PRINTDLG>());
                    pd.lStructSize = core::mem::size_of::<PRINTDLG>() as DWORD;
                    pd.hwndOwner = (*this_).__base0.m_hWnd;
                    pd.hDevMode = core::ptr::null_mut();  // freed by user
                    pd.hDevNames = core::ptr::null_mut();  // freed by user
                    pd.Flags = PD_USEDEVMODECOPIESANDCOLLATE | PD_RETURNDC;
                    pd.nCopies = 1;
                    pd.nFromPage = 0xffff;
                    pd.nToPage = 0xffff;
                    pd.nMinPage = 1;
                    pd.nMaxPage = 0xffff;
                    if PrintDlg(core::ptr::addr_of_mut!(pd)) == (TRUE as i32) {
                        BitBlt(pd.hDC, 0, 0, imageModel.GetWidth(), imageModel.GetHeight(), imageModel.GetDC(), 0, 0, SRCCOPY);
                        DeleteDC(pd.hDC);
                    }
                    if !pd.hDevMode.is_null() {
                        GlobalFree(pd.hDevMode);
                    }
                    if !pd.hDevNames.is_null() {
                        GlobalFree(pd.hDevNames);
                    }
                    break 'sw;
                }
                IDM_FILESEND => {
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    if OpenMailer((*this_).__base0.m_hWnd, addr_of!(g_szFileName) as *const WCHAR) == 0 {
                        ShowError((IDS_CANTSENDMAIL) as INT);
                    }
                    break 'sw;
                }
                IDM_FILEASWALLPAPERPLANE => {
                    RegistrySettings::SetWallpaper(addr_of!(g_szFileName) as *const WCHAR, RegistrySettings_WallpaperStyle_TILED);
                    break 'sw;
                }
                IDM_FILEASWALLPAPERCENTERED => {
                    RegistrySettings::SetWallpaper(addr_of!(g_szFileName) as *const WCHAR, RegistrySettings_WallpaperStyle_CENTERED);
                    break 'sw;
                }
                IDM_FILEASWALLPAPERSTRETCHED => {
                    RegistrySettings::SetWallpaper(addr_of!(g_szFileName) as *const WCHAR, RegistrySettings_WallpaperStyle_STRETCHED);
                    break 'sw;
                }
                IDM_FILE1 | IDM_FILE2 | IDM_FILE3 | IDM_FILE4 => {
                    {
                        let iFile: INT = (LOWORD!(wParam) as INT) - (IDM_FILE1 as i32);
                        if (*this_).ConfirmSave() != 0 {
                            DoLoadImageFile((*this_).__base0.m_hWnd, registrySettings.strFiles[iFile as usize].GetString(), (TRUE) as BOOL);
                        }
                        break 'sw;
                    }
                }
                IDM_EDITUNDO => {
                    if textShown != 0 {
                        textEditWindow.__base0.PostMessageW(WM_UNDO, 0, 0);
                        break 'sw;
                    }
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    imageModel.Undo((FALSE) as BOOL);
                    break 'sw;
                }
                IDM_EDITREDO => {
                    if textShown != 0 {
                        // There is no "WM_REDO" in EDIT control
                        break 'sw;
                    }
                    canvasWindow.OnEndDraw((FALSE) as BOOL);
                    imageModel.Redo();
                    break 'sw;
                }
                IDM_EDITCOPY => {
                    if textShown != 0 {
                        textEditWindow.__base0.SendMessageW__0(WM_COPY, 0, 0);
                        break 'sw;
                    }
                    if selectionModel.m_bShow == 0 || (*this_).__base0.OpenClipboard() == 0 {
                        break 'sw;
                    }
    
                    EmptyClipboard();
    
                    selectionModel.TakeOff();
    
                    {
                        let hbmCopy: HBITMAP = selectionModel.GetSelectionContents();
                        let hGlobal: HGLOBAL = BitmapToClipboardDIB(hbmCopy);
                        if !hGlobal.is_null() {
                            SetClipboardData(CF_DIB, hGlobal);
                        } else {
                            ShowOutOfMemory();
                            imageModel.ClearHistory();
                        }
                        DeleteObject((hbmCopy).cast());
                    }
    
                    CloseClipboard();
                    break 'sw;
                }
                IDM_EDITCUT => {
                    if textShown != 0 {
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
                    if textShown != 0 {
                        textEditWindow.__base0.SendMessageW__0(WM_PASTE, 0, 0);
                        break 'sw;
                    }
    
                    if (*this_).__base0.OpenClipboard() == 0 {
                        break 'sw;
                    }
    
                    // In many cases, CF_ENHMETAFILE provides a better image than CF_DIB
                    if IsClipboardFormatAvailable(CF_ENHMETAFILE) != 0 {
                        let hEMF: HENHMETAFILE = GetClipboardData(CF_ENHMETAFILE) as HENHMETAFILE;
                        if !hEMF.is_null() {
                            let hbm: HBITMAP = BitmapFromHEMF(hEMF);
                            DeleteEnhMetaFile(hEMF);
                            if !hbm.is_null() {
                                (*this_).InsertSelectionFromHBITMAP(hbm, (*this_).__base0.m_hWnd);
                                CloseClipboard();
                                break 'sw;
                            }
                        }
                    }
    
                    // In many cases, CF_DIB provides a better image than CF_BITMAP
                    if IsClipboardFormatAvailable(CF_DIB) != 0 {
                        let hbm: HBITMAP = BitmapFromClipboardDIB(GetClipboardData(CF_DIB));
                        if !hbm.is_null() {
                            (*this_).InsertSelectionFromHBITMAP(hbm, (*this_).__base0.m_hWnd);
                            CloseClipboard();
                            break 'sw;
                        }
                    }
    
                    // The last resort
                    if IsClipboardFormatAvailable(CF_BITMAP) != 0 {
                        let hbm: HBITMAP = GetClipboardData(CF_BITMAP) as HBITMAP;
                        if !hbm.is_null() {
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
                        let hbmSelection: HBITMAP = selectionModel.GetSelectionContents();
                        if !hbmSelection.is_null() {
                            imageModel.PushImageForUndo__1(hbmSelection);
                            selectionModel.HideSelection();
                            imageModel.NotifyImageChanged();
                        }
                        break 'sw;
                    }
                }
                IDM_EDITDELETESELECTION => {
                    {
                        if textShown != 0 {
                            textEditWindow.__base0.SendMessageW__0(WM_CLEAR, 0, 0);
                            break 'sw;
                        }
                        'sw3: {
                            match toolsModel.GetActiveTool() {
                                TOOL_FREESEL | TOOL_RECTSEL => {
                                    selectionModel.DeleteSelection();
                                    break 'sw3;
                                }
    
                                TOOL_TEXT => {
                                    canvasWindow.OnEndDraw((TRUE) as BOOL);
                                    break 'sw3;
                                }
                                _ => {
                                    break 'sw3;
                                }
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_EDITSELECTALL => {
                    {
                        if textShown != 0 {
                            textEditWindow.__base0.SendMessageW__0(EM_SETSEL, 0, -1);
                            break 'sw;
                        }
                        let hToolbar: HWND = FindWindowEx(toolBoxContainer.__base0.m_hWnd, core::ptr::null_mut(), TOOLBARCLASSNAME, core::ptr::null_mut());
                        SendMessageW(hToolbar, TB_CHECKBUTTON, (ID_RECTSEL) as WPARAM, MAKELPARAM!(TRUE, 0));
                        toolsModel.selectAll();
                        canvasWindow.__base0.Invalidate((TRUE) as BOOL);
                        break 'sw;
                    }
                }
                IDM_EDITCOPYTO => {
                    {
                        let mut szFileName: [WCHAR; (MAX_LONG_PATH) as _] = [0 as WCHAR; (MAX_LONG_PATH) as _];
                        LoadStringW(g_hinstExe, IDS_DEFAULTFILENAME, szFileName.as_mut_ptr(), _countof!(szFileName) as INT);
                        if (*this_).GetSaveFileNameW(szFileName.as_mut_ptr(), _countof!(szFileName) as INT) != 0 {
                            let hbmSelection: HBITMAP = selectionModel.GetSelectionContents();
                            if hbmSelection.is_null() {
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
                        let mut szFileName: [WCHAR; (MAX_LONG_PATH) as _] = core::ptr::read(CPP_COM_WSTR_96A296D224F2.as_ptr() as *const [WCHAR; (MAX_LONG_PATH) as _]);
                        if (*this_).GetOpenFileNameW(szFileName.as_mut_ptr(), _countof!(szFileName) as INT) != 0 {
                            let hbmNew: HBITMAP = DoLoadImageFile((*this_).__base0.m_hWnd, szFileName.as_ptr(), (FALSE) as BOOL);
                            if !hbmNew.is_null() {
                                (*this_).InsertSelectionFromHBITMAP(hbmNew, (*this_).__base0.m_hWnd);
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_COLORSEDITPALETTE => {
                    {
                        let mut rgbColor: COLORREF = paletteModel.GetFgColor();
                        if (*this_).ChooseColorW(core::ptr::addr_of_mut!(rgbColor)) != 0 {
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
                        if selectionModel.m_bShow != 0 {
                            selectionModel.InvertSelection();
                        } else {
                            imageModel.InvertColors();
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGEDELETEIMAGE => {
                    imageModel.PushImageForUndo__0();
                    Rect__0(imageModel.GetDC(), 0, 0, imageModel.GetWidth(), imageModel.GetHeight(), paletteModel.GetBgColor(), paletteModel.GetBgColor(), 0, (TRUE as i32));
                    imageModel.NotifyImageChanged();
                    break 'sw;
                }
                IDM_IMAGEROTATEMIRROR => {
                    {
                        let mut waitCursor: CWaitCursor = CWaitCursor__make_0();
                        canvasWindow.updateScrollPos(0, 0);
                        'sw2: {
                            match mirrorRotateDialog.DoModal(mainWindow.__base0.m_hWnd, 0) {
                                1 => { /* flip horizontally */
                                    {
                                        if selectionModel.m_bShow != 0 {
                                            selectionModel.FlipHorizontally();
                                        } else {
                                            imageModel.FlipHorizontally();
                                        }
                                        break 'sw2;
                                    }
                                }
                                2 => { /* flip vertically */
                                    {
                                        if selectionModel.m_bShow != 0 {
                                            selectionModel.FlipVertically();
                                        } else {
                                            imageModel.FlipVertically();
                                        }
                                        break 'sw2;
                                    }
                                }
                                3 => { /* rotate 90 degrees */
                                    {
                                        if selectionModel.m_bShow != 0 {
                                            selectionModel.RotateNTimes90Degrees(1);
                                        } else {
                                            imageModel.RotateNTimes90Degrees(1);
                                        }
                                        break 'sw2;
                                    }
                                }
                                4 => { /* rotate 180 degrees */
                                    {
                                        if selectionModel.m_bShow != 0 {
                                            selectionModel.RotateNTimes90Degrees(2);
                                        } else {
                                            imageModel.RotateNTimes90Degrees(2);
                                        }
                                        break 'sw2;
                                    }
                                }
                                5 => { /* rotate 270 degrees */
                                    {
                                        if selectionModel.m_bShow != 0 {
                                            selectionModel.RotateNTimes90Degrees(3);
                                        } else {
                                            imageModel.RotateNTimes90Degrees(3);
                                        }
                                        break 'sw2;
                                    }
                                }
                                _ => {
                                    break 'sw2;
                                }
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGEATTRIBUTES => {
                    {
                        if attributesDialog.DoModal(mainWindow.__base0.m_hWnd, 0) != 0 {
                            let mut waitCursor: CWaitCursor = CWaitCursor__make_0();
                            if attributesDialog.m_bBlackAndWhite != 0 && imageModel.IsBlackAndWhite() == 0 {
                                let mut strText = CStringW::from_resource(IDS_LOSECOLOR);
                                let mut strTitle = CStringW::from_resource(IDS_PROGRAMNAME);
                                let id: INT = (*this_).__base0.MessageBoxW(strText.GetString(), strTitle.GetString(), MB_ICONINFORMATION | MB_YESNOCANCEL);
                                if id != (IDYES as i32) {
                                    break 'sw;
                                }
    
                                imageModel.PushBlackAndWhite();
                            }
    
                            if imageModel.GetWidth() != attributesDialog.newWidth ||
                                imageModel.GetHeight() != attributesDialog.newHeight {
                                imageModel.Crop(attributesDialog.newWidth, attributesDialog.newHeight, 0, 0);
                            }
                        }
                        break 'sw;
                    }
                }
                IDM_IMAGESTRETCHSKEW => {
                    {
                        if stretchSkewDialog.DoModal(mainWindow.__base0.m_hWnd, 0) != 0 {
                            let mut waitCursor: CWaitCursor = CWaitCursor__make_0();
                            if selectionModel.m_bShow != 0 {
                                selectionModel.StretchSkew(stretchSkewDialog.percentage.x, stretchSkewDialog.percentage.y,
                                                           stretchSkewDialog.angle.x, stretchSkewDialog.angle.y);
                            } else {
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
                        let hbmCopy: HBITMAP = selectionModel.GetSelectionContents();
                        imageModel.PushImageForUndo__1(hbmCopy);
                        selectionModel.HideSelection();
                        break 'sw;
                    }
                }
                IDM_VIEWTOOLBOX => {
                    registrySettings.ShowToolBox = (toolBoxContainer.__base0.IsWindowVisible() == 0) as DWORD;
                    toolBoxContainer.__base0.ShowWindow(if registrySettings.ShowToolBox != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_VIEWCOLORPALETTE => {
                    registrySettings.ShowPalette = (paletteWindow.__base0.IsWindowVisible() == 0) as DWORD;
                    paletteWindow.__base0.ShowWindow(if registrySettings.ShowPalette != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_VIEWSTATUSBAR => {
                    registrySettings.ShowStatusBar = (IsWindowVisible(g_hStatusBar) == 0) as DWORD;
                    ShowWindow(g_hStatusBar, if registrySettings.ShowStatusBar != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
                    (*this_).alignChildrenToMainWindow();
                    break 'sw;
                }
                IDM_FORMATICONBAR => {
                    if toolsModel.GetActiveTool() == TOOL_TEXT {
                        if fontsDialog.__base0.IsWindow() == 0 {
                            fontsDialog.Create(mainWindow.op_to_HWND(), 0);
                        }
                        registrySettings.ShowTextTool = (IsWindowVisible(fontsDialog.op_to_HWND()) == 0) as DWORD;
                        fontsDialog.__base0.ShowWindow(if registrySettings.ShowTextTool != 0 { (SW_SHOW as i32) } else { (SW_HIDE as i32) });
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
                    registrySettings.ShowThumbnail = (IsWindowVisible(miniature.op_to_HWND()) == 0) as DWORD;
                    miniature.DoCreate((*this_).__base0.m_hWnd);
                    miniature.__base0.ShowWindow(if registrySettings.ShowThumbnail != 0 { (SW_SHOWNOACTIVATE as i32) } else { (SW_HIDE as i32) });
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
                    fullscreenWindow.__base0.ShowWindow((SW_SHOWMAXIMIZED as i32));
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
        0
    }
    pub unsafe fn OnMouseWheel(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut zDelta: INT = HIWORD!(wParam) as SHORT as INT;
    
        if GetKeyState(0x11 /* VK_CONTROL */) < 0 { // Ctrl+Wheel
            if zDelta < 0 {
                if toolsModel.GetZoom() > (MIN_ZOOM as i32) {
                    canvasWindow.zoomTo(toolsModel.GetZoom() / 2, 0, 0);
                }
            } else if zDelta > 0 {
                if toolsModel.GetZoom() < (MAX_ZOOM as i32) {
                    canvasWindow.zoomTo(toolsModel.GetZoom() * 2, 0, 0);
                }
            }
        } else { // Wheel only
            let mut nCount: UINT = 3;
            if GetAsyncKeyState(0x10 /* VK_SHIFT */) < 0 {
                // #define SPI_GETWHEELSCROLLCHARS 0x006C  // Needed for pre-NT6 PSDK
                SystemParametersInfoW(SPI_GETWHEELSCROLLCHARS, 0, (core::ptr::addr_of_mut!(nCount)).cast(), 0);
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
                SystemParametersInfoW(SPI_GETWHEELSCROLLLINES, 0, (core::ptr::addr_of_mut!(nCount)).cast(), 0);
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
    
        if IsWindowVisible(g_hStatusBar) != 0
        {
            GetWindowRect(g_hStatusBar, core::ptr::addr_of_mut!(rc));
            rcSpace.bottom -= rc.bottom - rc.top;
        }
    
        let mut hDWP: HDWP = BeginDeferWindowPos(3);
    
        if IsWindowVisible(toolBoxContainer.op_to_HWND()) != 0
        {
            if registrySettings.Bar2ID == BAR2ID_RIGHT
            {
                hDWP = DeferWindowPos(hDWP, toolBoxContainer.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.right - (CX_TOOLBAR as i32), rcSpace.top,
                                        (CX_TOOLBAR as i32), rcSpace.bottom - rcSpace.top,
                                        uFlags);
                rcSpace.right -= (CX_TOOLBAR as i32);
            }
            else
            {
                hDWP = DeferWindowPos(hDWP, toolBoxContainer.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.top,
                                        (CX_TOOLBAR as i32), rcSpace.bottom - rcSpace.top,
                                        uFlags);
                rcSpace.left += (CX_TOOLBAR as i32);
            }
        }
    
        if IsWindowVisible(paletteWindow.op_to_HWND()) != 0
        {
            if registrySettings.Bar1ID == BAR1ID_BOTTOM
            {
                hDWP = DeferWindowPos(hDWP, paletteWindow.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.bottom - (CY_PALETTE as i32),
                                        rcSpace.right - rcSpace.left, (CY_PALETTE as i32),
                                        uFlags);
                rcSpace.bottom -= (CY_PALETTE as i32);
            }
            else
            {
                hDWP = DeferWindowPos(hDWP, paletteWindow.op_to_HWND(), core::ptr::null_mut(),
                                        rcSpace.left, rcSpace.top,
                                        rcSpace.right - rcSpace.left, (CY_PALETTE as i32),
                                        uFlags);
                rcSpace.top += (CY_PALETTE as i32);
            }
        }
    
        if canvasWindow.__base0.IsWindow() != 0
        {
            hDWP = DeferWindowPos(hDWP, canvasWindow.op_to_HWND(), core::ptr::null_mut(),
                                    rcSpace.left, rcSpace.top,
                                    rcSpace.right - rcSpace.left, rcSpace.bottom - rcSpace.top,
                                    uFlags);
        }
    
        EndDeferWindowPos(hDWP);
    }
    pub unsafe fn saveImage(&mut self, mut overwrite: BOOL) {
        let this_: *mut Self = self;
        canvasWindow.OnEndDraw((FALSE) as BOOL);
    
        // Is the extension not supported?
        let pchDotExt: PWCHAR = PathFindExtensionW(addr_of_mut!(g_szFileName) as *mut u16);
        if !pchDotExt.is_null() && *pchDotExt != 0 && CImageDx::IsExtensionSupported(pchDotExt) == 0 {
            // Remove the extension
            PathRemoveExtensionW(addr_of_mut!(g_szFileName) as *mut u16);
            // No overwrite
            overwrite = (FALSE as i32);
        }
    
        if g_isAFile != 0 && overwrite != 0 {
            imageModel.SaveImage(addr_of_mut!(g_szFileName) as *mut u16);
        } else if (*this_).GetSaveFileNameW(
            addr_of_mut!(g_szFileName) as *mut u16,
            (_countof!(g_szFileName)) as i32,
        ) != 0
        {
            imageModel.SaveImage(addr_of_mut!(g_szFileName) as *mut u16);
        }
    }
    pub unsafe fn InsertSelectionFromHBITMAP(&mut self, bitmap: HBITMAP, window: HWND) {
        let this_: *mut Self = self;
        let width: i32 = GetDIBWidth(bitmap);
        let height: i32 = GetDIBHeight(bitmap);
        let mut curWidth: i32 = imageModel.GetWidth();
        let mut curHeight: i32 = imageModel.GetHeight();
    
        if width > curWidth || height > curHeight {
            let mut shouldEnlarge: BOOL = (TRUE as i32);
    
            if g_askBeforeEnlarging != 0 {
                let mut programname: [WCHAR; 20] = [0; 20];
                let mut shouldEnlargePromptText: [WCHAR; 100] = [0; 100];
    
                LoadStringW(g_hinstExe, IDS_PROGRAMNAME, programname.as_mut_ptr(), (_countof!(programname)) as i32);
                LoadStringW(g_hinstExe, IDS_ENLARGEPROMPTTEXT, shouldEnlargePromptText.as_mut_ptr(), (_countof!(shouldEnlargePromptText)) as i32);
    
                match ((*this_).__base0.MessageBoxW(shouldEnlargePromptText.as_mut_ptr(), programname.as_mut_ptr(), MB_YESNOCANCEL | MB_ICONQUESTION)) as u32 {
                    IDYES => {
                    }
                    IDNO => {
                        shouldEnlarge = (FALSE as i32);
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
        selectionModel.m_bShow = (TRUE as i32);
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
            CStringW::from_resource(IDS_SAVEPROMPTTEXT).GetString(),
            PathFindFileName(addr_of_mut!(g_szFileName) as *mut WCHAR)
        );
    
        match ((*this_).__base0.MessageBoxW(
            strSavePromptText.GetString(),
            strProgramName.GetString(),
            MB_YESNOCANCEL | MB_ICONQUESTION,
        )) as u32 {
            IDYES => {
                (*this_).saveImage((TRUE) as BOOL);
                return imageModel.IsImageSaved() as BOOL;
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
    
        let cMenuItems: INT = GetMenuItemCount(hPopupMenu);
    
        let mut iItem: INT = 0;
        while iItem < (MAX_RECENT_FILES as i32) {
            let strFile = &mut registrySettings.strFiles[iItem as usize];
            if strFile.IsEmpty() {
                break;
            }
    
            // Condense the lengthy pathname by using '...'
            // CPath pathFile(strFile):
            let mut pathFileBuf = [0u16; MAX_PATH as usize];
            lstrcpynW(pathFileBuf.as_mut_ptr(), strFile.GetString(), MAX_PATH as INT);
            // pathFile.CompactPathEx(COMMENT_TOKEN_ECHO____);
            PathCompactPathExW(
                pathFileBuf.as_mut_ptr(),
                pathFileBuf.as_ptr(),
                MAX_RECENT_PATHNAME_DISPLAY as UINT,
                0,
            );
            assert!(wcslen(pathFileBuf.as_ptr()) <= (MAX_RECENT_PATHNAME_DISPLAY as usize));
    
            // Add an accelerator (by '&') to the item number for quick access
            let mut szText = [0u16; (4 + MAX_RECENT_PATHNAME_DISPLAY + 1) as usize];
            StringCchPrintfW!(
                szText.as_mut_ptr(),
                _countof!(szText),
                CPP_COM_WSTR_773566D1D102.as_ptr(),
                iItem + 1,
                pathFileBuf.as_ptr(),
            );
    
            let iMenuItem: INT = (cMenuItems - 2) + iItem;
            InsertMenu(
                hPopupMenu,
                iMenuItem as UINT,
                MF_BYPOSITION | MF_STRING,
                (IDM_FILE1 + iItem as UINT) as UINT_PTR,
                szText.as_ptr(),
            );
    
            iItem += 1;
        }
    }
}
