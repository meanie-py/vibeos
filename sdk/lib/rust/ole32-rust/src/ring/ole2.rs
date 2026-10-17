use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::compobj::ffi::DefWindowProcW;
use crate::compobj::ffi::RegisterClassW;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local import surface.
//
// Everything the shared prelude already carries (win32/user32/gdi32/crt/
// ole32_exports/commacros/iids/win_consts) is used from there. What remains
// below is the tail the shared surface does not (yet) provide. It is all
// PRIVATE (no `pub`) so it does not leak back out through the prelude's
// `pub use crate::ring::ole2::*` glob and collide with a future hoist.
//
// ABI rule (win32.rs): every WINAPI import is `extern "system"` — an
// `extern "C"` decl links on amd64/arm64 and silently breaks i386 (stdcall
// decoration `_Foo@N`). The only `extern "C"` blocks here are (a) data
// statics (IIDs — no calling convention) and (b) ole32's own `#[no_mangle]
// extern "C"` internals in compobj (COM_OpenKeyForCLSID & friends).
// ---------------------------------------------------------------------------

// --- handle/base types the SDK headers define but com-sys does not export ---
type HCURSOR = *mut c_void; // winuser.h DECLARE_HANDLE(HCURSOR)
type HBRUSH = *mut c_void; // wingdi.h DECLARE_HANDLE(HBRUSH)
type ATOM = u16; // minwindef.h
type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;
type TIMERPROC = Option<unsafe extern "system" fn(HWND, UINT, UINT_PTR, DWORD)>;

/// winuser.h:3024 `tagACCEL` — { BYTE fVirt; WORD key; WORD cmd; }. NOT inside the
/// pshpack2 region (that starts further down the header), so natural alignment:
/// fVirt@0, key@2, cmd@4, size 6.
#[repr(C)]
#[derive(Copy, Clone)]
struct ACCEL {
    fVirt: BYTE,
    key: WORD,
    cmd: WORD,
}
type LPACCEL = *mut ACCEL;

/// winuser.h:3126 `tagCWPSTRUCT`
#[repr(C)]
struct CWPSTRUCT {
    lParam: LPARAM,
    wParam: WPARAM,
    message: UINT,
    hwnd: HWND,
}
type LPCWPSTRUCT = *mut CWPSTRUCT;

/// winuser.h:3186 `tagDRAWITEMSTRUCT` (only CtlType is read here)
#[repr(C)]
struct DRAWITEMSTRUCT {
    CtlType: UINT,
    CtlID: UINT,
    itemID: UINT,
    itemAction: UINT,
    itemState: UINT,
    hwndItem: HWND,
    hDC: HDC,
    rcItem: RECT,
    itemData: ULONG_PTR,
}
type LPDRAWITEMSTRUCT = *mut DRAWITEMSTRUCT;

/// winuser.h:3041 `tagCREATESTRUCTA` (only lpCreateParams is read here)
#[repr(C)]
struct CREATESTRUCTA {
    lpCreateParams: LPVOID,
    hInstance: HINSTANCE,
    hMenu: HMENU,
    hwndParent: HWND,
    cy: c_int,
    cx: c_int,
    y: c_int,
    x: c_int,
    style: LONG,
    lpszName: *const c_char,
    lpszClass: *const c_char,
    dwExStyle: DWORD,
}
type LPCREATESTRUCTA = *mut CREATESTRUCTA;

/// winuser.h:3277 `_WNDCLASSW`
#[repr(C)]
struct WNDCLASSW {
    style: UINT,
    lpfnWndProc: WNDPROC,
    cbClsExtra: c_int,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: *mut c_void,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: LPCWSTR,
    lpszClassName: LPCWSTR,
}

/// winnt.h `_MEMORY_BASIC_INFORMATION`
#[repr(C)]
struct MEMORY_BASIC_INFORMATION {
    BaseAddress: PVOID,
    AllocationBase: PVOID,
    AllocationProtect: DWORD,
    RegionSize: SIZE_T,
    State: DWORD,
    Protect: DWORD,
    Type: DWORD,
}

// --- winuser.h window messages (win_consts.rs carries only a subset; these are
//     the menu/window ones ole2.c needs). Cited to sdk/include/psdk/winuser.h. ---
const WM_CREATE: UINT = 0x0001; // winuser.h:1636
const WM_DESTROY: UINT = 0x0002; // winuser.h:1637
const WM_DRAWITEM: UINT = 0x002B; // winuser.h:1673 (43)
const WM_CHAR: UINT = 0x0102; // winuser.h:1745 (258)
const WM_SYSKEYDOWN: UINT = 0x0104; // winuser.h:1747 (260)
const WM_SYSCHAR: UINT = 0x0106; // winuser.h:1749 (262)
const WM_COMMAND: UINT = 0x0111; // winuser.h:1768 (273)
const WM_INITMENU: UINT = 0x0116; // winuser.h:1773 (278)
const WM_INITMENUPOPUP: UINT = 0x0117; // winuser.h:1774 (279)
const WM_MENUSELECT: UINT = 0x011F; // winuser.h:1775 (287)

/// winreg.h:29
const REG_DWORD: DWORD = 4;

/// ole2ver.h — `#define rmm 23` / `#define rup 824`
const rmm: DWORD = 23;
const rup: DWORD = 824;

/// objbase.h — CoInitializeEx apartment model (compobj/com.rs:153)
const COINIT_APARTMENTTHREADED: DWORD = 0x2;

// --- OLE interface pointer spellings consts.rs does not carry ---
type LPDROPTARGET = *mut IDropTarget;
type LPPERSISTSTORAGE = *mut IPersistStorage;
type LPOLEINPLACEFRAME = *mut IOleInPlaceFrame;
type LPOLEINPLACEACTIVEOBJECT = *mut IOleInPlaceActiveObject;

// --- IIDs iids.rs does not yet declare. Real link-time data symbols in uuid.lib;
//     NEVER hand-transcribe a GUID (a wrong byte is a silent QueryInterface miss). ---
extern "C" {
    static IID_IRunnableObject: GUID;
    static IID_IOleLink: GUID;
    static IID_IViewObject: GUID;
    static IID_IOleCache: GUID;
    static IID_IEnumOLEVERB: GUID;
}

// --- compobj.c internals (they ARE #[no_mangle] extern "C" Rust fns in
//     crate::compobj::compobj, so an extern "C" decl resolves at link) ---
extern "C" {
    fn COM_OpenKeyForCLSID(
        clsid: REFCLSID,
        keyname: LPCWSTR,
        access: REGSAM,
        subkey: *mut HKEY,
    ) -> HRESULT;
    fn open_classes_key(hkey: HKEY, name: LPCWSTR, access: REGSAM, retkey: *mut HKEY) -> LONG;
    fn actctx_get_miscstatus(clsid: *const CLSID, aspect: DWORD, status: *mut DWORD) -> BOOL;
}

/// compobj's `COM_CurrentInfo` / `COM_CurrentApt` are plain `pub unsafe fn` Rust
/// items, NOT `#[no_mangle]` link symbols — the extern decl rustconv emitted would
/// be undefined at link. Reach them by crate path instead (same bridge errorinfo.rs
/// uses). This ALSO fixes a live bug: the raw translation invented a local
/// `struct { ole_inits }` and cast COM_CurrentInfo()'s return to it, which reads
/// `ole_inits` at offset 0 — but in the real `oletls` that slot is `apt`.
#[inline]
unsafe fn COM_CurrentInfo() -> *mut crate::compobj::types::oletls {
    crate::compobj::compobj::COM_CurrentInfo()
}
#[inline]
unsafe fn COM_CurrentApt() -> *mut crate::compobj::types::apartment {
    crate::compobj::compobj::COM_CurrentApt()
}

// --- kernel32 / user32 / advapi32 the shared surface does not carry yet ---
extern "system" {
    // kernel32
    fn GetCurrentProcess() -> HANDLE;
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    fn DuplicateHandle(
        hSourceProcessHandle: HANDLE,
        hSourceHandle: HANDLE,
        hTargetProcessHandle: HANDLE,
        lpTargetHandle: *mut HANDLE,
        dwDesiredAccess: DWORD,
        bInheritHandle: BOOL,
        dwOptions: DWORD,
    ) -> BOOL;
    fn CreateFileMappingW(
        hFile: HANDLE,
        lpFileMappingAttributes: *mut c_void,
        flProtect: DWORD,
        dwMaximumSizeHigh: DWORD,
        dwMaximumSizeLow: DWORD,
        lpName: LPCWSTR,
    ) -> HANDLE;
    fn MapViewOfFile(
        hFileMappingObject: HANDLE,
        dwDesiredAccess: DWORD,
        dwFileOffsetHigh: DWORD,
        dwFileOffsetLow: DWORD,
        dwNumberOfBytesToMap: SIZE_T,
    ) -> LPVOID;
    fn UnmapViewOfFile(lpBaseAddress: LPVOID) -> BOOL;
    fn VirtualQuery(
        lpAddress: *const c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: SIZE_T,
    ) -> SIZE_T;

    // advapi32
    fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: *mut DWORD,
        lpReserved: *mut DWORD,
        lpcSubKeys: *mut DWORD,
        lpcbMaxSubKeyLen: *mut DWORD,
        lpcbMaxClassLen: *mut DWORD,
        lpcValues: *mut DWORD,
        lpcbMaxValueNameLen: *mut DWORD,
        lpcbMaxValueLen: *mut DWORD,
        lpcbSecurityDescriptor: *mut DWORD,
        lpftLastWriteTime: *mut FILETIME,
    ) -> LONG;

    // user32 — windows / timers / cursors
    fn GetParent(hWnd: HWND) -> HWND;
    fn WindowFromPoint(Point: POINT) -> HWND;
    fn SetTimer(hWnd: HWND, nIDEvent: UINT_PTR, uElapse: UINT, lpTimerFunc: TIMERPROC) -> UINT_PTR;
    fn KillTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
    fn GetWindowLongW(hWnd: HWND, nIndex: c_int) -> LONG;

    // ole32's own still-C exports (defaulthandler.c / storage32.c)
    fn OleCreateDefaultHandler(
        clsid: REFCLSID,
        pUnkOuter: *mut IUnknown,
        riid: REFIID,
        lplpObj: *mut LPVOID,
    ) -> HRESULT;
    fn WriteClassStg(pStg: *mut IStorage, rclsid: REFCLSID) -> HRESULT;
}

/// `SetWindowLongPtrW` / `GetWindowLongPtrA` are 64-bit-only exports: on 32-bit
/// winuser.h `#define`s them onto SetWindowLongW / GetWindowLongA, so asking the
/// linker for the *Ptr* symbol on i386 fails. Resolve per pointer width.
#[cfg(target_pointer_width = "64")]
extern "system" {
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetWindowLongPtrA(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
}
#[cfg(target_pointer_width = "32")]
extern "system" {
    fn SetWindowLongW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG) -> LONG;
    fn GetWindowLongA(hWnd: HWND, nIndex: c_int) -> LONG;
}

#[inline]
unsafe fn set_window_longptr_w(hwnd: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    #[cfg(target_pointer_width = "64")]
    {
        SetWindowLongPtrW(hwnd, index, value)
    }
    #[cfg(target_pointer_width = "32")]
    {
        SetWindowLongW(hwnd, index, value as LONG) as LONG_PTR
    }
}

#[inline]
unsafe fn get_window_longptr_a(hwnd: HWND, index: c_int) -> LONG_PTR {
    #[cfg(target_pointer_width = "64")]
    {
        GetWindowLongPtrA(hwnd, index)
    }
    #[cfg(target_pointer_width = "32")]
    {
        GetWindowLongA(hwnd, index) as LONG_PTR
    }
}

/// winuser.h: `MAKEINTRESOURCEW(i)` — the integer id packed into an LPWSTR.
#[inline]
fn MAKEINTRESOURCEW(i: WORD) -> LPCWSTR {
    i as usize as LPCWSTR
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct tagTrackerWindowInfo {
    pub dataObject: *mut IDataObject,
    pub dropSource: *mut IDropSource,
    pub dwOKEffect: DWORD,
    pub pdwEffect: *mut DWORD,
    pub trackingDone: BOOL,
    pub inTrackCall: BOOL,
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
    pub mgw: tagOleMenuGroupWidths,
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

pub static prop_oledroptarget: [WCHAR; 23] = [
    'O' as u16, 'l' as u16, 'e' as u16, 'D' as u16, 'r' as u16, 'o' as u16, 'p' as u16, 'T' as u16,
    'a' as u16, 'r' as u16, 'g' as u16, 'e' as u16, 't' as u16, 'I' as u16, 'n' as u16, 't' as u16,
    'e' as u16, 'r' as u16, 'f' as u16, 'a' as u16, 'c' as u16, 'e' as u16, 0,
];

pub static prop_marshalleddroptarget: [WCHAR; 25] = [
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
    QueryInterface: Some(DropTargetWrapper_QueryInterface),
    AddRef: Some(DropTargetWrapper_AddRef),
    Release: Some(DropTargetWrapper_Release),
    DragEnter: Some(DropTargetWrapper_DragEnter),
    DragOver: Some(DropTargetWrapper_DragOver),
    DragLeave: Some(DropTargetWrapper_DragLeave),
    Drop: Some(DropTargetWrapper_Drop),
};

#[repr(C)]
pub struct EnumOLEVERB {
    pub IEnumOLEVERB_iface: IEnumOLEVERB,
    pub ref_: LONG,
    pub hkeyVerb: HKEY,
    pub index: ULONG,
}

pub static EnumOLEVERB_VTable: IEnumOLEVERBVtbl = IEnumOLEVERBVtbl {
    QueryInterface: Some(EnumOLEVERB_QueryInterface),
    AddRef: Some(EnumOLEVERB_AddRef),
    Release: Some(EnumOLEVERB_Release),
    Next: Some(EnumOLEVERB_Next),
    Skip: Some(EnumOLEVERB_Skip),
    Reset: Some(EnumOLEVERB_Reset),
    Clone: Some(EnumOLEVERB_Clone),
};

const DRAG_TIMER_ID: UINT_PTR = 1;

/// ole2.c:160 OleBuildVersion [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleBuildVersion() -> DWORD {
    TRACE!(b"Returning version %d, build %d.\n\0".as_ptr(), rmm, rup);
    (rmm << 16) + rup
}

/// ole2.c:169 OleInitialize [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleInitialize(_reserved: LPVOID) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), reserved);

    // The first duty of OleInitialize is to initialize the COM libraries.
    let hr0 = CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);

    // If the CoInitializeEx call failed, the OLE libraries can't be initialized.
    if FAILED(hr0) {
        return hr0;
    }

    let info = COM_CurrentInfo();
    let hr: HRESULT = if (*info).ole_inits == 0 { S_OK } else { S_FALSE };

    // C: `if (!COM_CurrentInfo()->ole_inits++ && InterlockedIncrement(...) == 1)`
    // — the post-increment always happens; the interlocked bump only on the first.
    let was_zero = (*info).ole_inits == 0;
    (*info).ole_inits += 1;
    if was_zero
        && InterlockedIncrement(core::ptr::addr_of_mut!(OLE_moduleLockCount)) == 1
    {
        TRACE!(b"() - Initializing the OLE libraries\n\0".as_ptr());

        // wine-10: no OLEClipbrd_Initialize -- the clipboard is created on first use.
        OLEDD_Initialize();
        OLEMenu_Initialize();
    }

    hr
}

