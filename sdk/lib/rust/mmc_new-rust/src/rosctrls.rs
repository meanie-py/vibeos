//! The slice of `sdk/include/reactos/ui/rosctrls.h` (CListView, CTreeView, CStatusBar,
//! CToolbar) and the inherited ATL `CWindow` members that mmc_new calls on them. These are
//! header-only C++ methods, so the bindgen island carries only the `{ _base: ATL_CWindow }`
//! structs; each method below is the header's body with its message id and WPARAM/LPARAM
//! packing kept exactly as written there.
//! Unicode build: ATL's `SendMessage`, `LV_ITEM`, `LVM_FINDITEM` etc. are the `W` forms.
use crate::prelude::*;

/// ATL `CWindow` members the control wrappers inherit. The cppapp-rendered `CWindow` (a
/// separate struct with the same one-field layout) carries the ones the TUs reached; the
/// control wrappers deref to the island's `ATL_CWindow`, which has no methods, so the ones
/// mmc_new calls on a control are written here with the header's bodies.
impl ATL_CWindow {
    #[inline]
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self.m_hWnd, message, wParam, lParam)
    }
    #[inline]
    pub unsafe fn MoveWindow(&mut self, x: c_int, y: c_int, nWidth: c_int, nHeight: c_int, bRepaint: BOOL) -> BOOL {
        MoveWindow(self.m_hWnd, x, y, nWidth, nHeight, bRepaint)
    }
    #[inline]
    pub unsafe fn GetClientRect(&mut self, lpRect: LPRECT) -> BOOL {
        GetClientRect(self.m_hWnd, lpRect)
    }
    #[inline]
    pub unsafe fn ScreenToClient(&mut self, lpPoint: LPPOINT) -> BOOL {
        ScreenToClient(self.m_hWnd, lpPoint)
    }
    /// `BOOL ClientToScreen(LPPOINT)`; the `__0` is the name the translation gave the
    /// overload that takes a POINT pointer.
    #[inline]
    pub unsafe fn ClientToScreen__0(&mut self, lpPoint: LPPOINT) -> BOOL {
        ClientToScreen(self.m_hWnd, lpPoint)
    }
    #[inline]
    pub unsafe fn Attach(&mut self, hWndNew: HWND) {
        self.m_hWnd = hWndNew;
    }
    #[inline]
    pub unsafe fn SetWindowPos(&mut self, hWndInsertAfter: HWND, x: c_int, y: c_int, cx: c_int, cy: c_int, nFlags: UINT) -> BOOL {
        SetWindowPos(self.m_hWnd, hWndInsertAfter, x, y, cx, cy, nFlags)
    }
}

