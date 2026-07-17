//! PC-link fallbacks for `vidfb.c`/`uefildr.c` UEFI-only symbols.
//!
//! `arch/vidfb.c` and `arch/uefi/uefildr.c` are UEFI-only translation
//! units (built only via `uefi.cmake` → `uefifreeldr_common`/`uefildr`).
//! In the original C, `freeldr_pe` (PC/pcat i386) never compiled
//! `ueficon.c`/`uefivid.c`/`vidfb.c`/`uefildr.c`, so the PC binary
//! never referenced any of their symbols.
//!
//! The Rust port made `ueficon.rs`/`uefivid.rs` unconditional `pub mod`
//! entries in `lib.rs` (commenting a `pub mod` is the forbidden
//! AGENTS.md anti-pattern). Several of their `#[no_mangle]` fns are
//! console-/video-vtable entries, so the PC link keeps them and their
//! `extern` references to vidfb/uefildr symbols become undefined
//! (those C units aren't in the PC link).
//!
//! Resolution: this is an **isolated fallback module**. With the
//! workspace firewall (`lto=false`, `codegen-units=256`) each
//! `#[no_mangle]` item compiles to its own object inside
//! `libfreeldr_rust.a`, so lld extracts a member ONLY to satisfy an
//! otherwise-undefined symbol:
//!   * PC link  — nothing else defines these → these members are
//!     pulled.
//!   * UEFI link — `vidfb.c`/`uefildr.c` (in `uefifreeldr_common`,
//!     ahead of the rust lib) define them strongly → these members are
//!     NEVER pulled → zero duplicate-symbol risk.
//!
//! Behaviour: on PC the UEFI framebuffer console / EFI system table are
//! never installed/initialised (PC uses the VGA console + BIOS), so
//! these are unreachable at runtime. The real implementations live in
//! vidfb.c/uefildr.c and are what the UEFI build links. `framebufInfo`
//! is a vidfb.c file-static (no external linkage), so a separate module
//! cannot reproduce the bodies — the only correct PC body is inert.
//! `PixelBitmasksToBpp` is pure (FORCEINLINE in vidfb.h) so it is
//! implemented faithfully (popcount of the OR'd pixel masks).

use ntkrnl_sys::base::{UCHAR, ULONG, PULONG, ULONG_PTR, BOOLEAN};
use core::ffi::c_void;

// vidfb.c:513 `VOID FbConsScrollUp(UCHAR Attr)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FbConsScrollUp(_Attr: UCHAR) {}

// vidfb.c:445 `VOID FbConsPutChar(UCHAR Char, UCHAR Attr, ULONG Column,
// ULONG Row)`. (uefivid.rs decl: `fn FbConsPutChar(i32,u8,u32,u32)` —
// ABI-identical on i386: all 4 stack args.)
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FbConsPutChar(
    _Char: i32,
    _Attr: u8,
    _Column: u32,
    _Row: u32,
) {
}

// vidfb.c:462 `VOID FbConsGetDisplaySize(PULONG Width, PULONG Height,
// PULONG Depth)`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FbConsGetDisplaySize(
    Width: PULONG,
    Height: PULONG,
    Depth: PULONG,
) {
    if !Width.is_null() {
        *Width = 0;
    }
    if !Height.is_null() {
        *Height = 0;
    }
    if !Depth.is_null() {
        *Depth = 0;
    }
}

// vidfb.h:55 `BOOLEAN VidFbInitializeVideo(...)`. Opaque-pointer args
// (ABI = pointer-width, matching uefivid.rs's typed decl on i386).
// Inert on PC (no framebuffer); returns FALSE.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn VidFbInitializeVideo(
    _pFbData: *mut *mut c_void,
    _BaseAddress: ULONG_PTR,
    _BufferSize: ULONG,
    _ScreenWidth: u32,
    _ScreenHeight: u32,
    _PixelsPerScanLine: u32,
    _BitsPerPixel: u32,
    _PixelMasks: *mut c_void,
) -> BOOLEAN {
    0 // FALSE
}

// vidfb.h:41 `FORCEINLINE ULONG PixelBitmasksToBpp(R,G,B,Reserved)` =
// CountNumberOfBits(R|G|B|Reserved) — pure; faithful (popcount).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PixelBitmasksToBpp(
    RedMask: ULONG,
    GreenMask: ULONG,
    BlueMask: ULONG,
    ReservedMask: ULONG,
) -> ULONG {
    (RedMask | GreenMask | BlueMask | ReservedMask).count_ones() as ULONG
}

// uefildr.c:16 `EFI_SYSTEM_TABLE *GlobalSystemTable;` — UEFI-only
// global data. ueficon.rs/uefivid.rs read it (UEFI-dead on PC).
// Pointer-width; the UEFI build links uefildr.c's strong definition.
#[no_mangle]
pub static mut GlobalSystemTable: *mut c_void = core::ptr::null_mut();
