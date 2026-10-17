use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_26A6FBF4F41B: [WCHAR; 17] = [0x0043, 0x0069, 0x0063, 0x0065, 0x0072, 0x006F, 0x0055, 0x0049, 0x0057, 0x006E, 0x0064, 0x0046, 0x0072, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_64D45D11D517: [WCHAR; 19] = [0x004D, 0x0053, 0x0049, 0x004D, 0x0045, 0x005F, 0x0050, 0x006F, 0x0070, 0x0075, 0x0070, 0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0000];

use core::ffi::c_void;

impl CUIFSystemInfo {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CUIFTheme {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}

impl CUIFObjectArray {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn Add(&mut self, pObject: *mut CUIFObject) -> BOOL {
        let this_: *mut Self = self;
        if pObject.is_null() || (*this_).__base0.Find(pObject) >= 0 {
            return (FALSE) as BOOL;
        }

        let ppNew: *mut *mut CUIFObject = (*this_).__base0.Append(1);
        if ppNew.is_null() {
            return (FALSE) as BOOL;
        }

        *ppNew = pObject;
        return (TRUE) as BOOL;
    }
    pub unsafe fn Remove(&mut self, pObject: *mut CUIFObject) -> BOOL {
        let this_: *mut Self = self;
            if pObject.is_null() {
                return (FALSE) as BOOL;
            }

            let iItem: isize = (*this_).__base0.Find(pObject);
            if iItem < 0 {
                return (FALSE) as BOOL;
            }

            if (iItem as size_t) + 1 < (*this_).__base0.size() {
                MoveMemory!(
                    (*this_).__base0.data().offset(iItem) as *mut core::ffi::c_void,
                    (*this_).__base0.data().offset(iItem + 1) as *mut core::ffi::c_void,
                    (((*this_).__base0.size() as isize - (iItem + 1)) as usize)
                        * core::mem::size_of::<*mut CUIFObject>(),
                );
            }

            (*this_).m_cItems -= 1;
            return (TRUE) as BOOL;
        }
    pub unsafe fn GetLast(&mut self) -> *mut CUIFObject {
        let this_: *mut Self = self;
        if (*this_).__base0.empty() {
            return core::ptr::null_mut();
        }
        return (&mut (*this_).__base0)[(*this_).__base0.size() - 1];
    }
}

impl CUIFObject {
    pub unsafe fn Initialize__body(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (TRUE as i32)
    }
    pub unsafe fn OnTimer__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnLButtonDown__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMButtonDown__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnRButtonDown__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnLButtonUp__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMButtonUp__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnRButtonUp__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMouseMove__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMouseIn__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMouseOut__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn GetToolTip__body(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        (*this_).m_pszToolTip
    }
    pub unsafe fn OnShowToolTip__body(&mut self) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnHideToolTip__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnPaintTheme__body(&mut self, hDC: HDC,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE as i32)
    }
    pub unsafe fn OnPaintNoTheme__body(&mut self, hDC: HDC) {
        let this_: *mut Self = self;}
}

impl CUIFColorTable {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn DoneColor__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn Update(&mut self) {
        let this_: *mut Self = self;
        (*this_).DoneColor();
        (*this_).DoneBrush();
        (*this_).InitColor();
        (*this_).InitBrush();
    }
}

impl CUIFColorTableSys {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn GetColor(&mut self, iColor: INT) -> COLORREF {
        let this_: *mut Self = self;
        return (*this_).m_rgbColors[iColor as usize];
    }
}

impl CUIFColorTableOff10 {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {}
    }
    pub unsafe fn GetColor(&mut self, iColor: INT) -> COLORREF {
        let this_: *mut Self = self;
        return (*this_).m_rgbColors[iColor as usize];
    }
}

