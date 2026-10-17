use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, NtClose, NtCreateEvent, NtSetEvent, NtWaitForMultipleObjects, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER, RtlAllocateHeap, RtlFreeHeap, RtlQueueWorkItem, WAITORTIMERCALLBACKFUNC, WORKERCALLBACKFUNC};
use crate::shared::*;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _RTLP_WAIT {
    pub Object: HANDLE,
    pub CallbackInProgress: BOOLEAN,
    pub CancelEvent: HANDLE,
    pub DeleteCount: LONG,
    pub CompletionEvent: HANDLE,
    pub Flags: ULONG,
    pub Callback: WAITORTIMERCALLBACKFUNC,
    pub Context: PVOID,
    pub Milliseconds: ULONG,
}
pub type RTLP_WAIT = _RTLP_WAIT;
pub type PRTLP_WAIT = *mut _RTLP_WAIT;

unsafe fn get_nt_timeout(pTime: PLARGE_INTEGER, timeout: ULONG) -> PLARGE_INTEGER {
    if timeout == INFINITE {
        return core::ptr::null_mut();
    }
    (*pTime).QuadPart = (timeout as u64).wrapping_mul((-10000i64) as u64) as i64;
    pTime
}
unsafe extern "system" fn Wait_thread_proc(Arg: LPVOID) {
    let Wait = Arg as PRTLP_WAIT;
    let mut Status: NTSTATUS;
    let alertable: BOOLEAN = (((*Wait).Flags & WT_EXECUTEINIOTHREAD) != 0) as u8;
    let mut handles: [HANDLE; 2] = [(*Wait).CancelEvent, (*Wait).Object];
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let completion_event: HANDLE;

    loop {
        Status = NtWaitForMultipleObjects(
            2,
            handles.as_mut_ptr(),
            WaitAny,
            alertable,
            get_nt_timeout(&mut timeout, (*Wait).Milliseconds),
        );

        if Status == STATUS_WAIT_1 || Status == STATUS_TIMEOUT {
            let TimerOrWaitFired: BOOLEAN;

            if Status == STATUS_WAIT_1 {
                TimerOrWaitFired = 0u8;
            } else {
                TimerOrWaitFired = 1u8;
            }
            (*Wait).CallbackInProgress = 1u8;
            ((*Wait).Callback.unwrap())((*Wait).Context, TimerOrWaitFired);
            (*Wait).CallbackInProgress = 0u8;

            if (*Wait).Flags & WT_EXECUTEONLYONCE != 0 {
                break;
            }
        } else if Status != STATUS_USER_APC {
            break;
        }
    }

    completion_event = (*Wait).CompletionEvent;
    if !completion_event.is_null() {
        NtSetEvent(completion_event, core::ptr::null_mut());
    }

    if InterlockedIncrement(core::ptr::addr_of_mut!((*Wait).DeleteCount)) == 2 {
        NtClose((*Wait).CancelEvent);
        RtlFreeHeap(RtlGetProcessHeap(), 0, Wait as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlRegisterWait(
    NewWaitObject: PHANDLE,
    Object: HANDLE,
    Callback: WAITORTIMERCALLBACKFUNC,
    Context: PVOID,
    Milliseconds: ULONG,
    Flags: ULONG,
) -> NTSTATUS {
    let Wait: PRTLP_WAIT = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<RTLP_WAIT>() as _,
    ) as PRTLP_WAIT;
    if Wait.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*Wait).Object = Object;
    (*Wait).Callback = Callback;
    (*Wait).Context = Context;
    (*Wait).Milliseconds = Milliseconds;
    (*Wait).Flags = Flags;
    (*Wait).CallbackInProgress = 0u8;
    (*Wait).DeleteCount = 0;
    (*Wait).CompletionEvent = core::ptr::null_mut();

    let mut Status = NtCreateEvent(
        core::ptr::addr_of_mut!((*Wait).CancelEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );

    if Status != STATUS_SUCCESS {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Wait as PVOID);
        return Status;
    }

    let Flags = Flags
        & (WT_EXECUTEINIOTHREAD
            | WT_EXECUTEINPERSISTENTTHREAD
            | WT_EXECUTELONGFUNCTION
            | WT_TRANSFER_IMPERSONATION);

    // WORKERCALLBACKFUNC is extern "stdcall" on i386 / extern "C" on amd64+arm64; our
    // extern "system" proc is ABI-identical per arch, so transmute satisfies the type checker.
    let cb: WORKERCALLBACKFUNC =
        core::mem::transmute(Some(Wait_thread_proc as unsafe extern "system" fn(LPVOID)));
    Status = RtlQueueWorkItem(cb, Wait as PVOID, Flags);

    if Status != STATUS_SUCCESS {
        NtClose((*Wait).CancelEvent);
        RtlFreeHeap(RtlGetProcessHeap(), 0, Wait as PVOID);
        return Status;
    }

    *NewWaitObject = Wait as HANDLE;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeregisterWaitEx(
    WaitHandle: HANDLE,
    mut CompletionEvent: HANDLE,
) -> NTSTATUS {
    let Wait: PRTLP_WAIT = WaitHandle as PRTLP_WAIT;
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    NtSetEvent((*Wait).CancelEvent, core::ptr::null_mut());
    if (*Wait).CallbackInProgress != 0 {
        if CompletionEvent != core::ptr::null_mut() {
            if CompletionEvent == INVALID_HANDLE_VALUE as HANDLE {
                Status = NtCreateEvent(
                    &mut CompletionEvent,
                    EVENT_ALL_ACCESS,
                    core::ptr::null_mut(),
                    _EVENT_TYPE_NotificationEvent,
                    0u8,
                );

                if Status != STATUS_SUCCESS as NTSTATUS {
                    return Status;
                }

                InterlockedExchangePointer(
                    core::ptr::addr_of_mut!((*Wait).CompletionEvent),
                    CompletionEvent,
                );

                if (*Wait).CallbackInProgress != 0 {
                    NtWaitForSingleObject(CompletionEvent, 0u8, core::ptr::null_mut());
                }

                NtClose(CompletionEvent);
            } else {
                InterlockedExchangePointer(
                    core::ptr::addr_of_mut!((*Wait).CompletionEvent),
                    CompletionEvent,
                );

                if (*Wait).CallbackInProgress != 0 {
                    Status = STATUS_PENDING;
                }
            }
        } else {
            Status = STATUS_PENDING;
        }
    }

    if InterlockedIncrement(core::ptr::addr_of_mut!((*Wait).DeleteCount)) == 2 {
        Status = STATUS_SUCCESS as NTSTATUS;
        NtClose((*Wait).CancelEvent);
        RtlFreeHeap(RtlGetProcessHeap(), 0, Wait as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeregisterWait(WaitHandle: HANDLE) -> NTSTATUS {
    RtlDeregisterWaitEx(WaitHandle, core::ptr::null_mut())
}
