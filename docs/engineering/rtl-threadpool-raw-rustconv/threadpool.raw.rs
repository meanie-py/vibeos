use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{DWORD_PTR, FILE_COMPLETION_INFORMATION, FileCompletionInformation, IMAGE_NT_HEADERS, _IO_STATUS_BLOCK, LARGE_INTEGER, LIST_ENTRY, NtClose, NtCreateEvent, NtCreateIoCompletion, NtCurrentTeb, NtQueryPerformanceCounter, NtQuerySystemTime, NtReleaseMutant, NtReleaseSemaphore, NtRemoveIoCompletion, NtSetEvent, NtSetInformationFile, NtSetInformationThread, NtSetIoCompletion, NtWaitForMultipleObjects, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER, PTHREAD_START_ROUTINE, RTL_CONDITION_VARIABLE, RTL_CRITICAL_SECTION, RTL_CRITICAL_SECTION_DEBUG, RtlAllocateHeap, RtlCreateUserThread, RtlDeleteCriticalSection, RtlEnterCriticalSection, RtlExitUserThread, RtlFreeHeap, RtlImageNtHeader, RtlInitUnicodeString, RtlInitializeConditionVariable, RtlInitializeCriticalSection, RtlLeaveCriticalSection, RtlNtStatusToDosError, RtlSleepConditionVariableCS, RtlWakeAllConditionVariable, RtlWakeConditionVariable, _EVENT_TYPE_SynchronizationEvent, THREAD_NAME_INFORMATION, ThreadNameInformation};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const EXPIRE_NEVER: u64 = !0u64;

pub const TIMER_QUEUE_MAGIC: u32 = 0x516d6954;

pub const THREADPOOL_WORKER_TIMEOUT: u32 = 5000;

pub const TP_OBJECT_TYPE_SIMPLE: u32 = 0;
pub const TP_OBJECT_TYPE_WORK: u32 = 1;
pub const TP_OBJECT_TYPE_TIMER: u32 = 2;
pub const TP_OBJECT_TYPE_WAIT: u32 = 3;
pub const TP_OBJECT_TYPE_IO: u32 = 4;

