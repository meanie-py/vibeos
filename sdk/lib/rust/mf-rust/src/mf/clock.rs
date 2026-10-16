use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const CLOCK_CMD_START: i32 = 0;
pub const CLOCK_CMD_STOP: i32 = 1;
pub const CLOCK_CMD_PAUSE: i32 = 2;
pub const CLOCK_CMD_SET_RATE: i32 = 3;
pub const CLOCK_CMD_MAX: i32 = 4;

pub const CLOCK_NOTIFY_START: i32 = 0;
pub const CLOCK_NOTIFY_STOP: i32 = 1;
pub const CLOCK_NOTIFY_PAUSE: i32 = 2;
pub const CLOCK_NOTIFY_RESTART: i32 = 3;
pub const CLOCK_NOTIFY_SET_RATE: i32 = 4;

#[repr(C)]
pub struct clock_sink {
    pub entry: list,
    pub state_sink: *mut IMFClockStateSink,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union clock_state_change_param_u {
    pub offset: LONGLONG,
    pub rate: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct clock_state_change_param {
    pub u: clock_state_change_param_u,
}

#[repr(C)]
pub struct sink_notification {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub system_time: MFTIME,
    pub param: clock_state_change_param,
    pub notification: i32,
    pub sink: *mut IMFClockStateSink,
}

#[repr(C)]
pub struct clock_timer {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub time: LONGLONG,
    pub result: *mut IMFAsyncResult,
    pub entry: list,
}

#[repr(C)]
pub struct presentation_clock {
    pub IMFPresentationClock_iface: IMFPresentationClock,
    pub IMFRateControl_iface: IMFRateControl,
    pub IMFTimer_iface: IMFTimer,
    pub IMFShutdown_iface: IMFShutdown,
    pub sink_callback: IMFAsyncCallback,
    pub refcount: LONG,
    pub time_source: *mut IMFPresentationTimeSource,
    pub time_source_sink: *mut IMFClockStateSink,
    pub state: MFCLOCK_STATE,
    pub start_offset: LONGLONG,
    pub sinks: list,
    pub timers: list,
    pub rate: f32,
    pub frequency: LONGLONG,
    pub cs: CRITICAL_SECTION,
    pub is_shut_down: BOOL,
    pub key: DWORD,
}

unsafe extern "C" fn impl_from_IMFPresentationClock(iface: *mut IMFPresentationClock) -> *mut presentation_clock {
    return (iface as *mut u8).sub(core::mem::offset_of!(presentation_clock, IMFPresentationClock_iface)) as *mut presentation_clock;
}

unsafe extern "C" fn impl_from_IMFRateControl(mut iface: *mut IMFRateControl) -> *mut presentation_clock {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(presentation_clock, IMFRateControl_iface),
    ) as *mut presentation_clock;
}

unsafe extern "C" fn impl_from_IMFTimer(iface: *mut IMFTimer) -> *mut presentation_clock {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(presentation_clock, IMFTimer_iface))
        as *mut presentation_clock;
}

unsafe extern "C" fn impl_from_IMFShutdown(iface: *mut IMFShutdown) -> *mut presentation_clock {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(presentation_clock, IMFShutdown_iface)) as *mut presentation_clock;
}

unsafe extern "C" fn impl_from_sink_callback_IMFAsyncCallback(
    iface: *mut IMFAsyncCallback,
) -> *mut presentation_clock {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(presentation_clock, sink_callback),
    ) as *mut presentation_clock;
}

unsafe extern "C" fn impl_clock_timer_from_IUnknown(iface: *mut IUnknown) -> *mut clock_timer {
    return CONTAINING_RECORD!(iface, clock_timer, IUnknown_iface);
}

unsafe extern "C" fn impl_sink_notification_from_IUnknown(iface: *mut IUnknown) -> *mut sink_notification {
    return (iface as *mut u8).sub(core::mem::offset_of!(sink_notification, IUnknown_iface)) as *mut sink_notification;
}

