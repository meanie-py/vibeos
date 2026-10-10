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

/// `winuser.h` — `#define IS_INTRESOURCE(_r) ((((ULONG_PTR)(_r)) >> 16) == 0)`
///
/// `bool`, not C's `int`: the call sites test it both bare and with `!= 0`, and an `int` result
/// would let `!IS_INTRESOURCE(p)` compile as a bitwise NOT that is never zero. A `bool` turns
/// every `!= 0` site into a compile error instead.
#[macro_export]
macro_rules! IS_INTRESOURCE {
    ($r:expr $(,)?) => {
        ((($r) as usize) >> 16) == 0
    };
}

/// `winbase.h` — `#define MAKEINTATOM(i) (LPWSTR)((ULONG_PTR)((WORD)(i)))` (UNICODE).
#[macro_export]
macro_rules! MAKEINTATOM {
    ($i:expr $(,)?) => {
        ((($i) as usize) & 0xffff) as *mut u16
    };
}

/// `winuser.h` — `#define GET_WHEEL_DELTA_WPARAM(wParam) ((short)HIWORD(wParam))`
#[macro_export]
macro_rules! GET_WHEEL_DELTA_WPARAM {
    ($w:expr $(,)?) => {
        (((($w) as usize) >> 16) & 0xffff) as u16 as i16
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

/// `wingdi.h:3384` — `#define GetAValue(rgb) ((BYTE)((rgb)>>24))`
#[macro_export]
macro_rules! GetAValue {
    ($rgb:expr $(,)?) => {
        ((($rgb as u32) >> 24) & 0xff) as u8
    };
}

/// `wingdi.h:3387` — `#define RGBA(r,g,b,a) ((COLORREF)((((DWORD)(BYTE)(a))<<24) | RGB(r,g,b)))`
#[macro_export]
macro_rules! RGBA {
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => {
        (((($a as u32) & 0xff) << 24) | $crate::RGB!($r, $g, $b)) as u32
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

/// `ntdef.h` — `#define RtlCopyMemory(D, S, L) memcpy((D), (S), (L))`. The core intrinsic
/// lowers to the imported `memcpy`; that is only a hazard INSIDE a memcpy implementation.
#[macro_export]
macro_rules! RtlCopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {{
        core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $len as usize);
    }};
}

/// `winbase.h` — `#define CopyMemory RtlCopyMemory`.
#[macro_export]
macro_rules! CopyMemory {
    ($($args:tt)*) => {
        $crate::RtlCopyMemory!($($args)*)
    };
}

/// `winbase.h` — `#define MoveMemory RtlMoveMemory`.
#[macro_export]
macro_rules! MoveMemory {
    ($($args:tt)*) => {
        $crate::RtlMoveMemory!($($args)*)
    };
}

/// `ntdef.h` — `#define RtlMoveMemory(D, S, L) memmove((D), (S), (L))`: overlap allowed.
#[macro_export]
macro_rules! RtlMoveMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {{
        core::ptr::copy($src as *const u8, $dst as *mut u8, $len as usize);
    }};
}

/// `ntdef.h` — `#define RtlFillMemory(D, L, F) memset((D), (F), (L))`. NB the argument order:
/// LENGTH before FILL, unlike memset.
#[macro_export]
macro_rules! RtlFillMemory {
    ($dst:expr, $len:expr, $fill:expr $(,)?) => {{
        core::ptr::write_bytes($dst as *mut u8, ($fill) as u8, $len as usize);
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

/// Wine's `#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))` — the Wine-derived DLLs'
/// spelling of [`ARRAYSIZE`], `u32` for the same reason: comctl32's 63 sites feed `UINT`/`INT`.
#[macro_export]
macro_rules! ARRAY_SIZE {
    ($a:expr $(,)?) => {
        ($a).len() as u32
    };
}

/// `ntbasedef.h` — `#define RTL_NUMBER_OF(A) (sizeof(A)/sizeof((A)[0]))` (`RTL_NUMBER_OF_V1`).
/// The kernel's spelling of [`ARRAYSIZE`], and `u32` for the same reason as [`_countof`]: the
/// kernel call sites compare it against `ULONG` indices (pciidex's chipset tables: 14 sites).
#[macro_export]
macro_rules! RTL_NUMBER_OF {
    ($a:expr $(,)?) => {
        ($a).len() as u32
    };
}

/// `ntbasedef.h:715` — `#define RTL_NUMBER_OF_FIELD(type, field)
/// (RTL_NUMBER_OF(RTL_FIELD_TYPE(type, field)))`: the element count of an array FIELD, taken
/// from the type alone. C uses it as an array bound (setup's `WCHAR PathBuffer[
/// RTL_NUMBER_OF_FIELD(VOLINFO, DeviceName) + 1]`), so this is const-evaluable and `usize`.
/// No instance is read: the field's address is formed in an uninitialised slot, never a null.
#[macro_export]
macro_rules! RTL_NUMBER_OF_FIELD {
    (@field $t:ty, $($f:tt)+) => {{
        const fn __len<T, const N: usize>(_: *const [T; N]) -> usize {
            N
        }
        let slot = core::mem::MaybeUninit::<$t>::uninit();
        __len(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($f)+) })
    }};
    // One field name, with the trailing comma rustfmt adds to a call it wraps over lines
    // (usetup's `RTL_NUMBER_OF_FIELD(\n VOLINFO,\n DeviceName,\n)`), which the `tt` arm
    // below would swallow into the field path.
    ($t:ty, $f:ident $(,)?) => {
        $crate::RTL_NUMBER_OF_FIELD!(@field $t, $f)
    };
    ($t:ty, $($f:tt)+) => {
        $crate::RTL_NUMBER_OF_FIELD!(@field $t, $($f)+)
    };
}

/// The pieces `DPRINT1!` expands to. Not for direct use.
#[doc(hidden)]
pub mod dprint {
    extern "C" {
        /// ntdll's `ULONG __cdecl DbgPrint(PCCH Format, ...)`.
        pub fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
    }

    /// A DPRINT1 format, NUL-terminated: used in place if it already is, else copied
    /// (truncated) into the caller's stack buffer. Translations spell the format three ways
    /// (a str literal, a byte literal, a pointer).
    pub trait DprintFmt {
        fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char;
    }
    fn terminated(bytes: &[u8], buf: &mut [u8; 512]) -> *const core::ffi::c_char {
        if bytes.last() == Some(&0) {
            return bytes.as_ptr() as *const core::ffi::c_char;
        }
        let n = bytes.len().min(buf.len() - 1);
        buf[..n].copy_from_slice(&bytes[..n]);
        buf[n] = 0;
        buf.as_ptr() as *const core::ffi::c_char
    }
    impl DprintFmt for &'static str {
        fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char {
            terminated(self.as_bytes(), buf)
        }
    }
    impl<const N: usize> DprintFmt for &'static [u8; N] {
        fn fmt_ptr(self, buf: &mut [u8; 512]) -> *const core::ffi::c_char {
            terminated(self, buf)
        }
    }
    impl DprintFmt for *const i8 {
        fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
            self as *const core::ffi::c_char
        }
    }
    impl DprintFmt for *const u8 {
        fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
            self as *const core::ffi::c_char
        }
    }
    impl DprintFmt for *mut i8 {
        fn fmt_ptr(self, _: &mut [u8; 512]) -> *const core::ffi::c_char {
            self as *const core::ffi::c_char
        }
    }
}

