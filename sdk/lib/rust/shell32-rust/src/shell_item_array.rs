// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellItemArray {
    pub lpVtbl: *const IShellItemArrayVtbl,
}
#[repr(C)]
pub struct IShellItemArrayVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellItemArray) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellItemArray) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        pbc: *mut IBindCtx,
        bhid: REFGUID,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetPropertyStore: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        flags: GETPROPERTYSTOREFLAGS,
        riid: REFIID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetPropertyDescriptionList: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        keyType: REFPROPERTYKEY,
        riid: REFIID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        AttribFlags: SIATTRIBFLAGS,
        sfgaoMask: SFGAOF,
        psfgaoAttribs: *mut SFGAOF,
    ) -> HRESULT,
    pub GetCount:
        unsafe extern "system" fn(This: *mut IShellItemArray, pdwNumItems: *mut DWORD) -> HRESULT,
    pub GetItemAt: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        dwIndex: DWORD,
        ppsi: *mut *mut IShellItem,
    ) -> HRESULT,
    pub EnumItems: unsafe extern "system" fn(
        This: *mut IShellItemArray,
        ppenumShellItems: *mut *mut IEnumShellItems,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IShellItemArray: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IBindCtx,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound:
        unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound:
        unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions:
        unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions:
        unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(
        This: *mut IBindCtx,
        pprot: *mut *mut IRunningObjectTable,
    ) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(
        This: *mut IBindCtx,
        pszKey: LPOLESTR,
        punk: *mut IUnknown,
    ) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(
        This: *mut IBindCtx,
        pszKey: LPOLESTR,
        ppunk: *mut *mut IUnknown,
    ) -> HRESULT,
    pub EnumObjectParam:
        unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam:
        unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumShellItems {
    pub lpVtbl: *const IEnumShellItemsVtbl,
}
#[repr(C)]
pub struct IEnumShellItemsVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IEnumShellItems,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumShellItems) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumShellItems) -> ULONG,
    pub Next: unsafe extern "system" fn(
        This: *mut IEnumShellItems,
        celt: ULONG,
        rgelt: *mut *mut IShellItem,
        pceltFetched: *mut ULONG,
    ) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumShellItems, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumShellItems) -> HRESULT,
    pub Clone: unsafe extern "system" fn(
        This: *mut IEnumShellItems,
        ppenum: *mut *mut IEnumShellItems,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumShellItems: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *const IShellItemVtbl,
}
#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellItem,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(
        This: *mut IShellItem,
        pbc: *mut IBindCtx,
        rbhid: REFGUID,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetParent:
        unsafe extern "system" fn(This: *mut IShellItem, ppsi: *mut *mut IShellItem) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        This: *mut IShellItem,
        sigdnName: SIGDN,
        ppszName: *mut LPWSTR,
    ) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        This: *mut IShellItem,
        sfgaoMask: SFGAOF,
        psfgaoAttribs: *mut SFGAOF,
    ) -> HRESULT,
    pub Compare: unsafe extern "system" fn(
        This: *mut IShellItem,
        psi: *mut IShellItem,
        hint: SICHINTF,
        piOrder: *mut i32,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IShellItem: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumString {
    pub lpVtbl: *const IEnumStringVtbl,
}
#[repr(C)]
pub struct IEnumStringVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IEnumString,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Next: unsafe extern "system" fn(
        This: *mut IEnumString,
        celt: ULONG,
        rgelt: *mut LPOLESTR,
        pceltFetched: *mut ULONG,
    ) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumString) -> HRESULT,
    pub Clone:
        unsafe extern "system" fn(This: *mut IEnumString, ppenum: *mut *mut IEnumString) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumString: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        grfFlags: DWORD,
        punkObject: *mut IUnknown,
        pmkObjectName: *mut IMoniker,
        pdwRegister: *mut DWORD,
    ) -> HRESULT,
    pub Revoke:
        unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        pmkObjectName: *mut IMoniker,
    ) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        pmkObjectName: *mut IMoniker,
        ppunkObject: *mut *mut IUnknown,
    ) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        dwRegister: DWORD,
        pfiletime: *mut FILETIME,
    ) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        pmkObjectName: *mut IMoniker,
        pfiletime: *mut FILETIME,
    ) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(
        This: *mut IRunningObjectTable,
        ppenumMoniker: *mut *mut IEnumMoniker,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IRunningObjectTable: IID;
}

