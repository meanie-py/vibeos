use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn fixme_ole(format: *const i8, ...);
    fn wcstoul( nptr: *const WCHAR, endptr: *mut *mut WCHAR, base: i32, ) -> u32;
}
extern "C" {
    fn COM_OpenKeyForCLSID( clsid: *const CLSID, keyname: LPCWSTR, access: u32, hkey: *mut HKEY, ) -> HRESULT;
    fn CoTaskMemAlloc(cb: usize) -> LPVOID;
    fn CoTaskMemFree(pv: LPVOID);
    fn RegCloseKey(hKey: HKEY) -> i32;
    fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: *mut DWORD, lpType: *mut DWORD, lpData: *mut u8, lpcbData: *mut DWORD, ) -> i32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn GetCurrentThreadId() -> u32;
    fn SetWindowsHookExW(idHook: i32, lpfn: Option<unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT>, hmod: HINSTANCE, dwThreadId: u32) -> HHOOK;
    fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
    fn GetMenuItemCount(hMenu: HMENU) -> i32;
    fn GetSubMenu(hMenu: HMENU, nPos: i32) -> HMENU;
    fn CopyAcceleratorTableW(hAccelSrc: HACCEL, lpAccelDst: LPACCEL, cAccelEntries: i32) -> i32;
    fn GetKeyState(nVirtKey: i32) -> i16;
    fn TRACE_(channel: *const i8, fmt: *const i8, ...);
    fn WARN_(channel: *const i8, fmt: *const i8, ...);
    fn DeleteEnhMetaFile(hemf: HENHMETAFILE) -> BOOL;
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
    fn DeleteMetaFile(hmf: HMETAFILE) -> BOOL;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn GetKeyboardState(lpKeyState: *mut BYTE) -> BOOL;
    fn CLSIDFromString(lpsz: *const u16, pclsid: *mut CLSID) -> HRESULT;
    fn RegQueryValueW(hKey: HKEY, lpSubKey: *mut u16, lpData: *mut u16, lpcbData: *mut i32) -> i32;
    fn RegSetValueW(hKey: HKEY, lpSubKey: *const u16, dwType: u32, lpData: *const u16, cbData: u32) -> i32;
    fn StringFromGUID2(rguid: *const CLSID, lpsz: *mut u16, cchMax: i32) -> i32;
    fn CoLockObjectExternal(pUnk: LPUNKNOWN, fLock: BOOL, fLastUnlockReleases: BOOL) -> HRESULT;
    fn IsWindow(hwnd: HWND) -> i32;
    fn GetPropW(hwnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn RemovePropW(hwnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn CoReleaseMarshalData(pStm: *mut IStream) -> HRESULT;
    fn CloseHandle(hObject: HANDLE) -> i32;
    fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LONG;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcstol(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> i32;
    fn COM_CurrentInfo() -> *mut core::ffi::c_void;
    fn CoUninitialize();
    fn CallNextHookEx(hhk: HHOOK, nCode: c_int, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetPropW(hWnd: HWND, lpString: LPCWSTR, hData: HANDLE) -> i32;
    fn IsWindowUnicode(hwnd: HWND) -> BOOL;
    fn PostMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
    fn PostMessageA(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
    fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: u32) -> HRESULT;
    fn actctx_get_miscstatus( clsid: *const CLSID, dwAspect: DWORD, pdwStatus: *mut DWORD, ) -> i32;
    fn open_classes_key( hkey: HKEY, name: LPCWSTR, access: u32, retkey: *mut HKEY, ) -> LONG;
    fn ReadFmtUserTypeStg(pstg: LPSTORAGE, pcf: *mut CLIPFORMAT, lplpszUserType: *mut LPOLESTR) -> HRESULT;
    fn SetConvertStg(pstg: LPSTORAGE, fConvert: BOOL) -> HRESULT;
    fn WriteFmtUserTypeStg(pstg: LPSTORAGE, cf: CLIPFORMAT, lpszUserType: LPOLESTR) -> HRESULT;
    fn SafeArrayCopy(psa: *mut SAFEARRAY, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT;
    fn lstrlenA(lpString: LPCSTR) -> i32;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn strlen(s: *const i8) -> usize;
    fn COM_CurrentApt() -> *mut core::ffi::c_void;
    fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: BOOL, ppstm: *mut *mut IStream) -> HRESULT;
    fn CoMarshalInterface(pStm: *mut IStream, riid: *const CLSID, pUnk: *mut IUnknown, dwDestContext: DWORD, pvDestContext: LPVOID, mshlflags: DWORD) -> HRESULT;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
    fn GetCurrentProcessId() -> DWORD;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ReleaseCapture() -> BOOL;
    fn CreateWindowW( lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: c_int, y: c_int, nWidth: c_int, nHeight: c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND;
    fn DestroyWindow(hWnd: HWND) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn PostQuitMessage(nExitCode: c_int);
    fn SetCapture(hWnd: HWND) -> HWND;
    fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT;
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct tagTrackerWindowInfo {
    pub dataObject: *mut IDataObject,
    pub dropSource: *mut IDropSource,
    pub dwOKEffect: DWORD,
    pub pdwEffect: *mut DWORD,
    pub trackingDone: BOOL,
    pub returnValue: HRESULT,
    pub escPressed: BOOL,
    pub curTargetHWND: HWND,
    pub curDragTarget: *mut IDropTarget,
    pub accepterHWND: HWND,
    pub curMousePos: POINTL,
    pub dwKeyState: DWORD,
}
pub type TrackerWindowInfo = tagTrackerWindowInfo;

#[repr(C)]
pub struct tagOleMenuDescriptor {
    pub hwndFrame: HWND,
    pub hwndActiveObject: HWND,
    pub mgw: OLEMENUGROUPWIDTHS,
    pub hmenuCombined: HMENU,
    pub bIsServerItem: BOOL,
}
pub type OleMenuDescriptor = tagOleMenuDescriptor;

#[repr(C)]
pub struct tagOleMenuHookItem {
    pub tid: DWORD,
    pub hHeap: HANDLE,
    pub GetMsg_hHook: HHOOK,
    pub CallWndProc_hHook: HHOOK,
    pub next: *mut tagOleMenuHookItem,
}
pub type OleMenuHookItem = tagOleMenuHookItem;

pub static mut hook_list: *mut OleMenuHookItem = core::ptr::null_mut();

pub static mut OLE_moduleLockCount: LONG = 0;

pub static OLEDD_DRAGTRACKERCLASS: [WCHAR; 22] = [
    'W' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 'D' as u16, 'r' as u16, 'a' as u16, 'g' as u16,
    'D' as u16, 'r' as u16, 'o' as u16, 'p' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'c' as u16,
    'k' as u16, 'e' as u16, 'r' as u16, '3' as u16, '2' as u16, 0,
];

pub static prop_olemenuW: [WCHAR; 23] = [
    'P' as u16, 'R' as u16, 'O' as u16, 'P' as u16, '_' as u16, 'O' as u16, 'L' as u16, 'E' as u16,
    'M' as u16, 'e' as u16, 'n' as u16, 'u' as u16, 'D' as u16, 'e' as u16, 's' as u16, 'c' as u16,
    'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'o' as u16, 'r' as u16, 0,
];

pub static prop_oledroptarget: [WCHAR; 24] = [
    'O' as u16, 'l' as u16, 'e' as u16, 'D' as u16, 'r' as u16, 'o' as u16, 'p' as u16, 'T' as u16,
    'a' as u16, 'r' as u16, 'g' as u16, 'e' as u16, 't' as u16, 'I' as u16, 'n' as u16, 't' as u16,
    'e' as u16, 'r' as u16, 'f' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 0,
];

pub static prop_marshalleddroptarget: [WCHAR; 26] = [
    'W' as u16, 'i' as u16, 'n' as u16, 'e' as u16, 'M' as u16, 'a' as u16, 'r' as u16, 's' as u16,
    'h' as u16, 'a' as u16, 'l' as u16, 'l' as u16, 'e' as u16, 'd' as u16, 'D' as u16, 'r' as u16,
    'o' as u16, 'p' as u16, 'T' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'e' as u16, 't' as u16,
    0,
];

pub static emptyW: [WCHAR; 1] = [0];

#[repr(C)]
pub struct DropTargetWrapper {
    pub IDropTarget_iface: IDropTarget,
    pub hwnd: HWND,
    pub refs: LONG,
}

pub static DropTargetWrapperVTbl: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: DropTargetWrapper_QueryInterface,
    AddRef: DropTargetWrapper_AddRef,
    Release: DropTargetWrapper_Release,
    DragEnter: DropTargetWrapper_DragEnter,
    DragOver: DropTargetWrapper_DragOver,
    DragLeave: DropTargetWrapper_DragLeave,
    Drop: DropTargetWrapper_Drop,
};

#[repr(C)]
pub struct EnumOLEVERB {
    pub IEnumOLEVERB_iface: IEnumOLEVERB,
    pub ref_: LONG,
    pub hkeyVerb: HKEY,
    pub index: ULONG,
}

pub static EnumOLEVERB_VTable: IEnumOLEVERBVtbl = IEnumOLEVERBVtbl {
    QueryInterface: EnumOLEVERB_QueryInterface,
    AddRef: EnumOLEVERB_AddRef,
    Release: EnumOLEVERB_Release,
    Next: EnumOLEVERB_Next,
    Skip: EnumOLEVERB_Skip,
    Reset: EnumOLEVERB_Reset,
    Clone: EnumOLEVERB_Clone,
};

pub const DRAG_TIMER_ID: u32 = 1;

#[inline]

#[repr(C)]
struct COM_CurrentInfoStruct {
    ole_inits: i32,
}

#[no_mangle]
pub unsafe extern "system" fn OleBuildVersion() -> DWORD {
    TRACE!(b"Returning version %d, build %d.\n\0".as_ptr() as PCHAR, rmm, rup);
    (rmm << 16) + rup
}

#[no_mangle]
pub unsafe extern "system" fn OleInitialize(reserved: LPVOID) -> HRESULT {

    const COINIT_APARTMENTTHREADED: u32 = 2;
    const S_OK: HRESULT = 0;
    const S_FALSE: HRESULT = 1;

    TRACE!(b"(%p)\n\0".as_ptr(), reserved);

    let hr = CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);

    if FAILED(hr) != 0 {
        return hr;
    }

    let com_info = COM_CurrentInfo() as *mut ComCurrentInfo;
    let hr = if (*com_info).ole_inits == 0 {
        S_OK
    } else {
        S_FALSE
    };

    (*com_info).ole_inits += 1;
    if (*com_info).ole_inits == 1 && InterlockedIncrement(&mut OLE_moduleLockCount) == 1 {
        TRACE!(b"() - Initializing the OLE libraries\n\0".as_ptr());

        OLEClipbrd_Initialize();
        OLEDD_Initialize();
        OLEMenu_Initialize();
    }

    hr
}

#[repr(C)]
struct ComCurrentInfo {
    ole_inits: u32,
}

#[no_mangle]
pub unsafe extern "system" fn OleUninitialize() {
    TRACE!(b"()\n\0".as_ptr());

    let current_info = COM_CurrentInfo() as *mut COM_CurrentInfoStruct;

    if (*current_info).ole_inits == 0 {
        WARN!(b"ole_inits is already 0\n\0".as_ptr());
        return;
    }

    (*current_info).ole_inits -= 1;

    if (*current_info).ole_inits == 0 && InterlockedDecrement(core::ptr::addr_of_mut!(OLE_moduleLockCount)) == 0 {
        TRACE!(b"() - Freeing the last reference count\n\0".as_ptr());

        OLEClipbrd_UnInitialize();

        OLEMenu_UnInitialize();
    }

    CoUninitialize();
}

#[no_mangle]
pub unsafe extern "system" fn OleInitializeWOW(x: u32, y: u32) -> HRESULT {
    FIXME!(b"(0x%08x, 0x%08x),stub!\n\0".as_ptr() as *const i8, x, y);
    return 0;
}

unsafe fn get_droptarget_handle(hwnd: HWND) -> HANDLE {
    GetPropW(hwnd, prop_marshalleddroptarget.as_ptr())
}

unsafe fn is_droptarget(hwnd: HWND) -> BOOL {
    (get_droptarget_handle(hwnd) != 0) as BOOL
}

unsafe fn is_acceptfiles(hwnd: HWND) -> BOOL {
    (if (GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_ACCEPTFILES) != 0 { 1 } else { 0 })
}

unsafe fn get_droptarget_local_handle(hwnd: HWND) -> HANDLE {
    let mut local_handle: HANDLE = core::ptr::null_mut();

    let handle = get_droptarget_handle(hwnd);

    if !handle.is_null() {
        let mut pid: DWORD = 0;
        let process: HANDLE;

        GetWindowThreadProcessId(hwnd, &mut pid);
        process = OpenProcess(PROCESS_DUP_HANDLE, 0u8, pid);
        if !process.is_null() {
            DuplicateHandle(
                process,
                handle,
                GetCurrentProcess(),
                &mut local_handle,
                0,
                0u8,
                DUPLICATE_SAME_ACCESS,
            );
            CloseHandle(process);
        }
    }
    local_handle
}

unsafe fn create_map_from_stream(stream: *mut IStream, map: *mut HANDLE) -> HRESULT {
    let mut hmem: HGLOBAL = core::ptr::null_mut();
    let size: DWORD;
    let hr: HRESULT;
    let data: *mut core::ffi::c_void;

    hr = GetHGlobalFromStream(stream, &mut hmem);
    if FAILED(hr) != 0 {
        return hr;
    }

    size = GlobalSize(hmem);
    *map = CreateFileMappingW(
        INVALID_HANDLE_VALUE,
        core::ptr::null_mut(),
        PAGE_READWRITE,
        0,
        size,
        core::ptr::null(),
    );
    if (*map).is_null() {
        return E_OUTOFMEMORY;
    }

    data = MapViewOfFile(*map, FILE_MAP_WRITE, 0, 0, size);
    memcpy(data, GlobalLock(hmem), size as usize);
    GlobalUnlock(hmem);
    UnmapViewOfFile(data);
    S_OK
}

unsafe fn create_stream_from_map(map: HANDLE, stream: *mut *mut IStream) -> HRESULT {
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut hmem: HGLOBAL;
    let mut data: LPVOID;
    let mut info: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

    data = MapViewOfFile(map, FILE_MAP_READ, 0, 0, 0);
    if data.is_null() {
        return hr;
    }

    VirtualQuery(data, &mut info, core::mem::size_of::<MEMORY_BASIC_INFORMATION>());
    TRACE!(b"size %d\n\0".as_ptr() as PCSTR, info.RegionSize as i32);

    hmem = GlobalAlloc(GMEM_MOVEABLE, info.RegionSize);
    if !hmem.is_null() {
        core::ptr::copy_nonoverlapping(
            data as *const u8,
            GlobalLock(hmem) as *mut u8,
            info.RegionSize
        );
        GlobalUnlock(hmem);
        hr = CreateStreamOnHGlobal(hmem, 1, stream);
    }
    UnmapViewOfFile(data);
    hr
}

unsafe fn impl_from_IDropTarget(iface: *mut IDropTarget) -> *mut DropTargetWrapper {
    (iface as *mut u8).sub(core::mem::offset_of!(DropTargetWrapper, IDropTarget_iface)) as *mut DropTargetWrapper
}
unsafe fn DropTargetWrapper_QueryInterface(
    iface: *mut IDropTarget,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IDropTarget(iface);
    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_IDropTarget) != 0 {
        IDropTarget_AddRef(core::ptr::addr_of_mut!((*This).IDropTarget_iface));
        *ppvObject = core::ptr::addr_of_mut!((*This).IDropTarget_iface) as *mut core::ffi::c_void;
        return S_OK;
    }
    *ppvObject = core::ptr::null_mut();
    E_NOINTERFACE
}

unsafe fn DropTargetWrapper_AddRef(iface: *mut IDropTarget) -> ULONG {
    let This = impl_from_IDropTarget(iface);
    InterlockedIncrement(&mut (*This).refs) as ULONG
}

unsafe fn DropTargetWrapper_Release(iface: *mut IDropTarget) -> ULONG {
    let This = impl_from_IDropTarget(iface);
    let refs = InterlockedDecrement(&mut (*This).refs);
    if refs == 0 {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    refs as ULONG
}

unsafe fn get_target_from_wrapper(wrapper: *mut IDropTarget, target: *mut *mut IDropTarget) -> HRESULT {
    let This = impl_from_IDropTarget(wrapper);
    *target = GetPropW((*This).hwnd, prop_oledroptarget.as_ptr()) as *mut IDropTarget;
    if (*target).is_null() {
        return DRAGDROP_E_NOTREGISTERED;
    }
    IDropTarget_AddRef(*target);
    return S_OK;
}

unsafe fn DropTargetWrapper_DragEnter(
    iface: *mut IDropTarget,
    pDataObj: *mut IDataObject,
    grfKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut target: *mut IDropTarget = core::ptr::null_mut();
    let r = get_target_from_wrapper(iface, &mut target);

    if SUCCEEDED(r) != 0 {
        let r2 = IDropTarget_DragEnter(target, pDataObj, grfKeyState, pt, pdwEffect);
        IDropTarget_Release(target);
        return r2;
    }
    r
}
unsafe fn DropTargetWrapper_DragOver(
    iface: *mut IDropTarget,
    grfKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut target: *mut IDropTarget = core::ptr::null_mut();
    let r = get_target_from_wrapper(iface, &mut target);

    if SUCCEEDED(r) != 0 {
        let r = IDropTarget_DragOver(target, grfKeyState, pt, pdwEffect);
        IDropTarget_Release(target);
        return r;
    }
    r
}

unsafe fn DropTargetWrapper_DragLeave(iface: *mut IDropTarget) -> HRESULT {
    let mut target: *mut IDropTarget = core::ptr::null_mut();
    let r = get_target_from_wrapper(iface, &mut target);

    if SUCCEEDED(r) != 0 {
        let r = IDropTarget_DragLeave(target);
        IDropTarget_Release(target);
        return r;
    }
    r
}
unsafe fn DropTargetWrapper_Drop(
    iface: *mut IDropTarget,
    pDataObj: *mut IDataObject,
    grfKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut target: *mut IDropTarget = core::ptr::null_mut();
    let mut r = get_target_from_wrapper(iface, &mut target);

    if SUCCEEDED(r) != 0 {
        r = IDropTarget_Drop(target, pDataObj, grfKeyState, pt, pdwEffect);
        IDropTarget_Release(target);
    }
    r
}

unsafe fn WrapDropTarget(hwnd: HWND) -> *mut IDropTarget {

    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DropTargetWrapper>(),
    ) as *mut DropTargetWrapper;

    if !This.is_null() {
        (*This).IDropTarget_iface.lpVtbl = &DropTargetWrapperVTbl;
        (*This).hwnd = hwnd;
        (*This).refs = 1;
    }
    core::ptr::addr_of_mut!((*This).IDropTarget_iface)
}

unsafe fn get_droptarget_pointer(hwnd: HWND) -> *mut IDropTarget {
    let mut droptarget: *mut IDropTarget = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();

    let map = get_droptarget_local_handle(hwnd);
    if map.is_null() {
        return core::ptr::null_mut();
    }

    if SUCCEEDED(create_stream_from_map(map, &mut stream)) != 0 {
        CoUnmarshalInterface(stream, &IID_IDropTarget, &mut droptarget as *mut *mut IDropTarget as *mut LPVOID);
        IStream_Release(stream);
    }
    CloseHandle(map);
    droptarget
}

#[no_mangle]
pub unsafe extern "system" fn RegisterDragDrop(hwnd: HWND, pDropTarget: LPDROPTARGET) -> HRESULT {

    const IID_IDropTarget: CLSID = CLSID {
        Data1: 0x00000122,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    const MSHCTX_LOCAL: DWORD = 0;
    const MSHLFLAGS_TABLESTRONG: DWORD = 1;
    const STREAM_SEEK_SET: DWORD = 0;
    const E_OUTOFMEMORY: HRESULT = 0x8007000E_u32 as HRESULT;
    const E_INVALIDARG: HRESULT = 0x80070057_u32 as HRESULT;
    const DRAGDROP_E_INVALIDHWND: HRESULT = 0x80040010_u32 as HRESULT;
    const DRAGDROP_E_ALREADYREGISTERED: HRESULT = 0x80040101_u32 as HRESULT;

    let mut pid: DWORD = 0;
    let hr: HRESULT;
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut map: HANDLE = core::ptr::null_mut();
    let wrapper: *mut IDropTarget;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, hwnd, pDropTarget);

    if COM_CurrentApt().is_null() {
        ERR!(b"COM not initialized\n\0".as_ptr() as *const i8);
        return E_OUTOFMEMORY;
    }

    if pDropTarget.is_null() {
        return E_INVALIDARG;
    }

    if IsWindow(hwnd) == 0 {
        ERR!(b"invalid hwnd %p\n\0".as_ptr() as *const i8, hwnd);
        return DRAGDROP_E_INVALIDHWND;
    }

    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != GetCurrentProcessId() {
        FIXME!(b"register for another process windows is disabled\n\0".as_ptr() as *const i8);
        return DRAGDROP_E_INVALIDHWND;
    }

    if is_droptarget(hwnd) != 0 {
        return DRAGDROP_E_ALREADYREGISTERED;
    }

    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut stream);
    if FAILED(hr) != 0 {
        return hr;
    }

    wrapper = WrapDropTarget(hwnd);
    if wrapper.is_null() {
        IStream_Release(stream);
        return E_OUTOFMEMORY;
    }
    hr = CoMarshalInterface(stream, &IID_IDropTarget, wrapper as *mut IUnknown, MSHCTX_LOCAL, core::ptr::null_mut(), MSHLFLAGS_TABLESTRONG);
    IDropTarget_Release(wrapper);

    if SUCCEEDED(hr) != 0 {
        hr = create_map_from_stream(stream, &mut map);
        if SUCCEEDED(hr) != 0 {
            IDropTarget_AddRef(pDropTarget);
            SetPropW(hwnd, prop_oledroptarget.as_ptr(), pDropTarget as HANDLE);
            SetPropW(hwnd, prop_marshalleddroptarget.as_ptr(), map);
        } else {
            let mut zero: LARGE_INTEGER = core::mem::zeroed();
            (*(&mut zero as *mut LARGE_INTEGER as *mut i64)) = 0;
            IStream_Seek(stream, zero, STREAM_SEEK_SET, core::ptr::null_mut());
            CoReleaseMarshalData(stream);
        }
    }
    IStream_Release(stream);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn RevokeDragDrop(hwnd: HWND) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), hwnd);

    if IsWindow(hwnd) == 0 {
        ERR!(b"invalid hwnd %p\n\0".as_ptr(), hwnd);
        return 0x80040010u32 as HRESULT; // DRAGDROP_E_INVALIDHWND
    }

    let map = get_droptarget_handle(hwnd);
    if map.is_null() {
        return 0x80040011u32 as HRESULT; // DRAGDROP_E_NOTREGISTERED
    }

    let drop_target = GetPropW(hwnd, prop_oledroptarget.as_ptr()) as *mut IDropTarget;
    if !drop_target.is_null() {
        IDropTarget_Release(drop_target);
    }

    RemovePropW(hwnd, prop_oledroptarget.as_ptr());
    RemovePropW(hwnd, prop_marshalleddroptarget.as_ptr());

    let mut stream: *mut IStream = core::ptr::null_mut();
    let hr = create_stream_from_map(map, &mut stream);
    if SUCCEEDED(hr) != 0 {
        CoReleaseMarshalData(stream);
        IStream_Release(stream);
    }
    CloseHandle(map);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleRegGetUserType(
    clsid: *const CLSID,
    form: DWORD,
    usertype: *mut LPOLESTR,
) -> HRESULT {

    static auxusertypeW: [u16; 15] = [
        b'A' as u16, b'u' as u16, b'x' as u16, b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16,
        b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, b'\\' as u16, b'%' as u16, b'd' as u16,
        0,
    ];

    let mut valuetype: DWORD = 0;
    let mut valuelen: DWORD = 0;
    let mut auxkeynameW: [u16; 16] = [0; 16];
    let mut usertypekey: HKEY = core::ptr::null_mut();
    let mut hres: HRESULT;
    let ret: i32;

    TRACE!(
        b"(%s, %u, %p)\n\0".as_ptr(),
        debugstr_guid(clsid),
        form,
        usertype,
    );

    if usertype.is_null() {
        return E_INVALIDARG;
    }

    *usertype = core::ptr::null_mut();

    hres = COM_OpenKeyForCLSID(clsid, core::ptr::null(), KEY_READ, &mut usertypekey);
    if FAILED(hres) != 0 {
        return hres;
    }

    valuelen = 0;

    if form != USERCLASSTYPE_FULL {
        let mut auxkey: HKEY = core::ptr::null_mut();

        swprintf(
            auxkeynameW.as_mut_ptr(),
            auxusertypeW.as_ptr(),
            form,
        );

        if COM_OpenKeyForCLSID(clsid, auxkeynameW.as_ptr(), KEY_READ, &mut auxkey) == S_OK {
            if RegQueryValueExW(
                auxkey,
                emptyW.as_ptr(),
                core::ptr::null_mut(),
                &mut valuetype,
                core::ptr::null_mut(),
                &mut valuelen,
            ) == 0
                && valuelen != 0
            {
                RegCloseKey(usertypekey);
                usertypekey = auxkey;
            } else {
                RegCloseKey(auxkey);
            }
        }
    }

    valuelen = 0;
    if RegQueryValueExW(
        usertypekey,
        emptyW.as_ptr(),
        core::ptr::null_mut(),
        &mut valuetype,
        core::ptr::null_mut(),
        &mut valuelen,
    ) != 0
    {
        RegCloseKey(usertypekey);
        return REGDB_E_READREGDB;
    }

    *usertype = CoTaskMemAlloc(valuelen as usize) as LPOLESTR;
    if (*usertype).is_null() {
        RegCloseKey(usertypekey);
        return E_OUTOFMEMORY;
    }

    ret = RegQueryValueExW(
        usertypekey,
        emptyW.as_ptr(),
        core::ptr::null_mut(),
        &mut valuetype,
        *usertype as *mut u8,
        &mut valuelen,
    );
    RegCloseKey(usertypekey);
    if ret != ERROR_SUCCESS as i32 {
        CoTaskMemFree(*usertype as LPVOID);
        *usertype = core::ptr::null_mut();
        return REGDB_E_READREGDB;
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn DoDragDrop(
    pDataObject: *mut IDataObject,
    pDropSource: *mut IDropSource,
    dwOKEffect: DWORD,
    pdwEffect: *mut DWORD,
) -> HRESULT {

    static trackerW: [u16; 14] = [
        'T' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 'k' as u16, 'e' as u16, 'r' as u16,
        'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 0,
    ];

    TRACE!(
        b"(%p, %p, %08x, %p)\n\0".as_ptr() as LPCSTR,
        pDataObject,
        pDropSource,
        dwOKEffect,
        pdwEffect,
    );

    if pDataObject.is_null() || pDropSource.is_null() || pdwEffect.is_null() {
        return E_INVALIDARG;
    }

    let mut trackerInfo: TrackerWindowInfo = core::mem::zeroed();
    trackerInfo.dataObject = pDataObject;
    trackerInfo.dropSource = pDropSource;
    trackerInfo.dwOKEffect = dwOKEffect;
    trackerInfo.pdwEffect = pdwEffect;
    trackerInfo.trackingDone = 0u8;
    trackerInfo.escPressed = 0u8;
    trackerInfo.curTargetHWND = core::ptr::null_mut();
    trackerInfo.curDragTarget = core::ptr::null_mut();
    trackerInfo.accepterHWND = core::ptr::null_mut();

    let hwndTrackWindow = CreateWindowW(
        OLEDD_DRAGTRACKERCLASS.as_ptr(),
        trackerW.as_ptr(),
        WS_POPUP,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut trackerInfo as *mut _ as LPVOID,
    );

    if !hwndTrackWindow.is_null() {
        SetCapture(hwndTrackWindow);

        let mut msg: MSG = core::mem::zeroed();
        msg.message = 0;

        while trackerInfo.trackingDone == 0
            && GetMessageW(
                &mut msg,
                core::ptr::null_mut(),
                0,
                0,
            ) != 0
        {
            trackerInfo.curMousePos.x = msg.pt.x;
            trackerInfo.curMousePos.y = msg.pt.y;
            trackerInfo.dwKeyState = OLEDD_GetButtonState();

            if msg.message >= WM_KEYFIRST && msg.message <= WM_KEYLAST {
                if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE as WPARAM {
                    trackerInfo.escPressed = 1u8;
                }

                OLEDD_TrackStateChange(&mut trackerInfo);
            } else {
                DispatchMessageW(&msg);
            }
        }

        if msg.message == WM_QUIT {
            PostQuitMessage(msg.wParam as c_int);
        }

        DestroyWindow(hwndTrackWindow);

        return trackerInfo.returnValue;
    }

    E_FAIL
}

#[no_mangle]
pub unsafe extern "system" fn OleQueryLinkFromData(
    pSrcDataObject: *mut IDataObject
) -> HRESULT {
    fixme_ole(b"(%p),stub!\n\0".as_ptr() as *const i8, pSrcDataObject);
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn OleRegGetMiscStatus(
    clsid: *const CLSID,
    dwAspect: DWORD,
    pdwStatus: *mut DWORD,
) -> HRESULT {

    static miscstatusW: [u16; 11] = [
        b'M' as u16, b'i' as u16, b's' as u16, b'c' as u16, b'S' as u16, b't' as u16, b'a' as u16,
        b't' as u16, b'u' as u16, b's' as u16, 0,
    ];
    static dfmtW: [u16; 3] = [b'%' as u16, b'd' as u16, 0];

    let mut keyName: [u16; 16] = [0; 16];
    let mut miscStatusKey: HKEY = core::ptr::null_mut();
    let mut aspectKey: HKEY = core::ptr::null_mut();
    let mut result: LONG;
    let mut hr: HRESULT;

    TRACE!(
        b"(%s, %d, %p)\n\0".as_ptr(),
        debugstr_guid(clsid),
        dwAspect,
        pdwStatus,
    );

    if pdwStatus.is_null() {
        return 0x80070057u32 as i32;
    }

    *pdwStatus = 0;

    if actctx_get_miscstatus(clsid, dwAspect, pdwStatus) != 0 {
        return 0;
    }

    hr = COM_OpenKeyForCLSID(clsid, miscstatusW.as_ptr(), KEY_READ, &mut miscStatusKey);
    if FAILED(hr) != 0 {
        return if hr == 0x80040152u32 as i32 { 0 } else { hr };
    }

    OLEUTL_ReadRegistryDWORDValue(miscStatusKey, pdwStatus);

    swprintf(keyName.as_mut_ptr(), dfmtW.as_ptr(), dwAspect);

    result = open_classes_key(miscStatusKey, keyName.as_ptr(), KEY_READ, &mut aspectKey);
    if result == ERROR_SUCCESS as i32 {
        OLEUTL_ReadRegistryDWORDValue(aspectKey, pdwStatus);
        RegCloseKey(aspectKey);
    }

    RegCloseKey(miscStatusKey);
    0
}

unsafe fn impl_from_IEnumOLEVERB(iface: *mut IEnumOLEVERB) -> *mut EnumOLEVERB {
    (iface as *mut u8).offset(-(core::mem::offset_of!(EnumOLEVERB, IEnumOLEVERB_iface) as isize)) as *mut EnumOLEVERB
}
unsafe fn EnumOLEVERB_QueryInterface(
    iface: *mut IEnumOLEVERB,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(b"(%s, %p)\n\0".as_ptr(), debugstr_guid(riid), ppv);
    if IsEqualIID(riid, &IID_IUnknown) != 0 ||
       IsEqualIID(riid, &IID_IEnumOLEVERB) != 0
    {
        IEnumOLEVERB_AddRef(iface);
        *ppv = iface as *mut core::ffi::c_void;
        return S_OK;
    }
    return E_NOINTERFACE;
}

unsafe fn EnumOLEVERB_AddRef(iface: *mut IEnumOLEVERB) -> ULONG {
    let This = impl_from_IEnumOLEVERB(iface);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn EnumOLEVERB_Release(iface: *mut IEnumOLEVERB) -> ULONG {
    let This = impl_from_IEnumOLEVERB(iface);
    let refs = InterlockedDecrement(&mut (*This).ref_);
    TRACE!(b"()\n\0".as_ptr() as *const i8);
    if refs == 0 {
        RegCloseKey((*This).hkeyVerb);
        HeapFree(GetProcessHeap(), 0, This as *mut _);
    }
    refs as ULONG
}

unsafe fn EnumOLEVERB_Next(
    iface: *mut IEnumOLEVERB,
    celt: ULONG,
    rgelt: *mut OLEVERB,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let this = impl_from_IEnumOLEVERB(iface);
    let mut hr: HRESULT = S_OK;

    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }

    let mut celt_remaining = celt;
    let mut rgelt_ptr = rgelt;

    while celt_remaining > 0 {
        let mut wszSubKey: [u16; 20] = [0; 20];
        let mut cbData: LONG = 0;

        let res = RegEnumKeyW(
            (*this).hkeyVerb,
            (*this).index,
            wszSubKey.as_mut_ptr(),
            20,
        );

        if res == ERROR_NO_MORE_ITEMS as LONG {
            hr = S_FALSE;
            break;
        } else if res != ERROR_SUCCESS as LONG {
            hr = REGDB_E_READREGDB;
            break;
        }

        let res = RegQueryValueW(
            (*this).hkeyVerb,
            wszSubKey.as_ptr(),
            core::ptr::null_mut(),
            &mut cbData,
        );
        if res != ERROR_SUCCESS as LONG {
            hr = REGDB_E_READREGDB;
            break;
        }

        let pwszOLEVERB = CoTaskMemAlloc(cbData as usize) as LPWSTR;
        if pwszOLEVERB.is_null() {
            hr = E_OUTOFMEMORY;
            break;
        }

        let res = RegQueryValueW(
            (*this).hkeyVerb,
            wszSubKey.as_ptr(),
            pwszOLEVERB,
            &mut cbData,
        );
        if res != ERROR_SUCCESS as LONG {
            hr = REGDB_E_READREGDB;
            CoTaskMemFree(pwszOLEVERB as LPVOID);
            break;
        }

        let pwszMenuFlags = wcschr(pwszOLEVERB, ',' as u16);
        if pwszMenuFlags.is_null() {
            hr = OLEOBJ_E_INVALIDVERB;
            CoTaskMemFree(pwszOLEVERB as LPVOID);
            break;
        }
        *pwszMenuFlags = 0;
        let pwszMenuFlags = pwszMenuFlags.offset(1);

        let pwszAttribs = wcschr(pwszMenuFlags, ',' as u16);
        if pwszAttribs.is_null() {
            hr = OLEOBJ_E_INVALIDVERB;
            CoTaskMemFree(pwszOLEVERB as LPVOID);
            break;
        }
        *pwszAttribs = 0;
        let pwszAttribs = pwszAttribs.offset(1);

        (*rgelt_ptr).lVerb = wcstol(wszSubKey.as_ptr(), core::ptr::null_mut(), 10);
        (*rgelt_ptr).lpszVerbName = pwszOLEVERB;
        (*rgelt_ptr).fuFlags = wcstol(pwszMenuFlags, core::ptr::null_mut(), 10) as u32;
        (*rgelt_ptr).grfAttribs = wcstol(pwszAttribs, core::ptr::null_mut(), 10) as u32;

        if !pceltFetched.is_null() {
            *pceltFetched += 1;
        }
        (*this).index += 1;

        celt_remaining -= 1;
        rgelt_ptr = rgelt_ptr.offset(1);
    }

    hr
}
unsafe fn EnumOLEVERB_Skip(
    iface: *mut IEnumOLEVERB,
    celt: ULONG,
) -> HRESULT {
    let This = impl_from_IEnumOLEVERB(iface);

    TRACE!(b"(%d)\n\0".as_ptr(), celt);

    (*This).index = (*This).index.wrapping_add(celt);
    S_OK
}
unsafe fn EnumOLEVERB_Reset(
    iface: *mut IEnumOLEVERB
) -> HRESULT {
    let This = impl_from_IEnumOLEVERB(iface);

    TRACE!(b"()\n\0".as_ptr() as LPCSTR);

    (*This).index = 0;
    S_OK
}
unsafe fn EnumOLEVERB_Clone(
    iface: *mut IEnumOLEVERB,
    ppenum: *mut *mut IEnumOLEVERB) -> HRESULT {
    let This = impl_from_IEnumOLEVERB(iface);
    let mut hkeyVerb: HKEY = core::ptr::null_mut();
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, ppenum);
    if DuplicateHandle(GetCurrentProcess(), (*This).hkeyVerb as HANDLE, GetCurrentProcess(), &mut hkeyVerb as *mut HKEY as *mut HANDLE, 0, 0, DUPLICATE_SAME_ACCESS) == 0
    {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    return EnumOLEVERB_Construct(hkeyVerb, (*This).index, ppenum);
}

unsafe fn EnumOLEVERB_Construct(hkeyVerb: HKEY, index: ULONG, ppenum: *mut *mut IEnumOLEVERB) -> HRESULT {

    let This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<EnumOLEVERB>()) as *mut EnumOLEVERB;
    if This.is_null() {
        RegCloseKey(hkeyVerb);
        return E_OUTOFMEMORY;
    }
    (*This).IEnumOLEVERB_iface.lpVtbl = &EnumOLEVERB_VTable;
    (*This).ref_ = 1;
    (*This).index = index;
    (*This).hkeyVerb = hkeyVerb;
    *ppenum = &mut (*This).IEnumOLEVERB_iface;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn OleRegEnumVerbs(clsid: *const CLSID, ppenum: *mut *mut IEnumOLEVERB) -> HRESULT {

    static WSZVERB: [u16; 5] = [b'V' as u16, b'e' as u16, b'r' as u16, b'b' as u16, 0];

    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_guid(clsid), ppenum);

    let mut hkeyVerb: HKEY = core::ptr::null_mut();
    let res = COM_OpenKeyForCLSID(clsid, WSZVERB.as_ptr(), KEY_READ, core::ptr::addr_of_mut!(hkeyVerb));
    if FAILED(res) != 0 {
        if res == 0x80040154u32 as i32 {
            ERR!(b"CLSID %s not registered\n\0".as_ptr() as *const i8, debugstr_guid(clsid));
        } else if res == 0x80040152u32 as i32 {
            ERR!(b"no Verbs key for class %s\n\0".as_ptr() as *const i8, debugstr_guid(clsid));
        } else {
            ERR!(b"failed to open Verbs key for CLSID %s with error %d\n\0".as_ptr() as *const i8,
                debugstr_guid(clsid), res);
        }
        return res;
    }

    let mut dwSubKeys: u32 = 0;
    let res = RegQueryInfoKeyW(hkeyVerb, core::ptr::null_mut(), core::ptr::null_mut(),
                               core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSubKeys),
                               core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                               core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                               core::ptr::null_mut());
    if res != ERROR_SUCCESS as i32 {
        ERR!(b"failed to get subkey count with error %d\n\0".as_ptr() as *const i8, GetLastError());
        return 0x80040150u32 as i32;
    }

    if dwSubKeys == 0 {
        WARN!(b"class %s has no verbs\n\0".as_ptr() as *const i8, debugstr_guid(clsid));
        RegCloseKey(hkeyVerb);
        return 0x80040180u32 as i32;
    }

    return EnumOLEVERB_Construct(hkeyVerb, 0, ppenum);
}

#[no_mangle]
pub unsafe extern "system" fn OleSetContainedObject(
    pUnknown: LPUNKNOWN,
    fContained: BOOL,
) -> HRESULT {
    let mut runnable: *mut IRunnableObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(b"(%p,%x)\n\0".as_ptr() as PCHAR, pUnknown, fContained);

    hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runnable as *mut *mut IRunnableObject as *mut LPVOID,
    );

    if SUCCEEDED(hres) != 0 {
        hres = IRunnableObject_SetContainedObject(runnable, fContained);

        IRunnableObject_Release(runnable);

        return hres;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn OleRun(pUnknown: LPUNKNOWN) -> HRESULT {
    let mut runable: *mut IRunnableObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, pUnknown);

    hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runable as *mut *mut IRunnableObject as *mut LPVOID
    );
    if FAILED(hres) {
        return S_OK;
    }

    hres = IRunnableObject_Run(runable, core::ptr::null_mut());
    IRunnableObject_Release(runable);
    hres
}

