use crate::evr::evr::evr;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut tracking_thread: tracking_thread = tracking_thread {
    hthread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    tid: 0,
    refcount: 0,
};

static mut tracking_thread_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

// #define COBJMACROS: no declaration emitted.
// WINE_DEFAULT_DEBUG_CHANNEL(evr): debug channel "evr"; diagnostics use the crate's DPRINT/WARN/ERR macros.

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct surface_buffer {
    pub IMFMediaBuffer_iface: IMFMediaBuffer,
    pub IMFGetService_iface: IMFGetService,
    pub refcount: LONG,

    pub surface: *mut IUnknown,
    pub length: ULONG,
}

#[allow(non_camel_case_types, non_upper_case_globals)]
pub const SAMPLE_PROP_HAS_DURATION: i32 = 1 << 0;
#[allow(non_camel_case_types, non_upper_case_globals)]
pub const SAMPLE_PROP_HAS_TIMESTAMP: i32 = 1 << 1;
#[allow(non_camel_case_types, non_upper_case_globals)]
pub const SAMPLE_PROP_HAS_DESIRED_PROPS: i32 = 1 << 2;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct video_sample {
    pub IMFSample_iface: IMFSample,
    pub IMFTrackedSample_iface: IMFTrackedSample,
    pub IMFDesiredSample_iface: IMFDesiredSample,
    pub refcount: LONG,

    pub sample: *mut IMFSample,

    pub tracked_result: *mut IMFAsyncResult,
    pub tracked_refcount: LONG,

    pub timestamp: LONGLONG,
    pub duration: LONGLONG,
    pub desired_timestamp: LONGLONG,
    pub desired_duration: LONGLONG,
    pub flags: u32,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct tracked_async_result {
    pub result: MFASYNCRESULT,
    pub refcount: LONG,
    pub object: *mut IUnknown,
    pub state: *mut IUnknown,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct tracking_thread {
    pub hthread: HANDLE,
    pub tid: DWORD,
    pub refcount: LONG,
}

#[allow(non_camel_case_types, non_upper_case_globals)]
pub const TRACKM_STOP: u32 = WM_USER as u32;
#[allow(non_camel_case_types, non_upper_case_globals)]
pub const TRACKM_INVOKE: u32 = WM_USER as u32 + 1;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct sample_allocator {
    pub IMFVideoSampleAllocator_iface: IMFVideoSampleAllocator,
    pub IMFVideoSampleAllocatorCallback_iface: IMFVideoSampleAllocatorCallback,
    pub tracking_callback: IMFAsyncCallback,
    pub refcount: LONG,

    pub callback: *mut IMFVideoSampleAllocatorNotify,
    pub device_manager: *mut IDirect3DDeviceManager9,
    pub free_sample_count: u32,
    pub free_samples: list,
    pub used_samples: list,
    pub lock_notify_release: BOOL,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct queued_sample {
    pub entry: list,
    pub sample: *mut IMFSample,
}

pub unsafe extern "C" fn debugstr_time(time: LONGLONG) -> *const ::core::ffi::c_char {
    let mut abstime: ULONGLONG = (if time >= 0 { time } else { -time }) as ULONGLONG;
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut buffer: [::core::ffi::c_char; 23] = [0 as ::core::ffi::c_char; 23];
    let mut rev: [::core::ffi::c_char; 23] = [0 as ::core::ffi::c_char; 23];

    while abstime != 0 || i <= 8 as ::core::ffi::c_uint {
        let fresh15 = i;
        i = i.wrapping_add(1);
        buffer[fresh15 as usize] = (('0' as i32 as ULONGLONG)
            .wrapping_add(abstime % 10 as ULONGLONG))
            as ::core::ffi::c_char;
        abstime = abstime / 10 as ULONGLONG;
        if i == 7 as ::core::ffi::c_uint {
            let fresh16 = i;
            i = i.wrapping_add(1);
            buffer[fresh16 as usize] = '.' as i32 as ::core::ffi::c_char;
        }
    }
    if time < 0 as LONGLONG {
        let fresh17 = i;
        i = i.wrapping_add(1);
        buffer[fresh17 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    loop {
        let fresh18 = i;
        i = i.wrapping_sub(1);
        if !(fresh18 != 0) {
            break;
        }
        let fresh19 = j;
        j = j.wrapping_add(1);
        rev[fresh19 as usize] = buffer[i as usize];
    }
    while rev[j.wrapping_sub(1 as ::core::ffi::c_uint) as usize] as ::core::ffi::c_int == '0' as i32
        && rev[j.wrapping_sub(2 as ::core::ffi::c_uint) as usize] as ::core::ffi::c_int
            != '.' as i32
    {
        j = j.wrapping_sub(1);
    }
    rev[j as usize] = 0 as ::core::ffi::c_char;

    return wine_dbg_sprintf(
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        core::ptr::addr_of_mut!(rev) as *mut ::core::ffi::c_char,
    );
}

unsafe extern "C" fn impl_from_IMFSample(mut iface: *mut IMFSample) -> *mut video_sample {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(video_sample, IMFSample_iface))
        as *mut video_sample;
}

unsafe extern "C" fn impl_from_IMFTrackedSample(iface: *mut IMFTrackedSample) -> *mut video_sample {
    return (iface as *mut u8).sub(core::mem::offset_of!(video_sample, IMFTrackedSample_iface))
        as *mut video_sample;
}

unsafe extern "C" fn impl_from_IMFDesiredSample(iface: *mut IMFDesiredSample) -> *mut video_sample {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(video_sample, IMFDesiredSample_iface)) as *mut video_sample;
}

unsafe extern "C" fn impl_from_IMFMediaBuffer(iface: *mut IMFMediaBuffer) -> *mut surface_buffer {
    return CONTAINING_RECORD!(iface, surface_buffer, IMFMediaBuffer_iface);
}

unsafe extern "C" fn impl_from_IMFGetService(iface: *mut IMFGetService) -> *mut surface_buffer {
    return CONTAINING_RECORD!(iface, surface_buffer, IMFGetService_iface);
}

unsafe extern "C" fn impl_from_IMFAsyncResult(iface: *mut IMFAsyncResult) -> *mut tracked_async_result {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(tracked_async_result, result.AsyncResult)) as *mut tracked_async_result;
}

unsafe extern "system" fn tracked_async_result_QueryInterface(iface: *mut IMFAsyncResult, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncResult)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncResult_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "system" fn tracked_async_result_AddRef(iface: *mut IMFAsyncResult) -> ULONG {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*result).refcount) as *mut i32) as ULONG;

    return refcount;
}

unsafe extern "system" fn tracked_async_result_Release(iface: *mut IMFAsyncResult) -> ULONG {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*result).refcount)) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*result).result.pCallback.is_null() {
            IMFAsyncCallback_Release((*result).result.pCallback);
        }
        if !(*result).object.is_null() {
            IUnknown_Release((*result).object);
        }
        if !(*result).state.is_null() {
            IUnknown_Release((*result).state);
        }
        free(result as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn tracked_async_result_GetState(iface: *mut IMFAsyncResult, state: *mut *mut IUnknown) -> HRESULT {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);

    if (*result).state.is_null() {
        return E_POINTER;
    }

    *state = (*result).state;
    IUnknown_AddRef(*state);

    return S_OK;
}

