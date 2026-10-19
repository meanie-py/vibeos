use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mstask): debug channel macro, handled by the DPRINT*!/ERR!/WARN! macros.

#[repr(C)]
pub struct TaskImpl {
    pub ITask_iface: ITask,
    pub IPersistFile_iface: IPersistFile,
    pub r#ref: i32,
    pub taskName: LPWSTR,
    pub applicationName: LPWSTR,
    pub parameters: LPWSTR,
    pub comment: LPWSTR,
    pub maxRunTime: u32,
    pub accountName: LPWSTR,
}
unsafe extern "C" fn impl_from_ITask(iface: *mut ITask) -> *mut TaskImpl {
    return CONTAINING_RECORD!(iface, TaskImpl, ITask_iface);
}

unsafe extern "C" fn impl_from_IPersistFile(iface: *mut IPersistFile) -> *mut TaskImpl {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(TaskImpl, IPersistFile_iface)) as *mut TaskImpl;
}

unsafe extern "C" fn TaskDestructor(mut This: *mut TaskImpl) {
    TRACE!("%p\n", This);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).accountName as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).comment as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).parameters as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, (*This).taskName as LPVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, This as LPVOID);
    InterlockedDecrement(core::ptr::addr_of_mut!(dll_ref) as *mut core::ffi::c_long);
}

unsafe extern "system" fn MSTASK_ITask_QueryInterface(
    iface: *mut ITask,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut This: *mut TaskImpl = impl_from_ITask(iface);

    if ppvObject.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID!(riid, &raw const IID_IUnknown) != false
        || IsEqualGUID!(riid, &raw const IID_ITask) != false
    {
        *ppvObject = &raw mut (*This).ITask_iface as *mut core::ffi::c_void;
        ITask_AddRef(iface);
        return S_OK;
    } else if IsEqualGUID!(riid, &raw const IID_IPersistFile) != false {
        *ppvObject = &raw mut (*This).IPersistFile_iface as *mut core::ffi::c_void;
        ITask_AddRef(iface);
        return S_OK;
    }

    WARN!("Unknown interface: %s\n", debugstr_guid(riid));
    *ppvObject = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn MSTASK_ITask_AddRef(iface: *mut ITask) -> ULONG {
    let This: *mut TaskImpl = impl_from_ITask(iface);
    let mut ref_0: ULONG;
    ref_0 = InterlockedIncrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut LONG) as ULONG;
    return ref_0;
}