unsafe extern "system" fn sink_notification_QueryInterface(iface: *mut IUnknown, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *out = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn sink_notification_AddRef(mut iface: *mut IUnknown) -> ULONG {
    let mut notification: *mut sink_notification = impl_sink_notification_from_IUnknown(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        &raw mut (*notification).refcount as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"sink_notification_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            155 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn sink_notification_Release(iface: *mut IUnknown) -> ULONG {
    let notification: *mut sink_notification = impl_sink_notification_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*notification).refcount) as *mut LONG,
    ) as ULONG;

    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const core::ffi::c_char,
            b"sink_notification_Release\0" as *const u8 as *const core::ffi::c_char,
            165 as i32,
            b"%p, refcount %lu.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            refcount,
        );
    }

    if refcount == 0 {
        IMFClockStateSink_Release((*notification).sink);
        free(notification as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn clock_notify_async_sink(
    mut clock: *mut presentation_clock,
    mut system_time: MFTIME,
    mut param: clock_state_change_param,
    mut notification: clock_notification,
    mut sink: *mut IMFClockStateSink,
) {
    let mut object: *mut sink_notification = ::core::ptr::null_mut::<sink_notification>();
    let mut result: *mut IMFAsyncResult = ::core::ptr::null_mut::<IMFAsyncResult>();
    let mut hr: HRESULT = 0;
    object = malloc(::core::mem::size_of::<sink_notification>() as size_t)
        as *mut sink_notification;
    if object.is_null() {
        return;
    }
    (*object).IUnknown_iface.lpVtbl = &raw const sinknotificationvtbl;
    (*object).refcount = 1 as LONG;
    (*object).system_time = system_time;
    (*object).param = param;
    (*object).notification = (notification) as i32;
    (*object).sink = sink;
    IMFClockStateSink_AddRef((*object).sink);
    hr = MFCreateAsyncResult(
        &raw mut (*object).IUnknown_iface,
        &raw mut (*clock).sink_callback,
        ::core::ptr::null_mut::<IUnknown>(),
        &raw mut result,
    );
    (*(*object).IUnknown_iface.lpVtbl)
        .Release
        .unwrap_unchecked()(&raw mut (*object).IUnknown_iface);
    if hr >= 0 {
        MFPutWorkItemEx(MFASYNC_CALLBACK_QUEUE_STANDARD as DWORD, result);
        IMFAsyncResult_Release(result);
    }
}

unsafe extern "system" fn present_clock_QueryInterface(iface: *mut IMFPresentationClock, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFPresentationClock)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClock)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFRateControl)) != 0
    {
        *out = core::ptr::addr_of_mut!((*clock).IMFRateControl_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTimer)) != 0
    {
        *out = core::ptr::addr_of_mut!((*clock).IMFTimer_iface) as *mut core::ffi::c_void;
    }
    else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFShutdown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*clock).IMFShutdown_iface) as *mut core::ffi::c_void;
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

unsafe extern "system" fn present_clock_AddRef(iface: *mut IMFPresentationClock) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*clock).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    return refcount;
}
unsafe extern "system" fn present_clock_Release(iface: *mut IMFPresentationClock) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*clock).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    let mut timer: *mut clock_timer;
    let mut timer2: *mut clock_timer;
    let mut sink: *mut clock_sink;
    let mut sink2: *mut clock_sink;

    if refcount == 0 {
        if !(*clock).time_source.is_null() {
            IMFPresentationTimeSource_Release((*clock).time_source);
        }
        if !(*clock).time_source_sink.is_null() {
            IMFClockStateSink_Release((*clock).time_source_sink);
        }
        sink = ((*clock).sinks.next as *mut u8)
            .sub(core::mem::offset_of!(clock_sink, entry)) as *mut clock_sink;
        sink2 = (((*sink).entry.next) as *mut u8)
            .sub(core::mem::offset_of!(clock_sink, entry)) as *mut clock_sink;
        while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*clock).sinks) {
            list_remove(core::ptr::addr_of_mut!((*sink).entry));
            IMFClockStateSink_Release((*sink).state_sink);
            free(sink as *mut ::core::ffi::c_void);
            sink = sink2;
            sink2 = (((*sink).entry.next) as *mut u8)
                .sub(core::mem::offset_of!(clock_sink, entry)) as *mut clock_sink;
        }
        timer = ((*clock).timers.next as *mut u8)
            .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        timer2 = (((*timer).entry.next) as *mut u8)
            .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        while core::ptr::addr_of_mut!((*timer).entry) != core::ptr::addr_of_mut!((*clock).timers) {
            list_remove(core::ptr::addr_of_mut!((*timer).entry));
            IUnknown_Release(core::ptr::addr_of_mut!((*timer).IUnknown_iface));
            timer = timer2;
            timer2 = (((*timer).entry.next) as *mut u8)
                .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
        free(clock as *mut ::core::ffi::c_void);
    }

    refcount
}