#[repr(C)]
pub struct rtl_work_item {
    pub function: PRTL_WORK_ITEM_ROUTINE,
    pub context: PVOID,
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

#[repr(C)]
pub struct queue_timer {
    pub q: *mut timer_queue,
    pub entry: list,
    pub runcount: ULONG,
    pub callback: RTL_WAITORTIMERCALLBACKFUNC,
    pub param: PVOID,
    pub period: DWORD,
    pub flags: ULONG,
    pub expire: ULONGLONG,
    pub destroy: BOOL,
    pub event: HANDLE,
}

#[repr(C)]
pub struct io_completion {
    pub iosb: _IO_STATUS_BLOCK,
    pub cvalue: ULONG_PTR,
}

#[repr(C)]
pub struct waitqueue_bucket {
    pub bucket_entry: list,
    pub objcount: LONG,
    pub reserved: list,
    pub waiting: list,
    pub update_event: HANDLE,
    pub alertable: BOOL,
}

#[repr(C)]
pub struct threadpool {
    pub refcount: LONG,
    pub objcount: LONG,
    pub shutdown: BOOL,
    pub cs: RTL_CRITICAL_SECTION,
    pub pools: [list; 3],
    pub update_event: RTL_CONDITION_VARIABLE,
    pub max_workers: ::core::ffi::c_int,
    pub min_workers: ::core::ffi::c_int,
    pub num_workers: ::core::ffi::c_int,
    pub num_busy_workers: ::core::ffi::c_int,
    pub compl_port: HANDLE,
    pub stack_info: TP_POOL_STACK_INFORMATION,
}

#[repr(C)]
pub struct threadpool_object_u_simple {
    pub callback: PTP_SIMPLE_CALLBACK,
}

#[repr(C)]
pub struct threadpool_object_u_work {
    pub callback: PTP_WORK_CALLBACK,
}

#[repr(C)]
pub struct threadpool_object_u_timer {
    pub callback: PTP_TIMER_CALLBACK,
    pub timer_initialized: BOOL,
    pub timer_pending: BOOL,
    pub timer_entry: list,
    pub timer_set: BOOL,
    pub timeout: ULONGLONG,
    pub period: LONG,
    pub window_length: LONG,
}

#[repr(C)]
pub struct threadpool_object_u_wait {
    pub callback: PTP_WAIT_CALLBACK,
    pub signaled: LONG,
    pub bucket: *mut waitqueue_bucket,
    pub wait_pending: BOOL,
    pub wait_entry: list,
    pub timeout: ULONGLONG,
    pub handle: HANDLE,
    pub flags: DWORD,
    pub rtl_callback: RTL_WAITORTIMERCALLBACKFUNC,
}

#[repr(C)]
pub struct threadpool_object_u_io {
    pub callback: PTP_IO_CALLBACK,
    pub pending_count: ::core::ffi::c_uint,
    pub skipped_count: ::core::ffi::c_uint,
    pub completion_count: ::core::ffi::c_uint,
    pub completion_max: ::core::ffi::c_uint,
    pub shutting_down: BOOL,
    pub completions: *mut io_completion,
}

#[repr(C)]
pub union threadpool_object_u {
    pub simple: ::core::mem::ManuallyDrop<threadpool_object_u_simple>,
    pub work: ::core::mem::ManuallyDrop<threadpool_object_u_work>,
    pub timer: ::core::mem::ManuallyDrop<threadpool_object_u_timer>,
    pub wait: ::core::mem::ManuallyDrop<threadpool_object_u_wait>,
    pub io: ::core::mem::ManuallyDrop<threadpool_object_u_io>,
}

#[repr(C)]
pub struct threadpool_object {
    pub win32_callback: *mut ::core::ffi::c_void,
    pub refcount: LONG,
    pub shutdown: BOOL,
    pub type_: u32,
    pub pool: *mut threadpool,
    pub group: *mut threadpool_group,
    pub userdata: PVOID,
    pub group_cancel_callback: PTP_CLEANUP_GROUP_CANCEL_CALLBACK,
    pub finalization_callback: PTP_SIMPLE_CALLBACK,
    pub may_run_long: BOOL,
    pub race_dll: HMODULE,
    pub priority: TP_CALLBACK_PRIORITY,
    pub group_entry: list,
    pub is_group_member: BOOL,
    pub pool_entry: list,
    pub finished_event: RTL_CONDITION_VARIABLE,
    pub group_finished_event: RTL_CONDITION_VARIABLE,
    pub completed_event: HANDLE,
    pub num_pending_callbacks: LONG,
    pub num_running_callbacks: LONG,
    pub num_associated_callbacks: LONG,
    pub u: threadpool_object_u,
}

#[repr(C)]
pub struct threadpool_instance_cleanup {
    pub critical_section: *mut RTL_CRITICAL_SECTION,
    pub mutex: HANDLE,
    pub semaphore: HANDLE,
    pub semaphore_count: LONG,
    pub event: HANDLE,
    pub library: HMODULE,
}

#[repr(C)]
pub struct threadpool_instance {
    pub object: *mut threadpool_object,
    pub threadid: DWORD,
    pub associated: BOOL,
    pub may_run_long: BOOL,
    pub cleanup: threadpool_instance_cleanup,
}

#[repr(C)]
pub struct threadpool_group {
    pub refcount: LONG,
    pub shutdown: BOOL,
    pub cs: RTL_CRITICAL_SECTION,
    pub members: list,
}

unsafe fn impl_from_TP_POOL(pool: *mut TP_POOL) -> *mut threadpool {
    pool as *mut threadpool
}

unsafe fn impl_from_TP_WORK(work: *mut TP_WORK) -> *mut threadpool_object {
    let object = work as *mut threadpool_object;
    object
}

unsafe fn impl_from_TP_TIMER(timer: *mut TP_TIMER) -> *mut threadpool_object {
    let object = timer as *mut threadpool_object;
    object
}

unsafe fn impl_from_TP_WAIT(wait: *mut TP_WAIT) -> *mut threadpool_object {
    let object = wait as *mut threadpool_object;
    object
}

unsafe fn impl_from_TP_IO(io: *mut TP_IO) -> *mut threadpool_object {
    let object = io as *mut threadpool_object;
    object
}

unsafe fn impl_from_TP_CLEANUP_GROUP(group: *mut TP_CLEANUP_GROUP) -> *mut threadpool_group {
    group as *mut threadpool_group
}

unsafe fn impl_from_TP_CALLBACK_INSTANCE( instance: *mut TP_CALLBACK_INSTANCE ) -> *mut threadpool_instance {
    instance as *mut threadpool_instance
}

unsafe fn array_reserve(elements: *mut *mut core::ffi::c_void, capacity: *mut core::ffi::c_uint, count: core::ffi::c_uint, size: core::ffi::c_uint) -> BOOL {
    let mut new_capacity: core::ffi::c_uint;
    let max_capacity: core::ffi::c_uint;
    let new_elements: *mut core::ffi::c_void;

    if count <= *capacity {
        return 1u8 as BOOL;
    }

    max_capacity = ((!(0 as SIZE_T)) / (size as SIZE_T)) as core::ffi::c_uint;
    if count > max_capacity {
        return 0u8 as BOOL;
    }

    new_capacity = if 4 > *capacity { 4 } else { *capacity };
    while new_capacity < count && new_capacity <= max_capacity / 2 {
        new_capacity *= 2;
    }
    if new_capacity < count {
        new_capacity = max_capacity;
    }

    new_elements = RtlReAllocateHeap(GetProcessHeap(), 0, *elements, (new_capacity * size) as SIZE_T);
    if new_elements.is_null() {
        return 0u8 as BOOL;
    }

    *elements = new_elements;
    *capacity = new_capacity;

    1u8 as BOOL
}

unsafe fn set_thread_name(name: *const WCHAR) {
}
unsafe fn process_rtl_work_item(instance: *mut TP_CALLBACK_INSTANCE, userdata: *mut core::ffi::c_void) {
    let item = userdata as *mut rtl_work_item;

    ((*item).function.unwrap())((*item).context);

    RtlFreeHeap(GetProcessHeap(), 0, item as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueueWorkItem(function: WORKERCALLBACKFUNC, context: PVOID, flags: ULONG) -> NTSTATUS {
    let mut environment: TP_CALLBACK_ENVIRON = core::mem::zeroed();
    let status: NTSTATUS;

    let item = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<rtl_work_item>() as SIZE_T) as *mut rtl_work_item;
    if item.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    core::ptr::write_bytes(core::ptr::addr_of_mut!(environment), 0, 1);
    environment.Version = 1;
    environment.u.s.set_LongFunction(((flags & WT_EXECUTELONGFUNCTION) != 0) as ULONG);
    environment.u.s.set_Persistent(((flags & WT_EXECUTEINPERSISTENTTHREAD) != 0) as ULONG);

    (*item).function = function;
    (*item).context = context;

    status = TpSimpleTryPost(Some(process_rtl_work_item), item as PVOID, core::ptr::addr_of_mut!(environment));
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, item as PVOID);
    }
    status
}
unsafe fn iocp_poller(Arg: LPVOID) -> DWORD {
    let cport: HANDLE = Arg;

    loop {
        let mut callback: PRTL_OVERLAPPED_COMPLETION_ROUTINE = core::mem::zeroed();
        let mut overlapped: LPVOID = core::ptr::null_mut();
        let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

        let res = NtRemoveIoCompletion(
            cport,
            core::ptr::addr_of_mut!(callback) as PVOID,
            core::ptr::addr_of_mut!(overlapped) as PVOID,
            &mut iosb,
            core::ptr::null_mut(),
        );

        if res != 0 {
            ERR(b"NtRemoveIoCompletion failed: 0x%lx\n\0".as_ptr() as PCHAR, res);
        } else {
            let mut transferred: DWORD = 0;
            let mut err: DWORD = 0;

            if iosb.__bindgen_anon_1.Status == STATUS_SUCCESS {
                transferred = iosb.Information as DWORD;
            } else {
                err = RtlNtStatusToDosError(iosb.__bindgen_anon_1.Status);
            }

            if let Some(cb) = callback {
                cb(err, transferred, overlapped);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetIoCompletionCallback(FileHandle: HANDLE, Function: PRTL_OVERLAPPED_COMPLETION_ROUTINE, Flags: ULONG) -> NTSTATUS {
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut info: FILE_COMPLETION_INFORMATION = core::mem::zeroed();

    if old_threadpool.compl_port.is_null() {
        let mut res: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

        RtlEnterCriticalSection(core::ptr::addr_of_mut!(old_threadpool.threadpool_compl_cs));
        if old_threadpool.compl_port.is_null() {
            let mut cport: HANDLE = core::ptr::null_mut();

            res = NtCreateIoCompletion(core::ptr::addr_of_mut!(cport), IO_COMPLETION_ALL_ACCESS, core::ptr::null_mut(), 0);
            if res == 0 {
                res = RtlQueueWorkItem(core::mem::transmute(iocp_poller as *const ()), cport, WT_EXECUTEDEFAULT);
                if res == 0 {
                    old_threadpool.compl_port = cport;
                } else {
                    NtClose(cport);
                }
            }
        }
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(old_threadpool.threadpool_compl_cs));
        if res != 0 {
            return res;
        }
    }

    info.CompletionPort = old_threadpool.compl_port;
    info.CompletionKey = Function as ULONG_PTR;

    NtSetInformationFile(FileHandle, core::ptr::addr_of_mut!(iosb), core::ptr::addr_of_mut!(info) as PVOID, core::mem::size_of::<FILE_COMPLETION_INFORMATION>() as ULONG, FileCompletionInformation)
}

unsafe fn get_nt_timeout( pTime: PLARGE_INTEGER, timeout: ULONG ) -> PLARGE_INTEGER {
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
    RtlFreeHeap(GetProcessHeap(), 0, t as *mut core::ffi::c_void);

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
unsafe fn timer_callback_wrapper(p: LPVOID) -> DWORD {
    let t: *mut queue_timer = p as *mut queue_timer;
    ((*t).callback.unwrap())((*t).param, 1u8);
    timer_cleanup_callback(t);
    0
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
            let cur: *mut queue_timer = LIST_ENTRY!(ptr, queue_timer, entry);
            if time < (*cur).expire {
                break;
            }
            ptr = (*ptr).next;
        }
    }
    list_add_before(ptr, core::ptr::addr_of_mut!((*t).entry));

    (*t).expire = time;

    if set_event != 0 && core::ptr::addr_of_mut!((*t).entry) as *mut list == list_head(core::ptr::addr_of_mut!((*q).timers)) {
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
        t = ((list_head(core::ptr::addr_of_mut!((*q).timers)) as usize)
            - (core::mem::offset_of!(queue_timer, entry))) as *mut queue_timer;
        now = queue_current_time();
        if (*t).destroy == 0 && (*t).expire <= now {
            (*t).runcount += 1;
            if (*t).period != 0 {
                let mut n = (*t).expire + (*t).period as ULONGLONG;
                if n < now {
                    n = now + (*t).period as ULONGLONG;
                }
                next = n;
            } else {
                next = EXPIRE_NEVER;
            }
            queue_move_timer(t, next, 0);
        } else {
            t = core::ptr::null_mut();
        }
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    if !t.is_null() {
        if (*t).flags & WT_EXECUTEINTIMERTHREAD != 0 {
            timer_callback_wrapper(t as LPVOID);
        } else {
            let flags: ULONG = (*t).flags
                & (WT_EXECUTEINIOTHREAD | WT_EXECUTEINPERSISTENTTHREAD
                    | WT_EXECUTELONGFUNCTION | WT_TRANSFER_IMPERSONATION);
            let status: NTSTATUS = RtlQueueWorkItem(Some(timer_callback_wrapper), t as PVOID, flags);
            if status != STATUS_SUCCESS as NTSTATUS {
                timer_cleanup_callback(t);
            }
        }
    }
}

unsafe fn queue_get_timeout(q: *mut timer_queue) -> ULONG {
    let mut timeout: ULONG = INFINITE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    let head = list_head(core::ptr::addr_of_mut!((*q).timers));
    if !head.is_null() {
        let t = (head as *mut u8).offset(-(core::mem::offset_of!(queue_timer, entry) as isize)) as *mut queue_timer;

        if (*t).expire != EXPIRE_NEVER {
            let time = queue_current_time();
            timeout = if (*t).expire < time { 0 } else { ((*t).expire - time) as ULONG };
        }
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*q).cs));

    timeout
}
unsafe fn timer_queue_thread_proc(p: PVOID) -> ULONG {
    let q = p as *mut timer_queue;
    let mut timeout_ms: ULONG;

    set_thread_name(b"wine_threadpool_timer_queue\0".as_ptr() as *const WCHAR);
    timeout_ms = INFINITE;
    loop {
        let mut timeout: LARGE_INTEGER = core::mem::zeroed();
        let status: NTSTATUS;
        let mut done: BOOL = 0;

        status = NtWaitForSingleObject(
            (*q).event, 0u8, get_nt_timeout(&mut timeout, timeout_ms));

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
    RtlFreeHeap(GetProcessHeap(), 0, q as PVOID);
    RtlExitUserThread(0);
    STATUS_SUCCESS as ULONG
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
    let q = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<timer_queue>()) as *mut timer_queue;
    if q.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    list_init(core::ptr::addr_of_mut!((*q).timers));
    (*q).quit = 0;
    (*q).magic = TIMER_QUEUE_MAGIC as u32;
    status = NtCreateEvent(
        core::ptr::addr_of_mut!((*q).event),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );
    if status != STATUS_SUCCESS {
        RtlFreeHeap(GetProcessHeap(), 0, q as PVOID);
        return status;
    }
    status = RtlCreateUserThread(
        GetCurrentProcess(),
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        Some(timer_queue_thread_proc),
        q as PVOID,
        core::ptr::addr_of_mut!((*q).thread),
        core::ptr::null_mut(),
    );
    if status != STATUS_SUCCESS {
        NtClose((*q).event);
        RtlFreeHeap(GetProcessHeap(), 0, q as PVOID);
        return status;
    }

    *NewTimerQueue = q as HANDLE;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteTimerQueueEx(TimerQueue: HANDLE, CompletionEvent: HANDLE) -> NTSTATUS {
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
        let mut t = (*core::ptr::addr_of_mut!((*q).timers)).next as *mut queue_timer;
        let mut temp = (*((*core::ptr::addr_of_mut!((*q).timers)).next)).next as *mut queue_timer;
        while core::ptr::addr_of_mut!((*t).entry) != core::ptr::addr_of_mut!((*q).timers) as *mut list {
            queue_destroy_timer(t);
            t = temp;
            temp = (*((*core::ptr::addr_of_mut!((*t).entry)).next)).next as *mut queue_timer;
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
                let p: PVOID = InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!(default_timer_queue) as *mut *mut core::ffi::c_void,
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
    NewTimer: *mut HANDLE,
    Callback: RTL_WAITORTIMERCALLBACKFUNC,
    Parameter: PVOID,
    DueTime: DWORD,
    Period: DWORD,
    Flags: ULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let t: *mut queue_timer;
    let q: *mut timer_queue = get_timer_queue(TimerQueue);

    if q.is_null() {
        return STATUS_NO_MEMORY;
    }
    if (*q).magic != TIMER_QUEUE_MAGIC {
        return STATUS_INVALID_HANDLE;
    }

    t = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        core::mem::size_of::<queue_timer>() as SIZE_T,
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
        RtlFreeHeap(GetProcessHeap(), 0, t as PVOID);
    }

    status
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
    let _ = TimerQueue;
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
            &mut event,
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
unsafe fn timerqueue_thread_proc(param: PVOID) -> ULONG {
    let _ = param;
    let mut timeout_lower: ULONGLONG;
    let mut timeout_upper: ULONGLONG;
    let mut new_timeout: ULONGLONG;
    let mut other_timer: *mut threadpool_object;
    let mut now: LARGE_INTEGER = core::mem::zeroed();
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ptr: *mut list;

    set_thread_name(b"w\0i\0n\0e\0_\0t\0h\0r\0e\0a\0d\0p\0o\0o\0l\0_\0t\0i\0m\0e\0r\0q\0u\0e\0u\0e\0\0\0".as_ptr() as *const WCHAR);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    loop {
        NtQuerySystemTime(core::ptr::addr_of_mut!(now));

        /* Check for expired timers. */
        loop {
            ptr = list_head(core::ptr::addr_of_mut!(timerqueue.pending_timers));
            if ptr.is_null() {
                break;
            }
            let timer: *mut threadpool_object = (ptr as usize
                - core::mem::offset_of!(threadpool_object, u.timer.timer_entry))
                as *mut threadpool_object;
            if (*timer).u.timer.timeout > now.QuadPart as ULONGLONG {
                break;
            }

            /* Queue a new callback in one of the worker threads. */
            list_remove(core::ptr::addr_of_mut!((*timer).u.timer.timer_entry));
            (*timer).u.timer.timer_pending = 0;
            tp_object_submit(timer, 0);

            /* Insert the timer back into the queue, except it's marked for shutdown. */
            if (*timer).u.timer.period != 0 && (*timer).shutdown == 0 {
                (*timer).u.timer.timeout += (*timer).u.timer.period as ULONGLONG * 10000;
                if (*timer).u.timer.timeout <= now.QuadPart as ULONGLONG {
                    (*timer).u.timer.timeout = now.QuadPart as ULONGLONG + 1;
                }

                other_timer = (core::ptr::addr_of_mut!(timerqueue.pending_timers) as usize
                    - core::mem::offset_of!(threadpool_object, u.timer.timer_entry))
                    as *mut threadpool_object;
                let mut entry = (*core::ptr::addr_of_mut!(timerqueue.pending_timers)).next;
                while entry != core::ptr::addr_of_mut!(timerqueue.pending_timers) {
                    other_timer = (entry as usize
                        - core::mem::offset_of!(threadpool_object, u.timer.timer_entry))
                        as *mut threadpool_object;
                    if (*timer).u.timer.timeout < (*other_timer).u.timer.timeout {
                        break;
                    }
                    entry = (*entry).next;
                }
                list_add_before(
                    core::ptr::addr_of_mut!((*other_timer).u.timer.timer_entry),
                    core::ptr::addr_of_mut!((*timer).u.timer.timer_entry),
                );
                (*timer).u.timer.timer_pending = 1;
            }
        }

        timeout_lower = MAXLONGLONG as ULONGLONG;
        timeout_upper = MAXLONGLONG as ULONGLONG;

        /* Determine next timeout and use the window length to optimize wakeup times. */
        let mut entry = (*core::ptr::addr_of_mut!(timerqueue.pending_timers)).next;
        while entry != core::ptr::addr_of_mut!(timerqueue.pending_timers) {
            other_timer = (entry as usize
                - core::mem::offset_of!(threadpool_object, u.timer.timer_entry))
                as *mut threadpool_object;
            if (*other_timer).u.timer.timeout >= timeout_upper {
                break;
            }

            timeout_lower = (*other_timer).u.timer.timeout;
            new_timeout = timeout_lower + (*other_timer).u.timer.window_length as ULONGLONG * 10000;
            if new_timeout < timeout_upper {
                timeout_upper = new_timeout;
            }
            entry = (*entry).next;
        }

        /* Wait for timer update events or until the next timer expires. */
        if timerqueue.objcount != 0 {
            timeout.QuadPart = timeout_lower as i64;
            RtlSleepConditionVariableCS(
                core::ptr::addr_of_mut!(timerqueue.update_event),
                core::ptr::addr_of_mut!(timerqueue.cs),
                core::ptr::addr_of_mut!(timeout),
            );
            continue;
        }

        /* All timers have been destroyed, if no new timers are created
         * within some amount of time, then we can shutdown this thread. */
        timeout.QuadPart = (THREADPOOL_WORKER_TIMEOUT as ULONGLONG).wrapping_mul((-10000i64) as ULONGLONG) as i64;
        if RtlSleepConditionVariableCS(
            core::ptr::addr_of_mut!(timerqueue.update_event),
            core::ptr::addr_of_mut!(timerqueue.cs),
            core::ptr::addr_of_mut!(timeout),
        ) == STATUS_TIMEOUT
            && timerqueue.objcount == 0
        {
            break;
        }
    }

    timerqueue.thread_running = 0;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));

    RtlExitUserThread(0);
    STATUS_SUCCESS as ULONG
}

unsafe fn tp_new_worker_thread(pool: *mut threadpool) -> NTSTATUS {
    let mut thread: HANDLE = core::ptr::null_mut();
    let status: NTSTATUS;

    status = RtlCreateUserThread(
        GetCurrentProcess(),
        core::ptr::null_mut(),
        0u8,
        0,
        (*pool).stack_info.StackReserve,
        (*pool).stack_info.StackCommit,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(*mut core::ffi::c_void) -> ULONG,
            unsafe extern "system" fn(*mut core::ffi::c_void),
        >(threadpool_worker_proc)),
        pool as *mut core::ffi::c_void,
        &mut thread,
        core::ptr::null_mut(),
    );

    if status == STATUS_SUCCESS {
        InterlockedIncrement(core::ptr::addr_of_mut!((*pool).refcount));
        (*pool).num_workers += 1;
        NtClose(thread);
    }
    status
}

