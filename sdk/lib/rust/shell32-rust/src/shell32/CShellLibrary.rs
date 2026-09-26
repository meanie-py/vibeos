use crate::prelude::*;
use crate::shellcom::SIGDN;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellLibrary {
    pub lpVtbl: *const IShellLibraryVtbl,
}
#[repr(C)]
pub struct IShellLibraryVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellLibrary, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellLibrary) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellLibrary) -> ULONG,
    pub AddFolder: unsafe extern "system" fn(This: *mut IShellLibrary, psiLocation: *mut IShellItem) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IShellLibrary) -> HRESULT,
    pub GetDefaultSaveFolder: unsafe extern "system" fn(This: *mut IShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetFolders: unsafe extern "system" fn(This: *mut IShellLibrary, lff: LIBRARYFOLDERFILTER, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetFolderType: unsafe extern "system" fn(This: *mut IShellLibrary, pftid: *mut FOLDERTYPEID) -> HRESULT,
    pub GetIcon: unsafe extern "system" fn(This: *mut IShellLibrary, ppszIcon: *mut LPWSTR) -> HRESULT,
    pub GetOptions: unsafe extern "system" fn(This: *mut IShellLibrary, plofOptions: *mut LIBRARYOPTIONFLAGS) -> HRESULT,
    pub LoadLibraryFromItem: unsafe extern "system" fn(This: *mut IShellLibrary, psiLibrary: *mut IShellItem, grfMode: DWORD) -> HRESULT,
    pub LoadLibraryFromKnownFolder: unsafe extern "system" fn(This: *mut IShellLibrary, kfidLibrary: REFKNOWNFOLDERID, grfMode: DWORD) -> HRESULT,
    pub RemoveFolder: unsafe extern "system" fn(This: *mut IShellLibrary, psiLocation: *mut IShellItem) -> HRESULT,
    pub ResolveFolder: unsafe extern "system" fn(This: *mut IShellLibrary, psiFolderToResolve: *mut IShellItem, dwTimeout: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IShellLibrary, psiFolderToSaveIn: *mut IShellItem, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT,
    pub SaveInKnownFolder: unsafe extern "system" fn(This: *mut IShellLibrary, kfidToSaveIn: REFKNOWNFOLDERID, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT,
    pub SetDefaultSaveFolder: unsafe extern "system" fn(This: *mut IShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, psi: *mut IShellItem) -> HRESULT,
    pub SetFolderType: unsafe extern "system" fn(This: *mut IShellLibrary, ftid: REFFOLDERTYPEID) -> HRESULT,
    pub SetIcon: unsafe extern "system" fn(This: *mut IShellLibrary, pszIcon: LPCWSTR) -> HRESULT,
    pub SetOptions: unsafe extern "system" fn(This: *mut IShellLibrary, lofMask: LIBRARYOPTIONFLAGS, lofOptions: LIBRARYOPTIONFLAGS) -> HRESULT,
}
extern "C" {
    pub static IID_IShellLibrary: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *const IShellItemVtbl,
}
#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellItem, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(This: *mut IShellItem, pbc: *mut IBindCtx, rbhid: REFGUID, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetParent: unsafe extern "system" fn(This: *mut IShellItem, ppsi: *mut *mut IShellItem) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IShellItem, sigdnName: SIGDN, ppszName: *mut LPWSTR) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(This: *mut IShellItem, sfgaoMask: SFGAOF, psfgaoAttribs: *mut SFGAOF) -> HRESULT,
    pub Compare: unsafe extern "system" fn(This: *mut IShellItem, psi: *mut IShellItem, hint: SICHINTF, piOrder: *mut i32) -> HRESULT,
}
extern "C" {
    pub static IID_IShellItem: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBindCtx, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(This: *mut IBindCtx, pprot: *mut *mut IRunningObjectTable) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, punk: *mut IUnknown) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRunningObjectTable, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT,
    pub Revoke: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD, pfiletime: *mut FILETIME) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, pfiletime: *mut FILETIME) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
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
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
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
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG, rgelt: *mut *mut IMoniker, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT,
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
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: REFIID, ppvResult: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, dwReduceHowFar: DWORD, ppmkToLeft: *mut *mut IMoniker, ppmkReduced: *mut *mut IMoniker) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT,
    pub Enum: unsafe extern "system" fn(This: *mut IMoniker, fForward: BOOL, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pmkNewlyRunning: *mut IMoniker) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pFileTime: *mut FILETIME) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkPrefix: *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkRelPath: *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
}

