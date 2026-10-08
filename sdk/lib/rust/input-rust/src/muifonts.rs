//! `base/setup/lib/muifonts.h`'s WCHAR arrays, which input_list.c DEFINES by including the
//! header (data in a header, so no translated function carries it). Generated from the
//! header's own initialisers -- units verbatim, char literals as their code -- not copied
//! from setuplib's mui.rs. `static mut` because the C arrays are non-const `WCHAR[]`.
#![allow(non_upper_case_globals)]
use crate::prelude::WCHAR;

pub static mut CSF_LocalName0: [WCHAR; 3] = [0x5b8b, 0x4f53, 0x0000];
pub static mut CSF_LocalName1: [WCHAR; 5] = [0x4e2d, 0x6613, 0x5b8b, 0x4f53, 0x0000];
pub static mut CSF_LocalName2: [WCHAR; 4] = [0x65b0, 0x5b8b, 0x4f53, 0x0000];
pub static mut CSF_LocalName3: [WCHAR; 3] = [0xfc4b, 0x91cc, 0x0000];
pub static mut CSF_LocalName4: [WCHAR; 5] = [0x004d, 0x0053, 0x5b8b, 0x4f53, 0x0000];
pub static mut CTF_LocalName0: [WCHAR; 4] = [0x7d30, 0x660e, 0x9ad4, 0x0000];
pub static mut CTF_LocalName1: [WCHAR; 5] = [0x65b0, 0x7d30, 0x660e, 0x9ad4, 0x0000];
pub static mut CTF_LocalName2: [WCHAR; 6] = [0x83ef, 0x5eb7, 0x4e2d, 0x660e, 0x9ad4, 0x0000];
pub static mut CTF_LocalName3: [WCHAR; 6] = [0x83ef, 0x5eb7, 0x7c97, 0x660e, 0x9ad4, 0x0000];
pub static mut JF_LocalName0: [WCHAR; 6] = [0xff2d, 0xff33, 0x0020, 0x660e, 0x671d, 0x0000];
pub static mut JF_LocalName1: [WCHAR; 7] = [0xff2d, 0xff33, 0x0020, 0xff30, 0x660e, 0x671d, 0x0000];
pub static mut JF_LocalName2: [WCHAR; 8] = [0xff2d, 0xff33, 0x0020, 0x30b4, 0x30b7, 0x30c3, 0x30af, 0x0000];
pub static mut JF_LocalName3: [WCHAR; 9] = [0xff2d, 0xff33, 0x0020, 0xff30, 0x30b4, 0x30b7, 0x30c3, 0x30af, 0x0000];
pub static mut KF_LocalName0: [WCHAR; 3] = [0xbc14, 0xd0d5, 0x0000];
pub static mut KF_LocalName1: [WCHAR; 4] = [0xbc14, 0xd0d5, 0xccb4, 0x0000];
pub static mut KF_LocalName2: [WCHAR; 3] = [0xb3cb, 0xc6c0, 0x0000];
pub static mut KF_LocalName3: [WCHAR; 4] = [0xb3cb, 0xc6c0, 0xccb4, 0x0000];
pub static mut KF_LocalName4: [WCHAR; 3] = [0xad74, 0xb9bc, 0x0000];
pub static mut KF_LocalName5: [WCHAR; 4] = [0xad74, 0xb9bc, 0xccb4, 0x0000];
pub static mut KF_LocalName6: [WCHAR; 3] = [0xad81, 0xc11c, 0x0000];
pub static mut KF_LocalName7: [WCHAR; 4] = [0xad81, 0xc11c, 0xccb4, 0x0000];