impl CListView {
    #[inline]
    pub unsafe fn SendMessage(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self._base.m_hWnd, msg, wParam, lParam)
    }
    /// `HWND Create(HWND hWndParent, _U_RECT rect, LPCTSTR szWindowName = NULL, DWORD dwStyle = 0,
    /// DWORD dwExStyle = 0, _U_MENUorID MenuOrID = 0U, LPVOID lpCreateParam = NULL)`: the
    /// header dereferences `rect.m_lpRect` unconditionally, so `rect` must not be null.
    pub unsafe fn Create(&mut self, hWndParent: HWND, rect: *mut RECT, szWindowName: LPCWSTR, dwStyle: DWORD, dwExStyle: DWORD, MenuOrID: UINT) -> HWND {
        self._base.m_hWnd = CreateWindowExW(
            dwExStyle,
            WC_LISTVIEWW.as_ptr(),
            szWindowName,
            dwStyle,
            (*rect).left,
            (*rect).top,
            (*rect).right - (*rect).left,
            (*rect).bottom - (*rect).top,
            hWndParent,
            MenuOrID as usize as HMENU,
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut(),
        );
        self._base.m_hWnd
    }
    pub unsafe fn SetImageList(&self, himl: HIMAGELIST, iImageList: c_int) -> HIMAGELIST {
        self.SendMessage(LVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST
    }
    /// `int InsertColumn(int iCol, LV_COLUMN *pcol)`.
    pub unsafe fn InsertColumn__0(&self, iCol: c_int, pcol: *mut LVCOLUMNW) -> c_int {
        self.SendMessage(LVM_INSERTCOLUMNW, iCol as WPARAM, pcol as LPARAM) as c_int
    }
    /// `int InsertColumn(int iCol, LPWSTR pszText, int fmt, int width = -1, int iSubItem = -1,
    /// int iImage = -1, int iOrder = -1)`, every default spelled out.
    pub unsafe fn InsertColumn(&self, iCol: c_int, pszText: LPWSTR, fmt: c_int, width: c_int, iSubItem: c_int) -> c_int {
        let mut column: LVCOLUMNW = core::mem::zeroed();
        column.mask = LVCF_TEXT | LVCF_FMT;
        column.pszText = pszText;
        column.fmt = fmt;
        if width != -1 {
            column.mask |= LVCF_WIDTH;
            column.cx = width;
        }
        if iSubItem != -1 {
            column.mask |= LVCF_SUBITEM;
            column.iSubItem = iSubItem;
        }
        self.InsertColumn__0(iCol, core::ptr::addr_of_mut!(column))
    }
    pub unsafe fn InsertItem(&self, pitem: *const LVITEMW) -> c_int {
        self.SendMessage(LVM_INSERTITEMW, 0, pitem as LPARAM) as c_int
    }
    pub unsafe fn GetItem(&self, pitem: *mut LVITEMW) -> BOOL {
        self.SendMessage(LVM_GETITEMW, 0, pitem as LPARAM) as BOOL
    }
    pub unsafe fn DeleteAllItems(&self) -> BOOL {
        self.SendMessage(LVM_DELETEALLITEMS, 0, 0) as BOOL
    }
    /// `int GetNextItem(int i, WORD flags)`: `MAKELPARAM(flags, 0)`.
    pub unsafe fn GetNextItem(&self, i: c_int, flags: WORD) -> c_int {
        self.SendMessage(LVM_GETNEXTITEM, i as WPARAM, flags as LPARAM) as c_int
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
}

impl CStatusBar {
    /// `VOID SetText(LPCWSTR lpszText)`: SB_SETTEXT with SBT_NOBORDERS in the WPARAM.
    pub unsafe fn SetText(&self, lpszText: LPCWSTR) {
        SendMessageW(self._base.m_hWnd, SB_SETTEXTW, SBT_NOBORDERS as WPARAM, lpszText as LPARAM);
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND, hMenu: HMENU) -> HWND {
        self._base.m_hWnd = CreateWindowExW(
            0,
            STATUSCLASSNAMEW.as_ptr(),
            core::ptr::null(),
            WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
            0,
            0,
            0,
            0,
            hwndParent,
            hMenu,
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut(),
        );
        self._base.m_hWnd
    }
}

impl CToolbar {
    #[inline]
    pub unsafe fn SendMessageW(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self._base.m_hWnd, msg, wParam, lParam)
    }
    /// `HWND Create(HWND hWndParent, DWORD dwStyles = 0, DWORD dwExStyles = 0)`.
    pub unsafe fn Create(&mut self, hWndParent: HWND, mut dwStyles: DWORD, mut dwExStyles: DWORD) -> HWND {
        if dwStyles == 0 {
            dwStyles = WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN;
        }
        if dwExStyles == 0 {
            dwExStyles = WS_EX_TOOLWINDOW;
        }
        self._base.m_hWnd = CreateWindowExW(
            dwExStyles,
            TOOLBARCLASSNAMEW.as_ptr(),
            core::ptr::null(),
            dwStyles,
            0,
            0,
            0,
            0,
            hWndParent,
            core::ptr::null_mut(),
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut(),
        );
        if self._base.m_hWnd.is_null() {
            return core::ptr::null_mut();
        }
        /* Identify the version we're using */
        self.SetButtonStructSize();
        self._base.m_hWnd
    }
    pub unsafe fn SetButtonStructSize(&self) -> DWORD {
        self.SendMessageW(TB_BUTTONSTRUCTSIZE, core::mem::size_of::<TBBUTTON>() as WPARAM, 0) as DWORD
    }
    pub unsafe fn SetImageList(&self, himl: HIMAGELIST) -> HIMAGELIST {
        self.SendMessageW(TB_SETIMAGELIST, 0, himl as LPARAM) as HIMAGELIST
    }
    pub unsafe fn GetButton(&self, index: c_int, btn: *mut TBBUTTON) -> DWORD {
        self.SendMessageW(TB_GETBUTTON, index as WPARAM, btn as LPARAM) as DWORD
    }
    pub unsafe fn InsertButton(&self, insertAt: c_int, btn: *mut TBBUTTON) -> DWORD {
        self.SendMessageW(TB_INSERTBUTTONW, insertAt as WPARAM, btn as LPARAM) as DWORD
    }
    pub unsafe fn DeleteButton(&self, index: c_int) -> DWORD {
        self.SendMessageW(TB_DELETEBUTTON, index as WPARAM, 0) as DWORD
    }
}

