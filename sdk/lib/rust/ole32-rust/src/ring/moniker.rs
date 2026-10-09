use crate::prelude::*;
use core::ffi::{c_int, c_void};

// ---------------------------------------------------------------------------
// irot.h — widl-generated from wine/irot.idl (see build-<arch>/dll/win32/ole32/irot.h).
//
// The Irot RPC *client stubs* are generated into irot_c.c, which is compiled into
// ole32.dll, so these are real link-time symbols in our own image. widl spells them
// `__cdecl`, so `extern "C"` is exact on every arch — this is one of the rare
// non-WINAPI surfaces (contrast crate::win32 / crate::user32, where `extern "C"`
// would be an i386 stdcall-decoration bug).
//
// MIDL_user_allocate / MIDL_user_free go the OTHER way: rpcndr.h declares them
// `__RPC_USER`, which rpc.h defines as __stdcall → they must be `extern "system"`.
// ---------------------------------------------------------------------------

/// irot.idl: `typedef handle_t IrotHandle;`
pub type IrotHandle = *mut c_void;
/// irot.idl: `typedef [context_handle] void *IrotContextHandle;`
pub type IrotContextHandle = *mut c_void;
/// irot.idl: `typedef DWORD IrotCookie;`
pub type IrotCookie = DWORD;

/// irot.h: `struct tagMonikerComparisonData { ULONG ulCntData; BYTE abData[1]; }`
///
/// `abData` is a CONFORMANT array — the struct is always over-allocated to
/// `offset_of!(.., abData) + ulCntData` bytes. Never index `abData` with a runtime
/// index: Rust bounds-checks `[u8; 1]` and would panic. Always go through
/// `abData.as_ptr()` / `.as_mut_ptr()`.
#[repr(C)]
pub struct MonikerComparisonData {
    pub ulCntData: ULONG,
    pub abData: [BYTE; 1],
}

/// irot.h: `struct tagInterfaceData { ULONG ulCntData; BYTE abData[1]; }` — same
/// layout as objidl's `MInterfacePointer`, which is what moniker.c's Register path
/// spells the object blob as. Conformant array; see MonikerComparisonData above.
#[repr(C)]
pub struct InterfaceData {
    pub ulCntData: ULONG,
    pub abData: [BYTE; 1],
}
pub type PInterfaceData = *mut InterfaceData;

/// irot.h: `struct tagInterfaceList { ULONG size; PInterfaceData interfaces[1]; }`
/// Conformant array — same indexing rule as above.
#[repr(C)]
pub struct InterfaceList {
    pub size: ULONG,
    pub interfaces: [PInterfaceData; 1],
}
pub type PInterfaceList = *mut InterfaceList;

extern "C" {
    fn IrotRegister(
        h: IrotHandle,
        moniker_data: *const MonikerComparisonData,
        object: *const InterfaceData,
        moniker: *const InterfaceData,
        time: *const FILETIME,
        grfFlags: DWORD,
        cookie: *mut IrotCookie,
        ctxt_handle: *mut IrotContextHandle,
    ) -> HRESULT;
    fn IrotRevoke(
        h: IrotHandle,
        cookie: IrotCookie,
        ctxt_handle: *mut IrotContextHandle,
        object: *mut PInterfaceData,
        moniker: *mut PInterfaceData,
    ) -> HRESULT;
    fn IrotIsRunning(h: IrotHandle, moniker_data: *const MonikerComparisonData) -> HRESULT;
    fn IrotGetObject(
        h: IrotHandle,
        moniker_data: *const MonikerComparisonData,
        obj: *mut PInterfaceData,
        cookie: *mut IrotCookie,
    ) -> HRESULT;
    fn IrotNoteChangeTime(h: IrotHandle, cookie: IrotCookie, time: *const FILETIME) -> HRESULT;
    fn IrotGetTimeOfLastChange(
        h: IrotHandle,
        moniker_data: *const MonikerComparisonData,
        time: *mut FILETIME,
    ) -> HRESULT;
    fn IrotEnumRunning(h: IrotHandle, list: *mut PInterfaceList) -> HRESULT;
}

/// rpcnterr.h: `#define RPC_S_OK ERROR_SUCCESS`
const RPC_S_OK: LONG = 0;

/// winsvc.h — the subset of the SCM surface start_rpcss() needs. Not in the shared
/// surface (crate::user32 declares the *calls*, but not this out-struct).
#[repr(C)]
struct SERVICE_STATUS_PROCESS {
    dwServiceType: DWORD,
    dwCurrentState: DWORD,
    dwControlsAccepted: DWORD,
    dwWin32ExitCode: DWORD,
    dwServiceSpecificExitCode: DWORD,
    dwCheckPoint: DWORD,
    dwWaitHint: DWORD,
    dwProcessId: DWORD,
    dwServiceFlags: DWORD,
}
const SERVICE_START: DWORD = 0x0010;
const SERVICE_QUERY_STATUS: DWORD = 0x0004;
const SC_STATUS_PROCESS_INFO: c_int = 0;
const SERVICE_RUNNING: DWORD = 4;
const SERVICE_START_PENDING: DWORD = 2;

/// winnt.h EXCEPTION_RECORD / EXCEPTION_POINTERS, trimmed to the prefix rpc_filter
/// reads (`eptr->ExceptionRecord->ExceptionCode`). Both fields are FIRST in their
/// struct, so this prefix is layout-exact for that access. Not in com-sys (which
/// covers COM, not winnt).
#[repr(C)]
pub struct EXCEPTION_RECORD_PREFIX {
    pub ExceptionCode: DWORD,
}
#[repr(C)]
pub struct EXCEPTION_POINTERS {
    pub ExceptionRecord: *mut EXCEPTION_RECORD_PREFIX,
    pub ContextRecord: *mut c_void,
}

/// compobj_private.h DEBUG_SET_CRITSEC_NAME / DEBUG_CLEAR_CRITSEC_NAME — they only
/// stash a name string in the critsec's debug info; no locking effect. crate::win32
/// keeps CRITICAL_SECTION.DebugInfo opaque, so these are no-ops here (same treatment
/// as ring::hglobalstream).
#[inline]
unsafe fn debug_set_critsec_name(_cs: *mut CRITICAL_SECTION, _name: &str) {}
#[inline]
unsafe fn debug_clear_critsec_name(_cs: *mut CRITICAL_SECTION) {}

/// winbase.h InterlockedCompareExchangePointer. crate::winemacros only carries the
/// 32-bit forms; this is the pointer-width one get_irot_handle needs. Returns the
/// PREVIOUS value, like the C intrinsic.
#[inline]
unsafe fn InterlockedCompareExchangePointer(
    dest: *mut *mut c_void,
    exchange: *mut c_void,
    comparand: *mut c_void,
) -> *mut c_void {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let a = &*(dest as *const AtomicUsize);
    match a.compare_exchange(
        comparand as usize,
        exchange as usize,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(prev) => prev as *mut c_void,
        Err(prev) => prev as *mut c_void,
    }
}

/// see MSDN docs for IROTData::GetComparisonData, which states what this constant is
pub const MAX_COMPARISON_DATA: ULONG = 2048;

/// The RPC exception filter the C installs around every Irot call
/// (`__TRY { IrotXxx(..) } __EXCEPT(rpc_filter) { hr = HRESULT_FROM_WIN32(GetExceptionCode()); }`).
///
/// We have no SEH in this crate, so the Irot calls below are UNGUARDED: an RPC
/// exception (e.g. rpcss not reachable) propagates instead of being converted to an
/// HRESULT. That is a known faithfulness gap, kept visible here rather than papered
/// over — the filter itself is translated and ready for the day the ring gets a
/// try/except facility.
#[allow(dead_code)]
unsafe fn rpc_filter(eptr: *mut EXCEPTION_POINTERS) -> LONG {
    I_RpcExceptionFilter((*(*eptr).ExceptionRecord).ExceptionCode) as LONG
}

