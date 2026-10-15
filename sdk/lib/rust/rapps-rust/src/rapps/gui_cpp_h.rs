use crate::prelude::*;
use crate::rapps::loaddlg_cpp_h::DownloadApplication;
use crate::rapps::appinfo_h_h::IsAvailableEnum;
use crate::rapps::appinfo_h_h::IsInstalledEnum;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_E22B677384CD: [WCHAR; 25] = [0x0048, 0x0065, 0x006C, 0x0070, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut m_hLastFocus: HWND = core::ptr::null_mut();
pub static mut m_PendingInstalledViewRefresh: bool = false;
































#[no_mangle]
pub unsafe extern "C" fn MainWindowLoop(wnd: *mut CMainWindow, nShowCmd: INT) {
    let mut KeyBrd: HACCEL;
    let mut Msg: MSG = core::mem::zeroed();

    hMainWnd = (*wnd).Create();
    if hMainWnd.is_null() {
        return;
    }

    /* Maximize it if we must */
    (*wnd).__base0.ShowWindow(
        if SettingsInfo.bSaveWndPos != 0 && SettingsInfo.Maximized != 0 {
            (SW_MAXIMIZE as i32)
        } else {
            nShowCmd
        },
    );
    (*wnd).__base0.UpdateWindow();

    /* Load the menu hotkeys */
    KeyBrd = LoadAcceleratorsW(core::ptr::null_mut(), MAKEINTRESOURCEW!(HOTKEYS));

    /* Message Loop */
    while GetMessageW(
        core::ptr::addr_of_mut!(Msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        if TranslateAcceleratorW(hMainWnd, KeyBrd, core::ptr::addr_of_mut!(Msg)) == 0 {
            if Msg.message == WM_CHAR && Msg.wParam == (VK_TAB as _) {
                // Move backwards if shift is held down
                let direction: i32 = if (GetKeyState((VK_SHIFT as i32)) as i32 & 0x8000) != 0 {
                    -1
                } else {
                    1
                };

                (*wnd).HandleTabOrder(direction);
                continue;
            }

            TranslateMessage(core::ptr::addr_of_mut!(Msg));
            DispatchMessageW(core::ptr::addr_of_mut!(Msg));
        }
    }
}

impl CSideTreeView {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).hImageTreeView = ImageList_Create((TREEVIEW_ICON_SIZE as i32), (TREEVIEW_ICON_SIZE as i32), (GetSystemColorDepth() | (ILC_MASK as i32)) as UINT, 0, 1);
        {
        }
    }
    pub unsafe fn AddItem(&mut self, hParent: HTREEITEM, Text: &mut CStringW, Image: INT, SelectedImage: INT, lParam: LPARAM) -> HTREEITEM {
        let this_: *mut Self = self;
        return (*this_).__base0.__base2.AddItem(hParent, Text.GetString() as LPWSTR, Image, SelectedImage, lParam);
    }
    pub unsafe fn AddCategory(&mut self, hRootItem: HTREEITEM, TextIndex: UINT, IconIndex: UINT) -> HTREEITEM {
        let this_: *mut Self = self;
        let mut szText: CStringW = CStringW::new();
        let mut Index: INT = 0;
        let mut hIcon: HICON;

        hIcon = LoadImageW(
            hInst, MAKEINTRESOURCE!(IconIndex), IMAGE_ICON, (TREEVIEW_ICON_SIZE as i32), (TREEVIEW_ICON_SIZE as i32), LR_CREATEDIBSECTION) as HICON;
        if !hIcon.is_null() {
            Index = ImageList_AddIcon!((*this_).hImageTreeView, hIcon);
            DestroyIcon(hIcon);
        }

        szText.LoadStringW(TextIndex);
        return (*this_).AddItem(hRootItem, &mut szText, Index, Index, TextIndex as LPARAM);
    }
    pub unsafe fn SetImageList(&mut self) -> HIMAGELIST {
        let this_: *mut Self = self;
        return (*this_).__base0.__base2.SetImageList((*this_).hImageTreeView, (TVSIL_NORMAL as i32));
    }
    pub unsafe fn DestroyImageList(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).hImageTreeView.is_null() {
            ImageList_Destroy((*this_).hImageTreeView);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).DestroyImageList();
    }
}