unsafe extern "system" fn present_clock_GetClockCharacteristics(iface: *mut IMFPresentationClock, flags: *mut DWORD) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut hr: HRESULT = MF_E_CLOCK_NO_TIME_SOURCE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    if !(*clock).time_source.is_null() {
        hr = IMFPresentationTimeSource_GetClockCharacteristics((*clock).time_source, flags);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_GetCorrelatedTime(
    iface: *mut IMFPresentationClock,
    reserved: DWORD,
    clock_time: *mut LONGLONG,
    system_time: *mut MFTIME,
) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut hr: HRESULT = MF_E_CLOCK_NO_TIME_SOURCE;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"present_clock_GetCorrelatedTime\0" as *const u8 as *const ::core::ffi::c_char,
            309 as ::core::ffi::c_int,
            b"%p, %#lx, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            reserved,
            clock_time,
            system_time,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    if !(*clock).time_source.is_null() {
        hr = IMFPresentationTimeSource_GetCorrelatedTime(
            (*clock).time_source,
            reserved,
            clock_time,
            system_time,
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_GetContinuityKey(iface: *mut IMFPresentationClock, key: *mut DWORD) -> HRESULT {
    *key = 0;

    return S_OK;
}

unsafe extern "system" fn present_clock_GetState(iface: *mut IMFPresentationClock, reserved: DWORD, state: *mut MFCLOCK_STATE) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);

    EnterCriticalSection(&raw mut (*clock).cs);
    *state = (*clock).state;
    LeaveCriticalSection(&raw mut (*clock).cs);

    return S_OK;
}

unsafe extern "system" fn present_clock_GetProperties(iface: *mut IMFPresentationClock, props: *mut MFCLOCK_PROPERTIES) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut hr: HRESULT = MF_E_CLOCK_NO_TIME_SOURCE;
    TRACE!("%p, %p.\n", iface, props);
    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    if !(*clock).time_source.is_null() {
        hr = IMFPresentationTimeSource_GetProperties((*clock).time_source, props);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    return hr;
}
unsafe extern "system" fn present_clock_SetTimeSource(
    iface: *mut IMFPresentationClock,
    time_source: *mut IMFPresentationTimeSource,
) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut props: MFCLOCK_PROPERTIES = core::mem::zeroed::<MFCLOCK_PROPERTIES>();
    let mut source_clock: *mut IMFClock = core::ptr::null_mut::<IMFClock>();
    let mut hr: HRESULT;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const core::ffi::c_char,
            b"present_clock_SetTimeSource\0" as *const u8 as *const core::ffi::c_char,
            364 as core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            time_source,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    if !(*clock).time_source.is_null() {
        IMFPresentationTimeSource_Release((*clock).time_source);
    }
    if !(*clock).time_source_sink.is_null() {
        IMFClockStateSink_Release((*clock).time_source_sink);
    }
    (*clock).time_source = core::ptr::null_mut::<IMFPresentationTimeSource>();
    (*clock).time_source_sink = core::ptr::null_mut::<IMFClockStateSink>();

    hr = IMFPresentationTimeSource_QueryInterface(
        time_source,
        core::ptr::addr_of!(IID_IMFClockStateSink),
        core::ptr::addr_of_mut!((*clock).time_source_sink) as *mut *mut core::ffi::c_void,
    );
    if hr >= 0 {
        (*clock).time_source = time_source;
        IMFPresentationTimeSource_AddRef((*clock).time_source);
    }

    if IMFPresentationTimeSource_GetUnderlyingClock(time_source, core::ptr::addr_of_mut!(source_clock)) >= 0 {
        if IMFClock_GetProperties(source_clock, core::ptr::addr_of_mut!(props)) >= 0 {
            (*clock).frequency = props.qwClockFrequency as LONGLONG;
        }
        IMFClock_Release(source_clock);
    }

    if (*clock).frequency == 0 {
        (*clock).frequency = MFCLOCK_FREQUENCY_HNS as LONGLONG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}
unsafe extern "system" fn present_clock_GetTimeSource(
    iface: *mut IMFPresentationClock,
    time_source: *mut *mut IMFPresentationTimeSource,
) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut hr: HRESULT = S_OK;

    if time_source.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(&raw mut (*clock).cs);
    if !(*clock).time_source.is_null() {
        *time_source = (*clock).time_source;
        IMFPresentationTimeSource_AddRef(*time_source);
    } else {
        hr = MF_E_CLOCK_NO_TIME_SOURCE;
    }
    LeaveCriticalSection(&raw mut (*clock).cs);

    return hr;
}
unsafe extern "system" fn present_clock_GetTime(iface: *mut IMFPresentationClock, time: *mut MFTIME) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut hr: HRESULT = MF_E_CLOCK_NO_TIME_SOURCE;
    let mut systime: MFTIME = 0;

    TRACE!("%p, %p.\n", iface, time);

    if time.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    if !(*clock).time_source.is_null() {
        hr = IMFPresentationTimeSource_GetCorrelatedTime(
            (*clock).time_source,
            0,
            time as *mut LONGLONG,
            core::ptr::addr_of_mut!(systime),
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_AddClockStateSink(
    iface: *mut IMFPresentationClock,
    state_sink: *mut IMFClockStateSink,
) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut sink: *mut clock_sink = core::ptr::null_mut();
    let mut cur: *mut clock_sink = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    if state_sink.is_null() {
        return E_INVALIDARG;
    }

    sink = malloc(core::mem::size_of::<clock_sink>() as size_t) as *mut clock_sink;
    if sink.is_null() {
        return E_OUTOFMEMORY;
    }

    (*sink).state_sink = state_sink;
    IMFClockStateSink_AddRef((*sink).state_sink);

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    cur = ((*clock).sinks.next as *mut u8)
        .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
    while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!((*clock).sinks) {
        if (*cur).state_sink == state_sink {
            hr = E_INVALIDARG;
            break;
        } else {
            cur = ((*cur).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
        }
    }

    if hr >= 0 {
        let notifications: [clock_notification; (MFCLOCK_STATE_PAUSED + 1) as _] = {
            let mut a = [CLOCK_NOTIFY_START as clock_notification; (MFCLOCK_STATE_PAUSED + 1) as _];
            a[MFCLOCK_STATE_RUNNING as usize] = CLOCK_NOTIFY_START as clock_notification;
            a[MFCLOCK_STATE_STOPPED as usize] = CLOCK_NOTIFY_STOP as clock_notification;
            a[MFCLOCK_STATE_PAUSED as usize] = CLOCK_NOTIFY_PAUSE as clock_notification;
            a
        };
        let mut param: clock_state_change_param = core::mem::zeroed();

        if (*clock).is_shut_down == 0
            && (*clock).state != MFCLOCK_STATE_INVALID
            /* Don't notify a sink that is also the time source */
            && (*clock).time_source_sink != (*sink).state_sink
        {
            param.u.offset = (*clock).start_offset;
            clock_notify_async_sink(
                clock,
                MFGetSystemTime(),
                param,
                notifications[(*clock).state as usize],
                (*sink).state_sink,
            );
        }

        list_add_tail(
            core::ptr::addr_of_mut!((*clock).sinks),
            core::ptr::addr_of_mut!((*sink).entry),
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    if hr < 0 {
        IMFClockStateSink_Release((*sink).state_sink);
        free(sink as *mut core::ffi::c_void);
    }

    return hr;
}

unsafe extern "system" fn present_clock_RemoveClockStateSink(
    iface: *mut IMFPresentationClock,
    state_sink: *mut IMFClockStateSink,
) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut sink: *mut clock_sink;

    if state_sink.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    sink = ((*clock).sinks.next as *mut u8)
        .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
    while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*clock).sinks) {
        if (*sink).state_sink == state_sink {
            IMFClockStateSink_Release((*sink).state_sink);
            list_remove(core::ptr::addr_of_mut!((*sink).entry));
            free(sink as *mut core::ffi::c_void);
            break;
        } else {
            sink = ((*sink).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return S_OK;
}

unsafe extern "C" fn clock_call_state_change(
    mut system_time: MFTIME,
    mut param: clock_state_change_param,
    mut notification: clock_notification,
    mut sink: *mut IMFClockStateSink,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    match (notification) as i32 {
        CLOCK_NOTIFY_START => {
            hr = IMFClockStateSink_OnClockStart(sink, system_time, param.u.offset);
        }
        CLOCK_NOTIFY_STOP => {
            hr = IMFClockStateSink_OnClockStop(sink, system_time);
        }
        CLOCK_NOTIFY_PAUSE => {
            hr = IMFClockStateSink_OnClockPause(sink, system_time);
        }
        CLOCK_NOTIFY_RESTART => {
            hr = IMFClockStateSink_OnClockRestart(sink, system_time);
        }
        CLOCK_NOTIFY_SET_RATE => {
            /* System time source does not allow 0.0 rate, presentation clock allows it without raising errors. */
            IMFClockStateSink_OnClockSetRate(sink, system_time, param.u.rate);
        }
        _ => {}
    }

    return hr;
}
unsafe extern "system" fn presentation_clock_timer_callback(context: *mut IUnknown) {
    let clock: *mut presentation_clock = impl_from_IMFPresentationClock(context as *mut IMFPresentationClock);
    let mut time_source: *mut IMFPresentationTimeSource = core::ptr::null_mut();
    let mut timer: *mut clock_timer = core::ptr::null_mut();
    let mut next: *mut clock_timer = core::ptr::null_mut();
    let mut time: LONGLONG = 0;
    let mut systime: LONGLONG = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    time_source = (*clock).time_source;
    if !time_source.is_null() {
        IMFPresentationTimeSource_AddRef(time_source);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    /* We don't hold the critical section during a call to GetCorrelatedTime as it can result in a deadlock. */
    if !time_source.is_null()
        && (IMFPresentationTimeSource_GetCorrelatedTime(
            time_source,
            0,
            core::ptr::addr_of_mut!(time),
            core::ptr::addr_of_mut!(systime),
        )) >= 0
    {
        EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
        timer = ((*clock).timers.next as *mut u8)
            .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        next = ((*timer).entry.next as *mut u8)
            .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        while core::ptr::addr_of_mut!((*timer).entry) != core::ptr::addr_of_mut!((*clock).timers) {
            if (*timer).time > time + 50000 {
                break;
            }

            list_remove(core::ptr::addr_of_mut!((*timer).entry));
            MFInvokeCallback((*timer).result);
            IUnknown_Release(core::ptr::addr_of_mut!((*timer).IUnknown_iface));

            timer = next;
            next = ((*timer).entry.next as *mut u8)
                .sub(core::mem::offset_of!(clock_timer, entry)) as *mut clock_timer;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    }

    if !time_source.is_null() {
        IMFPresentationTimeSource_Release(time_source);
    }
}

unsafe extern "C" fn clock_change_state(
    clock: *mut presentation_clock,
    command: clock_command,
    param: clock_state_change_param,
) -> HRESULT {
    let mut states = [MFCLOCK_STATE_INVALID; (CLOCK_CMD_MAX as _)];
    states[CLOCK_CMD_START as usize] = MFCLOCK_STATE_RUNNING;
    states[CLOCK_CMD_STOP as usize] = MFCLOCK_STATE_STOPPED;
    states[CLOCK_CMD_PAUSE as usize] = MFCLOCK_STATE_PAUSED;
    let mut notifications = [CLOCK_NOTIFY_START; (CLOCK_CMD_MAX as _)];
    notifications[CLOCK_CMD_START as usize] = CLOCK_NOTIFY_START;
    notifications[CLOCK_CMD_STOP as usize] = CLOCK_NOTIFY_STOP;
    notifications[CLOCK_CMD_PAUSE as usize] = CLOCK_NOTIFY_PAUSE;
    notifications[CLOCK_CMD_SET_RATE as usize] = CLOCK_NOTIFY_SET_RATE;
    let mut notification: clock_notification = (CLOCK_NOTIFY_START as u32);
    let mut sink: *mut clock_sink = core::ptr::null_mut();
    let mut old_state: MFCLOCK_STATE = MFCLOCK_STATE_INVALID;
    let mut system_time: MFTIME = 0;
    let mut hr: HRESULT = 0;

    if (*clock).time_source.is_null() {
        return MF_E_CLOCK_NO_TIME_SOURCE;
    }

    if command as u32 != CLOCK_CMD_SET_RATE as u32
        && (*clock).state as u32 == states[command as usize] as u32
        && (*clock).state as u32 != MFCLOCK_STATE_RUNNING as u32
    {
        return MF_E_CLOCK_STATE_ALREADY_SET;
    }

    /* Invalid transitions. */
    if command as u32 == CLOCK_CMD_PAUSE as u32
        && ((*clock).state as u32 == MFCLOCK_STATE_STOPPED as u32
            || (*clock).state as u32 == MFCLOCK_STATE_PAUSED as u32)
    {
        return MF_E_INVALIDREQUEST;
    }

    system_time = MFGetSystemTime();

    if command as u32 == CLOCK_CMD_START as u32
        && (*clock).state as u32 == MFCLOCK_STATE_PAUSED as u32
        && param.u.offset == (PRESENTATION_CURRENT_POSITION as _)
    {
        notification = (CLOCK_NOTIFY_RESTART as u32);
    } else {
        notification = (notifications[command as usize]) as u32;
    }

    hr = clock_call_state_change(system_time, param, notification, (*clock).time_source_sink);
    if hr < 0 {
        return hr;
    }

    old_state = (*clock).state;
    if command as u32 != CLOCK_CMD_SET_RATE as u32 {
        (*clock).state = states[command as usize];
    }

    /* Dump all pending timer requests immediately on start; otherwise try to cancel scheduled items when
       transitioning from running state. */
    if ((*clock).state as u32 == MFCLOCK_STATE_RUNNING as u32)
        ^ (old_state as u32 == MFCLOCK_STATE_RUNNING as u32)
    {
        if (*clock).state as u32 == MFCLOCK_STATE_RUNNING as u32 {
            hr = MFAddPeriodicCallback(
                Some(presentation_clock_timer_callback),
                core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface) as *mut IUnknown,
                core::ptr::addr_of_mut!((*clock).key),
            );
            if hr < 0 {
                ERR!("Failed to start periodic callback %#lx\n", hr);
            }
        } else {
            if (*clock).key != 0 {
                MFRemovePeriodicCallback((*clock).key);
                (*clock).key = 0;
            }
        }
    }

    /* LIST_FOR_EACH_ENTRY(sink, &clock->sinks, struct clock_sink, entry) */
    sink = ((*clock).sinks.next as *mut u8)
        .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
    while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*clock).sinks) {
        /* Don't notify a sink that is also the time source */
        if (*clock).time_source_sink != (*sink).state_sink {
            clock_notify_async_sink(clock, system_time, param, notification, (*sink).state_sink);
        }
        sink = ((*sink).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(clock_sink, entry) as isize)) as *mut clock_sink;
    }

    return S_OK;
}
unsafe extern "system" fn present_clock_Start(
    mut iface: *mut IMFPresentationClock,
    mut start_offset: LONGLONG,
) -> HRESULT {
    let mut clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut param: clock_state_change_param = clock_state_change_param {
        u: clock_state_change_param_u {
            offset: 0,
        },
    };
    let mut hr: HRESULT = 0;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"present_clock_Start\0" as *const u8 as *const ::core::ffi::c_char,
            671 as ::core::ffi::c_int,
            b"%p, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_time(start_offset),
        );
    }
    EnterCriticalSection(&raw mut (*clock).cs);
    param.u.offset = start_offset;
    (*clock).start_offset = param.u.offset;
    hr = clock_change_state(clock, (CLOCK_CMD_START) as clock_command, param);
    LeaveCriticalSection(&raw mut (*clock).cs);

    return hr;
}

