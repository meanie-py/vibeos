use crate::prelude::*;
pub const MAX_COMPARISON_DATA: usize = 2048;

#[repr(C)]
pub struct rot_entry {
    pub entry: _LIST_ENTRY,
    pub object: *mut InterfaceData,
    pub moniker_data: *mut MonikerComparisonData,
    pub cookie: u32,
    pub last_modified: FILETIME,
    pub ctxt_handle: IrotContextHandle,
}

pub type Prot_entry = *mut rot_entry;

#[repr(C)]
pub struct RunningObjectTableImpl {
    pub IRunningObjectTable_iface: IRunningObjectTable,
    pub ref_: i32,
    pub rot: _LIST_ENTRY,
    pub lock: RTL_CRITICAL_SECTION,
}

pub type PRunningObjectTableImpl = *mut RunningObjectTableImpl;

pub static mut runningObjectTableInstance: *mut RunningObjectTableImpl = core::ptr::null_mut();
pub static mut irot_handle: IrotHandle = 0;

#[repr(C)]
pub struct EnumMonikerImpl {
    pub IEnumMoniker_iface: IEnumMoniker,
    pub ref_: i32,
    pub moniker_list: *mut InterfaceList,
    pub pos: u32,
}

pub type PEnumMonikerImpl = *mut EnumMonikerImpl;

#[repr(C)]
pub struct MonikerMarshal {
    pub IUnknown_iface: IUnknown,
    pub IMarshal_iface: IMarshal,
    pub ref_: i32,
    pub moniker: *mut IMoniker,
}

pub type PMonikerMarshal = *mut MonikerMarshal;

unsafe fn rpc_filter(eptr: *mut EXCEPTION_POINTERS) -> i32 {
    extern "C" {

        fn I_RpcExceptionFilter(code: u32) -> i32;
}
    I_RpcExceptionFilter((*(*eptr).ExceptionRecord).ExceptionCode)
}

unsafe fn impl_from_IRunningObjectTable(iface: *mut IRunningObjectTable) -> *mut RunningObjectTableImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(RunningObjectTableImpl, IRunningObjectTable_iface) as isize)) as *mut RunningObjectTableImpl
}

unsafe fn impl_from_IEnumMoniker(iface: *mut IEnumMoniker) -> *mut EnumMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(EnumMonikerImpl, IEnumMoniker_iface) as isize)) as *mut EnumMonikerImpl
}

unsafe fn get_irot_handle() -> IrotHandle {
    if irot_handle.is_null() {
        let mut status: RPC_STATUS;
        let mut binding: RPC_WSTR = core::ptr::null_mut();
        let mut new_handle: IrotHandle = core::ptr::null_mut();
        let ncacn_np: [u16; 10] = [b'n' as u16, b'c' as u16, b'a' as u16, b'c' as u16, b'n' as u16, b'_' as u16, b'n' as u16, b'p' as u16, 0, 0];
        let endpoint: [u16; 7] = [b'\\' as u16, b'p' as u16, b'i' as u16, b'p' as u16, b'e' as u16, 0, 0];

        status = RpcStringBindingComposeW(
            core::ptr::null_mut(),
            ncacn_np.as_ptr() as *mut u16,
            core::ptr::null_mut(),
            endpoint.as_ptr() as *mut u16,
            core::ptr::null_mut(),
            &mut binding
        );

        if status == RPC_S_OK {
            status = RpcBindingFromStringBindingW(binding, &mut new_handle);
            RpcStringFreeW(&mut binding);
        }

        if status != RPC_S_OK {
            return core::ptr::null_mut();
        }

        if !InterlockedCompareExchangePointer(
            &mut irot_handle as *mut _ as *mut *mut core::ffi::c_void,
            new_handle as *mut core::ffi::c_void,
            core::ptr::null_mut()
        ).is_null() {
            RpcBindingFree(&mut new_handle);
        }
    }
    irot_handle
}

extern "C" {

    fn RpcStringBindingComposeW(
        ObjUuid: *mut u16,
        ProtSeq: *mut u16,
        NetworkAddr: *mut u16,
        EndPoint: *mut u16,
        Options: *mut u16,
        StringBinding: *mut RPC_WSTR
    ) -> RPC_STATUS;

    fn RpcBindingFromStringBindingW(
        StringBinding: RPC_WSTR,
        Binding: *mut IrotHandle
    ) -> RPC_STATUS;

    fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS;

    fn RpcBindingFree(Binding: *mut IrotHandle) -> RPC_STATUS;
}

type RPC_STATUS = u32;
type RPC_WSTR = *mut u16;
const RPC_S_OK: RPC_STATUS = 0;

