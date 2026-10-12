use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn QuerySupportedMedia(
    DriveRoot: PWCHAR,
    CurrentMedia: *mut FMIFS_MEDIA_FLAG,
    Unknown3: ULONG,
    Unknown4: PULONG,
) -> BOOL {
    (FALSE as i32)
}