/// `reactos/debug.h` `DPRINT1` as this tree builds it (`-DDBG=1`): `if (DbgPrint("(%s:%d) "
/// fmt, __RELFILE__, __LINE__, ...)) DbgPrint("(%s:%d) DbgPrint() failed!\n", ..)`, the prefix
/// printed separately because Rust cannot paste it onto a runtime format. The file is the
/// Rust one. A MACRO, never an out-of-line fn (an out-of-line no-op is an optimization
/// barrier). setuplib carried this by hand; every crate `rustconv port` scaffolded after it
/// got a no-op, and the deferred lint (`scan_dropped_prints`) caught two of them.
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __dprint1_buf = [0u8; 512];
        let __dprint1_fmt = $crate::dprint::DprintFmt::fmt_ptr($fmt, &mut __dprint1_buf);
        let __dprint1_file = concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char;
        $crate::dprint::DbgPrint(b"(%s:%d) \0".as_ptr() as *const core::ffi::c_char, __dprint1_file, line!());
        if $crate::dprint::DbgPrint(__dprint1_fmt $(, $arg)*) != 0 {
            $crate::dprint::DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                __dprint1_file,
                line!(),
            );
        }
    }};
}

/// `guiddef.h:147` — `#define IsEqualGUID(rguid1, rguid2) (!memcmp(rguid1, rguid2,
/// sizeof(GUID)))`, for C callers (C++ gets an inline operator). `GUID` is 16 bytes on every
/// target, so the shared crate needs no GUID type: each argument may be a reference or a raw
/// pointer to one. Eight crates had re-ported this by hand, three with different signatures.
#[macro_export]
macro_rules! IsEqualGUID {
    ($a:expr, $b:expr $(,)?) => {{
        let a = $a as *const _ as *const u8;
        let b = $b as *const _ as *const u8;
        #[allow(unused_unsafe)]
        unsafe {
            core::slice::from_raw_parts(a, 16) == core::slice::from_raw_parts(b, 16)
        }
    }};
}

/// `ntbasedef.h:110` — `#define FIELD_OFFSET(Type, Field) ((LONG)__builtin_offsetof(Type,
/// Field))`. `LONG` is `i32` on every target (LLP64), so the result type is fixed.
#[macro_export]
macro_rules! FIELD_OFFSET {
    // `FIELD_OFFSET(DEVICE_RELATIONS, Objects[Count])` -- the size of a variable-length
    // struct holding `Count` elements. `offset_of!` refuses an index, so add the element
    // stride by hand: offsetof(field) + index * sizeof(element), as clang folds it.
    ($t:ty, $($f:ident).+ [ $idx:expr ]) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$t>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($f).+) });
        ((core::mem::offset_of!($t, $($f).+) + ($idx) as usize * stride) as i32)
    }};
    ($t:ty, $($f:ident).+ $(,)?) => {
        (core::mem::offset_of!($t, $($f).+) as i32)
    };
    // An index INSIDE the path (`List[0].PartialResourceList.PartialDescriptors`):
    // offset_of! refuses it, so take the member's address in an uninitialised slot and
    // subtract the base. No reference is formed and nothing is read.
    ($t:ty, $($f:tt)+) => {{
        let slot = core::mem::MaybeUninit::<$t>::uninit();
        let base = slot.as_ptr();
        let field = unsafe { core::ptr::addr_of!((*base).$($f)+) };
        ((field as *const u8 as usize) - (base as *const u8 as usize)) as i32
    }};
}

/// `ntbasedef.h:682` — `#define RTL_FIELD_SIZE(type, field) (sizeof(((type *)0)->field))`, a
/// `size_t`. Const-evaluable; the field's address is formed in an uninitialised slot.
#[macro_export]
macro_rules! RTL_FIELD_SIZE {
    ($t:ty, $($f:tt)+) => {{
        const fn __size<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$t>::uninit();
        __size(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($f)+) })
    }};
}

/// `ntbasedef.h:684` — `FIELD_OFFSET(type, field) + RTL_FIELD_SIZE(type, field)`: `LONG +
/// size_t` is a `size_t` in C, so this is `usize`.
#[macro_export]
macro_rules! RTL_SIZEOF_THROUGH_FIELD {
    // Through FIELD_OFFSET!, which also takes a path with an index in it.
    ($t:ty, $($f:tt)+) => {
        ($crate::FIELD_OFFSET!($t, $($f)+) as usize + $crate::RTL_FIELD_SIZE!($t, $($f)+))
    };
}

/// `ndk/rtlfuncs.h` — `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)`. The one
/// macro here that names something outside `core`: `NtCurrentPeb` resolves at the CALL SITE,
/// which every user-mode crate has through its island's `ntdll_sys::nt` glob. A macro rather
/// than a fn so it can never collide with the `fn RtlGetProcessHeap` several crates already
/// define (a different namespace), and `port resolve` does not import it into those.
#[macro_export]
macro_rules! RtlGetProcessHeap {
    () => {
        (*NtCurrentPeb()).ProcessHeap
    };
}

