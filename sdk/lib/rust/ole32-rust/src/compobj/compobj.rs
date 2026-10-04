//! compobj.rs — Rust translation of dll/win32/ole32/compobj.c (wine COMPOBJ
//! library: CoInitialize/Ex, apartments, class-object activation, registered
//! class table, CLSID/IID string functions, CoFreeUnusedLibraries).
//!
//! Faithful 1:1 translation: exact HRESULTs, error paths, refcounts, critical-
//! section usage, list manipulation. Deviations are deliberate + listed in
//! MANIFEST.md (static CS init moved to DllMain attach; ERR/FIXME forward to
//! OutputDebugStringA; StringFromGUID2's swprintf hand-rolled byte-exact;
//! C `assert` prints an ERR instead of being an NDEBUG no-op).
//!
//! Copyright (original C): 1995 Martin von Loewis, 1998 Justin Bradford,
//! 1999 Francis Beaudet, 1999 Sylvain St-Germain, 2002 Marcus Meissner,
//! 2004 Mike Hearn, 2005-2006 Robert Shearman (for CodeWeavers). LGPL 2.1+.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_variables, unused_assignments, unused_mut)]
#![allow(static_mut_refs)]

use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{null, null_mut};

use super::com::*;
use super::ffi::*;
use super::types::*;

// ---------------------------------------------------------------------------
// wine debug channel (ole). TRACE/WARN compile out (off by default at
// runtime in the C too); ERR/FIXME forward to OutputDebugStringA.
// ---------------------------------------------------------------------------
macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
macro_rules! WARN {
    ($($t:tt)*) => {{}};
}
macro_rules! ERR {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::ffi::dbg_print(format_args!(concat!("err:ole:compobj ", $fmt, "\n") $(, $e)*))
    };
}
macro_rules! FIXME {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::ffi::dbg_print(format_args!(concat!("fixme:ole:compobj ", $fmt, "\n") $(, $e)*))
    };
}
/// C `assert()` — NDEBUG no-op in the C build; here it reports loudly but
/// (like the C) does not alter control flow.
macro_rules! c_assert {
    ($cond:expr, $what:literal) => {
        if !($cond) {
            ERR!("assert failed: {}", $what);
        }
    };
}

// ---------------------------------------------------------------------------
// Cross-file externs (ole32-internal; each listed in MANIFEST.md).
// Lane B files: marshal.c, stubmanager.c, rpc.c, ftmarshal.c, git.c.
// Still-C ole32 files: ifs.c, hglobalstream.c, bindctx.c, moniker.c,
// storage32.c, oleproxy.c, defaulthandler.c, dlldata.c.
// ---------------------------------------------------------------------------

// WINAPI (stdcall on i386) exports of ole32 itself / private WINAPI helpers.
extern "system" {
    // ifs.c
    pub fn CoTaskMemAlloc(size: SIZE_T) -> PVOID;
    pub fn CoTaskMemFree(ptr: PVOID);
    // hglobalstream.c
    pub fn CreateStreamOnHGlobal(
        hGlobal: HANDLE,
        fDeleteOnRelease: BOOL,
        ppstm: *mut *mut IStream,
    ) -> HRESULT;
    // marshal.c (Lane B)
    pub fn CoMarshalInterface(
        pStream: *mut IStream,
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: PVOID,
        mshlFlags: DWORD,
    ) -> HRESULT;
    pub fn CoUnmarshalInterface(pStream: *mut IStream, riid: REFIID, ppv: *mut *mut c_void)
        -> HRESULT;
    pub fn CoReleaseMarshalData(pStream: *mut IStream) -> HRESULT;
    // bindctx.c
    pub fn CreateBindCtx(reserved: DWORD, ppbc: *mut *mut IBindCtx) -> HRESULT;
    // moniker.c
    pub fn MkParseDisplayName(
        pbc: *mut IBindCtx,
        szDisplayName: LPCOLESTR,
        pchEaten: *mut ULONG,
        ppmk: *mut *mut IMoniker,
    ) -> HRESULT;
    pub fn RunningObjectTableImpl_Initialize() -> HRESULT;
    pub fn RunningObjectTableImpl_UnInitialize() -> HRESULT;
    // storage32.c
    pub fn GetClassFile(szFilename: LPCOLESTR, pclsid: *mut CLSID) -> HRESULT;
    // oleproxy.c — ole32's own DllGetClassObject export.
    pub fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    // dlldata.c (widl-generated)
    pub fn OLE32_DllRegisterServer() -> HRESULT;
    pub fn OLE32_DllUnregisterServer() -> HRESULT;
}