unsafe extern "system" fn tracked_async_result_GetStatus(iface: *mut IMFAsyncResult) -> HRESULT {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);

    return (*result).result.hrStatusResult;
}

unsafe extern "system" fn tracked_async_result_SetStatus(iface: *mut IMFAsyncResult, status: HRESULT) -> HRESULT {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);

    TRACE!("{:p}, {:#x}.\n", iface, status);

    (*result).result.hrStatusResult = status;

    return S_OK;
}

unsafe extern "system" fn tracked_async_result_GetObject(iface: *mut IMFAsyncResult, object: *mut *mut IUnknown) -> HRESULT {
    let result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);

    if (*result).object.is_null() {
        return E_POINTER;
    }

    *object = (*result).object;
    IUnknown_AddRef(*object);

    return S_OK;
}
unsafe extern "system" fn tracked_async_result_GetStateNoAddRef(iface: *mut IMFAsyncResult) -> *mut IUnknown {
    let mut result: *mut tracked_async_result = impl_from_IMFAsyncResult(iface);

    return (*result).state;
}

unsafe extern "C" fn create_async_result(
    mut object: *mut IUnknown,
    mut callback: *mut IMFAsyncCallback,
    mut state: *mut IUnknown,
    mut out: *mut *mut IMFAsyncResult,
) -> HRESULT {
    let mut result: *mut tracked_async_result = core::ptr::null_mut();
    result = calloc((1 as size_t) as _, (core::mem::size_of::<tracked_async_result>() as size_t) as _) as *mut tracked_async_result;
    if result.is_null() {
        return E_OUTOFMEMORY;
    }
    (*result).result.AsyncResult.lpVtbl = core::ptr::addr_of!(tracked_async_result_vtbl);
    (*result).refcount = 1 as LONG;
    (*result).object = object;
    if !(*result).object.is_null() {
        IUnknown_AddRef((*result).object);
    }
    (*result).result.pCallback = callback;
    if !(*result).result.pCallback.is_null() {
        IMFAsyncCallback_AddRef((*result).result.pCallback);
    }
    (*result).state = state;
    if !(*result).state.is_null() {
        IUnknown_AddRef((*result).state);
    }
    *out = core::ptr::addr_of_mut!((*result).result.AsyncResult);
    return S_OK;
}

unsafe extern "system" fn tracking_thread_proc(arg: *mut core::ffi::c_void) -> DWORD {
    let mut ready_event: HANDLE = arg as HANDLE;
    let mut stop_thread: BOOL = (FALSE as i32);
    let mut result: *mut IMFAsyncResult = core::ptr::null_mut::<IMFAsyncResult>();
    let mut data: *mut MFASYNCRESULT = core::ptr::null_mut::<MFASYNCRESULT>();
    let mut msg: MSG = MSG {
        hwnd: core::ptr::null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 },
    };

    PeekMessageW(
        &raw mut msg,
        core::ptr::null_mut(),
        WM_USER as UINT,
        WM_USER as UINT,
        PM_NOREMOVE as UINT,
    );

    SetEvent(ready_event);

    while stop_thread == 0 {
        MsgWaitForMultipleObjects(
            0 as DWORD,
            core::ptr::null::<HANDLE>(),
            (FALSE) as BOOL,
            INFINITE as DWORD,
            QS_POSTMESSAGE as DWORD,
        );

        while PeekMessageW(
            &raw mut msg,
            core::ptr::null_mut(),
            0 as UINT,
            0 as UINT,
            PM_REMOVE as UINT,
        ) != 0
        {
            match msg.message {
                TRACKM_INVOKE => {
                    result = msg.lParam as *mut IMFAsyncResult;
                    data = result as *mut MFASYNCRESULT;
                    if !(*data).pCallback.is_null() {
                        IMFAsyncCallback_Invoke((*data).pCallback, result);
                    }
                    IMFAsyncResult_Release(result);
                }
                TRACKM_STOP => {
                    stop_thread = (TRUE as i32);
                }
                _ => {}
            }
        }
    }

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"tracking_thread_proc\0" as *const u8 as *const core::ffi::c_char,
            327 as core::ffi::c_int,
            b"Tracking thread exiting.\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }

    return 0;
}

unsafe extern "C" fn video_sample_create_tracking_thread() {
    EnterCriticalSection(core::ptr::addr_of_mut!(tracking_thread_cs));

    tracking_thread.refcount += 1;
    if tracking_thread.refcount == 1 {
        let mut ready_event: HANDLE = core::ptr::null_mut();

        ready_event = CreateEventW(
            core::ptr::null_mut(),
            (FALSE) as BOOL,
            (FALSE) as BOOL,
            core::ptr::null(),
        );

        tracking_thread.hthread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(tracking_thread_proc),
            ready_event as PVOID,
            0,
            core::ptr::addr_of_mut!(tracking_thread.tid),
        );
        if tracking_thread.hthread.is_null() {
            WARN!("Failed to create sample tracking thread.\n");
            CloseHandle(ready_event);
            LeaveCriticalSection(core::ptr::addr_of_mut!(tracking_thread_cs));
            return;
        }

        WaitForSingleObject(ready_event, INFINITE);
        CloseHandle(ready_event);

        TRACE!(
            "Create tracking thread %#lx.\n",
            tracking_thread.tid
        );
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(tracking_thread_cs));
}

unsafe extern "C" fn video_sample_stop_tracking_thread() {
    EnterCriticalSection(&raw mut tracking_thread_cs);
    tracking_thread.refcount -= 1;
    if tracking_thread.refcount == 0 {
        PostThreadMessageW(
            tracking_thread.tid,
            TRACKM_STOP as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        CloseHandle(tracking_thread.hthread);
        memset(
            &raw mut tracking_thread as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tracking_thread>() as size_t,
        );
    }
    LeaveCriticalSection(&raw mut tracking_thread_cs);
}

unsafe extern "C" fn video_sample_tracking_thread_invoke(result: *mut IMFAsyncResult) {
    if tracking_thread.tid == 0 {
        WARN!("Sample tracking thread is not initialized.\n");
        return;
    }

    IMFAsyncResult_AddRef(result);
    PostThreadMessageW(tracking_thread.tid, TRACKM_INVOKE, 0, result as LPARAM);
}

unsafe extern "C" fn impl_from_IMFVideoSampleAllocator(
    iface: *mut IMFVideoSampleAllocator,
) -> *mut sample_allocator {
    return (iface as *mut u8)
        .wrapping_sub(core::mem::offset_of!(sample_allocator, IMFVideoSampleAllocator_iface))
        as *mut sample_allocator;
}

unsafe extern "C" fn impl_from_IMFVideoSampleAllocatorCallback(
    iface: *mut IMFVideoSampleAllocatorCallback,
) -> *mut sample_allocator {
    return (iface as *mut u8)
        .sub(core::mem::offset_of!(sample_allocator, IMFVideoSampleAllocatorCallback_iface))
        as *mut sample_allocator;
}

unsafe extern "C" fn impl_from_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut sample_allocator {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(sample_allocator, tracking_callback)) as *mut sample_allocator;
}

