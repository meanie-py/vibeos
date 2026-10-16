use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug channel registration, macro-generated.
// Channel name kept for reference; the logging macros take it from the crate.
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &[u8] = b"mfplat\0";

pub const QUALITY_MANAGER_READY: i32 = 0;
pub const QUALITY_MANAGER_SHUT_DOWN: i32 = 1;

#[repr(C)]
pub struct quality_manager {
    pub IMFQualityManager_iface: IMFQualityManager,
    pub IMFClockStateSink_iface: IMFClockStateSink,
    pub refcount: LONG,
    pub topology: *mut IMFTopology,
    pub clock: *mut IMFPresentationClock,
    pub state: u32,
    pub cs: CRITICAL_SECTION,
}

unsafe extern "C" fn impl_from_IMFQualityManager(iface: *mut IMFQualityManager) -> *mut quality_manager {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(quality_manager, IMFQualityManager_iface)) as *mut quality_manager;
}

unsafe extern "C" fn impl_from_IMFClockStateSink(iface: *mut IMFClockStateSink) -> *mut quality_manager {
    return (iface as *mut u8).wrapping_sub(
        core::mem::offset_of!(quality_manager, IMFClockStateSink_iface),
    ) as *mut quality_manager;
}

unsafe extern "system" fn standard_quality_manager_QueryInterface(iface: *mut IMFQualityManager, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let manager: *mut quality_manager = impl_from_IMFQualityManager(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFQualityManager)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFClockStateSink)) != 0 {
        *out = core::ptr::addr_of_mut!((*manager).IMFClockStateSink_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unsupported %s.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn standard_quality_manager_AddRef(iface: *mut IMFQualityManager) -> ULONG {
    let mut manager: *mut quality_manager = impl_from_IMFQualityManager(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*manager).refcount) as *mut ::core::ffi::c_long,
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
            b"dll/win32/mf/quality.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"standard_quality_manager_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            89 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn standard_quality_manager_Release(mut iface: *mut IMFQualityManager) -> ULONG {
    let mut manager: *mut quality_manager = impl_from_IMFQualityManager(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        &raw mut (*manager).refcount as *mut ::core::ffi::c_long,
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
            b"dll/win32/mf/quality.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"standard_quality_manager_Release\0" as *const u8 as *const ::core::ffi::c_char,
            99 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        if !(*manager).clock.is_null() {
            IMFPresentationClock_Release((*manager).clock);
        }
        if !(*manager).topology.is_null() {
            IMFTopology_Release((*manager).topology);
        }
        DeleteCriticalSection(&raw mut (*manager).cs);
        free(manager as *mut ::core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "C" fn standard_quality_manager_set_topology(manager: *mut quality_manager, topology: *mut IMFTopology) {
    if !(*manager).topology.is_null() {
        IMFTopology_Release((*manager).topology);
    }
    (*manager).topology = topology;
    if !(*manager).topology.is_null() {
        IMFTopology_AddRef((*manager).topology);
    }
}

unsafe extern "system" fn standard_quality_manager_NotifyTopology(iface: *mut IMFQualityManager, topology: *mut IMFTopology) -> HRESULT {
    let manager: *mut quality_manager = impl_from_IMFQualityManager(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));
    if (*manager).state == (QUALITY_MANAGER_SHUT_DOWN as u32) {
        hr = MF_E_SHUTDOWN;
    } else {
        standard_quality_manager_set_topology(manager, topology);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    return hr;
}

unsafe extern "C" fn standard_quality_manager_release_clock(manager: *mut quality_manager) {
    if !(*manager).clock.is_null() {
        IMFPresentationClock_RemoveClockStateSink(
            (*manager).clock,
            core::ptr::addr_of_mut!((*manager).IMFClockStateSink_iface),
        );
        IMFPresentationClock_Release((*manager).clock);
    }
    (*manager).clock = core::ptr::null_mut();
}

unsafe extern "system" fn standard_quality_manager_NotifyPresentationClock(
    iface: *mut IMFQualityManager,
    clock: *mut IMFPresentationClock,
) -> HRESULT {
    let manager: *mut quality_manager = impl_from_IMFQualityManager(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %p.\n", iface, clock);

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));
    if (*manager).state == (QUALITY_MANAGER_SHUT_DOWN as u32) {
        hr = MF_E_SHUTDOWN;
    } else if clock.is_null() {
        hr = E_POINTER;
    } else {
        standard_quality_manager_release_clock(manager);
        (*manager).clock = clock;
        IMFPresentationClock_AddRef((*manager).clock);
        if IMFPresentationClock_AddClockStateSink(
            (*manager).clock,
            core::ptr::addr_of_mut!((*manager).IMFClockStateSink_iface),
        ) < 0
        {
            WARN!("Failed to set state sink.\n");
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    return hr;
}
unsafe extern "system" fn standard_quality_manager_NotifyProcessInput(
    iface: *mut IMFQualityManager,
    node: *mut IMFTopologyNode,
    input_index: LONG,
    sample: *mut IMFSample,
) -> HRESULT {
    // TRACE("%p, %p, %ld, %p stub.\n", iface, node, input_index, sample);
    E_NOTIMPL
}
unsafe extern "system" fn standard_quality_manager_NotifyProcessOutput(
    iface: *mut IMFQualityManager,
    node: *mut IMFTopologyNode,
    output_index: LONG,
    sample: *mut IMFSample,
) -> HRESULT {
    // TRACE("%p, %p, %ld, %p stub.\n", iface, node, output_index, sample);
    E_NOTIMPL
}
unsafe extern "system" fn standard_quality_manager_NotifyQualityEvent(
    iface: *mut IMFQualityManager,
    object: *mut IUnknown,
    event: *mut IMFMediaEvent,
) -> HRESULT {
    // FIXME("%p, %p, %p stub.\n", iface, object, event);
    E_NOTIMPL
}

unsafe extern "system" fn standard_quality_manager_Shutdown(iface: *mut IMFQualityManager) -> HRESULT {
    let mut manager: *mut quality_manager = impl_from_IMFQualityManager(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*manager).cs));
    if (*manager).state != (QUALITY_MANAGER_SHUT_DOWN as u32) {
        standard_quality_manager_release_clock(manager);
        standard_quality_manager_set_topology(manager, core::ptr::null_mut());
        (*manager).state = (QUALITY_MANAGER_SHUT_DOWN as u32);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*manager).cs));

    return S_OK;
}

unsafe extern "system" fn standard_quality_manager_sink_QueryInterface(
    iface: *mut IMFClockStateSink,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let manager: *mut quality_manager = impl_from_IMFClockStateSink(iface);
    return IMFQualityManager_QueryInterface(core::ptr::addr_of_mut!((*manager).IMFQualityManager_iface), riid, obj);
}

unsafe extern "system" fn standard_quality_manager_sink_AddRef(iface: *mut IMFClockStateSink) -> ULONG {
    let manager: *mut quality_manager = impl_from_IMFClockStateSink(iface);
    return IMFQualityManager_AddRef(core::ptr::addr_of_mut!((*manager).IMFQualityManager_iface));
}

unsafe extern "system" fn standard_quality_manager_sink_Release(iface: *mut IMFClockStateSink) -> ULONG {
    let mut manager: *mut quality_manager = impl_from_IMFClockStateSink(iface);
    return IMFQualityManager_Release(core::ptr::addr_of_mut!((*manager).IMFQualityManager_iface));
}
unsafe extern "system" fn standard_quality_manager_sink_OnClockStart(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
    offset: LONGLONG,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn standard_quality_manager_sink_OnClockStop(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn standard_quality_manager_sink_OnClockPause(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn standard_quality_manager_sink_OnClockRestart(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn standard_quality_manager_sink_OnClockSetRate(
    iface: *mut IMFClockStateSink,
    systime: MFTIME,
    rate: f32,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateStandardQualityManager(manager: *mut *mut IMFQualityManager) -> HRESULT {
    let mut object: *mut quality_manager = core::ptr::null_mut();

    object = calloc(1, (core::mem::size_of::<quality_manager>()) as _) as *mut quality_manager;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFQualityManager_iface.lpVtbl = core::ptr::addr_of!(standard_quality_manager_vtbl);
    (*object).IMFClockStateSink_iface.lpVtbl = core::ptr::addr_of!(standard_quality_manager_sink_vtbl);
    (*object).refcount = 1;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*object).cs));

    *manager = core::ptr::addr_of_mut!((*object).IMFQualityManager_iface);

    return S_OK;
}

// COM vtable tables — generated from quality.c by vtbl_gen.py
/// quality.c:220 `static const IMFQualityManagerVtbl standard_quality_manager_vtbl`  (slot order: rust:types_amd64.rs)
pub static standard_quality_manager_vtbl: IMFQualityManagerVtbl = IMFQualityManagerVtbl {
    QueryInterface: Some(standard_quality_manager_QueryInterface),
    AddRef: Some(standard_quality_manager_AddRef),
    Release: Some(standard_quality_manager_Release),
    NotifyTopology: Some(standard_quality_manager_NotifyTopology),
    NotifyPresentationClock: Some(standard_quality_manager_NotifyPresentationClock),
    NotifyProcessInput: Some(standard_quality_manager_NotifyProcessInput),
    NotifyProcessOutput: Some(standard_quality_manager_NotifyProcessOutput),
    NotifyQualityEvent: Some(standard_quality_manager_NotifyQualityEvent),
    Shutdown: Some(standard_quality_manager_Shutdown),
};

/// quality.c:282 `static const IMFClockStateSinkVtbl standard_quality_manager_sink_vtbl`  (slot order: rust:types_amd64.rs)
pub static standard_quality_manager_sink_vtbl: IMFClockStateSinkVtbl = IMFClockStateSinkVtbl {
    QueryInterface: Some(standard_quality_manager_sink_QueryInterface),
    AddRef: Some(standard_quality_manager_sink_AddRef),
    Release: Some(standard_quality_manager_sink_Release),
    OnClockStart: Some(standard_quality_manager_sink_OnClockStart),
    OnClockStop: Some(standard_quality_manager_sink_OnClockStop),
    OnClockPause: Some(standard_quality_manager_sink_OnClockPause),
    OnClockRestart: Some(standard_quality_manager_sink_OnClockRestart),
    OnClockSetRate: Some(standard_quality_manager_sink_OnClockSetRate),
};