// cdecl private helpers.
extern "C" {
    // marshal.c (Lane B)
    pub fn apartment_disconnectproxies(apt: *mut apartment) -> HRESULT;
    // stubmanager.c (Lane B)
    pub fn stub_manager_int_release(m: *mut stub_manager) -> ULONG;
    pub fn stub_manager_ext_addref(m: *mut stub_manager, refs: ULONG, tableweak: BOOL) -> ULONG;
    pub fn stub_manager_ext_release(
        m: *mut stub_manager,
        refs: ULONG,
        tableweak: BOOL,
        last_unlock_releases: BOOL,
    ) -> ULONG;
    pub fn get_stub_manager_from_object(
        apt: *mut APARTMENT,
        object: *mut IUnknown,
        alloc: BOOL,
    ) -> *mut stub_manager;
    pub fn stub_manager_disconnect(m: *mut stub_manager);
    // rpc.c (Lane B)
    pub fn RPC_StartLocalServer(
        clsid: REFCLSID,
        stream: *mut IStream,
        multi_use: BOOL,
        registration: *mut *mut c_void,
    ) -> HRESULT;
    pub fn RPC_StopLocalServer(registration: *mut c_void);
    pub fn RPC_GetLocalClassObject(rclsid: REFCLSID, iid: REFIID, ppv: *mut *mut c_void)
        -> HRESULT;
    pub fn RPC_RegisterChannelHook(rguid: REFGUID, hook: *mut IChannelHook) -> HRESULT;
    pub fn RPC_UnregisterAllChannelHooks();
    pub fn RPC_ExecuteCall(params: *mut dispatch_params);
    // ftmarshal.c (Lane B)
    pub fn FTMarshalCF_Create(riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    // git.c (Lane B)
    pub fn get_std_git() -> *mut IGlobalInterfaceTable;
    pub fn release_std_git();
    // defaulthandler.c
    pub fn HandlerCF_Create(rclsid: REFCLSID, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    // dlldata.c
    pub static mut hProxyDll: HINSTANCE;
    // oleproxy.c
    pub static mut GlobalOptionsCF: IClassFactory;
}

// ---------------------------------------------------------------------------
// Module-internal state (compobj.c file statics)
// ---------------------------------------------------------------------------

static mut MTA: *mut APARTMENT = null_mut(); // protected by csApartment
static mut MainApartment: *mut APARTMENT = null_mut(); // the first STA apartment
static mut apts: list = list::zeroed(); // protected by csApartment; LIST_INIT in attach

static mut csApartment: CRITICAL_SECTION = CRITICAL_SECTION_STATIC_INIT;

static mut registered_psclsid_list: list = list::zeroed(); // LIST_INIT in attach
static mut cs_registered_psclsid_list: CRITICAL_SECTION = CRITICAL_SECTION_STATIC_INIT;

/// Lock count of CoInitialize calls; COM libraries freed when it hits 0.
static mut s_COMLockCount: LONG = 0;
/// Reference count used by CoAddRefServerProcess/CoReleaseServerProcess.
static mut s_COMServerProcessReferences: LONG = 0;

static mut RegisteredClassList: list = list::zeroed(); // LIST_INIT in attach
static mut csRegisteredClassList: CRITICAL_SECTION = CRITICAL_SECTION_STATIC_INIT;

static mut openDllList: list = list::zeroed(); // LIST_INIT in attach
static mut csOpenDllList: CRITICAL_SECTION = CRITICAL_SECTION_STATIC_INIT;

static mut classes_root_hkey: HKEY = null_mut();

static mut apt_win_class: ATOM = 0;

/// function-static in CoRegisterClassObject
static mut next_cookie: LONG = 0;

/// static INIT_ONCE class_init_once = INIT_ONCE_STATIC_INIT
static mut class_init_once: INIT_ONCE = null_mut();

/// One-time module init, called from DllMain(DLL_PROCESS_ATTACH). Replaces the
/// C's static initializers for the four critical sections + four list heads
/// (Rust statics cannot be self-referential the way LIST_INIT is).
pub unsafe fn compobj_process_attach() {
    InitializeCriticalSection(&raw mut csApartment);
    InitializeCriticalSection(&raw mut cs_registered_psclsid_list);
    InitializeCriticalSection(&raw mut csRegisteredClassList);
    InitializeCriticalSection(&raw mut csOpenDllList);
    list_init(&raw mut apts);
    list_init(&raw mut registered_psclsid_list);
    list_init(&raw mut RegisteredClassList);
    list_init(&raw mut openDllList);
}

// ---------------------------------------------------------------------------
// COM_CurrentInfo / COM_CurrentApt / COM_CurrentCausalityId
// (compobj_private.h static inlines — private per-TU in C, private here)
// ---------------------------------------------------------------------------

/// will create if necessary
pub unsafe fn COM_CurrentInfo() -> *mut oletls {
    let slot = teb_reserved_for_ole();
    if (*slot).is_null() {
        let info = heap_alloc_zero(size_of::<oletls>()) as *mut oletls;
        if !info.is_null() {
            list_init(&raw mut (*info).spies);
        }
        *slot = info as PVOID;
    }
    *slot as *mut oletls
}

pub unsafe fn COM_CurrentApt() -> *mut APARTMENT {
    // NOTE: like the C, dereferences a NULL oletls on OOM.
    (*COM_CurrentInfo()).apt
}

pub unsafe fn COM_CurrentCausalityId() -> GUID {
    let info = COM_CurrentInfo();
    if info.is_null() {
        return GUID_NULL;
    }
    if guid_eq(&(*info).causality_id, &GUID_NULL) {
        CoCreateGuid(&raw mut (*info).causality_id);
        (*info).flags |= OLETLS_UUIDINITIALIZED;
    }
    (*info).causality_id
}

// ---------------------------------------------------------------------------
// activation-context miscstatus (used by ole2.c via compobj_private.h)
// ---------------------------------------------------------------------------

#[inline]
fn dvaspect_to_miscfields(aspect: DWORD) -> comclass_miscfields {
    match aspect {
        DVASPECT_CONTENT => MiscStatusContent,
        DVASPECT_THUMBNAIL => MiscStatusThumbnail,
        DVASPECT_ICON => MiscStatusIcon,
        DVASPECT_DOCPRINT => MiscStatusDocPrint,
        _ => MiscStatus,
    }
}

#[no_mangle]
pub unsafe extern "C" fn actctx_get_miscstatus(
    clsid: *const CLSID,
    aspect: DWORD,
    status: *mut DWORD,
) -> BOOL {
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();

    data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    if FindActCtxSectionGuid(
        0,
        null(),
        ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION,
        clsid,
        &raw mut data,
    ) != FALSE
    {
        let comclass = data.lpData as *mut comclassredirect_data;
        let mut misc = dvaspect_to_miscfields(aspect);

        if ((*comclass).miscmask as comclass_miscfields & misc) == 0 {
            if ((*comclass).miscmask as comclass_miscfields & MiscStatus) == 0 {
                *status = 0;
                return TRUE;
            }
            misc = MiscStatus;
        }

        match misc {
            m if m == MiscStatus => *status = (*comclass).miscstatus,
            m if m == MiscStatusIcon => *status = (*comclass).miscstatusicon,
            m if m == MiscStatusContent => *status = (*comclass).miscstatuscontent,
            m if m == MiscStatusThumbnail => *status = (*comclass).miscstatusthumbnail,
            m if m == MiscStatusDocPrint => *status = (*comclass).miscstatusdocprint,
            _ => {}
        }

        TRUE
    } else {
        FALSE
    }
}

// ---------------------------------------------------------------------------
// HKCR access (create_key / create_classes_key / open_classes_key)
// ---------------------------------------------------------------------------

/// wrapper for NtCreateKey that creates the key recursively if necessary
unsafe fn create_key(
    retkey: *mut HKEY,
    access: ACCESS_MASK,
    attr: *mut OBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let mut status = NtCreateKey(retkey as *mut HANDLE, access, attr, 0, null_mut(), 0, null_mut());

    if status == STATUS_OBJECT_NAME_NOT_FOUND {
        let mut subkey: HANDLE = null_mut();
        let root: HANDLE = (*attr).RootDirectory;
        let buffer: *mut WCHAR = (*(*attr).ObjectName).Buffer;
        let len: DWORD = (*(*attr).ObjectName).Length as DWORD / size_of::<WCHAR>() as DWORD;
        let mut pos: DWORD = 0;
        let mut i: DWORD = 0;
        let mut str_: UNICODE_STRING = core::mem::zeroed();

        while i < len && *buffer.add(i as usize) != '\\' as u16 {
            i += 1;
        }
        if i == len {
            return status;
        }

        let attrs = (*attr).Attributes;
        (*attr).ObjectName = &raw mut str_;

        while i < len {
            str_.Buffer = buffer.add(pos as usize);
            str_.Length = ((i - pos) as usize * size_of::<WCHAR>()) as u16;
            status = NtCreateKey(&raw mut subkey, access, attr, 0, null_mut(), 0, null_mut());
            if (*attr).RootDirectory != root {
                NtClose((*attr).RootDirectory);
            }
            if status != 0 {
                return status;
            }
            (*attr).RootDirectory = subkey;
            while i < len && *buffer.add(i as usize) == '\\' as u16 {
                i += 1;
            }
            pos = i;
            while i < len && *buffer.add(i as usize) != '\\' as u16 {
                i += 1;
            }
        }
        str_.Buffer = buffer.add(pos as usize);
        str_.Length = ((i - pos) as usize * size_of::<WCHAR>()) as u16;
        (*attr).Attributes = attrs;
        status = NtCreateKey(retkey as *mut HANDLE, access, attr, 0, null_mut(), 0, null_mut());
        if (*attr).RootDirectory != root {
            NtClose((*attr).RootDirectory);
        }
    }
    status
}

/// create the special HKEY_CLASSES_ROOT key
unsafe fn create_classes_root_hkey(access: DWORD) -> HKEY {
    let mut hkey: HKEY = null_mut();
    let mut ret: HKEY = null_mut();
    let mut attr: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut name: UNICODE_STRING = core::mem::zeroed();
    let classes_rootW = wide!("\\REGISTRY\\Machine\\Software\\Classes");

    attr.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = null_mut();
    attr.ObjectName = &raw mut name;
    // C `#ifdef __REACTOS__`: our configuration manager matches key names
    // case-sensitively unless asked, so `CLSID\{abc...}` vs `{ABC...}` missed.
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.SecurityDescriptor = null_mut();
    attr.SecurityQualityOfService = null_mut();
    RtlInitUnicodeString(&raw mut name, classes_rootW.as_ptr());
    if create_key(&raw mut hkey, access, &raw mut attr) != 0 {
        return null_mut();
    }
    TRACE!();

    if (access & KEY_WOW64_64KEY) == 0 {
        ret = InterlockedCompareExchangePointer(
            &raw mut classes_root_hkey as *mut PVOID,
            hkey,
            null_mut(),
        ) as HKEY;
        if ret.is_null() {
            ret = hkey;
        } else {
            NtClose(hkey); // somebody beat us to it
        }
    } else {
        ret = hkey;
    }
    ret
}

/// map the hkey from special root to normal key if necessary
#[inline]
unsafe fn get_classes_root_hkey(hkey: HKEY, access: REGSAM) -> HKEY {
    let mut hkey = hkey;
    let mut ret: HKEY = hkey;
    let is_win64 = size_of::<*mut c_void>() > size_of::<i32>();
    let force_wow32 = is_win64 && (access & KEY_WOW64_32KEY) != 0;

    if hkey == HKEY_CLASSES_ROOT && {
        if (access & KEY_WOW64_64KEY) != 0 {
            true
        } else {
            ret = classes_root_hkey;
            ret.is_null()
        }
    } {
        ret = create_classes_root_hkey(MAXIMUM_ALLOWED | (access & KEY_WOW64_64KEY));
    }
    if force_wow32 && !ret.is_null() && ret == classes_root_hkey {
        let wow6432nodeW = wide!("Wow6432Node");
        let access = access & !KEY_WOW64_32KEY;
        let mut hkey2: HKEY = null_mut();
        if create_classes_key(classes_root_hkey, wow6432nodeW.as_ptr(), access, &raw mut hkey2)
            != 0
        {
            return null_mut();
        }
        hkey = hkey2;
        ret = hkey;
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn create_classes_key(
    hkey: HKEY,
    name: *const WCHAR,
    access: REGSAM,
    retkey: *mut HKEY,
) -> LSTATUS {
    let hkey = get_classes_root_hkey(hkey, access);
    if hkey.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    let mut attr: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut nameW: UNICODE_STRING = core::mem::zeroed();

    attr.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = hkey;
    attr.ObjectName = &raw mut nameW;
    // C `#ifdef __REACTOS__`: our configuration manager matches key names
    // case-sensitively unless asked, so `CLSID\{abc...}` vs `{ABC...}` missed.
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.SecurityDescriptor = null_mut();
    attr.SecurityQualityOfService = null_mut();
    RtlInitUnicodeString(&raw mut nameW, name);

    RtlNtStatusToDosError(create_key(retkey, access, &raw mut attr)) as LSTATUS
}

#[no_mangle]
pub unsafe extern "C" fn open_classes_key(
    hkey: HKEY,
    name: *const WCHAR,
    access: REGSAM,
    retkey: *mut HKEY,
) -> LSTATUS {
    let hkey = get_classes_root_hkey(hkey, access);
    if hkey.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    let mut attr: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut nameW: UNICODE_STRING = core::mem::zeroed();

    attr.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = hkey;
    attr.ObjectName = &raw mut nameW;
    // C `#ifdef __REACTOS__`: our configuration manager matches key names
    // case-sensitively unless asked, so `CLSID\{abc...}` vs `{ABC...}` missed.
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.SecurityDescriptor = null_mut();
    attr.SecurityQualityOfService = null_mut();
    RtlInitUnicodeString(&raw mut nameW, name);

    RtlNtStatusToDosError(NtOpenKey(retkey as *mut HANDLE, access, &raw mut attr)) as LSTATUS
}

// ---------------------------------------------------------------------------
// OpenDllList implementation
// ---------------------------------------------------------------------------

unsafe fn COMPOBJ_DllList_Get(library_name: LPCWSTR) -> *mut OpenDll {
    let mut ret: *mut OpenDll = null_mut();
    EnterCriticalSection(&raw mut csOpenDllList);
    let mut cursor = openDllList.next;
    while cursor != &raw mut openDllList {
        let ptr: *mut OpenDll = list_entry(cursor, offset_of!(OpenDll, entry));
        if wcsicmp(library_name, (*ptr).library_name) == 0
            && InterlockedIncrement(&raw mut (*ptr).refs) != 1
        /* entry is being destroyed if == 1 */
        {
            ret = ptr;
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(&raw mut csOpenDllList);
    ret
}

/// caller must ensure that library_name is not already in the open dll list
unsafe fn COMPOBJ_DllList_Add(library_name: LPCWSTR, ret: *mut *mut OpenDll) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    TRACE!();

    *ret = COMPOBJ_DllList_Get(library_name);
    if !(*ret).is_null() {
        return S_OK;
    }

    /* do this outside the csOpenDllList to avoid creating a lock dependency on
     * the loader lock */
    let hLibrary = LoadLibraryExW(library_name, null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH);
    if hLibrary.is_null() {
        ERR!("couldn't load in-process dll {}", DbgW(library_name));
        /* failure: DLL could not be loaded */
        return E_ACCESSDENIED; /* FIXME: or should this be CO_E_DLLNOTFOUND? */
    }

    let DllCanUnloadNow: DllCanUnloadNowFunc =
        core::mem::transmute(GetProcAddress(hLibrary, c"DllCanUnloadNow".as_ptr()));
    /* Note: failing to find DllCanUnloadNow is not a failure */
    let DllGetClassObject: DllGetClassObjectFunc =
        core::mem::transmute(GetProcAddress(hLibrary, c"DllGetClassObject".as_ptr()));
    if DllGetClassObject.is_none() {
        /* failure: the dll did not export DllGetClassObject */
        ERR!("couldn't find function DllGetClassObject in {}", DbgW(library_name));
        FreeLibrary(hLibrary);
        return CO_E_DLLNOTFOUND;
    }

    EnterCriticalSection(&raw mut csOpenDllList);

    *ret = COMPOBJ_DllList_Get(library_name);
    if !(*ret).is_null() {
        /* another caller to this function already added the dll while we
         * weren't in the critical section */
        FreeLibrary(hLibrary);
    } else {
        let len = lstrlenW(library_name);
        let entry = HeapAlloc(GetProcessHeap(), 0, size_of::<OpenDll>()) as *mut OpenDll;
        if !entry.is_null() {
            (*entry).library_name = HeapAlloc(
                GetProcessHeap(),
                0,
                (len as usize + 1) * size_of::<WCHAR>(),
            ) as LPWSTR;
        }
        if !entry.is_null() && !(*entry).library_name.is_null() {
            memcpy(
                (*entry).library_name as PVOID,
                library_name as *const c_void,
                (len as usize + 1) * size_of::<WCHAR>(),
            );
            (*entry).library = hLibrary;
            (*entry).refs = 1;
            (*entry).DllCanUnloadNow = DllCanUnloadNow;
            (*entry).DllGetClassObject = DllGetClassObject;
            list_add_tail(&raw mut openDllList, &raw mut (*entry).entry);
            *ret = entry;
        } else {
            HeapFree(GetProcessHeap(), 0, entry as PVOID);
            hr = E_OUTOFMEMORY;
            FreeLibrary(hLibrary);
        }
    }

    LeaveCriticalSection(&raw mut csOpenDllList);

    hr
}

/// pass FALSE for free_entry to release a reference without destroying the
/// entry if it reaches zero or TRUE otherwise
unsafe fn COMPOBJ_DllList_ReleaseRef(entry: *mut OpenDll, free_entry: BOOL) {
    if InterlockedDecrement(&raw mut (*entry).refs) == 0 && free_entry != FALSE {
        EnterCriticalSection(&raw mut csOpenDllList);
        list_remove(&raw mut (*entry).entry);
        LeaveCriticalSection(&raw mut csOpenDllList);

        TRACE!();
        FreeLibrary((*entry).library);

        HeapFree(GetProcessHeap(), 0, (*entry).library_name as PVOID);
        HeapFree(GetProcessHeap(), 0, entry as PVOID);
    }
}

/// frees memory associated with active dll list
unsafe fn COMPOBJ_DllList_Free() {
    EnterCriticalSection(&raw mut csOpenDllList);
    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = openDllList.next;
    while cursor != &raw mut openDllList {
        let next = (*cursor).next;
        let entry: *mut OpenDll = list_entry(cursor, offset_of!(OpenDll, entry));

        list_remove(&raw mut (*entry).entry);

        HeapFree(GetProcessHeap(), 0, (*entry).library_name as PVOID);
        HeapFree(GetProcessHeap(), 0, entry as PVOID);
        cursor = next;
    }
    LeaveCriticalSection(&raw mut csOpenDllList);
    DeleteCriticalSection(&raw mut csOpenDllList);
}

// ---------------------------------------------------------------------------
// Apartment management
// ---------------------------------------------------------------------------

unsafe fn apartment_addref(apt: *mut apartment) -> DWORD {
    let refs = InterlockedIncrement(&raw mut (*apt).refs) as DWORD;
    TRACE!();
    refs
}

/// allocates memory and fills in the necessary fields for a new apartment
/// object. must be called inside apartment cs
unsafe fn apartment_construct(model: DWORD) -> *mut APARTMENT {
    TRACE!();

    let apt =
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size_of::<APARTMENT>()) as *mut APARTMENT;
    // NOTE: like the C, no NULL check — OOM faults here.
    (*apt).tid = GetCurrentThreadId();

    list_init(&raw mut (*apt).proxies);
    list_init(&raw mut (*apt).stubmgrs);
    list_init(&raw mut (*apt).loaded_dlls);
    list_init(&raw mut (*apt).usage_cookies);
    (*apt).ipidc = 0;
    (*apt).refs = 1;
    (*apt).remunk_exported = FALSE;
    (*apt).oidc = 1;
    InitializeCriticalSection(&raw mut (*apt).cs);

    (*apt).multi_threaded = ((model & COINIT_APARTMENTTHREADED) == 0) as BOOL;

    if (*apt).multi_threaded != FALSE {
        /* FIXME: should be randomly generated by in an RPC call to rpcss */
        (*apt).oxid = ((GetCurrentProcessId() as OXID) << 32) | 0xcafe;
    } else {
        /* FIXME: should be randomly generated by in an RPC call to rpcss */
        (*apt).oxid = ((GetCurrentProcessId() as OXID) << 32) | GetCurrentThreadId() as OXID;
    }

    TRACE!();

    list_add_head(&raw mut apts, &raw mut (*apt).entry);

    apt
}

/// gets and existing apartment if one exists or otherwise creates an apartment
/// structure which stores OLE apartment-local information and stores a pointer
/// to it in the thread-local storage
unsafe fn apartment_get_or_create(model: DWORD) -> *mut APARTMENT {
    let mut apt = COM_CurrentApt();

    if apt.is_null() {
        if (model & COINIT_APARTMENTTHREADED) != 0 {
            EnterCriticalSection(&raw mut csApartment);

            apt = apartment_construct(model);
            if MainApartment.is_null() {
                MainApartment = apt;
                (*apt).main = TRUE;
                TRACE!();
            }

            // wine-10 (combase apartment_get_or_create)
            (*COM_CurrentInfo()).flags |= OLETLS_APARTMENTTHREADED;
            if (model & COINIT_DISABLE_OLE1DDE) != 0 {
                (*COM_CurrentInfo()).flags |= OLETLS_DISABLE_OLE1DDE;
            }

            LeaveCriticalSection(&raw mut csApartment);

            if (*apt).main != FALSE {
                apartment_createwindowifneeded(apt);
            }
        } else {
            EnterCriticalSection(&raw mut csApartment);

            /* The multi-threaded apartment (MTA) contains zero or more threads interacting
             * with free threaded (ie thread safe) COM objects. There is only ever one MTA
             * in a process */
            if !MTA.is_null() {
                TRACE!();
                apartment_addref(MTA);
            } else {
                MTA = apartment_construct(model);
            }

            (*COM_CurrentInfo()).flags |= OLETLS_MULTITHREADED | OLETLS_DISABLE_OLE1DDE;

            apt = MTA;

            LeaveCriticalSection(&raw mut csApartment);
        }
        (*COM_CurrentInfo()).apt = apt;
    }

    apt
}

#[inline]
unsafe fn apartment_is_model(apt: *const APARTMENT, model: DWORD) -> bool {
    (*apt).multi_threaded == ((model & COINIT_APARTMENTTHREADED) == 0) as BOOL
}

/// gets the multi-threaded apartment if it exists. The caller must
/// release the reference from the apartment as soon as the apartment pointer
/// is no longer required.
unsafe fn apartment_find_mta() -> *mut APARTMENT {
    EnterCriticalSection(&raw mut csApartment);

    let apt = MTA;
    if !apt.is_null() {
        apartment_addref(apt);
    }

    LeaveCriticalSection(&raw mut csApartment);

    apt
}

/// combase `apartment_get_mta`: the MTA with a reference, or NULL. Exported for the
/// marshal module's MTA ipid lookup.
#[no_mangle]
pub unsafe extern "C" fn apartment_get_mta() -> *mut APARTMENT {
    apartment_find_mta()
}

/// Return the current apartment if it exists, or, failing that, the MTA. Caller
/// must free the returned apartment in either case.
#[no_mangle]
pub unsafe extern "C" fn apartment_get_current_or_mta() -> *mut APARTMENT {
    let apt = COM_CurrentApt();
    if !apt.is_null() {
        apartment_addref(apt);
        return apt;
    }
    apartment_find_mta()
}

unsafe fn COM_RevokeRegisteredClassObject(curClass: *mut RegisteredClass) {
    list_remove(&raw mut (*curClass).entry);

    if ((*curClass).runContext & CLSCTX_LOCAL_SERVER) != 0 {
        RPC_StopLocalServer((*curClass).RpcRegistration);
    }

    IUnknown_Release((*curClass).classObject);
    HeapFree(GetProcessHeap(), 0, curClass as PVOID);
}

unsafe fn COM_RevokeAllClasses(apt: *const apartment) {
    EnterCriticalSection(&raw mut csRegisteredClassList);

    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = RegisteredClassList.next;
    while cursor != &raw mut RegisteredClassList {
        let next = (*cursor).next;
        let curClass: *mut RegisteredClass =
            list_entry(cursor, offset_of!(RegisteredClass, entry));
        if (*curClass).apartment_id == (*apt).oxid {
            COM_RevokeRegisteredClassObject(curClass);
        }
        cursor = next;
    }

    LeaveCriticalSection(&raw mut csRegisteredClassList);
}

unsafe fn revoke_registered_psclsids() {
    EnterCriticalSection(&raw mut cs_registered_psclsid_list);

    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = registered_psclsid_list.next;
    while cursor != &raw mut registered_psclsid_list {
        let next = (*cursor).next;
        let psclsid: *mut registered_psclsid =
            list_entry(cursor, offset_of!(registered_psclsid, entry));
        list_remove(&raw mut (*psclsid).entry);
        HeapFree(GetProcessHeap(), 0, psclsid as PVOID);
        cursor = next;
    }

    LeaveCriticalSection(&raw mut cs_registered_psclsid_list);
}

// ---------------------------------------------------------------------------
// Manual reset event object (CLSID_ManualResetEvent)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn impl_from_ISynchronize(iface: *mut ISynchronize) -> *mut MREImpl {
    (iface as *mut u8).sub(offset_of!(MREImpl, ISynchronize_iface)) as *mut MREImpl
}

unsafe extern "system" fn ISynchronize_fnQueryInterface(
    iface: *mut ISynchronize,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ISynchronize(iface);

    TRACE!();

    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_ISynchronize) {
        *ppv = &raw mut (*This).ISynchronize_iface as *mut c_void;
    } else if guid_eq(riid, &IID_ISynchronizeHandle) {
        *ppv = &raw mut (*This).ISynchronizeHandle_iface as *mut c_void;
    } else {
        ERR!("Unknown interface {} requested.", DbgGuid(riid));
        *ppv = null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    S_OK
}

unsafe extern "system" fn ISynchronize_fnAddRef(iface: *mut ISynchronize) -> ULONG {
    let This = impl_from_ISynchronize(iface);
    let ref_ = InterlockedIncrement(&raw mut (*This).ref_);
    TRACE!();
    ref_ as ULONG
}

unsafe extern "system" fn ISynchronize_fnRelease(iface: *mut ISynchronize) -> ULONG {
    let This = impl_from_ISynchronize(iface);
    let ref_ = InterlockedDecrement(&raw mut (*This).ref_);
    TRACE!();

    if ref_ == 0 {
        CloseHandle((*This).event);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }

    ref_ as ULONG
}

unsafe extern "system" fn ISynchronize_fnWait(
    iface: *mut ISynchronize,
    dwFlags: DWORD,
    dwMilliseconds: DWORD,
) -> HRESULT {
    let This = impl_from_ISynchronize(iface);
    let mut index: DWORD = 0;
    TRACE!();
    CoWaitForMultipleHandles(dwFlags, dwMilliseconds, 1, &raw mut (*This).event, &raw mut index)
}

unsafe extern "system" fn ISynchronize_fnSignal(iface: *mut ISynchronize) -> HRESULT {
    let This = impl_from_ISynchronize(iface);
    TRACE!();
    SetEvent((*This).event);
    S_OK
}

unsafe extern "system" fn ISynchronize_fnReset(iface: *mut ISynchronize) -> HRESULT {
    let This = impl_from_ISynchronize(iface);
    TRACE!();
    ResetEvent((*This).event);
    S_OK
}

static vt_ISynchronize: ISynchronizeVtbl = ISynchronizeVtbl {
    QueryInterface: ISynchronize_fnQueryInterface,
    AddRef: ISynchronize_fnAddRef,
    Release: ISynchronize_fnRelease,
    Wait: ISynchronize_fnWait,
    Signal: ISynchronize_fnSignal,
    Reset: ISynchronize_fnReset,
};

#[inline]
unsafe fn impl_from_ISynchronizeHandle(iface: *mut ISynchronizeHandle) -> *mut MREImpl {
    (iface as *mut u8).sub(offset_of!(MREImpl, ISynchronizeHandle_iface)) as *mut MREImpl
}

unsafe extern "system" fn SynchronizeHandle_QueryInterface(
    iface: *mut ISynchronizeHandle,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ISynchronizeHandle(iface);
    let sync = &raw mut (*This).ISynchronize_iface;
    ((*(*sync).lpVtbl).QueryInterface)(sync, riid, ppv)
}

unsafe extern "system" fn SynchronizeHandle_AddRef(iface: *mut ISynchronizeHandle) -> ULONG {
    let This = impl_from_ISynchronizeHandle(iface);
    let sync = &raw mut (*This).ISynchronize_iface;
    ((*(*sync).lpVtbl).AddRef)(sync)
}

unsafe extern "system" fn SynchronizeHandle_Release(iface: *mut ISynchronizeHandle) -> ULONG {
    let This = impl_from_ISynchronizeHandle(iface);
    let sync = &raw mut (*This).ISynchronize_iface;
    ((*(*sync).lpVtbl).Release)(sync)
}

unsafe extern "system" fn SynchronizeHandle_GetHandle(
    iface: *mut ISynchronizeHandle,
    ph: *mut HANDLE,
) -> HRESULT {
    let This = impl_from_ISynchronizeHandle(iface);

    *ph = (*This).event;
    S_OK
}

static SynchronizeHandleVtbl: ISynchronizeHandleVtbl = ISynchronizeHandleVtbl {
    QueryInterface: SynchronizeHandle_QueryInterface,
    AddRef: SynchronizeHandle_AddRef,
    Release: SynchronizeHandle_Release,
    GetHandle: SynchronizeHandle_GetHandle,
};

unsafe fn ManualResetEvent_Construct(
    punkouter: *mut IUnknown,
    iid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size_of::<MREImpl>()) as *mut MREImpl;

    if !punkouter.is_null() {
        FIXME!("Aggregation not implemented.");
    }

    // NOTE: like the C, no NULL check on the allocation.
    (*This).ref_ = 1;
    (*This).ISynchronize_iface.lpVtbl = &vt_ISynchronize;
    (*This).ISynchronizeHandle_iface.lpVtbl = &SynchronizeHandleVtbl;
    (*This).event = CreateEventW(null_mut(), TRUE, FALSE, null());

    let sync = &raw mut (*This).ISynchronize_iface;
    let hr = ((*(*sync).lpVtbl).QueryInterface)(sync, iid, ppv);
    ((*(*sync).lpVtbl).Release)(sync);
    hr
}

// ---------------------------------------------------------------------------
// LocalServer object (marshallable IServiceProvider over registered classes)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn impl_from_IServiceProvider(iface: *mut IServiceProvider) -> *mut LocalServer {
    (iface as *mut u8).sub(offset_of!(LocalServer, IServiceProvider_iface)) as *mut LocalServer
}

unsafe extern "system" fn LocalServer_QueryInterface(
    iface: *mut IServiceProvider,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IServiceProvider(iface);

    TRACE!();

    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IServiceProvider) {
        *ppv = &raw mut (*This).IServiceProvider_iface as *mut c_void;
    } else {
        *ppv = null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    S_OK
}

unsafe extern "system" fn LocalServer_AddRef(iface: *mut IServiceProvider) -> ULONG {
    let This = impl_from_IServiceProvider(iface);
    let ref_ = InterlockedIncrement(&raw mut (*This).ref_);

    TRACE!();

    ref_ as ULONG
}

unsafe extern "system" fn LocalServer_Release(iface: *mut IServiceProvider) -> ULONG {
    let This = impl_from_IServiceProvider(iface);
    let ref_ = InterlockedDecrement(&raw mut (*This).ref_);

    TRACE!();

    if ref_ == 0 {
        c_assert!((*This).apt.is_null(), "!This->apt");
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }

    ref_ as ULONG
}

unsafe extern "system" fn LocalServer_QueryService(
    iface: *mut IServiceProvider,
    guid: REFGUID,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IServiceProvider(iface);
    let apt = COM_CurrentApt();
    let mut hres: HRESULT = E_FAIL;

    TRACE!();

    if (*This).apt.is_null() {
        return E_UNEXPECTED;
    }

    EnterCriticalSection(&raw mut csRegisteredClassList);

    let mut cursor = RegisteredClassList.next;
    while cursor != &raw mut RegisteredClassList {
        let iter: *mut RegisteredClass = list_entry(cursor, offset_of!(RegisteredClass, entry));
        if (*iter).apartment_id == (*apt).oxid
            && ((*iter).runContext & CLSCTX_LOCAL_SERVER) != 0
            && guid_eq(&(*iter).classIdentifier, guid)
        {
            hres = IUnknown_QueryInterface((*iter).classObject, riid, ppv);
            break;
        }
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(&raw mut csRegisteredClassList);

    hres
}

static LocalServerVtbl: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: LocalServer_QueryInterface,
    AddRef: LocalServer_AddRef,
    Release: LocalServer_Release,
    QueryService: LocalServer_QueryService,
};

unsafe fn get_local_server_stream(apt: *mut APARTMENT, ret: *mut *mut IStream) -> HRESULT {
    let mut hres: HRESULT = S_OK;

    EnterCriticalSection(&raw mut (*apt).cs);

    if (*apt).local_server.is_null() {
        let obj = heap_alloc(size_of::<LocalServer>()) as *mut LocalServer;
        if !obj.is_null() {
            (*obj).IServiceProvider_iface.lpVtbl = &LocalServerVtbl;
            (*obj).ref_ = 1;
            (*obj).apt = apt;

            hres = CreateStreamOnHGlobal(null_mut(), TRUE, &raw mut (*obj).marshal_stream);
            if SUCCEEDED(hres) {
                hres = CoMarshalInterface(
                    (*obj).marshal_stream,
                    &IID_IServiceProvider,
                    &raw mut (*obj).IServiceProvider_iface as *mut IUnknown,
                    MSHCTX_LOCAL,
                    null_mut(),
                    MSHLFLAGS_TABLESTRONG,
                );
                if FAILED(hres) {
                    let s = (*obj).marshal_stream;
                    ((*(*s).lpVtbl).Release)(s);
                }
            }

            if SUCCEEDED(hres) {
                (*apt).local_server = obj;
            } else {
                heap_free(obj as PVOID);
            }
        } else {
            hres = E_OUTOFMEMORY;
        }
    }

    if SUCCEEDED(hres) {
        let s = (*(*apt).local_server).marshal_stream;
        hres = ((*(*s).lpVtbl).Clone)(s, ret);
    }

    LeaveCriticalSection(&raw mut (*apt).cs);

    if FAILED(hres) {
        ERR!("Failed: {:08x}", hres as u32);
    }
    hres
}

/***********************************************************************
 *           CoRevokeClassObject [OLE32.@]
 *
 * Removes a class object from the class registry.
 *
 * Must be called from the same apartment that called CoRegisterClassObject(),
 * otherwise it will fail with RPC_E_WRONG_THREAD.
 */
#[no_mangle]
pub unsafe extern "system" fn CoRevokeClassObject(dwRegister: DWORD) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;

    TRACE!();

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("COM was not initialized");
        return CO_E_NOTINITIALIZED;
    }

    EnterCriticalSection(&raw mut csRegisteredClassList);

    let mut cursor = RegisteredClassList.next;
    while cursor != &raw mut RegisteredClassList {
        let curClass: *mut RegisteredClass = list_entry(cursor, offset_of!(RegisteredClass, entry));
        /*
         * Check if we have a match on the cookie.
         */
        if (*curClass).dwCookie == dwRegister {
            if (*curClass).apartment_id == (*apt).oxid {
                COM_RevokeRegisteredClassObject(curClass);
                hr = S_OK;
            } else {
                ERR!("called from wrong apartment, should be called from {:x}",
                    (*curClass).apartment_id);
                hr = RPC_E_WRONG_THREAD;
            }
            break;
        }
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(&raw mut csRegisteredClassList);
    apartment_release(apt);
    hr
}

/// frees unused libraries loaded by apartment_getclassobject by calling the
/// DLL's DllCanUnloadNow entry point
unsafe fn apartment_freeunusedlibraries(apt: *mut apartment, delay: DWORD) {
    EnterCriticalSection(&raw mut (*apt).cs);
    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = (*apt).loaded_dlls.next;
    while cursor != &raw mut (*apt).loaded_dlls {
        let next = (*cursor).next;
        let entry: *mut apartment_loaded_dll =
            list_entry(cursor, offset_of!(apartment_loaded_dll, entry));

        let can_unload = match (*(*entry).dll).DllCanUnloadNow {
            Some(f) => f() == S_OK,
            None => false,
        };
        if can_unload {
            let mut real_delay: DWORD = delay;

            if real_delay == INFINITE {
                /* DLLs that return multi-threaded objects aren't unloaded
                 * straight away to cope for programs that have races between
                 * last object destruction and threads in the DLLs that haven't
                 * finished, despite DllCanUnloadNow returning S_OK */
                if (*entry).multi_threaded != FALSE {
                    real_delay = 10 * 60 * 1000; /* 10 minutes */
                } else {
                    real_delay = 0;
                }
            }

            if real_delay == 0
                || ((*entry).unload_time != 0
                    && (GetTickCount().wrapping_sub((*entry).unload_time) as i32) > 0)
            {
                list_remove(&raw mut (*entry).entry);
                COMPOBJ_DllList_ReleaseRef((*entry).dll, TRUE);
                HeapFree(GetProcessHeap(), 0, entry as PVOID);
            } else {
                (*entry).unload_time = GetTickCount().wrapping_add(real_delay);
                if (*entry).unload_time == 0 {
                    (*entry).unload_time = 1;
                }
            }
        } else if (*entry).unload_time != 0 {
            (*entry).unload_time = 0;
        }
        cursor = next;
    }
    LeaveCriticalSection(&raw mut (*apt).cs);
}

