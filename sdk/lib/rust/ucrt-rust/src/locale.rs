//! UCRT's locale data layout — the substrate every `_l`-suffixed body needs.
//!
//! ## Why this file exists at all
//!
//! 809 of ucrtbase's owed exports already have a Rust body in `msvcrt-crt-rust`,
//! and the obvious move is to reuse them. For every `_l`-suffixed one that is
//! **unsafe**, and silently so: `msvcrt-crt-rust` is bound against msvcrt's own
//! headers, where a `_locale_t` points at `threadlocaleinfostruct`
//! (`sdk/include/vcruntime/crtdefs.h:261`). A caller reaching ucrtbase passes a
//! pointer to `__crt_locale_data` instead, and the two are not variations on a
//! theme — they are different shapes:
//!
//! * msvcrt leads with `refcount, lc_codepage, lc_collate_cp, lc_handle[6],
//!   lc_id[6]`; UCRT leads with a `_public` sub-struct and has no `lc_handle`
//!   or `lc_id` at all.
//! * UCRT adds `lc_time_cp` and a trailing `locale_name[6]`.
//! * `pctype`/`mb_cur_max`/`lc_codepage` are inline members in msvcrt but live
//!   inside `_public` in UCRT.
//!
//! So essentially every field lands somewhere else. Dereferencing a UCRT
//! `_locale_t` through msvcrt's layout reads adjacent memory and keeps going —
//! no fault, no diagnostic, wrong answers. This module is the one place that
//! knows UCRT's shape, so that mistake can only be made here.
//!
//! ## The design rule that follows
//!
//! **The shared core never sees a `_locale_t`.** When a family is lifted into a
//! neutral core crate for both DLLs to use (the `cnt-rust` pattern — see this
//! crate's Cargo.toml), the core takes already-extracted values: a ctype
//! pointer, a codepage, an `mb_cur_max`. Each DLL's `#[no_mangle]` shim does the
//! struct access with its own layout and passes plain values in. That keeps
//! layout knowledge in two small places instead of smeared through every body,
//! and it is what makes one implementation serve two ABIs.
//!
//! ## Where these numbers come from, and why they are written down
//!
//! Measured, not derived: a probe TU emitting `__builtin_offsetof` for every
//! field of the real `struct __crt_locale_data`, compiled with the BUILD'S OWN
//! flags — the exact argv from `build-<arch>/compile_commands.json` for a ucrt
//! TU, source and output swapped. Reconstructing the compiler invocation by
//! hand instead had it silently resolving the toolchain's `winnt.h` rather than
//! ours, which is the sort of answer that looks right and is not.
//!
//! They are recorded here because the ground truth is about to be DELETED.
//! `sdk/lib/ucrt/inc/corecrt_internal.h` is vendored Windows SDK source and goes
//! when the last family that needs it goes; after that there is nothing in the
//! tree to re-derive these from. The `const` assertions below turn the table
//! into something the compiler checks on every build of every arch, so a
//! mistake here is a build error rather than a corrupted read.
//!
//! To re-derive while the header still exists, see the commit that added this
//! file. `_public` sits at offset 0 on all three arches — it is absent from the
//! probe's output only because a zero-valued initialiser lands in `.bss`.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ushort, c_void};

/// The public head of UCRT's locale data — `__crt_locale_data_public`
/// (`sdk/include/ucrt/corecrt.h:621`). A `_locale_t` is a pointer to a
/// `__crt_locale_pointers` whose first member points here, and these three
/// fields are the ones ordinary locale-aware code actually reads.
#[repr(C)]
pub struct CrtLocaleDataPublic {
    pub locale_pctype: *const c_ushort,
    pub locale_mb_cur_max: c_int,
    pub locale_lc_codepage: c_uint,
}

/// One entry of `lc_category[6]` — `__crt_locale_refcount`.
#[repr(C)]
pub struct CrtLocaleRefcount {
    pub locale: *mut c_char,
    pub wlocale: *mut u16,
    pub refcount: *mut c_long,
    pub wrefcount: *mut c_long,
}

/// UCRT's `struct __crt_locale_data` (`sdk/lib/ucrt/inc/corecrt_internal.h:502`).
///
/// NOT interchangeable with msvcrt's `threadlocaleinfostruct`. See the module
/// docs: the shapes differ from the second field onwards.
impl CrtLocaleData {
    /// The public head, where `_locale_pctype` lives. A method rather than a
    /// bare field access so the read site reads like the C's
    /// `locinfo->_public._locale_pctype`.
    pub fn public_ref(&self) -> *const CrtLocaleDataPublic {
        &self.public
    }
}