unsafe extern "system" fn sample_allocator_QueryInterface(iface: *mut IMFVideoSampleAllocator, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocator)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocatorCallback)) != 0 {
        *obj = core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocatorCallback_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn sample_allocator_AddRef(iface: *mut IMFVideoSampleAllocator) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*allocator).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"sample_allocator_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            454 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "C" fn sample_allocator_release_samples(allocator: *mut sample_allocator) {
    let mut iter: *mut queued_sample;
    let mut iter2: *mut queued_sample;
    iter = ((*allocator).free_samples.next as *mut u8)
        .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    iter2 = ((*iter).entry.next as *mut u8)
        .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).free_samples) {
        list_remove(core::ptr::addr_of_mut!((*iter).entry));
        IMFSample_Release((*iter).sample);
        free(iter as *mut core::ffi::c_void);
        iter = iter2;
        iter2 = ((*iter).entry.next as *mut u8)
            .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    }

    iter = ((*allocator).used_samples.next as *mut u8)
        .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    iter2 = ((*iter).entry.next as *mut u8)
        .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).used_samples) {
        list_remove(core::ptr::addr_of_mut!((*iter).entry));
        free(iter as *mut core::ffi::c_void);
        iter = iter2;
        iter2 = ((*iter).entry.next as *mut u8)
            .sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    }
}

unsafe extern "system" fn sample_allocator_Release(iface: *mut IMFVideoSampleAllocator) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*allocator).refcount) as *mut i32) as ULONG;

    if refcount == 0 {
        if !(*allocator).callback.is_null() {
            IMFVideoSampleAllocatorNotify_Release((*allocator).callback);
        }
        if !(*allocator).device_manager.is_null() {
            IDirect3DDeviceManager9_Release((*allocator).device_manager);
        }
        sample_allocator_release_samples(allocator);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
        free(allocator as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn sample_allocator_SetDirectXManager(iface: *mut IMFVideoSampleAllocator, manager: *mut IUnknown) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);
    let mut device_manager: *mut IDirect3DDeviceManager9 = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !manager.is_null() && {
        hr = IUnknown_QueryInterface(
            manager,
            core::ptr::addr_of!(IID_IDirect3DDeviceManager9),
            core::ptr::addr_of_mut!(device_manager) as *mut *mut core::ffi::c_void,
        );
        hr < 0
    } {
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    if !(*allocator).device_manager.is_null() {
        IDirect3DDeviceManager9_Release((*allocator).device_manager);
    }
    (*allocator).device_manager = device_manager;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "system" fn sample_allocator_UninitializeSampleAllocator(iface: *mut IMFVideoSampleAllocator) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    sample_allocator_release_samples(allocator);
    (*allocator).free_sample_count = 0;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "C" fn sample_allocator_create_samples(allocator: *mut sample_allocator, sample_count: ::core::ffi::c_uint, media_type: *mut IMFMediaType) -> HRESULT {
    let mut service: *mut IDirectXVideoProcessorService = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;
    let mut width: ::core::ffi::c_uint = 0;
    let mut height: ::core::ffi::c_uint = 0;
    let mut surface: *mut IDirect3DSurface9 = core::ptr::null_mut();
    let mut hdevice: HANDLE = core::ptr::null_mut();
    let mut major: GUID = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();
    let mut frame_size: UINT64 = 0;
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut format: D3DFORMAT = 0;
    let mut hr: HRESULT = 0;

    if IMFMediaType_GetMajorType(media_type, &raw mut major) < 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    if !IsEqualGUID!(&raw mut major, &raw const MFMediaType_Video) {
        return MF_E_INVALIDMEDIATYPE;
    }

    if IMFMediaType_GetUINT64(media_type, &raw const MF_MT_FRAME_SIZE, &raw mut frame_size) < 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    if IMFMediaType_GetGUID(media_type, &raw const MF_MT_SUBTYPE, &raw mut subtype) < 0 {
        return MF_E_INVALIDMEDIATYPE;
    }

    format = subtype.Data1 as D3DFORMAT;
    height = frame_size as ::core::ffi::c_uint;
    width = (frame_size >> 32) as ::core::ffi::c_uint;

    if !(*allocator).device_manager.is_null() {
        hr = IDirect3DDeviceManager9_OpenDeviceHandle((*allocator).device_manager, &raw mut hdevice);
        if hr >= 0 {
            hr = IDirect3DDeviceManager9_GetVideoService(
                (*allocator).device_manager,
                hdevice,
                &raw const IID_IDirectXVideoProcessorService,
                &raw mut service as *mut *mut ::core::ffi::c_void,
            );
        }

        if hr < 0 {
            WARN!("Failed to get processor service, %#lx.\n", hr);
            return hr;
        }
    }

    sample_allocator_release_samples(allocator);

    i = 0 as ::core::ffi::c_uint;
    while i < sample_count {
        let mut queued_sample: *mut queued_sample = core::ptr::null_mut();
        let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();

        hr = MFCreateVideoSampleFromSurface(core::ptr::null_mut(), &raw mut sample);
        if hr >= 0 {
            if !service.is_null() {
                hr = IDirectXVideoProcessorService_CreateSurface(
                    service,
                    width as UINT,
                    height as UINT,
                    0 as UINT,
                    format,
                    D3DPOOL_DEFAULT as D3DPOOL,
                    0 as DWORD,
                    DXVA2_VideoProcessorRenderTarget as DWORD,
                    &raw mut surface,
                    core::ptr::null_mut(),
                );
                if hr >= 0 {
                    hr = MFCreateDXSurfaceBuffer(
                        &raw const IID_IDirect3DSurface9,
                        surface as *mut IUnknown,
                        (FALSE) as BOOL,
                        &raw mut buffer,
                    );
                    IDirect3DSurface9_Release(surface);
                }
            } else {
                hr = MFCreate2DMediaBuffer(
                    width as DWORD,
                    height as DWORD,
                    format as DWORD,
                    (FALSE) as BOOL,
                    &raw mut buffer,
                );
            }

            if hr >= 0 {
                hr = IMFSample_AddBuffer(sample, buffer);
                IMFMediaBuffer_Release(buffer);
            }
        }

        if hr < 0 {
            WARN!("Unable to allocate %u samples.\n", sample_count);
            sample_allocator_release_samples(allocator);
            break;
        }

        queued_sample = malloc(core::mem::size_of::<queued_sample>()) as *mut queued_sample;
        (*queued_sample).sample = sample;
        list_add_tail(&raw mut (*allocator).free_samples, &raw mut (*queued_sample).entry);
        (*allocator).free_sample_count += 1;

        i += 1;
    }

    if !service.is_null() {
        IDirectXVideoProcessorService_Release(service);
    }

    if !(*allocator).device_manager.is_null() {
        IDirect3DDeviceManager9_CloseDeviceHandle((*allocator).device_manager, hdevice);
    }

    return hr;
}

unsafe extern "system" fn sample_allocator_InitializeSampleAllocator(
    iface: *mut IMFVideoSampleAllocator,
    mut sample_count: DWORD,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);
    let mut hr: HRESULT;

    if sample_count == 0 {
        sample_count = 1 as DWORD;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    hr = sample_allocator_create_samples(allocator, sample_count, media_type);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return hr;
}

unsafe extern "system" fn sample_allocator_AllocateSample(iface: *mut IMFVideoSampleAllocator, out: *mut *mut IMFSample) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocator(iface);
    let mut tracked_sample: *mut IMFTrackedSample = core::ptr::null_mut();
    let mut sample: *mut IMFSample = core::ptr::null_mut();
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    if list_empty(core::ptr::addr_of_mut!((*allocator).free_samples)) != 0
        && list_empty(core::ptr::addr_of_mut!((*allocator).used_samples)) != 0
    {
        hr = MF_E_NOT_INITIALIZED;
    } else if list_empty(core::ptr::addr_of_mut!((*allocator).free_samples)) != 0 {
        hr = MF_E_SAMPLEALLOCATOR_EMPTY;
    } else {
        let head: *mut list = list_head(core::ptr::addr_of_mut!((*allocator).free_samples));

        sample = (*((head as *mut u8).sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample)).sample;

        hr = IMFSample_QueryInterface(
            sample,
            core::ptr::addr_of!(IID_IMFTrackedSample),
            core::ptr::addr_of_mut!(tracked_sample) as *mut *mut core::ffi::c_void,
        );
        if hr >= 0 {
            hr = IMFTrackedSample_SetAllocator(
                tracked_sample,
                core::ptr::addr_of_mut!((*allocator).tracking_callback),
                core::ptr::null_mut(),
            );
            IMFTrackedSample_Release(tracked_sample);
        }

        if hr >= 0 {
            list_remove(head);
            list_add_tail(core::ptr::addr_of_mut!((*allocator).used_samples), head);
            (*allocator).free_sample_count = (*allocator).free_sample_count.wrapping_sub(1);

            /* Reference counter is not increased when sample is returned, so next release could trigger
               tracking condition. This is balanced by incremented reference counter when sample is returned
               back to the free list. */
            *out = sample;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return hr;
}

unsafe extern "system" fn sample_allocator_callback_QueryInterface(
    iface: *mut IMFVideoSampleAllocatorCallback,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocator_QueryInterface(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface), riid, obj);
}

unsafe extern "system" fn sample_allocator_callback_AddRef(iface: *mut IMFVideoSampleAllocatorCallback) -> ULONG {
    let mut allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocator_AddRef(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface));
}

unsafe extern "system" fn sample_allocator_callback_Release(iface: *mut IMFVideoSampleAllocatorCallback) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);
    return IMFVideoSampleAllocator_Release(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface));
}

