use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;








































impl CListView {
    pub unsafe fn Create(&mut self, hWndParent: HWND, rect: _U_RECT, szWindowName: LPCTSTR, dwStyle: DWORD, dwExStyle: DWORD, MenuOrID: _U_MENUorID, lpCreateParam: LPVOID) -> HWND {
        let this_: *mut Self = self;
        (*this_).__base0.m_hWnd = CreateWindowEx(dwExStyle,
                                  WC_LISTVIEW,
                                  szWindowName,
                                  dwStyle,
                                  (*rect.m_lpRect).left,
                                  (*rect.m_lpRect).top,
                                  (*rect.m_lpRect).right - (*rect.m_lpRect).left,
                                  (*rect.m_lpRect).bottom - (*rect.m_lpRect).top,
                                  hWndParent,
                                  MenuOrID.m_hMenu,
                                  _AtlBaseModule.GetModuleInstance(),
                                  lpCreateParam);

        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn SetRedraw(&mut self, redraw: BOOL) {
        let this_: *mut Self = self;
            (*this_).__base0.SendMessageW__0(WM_SETREDRAW, redraw as WPARAM, 0);
        }
    pub unsafe fn SetExtendedListViewStyle(&mut self, dw: DWORD, dwMask: DWORD) -> DWORD {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_SETEXTENDEDLISTVIEWSTYLE as _, dwMask as _, dw as _) as DWORD;
    }
    pub unsafe fn InsertColumn__0(&mut self, iCol: i32, pcol: *mut LVCOLUMNW) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_INSERTCOLUMN) as UINT, iCol as WPARAM, pcol as LPARAM) as i32;
    }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST, iImageList: i32) -> HIMAGELIST {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_SETIMAGELIST as UINT, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST;
    }
    pub unsafe fn InsertItem(&mut self, pitem: *const LVITEMW) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_INSERTITEM) as UINT, 0, pitem as LPARAM) as i32;
    }
    pub unsafe fn GetItem(&mut self, pitem: *mut LVITEMW) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_GETITEM) as UINT, 0, pitem as LPARAM) as BOOL;
    }
    pub unsafe fn FindItem(&mut self, iStart: i32, plvfi: *const LVFINDINFOW) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_FINDITEM) as UINT, iStart as WPARAM, plvfi as LPARAM) as BOOL;
    }
    pub unsafe fn GetItemCount(&mut self) -> i32 {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(LVM_GETITEMCOUNT, 0, 0)) as i32;
    }
    pub unsafe fn DeleteAllItems(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(LVM_DELETEALLITEMS, 0, 0)) as BOOL;
    }
    pub unsafe fn GetSelectedCount(&mut self) -> UINT {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_GETSELECTEDCOUNT, 0, 0) as UINT;
    }
    pub unsafe fn SortItems(&mut self, pfnCompare: PFNLVCOMPARE, lParam: PVOID) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_SORTITEMS, lParam as WPARAM, pfnCompare as LPARAM) as BOOL;
    }
    pub unsafe fn GetSelectionMark(&mut self) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_GETSELECTIONMARK, 0, 0) as i32;
    }
    pub unsafe fn GetNextItem(&mut self, i: i32, flags: WORD) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_GETNEXTITEM, i as WPARAM, MAKELPARAM!(flags, 0)) as i32;
    }
    pub unsafe fn GetItemState(&mut self, i: i32, mask: UINT) -> UINT {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_GETITEMSTATE, i as WPARAM, mask as LPARAM) as UINT;
    }
    pub unsafe fn SetItemState(&mut self, i: i32, state: UINT, mask: UINT) {
        let this_: *mut Self = self;
            let mut item: LVITEMW = core::mem::zeroed();
            item.stateMask = mask;
            item.state = state;
            (*this_).__base0.SendMessageW__0(LVM_SETITEMSTATE, i as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM);
        }
    pub unsafe fn SetItemText(&mut self, i: i32, subItem: i32, text: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        let mut item: LVITEMW = core::mem::zeroed();
        item.iSubItem = subItem;
        item.pszText = text as LPWSTR;
        return (*this_).__base0.SendMessageW__0((LVM_SETITEMTEXT) as UINT, i as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM) as BOOL;
    }
    pub unsafe fn GetItemData(&mut self, i: i32) -> DWORD_PTR {
        let this_: *mut Self = self;
            let mut lvItem: LVITEMW = core::mem::zeroed();
            lvItem.mask = LVIF_PARAM;
            lvItem.iItem = i;
            let ret: BOOL = (*this_).GetItem(core::ptr::addr_of_mut!(lvItem));
            return (if ret != 0 { lvItem.lParam as DWORD_PTR } else { 0 }) as DWORD_PTR;
        }
    pub unsafe fn GetItemText(&mut self, iItem: i32, iSubItem: i32, pszText: LPTSTR, cchTextMax: i32) {
        let this_: *mut Self = self;
        let mut itemInfo: LV_ITEM = core::mem::zeroed();
        itemInfo.iSubItem = iSubItem;
        itemInfo.pszText = pszText;
        itemInfo.cchTextMax = cchTextMax;

        (*this_).__base0.SendMessageW__0((LVM_GETITEMTEXT) as UINT, iItem as WPARAM, core::ptr::addr_of_mut!(itemInfo) as LPARAM);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}