// Deterministic from unknwn.h
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IUnknown,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
}
extern "C" {
    pub static IID_IUnknown: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumMoniker {
    pub lpVtbl: *const IEnumMonikerVtbl,
}
#[repr(C)]
pub struct IEnumMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IEnumMoniker,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(
        This: *mut IEnumMoniker,
        celt: ULONG,
        rgelt: *mut *mut IMoniker,
        pceltFetched: *mut ULONG,
    ) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(
        This: *mut IEnumMoniker,
        ppenum: *mut *mut IEnumMoniker,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IMoniker,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(
        This: *mut IMoniker,
        pStm: *mut IStream,
        fClearDirty: BOOL,
    ) -> HRESULT,
    pub GetSizeMax:
        unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        riidResult: REFIID,
        ppvResult: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        riid: REFIID,
        ppvObj: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        dwReduceHowFar: DWORD,
        ppmkToLeft: *mut *mut IMoniker,
        ppmkReduced: *mut *mut IMoniker,
    ) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(
        This: *mut IMoniker,
        pmkRight: *mut IMoniker,
        fOnlyIfNotGeneric: BOOL,
        ppmkComposite: *mut *mut IMoniker,
    ) -> HRESULT,
    pub Enum: unsafe extern "system" fn(
        This: *mut IMoniker,
        fForward: BOOL,
        ppenumMoniker: *mut *mut IEnumMoniker,
    ) -> HRESULT,
    pub IsEqual:
        unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        pmkNewlyRunning: *mut IMoniker,
    ) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        pFileTime: *mut FILETIME,
    ) -> HRESULT,
    pub Inverse:
        unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(
        This: *mut IMoniker,
        pmkOther: *mut IMoniker,
        ppmkPrefix: *mut *mut IMoniker,
    ) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(
        This: *mut IMoniker,
        pmkOther: *mut IMoniker,
        ppmkRelPath: *mut *mut IMoniker,
    ) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        ppszDisplayName: *mut LPOLESTR,
    ) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        pszDisplayName: LPOLESTR,
        pchEaten: *mut ULONG,
        ppmkOut: *mut *mut IMoniker,
    ) -> HRESULT,
    pub IsSystemMoniker:
        unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IStream,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(
        This: *mut IStream,
        pv: *mut core::ffi::c_void,
        cb: ULONG,
        pcbRead: *mut ULONG,
    ) -> HRESULT,
    pub Write: unsafe extern "system" fn(
        This: *mut IStream,
        pv: *const core::ffi::c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT,
    pub Seek: unsafe extern "system" fn(
        This: *mut IStream,
        dlibMove: LARGE_INTEGER,
        dwOrigin: DWORD,
        plibNewPosition: *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub SetSize:
        unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(
        This: *mut IStream,
        pstm: *mut IStream,
        cb: ULARGE_INTEGER,
        pcbRead: *mut ULARGE_INTEGER,
        pcbWritten: *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT,
    pub Stat: unsafe extern "system" fn(
        This: *mut IStream,
        pstatstg: *mut STATSTG,
        grfStatFlag: DWORD,
    ) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}
extern "C" {
    pub static IID_IStream: IID;
}

#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
}

// Deterministic COM scaffold for CShellItemArray; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"]
    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"]
    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize {
    fn cpp_com_size(self) -> usize;
}
impl CppComSize for usize {
    fn cpp_com_size(self) -> usize {
        self
    }
}
impl CppComSize for u32 {
    fn cpp_com_size(self) -> usize {
        self as usize
    }
}
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CShellItemArray {
    pub IShellItemArray_iface: IShellItemArray,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pCIDA: *mut CIDA,
    pub m_Medium: STGMEDIUM,
}

unsafe extern "system" fn CShellItemArray_IShellItemArray_QueryInterface(
    This: *mut IShellItemArray,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_AddRef(
    This: *mut IShellItemArray,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_AddRef(this)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_Release(
    This: *mut IShellItemArray,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_Release(this)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_BindToHandler(
    This: *mut IShellItemArray,
    pbc: *mut IBindCtx,
    bhid: REFGUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_BindToHandler_body(this, pbc, bhid, riid, ppvOut)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_GetPropertyStore(
    This: *mut IShellItemArray,
    flags: GETPROPERTYSTOREFLAGS,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_GetPropertyStore_body(this, flags, riid, ppv)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_GetPropertyDescriptionList(
    This: *mut IShellItemArray,
    keyType: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_GetPropertyDescriptionList_body(this, keyType, riid, ppv)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_GetAttributes(
    This: *mut IShellItemArray,
    AttribFlags: SIATTRIBFLAGS,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_GetAttributes_v1_body(this, AttribFlags, sfgaoMask, psfgaoAttribs)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_GetCount(
    This: *mut IShellItemArray,
    pdwNumItems: *mut DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_GetCount_body(this, pdwNumItems)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_GetItemAt(
    This: *mut IShellItemArray,
    dwIndex: DWORD,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_GetItemAt_body(this, dwIndex, ppsi)
}
unsafe extern "system" fn CShellItemArray_IShellItemArray_EnumItems(
    This: *mut IShellItemArray,
    ppenumShellItems: *mut *mut IEnumShellItems,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CShellItemArray,
        IShellItemArray_iface
    )) as *mut CShellItemArray;
    CShellItemArray_EnumItems_body(this, ppenumShellItems)
}

pub static CSHELLITEMARRAY_ISHELLITEMARRAY_VTBL: IShellItemArrayVtbl = IShellItemArrayVtbl {
    QueryInterface: CShellItemArray_IShellItemArray_QueryInterface,
    AddRef: CShellItemArray_IShellItemArray_AddRef,
    Release: CShellItemArray_IShellItemArray_Release,
    BindToHandler: CShellItemArray_IShellItemArray_BindToHandler,
    GetPropertyStore: CShellItemArray_IShellItemArray_GetPropertyStore,
    GetPropertyDescriptionList: CShellItemArray_IShellItemArray_GetPropertyDescriptionList,
    GetAttributes: CShellItemArray_IShellItemArray_GetAttributes,
    GetCount: CShellItemArray_IShellItemArray_GetCount,
    GetItemAt: CShellItemArray_IShellItemArray_GetItemAt,
    EnumItems: CShellItemArray_IShellItemArray_EnumItems,
};

unsafe fn CShellItemArray_QueryInterface(
    this: *mut CShellItemArray,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellItemArray))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IShellItemArray_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CShellItemArray_AddRef(this);
    S_OK
}

unsafe fn CShellItemArray_AddRef(this: *mut CShellItemArray) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CShellItemArray_Release(this: *mut CShellItemArray) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CShellItemArray_dtor_body(this);
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

// Internal activation: shellutils.h ShellObjectCreator<T> uses this non-aggregating factory.
pub unsafe fn CShellItemArray_CreateInstance(
    pdo: *mut IDataObject,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CShellItemArray>(),
    ) as *mut CShellItemArray;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CShellItemArray {
            IShellItemArray_iface: IShellItemArray {
                lpVtbl: core::ptr::addr_of!(CSHELLITEMARRAY_ISHELLITEMARRAY_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            m_pCIDA: core::mem::zeroed(),
            m_Medium: core::mem::zeroed(),
        },
    );
    Shell32RustObjectLock();
    CShellItemArray_ctor_body(this);
    let mut hr = CShellItemArray_Initialize_body(this, pdo);
    if hr >= 0 {
        hr = CShellItemArray_QueryInterface(this, riid, ppv);
    }
    CShellItemArray_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPOLESTR,
    pub r#type: DWORD,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

extern "system" {
    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
    fn SHCreateShellItem(
        parent: PCIDLIST_ABSOLUTE,
        folder: *mut IShellFolder,
        child: LPCITEMIDLIST,
        item: *mut *mut IShellItem,
    ) -> HRESULT;
    #[link_name = "CDataObjectHIDA_CreateCIDA_RustBridge"]
    fn CDataObjectHIDA_CreateCIDA(
        data: *mut IDataObject,
        cida: *mut *mut CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT;
    #[link_name = "CDataObjectHIDA_DestroyCIDA_RustBridge"]
    fn CDataObjectHIDA_DestroyCIDA(cida: *mut CIDA, medium: *mut STGMEDIUM);
}

extern "C" {
    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;
}

extern "C" {
    static BHID_DataObject: GUID;
    static BHID_SFObject: GUID;
    static BHID_SFUIObject: GUID;
    static BHID_SFViewObject: GUID;
    static CLSID_NULL: GUID;
    static CLSID_ShellItem: GUID;
    static IID_IDataObject: GUID;
    static IID_IPersistFolder2: GUID;
    static IID_IPersistIDList: GUID;
    static IID_IShellFolder: GUID;
}

pub struct CppComPtrGuard<T> {
    slot: *mut *mut T,
}
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self {
        Self { slot }
    }
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

use crate::shellcom::*;
use crate::types::*;
use core::ffi::c_void;

pub const TYMED_NULL: DWORD = 0;

#[inline]
unsafe fn HIDA_GetPIDLFolder(cida: *const CIDA) -> PCIDLIST_ABSOLUTE {
    (cida as *const u8).add((*cida).aoffset[0] as usize) as PCIDLIST_ABSOLUTE
}

#[inline]
unsafe fn HIDA_GetPIDLItem(cida: *const CIDA, index: UINT) -> PCUITEMID_CHILD {
    let offsets = core::ptr::addr_of!((*cida).aoffset) as *const UINT;
    (cida as *const u8).add(*offsets.add(index as usize + 1) as usize) as PCUITEMID_CHILD
}

pub unsafe fn CShellItemArray_ctor_body(this: *mut CShellItemArray) -> () {
    (*this).m_pCIDA = core::ptr::null_mut();
    (*this).m_Medium.tymed = TYMED_NULL;
}

pub unsafe fn CShellItemArray_dtor_body(this: *mut CShellItemArray) -> () {
    CDataObjectHIDA_DestroyCIDA((*this).m_pCIDA, core::ptr::addr_of_mut!((*this).m_Medium));
}

pub unsafe fn CShellItemArray_Initialize_body(
    this: *mut CShellItemArray,
    pdo: *mut IDataObject,
) -> HRESULT {
    CDataObjectHIDA_CreateCIDA(
        pdo,
        core::ptr::addr_of_mut!((*this).m_pCIDA),
        core::ptr::addr_of_mut!((*this).m_Medium),
    )
}

pub unsafe fn CShellItemArray_GetCount_helper_body(this: *mut CShellItemArray) -> UINT {
    (*(*this).m_pCIDA).cidl
}

pub unsafe fn CShellItemArray_BindToHandler_body(
    this: *mut CShellItemArray,
    pbc: *mut IBindCtx,
    rbhid: REFGUID,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CShellItemArray_GetPropertyStore_body(
    this: *mut CShellItemArray,
    flags: GETPROPERTYSTOREFLAGS,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CShellItemArray_GetPropertyDescriptionList_body(
    this: *mut CShellItemArray,
    keyType: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CShellItemArray_GetAttributes_v1_body(
    this: *mut CShellItemArray,
    dwAttribFlags: SIATTRIBFLAGS,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    *psfgaoAttribs = 0;
    E_NOTIMPL
}

pub unsafe fn CShellItemArray_GetCount_body(
    this: *mut CShellItemArray,
    pCount: *mut DWORD,
) -> HRESULT {
    *pCount = if !(*this).m_pCIDA.is_null() {
        CShellItemArray_GetCount_helper_body(this)
    } else {
        0
    };
    S_OK
}

pub unsafe fn CShellItemArray_GetItemAt_body(
    this: *mut CShellItemArray,
    nIndex: DWORD,
    ppItem: *mut *mut IShellItem,
) -> HRESULT {
    if ppItem.is_null() {
        return E_INVALIDARG;
    }
    *ppItem = core::ptr::null_mut();
    if (*this).m_pCIDA.is_null() {
        return E_UNEXPECTED;
    }
    if nIndex >= CShellItemArray_GetCount_helper_body(this) {
        return E_FAIL;
    }
    SHCreateShellItem(
        HIDA_GetPIDLFolder((*this).m_pCIDA),
        core::ptr::null_mut(),
        HIDA_GetPIDLItem((*this).m_pCIDA, nIndex),
        ppItem,
    )
}

pub unsafe fn CShellItemArray_EnumItems_body(
    this: *mut CShellItemArray,
    ppESI: *mut *mut IEnumShellItems,
) -> HRESULT {
    *ppESI = core::ptr::null_mut();
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellItemArrayFromDataObject(
    pdo: *mut IDataObject,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    CShellItemArray_CreateInstance(pdo, riid, ppv)
}


// CSimpleShellItemArray: IShellItemArray over a plain heap array of AddRef'd IShellItem*.
const SIATTRIBFLAGS_AND: SIATTRIBFLAGS = 1;
const SIATTRIBFLAGS_MASK: SIATTRIBFLAGS = 3;

#[repr(C)]
pub struct CSimpleShellItemArray {
    pub IShellItemArray_iface: IShellItemArray,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_items: *mut *mut IShellItem,
    pub m_count: usize,
}

unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_QueryInterface(
    This: *mut IShellItemArray,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_AddRef(
    This: *mut IShellItemArray,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_AddRef(this)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_Release(
    This: *mut IShellItemArray,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_Release(this)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_BindToHandler(
    This: *mut IShellItemArray,
    pbc: *mut IBindCtx,
    bhid: REFGUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_BindToHandler_body(this, pbc, bhid, riid, ppvOut)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_GetPropertyStore(
    This: *mut IShellItemArray,
    flags: GETPROPERTYSTOREFLAGS,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_GetPropertyStore_body(this, flags, riid, ppv)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_GetPropertyDescriptionList(
    This: *mut IShellItemArray,
    keyType: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_GetPropertyDescriptionList_body(this, keyType, riid, ppv)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_GetAttributes(
    This: *mut IShellItemArray,
    AttribFlags: SIATTRIBFLAGS,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_GetAttributes_body(this, AttribFlags, sfgaoMask, psfgaoAttribs)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_GetCount(
    This: *mut IShellItemArray,
    pdwNumItems: *mut DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_GetCount_body(this, pdwNumItems)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_GetItemAt(
    This: *mut IShellItemArray,
    dwIndex: DWORD,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_GetItemAt_body(this, dwIndex, ppsi)
}
unsafe extern "system" fn CSimpleShellItemArray_IShellItemArray_EnumItems(
    This: *mut IShellItemArray,
    ppenumShellItems: *mut *mut IEnumShellItems,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(
        CSimpleShellItemArray,
        IShellItemArray_iface
    )) as *mut CSimpleShellItemArray;
    CSimpleShellItemArray_EnumItems_body(this, ppenumShellItems)
}

pub static CSIMPLESHELLITEMARRAY_ISHELLITEMARRAY_VTBL: IShellItemArrayVtbl = IShellItemArrayVtbl {
    QueryInterface: CSimpleShellItemArray_IShellItemArray_QueryInterface,
    AddRef: CSimpleShellItemArray_IShellItemArray_AddRef,
    Release: CSimpleShellItemArray_IShellItemArray_Release,
    BindToHandler: CSimpleShellItemArray_IShellItemArray_BindToHandler,
    GetPropertyStore: CSimpleShellItemArray_IShellItemArray_GetPropertyStore,
    GetPropertyDescriptionList: CSimpleShellItemArray_IShellItemArray_GetPropertyDescriptionList,
    GetAttributes: CSimpleShellItemArray_IShellItemArray_GetAttributes,
    GetCount: CSimpleShellItemArray_IShellItemArray_GetCount,
    GetItemAt: CSimpleShellItemArray_IShellItemArray_GetItemAt,
    EnumItems: CSimpleShellItemArray_IShellItemArray_EnumItems,
};

unsafe fn CSimpleShellItemArray_QueryInterface(
    this: *mut CSimpleShellItemArray,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellItemArray))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IShellItemArray_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CSimpleShellItemArray_AddRef(this);
    S_OK
}

unsafe fn CSimpleShellItemArray_AddRef(this: *mut CSimpleShellItemArray) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CSimpleShellItemArray_Release(this: *mut CSimpleShellItemArray) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CSimpleShellItemArray_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

// ~CSimpleShellItemArray: CAtlArray<CComPtr<IShellItem>> releases every held item, then its buffer.
pub unsafe fn CSimpleShellItemArray_dtor_body(this: *mut CSimpleShellItemArray) -> () {
    let items = (*this).m_items;
    if !items.is_null() {
        for i in 0..(*this).m_count {
            let item = *items.add(i);
            if !item.is_null() {
                ((*(*item).lpVtbl).Release)(item);
            }
        }
        HeapFree(GetProcessHeap(), 0, items as *mut core::ffi::c_void);
        (*this).m_items = core::ptr::null_mut();
        (*this).m_count = 0;
    }
}

pub unsafe fn CSimpleShellItemArray_Initialize_body(
    this: *mut CSimpleShellItemArray,
    pitems: *mut *mut IShellItem,
    count: UINT,
) -> HRESULT {
    // m_items.SetCount(0) cannot fail here; the array is sized once up front instead of grown per Add.
    // A zero-length request still gets a valid 1-slot block so m_items is never a dangling pointer.
    let bytes = (count as usize).max(1) * core::mem::size_of::<*mut IShellItem>();
    let items = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, bytes) as *mut *mut IShellItem;
    if items.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).m_items = items;
    (*this).m_count = 0;
    for i in 0..count as usize {
        let ptr = *pitems.add(i);
        if ptr.is_null() {
            return E_INVALIDARG;
        }
        ((*(*ptr).lpVtbl).AddRef)(ptr);
        *items.add((*this).m_count) = ptr;
        (*this).m_count += 1;
    }
    S_OK
}

pub unsafe fn CSimpleShellItemArray_BindToHandler_body(
    this: *mut CSimpleShellItemArray,
    pbc: *mut IBindCtx,
    rbhid: REFGUID,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CSimpleShellItemArray_GetPropertyStore_body(
    this: *mut CSimpleShellItemArray,
    flags: GETPROPERTYSTOREFLAGS,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CSimpleShellItemArray_GetPropertyDescriptionList_body(
    this: *mut CSimpleShellItemArray,
    keyType: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = NULL;
    E_NOTIMPL
}

pub unsafe fn CSimpleShellItemArray_GetAttributes_body(
    this: *mut CSimpleShellItemArray,
    dwAttribFlags: SIATTRIBFLAGS,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut attr: SFGAOF = 0;

    if psfgaoAttribs.is_null() {
        return E_POINTER;
    }

    for i in 0..(*this).m_count {
        let item = *(*this).m_items.add(i);
        hr = ((*(*item).lpVtbl).GetAttributes)(item, sfgaoMask, &mut attr);
        if hr < 0 {
            break;
        }

        if i == 0 {
            *psfgaoAttribs = attr;
            continue;
        }

        if (dwAttribFlags & SIATTRIBFLAGS_MASK) == SIATTRIBFLAGS_AND {
            *psfgaoAttribs &= attr;
        } else {
            *psfgaoAttribs |= attr;
        }
    }

    if hr >= 0 {
        if *psfgaoAttribs == sfgaoMask {
            return S_OK;
        }
        return S_FALSE;
    }
    hr
}

pub unsafe fn CSimpleShellItemArray_GetCount_body(
    this: *mut CSimpleShellItemArray,
    pdwNumItems: *mut DWORD,
) -> HRESULT {
    if pdwNumItems.is_null() {
        return E_INVALIDARG;
    }
    *pdwNumItems = (*this).m_count as DWORD;
    S_OK
}

pub unsafe fn CSimpleShellItemArray_GetItemAt_body(
    this: *mut CSimpleShellItemArray,
    dwIndex: DWORD,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    if ppsi.is_null() {
        return E_INVALIDARG;
    }
    *ppsi = core::ptr::null_mut();
    if dwIndex as usize >= (*this).m_count {
        return E_FAIL;
    }
    let item = *(*this).m_items.add(dwIndex as usize);
    *ppsi = item;
    ((*(*item).lpVtbl).AddRef)(item);
    S_OK
}

pub unsafe fn CSimpleShellItemArray_EnumItems_body(
    this: *mut CSimpleShellItemArray,
    ppenumShellItems: *mut *mut IEnumShellItems,
) -> HRESULT {
    *ppenumShellItems = core::ptr::null_mut();
    E_NOTIMPL
}

// Internal activation: shellutils.h ShellObjectCreatorInit<T> uses this non-aggregating factory.
pub unsafe fn CreateShellItemArrayFromItems(
    items: *mut *mut IShellItem,
    cidl: UINT,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CSimpleShellItemArray>(),
    ) as *mut CSimpleShellItemArray;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CSimpleShellItemArray {
            IShellItemArray_iface: IShellItemArray {
                lpVtbl: core::ptr::addr_of!(CSIMPLESHELLITEMARRAY_ISHELLITEMARRAY_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            m_items: core::ptr::null_mut(),
            m_count: 0,
        },
    );
    Shell32RustObjectLock();
    let mut hr = CSimpleShellItemArray_Initialize_body(this, items, cidl);
    if hr >= 0 {
        hr = CSimpleShellItemArray_QueryInterface(this, riid, ppv);
    }
    CSimpleShellItemArray_Release(this);
    hr
}

// CComHeapPtr<IShellItem *> array: releases every created item, then frees the buffer.
unsafe fn CSimpleShellItemArray_free_scratch(array: *mut *mut IShellItem, cidl: UINT) {
    for i in 0..cidl as usize {
        let item = *array.add(i);
        if !item.is_null() {
            ((*(*item).lpVtbl).Release)(item);
        }
    }
    HeapFree(GetProcessHeap(), 0, array as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellItemArray(
    pidlParent: PCIDLIST_ABSOLUTE,
    psf: *mut IShellFolder,
    cidl: UINT,
    ppidl: *const PCUITEMID_CHILD,
    ppsiItemArray: *mut *mut IShellItemArray,
) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;

    if ppsiItemArray.is_null() {
        return E_POINTER;
    }
    *ppsiItemArray = core::ptr::null_mut();

    if pidlParent.is_null() && psf.is_null() {
        return E_POINTER;
    }

    if ppidl.is_null() {
        return E_INVALIDARG;
    }

    // CComHeapPtr::Allocate(0) allocates zero bytes; keep one slot so a valid block still comes back.
    let array = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (cidl as usize).max(1) * core::mem::size_of::<*mut IShellItem>(),
    ) as *mut *mut IShellItem;
    if array.is_null() {
        return E_OUTOFMEMORY;
    }

    for i in 0..cidl as usize {
        hr = SHCreateShellItem(pidlParent, psf, *ppidl.add(i) as LPCITEMIDLIST, array.add(i));
        if hr < 0 {
            break;
        }
    }

    if hr >= 0 {
        hr = CreateShellItemArrayFromItems(
            array,
            cidl,
            core::ptr::addr_of!(IID_IShellItemArray),
            ppsiItemArray as *mut *mut core::ffi::c_void,
        );
    }

    CSimpleShellItemArray_free_scratch(array, cidl);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellItemArrayFromIDLists(
    cidl: UINT,
    pidlArray: *const PCIDLIST_ABSOLUTE,
    ppsiItemArray: *mut *mut IShellItemArray,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if ppsiItemArray.is_null() {
        return E_POINTER;
    }
    *ppsiItemArray = core::ptr::null_mut();

    if cidl == 0 || pidlArray.is_null() {
        return E_INVALIDARG;
    }

    let array = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (cidl as usize) * core::mem::size_of::<*mut IShellItem>(),
    ) as *mut *mut IShellItem;
    if array.is_null() {
        return E_OUTOFMEMORY;
    }

    for i in 0..cidl as usize {
        hr = SHCreateShellItem(
            core::ptr::null(),
            core::ptr::null_mut(),
            *pidlArray.add(i) as LPCITEMIDLIST,
            array.add(i),
        );
        if hr < 0 {
            break;
        }
    }

    if hr >= 0 {
        hr = CreateShellItemArrayFromItems(
            array,
            cidl,
            core::ptr::addr_of!(IID_IShellItemArray),
            ppsiItemArray as *mut *mut core::ffi::c_void,
        );
    }

    CSimpleShellItemArray_free_scratch(array, cidl);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellItemArrayFromShellItem(
    psi: *mut IShellItem,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut items: [*mut IShellItem; 1] = [psi];

    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();

    if psi.is_null() {
        return E_INVALIDARG;
    }

    CreateShellItemArrayFromItems(items.as_mut_ptr(), 1, riid, ppv)
}