unsafe extern "system" fn sample_allocator_callback_SetCallback(
    iface: *mut IMFVideoSampleAllocatorCallback,
    callback: *mut IMFVideoSampleAllocatorNotify,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);

    TRACE!("%p, %p.\n", iface, callback);

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
    if !(*allocator).callback.is_null() {
        IMFVideoSampleAllocatorNotify_Release((*allocator).callback);
    }
    (*allocator).callback = callback;
    if !(*allocator).callback.is_null() {
        IMFVideoSampleAllocatorNotify_AddRef((*allocator).callback);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "system" fn sample_allocator_callback_GetFreeSampleCount(
    iface: *mut IMFVideoSampleAllocatorCallback,
    count: *mut LONG,
) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFVideoSampleAllocatorCallback(iface);

    TRACE!("%p, %p.\n", iface, count);

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));
    if !count.is_null() {
        *count = (*allocator).free_sample_count as LONG;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    return S_OK;
}

unsafe extern "system" fn sample_allocator_tracking_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn sample_allocator_tracking_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    return IMFVideoSampleAllocator_AddRef(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface));
}

unsafe extern "system" fn sample_allocator_tracking_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    return IMFVideoSampleAllocator_Release(core::ptr::addr_of_mut!((*allocator).IMFVideoSampleAllocator_iface));
}
unsafe extern "system" fn sample_allocator_tracking_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn sample_allocator_tracking_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let allocator: *mut sample_allocator = impl_from_IMFAsyncCallback(iface);
    let mut iter: *mut queued_sample = core::ptr::null_mut::<queued_sample>();
    let mut object: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut sample: *mut IMFSample = core::ptr::null_mut::<IMFSample>();
    let mut callback: *mut IMFVideoSampleAllocatorNotify = core::ptr::null_mut::<IMFVideoSampleAllocatorNotify>();
    let mut hr: HRESULT;

    if (IMFAsyncResult_GetObject(result, core::ptr::addr_of_mut!(object)) as HRESULT) < 0 {
        return E_UNEXPECTED;
    }

    hr = IUnknown_QueryInterface(object, core::ptr::addr_of!(IID_IMFSample), core::ptr::addr_of_mut!(sample) as *mut *mut core::ffi::c_void);
    IUnknown_Release(object);
    if hr < 0 {
        return E_UNEXPECTED;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    // LIST_FOR_EACH_ENTRY(iter, &allocator->used_samples, struct queued_sample, entry)
    iter = ((*allocator).used_samples.next as *mut u8).sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
    while core::ptr::addr_of_mut!((*iter).entry) != core::ptr::addr_of_mut!((*allocator).used_samples) {
        if sample == (*iter).sample {
            list_remove(core::ptr::addr_of_mut!((*iter).entry));
            list_add_tail(core::ptr::addr_of_mut!((*allocator).free_samples), core::ptr::addr_of_mut!((*iter).entry));
            IMFSample_AddRef((*iter).sample);
            (*allocator).free_sample_count = (*allocator).free_sample_count.wrapping_add(1);
            break;
        } else {
            iter = ((*iter).entry.next as *mut u8).sub(core::mem::offset_of!(queued_sample, entry)) as *mut queued_sample;
        }
    }

    IMFSample_Release(sample);

    if !(*allocator).callback.is_null() {
        if (*allocator).lock_notify_release != 0 {
            IMFVideoSampleAllocatorNotify_NotifyRelease((*allocator).callback);
        } else {
            callback = (*allocator).callback;
            IMFVideoSampleAllocatorNotify_AddRef(callback);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*allocator).cs));

    if !callback.is_null() {
        IMFVideoSampleAllocatorNotify_NotifyRelease(callback);
        IMFVideoSampleAllocatorNotify_Release(callback);
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn create_video_sample_allocator(mut lock_notify_release: BOOL, riid: *const IID, mut obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut object: *mut sample_allocator = ::core::ptr::null_mut::<sample_allocator>();
    let mut hr: HRESULT = 0;
    object = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<sample_allocator>() as size_t) as _,
    ) as *mut sample_allocator;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFVideoSampleAllocator_iface.lpVtbl = &raw const sample_allocator_vtbl;
    (*object).IMFVideoSampleAllocatorCallback_iface.lpVtbl =
        &raw const sample_allocator_callback_vtbl;
    (*object).tracking_callback.lpVtbl = &raw const sample_allocator_tracking_callback_vtbl;
    (*object).refcount = 1 as LONG;
    list_init(&raw mut (*object).used_samples);
    list_init(&raw mut (*object).free_samples);
    (*object).lock_notify_release = lock_notify_release;
    InitializeCriticalSection(&raw mut (*object).cs);

    hr = IMFVideoSampleAllocator_QueryInterface(
        &raw mut (*object).IMFVideoSampleAllocator_iface,
        riid,
        obj,
    );
    IMFVideoSampleAllocator_Release(&raw mut (*object).IMFVideoSampleAllocator_iface);

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoSampleAllocator(riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    return create_video_sample_allocator((TRUE) as BOOL, riid, obj);
}

