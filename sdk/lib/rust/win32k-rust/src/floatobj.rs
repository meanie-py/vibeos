// Shared per-arch FLOATOBJ shim for the gdi/ntgdi geometry wave (xformobj, arc,
// stockobj, …).
//
// On i386 (kernel) FLOATOBJ is a {ul1,ul2} struct (EFLOAT_S {lMant,lExp}); the
// arithmetic ops are real ENGAPI functions (APIENTRY=stdcall) from
// gdi/eng/i386/floatobj.{S,c}, while the predicate/convert helpers are
// FORCEINLINE over EFLOAT_S (gdi/eng/floatobj.h — NO exported symbol). On
// amd64/arm64 FLOATOBJ is a plain FLOAT and every op is an inline float macro
// (winddi.h / floatobj.h). FLOATL is the wire form: DWORD(u32) on i386, FLOAT
// on amd64/arm64. Both are reproduced faithfully.

use win32k_sys::win32k::*;

// ── Group A — real engine functions on i386 (stdcall) ──────────────────────
#[cfg(target_arch = "x86")]
extern "system" {
    pub fn FLOATOBJ_Add(pfo: PFLOATOBJ, pfo2: PFLOATOBJ);
    pub fn FLOATOBJ_Sub(pfo: PFLOATOBJ, pfo2: PFLOATOBJ);
    pub fn FLOATOBJ_Mul(pfo: PFLOATOBJ, pfo2: PFLOATOBJ);
    pub fn FLOATOBJ_MulLong(pfo: PFLOATOBJ, l: i32);
    pub fn FLOATOBJ_AddLong(pfo: PFLOATOBJ, l: i32);
    pub fn FLOATOBJ_DivLong(pfo: PFLOATOBJ, l: i32);
    pub fn FLOATOBJ_Div(pfo: *mut FLOATOBJ, pfo2: *mut FLOATOBJ);
    pub fn FLOATOBJ_Neg(pfo: *mut FLOATOBJ);
    pub fn FLOATOBJ_SetFloat(pf: *mut FLOATOBJ, f: FLOATL);
    pub fn FLOATOBJ_SetLong(pf: *mut FLOATOBJ, l: i32);
    pub fn FLOATOBJ_GetFloat(pf: *const FLOATOBJ) -> FLOATL;
    pub fn EngMulDiv(a: i32, b: i32, c: i32) -> i32;
}

// ── Group A — inline f32 ops on amd64/arm64 (FLOATOBJ == FLOAT == f32) ───────
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_Add(pf: PFLOATOBJ, pf1: PFLOATOBJ) { *pf += *pf1; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_Sub(pf: PFLOATOBJ, pf1: PFLOATOBJ) { *pf -= *pf1; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_Mul(pf: PFLOATOBJ, pf1: PFLOATOBJ) { *pf *= *pf1; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_MulLong(pf: PFLOATOBJ, l: i32) { *pf *= l as FLOATOBJ; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_AddLong(pf: PFLOATOBJ, l: i32) { *pf += l as FLOATOBJ; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_DivLong(pf: PFLOATOBJ, l: i32) { *pf /= l as FLOATOBJ; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_Div(pf: *mut FLOATOBJ, pf1: *mut FLOATOBJ) { *pf /= *pf1; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_Neg(pf: *mut FLOATOBJ) { *pf = -*pf; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_SetFloat(pf: *mut FLOATOBJ, f: FLOATL) { *pf = f; }
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_SetLong(pf: *mut FLOATOBJ, l: i32) { *pf = l as FLOATOBJ; }
// win32ss/pch.h redefines this as (*(pf)) for win32k: the float itself. winddi.h's
// (*(PULONG)(pf)) only makes sense where FLOATL is DWORD (i386 kernel); here FLOATL is FLOAT,
// and taking the bits and converting them turned 1.0 into 1065353216.0 for every
// GetWorldTransform on amd64/arm64.
#[cfg(not(target_arch = "x86"))]
#[inline(always)] pub unsafe fn FLOATOBJ_GetFloat(pf: *const FLOATOBJ) -> FLOATL { *pf }

// ── Group B — predicate/convert helpers (FORCEINLINE i386 / macro non-x86) ──
// No exported symbol on either arch, so always inline.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_Equal0(pf: *const FLOATOBJ) -> u32 {
    (((*pf).ul1 == 0) && ((*pf).ul2 == 0)) as u32
}
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_Equal1(pf: *const FLOATOBJ) -> u32 {
    (((*pf).ul1 == 0x40000000) && ((*pf).ul2 == 2)) as u32
}
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_GetFix(pf: *const FLOATOBJ) -> i32 {
    let l_mant = (*pf).ul1 as i32;
    let l_exp = (*pf).ul2 as i32;
    let shift = 28 - l_exp;
    if shift >= 0 { l_mant >> shift } else { l_mant << (-shift) }
}
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_IsLong(pf: *const FLOATOBJ) -> u32 {
    let l_mant = (*pf).ul1 as i32;
    let ul_shift = (*pf).ul2;
    if ul_shift < 32 { (l_mant.wrapping_shl(ul_shift) == 0) as u32 } else { (ul_shift == 32) as u32 }
}
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_bConvertToLong(pf: *const FLOATOBJ, pl: *mut i32) -> i32 {
    let l_mant = (*pf).ul1 as i32;
    let l_exp = (*pf).ul2 as i32;
    if l_exp > 32 { return 0; }
    if l_exp < 2 { *pl = 0; return 1; }
    *pl = EngMulDiv(l_mant, 1 << (l_exp - 2), 0x40000000);
    1
}
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn FLOATOBJ_Set0(pf: *mut core::ffi::c_void) {
    let p = pf as *mut FLOATOBJ;
    (*p).ul1 = 0;
    (*p).ul2 = 0;
}

#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_Equal0(pf: *const FLOATOBJ) -> u32 { (*pf == 0.0) as u32 }
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_Equal1(pf: *const FLOATOBJ) -> u32 { (*pf == 1.0) as u32 }
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_GetFix(pf: *const FLOATOBJ) -> i32 { (*pf * 16.0) as i32 }
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_IsLong(pf: *const FLOATOBJ) -> u32 { ((((*pf) as i32) as f32) == *pf) as u32 }
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_bConvertToLong(pf: *const FLOATOBJ, pl: *mut i32) -> i32 { *pl = *pf as i32; 1 }
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
pub unsafe fn FLOATOBJ_Set0(pf: *mut core::ffi::c_void) { *(pf as *mut f32) = 0.0; }
