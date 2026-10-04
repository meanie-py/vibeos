//! The Win32 header *macros* and folded constants, ported once.
//!
//! # Why this crate exists
//!
//! bindgen emits types, functions and object-like constants. It does not emit **function-like
//! macros**, and it drops the constants whose definition it cannot fold (a cast expression, a
//! `sizeof`, a pointer literal). Every module we translate therefore has to re-port the same
//! handful of `minwindef.h` / `winuser.h` / `wingdi.h` / `ntbasedef.h` macros into its own
//! prelude, by hand, from whatever the model wrote that day.
//!
//! By the time this crate was written, `LOWORD` alone existed **32 times across 12 crates in 8
//! distinct spellings**, `HIWORD` in 8, `ZeroMemory` in 6, and `DWLP_USER` in 9. That is not a
//! tidiness problem: each spelling is an independent chance to get one wrong, and nothing
//! compares them.
//!
//! # The type basis is core primitives, deliberately
//!
//! Consumers each have their own bindgen island and a shared crate cannot depend on any of them.
//! It does not need to. `WORD`, `BYTE`, `COLORREF`, `DWORD_PTR` and `LPWSTR` are transparent
//! aliases for `u16`, `u8`, `u32`, `usize` and `*mut u16` in every island in the tree, and Rust
//! unifies type aliases structurally — so a caller's `WORD` *is* this crate's `u16`. This is the
//! same basis `strsafe-rust` established.
//!
//! For that reason **the macro bodies spell the primitive, never the Win32 alias.** `macro_rules!`
//! is not hygienic for type paths: a body containing `as WORD` would resolve `WORD` at the *call
//! site*, so it would compile in a consumer whose prelude happens to glob `WORD` into scope and
//! fail in one that does not. Each macro's doc comment carries the C signature it is answering to.
//!
//! # Faithfulness
//!
//! The bodies below are transcribed from the headers in `sdk/include/`, not from the copies they
//! replace, and two of those copies turned out to disagree with the header:
//!
//! - **`RGB`** — `wingdi.h:3386` masks every component to `BYTE` *before* shifting. All four
//!   in-tree copies (aclui, desk, sysdm, syssetup) shifted the unmasked value, so a component
//!   above 255 bled into the next channel: `RGB(r, 0x1FF, b)` yielded `0x0001FF00 | b << 16`,
//!   green corrupting blue, where C yields `0xFF00`. Reachable only from a caller that does its
//!   own arithmetic on a channel; the tree's five call sites all pass `GetRValue`-style bytes,
//!   so this fixes a latent trap rather than a live bug.
//! - **`GetGValue`** — `wingdi.h:3382` truncates to `WORD` before `>> 8`; the in-tree copies
//!   shifted the full `COLORREF`. Same result for every input (both select bits 8..15), so this
//!   one is a re-spelling, not a fix. It is recorded because proving it *was* a re-spelling is
//!   the work, and the next reader should not have to redo it.
//!
//! # The boundary: fixed-width primitives only
//!
//! "Core primitive" is a narrower licence than it first looks, and the compiler drew the line
//! during this extraction rather than the design doing it up front:
//!
//! - **`HWND` is `*mut HWND__`**, a bindgen *opaque struct* emitted separately into every island.
//!   It is a NOMINAL type, so a shared `*mut c_void` does not unify with it. `HANDLE`, by
//!   contrast, really is `*mut c_void` everywhere, which is why `INVALID_HANDLE_VALUE` is here
//!   and `HWND_BROADCAST` is not.
//! - **`LPARAM`/`WPARAM`/`LRESULT` are not `isize`/`usize`.** They alias `LONG_PTR`/`UINT_PTR`,
//!   which bindgen emits as `c_long`/`c_uint` on i386 and `c_longlong`/`c_ulonglong` on the
//!   64-bit targets — that is, a *fixed-width* type that changes with the arch. `isize` is a
//!   distinct primitive from `i32` even where both are 32 bits, so a shared macro returning
//!   `isize` fails to compile against every one of them.
//!
//! So the rule for adding anything here: share it only if its result type is a fixed-width
//! primitive that is the SAME on all three targets (`u8`, `u16`, `u32`, `i32`, `*mut u16`).
//! Anything pointer-width or nominal belongs to the consumer, where the island's own alias is
//! in scope.
//!
//! # What is deliberately NOT here
//!
//! - **`MAKEINTRESOURCE`** (no suffix) — it is `MAKEINTRESOURCEW` only because these modules
//!   compile `-DUNICODE`. That is a per-consumer property, so the TCHAR-neutral spelling stays in
//!   each consumer's prelude as a one-line alias onto `MAKEINTRESOURCEW!`. A shared crate that
//!   exported it would silently impose UNICODE on a module that is not.
//! - **`ListView_*` / `PropSheet_*`** — they expand to `SendMessageW` against island *struct*
//!   types (`LVITEMW`). Struct layout is exactly what a shared crate cannot see, and the
//!   core-primitive basis above is the precondition that makes sharing safe at all.
//! - **`GetWindowLongPtr` / `SetWindowLongPtr`** — genuinely `#[cfg]`-split by pointer width and
//!   bound to each consumer's `extern` block.
//! - **The `as UINT` `LOWORD` family** — shell32 and menuband spell `LOWORD`/`HIWORD` with a
//!   `UINT` result across 21 sites. The value is identical (the mask makes the width
//!   irrelevant), but the *expression type* is not, so migrating them is a separate change
//!   against a separate crate and is not bundled here.