#[no_mangle]
pub unsafe extern "C" fn apartment_release(apt: *mut apartment) -> DWORD {
    EnterCriticalSection(&raw mut csApartment);

    let ret = InterlockedDecrement(&raw mut (*apt).refs) as DWORD;
    TRACE!();

    if (*apt).being_destroyed != FALSE {
        LeaveCriticalSection(&raw mut csApartment);
        return ret;
    }

    /* destruction stuff that needs to happen under csApartment CS */
    if ret == 0 {
        (*apt).being_destroyed = TRUE;
        if apt == MTA {
            MTA = null_mut();
        } else if apt == MainApartment {
            MainApartment = null_mut();
        }
        list_remove(&raw mut (*apt).entry);
    }

    LeaveCriticalSection(&raw mut csApartment);

    if ret == 0 {
        TRACE!();

        if !(*apt).local_server.is_null() {
            let local_server: *mut LocalServer = (*apt).local_server;
            let zero: i64 = 0;

            let s = (*local_server).marshal_stream;
            ((*(*s).lpVtbl).Seek)(s, zero, STREAM_SEEK_SET, null_mut());
            CoReleaseMarshalData(s);
            ((*(*s).lpVtbl).Release)(s);
            (*local_server).marshal_stream = null_mut();

            (*apt).local_server = null_mut();
            (*local_server).apt = null_mut();
            let sp = &raw mut (*local_server).IServiceProvider_iface;
            ((*(*sp).lpVtbl).Release)(sp);
        }

        /* Release the references to the registered class objects */
        COM_RevokeAllClasses(apt);

        /* no locking is needed for this apartment, because no other thread
         * can access it at this point */

        apartment_disconnectproxies(apt);

        if !(*apt).win.is_null() {
            DestroyWindow((*apt).win);
        }
        if (*apt).host_apt_tid != 0 {
            PostThreadMessageW((*apt).host_apt_tid, WM_QUIT, 0, 0);
        }

        // LIST_FOR_EACH_SAFE over stubmgrs
        let mut cursor = (*apt).stubmgrs.next;
        while cursor != &raw mut (*apt).stubmgrs {
            let cursor2 = (*cursor).next;
            // struct stub_manager: `entry` is the FIRST field (compobj_private.h),
            // so LIST_ENTRY(cursor, struct stub_manager, entry) == cursor.
            let stubmgr = cursor as *mut stub_manager;
            /* release the implicit reference given by the fact that the
             * stub has external references (it must do since it is in the
             * stub manager list in the apartment and all non-apartment users
             * must have a ref on the apartment and so it cannot be destroyed).
             */
            stub_manager_int_release(stubmgr);
            cursor = cursor2;
        }

        /* if this assert fires, then another thread took a reference to a
         * stub manager without taking a reference to the containing
         * apartment, which it must do. */
        c_assert!(list_empty(&raw const (*apt).stubmgrs), "list_empty(&apt->stubmgrs)");

        if !(*apt).filter.is_null() {
            let f = (*apt).filter;
            ((*(*f).lpVtbl).Release)(f);
        }

        /* free as many unused libraries as possible... */
        apartment_freeunusedlibraries(apt, 0);

        /* ... and free the memory for the apartment loaded dll entry and
         * release the dll list reference without freeing the library for the
         * rest */
        loop {
            let cursor = list_head(&raw const (*apt).loaded_dlls);
            if cursor.is_null() {
                break;
            }
            let aldll: *mut apartment_loaded_dll =
                list_entry(cursor, offset_of!(apartment_loaded_dll, entry));
            COMPOBJ_DllList_ReleaseRef((*aldll).dll, FALSE);
            list_remove(cursor);
            HeapFree(GetProcessHeap(), 0, aldll as PVOID);
        }

        DeleteCriticalSection(&raw mut (*apt).cs);

        HeapFree(GetProcessHeap(), 0, apt as PVOID);
    }

    ret
}

/* The given OXID must be local to this process:
 *
 * The ref parameter is here mostly to ensure people remember that
 * they get one, you should normally take a ref for thread safety.
 */
#[no_mangle]
pub unsafe extern "C" fn apartment_findfromoxid(oxid: OXID, ref_: BOOL) -> *mut APARTMENT {
    let mut result: *mut APARTMENT = null_mut();

    EnterCriticalSection(&raw mut csApartment);
    let mut cursor = apts.next;
    while cursor != &raw mut apts {
        let apt: *mut apartment = list_entry(cursor, offset_of!(apartment, entry));
        if (*apt).oxid == oxid {
            result = apt;
            if ref_ != FALSE {
                apartment_addref(result);
            }
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(&raw mut csApartment);

    result
}

/// gets the apartment which has a given creator thread ID. The caller must
/// release the reference from the apartment as soon as the apartment pointer
/// is no longer required.
#[no_mangle]
pub unsafe extern "C" fn apartment_findfromtid(tid: DWORD) -> *mut APARTMENT {
    let mut result: *mut APARTMENT = null_mut();

    EnterCriticalSection(&raw mut csApartment);
    let mut cursor = apts.next;
    while cursor != &raw mut apts {
        let apt: *mut apartment = list_entry(cursor, offset_of!(apartment, entry));
        if (*apt).tid == tid {
            result = apt;
            apartment_addref(result);
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(&raw mut csApartment);

    result
}

/// gets the main apartment if it exists. The caller must release the
/// reference from the apartment as soon as the pointer is no longer required.
unsafe fn apartment_findmain() -> *mut APARTMENT {
    EnterCriticalSection(&raw mut csApartment);

    let result = MainApartment;
    if !result.is_null() {
        apartment_addref(result);
    }

    LeaveCriticalSection(&raw mut csApartment);

    result
}

/// gets the specified class object by loading the appropriate DLL, if
/// necessary and calls the DllGetClassObject function for the DLL
unsafe fn apartment_getclassobject(
    apt: *mut apartment,
    dllpath: LPCWSTR,
    apartment_threaded: BOOL,
    rclsid: REFCLSID,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let wszOle32 = wide!("ole32.dll");
    let mut hr: HRESULT = S_OK;
    let mut found = FALSE;
    let mut aldll: *mut apartment_loaded_dll = null_mut();

    if wcsicmp(dllpath, wszOle32.as_ptr()) == 0 {
        /* we don't need to control the lifetime of this dll, so use the local
         * implementation of DllGetClassObject directly */
        TRACE!();
        hr = DllGetClassObject(rclsid, riid, ppv);

        if hr != S_OK {
            ERR!("DllGetClassObject returned error {:#010x} for dll {}", hr as u32, DbgW(dllpath));
        }

        return hr;
    }

    EnterCriticalSection(&raw mut (*apt).cs);

    let mut cursor = (*apt).loaded_dlls.next;
    while cursor != &raw mut (*apt).loaded_dlls {
        aldll = list_entry(cursor, offset_of!(apartment_loaded_dll, entry));
        if wcsicmp(dllpath, (*(*aldll).dll).library_name) == 0 {
            TRACE!();
            found = TRUE;
            break;
        }
        cursor = (*cursor).next;
    }

    if found == FALSE {
        aldll = HeapAlloc(GetProcessHeap(), 0, size_of::<apartment_loaded_dll>())
            as *mut apartment_loaded_dll;
        if aldll.is_null() {
            hr = E_OUTOFMEMORY;
        }
        if SUCCEEDED(hr) {
            (*aldll).unload_time = 0;
            (*aldll).multi_threaded = FALSE;
            hr = COMPOBJ_DllList_Add(dllpath, &raw mut (*aldll).dll);
            if FAILED(hr) {
                HeapFree(GetProcessHeap(), 0, aldll as PVOID);
            }
        }
        if SUCCEEDED(hr) {
            TRACE!();
            list_add_tail(&raw mut (*apt).loaded_dlls, &raw mut (*aldll).entry);
        }
    }

    LeaveCriticalSection(&raw mut (*apt).cs);

    if SUCCEEDED(hr) {
        /* one component being multi-threaded overrides any number of
         * apartment-threaded components */
        if apartment_threaded == FALSE {
            (*aldll).multi_threaded = TRUE;
        }

        TRACE!();
        /* OK: get the ClassObject */
        hr = match (*(*aldll).dll).DllGetClassObject {
            Some(f) => f(rclsid, riid, ppv),
            None => {
                // unreachable: COMPOBJ_DllList_Add rejects DLLs without the export
                ERR!("DllGetClassObject vanished from {}", DbgW(dllpath));
                CO_E_DLLNOTFOUND
            }
        };

        if hr != S_OK {
            ERR!("DllGetClassObject returned error {:#010x} for dll {}", hr as u32, DbgW(dllpath));
        }
    }

    hr
}

/// Returns expanded dll path from the registry or activation context.
unsafe fn get_object_dll_path(
    regdata: *const class_reg_data,
    dst: *mut WCHAR,
    dstlen: DWORD,
) -> BOOL {
    let mut ret: DWORD;

    if (*regdata).hkey != FALSE {
        let mut keytype: DWORD = 0;
        let mut src: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        let mut dwLength: DWORD = dstlen * size_of::<WCHAR>() as DWORD;

        ret = RegQueryValueExW(
            (*regdata).u.hkey,
            null(),
            null_mut(),
            &raw mut keytype,
            src.as_mut_ptr() as *mut BYTE,
            &raw mut dwLength,
        ) as DWORD;
        if ret == ERROR_SUCCESS as DWORD {
            if keytype == REG_EXPAND_SZ {
                if dstlen <= ExpandEnvironmentStringsW(src.as_ptr(), dst, dstlen) {
                    ret = ERROR_MORE_DATA as DWORD;
                }
            } else {
                let quote_start = wcschr(src.as_ptr(), '"' as u16);
                if !quote_start.is_null() {
                    let quote_end = wcschr(quote_start.add(1), '"' as u16);
                    if !quote_end.is_null() {
                        let n = quote_end.offset_from(quote_start) as usize - 1;
                        memmove(
                            src.as_mut_ptr() as PVOID,
                            quote_start.add(1) as *const c_void,
                            n * size_of::<WCHAR>(),
                        );
                        src[n] = 0;
                    }
                }
                lstrcpynW(dst, src.as_ptr(), dstlen as INT);
            }
        }
        (ret == 0) as BOOL
    } else {
        let dllW = wide!(".dll");
        let mut cookie: ULONG_PTR = 0;

        *dst = 0;
        let nameW = ((*regdata).u.actctx.section as *mut u8)
            .add((*(*regdata).u.actctx.data).name_offset as usize) as *mut WCHAR;
        ActivateActCtx((*regdata).u.actctx.hactctx, &raw mut cookie);
        ret = SearchPathW(null(), nameW, dllW.as_ptr(), dstlen, dst, null_mut());
        DeactivateActCtx(0, cookie);
        (*dst != 0) as BOOL
    }
}

unsafe fn apartment_hostobject(
    apt: *mut apartment,
    params: *const host_object_params,
) -> HRESULT {
    let mut object: *mut IUnknown = null_mut();
    let llZero: i64 = 0;
    let mut dllpath: [WCHAR; MAX_PATH + 1] = [0; MAX_PATH + 1];

    TRACE!();

    if get_object_dll_path(&(*params).regdata, dllpath.as_mut_ptr(), (MAX_PATH + 1) as DWORD)
        == FALSE
    {
        /* failure: CLSID is not found in registry */
        WARN!();
        return REGDB_E_CLASSNOTREG;
    }

    let mut hr = apartment_getclassobject(
        apt,
        dllpath.as_ptr(),
        (*params).apartment_threaded,
        &(*params).clsid,
        &(*params).iid,
        &raw mut object as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = CoMarshalInterface(
        (*params).stream,
        &(*params).iid,
        object,
        MSHCTX_INPROC,
        null_mut(),
        MSHLFLAGS_NORMAL,
    );
    if FAILED(hr) {
        IUnknown_Release(object);
    }
    let s = (*params).stream;
    ((*(*s).lpVtbl).Seek)(s, llZero, STREAM_SEEK_SET, null_mut());

    hr
}

unsafe extern "system" fn apartment_wndproc(
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match msg {
        DM_EXECUTERPC => {
            RPC_ExecuteCall(lParam as *mut dispatch_params);
            0
        }
        DM_HOSTOBJECT => {
            apartment_hostobject(COM_CurrentApt(), lParam as *const host_object_params) as LRESULT
        }
        _ => DefWindowProcW(hWnd, msg, wParam, lParam),
    }
}

/// thread for hosting an object to allow an object to appear to be created in
/// an apartment with an incompatible threading model
unsafe extern "system" fn apartment_hostobject_thread(p: PVOID) -> DWORD {
    let params = p as *mut host_thread_params;
    let mut msg: MSG = core::mem::zeroed();

    TRACE!();

    let hr = CoInitializeEx(null_mut(), (*params).threading_model);
    if FAILED(hr) {
        return hr as DWORD;
    }

    let apt = COM_CurrentApt();
    if (*params).threading_model == COINIT_APARTMENTTHREADED {
        apartment_createwindowifneeded(apt);
        (*params).apartment_hwnd = apartment_getwindow(apt);
    } else {
        (*params).apartment_hwnd = null_mut();
    }

    /* force the message queue to be created before signaling parent thread */
    PeekMessageW(&raw mut msg, null_mut(), WM_USER, WM_USER, PM_NOREMOVE);

    SetEvent((*params).ready_event);
    /* can't touch params after here as it may be invalid */
    let params: *mut host_thread_params = null_mut();
    let _ = params;

    while GetMessageW(&raw mut msg, null_mut(), 0, 0) != FALSE {
        if msg.hwnd.is_null() && msg.message == DM_HOSTOBJECT {
            let obj_params = msg.lParam as *mut host_object_params;
            (*obj_params).hr = apartment_hostobject(apt, obj_params);
            SetEvent((*obj_params).event);
        } else {
            TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
    }

    TRACE!();

    CoUninitialize();

    S_OK as DWORD
}

/// finds or creates a host apartment, creates the object inside it and returns
/// a proxy to it so that the object can be used in the apartment of the
/// caller of this function
unsafe fn apartment_hostobject_in_hostapt(
    apt: *mut apartment,
    multi_threaded: BOOL,
    main_apartment: BOOL,
    regdata: *const class_reg_data,
    rclsid: REFCLSID,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut params: host_object_params = core::mem::zeroed();
    let mut apartment_hwnd: HWND = null_mut();
    let mut apartment_tid: DWORD = 0;
    let mut hr: HRESULT;

    if multi_threaded == FALSE && main_apartment != FALSE {
        let host_apt = apartment_findmain();
        if !host_apt.is_null() {
            apartment_hwnd = apartment_getwindow(host_apt);
            apartment_release(host_apt);
        }
    }

    if apartment_hwnd.is_null() {
        EnterCriticalSection(&raw mut (*apt).cs);

        if (*apt).host_apt_tid == 0 {
            let mut thread_params: host_thread_params = core::mem::zeroed();
            let mut handles: [HANDLE; 2] = [null_mut(); 2];

            thread_params.threading_model = if multi_threaded != FALSE {
                COINIT_MULTITHREADED
            } else {
                COINIT_APARTMENTTHREADED
            };
            thread_params.ready_event = CreateEventW(null_mut(), FALSE, FALSE, null());
            handles[0] = thread_params.ready_event;
            thread_params.apartment_hwnd = null_mut();
            handles[1] = CreateThread(
                null_mut(),
                0,
                apartment_hostobject_thread,
                &raw mut thread_params as PVOID,
                0,
                &raw mut (*apt).host_apt_tid,
            );
            if handles[1].is_null() {
                CloseHandle(handles[0]);
                LeaveCriticalSection(&raw mut (*apt).cs);
                return E_OUTOFMEMORY;
            }
            let wait_value = WaitForMultipleObjects(2, handles.as_ptr(), FALSE, INFINITE);
            CloseHandle(handles[0]);
            CloseHandle(handles[1]);
            if wait_value == WAIT_OBJECT_0 {
                (*apt).host_apt_hwnd = thread_params.apartment_hwnd;
            } else {
                LeaveCriticalSection(&raw mut (*apt).cs);
                return E_OUTOFMEMORY;
            }
        }

        if multi_threaded != FALSE || main_apartment == FALSE {
            apartment_hwnd = (*apt).host_apt_hwnd;
            apartment_tid = (*apt).host_apt_tid;
        }

        LeaveCriticalSection(&raw mut (*apt).cs);
    }

    /* another thread may have become the main apartment in the time it took
     * us to create the thread for the host apartment */
    if apartment_hwnd.is_null() && multi_threaded == FALSE && main_apartment != FALSE {
        let host_apt = apartment_findmain();
        if !host_apt.is_null() {
            apartment_hwnd = apartment_getwindow(host_apt);
            apartment_release(host_apt);
        }
    }

    params.regdata = core::ptr::read(regdata);
    params.clsid = *rclsid;
    params.iid = *riid;
    hr = CreateStreamOnHGlobal(null_mut(), TRUE, &raw mut params.stream);
    if FAILED(hr) {
        return hr;
    }
    params.apartment_threaded = (multi_threaded == FALSE) as BOOL;
    if multi_threaded != FALSE {
        params.hr = S_OK;
        params.event = CreateEventW(null_mut(), FALSE, FALSE, null());
        if PostThreadMessageW(apartment_tid, DM_HOSTOBJECT, 0, &raw mut params as LPARAM) == FALSE
        {
            hr = E_OUTOFMEMORY;
        } else {
            WaitForSingleObject(params.event, INFINITE);
            hr = params.hr;
        }
        CloseHandle(params.event);
    } else {
        if apartment_hwnd.is_null() {
            ERR!("host apartment didn't create window");
            hr = E_OUTOFMEMORY;
        } else {
            hr = SendMessageW(apartment_hwnd, DM_HOSTOBJECT, 0, &raw mut params as LPARAM)
                as HRESULT;
        }
    }
    if SUCCEEDED(hr) {
        hr = CoUnmarshalInterface(params.stream, riid, ppv);
    }
    let s = params.stream;
    ((*(*s).lpVtbl).Release)(s);
    hr
}

static wszAptWinClass: &[u16] = wide!("OleMainThreadWndClass");

unsafe extern "system" fn register_class(
    once: *mut INIT_ONCE,
    param: PVOID,
    context: *mut PVOID,
) -> BOOL {
    let mut wclass: WNDCLASSW = core::mem::zeroed();

    /* Dispatching to the correct thread in an apartment is done through
     * window messages rather than RPC transports. When an interface is
     * marshalled into another apartment in the same process, a window of the
     * following class is created. The *caller* of CoMarshalInterface (i.e., the
     * application) is responsible for pumping the message loop in that thread.
     * The WM_USER messages which point to the RPCs are then dispatched to
     * apartment_wndproc by the user's code from the apartment in which the
     * interface was unmarshalled.
     */
    wclass.lpfnWndProc = Some(apartment_wndproc);
    wclass.hInstance = hProxyDll;
    wclass.lpszClassName = wszAptWinClass.as_ptr();
    apt_win_class = RegisterClassW(&raw const wclass);
    TRUE
}

/// create a window for the apartment or return the current one if one has
/// already been created
#[no_mangle]
pub unsafe extern "C" fn apartment_createwindowifneeded(apt: *mut apartment) -> HRESULT {
    if (*apt).multi_threaded != FALSE {
        return S_OK;
    }

    if (*apt).win.is_null() {
        InitOnceExecuteOnce(&raw mut class_init_once, Some(register_class), null_mut(), null_mut());

        let hwnd = CreateWindowExW(
            0,
            wszAptWinClass.as_ptr(),
            null(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            hProxyDll,
            null_mut(),
        );
        if hwnd.is_null() {
            ERR!("CreateWindow failed with error {}", GetLastError());
            return HRESULT_FROM_WIN32(GetLastError());
        }
        if !InterlockedCompareExchangePointer(
            &raw mut (*apt).win as *mut PVOID,
            hwnd,
            null_mut(),
        )
        .is_null()
        {
            /* someone beat us to it */
            DestroyWindow(hwnd);
        }
    }

    S_OK
}

/// retrieves the window for the main- or apartment-threaded apartment
#[no_mangle]
pub unsafe extern "C" fn apartment_getwindow(apt: *const apartment) -> HWND {
    c_assert!((*apt).multi_threaded == FALSE, "!apt->multi_threaded");
    (*apt).win
}

unsafe fn COM_TlsDestroy() {
    let slot = teb_reserved_for_ole();
    let info = *slot as *mut oletls;
    if !info.is_null() {
        if !(*info).apt.is_null() {
            apartment_release((*info).apt);
        }
        if !(*info).errorinfo.is_null() {
            IUnknown_Release((*info).errorinfo);
        }
        if !(*info).state.is_null() {
            IUnknown_Release((*info).state);
        }

        // LIST_FOR_EACH_ENTRY_SAFE over spies
        let mut cursor = (*info).spies.next;
        while cursor != &raw mut (*info).spies {
            let cursor2 = (*cursor).next;
            let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
            list_remove(&raw mut (*spy).entry);
            if !(*spy).spy.is_null() {
                let s = (*spy).spy;
                ((*(*s).lpVtbl).Release)(s);
            }
            heap_free(spy as PVOID);
            cursor = cursor2;
        }

        if !(*info).context_token.is_null() {
            let ct = (*info).context_token;
            ((*(*ct).lpVtbl).Release)(ct);
        }

        HeapFree(GetProcessHeap(), 0, info as PVOID);
        *slot = null_mut();
    }
}

/******************************************************************************
 *           CoBuildVersion [OLE32.@]
 *
 * RETURNS: Current build version, hiword is majornumber, loword is minornumber
 */
#[no_mangle]
pub unsafe extern "system" fn CoBuildVersion() -> DWORD {
    TRACE!();
    (rmm << 16) + rup
}

unsafe fn get_spy_entry(info: *mut oletls, id: u32) -> *mut init_spy {
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if id == (*spy).id && !(*spy).spy.is_null() {
            return spy;
        }
        cursor = (*cursor).next;
    }

    null_mut()
}

/*
 * When locked, don't modify list (unless we add a new head), so that it's
 * safe to iterate it. Freeing of list entries is delayed and done on unlock.
 */
#[inline]
unsafe fn lock_init_spies(info: *mut oletls) {
    (*info).spies_lock += 1;
}

unsafe fn unlock_init_spies(info: *mut oletls) {
    (*info).spies_lock -= 1;
    if (*info).spies_lock != 0 {
        return;
    }

    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let next = (*cursor).next;
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if !(*spy).spy.is_null() {
            cursor = next;
            continue;
        }
        list_remove(&raw mut (*spy).entry);
        heap_free(spy as PVOID);
        cursor = next;
    }
}

/******************************************************************************
 *              CoRegisterInitializeSpy [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterInitializeSpy(
    spy: *mut IInitializeSpy,
    cookie: *mut ULARGE_INTEGER_S,
) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if spy.is_null() || cookie.is_null() || info.is_null() {
        if info.is_null() {
            WARN!();
        }
        return E_INVALIDARG;
    }

    let mut spy = spy;
    let hr = ((*(*spy).lpVtbl).QueryInterface)(
        spy,
        &IID_IInitializeSpy,
        &raw mut spy as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    let entry = heap_alloc(size_of::<init_spy>()) as *mut init_spy;
    if entry.is_null() {
        ((*(*spy).lpVtbl).Release)(spy);
        return E_OUTOFMEMORY;
    }

    (*entry).spy = spy;

    let mut id: u32 = 0;
    while !get_spy_entry(info, id).is_null() {
        id += 1;
    }

    (*entry).id = id;
    list_add_head(&raw mut (*info).spies, &raw mut (*entry).entry);

    (*cookie).HighPart = GetCurrentThreadId();
    (*cookie).LowPart = (*entry).id;

    S_OK
}

/******************************************************************************
 *              CoRevokeInitializeSpy [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoRevokeInitializeSpy(cookie: ULARGE_INTEGER_S) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if info.is_null() || cookie.HighPart != GetCurrentThreadId() {
        return E_INVALIDARG;
    }

    let spy = get_spy_entry(info, cookie.LowPart);
    if spy.is_null() {
        return E_INVALIDARG;
    }

    let s = (*spy).spy;
    ((*(*s).lpVtbl).Release)(s);
    (*spy).spy = null_mut();
    if (*info).spies_lock == 0 {
        list_remove(&raw mut (*spy).entry);
        heap_free(spy as PVOID);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn enter_apartment(info: *mut oletls, model: DWORD) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if (*info).apt.is_null() {
        if apartment_get_or_create(model).is_null() {
            return E_OUTOFMEMORY;
        }
    } else if !apartment_is_model((*info).apt, model) {
        WARN!();
        return RPC_E_CHANGED_MODE;
    } else {
        hr = S_FALSE;
    }

    (*info).inits += 1;

    hr
}

#[no_mangle]
pub unsafe extern "C" fn leave_apartment(info: *mut oletls) {
    (*info).inits -= 1;
    if (*info).inits == 0 {
        if (*info).ole_inits != 0 {
            WARN!();
        }
        apartment_release((*info).apt);
        (*info).apt = null_mut();
        (*info).flags &= !(OLETLS_DISABLE_OLE1DDE | OLETLS_APARTMENTTHREADED | OLETLS_MULTITHREADED);
    }
}

/******************************************************************************
 *		CoInitialize	[OLE32.@]
 *
 * Initializes the COM libraries by calling CoInitializeEx with
 * COINIT_APARTMENTTHREADED, ie it enters a STA thread.
 */
#[no_mangle]
pub unsafe extern "system" fn CoInitialize(lpReserved: LPVOID) -> HRESULT {
    /*
     * Just delegate to the newer method.
     */
    CoInitializeEx(lpReserved, COINIT_APARTMENTTHREADED)
}

/******************************************************************************
 *		CoInitializeEx	[OLE32.@]
 *
 * RETURNS
 *  S_OK               if successful,
 *  S_FALSE            if this function was called already.
 *  RPC_E_CHANGED_MODE if a previous call to CoInitializeEx specified another
 *                     threading model.
 */
#[no_mangle]
pub unsafe extern "system" fn CoInitializeEx(lpReserved: LPVOID, dwCoInit: DWORD) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if !lpReserved.is_null() {
        ERR!(
            "({:p}, {:x}) - Bad parameter passed-in {:p}, must be an old Windows Application",
            lpReserved, dwCoInit, lpReserved
        );
    }

    /*
     * Check the lock count. If this is the first time going through the initialize
     * process, we have to initialize the libraries.
     *
     * And crank-up that lock count.
     */
    if InterlockedExchangeAdd(&raw mut s_COMLockCount, 1) == 0 {
        /*
         * Initialize the various COM libraries and data structures.
         */
        TRACE!();

        /* we may need to defer this until after apartment initialisation */
        RunningObjectTableImpl_Initialize();
    }

    lock_init_spies(info);
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if !(*spy).spy.is_null() {
            let s = (*spy).spy;
            ((*(*s).lpVtbl).PreInitialize)(s, dwCoInit, (*info).inits);
        }
        cursor = (*cursor).next;
    }
    unlock_init_spies(info);

    let mut hr = enter_apartment(info, dwCoInit);

    lock_init_spies(info);
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if !(*spy).spy.is_null() {
            let s = (*spy).spy;
            hr = ((*(*s).lpVtbl).PostInitialize)(s, hr, dwCoInit, (*info).inits);
        }
        cursor = (*cursor).next;
    }
    unlock_init_spies(info);

    hr
}

/***********************************************************************
 *           CoUninitialize   [OLE32.@]
 *
 * This method will decrement the refcount on the current apartment, freeing
 * the resources associated with it if it is the last thread in the apartment.
 * If the last apartment is freed, the function will additionally release
 * any COM resources associated with the process.
 */
#[no_mangle]
pub unsafe extern "system" fn CoUninitialize() {
    let info = COM_CurrentInfo();

    TRACE!();

    /* will only happen on OOM */
    if info.is_null() {
        return;
    }

    lock_init_spies(info);
    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let next = (*cursor).next;
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if !(*spy).spy.is_null() {
            let s = (*spy).spy;
            ((*(*s).lpVtbl).PreUninitialize)(s, (*info).inits);
        }
        cursor = next;
    }
    unlock_init_spies(info);

    /* sanity check */
    if (*info).inits == 0 {
        ERR!("Mismatched CoUninitialize");

        lock_init_spies(info);
        let mut cursor = (*info).spies.next;
        while cursor != &raw mut (*info).spies {
            let next = (*cursor).next;
            let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
            if !(*spy).spy.is_null() {
                let s = (*spy).spy;
                ((*(*s).lpVtbl).PostUninitialize)(s, (*info).inits);
            }
            cursor = next;
        }
        unlock_init_spies(info);

        return;
    }

    leave_apartment(info);

    /*
     * Decrease the reference count.
     * If we are back to 0 locks on the COM library, make sure we free
     * all the associated data structures.
     */
    let lCOMRefCnt = InterlockedExchangeAdd(&raw mut s_COMLockCount, -1);
    if lCOMRefCnt == 1 {
        TRACE!();

        revoke_registered_psclsids();
        RunningObjectTableImpl_UnInitialize();
    } else if lCOMRefCnt < 1 {
        ERR!("CoUninitialize() - not CoInitialized.");
        InterlockedExchangeAdd(&raw mut s_COMLockCount, 1); /* restore the lock count. */
    }

    lock_init_spies(info);
    let mut cursor = (*info).spies.next;
    while cursor != &raw mut (*info).spies {
        let spy: *mut init_spy = list_entry(cursor, offset_of!(init_spy, entry));
        if !(*spy).spy.is_null() {
            let s = (*spy).spy;
            ((*(*s).lpVtbl).PostUninitialize)(s, (*info).inits);
        }
        cursor = (*cursor).next;
    }
    unlock_init_spies(info);
}

/******************************************************************************
 *		CoDisconnectObject	[OLE32.@]
 *
 * Disconnects all connections to this object from remote processes. Dispatches
 * pending RPCs while blocking new RPCs from occurring, and then calls
 * IMarshal::DisconnectObject on the given object.
 */
#[no_mangle]
pub unsafe extern "system" fn CoDisconnectObject(lpUnk: *mut IUnknown, reserved: DWORD) -> HRESULT {
    let mut marshal: *mut IMarshal = null_mut();

    TRACE!();

    if lpUnk.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = IUnknown_QueryInterface(
        lpUnk,
        &IID_IMarshal,
        &raw mut marshal as *mut *mut c_void,
    );
    if hr == S_OK {
        hr = ((*(*marshal).lpVtbl).DisconnectObject)(marshal, reserved);
        ((*(*marshal).lpVtbl).Release)(marshal);
        return hr;
    }

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }

    let manager = get_stub_manager_from_object(apt, lpUnk, FALSE);
    if !manager.is_null() {
        stub_manager_disconnect(manager);
        /* Release stub manager twice, to remove the apartment reference. */
        stub_manager_int_release(manager);
        stub_manager_int_release(manager);
    }

    /* Note: native is pretty broken here because it just silently
     * fails, without returning an appropriate error code if the object was
     * not found, making apps think that the object was disconnected, when
     * it actually wasn't */

    apartment_release(apt);
    S_OK
}