unsafe extern "system" fn video_sample_QueryInterface(iface: *mut IMFSample, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSample)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*sample).IMFSample_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTrackedSample)) != 0
    {
        *out = core::ptr::addr_of_mut!((*sample).IMFTrackedSample_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFDesiredSample)) != 0
    {
        *out = core::ptr::addr_of_mut!((*sample).IMFDesiredSample_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn video_sample_AddRef(iface: *mut IMFSample) -> ULONG {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*sample).refcount) as *mut i32) as ULONG;

    return refcount;
}

unsafe extern "system" fn video_sample_Release(iface: *mut IMFSample) -> ULONG {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*sample).refcount) as *mut i32) as ULONG;
    let mut tracked_result: *mut IMFAsyncResult = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    if !(*sample).tracked_result.is_null() && (*sample).tracked_refcount as ULONG == refcount {
        tracked_result = (*sample).tracked_result;
        video_sample_tracking_thread_invoke(tracked_result);
        (*sample).tracked_result = core::ptr::null_mut();
        (*sample).tracked_refcount = 0 as LONG;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));

    if !tracked_result.is_null() {
        IMFAsyncResult_Release(tracked_result);
    }

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        video_sample_stop_tracking_thread();
        if !(*sample).sample.is_null() {
            IMFSample_Release((*sample).sample);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
        free(sample as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn video_sample_GetItem(
    iface: *mut IMFSample,
    key: REFGUID,
    value: *mut PROPVARIANT,
) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_GetItem\0".as_ptr() as *const i8,
            979i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
        );
    }

    return IMFSample_GetItem((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_GetItemType(iface: *mut IMFSample, key: REFGUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetItemType((*sample).sample, key, type_);
}

unsafe extern "system" fn video_sample_CompareItem(iface: *mut IMFSample, key: *const GUID, value: *const PROPVARIANT, result: *mut BOOL) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_CompareItem\0".as_ptr() as *const i8,
            997 as i32,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
            result,
        );
    }
    return IMFSample_CompareItem((*sample).sample, key, value, result);
}

unsafe extern "system" fn video_sample_Compare(
    iface: *mut IMFSample,
    theirs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_Compare\0".as_ptr() as *const i8,
            1007i32,
            b"%p, %p, %d, %p.\n\0".as_ptr() as *const i8,
            iface,
            theirs,
            type_0 as u32,
            result,
        );
    }
    return IMFSample_Compare((*sample).sample, theirs, type_0, result);
}

unsafe extern "system" fn video_sample_GetUINT32(iface: *mut IMFSample, key: *const GUID, value: *mut UINT32) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);

    return IMFSample_GetUINT32((*sample).sample, key, value);
}
unsafe extern "system" fn video_sample_GetUINT64(iface: *mut IMFSample, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetUINT64((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_GetDouble(iface: *mut IMFSample, key: *const GUID, value: *mut f64) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);
    return IMFSample_GetDouble((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_GetGUID(iface: *mut IMFSample, key: REFGUID, value: *mut GUID) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_GetGUID\0".as_ptr() as *const i8,
            1043i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }

    return IMFSample_GetGUID((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_GetStringLength(
    mut iface: *mut IMFSample,
    key: *const GUID,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const i8,
            b"video_sample_GetStringLength\0" as *const u8 as *const i8,
            1052 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            length,
        );
    }
    return IMFSample_GetStringLength((*sample).sample, key, length);
}

unsafe extern "system" fn video_sample_GetString(iface: *mut IMFSample, key: REFGUID, value: *mut WCHAR, size: UINT32, length: *mut UINT32) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetString((*sample).sample, key, value, size, length);
}

unsafe extern "system" fn video_sample_GetAllocatedString(
    iface: *mut IMFSample,
    key: *const GUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_GetAllocatedString\0".as_ptr() as *const i8,
            1070 as i32,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            length,
        );
    }
    return IMFSample_GetAllocatedString((*sample).sample, key, value as *mut LPWSTR, length);
}
unsafe extern "system" fn video_sample_GetBlobSize(iface: *mut IMFSample, key: *const GUID, size: *mut UINT32) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), size);
    return IMFSample_GetBlobSize((*sample).sample, key, size);
}

unsafe extern "system" fn video_sample_GetBlob(
    iface: *mut IMFSample,
    key: *const GUID,
    buf: *mut UINT8,
    bufsize: UINT32,
    blobsize: *mut UINT32,
) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const i8,
            b"video_sample_GetBlob\0" as *const u8 as *const i8,
            1088 as i32,
            b"%p, %s, %p, %u, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key),
            buf,
            bufsize,
            blobsize,
        );
    }

    return IMFSample_GetBlob((*sample).sample, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn video_sample_GetAllocatedBlob(iface: *mut IMFSample, key: REFGUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_GetAllocatedBlob\0".as_ptr() as *const i8,
            1097 as i32,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }
    return IMFSample_GetAllocatedBlob((*sample).sample, key, buf, size);
}

unsafe extern "system" fn video_sample_GetUnknown(iface: *mut IMFSample, key: *const GUID, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetUnknown((*sample).sample, key, riid, out);
}

unsafe extern "system" fn video_sample_SetItem(iface: *mut IMFSample, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"video_sample_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
            1115 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFSample_SetItem((*sample).sample, key, value);
}
unsafe extern "system" fn video_sample_DeleteItem(
    mut iface: *mut IMFSample,
    key: *const GUID,
) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr()
                as *const i8,
            b"video_sample_DeleteItem\0".as_ptr() as *const i8,
            1124i32,
            b"%p, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
        );
    }
    return IMFSample_DeleteItem((*sample).sample, key);
}

unsafe extern "system" fn video_sample_DeleteAllItems(mut iface: *mut IMFSample) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const i8,
            b"video_sample_DeleteAllItems\0" as *const u8 as *const i8,
            1133 as i32,
            b"%p.\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return IMFSample_DeleteAllItems((*sample).sample);
}

unsafe extern "system" fn video_sample_SetUINT32(iface: *mut IMFSample, key: *const GUID, value: UINT32) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetUINT32((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_SetUINT64(iface: *mut IMFSample, key: *const GUID, value: UINT64) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetUINT64((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_SetDouble(iface: *mut IMFSample, key: *const GUID, value: f64) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0" as *const u8 as *const i8,
            b"video_sample_SetDouble\0" as *const u8 as *const i8,
            1160i32,
            b"%p, %s, %f.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
        );
    }
    return IMFSample_SetDouble((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_SetGUID(iface: *mut IMFSample, key: *const GUID, value: *const GUID) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetGUID((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_SetString(
    iface: *mut IMFSample,
    key: *const GUID,
    value: *const u16,
) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0".as_ptr() as *const i8,
            b"video_sample_SetString\0".as_ptr() as *const i8,
            1178 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_w(value),
        );
    }
    return IMFSample_SetString((*sample).sample, key, value);
}

unsafe extern "system" fn video_sample_SetBlob(iface: *mut IMFSample, key: REFGUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetBlob((*sample).sample, key, buf, size);
}
unsafe extern "system" fn video_sample_SetUnknown(iface: *mut IMFSample, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetUnknown((*sample).sample, key, unknown);
}

unsafe extern "system" fn video_sample_LockStore(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_LockStore((*sample).sample);
}
unsafe extern "system" fn video_sample_UnlockStore(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    TRACE!("%p.\n", iface);

    return IMFSample_UnlockStore((*sample).sample);
}

unsafe extern "system" fn video_sample_GetCount(iface: *mut IMFSample, count: *mut UINT32) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetCount((*sample).sample, count);
}

unsafe extern "system" fn video_sample_GetItemByIndex(iface: *mut IMFSample, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return IMFSample_GetItemByIndex((*sample).sample, index, key, value);
}

unsafe extern "system" fn video_sample_CopyAllItems(iface: *mut IMFSample, dest: *mut IMFAttributes) -> HRESULT {
    // TRACE("%p, %p.\n", iface, dest);
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_CopyAllItems((*sample).sample, dest);
}

unsafe extern "system" fn video_sample_GetSampleFlags(iface: *mut IMFSample, flags: *mut DWORD) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetSampleFlags((*sample).sample, flags);
}

unsafe extern "system" fn video_sample_SetSampleFlags(iface: *mut IMFSample, flags: DWORD) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_SetSampleFlags((*sample).sample, flags);
}