#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_void;

// The Win32 spellings, for use in THIS crate's signatures and doc comments. Consumers do not
// need to import them — every alias below is the same type their own island already gives them.
pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;
pub type LONG = i32;
pub type LANGID = u16;
pub type LCID = u32;
pub type COLORREF = u32;
pub type WCHAR = u16;
pub type LPWSTR = *mut u16;
pub type LPSTR = *mut i8;
pub type HANDLE = *mut c_void;

// ================= `L"..."` — a UTF-16 literal written as bytes, at 2-byte alignment ============

/// A UTF-16LE literal spelled as a byte string (`b"L\0i\0b\0\0\0"`, the form rustconv and
/// the translations write `L"Lib"` in), placed at a 2-byte-aligned address, as a `*const u16`.
///
/// `b"...".as_ptr() as LPCWSTR` compiles and usually works, and that is the trap: a byte
/// string has alignment 1, so the linker may put it at an ODD address. User-mode x86 code
/// reads it fine; the kernel does not. `ProbeForRead(p, len, sizeof(WCHAR))` raises
/// STATUS_DATATYPE_MISALIGNMENT, and ws2_32's `L"LibraryPath"` landing on 0x7be41b61 when
/// msvcrtex became Rust (2026-10-04) bugchecked the i386 guest. The bytes stay exactly as
/// written (`tools/lint_wide_strings.py` still checks their encoding); only the placement
/// changes. `tools/scan_unaligned_wide.py` flags the unaligned spelling.
#[macro_export]
macro_rules! wide_bytes {
    ($b:literal) => {{
        #[repr(C, align(2))]
        struct Aligned<const N: usize>([u8; N]);
        const BYTES: &[u8] = $b;
        static S: Aligned<{ BYTES.len() }> = Aligned(*$b);
        S.0.as_ptr() as *const u16
    }};
}

// ================= minwindef.h — word and byte extraction ====================================

/// `minwindef.h:189` — `#define LOWORD(l) ((WORD)((DWORD_PTR)(l) & 0xffff))`
///
/// Result is `WORD` (`u16`), which is what the C macro's outer cast says.
#[macro_export]
macro_rules! LOWORD {
    ($l:expr $(,)?) => {
        (($l as usize) & 0xffff) as u16
    };
}

/// `minwindef.h:190` — `#define HIWORD(l) ((WORD)(((DWORD_PTR)(l) >> 16) & 0xffff))`
#[macro_export]
macro_rules! HIWORD {
    ($l:expr $(,)?) => {
        ((($l as usize) >> 16) & 0xffff) as u16
    };
}

/// `minwindef.h:191` — `#define LOBYTE(w) ((BYTE)((DWORD_PTR)(w) & 0xff))`
#[macro_export]
macro_rules! LOBYTE {
    ($w:expr $(,)?) => {
        (($w as usize) & 0xff) as u8
    };
}

/// `minwindef.h:192` — `#define HIBYTE(w) ((BYTE)(((DWORD_PTR)(w) >> 8) & 0xff))`
#[macro_export]
macro_rules! HIBYTE {
    ($w:expr $(,)?) => {
        ((($w as usize) >> 8) & 0xff) as u8
    };
}

/// `minwindef.h:187` — `#define MAKEWORD(bLow, bHigh) ((WORD)(((BYTE)((DWORD_PTR)(bLow) & 0xff))
/// | (((WORD)((BYTE)((DWORD_PTR)(bHigh) & 0xff))) << 8)))`
#[macro_export]
macro_rules! MAKEWORD {
    ($low:expr, $high:expr $(,)?) => {
        ((($low as usize) & 0xff) as u16) | (((($high as usize) & 0xff) as u16) << 8)
    };
}

