//! git.rs — translation of dll/win32/ole32/git.c (the StdGlobalInterfaceTable
//! object: process-wide singleton marshalling interfaces between apartments
//! using cookies).
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_unsafe,
    unused_mut,
    unused_assignments,
    static_mut_refs,
    clippy::all
)]

use super::types::*;

/* Each entry in the linked list of GIT entries */
#[repr(C)]
struct StdGITEntry {
    cookie: DWORD,
    iid: IID,             /* IID of the interface */
    stream: *mut IStream, /* Holds the marshalled interface */
    entry: list,
}

/* Class data */
#[repr(C)]
struct StdGlobalInterfaceTableImpl {
    IGlobalInterfaceTable_iface: IGlobalInterfaceTable,
    list: list,
    nextCookie: ULONG,
}

static mut std_git: *mut IGlobalInterfaceTable = core::ptr::null_mut();

/* statically initialized like the C (see types::static_critsec re DebugInfo) */
static mut git_section: CRITICAL_SECTION = static_critsec();

#[inline]
unsafe fn impl_from_IGlobalInterfaceTable(
    iface: *mut IGlobalInterfaceTable,
) -> *mut StdGlobalInterfaceTableImpl {
    containing_record!(iface, StdGlobalInterfaceTableImpl, IGlobalInterfaceTable_iface)
}

/// A helper function to traverse the list and find the entry that matches the
/// cookie. Returns NULL if not found. Must be called inside git_section
/// critical section.
unsafe fn StdGlobalInterfaceTable_FindEntry(
    This: *mut StdGlobalInterfaceTableImpl,
    cookie: DWORD,
) -> *mut StdGITEntry {
    TRACE!("This=%p, cookie=0x%x\n");

    let head = core::ptr::addr_of_mut!((*This).list);
    let mut cursor = (*head).next;
    while cursor != head {
        let e = containing_record!(cursor, StdGITEntry, entry);
        if (*e).cookie == cookie {
            return e;
        }
        cursor = (*cursor).next;
    }

    TRACE!("Entry not found\n");
    core::ptr::null_mut()
}

/*
 * Here's the boring boilerplate stuff for IUnknown
 */

unsafe extern "system" fn StdGlobalInterfaceTable_QueryInterface(
    iface: *mut IGlobalInterfaceTable,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    /* Make sure silly coders can't crash us */
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut(); /* assume we don't have the interface */

    /* Do we implement that interface? */
    if IsEqualIID(&IID_IUnknown, riid) || IsEqualIID(&IID_IGlobalInterfaceTable, riid) {
        *ppvObject = iface as *mut c_void;
    } else {
        FIXME!("(%s), not supported.\n");
        return E_NOINTERFACE;
    }

    /* Now inc the refcount */
    iface.AddRef();
    S_OK
}

unsafe extern "system" fn StdGlobalInterfaceTable_AddRef(iface: *mut IGlobalInterfaceTable) -> ULONG {
    1
}

unsafe extern "system" fn StdGlobalInterfaceTable_Release(iface: *mut IGlobalInterfaceTable) -> ULONG {
    1
}

/*
 * Now implement the actual IGlobalInterfaceTable interface
 */

