use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, NtClose, NtCreateEvent, NtSetEvent, NtWaitForMultipleObjects, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER, RtlAllocateHeap, RtlFreeHeap, RtlQueueWorkItem, WAITORTIMERCALLBACKFUNC, WORKERCALLBACKFUNC};
use crate::shared::*;

// ---- file-scope preamble (translated) ----
// DIVERGES from wait.c: the C keeps a BOOLEAN CallbackInProgress and records the caller's
// CompletionEvent only if a callback is running at that instant, so a wait deregistered while
// idle -- or after its WT_EXECUTEONLYONCE callback already returned -- never signals the event.
// Windows always signals it (waitprobe.win11.txt, cases A/B). base::win::ObjectWatcher::
// StopWatching passes its own event and then waits on it, so that hung Chrome's browser UI
// thread for good. The C also races: the wait thread can read CompletionEvent as NULL just
// before the deregister stores it, and can start a callback after a blocking deregister
// returned. Here `State` arbitrates: the wait thread starts a callback only while CANCELLED is
// clear, and whichever side bumps DeleteCount to 2 last signals CompletionEvent and frees.
const WAIT_CANCELLED: i32 = 1;
const WAIT_IN_CALLBACK: i32 = 2;

#[repr(C)]
pub struct _RTLP_WAIT {
    pub Object: HANDLE,
    pub State: LONG,
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

unsafe fn state(Wait: PRTLP_WAIT) -> &'static core::sync::atomic::AtomicI32 {
    &*(core::ptr::addr_of_mut!((*Wait).State) as *const core::sync::atomic::AtomicI32)
}

/// Called by each of the two owners (wait thread, deregister) when done with `Wait`; the second
/// signals the completion event the deregister left (if any) and frees. Returns true if it freed.
unsafe fn release(Wait: PRTLP_WAIT) -> bool {
    if InterlockedIncrement(core::ptr::addr_of_mut!((*Wait).DeleteCount)) != 2 {
        return false;
    }
    let completion_event = (*Wait).CompletionEvent;
    if !completion_event.is_null() {
        NtSetEvent(completion_event, core::ptr::null_mut());
    }
    NtClose((*Wait).CancelEvent);
    RtlFreeHeap(RtlGetProcessHeap(), 0, Wait as PVOID);
    true
}

unsafe fn get_nt_timeout(pTime: PLARGE_INTEGER, timeout: ULONG) -> PLARGE_INTEGER {
    if timeout == INFINITE {
        return core::ptr::null_mut();
    }
    (*pTime).QuadPart = (timeout as u64).wrapping_mul((-10000i64) as u64) as i64;
    pTime
}
unsafe extern "system" fn Wait_thread_proc(Arg: LPVOID) {
    use core::sync::atomic::Ordering::SeqCst;
    let Wait = Arg as PRTLP_WAIT;
    let mut Status: NTSTATUS;
    let alertable: BOOLEAN = (((*Wait).Flags & WT_EXECUTEINIOTHREAD) != 0) as u8;
    let mut handles: [HANDLE; 2] = [(*Wait).CancelEvent, (*Wait).Object];
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();

    'wait: loop {
        Status = NtWaitForMultipleObjects(
            2,
            handles.as_mut_ptr(),
            WaitAny,
            alertable,
            get_nt_timeout(&mut timeout, (*Wait).Milliseconds),
        );

        if Status == STATUS_WAIT_1 || Status == STATUS_TIMEOUT {
            let TimerOrWaitFired: BOOLEAN = (Status != STATUS_WAIT_1) as u8;

            // Enter the callback only if not deregistered meanwhile.
            let mut s = state(Wait).load(SeqCst);
            loop {
                if s & WAIT_CANCELLED != 0 {
                    break 'wait;
                }
                match state(Wait).compare_exchange(s, s | WAIT_IN_CALLBACK, SeqCst, SeqCst) {
                    Ok(_) => break,
                    Err(now) => s = now,
                }
            }
            ((*Wait).Callback.unwrap())((*Wait).Context, TimerOrWaitFired);
            let s = state(Wait).fetch_and(!WAIT_IN_CALLBACK, SeqCst);

            if (*Wait).Flags & WT_EXECUTEONLYONCE != 0 || s & WAIT_CANCELLED != 0 {
                break;
            }
        } else if Status != STATUS_USER_APC {
            break;
        }
    }

    release(Wait);
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
    (*Wait).State = 0;
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
    CompletionEvent: HANDLE,
) -> NTSTATUS {
    let Wait: PRTLP_WAIT = WaitHandle as PRTLP_WAIT;

    // From here on the wait thread starts no new callback; `in_callback` says whether one it
    // already started may still be running.
    let in_callback = state(Wait).fetch_or(WAIT_CANCELLED, core::sync::atomic::Ordering::SeqCst)
        & WAIT_IN_CALLBACK
        != 0;
    NtSetEvent((*Wait).CancelEvent, core::ptr::null_mut());

    if CompletionEvent == INVALID_HANDLE_VALUE as HANDLE {
        // Block until the running callback (if any) returns.
        if !in_callback {
            release(Wait);
            return STATUS_SUCCESS as NTSTATUS;
        }
        let mut done: HANDLE = core::ptr::null_mut();
        let Status = NtCreateEvent(
            &mut done,
            EVENT_ALL_ACCESS,
            core::ptr::null_mut(),
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
        if Status != STATUS_SUCCESS as NTSTATUS {
            // Cannot block: leave the wait to the thread, as a NULL event would.
            release(Wait);
            return Status;
        }
        (*Wait).CompletionEvent = done;
        if !release(Wait) {
            NtWaitForSingleObject(done, 0u8, core::ptr::null_mut());
        }
        NtClose(done);
        return STATUS_SUCCESS as NTSTATUS;
    }

    // NULL or a caller's event: return at once; the event is signaled by whichever side
    // finishes last (now, if the wait thread is already done).
    (*Wait).CompletionEvent = CompletionEvent;
    if release(Wait) || !in_callback {
        STATUS_SUCCESS as NTSTATUS
    } else {
        STATUS_PENDING
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeregisterWait(WaitHandle: HANDLE) -> NTSTATUS {
    RtlDeregisterWaitEx(WaitHandle, core::ptr::null_mut())
}