/******************************************************************************
 *		CoCreateGuid [OLE32.@]
 *
 * Simply forwards to UuidCreate in RPCRT4.
 */
#[no_mangle]
pub unsafe extern "system" fn CoCreateGuid(pguid: *mut GUID) -> HRESULT {
    if pguid.is_null() {
        return E_INVALIDARG;
    }

    let status = UuidCreate(pguid);
    if status == RPC_S_OK || status == RPC_S_UUID_LOCAL_ONLY {
        return S_OK;
    }
    HRESULT_FROM_WIN32(status as DWORD)
}

#[inline]
fn is_valid_hex(c: WCHAR) -> bool {
    ((c >= '0' as u16) && (c <= '9' as u16))
        || ((c >= 'a' as u16) && (c <= 'f' as u16))
        || ((c >= 'A' as u16) && (c <= 'F' as u16))
}

static guid_conv_table: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0usize;
    while i < 10 {
        t[0x30 + i] = i as u8; // '0'..'9'
        i += 1;
    }
    let mut i = 0usize;
    while i < 6 {
        t[0x41 + i] = 0xa + i as u8; // 'A'..'F'
        t[0x61 + i] = 0xa + i as u8; // 'a'..'f'
        i += 1;
    }
    t
};

/// conversion helper for CLSIDFromString/IIDFromString
unsafe fn guid_from_string(s: LPCWSTR, id: *mut GUID) -> BOOL {
    if s.is_null() || *s != '{' as u16 {
        memset(id as PVOID, 0, size_of::<CLSID>());
        if s.is_null() {
            return TRUE;
        }
        return FALSE;
    }

    TRACE!();

    /* in form {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX} */

    (*id).Data1 = 0;
    let mut i = 1usize;
    while i < 9 {
        if !is_valid_hex(*s.add(i)) {
            return FALSE;
        }
        (*id).Data1 = ((*id).Data1 << 4) | guid_conv_table[*s.add(i) as usize] as u32;
        i += 1;
    }
    if *s.add(9) != '-' as u16 {
        return FALSE;
    }

    (*id).Data2 = 0;
    let mut i = 10usize;
    while i < 14 {
        if !is_valid_hex(*s.add(i)) {
            return FALSE;
        }
        (*id).Data2 = ((*id).Data2 << 4) | guid_conv_table[*s.add(i) as usize] as u16;
        i += 1;
    }
    if *s.add(14) != '-' as u16 {
        return FALSE;
    }

    (*id).Data3 = 0;
    let mut i = 15usize;
    while i < 19 {
        if !is_valid_hex(*s.add(i)) {
            return FALSE;
        }
        (*id).Data3 = ((*id).Data3 << 4) | guid_conv_table[*s.add(i) as usize] as u16;
        i += 1;
    }
    if *s.add(19) != '-' as u16 {
        return FALSE;
    }

    let mut i = 20usize;
    while i < 37 {
        if i == 24 {
            if *s.add(i) != '-' as u16 {
                return FALSE;
            }
            i += 1;
        }
        if !is_valid_hex(*s.add(i)) || !is_valid_hex(*s.add(i + 1)) {
            return FALSE;
        }
        (*id).Data4[(i - 20) / 2] = (guid_conv_table[*s.add(i) as usize] << 4)
            | guid_conv_table[*s.add(i + 1) as usize];
        i += 2;
    }

    if *s.add(37) == '}' as u16 && *s.add(38) == 0 {
        return TRUE;
    }

    FALSE
}

/*****************************************************************************/

unsafe fn clsid_from_string_reg(progid: LPCOLESTR, clsid: *mut CLSID) -> HRESULT {
    let clsidW = wide!("\\CLSID");
    let mut buf2: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    let mut buf2len: LONG = size_of::<[WCHAR; CHARS_IN_GUID]>() as LONG;
    let mut xhkey: HKEY = null_mut();

    memset(clsid as PVOID, 0, size_of::<CLSID>());
    let buf = HeapAlloc(
        GetProcessHeap(),
        0,
        (lstrlenW(progid) as usize + 8) * size_of::<WCHAR>(),
    ) as LPWSTR;
    if buf.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpyW(buf, progid);
    lstrcatW(buf, clsidW.as_ptr());
    if open_classes_key(HKEY_CLASSES_ROOT, buf, MAXIMUM_ALLOWED, &raw mut xhkey) != 0 {
        HeapFree(GetProcessHeap(), 0, buf as PVOID);
        WARN!();
        return CO_E_CLASSSTRING;
    }
    HeapFree(GetProcessHeap(), 0, buf as PVOID);

    if RegQueryValueW(xhkey, null(), buf2.as_mut_ptr(), &raw mut buf2len) != 0 {
        RegCloseKey(xhkey);
        WARN!();
        return CO_E_CLASSSTRING;
    }
    RegCloseKey(xhkey);
    if guid_from_string(buf2.as_ptr(), clsid) != FALSE {
        S_OK
    } else {
        CO_E_CLASSSTRING
    }
}

/******************************************************************************
 *		CLSIDFromString	[OLE32.@]
 *
 * Converts a unique identifier from its string representation into
 * the GUID struct.
 */
#[no_mangle]
pub unsafe extern "system" fn CLSIDFromString(idstr: LPCOLESTR, id: *mut CLSID) -> HRESULT {
    let mut tmp_id: CLSID = core::mem::zeroed();

    if id.is_null() {
        return E_INVALIDARG;
    }

    if guid_from_string(idstr, id) != FALSE {
        return S_OK;
    }

    /* It appears a ProgID is also valid */
    let ret = clsid_from_string_reg(idstr, &raw mut tmp_id);
    if SUCCEEDED(ret) {
        *id = tmp_id;
    }

    ret
}

/******************************************************************************
 *		IIDFromString   [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn IIDFromString(s: LPCOLESTR, iid: *mut IID) -> HRESULT {
    TRACE!();

    if s.is_null() {
        memset(iid as PVOID, 0, size_of::<IID>());
        return S_OK;
    }

    /* length mismatch is a special case */
    if lstrlenW(s) as usize + 1 != CHARS_IN_GUID {
        return E_INVALIDARG;
    }

    if *s != '{' as u16 {
        return CO_E_IIDSTRING;
    }

    if guid_from_string(s, iid) != FALSE {
        S_OK
    } else {
        CO_E_IIDSTRING
    }
}

/******************************************************************************
 *		StringFromCLSID	[OLE32.@]
 *		StringFromIID   [OLE32.@]
 *
 * Converts a GUID into the respective string representation.
 * The target string is allocated using the OLE IMalloc.
 */
#[no_mangle]
pub unsafe extern "system" fn StringFromCLSID(id: REFCLSID, idstr: *mut LPOLESTR) -> HRESULT {
    *idstr = CoTaskMemAlloc(CHARS_IN_GUID * size_of::<WCHAR>()) as LPOLESTR;
    if (*idstr).is_null() {
        return E_OUTOFMEMORY;
    }
    StringFromGUID2(id, *idstr, CHARS_IN_GUID as INT);
    S_OK
}

/******************************************************************************
 *		StringFromGUID2	[OLE32.@]
 *
 * Modified version of StringFromCLSID that allows you to specify max
 * buffer size.
 *
 * RETURNS: The length of the resulting string in characters, or 0 on failure.
 */
#[no_mangle]
pub unsafe extern "system" fn StringFromGUID2(id: REFGUID, str_: LPOLESTR, cmax: INT) -> INT {
    // The C formats with swprintf("{%08X-%04X-...}"); hand-rolled here,
    // byte-exact (uppercase hex, fixed widths, NUL-terminated).
    if id.is_null() || (cmax as usize) < CHARS_IN_GUID {
        return 0;
    }
    const HEXW: &[u8; 16] = b"0123456789ABCDEF";
    unsafe fn put_hex(str_: LPOLESTR, pos: usize, val: u32, digits: usize) -> usize {
        const HEXW: &[u8; 16] = b"0123456789ABCDEF";
        let mut pos = pos;
        let mut i = digits;
        while i > 0 {
            i -= 1;
            let nib = (val >> (i * 4)) & 0xF;
            *str_.add(pos) = HEXW[nib as usize] as u16;
            pos += 1;
        }
        pos
    }
    let g = &*id;
    let mut pos = 0usize;
    *str_.add(pos) = '{' as u16;
    pos += 1;
    pos = put_hex(str_, pos, g.Data1, 8);
    *str_.add(pos) = '-' as u16;
    pos += 1;
    pos = put_hex(str_, pos, g.Data2 as u32, 4);
    *str_.add(pos) = '-' as u16;
    pos += 1;
    pos = put_hex(str_, pos, g.Data3 as u32, 4);
    *str_.add(pos) = '-' as u16;
    pos += 1;
    pos = put_hex(str_, pos, g.Data4[0] as u32, 2);
    pos = put_hex(str_, pos, g.Data4[1] as u32, 2);
    *str_.add(pos) = '-' as u16;
    pos += 1;
    let mut i = 2usize;
    while i < 8 {
        pos = put_hex(str_, pos, g.Data4[i] as u32, 2);
        i += 1;
    }
    *str_.add(pos) = '}' as u16;
    pos += 1;
    *str_.add(pos) = 0;
    CHARS_IN_GUID as INT
}

