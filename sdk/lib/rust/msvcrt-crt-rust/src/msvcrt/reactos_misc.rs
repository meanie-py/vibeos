//! `dll/win32/msvcrt/reactos/misc.c` — 5 exports (7 on i386).
//!
//! The module carries its DIRECTORY in its name because the basename is taken: `misc.rs` is
//! msvcrt's own Wine-derived `misc.c`. These two files are unrelated; ReactOS added this one
//! for the handful of MS-CRT names Wine never had.

use crate::msvcrt::data::MSVCRT__winver;
use crate::prelude::*;

/// `msvcrt.spec:694` — `@ extern _mbcasemap`. The DLL exports this array under its own name,
/// so `#[no_mangle]` is not a style choice: a mangled Rust static leaves the export naming
/// reactos/misc.c's copy while `__p__mbcasemap` hands out Rust's, and the two are different
/// addresses. That split shipped once already, for `_mbctype`.
///
/// The C writes it as eight literal rows of sixteen followed by an implicit zero tail. All it
/// is, is an ASCII case swap: `t['A'..='Z'] = 'a'..='z'` and `t['a'..='z'] = 'A'..='Z'`, every
/// other byte zero — including index 256, which exists only so a `unsigned char` index can
/// never run off the end. The `assert!`s below pin the four boundaries the loop form could
/// get wrong; they are const-evaluated, so an off-by-one is a build failure, not a runtime
/// surprise in some locale nobody tests.
/// The table as a `const`, so the assertions below can read it. They cannot read the `static
/// mut` — const-eval refuses to touch mutable global memory even before anything can have
/// written to it.
const MBCASEMAP: [core::ffi::c_uchar; 257] = {
    let mut t = [0u8; 257];
    let mut i = 0usize;
    while i < 26 {
        t[b'A' as usize + i] = b'a' + i as u8;
        t[b'a' as usize + i] = b'A' + i as u8;
        i += 1;
    }
    t
};

const _: () = {
    assert!(MBCASEMAP[b'@' as usize] == 0);
    assert!(MBCASEMAP[b'A' as usize] == b'a' && MBCASEMAP[b'Z' as usize] == b'z');
    assert!(MBCASEMAP[b'[' as usize] == 0 && MBCASEMAP[b'`' as usize] == 0);
    assert!(MBCASEMAP[b'a' as usize] == b'A' && MBCASEMAP[b'z' as usize] == b'Z');
    assert!(MBCASEMAP[b'{' as usize] == 0 && MBCASEMAP[256] == 0);
};

#[no_mangle]
pub static mut _mbcasemap: [core::ffi::c_uchar; 257] = MBCASEMAP;

/// `msvcrt.spec:442` — `@ extern -arch=i386,x86_64 _fileinfo`. Exported by name on two of the
/// three arches; defined on all three, exactly as the C does, because `_get_fileinfo` and
/// `_set_fileinfo` are exported everywhere and read it.
#[no_mangle]
pub static mut _fileinfo: core::ffi::c_int = -1;

#[no_mangle]
pub unsafe extern "C" fn __p__mbcasemap() -> *mut core::ffi::c_uchar {
    core::ptr::addr_of_mut!(_mbcasemap) as *mut core::ffi::c_uchar
}

#[no_mangle]
pub unsafe extern "C" fn __p__fileinfo() -> *mut core::ffi::c_int {
    core::ptr::addr_of_mut!(_fileinfo)
}

#[no_mangle]
pub unsafe extern "C" fn _get_fileinfo(_Value: *mut core::ffi::c_int) -> core::ffi::c_int {
    if MSVCRT_CHECK_PMT!(!_Value.is_null()) == 0 {
        return EINVAL as core::ffi::c_int;
    }

    *_Value = _fileinfo;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _set_fileinfo(_Value: core::ffi::c_int) -> core::ffi::c_int {
    _fileinfo = _Value;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_winver(_Value: *mut core::ffi::c_uint) -> core::ffi::c_int {
    if MSVCRT_CHECK_PMT!(!_Value.is_null()) == 0 {
        return EINVAL as core::ffi::c_int;
    }

    // The C reaches this through `extern unsigned int MSVCRT__winver;`. A Rust path instead
    // of an `extern "C"` block on purpose: data.rs owns the definition, so `cargo check`
    // proves the type here, where a redeclaration would only be checked at link time — and
    // link time cannot see a width mismatch at all.
    *_Value = MSVCRT__winver;
    0
}