#[repr(C)]
pub struct CrtLocaleData {
    pub public: CrtLocaleDataPublic,
    pub refcount: c_long,
    pub lc_collate_cp: c_uint,
    pub lc_time_cp: c_uint,
    pub lc_clike: c_int,
    pub lc_category: [CrtLocaleRefcount; 6],
    pub lconv_intl_refcount: *mut c_long,
    pub lconv_num_refcount: *mut c_long,
    pub lconv_mon_refcount: *mut c_long,
    pub lconv: *mut c_void,
    pub ctype1_refcount: *mut c_long,
    pub ctype1: *mut c_ushort,
    pub pclmap: *const u8,
    pub pcumap: *const u8,
    pub lc_time_curr: *const c_void,
    pub locale_name: [*mut u16; 6],
}


/// UCRT's `struct __crt_locale_pointers` (`sdk/include/ucrt/corecrt.h:628`).
/// **This is what a `_locale_t` actually points at** — the locale data hangs
/// off `locinfo`, and the multibyte data off `mbcinfo`. Code that treats a
/// `_locale_t` as a direct pointer to the locale data is off by one level.
#[repr(C)]
pub struct CrtLocalePointers {
    pub locinfo: *mut CrtLocaleData,
    pub mbcinfo: *mut CrtMultibyteData,
}

/// UCRT's `struct __crt_multibyte_data` (`sdk/lib/ucrt/inc/corecrt_internal.h`).
///
/// `mbctype` is the table the whole `_ismbb*` family indexes, and note it is
/// **257 bytes, indexed at `1 + byte`** — the leading entry is not padding, so
/// an implementation that indexes at `byte` is wrong for every input.
#[repr(C)]
pub struct CrtMultibyteData {
    pub refcount: c_long,
    pub mbcodepage: c_int,
    pub ismbcodepage: c_int,
    pub mbulinfo: [c_ushort; 6],
    pub mbctype: [u8; 257],
    pub mbcasemap: [u8; 256],
    pub mblocalename: *const u16,
}

// ── measured ground truth ────────────────────────────────────────────────────
//
// i386 differs from amd64/arm64 only by pointer width; amd64 and arm64 agree
// exactly. Checked at compile time on whichever arch is being built, so a
// reordered or dropped field cannot reach a link.

macro_rules! assert_offset {
    ($field:ident, $want:expr) => {
        const _: () = assert!(
            core::mem::offset_of!(CrtLocaleData, $field) == $want,
            concat!("__crt_locale_data.", stringify!($field), " is not where UCRT puts it"),
        );
    };
}

#[cfg(target_pointer_width = "32")]
mod pinned {
    use super::*;
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, locinfo) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, mbcinfo) == 4);
    const _: () = assert!(core::mem::size_of::<CrtLocalePointers>() == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, refcount) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcodepage) == 4);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, ismbcodepage) == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbulinfo) == 12);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbctype) == 24);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcasemap) == 281);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mblocalename) == 540);
    const _: () = assert!(core::mem::size_of::<CrtMultibyteData>() == 544);
    assert_offset!(public, 0);
    assert_offset!(refcount, 12);
    assert_offset!(lc_collate_cp, 16);
    assert_offset!(lc_time_cp, 20);
    assert_offset!(lc_clike, 24);
    assert_offset!(lc_category, 28);
    assert_offset!(lconv_intl_refcount, 124);
    assert_offset!(lconv_num_refcount, 128);
    assert_offset!(lconv_mon_refcount, 132);
    assert_offset!(lconv, 136);
    assert_offset!(ctype1_refcount, 140);
    assert_offset!(ctype1, 144);
    assert_offset!(pclmap, 148);
    assert_offset!(pcumap, 152);
    assert_offset!(lc_time_curr, 156);
    assert_offset!(locale_name, 160);
    const _: () = assert!(core::mem::size_of::<CrtLocaleData>() == 184);
}

#[cfg(target_pointer_width = "64")]
mod pinned {
    use super::*;
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, locinfo) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtLocalePointers, mbcinfo) == 8);
    const _: () = assert!(core::mem::size_of::<CrtLocalePointers>() == 16);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, refcount) == 0);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcodepage) == 4);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, ismbcodepage) == 8);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbulinfo) == 12);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbctype) == 24);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mbcasemap) == 281);
    const _: () = assert!(core::mem::offset_of!(CrtMultibyteData, mblocalename) == 544);
    const _: () = assert!(core::mem::size_of::<CrtMultibyteData>() == 552);
    assert_offset!(public, 0);
    assert_offset!(refcount, 16);
    assert_offset!(lc_collate_cp, 20);
    assert_offset!(lc_time_cp, 24);
    assert_offset!(lc_clike, 28);
    assert_offset!(lc_category, 32);
    assert_offset!(lconv_intl_refcount, 224);
    assert_offset!(lconv_num_refcount, 232);
    assert_offset!(lconv_mon_refcount, 240);
    assert_offset!(lconv, 248);
    assert_offset!(ctype1_refcount, 256);
    assert_offset!(ctype1, 264);
    assert_offset!(pclmap, 272);
    assert_offset!(pcumap, 280);
    assert_offset!(lc_time_curr, 288);
    assert_offset!(locale_name, 296);
    const _: () = assert!(core::mem::size_of::<CrtLocaleData>() == 344);
}