unsafe extern "system" fn video_sample_GetSampleTime(iface: *mut IMFSample, timestamp: *mut LONGLONG) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, timestamp);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    if ((*sample).flags & (SAMPLE_PROP_HAS_TIMESTAMP as u32)) != 0 {
        *timestamp = (*sample).timestamp;
    } else {
        hr = MF_E_NO_SAMPLE_TIMESTAMP;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));

    return hr;
}

unsafe extern "system" fn video_sample_SetSampleTime(iface: *mut IMFSample, timestamp: LONGLONG) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);
    TRACE!("%p, %s.\n", iface, debugstr_time(timestamp));
    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    (*sample).timestamp = timestamp;
    (*sample).flags |= SAMPLE_PROP_HAS_TIMESTAMP as u32;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    return S_OK;
}

unsafe extern "system" fn video_sample_GetSampleDuration(iface: *mut IMFSample, duration: *mut LONGLONG) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    if (*sample).flags & SAMPLE_PROP_HAS_DURATION as u32 != 0 {
        *duration = (*sample).duration;
    } else {
        hr = MF_E_NO_SAMPLE_DURATION;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));

    return hr;
}

unsafe extern "system" fn video_sample_SetSampleDuration(iface: *mut IMFSample, duration: LONGLONG) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    EnterCriticalSection(&raw mut (*sample).cs);
    (*sample).duration = duration;
    (*sample).flags |= SAMPLE_PROP_HAS_DURATION as UINT32;
    LeaveCriticalSection(&raw mut (*sample).cs);

    return S_OK;
}

unsafe extern "system" fn video_sample_GetBufferCount(iface: *mut IMFSample, count: *mut DWORD) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetBufferCount((*sample).sample, count);
}

unsafe extern "system" fn video_sample_GetBufferByIndex(iface: *mut IMFSample, index: DWORD, buffer: *mut *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_GetBufferByIndex((*sample).sample, index, buffer);
}
unsafe extern "system" fn video_sample_ConvertToContiguousBuffer(
    iface: *mut IMFSample,
    buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    // TRACE("%p, %p.\n", iface, buffer);
    E_NOTIMPL
}

unsafe extern "system" fn video_sample_AddBuffer(iface: *mut IMFSample, buffer: *mut IMFMediaBuffer) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_AddBuffer((*sample).sample, buffer);
}

unsafe extern "system" fn video_sample_RemoveBufferByIndex(iface: *mut IMFSample, index: DWORD) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_RemoveBufferByIndex((*sample).sample, index);
}

unsafe extern "system" fn video_sample_RemoveAllBuffers(iface: *mut IMFSample) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFSample(iface);

    return IMFSample_RemoveAllBuffers((*sample).sample);
}

unsafe extern "system" fn video_sample_GetTotalLength(
    iface: *mut IMFSample,
    total_length: *mut DWORD,
) -> HRESULT {
    TRACE!("%p, %p.\n", iface, total_length);

    *total_length = 0;

    return S_OK;
}
unsafe extern "system" fn video_sample_CopyToBuffer(
    iface: *mut IMFSample,
    buffer: *mut IMFMediaBuffer,
) -> HRESULT {
    // TRACE("%p, %p.\n", iface, buffer);
    E_NOTIMPL
}

unsafe extern "system" fn tracked_video_sample_QueryInterface(iface: *mut IMFTrackedSample, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_QueryInterface(core::ptr::addr_of_mut!((*sample).IMFSample_iface), riid, obj);
}

