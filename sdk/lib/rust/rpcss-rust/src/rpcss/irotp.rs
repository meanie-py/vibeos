use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_0,
}

static mut RunningObjectTable: list = unsafe {
    list {
        next: &raw const RunningObjectTable as *mut list,
        prev: &raw const RunningObjectTable as *mut list,
    }
};

static mut critsect_debug: CRITICAL_SECTION_DEBUG = _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut csRunningObjectTable,
        ProcessLocksList: _LIST_ENTRY {
            Flink: unsafe { &raw mut critsect_debug.ProcessLocksList },
            Blink: unsafe { &raw mut critsect_debug.ProcessLocksList },
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        // The C's `(DWORD_PTR)(__FILE__ ": cs")` debugger label is zeroed: a pointer cannot become
        // an integer in a Rust static initializer (msvcrt's exit.rs takes the same divergence).
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    };

static mut csRunningObjectTable: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut last_cookie: LONG = 1 as LONG;

// WINE_DEFAULT_DEBUG_CHANNEL(rpcss): debug-channel macro, no Rust declaration emitted.

#[repr(C)]
pub struct rot_entry {
    pub entry: list,
    pub object: *mut InterfaceData,
    pub moniker: *mut InterfaceData,
    pub moniker_data: *mut MonikerComparisonData,
    pub cookie: DWORD,
    pub last_modified: FILETIME,
    pub refs: LONG,
}
pub type Prot_entry = *mut rot_entry;

// static struct list RunningObjectTable = LIST_INIT(RunningObjectTable);

// static CRITICAL_SECTION csRunningObjectTable;  (initialized below)

// static CRITICAL_SECTION_DEBUG critsect_debug = {...};
// NOTE: the C initializer casts a string literal address to DWORD_PTR, which is
// not permitted in a Rust const context; the debug name slot is zeroed here.

// static LONG last_cookie = 1;

