use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_25B61379E7E9: [WCHAR; 20] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0043, 0x006F, 0x006C, 0x0075, 0x006D, 0x006E, 0x0031, 0x0057, 0x0069, 0x0064, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_34D297E2847F: [WCHAR; 4] = [0x0032, 0x0030, 0x0030, 0x0000];
pub static CPP_COM_WSTR_3D44317232D6: [WCHAR; 7] = [0x0053, 0x0074, 0x0061, 0x0074, 0x0069, 0x0063, 0x0000];
pub static CPP_COM_WSTR_44C8518BCAAF: [WCHAR; 6] = [0x0066, 0x0061, 0x006C, 0x0073, 0x0065, 0x0000];
pub static CPP_COM_WSTR_4889C6392E22: [WCHAR; 5] = [0x0054, 0x0065, 0x0073, 0x0074, 0x0000];
pub static CPP_COM_WSTR_4F8320D91E97: [WCHAR; 2] = [0x0030, 0x0000];
pub static CPP_COM_WSTR_578E89C9480F: [WCHAR; 3] = [0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_60E82282DE60: [WCHAR; 9] = [0x0056, 0x0069, 0x0065, 0x0077, 0x004D, 0x006F, 0x0064, 0x0065, 0x0000];
pub static CPP_COM_WSTR_6AC123285056: [WCHAR; 12] = [0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_84602D53EA5F: [WCHAR; 12] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0042, 0x0061, 0x0072, 0x0000];
pub static CPP_COM_WSTR_87027C6B1057: [WCHAR; 15] = [0x0053, 0x0063, 0x006F, 0x0070, 0x0065, 0x0050, 0x0061, 0x006E, 0x0065, 0x0057, 0x0069, 0x0064, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_8E637B93C326: [WCHAR; 5] = [0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_A52B679307C3: [WCHAR; 22] = [0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0042, 0x0061, 0x0072, 0x0056, 0x0069, 0x0073, 0x0069, 0x0062, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_ACF7BB451AB4: [WCHAR; 12] = [0x0056, 0x0069, 0x0065, 0x0077, 0x004F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_BDCA0F318C15: [WCHAR; 17] = [0x0053, 0x0063, 0x006F, 0x0070, 0x0065, 0x0050, 0x0061, 0x006E, 0x0065, 0x0056, 0x0069, 0x0073, 0x0069, 0x0062, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_BEC2836C8636: [WCHAR; 19] = [0x0041, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0050, 0x0061, 0x006E, 0x0065, 0x0056, 0x0069, 0x0073, 0x0069, 0x0062, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CD28B1A69262: [WCHAR; 5] = [0x0074, 0x0072, 0x0075, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CD85856650BC: [WCHAR; 17] = [0x0041, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0050, 0x0061, 0x006E, 0x0065, 0x0057, 0x0069, 0x0064, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_E999B61173C7: [WCHAR; 20] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0043, 0x006F, 0x006C, 0x0075, 0x006D, 0x006E, 0x0030, 0x0057, 0x0069, 0x0064, 0x0074, 0x0068, 0x0000];

use core::ffi::c_void;

extern "system" {
    #[link_name = "DefMDIChildProcW"]
    fn DefMDIChildProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

// precomp.h: enum _LISTVIEW_MODE { SmallIcons, LargeIcons, List, Detail } -- Detail is 3 (the
// bought constant said 1, which is ListView_LargeIcons: the console started in large-icon mode).
const ListView_Detail: LISTVIEW_MODE = _LISTVIEW_MODE_ListView_Detail;































































impl CConsoleWnd {
    pub unsafe fn UpdateLayout(&mut self) {
        let this_: *mut Self = self;
        let mut Rect: RECT = core::mem::zeroed();
        let mut iTreeViewWidth: i32 = 0;
        let mut iTreeViewHeight: i32 = 0;
        let mut iListViewPosX: i32 = 0;
        let mut iListViewPosY: i32 = 0;
        let mut iListViewWidth: i32 = 0;
        let mut iListViewHeight: i32 = 0;
    
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(Rect));
        let iClientHeight: i32 = Rect.bottom - Rect.top;
        let iClientWidth: i32 = Rect.right - Rect.left;
    
        let mut iStatusHeight: i32 = 0;
    
        let mut iDescriptionBarHeight: i32 = 0;
    
        if (*this_).m_bTreeViewVisible != 0 {
            iTreeViewWidth = (*this_).m_iTreeViewWidth;
            iListViewPosX = (*this_).m_iTreeViewWidth + (*this_).m_iSplitterWidth;
            iListViewWidth = iClientWidth - (*this_).m_iTreeViewWidth - (*this_).m_iSplitterWidth;
        } else {
            iTreeViewWidth = 0;
            iListViewPosX = 0;
            iListViewWidth = iClientWidth;
        }
    
        iTreeViewHeight = iClientHeight;
        iListViewHeight = iClientHeight;
    
        if (*this_).m_bStatusBarVisible != 0 {
            iStatusHeight = (*this_).m_iStatusBarHeight;
            iTreeViewHeight -= iStatusHeight;
            iListViewHeight -= iStatusHeight;
        }
    
        if (*this_).m_bDescriptionBarVisible != 0 {
            iDescriptionBarHeight = 24;
            iListViewPosY += iDescriptionBarHeight;
            iListViewHeight -= iDescriptionBarHeight;
        }
    
        if (*this_).m_bActionsPaneVisible != 0 {
            iListViewWidth -= ((*this_).m_iActionsPaneWidth + (*this_).m_iSplitterWidth);
        }
    
        /* Move the status bar */
        (*this_).m_StatusBar.MoveWindow(0, iClientHeight - iStatusHeight, iClientWidth, iStatusHeight, (TRUE as _));
    
        /* Move the tree view */
        (*this_).m_TreeView.MoveWindow(0, 0, iTreeViewWidth, iTreeViewHeight, (TRUE as _));
    
        /* Move the list view */
        (*this_).m_ListView.MoveWindow(iListViewPosX, iListViewPosY, iListViewWidth, iListViewHeight, (TRUE as _));
    
        (*this_).m_DescriptionBar.MoveWindow(iListViewPosX, 0, iListViewWidth, iDescriptionBarHeight, (TRUE as i32));
    }
    pub unsafe fn AddTreeViewItemRecursive(&mut self, hParentTreeItem: HTREEITEM, Node: *mut CSnapin) {
        let this_: *mut Self = self;
        let mut hTreeItem: HTREEITEM;
    
        hTreeItem = (*this_).m_TreeView.AddItem(
            hParentTreeItem,
            (*Node).DisplayName().GetString() as LPWSTR,
            (*(*Node).CacheEntry()).NormalImageIndex(),
            (*(*Node).CacheEntry()).OpenImageIndex(),
            Node as LPARAM,
        );
        if !hTreeItem.is_null() {
            let mut pos: POSITION = (*Node).m_SubNodes.GetHeadPosition();
            while !pos.is_null() {
                let ChildNode: *mut CSnapin = *(*Node).m_SubNodes.GetNext(&mut pos);
                if !ChildNode.is_null() {
                    (*this_).AddTreeViewItemRecursive(hTreeItem, ChildNode);
                }
            }
    
            (*this_).m_TreeView.Expand(hTreeItem, TVE_EXPAND);
    
            if Node == (*this_).m_ViewSelectedNode {
                (*this_).m_TreeView.SelectItem(hTreeItem, TVGN_CARET);
            }
        }
    }
    pub unsafe fn UpdateTreeView(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_TreeView.DeleteItem(TVI_ROOT);
        (*this_).AddTreeViewItemRecursive(TVI_ROOT, (*this_).m_ViewRootNode);
    }
    pub unsafe fn __ctor_0(&mut self, MainWnd: *mut CMainWnd, RootNode: *mut CSnapin) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
            (*this_).m_MainWnd = MainWnd;
            (*this_).m_ViewRootNode = RootNode;
            (*this_).m_ViewSelectedNode = RootNode;
            (*this_).m_ListViewMode = ListView_Detail;
            (*this_).m_pfnSuperWindowProc = Some(DefMDIChildProc);
    
            (*this_).m_hMenuTreeView = LoadMenu(
                _AtlBaseModule.GetModuleInstance(),
                MAKEINTRESOURCE!(IDM_TREEVIEW_CONTEXT)
            );
    
            if (*this_).m_thunk.Init(None, core::ptr::null_mut()) == 0 {
                return;
            }
            _AtlWinModule.AddCreateWndData(core::ptr::addr_of_mut!((*this_).m_thunk.cd), this_);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DestroyMenu((*this_).m_hMenuTreeView);
    }
    pub unsafe fn OnCreate(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.SetWindowLongPtr(0, this_ as LONG_PTR);
    
        (*this_).m_bStatusBarVisible = (TRUE as i32);
        (*this_).m_bDescriptionBarVisible = (FALSE as i32);
        (*this_).m_bTreeViewVisible = (TRUE as i32);
        (*this_).m_bActionsPaneVisible = (FALSE as i32);
    
        (*this_).m_iSplitterWidth = 4;
        (*this_).m_iSplitSide = 0;
    
        let mut Rect: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(Rect));
        (*this_).m_iTreeViewWidth = (Rect.right - Rect.left) / 4;
        (*this_).m_iActionsPaneWidth = (Rect.right - Rect.left) / 4;
    
        (*this_).m_StatusBar.Create((*this_).__base0.m_hWnd, core::ptr::null_mut());
        (*this_).m_StatusBar.GetClientRect(core::ptr::addr_of_mut!(Rect));
        (*this_).m_iStatusBarHeight = Rect.bottom - Rect.top;
    
        Rect.right = 0;
        Rect.left = 0;
        Rect.top = 0;
        Rect.bottom = 0;
        (*this_).m_DescriptionBar.Create(
            CPP_COM_WSTR_3D44317232D6.as_ptr(),
            (*this_).__base0.m_hWnd,
            core::ptr::addr_of_mut!(Rect),
            CPP_COM_WSTR_6AC123285056.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | SS_OWNERDRAW,
            WS_EX_STATICEDGE,
            0,
            core::ptr::null_mut(),
        );
        (*this_).m_DescriptionBar.SetWindowTextW(CPP_COM_WSTR_4889C6392E22.as_ptr());
    
        (*this_).m_TreeView.Create(
            (*this_).__base0.m_hWnd,
            WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_SHOWSELALWAYS | TVS_EDITLABELS,
        );
    
        (*this_).m_TreeView.SetImageList((*(*this_).m_MainWnd).SnapinImageList(), (TVSIL_NORMAL as _));
    
        (*this_).m_ListView.Create(
            (*this_).__base0.m_hWnd,
            core::ptr::addr_of_mut!(Rect),
            core::ptr::null_mut(),
            WS_CHILD | WS_VISIBLE | ViewModeToStyle((*this_).m_ListViewMode),
            WS_EX_CLIENTEDGE,
            0,
        );
    
        (*this_).m_ViewId = (*(*this_).m_MainWnd).RegisterView(this_);
    
        (*this_).UpdateTreeView();
    
        let mdicreate: LPMDICREATESTRUCT = (*(lParam as LPCREATESTRUCT)).lpCreateParams as LPMDICREATESTRUCT;
        if (*mdicreate).lParam != 0 {
            (*this_).__base0.PostMessageW(WM_SYSCOMMAND, (SC_MAXIMIZE as _), 0);
        }
    
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*(*this_).m_MainWnd).UnregisterView(this_);
        (*(*this_).m_MainWnd).__base0.PostMessageW(WM_USER_CLOSE_CHILD, 0, 0);
        return 0;
    }
    pub unsafe fn OnSize(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateLayout();
        return DefMDIChildProc((*this_).__base0.m_hWnd, WM_SIZE, wParam, lParam);
    }
    pub unsafe fn OnContextMenu(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = core::mem::zeroed();
    
        if (wParam as HWND) == (*this_).m_TreeView._base.m_hWnd {
            let mut hti: TVHITTESTINFO = core::mem::zeroed();
            let mut nPos: i32 = 0;
    
            pt.x = LOWORD!(lParam as u32) as i16 as i32;
            pt.y = HIWORD!(lParam as u32) as i16 as i32;
    
            if pt.x == -1 && pt.y == -1 {
                let mut rc: RECT = core::mem::zeroed();
                hti.hItem = (*this_).m_TreeView.GetSelection();
                if hti.hItem != core::ptr::null_mut() {
                    TreeView_GetItemRect!((*this_).m_TreeView._base.m_hWnd, hti.hItem, core::ptr::addr_of_mut!(rc), TRUE);
                    pt.x = rc.left + 8;
                    pt.y = rc.top + 8;
                    (*this_).m_TreeView.ClientToScreen__0(core::ptr::addr_of_mut!(pt));
                    hti.flags = (TVHT_ONITEM as u32);
                } else {
                    hti.flags = 0;
                }
            } else {
                hti.pt.x = pt.x;
                hti.pt.y = pt.y;
                (*this_).m_TreeView.ScreenToClient(core::ptr::addr_of_mut!(hti.pt));
                (*this_).m_TreeView.HitTest(core::ptr::addr_of_mut!(hti));
    
                if !hti.hItem.is_null() {
                    if hti.hItem == (*this_).m_TreeView.GetSelection() {
                        nPos = 1;
                    }
                }
            }
    
            if (hti.flags & (TVHT_ONITEM as u32)) != 0 {
                let hContextMenu: HMENU = GetSubMenu((*this_).m_hMenuTreeView, nPos);
                if nPos == 1 {
                    CheckMenuRadioItem(hContextMenu,
                                       IDM_VIEW_LARGE_ICONS,
                                       IDM_VIEW_DETAILS,
                                       ViewModeToCmdId((*this_).m_ListViewMode),
                                       MF_BYCOMMAND);
                }
                TrackPopupMenu(hContextMenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, (*this_).__base0.m_hWnd, core::ptr::null_mut());
            }
        }
        return 0;
    }
    pub unsafe fn OnDrawItem(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let lpDrawItem: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;
        let mut WindowText: [u16; 256] = [0u16; 256];
    
        let hBrush: HBRUSH = GetSysColorBrush((COLOR_3DDKSHADOW as i32));
        FillRect((*lpDrawItem).hDC, core::ptr::addr_of_mut!((*lpDrawItem).rcItem), hBrush);
    
        SetTextColor((*lpDrawItem).hDC, GetSysColor((COLOR_3DHILIGHT as i32)));
    
        GetWindowTextW((*lpDrawItem).hwndItem, WindowText.as_mut_ptr(), (ARRAYSIZE!(WindowText)) as i32);
        SetBkMode((*lpDrawItem).hDC, (TRANSPARENT as i32));
    
        (*lpDrawItem).rcItem.left += 4;
        DrawTextW((*lpDrawItem).hDC, WindowText.as_ptr(), -1, core::ptr::addr_of_mut!((*lpDrawItem).rcItem),
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_MODIFYSTRING | DT_END_ELLIPSIS | DT_NOPREFIX);
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnNotify(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
    
        let phdr: *mut NMHDR = lParam as *mut NMHDR;
        match (*phdr).code {
            TVN_BEGINLABELEDIT => {
                return (FALSE) as LRESULT;
            }
    
            TVN_ENDLABELEDIT => {
                {
                    let pNM: *mut TV_DISPINFO = lParam as *mut TV_DISPINFO;
                    let pszText: LPWSTR = (*pNM).item.pszText;
                    let Snapin: *mut CSnapin = (*pNM).item.lParam as *mut CSnapin;
                    (*Snapin).SetDisplayName(pszText);
                    TreeView_SetItem!((*pNM).hdr.hwndFrom, core::ptr::addr_of_mut!((*pNM).item));
                    (*this_).m_DescriptionBar.SetWindowTextW((*Snapin).DisplayName().GetString());
                }
                return (FALSE) as LRESULT;
            }
    
            TVN_SELCHANGED => {
                {
                    let pNmTreeview: LPNMTREEVIEW = lParam as LPNMTREEVIEW;
                    let OldSnapin: *mut CSnapin = (*pNmTreeview).itemOld.lParam as *mut CSnapin;
                    let NewSnapin: *mut CSnapin = (*pNmTreeview).itemNew.lParam as *mut CSnapin;
    
                    (*this_).m_DescriptionBar.SetWindowTextW((*NewSnapin).DisplayName().GetString());
                    (*(*this_).m_MainWnd).UpdateAboutSnapinMenu(
                        if NewSnapin == (*(*this_).m_MainWnd).GetRootSnapin() {
                            core::ptr::null_mut()
                        } else {
                            NewSnapin
                        }
                    );
                    if (OldSnapin == (*(*this_).m_MainWnd).GetRootSnapin())
                        != (NewSnapin == (*(*this_).m_MainWnd).GetRootSnapin())
                    {
                        (*(*this_).m_MainWnd).UpdateUpToolButton(
                            if NewSnapin == (*(*this_).m_MainWnd).GetRootSnapin() {
                                (FALSE as i32)
                            } else {
                                (TRUE as i32)
                            }
                        );
                    }
                }
                return (FALSE) as LRESULT;
            }
            _ => {}
        }
    
        return 0;
    }
    pub unsafe fn OnActivate(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // m_MainWnd->UpdateToolbuttons(m_bTreeViewVisible, m_bActionsPaneVisible);
        (*(*this_).m_MainWnd).UpdateViewMenu((*this_).m_ListViewMode);
        return 0;
    }
    pub unsafe fn OnActionNewWindow(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hTreeItem: HTREEITEM = (*this_).m_TreeView.GetSelection();
        if hTreeItem.is_null() {
            return 0;
        }
    
        let Snapin: *mut CSnapin = (*this_).m_TreeView.GetItemData(hTreeItem) as *mut CSnapin;
        if Snapin.is_null() {
            return 0;
        }
    
        let mut mcs: MDICREATESTRUCT = core::mem::zeroed();
        let hChild: HWND;
        let mut title = CStringW::new();
    
        mcs.szTitle = (*Snapin).DisplayName().GetString() as LPCWSTR;
        mcs.szClass = CConsoleWnd::GetWndClassName();
        mcs.hOwner = _AtlBaseModule.GetModuleInstance();
        mcs.x = CW_USEDEFAULT;
        mcs.cx = mcs.x;
        mcs.y = CW_USEDEFAULT;
        mcs.cy = mcs.y;
        mcs.style = MDIS_ALLCHILDSTYLES;
    
        let child: *mut CConsoleWnd = CConsoleWnd__new_0((*this_).m_MainWnd, Snapin);
        let mut bMaximized: BOOL = (FALSE as i32);
        let hWndOld: HWND = (*(*this_).m_MainWnd).m_MDIClient.SendMessageW__0(
            WM_MDIGETACTIVE,
            0,
            core::ptr::addr_of_mut!(bMaximized) as LPARAM,
        ) as HWND;
        mcs.lParam = (bMaximized != 0 || hWndOld.is_null()) as LPARAM;
        hChild = (*(*this_).m_MainWnd).m_MDIClient.SendMessageW__0(
            WM_MDICREATE,
            0,
            core::ptr::addr_of_mut!(mcs) as LONG_PTR,
        ) as HWND;
        if hChild.is_null() {
            CConsoleWnd__delete(child);
        }
    
        return 0;
    }
    pub unsafe fn OnActionRename(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_TreeView.EditLabel((*this_).m_TreeView.GetSelection());
        return 0;
    }
    pub unsafe fn OnViewMode(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return (*(*this_).m_MainWnd).OnViewMode(wNotifyCode, wID, hWndCtl, bHandled);
    }
    pub unsafe fn OnViewCustomize(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return (*(*this_).m_MainWnd).OnViewCustomize(wNotifyCode, wID, hWndCtl, bHandled);
    }
    pub unsafe fn IsTreeViewVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bTreeViewVisible
    }
    pub unsafe fn SetTreeViewVisible(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bTreeViewVisible = bVisible;
        (*this_).UpdateLayout();
    }
    pub unsafe fn IsStatusBarVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bStatusBarVisible
    }
    pub unsafe fn SetStatusBarVisible(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bStatusBarVisible = bVisible;
        (*this_).UpdateLayout();
    }
    pub unsafe fn IsDescriptionBarVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bDescriptionBarVisible
    }
    pub unsafe fn SetDescriptionBarVisible(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bDescriptionBarVisible = bVisible;
        (*this_).UpdateLayout();
    }
    pub unsafe fn IsActionsPaneVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bActionsPaneVisible
    }
    pub unsafe fn SetActionsPaneVisible(&mut self, bVisible: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bActionsPaneVisible = bVisible;
        (*this_).UpdateLayout();
    }
    pub unsafe fn UpdateView(&mut self) {
        let this_: *mut Self = self;
        (*this_).UpdateTreeView();
    }
    pub unsafe fn SetStatusBarText(&mut self, pszStatusText: LPWSTR) {
        let this_: *mut Self = self;
        (*this_).m_StatusBar.SetText(pszStatusText);
    }
    pub unsafe fn SelectParent(&mut self) {
        let this_: *mut Self = self;
        let hItem: HTREEITEM = (*this_).m_TreeView.GetSelection();
        if hItem.is_null() {
            return;
        }
    
        if hItem == (*this_).m_TreeView.GetNextItem(hItem, (TVGN_ROOT as _)) {
            return;
        }
    
        let hParent: HTREEITEM = (*this_).m_TreeView.GetNextItem(hItem, (TVGN_PARENT as _));
        if !hParent.is_null() {
            (*this_).m_TreeView.SelectItem(hParent, TVGN_CARET);
        }
    }
    pub unsafe fn GetListViewMode(&mut self) -> LISTVIEW_MODE {
        let this_: *mut Self = self;
        (*this_).m_ListViewMode
    }
    pub unsafe fn SetListViewMode(&mut self, ListViewMode: LISTVIEW_MODE) {
        let this_: *mut Self = self;
        (*this_).m_ListViewMode = ListViewMode;
        SetWindowLong((*this_).m_ListView._base.m_hWnd, GWL_STYLE, (GetWindowLong((*this_).m_ListView._base.m_hWnd, GWL_STYLE) & (!LVS_TYPEMASK) as i32) | (ViewModeToStyle((*this_).m_ListViewMode)) as i32);
    }
    pub unsafe fn SaveView(&mut self, mscFile: *mut MscFile, pParentElement: *mut IXMLDOMElement) {
        let this_: *mut Self = self;
        let mut pViewElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pViewOptionsElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut szBuffer: [u16; 32] = [0; 32];
        let mut hr: HRESULT = S_OK;
    
        /* <View ID="1" ScopePaneWidth="292" ActionsPaneWidth="-1"> */
        'CleanUp: {
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_8E637B93C326.as_ptr(), pParentElement as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pViewElement));
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), (*this_).m_ViewId);
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_578E89C9480F.as_ptr(), szBuffer.as_ptr(), pViewElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), (*this_).m_iTreeViewWidth);
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_87027C6B1057.as_ptr(), szBuffer.as_ptr(), pViewElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), (*this_).m_iActionsPaneWidth);
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_CD85856650BC.as_ptr(), szBuffer.as_ptr(), pViewElement);
            if hr < 0 {
                break 'CleanUp;
            }
    
            (*mscFile).SaveWindowPlacement(core::ptr::addr_of_mut!((*this_).__base0), pViewElement);
    
            /* <ViewOptions ViewMode="Report" ScopePaneVisible="true" ActionsPaneVisible="true" DescriptionBarVisible="false" DefaultColumn0Width="200" DefaultColumn1Width="0"/> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_ACF7BB451AB4.as_ptr(), pViewElement as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pViewOptionsElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_60E82282DE60.as_ptr(), ViewModeToString(((*this_).m_ListViewMode) as i32), pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_BDCA0F318C15.as_ptr(), if (*this_).IsTreeViewVisible() != 0 { CPP_COM_WSTR_CD28B1A69262.as_ptr() } else { CPP_COM_WSTR_44C8518BCAAF.as_ptr() }, pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            }
            if (*this_).IsStatusBarVisible() == 0 {
                hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_84602D53EA5F.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pViewOptionsElement);
                if hr < 0 {
                    break 'CleanUp;
                }
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_BEC2836C8636.as_ptr(), if (*this_).IsActionsPaneVisible() != 0 { CPP_COM_WSTR_CD28B1A69262.as_ptr() } else { CPP_COM_WSTR_44C8518BCAAF.as_ptr() }, pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_A52B679307C3.as_ptr(), if (*this_).IsDescriptionBarVisible() != 0 { CPP_COM_WSTR_CD28B1A69262.as_ptr() } else { CPP_COM_WSTR_44C8518BCAAF.as_ptr() }, pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_E999B61173C7.as_ptr(), CPP_COM_WSTR_34D297E2847F.as_ptr(), pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_25B61379E7E9.as_ptr(), CPP_COM_WSTR_4F8320D91E97.as_ptr(), pViewOptionsElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
            SAFE_RELEASE!(pViewOptionsElement); /* </ViewOptions> */
        }
    
        // CleanUp:
        SAFE_RELEASE!(pViewElement); /* </View> */
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(&(*riid), &(IID_IConsole)) {
            *ppvObject = core::ptr::addr_of_mut!((*this_).__base1) as *mut core::ffi::c_void;
            (*this_).AddRef();
            return S_OK;
        }
        else if IsEqualGUID!(&(*riid), &(IID_IConsole2)) {
            *ppvObject = core::ptr::addr_of_mut!((*this_).__base1) as *mut core::ffi::c_void;
            (*this_).AddRef();
            return S_OK;
        }
        else if IsEqualGUID!(&(*riid), &(IID_IConsoleNameSpace)) {
            *ppvObject = core::ptr::addr_of_mut!((*this_).__base2) as *mut core::ffi::c_void;
            (*this_).AddRef();
            return S_OK;
        }
        else if IsEqualGUID!(&(*riid), &(IID_IConsoleNameSpace2)) {
            *ppvObject = core::ptr::addr_of_mut!((*this_).__base2) as *mut core::ffi::c_void;
            (*this_).AddRef();
            return S_OK;
        }
        else {
            let mut guidstr: CComHeapPtr<OLECHAR> = CComHeapPtr::new();
            let hr: HRESULT = StringFromCLSID(riid, guidstr.put());
            if hr >= 0 {
                DPRINT1!("Unhandled riid: %S\n", guidstr.op_to_LPCWSTR());
            }
            else {
                DPRINT1!("Unhandled riid\n");
            }
        }
    
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        // DPRINT("%s()\n", __FUNCTION__);
        2
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        // DPRINT("%s()\n", __FUNCTION__);
        1
    }
    pub unsafe fn SetHeader(&mut self, pHeader: LPHEADERCTRL) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn SetToolbar(&mut self, pToolbar: LPTOOLBAR) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn QueryResultView(&mut self, pUnknown: *mut LPUNKNOWN) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn QueryScopeImageList(&mut self, ppImageList: *mut LPIMAGELIST) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s(%p)\n", __FUNCTION__);
        if ppImageList.is_null() {
            return E_INVALIDARG;
        }
    
        __debugbreak();
        //*ppImageList = m_ScopeImageList;
        return S_OK;
    }
    pub unsafe fn QueryResultImageList(&mut self, ppImageList: *mut LPIMAGELIST) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn UpdateAllViews(&mut self, lpDataObject: LPDATAOBJECT, data: LPARAM, hint: LONG_PTR) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn QueryConsoleVerb(&mut self, ppConsoleVerb: *mut LPCONSOLEVERB) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn SelectScopeItem(&mut self, hScopeItem: HSCOPEITEM) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn GetMainWindow(&mut self, phwnd: *mut HWND) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn NewWindow(&mut self, hScopeItem: HSCOPEITEM, lOptions: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn Expand__0(&mut self, hItem: HSCOPEITEM, bExpand: BOOL) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn IsTaskpadViewPreferred(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn SetStatusText(&mut self, pszStatusText: LPWSTR) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn InsertItem(&mut self, item: LPSCOPEDATAITEM) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn DeleteItem(&mut self, hItem: HSCOPEITEM, fDeleteThis: LONG) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn SetItem(&mut self, item: LPSCOPEDATAITEM) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn GetItem(&mut self, item: LPSCOPEDATAITEM) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn GetChildItem(&mut self, item: HSCOPEITEM, pItemChild: *mut HSCOPEITEM, plCookie: *mut LONG) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn GetNextItem(&mut self, item: HSCOPEITEM, pItemNext: *mut HSCOPEITEM, plCookie: *mut LONG) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn GetParentItem(&mut self, item: HSCOPEITEM, pItemParent: *mut HSCOPEITEM, plCookie: *mut LONG) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn Expand__1(&mut self, hItem: HSCOPEITEM) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
    pub unsafe fn AddExtension(&mut self, hItem: HSCOPEITEM, lpClsid: LPCLSID) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
}

// IConsole2::MessageBox. The translation lost this member (the SDK macro MessageBox -> MessageBoxW
// made the vtable thunk resolve to CWindow::MessageBox); the C++ body is the same stub as its neighbours.
impl CConsoleWnd {
    pub unsafe fn MessageBoxW(&mut self, lpszText: LPCWSTR, lpszTitle: LPCWSTR, fuStyle: UINT, piRetval: *mut c_int) -> HRESULT {
        let this_: *mut Self = self;
        DPRINT!("%s()\n", __FUNCTION__);
        __debugbreak();
        return E_NOTIMPL;
    }
}