/* define the structure of the running object table elements */
#[repr(C)]
pub struct rot_entry {
    pub entry: list,
    /// marshaled running object
    pub object: *mut InterfaceData,
    /// moniker comparison data that identifies this object
    pub moniker_data: *mut MonikerComparisonData,
    /// cookie identifying this object
    pub cookie: DWORD,
    pub last_modified: FILETIME,
    pub ctxt_handle: IrotContextHandle,
}

/* define the RunningObjectTableImpl structure */
#[repr(C)]
pub struct RunningObjectTableImpl {
    pub IRunningObjectTable_iface: IRunningObjectTable,
    pub ref_: LONG,
    /// list of ROT entries
    pub rot: list,
    pub lock: CRITICAL_SECTION,
}

static mut runningObjectTableInstance: *mut RunningObjectTableImpl = core::ptr::null_mut();
static mut irot_handle: IrotHandle = core::ptr::null_mut();

/* define the EnumMonikerImpl structure */
#[repr(C)]
pub struct EnumMonikerImpl {
    pub IEnumMoniker_iface: IEnumMoniker,
    pub ref_: LONG,
    pub moniker_list: *mut InterfaceList,
    pub pos: ULONG,
}

unsafe fn impl_from_IRunningObjectTable(
    iface: *mut IRunningObjectTable,
) -> *mut RunningObjectTableImpl {
    CONTAINING_RECORD!(iface, RunningObjectTableImpl, IRunningObjectTable_iface)
}

unsafe fn impl_from_IEnumMoniker(iface: *mut IEnumMoniker) -> *mut EnumMonikerImpl {
    CONTAINING_RECORD!(iface, EnumMonikerImpl, IEnumMoniker_iface)
}

unsafe fn get_irot_handle() -> IrotHandle {
    if irot_handle.is_null() {
        let mut status: LONG;
        let mut binding: *mut u16 = core::ptr::null_mut();
        let mut new_handle: IrotHandle = core::ptr::null_mut();
        /// irot.idl: `#define IROT_PROTSEQ {'n','c','a','l','r','p','c',0}`
        const NCALRPC: [u16; 8] = [
            b'n' as u16,
            b'c' as u16,
            b'a' as u16,
            b'l' as u16,
            b'r' as u16,
            b'p' as u16,
            b'c' as u16,
            0,
        ];
        /// irot.idl: `#define IROT_ENDPOINT {'i','r','o','t',0}`
        const ENDPOINT: [u16; 5] = [b'i' as u16, b'r' as u16, b'o' as u16, b't' as u16, 0];

        status = RpcStringBindingComposeW(
            core::ptr::null(),
            NCALRPC.as_ptr(),
            core::ptr::null(),
            ENDPOINT.as_ptr(),
            core::ptr::null(),
            &mut binding,
        );
        if status == RPC_S_OK {
            status = RpcBindingFromStringBindingW(binding, &mut new_handle);
            RpcStringFreeW(&mut binding);
        }
        if status != RPC_S_OK {
            return core::ptr::null_mut();
        }
        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(irot_handle),
            new_handle,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            /* another thread beat us to it */
            RpcBindingFree(&mut new_handle);
        }
    }
    irot_handle
}

pub(crate) unsafe fn start_rpcss() -> BOOL {
    const RPCSSW: [u16; 6] = [
        b'R' as u16,
        b'p' as u16,
        b'c' as u16,
        b'S' as u16,
        b's' as u16,
        0,
    ];
    let scm: SC_HANDLE;
    let service: SC_HANDLE;
    let mut status: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut ret: BOOL = 0;

    TRACE!(b"\n\0".as_ptr());

    scm = OpenSCManagerW(core::ptr::null(), core::ptr::null(), 0);
    if scm.is_null() {
        ERR!(b"failed to open service manager\n\0".as_ptr());
        return 0;
    }
    service = OpenServiceW(scm, RPCSSW.as_ptr(), SERVICE_START | SERVICE_QUERY_STATUS);
    if service.is_null() {
        ERR!(b"failed to open RpcSs service\n\0".as_ptr());
        CloseServiceHandle(scm);
        return 0;
    }
    if StartServiceW(service, 0, core::ptr::null()) != 0
        || GetLastError() == ERROR_SERVICE_ALREADY_RUNNING
    {
        let start_time: u64 = GetTickCount64();
        loop {
            let mut dummy: DWORD = 0;

            if QueryServiceStatusEx(
                service,
                SC_STATUS_PROCESS_INFO,
                core::ptr::addr_of_mut!(status) as *mut u8,
                core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                &mut dummy,
            ) == 0
            {
                break;
            }
            if status.dwCurrentState == SERVICE_RUNNING {
                ret = 1;
                break;
            }
            if GetTickCount64() - start_time > 30000 {
                break;
            }
            Sleep(100);

            if status.dwCurrentState != SERVICE_START_PENDING {
                break;
            }
        }

        if status.dwCurrentState != SERVICE_RUNNING {
            WARN!(
                b"RpcSs failed to start %u\n\0".as_ptr(),
                status.dwCurrentState
            );
        }
    } else {
        ERR!(b"failed to start RpcSs service\n\0".as_ptr());
    }

    CloseServiceHandle(service);
    CloseServiceHandle(scm);
    ret
}

unsafe fn create_stream_on_mip_ro(mip: *const InterfaceData, stream: *mut *mut IStream) -> HRESULT {
    let hglobal: HGLOBAL = GlobalAlloc(0, (*mip).ulCntData as SIZE_T);
    let pv: LPVOID = GlobalLock(hglobal);
    memcpy(
        pv,
        (*mip).abData.as_ptr() as *const c_void,
        (*mip).ulCntData as usize,
    );
    GlobalUnlock(hglobal);
    CreateStreamOnHGlobal(hglobal, 1, stream)
}

unsafe fn rot_entry_delete(rot_entry: *mut rot_entry) {
    if (*rot_entry).cookie != 0 {
        let mut object: *mut InterfaceData = core::ptr::null_mut();
        let mut moniker: *mut InterfaceData = core::ptr::null_mut();

        // C: __TRY { .. } __EXCEPT(rpc_filter) { } — see the note on rpc_filter.
        IrotRevoke(
            get_irot_handle(),
            (*rot_entry).cookie,
            core::ptr::addr_of_mut!((*rot_entry).ctxt_handle),
            &mut object,
            &mut moniker,
        );

        MIDL_user_free(object as LPVOID);
        if !moniker.is_null() {
            let mut stream: *mut IStream = core::ptr::null_mut();
            let hr = create_stream_on_mip_ro(moniker, &mut stream);
            if hr == S_OK {
                CoReleaseMarshalData(stream);
                IStream_Release(stream);
            }
        }
        MIDL_user_free(moniker as LPVOID);
    }
    if !(*rot_entry).object.is_null() {
        let mut stream: *mut IStream = core::ptr::null_mut();
        let hr = create_stream_on_mip_ro((*rot_entry).object, &mut stream);
        if hr == S_OK {
            CoReleaseMarshalData(stream);
            IStream_Release(stream);
        }
    }
    HeapFree(GetProcessHeap(), 0, (*rot_entry).object as LPVOID);
    HeapFree(GetProcessHeap(), 0, (*rot_entry).moniker_data as LPVOID);
    HeapFree(GetProcessHeap(), 0, rot_entry as LPVOID);
}

