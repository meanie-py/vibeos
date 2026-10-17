use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{DWORD_PTR, FILE_COMPLETION_INFORMATION, _FILE_INFORMATION_CLASS_FileCompletionInformation as FileCompletionInformation, IMAGE_NT_HEADERS, _IO_STATUS_BLOCK, LARGE_INTEGER, LIST_ENTRY, NtClose, NtCreateEvent, NtDuplicateObject, NtCreateIoCompletion, NtQueryPerformanceCounter, NtQuerySystemTime, NtReleaseMutant, NtReleaseSemaphore, NtRemoveIoCompletion, NtSetEvent, NtSetInformationFile, NtSetInformationThread, NtSetIoCompletion, NtWaitForMultipleObjects, NtWaitForSingleObject, PHANDLE, PLARGE_INTEGER, PTHREAD_START_ROUTINE, RTL_CONDITION_VARIABLE, RTL_CRITICAL_SECTION, RtlAllocateHeap, RtlCreateUserThread, RtlDeleteCriticalSection, RtlEnterCriticalSection, RtlExitUserThread, RtlFreeHeap, RtlImageNtHeader, RtlInitUnicodeString, RtlInitializeConditionVariable, RtlInitializeCriticalSection, RtlLeaveCriticalSection, RtlNtStatusToDosError, RtlSleepConditionVariableCS, RtlWakeAllConditionVariable, RtlWakeConditionVariable, _EVENT_TYPE_SynchronizationEvent, THREAD_NAME_INFORMATION, _THREADINFOCLASS_ThreadNameInformation as ThreadNameInformation};
use core::ffi::c_void;
// crate substrate: win32 aliases, public TP_* types, WT_* flags,
// RtlGetProcessHeap / GetCurrent* helpers (NtCurrentTeb impl), and the
// wine intrusive-list port + LIST_ENTRY! macro.
use crate::types::*;
use crate::wine_list::*;

// DPRINT-style debug macros (ERR/FIXME/WARN/TRACE) → no-ops, mirroring
// the C `#define ERR!(...) DPRINT1(...)` etc. under NDEBUG. Args dropped.

// #define MAXIMUM_WAITQUEUE_OBJECTS (MAXIMUM_WAIT_OBJECTS - 1)
const MAXIMUM_WAITQUEUE_OBJECTS: LONG = MAXIMUM_WAIT_OBJECTS - 1;

