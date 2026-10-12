use crate::prelude::*;
use core::ffi::c_void;























impl CAddDialog {
    pub unsafe fn CreateSnapinAliases(&mut self, ParentAlias: *mut CSnapinAlias, ParentSnapin: *mut CSnapin) {
        let this_: *mut Self = self;
        let mut pos: POSITION = (*ParentSnapin).m_SubNodes.GetHeadPosition();
        while !pos.is_null() {
            let Snapin: *mut CSnapin = *(*ParentSnapin).m_SubNodes.GetNext(&mut pos);
            if !Snapin.is_null() {
                let Alias: *mut CSnapinAlias = CSnapinAlias__new_0(ParentAlias, Snapin);
                (*ParentAlias).m_SubNodes.AddTail(Alias);
                (*this_).CreateSnapinAliases(Alias, Snapin);
            }
        }
    }
    pub unsafe fn ReconnectSnapins(&mut self, ParentAlias: *mut CSnapinAlias, ParentSnapin: *mut CSnapin) {
        let this_: *mut Self = self;
        (*ParentSnapin).m_SubNodes.RemoveAll();
    
        let mut pos: POSITION = (*ParentAlias).m_SubNodes.GetHeadPosition();
        while !pos.is_null() {
            let Alias: *mut CSnapinAlias = *(*ParentAlias).m_SubNodes.GetNext(&mut pos);
            if !Alias.is_null() {
                if (*Alias).IsDeleted() == 0 {
                    (*ParentSnapin).m_SubNodes.AddTail((*Alias).Snapin());
                    (*this_).ReconnectSnapins(Alias, (*Alias).Snapin());
                }
            }
        }
    }
    pub unsafe fn DeleteSnapinAliasesRecursive(&mut self, ParentAlias: *mut CSnapinAlias, RemoveDeletedSnapins: BOOL) {
        let this_: *mut Self = self;
        let mut pos: POSITION = (*ParentAlias).m_SubNodes.GetHeadPosition();
        while !pos.is_null() {
            let Alias: *mut CSnapinAlias = *(*ParentAlias).m_SubNodes.GetNext(&mut pos);
            if !Alias.is_null() {
                (*this_).DeleteSnapinAliasesRecursive(Alias, RemoveDeletedSnapins);
                (*Alias).m_SubNodes.RemoveAll();
                if RemoveDeletedSnapins != 0 && ((*Alias).IsDeleted() != 0) {
                    CSnapin__delete((*Alias).Snapin());
                }
                CSnapinAlias__delete(Alias);
            }
        }
    }
    pub unsafe fn DeleteSnapinAliases(&mut self, RemoveDeletedSnapins: BOOL) {
        let this_: *mut Self = self;
        if !(*this_).m_RootAlias.is_null() {
            (*this_).DeleteSnapinAliasesRecursive((*this_).m_RootAlias, RemoveDeletedSnapins);
            (*(*this_).m_RootAlias).m_SubNodes.RemoveAll();
            // C source was corrupted here: `CSnapinAlias__delete(m_RootAlias)otAlias;`
            // Interpreted as deleting this_->m_RootAlias.
            CSnapinAlias__delete((*this_).m_RootAlias);
            (*this_).m_RootAlias = core::ptr::null_mut();
        }
    }
    pub unsafe fn __ctor_0(&mut self, MainWnd: *mut CMainWnd, Console: *mut CConsoleWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
            (*this_).m_MainWnd = MainWnd;
            // m_Console is a CComPtr<CConsoleWnd>: the assignment AddRef()s, and CConsoleWnd::AddRef/Release
            // are constants (2 / 1) that touch nothing, so a plain pointer is the same behaviour.
            (*this_).m_Console = Console;
            (*this_).m_Advanced = (FALSE as i32);
        }
    }
    pub unsafe fn InitLV(&mut self, listView: &mut CListView) {
        let this_: *mut Self = self;
        listView.DeleteAllItems();
    
        let mut module = CStringW::from_resource(IDS_MODULE);
        listView.InsertColumn(0, module.GetString() as LPWSTR, (LVCFMT_LEFT as _), 130, 0);
    
        let mut vendor = CStringW::from_resource(IDS_VENDOR);
        listView.InsertColumn(1, vendor.GetString() as LPWSTR, (LVCFMT_LEFT as _), 120, 1);
    
        listView.SetImageList((*(*this_).m_MainWnd).SnapinImageList(), (LVSIL_SMALL as _));
    }
    pub unsafe fn InsertListItem(&mut self, listView: &mut CListView, CacheEntry: *mut CSnapinCacheEntry) {
        let this_: *mut Self = self;
        let mut lvi: LVITEM = core::mem::zeroed();
        lvi.mask = LVIF_TEXT | LVIF_PARAM | LVIF_IMAGE;
        lvi.lParam = CacheEntry as LPARAM;
        lvi.pszText = (*CacheEntry).Name().GetString() as LPWSTR;
        lvi.iItem = (INT_MAX as i32);
        lvi.iImage = (*CacheEntry).NormalImageIndex();
        let item: i32 = listView.InsertItem(core::ptr::addr_of_mut!(lvi));
    
        listView.SetItemText(item, 1, (*CacheEntry).Provider().GetString() as LPWSTR);
    }
    pub unsafe fn InitTV(&mut self, treeView: &mut CTreeView) {
        let this_: *mut Self = self;
        treeView.SetImageList((*(*this_).m_MainWnd).SnapinImageList(), (TVSIL_NORMAL as _));
    }
    pub unsafe fn InsertTreeItem(&mut self, treeView: &mut CTreeView, Alias: *mut CSnapinAlias, hParent: HTREEITEM, hInsertAfter: HTREEITEM) -> HTREEITEM {
        let this_: *mut Self = self;
        let mut Insert: TVINSERTSTRUCTW = core::mem::zeroed();
    
        ZeroMemory!(core::ptr::addr_of_mut!(Insert) as *mut core::ffi::c_void, core::mem::size_of::<TVINSERTSTRUCTW>());
    
        Insert.__bindgen_anon_1.item.mask = TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
        Insert.__bindgen_anon_1.item.pszText = (*(*Alias).Snapin()).DisplayName().GetString() as LPWSTR;
        Insert.__bindgen_anon_1.item.iImage = (*(*(*Alias).Snapin()).CacheEntry()).NormalImageIndex();
        Insert.__bindgen_anon_1.item.iSelectedImage = (*(*(*Alias).Snapin()).CacheEntry()).OpenImageIndex();
        Insert.__bindgen_anon_1.item.lParam = Alias as LPARAM;
    
        Insert.hParent = hParent;
        Insert.hInsertAfter = hInsertAfter;
    
        return treeView.InsertItem(core::ptr::addr_of_mut!(Insert));
    }
    pub unsafe fn InitCB(&mut self, comboBox: &mut CWindow) {
        let this_: *mut Self = self;
        comboBox.SendMessageW__0(CBEM_SETIMAGELIST, 0, (*(*this_).m_MainWnd).SnapinImageList() as LPARAM);
    }
    pub unsafe fn AppendComboBoxItem(&mut self, comboBox: &mut CWindow, Alias: *mut CSnapinAlias, iIndent: i32) {
        let this_: *mut Self = self;
        let mut Item: COMBOBOXEXITEMW = core::mem::zeroed();
    
        ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut _, core::mem::size_of::<COMBOBOXEXITEMW>() as usize);
    
        Item.mask = CBEIF_TEXT | CBEIF_IMAGE | CBEIF_SELECTEDIMAGE | CBEIF_INDENT | CBEIF_LPARAM;
        Item.iItem = -1;
        Item.pszText = (*(*Alias).Snapin()).DisplayName().GetString() as LPWSTR;
        Item.cchTextMax = -1;
        Item.iImage = (*(*(*Alias).Snapin()).CacheEntry()).NormalImageIndex();
        Item.iSelectedImage = (*(*(*Alias).Snapin()).CacheEntry()).NormalImageIndex();
        Item.iIndent = iIndent;
        Item.lParam = Alias as LPARAM;
    
        comboBox.SendMessageW__0((CBEM_INSERTITEM) as UINT, 0, core::ptr::addr_of_mut!(Item) as LPARAM);
    }
    pub unsafe fn InsertSnapinAliasesRecursive(&mut self, ParentAlias: *mut CSnapinAlias, Indent: INT) {
        let this_: *mut Self = self;
        let mut pos: POSITION = (*ParentAlias).m_SubNodes.GetHeadPosition();
        while pos != core::ptr::null_mut() {
            let Alias: *mut CSnapinAlias = *(*ParentAlias).m_SubNodes.GetNext(&mut pos);
            if !Alias.is_null() {
                (*Alias).hTreeItem = (*this_).InsertTreeItem(
                    &mut *core::ptr::addr_of_mut!((*this_).m_Selected),
                    Alias,
                    (*ParentAlias).hTreeItem,
                    TVI_LAST,
                );
                (*this_).AppendComboBoxItem(
                    &mut *core::ptr::addr_of_mut!((*this_).m_ParentList),
                    Alias,
                    Indent,
                );
                (*this_).InsertSnapinAliasesRecursive(Alias, Indent + 1);
            }
        }
        (*this_).m_Selected.Expand((*ParentAlias).hTreeItem, TVE_EXPAND);
    }
    pub unsafe fn OnInitDialog(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*(*this_).m_Console).__base0.m_hWnd);
    
        (*this_).m_Available.Attach((*this_).__base0.GetDlgItem((IDC_ADD_LIST_AVAILABLE as i32)));
        (*this_).m_Selected.Attach((*this_).__base0.GetDlgItem((IDC_ADD_LIST_SELECTED as i32)));
        (*this_).m_BtnAdd.Attach((*this_).__base0.GetDlgItem((IDC_ADD_BUTTON_ADD as i32)));
        (*this_).m_BtnRemove.Attach((*this_).__base0.GetDlgItem((IDC_ADD_BUTTON_REMOVE as i32)));
        (*this_).m_BtnUp.Attach((*this_).__base0.GetDlgItem((IDC_ADD_BUTTON_UP as i32)));
        (*this_).m_BtnDown.Attach((*this_).__base0.GetDlgItem((IDC_ADD_BUTTON_DOWN as i32)));
        (*this_).m_Description.Attach((*this_).__base0.GetDlgItem((IDC_ADD_DESCRIPTION as i32)));
        (*this_).m_SelectedText.Attach((*this_).__base0.GetDlgItem((IDC_ADD_TEXT_SELECTED as i32)));
        (*this_).m_ParentText.Attach((*this_).__base0.GetDlgItem((IDC_ADD_TEXT_PARENT as i32)));
        (*this_).m_ParentList.Attach((*this_).__base0.GetDlgItem((IDC_ADD_LIST_PARENT as i32)));
    
        (*this_).m_ParentText.ShowWindow((SW_HIDE as i32));
        (*this_).m_ParentList.ShowWindow((SW_HIDE as i32));
    
        (*this_).InitLV(&mut (*this_).m_Available);
        (*this_).InitTV(&mut (*this_).m_Selected);
        (*this_).InitCB(&mut (*this_).m_ParentList);
    
        /* Get Snapins from the Cache */
        let mut i: i32 = 0;
        while i < (*(*this_).m_MainWnd).GetSnapinCacheCount() {
            (*this_).InsertListItem(&mut (*this_).m_Available, (*(*this_).m_MainWnd).GetSnapinCacheEntry(i));
            i += 1;
        }
    
        /* Create the snapin alias tree */
        (*this_).m_RootSnapin = (*(*this_).m_MainWnd).GetRootSnapin();
        (*this_).m_RootAlias = CSnapinAlias__new_0(core::ptr::null_mut(), (*this_).m_RootSnapin);
        (*this_).m_ParentAlias = (*this_).m_RootAlias;
    
        (*this_).CreateSnapinAliases((*this_).m_RootAlias, (*this_).m_RootSnapin);
    
        (*(*this_).m_RootAlias).hTreeItem = (*this_).InsertTreeItem(&mut (*this_).m_Selected, (*this_).m_RootAlias, TVI_ROOT, TVI_LAST);
    
        (*this_).AppendComboBoxItem(&mut (*this_).m_ParentList, (*this_).m_RootAlias, 0);
    
        (*this_).InsertSnapinAliasesRecursive((*this_).m_RootAlias, 1);
        (*this_).m_Selected.Expand((*(*this_).m_RootAlias).hTreeItem, TVE_EXPAND);
    
        (*this_).m_ParentList.SendMessageW__0(CB_SETCURSEL, 0, 0);
    
        (*this_).UpdateButtons();
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match (wID) as u32 {
            IDC_ADD_BUTTON_ADD => {
                let iItem: i32 = (*this_).m_Available.GetNextItem(-1, (LVNI_SELECTED as _));
                if iItem != -1 {
                    let CacheEntry: *mut CSnapinCacheEntry = (*this_).m_Available.GetItemData(iItem) as *mut CSnapinCacheEntry;
                    if !CacheEntry.is_null() {
                        let Alias: *mut CSnapinAlias = CSnapinAlias__new_0((*this_).m_ParentAlias, CSnapin__new_0(CacheEntry, core::ptr::null_mut()));
                        (*(*this_).m_ParentAlias).m_SubNodes.AddTail(Alias);
                        (*Alias).hTreeItem = (*this_).InsertTreeItem(&mut (*this_).m_Selected, Alias, (*(*this_).m_ParentAlias).hTreeItem, TVI_LAST);
                        (*this_).m_Selected.Expand((*(*this_).m_ParentAlias).hTreeItem, TVE_EXPAND);
                    }
                }
            }
    
            IDC_ADD_BUTTON_REMOVE => {
                let hTreeItem: HTREEITEM = (*this_).m_Selected.GetSelection();
                if hTreeItem != core::ptr::null_mut() {
                    let Alias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hTreeItem) as *mut CSnapinAlias;
                    if !Alias.is_null() {
                        (*Alias).Deleted();
                        (*this_).m_Selected.DeleteItem((*Alias).hTreeItem);
                    }
                }
            }
    
            IDC_ADD_BUTTON_UP => {
                let hSelectItem: HTREEITEM = (*this_).m_Selected.GetSelection();
                if hSelectItem != core::ptr::null_mut() {
                    let hParentItem: HTREEITEM = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_PARENT as _));
                    if hParentItem != core::ptr::null_mut() {
                        let ParentAlias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hParentItem) as *mut CSnapinAlias;
                        let Alias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hSelectItem) as *mut CSnapinAlias;
                        if !ParentAlias.is_null() && !Alias.is_null() {
                            let mut hPrevItem: HTREEITEM = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_PREVIOUS as _));
                            if hPrevItem != core::ptr::null_mut() {
                                hPrevItem = (*this_).m_Selected.GetNextItem(hPrevItem, (TVGN_PREVIOUS as _));
                                if hPrevItem == core::ptr::null_mut() {
                                    hPrevItem = TVI_FIRST;
                                }
    
                                let pos: POSITION = (*ParentAlias).m_SubNodes.FindPos(&Alias);
                                let mut prev: POSITION = pos;
                                (*ParentAlias).m_SubNodes.GetPrev(&mut prev);
                                (*ParentAlias).m_SubNodes.SwapElements(pos, prev);
    
                                (*this_).m_Selected.DeleteItem(hSelectItem);
                                (*Alias).hTreeItem = (*this_).InsertTreeItem(&mut (*this_).m_Selected, Alias, hParentItem, hPrevItem);
                                (*this_).m_Selected.SelectItem((*Alias).hTreeItem, TVGN_CARET);
                            }
                        }
                    }
                }
            }
    
            IDC_ADD_BUTTON_DOWN => {
                let hSelectItem: HTREEITEM = (*this_).m_Selected.GetSelection();
                if hSelectItem != core::ptr::null_mut() {
                    let hParentItem: HTREEITEM = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_PARENT as _));
                    if hParentItem != core::ptr::null_mut() {
                        let ParentAlias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hParentItem) as *mut CSnapinAlias;
                        let Alias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hSelectItem) as *mut CSnapinAlias;
                        if !ParentAlias.is_null() && !Alias.is_null() {
                            let hNextItem: HTREEITEM = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_NEXT as _));
                            if hNextItem != core::ptr::null_mut() {
                                let pos: POSITION = (*ParentAlias).m_SubNodes.FindPos(&Alias);
                                let mut next: POSITION = pos;
                                (*ParentAlias).m_SubNodes.GetNext(&mut next);
                                (*ParentAlias).m_SubNodes.SwapElements(pos, next);
    
                                (*this_).m_Selected.DeleteItem(hSelectItem);
                                (*Alias).hTreeItem = (*this_).InsertTreeItem(&mut (*this_).m_Selected, Alias, hParentItem, hNextItem);
                                (*this_).m_Selected.SelectItem((*Alias).hTreeItem, TVGN_CARET);
                            }
                        }
                    }
                }
            }
    
            IDC_ADD_ADVANCED => {
                let mut AdvancedDialog: CAddAdvancedDialog = CAddAdvancedDialog__make_0(this_);
                if AdvancedDialog.DoModal((*this_).__base0.m_hWnd, 0 as LPARAM) == (IDOK as _) {
                    let mut rc: RECT = core::mem::zeroed();
                    let Offset: i32 = 50;
    
                    GetWindowRect((*this_).m_Selected._base.m_hWnd, core::ptr::addr_of_mut!(rc));
                    MapWindowPoints(HWND_DESKTOP, (*this_).__base0.m_hWnd, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
                    if (*this_).m_Advanced != 0 {
                        rc.top += Offset;
                    } else {
                        rc.top -= Offset;
                    }
    
                    MoveWindow((*this_).m_Selected._base.m_hWnd, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, (TRUE as i32));
    
                    GetWindowRect((*this_).m_SelectedText.m_hWnd, core::ptr::addr_of_mut!(rc));
                    MapWindowPoints(HWND_DESKTOP, (*this_).__base0.m_hWnd, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
                    if (*this_).m_Advanced != 0 {
                        rc.top += Offset;
                        rc.bottom += Offset;
                    } else {
                        rc.top -= Offset;
                        rc.bottom -= Offset;
                    }
    
                    MoveWindow((*this_).m_SelectedText.m_hWnd, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, (TRUE as i32));
    
                    if (*this_).m_Advanced != 0 {
                        (*this_).m_ParentText.ShowWindow((SW_SHOW as i32));
                        (*this_).m_ParentList.ShowWindow((SW_SHOW as i32));
                    } else {
                        (*this_).m_ParentText.ShowWindow((SW_HIDE as i32));
                        (*this_).m_ParentList.ShowWindow((SW_HIDE as i32));
                    }
                }
            }
    
            IDC_ADD_LIST_PARENT => {
                let iItem: INT = (CWindow::SendMessageW__1((*this_).m_ParentList.op_to_HWND(), CB_GETCURSEL, 0, 0)) as i32;
                if iItem != CB_ERR {
                    (*this_).m_ParentAlias = CWindow::SendMessageW__1((*this_).m_ParentList.op_to_HWND(), CB_GETITEMDATA, (iItem) as WPARAM, 0) as *mut CSnapinAlias;
                }
            }
    
            IDOK => {
                (*this_).ReconnectSnapins((*this_).m_RootAlias, (*this_).m_RootSnapin);
                (*this_).DeleteSnapinAliases((TRUE) as BOOL);
                (*this_).EndDialog((IDOK as _));
                return 0;
            }
    
            IDCANCEL => {
                (*this_).DeleteSnapinAliases((FALSE) as BOOL);
                (*this_).EndDialog((IDCANCEL as _));
                return 0;
            }
    
            _ => {}
        }
    
        (*this_).UpdateButtons();
    
        0
    }
    pub unsafe fn OnItemChanged(&mut self, uCode: INT, hdr: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut iItem: i32 = (*this_).m_Available.GetNextItem(-1, (LVNI_SELECTED as _));
        let mut CacheEntry: *mut CSnapinCacheEntry = core::ptr::null_mut();
        if iItem != -1 && {
            CacheEntry = (*this_).m_Available.GetItemData(iItem) as *mut CSnapinCacheEntry;
            !CacheEntry.is_null()
        } {
            (*this_).m_Description.SetWindowTextW((*CacheEntry).Description().GetString() as LPWSTR);
        } else {
            (*this_).m_Description.SetWindowTextW(core::ptr::null_mut());
        }
    
        (*this_).UpdateButtons();
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnSelectionChanged(&mut self, uCode: INT, hdr: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateButtons();
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnItemDblClicked(&mut self, uCode: INT, hdr: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*hdr).hwndFrom == (*this_).m_Available._base.m_hWnd {
            let lpnmitem: LPNMITEMACTIVATE = hdr as LPNMITEMACTIVATE;
            if (*lpnmitem).iItem != -1 {
                let CacheEntry: *mut CSnapinCacheEntry = (*this_).m_Available.GetItemData((*lpnmitem).iItem) as *mut CSnapinCacheEntry;
                if !CacheEntry.is_null() {
                    let Alias: *mut CSnapinAlias = CSnapinAlias__new_0((*this_).m_ParentAlias, CSnapin__new_0(CacheEntry, core::ptr::null_mut()));
                    (*(*this_).m_ParentAlias).m_SubNodes.AddTail(Alias);
    //                Alias->Snapin()->OnAdd(m_Console);
                    let tv: *mut CTreeView = core::ptr::addr_of_mut!((*this_).m_Selected);
                    (*Alias).hTreeItem = (*this_).InsertTreeItem(&mut *tv, Alias, (*(*this_).m_ParentAlias).hTreeItem, TVI_LAST);
                    let sel: *mut CTreeView = core::ptr::addr_of_mut!((*this_).m_Selected);
                    (*sel).Expand((*(*this_).m_ParentAlias).hTreeItem, TVE_EXPAND);
                }
            }
        }
        return (TRUE) as LRESULT;
    }
    pub unsafe fn UpdateButtons(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_BtnAdd.EnableWindow((ListView_GetSelectedCount!((*this_).m_Available._base.m_hWnd) > 0) as i32);
    
        let mut ParentAlias: *mut CSnapinAlias = core::ptr::null_mut();
        let mut hTreeItem: HTREEITEM = (*this_).m_Selected.GetNextItem(core::ptr::null_mut(), (TVGN_CARET as _));
        if !hTreeItem.is_null() {
            let Alias: *mut CSnapinAlias = (*this_).m_Selected.GetItemData(hTreeItem) as *mut CSnapinAlias;
            if !Alias.is_null() {
                ParentAlias = (*Alias).Parent();
            }
        }
    
        (*this_).m_BtnRemove.EnableWindow((!hTreeItem.is_null() && !ParentAlias.is_null()) as i32);
    
        let mut hPrevItem: HTREEITEM = core::ptr::null_mut();
        let mut hNextItem: HTREEITEM = core::ptr::null_mut();
        let hSelectItem: HTREEITEM = (*this_).m_Selected.GetSelection();
        if !hSelectItem.is_null() {
            hPrevItem = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_PREVIOUS as _));
            hNextItem = (*this_).m_Selected.GetNextItem(hSelectItem, (TVGN_NEXT as _));
        }
    
        (*this_).m_BtnUp.EnableWindow((!hPrevItem.is_null()) as i32);
        (*this_).m_BtnDown.EnableWindow((!hNextItem.is_null()) as i32);
    }
    pub unsafe fn GetAdvanced(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_Advanced
    }
    pub unsafe fn SetAdvanced(&mut self, Advanced: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_Advanced = Advanced;
    }
}