/// moniker_data must be freed with HeapFree when no longer in use
unsafe fn get_moniker_comparison_data(
    pMoniker: *mut IMoniker,
    moniker_data: *mut *mut MonikerComparisonData,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pROTData: *mut IROTData = core::ptr::null_mut();
    hr = IMoniker_QueryInterface(
        pMoniker,
        core::ptr::addr_of!(IID_IROTData),
        &mut pROTData as *mut *mut IROTData as *mut *mut c_void,
    );
    if SUCCEEDED(hr) {
        let mut size: ULONG = MAX_COMPARISON_DATA;
        *moniker_data = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::offset_of!(MonikerComparisonData, abData) + size as usize) as SIZE_T,
        ) as *mut MonikerComparisonData;
        if (*moniker_data).is_null() {
            IROTData_Release(pROTData);
            return E_OUTOFMEMORY;
        }
        hr = IROTData_GetComparisonData(
            pROTData,
            (**moniker_data).abData.as_mut_ptr(),
            size,
            &mut size,
        );
        IROTData_Release(pROTData);
        if hr != S_OK {
            ERR!(
                b"Failed to copy comparison data into buffer, hr = 0x%08x\n\0".as_ptr(),
                hr
            );
            HeapFree(GetProcessHeap(), 0, *moniker_data as LPVOID);
            // wine-10: the caller's cleanup (rot_entry_delete) frees it again.
            *moniker_data = core::ptr::null_mut();
            return hr;
        }
        (**moniker_data).ulCntData = size;
    } else {
        let mut pbc: *mut IBindCtx = core::ptr::null_mut();
        let mut pszDisplayName: LPOLESTR = core::ptr::null_mut();
        let mut clsid: CLSID = core::mem::zeroed();
        let len: c_int;

        TRACE!(b"generating comparison data from display name\n\0".as_ptr());

        hr = CreateBindCtx(0, &mut pbc);
        if FAILED(hr) {
            return hr;
        }
        hr = IMoniker_GetDisplayName(pMoniker, pbc, core::ptr::null_mut(), &mut pszDisplayName);
        IBindCtx_Release(pbc);
        if FAILED(hr) {
            return hr;
        }
        hr = IMoniker_GetClassID(pMoniker, &mut clsid);
        if FAILED(hr) {
            CoTaskMemFree(pszDisplayName as LPVOID);
            return hr;
        }

        len = lstrlenW(pszDisplayName);
        let cb_data: usize =
            core::mem::size_of::<CLSID>() + (len as usize + 1) * core::mem::size_of::<u16>();
        *moniker_data = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::offset_of!(MonikerComparisonData, abData) + cb_data) as SIZE_T,
        ) as *mut MonikerComparisonData;
        if (*moniker_data).is_null() {
            CoTaskMemFree(pszDisplayName as LPVOID);
            return E_OUTOFMEMORY;
        }
        (**moniker_data).ulCntData = cb_data as ULONG;

        memcpy(
            (**moniker_data).abData.as_mut_ptr() as *mut c_void,
            core::ptr::addr_of!(clsid) as *const c_void,
            core::mem::size_of::<CLSID>(),
        );
        memcpy(
            (**moniker_data)
                .abData
                .as_mut_ptr()
                .add(core::mem::size_of::<CLSID>()) as *mut c_void,
            pszDisplayName as *const c_void,
            (len as usize + 1) * core::mem::size_of::<u16>(),
        );
        CoTaskMemFree(pszDisplayName as LPVOID);
    }
    S_OK
}

unsafe fn reduce_moniker(
    pmk: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    let mut pbc = pbc;
    let mut pbcNew: *mut IBindCtx = core::ptr::null_mut();
    let hr: HRESULT;
    if pbc.is_null() {
        let hr2 = CreateBindCtx(0, &mut pbcNew);
        if FAILED(hr2) {
            return hr2;
        }
        pbc = pbcNew;
    }
    hr = IMoniker_Reduce(
        pmk,
        pbc,
        MKRREDUCE_ALL,
        core::ptr::null_mut(),
        pmkReduced,
    );
    if FAILED(hr) {
        ERR!(b"reducing moniker failed with error 0x%08x\n\0".as_ptr(), hr);
    }
    if !pbcNew.is_null() {
        IBindCtx_Release(pbcNew);
    }
    hr
}

/***********************************************************************
 *        RunningObjectTable_QueryInterface
 */
stdmethod! {
fn RunningObjectTableImpl_QueryInterface(
    iface: *mut IRunningObjectTable,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* validate arguments */

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IRunningObjectTable), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IRunningObjectTable_iface) as *mut c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IRunningObjectTable_AddRef(iface);

    S_OK
}
}

/***********************************************************************
 *        RunningObjectTable_AddRef
 */
stdmethod! {
fn RunningObjectTableImpl_AddRef(iface: *mut IRunningObjectTable) -> ULONG {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

/***********************************************************************
 *        RunningObjectTable_Destroy
 */
unsafe fn RunningObjectTableImpl_Destroy() -> HRESULT {
    TRACE!(b"()\n\0".as_ptr());

    if runningObjectTableInstance.is_null() {
        return E_INVALIDARG;
    }

    /* free the ROT table memory */
    LIST_FOR_EACH_ENTRY_SAFE!(
        rot_entry,
        core::ptr::addr_of_mut!((*runningObjectTableInstance).rot),
        rot_entry,
        entry,
        {
            list_remove(core::ptr::addr_of_mut!((*rot_entry).entry));
            rot_entry_delete(rot_entry);
        }
    );

    debug_clear_critsec_name(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));
    DeleteCriticalSection(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));

    /* free the ROT structure memory */
    HeapFree(GetProcessHeap(), 0, runningObjectTableInstance as LPVOID);
    runningObjectTableInstance = core::ptr::null_mut();

    let mut old_handle: IrotHandle = irot_handle;
    irot_handle = core::ptr::null_mut();
    if !old_handle.is_null() {
        RpcBindingFree(&mut old_handle);
    }

    S_OK
}

/***********************************************************************
 *        RunningObjectTable_Release
 */
stdmethod! {
fn RunningObjectTableImpl_Release(iface: *mut IRunningObjectTable) -> ULONG {
    let This = impl_from_IRunningObjectTable(iface);
    let ref_: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* uninitialize ROT structure if there are no more references to it */
    if ref_ == 0 {
        LIST_FOR_EACH_ENTRY_SAFE!(
            rot_entry,
            core::ptr::addr_of_mut!((*This).rot),
            rot_entry,
            entry,
            {
                list_remove(core::ptr::addr_of_mut!((*rot_entry).entry));
                rot_entry_delete(rot_entry);
            }
        );
        /*  RunningObjectTable data structure will be not destroyed here ! the destruction will be done only
         *  when RunningObjectTableImpl_UnInitialize function is called
         */
    }

    ref_
}
}

/***********************************************************************
 *        RunningObjectTable_Register
 *
 * PARAMS
 * grfFlags       [in] Registration options
 * punkObject     [in] the object being registered
 * pmkObjectName  [in] the moniker of the object being registered
 * pdwRegister    [out] the value identifying the registration
 */
