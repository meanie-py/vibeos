//! WoW64 DC_ATTR layout shim (M7 — see docs/engineering/active-lanes/wow64-plan.md).
//!
//! The kernel's DC_ATTR is the 64-bit layout; a 32-bit gdi32 running under
//! WoW64 must read/write the shared struct at the 64-bit offsets (this is
//! what real Windows' wow64 gdi32 does too). `DC_ATTR64` mirrors the 64-bit
//! layout on the 32-bit compiler: every 8-byte pointer/handle slot is split
//! into a native-width `field` (the meaningful low half — little-endian) plus
//! a `field_hi` pad, and every alignment hole repr(C)-on-64 would insert is
//! an explicit `__padN`. That keeps each projected field the SAME Rust type
//! as in the native 32-bit DC_ATTR, so `dcattr_f!` can return one pointer
//! type from both branches.
//!
//! Invariant that makes the lo-half writes sound: the kernel zeroes the
//! DC_ATTR at allocation and only ever stores handle values with a zero
//! upper half, so the `_hi` words stay 0 forever and a 4-byte client write
//! composes to the correct 8-byte value.
//!
//! The three MATRIX and four FLOATOBJ fields are never accessed by client
//! code (verified by sweep) — they are opaque correctly-sized filler here
//! (64-bit FLOATOBJ is a 4-byte FLOAT; i386's 8-byte EFLOAT semantics do NOT
//! apply to the shared struct — another reason the guest must not touch
//! them).

#![cfg(target_arch = "x86")]

use win32k_sys::win32k::*;

/// 64-bit MATRIX: 6 x FLOATOBJ(4) + fxDx/fxDy (FIX = LONG) + flAccel. 36 bytes.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MATRIX64 {
    pub ef: [f32; 6],
    pub fxDx: i32,
    pub fxDy: i32,
    pub flAccel: u32,
}

#[repr(C, align(8))]
pub struct DC_ATTR64 {
    pub pvLDC: PVOID,
    pub pvLDC_hi: u32,
    pub ulDirty_: ULONG,
    pub __pad0: u32,
    pub hbrush: HANDLE,
    pub hbrush_hi: u32,
    pub hpen: HANDLE,
    pub hpen_hi: u32,
    pub crBackgroundClr: COLORREF,
    pub ulBackgroundClr: ULONG,
    pub crForegroundClr: COLORREF,
    pub ulForegroundClr: ULONG,
    pub crBrushClr: COLORREF,
    pub ulBrushClr: ULONG,
    pub crPenClr: COLORREF,
    pub ulPenClr: ULONG,
    pub iCS_CP: DWORD,
    pub iGraphicsMode: INT,
    pub jROP2: BYTE,
    pub jBkMode: BYTE,
    pub jFillMode: BYTE,
    pub jStretchBltMode: BYTE,
    pub ptlCurrent: POINTL,
    pub ptfxCurrent: POINTL,
    pub lBkMode: LONG,
    pub lFillMode: LONG,
    pub lStretchBltMode: LONG,
    pub flFontMapper: FLONG,
    pub lIcmMode: LONG,
    pub hcmXform: HANDLE,
    pub hcmXform_hi: u32,
    pub hColorSpace: HCOLORSPACE,
    pub hColorSpace_hi: u32,
    pub flIcmFlags: FLONG,
    pub IcmBrushColor: INT,
    pub IcmPenColor: INT,
    pub __pad1: u32,
    pub pvLIcm: PVOID,
    pub pvLIcm_hi: u32,
    pub flTextAlign: FLONG,
    pub lTextAlign: LONG,
    pub lTextExtra: LONG,
    pub lRelAbs: LONG,
    pub lBreakExtra: LONG,
    pub cBreak: LONG,
    pub hlfntNew: HANDLE,
    pub hlfntNew_hi: u32,
    pub mxWorldToDevice: MATRIX64,
    pub mxDeviceToWorld: MATRIX64,
    pub mxWorldToPage: MATRIX64,
    pub efM11PtoD: f32,
    pub efM22PtoD: f32,
    pub efDxPtoD: f32,
    pub efDyPtoD: f32,
    pub iMapMode: INT,
    pub dwLayout: DWORD,
    pub lWindowOrgx: LONG,
    pub ptlWindowOrg: POINTL,
    pub szlWindowExt: SIZEL,
    pub ptlViewportOrg: POINTL,
    pub szlViewportExt: SIZEL,
    pub flXform: FLONG,
    pub szlVirtualDevicePixel: SIZEL,
    pub szlVirtualDeviceMm: SIZEL,
    pub szlVirtualDeviceSize: SIZEL,
    pub ptlBrushOrigin: POINTL,
    pub VisRectRegion: RGN_ATTR,
    pub __pad2: u32,
}

// Offsets computed from the 64-bit repr(C) rules; drift here = silent guest
// corruption, so pin the load-bearing ones + total size.
const _: () = {
    assert!(core::mem::offset_of!(DC_ATTR64, ulDirty_) == 8);
    assert!(core::mem::offset_of!(DC_ATTR64, hbrush) == 16);
    assert!(core::mem::offset_of!(DC_ATTR64, hpen) == 24);
    assert!(core::mem::offset_of!(DC_ATTR64, ptlCurrent) == 76);
    assert!(core::mem::offset_of!(DC_ATTR64, hcmXform) == 112);
    assert!(core::mem::offset_of!(DC_ATTR64, pvLIcm) == 144);
    assert!(core::mem::offset_of!(DC_ATTR64, hlfntNew) == 176);
    assert!(core::mem::offset_of!(DC_ATTR64, mxWorldToDevice) == 184);
    assert!(core::mem::offset_of!(DC_ATTR64, iMapMode) == 308);
    assert!(core::mem::offset_of!(DC_ATTR64, ptlWindowOrg) == 320);
    assert!(core::mem::offset_of!(DC_ATTR64, flXform) == 352);
    assert!(core::mem::offset_of!(DC_ATTR64, VisRectRegion) == 388);
    assert!(core::mem::size_of::<DC_ATTR64>() == 416);
};
