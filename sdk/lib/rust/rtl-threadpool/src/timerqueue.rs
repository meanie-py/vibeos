use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, LIST_ENTRY, RTL_CRITICAL_SECTION, PTHREAD_START_ROUTINE, NtClose, NtCreateEvent, NtQueryPerformanceCounter, NtResumeThread, NtSetEvent, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER,  RtlAllocateHeap, RtlDeleteCriticalSection, RtlEnterCriticalSection, RtlFreeHeap, RtlInitializeCriticalSection, RtlLeaveCriticalSection, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;
use crate::types::*;
use crate::wine_list::*;
// RtlpStartThreadFunc / RtlpExitThreadFunc (globals) + RtlQueueWorkItem
// are provided by the workitem module (same crate, same C cohab set).
use crate::workitem::{RtlpStartThreadFunc, RtlpExitThreadFunc, RtlQueueWorkItem};

// ---- file-scope preamble (translated) ----
pub static mut TimerThreadHandle: HANDLE = core::ptr::null_mut();

#[repr(C)]
pub struct queue_timer {
    pub q: *mut timer_queue,
    pub entry: list,
    pub runcount: ULONG,
    pub callback: WAITORTIMERCALLBACKFUNC,
    pub param: PVOID,
    pub period: DWORD,
    pub flags: ULONG,
    pub expire: ULONGLONG,
    pub destroy: BOOL,
    pub event: HANDLE,
}

#[repr(C)]
pub struct timer_queue {
    pub magic: DWORD,
    pub cs: RTL_CRITICAL_SECTION,
    pub timers: list,
    pub quit: BOOL,
    pub event: HANDLE,
    pub thread: HANDLE,
}

pub const EXPIRE_NEVER: ULONGLONG = !(0 as ULONGLONG);

pub const TIMER_QUEUE_MAGIC: DWORD = 0x516d6954;
#[no_mangle]
pub unsafe extern "C" fn RtlpInitializeTimerThread() -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe fn get_nt_timeout(pTime: PLARGE_INTEGER, timeout: ULONG) -> PLARGE_INTEGER {
    if timeout == INFINITE {
        return core::ptr::null_mut();
    }
    (*pTime).QuadPart = (timeout as ULONGLONG).wrapping_mul((-10000i64) as ULONGLONG) as i64;
    pTime
}

unsafe fn queue_remove_timer(t: *mut queue_timer) {
    let q: *mut timer_queue = (*t).q;

    list_remove(core::ptr::addr_of_mut!((*t).entry));
    if !(*t).event.is_null() {
        NtSetEvent((*t).event, core::ptr::null_mut());
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, t as PVOID);

    if (*q).quit != 0 && list_empty(core::ptr::addr_of_mut!((*q).timers)) != 0 {
        NtSetEvent((*q).event, core::ptr::null_mut());
    }
}

unsafe fn timer_cleanup_callback(t: *mut queue_timer) {
    let q: *mut timer_queue = (*t).q;
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    (*t).runcount -= 1;

    if (*t).destroy != 0 && (*t).runcount == 0 {
        queue_remove_timer(t);
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));
}
unsafe extern "system" fn timer_callback_wrapper(p: LPVOID) {
    let t: *mut queue_timer = p as *mut queue_timer;
    ((*t).callback.unwrap())((*t).param, 1);
    timer_cleanup_callback(t);
}

unsafe fn queue_current_time() -> ULONGLONG {
    let mut now: LARGE_INTEGER = core::mem::zeroed();
    let mut freq: LARGE_INTEGER = core::mem::zeroed();
    NtQueryPerformanceCounter(&mut now, &mut freq);
    (now.QuadPart * 1000 / freq.QuadPart) as ULONGLONG
}

