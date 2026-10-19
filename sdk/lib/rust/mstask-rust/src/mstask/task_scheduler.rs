use crate::mstask::task::TaskConstructor;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
/// WINE_DEFAULT_DEBUG_CHANNEL(mstask): default debug channel name for this file.

#[repr(C)]
pub struct TaskSchedulerImpl {
    pub ITaskScheduler_iface: ITaskScheduler,
    pub r#ref: LONG,
}
pub type PTaskSchedulerImpl = *mut TaskSchedulerImpl;

#[repr(C)]
pub struct EnumWorkItemsImpl {
    pub IEnumWorkItems_iface: IEnumWorkItems,
    pub r#ref: LONG,
}
pub type PEnumWorkItemsImpl = *mut EnumWorkItemsImpl;

unsafe extern "C" fn impl_from_ITaskScheduler(iface: *mut ITaskScheduler) -> *mut TaskSchedulerImpl {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(TaskSchedulerImpl, ITaskScheduler_iface)) as *mut TaskSchedulerImpl;
}

unsafe extern "C" fn impl_from_IEnumWorkItems(iface: *mut IEnumWorkItems) -> *mut EnumWorkItemsImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(EnumWorkItemsImpl, IEnumWorkItems_iface)) as *mut EnumWorkItemsImpl;
}

unsafe extern "C" fn TaskSchedulerDestructor(This: *mut TaskSchedulerImpl) {
    HeapFree(GetProcessHeap(), 0, This as LPVOID);
    InterlockedDecrement(core::ptr::addr_of_mut!(dll_ref));
}

unsafe extern "system" fn EnumWorkItems_QueryInterface(
    iface: *mut IEnumWorkItems,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IEnumWorkItems))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        *obj = core::ptr::addr_of_mut!((*This).IEnumWorkItems_iface) as *mut core::ffi::c_void;
        IEnumWorkItems_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn EnumWorkItems_AddRef(iface: *mut IEnumWorkItems) -> ULONG {
    let This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    let ref_0: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "system" fn EnumWorkItems_Release(iface: *mut IEnumWorkItems) -> ULONG {
    let mut This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    let mut ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut LONG) as ULONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumWorkItems_Release\0" as *const u8 as *const ::core::ffi::c_char,
            84 as ::core::ffi::c_int,
            b"(%p)->(%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            ref_0,
        );
    }

    if ref_0 == 0 {
        HeapFree(GetProcessHeap(), 0 as DWORD, This as PVOID);
        InterlockedDecrement(core::ptr::addr_of_mut!(dll_ref) as *mut LONG);
    }

    return ref_0;
}
unsafe extern "system" fn EnumWorkItems_Next(
    iface: *mut IEnumWorkItems,
    count: ULONG,
    names: *mut *mut LPWSTR,
    fetched: *mut ULONG,
) -> HRESULT {
    let This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumWorkItems_Next\0" as *const u8 as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            b"(%p)->(%u %p %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            count,
            names,
            fetched,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn EnumWorkItems_Skip(iface: *mut IEnumWorkItems, count: ULONG) -> HRESULT {
    let mut This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumWorkItems_Skip\0" as *const u8 as *const ::core::ffi::c_char,
            105 as ::core::ffi::c_int,
            b"(%p)->(%u): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            This,
            count,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn EnumWorkItems_Reset(iface: *mut IEnumWorkItems) -> HRESULT {
    let mut This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const i8,
            b"EnumWorkItems_Reset\0" as *const u8 as *const i8,
            112i32,
            b"(%p): stub\n\0" as *const u8 as *const i8,
            This,
        );
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn EnumWorkItems_Clone(
    iface: *mut IEnumWorkItems,
    cloned: *mut *mut IEnumWorkItems,
) -> HRESULT {
    let mut This: *mut EnumWorkItemsImpl = impl_from_IEnumWorkItems(iface);
    return E_NOTIMPL;
}

unsafe extern "C" fn create_task_enum(ret: *mut *mut IEnumWorkItems) -> HRESULT {
    let mut tasks: *mut EnumWorkItemsImpl = core::ptr::null_mut::<EnumWorkItemsImpl>();

    *ret = core::ptr::null_mut::<IEnumWorkItems>();

    tasks = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<EnumWorkItemsImpl>() as SIZE_T,
    ) as *mut EnumWorkItemsImpl;

    if tasks.is_null() {
        return E_OUTOFMEMORY;
    }

    (*tasks).IEnumWorkItems_iface.lpVtbl = core::ptr::addr_of!(EnumWorkItemsVtbl);
    (*tasks).r#ref = 1 as LONG;

    *ret = core::ptr::addr_of_mut!((*tasks).IEnumWorkItems_iface);
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_ref) as *mut ::core::ffi::c_long);
    return S_OK;
}
unsafe extern "system" fn MSTASK_ITaskScheduler_QueryInterface(
    mut iface: *mut ITaskScheduler,
    riid: REFIID,
    mut ppvObject: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MSTASK_ITaskScheduler_QueryInterface\0" as *const u8
                as *const ::core::ffi::c_char,
            158 as ::core::ffi::c_int,
            b"IID: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(riid as *const _GUID),
        );
    }

    if IsEqualGUID!(riid, &raw const IID_IUnknown)
        || IsEqualGUID!(riid, &raw const IID_ITaskScheduler)
    {
        *ppvObject = &raw mut (*This).ITaskScheduler_iface as *mut ::core::ffi::c_void;
        ITaskScheduler_AddRef(iface);
        return S_OK;
    }

    *ppvObject = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn MSTASK_ITaskScheduler_AddRef(mut iface: *mut ITaskScheduler) -> ULONG {
    let mut This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MSTASK_ITaskScheduler_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            176 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).r#ref) as *mut LONG,
    ) as ULONG;
}