stdmethod! {
fn RunningObjectTableImpl_Register(
    iface: *mut IRunningObjectTable,
    grfFlags: DWORD,
    punkObject: *mut IUnknown,
    pmkObjectName: *mut IMoniker,
    pdwRegister: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut pmkObjectName = pmkObjectName;
    let rot_entry: *mut rot_entry;
    let mut hr: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let mshlflags: DWORD;
    let mut pbc: *mut IBindCtx = core::ptr::null_mut();
    let mut moniker: *mut InterfaceData = core::ptr::null_mut();

    TRACE!(
        b"(%p,%d,%p,%p,%p)\n\0".as_ptr(),
        This,
        grfFlags,
        punkObject,
        pmkObjectName,
        pdwRegister
    );

    if (grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT)) != 0 {
        ERR!(
            b"Invalid grfFlags: 0x%08x\n\0".as_ptr(),
            grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT)
        );
        return E_INVALIDARG;
    }

    if punkObject.is_null() || pmkObjectName.is_null() || pdwRegister.is_null() {
        return E_INVALIDARG;
    }

    rot_entry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<rot_entry>() as SIZE_T,
    ) as *mut rot_entry;
    if rot_entry.is_null() {
        return E_OUTOFMEMORY;
    }

    /* marshal object */
    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut pStream);
    if hr != S_OK {
        rot_entry_delete(rot_entry);
        return hr;
    }
    mshlflags = if (grfFlags & ROTFLAGS_REGISTRATIONKEEPSALIVE) != 0 {
        MSHLFLAGS_TABLESTRONG
    } else {
        MSHLFLAGS_TABLEWEAK
    };
    hr = CoMarshalInterface(
        pStream,
        core::ptr::addr_of!(IID_IUnknown),
        punkObject,
        MSHCTX_LOCAL | MSHCTX_NOSHAREDMEM,
        core::ptr::null_mut(),
        mshlflags,
    );
    /* FIXME: a cleaner way would be to create an IStream class that writes
     * directly to an MInterfacePointer */
    if hr == S_OK {
        let mut hglobal: HGLOBAL = core::ptr::null_mut();
        hr = GetHGlobalFromStream(pStream, &mut hglobal);
        if hr == S_OK {
            let size: SIZE_T = GlobalSize(hglobal);
            let pv: LPVOID = GlobalLock(hglobal);
            (*rot_entry).object = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::offset_of!(InterfaceData, abData) as SIZE_T + size,
            ) as *mut InterfaceData;
            (*(*rot_entry).object).ulCntData = size as ULONG;
            memcpy(
                (*(*rot_entry).object).abData.as_mut_ptr() as *mut c_void,
                pv,
                size as usize,
            );
            GlobalUnlock(hglobal);
        }
    }
    IStream_Release(pStream);
    if hr != S_OK {
        rot_entry_delete(rot_entry);
        return hr;
    }

    hr = CreateBindCtx(0, &mut pbc);
    if FAILED(hr) {
        rot_entry_delete(rot_entry);
        return hr;
    }

    hr = reduce_moniker(pmkObjectName, pbc, &mut pmkObjectName);
    if FAILED(hr) {
        rot_entry_delete(rot_entry);
        IBindCtx_Release(pbc);
        return hr;
    }

    hr = IMoniker_GetTimeOfLastChange(
        pmkObjectName,
        pbc,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*rot_entry).last_modified),
    );
    IBindCtx_Release(pbc);
    if FAILED(hr) {
        CoFileTimeNow(core::ptr::addr_of_mut!((*rot_entry).last_modified));
    }

    hr = get_moniker_comparison_data(
        pmkObjectName,
        core::ptr::addr_of_mut!((*rot_entry).moniker_data),
    );
    if hr != S_OK {
        rot_entry_delete(rot_entry);
        IMoniker_Release(pmkObjectName);
        return hr;
    }

    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut pStream);
    if hr != S_OK {
        rot_entry_delete(rot_entry);
        IMoniker_Release(pmkObjectName);
        return hr;
    }
    /* marshal moniker */
    hr = CoMarshalInterface(
        pStream,
        core::ptr::addr_of!(IID_IMoniker),
        pmkObjectName as *mut IUnknown,
        MSHCTX_LOCAL | MSHCTX_NOSHAREDMEM,
        core::ptr::null_mut(),
        MSHLFLAGS_TABLESTRONG,
    );
    /* FIXME: a cleaner way would be to create an IStream class that writes
     * directly to an MInterfacePointer */
    if hr == S_OK {
        let mut hglobal: HGLOBAL = core::ptr::null_mut();
        hr = GetHGlobalFromStream(pStream, &mut hglobal);
        if hr == S_OK {
            let size: SIZE_T = GlobalSize(hglobal);
            let pv: LPVOID = GlobalLock(hglobal);
            moniker = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::offset_of!(InterfaceData, abData) as SIZE_T + size,
            ) as *mut InterfaceData;
            (*moniker).ulCntData = size as ULONG;
            memcpy((*moniker).abData.as_mut_ptr() as *mut c_void, pv, size as usize);
            GlobalUnlock(hglobal);
        }
    }
    IStream_Release(pStream);
    IMoniker_Release(pmkObjectName);
    if hr != S_OK {
        HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
        rot_entry_delete(rot_entry);
        return hr;
    }

    loop {
        // C: __TRY { .. } __EXCEPT(rpc_filter) { hr = HRESULT_FROM_WIN32(GetExceptionCode()); }
        hr = IrotRegister(
            get_irot_handle(),
            (*rot_entry).moniker_data,
            (*rot_entry).object,
            moniker,
            core::ptr::addr_of!((*rot_entry).last_modified),
            grfFlags,
            core::ptr::addr_of_mut!((*rot_entry).cookie),
            core::ptr::addr_of_mut!((*rot_entry).ctxt_handle),
        );
        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
            continue;
        }
        break;
    }
    HeapFree(GetProcessHeap(), 0, moniker as LPVOID);
    if FAILED(hr) {
        rot_entry_delete(rot_entry);
        return hr;
    }

    /* gives a registration identifier to the registered object*/
    *pdwRegister = (*rot_entry).cookie;

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    list_add_tail(
        core::ptr::addr_of_mut!((*This).rot),
        core::ptr::addr_of_mut!((*rot_entry).entry),
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    hr
}
}

/***********************************************************************
 *        RunningObjectTable_Revoke
 *
 * PARAMS
 *  dwRegister [in] Value identifying registration to be revoked
 */
stdmethod! {
fn RunningObjectTableImpl_Revoke(iface: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE!(b"(%p,%d)\n\0".as_ptr(), This, dwRegister);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    // NOTE: the C uses the PLAIN (non-SAFE) LIST_FOR_EACH_ENTRY here even though the
    // body frees the element — that is sound only because it RETURNS immediately
    // after, never touching the freed node's links. Keep it that way.
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*rot_entry).cookie == dwRegister {
                list_remove(core::ptr::addr_of_mut!((*rot_entry).entry));
                LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

                rot_entry_delete(rot_entry);
                return S_OK;
            }
        }
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    E_INVALIDARG
}
}

/***********************************************************************
 *        RunningObjectTable_IsRunning
 *
 * PARAMS
 *  pmkObjectName [in]  moniker of the object whose status is desired
 */
stdmethod! {
fn RunningObjectTableImpl_IsRunning(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut pmkObjectName = pmkObjectName;
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), This, pmkObjectName);

    hr = reduce_moniker(pmkObjectName, core::ptr::null_mut(), &mut pmkObjectName);
    if FAILED(hr) {
        return hr;
    }
    hr = get_moniker_comparison_data(pmkObjectName, &mut moniker_data);
    IMoniker_Release(pmkObjectName);
    if hr != S_OK {
        return hr;
    }

    hr = S_FALSE;
    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
                && memcmp(
                    (*moniker_data).abData.as_ptr() as *const c_void,
                    (*(*rot_entry).moniker_data).abData.as_ptr() as *const c_void,
                    (*moniker_data).ulCntData as usize,
                ) == 0
            {
                hr = S_OK;
                break;
            }
        }
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    if hr == S_FALSE {
        loop {
            // C: __TRY/__EXCEPT(rpc_filter) — see the note on rpc_filter.
            hr = IrotIsRunning(get_irot_handle(), moniker_data);
            if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
                continue;
            }
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);

    hr
}
}

/***********************************************************************
 *        RunningObjectTable_GetObject
 *
 * PARAMS
 * pmkObjectName [in] Pointer to the moniker on the object
 * ppunkObject   [out] variable that receives the IUnknown interface pointer
 */