unsafe fn tp_timerqueue_lock(timer: *mut _threadpool_object) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    (*timer).u.timer.timer_initialized = 0;
    (*timer).u.timer.timer_pending = 0;
    (*timer).u.timer.timer_set = 0;
    (*timer).u.timer.timeout = 0;
    (*timer).u.timer.period = 0;
    (*timer).u.timer.window_length = 0;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));

    if timerqueue.thread_running == 0 {
        let mut thread: HANDLE = core::ptr::null_mut();
        status = RtlCreateUserThread(
            GetCurrentProcess(),
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            Some(timerqueue_thread_proc),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(thread),
            core::ptr::null_mut(),
        );
        if status == STATUS_SUCCESS as NTSTATUS {
            timerqueue.thread_running = 1;
            NtClose(thread);
        }
    }

    if status == STATUS_SUCCESS as NTSTATUS {
        (*timer).u.timer.timer_initialized = 1;
        timerqueue.objcount += 1;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    status
}

unsafe fn tp_timerqueue_unlock(timer: *mut _threadpool_object) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    if (*timer).u.timer.timer_initialized != 0 {
        if (*timer).u.timer.timer_pending != 0 {
            list_remove(core::ptr::addr_of_mut!((*timer).u.timer.timer_entry));
            (*timer).u.timer.timer_pending = 0;
        }

        timerqueue.objcount -= 1;
        if timerqueue.objcount == 0 {
            RtlWakeAllConditionVariable(core::ptr::addr_of_mut!(timerqueue.update_event));
        }

        (*timer).u.timer.timer_initialized = 0;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
}
unsafe fn waitqueue_thread_proc(param: PVOID) {
    const MAXIMUM_WAITQUEUE_OBJECTS: usize = (MAXIMUM_WAIT_OBJECTS - 1) as usize;

    let mut objects: [*mut threadpool_object; MAXIMUM_WAITQUEUE_OBJECTS] =
        [core::ptr::null_mut(); MAXIMUM_WAITQUEUE_OBJECTS];
    let mut handles: [HANDLE; MAXIMUM_WAITQUEUE_OBJECTS + 1] =
        [core::ptr::null_mut(); MAXIMUM_WAITQUEUE_OBJECTS + 1];
    let bucket = param as *mut waitqueue_bucket;
    let mut wait: *mut threadpool_object;
    let mut next: *mut threadpool_object;
    let mut now: LARGE_INTEGER = core::mem::zeroed();
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut num_handles: DWORD;
    let mut status: NTSTATUS;

    set_thread_name(L!("wine_threadpool_waitqueue"));

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    loop {
        NtQuerySystemTime(core::ptr::addr_of_mut!(now));
        timeout.QuadPart = MAXLONGLONG;
        num_handles = 0;

        wait = list_entry(
            (*bucket).waiting.next,
            core::ptr::null::<threadpool_object>(),
        );
        loop {
            let entry = core::ptr::addr_of_mut!((*bucket).waiting);
            let cur = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
            if cur as *const _ == entry as *const _ {
                break;
            }
            next = list_entry(
                (*wait).u.wait.wait_entry.next,
                core::ptr::null::<threadpool_object>(),
            );

            if (*wait).u.wait.timeout <= now.QuadPart as u64 {
                if ((*wait).u.wait.flags & WT_EXECUTEONLYONCE) != 0 {
                    list_remove(core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
                    list_add_tail(
                        core::ptr::addr_of_mut!((*bucket).reserved),
                        core::ptr::addr_of_mut!((*wait).u.wait.wait_entry),
                    );
                }
                if ((*wait).u.wait.flags & (WT_EXECUTEINWAITTHREAD | WT_EXECUTEINIOTHREAD)) != 0 {
                    InterlockedIncrement(core::ptr::addr_of_mut!((*wait).refcount));
                    (*wait).num_pending_callbacks += 1;
                    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*wait).pool).cs));
                    tp_object_execute(wait, 1);
                    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*wait).pool).cs));
                    tp_object_release(wait);
                } else {
                    tp_object_submit(wait, 0);
                }
            } else {
                if ((*wait).u.wait.timeout as i64) < timeout.QuadPart {
                    timeout.QuadPart = (*wait).u.wait.timeout as i64;
                }

                InterlockedIncrement(core::ptr::addr_of_mut!((*wait).refcount));
                objects[num_handles as usize] = wait;
                handles[num_handles as usize] = (*wait).u.wait.handle;
                num_handles += 1;
            }

            wait = next;
        }

        if (*bucket).objcount == 0 {
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
            timeout.QuadPart = (THREADPOOL_WORKER_TIMEOUT as u64).wrapping_mul((-10000i64) as u64) as i64;
            status = NtWaitForMultipleObjects(
                1,
                core::ptr::addr_of_mut!((*bucket).update_event),
                1,
                (*bucket).alertable as u8,
                core::ptr::addr_of_mut!(timeout),
            );
            RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

            if status == STATUS_TIMEOUT && (*bucket).objcount == 0 {
                break;
            }
        } else {
            handles[num_handles as usize] = (*bucket).update_event;
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
            status = NtWaitForMultipleObjects(
                num_handles + 1,
                handles.as_mut_ptr(),
                1,
                (*bucket).alertable as u8,
                core::ptr::addr_of_mut!(timeout),
            );
            RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

            if status >= STATUS_WAIT_0 && status < STATUS_WAIT_0 + num_handles as i32 {
                wait = objects[(status - STATUS_WAIT_0) as usize];
                if !(*wait).u.wait.bucket.is_null() {
                    if ((*wait).u.wait.flags & WT_EXECUTEONLYONCE) != 0 {
                        list_remove(core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
                        list_add_tail(
                            core::ptr::addr_of_mut!((*bucket).reserved),
                            core::ptr::addr_of_mut!((*wait).u.wait.wait_entry),
                        );
                    }
                    if ((*wait).u.wait.flags & (WT_EXECUTEINWAITTHREAD | WT_EXECUTEINIOTHREAD)) != 0 {
                        (*wait).u.wait.signaled += 1;
                        (*wait).num_pending_callbacks += 1;
                        RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*wait).pool).cs));
                        tp_object_execute(wait, 1);
                        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*wait).pool).cs));
                    } else {
                        tp_object_submit(wait, 1);
                    }
                }
            }

            while num_handles != 0 {
                num_handles -= 1;
                wait = objects[num_handles as usize];
                tp_object_release(wait);
            }
        }

        if waitqueue.num_buckets > 1
            && (*bucket).objcount != 0
            && (*bucket).objcount <= MAXIMUM_WAITQUEUE_OBJECTS as i32 * 1 / 3
        {
            let mut other_bucket: *mut waitqueue_bucket = list_entry(
                waitqueue.buckets.next,
                core::ptr::null::<waitqueue_bucket>(),
            );
            loop {
                let entry = core::ptr::addr_of_mut!(waitqueue.buckets);
                let cur = core::ptr::addr_of_mut!((*other_bucket).bucket_entry);
                if cur as *const _ == entry as *const _ {
                    break;
                }

                if other_bucket != bucket
                    && (*other_bucket).objcount != 0
                    && (*other_bucket).alertable == (*bucket).alertable
                    && (*other_bucket).objcount + (*bucket).objcount
                        <= MAXIMUM_WAITQUEUE_OBJECTS as i32 * 2 / 3
                {
                    (*other_bucket).objcount += (*bucket).objcount;
                    (*bucket).objcount = 0;

                    wait = list_entry(
                        (*bucket).reserved.next,
                        core::ptr::null::<threadpool_object>(),
                    );
                    loop {
                        let e = core::ptr::addr_of_mut!((*bucket).reserved);
                        let c = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
                        if c as *const _ == e as *const _ {
                            break;
                        }
                        (*wait).u.wait.bucket = other_bucket;
                        wait = list_entry(
                            (*wait).u.wait.wait_entry.next,
                            core::ptr::null::<threadpool_object>(),
                        );
                    }
                    list_move_tail(
                        core::ptr::addr_of_mut!((*other_bucket).reserved),
                        core::ptr::addr_of_mut!((*bucket).reserved),
                    );

                    wait = list_entry(
                        (*bucket).waiting.next,
                        core::ptr::null::<threadpool_object>(),
                    );
                    loop {
                        let e = core::ptr::addr_of_mut!((*bucket).waiting);
                        let c = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
                        if c as *const _ == e as *const _ {
                            break;
                        }
                        (*wait).u.wait.bucket = other_bucket;
                        wait = list_entry(
                            (*wait).u.wait.wait_entry.next,
                            core::ptr::null::<threadpool_object>(),
                        );
                    }
                    list_move_tail(
                        core::ptr::addr_of_mut!((*other_bucket).waiting),
                        core::ptr::addr_of_mut!((*bucket).waiting),
                    );

                    list_remove(core::ptr::addr_of_mut!((*bucket).bucket_entry));
                    list_add_tail(
                        core::ptr::addr_of_mut!(waitqueue.buckets),
                        core::ptr::addr_of_mut!((*bucket).bucket_entry),
                    );

                    NtSetEvent((*other_bucket).update_event, core::ptr::null_mut());
                    break;
                }

                other_bucket = list_entry(
                    (*other_bucket).bucket_entry.next,
                    core::ptr::null::<waitqueue_bucket>(),
                );
            }
        }
    }

    list_remove(core::ptr::addr_of_mut!((*bucket).bucket_entry));
    waitqueue.num_buckets -= 1;

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    NtClose((*bucket).update_event);

    RtlFreeHeap(GetProcessHeap(), 0, bucket as PVOID);
    RtlExitUserThread(0);
}