impl CToolbar {
    pub unsafe fn SetButtonStructSize(&mut self) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_BUTTONSTRUCTSIZE as u32, core::mem::size_of::<TBBUTTON>() as WPARAM, 0 as LPARAM)) as DWORD;
    }
    pub unsafe fn AddButtons(&mut self, count: i32, buttons: *mut TBBUTTON) -> DWORD {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((TB_ADDBUTTONS) as UINT, count as WPARAM, buttons as LPARAM) as DWORD;
    }
    pub unsafe fn DeleteButton(&mut self, index: i32) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_DELETEBUTTON, index as WPARAM, 0)) as DWORD;
    }
    pub unsafe fn GetButtonInfo(&mut self, cmdId: i32, info: *mut TBBUTTONINFOW) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0((TB_GETBUTTONINFO) as UINT, cmdId as WPARAM, info as LPARAM)) as DWORD;
    }
    pub unsafe fn AutoSize(&mut self) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_AUTOSIZE, 0, 0)) as DWORD;
    }
    pub unsafe fn GetIdealSize(&mut self, useHeight: BOOL, size: LPSIZE) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_GETIDEALSIZE, (useHeight) as WPARAM, size as LPARAM)) as DWORD;
    }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST) -> HIMAGELIST {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_SETIMAGELIST, 0, himl as LPARAM)) as HIMAGELIST;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}

impl CStatusBar {
    pub unsafe fn SetText(&mut self, lpszText: LPCWSTR) {
        let this_: *mut Self = self;
            (*this_).__base0.SendMessageW__0((SB_SETTEXT) as UINT, (SBT_NOBORDERS) as WPARAM, lpszText as LPARAM);
        }
    pub unsafe fn Create(&mut self, hwndParent: HWND, hMenu: HMENU) -> HWND {
        let this_: *mut Self = self;
        (*this_).__base0.m_hWnd = CreateWindowExW(0,
            STATUSCLASSNAMEW,
            core::ptr::null_mut(),
            WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
            0, 0, 0, 0,
            hwndParent,
            hMenu,
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut());

        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}

impl CTreeView {
    pub unsafe fn Create(&mut self, hwndParent: HWND, dwStyles: DWORD) -> HWND {
        let this_: *mut Self = self;
        if dwStyles == 0 {
            dwStyles = WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_SHOWSELALWAYS;
        }

        (*this_).__base0.m_hWnd = CreateWindowExW(WS_EX_CLIENTEDGE,
            WC_TREEVIEWW,
            CPP_COM_WSTR_96A296D224F2.as_ptr(),
            dwStyles,
            0, 28, 200, 350,
            hwndParent,
            core::ptr::null_mut(),
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut());

        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn AddItem(&mut self, hParent: HTREEITEM, lpText: LPWSTR, Image: INT, SelectedImage: INT, lParam: LPARAM) -> HTREEITEM {
        let this_: *mut Self = self;
            let mut Insert: TVINSERTSTRUCTW = core::mem::zeroed();

            ZeroMemory!(core::ptr::addr_of_mut!(Insert) as *mut _, core::mem::size_of::<TV_INSERTSTRUCT>());

            Insert.__bindgen_anon_1.item.mask = TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
            Insert.hInsertAfter = TVI_LAST;
            Insert.hParent = hParent;
            Insert.__bindgen_anon_1.item.iSelectedImage = SelectedImage;
            Insert.__bindgen_anon_1.item.iImage = Image;
            Insert.__bindgen_anon_1.item.lParam = lParam;
            Insert.__bindgen_anon_1.item.pszText = lpText;

            return (*this_).InsertItem(core::ptr::addr_of_mut!(Insert));
        }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST, iImageList: i32) -> HIMAGELIST {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST;
    }
    pub unsafe fn InsertItem(&mut self, pitem: *const TVINSERTSTRUCTW) -> HTREEITEM {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((TVM_INSERTITEM) as UINT, 0, pitem as LPARAM) as HTREEITEM;
    }
    pub unsafe fn Expand(&mut self, item: HTREEITEM, action: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TVM_EXPAND, action as WPARAM, item as LPARAM) as BOOL;
    }
    pub unsafe fn SelectItem(&mut self, item: HTREEITEM, action: DWORD) -> BOOL {
        let this_: *mut Self = self;
            return (*this_).__base0.SendMessageW__0(TVM_SELECTITEM, action as WPARAM, item as LPARAM) as BOOL;
        }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}

impl CTooltips {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}