impl CTreeView {
    #[inline]
    pub unsafe fn SendMessage(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        SendMessageW(self._base.m_hWnd, msg, wParam, lParam)
    }
    /// `HWND Create(HWND hwndParent, DWORD dwStyles = 0)`.
    pub unsafe fn Create(&mut self, hwndParent: HWND, mut dwStyles: DWORD) -> HWND {
        if dwStyles == 0 {
            dwStyles = WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_SHOWSELALWAYS;
        }
        self._base.m_hWnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            WC_TREEVIEWW.as_ptr(),
            [0u16].as_ptr(),
            dwStyles,
            0,
            28,
            200,
            350,
            hwndParent,
            core::ptr::null_mut(),
            _AtlBaseModule.GetModuleInstance(),
            core::ptr::null_mut(),
        );
        self._base.m_hWnd
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
    pub unsafe fn SetImageList(&self, himl: HIMAGELIST, iImageList: c_int) -> HIMAGELIST {
        self.SendMessage(TVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST
    }
    pub unsafe fn DeleteItem(&self, i: HTREEITEM) -> BOOL {
        self.SendMessage(TVM_DELETEITEM, 0, i as LPARAM) as BOOL
    }
    pub unsafe fn GetItem(&self, pitem: *mut TVITEMW) -> BOOL {
        self.SendMessage(TVM_GETITEMW, 0, pitem as LPARAM) as BOOL
    }
    pub unsafe fn EditLabel(&self, i: HTREEITEM) -> HWND {
        self.SendMessage(TVM_EDITLABELW, 0, i as LPARAM) as HWND
    }
    /// `HTREEITEM GetNextItem(HTREEITEM i, WORD flags)`: the flag is the WPARAM.
    pub unsafe fn GetNextItem(&self, i: HTREEITEM, flags: WORD) -> HTREEITEM {
        self.SendMessage(TVM_GETNEXTITEM, flags as WPARAM, i as LPARAM) as HTREEITEM
    }
    pub unsafe fn HitTest(&self, phtInfo: *mut TVHITTESTINFO) -> HTREEITEM {
        self.SendMessage(TVM_HITTEST, 0, phtInfo as LPARAM) as HTREEITEM
    }
    pub unsafe fn GetItemData(&self, item: HTREEITEM) -> DWORD_PTR {
        let mut lvItem: TVITEMW = core::mem::zeroed();
        lvItem.hItem = item;
        lvItem.mask = TVIF_PARAM;
        let ret = self.GetItem(core::ptr::addr_of_mut!(lvItem));
        if ret != 0 { lvItem.lParam as DWORD_PTR } else { 0 }
    }
    pub unsafe fn GetSelection(&self) -> HTREEITEM {
        self.GetNextItem(core::ptr::null_mut(), TVGN_CARET as WORD)
    }
    pub unsafe fn Expand(&self, item: HTREEITEM, action: DWORD) -> BOOL {
        self.SendMessage(TVM_EXPAND, action as WPARAM, item as LPARAM) as BOOL
    }
    /// `BOOL SelectItem(HTREEITEM item, DWORD action = TVGN_CARET)`: Rust has no default
    /// arguments, so callers of the one-argument form pass `TVGN_CARET`.
    pub unsafe fn SelectItem(&self, item: HTREEITEM, action: DWORD) -> BOOL {
        self.SendMessage(TVM_SELECTITEM, action as WPARAM, item as LPARAM) as BOOL
    }
}
