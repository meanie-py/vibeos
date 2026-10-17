// rustconv-assembly: class:CRecycleBinFolderViewCB
use crate::prelude::*;
use crate::recycle_bin::CRecycleBin_ParseRecycleBinPath_RustBridge;
use crate::wine::shlmenu::HMENU;
use crate::user_notification::MSG;
// Deterministic from shlobj.h
#[repr(C)]
pub struct IShellFolderViewCB {
    pub lpVtbl: *const IShellFolderViewCBVtbl,
}
#[repr(C)]
pub struct IShellFolderViewCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub MessageSFVCB: unsafe extern "system" fn(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewCB: IID;
}

// Deterministic COM scaffold for CRecycleBinFolderViewCB; method bodies are generated separately.
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
pub struct CRecycleBinFolderViewCB {
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pShellView: *mut IShellView,
    pub m_pRecycleBin: *mut IShellFolder2,
    pub m_nChangeNotif: ULONG,
    pub m_pidlParent: *mut ITEMIDLIST,
    pub m_pidls: [*mut ITEMIDLIST; 2],
}

unsafe extern "system" fn CRecycleBinFolderViewCB_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinFolderViewCB, IShellFolderViewCB_iface)) as *mut CRecycleBinFolderViewCB;
    CRecycleBinFolderViewCB_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinFolderViewCB_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinFolderViewCB, IShellFolderViewCB_iface)) as *mut CRecycleBinFolderViewCB;
    CRecycleBinFolderViewCB_AddRef(this)
}
unsafe extern "system" fn CRecycleBinFolderViewCB_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinFolderViewCB, IShellFolderViewCB_iface)) as *mut CRecycleBinFolderViewCB;
    CRecycleBinFolderViewCB_Release(this)
}
unsafe extern "system" fn CRecycleBinFolderViewCB_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinFolderViewCB, IShellFolderViewCB_iface)) as *mut CRecycleBinFolderViewCB;
    CRecycleBinFolderViewCB_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CRECYCLEBINFOLDERVIEWCB_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CRecycleBinFolderViewCB_IShellFolderViewCB_QueryInterface,
    AddRef: CRecycleBinFolderViewCB_IShellFolderViewCB_AddRef,
    Release: CRecycleBinFolderViewCB_IShellFolderViewCB_Release,
    MessageSFVCB: CRecycleBinFolderViewCB_IShellFolderViewCB_MessageSFVCB,
};

unsafe fn CRecycleBinFolderViewCB_QueryInterface(
    this: *mut CRecycleBinFolderViewCB, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecycleBinFolderViewCB_AddRef(this);
    S_OK
}

unsafe fn CRecycleBinFolderViewCB_AddRef(this: *mut CRecycleBinFolderViewCB) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecycleBinFolderViewCB_Release(this: *mut CRecycleBinFolderViewCB) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecycleBinFolderViewCB_dtor_body(this);
        if !(*this).m_pidlParent.is_null() { CoTaskMemFree((*this).m_pidlParent.cast()); }
        for slot in (*this).m_pidls.iter() {
            if !slot.is_null() { CoTaskMemFree(slot.cast()); }
        }
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
pub unsafe fn CRecycleBinFolderViewCB_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRecycleBinFolderViewCB>()) as *mut CRecycleBinFolderViewCB;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecycleBinFolderViewCB {
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CRECYCLEBINFOLDERVIEWCB_ISHELLFOLDERVIEWCB_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pShellView: core::mem::zeroed(),
        m_pRecycleBin: core::mem::zeroed(),
        m_nChangeNotif: core::mem::zeroed(),
        m_pidlParent: core::mem::zeroed(),
        m_pidls: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecycleBinFolderViewCB_ctor_body(this);
    let hr = CRecycleBinFolderViewCB_QueryInterface(this, riid, ppv);
    CRecycleBinFolderViewCB_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const SFVM_FSNOTIFY: DWORD = 14;

pub const SFVM_QUERYFSNOTIFY: DWORD = 25;

pub const SHCNE_ASSOCCHANGED: DWORD = 0x08000000;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_DELETE: DWORD = 0x00000004;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNE_RENAMEFOLDER: DWORD = 0x00020000;

pub const SHCNE_RENAMEITEM: DWORD = 0x00000001;

pub const SHCNE_RMDIR: DWORD = 0x00000010;

pub const SHCNE_UPDATEDIR: DWORD = 0x00001000;

pub const SHCNE_UPDATEIMAGE: DWORD = 0x00008000;

pub const SHCNE_UPDATEITEM: DWORD = 0x00002000;

pub const SHCNRF_InterruptLevel: DWORD = 0x0001;

pub const SHCNRF_NewDelivery: DWORD = 0x8000;

pub const SHCNRF_ShellLevel: DWORD = 0x0002;

pub const RECYCLEBINMAXDRIVECOUNT: INT = 26;

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

pub use crate::island::SHChangeNotifyEntry;

pub use crate::island::SHITEMID;

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellBrowser {
    pub lpVtbl: *const IShellBrowserVtbl,
}
#[repr(C)]
pub struct IShellBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT,
    pub RemoveMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB: unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB: unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB: unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT,
    pub GetControlWindow: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT,
    pub QueryActiveShellView: unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive: unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellBrowser: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellView {
    pub lpVtbl: *const IShellViewVtbl,
}
#[repr(C)]
pub struct IShellViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView: IID;
}

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn CoTaskMemFree(memory: LPVOID);
}

