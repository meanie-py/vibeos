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
    pub unsafe fn InsertColumn__0(&mut self, iCol: i32, pcol: *mut LVCOLUMNW) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_INSERTCOLUMN) as UINT, iCol as _, pcol as LPARAM) as i32;
    }
    pub unsafe fn InsertColumn__1(&mut self, iCol: i32, pszText: LPWSTR, fmt: i32, width: i32, iSubItem: i32, iImage: i32, iOrder: i32) -> i32 {
        let this_: *mut Self = self;
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
        if iImage != -1 {
            column.mask |= LVCF_IMAGE;
            column.iImage = iImage;
        }
        if iOrder != -1 {
            column.mask |= LVCF_ORDER;
            column.iOrder = iOrder;
        }
        return (*this_).InsertColumn__0(iCol, core::ptr::addr_of_mut!(column));
    }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST, iImageList: i32) -> HIMAGELIST {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(LVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM)) as HIMAGELIST;
    }
    pub unsafe fn InsertItem(&mut self, pitem: *const LVITEMW) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_INSERTITEM) as UINT, 0, pitem as LPARAM) as i32;
    }
    pub unsafe fn GetItem(&mut self, pitem: *mut LVITEMW) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((LVM_GETITEM) as UINT, 0, pitem as LPARAM) as BOOL;
    }
    pub unsafe fn DeleteAllItems(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_DELETEALLITEMS, 0, 0) as BOOL;
    }
    pub unsafe fn GetNextItem(&mut self, i: i32, flags: WORD) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(LVM_GETNEXTITEM, i as _, MAKELPARAM!(flags, 0)) as i32;
    }
    pub unsafe fn SetItemText(&mut self, i: i32, subItem: i32, text: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        let mut item: LVITEMW = core::mem::zeroed();
        item.iSubItem = subItem;
        item.pszText = text as LPWSTR;
        return ((*this_).__base0.SendMessageW__0((LVM_SETITEMTEXT) as UINT, (i) as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM)) as BOOL;
    }
    pub unsafe fn GetItemData(&mut self, i: i32) -> DWORD_PTR {
        let this_: *mut Self = self;
        let mut lvItem: LVITEMW = core::mem::zeroed();
        lvItem.mask = LVIF_PARAM;
        lvItem.iItem = i;
        let ret: BOOL = (*this_).GetItem(core::ptr::addr_of_mut!(lvItem));
        return (if ret != 0 { lvItem.lParam } else { 0 }) as DWORD_PTR;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
    }
}