impl CUIFSolidBrush {
    pub unsafe fn op_to_HBRUSH(&mut self) -> HBRUSH {
        let this_: *mut Self = self;
        return (*this_).m_hBrush;
    }
    pub unsafe fn __ctor_0(&mut self, rgbColor: COLORREF) {
        let this_: *mut Self = self;
        {
            (*this_).m_hBrush = CreateSolidBrush(rgbColor);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hBrush.is_null() {
            DeleteObject(((*this_).m_hBrush).cast());
            (*this_).m_hBrush = core::ptr::null_mut();
        }
    }
}

impl CUIFIcon {
    pub unsafe fn op_assign(&mut self, hIcon: HICON) -> &mut CUIFIcon {
        let this_: *mut Self = self;
        (*this_).m_hIcon = hIcon;
        if !(*this_).m_hImageList.is_null() {
            ImageList_Destroy((*this_).m_hImageList);
            (*this_).m_hImageList = core::ptr::null_mut();
        }
        &mut *this_
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CUIFBitmapDC {
    pub unsafe fn op_to_HDC(&mut self) -> HDC {
        let this_: *mut Self = self;
        return (*this_).m_hDC;
    }
    pub unsafe fn DetachBitmap(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        let hOldBitmap: HBITMAP = (*this_).m_hBitmap;
        (*this_).m_hBitmap = core::ptr::null_mut();
        return hOldBitmap;
    }
}

impl CUIFScheme {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_bMirroring = (FALSE as i32);
        {}
    }
}

impl CUIFSchemeDef {
    pub unsafe fn __ctor_0(&mut self, dwType: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_dwType = dwType;
        {}
    }
    pub unsafe fn CxWndBorder__body(&mut self) -> INT {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn CyWndBorder__body(&mut self) -> INT {
        let this_: *mut Self = self;
        1
    }
}

impl CUIFWindow {
    pub unsafe fn op_to_HWND(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).m_hWnd;
    }
    pub unsafe fn GetClassNameW__body(&mut self) -> LPCTSTR {
        let this_: *mut Self = self;
        return CPP_COM_WSTR_26A6FBF4F41B.as_ptr();
    }
    pub unsafe fn GetWndTitle__body(&mut self) -> LPCTSTR {
        let this_: *mut Self = self;
        return CPP_COM_WSTR_26A6FBF4F41B.as_ptr();
    }
    pub unsafe fn OnMouseOutFromWindow__body(&mut self, x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnCreate__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnDestroy__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnNCDestroy__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnSetFocus__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnKillFocus__body(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;}
    pub unsafe fn OnNotify__body(&mut self, hWnd: HWND,
        wParam: WPARAM,
        lParam: LPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnTimer__body(&mut self, wParam: WPARAM) {
        let this_: *mut Self = self;}
    pub unsafe fn OnSysColorChange__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnEndSession__body(&mut self, hWnd: HWND,
        wParam: WPARAM,
        lParam: LPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnKeyDown__body(&mut self, hWnd: HWND,
        wParam: WPARAM,
        lParam: LPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnKeyUp__body(&mut self, arg0: HWND,
        wParam: WPARAM,
        lParam: LPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnUser__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnActivate__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnWindowPosChanged__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnWindowPosChanging__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnNotifyFormat__body(&mut self, hWnd: HWND,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnShowWindow__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnDisplayChange__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT {
        let this_: *mut Self = self;
        0
    }
    pub unsafe fn OnGetObject__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn OnEraseBkGnd__body(&mut self, hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE as i32)
    }
    pub unsafe fn ModalMouseNotify__body(&mut self, uMsg: UINT,
        x: LONG,
        y: LONG,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnAnimationStart__body(&mut self) {
        let this_: *mut Self = self;}
}

impl CUIFMenuItem {
    pub unsafe fn Check(&mut self, bChecked: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bMenuItemChecked = bChecked;
    }
    pub unsafe fn Gray(&mut self, bGrayed: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bMenuItemGrayed = bGrayed;
    }
    pub unsafe fn SetBitmap(&mut self, hbmColor: HBITMAP) {
        let this_: *mut Self = self;
        (*this_).m_hbmColor = hbmColor;
    }
    pub unsafe fn SetSub(&mut self, pSubMenu: *mut CUIFMenu,
    ) {
        let this_: *mut Self = self;
        (*this_).m_pSubMenu = pSubMenu;
    }
    pub unsafe fn OnUnknownMethod__body(&mut self) {
        let this_: *mut Self = self;}
}

impl CUIFToolbarButton {
    pub unsafe fn OnUnknownMouse0__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnLeftClick__body(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn OnRightClick__body(&mut self) {
        let this_: *mut Self = self;}
}

impl CUIFBalloonWindow {
    pub unsafe fn GetClassNameW__body(&mut self) -> LPCTSTR {
        let this_: *mut Self = self;
        return CPP_COM_WSTR_64D45D11D517.as_ptr();
    }
    pub unsafe fn GetWndTitle__body(&mut self) -> LPCTSTR {
        let this_: *mut Self = self;
        return CPP_COM_WSTR_64D45D11D517.as_ptr();
    }
}