/// `minwindef.h:188` — `#define MAKELONG(wLow, wHigh) ((LONG)(((WORD)((DWORD_PTR)(wLow) &
/// 0xffff)) | (((DWORD)((WORD)((DWORD_PTR)(wHigh) & 0xffff))) << 16)))`
///
/// Result is `LONG` (`i32`), per the C macro's outer cast. Three in-tree copies (desk, intl,
/// sysdm) returned `DWORD` instead; the bit pattern is the same, so their call sites take an
/// `as u32` where they fed a `DWORD` parameter.
#[macro_export]
macro_rules! MAKELONG {
    ($low:expr, $high:expr $(,)?) => {
        (((($low as usize) & 0xffff) as u32) | (((($high as usize) & 0xffff) as u32) << 16)) as i32
    };
}

// MAKELPARAM / MAKEWPARAM / MAKELRESULT are NOT here — see "The boundary" in the module docs.
// They wrap MAKELONG in a cast to LPARAM/WPARAM/LRESULT, which are pointer-width island aliases,
// so they stay in each consumer's prelude spelled `$crate::MAKELONG!(..) as LPARAM`.

// ================= ntbasedef.h — language and locale identifiers ==============================

/// `ntbasedef.h:939` — `#define MAKELANGID(p, s) ((((USHORT)(s)) << 10) | (USHORT)(p))`
///
/// The C macro has no outer cast, so it formally yields `int` by integer promotion — but a
/// LANGID is 16 bits by definition and every caller in the tree assigns it to one, which is why
/// all three copies this replaces returned `WORD`. Result here is `LANGID` (`u16`).
#[macro_export]
macro_rules! MAKELANGID {
    ($primary:expr, $sub:expr $(,)?) => {
        (((($sub as u16) as u32) << 10) | (($primary as u16) as u32)) as u16
    };
}

/// `ntbasedef.h:940` — `#define PRIMARYLANGID(lgid) ((USHORT)(lgid) & 0x3ff)`
#[macro_export]
macro_rules! PRIMARYLANGID {
    ($lgid:expr $(,)?) => {
        (($lgid as u16) & 0x3ff) as u16
    };
}

/// `ntbasedef.h:941` — `#define SUBLANGID(lgid) ((USHORT)(lgid) >> 10)`
#[macro_export]
macro_rules! SUBLANGID {
    ($lgid:expr $(,)?) => {
        (($lgid as u16) >> 10) as u16
    };
}

/// `ntbasedef.h:942` — `#define MAKELCID(lgid, srtid) ((ULONG)(((ULONG)((USHORT)(srtid)) << 16)
/// | ((ULONG)((USHORT)(lgid)))))`
#[macro_export]
macro_rules! MAKELCID {
    ($lgid:expr, $srtid:expr $(,)?) => {
        (((($srtid as u16) as u32) << 16) | (($lgid as u16) as u32)) as u32
    };
}

/// `ntbasedef.h:947` — `#define LANGIDFROMLCID(lcid) ((USHORT)(lcid))`
#[macro_export]
macro_rules! LANGIDFROMLCID {
    ($lcid:expr $(,)?) => {
        ($lcid as u32) as u16
    };
}

/// `ntbasedef.h:948` — `#define SORTIDFROMLCID(lcid) ((USHORT)((((ULONG)(lcid)) >> 16) & 0xf))`
#[macro_export]
macro_rules! SORTIDFROMLCID {
    ($lcid:expr $(,)?) => {
        ((($lcid as u32) >> 16) & 0xf) as u16
    };
}

// ================= winuser.h — resource ids and mouse coordinates =============================

/// `winuser.h:582` — `#define MAKEINTRESOURCEW(i) ((LPWSTR)(ULONG_PTR)LOWORD(i))`
///
/// The TCHAR-neutral `MAKEINTRESOURCE` is NOT provided here: it is `-DUNICODE`-dependent and so
/// belongs to the consumer (see the module docs).
#[macro_export]
macro_rules! MAKEINTRESOURCEW {
    ($i:expr $(,)?) => {
        ((($i as usize) & 0xffff) as usize) as *mut u16
    };
}

/// `winuser.h:581` — `#define MAKEINTRESOURCEA(i) ((LPSTR)(ULONG_PTR)LOWORD(i))`
#[macro_export]
macro_rules! MAKEINTRESOURCEA {
    ($i:expr $(,)?) => {
        ((($i as usize) & 0xffff) as usize) as *mut i8
    };
}