#[no_mangle]
pub unsafe extern "system" fn OleLoad(
    pStg: *mut IStorage,
    riid: *const IID,
    pClientSite: *mut IOleClientSite,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut persistStorage: *mut IPersistStorage = core::ptr::null_mut();
    let mut pUnk: *mut IUnknown;
    let mut pOleObject: *mut IOleObject = core::ptr::null_mut();
    let mut storageInfo: STATSTG = core::mem::zeroed();
    let mut hres: HRESULT;

    TRACE!(
        b"(%p, %s, %p, %p)\n\0".as_ptr() as PCSTR,
        pStg,
        debugstr_guid(riid),
        pClientSite,
        ppvObj,
    );

    *ppvObj = core::ptr::null_mut();

    hres = IStorage_Stat(pStg, &mut storageInfo, STATFLAG_NONAME);
    if FAILED(hres) != 0 {
        return hres;
    }

    hres = CoCreateInstance(
        &storageInfo.clsid,
        core::ptr::null_mut(),
        CLSCTX_INPROC_HANDLER | CLSCTX_INPROC_SERVER,
        riid,
        &mut pUnk as *mut *mut IUnknown as *mut *mut core::ffi::c_void,
    );

    if FAILED(hres) != 0 {
        hres = OleCreateDefaultHandler(
            &storageInfo.clsid,
            core::ptr::null_mut(),
            riid,
            &mut pUnk as *mut *mut IUnknown as *mut *mut core::ffi::c_void,
        );
    }

    if FAILED(hres) != 0 {
        return hres;
    }

    if !pClientSite.is_null() {
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleObject,
            &mut pOleObject as *mut *mut IOleObject as *mut *mut core::ffi::c_void,
        );
        if SUCCEEDED(hres) != 0 {
            let mut dwStatus: DWORD = 0;
            hres = IOleObject_GetMiscStatus(pOleObject, DVASPECT_CONTENT, &mut dwStatus);
        }
    }

    hres = IUnknown_QueryInterface(
        pUnk,
        &IID_IPersistStorage,
        &mut persistStorage as *mut *mut IPersistStorage as *mut *mut core::ffi::c_void,
    );
    if SUCCEEDED(hres) != 0 {
        hres = IPersistStorage_Load(persistStorage, pStg);

        IPersistStorage_Release(persistStorage);
        persistStorage = core::ptr::null_mut();
    }

    if SUCCEEDED(hres) != 0 && !pClientSite.is_null() {
        hres = IOleObject_SetClientSite(pOleObject, pClientSite);
    }

    if !pOleObject.is_null() {
        IOleObject_Release(pOleObject);
    }

    if SUCCEEDED(hres) != 0 {
        let mut pOleLink: *mut IOleLink = core::ptr::null_mut();
        let hres1: HRESULT;
        hres1 = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleLink,
            &mut pOleLink as *mut *mut IOleLink as *mut *mut core::ffi::c_void,
        );
        if SUCCEEDED(hres1) != 0 {
            FIXME!(b"handle OLE link\n\0".as_ptr() as PCSTR);
            IOleLink_Release(pOleLink);
        }
    }

    if FAILED(hres) != 0 {
        IUnknown_Release(pUnk);
        pUnk = core::ptr::null_mut();
    }

    *ppvObj = pUnk as *mut core::ffi::c_void;

    return hres;
}

