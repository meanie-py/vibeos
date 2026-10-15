use crate::prelude::*;
use core::ffi::c_void;

const AlphaShift: u32 = 24;
const RedShift: u32 = 16;
const GreenShift: u32 = 8;
const BlueShift: u32 = 0;








impl Color {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).Argb = 0xff000000;
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, argb: ARGB) {
        let this_: *mut Self = self;
        (*this_).Argb = argb;
        {
        }
    }
    pub unsafe fn __ctor_2(&mut self, r: BYTE, g: BYTE, b: BYTE) {
        let this_: *mut Self = self;
        (*this_).Argb = Color::MakeARGB(0xFF, r, g, b);
        {
        }
    }
    pub unsafe fn __ctor_3(&mut self, a: BYTE, r: BYTE, g: BYTE, b: BYTE) {
        let this_: *mut Self = self;
        (*this_).Argb = Color::MakeARGB(a, r, g, b);
        {
        }
    }
    pub unsafe fn GetValue(&mut self) -> Gdiplus_ARGB {
        let this_: *mut Self = self;
        (*this_).Argb
    }
    pub unsafe fn MakeARGB(a: u8, r: u8, g: u8, b: u8) -> u32 {
        let mut a0: u32 = a as u32;
        let mut r0: u32 = r as u32;
        let mut g0: u32 = g as u32;
        let mut b0: u32 = b as u32;
        return (a0 << AlphaShift) | (r0 << RedShift) | (g0 << GreenShift) | (b0 << BlueShift);
    }
}