unsafe extern "system" fn StdGlobalInterfaceTable_RegisterInterfaceInGlobal(
    iface: *mut IGlobalInterfaceTable,
    pUnk: *mut IUnknown,
    riid: REFIID,
    pdwCookie: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IGlobalInterfaceTable(iface);
    let mut stream: *mut IStream = core::ptr::null_mut();
    let zero: LARGE_INTEGER = 0;

    TRACE!("iface=%p, pUnk=%p, riid=%s, pdwCookie=0x%p\n");

    if pUnk.is_null() {
        return E_INVALIDARG;
    }

    /* marshal the interface */
    TRACE!("About to marshal the interface\n");

    let mut hres = CreateStreamOnHGlobal(core::ptr::null_mut(), TRUE, (&raw mut stream).cast());
    if hres != S_OK {
        return hres;
    }
    hres = CoMarshalInterface(
        stream,
        riid,
        pUnk,
        MSHCTX_INPROC,
        core::ptr::null_mut(),
        MSHLFLAGS_TABLESTRONG,
    );
    if hres != S_OK {
        stream.Release();
        return hres;
    }

    stream.Seek(zero, STREAM_SEEK_SET, core::ptr::null_mut());

    let entry = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<StdGITEntry>())
        as *mut StdGITEntry;
    if entry.is_null() {
        CoReleaseMarshalData(stream);
        stream.Release();
        return E_OUTOFMEMORY;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(git_section));

    (*entry).iid = *riid;
    (*entry).stream = stream;
    (*entry).cookie = (*This).nextCookie;
    (*This).nextCookie += 1; /* inc the cookie count */

    /* insert the new entry at the end of the list */
    list_add_tail(
        core::ptr::addr_of_mut!((*This).list),
        core::ptr::addr_of_mut!((*entry).entry),
    );

    /* and return the cookie */
    *pdwCookie = (*entry).cookie;

    LeaveCriticalSection(core::ptr::addr_of_mut!(git_section));

    TRACE!("Cookie is 0x%x\n");
    S_OK
}

unsafe extern "system" fn StdGlobalInterfaceTable_RevokeInterfaceFromGlobal(
    iface: *mut IGlobalInterfaceTable,
    dwCookie: DWORD,
) -> HRESULT {
    let This = impl_from_IGlobalInterfaceTable(iface);

    TRACE!("iface=%p, dwCookie=0x%x\n");

    EnterCriticalSection(core::ptr::addr_of_mut!(git_section));

    let entry = StdGlobalInterfaceTable_FindEntry(This, dwCookie);
    if entry.is_null() {
        TRACE!("Entry not found\n");
        LeaveCriticalSection(core::ptr::addr_of_mut!(git_section));
        return E_INVALIDARG; /* not found */
    }

    list_remove(core::ptr::addr_of_mut!((*entry).entry));

    LeaveCriticalSection(core::ptr::addr_of_mut!(git_section));

    /* Free the stream */
    let hr = CoReleaseMarshalData((*entry).stream);
    if hr != S_OK {
        WARN!("Failed to release marshal data, hr = 0x%08x\n");
        return hr;
    }
    (*entry).stream.Release();

    HeapFree(GetProcessHeap(), 0, entry as PVOID);
    S_OK
}

unsafe extern "system" fn StdGlobalInterfaceTable_GetInterfaceFromGlobal(
    iface: *mut IGlobalInterfaceTable,
    dwCookie: DWORD,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IGlobalInterfaceTable(iface);
    let mut stream: *mut IStream = core::ptr::null_mut();

    TRACE!("dwCookie=0x%x, riid=%s, ppv=%p\n");

    EnterCriticalSection(core::ptr::addr_of_mut!(git_section));

    let entry = StdGlobalInterfaceTable_FindEntry(This, dwCookie);
    if entry.is_null() {
        WARN!("Entry for cookie 0x%x not found\n");
        LeaveCriticalSection(core::ptr::addr_of_mut!(git_section));
        return E_INVALIDARG;
    }

    TRACE!("entry=%p\n");

    let mut hres = (*entry).stream.Clone(&mut stream);

    LeaveCriticalSection(core::ptr::addr_of_mut!(git_section));

    if hres != S_OK {
        WARN!("Failed to clone stream with error 0x%08x\n");
        return hres;
    }

    /* unmarshal the interface */
    hres = CoUnmarshalInterface(stream, riid, ppv as *mut LPVOID);
    stream.Release();

    if hres != S_OK {
        WARN!("Failed to unmarshal stream\n");
        return hres;
    }

    TRACE!("ppv=%p\n");
    S_OK
}

/* Classfactory definition - despite what MSDN says, some programs need this */

