use crate::shim::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{EXCEPTION_POINTERS, LIST_ENTRY, PCONTEXT, PEXCEPTION_RECORD, PVECTORED_EXCEPTION_HANDLER, RTL_CRITICAL_SECTION, RtlAllocateHeap, RtlEnterCriticalSection, RtlFreeHeap, RtlInitializeCriticalSection, RtlLeaveCriticalSection};
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
pub static mut RtlpVectoredHandlerLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut RtlpVectoredExceptionList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut RtlpVectoredContinueList: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[repr(C)]
pub struct _RTL_VECTORED_HANDLER_ENTRY {
    pub ListEntry: LIST_ENTRY,
    pub VectoredHandler: PVECTORED_EXCEPTION_HANDLER,
    pub Refs: ULONG,
}
pub type RTL_VECTORED_HANDLER_ENTRY = _RTL_VECTORED_HANDLER_ENTRY;
pub type PRTL_VECTORED_HANDLER_ENTRY = *mut _RTL_VECTORED_HANDLER_ENTRY;
#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeVectoredExceptionHandling() {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));
    InitializeListHead(core::ptr::addr_of_mut!(RtlpVectoredExceptionList));
    InitializeListHead(core::ptr::addr_of_mut!(RtlpVectoredContinueList));
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCallVectoredHandlers(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
    VectoredHandlerList: PLIST_ENTRY,
) -> BOOLEAN {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut VectoredExceptionHandler: PRTL_VECTORED_HANDLER_ENTRY =
        core::ptr::null_mut();
    let mut VectoredHandler: PVECTORED_EXCEPTION_HANDLER;
    let mut ExceptionInfo: EXCEPTION_POINTERS = core::mem::zeroed();
    let mut HandlerRemoved: BOOLEAN;
    let mut HandlerReturn: LONG;

    HandlerRemoved = 0u8;
    HandlerReturn = EXCEPTION_CONTINUE_SEARCH;

    ExceptionInfo.ExceptionRecord = ExceptionRecord;
    ExceptionInfo.ContextRecord = Context;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    CurrentEntry = (*VectoredHandlerList).Flink;
    while CurrentEntry != VectoredHandlerList {
        VectoredExceptionHandler = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_VECTORED_HANDLER_ENTRY, ListEntry) as isize))
            as PRTL_VECTORED_HANDLER_ENTRY;

        (*VectoredExceptionHandler).Refs += 1;

        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

        let decoded = RtlDecodePointer(core::mem::transmute::<_, PVOID>(
            (*VectoredExceptionHandler).VectoredHandler,
        ));
        VectoredHandler = core::mem::transmute::<PVOID, PVECTORED_EXCEPTION_HANDLER>(decoded);

        HandlerReturn = (VectoredHandler.unwrap())(&mut ExceptionInfo);

        RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

        (*VectoredExceptionHandler).Refs -= 1;
        if (*VectoredExceptionHandler).Refs == 0 {
            if HandlerReturn == EXCEPTION_CONTINUE_EXECUTION {
                RemoveEntryList(core::ptr::addr_of_mut!((*VectoredExceptionHandler).ListEntry));
                HandlerRemoved = 1u8;
                break;
            }

            CurrentEntry = (*VectoredExceptionHandler).ListEntry.Flink;
            RemoveEntryList(core::ptr::addr_of_mut!((*VectoredExceptionHandler).ListEntry));

            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));
            RtlFreeHeap(RtlGetProcessHeap(), 0, VectoredExceptionHandler as PVOID);
            RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));
        } else {
            if HandlerReturn == EXCEPTION_CONTINUE_EXECUTION {
                break;
            } else {
                CurrentEntry = (*CurrentEntry).Flink;
            }
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    if HandlerRemoved != 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, VectoredExceptionHandler as PVOID);
    }

    if HandlerReturn == EXCEPTION_CONTINUE_EXECUTION {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpAddVectoredHandler(
    FirstHandler: ULONG,
    VectoredHandler: PVECTORED_EXCEPTION_HANDLER,
    VectoredHandlerList: PLIST_ENTRY,
) -> PVOID {

    let VectoredHandlerEntry = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<RTL_VECTORED_HANDLER_ENTRY>() as _,
    ) as PRTL_VECTORED_HANDLER_ENTRY;
    if VectoredHandlerEntry.is_null() {
        return core::ptr::null_mut();
    }

    (*VectoredHandlerEntry).VectoredHandler = core::mem::transmute(RtlEncodePointer(
        core::mem::transmute::<_, PVOID>(VectoredHandler),
    ));
    (*VectoredHandlerEntry).Refs = 1;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    if FirstHandler != 0 {
        InsertHeadList(
            VectoredHandlerList,
            core::ptr::addr_of_mut!((*VectoredHandlerEntry).ListEntry),
        );
    } else {
        InsertTailList(
            VectoredHandlerList,
            core::ptr::addr_of_mut!((*VectoredHandlerEntry).ListEntry),
        );
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    VectoredHandlerEntry as PVOID
}
#[no_mangle]
pub unsafe extern "system" fn RtlpRemoveVectoredHandler(VectoredHandlerHandle: PVOID,
                                                     VectoredHandlerList: PLIST_ENTRY) -> ULONG {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut VectoredExceptionHandler: *mut _RTL_VECTORED_HANDLER_ENTRY = core::ptr::null_mut();
    let mut HandlerRemoved: u8 = 0u8;
    let mut HandlerFound: u8 = 0u8;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    CurrentEntry = (*VectoredHandlerList).Flink;
    while CurrentEntry != VectoredHandlerList {
        VectoredExceptionHandler = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_VECTORED_HANDLER_ENTRY, ListEntry) as isize))
            as *mut _RTL_VECTORED_HANDLER_ENTRY;

        if VectoredExceptionHandler as PVOID == VectoredHandlerHandle {
            (*VectoredExceptionHandler).Refs -= 1;
            if (*VectoredExceptionHandler).Refs == 0 {
                RemoveEntryList(core::ptr::addr_of_mut!((*VectoredExceptionHandler).ListEntry));
                HandlerRemoved = 1u8;
            }

            HandlerFound = 1u8;
            break;
        } else {
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpVectoredHandlerLock));

    if HandlerRemoved != 0 {
        RtlFreeHeap(RtlGetProcessHeap(),
                    0,
                    VectoredExceptionHandler as PVOID);
    }

    HandlerFound as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlCallVectoredExceptionHandlers(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
) -> BOOLEAN {
    RtlpCallVectoredHandlers(
        ExceptionRecord,
        Context,
        core::ptr::addr_of_mut!(RtlpVectoredExceptionList),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlCallVectoredContinueHandlers(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
) {
    RtlpCallVectoredHandlers(
        ExceptionRecord,
        Context,
        core::ptr::addr_of_mut!(RtlpVectoredContinueList),
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddVectoredExceptionHandler(
    FirstHandler: ULONG,
    VectoredHandler: PVECTORED_EXCEPTION_HANDLER,
) -> PVOID {
    RtlpAddVectoredHandler(
        FirstHandler,
        VectoredHandler,
        core::ptr::addr_of_mut!(RtlpVectoredExceptionList),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddVectoredContinueHandler(
    FirstHandler: ULONG,
    VectoredHandler: PVECTORED_EXCEPTION_HANDLER,
) -> PVOID {
    RtlpAddVectoredHandler(
        FirstHandler,
        VectoredHandler,
        core::ptr::addr_of_mut!(RtlpVectoredContinueList),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlRemoveVectoredExceptionHandler(VectoredHandlerHandle: PVOID) -> ULONG {
    RtlpRemoveVectoredHandler(VectoredHandlerHandle,
                              core::ptr::addr_of_mut!(RtlpVectoredExceptionList))
}

#[no_mangle]
pub unsafe extern "system" fn RtlRemoveVectoredContinueHandler(VectoredHandlerHandle: PVOID) -> ULONG {
    RtlpRemoveVectoredHandler(VectoredHandlerHandle,
                              core::ptr::addr_of_mut!(RtlpVectoredContinueList))
}
