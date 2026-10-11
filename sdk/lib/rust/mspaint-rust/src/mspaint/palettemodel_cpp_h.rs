use crate::prelude::*;
use core::ffi::c_void;















impl PaletteModel {
    pub unsafe fn NotifyColorChanged(&mut self) {
        let this_: *mut Self = self;
        if paletteWindow.__base0.IsWindow() != 0 {
            paletteWindow.__base0.SendMessageW__0(WM_PALETTEMODELCOLORCHANGED, 0, 0);
        }
        if canvasWindow.__base0.IsWindow() != 0 {
            canvasWindow.__base0.SendMessageW__0(WM_PALETTEMODELCOLORCHANGED, 0, 0);
        }
        if textEditWindow.__base0.IsWindow() != 0 {
            textEditWindow.__base0.SendMessageW__0(WM_PALETTEMODELCOLORCHANGED, 0, 0);
        }
    }
    pub unsafe fn NotifyPaletteChanged(&mut self) {
        let this_: *mut Self = self;
        if paletteWindow.__base0.IsWindow() != 0 {
            paletteWindow.__base0.Invalidate(FALSE as BOOL);
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).m_fgColor = RGB!(0, 0, 0);
            (*this_).m_bgColor = RGB!(255, 255, 255);
            (*this_).SelectPalette(PAL_MODERN);
        }
    }
    pub unsafe fn SelectedPalette(&mut self) -> PAL_TYPE {
        let this_: *mut Self = self;
        (*this_).m_nSelectedPalette
    }
    pub unsafe fn SelectPalette(&mut self, nPalette: PAL_TYPE) {
        let this_: *mut Self = self;
        static modernColors: [COLORREF; (NUM_COLORS as _)] = [
            0x000000, 0x464646, 0x787878, 0x300099, 0x241ced, 0x0078ff, 0x0ec2ff,
            0x00f2ff, 0x1de6a8, 0x4cb122, 0xefb700, 0xf36d4d, 0x99362f, 0x98316f,
            0xffffff, 0xdcdcdc, 0xb4b4b4, 0x3c5a9c, 0xb1a3ff, 0x7aaae5, 0x9ce4f5,
            0xbdf9ff, 0xbcf9d3, 0x61bb9d, 0xead999, 0xd19a70, 0x8e6d54, 0xd5a5b5
        ];
        static oldColors: [COLORREF; (NUM_COLORS as _)] = [
            0x000000, 0x808080, 0x000080, 0x008080, 0x008000, 0x808000, 0x800000,
            0x800080, 0x408080, 0x404000, 0xff8000, 0x804000, 0xff0040, 0x004080,
            0xffffff, 0xc0c0c0, 0x0000ff, 0x00ffff, 0x00ff00, 0xffff00, 0xff0000,
            0xff00ff, 0x80ffff, 0x80ff00, 0xffff80, 0xff8080, 0x8000ff, 0x4080ff
        ];
        static grayscale: [COLORREF; (NUM_COLORS as _)] = [
            0x000000, 0x090909, 0x121212, 0x1C1C1C, 0x252525, 0x2F2F2F, 0x383838,
            0x424242, 0x4B4B4B, 0x555555, 0x5E5E5E, 0x676767, 0x717171, 0x7B7B7B,
            0xFFFFFF, 0xF6F6F6, 0xEDEDED, 0xE3E3E3, 0xD9D9D9, 0xD0D0D0, 0xC7C7C7,
            0xBDBDBD, 0xB4B4B4, 0xAAAAAA, 0xA1A1A1, 0x979797, 0x8E8E8E, 0x848484
        ];
        match nPalette {
            PAL_MODERN => {
                CopyMemory!(
                    (*this_).m_colors.as_mut_ptr() as *mut core::ffi::c_void,
                    modernColors.as_ptr() as *const core::ffi::c_void,
                    core::mem::size_of::<[COLORREF; 28 as usize]>(),
                );
            }
            PAL_OLDTYPE => {
                CopyMemory!(
                    (*this_).m_colors.as_mut_ptr() as *mut core::ffi::c_void,
                    oldColors.as_ptr() as *const core::ffi::c_void,
                    core::mem::size_of::<[COLORREF; 28 as usize]>(),
                );
            }
            PAL_GRAYSCALE | PAL_MONOCHROME => {
                CopyMemory!(
                    (*this_).m_colors.as_mut_ptr() as *mut core::ffi::c_void,
                    grayscale.as_ptr() as *const core::ffi::c_void,
                    core::mem::size_of::<[COLORREF; 28 as usize]>(),
                );
            }
            _ => {}
        }
        (*this_).m_nSelectedPalette = nPalette;
        (*this_).m_fgColor = RGB!(0, 0, 0);
        (*this_).m_bgColor = RGB!(255, 255, 255);
        (*this_).NotifyPaletteChanged();
    }
    pub unsafe fn GetColor(&mut self, nIndex: UINT) -> COLORREF {
        let this_: *mut Self = self;
        if nIndex < NUM_COLORS {
            return (*this_).m_colors[nIndex as usize];
        } else {
            return 0;
        }
    }
    pub unsafe fn SetColor(&mut self, nIndex: UINT, newColor: COLORREF) {
        let this_: *mut Self = self;
        if nIndex < NUM_COLORS {
            (*this_).m_colors[nIndex as usize] = newColor;
            (*this_).NotifyPaletteChanged();
        }
    }
    pub unsafe fn GetFgColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        (*this_).m_fgColor
    }
    pub unsafe fn SetFgColor(&mut self, newColor: COLORREF) {
        let this_: *mut Self = self;
        (*this_).m_fgColor = newColor;
        (*this_).NotifyColorChanged();
    }
    pub unsafe fn GetBgColor(&mut self) -> COLORREF {
        let this_: *mut Self = self;
        (*this_).m_bgColor
    }
    pub unsafe fn SetBgColor(&mut self, newColor: COLORREF) {
        let this_: *mut Self = self;
        (*this_).m_bgColor = newColor;
        (*this_).NotifyColorChanged();
    }
}