/// ole2.c:230 OleUninitialize [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleUninitialize() {
    TRACE!(b"()\n\0".as_ptr());

    let info = COM_CurrentInfo();

    if (*info).ole_inits == 0 {
        WARN!(b"ole_inits is already 0\n\0".as_ptr());
        return;
    }

    // C: `if (!--COM_CurrentInfo()->ole_inits && !InterlockedDecrement(...))`
    (*info).ole_inits -= 1;
    if (*info).ole_inits == 0
        && InterlockedDecrement(core::ptr::addr_of_mut!(OLE_moduleLockCount)) == 0
    {
        TRACE!(b"() - Freeing the last reference count\n\0".as_ptr());

        crate::ring::clipboard::clipbrd_uninitialize();
        OLEMenu_UnInitialize();
    }

    CoUninitialize();
}

/// ole2.c:269 OleInitializeWOW [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleInitializeWOW(x: DWORD, y: DWORD) -> HRESULT {
    FIXME!(b"(0x%08x, 0x%08x),stub!\n\0".as_ptr(), x, y);
    0
}

/// ole2.c:281 get_droptarget_handle
#[inline]
unsafe fn get_droptarget_handle(hwnd: HWND) -> HANDLE {
    GetPropW(hwnd, prop_marshalleddroptarget.as_ptr())
}

/// ole2.c:291 is_droptarget
#[inline]
unsafe fn is_droptarget(hwnd: HWND) -> BOOL {
    (!get_droptarget_handle(hwnd).is_null()) as BOOL
}

/// ole2.c:297 is_acceptfiles (ReactOS)
#[inline]
unsafe fn is_acceptfiles(hwnd: HWND) -> BOOL {
    ((GetWindowLongW(hwnd, GWL_EXSTYLE) as DWORD & WS_EX_ACCEPTFILES) != 0) as BOOL
}

/// ole2.c:308 get_droptarget_local_handle
unsafe fn get_droptarget_local_handle(hwnd: HWND) -> HANDLE {
    let mut local_handle: HANDLE = core::ptr::null_mut();

    let handle = get_droptarget_handle(hwnd);

    if !handle.is_null() {
        let mut pid: DWORD = 0;

        GetWindowThreadProcessId(hwnd, &mut pid);
        let process = OpenProcess(PROCESS_DUP_HANDLE, 0, pid);
        if !process.is_null() {
            DuplicateHandle(
                process,
                handle,
                GetCurrentProcess(),
                &mut local_handle,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            );
            CloseHandle(process);
        }
    }
    local_handle
}

/// ole2.c:337 create_map_from_stream
unsafe fn create_map_from_stream(stream: *mut IStream, map: *mut HANDLE) -> HRESULT {
    let mut hmem: HGLOBAL = core::ptr::null_mut();

    let hr = GetHGlobalFromStream(stream, &mut hmem);
    if FAILED(hr) {
        return hr;
    }

    let size: DWORD = GlobalSize(hmem) as DWORD;
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

    let data = MapViewOfFile(*map, FILE_MAP_WRITE, 0, 0, size as SIZE_T);
    memcpy(data, GlobalLock(hmem), size as usize);
    GlobalUnlock(hmem);
    UnmapViewOfFile(data);
    S_OK
}

/// ole2.c:363 create_stream_from_map
unsafe fn create_stream_from_map(map: HANDLE, stream: *mut *mut IStream) -> HRESULT {
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut info: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

    let data = MapViewOfFile(map, FILE_MAP_READ, 0, 0, 0);
    if data.is_null() {
        return hr;
    }

    VirtualQuery(
        data,
        &mut info,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
    );
    TRACE!(b"size %d\n\0".as_ptr(), info.RegionSize as c_int);

    let hmem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, info.RegionSize);
    if !hmem.is_null() {
        memcpy(GlobalLock(hmem), data, info.RegionSize as usize);
        GlobalUnlock(hmem);
        hr = CreateStreamOnHGlobal(hmem, 1, stream);
    }
    UnmapViewOfFile(data);
    hr
}

/// ole2.c:400 impl_from_IDropTarget
#[inline]
unsafe fn impl_from_IDropTarget(iface: *mut IDropTarget) -> *mut DropTargetWrapper {
    CONTAINING_RECORD!(iface, DropTargetWrapper, IDropTarget_iface)
}

stdmethod! {
    /// ole2.c:405
    fn DropTargetWrapper_QueryInterface(
        iface: *mut IDropTarget,
        riid: *const IID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        let This = impl_from_IDropTarget(iface);
        if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IDropTarget) {
            IDropTarget_AddRef(core::ptr::addr_of_mut!((*This).IDropTarget_iface));
            *ppvObject = core::ptr::addr_of_mut!((*This).IDropTarget_iface) as *mut c_void;
            return S_OK;
        }
        *ppvObject = core::ptr::null_mut();
        E_NOINTERFACE
    }
}

stdmethod! {
    /// ole2.c:421
    fn DropTargetWrapper_AddRef(iface: *mut IDropTarget) -> ULONG {
        let This = impl_from_IDropTarget(iface);
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG
    }
}

stdmethod! {
    /// ole2.c:427
    fn DropTargetWrapper_Release(iface: *mut IDropTarget) -> ULONG {
        let This = impl_from_IDropTarget(iface);
        let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
        if refs == 0 {
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
        }
        refs
    }
}

/// ole2.c:435 get_target_from_wrapper
#[inline]
unsafe fn get_target_from_wrapper(
    wrapper: *mut IDropTarget,
    target: *mut *mut IDropTarget,
) -> HRESULT {
    let This = impl_from_IDropTarget(wrapper);
    *target = GetPropW((*This).hwnd, prop_oledroptarget.as_ptr()) as *mut IDropTarget;
    if (*target).is_null() {
        return DRAGDROP_E_NOTREGISTERED;
    }
    IDropTarget_AddRef(*target);
    S_OK
}

stdmethod! {
    /// ole2.c:444
    fn DropTargetWrapper_DragEnter(
        iface: *mut IDropTarget,
        pDataObj: *mut IDataObject,
        grfKeyState: DWORD,
        pt: POINTL,
        pdwEffect: *mut DWORD,
    ) -> HRESULT {
        let mut target: *mut IDropTarget = core::ptr::null_mut();
        let mut r = get_target_from_wrapper(iface, &mut target);

        if SUCCEEDED(r) {
            r = IDropTarget_DragEnter(target, pDataObj, grfKeyState, pt, pdwEffect);
            IDropTarget_Release(target);
        }
        r
    }
}

stdmethod! {
    /// ole2.c:461
    fn DropTargetWrapper_DragOver(
        iface: *mut IDropTarget,
        grfKeyState: DWORD,
        pt: POINTL,
        pdwEffect: *mut DWORD,
    ) -> HRESULT {
        let mut target: *mut IDropTarget = core::ptr::null_mut();
        let mut r = get_target_from_wrapper(iface, &mut target);

        if SUCCEEDED(r) {
            r = IDropTarget_DragOver(target, grfKeyState, pt, pdwEffect);
            IDropTarget_Release(target);
        }
        r
    }
}

stdmethod! {
    /// ole2.c:477
    fn DropTargetWrapper_DragLeave(iface: *mut IDropTarget) -> HRESULT {
        let mut target: *mut IDropTarget = core::ptr::null_mut();
        let mut r = get_target_from_wrapper(iface, &mut target);

        if SUCCEEDED(r) {
            r = IDropTarget_DragLeave(target);
            IDropTarget_Release(target);
        }
        r
    }
}

stdmethod! {
    /// ole2.c:490
    fn DropTargetWrapper_Drop(
        iface: *mut IDropTarget,
        pDataObj: *mut IDataObject,
        grfKeyState: DWORD,
        pt: POINTL,
        pdwEffect: *mut DWORD,
    ) -> HRESULT {
        let mut target: *mut IDropTarget = core::ptr::null_mut();
        let mut r = get_target_from_wrapper(iface, &mut target);

        if SUCCEEDED(r) {
            r = IDropTarget_Drop(target, pDataObj, grfKeyState, pt, pdwEffect);
            IDropTarget_Release(target);
        }
        r
    }
}

/// ole2.c:518 WrapDropTarget
unsafe fn WrapDropTarget(hwnd: HWND) -> *mut IDropTarget {
    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DropTargetWrapper>() as usize,
    ) as *mut DropTargetWrapper;

    if !This.is_null() {
        (*This).IDropTarget_iface.lpVtbl =
            core::ptr::addr_of!(DropTargetWrapperVTbl) as *mut IDropTargetVtbl;
        (*This).hwnd = hwnd;
        (*This).refs = 1;
    }
    // As in the C: IDropTarget_iface is at offset 0, so this is NULL when the
    // allocation failed — which is exactly what RegisterDragDrop tests for.
    core::ptr::addr_of_mut!((*This).IDropTarget_iface)
}

/// ole2.c:536 get_droptarget_pointer
unsafe fn get_droptarget_pointer(hwnd: HWND) -> *mut IDropTarget {
    let mut droptarget: *mut IDropTarget = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();

    let map = get_droptarget_local_handle(hwnd);
    if map.is_null() {
        return core::ptr::null_mut();
    }

    if SUCCEEDED(create_stream_from_map(map, &mut stream)) {
        CoUnmarshalInterface(
            (stream).cast(),
            (&raw const IID_IDropTarget).cast(),
            &mut droptarget as *mut *mut IDropTarget as *mut LPVOID,
        );
        IStream_Release(stream);
    }
    CloseHandle(map);
    droptarget
}

/// ole2.c:557 RegisterDragDrop [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn RegisterDragDrop(hwnd: HWND, pDropTarget: LPDROPTARGET) -> HRESULT {
    let mut pid: DWORD = 0;
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut map: HANDLE = core::ptr::null_mut();

    TRACE!(b"(%p,%p)\n\0".as_ptr(), hwnd, pDropTarget);

    if COM_CurrentApt().is_null() {
        ERR!(b"COM not initialized\n\0".as_ptr());
        return E_OUTOFMEMORY;
    }

    if pDropTarget.is_null() {
        return E_INVALIDARG;
    }

    if IsWindow(hwnd) == 0 {
        ERR!(b"invalid hwnd %p\n\0".as_ptr(), hwnd);
        return DRAGDROP_E_INVALIDHWND;
    }

    // block register for other processes windows
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != GetCurrentProcessId() {
        FIXME!(b"register for another process windows is disabled\n\0".as_ptr());
        return DRAGDROP_E_INVALIDHWND;
    }

    // check if the window is already registered
    if is_droptarget(hwnd) != 0 {
        return DRAGDROP_E_ALREADYREGISTERED;
    }

    let mut hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut stream);
    if FAILED(hr) {
        return hr;
    }

    // IDropTarget::QueryInterface() shouldn't be called, some (broken) apps depend on this.
    let wrapper = WrapDropTarget(hwnd);
    if wrapper.is_null() {
        IStream_Release(stream);
        return E_OUTOFMEMORY;
    }
    hr = CoMarshalInterface(
        (stream).cast(),
        (&raw const IID_IDropTarget).cast(),
        (wrapper as *mut IUnknown).cast(),
        MSHCTX_LOCAL,
        core::ptr::null_mut(),
        MSHLFLAGS_TABLESTRONG,
    );
    IDropTarget_Release(wrapper);

    if SUCCEEDED(hr) {
        hr = create_map_from_stream(stream, &mut map);
        if SUCCEEDED(hr) {
            IDropTarget_AddRef(pDropTarget);
            SetPropW(hwnd, prop_oledroptarget.as_ptr(), pDropTarget as HANDLE);
            SetPropW(hwnd, prop_marshalleddroptarget.as_ptr(), map);
        } else {
            let mut zero: LARGE_INTEGER = core::mem::zeroed();
            zero.QuadPart = 0;
            IStream_Seek(stream, zero, STREAM_SEEK_SET, core::ptr::null_mut());
            CoReleaseMarshalData((stream).cast());
        }
    }
    IStream_Release(stream);

    hr
}

/// ole2.c:639 RevokeDragDrop [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn RevokeDragDrop(hwnd: HWND) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), hwnd);

    if IsWindow(hwnd) == 0 {
        ERR!(b"invalid hwnd %p\n\0".as_ptr(), hwnd);
        return DRAGDROP_E_INVALIDHWND;
    }

    // no registration data
    let map = get_droptarget_handle(hwnd);
    if map.is_null() {
        return DRAGDROP_E_NOTREGISTERED;
    }

    let drop_target = GetPropW(hwnd, prop_oledroptarget.as_ptr()) as *mut IDropTarget;
    if !drop_target.is_null() {
        IDropTarget_Release(drop_target);
    }

    RemovePropW(hwnd, prop_oledroptarget.as_ptr());
    RemovePropW(hwnd, prop_marshalleddroptarget.as_ptr());

    let mut stream: *mut IStream = core::ptr::null_mut();
    let hr = create_stream_from_map(map, &mut stream);
    if SUCCEEDED(hr) {
        CoReleaseMarshalData((stream).cast());
        IStream_Release(stream);
    }
    CloseHandle(map);

    hr
}