#[no_mangle]
pub unsafe extern "system" fn OleSave(
    pPS: LPPERSISTSTORAGE,
    pStg: LPSTORAGE,
    fSameAsLoad: BOOL,
) -> HRESULT {
    let mut objectClass: CLSID = core::mem::zeroed();

    TRACE!(
        b"(%p,%p,%x)\n\0".as_ptr() as *const i8,
        pPS,
        pStg,
        fSameAsLoad,
    );

    let mut hres = IPersistStorage_GetClassID(pPS, &mut objectClass);

    if SUCCEEDED(hres) != 0 {
        WriteClassStg(pStg, &objectClass);
    }

    hres = IPersistStorage_Save(pPS, pStg, fSameAsLoad);

    if SUCCEEDED(hres) != 0 {
        IStorage_Commit(pStg, STGC_DEFAULT);
    }

    hres
}

#[no_mangle]
pub unsafe extern "system" fn OleLockRunning(
    pUnknown: LPUNKNOWN,
    fLock: BOOL,
    fLastUnlockCloses: BOOL,
) -> HRESULT {
    let mut runnable: *mut IRunnableObject = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(
        b"(%p,%x,%x)\n\0".as_ptr() as PCHAR,
        pUnknown,
        fLock,
        fLastUnlockCloses,
    );

    hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runnable as *mut *mut IRunnableObject as *mut LPVOID,
    );

    if SUCCEEDED(hres) != 0 {
        hres = IRunnableObject_LockRunning(runnable, fLock, fLastUnlockCloses);

        IRunnableObject_Release(runnable);

        return hres;
    }

    S_OK
}