/// `ntbasedef.h:329` — `#define UNREFERENCED_PARAMETER(P) ((void)(P))`.
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),+ $(,)?) => {{
        $(let _ = &$p;)+
    }};
}

/// `ntdddisk.h:315` — `IsFTPartition(t)`: the NTFT bit (0x80) set and, with the VALID_NTFT
/// bits (0xC0) cleared, HUGE (0x06), IFS (0x07), FAT32 (0x0B) or FAT32_XINT13 (0x0C).
/// The partition-type macros take a `UCHAR` and are spelled with literal values, never the
/// `PARTITION_*` names, which would resolve in the caller's scope (see the top of the file).
#[macro_export]
macro_rules! IsFTPartition {
    ($t:expr $(,)?) => {{
        let t = ($t) as u8;
        (t & 0x80) != 0 && matches!(t & !0xC0, 0x06 | 0x07 | 0x0B | 0x0C)
    }};
}

/// `ntdddisk.h:321` — EXTENDED (0x05) or XINT13_EXTENDED (0x0F).
#[macro_export]
macro_rules! IsContainerPartition {
    ($t:expr $(,)?) => {
        matches!(($t) as u8, 0x05 | 0x0F)
    };
}

/// `ntdddisk.h:326`, the `__REACTOS__` arm (the tree always builds it): IsFTPartition, or FAT_12
/// 0x01, FAT_16 0x04, HUGE 0x06, IFS 0x07, FAT32 0x0B, FAT32_XINT13 0x0C, XINT13 0x0E,
/// LINUX 0x83, OLD_LINUX 0x43, ISO9660 0x96, FREEBSD 0xA5, OPENBSD 0xA6, NETBSD 0xA9.
#[macro_export]
macro_rules! IsRecognizedPartition {
    ($t:expr $(,)?) => {{
        let t = ($t) as u8;
        $crate::IsFTPartition!(t)
            || matches!(
                t,
                0x01 | 0x04
                    | 0x06
                    | 0x07
                    | 0x0B
                    | 0x0C
                    | 0x0E
                    | 0x83
                    | 0x43
                    | 0x96
                    | 0xA5
                    | 0xA6
                    | 0xA9
            )
    }};
}

/// `ntdef.h` / `obtypes.h` — `InitializeObjectAttributes(p, n, a, r, s)`, a BLOCK macro that
/// fills an `OBJECT_ATTRIBUTES` field by field. It stays a macro: it writes through the
/// destination pointer, so a function would need a `&mut` and change every call site. Hand
/// ported into kernel32 and ntmarta before this. `Length` is `size_of_val` of the pointee, so
/// the body never names `OBJECT_ATTRIBUTES` (see the hygiene note at the top); `p` is
/// evaluated once, where C evaluates it six times — the same for every side-effect-free `p`.
#[macro_export]
macro_rules! InitializeObjectAttributes {
    ($p:expr, $n:expr, $a:expr, $r:expr, $s:expr $(,)?) => {{
        let p = $p;
        (*p).Length = core::mem::size_of_val(&*p) as _;
        (*p).RootDirectory = $r;
        (*p).Attributes = $a as _;
        (*p).ObjectName = $n;
        (*p).SecurityDescriptor = $s;
        (*p).SecurityQualityOfService = core::ptr::null_mut();
    }};
}

// ================= ntdef.h / debug.h — assertions, strings, pointer and page helpers ==========

/// `debug.h` — `ASSERT(exp)`. Inert, matching the release (`!DBG`) build where the header
/// defines it to `((VOID)0)`: the expression is TYPE-CHECKED but never evaluated, so a body
/// that asserts on a name it never defines still fails to compile, and no side effect of the
/// expression runs. Any scalar C would accept is accepted (`ASSERT(FALSE)`, `ASSERT(p)`), which
/// `debug_assert!` (bool only) would not. A `macro_rules!`, never an out-of-line no-op `fn`
/// (AGENTS.md: a no-op call is an optimisation barrier that uncloaks ABI mismatches).
#[macro_export]
macro_rules! ASSERT {
    ($e:expr $(,)?) => {{
        if false {
            let _ = &$e;
        }
    }};
}

/// `ntdef.h` — `C_ASSERT(e)`: a compile-time assertion, usable as a statement in a body.
#[macro_export]
macro_rules! C_ASSERT {
    ($e:expr $(,)?) => {
        const _: () = assert!($e);
    };
}

/// A wide string `RTL_CONSTANT_STRING!` can measure: a `[u16; N]` array (C's `sizeof`, NULs
/// included, exactly) or a NUL-terminated pointer (walked to its NUL — what `sizeof` of the
/// literal it points at gives, for a literal with no embedded NUL).
pub trait WideSource {
    fn wide_ptr(&self) -> *mut u16;
    /// Bytes C's `sizeof(s)` reports: the terminator included.
    fn wide_bytes(&self) -> usize;
}

impl<const N: usize> WideSource for [u16; N] {
    fn wide_ptr(&self) -> *mut u16 {
        self.as_ptr() as *mut u16
    }
    fn wide_bytes(&self) -> usize {
        core::mem::size_of_val(self)
    }
}

impl WideSource for *const u16 {
    fn wide_ptr(&self) -> *mut u16 {
        *self as *mut u16
    }
    fn wide_bytes(&self) -> usize {
        let mut n = 0usize;
        // SAFETY: RTL_CONSTANT_STRING is only ever applied to a NUL-terminated literal.
        unsafe {
            while *self.add(n) != 0 {
                n += 1;
            }
        }
        (n + 1) * core::mem::size_of::<u16>()
    }
}

impl WideSource for *mut u16 {
    fn wide_ptr(&self) -> *mut u16 {
        *self
    }
    fn wide_bytes(&self) -> usize {
        (*self as *const u16).wide_bytes()
    }
}

