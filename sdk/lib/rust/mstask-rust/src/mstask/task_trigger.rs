use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mstask): function-like macro, no Rust declaration emitted.

#[repr(C)]
pub struct TaskTriggerImpl {
    pub ITaskTrigger_iface: ITaskTrigger,
    pub ref_: LONG,
    pub triggerCond: TASK_TRIGGER,
}

pub type PTaskTriggerImpl = *mut TaskTriggerImpl;

unsafe extern "C" fn impl_from_ITaskTrigger(mut iface: *mut ITaskTrigger) -> *mut TaskTriggerImpl {
    return ((iface as usize).wrapping_sub(core::mem::offset_of!(TaskTriggerImpl, ITaskTrigger_iface))) as *mut TaskTriggerImpl;
}

unsafe extern "system" fn MSTASK_ITaskTrigger_QueryInterface(
    iface: *mut ITaskTrigger,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut This: *mut TaskTriggerImpl = impl_from_ITaskTrigger(iface);
    if ppvObject.is_null() {
        return E_POINTER;
    }
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_ITaskTrigger))
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).ITaskTrigger_iface) as *mut core::ffi::c_void;
        ITaskTrigger_AddRef(iface);
        return S_OK;
    }
    WARN!("Unknown interface: %s\n", debugstr_guid(riid));
    *ppvObject = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn MSTASK_ITaskTrigger_AddRef(iface: *mut ITaskTrigger) -> ULONG {
    let This: *mut TaskTriggerImpl = impl_from_ITaskTrigger(iface);
    let mut ref_: ULONG;
    ref_ = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    return ref_;
}

unsafe extern "system" fn MSTASK_ITaskTrigger_Release(iface: *mut ITaskTrigger) -> ULONG {
    let This: *mut TaskTriggerImpl = impl_from_ITaskTrigger(iface);
    let mut ref_0: ULONG;
    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut LONG) as ULONG;
    if ref_0 == 0 {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        InterlockedDecrement(core::ptr::addr_of_mut!(dll_ref) as *mut LONG);
    }
    return ref_0;
}

