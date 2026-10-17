use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut session_list: *mut SessionInfo =
    ::core::ptr::null::<SessionInfo>() as *mut SessionInfo;

#[no_mangle]
pub static mut session_lock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

// #define NDEBUG  (flag-only macro with no value; no Rust constant emitted)
#[no_mangle]
pub unsafe extern "C" fn GetSession(
    device_type: DeviceType,
    device_id: UINT,
) -> *mut SessionInfo {
    let mut session_info: *mut SessionInfo;

    EnterCriticalSection(core::ptr::addr_of_mut!(session_lock));
    session_info = session_list;

    while !session_info.is_null() {
        if ((*session_info).device_type as u32 == device_type as u32)
            && ((*session_info).device_id == device_id)
        {
            LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));
            return session_info;
        }

        session_info = (*session_info).next as *mut SessionInfo;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn CreateSession(device_type: DeviceType, device_id: UINT, session_info: *mut *mut SessionInfo) -> MMRESULT {
    let heap: HANDLE = GetProcessHeap();

    ASSERT!(!session_info.is_null());

    EnterCriticalSection(core::ptr::addr_of_mut!(session_lock));

    /* Ensure we're not creating a duplicate session */

    if !GetSession(device_type, device_id).is_null() {
        DPRINT!("Already allocated session\n");
        LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));
        return MMSYSERR_ALLOCATED;
    }

    *session_info = HeapAlloc(
        heap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<SessionInfo>()) as SIZE_T,
    ) as *mut SessionInfo;

    if (*session_info).is_null() {
        DPRINT!("Failed to allocate mem for session info\n");
        LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));
        return MMSYSERR_NOMEM;
    }

    (**session_info).device_type = device_type;
    (**session_info).device_id = device_id;

    /* Add to the list */

    (**session_info).next = session_list;
    session_list = *session_info;

    LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn DestroySession(mut session: *mut SessionInfo) {
    let mut heap: HANDLE = GetProcessHeap();
    let mut session_node: *mut SessionInfo = core::ptr::null_mut::<SessionInfo>();
    let mut session_prev: *mut SessionInfo = core::ptr::null_mut::<SessionInfo>();

    EnterCriticalSection(core::ptr::addr_of_mut!(session_lock));

    session_node = session_list;
    session_prev = core::ptr::null_mut::<SessionInfo>();

    while !session_node.is_null() {
        if session_node == session {
            (*session_prev).next = (*session).next;
            break;
        } else {
            session_prev = session_node;
            session_node = (*session_node).next as *mut SessionInfo;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(session_lock));

    HeapFree(heap, 0, session as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn StartSessionThread(session_info: *mut SessionInfo) -> MMRESULT {
    let mut task: LPTASKCALLBACK;
    let mut result: MMRESULT;

    ASSERT!(!session_info.is_null());

    /* This is our "ready" event, sent when the thread is idle */

    (*session_info).thread.ready_event = CreateEvent(
        core::ptr::null_mut(),
        (FALSE as i32),
        (FALSE as i32),
        core::ptr::null_mut(),
    );

    if (*session_info).thread.ready_event.is_null() {
        DPRINT!("Couldn't create thread_ready event\n");
        return MMSYSERR_NOMEM;
    }

    /* This is our "go" event, sent when we want the thread to do something */

    (*session_info).thread.go_event = CreateEvent(
        core::ptr::null_mut(),
        (FALSE as i32),
        (FALSE as i32),
        core::ptr::null_mut(),
    );

    if (*session_info).thread.go_event.is_null() {
        DPRINT!("Couldn't create thread_go event\n");
        CloseHandle((*session_info).thread.ready_event);
        return MMSYSERR_NOMEM;
    }

    /* TODO - other kinds of devices need attention, too */
    task = if (*session_info).device_type as ::core::ffi::c_uint
        == WaveOutDevice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ::core::mem::transmute::<Option<unsafe extern "C" fn(LPVOID) -> DWORD>, LPTASKCALLBACK>(
            Some(WaveThread as unsafe extern "C" fn(LPVOID) -> DWORD),
        )
    } else {
        None
    };

    ASSERT!(task.is_some());

    /* Effectively, this is a beefed-up CreateThread */

    result = mmTaskCreate(
        task,
        core::ptr::addr_of_mut!((*session_info).thread.handle),
        (session_info as DWORD_PTR) as DWORD,
    );

    if result != MMSYSERR_NOERROR {
        DPRINT!("Task creation failed\n");
        CloseHandle((*session_info).thread.ready_event);
        CloseHandle((*session_info).thread.go_event);
        return result;
    }

    /* Wait for the thread to be ready before completing */

    WaitForSingleObject((*session_info).thread.ready_event, INFINITE);

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn CallSessionThread(session_info: *mut SessionInfo, function: ThreadFunction, thread_parameter: PVOID) -> MMRESULT {
    ASSERT!(session_info);

    (*session_info).thread.function = function;
    (*session_info).thread.parameter = thread_parameter;

    DPRINT!("Calling session thread\n");
    SetEvent((*session_info).thread.go_event);

    DPRINT!("Waiting for thread response\n");
    WaitForSingleObject((*session_info).thread.ready_event, INFINITE);

    return (*session_info).thread.result;
}
#[no_mangle]
pub unsafe extern "C" fn HandleBySessionThread(
    private_handle: DWORD_PTR,
    message: DWORD_PTR,
    parameter: DWORD_PTR,
) -> DWORD {
    return CallSessionThread(
        private_handle as *mut SessionInfo,
        message as ThreadFunction,
        parameter as PVOID,
    ) as DWORD;
}