unsafe fn queue_add_timer(t: *mut queue_timer, time: ULONGLONG, set_event: BOOL) {
    let q: *mut timer_queue = (*t).q;
    let mut ptr: *mut list = core::ptr::addr_of_mut!((*q).timers);

    if time != EXPIRE_NEVER {
        ptr = (*core::ptr::addr_of_mut!((*q).timers)).next;
        while ptr != core::ptr::addr_of_mut!((*q).timers) {
            let cur: *mut queue_timer = (ptr as usize - core::mem::offset_of!(queue_timer, entry)) as *mut queue_timer;
            if time < (*cur).expire {
                break;
            }
            ptr = (*ptr).next;
        }
    }

    list_add_before(ptr, core::ptr::addr_of_mut!((*t).entry));

    (*t).expire = time;

    if set_event != 0 && core::ptr::addr_of_mut!((*t).entry) == (*core::ptr::addr_of_mut!((*q).timers)).next {
        NtSetEvent((*q).event, core::ptr::null_mut());
    }
}

unsafe fn queue_move_timer(t: *mut queue_timer, time: ULONGLONG, set_event: BOOL) {
    list_remove(core::ptr::addr_of_mut!((*t).entry));
    queue_add_timer(t, time, set_event);
}

unsafe fn queue_timer_expire(q: *mut timer_queue) {
    let mut t: *mut queue_timer = core::ptr::null_mut();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    if !list_head(core::ptr::addr_of_mut!((*q).timers)).is_null() {
        let now: ULONGLONG;
        let next: ULONGLONG;
        t = LIST_ENTRY!(
            list_head(core::ptr::addr_of_mut!((*q).timers)),
            queue_timer,
            entry
        );
        now = queue_current_time();
        if (*t).destroy == 0 && (*t).expire <= now {
            (*t).runcount += 1;
            if (*t).period != 0 {
                next = (*t).expire + (*t).period as ULONGLONG;
                let next = if next < now {
                    now + (*t).period as ULONGLONG
                } else {
                    next
                };
                queue_move_timer(t, next, 0);
            } else {
                next = EXPIRE_NEVER;
                queue_move_timer(t, next, 0);
            }
        } else {
            t = core::ptr::null_mut();
        }
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    if !t.is_null() {
        if (*t).flags & WT_EXECUTEINTIMERTHREAD != 0 {
            timer_callback_wrapper(t as LPVOID);
        } else {
            let flags = (*t).flags
                & (WT_EXECUTEINIOTHREAD
                    | WT_EXECUTEINPERSISTENTTHREAD
                    | WT_EXECUTELONGFUNCTION
                    | WT_TRANSFER_IMPERSONATION);
            let status = RtlQueueWorkItem(Some(timer_callback_wrapper), t as PVOID, flags);
            if status != STATUS_SUCCESS {
                timer_cleanup_callback(t);
            }
        }
    }
}

unsafe fn queue_get_timeout(q: *mut timer_queue) -> ULONG {
    let t: *mut queue_timer;
    let mut timeout: ULONG = INFINITE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    let head = list_head(core::ptr::addr_of_mut!((*q).timers));
    if !head.is_null() {
        t = LIST_ENTRY!(head, queue_timer, entry);
        if (*t).expire != EXPIRE_NEVER {
            let time = queue_current_time();
            timeout = if (*t).expire < time { 0 } else { ((*t).expire - time) as ULONG };
        }
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    timeout
}
unsafe extern "system" fn timer_queue_thread_proc(p: LPVOID) -> DWORD {
    let q: *mut timer_queue = p as *mut timer_queue;
    let mut timeout_ms: ULONG;

    timeout_ms = INFINITE;
    loop {
        let mut timeout: LARGE_INTEGER = core::mem::zeroed();
        let status: NTSTATUS;
        let mut done: BOOL = 0;

        status = NtWaitForSingleObject(
            (*q).event,
            0u8,
            get_nt_timeout(core::ptr::addr_of_mut!(timeout), timeout_ms),
        );

        if status == STATUS_WAIT_0 {
            RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
            if (*q).quit != 0 && list_empty(core::ptr::addr_of_mut!((*q).timers)) != 0 {
                done = 1;
            }
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));
        } else if status == STATUS_TIMEOUT {
            queue_timer_expire(q);
        }

        if done != 0 {
            break;
        }

        timeout_ms = queue_get_timeout(q);
    }

    NtClose((*q).event);
    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    (*q).magic = 0;
    RtlFreeHeap(RtlGetProcessHeap(), 0, q as PVOID);
    (RtlpExitThreadFunc.unwrap())(STATUS_SUCCESS);
    0
}