unsafe extern "system" fn GITCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IClassFactory) {
        *ppv = iface as *mut c_void;
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn GITCF_AddRef(iface: *mut IClassFactory) -> ULONG {
    2
}

unsafe extern "system" fn GITCF_Release(iface: *mut IClassFactory) -> ULONG {
    1
}

unsafe extern "system" fn GITCF_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let git = get_std_git();
    let hr = git.QueryInterface(riid, ppv);
    git.Release();
    hr
}

unsafe extern "system" fn GITCF_LockServer(iface: *mut IClassFactory, fLock: BOOL) -> HRESULT {
    FIXME!("(%d), stub!\n");
    S_OK
}

static GITClassFactoryVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: GITCF_QueryInterface,
    AddRef: GITCF_AddRef,
    Release: GITCF_Release,
    CreateInstance: GITCF_CreateInstance,
    LockServer: GITCF_LockServer,
};

static git_classfactory: ComStatic<IClassFactory> = ComStatic(IClassFactory {
    lpVtbl: &GITClassFactoryVtbl,
});

/// (compobj_private.h: HRESULT StdGlobalInterfaceTable_GetFactory(LPVOID*))
#[no_mangle]
pub unsafe extern "C" fn StdGlobalInterfaceTable_GetFactory(ppv: *mut LPVOID) -> HRESULT {
    *ppv = core::ptr::addr_of!(git_classfactory.0) as *mut c_void;
    TRACE!("Returning GIT classfactory\n");
    S_OK
}

/* Virtual function table */
static StdGlobalInterfaceTableImpl_Vtbl: IGlobalInterfaceTableVtbl = IGlobalInterfaceTableVtbl {
    QueryInterface: StdGlobalInterfaceTable_QueryInterface,
    AddRef: StdGlobalInterfaceTable_AddRef,
    Release: StdGlobalInterfaceTable_Release,
    RegisterInterfaceInGlobal: StdGlobalInterfaceTable_RegisterInterfaceInGlobal,
    RevokeInterfaceFromGlobal: StdGlobalInterfaceTable_RevokeInterfaceFromGlobal,
    GetInterfaceFromGlobal: StdGlobalInterfaceTable_GetInterfaceFromGlobal,
};

/// (compobj_private.h: IGlobalInterfaceTable *get_std_git(void))
#[no_mangle]
pub unsafe extern "C" fn get_std_git() -> *mut IGlobalInterfaceTable {
    if std_git.is_null() {
        let newGIT = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<StdGlobalInterfaceTableImpl>(),
        ) as *mut StdGlobalInterfaceTableImpl;
        if newGIT.is_null() {
            return core::ptr::null_mut();
        }

        (*newGIT).IGlobalInterfaceTable_iface.lpVtbl = &StdGlobalInterfaceTableImpl_Vtbl;
        list_init(core::ptr::addr_of_mut!((*newGIT).list));
        (*newGIT).nextCookie = 0xf100; /* that's where windows starts, so that's where we start */

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(std_git) as *mut PVOID,
            core::ptr::addr_of_mut!((*newGIT).IGlobalInterfaceTable_iface) as PVOID,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            HeapFree(GetProcessHeap(), 0, newGIT as PVOID);
        } else {
            TRACE!("Created the GIT at %p\n");
        }
    }

    std_git
}

/// (compobj_private.h: void release_std_git(void))
#[no_mangle]
pub unsafe extern "C" fn release_std_git() {
    if std_git.is_null() {
        return;
    }

    let git = impl_from_IGlobalInterfaceTable(std_git);
    /* LIST_FOR_EACH_ENTRY_SAFE */
    let head = core::ptr::addr_of_mut!((*git).list);
    let mut cursor = (*head).next;
    while cursor != head {
        let next = (*cursor).next;
        let entry = containing_record!(cursor, StdGITEntry, entry);

        list_remove(core::ptr::addr_of_mut!((*entry).entry));

        CoReleaseMarshalData((*entry).stream);
        (*entry).stream.Release();
        HeapFree(GetProcessHeap(), 0, entry as PVOID);

        cursor = next;
    }

    HeapFree(GetProcessHeap(), 0, git as PVOID);
}
