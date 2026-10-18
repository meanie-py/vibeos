// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1A00F51F3029: [u16; 2] = [0x0028u16, 0x0000];
pub static RUSTCONV_WSTR_2157805E1164: [u16; 4] = [0x0023u16, 0x0025, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_3DBECEBCF87C: [u16; 3] = [0x005Cu16, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_4F2D892D6AF4: [u16; 2] = [0x0029u16, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_BCE941D196C2: [u16; 2] = [0x002Fu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_7425CE359F56: [u16; 36] = [
    0x005Cu16, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072,
    0x0028, 0x005F, 0x0054, 0x006F, 0x0074, 0x0061, 0x006C, 0x0029, 0x005C, 0x0025,
    0x0020, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072,
    0x0020, 0x0054, 0x0069, 0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_BC273F53134B: [u16; 23] = [
    0x005Cu16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x0020, 0x0055, 0x0070, 0x0020, 0x0054, 0x0069,
    0x006D, 0x0065, 0x0000,
];

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

static mut pdh_handle_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut pdh_handle_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut pdh_handle_cs_debug.ProcessLocksList,
            Blink: &raw mut pdh_handle_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        // Wine's initializer puts the address of a "file: name" string here; a pointer is not
        // an integer in a const initializer, and the slot only labels the lock for a debugger.
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    } };

static mut pdh_handle_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const pdh_handle_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut counter_sources: [source; 2] = unsafe {
    [
        source {
            index: 6 as DWORD,
            path: RUSTCONV_C2R_WSTR_7425CE359F56.as_ptr() as *const WCHAR,
            collect: Some(
                collect_processor_time,
            ),
            r#type: TYPE_PROCESSOR_TIME as DWORD,
            scale: -(5 as ::core::ffi::c_int) as LONG,
            base: 10000000 as LONGLONG,
        },
        source {
            index: 674 as DWORD,
            path: RUSTCONV_C2R_WSTR_BC273F53134B.as_ptr() as *const WCHAR,
            collect: Some(collect_uptime),
            r#type: TYPE_UPTIME as DWORD,
            scale: -(3 as ::core::ffi::c_int) as LONG,
            base: 1000 as LONGLONG,
        },
    ]
};



// Initializer for Spare[0] is the address of the __FILE__ string; a pointer-to-integer
// cast is not const-evaluable, so it is written as 0 here.

#[repr(C)]
pub union value {
    pub longvalue: LONG,
    pub doublevalue: f64,
    pub largevalue: LONGLONG,
}

#[repr(C)]
pub struct counter {
    pub magic: DWORD,
    pub entry: list,
    pub path: *mut WCHAR,
    pub r#type: DWORD,
    pub status: DWORD,
    pub scale: LONG,
    pub defaultscale: LONG,
    pub user: DWORD_PTR,
    pub queryuser: DWORD_PTR,
    pub base: LONGLONG,
    pub stamp: FILETIME,
    pub collect: Option<unsafe extern "system" fn(*mut counter)>,
    pub one: value,
    pub two: value,
}

pub const PDH_MAGIC_COUNTER: DWORD = 0x50444831; // 'PDH1'

#[repr(C)]
pub struct _PDH_FMT_COUNTERVALUE_ITEM_A {
    pub szName: LPSTR,
    pub FmtValue: PDH_FMT_COUNTERVALUE,
}
pub type PDH_FMT_COUNTERVALUE_ITEM_A = _PDH_FMT_COUNTERVALUE_ITEM_A;
pub type PPDH_FMT_COUNTERVALUE_ITEM_A = *mut _PDH_FMT_COUNTERVALUE_ITEM_A;

#[repr(C)]
pub struct _PDH_FMT_COUNTERVALUE_ITEM_W {
    pub szName: LPWSTR,
    pub FmtValue: PDH_FMT_COUNTERVALUE,
}
pub type PDH_FMT_COUNTERVALUE_ITEM_W = _PDH_FMT_COUNTERVALUE_ITEM_W;
pub type PPDH_FMT_COUNTERVALUE_ITEM_W = *mut _PDH_FMT_COUNTERVALUE_ITEM_W;

pub const PDH_MAGIC_QUERY: DWORD = 0x50444830; // 'PDH0'

#[repr(C)]
pub struct query {
    pub magic: DWORD,
    pub user: DWORD_PTR,
    pub thread: HANDLE,
    pub interval: DWORD,
    pub wait: HANDLE,
    pub stop: HANDLE,
    pub counters: list,
}

#[repr(C)]
pub struct source {
    pub index: DWORD,
    pub path: *const WCHAR,
    pub collect: Option<unsafe extern "system" fn(*mut counter)>,
    pub r#type: DWORD,
    pub scale: LONG,
    pub base: LONGLONG,
}

// Perflib constants used by the counter type macros (winperf.h values).

pub const TYPE_PROCESSOR_TIME: DWORD = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_RATE
    | PERF_TIMER_100NS
    | PERF_DELTA_COUNTER
    | PERF_INVERSE_COUNTER
    | PERF_DISPLAY_PERCENT;

pub const TYPE_UPTIME: DWORD = PERF_SIZE_LARGE
    | PERF_TYPE_COUNTER
    | PERF_COUNTER_ELAPSED
    | PERF_OBJECT_TIMER
    | PERF_DISPLAY_SECONDS;

// Contains raw pointers, so it is declared `static mut` (raw pointers are not Sync).

unsafe extern "C" fn pdh_strdup_aw(src: *const ::core::ffi::c_char) -> *mut WCHAR {
    let mut len: ::core::ffi::c_int = 0;
    let mut dst: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    if src.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    len = MultiByteToWideChar(
        CP_ACP,
        0,
        src as LPCSTR,
        -1,
        ::core::ptr::null_mut::<WCHAR>(),
        0,
    );
    dst = malloc(
        (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if !dst.is_null() {
        MultiByteToWideChar(
            CP_ACP,
            0,
            src as LPCSTR,
            -1,
            dst as LPWSTR,
            len,
        );
    }
    return dst;
}

unsafe extern "C" fn create_counter() -> *mut counter {
    let mut counter: *mut counter = core::ptr::null_mut::<counter>();
    counter = calloc(1, (core::mem::size_of::<counter>() as size_t) as _) as *mut counter;
    if !counter.is_null() {
        (*counter).magic = PDH_MAGIC_COUNTER as DWORD;
        return counter;
    }
    return core::ptr::null_mut::<counter>();
}

unsafe extern "C" fn destroy_counter(counter: *mut counter) {
    /* Ensure compiler doesn't optimize out the assignment with 0. */
    SecureZeroMemory(core::ptr::addr_of_mut!((*counter).magic) as PVOID, core::mem::size_of::<DWORD>() as SIZE_T);
    free((*counter).path as *mut core::ffi::c_void);
    free(counter as *mut core::ffi::c_void);
}

unsafe extern "C" fn create_query() -> *mut query {
    let mut query: *mut query = core::ptr::null_mut::<query>();
    query = calloc((1 as size_t) as _, (core::mem::size_of::<query>() as size_t) as _) as *mut query;
    if !query.is_null() {
        (*query).magic = PDH_MAGIC_QUERY as DWORD;
        list_init(core::ptr::addr_of_mut!((*query).counters));
        return query;
    }
    return core::ptr::null_mut::<query>();
}

unsafe extern "C" fn destroy_query(query: *mut query) {
    /* Ensure compiler doesn't optimize out the assignment with 0. */
    SecureZeroMemory(&raw mut (*query).magic as PVOID, core::mem::size_of::<DWORD>() as SIZE_T);
    free(query as *mut core::ffi::c_void);
}

unsafe extern "system" fn collect_processor_time(counter: *mut counter) {
    (*counter).two.largevalue = 500000 as LONGLONG; /* FIXME */
    (*counter).status = PDH_CSTATUS_VALID_DATA as DWORD;
}

unsafe extern "system" fn collect_uptime(counter: *mut counter) {
    (*counter).two.largevalue = GetTickCount64() as LONGLONG;
    (*counter).status = PDH_CSTATUS_VALID_DATA as DWORD;
}

unsafe extern "C" fn is_local_machine(name: *const WCHAR, len: DWORD) -> BOOL {
    let mut buf: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as _] = [0; (MAX_COMPUTERNAME_LENGTH + 1) as _];
    let mut buflen: DWORD = ARRAY_SIZE!(buf) as DWORD;

    if GetComputerNameW(buf.as_mut_ptr(), core::ptr::addr_of_mut!(buflen)) == 0 {
        return (FALSE) as BOOL;
    }
    return (len == buflen
        && _wcsnicmp(name, buf.as_ptr(), buflen as usize) == 0) as BOOL;
}

unsafe extern "C" fn pdh_match_path(fullpath: LPCWSTR, mut path: LPCWSTR) -> BOOL {
    let mut p: LPCWSTR = core::ptr::null_mut();
    if *path.offset(0 as isize) == '\\' as u16
        && *path.offset(1 as isize) == '\\' as u16
        && {
            p = wcschr(path.offset(2 as isize), '\\' as u16) as LPCWSTR;
            !p.is_null()
        }
        && is_local_machine(
            path.offset(2 as isize),
            (p.offset_from(path) as i64 - 2 as i64) as DWORD,
        ) != 0
    {
        path = path.offset(p.offset_from(path) as isize);
    }
    if !wcschr(path, '\\' as u16).is_null() {
        p = fullpath;
    } else {
        p = wcsrchr(fullpath, '\\' as u16).offset(1 as isize);
    }
    return (wcscmp(p, path) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PdhAddCounterA(
    query: PDH_HQUERY,
    path: LPCSTR,
    userdata: DWORD_PTR,
    counter: *mut PDH_HCOUNTER,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut pathW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PdhAddCounterA\0" as *const u8 as *const core::ffi::c_char,
            225 as core::ffi::c_int,
            b"%p %s %Ix %p\n\0" as *const u8 as *const core::ffi::c_char,
            query,
            debugstr_a(path as *const core::ffi::c_char),
            userdata,
            counter,
        );
    }
    if path.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    pathW = pdh_strdup_aw(path as *const core::ffi::c_char);
    if pathW.is_null() {
        return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
    }
    ret = PdhAddCounterW(query, pathW as LPCWSTR, userdata, counter);
    free(pathW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhAddCounterW(
    mut hquery: PDH_HQUERY,
    mut path: LPCWSTR,
    mut userdata: DWORD_PTR,
    mut hcounter: *mut PDH_HCOUNTER,
) -> PDH_STATUS {
    let mut query: *mut query = hquery as *mut query;
    let mut counter: *mut counter = core::ptr::null_mut::<counter>();
    let mut i: core::ffi::c_uint = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PdhAddCounterW\0" as *const u8 as *const core::ffi::c_char,
            248 as core::ffi::c_int,
            b"%p %s %Ix %p\n\0" as *const u8 as *const core::ffi::c_char,
            hquery,
            debugstr_w(path as *const WCHAR),
            userdata,
            hcounter,
        );
    }
    if path.is_null() || hcounter.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    if query.is_null() || (*query).magic != PDH_MAGIC_QUERY {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return (PDH_INVALID_HANDLE) as PDH_STATUS;
    }
    *hcounter = core::ptr::null_mut();
    i = 0;
    while (i as usize) < (ARRAY_SIZE!(counter_sources) as usize) {
        if pdh_match_path(counter_sources[i as usize].path, path) != 0 {
            counter = create_counter();
            if !counter.is_null() {
                (*counter).path =
                    _wcsdup(counter_sources[i as usize].path as *const WCHAR) as *mut WCHAR;
                (*counter).collect = counter_sources[i as usize].collect;
                (*counter).r#type = counter_sources[i as usize].r#type;
                (*counter).defaultscale = counter_sources[i as usize].scale;
                (*counter).base = counter_sources[i as usize].base;
                (*counter).queryuser = (*query).user;
                (*counter).user = userdata;
                list_add_tail(&raw mut (*query).counters, &raw mut (*counter).entry);
                *hcounter = counter as PDH_HCOUNTER;
                LeaveCriticalSection(&raw mut pdh_handle_cs);
                return (ERROR_SUCCESS) as PDH_STATUS;
            }
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            return (PDH_MEMORY_ALLOCATION_FAILURE) as PDH_STATUS;
        }
        i = i.wrapping_add(1);
    }
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return (PDH_CSTATUS_NO_COUNTER) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhAddEnglishCounterA(
    mut query: PDH_HQUERY,
    mut path: LPCSTR,
    mut userdata: DWORD_PTR,
    mut counter: *mut PDH_HCOUNTER,
) -> PDH_STATUS {
    if counter.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if query.is_null() {
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    return PdhAddCounterA(query, path, userdata, counter);
}

#[no_mangle]
pub unsafe extern "system" fn PdhAddEnglishCounterW(
    mut query: PDH_HQUERY,
    mut path: LPCWSTR,
    mut userdata: DWORD_PTR,
    mut counter: *mut PDH_HCOUNTER,
) -> PDH_STATUS {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhAddEnglishCounterW\0" as *const u8 as *const ::core::ffi::c_char,
            307 as ::core::ffi::c_int,
            b"%p %s %Ix %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            query,
            debugstr_w(path as *const WCHAR),
            userdata,
            counter,
        );
    }
    if counter.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if query.is_null() {
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    return PdhAddCounterW(query, path, userdata, counter);
}

unsafe extern "C" fn format_value(
    counter: *mut counter,
    format: DWORD,
    raw1: *mut value,
    raw2: *mut value,
    value: *mut PDH_FMT_COUNTERVALUE,
) -> PDH_STATUS {
    let mut factor: LONG = 0;

    factor = if (*counter).scale != 0 {
        (*counter).scale
    } else {
        (*counter).defaultscale
    };
    if format & PDH_FMT_LONG as DWORD != 0 {
        if format & PDH_FMT_1000 as DWORD != 0 {
            (*value).__bindgen_anon_1.longValue = (*raw2).longvalue * 1000 as LONG;
        } else {
            (*value).__bindgen_anon_1.longValue = ((*raw2).longvalue as ::core::ffi::c_double
                * pow(
                    10 as ::core::ffi::c_double,
                    factor as ::core::ffi::c_double,
                )) as LONG;
        }
    } else if format & PDH_FMT_LARGE as DWORD != 0 {
        if format & PDH_FMT_1000 as DWORD != 0 {
            (*value).__bindgen_anon_1.largeValue =
                (*raw2).largevalue * 1000 as LONGLONG;
        } else {
            (*value).__bindgen_anon_1.largeValue = ((*raw2).largevalue as ::core::ffi::c_double
                * pow(
                    10 as ::core::ffi::c_double,
                    factor as ::core::ffi::c_double,
                )) as LONGLONG;
        }
    } else if format & PDH_FMT_DOUBLE as DWORD != 0 {
        if format & PDH_FMT_1000 as DWORD != 0 {
            (*value).__bindgen_anon_1.doubleValue =
                (*raw2).doublevalue * 1000 as ::core::ffi::c_double;
        } else {
            (*value).__bindgen_anon_1.doubleValue = (*raw2).doublevalue
                * pow(
                    10 as ::core::ffi::c_double,
                    factor as ::core::ffi::c_double,
                );
        }
    } else {
        WARN!("unknown format %lx\n", format);
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhCalculateCounterFromRawValue(
    mut handle: PDH_HCOUNTER,
    mut format: DWORD,
    mut raw1: PPDH_RAW_COUNTER,
    mut raw2: PPDH_RAW_COUNTER,
    mut value: PPDH_FMT_COUNTERVALUE,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut counter: *mut counter = handle as *mut counter;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhCalculateCounterFromRawValue\0" as *const u8 as *const ::core::ffi::c_char,
            354 as ::core::ffi::c_int,
            b"%p 0x%08lx %p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            format,
            raw1,
            raw2,
            value,
        );
    }
    if value.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    ret = format_value(
        counter,
        format,
        &raw mut (*raw1).SecondValue as *mut value,
        &raw mut (*raw2).SecondValue as *mut value,
        value as *mut PDH_FMT_COUNTERVALUE,
    );
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhCloseQuery(mut handle: PDH_HQUERY) -> PDH_STATUS {
    let mut query: *mut query = handle as *mut query;
    let mut item: *mut list = ::core::ptr::null_mut::<list>();
    let mut next: *mut list = ::core::ptr::null_mut::<list>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhCloseQuery\0" as *const u8 as *const ::core::ffi::c_char,
            381 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
        );
    }

    EnterCriticalSection(&raw mut pdh_handle_cs);
    if query.is_null() || (*query).magic != PDH_MAGIC_QUERY as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }

    if !(*query).thread.is_null() {
        let mut thread: HANDLE = (*query).thread;
        SetEvent((*query).stop);
        LeaveCriticalSection(&raw mut pdh_handle_cs);

        WaitForSingleObject(thread, INFINITE as DWORD);

        EnterCriticalSection(&raw mut pdh_handle_cs);
        if (*query).magic != PDH_MAGIC_QUERY as DWORD {
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            return ERROR_SUCCESS as PDH_STATUS;
        }
        CloseHandle((*query).stop);
        CloseHandle((*query).thread);
        (*query).thread = ::core::ptr::null_mut::<::core::ffi::c_void>() as HANDLE;
    }

    item = (*query).counters.next;
    next = (*item).next;
    while item != &raw mut (*query).counters {
        let mut counter: *mut counter = (item as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(counter, entry) as isize),
        ) as *mut counter;

        list_remove(&raw mut (*counter).entry);
        destroy_counter(counter);
        item = next;
        next = (*item).next;
    }

    destroy_query(query);

    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ERROR_SUCCESS as PDH_STATUS;
}

unsafe extern "C" fn collect_query_data(query: *mut query) {
    let mut item: *mut list = core::ptr::null_mut();
    item = (*query).counters.next;
    while item != core::ptr::addr_of_mut!((*query).counters) {
        let mut time: SYSTEMTIME = core::mem::zeroed();
        let counter: *mut counter = (item as *mut u8)
            .offset(-(core::mem::offset_of!(counter, entry) as isize)) as *mut counter;
        ((*counter).collect.unwrap_unchecked())(counter);
        GetLocalTime(core::ptr::addr_of_mut!(time));
        SystemTimeToFileTime(
            core::ptr::addr_of_mut!(time),
            core::ptr::addr_of_mut!((*counter).stamp),
        );
        item = (*item).next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn PdhCollectQueryData(handle: PDH_HQUERY) -> PDH_STATUS {
    let mut query: *mut query = handle as *mut query;

    TRACE!("%p\n", handle);

    EnterCriticalSection(&raw mut pdh_handle_cs);
    if query.is_null() || (*query).magic != PDH_MAGIC_QUERY {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return (PDH_INVALID_HANDLE) as PDH_STATUS;
    }

    if list_empty(&raw mut (*query).counters) != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return (PDH_NO_DATA) as PDH_STATUS;
    }

    collect_query_data(query);

    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return (ERROR_SUCCESS) as PDH_STATUS;
}

unsafe extern "system" fn collect_query_thread(arg: *mut ::core::ffi::c_void) -> DWORD {
    let mut query: *mut query = arg as *mut query;
    let mut interval: DWORD = (*query).interval;
    let mut stop: HANDLE = (*query).stop;

    loop {
        if WaitForSingleObject(stop, interval) != WAIT_TIMEOUT as DWORD {
            ExitThread(0 as DWORD);
        }

        EnterCriticalSection(&raw mut pdh_handle_cs);
        if (*query).magic != PDH_MAGIC_QUERY as DWORD {
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            ExitThread(PDH_INVALID_HANDLE as DWORD);
        }

        collect_query_data(query);

        if SetEvent((*query).wait) == 0 {
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            ExitThread(0 as DWORD);
        }
        LeaveCriticalSection(&raw mut pdh_handle_cs);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PdhCollectQueryDataEx(
    handle: PDH_HQUERY,
    interval: DWORD,
    event: HANDLE,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut query: *mut query = handle as *mut query;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhCollectQueryDataEx\0" as *const u8 as *const ::core::ffi::c_char,
            504 as ::core::ffi::c_int,
            b"%p %ld %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            interval,
            event,
        );
    }

    EnterCriticalSection(&raw mut pdh_handle_cs);
    if query.is_null() || (*query).magic != PDH_MAGIC_QUERY {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return (PDH_INVALID_HANDLE) as PDH_STATUS;
    }
    if list_empty(&raw mut (*query).counters) != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return (PDH_NO_DATA) as PDH_STATUS;
    }
    if !(*query).thread.is_null() {
        let mut thread: HANDLE = (*query).thread;
        SetEvent((*query).stop);
        LeaveCriticalSection(&raw mut pdh_handle_cs);

        WaitForSingleObject(thread, INFINITE);

        EnterCriticalSection(&raw mut pdh_handle_cs);
        if (*query).magic != PDH_MAGIC_QUERY {
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            return (PDH_INVALID_HANDLE) as PDH_STATUS;
        }
        CloseHandle((*query).thread);
        (*query).thread = core::ptr::null_mut();
    } else {
        (*query).stop = CreateEventW(
            core::ptr::null_mut(),
            (FALSE) as BOOL,
            (FALSE) as BOOL,
            core::ptr::null(),
        );
        if (*query).stop.is_null() {
            ret = GetLastError() as PDH_STATUS;
            LeaveCriticalSection(&raw mut pdh_handle_cs);
            return ret;
        }
    }
    (*query).wait = event;
    (*query).interval = interval.wrapping_mul(1000);
    (*query).thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(collect_query_thread),
        query as PVOID,
        0,
        core::ptr::null_mut(),
    );
    if (*query).thread.is_null() {
        ret = GetLastError() as PDH_STATUS;
        CloseHandle((*query).stop);

        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return ret;
    }

    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return (ERROR_SUCCESS) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhCollectQueryDataWithTime(
    mut handle: PDH_HQUERY,
    mut timestamp: *mut LONGLONG,
) -> PDH_STATUS {
    let mut query: *mut query = handle as *mut query;
    let mut counter: *mut counter = core::ptr::null_mut::<counter>();
    let mut item: *mut list = core::ptr::null_mut::<list>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PdhCollectQueryDataWithTime\0" as *const u8 as *const core::ffi::c_char,
            564 as core::ffi::c_int,
            b"%p %p\n\0" as *const u8 as *const core::ffi::c_char,
            handle,
            timestamp,
        );
    }
    if timestamp.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    if query.is_null() || (*query).magic != PDH_MAGIC_QUERY as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if list_empty(&raw mut (*query).counters) != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_NO_DATA as PDH_STATUS;
    }
    collect_query_data(query);
    item = list_head(&raw mut (*query).counters);
    counter = (item as *mut core::ffi::c_char)
        .offset(-(core::mem::offset_of!(counter, entry) as isize)) as *mut counter;
    *timestamp = ((*counter).stamp.dwHighDateTime as LONGLONG)
        << 32
        | (*counter).stamp.dwLowDateTime as LONGLONG;
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhExpandWildCardPathA(
    szDataSource: LPCSTR,
    szWildCardPath: LPCSTR,
    mszExpandedPathList: LPSTR,
    pcchPathListLength: LPDWORD,
    dwFlags: DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %s, %p, %p, 0x%lx: stub\n", debugstr_a(szDataSource), debugstr_a(szWildCardPath), mszExpandedPathList, pcchPathListLength, dwFlags);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhExpandWildCardPathW(
    szDataSource: LPCWSTR,
    szWildCardPath: LPCWSTR,
    mszExpandedPathList: LPWSTR,
    pcchPathListLength: LPDWORD,
    dwFlags: DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %s, %p, %p, 0x%lx: stub\n", debugstr_w(szDataSource), debugstr_w(szWildCardPath), mszExpandedPathList, pcchPathListLength, dwFlags);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhExpandCounterPathA(
    szWildCardPath: LPCSTR,
    mszExpandedPathList: LPSTR,
    pcchPathListLength: LPDWORD,
) -> PDH_STATUS {
    return PdhExpandWildCardPathA(core::ptr::null(), szWildCardPath, mszExpandedPathList, pcchPathListLength, 0);
}