/// ole2.c:678 OleRegGetUserType [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleRegGetUserType(
    clsid: REFCLSID,
    form: DWORD,
    usertype: *mut LPOLESTR,
) -> HRESULT {
    static auxusertypeW: [WCHAR; 15] = [
        'A' as u16, 'u' as u16, 'x' as u16, 'U' as u16, 's' as u16, 'e' as u16, 'r' as u16,
        'T' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '\\' as u16, '%' as u16, 'd' as u16, 0,
    ];

    let mut valuetype: DWORD = 0;
    let mut valuelen: DWORD;
    let mut auxkeynameW: [WCHAR; 16] = [0; 16];
    let mut usertypekey: HKEY = core::ptr::null_mut();

    TRACE!(
        b"(%s, %u, %p)\n\0".as_ptr(),
        debugstr_guid(clsid as *const c_void),
        form,
        usertype,
    );

    if usertype.is_null() {
        return E_INVALIDARG;
    }

    *usertype = core::ptr::null_mut();

    // Return immediately if it's not registered.
    let hres = COM_OpenKeyForCLSID(clsid, core::ptr::null(), KEY_READ, &mut usertypekey);
    if FAILED(hres) {
        return hres;
    }

    valuelen = 0;

    // Try additional types if requested. If they don't exist fall back to USERCLASSTYPE_FULL.
    if form != USERCLASSTYPE_FULL {
        let mut auxkey: HKEY = core::ptr::null_mut();

        wsprintfW(auxkeynameW.as_mut_ptr(), auxusertypeW.as_ptr(), form);

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

    *usertype = CoTaskMemAlloc(valuelen as SIZE_T) as LPOLESTR;
    if (*usertype).is_null() {
        RegCloseKey(usertypekey);
        return E_OUTOFMEMORY;
    }

    let ret = RegQueryValueExW(
        usertypekey,
        emptyW.as_ptr(),
        core::ptr::null_mut(),
        &mut valuetype,
        *usertype as *mut u8,
        &mut valuelen,
    );
    RegCloseKey(usertypekey);
    if ret != ERROR_SUCCESS {
        CoTaskMemFree(*usertype as LPVOID);
        *usertype = core::ptr::null_mut();
        return REGDB_E_READREGDB;
    }

    S_OK
}

/// ole2.c:753 DoDragDrop [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn DoDragDrop(
    pDataObject: *mut IDataObject,
    pDropSource: *mut IDropSource,
    dwOKEffect: DWORD,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    static trackerW: [WCHAR; 14] = [
        'T' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 'k' as u16, 'e' as u16, 'r' as u16,
        'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 0,
    ];

    TRACE!(
        b"(%p, %p, %08x, %p)\n\0".as_ptr(),
        pDataObject,
        pDropSource,
        dwOKEffect,
        pdwEffect,
    );

    if pDataObject.is_null() || pDropSource.is_null() || pdwEffect.is_null() {
        return E_INVALIDARG;
    }

    // Setup the drag n drop tracking window.
    let mut trackerInfo: TrackerWindowInfo = core::mem::zeroed();
    trackerInfo.dataObject = pDataObject;
    trackerInfo.dropSource = pDropSource;
    trackerInfo.dwOKEffect = dwOKEffect;
    trackerInfo.pdwEffect = pdwEffect;
    trackerInfo.trackingDone = 0;
    trackerInfo.inTrackCall = 0;
    trackerInfo.escPressed = 0;
    trackerInfo.curTargetHWND = core::ptr::null_mut();
    trackerInfo.curDragTarget = core::ptr::null_mut();
    trackerInfo.accepterHWND = core::ptr::null_mut();

    // CreateWindowW is a winuser.h macro over CreateWindowExW(dwExStyle = 0, ...).
    let hwndTrackWindow = CreateWindowExW(
        0,
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
        &mut trackerInfo as *mut TrackerWindowInfo as LPVOID,
    );

    if !hwndTrackWindow.is_null() {
        // Capture the mouse input
        SetCapture((hwndTrackWindow).cast());

        let mut msg: MSG = core::mem::zeroed();
        msg.message = 0;

        // Pump messages. All mouse input should go to the capture window.
        while trackerInfo.trackingDone == 0
            && GetMessageW(
                (&mut msg as *mut MSG as *mut c_void).cast(),
                core::ptr::null_mut(),
                0,
                0,
            ) != 0
        {
            trackerInfo.curMousePos.x = msg.pt.x;
            trackerInfo.curMousePos.y = msg.pt.y;
            trackerInfo.dwKeyState = OLEDD_GetButtonState();

            if msg.message >= WM_KEYFIRST && msg.message <= WM_KEYLAST {
                // Keyboard messages: notify the drop source of the state change;
                // Escape gets a special meaning.
                if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE as WPARAM {
                    trackerInfo.escPressed = 1;
                }

                OLEDD_TrackStateChange(&mut trackerInfo);
            } else {
                // Dispatch the messages only when it's not a keyboard message.
                DispatchMessageW((&msg as *const MSG as *const c_void).cast());
            }
        }

        // re-post the quit message to outer message loop
        if msg.message == WM_QUIT {
            PostQuitMessage(msg.wParam as c_int);
        }

        // Destroy the temporary window.
        DestroyWindow(hwndTrackWindow);

        return trackerInfo.returnValue;
    }

    E_FAIL
}

/// ole2.c:854 OleQueryLinkFromData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleQueryLinkFromData(pSrcDataObject: *mut IDataObject) -> HRESULT {
    FIXME!(b"(%p),stub!\n\0".as_ptr(), pSrcDataObject);
    S_FALSE
}

/// ole2.c:864 OleRegGetMiscStatus [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleRegGetMiscStatus(
    clsid: REFCLSID,
    dwAspect: DWORD,
    pdwStatus: *mut DWORD,
) -> HRESULT {
    static miscstatusW: [WCHAR; 11] = [
        'M' as u16, 'i' as u16, 's' as u16, 'c' as u16, 'S' as u16, 't' as u16, 'a' as u16,
        't' as u16, 'u' as u16, 's' as u16, 0,
    ];
    static dfmtW: [WCHAR; 3] = ['%' as u16, 'd' as u16, 0];

    let mut keyName: [WCHAR; 16] = [0; 16];
    let mut miscStatusKey: HKEY = core::ptr::null_mut();
    let mut aspectKey: HKEY = core::ptr::null_mut();

    TRACE!(
        b"(%s, %d, %p)\n\0".as_ptr(),
        debugstr_guid(clsid as *const c_void),
        dwAspect,
        pdwStatus,
    );

    if pdwStatus.is_null() {
        return E_INVALIDARG;
    }

    *pdwStatus = 0;

    if actctx_get_miscstatus(clsid, dwAspect, pdwStatus) != 0 {
        return S_OK;
    }

    let hr = COM_OpenKeyForCLSID(clsid, miscstatusW.as_ptr(), KEY_READ, &mut miscStatusKey);
    if FAILED(hr) {
        // missing key is not a failure
        return if hr == REGDB_E_KEYMISSING { S_OK } else { hr };
    }

    OLEUTL_ReadRegistryDWORDValue(miscStatusKey, pdwStatus);

    // Open the key specific to the requested aspect.
    wsprintfW(keyName.as_mut_ptr(), dfmtW.as_ptr(), dwAspect);

    let result = open_classes_key(miscStatusKey, keyName.as_ptr(), KEY_READ, &mut aspectKey);
    if result == ERROR_SUCCESS {
        OLEUTL_ReadRegistryDWORDValue(aspectKey, pdwStatus);
        RegCloseKey(aspectKey);
    }

    RegCloseKey(miscStatusKey);
    S_OK
}

/// ole2.c:919 impl_from_IEnumOLEVERB
#[inline]
unsafe fn impl_from_IEnumOLEVERB(iface: *mut IEnumOLEVERB) -> *mut EnumOLEVERB {
    CONTAINING_RECORD!(iface, EnumOLEVERB, IEnumOLEVERB_iface)
}

stdmethod! {
    /// ole2.c:924
    fn EnumOLEVERB_QueryInterface(
        iface: *mut IEnumOLEVERB,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        TRACE!(b"(%s, %p)\n\0".as_ptr(), debugstr_guid(riid as *const c_void), ppv);
        if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IEnumOLEVERB) {
            IEnumOLEVERB_AddRef(iface);
            *ppv = iface as *mut c_void;
            return S_OK;
        }
        E_NOINTERFACE
    }
}

stdmethod! {
    /// ole2.c:938
    fn EnumOLEVERB_AddRef(iface: *mut IEnumOLEVERB) -> ULONG {
        let This = impl_from_IEnumOLEVERB(iface);
        TRACE!(b"()\n\0".as_ptr());
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
    }
}

stdmethod! {
    /// ole2.c:946
    fn EnumOLEVERB_Release(iface: *mut IEnumOLEVERB) -> ULONG {
        let This = impl_from_IEnumOLEVERB(iface);
        let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;
        TRACE!(b"()\n\0".as_ptr());
        if refs == 0 {
            RegCloseKey((*This).hkeyVerb);
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
        }
        refs
    }
}

stdmethod! {
    /// ole2.c:960
    fn EnumOLEVERB_Next(
        iface: *mut IEnumOLEVERB,
        celt: ULONG,
        rgelt: LPOLEVERB,
        pceltFetched: *mut ULONG,
    ) -> HRESULT {
        let This = impl_from_IEnumOLEVERB(iface);
        let mut hr: HRESULT = S_OK;

        TRACE!(b"(%d, %p, %p)\n\0".as_ptr(), celt, rgelt, pceltFetched);

        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        let mut celt_remaining = celt;
        let mut rgelt_ptr = rgelt;

        while celt_remaining != 0 {
            let mut wszSubKey: [WCHAR; 20] = [0; 20];
            let mut cbData: LONG = 0;

            let res = RegEnumKeyW(
                (*This).hkeyVerb,
                (*This).index,
                wszSubKey.as_mut_ptr(),
                wszSubKey.len() as DWORD,
            );
            if res == ERROR_NO_MORE_ITEMS as LONG {
                hr = S_FALSE;
                break;
            } else if res != ERROR_SUCCESS {
                ERR!(b"RegEnumKeyW failed with error %d\n\0".as_ptr(), res);
                hr = REGDB_E_READREGDB;
                break;
            }

            let res = RegQueryValueW(
                (*This).hkeyVerb,
                wszSubKey.as_ptr(),
                core::ptr::null_mut(),
                &mut cbData,
            );
            if res != ERROR_SUCCESS {
                ERR!(b"RegQueryValueW failed with error %d\n\0".as_ptr(), res);
                hr = REGDB_E_READREGDB;
                break;
            }

            let pwszOLEVERB = CoTaskMemAlloc(cbData as SIZE_T) as LPWSTR;
            if pwszOLEVERB.is_null() {
                hr = E_OUTOFMEMORY;
                break;
            }

            let res = RegQueryValueW(
                (*This).hkeyVerb,
                wszSubKey.as_ptr(),
                pwszOLEVERB,
                &mut cbData,
            );
            if res != ERROR_SUCCESS {
                ERR!(b"RegQueryValueW failed with error %d\n\0".as_ptr(), res);
                hr = REGDB_E_READREGDB;
                CoTaskMemFree(pwszOLEVERB as LPVOID);
                break;
            }

            TRACE!(b"verb string: %s\n\0".as_ptr(), debugstr_w(pwszOLEVERB as *const c_void));
            let pwszMenuFlags = wcschr(pwszOLEVERB, ',' as u16);
            if pwszMenuFlags.is_null() {
                hr = OLEOBJ_E_INVALIDVERB;
                CoTaskMemFree(pwszOLEVERB as LPVOID);
                break;
            }
            // nul terminate the name string and advance to first character
            *pwszMenuFlags = 0;
            let pwszMenuFlags = pwszMenuFlags.add(1);

            let pwszAttribs = wcschr(pwszMenuFlags, ',' as u16);
            if pwszAttribs.is_null() {
                hr = OLEOBJ_E_INVALIDVERB;
                CoTaskMemFree(pwszOLEVERB as LPVOID);
                break;
            }
            // nul terminate the menu string and advance to first character
            *pwszAttribs = 0;
            let pwszAttribs = pwszAttribs.add(1);

            // fill out structure for this verb
            (*rgelt_ptr).lVerb = wcstol(wszSubKey.as_ptr(), core::ptr::null_mut(), 10);
            (*rgelt_ptr).lpszVerbName = pwszOLEVERB; // user should free
            (*rgelt_ptr).fuFlags = wcstol(pwszMenuFlags, core::ptr::null_mut(), 10) as DWORD;
            (*rgelt_ptr).grfAttribs = wcstol(pwszAttribs, core::ptr::null_mut(), 10) as DWORD;

            if !pceltFetched.is_null() {
                *pceltFetched += 1;
            }
            (*This).index += 1;

            celt_remaining -= 1;
            rgelt_ptr = rgelt_ptr.add(1);
        }

        hr
    }
}

stdmethod! {
    /// ole2.c:1048
    fn EnumOLEVERB_Skip(iface: *mut IEnumOLEVERB, celt: ULONG) -> HRESULT {
        let This = impl_from_IEnumOLEVERB(iface);

        TRACE!(b"(%d)\n\0".as_ptr(), celt);

        (*This).index = (*This).index.wrapping_add(celt);
        S_OK
    }
}

stdmethod! {
    /// ole2.c:1059
    fn EnumOLEVERB_Reset(iface: *mut IEnumOLEVERB) -> HRESULT {
        let This = impl_from_IEnumOLEVERB(iface);

        TRACE!(b"()\n\0".as_ptr());

        (*This).index = 0;
        S_OK
    }
}

stdmethod! {
    /// ole2.c:1070
    fn EnumOLEVERB_Clone(
        iface: *mut IEnumOLEVERB,
        ppenum: *mut *mut IEnumOLEVERB,
    ) -> HRESULT {
        let This = impl_from_IEnumOLEVERB(iface);
        let mut hkeyVerb: HKEY = core::ptr::null_mut();
        TRACE!(b"(%p)\n\0".as_ptr(), ppenum);
        if DuplicateHandle(
            GetCurrentProcess(),
            (*This).hkeyVerb as HANDLE,
            GetCurrentProcess(),
            &mut hkeyVerb as *mut HKEY as *mut HANDLE,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        ) == 0
        {
            return HRESULT_FROM_WIN32(GetLastError());
        }
        EnumOLEVERB_Construct(hkeyVerb, (*This).index, ppenum)
    }
}

