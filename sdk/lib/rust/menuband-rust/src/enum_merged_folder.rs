use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::menu_toolbars::StrRetToStrW;
use crate::enum_merged_folder_support::s_DsaDeleteCallback;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *const IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumIDList) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumIDList: IID;
}

// Deterministic COM scaffold for CEnumMergedFolder; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn MenuBandRustModuleLock();
    fn MenuBandRustModuleUnlock();
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
pub struct CEnumMergedFolder {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_UserLocalFolder: *mut IShellFolder,
    pub m_AllUSersFolder: *mut IShellFolder,
    pub m_HwndOwner: HWND,
    pub m_Flags: SHCONTF,
    pub m_hDsa: HDSA,
    pub m_hDsaIndex: UINT,
    pub m_hDsaCount: UINT,
}

unsafe fn CEnumMergedFolder_cpp_com_assign_m_UserLocalFolder(this: *mut CEnumMergedFolder, value: *mut IShellFolder) {
    let old = (*this).m_UserLocalFolder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_UserLocalFolder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CEnumMergedFolder_cpp_com_assign_m_AllUSersFolder(this: *mut CEnumMergedFolder, value: *mut IShellFolder) {
    let old = (*this).m_AllUSersFolder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_AllUSersFolder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_AddRef(this)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_Release(this)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_Skip_body(this, celt)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_Reset_body(this)
}
unsafe extern "system" fn CEnumMergedFolder_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumMergedFolder, IEnumIDList_iface)) as *mut CEnumMergedFolder;
    CEnumMergedFolder_Clone_body(this, ppenum)
}

pub static CENUMMERGEDFOLDER_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CEnumMergedFolder_IEnumIDList_QueryInterface,
    AddRef: CEnumMergedFolder_IEnumIDList_AddRef,
    Release: CEnumMergedFolder_IEnumIDList_Release,
    Next: CEnumMergedFolder_IEnumIDList_Next,
    Skip: CEnumMergedFolder_IEnumIDList_Skip,
    Reset: CEnumMergedFolder_IEnumIDList_Reset,
    Clone: CEnumMergedFolder_IEnumIDList_Clone,
};

unsafe fn CEnumMergedFolder_QueryInterface(
    this: *mut CEnumMergedFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CEnumMergedFolder_AddRef(this);
    S_OK
}