/// `ntdef.h` — `RTL_CONSTANT_STRING(s)` = `{ sizeof(s) - sizeof((s)[0]), sizeof(s), (PWSTR)(s) }`.
/// `Length` excludes the terminator and `MaximumLength` includes it: a hand-inlined copy in
/// setuplib counted the NUL in both. Builds the CALLER's `_UNICODE_STRING` (bindgen's struct
/// tag, emitted into every island) — the one type this crate cannot share, see "The boundary".
#[macro_export]
macro_rules! RTL_CONSTANT_STRING {
    ($s:expr $(,)?) => {{
        // BORROW the place: `let src = $s;` would copy an array argument, and Buffer would
        // point at that stack copy (the known-answer test below caught exactly that).
        let src = &$s;
        let bytes = $crate::WideSource::wide_bytes(src);
        _UNICODE_STRING {
            Length: (bytes - core::mem::size_of::<u16>()) as u16,
            MaximumLength: bytes as u16,
            Buffer: $crate::WideSource::wide_ptr(src) as _,
        }
    }};
}

/// `basetsd.h` — `UlongToPtr(ul)` = `(PVOID)(ULONG_PTR)(ul)`: zero-extended, never sign-extended.
#[macro_export]
macro_rules! UlongToPtr {
    ($ul:expr $(,)?) => {
        (($ul) as u32 as usize as *mut core::ffi::c_void)
    };
}

// basetsd.h's integer <-> pointer conversions. On i386 these are MACROS, on _WIN64 `static
// inline` functions with the same bodies; the results are `void*` / 32-bit integers on every
// arch, so one definition serves all three. Signed sources SIGN-extend through `INT_PTR` /
// `LONG_PTR`; unsigned ones zero-extend through `UINT_PTR` / `ULONG_PTR`.

/// `IntToPtr(i)` = `((VOID*)(INT_PTR)((int)i))`.
#[macro_export]
macro_rules! IntToPtr {
    ($i:expr $(,)?) => {
        (($i) as i32 as isize as *mut core::ffi::c_void)
    };
}

/// `LongToPtr(l)` = `((VOID*)(LONG_PTR)((long)l))` (`long` is 32-bit on Windows).
#[macro_export]
macro_rules! LongToPtr {
    ($l:expr $(,)?) => {
        (($l) as i32 as isize as *mut core::ffi::c_void)
    };
}

/// `UIntToPtr(ui)` = `((VOID*)(UINT_PTR)((unsigned int)ui))`; `UintToPtr` is an alias.
#[macro_export]
macro_rules! UIntToPtr {
    ($ui:expr $(,)?) => {
        (($ui) as u32 as usize as *mut core::ffi::c_void)
    };
}

/// basetsd.h `#define UintToPtr(ui) UIntToPtr(ui)`.
#[macro_export]
macro_rules! UintToPtr {
    ($ui:expr $(,)?) => {
        $crate::UIntToPtr!($ui)
    };
}

/// `ULongToPtr(ul)` = `((VOID*)(ULONG_PTR)((unsigned long)ul))`.
#[macro_export]
macro_rules! ULongToPtr {
    ($ul:expr $(,)?) => {
        (($ul) as u32 as usize as *mut core::ffi::c_void)
    };
}

/// `ULongToHandle(h)` = `((HANDLE)(ULONG_PTR)(h))`; `UlongToHandle` is an alias.
#[macro_export]
macro_rules! ULongToHandle {
    ($h:expr $(,)?) => {
        (($h) as u32 as usize as *mut core::ffi::c_void)
    };
}

/// basetsd.h `#define UlongToHandle(ul) ULongToHandle(ul)`.
#[macro_export]
macro_rules! UlongToHandle {
    ($h:expr $(,)?) => {
        $crate::ULongToHandle!($h)
    };
}

/// `LongToHandle(h)` = `((HANDLE)(LONG_PTR)(h))`.
#[macro_export]
macro_rules! LongToHandle {
    ($h:expr $(,)?) => {
        (($h) as i32 as isize as *mut core::ffi::c_void)
    };
}

/// `PtrToInt(p)` = `((INT)(INT_PTR)(p))`; `PtrToLong` is the same width on Windows.
#[macro_export]
macro_rules! PtrToInt {
    ($p:expr $(,)?) => {
        (($p) as isize as i32)
    };
}

/// `PtrToLong(p)` = `((LONG)(LONG_PTR)(p))`.
#[macro_export]
macro_rules! PtrToLong {
    ($p:expr $(,)?) => {
        (($p) as isize as i32)
    };
}

/// `PtrToUint(p)` = `((UINT)(UINT_PTR)(p))`.
#[macro_export]
macro_rules! PtrToUint {
    ($p:expr $(,)?) => {
        (($p) as usize as u32)
    };
}

/// `HandleToUlong(h)` = `((ULONG)(ULONG_PTR)(h))`.
#[macro_export]
macro_rules! HandleToUlong {
    ($h:expr $(,)?) => {
        (($h) as usize as u32)
    };
}

/// `ndk/mmtypes.h` — `PAGE_ROUND_UP(x)` = `(((ULONG_PTR)(x) + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1))`.
/// The arithmetic is `ULONG_PTR` (`usize`) and so is the result; `PAGE_SIZE` is the caller's.
#[macro_export]
macro_rules! PAGE_ROUND_UP {
    ($x:expr $(,)?) => {
        ((($x) as usize).wrapping_add(PAGE_SIZE as usize - 1) & !(PAGE_SIZE as usize - 1))
    };
}

/// `basetsd.h` — `PtrToUlong(p)` = `(ULONG)(ULONG_PTR)(p)`: the low 32 bits of the pointer.
#[macro_export]
macro_rules! PtrToUlong {
    ($p:expr $(,)?) => {
        (($p) as usize as u32)
    };
}

/// `ntbasedef.h` — `DBG_UNREFERENCED_LOCAL_VARIABLE(L)` = `((void)(L))`.
#[macro_export]
macro_rules! DBG_UNREFERENCED_LOCAL_VARIABLE {
    ($l:expr $(,)?) => {{
        let _ = &$l;
    }};
}