/// `windowsx.h:299` — `#define GET_X_LPARAM(lp) ((int)(short)LOWORD(lp))`
///
/// The `(short)` is load-bearing: mouse coordinates are signed and go negative on a multi-monitor
/// desktop whose secondary display is left of the primary.
#[macro_export]
macro_rules! GET_X_LPARAM {
    ($lp:expr $(,)?) => {
        ((($lp as usize) & 0xffff) as u16 as i16) as i32
    };
}

/// `windowsx.h:300` — `#define GET_Y_LPARAM(lp) ((int)(short)HIWORD(lp))`
#[macro_export]
macro_rules! GET_Y_LPARAM {
    ($lp:expr $(,)?) => {
        (((($lp as usize) >> 16) & 0xffff) as u16 as i16) as i32
    };
}

// ================= wingdi.h — colour ==========================================================

/// `wingdi.h:3386` — `#define RGB(r,g,b) ((COLORREF)(((BYTE)(r)|((WORD)((BYTE)(g))<<8))|
/// (((DWORD)(BYTE)(b))<<16)))`
///
/// Each component is masked to `BYTE` **before** it is shifted. Every in-tree copy shifted the
/// unmasked value, which lets an out-of-range component bleed into the next channel (module docs).
#[macro_export]
macro_rules! RGB {
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        ((($r as u32) & 0xff) | ((($g as u32) & 0xff) << 8) | ((($b as u32) & 0xff) << 16)) as u32
    };
}

/// `wingdi.h:3381` — `#define GetRValue(rgb) ((BYTE)(rgb))`
#[macro_export]
macro_rules! GetRValue {
    ($rgb:expr $(,)?) => {
        (($rgb as u32) & 0xff) as u8
    };
}

/// `wingdi.h:3382` — `#define GetGValue(rgb) ((BYTE)(((WORD)(rgb)) >> 8))`
#[macro_export]
macro_rules! GetGValue {
    ($rgb:expr $(,)?) => {
        (((($rgb as u32) as u16) >> 8) & 0xff) as u8
    };
}

/// `wingdi.h:3383` — `#define GetBValue(rgb) ((BYTE)((rgb)>>16))`
#[macro_export]
macro_rules! GetBValue {
    ($rgb:expr $(,)?) => {
        ((($rgb as u32) >> 16) & 0xff) as u8
    };
}

// ================= winbase.h / crtdefs — the two odd ones =====================================

/// `winbase.h` — `#define ZeroMemory(Destination,Length) memset((Destination),0,(Length))`
///
/// Lowered to `core::ptr::write_bytes`, which LLVM emits as a `memset` call — NOT a hand-written
/// byte loop, which LLVM would fold back into a call to `memset` itself (AGENTS.md). Expands to a
/// statement, not a value: the C macro's `void*` result is never read, and all 135 call sites in
/// the tree use it as a statement.
///
/// # Safety
/// Caller must ensure `dst` is valid for `len` writes. Every call site is already inside `unsafe`.
#[macro_export]
macro_rules! ZeroMemory {
    ($dst:expr, $len:expr $(,)?) => {{
        core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize);
    }};
}

/// `winbase.h` — `#define RtlZeroMemory(Destination,Length) memset((Destination),0,(Length))`.
/// The NT spelling of [`ZeroMemory`]; both names appear in the same translated files.
///
/// # Safety
/// Same contract as [`ZeroMemory`].
#[macro_export]
macro_rules! RtlZeroMemory {
    ($dst:expr, $len:expr $(,)?) => {{
        core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize);
    }};
}

/// MSVC's `_countof(a)` — `(sizeof(a)/sizeof(a[0]))`.
///
/// Result is `u32`, not `usize`. C's is `size_t`, but this is the spelling the four crates it
/// replaces settled on because every one of the 201 call sites feeds a `DWORD`/`UINT` parameter
/// of a Win32 API. A site that wants a length takes an `as usize`.
#[macro_export]
macro_rules! _countof {
    ($a:expr $(,)?) => {
        ($a).len() as u32
    };
}

/// `winnt.h` — `#define ARRAYSIZE(A) RTL_NUMBER_OF_V2(A)`. The SAL spelling of [`_countof`],
/// same result type for the same reason.
#[macro_export]
macro_rules! ARRAYSIZE {
    ($a:expr $(,)?) => {
        ($a).len() as u32
    };
}

