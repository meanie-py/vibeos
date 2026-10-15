use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn MFAllocateWorkQueue(queue: *mut DWORD) -> HRESULT {
    return RtwqAllocateWorkQueue((RTWQ_STANDARD_WORKQUEUE) as RTWQ_WORKQUEUE_TYPE, queue);
}

#[no_mangle]
pub unsafe extern "system" fn MFPutWorkItem(
    queue: DWORD,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut hr: HRESULT;

    if {
        hr = RtwqCreateAsyncResult(
            core::ptr::null_mut::<IUnknown>(),
            callback as *mut IRtwqAsyncCallback,
            state,
            core::ptr::addr_of_mut!(result),
        );
        hr
    } < 0
    {
        return hr;
    }

    hr = RtwqPutWorkItem(queue, 0, result);

    IRtwqAsyncResult_Release(result);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFPutWorkItem2(
    queue: DWORD,
    priority: LONG,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!("%#lx, %ld, %p, %p.\n", queue, priority, callback, state);

    hr = RtwqCreateAsyncResult(
        core::ptr::null_mut::<IUnknown>(),
        callback as *mut IRtwqAsyncCallback,
        state,
        core::ptr::addr_of_mut!(result),
    );
    if hr < 0 {
        return hr;
    }

    hr = RtwqPutWorkItem(queue, priority, result);

    IRtwqAsyncResult_Release(result);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFPutWorkItemEx(queue: DWORD, result: *mut IMFAsyncResult) -> HRESULT {
    return RtwqPutWorkItem(queue, 0, result as *mut IRtwqAsyncResult);
}

#[no_mangle]
pub unsafe extern "system" fn MFPutWorkItemEx2(queue: DWORD, priority: LONG, result: *mut IMFAsyncResult) -> HRESULT {
    return RtwqPutWorkItem(queue, priority, result as *mut IRtwqAsyncResult);
}

#[no_mangle]
pub unsafe extern "system" fn MFScheduleWorkItem(
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
    timeout: INT64,
    key: *mut MFWORKITEM_KEY,
) -> HRESULT {
    let mut result: *mut IRtwqAsyncResult = core::ptr::null_mut::<IRtwqAsyncResult>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/queue.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"MFScheduleWorkItem\0" as *const u8 as *const core::ffi::c_char,
            104 as core::ffi::c_int,
            b"%p, %p, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            callback,
            state,
            wine_dbgstr_longlong(timeout as ULONGLONG),
            key,
        );
    }
    if ({
        hr = RtwqCreateAsyncResult(
            core::ptr::null_mut::<IUnknown>(),
            callback as *mut IRtwqAsyncCallback,
            state,
            core::ptr::addr_of_mut!(result),
        );
        hr
    }) < 0
    {
        return hr;
    }

    hr = RtwqScheduleWorkItem(result, timeout, key as *mut RTWQWORKITEM_KEY);

    IRtwqAsyncResult_Release(result);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFInvokeCallback(mut result: *mut IMFAsyncResult) -> HRESULT {
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mfplat/queue.c\0"
                as *const u8 as *const i8,
            b"MFInvokeCallback\0" as *const u8 as *const i8,
            121 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            result,
        );
    }
    return RtwqInvokeCallback(result as *mut IRtwqAsyncResult);
}

#[no_mangle]
pub unsafe extern "system" fn MFGetTimerPeriodicity(period: *mut DWORD) -> HRESULT {
    TRACE!("%p.\n", period);
    *period = 10;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFBeginRegisterWorkQueueWithMMCSS(
    queue: DWORD,
    usage_class: *const WCHAR,
    taskid: DWORD,
    callback: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    return RtwqBeginRegisterWorkQueueWithMMCSS(
        queue,
        usage_class,
        taskid,
        0,
        callback as *mut IRtwqAsyncCallback,
        state,
    );
}