unsafe extern "system" fn MSTASK_ITaskScheduler_Release(iface: *mut ITaskScheduler) -> ULONG {
    let mut This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    let mut ref_0: ULONG = 0;
    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        TaskSchedulerDestructor(This);
    }
    return ref_0;
}

unsafe extern "system" fn MSTASK_ITaskScheduler_SetTargetComputer(
    iface: *mut ITaskScheduler,
    pwszComputer: LPCWSTR,
) -> HRESULT {
    let This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    let mut buffer: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 3) as _] = [0; (MAX_COMPUTERNAME_LENGTH + 3) as _];
    let mut len: DWORD = MAX_COMPUTERNAME_LENGTH + 1;

    /* NULL is an alias for the local computer */
    if pwszComputer.is_null() {
        return S_OK;
    }

    buffer[0 as usize] = '\\' as u16;
    buffer[1 as usize] = '\\' as u16;
    if GetComputerNameW(
        (core::ptr::addr_of_mut!(buffer) as *mut WCHAR).offset(2 as isize),
        core::ptr::addr_of_mut!(len),
    ) != 0
    {
        if lstrcmpiW(
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPCWSTR,
            pwszComputer,
        ) == 0
            || lstrcmpiW(
                (core::ptr::addr_of_mut!(buffer) as *mut WCHAR).offset(2 as isize) as LPCWSTR,
                pwszComputer,
            ) == 0
        {
            return S_OK;
        }
    }

    return HRESULT_FROM_WIN32!(ERROR_BAD_NETPATH);
}