unsafe extern "system" fn MSTASK_ITask_Release(
        iface: *mut ITask) -> ULONG {
    let mut This: *mut TaskImpl = impl_from_ITask(iface);
    let mut ref_0: ULONG;
    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut core::ffi::c_long) as ULONG;
    if ref_0 == 0 {
        TaskDestructor(This);
    }
    return ref_0;
}
unsafe extern "system" fn MSTASK_ITask_CreateTrigger(
    mut iface: *mut ITask,
    mut piNewTrigger: *mut WORD,
    mut ppTrigger: *mut *mut ITaskTrigger,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"MSTASK_ITask_CreateTrigger\0" as *const u8 as *const ::core::ffi::c_char,
            115 as ::core::ffi::c_int,
            b"(%p, %p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            piNewTrigger,
            ppTrigger,
        );
    }
    return TaskTriggerConstructor(ppTrigger as *mut LPVOID);
}
unsafe extern "system" fn MSTASK_ITask_DeleteTrigger(
    iface: *mut ITask,
    iTrigger: WORD,
) -> HRESULT {
    // FIXME("(%p, %d): stub\n", iface, iTrigger);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetTriggerCount(
    iface: *mut ITask,
    plCount: *mut WORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, plCount);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetTrigger(
    iface: *mut ITask,
    iTrigger: WORD,
    ppTrigger: *mut *mut ITaskTrigger,
) -> HRESULT {
    // FIXME("(%p, %d, %p): stub\n", iface, iTrigger, ppTrigger);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetTriggerString(
    iface: *mut ITask,
    iTrigger: WORD,
    ppwszTrigger: *mut LPWSTR,
) -> HRESULT {
    // FIXME("(%p, %d, %p): stub\n", iface, iTrigger, ppwszTrigger);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetRunTimes(
    iface: *mut ITask,
    pstBegin: LPSYSTEMTIME,
    pstEnd: LPSYSTEMTIME,
    pCount: *mut WORD,
    rgstTaskTimes: *mut LPSYSTEMTIME,
) -> HRESULT {
    // FIXME("(%p, %p, %p, %p, %p): stub\n", iface, pstBegin, pstEnd, pCount, rgstTaskTimes);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetNextRunTime(
    iface: *mut ITask,
    pstNextRun: *mut SYSTEMTIME,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pstNextRun);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetIdleWait(
    iface: *mut ITask,
    wIdleMinutes: WORD,
    wDeadlineMinutes: WORD,
) -> HRESULT {
    // FIXME("(%p, %d, %d): stub\n", iface, wIdleMinutes, wDeadlineMinutes);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetIdleWait(
    iface: *mut ITask,
    pwIdleMinutes: *mut WORD,
    pwDeadlineMinutes: *mut WORD,
) -> HRESULT {
    // FIXME("(%p, %p, %p): stub\n", iface, pwIdleMinutes, pwDeadlineMinutes);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_Run(iface: *mut ITask) -> HRESULT {
    // FIXME("(%p): stub\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_Terminate(iface: *mut ITask) -> HRESULT {
    // FIXME("(%p): stub\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_EditWorkItem(
    iface: *mut ITask,
    hParent: HWND,
    dwReserved: DWORD,
) -> HRESULT {
    // FIXME("(%p, %p, %d): stub\n", iface, hParent, dwReserved);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetMostRecentRunTime(
    iface: *mut ITask,
    pstLastRun: *mut SYSTEMTIME,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pstLastRun);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetStatus(
    iface: *mut ITask,
    phrStatus: *mut HRESULT,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, phrStatus);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetExitCode(
    iface: *mut ITask,
    pdwExitCode: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pdwExitCode);
    E_NOTIMPL
}