// ================= constants bindgen folds away ===============================================

/// `ntbasedef.h:619` — `#define UNICODE_NULL ((WCHAR)0)`. A cast expression, so bindgen drops it.
pub const UNICODE_NULL: WCHAR = 0;

/// `ndk/extypes.h:77` — `#define INVALID_HANDLE_VALUE (HANDLE)-1`. A pointer literal.
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// HWND_BROADCAST is NOT here: `HWND` is `*mut HWND__`, a nominal opaque struct that bindgen
// emits into each island separately, so a shared definition cannot unify with a consumer's.
// It stays a one-liner in each prelude. (See "The boundary" above.)

/// `winuser.h` dialog-window extra-bytes offsets. These are `sizeof` expressions, so bindgen
/// cannot fold them and **they must stay computed** — `sizeof(LRESULT)` and `sizeof(DLGPROC)` are
/// both the pointer width, so `DWLP_USER` is 8 on i386 and 16 on amd64/arm64. A baked literal is
/// correct on exactly one of our three targets.
pub const DWLP_MSGRESULT: i32 = 0;
/// See [`DWLP_MSGRESULT`]. `DWLP_MSGRESULT + sizeof(LRESULT)`.
pub const DWLP_DLGPROC: i32 = DWLP_MSGRESULT + core::mem::size_of::<isize>() as i32;
/// See [`DWLP_MSGRESULT`]. `DWLP_DLGPROC + sizeof(DLGPROC)`.
pub const DWLP_USER: i32 = DWLP_DLGPROC + core::mem::size_of::<*mut c_void>() as i32;

#[cfg(test)]
mod tests {
    //! Known-answer tests. Every expected value is computed from the C macro in the header cited
    //! on the item, by hand, not by running the Rust — a test that re-derives the answer the same
    //! way the implementation does proves nothing.

