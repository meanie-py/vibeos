//! mountmgr.rs — faithful Rust bodies for the MOUNTMGR_IS_*_VOLUME_NAME macros
//! (sdk/include/ddk/mountmgr.h). They are C preprocessor macros (no symbol); the
//! kernel32 translation referenced them as `extern "system" fn(PUNICODE_STRING) ->
//! BOOLEAN`, so we provide real `#[no_mangle] extern "system"` definitions.

use ntdll_sys::nt::{BOOLEAN, PUNICODE_STRING};

/// MOUNTMGR_IS_VOLUME_NAME(s) — the base GUID-volume-name shape check.
#[inline]
unsafe fn is_volume_name(s: PUNICODE_STRING) -> bool {
    let len = (*s).Length;
    let b = (*s).Buffer;
    let ch = |i: usize| *b.add(i) as u32;
    (len == 96 || (len == 98 && ch(48) == b'\\' as u32))
        && ch(0) == b'\\' as u32
        && (ch(1) == b'?' as u32 || ch(1) == b'\\' as u32)
        && ch(2) == b'?' as u32
        && ch(3) == b'\\' as u32
        && ch(4) == b'V' as u32
        && ch(5) == b'o' as u32
        && ch(6) == b'l' as u32
        && ch(7) == b'u' as u32
        && ch(8) == b'm' as u32
        && ch(9) == b'e' as u32
        && ch(10) == b'{' as u32
        && ch(19) == b'-' as u32
        && ch(24) == b'-' as u32
        && ch(29) == b'-' as u32
        && ch(34) == b'-' as u32
        && ch(47) == b'}' as u32
}

/// MOUNTMGR_IS_NT_VOLUME_NAME(s)
#[no_mangle]
pub unsafe extern "system" fn MOUNTMGR_IS_NT_VOLUME_NAME(s: PUNICODE_STRING) -> BOOLEAN {
    (is_volume_name(s) && (*s).Length == 96 && *(*s).Buffer.add(1) == b'?' as u16) as BOOLEAN
}

/// MOUNTMGR_IS_DOS_VOLUME_NAME_WB(s)
#[no_mangle]
pub unsafe extern "system" fn MOUNTMGR_IS_DOS_VOLUME_NAME_WB(s: PUNICODE_STRING) -> BOOLEAN {
    (is_volume_name(s) && (*s).Length == 98 && *(*s).Buffer.add(1) == b'\\' as u16) as BOOLEAN
}