/// `mmfuncs.h` — `ALIGN_UP(size, type)` = `ALIGN_UP_BY(size, sizeof(type))`, rounding `size` up
/// to a multiple of `sizeof(type)` in `ULONG_PTR` arithmetic.
///
/// The second argument is a TYPE in C. A translation that already lowered it to
/// `core::mem::size_of::<T>() as U` is accepted too (the first arms): atapi's port wrote exactly
/// that, and it names the same alignment.
#[macro_export]
macro_rules! ALIGN_UP {
    ($size:expr, core::mem::size_of::<$t:ty>() $(as $u:ty)? $(,)?) => {
        $crate::ALIGN_UP!($size, $t)
    };
    ($size:expr, size_of::<$t:ty>() $(as $u:ty)? $(,)?) => {
        $crate::ALIGN_UP!($size, $t)
    };
    ($size:expr, $t:ty $(,)?) => {
        ((($size) as usize).wrapping_add(core::mem::size_of::<$t>() - 1)
            & !(core::mem::size_of::<$t>() - 1))
    };
}

/// `ntbasedef.h` — `RTL_CONTAINS_FIELD(Struct, Size, Field)`: does a buffer of `Size` bytes at
/// `Struct` reach the END of `Field`? C's comparison is an `int`, so this is `i32` 0/1.
/// `Struct` is evaluated more than once, as in C.
#[macro_export]
macro_rules! RTL_CONTAINS_FIELD {
    ($s:expr, $size:expr, $($f:ident).+ $(,)?) => {{
        const fn __size<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        let field = core::ptr::addr_of!((*$s).$($f).+);
        ((field as *const u8).wrapping_add(__size(field))
            <= ($s as *const u8).wrapping_add(($size) as usize)) as i32
    }};
}

// ================= srb.h / ata.h — storage request macros =====================================
// Each yields C's own result type (`int` for SRB_STATUS, the `(UCHAR)` cast for IDE_SET_*);
// the conversion C does implicitly at the use (`Mode = IDE_SET_UDMA_MODE(m)` stores a UCHAR
// into a ULONG) is the CALL SITE's, which crate-repair casts there. An `as _` here could not
// infer under `match SRB_STATUS!(s) { .. }`.

/// `srb.h` — `SRB_STATUS(Status)`: the status with its two flag bits cleared,
/// SRB_STATUS_AUTOSENSE_VALID (`0x80`) and SRB_STATUS_QUEUE_FROZEN (`0x40`). `int` arithmetic in C.
#[macro_export]
macro_rules! SRB_STATUS {
    ($s:expr $(,)?) => {
        ((($s) as i32) & !(0x80 | 0x40))
    };
}

/// `ata.h` — `IDE_SET_DEFAULT_PIO_MODE(mode)` = `(UCHAR)1`; `mode` is not evaluated.
#[macro_export]
macro_rules! IDE_SET_DEFAULT_PIO_MODE {
    ($m:expr $(,)?) => {
        1u8
    };
}

/// `ata.h` — `IDE_SET_ADVANCE_PIO_MODE(mode)` = `(UCHAR)((1 << 3) | (mode))`.
#[macro_export]
macro_rules! IDE_SET_ADVANCE_PIO_MODE {
    ($m:expr $(,)?) => {
        (((1 << 3) | ($m)) as u8)
    };
}

/// `ata.h` — `IDE_SET_SWDMA_MODE(mode)` = `(UCHAR)((1 << 4) | (mode))`.
#[macro_export]
macro_rules! IDE_SET_SWDMA_MODE {
    ($m:expr $(,)?) => {
        (((1 << 4) | ($m)) as u8)
    };
}

/// `ata.h` — `IDE_SET_MWDMA_MODE(mode)` = `(UCHAR)((1 << 5) | (mode))`.
#[macro_export]
macro_rules! IDE_SET_MWDMA_MODE {
    ($m:expr $(,)?) => {
        (((1 << 5) | ($m)) as u8)
    };
}

/// `ata.h` — `IDE_SET_UDMA_MODE(mode)` = `(UCHAR)((1 << 6) | (mode))`.
#[macro_export]
macro_rules! IDE_SET_UDMA_MODE {
    ($m:expr $(,)?) => {
        (((1 << 6) | ($m)) as u8)
    };
}

// ================= constants bindgen folds away ===============================================

/// `ntbasedef.h:619` — `#define UNICODE_NULL ((WCHAR)0)`. A cast expression, so bindgen drops it.
pub const UNICODE_NULL: WCHAR = 0;

/// `ndk/extypes.h:77` — `#define INVALID_HANDLE_VALUE (HANDLE)-1`. A pointer literal.
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// ================= CRT function-like macros ===================================================
//
// In C these are macros, so no DLL exports them under these names and resolve will not declare
// them `extern`; a module that calls them got nothing at all. They are functions here, not
// `macro_rules!`, because the translated C spells them as calls (`iswspace(c)`, `min(a, b)`).

/// `minmax.h` / `stdlib.h` — `#define min(a,b) (((a) < (b)) ? (a) : (b))`. Generic because the
/// macro is: the result has the CALLER's type, so nothing width-dependent is shared.
///
/// `extern "C"`, not the Rust ABI: i386 user code builds without SSE, where the Rust ABI
/// cannot pass an f64 at all and rejects the `min::<f64>` instantiation at codegen (opengl32,
/// winegdi). The C ABI passes it on the stack (field manual, "i386 no-SSE float ABI").
#[inline]
pub extern "C" fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// `minmax.h` / `stdlib.h` — `#define max(a,b) (((a) > (b)) ? (a) : (b))`.
#[inline]
pub extern "C" fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ================= wine/heap.h =================================================================
//
// `static inline` in the header, so no DLL exports them and every wine-derived crate carried a
// hand copy (wldap32, advapi32, ...). One copy here. `SIZE_T` is `usize` on every target.

extern "system" {
    // kernel32. Private, like iswctype below: a consumer's own declarations stay its own.
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(heap: *mut core::ffi::c_void, flags: u32, bytes: usize) -> *mut core::ffi::c_void;
    fn HeapReAlloc(
        heap: *mut core::ffi::c_void,
        flags: u32,
        mem: *mut core::ffi::c_void,
        bytes: usize,
    ) -> *mut core::ffi::c_void;
    fn HeapFree(heap: *mut core::ffi::c_void, flags: u32, mem: *mut core::ffi::c_void) -> i32;
}

