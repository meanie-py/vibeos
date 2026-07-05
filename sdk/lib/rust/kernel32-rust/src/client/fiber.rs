use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{BaseCreateStack, BaseInitializeContext, ExitThread, RtlZeroMemory};

extern "system" {
    fn GetProcessHeap() -> HANDLE;
}

// FIBER, PFIBER, RTL_FLS_DATA, PRTL_FLS_DATA, LIST_ENTRY, PLIST_ENTRY,
// PFLS_CALLBACK_FUNCTION, LPFIBER_START_ROUTINE, LPVOID, PTEB, PPEB,
// PACTIVATION_CONTEXT_STACK, and all the FLS_*/FIBER_*/CONTEXT_*/ERROR_*/HEAP_*/
// MEM_* constants come from ntdll_sys::nt::* (bindgen) — do not redefine.

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// EXCEPTION_CHAIN_END is `((PEXCEPTION_REGISTRATION_RECORD)-1)` (winnt.h) — not
// bindgen-evaluable.
const EXCEPTION_CHAIN_END: *mut _EXCEPTION_REGISTRATION_RECORD =
    !0usize as *mut _EXCEPTION_REGISTRATION_RECORD;

// PEB.FlsCallback is typed `*mut PVOID` by bindgen, but the slots hold
// PFLS_CALLBACK_FUNCTION values (C indexes it as such). View it that way.
#[inline]
unsafe fn fls_callbacks(Peb: PPEB) -> *mut PFLS_CALLBACK_FUNCTION {
    (*Peb).FlsCallback as *mut PFLS_CALLBACK_FUNCTION
}

#[no_mangle]
pub unsafe extern "system" fn BaseRundownFls(FlsData: PVOID) {
    let mut n: u32;
    let FlsHighIndex: u32;
    let pFlsData: PRTL_FLS_DATA;
    let mut lpCallback: PFLS_CALLBACK_FUNCTION;

    pFlsData = FlsData as PRTL_FLS_DATA;

    RtlAcquirePebLock();
    FlsHighIndex = (*NtCurrentPeb()).FlsHighIndex;
    RemoveEntryList(&mut (*pFlsData).ListEntry);
    RtlReleasePebLock();

    n = 1;
    while n <= FlsHighIndex {
        lpCallback = *fls_callbacks(NtCurrentPeb()).add(n as usize);
        if lpCallback.is_some() && !(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize)).is_null() {
            (lpCallback.unwrap())(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize));
        }
        n += 1;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, FlsData);
}