unsafe extern "system" fn present_clock_Stop(mut iface: *mut IMFPresentationClock) -> HRESULT {
    let mut clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut param: clock_state_change_param = core::mem::zeroed();
    param.u.offset = 0 as LONGLONG;
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"present_clock_Stop\0" as *const u8 as *const ::core::ffi::c_char,
            687 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    hr = clock_change_state(clock, CLOCK_CMD_STOP as _, param);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_Pause(mut iface: *mut IMFPresentationClock) -> HRESULT {
    let mut clock: *mut presentation_clock = impl_from_IMFPresentationClock(iface);
    let mut param: clock_state_change_param = clock_state_change_param {
        u: clock_state_change_param_u {
            offset: 0 as LONGLONG,
        },
    };
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/clock.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"present_clock_Pause\0" as *const u8 as *const ::core::ffi::c_char,
            702 as ::core::ffi::c_int,
            b"%p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
        );
    }
    EnterCriticalSection(&raw mut (*clock).cs);
    hr = clock_change_state(clock, (CLOCK_CMD_PAUSE) as clock_command, param);
    LeaveCriticalSection(&raw mut (*clock).cs);

    return hr;
}

unsafe extern "system" fn present_clock_rate_control_QueryInterface(iface: *mut IMFRateControl, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFRateControl(iface);
    return IMFPresentationClock_QueryInterface(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface), riid, out);
}

