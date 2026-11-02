use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Wow64pPsi64: *const SERVERINFO = ::core::ptr::null::<SERVERINFO>();

static mut Wow64pGuestPfnA: *const ULONG = ::core::ptr::null::<ULONG>();

static mut Wow64pGuestPfnW: *const ULONG = ::core::ptr::null::<ULONG>();

const _: () = assert!(core::mem::size_of::<PFNCLIENT>() % core::mem::size_of::<ULONG_PTR>() == 0);

pub const N_PFNCLIENT: usize = core::mem::size_of::<PFNCLIENT>() / core::mem::size_of::<ULONG_PTR>();
#[no_mangle]
pub unsafe extern "C" fn Wow64pPfnMapSetPsi(Psi64: ULONGLONG) {
    Wow64pPsi64 = (Psi64 as usize) as *const SERVERINFO;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pPfnMapSetGuest(PfnA32: ULONG, PfnW32: ULONG) {
    Wow64pGuestPfnA = PfnA32 as ULONG_PTR as *const ULONG;
    Wow64pGuestPfnW = PfnW32 as ULONG_PTR as *const ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pMapClientProc(mut Proc64: ULONGLONG) -> ULONG {
    let mut A: *const ULONG_PTR = ::core::ptr::null::<ULONG_PTR>();
    let mut W: *const ULONG_PTR = ::core::ptr::null::<ULONG_PTR>();
    let mut i: SIZE_T = 0;
    if Proc64 == 0
        || Proc64 <= (0xFFFFFFFF) as ULONGLONG
        || Wow64pPsi64.is_null()
        || Wow64pGuestPfnA.is_null()
        || Wow64pGuestPfnW.is_null()
    {
        return Proc64 as ULONG;
    }
    A = &raw const (*Wow64pPsi64).apfnClientA as *const ULONG_PTR;
    W = &raw const (*Wow64pPsi64).apfnClientW as *const ULONG_PTR;
    i = (0) as SIZE_T;
    while (i as usize) < (N_PFNCLIENT as usize) {
        if *W.offset(i as isize) == Proc64 {
            return *Wow64pGuestPfnW.offset(i as isize);
        }
        if *A.offset(i as isize) == Proc64 {
            return *Wow64pGuestPfnA.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    return Proc64 as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pUnmapClientProc(mut Proc32: ULONG) -> ULONGLONG {
    let mut A: *const ULONG_PTR = ::core::ptr::null::<ULONG_PTR>();
    let mut W: *const ULONG_PTR = ::core::ptr::null::<ULONG_PTR>();
    let mut i: SIZE_T = 0;
    if Proc32 == 0
        || Wow64pPsi64.is_null()
        || Wow64pGuestPfnA.is_null()
        || Wow64pGuestPfnW.is_null()
    {
        return Proc32 as ULONGLONG;
    }
    A = &raw const (*Wow64pPsi64).apfnClientA as *const ULONG_PTR;
    W = &raw const (*Wow64pPsi64).apfnClientW as *const ULONG_PTR;
    i = 0 as SIZE_T;
    while (i as usize) < N_PFNCLIENT {
        if *Wow64pGuestPfnW.offset(i as isize) == Proc32 {
            return *W.offset(i as isize) as ULONGLONG;
        }
        if *Wow64pGuestPfnA.offset(i as isize) == Proc32 {
            return *A.offset(i as isize) as ULONGLONG;
        }
        i = i.wrapping_add(1);
    }
    return Proc32 as ULONGLONG;
}