unsafe extern "system" fn MSTASK_ITask_SetComment(
    iface: *mut ITask,
    pwszComment: LPCWSTR,
) -> HRESULT {
    let mut n: DWORD;
    let This: *mut TaskImpl = impl_from_ITask(iface);
    let mut tmp_comment: LPWSTR;

    /* Empty comment */
    if *pwszComment.offset(0 as isize) == (0) as WCHAR {
        HeapFree(GetProcessHeap(), (0) as DWORD, (*This).comment as PVOID);
        (*This).comment = core::ptr::null_mut();
        return S_OK;
    }

    /* Set to pwszComment */
    n = (lstrlenW(pwszComment) + (1) as i32) as DWORD;
    tmp_comment = HeapAlloc(
        GetProcessHeap(),
        (0) as DWORD,
        (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if tmp_comment.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(tmp_comment, pwszComment);
    HeapFree(GetProcessHeap(), (0) as DWORD, (*This).comment as PVOID);
    (*This).comment = tmp_comment;

    return S_OK;
}

unsafe extern "system" fn MSTASK_ITask_GetComment(
    iface: *mut ITask,
    ppwszComment: *mut LPWSTR,
) -> HRESULT {
    let mut n: DWORD;
    let This: *mut TaskImpl = impl_from_ITask(iface);

    n = if !(*This).comment.is_null() {
        (lstrlenW((*This).comment as LPCWSTR) + 1 as i32) as DWORD
    } else {
        1 as DWORD
    };
    *ppwszComment = CoTaskMemAlloc(
        (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if (*ppwszComment).is_null() {
        return E_OUTOFMEMORY as HRESULT;
    }

    if (*This).comment.is_null() {
        **ppwszComment.offset(0 as isize) = 0 as WCHAR;
    } else {
        lstrcpyW(*ppwszComment, (*This).comment as LPCWSTR);
    }

    return S_OK as HRESULT;
}
unsafe extern "system" fn MSTASK_ITask_SetCreator(
    iface: *mut ITask,
    pwszCreator: LPCWSTR,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pwszCreator);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetCreator(
    iface: *mut ITask,
    ppwszCreator: *mut LPWSTR,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, ppwszCreator);
    E_NOTIMPL
}

unsafe extern "system" fn MSTASK_ITask_SetWorkItemData(
    mut iface: *mut ITask,
    mut cBytes: WORD,
    mut rgbData: *mut BYTE,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_FIXME as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task.c\0"
                as *const u8 as *const i8,
            b"MSTASK_ITask_SetWorkItemData\0" as *const u8 as *const i8,
            311i32,
            b"(%p, %d, %p): stub\n\0" as *const u8 as *const i8,
            iface,
            cBytes as i32,
            rgbData,
        );
    }
    return E_NOTIMPL;
}
unsafe extern "system" fn MSTASK_ITask_GetWorkItemData(
    iface: *mut ITask,
    pcBytes: *mut WORD,
    ppBytes: *mut *mut BYTE,
) -> HRESULT {
    // FIXME("(%p, %p, %p): stub\n", iface, pcBytes, ppBytes);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetErrorRetryCount(
    iface: *mut ITask,
    wRetryCount: WORD,
) -> HRESULT {
    // FIXME("(%p, %d): stub\n", iface, wRetryCount);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetErrorRetryCount(
    iface: *mut ITask,
    pwRetryCount: *mut WORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pwRetryCount);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetErrorRetryInterval(
    iface: *mut ITask,
    wRetryInterval: WORD,
) -> HRESULT {
    // FIXME("(%p, %d): stub\n", iface, wRetryInterval);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetErrorRetryInterval(
    iface: *mut ITask,
    pwRetryInterval: *mut WORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pwRetryInterval);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetFlags(iface: *mut ITask, dwFlags: DWORD) -> HRESULT {
    // FIXME("(%p, 0x%08x): stub\n", iface, dwFlags);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetFlags(
    iface: *mut ITask,
    pdwFlags: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pdwFlags);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetAccountInformation(
    iface: *mut ITask,
    pwszAccountName: LPCWSTR,
    pwszPassword: LPCWSTR,
) -> HRESULT {
    let mut n: DWORD;
    let This: *mut TaskImpl = impl_from_ITask(iface);
    let mut tmp_account_name: LPWSTR;

    TRACE!(
        "(%p, %s, %s): partial stub\n",
        iface,
        debugstr_w(pwszAccountName as *const WCHAR),
        debugstr_w(pwszPassword as *const WCHAR)
    );

    if !pwszPassword.is_null() {
        FIXME!("Partial stub ignores passwords\n");
    }

    n = (lstrlenW(pwszAccountName) + 1) as DWORD;
    tmp_account_name = HeapAlloc(
        GetProcessHeap(),
        0,
        (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if tmp_account_name.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(tmp_account_name, pwszAccountName);
    HeapFree(
        GetProcessHeap(),
        0,
        (*This).accountName as PVOID,
    );
    (*This).accountName = tmp_account_name;
    return S_OK;
}
unsafe extern "system" fn MSTASK_ITask_GetAccountInformation(
        iface: *mut ITask,
        ppwszAccountName: *mut LPWSTR) -> HRESULT {
    let mut n: DWORD = 0;
    let This: *mut TaskImpl = impl_from_ITask(iface);

    /* This implements the WinXP behavior when accountName has not yet
     * set.  Win2K behaves differently, returning SCHED_E_CANNOT_OPEN_TASK */
    if (*This).accountName.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    }

    n = (lstrlenW((*This).accountName as LPCWSTR) + 1) as DWORD;
    *ppwszAccountName = CoTaskMemAlloc((n as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
    if (*ppwszAccountName).is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(*ppwszAccountName, (*This).accountName as LPCWSTR);
    return S_OK;
}

unsafe extern "system" fn MSTASK_ITask_SetApplicationName(
    iface: *mut ITask,
    pwszApplicationName: LPCWSTR,
) -> HRESULT {
    let mut n: DWORD = 0;
    let This: *mut TaskImpl = impl_from_ITask(iface);
    let mut tmp_name: LPWSTR = core::ptr::null_mut();

    /* Empty application name */
    if *pwszApplicationName.offset(0 as isize) == 0 {
        HeapFree(
            GetProcessHeap(),
            0,
            (*This).applicationName as LPVOID,
        );
        (*This).applicationName = core::ptr::null_mut();
        return S_OK;
    }

    /* Attempt to set pwszApplicationName to a path resolved application name */
    n = SearchPathW(
        core::ptr::null(),
        pwszApplicationName,
        core::ptr::null(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if n != 0 {
        tmp_name = HeapAlloc(
            GetProcessHeap(),
            0,
            (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if tmp_name.is_null() {
            return E_OUTOFMEMORY;
        }
        n = SearchPathW(
            core::ptr::null(),
            pwszApplicationName,
            core::ptr::null(),
            n,
            tmp_name,
            core::ptr::null_mut(),
        );
        if n != 0 {
            HeapFree(
                GetProcessHeap(),
                0,
                (*This).applicationName as LPVOID,
            );
            (*This).applicationName = tmp_name;
            return S_OK;
        } else {
            HeapFree(
                GetProcessHeap(),
                0,
                tmp_name as LPVOID,
            );
        }
    }

    /* If unable to path resolve name, simply set to pwszApplicationName */
    n = (lstrlenW(pwszApplicationName) + 1) as DWORD;
    tmp_name = HeapAlloc(
        GetProcessHeap(),
        0,
        (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if tmp_name.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(tmp_name, pwszApplicationName);
    HeapFree(
        GetProcessHeap(),
        0,
        (*This).applicationName as LPVOID,
    );
    (*This).applicationName = tmp_name;
    return S_OK;
}

unsafe extern "system" fn MSTASK_ITask_GetApplicationName(
    iface: *mut ITask,
    ppwszApplicationName: *mut LPWSTR,
) -> HRESULT {
    let mut n: DWORD;
    let mut This: *mut TaskImpl = impl_from_ITask(iface);

    n = if !(*This).applicationName.is_null() {
        (lstrlenW((*This).applicationName as LPCWSTR) as DWORD) + 1 as DWORD
    } else {
        1 as DWORD
    };
    *ppwszApplicationName =
        CoTaskMemAlloc((n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T))
            as LPWSTR;
    if (*ppwszApplicationName).is_null() {
        return E_OUTOFMEMORY;
    }

    if (*This).applicationName.is_null() {
        **ppwszApplicationName.offset(0 as isize) = 0 as WCHAR;
    } else {
        lstrcpyW(*ppwszApplicationName, (*This).applicationName as LPCWSTR);
    }

    return S_OK;
}
unsafe extern "system" fn MSTASK_ITask_SetParameters(
    mut iface: *mut ITask,
    mut pwszParameters: LPCWSTR,
) -> HRESULT {
    let mut n: DWORD = 0;
    let mut This: *mut TaskImpl = impl_from_ITask(iface);
    let mut tmp_parameters: LPWSTR = core::ptr::null_mut();

    /* Empty parameter list */
    if *pwszParameters.offset(0 as isize) == 0 {
        HeapFree(GetProcessHeap(), 0, (*This).parameters as PVOID);
        (*This).parameters = core::ptr::null_mut();
        return S_OK;
    }

    /* Set to pwszParameters */
    n = (lstrlenW(pwszParameters) + 1) as DWORD;
    tmp_parameters = HeapAlloc(
        GetProcessHeap(),
        0,
        (n as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if tmp_parameters.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(tmp_parameters, pwszParameters);
    HeapFree(GetProcessHeap(), 0, (*This).parameters as PVOID);
    (*This).parameters = tmp_parameters;
    return S_OK;
}

unsafe extern "system" fn MSTASK_ITask_GetParameters(
    mut iface: *mut ITask,
    mut ppwszParameters: *mut LPWSTR,
) -> HRESULT {
    let mut n: DWORD = 0;
    let mut This: *mut TaskImpl = impl_from_ITask(iface);

    n = (if !(*This).parameters.is_null() {
        lstrlenW((*This).parameters as LPCWSTR) + 1 as i32
    } else {
        1 as i32
    }) as DWORD;
    *ppwszParameters =
        CoTaskMemAlloc(((n as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T) as LPWSTR;
    if (*ppwszParameters).is_null() {
        return E_OUTOFMEMORY;
    }

    if (*This).parameters.is_null() {
        *(*ppwszParameters).add(0) = 0 as WCHAR;
    } else {
        lstrcpyW(*ppwszParameters, (*This).parameters as LPCWSTR);
    }

    return S_OK;
}
unsafe extern "system" fn MSTASK_ITask_SetWorkingDirectory(
    iface: *mut ITask,
    pwszWorkingDirectory: LPCWSTR,
) -> HRESULT {
    // FIXME("(%p, %s): stub\n", iface, debugstr_w(pwszWorkingDirectory));
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetWorkingDirectory(
    iface: *mut ITask,
    ppwszWorkingDirectory: *mut LPWSTR,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, ppwszWorkingDirectory);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetPriority(
    iface: *mut ITask,
    dwPriority: DWORD,
) -> HRESULT {
    // FIXME("(%p, 0x%08x): stub\n", iface, dwPriority);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetPriority(
    iface: *mut ITask,
    pdwPriority: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pdwPriority);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetTaskFlags(
    iface: *mut ITask,
    dwFlags: DWORD,
) -> HRESULT {
    // FIXME("(%p, 0x%08x): stub\n", iface, dwFlags);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_GetTaskFlags(
    iface: *mut ITask,
    pdwFlags: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pdwFlags);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_ITask_SetMaxRunTime(
        iface: *mut ITask,
        dwMaxRunTime: DWORD) -> HRESULT {
    let mut This: *mut TaskImpl = impl_from_ITask(iface);

    (*This).maxRunTime = dwMaxRunTime;
    return S_OK;
}

unsafe extern "system" fn MSTASK_ITask_GetMaxRunTime(
    iface: *mut ITask,
    pdwMaxRunTime: *mut DWORD,
) -> HRESULT {
    let mut This: *mut TaskImpl = impl_from_ITask(iface);

    *pdwMaxRunTime = (*This).maxRunTime;
    return S_OK;
}

unsafe extern "system" fn MSTASK_IPersistFile_QueryInterface(
    iface: *mut IPersistFile,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut TaskImpl = impl_from_IPersistFile(iface);
    TRACE!("(%p, %s, %p)\n", iface, debugstr_guid(riid as *const _GUID), ppvObject);
    return ITask_QueryInterface(core::ptr::addr_of_mut!((*This).ITask_iface), riid, ppvObject);
}

unsafe extern "system" fn MSTASK_IPersistFile_AddRef(iface: *mut IPersistFile) -> ULONG {
    let This: *mut TaskImpl = impl_from_IPersistFile(iface);
    let mut ref_: ULONG;
    ref_ = (InterlockedIncrement(core::ptr::addr_of_mut!((*This).r#ref))) as u32;
    return ref_;
}

unsafe extern "system" fn MSTASK_IPersistFile_Release(iface: *mut IPersistFile) -> ULONG {
    let This: *mut TaskImpl = impl_from_IPersistFile(iface);
    let mut ref_0: ULONG = 0;
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/mstask/task.c\0"
                as *const u8 as *const i8,
            b"MSTASK_IPersistFile_Release\0" as *const u8 as *const i8,
            636 as i32,
            b"\n\0" as *const u8 as *const i8,
        );
    }
    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).r#ref) as *mut i32) as ULONG;
    if ref_0 == 0 {
        TaskDestructor(This);
    }
    return ref_0;
}
unsafe extern "system" fn MSTASK_IPersistFile_GetClassID(
    iface: *mut IPersistFile,
    pClassID: *mut CLSID,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pClassID);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_IPersistFile_IsDirty(iface: *mut IPersistFile) -> HRESULT {
    // FIXME("(%p): stub\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_IPersistFile_Load(
    iface: *mut IPersistFile,
    pszFileName: LPCOLESTR,
    dwMode: DWORD,
) -> HRESULT {
    // FIXME("(%p, %p, 0x%08x): stub\n", iface, pszFileName, dwMode);
    E_NOTIMPL
}

unsafe extern "system" fn MSTASK_IPersistFile_Save(
    iface: *mut IPersistFile,
    pszFileName: LPCOLESTR,
    fRemember: BOOL,
) -> HRESULT {
    WARN!("Returning S_OK but not writing to disk: %s %d\n",
            debugstr_w(pszFileName as *const WCHAR), fRemember);
    return S_OK;
}
unsafe extern "system" fn MSTASK_IPersistFile_SaveCompleted(
    iface: *mut IPersistFile,
    pszFileName: LPCOLESTR,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, pszFileName);
    E_NOTIMPL
}
unsafe extern "system" fn MSTASK_IPersistFile_GetCurFile(
    iface: *mut IPersistFile,
    ppszFileName: *mut LPOLESTR,
) -> HRESULT {
    // FIXME("(%p, %p): stub\n", iface, ppszFileName);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn TaskConstructor(pwszTaskName: LPCWSTR, ppObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut TaskImpl = core::ptr::null_mut::<TaskImpl>();
    let mut n: core::ffi::c_int = 0;

    TRACE!("(%s, %p)\n", debugstr_w(pwszTaskName), ppObj);

    This = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<TaskImpl>() as SIZE_T,
    ) as *mut TaskImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ITask_iface.lpVtbl = core::ptr::addr_of!(MSTASK_ITaskVtbl);
    (*This).IPersistFile_iface.lpVtbl = core::ptr::addr_of!(MSTASK_IPersistFileVtbl);
    (*This).r#ref = 1 as LONG;
    n = ((lstrlenW(pwszTaskName) + 1 as core::ffi::c_int) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as core::ffi::c_int;
    (*This).taskName = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        n as SIZE_T,
    ) as LPWSTR;
    if (*This).taskName.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as LPVOID,
        );
        return E_OUTOFMEMORY;
    }
    lstrcpyW((*This).taskName, pwszTaskName);
    (*This).applicationName = core::ptr::null_mut::<WCHAR>();
    (*This).parameters = core::ptr::null_mut::<WCHAR>();
    (*This).comment = core::ptr::null_mut::<WCHAR>();
    (*This).accountName = core::ptr::null_mut::<WCHAR>();

    /* Default time is 3 days = 259200000 ms */
    (*This).maxRunTime = 259200000 as DWORD;

    *ppObj = core::ptr::addr_of_mut!((*This).ITask_iface) as LPVOID;
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_ref) as *mut core::ffi::c_long);
    return S_OK;
}

// COM vtable tables — generated from task.c by vtbl_gen.py
/// task.c:695 `static const ITaskVtbl MSTASK_ITaskVtbl`  (slot order: rust:types_amd64.rs)
pub static MSTASK_ITaskVtbl: ITaskVtbl = ITaskVtbl {
    QueryInterface: Some(MSTASK_ITask_QueryInterface),
    AddRef: Some(MSTASK_ITask_AddRef),
    Release: Some(MSTASK_ITask_Release),
    CreateTrigger: Some(MSTASK_ITask_CreateTrigger),
    DeleteTrigger: Some(MSTASK_ITask_DeleteTrigger),
    GetTriggerCount: Some(MSTASK_ITask_GetTriggerCount),
    GetTrigger: Some(MSTASK_ITask_GetTrigger),
    GetTriggerString: Some(MSTASK_ITask_GetTriggerString),
    GetRunTimes: Some(MSTASK_ITask_GetRunTimes),
    GetNextRunTime: Some(MSTASK_ITask_GetNextRunTime),
    SetIdleWait: Some(MSTASK_ITask_SetIdleWait),
    GetIdleWait: Some(MSTASK_ITask_GetIdleWait),
    Run: Some(MSTASK_ITask_Run),
    Terminate: Some(MSTASK_ITask_Terminate),
    EditWorkItem: Some(MSTASK_ITask_EditWorkItem),
    GetMostRecentRunTime: Some(MSTASK_ITask_GetMostRecentRunTime),
    GetStatus: Some(MSTASK_ITask_GetStatus),
    GetExitCode: Some(MSTASK_ITask_GetExitCode),
    SetComment: Some(MSTASK_ITask_SetComment),
    GetComment: Some(MSTASK_ITask_GetComment),
    SetCreator: Some(MSTASK_ITask_SetCreator),
    GetCreator: Some(MSTASK_ITask_GetCreator),
    SetWorkItemData: Some(MSTASK_ITask_SetWorkItemData),
    GetWorkItemData: Some(MSTASK_ITask_GetWorkItemData),
    SetErrorRetryCount: Some(MSTASK_ITask_SetErrorRetryCount),
    GetErrorRetryCount: Some(MSTASK_ITask_GetErrorRetryCount),
    SetErrorRetryInterval: Some(MSTASK_ITask_SetErrorRetryInterval),
    GetErrorRetryInterval: Some(MSTASK_ITask_GetErrorRetryInterval),
    SetFlags: Some(MSTASK_ITask_SetFlags),
    GetFlags: Some(MSTASK_ITask_GetFlags),
    SetAccountInformation: Some(MSTASK_ITask_SetAccountInformation),
    GetAccountInformation: Some(MSTASK_ITask_GetAccountInformation),
    SetApplicationName: Some(MSTASK_ITask_SetApplicationName),
    GetApplicationName: Some(MSTASK_ITask_GetApplicationName),
    SetParameters: Some(MSTASK_ITask_SetParameters),
    GetParameters: Some(MSTASK_ITask_GetParameters),
    SetWorkingDirectory: Some(MSTASK_ITask_SetWorkingDirectory),
    GetWorkingDirectory: Some(MSTASK_ITask_GetWorkingDirectory),
    SetPriority: Some(MSTASK_ITask_SetPriority),
    GetPriority: Some(MSTASK_ITask_GetPriority),
    SetTaskFlags: Some(MSTASK_ITask_SetTaskFlags),
    GetTaskFlags: Some(MSTASK_ITask_GetTaskFlags),
    SetMaxRunTime: Some(MSTASK_ITask_SetMaxRunTime),
    GetMaxRunTime: Some(MSTASK_ITask_GetMaxRunTime),
};

/// task.c:743 `static const IPersistFileVtbl MSTASK_IPersistFileVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static MSTASK_IPersistFileVtbl: IPersistFileVtbl = IPersistFileVtbl {
    QueryInterface: Some(MSTASK_IPersistFile_QueryInterface),
    AddRef: Some(MSTASK_IPersistFile_AddRef),
    Release: Some(MSTASK_IPersistFile_Release),
    GetClassID: Some(MSTASK_IPersistFile_GetClassID),
    IsDirty: Some(MSTASK_IPersistFile_IsDirty),
    Load: Some(MSTASK_IPersistFile_Load),
    Save: Some(MSTASK_IPersistFile_Save),
    SaveCompleted: Some(MSTASK_IPersistFile_SaveCompleted),
    GetCurFile: Some(MSTASK_IPersistFile_GetCurFile),
};