stdmethod! {
fn RunningObjectTableImpl_GetObject(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
    ppunkObject: *mut *mut IUnknown,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut pmkObjectName = pmkObjectName;
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();
    let mut object: *mut InterfaceData = core::ptr::null_mut();
    let mut cookie: IrotCookie = 0;
    let mut hr: HRESULT;

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), This, pmkObjectName, ppunkObject);

    if ppunkObject.is_null() {
        return E_POINTER;
    }

    *ppunkObject = core::ptr::null_mut();

    hr = reduce_moniker(pmkObjectName, core::ptr::null_mut(), &mut pmkObjectName);
    if FAILED(hr) {
        return hr;
    }
    hr = get_moniker_comparison_data(pmkObjectName, &mut moniker_data);
    IMoniker_Release(pmkObjectName);
    if hr != S_OK {
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
                && memcmp(
                    (*moniker_data).abData.as_ptr() as *const c_void,
                    (*(*rot_entry).moniker_data).abData.as_ptr() as *const c_void,
                    (*moniker_data).ulCntData as usize,
                ) == 0
            {
                let mut pStream: *mut IStream = core::ptr::null_mut();
                hr = create_stream_on_mip_ro((*rot_entry).object, &mut pStream);
                // wine-10: drop the ROT lock BEFORE unmarshalling. Unmarshal can
                // call out of apartment, and a server that touches this ROT from
                // there would deadlock against us.
                LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

                if hr == S_OK {
                    hr = CoUnmarshalInterface(
                        pStream,
                        core::ptr::addr_of!(IID_IUnknown),
                        ppunkObject as *mut LPVOID,
                    );
                    IStream_Release(pStream);
                }

                HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);

                return hr;
            }
        }
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    TRACE!(b"moniker unavailable locally, calling SCM\n\0".as_ptr());

    loop {
        // C: __TRY/__EXCEPT(rpc_filter) — see the note on rpc_filter.
        hr = IrotGetObject(get_irot_handle(), moniker_data, &mut object, &mut cookie);
        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
            continue;
        }
        break;
    }

    if SUCCEEDED(hr) {
        let mut pStream: *mut IStream = core::ptr::null_mut();
        hr = create_stream_on_mip_ro(object, &mut pStream);
        if hr == S_OK {
            hr = CoUnmarshalInterface(
                pStream,
                core::ptr::addr_of!(IID_IUnknown),
                ppunkObject as *mut LPVOID,
            );
            IStream_Release(pStream);
        }
    } else {
        WARN!(
            b"Moniker unavailable, IrotGetObject returned 0x%08x\n\0".as_ptr(),
            hr
        );
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);

    hr
}
}

/***********************************************************************
 *        RunningObjectTable_NoteChangeTime
 *
 * PARAMS
 *  dwRegister [in] Value identifying registration being updated
 *  pfiletime  [in] Pointer to structure containing object's last change time
 */
stdmethod! {
fn RunningObjectTableImpl_NoteChangeTime(
    iface: *mut IRunningObjectTable,
    dwRegister: DWORD,
    pfiletime: *mut FILETIME,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut hr: HRESULT = E_INVALIDARG;

    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), This, dwRegister, pfiletime);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*rot_entry).cookie == dwRegister {
                (*rot_entry).last_modified = *pfiletime;
                LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

                loop {
                    // C: __TRY/__EXCEPT(rpc_filter) — see the note on rpc_filter.
                    hr = IrotNoteChangeTime(get_irot_handle(), dwRegister, pfiletime);
                    if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
                        continue;
                    }
                    break;
                }

                /* C: goto done — skips the second LeaveCriticalSection (already left) */
                TRACE!(b"-- 0x08%x\n\0".as_ptr(), hr);
                return hr;
            }
        }
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    TRACE!(b"-- 0x08%x\n\0".as_ptr(), hr);
    hr
}
}

/***********************************************************************
 *        RunningObjectTable_GetTimeOfLastChange
 *
 * PARAMS
 *  pmkObjectName  [in]  moniker of the object whose status is desired
 *  pfiletime      [out] structure that receives object's last change time
 */
stdmethod! {
fn RunningObjectTableImpl_GetTimeOfLastChange(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
    pfiletime: *mut FILETIME,
) -> HRESULT {
    let mut hr: HRESULT;
    let This = impl_from_IRunningObjectTable(iface);
    let mut pmkObjectName = pmkObjectName;
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), This, pmkObjectName, pfiletime);

    if pmkObjectName.is_null() || pfiletime.is_null() {
        return E_INVALIDARG;
    }

    hr = reduce_moniker(pmkObjectName, core::ptr::null_mut(), &mut pmkObjectName);
    if FAILED(hr) {
        return hr;
    }
    hr = get_moniker_comparison_data(pmkObjectName, &mut moniker_data);
    IMoniker_Release(pmkObjectName);
    if hr != S_OK {
        return hr;
    }

    hr = MK_E_UNAVAILABLE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
                && memcmp(
                    (*moniker_data).abData.as_ptr() as *const c_void,
                    (*(*rot_entry).moniker_data).abData.as_ptr() as *const c_void,
                    (*moniker_data).ulCntData as usize,
                ) == 0
            {
                *pfiletime = (*rot_entry).last_modified;
                hr = S_OK;
                break;
            }
        }
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    if hr != S_OK {
        loop {
            // C: __TRY/__EXCEPT(rpc_filter) — see the note on rpc_filter.
            hr = IrotGetTimeOfLastChange(get_irot_handle(), moniker_data, pfiletime);
            if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
                continue;
            }
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);

    TRACE!(b"-- 0x%08x\n\0".as_ptr(), hr);
    hr
}
}

/***********************************************************************
 *        RunningObjectTable_EnumRunning
 *
 * PARAMS
 *  ppenumMoniker  [out]  receives the IEnumMoniker interface pointer
 */
stdmethod! {
fn RunningObjectTableImpl_EnumRunning(
    iface: *mut IRunningObjectTable,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let _This = impl_from_IRunningObjectTable(iface);
    let mut interface_list: *mut InterfaceList = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p, %p)\n\0".as_ptr(), This, ppenumMoniker);

    *ppenumMoniker = core::ptr::null_mut();

    loop {
        // C: __TRY/__EXCEPT(rpc_filter) — see the note on rpc_filter.
        hr = IrotEnumRunning(get_irot_handle(), &mut interface_list);
        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) && start_rpcss() != 0 {
            continue;
        }
        break;
    }

    if SUCCEEDED(hr) {
        hr = EnumMonikerImpl_CreateEnumROTMoniker(interface_list, 0, ppenumMoniker);
    }

    hr
}
}

/* Virtual function table for the IRunningObjectTable class. */
static VT_RunningObjectTableImpl: IRunningObjectTableVtbl = IRunningObjectTableVtbl {
    QueryInterface: Some(RunningObjectTableImpl_QueryInterface),
    AddRef: Some(RunningObjectTableImpl_AddRef),
    Release: Some(RunningObjectTableImpl_Release),
    Register: Some(RunningObjectTableImpl_Register),
    Revoke: Some(RunningObjectTableImpl_Revoke),
    IsRunning: Some(RunningObjectTableImpl_IsRunning),
    GetObjectA: Some(RunningObjectTableImpl_GetObject),
    NoteChangeTime: Some(RunningObjectTableImpl_NoteChangeTime),
    GetTimeOfLastChange: Some(RunningObjectTableImpl_GetTimeOfLastChange),
    EnumRunning: Some(RunningObjectTableImpl_EnumRunning),
};

/***********************************************************************
 *        RunningObjectTable_Initialize
 */
#[no_mangle]
pub unsafe extern "system" fn RunningObjectTableImpl_Initialize() -> HRESULT {
    TRACE!(b"\n\0".as_ptr());

    /* create the unique instance of the RunningObjectTableImpl structure */
    runningObjectTableInstance = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<RunningObjectTableImpl>() as SIZE_T,
    ) as *mut RunningObjectTableImpl;

    if runningObjectTableInstance.is_null() {
        return E_OUTOFMEMORY;
    }

    /* initialize the virtual table function */
    (*runningObjectTableInstance)
        .IRunningObjectTable_iface
        .lpVtbl = &VT_RunningObjectTableImpl as *const IRunningObjectTableVtbl
        as *mut IRunningObjectTableVtbl;

    /* the initial reference is set to "1" so that it isn't destroyed after its
     * first use until the process is destroyed, as the running object table is
     * a process-wide cache of a global table */
    (*runningObjectTableInstance).ref_ = 1;

    list_init(core::ptr::addr_of_mut!((*runningObjectTableInstance).rot));
    InitializeCriticalSection(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));
    debug_set_critsec_name(
        core::ptr::addr_of_mut!((*runningObjectTableInstance).lock),
        "RunningObjectTableImpl.lock",
    );

    S_OK
}