unsafe fn tp_waitqueue_lock(wait: *mut threadpool_object) -> NTSTATUS {
    let mut bucket: *mut waitqueue_bucket;
    let mut status: NTSTATUS;
    let mut thread: HANDLE = core::ptr::null_mut();
    let alertable: BOOL = (((*wait).u.wait.flags & WT_EXECUTEINIOTHREAD) != 0) as BOOL;

    (*wait).u.wait.signaled = 0;
    (*wait).u.wait.bucket = core::ptr::null_mut();
    (*wait).u.wait.wait_pending = 0;
    (*wait).u.wait.timeout = 0;
    (*wait).u.wait.handle = INVALID_HANDLE_VALUE as HANDLE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    status = 'out: {
        bucket = (*(waitqueue.buckets.next as *mut waitqueue_bucket as usize as *mut _ as *mut u8))
            as *mut waitqueue_bucket;
        let mut entry = waitqueue.buckets.next;
        while entry != core::ptr::addr_of_mut!(waitqueue.buckets) {
            bucket = (entry as *mut u8)
                .offset(-(core::mem::offset_of!(waitqueue_bucket, bucket_entry) as isize))
                as *mut waitqueue_bucket;

            if (*bucket).objcount < MAXIMUM_WAITQUEUE_OBJECTS as LONG && (*bucket).alertable == alertable {
                list_add_tail(core::ptr::addr_of_mut!((*bucket).reserved), core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
                (*wait).u.wait.bucket = bucket;
                (*bucket).objcount += 1;

                break 'out STATUS_SUCCESS;
            }

            entry = (*entry).next;
        }

        bucket = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<waitqueue_bucket>()) as *mut waitqueue_bucket;
        if bucket.is_null() {
            break 'out STATUS_NO_MEMORY;
        }

        (*bucket).objcount = 0;
        (*bucket).alertable = alertable;
        list_init(core::ptr::addr_of_mut!((*bucket).reserved));
        list_init(core::ptr::addr_of_mut!((*bucket).waiting));

        status = NtCreateEvent(core::ptr::addr_of_mut!((*bucket).update_event), EVENT_ALL_ACCESS,
                               core::ptr::null_mut(), _EVENT_TYPE_SynchronizationEvent, 0);
        if status != 0 {
            RtlFreeHeap(GetProcessHeap(), 0, bucket as PVOID);
            break 'out status;
        }

        status = RtlCreateUserThread(GetCurrentProcess(), core::ptr::null_mut(), 0, 0, 0, 0,
                                     core::mem::transmute::<_, PTHREAD_START_ROUTINE>(waitqueue_thread_proc as usize), bucket as PVOID,
                                     core::ptr::addr_of_mut!(thread), core::ptr::null_mut());
        if status == STATUS_SUCCESS {
            list_add_tail(core::ptr::addr_of_mut!(waitqueue.buckets), core::ptr::addr_of_mut!((*bucket).bucket_entry));
            waitqueue.num_buckets += 1;

            list_add_tail(core::ptr::addr_of_mut!((*bucket).reserved), core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
            (*wait).u.wait.bucket = bucket;
            (*bucket).objcount += 1;

            NtClose(thread);
        } else {
            NtClose((*bucket).update_event);
            RtlFreeHeap(GetProcessHeap(), 0, bucket as PVOID);
        }

        status
    };

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    status
}

unsafe fn tp_waitqueue_unlock(wait: *mut threadpool_object) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    if !(*wait).u.wait.bucket.is_null() {
        let bucket = (*wait).u.wait.bucket;

        list_remove(core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
        (*wait).u.wait.bucket = core::ptr::null_mut();
        (*bucket).objcount -= 1;

        NtSetEvent((*bucket).update_event, core::ptr::null_mut());
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
}
unsafe fn ioqueue_thread_proc(param: PVOID) -> ULONG {
    let _ = param;
    let mut completion: *mut io_completion;
    let mut io: *mut threadpool_object;
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut key: PVOID = core::ptr::null_mut();
    let mut value: PVOID = core::ptr::null_mut();
    let mut destroy: BOOL;
    let mut skip: BOOL;
    let mut status: NTSTATUS;

    set_thread_name(b"wine_threadpool_ioqueue\0".as_ptr() as *const WCHAR as *const u16 as *const WCHAR);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));

    loop {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
        status = NtRemoveIoCompletion(
            ioqueue.port,
            core::ptr::addr_of_mut!(key),
            core::ptr::addr_of_mut!(value),
            core::ptr::addr_of_mut!(iosb),
            core::ptr::null_mut(),
        );
        let _ = status;
        RtlEnterCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));

        destroy = 0;
        skip = 0;
        io = key as *mut threadpool_object;

        if !io.is_null() && ((*io).shutdown != 0 || (*io).u.io.shutting_down != 0) {
            RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*io).pool).cs));
            if (*io).u.io.pending_count == 0 {
                if (*io).u.io.skipped_count != 0 {
                    (*io).u.io.skipped_count -= 1;
                }

                if (*io).u.io.skipped_count != 0 {
                    skip = 1;
                } else {
                    destroy = 1;
                }
            }
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*io).pool).cs));
            if skip != 0 {
                continue;
            }
        }

        if destroy != 0 {
            ioqueue.objcount -= 1;
            (*io).shutdown = 1;
            tp_object_release(io);
        } else if !io.is_null() {
            RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*io).pool).cs));

            if (*io).u.io.pending_count != 0 {
                (*io).u.io.pending_count -= 1;
                if array_reserve(
                    core::ptr::addr_of_mut!((*io).u.io.completions) as *mut *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!((*io).u.io.completion_max),
                    (*io).u.io.completion_count + 1,
                    core::mem::size_of::<io_completion>() as core::ffi::c_uint,
                ) == 0
                {
                    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*io).pool).cs));
                    continue;
                }

                completion = (*io).u.io.completions.add((*io).u.io.completion_count as usize);
                (*io).u.io.completion_count += 1;
                (*completion).iosb = iosb;
                (*completion).cvalue = value as ULONG_PTR;

                tp_object_submit(io, 0);
            }
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*io).pool).cs));
        }

        if ioqueue.objcount == 0 {
            let mut timeout: LARGE_INTEGER = core::mem::zeroed();
            timeout.QuadPart = (THREADPOOL_WORKER_TIMEOUT as u64).wrapping_mul((-10000i64) as u64) as i64;
            if RtlSleepConditionVariableCS(
                core::ptr::addr_of_mut!(ioqueue.update_event),
                core::ptr::addr_of_mut!(ioqueue.cs),
                core::ptr::addr_of_mut!(timeout),
            ) == STATUS_TIMEOUT
                && ioqueue.objcount == 0
            {
                break;
            }
        }
    }

    ioqueue.thread_running = 0;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));

    RtlExitUserThread(0);

    STATUS_SUCCESS as ULONG
}