/// open HKCR\CLSID\{string form of clsid}\{keyname} key
#[no_mangle]
pub unsafe extern "C" fn COM_OpenKeyForCLSID(
    clsid: REFCLSID,
    keyname: LPCWSTR,
    access: REGSAM,
    subkey: *mut HKEY,
) -> HRESULT {
    let wszCLSIDSlash = wide!("CLSID\\");
    // WCHAR path[CHARS_IN_GUID + ARRAY_SIZE(wszCLSIDSlash) - 1]
    let mut path: [WCHAR; CHARS_IN_GUID + 7 - 1] = [0; CHARS_IN_GUID + 6];
    let mut key: HKEY = null_mut();

    lstrcpyW(path.as_mut_ptr(), wszCLSIDSlash.as_ptr());
    StringFromGUID2(
        clsid,
        path.as_mut_ptr().add(lstrlenW(wszCLSIDSlash.as_ptr()) as usize),
        CHARS_IN_GUID as INT,
    );
    let res = open_classes_key(
        HKEY_CLASSES_ROOT,
        path.as_ptr(),
        if !keyname.is_null() { KEY_READ } else { access },
        &raw mut key,
    );
    if res == ERROR_FILE_NOT_FOUND {
        return REGDB_E_CLASSNOTREG;
    } else if res != ERROR_SUCCESS {
        return REGDB_E_READREGDB;
    }

    if keyname.is_null() {
        *subkey = key;
        return S_OK;
    }

    let res = open_classes_key(key, keyname, access, subkey);
    RegCloseKey(key);
    if res == ERROR_FILE_NOT_FOUND {
        return REGDB_E_KEYMISSING;
    } else if res != ERROR_SUCCESS {
        return REGDB_E_READREGDB;
    }

    S_OK
}

/// open HKCR\AppId\{string form of appid clsid} key
#[no_mangle]
pub unsafe extern "C" fn COM_OpenKeyForAppIdFromCLSID(
    clsid: REFCLSID,
    access: REGSAM,
    subkey: *mut HKEY,
) -> HRESULT {
    let szAppId = wide!("AppId");
    let szAppIdKey = wide!("AppId\\");
    let mut buf: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    // WCHAR keyname[ARRAY_SIZE(szAppIdKey) + CHARS_IN_GUID]
    let mut keyname: [WCHAR; 7 + CHARS_IN_GUID] = [0; 7 + CHARS_IN_GUID];
    let mut hkey: HKEY = null_mut();
    let mut type_: DWORD = 0;

    /* read the AppID value under the class's key */
    let hr = COM_OpenKeyForCLSID(clsid, null(), KEY_READ, &raw mut hkey);
    if FAILED(hr) {
        return hr;
    }

    let mut size: DWORD = size_of::<[WCHAR; CHARS_IN_GUID]>() as DWORD;
    let res = RegQueryValueExW(
        hkey,
        szAppId.as_ptr(),
        null_mut(),
        &raw mut type_,
        buf.as_mut_ptr() as *mut BYTE,
        &raw mut size,
    );
    RegCloseKey(hkey);
    if res == ERROR_FILE_NOT_FOUND {
        return REGDB_E_KEYMISSING;
    } else if res != ERROR_SUCCESS || type_ != REG_SZ {
        return REGDB_E_READREGDB;
    }

    lstrcpyW(keyname.as_mut_ptr(), szAppIdKey.as_ptr());
    lstrcatW(keyname.as_mut_ptr(), buf.as_ptr());
    let res = open_classes_key(HKEY_CLASSES_ROOT, keyname.as_ptr(), access, subkey);
    if res == ERROR_FILE_NOT_FOUND {
        return REGDB_E_KEYMISSING;
    } else if res != ERROR_SUCCESS {
        return REGDB_E_READREGDB;
    }

    S_OK
}

/******************************************************************************
 *               ProgIDFromCLSID [OLE32.@]
 *
 * Converts a class id into the respective program ID.
 */
#[no_mangle]
pub unsafe extern "system" fn ProgIDFromCLSID(
    clsid: REFCLSID,
    ppszProgID: *mut LPOLESTR,
) -> HRESULT {
    let wszProgID = wide!("ProgID");
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    let mut hkey: HKEY = null_mut();
    let mut ret: HRESULT;
    let mut progidlen: LONG = 0;

    if ppszProgID.is_null() {
        return E_INVALIDARG;
    }

    *ppszProgID = null_mut();

    data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    if FindActCtxSectionGuid(
        0,
        null(),
        ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION,
        clsid,
        &raw mut data,
    ) != FALSE
    {
        let comclass = data.lpData as *mut comclassredirect_data;
        if (*comclass).progid_len != 0 {
            *ppszProgID = CoTaskMemAlloc(
                (*comclass).progid_len as usize + size_of::<WCHAR>(),
            ) as LPOLESTR;
            if (*ppszProgID).is_null() {
                return E_OUTOFMEMORY;
            }

            let ptrW =
                (comclass as *mut u8).add((*comclass).progid_offset as usize) as *mut WCHAR;
            memcpy(
                *ppszProgID as PVOID,
                ptrW as *const c_void,
                (*comclass).progid_len as usize + size_of::<WCHAR>(),
            );
            return S_OK;
        } else {
            return REGDB_E_CLASSNOTREG;
        }
    }

    ret = COM_OpenKeyForCLSID(clsid, wszProgID.as_ptr(), KEY_READ, &raw mut hkey);
    if FAILED(ret) {
        return ret;
    }

    if RegQueryValueW(hkey, null(), null_mut(), &raw mut progidlen) != 0 {
        ret = REGDB_E_CLASSNOTREG;
    }

    if ret == S_OK {
        *ppszProgID = CoTaskMemAlloc(progidlen as usize * size_of::<WCHAR>()) as LPOLESTR;
        if !(*ppszProgID).is_null() {
            if RegQueryValueW(hkey, null(), *ppszProgID, &raw mut progidlen) != 0 {
                ret = REGDB_E_CLASSNOTREG;
                CoTaskMemFree(*ppszProgID as PVOID);
                *ppszProgID = null_mut();
            }
        } else {
            ret = E_OUTOFMEMORY;
        }
    }

    RegCloseKey(hkey);
    ret
}

/******************************************************************************
 *		CLSIDFromProgID	[OLE32.@]
 *
 * Converts a program id into the respective GUID.
 */
#[no_mangle]
pub unsafe extern "system" fn CLSIDFromProgID(progid: LPCOLESTR, clsid: *mut CLSID) -> HRESULT {
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();

    if progid.is_null() || clsid.is_null() {
        return E_INVALIDARG;
    }

    data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    if FindActCtxSectionStringW(
        0,
        null(),
        ACTIVATION_CONTEXT_SECTION_COM_PROGID_REDIRECTION,
        progid,
        &raw mut data,
    ) != FALSE
    {
        let progiddata = data.lpData as *mut progidredirect_data;
        let alias =
            (data.lpSectionBase as *mut u8).add((*progiddata).clsid_offset as usize) as *mut CLSID;
        *clsid = *alias;
        return S_OK;
    }

    clsid_from_string_reg(progid, clsid)
}

/******************************************************************************
 *              CLSIDFromProgIDEx [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CLSIDFromProgIDEx(progid: LPCOLESTR, clsid: *mut CLSID) -> HRESULT {
    FIXME!("{},{:p}: semi-stub", DbgW(progid), clsid);

    CLSIDFromProgID(progid, clsid)
}

unsafe fn get_ps_clsid_from_registry(
    path: *const WCHAR,
    access: REGSAM,
    pclsid: *mut CLSID,
) -> HRESULT {
    let mut hkey: HKEY = null_mut();
    let mut value: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];

    let access = access | KEY_READ;

    if open_classes_key(HKEY_CLASSES_ROOT, path, access, &raw mut hkey) != 0 {
        return REGDB_E_IIDNOTREG;
    }

    let mut len: DWORD = size_of::<[WCHAR; CHARS_IN_GUID]>() as DWORD;
    if ERROR_SUCCESS
        != RegQueryValueExW(
            hkey,
            null(),
            null_mut(),
            null_mut(),
            value.as_mut_ptr() as *mut BYTE,
            &raw mut len,
        )
    {
        return REGDB_E_IIDNOTREG;
    }
    RegCloseKey(hkey);

    if CLSIDFromString(value.as_ptr(), pclsid) != NOERROR {
        return REGDB_E_IIDNOTREG;
    }

    S_OK
}

/*****************************************************************************
 *             CoGetPSClsid [OLE32.@]
 *
 * Retrieves the CLSID of the proxy/stub factory that implements
 * IPSFactoryBuffer for the specified interface.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetPSClsid(riid: REFIID, pclsid: *mut CLSID) -> HRESULT {
    let wszInterface = wide!("Interface\\"); // 11 chars incl. NUL
    let wszPSC = wide!("\\ProxyStubClsid32"); // 18 chars incl. NUL
    // WCHAR path[ARRAY_SIZE(wszInterface) - 1 + CHARS_IN_GUID - 1 + ARRAY_SIZE(wszPSC)]
    let mut path: [WCHAR; 10 + CHARS_IN_GUID - 1 + 18] = [0; 10 + CHARS_IN_GUID - 1 + 18];
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    let opposite: REGSAM = if size_of::<*mut c_void>() > size_of::<i32>() {
        KEY_WOW64_32KEY
    } else {
        KEY_WOW64_64KEY
    };
    let mut is_wow64: BOOL = FALSE;

    TRACE!();

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }
    apartment_release(apt);

    if pclsid.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(&raw mut cs_registered_psclsid_list);

    let mut cursor = registered_psclsid_list.next;
    while cursor != &raw mut registered_psclsid_list {
        let rpsc: *mut registered_psclsid =
            list_entry(cursor, offset_of!(registered_psclsid, entry));
        if guid_eq(&(*rpsc).iid, riid) {
            *pclsid = (*rpsc).clsid;
            LeaveCriticalSection(&raw mut cs_registered_psclsid_list);
            return S_OK;
        }
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(&raw mut cs_registered_psclsid_list);

    data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    if FindActCtxSectionGuid(
        0,
        null(),
        ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION,
        riid,
        &raw mut data,
    ) != FALSE
    {
        let ifaceps = data.lpData as *mut ifacepsredirect_data;
        *pclsid = (*ifaceps).iid;
        return S_OK;
    }

    /* Interface\\{string form of riid}\\ProxyStubClsid32 */
    lstrcpyW(path.as_mut_ptr(), wszInterface.as_ptr());
    StringFromGUID2(riid, path.as_mut_ptr().add(11 - 1), CHARS_IN_GUID as INT);
    lstrcpyW(
        path.as_mut_ptr().add(11 - 1 + CHARS_IN_GUID - 1),
        wszPSC.as_ptr(),
    );

    let mut hr = get_ps_clsid_from_registry(path.as_ptr(), 0, pclsid);
    if FAILED(hr)
        && (opposite == KEY_WOW64_32KEY
            || (IsWow64Process(GetCurrentProcess(), &raw mut is_wow64) != FALSE
                && is_wow64 != FALSE))
    {
        hr = get_ps_clsid_from_registry(path.as_ptr(), opposite, pclsid);
    }

    if hr == S_OK {
        TRACE!();
    } else {
        WARN!();
    }

    hr
}

/*****************************************************************************
 *             CoRegisterPSClsid [OLE32.@]
 *
 * Register a proxy/stub CLSID for the given interface in the current process
 * only.
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterPSClsid(riid: REFIID, rclsid: REFCLSID) -> HRESULT {
    TRACE!();

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }
    apartment_release(apt);

    EnterCriticalSection(&raw mut cs_registered_psclsid_list);

    let mut cursor = registered_psclsid_list.next;
    while cursor != &raw mut registered_psclsid_list {
        let rpsc: *mut registered_psclsid =
            list_entry(cursor, offset_of!(registered_psclsid, entry));
        if guid_eq(&(*rpsc).iid, riid) {
            (*rpsc).clsid = *rclsid;
            LeaveCriticalSection(&raw mut cs_registered_psclsid_list);
            return S_OK;
        }
        cursor = (*cursor).next;
    }

    let rpsc =
        HeapAlloc(GetProcessHeap(), 0, size_of::<registered_psclsid>()) as *mut registered_psclsid;
    if rpsc.is_null() {
        LeaveCriticalSection(&raw mut cs_registered_psclsid_list);
        return E_OUTOFMEMORY;
    }

    (*rpsc).iid = *riid;
    (*rpsc).clsid = *rclsid;
    list_add_head(&raw mut registered_psclsid_list, &raw mut (*rpsc).entry);

    LeaveCriticalSection(&raw mut cs_registered_psclsid_list);

    S_OK
}

/***
 * COM_GetRegisteredClassObject
 *
 * This internal method is used to scan the registered class list to
 * find a class object.
 */
unsafe fn COM_GetRegisteredClassObject(
    apt: *const apartment,
    rclsid: REFCLSID,
    dwClsContext: DWORD,
    ppUnk: *mut *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT = S_FALSE;

    EnterCriticalSection(&raw mut csRegisteredClassList);

    let mut cursor = RegisteredClassList.next;
    while cursor != &raw mut RegisteredClassList {
        let curClass: *mut RegisteredClass = list_entry(cursor, offset_of!(RegisteredClass, entry));
        /*
         * Check if we have a match on the class ID and context.
         */
        if (*apt).oxid == (*curClass).apartment_id
            && (dwClsContext & (*curClass).runContext) != 0
            && guid_eq(&(*curClass).classIdentifier, rclsid)
        {
            /*
             * We have a match, return the pointer to the class object.
             */
            *ppUnk = (*curClass).classObject;

            IUnknown_AddRef((*curClass).classObject);

            hr = S_OK;
            break;
        }
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(&raw mut csRegisteredClassList);

    hr
}

/******************************************************************************
 *		CoRegisterClassObject	[OLE32.@]
 *
 * Registers the class object for a given class ID. Servers housed in EXE
 * files use this method instead of exporting DllGetClassObject to allow
 * other code to connect to their objects.
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterClassObject(
    rclsid: REFCLSID,
    pUnk: *mut IUnknown,
    dwClsContext: DWORD,
    flags: DWORD,
    lpdwRegister: *mut DWORD,
) -> HRESULT {
    let mut dwClsContext = dwClsContext;
    let mut foundObject: *mut IUnknown = null_mut();

    TRACE!();

    if lpdwRegister.is_null() || pUnk.is_null() {
        return E_INVALIDARG;
    }

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("COM was not initialized");
        return CO_E_NOTINITIALIZED;
    }

    *lpdwRegister = 0;

    /* REGCLS_MULTIPLEUSE implies registering as inproc server. This is what
     * differentiates the flag from REGCLS_MULTI_SEPARATE. */
    if (flags & REGCLS_MULTIPLEUSE) != 0 {
        dwClsContext |= CLSCTX_INPROC_SERVER;
    }

    /*
     * First, check if the class is already registered.
     * If it is, this should cause an error.
     */
    let mut hr = COM_GetRegisteredClassObject(apt, rclsid, dwClsContext, &raw mut foundObject);
    if hr == S_OK {
        if (flags & REGCLS_MULTIPLEUSE) != 0 {
            if (dwClsContext & CLSCTX_LOCAL_SERVER) != 0 {
                hr = CoLockObjectExternal(foundObject, TRUE, FALSE);
            }
            IUnknown_Release(foundObject);
            apartment_release(apt);
            return hr;
        }
        IUnknown_Release(foundObject);
        ERR!("object already registered for class {}", DbgGuid(rclsid));
        apartment_release(apt);
        return CO_E_OBJISREG;
    }

    let newClass =
        HeapAlloc(GetProcessHeap(), 0, size_of::<RegisteredClass>()) as *mut RegisteredClass;
    if newClass.is_null() {
        apartment_release(apt);
        return E_OUTOFMEMORY;
    }

    (*newClass).classIdentifier = *rclsid;
    (*newClass).apartment_id = (*apt).oxid;
    (*newClass).runContext = dwClsContext;
    (*newClass).connectFlags = flags;
    (*newClass).RpcRegistration = null_mut();

    (*newClass).dwCookie = InterlockedIncrement(&raw mut next_cookie) as DWORD;
    if (*newClass).dwCookie == 0 {
        (*newClass).dwCookie = InterlockedIncrement(&raw mut next_cookie) as DWORD;
    }

    /*
     * Since we're making a copy of the object pointer, we have to increase its
     * reference count.
     */
    (*newClass).classObject = pUnk;
    IUnknown_AddRef((*newClass).classObject);

    EnterCriticalSection(&raw mut csRegisteredClassList);
    list_add_tail(&raw mut RegisteredClassList, &raw mut (*newClass).entry);
    LeaveCriticalSection(&raw mut csRegisteredClassList);

    *lpdwRegister = (*newClass).dwCookie;

    if (dwClsContext & CLSCTX_LOCAL_SERVER) != 0 {
        let mut marshal_stream: *mut IStream = null_mut();

        hr = get_local_server_stream(apt, &raw mut marshal_stream);
        if FAILED(hr) {
            apartment_release(apt);
            return hr;
        }

        hr = RPC_StartLocalServer(
            &(*newClass).classIdentifier,
            marshal_stream,
            (flags & (REGCLS_MULTIPLEUSE | REGCLS_MULTI_SEPARATE)) as BOOL,
            &raw mut (*newClass).RpcRegistration,
        );
        ((*(*marshal_stream).lpVtbl).Release)(marshal_stream);
    }
    apartment_release(apt);
    S_OK
}

unsafe fn get_threading_model(data: *const class_reg_data) -> comclass_threadingmodel {
    if (*data).hkey != FALSE {
        let wszThreadingModel = wide!("ThreadingModel");
        let wszApartment = wide!("Apartment");
        let wszFree = wide!("Free");
        let wszBoth = wide!("Both");
        let mut threading_model: [WCHAR; 10] = [0; 10]; /* lstrlenW(L"apartment")+1 */
        let mut dwLength: DWORD = size_of::<[WCHAR; 10]>() as DWORD;
        let mut keytype: DWORD = 0;

        let ret = RegQueryValueExW(
            (*data).u.hkey,
            wszThreadingModel.as_ptr(),
            null_mut(),
            &raw mut keytype,
            threading_model.as_mut_ptr() as *mut BYTE,
            &raw mut dwLength,
        );
        if ret != ERROR_SUCCESS || keytype != REG_SZ {
            threading_model[0] = 0;
        }

        if wcsicmp(threading_model.as_ptr(), wszApartment.as_ptr()) == 0 {
            return ThreadingModel_Apartment;
        }
        if wcsicmp(threading_model.as_ptr(), wszFree.as_ptr()) == 0 {
            return ThreadingModel_Free;
        }
        if wcsicmp(threading_model.as_ptr(), wszBoth.as_ptr()) == 0 {
            return ThreadingModel_Both;
        }

        /* there's not specific handling for this case */
        if threading_model[0] != 0 {
            return ThreadingModel_Neutral;
        }
        ThreadingModel_No
    } else {
        (*(*data).u.actctx.data).model as comclass_threadingmodel
    }
}

unsafe fn get_inproc_class_object(
    apt: *mut APARTMENT,
    regdata: *const class_reg_data,
    rclsid: REFCLSID,
    riid: REFIID,
    hostifnecessary: BOOL,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut dllpath: [WCHAR; MAX_PATH + 1] = [0; MAX_PATH + 1];
    let apartment_threaded: BOOL;

    if hostifnecessary != FALSE {
        let model = get_threading_model(regdata);

        if model == ThreadingModel_Apartment {
            apartment_threaded = TRUE;
            if (*apt).multi_threaded != FALSE {
                return apartment_hostobject_in_hostapt(
                    apt, FALSE, FALSE, regdata, rclsid, riid, ppv,
                );
            }
        } else if model == ThreadingModel_Free {
            apartment_threaded = FALSE;
            if (*apt).multi_threaded == FALSE {
                return apartment_hostobject_in_hostapt(
                    apt, TRUE, FALSE, regdata, rclsid, riid, ppv,
                );
            }
        }
        /* everything except "Apartment", "Free" and "Both" */
        else if model != ThreadingModel_Both {
            apartment_threaded = TRUE;
            /* everything else is main-threaded */
            if model != ThreadingModel_No {
                FIXME!(
                    "unrecognised threading model {} for object {}, should be main-threaded?",
                    model, DbgGuid(rclsid)
                );
            }

            if (*apt).multi_threaded != FALSE || (*apt).main == FALSE {
                return apartment_hostobject_in_hostapt(
                    apt, FALSE, TRUE, regdata, rclsid, riid, ppv,
                );
            }
        } else {
            apartment_threaded = FALSE;
        }
    } else {
        apartment_threaded = ((*apt).multi_threaded == FALSE) as BOOL;
    }

    if get_object_dll_path(regdata, dllpath.as_mut_ptr(), (MAX_PATH + 1) as DWORD) == FALSE {
        /* failure: CLSID is not found in registry */
        WARN!();
        return REGDB_E_CLASSNOTREG;
    }

    apartment_getclassobject(apt, dllpath.as_ptr(), apartment_threaded, rclsid, riid, ppv)
}

