use crate::fsutil::dirty::DirtyMain;
use crate::fsutil::common::FindHandler;
use crate::fsutil::fsinfo::FsInfoMain;
use crate::fsutil::hardlink::HardLinkMain;
use crate::fsutil::common::PrintDefaultUsage;
use crate::fsutil::volume::VolumeMain;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1BFF234A2999: [u16; 40] = [
    0x0047u16, 0x0061, 0x0074, 0x0068, 0x0065, 0x0072, 0x0073, 0x0020, 0x0069, 0x006E,
    0x0066, 0x006F, 0x0072, 0x006D, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073,
    0x0020, 0x0061, 0x0062, 0x006F, 0x0075, 0x0074, 0x0020, 0x0066, 0x0069, 0x006C,
    0x0065, 0x0020, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_33F95C66E883: [u16; 7] = [0x0066u16, 0x0073, 0x0069, 0x006E, 0x0066, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_54D761CB7BDA: [u16; 26] = [
    0x004Du16, 0x0061, 0x006E, 0x0069, 0x0070, 0x0075, 0x006C, 0x0061, 0x0074, 0x0065,
    0x0073, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0064, 0x0069, 0x0072, 0x0074,
    0x0079, 0x0020, 0x0062, 0x0069, 0x0074, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_64698653071B: [u16; 16] = [
    0x004Du16, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0073, 0x0020, 0x0076, 0x006F,
    0x006C, 0x0075, 0x006D, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_6A7AA7C7C64A: [u16; 9] = [0x0068u16, 0x0061, 0x0072, 0x0064, 0x006C, 0x0069, 0x006E, 0x006B, 0x0000];
pub static RUSTCONV_C2R_WSTR_ADAABD1CCF26: [u16; 19] = [
    0x0048u16, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0073, 0x0020, 0x0068, 0x0061,
    0x0072, 0x0064, 0x0020, 0x006C, 0x0069, 0x006E, 0x006B, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_B79F7E106334: [u16; 6] = [0x0064u16, 0x0069, 0x0072, 0x0074, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_FFA3F54C0D9E: [u16; 7] = [0x0076u16, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000];

static mut HandlersList: [HandlerItem; 4] = unsafe {
    [
        HandlerItem {
            Handler: Some(
                DirtyMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_B79F7E106334.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_54D761CB7BDA.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                FsInfoMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_33F95C66E883.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_1BFF234A2999.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                HardLinkMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_6A7AA7C7C64A.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_ADAABD1CCF26.as_ptr() as *const TCHAR,
        },
        HandlerItem {
            Handler: Some(
                VolumeMain
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *const TCHAR,
                    ) -> ::core::ffi::c_int,
            ),
            Command: RUSTCONV_C2R_WSTR_FFA3F54C0D9E.as_ptr() as *const TCHAR,
            Desc: RUSTCONV_C2R_WSTR_64698653071B.as_ptr() as *const TCHAR,
        },
    ]
};

pub static mut DIRTY_NAME: [TCHAR; 6] = [b'd' as TCHAR, b'i' as TCHAR, b'r' as TCHAR, b't' as TCHAR, b'y' as TCHAR, 0];
pub static mut DIRTY_HELP: [TCHAR; 26] = [b'M' as TCHAR, b'a' as TCHAR, b'n' as TCHAR, b'i' as TCHAR, b'p' as TCHAR, b'u' as TCHAR, b'l' as TCHAR, b'a' as TCHAR, b't' as TCHAR, b'e' as TCHAR, b's' as TCHAR, b' ' as TCHAR, b't' as TCHAR, b'h' as TCHAR, b'e' as TCHAR, b' ' as TCHAR, b'd' as TCHAR, b'i' as TCHAR, b'r' as TCHAR, b't' as TCHAR, b'y' as TCHAR, b' ' as TCHAR, b'b' as TCHAR, b'i' as TCHAR, b't' as TCHAR, 0];
pub static mut FSINFO_NAME: [TCHAR; 7] = [b'f' as TCHAR, b's' as TCHAR, b'i' as TCHAR, b'n' as TCHAR, b'f' as TCHAR, b'o' as TCHAR, 0];
pub static mut FSINFO_HELP: [TCHAR; 40] = [b'G' as TCHAR, b'a' as TCHAR, b't' as TCHAR, b'h' as TCHAR, b'e' as TCHAR, b'r' as TCHAR, b's' as TCHAR, b' ' as TCHAR, b'i' as TCHAR, b'n' as TCHAR, b'f' as TCHAR, b'o' as TCHAR, b'r' as TCHAR, b'm' as TCHAR, b'a' as TCHAR, b't' as TCHAR, b'i' as TCHAR, b'o' as TCHAR, b'n' as TCHAR, b's' as TCHAR, b' ' as TCHAR, b'a' as TCHAR, b'b' as TCHAR, b'o' as TCHAR, b'u' as TCHAR, b't' as TCHAR, b' ' as TCHAR, b'f' as TCHAR, b'i' as TCHAR, b'l' as TCHAR, b'e' as TCHAR, b' ' as TCHAR, b's' as TCHAR, b'y' as TCHAR, b's' as TCHAR, b't' as TCHAR, b'e' as TCHAR, b'm' as TCHAR, b's' as TCHAR, 0];
pub static mut HARDLINK_NAME: [TCHAR; 9] = [b'h' as TCHAR, b'a' as TCHAR, b'r' as TCHAR, b'd' as TCHAR, b'l' as TCHAR, b'i' as TCHAR, b'n' as TCHAR, b'k' as TCHAR, 0];
pub static mut HARDLINK_HELP: [TCHAR; 19] = [b'H' as TCHAR, b'a' as TCHAR, b'n' as TCHAR, b'd' as TCHAR, b'l' as TCHAR, b'e' as TCHAR, b's' as TCHAR, b' ' as TCHAR, b'h' as TCHAR, b'a' as TCHAR, b'r' as TCHAR, b'd' as TCHAR, b' ' as TCHAR, b'l' as TCHAR, b'i' as TCHAR, b'n' as TCHAR, b'k' as TCHAR, b's' as TCHAR, 0];
pub static mut VOLUME_NAME: [TCHAR; 7] = [b'v' as TCHAR, b'o' as TCHAR, b'l' as TCHAR, b'u' as TCHAR, b'm' as TCHAR, b'e' as TCHAR, 0];
pub static mut VOLUME_HELP: [TCHAR; 16] = [b'M' as TCHAR, b'a' as TCHAR, b'n' as TCHAR, b'a' as TCHAR, b'g' as TCHAR, b'e' as TCHAR, b's' as TCHAR, b' ' as TCHAR, b'v' as TCHAR, b'o' as TCHAR, b'l' as TCHAR, b'u' as TCHAR, b'm' as TCHAR, b'e' as TCHAR, b's' as TCHAR, 0];

unsafe extern "C" fn PrintUsage(Command: *const TCHAR) {
    PrintDefaultUsage(
        (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */,
        Command,
        core::ptr::addr_of!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
            as core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *const TCHAR) -> i32 {
    return FindHandler(
        argc,
        argv,
        core::ptr::addr_of_mut!(HandlersList) as *mut HandlerItem,
        (core::mem::size_of_val(&*core::ptr::addr_of!(HandlersList))
            / core::mem::size_of_val(&(*core::ptr::addr_of!(HandlersList))[0]))
            as i32,
        Some(PrintUsage as unsafe extern "C" fn(*const TCHAR) -> ()),
    );
}
