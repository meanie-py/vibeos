use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Wow64CbMarshalTable: [WOW64_CB_MARSHAL; 6] = unsafe {
    [
        _WOW64_CB_MARSHAL {
            Msg: 0x2b as ::core::ffi::c_uint,
            Size32: 48 as ::core::ffi::c_uint,
            Size64: 64 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown_tagDRAWITEMSTRUCT
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp_tagDRAWITEMSTRUCT
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
        _WOW64_CB_MARSHAL {
            Msg: 0x2c as ::core::ffi::c_uint,
            Size32: 24 as ::core::ffi::c_uint,
            Size64: 32 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown_tagMEASUREITEMSTRUCT
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp_tagMEASUREITEMSTRUCT
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
        _WOW64_CB_MARSHAL {
            Msg: 0x53 as ::core::ffi::c_uint,
            Size32: 28 as ::core::ffi::c_uint,
            Size64: 40 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown_tagHELPINFO
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp_tagHELPINFO
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
        _WOW64_CB_MARSHAL {
            Msg: 0x213 as ::core::ffi::c_uint,
            Size32: 12 as ::core::ffi::c_uint,
            Size64: 24 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown_tagMDINEXTMENU
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp_tagMDINEXTMENU
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
        _WOW64_CB_MARSHAL {
            Msg: 0x46 as ::core::ffi::c_uint,
            Size32: 28 as ::core::ffi::c_uint,
            Size64: 40 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown__WINDOWPOS
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp__WINDOWPOS
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
        _WOW64_CB_MARSHAL {
            Msg: 0x47 as ::core::ffi::c_uint,
            Size32: 28 as ::core::ffi::c_uint,
            Size64: 40 as ::core::ffi::c_uint,
            Down: Some(
                WowCbDown__WINDOWPOS
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
            Up: Some(
                WowCbUp__WINDOWPOS
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_uchar,
                        *mut ::core::ffi::c_uchar,
                    ) -> (),
            ),
        },
    ]
};

pub type ULONG = u32;
pub type LONG = i32;
pub type LONGLONG = i64;


pub type PWOW64_CB_MARSHAL = *mut WOW64_CB_MARSHAL;

unsafe extern "C" fn WowCbCpy(mut d: *mut u8, mut s: *const u8, mut n: u32) {
    while n != 0 {
        n = n.wrapping_sub(1);
        *d = *s;
        d = d.offset(1);
        s = s.offset(1);
    }
}

unsafe extern "C" fn WowCbZero(mut d: *mut ::core::ffi::c_uchar, mut n: ::core::ffi::c_uint) {
    loop {
        let fresh3 = n;
        n = n.wrapping_sub(1);
        if !(fresh3 != 0) {
            break;
        }
        let fresh4 = d;
        d = d.offset(1);
        *fresh4 = 0 as ::core::ffi::c_uchar;
    }
}

unsafe extern "C" fn WowCbSx(d: *mut u8, s: *const u8) {
    let mut v: LONG = 0;
    let mut w: LONGLONG = 0;
    WowCbCpy(core::ptr::addr_of_mut!(v) as *mut u8, s, 4);
    w = v as LONGLONG;
    WowCbCpy(d, core::ptr::addr_of_mut!(w) as *const u8, 8);
}

unsafe extern "C" fn WowCbNx(d: *mut u8, s: *const u8) {
    WowCbCpy(d, s, 4);
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagDRAWITEMSTRUCT(s: *const u8, d: *mut u8) {
    WowCbZero(d, 48);
    WowCbCpy(d.add(0), s.add(0), 4); /* CtlType */
    WowCbCpy(d.add(4), s.add(4), 4); /* CtlID */
    WowCbCpy(d.add(8), s.add(8), 4); /* itemID */
    WowCbCpy(d.add(12), s.add(12), 4); /* itemAction */
    WowCbCpy(d.add(16), s.add(16), 4); /* itemState */
    WowCbNx(d.add(20), s.add(24)); /* hwndItem */
    WowCbNx(d.add(24), s.add(32)); /* hDC */
    WowCbCpy(d.add(28), s.add(40), 16); /* rcItem */
    WowCbNx(d.add(44), s.add(56)); /* itemData */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagDRAWITEMSTRUCT(s: *const u8, d: *mut u8) {
    WowCbCpy(d.offset(0 as isize), s.offset(0 as isize), 4 as ::core::ffi::c_uint); /* CtlType */
    WowCbCpy(d.offset(4 as isize), s.offset(4 as isize), 4 as ::core::ffi::c_uint); /* CtlID */
    WowCbCpy(d.offset(8 as isize), s.offset(8 as isize), 4 as ::core::ffi::c_uint); /* itemID */
    WowCbCpy(d.offset(12 as isize), s.offset(12 as isize), 4 as ::core::ffi::c_uint); /* itemAction */
    WowCbCpy(d.offset(16 as isize), s.offset(16 as isize), 4 as ::core::ffi::c_uint); /* itemState */
    WowCbSx(d.offset(24 as isize), s.offset(20 as isize)); /* hwndItem */
    WowCbSx(d.offset(32 as isize), s.offset(24 as isize)); /* hDC */
    WowCbCpy(d.offset(40 as isize), s.offset(28 as isize), 16 as ::core::ffi::c_uint); /* rcItem */
    WowCbSx(d.offset(56 as isize), s.offset(44 as isize)); /* itemData */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagMEASUREITEMSTRUCT(s: *const u8, d: *mut u8) {
    WowCbZero(d, 24);
    WowCbCpy(d.add(0 as usize), s.add(0 as usize), 4); /* CtlType */
    WowCbCpy(d.add(4 as usize), s.add(4 as usize), 4); /* CtlID */
    WowCbCpy(d.add(8 as usize), s.add(8 as usize), 4); /* itemID */
    WowCbCpy(d.add(12 as usize), s.add(12 as usize), 4); /* itemWidth */
    WowCbCpy(d.add(16 as usize), s.add(16 as usize), 4); /* itemHeight */
    WowCbNx(d.add(20 as usize), s.add(24 as usize)); /* itemData */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagMEASUREITEMSTRUCT(s: *const u8, d: *mut u8) {
    WowCbCpy(d.add(0 as usize), s.add(0 as usize), 4 as u32); /* CtlType */
    WowCbCpy(d.add(4 as usize), s.add(4 as usize), 4 as u32); /* CtlID */
    WowCbCpy(d.add(8 as usize), s.add(8 as usize), 4 as u32); /* itemID */
    WowCbCpy(d.add(12 as usize), s.add(12 as usize), 4 as u32); /* itemWidth */
    WowCbCpy(d.add(16 as usize), s.add(16 as usize), 4 as u32); /* itemHeight */
    WowCbSx(d.add(24 as usize), s.add(20 as usize)); /* itemData */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagHELPINFO(s: *const u8, d: *mut u8) {
    WowCbZero(d, 28);
    WowCbCpy(d.add(0), s.add(0), 4); /* cbSize */
    WowCbCpy(d.add(4), s.add(4), 4); /* iContextType */
    WowCbCpy(d.add(8), s.add(8), 4); /* iCtrlId */
    WowCbNx(d.add(12), s.add(16)); /* hItemHandle */
    WowCbNx(d.add(16), s.add(24)); /* dwContextId */
    WowCbCpy(d.add(20), s.add(32), 8); /* MousePos */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagHELPINFO(s: *const u8, d: *mut u8) {
    WowCbCpy(d.offset(0 as isize), s.offset(0 as isize), 4 as u32); // cbSize
    WowCbCpy(d.offset(4 as isize), s.offset(4 as isize), 4 as u32); // iContextType
    WowCbCpy(d.offset(8 as isize), s.offset(8 as isize), 4 as u32); // iCtrlId
    WowCbSx(d.offset(16 as isize), s.offset(12 as isize)); // hItemHandle
    WowCbSx(d.offset(24 as isize), s.offset(16 as isize)); // dwContextId
    WowCbCpy(d.offset(32 as isize), s.offset(20 as isize), 8 as u32); // MousePos
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagMDINEXTMENU(s: *const u8, d: *mut u8) {
    WowCbZero(d, 12 as u32);
    WowCbNx(d.add(0 as usize), s.add(0 as usize)); /* hmenuIn */
    WowCbNx(d.add(4 as usize), s.add(8 as usize)); /* hmenuNext */
    WowCbNx(d.add(8 as usize), s.add(16 as usize)); /* hwndNext */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagMDINEXTMENU(s: *const u8, d: *mut u8) {
    WowCbSx(d.add(0), s.add(0)); /* hmenuIn */
    WowCbSx(d.add(8), s.add(4)); /* hmenuNext */
    WowCbSx(d.add(16), s.add(8)); /* hwndNext */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown__WINDOWPOS(s: *const ::core::ffi::c_uchar, d: *mut ::core::ffi::c_uchar) {
    WowCbZero(d, 28);
    WowCbNx(d.add(0), s.add(0)); /* hwnd */
    WowCbNx(d.add(4), s.add(8)); /* hwndInsertAfter */
    WowCbCpy(d.add(8), s.add(16), 4); /* x */
    WowCbCpy(d.add(12), s.add(20), 4); /* y */
    WowCbCpy(d.add(16), s.add(24), 4); /* cx */
    WowCbCpy(d.add(20), s.add(28), 4); /* cy */
    WowCbCpy(d.add(24), s.add(32), 4); /* flags */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp__WINDOWPOS(s: *const u8, d: *mut u8) {
    WowCbSx(d.add(0), s.add(0)); /* hwnd */
    WowCbSx(d.add(8), s.add(4)); /* hwndInsertAfter */
    WowCbCpy(d.add(16), s.add(8), 4); /* x */
    WowCbCpy(d.add(20), s.add(12), 4); /* y */
    WowCbCpy(d.add(24), s.add(16), 4); /* cx */
    WowCbCpy(d.add(28), s.add(20), 4); /* cy */
    WowCbCpy(d.add(32), s.add(24), 4); /* flags */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagNCCALCSIZE_PARAMS(s: *const ::core::ffi::c_uchar, d: *mut ::core::ffi::c_uchar) {
    WowCbZero(d, 52 as u32);
    WowCbCpy(d.add(0 as usize), s.add(0 as usize), 48 as u32); /* rgrc */
    WowCbNx(d.add(48 as usize), s.add(48 as usize)); /* lppos */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagNCCALCSIZE_PARAMS(s: *const u8, d: *mut u8) {
    WowCbCpy(d.add(0), s.add(0), 48 as u32); /* rgrc */
    /* lppos: kernel pointer preserved (guest value is scratch) */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbDown_tagCOPYDATASTRUCT(s: *const ::core::ffi::c_uchar, d: *mut ::core::ffi::c_uchar) {
    WowCbZero(d, 12 as u32);
    WowCbNx(d.add(0 as usize), s.add(0 as usize)); /* dwData */
    WowCbCpy(d.add(4 as usize), s.add(8 as usize), 4 as u32); /* cbData */
    WowCbNx(d.add(8 as usize), s.add(16 as usize)); /* lpData */
}
#[no_mangle]
pub unsafe extern "C" fn WowCbUp_tagCOPYDATASTRUCT(s: *const ::core::ffi::c_uchar, d: *mut ::core::ffi::c_uchar) {
    WowCbSx(d.offset(0 as isize), s.offset(0 as isize)); /* dwData */
    WowCbCpy(d.offset(8 as isize), s.offset(4 as isize), 4 as ::core::ffi::c_uint); /* cbData */
    WowCbSx(d.offset(16 as isize), s.offset(8 as isize)); /* lpData */
}
#[no_mangle]
pub unsafe extern "C" fn Wow64CbFindMarshal(Msg: ::core::ffi::c_uint) -> *const WOW64_CB_MARSHAL {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize)
        < (::core::mem::size_of_val(&Wow64CbMarshalTable)
            .wrapping_div(::core::mem::size_of_val(&Wow64CbMarshalTable[0])))
    {
        if Wow64CbMarshalTable[i as usize].Msg == Msg {
            return core::ptr::addr_of!(Wow64CbMarshalTable[i as usize]);
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null();
}