/***********************************************************************
 *        RunningObjectTable_UnInitialize
 */
#[no_mangle]
pub unsafe extern "system" fn RunningObjectTableImpl_UnInitialize() -> HRESULT {
    TRACE!(b"\n\0".as_ptr());

    if runningObjectTableInstance.is_null() {
        return E_POINTER;
    }

    RunningObjectTableImpl_Release(core::ptr::addr_of_mut!(
        (*runningObjectTableInstance).IRunningObjectTable_iface
    ));

    RunningObjectTableImpl_Destroy();

    S_OK
}

/***********************************************************************
 *           GetRunningObjectTable (OLE32.@)
 *
 * Retrieves the global running object table.
 */
#[no_mangle]
pub unsafe extern "system" fn GetRunningObjectTable(
    reserved: DWORD,
    pprot: *mut LPRUNNINGOBJECTTABLE,
) -> HRESULT {
    let riid: IID = IID_IRunningObjectTable;
    let res: HRESULT;

    TRACE!(b"()\n\0".as_ptr());

    if reserved != 0 {
        return E_UNEXPECTED;
    }

    if runningObjectTableInstance.is_null() {
        return CO_E_NOTINITIALIZED;
    }

    res = IRunningObjectTable_QueryInterface(
        core::ptr::addr_of_mut!((*runningObjectTableInstance).IRunningObjectTable_iface),
        &riid,
        pprot as *mut *mut c_void,
    );

    res
}

unsafe fn get_moniker_for_progid_display_name(
    pbc: *mut IBindCtx,
    szDisplayName: LPCOLESTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut hr: HRESULT;
    let progid: LPWSTR;
    let mut start: LPCWSTR = szDisplayName;
    let mut end: LPCWSTR;
    let len: c_int;
    let mut class_moniker: *mut IMoniker = core::ptr::null_mut();

    if *start == b'@' as u16 {
        start = start.offset(1);
    }

    /* find end delimiter */
    end = start;
    while *end != 0 {
        if *end == b':' as u16 {
            break;
        }
        end = end.offset(1);
    }

    len = end.offset_from(start) as c_int;

    /* must start with '@' or have a ':' somewhere and mustn't be one character
     * long (since that looks like an absolute path) */
    if ((start == szDisplayName) && (*end == 0)) || (len <= 1) {
        return MK_E_SYNTAX;
    }

    progid = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if !progid.is_null() {
        memcpy(
            progid as *mut c_void,
            start as *const c_void,
            len as usize * core::mem::size_of::<u16>(),
        );
        *progid.offset(len as isize) = 0;
    }
    hr = CLSIDFromProgID(progid, &mut clsid);
    HeapFree(GetProcessHeap(), 0, progid as LPVOID);
    if FAILED(hr) {
        return MK_E_SYNTAX;
    }

    hr = CreateClassMoniker(&clsid, &mut class_moniker);
    if SUCCEEDED(hr) {
        let mut pdn: *mut IParseDisplayName = core::ptr::null_mut();
        hr = IMoniker_BindToObject(
            class_moniker,
            pbc,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IParseDisplayName),
            &mut pdn as *mut *mut IParseDisplayName as *mut *mut c_void,
        );
        /* fallback to using IClassFactory to get IParseDisplayName -
         * adsldp.dll depends on this */
        if FAILED(hr) {
            let mut pcf: *mut IClassFactory = core::ptr::null_mut();
            hr = IMoniker_BindToObject(
                class_moniker,
                pbc,
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IClassFactory),
                &mut pcf as *mut *mut IClassFactory as *mut *mut c_void,
            );
            if SUCCEEDED(hr) {
                hr = IClassFactory_CreateInstance(
                    pcf,
                    core::ptr::null_mut(),
                    core::ptr::addr_of!(IID_IParseDisplayName),
                    &mut pdn as *mut *mut IParseDisplayName as *mut *mut c_void,
                );
                IClassFactory_Release(pcf);
            }
        }
        IMoniker_Release(class_moniker);
        if SUCCEEDED(hr) {
            hr = IParseDisplayName_ParseDisplayName(
                pdn,
                pbc,
                szDisplayName as LPOLESTR,
                pchEaten,
                ppmk,
            );
            IParseDisplayName_Release(pdn);
        }
    }
    hr
}

/******************************************************************************
 *              MkParseDisplayName        [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn MkParseDisplayName(
    pbc: *mut IBindCtx,
    szDisplayName: LPCOLESTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let mut szDisplayName = szDisplayName;
    let mut hr: HRESULT;
    const WSZ_CLSID_COLON: [u16; 6] = [
        b'c' as u16,
        b'l' as u16,
        b's' as u16,
        b'i' as u16,
        b'd' as u16,
        b':' as u16,
    ];
    let mut moniker: *mut IMoniker = core::ptr::null_mut();
    let mut chEaten: DWORD = 0;

    TRACE!(
        b"(%p, %s, %p, %p)\n\0".as_ptr(),
        pbc,
        debugstr_w(szDisplayName),
        pchEaten,
        ppmk
    );

    if pbc.is_null() || IsValidInterface(pbc as LPUNKNOWN) == 0 {
        return E_INVALIDARG;
    }

    if szDisplayName.is_null() || *szDisplayName == 0 {
        return E_INVALIDARG;
    }

    if pchEaten.is_null() || ppmk.is_null() {
        return E_INVALIDARG;
    }

    *pchEaten = 0;
    *ppmk = core::ptr::null_mut();

    if _wcsnicmp(szDisplayName, WSZ_CLSID_COLON.as_ptr(), WSZ_CLSID_COLON.len()) == 0 {
        hr = ClassMoniker_CreateFromDisplayName(
            pbc,
            szDisplayName,
            &mut chEaten,
            &mut moniker,
        );
        if FAILED(hr) && (hr != MK_E_SYNTAX) {
            return hr;
        }
    } else {
        hr = get_moniker_for_progid_display_name(pbc, szDisplayName, &mut chEaten, &mut moniker);
        if FAILED(hr) && (hr != MK_E_SYNTAX) {
            return hr;
        }
    }

    if FAILED(hr) {
        hr = FileMoniker_CreateFromDisplayName(pbc, szDisplayName, &mut chEaten, &mut moniker);
        if FAILED(hr) && (hr != MK_E_SYNTAX) {
            return hr;
        }
    }

    if SUCCEEDED(hr) {
        loop {
            let mut next_moniker: *mut IMoniker = core::ptr::null_mut();
            *pchEaten += chEaten;
            szDisplayName = szDisplayName.add(chEaten as usize);
            if *szDisplayName == 0 {
                *ppmk = moniker;
                return S_OK;
            }
            chEaten = 0;
            hr = IMoniker_ParseDisplayName(
                moniker,
                pbc,
                core::ptr::null_mut(),
                szDisplayName as LPOLESTR,
                &mut chEaten,
                &mut next_moniker,
            );
            IMoniker_Release(moniker);
            if FAILED(hr) {
                *pchEaten = 0;
                break;
            }
            moniker = next_moniker;
        }
    }

    hr
}

/***********************************************************************
 *        GetClassFile (OLE32.@)
 *
 * Retrieves the class ID associated with the given filename.
 */
