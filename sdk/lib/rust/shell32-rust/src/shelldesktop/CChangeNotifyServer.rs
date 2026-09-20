use crate::prelude::*;
use crate::shelldesktop::CChangeNotifyServer_window::{
    CChangeNotifyServer_SubclassWindow, SHCreateDefaultWorkerWindow,
};
use crate::atlwin::AtlWindowState;
use crate::shelldesktop::CDirectoryWatcher_bridge::{
    CDirectoryWatcher, CDirectoryWatcher_Create_RustBridge,
    CDirectoryWatcher_RequestAddWatcher_RustBridge,
    CDirectoryWatcher_RequestAllWatchersTermination_RustBridge,
    CDirectoryWatcher_RequestTermination_RustBridge, CDirectoryWatcher_delete_RustBridge,
};
use crate::wine::pidl::ILIsParent;
// Deterministic from oleidl.h
#[repr(C)]
pub struct IOleWindow {
    pub lpVtbl: *const IOleWindowVtbl,
}
#[repr(C)]
pub struct IOleWindowVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOleWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IOleWindow: IID;
}

#[repr(C)]
pub struct CWatchItem {
    pub nRegID: UINT,
    pub dwUserPID: DWORD,
    pub pRegEntry: LPREGENTRY,
    pub hwndBroker: HWND,
    pub pDirWatch: *mut CDirectoryWatcher,
}

// Deterministic COM scaffold for CChangeNotifyServer; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CChangeNotifyServer {
    pub IOleWindow_iface: IOleWindow,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWnd: HWND,
    pub m_atl: AtlWindowState,
    pub m_nNextRegID: UINT,
    pub m_items: CSimpleArray<*mut CWatchItem>,
}

unsafe extern "system" fn CChangeNotifyServer_IOleWindow_QueryInterface(This: *mut IOleWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CChangeNotifyServer, IOleWindow_iface)) as *mut CChangeNotifyServer;
    CChangeNotifyServer_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CChangeNotifyServer_IOleWindow_AddRef(This: *mut IOleWindow) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CChangeNotifyServer, IOleWindow_iface)) as *mut CChangeNotifyServer;
    CChangeNotifyServer_AddRef(this)
}
unsafe extern "system" fn CChangeNotifyServer_IOleWindow_Release(This: *mut IOleWindow) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CChangeNotifyServer, IOleWindow_iface)) as *mut CChangeNotifyServer;
    CChangeNotifyServer_Release(this)
}
unsafe extern "system" fn CChangeNotifyServer_IOleWindow_GetWindow(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CChangeNotifyServer, IOleWindow_iface)) as *mut CChangeNotifyServer;
    CChangeNotifyServer_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CChangeNotifyServer_IOleWindow_ContextSensitiveHelp(This: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CChangeNotifyServer, IOleWindow_iface)) as *mut CChangeNotifyServer;
    CChangeNotifyServer_ContextSensitiveHelp_body(this, fEnterMode)
}

pub static CCHANGENOTIFYSERVER_IOLEWINDOW_VTBL: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CChangeNotifyServer_IOleWindow_QueryInterface,
    AddRef: CChangeNotifyServer_IOleWindow_AddRef,
    Release: CChangeNotifyServer_IOleWindow_Release,
    GetWindow: CChangeNotifyServer_IOleWindow_GetWindow,
    ContextSensitiveHelp: CChangeNotifyServer_IOleWindow_ContextSensitiveHelp,
};

unsafe fn CChangeNotifyServer_QueryInterface(
    this: *mut CChangeNotifyServer, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleWindow_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CChangeNotifyServer_AddRef(this);
    S_OK
}

unsafe fn CChangeNotifyServer_AddRef(this: *mut CChangeNotifyServer) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CChangeNotifyServer_Release(this: *mut CChangeNotifyServer) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CChangeNotifyServer_dtor_body(this);
        core::ptr::drop_in_place(&mut (*this).m_items);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Internal activation: this class has no CLSID; the DLL constructs it directly through