unsafe fn CEnumMergedFolder_AddRef(this: *mut CEnumMergedFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CEnumMergedFolder_Release(this: *mut CEnumMergedFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CEnumMergedFolder_dtor_body(this);
        CEnumMergedFolder_cpp_com_assign_m_UserLocalFolder(this, core::ptr::null_mut());
        CEnumMergedFolder_cpp_com_assign_m_AllUSersFolder(this, core::ptr::null_mut());
        MenuBandRustModuleUnlock();
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
pub unsafe fn CEnumMergedFolder_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CEnumMergedFolder>()) as *mut CEnumMergedFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CEnumMergedFolder {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CENUMMERGEDFOLDER_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_UserLocalFolder: core::mem::zeroed(),
        m_AllUSersFolder: core::mem::zeroed(),
        m_HwndOwner: core::mem::zeroed(),
        m_Flags: core::mem::zeroed(),
        m_hDsa: core::mem::zeroed(),
        m_hDsaIndex: core::mem::zeroed(),
        m_hDsaCount: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CEnumMergedFolder_ctor_body(this);
    let hr = CEnumMergedFolder_QueryInterface(this, riid, ppv);
    CEnumMergedFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

#[repr(C)]
pub struct LocalPidlInfo {
    pub shared: BOOL,
    pub parent: *mut IShellFolder,
    pub pidl: LPITEMIDLIST,
    pub pidl2: LPITEMIDLIST,
    pub parseName: LPCWSTR,
}

pub const CSTR_EQUAL: DWORD = 2;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const NORM_IGNORECASE: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_INFOLDER: DWORD = 1;

#[repr(C)]
pub struct SHCOLUMNID {
    pub fmtid: GUID,
    pub pid: DWORD,
}

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
}

extern "C" {
    static IID_IAugmentedShellFolder: GUID;
    static IID_IShellFolder: GUID;
    static IID_IShellFolder2: GUID;
}

pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

use core::ffi::c_void;

pub unsafe fn CEnumMergedFolder_ctor_body(this: *mut CEnumMergedFolder) -> () {
    (*this).m_UserLocalFolder = core::ptr::null_mut();
    (*this).m_AllUSersFolder = core::ptr::null_mut();
    (*this).m_HwndOwner = core::ptr::null_mut();
    (*this).m_Flags = 0;
    (*this).m_hDsa = core::ptr::null_mut();
    (*this).m_hDsaIndex = 0;
    (*this).m_hDsaCount = 0;
}

pub unsafe fn CEnumMergedFolder_dtor_body(this: *mut CEnumMergedFolder) -> () {
    DSA_DestroyCallback((*this).m_hDsa, Some(crate::enum_merged_folder_support::s_DsaDeleteCallback), this as *mut core::ffi::c_void);
}

pub unsafe fn CEnumMergedFolder_DsaDeleteCallback_body(this: *mut CEnumMergedFolder, info: *mut LocalPidlInfo) -> i32 {
    ILFree((*info).pidl);
    if !(*info).pidl2.is_null() {
        ILFree((*info).pidl2);
    }
    CoTaskMemFree((*info).parseName as *mut core::ffi::c_void);
    0
}

pub unsafe fn CEnumMergedFolder_s_DsaDeleteCallback_body(this: *mut CEnumMergedFolder, pItem: *mut core::ffi::c_void, pData: *mut core::ffi::c_void) -> i32 {
    let mf = pData as *mut CEnumMergedFolder;
    let item = pItem as *mut LocalPidlInfo;
    CEnumMergedFolder_DsaDeleteCallback_body(mf, item)
}

pub unsafe fn CEnumMergedFolder_SetSources_body(this: *mut CEnumMergedFolder, userLocal: *mut IShellFolder, allUSers: *mut IShellFolder) -> HRESULT {
    CEnumMergedFolder_cpp_com_assign_m_UserLocalFolder(this, userLocal);
    CEnumMergedFolder_cpp_com_assign_m_AllUSersFolder(this, allUSers);
    return S_OK;
}

pub unsafe fn CEnumMergedFolder_Begin_body(this: *mut CEnumMergedFolder, hwndOwner: HWND, flags: SHCONTF) -> HRESULT {
    unsafe {

        let mut hr: HRESULT;
        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();

        if (*this).m_hDsa as usize != 0 && (*this).m_HwndOwner == hwndOwner && (*this).m_Flags == flags {
            return CEnumMergedFolder_Reset_body(this);
        }

        let mut userLocal: *mut IEnumIDList = core::ptr::null_mut();
        let _userLocal_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(userLocal));
        let mut allUsers: *mut IEnumIDList = core::ptr::null_mut();
        let _allUsers_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(allUsers));

        hr = ((*(*(*this).m_UserLocalFolder).lpVtbl).EnumObjects)((*this).m_UserLocalFolder, hwndOwner.cast::<core::ffi::c_void>(), flags, core::ptr::addr_of_mut!(userLocal).cast());
        if ((hr) < 0) { return hr; }
        hr = ((*(*(*this).m_AllUSersFolder).lpVtbl).EnumObjects)((*this).m_AllUSersFolder, hwndOwner.cast::<core::ffi::c_void>(), flags, core::ptr::addr_of_mut!(allUsers).cast());
        if ((hr) < 0) { return hr; }

        if (*this).m_hDsa as usize == 0 {
            (*this).m_hDsa = DSA_Create(core::mem::size_of::<LocalPidlInfo>() as i32, 10);
        }

        DSA_EnumCallback((*this).m_hDsa, Some(crate::enum_merged_folder_support::s_DsaDeleteCallback), this as *mut core::ffi::c_void);
        DSA_DeleteAllItems((*this).m_hDsa);
        (*this).m_hDsaCount = 0;

        loop {
            hr = ((*(*userLocal).lpVtbl).Next)(userLocal, 1, core::ptr::addr_of_mut!(pidl), core::ptr::null_mut());
            if ((hr) < 0) { return hr; }
            if hr == S_FALSE { break; }

            let mut name: LPOLESTR = core::ptr::null_mut();
            let mut str: STRRET = core::mem::zeroed();
            str.uType = STRRET_WSTR;
            hr = ((*(*(*this).m_UserLocalFolder).lpVtbl).GetDisplayNameOf)((*this).m_UserLocalFolder, pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, core::ptr::addr_of_mut!(str).cast());
            if hr < 0 { return hr; }
            StrRetToStrW((core::ptr::addr_of_mut!(str)).cast(), pidl, core::ptr::addr_of_mut!(name));

            let mut info = LocalPidlInfo {
                shared: FALSE as i32,
                parent: (*this).m_UserLocalFolder,
                pidl: ILClone(pidl),
                pidl2: core::ptr::null_mut(),
                parseName: name,
            };

            ILFree(pidl);

            let mut idx: i32 = DSA_InsertItem((*this).m_hDsa, DSA_APPEND as i32, core::ptr::addr_of_mut!(info) as *mut LocalPidlInfo as *mut core::ffi::c_void);

            (*this).m_hDsaCount += 1;
        }

        loop {
            hr = ((*(*allUsers).lpVtbl).Next)(allUsers, 1, core::ptr::addr_of_mut!(pidl), core::ptr::null_mut());
            if ((hr) < 0) { return hr; }
            if hr == S_FALSE { break; }

            let mut name: LPOLESTR = core::ptr::null_mut();
            let mut str: STRRET = core::mem::zeroed();
            str.uType = STRRET_WSTR;
            hr = ((*(*(*this).m_AllUSersFolder).lpVtbl).GetDisplayNameOf)((*this).m_AllUSersFolder, pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, core::ptr::addr_of_mut!(str).cast());
            if hr < 0 { return hr; }
            StrRetToStrW((core::ptr::addr_of_mut!(str)).cast(), pidl, core::ptr::addr_of_mut!(name));

            let mut info = LocalPidlInfo {
                shared: FALSE as i32,
                parent: (*this).m_AllUSersFolder,
                pidl: ILClone(pidl),
                pidl2: core::ptr::null_mut(),
                parseName: name,
            };

            ILFree(pidl);

            let mut bShared: i32 = FALSE as i32;
            let mut i: i32 = 0;
            let count: i32 = (*this).m_hDsaCount as i32;
            while i < count {
                let pInfo: *mut LocalPidlInfo = DSA_GetItemPtr((*this).m_hDsa, i) as *mut LocalPidlInfo;

                let order: i32 = CompareStringW(
                    GetThreadLocale(),
                    NORM_IGNORECASE,
                    (*pInfo).parseName,
                    lstrlenW((*pInfo).parseName),
                    info.parseName,
                    lstrlenW(info.parseName),
                );

                if order == CSTR_EQUAL as i32 {
                    bShared = TRUE as i32;
                    (*pInfo).shared = TRUE as i32;
                    (*pInfo).pidl2 = info.pidl;
                    CoTaskMemFree(name as *mut core::ffi::c_void);
                    break;
                }
                i += 1;
            }

            if bShared == 0 {
                let mut idx: i32 = DSA_InsertItem((*this).m_hDsa, DSA_APPEND as i32, core::ptr::addr_of_mut!(info) as *mut LocalPidlInfo as *mut core::ffi::c_void);
                (*this).m_hDsaCount += 1;
            }
        }

        (*this).m_HwndOwner = hwndOwner;
        (*this).m_Flags = flags;

        return CEnumMergedFolder_Reset_body(this);
    }
}