unsafe fn OLEMenu_Initialize() {
}

unsafe fn OLEMenu_UnInitialize() {
}

unsafe fn OLEMenu_InstallHooks(tid: u32) -> i32 {
    let pHookItem: *mut OleMenuHookItem;

    pHookItem = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<OleMenuHookItem>()) as *mut OleMenuHookItem;
    if pHookItem.is_null() {
        return 0;
    }

    (*pHookItem).tid = tid;
    (*pHookItem).hHeap = GetProcessHeap();
    (*pHookItem).CallWndProc_hHook = core::ptr::null_mut();

    (*pHookItem).GetMsg_hHook = SetWindowsHookExW(
        WH_GETMESSAGE,
        Some(OLEMenu_GetMsgProc),
        core::ptr::null_mut(),
        GetCurrentThreadId()
    );
    if (*pHookItem).GetMsg_hHook.is_null() {
        UnhookWindowsHookEx((*pHookItem).GetMsg_hHook);
        UnhookWindowsHookEx((*pHookItem).CallWndProc_hHook);
        HeapFree((*pHookItem).hHeap, 0, pHookItem as *mut core::ffi::c_void);
        return 0;
    }

    (*pHookItem).CallWndProc_hHook = SetWindowsHookExW(
        WH_CALLWNDPROC,
        Some(OLEMenu_CallWndProc),
        core::ptr::null_mut(),
        GetCurrentThreadId()
    );
    if (*pHookItem).CallWndProc_hHook.is_null() {
        UnhookWindowsHookEx((*pHookItem).GetMsg_hHook);
        UnhookWindowsHookEx((*pHookItem).CallWndProc_hHook);
        HeapFree((*pHookItem).hHeap, 0, pHookItem as *mut core::ffi::c_void);
        return 0;
    }

    (*pHookItem).next = hook_list;
    hook_list = pHookItem;

    return 1;
}

unsafe fn OLEMenu_UnInstallHooks(tid: u32) -> i32 {

    let mut pHookItem: *mut OleMenuHookItem = core::ptr::null_mut();
    let mut ppHook: *mut *mut OleMenuHookItem = core::ptr::addr_of_mut!(hook_list);

    while !(*ppHook).is_null() {
        if (**ppHook).tid == tid {
            pHookItem = *ppHook;
            *ppHook = (*pHookItem).next;
            break;
        }
        ppHook = core::ptr::addr_of_mut!((**ppHook).next);
    }
    if pHookItem.is_null() {
        return 0;
    }

    'CLEANUP: {
        if UnhookWindowsHookEx((*pHookItem).GetMsg_hHook) == 0 {
            break 'CLEANUP;
        }
        if UnhookWindowsHookEx((*pHookItem).CallWndProc_hHook) == 0 {
            break 'CLEANUP;
        }

        HeapFree((*pHookItem).hHeap, 0, pHookItem as LPVOID);

        return 1;
    }

    HeapFree((*pHookItem).hHeap, 0, pHookItem as LPVOID);

    return 0;
}

unsafe fn OLEMenu_IsHookInstalled(tid: DWORD) -> *mut OleMenuHookItem {
    let mut pHookItem: *mut OleMenuHookItem;

    pHookItem = hook_list;
    while !pHookItem.is_null() {
        if tid == (*pHookItem).tid {
            return pHookItem;
        }
        pHookItem = (*pHookItem).next;
    }

    core::ptr::null_mut()
}

unsafe fn OLEMenu_FindMainMenuIndex(hMainMenu: HMENU, hPopupMenu: HMENU, pnPos: *mut u32) -> BOOL {

    let nItems: i32 = GetMenuItemCount(hMainMenu);
    let mut i: i32 = 0;

    while i < nItems {
        let hsubmenu: HMENU = GetSubMenu(hMainMenu, i);

        if !hsubmenu.is_null() {
            if hsubmenu == hPopupMenu {
                if !pnPos.is_null() {
                    *pnPos = i as u32;
                }
                return 1;
            } else if OLEMenu_FindMainMenuIndex(hsubmenu, hPopupMenu, core::ptr::null_mut()) != 0 {
                if !pnPos.is_null() {
                    *pnPos = i as u32;
                }
                return 1;
            }
        }

        i += 1;
    }

    0
}

unsafe fn OLEMenu_SetIsServerMenu(hmenu: HMENU, pOleMenuDescriptor: *mut OleMenuDescriptor) -> BOOL {
    let mut nPos: u32 = 0;
    let mut nWidth: u32;
    let mut i: u32;

    (*pOleMenuDescriptor).bIsServerItem = 0;

    if hmenu == (*pOleMenuDescriptor).hmenuCombined {
        return 0;
    }

    if OLEMenu_FindMainMenuIndex((*pOleMenuDescriptor).hmenuCombined, hmenu, &mut nPos) == 0 {
        return 0;
    }

    i = 0;
    nWidth = 0;
    while i < 6 {
        nWidth += (*pOleMenuDescriptor).mgw.width[i as usize] as u32;
        if nPos < nWidth {
            (*pOleMenuDescriptor).bIsServerItem = (i % 2) as i32;
            break;
        }
        i += 1;
    }

    (*pOleMenuDescriptor).bIsServerItem
}
unsafe fn OLEMenu_CallWndProc(code: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {

    let mut pMsg: LPCWPSTRUCT;
    let mut hOleMenu: HOLEMENU = 0 as HOLEMENU;
    let mut pOleMenuDescriptor: *mut OleMenuDescriptor = core::ptr::null_mut();
    let mut pHookItem: *mut OleMenuHookItem = core::ptr::null_mut();
    let fuFlags: WORD;

    TRACE!(b"%i, %04lx, %08lx\n\0".as_ptr() as LPCSTR, code, wParam, lParam);

    if HC_ACTION != code {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                return 0;
            }

            return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
        }
    }

    pMsg = lParam as LPCWPSTRUCT;

    hOleMenu = GetPropW((*pMsg).hwnd, prop_olemenuW.as_ptr()) as HOLEMENU;
    if hOleMenu.is_null() {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                return 0;
            }

            return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
        }
    }

    pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                return 0;
            }

            return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
        }
    }

    match (*pMsg).message {
        WM_INITMENU => {
            (*pOleMenuDescriptor).bIsServerItem = 0;

            SendMessageW(
                (*pOleMenuDescriptor).hwndActiveObject,
                (*pMsg).message,
                (*pMsg).wParam,
                (*pMsg).lParam,
            );
            'NEXTHOOK: loop {
                if !pOleMenuDescriptor.is_null() {
                    GlobalUnlock(hOleMenu);
                }

                pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                if pHookItem.is_null() {
                    WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                    return 0;
                }

                return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
            }
        }
        WM_INITMENUPOPUP => {
            OLEMenu_SetIsServerMenu((*pMsg).wParam as HMENU, pOleMenuDescriptor);
        }
        WM_MENUSELECT => {
            fuFlags = HIWORD((*pMsg).wParam as DWORD);
            if (fuFlags & MF_SYSMENU) != 0 {
                'NEXTHOOK: loop {
                    if !pOleMenuDescriptor.is_null() {
                        GlobalUnlock(hOleMenu);
                    }

                    pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                    if pHookItem.is_null() {
                        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                        return 0;
                    }

                    return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
                }
            } else if (fuFlags & MF_POPUP) != 0 {
                OLEMenu_SetIsServerMenu((*pMsg).lParam as HMENU, pOleMenuDescriptor);
            }
        }
        WM_DRAWITEM => {
            let lpdis: LPDRAWITEMSTRUCT = (*pMsg).lParam as LPDRAWITEMSTRUCT;
            if (*pMsg).wParam != 0 || (*lpdis).CtlType != ODT_MENU {
                'NEXTHOOK: loop {
                    if !pOleMenuDescriptor.is_null() {
                        GlobalUnlock(hOleMenu);
                    }

                    pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                    if pHookItem.is_null() {
                        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                        return 0;
                    }

                    return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
                }
            }
        }
        _ => {
            'NEXTHOOK: loop {
                if !pOleMenuDescriptor.is_null() {
                    GlobalUnlock(hOleMenu);
                }

                pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                if pHookItem.is_null() {
                    WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
                    return 0;
                }

                return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
            }
        }
    }

    if (*pOleMenuDescriptor).bIsServerItem != 0 {
        SendMessageW(
            (*pOleMenuDescriptor).hwndActiveObject,
            (*pMsg).message,
            (*pMsg).wParam,
            (*pMsg).lParam,
        );
    }

    'NEXTHOOK: loop {
        if !pOleMenuDescriptor.is_null() {
            GlobalUnlock(hOleMenu);
        }

        pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
        if pHookItem.is_null() {
            WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as LPCSTR);
            return 0;
        }

        return CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam);
    }
}

