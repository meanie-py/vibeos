//! objects/efloat.c → Rust. FLOAT↔EFLOAT conversion (i386-only).
//
// CFG-GATE: x86-only — the integrator MUST wire this as
//   `#[cfg(target_arch = "x86")] pub mod efloat;`
// (matches the CMake i386-gate `if(ARCH STREQUAL i386) list(APPEND SOURCE objects/efloat.c)`).
// The whole file is therefore written for the `_X86_` branch only.
//
// NEEDS-TYPE: none — `EFLOAT_S { lMant: LONG, lExp: LONG }` and the `gxf_long` union
//   (`{ f: FLOAT, l: ULONG }`) both come from `win32k_sys::win32k` bindgen. `FLOATL` is
//   `DWORD` (an INTEGER holding a float's bits, NOT f32) → the EFtoF/FtoEF ABI boundary is
//   all-integer, so there is NO i386 no-SSE float-ABI problem; the bodies are pure integer
//   bit-twiddling. (The `gxf_long.f: FLOAT` union *field* is fine — a struct field, not a
//   fn-boundary value — and is never touched on the `_X86_` path: we use `worker.l`.)
// NEEDS-IMPORT: none — both functions are leaf bit-twiddlers, no cross-file calls.

use win32k_sys::win32k::*;

// /* the following deal with IEEE single-precision numbers */
const EXCESS: ULONG = 126; // EXCESS 126L
const SIGNBIT: ULONG = 0x80000000; // SIGNBIT 0x80000000L

// SIGN(fp)    ((fp) & SIGNBIT)
#[inline(always)]
fn sign(fp: ULONG) -> ULONG {
    fp & SIGNBIT
}
// EXP(fp)     (((fp) >> 23L) & 0xFF)
#[inline(always)]
fn exp(fp: ULONG) -> ULONG {
    (fp >> 23) & 0xFF
}
// MANT(fp)    ((fp) & 0x7FFFFFL)
#[inline(always)]
fn mant(fp: ULONG) -> ULONG {
    fp & 0x7FFFFF
}
// PACK(s,e,m) ((s) | ((e) << 23L) | (m))
#[inline(always)]
fn pack(s: ULONG, e: ULONG, m: ULONG) -> ULONG {
    s | (e << 23) | m
}

#[no_mangle]
pub unsafe extern "fastcall" fn EFtoF(efp: *mut EFLOAT_S) -> FLOATL {
    // if (!efp->lMant) return 0;
    if (*efp).lMant == 0 {
        return 0;
    }

    let mut mant_v: ULONG = (*efp).lMant as ULONG; // Mant = efp->lMant;
    let mut exp_v: ULONG = (*efp).lExp as ULONG; //   Exp  = efp->lExp;
    let sign_v: ULONG = sign(mant_v); //              Sign = SIGN(Mant);

    if sign_v != 0 {
        // if (Sign) Mant = -(LONG)Mant;
        mant_v = (mant_v as LONG).wrapping_neg() as ULONG;
    }
    mant_v >>= 7; //                       Mant >>= 7;
    exp_v = exp_v.wrapping_add(EXCESS - 1); // Exp += (EXCESS-1);

    mant_v = mant(mant_v); //              Mant = MANT(Mant);
    pack(sign_v, exp_v, mant_v) //         return PACK(Sign, Exp, Mant);
}

#[no_mangle]
pub unsafe extern "fastcall" fn FtoEF(efp: *mut EFLOAT_S, f: FLOATL) {
    let mut sign_v: ULONG = 0; // ULONG Mant, Exp, Sign = 0;
    let mut worker: gxf_long = core::mem::zeroed();

    // #ifdef _X86_  worker.l = f; // It's a float stored in a long.
    worker.l = f;

    let mut exp_v: ULONG = exp(worker.l); //  Exp  = EXP(worker.l);
    let mut mant_v: ULONG = mant(worker.l); // Mant = MANT(worker.l);
    if sign(worker.l) != 0 {
        // if (SIGN(worker.l)) Sign = -1;
        sign_v = (-1i32) as ULONG;
    }

    mant_v = (mant_v << 7) | 0x40000000; // Mant = ((Mant << 7) | 0x40000000);
    mant_v ^= sign_v; //                    Mant ^= Sign;
    mant_v = mant_v.wrapping_sub(sign_v); // Mant -= Sign;
    exp_v = exp_v.wrapping_sub(EXCESS - 1); // Exp -= (EXCESS-1);

    // efp->lMant = Mant;  efp->lExp = Exp;  (volatile out-pointer writes — captures(none) guard)
    core::ptr::addr_of_mut!((*efp).lMant).write_volatile(mant_v as LONG);
    core::ptr::addr_of_mut!((*efp).lExp).write_volatile(exp_v as LONG);
}