// Deterministic COM scaffold for CShellLibrary; method bodies are generated separately.
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

#[repr(C)]
pub struct CShellLibrary {
    pub IShellLibrary_iface: IShellLibrary,
    pub ref_count: core::sync::atomic::AtomicI32,
}

unsafe extern "system" fn CShellLibrary_IShellLibrary_QueryInterface(This: *mut IShellLibrary, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_AddRef(This: *mut IShellLibrary) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_AddRef(this)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_Release(This: *mut IShellLibrary) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_Release(this)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_AddFolder(This: *mut IShellLibrary, psiLocation: *mut IShellItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_AddFolder_body(this, psiLocation)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_Commit(This: *mut IShellLibrary) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_Commit_body(this)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_GetDefaultSaveFolder(This: *mut IShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_GetDefaultSaveFolder_body(this, dsft, riid, ppv)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_GetFolders(This: *mut IShellLibrary, lff: LIBRARYFOLDERFILTER, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_GetFolders_body(this, lff, riid, ppv)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_GetFolderType(This: *mut IShellLibrary, pftid: *mut FOLDERTYPEID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_GetFolderType_body(this, pftid)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_GetIcon(This: *mut IShellLibrary, ppszIcon: *mut LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_GetIcon_body(this, ppszIcon)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_GetOptions(This: *mut IShellLibrary, plofOptions: *mut LIBRARYOPTIONFLAGS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_GetOptions_body(this, plofOptions)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_LoadLibraryFromItem(This: *mut IShellLibrary, psiLibrary: *mut IShellItem, grfMode: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_LoadLibraryFromItem_body(this, psiLibrary, grfMode)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_LoadLibraryFromKnownFolder(This: *mut IShellLibrary, kfidLibrary: REFKNOWNFOLDERID, grfMode: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_LoadLibraryFromKnownFolder_body(this, kfidLibrary, grfMode)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_RemoveFolder(This: *mut IShellLibrary, psiLocation: *mut IShellItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_RemoveFolder_body(this, psiLocation)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_ResolveFolder(This: *mut IShellLibrary, psiFolderToResolve: *mut IShellItem, dwTimeout: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_ResolveFolder_body(this, psiFolderToResolve, dwTimeout, riid, ppv)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_Save(This: *mut IShellLibrary, psiFolderToSaveIn: *mut IShellItem, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_Save_v1_body(this, psiFolderToSaveIn, pszLibraryName, lsf, ppsiSavedTo)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_SaveInKnownFolder(This: *mut IShellLibrary, kfidToSaveIn: REFKNOWNFOLDERID, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_SaveInKnownFolder_body(this, kfidToSaveIn, pszLibraryName, lsf, ppsiSavedTo)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_SetDefaultSaveFolder(This: *mut IShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, psi: *mut IShellItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_SetDefaultSaveFolder_body(this, dsft, psi)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_SetFolderType(This: *mut IShellLibrary, ftid: REFFOLDERTYPEID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_SetFolderType_body(this, ftid)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_SetIcon(This: *mut IShellLibrary, pszIcon: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_SetIcon_body(this, pszIcon)
}
unsafe extern "system" fn CShellLibrary_IShellLibrary_SetOptions(This: *mut IShellLibrary, lofMask: LIBRARYOPTIONFLAGS, lofOptions: LIBRARYOPTIONFLAGS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLibrary, IShellLibrary_iface)) as *mut CShellLibrary;
    CShellLibrary_SetOptions_body(this, lofMask, lofOptions)
}

pub static CSHELLLIBRARY_ISHELLLIBRARY_VTBL: IShellLibraryVtbl = IShellLibraryVtbl {
    QueryInterface: CShellLibrary_IShellLibrary_QueryInterface,
    AddRef: CShellLibrary_IShellLibrary_AddRef,
    Release: CShellLibrary_IShellLibrary_Release,
    AddFolder: CShellLibrary_IShellLibrary_AddFolder,
    Commit: CShellLibrary_IShellLibrary_Commit,
    GetDefaultSaveFolder: CShellLibrary_IShellLibrary_GetDefaultSaveFolder,
    GetFolders: CShellLibrary_IShellLibrary_GetFolders,
    GetFolderType: CShellLibrary_IShellLibrary_GetFolderType,
    GetIcon: CShellLibrary_IShellLibrary_GetIcon,
    GetOptions: CShellLibrary_IShellLibrary_GetOptions,
    LoadLibraryFromItem: CShellLibrary_IShellLibrary_LoadLibraryFromItem,
    LoadLibraryFromKnownFolder: CShellLibrary_IShellLibrary_LoadLibraryFromKnownFolder,
    RemoveFolder: CShellLibrary_IShellLibrary_RemoveFolder,
    ResolveFolder: CShellLibrary_IShellLibrary_ResolveFolder,
    Save: CShellLibrary_IShellLibrary_Save,
    SaveInKnownFolder: CShellLibrary_IShellLibrary_SaveInKnownFolder,
    SetDefaultSaveFolder: CShellLibrary_IShellLibrary_SetDefaultSaveFolder,
    SetFolderType: CShellLibrary_IShellLibrary_SetFolderType,
    SetIcon: CShellLibrary_IShellLibrary_SetIcon,
    SetOptions: CShellLibrary_IShellLibrary_SetOptions,
};

unsafe fn CShellLibrary_QueryInterface(
    this: *mut CShellLibrary, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellLibrary)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellLibrary_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CShellLibrary_AddRef(this);
    S_OK
}

unsafe fn CShellLibrary_AddRef(this: *mut CShellLibrary) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CShellLibrary_Release(this: *mut CShellLibrary) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CShellLibrary_dtor_body(this);
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

// Activation: CLSID_ShellLibrary currently maps to CShellLibrary in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CShellLibrary_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CShellLibrary>()) as *mut CShellLibrary;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CShellLibrary {
        IShellLibrary_iface: IShellLibrary { lpVtbl: core::ptr::addr_of!(CSHELLLIBRARY_ISHELLLIBRARY_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
    });
    Shell32RustObjectLock();
    CShellLibrary_ctor_body(this);
    let hr = CShellLibrary_QueryInterface(this, riid, ppv);
    CShellLibrary_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_SHELLLIBRARY: UINT = 165;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

extern "C" {
    static CLSID_ShellLibrary: GUID;
}

use core::ffi::c_void;

pub unsafe fn CShellLibrary_ctor_body(this: *mut CShellLibrary) -> () {}

pub unsafe fn CShellLibrary_dtor_body(this: *mut CShellLibrary) -> () {}

pub unsafe fn CShellLibrary_AddFolder_body(this: *mut CShellLibrary, psiLocation: *mut IShellItem) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_Commit_body(this: *mut CShellLibrary) -> HRESULT { S_OK }

pub unsafe fn CShellLibrary_GetDefaultSaveFolder_body(this: *mut CShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_GetFolders_body(this: *mut CShellLibrary, lff: LIBRARYFOLDERFILTER, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_GetFolderType_body(this: *mut CShellLibrary, pftid: *mut FOLDERTYPEID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_GetIcon_body(this: *mut CShellLibrary, ppszIcon: *mut LPWSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_GetOptions_body(this: *mut CShellLibrary, plofOptions: *mut LIBRARYOPTIONFLAGS) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_LoadLibraryFromItem_body(this: *mut CShellLibrary, psiLibrary: *mut IShellItem, grfMode: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_LoadLibraryFromKnownFolder_body(this: *mut CShellLibrary, kfidLibrary: REFKNOWNFOLDERID, grfMode: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_RemoveFolder_body(this: *mut CShellLibrary, psiLocation: *mut IShellItem) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_ResolveFolder_body(this: *mut CShellLibrary, psiFolderToResolve: *mut IShellItem, dwTimeout: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_Save_v1_body(this: *mut CShellLibrary, psiFolderToSaveIn: *mut IShellItem, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_SaveInKnownFolder_body(this: *mut CShellLibrary, kfidToSaveIn: REFKNOWNFOLDERID, pszLibraryName: LPCWSTR, lsf: LIBRARYSAVEFLAGS, ppsiSavedTo: *mut *mut IShellItem) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_SetDefaultSaveFolder_body(this: *mut CShellLibrary, dsft: DEFAULTSAVEFOLDERTYPE, psi: *mut IShellItem) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_SetFolderType_body(this: *mut CShellLibrary, ftid: REFFOLDERTYPEID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_SetIcon_body(this: *mut CShellLibrary, pszIcon: LPCWSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLibrary_SetOptions_body(this: *mut CShellLibrary, lofMask: LIBRARYOPTIONFLAGS, lofOptions: LIBRARYOPTIONFLAGS) -> HRESULT { E_NOTIMPL }