/// ole2.c:1093 EnumOLEVERB_Construct
unsafe fn EnumOLEVERB_Construct(
    hkeyVerb: HKEY,
    index: ULONG,
    ppenum: *mut *mut IEnumOLEVERB,
) -> HRESULT {
    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumOLEVERB>() as usize,
    ) as *mut EnumOLEVERB;
    if This.is_null() {
        RegCloseKey(hkeyVerb);
        return E_OUTOFMEMORY;
    }
    (*This).IEnumOLEVERB_iface.lpVtbl =
        core::ptr::addr_of!(EnumOLEVERB_VTable) as *mut IEnumOLEVERBVtbl;
    (*This).ref_ = 1;
    (*This).index = index;
    (*This).hkeyVerb = hkeyVerb;
    *ppenum = core::ptr::addr_of_mut!((*This).IEnumOLEVERB_iface);
    S_OK
}

/// ole2.c:1125 OleRegEnumVerbs [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleRegEnumVerbs(
    clsid: REFCLSID,
    ppenum: *mut *mut IEnumOLEVERB,
) -> HRESULT {
    static wszVerb: [WCHAR; 5] = ['V' as u16, 'e' as u16, 'r' as u16, 'b' as u16, 0];

    TRACE!(
        b"(%s, %p)\n\0".as_ptr(),
        debugstr_guid(clsid as *const c_void),
        ppenum
    );

    let mut hkeyVerb: HKEY = core::ptr::null_mut();
    let res = COM_OpenKeyForCLSID(clsid, wszVerb.as_ptr(), KEY_READ, &mut hkeyVerb);
    if FAILED(res) {
        if res == REGDB_E_CLASSNOTREG {
            ERR!(
                b"CLSID %s not registered\n\0".as_ptr(),
                debugstr_guid(clsid as *const c_void)
            );
        } else if res == REGDB_E_KEYMISSING {
            ERR!(
                b"no Verbs key for class %s\n\0".as_ptr(),
                debugstr_guid(clsid as *const c_void)
            );
        } else {
            ERR!(
                b"failed to open Verbs key for CLSID %s with error %d\n\0".as_ptr(),
                debugstr_guid(clsid as *const c_void),
                res
            );
        }
        return res;
    }

    let mut dwSubKeys: DWORD = 0;
    let res = RegQueryInfoKeyW(
        hkeyVerb,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSubKeys,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if res != ERROR_SUCCESS {
        ERR!(
            b"failed to get subkey count with error %d\n\0".as_ptr(),
            GetLastError()
        );
        return REGDB_E_READREGDB;
    }

    if dwSubKeys == 0 {
        WARN!(
            b"class %s has no verbs\n\0".as_ptr(),
            debugstr_guid(clsid as *const c_void)
        );
        RegCloseKey(hkeyVerb);
        return OLEOBJ_E_NOVERBS;
    }

    EnumOLEVERB_Construct(hkeyVerb, 0, ppenum)
}

/// ole2.c:1168 OleSetContainedObject [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleSetContainedObject(
    pUnknown: LPUNKNOWN,
    fContained: BOOL,
) -> HRESULT {
    let mut runnable: *mut IRunnableObject = core::ptr::null_mut();

    TRACE!(b"(%p,%x)\n\0".as_ptr(), pUnknown, fContained);

    let mut hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runnable as *mut *mut IRunnableObject as *mut *mut c_void,
    );

    if SUCCEEDED(hres) {
        hres = IRunnableObject_SetContainedObject(runnable, fContained);

        IRunnableObject_Release(runnable);

        return hres;
    }

    S_OK
}

/// ole2.c:1205 OleRun [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleRun(pUnknown: LPUNKNOWN) -> HRESULT {
    let mut runable: *mut IRunnableObject = core::ptr::null_mut();

    TRACE!(b"(%p)\n\0".as_ptr(), pUnknown);

    let hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runable as *mut *mut IRunnableObject as *mut *mut c_void,
    );
    if FAILED(hres) {
        return S_OK; // Appears to return no error.
    }

    let hres = IRunnableObject_Run(runable, core::ptr::null_mut());
    IRunnableObject_Release(runable);
    hres
}

/// ole2.c:1224 OleLoad [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleLoad(
    pStg: LPSTORAGE,
    riid: REFIID,
    pClientSite: LPOLECLIENTSITE,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut persistStorage: *mut IPersistStorage = core::ptr::null_mut();
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let mut pOleObject: *mut IOleObject = core::ptr::null_mut();
    let mut storageInfo: STATSTG = core::mem::zeroed();

    TRACE!(
        b"(%p, %s, %p, %p)\n\0".as_ptr(),
        pStg,
        debugstr_guid(riid as *const c_void),
        pClientSite,
        ppvObj,
    );

    *ppvObj = core::ptr::null_mut();

    // Get the class ID for the object.
    let mut hres = IStorage_Stat(pStg, &mut storageInfo, STATFLAG_NONAME);
    if FAILED(hres) {
        return hres;
    }

    // Now, try and create the handler for the object
    hres = CoCreateInstance(
        (&raw const storageInfo.clsid).cast(),
        core::ptr::null_mut(),
        CLSCTX_INPROC_HANDLER | CLSCTX_INPROC_SERVER,
        (riid).cast(),
        &mut pUnk as *mut *mut IUnknown as *mut LPVOID,
    );

    // If that fails, as it will most times, load the default OLE handler.
    if FAILED(hres) {
        hres = OleCreateDefaultHandler(
            &storageInfo.clsid,
            core::ptr::null_mut(),
            riid,
            &mut pUnk as *mut *mut IUnknown as *mut LPVOID,
        );
    }

    // If we couldn't find a handler... this is bad. Abort the whole thing.
    if FAILED(hres) {
        return hres;
    }

    if !pClientSite.is_null() {
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleObject,
            &mut pOleObject as *mut *mut IOleObject as *mut *mut c_void,
        );
        if SUCCEEDED(hres) {
            let mut dwStatus: DWORD = 0;
            IOleObject_GetMiscStatus(pOleObject, DVASPECT_CONTENT, &mut dwStatus);
        }
    }

    // Initialize the object with its IPersistStorage interface.
    hres = IUnknown_QueryInterface(
        pUnk,
        &IID_IPersistStorage,
        &mut persistStorage as *mut *mut IPersistStorage as *mut *mut c_void,
    );
    if SUCCEEDED(hres) {
        hres = IPersistStorage_Load(persistStorage, pStg);

        IPersistStorage_Release(persistStorage);
        persistStorage = core::ptr::null_mut();
        let _ = persistStorage;
    }

    if SUCCEEDED(hres) && !pClientSite.is_null() {
        // Inform the new object of its client site.
        hres = IOleObject_SetClientSite(pOleObject, pClientSite);
    }

    // Cleanup interfaces used internally
    if !pOleObject.is_null() {
        IOleObject_Release(pOleObject);
    }

    if SUCCEEDED(hres) {
        let mut pOleLink: *mut IOleLink = core::ptr::null_mut();
        let hres1 = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleLink,
            &mut pOleLink as *mut *mut IOleLink as *mut *mut c_void,
        );
        if SUCCEEDED(hres1) {
            FIXME!(b"handle OLE link\n\0".as_ptr());
            IOleLink_Release(pOleLink);
        }
    }

    if FAILED(hres) {
        IUnknown_Release(pUnk);
        pUnk = core::ptr::null_mut();
    }

    *ppvObj = pUnk as LPVOID;

    hres
}

/// ole2.c:1338 OleSave [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleSave(
    pPS: LPPERSISTSTORAGE,
    pStg: LPSTORAGE,
    fSameAsLoad: BOOL,
) -> HRESULT {
    let mut objectClass: CLSID = core::mem::zeroed();

    TRACE!(b"(%p,%p,%x)\n\0".as_ptr(), pPS, pStg, fSameAsLoad);

    // First, we transfer the class ID (if available)
    let mut hres = IPersistStorage_GetClassID(pPS, &mut objectClass);

    if SUCCEEDED(hres) {
        WriteClassStg(pStg, &objectClass);
    }

    // Then, we ask the object to save itself to the storage. If it is
    // successful, we commit the storage.
    hres = IPersistStorage_Save(pPS, pStg, fSameAsLoad);

    if SUCCEEDED(hres) {
        IStorage_Commit(pStg, STGC_DEFAULT);
    }

    hres
}

/// ole2.c:1377 OleLockRunning [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleLockRunning(
    pUnknown: LPUNKNOWN,
    fLock: BOOL,
    fLastUnlockCloses: BOOL,
) -> HRESULT {
    let mut runnable: *mut IRunnableObject = core::ptr::null_mut();

    TRACE!(
        b"(%p,%x,%x)\n\0".as_ptr(),
        pUnknown,
        fLock,
        fLastUnlockCloses,
    );

    let mut hres = IUnknown_QueryInterface(
        pUnknown,
        &IID_IRunnableObject,
        &mut runnable as *mut *mut IRunnableObject as *mut *mut c_void,
    );

    if SUCCEEDED(hres) {
        hres = IRunnableObject_LockRunning(runnable, fLock, fLastUnlockCloses);

        IRunnableObject_Release(runnable);

        return hres;
    }

    S_OK
}

/// ole2.c:1411 OLEMenu_Initialize
unsafe fn OLEMenu_Initialize() {}

/// ole2.c:1420 OLEMenu_UnInitialize
unsafe fn OLEMenu_UnInitialize() {}

/// ole2.c:1431 OLEMenu_InstallHooks
unsafe fn OLEMenu_InstallHooks(tid: DWORD) -> BOOL {
    // Create an entry for the hook table
    let pHookItem = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<OleMenuHookItem>() as usize,
    ) as *mut OleMenuHookItem;
    if pHookItem.is_null() {
        return 0;
    }

    (*pHookItem).tid = tid;
    (*pHookItem).hHeap = GetProcessHeap();
    (*pHookItem).CallWndProc_hHook = core::ptr::null_mut();

    // Install a thread scope message hook for WH_GETMESSAGE
    (*pHookItem).GetMsg_hHook = SetWindowsHookExW(
        WH_GETMESSAGE,
        Some(OLEMenu_GetMsgProc),
        core::ptr::null_mut(),
        GetCurrentThreadId(),
    );
    if (*pHookItem).GetMsg_hHook.is_null() {
        return OLEMenu_InstallHooks_cleanup(pHookItem);
    }

    // Install a thread scope message hook for WH_CALLWNDPROC
    (*pHookItem).CallWndProc_hHook = SetWindowsHookExW(
        WH_CALLWNDPROC,
        Some(OLEMenu_CallWndProc),
        core::ptr::null_mut(),
        GetCurrentThreadId(),
    );
    if (*pHookItem).CallWndProc_hHook.is_null() {
        return OLEMenu_InstallHooks_cleanup(pHookItem);
    }

    // Insert the hook table entry
    (*pHookItem).next = hook_list;
    hook_list = pHookItem;

    1
}

/// ole2.c:1462 the `CLEANUP:` label of OLEMenu_InstallHooks
unsafe fn OLEMenu_InstallHooks_cleanup(pHookItem: *mut OleMenuHookItem) -> BOOL {
    // Unhook any hooks
    if !(*pHookItem).GetMsg_hHook.is_null() {
        UnhookWindowsHookEx((*pHookItem).GetMsg_hHook);
    }
    if !(*pHookItem).CallWndProc_hHook.is_null() {
        UnhookWindowsHookEx((*pHookItem).CallWndProc_hHook);
    }
    // Release the hook table entry
    HeapFree((*pHookItem).hHeap, 0, pHookItem as LPVOID);

    0
}

/// ole2.c:1481 OLEMenu_UnInstallHooks
unsafe fn OLEMenu_UnInstallHooks(tid: DWORD) -> BOOL {
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
        // Uninstall the hooks installed for this thread
        if UnhookWindowsHookEx((*pHookItem).GetMsg_hHook) == 0 {
            break 'CLEANUP;
        }
        if UnhookWindowsHookEx((*pHookItem).CallWndProc_hHook) == 0 {
            break 'CLEANUP;
        }

        // Release the hook table entry
        HeapFree((*pHookItem).hHeap, 0, pHookItem as LPVOID);

        return 1;
    }

    // CLEANUP: release the hook table entry
    HeapFree((*pHookItem).hHeap, 0, pHookItem as LPVOID);

    0
}

/// ole2.c:1523 OLEMenu_IsHookInstalled
unsafe fn OLEMenu_IsHookInstalled(tid: DWORD) -> *mut OleMenuHookItem {
    // Do a simple linear search for an entry whose tid matches ours.
    let mut pHookItem = hook_list;
    while !pHookItem.is_null() {
        if tid == (*pHookItem).tid {
            return pHookItem;
        }
        pHookItem = (*pHookItem).next;
    }

    core::ptr::null_mut()
}

/// ole2.c:1546 OLEMenu_FindMainMenuIndex
unsafe fn OLEMenu_FindMainMenuIndex(hMainMenu: HMENU, hPopupMenu: HMENU, pnPos: *mut UINT) -> BOOL {
    let nItems: c_int = GetMenuItemCount(hMainMenu);

    let mut i: c_int = 0;
    while i < nItems {
        // Is the current item a submenu?
        let hsubmenu: HMENU = GetSubMenu(hMainMenu, i);

        if !hsubmenu.is_null() {
            // If the handle is the same we're done
            if hsubmenu == hPopupMenu {
                if !pnPos.is_null() {
                    *pnPos = i as UINT;
                }
                return 1;
            }
            // Recursively search without updating pnPos
            else if OLEMenu_FindMainMenuIndex(hsubmenu, hPopupMenu, core::ptr::null_mut()) != 0 {
                if !pnPos.is_null() {
                    *pnPos = i as UINT;
                }
                return 1;
            }
        }

        i += 1;
    }

    0
}

/// ole2.c:1589 OLEMenu_SetIsServerMenu
unsafe fn OLEMenu_SetIsServerMenu(hmenu: HMENU, pOleMenuDescriptor: *mut OleMenuDescriptor) -> BOOL {
    let mut nPos: UINT = 0;

    (*pOleMenuDescriptor).bIsServerItem = 0;

    // Don't bother searching if the popup is the combined menu itself
    if hmenu == (*pOleMenuDescriptor).hmenuCombined {
        return 0;
    }

    // Find the menu item index in the shared OLE menu that this item belongs to
    if OLEMenu_FindMainMenuIndex((*pOleMenuDescriptor).hmenuCombined, hmenu, &mut nPos) == 0 {
        return 0;
    }

    // The group widths array has counts for the number of elements in the groups
    // File, Edit, Container, Object, Window, Help. The Edit, Object & Help groups
    // belong to the server object and the other three belong to the container.
    let mut nWidth: UINT = 0;
    let mut i: UINT = 0;
    while i < 6 {
        nWidth += (*pOleMenuDescriptor).mgw.width[i as usize] as UINT;
        if nPos < nWidth {
            // Odd elements are server menu widths
            (*pOleMenuDescriptor).bIsServerItem = (i % 2) as BOOL;
            break;
        }
        i += 1;
    }

    (*pOleMenuDescriptor).bIsServerItem
}

