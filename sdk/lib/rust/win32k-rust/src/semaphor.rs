use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExDeleteResourceLite, ExEnterCriticalRegionAndAcquireResourceExclusive, ExEnterCriticalRegionAndAcquireResourceShared, ExInitializeResourceLite, ExIsResourceAcquiredExclusiveLite, ExReleaseResourceAndLeaveCriticalRegion, PsGetCurrentThread, PsGetThreadWin32Thread};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the preamble.
// The C code contains only SAL annotations and macro definitions
// that do not translate to Rust declarations.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngReleaseSemaphore(
    hsem: HSEMAPHORE
) {
    let W32Thread: PTHREADINFO;

    if hsem.is_null() {
        return;
    }

    W32Thread = PsGetThreadWin32Thread(PsGetCurrentThread());
    if !W32Thread.is_null() {
        (*W32Thread).dwEngAcquireCount = (*W32Thread).dwEngAcquireCount.wrapping_sub(1);
    }
    ExReleaseResourceAndLeaveCriticalRegion(hsem as PERESOURCE);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngReleaseSemaphore(
    hsem: HSEMAPHORE
) {
    let W32Thread: PTHREADINFO;

    if hsem.is_null() {
        return;
    }

    W32Thread = PsGetThreadWin32Thread(PsGetCurrentThread());
    if !W32Thread.is_null() {
        (*W32Thread).dwEngAcquireCount = (*W32Thread).dwEngAcquireCount.wrapping_sub(1);
    }
    ExReleaseResourceAndLeaveCriticalRegion(hsem as PERESOURCE);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngAcquireSemaphoreShared(
    hsem: HSEMAPHORE
) {
    let pti: PTHREADINFO;

    ExEnterCriticalRegionAndAcquireResourceShared(hsem as PERESOURCE);
    pti = PsGetThreadWin32Thread(PsGetCurrentThread());
    if !pti.is_null() {
        (*pti).dwEngAcquireCount = (*pti).dwEngAcquireCount.wrapping_add(1);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngAcquireSemaphoreShared(
    hsem: HSEMAPHORE
) {
    let pti: PTHREADINFO;

    ExEnterCriticalRegionAndAcquireResourceShared(hsem as PERESOURCE);
    pti = PsGetThreadWin32Thread(PsGetCurrentThread());
    if !pti.is_null() {
        (*pti).dwEngAcquireCount = (*pti).dwEngAcquireCount.wrapping_add(1);
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngDeleteSemaphore(
    hsem: HSEMAPHORE
) {
    ExDeleteResourceLite(hsem as PERESOURCE);
    ExFreePoolWithTag(hsem as PVOID, GDITAG_SEMAPHORE);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngDeleteSemaphore(
    hsem: HSEMAPHORE
) {
    ExDeleteResourceLite(hsem as PERESOURCE);
    ExFreePoolWithTag(hsem as PVOID, GDITAG_SEMAPHORE);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngIsSemaphoreOwned(
    hsem: HSEMAPHORE
) -> i32 {
    (*(hsem as PERESOURCE)).ActiveCount > 0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngIsSemaphoreOwned(
    hsem: HSEMAPHORE
) -> i32 {
    (*(hsem as PERESOURCE)).ActiveCount > 0
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngIsSemaphoreOwnedByCurrentThread(
    hsem: HSEMAPHORE,
) -> i32 {
    ExIsResourceAcquiredExclusiveLite(hsem as PERESOURCE)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngIsSemaphoreOwnedByCurrentThread(
    hsem: HSEMAPHORE,
) -> i32 {
    ExIsResourceAcquiredExclusiveLite(hsem as PERESOURCE)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngInitializeSafeSemaphore(
    Semaphore: *mut ENGSAFESEMAPHORE,
) -> i32 {
    let hSem: HSEMAPHORE;

    if InterlockedIncrement(&mut (*Semaphore).lCount) == 1 {
        hSem = EngCreateSemaphore();
        if hSem == 0 {
            InterlockedDecrement(&mut (*Semaphore).lCount);
            return 0;
        }
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*Semaphore).hsem) as *mut *mut core::ffi::c_void,
            hSem as *mut core::ffi::c_void,
        );
    } else {
        while (*Semaphore).hsem == core::ptr::null_mut() {}
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngInitializeSafeSemaphore(
    Semaphore: *mut ENGSAFESEMAPHORE,
) -> i32 {
    let hSem: HSEMAPHORE;

    if InterlockedIncrement(&mut (*Semaphore).lCount) == 1 {
        hSem = EngCreateSemaphore();
        if hSem == 0 {
            InterlockedDecrement(&mut (*Semaphore).lCount);
            return 0;
        }
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*Semaphore).hsem) as *mut *mut core::ffi::c_void,
            hSem as *mut core::ffi::c_void,
        );
    } else {
        while (*Semaphore).hsem == core::ptr::null_mut() {}
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngDeleteSafeSemaphore(
    pssem: *mut ENGSAFESEMAPHORE
) {
    if InterlockedDecrement(&mut (*pssem).lCount) == 0 {
        EngDeleteSemaphore((*pssem).hsem);
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*pssem).hsem) as *mut PVOID,
            core::ptr::null_mut()
        );
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngDeleteSafeSemaphore(
    pssem: *mut ENGSAFESEMAPHORE
) {
    if InterlockedDecrement(&mut (*pssem).lCount) == 0 {
        EngDeleteSemaphore((*pssem).hsem);
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*pssem).hsem) as *mut PVOID,
            core::ptr::null_mut()
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// __drv_allocatesMem(Mem)
// _Post_writable_byte_size_(sizeof(ERESOURCE))
// HSEMAPHORE
// APIENTRY
// EngCreateSemaphore(
//     VOID)
// {
//     // www.osr.com/ddk/graphics/gdifncs_95lz.htm
//     PERESOURCE psem = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(ERESOURCE),
//                                             GDITAG_SEMAPHORE);
//     if (!psem)
//         return NULL;
//
//     if (!NT_SUCCESS(ExInitializeResourceLite(psem)))
//     {
//         ExFreePoolWithTag(psem, GDITAG_SEMAPHORE );
//         return NULL;
//     }
//
//     return (HSEMAPHORE)psem;
// }
//
// /*
//  * @implemented
//  */
// _Requires_lock_not_held_(*hsem)
// _Acquires_exclusive_lock_(*hsem)
// _Acquires_lock_(_Global_critical_region_)
// VOID
// APIENTRY
// EngAcquireSemaphore(
//     _Inout_ HSEMAPHORE hsem)
// {
//     // www.osr.com/ddk/graphics/gdifncs_14br.htm
//     PTHREADINFO W32Thread;
//
//     /* On Windows a NULL hsem is ignored */
//     if (hsem == NULL)
//     {
//         DPRINT1("EngAcquireSemaphore called with hsem == NULL!\n");
//         return;
//     }
//
//     ExEnterCriticalRegionAndAcquireResourceExclusive((PERESOURCE)hsem);
//     W32Thread = PsGetThreadWin32Thread(PsGetCurrentThread());
//     if (W32Thread) W32Thread->dwEngAcquireCount++;
// }
//
// /*
//  * @implemented
//  */
// _Requires_lock_held_(*hsem)
// _Releases_lock_(*hsem)
// _Releases_lock_(_Global_critical_region_)
// VOID
// APIENTRY
// EngReleaseSemaphore(
//     _Inout_ HSEMAPHORE hsem)
// {
//     // www.osr.com/ddk/graphics/gdifncs_5u3r.htm
//     PTHREADINFO W32Thread;
//
//     /* On Windows a NULL hsem is ignored */
//     if (hsem == NULL)
//     {
//         DPRINT1("EngReleaseSemaphore called with hsem == NULL!\n");
//         return;
//     }
//
//     W32Thread = PsGetThreadWin32Thread(PsGetCurrentThread());
//     if (W32Thread) --W32Thread->dwEngAcquireCount;
//     ExReleaseResourceAndLeaveCriticalRegion((PERESOURCE)hsem);
// }
//
// _Acquires_lock_(_Global_critical_region_)
// _Requires_lock_not_held_(*hsem)
// _Acquires_shared_lock_(*hsem)
// VOID
// NTAPI
// EngAcquireSemaphoreShared(
//      _Inout_ HSEMAPHORE hsem)
// {
//     PTHREADINFO pti;
//
//     ASSERT(hsem);
//     ExEnterCriticalRegionAndAcquireResourceShared((PERESOURCE)hsem);
//     pti = PsGetThreadWin32Thread(PsGetCurrentThread());
//     if (pti) ++pti->dwEngAcquireCount;
// }
//
// /*
//  * @implemented
//  */
// _Requires_lock_not_held_(*hsem)
// VOID
// APIENTRY
// EngDeleteSemaphore(
//     _Inout_ __drv_freesMem(Mem) HSEMAPHORE hsem)
// {
//     // www.osr.com/ddk/graphics/gdifncs_13c7.htm
//     ASSERT(hsem);
//
//     ExDeleteResourceLite((PERESOURCE)hsem);
//     ExFreePoolWithTag((PVOID)hsem, GDITAG_SEMAPHORE);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngIsSemaphoreOwned(
//     _In_ HSEMAPHORE hsem)
// {
//     // www.osr.com/ddk/graphics/gdifncs_6wmf.htm
//     ASSERT(hsem);
//     return (((PERESOURCE)hsem)->ActiveCount > 0);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngIsSemaphoreOwnedByCurrentThread(
//     _In_ HSEMAPHORE hsem)
// {
//     // www.osr.com/ddk/graphics/gdifncs_9yxz.htm
//     ASSERT(hsem);
//     return ExIsResourceAcquiredExclusiveLite((PERESOURCE)hsem);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngInitializeSafeSemaphore(
//     _Out_ ENGSAFESEMAPHORE *Semaphore)
// {
//     HSEMAPHORE hSem;
//
//     if (InterlockedIncrement(&Semaphore->lCount) == 1)
//     {
//         /* Create the semaphore */
//         hSem = EngCreateSemaphore();
//         if (hSem == 0)
//         {
//             InterlockedDecrement(&Semaphore->lCount);
//             return FALSE;
//         }
//         /* FIXME: Not thread-safe! Check result of InterlockedCompareExchangePointer
//                   and delete semaphore if already initialized! */
//         (void)InterlockedExchangePointer((volatile PVOID *)&Semaphore->hsem, hSem);
//     }
//     else
//     {
//         /* Wait for the other thread to create the semaphore */
//         ASSERT(Semaphore->lCount > 1);
//         ASSERT_IRQL_LESS_OR_EQUAL(PASSIVE_LEVEL);
//         while (Semaphore->hsem == NULL);
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngDeleteSafeSemaphore(
//     _Inout_ _Post_invalid_ ENGSAFESEMAPHORE *pssem)
// {
//     if (InterlockedDecrement(&pssem->lCount) == 0)
//     {
//         /* FIXME: Not thread-safe! Use result of InterlockedCompareExchangePointer! */
//         EngDeleteSemaphore(pssem->hsem);
//         (void)InterlockedExchangePointer((volatile PVOID *)&pssem->hsem, NULL);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