unsafe fn OLEMenu_GetMsgProc(code: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {

    let mut pMsg: LPMSG;
    let mut hOleMenu: HOLEMENU = 0 as HOLEMENU;
    let mut pOleMenuDescriptor: *mut OleMenuDescriptor = core::ptr::null_mut();
    let mut pHookItem: *mut OleMenuHookItem = core::ptr::null_mut();
    let wCode: WORD;

    TRACE!(b"%i, %04lx, %08lx\n\0".as_ptr() as *const i8, code, wParam, lParam);

    if HC_ACTION != code {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
                return 0 as LRESULT;
            }

            return CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam);
        }
    }

    pMsg = lParam as LPMSG;

    hOleMenu = GetPropW((*pMsg).hwnd, prop_olemenuW.as_ptr()) as HOLEMENU;
    if hOleMenu.is_null() {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
                return 0 as LRESULT;
            }

            return CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam);
        }
    }

    match (*pMsg).message {
        WM_COMMAND => {
            wCode = HIWORD((*pMsg).wParam as DWORD);
            if wCode != 0 {
                'NEXTHOOK: loop {
                    if !pOleMenuDescriptor.is_null() {
                        GlobalUnlock(hOleMenu);
                    }

                    pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                    if pHookItem.is_null() {
                        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
                        return 0 as LRESULT;
                    }

                    return CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam);
                }
            }
        }
        _ => {
            'NEXTHOOK: loop {
                if !pOleMenuDescriptor.is_null() {
                    GlobalUnlock(hOleMenu);
                }

                pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
                if pHookItem.is_null() {
                    WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
                    return 0 as LRESULT;
                }

                return CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam);
            }
        }
    }

    pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        'NEXTHOOK: loop {
            if !pOleMenuDescriptor.is_null() {
                GlobalUnlock(hOleMenu);
            }

            pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
            if pHookItem.is_null() {
                WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
                return 0 as LRESULT;
            }

            return CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam);
        }
    }

    if (*pOleMenuDescriptor).bIsServerItem != 0 {
        (*pMsg).hwnd = (*pOleMenuDescriptor).hwndActiveObject;
    }

    if !pOleMenuDescriptor.is_null() {
        GlobalUnlock(hOleMenu);
    }

    pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
    if pHookItem.is_null() {
        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr() as *const i8);
        return 0 as LRESULT;
    }

    CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateMenuDescriptor(
    hmenuCombined: HMENU,
    lpMenuWidths: *mut OLEMENUGROUPWIDTHS,
) -> HOLEMENU {
    let hOleMenu: HOLEMENU;
    let pOleMenuDescriptor: *mut OleMenuDescriptor;

    if hmenuCombined.is_null() || lpMenuWidths.is_null() {
        return core::ptr::null_mut();
    }

    hOleMenu = GlobalAlloc(
        GMEM_MOVEABLE | GMEM_ZEROINIT,
        core::mem::size_of::<OleMenuDescriptor>(),
    );
    if hOleMenu.is_null() {
        return core::ptr::null_mut();
    }

    pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        return core::ptr::null_mut();
    }

    for i in 0..6 {
        (*pOleMenuDescriptor).mgw.width[i] = (*lpMenuWidths).width[i];
    }

    (*pOleMenuDescriptor).hmenuCombined = hmenuCombined;
    (*pOleMenuDescriptor).bIsServerItem = 0;
    GlobalUnlock(hOleMenu);

    hOleMenu
}