unsafe fn start_rpcss() -> u8 {
    static rpcssW: [u16; 6] = [b'R' as u16, b'p' as u16, b'c' as u16, b'S' as u16, b's' as u16, 0];
    let mut scm: *mut core::ffi::c_void;
    let mut service: *mut core::ffi::c_void;
    let mut status: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut ret: u8 = 0u8;

    extern "C" {

        fn OpenSCManagerW(lpMachineName: LPCWSTR, lpDatabaseName: LPCWSTR, dwDesiredAccess: u32) -> *mut core::ffi::c_void;
        fn OpenServiceW(hSCManager: *mut core::ffi::c_void, lpServiceName: LPCWSTR, dwDesiredAccess: u32) -> *mut core::ffi::c_void;
        fn StartServiceW(hService: *mut core::ffi::c_void, dwNumServiceArgs: u32, lpServiceArgVectors: *mut LPCWSTR) -> i32;
        fn QueryServiceStatusEx(hService: *mut core::ffi::c_void, InfoLevel: u32, lpBuffer: *mut u8, cbBufSize: u32, pcbBytesNeeded: LPDWORD) -> i32;
        fn GetTickCount64() -> u64;
        fn Sleep(dwMilliseconds: u32);
        fn CloseServiceHandle(hSCObject: *mut core::ffi::c_void) -> i32;
}

    const SERVICE_START: u32 = 0x0010;
    const SERVICE_QUERY_STATUS: u32 = 0x0004;
    const SC_STATUS_PROCESS_INFO: u32 = 0;
    const SERVICE_RUNNING: u32 = 4;
    const SERVICE_START_PENDING: u32 = 2;

    #[repr(C)]
    struct SERVICE_STATUS_PROCESS {
        dwServiceType: u32,
        dwCurrentState: u32,
        dwControlsAccepted: u32,
        dwWin32ExitCode: u32,
        dwServiceSpecificExitCode: u32,
        dwCheckPoint: u32,
        dwWaitHint: u32,
        dwProcessId: u32,
        dwServiceFlags: u32,
    }

    TRACE(b"\n\0".as_ptr());

    scm = OpenSCManagerW(core::ptr::null(), core::ptr::null(), 0);
    if scm.is_null() {
        ERR(b"failed to open service manager\n\0".as_ptr());
        return 0u8;
    }
    service = OpenServiceW(scm, rpcssW.as_ptr(), SERVICE_START | SERVICE_QUERY_STATUS);
    if service.is_null() {
        ERR(b"failed to open RpcSs service\n\0".as_ptr());
        CloseServiceHandle(scm);
        return 0u8;
    }
    if StartServiceW(service, 0, core::ptr::null_mut()) != 0 || GetLastError() == ERROR_SERVICE_ALREADY_RUNNING {
        let start_time: u64 = GetTickCount64();
        loop {
            let mut dummy: u32 = 0;

            if QueryServiceStatusEx(service, SC_STATUS_PROCESS_INFO,
                                   &mut status as *mut _ as *mut u8,
                                   core::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                                   &mut dummy) == 0 {
                break;
            }
            if status.dwCurrentState == SERVICE_RUNNING {
                ret = 1u8;
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
            WARN(b"RpcSs failed to start %u\n\0".as_ptr(), status.dwCurrentState);
        }
    } else {
        ERR(b"failed to start RpcSs service\n\0".as_ptr());
    }

    CloseServiceHandle(service);
    CloseServiceHandle(scm);
    ret
}

unsafe fn create_stream_on_mip_ro(mip: *const InterfaceData, stream: *mut *mut IStream) -> HRESULT {
    extern "C" {
        fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: i32, ppstm: *mut *mut IStream) -> HRESULT;
}
    extern "C" {

        fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

    let hglobal = GlobalAlloc(0, (*mip).ulCntData as usize);
    let pv = GlobalLock(hglobal);
    memcpy(pv, (*mip).abData.as_ptr() as *const core::ffi::c_void, (*mip).ulCntData as usize);
    GlobalUnlock(hglobal);
    CreateStreamOnHGlobal(hglobal, 1, stream)
}

unsafe fn get_moniker_comparison_data(pMoniker: *mut IMoniker, moniker_data: *mut *mut MonikerComparisonData) -> HRESULT {
    let mut hr: HRESULT;
    let mut pROTData: *mut IROTData = core::ptr::null_mut();
    hr = IMoniker_QueryInterface(pMoniker, &IID_IROTData, &mut pROTData as *mut *mut IROTData as *mut LPVOID);
    if SUCCEEDED(hr) {
        let size: ULONG = MAX_COMPARISON_DATA;
        *moniker_data = HeapAlloc(GetProcessHeap(), 0, FIELD_OFFSET(MonikerComparisonData, abData[size])) as *mut MonikerComparisonData;
        if (*moniker_data).is_null() {
            IROTData_Release(pROTData);
            return E_OUTOFMEMORY;
        }
        let mut actual_size = size;
        hr = IROTData_GetComparisonData(pROTData, (**moniker_data).abData.as_mut_ptr(), size, &mut actual_size);
        IROTData_Release(pROTData);
        if hr != S_OK {
            ERR(b"Failed to copy comparison data into buffer, hr = 0x%08x\n\0".as_ptr() as PCHAR, hr);
            HeapFree(GetProcessHeap(), 0, *moniker_data as LPVOID);
            return hr;
        }
        (**moniker_data).ulCntData = actual_size;
    } else {
        let mut pbc: *mut IBindCtx = core::ptr::null_mut();
        let mut pszDisplayName: LPWSTR = core::ptr::null_mut();
        let mut clsid: CLSID = core::mem::zeroed();
        let len: i32;

        TRACE(b"generating comparison data from display name\n\0".as_ptr() as PCHAR);

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
        *moniker_data = HeapAlloc(GetProcessHeap(), 0,
            FIELD_OFFSET(MonikerComparisonData, abData[core::mem::size_of::<CLSID>() + ((len+1) as usize) * core::mem::size_of::<u16>()])) as *mut MonikerComparisonData;
        if (*moniker_data).is_null() {
            CoTaskMemFree(pszDisplayName as LPVOID);
            return E_OUTOFMEMORY;
        }
        (**moniker_data).ulCntData = (core::mem::size_of::<CLSID>() + ((len+1) as usize) * core::mem::size_of::<u16>()) as ULONG;

        memcpy((**moniker_data).abData.as_mut_ptr() as *mut core::ffi::c_void, &clsid as *const CLSID as *const core::ffi::c_void, core::mem::size_of::<CLSID>());
        memcpy((**moniker_data).abData.as_mut_ptr().add(core::mem::size_of::<CLSID>()) as *mut core::ffi::c_void, pszDisplayName as *const core::ffi::c_void, ((len+1) as usize) * core::mem::size_of::<u16>());
        CoTaskMemFree(pszDisplayName as LPVOID);
    }
    return S_OK;
}

unsafe fn reduce_moniker(pmk: *mut IMoniker, pbc: *mut IBindCtx, pmkReduced: *mut *mut IMoniker) -> HRESULT {
    let mut pbcNew: *mut IBindCtx = core::ptr::null_mut();
    let mut hr: HRESULT;
    if pbc.is_null() {
        hr = CreateBindCtx(0, &mut pbcNew);
        if FAILED(hr) != 0 {
            return hr;
        }
        pbc = pbcNew;
    }
    hr = IMoniker_Reduce(pmk, pbc, MKRREDUCE_ALL, core::ptr::null_mut(), pmkReduced);
    if FAILED(hr) != 0 {
        ERR(b"reducing moniker failed with error 0x%08x\n\0".as_ptr() as *const i8, hr);
    }
    if !pbcNew.is_null() {
        IBindCtx_Release(pbcNew);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RunningObjectTableImpl_Initialize() -> HRESULT {
    TRACE(b"\n\0".as_ptr() as *const i8);

    runningObjectTableInstance = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<RunningObjectTableImpl>() as usize
    ) as *mut RunningObjectTableImpl;

    if runningObjectTableInstance.is_null() {
        return E_OUTOFMEMORY;
    }

    (*runningObjectTableInstance).IRunningObjectTable_iface.lpVtbl =
        &VT_RunningObjectTableImpl as *const IRunningObjectTableVtbl;

    (*runningObjectTableInstance).ref_0 = 1;

    list_init(core::ptr::addr_of_mut!((*runningObjectTableInstance).rot));
    InitializeCriticalSection(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));
    DEBUG_SET_CRITSEC_NAME(
        core::ptr::addr_of_mut!((*runningObjectTableInstance).lock),
        b"RunningObjectTableImpl.lock\0".as_ptr() as *const i8
    );

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetRunningObjectTable(
    reserved: DWORD,
    pprot: *mut LPRUNNINGOBJECTTABLE,
) -> HRESULT {


    let riid: IID = IID_IRunningObjectTable;
    let res: HRESULT;

    TRACE(b"()\n\0".as_ptr() as LPCSTR);

    if reserved != 0 {
        return E_UNEXPECTED;
    }

    if runningObjectTableInstance.is_null() {
        return CO_E_NOTINITIALIZED;
    }

    res = IRunningObjectTable_QueryInterface(
        core::ptr::addr_of_mut!((*runningObjectTableInstance).IRunningObjectTable_iface),
        &riid,
        pprot as *mut *mut core::ffi::c_void,
    );

    res
}

unsafe fn get_moniker_for_progid_display_name(
    pbc: *mut core::ffi::c_void,
    szDisplayName: LPCWSTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut progid: LPWSTR;
    let mut start: LPCWSTR = szDisplayName;
    let mut end: LPCWSTR;
    let len: i32;
    let mut class_moniker: *mut core::ffi::c_void = core::ptr::null_mut();

    if *start == '@' as u16 {
        start = start.offset(1);
    }

    end = start;
    while *end != 0 {
        if *end == ':' as u16 {
            break;
        }
        end = end.offset(1);
    }

    len = end.offset_from(start) as i32;

    if ((start == szDisplayName) && (*end == 0)) || (len <= 1) {
        return MK_E_SYNTAX;
    }

    progid = HeapAlloc(GetProcessHeap(), 0, ((len + 1) * 2) as usize) as LPWSTR;
    if !progid.is_null() {
        memcpy(
            progid as *mut core::ffi::c_void,
            start as *const core::ffi::c_void,
            (len * 2) as usize,
        );
        *progid.offset(len as isize) = 0;
    }
    hr = CLSIDFromProgID(progid, &mut clsid);
    HeapFree(GetProcessHeap(), 0, progid as *mut core::ffi::c_void);
    if FAILED(hr) != 0 {
        return MK_E_SYNTAX;
    }

    hr = CreateClassMoniker(&clsid, &mut class_moniker);
    if SUCCEEDED(hr) != 0 {
        let mut pdn: *mut core::ffi::c_void = core::ptr::null_mut();
        hr = IMoniker_BindToObject(
            class_moniker,
            pbc,
            core::ptr::null_mut(),
            &IID_IParseDisplayName,
            &mut pdn as *mut *mut core::ffi::c_void as *mut LPVOID,
        );
        if FAILED(hr) != 0 {
            let mut pcf: *mut core::ffi::c_void = core::ptr::null_mut();
            hr = IMoniker_BindToObject(
                class_moniker,
                pbc,
                core::ptr::null_mut(),
                &IID_IClassFactory,
                &mut pcf as *mut *mut core::ffi::c_void as *mut LPVOID,
            );
            if SUCCEEDED(hr) != 0 {
                hr = IClassFactory_CreateInstance(
                    pcf,
                    core::ptr::null_mut(),
                    &IID_IParseDisplayName,
                    &mut pdn as *mut *mut core::ffi::c_void as *mut LPVOID,
                );
                IClassFactory_Release(pcf);
            }
        }
        IMoniker_Release(class_moniker);
        if SUCCEEDED(hr) != 0 {
            hr = IParseDisplayName_ParseDisplayName(pdn, pbc, szDisplayName as LPWSTR, pchEaten, ppmk);
            IParseDisplayName_Release(pdn);
        }
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn GetClassFile(
    filePathName: LPCOLESTR,
    pclsid: *mut CLSID,
) -> HRESULT {
    let mut pstg: *mut IStorage = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut nbElm: i32;
    let mut length: i32;
    let mut i: i32;
    let mut sizeProgId: i32 = 0;
    let mut ret: i32;
    let mut pathDec: *mut LPOLESTR = core::ptr::null_mut();
    let mut absFile: LPOLESTR;
    let mut progId: LPOLESTR = core::ptr::null_mut();
    let mut extension: LPWSTR;
    static bkslashW: [u16; 2] = ['\\' as u16, 0];
    static dotW: [u16; 2] = ['.' as u16, 0];

    extern "C" {

        fn StgIsStorageFile(pwcsName: LPCOLESTR) -> HRESULT;
        fn StgOpenStorage(
            pwcsName: LPCOLESTR,
            pstgPriority: *mut IStorage,
            grfMode: u32,
            snbExclude: *mut *mut u16,
            reserved: u32,
            ppstgOpen: *mut *mut IStorage,
        ) -> HRESULT;
        fn ReadClassStg(pstg: *mut IStorage, pclsid: *mut CLSID) -> HRESULT;
        fn FileMonikerImpl_DecomposePath(
            path: LPCOLESTR,
            pathDec: *mut *mut LPOLESTR,
        ) -> i32;
        fn wcscmp(string1: LPCWSTR, string2: LPCWSTR) -> i32;
        fn RegQueryValueW(
            hKey: *mut core::ffi::c_void,
            lpSubKey: LPCWSTR,
            lpData: LPWSTR,
            lpcbData: *mut i32,
        ) -> i32;
        fn CLSIDFromProgID(lpszProgID: LPCOLESTR, lpclsid: *mut CLSID) -> HRESULT;
}

    const STGM_READ: u32 = 0x00000000;
    const STGM_SHARE_DENY_WRITE: u32 = 0x00000020;
    const S_OK: HRESULT = 0;
    const MK_E_INVALIDEXTENSION: HRESULT = 0x800401E6u32 as i32;
    static mut HKEY_CLASSES_ROOT: *mut core::ffi::c_void = 0x80000000 as *mut core::ffi::c_void;

    TRACE(
        b"%s, %p\n\0".as_ptr() as *const i8,
        debugstr_w(filePathName),
        pclsid,
    );

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

    nbElm = FileMonikerImpl_DecomposePath(filePathName, &mut pathDec);
    absFile = *pathDec.offset((nbElm - 1) as isize);

    if wcscmp(absFile, bkslashW.as_ptr()) == 0 {
        CoTaskMemFree(pathDec as LPVOID);
        return MK_E_INVALIDEXTENSION;
    }

    extension = core::ptr::null_mut();
    length = lstrlenW(absFile);
    i = length - 1;
    while i >= 0 {
        extension = absFile.offset(i as isize);
        if *extension == '.' as u16 {
            break;
        }
        i -= 1;
    }

    if extension.is_null() || wcscmp(extension, dotW.as_ptr()) == 0 {
        CoTaskMemFree(pathDec as LPVOID);
        return MK_E_INVALIDEXTENSION;
    }

    ret = RegQueryValueW(HKEY_CLASSES_ROOT, extension, core::ptr::null_mut(), &mut sizeProgId);
    if ret == 0 {
        progId = CoTaskMemAlloc(sizeProgId as usize) as LPOLESTR;
        ret = RegQueryValueW(HKEY_CLASSES_ROOT, extension, progId, &mut sizeProgId);
        if ret == 0 {
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

unsafe fn EnumMonikerImpl_CreateEnumROTMoniker(
    moniker_list: *mut InterfaceList,
    current_pos: ULONG,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let mut This: *mut EnumMonikerImpl = core::ptr::null_mut();

    if ppenumMoniker.is_null() {
        return E_INVALIDARG;
    }

    This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<EnumMonikerImpl>()) as *mut EnumMonikerImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    TRACE(b"(%p)\n\0".as_ptr() as LPCWSTR, This);

    (*This).IEnumMoniker_iface.lpVtbl = &VT_EnumMonikerImpl;

    (*This).ref_ = 1;
    (*This).pos = current_pos;
    (*This).moniker_list = moniker_list;

    *ppenumMoniker = &mut (*This).IEnumMoniker_iface;

    S_OK
}

unsafe fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut MonikerMarshal {
    (iface as *mut u8).offset(-(core::mem::offset_of!(MonikerMarshal, IUnknown_iface) as isize)) as *mut MonikerMarshal
}

unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut MonikerMarshal {
    (iface as *mut u8).offset(-(core::mem::offset_of!(MonikerMarshal, IMarshal_iface) as isize)) as *mut MonikerMarshal
}

unsafe fn MonikerMarshal_ReleaseMarshalData(iface: LPMARSHAL, pStm: *mut IStream) -> HRESULT {
    TRACE(b"()\n\0".as_ptr() as LPCWSTR);
    S_OK
}

unsafe fn MonikerMarshal_DisconnectObject(
    iface: LPMARSHAL,
    dwReserved: DWORD,
) -> HRESULT {
    TRACE(b"()\n\0".as_ptr() as LPCSTR);
    S_OK
}

unsafe fn MonikerMarshal_Create(inner: *mut IMoniker, outer: *mut *mut IUnknown) -> HRESULT {
    let This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MonikerMarshal>()) as *mut MonikerMarshal;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IUnknown_iface.lpVtbl = &VT_MonikerMarshalInner;
    (*This).IMarshal_iface.lpVtbl = &VT_MonikerMarshal;
    (*This).ref_ = 1;
    (*This).moniker = inner;

    *outer = &mut (*This).IUnknown_iface as *mut IUnknown;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(size: usize) -> *mut core::ffi::c_void {

    HeapAlloc(GetProcessHeap(), 0, size)
}

#[no_mangle]
pub unsafe extern "C" fn MIDL_user_free(p: LPVOID) {
    HeapFree(GetProcessHeap(), 0, p);
}

unsafe fn RunningObjectTableImpl_QueryInterface(
    iface: *mut IRunningObjectTable,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_IRunningObjectTable) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IRunningObjectTable_iface) as *mut core::ffi::c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IRunningObjectTable_AddRef(iface);

    S_OK
}

unsafe fn RunningObjectTableImpl_AddRef(iface: *mut IRunningObjectTable) -> ULONG {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE(b"(%p)\n\0".as_ptr() as PCHAR, This);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn EnumMonikerImpl_QueryInterface(
    iface: *mut IEnumMoniker,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    TRACE(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IEnumMoniker), riid) != 0
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumMoniker_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }

    IEnumMoniker_AddRef(iface);
    S_OK
}

unsafe fn EnumMonikerImpl_AddRef(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);

    TRACE(b"(%p)\n\0".as_ptr() as PCHAR, This);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn EnumMonikerImpl_Release(iface: *mut IEnumMoniker) -> u32 {
    let This = impl_from_IEnumMoniker(iface);
    let ref_: u32;

    TRACE(b"(%p)\n\0".as_ptr() as PCHAR, This);

    ref_ = InterlockedDecrement(&mut (*This).ref_) as u32;

    if ref_ == 0 {
        let mut i: u32;

        TRACE(b"(%p) Deleting\n\0".as_ptr() as PCHAR, This);

        i = 0;
        while i < (*(*This).moniker_list).size {
            HeapFree(GetProcessHeap(), 0, (*(*This).moniker_list).interfaces[i as usize] as LPVOID);
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, (*This).moniker_list as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    return ref_;
}

unsafe fn EnumMonikerImpl_Skip(iface: *mut IEnumMoniker, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    TRACE(b"(%p)\n\0".as_ptr() as *const i8, This);

    if (*This).pos + celt >= (*(*This).moniker_list).size {
        return S_FALSE;
    }

    (*This).pos += celt;

    S_OK
}

unsafe fn EnumMonikerImpl_Reset(iface: *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    (*This).pos = 0;

    TRACE(b"(%p)\n\0".as_ptr() as LPCWSTR, This);

    S_OK
}

unsafe fn RunningObjectTableImpl_NoteChangeTime(
    iface: *mut IRunningObjectTable,
    dwRegister: u32,
    pfiletime: *mut FILETIME,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut hr: HRESULT = 0x80070057u32 as HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    let mut current = (*This).rot.Flink;
    while current != core::ptr::addr_of_mut!((*This).rot) {
        let rot_entry = (current as *mut u8).offset(-(core::mem::offset_of!(rot_entry, entry) as isize)) as *mut rot_entry;

        if (*rot_entry).cookie == dwRegister {
            (*rot_entry).last_modified = *pfiletime;
            LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

            loop {
                hr = IrotNoteChangeTime(get_irot_handle(), dwRegister, pfiletime);

                if hr == ((0x800706BAu32 as HRESULT) & 0x0000FFFF) | 0x80070000 {
                    if start_rpcss() != 0 {
                        continue;
                    }
                }
                break;
            }

            return hr;
        }

        current = (*current).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));
    hr
}

unsafe fn RunningObjectTableImpl_EnumRunning(
    iface: *mut IRunningObjectTable,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut interface_list: *mut InterfaceList = core::ptr::null_mut();
    let mut hr: HRESULT;

    *ppenumMoniker = core::ptr::null_mut();

    loop {
        hr = IrotEnumRunning(get_irot_handle(), &mut interface_list);

        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) {
            if start_rpcss() != 0 {
                continue;
            }
        }
        break;
    }

    if SUCCEEDED(hr) {
        hr = EnumMonikerImpl_CreateEnumROTMoniker(interface_list, 0, ppenumMoniker);
    }

    hr
}

unsafe fn EnumMonikerImpl_Next(
    iface: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut i: ULONG;
    let This = impl_from_IEnumMoniker(iface);
    let mut hr: HRESULT = S_OK;

    TRACE(
        b"(%p) TabCurrentPos %d Tablastindx %d\n\0".as_ptr() as PCHAR,
        This,
        (*This).pos,
        (*(*This).moniker_list).size,
    );

    i = 0;
    while (*This).pos < (*(*This).moniker_list).size && i < celt {
        let mut stream: *mut IStream = core::ptr::null_mut();
        hr = create_stream_on_mip_ro(
            *(*(*This).moniker_list).interfaces.offset((*This).pos as isize),
            &mut stream,
        );
        (*This).pos += 1;
        if hr != S_OK {
            break;
        }
        hr = CoUnmarshalInterface(
            stream,
            &IID_IMoniker,
            &mut *rgelt.offset(i as isize) as *mut *mut IMoniker as *mut LPVOID,
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

unsafe fn RunningObjectTableImpl_IsRunning(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut pmkObjectName = pmkObjectName;

    hr = reduce_moniker(pmkObjectName, core::ptr::null_mut(), &mut pmkObjectName);
    if hr < 0 {
        return hr;
    }
    hr = get_moniker_comparison_data(pmkObjectName, &mut moniker_data);
    IMoniker_Release(pmkObjectName);
    if hr != 0 {
        return hr;
    }

    hr = 1;
    EnterCriticalSection(&mut (*This).lock);
    let mut current = (*This).rot.next;
    while current != &mut (*This).rot as *mut LIST_ENTRY {
        let rot_entry = (current as *const u8).offset(-(core::mem::offset_of!(rot_entry, entry) as isize)) as *const rot_entry;
        if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
            && memcmp(
                (*moniker_data).abData.as_ptr() as *const _,
                (*(*rot_entry).moniker_data).abData.as_ptr() as *const _,
                (*moniker_data).ulCntData as usize,
            ) == 0
        {
            hr = 0;
            break;
        }
        current = (*current).next;
    }
    LeaveCriticalSection(&mut (*This).lock);

    if hr == 1 {
        loop {
            hr = IrotIsRunning(get_irot_handle(), moniker_data);
            if hr == ((0x80070000u32 | RPC_S_SERVER_UNAVAILABLE) as i32) {
                if start_rpcss() != 0 {
                    continue;
                }
            }
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);
    hr
}

unsafe fn RunningObjectTableImpl_GetObject(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
    ppunkObject: *mut *mut IUnknown,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();
    let mut object: *mut InterfaceData = core::ptr::null_mut();
    let mut cookie: IrotCookie = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut pmkObjectName = pmkObjectName;

    TRACE(b"(%p,%p,%p)\n\0".as_ptr() as PCHAR, This, pmkObjectName, ppunkObject);

    if ppunkObject.is_null() {
        return E_POINTER;
    }

    *ppunkObject = core::ptr::null_mut();

    hr = reduce_moniker(pmkObjectName, core::ptr::null_mut(), &mut pmkObjectName);
    if FAILED(hr) != 0 {
        return hr;
    }
    hr = get_moniker_comparison_data(pmkObjectName, &mut moniker_data);
    IMoniker_Release(pmkObjectName);
    if hr != S_OK {
        return hr;
    }

    EnterCriticalSection(&mut (*This).lock);
    let mut current = (*This).rot.entry.next;
    while current != &mut (*This).rot.entry as *mut LIST_ENTRY {
        let rot_entry = (current as usize - core::mem::offset_of!(rot_entry, entry)) as *mut rot_entry;

        if (*(*rot_entry).moniker_data).ulCntData == (*moniker_data).ulCntData
            && memcmp(
                (*moniker_data).abData.as_ptr() as *const _,
                (*(*rot_entry).moniker_data).abData.as_ptr() as *const _,
                (*moniker_data).ulCntData as usize,
            ) == 0
        {
            let mut pStream: *mut IStream = core::ptr::null_mut();
            hr = create_stream_on_mip_ro((*rot_entry).object, &mut pStream);
            if hr == S_OK {
                hr = CoUnmarshalInterface(pStream, &IID_IUnknown, ppunkObject as *mut LPVOID);
                IStream_Release(pStream);
            }

            LeaveCriticalSection(&mut (*This).lock);
            HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);

            return hr;
        }
        current = (*current).next;
    }
    LeaveCriticalSection(&mut (*This).lock);

    TRACE(b"moniker unavailable locally, calling SCM\n\0".as_ptr() as PCHAR);

    loop {
        hr = IrotGetObject(get_irot_handle(), moniker_data, &mut object, &mut cookie);

        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) {
            if start_rpcss() != 0 {
                continue;
            }
        }
        break;
    }

    if SUCCEEDED(hr) != 0 {
        let mut pStream: *mut IStream = core::ptr::null_mut();
        hr = create_stream_on_mip_ro(object, &mut pStream);
        if hr == S_OK {
            hr = CoUnmarshalInterface(pStream, &IID_IUnknown, ppunkObject as *mut LPVOID);
            IStream_Release(pStream);
        }
    } else {
        WARN(b"Moniker unavailable, IrotGetObject returned 0x%08x\n\0".as_ptr() as PCHAR, hr);
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);
    return hr;
}

unsafe extern "system" fn RunningObjectTableImpl_GetTimeOfLastChange(
    iface: *mut IRunningObjectTable,
    pmkObjectName: *mut IMoniker,
    pfiletime: *mut FILETIME,
) -> HRESULT {
    let mut hr: HRESULT = MK_E_UNAVAILABLE;
    let This = impl_from_IRunningObjectTable(iface);
    let mut moniker_data: *mut MonikerComparisonData = core::ptr::null_mut();
    let mut pmkObjectName = pmkObjectName;

    TRACE(b"(%p,%p,%p)\n\0".as_ptr() as LPCWSTR, This, pmkObjectName, pfiletime);

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

    EnterCriticalSection(&mut (*This).lock);
    LIST_FOR_EACH_ENTRY!(
        rot_entry,
        core::ptr::addr_of_mut!((*This).rot),
        rot_entry,
        entry,
        {
            if (*rot_entry).moniker_data.as_ref().unwrap().ulCntData == (*moniker_data).ulCntData
                && memcmp(
                    (*moniker_data).abData.as_ptr() as *const core::ffi::c_void,
                    (*rot_entry).moniker_data.as_ref().unwrap().abData.as_ptr() as *const core::ffi::c_void,
                    (*moniker_data).ulCntData as usize,
                ) == 0
            {
                *pfiletime = (*rot_entry).last_modified;
                hr = S_OK;
                break;
            }
        }
    );
    LeaveCriticalSection(&mut (*This).lock);

    if hr != S_OK {
        loop {
            hr = IrotGetTimeOfLastChange(get_irot_handle(), moniker_data, pfiletime);
            if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) {
                if start_rpcss() != 0 {
                    continue;
                }
            }
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, moniker_data as LPVOID);
    TRACE(b"-- 0x%08x\n\0".as_ptr() as LPCWSTR, hr);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn MkParseDisplayName(
    pbc: *mut core::ffi::c_void,
    szDisplayName: LPCWSTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut core::ffi::c_void,
) -> HRESULT {
    extern "C" {
        fn IsValidInterface(punk: *mut core::ffi::c_void) -> i32;
        fn _wcsnicmp(s1: LPCWSTR, s2: LPCWSTR, n: usize) -> i32;
        fn ClassMoniker_CreateFromDisplayName(
            pbc: *mut core::ffi::c_void,
            szDisplayName: LPCWSTR,
            pchEaten: LPDWORD,
            ppmk: *mut *mut core::ffi::c_void,
        ) -> HRESULT;
        fn FileMoniker_CreateFromDisplayName(
            pbc: *mut core::ffi::c_void,
            szDisplayName: LPCWSTR,
            pchEaten: LPDWORD,
            ppmk: *mut *mut core::ffi::c_void,
        ) -> HRESULT;
}

    const MK_E_SYNTAX: HRESULT = 0x800401E4u32 as i32;
    const S_OK: HRESULT = 0;
    const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;

    let mut hr: HRESULT = MK_E_SYNTAX;
    static wszClsidColon: [u16; 6] = [b'c' as u16, b'l' as u16, b's' as u16, b'i' as u16, b'd' as u16, b':' as u16];
    let mut moniker: *mut core::ffi::c_void;
    let mut chEaten: u32;

    TRACE(b"(%p, %s, %p, %p)\n\0".as_ptr() as *const i8, pbc, debugstr_w(szDisplayName), pchEaten, ppmk);

    if pbc.is_null() || IsValidInterface(pbc) == 0 {
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

    if _wcsnicmp(szDisplayName, wszClsidColon.as_ptr(), 6) == 0 {
        hr = ClassMoniker_CreateFromDisplayName(pbc, szDisplayName, &mut chEaten, &mut moniker);
        if hr < 0 && hr != MK_E_SYNTAX {
            return hr;
        }
    } else {
        hr = get_moniker_for_progid_display_name(pbc, szDisplayName, &mut chEaten, &mut moniker);
        if hr < 0 && hr != MK_E_SYNTAX {
            return hr;
        }
    }

    if hr < 0 {
        hr = FileMoniker_CreateFromDisplayName(pbc, szDisplayName, &mut chEaten, &mut moniker);
        if hr < 0 && hr != MK_E_SYNTAX {
            return hr;
        }
    }

    if hr >= 0 {
        let mut szDisplayName = szDisplayName;
        loop {
            let mut next_moniker: *mut core::ffi::c_void;
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
                szDisplayName as LPWSTR,
                &mut chEaten,
                &mut next_moniker,
            );
            IMoniker_Release(moniker);
            if hr < 0 {
                *pchEaten = 0;
                break;
            }
            moniker = next_moniker;
        }
    }

    hr
}

unsafe extern "system" fn EnumMonikerImpl_Clone(iface: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);
    let mut moniker_list: *mut InterfaceList;
    let mut i: ULONG;

    TRACE(b"(%p)\n\0".as_ptr() as PCHAR, This);

    *ppenum = core::ptr::null_mut();

    moniker_list = HeapAlloc(GetProcessHeap(), 0, FIELD_OFFSET(InterfaceList, interfaces[(*(*This).moniker_list).size as usize])) as *mut InterfaceList;
    if moniker_list.is_null() {
        return E_OUTOFMEMORY;
    }

    (*moniker_list).size = (*(*This).moniker_list).size;
    i = 0;
    while i < (*(*This).moniker_list).size {
        let size = FIELD_OFFSET(InterfaceData, abData[(*(*(*This).moniker_list).interfaces[i as usize]).ulCntData as usize]);
        (*moniker_list).interfaces[i as usize] = HeapAlloc(GetProcessHeap(), 0, size) as *mut InterfaceData;
        if (*moniker_list).interfaces[i as usize].is_null() {
            let end = i;
            i = 0;
            while i < end {
                HeapFree(GetProcessHeap(), 0, (*moniker_list).interfaces[i as usize] as LPVOID);
                i += 1;
            }
            HeapFree(GetProcessHeap(), 0, moniker_list as LPVOID);
            return E_OUTOFMEMORY;
        }
        memcpy((*moniker_list).interfaces[i as usize] as *mut core::ffi::c_void, (*(*(*This).moniker_list).interfaces[i as usize]) as *const core::ffi::c_void, size);
        i += 1;
    }

    return EnumMonikerImpl_CreateEnumROTMoniker(moniker_list, (*This).pos, ppenum);
}

unsafe fn MonikerMarshalInner_QueryInterface(
    iface: *mut IUnknown,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IUnknown(iface);
    TRACE(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_guid(riid), ppv);
    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_IMarshal) != 0 {
        *ppv = core::ptr::addr_of_mut!((*This).IMarshal_iface) as LPVOID;
        IMarshal_AddRef(core::ptr::addr_of_mut!((*This).IMarshal_iface));
        return S_OK;
    }
    FIXME(b"No interface for %s\n\0".as_ptr() as *const i8, debugstr_guid(riid));
    E_NOINTERFACE
}

unsafe fn MonikerMarshalInner_AddRef(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn MonikerMarshalInner_Release(iface: *mut IUnknown) -> ULONG {
    let This = impl_from_IUnknown(iface);
    let ref_ = InterlockedDecrement(&mut (*This).ref_);

    if ref_ == 0 {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_ as ULONG
}

unsafe fn MonikerMarshal_QueryInterface(iface: *mut IMarshal, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    IMoniker_QueryInterface((*This).moniker, riid, ppv)
}

unsafe fn MonikerMarshal_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    IMoniker_AddRef((*This).moniker)
}

unsafe fn MonikerMarshal_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    IMoniker_Release((*This).moniker)
}

unsafe extern "system" fn MonikerMarshal_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut core::ffi::c_void,
    dwDestContext: u32,
    pvDestContext: *mut core::ffi::c_void,
    mshlflags: u32,
    pCid: *mut CLSID,
) -> HRESULT {


    let This = impl_from_IMarshal(iface);

    TRACE(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pCid,
    );

    IMoniker_GetClassID((*This).moniker, pCid)
}

unsafe fn MonikerMarshal_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut core::ffi::c_void,
    dwDestContext: u32,
    pvDestContext: *mut core::ffi::c_void,
    mshlflags: u32,
    pSize: *mut u32,
) -> HRESULT {
    let this = impl_from_IMarshal(iface);
    let mut size: ULARGE_INTEGER = core::mem::zeroed();



    TRACE(
        b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr(),
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
        pSize,
    );

    let hr = IMoniker_GetSizeMax((*this).moniker, &mut size);
    if hr == 0 {
        *pSize = *size.QuadPart as u32;
    }
    hr
}

unsafe extern "system" fn MonikerMarshal_MarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    pv: *mut core::ffi::c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut core::ffi::c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE(
        b"(%p, %s, %p, %x, %p, %x)\n\0".as_ptr() as *const i8,
        pStm,
        debugstr_guid(riid),
        pv,
        dwDestContext,
        pvDestContext,
        mshlflags,
    );

    IMoniker_Save((*This).moniker, pStm, 0u8)
}

unsafe fn MonikerMarshal_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = impl_from_IMarshal(iface);
    let mut hr: HRESULT;

    TRACE(
        b"(%p, %s, %p)\n\0".as_ptr() as *const i8,
        pStm,
        debugstr_guid(riid),
        ppv,
    );

    hr = IMoniker_Load((*this).moniker, pStm);
    if hr == 0 {
        hr = IMoniker_QueryInterface((*this).moniker, riid, ppv);
    }
    hr
}

