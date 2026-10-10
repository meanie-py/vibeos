use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FtfdQueryGlyphAttrs(pfo: *mut FONTOBJ, iMode: ULONG) -> PFD_GLYPHATTR {
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn FtfdQueryFontData(dhpdev: DHPDEV, pfo: *mut FONTOBJ, iMode: ULONG, hg: HGLYPH, pgd: *mut GLYPHDATA, pv: PVOID, cjSize: ULONG) -> LONG {
    (FD_ERROR) as i32
}
