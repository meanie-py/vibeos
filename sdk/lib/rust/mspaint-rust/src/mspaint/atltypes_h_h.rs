use crate::prelude::*;
use core::ffi::c_void;





























impl CPoint {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.y = 0;
            (*this_).__base0.x = (*this_).__base0.y;
        }
    }
    pub unsafe fn __ctor_1(&mut self, initX: i32, initY: i32) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.x = initX;
            (*this_).__base0.y = initY;
        }
    }
    pub unsafe fn __ctor_2(&mut self, initPt: POINT) {
        let this_: *mut Self = self;
        {
            *(this_ as *mut POINT) = initPt;
        }
    }
    pub unsafe fn __ctor_3(&mut self, initSize: SIZE) {
        let this_: *mut Self = self;
        {
            *(this_ as *mut SIZE) = initSize;
        }
    }
    pub unsafe fn __ctor_4(&mut self, dwPoint: LPARAM) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.x = LOWORD!(dwPoint) as _;
            (*this_).__base0.y = HIWORD!(dwPoint) as _;
        }
    }
}

impl CSize {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.cy = 0;
            (*this_).__base0.cx = 0;
        }
    }
    pub unsafe fn __ctor_1(&mut self, initCX: i32, initCY: i32) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.cx = initCX;
            (*this_).__base0.cy = initCY;
        }
    }
    pub unsafe fn __ctor_2(&mut self, initSize: SIZE) {
        let this_: *mut Self = self;
        {
            *(this_ as *mut SIZE) = initSize;
        }
    }
    pub unsafe fn __ctor_3(&mut self, initPt: POINT) {
        let this_: *mut Self = self;
        {
            *(this_ as *mut POINT) = initPt;
        }
    }
    pub unsafe fn __ctor_4(&mut self, dwSize: DWORD) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.cx = LOWORD!(dwSize) as _;
            (*this_).__base0.cy = HIWORD!(dwSize) as _;
        }
    }
}

impl CRect {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            let __v = 0;
            (*this_).__base0.bottom = __v;
            (*this_).__base0.right = __v;
            (*this_).__base0.top = __v;
            (*this_).__base0.left = __v;
        }
    }
    pub unsafe fn __ctor_1(&mut self, l: i32, t: i32, r: i32, b: i32) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = l;
            (*this_).__base0.top = t;
            (*this_).__base0.right = r;
            (*this_).__base0.bottom = b;
        }
    }
    pub unsafe fn __ctor_2(&mut self, srcRect: *const RECT) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = (*srcRect).left;
            (*this_).__base0.top = (*srcRect).top;
            (*this_).__base0.right = (*srcRect).right;
            (*this_).__base0.bottom = (*srcRect).bottom;
        }
    }
    pub unsafe fn __ctor_3(&mut self, lpSrcRect: LPCRECT) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = (*lpSrcRect).left;
            (*this_).__base0.top = (*lpSrcRect).top;
            (*this_).__base0.right = (*lpSrcRect).right;
            (*this_).__base0.bottom = (*lpSrcRect).bottom;
        }
    }
    pub unsafe fn __ctor_4(&mut self, point: POINT, size: SIZE) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = point.x;
            (*this_).__base0.top = point.y;
            (*this_).__base0.right = point.x + size.cx;
            (*this_).__base0.bottom = point.y + size.cy;
        }
    }
    pub unsafe fn __ctor_5(&mut self, topLeft: POINT, bottomRight: POINT) {
        let this_: *mut Self = self;
        {
            (*this_).__base0.left = topLeft.x;
            (*this_).__base0.top = topLeft.y;
            (*this_).__base0.right = bottomRight.x;
            (*this_).__base0.bottom = bottomRight.y;
        }
    }
    pub unsafe fn EqualRect(&mut self, lpRect: LPCRECT) -> i32 {
        let this_: *mut Self = self;
        return EqualRect(this_ as LPCRECT, lpRect);
    }
    pub unsafe fn Height(&mut self) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.bottom - (*this_).__base0.top;
    }
    pub unsafe fn InflateRect__0(&mut self, x: i32, y: i32) {
        let this_: *mut Self = self;
        InflateRect(this_ as *mut RECT, x, y);
    }
    pub unsafe fn IntersectRect(&mut self, lpRect1: LPCRECT, lpRect2: LPCRECT) -> i32 {
        let this_: *mut Self = self;
        return IntersectRect(this_ as *mut RECT, lpRect1, lpRect2);
    }
    pub unsafe fn IsRectEmpty(&mut self) -> i32 {
        let this_: *mut Self = self;
        return IsRectEmpty(this_);
    }
    pub unsafe fn NormalizeRect(&mut self) {
        let this_: *mut Self = self;
        if (*this_).__base0.left > (*this_).__base0.right {
            let tmp: LONG = (*this_).__base0.left;
            (*this_).__base0.left = (*this_).__base0.right;
            (*this_).__base0.right = tmp;
        }
        if (*this_).__base0.top > (*this_).__base0.bottom {
            let tmp: LONG = (*this_).__base0.top;
            (*this_).__base0.top = (*this_).__base0.bottom;
            (*this_).__base0.bottom = tmp;
        }
    }
    pub unsafe fn OffsetRect__0(&mut self, x: i32, y: i32) {
        let this_: *mut Self = self;
        OffsetRect((this_).cast(), x, y);
    }
    pub unsafe fn PtInRect(&mut self, point: POINT) -> BOOL {
        let this_: *mut Self = self;
        return PtInRect(this_ as *mut RECT, point);
    }
    pub unsafe fn SetRect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let this_: *mut Self = self;
        (*this_).__base0.left = x1;
        (*this_).__base0.top = y1;
        (*this_).__base0.right = x2;
        (*this_).__base0.bottom = y2;
    }
    pub unsafe fn SetRectEmpty(&mut self) {
        let this_: *mut Self = self;
        ZeroMemory!(this_ as *mut core::ffi::c_void, core::mem::size_of::<CRect>());
    }
    pub unsafe fn UnionRect(&mut self, lpRect1: LPCRECT, lpRect2: LPCRECT) -> BOOL {
        let this_: *mut Self = self;
        return UnionRect((this_).cast(), lpRect1, lpRect2);
    }
    pub unsafe fn Width(&mut self) -> i32 {
        let this_: *mut Self = self;
        return (*this_).__base0.right - (*this_).__base0.left;
    }
}
