//! consrv/frontends/gui/helpers.rs — Shared C-macro Rust translations
//! that gui/*.rs need (resource IDs, settings.h colour macros,
//! conio rect/coord macros that aren't shared with condrv, Win32
//! lparam helpers).
//!
//! These were all `#define` macros in the C source. Putting them in
//! one module keeps each gui/*.rs file slimmer and avoids redeclaring
//! them in every translation unit.

use winsrv_sys::winsrv::*;

// ---- resource.h system menu / string IDs (gui/resource.h) ----

// System menu IDs are u32 (used in match arms against LOWORD(wParam)
// which is UINT). Static array initializers cast them to u16 inline.
pub const ID_SYSTEM_EDIT_MARK: u32      = 0xFFF0;
pub const ID_SYSTEM_EDIT_COPY: u32      = 0xFFF1;
pub const ID_SYSTEM_EDIT_PASTE: u32     = 0xFFF2;
pub const ID_SYSTEM_EDIT_SELECTALL: u32 = 0xFFF3;
pub const ID_SYSTEM_EDIT_SCROLL: u32    = 0xFFF4;
pub const ID_SYSTEM_EDIT_FIND: u32      = 0xFFF5;
pub const ID_SYSTEM_DEFAULTS: u32       = 0xFFF6;
pub const ID_SYSTEM_PROPERTIES: u32     = 0xFFF7;

pub const IDS_EDIT: u32           = 204;
pub const IDS_MARK: u32           = 205;
pub const IDS_COPY: u32           = 206;
pub const IDS_PASTE: u32          = 207;
pub const IDS_SELECTALL: u32      = 208;
pub const IDS_SCROLL: u32         = 209;
pub const IDS_FIND: u32           = 210;
pub const IDS_DEFAULTS: u32       = 211;
pub const IDS_PROPERTIES: u32     = 212;
pub const IDS_MARK_TITLE: u32     = 220;
pub const IDS_SELECT_TITLE: u32   = 221;
pub const IDI_TERMINAL: u32       = 1;
pub const IDS_TERMINAL_TITLE: u32 = 400;

// ---- winuser.h class-style / window-style constants ----

pub const CW_USEDEFAULT: i32 = 0x80000000_u32 as i32;

/// `IDC_ARROW` — `MAKEINTRESOURCE(32512)` standard arrow cursor.
pub const IDC_ARROW: *const u16 = 32512_usize as *const u16;

/// `HWND_DESKTOP` — desktop window pseudo-handle (`(HWND)0`).
pub const HWND_DESKTOP: HWND = core::ptr::null_mut();

/// `HWND_MESSAGE` — message-only window pseudo-handle (`(HWND)(-3)`).
pub const HWND_MESSAGE: HWND = (-3_isize) as usize as HWND;

// ---- winuser.h LPARAM unpacker macros ----

#[inline]
pub fn MAKEINTRESOURCEW(i: u32) -> *const u16 {
    (i as usize) as *const u16
}

#[inline]
pub fn GET_KEYSTATE_WPARAM<W: Into<u64>>(w: W) -> u32 {
    (w.into() as u32) & 0xFFFF
}

#[inline]
pub fn GET_WHEEL_DELTA_WPARAM<W: Into<u64>>(w: W) -> i16 {
    (((w.into() as u32) >> 16) & 0xFFFF) as i16
}

#[inline]
pub fn GET_XBUTTON_WPARAM<W: Into<u64>>(w: W) -> u32 {
    ((w.into() as u32) >> 16) & 0xFFFF
}

/// `LOWORD(l)` — extract low 16 bits. C `#define LOWORD(l) ((WORD)(((DWORD_PTR)(l)) & 0xffff))`.
/// Use a u32 arg since WPARAM is `c_uint` on i386 (and amd64/arm64 truncate via the mask).
#[inline]
pub fn LOWORD<L: Into<u64>>(l: L) -> u16 {
    (l.into() & 0xFFFF) as u16
}