// shellutils.h ShellObjectCreator<T> or ATL's own _CreatorClass::CreateInstance, both of
// which take (outer, riid, ppv) — the signature of this factory.
pub unsafe fn CChangeNotifyServer_CreateInstance(
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CChangeNotifyServer>()) as *mut CChangeNotifyServer;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CChangeNotifyServer {
        IOleWindow_iface: IOleWindow { lpVtbl: core::ptr::addr_of!(CCHANGENOTIFYSERVER_IOLEWINDOW_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWnd: core::mem::zeroed(),
        m_atl: core::mem::zeroed(),
        m_nNextRegID: core::mem::zeroed(),
        m_items: CSimpleArray::new(),
    });
    Shell32RustObjectLock();
    CChangeNotifyServer_ctor_body(this);
    let mut hr = CChangeNotifyServer_Initialize_body(this);
    if hr >= 0 {
        hr = CChangeNotifyServer_QueryInterface(this, riid, ppv);
    }
    CChangeNotifyServer_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const SHCNE_GLOBALEVENTS: DWORD = 0x0C0581E0;

pub const SHCNE_INTERRUPT: DWORD = 0x80000000;

pub const SHCNRF_InterruptLevel: DWORD = 0x0001;

pub const SHCNRF_ShellLevel: DWORD = 0x0002;

pub const WM_DESTROY: DWORD = 2;

extern "system" {
    fn DestroyWindow(window: HWND) -> BOOL;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn GetLastError() -> DWORD;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

extern "C" {
    fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
}
#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
}

use core::ffi::c_void;

pub unsafe fn CChangeNotifyServer_ctor_body(this: *mut CChangeNotifyServer) -> () {
    (*this).m_nNextRegID = INVALID_REG_ID;
}

pub unsafe fn CChangeNotifyServer_dtor_body(this: *mut CChangeNotifyServer) -> () {}

pub unsafe fn CChangeNotifyServer_AddItem_body(this: *mut CChangeNotifyServer, pItem: *mut CWatchItem) -> BOOL {
    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        if *((*this).m_items.GetAt(i)) == core::ptr::null_mut() {
            *((*this).m_items.GetAt(i)) = pItem;
            return TRUE;
        }
        i += 1;
    }

    (*this).m_items.Add(pItem);
    return TRUE;
}

pub unsafe fn CChangeNotifyServer_DestroyItem_body(this: *mut CChangeNotifyServer, pItem: *mut CWatchItem, phwndBroker: *mut HWND) -> () {
    debug_assert!(!pItem.is_null());

    let hwndBroker = (*pItem).hwndBroker;
    (*pItem).hwndBroker = core::ptr::null_mut();
    if !hwndBroker.is_null() && hwndBroker != *phwndBroker {
        DestroyWindow(hwndBroker);
        *phwndBroker = hwndBroker;
    }

    let pDirWatch = (*pItem).pDirWatch;
    (*pItem).pDirWatch = core::ptr::null_mut();
    if !pDirWatch.is_null() {
        CDirectoryWatcher_RequestTermination_RustBridge(pDirWatch);
    }

    SHFree((*pItem).pRegEntry);
    // CChangeNotifyServer.cpp:137 `delete pItem`, matching the malloc in OnRegister.
    free(pItem as *mut core::ffi::c_void);
}

pub unsafe fn CChangeNotifyServer_DestroyAllItems_body(this: *mut CChangeNotifyServer) -> () {
    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        let pItem = *((*this).m_items.GetAt(i));
        if !pItem.is_null() {
            let mut hwndBroker: HWND = core::ptr::null_mut();
            CChangeNotifyServer_DestroyItem_body(this, pItem, &mut hwndBroker);
            *((*this).m_items.GetAt(i)) = core::ptr::null_mut();
        }
        i += 1;
    }
    (*this).m_items.RemoveAll();
}