#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

use core::ffi::c_void;

pub unsafe fn CRecycleBinFolderViewCB_ctor_body(this: *mut CRecycleBinFolderViewCB) -> () {}

pub unsafe fn CRecycleBinFolderViewCB_dtor_body(this: *mut CRecycleBinFolderViewCB) -> () {
    if (*this).m_nChangeNotif != 0 {
        SHChangeNotifyDeregister((*this).m_nChangeNotif);
    }
}

pub unsafe fn CRecycleBinFolderViewCB_Initialize_body(this: *mut CRecycleBinFolderViewCB, pRecycleBin: *mut IShellFolder2, psv: *mut IShellView, pidlParent: LPCITEMIDLIST) -> () {
    (*this).m_pRecycleBin = pRecycleBin;
    (*this).m_pShellView = psv;
    // C++: `m_pidlParent.Attach(ILClone(pidlParent))`. `CComHeapPtr` lowers to a RAW pointer
    // here, so ATL's `Attach` — `Allocator::Free(m_pData); m_pData = lp;` (atlalloc.h) — has
    // to be written out. rustconv deliberately leaves `Attach`/`Detach` unlowered rather than
    // guessing between CoTaskMemFree and Release, so this is hand-written; dropping the free
    // leaks a PIDL on every re-Initialize, with nothing red anywhere.
    //
    // `.cast()`: the module names the PIDL both as `ITEMIDLIST` (the field, from the class
    // layout) and as `_ITEMIDLIST` (what `ILClone` returns, from the island). Same record,
    // two nominal spellings — the same bridging `CDirectoryList.rs` does for CDirectoryItem.
    if !(*this).m_pidlParent.is_null() {
        CoTaskMemFree((*this).m_pidlParent.cast());
    }
    (*this).m_pidlParent = ILClone(pidlParent).cast();
}

pub unsafe fn CRecycleBinFolderViewCB_RegisterChangeNotify_body(this: *mut CRecycleBinFolderViewCB, hwndView: HWND) -> HRESULT {
    if hwndView.is_null() {
        return E_FAIL;
    }

    let mut pidls: [*mut ITEMIDLIST; RECYCLEBINMAXDRIVECOUNT as usize] =
        [core::ptr::null_mut(); RECYCLEBINMAXDRIVECOUNT as usize];
    let mut entries: [SHChangeNotifyEntry; RECYCLEBINMAXDRIVECOUNT as usize] =
        unsafe { core::mem::zeroed() };

    let mut iEntry: INT = 0;
    let mut iDrive: INT = 0;
    while iDrive < RECYCLEBINMAXDRIVECOUNT {
        let mut szBinPath: [WCHAR; MAX_PATH as usize] =
            [0; MAX_PATH as usize];
        let hr: HRESULT = GetRecycleBinPathFromDriveNumber(iDrive as DWORD, szBinPath.as_mut_ptr());
        if hr < 0 {
            iDrive += 1;
            continue;
        }
        let pidl: *mut ITEMIDLIST = ILCreateFromPathW(szBinPath.as_ptr()).cast();
        if pidl.is_null() {
            iDrive += 1;
            continue;
        }
        entries[iEntry as usize].pidl = pidl.cast();
        pidls[iEntry as usize] = pidl;
        entries[iEntry as usize].fRecursive = FALSE;
        iEntry += 1;
        iDrive += 1;
    }

    let dwFlags: DWORD = SHCNRF_InterruptLevel | SHCNRF_ShellLevel | SHCNRF_NewDelivery;
    let dwEvents: DWORD =
        SHCNE_CREATE |
        SHCNE_DELETE |
        SHCNE_RENAMEITEM |
        SHCNE_UPDATEITEM |
        SHCNE_MKDIR |
        SHCNE_RMDIR |
        SHCNE_RENAMEFOLDER |
        SHCNE_UPDATEDIR |
        SHCNE_UPDATEIMAGE |
        SHCNE_ASSOCCHANGED;
    (*this).m_nChangeNotif = SHChangeNotifyRegister(
        hwndView,
        dwFlags as INT,
        dwEvents as INT,
        SHV_CHANGE_NOTIFY,
        iEntry,
        // SHChangeNotifyRegister takes a MUTABLE entry array.
        entries.as_mut_ptr().cast(),
    );

    if (*this).m_nChangeNotif == 0 {
        WARN!("SHChangeNotifyRegister failed\n");
    }

    while iEntry > 0 {
        iEntry -= 1;
        ILFree(pidls[iEntry as usize].cast());
    }

    if (*this).m_nChangeNotif != 0 {
        S_OK
    } else {
        E_FAIL
    }
}