impl CMainWindow {
    pub unsafe fn __ctor_0(&mut self, db: *mut CAppDB, bAppwiz: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_ClientPanel = core::ptr::null_mut();
        (*this_).m_VSplitter = core::ptr::null_mut();
        (*this_).m_TreeView = core::ptr::null_mut();
        (*this_).m_StatusBar = core::ptr::null_mut();
        (*this_).m_ApplicationView = core::ptr::null_mut();
        (*this_).m_Db = db;
        (*this_).bUpdating = (FALSE as i32);
        (*this_).m_bAppwizMode = bAppwiz;
        (*this_).SelectedEnumType = ENUM_ALL_INSTALLED;
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).LayoutCleanup();
    }
    pub unsafe fn InitCategoriesList(&mut self) {
        let this_: *mut Self = self;
        let mut hRootItemAvailable: HTREEITEM = core::mem::zeroed();

        (*this_).hRootItemInstalled = (*(*this_).m_TreeView).AddCategory(TVI_ROOT, IDS_INSTALLED, IDI_CATEGORY);
        (*(*this_).m_TreeView).AddCategory((*this_).hRootItemInstalled, IDS_APPLICATIONS, IDI_APPS);
        (*(*this_).m_TreeView).AddCategory((*this_).hRootItemInstalled, IDS_UPDATES, IDI_APPUPD);

        // Do not show any other categories in APPWIZ-mode.
        'Finish: {
            if (*this_).m_bAppwizMode != 0 {
                break 'Finish;
            }

            (*(*this_).m_TreeView).AddCategory(TVI_ROOT, IDS_SELECTEDFORINST, IDI_SELECTEDFORINST);

            hRootItemAvailable = (*(*this_).m_TreeView).AddCategory(TVI_ROOT, IDS_AVAILABLEFORINST, IDI_CATEGORY);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_AUDIO, IDI_CAT_AUDIO);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_VIDEO, IDI_CAT_VIDEO);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_GRAPHICS, IDI_CAT_GRAPHICS);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_GAMES, IDI_CAT_GAMES);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_INTERNET, IDI_CAT_INTERNET);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_OFFICE, IDI_CAT_OFFICE);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_DEVEL, IDI_CAT_DEVEL);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_EDU, IDI_CAT_EDU);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_ENGINEER, IDI_CAT_ENGINEER);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_FINANCE, IDI_CAT_FINANCE);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_SCIENCE, IDI_CAT_SCIENCE);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_TOOLS, IDI_CAT_TOOLS);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_DRIVERS, IDI_CAT_DRIVERS);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_LIBS, IDI_CAT_LIBS);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_THEMES, IDI_CAT_THEMES);
            (*(*this_).m_TreeView).AddCategory(hRootItemAvailable, IDS_CAT_OTHER, IDI_CAT_OTHER);
        }

        // Finish:
        (*(*this_).m_TreeView).SetImageList();
        (*(*this_).m_TreeView).__base0.__base2.Expand((*this_).hRootItemInstalled, TVE_EXPAND);
        if (*this_).m_bAppwizMode == 0 {
            (*(*this_).m_TreeView).__base0.__base2.Expand(hRootItemAvailable, TVE_EXPAND);
        }
        (*(*this_).m_TreeView).__base0.__base2.SelectItem(
            if (*this_).m_bAppwizMode != 0 { (*this_).hRootItemInstalled } else { hRootItemAvailable },
            TVGN_CARET,
        );
    }
    pub unsafe fn CreateStatusBar(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_StatusBar = CUiWindow_CStatusBar__new_0();
        (*(*this_).m_StatusBar).__base1.m_VerticalAlignment = UiAlign_RightBtm;
        (*(*this_).m_StatusBar).__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*(*this_).m_ClientPanel).__base2.Children()).__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_StatusBar).__base0));

        return ((*(*this_).m_StatusBar).__base2.Create((*this_).__base0.m_hWnd, (IDC_STATUSBAR as usize) as HMENU) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn CreateTreeView(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_TreeView = CSideTreeView__new_0();
        (*(*this_).m_TreeView).__base0.__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_TreeView).__base0.__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_VSplitter).First().__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_TreeView).__base0.__base0));

        return ((*(*this_).m_TreeView).__base0.__base2.Create((*this_).__base0.m_hWnd, 0) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn CreateApplicationView(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_ApplicationView = CApplicationView__new_0(this_); // pass this to ApplicationView for callback purpose
        (*(*this_).m_ApplicationView).__base0.__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_ApplicationView).__base0.__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_VSplitter).Second().__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_ApplicationView).__base0.__base0));

        return ((*(*this_).m_ApplicationView).Create((*this_).__base0.m_hWnd) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn CreateVSplitter(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_VSplitter = CUiSplitPanel__new_0();
        (*(*this_).m_VSplitter).__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_VSplitter).__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_VSplitter).m_DynamicFirst = (FALSE as i32);
        (*(*this_).m_VSplitter).m_Horizontal = (FALSE as i32);
        (*(*this_).m_VSplitter).m_MinFirst = 0;

        // TODO: m_MinSecond should be calculate dynamically instead of hard-coded
        (*(*this_).m_VSplitter).m_MinSecond = 480;
        (*(*this_).m_VSplitter).m_Pos = 240;
        (*(*(*this_).m_ClientPanel).__base2.Children()).__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_VSplitter).__base0));

        ((*(*this_).m_VSplitter).Create((*this_).__base0.m_hWnd) != core::ptr::null_mut()) as BOOL
    }
    pub unsafe fn CreateLayout(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let mut b: BOOL = (TRUE as i32);
        (*this_).bUpdating = (TRUE as i32);

        (*this_).m_ClientPanel = CUiPanel__new_0();
        (*(*this_).m_ClientPanel).__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_ClientPanel).__base1.m_HorizontalAlignment = UiAlign_Stretch;

        // Top level
        b = (b != 0 && (*this_).CreateStatusBar() != 0) as BOOL;
        b = (b != 0 && (*this_).CreateVSplitter() != 0) as BOOL;

        // Inside V Splitter
        b = (b != 0 && (*this_).CreateTreeView() != 0) as BOOL;
        b = (b != 0 && (*this_).CreateApplicationView() != 0) as BOOL;

        if b != 0 {
            let mut rBottom: RECT = core::mem::zeroed();

            /* Size status bar */
            (*(*this_).m_StatusBar).__base2.__base0.SendMessageW__0(WM_SIZE, 0, 0);

            GetWindowRect((*(*this_).m_StatusBar).__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rBottom));

            (*(*this_).m_VSplitter).__base1.m_Margin.__base0.__base0.bottom = rBottom.bottom - rBottom.top;
        }

        (*this_).bUpdating = (FALSE as i32);
        return b;
    }
    pub unsafe fn LayoutCleanup(&mut self) {
        let this_: *mut Self = self;
        CSideTreeView__delete((*this_).m_TreeView);
        CApplicationView__delete((*this_).m_ApplicationView);
        CUiSplitPanel__delete((*this_).m_VSplitter);
        CUiWindow_CStatusBar__delete((*this_).m_StatusBar);
        return;
    }
    pub unsafe fn InitControls(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).CreateLayout() != 0 {
            (*this_).InitCategoriesList();
            (*this_).UpdateStatusBarText();

            return (TRUE) as BOOL;
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn OnSize(&mut self, hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if wParam == (SIZE_MINIMIZED as _) {
            return;
        }

        /* Size status bar */
        (*(*this_).m_StatusBar).__base2.__base0.SendMessageW__0(WM_SIZE, 0, 0);

        let mut r: RECT = RECT {
            left: 0,
            top: 0,
            right: LOWORD!(lParam) as i32,
            bottom: HIWORD!(lParam) as i32,
        };
        let mut hdwp: HDWP = core::ptr::null_mut();
        let count: i32 = ((*(*this_).m_ClientPanel).CountSizableChildren()) as i32;

        hdwp = BeginDeferWindowPos(count);
        if !hdwp.is_null() {
            hdwp = (*(*this_).m_ClientPanel).OnParentSize(r, hdwp);
            if !hdwp.is_null() {
                EndDeferWindowPos(hdwp);
            }
        }
    }
    pub unsafe fn CheckAvailable(&mut self) {
        let this_: *mut Self = self;
        if (*(*this_).m_Db).GetAvailableCount() == 0 {
            let mut lock: CUpdateDatabaseMutex = CUpdateDatabaseMutex__make_0();
            (*(*this_).m_Db).RemoveCached();
            (*(*this_).m_Db).UpdateAvailable();
        }
    }
    pub unsafe fn RemoveSelectedAppFromRegistry(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if !IsInstalledEnum(((*this_).SelectedEnumType) as i32) != 0 {
            return (FALSE) as BOOL;
        }

        let mut szMsgText = CStringW::new();
        let mut szMsgTitle = CStringW::new();

        if szMsgText.LoadStringW(IDS_APP_REG_REMOVE) == 0 || szMsgTitle.LoadStringW(IDS_INFORMATION) == 0 {
            return (FALSE) as BOOL;
        }

        let InstalledApp: *mut CAppInfo = (*(*this_).m_ApplicationView).GetFocusedItemData() as *mut CAppInfo;
        if InstalledApp.is_null() {
            return (FALSE) as BOOL;
        }

        if (*this_).__base0.MessageBoxW(
            szMsgText.GetString(),
            szMsgTitle.GetString(),
            MB_YESNO | MB_ICONQUESTION,
        ) == (IDYES as i32)
        {
            return (CAppDB::RemoveInstalledAppFromRegistry(InstalledApp) == ERROR_SUCCESS) as BOOL;
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn UninstallSelectedApp(&mut self, bModify: BOOL) -> BOOL {
        let this_: *mut Self = self;
        if IsInstalledEnum(((*this_).SelectedEnumType) as i32) == 0 {
            return (FALSE) as BOOL;
        }

        let InstalledApp: *mut CAppInfo = (*(*this_).m_ApplicationView).GetFocusedItemData() as *mut CAppInfo;
        if InstalledApp.is_null() {
            return (FALSE) as BOOL;
        }

        return (*InstalledApp).UninstallApplication(if bModify != 0 { UCF_MODIFY } else { UCF_NONE });
    }
    pub unsafe fn ProcessWindowMessage(&mut self, hwnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: &mut LRESULT, dwMapId: DWORD) -> BOOL {
        let this_: *mut Self = self;
        *theResult = 0;
        match Msg {
            WM_CREATE => {
                if (*this_).InitControls() == 0 {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                PostMessageW(hwnd, DM_REPOSITION, 0, 0);
            }

            WM_DESTROY => {
                hMainWnd = core::ptr::null_mut();
                SaveSettings(hwnd, core::ptr::addr_of_mut!(SettingsInfo));
                FreeLogs();

                CUiPanel__delete((*this_).m_ClientPanel);

                PostQuitMessage(0);
                return 0;
            }

            WM_CLOSE => {
                (*this_).__base0.ShowWindow((SW_HIDE as i32));
                return g_Busy;
            }

            WM_NOTIFY_OPERATIONCOMPLETED => {
                if g_Busy == 0 && (*this_).__base0.IsWindowVisible() == 0 {
                    (*this_).__base0.SendMessageW__0(WM_CLOSE, 0, 0);
                }
            }

            WM_NOTIFY_INSTALLERFINISHED => {
                m_PendingInstalledViewRefresh = true; // Something just installed, our uninstall list is probably outdated
                (*(*this_).m_ApplicationView).RefreshAvailableItem(lParam as PCWSTR);
            }

            DM_REPOSITION => {
                EmulateDialogReposition(hwnd); // We are not a real dialog, we need help from a real one
            }

            WM_ACTIVATE => {
                if LOWORD!(wParam) == (WA_INACTIVE as u16) {
                    m_hLastFocus = GetFocus();
                }
            }

            WM_SETFOCUS => {
                if !m_hLastFocus.is_null() {
                    SetFocus(m_hLastFocus);
                }
            }

            WM_NEXTDLGCTL => {
                if LOWORD!(lParam) == 0 {
                    (*this_).HandleTabOrder(if wParam != 0 { -1 } else { 1 });
                } else if wParam != 0 {
                    SetFocus(wParam as HWND);
                }
            }

            WM_COMMAND => {
                (*this_).OnCommand(wParam, lParam);
            }

            WM_NOTIFY => {
                let data: LPNMHDR = lParam as LPNMHDR;

                match (*data).code {
                    TVN_ITEMEXPANDING => {
                        if (*data).hwndFrom == ((*(*this_).m_TreeView).__base0.__base2.__base0.m_hWnd).cast() {
                            // APPWIZ-mode: forbid item collapse.
                            // FIXME: Prevent collapse (COMCTL32 is buggy)
                            // https://bugs.winehq.org/show_bug.cgi?id=53727
                            if (*this_).m_bAppwizMode != 0 && (((*(lParam as LPNMTREEVIEW)).action & TVE_TOGGLE) == TVE_COLLAPSE) {
                                *theResult = TRUE as _;
                                return (TRUE) as BOOL; // Handled
                            }
                        }
                    }

                    TVN_SELCHANGED => {
                        if (*data).hwndFrom == ((*(*this_).m_TreeView).__base0.__base2.__base0.m_hWnd).cast() {
                            match (*(lParam as LPNMTREEVIEW)).itemNew.lParam {
                                __c if __c == IDS_INSTALLED as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_ALL_INSTALLED, (FALSE) as BOOL, (FALSE) as BOOL);
                                }

                                __c if __c == IDS_APPLICATIONS as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_INSTALLED_APPLICATIONS, (FALSE) as BOOL, (FALSE) as BOOL);
                                }

                                __c if __c == IDS_UPDATES as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_UPDATES, (FALSE) as BOOL, (FALSE) as BOOL);
                                }

                                __c if __c == IDS_AVAILABLEFORINST as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_ALL_AVAILABLE, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_AUDIO as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_AUDIO, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_DEVEL as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_DEVEL, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_DRIVERS as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_DRIVERS, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_EDU as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_EDU, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_ENGINEER as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_ENGINEER, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_FINANCE as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_FINANCE, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_GAMES as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_GAMES, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_GRAPHICS as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_GRAPHICS, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_INTERNET as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_INTERNET, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_LIBS as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_LIBS, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_OFFICE as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_OFFICE, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_OTHER as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_OTHER, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_SCIENCE as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_SCIENCE, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_TOOLS as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_TOOLS, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_VIDEO as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_VIDEO, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_CAT_THEMES as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_THEMES, (FALSE) as BOOL, (TRUE) as BOOL);
                                }

                                __c if __c == IDS_SELECTEDFORINST as _ => {
                                    (*this_).UpdateApplicationsList(ENUM_CAT_SELECTED, (FALSE) as BOOL, (FALSE) as BOOL);
                                }

                                _ => {}
                            }
                        }
                    }

                    _ => {}
                }
            }

            WM_SIZE => {
                (*this_).OnSize(hwnd, wParam, lParam);
            }

            WM_SIZING => {
                let pRect: LPRECT = lParam as LPRECT;

                if (*pRect).right - (*pRect).left < 565 {
                    (*pRect).right = (*pRect).left + 565;
                }

                if (*pRect).bottom - (*pRect).top < 300 {
                    (*pRect).bottom = (*pRect).top + 300;
                }

                return (TRUE) as BOOL;
            }

            WM_SYSCOLORCHANGE => {
                /* Forward WM_SYSCOLORCHANGE to common controls */
                (*(*this_).m_ApplicationView).__base0.__base2.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
                (*(*this_).m_TreeView).__base0.__base2.__base0.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
            }

            WM_SETTINGCHANGE => {
                if wParam == (SPI_SETNONCLIENTMETRICS as _) || wParam == (SPI_SETICONMETRICS as _) {
                    DestroyIcon(g_hDefaultPackageIcon);
                    g_hDefaultPackageIcon = core::ptr::null_mut(); // Trigger imagelist recreation on next load
                }
            }

            WM_TIMER => {
                if wParam == SEARCH_TIMER_ID as WPARAM {
                    KillTimer(hwnd, SEARCH_TIMER_ID as UINT_PTR);

                    (*this_).UpdateApplicationsList((*this_).SelectedEnumType, (FALSE) as BOOL, (FALSE) as BOOL);
                }
            }

            _ => {}
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn ShowAboutDlg(&mut self) {
        let this_: *mut Self = self;
        let mut szApp = CStringW::new();
        let mut szAuthors = CStringW::new();
        szApp.LoadStringW(IDS_APPTITLE);
        szAuthors.LoadStringW(IDS_APP_AUTHORS);
        ShellAboutW(
            (*this_).__base0.m_hWnd,
            szApp.GetString(),
            szAuthors.GetString(),
            LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_MAIN)),
        );
    }
    pub unsafe fn OnCommand(&mut self, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        let bReload: BOOL = (TRUE as i32);
        let wCommand: WORD = LOWORD!(wParam);

        if lParam == 0 {
            match (wCommand) as u32 {
                ID_SETTINGS => {
                    CreateSettingsDlg((*this_).__base0.m_hWnd);
                }

                ID_EXIT => {
                    (*this_).__base0.PostMessageW(WM_CLOSE, 0, 0);
                }

                ID_SEARCH => {
                    (*(*this_).m_ApplicationView).SetFocusOnSearchBar();
                }

                ID_INSTALL => {
                    if IsAvailableEnum(((*this_).SelectedEnumType) as INT) != 0 {
                        if (*this_).m_Selected.IsEmpty() == false {
                            if DownloadListOfApplications(&mut (*this_).m_Selected, 0) != 0 {
                                (*this_).m_Selected.RemoveAll();
                                (*this_).UpdateApplicationsList((*this_).SelectedEnumType, (FALSE) as BOOL, (FALSE) as BOOL);
                            }
                        } else {
                            let App: *mut CAppInfo = (*(*this_).m_ApplicationView).GetFocusedItemData() as *mut CAppInfo;
                            if !App.is_null() {
                                (*this_).InstallApplication(App);
                            }
                        }
                    }
                }

                ID_UNINSTALL => {
                    if (*this_).UninstallSelectedApp((FALSE) as BOOL) != 0 {
                        (*this_).UpdateApplicationsList((*this_).SelectedEnumType, bReload, (FALSE) as BOOL);
                    }
                }

                ID_MODIFY => {
                    if (*this_).UninstallSelectedApp((TRUE) as BOOL) != 0 {
                        (*this_).UpdateApplicationsList((*this_).SelectedEnumType, bReload, (FALSE) as BOOL);
                    }
                }

                ID_REGREMOVE => {
                    if (*this_).RemoveSelectedAppFromRegistry() != 0 {
                        (*this_).UpdateApplicationsList((*this_).SelectedEnumType, bReload, (FALSE) as BOOL);
                    }
                }

                ID_REFRESH => {
                    (*this_).UpdateApplicationsList((*this_).SelectedEnumType, bReload, (FALSE) as BOOL);
                }

                ID_RESETDB => {
                    let mut lock: CUpdateDatabaseMutex = CUpdateDatabaseMutex__make_0();
                    (*(*this_).m_Db).RemoveCached();
                    (*this_).UpdateApplicationsList((*this_).SelectedEnumType, bReload, (FALSE) as BOOL);
                    /* Force database update even if "Available apps" view is not active */
                    if IsAvailableEnum(((*this_).SelectedEnumType) as INT) == 0 {
                        (*(*this_).m_Db).UpdateAvailable();
                    }
                    let _ = &mut lock;
                }

                ID_HELP => {
                    (*this_).__base0.MessageBoxW(CPP_COM_WSTR_E22B677384CD.as_ptr(), core::ptr::null_mut(), MB_OK);
                }

                ID_ABOUT => {
                    (*this_).ShowAboutDlg();
                }

                ID_CHECK_ALL => {
                    (*(*this_).m_ApplicationView).CheckAll();
                }

                ID_ACTIVATE_APPWIZ => {
                    if (*this_).hRootItemInstalled as usize != 0 {
                        (*(*this_).m_TreeView).__base0.__base2.SelectItem((*this_).hRootItemInstalled, TVGN_CARET);
                    }
                }

                _ => {}
            }
        }
    }
    pub unsafe fn UpdateStatusBarText(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_StatusBar.is_null() {
            let mut szBuffer = CStringW::new();
            atl_format_id!(
                szBuffer,
                IDS_APPS_COUNT,
                (*(*this_).m_ApplicationView).GetItemCount()
            );

            // Append the number of selected apps if not in APPWIZ-mode.
            if (*this_).m_bAppwizMode == 0 {
                let mut szBuffer2 = CStringW::new();
                atl_format_id!(
                    szBuffer2,
                    IDS_APPS_SELECT_COUNT,
                    (*this_).m_Selected.GetCount()
                );
                szBuffer += szBuffer2.GetString();
            }

            (*(*this_).m_StatusBar).__base2.SetText(szBuffer.GetString());
        }
    }
    pub unsafe fn UpdateApplicationsList(&mut self, EnumType: AppsCategories, mut bReload: BOOL, bCheckAvailable: BOOL) {
        let this_: *mut Self = self;
        // Only installed applications should be enumerated in APPWIZ-mode.
        if (*this_).m_bAppwizMode != 0 && ((IsInstalledEnum((EnumType) as i32)) == 0) {
            ATLASSERT!(FALSE && "Should not be called in APPWIZ-mode");
            return;
        }

        (*this_).bUpdating = (TRUE as i32);

        {
            let hCursor: HCURSOR = LoadCursor(core::ptr::null_mut(), IDC_APPSTARTING);
            if !hCursor.is_null() {
                // The database (.ini files) is parsed on the UI thread, let the user know we are busy
                SetCursor(hCursor);
                (*this_).__base0.PostMessageW(WM_SETCURSOR, (*this_).__base0.m_hWnd as WPARAM, (MAKELONG!(HTCLIENT, WM_MOUSEMOVE)) as _);
            }
        }

        if bCheckAvailable != 0 {
            (*this_).CheckAvailable();
        }

        if m_PendingInstalledViewRefresh && ((IsInstalledEnum((EnumType) as i32)) != 0) && ((IsInstalledEnum(((*this_).SelectedEnumType) as i32)) == 0) {
            m_PendingInstalledViewRefresh = ((FALSE) != 0);
            bReload = (TRUE as i32); // Reload because we are switching from Available to Installed after something installed
        }

        let TryRestoreSelection: BOOL = ((*this_).SelectedEnumType == EnumType) as BOOL;
        if (*this_).SelectedEnumType != EnumType {
            (*this_).SelectedEnumType = EnumType;
        }

        let mut RestoreSelection: RESTORELISTSELECTION = RESTORELISTSELECTION__make_0();
        if TryRestoreSelection != 0 {
            (*(*this_).m_ApplicationView).GetRestoreListSelectionData(&mut RestoreSelection);
        }

        if bReload != 0 {
            (*this_).m_Selected.RemoveAll();
        }

        (*(*this_).m_ApplicationView).SetRedraw((FALSE) as BOOL);
        if ((IsInstalledEnum((EnumType) as i32)) != 0) {
            if bReload != 0 {
                (*(*this_).m_Db).UpdateInstalled();
            }

            // set the display type of application-view. this will remove all the item in application-view too.
            (*(*this_).m_ApplicationView).SetDisplayAppType(AppViewTypeInstalledApps);

            let mut List: CAtlList<*mut CAppInfo> = CAtlList::new();
            (*(*this_).m_Db).GetApps(&mut List, EnumType);
            (*this_).AddApplicationsToView(&mut List);
        }
        else if ((IsAvailableEnum((EnumType) as INT)) != 0) {
            // We shouldn't get there in APPWIZ-mode.
            ATLASSERT!((*this_).m_bAppwizMode == 0);

            if bReload != 0 {
                (*(*this_).m_Db).UpdateAvailable();
            }

            // set the display type of application-view. this will remove all the item in application-view too.
            (*(*this_).m_ApplicationView).SetDisplayAppType(AppViewTypeAvailableApps);

            // enum available softwares
            if EnumType == ENUM_CAT_SELECTED {
                (*this_).AddApplicationsToView(&mut (*this_).m_Selected);
            }
            else {
                let mut List: CAtlList<*mut CAppInfo> = CAtlList::new();
                (*(*this_).m_Db).GetApps(&mut List, EnumType);
                (*this_).AddApplicationsToView(&mut List);
            }
        }
        else {
            ATLASSERT!(0 && "This should be unreachable!");
        }

        if TryRestoreSelection != 0 {
            (*(*this_).m_ApplicationView).RestoreListSelection(&RestoreSelection);
        }
        (*(*this_).m_ApplicationView).SetRedraw((TRUE) as BOOL);
        (*(*this_).m_ApplicationView).__base0.__base2.RedrawWindow(core::ptr::null(), core::ptr::null_mut(), RDW_INVALIDATE | RDW_ALLCHILDREN); // force the child window to repaint
        (*this_).UpdateStatusBarText();

        let mut text = CStringW::new();
        if (*(*this_).m_ApplicationView).GetItemCount() == 0 && !(*this_).szSearchPattern.IsEmpty() {
            text.LoadStringW(IDS_NO_SEARCH_RESULTS);
        }
        (*(*this_).m_ApplicationView).SetWatermark(&text);

        (*this_).bUpdating = (FALSE as i32);
    }
    pub unsafe fn AddApplicationsToView(&mut self, List: &mut CAtlList<*mut CAppInfo>) {
        let this_: *mut Self = self;
        let mut CurrentListPosition: POSITION = List.GetHeadPosition();
        while CurrentListPosition as usize != 0 {
            let Info: *mut CAppInfo = *List.GetNext(&mut CurrentListPosition);
            if (*this_).szSearchPattern.IsEmpty() || SearchPatternMatch((&(*Info).szDisplayName).GetString(), (&(*this_).szSearchPattern).GetString()) != 0 ||
                SearchPatternMatch((&(*Info).szComments).GetString(), (&(*this_).szSearchPattern).GetString()) != 0
            {
                let bSelected: BOOL = (*this_).m_Selected.Find(&Info) as BOOL;
                (*(*this_).m_ApplicationView).AddApplication(Info, bSelected);
            }
        }
        (*(*this_).m_ApplicationView).AddApplication(core::ptr::null_mut(), (FALSE) as BOOL); // Tell the list we are done
    }
    pub unsafe fn GetWndClassInfo() -> *mut CWndClassInfo {
        // A C++ function-local `static` is built once: re-running the initializer on every call would
        // reset m_atom to 0 and re-register the class.
        static mut wc: CWndClassInfo = unsafe { core::mem::zeroed() };
        static mut INITIALIZED: bool = false;
        if !INITIALIZED {
            let csStyle: DWORD = CS_VREDRAW | CS_HREDRAW;
            wc = CWndClassInfo {
                // the island spells m_wc as WNDCLASSEXA; the A/W layouts are identical
                m_wc: core::mem::transmute(WNDCLASSEXW {
                    cbSize: core::mem::size_of::<WNDCLASSEX>() as UINT,
                    style: csStyle,
                    lpfnWndProc: Some(core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, _>(StartWindowProc)),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: core::ptr::null_mut(),
                    hIcon: (LoadIconW(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCEW!(IDI_MAIN))).cast(),
                    hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                    hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
                    lpszMenuName: MAKEINTRESOURCEW!(IDR_MAINMENU),
                    lpszClassName: szWindowClass,
                    hIconSm: core::ptr::null_mut(),
                }),
                m_lpszOrigName: core::ptr::null_mut(),
                pWndProc: None,
                m_lpszCursorID: IDC_ARROW,
                m_bSystemCursor: (TRUE as i32),
                m_atom: 0,
                m_szAutoName: core::mem::zeroed(), // _T("") -- array length is 5 + 2*sizeof(void*), per arch
            };
            INITIALIZED = true;
        }
        return core::ptr::addr_of_mut!(wc);
    }
    pub unsafe fn Create(&mut self) -> HWND {
        let this_: *mut Self = self;
        let szWindowName = CStringW::from_resource(if (*this_).m_bAppwizMode != 0 { IDS_APPWIZ_TITLE } else { IDS_APPTITLE });

        let mut r = RECT {
            left: if SettingsInfo.bSaveWndPos != 0 { SettingsInfo.Left } else { CW_USEDEFAULT },
            top: if SettingsInfo.bSaveWndPos != 0 { SettingsInfo.Top } else { CW_USEDEFAULT },
            right: if SettingsInfo.bSaveWndPos != 0 { SettingsInfo.Width } else { 680 },
            bottom: if SettingsInfo.bSaveWndPos != 0 { SettingsInfo.Height } else { 450 },
        };
        r.right += r.left;
        r.bottom += r.top;

        return (*this_).CWindowImpl_Create(
            core::ptr::null_mut(),
            &mut r,
            szWindowName.GetString(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            WS_EX_WINDOWEDGE,
            0,
        );
    }
    pub unsafe fn ItemCheckStateChanged(&mut self, bChecked: BOOL, CallbackParam: LPVOID) {
        let this_: *mut Self = self;
        if (*this_).bUpdating != 0 {
            return;
        }

        let Info: *mut CAppInfo = CallbackParam as *mut CAppInfo;

        if bChecked != 0 {
            (*this_).m_Selected.AddTail(Info);
        } else {
            // CAtlList::Find(Info) returns the element's POSITION; atl_rust's Find only answers bool, so walk the list
            let mut SelPos: POSITION = core::ptr::null_mut();
            let mut Walk: POSITION = (*this_).m_Selected.GetHeadPosition();
            while !Walk.is_null() {
                let PosCur: POSITION = Walk;
                if *(*this_).m_Selected.GetNext(&mut Walk) == Info {
                    SelPos = PosCur;
                    break;
                }
            }
            ATLASSERT!(!SelPos.is_null());

            if !SelPos.is_null() {
                (*this_).m_Selected.RemoveAt(SelPos);
            }
        }

        (*this_).UpdateStatusBarText();
    }
    pub unsafe fn InstallApplication(&mut self, Info: *mut CAppInfo) -> BOOL {
        let this_: *mut Self = self;
        return (!Info.is_null() && (DownloadApplication(Info) != 0)) as BOOL;
    }
    pub unsafe fn SearchTextChanged(&mut self, SearchText: &mut CStringW) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).szSearchPattern.Compare(SearchText.GetString()) == 0 {
            return (FALSE) as BOOL;
        }

        (*this_).szSearchPattern.SetString(SearchText.GetString(), SearchText.GetLength());

        let mut dwDelay: DWORD = 0;
        SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, core::ptr::addr_of_mut!(dwDelay) as *mut DWORD as LPVOID, 0);
        (*this_).__base0.SetTimer(SEARCH_TIMER_ID as UINT_PTR, dwDelay, None);

        return (TRUE) as BOOL;
    }
    pub unsafe fn HandleTabOrder(&mut self, direction: i32) {
        let this_: *mut Self = self;
        let mut TabOrderHwndList: CSimpleArray<HWND> = CSimpleArray::new();

        (*(*this_).m_TreeView).__base0.AppendTabOrderWindow(direction, &mut TabOrderHwndList);
        (*(*this_).m_ApplicationView).AppendTabOrderWindow(direction, &mut TabOrderHwndList);

        if TabOrderHwndList.GetSize() == 0 {
            // in case the list is empty
            return;
        }

        let mut FocusIndex: i32;

        FocusIndex = TabOrderHwndList.Find(&GetFocus());
        if FocusIndex == -1 {
            FocusIndex = 0; // focus the first window in the list
        } else {
            FocusIndex += direction;
            FocusIndex +=
                TabOrderHwndList.GetSize(); // FocusIndex might be negative. we don't want to mod a negative number
            FocusIndex %= TabOrderHwndList.GetSize();
        }

        SetFocus(*TabOrderHwndList.GetAt(FocusIndex));
        return;
    }
}
