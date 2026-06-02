//! Shared helpers for the gdi/dib per-bit-depth renderers.
//!
//! These are C macros (`dib.h`, `gdi/eng/inteng.h`, `xdk/mmfuncs.h`),
//! `FORCEINLINE`s (`gdi/ntgdi/rect.h`), pool tags (`tags.h`) and cross-file
//! symbol declarations that the rustconv translation either imported as
//! phantom `extern "C"` declarations (no real symbol → never links) or omitted.
//! Centralized here and pulled in via `use crate::dib_helpers::*`. The
//! macro/inline bodies mirror the proven eng implementations (bitblt.rs) — eng
//! smoke is green with these, so they are the trusted reference.
//!
//! `XLATEOBJ_iXlate`, `notmask`, `altnotmask` are NOT here — win32k_sys
//! exports them with correct types, so consumers get them from that glob.
#![allow(non_snake_case)]

use win32k_sys::win32k::*;

// Pool allocator entrypoints (not in the win32k_sys glob the renderers import).
pub(crate) use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};

extern "C" {
    // Dispatch table — real array in dib.c (kept C). win32k_sys's bindgen
    // version is `[DIB_FUNCTIONS; 0]` (flexible array → useless), so declare
    // the true length here. PRIVATE (used only by DIB_GetSource{,Index}); a
    // `pub(crate)` re-export would collide with the bindgen glob. Consumers
    // that index the table directly declare their own local copy (shadows
    // the bindgen glob — the bitblt.rs pattern).
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11];

    // ROP combiner — real cdecl function in dib.c (kept C).
    pub(crate) fn DIB_DoRop(Rop: u32, Dest: u32, Source: u32, Pattern: u32) -> u32;

    // Kernel debug print (ntoskrnl) — variadic cdecl.
    pub(crate) fn DbgPrint(format: *const u8, ...) -> i32;

    // Cross-bit-depth GetPixel / HLine references (defined in the sibling
    // renderer modules as #[no_mangle], or i386 hand-asm for HLine). Declared
    // here so any renderer can call across formats; the defining module's own
    // #[no_mangle] item shadows this glob import.
    pub(crate) fn DIB_1BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_4BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_8BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_16BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_24BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_32BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    pub(crate) fn DIB_24BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32);
    pub(crate) fn DIB_32BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32);
}

// `RECTL_vMakeWellOrdered` is FASTCALL in C (rect.h) — the exact i386
// decoration (`@RECTL_vMakeWellOrdered@4`) matters, so dual-cfg like bitblt.rs.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub(crate) fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub(crate) fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
}

// TAG_DIB = ' BID' (tags.h) — C multichar constant, big-endian:
// (' '<<24)|('B'<<16)|('I'<<8)|'D' = 0x20424944.
pub(crate) const TAG_DIB: u32 = 0x2042_4944;

// ALIGN_UP_BY(size, align) = (size + align - 1) & ~(align - 1)   [mmfuncs.h]
#[inline(always)]
pub(crate) fn ALIGN_UP_BY(size: usize, align: usize) -> usize {
    (size.wrapping_add(align).wrapping_sub(1)) & !(align.wrapping_sub(1))
}

// C `abs(int)`
#[inline(always)]
pub(crate) fn abs(x: i32) -> i32 {
    x.abs()
}

// ExAllocatePoolZero is FORCEINLINE in C (xdk/exfuncs.h) — alloc then zero. It
// has no importable symbol (win32k links nothing for it; the C call inlines),
// so replicate the wrapper. POOL_TYPE is a u32 enum (ntkrnl_sys).
#[inline(always)]
pub(crate) unsafe fn ExAllocatePoolZero(pool_type: u32, n: SIZE_T, tag: u32) -> PVOID {
    let p = ExAllocatePoolWithTag(pool_type, n, tag);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, n as usize);
    }
    p
}

// MASK1BPP(x) = (1 << (7 - ((x) & 7)))   [dib.h]
#[inline(always)]
pub(crate) unsafe fn MASK1BPP(x: i32) -> u8 {
    (1i32 << (7 - (x & 7))) as u8
}

// ROP4_USES_SOURCE(Rop4)  = ((((Rop4) & 0xCCCC) >> 2) != ((Rop4) & 0x3333))  [inteng.h]
#[inline(always)]
pub(crate) unsafe fn ROP4_USES_SOURCE(rop4: u32) -> u32 {
    if ((rop4 & 0xCCCC) >> 2) != (rop4 & 0x3333) { 1 } else { 0 }
}

// ROP4_USES_PATTERN(Rop4) = ((((Rop4) & 0xF0F0) >> 4) != ((Rop4) & 0x0F0F))  [inteng.h]
#[inline(always)]
pub(crate) unsafe fn ROP4_USES_PATTERN(rop4: u32) -> u32 {
    if ((rop4 & 0xF0F0) >> 4) != (rop4 & 0x0F0F) { 1 } else { 0 }
}

// RECTL_lGetWidth(prcl)  = prcl->right - prcl->left    [rect.h FORCEINLINE]
#[inline(always)]
pub(crate) unsafe fn RECTL_lGetWidth(prcl: *const RECTL) -> i32 {
    (*prcl).right - (*prcl).left
}

// RECTL_lGetHeight(prcl) = prcl->bottom - prcl->top    [rect.h FORCEINLINE]
#[inline(always)]
pub(crate) unsafe fn RECTL_lGetHeight(prcl: *const RECTL) -> i32 {
    (*prcl).bottom - (*prcl).top
}

// BitsPerFormat(Format) = gajBitsPerFormat[Format]     [surface.h] — mirror engmisc.rs
#[inline(always)]
pub(crate) unsafe fn BitsPerFormat(format: u32) -> u32 {
    *core::ptr::addr_of!(gajBitsPerFormat)
        .cast::<u8>()
        .add(format as usize) as u32
}

// DIB_GetSource(Surf,sx,sy,Xlate) = XLATEOBJ_iXlate(Xlate, <fmt>.DIB_GetPixel(Surf,sx,sy))  [dib.h]
#[inline(always)]
pub(crate) unsafe fn DIB_GetSource(
    pso: *mut SURFOBJ,
    x: i32,
    y: i32,
    pxlo: *mut XLATEOBJ,
) -> u32 {
    let fmt = (*pso).iBitmapFormat as usize;
    let pixel = match DibFunctionsForBitmapFormat[fmt].DIB_GetPixel {
        Some(f) => f(pso, x, y),
        None => 0,
    };
    XLATEOBJ_iXlate(pxlo, pixel)
}

// DIB_GetSourceIndex(Surf,sx,sy) = <fmt>.DIB_GetPixel(Surf,sx,sy)   [dib.h]
#[inline(always)]
pub(crate) unsafe fn DIB_GetSourceIndex(pso: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    let fmt = (*pso).iBitmapFormat as usize;
    match DibFunctionsForBitmapFormat[fmt].DIB_GetPixel {
        Some(f) => f(pso, x, y),
        None => 0,
    }
}