pub unsafe fn CEnumMergedFolder_FindPidlInList_body(this: *mut CEnumMergedFolder, hwndOwner: HWND, pcidl: LPCITEMIDLIST, pinfo: *mut LocalPidlInfo) -> HRESULT {
    let mut hr: HRESULT;

    if (*this).m_hDsa as usize == 0 {
        let _ = CEnumMergedFolder_Begin_body(this, hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS);
    }

    let mut i: i32 = 0;
    while i < (*this).m_hDsaCount as i32 {
        let pInfo: *mut LocalPidlInfo = DSA_GetItemPtr((*this).m_hDsa, i) as *mut LocalPidlInfo;
        if pInfo.is_null() {
            return E_FAIL;
        }

        hr = ((*(*(*pInfo).parent).lpVtbl).CompareIDs)((*pInfo).parent, 0, (*pInfo).pidl, pcidl);
        if ((hr) < 0) {
            return hr;
        }

        if hr == S_OK {
            core::ptr::write(pinfo, core::ptr::read(pInfo));
            return S_OK;
        }

        i += 1;
    }

    return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG);
}

pub unsafe fn CEnumMergedFolder_FindByName_body(this: *mut CEnumMergedFolder, hwndOwner: HWND, strParsingName: LPCWSTR, pinfo: *mut LocalPidlInfo) -> HRESULT {
    if (*this).m_hDsa.is_null() {
        CEnumMergedFolder_Begin_body(this, hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS);
    }

    let mut i: i32 = 0;
    while i < (*this).m_hDsaCount as i32 {
        let pInfo = DSA_GetItemPtr((*this).m_hDsa, i) as *mut LocalPidlInfo;
        if pInfo.is_null() {
            return E_FAIL;
        }

        let order = CompareStringW(
            GetThreadLocale(),
            NORM_IGNORECASE,
            (*pInfo).parseName,
            lstrlenW((*pInfo).parseName),
            strParsingName,
            lstrlenW(strParsingName),
        );

        if order == CSTR_EQUAL as i32 {
            core::ptr::write(pinfo, core::ptr::read(pInfo));
            return S_OK;
        }

        i += 1;
    }

    HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG)
}