// L"..." wide-string literal → a const-built, properly-aligned UTF-16
// array (ASCII only, which is all the thread-name literals use).
macro_rules! L {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        static W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        W.as_ptr()
    }};
}

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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct threadpool_object_u_simple {
    pub callback: PTP_SIMPLE_CALLBACK,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct threadpool_object_u_work {
    pub callback: PTP_WORK_CALLBACK,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct threadpool_object_u_wait {
    pub callback: PTP_WAIT_CALLBACK,
    pub signaled: LONG,
    pub bucket: *mut waitqueue_bucket,
    pub wait_pending: BOOL,
    pub wait_entry: list,
    pub timeout: ULONGLONG,
    pub handle: HANDLE,
    /// Our own duplicate of `handle`, which is what the wait thread waits on. A caller may
    /// close its handle while the wait is pending (rtworkq closes a waitable timer right after
    /// submitting it), and NtWaitForMultipleObjects fails the WHOLE bucket on one invalid handle:
    /// every other wait sharing the thread would stall. Wine's fix, ported.
    pub duped_handle: HANDLE,
    pub flags: DWORD,
    pub rtl_callback: RTL_WAITORTIMERCALLBACKFUNC,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
    pub simple: threadpool_object_u_simple,
    pub work: threadpool_object_u_work,
    pub timer: threadpool_object_u_timer,
    pub wait: threadpool_object_u_wait,
    pub io: threadpool_object_u_io,
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
    /// Bumped when TpSetWait points a wait at a different handle, so a signal the wait thread
    /// collected for the old handle is not delivered against the new one.
    pub update_serial: LONG,
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

// ---- file-scope global singletons (threadpool.c anonymous statics) ----
// const zero-init helpers (RTL_CRITICAL_SECTION = {0}; condvar = NULL).
const ZERO_CS: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
const ZERO_CV: RTL_CONDITION_VARIABLE = RTL_CONDITION_VARIABLE {
    Ptr: core::ptr::null_mut(),
};
const ZERO_LIST: list = list {
    next: core::ptr::null_mut(),
    prev: core::ptr::null_mut(),
};

#[repr(C)]
pub struct old_threadpool_t {
    pub compl_port: HANDLE,
    pub threadpool_compl_cs: RTL_CRITICAL_SECTION,
}
pub static mut old_threadpool: old_threadpool_t = old_threadpool_t {
    compl_port: core::ptr::null_mut(),
    threadpool_compl_cs: ZERO_CS,
};

#[repr(C)]
pub struct timerqueue_t {
    pub cs: RTL_CRITICAL_SECTION,
    pub objcount: LONG,
    pub thread_running: BOOL,
    pub pending_timers: list, // sorted by expiration time
    pub update_event: RTL_CONDITION_VARIABLE,
}
// pending_timers is LIST_INIT'd at runtime in RtlpInitializeThreadPooling
// (equivalent to the C static LIST_INIT; same process-init ordering as
// the RtlInitializeCriticalSection calls already there).
pub static mut timerqueue: timerqueue_t = timerqueue_t {
    cs: ZERO_CS,
    objcount: 0,
    thread_running: 0,
    pending_timers: ZERO_LIST,
    update_event: ZERO_CV,
};

#[repr(C)]
pub struct waitqueue_t {
    pub cs: RTL_CRITICAL_SECTION,
    pub num_buckets: LONG,
    pub buckets: list,
}
pub static mut waitqueue: waitqueue_t = waitqueue_t {
    cs: ZERO_CS,
    num_buckets: 0,
    buckets: ZERO_LIST,
};

#[repr(C)]
pub struct ioqueue_t {
    pub cs: RTL_CRITICAL_SECTION,
    pub objcount: LONG,
    pub thread_running: BOOL,
    pub port: HANDLE,
    pub update_event: RTL_CONDITION_VARIABLE,
}
pub static mut ioqueue: ioqueue_t = ioqueue_t {
    cs: ZERO_CS,
    objcount: 0,
    thread_running: 0,
    port: core::ptr::null_mut(),
    update_event: ZERO_CV,
};

// offset of `timer_entry` within the object's `u` union (the union's
// `timer` arm is at union offset 0, so this is the offset inside the
// timer struct). Used with offset_of!(threadpool_object, u) for the
// LIST_FOR_EACH_ENTRY container-of over timerqueue.pending_timers.
#[inline(always)]
fn offset_of_timer_entry() -> usize {
    core::mem::offset_of!(threadpool_object_u_timer, timer_entry)
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
    // threadpool.c: `#ifndef __REACTOS__ // This is impossible on non vista+`.
    if cfg!(not_reactos) {
        let mut info: THREAD_NAME_INFORMATION = core::mem::zeroed();
        RtlInitUnicodeString(core::ptr::addr_of_mut!(info.ThreadName), name);
        NtSetInformationThread(
            GetCurrentThread(),
            ThreadNameInformation,
            core::ptr::addr_of_mut!(info) as PVOID,
            core::mem::size_of::<THREAD_NAME_INFORMATION>() as ULONG,
        );
    }
}
unsafe extern "system" fn timerqueue_thread_proc(param: PVOID) -> ULONG {
    let _ = param;
    let mut timeout_lower: ULONGLONG;
    let mut timeout_upper: ULONGLONG;
    let mut new_timeout: ULONGLONG;
    let mut other_timer: *mut threadpool_object;
    let mut now: LARGE_INTEGER = core::mem::zeroed();
    let mut timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ptr: *mut list;

    set_thread_name(win32macros_rust::wide_bytes!(b"w\0i\0n\0e\0_\0t\0h\0r\0e\0a\0d\0p\0o\0o\0l\0_\0t\0i\0m\0e\0r\0q\0u\0e\0u\0e\0\0\0") as *const WCHAR);

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
        (*pool).stack_info.StackReserve as _,
        (*pool).stack_info.StackCommit as _,
        core::mem::transmute::<_, PTHREAD_START_ROUTINE>(threadpool_worker_proc as usize),
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
            core::mem::transmute::<_, PTHREAD_START_ROUTINE>(timerqueue_thread_proc as usize),
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
unsafe extern "system" fn waitqueue_thread_proc(param: PVOID) {
    const MAXIMUM_WAITQUEUE_OBJECTS: usize = (MAXIMUM_WAIT_OBJECTS - 1) as usize;

    let mut objects: [*mut threadpool_object; MAXIMUM_WAITQUEUE_OBJECTS] =
        [core::ptr::null_mut(); MAXIMUM_WAITQUEUE_OBJECTS];
    let mut handles: [HANDLE; MAXIMUM_WAITQUEUE_OBJECTS + 1] =
        [core::ptr::null_mut(); MAXIMUM_WAITQUEUE_OBJECTS + 1];
    let mut update_serials: [LONG; MAXIMUM_WAITQUEUE_OBJECTS] = [0; MAXIMUM_WAITQUEUE_OBJECTS];
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

        wait = LIST_ENTRY!(
            (*bucket).waiting.next,
            threadpool_object, u.wait.wait_entry
        );
        loop {
            let entry = core::ptr::addr_of_mut!((*bucket).waiting);
            let cur = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
            if cur as *const _ == entry as *const _ {
                break;
            }
            next = LIST_ENTRY!(
                (*wait).u.wait.wait_entry.next,
                threadpool_object, u.wait.wait_entry
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
                handles[num_handles as usize] = if (*wait).u.wait.duped_handle.is_null() {
                    (*wait).u.wait.handle
                } else {
                    (*wait).u.wait.duped_handle
                };
                update_serials[num_handles as usize] = (*wait).update_serial;
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
                if !(*wait).u.wait.bucket.is_null()
                    && (*wait).update_serial == update_serials[(status - STATUS_WAIT_0) as usize]
                {
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
            let mut other_bucket: *mut waitqueue_bucket = LIST_ENTRY!(
                waitqueue.buckets.next,
                waitqueue_bucket, bucket_entry
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

                    wait = LIST_ENTRY!(
                        (*bucket).reserved.next,
                        threadpool_object, u.wait.wait_entry
                    );
                    loop {
                        let e = core::ptr::addr_of_mut!((*bucket).reserved);
                        let c = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
                        if c as *const _ == e as *const _ {
                            break;
                        }
                        (*wait).u.wait.bucket = other_bucket;
                        wait = LIST_ENTRY!(
                            (*wait).u.wait.wait_entry.next,
                            threadpool_object, u.wait.wait_entry
                        );
                    }
                    list_move_tail(
                        core::ptr::addr_of_mut!((*other_bucket).reserved),
                        core::ptr::addr_of_mut!((*bucket).reserved),
                    );

                    wait = LIST_ENTRY!(
                        (*bucket).waiting.next,
                        threadpool_object, u.wait.wait_entry
                    );
                    loop {
                        let e = core::ptr::addr_of_mut!((*bucket).waiting);
                        let c = core::ptr::addr_of_mut!((*wait).u.wait.wait_entry);
                        if c as *const _ == e as *const _ {
                            break;
                        }
                        (*wait).u.wait.bucket = other_bucket;
                        wait = LIST_ENTRY!(
                            (*wait).u.wait.wait_entry.next,
                            threadpool_object, u.wait.wait_entry
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

                other_bucket = LIST_ENTRY!(
                    (*other_bucket).bucket_entry.next,
                    waitqueue_bucket, bucket_entry
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
    (*wait).u.wait.duped_handle = core::ptr::null_mut();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));

    status = 'out: {
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

        bucket = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<waitqueue_bucket>() as _) as *mut waitqueue_bucket;
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
unsafe extern "system" fn ioqueue_thread_proc(param: PVOID) -> ULONG {
    let _ = param;
    let mut completion: *mut io_completion;
    let mut io: *mut threadpool_object;
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut key: PVOID = core::ptr::null_mut();
    let mut value: PVOID = core::ptr::null_mut();
    let mut destroy: BOOL;
    let mut skip: BOOL;
    let mut status: NTSTATUS;

    set_thread_name(L!("wine_threadpool_ioqueue"));

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
            core::mem::transmute::<_, PTHREAD_START_ROUTINE>(ioqueue_thread_proc as usize),
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

    pool = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool>() as _) as *mut threadpool;
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
    (*pool).stack_info.StackReserve = (*nt).OptionalHeader.SizeOfStackReserve as usize;
    (*pool).stack_info.StackCommit = (*nt).OptionalHeader.SizeOfStackCommit as usize;

    TRACE!(b"allocated threadpool %p\n\0".as_ptr() as *const i8, pool);

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
    // C: static struct threadpool *default_threadpool = NULL;
    static mut default_threadpool: *mut threadpool = core::ptr::null_mut();
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
        core::mem::size_of::<threadpool_group>() as _,
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
    (*object).update_serial = 0;

    if !environment.is_null() {
        if (*environment).Version != 1 && (*environment).Version != 3 {
            FIXME!("unsupported environment version %lu\n", (*environment).Version);
        }

        (*object).group = impl_from_TP_CLEANUP_GROUP((*environment).CleanupGroup);
        (*object).group_cancel_callback = (*environment).CleanupGroupCancelCallback;
        (*object).finalization_callback = (*environment).FinalizationCallback;
        (*object).may_run_long = ((*environment).u.s.LongFunction() != 0) as BOOL;
        (*object).race_dll = (*environment).RaceDll;

        if !(*environment).ActivationContext.is_null() {
            FIXME!("activation context not supported yet\n");
        }

        if (*environment).u.s.Persistent() != 0 {
            FIXME!("persistent threads not supported yet\n");
        }
    }

    if !(*object).race_dll.is_null() {
        LdrAddRefDll(0, (*object).race_dll);
    }

    TRACE!(b"allocated object %p of type %u\n\0".as_ptr() as *const i8, object, (*object).type_);

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
    if (*object).type_ == TP_OBJECT_TYPE_IO
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
            NtSetIoCompletion(ioqueue.port, core::ptr::null_mut(), core::ptr::null_mut(), STATUS_SUCCESS, 0);
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
}

unsafe fn tp_object_prepare_shutdown(object: *mut threadpool_object) {
    if (*object).type_ == TP_OBJECT_TYPE_TIMER {
        tp_timerqueue_unlock(object);
    } else if (*object).type_ == TP_OBJECT_TYPE_WAIT {
        tp_waitqueue_unlock(object);
    } else if (*object).type_ == TP_OBJECT_TYPE_IO {
        tp_ioqueue_unlock(object);
    }
}

unsafe fn tp_wait_close_duped_handle(wait: *mut threadpool_object) {
    if !(*wait).u.wait.duped_handle.is_null() {
        NtClose((*wait).u.wait.duped_handle);
        (*wait).u.wait.duped_handle = core::ptr::null_mut();
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

    if (*object).type_ == TP_OBJECT_TYPE_WAIT {
        tp_wait_close_duped_handle(object);
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

    if (*object).type_ == TP_OBJECT_TYPE_WAIT {
        wait_result = if (*object).u.wait.signaled != 0 { WAIT_OBJECT_0 } else { WAIT_TIMEOUT };
        if wait_result == WAIT_OBJECT_0 {
            (*object).u.wait.signaled -= 1;
        }
    } else if (*object).type_ == TP_OBJECT_TYPE_IO {
        (*object).u.io.completion_count -= 1;
        completion = *(*object).u.io.completions.offset((*object).u.io.completion_count as isize);
    }

    (*object).num_associated_callbacks += 1;
    (*object).num_running_callbacks += 1;
    RtlLeaveCriticalSection(&mut (*pool).cs);
    if wait_thread != 0 {
        RtlLeaveCriticalSection(&raw mut waitqueue.cs);
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

    if (*object).type_ == TP_OBJECT_TYPE_SIMPLE {
        (*object).u.simple.callback.unwrap()(callback_instance, (*object).userdata);
    } else if (*object).type_ == TP_OBJECT_TYPE_WORK {
        (*object).u.work.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_WORK);
    } else if (*object).type_ == TP_OBJECT_TYPE_TIMER {
        (*object).u.timer.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_TIMER);
    } else if (*object).type_ == TP_OBJECT_TYPE_WAIT {
        (*object).u.wait.callback.unwrap()(callback_instance, (*object).userdata, object as *mut TP_WAIT, wait_result);
    } else if (*object).type_ == TP_OBJECT_TYPE_IO {
        (*object).u.io.callback.unwrap()(callback_instance, (*object).userdata,
            completion.cvalue as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(completion.iosb) as *mut core::ffi::c_void, object as *mut TP_IO);
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
        RtlEnterCriticalSection(&raw mut waitqueue.cs);
    }
    RtlEnterCriticalSection(&mut (*pool).cs);

    if (*object).type_ == TP_OBJECT_TYPE_SIMPLE {
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
unsafe extern "system" fn threadpool_worker_proc(param: PVOID) -> ULONG {
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
        core::mem::size_of::<threadpool_object>() as _,
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
        (8usize * core::mem::size_of::<io_completion>()) as _,
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

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as _) as *mut threadpool_object;
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

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as _) as *mut threadpool_object;
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
    let status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as _) as *mut threadpool_object;
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
    // LIST_FOR_EACH_ENTRY_SAFE: both cursors are container_of(entry, group_entry). The first
    // translation cast the list links straight to *mut threadpool_object, so it read each list
    // node as an object and its head check never matched: it bumped refcounts in whatever lay
    // past the node and walked off into it, on every call (an empty group included).
    let list_head_ptr = core::ptr::addr_of_mut!((*this).members);
    let mut object: *mut threadpool_object =
        LIST_ENTRY!((*list_head_ptr).next, threadpool_object, group_entry);
    loop {
        if core::ptr::addr_of_mut!((*object).group_entry) == list_head_ptr {
            break;
        }
        let next: *mut threadpool_object =
            LIST_ENTRY!((*object).group_entry.next, threadpool_object, group_entry);

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

            if (*obj).type_ != TP_OBJECT_TYPE_SIMPLE {
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

        // LIST_FOR_EACH_ENTRY( other_timer, &timerqueue.pending_timers,
        //                      struct threadpool_object, u.timer.timer_entry )
        let mut entry = timerqueue.pending_timers.next;
        loop {
            other_timer = (entry as usize as isize
                - (core::mem::offset_of!(threadpool_object, u) as isize
                    + offset_of_timer_entry() as isize)) as *mut threadpool_object;
            // stop when we've walked back to the list head sentinel
            if core::ptr::addr_of_mut!((*other_timer).u.timer.timer_entry)
                == core::ptr::addr_of_mut!(timerqueue.pending_timers)
            {
                break;
            }
            if (*this).u.timer.timeout < (*other_timer).u.timer.timeout {
                break;
            }
            entry = (*entry).next;
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

    let same_handle = (*this).u.wait.handle == handle;
    tp_wait_close_duped_handle(this);
    if !handle.is_null()
        && NtDuplicateObject(
            -1isize as HANDLE, // NtCurrentProcess()
            handle,
            -1isize as HANDLE,
            core::ptr::addr_of_mut!((*this).u.wait.duped_handle),
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        ) != 0
    {
        // Wait on the caller's handle as before; only closing it early is unprotected.
        (*this).u.wait.duped_handle = core::ptr::null_mut();
    }
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

        if !same_handle {
            (*this).update_serial += 1;
        }
        NtSetEvent((*bucket).update_event, core::ptr::null_mut());
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
}

#[no_mangle]
pub unsafe extern "system" fn TpSimpleTryPost(callback: PTP_SIMPLE_CALLBACK, userdata: PVOID, environment: *mut TP_CALLBACK_ENVIRON) -> NTSTATUS {
    let mut pool: *mut threadpool = core::ptr::null_mut();
    let status: NTSTATUS;

    let object = RtlAllocateHeap(GetProcessHeap(), 0, core::mem::size_of::<threadpool_object>() as _) as *mut threadpool_object;
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
#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeThreadPooling() {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(old_threadpool.threadpool_compl_cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(timerqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(waitqueue.cs));
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(ioqueue.cs));
    // The C file statically LIST_INIT's these (self-referential); do it
    // at runtime here instead (runs at process init before any Tp* use,
    // same ordering guarantee as the critical-section inits above).
    list_init(core::ptr::addr_of_mut!(timerqueue.pending_timers));
    list_init(core::ptr::addr_of_mut!(waitqueue.buckets));
}