/// `winnt.h` — `#define HEAP_ZERO_MEMORY 8`.
const HEAP_ZERO_MEMORY: u32 = 0x8;

/// `wine/heap.h` — `HeapAlloc(GetProcessHeap(), 0, len)`.
#[inline]
pub unsafe fn heap_alloc(len: usize) -> *mut core::ffi::c_void {
    unsafe { HeapAlloc(GetProcessHeap(), 0, len) }
}

/// `wine/heap.h` — `HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)`.
#[inline]
pub unsafe fn heap_alloc_zero(len: usize) -> *mut core::ffi::c_void {
    unsafe { HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len) }
}

/// `wine/heap.h` — a NULL `mem` allocates, as `realloc` does.
#[inline]
pub unsafe fn heap_realloc(mem: *mut core::ffi::c_void, len: usize) -> *mut core::ffi::c_void {
    if mem.is_null() {
        return unsafe { HeapAlloc(GetProcessHeap(), 0, len) };
    }
    unsafe { HeapReAlloc(GetProcessHeap(), 0, mem, len) }
}

/// `wine/heap.h` — returns nothing; HeapFree's BOOL is dropped, as in the C.
#[inline]
pub unsafe fn heap_free(mem: *mut core::ffi::c_void) {
    unsafe { HeapFree(GetProcessHeap(), 0, mem) };
}

/// `wine/heap.h` — NULL when `count * size` overflows. `checked_mul` is the C's
/// `size && len / size != count` test.
#[inline]
pub unsafe fn heap_calloc(count: usize, size: usize) -> *mut core::ffi::c_void {
    match count.checked_mul(size) {
        Some(len) => unsafe { HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len) },
        None => core::ptr::null_mut(),
    }
}

// ================= wine/debug.h string helpers ==================================================
//
// `static __inline` over libwine's `wine_dbgstr_wn`/`wine_dbgstr_an`, so they link only into a
// module that links `wine` -- which every module that names them does (they are wine/debug.h).
// A per-crate `extern` copy of `debugstr_w` (the consensus route) links to nothing: inetcpl.

extern "C" {
    fn wine_dbgstr_wn(s: *const u16, n: i32) -> *const core::ffi::c_char;
    fn wine_dbgstr_an(s: *const core::ffi::c_char, n: i32) -> *const core::ffi::c_char;
}

/// `wine/debug.h` — `wine_dbgstr_wn(s, n)`.
#[inline]
pub unsafe fn debugstr_wn(s: *const u16, n: i32) -> *const core::ffi::c_char {
    unsafe { wine_dbgstr_wn(s, n) }
}

/// `wine/debug.h` — `wine_dbgstr_wn(s, -1)`.
#[inline]
pub unsafe fn debugstr_w(s: *const u16) -> *const core::ffi::c_char {
    unsafe { wine_dbgstr_wn(s, -1) }
}

/// `wine/debug.h` — `wine_dbgstr_an(s, n)`.
#[inline]
pub unsafe fn debugstr_an(s: *const core::ffi::c_char, n: i32) -> *const core::ffi::c_char {
    unsafe { wine_dbgstr_an(s, n) }
}

/// `wine/debug.h` — `wine_dbgstr_an(s, -1)`.
#[inline]
pub unsafe fn debugstr_a(s: *const core::ffi::c_char) -> *const core::ffi::c_char {
    unsafe { wine_dbgstr_an(s, -1) }
}

extern "C" {
    // `int __cdecl iswctype(wint_t, wctype_t)`, exported by both ntdll and msvcrt. Private:
    // consumers that declare it themselves keep their own declaration.
    fn iswctype(c: u16, ty: u16) -> i32;
}

// `ctype.h:65-76` class masks.
const _UPPER: u16 = 0x1;
const _LOWER: u16 = 0x2;
const _DIGIT: u16 = 0x4;
const _SPACE: u16 = 0x8;
const _PUNCT: u16 = 0x10;
const _CONTROL: u16 = 0x20;
const _BLANK: u16 = 0x40;
const _HEX: u16 = 0x80;
const _ALPHA: u16 = 0x0100 | _UPPER | _LOWER;

/// `ctype.h:664` — `#define iswalpha(_c) (iswctype(_c,_ALPHA))`. `wint_t` is `u16`.
#[inline]
pub unsafe fn iswalpha(c: u16) -> i32 {
    unsafe { iswctype(c, _ALPHA) }
}

/// `ctype.h:665` — `iswctype(_c,_UPPER)`.
#[inline]
pub unsafe fn iswupper(c: u16) -> i32 {
    unsafe { iswctype(c, _UPPER) }
}

/// `ctype.h:666` — `iswctype(_c,_LOWER)`.
#[inline]
pub unsafe fn iswlower(c: u16) -> i32 {
    unsafe { iswctype(c, _LOWER) }
}

/// `ctype.h:667` — `iswctype(_c,_DIGIT)`.
#[inline]
pub unsafe fn iswdigit(c: u16) -> i32 {
    unsafe { iswctype(c, _DIGIT) }
}

/// `ctype.h:668` — `iswctype(_c,_HEX)`.
#[inline]
pub unsafe fn iswxdigit(c: u16) -> i32 {
    unsafe { iswctype(c, _HEX) }
}

/// `ctype.h:669` — `iswctype(_c,_SPACE)`.
#[inline]
pub unsafe fn iswspace(c: u16) -> i32 {
    unsafe { iswctype(c, _SPACE) }
}

/// `ctype.h:670` — `iswctype(_c,_PUNCT)`.
#[inline]
pub unsafe fn iswpunct(c: u16) -> i32 {
    unsafe { iswctype(c, _PUNCT) }
}

/// `ctype.h:671` — `iswctype(_c,_ALPHA|_DIGIT)`.
#[inline]
pub unsafe fn iswalnum(c: u16) -> i32 {
    unsafe { iswctype(c, _ALPHA | _DIGIT) }
}

/// `ctype.h:672` — `iswctype(_c,_BLANK|_PUNCT|_ALPHA|_DIGIT)`.
#[inline]
pub unsafe fn iswprint(c: u16) -> i32 {
    unsafe { iswctype(c, _BLANK | _PUNCT | _ALPHA | _DIGIT) }
}