pub unsafe fn CChangeNotifyServer_RemoveItemsByRegID_body(this: *mut CChangeNotifyServer, nRegID: UINT) -> BOOL {
    let mut bFound: BOOL = FALSE;
    let mut hwndBroker: HWND = core::ptr::null_mut();
    assert!(nRegID != INVALID_REG_ID);
    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        let pItem: *mut CWatchItem = *(*this).m_items.GetAt(i);
        if !pItem.is_null() && (*pItem).nRegID == nRegID {
            bFound = TRUE;
            CChangeNotifyServer_DestroyItem_body(this, pItem, &mut hwndBroker);
            *(*this).m_items.GetAt(i) = core::ptr::null_mut();
        }
        i += 1;
    }
    bFound
}

pub unsafe fn CChangeNotifyServer_RemoveItemsByProcess_body(this: *mut CChangeNotifyServer, dwUserPID: DWORD) -> BOOL {
    let mut bFound: BOOL = FALSE;
    let mut hwndBroker: HWND = core::ptr::null_mut();
    assert!(dwUserPID != 0);
    let mut i: INT = 0;
    while i < ((*this).m_items).GetSize()
    {
        let pItem: *mut CWatchItem = *(((*this).m_items).GetAt(i));
        if pItem != core::ptr::null_mut() && (*pItem).dwUserPID == dwUserPID
        {
            bFound = TRUE;
            CChangeNotifyServer_DestroyItem_body(this, pItem, &mut hwndBroker as *mut HWND);
            *(((*this).m_items).GetAt(i)) = core::ptr::null_mut();
        }
        i += 1;
    }
    return bFound;
}

pub unsafe fn CreateDirectoryWatcherFromRegEntry_body(pRegEntry: LPREGENTRY) -> *mut CDirectoryWatcher {
    unsafe {
        if (*pRegEntry).ibPidl == 0 {
            return core::ptr::null_mut();
        }

        let mut szPath = [0u16; MAX_PATH as usize];
        let pidl: LPITEMIDLIST =
            pRegEntry.cast::<u8>().add((*pRegEntry).ibPidl as usize) as LPITEMIDLIST;

        if SHGetPathFromIDListW(pidl, szPath.as_mut_ptr()) == 0
            || PathIsDirectoryW(szPath.as_mut_ptr()) == 0
        {
            return core::ptr::null_mut();
        }

        let pDirectoryWatcher: *mut CDirectoryWatcher =
            CDirectoryWatcher_Create_RustBridge(
                (*pRegEntry).hwnd,
                szPath.as_mut_ptr(),
                (*pRegEntry).fRecursive,
            );
        if pDirectoryWatcher.is_null() {
            return core::ptr::null_mut();
        }

        pDirectoryWatcher
    }
}