unsafe extern "system" fn tracked_video_sample_AddRef(iface: *mut IMFTrackedSample) -> ULONG {
    let mut sample: *mut video_sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_AddRef(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn tracked_video_sample_Release(mut iface: *mut IMFTrackedSample) -> ULONG {
    let mut sample: *mut video_sample = impl_from_IMFTrackedSample(iface);
    return IMFSample_Release(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn tracked_video_sample_SetAllocator(
    iface: *mut IMFTrackedSample,
    sample_allocator: *mut IMFAsyncCallback,
    state: *mut IUnknown,
) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFTrackedSample(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p, %p.\n", iface, sample_allocator, state);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    if !(*sample).tracked_result.is_null() {
        hr = MF_E_NOTACCEPTING;
    } else {
        hr = create_async_result(
            iface as *mut IUnknown,
            sample_allocator,
            state,
            core::ptr::addr_of_mut!((*sample).tracked_result),
        );
        if hr >= 0 {
            /* Account for additional refcount brought by 'state' object. This threshold is used
               on Release() to invoke tracker callback.  */
            (*sample).tracked_refcount = 1;
            if state == core::ptr::addr_of_mut!((*sample).IMFTrackedSample_iface) as *mut IUnknown
                || state == core::ptr::addr_of_mut!((*sample).IMFSample_iface) as *mut IUnknown
            {
                (*sample).tracked_refcount += 1;
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));

    return hr;
}
unsafe extern "system" fn desired_video_sample_QueryInterface(iface: *mut IMFDesiredSample, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut sample: *mut video_sample = impl_from_IMFDesiredSample(iface);
    return IMFSample_QueryInterface(core::ptr::addr_of_mut!((*sample).IMFSample_iface), riid, obj);
}

unsafe extern "system" fn desired_video_sample_AddRef(iface: *mut IMFDesiredSample) -> ULONG {
    let sample: *mut video_sample = impl_from_IMFDesiredSample(iface);
    return IMFSample_AddRef(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn desired_video_sample_Release(iface: *mut IMFDesiredSample) -> ULONG {
    let sample: *mut video_sample = impl_from_IMFDesiredSample(iface);
    return IMFSample_Release(core::ptr::addr_of_mut!((*sample).IMFSample_iface));
}

unsafe extern "system" fn desired_video_sample_GetDesiredSampleTimeAndDuration(iface: *mut IMFDesiredSample, sample_time: *mut LONGLONG, sample_duration: *mut LONGLONG) -> HRESULT {
    let sample: *mut video_sample = impl_from_IMFDesiredSample(iface);
    let mut hr: HRESULT = S_OK;

    if sample_time.is_null() || sample_duration.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    if (*sample).flags & (SAMPLE_PROP_HAS_DESIRED_PROPS as u32) != 0 {
        *sample_time = (*sample).desired_timestamp;
        *sample_duration = (*sample).desired_duration;
    } else {
        hr = MF_E_NOT_AVAILABLE;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));

    return hr;
}

unsafe extern "system" fn desired_video_sample_SetDesiredSampleTimeAndDuration(
    iface: *mut IMFDesiredSample,
    sample_time: LONGLONG,
    sample_duration: LONGLONG,
) {
    let sample: *mut video_sample = impl_from_IMFDesiredSample(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    (*sample).flags |= SAMPLE_PROP_HAS_DESIRED_PROPS as core::ffi::c_uint;
    (*sample).desired_timestamp = sample_time;
    (*sample).desired_duration = sample_duration;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
}

unsafe extern "system" fn desired_video_sample_Clear(iface: *mut IMFDesiredSample) {
    let mut sample: *mut video_sample = impl_from_IMFDesiredSample(iface);

    TRACE!("%p.\n", iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
    (*sample).flags = 0;
    IMFSample_SetSampleFlags((*sample).sample, 0);
    IMFSample_DeleteAllItems((*sample).sample);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*sample).cs));
}

unsafe extern "system" fn surface_buffer_QueryInterface(iface: *mut IMFMediaBuffer, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut surface_buffer = impl_from_IMFMediaBuffer(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaBuffer)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFGetService)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*buffer).IMFGetService_iface) as *mut core::ffi::c_void;
    }
    else
    {
        WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*obj as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn surface_buffer_AddRef(iface: *mut IMFMediaBuffer) -> ULONG {
    let buffer: *mut surface_buffer = impl_from_IMFMediaBuffer(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*buffer).refcount) as *mut i32) as ULONG;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/sample.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"surface_buffer_AddRef\0" as *const u8 as *const core::ffi::c_char,
            1610 as core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "system" fn surface_buffer_Release(iface: *mut IMFMediaBuffer) -> ULONG {
    let buffer: *mut surface_buffer = impl_from_IMFMediaBuffer(iface);
    let refcount: ULONG = InterlockedDecrement(&raw mut (*buffer).refcount as *mut LONG) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        IUnknown_Release((*buffer).surface);
        free(buffer as *mut core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn surface_buffer_Lock(
    iface: *mut IMFMediaBuffer,
    data: *mut *mut BYTE,
    maxlength: *mut DWORD,
    length: *mut DWORD,
) -> HRESULT {
    // TRACE("%p, %p, %p, %p.\n", iface, data, maxlength, length);
    E_NOTIMPL
}
unsafe extern "system" fn surface_buffer_Unlock(iface: *mut IMFMediaBuffer) -> HRESULT {
    // TRACE("%p.\n", iface);
    E_NOTIMPL
}

unsafe extern "system" fn surface_buffer_GetCurrentLength(iface: *mut IMFMediaBuffer, length: *mut DWORD) -> HRESULT {
    let buffer: *mut surface_buffer = impl_from_IMFMediaBuffer(iface);

    *length = (*buffer).length;

    return S_OK;
}

unsafe extern "system" fn surface_buffer_SetCurrentLength(iface: *mut IMFMediaBuffer, length: DWORD) -> HRESULT {
    let buffer: *mut surface_buffer = impl_from_IMFMediaBuffer(iface);

    (*buffer).length = length;

    return S_OK;
}
unsafe extern "system" fn surface_buffer_GetMaxLength(
    iface: *mut IMFMediaBuffer,
    length: *mut DWORD,
) -> HRESULT {
    // TRACE("%p.\n", iface);
    E_NOTIMPL
}

unsafe extern "system" fn surface_buffer_gs_QueryInterface(iface: *mut IMFGetService, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let buffer: *mut surface_buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_QueryInterface(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface), riid, obj);
}

unsafe extern "system" fn surface_buffer_gs_AddRef(iface: *mut IMFGetService) -> ULONG {
    let buffer: *mut surface_buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_AddRef(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "system" fn surface_buffer_gs_Release(iface: *mut IMFGetService) -> ULONG {
    let buffer: *mut surface_buffer = impl_from_IMFGetService(iface);
    return IMFMediaBuffer_Release(core::ptr::addr_of_mut!((*buffer).IMFMediaBuffer_iface));
}

unsafe extern "system" fn surface_buffer_gs_GetService(iface: *mut IMFGetService, service: *const GUID, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut buffer: *mut surface_buffer = impl_from_IMFGetService(iface);

    if IsEqualGUID!(service as *const GUID, core::ptr::addr_of!(MR_BUFFER_SERVICE)) != false {
        return IUnknown_QueryInterface((*buffer).surface, riid, obj);
    }

    return E_NOINTERFACE;
}

unsafe extern "C" fn create_surface_buffer(
    mut surface: *mut IUnknown,
    mut buffer: *mut *mut IMFMediaBuffer,
) -> HRESULT {
    let mut object: *mut surface_buffer = core::ptr::null_mut::<surface_buffer>();
    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<surface_buffer>() as usize) as _,
    ) as *mut surface_buffer;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFMediaBuffer_iface.lpVtbl = core::ptr::addr_of!(surface_buffer_vtbl);
    (*object).IMFGetService_iface.lpVtbl = core::ptr::addr_of!(surface_buffer_gs_vtbl);
    (*object).refcount = 1 as LONG;
    (*object).surface = surface;
    IUnknown_AddRef((*object).surface);
    *buffer = core::ptr::addr_of_mut!((*object).IMFMediaBuffer_iface);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoSampleFromSurface(surface: *mut IUnknown, sample: *mut *mut IMFSample) -> HRESULT {
    let mut object: *mut video_sample = core::ptr::null_mut();
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc(1, (core::mem::size_of::<video_sample>()) as _) as *mut video_sample;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFSample_iface.lpVtbl = core::ptr::addr_of!(video_sample_vtbl);
    (*object).IMFTrackedSample_iface.lpVtbl = core::ptr::addr_of!(tracked_video_sample_vtbl);
    (*object).IMFDesiredSample_iface.lpVtbl = core::ptr::addr_of!(desired_video_sample_vtbl);
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    hr = MFCreateSample(core::ptr::addr_of_mut!((*object).sample));
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    if !surface.is_null() && {
        hr = create_surface_buffer(surface, core::ptr::addr_of_mut!(buffer));
        hr < 0
    } {
        IMFSample_Release(core::ptr::addr_of_mut!((*object).IMFSample_iface));
        return hr;
    }

    if !buffer.is_null() {
        IMFSample_AddBuffer((*object).sample, buffer);
        IMFMediaBuffer_Release(buffer);
    }

    video_sample_create_tracking_thread();

    *sample = core::ptr::addr_of_mut!((*object).IMFSample_iface);

    return S_OK;
}

// COM vtable tables — generated from sample.c by vtbl_gen.py
/// sample.c:235 `static const IMFAsyncResultVtbl tracked_async_result_vtbl`  (slot order: rust:types_amd64.rs)
pub static tracked_async_result_vtbl: IMFAsyncResultVtbl = IMFAsyncResultVtbl {
    QueryInterface: Some(tracked_async_result_QueryInterface),
    AddRef: Some(tracked_async_result_AddRef),
    Release: Some(tracked_async_result_Release),
    GetState: Some(tracked_async_result_GetState),
    GetStatus: Some(tracked_async_result_GetStatus),
    SetStatus: Some(tracked_async_result_SetStatus),
    GetObject: Some(tracked_async_result_GetObject),
    GetStateNoAddRef: Some(tracked_async_result_GetStateNoAddRef),
};

/// sample.c:701 `static const IMFVideoSampleAllocatorVtbl sample_allocator_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_vtbl: IMFVideoSampleAllocatorVtbl = IMFVideoSampleAllocatorVtbl {
    QueryInterface: Some(sample_allocator_QueryInterface),
    AddRef: Some(sample_allocator_AddRef),
    Release: Some(sample_allocator_Release),
    SetDirectXManager: Some(sample_allocator_SetDirectXManager),
    UninitializeSampleAllocator: Some(sample_allocator_UninitializeSampleAllocator),
    InitializeSampleAllocator: Some(sample_allocator_InitializeSampleAllocator),
    AllocateSample: Some(sample_allocator_AllocateSample),
};

/// sample.c:764 `static const IMFVideoSampleAllocatorCallbackVtbl sample_allocator_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_callback_vtbl: IMFVideoSampleAllocatorCallbackVtbl = IMFVideoSampleAllocatorCallbackVtbl {
    QueryInterface: Some(sample_allocator_callback_QueryInterface),
    AddRef: Some(sample_allocator_callback_AddRef),
    Release: Some(sample_allocator_callback_Release),
    SetCallback: Some(sample_allocator_callback_SetCallback),
    GetFreeSampleCount: Some(sample_allocator_callback_GetFreeSampleCount),
};

/// sample.c:862 `static const IMFAsyncCallbackVtbl sample_allocator_tracking_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static sample_allocator_tracking_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(sample_allocator_tracking_callback_QueryInterface),
    AddRef: Some(sample_allocator_tracking_callback_AddRef),
    Release: Some(sample_allocator_tracking_callback_Release),
    GetParameters: Some(sample_allocator_tracking_callback_GetParameters),
    Invoke: Some(sample_allocator_tracking_callback_Invoke),
};