/// `HIWORD(l)` — extract high 16 bits of low 32. Polymorphic over int types.
#[inline]
pub fn HIWORD<L: Into<u64>>(l: L) -> u16 {
    ((l.into() >> 16) & 0xFFFF) as u16
}

#[inline]
pub fn GET_X_LPARAM<L: Into<i64>>(lp: L) -> i32 {
    (lp.into() & 0xFFFF) as i16 as i32
}

#[inline]
pub fn GET_Y_LPARAM<L: Into<i64>>(lp: L) -> i32 {
    ((lp.into() >> 16) & 0xFFFF) as i16 as i32
}

#[inline]
pub fn MAKELPARAM(l: u16, h: u16) -> isize {
    ((l as u32) | ((h as u32) << 16)) as i32 as isize
}

// ---- wingdi.h colour helpers ----

#[inline]
pub fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

#[inline]
pub fn PALETTERGB(r: u8, g: u8, b: u8) -> COLORREF {
    0x02000000 | RGB(r, g, b)
}

// ---- consrv/include/settings.h ----

/// `RGBFromAttrib(Console, Attr)` — Console-local palette colour for a
/// console attribute (low 4 bits = foreground index).
#[inline]
pub unsafe fn RGBFromAttrib(Console: PCONSRV_CONSOLE, Attribute: u16) -> COLORREF {
    (*Console).Colors[(Attribute as usize) & 0xF]
}

/// `TextAttribFromAttrib(Attr)` — Foreground index when not reverse-
/// video, background-as-foreground otherwise.
#[inline]
pub fn TextAttribFromAttrib(Attribute: u16) -> u16 {
    if (Attribute & COMMON_LVB_REVERSE_VIDEO as u16) == 0 {
        Attribute & 0xF
    } else {
        (Attribute >> 4) & 0xF
    }
}

/// `BkgdAttribFromAttrib(Attr)` — Background index when not reverse-
/// video, foreground-as-background otherwise.
#[inline]
pub fn BkgdAttribFromAttrib(Attribute: u16) -> u16 {
    if (Attribute & COMMON_LVB_REVERSE_VIDEO as u16) == 0 {
        (Attribute >> 4) & 0xF
    } else {
        Attribute & 0xF
    }
}

// ---- conio rect macros (long-flavour variant for graphics.c / text.c) ----

/// `ConioInitLongRect(Rect, Top, Left, Bottom, Right)` — initialize a
/// `RECT` (LONG-fielded, lowercase top/left/bottom/right) rectangle.
/// Distinct from the SMALL_RECT-fielded `ConioInitRect` macro in
/// condrv/text.rs.
#[inline]
pub unsafe fn ConioInitLongRect(Rect: *mut RECT, Top: i32, Left: i32, Bottom: i32, Right: i32) {
    (*Rect).top    = Top;
    (*Rect).left   = Left;
    (*Rect).bottom = Bottom;
    (*Rect).right  = Right;
}

// ---- conwnd.c IS_WORD_SEP macro ----

/// `IS_WORD_SEP(c)` — wide-char word-separator predicate (\0, space,
/// tab, CR, LF).
#[inline]
pub fn IS_WORD_SEP(c: u16) -> bool {
    c == 0 || c == b' ' as u16 || c == b'\t' as u16 || c == b'\r' as u16 || c == b'\n' as u16
}

// ---- inline atomic shims (Rust core::sync::atomic equivalents that
//      LLM translations expect as global fns named like the C
//      Interlocked* family).

/// `InterlockedIncrement(p)` — 32-bit interlocked add 1.
#[inline]
pub unsafe fn InterlockedIncrement(p: *mut i32) -> i32 {
    let v = core::sync::atomic::AtomicI32::from_ptr(p);
    v.fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}

/// `InterlockedDecrement(p)` — 32-bit interlocked sub 1.
#[inline]
pub unsafe fn InterlockedDecrement(p: *mut i32) -> i32 {
    let v = core::sync::atomic::AtomicI32::from_ptr(p);
    v.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}

/// `_InterlockedIncrement(p)` — MS-compiler intrinsic name; aliases
/// `InterlockedIncrement`.
#[inline]
pub unsafe fn _InterlockedIncrement(p: *mut i32) -> i32 {
    InterlockedIncrement(p)
}