unsafe extern "system" fn MSTASK_ITaskTrigger_SetTrigger(
    iface: *mut ITaskTrigger,
    pTrigger: PTASK_TRIGGER,
) -> HRESULT {
    let This: *mut TaskTriggerImpl = impl_from_ITaskTrigger(iface);
    let mut field_time: TIME_FIELDS = core::mem::zeroed();
    let mut sys_time: LARGE_INTEGER = core::mem::zeroed();
    let mut tmp_trigger_cond: TASK_TRIGGER = core::mem::zeroed();

    TRACE!("(%p, %p)\n", iface, pTrigger);

    /* Verify valid structure size */
    if (*pTrigger).cbTriggerSize as usize != core::mem::size_of::<TASK_TRIGGER>() as usize {
        return E_INVALIDARG;
    }
    tmp_trigger_cond.cbTriggerSize = (*pTrigger).cbTriggerSize;

    /* Reserved field must be zero */
    tmp_trigger_cond.Reserved1 = 0;

    /* Verify and set valid start date and time */
    memset(
        core::ptr::addr_of_mut!(field_time) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<TIME_FIELDS>() as usize,
    );
    field_time.Year = (*pTrigger).wBeginYear as CSHORT;
    field_time.Month = (*pTrigger).wBeginMonth as CSHORT;
    field_time.Day = (*pTrigger).wBeginDay as CSHORT;
    field_time.Hour = (*pTrigger).wStartHour as CSHORT;
    field_time.Minute = (*pTrigger).wStartMinute as CSHORT;
    if RtlTimeFieldsToTime(
        core::ptr::addr_of_mut!(field_time),
        core::ptr::addr_of_mut!(sys_time),
    ) == 0
    {
        return E_INVALIDARG;
    }
    tmp_trigger_cond.wBeginYear = (*pTrigger).wBeginYear;
    tmp_trigger_cond.wBeginMonth = (*pTrigger).wBeginMonth;
    tmp_trigger_cond.wBeginDay = (*pTrigger).wBeginDay;
    tmp_trigger_cond.wStartHour = (*pTrigger).wStartHour;
    tmp_trigger_cond.wStartMinute = (*pTrigger).wStartMinute;

    /* Verify valid end date if TASK_TRIGGER_FLAG_HAS_END_DATE flag is set */
    if (*pTrigger).rgFlags & TASK_TRIGGER_FLAG_HAS_END_DATE as DWORD != 0 {
        memset(
            core::ptr::addr_of_mut!(field_time) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<TIME_FIELDS>() as usize,
        );
        field_time.Year = (*pTrigger).wEndYear as CSHORT;
        field_time.Month = (*pTrigger).wEndMonth as CSHORT;
        field_time.Day = (*pTrigger).wEndDay as CSHORT;
        if RtlTimeFieldsToTime(
            core::ptr::addr_of_mut!(field_time),
            core::ptr::addr_of_mut!(sys_time),
        ) == 0
        {
            return E_INVALIDARG;
        }
    }

    /* Set valid end date independent of TASK_TRIGGER_FLAG_HAS_END_DATE flag */
    tmp_trigger_cond.wEndYear = (*pTrigger).wEndYear;
    tmp_trigger_cond.wEndMonth = (*pTrigger).wEndMonth;
    tmp_trigger_cond.wEndDay = (*pTrigger).wEndDay;

    /* Verify duration and interval pair */
    if (*pTrigger).MinutesDuration <= (*pTrigger).MinutesInterval
        && (*pTrigger).MinutesInterval > 0 as DWORD
    {
        return E_INVALIDARG;
    }
    tmp_trigger_cond.MinutesDuration = (*pTrigger).MinutesDuration;
    tmp_trigger_cond.MinutesInterval = (*pTrigger).MinutesInterval;

    /* Copy over flags */
    tmp_trigger_cond.rgFlags = (*pTrigger).rgFlags;

    /* Set TriggerType dependent fields of Type union */
    tmp_trigger_cond.TriggerType = (*pTrigger).TriggerType;
    match (*pTrigger).TriggerType as u32 {
        TASK_TIME_TRIGGER_DAILY => {
            tmp_trigger_cond.Type.Daily.DaysInterval = (*pTrigger).Type.Daily.DaysInterval;
        }
        TASK_TIME_TRIGGER_WEEKLY => {
            tmp_trigger_cond.Type.Weekly.WeeksInterval = (*pTrigger).Type.Weekly.WeeksInterval;
            tmp_trigger_cond.Type.Weekly.rgfDaysOfTheWeek =
                (*pTrigger).Type.Weekly.rgfDaysOfTheWeek;
        }
        TASK_TIME_TRIGGER_MONTHLYDATE => {
            tmp_trigger_cond.Type.MonthlyDate.rgfDays = (*pTrigger).Type.MonthlyDate.rgfDays;
            tmp_trigger_cond.Type.MonthlyDate.rgfMonths = (*pTrigger).Type.MonthlyDate.rgfMonths;
        }
        TASK_TIME_TRIGGER_MONTHLYDOW => {
            tmp_trigger_cond.Type.MonthlyDOW.wWhichWeek = (*pTrigger).Type.MonthlyDOW.wWhichWeek;
            tmp_trigger_cond.Type.MonthlyDOW.rgfDaysOfTheWeek =
                (*pTrigger).Type.MonthlyDOW.rgfDaysOfTheWeek;
            tmp_trigger_cond.Type.MonthlyDOW.rgfMonths = (*pTrigger).Type.MonthlyDOW.rgfMonths;
        }
        TASK_TIME_TRIGGER_ONCE | TASK_EVENT_TRIGGER_ON_IDLE | TASK_EVENT_TRIGGER_AT_SYSTEMSTART | TASK_EVENT_TRIGGER_AT_LOGON | _ => {
            tmp_trigger_cond.Type = (*This).triggerCond.Type;
        }
    }

    /* Reserved field must be zero */
    tmp_trigger_cond.Reserved2 = 0;

    /* wRandomMinutesInterval not currently used and is initialized to zero */
    tmp_trigger_cond.wRandomMinutesInterval = 0;

    /* Update object copy of triggerCond */
    (*This).triggerCond = tmp_trigger_cond;

    return S_OK;
}

