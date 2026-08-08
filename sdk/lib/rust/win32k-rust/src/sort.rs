use win32k_sys::win32k::*;

extern "C" {
    fn qsort( base: *mut core::ffi::c_void, num: usize, size: usize, compar: SORTCOMP, );
}
#[no_mangle]
pub unsafe extern "system" fn EngSort(
    Buf: PBYTE,
    ElemSize: u32,
    ElemCount: u32,
    CompFunc: SORTCOMP,
) {
    
    qsort(
        Buf as *mut core::ffi::c_void,
        ElemCount as usize,
        ElemSize as usize,
        CompFunc,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:          ReactOS win32 subsystem
//  * PURPOSE:
//  * FILE:             win32ss/gdi/eng/sort.c
//  * PROGRAMER:        ReactOS Team
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// void
// APIENTRY
// EngSort(IN OUT PBYTE Buf, IN ULONG ElemSize, IN ULONG ElemCount, IN SORTCOMP CompFunc)
// {
//     qsort(Buf, ElemCount, ElemSize, CompFunc);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
