use crate::prelude::*;
use core::ffi::c_void;










impl CSplitter {
    pub unsafe fn Init(&mut self, parent: CWindow, left: CWindow, right: CWindow) {
        let this_: *mut Self = self;
            (*this_).m_Parent = parent;
            (*this_).m_Left = left;
            (*this_).m_Right = right;
        }
    pub unsafe fn ResizeWnd(&mut self, width: i32, height: i32) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).m_Parent.GetClientRect(core::ptr::addr_of_mut!(rc));
    
        let mut hdwp: HDWP = BeginDeferWindowPos(2);
    
        if hdwp as usize != 0 {
            hdwp = (*this_).m_Left.DeferWindowPos(
                hdwp,
                core::ptr::null_mut(),
                0,
                0,
                (*this_).m_splitterPos - (*this_).m_splitterSize / 2,
                height,
                (SWP_NOZORDER | SWP_NOACTIVATE),
            );
        }
        if hdwp as usize != 0 {
            hdwp = (*this_).m_Right.DeferWindowPos(
                hdwp,
                core::ptr::null_mut(),
                (*this_).m_splitterPos + (*this_).m_splitterSize / 2,
                0,
                width - ((*this_).m_splitterPos + (*this_).m_splitterSize / 2),
                height,
                (SWP_NOZORDER | SWP_NOACTIVATE),
            );
        }
        if hdwp as usize != 0 {
            EndDeferWindowPos(hdwp);
        }
    }
    pub unsafe fn DragSplitter(&mut self, x: INT) {
        let this_: *mut Self = self;
        (*this_).m_splitterPos = max!(x, 120);
        let mut rc: RECT = core::mem::zeroed();
        (*this_).m_Parent.GetClientRect(core::ptr::addr_of_mut!(rc));
        (*this_).ResizeWnd(rc.right - rc.left, rc.bottom - rc.top);
    }
    pub unsafe fn OnSize(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            (*this_).ResizeWnd(LOWORD!(lParam) as i32, HIWORD!(lParam) as i32);
            return (TRUE) as LRESULT;
        }
    pub unsafe fn OnSetCursor(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            if LOWORD!(lParam) == (HTCLIENT as u16)
            {
                let mut pt: POINT = core::mem::zeroed();
                GetCursorPos(core::ptr::addr_of_mut!(pt));
                (*this_).m_Parent.ScreenToClient(core::ptr::addr_of_mut!(pt));
                if pt.x >= (*this_).m_splitterPos - (*this_).m_splitterSize / 2 && pt.x <= (*this_).m_splitterPos + (*this_).m_splitterSize / 2
                {
                    SetCursor(LoadCursor(core::ptr::null_mut(), IDC_SIZEWE));
                    *bHandled = (TRUE as i32);
                    return (TRUE) as LRESULT;
                }
            }
            *bHandled = (FALSE as i32);
            return (FALSE) as LRESULT;
        }
    pub unsafe fn OnLButtonDown(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let x: INT = GET_X_LPARAM!(lParam);
    
        if x >= (*this_).m_splitterPos - (*this_).m_splitterSize / 2 && x <= (*this_).m_splitterPos + (*this_).m_splitterSize / 2 {
            (*this_).m_Parent.SetCapture();
    
            *bHandled = (TRUE as i32);
            return (TRUE) as LRESULT;
        }
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnMouseMove(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() == (*this_).m_Parent.m_hWnd {
            (*this_).DragSplitter(GET_X_LPARAM!(lParam));
        }
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnLButtonUp(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() == (*this_).m_Parent.m_hWnd {
            (*this_).DragSplitter(GET_X_LPARAM!(lParam));
            ReleaseCapture();
        }
        return (TRUE) as LRESULT;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_splitterPos = 280;
        (*this_).m_splitterSize = 4;
    }
}