unsafe extern "C" fn rot_entry_release(rot_entry: *mut rot_entry) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*rot_entry).refs) as *mut i32) == 0 {
        free((*rot_entry).object as *mut core::ffi::c_void);
        free((*rot_entry).moniker as *mut core::ffi::c_void);
        free((*rot_entry).moniker_data as *mut core::ffi::c_void);
        free(rot_entry as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IrotRegister(
    h: IrotHandle,
    data: *const MonikerComparisonData,
    obj: *const InterfaceData,
    mk: *const InterfaceData,
    time: *const FILETIME,
    grfFlags: DWORD,
    cookie: *mut IrotCookie,
    ctxt_handle: *mut IrotContextHandle,
) -> HRESULT {
    let mut rot_entry: *mut rot_entry = core::ptr::null_mut::<rot_entry>();
    let mut existing_rot_entry: *mut rot_entry = core::ptr::null_mut::<rot_entry>();
    let mut hr: HRESULT = 0;

    if grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT) as DWORD != 0 {
        WINE_ERR!(
            "Invalid grfFlags: 0x%08lx\n",
            grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT) as DWORD,
        );
        return E_INVALIDARG;
    }

    rot_entry = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<rot_entry>() as size_t) as _,
    ) as *mut rot_entry;
    if rot_entry.is_null() {
        return E_OUTOFMEMORY;
    }

    (*rot_entry).refs = 1;
    (*rot_entry).object = malloc(
        (FIELD_OFFSET!(InterfaceData, abData[(*obj).ulCntData as usize]) as usize) as size_t,
    ) as *mut InterfaceData;
    if (*rot_entry).object.is_null() {
        rot_entry_release(rot_entry);
        return E_OUTOFMEMORY;
    }
    (*(*rot_entry).object).ulCntData = (*obj).ulCntData;
    memcpy(
        core::ptr::addr_of_mut!((*(*rot_entry).object).abData) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*obj).abData) as *const core::ffi::c_void,
        (*obj).ulCntData as size_t,
    );

    (*rot_entry).last_modified = *time;

    (*rot_entry).moniker = malloc(
        (FIELD_OFFSET!(InterfaceData, abData[(*mk).ulCntData as usize]) as usize) as size_t,
    ) as *mut InterfaceData;
    if (*rot_entry).moniker.is_null() {
        rot_entry_release(rot_entry);
        return E_OUTOFMEMORY;
    }
    (*(*rot_entry).moniker).ulCntData = (*mk).ulCntData;
    memcpy(
        core::ptr::addr_of_mut!((*(*rot_entry).moniker).abData) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*mk).abData) as *const core::ffi::c_void,
        (*mk).ulCntData as size_t,
    );

    (*rot_entry).moniker_data = malloc(
        (FIELD_OFFSET!(MonikerComparisonData, abData[(*data).ulCntData as usize]) as usize) as size_t,
    ) as *mut MonikerComparisonData;
    if (*rot_entry).moniker_data.is_null() {
        rot_entry_release(rot_entry);
        return E_OUTOFMEMORY;
    }
    (*(*rot_entry).moniker_data).ulCntData = (*data).ulCntData;
    memcpy(
        core::ptr::addr_of_mut!((*(*rot_entry).moniker_data).abData) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*data).abData) as *const core::ffi::c_void,
        (*data).ulCntData as size_t,
    );

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    hr = S_OK;

    // LIST_FOR_EACH_ENTRY(existing_rot_entry, &RunningObjectTable, struct rot_entry, entry)
    existing_rot_entry = (RunningObjectTable.next as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry)) as *mut rot_entry;
    while core::ptr::addr_of_mut!((*existing_rot_entry).entry)
        != core::ptr::addr_of_mut!(RunningObjectTable)
    {
        if (*(*existing_rot_entry).moniker_data).ulCntData == (*data).ulCntData
            && memcmp(
                core::ptr::addr_of!((*data).abData) as *const core::ffi::c_void,
                core::ptr::addr_of_mut!((*(*existing_rot_entry).moniker_data).abData)
                    as *const core::ffi::c_void,
                (*data).ulCntData as size_t,
            ) == 0
        {
            hr = MK_S_MONIKERALREADYREGISTERED;
            WINE_TRACE!(
                "moniker already registered with cookie %ld\n",
                (*existing_rot_entry).cookie,
            );
            break;
        } else {
            existing_rot_entry = ((*existing_rot_entry).entry.next as *mut u8)
                .sub(core::mem::offset_of!(rot_entry, entry)) as *mut rot_entry;
        }
    }

    list_add_tail(
        core::ptr::addr_of_mut!(RunningObjectTable),
        (core::ptr::addr_of_mut!((*rot_entry).entry)).cast(),
    );

    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    /* gives a registration identifier to the registered object */
    (*rot_entry).cookie = InterlockedIncrement(core::ptr::addr_of_mut!(last_cookie)) as DWORD;
    *cookie = (*rot_entry).cookie as IrotCookie;
    *ctxt_handle = rot_entry as IrotContextHandle;

    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn IrotRevoke(
    h: IrotHandle,
    cookie: IrotCookie,
    ctxt_handle: *mut IrotContextHandle,
    obj: *mut PInterfaceData,
    mk: *mut PInterfaceData,
) -> HRESULT {
    let mut rot_entry: *mut rot_entry = core::ptr::null_mut();

    WINE_TRACE!(b"%ld\n\0".as_ptr() as *const i8, cookie);

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    rot_entry = ((RunningObjectTable.next as *mut u8)
        .offset(-(core::mem::offset_of!(rot_entry, entry) as isize)))
        as *mut rot_entry;
    while core::ptr::addr_of_mut!((*rot_entry).entry)
        != core::ptr::addr_of_mut!(RunningObjectTable)
    {
        if (*rot_entry).cookie == cookie {
            let mut hr: HRESULT = S_OK;

            list_remove((core::ptr::addr_of_mut!((*rot_entry).entry)).cast());
            LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

            *obj = MIDL_user_allocate(
                (core::mem::offset_of!(InterfaceData, abData)
                    + (*(*rot_entry).object).ulCntData as usize
                        * core::mem::size_of::<BYTE>()) as SIZE_T,
            ) as PInterfaceData;
            *mk = MIDL_user_allocate(
                (core::mem::offset_of!(InterfaceData, abData)
                    + (*(*rot_entry).moniker).ulCntData as usize
                        * core::mem::size_of::<BYTE>()) as SIZE_T,
            ) as PInterfaceData;
            if !(*obj).is_null() && !(*mk).is_null() {
                (**obj).ulCntData = (*(*rot_entry).object).ulCntData;
                memcpy(
                    core::ptr::addr_of_mut!((**obj).abData) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*(*rot_entry).object).abData) as *const core::ffi::c_void,
                    (**obj).ulCntData as usize,
                );
                (**mk).ulCntData = (*(*rot_entry).moniker).ulCntData;
                memcpy(
                    core::ptr::addr_of_mut!((**mk).abData) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*(*rot_entry).moniker).abData) as *const core::ffi::c_void,
                    (**mk).ulCntData as usize,
                );
            } else {
                MIDL_user_free(*obj as *mut core::ffi::c_void);
                MIDL_user_free(*mk as *mut core::ffi::c_void);
                hr = E_OUTOFMEMORY;
            }

            rot_entry_release(rot_entry);
            *ctxt_handle = core::ptr::null_mut();
            return hr;
        }
        rot_entry = (((*rot_entry).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(rot_entry, entry) as isize)))
            as *mut rot_entry;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    return E_INVALIDARG;
}
#[no_mangle]
pub unsafe extern "C" fn IrotIsRunning(
    h: IrotHandle,
    data: *const MonikerComparisonData,
) -> HRESULT {
    let mut rot_entry: *const rot_entry = core::ptr::null::<rot_entry>();
    let mut hr: HRESULT = S_FALSE;

    WINE_TRACE!(b"\n\0".as_ptr() as *const ::core::ffi::c_char);

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    // LIST_FOR_EACH_ENTRY starts at head->next; the translation started at the head itself,
    // so the loop never ran and no moniker was ever found.
    rot_entry = (RunningObjectTable.next as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry)) as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != (core::ptr::addr_of_mut!(RunningObjectTable) as *const list).cast()
    {
        if (*(*rot_entry).moniker_data).ulCntData == (*data).ulCntData
            && memcmp(
                core::ptr::addr_of!((*data).abData) as *const ::core::ffi::c_void,
                core::ptr::addr_of!((*(*rot_entry).moniker_data).abData) as *const ::core::ffi::c_void,
                (*data).ulCntData as usize,
            ) == 0
        {
            hr = S_OK;
            break;
        } else {
            rot_entry = ((*rot_entry).entry.next as *mut u8)
                .sub(core::mem::offset_of!(rot_entry, entry)) as *const rot_entry;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn IrotGetObject(
    h: IrotHandle,
    moniker_data: *const MonikerComparisonData,
    obj: *mut PInterfaceData,
    cookie: *mut IrotCookie,
) -> HRESULT {
    let mut rot_entry: *const rot_entry = core::ptr::null::<rot_entry>();

    WINE_TRACE!("%p\n", moniker_data);

    *cookie = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    // LIST_FOR_EACH_ENTRY starts at head->next; the translation started at the head itself,
    // so the loop never ran and no moniker was ever found.
    rot_entry = (RunningObjectTable.next as *mut u8)
        .offset(-(core::mem::offset_of!(rot_entry, entry) as isize))
        as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != (core::ptr::addr_of_mut!(RunningObjectTable) as *const list).cast()
    {
        let mut hr: HRESULT = S_OK;
        if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
            && memcmp(
                core::ptr::addr_of!((*moniker_data).abData) as *const core::ffi::c_void,
                core::ptr::addr_of_mut!((*(*rot_entry).moniker_data).abData)
                    as *const core::ffi::c_void,
                (*moniker_data).ulCntData as usize,
            ) == 0
        {
            *obj = MIDL_user_allocate(FIELD_OFFSET!(
                InterfaceData,
                abData[(*(*rot_entry).object).ulCntData as usize]
            ) as SIZE_T) as PInterfaceData;
            if !(*obj).is_null() {
                (**obj).ulCntData = (*(*rot_entry).object).ulCntData;
                memcpy(
                    core::ptr::addr_of_mut!((**obj).abData) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!((*(*rot_entry).object).abData)
                        as *const core::ffi::c_void,
                    (**obj).ulCntData as usize,
                );

                *cookie = (*rot_entry).cookie;
            } else {
                hr = E_OUTOFMEMORY;
            }

            LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

            return hr;
        }
        rot_entry = ((*rot_entry).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(rot_entry, entry) as isize))
            as *const rot_entry;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    return MK_E_UNAVAILABLE;
}
#[no_mangle]
pub unsafe extern "C" fn IrotNoteChangeTime(
    h: IrotHandle,
    cookie: IrotCookie,
    last_modified_time: *const FILETIME,
) -> HRESULT {
    let mut rot_entry: *mut rot_entry = core::ptr::null_mut();

    WINE_TRACE!("%ld %p\n", cookie, last_modified_time);

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    rot_entry = ((*core::ptr::addr_of_mut!(RunningObjectTable)).next as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry)) as *mut rot_entry;
    while core::ptr::addr_of_mut!((*rot_entry).entry)
        != core::ptr::addr_of_mut!(RunningObjectTable)
    {
        if (*rot_entry).cookie == cookie {
            (*rot_entry).last_modified = *last_modified_time;
            LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
            return S_OK;
        }
        rot_entry = (((*rot_entry).entry.next as *mut u8)
            .sub(core::mem::offset_of!(rot_entry, entry))) as *mut rot_entry;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    return E_INVALIDARG;
}
#[no_mangle]
pub unsafe extern "C" fn IrotGetTimeOfLastChange(
    h: IrotHandle,
    moniker_data: *const MonikerComparisonData,
    time: *mut FILETIME,
) -> HRESULT {
    let mut rot_entry: *const rot_entry = core::ptr::null();
    let mut hr: HRESULT = MK_E_UNAVAILABLE;

    WINE_TRACE!(b"%p\n\0".as_ptr() as *const i8, moniker_data);

    memset(
        time as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<FILETIME>(),
    );

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    // LIST_FOR_EACH_ENTRY(rot_entry, &RunningObjectTable, const struct rot_entry, entry)
    rot_entry = ((RunningObjectTable.next as *mut u8)
        .sub(core::mem::offset_of!(rot_entry, entry))) as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != (core::ptr::addr_of_mut!(RunningObjectTable) as *const list).cast()
    {
        if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
            && memcmp(
                core::ptr::addr_of!((*moniker_data).abData) as *const core::ffi::c_void,
                core::ptr::addr_of!((*(*rot_entry).moniker_data).abData)
                    as *const core::ffi::c_void,
                (*moniker_data).ulCntData as usize,
            ) == 0
        {
            *time = (*rot_entry).last_modified;
            hr = S_OK;
            break;
        } else {
            rot_entry = (((*rot_entry).entry.next as *mut u8)
                .sub(core::mem::offset_of!(rot_entry, entry)))
                as *const rot_entry;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn IrotEnumRunning(
    h: IrotHandle,
    list: *mut PInterfaceList,
) -> HRESULT {
    let mut rot_entry: *const rot_entry = core::ptr::null::<rot_entry>();
    let mut hr: HRESULT = S_OK;
    let mut moniker_count: ULONG = 0;
    let mut i: ULONG = 0;

    WINE_TRACE!(b"\n\0".as_ptr() as *const i8);

    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));

    rot_entry = ((*core::ptr::addr_of_mut!(RunningObjectTable)).next as *mut u8).offset(
        -(core::mem::offset_of!(rot_entry, entry) as isize),
    ) as *const rot_entry;
    while core::ptr::addr_of!((*rot_entry).entry)
        != (core::ptr::addr_of_mut!(RunningObjectTable) as *const list).cast()
    {
        moniker_count += 1;
        rot_entry = (((*rot_entry).entry.next) as *mut u8).offset(
            -(core::mem::offset_of!(rot_entry, entry) as isize),
        ) as *const rot_entry;
    }

    *list = MIDL_user_allocate(
        (core::mem::offset_of!(InterfaceList, interfaces)
            + (moniker_count as usize) * core::mem::size_of::<InterfaceData>())
            as SIZE_T,
    ) as PInterfaceList;
    if !(*list).is_null() {
        (**list).size = moniker_count;
        rot_entry = ((*core::ptr::addr_of_mut!(RunningObjectTable)).next as *mut u8).offset(
            -(core::mem::offset_of!(rot_entry, entry) as isize),
        ) as *const rot_entry;
        while core::ptr::addr_of!((*rot_entry).entry)
            != (core::ptr::addr_of_mut!(RunningObjectTable) as *const list).cast()
        {
            *(core::ptr::addr_of_mut!((**list).interfaces) as *mut PInterfaceData)
                .offset(i as isize) = MIDL_user_allocate(
                (core::mem::offset_of!(InterfaceData, abData)
                    + (*(*rot_entry).moniker).ulCntData as usize) as SIZE_T,
            ) as PInterfaceData;
            if (*(core::ptr::addr_of_mut!((**list).interfaces) as *mut PInterfaceData)
                .offset(i as isize))
            .is_null()
            {
                let end: ULONG = i.wrapping_sub(1);
                i = 0;
                while i < end {
                    MIDL_user_free(
                        *(core::ptr::addr_of_mut!((**list).interfaces) as *mut PInterfaceData)
                            .offset(i as isize) as *mut core::ffi::c_void,
                    );
                    i = i.wrapping_add(1);
                }
                MIDL_user_free(*list as *mut core::ffi::c_void);
                hr = E_OUTOFMEMORY;
                break;
            }
            (**(core::ptr::addr_of_mut!((**list).interfaces) as *mut PInterfaceData)
                .offset(i as isize))
            .ulCntData = (*(*rot_entry).moniker).ulCntData;
            memcpy(
                core::ptr::addr_of_mut!(
                    (**(core::ptr::addr_of_mut!((**list).interfaces) as *mut PInterfaceData)
                        .offset(i as isize))
                    .abData
                ) as *mut u8 as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*(*rot_entry).moniker).abData) as *mut u8
                    as *const core::ffi::c_void,
                (*(*rot_entry).moniker).ulCntData as usize,
            );
            i = i.wrapping_add(1);
            rot_entry = (((*rot_entry).entry.next) as *mut u8).offset(
                -(core::mem::offset_of!(rot_entry, entry) as isize),
            ) as *const rot_entry;
        }
    } else {
        hr = E_OUTOFMEMORY;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn IrotContextHandle_rundown(ctxt_handle: IrotContextHandle) {
    let rot_entry: *mut rot_entry = ctxt_handle as *mut rot_entry;
    EnterCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    list_remove((core::ptr::addr_of_mut!((*rot_entry).entry)).cast());
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRunningObjectTable));
    rot_entry_release(rot_entry);
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(size: SIZE_T) -> *mut core::ffi::c_void {
    I_RpcAllocate((size) as u32)
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(p: *mut core::ffi::c_void) {
    I_RpcFree(p);
}