#[no_mangle]
pub unsafe extern "system" fn OleDestroyMenuDescriptor(
    hmenuDescriptor: HOLEMENU,
) -> HRESULT {
    if !hmenuDescriptor.is_null() {
        GlobalFree(hmenuDescriptor as HGLOBAL);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn OleSetMenuDescriptor(
    hOleMenu: HOLEMENU,
    hwndFrame: HWND,
    hwndActiveObject: HWND,
    lpFrame: LPOLEINPLACEFRAME,
    lpActiveObject: LPOLEINPLACEACTIVEOBJECT,
) -> HRESULT {

    let mut pOleMenuDescriptor: *mut OleMenuDescriptor = core::ptr::null_mut();

    if hwndFrame.is_null() || (!hOleMenu.is_null() && hwndActiveObject.is_null()) {
        return E_INVALIDARG;
    }

    if !lpFrame.is_null() || !lpActiveObject.is_null() {
        FIXME!(
            b"(%p, %p, %p, %p, %p), Context sensitive help filtering not implemented!\n\0"
                .as_ptr() as PCHAR,
            hOleMenu,
            hwndFrame,
            hwndActiveObject,
            lpFrame,
            lpActiveObject,
        );
    }

    if !hOleMenu.is_null() {
        if !OLEMenu_IsHookInstalled(GetCurrentThreadId()).is_null() {
            return E_FAIL;
        }

        pOleMenuDescriptor = GlobalLock(hOleMenu as HGLOBAL) as *mut OleMenuDescriptor;
        if pOleMenuDescriptor.is_null() {
            return E_UNEXPECTED;
        }

        (*pOleMenuDescriptor).hwndFrame = hwndFrame;
        (*pOleMenuDescriptor).hwndActiveObject = hwndActiveObject;

        GlobalUnlock(hOleMenu as HGLOBAL);
        pOleMenuDescriptor = core::ptr::null_mut();

        SetPropW(hwndFrame, prop_olemenuW.as_ptr(), hOleMenu as HANDLE);

        if OLEMenu_InstallHooks(GetCurrentThreadId()) == 0 {
            return E_FAIL;
        }
    } else {
        if OLEMenu_UnInstallHooks(GetCurrentThreadId()) == 0 {
            return E_FAIL;
        }

        RemovePropW(hwndFrame, prop_olemenuW.as_ptr());
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IsAccelerator(
    hAccel: HACCEL,
    cAccelEntries: i32,
    lpMsg: LPMSG,
    lpwCmd: *mut u16,
) -> i32 {

    if lpMsg.is_null() {
        return 0;
    }
    if hAccel.is_null() {
        WARN_(
            b"accel\0".as_ptr() as *const i8,
            b"NULL accel handle\n\0".as_ptr() as *const i8,
        );
        return 0;
    }
    if (*lpMsg).message != 0x0100
        && (*lpMsg).message != 0x0104
        && (*lpMsg).message != 0x0106
        && (*lpMsg).message != 0x0102
    {
        return 0;
    }

    let lpAccelTbl = HeapAlloc(
        GetProcessHeap(),
        0,
        (cAccelEntries as usize) * core::mem::size_of::<ACCEL>(),
    ) as LPACCEL;
    if lpAccelTbl.is_null() {
        return 0;
    }

    if CopyAcceleratorTableW(hAccel, lpAccelTbl, cAccelEntries) != cAccelEntries {
        WARN_(
            b"accel\0".as_ptr() as *const i8,
            b"CopyAcceleratorTableW failed\n\0".as_ptr() as *const i8,
        );
        HeapFree(GetProcessHeap(), 0, lpAccelTbl as *mut core::ffi::c_void);
        return 0;
    }

    TRACE_(
        b"accel\0".as_ptr() as *const i8,
        b"hAccel=%p, cAccelEntries=%d,msg->hwnd=%p, msg->message=%04x, wParam=%08lx, lParam=%08lx\n\0".as_ptr() as *const i8,
        hAccel,
        cAccelEntries,
        (*lpMsg).hwnd,
        (*lpMsg).message,
        (*lpMsg).wParam,
        (*lpMsg).lParam,
    );

    let mut i = 0;
    while i < cAccelEntries {
        if (*lpAccelTbl.offset(i as isize)).key != (*lpMsg).wParam as u16 {
            i += 1;
            continue;
        }

        if (*lpMsg).message == 0x0102 {
            if ((*lpAccelTbl.offset(i as isize)).fVirt & FALT) == 0
                && ((*lpAccelTbl.offset(i as isize)).fVirt & FVIRTKEY) == 0
            {
                TRACE_(
                    b"accel\0".as_ptr() as *const i8,
                    b"found accel for WM_CHAR: ('%c')\n\0".as_ptr() as *const i8,
                    ((*lpMsg).wParam & 0xff) as u8 as i32,
                );
                if !lpwCmd.is_null() {
                    *lpwCmd = (*lpAccelTbl.offset(i as isize)).cmd;
                }
                HeapFree(GetProcessHeap(), 0, lpAccelTbl as *mut core::ffi::c_void);
                return 1;
            }
        } else {
            if ((*lpAccelTbl.offset(i as isize)).fVirt & FVIRTKEY) != 0 {
                TRACE_(
                    b"accel\0".as_ptr() as *const i8,
                    b"found accel for virt_key %04lx (scan %04x)\n\0".as_ptr() as *const i8,
                    (*lpMsg).wParam,
                    (((*lpMsg).lParam >> 16) & 0xff) as u32,
                );
                let mut mask = 0u8;
                if (GetKeyState(VK_SHIFT as i32) as i32 & 0x8000) != 0 {
                    mask |= FSHIFT;
                }
                if (GetKeyState(VK_CONTROL as i32) as i32 & 0x8000) != 0 {
                    mask |= FCONTROL;
                }
                if (GetKeyState(VK_MENU as i32) as i32 & 0x8000) != 0 {
                    mask |= FALT;
                }
                if mask == ((*lpAccelTbl.offset(i as isize)).fVirt & (FSHIFT | FCONTROL | FALT)) {
                    if !lpwCmd.is_null() {
                        *lpwCmd = (*lpAccelTbl.offset(i as isize)).cmd;
                    }
                    HeapFree(GetProcessHeap(), 0, lpAccelTbl as *mut core::ffi::c_void);
                    return 1;
                }
                TRACE_(
                    b"accel\0".as_ptr() as *const i8,
                    b"incorrect SHIFT/CTRL/ALT-state\n\0".as_ptr() as *const i8,
                );
            } else {
                if ((*lpMsg).lParam & 0x01000000) == 0 {
                    if ((*lpAccelTbl.offset(i as isize)).fVirt & FALT) != 0
                        && ((*lpMsg).lParam & 0x20000000) != 0
                    {
                        TRACE_(
                            b"accel\0".as_ptr() as *const i8,
                            b"found accel for Alt-%c\n\0".as_ptr() as *const i8,
                            ((*lpMsg).wParam & 0xff) as u8 as i32,
                        );
                        if !lpwCmd.is_null() {
                            *lpwCmd = (*lpAccelTbl.offset(i as isize)).cmd;
                        }
                        HeapFree(GetProcessHeap(), 0, lpAccelTbl as *mut core::ffi::c_void);
                        return 1;
                    }
                }
            }
        }
        i += 1;
    }

    WARN_(
        b"accel\0".as_ptr() as *const i8,
        b"couldn't translate accelerator key\n\0".as_ptr() as *const i8,
    );
    HeapFree(GetProcessHeap(), 0, lpAccelTbl as *mut core::ffi::c_void);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseStgMedium(pmedium: *mut STGMEDIUM) {

    match (*pmedium).tymed {
        TYMED_HGLOBAL => {
            if (*pmedium).pUnkForRelease == core::ptr::null_mut()
                && (*pmedium).__bindgen_anon_1.hGlobal != core::ptr::null_mut()
            {
                GlobalFree((*pmedium).__bindgen_anon_1.hGlobal);
            }
        }
        TYMED_FILE => {
            if (*pmedium).__bindgen_anon_1.lpszFileName != core::ptr::null_mut() {
                if (*pmedium).pUnkForRelease == core::ptr::null_mut() {
                    DeleteFileW((*pmedium).__bindgen_anon_1.lpszFileName);
                }
                CoTaskMemFree((*pmedium).__bindgen_anon_1.lpszFileName as LPVOID);
            }
        }
        TYMED_ISTREAM => {
            if (*pmedium).__bindgen_anon_1.pstm != core::ptr::null_mut() {
                IStream_Release((*pmedium).__bindgen_anon_1.pstm);
            }
        }
        TYMED_ISTORAGE => {
            if (*pmedium).__bindgen_anon_1.pstg != core::ptr::null_mut() {
                IStorage_Release((*pmedium).__bindgen_anon_1.pstg);
            }
        }
        TYMED_GDI => {
            if (*pmedium).pUnkForRelease == core::ptr::null_mut()
                && (*pmedium).__bindgen_anon_1.hBitmap != core::ptr::null_mut()
            {
                DeleteObject((*pmedium).__bindgen_anon_1.hBitmap);
            }
        }
        TYMED_MFPICT => {
            if (*pmedium).pUnkForRelease == core::ptr::null_mut()
                && (*pmedium).__bindgen_anon_1.hMetaFilePict != core::ptr::null_mut()
            {
                let pMP = GlobalLock((*pmedium).__bindgen_anon_1.hMetaFilePict) as LPMETAFILEPICT;
                DeleteMetaFile((*pMP).hMF);
                GlobalUnlock((*pmedium).__bindgen_anon_1.hMetaFilePict);
                GlobalFree((*pmedium).__bindgen_anon_1.hMetaFilePict);
            }
        }
        TYMED_ENHMF => {
            if (*pmedium).pUnkForRelease == core::ptr::null_mut()
                && (*pmedium).__bindgen_anon_1.hEnhMetaFile != core::ptr::null_mut()
            {
                DeleteEnhMetaFile((*pmedium).__bindgen_anon_1.hEnhMetaFile);
            }
        }
        TYMED_NULL | _ => {}
    }
    (*pmedium).tymed = TYMED_NULL;

    if (*pmedium).pUnkForRelease != core::ptr::null_mut() {
        IUnknown_Release((*pmedium).pUnkForRelease);
        (*pmedium).pUnkForRelease = core::ptr::null_mut();
    }
}

unsafe fn OLEDD_Initialize() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    wndClass.style = CS_GLOBALCLASS;
    wndClass.lpfnWndProc = Some(OLEDD_DragTrackerWindowProc);
    wndClass.cbClsExtra = 0;
    wndClass.cbWndExtra = core::mem::size_of::<*mut TrackerWindowInfo>() as i32;
    wndClass.hCursor = core::ptr::null_mut();
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = OLEDD_DRAGTRACKERCLASS.as_ptr();

    RegisterClassW(core::ptr::addr_of!(wndClass));
}

unsafe fn OLEDD_DragTrackerWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            let createStruct = lParam as LPCREATESTRUCTA;
            SetWindowLongPtrW(hwnd, 0, (*createStruct).lpCreateParams as LONG_PTR);
            SetTimer(hwnd, DRAG_TIMER_ID, 50, core::ptr::null_mut());
        }
        WM_TIMER | WM_MOUSEMOVE | WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP |
        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN => {
            let trackerInfo = GetWindowLongPtrA(hwnd, 0) as *mut TrackerWindowInfo;
            if (*trackerInfo).trackingDone == 0 {
                OLEDD_TrackStateChange(trackerInfo);
            }
        }
        WM_DESTROY => {
            KillTimer(hwnd, DRAG_TIMER_ID);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, uMsg, wParam, lParam)
}

unsafe fn DefaultDragEnter(
    hwndTarget: HWND,
    pDataObj: *mut IDataObject,
    grfKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut fme: FORMATETC = core::mem::zeroed();

    core::ptr::write_bytes(&mut fme as *mut FORMATETC, 0, 1);
    fme.cfFormat = CF_HDROP as u16;
    fme.ptd = core::ptr::null_mut();
    fme.dwAspect = DVASPECT_CONTENT;
    fme.lindex = -1i32;
    fme.tymed = TYMED_HGLOBAL;
    hr = ((*(*pDataObj).lpVtbl).QueryGetData)(pDataObj, &mut fme);

    *pdwEffect = if hr >= 0 { DROPEFFECT_COPY } else { DROPEFFECT_NONE };

    if *pdwEffect == DROPEFFECT_NONE {
        return DRAGDROP_S_CANCEL;
    }

    return S_OK;
}

unsafe fn DefaultDrop(
    hwndAccepter: HWND,
    pDataObj: *mut IDataObject,
    grfKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {

    let mut fme: FORMATETC = core::mem::zeroed();
    let mut stgm: STGMEDIUM = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();

    ZeroMemory(&mut fme as *mut _ as PVOID, core::mem::size_of::<FORMATETC>());
    fme.cfFormat = CF_HDROP;
    fme.ptd = core::ptr::null_mut();
    fme.dwAspect = DVASPECT_CONTENT;
    fme.lindex = -1i32;
    fme.tymed = TYMED_HGLOBAL;
    hr = ((*(*pDataObj).lpVtbl).QueryGetData)(pDataObj, &mut fme);
    if FAILED(hr) != 0 {
        return hr;
    }

    ZeroMemory(&mut stgm as *mut _ as PVOID, core::mem::size_of::<STGMEDIUM>());
    hr = ((*(*pDataObj).lpVtbl).GetData)(pDataObj, &mut fme, &mut stgm);
    if SUCCEEDED(hr) != 0 {
        hGlobal = stgm.DUMMYUNIONNAME.hGlobal;
        if !hGlobal.is_null() {
            if IsWindowUnicode(hwndAccepter) != 0 {
                PostMessageW(hwndAccepter, WM_DROPFILES, hGlobal as WPARAM, 0);
            } else {
                PostMessageA(hwndAccepter, WM_DROPFILES, hGlobal as WPARAM, 0);
            }
        }
        ReleaseStgMedium(&mut stgm);
    }

    return hr;
}

unsafe fn drag_enter(info: *mut TrackerWindowInfo, mut new_target: HWND) {
    let mut hr: HRESULT;
    let mut dwEffect: DWORD = *(*info).pdwEffect;

    (*info).curTargetHWND = new_target;

    (*info).accepterHWND = core::ptr::null_mut();
    while !new_target.is_null() && is_droptarget(new_target) == 0 {
        if is_acceptfiles(new_target) != 0 {
            dwEffect = (*info).dwOKEffect;
            hr = DefaultDragEnter(
                new_target,
                (*info).dataObject,
                (*info).dwKeyState,
                (*info).curMousePos,
                &mut dwEffect,
            );
            dwEffect &= (*info).dwOKEffect;

            if hr == S_OK {
                (*info).accepterHWND = new_target;
                (*info).curDragTarget = core::ptr::null_mut();
                *(*info).pdwEffect = dwEffect;
                return;
            }
        }
        new_target = GetParent(new_target);
    }

    (*info).curDragTarget = get_droptarget_pointer(new_target);

    if !(*info).curDragTarget.is_null() {
        *(*info).pdwEffect = (*info).dwOKEffect;
        hr = IDropTarget_DragEnter(
            (*info).curDragTarget,
            (*info).dataObject,
            (*info).dwKeyState,
            (*info).curMousePos,
            (*info).pdwEffect,
        );
        *(*info).pdwEffect &= (*info).dwOKEffect;

        if hr != S_OK {
            IDropTarget_Release((*info).curDragTarget);
            (*info).curDragTarget = core::ptr::null_mut();
            (*info).curTargetHWND = core::ptr::null_mut();
            (*info).accepterHWND = core::ptr::null_mut();
        }
    }
}

unsafe fn drag_end(info: *mut TrackerWindowInfo) {

    let mut hr: HRESULT;

    (*info).trackingDone = 1u8;
    ReleaseCapture();

    if !(*info).curDragTarget.is_null() {
        if (*info).returnValue == DRAGDROP_S_DROP && *(*info).pdwEffect != DROPEFFECT_NONE {
            *(*info).pdwEffect = (*info).dwOKEffect;
            hr = IDropTarget_Drop(
                (*info).curDragTarget,
                (*info).dataObject,
                (*info).dwKeyState,
                (*info).curMousePos,
                (*info).pdwEffect,
            );
            *(*info).pdwEffect &= (*info).dwOKEffect;

            if FAILED(hr) != 0 {
                (*info).returnValue = hr;
            }
        } else {
            IDropTarget_DragLeave((*info).curDragTarget);
            *(*info).pdwEffect = DROPEFFECT_NONE;
        }
        IDropTarget_Release((*info).curDragTarget);
        (*info).curDragTarget = core::ptr::null_mut();
    } else if !(*info).accepterHWND.is_null() {
        if (*info).returnValue == DRAGDROP_S_DROP && *(*info).pdwEffect != DROPEFFECT_NONE {
            *(*info).pdwEffect = (*info).dwOKEffect;
            hr = DefaultDrop(
                (*info).accepterHWND,
                (*info).dataObject,
                (*info).dwKeyState,
                (*info).curMousePos,
                (*info).pdwEffect,
            );
            *(*info).pdwEffect &= (*info).dwOKEffect;

            if FAILED(hr) != 0 {
                (*info).returnValue = hr;
            }
        } else {
            *(*info).pdwEffect = DROPEFFECT_NONE;
        }
        (*info).accepterHWND = core::ptr::null_mut();
    } else {
        *(*info).pdwEffect = DROPEFFECT_NONE;
    }
}

unsafe fn give_feedback(info: *mut TrackerWindowInfo) -> HRESULT {
    let hr: HRESULT;
    let res: i32;
    let cur: HCURSOR;

    if (*info).curDragTarget.is_null() && (*info).accepterHWND.is_null() {
        *(*info).pdwEffect = DROPEFFECT_NONE;
    }

    hr = ((*(*(*info).dropSource).lpVtbl).GiveFeedback)((*info).dropSource, *(*info).pdwEffect);

    if hr == DRAGDROP_S_USEDEFAULTCURSORS {
        if *(*info).pdwEffect & DROPEFFECT_MOVE != 0 {
            res = CURSOR_MOVE;
        } else if *(*info).pdwEffect & DROPEFFECT_COPY != 0 {
            res = CURSOR_COPY;
        } else if *(*info).pdwEffect & DROPEFFECT_LINK != 0 {
            res = CURSOR_LINK;
        } else {
            res = CURSOR_NODROP;
        }

        cur = LoadCursorW(hProxyDll, MAKEINTRESOURCEW(res as u16));
        SetCursor(cur);
    }

    hr
}

unsafe fn OLEDD_TrackStateChange(trackerInfo: *mut TrackerWindowInfo) {
    let mut hwndNewTarget: HWND = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();

    pt.x = (*trackerInfo).curMousePos.x;
    pt.y = (*trackerInfo).curMousePos.y;
    hwndNewTarget = WindowFromPoint(pt);

    (*trackerInfo).returnValue = IDropSource_QueryContinueDrag(
        (*trackerInfo).dropSource,
        (*trackerInfo).escPressed,
        (*trackerInfo).dwKeyState,
    );

    if (*trackerInfo).curTargetHWND != hwndNewTarget
        && ((*trackerInfo).returnValue == S_OK || (*trackerInfo).returnValue == DRAGDROP_S_DROP)
    {
        if !(*trackerInfo).curDragTarget.is_null() {
            IDropTarget_DragLeave((*trackerInfo).curDragTarget);
            IDropTarget_Release((*trackerInfo).curDragTarget);
            (*trackerInfo).curDragTarget = core::ptr::null_mut();
            (*trackerInfo).curTargetHWND = core::ptr::null_mut();
        }
        (*trackerInfo).accepterHWND = core::ptr::null_mut();

        if !hwndNewTarget.is_null() {
            drag_enter(trackerInfo, hwndNewTarget);
        }

        give_feedback(trackerInfo);
    }

    if (*trackerInfo).returnValue == S_OK {
        if !(*trackerInfo).curDragTarget.is_null() {
            *(*trackerInfo).pdwEffect = (*trackerInfo).dwOKEffect;
            IDropTarget_DragOver(
                (*trackerInfo).curDragTarget,
                (*trackerInfo).dwKeyState,
                (*trackerInfo).curMousePos,
                (*trackerInfo).pdwEffect,
            );
            *(*trackerInfo).pdwEffect &= (*trackerInfo).dwOKEffect;
        } else if !(*trackerInfo).accepterHWND.is_null() {
            *(*trackerInfo).pdwEffect = (*trackerInfo).dwOKEffect;
        }
        give_feedback(trackerInfo);
    } else {
        drag_end(trackerInfo);
    }
}

unsafe fn OLEDD_GetButtonState() -> DWORD {

    let mut keyboardState: [BYTE; 256] = [0; 256];
    let mut keyMask: DWORD = 0;

    GetKeyboardState(keyboardState.as_mut_ptr());

    if (keyboardState[VK_SHIFT as usize] & 0x80) != 0 {
        keyMask |= MK_SHIFT;
    }

    if (keyboardState[VK_CONTROL as usize] & 0x80) != 0 {
        keyMask |= MK_CONTROL;
    }

    if (keyboardState[VK_MENU as usize] & 0x80) != 0 {
        keyMask |= MK_ALT;
    }

    if (keyboardState[VK_LBUTTON as usize] & 0x80) != 0 {
        keyMask |= MK_LBUTTON;
    }

    if (keyboardState[VK_RBUTTON as usize] & 0x80) != 0 {
        keyMask |= MK_RBUTTON;
    }

    if (keyboardState[VK_MBUTTON as usize] & 0x80) != 0 {
        keyMask |= MK_MBUTTON;
    }

    keyMask
}
unsafe fn OLEUTL_ReadRegistryDWORDValue(
    regKey: HKEY,
    pdwValue: *mut DWORD) {
    let mut buffer: [WCHAR; 20] = [0; 20];
    let mut cbData: DWORD = core::mem::size_of_val(&buffer) as DWORD;
    let mut dwKeyType: DWORD = 0;
    let lres: LONG;

    lres = RegQueryValueExW(
        regKey,
        emptyW.as_ptr(),
        core::ptr::null_mut(),
        &mut dwKeyType,
        buffer.as_mut_ptr() as LPBYTE,
        &mut cbData,
    );

    if lres == ERROR_SUCCESS as LONG {
        match dwKeyType {
            REG_DWORD => {
                *pdwValue = *(buffer.as_ptr() as *const DWORD);
            }
            REG_EXPAND_SZ | REG_MULTI_SZ | REG_SZ => {
                *pdwValue = wcstoul(buffer.as_ptr(), core::ptr::null_mut(), 10);
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn OleDraw(
    pUnk: *mut IUnknown,
    dwAspect: DWORD,
    hdcDraw: HDC,
    rect: LPCRECT,
) -> HRESULT {
    let mut viewobject: *mut IViewObject = core::ptr::null_mut();

    if pUnk.is_null() {
        return E_INVALIDARG;
    }

    let hres = IUnknown_QueryInterface(
        pUnk,
        &IID_IViewObject,
        &mut viewobject as *mut *mut IViewObject as *mut *mut core::ffi::c_void,
    );

    if SUCCEEDED(hres) != 0 {
        let hres = IViewObject_Draw(
            viewobject,
            dwAspect,
            -1i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hdcDraw,
            rect as *mut RECTL,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        );
        IViewObject_Release(viewobject);
        return hres;
    } else {
        return DV_E_NOIVIEWOBJECT;
    }
}

#[no_mangle]
pub unsafe extern "system" fn OleTranslateAccelerator(
    lpFrame: LPOLEINPLACEFRAME,
    lpFrameInfo: LPOLEINPLACEFRAMEINFO,
    lpmsg: LPMSG,
) -> HRESULT {
    let mut wID: u16 = 0;

    TRACE!(
        b"(%p,%p,%p)\n\0".as_ptr() as PCHAR,
        lpFrame,
        lpFrameInfo,
        lpmsg,
    );

    if IsAccelerator(
        (*lpFrameInfo).haccel,
        (*lpFrameInfo).cAccelEntries,
        lpmsg,
        &mut wID,
    ) != 0
    {
        return IOleInPlaceFrame_TranslateAccelerator(lpFrame, lpmsg, wID);
    }

    return S_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn OleCreate(
    rclsid: REFCLSID,
    riid: REFIID,
    renderopt: DWORD,
    pFormatEtc: LPFORMATETC,
    pClientSite: LPOLECLIENTSITE,
    pStg: LPSTORAGE,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut hres: HRESULT;
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let mut pOleObject: *mut IOleObject = core::ptr::null_mut();

    TRACE!(
        b"(%s, %s, %d, %p, %p, %p, %p)\n\0".as_ptr() as PCHAR,
        debugstr_guid(rclsid),
        debugstr_guid(riid),
        renderopt,
        pFormatEtc,
        pClientSite,
        pStg,
        ppvObj,
    );

    hres = CoCreateInstance(
        rclsid,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER,
        riid,
        &mut pUnk as *mut *mut IUnknown as *mut LPVOID,
    );

    if SUCCEEDED(hres) != 0 {
        hres = IStorage_SetClass(pStg, rclsid);
    }

    if !pClientSite.is_null() && SUCCEEDED(hres) != 0 {
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleObject,
            &mut pOleObject as *mut *mut IOleObject as *mut LPVOID,
        );
        if SUCCEEDED(hres) != 0 {
            let mut dwStatus: DWORD = 0;
            IOleObject_GetMiscStatus(pOleObject, DVASPECT_CONTENT, &mut dwStatus);
        }
    }

    if SUCCEEDED(hres) != 0 {
        let mut pPS: *mut IPersistStorage = core::ptr::null_mut();
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IPersistStorage,
            &mut pPS as *mut *mut IPersistStorage as *mut LPVOID,
        );
        if SUCCEEDED(hres) != 0 {
            TRACE!(b"trying to set stg %p\n\0".as_ptr() as PCHAR, pStg);
            hres = IPersistStorage_InitNew(pPS, pStg);
            TRACE!(b"-- result 0x%08x\n\0".as_ptr() as PCHAR, hres);
            IPersistStorage_Release(pPS);
        }
    }

    if !pClientSite.is_null() && SUCCEEDED(hres) != 0 {
        TRACE!(
            b"trying to set clientsite %p\n\0".as_ptr() as PCHAR,
            pClientSite,
        );
        hres = IOleObject_SetClientSite(pOleObject, pClientSite);
        TRACE!(b"-- result 0x%08x\n\0".as_ptr() as PCHAR, hres);
    }

    if !pOleObject.is_null() {
        IOleObject_Release(pOleObject);
    }

    if ((renderopt == OLERENDER_DRAW) || (renderopt == OLERENDER_FORMAT)) && SUCCEEDED(hres) != 0 {
        hres = OleRun(pUnk);
        if SUCCEEDED(hres) != 0 {
            let mut pOleCache: *mut IOleCache = core::ptr::null_mut();

            if SUCCEEDED(IUnknown_QueryInterface(
                pUnk,
                &IID_IOleCache,
                &mut pOleCache as *mut *mut IOleCache as *mut *mut core::ffi::c_void,
            )) != 0
            {
                let mut dwConnection: DWORD = 0;
                if renderopt == OLERENDER_DRAW && pFormatEtc.is_null() {
                    let mut pfe: FORMATETC = core::mem::zeroed();
                    pfe.cfFormat = 0;
                    pfe.ptd = core::ptr::null_mut();
                    pfe.dwAspect = DVASPECT_CONTENT;
                    pfe.lindex = -1;
                    pfe.tymed = TYMED_NULL;
                    hres = IOleCache_Cache(pOleCache, &mut pfe, ADVF_PRIMEFIRST, &mut dwConnection);
                } else {
                    hres = IOleCache_Cache(pOleCache, pFormatEtc, ADVF_PRIMEFIRST, &mut dwConnection);
                }
                IOleCache_Release(pOleCache);
            }
        }
    }

    if FAILED(hres) != 0 && !pUnk.is_null() {
        IUnknown_Release(pUnk);
        pUnk = core::ptr::null_mut();
    }

    *ppvObj = pUnk as LPVOID;

    TRACE!(b"-- %p\n\0".as_ptr() as PCHAR, pUnk);
    hres
}

#[no_mangle]
pub unsafe extern "system" fn OleGetAutoConvert(clsidOld: *const CLSID, pClsidNew: *mut CLSID) -> HRESULT {
    static wszAutoConvertTo: [u16; 14] = [
        'A' as u16, 'u' as u16, 't' as u16, 'o' as u16, 'C' as u16, 'o' as u16, 'n' as u16,
        'v' as u16, 'e' as u16, 'r' as u16, 't' as u16, 'T' as u16, 'o' as u16, 0
    ];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut buf: [u16; 39] = [0; 39];
    let mut len: i32;
    let mut res: HRESULT = 0;

    res = COM_OpenKeyForCLSID(clsidOld, wszAutoConvertTo.as_ptr(), KEY_READ, &mut hkey);
    if FAILED(res) != 0 {
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }

    len = (core::mem::size_of_val(&buf)) as i32;
    if RegQueryValueW(hkey, core::ptr::null_mut(), buf.as_mut_ptr(), &mut len) != 0 {
        res = 0x80040150u32 as HRESULT;
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }
    res = CLSIDFromString(buf.as_ptr(), pClsidNew);
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn OleSetAutoConvert(clsidOld: *const CLSID, clsidNew: *const CLSID) -> HRESULT {

    static wszAutoConvertTo: [u16; 14] = [65, 117, 116, 111, 67, 111, 110, 118, 101, 114, 116, 84, 111, 0];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut szClsidNew: [u16; 39] = [0; 39];
    let mut res: HRESULT = 0;

    TRACE!(b"(%s,%s)\n\0".as_ptr(), debugstr_guid(clsidOld), debugstr_guid(clsidNew));

    res = COM_OpenKeyForCLSID(clsidOld, core::ptr::null(), KEY_READ | KEY_WRITE, &mut hkey);
    if FAILED(res) != 0 {
        if !hkey.is_null() { RegCloseKey(hkey); }
        return res;
    }
    StringFromGUID2(clsidNew, szClsidNew.as_mut_ptr(), 39);
    if RegSetValueW(hkey, wszAutoConvertTo.as_ptr(), REG_SZ, szClsidNew.as_ptr(), ((lstrlenW(szClsidNew.as_ptr()) + 1) * 2) as u32) != 0 {
        res = 0x80040150u32 as i32;
    }

    if !hkey.is_null() { RegCloseKey(hkey); }
    res
}

#[no_mangle]
pub unsafe extern "system" fn OleDoAutoConvert(pStg: LPSTORAGE, pClsidNew: *mut CLSID) -> HRESULT {

    let mut user_type_old: LPOLESTR = core::ptr::null_mut();
    let mut user_type_new: LPOLESTR;
    let mut cf: CLIPFORMAT;
    let mut stat: STATSTG = core::mem::zeroed();
    let mut clsid: CLSID = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!(b"(%p, %p)\n\0".as_ptr(), pStg, pClsidNew);

    *pClsidNew = CLSID_NULL;
    if pStg.is_null() {
        return E_INVALIDARG;
    }
    hr = IStorage_Stat(pStg, &mut stat, STATFLAG_NONAME);
    if FAILED(hr) != 0 {
        return hr;
    }

    *pClsidNew = stat.clsid;
    hr = OleGetAutoConvert(&stat.clsid, &mut clsid);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = IStorage_SetClass(pStg, &clsid);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = ReadFmtUserTypeStg(pStg, &mut cf, &mut user_type_old);
    if FAILED(hr) != 0 {
        cf = 0;
        user_type_new = core::ptr::null_mut();
    } else {
        user_type_new = core::ptr::null_mut();
    }

    hr = OleRegGetUserType(&clsid, USERCLASSTYPE_FULL, &mut user_type_new);
    if FAILED(hr) != 0 {
        user_type_new = core::ptr::null_mut();
    }

    hr = WriteFmtUserTypeStg(pStg, cf, user_type_new);
    CoTaskMemFree(user_type_new as LPVOID);
    if FAILED(hr) != 0 {
        CoTaskMemFree(user_type_old as LPVOID);
        IStorage_SetClass(pStg, &stat.clsid);
        return hr;
    }

    hr = SetConvertStg(pStg, 1);
    if FAILED(hr) != 0 {
        WriteFmtUserTypeStg(pStg, cf, user_type_old);
        IStorage_SetClass(pStg, &stat.clsid);
    } else {
        *pClsidNew = clsid;
    }
    CoTaskMemFree(user_type_old as LPVOID);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleIsRunning(object: LPOLEOBJECT) -> BOOL {
    let mut pRunnable: *mut IRunnableObject = core::ptr::null_mut();
    let hr: HRESULT;
    let running: BOOL;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, object);

    if object.is_null() {
        return 0;
    }

    hr = IOleObject_QueryInterface(
        object,
        &IID_IRunnableObject,
        &mut pRunnable as *mut *mut IRunnableObject as *mut LPVOID,
    );
    if FAILED(hr) {
        return 1;
    }
    running = IRunnableObject_IsRunning(pRunnable);
    IRunnableObject_Release(pRunnable);
    return running;
}

#[no_mangle]
pub unsafe extern "system" fn OleNoteObjectExternal(pUnknown: LPUNKNOWN, bVisible: BOOL) -> HRESULT {

    TRACE!(
        b"(%p, %s)\n\0".as_ptr(),
        pUnknown,
        if bVisible != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() }
    );
    CoLockObjectExternal(pUnknown, bVisible, 1)
}

unsafe fn OLE_FreeClipDataArray(count: ULONG, pClipDataArray: *mut CLIPDATA) {
    let mut i: ULONG = 0;
    while i < count {
        CoTaskMemFree((*pClipDataArray.add(i as usize)).pClipData as LPVOID);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn PropSysAllocString(str: LPCOLESTR) -> BSTR {
    SysAllocString(str)
}

#[no_mangle]
pub unsafe extern "system" fn PropSysFreeString(str: LPOLESTR) {
    SysFreeString(str);
}

unsafe fn PROPVARIANT_ValidateType(vt: VARTYPE) -> HRESULT {
    match vt {
        VT_EMPTY
        | VT_NULL
        | VT_I1
        | VT_I2
        | VT_I4
        | VT_I8
        | VT_R4
        | VT_R8
        | VT_CY
        | VT_DATE
        | VT_BSTR
        | VT_ERROR
        | VT_BOOL
        | VT_DECIMAL
        | VT_UI1
        | VT_UI2
        | VT_UI4
        | VT_UI8
        | VT_INT
        | VT_UINT
        | VT_LPSTR
        | VT_LPWSTR
        | VT_FILETIME
        | VT_BLOB
        | VT_DISPATCH
        | VT_UNKNOWN
        | VT_STREAM
        | VT_STORAGE
        | VT_STREAMED_OBJECT
        | VT_STORED_OBJECT
        | VT_BLOB_OBJECT
        | VT_CF
        | VT_CLSID => S_OK,
        _ if vt == VT_I1 | VT_VECTOR => S_OK,
        _ if vt == VT_I2 | VT_VECTOR => S_OK,
        _ if vt == VT_I4 | VT_VECTOR => S_OK,
        _ if vt == VT_I8 | VT_VECTOR => S_OK,
        _ if vt == VT_R4 | VT_VECTOR => S_OK,
        _ if vt == VT_R8 | VT_VECTOR => S_OK,
        _ if vt == VT_CY | VT_VECTOR => S_OK,
        _ if vt == VT_DATE | VT_VECTOR => S_OK,
        _ if vt == VT_BSTR | VT_VECTOR => S_OK,
        _ if vt == VT_ERROR | VT_VECTOR => S_OK,
        _ if vt == VT_BOOL | VT_VECTOR => S_OK,
        _ if vt == VT_VARIANT | VT_VECTOR => S_OK,
        _ if vt == VT_UI1 | VT_VECTOR => S_OK,
        _ if vt == VT_UI2 | VT_VECTOR => S_OK,
        _ if vt == VT_UI4 | VT_VECTOR => S_OK,
        _ if vt == VT_UI8 | VT_VECTOR => S_OK,
        _ if vt == VT_LPSTR | VT_VECTOR => S_OK,
        _ if vt == VT_LPWSTR | VT_VECTOR => S_OK,
        _ if vt == VT_FILETIME | VT_VECTOR => S_OK,
        _ if vt == VT_CF | VT_VECTOR => S_OK,
        _ if vt == VT_CLSID | VT_VECTOR => S_OK,
        _ if vt == VT_ARRAY | VT_I1 => S_OK,
        _ if vt == VT_ARRAY | VT_UI1 => S_OK,
        _ if vt == VT_ARRAY | VT_I2 => S_OK,
        _ if vt == VT_ARRAY | VT_UI2 => S_OK,
        _ if vt == VT_ARRAY | VT_I4 => S_OK,
        _ if vt == VT_ARRAY | VT_UI4 => S_OK,
        _ if vt == VT_ARRAY | VT_INT => S_OK,
        _ if vt == VT_ARRAY | VT_UINT => S_OK,
        _ if vt == VT_ARRAY | VT_R4 => S_OK,
        _ if vt == VT_ARRAY | VT_R8 => S_OK,
        _ if vt == VT_ARRAY | VT_CY => S_OK,
        _ if vt == VT_ARRAY | VT_DATE => S_OK,
        _ if vt == VT_ARRAY | VT_BSTR => S_OK,
        _ if vt == VT_ARRAY | VT_BOOL => S_OK,
        _ if vt == VT_ARRAY | VT_DECIMAL => S_OK,
        _ if vt == VT_ARRAY | VT_DISPATCH => S_OK,
        _ if vt == VT_ARRAY | VT_UNKNOWN => S_OK,
        _ if vt == VT_ARRAY | VT_ERROR => S_OK,
        _ if vt == VT_ARRAY | VT_VARIANT => S_OK,
        _ => {
            WARN!(b"Bad type %d\n\0".as_ptr() as *const i8, vt);
            STG_E_INVALIDPARAMETER
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), pvar);

    if pvar.is_null() {
        return S_OK;
    }

    let mut hr = PROPVARIANT_ValidateType((*pvar).vt);
    if FAILED(hr) != 0 {
        memset(pvar as *mut core::ffi::c_void, 0, core::mem::size_of::<PROPVARIANT>());
        return hr;
    }

    match (*pvar).vt {
        VT_EMPTY | VT_NULL | VT_I1 | VT_I2 | VT_I4 | VT_I8 | VT_R4 | VT_R8 | VT_CY | VT_DATE |
        VT_ERROR | VT_BOOL | VT_DECIMAL | VT_UI1 | VT_UI2 | VT_UI4 | VT_UI8 | VT_INT | VT_UINT |
        VT_FILETIME => {}
        VT_DISPATCH | VT_UNKNOWN | VT_STREAM | VT_STREAMED_OBJECT | VT_STORAGE | VT_STORED_OBJECT => {
            if !(*pvar).u.pStream.is_null() {
                IStream_Release((*pvar).u.pStream);
            }
        }
        VT_CLSID | VT_LPSTR | VT_LPWSTR => {
            CoTaskMemFree((*pvar).u.puuid as LPVOID);
        }
        VT_BLOB | VT_BLOB_OBJECT => {
            CoTaskMemFree((*pvar).u.blob.pBlobData as LPVOID);
        }
        VT_BSTR => {
            PropSysFreeString((*pvar).u.bstrVal);
        }
        VT_CF => {
            if !(*pvar).u.pclipdata.is_null() {
                OLE_FreeClipDataArray(1, (*pvar).u.pclipdata);
                CoTaskMemFree((*pvar).u.pclipdata as LPVOID);
            }
        }
        _ => {
            if ((*pvar).vt & VT_VECTOR) != 0 {
                match (*pvar).vt & !VT_VECTOR {
                    VT_VARIANT => {
                        FreePropVariantArray((*pvar).u.capropvar.cElems, (*pvar).u.capropvar.pElems);
                    }
                    VT_CF => {
                        OLE_FreeClipDataArray((*pvar).u.caclipdata.cElems, (*pvar).u.caclipdata.pElems);
                    }
                    VT_BSTR => {
                        for i in 0..(*pvar).u.cabstr.cElems {
                            PropSysFreeString(*(*pvar).u.cabstr.pElems.offset(i as isize));
                        }
                    }
                    VT_LPSTR => {
                        for i in 0..(*pvar).u.calpstr.cElems {
                            CoTaskMemFree(*(*pvar).u.calpstr.pElems.offset(i as isize) as LPVOID);
                        }
                    }
                    VT_LPWSTR => {
                        for i in 0..(*pvar).u.calpwstr.cElems {
                            CoTaskMemFree(*(*pvar).u.calpwstr.pElems.offset(i as isize) as LPVOID);
                        }
                    }
                    _ => {}
                }
                if ((*pvar).vt & !VT_VECTOR) != 0 {
                    CoTaskMemFree((*pvar).u.capropvar.pElems as LPVOID);
                }
            } else if ((*pvar).vt & VT_ARRAY) != 0 {
                hr = SafeArrayDestroy((*pvar).u.parray);
            } else {
                WARN!(b"Invalid/unsupported type %d\n\0".as_ptr(), (*pvar).vt);
                hr = STG_E_INVALIDPARAMETER;
            }
        }
    }

    memset(pvar as *mut core::ffi::c_void, 0, core::mem::size_of::<PROPVARIANT>());
    hr
}
#[no_mangle]
pub unsafe extern "C" fn PropVariantCopy(
    pvarDest: *mut PROPVARIANT,
    pvarSrc: *const PROPVARIANT,
) -> HRESULT {

    let len: ULONG;
    let hr: HRESULT;

    TRACE!(b"(%p, %p vt %04x)\n\0".as_ptr() as *const i8, pvarDest as *mut core::ffi::c_void, pvarSrc as *const core::ffi::c_void, (*pvarSrc).vt);

    hr = PROPVARIANT_ValidateType((*pvarSrc).vt);
    if FAILED(hr) != 0 {
        return DISP_E_BADVARTYPE;
    }

    *pvarDest = *pvarSrc;

    match (*pvarSrc).vt as u32 {
        VT_EMPTY | VT_NULL | VT_I1 | VT_UI1 | VT_I2 | VT_UI2 | VT_BOOL | VT_DECIMAL |
        VT_I4 | VT_UI4 | VT_R4 | VT_ERROR | VT_I8 | VT_UI8 | VT_INT | VT_UINT |
        VT_R8 | VT_CY | VT_DATE | VT_FILETIME => {},
        VT_DISPATCH | VT_UNKNOWN | VT_STREAM | VT_STREAMED_OBJECT | VT_STORAGE | VT_STORED_OBJECT => {
            if !(*pvarDest).u.pStream.is_null() {
                IStream_AddRef((*pvarDest).u.pStream);
            }
        },
        VT_CLSID => {
            (*pvarDest).u.puuid = CoTaskMemAlloc(core::mem::size_of::<CLSID>()) as *mut CLSID;
            *(*pvarDest).u.puuid = *(*pvarSrc).u.puuid;
        },
        VT_LPSTR => {
            if !(*pvarSrc).u.pszVal.is_null() {
                len = strlen((*pvarSrc).u.pszVal as *const i8) as ULONG;
                (*pvarDest).u.pszVal = CoTaskMemAlloc(((len + 1) * core::mem::size_of::<i8>() as u32) as usize) as *mut i8;
                core::ptr::copy_nonoverlapping((*pvarSrc).u.pszVal, (*pvarDest).u.pszVal, ((len + 1) * core::mem::size_of::<i8>() as u32) as usize);
            }
        },
        VT_LPWSTR => {
            if !(*pvarSrc).u.pwszVal.is_null() {
                len = lstrlenW((*pvarSrc).u.pwszVal) as ULONG;
                (*pvarDest).u.pwszVal = CoTaskMemAlloc(((len + 1) * core::mem::size_of::<u16>() as u32) as usize) as *mut u16;
                core::ptr::copy_nonoverlapping((*pvarSrc).u.pwszVal, (*pvarDest).u.pwszVal, ((len + 1) * core::mem::size_of::<u16>() as u32) as usize);
            }
        },
        VT_BLOB | VT_BLOB_OBJECT => {
            if !(*pvarSrc).u.blob.pBlobData.is_null() {
                len = (*pvarSrc).u.blob.cbSize;
                (*pvarDest).u.blob.pBlobData = CoTaskMemAlloc(len as usize) as *mut u8;
                core::ptr::copy_nonoverlapping((*pvarSrc).u.blob.pBlobData, (*pvarDest).u.blob.pBlobData, len as usize);
            }
        },
        VT_BSTR => {
            (*pvarDest).u.bstrVal = PropSysAllocString((*pvarSrc).u.bstrVal);
        },
        VT_CF => {
            if !(*pvarSrc).u.pclipdata.is_null() {
                len = (*(*pvarSrc).u.pclipdata).cbSize - core::mem::size_of::<ULONG>() as u32;
                (*pvarDest).u.pclipdata = CoTaskMemAlloc(core::mem::size_of::<CLIPDATA>()) as *mut CLIPDATA;
                (*(*pvarDest).u.pclipdata).cbSize = (*(*pvarSrc).u.pclipdata).cbSize;
                (*(*pvarDest).u.pclipdata).ulClipFmt = (*(*pvarSrc).u.pclipdata).ulClipFmt;
                (*(*pvarDest).u.pclipdata).pClipData = CoTaskMemAlloc(len as usize) as *mut u8;
                core::ptr::copy_nonoverlapping((*(*pvarSrc).u.pclipdata).pClipData, (*(*pvarDest).u.pclipdata).pClipData, len as usize);
            }
        },
        _ => {
            if ((*pvarSrc).vt & VT_VECTOR as u16) != 0 {
                let elemSize: i32;

                match ((*pvarSrc).vt & !(VT_VECTOR as u16)) as u32 {
                    VT_I1 => elemSize = core::mem::size_of::<i8>() as i32,
                    VT_UI1 => elemSize = core::mem::size_of::<u8>() as i32,
                    VT_I2 => elemSize = core::mem::size_of::<i16>() as i32,
                    VT_UI2 => elemSize = core::mem::size_of::<u16>() as i32,
                    VT_BOOL => elemSize = core::mem::size_of::<VARIANT_BOOL>() as i32,
                    VT_I4 => elemSize = core::mem::size_of::<i32>() as i32,
                    VT_UI4 => elemSize = core::mem::size_of::<u32>() as i32,
                    VT_R4 => elemSize = core::mem::size_of::<f32>() as i32,
                    VT_R8 => elemSize = core::mem::size_of::<f64>() as i32,
                    VT_ERROR => elemSize = core::mem::size_of::<SCODE>() as i32,
                    VT_I8 => elemSize = core::mem::size_of::<i64>() as i32,
                    VT_UI8 => elemSize = core::mem::size_of::<u64>() as i32,
                    VT_CY => elemSize = core::mem::size_of::<CY>() as i32,
                    VT_DATE => elemSize = core::mem::size_of::<DATE>() as i32,
                    VT_FILETIME => elemSize = core::mem::size_of::<FILETIME>() as i32,
                    VT_CLSID => elemSize = core::mem::size_of::<CLSID>() as i32,
                    VT_CF => elemSize = core::mem::size_of::<CLIPDATA>() as i32,
                    VT_BSTR => elemSize = core::mem::size_of::<BSTR>() as i32,
                    VT_LPSTR => elemSize = core::mem::size_of::<LPSTR>() as i32,
                    VT_LPWSTR => elemSize = core::mem::size_of::<LPWSTR>() as i32,
                    VT_VARIANT => elemSize = core::mem::size_of::<PROPVARIANT>() as i32,
                    _ => {
                        FIXME!(b"Invalid element type: %ul\n\0".as_ptr() as *const i8, (*pvarSrc).vt & !(VT_VECTOR as u16));
                        return E_INVALIDARG;
                    }
                }
                len = (*pvarSrc).u.capropvar.cElems;
                (*pvarDest).u.capropvar.pElems = if len != 0 { CoTaskMemAlloc((len * elemSize as u32) as usize) as *mut PROPVARIANT } else { core::ptr::null_mut() };
                if (*pvarSrc).vt == (VT_VECTOR | VT_VARIANT) as u16 {
                    for i in 0..len {
                        PropVariantCopy(&mut (*(*pvarDest).u.capropvar.pElems.add(i as usize)), &(*(*pvarSrc).u.capropvar.pElems.add(i as usize)));
                    }
                } else if (*pvarSrc).vt == (VT_VECTOR | VT_CF) as u16 {
                    FIXME!(b"Copy clipformats\n\0".as_ptr() as *const i8);
                } else if (*pvarSrc).vt == (VT_VECTOR | VT_BSTR) as u16 {
                    for i in 0..len {
                        *(*pvarDest).u.cabstr.pElems.add(i as usize) = PropSysAllocString(*(*pvarSrc).u.cabstr.pElems.add(i as usize));
                    }
                } else if (*pvarSrc).vt == (VT_VECTOR | VT_LPSTR) as u16 {
                    let mut strLen: usize;
                    for i in 0..len {
                        strLen = (lstrlenA(*(*pvarSrc).u.calpstr.pElems.add(i as usize)) + 1) as usize;
                        *(*pvarDest).u.calpstr.pElems.add(i as usize) = CoTaskMemAlloc(strLen) as LPSTR;
                        memcpy(*(*pvarDest).u.calpstr.pElems.add(i as usize) as *mut core::ffi::c_void, *(*pvarSrc).u.calpstr.pElems.add(i as usize) as *const core::ffi::c_void, strLen);
                    }
                } else if (*pvarSrc).vt == (VT_VECTOR | VT_LPWSTR) as u16 {
                    let mut strLen: usize;
                    for i in 0..len {
                        strLen = ((lstrlenW(*(*pvarSrc).u.calpwstr.pElems.add(i as usize)) + 1) as usize) * core::mem::size_of::<u16>();
                        *(*pvarDest).u.calpstr.pElems.add(i as usize) = CoTaskMemAlloc(strLen) as LPSTR;
                        memcpy(*(*pvarDest).u.calpstr.pElems.add(i as usize) as *mut core::ffi::c_void, *(*pvarSrc).u.calpstr.pElems.add(i as usize) as *const core::ffi::c_void, strLen);
                    }
                } else {
                    core::ptr::copy_nonoverlapping((*pvarSrc).u.capropvar.pElems, (*pvarDest).u.capropvar.pElems, (len * elemSize as u32) as usize);
                }
            } else if ((*pvarSrc).vt & VT_ARRAY as u16) != 0 {
                (*pvarDest).u.uhVal.QuadPart = 0;
                return SafeArrayCopy((*pvarSrc).u.parray, core::ptr::addr_of_mut!((*pvarDest).u.parray));
            } else {
                WARN!(b"Invalid/unsupported type %d\n\0".as_ptr() as *const i8, (*pvarSrc).vt);
            }
        }
    }

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn FreePropVariantArray(
    cVariants: ULONG,
    rgvars: *mut PROPVARIANT,
) -> HRESULT {
    if rgvars.is_null() {
        return E_INVALIDARG;
    }

    for i in 0..cVariants {
        PropVariantClear(rgvars.add(i as usize));
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllDebugObjectRPCHook(b: BOOL, dummy: *mut core::ffi::c_void) -> BOOL {
    FIXME!(b"stub\n\0".as_ptr() as *const i8);
    1
}
