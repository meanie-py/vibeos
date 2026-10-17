use crate::prelude::*;
use crate::mmc_new::misc_cpp_h::DocumentModeToString;
use crate::mmc_new::misc_cpp_h::StringToDocumentMode;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_010B227F6E6B: [WCHAR; 10] = [0x0053, 0x0063, 0x006F, 0x0070, 0x0065, 0x0054, 0x0072, 0x0065, 0x0065, 0x0000];
pub static CPP_COM_WSTR_087E14528D4B: [WCHAR; 14] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0053, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0042, 0x0061, 0x0072, 0x0000];
pub static CPP_COM_WSTR_1E4595249275: [WCHAR; 12] = [0x0053, 0x006E, 0x0061, 0x0070, 0x0069, 0x006E, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x0000];
pub static CPP_COM_WSTR_2070268C40FC: [WCHAR; 14] = [0x007B, 0x0052, 0x0061, 0x006E, 0x0064, 0x006F, 0x006D, 0x0020, 0x0047, 0x0055, 0x0049, 0x0044, 0x007D, 0x0000];
pub static CPP_COM_WSTR_21FA398D73ED: [WCHAR; 7] = [0x0053, 0x006E, 0x0061, 0x0070, 0x0069, 0x006E, 0x0000];
pub static CPP_COM_WSTR_23EB6AA79FD7: [WCHAR; 7] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0025, 0x0075, 0x0000];
pub static CPP_COM_WSTR_31B6D31F4B51: [WCHAR; 4] = [0x0032, 0x002E, 0x0030, 0x0000];
pub static CPP_COM_WSTR_3A86FDDAE8D1: [WCHAR; 15] = [0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_3E45CCDFE961: [WCHAR; 9] = [0x0050, 0x0072, 0x006F, 0x0076, 0x0069, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_415A7A584334: [WCHAR; 4] = [0x0078, 0x006D, 0x006C, 0x0000];
pub static CPP_COM_WSTR_42F9B7B7BA4A: [WCHAR; 12] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x004D, 0x006F, 0x0064, 0x0065, 0x0000];
pub static CPP_COM_WSTR_45BE18936901: [WCHAR; 31] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x004D, 0x004D, 0x0043, 0x005C, 0x0053, 0x006E, 0x0061, 0x0070, 0x0049, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_50421CBFDACD: [WCHAR; 14] = [0x0076, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x003D, 0x0027, 0x0031, 0x002E, 0x0030, 0x0027, 0x0000];
pub static CPP_COM_WSTR_5BDD08CA353B: [WCHAR; 14] = [0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_6EDB8C81466D: [WCHAR; 16] = [0x004C, 0x006F, 0x0067, 0x0069, 0x0063, 0x0061, 0x006C, 0x0052, 0x0065, 0x0061, 0x0064, 0x004F, 0x006E, 0x006C, 0x0079, 0x0000];
pub static CPP_COM_WSTR_72A9B27FE1F3: [WCHAR; 21] = [0x0041, 0x006C, 0x006C, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_733FB8CD60A4: [WCHAR; 6] = [0x004E, 0x006F, 0x0064, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_76E480513E2C: [WCHAR; 5] = [0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_77065B4E456D: [WCHAR; 10] = [0x004D, 0x0044, 0x0049, 0x0043, 0x004C, 0x0049, 0x0045, 0x004E, 0x0054, 0x0000];
pub static CPP_COM_WSTR_7AB474D18DCE: [WCHAR; 25] = [0x0050, 0x0072, 0x0065, 0x0076, 0x0065, 0x006E, 0x0074, 0x0056, 0x0069, 0x0065, 0x0077, 0x0043, 0x0075, 0x0073, 0x0074, 0x006F, 0x006D, 0x0069, 0x007A, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_831513BFA930: [WCHAR; 16] = [0x004D, 0x004D, 0x0043, 0x005F, 0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_8E637B93C326: [WCHAR; 5] = [0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_95F7C62C5CD8: [WCHAR; 5] = [0x002E, 0x006D, 0x0073, 0x0063, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9FB14923561E: [WCHAR; 4] = [0x006D, 0x0073, 0x0063, 0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B442578B5D74: [WCHAR; 6] = [0x0025, 0x0064, 0x0020, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_BF422010D46F: [WCHAR; 39] = [0x007B, 0x0043, 0x0039, 0x0036, 0x0034, 0x0030, 0x0031, 0x0043, 0x0043, 0x002D, 0x0030, 0x0045, 0x0031, 0x0037, 0x002D, 0x0031, 0x0031, 0x0044, 0x0033, 0x002D, 0x0038, 0x0038, 0x0035, 0x0042, 0x002D, 0x0030, 0x0030, 0x0043, 0x0030, 0x0034, 0x0046, 0x0037, 0x0032, 0x0043, 0x0037, 0x0031, 0x0037, 0x007D, 0x0000];
pub static CPP_COM_WSTR_C34F032A7EFF: [WCHAR; 11] = [0x0046, 0x0072, 0x0061, 0x006D, 0x0065, 0x0053, 0x0074, 0x0061, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_C962F7CFEA36: [WCHAR; 6] = [0x0056, 0x0069, 0x0065, 0x0077, 0x0073, 0x0000];
pub static CPP_COM_WSTR_CA7636CE488C: [WCHAR; 5] = [0x004E, 0x006F, 0x0064, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CD28B1A69262: [WCHAR; 5] = [0x0074, 0x0072, 0x0075, 0x0065, 0x0000];
pub static CPP_COM_WSTR_D258F5DCE725: [WCHAR; 45] = [0x004D, 0x0053, 0x0043, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x0028, 0x002A, 0x002E, 0x006D, 0x0073, 0x0063, 0x0029, 0x0000, 0x002A, 0x002E, 0x006D, 0x0073, 0x0063, 0x0000, 0x0041, 0x006C, 0x006C, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x0028, 0x002A, 0x002E, 0x002A, 0x0029, 0x0000, 0x002A, 0x002E, 0x002A, 0x0000, 0x0000];
pub static CPP_COM_WSTR_D2D27D69FC0A: [WCHAR; 2] = [0x0023, 0x0000];
pub static CPP_COM_WSTR_E99A648017F2: [WCHAR; 45] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x004D, 0x004D, 0x0043, 0x005F, 0x004E, 0x0045, 0x0057, 0x005C, 0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0020, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// BTN_* and IDM_TB_* values are not in the provided context; they must come from the header defining them.

// ---- file-scope preamble (translated) ----
pub static TbButtons: [TBBUTTON; 10] = [
    TBBUTTON { iBitmap: (BTN_UNDO as i32), idCommand: (IDM_TB_UNDO as i32), fsState: 0, fsStyle: 0x0000 /* BTNS_BUTTON */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_REDO as i32), idCommand: (IDM_TB_REDO as i32), fsState: 0, fsStyle: 0x0000 /* BTNS_BUTTON */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: 4, idCommand: IDC_STATIC, fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0001 /* BTNS_SEP */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_UP as i32), idCommand: (IDM_TB_UP as i32), fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0000 /* BTNS_BUTTON */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_SCOPE_PANE as i32), idCommand: (IDM_TB_SCOPE_PANE as i32), fsState: 0x04 | 0x01 /* TBSTATE_ENABLED | TBSTATE_CHECKED */, fsStyle: 0x0002 /* BTNS_CHECK */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: 4, idCommand: IDC_STATIC, fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0001 /* BTNS_SEP */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_EXPORT_LIST as i32), idCommand: (IDM_TB_EXPORT_LIST as i32), fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0000 /* BTNS_BUTTON */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: 4, idCommand: IDC_STATIC, fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0001 /* BTNS_SEP */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_HELP as i32), idCommand: (IDM_TB_HELP as i32), fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0000 /* BTNS_BUTTON */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
    TBBUTTON { iBitmap: (BTN_ACTIONS_PANE as i32), idCommand: (IDM_TB_ACTIONS_PANE as i32), fsState: 0x04 /* TBSTATE_ENABLED */, fsStyle: 0x0002 /* BTNS_CHECK */, bReserved: unsafe { core::mem::zeroed() }, dwData: 0, iString: 0 },
];

const DocumentMode_Author: DOCUMENT_MODE = 0;


















































































impl CMainWnd {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_ConsoleNumber = 0;
        (*this_).m_DocumentMode = DocumentMode_Author;
        (*this_).m_LogicalReadOnly = (FALSE as i32);
        (*this_).m_PreventViewCustomization = (FALSE as i32);
        (*this_).m_ToolBar.__ctor_0();
        (*this_).m_bToolBarVisible = 1;
        (*this_).m_NextViewId = 1;
        (*this_).m_RootNode = core::ptr::null_mut();
        (*this_).m_NextNodeId = 1;
        (*this_).m_MDIClient.__ctor_0(core::ptr::null_mut());
        (*this_).m_TempNodeMap = core::ptr::null_mut();
        {
            (*this_).m_FrameThunk.Init(Some(CMainWnd::XDefFrameProc), this_ as *mut c_void);
            (*this_).m_pfnSuperWindowProc = (*this_).m_FrameThunk.GetWNDPROC();
            CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);
            (*this_).m_hMenuConsoleSmall = LoadMenu(
                _AtlBaseModule.GetModuleInstance(),
                MAKEINTRESOURCE!(IDM_CONSOLE_SMALL),
            );
            (*this_).m_hMenuConsoleLarge = LoadMenu(
                _AtlBaseModule.GetModuleInstance(),
                MAKEINTRESOURCE!(IDM_CONSOLE_LARGE),
            );
            (*this_).m_hSnapinImageList = ImageList_Create(
                GetSystemMetrics((SM_CXSMICON as i32)),
                GetSystemMetrics((SM_CYSMICON as i32)),
                ILC_MASK | ILC_COLOR32,
                0,
                4,
            );

            (*this_).m_hToolBarImageList = ImageList_Create(
                GetSystemMetrics((SM_CXSMICON as i32)),
                GetSystemMetrics((SM_CYSMICON as i32)),
                ILC_MASK | ILC_COLOR32,
                0,
                4,
            );

            let hToolBarBitmap: HBITMAP = LoadBitmapW(
                _AtlBaseModule.GetModuleInstance(),
                MAKEINTRESOURCE!(IDB_TOOLBAR),
            );
            ImageList_AddMasked(
                (*this_).m_hToolBarImageList,
                hToolBarBitmap,
                RGB!(0, 0, 0),
            );
            DeleteObject((hToolBarBitmap).cast());

            (*this_).LoadSnapinCache();
            (*this_).LoadRecentFiles();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DestroyMenu((*this_).m_hMenuConsoleSmall);
        DestroyMenu((*this_).m_hMenuConsoleLarge);
        ImageList_Destroy((*this_).m_hSnapinImageList);
        ImageList_Destroy((*this_).m_hToolBarImageList);
        CoUninitialize();
    }
    pub unsafe fn OnCreate(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ccs: CLIENTCREATESTRUCT = core::mem::zeroed();
        let lpFileName: LPCTSTR = (*(lParam as LPCREATESTRUCT)).lpCreateParams as LPCTSTR;
        let mut rect: RECT = core::mem::zeroed();

        (*this_).m_bStandardMenusVisible = (TRUE as i32);

        /* Create and initialize the Toolbar */
        (*this_).m_bToolBarVisible = (TRUE as i32);
        (*this_).m_ToolBar.Create((*this_).__base0.m_hWnd,
                         WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | TBSTYLE_FLAT | TBSTYLE_TOOLTIPS,
                         0);

        (*this_).m_ToolBar.SendMessageW__0(TB_SETBITMAPSIZE, 0, MAKELONG!(16, 16) as LPARAM);
        (*this_).m_ToolBar.SendMessageW__0(TB_BUTTONSTRUCTSIZE, core::mem::size_of::<TBBUTTON>() as WPARAM, 0);

        (*this_).m_ToolBar.SetImageList((*this_).m_hToolBarImageList);
        (*this_).m_ToolBar.SendMessageW__0(TB_ADDBUTTONSW, _countof!(TbButtons) as WPARAM, TbButtons.as_ptr() as LPARAM);

        (*this_).m_ToolBar.GetClientRect(core::ptr::addr_of_mut!(rect));
        (*this_).m_iToolBarHeight = rect.bottom - rect.top;

        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rect));
        rect.top += (*this_).m_iToolBarHeight;

        /* Create the MDI client window */
        ccs.hWindowMenu = (GetSubMenu((*this_).m_hMenuConsoleLarge, 4)).cast();
        ccs.idFirstChild = IDM_MDI_FIRSTCHILD;

        (*this_).m_MDIClient.Create(CPP_COM_WSTR_77065B4E456D.as_ptr(), (*this_).__base0.m_hWnd, core::ptr::addr_of_mut!(rect), core::ptr::null_mut() as LPCTSTR, WS_CHILD | WS_CLIPCHILDREN | WS_VSCROLL | WS_HSCROLL, WS_EX_CLIENTEDGE, 0, core::ptr::addr_of_mut!(ccs) as LPVOID);
        (*this_).m_MDIClient.ShowWindow((SW_SHOW as i32));
        (*this_).m_MDIClient.UpdateWindow();

        if lpFileName == core::ptr::null_mut() {
            (*this_).__base0.PostMessageW(WM_COMMAND, (IDM_FILE_NEW as _), 0 as LPARAM);
        }
        return 0;
    }
    pub unsafe fn OnSize(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateLayout();
        return 0;
    }
    pub unsafe fn OnCloseChild(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateLayout();
        (*this_).UpdateTitle();
        (*this_).UpdateMenu();
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        PostQuitMessage(0);
        (*this_).__base0.SetMenu(core::ptr::null_mut());
        return 0;
    }
    pub unsafe fn OnClose(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.DestroyWindow();
        return 0;
    }
    pub unsafe fn OnNotify(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let phdr: *mut NMHDR = lParam as *mut NMHDR;
        match (*phdr).code {
            TBN_GETINFOTIP => {
                {
                    let lptbgit: LPNMTBGETINFOTIP = lParam as LPNMTBGETINFOTIP;

                    let toolTipText = CStringW::from_ptr(MAKEINTRESOURCE!((*lptbgit).iItem));
                    wcscpy((*lptbgit).pszText, toolTipText.GetString());
                }
            }
            _ => {}
        }

        return 0;
    }
    pub unsafe fn OnMenuSelect(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        let menuItem: UINT = LOWORD!(wParam) as UINT;
        // let flags = HIWORD(wParam);
        // let hMenu = lParam as HMENU;
        let mut menuText: CStringW = CStringW::from_resource(menuItem);
        (*child).SetStatusBarText(menuText.GetString() as LPWSTR);
        return 0;
    }
    pub unsafe fn OnFileNew(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let rootName = CStringW::from_resource(IDS_CONSOLEROOT);

        /* Close all views */
        let mut pos: POSITION = (*this_).m_ViewList.GetHeadPosition();
        while !pos.is_null() {
            let console: *mut CConsoleWnd = *(*this_).m_ViewList.GetNext(&mut pos);
            (*console).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
        }

        (*this_).InitMain();

        /* Create a new snapin root node */
        let CacheEntry: *mut CSnapinCacheEntry = (*this_).GetSnapinCacheEntryByGuid((CPP_COM_WSTR_BF422010D46F.as_ptr() as PWSTR));
        if CacheEntry.is_null() {
            DPRINT1!("No folder cache entry!\n");
            return 0;
        }

        // m_RootNode = new CSnapin(CacheEntry, m_NextNodeId++, rootName.GetString());
        let NodeId: UINT = (*this_).m_NextNodeId;
        (*this_).m_NextNodeId += 1;
        (*this_).m_RootNode = CSnapin__new_0(CacheEntry, NodeId, rootName.GetString() as PWSTR);
        if CacheEntry.is_null() {
            DPRINT1!("No root folder!\n");
            return 0;
        }

        if (*this_).CreateView((*this_).m_RootNode, core::ptr::null_mut()) != 0 {
            (*this_).m_ConsoleNumber += 1;
        }

        return 1;
    }
    pub unsafe fn OnFileOpen(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut openas: OPENFILENAME = core::mem::zeroed();
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        ZeroMemory!(core::ptr::addr_of_mut!(openas) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAME>());
        wcscpy(szPath.as_mut_ptr(), CPP_COM_WSTR_96A296D224F2.as_ptr());

        openas.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
        openas.hwndOwner = (*this_).__base0.m_hWnd;
        openas.hInstance = _AtlBaseModule.GetModuleInstance();
        openas.lpstrFilter = CPP_COM_WSTR_D258F5DCE725.as_ptr();
        openas.lpstrFile = szPath.as_mut_ptr();
        openas.nMaxFile = MAX_PATH;
        openas.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;
        openas.lpstrDefExt = CPP_COM_WSTR_9FB14923561E.as_ptr();

        if GetOpenFileNameW(core::ptr::addr_of_mut!(openas)) != 0 {
            /* Close all views */
            let mut pos: POSITION = (*this_).m_ViewList.GetHeadPosition();
            while !pos.is_null() {
                let console: *mut CConsoleWnd = *(*this_).m_ViewList.GetNext(&mut pos);
                (*console).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
            }

            (*this_).InitMain();

            (*this_).m_Filename = CStringW::from_ptr(szPath.as_ptr());
            let ret: LRESULT = (*this_).LoadMscFile(&(*this_).m_Filename);
            if ret == ERROR_SUCCESS as LRESULT {
                (*this_).AddToRecentFiles(&(*this_).m_Filename);
                (*this_).m_ConsoleNumber += 1;
            }
        }

        return 0;
    }
    pub unsafe fn OnFileSave(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        if (*this_).m_Filename.IsEmpty() {
            return (*this_).OnFileSaveAs(wNotifyCode, wID, hWndCtl, bHandled);
        }

        return (*this_).SaveMscFile(&*core::ptr::addr_of!((*this_).m_Filename));
    }
    pub unsafe fn OnFileSaveAs(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut saveas: OPENFILENAME = core::mem::zeroed();
        let mut szPath: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];

        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        ZeroMemory!(core::ptr::addr_of_mut!(saveas) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAME>());
        if !(*this_).m_Filename.IsEmpty() {
            wcscpy(szPath.as_mut_ptr(), (*this_).m_Filename.GetString());
        } else {
            (*child).__base0.GetWindowTextW__0(szPath.as_mut_ptr(), (MAX_PATH as i32));
            (*this_).CreateNewFilename(szPath.as_mut_ptr(), MAX_PATH, (*this_).m_ConsoleNumber as DWORD);
            wcscat(szPath.as_mut_ptr(), CPP_COM_WSTR_95F7C62C5CD8.as_ptr());
        }

        saveas.lStructSize = (core::mem::size_of::<OPENFILENAME>()) as _;
        saveas.hwndOwner = (*this_).__base0.m_hWnd;
        saveas.hInstance = _AtlBaseModule.GetModuleInstance();
        saveas.lpstrFilter = CPP_COM_WSTR_D258F5DCE725.as_ptr();
        saveas.lpstrFile = szPath.as_mut_ptr();
        saveas.nMaxFile = MAX_PATH;
        saveas.Flags = OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_OVERWRITEPROMPT;
        saveas.lpstrDefExt = CPP_COM_WSTR_9FB14923561E.as_ptr();

        if GetSaveFileNameW(core::ptr::addr_of_mut!(saveas)) != 0 {
            (*this_).m_Filename = CAtlString::from_ptr(szPath.as_ptr());
            let ret: LRESULT = (*this_).SaveMscFile(&(*this_).m_Filename);
            if ret == (ERROR_SUCCESS as _) {
                (*this_).AddToRecentFiles(&(*this_).m_Filename);
            }
            return ret;
        }

        return 0;
    }
    pub unsafe fn OnFileAdd(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child == core::ptr::null_mut() {
            return 0;
        }
        let mut dlg: CAddDialog = CAddDialog__make_0(this_, child);
        if dlg.DoModal((*this_).__base0.m_hWnd, child as LPARAM) == (IDOK as _) {
            (*this_).UpdateViews();
        }

        return 0;
    }
    pub unsafe fn OnFileOptions(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }
        let mut dlg: COptionsDialog = COptionsDialog__make_0(this_, child);
        dlg.DoModal((*this_).__base0.m_hWnd, child as LPARAM);
        return 0;
    }
    pub unsafe fn OnFileRecent(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut mi: MENUITEMINFOW = core::mem::zeroed();

        mi.cbSize = (core::mem::size_of::<MENUITEMINFOW>()) as _;
        mi.fMask = MIIM_DATA;
        if GetMenuItemInfoW((*this_).__base0.GetMenu(), wID as UINT, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi)) != 0 {
            let FileEntry: *mut CRecentFileEntry = mi.dwItemData as *mut CRecentFileEntry;

            /* Close all views */
            let mut pos: POSITION = (*this_).m_ViewList.GetHeadPosition();
            while !pos.is_null() {
                let console: *mut CConsoleWnd = *(*this_).m_ViewList.GetNext(&mut pos);
                (*console).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
            }

            (*this_).InitMain();

            let ret: LRESULT = (*this_).LoadMscFile((*FileEntry).FileName());
            if ret == ERROR_SUCCESS as LRESULT {
                (*this_).AddToRecentFiles((*FileEntry).FileName());
                (*this_).m_ConsoleNumber += 1;
            }
        }

        return 0;
    }
    pub unsafe fn OnFileExit(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.PostMessageW(WM_CLOSE, 0, 0);
        return 0;
    }
    pub unsafe fn OnActionNewWindow(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).OnActionNewWindow(wNotifyCode, wID, hWndCtl, bHandled);

        return 0;
    }
    pub unsafe fn OnActionRename(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).OnActionRename(wNotifyCode, wID, hWndCtl, bHandled);

        return 0;
    }
    pub unsafe fn OnViewMode(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).SetListViewMode(CmdIdToViewMode((wID) as UINT));
        CheckMenuRadioItem((*this_).__base0.GetMenu(),
                           IDM_VIEW_LARGE_ICONS,
                           IDM_VIEW_DETAILS,
                           (wID) as UINT,
                           MF_BYCOMMAND);

        return 0;
    }
    pub unsafe fn OnViewCustomize(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        let mut dlg2: CCustomizeDialog = CCustomizeDialog__make_0(this_, child);
        dlg2.DoModal((*this_).__base0.m_hWnd, child as LPARAM);
        return 0;
    }
    pub unsafe fn OnWindowsNew(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).CreateView((*this_).m_RootNode, core::ptr::null_mut());
        return 0;
    }
    pub unsafe fn OnWindowsCascade(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_MDIClient.SendMessageW__0(WM_MDICASCADE, 0, 0);
        return 0;
    }
    pub unsafe fn OnWindowsTile(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_MDIClient.SendMessageW__0(WM_MDITILE, (MDITILE_HORIZONTAL) as WPARAM, 0);
        return 0;
    }
    pub unsafe fn OnWindowsArrange(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_MDIClient.SendMessageW__0(WM_MDIICONARRANGE, 0, 0);
        return 0;
    }
    pub unsafe fn OnHelpAboutMMC(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hIcon: HICON = LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP));
        let AboutTitle = CStringW::from_resource(IDS_ABOUTTITLE);
        let AppTitle = CStringW::from_resource(IDS_APPTITLE);
        let mut TitleString = CStringW::new();

        TitleString = AboutTitle;
        TitleString += CPP_COM_WSTR_D2D27D69FC0A.as_ptr();
        TitleString += AppTitle.GetString();

        ShellAboutW((*this_).__base0.m_hWnd, TitleString.GetString() as LPWSTR, core::ptr::null_mut(), hIcon);

        return 0;
    }
    pub unsafe fn OnHelpAboutSnapin(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut mi: MENUITEMINFOW = core::mem::zeroed();

        mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = MIIM_DATA;
        if GetMenuItemInfoW((*this_).__base0.GetMenu(), wID as UINT, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi)) != 0 {
            let Snapin: *mut CSnapin = mi.dwItemData as *mut CSnapin;

            let mut dlg: CAboutSnapinDialog = CAboutSnapinDialog__make_0(this_, Snapin);
            dlg.DoModal((*this_).__base0.m_hWnd, 0 as LPARAM);
        }

        return 0;
    }
    pub unsafe fn OnToolbarSelectParent(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).SelectParent();
        return 0;
    }
    pub unsafe fn OnToolbarScopePane(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).SetTreeViewVisible(((*child).IsTreeViewVisible() == 0) as BOOL);
        return 0;
    }
    pub unsafe fn OnToolbarActionsPane(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let child: *mut CConsoleWnd = (*this_).GetActiveChildInfo();
        if child.is_null() {
            return 0;
        }

        (*child).SetActionsPaneVisible((((*child).IsActionsPaneVisible()) == 0) as BOOL);
        return 0;
    }
    pub unsafe fn OnMDIForward(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hChild: HWND = (*this_).m_MDIClient.SendMessageW__0(WM_MDIGETACTIVE, 0, 0) as HWND;
        if !hChild.is_null() {
            let wParam: WPARAM = MAKEWPARAM!(wNotifyCode, wID);
            let lParam: LPARAM = hWndCtl as LPARAM;
            SendMessage(hChild, WM_COMMAND, wParam, lParam);
        }
        return 0;
    }
    pub unsafe fn CreateView(&mut self, RootNode: *mut CSnapin, SelectedNode: *mut CSnapin) -> BOOL {
        let this_: *mut Self = self;
        let mut mcs: MDICREATESTRUCT = core::mem::zeroed();
        let mut hChild: HWND;

        mcs.szTitle = (*RootNode).DisplayName().GetString();
        mcs.szClass = CConsoleWnd::GetWndClassName();
        mcs.hOwner = _AtlBaseModule.GetModuleInstance();
        mcs.cx = CW_USEDEFAULT;
        mcs.x = mcs.cx;
        mcs.cy = CW_USEDEFAULT;
        mcs.y = mcs.cy;
        mcs.style = MDIS_ALLCHILDSTYLES;
        let mut bMaximized: BOOL = (FALSE as i32);
        let hWndOld: HWND = (*this_).m_MDIClient.SendMessageW__0(WM_MDIGETACTIVE, 0, core::ptr::addr_of_mut!(bMaximized) as LPARAM) as HWND;
        mcs.lParam = ((bMaximized != 0) || (hWndOld as usize == 0)) as LPARAM;
        /* This object registers itself in the _AtlWinModule to be assigned to the next window created */
        let child: *mut CConsoleWnd = CConsoleWnd__new_0(this_, RootNode, SelectedNode);
        /* Ask for a new MDI Child window */
        hChild = (*this_).m_MDIClient.SendMessageW__0(WM_MDICREATE, 0, core::ptr::addr_of_mut!(mcs) as LONG_PTR) as HWND;
        if hChild as usize == 0
        {
            CConsoleWnd__delete(child);
            return (FALSE) as BOOL;
        }

        (*this_).UpdateLayout();
        (*this_).UpdateTitle();
        (*this_).UpdateMenu();
        (*this_).UpdateViewMenu((*child).GetListViewMode());

        return (TRUE) as BOOL;
    }
    pub unsafe fn LoadSnapinCache(&mut self) -> LRESULT {
        let this_: *mut Self = self;
        let mut SnapinsKey: CRegKey = CRegKey__make_0();
        if ERROR_SUCCESS == (SnapinsKey.Open(HKEY_LOCAL_MACHINE, CPP_COM_WSTR_45BE18936901.as_ptr(), KEY_READ)) as u32 {
            let mut namebuf = [0u16; (MAX_PATH) as _];
            let mut index: DWORD = 0;
            let mut namelen: DWORD = _countof!(namebuf);
            while ERROR_SUCCESS == (SnapinsKey.EnumKey(
                {
                    let _old = index;
                    index = index.wrapping_add(1);
                    _old
                },
                namebuf.as_mut_ptr(),
                core::ptr::addr_of_mut!(namelen),
                core::ptr::null_mut(),
            )) as u32 {
                DPRINT1!("%S\n", namebuf);
                let snapin: *mut CSnapinCacheEntry = CSnapinCacheEntry::Create(
                    &mut SnapinsKey,
                    namebuf.as_mut_ptr(),
                    (*this_).m_hSnapinImageList,
                );
                DPRINT1!("CacheEntry %p\n", snapin);
                if !snapin.is_null() {
                    (*this_).m_SnapinCache.Add(snapin);
                }
                namelen = _countof!(namebuf);
            }
            SnapinsKey.Close();
        }
        return (S_OK) as LRESULT;
    }
    pub unsafe fn UpdateRecentFilesMenu(&mut self) {
        let this_: *mut Self = self;
        let mut mi: MENUITEMINFOW = core::mem::zeroed();
        let hMenu: HMENU = (*this_).__base0.GetMenu();

        mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = MIIM_ID | MIIM_STRING | MIIM_FTYPE | MIIM_DATA;
        mi.fType = MFT_STRING;
        mi.wID = IDM_FILE_RECENT1;

        RemoveMenu(hMenu, IDM_FILE_RECENT_SEP, MF_BYCOMMAND);

        let mut i: INT = 0;
        let mut ValueName = CStringW::new();
        let mut pos = (*this_).m_RecentFilesList.GetHeadPosition();
        while pos != core::ptr::null_mut() {
            RemoveMenu(hMenu, IDM_FILE_RECENT1 + (i) as u32, MF_BYCOMMAND);

            let FileEntry: *mut CRecentFileEntry = *(*this_).m_RecentFilesList.GetNext(&mut pos);
            atl_format!(
                ValueName,
                CPP_COM_WSTR_B442578B5D74.as_ptr(),
                i + 1,
                (*FileEntry).DisplayName().GetString()
            );
            mi.dwTypeData = ValueName.GetString() as LPWSTR;
            mi.dwItemData = FileEntry as ULONG_PTR;

            InsertMenuItemW(hMenu, IDM_FILE_EXIT, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi));
            mi.wID += 1;

            i += 1;
            if i >= (MAX_RECENT_FILES as i32) {
                break;
            }
        }

        mi.fType = MFT_SEPARATOR;
        mi.fMask = MIIM_FTYPE | MIIM_ID;
        InsertMenuItemW(hMenu, IDM_FILE_EXIT, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi));
    }
    pub unsafe fn LoadRecentFiles(&mut self) {
        let this_: *mut Self = self;
        let mut RecentFilesKey = CRegKey__make_0();
        if ERROR_SUCCESS == (RecentFilesKey.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_E99A648017F2.as_ptr(), KEY_READ)) as u32 {
            let mut pathBuf = [0u16; (MAX_PATH) as _];
            let mut pathLength: ULONG = 0;
            let mut i: ULONG = 0;
            let mut err: DWORD = 0;

            let mut valueName = CStringW::new();
            let mut fileName = CStringW::new();

            i = 0;
            while i < MAX_RECENT_FILES {
                atl_format!(valueName, CPP_COM_WSTR_23EB6AA79FD7.as_ptr(), i + 1);
                pathLength = _countof!(pathBuf);
                err = (RecentFilesKey.QueryStringValue(valueName.GetString(),
                                                      pathBuf.as_mut_ptr(),
                                                      core::ptr::addr_of_mut!(pathLength))) as u32;
                if err == ERROR_SUCCESS {
                    let fileName = CAtlString::from_ptr(pathBuf.as_ptr());
                    (*this_).m_RecentFilesList.AddTail(CRecentFileEntry__new_0(&fileName));
                }
                i += 1;
            }

            RecentFilesKey.Close();
        }

        (*this_).UpdateRecentFilesMenu();
    }
    pub unsafe fn AddToRecentFiles(&mut self, FileName: &CAtlString) {
        let this_: *mut Self = self;
        let mut pos: POSITION = (*this_).m_RecentFilesList.GetHeadPosition();
        while !pos.is_null() {
            let ValueData: *mut CRecentFileEntry = *(*this_).m_RecentFilesList.GetAt(pos);
            if *(*ValueData).FileName() == *FileName {
                break;
            }

            (*this_).m_RecentFilesList.GetNext(&mut pos);
        }

        if pos.is_null() {
            /* Insert at top */
            (*this_).m_RecentFilesList.AddHead(CRecentFileEntry__new_0(FileName));
            if (*this_).m_RecentFilesList.GetCount() > 4 {
                (*this_).m_RecentFilesList.RemoveTail();
            }
        } else {
            /* Move to top */
            /* m_RecentFilesList.MoveToHead(pos); */
            let ValueData: *mut CRecentFileEntry = *(*this_).m_RecentFilesList.GetAt(pos);
            (*this_).m_RecentFilesList.RemoveAt(pos);
            (*this_).m_RecentFilesList.AddHead(ValueData);
        }

        /* Update the registry key */
        let mut RecentFilesKey = CRegKey__make_0();
        if ERROR_SUCCESS == (RecentFilesKey.Create(
            HKEY_CURRENT_USER,
            CPP_COM_WSTR_E99A648017F2.as_ptr(),
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32 {
            let mut count: INT = 0;
            let mut ValueName = CAtlString::new();
            pos = (*this_).m_RecentFilesList.GetHeadPosition();
            while !pos.is_null() {
                let ValueData: *mut CRecentFileEntry = *(*this_).m_RecentFilesList.GetNext(&mut pos);
                atl_format!(ValueName, CPP_COM_WSTR_23EB6AA79FD7.as_ptr(), count + 1);
                RecentFilesKey.SetStringValue(
                    ValueName.GetString(),
                    (*ValueData).FileName().GetString(),
                    REG_SZ,
                );
                count += 1;
                if count >= (MAX_RECENT_FILES as i32) {
                    break;
                }
            }

            RecentFilesKey.Close();
        }

        (*this_).UpdateRecentFilesMenu();
    }
    pub unsafe fn CreateNewFilename(&mut self, pBuffer: PWSTR, dwSize: DWORD, Number: DWORD) -> DWORD {
        let this_: *mut Self = self;
        let mut args: [DWORD_PTR; 1] = [0 as DWORD_PTR; 1];
        args[0] = Number as DWORD_PTR;
        let mut str = CStringW::new();

        str.LoadStringW(IDS_CONSOLETITLE);

        return FormatMessageW(FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ARGUMENT_ARRAY,
                                str.GetString() as *const core::ffi::c_void, 0, 0, pBuffer, dwSize, args.as_mut_ptr() as *mut va_list);
    }
    pub unsafe fn SaveMscFile(&mut self, FileName: &CAtlString) -> LRESULT {
        let this_: *mut Self = self;
        let mut pRootNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pFrameStateNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pViewsNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pScopeTreeNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pSnapinCacheNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pSnapinNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pNodesNode: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pos: POSITION;
        let mut console: *mut CConsoleWnd;
        let mut hr: HRESULT = S_OK;

        let mscFile: *mut MscFile = MscFile__new_0(FileName.GetString());

        'CleanUp: {
            hr = (*mscFile).CreateAndInitDOM();
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }

            /* <?xml version="1.0"?> */
            hr = (*mscFile).CreateAndAddPINode(CPP_COM_WSTR_415A7A584334.as_ptr(), CPP_COM_WSTR_50421CBFDACD.as_ptr());
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }

            /* <MMC_ConsoleFile ConsoleVersion="2.0" ProgramMode="Author"> */
            hr = (*mscFile).CreateElement(CPP_COM_WSTR_831513BFA930.as_ptr(), &mut pRootNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_3A86FDDAE8D1.as_ptr(), CPP_COM_WSTR_31B6D31F4B51.as_ptr(), pRootNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_42F9B7B7BA4A.as_ptr(), DocumentModeToString((*this_).m_DocumentMode), pRootNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }

            /* <ConsoleFileID> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_5BDD08CA353B.as_ptr(), (pRootNode).cast(), &mut pNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddTextNode(CPP_COM_WSTR_2070268C40FC.as_ptr(), (pNode).cast());
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            SAFE_RELEASE!(pNode); /* </ConsoleFileID> */

            /* <FrameState> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_C34F032A7EFF.as_ptr(), (pRootNode).cast(), &mut pFrameStateNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_087E14528D4B.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pFrameStateNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            if ((*this_).m_DocumentMode != DocumentMode_Author) && ((*this_).m_LogicalReadOnly != 0) {
                hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_6EDB8C81466D.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pFrameStateNode);
                if hr < 0 { // FAILED(hr)
                    break 'CleanUp;
                }
            }
            if ((*this_).m_DocumentMode != DocumentMode_Author) && ((*this_).m_PreventViewCustomization != 0) {
                hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_7AB474D18DCE.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pFrameStateNode);
                if hr < 0 { // FAILED(hr)
                    break 'CleanUp;
                }
            }

            (*mscFile).SaveWindowPlacement(core::ptr::addr_of_mut!((*this_).__base0), pFrameStateNode);

            SAFE_RELEASE!(pFrameStateNode); /* </FrameState> */

            /* <Views> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_C962F7CFEA36.as_ptr(), (pRootNode).cast(), &mut pViewsNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            pos = (*this_).m_ViewList.GetHeadPosition();
            loop {
                console = *(*this_).m_ViewList.GetNext(&mut pos);
                if !console.is_null() {
                    (*console).SaveView(mscFile, pViewsNode);
                }

                if !(pos != core::ptr::null_mut()) {
                    break;
                }
            }
            SAFE_RELEASE!(pViewsNode); /* </Views> */

            /* <ScopeTree> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_010B227F6E6B.as_ptr(), (pRootNode).cast(), &mut pScopeTreeNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }

            /* <SnapinCache> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_1E4595249275.as_ptr(), (pScopeTreeNode).cast(), &mut pSnapinCacheNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }

            let mut i: i32 = 0;
            while i < (*this_).GetSnapinCacheCount() {
                let entry: *mut CSnapinCacheEntry = (*this_).GetSnapinCacheEntry(i);
                if !entry.is_null() {
                    /* <Snapin> */
                    hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_21FA398D73ED.as_ptr(), (pSnapinCacheNode).cast(), &mut pSnapinNode);
                    if hr < 0 { // FAILED(hr)
                        break 'CleanUp;
                    }
                    hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_AFF3436D7481.as_ptr(), (*entry).GuidString().GetString(), pSnapinNode);
                    if hr < 0 { // FAILED(hr)
                        break 'CleanUp;
                    }
                    hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_72A9B27FE1F3.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pSnapinNode); /* FIXME */
                    if hr < 0 { // FAILED(hr)
                        break 'CleanUp;
                    }
                    hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_76E480513E2C.as_ptr(), (*entry).Name().GetString(), pSnapinNode);
                    if hr < 0 { // FAILED(hr)
                        break 'CleanUp;
                    }
                    hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_3E45CCDFE961.as_ptr(), (*entry).Provider().GetString(), pSnapinNode);
                    if hr < 0 { // FAILED(hr)
                        break 'CleanUp;
                    }
                    SAFE_RELEASE!(pSnapinNode); /* </Snapin> */
                }
                i += 1;
            }
            SAFE_RELEASE!(pSnapinCacheNode); /* </SnapinCache> */

            /* <Nodes> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_733FB8CD60A4.as_ptr(), (pScopeTreeNode).cast(), &mut pNodesNode);
            if hr < 0 { // FAILED(hr)
                break 'CleanUp;
            }
            if !(*this_).m_RootNode.is_null() {
                (*(*this_).m_RootNode).SaveNode(mscFile, pNodesNode);
            }
            SAFE_RELEASE!(pNodesNode); /* </Nodes> */

            SAFE_RELEASE!(pScopeTreeNode); /* </ScopeTree> */

            hr = (*mscFile).AppendChildToParent(pRootNode as *mut IXMLDOMNode, core::ptr::null_mut());

            if hr < 0 { // FAILED(hr)

                break 'CleanUp;

            }

            hr = (*mscFile).SaveDOM();

            if hr < 0 { // FAILED(hr)

                break 'CleanUp;

            }
        }

        // CleanUp:
        SAFE_RELEASE!(pRootNode); /* </MMC_ConsoleFile> */

        MscFile__delete(mscFile);

        return 0;
    }
    pub unsafe fn LoadMscFile(&mut self, FileName: &CAtlString) -> LRESULT {
        let this_: *mut Self = self;
        let mut pRootElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pFrameStateElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut ProgramMode: VARIANT = core::mem::zeroed();
        let mut hr: HRESULT = S_OK;

        (*this_).m_TempNodeMap = cpp_new::<CSimpleMap<UINT, *mut CSnapin>>(CSimpleMap::new());

        VariantInit(core::ptr::addr_of_mut!(ProgramMode));

        (*this_).m_DocumentMode = DocumentMode_Author;
        (*this_).m_LogicalReadOnly = (FALSE as i32);
        (*this_).m_PreventViewCustomization = (FALSE as i32);

        let mscFile: *mut MscFile = MscFile__new_0(FileName.GetString());

        'done: {
            hr = (*mscFile).CreateAndInitDOM();
            if hr < 0 {
                break 'done;
            }

            hr = (*mscFile).LoadDOM();
            if hr < 0 {
                break 'done;
            }

            hr = (*mscFile).CheckMscFile(core::ptr::addr_of_mut!(pRootElement));
            if hr != S_OK {
                break 'done;
            }

            hr = (*mscFile).GetAttribute(
                pRootElement,
                CPP_COM_WSTR_42F9B7B7BA4A.as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(ProgramMode),
            );
            if hr != S_OK {
                break 'done;
            }

            hr = StringToDocumentMode(V_BSTR!(core::ptr::addr_of_mut!(ProgramMode)), core::ptr::addr_of_mut!((*this_).m_DocumentMode));
            if hr != S_OK {
                break 'done;
            }

            hr = (*mscFile).GetElement(
                pRootElement,
                CPP_COM_WSTR_C34F032A7EFF.as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(pFrameStateElement),
            );
            if hr != S_OK {
                break 'done;
            }

            if (*this_).m_DocumentMode != DocumentMode_Author {
                let mut LogicalReadOnly: VARIANT = core::mem::zeroed();
                let mut PreventViewCustomization: VARIANT = core::mem::zeroed();

                VariantInit(core::ptr::addr_of_mut!(LogicalReadOnly));
                VariantInit(core::ptr::addr_of_mut!(PreventViewCustomization));

                hr = (*mscFile).GetAttribute(
                    pFrameStateElement,
                    CPP_COM_WSTR_6EDB8C81466D.as_ptr() as LPWSTR,
                    core::ptr::addr_of_mut!(LogicalReadOnly),
                );
                if hr == S_OK
                    && _wcsicmp(V_BSTR!(core::ptr::addr_of_mut!(LogicalReadOnly)), CPP_COM_WSTR_CD28B1A69262.as_ptr()) == 0
                {
                    (*this_).m_LogicalReadOnly = (TRUE as i32);
                }

                hr = (*mscFile).GetAttribute(
                    pFrameStateElement,
                    CPP_COM_WSTR_7AB474D18DCE.as_ptr() as LPWSTR,
                    core::ptr::addr_of_mut!(PreventViewCustomization),
                );
                if hr == S_OK
                    && _wcsicmp(V_BSTR!(core::ptr::addr_of_mut!(PreventViewCustomization)), CPP_COM_WSTR_CD28B1A69262.as_ptr()) == 0
                {
                    (*this_).m_PreventViewCustomization = (TRUE as i32);
                }

                VariantClear(core::ptr::addr_of_mut!(LogicalReadOnly));
                VariantClear(core::ptr::addr_of_mut!(PreventViewCustomization));
                hr = S_OK;
            }

            hr = (*mscFile).ParseWindowPlacement(core::ptr::addr_of_mut!((*this_).__base0), pFrameStateElement);
            if hr != S_OK {
                break 'done;
            }

            /* FIXME Parse other frame settings */

            /* Parse the ScopeTree element */
            hr = (*this_).ParseScopeTree(mscFile, pRootElement);
            if hr != S_OK {
                break 'done;
            }

            /* Parse the Views element */
            hr = (*this_).ParseViewsElement(mscFile, pRootElement);
            if hr != S_OK {
                break 'done;
            }
        }

        // done:
        VariantClear(core::ptr::addr_of_mut!(ProgramMode));

        if !pFrameStateElement.is_null() {
            (*pFrameStateElement).Release();
        }

        if !pRootElement.is_null() {
            (*pRootElement).Release();
        }

        cpp_delete((*this_).m_TempNodeMap);
        (*this_).m_TempNodeMap = core::ptr::null_mut();

        MscFile__delete(mscFile);

        if hr == S_FALSE {
            // MessageBox(NULL, L"Invalid file!", FileName.GetString(), MB_OK);
        } else if hr != S_OK {
            // MessageBox(NULL, L"Failure!", FileName.GetString(), MB_OK);
        }

        return (ERROR_SUCCESS) as LRESULT;
    }
    pub unsafe fn ParseScopeTree(&mut self, mscFile: *mut MscFile, pRootElement: *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pScopeTreeElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pNodesElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pNodeElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut hr: HRESULT = S_OK;

        'done: {
            hr = (*mscFile).GetElement(pRootElement, (CPP_COM_WSTR_010B227F6E6B.as_ptr() as LPWSTR), core::ptr::addr_of_mut!(pScopeTreeElement));
            if hr != S_OK {
                break 'done;
            }

            /* Fixme: SnapinCache Element */

            /* Nodes Element */
            hr = (*mscFile).GetElement(pScopeTreeElement, (CPP_COM_WSTR_733FB8CD60A4.as_ptr() as LPWSTR), core::ptr::addr_of_mut!(pNodesElement));
            if hr != S_OK {
                break 'done;
            }

            hr = (*mscFile).GetElement(pNodesElement, (CPP_COM_WSTR_CA7636CE488C.as_ptr() as LPWSTR), core::ptr::addr_of_mut!(pNodeElement));
            if hr != S_OK {
                break 'done;
            }

            hr = CSnapin::ParseSnapin(this_, mscFile, pNodeElement, core::ptr::addr_of_mut!((*this_).m_RootNode));
        }

        if !pNodeElement.is_null() {
            (*pNodeElement).Release();
        }

        if !pNodesElement.is_null() {
            (*pNodesElement).Release();
        }

        if !pScopeTreeElement.is_null() {
            (*pScopeTreeElement).Release();
        }

        hr
    }
    pub unsafe fn ParseViewsElement(&mut self, mscFile: *mut MscFile, pRootElement: *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pViewsElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pChildList: *mut IXMLDOMNodeList = core::ptr::null_mut();
        let mut nextItem: *mut IXMLDOMNode = core::ptr::null_mut();
        let mut NodeType: DOMNodeType = core::mem::zeroed();
        let mut NodeName: BSTR = core::ptr::null_mut();
        let mut hr: HRESULT = S_OK;

        'done: {
            hr = (*mscFile).GetElement(pRootElement, (CPP_COM_WSTR_C962F7CFEA36.as_ptr() as LPWSTR), core::ptr::addr_of_mut!(pViewsElement));
            if hr != S_OK {
                break 'done;
            }

            hr = (*pViewsElement).get_childNodes(core::ptr::addr_of_mut!(pChildList));
            if ((hr) < 0) {
                break 'done;
            }

            loop {
                hr = (*pChildList).nextNode(core::ptr::addr_of_mut!(nextItem));
                if hr == S_FALSE {
                    hr = S_OK;
                    break;
                }

                hr = (*nextItem).get_nodeType(core::ptr::addr_of_mut!(NodeType));
                if hr == S_OK {
                    if NodeType == NODE_ELEMENT {
                        NodeName = core::ptr::null_mut();
                        hr = (*nextItem).get_baseName(core::ptr::addr_of_mut!(NodeName));
                        if hr == S_OK {
                            if _wcsicmp(NodeName, CPP_COM_WSTR_8E637B93C326.as_ptr()) == 0 {
                                CConsoleWnd::ParseView(this_, mscFile, nextItem as *mut IXMLDOMElement);
                            }
                            SysFreeString(NodeName);
                            NodeName = core::ptr::null_mut();
                        }
                    }
                }

                (*nextItem).Release();
            }
        }

        // done:
        if !pChildList.is_null() {
            (*pChildList).Release();
        }

        if !NodeName.is_null() {
            SysFreeString(NodeName);
        }

        if !pViewsElement.is_null() {
            (*pViewsElement).Release();
        }

        hr
    }
    pub unsafe fn GetConsoleTitleW(&mut self) -> *mut CAtlString {
        let this_: *mut Self = self;
        return core::ptr::addr_of_mut!((*this_).m_ConsoleTitle);
    }
    pub unsafe fn SetConsoleTitleW(&mut self, consoleTitle: CAtlString) {
        let this_: *mut Self = self;
        (*this_).m_ConsoleTitle = consoleTitle.clone();
        (*this_).__base0.SetWindowTextW(consoleTitle.GetString());
    }
    pub unsafe fn IsToolBarVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bToolBarVisible
    }
    pub unsafe fn SetToolBarVisible(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bToolBarVisible = bVisible;
        (*this_).UpdateLayout();
    }
    pub unsafe fn AreStandardMenusVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bStandardMenusVisible
    }
    pub unsafe fn SetStandardMenusVisible(&mut self, bVisible: BOOL,
    ) {
        let this_: *mut Self = self;
        (*this_).m_bStandardMenusVisible = bVisible;
    }
    pub unsafe fn UpdateAboutSnapinMenu(&mut self, Snapin: *mut CSnapin) {
        let this_: *mut Self = self;
        let hMenu: HMENU = (*this_).__base0.GetMenu();

        RemoveMenu(hMenu, IDM_HELP_ABOUT_SNAPIN, MF_BYCOMMAND);

        if !Snapin.is_null() {
            let mut mi: MENUITEMINFOW = core::mem::zeroed();
            let mut ValueName: CStringW = CStringW::new();
            let mut AboutString: CStringW = CStringW::from_resource(MAKEINTRESOURCE!(IDS_ABOUT_SNAPIN) as UINT);

            atl_format!(ValueName, AboutString.GetString(), (*Snapin).Name().GetString());

            mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
            mi.fMask = MIIM_ID | MIIM_STRING | MIIM_FTYPE | MIIM_DATA;
            mi.fType = MFT_STRING;
            mi.wID = IDM_HELP_ABOUT_SNAPIN;

            mi.dwTypeData = ValueName.GetString() as LPWSTR;
            mi.dwItemData = (Snapin as usize) as _;

            /* FIXME: Append this menu item */
            InsertMenuItemW(hMenu, IDM_HELP_ABOUT_MMC, (FALSE) as BOOL, &mut mi);
        }
    }
    pub unsafe fn UpdateViewMenu(&mut self, ListViewMode: LISTVIEW_MODE) {
        let this_: *mut Self = self;
        CheckMenuRadioItem((*this_).__base0.GetMenu(),
                           IDM_VIEW_LARGE_ICONS,
                           IDM_VIEW_DETAILS,
                           ViewModeToCmdId(ListViewMode),
                           MF_BYCOMMAND);
    }
    pub unsafe fn UpdateUpToolButton(&mut self, bShow: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_ToolBar.HideButton((IDM_TB_UP as i32), (bShow == 0) as BOOL);
    }
    pub unsafe fn UpdateToolbuttons(&mut self, bTreeViewVisible: BOOL, bActionsPaneVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_ToolBar.CheckButton((IDM_TB_SCOPE_PANE as i32), bTreeViewVisible);
        (*this_).m_ToolBar.CheckButton((IDM_TB_ACTIONS_PANE as i32), bActionsPaneVisible);
    }
    pub unsafe fn UpdateLayout(&mut self) {
        let this_: *mut Self = self;
        let mut rcClient: RECT = core::mem::zeroed();
        let mut rcToolBar: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient));

        let mut nToolBarFlags: i32 = (SWP_NOZORDER as i32);
        if (*this_).m_bToolBarVisible != 0 && (*this_).m_ViewList.IsEmpty() == false {
            rcClient.top += (*this_).m_iToolBarHeight;
            rcClient.bottom -= (*this_).m_iToolBarHeight;

            rcToolBar.right = rcClient.right;
            rcToolBar.bottom = (*this_).m_iToolBarHeight;
            nToolBarFlags |= (SWP_SHOWWINDOW as i32);
        } else {
            nToolBarFlags |= (SWP_HIDEWINDOW as i32);
        }

        (*this_).m_ToolBar.__base0.SetWindowPos(core::ptr::null_mut(), rcToolBar.left, rcToolBar.top, rcToolBar.right, rcToolBar.bottom, nToolBarFlags as UINT);

        (*this_).m_MDIClient.SetWindowPos(core::ptr::null_mut(), rcClient.left, rcClient.top, rcClient.right, rcClient.bottom, (SWP_NOZORDER) as UINT);
    }
    pub unsafe fn UpdateViews(&mut self) {
        let this_: *mut Self = self;
        let mut console: *mut CConsoleWnd;
        let mut pos: POSITION;

        pos = (*this_).m_ViewList.GetHeadPosition();
        while !pos.is_null() {
            console = *(*this_).m_ViewList.GetNext(&mut pos);
            if !console.is_null() {
                (*console).UpdateView();
            }
        }
    }
    pub unsafe fn UpdateMenu(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_ViewList.IsEmpty() != false {
            (*this_).__base0.SetMenu((*this_).m_hMenuConsoleSmall);
        } else {
            (*this_).__base0.SetMenu((*this_).m_hMenuConsoleLarge);
            (*this_).UpdateRecentFilesMenu();
        }
    }
    pub unsafe fn UpdateTitle(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_ViewList.IsEmpty() != false {
            (*this_).m_ConsoleTitle.LoadStringW(IDS_APPTITLE);
        } else {
            let ConsoleTitle: *mut CAtlString = core::ptr::addr_of_mut!((*this_).m_ConsoleTitle);
            (*this_).CreateNewConsoleTitle(&mut *ConsoleTitle);
        }
        (*this_).__base0.SetWindowTextW((*this_).m_ConsoleTitle.GetString());
    }
    pub unsafe fn GetSnapinCacheCount(&mut self) -> i32 {
        let this_: *mut Self = self;
        return (*this_).m_SnapinCache.GetSize();
    }
    pub unsafe fn GetSnapinCacheEntry(&mut self, nIndex: i32) -> *mut CSnapinCacheEntry {
        let this_: *mut Self = self;
        return *(*this_).m_SnapinCache.GetAt(nIndex);
    }
    pub unsafe fn GetSnapinCacheEntryByGuid(&mut self, pszGuid: PWSTR) -> *mut CSnapinCacheEntry {
        let this_: *mut Self = self;
        let mut i: i32 = 0;
        while i < (*this_).m_SnapinCache.GetSize() {
            let entry: *mut CSnapinCacheEntry = *(*this_).m_SnapinCache.GetAt(i);
            if (*entry).GuidString().CompareNoCase(pszGuid) == 0 {
                return entry;
            }
            i += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn SnapinImageList(&mut self) -> HIMAGELIST {
        let this_: *mut Self = self;
        (*this_).m_hSnapinImageList
    }
    pub unsafe fn GetRootSnapin(&mut self) -> *mut CSnapin {
        let this_: *mut Self = self;
        (*this_).m_RootNode
    }
    pub unsafe fn RegisterView(&mut self, pView: *mut CConsoleWnd) {
        let this_: *mut Self = self;
        (*this_).m_ViewList.AddTail(pView);
        if (*pView).m_ViewId == 0 {
            (*pView).m_ViewId = (*this_).m_NextViewId;
            (*this_).m_NextViewId = (*this_).m_NextViewId.wrapping_add(1);
        } else if (*this_).m_NextViewId <= (*pView).m_ViewId {
            (*this_).m_NextViewId = (*pView).m_ViewId + 1;
        }
    }
    pub unsafe fn UnregisterView(&mut self, pView: *mut CConsoleWnd) {
        let this_: *mut Self = self;
        let pos: POSITION = (*this_).m_ViewList.FindPos(&pView);
        if !pos.is_null() {
            (*this_).m_ViewList.RemoveAt(pos);
            (*pView).m_ViewId = 0;
        }
    }
    pub unsafe fn GetNextNodeId(&mut self) -> UINT {
        let this_: *mut Self = self;
        let _ret = (*this_).m_NextNodeId;
        (*this_).m_NextNodeId = (*this_).m_NextNodeId.wrapping_add(1);
        _ret
    }
    pub unsafe fn UpdateNextNodeId(&mut self, NodeId: UINT) {
        let this_: *mut Self = self;
        if NodeId >= (*this_).m_NextNodeId {
            (*this_).m_NextNodeId = NodeId + 1;
        }
    }
    pub unsafe fn GetDocumentMode(&mut self) -> DOCUMENT_MODE {
        let this_: *mut Self = self;
        (*this_).m_DocumentMode
    }
    pub unsafe fn SetDocumentMode(&mut self, DocumentMode: DOCUMENT_MODE,
    ) {
        let this_: *mut Self = self;
        (*this_).m_DocumentMode = DocumentMode;
    }
    pub unsafe fn GetLogicalReadOnly(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_LogicalReadOnly
    }
    pub unsafe fn SetLogicalReadOnly(&mut self, LogicalReadOnly: BOOL,
    ) {
        let this_: *mut Self = self;
        (*this_).m_LogicalReadOnly = LogicalReadOnly;
    }
    pub unsafe fn GetPreventViewCustomization(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_PreventViewCustomization
    }
    pub unsafe fn SetPreventViewCustomization(&mut self, PreventCustomization: BOOL,
    ) {
        let this_: *mut Self = self;
        (*this_).m_PreventViewCustomization = PreventCustomization;
    }
}