unsafe extern "system" fn present_clock_rate_control_AddRef(iface: *mut IMFRateControl) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFRateControl(iface);
    return IMFPresentationClock_AddRef(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_rate_control_Release(iface: *mut IMFRateControl) -> ULONG {
    let mut clock: *mut presentation_clock = impl_from_IMFRateControl(iface);
    return IMFPresentationClock_Release(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_rate_SetRate(iface: *mut IMFRateControl, thin: BOOL, rate: f32) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFRateControl(iface);
    let mut param: clock_state_change_param = core::mem::zeroed();
    let mut hr: HRESULT;

    if thin != 0 {
        return MF_E_THINNING_UNSUPPORTED;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    param.u.rate = rate;
    hr = clock_change_state(clock, (CLOCK_CMD_SET_RATE) as clock_command, param);
    if hr >= 0 {
        (*clock).rate = rate;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_rate_GetRate(iface: *mut IMFRateControl, thin: *mut BOOL, rate: *mut f32) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFRateControl(iface);

    if rate.is_null() {
        return E_INVALIDARG;
    }

    if !thin.is_null() {
        *thin = (FALSE as i32);
    }

    EnterCriticalSection(&raw mut (*clock).cs);
    *rate = (*clock).rate;
    LeaveCriticalSection(&raw mut (*clock).cs);

    return S_OK;
}

unsafe extern "system" fn present_clock_timer_QueryInterface(iface: *mut IMFTimer, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFTimer(iface);
    return IMFPresentationClock_QueryInterface(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface), riid, out);
}

unsafe extern "system" fn present_clock_timer_AddRef(iface: *mut IMFTimer) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFTimer(iface);
    return IMFPresentationClock_AddRef(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_timer_Release(iface: *mut IMFTimer) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFTimer(iface);
    return IMFPresentationClock_Release(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "C" fn present_clock_schedule_timer(clock: *mut presentation_clock, flags: DWORD, mut time: LONGLONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown, timer: *mut clock_timer) -> HRESULT {
    let mut systime: MFTIME = 0;
    let mut clocktime: MFTIME = 0;
    let mut hr: HRESULT = 0;

    if flags & MFTIMER_RELATIVE != 0 {
        if (*clock).time_source.is_null() {
            return MF_E_CLOCK_NO_TIME_SOURCE;
        }

        hr = IMFPresentationTimeSource_GetCorrelatedTime((*clock).time_source, 0, core::ptr::addr_of_mut!(clocktime), core::ptr::addr_of_mut!(systime));
        if hr < 0 {
            WARN!("Failed to get clock time, hr %#lx.\n", hr);
            return hr;
        }
        time += clocktime as LONGLONG;
    }

    (*timer).time = time;

    return MFCreateAsyncResult(core::ptr::null_mut(), callback, state, core::ptr::addr_of_mut!((*timer).result));
}

unsafe extern "system" fn clock_timer_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn clock_timer_AddRef(iface: *mut IUnknown) -> ULONG {
    let timer: *mut clock_timer = impl_clock_timer_from_IUnknown(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*timer).refcount) as *mut LONG) as ULONG;
}

unsafe extern "system" fn clock_timer_Release(iface: *mut IUnknown) -> ULONG {
    let timer: *mut clock_timer = impl_clock_timer_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*timer).refcount)) as ULONG;

    if refcount == 0 {
        IMFAsyncResult_Release((*timer).result);
        free(timer as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn present_clock_timer_SetTimer(iface: *mut IMFTimer, flags: DWORD, time: LONGLONG, callback: *mut IMFAsyncCallback, state: *mut IUnknown, cancel_key: *mut *mut IUnknown) -> HRESULT {
    let mut clock: *mut presentation_clock = impl_from_IMFTimer(iface);
    let mut clock_timer: *mut clock_timer = core::ptr::null_mut();
    let mut cur: *mut clock_timer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %#lx, %s, %p, %p, %p.\n", iface, flags, debugstr_time(time), callback, state, cancel_key);

    clock_timer = calloc(1, (core::mem::size_of::<clock_timer>()) as _) as *mut clock_timer;
    if clock_timer.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = present_clock_schedule_timer(clock, flags, time, callback, state, clock_timer);
    if hr < 0 {
        free(clock_timer as *mut core::ffi::c_void);
        return hr;
    }

    (*clock_timer).IUnknown_iface.lpVtbl = core::ptr::addr_of!(clock_timer_vtbl);
    (*clock_timer).refcount = 1;

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    if (*clock).state == MFCLOCK_STATE_STOPPED {
        hr = MF_S_CLOCK_STOPPED;
    }

    if hr >= 0 {
        cur = ((*clock).timers.prev as *mut u8)
            .offset(-(core::mem::offset_of!(clock_timer, entry) as isize)) as *mut clock_timer;
        while core::ptr::addr_of_mut!((*cur).entry) != core::ptr::addr_of_mut!((*clock).timers) {
            if (*cur).time <= time {
                break;
            }
            cur = ((*cur).entry.prev as *mut u8)
                .offset(-(core::mem::offset_of!(clock_timer, entry) as isize)) as *mut clock_timer;
        }
        list_add_after(core::ptr::addr_of_mut!((*cur).entry), core::ptr::addr_of_mut!((*clock_timer).entry));

        if !cancel_key.is_null() {
            *cancel_key = core::ptr::addr_of_mut!((*clock_timer).IUnknown_iface);
            IUnknown_AddRef(*cancel_key);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    if hr < 0 {
        IUnknown_Release(core::ptr::addr_of_mut!((*clock_timer).IUnknown_iface));
    }

    return hr;
}

unsafe extern "system" fn present_clock_timer_CancelTimer(iface: *mut IMFTimer, cancel_key: *mut IUnknown) -> HRESULT {
    let mut clock: *mut presentation_clock = impl_from_IMFTimer(iface);
    let mut timer: *mut clock_timer = core::ptr::null_mut::<clock_timer>();

    TRACE!("%p, %p.\n", iface, cancel_key);

    EnterCriticalSection(&raw mut (*clock).cs);

    timer = ((*clock).timers.next as *mut u8).offset(
        -(&raw mut (*core::ptr::null_mut::<clock_timer>()).entry as usize as isize),
    ) as *mut clock_timer;
    while &raw mut (*timer).entry != &raw mut (*clock).timers {
        if &raw mut (*timer).IUnknown_iface == cancel_key {
            list_remove(&raw mut (*timer).entry);
            IUnknown_Release(&raw mut (*timer).IUnknown_iface);
            break;
        } else {
            timer = ((*timer).entry.next as *mut u8).offset(
                -(&raw mut (*core::ptr::null_mut::<clock_timer>()).entry as usize as isize),
            ) as *mut clock_timer;
        }
    }

    LeaveCriticalSection(&raw mut (*clock).cs);

    return S_OK;
}

unsafe extern "system" fn present_clock_shutdown_QueryInterface(iface: *mut IMFShutdown, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFShutdown(iface);
    return IMFPresentationClock_QueryInterface(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface), riid, out);
}
unsafe extern "system" fn present_clock_shutdown_AddRef(iface: *mut IMFShutdown) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFShutdown(iface);
    return IMFPresentationClock_AddRef(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_shutdown_Release(iface: *mut IMFShutdown) -> ULONG {
    let clock: *mut presentation_clock = impl_from_IMFShutdown(iface);
    return IMFPresentationClock_Release(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_shutdown_Shutdown(iface: *mut IMFShutdown) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFShutdown(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    (*clock).is_shut_down = (TRUE as i32);
    if (*clock).key != 0 {
        MFRemovePeriodicCallback((*clock).key);
        (*clock).key = 0;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return S_OK;
}

unsafe extern "system" fn present_clock_shutdown_GetShutdownStatus(iface: *mut IMFShutdown, status: *mut MFSHUTDOWN_STATUS) -> HRESULT {
    let clock: *mut presentation_clock = impl_from_IMFShutdown(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, status);

    if status.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*clock).cs));
    if (*clock).is_shut_down != 0 {
        *status = MFSHUTDOWN_COMPLETED;
    } else {
        hr = MF_E_INVALIDREQUEST;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*clock).cs));

    return hr;
}

unsafe extern "system" fn present_clock_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", wine_dbgstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn present_clock_sink_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let clock: *mut presentation_clock = impl_from_sink_callback_IMFAsyncCallback(iface);
    return IMFPresentationClock_AddRef(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}

unsafe extern "system" fn present_clock_sink_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let clock: *mut presentation_clock = impl_from_sink_callback_IMFAsyncCallback(iface);
    return IMFPresentationClock_Release(core::ptr::addr_of_mut!((*clock).IMFPresentationClock_iface));
}
unsafe extern "system" fn present_clock_callback_GetParameters(iface: *mut IMFAsyncCallback, flags: *mut DWORD, queue: *mut DWORD) -> HRESULT {
    *flags = 0;
    *queue = MFASYNC_CALLBACK_QUEUE_MULTITHREADED;
    return S_OK;
}

unsafe extern "system" fn present_clock_sink_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let mut data: *mut sink_notification;
    let mut object: *mut IUnknown = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = IMFAsyncResult_GetObject(result, core::ptr::addr_of_mut!(object));
    if hr < 0 {
        return hr;
    }

    data = impl_sink_notification_from_IUnknown(object);

    clock_call_state_change((*data).system_time, (*data).param, ((*data).notification) as clock_notification, (*data).sink);

    IUnknown_Release(object);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreatePresentationClock(clock: *mut *mut IMFPresentationClock) -> HRESULT {
    let mut object: *mut presentation_clock = core::ptr::null_mut();

    object = calloc(1, (core::mem::size_of::<presentation_clock>()) as _) as *mut presentation_clock;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFPresentationClock_iface.lpVtbl = core::ptr::addr_of!(presentationclockvtbl);
    (*object).IMFRateControl_iface.lpVtbl = core::ptr::addr_of!(presentclockratecontrolvtbl);
    (*object).IMFTimer_iface.lpVtbl = core::ptr::addr_of!(presentclocktimervtbl);
    (*object).IMFShutdown_iface.lpVtbl = core::ptr::addr_of!(presentclockshutdownvtbl);
    (*object).sink_callback.lpVtbl = core::ptr::addr_of!(presentclocksinkcallbackvtbl);
    (*object).refcount = 1;
    list_init(core::ptr::addr_of_mut!((*object).sinks));
    list_init(core::ptr::addr_of_mut!((*object).timers));
    (*object).rate = 1.0f32;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    *clock = core::ptr::addr_of_mut!((*object).IMFPresentationClock_iface) as *mut IMFPresentationClock;

    return S_OK;
}

// COM vtable tables — generated from clock.c by vtbl_gen.py
/// clock.c:176 `static const IUnknownVtbl sinknotificationvtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static sinknotificationvtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(sink_notification_QueryInterface),
    AddRef: Some(sink_notification_AddRef),
    Release: Some(sink_notification_Release),
};

/// clock.c:711 `static const IMFPresentationClockVtbl presentationclockvtbl`  (slot order: rust:types_amd64.rs)
pub static presentationclockvtbl: IMFPresentationClockVtbl = IMFPresentationClockVtbl {
    QueryInterface: Some(present_clock_QueryInterface),
    AddRef: Some(present_clock_AddRef),
    Release: Some(present_clock_Release),
    GetClockCharacteristics: Some(present_clock_GetClockCharacteristics),
    GetCorrelatedTime: Some(present_clock_GetCorrelatedTime),
    GetContinuityKey: Some(present_clock_GetContinuityKey),
    GetState: Some(present_clock_GetState),
    GetProperties: Some(present_clock_GetProperties),
    SetTimeSource: Some(present_clock_SetTimeSource),
    GetTimeSource: Some(present_clock_GetTimeSource),
    GetTime: Some(present_clock_GetTime),
    AddClockStateSink: Some(present_clock_AddClockStateSink),
    RemoveClockStateSink: Some(present_clock_RemoveClockStateSink),
    Start: Some(present_clock_Start),
    Stop: Some(present_clock_Stop),
    Pause: Some(present_clock_Pause),
};

/// clock.c:788 `static const IMFRateControlVtbl presentclockratecontrolvtbl`  (slot order: rust:types_amd64.rs)
pub static presentclockratecontrolvtbl: IMFRateControlVtbl = IMFRateControlVtbl {
    QueryInterface: Some(present_clock_rate_control_QueryInterface),
    AddRef: Some(present_clock_rate_control_AddRef),
    Release: Some(present_clock_rate_control_Release),
    SetRate: Some(present_clock_rate_SetRate),
    GetRate: Some(present_clock_rate_GetRate),
};

/// clock.c:872 `static const IUnknownVtbl clock_timer_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static clock_timer_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(clock_timer_QueryInterface),
    AddRef: Some(clock_timer_AddRef),
    Release: Some(clock_timer_Release),
};