unsafe fn rot_entry_delete(rot_entry: *mut rot_entry) {
    if (*rot_entry).cookie != 0 {
        let mut object: *mut InterfaceData = core::ptr::null_mut();
        let mut moniker: *mut InterfaceData = core::ptr::null_mut();

        IrotRevoke(
            get_irot_handle(),
            (*rot_entry).cookie,
            core::ptr::addr_of_mut!((*rot_entry).ctxt_handle),
            core::ptr::addr_of_mut!(object),
            core::ptr::addr_of_mut!(moniker),
        );

        MIDL_user_free(object as LPVOID);
        if !moniker.is_null() {
            let mut stream: *mut IStream = core::ptr::null_mut();
            let hr = create_stream_on_mip_ro(moniker, core::ptr::addr_of_mut!(stream));
            if hr == 0 {
                CoReleaseMarshalData(stream);
                IStream_Release(stream);
            }
        }
        MIDL_user_free(moniker as LPVOID);
    }
    if !(*rot_entry).object.is_null() {
        let mut stream: *mut IStream = core::ptr::null_mut();
        let hr = create_stream_on_mip_ro((*rot_entry).object, core::ptr::addr_of_mut!(stream));
        if hr == 0 {
            CoReleaseMarshalData(stream);
            IStream_Release(stream);
        }
    }
    HeapFree(GetProcessHeap(), 0, (*rot_entry).object as LPVOID);
    HeapFree(GetProcessHeap(), 0, (*rot_entry).moniker_data as LPVOID);
    HeapFree(GetProcessHeap(), 0, rot_entry as LPVOID);
}