/// ole2.c:1712 — the shared `NEXTHOOK:` tail of OLEMenu_CallWndProc.
unsafe fn OLEMenu_CallWndProc_nexthook(
    hOleMenu: HOLEMENU,
    pOleMenuDescriptor: *mut OleMenuDescriptor,
    code: c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if !pOleMenuDescriptor.is_null() {
        GlobalUnlock(hOleMenu);
    }

    // Lookup the hook item for the current thread
    let pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
    if pHookItem.is_null() {
        // This should never fail!!
        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr());
        return 0;
    }

    // Pass on the message to the next hooker
    CallNextHookEx((*pHookItem).CallWndProc_hHook, code, wParam, lParam)
}

/// ole2.c:1628 OLEMenu_CallWndProc — WH_CALLWNDPROC filter (callback)
unsafe extern "system" fn OLEMenu_CallWndProc(
    code: c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut hOleMenu: HOLEMENU = core::ptr::null_mut();
    let mut pOleMenuDescriptor: *mut OleMenuDescriptor = core::ptr::null_mut();

    TRACE!(b"%i, %04lx, %08lx\n\0".as_ptr(), code, wParam, lParam);

    // Check if we're being asked to process the message
    if code != HC_ACTION as c_int {
        return OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // Retrieve the current message being dispatched from lParam
    let pMsg: LPCWPSTRUCT = lParam as LPCWPSTRUCT;

    // Check if the message is destined for a window we are interested in.
    hOleMenu = GetPropW((*pMsg).hwnd, prop_olemenuW.as_ptr()) as HOLEMENU;
    if hOleMenu.is_null() {
        return OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // Get the menu descriptor
    pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        // Bad descriptor!
        return OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // Process menu messages
    match (*pMsg).message {
        WM_INITMENU => {
            // Reset the menu descriptor state
            (*pOleMenuDescriptor).bIsServerItem = 0;

            // Send this message to the server as well
            SendMessageW(
                ((*pOleMenuDescriptor).hwndActiveObject).cast(),
                (*pMsg).message,
                ((*pMsg).wParam) as usize,
                ((*pMsg).lParam) as isize,
            );
            return OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
        }
        WM_INITMENUPOPUP => {
            // Save the state for whether this is a server owned menu
            OLEMenu_SetIsServerMenu((*pMsg).wParam as HMENU, pOleMenuDescriptor);
        }
        WM_MENUSELECT => {
            let fuFlags: WORD = HIWORD((*pMsg).wParam as DWORD); // Get flags
            if (fuFlags as DWORD & MF_SYSMENU) != 0 {
                return OLEMenu_CallWndProc_nexthook(
                    hOleMenu,
                    pOleMenuDescriptor,
                    code,
                    wParam,
                    lParam,
                );
            }
            // Save the state for whether this is a server owned popup menu
            else if (fuFlags as DWORD & MF_POPUP) != 0 {
                OLEMenu_SetIsServerMenu((*pMsg).lParam as HMENU, pOleMenuDescriptor);
            }
        }
        WM_DRAWITEM => {
            let lpdis: LPDRAWITEMSTRUCT = (*pMsg).lParam as LPDRAWITEMSTRUCT;
            if (*pMsg).wParam != 0 || (*lpdis).CtlType != ODT_MENU as UINT {
                // Not a menu message
                return OLEMenu_CallWndProc_nexthook(
                    hOleMenu,
                    pOleMenuDescriptor,
                    code,
                    wParam,
                    lParam,
                );
            }
        }
        _ => {
            return OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
        }
    }

    // If the message was for the server dispatch it accordingly
    if (*pOleMenuDescriptor).bIsServerItem != 0 {
        SendMessageW(
            ((*pOleMenuDescriptor).hwndActiveObject).cast(),
            (*pMsg).message,
            ((*pMsg).wParam) as usize,
            ((*pMsg).lParam) as isize,
        );
    }

    OLEMenu_CallWndProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam)
}

/// ole2.c:1786 — the shared `NEXTHOOK:` tail of OLEMenu_GetMsgProc.
unsafe fn OLEMenu_GetMsgProc_nexthook(
    hOleMenu: HOLEMENU,
    pOleMenuDescriptor: *mut OleMenuDescriptor,
    code: c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if !pOleMenuDescriptor.is_null() {
        GlobalUnlock(hOleMenu);
    }

    // Lookup the hook item for the current thread
    let pHookItem = OLEMenu_IsHookInstalled(GetCurrentThreadId());
    if pHookItem.is_null() {
        // This should never fail!!
        WARN!(b"could not retrieve hHook for current thread!\n\0".as_ptr());
        return 0;
    }

    // Pass on the message to the next hooker
    CallNextHookEx((*pHookItem).GetMsg_hHook, code, wParam, lParam)
}

/// ole2.c:1733 OLEMenu_GetMsgProc — WH_GETMESSAGE filter (callback)
unsafe extern "system" fn OLEMenu_GetMsgProc(
    code: c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut hOleMenu: HOLEMENU = core::ptr::null_mut();
    let mut pOleMenuDescriptor: *mut OleMenuDescriptor = core::ptr::null_mut();

    TRACE!(b"%i, %04lx, %08lx\n\0".as_ptr(), code, wParam, lParam);

    // Check if we're being asked to process a messages
    if code != HC_ACTION as c_int {
        return OLEMenu_GetMsgProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // Retrieve the current message being dispatched from lParam
    let pMsg: LPMSG = lParam as LPMSG;

    // Check if the message is destined for a window we are interested in.
    hOleMenu = GetPropW((*pMsg).hwnd, prop_olemenuW.as_ptr()) as HOLEMENU;
    if hOleMenu.is_null() {
        return OLEMenu_GetMsgProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // Process menu messages
    match (*pMsg).message {
        WM_COMMAND => {
            let wCode: WORD = HIWORD((*pMsg).wParam as DWORD); // Get notification code
            if wCode != 0 {
                // Not a menu message
                return OLEMenu_GetMsgProc_nexthook(
                    hOleMenu,
                    pOleMenuDescriptor,
                    code,
                    wParam,
                    lParam,
                );
            }
        }
        _ => {
            return OLEMenu_GetMsgProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
        }
    }

    // Get the menu descriptor
    pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        // Bad descriptor!
        return OLEMenu_GetMsgProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam);
    }

    // If the message was for the server dispatch it accordingly
    if (*pOleMenuDescriptor).bIsServerItem != 0 {
        // Change the hWnd in the message to the active objects hWnd. The message
        // loop which reads this message will automatically dispatch it to the
        // embedded objects window.
        (*pMsg).hwnd = (*pOleMenuDescriptor).hwndActiveObject;
    }

    OLEMenu_GetMsgProc_nexthook(hOleMenu, pOleMenuDescriptor, code, wParam, lParam)
}

/// ole2.c:1812 OleCreateMenuDescriptor [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateMenuDescriptor(
    hmenuCombined: HMENU,
    lpMenuWidths: LPOLEMENUGROUPWIDTHS,
) -> HOLEMENU {
    if hmenuCombined.is_null() || lpMenuWidths.is_null() {
        return core::ptr::null_mut();
    }

    // Create an OLE menu descriptor
    let hOleMenu: HOLEMENU = GlobalAlloc(
        GMEM_MOVEABLE | GMEM_ZEROINIT,
        core::mem::size_of::<OleMenuDescriptor>() as SIZE_T,
    );
    if hOleMenu.is_null() {
        return core::ptr::null_mut();
    }

    let pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
    if pOleMenuDescriptor.is_null() {
        return core::ptr::null_mut();
    }

    // Initialize menu group widths and hmenu
    for i in 0..6 {
        (*pOleMenuDescriptor).mgw.width[i] = (*lpMenuWidths).width[i];
    }

    (*pOleMenuDescriptor).hmenuCombined = hmenuCombined;
    (*pOleMenuDescriptor).bIsServerItem = 0;
    GlobalUnlock(hOleMenu);

    hOleMenu
}

/// ole2.c:1847 OleDestroyMenuDescriptor [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleDestroyMenuDescriptor(hmenuDescriptor: HOLEMENU) -> HRESULT {
    if !hmenuDescriptor.is_null() {
        GlobalFree(hmenuDescriptor);
    }
    S_OK
}

/// ole2.c:1875 OleSetMenuDescriptor [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleSetMenuDescriptor(
    hOleMenu: HOLEMENU,
    hwndFrame: HWND,
    hwndActiveObject: HWND,
    lpFrame: LPOLEINPLACEFRAME,
    lpActiveObject: LPOLEINPLACEACTIVEOBJECT,
) -> HRESULT {
    // Check args
    if hwndFrame.is_null() || (!hOleMenu.is_null() && hwndActiveObject.is_null()) {
        return E_INVALIDARG;
    }

    if !lpFrame.is_null() || !lpActiveObject.is_null() {
        FIXME!(
            b"(%p, %p, %p, %p, %p), Context sensitive help filtering not implemented!\n\0".as_ptr(),
            hOleMenu,
            hwndFrame,
            hwndActiveObject,
            lpFrame,
            lpActiveObject,
        );
    }

    // Set up a message hook to intercept the containers frame window messages.
    if !hOleMenu.is_null() {
        // Want to install dispatching code.
        // If OLEMenu hooks are already installed for this thread, fail.
        if !OLEMenu_IsHookInstalled(GetCurrentThreadId()).is_null() {
            return E_FAIL;
        }

        // Get the menu descriptor
        let pOleMenuDescriptor = GlobalLock(hOleMenu) as *mut OleMenuDescriptor;
        if pOleMenuDescriptor.is_null() {
            return E_UNEXPECTED;
        }

        // Update the menu descriptor
        (*pOleMenuDescriptor).hwndFrame = hwndFrame;
        (*pOleMenuDescriptor).hwndActiveObject = hwndActiveObject;

        GlobalUnlock(hOleMenu);

        // Add a menu descriptor windows property to the frame window
        SetPropW(hwndFrame, prop_olemenuW.as_ptr(), hOleMenu);

        // Install thread scope message hooks for WH_GETMESSAGE and WH_CALLWNDPROC
        if OLEMenu_InstallHooks(GetCurrentThreadId()) == 0 {
            return E_FAIL;
        }
    } else {
        // Want to uninstall dispatching code — uninstall the hooks
        if OLEMenu_UnInstallHooks(GetCurrentThreadId()) == 0 {
            return E_FAIL;
        }

        // Remove the menu descriptor property from the frame window
        RemovePropW(hwndFrame, prop_olemenuW.as_ptr());
    }

    S_OK
}

/// ole2.c:1949 IsAccelerator [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn IsAccelerator(
    hAccel: HACCEL,
    cAccelEntries: c_int,
    lpMsg: LPMSG,
    lpwCmd: *mut WORD,
) -> BOOL {
    if lpMsg.is_null() {
        return 0;
    }
    if hAccel.is_null() {
        WARN!(b"NULL accel handle\n\0".as_ptr());
        return 0;
    }
    if (*lpMsg).message != WM_KEYDOWN
        && (*lpMsg).message != WM_SYSKEYDOWN
        && (*lpMsg).message != WM_SYSCHAR
        && (*lpMsg).message != WM_CHAR
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
    if CopyAcceleratorTableW(hAccel as HANDLE, lpAccelTbl as *mut c_void, cAccelEntries)
        != cAccelEntries
    {
        WARN!(b"CopyAcceleratorTableW failed\n\0".as_ptr());
        HeapFree(GetProcessHeap(), 0, lpAccelTbl as LPVOID);
        return 0;
    }

    TRACE!(
        b"hAccel=%p, cAccelEntries=%d, msg->hwnd=%p, msg->message=%04x, wParam=%08lx, lParam=%08lx\n\0"
            .as_ptr(),
        hAccel,
        cAccelEntries,
        (*lpMsg).hwnd,
        (*lpMsg).message,
        (*lpMsg).wParam,
        (*lpMsg).lParam,
    );

    let mut i: c_int = 0;
    while i < cAccelEntries {
        let entry = &*lpAccelTbl.offset(i as isize);
        if entry.key != (*lpMsg).wParam as WORD {
            i += 1;
            continue;
        }

        if (*lpMsg).message == WM_CHAR {
            if (entry.fVirt as DWORD & FALT) == 0 && (entry.fVirt as DWORD & FVIRTKEY) == 0 {
                TRACE!(b"found accel for WM_CHAR: ('%c')\n\0".as_ptr(), (*lpMsg).wParam & 0xff);
                // found:
                if !lpwCmd.is_null() {
                    *lpwCmd = entry.cmd;
                }
                HeapFree(GetProcessHeap(), 0, lpAccelTbl as LPVOID);
                return 1;
            }
        } else if (entry.fVirt as DWORD & FVIRTKEY) != 0 {
            TRACE!(
                b"found accel for virt_key %04lx (scan %04x)\n\0".as_ptr(),
                (*lpMsg).wParam,
                ((*lpMsg).lParam >> 16) & 0xff,
            );
            let mut mask: DWORD = 0;
            if (GetKeyState(VK_SHIFT as c_int) as c_int & 0x8000) != 0 {
                mask |= FSHIFT;
            }
            if (GetKeyState(VK_CONTROL as c_int) as c_int & 0x8000) != 0 {
                mask |= FCONTROL;
            }
            if (GetKeyState(VK_MENU as c_int) as c_int & 0x8000) != 0 {
                mask |= FALT;
            }
            if mask == (entry.fVirt as DWORD & (FSHIFT | FCONTROL | FALT)) {
                // found:
                if !lpwCmd.is_null() {
                    *lpwCmd = entry.cmd;
                }
                HeapFree(GetProcessHeap(), 0, lpAccelTbl as LPVOID);
                return 1;
            }
            TRACE!(b"incorrect SHIFT/CTRL/ALT-state\n\0".as_ptr());
        } else if ((*lpMsg).lParam & 0x0100_0000) == 0 {
            // no special_key
            if (entry.fVirt as DWORD & FALT) != 0 && ((*lpMsg).lParam & 0x2000_0000) != 0 {
                // ALT pressed
                TRACE!(b"found accel for Alt-%c\n\0".as_ptr(), (*lpMsg).wParam & 0xff);
                // found:
                if !lpwCmd.is_null() {
                    *lpwCmd = entry.cmd;
                }
                HeapFree(GetProcessHeap(), 0, lpAccelTbl as LPVOID);
                return 1;
            }
        }
        i += 1;
    }

    WARN!(b"couldn't translate accelerator key\n\0".as_ptr());
    HeapFree(GetProcessHeap(), 0, lpAccelTbl as LPVOID);
    0
}