/***********************************************************************
 *           CoGetClassObject [OLE32.@]
 *
 * Creates an object of the specified class.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetClassObject(
    rclsid: REFCLSID,
    dwClsContext: DWORD,
    pServerInfo: *mut COSERVERINFO,
    iid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut clsreg: class_reg_data = core::mem::zeroed();
    let mut regClassObject: *mut IUnknown = null_mut();
    let mut hres: HRESULT = E_UNEXPECTED;

    TRACE!();

    if ppv.is_null() {
        return E_INVALIDARG;
    }

    *ppv = null_mut();

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }

    if !pServerInfo.is_null() {
        FIXME!(
            "pServerInfo->name={} pAuthInfo={:p}",
            DbgW((*pServerInfo).pwszName),
            (*pServerInfo).pAuthInfo
        );
    }

    if (CLSCTX_INPROC_SERVER & dwClsContext) != 0 {
        if guid_eq(rclsid, &CLSID_InProcFreeMarshaler) {
            apartment_release(apt);
            return FTMarshalCF_Create(iid, ppv);
        }
        if guid_eq(rclsid, &CLSID_GlobalOptions) {
            let cf = &raw mut GlobalOptionsCF;
            return ((*(*cf).lpVtbl).QueryInterface)(cf, iid, ppv);
        }
    }

    if (CLSCTX_INPROC & dwClsContext) != 0 {
        let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();

        data.cbSize = size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
        /* search activation context first */
        if FindActCtxSectionGuid(
            FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX,
            null(),
            ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION,
            rclsid,
            &raw mut data,
        ) != FALSE
        {
            let comclass = data.lpData as *mut comclassredirect_data;

            clsreg.u.actctx = class_reg_data_u_actctx {
                hactctx: data.hActCtx,
                data: data.lpData as *mut comclassredirect_data,
                section: data.lpSectionBase,
            };
            clsreg.hkey = FALSE;

            hres = get_inproc_class_object(
                apt,
                &clsreg,
                &(*comclass).clsid,
                iid,
                ((dwClsContext & WINE_CLSCTX_DONT_HOST) == 0) as BOOL,
                ppv,
            );
            ReleaseActCtx(data.hActCtx);
            apartment_release(apt);
            return hres;
        }
    }

    /*
     * First, try and see if we can't match the class ID with one of the
     * registered classes.
     */
    if S_OK == COM_GetRegisteredClassObject(apt, rclsid, dwClsContext, &raw mut regClassObject) {
        /* Get the required interface from the retrieved pointer. */
        hres = IUnknown_QueryInterface(regClassObject, iid, ppv);

        /*
         * Since QI got another reference on the pointer, we want to release the
         * one we already have. If QI was unsuccessful, this will release the object. This
         * is good since we are not returning it in the "out" parameter.
         */
        IUnknown_Release(regClassObject);
        apartment_release(apt);
        return hres;
    }

    /* First try in-process server */
    if (CLSCTX_INPROC_SERVER & dwClsContext) != 0 {
        let wszInprocServer32 = wide!("InprocServer32");
        let mut hkey: HKEY = null_mut();

        hres = COM_OpenKeyForCLSID(rclsid, wszInprocServer32.as_ptr(), KEY_READ, &raw mut hkey);
        if FAILED(hres) {
            if hres == REGDB_E_CLASSNOTREG {
                ERR!("class {} not registered", DbgGuid(rclsid));
            } else if hres == REGDB_E_KEYMISSING {
                WARN!();
                hres = REGDB_E_CLASSNOTREG;
            }
        }

        if SUCCEEDED(hres) {
            clsreg.u.hkey = hkey;
            clsreg.hkey = TRUE;

            hres = get_inproc_class_object(
                apt,
                &clsreg,
                rclsid,
                iid,
                ((dwClsContext & WINE_CLSCTX_DONT_HOST) == 0) as BOOL,
                ppv,
            );
            RegCloseKey(hkey);
        }

        /* return if we got a class, otherwise fall through to one of the
         * other types */
        if SUCCEEDED(hres) {
            apartment_release(apt);
            return hres;
        }
    }

    /* Next try in-process handler */
    if (CLSCTX_INPROC_HANDLER & dwClsContext) != 0 {
        let wszInprocHandler32 = wide!("InprocHandler32");
        let mut hkey: HKEY = null_mut();

        hres = COM_OpenKeyForCLSID(rclsid, wszInprocHandler32.as_ptr(), KEY_READ, &raw mut hkey);
        if FAILED(hres) {
            if hres == REGDB_E_CLASSNOTREG {
                ERR!("class {} not registered", DbgGuid(rclsid));
            } else if hres == REGDB_E_KEYMISSING {
                WARN!();
                hres = REGDB_E_CLASSNOTREG;
            }
        }

        if SUCCEEDED(hres) {
            clsreg.u.hkey = hkey;
            clsreg.hkey = TRUE;

            hres = get_inproc_class_object(
                apt,
                &clsreg,
                rclsid,
                iid,
                ((dwClsContext & WINE_CLSCTX_DONT_HOST) == 0) as BOOL,
                ppv,
            );
            RegCloseKey(hkey);
        }

        /* return if we got a class, otherwise fall through to one of the
         * other types */
        if SUCCEEDED(hres) {
            apartment_release(apt);
            return hres;
        }
    }
    apartment_release(apt);

    /* Next try out of process */
    if (CLSCTX_LOCAL_SERVER & dwClsContext) != 0 {
        hres = RPC_GetLocalClassObject(rclsid, iid, ppv);
        if SUCCEEDED(hres) {
            return hres;
        }
    }

    /* Finally try remote: this requires networked DCOM (a lot of work) */
    if (CLSCTX_REMOTE_SERVER & dwClsContext) != 0 {
        FIXME!("CLSCTX_REMOTE_SERVER not supported");
        hres = REGDB_E_CLASSNOTREG;
    }

    if FAILED(hres) {
        ERR!(
            "no class object {} could be created for context {:#x}",
            DbgGuid(rclsid), dwClsContext
        );
    }
    hres
}

/***********************************************************************
 *        CoResumeClassObjects (OLE32.@)
 *
 * Resumes all class objects registered with REGCLS_SUSPENDED.
 */
#[no_mangle]
pub unsafe extern "system" fn CoResumeClassObjects() -> HRESULT {
    FIXME!("stub");
    S_OK
}

/***********************************************************************
 *           CoCreateInstance [OLE32.@]
 *
 * Creates an instance of the specified class.
 */
#[no_mangle]
pub unsafe extern "system" fn CoCreateInstance(
    rclsid: REFCLSID,
    pUnkOuter: *mut IUnknown,
    dwClsContext: DWORD,
    iid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut multi_qi = MULTI_QI { pIID: iid, pItf: null_mut(), hr: 0 };

    TRACE!();

    if ppv.is_null() {
        return E_POINTER;
    }

    let hres = CoCreateInstanceEx(rclsid, pUnkOuter, dwClsContext, null_mut(), 1, &raw mut multi_qi);
    *ppv = multi_qi.pItf as LPVOID;
    hres
}

unsafe fn init_multi_qi(count: DWORD, mqi: *mut MULTI_QI, hr: HRESULT) {
    let mut i: ULONG = 0;
    while i < count {
        (*mqi.add(i as usize)).pItf = null_mut();
        (*mqi.add(i as usize)).hr = hr;
        i += 1;
    }
}

unsafe fn return_multi_qi(
    unk: *mut IUnknown,
    count: DWORD,
    mqi: *mut MULTI_QI,
    include_unk: BOOL,
) -> HRESULT {
    let mut index: ULONG = 0;
    let mut fetched: ULONG = 0;

    if include_unk != FALSE {
        (*mqi).hr = S_OK;
        (*mqi).pItf = unk;
        index = 1;
        fetched = 1;
    }

    while index < count {
        let e = mqi.add(index as usize);
        (*e).hr = IUnknown_QueryInterface(
            unk,
            (*e).pIID,
            &raw mut (*e).pItf as *mut *mut c_void,
        );
        if (*e).hr == S_OK {
            fetched += 1;
        }
        index += 1;
    }

    if include_unk == FALSE {
        IUnknown_Release(unk);
    }

    if fetched == 0 {
        return E_NOINTERFACE;
    }

    if fetched == count {
        S_OK
    } else {
        CO_S_NOTALLINTERFACES
    }
}

/***********************************************************************
 *           CoCreateInstanceEx [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoCreateInstanceEx(
    rclsid: REFCLSID,
    pUnkOuter: *mut IUnknown,
    dwClsContext: DWORD,
    pServerInfo: *mut COSERVERINFO,
    cmq: ULONG,
    pResults: *mut MULTI_QI,
) -> HRESULT {
    let mut unk: *mut IUnknown = null_mut();
    let mut cf: *mut IClassFactory = null_mut();
    let mut clsid: CLSID = core::mem::zeroed();

    TRACE!();

    if cmq == 0 || pResults.is_null() {
        return E_INVALIDARG;
    }

    if !pServerInfo.is_null() {
        FIXME!("() non-NULL pServerInfo not supported!");
    }

    init_multi_qi(cmq, pResults, E_NOINTERFACE);

    let mut hres = CoGetTreatAsClass(rclsid, &raw mut clsid);
    if FAILED(hres) {
        clsid = *rclsid;
    }

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }
    apartment_release(apt);

    /*
     * The Standard Global Interface Table (GIT) object is a process-wide singleton.
     */
    if guid_eq(&clsid, &CLSID_StdGlobalInterfaceTable) {
        let git = get_std_git();
        TRACE!();
        return return_multi_qi(git as *mut IUnknown, cmq, pResults, FALSE);
    }

    if guid_eq(&clsid, &CLSID_ManualResetEvent) {
        hres = ManualResetEvent_Construct(
            pUnkOuter,
            (*pResults).pIID,
            &raw mut unk as *mut *mut c_void,
        );
        if FAILED(hres) {
            return hres;
        }
        return return_multi_qi(unk, cmq, pResults, TRUE);
    }

    /*
     * Get a class factory to construct the object we want.
     */
    hres = CoGetClassObject(
        &clsid,
        dwClsContext,
        null_mut(),
        &IID_IClassFactory,
        &raw mut cf as *mut LPVOID,
    );
    if FAILED(hres) {
        return hres;
    }

    /*
     * Create the object and don't forget to release the factory
     */
    hres = ((*(*cf).lpVtbl).CreateInstance)(
        cf,
        pUnkOuter,
        (*pResults).pIID,
        &raw mut unk as *mut *mut c_void,
    );
    ((*(*cf).lpVtbl).Release)(cf);
    if FAILED(hres) {
        if hres == CLASS_E_NOAGGREGATION && !pUnkOuter.is_null() {
            FIXME!("Class {} does not support aggregation", DbgGuid(&clsid));
        } else {
            FIXME!(
                "no instance created for interface {} of class {}, hres is {:#010x}",
                DbgGuid((*pResults).pIID), DbgGuid(&clsid), hres as u32
            );
        }
        return hres;
    }

    return_multi_qi(unk, cmq, pResults, TRUE)
}

/***********************************************************************
 *           CoGetInstanceFromFile [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetInstanceFromFile(
    server_info: *mut COSERVERINFO,
    rclsid: *mut CLSID,
    outer: *mut IUnknown,
    cls_context: DWORD,
    grfmode: DWORD,
    filename: *mut u16,
    count: DWORD,
    results: *mut MULTI_QI,
) -> HRESULT {
    let mut pf: *mut IPersistFile = null_mut();
    let mut unk: *mut IUnknown = null_mut();
    let mut clsid: CLSID = core::mem::zeroed();
    let mut rclsid = rclsid;

    if count == 0 || results.is_null() {
        return E_INVALIDARG;
    }

    if !server_info.is_null() {
        FIXME!("() non-NULL server_info not supported");
    }

    init_multi_qi(count, results, E_NOINTERFACE);

    /* optionally get CLSID from a file */
    if rclsid.is_null() {
        let hr = GetClassFile(filename, &raw mut clsid);
        if FAILED(hr) {
            ERR!("failed to get CLSID from a file");
            return hr;
        }

        rclsid = &raw mut clsid;
    }

    let mut hr = CoCreateInstance(
        rclsid,
        outer,
        cls_context,
        &IID_IUnknown,
        &raw mut unk as *mut LPVOID,
    );

    if hr != S_OK {
        init_multi_qi(count, results, hr);
        return hr;
    }

    /* init from file */
    hr = IUnknown_QueryInterface(unk, &IID_IPersistFile, &raw mut pf as *mut *mut c_void);
    if FAILED(hr) {
        init_multi_qi(count, results, hr);
        IUnknown_Release(unk);
        return hr;
    }

    hr = ((*(*pf).lpVtbl).Load)(pf, filename, grfmode);
    ((*(*pf).lpVtbl).Release)(pf);
    if SUCCEEDED(hr) {
        return_multi_qi(unk, count, results, FALSE)
    } else {
        init_multi_qi(count, results, hr);
        IUnknown_Release(unk);
        hr
    }
}

/***********************************************************************
 *           CoGetInstanceFromIStorage [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetInstanceFromIStorage(
    server_info: *mut COSERVERINFO,
    rclsid: *mut CLSID,
    outer: *mut IUnknown,
    cls_context: DWORD,
    storage: *mut IStorage,
    count: DWORD,
    results: *mut MULTI_QI,
) -> HRESULT {
    let mut ps: *mut IPersistStorage = null_mut();
    let mut unk: *mut IUnknown = null_mut();
    let mut stat: STATSTG = core::mem::zeroed();
    let mut rclsid = rclsid;

    if count == 0 || results.is_null() || storage.is_null() {
        return E_INVALIDARG;
    }

    if !server_info.is_null() {
        FIXME!("() non-NULL server_info not supported");
    }

    init_multi_qi(count, results, E_NOINTERFACE);

    /* optionally get CLSID from a file */
    if rclsid.is_null() {
        memset(&raw mut stat.clsid as PVOID, 0, size_of::<CLSID>());
        let hr = ((*(*storage).lpVtbl).Stat)(storage, &raw mut stat, STATFLAG_NONAME);
        if FAILED(hr) {
            ERR!("failed to get CLSID from a file");
            return hr;
        }

        rclsid = &raw mut stat.clsid;
    }

    let mut hr = CoCreateInstance(
        rclsid,
        outer,
        cls_context,
        &IID_IUnknown,
        &raw mut unk as *mut LPVOID,
    );

    if hr != S_OK {
        return hr;
    }

    /* init from IStorage */
    hr = IUnknown_QueryInterface(unk, &IID_IPersistStorage, &raw mut ps as *mut *mut c_void);
    if FAILED(hr) {
        ERR!("failed to get IPersistStorage");
    }

    if !ps.is_null() {
        ((*(*ps).lpVtbl).Load)(ps, storage);
        ((*(*ps).lpVtbl).Release)(ps);
    }

    return_multi_qi(unk, count, results, FALSE)
}

/***********************************************************************
 *           CoLoadLibrary (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn CoLoadLibrary(lpszLibName: LPOLESTR, bAutoFree: BOOL) -> HINSTANCE {
    TRACE!();

    LoadLibraryExW(lpszLibName, null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH)
}

/***********************************************************************
 *           CoFreeLibrary [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoFreeLibrary(hLibrary: HINSTANCE) {
    FreeLibrary(hLibrary);
}

/***********************************************************************
 *           CoFreeAllLibraries [OLE32.@]
 *
 * Function for backwards compatibility only. Does nothing.
 */
#[no_mangle]
pub unsafe extern "system" fn CoFreeAllLibraries() {
    /* NOP */
}

/***********************************************************************
 *           CoFreeUnusedLibrariesEx [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoFreeUnusedLibrariesEx(dwUnloadDelay: DWORD, dwReserved: DWORD) {
    let apt = COM_CurrentApt();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return;
    }

    apartment_freeunusedlibraries(apt, dwUnloadDelay);
}

/***********************************************************************
 *           CoFreeUnusedLibraries [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoFreeUnusedLibraries() {
    CoFreeUnusedLibrariesEx(INFINITE, 0);
}

/***********************************************************************
 *           CoFileTimeNow [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoFileTimeNow(lpFileTime: *mut FILETIME) -> HRESULT {
    GetSystemTimeAsFileTime(lpFileTime);
    S_OK
}

/******************************************************************************
 *		CoLockObjectExternal	[OLE32.@]
 *
 * Increments or decrements the external reference count of a stub object.
 */
#[no_mangle]
pub unsafe extern "system" fn CoLockObjectExternal(
    pUnk: *mut IUnknown,
    fLock: BOOL,
    fLastUnlockReleases: BOOL,
) -> HRESULT {
    TRACE!();

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }

    let stubmgr = get_stub_manager_from_object(apt, pUnk, fLock);
    if stubmgr.is_null() {
        WARN!();
        /* Note: native is pretty broken here because it just silently
         * fails, without returning an appropriate error code, making apps
         * think that the object was disconnected, when it actually wasn't */
        apartment_release(apt);
        return S_OK;
    }

    if fLock != FALSE {
        stub_manager_ext_addref(stubmgr, 1, FALSE);
    } else {
        stub_manager_ext_release(stubmgr, 1, FALSE, fLastUnlockReleases);
    }

    stub_manager_int_release(stubmgr);
    apartment_release(apt);
    S_OK
}

/***********************************************************************
 *           CoInitializeWOW (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn CoInitializeWOW(x: DWORD, y: DWORD) -> HRESULT {
    FIXME!("({:#010x},{:#010x}),stub!", x, y);
    0
}

/***********************************************************************
 *           CoGetState [OLE32.@]
 *
 * Retrieves the thread state object previously stored by CoSetState().
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetState(ppv: *mut *mut IUnknown) -> HRESULT {
    let info = COM_CurrentInfo();
    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppv = null_mut();

    if !(*info).state.is_null() {
        IUnknown_AddRef((*info).state);
        *ppv = (*info).state;
        TRACE!();
    }

    S_OK
}

/***********************************************************************
 *           CoSetState [OLE32.@]
 *
 * Sets the thread state object.
 */
#[no_mangle]
pub unsafe extern "system" fn CoSetState(pv: *mut IUnknown) -> HRESULT {
    let info = COM_CurrentInfo();
    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    if !pv.is_null() {
        IUnknown_AddRef(pv);
    }

    if !(*info).state.is_null() {
        TRACE!();
        IUnknown_Release((*info).state);
    }

    (*info).state = pv;

    S_OK
}

/******************************************************************************
 *              CoTreatAsClass        [OLE32.@]
 *
 * Sets the TreatAs value of a class.
 */
#[no_mangle]
pub unsafe extern "system" fn CoTreatAsClass(clsidOld: REFCLSID, clsidNew: REFCLSID) -> HRESULT {
    let wszAutoTreatAs = wide!("AutoTreatAs");
    let wszTreatAs = wide!("TreatAs");
    let mut hkey: HKEY = null_mut();
    let mut szClsidNew: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    let mut res: HRESULT = S_OK;
    let mut auto_treat_as: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    let mut auto_treat_as_size: LONG = size_of::<[WCHAR; CHARS_IN_GUID]>() as LONG;
    let mut id: CLSID = core::mem::zeroed();

    res = COM_OpenKeyForCLSID(clsidOld, null(), KEY_READ | KEY_WRITE, &raw mut hkey);
    if FAILED(res) {
        // goto done
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }

    'done: {
        if guid_eq(clsidOld, clsidNew) {
            if RegQueryValueW(
                hkey,
                wszAutoTreatAs.as_ptr(),
                auto_treat_as.as_mut_ptr(),
                &raw mut auto_treat_as_size,
            ) == 0
                && CLSIDFromString(auto_treat_as.as_ptr(), &raw mut id) == S_OK
            {
                if RegSetValueW(
                    hkey,
                    wszTreatAs.as_ptr(),
                    REG_SZ,
                    auto_treat_as.as_ptr(),
                    size_of::<[WCHAR; CHARS_IN_GUID]>() as DWORD,
                ) != 0
                {
                    res = REGDB_E_WRITEREGDB;
                    break 'done;
                }
            } else {
                if RegDeleteKeyW(hkey, wszTreatAs.as_ptr()) != 0 {
                    res = REGDB_E_WRITEREGDB;
                }
                break 'done;
            }
        } else {
            if guid_eq(clsidNew, &CLSID_NULL) {
                RegDeleteKeyW(hkey, wszTreatAs.as_ptr());
            } else {
                if StringFromGUID2(clsidNew, szClsidNew.as_mut_ptr(), CHARS_IN_GUID as INT) == 0 {
                    WARN!();
                    res = E_FAIL;
                    break 'done;
                }

                if RegSetValueW(
                    hkey,
                    wszTreatAs.as_ptr(),
                    REG_SZ,
                    szClsidNew.as_ptr(),
                    size_of::<[WCHAR; CHARS_IN_GUID]>() as DWORD,
                ) != ERROR_SUCCESS
                {
                    WARN!();
                    res = REGDB_E_WRITEREGDB;
                    break 'done;
                }
            }
        }
    }

    // done:
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
    res
}