    use crate::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, INVALID_HANDLE_VALUE, UNICODE_NULL};

    #[test]
    fn word_and_byte_extraction_selects_the_documented_bits() {
        assert_eq!(LOWORD!(0x1234_5678_u32), 0x5678);
        assert_eq!(HIWORD!(0x1234_5678_u32), 0x1234);
        assert_eq!(LOBYTE!(0x1234_u32), 0x34);
        assert_eq!(HIBYTE!(0x1234_u32), 0x12);
        // The mask, not the width, is what makes the result: a 64-bit input keeps bits 0..15
        // and 16..31 exactly as a 32-bit one does.
        assert_eq!(LOWORD!(0xdead_beef_1234_5678_u64), 0x5678);
        assert_eq!(HIWORD!(0xdead_beef_1234_5678_u64), 0x1234);
    }

    #[test]
    fn make_word_and_long_pack_low_then_high() {
        assert_eq!(MAKEWORD!(0x34_u32, 0x12_u32), 0x1234_u16);
        assert_eq!(MAKELONG!(0x5678_u32, 0x1234_u32), 0x1234_5678_u32 as i32);
        // Both arguments are masked before packing, so an over-wide low cannot reach the high half.
        assert_eq!(
            MAKELONG!(0xdead_5678_u32, 0x1234_u32),
            0x1234_5678_u32 as i32
        );
        assert_eq!(MAKEWORD!(0xff34_u32, 0xff12_u32), 0x1234_u16);
    }

    #[test]
    fn make_long_is_signed_because_the_c_macro_casts_to_long() {
        // 0x8000_0000 as LONG is negative. Three in-tree copies returned DWORD and would have
        // given 2147483648 here; the header says LONG.
        let v = MAKELONG!(0x0000_u32, 0x8000_u32);
        assert_eq!(v, i32::MIN);
        assert!(v < 0);
    }

    #[test]
    fn language_identifiers_pack_sublang_into_the_high_six_bits() {
        // LANG_ENGLISH = 0x09, SUBLANG_ENGLISH_US = 0x01 -> 0x0409, the canonical en-US LANGID.
        assert_eq!(MAKELANGID!(0x09_u16, 0x01_u16), 0x0409);
        assert_eq!(PRIMARYLANGID!(0x0409_u16), 0x09);
        assert_eq!(SUBLANGID!(0x0409_u16), 0x01);
        // MAKELCID(0x0409, SORT_DEFAULT=0) is 0x0000_0409.
        assert_eq!(MAKELCID!(0x0409_u16, 0x0_u16), 0x0000_0409_u32);
        assert_eq!(LANGIDFROMLCID!(0x0004_0409_u32), 0x0409);
        assert_eq!(SORTIDFROMLCID!(0x0004_0409_u32), 0x4);
    }

    #[test]
    fn make_int_resource_keeps_only_the_low_word() {
        assert_eq!(MAKEINTRESOURCEW!(102_u32) as usize, 102);
        assert_eq!(MAKEINTRESOURCEA!(102_u32) as usize, 102);
        // A pointer this small is never dereferenced: the WORD *is* the resource id, and
        // FindResourceW tells them apart by IS_INTRESOURCE (the high bits being zero).
        assert_eq!(MAKEINTRESOURCEW!(0x1234_5678_u32) as usize, 0x5678);
    }

    #[test]
    fn mouse_coordinates_are_signed() {
        assert_eq!(GET_X_LPARAM!(0x0010_0020_usize), 0x20);
        assert_eq!(GET_Y_LPARAM!(0x0010_0020_usize), 0x10);
        // A window left of the primary monitor reports a negative x. 0xFFF0 as short is -16;
        // an unsigned port would report 65520 and place the click off-screen to the right.
        assert_eq!(GET_X_LPARAM!(0x0000_fff0_usize), -16);
        assert_eq!(GET_Y_LPARAM!(0xfff0_0000_usize), -16);
    }

    #[test]
    fn rgb_masks_each_component_before_shifting_it() {
        assert_eq!(RGB!(0x12_u32, 0x34_u32, 0x56_u32), 0x0056_3412);
        assert_eq!(GetRValue!(0x0056_3412_u32), 0x12);
        assert_eq!(GetGValue!(0x0056_3412_u32), 0x34);
        assert_eq!(GetBValue!(0x0056_3412_u32), 0x56);
    }

    #[test]
    fn an_out_of_range_component_cannot_bleed_into_the_next_channel() {
        // This is the case every in-tree copy got wrong: unmasked, 0x1FF << 8 is 0x0001_FF00,
        // so green would have overwritten blue's low bit.
        assert_eq!(RGB!(0_u32, 0x1ff_u32, 0_u32), 0x0000_ff00);
        assert_eq!(GetBValue!(RGB!(0_u32, 0x1ff_u32, 0_u32)), 0);
    }

    #[test]
    fn get_g_value_truncates_to_word_first_exactly_as_the_header_does() {
        // wingdi.h casts to WORD before >> 8. Bits above 15 must not reach the result.
        assert_eq!(GetGValue!(0xffff_3400_u32), 0x34);
    }

    #[test]
    fn zero_memory_writes_the_whole_length() {
        let mut buf = [0xAAu8; 8];
        unsafe { ZeroMemory!(buf.as_mut_ptr(), 6) };
        assert_eq!(buf, [0, 0, 0, 0, 0, 0, 0xAA, 0xAA]);
    }

    #[test]
    fn countof_is_the_element_count_not_the_byte_count() {
        let wide = [0u16; 260];
        assert_eq!(_countof!(wide), 260u32);
    }

    #[test]
    fn dwlp_offsets_track_the_pointer_width_of_the_target() {
        // The whole reason these are computed. A literal 8 is right on i386 and wrong on the
        // other two targets; a literal 16 is the reverse.
        let ptr = core::mem::size_of::<usize>() as i32;
        assert_eq!(DWLP_MSGRESULT, 0);
        assert_eq!(DWLP_DLGPROC, ptr);
        assert_eq!(DWLP_USER, 2 * ptr);
        assert_eq!(
            DWLP_USER,
            if cfg!(target_pointer_width = "32") {
                8
            } else {
                16
            }
        );
    }

    #[test]
    fn wide_bytes_are_aligned_and_unchanged() {
        // Several in a row, so at least one would land odd if alignment were 1.
        let ps = [
            wide_bytes!(b"A\0\0\0"),
            wide_bytes!(b"L\0i\0b\0r\0a\0r\0y\0P\0a\0t\0h\0\0\0"),
            wide_bytes!(b"x\0\0\0"),
        ];
        for p in ps {
            assert_eq!(p as usize % 2, 0);
        }
        let s = unsafe { core::slice::from_raw_parts(ps[1], 12) };
        let want: [u16; 12] = [76, 105, 98, 114, 97, 114, 121, 80, 97, 116, 104, 0];
        assert_eq!(s, &want);
    }

    #[test]
    fn the_folded_constants_are_the_values_their_headers_cast() {
        assert_eq!(UNICODE_NULL, 0u16);
        assert_eq!(INVALID_HANDLE_VALUE as isize, -1);
    }
}