/// ole2.c:2033 ReleaseStgMedium [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn ReleaseStgMedium(pmedium: *mut STGMEDIUM) {
    // ole2.c: `if (!pmedium) return;`
    if pmedium.is_null() {
        return;
    }
    match (*pmedium).tymed {
        TYMED_HGLOBAL => {
            if (*pmedium).pUnkForRelease.is_null()
                && !(*pmedium).__bindgen_anon_1.hGlobal.is_null()
            {
                GlobalFree((*pmedium).__bindgen_anon_1.hGlobal);
            }
        }
        TYMED_FILE => {
            if !(*pmedium).__bindgen_anon_1.lpszFileName.is_null() {
                if (*pmedium).pUnkForRelease.is_null() {
                    DeleteFileW((*pmedium).__bindgen_anon_1.lpszFileName);
                }

                CoTaskMemFree((*pmedium).__bindgen_anon_1.lpszFileName as LPVOID);
            }
        }
        TYMED_ISTREAM => {
            if !(*pmedium).__bindgen_anon_1.pstm.is_null() {
                IStream_Release((*pmedium).__bindgen_anon_1.pstm);
            }
        }
        TYMED_ISTORAGE => {
            if !(*pmedium).__bindgen_anon_1.pstg.is_null() {
                IStorage_Release((*pmedium).__bindgen_anon_1.pstg);
            }
        }
        TYMED_GDI => {
            if (*pmedium).pUnkForRelease.is_null()
                && !(*pmedium).__bindgen_anon_1.hBitmap.is_null()
            {
                DeleteObject((*pmedium).__bindgen_anon_1.hBitmap as HGDIOBJ);
            }
        }
        TYMED_MFPICT => {
            if (*pmedium).pUnkForRelease.is_null()
                && !(*pmedium).__bindgen_anon_1.hMetaFilePict.is_null()
            {
                let pMP = GlobalLock((*pmedium).__bindgen_anon_1.hMetaFilePict) as LPMETAFILEPICT;
                DeleteMetaFile((*pMP).hMF);
                GlobalUnlock((*pmedium).__bindgen_anon_1.hMetaFilePict);
                GlobalFree((*pmedium).__bindgen_anon_1.hMetaFilePict);
            }
        }
        TYMED_ENHMF => {
            if (*pmedium).pUnkForRelease.is_null()
                && !(*pmedium).__bindgen_anon_1.hEnhMetaFile.is_null()
            {
                DeleteEnhMetaFile((*pmedium).__bindgen_anon_1.hEnhMetaFile);
            }
        }
        // TYMED_NULL and everything else
        _ => {}
    }
    (*pmedium).tymed = TYMED_NULL;

    // After cleaning up, the unknown is released
    if !(*pmedium).pUnkForRelease.is_null() {
        IUnknown_Release((*pmedium).pUnkForRelease);
        (*pmedium).pUnkForRelease = core::ptr::null_mut();
    }
}

/// ole2.c:2123 OLEDD_Initialize
unsafe fn OLEDD_Initialize() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    wndClass.style = CS_GLOBALCLASS;
    wndClass.lpfnWndProc = Some(OLEDD_DragTrackerWindowProc);
    wndClass.cbClsExtra = 0;
    wndClass.cbWndExtra = core::mem::size_of::<*mut TrackerWindowInfo>() as c_int;
    wndClass.hCursor = core::ptr::null_mut();
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = OLEDD_DRAGTRACKERCLASS.as_ptr();

    RegisterClassW((&raw const wndClass).cast());
}

/// ole2.c:2150 OLEDD_DragTrackerWindowProc
unsafe extern "system" fn OLEDD_DragTrackerWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            let createStruct = lParam as LPCREATESTRUCTA;

            set_window_longptr_w(hwnd, 0, (*createStruct).lpCreateParams as LONG_PTR);
            SetTimer(hwnd, DRAG_TIMER_ID, 50, None);
        }
        WM_TIMER
        | WM_MOUSEMOVE
        | WM_LBUTTONUP
        | WM_MBUTTONUP
        | WM_RBUTTONUP
        | WM_LBUTTONDOWN
        | WM_MBUTTONDOWN
        | WM_RBUTTONDOWN => {
            let trackerInfo = get_window_longptr_a(hwnd, 0) as *mut TrackerWindowInfo;
            if (*trackerInfo).trackingDone == 0 {
                OLEDD_TrackStateChange(trackerInfo);
            }
        }
        WM_DESTROY => {
            KillTimer(hwnd, DRAG_TIMER_ID);
        }
        _ => {}
    }

    // This is a window proc after all. Let's call the default.
    (DefWindowProcW((hwnd).cast(), uMsg, (wParam) as usize, (lParam) as isize)) as _
}

/// ole2.c:2195 DefaultDragEnter (ReactOS)
unsafe fn DefaultDragEnter(
    _hwndTarget: HWND,
    pDataObj: *mut IDataObject,
    _grfKeyState: DWORD,
    _pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut fme: FORMATETC = core::mem::zeroed();

    ZeroMemory(
        &mut fme as *mut FORMATETC as *mut c_void,
        core::mem::size_of::<FORMATETC>(),
    );
    fme.cfFormat = CF_HDROP as CLIPFORMAT;
    fme.ptd = core::ptr::null_mut();
    fme.dwAspect = DVASPECT_CONTENT;
    fme.lindex = -1;
    fme.tymed = TYMED_HGLOBAL;
    let hr = IDataObject_QueryGetData(pDataObj, &mut fme);

    *pdwEffect = if SUCCEEDED(hr) {
        DROPEFFECT_COPY
    } else {
        DROPEFFECT_NONE
    };

    if *pdwEffect == DROPEFFECT_NONE {
        return DRAGDROP_S_CANCEL;
    }

    S_OK
}

/// ole2.c:2220 DefaultDrop (ReactOS)
unsafe fn DefaultDrop(
    hwndAccepter: HWND,
    pDataObj: *mut IDataObject,
    _grfKeyState: DWORD,
    _pt: POINTL,
    _pdwEffect: *mut DWORD,
) -> HRESULT {
    let mut fme: FORMATETC = core::mem::zeroed();
    let mut stgm: STGMEDIUM = core::mem::zeroed();

    ZeroMemory(
        &mut fme as *mut FORMATETC as *mut c_void,
        core::mem::size_of::<FORMATETC>(),
    );
    fme.cfFormat = CF_HDROP as CLIPFORMAT;
    fme.ptd = core::ptr::null_mut();
    fme.dwAspect = DVASPECT_CONTENT;
    fme.lindex = -1;
    fme.tymed = TYMED_HGLOBAL;
    let mut hr = IDataObject_QueryGetData(pDataObj, &mut fme);
    if FAILED(hr) {
        return hr;
    }

    ZeroMemory(
        &mut stgm as *mut STGMEDIUM as *mut c_void,
        core::mem::size_of::<STGMEDIUM>(),
    );
    hr = IDataObject_GetData(pDataObj, &mut fme, &mut stgm);
    if SUCCEEDED(hr) {
        let hGlobal: HGLOBAL = stgm.__bindgen_anon_1.hGlobal;
        if !hGlobal.is_null() {
            if IsWindowUnicode(hwndAccepter) != 0 {
                PostMessageW((hwndAccepter).cast(), WM_DROPFILES, hGlobal as usize, 0);
            } else {
                PostMessageA(hwndAccepter, WM_DROPFILES, hGlobal as WPARAM, 0);
            }
        }
        ReleaseStgMedium(&mut stgm);
    }

    hr
}

/// ole2.c:2260 drag_enter
unsafe fn drag_enter(info: *mut TrackerWindowInfo, mut new_target: HWND) {
    let mut dwEffect: DWORD;

    (*info).curTargetHWND = new_target;

    (*info).accepterHWND = core::ptr::null_mut();
    while !new_target.is_null() && is_droptarget(new_target) == 0 {
        if is_acceptfiles(new_target) != 0 {
            dwEffect = (*info).dwOKEffect;
            let hr = DefaultDragEnter(
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
        let hr = IDropTarget_DragEnter(
            (*info).curDragTarget,
            (*info).dataObject,
            (*info).dwKeyState,
            (*info).curMousePos,
            (*info).pdwEffect,
        );
        *(*info).pdwEffect &= (*info).dwOKEffect;

        // failed DragEnter() means invalid target
        if hr != S_OK {
            IDropTarget_Release((*info).curDragTarget);
            (*info).curDragTarget = core::ptr::null_mut();
            (*info).curTargetHWND = core::ptr::null_mut();
            (*info).accepterHWND = core::ptr::null_mut();
        }
    }
}

/// ole2.c:2319 drag_end
unsafe fn drag_end(info: *mut TrackerWindowInfo) {
    (*info).trackingDone = 1;
    ReleaseCapture();

    if !(*info).curDragTarget.is_null() {
        if (*info).returnValue == DRAGDROP_S_DROP && *(*info).pdwEffect != DROPEFFECT_NONE {
            *(*info).pdwEffect = (*info).dwOKEffect;
            let hr = IDropTarget_Drop(
                (*info).curDragTarget,
                (*info).dataObject,
                (*info).dwKeyState,
                (*info).curMousePos,
                (*info).pdwEffect,
            );
            *(*info).pdwEffect &= (*info).dwOKEffect;

            if FAILED(hr) {
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
            let hr = DefaultDrop(
                (*info).accepterHWND,
                (*info).dataObject,
                (*info).dwKeyState,
                (*info).curMousePos,
                (*info).pdwEffect,
            );
            *(*info).pdwEffect &= (*info).dwOKEffect;

            if FAILED(hr) {
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

/// ole2.c:2372 give_feedback
unsafe fn give_feedback(info: *mut TrackerWindowInfo) -> HRESULT {
    if (*info).curDragTarget.is_null() && (*info).accepterHWND.is_null() {
        *(*info).pdwEffect = DROPEFFECT_NONE;
    }

    let hr = IDropSource_GiveFeedback((*info).dropSource, *(*info).pdwEffect);

    if hr == DRAGDROP_S_USEDEFAULTCURSORS {
        let res: c_int = if *(*info).pdwEffect & DROPEFFECT_MOVE != 0 {
            CURSOR_MOVE
        } else if *(*info).pdwEffect & DROPEFFECT_COPY != 0 {
            CURSOR_COPY
        } else if *(*info).pdwEffect & DROPEFFECT_LINK != 0 {
            CURSOR_LINK
        } else {
            CURSOR_NODROP
        };

        let cur = LoadCursorW(
            crate::compobj::compobj::hProxyDll as HINSTANCE,
            MAKEINTRESOURCEW(res as WORD),
        );
        SetCursor(cur);
    }

    hr
}

/// ole2.c:2416 OLEDD_TrackStateChange
unsafe fn OLEDD_TrackStateChange(trackerInfo: *mut TrackerWindowInfo) {
    let mut pt: POINT = core::mem::zeroed();

    // QueryContinueDrag may run a message loop and land back here (wine-10);
    // don't recurse into a second drag-state update mid-flight.
    if (*trackerInfo).inTrackCall != 0 {
        return;
    }
    (*trackerInfo).inTrackCall = 1;

    // Get the handle of the window under the mouse
    pt.x = (*trackerInfo).curMousePos.x;
    pt.y = (*trackerInfo).curMousePos.y;
    let hwndNewTarget: HWND = WindowFromPoint(pt);

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

    (*trackerInfo).inTrackCall = 0;
}

/// ole2.c:2484 OLEDD_GetButtonState
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

/// ole2.c:2526 OLEUTL_ReadRegistryDWORDValue
unsafe fn OLEUTL_ReadRegistryDWORDValue(regKey: HKEY, pdwValue: *mut DWORD) {
    let mut buffer: [WCHAR; 20] = [0; 20];
    let mut cbData: DWORD = core::mem::size_of_val(&buffer) as DWORD;
    let mut dwKeyType: DWORD = 0;

    let lres = RegQueryValueExW(
        regKey,
        emptyW.as_ptr(),
        core::ptr::null_mut(),
        &mut dwKeyType,
        buffer.as_mut_ptr() as *mut u8,
        &mut cbData,
    );

    if lres == ERROR_SUCCESS {
        match dwKeyType {
            REG_DWORD => {
                *pdwValue = core::ptr::read_unaligned(buffer.as_ptr() as *const DWORD);
            }
            REG_EXPAND_SZ | REG_MULTI_SZ | REG_SZ => {
                *pdwValue = wcstoul(buffer.as_ptr(), core::ptr::null_mut(), 10) as DWORD;
            }
            _ => {}
        }
    }
}

/// ole2.c:2565 OleDraw [OLE32.@]
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
        &mut viewobject as *mut *mut IViewObject as *mut *mut c_void,
    );
    if SUCCEEDED(hres) {
        let hres = IViewObject_Draw(
            viewobject,
            dwAspect,
            -1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hdcDraw,
            rect as LPCRECTL,
            core::ptr::null(),
            None,
            0,
        );
        IViewObject_Release(viewobject);
        hres
    } else {
        DV_E_NOIVIEWOBJECT
    }
}

/// ole2.c:2592 OleTranslateAccelerator [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleTranslateAccelerator(
    lpFrame: LPOLEINPLACEFRAME,
    lpFrameInfo: LPOLEINPLACEFRAMEINFO,
    lpmsg: LPMSG,
) -> HRESULT {
    let mut wID: WORD = 0;

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), lpFrame, lpFrameInfo, lpmsg);

    if IsAccelerator(
        (*lpFrameInfo).haccel,
        (*lpFrameInfo).cAccelEntries as c_int,
        lpmsg,
        &mut wID,
    ) != 0
    {
        return IOleInPlaceFrame_TranslateAccelerator(lpFrame, lpmsg, wID);
    }

    S_FALSE
}

/// ole2.c:2609 OleCreate [OLE32.@]
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
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let mut pOleObject: *mut IOleObject = core::ptr::null_mut();

    TRACE!(
        b"(%s, %s, %d, %p, %p, %p, %p)\n\0".as_ptr(),
        debugstr_guid(rclsid as *const c_void),
        debugstr_guid(riid as *const c_void),
        renderopt,
        pFormatEtc,
        pClientSite,
        pStg,
        ppvObj,
    );

    let mut hres = CoCreateInstance(
        (rclsid).cast(),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER,
        (riid).cast(),
        &mut pUnk as *mut *mut IUnknown as *mut LPVOID,
    );

    if SUCCEEDED(hres) {
        hres = IStorage_SetClass(pStg, rclsid);
    }

    if !pClientSite.is_null() && SUCCEEDED(hres) {
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IOleObject,
            &mut pOleObject as *mut *mut IOleObject as *mut *mut c_void,
        );
        if SUCCEEDED(hres) {
            let mut dwStatus: DWORD = 0;
            IOleObject_GetMiscStatus(pOleObject, DVASPECT_CONTENT, &mut dwStatus);
        }
    }

    if SUCCEEDED(hres) {
        let mut pPS: *mut IPersistStorage = core::ptr::null_mut();
        hres = IUnknown_QueryInterface(
            pUnk,
            &IID_IPersistStorage,
            &mut pPS as *mut *mut IPersistStorage as *mut *mut c_void,
        );
        if SUCCEEDED(hres) {
            TRACE!(b"trying to set stg %p\n\0".as_ptr(), pStg);
            hres = IPersistStorage_InitNew(pPS, pStg);
            TRACE!(b"-- result 0x%08x\n\0".as_ptr(), hres);
            IPersistStorage_Release(pPS);
        }
    }

    if !pClientSite.is_null() && SUCCEEDED(hres) {
        TRACE!(b"trying to set clientsite %p\n\0".as_ptr(), pClientSite);
        hres = IOleObject_SetClientSite(pOleObject, pClientSite);
        TRACE!(b"-- result 0x%08x\n\0".as_ptr(), hres);
    }

    if !pOleObject.is_null() {
        IOleObject_Release(pOleObject);
    }

    if ((renderopt == OLERENDER_DRAW) || (renderopt == OLERENDER_FORMAT)) && SUCCEEDED(hres) {
        hres = OleRun(pUnk);
        if SUCCEEDED(hres) {
            let mut pOleCache: *mut IOleCache = core::ptr::null_mut();

            if SUCCEEDED(IUnknown_QueryInterface(
                pUnk,
                &IID_IOleCache,
                &mut pOleCache as *mut *mut IOleCache as *mut *mut c_void,
            )) {
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
                    hres =
                        IOleCache_Cache(pOleCache, pFormatEtc, ADVF_PRIMEFIRST, &mut dwConnection);
                }
                IOleCache_Release(pOleCache);
            }
        }
    }

    if FAILED(hres) && !pUnk.is_null() {
        IUnknown_Release(pUnk);
        pUnk = core::ptr::null_mut();
    }

    *ppvObj = pUnk as LPVOID;

    TRACE!(b"-- %p\n\0".as_ptr(), pUnk);
    hres
}

