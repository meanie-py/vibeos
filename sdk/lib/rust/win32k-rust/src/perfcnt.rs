use ntkrnl_sys::ddk::{KeQueryPerformanceCounter};

// ---- file-scope preamble (translated) ----
// (empty preamble — perfcnt.c has no file-scope declarations)

#[no_mangle]
pub unsafe extern "system" fn EngQueryPerformanceFrequency(Frequency: *mut i64) {
    let mut Freq: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
    
    KeQueryPerformanceCounter(&mut Freq);
    *Frequency = Freq.QuadPart;
}
#[no_mangle]
pub unsafe extern "system" fn EngQueryPerformanceCounter(Count: *mut i64) {
    let PerfCount = KeQueryPerformanceCounter(core::ptr::null_mut());
    *Count = PerfCount.QuadPart;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Performance Counter Functions
//  * FILE:              win32ss/gdi/eng/perfcnt.c
//  * PROGRAMER:         Ge van Geldorp
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
// VOID APIENTRY
// EngQueryPerformanceFrequency(LONGLONG *Frequency)
// {
//     LARGE_INTEGER Freq;
//
//     KeQueryPerformanceCounter(&Freq);
//     *Frequency = Freq.QuadPart;
// }
//
// /*
//  * @implemented
//  */
// VOID APIENTRY
// EngQueryPerformanceCounter(LONGLONG *Count)
// {
//     LARGE_INTEGER PerfCount;
//
//     PerfCount = KeQueryPerformanceCounter(NULL);
//     *Count = PerfCount.QuadPart;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