unsafe fn tp_ioqueue_lock(io: *mut threadpool_object, file: HANDLE) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));

    if ioqueue.port.is_null() {
        status = NtCreateIoCompletion(
            core::ptr::addr_of_mut!(ioqueue.port),
            IO_COMPLETION_ALL_ACCESS,
            core::ptr::null_mut(),
            0,
        );
        if status != 0 {
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
            return status;
        }
    }

    if ioqueue.thread_running == 0 {
        let mut thread: HANDLE = core::ptr::null_mut();

        status = RtlCreateUserThread(
            GetCurrentProcess(),
            core::ptr::null_mut(),
            0u8,
            0,
            0,
            0,
            Some(ioqueue_thread_proc),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(thread),
            core::ptr::null_mut(),
        );
        if status == 0 {
            ioqueue.thread_running = 1;
            NtClose(thread);
        }
    }

    if status == STATUS_SUCCESS {
        let mut info: FILE_COMPLETION_INFORMATION = core::mem::zeroed();
        let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

        info.Port = ioqueue.port;
        info.Key = io as PVOID;

        status = NtSetInformationFile(
            file,
            core::ptr::addr_of_mut!(iosb),
            core::ptr::addr_of_mut!(info) as PVOID,
            core::mem::size_of::<FILE_COMPLETION_INFORMATION>() as u32,
            FileCompletionInformation,
        );
    }

    if status == STATUS_SUCCESS {
        let old = ioqueue.objcount;
        ioqueue.objcount += 1;
        if old == 0 {
            RtlWakeConditionVariable(core::ptr::addr_of_mut!(ioqueue.update_event));
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
    status
}

unsafe fn tp_threadpool_alloc(out: *mut *mut threadpool) -> NTSTATUS {
    let nt: *mut IMAGE_NT_HEADERS = RtlImageNtHeader(
        (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ImageBaseAddress,
    );
    let pool: *mut threadpool;
    let mut i: u32;

    pool = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool>()) as *mut threadpool;
    if pool.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    (*pool).refcount = 1;
    (*pool).objcount = 0;
    (*pool).shutdown = 0;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    i = 0;
    while (i as usize) < (*pool).pools.len() {
        list_init(core::ptr::addr_of_mut!((*pool).pools[i as usize]));
        i += 1;
    }
    RtlInitializeConditionVariable(core::ptr::addr_of_mut!((*pool).update_event));

    (*pool).max_workers = 500;
    (*pool).min_workers = 0;
    (*pool).num_workers = 0;
    (*pool).num_busy_workers = 0;
    (*pool).stack_info.StackReserve = (*nt).OptionalHeader.SizeOfStackReserve;
    (*pool).stack_info.StackCommit = (*nt).OptionalHeader.SizeOfStackCommit;

    TRACE(b"allocated threadpool %p\n\0".as_ptr() as *const i8, pool);

    *out = pool;
    STATUS_SUCCESS as NTSTATUS
}

unsafe fn tp_threadpool_shutdown(pool: *mut threadpool) {
    (*pool).shutdown = 1;
    RtlWakeAllConditionVariable(core::ptr::addr_of_mut!((*pool).update_event));
}

unsafe fn tp_threadpool_release(pool: *mut threadpool) -> BOOL {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*pool).refcount)) != 0 {
        return 0u8 as BOOL;
    }

    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    RtlFreeHeap(GetProcessHeap(), 0, pool as PVOID);
    1u8 as BOOL
}

unsafe fn tp_threadpool_lock(out: *mut *mut threadpool, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    if !environment.is_null() {
        pool = (*environment).Pool as *mut threadpool;
    }

    if pool.is_null() {
        if default_threadpool.is_null() {
            status = tp_threadpool_alloc(core::ptr::addr_of_mut!(pool));
            if status != STATUS_SUCCESS as NTSTATUS {
                return status;
            }

            if !InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(default_threadpool) as *mut PVOID,
                pool as PVOID,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                tp_threadpool_shutdown(pool);
                tp_threadpool_release(pool);
            }
        }

        pool = default_threadpool;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    if (*pool).num_workers == 0 {
        status = tp_new_worker_thread(pool);
    }

    if status == STATUS_SUCCESS as NTSTATUS {
        InterlockedIncrement(core::ptr::addr_of_mut!((*pool).refcount));
        (*pool).objcount += 1;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    if status != STATUS_SUCCESS as NTSTATUS {
        return status;
    }

    *out = pool;
    STATUS_SUCCESS as NTSTATUS
}

unsafe fn tp_threadpool_unlock(pool: *mut threadpool) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
    (*pool).objcount -= 1;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
    tp_threadpool_release(pool);
}

unsafe fn tp_group_alloc(out: *mut *mut threadpool_group) -> NTSTATUS {
    let group: *mut threadpool_group = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        core::mem::size_of::<threadpool_group>() as SIZE_T,
    ) as *mut threadpool_group;
    if group.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*group).refcount = 1;
    (*group).shutdown = 0;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*group).cs));

    list_init(core::ptr::addr_of_mut!((*group).members));

    *out = group;
    STATUS_SUCCESS
}

unsafe fn tp_group_shutdown(group: *mut threadpool_group) {
    (*group).shutdown = 1;
}

unsafe fn tp_group_release(group: *mut threadpool_group) -> BOOL {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*group).refcount)) != 0 {
        return 0;
    }

    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*group).cs));

    RtlFreeHeap(GetProcessHeap(), 0, group as PVOID);
    1
}

unsafe fn tp_object_initialize(object: *mut threadpool_object, pool: *mut threadpool, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) {
    let is_simple_callback: BOOL = ((*object).type_ == TP_OBJECT_TYPE_SIMPLE) as BOOL;

    (*object).refcount = 1;
    (*object).shutdown = 0;

    (*object).pool = pool;
    (*object).group = core::ptr::null_mut();
    (*object).userdata = userdata;
    (*object).group_cancel_callback = None;
    (*object).finalization_callback = None;
    (*object).may_run_long = 0;
    (*object).race_dll = core::ptr::null_mut();
    (*object).priority = TP_CALLBACK_PRIORITY_NORMAL;

    core::ptr::write_bytes(core::ptr::addr_of_mut!((*object).group_entry), 0, 1);
    (*object).is_group_member = 0;

    core::ptr::write_bytes(core::ptr::addr_of_mut!((*object).pool_entry), 0, 1);
    RtlInitializeConditionVariable(core::ptr::addr_of_mut!((*object).finished_event));
    RtlInitializeConditionVariable(core::ptr::addr_of_mut!((*object).group_finished_event));
    (*object).completed_event = core::ptr::null_mut();
    (*object).num_pending_callbacks = 0;
    (*object).num_running_callbacks = 0;
    (*object).num_associated_callbacks = 0;

    if !environment.is_null() {
        if (*environment).Version != 1 && (*environment).Version != 3 {
            FIXME(b"unsupported environment version %lu\n\0".as_ptr() as *const i8, (*environment).Version);
        }

        (*object).group = impl_from_TP_CLEANUP_GROUP((*environment).CleanupGroup);
        (*object).group_cancel_callback = (*environment).CleanupGroupCancelCallback;
        (*object).finalization_callback = (*environment).FinalizationCallback;
        (*object).may_run_long = ((*environment).u.s.LongFunction() != 0) as BOOL;
        (*object).race_dll = (*environment).RaceDll;

        if !(*environment).ActivationContext.is_null() {
            FIXME(b"activation context not supported yet\n\0".as_ptr() as *const i8);
        }

        if (*environment).u.s.Persistent() != 0 {
            FIXME(b"persistent threads not supported yet\n\0".as_ptr() as *const i8);
        }
    }

    if !(*object).race_dll.is_null() {
        LdrAddRefDll(0, (*object).race_dll);
    }

    TRACE(b"allocated object %p of type %u\n\0".as_ptr() as *const i8, object, (*object).type_);

    if is_simple_callback != 0 {
        tp_object_submit(object, 0);
    }

    if !(*object).group.is_null() {
        let group: *mut threadpool_group = (*object).group;
        InterlockedIncrement(core::ptr::addr_of_mut!((*group).refcount));

        RtlEnterCriticalSection(core::ptr::addr_of_mut!((*group).cs));
        list_add_tail(core::ptr::addr_of_mut!((*group).members), core::ptr::addr_of_mut!((*object).group_entry));
        (*object).is_group_member = 1;
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*group).cs));
    }

    if is_simple_callback != 0 {
        tp_object_release(object);
    }
}

unsafe fn tp_object_prio_queue(object: *mut threadpool_object) {
    (*(*object).pool).num_busy_workers += 1;
    list_add_tail(
        core::ptr::addr_of_mut!((*(*object).pool).pools[(*object).priority as usize]),
        core::ptr::addr_of_mut!((*object).pool_entry),
    );
}

unsafe fn tp_object_submit(object: *mut threadpool_object, signaled: BOOL) {
    let pool: *mut threadpool = (*object).pool;
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    if (*pool).num_busy_workers >= (*pool).num_workers && (*pool).num_workers < (*pool).max_workers {
        status = tp_new_worker_thread(pool);
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*object).refcount));
    let prev = (*object).num_pending_callbacks;
    (*object).num_pending_callbacks += 1;
    if prev == 0 {
        tp_object_prio_queue(object);
    }

    if (*object).type_ == TP_OBJECT_TYPE_WAIT && signaled != 0 {
        (*object).u.wait.signaled += 1;
    }

    if status != STATUS_SUCCESS {
        RtlWakeConditionVariable(core::ptr::addr_of_mut!((*pool).update_event));
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
}