pub unsafe fn CChangeNotifyServer_OnRegister_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let hRegEntry = wParam as HANDLE;
    let dwOwnerPID = lParam as DWORD;
    let pRegEntry = SHLockSharedEx(hRegEntry, dwOwnerPID, TRUE) as LPREGENTRY;
    if pRegEntry.is_null() || (*pRegEntry).dwMagic != REGENTRY_MAGIC {
        SHUnlockShared(pRegEntry.cast());
        return FALSE as LRESULT;
    }

    if (*pRegEntry).nRegID == INVALID_REG_ID {
        (*pRegEntry).nRegID = CChangeNotifyServer_GetNextRegID_body(this);
    }

    let mut dwUserPID: DWORD = 0;
    GetWindowThreadProcessId((*pRegEntry).hwnd, &mut dwUserPID);

    let hwndBroker = (*pRegEntry).hwndBroker;

    let pNewEntry = SHAlloc((*pRegEntry).cbSize as SIZE_T) as LPREGENTRY;
    if pNewEntry.is_null() {
        (*pRegEntry).nRegID = INVALID_REG_ID;
        SHUnlockShared(pRegEntry.cast());
        return FALSE as LRESULT;
    }
    CopyMemory(pNewEntry, pRegEntry, (*pRegEntry).cbSize as usize);

    let mut pDirWatch: *mut CDirectoryWatcher = core::ptr::null_mut();
    if (*pRegEntry).ibPidl != 0 && ((*pRegEntry).fSources & (SHCNRF_InterruptLevel as i32)) != 0 {
        pDirWatch = CreateDirectoryWatcherFromRegEntry_body(pRegEntry);
        if !pDirWatch.is_null() && CDirectoryWatcher_RequestAddWatcher_RustBridge(pDirWatch) == 0 {
            (*pRegEntry).nRegID = INVALID_REG_ID;
            SHUnlockShared(pRegEntry.cast());
            // CChangeNotifyServer.cpp:263 `delete pDirWatch` -- matching the `new` in
            // CDirectoryWatcher.cpp:93, which is still C++. malloc/free is what makes
            // those two halves agree.
            CDirectoryWatcher_delete_RustBridge(pDirWatch);
            return FALSE as LRESULT;
        }
    }

    SHUnlockShared(pRegEntry.cast());

    // CChangeNotifyServer.cpp:272 `new CWatchItem { ... }`. In this tree `operator new` IS
    // malloc and `operator delete` IS free (sdk/lib/cpprt/.../cpprt.cpp), so malloc/free is
    // the faithful pair -- and the only one that agrees with the C++ half across a boundary,
    // since CDirectoryWatcher.cpp still `new`s the watcher this struct points at. Rust's Box
    // would free through a different allocator entirely (and shell32-rust is no_std with no
    // `alloc` crate, so it does not even build).
    //
    // The write is unconditional because the C++ is: it does not check `new` for null before
    // handing the pointer to AddItem, and neither arm of that is more correct than the other.
    let pItem: *mut CWatchItem = malloc(core::mem::size_of::<CWatchItem>()) as *mut CWatchItem;
    core::ptr::write(pItem, CWatchItem {
        nRegID: (*this).m_nNextRegID,
        dwUserPID: dwUserPID,
        pRegEntry: pNewEntry,
        hwndBroker,
        pDirWatch,
    });
    return CChangeNotifyServer_AddItem_body(this, pItem) as LRESULT;
}

pub unsafe fn CChangeNotifyServer_OnUnRegister_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let nRegID: UINT = wParam as UINT;
    if nRegID == INVALID_REG_ID {
        return FALSE as LRESULT;
    }

    return CChangeNotifyServer_RemoveItemsByRegID_body(this, nRegID) as LRESULT;
}

pub unsafe fn CChangeNotifyServer_OnDeliverNotification_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let hTicket: HANDLE = wParam as HANDLE;
    let dwOwnerPID: DWORD = lParam as DWORD;

    let ret: BOOL = CChangeNotifyServer_DeliverNotification_body(this, hTicket, dwOwnerPID);

    SHFreeShared(hTicket, dwOwnerPID);

    ret as LRESULT
}

pub unsafe fn CChangeNotifyServer_OnSuspendResume_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    // FIXME
    FALSE as LRESULT
}

pub unsafe fn CChangeNotifyServer_OnRemoveByPID_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let dwUserPID: DWORD = wParam as DWORD;
    CChangeNotifyServer_RemoveItemsByProcess_body(this, dwUserPID);
    return 0;
}

pub unsafe fn CChangeNotifyServer_OnDestroy_body(this: *mut CChangeNotifyServer, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    CChangeNotifyServer_DestroyAllItems_body(this);
    CDirectoryWatcher_RequestAllWatchersTermination_RustBridge();
    return 0;
}

pub unsafe fn CChangeNotifyServer_GetNextRegID_body(this: *mut CChangeNotifyServer) -> UINT {
    (*this).m_nNextRegID += 1;
    if (*this).m_nNextRegID == INVALID_REG_ID {
        (*this).m_nNextRegID += 1;
    }
    (*this).m_nNextRegID
}

pub unsafe fn CChangeNotifyServer_DeliverNotification_body(this: *mut CChangeNotifyServer, hTicket: HANDLE, dwOwnerPID: DWORD) -> BOOL {
    let pTicket = SHLockSharedEx(hTicket, dwOwnerPID, FALSE) as *mut DELITICKET;
    if pTicket == core::ptr::null_mut() || (*pTicket).dwMagic != DELITICKET_MAGIC {
        SHUnlockShared(pTicket.cast());
        return FALSE;
    }

    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        let pItem = *((*this).m_items.GetAt(i));
        if pItem == core::ptr::null_mut() {
            i += 1;
            continue;
        }

        let pRegEntry = (*pItem).pRegEntry;
        if pRegEntry == core::ptr::null_mut() || (*pRegEntry).dwMagic != REGENTRY_MAGIC {
            i += 1;
            continue;
        }

        let bNotify = CChangeNotifyServer_ShouldNotify_body(this, pTicket, pRegEntry);
        if bNotify != 0 {
            SendMessageW((*pRegEntry).hwnd, (*pRegEntry).uMsg, hTicket as WPARAM, dwOwnerPID as LPARAM);
        }

        i += 1;
    }

    SHUnlockShared(pTicket.cast());
    return TRUE;
}

pub unsafe fn CChangeNotifyServer_ShouldNotify_body(this: *mut CChangeNotifyServer, pTicket: LPDELITICKET, pRegEntry: LPREGENTRY) -> BOOL {
    if ((*pTicket).wEventId & (SHCNE_INTERRUPT as i32)) != 0 {
        if ((*pRegEntry).fSources & (SHCNRF_InterruptLevel as i32)) == 0 {
            return FALSE;
        }
        if (*pRegEntry).ibPidl == 0 {
            return FALSE;
        }
    } else {
        if ((*pRegEntry).fSources & (SHCNRF_ShellLevel as i32)) == 0 {
            return FALSE;
        }
    }

    if ((*pTicket).wEventId & (*pRegEntry).fEvents) == 0 {
        return FALSE;
    }

    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidl1: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidl2: LPITEMIDLIST = core::ptr::null_mut();

    if (*pRegEntry).ibPidl != 0 {
        pidl = ((pRegEntry as *mut u8).add((*pRegEntry).ibPidl as usize)) as LPITEMIDLIST;
    }
    if (*pTicket).ibOffset1 != 0 {
        pidl1 = ((pTicket as *mut u8).add((*pTicket).ibOffset1 as usize)) as LPITEMIDLIST;
    }
    if (*pTicket).ibOffset2 != 0 {
        pidl2 = ((pTicket as *mut u8).add((*pTicket).ibOffset2 as usize)) as LPITEMIDLIST;
    }

    if pidl.is_null() || ((*pTicket).wEventId & (SHCNE_GLOBALEVENTS as i32)) != 0 {
        return TRUE;
    }

    if (*pRegEntry).fRecursive != 0 {
        if ILIsParent(pidl, pidl1, FALSE) != 0
            || (!pidl2.is_null() && ILIsParent(pidl, pidl2, FALSE) != 0)
        {
            return TRUE;
        }
    } else {
        if ILIsEqual(pidl, pidl1) != 0
            || ILIsParent(pidl, pidl1, TRUE) != 0
            || (!pidl2.is_null() && ILIsParent(pidl, pidl2, TRUE) != 0)
        {
            return TRUE;
        }
    }

    return FALSE;
}

pub unsafe fn CChangeNotifyServer_GetWindow_body(this: *mut CChangeNotifyServer, phwnd: *mut HWND) -> HRESULT {
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

pub unsafe fn CChangeNotifyServer_ContextSensitiveHelp_body(this: *mut CChangeNotifyServer, fEnterMode: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CChangeNotifyServer_Initialize_body(this: *mut CChangeNotifyServer) -> HRESULT {
    let hwnd = SHCreateDefaultWorkerWindow();
    if hwnd.is_null() {
        return E_FAIL;
    }
    CChangeNotifyServer_SubclassWindow(this, hwnd);
    return S_OK;
}
