use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{DWORD_PTR, FILE_COMPLETION_INFORMATION, FileCompletionInformation, IMAGE_NT_HEADERS, _IO_STATUS_BLOCK, LARGE_INTEGER, LIST_ENTRY, NtClose, NtCreateEvent, NtCreateIoCompletion, NtCurrentTeb, NtQueryPerformanceCounter, NtQuerySystemTime, NtReleaseMutant, NtReleaseSemaphore, NtRemoveIoCompletion, NtSetEvent, NtSetInformationFile, NtSetInformationThread, NtSetIoCompletion, NtWaitForMultipleObjects, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER, PTHREAD_START_ROUTINE, RTL_CONDITION_VARIABLE, RTL_CRITICAL_SECTION, RTL_CRITICAL_SECTION_DEBUG, RtlAllocateHeap, RtlCreateTimer, RtlCreateUserThread, RtlDeleteCriticalSection, RtlDeleteTimer, RtlDeleteTimerQueueEx, RtlDeregisterWait, RtlDeregisterWaitEx, RtlEnterCriticalSection, RtlExitUserThread, RtlFreeHeap, RtlImageNtHeader, RtlInitUnicodeString, RtlInitializeConditionVariable, RtlInitializeCriticalSection, RtlLeaveCriticalSection, RtlNtStatusToDosError, RtlQueueWorkItem, RtlRegisterWait, RtlSetIoCompletionCallback, RtlSleepConditionVariableCS, RtlUpdateTimer, RtlWakeAllConditionVariable, RtlWakeConditionVariable, _EVENT_TYPE_SynchronizationEvent, THREAD_NAME_INFORMATION, ThreadNameInformation};
use core::ffi::c_void;

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
    let new_capacity: core::ffi::c_uint;
    let max_capacity: core::ffi::c_uint;
    let mut nc: core::ffi::c_uint;
    let new_elements: *mut core::ffi::c_void;

    if count <= *capacity {
        return 1;
    }

    max_capacity = ((!(0 as SIZE_T)) / (size as SIZE_T)) as core::ffi::c_uint;
    if count > max_capacity {
        return 0;
    }

    nc = if 4u32 > *capacity { 4u32 } else { *capacity };
    while nc < count && nc <= max_capacity / 2 {
        nc *= 2;
    }
    if nc < count {
        nc = max_capacity;
    }
    new_capacity = nc;

    new_elements = RtlReAllocateHeap(GetProcessHeap(), 0, *elements, (new_capacity as SIZE_T) * (size as SIZE_T));
    if new_elements.is_null() {
        return 0;
    }

    *elements = new_elements;
    *capacity = new_capacity;

    1
}