/******************************************************************************
 *              CoGetTreatAsClass        [OLE32.@]
 *
 * Gets the TreatAs value of a class.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetTreatAsClass(clsidOld: REFCLSID, clsidNew: *mut CLSID) -> HRESULT {
    let wszTreatAs = wide!("TreatAs");
    let mut hkey: HKEY = null_mut();
    let mut szClsidNew: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    let mut res: HRESULT = S_OK;
    let mut len: LONG = size_of::<[WCHAR; CHARS_IN_GUID]>() as LONG;

    TRACE!();

    if clsidOld.is_null() || clsidNew.is_null() {
        return E_INVALIDARG;
    }

    *clsidNew = *clsidOld; /* copy over old value */

    'done: {
        res = COM_OpenKeyForCLSID(clsidOld, wszTreatAs.as_ptr(), KEY_READ, &raw mut hkey);
        if FAILED(res) {
            res = S_FALSE;
            break 'done;
        }
        if RegQueryValueW(hkey, null(), szClsidNew.as_mut_ptr(), &raw mut len) != 0 {
            res = S_FALSE;
            break 'done;
        }
        res = CLSIDFromString(szClsidNew.as_ptr(), clsidNew);
        if FAILED(res) {
            ERR!(
                "Failed CLSIDFromStringA({}), hres {:#010x}",
                DbgW(szClsidNew.as_ptr()), res as u32
            );
        }
    }

    // done:
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
    res
}

/******************************************************************************
 *		CoGetCurrentProcess	[OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetCurrentProcess() -> DWORD {
    GetCurrentProcessId()
}

/***********************************************************************
 *              CoGetCurrentLogicalThreadId        [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetCurrentLogicalThreadId(id: *mut GUID) -> HRESULT {
    TRACE!();

    if id.is_null() {
        return E_INVALIDARG;
    }

    *id = COM_CurrentCausalityId();
    S_OK
}

/******************************************************************************
 *		CoRegisterMessageFilter	[OLE32.@]
 *
 * Registers a message filter.
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterMessageFilter(
    lpMessageFilter: LPMESSAGEFILTER,
    lplpMessageFilter: *mut LPMESSAGEFILTER,
) -> HRESULT {
    TRACE!();

    let apt = COM_CurrentApt();

    /* can't set a message filter in a multi-threaded apartment */
    if apt.is_null() || (*apt).multi_threaded != FALSE {
        WARN!();
        return CO_E_NOT_SUPPORTED;
    }

    if !lpMessageFilter.is_null() {
        ((*(*lpMessageFilter).lpVtbl).AddRef)(lpMessageFilter);
    }

    EnterCriticalSection(&raw mut (*apt).cs);

    let lpOldMessageFilter = (*apt).filter;
    (*apt).filter = lpMessageFilter;

    LeaveCriticalSection(&raw mut (*apt).cs);

    if !lplpMessageFilter.is_null() {
        *lplpMessageFilter = lpOldMessageFilter;
    } else if !lpOldMessageFilter.is_null() {
        ((*(*lpOldMessageFilter).lpVtbl).Release)(lpOldMessageFilter);
    }

    S_OK
}

/***********************************************************************
 *           CoIsOle1Class [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoIsOle1Class(clsid: REFCLSID) -> BOOL {
    FIXME!("{}", DbgGuid(clsid));
    FALSE
}

/***********************************************************************
 *           IsEqualGUID [OLE32.@]
 *
 * Compares two Unique Identifiers.
 */
#[no_mangle]
pub unsafe extern "system" fn IsEqualGUID(rguid1: REFGUID, rguid2: REFGUID) -> BOOL {
    (memcmp(rguid1 as *const c_void, rguid2 as *const c_void, size_of::<GUID>()) == 0) as BOOL
}

/***********************************************************************
 *           CoInitializeSecurity [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoInitializeSecurity(
    pSecDesc: PVOID,
    cAuthSvc: LONG,
    asAuthSvc: *mut SOLE_AUTHENTICATION_SERVICE,
    pReserved1: PVOID,
    dwAuthnLevel: DWORD,
    dwImpLevel: DWORD,
    pReserved2: PVOID,
    dwCapabilities: DWORD,
    pReserved3: PVOID,
) -> HRESULT {
    FIXME!(
        "({:p},{},{:p},{:p},{},{},{:p},{},{:p}) - stub!",
        pSecDesc, cAuthSvc, asAuthSvc, pReserved1, dwAuthnLevel, dwImpLevel, pReserved2,
        dwCapabilities, pReserved3
    );
    S_OK
}

/***********************************************************************
 *           CoSuspendClassObjects [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoSuspendClassObjects() -> HRESULT {
    FIXME!("");
    S_OK
}

/***********************************************************************
 *           CoAddRefServerProcess [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoAddRefServerProcess() -> ULONG {
    TRACE!();

    EnterCriticalSection(&raw mut csRegisteredClassList);
    s_COMServerProcessReferences += 1;
    let refs = s_COMServerProcessReferences as ULONG;
    LeaveCriticalSection(&raw mut csRegisteredClassList);

    TRACE!();

    refs
}

/***********************************************************************
 *           CoReleaseServerProcess [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoReleaseServerProcess() -> ULONG {
    TRACE!();

    EnterCriticalSection(&raw mut csRegisteredClassList);

    s_COMServerProcessReferences -= 1;
    let refs = s_COMServerProcessReferences as ULONG;
    /* FIXME: if (!refs) COM_SuspendClassObjects(); */

    LeaveCriticalSection(&raw mut csRegisteredClassList);

    TRACE!();

    refs
}

/***********************************************************************
 *           CoIsHandlerConnected [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoIsHandlerConnected(pUnk: *mut IUnknown) -> BOOL {
    FIXME!("{:p}", pUnk);

    TRUE
}

/***********************************************************************
 *           CoAllowSetForegroundWindow [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoAllowSetForegroundWindow(
    pUnk: *mut IUnknown,
    pvReserved: PVOID,
) -> HRESULT {
    FIXME!("({:p}, {:p}): stub", pUnk, pvReserved);
    S_OK
}

/***********************************************************************
 *           CoQueryProxyBlanket [OLE32.@]
 *
 * Retrieves the security settings being used by a proxy.
 */
#[no_mangle]
pub unsafe extern "system" fn CoQueryProxyBlanket(
    pProxy: *mut IUnknown,
    pAuthnSvc: *mut DWORD,
    pAuthzSvc: *mut DWORD,
    ppServerPrincName: *mut LPWSTR,
    pAuthnLevel: *mut DWORD,
    pImpLevel: *mut DWORD,
    ppAuthInfo: *mut *mut c_void,
    pCapabilities: *mut DWORD,
) -> HRESULT {
    let mut pCliSec: *mut IClientSecurity = null_mut();

    TRACE!();

    let mut hr = IUnknown_QueryInterface(
        pProxy,
        &IID_IClientSecurity,
        &raw mut pCliSec as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        hr = ((*(*pCliSec).lpVtbl).QueryBlanket)(
            pCliSec,
            pProxy,
            pAuthnSvc,
            pAuthzSvc,
            ppServerPrincName,
            pAuthnLevel,
            pImpLevel,
            ppAuthInfo,
            pCapabilities,
        );
        ((*(*pCliSec).lpVtbl).Release)(pCliSec);
    }

    if FAILED(hr) {
        ERR!("-- failed with {:#010x}", hr as u32);
    }
    hr
}

/***********************************************************************
 *           CoSetProxyBlanket [OLE32.@]
 *
 * Sets the security settings for a proxy.
 */
#[no_mangle]
pub unsafe extern "system" fn CoSetProxyBlanket(
    pProxy: *mut IUnknown,
    AuthnSvc: DWORD,
    AuthzSvc: DWORD,
    pServerPrincName: LPWSTR,
    AuthnLevel: DWORD,
    ImpLevel: DWORD,
    pAuthInfo: PVOID,
    Capabilities: DWORD,
) -> HRESULT {
    let mut pCliSec: *mut IClientSecurity = null_mut();

    TRACE!();

    let mut hr = IUnknown_QueryInterface(
        pProxy,
        &IID_IClientSecurity,
        &raw mut pCliSec as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        hr = ((*(*pCliSec).lpVtbl).SetBlanket)(
            pCliSec,
            pProxy,
            AuthnSvc,
            AuthzSvc,
            pServerPrincName,
            AuthnLevel,
            ImpLevel,
            pAuthInfo,
            Capabilities,
        );
        ((*(*pCliSec).lpVtbl).Release)(pCliSec);
    }

    if FAILED(hr) {
        ERR!("-- failed with {:#010x}", hr as u32);
    }
    hr
}

/***********************************************************************
 *           CoCopyProxy [OLE32.@]
 *
 * Copies a proxy.
 */
#[no_mangle]
pub unsafe extern "system" fn CoCopyProxy(
    pProxy: *mut IUnknown,
    ppCopy: *mut *mut IUnknown,
) -> HRESULT {
    let mut pCliSec: *mut IClientSecurity = null_mut();

    TRACE!();

    let mut hr = IUnknown_QueryInterface(
        pProxy,
        &IID_IClientSecurity,
        &raw mut pCliSec as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        hr = ((*(*pCliSec).lpVtbl).CopyProxy)(pCliSec, pProxy, ppCopy);
        ((*(*pCliSec).lpVtbl).Release)(pCliSec);
    }

    if FAILED(hr) {
        ERR!("-- failed with {:#010x}", hr as u32);
    }
    hr
}

/***********************************************************************
 *           CoGetCallContext [OLE32.@]
 *
 * Gets the context of the currently executing server call in the current
 * thread.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetCallContext(riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    if (*info).call_state.is_null() {
        return RPC_E_CALL_COMPLETE;
    }

    IUnknown_QueryInterface((*info).call_state, riid, ppv)
}

/***********************************************************************
 *           CoSwitchCallContext [OLE32.@]
 *
 * Switches the context of the currently executing server call in the current
 * thread.
 */
#[no_mangle]
pub unsafe extern "system" fn CoSwitchCallContext(
    pObject: *mut IUnknown,
    ppOldObject: *mut *mut IUnknown,
) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppOldObject = (*info).call_state;
    (*info).call_state = pObject; /* CoSwitchCallContext does not addref nor release objects */

    S_OK
}

/***********************************************************************
 *           CoQueryClientBlanket [OLE32.@]
 *
 * Retrieves the authentication information about the client of the currently
 * executing server call in the current thread.
 */
#[no_mangle]
pub unsafe extern "system" fn CoQueryClientBlanket(
    pAuthnSvc: *mut DWORD,
    pAuthzSvc: *mut DWORD,
    pServerPrincName: *mut LPWSTR,
    pAuthnLevel: *mut DWORD,
    pImpLevel: *mut DWORD,
    pPrivs: *mut RPC_AUTHZ_HANDLE,
    pCapabilities: *mut DWORD,
) -> HRESULT {
    let mut pSrvSec: *mut IServerSecurity = null_mut();

    TRACE!();

    let mut hr = CoGetCallContext(&IID_IServerSecurity, &raw mut pSrvSec as *mut *mut c_void);
    if SUCCEEDED(hr) {
        hr = ((*(*pSrvSec).lpVtbl).QueryBlanket)(
            pSrvSec,
            pAuthnSvc,
            pAuthzSvc,
            pServerPrincName,
            pAuthnLevel,
            pImpLevel,
            pPrivs,
            pCapabilities,
        );
        ((*(*pSrvSec).lpVtbl).Release)(pSrvSec);
    }

    hr
}

/***********************************************************************
 *           CoImpersonateClient [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoImpersonateClient() -> HRESULT {
    let mut pSrvSec: *mut IServerSecurity = null_mut();

    TRACE!();

    let mut hr = CoGetCallContext(&IID_IServerSecurity, &raw mut pSrvSec as *mut *mut c_void);
    if SUCCEEDED(hr) {
        hr = ((*(*pSrvSec).lpVtbl).ImpersonateClient)(pSrvSec);
        ((*(*pSrvSec).lpVtbl).Release)(pSrvSec);
    }

    hr
}

/***********************************************************************
 *           CoRevertToSelf [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoRevertToSelf() -> HRESULT {
    let mut pSrvSec: *mut IServerSecurity = null_mut();

    TRACE!();

    let mut hr = CoGetCallContext(&IID_IServerSecurity, &raw mut pSrvSec as *mut *mut c_void);
    if SUCCEEDED(hr) {
        hr = ((*(*pSrvSec).lpVtbl).RevertToSelf)(pSrvSec);
        ((*(*pSrvSec).lpVtbl).Release)(pSrvSec);
    }

    hr
}

unsafe fn COM_PeekMessage(apt: *mut apartment, msg: *mut MSG) -> BOOL {
    /* first try to retrieve messages for incoming COM calls to the apartment window */
    if (!(*apt).win.is_null()
        && PeekMessageW(msg, (*apt).win, 0, 0, PM_REMOVE | PM_NOYIELD) != FALSE)
        /* next retrieve other messages necessary for the app to remain responsive */
        || PeekMessageW(msg, null_mut(), WM_DDE_FIRST, WM_DDE_LAST, PM_REMOVE | PM_NOYIELD)
            != FALSE
        || PeekMessageW(
            msg,
            null_mut(),
            0,
            0,
            PM_QS_PAINT | PM_QS_SENDMESSAGE | PM_REMOVE | PM_NOYIELD,
        ) != FALSE
    {
        TRUE
    } else {
        FALSE
    }
}

/***********************************************************************
 *           CoWaitForMultipleHandles [OLE32.@]
 *
 * Waits for one or more handles to become signaled.
 */
#[no_mangle]
pub unsafe extern "system" fn CoWaitForMultipleHandles(
    dwFlags: DWORD,
    dwTimeout: DWORD,
    cHandles: ULONG,
    pHandles: *mut HANDLE,
    lpdwindex: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let start_time = GetTickCount();
    let apt = COM_CurrentApt();
    let message_loop = !apt.is_null() && (*apt).multi_threaded == FALSE;
    let mut check_apc = (dwFlags & COWAIT_ALERTABLE) != 0;
    let mut post_quit = FALSE;
    let mut exit_code: UINT = 0;

    TRACE!();

    if lpdwindex.is_null() {
        return E_INVALIDARG;
    }

    *lpdwindex = 0;

    if pHandles.is_null() {
        return E_INVALIDARG;
    }

    if cHandles == 0 {
        return RPC_E_NO_SYNC;
    }

    loop {
        let now = GetTickCount();
        let mut res: DWORD;

        if now.wrapping_sub(start_time) > dwTimeout {
            hr = RPC_S_CALLPENDING;
            break;
        }

        if message_loop {
            let wait_flags: DWORD = (if (dwFlags & COWAIT_WAITALL) != 0 { MWMO_WAITALL } else { 0 })
                | (if (dwFlags & COWAIT_ALERTABLE) != 0 { MWMO_ALERTABLE } else { 0 });

            TRACE!();

            res = WAIT_TIMEOUT;

            if check_apc {
                res = WaitForMultipleObjectsEx(
                    cHandles,
                    pHandles,
                    ((dwFlags & COWAIT_WAITALL) != 0) as BOOL,
                    0,
                    TRUE,
                );
                check_apc = false;
            }

            if res == WAIT_TIMEOUT {
                res = MsgWaitForMultipleObjectsEx(
                    cHandles,
                    pHandles,
                    if dwTimeout == INFINITE {
                        INFINITE
                    } else {
                        start_time.wrapping_add(dwTimeout).wrapping_sub(now)
                    },
                    QS_SENDMESSAGE | QS_ALLPOSTMESSAGE | QS_PAINT,
                    wait_flags,
                );
            }

            if res == WAIT_OBJECT_0 + cHandles {
                /* messages available */
                let mut msg: MSG = core::mem::zeroed();
                let mut count: i32 = 0;

                /* call message filter */

                if !(*COM_CurrentApt()).filter.is_null() {
                    let pendingtype: DWORD = if (*COM_CurrentInfo()).pending_call_count_server != 0
                    {
                        PENDINGTYPE_NESTED
                    } else {
                        PENDINGTYPE_TOPLEVEL
                    };
                    let f = (*COM_CurrentApt()).filter;
                    let be_handled = ((*(*f).lpVtbl).MessagePending)(
                        f,
                        null_mut(), /* FIXME */
                        now.wrapping_sub(start_time),
                        pendingtype,
                    );
                    TRACE!();
                    match be_handled {
                        PENDINGMSG_CANCELCALL => {
                            WARN!();
                            hr = RPC_E_CALL_CANCELED;
                        }
                        /* PENDINGMSG_WAITNOPROCESS | PENDINGMSG_WAITDEFPROCESS:
                         * FIXME: MSDN is very vague about the difference
                         * between WAITNOPROCESS and WAITDEFPROCESS - there
                         * appears to be none, so it is possibly a left-over
                         * from the 16-bit world. */
                        _ => {}
                    }
                }

                if (*apt).win.is_null() {
                    /* If window is NULL on apartment, peek at messages so that it will not trigger
                     * MsgWaitForMultipleObjects next time. */
                    PeekMessageW(
                        null_mut(),
                        null_mut(),
                        0,
                        0,
                        PM_QS_POSTMESSAGE | PM_NOREMOVE | PM_NOYIELD,
                    );
                }
                /* some apps (e.g. Visio 2010) don't handle WM_PAINT properly and loop forever,
                 * so after processing 100 messages we go back to checking the wait handles */
                loop {
                    count += 1;
                    if count > 100 || COM_PeekMessage(apt, &raw mut msg) == FALSE {
                        break;
                    }
                    if msg.message == WM_QUIT {
                        TRACE!();
                        post_quit = TRUE;
                        exit_code = msg.wParam as UINT;
                    } else {
                        TRACE!();
                        TranslateMessage(&raw const msg);
                        DispatchMessageW(&raw const msg);
                    }
                }
                continue;
            }
        } else {
            TRACE!();

            res = WaitForMultipleObjectsEx(
                cHandles,
                pHandles,
                ((dwFlags & COWAIT_WAITALL) != 0) as BOOL,
                if dwTimeout == INFINITE {
                    INFINITE
                } else {
                    start_time.wrapping_add(dwTimeout).wrapping_sub(now)
                },
                ((dwFlags & COWAIT_ALERTABLE) != 0) as BOOL,
            );
        }

        match res {
            WAIT_TIMEOUT => {
                hr = RPC_S_CALLPENDING;
            }
            WAIT_FAILED => {
                hr = HRESULT_FROM_WIN32(GetLastError());
            }
            _ => {
                *lpdwindex = res;
            }
        }
        break;
    }
    if post_quit != FALSE {
        PostQuitMessage(exit_code as INT);
    }
    TRACE!();
    hr
}

/***********************************************************************
 *           CoGetObject [OLE32.@]
 *
 * Gets the object named by converting the name to a moniker and binding to it.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetObject(
    pszName: LPCWSTR,
    pBindOptions: *mut BIND_OPTS,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut pbc: *mut IBindCtx = null_mut();

    *ppv = null_mut();

    let mut hr = CreateBindCtx(0, &raw mut pbc);
    if SUCCEEDED(hr) {
        if !pBindOptions.is_null() {
            hr = ((*(*pbc).lpVtbl).SetBindOptions)(pbc, pBindOptions);
        }

        if SUCCEEDED(hr) {
            let mut chEaten: ULONG = 0;
            let mut pmk: *mut IMoniker = null_mut();

            hr = MkParseDisplayName(pbc, pszName, &raw mut chEaten, &raw mut pmk);
            if SUCCEEDED(hr) {
                hr = ((*(*pmk).lpVtbl).BindToObject)(pmk, pbc, null_mut(), riid, ppv);
                ((*(*pmk).lpVtbl).Release)(pmk);
            }
        }

        ((*(*pbc).lpVtbl).Release)(pbc);
    }
    hr
}

/***********************************************************************
 *           CoRegisterChannelHook [OLE32.@]
 *
 * Registers a process-wide hook that is called during ORPC calls.
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterChannelHook(
    guidExtension: REFGUID,
    pChannelHook: *mut IChannelHook,
) -> HRESULT {
    TRACE!();

    RPC_RegisterChannelHook(guidExtension, pChannelHook)
}

// ---------------------------------------------------------------------------
// Thread context object (IComThreadingInfo / IContextCallback / IObjContext)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn impl_from_IComThreadingInfo(iface: *mut IComThreadingInfo) -> *mut Context {
    (iface as *mut u8).sub(offset_of!(Context, IComThreadingInfo_iface)) as *mut Context
}

#[inline]
unsafe fn impl_from_IContextCallback(iface: *mut IContextCallback) -> *mut Context {
    (iface as *mut u8).sub(offset_of!(Context, IContextCallback_iface)) as *mut Context
}

#[inline]
unsafe fn impl_from_IObjContext(iface: *mut IObjContext) -> *mut Context {
    (iface as *mut u8).sub(offset_of!(Context, IObjContext_iface)) as *mut Context
}

unsafe fn Context_QueryInterface(
    iface: *mut Context,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    *ppv = null_mut();

    if guid_eq(riid, &IID_IComThreadingInfo) || guid_eq(riid, &IID_IUnknown) {
        *ppv = &raw mut (*iface).IComThreadingInfo_iface as LPVOID;
    } else if guid_eq(riid, &IID_IContextCallback) {
        *ppv = &raw mut (*iface).IContextCallback_iface as LPVOID;
    } else if guid_eq(riid, &IID_IObjContext) {
        *ppv = &raw mut (*iface).IObjContext_iface as LPVOID;
    }

    if !(*ppv).is_null() {
        IUnknown_AddRef(*ppv as *mut IUnknown);
        return S_OK;
    }

    FIXME!("interface not implemented {}", DbgGuid(riid));
    E_NOINTERFACE
}

unsafe fn Context_AddRef(This: *mut Context) -> ULONG {
    InterlockedIncrement(&raw mut (*This).refs) as ULONG
}

unsafe fn Context_Release(This: *mut Context) -> ULONG {
    /* Context instance is initially created with CoGetContextToken() with refcount set to 0,
       releasing context while refcount is at 0 destroys it. */
    if (*This).refs == 0 {
        HeapFree(GetProcessHeap(), 0, This as PVOID);
        return 0;
    }

    InterlockedDecrement(&raw mut (*This).refs) as ULONG
}