unsafe fn RunningObjectTableImpl_Destroy() -> HRESULT {
    let mut cursor: *mut LIST_ENTRY;
    let mut cursor2: *mut LIST_ENTRY;
    let old_handle: IrotHandle;

    TRACE(b"()\n\0".as_ptr() as *const i8);

    if runningObjectTableInstance.is_null() {
        return E_INVALIDARG;
    }

    LIST_FOR_EACH_SAFE(
        cursor,
        cursor2,
        core::ptr::addr_of_mut!((*runningObjectTableInstance).rot)
    );
    while !cursor.is_null() {
        let rot_entry = CONTAINING_RECORD!(cursor, rot_entry, entry);
        list_remove(core::ptr::addr_of_mut!((*rot_entry).entry));
        rot_entry_delete(rot_entry);
        cursor = cursor2;
        if !cursor2.is_null() {
            cursor2 = (*cursor2).Flink;
        }
    }

    DEBUG_CLEAR_CRITSEC_NAME(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));
    DeleteCriticalSection(core::ptr::addr_of_mut!((*runningObjectTableInstance).lock));

    HeapFree(GetProcessHeap(), 0, runningObjectTableInstance as LPVOID);
    runningObjectTableInstance = core::ptr::null_mut();

    old_handle = irot_handle;
    irot_handle = core::ptr::null_mut();
    if !old_handle.is_null() {
        RpcBindingFree(core::ptr::addr_of_mut!(old_handle as IrotHandle) as *mut *mut core::ffi::c_void);
    }

    S_OK
}