pub unsafe fn CRecycleBinFolderViewCB_TranslatePidl_body(this: *mut CRecycleBinFolderViewCB, ppidlNew: *mut LPITEMIDLIST, pidl: LPCITEMIDLIST) -> HRESULT {
*ppidlNew = core::ptr::null_mut();

    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if SHGetPathFromIDListW(pidl, szPath.as_mut_ptr()) == 0 {
        return E_FAIL;
    }

    let mut pidlChild: *mut ITEMIDLIST = core::ptr::null_mut();
CRecycleBin_ParseRecycleBinPath_RustBridge((*this).m_pRecycleBin.cast(), szPath.as_mut_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(pidlChild).cast(), core::ptr::null_mut());
    if pidlChild.is_null() {
        return E_FAIL;
    }

    *ppidlNew = ILCombine((*this).m_pidlParent.cast(), pidlChild.cast()).cast();

    return if !(*ppidlNew).is_null() { S_OK } else { E_OUTOFMEMORY };
}

pub unsafe fn CRecycleBinFolderViewCB_TranslateTwoPIDLs_body(this: *mut CRecycleBinFolderViewCB, pidls: *mut PIDLIST_ABSOLUTE) -> () {
let mut hr;
    if !(*pidls.add(0)).is_null() {
        let old_pidl = (*this).m_pidls[0];
        if !old_pidl.is_null() {
            CoTaskMemFree(old_pidl as *mut _);
            (*this).m_pidls[0] = core::ptr::null_mut();
        }
        hr = CRecycleBinFolderViewCB_TranslatePidl_body(this, core::ptr::addr_of_mut!((*this).m_pidls[0]).cast(),
            (*pidls.add(0)).cast(),
        );
        if !((hr) < 0) {
            *pidls.add(0) = (*this).m_pidls[0].cast();
        }
    }
    if !(*pidls.add(1)).is_null() {
        let old_pidl = (*this).m_pidls[1];
        if !old_pidl.is_null() {
            CoTaskMemFree(old_pidl as *mut _);
            (*this).m_pidls[1] = core::ptr::null_mut();
        }
        hr = CRecycleBinFolderViewCB_TranslatePidl_body(this, core::ptr::addr_of_mut!((*this).m_pidls[1]).cast(),
            (*pidls.add(1)).cast(),
        );
        if !((hr) < 0) {
            *pidls.add(1) = (*this).m_pidls[1].cast();
        }
    }
}