/// `ctype.h:673` — `iswctype(_c,_PUNCT|_ALPHA|_DIGIT)`.
#[inline]
pub unsafe fn iswgraph(c: u16) -> i32 {
    unsafe { iswctype(c, _PUNCT | _ALPHA | _DIGIT) }
}

/// `ctype.h:674` — `iswctype(_c,_CONTROL)`.
#[inline]
pub unsafe fn iswcntrl(c: u16) -> i32 {
    unsafe { iswctype(c, _CONTROL) }
}

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

    #[allow(non_camel_case_types, non_snake_case)]
    #[derive(Debug, PartialEq)]
    struct _UNICODE_STRING {
        Length: u16,
        MaximumLength: u16,
        Buffer: *mut u16,
    }

    #[test]
    fn rtl_constant_string_excludes_the_nul_from_length_only() {
        // L"ab" is 3 WCHARs: sizeof = 6, Length = 6 - 2 = 4.
        static W: [u16; 3] = [b'a' as u16, b'b' as u16, 0];
        let a = RTL_CONSTANT_STRING!(W);
        assert_eq!((a.Length, a.MaximumLength), (4, 6));
        assert_eq!(a.Buffer as *const u16, W.as_ptr());
        // The pointer form the translations pass (`X.as_ptr()`) measures the same.
        let p = RTL_CONSTANT_STRING!(W.as_ptr());
        assert_eq!((p.Length, p.MaximumLength), (4, 6));
    }

    #[test]
    fn assert_type_checks_but_never_evaluates() {
        let mut hits = 0;
        let mut bump = || {
            hits += 1;
            0
        };
        ASSERT!(bump() == 1);
        ASSERT!(0);
        assert_eq!(hits, 0);
        C_ASSERT!(core::mem::size_of::<u16>() == 2);
    }

    #[test]
    fn page_round_up_and_ulong_to_ptr() {
        const PAGE_SIZE: u32 = 0x1000;
        assert_eq!(PAGE_ROUND_UP!(0u32), 0);
        assert_eq!(PAGE_ROUND_UP!(1u32), 0x1000);
        assert_eq!(PAGE_ROUND_UP!(0x1000u32), 0x1000);
        assert_eq!(PAGE_ROUND_UP!(0x1001u32), 0x2000);
        // Zero-extended: ULONG 0xFFFFFFFF is never a sign-extended -1 pointer.
        assert_eq!(UlongToPtr!(0xFFFF_FFFFu32) as usize, 0xFFFF_FFFFusize);
    }

    #[test]
    fn ddk_pointer_alignment_and_field_macros() {
        let p = 0x1_2345_6789usize as *const core::ffi::c_void;
        assert_eq!(PtrToUlong!(p), 0x2345_6789);
        // ALIGN_UP(5, ULONG) = 8; ALIGN_UP(8, ULONG) = 8; the size_of spelling is the same.
        assert_eq!(ALIGN_UP!(5u32, u32), 8);
        assert_eq!(ALIGN_UP!(8u32, u32), 8);
        assert_eq!(ALIGN_UP!(37usize, core::mem::size_of::<u32>() as u32), 40);
        #[repr(C)]
        struct S {
            a: u32,
            b: u16,
        }
        let s = S { a: 0, b: 0 };
        let ps: *const S = &s;
        let x = 0;
        DBG_UNREFERENCED_LOCAL_VARIABLE!(x);
        // `b` ends at byte 6: a 6-byte buffer contains it, a 5-byte one does not.
        unsafe {
            assert_eq!(RTL_CONTAINS_FIELD!(ps, 6u32, b), 1);
            assert_eq!(RTL_CONTAINS_FIELD!(ps, 5u32, b), 0);
            assert_eq!(RTL_CONTAINS_FIELD!(ps, 4u32, a,), 1);
        }
        let _ = (s.a, s.b);
    }

    #[test]
    fn storage_request_macros_match_srb_h_and_ata_h() {
        // SRB_STATUS_ERROR (0x04) | AUTOSENSE_VALID (0x80) | QUEUE_FROZEN (0x40) -> 0x04.
        let st: i32 = SRB_STATUS!(0xC4u8);
        assert_eq!(st, 0x04);
        match SRB_STATUS!(0x41u8) {
            0x01 => {}
            _ => panic!("SRB_STATUS must strip QUEUE_FROZEN"),
        }
        // UDMA mode 5: 0x40 | 5 = 0x45.
        assert_eq!(IDE_SET_UDMA_MODE!(5u32), 0x45u8);
        assert_eq!(IDE_SET_MWDMA_MODE!(2u32), 0x22u8);
        assert_eq!(IDE_SET_SWDMA_MODE!(1u32), 0x11u8);
        assert_eq!(IDE_SET_ADVANCE_PIO_MODE!(4u32), 0x0Cu8);
        assert_eq!(IDE_SET_DEFAULT_PIO_MODE!(9u32), 1u8);
    }

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
    fn is_equal_guid_compares_all_sixteen_bytes() {
        let a: [u32; 4] = [1, 2, 3, 4];
        let mut b = a;
        assert!(IsEqualGUID!(&a, &b));
        b[3] = 5;
        assert!(!IsEqualGUID!(&a, core::ptr::addr_of!(b)));
    }

    #[test]
    fn number_of_field_is_the_array_length_and_usable_as_a_bound() {
        #[allow(dead_code)]
        struct VolInfo {
            flags: u32,
            device_name: [u16; 260],
        }
        let buf: [u16; RTL_NUMBER_OF_FIELD!(VolInfo, device_name) + 1] =
            [0; RTL_NUMBER_OF_FIELD!(VolInfo, device_name) + 1];
        assert_eq!(buf.len(), 261);
        #[rustfmt::skip]
        let wrapped = RTL_NUMBER_OF_FIELD!(
            VolInfo,
            device_name,
        );
        assert_eq!(wrapped, 260);
    }

    #[test]
    fn field_macros_match_the_layout() {
        #[allow(dead_code)]
        #[repr(C)]
        struct Opts {
            signature: [u8; 8],
            path: *const u16,
            options: *const u16,
        }
        let p = core::mem::size_of::<*const u16>();
        assert_eq!(FIELD_OFFSET!(Opts, path), 8);
        assert_eq!(FIELD_OFFSET!(Opts, options), (8 + p) as i32);
        assert_eq!(RTL_FIELD_SIZE!(Opts, signature), 8);
        assert_eq!(RTL_SIZEOF_THROUGH_FIELD!(Opts, path), 8 + p);
        // A field PATH, as C's `field` may be (the anon-union rule writes `__bindgen_anon_1.Mbr`)
        #[allow(dead_code)]
        struct Outer {
            tag: u32,
            inner: Opts,
        }
        assert_eq!(RTL_FIELD_SIZE!(Outer, inner.signature), 8);
        assert_eq!(
            RTL_SIZEOF_THROUGH_FIELD!(Outer, inner.signature),
            core::mem::offset_of!(Outer, inner) + 8
        );
        assert_eq!(
            FIELD_OFFSET!(Outer, inner.path),
            (core::mem::offset_of!(Outer, inner) + 8) as i32
        );
        let unused = 3;
        UNREFERENCED_PARAMETER!(unused);
    }

    #[test]
    fn field_offset_takes_an_index_where_c_does() {
        #[repr(C)]
        #[allow(dead_code)]
        struct Desc {
            kind: u8,
            data: [u32; 3],
        }
        #[repr(C)]
        #[allow(dead_code)]
        struct List {
            count: u32,
            list: [Desc; 1],
        }
        let d = core::mem::size_of::<Desc>();
        let base = core::mem::offset_of!(List, list);
        // A trailing index past the declared length: DEVICE_RELATIONS' Objects[Count].
        assert_eq!(FIELD_OFFSET!(List, list[3]), (base + 3 * d) as i32);
        let table = [0u16; 7];
        assert_eq!(RTL_NUMBER_OF!(table), 7u32);
        // An index inside the path: CM_RESOURCE_LIST's List[0].PartialResourceList....
        assert_eq!(
            FIELD_OFFSET!(List, list[0].data),
            (base + core::mem::offset_of!(Desc, data)) as i32
        );
    }

    #[test]
    fn copy_move_fill_follow_the_ntdef_argument_order() {
        let src = [1u8, 2, 3, 4];
        let mut dst = [0u8; 4];
        unsafe {
            RtlCopyMemory!(dst.as_mut_ptr(), src.as_ptr(), 3);
        }
        assert_eq!(dst, [1, 2, 3, 0]);
        unsafe {
            RtlMoveMemory!(dst.as_mut_ptr().add(1), dst.as_ptr(), 3);
            RtlFillMemory!(dst.as_mut_ptr(), 1, 0xAA);
        }
        assert_eq!(dst, [0xAA, 1, 2, 3]);
    }

    #[test]
    fn partition_type_predicates_match_ntdddisk() {
        assert!(
            IsContainerPartition!(0x05u8)
                && IsContainerPartition!(0x0F)
                && !IsContainerPartition!(0x07)
        );
        assert!(IsFTPartition!(0x86u8) && IsFTPartition!(0xC7u8) && !IsFTPartition!(0x81u8));
        for t in [
            0x01u8, 0x04, 0x06, 0x07, 0x0B, 0x0C, 0x0E, 0x83, 0x43, 0x96, 0xA5, 0xA6, 0xA9, 0x86,
        ] {
            assert!(IsRecognizedPartition!(t), "{t:#x}");
        }
        for t in [0x00u8, 0x05, 0x0F, 0x81, 0x84, 0xEE] {
            assert!(!IsRecognizedPartition!(t), "{t:#x}");
        }
    }

    #[test]
    fn initialize_object_attributes_fills_every_field() {
        #[allow(non_snake_case)]
        struct Oa {
            Length: u32,
            RootDirectory: *mut u8,
            ObjectName: *mut u8,
            Attributes: u32,
            SecurityDescriptor: *mut u8,
            SecurityQualityOfService: *mut u8,
        }
        let mut root = 0u8;
        let mut o = Oa {
            Length: 7,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: 1 as *mut u8,
        };
        unsafe {
            InitializeObjectAttributes!(
                &mut o as *mut Oa,
                core::ptr::null_mut(),
                0x40,
                &mut root,
                core::ptr::null_mut()
            );
        }
        assert_eq!(o.Length as usize, core::mem::size_of::<Oa>());
        assert_eq!(o.Attributes, 0x40);
        assert_eq!(o.RootDirectory, &mut root as *mut u8);
        assert!(o.SecurityQualityOfService.is_null());
    }

    #[test]
    fn the_folded_constants_are_the_values_their_headers_cast() {
        assert_eq!(UNICODE_NULL, 0u16);
        assert_eq!(INVALID_HANDLE_VALUE as isize, -1);
    }
}