#[no_mangle]
pub unsafe extern "system" fn GetClassFile(
    filePathName: LPCOLESTR,
    pclsid: *mut CLSID,
) -> HRESULT {
    let mut pstg: *mut IStorage = core::ptr::null_mut();
    let mut res: HRESULT;
    let nbElm: c_int;
    let length: c_int;
    let mut i: c_int;
    let mut sizeProgId: LONG = 0;
    let mut ret: LONG;
    let mut pathDec: *mut LPWSTR = core::ptr::null_mut();
    let absFile: LPOLESTR;
    let progId: LPOLESTR;
    let mut extension: LPWSTR;
    const BKSLASHW: [u16; 2] = [b'\\' as u16, 0];
    const DOTW: [u16; 2] = [b'.' as u16, 0];

    TRACE!(b"%s, %p\n\0".as_ptr(), debugstr_w(filePathName), pclsid);

    /* if the file contain a storage object the return the CLSID written by IStorage_SetClass method*/
    if StgIsStorageFile(filePathName) == S_OK {
        res = StgOpenStorage(
            filePathName,
            core::ptr::null_mut(),
            STGM_READ | STGM_SHARE_DENY_WRITE,
            core::ptr::null_mut(),
            0,
            &mut pstg,
        );

        if SUCCEEDED(res) {
            res = ReadClassStg(pstg, pclsid);
            IStorage_Release(pstg);
        }

        return res;
    }

    /* if the above strategies fail then search for the extension key in the registry */

    /* get the last element (absolute file) in the path name */
    nbElm = FileMonikerImpl_DecomposePath(filePathName, &mut pathDec);
    absFile = *pathDec.offset((nbElm - 1) as isize);

    /* failed if the path represents a directory and not an absolute file name*/
    if wcscmp(absFile, BKSLASHW.as_ptr()) == 0 {
        CoTaskMemFree(pathDec as LPVOID);
        return MK_E_INVALIDEXTENSION;
    }

    /* get the extension of the file */
    extension = core::ptr::null_mut();
    length = lstrlenW(absFile);
    i = length - 1;
    while i >= 0 {
        extension = absFile.offset(i as isize);
        if *extension == b'.' as u16 {
            break;
        }
        i -= 1;
    }

    if extension.is_null() || wcscmp(extension, DOTW.as_ptr()) == 0 {
        CoTaskMemFree(pathDec as LPVOID);
        return MK_E_INVALIDEXTENSION;
    }

    ret = RegQueryValueW(
        HKEY_CLASSES_ROOT,
        extension,
        core::ptr::null_mut(),
        &mut sizeProgId,
    );
    if ret == 0 {
        /* get the progId associated to the extension */
        progId = CoTaskMemAlloc(sizeProgId as SIZE_T) as LPOLESTR;
        ret = RegQueryValueW(HKEY_CLASSES_ROOT, extension, progId, &mut sizeProgId);
        if ret == 0 {
            /* return the clsid associated to the progId */
            res = CLSIDFromProgID(progId, pclsid);
        } else {
            res = HRESULT_FROM_WIN32(ret as u32);
        }
        CoTaskMemFree(progId as LPVOID);
    } else {
        res = HRESULT_FROM_WIN32(ret as u32);
    }

    i = 0;
    while !(*pathDec.offset(i as isize)).is_null() {
        CoTaskMemFree(*pathDec.offset(i as isize) as LPVOID);
        i += 1;
    }
    CoTaskMemFree(pathDec as LPVOID);

    if res != S_OK {
        MK_E_INVALIDEXTENSION
    } else {
        res
    }
}

/***********************************************************************
 *        EnumMoniker_QueryInterface
 */
stdmethod! {
fn EnumMonikerImpl_QueryInterface(
    iface: *mut IEnumMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    /* validate arguments */
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IEnumMoniker), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumMoniker_iface) as *mut c_void;
    } else {
        return E_NOINTERFACE;
    }

    IEnumMoniker_AddRef(iface);
    S_OK
}
}

/***********************************************************************
 *        EnumMoniker_AddRef
 */
stdmethod! {
fn EnumMonikerImpl_AddRef(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

/***********************************************************************
 *        EnumMoniker_release
 */
stdmethod! {
fn EnumMonikerImpl_Release(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);
    let ref_: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* uninitialize ROT structure if there are no more references to it */
    if ref_ == 0 {
        let mut i: ULONG;

        TRACE!(b"(%p) Deleting\n\0".as_ptr(), This);

        i = 0;
        while i < (*(*This).moniker_list).size {
            HeapFree(
                GetProcessHeap(),
                0,
                *(*(*This).moniker_list).interfaces.as_ptr().add(i as usize) as LPVOID,
            );
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, (*This).moniker_list as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

/***********************************************************************
 *        EnumMoniker_Next
 */
stdmethod! {
fn EnumMonikerImpl_Next(
    iface: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut i: ULONG;
    let This = impl_from_IEnumMoniker(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!(
        b"(%p) TabCurrentPos %d Tablastindx %d\n\0".as_ptr(),
        This,
        (*This).pos,
        (*(*This).moniker_list).size
    );

    /* retrieve the requested number of moniker from the current position */
    i = 0;
    while ((*This).pos < (*(*This).moniker_list).size) && (i < celt) {
        let mut stream: *mut IStream = core::ptr::null_mut();
        let mip = *(*(*This).moniker_list)
            .interfaces
            .as_ptr()
            .add((*This).pos as usize);
        (*This).pos += 1;
        hr = create_stream_on_mip_ro(mip, &mut stream);
        if hr != S_OK {
            break;
        }
        hr = CoUnmarshalInterface(
            stream,
            core::ptr::addr_of!(IID_IMoniker),
            rgelt.add(i as usize) as *mut LPVOID,
        );
        IStream_Release(stream);
        if hr != S_OK {
            break;
        }
        i += 1;
    }

    if !pceltFetched.is_null() {
        *pceltFetched = i;
    }

    if hr != S_OK {
        return hr;
    }

    if i == celt {
        S_OK
    } else {
        S_FALSE
    }
}
}

/***********************************************************************
 *        EnumMoniker_Skip
 */
stdmethod! {
fn EnumMonikerImpl_Skip(iface: *mut IEnumMoniker, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    if (*This).pos + celt >= (*(*This).moniker_list).size {
        return S_FALSE;
    }

    (*This).pos += celt;

    S_OK
}
}

/***********************************************************************
 *        EnumMoniker_Reset
 */
stdmethod! {
fn EnumMonikerImpl_Reset(iface: *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    (*This).pos = 0; /* set back to start of list */

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    S_OK
}
}

/***********************************************************************
 *        EnumMoniker_Clone
 */
stdmethod! {
fn EnumMonikerImpl_Clone(iface: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);
    let moniker_list: *mut InterfaceList;
    let mut i: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    *ppenum = core::ptr::null_mut();

    moniker_list = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::offset_of!(InterfaceList, interfaces)
            + (*(*This).moniker_list).size as usize * core::mem::size_of::<PInterfaceData>())
            as SIZE_T,
    ) as *mut InterfaceList;
    if moniker_list.is_null() {
        return E_OUTOFMEMORY;
    }

    (*moniker_list).size = (*(*This).moniker_list).size;
    i = 0;
    while i < (*(*This).moniker_list).size {
        let src = *(*(*This).moniker_list).interfaces.as_ptr().add(i as usize);
        let size: usize =
            core::mem::offset_of!(InterfaceData, abData) + (*src).ulCntData as usize;
        let dst = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut InterfaceData;
        *(*moniker_list).interfaces.as_mut_ptr().add(i as usize) = dst;
        if dst.is_null() {
            let end = i;
            i = 0;
            while i < end {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    *(*moniker_list).interfaces.as_ptr().add(i as usize) as LPVOID,
                );
                i += 1;
            }
            HeapFree(GetProcessHeap(), 0, moniker_list as LPVOID);
            return E_OUTOFMEMORY;
        }
        memcpy(dst as *mut c_void, src as *const c_void, size);
        i += 1;
    }

    /* copy the enum structure */
    EnumMonikerImpl_CreateEnumROTMoniker(moniker_list, (*This).pos, ppenum)
}
}

/* Virtual function table for the IEnumMoniker class. */
static VT_EnumMonikerImpl: IEnumMonikerVtbl = IEnumMonikerVtbl {
    QueryInterface: Some(EnumMonikerImpl_QueryInterface),
    AddRef: Some(EnumMonikerImpl_AddRef),
    Release: Some(EnumMonikerImpl_Release),
    Next: Some(EnumMonikerImpl_Next),
    Skip: Some(EnumMonikerImpl_Skip),
    Reset: Some(EnumMonikerImpl_Reset),
    Clone: Some(EnumMonikerImpl_Clone),
};

/***********************************************************************
 *        EnumMonikerImpl_CreateEnumROTMoniker
 *        Used by EnumRunning to create the structure and EnumClone
 *        to copy the structure
 */
unsafe fn EnumMonikerImpl_CreateEnumROTMoniker(
    moniker_list: *mut InterfaceList,
    current_pos: ULONG,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let This: *mut EnumMonikerImpl;

    if ppenumMoniker.is_null() {
        return E_INVALIDARG;
    }

    This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumMonikerImpl>() as SIZE_T,
    ) as *mut EnumMonikerImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    /* initialize the virtual table function */
    (*This).IEnumMoniker_iface.lpVtbl =
        &VT_EnumMonikerImpl as *const IEnumMonikerVtbl as *mut IEnumMonikerVtbl;

    /* the initial reference is set to "1" */
    (*This).ref_ = 1; /* set the ref count to one         */
    (*This).pos = current_pos; /* Set the list start posn */
    (*This).moniker_list = moniker_list;

    *ppenumMoniker = core::ptr::addr_of_mut!((*This).IEnumMoniker_iface);

    S_OK
}

/* Shared implementation of moniker marshaler based on saving and loading of
 * monikers */

#[repr(C)]
pub struct MonikerMarshal {
    pub IUnknown_iface: IUnknown,
    pub IMarshal_iface: IMarshal,
    pub ref_: LONG,
    pub moniker: *mut IMoniker,
}

unsafe fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut MonikerMarshal {
    CONTAINING_RECORD!(iface, MonikerMarshal, IUnknown_iface)
}

unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut MonikerMarshal {
    CONTAINING_RECORD!(iface, MonikerMarshal, IMarshal_iface)
}

stdmethod! {
fn MonikerMarshalInner_QueryInterface(
    iface: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IUnknown(iface);
    TRACE!(b"(%s, %p)\n\0".as_ptr(), debugstr_guid(riid), ppv);
    *ppv = core::ptr::null_mut();
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid)
    {
        *ppv = core::ptr::addr_of_mut!((*This).IMarshal_iface) as *mut c_void;
        IMarshal_AddRef(core::ptr::addr_of_mut!((*This).IMarshal_iface));
        return S_OK;
    }
    FIXME!(b"No interface for %s\n\0".as_ptr(), debugstr_guid(riid));
    E_NOINTERFACE
}
}