unsafe fn tp_object_cancel(object: *mut threadpool_object) {
    let pool = (*object).pool;
    let mut pending_callbacks: LONG = 0;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
    if (*object).num_pending_callbacks != 0 {
        pending_callbacks = (*object).num_pending_callbacks;
        (*object).num_pending_callbacks = 0;
        list_remove(core::ptr::addr_of_mut!((*object).pool_entry));

        if (*object).type_ == TP_OBJECT_TYPE_WAIT {
            (*object).u.wait.signaled = 0;
        }
    }
    if (*object).type_ == TP_OBJECT_TYPE_IO {
        (*object).u.io.skipped_count += (*object).u.io.pending_count;
        (*object).u.io.pending_count = 0;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    while pending_callbacks != 0 {
        pending_callbacks -= 1;
        tp_object_release(object);
    }
}

unsafe fn object_is_finished(object: *mut threadpool_object, group: BOOL) -> BOOL {
    if (*object).num_pending_callbacks != 0 {
        return 0;
    }
    if (*object).type == enum_threadpool_objtype_TP_OBJECT_TYPE_IO
        && (*object).u.io.pending_count != 0
    {
        return 0;
    }

    if group != 0 {
        if (*object).num_running_callbacks == 0 { 1 } else { 0 }
    } else {
        if (*object).num_associated_callbacks == 0 { 1 } else { 0 }
    }
}

unsafe fn tp_object_wait(object: *mut threadpool_object, group_wait: BOOL) {
    let pool = (*object).pool;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
    while object_is_finished(object, group_wait) == 0 {
        if group_wait != 0 {
            RtlSleepConditionVariableCS(
                core::ptr::addr_of_mut!((*object).group_finished_event),
                core::ptr::addr_of_mut!((*pool).cs),
                core::ptr::null_mut(),
            );
        } else {
            RtlSleepConditionVariableCS(
                core::ptr::addr_of_mut!((*object).finished_event),
                core::ptr::addr_of_mut!((*pool).cs),
                core::ptr::null_mut(),
            );
        }
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
}

unsafe fn tp_ioqueue_unlock(io: *mut threadpool_object) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));

    if (*io).shutdown == 0 {
        ioqueue.objcount -= 1;
        if ioqueue.objcount == 0 {
            NtSetIoCompletion(ioqueue.port, 0, 0, STATUS_SUCCESS, 0);
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
}

unsafe fn tp_object_prepare_shutdown(object: *mut threadpool_object) {
    if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_TIMER {
        tp_timerqueue_unlock(object);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_WAIT {
        tp_waitqueue_unlock(object);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_IO {
        tp_ioqueue_unlock(object);
    }
}

unsafe fn tp_object_release(object: *mut threadpool_object) -> BOOL {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*object).refcount)) != 0 {
        return 0;
    }

    if !(*object).group.is_null() {
        let group = (*object).group;

        RtlEnterCriticalSection(core::ptr::addr_of_mut!((*group).cs));
        if (*object).is_group_member != 0 {
            list_remove(core::ptr::addr_of_mut!((*object).group_entry));
            (*object).is_group_member = 0;
        }
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*group).cs));

        tp_group_release(group);
    }

    tp_threadpool_unlock((*object).pool);

    if !(*object).race_dll.is_null() {
        LdrUnloadDll((*object).race_dll);
    }

    if !(*object).completed_event.is_null()
        && (*object).completed_event != INVALID_HANDLE_VALUE as HANDLE
    {
        NtSetEvent((*object).completed_event, core::ptr::null_mut());
    }

    RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
    1
}

unsafe fn threadpool_get_next_item(pool: *const threadpool) -> *mut list {
    let mut ptr: *mut list = core::ptr::null_mut();

    let count = (*pool).pools.len();
    let mut i: usize = 0;
    while i < count {
        ptr = list_head(core::ptr::addr_of!((*pool).pools[i]) as *const list);
        if !ptr.is_null() {
            break;
        }
        i += 1;
    }

    ptr
}

unsafe fn tp_object_execute(object: *mut threadpool_object, wait_thread: BOOL) {
    let callback_instance: *mut TP_CALLBACK_INSTANCE;
    let mut instance: threadpool_instance = core::mem::zeroed();
    let mut completion: io_completion = core::mem::zeroed();
    let pool = (*object).pool;
    let mut wait_result: TP_WAIT_RESULT = 0;
    let mut status: NTSTATUS;

    (*object).num_pending_callbacks -= 1;

    if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_WAIT {
        wait_result = if (*object).u.wait.signaled != 0 { WAIT_OBJECT_0 } else { WAIT_TIMEOUT };
        if wait_result == WAIT_OBJECT_0 {
            (*object).u.wait.signaled -= 1;
        }
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_IO {
        (*object).u.io.completion_count -= 1;
        completion = *(*object).u.io.completions.offset((*object).u.io.completion_count as isize);
    }

    (*object).num_associated_callbacks += 1;
    (*object).num_running_callbacks += 1;
    RtlLeaveCriticalSection(&mut (*pool).cs);
    if wait_thread != 0 {
        RtlLeaveCriticalSection(&mut waitqueue.cs);
    }

    callback_instance = &mut instance as *mut threadpool_instance as *mut TP_CALLBACK_INSTANCE;
    instance.object = object;
    instance.threadid = GetCurrentThreadId();
    instance.associated = 1;
    instance.may_run_long = (*object).may_run_long;
    instance.cleanup.critical_section = core::ptr::null_mut();
    instance.cleanup.mutex = core::ptr::null_mut();
    instance.cleanup.semaphore = core::ptr::null_mut();
    instance.cleanup.semaphore_count = 0;
    instance.cleanup.event = core::ptr::null_mut();
    instance.cleanup.library = core::ptr::null_mut();

    if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_SIMPLE {
        (*object).u.simple.callback.unwrap()(callback_instance, (*object).userdata);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_WORK {
        (*object).u.work.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_WORK);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_TIMER {
        (*object).u.timer.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_TIMER);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_WAIT {
        (*object).u.wait.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_WAIT, wait_result);
    } else if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_IO {
        (*object).u.io.callback.unwrap()(callback_instance, (*object).userdata,
            completion.cvalue as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(completion.iosb), object as *mut TP_IO);
    }

    if (*object).finalization_callback.is_some() {
        (*object).finalization_callback.unwrap()(callback_instance, (*object).userdata);
    }

    'skip_cleanup: {
        if !instance.cleanup.critical_section.is_null() {
            RtlLeaveCriticalSection(instance.cleanup.critical_section);
        }
        if !instance.cleanup.mutex.is_null() {
            status = NtReleaseMutant(instance.cleanup.mutex, core::ptr::null_mut());
            if status != STATUS_SUCCESS {
                break 'skip_cleanup;
            }
        }
        if !instance.cleanup.semaphore.is_null() {
            status = NtReleaseSemaphore(instance.cleanup.semaphore, instance.cleanup.semaphore_count, core::ptr::null_mut());
            if status != STATUS_SUCCESS {
                break 'skip_cleanup;
            }
        }
        if !instance.cleanup.event.is_null() {
            status = NtSetEvent(instance.cleanup.event, core::ptr::null_mut());
            if status != STATUS_SUCCESS {
                break 'skip_cleanup;
            }
        }
        if !instance.cleanup.library.is_null() {
            LdrUnloadDll(instance.cleanup.library);
        }
    }

    if wait_thread != 0 {
        RtlEnterCriticalSection(&mut waitqueue.cs);
    }
    RtlEnterCriticalSection(&mut (*pool).cs);

    if (*object).type_ == _threadpool_objtype_TP_OBJECT_TYPE_SIMPLE {
        tp_object_prepare_shutdown(object);
        (*object).shutdown = 1;
    }

    (*object).num_running_callbacks -= 1;
    if object_is_finished(object, 1) != 0 {
        RtlWakeAllConditionVariable(&mut (*object).group_finished_event);
    }

    if instance.associated != 0 {
        (*object).num_associated_callbacks -= 1;
        if object_is_finished(object, 0) != 0 {
            RtlWakeAllConditionVariable(&mut (*object).finished_event);
        }
    }
}
unsafe fn threadpool_worker_proc(param: PVOID) -> ULONG {
    let pool = param as *mut threadpool;
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ptr: *mut list;

    set_thread_name(core::ptr::null());

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*pool).cs));
    loop {
        loop {
            ptr = threadpool_get_next_item(pool);
            if ptr.is_null() {
                break;
            }

            let object = (ptr as usize
                - core::mem::offset_of!(threadpool_object, pool_entry))
                as *mut threadpool_object;

            list_remove(core::ptr::addr_of_mut!((*object).pool_entry));
            if (*object).num_pending_callbacks > 1 {
                tp_object_prio_queue(object);
            }

            tp_object_execute(object, 0);

            (*pool).num_busy_workers -= 1;

            tp_object_release(object);
        }

        if (*pool).shutdown != 0 {
            break;
        }

        timeout.QuadPart = (THREADPOOL_WORKER_TIMEOUT as u64).wrapping_mul((-10000i64) as u64) as i64;
        if RtlSleepConditionVariableCS(
            core::ptr::addr_of_mut!((*pool).update_event),
            core::ptr::addr_of_mut!((*pool).cs),
            core::ptr::addr_of_mut!(timeout),
        ) == STATUS_TIMEOUT
            && threadpool_get_next_item(pool).is_null()
            && ((*pool).num_workers
                > if (*pool).min_workers > 1 { (*pool).min_workers } else { 1 }
                || ((*pool).min_workers == 0 && (*pool).objcount == 0))
        {
            break;
        }
    }
    (*pool).num_workers -= 1;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    tp_threadpool_release(pool);
    RtlExitUserThread(0);

    STATUS_SUCCESS as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocCleanupGroup(out: *mut *mut TP_CLEANUP_GROUP) -> NTSTATUS {
    tp_group_alloc(out as *mut *mut threadpool_group)
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocIoCompletion(
    out: *mut *mut TP_IO,
    file: HANDLE,
    callback: PTP_IO_CALLBACK,
    userdata: *mut core::ffi::c_void,
    environment: *mut TP_CALLBACK_ENVIRON,
) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let object = RtlAllocateHeap(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<threadpool_object>(),
    ) as *mut threadpool_object;
    if object.is_null() {
        return STATUS_NO_MEMORY;
    }

    status = tp_threadpool_lock(core::ptr::addr_of_mut!(pool), environment);
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, object as *mut core::ffi::c_void);
        return status;
    }

    (*object).type_ = TP_OBJECT_TYPE_IO;
    (*object).u.io.callback = callback;
    (*object).u.io.completions = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        8 * core::mem::size_of::<io_completion>(),
    ) as *mut io_completion;
    if (*object).u.io.completions.is_null() {
        tp_threadpool_unlock(pool);
        RtlFreeHeap(GetProcessHeap(), 0, object as *mut core::ffi::c_void);
        return status;
    }

    status = tp_ioqueue_lock(object, file);
    if status != 0 {
        tp_threadpool_unlock(pool);
        RtlFreeHeap(
            GetProcessHeap(),
            0,
            (*object).u.io.completions as *mut core::ffi::c_void,
        );
        RtlFreeHeap(GetProcessHeap(), 0, object as *mut core::ffi::c_void);
        return status;
    }

    tp_object_initialize(object, pool, userdata, environment);

    *out = object as *mut TP_IO;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocPool(out: *mut *mut TP_POOL, reserved: PVOID) -> NTSTATUS {
    if !reserved.is_null() {
    }

    tp_threadpool_alloc(out as *mut *mut threadpool)
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocTimer(out: *mut *mut TP_TIMER, callback: PTP_TIMER_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as SIZE_T) as *mut threadpool_object;
    if object.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    status = tp_threadpool_lock(core::ptr::addr_of_mut!(pool), environment);
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    (*object).type_ = TP_OBJECT_TYPE_TIMER;
    (*object).u.timer.callback = callback;

    status = tp_timerqueue_lock(object);
    if status != 0 {
        tp_threadpool_unlock(pool);
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    tp_object_initialize(object, pool, userdata, environment);

    *out = object as *mut TP_TIMER;
    STATUS_SUCCESS as NTSTATUS
}

unsafe fn tp_alloc_wait(out: *mut *mut TP_WAIT, callback: PTP_WAIT_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON, flags: DWORD) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as SIZE_T) as *mut threadpool_object;
    if object.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    status = tp_threadpool_lock(core::ptr::addr_of_mut!(pool), environment);
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    (*object).type_ = TP_OBJECT_TYPE_WAIT;
    (*object).u.wait.callback = callback;
    (*object).u.wait.flags = flags;

    status = tp_waitqueue_lock(object);
    if status != 0 {
        tp_threadpool_unlock(pool);
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    tp_object_initialize(object, pool, userdata, environment);

    *out = object as *mut TP_WAIT;
    STATUS_SUCCESS as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocWait(out: *mut *mut TP_WAIT, callback: PTP_WAIT_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    tp_alloc_wait(out, callback, userdata, environment, WT_EXECUTEONLYONCE as DWORD)
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocWork(out: *mut *mut TP_WORK, callback: PTP_WORK_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as SIZE_T) as *mut threadpool_object;
    if object.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    status = tp_threadpool_lock(core::ptr::addr_of_mut!(pool), environment);
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    (*object).type_ = TP_OBJECT_TYPE_WORK;
    (*object).u.work.callback = callback;
    tp_object_initialize(object, pool, userdata, environment);

    *out = object as *mut TP_WORK;
    STATUS_SUCCESS as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn TpCancelAsyncIoOperation(io: *mut TP_IO) {
    let this = impl_from_TP_IO(io);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));

    (*this).u.io.pending_count -= 1;
    if object_is_finished(this, 1) != 0 {
        RtlWakeAllConditionVariable(core::ptr::addr_of_mut!((*this).group_finished_event));
    }
    if object_is_finished(this, 0) != 0 {
        RtlWakeAllConditionVariable(core::ptr::addr_of_mut!((*this).finished_event));
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackLeaveCriticalSectionOnCompletion( instance: *mut TP_CALLBACK_INSTANCE, crit: *mut CRITICAL_SECTION ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );

    if (*this).cleanup.critical_section.is_null() {
        (*this).cleanup.critical_section = crit;
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackMayRunLong( instance: *mut TP_CALLBACK_INSTANCE ) -> NTSTATUS {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );
    let object = (*this).object;
    let pool: *mut threadpool;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if (*this).threadid != GetCurrentThreadId() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*this).may_run_long != 0 {
        return STATUS_SUCCESS;
    }

    pool = (*object).pool;
    RtlEnterCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );

    if (*pool).num_busy_workers >= (*pool).num_workers {
        if (*pool).num_workers < (*pool).max_workers {
            status = tp_new_worker_thread( pool );
        } else {
            status = STATUS_TOO_MANY_THREADS;
        }
    }

    RtlLeaveCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );
    (*this).may_run_long = 1;
    status
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackReleaseMutexOnCompletion( instance: *mut TP_CALLBACK_INSTANCE, mutex: HANDLE ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );

    if (*this).cleanup.mutex.is_null() {
        (*this).cleanup.mutex = mutex;
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackReleaseSemaphoreOnCompletion( instance: *mut TP_CALLBACK_INSTANCE, semaphore: HANDLE, count: DWORD ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );

    if (*this).cleanup.semaphore.is_null() {
        (*this).cleanup.semaphore = semaphore;
        (*this).cleanup.semaphore_count = count as LONG;
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackSetEventOnCompletion( instance: *mut TP_CALLBACK_INSTANCE, event: HANDLE ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );

    if (*this).cleanup.event.is_null() {
        (*this).cleanup.event = event;
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpCallbackUnloadDllOnCompletion( instance: *mut TP_CALLBACK_INSTANCE, module: HMODULE ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );

    if (*this).cleanup.library.is_null() {
        (*this).cleanup.library = module;
    }
}
#[no_mangle]
pub unsafe extern "system" fn TpDisassociateCallback( instance: *mut TP_CALLBACK_INSTANCE ) {
    let this = impl_from_TP_CALLBACK_INSTANCE( instance );
    let object = (*this).object;
    let pool;

    if (*this).threadid != GetCurrentThreadId()
    {
        return;
    }

    if (*this).associated == 0
    {
        return;
    }

    pool = (*object).pool;
    RtlEnterCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );

    (*object).num_associated_callbacks -= 1;
    if object_is_finished( object, 0 ) != 0
    {
        RtlWakeAllConditionVariable( core::ptr::addr_of_mut!((*object).finished_event) );
    }

    RtlLeaveCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );
    (*this).associated = 0;
}