impl CToolbar {
    pub unsafe fn Create(&mut self, hWndParent: HWND, mut dwStyles: DWORD, mut dwExStyles: DWORD) -> HWND {
        let this_: *mut Self = self;
        if dwStyles == 0 {
            dwStyles = WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN;
        }

        if dwExStyles == 0 {
            dwExStyles = WS_EX_TOOLWINDOW;
        }

        (*this_).__base0.m_hWnd = CreateWindowExW(dwExStyles,
                                TOOLBARCLASSNAME,
                                core::ptr::null_mut(),
                                dwStyles,
                                0, 0, 0, 0, hWndParent,
                                core::ptr::null_mut(),
                                _AtlBaseModule.GetModuleInstance(),
                                core::ptr::null_mut());

        if (*this_).__base0.m_hWnd.is_null() {
            return core::ptr::null_mut();
        }

        /* Identify the version we're using */
        (*this_).SetButtonStructSize();

        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn SetButtonStructSize(&mut self) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_BUTTONSTRUCTSIZE as u32, core::mem::size_of::<TBBUTTON>() as WPARAM, 0 as LPARAM)) as DWORD;
    }
    pub unsafe fn CheckButton(&mut self, cmdId: i32, bCheck: BOOL) -> DWORD {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TB_CHECKBUTTON, (cmdId) as WPARAM, MAKELPARAM!(bCheck, 0))) as DWORD;
    }
    pub unsafe fn HideButton(&mut self, cmdId: i32, bHide: BOOL) -> DWORD {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TB_HIDEBUTTON, (cmdId) as WPARAM, MAKELPARAM!(bHide as WORD, 0 as WORD)) as DWORD;
    }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST) -> HIMAGELIST {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TB_SETIMAGELIST, 0, himl as LPARAM) as HIMAGELIST;
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
            STATUSCLASSNAMEW.as_ptr(),
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
        let mut dwStyles = dwStyles;
        if dwStyles == 0 {
            dwStyles = WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_SHOWSELALWAYS;
        }

        (*this_).__base0.m_hWnd = CreateWindowExW(WS_EX_CLIENTEDGE,
            WC_TREEVIEWW.as_ptr(),
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

            ZeroMemory!(core::ptr::addr_of_mut!(Insert) as LPVOID, core::mem::size_of::<TV_INSERTSTRUCT>());

            Insert.__bindgen_anon_1.item.mask = TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
            Insert.hInsertAfter = TVI_LAST;
            Insert.hParent = hParent;
            Insert.__bindgen_anon_1.item.iSelectedImage = SelectedImage;
            Insert.__bindgen_anon_1.item.iImage = Image;
            Insert.__bindgen_anon_1.item.lParam = lParam;
            Insert.__bindgen_anon_1.item.pszText = lpText;

            return (*this_).InsertItem(core::ptr::addr_of!(Insert));
        }
    pub unsafe fn SetImageList(&mut self, himl: HIMAGELIST, iImageList: i32) -> HIMAGELIST {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TVM_SETIMAGELIST, iImageList as _, himl as LPARAM)) as HIMAGELIST;
    }
    pub unsafe fn InsertItem(&mut self, pitem: *const TVINSERTSTRUCTW) -> HTREEITEM {
        let this_: *mut Self = self;
            return (*this_).__base0.SendMessageW__0((TVM_INSERTITEM) as UINT, 0, pitem as LPARAM) as HTREEITEM;
        }
    pub unsafe fn DeleteItem(&mut self, i: HTREEITEM) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TVM_DELETEITEM, 0, i as LPARAM) as BOOL;
    }
    pub unsafe fn GetItem(&mut self, pitem: *mut TVITEMW) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((TVM_GETITEM) as UINT, 0, pitem as LPARAM) as BOOL;
    }
    pub unsafe fn EditLabel(&mut self, i: HTREEITEM) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0((TVM_EDITLABEL) as UINT, 0, i as LPARAM) as HWND;
    }
    pub unsafe fn GetNextItem(&mut self, i: HTREEITEM, flags: WORD) -> HTREEITEM {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TVM_GETNEXTITEM, (flags) as WPARAM, i as LPARAM) as HTREEITEM;
    }
    pub unsafe fn HitTest(&mut self, phtInfo: *mut TVHITTESTINFO) -> HTREEITEM {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TVM_HITTEST, 0, phtInfo as LPARAM)) as HTREEITEM;
    }
    pub unsafe fn GetItemData(&mut self, item: HTREEITEM) -> DWORD_PTR {
        let this_: *mut Self = self;
            let mut lvItem: TVITEMW = core::mem::zeroed();
            lvItem.hItem = item;
            lvItem.mask = TVIF_PARAM;
            let ret: BOOL = (*this_).GetItem(core::ptr::addr_of_mut!(lvItem));
            return (if ret != 0 { lvItem.lParam as DWORD_PTR } else { core::ptr::null_mut::<u8>() as DWORD_PTR });
        }
    pub unsafe fn GetSelection(&mut self) -> HTREEITEM {
        let this_: *mut Self = self;
        return (*this_).GetNextItem(core::ptr::null_mut(), (TVGN_CARET as u16));
    }
    pub unsafe fn Expand(&mut self, item: HTREEITEM, action: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).__base0.SendMessageW__0(TVM_EXPAND, action as WPARAM, item as LPARAM) as BOOL;
    }
    pub unsafe fn SelectItem(&mut self, item: HTREEITEM, action: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).__base0.SendMessageW__0(TVM_SELECTITEM, (action) as WPARAM, item as LPARAM)) as BOOL;
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