unsafe fn queue_destroy_timer(t: *mut queue_timer) {
    (*t).destroy = 1;
    if (*t).runcount == 0 {
        queue_remove_timer(t);
    } else {
        queue_move_timer(t, !(0 as ULONGLONG), 0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateTimerQueue(NewTimerQueue: PHANDLE) -> NTSTATUS {
    let mut status: NTSTATUS;
    let q = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<timer_queue>() as _,
    ) as *mut timer_queue;
    if q.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    list_init(core::ptr::addr_of_mut!((*q).timers));
    (*q).quit = FALSE as BOOL;
    (*q).magic = TIMER_QUEUE_MAGIC;
    status = NtCreateEvent(
        core::ptr::addr_of_mut!((*q).event),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        FALSE as BOOLEAN,
    );
    if status != STATUS_SUCCESS {
        RtlFreeHeap(RtlGetProcessHeap(), 0, q as PVOID);
        return status;
    }
    status = (RtlpStartThreadFunc.unwrap())(
        core::mem::transmute::<_, PTHREAD_START_ROUTINE>(timer_queue_thread_proc as usize),
        q as PVOID,
        core::ptr::addr_of_mut!((*q).thread),
    );
    if status != STATUS_SUCCESS {
        NtClose((*q).event);
        RtlFreeHeap(RtlGetProcessHeap(), 0, q as PVOID);
        return status;
    }

    NtResumeThread((*q).thread, core::ptr::null_mut());
    *NewTimerQueue = q as HANDLE;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteTimerQueueEx(
    TimerQueue: HANDLE,
    CompletionEvent: HANDLE,
) -> NTSTATUS {
    let q = TimerQueue as *mut timer_queue;
    let thread: HANDLE;
    let status: NTSTATUS;

    if q.is_null() || (*q).magic != TIMER_QUEUE_MAGIC {
        return STATUS_INVALID_HANDLE;
    }

    thread = (*q).thread;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    (*q).quit = 1;
    if !list_head(core::ptr::addr_of_mut!((*q).timers)).is_null() {
        let mut cursor = (*q).timers.next;
        let head = core::ptr::addr_of_mut!((*q).timers);
        while cursor != head {
            let t = (cursor as *mut u8)
                .offset(-(core::mem::offset_of!(queue_timer, entry) as isize))
                as *mut queue_timer;
            let next = (*cursor).next;
            queue_destroy_timer(t);
            cursor = next;
        }
    } else {
        NtSetEvent((*q).event, core::ptr::null_mut());
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    if CompletionEvent == INVALID_HANDLE_VALUE as HANDLE {
        NtWaitForSingleObject(thread, 0u8, core::ptr::null_mut());
        status = STATUS_SUCCESS;
    } else {
        if !CompletionEvent.is_null() {
            NtWaitForSingleObject(thread, 0u8, core::ptr::null_mut());
            NtSetEvent(CompletionEvent, core::ptr::null_mut());
        }
        status = STATUS_PENDING;
    }

    NtClose(thread);
    status
}

unsafe fn get_timer_queue(TimerQueue: HANDLE) -> *mut timer_queue {
    static mut default_timer_queue: *mut timer_queue = core::ptr::null_mut();

    if !TimerQueue.is_null() {
        return TimerQueue as *mut timer_queue;
    } else {
        if default_timer_queue.is_null() {
            let mut q: HANDLE = core::ptr::null_mut();
            let status: NTSTATUS = RtlCreateTimerQueue(core::ptr::addr_of_mut!(q));
            if status == STATUS_SUCCESS {
                let p = InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!(default_timer_queue) as *mut PVOID,
                    q,
                    core::ptr::null_mut(),
                );
                if !p.is_null() {
                    RtlDeleteTimerQueueEx(q, core::ptr::null_mut());
                }
            }
        }
        return default_timer_queue;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateTimer(
    TimerQueue: HANDLE,
    NewTimer: PHANDLE,
    Callback: WAITORTIMERCALLBACKFUNC,
    Parameter: PVOID,
    DueTime: DWORD,
    Period: DWORD,
    Flags: ULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let q: *mut timer_queue = get_timer_queue(TimerQueue);

    if q.is_null() {
        return STATUS_NO_MEMORY;
    }
    if (*q).magic != TIMER_QUEUE_MAGIC {
        return STATUS_INVALID_HANDLE;
    }

    let t: *mut queue_timer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<queue_timer>() as _,
    ) as *mut queue_timer;
    if t.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*t).q = q;
    (*t).runcount = 0;
    (*t).callback = Callback;
    (*t).param = Parameter;
    (*t).period = Period;
    (*t).flags = Flags;
    (*t).destroy = 0;
    (*t).event = core::ptr::null_mut();

    status = STATUS_SUCCESS;
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    if (*q).quit != 0 {
        status = STATUS_INVALID_HANDLE;
    } else {
        queue_add_timer(t, queue_current_time() + DueTime as ULONGLONG, 1);
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    if status == STATUS_SUCCESS {
        *NewTimer = t as HANDLE;
    } else {
        RtlFreeHeap(RtlGetProcessHeap(), 0, t as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetTimer(
    TimerQueue: HANDLE,
    NewTimer: PHANDLE,
    Callback: WAITORTIMERCALLBACKFUNC,
    Parameter: PVOID,
    DueTime: DWORD,
    Period: DWORD,
    Flags: ULONG,
) -> NTSTATUS {
    RtlCreateTimer(
        TimerQueue,
        NewTimer,
        Callback,
        Parameter,
        DueTime,
        Period,
        Flags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlUpdateTimer(
    TimerQueue: HANDLE,
    Timer: HANDLE,
    DueTime: DWORD,
    Period: DWORD,
) -> NTSTATUS {
    let t = Timer as *mut queue_timer;
    let q = (*t).q;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    if (*t).expire != !(0 as ULONGLONG) {
        (*t).period = Period;
        queue_move_timer(t, queue_current_time() + DueTime as ULONGLONG, 1);
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteTimer(
    TimerQueue: HANDLE,
    Timer: HANDLE,
    CompletionEvent: HANDLE,
) -> NTSTATUS {
    let t: *mut queue_timer = Timer as *mut queue_timer;
    let q: *mut timer_queue;
    let mut status: NTSTATUS = STATUS_PENDING;
    let mut event: HANDLE = core::ptr::null_mut();

    if Timer.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }
    q = (*t).q;
    if CompletionEvent == INVALID_HANDLE_VALUE as HANDLE {
        status = NtCreateEvent(
            core::ptr::addr_of_mut!(event),
            EVENT_ALL_ACCESS,
            core::ptr::null_mut(),
            _EVENT_TYPE_SynchronizationEvent,
            0u8,
        );
        if status == STATUS_SUCCESS {
            status = STATUS_PENDING;
        }
    } else if !CompletionEvent.is_null() {
        event = CompletionEvent;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    (*t).event = event;
    if (*t).runcount == 0 && !event.is_null() {
        status = STATUS_SUCCESS;
    }
    queue_destroy_timer(t);
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    if CompletionEvent == INVALID_HANDLE_VALUE as HANDLE && !event.is_null() {
        if status == STATUS_PENDING {
            NtWaitForSingleObject(event, 0u8, core::ptr::null_mut());
            status = STATUS_SUCCESS;
        }
        NtClose(event);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlCancelTimer(
    TimerQueue: HANDLE,
    Timer: HANDLE,
) -> NTSTATUS {
    RtlDeleteTimer(TimerQueue, Timer, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteTimerQueue(TimerQueue: HANDLE) -> NTSTATUS {
    RtlDeleteTimerQueueEx(TimerQueue, INVALID_HANDLE_VALUE as HANDLE)
}
