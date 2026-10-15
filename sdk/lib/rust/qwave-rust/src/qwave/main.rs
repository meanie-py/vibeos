use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut qos_handles: *mut qos_handle = ::core::ptr::null::<qos_handle>() as *mut qos_handle;

static mut qos_lock: LONG = 0;

// WINE_DEFAULT_DEBUG_CHANNEL(qwave);

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct qos_handle {
    pub next: *mut qos_handle,
}

unsafe extern "C" fn lock_handles() {
    while InterlockedCompareExchange(core::ptr::addr_of_mut!(qos_lock), 1, 0) != 0 {
        SwitchToThread();
    }
}

unsafe extern "C" fn unlock_handles() {
    InterlockedExchange(core::ptr::addr_of_mut!(qos_lock) as *mut LONG, 0);
}

unsafe extern "C" fn is_valid_handle(handle: HANDLE) -> BOOL {
    let mut qos: *mut qos_handle = core::ptr::null_mut();
    lock_handles();
    qos = qos_handles;
    while !qos.is_null() {
        if qos as HANDLE == handle {
            break;
        }
        qos = (*qos).next;
    }
    unlock_handles();
    return (qos != core::ptr::null_mut()) as BOOL;
}

unsafe extern "C" fn flow_unsupported(handle: HANDLE) -> BOOL {
    SetLastError((if is_valid_handle(handle) != 0 { ERROR_NOT_SUPPORTED } else { ERROR_INVALID_HANDLE }) as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QOSCreateHandle(version: PQOS_VERSION, handle: PHANDLE) -> BOOL {
    let mut qos: *mut qos_handle = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/qwave/main.c\0".as_ptr() as *const i8,
            b"QOSCreateHandle\0".as_ptr() as *const i8,
            75i32,
            b"%p %p\n\0".as_ptr() as *const i8,
            version,
            handle,
        );
    }
    if version.is_null()
        || !(((*version).MajorVersion as i32 == 1)
            && ((*version).MinorVersion as i32 == 0))
        || handle.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    qos = malloc(core::mem::size_of::<qos_handle>() as size_t) as *mut qos_handle;
    if qos.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }
    lock_handles();
    (*qos).next = qos_handles;
    qos_handles = qos;
    unlock_handles();
    *handle = qos as HANDLE;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QOSAddSocketToFlow(
    mut handle: HANDLE,
    mut socket: SOCKET,
    mut addr: PSOCKADDR,
    mut traffictype: QOS_TRAFFIC_TYPE,
    mut flags: DWORD,
    mut flowid: PQOS_FLOWID,
) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/qwave/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"QOSAddSocketToFlow\0" as *const u8 as *const ::core::ffi::c_char,
            101 as ::core::ffi::c_int,
            b"%p, %Ix, %p, %d, 0x%08lx, %p stub!\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            socket,
            addr,
            traffictype as ::core::ffi::c_uint,
            flags,
            flowid,
        );
    }
    return flow_unsupported(handle);
}

#[no_mangle]
pub unsafe extern "system" fn QOSCloseHandle(handle: HANDLE) -> BOOL {
    let mut qos: *mut qos_handle = core::ptr::null_mut::<qos_handle>();
    let mut link: *mut *mut qos_handle = core::ptr::null_mut::<*mut qos_handle>();

    lock_handles();
    link = core::ptr::addr_of_mut!(qos_handles);
    loop {
        qos = *link;
        if qos.is_null() {
            break;
        }
        if qos as HANDLE == handle {
            *link = (*qos).next;
            break;
        } else {
            link = core::ptr::addr_of_mut!((*qos).next);
        }
    }
    unlock_handles();
    if qos.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }
    free(qos as *mut core::ffi::c_void);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QOSRemoveSocketFromFlow(handle: HANDLE, socket: SOCKET, flowid: QOS_FLOWID, flags: DWORD) -> BOOL {
    return flow_unsupported(handle);
}

#[no_mangle]
pub unsafe extern "system" fn QOSQueryFlow(
    handle: HANDLE,
    flowid: QOS_FLOWID,
    op: QOS_QUERY_FLOW,
    size: PULONG,
    buffer: PVOID,
    flags: DWORD,
    overlap: LPOVERLAPPED,
) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/qwave/main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"QOSQueryFlow\0" as *const u8 as *const ::core::ffi::c_char,
            133 as ::core::ffi::c_int,
            b"%p, %lu, %d, %p, %p, 0x%08lx, %p stub!\n\0" as *const u8
                as *const ::core::ffi::c_char,
            handle,
            flowid,
            op as ::core::ffi::c_uint,
            size,
            buffer,
            flags,
            overlap,
        );
    }
    return flow_unsupported(handle);
}

#[no_mangle]
pub unsafe extern "system" fn QOSSetFlow(handle: HANDLE, flowid: QOS_FLOWID, op: QOS_SET_FLOW, size: ULONG, buffer: PVOID, flags: DWORD, overlap: LPOVERLAPPED) -> BOOL {
    return flow_unsupported(handle);
}