pub unsafe fn CEnumMergedFolder_Next_body(this: *mut CEnumMergedFolder, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }

    if (*this).m_hDsaIndex == (*this).m_hDsaCount {
        return S_FALSE;
    }

    let mut i: i32 = 0;
    while i < celt as i32 {
        let tinfo = DSA_GetItemPtr((*this).m_hDsa, (*this).m_hDsaIndex as i32) as *mut LocalPidlInfo;
        if tinfo.is_null() {
            return E_FAIL;
        }

        let info: LocalPidlInfo = core::ptr::read(tinfo);

        // FIXME: ILClone shouldn't be needed here! This should be causing leaks
        if !rgelt.is_null() {
            *rgelt.add(i as usize) = ILClone(info.pidl);
        }
        i += 1;

        (*this).m_hDsaIndex += 1;
        if (*this).m_hDsaIndex == (*this).m_hDsaCount {
            if !pceltFetched.is_null() {
                *pceltFetched = i as ULONG;
            }
            return if i == celt as i32 { S_OK } else { S_FALSE };
        }
    }

    if !pceltFetched.is_null() {
        *pceltFetched = celt;
    }
    return S_OK;
}

pub unsafe fn CEnumMergedFolder_Skip_body(this: *mut CEnumMergedFolder, celt: ULONG) -> HRESULT {
    return CEnumMergedFolder_Next_body(this, celt, core::ptr::null_mut(), core::ptr::null_mut());
}

pub unsafe fn CEnumMergedFolder_Reset_body(this: *mut CEnumMergedFolder) -> HRESULT {
    (*this).m_hDsaIndex = 0;
    S_OK
}

pub unsafe fn CEnumMergedFolder_Clone_body(this: *mut CEnumMergedFolder, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }
