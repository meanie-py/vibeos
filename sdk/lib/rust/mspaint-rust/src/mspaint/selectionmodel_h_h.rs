use crate::prelude::*;
use core::ffi::c_void;



impl SelectionModel {
    pub unsafe fn DrawSelection__1(&mut self, hDCImage: HDC, crBg: COLORREF, bBgTransparent: BOOL) {
        let this_: *mut Self = self;
        return (*this_).DrawSelection__2(hDCImage, crBg, bBgTransparent, &*core::ptr::addr_of!((*this_).m_rc));
    }
    pub unsafe fn DrawSelection__2(&mut self, hDCImage: HDC, crBg: COLORREF, bBgTransparent: BOOL, rc: &CRect) {
        let this_: *mut Self = self;
        return (*this_).DrawSelection__0(hDCImage, crBg, bBgTransparent, rc, (*this_).m_hbmColor);
    }
}