#[no_mangle]
pub unsafe extern "system" fn ConvertFiberToThread() -> BOOL {
    let Teb: PTEB;
    let FiberData: PFIBER;

    Teb = NtCurrentTeb();
    if (*Teb).HasFiberData == 0 {
        SetLastError(ERROR_ALREADY_THREAD);
        return 0i32;
    }

    (*Teb).HasFiberData = FALSE as BOOLEAN;
    FiberData = (*Teb).NtTib.__bindgen_anon_1.FiberData as PFIBER;
    (*Teb).NtTib.__bindgen_anon_1.FiberData = core::ptr::null_mut();

    RtlFreeHeap(RtlGetProcessHeap(), 0, FiberData as LPVOID);

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn ConvertThreadToFiberEx(
    lpParameter: LPVOID,
    dwFlags: DWORD,
) -> LPVOID {
    let Teb: PTEB;
    let Fiber: PFIBER;

    /* Check for invalid flags */
    if (dwFlags & !FIBER_FLAG_FLOAT_SWITCH) != 0 {
        /* Fail */
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Are we already a fiber? */
    Teb = NtCurrentTeb();
    if (*Teb).HasFiberData != 0 {
        /* Fail */
        SetLastError(ERROR_ALREADY_FIBER);
        return core::ptr::null_mut();
    }

    /* Allocate the fiber */
    Fiber = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<FIBER>() as SIZE_T,
    ) as PFIBER;
    if Fiber.is_null() {
        /* Fail */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    /* Copy some contextual data from the thread to the fiber */
    (*Fiber).FiberData = lpParameter;
    (*Fiber).ExceptionList = (*Teb).NtTib.ExceptionList;
    (*Fiber).StackBase = (*Teb).NtTib.StackBase;
    (*Fiber).StackLimit = (*Teb).NtTib.StackLimit;
    (*Fiber).DeallocationStack = (*Teb).DeallocationStack;
    (*Fiber).FlsData = (*Teb).FlsData;
    (*Fiber).GuaranteedStackBytes = (*Teb).GuaranteedStackBytes;
    (*Fiber).ActivationContextStackPointer = (*Teb).ActivationContextStackPointer;

    /* Save FPU State if requested, otherwise just the basic registers */
    (*Fiber).FiberContext.ContextFlags = if (dwFlags & FIBER_FLAG_FLOAT_SWITCH) != 0 {
        CONTEXT_FULL | CONTEXT_FLOATING_POINT
    } else {
        CONTEXT_FULL
    };

    /* Associate the fiber to the current thread */
    (*Teb).NtTib.__bindgen_anon_1.FiberData = Fiber as LPVOID;
    (*Teb).HasFiberData = TRUE as BOOLEAN;

    /* Return opaque fiber data */
    return Fiber as LPVOID;
}

#[no_mangle]
pub unsafe extern "system" fn ConvertThreadToFiber(
    lpParameter: LPVOID,
) -> LPVOID {
    return ConvertThreadToFiberEx(lpParameter, 0);
}

#[no_mangle]
pub unsafe extern "system" fn CreateFiber(
    dwStackSize: SIZE_T,
    lpStartAddress: LPFIBER_START_ROUTINE,
    lpParameter: LPVOID,
) -> LPVOID {
    return CreateFiberEx(dwStackSize, 0, 0, lpStartAddress, lpParameter);
}

#[no_mangle]
pub unsafe extern "system" fn CreateFiberEx(
    dwStackCommitSize: SIZE_T,
    dwStackReserveSize: SIZE_T,
    dwFlags: DWORD,
    lpStartAddress: LPFIBER_START_ROUTINE,
    lpParameter: LPVOID,
) -> LPVOID {
    let Fiber: PFIBER;
    let mut Status: NTSTATUS;
    let mut InitialTeb: INITIAL_TEB = core::mem::zeroed();
    let mut ActivationContextStackPointer: PACTIVATION_CONTEXT_STACK;

    /* Check for invalid flags */
    if (dwFlags & !FIBER_FLAG_FLOAT_SWITCH) != 0 {
        /* Fail */
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Allocate the Activation Context Stack */
    ActivationContextStackPointer = core::ptr::null_mut();
    Status = RtlAllocateActivationContextStack(&mut ActivationContextStackPointer);
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Allocate the fiber */
    Fiber = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<FIBER>() as SIZE_T,
    ) as PFIBER;
    if Fiber.is_null() {
        /* Free the activation context stack */
        RtlFreeActivationContextStack(ActivationContextStackPointer);

        /* Fail */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    /* Create the stack for the fiber */
    Status = BaseCreateStack(
        NtCurrentProcess(),
        dwStackCommitSize,
        dwStackReserveSize,
        &mut InitialTeb,
    );
    if !NT_SUCCESS(Status) {
        /* Free the fiber */
        RtlFreeHeap(
            GetProcessHeap(),
            0,
            Fiber as LPVOID,
        );

        /* Free the activation context stack */
        RtlFreeActivationContextStack(ActivationContextStackPointer);

        /* Failure */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Clear the context */
    RtlZeroMemory(
        &mut (*Fiber).FiberContext as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<CONTEXT>() as SIZE_T,
    );

    /* Copy the data into the fiber */
    (*Fiber).StackBase = InitialTeb.StackBase;
    (*Fiber).StackLimit = InitialTeb.StackLimit;
    (*Fiber).DeallocationStack = InitialTeb.AllocatedStackBase;
    (*Fiber).FiberData = lpParameter;
    (*Fiber).ExceptionList = EXCEPTION_CHAIN_END;
    (*Fiber).GuaranteedStackBytes = 0;
    (*Fiber).FlsData = core::ptr::null_mut();
    (*Fiber).ActivationContextStackPointer = ActivationContextStackPointer;

    /* Save FPU State if requested, otherwise just the basic registers */
    (*Fiber).FiberContext.ContextFlags = if (dwFlags & FIBER_FLAG_FLOAT_SWITCH) != 0 {
        CONTEXT_FULL | CONTEXT_FLOATING_POINT
    } else {
        CONTEXT_FULL
    };

    /* Initialize the context for the fiber */
    BaseInitializeContext(
        &mut (*Fiber).FiberContext,
        lpParameter,
        lpStartAddress.map_or(core::ptr::null_mut(), |f| f as PVOID),
        InitialTeb.StackBase,
        2,
    );

    /* Return the Fiber */
    Fiber as LPVOID
}
#[no_mangle]
pub unsafe extern "system" fn DeleteFiber(lpFiber: LPVOID) {
    let mut Size: SIZE_T;
    let Fiber: PFIBER;
    let Teb: PTEB;

    Teb = NtCurrentTeb();
    Fiber = lpFiber as PFIBER;
    if (*Teb).HasFiberData != 0 && (*Teb).NtTib.__bindgen_anon_1.FiberData == Fiber as PVOID {
        ExitThread(1);
    }

    Size = 0;
    NtFreeVirtualMemory(
        NtCurrentProcess(),
        &mut (*Fiber).DeallocationStack as *mut _ as *mut *mut _,
        &mut Size as *mut _,
        MEM_RELEASE,
    );

    if !(*Fiber).FlsData.is_null() {
        BaseRundownFls((*Fiber).FlsData);
    }

    RtlFreeActivationContextStack((*Fiber).ActivationContextStackPointer);

    RtlFreeHeap(RtlGetProcessHeap(), 0, lpFiber);
}

#[no_mangle]
pub unsafe extern "system" fn IsThreadAFiber() -> BOOL {
    (*NtCurrentTeb()).HasFiberData as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn FlsAlloc(lpCallback: PFLS_CALLBACK_FUNCTION) -> DWORD {
    let mut dwFlsIndex: DWORD;
    let Peb: PPEB = NtCurrentPeb();
    let mut pFlsData: PRTL_FLS_DATA;

    RtlAcquirePebLock();

    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;

    if (*Peb).FlsCallback == core::ptr::null_mut()
        && {
            (*Peb).FlsCallback = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                ((FLS_MAXIMUM_AVAILABLE as usize) * core::mem::size_of::<LPVOID>()) as SIZE_T,
            ) as *mut _;
            (*Peb).FlsCallback.is_null()
        }
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        dwFlsIndex = FLS_OUT_OF_INDEXES;
    } else {
        dwFlsIndex = RtlFindClearBitsAndSet(((*Peb).FlsBitmap as PRTL_BITMAP), 1, 1);
        if dwFlsIndex != FLS_OUT_OF_INDEXES {
            if pFlsData.is_null()
                && {
                    pFlsData = RtlAllocateHeap(
                        RtlGetProcessHeap(),
                        HEAP_ZERO_MEMORY,
                        core::mem::size_of::<RTL_FLS_DATA>() as SIZE_T,
                    ) as PRTL_FLS_DATA;
                    pFlsData.is_null()
                }
            {
                RtlClearBits(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1);
                dwFlsIndex = FLS_OUT_OF_INDEXES;
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                if (*NtCurrentTeb()).FlsData.is_null() {
                    (*NtCurrentTeb()).FlsData = pFlsData as PVOID;
                    InsertTailList(&mut (*Peb).FlsListHead, &mut (*pFlsData).ListEntry);
                }

                *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = core::ptr::null_mut();
                *fls_callbacks(Peb).add(dwFlsIndex as usize) = lpCallback;

                if dwFlsIndex > (*Peb).FlsHighIndex {
                    (*Peb).FlsHighIndex = dwFlsIndex;
                }
            }
        } else {
            SetLastError(ERROR_NO_MORE_ITEMS);
        }
    }
    RtlReleasePebLock();
    return dwFlsIndex;
}

#[no_mangle]
pub unsafe extern "system" fn FlsFree(dwFlsIndex: DWORD) -> BOOL {
    let mut ret: BOOL;
    let Peb: PPEB = NtCurrentPeb();

    if dwFlsIndex >= FLS_MAXIMUM_AVAILABLE as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    RtlAcquirePebLock();

    // _SEH2_TRY
    {
        ret = if RtlAreBitsSet(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1) != 0 { 1 } else { 0 };
        if ret != 0 {
            let mut Entry: PLIST_ENTRY;
            let lpCallback: PFLS_CALLBACK_FUNCTION;

            RtlClearBits(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1);
            lpCallback = *fls_callbacks(Peb).add(dwFlsIndex as usize);

            Entry = (*Peb).FlsListHead.Flink;
            while Entry != &mut (*Peb).FlsListHead as *mut _ as PLIST_ENTRY {
                let pFlsData: PRTL_FLS_DATA = {
                    let _addr = Entry as *mut u8;
                    let _offset = core::mem::offset_of!(RTL_FLS_DATA, ListEntry) as isize;
                    _addr.offset(-_offset) as *mut RTL_FLS_DATA
                };
                if *core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) != core::ptr::null_mut() {
                    if let Some(cb) = lpCallback {
                        cb(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize));
                    }
                    *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = core::ptr::null_mut();
                }
                Entry = (*Entry).Flink;
            }
            *fls_callbacks(Peb).add(dwFlsIndex as usize) = None;
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    // _SEH2_FINALLY
    {
        RtlReleasePebLock();
    }
    // _SEH2_END

    ret
}

#[no_mangle]
pub unsafe extern "system" fn FlsGetValue(dwFlsIndex: DWORD) -> LPVOID {
    let pFlsData: PRTL_FLS_DATA;
    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;
    if dwFlsIndex == 0 || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE || pFlsData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }
    SetLastError(ERROR_SUCCESS);
    return *core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize);
}

#[no_mangle]
pub unsafe extern "system" fn FlsSetValue(
    dwFlsIndex: DWORD,
    lpFlsData: PVOID,
) -> BOOL {
    let mut pFlsData: PRTL_FLS_DATA;

    if dwFlsIndex == 0 || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;

    if (*NtCurrentTeb()).FlsData.is_null()
        && {
            (*NtCurrentTeb()).FlsData = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<RTL_FLS_DATA>() as SIZE_T,
            );
            (*NtCurrentTeb()).FlsData.is_null()
        }
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    if pFlsData.is_null() {
        pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;
        RtlAcquirePebLock();
        InsertTailList(
            &mut (*NtCurrentPeb()).FlsListHead,
            &mut (*pFlsData).ListEntry,
        );
        RtlReleasePebLock();
    }
    *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = lpFlsData;
    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Libraries
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Fiber Implementation
//  * COPYRIGHT:   Copyright 2005-2011 Alex Ionescu (alex@relsoft.net)
//  *              Copyright 2003-2008 KJK::Hyperion (noog@libero.it)
//  *              Copyright 2018 Mark Jansen (mark.jansen@reactos.org)
//  */
//
// #include <k32.h>
// #include <ndk/rtltypes.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _M_IX86
// C_ASSERT(FIELD_OFFSET(FIBER, ExceptionList) == 0x04);
// C_ASSERT(FIELD_OFFSET(FIBER, StackBase) == 0x08);
// C_ASSERT(FIELD_OFFSET(FIBER, StackLimit) == 0x0C);
// C_ASSERT(FIELD_OFFSET(FIBER, DeallocationStack) == 0x10);
// C_ASSERT(FIELD_OFFSET(FIBER, FiberContext) == 0x14);
// C_ASSERT(FIELD_OFFSET(FIBER, GuaranteedStackBytes) == 0x2E0);
// C_ASSERT(FIELD_OFFSET(FIBER, FlsData) == 0x2E4);
// C_ASSERT(FIELD_OFFSET(FIBER, ActivationContextStackPointer) == 0x2E8);
// C_ASSERT(RTL_FLS_MAXIMUM_AVAILABLE == FLS_MAXIMUM_AVAILABLE);
// #endif // _M_IX86
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// WINAPI
// BaseRundownFls( PVOID FlsData)
// {
//     ULONG n, FlsHighIndex;
//     PRTL_FLS_DATA pFlsData;
//     PFLS_CALLBACK_FUNCTION lpCallback;
//
//     pFlsData = FlsData;
//
//     RtlAcquirePebLock();
//     FlsHighIndex = NtCurrentPeb()->FlsHighIndex;
//     RemoveEntryList(&pFlsData->ListEntry);
//     RtlReleasePebLock();
//
//     for (n = 1; n <= FlsHighIndex; ++n)
//     {
//         lpCallback = NtCurrentPeb()->FlsCallback[n];
//         if (lpCallback && pFlsData->Data[n])
//         {
//             lpCallback(pFlsData->Data[n]);
//         }
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FlsData);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ConvertFiberToThread(VOID)
// {
//     PTEB Teb;
//     PFIBER FiberData;
//     DPRINT1("Converting Fiber to Thread\n");
//
//     /* Check if the thread is already not a fiber */
//     Teb = NtCurrentTeb();
//     if (!Teb->HasFiberData)
//     {
//         /* Fail */
//         SetLastError(ERROR_ALREADY_THREAD);
//         return FALSE;
//     }
//
//     /* This thread won't run a fiber anymore */
//     Teb->HasFiberData = FALSE;
//     FiberData = Teb->NtTib.FiberData;
//     Teb->NtTib.FiberData = NULL;
//
//     /* Free the fiber */
//     ASSERT(FiberData != NULL);
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FiberData);
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// ConvertThreadToFiberEx( LPVOID lpParameter,
//                         DWORD dwFlags)
// {
//     PTEB Teb;
//     PFIBER Fiber;
//     DPRINT1("Converting Thread to Fiber\n");
//
//     /* Check for invalid flags */
//     if (dwFlags & ~FIBER_FLAG_FLOAT_SWITCH)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Are we already a fiber? */
//     Teb = NtCurrentTeb();
//     if (Teb->HasFiberData)
//     {
//         /* Fail */
//         SetLastError(ERROR_ALREADY_FIBER);
//         return NULL;
//     }
//
//     /* Allocate the fiber */
//     Fiber = RtlAllocateHeap(RtlGetProcessHeap(),
//                             0,
//                             sizeof(FIBER));
//     if (!Fiber)
//     {
//         /* Fail */
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Copy some contextual data from the thread to the fiber */
//     Fiber->FiberData = lpParameter;
//     Fiber->ExceptionList = Teb->NtTib.ExceptionList;
//     Fiber->StackBase = Teb->NtTib.StackBase;
//     Fiber->StackLimit = Teb->NtTib.StackLimit;
//     Fiber->DeallocationStack = Teb->DeallocationStack;
//     Fiber->FlsData = Teb->FlsData;
//     Fiber->GuaranteedStackBytes = Teb->GuaranteedStackBytes;
//     Fiber->ActivationContextStackPointer = Teb->ActivationContextStackPointer;
//
//     /* Save FPU State if requested, otherwise just the basic registers */
//     Fiber->FiberContext.ContextFlags = (dwFlags & FIBER_FLAG_FLOAT_SWITCH) ?
//                                        (CONTEXT_FULL | CONTEXT_FLOATING_POINT) :
//                                        CONTEXT_FULL;
//
//     /* Associate the fiber to the current thread */
//     Teb->NtTib.FiberData = Fiber;
//     Teb->HasFiberData = TRUE;
//
//     /* Return opaque fiber data */
//     return (LPVOID)Fiber;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// ConvertThreadToFiber( LPVOID lpParameter)
// {
//     /* Call the newer function */
//     return ConvertThreadToFiberEx(lpParameter,
//                                   0);
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// CreateFiber( SIZE_T dwStackSize,
//              LPFIBER_START_ROUTINE lpStartAddress,
//              LPVOID lpParameter)
// {
//     /* Call the Newer Function */
//     return CreateFiberEx(dwStackSize,
//                          0,
//                          0,
//                          lpStartAddress,
//                          lpParameter);
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// CreateFiberEx( SIZE_T dwStackCommitSize,
//                SIZE_T dwStackReserveSize,
//                DWORD dwFlags,
//                LPFIBER_START_ROUTINE lpStartAddress,
//                LPVOID lpParameter)
// {
//     PFIBER Fiber;
//     NTSTATUS Status;
//     INITIAL_TEB InitialTeb;
//     PACTIVATION_CONTEXT_STACK ActivationContextStackPointer;
//     DPRINT("Creating Fiber\n");
//
//     /* Check for invalid flags */
//     if (dwFlags & ~FIBER_FLAG_FLOAT_SWITCH)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Allocate the Activation Context Stack */
//     ActivationContextStackPointer = NULL;
//     Status = RtlAllocateActivationContextStack(&ActivationContextStackPointer);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Allocate the fiber */
//     Fiber = RtlAllocateHeap(RtlGetProcessHeap(),
//                             0,
//                             sizeof(FIBER));
//     if (!Fiber)
//     {
//         /* Free the activation context stack */
//         RtlFreeActivationContextStack(ActivationContextStackPointer);
//
//         /* Fail */
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Create the stack for the fiber */
//     Status = BaseCreateStack(NtCurrentProcess(),
//                               dwStackCommitSize,
//                               dwStackReserveSize,
//                               &InitialTeb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the fiber */
//         RtlFreeHeap(GetProcessHeap(),
//                     0,
//                     Fiber);
//
//         /* Free the activation context stack */
//         RtlFreeActivationContextStack(ActivationContextStackPointer);
//
//         /* Failure */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Clear the context */
//     RtlZeroMemory(&Fiber->FiberContext,
//                   sizeof(CONTEXT));
//
//     /* Copy the data into the fiber */
//     Fiber->StackBase = InitialTeb.StackBase;
//     Fiber->StackLimit = InitialTeb.StackLimit;
//     Fiber->DeallocationStack = InitialTeb.AllocatedStackBase;
//     Fiber->FiberData = lpParameter;
//     Fiber->ExceptionList = EXCEPTION_CHAIN_END;
//     Fiber->GuaranteedStackBytes = 0;
//     Fiber->FlsData = NULL;
//     Fiber->ActivationContextStackPointer = ActivationContextStackPointer;
//
//     /* Save FPU State if requested, otherwise just the basic registers */
//     Fiber->FiberContext.ContextFlags = (dwFlags & FIBER_FLAG_FLOAT_SWITCH) ?
//                                        (CONTEXT_FULL | CONTEXT_FLOATING_POINT) :
//                                        CONTEXT_FULL;
//
//     /* Initialize the context for the fiber */
//     BaseInitializeContext(&Fiber->FiberContext,
//                           lpParameter,
//                           lpStartAddress,
//                           InitialTeb.StackBase,
//                           2);
//
//     /* Return the Fiber */
//     return Fiber;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// DeleteFiber( LPVOID lpFiber)
// {
//     SIZE_T Size;
//     PFIBER Fiber;
//     PTEB Teb;
//
//     /* Are we deleting ourselves? */
//     Teb = NtCurrentTeb();
//     Fiber = (PFIBER)lpFiber;
//     if ((Teb->HasFiberData) &&
//         (Teb->NtTib.FiberData == Fiber))
//     {
//         /* Just exit */
//         ExitThread(1);
//     }
//
//     /* Not ourselves, de-allocate the stack */
//     Size = 0 ;
//     NtFreeVirtualMemory(NtCurrentProcess(),
//                         &Fiber->DeallocationStack,
//                         &Size,
//                         MEM_RELEASE);
//
//     /* Get rid of FLS */
//     if (Fiber->FlsData) BaseRundownFls(Fiber->FlsData);
//
//     /* Get rid of the activation context stack */
//     RtlFreeActivationContextStack(Fiber->ActivationContextStackPointer);
//
//     /* Free the fiber data */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 lpFiber);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsThreadAFiber(VOID)
// {
//     /* Return flag in the TEB */
//     return NtCurrentTeb()->HasFiberData;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// FlsAlloc(PFLS_CALLBACK_FUNCTION lpCallback)
// {
//     DWORD dwFlsIndex;
//     PPEB Peb = NtCurrentPeb();
//     PRTL_FLS_DATA pFlsData;
//
//     RtlAcquirePebLock();
//
//     pFlsData = NtCurrentTeb()->FlsData;
//
//     if (!Peb->FlsCallback &&
//         !(Peb->FlsCallback = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY,
//                                              FLS_MAXIMUM_AVAILABLE * sizeof(PVOID))))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         dwFlsIndex = FLS_OUT_OF_INDEXES;
//     }
//     else
//     {
//         dwFlsIndex = RtlFindClearBitsAndSet(Peb->FlsBitmap, 1, 1);
//         if (dwFlsIndex != FLS_OUT_OF_INDEXES)
//         {
//             if (!pFlsData &&
//                 !(pFlsData = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(RTL_FLS_DATA))))
//             {
//                 RtlClearBits(Peb->FlsBitmap, dwFlsIndex, 1);
//                 dwFlsIndex = FLS_OUT_OF_INDEXES;
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             }
//             else
//             {
//                 if (!NtCurrentTeb()->FlsData)
//                 {
//                     NtCurrentTeb()->FlsData = pFlsData;
//                     InsertTailList(&Peb->FlsListHead, &pFlsData->ListEntry);
//                 }
//
//                 pFlsData->Data[dwFlsIndex] = NULL; /* clear the value */
//                 Peb->FlsCallback[dwFlsIndex] = lpCallback;
//
//                 if (dwFlsIndex > Peb->FlsHighIndex)
//                     Peb->FlsHighIndex = dwFlsIndex;
//             }
//         }
//         else
//         {
//             SetLastError(ERROR_NO_MORE_ITEMS);
//         }
//     }
//     RtlReleasePebLock();
//     return dwFlsIndex;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FlsFree(DWORD dwFlsIndex)
// {
//     BOOL ret;
//     PPEB Peb = NtCurrentPeb();
//
//     if (dwFlsIndex >= FLS_MAXIMUM_AVAILABLE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlAcquirePebLock();
//
//     _SEH2_TRY
//     {
//         ret = RtlAreBitsSet(Peb->FlsBitmap, dwFlsIndex, 1);
//         if (ret)
//         {
//             PLIST_ENTRY Entry;
//             PFLS_CALLBACK_FUNCTION lpCallback;
//
//             RtlClearBits(Peb->FlsBitmap, dwFlsIndex, 1);
//             lpCallback = Peb->FlsCallback[dwFlsIndex];
//
//             for (Entry = Peb->FlsListHead.Flink; Entry != &Peb->FlsListHead; Entry = Entry->Flink)
//             {
//                 PRTL_FLS_DATA pFlsData;
//
//                 pFlsData = CONTAINING_RECORD(Entry, RTL_FLS_DATA, ListEntry);
//                 if (pFlsData->Data[dwFlsIndex])
//                 {
//                     if (lpCallback)
//                     {
//                         lpCallback(pFlsData->Data[dwFlsIndex]);
//                     }
//                     pFlsData->Data[dwFlsIndex] = NULL;
//                 }
//             }
//             Peb->FlsCallback[dwFlsIndex] = NULL;
//         }
//         else
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//         }
//     }
//     _SEH2_FINALLY
//     {
//         RtlReleasePebLock();
//     }
//     _SEH2_END;
//
//     return ret;
// }
//
//
// /*
//  * @implemented
//  */
// PVOID
// WINAPI
// FlsGetValue(DWORD dwFlsIndex)
// {
//     PRTL_FLS_DATA pFlsData;
//
//     pFlsData = NtCurrentTeb()->FlsData;
//     if (!dwFlsIndex || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE || !pFlsData)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     SetLastError(ERROR_SUCCESS);
//     return pFlsData->Data[dwFlsIndex];
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FlsSetValue(DWORD dwFlsIndex,
//             PVOID lpFlsData)
// {
//     PRTL_FLS_DATA pFlsData;
//
//     if (!dwFlsIndex || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pFlsData = NtCurrentTeb()->FlsData;
//
//     if (!NtCurrentTeb()->FlsData &&
//         !(NtCurrentTeb()->FlsData = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY,
//                                                     sizeof(RTL_FLS_DATA))))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     if (!pFlsData)
//     {
//         pFlsData = NtCurrentTeb()->FlsData;
//         RtlAcquirePebLock();
//         InsertTailList(&NtCurrentPeb()->FlsListHead, &pFlsData->ListEntry);
//         RtlReleasePebLock();
//     }
//     pFlsData->Data[dwFlsIndex] = lpFlsData;
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