/// clock.c:953 `static const IMFTimerVtbl presentclocktimervtbl`  (slot order: rust:types_amd64.rs)
pub static presentclocktimervtbl: IMFTimerVtbl = IMFTimerVtbl {
    QueryInterface: Some(present_clock_timer_QueryInterface),
    AddRef: Some(present_clock_timer_AddRef),
    Release: Some(present_clock_timer_Release),
    SetTimer: Some(present_clock_timer_SetTimer),
    CancelTimer: Some(present_clock_timer_CancelTimer),
};

/// clock.c:1018 `static const IMFShutdownVtbl presentclockshutdownvtbl`  (slot order: rust:types_amd64.rs)
pub static presentclockshutdownvtbl: IMFShutdownVtbl = IMFShutdownVtbl {
    QueryInterface: Some(present_clock_shutdown_QueryInterface),
    AddRef: Some(present_clock_shutdown_AddRef),
    Release: Some(present_clock_shutdown_Release),
    Shutdown: Some(present_clock_shutdown_Shutdown),
    GetShutdownStatus: Some(present_clock_shutdown_GetShutdownStatus),
};

/// clock.c:1079 `static const IMFAsyncCallbackVtbl presentclocksinkcallbackvtbl`  (slot order: rust:types_amd64.rs)
pub static presentclocksinkcallbackvtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(present_clock_callback_QueryInterface),
    AddRef: Some(present_clock_sink_callback_AddRef),
    Release: Some(present_clock_sink_callback_Release),
    GetParameters: Some(present_clock_callback_GetParameters),
    Invoke: Some(present_clock_sink_callback_Invoke),
};
