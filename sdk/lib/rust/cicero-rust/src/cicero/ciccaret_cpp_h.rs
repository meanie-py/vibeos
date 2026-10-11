use crate::prelude::*;
use core::ffi::c_void;











impl CicCaret {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        {
            (*this_).m_bCaretBlinking = (FALSE) as i32;
            (*this_).m_bCaretVisible = (FALSE) as i32;
            (*this_).m_uCaretBlinkTimerID = 0;
            (*this_).m_pt.y = 0;
            (*this_).m_pt.x = 0;
            (*this_).m_size.cy = 0;
            (*this_).m_size.cx = 0;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).HideCaret();
        KillTimer((*this_).m_hWnd, ((*this_).m_uCaretBlinkTimerID) as UINT_PTR);
        (*this_).m_uCaretBlinkTimerID = 0;
    }
    pub unsafe fn CreateCaret(&mut self, hWnd: HWND, size: SIZE) {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWnd;
        (*this_).m_size = size;
        if IsWindow(hWnd) != 0 {
            (*this_).m_uCaretBlinkTimerID = (SetTimer((*this_).m_hWnd, (TIMER_ID) as UINT_PTR, GetCaretBlinkTime(), None)) as u32;
        }
    }
    pub unsafe fn DestroyCaret(&mut self) {
        let this_: *mut Self = self;
        (*this_).HideCaret();
        KillTimer((*this_).m_hWnd, ((*this_).m_uCaretBlinkTimerID) as UINT_PTR);
        (*this_).m_uCaretBlinkTimerID = 0;
    }
    pub unsafe fn HideCaret(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bCaretVisible != 0 {
            (*this_).m_bCaretVisible = (FALSE) as i32;
            (*this_).InvertCaret();
        }
        (*this_).m_bCaretBlinking = (FALSE) as i32;
    }
    pub unsafe fn InvertCaret(&mut self) {
        let this_: *mut Self = self;
        let hDC: HDC = GetDC((*this_).m_hWnd);
        PatBlt(hDC, (*this_).m_pt.x, (*this_).m_pt.y, (*this_).m_size.cx, (*this_).m_size.cy, DSTINVERT);
        ReleaseDC((*this_).m_hWnd, hDC);
    }
    pub unsafe fn OnTimer(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bCaretBlinking != 0 {
            (*this_).m_bCaretVisible = if (*this_).m_bCaretVisible != 0 { 0 } else { 1 };
            (*this_).InvertCaret();
        }
    }
    pub unsafe fn SetCaretPos(&mut self, pt: POINT) {
        let this_: *mut Self = self;
        let bCaretVisible: BOOL = (*this_).m_bCaretVisible;
        if bCaretVisible != 0 {
            (*this_).InvertCaret();
        }
    
        (*this_).m_pt = pt;
    
        if bCaretVisible != 0 {
            (*this_).InvertCaret();
        }
    }
    pub unsafe fn SetBlinking(&mut self, bBlinking: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bCaretBlinking = bBlinking;
    }
}
