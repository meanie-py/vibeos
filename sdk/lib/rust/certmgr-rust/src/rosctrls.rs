//! The slice of `sdk/include/reactos/ui/rosctrls.h` (CListView, CTreeView) and the inherited
//! ATL `CWindow` members that certmgr calls. These are header-only C++ methods, so the bindgen
//! island carries only the `{ _base: ATL_CWindow }` structs; each method below is the header's
//! body with its message id and WPARAM/LPARAM packing kept exactly as written there.
//! Unicode build: ATL's `SendMessage`, `LV_ITEM`, `LVM_FINDITEM` etc. are the `W` forms.
use crate::prelude::*;

impl CListView {
    #[inline]
    pub unsafe fn SendMessage(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self._base.m_hWnd, msg, wParam, lParam)
    }
    /// ATL `CWindow::Attach`.
    #[inline]
    pub unsafe fn Attach(&mut self, hWndNew: HWND) {
        self._base.m_hWnd = hWndNew;
    }
    /// ATL `CWindow::GetWindowLongPtr`.
    #[inline]
    pub unsafe fn GetWindowLongPtr(&self, nIndex: c_int) -> LONG_PTR {
        GetWindowLongPtr(self._base.m_hWnd, nIndex)
    }
    /// `int InsertColumn(int iCol, LPWSTR pszText, int fmt, int width = -1, int iSubItem = -1,
    /// int iImage = -1, int iOrder = -1)`: callers here pass the first four, so the three
    /// defaulted (-1) members add no mask bits and are elided.
    pub unsafe fn InsertColumn(&self, iCol: c_int, pszText: LPWSTR, fmt: c_int, width: c_int) -> c_int {
        let mut column: LVCOLUMNW = core::mem::zeroed();
        column.mask = LVCF_TEXT | LVCF_FMT;
        column.pszText = pszText;
        column.fmt = fmt;
        if width != -1 {
            column.mask |= LVCF_WIDTH;
            column.cx = width;
        }
        self.SendMessage(LVM_INSERTCOLUMNW, iCol as WPARAM, core::ptr::addr_of_mut!(column) as LPARAM) as c_int
    }
    pub unsafe fn InsertItem(&self, pitem: *const LVITEMW) -> c_int {
        self.SendMessage(LVM_INSERTITEMW, 0, pitem as LPARAM) as c_int
    }
    pub unsafe fn GetItem(&self, pitem: *mut LVITEMW) -> BOOL {
        self.SendMessage(LVM_GETITEMW, 0, pitem as LPARAM) as BOOL
    }
    /// `BOOL FindItem(int iStart, const LV_FINDINFO *)`: the header returns the message result
    /// (an item index) through a BOOL cast; kept as the header writes it.
    pub unsafe fn FindItem(&self, iStart: c_int, plvfi: *const LVFINDINFOW) -> BOOL {
        self.SendMessage(LVM_FINDITEMW, iStart as WPARAM, plvfi as LPARAM) as BOOL
    }
    pub unsafe fn GetItemCount(&self) -> c_int {
        self.SendMessage(LVM_GETITEMCOUNT, 0, 0) as c_int
    }
    pub unsafe fn DeleteAllItems(&self) -> BOOL {
        self.SendMessage(LVM_DELETEALLITEMS, 0, 0) as BOOL
    }
    /// The comparator goes in the LPARAM and the context in the WPARAM.
    pub unsafe fn SortItems(&self, pfnCompare: PFNLVCOMPARE, lParam: PVOID) -> BOOL {
        let f: usize = match pfnCompare { Some(f) => f as usize, None => 0 };
        self.SendMessage(LVM_SORTITEMS, lParam as WPARAM, f as LPARAM) as BOOL
    }
    pub unsafe fn SetItemText(&self, i: c_int, subItem: c_int, text: LPCWSTR) -> BOOL {
        // The header leaves the rest of the stack LVITEMW uninitialised; zeroed is the
        // defined subset of that.
        let mut item: LVITEMW = core::mem::zeroed();
        item.iSubItem = subItem;
        item.pszText = text as LPWSTR;
        self.SendMessage(LVM_SETITEMTEXTW, i as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM) as BOOL
    }
    pub unsafe fn GetItemData(&self, i: c_int) -> DWORD_PTR {
        let mut lvItem: LVITEMW = core::mem::zeroed();
        lvItem.mask = LVIF_PARAM;
        lvItem.iItem = i;
        let ret = self.GetItem(core::ptr::addr_of_mut!(lvItem));
        if ret != 0 { lvItem.lParam as DWORD_PTR } else { 0 }
    }
    pub unsafe fn GetItemText(&self, iItem: c_int, iSubItem: c_int, pszText: LPWSTR, cchTextMax: c_int) {
        let mut itemInfo: LVITEMW = core::mem::zeroed();
        itemInfo.iSubItem = iSubItem;
        itemInfo.pszText = pszText;
        itemInfo.cchTextMax = cchTextMax;
        self.SendMessage(LVM_GETITEMTEXTW, iItem as WPARAM, core::ptr::addr_of_mut!(itemInfo) as LPARAM);
    }
}

/// `TVI_LAST` = `((HTREEITEM)(ULONG_PTR)-0xFFFE)`.
pub const TVI_LAST: HTREEITEM = (-0xFFFEisize) as usize as HTREEITEM;

impl CTreeView {
    #[inline]
    pub unsafe fn SendMessage(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self._base.m_hWnd, msg, wParam, lParam)
    }
    #[inline]
    pub unsafe fn Attach(&mut self, hWndNew: HWND) {
        self._base.m_hWnd = hWndNew;
    }
    pub unsafe fn InsertItem(&self, lpis: *const TVINSERTSTRUCTW) -> HTREEITEM {
        self.SendMessage(TVM_INSERTITEMW, 0, lpis as LPARAM) as HTREEITEM
    }
    pub unsafe fn AddItem(&self, hParent: HTREEITEM, lpText: LPWSTR, Image: INT, SelectedImage: INT, lParam: LPARAM) -> HTREEITEM {
        let mut Insert: TVINSERTSTRUCTW = core::mem::zeroed();
        Insert.__bindgen_anon_1.item.mask = TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
        Insert.hInsertAfter = TVI_LAST;
        Insert.hParent = hParent;
        Insert.__bindgen_anon_1.item.iSelectedImage = SelectedImage;
        Insert.__bindgen_anon_1.item.iImage = Image;
        Insert.__bindgen_anon_1.item.lParam = lParam;
        Insert.__bindgen_anon_1.item.pszText = lpText;
        self.InsertItem(core::ptr::addr_of!(Insert))
    }
    pub unsafe fn Expand(&self, item: HTREEITEM, action: DWORD) -> BOOL {
        self.SendMessage(TVM_EXPAND, action as WPARAM, item as LPARAM) as BOOL
    }
}

/// commctrl.h `ListView_SetExtendedListViewStyleEx(hwnd, dwMask, dw)`.
#[inline]
pub unsafe fn ListView_SetExtendedListViewStyleEx(hwnd: HWND, dwMask: DWORD, dw: DWORD) -> DWORD {
    SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, dwMask as WPARAM, dw as LPARAM) as DWORD
}