unsafe extern "system" fn MSTASK_ITaskTrigger_GetTrigger(
    mut iface: *mut ITaskTrigger,
    mut pTrigger: PTASK_TRIGGER,
) -> HRESULT {
    let mut This: *mut TaskTriggerImpl = impl_from_ITaskTrigger(iface);

    TRACE!("(%p, %p)\n", iface, pTrigger);

    /* Native implementation doesn't verify equivalent cbTriggerSize fields */

    /* Copy relevant fields of the structure */
    (*pTrigger).cbTriggerSize = (*This).triggerCond.cbTriggerSize;
    (*pTrigger).Reserved1 = 0;
    (*pTrigger).wBeginYear = (*This).triggerCond.wBeginYear;
    (*pTrigger).wBeginMonth = (*This).triggerCond.wBeginMonth;
    (*pTrigger).wBeginDay = (*This).triggerCond.wBeginDay;
    (*pTrigger).wEndYear = (*This).triggerCond.wEndYear;
    (*pTrigger).wEndMonth = (*This).triggerCond.wEndMonth;
    (*pTrigger).wEndDay = (*This).triggerCond.wEndDay;
    (*pTrigger).wStartHour = (*This).triggerCond.wStartHour;
    (*pTrigger).wStartMinute = (*This).triggerCond.wStartMinute;
    (*pTrigger).MinutesDuration = (*This).triggerCond.MinutesDuration;
    (*pTrigger).MinutesInterval = (*This).triggerCond.MinutesInterval;
    (*pTrigger).rgFlags = (*This).triggerCond.rgFlags;
    (*pTrigger).TriggerType = (*This).triggerCond.TriggerType;
    match (*This).triggerCond.TriggerType as ::core::ffi::c_uint {
        TASK_TIME_TRIGGER_DAILY => {
            (*pTrigger).Type.Daily.DaysInterval = (*This).triggerCond.Type.Daily.DaysInterval;
        }
        TASK_TIME_TRIGGER_WEEKLY => {
            (*pTrigger).Type.Weekly.WeeksInterval = (*This).triggerCond.Type.Weekly.WeeksInterval;
            (*pTrigger).Type.Weekly.rgfDaysOfTheWeek =
                (*This).triggerCond.Type.Weekly.rgfDaysOfTheWeek;
        }
        TASK_TIME_TRIGGER_MONTHLYDATE => {
            (*pTrigger).Type.MonthlyDate.rgfDays = (*This).triggerCond.Type.MonthlyDate.rgfDays;
            (*pTrigger).Type.MonthlyDate.rgfMonths = (*This).triggerCond.Type.MonthlyDate.rgfMonths;
        }
        TASK_TIME_TRIGGER_MONTHLYDOW => {
            (*pTrigger).Type.MonthlyDOW.wWhichWeek = (*This).triggerCond.Type.MonthlyDOW.wWhichWeek;
            (*pTrigger).Type.MonthlyDOW.rgfDaysOfTheWeek =
                (*This).triggerCond.Type.MonthlyDOW.rgfDaysOfTheWeek;
            (*pTrigger).Type.MonthlyDOW.rgfMonths = (*This).triggerCond.Type.MonthlyDOW.rgfMonths;
        }
        TASK_TIME_TRIGGER_ONCE | TASK_EVENT_TRIGGER_ON_IDLE | TASK_EVENT_TRIGGER_AT_SYSTEMSTART | TASK_EVENT_TRIGGER_AT_LOGON | _ => {}
    }
    (*pTrigger).Reserved2 = 0;
    (*pTrigger).wRandomMinutesInterval = 0;
    return S_OK;
}
unsafe extern "system" fn MSTASK_ITaskTrigger_GetTriggerString(
    iface: *mut ITaskTrigger,
    ppwszTrigger: *mut LPWSTR,
) -> HRESULT {
    // FIXME("Not implemented\n");
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn TaskTriggerConstructor(ppObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut TaskTriggerImpl = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<TaskTriggerImpl>()) as SIZE_T,
    ) as *mut TaskTriggerImpl;
    let mut time: SYSTEMTIME = core::mem::zeroed();
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ITaskTrigger_iface.lpVtbl = core::ptr::addr_of!(MSTASK_ITaskTriggerVtbl) as _;
    (*This).ref_ = 1;

    /* Most fields of triggerCond default to zero.  Initialize other
     * fields to default values. */
    memset(
        core::ptr::addr_of_mut!((*This).triggerCond) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<TASK_TRIGGER>(),
    );
    GetLocalTime(core::ptr::addr_of_mut!(time));
    (*This).triggerCond.cbTriggerSize = core::mem::size_of::<TASK_TRIGGER>() as WORD;
    (*This).triggerCond.wBeginYear = time.wYear;
    (*This).triggerCond.wBeginMonth = time.wMonth;
    (*This).triggerCond.wBeginDay = time.wDay;
    (*This).triggerCond.wStartHour = time.wHour;
    (*This).triggerCond.wStartMinute = time.wMinute;
    (*This).triggerCond.rgFlags = TASK_TRIGGER_FLAG_DISABLED;
    (*This).triggerCond.TriggerType = TASK_TIME_TRIGGER_DAILY;
    (*This).triggerCond.Type.Daily.DaysInterval = 1;

    *ppObj = core::ptr::addr_of_mut!((*This).ITaskTrigger_iface) as LPVOID;
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_ref) as *mut LONG);
    return S_OK;
}

// COM vtable tables — generated from task_trigger.c by vtbl_gen.py
/// task_trigger.c:267 `static const ITaskTriggerVtbl MSTASK_ITaskTriggerVtbl`  (slot order: rust:types_amd64.rs)
pub static MSTASK_ITaskTriggerVtbl: ITaskTriggerVtbl = ITaskTriggerVtbl {
    QueryInterface: Some(MSTASK_ITaskTrigger_QueryInterface),
    AddRef: Some(MSTASK_ITaskTrigger_AddRef),
    Release: Some(MSTASK_ITaskTrigger_Release),
    SetTrigger: Some(MSTASK_ITaskTrigger_SetTrigger),
    GetTrigger: Some(MSTASK_ITaskTrigger_GetTrigger),
    GetTriggerString: Some(MSTASK_ITaskTrigger_GetTriggerString),
};
