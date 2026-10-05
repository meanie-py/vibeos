use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag,
    ExDeleteResourceLite,
    ExEnterCriticalRegionAndAcquireResourceExclusive,
    ExEnterCriticalRegionAndAcquireResourceShared,
    ExFreePoolWithTag,
    ExInitializeResourceLite,
    ExIsResourceAcquiredExclusiveLite,
    ExReleaseResourceAndLeaveCriticalRegion,
    PsGetCurrentThread,
    PsGetThreadWin32Thread,
};
use ntkrnl_sys::interlocked::{
    InterlockedDecrement,
    InterlockedExchangePointer,
    InterlockedIncrement,
};
use ntkrnl_sys::base::NT_SUCCESS;

// Cross-crate kernel-type alias (see §5 of win32k_translation_prompt.md):
// `_ERESOURCE` is bindgen'd in BOTH `win32k_sys::win32k` and
// `ntkrnl_sys::ddk` as DISTINCT types. The kernel APIs we call here all
// live in `ntkrnl_sys::ddk` and want THEIR `PERESOURCE`, so use the
// ddk-qualified alias for every HSEMAPHORE→PERESOURCE cast in this file.
type DdkPERESOURCE = ntkrnl_sys::ddk::PERESOURCE;

// GDITAG_SEMAPHORE = 'mesG' (win32ss/user/ntuser/tags.h:159).
const GDITAG_SEMAPHORE: u32 = 0x6d655347u32;

// ---- file-scope preamble (translated) ----
// No file-scope preamble declarations to translate.
// The provided C code contains only SAL annotations and a #define,
// neither of which produce Rust declarations.

#[no_mangle]
pub unsafe extern "system" fn EngCreateSemaphore() -> HSEMAPHORE {
    // www.osr.com/ddk/graphics/gdifncs_95lz.htm
    let psem: DdkPERESOURCE = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ntkrnl_sys::ddk::_ERESOURCE>() as SIZE_T,
        GDITAG_SEMAPHORE,
    ) as DdkPERESOURCE;
    if psem.is_null() {
        return core::ptr::null_mut();
    }

    if !NT_SUCCESS(ExInitializeResourceLite(psem)) {
        ExFreePoolWithTag(psem as *mut core::ffi::c_void, GDITAG_SEMAPHORE);
        return core::ptr::null_mut();
    }

    psem as HSEMAPHORE
}

#[no_mangle]
pub unsafe extern "system" fn EngAcquireSemaphore(hsem: HSEMAPHORE) {
    // www.osr.com/ddk/graphics/gdifncs_14br.htm
    // On Windows a NULL hsem is ignored
    if hsem.is_null() {
        return;
    }

    ExEnterCriticalRegionAndAcquireResourceExclusive(hsem as DdkPERESOURCE);
    // dwEngAcquireCount lives on W32THREAD (THREADINFO's prefix). The
    // bindgen'd _THREADINFO buries it in __bindgen_padding_0, so go via
    // PW32THREAD.
    let w32thread = PsGetThreadWin32Thread(PsGetCurrentThread()) as PW32THREAD;
    if !w32thread.is_null() {
        (*w32thread).dwEngAcquireCount = (*w32thread).dwEngAcquireCount.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EngReleaseSemaphore(hsem: HSEMAPHORE) {
    if hsem.is_null() {
        return;
    }

    let w32thread = PsGetThreadWin32Thread(PsGetCurrentThread()) as PW32THREAD;
    if !w32thread.is_null() {
        (*w32thread).dwEngAcquireCount = (*w32thread).dwEngAcquireCount.wrapping_sub(1);
    }
    ExReleaseResourceAndLeaveCriticalRegion(hsem as DdkPERESOURCE);
}

#[no_mangle]
pub unsafe extern "system" fn EngAcquireSemaphoreShared(hsem: HSEMAPHORE) {
    ExEnterCriticalRegionAndAcquireResourceShared(hsem as DdkPERESOURCE);
    let pti = PsGetThreadWin32Thread(PsGetCurrentThread()) as PW32THREAD;
    if !pti.is_null() {
        (*pti).dwEngAcquireCount = (*pti).dwEngAcquireCount.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EngDeleteSemaphore(hsem: HSEMAPHORE) {
    ExDeleteResourceLite(hsem as DdkPERESOURCE);
    ExFreePoolWithTag(hsem as *mut core::ffi::c_void, GDITAG_SEMAPHORE);
}

#[no_mangle]
pub unsafe extern "system" fn EngIsSemaphoreOwned(hsem: HSEMAPHORE) -> BOOL {
    ((*(hsem as DdkPERESOURCE)).ActiveCount > 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EngIsSemaphoreOwnedByCurrentThread(hsem: HSEMAPHORE) -> BOOL {
    ExIsResourceAcquiredExclusiveLite(hsem as DdkPERESOURCE) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EngInitializeSafeSemaphore(
    Semaphore: *mut ENGSAFESEMAPHORE,
) -> BOOL {
    if InterlockedIncrement(&mut (*Semaphore).lCount) == 1 {
        let hSem = EngCreateSemaphore();
        if hSem.is_null() {
            InterlockedDecrement(&mut (*Semaphore).lCount);
            return 0;
        }
        InterlockedExchangePointer(
            &mut (*Semaphore).hsem as *mut HSEMAPHORE as *mut *mut core::ffi::c_void,
            hSem as *mut core::ffi::c_void,
        );
    } else {
        // Volatile wait: a plain load is hoisted out of the loop by LLVM,
        // leaving an unconditional spin (`b .` on arm64) if the creator
        // thread hasn't published hsem yet.
        while core::ptr::read_volatile(core::ptr::addr_of!((*Semaphore).hsem)).is_null() {
            core::hint::spin_loop();
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn EngDeleteSafeSemaphore(pssem: *mut ENGSAFESEMAPHORE) {
    if InterlockedDecrement(&mut (*pssem).lCount) == 0 {
        EngDeleteSemaphore((*pssem).hsem);
        InterlockedExchangePointer(
            &mut (*pssem).hsem as *mut HSEMAPHORE as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
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
