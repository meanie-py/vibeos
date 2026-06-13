//! startup/cinitexe.c — the C-initializer-section boundary markers.
//!
//! `_initterm(__xi_a, __xi_z)` / `_initterm(__xc_a, __xc_z)` walk every
//! pointer the linker placed between these markers. lld merges `.CRT$X*`
//! sections and sorts by the full `$`-suffixed name, so the layout is:
//!   .CRT$XIA  __xi_a   (start)
//!   .CRT$XIAA pre_c_init / mingw_pcinit         (entry.rs / crtdll.rs)
//!   .CRT$XIC  __mingw_pinit (lconv)             (misc.rs)
//!   .CRT$XIZ  __xi_z   (end)
//! and likewise XCA/XCAA/XCZ for the C++ ctor list.
//!
//! `#[used]` keeps the markers from being dropped; `#[no_mangle]` pins the
//! exact symbol the init walk and other TUs reference.

use crate::winapi::Pvfv;

#[no_mangle]
#[link_section = ".CRT$XIA"]
#[used]
pub static __xi_a: [Pvfv; 1] = [None];

#[no_mangle]
#[link_section = ".CRT$XIZ"]
#[used]
pub static __xi_z: [Pvfv; 1] = [None];

#[no_mangle]
#[link_section = ".CRT$XCA"]
#[used]
pub static __xc_a: [Pvfv; 1] = [None];

#[no_mangle]
#[link_section = ".CRT$XCZ"]
#[used]
pub static __xc_z: [Pvfv; 1] = [None];