#[no_mangle]
pub unsafe extern "system" fn PdhExpandCounterPathW(
    szWildCardPath: LPCWSTR,
    mszExpandedPathList: LPWSTR,
    pcchPathListLength: LPDWORD,
) -> PDH_STATUS {
    return PdhExpandWildCardPathW(
        core::ptr::null(),
        szWildCardPath,
        mszExpandedPathList,
        pcchPathListLength,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetCounterInfoA(
    handle: PDH_HCOUNTER,
    text: BOOLEAN,
    size: LPDWORD,
    info: PPDH_COUNTER_INFO_A,
) -> PDH_STATUS {
    let mut counter: *mut counter = handle as *mut counter;

    TRACE!("%p %d %p %p\n", handle, text as i32, size, info);

    EnterCriticalSection(&raw mut pdh_handle_cs);
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if size.is_null() {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if (*size as usize) < core::mem::size_of::<PDH_COUNTER_INFO_A>() as usize {
        *size = core::mem::size_of::<PDH_COUNTER_INFO_A>() as DWORD;
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_MORE_DATA as PDH_STATUS;
    }

    memset(
        info as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<PDH_COUNTER_INFO_A>() as usize,
    );

    (*info).dwType = (*counter).r#type;
    (*info).CStatus = (*counter).status;
    (*info).lScale = (*counter).scale;
    (*info).lDefaultScale = (*counter).defaultscale;
    (*info).dwUserData = (*counter).user;
    (*info).dwQueryUserData = (*counter).queryuser;

    *size = core::mem::size_of::<PDH_COUNTER_INFO_A>() as DWORD;

    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetCounterInfoW(
    handle: PDH_HCOUNTER,
    text: BOOLEAN,
    size: LPDWORD,
    info: PPDH_COUNTER_INFO_W,
) -> PDH_STATUS {
    let counter: *mut counter = handle as *mut counter;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhGetCounterInfoW\0" as *const u8 as *const ::core::ffi::c_char,
            676 as ::core::ffi::c_int,
            b"%p %d %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            text as ::core::ffi::c_int,
            size,
            info,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if size.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if (*size as usize) < (core::mem::size_of::<PDH_COUNTER_INFO_W>() as usize) {
        *size = core::mem::size_of::<PDH_COUNTER_INFO_W>() as DWORD;
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_MORE_DATA as PDH_STATUS;
    }

    memset(
        info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (core::mem::size_of::<PDH_COUNTER_INFO_W>() as usize) as size_t,
    );

    (*info).dwType = (*counter).r#type;
    (*info).CStatus = (*counter).status;
    (*info).lScale = (*counter).scale;
    (*info).lDefaultScale = (*counter).defaultscale;
    (*info).dwUserData = (*counter).user;
    (*info).dwQueryUserData = (*counter).queryuser;

    *size = core::mem::size_of::<PDH_COUNTER_INFO_W>() as DWORD;

    LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetCounterTimeBase(
    mut handle: PDH_HCOUNTER,
    mut base: *mut LONGLONG,
) -> PDH_STATUS {
    let mut counter: *mut counter = handle as *mut counter;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhGetCounterTimeBase\0" as *const u8 as *const ::core::ffi::c_char,
            718 as ::core::ffi::c_int,
            b"%p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            base,
        );
    }
    if base.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    *base = (*counter).base;
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetDllVersion(version: LPDWORD) -> PDH_STATUS {
    if version.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }

    *version = PDH_VERSION as DWORD;

    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetFormattedCounterValue(
    handle: PDH_HCOUNTER,
    format: DWORD,
    type_0: LPDWORD,
    value: PPDH_FMT_COUNTERVALUE,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut counter: *mut counter = handle as *mut counter;

    TRACE!(
        "%p %lx %p %p\n",
        handle,
        format,
        type_0,
        value
    );

    if value.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }

    EnterCriticalSection(&raw mut pdh_handle_cs);
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if (*counter).status != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_DATA as PDH_STATUS;
    }
    ret = format_value(
        counter,
        format,
        &raw mut (*counter).one,
        &raw mut (*counter).two,
        value as *mut PDH_FMT_COUNTERVALUE,
    );
    if ret == 0 {
        (*value).CStatus = ERROR_SUCCESS as DWORD;
        if !type_0.is_null() {
            *type_0 = (*counter).r#type;
        }
    }

    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ret;
}

unsafe extern "C" fn get_formatted_counter_array(
    mut handle: PDH_HCOUNTER,
    mut format: DWORD,
    mut bufsize: LPDWORD,
    mut count: LPDWORD,
    mut buffer: *mut ::core::ffi::c_void,
    mut wide: BOOL,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS;
    let mut counter: *mut counter = handle as *mut counter;
    let mut value: PDH_FMT_COUNTERVALUE = core::mem::zeroed();
    let mut needed: DWORD = (core::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>() as usize)
        .wrapping_add(if wide != 0 {
            core::mem::size_of::<WCHAR>() as usize
        } else {
            core::mem::size_of::<CHAR>() as usize
        }) as DWORD;

    /* Only single-instance counters exist here, so the array always has one item. */
    *count = 1 as DWORD;

    if buffer.is_null() || *bufsize < needed {
        *bufsize = needed;
        return PDH_MORE_DATA as PDH_STATUS;
    }

    ret = format_value(
        counter,
        format,
        &raw mut (*counter).one,
        &raw mut (*counter).two,
        &raw mut value,
    );
    if ret == 0 {
        let mut item: PPDH_FMT_COUNTERVALUE_ITEM_W = buffer as PPDH_FMT_COUNTERVALUE_ITEM_W;
        let mut name: *mut ::core::ffi::c_char = (buffer as *mut ::core::ffi::c_char)
            .offset(core::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>() as usize as isize);
        (*item).FmtValue = value;
        (*item).FmtValue.CStatus = ERROR_SUCCESS as DWORD;
        if wide != 0 {
            (*item).szName = name as LPWSTR;
            *(name as *mut WCHAR) = 0 as WCHAR;
        } else {
            (*(item as PPDH_FMT_COUNTERVALUE_ITEM_A)).szName = name as LPSTR;
            *name = 0 as ::core::ffi::c_char;
        }
    }
    *bufsize = needed;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetFormattedCounterArrayW(
    mut handle: PDH_HCOUNTER,
    mut format: DWORD,
    mut bufsize: LPDWORD,
    mut count: LPDWORD,
    mut buffer: PPDH_FMT_COUNTERVALUE_ITEM_W,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut counter: *mut counter = ::core::ptr::null_mut::<counter>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhGetFormattedCounterArrayW\0" as *const u8 as *const ::core::ffi::c_char,
            830 as ::core::ffi::c_int,
            b"%p %lx %p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            format,
            bufsize,
            count,
            buffer,
        );
    }
    if bufsize.is_null() || count.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    counter = handle as *mut counter;
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if (*counter).status != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_DATA as PDH_STATUS;
    }
    ret = get_formatted_counter_array(
        handle,
        format,
        bufsize,
        count,
        buffer as *mut ::core::ffi::c_void,
        TRUE as BOOL,
    );
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetFormattedCounterArrayA(
    mut handle: PDH_HCOUNTER,
    mut format: DWORD,
    mut bufsize: LPDWORD,
    mut count: LPDWORD,
    mut buffer: PPDH_FMT_COUNTERVALUE_ITEM_A,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut counter: *mut counter = core::ptr::null_mut::<counter>();
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const i8,
            b"PdhGetFormattedCounterArrayA\0" as *const u8 as *const i8,
            863 as i32,
            b"%p %lx %p %p %p\n\0" as *const u8 as *const i8,
            handle,
            format,
            bufsize,
            count,
            buffer,
        );
    }
    if bufsize.is_null() || count.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    counter = handle as *mut counter;
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if (*counter).status != 0 {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_DATA as PDH_STATUS;
    }
    ret = get_formatted_counter_array(
        handle,
        format,
        bufsize,
        count,
        buffer as *mut core::ffi::c_void,
        FALSE as BOOL,
    );
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetRawCounterValue(
    handle: PDH_HCOUNTER,
    type_0: LPDWORD,
    value: PPDH_RAW_COUNTER,
) -> PDH_STATUS {
    let counter: *mut counter = handle as *mut counter;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhGetRawCounterValue\0" as *const u8 as *const ::core::ffi::c_char,
            894 as ::core::ffi::c_int,
            b"%p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            type_0,
            value,
        );
    }
    if value.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    EnterCriticalSection(&raw mut pdh_handle_cs);
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(&raw mut pdh_handle_cs);
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    (*value).CStatus = (*counter).status;
    (*value).TimeStamp.dwLowDateTime = (*counter).stamp.dwLowDateTime;
    (*value).TimeStamp.dwHighDateTime = (*counter).stamp.dwHighDateTime;
    (*value).FirstValue = (*counter).one.largevalue;
    (*value).SecondValue = (*counter).two.largevalue;
    (*value).MultiCount = 1 as DWORD;
    if !type_0.is_null() {
        *type_0 = (*counter).r#type;
    }
    LeaveCriticalSection(&raw mut pdh_handle_cs);
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhLookupPerfIndexByNameA(
    machine: LPCSTR,
    name: LPCSTR,
    index: LPDWORD,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut machineW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut nameW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PdhLookupPerfIndexByNameA\0" as *const u8 as *const core::ffi::c_char,
            927 as core::ffi::c_int,
            b"%s %s %p\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_a(machine as *const core::ffi::c_char),
            debugstr_a(name as *const core::ffi::c_char),
            index,
        );
    }
    if name.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if !machine.is_null() && {
        machineW = pdh_strdup_aw(machine as *const core::ffi::c_char);
        machineW.is_null()
    } {
        return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
    }
    nameW = pdh_strdup_aw(name as *const core::ffi::c_char);
    if nameW.is_null() {
        return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
    }
    ret = PdhLookupPerfIndexByNameW(machineW as LPCWSTR, nameW as LPCWSTR, index);
    free(nameW as *mut core::ffi::c_void);
    free(machineW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhLookupPerfIndexByNameW(
    machine: LPCWSTR,
    name: LPCWSTR,
    index: LPDWORD,
) -> PDH_STATUS {
    let mut i: ::core::ffi::c_uint;

    TRACE!("%s %s %p\n", debugstr_w(machine), debugstr_w(name), index);

    if name.is_null() || index.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }

    if !machine.is_null() {
        FIXME!("remote machine not supported\n");
        return (PDH_CSTATUS_NO_MACHINE) as PDH_STATUS;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(counter_sources) as usize {
        if pdh_match_path(counter_sources[i as usize].path, name) != 0 {
            *index = counter_sources[i as usize].index;
            return (ERROR_SUCCESS) as PDH_STATUS;
        }
        i = i.wrapping_add(1);
    }
    return (PDH_STRING_NOT_FOUND) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhLookupPerfNameByIndexA(
    machine: LPCSTR,
    index: DWORD,
    buffer: LPSTR,
    size: LPDWORD,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut machineW: *mut WCHAR = core::ptr::null_mut();
    let mut bufferW: [WCHAR; (PDH_MAX_COUNTER_NAME) as _] = [0; (PDH_MAX_COUNTER_NAME) as _];
    let mut sizeW: DWORD = ARRAY_SIZE!(bufferW) as DWORD;
    TRACE!(
        "%s %ld %p %p\n",
        debugstr_a(machine as *const i8),
        index,
        buffer,
        size
    );
    if buffer.is_null() || size.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if !machine.is_null() && {
        machineW = pdh_strdup_aw(machine as *const i8);
        machineW.is_null()
    } {
        return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
    }
    ret = PdhLookupPerfNameByIndexW(
        machineW as LPCWSTR,
        index,
        core::ptr::addr_of_mut!(bufferW) as LPWSTR,
        core::ptr::addr_of_mut!(sizeW),
    );
    if ret == 0 {
        let mut required: i32 = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(bufferW) as *mut WCHAR as LPCWSTR,
            -(1 as i32),
            core::ptr::null_mut::<CHAR>(),
            0 as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        if *size < required as DWORD {
            ret = PDH_MORE_DATA as PDH_STATUS;
        } else {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(bufferW) as *mut WCHAR as LPCWSTR,
                -(1 as i32),
                buffer,
                required,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        }
        *size = required as DWORD;
    }
    free(machineW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhLookupPerfNameByIndexW(
    machine: LPCWSTR,
    index: DWORD,
    buffer: LPWSTR,
    size: LPDWORD,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut i: core::ffi::c_uint;

    if !machine.is_null() {
        return (PDH_CSTATUS_NO_MACHINE) as PDH_STATUS;
    }

    if buffer.is_null() || size.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }
    if index == 0 {
        return (ERROR_SUCCESS) as PDH_STATUS;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(counter_sources) as usize {
        if counter_sources[i as usize].index == index {
            let mut p: *mut WCHAR = wcsrchr(
                counter_sources[i as usize].path as *const WCHAR,
                '\\' as u16,
            )
            .offset(1);
            let mut required: core::ffi::c_uint =
                (lstrlenW(p as LPCWSTR) + 1) as core::ffi::c_uint;

            if *size < required as DWORD {
                ret = (PDH_MORE_DATA) as i32;
            } else {
                lstrcpyW(buffer, p as LPCWSTR);
                ret = (ERROR_SUCCESS as i32);
            }
            *size = required as DWORD;
            return ret;
        }
        i += 1;
    }
    return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhOpenQueryA(
    source: LPCSTR,
    userdata: DWORD_PTR,
    query: *mut PDH_HQUERY,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut sourceW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhOpenQueryA\0" as *const u8 as *const ::core::ffi::c_char,
            1045 as ::core::ffi::c_int,
            b"%s %Ix %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_a(source as *const ::core::ffi::c_char),
            userdata,
            query,
        );
    }
    if !source.is_null() && {
        sourceW = pdh_strdup_aw(source as *const ::core::ffi::c_char);
        sourceW.is_null()
    } {
        return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
    }
    ret = PdhOpenQueryW(sourceW as LPCWSTR, userdata, query);
    free(sourceW as *mut ::core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhOpenQueryW(
    source: LPCWSTR,
    userdata: DWORD_PTR,
    handle: *mut PDH_HQUERY,
) -> PDH_STATUS {
    let mut query: *mut query = core::ptr::null_mut::<query>();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PdhOpenQueryW\0" as *const u8 as *const core::ffi::c_char,
            1062 as i32,
            b"%s %Ix %p\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(source as *const WCHAR),
            userdata,
            handle,
        );
    }
    if handle.is_null() {
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    if !source.is_null() {
        if (*__wine_dbch___default).flags as i32
            & (1 as i32) << __WINE_DBCL_FIXME as i32
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_0) as u32,
                __dbch_0,
                b"dll/win32/pdh/pdh_main.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"PdhOpenQueryW\0" as *const u8 as *const core::ffi::c_char,
                1068 as i32,
                b"log file data source not supported\n\0" as *const u8
                    as *const core::ffi::c_char,
            );
        }
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }
    query = create_query();
    if !query.is_null() {
        (*query).user = userdata;
        *handle = query as PDH_HQUERY;
        return ERROR_SUCCESS as PDH_STATUS;
    }
    return PDH_MEMORY_ALLOCATION_FAILURE as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhRemoveCounter(handle: PDH_HCOUNTER) -> PDH_STATUS {
    let mut counter: *mut counter = handle as *mut counter;

    TRACE!("%p\n", handle);

    EnterCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }

    list_remove(core::ptr::addr_of_mut!((*counter).entry));
    destroy_counter(counter);

    LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhSetCounterScaleFactor(handle: PDH_HCOUNTER, factor: LONG) -> PDH_STATUS {
    let counter: *mut counter = handle as *mut counter;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"PdhSetCounterScaleFactor\0" as *const u8 as *const ::core::ffi::c_char,
            1111 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
        );
    }
    EnterCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    if counter.is_null() || (*counter).magic != PDH_MAGIC_COUNTER as DWORD {
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_INVALID_HANDLE as PDH_STATUS;
    }
    if factor < PDH_MIN_SCALE as LONG || factor > PDH_MAX_SCALE as LONG {
        LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
        return PDH_INVALID_ARGUMENT as PDH_STATUS;
    }

    (*counter).scale = factor;

    LeaveCriticalSection(core::ptr::addr_of_mut!(pdh_handle_cs));
    return ERROR_SUCCESS as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhValidatePathA(path: LPCSTR) -> PDH_STATUS {
    let mut ret: PDH_STATUS = 0;
    let mut pathW: *mut WCHAR = core::ptr::null_mut();
    TRACE!("%s\n", debugstr_a(path as *const i8));
    if path.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }
    pathW = pdh_strdup_aw(path as *const i8);
    if pathW.is_null() {
        return (PDH_MEMORY_ALLOCATION_FAILURE) as PDH_STATUS;
    }

    ret = PdhValidatePathW(pathW as LPCWSTR);

    free(pathW as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn validate_path(mut path: LPCWSTR) -> PDH_STATUS {
    if path.is_null() || *path == 0 {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }
    let fresh15 = path;
    path = path.add(1);
    if *fresh15 != '\\' as u16
        || wcschr(path, '\\' as u16).is_null()
    {
        return (PDH_CSTATUS_BAD_COUNTERNAME) as PDH_STATUS;
    }
    return (ERROR_SUCCESS) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhValidatePathW(path: LPCWSTR) -> PDH_STATUS {
    let mut ret: PDH_STATUS;
    let mut i: u32;
    TRACE!("%s\n", debugstr_w(path));
    ret = validate_path(path);
    if ret != 0 {
        return ret;
    }
    i = 0;
    while i < ARRAY_SIZE!(counter_sources) as u32 {
        if pdh_match_path(counter_sources[i as usize].path, path) != 0 {
            return (ERROR_SUCCESS) as PDH_STATUS;
        }
        i = i.wrapping_add(1);
    }
    return (PDH_CSTATUS_NO_COUNTER) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhVbAddCounter(
    mut query: PDH_HQUERY,
    mut path: LPCSTR,
    mut counter: *mut PDH_HCOUNTER,
) -> PDH_STATUS {
    FIXME!("%p, %s, %p: stub!\n", query, debugstr_a(path as *const i8), counter);
    if path.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }
    return (PDH_NOT_IMPLEMENTED) as PDH_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn PdhVbGetDoubleCounterValue(
    mut handle: PDH_HCOUNTER,
    mut counter_status: *mut PDH_STATUS,
) -> ::core::ffi::c_double {
    let mut value: PDH_FMT_COUNTERVALUE = _PDH_FMT_COUNTERVALUE {
        CStatus: 0,
        __bindgen_anon_1: _PDH_FMT_COUNTERVALUE__bindgen_ty_1 { longValue: 0 },
    };
    let mut status: PDH_STATUS = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/pdh/pdh_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PdhVbGetDoubleCounterValue\0" as *const u8 as *const ::core::ffi::c_char,
            1195 as ::core::ffi::c_int,
            b"%p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            counter_status,
        );
    }
    memset(
        &raw mut value as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<PDH_FMT_COUNTERVALUE>() as size_t,
    );
    status = PdhGetFormattedCounterValue(
        handle,
        PDH_FMT_DOUBLE as DWORD,
        ::core::ptr::null_mut::<DWORD>(),
        &raw mut value,
    );
    if !counter_status.is_null() {
        *counter_status = status;
    }
    return value.__bindgen_anon_1.doubleValue;
}

#[no_mangle]
pub unsafe extern "system" fn PdhValidatePathExA(source: PDH_HLOG, path: LPCSTR) -> PDH_STATUS {
    TRACE!("%p %s\n", source, debugstr_a(path));
    if !source.is_null() {
        FIXME!("log file data source not supported\n");
        return ERROR_SUCCESS as PDH_STATUS;
    }
    return PdhValidatePathA(path);
}

#[no_mangle]
pub unsafe extern "system" fn PdhValidatePathExW(source: PDH_HLOG, path: LPCWSTR) -> PDH_STATUS {
    TRACE!("%p %s\n", source, debugstr_w(path as *const WCHAR));
    if !source.is_null() {
        FIXME!("log file data source not supported\n");
        return (ERROR_SUCCESS) as PDH_STATUS;
    }
    return PdhValidatePathW(path);
}

#[no_mangle]
pub unsafe extern "system" fn PdhMakeCounterPathA(
    e: *mut PDH_COUNTER_PATH_ELEMENTS_A,
    buffer: LPSTR,
    buflen: LPDWORD,
    flags: DWORD,
) -> PDH_STATUS {
    let mut ret: PDH_STATUS = (PDH_MEMORY_ALLOCATION_FAILURE) as i32;
    let mut eW: PDH_COUNTER_PATH_ELEMENTS_W = core::mem::zeroed();
    let mut bufferW: *mut WCHAR = core::ptr::null_mut();
    let mut buflenW: DWORD = 0;

    if e.is_null() || buflen.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }

    memset(
        &raw mut eW as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<PDH_COUNTER_PATH_ELEMENTS_W>(),
    );

    'done: {
        if (!(*e).szMachineName.is_null() && {
            eW.szMachineName = pdh_strdup_aw((*e).szMachineName as *const core::ffi::c_char) as LPWSTR;
            eW.szMachineName.is_null()
        }) {
            break 'done;
        }
        if (!(*e).szObjectName.is_null() && {
            eW.szObjectName = pdh_strdup_aw((*e).szObjectName as *const core::ffi::c_char) as LPWSTR;
            eW.szObjectName.is_null()
        }) {
            break 'done;
        }
        if (!(*e).szInstanceName.is_null() && {
            eW.szInstanceName = pdh_strdup_aw((*e).szInstanceName as *const core::ffi::c_char) as LPWSTR;
            eW.szInstanceName.is_null()
        }) {
            break 'done;
        }
        if (!(*e).szParentInstance.is_null() && {
            eW.szParentInstance = pdh_strdup_aw((*e).szParentInstance as *const core::ffi::c_char) as LPWSTR;
            eW.szParentInstance.is_null()
        }) {
            break 'done;
        }
        if (!(*e).szCounterName.is_null() && {
            eW.szCounterName = pdh_strdup_aw((*e).szCounterName as *const core::ffi::c_char) as LPWSTR;
            eW.szCounterName.is_null()
        }) {
            break 'done;
        }
        eW.dwInstanceIndex = (*e).dwInstanceIndex;

        buflenW = 0;
        ret = PdhMakeCounterPathW(
            &raw mut eW,
            core::ptr::null_mut::<WCHAR>(),
            &raw mut buflenW,
            flags,
        );
        if (ret) as u32 == PDH_MORE_DATA {
            bufferW = malloc(
                (buflenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if !bufferW.is_null() {
                ret = PdhMakeCounterPathW(
                    &raw mut eW,
                    bufferW as LPWSTR,
                    &raw mut buflenW,
                    flags,
                );
                if ret == 0 {
                    let len: core::ffi::c_int = WideCharToMultiByte(
                        CP_ACP,
                        0,
                        bufferW as LPCWSTR,
                        -(1),
                        core::ptr::null_mut::<CHAR>(),
                        0,
                        core::ptr::null::<CHAR>(),
                        core::ptr::null_mut::<BOOL>(),
                    );
                    if *buflen >= len as DWORD {
                        WideCharToMultiByte(
                            CP_ACP,
                            0,
                            bufferW as LPCWSTR,
                            -(1),
                            buffer,
                            *buflen as core::ffi::c_int,
                            core::ptr::null::<CHAR>(),
                            core::ptr::null_mut::<BOOL>(),
                        );
                    } else {
                        ret = (PDH_MORE_DATA) as i32;
                    }
                    *buflen = len as DWORD;
                }
                free(bufferW as *mut core::ffi::c_void);
            } else {
                ret = (PDH_MEMORY_ALLOCATION_FAILURE) as i32;
            }
        }
    }

    free(eW.szMachineName as *mut core::ffi::c_void);
    free(eW.szObjectName as *mut core::ffi::c_void);
    free(eW.szInstanceName as *mut core::ffi::c_void);
    free(eW.szParentInstance as *mut core::ffi::c_void);
    free(eW.szCounterName as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhMakeCounterPathW(
    e: *mut PDH_COUNTER_PATH_ELEMENTS_W,
    buffer: LPWSTR,
    buflen: LPDWORD,
    flags: DWORD,
) -> PDH_STATUS {
    let mut path: [WCHAR; (PDH_MAX_COUNTER_NAME) as _] = [0 as WCHAR; (PDH_MAX_COUNTER_NAME) as _];
    let mut instance: [WCHAR; 12] = [0 as WCHAR; 12];
    let mut ret: PDH_STATUS = (ERROR_SUCCESS as i32);
    let mut len: DWORD = 0;

    if flags != 0 {
        FIXME!("unimplemented flags 0x%08lx\n", flags);
    }

    if e.is_null() || (*e).szCounterName.is_null() || (*e).szObjectName.is_null() || buflen.is_null() {
        return (PDH_INVALID_ARGUMENT) as PDH_STATUS;
    }

    path[0 as usize] = 0 as WCHAR;
    if !(*e).szMachineName.is_null() {
        lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_3DBECEBCF87C.as_ptr());
        lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, (*e).szMachineName as LPCWSTR);
    }
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, (*e).szObjectName as LPCWSTR);
    if !(*e).szInstanceName.is_null() {
        lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_1A00F51F3029.as_ptr());
        if !(*e).szParentInstance.is_null() {
            lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, (*e).szParentInstance as LPCWSTR);
            lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_BCE941D196C2.as_ptr());
        }
        lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, (*e).szInstanceName as LPCWSTR);
        swprintf(
            core::ptr::addr_of_mut!(instance) as *mut WCHAR,
            (ARRAY_SIZE!(instance)) as _,
            RUSTCONV_WSTR_2157805E1164.as_ptr(),
            (*e).dwInstanceIndex,
        );
        lstrcatW(
            core::ptr::addr_of_mut!(path) as LPWSTR,
            core::ptr::addr_of_mut!(instance) as *mut WCHAR as LPCWSTR,
        );
        lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_4F2D892D6AF4.as_ptr());
    }
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    lstrcatW(core::ptr::addr_of_mut!(path) as LPWSTR, (*e).szCounterName as LPCWSTR);

    len = (lstrlenW(core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR)
        + 1) as DWORD;
    if *buflen >= len {
        lstrcpyW(buffer, core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR);
    } else {
        ret = (PDH_MORE_DATA) as i32;
    }
    *buflen = len;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PdhEnumObjectItemsA(
    szDataSource: LPCSTR,
    szMachineName: LPCSTR,
    szObjectName: LPCSTR,
    mszCounterList: LPSTR,
    pcchCounterListLength: LPDWORD,
    mszInstanceList: LPSTR,
    pcchInstanceListLength: LPDWORD,
    dwDetailLevel: DWORD,
    dwFlags: DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %s, %s, %p, %p, %p, %p, %ld, 0x%lx: stub\n", debugstr_a(szDataSource), debugstr_a(szMachineName), debugstr_a(szObjectName), mszCounterList, pcchCounterListLength, mszInstanceList, pcchInstanceListLength, dwDetailLevel, dwFlags);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhEnumObjectItemsW(
    szDataSource: LPCWSTR,
    szMachineName: LPCWSTR,
    szObjectName: LPCWSTR,
    mszCounterList: LPWSTR,
    pcchCounterListLength: LPDWORD,
    mszInstanceList: LPWSTR,
    pcchInstanceListLength: LPDWORD,
    dwDetailLevel: DWORD,
    dwFlags: DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %s, %s, %p, %p, %p, %p, %ld, 0x%lx: stub\n", debugstr_w(szDataSource), debugstr_w(szMachineName), debugstr_w(szObjectName), mszCounterList, pcchCounterListLength, mszInstanceList, pcchInstanceListLength, dwDetailLevel, dwFlags);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhSetDefaultRealTimeDataSource(source: DWORD) -> PDH_STATUS {
    // FIXME("%lu\n", source);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetLogFileTypeA(
    log: *const i8,
    r#type: *mut DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %p: stub\n", debugstr_a(log), type);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhGetLogFileTypeW(
    log: *const WCHAR,
    r#type: *mut DWORD,
) -> PDH_STATUS {
    // FIXME("%s, %p: stub\n", debugstr_w(log), type);
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhBindInputDataSourceA(
    source: *mut PDH_HLOG,
    filenamelist: *const i8,
) -> PDH_STATUS {
    // FIXME("%p %s: stub\n", source, debugstr_a(filenamelist));
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhBindInputDataSourceW(
    source: *mut PDH_HLOG,
    filenamelist: *const WCHAR,
) -> PDH_STATUS {
    // FIXME("%p %s: stub\n", source, debugstr_w(filenamelist));
    (PDH_NOT_IMPLEMENTED) as i32
}

#[no_mangle]
pub unsafe extern "system" fn PdhConnectMachineA(name: *const i8) -> PDH_STATUS {
    // FIXME("%s: stub\n", debugstr_a(name));
    (PDH_NOT_IMPLEMENTED) as i32
}