/// `winnt.h` `InterlockedIncrement` / `InterlockedDecrement` — intrinsics under MSVC and
/// `static __inline` here, so no object file defines them. Rust's atomics are the portable
/// spelling of the same LOCK-prefixed read-modify-write, returning the NEW value as Win32 does.
/// `LONG` is `i32` on all three targets, which is what lets this be shared.
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut i32) -> i32 {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}

/// See [`InterlockedIncrement`].
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut i32) -> i32 {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

/// `InterlockedExchange`: returns the OLD value. An intrinsic on amd64/arm64 with no symbol to
/// import; gdi32_vista's resolve copied a kernel shim's `_InterlockedExchange` extern instead,
/// which no user-mode DLL can link.
#[inline(always)]
pub unsafe fn InterlockedExchange(target: *mut i32, value: i32) -> i32 {
    (*(target as *const core::sync::atomic::AtomicI32))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}

/// `InterlockedCompareExchange(Destination, Exchange, Comperand)`: stores `exchange` only when
/// the current value equals `comperand`, and returns the value it found either way.
#[inline(always)]
pub unsafe fn InterlockedCompareExchange(destination: *mut i32, exchange: i32, comperand: i32) -> i32 {
    match (*(destination as *const core::sync::atomic::AtomicI32)).compare_exchange(
        comperand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) | Err(v) => v,
    }
}