unsafe fn set_thread_name(name: *const WCHAR) {
    let mut info: THREAD_NAME_INFORMATION = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(info.ThreadName), name);
    NtSetInformationThread(
        GetCurrentThread(),
        ThreadNameInformation,
        core::ptr::addr_of_mut!(info) as *mut _,
        core::mem::size_of::<THREAD_NAME_INFORMATION>() as u32,
    );
}
unsafe fn process_rtl_work_item(instance: *mut TP_CALLBACK_INSTANCE, userdata: *mut core::ffi::c_void) {
    let item = userdata as *mut rtl_work_item;

    ((*item).function.unwrap())((*item).context);

    RtlFreeHeap(GetProcessHeap(), 0, item as PVOID);
}
unsafe fn iocp_poller(Arg: LPVOID) -> DWORD {
    let cport: HANDLE = Arg;

    loop {
        let mut callback: PRTL_OVERLAPPED_COMPLETION_ROUTINE = core::mem::zeroed();
        let mut overlapped: LPVOID = core::ptr::null_mut();
        let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

        let res = NtRemoveIoCompletion(
            cport,
            core::ptr::addr_of_mut!(callback) as PULONG_PTR,
            core::ptr::addr_of_mut!(overlapped) as PULONG_PTR,
            &mut iosb,
            core::ptr::null_mut(),
        );

        if res != 0 {
            // ERR("NtRemoveIoCompletion failed: 0x%lx\n", res);
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

unsafe fn get_nt_timeout( pTime: PLARGE_INTEGER, timeout: ULONG ) -> PLARGE_INTEGER {
    if timeout == INFINITE {
        return core::ptr::null_mut();
    }
    (*pTime).QuadPart = (timeout as ULONGLONG).wrapping_mul(-10000i64 as ULONGLONG) as i64;
    pTime
}

unsafe fn queue_remove_timer(t: *mut queue_timer) {
    let q: *mut timer_queue = (*t).q;

    list_remove(core::ptr::addr_of_mut!((*t).entry));
    if !(*t).event.is_null() {
        NtSetEvent((*t).event, core::ptr::null_mut());
    }
    RtlFreeHeap(GetProcessHeap(), 0, t as PVOID);

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

unsafe fn queue_current_time() -> ULONGLONG {
    let mut now: LARGE_INTEGER = core::mem::zeroed();
    let mut freq: LARGE_INTEGER = core::mem::zeroed();
    NtQueryPerformanceCounter(&mut now, &mut freq);
    ((now.QuadPart * 1000) / freq.QuadPart) as ULONGLONG
}

unsafe fn queue_add_timer(t: *mut queue_timer, time: ULONGLONG, set_event: BOOL) {
    let q: *mut timer_queue = (*t).q;
    let mut ptr: *mut list = core::ptr::addr_of_mut!((*q).timers);

    if time != EXPIRE_NEVER {
        ptr = (*ptr).next;
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

    if set_event != 0 && core::ptr::addr_of_mut!((*t).entry) == list_head(core::ptr::addr_of_mut!((*q).timers)) {
        NtSetEvent((*q).event, core::ptr::null_mut());
    }
}

unsafe fn queue_move_timer(t: *mut queue_timer, time: ULONGLONG, set_event: BOOL) {
    list_remove(core::ptr::addr_of_mut!((*t).entry));
    queue_add_timer(t, time, set_event);
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

#[no_mangle]
pub unsafe extern "system" fn RtlCreateTimerQueue(NewTimerQueue: PHANDLE) -> NTSTATUS {
    let status: NTSTATUS;
    let q = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<timer_queue>()) as *mut timer_queue;
    if q.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*q).cs));
    list_init(core::ptr::addr_of_mut!((*q).timers));
    (*q).quit = 0;
    (*q).magic = TIMER_QUEUE_MAGIC as u32;
    let status = NtCreateEvent(
        core::ptr::addr_of_mut!((*q).event),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    if status != STATUS_SUCCESS {
        RtlFreeHeap(GetProcessHeap(), 0, q as *mut _);
        return status;
    }
    let status = RtlCreateUserThread(
        GetCurrentProcess(),
        core::ptr::null_mut(),
        0u8,
        0,
        0,
        0,
        Some(timer_queue_thread_proc),
        q as *mut _,
        core::ptr::addr_of_mut!((*q).thread),
        core::ptr::null_mut(),
    );
    if status != STATUS_SUCCESS {
        NtClose((*q).event);
        RtlFreeHeap(GetProcessHeap(), 0, q as *mut _);
        return status;
    }

    *NewTimerQueue = q as HANDLE;
    STATUS_SUCCESS
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
        Some(threadpool_worker_proc),
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

unsafe fn tp_timerqueue_lock(timer: *mut threadpool_object) -> NTSTATUS {
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

unsafe fn tp_timerqueue_unlock(timer: *mut threadpool_object) {
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

    'out: {
        /* Try to assign to existing bucket if possible. */
        bucket = LIST_ENTRY!(waitqueue.buckets.Flink, waitqueue_bucket, bucket_entry);
        while core::ptr::addr_of_mut!((*bucket).bucket_entry) != core::ptr::addr_of_mut!(waitqueue.buckets) {
            if (*bucket).objcount < MAXIMUM_WAITQUEUE_OBJECTS && (*bucket).alertable == alertable {
                list_add_tail(core::ptr::addr_of_mut!((*bucket).reserved), core::ptr::addr_of_mut!((*wait).u.wait.wait_entry));
                (*wait).u.wait.bucket = bucket;
                (*bucket).objcount += 1;

                status = STATUS_SUCCESS;
                break 'out;
            }
            bucket = LIST_ENTRY!((*bucket).bucket_entry.Flink, waitqueue_bucket, bucket_entry);
        }

        /* Create a new bucket and corresponding worker thread. */
        bucket = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<waitqueue_bucket>() as SIZE_T) as *mut waitqueue_bucket;
        if bucket.is_null() {
            status = STATUS_NO_MEMORY;
            break 'out;
        }

        (*bucket).objcount = 0;
        (*bucket).alertable = alertable;
        list_init(core::ptr::addr_of_mut!((*bucket).reserved));
        list_init(core::ptr::addr_of_mut!((*bucket).waiting));

        status = NtCreateEvent(core::ptr::addr_of_mut!((*bucket).update_event), EVENT_ALL_ACCESS,
                               core::ptr::null_mut(), _EVENT_TYPE_SynchronizationEvent, 0);
        if status != 0 {
            RtlFreeHeap(GetProcessHeap(), 0, bucket as PVOID);
            break 'out;
        }

        status = RtlCreateUserThread(GetCurrentProcess(), core::ptr::null_mut(), 0, 0, 0, 0,
                                     core::mem::transmute::<_, PTHREAD_START_ROUTINE>(waitqueue_thread_proc as usize), bucket as PVOID, core::ptr::addr_of_mut!(thread), core::ptr::null_mut());
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
    }

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

    let pool: *mut threadpool =
        RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool>()) as *mut threadpool;
    if pool.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*pool).refcount = 1;
    (*pool).objcount = 0;
    (*pool).shutdown = 0;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    let count = (*pool).pools.len();
    let mut i: u32 = 0;
    while (i as usize) < count {
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
    STATUS_SUCCESS
}

unsafe fn tp_threadpool_shutdown(pool: *mut threadpool) {
    (*pool).shutdown = 1;
    RtlWakeAllConditionVariable(core::ptr::addr_of_mut!((*pool).update_event));
}
unsafe fn tp_threadpool_release( pool: *mut threadpool ) -> BOOL {
    if InterlockedDecrement( core::ptr::addr_of_mut!((*pool).refcount) ) != 0 {
        return 0u8 as BOOL;
    }

    RtlDeleteCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );

    RtlFreeHeap( GetProcessHeap(), 0, pool as *mut core::ffi::c_void );
    1u8 as BOOL
}

unsafe fn tp_threadpool_lock(out: *mut *mut threadpool, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if !environment.is_null() {
        pool = (*environment).Pool as *mut threadpool;
    }

    if pool.is_null() {
        if default_threadpool.is_null() {
            status = tp_threadpool_alloc(&mut pool);
            if status != STATUS_SUCCESS {
                return status;
            }

            if !InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(default_threadpool) as *mut *mut core::ffi::c_void,
                pool as *mut core::ffi::c_void,
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

    if status == STATUS_SUCCESS {
        InterlockedIncrement(core::ptr::addr_of_mut!((*pool).refcount));
        (*pool).objcount += 1;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*pool).cs));

    if status != STATUS_SUCCESS {
        return status;
    }

    *out = pool;
    STATUS_SUCCESS
}
unsafe fn tp_threadpool_unlock( pool: *mut threadpool ) {
    RtlEnterCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );
    (*pool).objcount -= 1;
    RtlLeaveCriticalSection( core::ptr::addr_of_mut!((*pool).cs) );
    tp_threadpool_release( pool );
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

unsafe fn tp_object_prio_queue(object: *mut threadpool_object) {
    (*(*object).pool).num_busy_workers += 1;
    list_add_tail(
        core::ptr::addr_of_mut!((*(*object).pool).pools[(*object).priority as usize]),
        core::ptr::addr_of_mut!((*object).pool_entry),
    );
}

unsafe fn object_is_finished(object: *mut threadpool_object, group: BOOL) -> BOOL {
    if (*object).num_pending_callbacks != 0 {
        return 0;
    }
    if (*object).type == TP_OBJECT_TYPE_IO && (*object).u.io.pending_count != 0 {
        return 0;
    }

    if group != 0 {
        ((*object).num_running_callbacks == 0) as BOOL
    } else {
        ((*object).num_associated_callbacks == 0) as BOOL
    }
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

unsafe fn threadpool_get_next_item(pool: *const threadpool) -> *mut list {
    let mut ptr: *mut list = core::ptr::null_mut();

    let count = (*pool).pools.len();
    for i in 0..count {
        ptr = list_head(core::ptr::addr_of!((*pool).pools[i]) as *const list);
        if !ptr.is_null() {
            break;
        }
    }

    ptr
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocCleanupGroup(out: *mut *mut TP_CLEANUP_GROUP) -> NTSTATUS {
    tp_group_alloc(out as *mut *mut threadpool_group)
}

#[no_mangle]
pub unsafe extern "system" fn TpAllocPool(out: *mut *mut TP_POOL, reserved: PVOID) -> NTSTATUS {
    if !reserved.is_null() {
    }

    tp_threadpool_alloc(out as *mut *mut threadpool)
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
pub unsafe extern "system" fn TpIsTimerSet(timer: *mut TP_TIMER) -> BOOL {
    let this = impl_from_TP_TIMER(timer);

    (*this).u.timer.timer_set
}

#[no_mangle]
pub unsafe extern "system" fn TpReleaseCleanupGroup(group: *mut TP_CLEANUP_GROUP) {
    let this = impl_from_TP_CLEANUP_GROUP(group);

    tp_group_shutdown(this);
    tp_group_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpReleasePool(pool: *mut TP_POOL) {
    let this = impl_from_TP_POOL(pool);

    tp_threadpool_shutdown(this);
    tp_threadpool_release(this);
}

#[no_mangle]
pub unsafe extern "system" fn TpSetPoolMaxThreads(pool: *mut TP_POOL, maximum: DWORD) {
    let this = impl_from_TP_POOL(pool);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));
    (*this).max_workers = core::cmp::max(maximum, 1) as core::ffi::c_int;
    (*this).min_workers = core::cmp::min((*this).min_workers, (*this).max_workers);
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*this).cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpSetPoolMinThreads(pool: *mut TP_POOL, minimum: DWORD) -> BOOL {
    let this: *mut threadpool = impl_from_TP_POOL(pool);
    let mut status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*this).cs));

    while ((*this).num_workers as DWORD) < minimum {
        status = tp_new_worker_thread(this);
        if status != STATUS_SUCCESS as NTSTATUS {
            break;
        }
    }

    if status == STATUS_SUCCESS as NTSTATUS {
        (*this).min_workers = minimum as core::ffi::c_int;
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
pub unsafe extern "system" fn TpStartAsyncIoOperation(io: *mut TP_IO) {
    let this = impl_from_TP_IO(io);

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));

    (*this).u.io.pending_count += 1;

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*(*this).pool).cs));
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
    if let Some(callback) = (*object).u.wait.rtl_callback {
        callback(userdata, if result != STATUS_WAIT_0 as u32 { 1u8 } else { 0u8 });
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeThreadPooling() {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(old_threadpool.threadpool_compl_cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
}
