use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::PFAST_MUTEX;

extern "C" {
    fn _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    fn _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex: PFAST_MUTEX);
    fn _ExAcquireFastMutex(FastMutex: PFAST_MUTEX);
    fn _ExReleaseFastMutex(FastMutex: PFAST_MUTEX);
    fn _ExAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    fn _ExReleaseFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    fn _ExTryToAcquireFastMutex(FastMutex: PFAST_MUTEX) -> BOOLEAN;
}

// On i386 these are FASTCALL (extern "fastcall"), on 64-bit FASTCALL is the default.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExEnterCriticalRegionAndAcquireFastMutexUnsafe(
    FastMutex: PFAST_MUTEX,
) {
    _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExEnterCriticalRegionAndAcquireFastMutexUnsafe(
    FastMutex: PFAST_MUTEX,
) {
    _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(
    FastMutex: PFAST_MUTEX,
) {
    _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(
    FastMutex: PFAST_MUTEX,
) {
    _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExAcquireFastMutex(FastMutex: PFAST_MUTEX) {
    _ExAcquireFastMutex(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExAcquireFastMutex(FastMutex: PFAST_MUTEX) {
    _ExAcquireFastMutex(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExReleaseFastMutex(FastMutex: PFAST_MUTEX) {
    _ExReleaseFastMutex(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExReleaseFastMutex(FastMutex: PFAST_MUTEX) {
    _ExReleaseFastMutex(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX) {
    _ExAcquireFastMutexUnsafe(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX) {
    _ExAcquireFastMutexUnsafe(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExReleaseFastMutexUnsafe(FastMutex: PFAST_MUTEX) {
    _ExReleaseFastMutexUnsafe(FastMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExReleaseFastMutexUnsafe(FastMutex: PFAST_MUTEX) {
    _ExReleaseFastMutexUnsafe(FastMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn ExTryToAcquireFastMutex(FastMutex: PFAST_MUTEX) -> BOOLEAN {
    _ExTryToAcquireFastMutex(FastMutex)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ExTryToAcquireFastMutex(FastMutex: PFAST_MUTEX) -> BOOLEAN {
    _ExTryToAcquireFastMutex(FastMutex)
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/fmutex.c
//  * PURPOSE:         Implements fast mutexes
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Undefine some macros we implement here */
// #undef ExEnterCriticalRegionAndAcquireFastMutexUnsafe
// #undef ExReleaseFastMutexUnsafeAndLeaveCriticalRegion
// #undef ExAcquireFastMutex
// #undef ExReleaseFastMutex
// #undef ExAcquireFastMutexUnsafe
// #undef ExReleaseFastMutexUnsafe
// #undef ExTryToAcquireFastMutex
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExEnterCriticalRegionAndAcquireFastMutexUnsafe(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Call the inline */
//     _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Call the inline */
//     _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExAcquireFastMutex(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Call the inline */
//     _ExAcquireFastMutex(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExReleaseFastMutex(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Call the inline */
//     _ExReleaseFastMutex(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExAcquireFastMutexUnsafe(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Acquire the mutex unsafely */
//     _ExAcquireFastMutexUnsafe(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FASTCALL
// ExReleaseFastMutexUnsafe(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Release the mutex unsafely */
//     _ExReleaseFastMutexUnsafe(FastMutex);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// FASTCALL
// ExTryToAcquireFastMutex(IN OUT PFAST_MUTEX FastMutex)
// {
//     /* Call the inline */
//     return _ExTryToAcquireFastMutex(FastMutex);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