unsafe extern "system" fn MSTASK_ITaskScheduler_GetTargetComputer(
    iface: *mut ITaskScheduler,
    ppwszComputer: *mut LPWSTR,
) -> HRESULT {
    let mut This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    let mut buffer: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = (MAX_COMPUTERNAME_LENGTH + 1) as DWORD;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task_scheduler.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"MSTASK_ITaskScheduler_GetTargetComputer\0" as *const u8
                as *const core::ffi::c_char,
            227 as core::ffi::c_int,
            b"(%p)->(%p)\n\0" as *const u8 as *const core::ffi::c_char,
            This,
            ppwszComputer,
        );
    }

    if ppwszComputer.is_null() {
        return E_INVALIDARG;
    }

    buffer = CoTaskMemAlloc(
        ((MAX_COMPUTERNAME_LENGTH + 3) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if !buffer.is_null() {
        *buffer.offset(0 as isize) = '\\' as u16;
        *buffer.offset(1 as isize) = '\\' as u16;
        if GetComputerNameW(
            buffer.offset(2 as isize),
            core::ptr::addr_of_mut!(len),
        ) != 0
        {
            *ppwszComputer = buffer;
            return S_OK;
        }
        CoTaskMemFree(buffer as LPVOID);
    }
    *ppwszComputer = core::ptr::null_mut();
    let err: DWORD = GetLastError();
    return HRESULT_FROM_WIN32!(err);
}

unsafe extern "system" fn MSTASK_ITaskScheduler_Enum(
    iface: *mut ITaskScheduler,
    tasks: *mut *mut IEnumWorkItems,
) -> HRESULT {
    let This: *mut TaskSchedulerImpl = impl_from_ITaskScheduler(iface);
    TRACE!("(%p)->(%p)\n", This, tasks);

    if tasks.is_null() {
        return E_INVALIDARG;
    }

    return create_task_enum(tasks);
}
unsafe extern "system" fn MSTASK_ITaskScheduler_Activate(
    iface: *mut ITaskScheduler,
    pwszName: LPCWSTR,
    riid: REFIID,
    ppunk: *mut *mut IUnknown,
) -> HRESULT {
    // TRACE("%p, %s, %s, %p: stub\n", iface, debugstr_w(pwszName), debugstr_guid(riid), ppunk);
    // FIXME("Partial stub always returning COR_E_FILENOTFOUND\n");
    COR_E_FILENOTFOUND
}
unsafe extern "system" fn MSTASK_ITaskScheduler_Delete(
    iface: *mut ITaskScheduler,
    pwszName: LPCWSTR,
) -> HRESULT {
    // FIXME("%p, %s: stub\n", iface, debugstr_w(pwszName));
    E_NOTIMPL
}

unsafe extern "system" fn MSTASK_ITaskScheduler_NewWorkItem(
    iface: *mut ITaskScheduler,
    pwszTaskName: LPCWSTR,
    rclsid: REFCLSID,
    riid: REFIID,
    ppunk: *mut *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT;

    if IsEqualGUID!(rclsid, core::ptr::addr_of!(CLSID_CTask)) == false {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_ITask)) == false {
        return E_NOINTERFACE;
    }

    hr = TaskConstructor(pwszTaskName, ppunk as *mut LPVOID);
    return hr;
}
unsafe extern "system" fn MSTASK_ITaskScheduler_AddWorkItem(
    iface: *mut ITaskScheduler,
    pwszTaskName: LPCWSTR,
    pWorkItem: *mut IScheduledWorkItem,
) -> HRESULT {
    // FIXME("%p, %s, %p: stub\n", iface, debugstr_w(pwszTaskName), pWorkItem);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITaskScheduler_IsOfType(
    iface: *mut ITaskScheduler,
    pwszName: LPCWSTR,
    riid: REFIID,
) -> HRESULT {
    // FIXME("%p, %s, %s: stub\n", iface, debugstr_w(pwszName), debugstr_guid(riid));
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn TaskSchedulerConstructor(ppObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut TaskSchedulerImpl = core::ptr::null_mut::<TaskSchedulerImpl>();
    This = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<TaskSchedulerImpl>() as SIZE_T,
    ) as *mut TaskSchedulerImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }
    (*This).ITaskScheduler_iface.lpVtbl = &raw const MSTASK_ITaskSchedulerVtbl;
    (*This).r#ref = 1 as LONG;

    *ppObj = &raw mut (*This).ITaskScheduler_iface as LPVOID;
    InterlockedIncrement(&raw mut dll_ref as *mut ::core::ffi::c_long);
    return S_OK;
}

// COM vtable tables — generated from task_scheduler.c by vtbl_gen.py
/// task_scheduler.c:123 `static const IEnumWorkItemsVtbl EnumWorkItemsVtbl`  (slot order: rust:types_amd64.rs)
pub static EnumWorkItemsVtbl: IEnumWorkItemsVtbl = IEnumWorkItemsVtbl {
    QueryInterface: Some(EnumWorkItems_QueryInterface),
    AddRef: Some(EnumWorkItems_AddRef),
    Release: Some(EnumWorkItems_Release),
    Next: Some(EnumWorkItems_Next),
    Skip: Some(EnumWorkItems_Skip),
    Reset: Some(EnumWorkItems_Reset),
    Clone: Some(EnumWorkItems_Clone),
};

/// task_scheduler.c:323 `static const ITaskSchedulerVtbl MSTASK_ITaskSchedulerVtbl`  (slot order: rust:types_amd64.rs)
pub static MSTASK_ITaskSchedulerVtbl: ITaskSchedulerVtbl = ITaskSchedulerVtbl {
    QueryInterface: Some(MSTASK_ITaskScheduler_QueryInterface),
    AddRef: Some(MSTASK_ITaskScheduler_AddRef),
    Release: Some(MSTASK_ITaskScheduler_Release),
    SetTargetComputer: Some(MSTASK_ITaskScheduler_SetTargetComputer),
    GetTargetComputer: Some(MSTASK_ITaskScheduler_GetTargetComputer),
    Enum: Some(MSTASK_ITaskScheduler_Enum),
    Activate: Some(MSTASK_ITaskScheduler_Activate),
    Delete: Some(MSTASK_ITaskScheduler_Delete),
    NewWorkItem: Some(MSTASK_ITaskScheduler_NewWorkItem),
    AddWorkItem: Some(MSTASK_ITaskScheduler_AddWorkItem),
    IsOfType: Some(MSTASK_ITaskScheduler_IsOfType),
};