unsafe fn RunningObjectTableImpl_Release(iface: *mut IRunningObjectTable) -> ULONG {
    let This = impl_from_IRunningObjectTable(iface);
    let ref_ = InterlockedDecrement(&mut (*This).ref_);

    if ref_ == 0 {
        let mut cursor = (*This).rot.Flink;
        while cursor != core::ptr::addr_of_mut!((*This).rot) {
            let next = (*cursor).Flink;
            let rot_entry = CONTAINING_RECORD!(cursor, rot_entry, entry);
            list_remove(&mut (*rot_entry).entry);
            rot_entry_delete(rot_entry);
            cursor = next;
        }
    }

    ref_ as ULONG
}

unsafe fn RunningObjectTableImpl_Register(
    iface: *mut IRunningObjectTable,
    grfFlags: DWORD,
    punkObject: *mut IUnknown,
    pmkObjectName: *mut IMoniker,
    pdwRegister: LPDWORD,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);
    let mut rot_entry: *mut rot_entry;
    let mut hr: HRESULT = S_OK;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let mshlflags: DWORD;
    let mut pbc: *mut IBindCtx;
    let mut moniker: *mut InterfaceData = core::ptr::null_mut();

    extern "C" {

        fn CoFileTimeNow(lpFileTime: *mut FILETIME) -> HRESULT;
        fn CoMarshalInterface(
            pStm: *mut IStream,
            riid: *const IID,
            pUnk: *mut IUnknown,
            dwDestContext: DWORD,
            pvDestContext: LPVOID,
            mshlflags: DWORD,
        ) -> HRESULT;
        fn CreateBindCtx(reserved: DWORD, ppbc: *mut *mut IBindCtx) -> HRESULT;
        fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: i32, ppstm: *mut *mut IStream) -> HRESULT;
        fn FIELD_OFFSET(type_: *const (), field: *const ()) -> usize;
        fn GetExceptionCode() -> DWORD;
        fn GetHGlobalFromStream(pstm: *mut IStream, phglobal: *mut HGLOBAL) -> HRESULT;
        fn IrotRegister(
            handle: IrotHandle,
            moniker_data: *const MonikerComparisonData,
            object: *const InterfaceData,
            moniker: *const InterfaceData,
            pfiletime: *const FILETIME,
            grfFlags: DWORD,
            cookie: *mut DWORD,
            ctxt_handle: *mut IrotContextHandle,
        ) -> HRESULT;
        fn memcpy(dest: *mut (), src: *const (), n: usize) -> *mut ();
}

    TRACE(
        b"(%p,%d,%p,%p,%p)\n\0".as_ptr() as *const i8,
        This,
        grfFlags,
        punkObject,
        pmkObjectName,
        pdwRegister,
    );

    if (grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT)) != 0 {
        ERR(
            b"Invalid grfFlags: 0x%08x\n\0".as_ptr() as *const i8,
            grfFlags & !(ROTFLAGS_REGISTRATIONKEEPSALIVE | ROTFLAGS_ALLOWANYCLIENT),
        );
        return E_INVALIDARG;
    }

    if punkObject.is_null() || pmkObjectName.is_null() || pdwRegister.is_null() {
        return E_INVALIDARG;
    }

    rot_entry = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<rot_entry>()) as *mut rot_entry;
    if rot_entry.is_null() {
        return E_OUTOFMEMORY;
    }

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
        &IID_IUnknown,
        punkObject,
        MSHCTX_LOCAL | MSHCTX_NOSHAREDMEM,
        core::ptr::null_mut(),
        mshlflags,
    );
    if hr == S_OK {
        let mut hglobal: HGLOBAL = core::ptr::null_mut();
        hr = GetHGlobalFromStream(pStream, &mut hglobal);
        if hr == S_OK {
            let size = GlobalSize(hglobal);
            let pv = GlobalLock(hglobal);
            (*rot_entry).object = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::offset_of!(MInterfacePointer, abData) + size,
            ) as *mut InterfaceData;
            (*(*rot_entry).object).ulCntData = size as ULONG;
            memcpy(
                (*(*rot_entry).object).abData.as_mut_ptr() as *mut (),
                pv,
                size,
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
    if FAILED(hr) != 0 {
        rot_entry_delete(rot_entry);
        return hr;
    }

    hr = reduce_moniker(pmkObjectName, pbc, &mut pmkObjectName);
    if FAILED(hr) != 0 {
        rot_entry_delete(rot_entry);
        IBindCtx_Release(pbc);
        return hr;
    }

    hr = IMoniker_GetTimeOfLastChange(
        pmkObjectName,
        pbc,
        core::ptr::null_mut(),
        &mut (*rot_entry).last_modified,
    );
    IBindCtx_Release(pbc);
    if FAILED(hr) != 0 {
        CoFileTimeNow(&mut (*rot_entry).last_modified);
        hr = S_OK;
    }

    hr = get_moniker_comparison_data(pmkObjectName, &mut (*rot_entry).moniker_data);
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
    hr = CoMarshalInterface(
        pStream,
        &IID_IMoniker,
        pmkObjectName as *mut IUnknown,
        MSHCTX_LOCAL | MSHCTX_NOSHAREDMEM,
        core::ptr::null_mut(),
        MSHLFLAGS_TABLESTRONG,
    );
    if hr == S_OK {
        let mut hglobal: HGLOBAL = core::ptr::null_mut();
        hr = GetHGlobalFromStream(pStream, &mut hglobal);
        if hr == S_OK {
            let size = GlobalSize(hglobal);
            let pv = GlobalLock(hglobal);
            moniker = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::offset_of!(InterfaceData, abData) + size,
            ) as *mut InterfaceData;
            (*moniker).ulCntData = size as ULONG;
            memcpy((*moniker).abData.as_mut_ptr() as *mut (), pv, size);
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
        hr = IrotRegister(
            get_irot_handle(),
            (*rot_entry).moniker_data,
            (*rot_entry).object,
            moniker,
            &(*rot_entry).last_modified,
            grfFlags,
            &mut (*rot_entry).cookie,
            &mut (*rot_entry).ctxt_handle,
        );
        if hr == HRESULT_FROM_WIN32(RPC_S_SERVER_UNAVAILABLE) {
            if start_rpcss() != 0 {
                continue;
            }
        }
        break;
    }

    hr
}

unsafe fn RunningObjectTableImpl_Revoke(
    iface: *mut IRunningObjectTable,
    dwRegister: DWORD,
) -> HRESULT {
    let This = impl_from_IRunningObjectTable(iface);

    TRACE(b"(%p,%d)\n\0".as_ptr() as LPCWSTR, This, dwRegister);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    let mut current = (*This).rot.Flink;
    while current != core::ptr::addr_of_mut!((*This).rot) {
        let rot_entry = CONTAINING_RECORD!(current, rot_entry, entry) as *mut rot_entry;

        if (*rot_entry).cookie == dwRegister {
            list_remove(core::ptr::addr_of_mut!((*rot_entry).entry));
            LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

            rot_entry_delete(rot_entry);
            return S_OK;
        }

        current = (*current).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).lock));

    E_INVALIDARG
}

#[no_mangle]
pub unsafe extern "system" fn RunningObjectTableImpl_UnInitialize() -> HRESULT {
    TRACE(b"\n\0".as_ptr() as LPCWSTR);

    if runningObjectTableInstance.is_null() {
        return E_POINTER;
    }

    RunningObjectTableImpl_Release(core::ptr::addr_of_mut!((*runningObjectTableInstance).IRunningObjectTable_iface));

    RunningObjectTableImpl_Destroy();

    return S_OK;
}