/// `_InterlockedDecrement(p)` — MS-compiler intrinsic name; aliases
/// `InterlockedDecrement`.
#[inline]
pub unsafe fn _InterlockedDecrement(p: *mut i32) -> i32 {
    InterlockedDecrement(p)
}

// NOTE: NT_SUCCESS and InterlockedExchangePointer are in
// winsrv_sys::consts — do not redeclare here (would E0659-conflict
// with the `use winsrv_sys::consts::*;` glob already in every gui/*.rs).

/// `atomic_xchg_seqcst(p, v)` — generic atomic swap (used by a few
/// rustconv outputs). Lives here because `core::intrinsics::atomic_xchg_seqcst`
/// is an unstable nightly intrinsic — translations should use this stable
/// `core::sync::atomic` shim instead.
#[inline]
pub unsafe fn atomic_xchg_seqcst<T>(p: *mut T, v: T) -> T {
    core::ptr::replace(p, v)
}

/// `STATUS_INVALID_VIEW_SIZE` — section-API error not in the
/// hand-written nt:: STATUS_* list.
pub const STATUS_INVALID_VIEW_SIZE: i32 = 0xC000001F_u32 as i32;

// ---- strsafe.h inline wrappers (bindgen doesn't emit static inlines) ----

/// `StringCchCopyNW(pszDest, cchDest, pszSrc, cchToCopy)` — copies at
/// most `cchToCopy` chars from `pszSrc` to `pszDest`, NUL-terminates
/// `pszDest`, truncates safely if needed. Returns S_OK or STRSAFE_E_*.
/// HRESULT shape (i32): 0 = S_OK, 0x8007007A = STRSAFE_E_INSUFFICIENT_BUFFER.
#[inline]
pub unsafe fn StringCchCopyNW(
    pszDest: *mut u16, cchDest: usize, pszSrc: *const u16, cchToCopy: usize,
) -> i32 {
    if cchDest == 0 || pszDest.is_null() {
        return 0x80070057_u32 as i32; // E_INVALIDARG
    }
    let mut i = 0usize;
    let max = if cchToCopy < cchDest - 1 { cchToCopy } else { cchDest - 1 };
    while i < max && !pszSrc.is_null() && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    0 // S_OK
}

/// `StringCchCatW(pszDest, cchDest, pszSrc)` — append `pszSrc` to
/// `pszDest`, NUL-terminating, with overflow-safe truncation.
#[inline]
pub unsafe fn StringCchCatW(pszDest: *mut u16, cchDest: usize, pszSrc: *const u16) -> i32 {
    if cchDest == 0 || pszDest.is_null() || pszSrc.is_null() {
        return 0x80070057_u32 as i32; // E_INVALIDARG
    }
    // Find end of pszDest
    let mut dst_len = 0usize;
    while dst_len < cchDest && *pszDest.add(dst_len) != 0 {
        dst_len += 1;
    }
    if dst_len >= cchDest - 1 {
        return 0x8007007A_u32 as i32; // STRSAFE_E_INSUFFICIENT_BUFFER
    }
    let mut i = 0usize;
    while dst_len + i < cchDest - 1 && *pszSrc.add(i) != 0 {
        *pszDest.add(dst_len + i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(dst_len + i) = 0;
    0
}

/// `ARRAYSIZE(x)` — `_countof(x)` for C arrays. In Rust this is a
/// generic helper that takes a slice reference and returns its len.
/// At call sites where rustconv lowered the C macro to `ARRAYSIZE(arr)`
/// the arg is already a typed array reference.
#[inline]
pub fn ARRAYSIZE<T, const N: usize>(_arr: &[T; N]) -> usize { N }

/// `SendNotifyMessage` — C macro alias for `SendNotifyMessageW` when
/// `UNICODE` is defined (always the case in consrv).
#[inline]
pub unsafe fn SendNotifyMessage(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    SendNotifyMessageW(hWnd, msg, wParam, lParam)
}