#[no_mangle]
pub unsafe extern "system" fn TpIsTimerSet(timer: *mut TP_TIMER) -> BOOL {
    let this = impl_from_TP_TIMER(timer);

    (*this).u.timer.timer_set
}

#[no_mangle]
pub unsafe extern "system" fn TpPostWork(work: *mut TP_WORK) {
    let this: *mut threadpool_object = impl_from_TP_WORK(work);

    tp_object_submit(this, 0u8 as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseCleanupGroup(group: *mut TP_CLEANUP_GROUP) {
    let this = impl_from_TP_CLEANUP_GROUP(group);

    tp_group_shutdown(this);
    tp_group_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseCleanupGroupMembers(
    group: *mut TP_CLEANUP_GROUP,
    cancel_pending: BOOL,
    userdata: PVOID,
) {
    let this = impl_from_TP_CLEANUP_GROUP(group);
    let mut members: list = core::mem::zeroed();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    /* Unset group, increase references, and mark objects for shutdown */
    let mut object: *mut threadpool_object =
        (*core::ptr::addr_of_mut!((*this).members)).next as *mut threadpool_object;
    let list_head_ptr = core::ptr::addr_of_mut!((*this).members);
    loop {
        let object_entry = core::ptr::addr_of_mut!((*this).members);
        let cur_entry = ((object as *mut u8)
            .offset(core::mem::offset_of!(threadpool_object, group_entry) as isize))
            as *mut list;
        if cur_entry == object_entry {
            break;
        }
        let next = (*core::ptr::addr_of_mut!((*object).group_entry)).next as *mut threadpool_object;

        if InterlockedIncrement(core::ptr::addr_of_mut!((*object).refcount)) == 1 {
            /* Object is basically already destroyed, but group reference
             * was not deleted yet. We can safely ignore this object. */
            InterlockedDecrement(core::ptr::addr_of_mut!((*object).refcount));
            list_remove(core::ptr::addr_of_mut!((*object).group_entry));
            (*object).is_group_member = 0;
            object = next;
            continue;
        }

        (*object).is_group_member = 0;
        tp_object_prepare_shutdown(object);

        object = next;
    }
    let _ = list_head_ptr;

    /* Move members to a new temporary list */
    list_init(core::ptr::addr_of_mut!(members));
    list_move_tail(core::ptr::addr_of_mut!(members), core::ptr::addr_of_mut!((*this).members));

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    /* Cancel pending callbacks if requested */
    if cancel_pending != 0 {
        let mut obj: *mut threadpool_object =
            (members.next as *mut u8).offset(-(core::mem::offset_of!(threadpool_object, group_entry) as isize))
                as *mut threadpool_object;
        loop {
            let cur_entry = core::ptr::addr_of_mut!((*obj).group_entry);
            if cur_entry == core::ptr::addr_of_mut!(members) {
                break;
            }
            tp_object_cancel(obj);

            obj = ((*core::ptr::addr_of_mut!((*obj).group_entry)).next as *mut u8)
                .offset(-(core::mem::offset_of!(threadpool_object, group_entry) as isize))
                as *mut threadpool_object;
        }
    }

    /* Wait for remaining callbacks to finish */
    let mut obj: *mut threadpool_object = (members.next as *mut u8)
        .offset(-(core::mem::offset_of!(threadpool_object, group_entry) as isize))
        as *mut threadpool_object;
    loop {
        let cur_entry = core::ptr::addr_of_mut!((*obj).group_entry);
        if cur_entry == core::ptr::addr_of_mut!(members) {
            break;
        }
        let next: *mut threadpool_object = ((*core::ptr::addr_of_mut!((*obj).group_entry)).next
            as *mut u8)
            .offset(-(core::mem::offset_of!(threadpool_object, group_entry) as isize))
            as *mut threadpool_object;

        tp_object_wait(obj, 1);

        if (*obj).shutdown == 0 {
            /* Execute group cancellation callback if defined, and if this was actually a group cancel. */
            if cancel_pending != 0 && (*obj).group_cancel_callback.is_some() {
                ((*obj).group_cancel_callback.unwrap())((*obj).userdata, userdata);
            }

            if (*obj).type_ != _threadpool_objtype_TP_OBJECT_TYPE_SIMPLE {
                tp_object_release(obj);
            }
        }

        (*obj).shutdown = 1;
        tp_object_release(obj);

        obj = next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseIoCompletion(io: *mut TP_IO) {
    let this = impl_from_TP_IO(io);
    let can_destroy: BOOL;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));
    (*this).u.io.shutting_down = 1;
    can_destroy = ((*this).u.io.pending_count == 0 && (*this).u.io.skipped_count == 0) as BOOL;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));

    if can_destroy != 0 {
        tp_object_prepare_shutdown(this);
        (*this).shutdown = 1;
        tp_object_release(this);
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpReleasePool(pool: *mut TP_POOL) {
    let this = impl_from_TP_POOL(pool);

    tp_threadpool_shutdown(this);
    tp_threadpool_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseTimer(timer: *mut TP_TIMER) {
    let this = impl_from_TP_TIMER(timer);

    tp_object_prepare_shutdown(this);
    (*this).shutdown = 1;
    tp_object_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseWait(wait: *mut TP_WAIT) {
    let this = impl_from_TP_WAIT(wait);

    tp_object_prepare_shutdown(this);
    (*this).shutdown = 1;
    tp_object_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseWork(work: *mut TP_WORK) {
    let this = impl_from_TP_WORK(work);

    tp_object_prepare_shutdown(this);
    (*this).shutdown = 1;
    tp_object_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpSetPoolMaxThreads(pool: *mut TP_POOL, maximum: DWORD) {
    let this = impl_from_TP_POOL(pool);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));
    (*this).max_workers = core::cmp::max(maximum, 1) as i32;
    (*this).min_workers = core::cmp::min((*this).min_workers, (*this).max_workers);
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpSetPoolMinThreads(pool: *mut TP_POOL, minimum: DWORD) -> BOOL {
    let this = impl_from_TP_POOL(pool);
    let mut status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    while ((*this).num_workers as DWORD) < minimum {
        status = tp_new_worker_thread(this);
        if status != STATUS_SUCCESS as NTSTATUS {
            break;
        }
    }

    if status == STATUS_SUCCESS as NTSTATUS {
        (*this).min_workers = minimum as i32;
        (*this).max_workers = if (*this).min_workers > (*this).max_workers {
            (*this).min_workers
        } else {
            (*this).max_workers
        };
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));
    (status == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn TpSetTimer(timer: *mut TP_TIMER, mut timeout: *mut LARGE_INTEGER, period: LONG, window_length: LONG) {
    let this: *mut threadpool_object = impl_from_TP_TIMER(timer);
    let mut other_timer: *mut threadpool_object;
    let mut submit_timer: BOOL = 0u8 as BOOL;
    let mut timestamp: ULONGLONG = 0;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));

    (*this).u.timer.timer_set = if timeout.is_null() { 0u8 as BOOL } else { 1u8 as BOOL };

    if !timeout.is_null() {
        timestamp = (*timeout).QuadPart as ULONGLONG;
        if (timestamp as i64) < 0 {
            let mut now: LARGE_INTEGER = core::mem::zeroed();
            NtQuerySystemTime(&mut now);
            timestamp = (now.QuadPart as ULONGLONG).wrapping_sub(timestamp);
        } else if timestamp == 0 {
            if period == 0 {
                timeout = core::ptr::null_mut();
            } else {
                let mut now: LARGE_INTEGER = core::mem::zeroed();
                NtQuerySystemTime(&mut now);
                timestamp = (now.QuadPart as ULONGLONG).wrapping_add((period as ULONGLONG).wrapping_mul(10000));
            }
            submit_timer = 1u8 as BOOL;
        }
    }

    if (*this).u.timer.timer_pending != 0 {
        list_remove(core::ptr::addr_of_mut!((*this).u.timer.timer_entry));
        (*this).u.timer.timer_pending = 0u8 as BOOL;
    }

    if !timeout.is_null() {
        (*this).u.timer.timeout = timestamp;
        (*this).u.timer.period = period;
        (*this).u.timer.window_length = window_length;

        other_timer = LIST_ENTRY(
            (*timerqueue.pending_timers.Flink).Flink as *mut _,
            0usize,
        ) as *mut threadpool_object;
        let mut entry = (timerqueue.pending_timers).Flink;
        loop {
            if entry == core::ptr::addr_of_mut!(timerqueue.pending_timers) {
                other_timer = (entry as usize as isize
                    - (core::mem::offset_of!(threadpool_object, u) as isize
                        + offset_of_timer_entry() as isize)) as *mut threadpool_object;
                break;
            }
            other_timer = (entry as usize as isize
                - (core::mem::offset_of!(threadpool_object, u) as isize
                    + offset_of_timer_entry() as isize)) as *mut threadpool_object;
            if (*this).u.timer.timeout < (*other_timer).u.timer.timeout {
                break;
            }
            entry = (*entry).Flink;
        }

        list_add_before(
            core::ptr::addr_of_mut!((*other_timer).u.timer.timer_entry),
            core::ptr::addr_of_mut!((*this).u.timer.timer_entry),
        );

        if list_head(core::ptr::addr_of_mut!(timerqueue.pending_timers))
            == core::ptr::addr_of_mut!((*this).u.timer.timer_entry)
        {
            RtlWakeAllConditionVariable(core::ptr::addr_of_mut!(timerqueue.update_event));
        }

        (*this).u.timer.timer_pending = 1u8 as BOOL;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));

    if submit_timer != 0 {
        tp_object_submit(this, 0u8 as BOOL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn TpSetWait(wait: *mut TP_WAIT, handle: HANDLE, timeout: *mut LARGE_INTEGER) {
    let this = impl_from_TP_WAIT(wait);
    let mut timestamp: ULONGLONG = MAXLONGLONG as ULONGLONG;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    (*this).u.wait.handle = handle;

    if !handle.is_null() || (*this).u.wait.wait_pending != 0 {
        let bucket = (*this).u.wait.bucket;
        list_remove(core::ptr::addr_of_mut!((*this).u.wait.wait_entry));

        if !handle.is_null() && !timeout.is_null() {
            timestamp = (*timeout).QuadPart as ULONGLONG;
            if (timestamp as LONGLONG) < 0 {
                let mut now: LARGE_INTEGER = core::mem::zeroed();
                NtQuerySystemTime(&mut now);
                timestamp = (now.QuadPart as ULONGLONG).wrapping_sub(timestamp);
            }
        }

        if !handle.is_null() {
            list_add_tail(core::ptr::addr_of_mut!((*bucket).waiting), core::ptr::addr_of_mut!((*this).u.wait.wait_entry));
            (*this).u.wait.wait_pending = 1;
            (*this).u.wait.timeout = timestamp;
        } else {
            list_add_tail(core::ptr::addr_of_mut!((*bucket).reserved), core::ptr::addr_of_mut!((*this).u.wait.wait_entry));
            (*this).u.wait.wait_pending = 0;
        }

        NtSetEvent((*bucket).update_event, core::ptr::null_mut());
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpSimpleTryPost(callback: PTP_SIMPLE_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as SIZE_T) as *mut threadpool_object;
    if object.is_null() {
        return STATUS_NO_MEMORY as NTSTATUS;
    }

    status = tp_threadpool_lock(core::ptr::addr_of_mut!(pool), environment);
    if status != 0 {
        RtlFreeHeap(GetProcessHeap(), 0, object as PVOID);
        return status;
    }

    (*object).type_ = TP_OBJECT_TYPE_SIMPLE;
    (*object).u.simple.callback = callback;
    tp_object_initialize(object, pool, userdata, environment);

    STATUS_SUCCESS as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn TpStartAsyncIoOperation(io: *mut TP_IO) {
    let this = impl_from_TP_IO(io);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));

    (*this).u.io.pending_count += 1;

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpWaitForIoCompletion(io: *mut TP_IO, cancel_pending: BOOL) {
    let this = impl_from_TP_IO(io);

    if cancel_pending != 0 {
        tp_object_cancel(this);
    }
    tp_object_wait(this, 0);
}

#[no_mangle]
pub unsafe extern "system" fn TpWaitForTimer(timer: *mut TP_TIMER, cancel_pending: BOOL) {
    let this = impl_from_TP_TIMER(timer);

    if cancel_pending != 0 {
        tp_object_cancel(this);
    }
    tp_object_wait(this, 0);
}

#[no_mangle]
pub unsafe extern "system" fn TpWaitForWait(wait: *mut TP_WAIT, cancel_pending: BOOL) {
    let this = impl_from_TP_WAIT(wait);

    if cancel_pending != 0 {
        tp_object_cancel(this);
    }
    tp_object_wait(this, 0);
}

#[no_mangle]
pub unsafe extern "system" fn TpWaitForWork(work: *mut TP_WORK, cancel_pending: BOOL) {
    let this = impl_from_TP_WORK(work);

    if cancel_pending != 0 {
        tp_object_cancel(this);
    }
    tp_object_wait(this, 0);
}

#[no_mangle]
pub unsafe extern "system" fn TpSetPoolStackInformation(pool: *mut TP_POOL, stack_info: *mut TP_POOL_STACK_INFORMATION) -> NTSTATUS {
    let this = impl_from_TP_POOL(pool);

    if stack_info.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));
    (*this).stack_info = *stack_info;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn TpQueryPoolStackInformation(
    pool: *mut TP_POOL,
    stack_info: *mut TP_POOL_STACK_INFORMATION,
) -> NTSTATUS {
    let this = impl_from_TP_POOL(pool);

    if stack_info.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));
    *stack_info = (*this).stack_info;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    STATUS_SUCCESS
}
unsafe fn rtl_wait_callback(instance: *mut TP_CALLBACK_INSTANCE, userdata: *mut core::ffi::c_void, wait: *mut TP_WAIT, result: TP_WAIT_RESULT) {
    let object = impl_from_TP_WAIT(wait);
    let callback = (*object).u.wait.rtl_callback.unwrap();
    callback(userdata, if result != STATUS_WAIT_0 as u32 { 1u8 } else { 0u8 });
}

#[no_mangle]
pub unsafe extern "system" fn RtlRegisterWait(out: *mut HANDLE, handle: HANDLE, callback: RTL_WAITORTIMERCALLBACKFUNC, context: *mut core::ffi::c_void, milliseconds: ULONG, mut flags: ULONG) -> NTSTATUS {
    let mut environment: TP_CALLBACK_ENVIRON = core::mem::zeroed();
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let status: NTSTATUS;
    let mut wait: *mut TP_WAIT = core::ptr::null_mut();

    memset(core::ptr::addr_of_mut!(environment) as *mut core::ffi::c_void, 0, core::mem::size_of::<TP_CALLBACK_ENVIRON>());
    environment.Version = 1;
    environment.u.s.set_LongFunction(if (flags & WT_EXECUTELONGFUNCTION) != 0 { 1 } else { 0 });
    environment.u.s.set_Persistent(if (flags & WT_EXECUTEINPERSISTENTTHREAD) != 0 { 1 } else { 0 });

    flags &= WT_EXECUTEONLYONCE | WT_EXECUTEINWAITTHREAD | WT_EXECUTEINIOTHREAD;
    status = tp_alloc_wait(core::ptr::addr_of_mut!(wait), Some(rtl_wait_callback), context, core::ptr::addr_of_mut!(environment), flags);
    if status != 0 {
        return status;
    }

    let object = impl_from_TP_WAIT(wait);
    (*object).u.wait.rtl_callback = callback;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    TpSetWait(object as *mut TP_WAIT, handle, get_nt_timeout(core::ptr::addr_of_mut!(timeout), milliseconds));

    *out = object as HANDLE;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    STATUS_SUCCESS as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeregisterWaitEx(handle: HANDLE, event: HANDLE) -> NTSTATUS {
    let object = handle as *mut _threadpool_object;
    let status: NTSTATUS;

    if object.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    TpSetWait(object as *mut TP_WAIT, core::ptr::null_mut(), core::ptr::null_mut());

    if event == INVALID_HANDLE_VALUE {
        TpWaitForWait(object as *mut TP_WAIT, 1);
    } else {
        (*object).completed_event = event;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*object).pool).cs));
    if (*object).num_pending_callbacks + (*object).num_running_callbacks
        + (*object).num_associated_callbacks != 0
    {
        status = STATUS_PENDING;
    } else {
        status = STATUS_SUCCESS;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*object).pool).cs));

    TpReleaseWait(object as *mut TP_WAIT);
    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeregisterWait(WaitHandle: HANDLE) -> NTSTATUS {
    RtlDeregisterWaitEx(WaitHandle, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeThreadPooling() {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(old_threadpool.threadpool_compl_cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
}