/// ole2.c:2704 OleGetAutoConvert [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleGetAutoConvert(
    clsidOld: REFCLSID,
    pClsidNew: *mut CLSID,
) -> HRESULT {
    static wszAutoConvertTo: [WCHAR; 14] = [
        'A' as u16, 'u' as u16, 't' as u16, 'o' as u16, 'C' as u16, 'o' as u16, 'n' as u16,
        'v' as u16, 'e' as u16, 'r' as u16, 't' as u16, 'T' as u16, 'o' as u16, 0,
    ];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut buf: [WCHAR; CHARS_IN_GUID as usize] = [0; CHARS_IN_GUID as usize];
    let mut res: HRESULT;

    res = COM_OpenKeyForCLSID(clsidOld, wszAutoConvertTo.as_ptr(), KEY_READ, &mut hkey);
    if FAILED(res) {
        // done:
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }

    let mut len: LONG = core::mem::size_of_val(&buf) as LONG;
    if RegQueryValueW(hkey, core::ptr::null(), buf.as_mut_ptr(), &mut len) != 0 {
        res = REGDB_E_KEYMISSING;
        // done:
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }
    res = CLSIDFromString(buf.as_ptr(), pClsidNew);
    // done:
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
    res
}

/// ole2.c:2731 OleSetAutoConvert [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleSetAutoConvert(
    clsidOld: REFCLSID,
    clsidNew: REFCLSID,
) -> HRESULT {
    static wszAutoConvertTo: [WCHAR; 14] = [
        'A' as u16, 'u' as u16, 't' as u16, 'o' as u16, 'C' as u16, 'o' as u16, 'n' as u16,
        'v' as u16, 'e' as u16, 'r' as u16, 't' as u16, 'T' as u16, 'o' as u16, 0,
    ];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut szClsidNew: [WCHAR; CHARS_IN_GUID as usize] = [0; CHARS_IN_GUID as usize];
    let mut res: HRESULT;

    TRACE!(
        b"(%s,%s)\n\0".as_ptr(),
        debugstr_guid(clsidOld as *const c_void),
        debugstr_guid(clsidNew as *const c_void)
    );

    res = COM_OpenKeyForCLSID(
        clsidOld,
        core::ptr::null(),
        KEY_READ | KEY_WRITE,
        &mut hkey,
    );
    if FAILED(res) {
        // done:
        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
        return res;
    }
    StringFromGUID2(clsidNew, szClsidNew.as_mut_ptr(), CHARS_IN_GUID as c_int);
    if RegSetValueW(
        hkey,
        wszAutoConvertTo.as_ptr(),
        REG_SZ,
        szClsidNew.as_ptr(),
        ((lstrlenW(szClsidNew.as_ptr()) + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD,
    ) != 0
    {
        res = REGDB_E_WRITEREGDB;
    }

    // done:
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }
    res
}

/// ole2.c:2758 OleDoAutoConvert [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleDoAutoConvert(
    pStg: LPSTORAGE,
    pClsidNew: *mut CLSID,
) -> HRESULT {
    let mut user_type_old: LPOLESTR = core::ptr::null_mut();
    let mut user_type_new: LPOLESTR = core::ptr::null_mut();
    let mut cf: CLIPFORMAT = 0;
    let mut stat: STATSTG = core::mem::zeroed();
    let mut clsid: CLSID = core::mem::zeroed();

    TRACE!(b"(%p, %p)\n\0".as_ptr(), pStg, pClsidNew);

    *pClsidNew = CLSID_NULL;
    if pStg.is_null() {
        return E_INVALIDARG;
    }
    let mut hr = IStorage_Stat(pStg, &mut stat, STATFLAG_NONAME);
    if FAILED(hr) {
        return hr;
    }

    *pClsidNew = stat.clsid;
    hr = OleGetAutoConvert(&stat.clsid, &mut clsid);
    if FAILED(hr) {
        return hr;
    }

    hr = IStorage_SetClass(pStg, &clsid);
    if FAILED(hr) {
        return hr;
    }

    hr = ReadFmtUserTypeStg(pStg, &mut cf, &mut user_type_old);
    if FAILED(hr) {
        cf = 0;
        user_type_new = core::ptr::null_mut();
        let _ = user_type_new;
    }

    hr = OleRegGetUserType(&clsid, USERCLASSTYPE_FULL, &mut user_type_new);
    if FAILED(hr) {
        user_type_new = core::ptr::null_mut();
    }

    hr = WriteFmtUserTypeStg(pStg, cf, user_type_new);
    CoTaskMemFree(user_type_new as LPVOID);
    if FAILED(hr) {
        CoTaskMemFree(user_type_old as LPVOID);
        IStorage_SetClass(pStg, &stat.clsid);
        return hr;
    }

    hr = SetConvertStg(pStg, 1);
    if FAILED(hr) {
        WriteFmtUserTypeStg(pStg, cf, user_type_old);
        IStorage_SetClass(pStg, &stat.clsid);
    } else {
        *pClsidNew = clsid;
    }
    CoTaskMemFree(user_type_old as LPVOID);
    hr
}

/// ole2.c:2818 OleIsRunning [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleIsRunning(object: LPOLEOBJECT) -> BOOL {
    let mut pRunnable: *mut IRunnableObject = core::ptr::null_mut();

    TRACE!(b"(%p)\n\0".as_ptr(), object);

    if object.is_null() {
        return 0;
    }

    let hr = IOleObject_QueryInterface(
        object,
        &IID_IRunnableObject,
        &mut pRunnable as *mut *mut IRunnableObject as *mut *mut c_void,
    );
    if FAILED(hr) {
        return 1;
    }
    let running = IRunnableObject_IsRunning(pRunnable);
    IRunnableObject_Release(pRunnable);
    running
}

/// ole2.c:2839 OleNoteObjectVisible [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleNoteObjectVisible(
    pUnknown: LPUNKNOWN,
    bVisible: BOOL,
) -> HRESULT {
    TRACE!(
        b"(%p, %s)\n\0".as_ptr(),
        pUnknown,
        if bVisible != 0 {
            b"TRUE\0".as_ptr()
        } else {
            b"FALSE\0".as_ptr()
        }
    );
    CoLockObjectExternal(pUnknown, bVisible, 1)
}

/// ole2.c:2852 OLE_FreeClipDataArray [internal]
unsafe fn OLE_FreeClipDataArray(count: ULONG, pClipDataArray: *mut CLIPDATA) {
    let mut i: ULONG = 0;
    while i < count {
        CoTaskMemFree((*pClipDataArray.add(i as usize)).pClipData as LPVOID);
        i += 1;
    }
}

/// ole2.c:2864 PropSysAllocString [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn PropSysAllocString(str: LPCOLESTR) -> BSTR {
    SysAllocString(str)
}

/// ole2.c:2874 PropSysFreeString [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn PropSysFreeString(str: LPOLESTR) {
    SysFreeString(str);
}

/// ole2.c:2882 PROPVARIANT_ValidateType
unsafe fn PROPVARIANT_ValidateType(vt: VARTYPE) -> HRESULT {
    match vt as DWORD {
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
        v if v == VT_I1 | VT_VECTOR => S_OK,
        v if v == VT_I2 | VT_VECTOR => S_OK,
        v if v == VT_I4 | VT_VECTOR => S_OK,
        v if v == VT_I8 | VT_VECTOR => S_OK,
        v if v == VT_R4 | VT_VECTOR => S_OK,
        v if v == VT_R8 | VT_VECTOR => S_OK,
        v if v == VT_CY | VT_VECTOR => S_OK,
        v if v == VT_DATE | VT_VECTOR => S_OK,
        v if v == VT_BSTR | VT_VECTOR => S_OK,
        v if v == VT_ERROR | VT_VECTOR => S_OK,
        v if v == VT_BOOL | VT_VECTOR => S_OK,
        v if v == VT_VARIANT | VT_VECTOR => S_OK,
        v if v == VT_UI1 | VT_VECTOR => S_OK,
        v if v == VT_UI2 | VT_VECTOR => S_OK,
        v if v == VT_UI4 | VT_VECTOR => S_OK,
        v if v == VT_UI8 | VT_VECTOR => S_OK,
        v if v == VT_LPSTR | VT_VECTOR => S_OK,
        v if v == VT_LPWSTR | VT_VECTOR => S_OK,
        v if v == VT_FILETIME | VT_VECTOR => S_OK,
        v if v == VT_CF | VT_VECTOR => S_OK,
        v if v == VT_CLSID | VT_VECTOR => S_OK,
        v if v == VT_ARRAY | VT_I1 => S_OK,
        v if v == VT_ARRAY | VT_UI1 => S_OK,
        v if v == VT_ARRAY | VT_I2 => S_OK,
        v if v == VT_ARRAY | VT_UI2 => S_OK,
        v if v == VT_ARRAY | VT_I4 => S_OK,
        v if v == VT_ARRAY | VT_UI4 => S_OK,
        v if v == VT_ARRAY | VT_INT => S_OK,
        v if v == VT_ARRAY | VT_UINT => S_OK,
        v if v == VT_ARRAY | VT_R4 => S_OK,
        v if v == VT_ARRAY | VT_R8 => S_OK,
        v if v == VT_ARRAY | VT_CY => S_OK,
        v if v == VT_ARRAY | VT_DATE => S_OK,
        v if v == VT_ARRAY | VT_BSTR => S_OK,
        v if v == VT_ARRAY | VT_BOOL => S_OK,
        v if v == VT_ARRAY | VT_DECIMAL => S_OK,
        v if v == VT_ARRAY | VT_DISPATCH => S_OK,
        v if v == VT_ARRAY | VT_UNKNOWN => S_OK,
        v if v == VT_ARRAY | VT_ERROR => S_OK,
        v if v == VT_ARRAY | VT_VARIANT => S_OK,
        _ => {
            WARN!(b"Bad type %d\n\0".as_ptr(), vt);
            STG_E_INVALIDPARAMETER
        }
    }
}