stdmethod! {
fn MonikerMarshalInner_AddRef(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn MonikerMarshalInner_Release(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);
    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_ == 0 {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_
}
}

static VT_MonikerMarshalInner: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(MonikerMarshalInner_QueryInterface),
    AddRef: Some(MonikerMarshalInner_AddRef),
    Release: Some(MonikerMarshalInner_Release),
};

stdmethod! {
fn MonikerMarshal_QueryInterface(
    iface: *mut IMarshal,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    IMoniker_QueryInterface((*This).moniker, riid, ppv)
}
}

stdmethod! {
fn MonikerMarshal_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    IMoniker_AddRef((*This).moniker)
}
}

stdmethod! {
fn MonikerMarshal_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    IMoniker_Release((*This).moniker)
}
}

stdmethod! {
fn MonikerMarshal_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pCid
    );

    IMoniker_GetClassID((*This).moniker, pCid)
}
}

stdmethod! {
fn MonikerMarshal_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let hr: HRESULT;
    let mut size: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pSize
    );

    hr = IMoniker_GetSizeMax((*This).moniker, &mut size);
    if hr == S_OK {
        *pSize = size.QuadPart as DWORD;
    }
    hr
}
}

stdmethod! {
fn MonikerMarshal_MarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(
        b"(%p, %s, %p, %x, %p, %x)\n\0".as_ptr(),
        pStm,
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags
    );

    IMoniker_Save((*This).moniker, pStm, 0 /* FALSE */)
}
}

stdmethod! {
fn MonikerMarshal_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr(), pStm, debugstr_guid(riid), ppv);

    hr = IMoniker_Load((*This).moniker, pStm);
    if hr == S_OK {
        hr = IMoniker_QueryInterface((*This).moniker, riid, ppv);
    }
    hr
}
}

stdmethod! {
fn MonikerMarshal_ReleaseMarshalData(iface: *mut IMarshal, pStm: *mut IStream) -> HRESULT {
    TRACE!(b"()\n\0".as_ptr());
    /* can't release a state-based marshal as nothing on server side to
     * release */
    S_OK
}
}

stdmethod! {
fn MonikerMarshal_DisconnectObject(iface: *mut IMarshal, dwReserved: DWORD) -> HRESULT {
    TRACE!(b"()\n\0".as_ptr());
    /* can't disconnect a state-based marshal as nothing on server side to
     * disconnect from */
    S_OK
}
}

static VT_MonikerMarshal: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Some(MonikerMarshal_QueryInterface),
    AddRef: Some(MonikerMarshal_AddRef),
    Release: Some(MonikerMarshal_Release),
    GetUnmarshalClass: Some(MonikerMarshal_GetUnmarshalClass),
    GetMarshalSizeMax: Some(MonikerMarshal_GetMarshalSizeMax),
    MarshalInterface: Some(MonikerMarshal_MarshalInterface),
    UnmarshalInterface: Some(MonikerMarshal_UnmarshalInterface),
    ReleaseMarshalData: Some(MonikerMarshal_ReleaseMarshalData),
    DisconnectObject: Some(MonikerMarshal_DisconnectObject),
};

/// moniker.c: `HRESULT MonikerMarshal_Create(IMoniker *inner, IUnknown **outer)` — a
/// plain (cdecl) ole32-internal entry point, called by every moniker flavor's
/// QueryInterface(IID_IMarshal). NOT WINAPI, so `extern "C"` is correct here.
#[no_mangle]
pub unsafe extern "C" fn MonikerMarshal_Create(
    inner: *mut IMoniker,
    outer: *mut *mut IUnknown,
) -> HRESULT {
    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<MonikerMarshal>() as SIZE_T,
    ) as *mut MonikerMarshal;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IUnknown_iface.lpVtbl =
        &VT_MonikerMarshalInner as *const IUnknownVtbl as *mut IUnknownVtbl;
    (*This).IMarshal_iface.lpVtbl = &VT_MonikerMarshal as *const IMarshalVtbl as *mut IMarshalVtbl;
    (*This).ref_ = 1;
    (*This).moniker = inner;

    *outer = core::ptr::addr_of_mut!((*This).IUnknown_iface);
    S_OK
}

/// rpcndr.h: `void * __RPC_USER MIDL_user_allocate(SIZE_T)` — __RPC_USER is __stdcall
/// (rpc.h), so this MUST be `extern "system"`: the RPC runtime + the widl-generated
/// irot_c.c stubs call it through that ABI. An `extern "C"` spelling links on
/// amd64/arm64 and corrupts the stack on i386.
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(size: SIZE_T) -> LPVOID {
    HeapAlloc(GetProcessHeap(), 0, size)
}

/// rpcndr.h: `void __RPC_USER MIDL_user_free(void *)` — see MIDL_user_allocate.
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(p: LPVOID) {
    HeapFree(GetProcessHeap(), 0, p);
}