unsafe extern "system" fn Context_CTI_QueryInterface(
    iface: *mut IComThreadingInfo,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    Context_QueryInterface(impl_from_IComThreadingInfo(iface), riid, ppv)
}

unsafe extern "system" fn Context_CTI_AddRef(iface: *mut IComThreadingInfo) -> ULONG {
    Context_AddRef(impl_from_IComThreadingInfo(iface))
}

unsafe extern "system" fn Context_CTI_Release(iface: *mut IComThreadingInfo) -> ULONG {
    Context_Release(impl_from_IComThreadingInfo(iface))
}

unsafe extern "system" fn Context_CTI_GetCurrentApartmentType(
    iface: *mut IComThreadingInfo,
    apttype: *mut APTTYPE,
) -> HRESULT {
    let mut qualifier: APTTYPEQUALIFIER = 0;

    TRACE!();

    CoGetApartmentType(apttype, &raw mut qualifier)
}

unsafe extern "system" fn Context_CTI_GetCurrentThreadType(
    iface: *mut IComThreadingInfo,
    thdtype: *mut THDTYPE,
) -> HRESULT {
    let mut qualifier: APTTYPEQUALIFIER = 0;
    let mut apttype: APTTYPE = 0;

    let hr = CoGetApartmentType(&raw mut apttype, &raw mut qualifier);
    if FAILED(hr) {
        return hr;
    }

    TRACE!();

    match apttype {
        APTTYPE_STA | APTTYPE_MAINSTA => *thdtype = THDTYPE_PROCESSMESSAGES,
        _ => *thdtype = THDTYPE_BLOCKMESSAGES,
    }
    S_OK
}

unsafe extern "system" fn Context_CTI_GetCurrentLogicalThreadId(
    iface: *mut IComThreadingInfo,
    logical_thread_id: *mut GUID,
) -> HRESULT {
    TRACE!();
    CoGetCurrentLogicalThreadId(logical_thread_id)
}

unsafe extern "system" fn Context_CTI_SetCurrentLogicalThreadId(
    iface: *mut IComThreadingInfo,
    logical_thread_id: REFGUID,
) -> HRESULT {
    FIXME!("({}): stub", DbgGuid(logical_thread_id));
    E_NOTIMPL
}

static Context_Threading_Vtbl: IComThreadingInfoVtbl = IComThreadingInfoVtbl {
    QueryInterface: Context_CTI_QueryInterface,
    AddRef: Context_CTI_AddRef,
    Release: Context_CTI_Release,
    GetCurrentApartmentType: Context_CTI_GetCurrentApartmentType,
    GetCurrentThreadType: Context_CTI_GetCurrentThreadType,
    GetCurrentLogicalThreadId: Context_CTI_GetCurrentLogicalThreadId,
    SetCurrentLogicalThreadId: Context_CTI_SetCurrentLogicalThreadId,
};

unsafe extern "system" fn Context_CC_QueryInterface(
    iface: *mut IContextCallback,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    Context_QueryInterface(impl_from_IContextCallback(iface), riid, ppv)
}

unsafe extern "system" fn Context_CC_AddRef(iface: *mut IContextCallback) -> ULONG {
    Context_AddRef(impl_from_IContextCallback(iface))
}

unsafe extern "system" fn Context_CC_Release(iface: *mut IContextCallback) -> ULONG {
    Context_Release(impl_from_IContextCallback(iface))
}

unsafe extern "system" fn Context_CC_ContextCallback(
    iface: *mut IContextCallback,
    pCallback: PFNCONTEXTCALL,
    param: *mut ComCallData,
    riid: REFIID,
    method: i32,
    punk: *mut IUnknown,
) -> HRESULT {
    let This = impl_from_IContextCallback(iface);

    FIXME!(
        "({:p}/{:p})->({:p}, {:p}, {}, {}, {:p})",
        This,
        iface,
        pCallback.map_or(null::<c_void>(), |f| f as *const c_void),
        param,
        DbgGuid(riid),
        method,
        punk
    );
    E_NOTIMPL
}

static Context_Callback_Vtbl: IContextCallbackVtbl = IContextCallbackVtbl {
    QueryInterface: Context_CC_QueryInterface,
    AddRef: Context_CC_AddRef,
    Release: Context_CC_Release,
    ContextCallback: Context_CC_ContextCallback,
};

unsafe extern "system" fn Context_OC_QueryInterface(
    iface: *mut IObjContext,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    Context_QueryInterface(impl_from_IObjContext(iface), riid, ppv)
}

unsafe extern "system" fn Context_OC_AddRef(iface: *mut IObjContext) -> ULONG {
    Context_AddRef(impl_from_IObjContext(iface))
}

unsafe extern "system" fn Context_OC_Release(iface: *mut IObjContext) -> ULONG {
    Context_Release(impl_from_IObjContext(iface))
}

unsafe extern "system" fn Context_OC_SetProperty(
    iface: *mut IObjContext,
    propid: REFGUID,
    flags: ULONG,
    punk: *mut IUnknown,
) -> HRESULT {
    let This = impl_from_IObjContext(iface);

    FIXME!("({:p}/{:p})->({}, {:x}, {:p})", This, iface, DbgGuid(propid), flags, punk);
    E_NOTIMPL
}

unsafe extern "system" fn Context_OC_RemoveProperty(
    iface: *mut IObjContext,
    propid: REFGUID,
) -> HRESULT {
    let This = impl_from_IObjContext(iface);

    FIXME!("({:p}/{:p})->({})", This, iface, DbgGuid(propid));
    E_NOTIMPL
}

unsafe extern "system" fn Context_OC_GetProperty(
    iface: *mut IObjContext,
    propid: REFGUID,
    flags: *mut ULONG,
    punk: *mut *mut IUnknown,
) -> HRESULT {
    let This = impl_from_IObjContext(iface);

    FIXME!("({:p}/{:p})->({}, {:p}, {:p})", This, iface, DbgGuid(propid), flags, punk);
    E_NOTIMPL
}

unsafe extern "system" fn Context_OC_EnumContextProps(
    iface: *mut IObjContext,
    props: *mut *mut IEnumContextProps,
) -> HRESULT {
    let This = impl_from_IObjContext(iface);

    FIXME!("({:p}/{:p})->({:p})", This, iface, props);
    E_NOTIMPL
}

unsafe extern "system" fn Context_OC_Reserved1(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved2(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved3(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved4(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved5(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved6(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

unsafe extern "system" fn Context_OC_Reserved7(iface: *mut IObjContext) {
    let This = impl_from_IObjContext(iface);
    FIXME!("({:p}/{:p})", This, iface);
}

static Context_Object_Vtbl: IObjContextVtbl = IObjContextVtbl {
    QueryInterface: Context_OC_QueryInterface,
    AddRef: Context_OC_AddRef,
    Release: Context_OC_Release,
    SetProperty: Context_OC_SetProperty,
    RemoveProperty: Context_OC_RemoveProperty,
    GetProperty: Context_OC_GetProperty,
    EnumContextProps: Context_OC_EnumContextProps,
    Reserved1: Context_OC_Reserved1,
    Reserved2: Context_OC_Reserved2,
    Reserved3: Context_OC_Reserved3,
    Reserved4: Context_OC_Reserved4,
    Reserved5: Context_OC_Reserved5,
    Reserved6: Context_OC_Reserved6,
    Reserved7: Context_OC_Reserved7,
};

/***********************************************************************
 *           CoGetObjectContext [OLE32.@]
 *
 * Retrieves an object associated with the current context (i.e. apartment).
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetObjectContext(riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    let mut context: *mut IObjContext = null_mut();

    TRACE!();

    *ppv = null_mut();
    let hr = CoGetContextToken(&raw mut context as *mut ULONG_PTR);
    if FAILED(hr) {
        return hr;
    }

    ((*(*context).lpVtbl).QueryInterface)(context, riid, ppv)
}

/***********************************************************************
 *           CoGetContextToken [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetContextToken(token: *mut ULONG_PTR) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("apartment not initialised");
        return CO_E_NOTINITIALIZED;
    }
    apartment_release(apt);

    if token.is_null() {
        return E_POINTER;
    }

    if (*info).context_token.is_null() {
        let context = HeapAlloc(GetProcessHeap(), 0, size_of::<Context>()) as *mut Context;
        if context.is_null() {
            return E_OUTOFMEMORY;
        }

        (*context).IComThreadingInfo_iface.lpVtbl = &Context_Threading_Vtbl;
        (*context).IContextCallback_iface.lpVtbl = &Context_Callback_Vtbl;
        (*context).IObjContext_iface.lpVtbl = &Context_Object_Vtbl;
        /* Context token does not take a reference, it's always zero until the
           interface is explicitly requested with CoGetObjectContext(). */
        (*context).refs = 0;

        (*info).context_token = &raw mut (*context).IObjContext_iface;
    }

    *token = (*info).context_token as ULONG_PTR;
    TRACE!();

    S_OK
}

/***********************************************************************
 *           CoGetDefaultContext [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetDefaultContext(
    type_: APTTYPE,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    FIXME!("{} {} {:p} stub", type_, DbgGuid(riid), ppv);
    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "C" fn Handler_DllGetClassObject(
    rclsid: REFCLSID,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let wszInprocHandler32 = wide!("InprocHandler32");
    let mut hkey: HKEY = null_mut();

    let hres = COM_OpenKeyForCLSID(rclsid, wszInprocHandler32.as_ptr(), KEY_READ, &raw mut hkey);
    if SUCCEEDED(hres) {
        let mut regdata: class_reg_data = core::mem::zeroed();
        let mut dllpath: [WCHAR; MAX_PATH + 1] = [0; MAX_PATH + 1];

        regdata.u.hkey = hkey;
        regdata.hkey = TRUE;

        if get_object_dll_path(&regdata, dllpath.as_mut_ptr(), (MAX_PATH + 1) as DWORD) != FALSE {
            let wszOle32 = wide!("ole32.dll");
            if wcsicmp(dllpath.as_ptr(), wszOle32.as_ptr()) == 0 {
                RegCloseKey(hkey);
                return HandlerCF_Create(rclsid, riid, ppv);
            }
        } else {
            WARN!();
        }
        RegCloseKey(hkey);
    }

    CLASS_E_CLASSNOTAVAILABLE
}

/***********************************************************************
 *           CoGetApartmentType [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetApartmentType(
    type_: *mut APTTYPE,
    qualifier: *mut APTTYPEQUALIFIER,
) -> HRESULT {
    let info = COM_CurrentInfo();

    TRACE!();

    if type_.is_null() || qualifier.is_null() {
        return E_INVALIDARG;
    }

    if info.is_null() {
        return E_OUTOFMEMORY;
    }

    if (*info).apt.is_null() {
        *type_ = APTTYPE_CURRENT;
    } else if (*(*info).apt).multi_threaded != FALSE {
        *type_ = APTTYPE_MTA;
    } else if (*(*info).apt).main != FALSE {
        *type_ = APTTYPE_MAINSTA;
    } else {
        *type_ = APTTYPE_STA;
    }

    *qualifier = APTTYPEQUALIFIER_NONE;

    if (*info).apt.is_null() {
        let apt = apartment_find_mta();
        if !apt.is_null() {
            apartment_release(apt);
            *type_ = APTTYPE_MTA;
            *qualifier = APTTYPEQUALIFIER_IMPLICIT_MTA;
            // wine-10: an implicit MTA is a success, not CO_E_NOTINITIALIZED.
            return S_OK;
        }
    }

    if !(*info).apt.is_null() {
        S_OK
    } else {
        CO_E_NOTINITIALIZED
    }
}

/// combase apartment.c `struct mta_cookie`: the opaque CO_MTA_USAGE_COOKIE.
#[repr(C)]
struct mta_cookie {
    entry: list,
}

/***********************************************************************
 *           CoIncrementMTAUsage [OLE32.@]  (wine-10 combase)
 */
#[no_mangle]
pub unsafe extern "system" fn CoIncrementMTAUsage(cookie: *mut PVOID) -> HRESULT {
    TRACE!();

    *cookie = null_mut();

    let mta_cookie = heap_alloc(size_of::<mta_cookie>()) as *mut mta_cookie;
    if mta_cookie.is_null() {
        return E_OUTOFMEMORY;
    }

    EnterCriticalSection(&raw mut csApartment);

    if !MTA.is_null() {
        apartment_addref(MTA);
    } else {
        MTA = apartment_construct(COINIT_MULTITHREADED);
    }
    list_add_head(&raw mut (*MTA).usage_cookies, &raw mut (*mta_cookie).entry);

    LeaveCriticalSection(&raw mut csApartment);

    *cookie = mta_cookie as PVOID;

    S_OK
}

/***********************************************************************
 *           CoDecrementMTAUsage [OLE32.@]  (wine-10 combase)
 */
#[no_mangle]
pub unsafe extern "system" fn CoDecrementMTAUsage(cookie: PVOID) -> HRESULT {
    TRACE!();

    EnterCriticalSection(&raw mut csApartment);

    if !MTA.is_null() {
        let head = &raw mut (*MTA).usage_cookies;
        let mut cur = (*head).next;
        while cur != head {
            if cur as PVOID == cookie {
                // `entry` is the first field, so the node IS the cookie.
                list_remove(cur);
                heap_free(cur as PVOID);
                // csApartment is a recursive critical section; apartment_release
                // re-enters it, exactly as combase's does under apt_cs.
                apartment_release(MTA);
                break;
            }
            cur = (*cur).next;
        }
    }

    LeaveCriticalSection(&raw mut csApartment);

    S_OK
}

/***********************************************************************
 *           CoCreateInstanceFromApp [OLE32.@]  (wine-10 combase)
 */
#[no_mangle]
pub unsafe extern "system" fn CoCreateInstanceFromApp(
    rclsid: REFCLSID,
    outer: *mut IUnknown,
    cls_context: DWORD,
    server_info: PVOID,
    count: ULONG,
    results: *mut MULTI_QI,
) -> HRESULT {
    TRACE!();

    CoCreateInstanceEx(
        rclsid,
        outer,
        cls_context | CLSCTX_APPCONTAINER,
        server_info as *mut COSERVERINFO,
        count,
        results,
    )
}

/***********************************************************************
 *           CoDisableCallCancellation [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoDisableCallCancellation(reserved: PVOID) -> HRESULT {
    FIXME!("({:p}): stub", reserved);

    E_NOTIMPL
}

/***********************************************************************
 *           CoEnableCallCancellation [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoEnableCallCancellation(reserved: PVOID) -> HRESULT {
    FIXME!("({:p}): stub", reserved);

    E_NOTIMPL
}

/***********************************************************************
 *           CoRegisterSurrogate [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterSurrogate(surrogate: *mut ISurrogate) -> HRESULT {
    FIXME!("({:p}): stub", surrogate);

    E_NOTIMPL
}

/***********************************************************************
 *           CoRegisterSurrogateEx [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoRegisterSurrogateEx(guid: REFGUID, reserved: PVOID) -> HRESULT {
    FIXME!("({} {:p}): stub", DbgGuid(guid), reserved);

    E_NOTIMPL
}

// ---------------------------------------------------------------------------
// GlobalOptions object (CLSID_GlobalOptions)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn impl_from_IGlobalOptions(iface: *mut IGlobalOptions) -> *mut GlobalOptions {
    (iface as *mut u8).sub(offset_of!(GlobalOptions, IGlobalOptions_iface)) as *mut GlobalOptions
}

unsafe extern "system" fn GlobalOptions_QueryInterface(
    iface: *mut IGlobalOptions,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IGlobalOptions(iface);

    TRACE!();

    if guid_eq(&IID_IGlobalOptions, riid) || guid_eq(&IID_IUnknown, riid) {
        *ppv = iface as *mut c_void;
    } else {
        *ppv = null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    S_OK
}

unsafe extern "system" fn GlobalOptions_AddRef(iface: *mut IGlobalOptions) -> ULONG {
    let This = impl_from_IGlobalOptions(iface);
    let ref_ = InterlockedIncrement(&raw mut (*This).ref_);

    TRACE!();

    ref_ as ULONG
}

unsafe extern "system" fn GlobalOptions_Release(iface: *mut IGlobalOptions) -> ULONG {
    let This = impl_from_IGlobalOptions(iface);
    let ref_ = InterlockedDecrement(&raw mut (*This).ref_);

    TRACE!();

    if ref_ == 0 {
        heap_free(This as PVOID);
    }

    ref_ as ULONG
}

unsafe extern "system" fn GlobalOptions_Set(
    iface: *mut IGlobalOptions,
    property: GLOBALOPT_PROPERTIES,
    value: ULONG_PTR,
) -> HRESULT {
    let This = impl_from_IGlobalOptions(iface);
    FIXME!("({:p})->({} {:x})", This, property, value);
    S_OK
}

unsafe extern "system" fn GlobalOptions_Query(
    iface: *mut IGlobalOptions,
    property: GLOBALOPT_PROPERTIES,
    value: *mut ULONG_PTR,
) -> HRESULT {
    let This = impl_from_IGlobalOptions(iface);
    FIXME!("({:p})->({} {:p})", This, property, value);
    E_NOTIMPL
}

static GlobalOptionsVtbl: IGlobalOptionsVtbl = IGlobalOptionsVtbl {
    QueryInterface: GlobalOptions_QueryInterface,
    AddRef: GlobalOptions_AddRef,
    Release: GlobalOptions_Release,
    Set: GlobalOptions_Set,
    Query: GlobalOptions_Query,
};

#[no_mangle]
pub unsafe extern "system" fn GlobalOptions_CreateInstance(
    iface: *mut IClassFactory,
    outer: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    TRACE!();

    if !outer.is_null() {
        return E_INVALIDARG;
    }

    let global_options = heap_alloc(size_of::<GlobalOptions>()) as *mut GlobalOptions;
    if global_options.is_null() {
        return E_OUTOFMEMORY;
    }
    (*global_options).IGlobalOptions_iface.lpVtbl = &GlobalOptionsVtbl;
    (*global_options).ref_ = 1;

    let go = &raw mut (*global_options).IGlobalOptions_iface;
    let hres = ((*(*go).lpVtbl).QueryInterface)(go, riid, ppv);
    ((*(*go).lpVtbl).Release)(go);
    hres
}

/***********************************************************************
 *		DllMain (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    TRACE!();

    match fdwReason {
        DLL_PROCESS_ATTACH => {
            // Rust-translation addition: initialize the CSes/list heads that
            // were C static initializers (see MANIFEST.md).
            compobj_process_attach();
            hProxyDll = hinstDLL;
        }

        DLL_PROCESS_DETACH => {
            // wine-10: the lazily created OLE clipboard lives until here.
            crate::ring::clipboard::clipbrd_destroy();
            if !reserved.is_null() {
                return TRUE;
            }
            release_std_git();
            if apt_win_class != 0 {
                UnregisterClassW(MAKEINTATOM(apt_win_class), hProxyDll);
            }
            RPC_UnregisterAllChannelHooks();
            COMPOBJ_DllList_Free();
            DeleteCriticalSection(&raw mut csRegisteredClassList);
            DeleteCriticalSection(&raw mut csApartment);
        }

        DLL_THREAD_DETACH => {
            COM_TlsDestroy();
        }

        _ => {}
    }
    TRUE
}

/***********************************************************************
 *		DllRegisterServer (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    OLE32_DllRegisterServer()
}

/***********************************************************************
 *		DllUnregisterServer (OLE32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    OLE32_DllUnregisterServer()
}