/// ole2.c:2968 PropVariantClear [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn PropVariantClear(pvar: *mut PROPVARIANT) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), pvar);

    if pvar.is_null() {
        return S_OK;
    }

    let mut hr = PROPVARIANT_ValidateType((*pvar).vt);
    if FAILED(hr) {
        memset(pvar as *mut c_void, 0, core::mem::size_of::<PROPVARIANT>());
        return hr;
    }

    let vt = (*pvar).vt as DWORD;
    match vt {
        VT_EMPTY | VT_NULL | VT_I1 | VT_I2 | VT_I4 | VT_I8 | VT_R4 | VT_R8 | VT_CY | VT_DATE
        | VT_ERROR | VT_BOOL | VT_DECIMAL | VT_UI1 | VT_UI2 | VT_UI4 | VT_UI8 | VT_INT
        | VT_UINT | VT_FILETIME => {}
        VT_DISPATCH | VT_UNKNOWN | VT_STREAM | VT_STREAMED_OBJECT | VT_STORAGE
        | VT_STORED_OBJECT => {
            if !(*pvar).__bindgen_anon_1.pStream.is_null() {
                IStream_Release((*pvar).__bindgen_anon_1.pStream);
            }
        }
        VT_CLSID | VT_LPSTR | VT_LPWSTR => {
            // pick an arbitrary typed pointer — we don't care about the type as we
            // are just freeing it
            CoTaskMemFree((*pvar).__bindgen_anon_1.puuid as LPVOID);
        }
        VT_BLOB | VT_BLOB_OBJECT => {
            CoTaskMemFree((*pvar).__bindgen_anon_1.blob.pBlobData as LPVOID);
        }
        VT_BSTR => {
            PropSysFreeString((*pvar).__bindgen_anon_1.bstrVal);
        }
        VT_CF => {
            if !(*pvar).__bindgen_anon_1.pclipdata.is_null() {
                OLE_FreeClipDataArray(1, (*pvar).__bindgen_anon_1.pclipdata);
                CoTaskMemFree((*pvar).__bindgen_anon_1.pclipdata as LPVOID);
            }
        }
        _ => {
            if (vt & VT_VECTOR) != 0 {
                match vt & !VT_VECTOR {
                    VT_VARIANT => {
                        FreePropVariantArray(
                            (*pvar).__bindgen_anon_1.capropvar.cElems,
                            (*pvar).__bindgen_anon_1.capropvar.pElems,
                        );
                    }
                    VT_CF => {
                        OLE_FreeClipDataArray(
                            (*pvar).__bindgen_anon_1.caclipdata.cElems,
                            (*pvar).__bindgen_anon_1.caclipdata.pElems,
                        );
                    }
                    VT_BSTR => {
                        for i in 0..(*pvar).__bindgen_anon_1.cabstr.cElems {
                            PropSysFreeString(
                                *(*pvar).__bindgen_anon_1.cabstr.pElems.add(i as usize),
                            );
                        }
                    }
                    VT_LPSTR => {
                        for i in 0..(*pvar).__bindgen_anon_1.calpstr.cElems {
                            CoTaskMemFree(
                                *(*pvar).__bindgen_anon_1.calpstr.pElems.add(i as usize) as LPVOID,
                            );
                        }
                    }
                    VT_LPWSTR => {
                        for i in 0..(*pvar).__bindgen_anon_1.calpwstr.cElems {
                            CoTaskMemFree(
                                *(*pvar).__bindgen_anon_1.calpwstr.pElems.add(i as usize) as LPVOID,
                            );
                        }
                    }
                    _ => {}
                }
                if (vt & !VT_VECTOR) != 0 {
                    // pick an arbitrary VT_VECTOR structure — they all have the same
                    // memory layout
                    CoTaskMemFree((*pvar).__bindgen_anon_1.capropvar.pElems as LPVOID);
                }
            } else if (vt & VT_ARRAY) != 0 {
                hr = SafeArrayDestroy((*pvar).__bindgen_anon_1.parray);
            } else {
                WARN!(b"Invalid/unsupported type %d\n\0".as_ptr(), (*pvar).vt);
                hr = STG_E_INVALIDPARAMETER;
            }
        }
    }

    memset(pvar as *mut c_void, 0, core::mem::size_of::<PROPVARIANT>());
    hr
}

/// ole2.c:3086 PropVariantCopy [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn PropVariantCopy(
    pvarDest: *mut PROPVARIANT,
    pvarSrc: *const PROPVARIANT,
) -> HRESULT {
    let len: ULONG;

    TRACE!(
        b"(%p, %p vt %04x)\n\0".as_ptr(),
        pvarDest,
        pvarSrc,
        (*pvarSrc).vt
    );

    let hr = PROPVARIANT_ValidateType((*pvarSrc).vt);
    if FAILED(hr) {
        return DISP_E_BADVARTYPE;
    }

    // this will deal with most cases
    *pvarDest = *pvarSrc;

    let vt = (*pvarSrc).vt as DWORD;
    match vt {
        VT_EMPTY | VT_NULL | VT_I1 | VT_UI1 | VT_I2 | VT_UI2 | VT_BOOL | VT_DECIMAL | VT_I4
        | VT_UI4 | VT_R4 | VT_ERROR | VT_I8 | VT_UI8 | VT_INT | VT_UINT | VT_R8 | VT_CY
        | VT_DATE | VT_FILETIME => {}
        VT_DISPATCH | VT_UNKNOWN | VT_STREAM | VT_STREAMED_OBJECT | VT_STORAGE
        | VT_STORED_OBJECT => {
            if !(*pvarDest).__bindgen_anon_1.pStream.is_null() {
                IStream_AddRef((*pvarDest).__bindgen_anon_1.pStream);
            }
        }
        VT_CLSID => {
            (*pvarDest).__bindgen_anon_1.puuid =
                CoTaskMemAlloc(core::mem::size_of::<CLSID>() as SIZE_T) as *mut CLSID;
            *(*pvarDest).__bindgen_anon_1.puuid = *(*pvarSrc).__bindgen_anon_1.puuid;
        }
        VT_LPSTR => {
            if !(*pvarSrc).__bindgen_anon_1.pszVal.is_null() {
                len = strlen((*pvarSrc).__bindgen_anon_1.pszVal) as ULONG;
                (*pvarDest).__bindgen_anon_1.pszVal =
                    CoTaskMemAlloc((len as SIZE_T + 1) * core::mem::size_of::<CHAR>() as SIZE_T)
                        as LPSTR;
                memcpy(
                    (*pvarDest).__bindgen_anon_1.pszVal as *mut c_void,
                    (*pvarSrc).__bindgen_anon_1.pszVal as *const c_void,
                    (len as usize + 1) * core::mem::size_of::<CHAR>(),
                );
            }
        }
        VT_LPWSTR => {
            if !(*pvarSrc).__bindgen_anon_1.pwszVal.is_null() {
                len = lstrlenW((*pvarSrc).__bindgen_anon_1.pwszVal) as ULONG;
                (*pvarDest).__bindgen_anon_1.pwszVal =
                    CoTaskMemAlloc((len as SIZE_T + 1) * core::mem::size_of::<WCHAR>() as SIZE_T)
                        as LPWSTR;
                memcpy(
                    (*pvarDest).__bindgen_anon_1.pwszVal as *mut c_void,
                    (*pvarSrc).__bindgen_anon_1.pwszVal as *const c_void,
                    (len as usize + 1) * core::mem::size_of::<WCHAR>(),
                );
            }
        }
        VT_BLOB | VT_BLOB_OBJECT => {
            if !(*pvarSrc).__bindgen_anon_1.blob.pBlobData.is_null() {
                len = (*pvarSrc).__bindgen_anon_1.blob.cbSize;
                (*pvarDest).__bindgen_anon_1.blob.pBlobData =
                    CoTaskMemAlloc(len as SIZE_T) as *mut BYTE;
                memcpy(
                    (*pvarDest).__bindgen_anon_1.blob.pBlobData as *mut c_void,
                    (*pvarSrc).__bindgen_anon_1.blob.pBlobData as *const c_void,
                    len as usize,
                );
            }
        }
        VT_BSTR => {
            (*pvarDest).__bindgen_anon_1.bstrVal =
                PropSysAllocString((*pvarSrc).__bindgen_anon_1.bstrVal);
        }
        VT_CF => {
            if !(*pvarSrc).__bindgen_anon_1.pclipdata.is_null() {
                len = (*(*pvarSrc).__bindgen_anon_1.pclipdata).cbSize
                    - core::mem::size_of::<ULONG>() as ULONG;
                (*pvarDest).__bindgen_anon_1.pclipdata =
                    CoTaskMemAlloc(core::mem::size_of::<CLIPDATA>() as SIZE_T) as *mut CLIPDATA;
                (*(*pvarDest).__bindgen_anon_1.pclipdata).cbSize =
                    (*(*pvarSrc).__bindgen_anon_1.pclipdata).cbSize;
                (*(*pvarDest).__bindgen_anon_1.pclipdata).ulClipFmt =
                    (*(*pvarSrc).__bindgen_anon_1.pclipdata).ulClipFmt;
                (*(*pvarDest).__bindgen_anon_1.pclipdata).pClipData =
                    CoTaskMemAlloc(len as SIZE_T) as *mut BYTE;
                memcpy(
                    (*(*pvarDest).__bindgen_anon_1.pclipdata).pClipData as *mut c_void,
                    (*(*pvarSrc).__bindgen_anon_1.pclipdata).pClipData as *const c_void,
                    len as usize,
                );
            }
        }
        _ => {
            if (vt & VT_VECTOR) != 0 {
                let elemSize: c_int;

                match vt & !VT_VECTOR {
                    VT_I1 => elemSize = core::mem::size_of::<CHAR>() as c_int,
                    VT_UI1 => elemSize = core::mem::size_of::<UCHAR>() as c_int,
                    VT_I2 => elemSize = core::mem::size_of::<i16>() as c_int,
                    VT_UI2 => elemSize = core::mem::size_of::<USHORT>() as c_int,
                    VT_BOOL => elemSize = core::mem::size_of::<VARIANT_BOOL>() as c_int,
                    VT_I4 => elemSize = core::mem::size_of::<LONG>() as c_int,
                    VT_UI4 => elemSize = core::mem::size_of::<ULONG>() as c_int,
                    VT_R4 => elemSize = core::mem::size_of::<f32>() as c_int,
                    VT_R8 => elemSize = core::mem::size_of::<f64>() as c_int,
                    VT_ERROR => elemSize = core::mem::size_of::<SCODE>() as c_int,
                    VT_I8 => elemSize = core::mem::size_of::<LARGE_INTEGER>() as c_int,
                    VT_UI8 => elemSize = core::mem::size_of::<ULARGE_INTEGER>() as c_int,
                    VT_CY => elemSize = core::mem::size_of::<CY>() as c_int,
                    VT_DATE => elemSize = core::mem::size_of::<DATE>() as c_int,
                    VT_FILETIME => elemSize = core::mem::size_of::<FILETIME>() as c_int,
                    VT_CLSID => elemSize = core::mem::size_of::<CLSID>() as c_int,
                    VT_CF => elemSize = core::mem::size_of::<CLIPDATA>() as c_int,
                    VT_BSTR => elemSize = core::mem::size_of::<BSTR>() as c_int,
                    VT_LPSTR => elemSize = core::mem::size_of::<LPSTR>() as c_int,
                    VT_LPWSTR => elemSize = core::mem::size_of::<LPWSTR>() as c_int,
                    VT_VARIANT => elemSize = core::mem::size_of::<PROPVARIANT>() as c_int,

                    _ => {
                        FIXME!(b"Invalid element type: %ul\n\0".as_ptr(), vt & !VT_VECTOR);
                        return E_INVALIDARG;
                    }
                }
                len = (*pvarSrc).__bindgen_anon_1.capropvar.cElems;
                (*pvarDest).__bindgen_anon_1.capropvar.pElems = if len != 0 {
                    CoTaskMemAlloc(len as SIZE_T * elemSize as SIZE_T) as *mut PROPVARIANT
                } else {
                    core::ptr::null_mut()
                };
                if vt == (VT_VECTOR | VT_VARIANT) {
                    for i in 0..len {
                        PropVariantCopy(
                            (*pvarDest).__bindgen_anon_1.capropvar.pElems.add(i as usize),
                            (*pvarSrc).__bindgen_anon_1.capropvar.pElems.add(i as usize),
                        );
                    }
                } else if vt == (VT_VECTOR | VT_CF) {
                    FIXME!(b"Copy clipformats\n\0".as_ptr());
                } else if vt == (VT_VECTOR | VT_BSTR) {
                    for i in 0..len {
                        *(*pvarDest).__bindgen_anon_1.cabstr.pElems.add(i as usize) =
                            PropSysAllocString(
                                *(*pvarSrc).__bindgen_anon_1.cabstr.pElems.add(i as usize),
                            );
                    }
                } else if vt == (VT_VECTOR | VT_LPSTR) {
                    for i in 0..len {
                        let strLen = (lstrlenA(
                            *(*pvarSrc).__bindgen_anon_1.calpstr.pElems.add(i as usize),
                        ) + 1) as usize;
                        *(*pvarDest).__bindgen_anon_1.calpstr.pElems.add(i as usize) =
                            CoTaskMemAlloc(strLen as SIZE_T) as LPSTR;
                        memcpy(
                            *(*pvarDest).__bindgen_anon_1.calpstr.pElems.add(i as usize)
                                as *mut c_void,
                            *(*pvarSrc).__bindgen_anon_1.calpstr.pElems.add(i as usize)
                                as *const c_void,
                            strLen,
                        );
                    }
                } else if vt == (VT_VECTOR | VT_LPWSTR) {
                    for i in 0..len {
                        // NOTE (faithful to the C): wine copies through calpstr here,
                        // not calpwstr. Same union slot, so it works out.
                        let strLen = ((lstrlenW(
                            *(*pvarSrc).__bindgen_anon_1.calpwstr.pElems.add(i as usize),
                        ) + 1) as usize)
                            * core::mem::size_of::<WCHAR>();
                        *(*pvarDest).__bindgen_anon_1.calpstr.pElems.add(i as usize) =
                            CoTaskMemAlloc(strLen as SIZE_T) as LPSTR;
                        memcpy(
                            *(*pvarDest).__bindgen_anon_1.calpstr.pElems.add(i as usize)
                                as *mut c_void,
                            *(*pvarSrc).__bindgen_anon_1.calpstr.pElems.add(i as usize)
                                as *const c_void,
                            strLen,
                        );
                    }
                } else {
                    memcpy(
                        (*pvarDest).__bindgen_anon_1.capropvar.pElems as *mut c_void,
                        (*pvarSrc).__bindgen_anon_1.capropvar.pElems as *const c_void,
                        len as usize * elemSize as usize,
                    );
                }
            } else if (vt & VT_ARRAY) != 0 {
                (*pvarDest).__bindgen_anon_1.uhVal.QuadPart = 0;
                return SafeArrayCopy(
                    (*pvarSrc).__bindgen_anon_1.parray,
                    core::ptr::addr_of_mut!((*pvarDest).__bindgen_anon_1.parray),
                );
            } else {
                WARN!(b"Invalid/unsupported type %d\n\0".as_ptr(), (*pvarSrc).vt);
            }
        }
    }

    S_OK
}

/// ole2.c:3267 FreePropVariantArray [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn FreePropVariantArray(
    cVariants: ULONG,
    rgvars: *mut PROPVARIANT,
) -> HRESULT {
    TRACE!(b"(%u, %p)\n\0".as_ptr(), cVariants, rgvars);

    if rgvars.is_null() {
        return E_INVALIDARG;
    }

    for i in 0..cVariants {
        PropVariantClear(rgvars.add(i as usize));
    }

    S_OK
}

/// ole2.c:3288 DllDebugObjectRPCHook [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn DllDebugObjectRPCHook(_b: BOOL, _dummy: *mut c_void) -> BOOL {
    FIXME!(b"stub\n\0".as_ptr());
    1
}