/// sample.c:1394 `static const IMFSampleVtbl video_sample_vtbl`  (slot order: rust:types_amd64.rs)
pub static video_sample_vtbl: IMFSampleVtbl = IMFSampleVtbl {
    QueryInterface: Some(video_sample_QueryInterface),
    AddRef: Some(video_sample_AddRef),
    Release: Some(video_sample_Release),
    GetItem: Some(video_sample_GetItem),
    GetItemType: Some(video_sample_GetItemType),
    CompareItem: Some(video_sample_CompareItem),
    Compare: Some(video_sample_Compare),
    GetUINT32: Some(video_sample_GetUINT32),
    GetUINT64: Some(video_sample_GetUINT64),
    GetDouble: Some(video_sample_GetDouble),
    GetGUID: Some(video_sample_GetGUID),
    GetStringLength: Some(video_sample_GetStringLength),
    GetString: Some(video_sample_GetString),
    GetAllocatedString: Some(video_sample_GetAllocatedString),
    GetBlobSize: Some(video_sample_GetBlobSize),
    GetBlob: Some(video_sample_GetBlob),
    GetAllocatedBlob: Some(video_sample_GetAllocatedBlob),
    GetUnknown: Some(video_sample_GetUnknown),
    SetItem: Some(video_sample_SetItem),
    DeleteItem: Some(video_sample_DeleteItem),
    DeleteAllItems: Some(video_sample_DeleteAllItems),
    SetUINT32: Some(video_sample_SetUINT32),
    SetUINT64: Some(video_sample_SetUINT64),
    SetDouble: Some(video_sample_SetDouble),
    SetGUID: Some(video_sample_SetGUID),
    SetString: Some(video_sample_SetString),
    SetBlob: Some(video_sample_SetBlob),
    SetUnknown: Some(video_sample_SetUnknown),
    LockStore: Some(video_sample_LockStore),
    UnlockStore: Some(video_sample_UnlockStore),
    GetCount: Some(video_sample_GetCount),
    GetItemByIndex: Some(video_sample_GetItemByIndex),
    CopyAllItems: Some(video_sample_CopyAllItems),
    GetSampleFlags: Some(video_sample_GetSampleFlags),
    SetSampleFlags: Some(video_sample_SetSampleFlags),
    GetSampleTime: Some(video_sample_GetSampleTime),
    SetSampleTime: Some(video_sample_SetSampleTime),
    GetSampleDuration: Some(video_sample_GetSampleDuration),
    SetSampleDuration: Some(video_sample_SetSampleDuration),
    GetBufferCount: Some(video_sample_GetBufferCount),
    GetBufferByIndex: Some(video_sample_GetBufferByIndex),
    ConvertToContiguousBuffer: Some(video_sample_ConvertToContiguousBuffer),
    AddBuffer: Some(video_sample_AddBuffer),
    RemoveBufferByIndex: Some(video_sample_RemoveBufferByIndex),
    RemoveAllBuffers: Some(video_sample_RemoveAllBuffers),
    GetTotalLength: Some(video_sample_GetTotalLength),
    CopyToBuffer: Some(video_sample_CopyToBuffer),
};

/// sample.c:1493 `static const IMFTrackedSampleVtbl tracked_video_sample_vtbl`  (slot order: rust:types_amd64.rs)
pub static tracked_video_sample_vtbl: IMFTrackedSampleVtbl = IMFTrackedSampleVtbl {
    QueryInterface: Some(tracked_video_sample_QueryInterface),
    AddRef: Some(tracked_video_sample_AddRef),
    Release: Some(tracked_video_sample_Release),
    SetAllocator: Some(tracked_video_sample_SetAllocator),
};

/// sample.c:1570 `static const IMFDesiredSampleVtbl desired_video_sample_vtbl`  (slot order: rust:types_amd64.rs)
pub static desired_video_sample_vtbl: IMFDesiredSampleVtbl = IMFDesiredSampleVtbl {
    QueryInterface: Some(desired_video_sample_QueryInterface),
    AddRef: Some(desired_video_sample_AddRef),
    Release: Some(desired_video_sample_Release),
    GetDesiredSampleTimeAndDuration: Some(desired_video_sample_GetDesiredSampleTimeAndDuration),
    SetDesiredSampleTimeAndDuration: Some(desired_video_sample_SetDesiredSampleTimeAndDuration),
    Clear: Some(desired_video_sample_Clear),
};

/// sample.c:1674 `static const IMFMediaBufferVtbl surface_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static surface_buffer_vtbl: IMFMediaBufferVtbl = IMFMediaBufferVtbl {
    QueryInterface: Some(surface_buffer_QueryInterface),
    AddRef: Some(surface_buffer_AddRef),
    Release: Some(surface_buffer_Release),
    Lock: Some(surface_buffer_Lock),
    Unlock: Some(surface_buffer_Unlock),
    GetCurrentLength: Some(surface_buffer_GetCurrentLength),
    SetCurrentLength: Some(surface_buffer_SetCurrentLength),
    GetMaxLength: Some(surface_buffer_GetMaxLength),
};

/// sample.c:1716 `static const IMFGetServiceVtbl surface_buffer_gs_vtbl`  (slot order: rust:types_amd64.rs)
pub static surface_buffer_gs_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(surface_buffer_gs_QueryInterface),
    AddRef: Some(surface_buffer_gs_AddRef),
    Release: Some(surface_buffer_gs_Release),
    GetService: Some(surface_buffer_gs_GetService),
};
