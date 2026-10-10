// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_89E5F7D93A03: [u16; 8] = [0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut EventSourceListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut EventSourceListCs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

static mut AddNewEventSource_unused: () = ();
#[no_mangle]
pub unsafe extern "C" fn InitEventSourceList() {
    InitializeCriticalSection(core::ptr::addr_of_mut!(EventSourceListCs));
    InitializeListHead(core::ptr::addr_of_mut!(EventSourceListHead));
}

unsafe extern "C" fn DumpEventSourceList() {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut EventSource: PEVENTSOURCE = core::ptr::null_mut();

    DPRINT!(b"DumpEventSourceList()\n\0".as_ptr() as *const i8);
    EnterCriticalSection(core::ptr::addr_of_mut!(EventSourceListCs));

    CurrentEntry = (*core::ptr::addr_of_mut!(EventSourceListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(EventSourceListHead) {
        EventSource = (CurrentEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(EVENTSOURCE, EventSourceListEntry) as ULONG_PTR,
        ) as PEVENTSOURCE;

        DPRINT!(b"EventSource->szName: %S\n\0".as_ptr() as *const i8, (*EventSource).szName);

        CurrentEntry = (*CurrentEntry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(EventSourceListCs));

    DPRINT!(b"Done\n\0".as_ptr() as *const i8);
}

unsafe extern "C" fn AddNewEventSource(mut pLogFile: PLOGFILE, mut lpSourceName: PWSTR) -> BOOL {
    let mut lpEventSource: PEVENTSOURCE = core::ptr::null_mut();
    lpEventSource = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::offset_of!(EVENTSOURCE, szName)
            + (wcslen(lpSourceName as *const u16) + (1) as usize)
                * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PEVENTSOURCE;
    if !lpEventSource.is_null() {
        wcscpy(
            &raw mut (*lpEventSource).szName as *mut u16,
            lpSourceName as *const u16,
        );
        (*lpEventSource).LogFile = pLogFile;

        EnterCriticalSection(&raw mut EventSourceListCs);
        InsertTailList(
            &raw mut EventSourceListHead,
            &raw mut (*lpEventSource).EventSourceListEntry,
        );
        LeaveCriticalSection(&raw mut EventSourceListCs);
    }

    return (lpEventSource != core::ptr::null_mut()) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LoadEventSources(hKey: HKEY, pLogFile: PLOGFILE) -> BOOL {
    let mut Success: BOOL = 0;
    let mut dwNumSubKeys: DWORD = 0;
    let mut dwMaxSubKeyLength: DWORD = 0;
    let mut dwEventSourceNameLength: DWORD = 0;
    let mut MaxValueLen: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut Buf: PWSTR = core::ptr::null_mut();
    let mut SourceList: PWSTR = core::ptr::null_mut();
    let mut Source: PWSTR = core::ptr::null_mut();
    let mut cchRemaining: size_t = 0 as size_t;
    let mut Result: LONG = 0;

    DPRINT!(b"LoadEventSources\n\0".as_ptr() as PCSTR);

    Result = RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwNumSubKeys,
        &mut dwMaxSubKeyLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Result != ERROR_SUCCESS as LONG {
        DPRINT1!(b"RegQueryInfoKeyW failed: %lu\n\0".as_ptr() as PCSTR, Result);
        return FALSE as BOOL;
    }

    dwMaxSubKeyLength = dwMaxSubKeyLength.wrapping_add(1);

    Buf = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwMaxSubKeyLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if Buf.is_null() {
        DPRINT1!(b"Error: cannot allocate heap!\n\0".as_ptr() as PCSTR);
        return FALSE as BOOL;
    }

    /*
     * Allocate a buffer for storing the names of the sources as a REG_MULTI_SZ
     * in the registry. Also add the event log as its own source.
     * Add a final NULL-terminator.
     */
    MaxValueLen = (dwNumSubKeys.wrapping_mul(dwMaxSubKeyLength) as size_t)
        .wrapping_add(wcslen((*pLogFile).LogName))
        .wrapping_add(2 as size_t) as DWORD;
    SourceList = HeapAlloc(
        GetProcessHeap(),
        0,
        (MaxValueLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if SourceList.is_null() {
        DPRINT1!(b"Error: cannot allocate heap!\n\0".as_ptr() as PCSTR);
        /* It is not dramatic if we cannot create it */
    } else {
        cchRemaining = MaxValueLen as size_t;
        Source = SourceList;
    }

    /*
     * Enumerate all the subkeys of the event log key, that constitute
     * all the possible event sources for this event log. At this point,
     * skip the possible existing source having the same name as the
     * event log, it will be added later on.
     */
    dwEventSourceNameLength = dwMaxSubKeyLength;
    dwIndex = 0;
    while RegEnumKeyExW(
        hKey,
        dwIndex,
        Buf,
        &mut dwEventSourceNameLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == ERROR_SUCCESS as LONG
    {
        if _wcsicmp((*pLogFile).LogName, Buf) != 0 {
            DPRINT!(b"Event Source: %S\n\0".as_ptr() as PCSTR, Buf);
            Success = AddNewEventSource(pLogFile, Buf);
            if Success != 0 && !Source.is_null() {
                /* Append the event source name and an extra NULL-terminator */
                StringCchCopyExW(
                    Source,
                    cchRemaining,
                    Buf,
                    core::ptr::addr_of_mut!(Source),
                    core::ptr::addr_of_mut!(cchRemaining),
                    0,
                );
                if cchRemaining > 0 {
                    Source = Source.offset(1);
                    *Source = 0x0000u16;
                    cchRemaining = cchRemaining.wrapping_sub(1);
                }
            }
        }

        dwEventSourceNameLength = dwMaxSubKeyLength;
        dwIndex = dwIndex.wrapping_add(1);
    }

    /* Finally, allow the event log itself to be its own source */
    DPRINT!(b"Event Source: %S\n\0".as_ptr() as PCSTR, (*pLogFile).LogName);
    Success = AddNewEventSource(pLogFile, (*pLogFile).LogName);
    if Success != 0 && !Source.is_null() {
        /* Append the event source name and an extra NULL-terminator */
        StringCchCopyExW(
            Source,
            cchRemaining,
            (*pLogFile).LogName,
            core::ptr::addr_of_mut!(Source),
            core::ptr::addr_of_mut!(cchRemaining),
            0,
        );
        if cchRemaining > 0 {
            Source = Source.offset(1);
            *Source = 0x0000u16;
            cchRemaining = cchRemaining.wrapping_sub(1);
        }
    }

    /* Save the list of sources in the registry */
    Result = RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_89E5F7D93A03.as_ptr(),
        0,
        REG_MULTI_SZ,
        SourceList as LPBYTE as *const BYTE,
        ((MaxValueLen as usize)
            .wrapping_sub(cchRemaining as usize)
            .wrapping_add(1 as usize))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
    );
    if Result != ERROR_SUCCESS as LONG {
        DPRINT1!(b"RegSetValueExW failed: %lu\n\0".as_ptr() as PCSTR, Result);
    }

    if !SourceList.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            SourceList as PVOID,
        );
    }

    HeapFree(
        GetProcessHeap(),
        0,
        Buf as PVOID,
    );

    DumpEventSourceList();

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetEventSourceByName(Name: LPCWSTR) -> PEVENTSOURCE {
    let mut CurrentEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Item: PEVENTSOURCE = core::ptr::null_mut();
    let mut Result: PEVENTSOURCE = core::ptr::null_mut();
    EnterCriticalSection(core::ptr::addr_of_mut!(EventSourceListCs));
    CurrentEntry = (*core::ptr::addr_of_mut!(EventSourceListHead)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(EventSourceListHead) {
        Item = CONTAINING_RECORD!(CurrentEntry, EVENTSOURCE, EventSourceListEntry);
        if _wcsicmp(core::ptr::addr_of_mut!((*Item).szName) as *mut WCHAR, Name as *const WCHAR) == 0
        {
            Result = Item;
            break;
        }
        CurrentEntry = (*CurrentEntry).Flink;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(EventSourceListCs));
    return Result;
}
