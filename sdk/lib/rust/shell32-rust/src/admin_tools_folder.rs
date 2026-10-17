// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellFolder2 {
    pub lpVtbl: *const IShellFolder2Vtbl,
}
#[repr(C)]
pub struct IShellFolder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolder2) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        hwndOwner: HWND,
        pbcReserved: LPBC,
        lpszDisplayName: LPOLESTR,
        pchEaten: *mut ULONG,
        ppidl: *mut PIDLIST_RELATIVE,
        pdwAttributes: *mut ULONG,
    ) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        hwndOwner: HWND,
        grfFlags: SHCONTF,
        ppenumIDList: *mut *mut IEnumIDList,
    ) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        pidl: PCUIDLIST_RELATIVE,
        pbcReserved: LPBC,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        pidl: PCUIDLIST_RELATIVE,
        pbcReserved: LPBC,
        riid: REFIID,
        ppvObj: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        lParam: LPARAM,
        pidl1: PCUIDLIST_RELATIVE,
        pidl2: PCUIDLIST_RELATIVE,
    ) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        hwndOwner: HWND,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        cidl: UINT,
        apidl: PCUITEMID_CHILD_ARRAY,
        rgfInOut: *mut SFGAOF,
    ) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        hwndOwner: HWND,
        cidl: UINT,
        apidl: PCUITEMID_CHILD_ARRAY,
        riid: REFIID,
        prgfInOut: *mut UINT,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        pidl: PCUITEMID_CHILD,
        uFlags: SHGDNF,
        lpName: *mut STRRET,
    ) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        hwnd: HWND,
        pidl: PCUITEMID_CHILD,
        lpszName: LPCOLESTR,
        uFlags: SHGDNF,
        ppidlOut: *mut PITEMID_CHILD,
    ) -> HRESULT,
    pub GetDefaultSearchGUID:
        unsafe extern "system" fn(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT,
    pub EnumSearches: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT,
    pub GetDefaultColumn: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        dwReserved: DWORD,
        pSort: *mut ULONG,
        pDisplay: *mut ULONG,
    ) -> HRESULT,
    pub GetDefaultColumnState: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        iColumn: UINT,
        pcsFlags: *mut SHCOLSTATEF,
    ) -> HRESULT,
    pub GetDetailsEx: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        pidl: PCUITEMID_CHILD,
        pscid: *const SHCOLUMNID,
        pv: *mut VARIANT,
    ) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        pidl: PCUITEMID_CHILD,
        iColumn: UINT,
        psd: *mut SHELLDETAILS,
    ) -> HRESULT,
    pub MapColumnToSCID: unsafe extern "system" fn(
        This: *mut IShellFolder2,
        iColumn: UINT,
        pscid: *mut SHCOLUMNID,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolder2: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *const IShellFolderVtbl,
}
#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellFolder,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolder) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(
        This: *mut IShellFolder,
        hwndOwner: HWND,
        pbcReserved: LPBC,
        lpszDisplayName: LPOLESTR,
        pchEaten: *mut ULONG,
        ppidl: *mut PIDLIST_RELATIVE,
        pdwAttributes: *mut ULONG,
    ) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(
        This: *mut IShellFolder,
        hwndOwner: HWND,
        grfFlags: SHCONTF,
        ppenumIDList: *mut *mut IEnumIDList,
    ) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(
        This: *mut IShellFolder,
        pidl: PCUIDLIST_RELATIVE,
        pbcReserved: LPBC,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        This: *mut IShellFolder,
        pidl: PCUIDLIST_RELATIVE,
        pbcReserved: LPBC,
        riid: REFIID,
        ppvObj: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(
        This: *mut IShellFolder,
        lParam: LPARAM,
        pidl1: PCUIDLIST_RELATIVE,
        pidl2: PCUIDLIST_RELATIVE,
    ) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(
        This: *mut IShellFolder,
        hwndOwner: HWND,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(
        This: *mut IShellFolder,
        cidl: UINT,
        apidl: PCUITEMID_CHILD_ARRAY,
        rgfInOut: *mut SFGAOF,
    ) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(
        This: *mut IShellFolder,
        hwndOwner: HWND,
        cidl: UINT,
        apidl: PCUITEMID_CHILD_ARRAY,
        riid: REFIID,
        prgfInOut: *mut UINT,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(
        This: *mut IShellFolder,
        pidl: PCUITEMID_CHILD,
        uFlags: SHGDNF,
        lpName: *mut STRRET,
    ) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(
        This: *mut IShellFolder,
        hwnd: HWND,
        pidl: PCUITEMID_CHILD,
        lpszName: LPCOLESTR,
        uFlags: SHGDNF,
        ppidlOut: *mut PITEMID_CHILD,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolder: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IPersistFolder {
    pub lpVtbl: *const IPersistFolderVtbl,
}
#[repr(C)]
pub struct IPersistFolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IPersistFolder,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFolder) -> ULONG,
    pub GetClassID:
        unsafe extern "system" fn(This: *mut IPersistFolder, pClassID: *mut CLSID) -> HRESULT,
    pub Initialize:
        unsafe extern "system" fn(This: *mut IPersistFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFolder: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IPersistFolder2 {
    pub lpVtbl: *const IPersistFolder2Vtbl,
}
#[repr(C)]
pub struct IPersistFolder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IPersistFolder2,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFolder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFolder2) -> ULONG,
    pub GetClassID:
        unsafe extern "system" fn(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT,
    pub Initialize:
        unsafe extern "system" fn(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetCurFolder: unsafe extern "system" fn(
        This: *mut IPersistFolder2,
        ppidl: *mut PIDLIST_ABSOLUTE,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFolder2: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IPersist,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT,
}
extern "C" {
    pub static IID_IPersist: IID;
}

// Deterministic COM scaffold for CAdminToolsFolder; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CAdminToolsFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IShellFolder_iface: IShellFolder,
    pub IPersistFolder_iface: IPersistFolder,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IPersist_iface: IPersist,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pisfInner: *mut IShellFolder2,
    pub m_pidlInner: LPITEMIDLIST,
}

unsafe fn CAdminToolsFolder_cpp_com_assign_m_pisfInner(
    this: *mut CAdminToolsFolder,
    value: *mut IShellFolder2,
) {
    let old = (*this).m_pisfInner;
    if old == value {
        return;
    }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pisfInner = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_QueryInterface(
    This: *mut IShellFolder2,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_AddRef(
    This: *mut IShellFolder2,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_AddRef(this)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_Release(
    This: *mut IShellFolder2,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_Release(this)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_ParseDisplayName(
    This: *mut IShellFolder2,
    hwndOwner: HWND,
    pbcReserved: LPBC,
    lpszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppidl: *mut PIDLIST_RELATIVE,
    pdwAttributes: *mut ULONG,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_ParseDisplayName_body(
        this,
        hwndOwner,
        pbcReserved,
        lpszDisplayName,
        pchEaten,
        ppidl,
        pdwAttributes,
    )
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_EnumObjects(
    This: *mut IShellFolder2,
    hwndOwner: HWND,
    grfFlags: SHCONTF,
    ppenumIDList: *mut *mut IEnumIDList,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_BindToObject(
    This: *mut IShellFolder2,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_BindToStorage(
    This: *mut IShellFolder2,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_CompareIDs(
    This: *mut IShellFolder2,
    lParam: LPARAM,
    pidl1: PCUIDLIST_RELATIVE,
    pidl2: PCUIDLIST_RELATIVE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_CreateViewObject(
    This: *mut IShellFolder2,
    hwndOwner: HWND,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetAttributesOf(
    This: *mut IShellFolder2,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    rgfInOut: *mut SFGAOF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetUIObjectOf(
    This: *mut IShellFolder2,
    hwndOwner: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID,
    prgfInOut: *mut UINT,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDisplayNameOf(
    This: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    uFlags: SHGDNF,
    lpName: *mut STRRET,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_SetNameOf(
    This: *mut IShellFolder2,
    hwnd: HWND,
    pidl: PCUITEMID_CHILD,
    lpszName: LPCOLESTR,
    uFlags: SHGDNF,
    ppidlOut: *mut PITEMID_CHILD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDefaultSearchGUID(
    This: *mut IShellFolder2,
    lpguid: *mut GUID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_EnumSearches(
    This: *mut IShellFolder2,
    ppenum: *mut *mut IEnumExtraSearch,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDefaultColumn(
    This: *mut IShellFolder2,
    dwReserved: DWORD,
    pSort: *mut ULONG,
    pDisplay: *mut ULONG,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDefaultColumnState(
    This: *mut IShellFolder2,
    iColumn: UINT,
    pcsFlags: *mut SHCOLSTATEF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDetailsEx(
    This: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    pscid: *const SHCOLUMNID,
    pv: *mut VARIANT,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_GetDetailsOf(
    This: *mut IShellFolder2,
    pidl: PCUITEMID_CHILD,
    iColumn: UINT,
    psd: *mut SHELLDETAILS,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder2_MapColumnToSCID(
    This: *mut IShellFolder2,
    iColumn: UINT,
    pscid: *mut SHCOLUMNID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IShellFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CADMINTOOLSFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CAdminToolsFolder_IShellFolder2_QueryInterface,
    AddRef: CAdminToolsFolder_IShellFolder2_AddRef,
    Release: CAdminToolsFolder_IShellFolder2_Release,
    ParseDisplayName: CAdminToolsFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CAdminToolsFolder_IShellFolder2_EnumObjects,
    BindToObject: CAdminToolsFolder_IShellFolder2_BindToObject,
    BindToStorage: CAdminToolsFolder_IShellFolder2_BindToStorage,
    CompareIDs: CAdminToolsFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CAdminToolsFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CAdminToolsFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CAdminToolsFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CAdminToolsFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CAdminToolsFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CAdminToolsFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CAdminToolsFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CAdminToolsFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CAdminToolsFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CAdminToolsFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CAdminToolsFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CAdminToolsFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CAdminToolsFolder_IShellFolder_QueryInterface(
    This: *mut IShellFolder,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_AddRef(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_AddRef(this)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_Release(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_Release(this)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_ParseDisplayName(
    This: *mut IShellFolder,
    hwndOwner: HWND,
    pbcReserved: LPBC,
    lpszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppidl: *mut PIDLIST_RELATIVE,
    pdwAttributes: *mut ULONG,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_ParseDisplayName_body(
        this,
        hwndOwner,
        pbcReserved,
        lpszDisplayName,
        pchEaten,
        ppidl,
        pdwAttributes,
    )
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_EnumObjects(
    This: *mut IShellFolder,
    hwndOwner: HWND,
    grfFlags: SHCONTF,
    ppenumIDList: *mut *mut IEnumIDList,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_BindToObject(
    This: *mut IShellFolder,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_BindToStorage(
    This: *mut IShellFolder,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_CompareIDs(
    This: *mut IShellFolder,
    lParam: LPARAM,
    pidl1: PCUIDLIST_RELATIVE,
    pidl2: PCUIDLIST_RELATIVE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_CreateViewObject(
    This: *mut IShellFolder,
    hwndOwner: HWND,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_GetAttributesOf(
    This: *mut IShellFolder,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    rgfInOut: *mut SFGAOF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_GetUIObjectOf(
    This: *mut IShellFolder,
    hwndOwner: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID,
    prgfInOut: *mut UINT,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_GetDisplayNameOf(
    This: *mut IShellFolder,
    pidl: PCUITEMID_CHILD,
    uFlags: SHGDNF,
    lpName: *mut STRRET,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CAdminToolsFolder_IShellFolder_SetNameOf(
    This: *mut IShellFolder,
    hwnd: HWND,
    pidl: PCUITEMID_CHILD,
    lpszName: LPCOLESTR,
    uFlags: SHGDNF,
    ppidlOut: *mut PITEMID_CHILD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IShellFolder_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}

pub static CADMINTOOLSFOLDER_ISHELLFOLDER_VTBL: IShellFolderVtbl = IShellFolderVtbl {
    QueryInterface: CAdminToolsFolder_IShellFolder_QueryInterface,
    AddRef: CAdminToolsFolder_IShellFolder_AddRef,
    Release: CAdminToolsFolder_IShellFolder_Release,
    ParseDisplayName: CAdminToolsFolder_IShellFolder_ParseDisplayName,
    EnumObjects: CAdminToolsFolder_IShellFolder_EnumObjects,
    BindToObject: CAdminToolsFolder_IShellFolder_BindToObject,
    BindToStorage: CAdminToolsFolder_IShellFolder_BindToStorage,
    CompareIDs: CAdminToolsFolder_IShellFolder_CompareIDs,
    CreateViewObject: CAdminToolsFolder_IShellFolder_CreateViewObject,
    GetAttributesOf: CAdminToolsFolder_IShellFolder_GetAttributesOf,
    GetUIObjectOf: CAdminToolsFolder_IShellFolder_GetUIObjectOf,
    GetDisplayNameOf: CAdminToolsFolder_IShellFolder_GetDisplayNameOf,
    SetNameOf: CAdminToolsFolder_IShellFolder_SetNameOf,
};

unsafe extern "system" fn CAdminToolsFolder_IPersistFolder_QueryInterface(
    This: *mut IPersistFolder,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder_AddRef(
    This: *mut IPersistFolder,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_AddRef(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder_Release(
    This: *mut IPersistFolder,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_Release(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder_GetClassID(
    This: *mut IPersistFolder,
    pClassID: *mut CLSID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder_Initialize(
    This: *mut IPersistFolder,
    pidl: PCIDLIST_ABSOLUTE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_Initialize_body(this, pidl)
}

pub static CADMINTOOLSFOLDER_IPERSISTFOLDER_VTBL: IPersistFolderVtbl = IPersistFolderVtbl {
    QueryInterface: CAdminToolsFolder_IPersistFolder_QueryInterface,
    AddRef: CAdminToolsFolder_IPersistFolder_AddRef,
    Release: CAdminToolsFolder_IPersistFolder_Release,
    GetClassID: CAdminToolsFolder_IPersistFolder_GetClassID,
    Initialize: CAdminToolsFolder_IPersistFolder_Initialize,
};

unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_QueryInterface(
    This: *mut IPersistFolder2,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_AddRef(
    This: *mut IPersistFolder2,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_AddRef(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_Release(
    This: *mut IPersistFolder2,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_Release(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_GetClassID(
    This: *mut IPersistFolder2,
    pClassID: *mut CLSID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_Initialize(
    This: *mut IPersistFolder2,
    pidl: PCIDLIST_ABSOLUTE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CAdminToolsFolder_IPersistFolder2_GetCurFolder(
    This: *mut IPersistFolder2,
    ppidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CAdminToolsFolder,
        IPersistFolder2_iface
    )) as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetCurFolder_body(this, ppidl)
}

pub static CADMINTOOLSFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CAdminToolsFolder_IPersistFolder2_QueryInterface,
    AddRef: CAdminToolsFolder_IPersistFolder2_AddRef,
    Release: CAdminToolsFolder_IPersistFolder2_Release,
    GetClassID: CAdminToolsFolder_IPersistFolder2_GetClassID,
    Initialize: CAdminToolsFolder_IPersistFolder2_Initialize,
    GetCurFolder: CAdminToolsFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CAdminToolsFolder_IPersist_QueryInterface(
    This: *mut IPersist,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IPersist_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CAdminToolsFolder_IPersist_AddRef(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IPersist_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_AddRef(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersist_Release(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IPersist_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_Release(this)
}
unsafe extern "system" fn CAdminToolsFolder_IPersist_GetClassID(
    This: *mut IPersist,
    pClassID: *mut CLSID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CAdminToolsFolder, IPersist_iface))
        as *mut CAdminToolsFolder;
    CAdminToolsFolder_GetClassID_body(this, pClassID)
}

pub static CADMINTOOLSFOLDER_IPERSIST_VTBL: IPersistVtbl = IPersistVtbl {
    QueryInterface: CAdminToolsFolder_IPersist_QueryInterface,
    AddRef: CAdminToolsFolder_IPersist_AddRef,
    Release: CAdminToolsFolder_IPersist_Release,
    GetClassID: CAdminToolsFolder_IPersist_GetClassID,
};

unsafe fn CAdminToolsFolder_QueryInterface(
    this: *mut CAdminToolsFolder,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersist_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CAdminToolsFolder_AddRef(this);
    S_OK
}

unsafe fn CAdminToolsFolder_AddRef(this: *mut CAdminToolsFolder) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CAdminToolsFolder_Release(this: *mut CAdminToolsFolder) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CAdminToolsFolder_dtor_body(this);
        CAdminToolsFolder_cpp_com_assign_m_pisfInner(this, core::ptr::null_mut());
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_AdminFolderShortcut currently maps to CAdminToolsFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle]
pub unsafe extern "system" fn CAdminToolsFolder_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<CAdminToolsFolder>()) as _,
    ) as *mut CAdminToolsFolder;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CAdminToolsFolder {
            IShellFolder2_iface: IShellFolder2 {
                lpVtbl: core::ptr::addr_of!(CADMINTOOLSFOLDER_ISHELLFOLDER2_VTBL),
            },
            IShellFolder_iface: IShellFolder {
                lpVtbl: core::ptr::addr_of!(CADMINTOOLSFOLDER_ISHELLFOLDER_VTBL),
            },
            IPersistFolder_iface: IPersistFolder {
                lpVtbl: core::ptr::addr_of!(CADMINTOOLSFOLDER_IPERSISTFOLDER_VTBL),
            },
            IPersistFolder2_iface: IPersistFolder2 {
                lpVtbl: core::ptr::addr_of!(CADMINTOOLSFOLDER_IPERSISTFOLDER2_VTBL),
            },
            IPersist_iface: IPersist {
                lpVtbl: core::ptr::addr_of!(CADMINTOOLSFOLDER_IPERSIST_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            m_pisfInner: core::mem::zeroed(),
            m_pidlInner: core::mem::zeroed(),
        },
    );
    Shell32RustObjectLock();
    CAdminToolsFolder_ctor_body(this);
    let hr = CAdminToolsFolder_QueryInterface(this, riid, ppv);
    CAdminToolsFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

extern "system" {
    fn SHFree(memory: LPVOID);
    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
    #[link_name = "Shell32RustCoCreateInitSF"]
    fn SHELL32_CoCreateInitSF(
        root: LPCITEMIDLIST,
        clsid: *const GUID,
        csidl: INT,
        riid: REFIID,
        out: *mut *mut core::ffi::c_void,
    ) -> HRESULT;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    fn memcpy(
        dst: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        bytes: usize,
    ) -> *mut core::ffi::c_void;
}

extern "C" {
    static CLSID_AdminFolderShortcut: GUID;
    static CLSID_ShellFSFolder: GUID;
}

use crate::shellcom::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use crate::types::*;
use core::ffi::c_void;

pub unsafe fn CAdminToolsFolder_ctor_body(this: *mut CAdminToolsFolder) -> () {
    (*this).m_pidlInner = core::ptr::null_mut();
}

pub unsafe fn CAdminToolsFolder_dtor_body(this: *mut CAdminToolsFolder) -> () {
    if !(*this).m_pidlInner.is_null() {
        SHFree(((*this).m_pidlInner) as LPVOID);
    }
}

pub unsafe fn CAdminToolsFolder_ParseDisplayName_body(
    this: *mut CAdminToolsFolder,
    hwndOwner: HWND,
    pbc: LPBC,
    lpszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppidl: *mut PIDLIST_RELATIVE,
    pdwAttributes: *mut ULONG,
) -> HRESULT {
    let pInner = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).ParseDisplayName)(
        pInner,
        hwndOwner,
        pbc,
        lpszDisplayName,
        pchEaten,
        ppidl,
        pdwAttributes,
    )
}

pub unsafe fn CAdminToolsFolder_EnumObjects_body(
    this: *mut CAdminToolsFolder,
    hwndOwner: HWND,
    dwFlags: SHCONTF,
    ppEnumIDList: *mut *mut IEnumIDList,
) -> HRESULT {
    let pisf = (*this).m_pisfInner;
    ((*(*pisf).lpVtbl).EnumObjects)(pisf, hwndOwner, dwFlags, ppEnumIDList)
}

pub unsafe fn CAdminToolsFolder_BindToObject_body(
    this: *mut CAdminToolsFolder,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let inner = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).BindToObject)(inner, pidl, pbcReserved, riid, ppvOut)
}

pub unsafe fn CAdminToolsFolder_BindToStorage_body(
    this: *mut CAdminToolsFolder,
    pidl: PCUIDLIST_RELATIVE,
    pbcReserved: LPBC,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    ((*(*(*this).m_pisfInner).lpVtbl).BindToStorage)(
        (*this).m_pisfInner,
        pidl,
        pbcReserved,
        riid,
        ppvOut,
    )
}

pub unsafe fn CAdminToolsFolder_CompareIDs_body(
    this: *mut CAdminToolsFolder,
    lParam: LPARAM,
    pidl1: PCUIDLIST_RELATIVE,
    pidl2: PCUIDLIST_RELATIVE,
) -> HRESULT {
    let inner = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).CompareIDs)(inner, lParam as LPARAM, pidl1, pidl2)
}

pub unsafe fn CAdminToolsFolder_CreateViewObject_body(
    this: *mut CAdminToolsFolder,
    hwndOwner: HWND,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    ((*(*(*this).m_pisfInner).lpVtbl).CreateViewObject)(
        (*this).m_pisfInner,
        hwndOwner,
        riid,
        ppvOut,
    )
}

pub unsafe fn CAdminToolsFolder_GetAttributesOf_body(
    this: *mut CAdminToolsFolder,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    rgfInOut: *mut SFGAOF,
) -> HRESULT {
    let dw_admin_tools_attributes: u32 = SFGAO_STORAGE
        | SFGAO_STORAGEANCESTOR
        | SFGAO_FILESYSANCESTOR
        | SFGAO_FOLDER
        | SFGAO_FILESYSTEM
        | SFGAO_HASSUBFOLDER;

    if cidl != 0 {
        let inner = (*this).m_pisfInner;
        return ((*(*inner).lpVtbl).GetAttributesOf)(inner, cidl, apidl, rgfInOut);
    }

    if rgfInOut.is_null() || apidl.is_null() {
        return E_INVALIDARG;
    }

    if *rgfInOut == 0 {
        *rgfInOut = !0;
    }

    *rgfInOut &= dw_admin_tools_attributes;
    *rgfInOut &= !SFGAO_VALIDATE;

    S_OK
}

pub unsafe fn CAdminToolsFolder_GetUIObjectOf_body(
    this: *mut CAdminToolsFolder,
    hwndOwner: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID,
    prgfInOut: *mut UINT,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    ((*(*(*this).m_pisfInner).lpVtbl).GetUIObjectOf)(
        (*this).m_pisfInner,
        hwndOwner,
        cidl,
        apidl,
        riid,
        prgfInOut,
        ppvOut,
    )
}

pub unsafe fn CAdminToolsFolder_GetDisplayNameOf_body(
    this: *mut CAdminToolsFolder,
    pidl: PCUITEMID_CHILD,
    dwFlags: SHGDNF,
    strRet: *mut STRRET,
) -> HRESULT {
    if strRet.is_null() || pidl.is_null() {
        return E_INVALIDARG;
    }

    /* /* If we got an fs item just forward to the fs folder */ */
    if _ILIsSpecialFolder(pidl) == 0 {
        let pInner = (*this).m_pisfInner;
        return ((*(*pInner).lpVtbl).GetDisplayNameOf)(pInner, pidl, dwFlags, strRet);
    }

    /* The caller wants our path. Let fs folder handle it */
    let for_type = ((dwFlags as DWORD) & 0xff);
    let relation = ((dwFlags as DWORD) & 0xff00);
    if for_type == SHGDN_NORMAL && (relation & SHGDN_FORPARSING) != 0 {
        let mut pidl = pidl; // allow rebinding for the empty pidl case
                             /* Give an empty pidl to the fs folder to tell us its path */
        if (*pidl).mkid.cb != 0 {
            pidl = ILGetNext(pidl);
        }
        let pInner = (*this).m_pisfInner;
        return ((*(*pInner).lpVtbl).GetDisplayNameOf)(pInner, pidl, dwFlags, strRet);
    }

    E_INVALIDARG
}

pub unsafe fn CAdminToolsFolder_SetNameOf_body(
    this: *mut CAdminToolsFolder,
    hwndOwner: HWND,
    pidl: PCUITEMID_CHILD,
    lpName: LPCOLESTR,
    dwFlags: SHGDNF,
    pPidlOut: *mut PITEMID_CHILD,
) -> HRESULT {
    let pInner: *mut IShellFolder2 = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).SetNameOf)(pInner, hwndOwner, pidl, lpName, dwFlags, pPidlOut)
}

pub unsafe fn CAdminToolsFolder_GetDefaultSearchGUID_body(
    this: *mut CAdminToolsFolder,
    pguid: *mut GUID,
) -> HRESULT {
    let inner = (*this).m_pisfInner;
    let vtbl = (*(*inner).lpVtbl).GetDefaultSearchGUID;
    vtbl(inner, pguid)
}

pub unsafe fn CAdminToolsFolder_EnumSearches_body(
    this: *mut CAdminToolsFolder,
    ppenum: *mut *mut IEnumExtraSearch,
) -> HRESULT {
    let pInner = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).EnumSearches)(pInner, ppenum)
}

pub unsafe fn CAdminToolsFolder_GetDefaultColumn_body(
    this: *mut CAdminToolsFolder,
    dwRes: DWORD,
    pSort: *mut ULONG,
    pDisplay: *mut ULONG,
) -> HRESULT {
    let inner = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).GetDefaultColumn)(inner, dwRes, pSort, pDisplay)
}

pub unsafe fn CAdminToolsFolder_GetDefaultColumnState_body(
    this: *mut CAdminToolsFolder,
    iColumn: UINT,
    pcsFlags: *mut SHCOLSTATEF,
) -> HRESULT {
    let inner = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).GetDefaultColumnState)(inner, iColumn, pcsFlags)
}

pub unsafe fn CAdminToolsFolder_GetDetailsEx_body(
    this: *mut CAdminToolsFolder,
    pidl: PCUITEMID_CHILD,
    pscid: *const SHCOLUMNID,
    pv: *mut VARIANT,
) -> HRESULT {
    let p_inner = (*this).m_pisfInner;
    ((*(*p_inner).lpVtbl).GetDetailsEx)(p_inner, pidl, pscid, pv)
}

pub unsafe fn CAdminToolsFolder_GetDetailsOf_body(
    this: *mut CAdminToolsFolder,
    pidl: PCUITEMID_CHILD,
    iColumn: UINT,
    psd: *mut SHELLDETAILS,
) -> HRESULT {
    let p_inner: *mut IShellFolder2 = (*this).m_pisfInner;
    ((*(*p_inner).lpVtbl).GetDetailsOf)(p_inner, pidl, iColumn, psd)
}

pub unsafe fn CAdminToolsFolder_MapColumnToSCID_body(
    this: *mut CAdminToolsFolder,
    column: UINT,
    pscid: *mut SHCOLUMNID,
) -> HRESULT {
    let inner: *mut IShellFolder2 = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).MapColumnToSCID)(inner, column, pscid)
}

pub unsafe fn CAdminToolsFolder_GetClassID_body(
    this: *mut CAdminToolsFolder,
    lpClassId: *mut CLSID,
) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    memcpy(
        lpClassId as *mut _,
        &CLSID_AdminFolderShortcut as *const _ as *const _,
        ::core::mem::size_of::<CLSID>(),
    );
    S_OK
}

pub unsafe fn CAdminToolsFolder_Initialize_body(
    this: *mut CAdminToolsFolder,
    pidl: PCIDLIST_ABSOLUTE,
) -> HRESULT {
    (*this).m_pidlInner = ILClone(pidl);
    if (*this).m_pidlInner.is_null() {
        return E_OUTOFMEMORY;
    }
    SHELL32_CoCreateInitSF(
        (*this).m_pidlInner,
        &CLSID_ShellFSFolder,
        CSIDL_COMMON_ADMINTOOLS,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_pisfInner).cast(),
    )
}

pub unsafe fn CAdminToolsFolder_GetCurFolder_body(
    this: *mut CAdminToolsFolder,
    pidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    if pidl.is_null() {
        return E_POINTER;
    }
    *pidl = ILClone((*this).m_pidlInner);
    S_OK
}