pub unsafe fn CRecycleBinFolderViewCB_MessageSFVCB_body(this: *mut CRecycleBinFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SFVM_QUERYFSNOTIFY => {
let mut hwndView: HWND = core::mem::zeroed();
            let hr: HRESULT = ((*(*(*this).m_pShellView).lpVtbl).GetWindow)((*this).m_pShellView, &mut hwndView);
            if ((hr) < 0) {
                return hr;
            }
            CRecycleBinFolderViewCB_RegisterChangeNotify_body(this, hwndView);
            S_OK
        }
        SFVM_FSNOTIFY => {
            CRecycleBinFolderViewCB_TranslateTwoPIDLs_body(this, wParam as *mut PIDLIST_ABSOLUTE);
            S_OK
        }
        _ => E_NOTIMPL
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     Recycle Bin virtual folder callback
//  * COPYRIGHT:   Copyright 2026 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(CRecycleBin);
//
// void CRecycleBinFolderViewCB::Initialize(IShellFolder2 *pRecycleBin, IShellView *psv, LPCITEMIDLIST pidlParent)
// {
//     m_pRecycleBin = pRecycleBin;
//     m_pShellView = psv;
//     m_pidlParent.Attach(ILClone(pidlParent));
// }
// HRESULT CRecycleBinFolderViewCB::RegisterChangeNotify(HWND hwndView)
// {
//     if (!hwndView)
//     {
//         WARN("!hwndView\n");
//         return E_FAIL;
//     }
//
//     LPITEMIDLIST pidls[RECYCLEBINMAXDRIVECOUNT] = { NULL };
//     SHChangeNotifyEntry entries[RECYCLEBINMAXDRIVECOUNT] = {};
//
//     // Populate entries
//     INT iEntry = 0;
//     for (INT iDrive = 0; iDrive < RECYCLEBINMAXDRIVECOUNT; ++iDrive)
//     {
//         WCHAR szBinPath[MAX_PATH];
//         HRESULT hr = GetRecycleBinPathFromDriveNumber(iDrive, szBinPath);
//         if (FAILED(hr))
//             continue;
//         LPITEMIDLIST pidl = ILCreateFromPathW(szBinPath);
//         if (!pidl)
//             continue;
//         entries[iEntry].pidl = pidls[iEntry] = pidl;
//         entries[iEntry].fRecursive = FALSE;
//         ++iEntry;
//     }
//
//     // Register
//     const DWORD dwFlags = SHCNRF_InterruptLevel | SHCNRF_ShellLevel | SHCNRF_NewDelivery;
//     const DWORD dwEvents =
//         SHCNE_CREATE |
//         SHCNE_DELETE |
//         SHCNE_RENAMEITEM |
//         SHCNE_UPDATEITEM |
//         SHCNE_MKDIR |
//         SHCNE_RMDIR |
//         SHCNE_RENAMEFOLDER |
//         SHCNE_UPDATEDIR |
//         SHCNE_UPDATEIMAGE |
//         SHCNE_ASSOCCHANGED;
//     m_nChangeNotif = SHChangeNotifyRegister(hwndView, dwFlags, dwEvents, SHV_CHANGE_NOTIFY,
//                                             iEntry, entries);
//     if (!m_nChangeNotif)
//         WARN("SHChangeNotifyRegister failed\n");
//
//     // Clean up
//     while (iEntry > 0)
//         ILFree(pidls[--iEntry]);
//
//     return m_nChangeNotif ? S_OK : E_FAIL;
// }
// HRESULT CRecycleBinFolderViewCB::TranslatePidl(LPITEMIDLIST *ppidlNew, LPCITEMIDLIST pidl)
// {
//     ATLASSERT(ppidlNew);
//     *ppidlNew = NULL;
//
//     WCHAR szPath[MAX_PATH];
//     if (!SHGetPathFromIDListW(pidl, szPath))
//     {
//         ERR("!SHGetPathFromIDListW\n");
//         return E_FAIL;
//     }
//
//     CComHeapPtr<ITEMIDLIST> pidlChild;
//     ATLASSERT(m_pRecycleBin);
//     CRecycleBin_ParseRecycleBinPath_RustBridge(m_pRecycleBin, szPath, NULL, &pidlChild, NULL);
//     if (!pidlChild)
//     {
//         ERR("!pidlChild\n");
//         return E_FAIL;
//     }
//
//     *ppidlNew = ILCombine(m_pidlParent, pidlChild);
//
//     return *ppidlNew ? S_OK : E_OUTOFMEMORY;
// }
// void CRecycleBinFolderViewCB::TranslateTwoPIDLs(PIDLIST_ABSOLUTE* pidls)
// {
//     ATLASSERT(pidls);
//
//     HRESULT hr;
//     if (pidls[0])
//     {
//         m_pidls[0].Free();
//         hr = TranslatePidl(&m_pidls[0], pidls[0]);
//         if (!FAILED_UNEXPECTEDLY(hr))
//             pidls[0] = m_pidls[0];
//     }
//     if (pidls[1])
//     {
//         m_pidls[1].Free();
//         hr = TranslatePidl(&m_pidls[1], pidls[1]);
//         if (!FAILED_UNEXPECTEDLY(hr))
//             pidls[1] = m_pidls[1];
//     }
// }
// HRESULT STDMETHODCALLTYPE
// CRecycleBinFolderViewCB::MessageSFVCB(UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case SFVM_QUERYFSNOTIFY: // Register change notification
//         {
//             // Now, we can get the view window
//             ATLASSERT(m_pShellView);
//             HWND hwndView;
//             HRESULT hr = m_pShellView->GetWindow(&hwndView);
//             if (FAILED_UNEXPECTEDLY(hr))
//                 return hr;
//             RegisterChangeNotify(hwndView);
//             return S_OK;
//         }
//         case SFVM_FSNOTIFY: // Change notification
//         {
//             TranslateTwoPIDLs((PIDLIST_ABSOLUTE*)wParam);
//             return S_OK;
//         }
//     }
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
