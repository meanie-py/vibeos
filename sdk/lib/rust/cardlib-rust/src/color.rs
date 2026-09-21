//! Colour arithmetic for the table furniture (the recessed outline of an empty
//! pile, and the edges of a fan).
//!
//! Integer-only on purpose. Floating point here would be three multiplications
//! that pull `compiler_builtins`' soft-float helpers into a library that also
//! builds for the i386 no-SSE cell — all cost, no benefit, for arithmetic that
//! is exact in `u32`.

use crate::sys::{COLORREF, GetBValue, GetGValue, GetRValue, RGB};

/// Scale every channel by `percent`/100. Below 100 darkens.
pub fn color_scale(col: COLORREF, percent: u32) -> COLORREF {
    let ch = |v: u8| -> u8 { ((v as u32 * percent) / 100).min(255) as u8 };
    RGB(ch(GetRValue(col)), ch(GetGValue(col)), ch(GetBValue(col)))
}

/// 70% brightness — the recessed outline of an empty pile.
pub fn color_darker(col: COLORREF) -> COLORREF {
    color_scale(col, 70)
}

/// Move each channel `percent` of the way to white.
pub fn color_lighter(col: COLORREF, percent: u32) -> COLORREF {
    let ch = |v: u8| -> u8 {
        let headroom = 255u32 - v as u32;
        (v as u32 + headroom * percent / 100).min(255) as u8
    };
    RGB(ch(GetRValue(col)), ch(GetGValue(col)), ch(GetBValue(col)))
}
