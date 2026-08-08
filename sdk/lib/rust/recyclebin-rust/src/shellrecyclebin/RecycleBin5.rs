use crate::prelude::*;
// Deterministic from recyclebin_v5.h
#[repr(C)]
pub struct IRecycleBin5 {
    pub lpVtbl: *const IRecycleBin5Vtbl,
}
#[repr(C)]
pub struct IRecycleBin5Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBin5, riid: IN REFIID, ppvObject: *mut *mut OUT void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBin5) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBin5) -> ULONG,
    pub DeleteFile: unsafe extern "system" fn(This: *mut IRecycleBin5, szFileName: IN LPCWSTR) -> HRESULT,
    pub EmptyRecycleBin: unsafe extern "system" fn(ppEnumList: *mut *mut ); STDMETHOD(EnumObjects)(THIS_ OUT IRecycleBinEnumList) -> HRESULT,
    pub GetDirectory: unsafe extern "system" fn(This: *mut IRecycleBin5, szPath: LPWSTR) -> HRESULT,
    pub Delete: unsafe extern "system" fn(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT,
    pub Restore: unsafe extern "system" fn(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT,
    pub RemoveFromDatabase: unsafe extern "system" fn(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT,
    pub OnClosing: unsafe extern "system" fn(This: *mut IRecycleBin5, prbel: *mut IN IRecycleBinEnumList) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBin5: IID;
}

// Deterministic from recyclebin.h
#[repr(C)]
pub struct IRecycleBinEnumList {
    pub lpVtbl: *const IRecycleBinEnumListVtbl,
}
#[repr(C)]
pub struct IRecycleBinEnumListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, celt: DWORD) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBinEnumList: IID;
}

// Deterministic from recyclebin.h
#[repr(C)]
pub struct IRecycleBinFile {
    pub lpVtbl: *const IRecycleBinFileVtbl,
}
#[repr(C)]
pub struct IRecycleBinFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBinFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> ULONG,
    pub IsEqualIdentity: unsafe extern "system" fn(This: *mut IRecycleBinFile, pFI: *const RECYCLEBINFILEIDENTITY) -> HRESULT,
    pub GetInfo: unsafe extern "system" fn(This: *mut IRecycleBinFile, pInfo: PDELETED_FILE_INFO) -> HRESULT,
    pub GetLastModificationTime: unsafe extern "system" fn(This: *mut IRecycleBinFile, pLastModificationTime: *mut FILETIME) -> HRESULT,
    pub GetDeletionTime: unsafe extern "system" fn(This: *mut IRecycleBinFile, pDeletionTime: *mut FILETIME) -> HRESULT,
    pub GetFileSize: unsafe extern "system" fn(This: *mut IRecycleBinFile, pFileSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub GetPhysicalFileSize: unsafe extern "system" fn(This: *mut IRecycleBinFile, pPhysicalFileSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(This: *mut IRecycleBinFile, pAttributes: *mut DWORD) -> HRESULT,
    pub GetFileName: unsafe extern "system" fn(This: *mut IRecycleBinFile, BufferSize: SIZE_T, Buffer: LPWSTR, RequiredSize: *mut SIZE_T) -> HRESULT,
    pub Delete: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
    pub Restore: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
    pub RemoveFromDatabase: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBinFile: IID;
}

// Deterministic COM scaffold for RecycleBin5; method bodies are generated separately.
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
pub struct RecycleBin5 {
    pub IRecycleBin5_iface: IRecycleBin5,
    pub m_ref: LONG,
    pub m_hInfo: HANDLE,
    pub m_hInfoMapped: HANDLE,
    pub m_EnumeratorCount: DWORD,
    pub m_VolumePath: CStringW,
    pub m_Folder: CStringW,
}

unsafe fn RecycleBin5_cpp_com_assign_m_VolumePath(this: *mut RecycleBin5, value: LPCWSTR) {
    (*this).m_VolumePath.assign(value);
}

unsafe fn RecycleBin5_cpp_com_assign_m_Folder(this: *mut RecycleBin5, value: LPCWSTR) {
    (*this).m_Folder.assign(value);
}

unsafe extern "system" fn RecycleBin5_IRecycleBin5_QueryInterface(This: *mut IRecycleBin5, riid: IN REFIID, ppvObject: *mut *mut OUT void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_QueryInterface_body(this, riid, ppvObject)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_AddRef(This: *mut IRecycleBin5) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_AddRef_body(this)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_Release(This: *mut IRecycleBin5) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_Release_body(this)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_DeleteFile(This: *mut IRecycleBin5, szFileName: IN LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_DeleteFile_body(this, szFileName)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_EmptyRecycleBin(ppEnumList: *mut *mut ); STDMETHOD(EnumObjects)(THIS_ OUT IRecycleBinEnumList) -> HRESULT {
    let this = (ppEnumList as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_EmptyRecycleBin_body(this)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_GetDirectory(This: *mut IRecycleBin5, szPath: LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_GetDirectory_body(this, szPath)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_Delete(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_Delete_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_Restore(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_Restore_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_RemoveFromDatabase(This: *mut IRecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_RemoveFromDatabase_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_OnClosing(This: *mut IRecycleBin5, prbel: *mut IN IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_OnClosing_body(this, prbel)
}

pub static RECYCLEBIN5_IRECYCLEBIN5_VTBL: IRecycleBin5Vtbl = IRecycleBin5Vtbl {
    QueryInterface: RecycleBin5_IRecycleBin5_QueryInterface,
    AddRef: RecycleBin5_IRecycleBin5_AddRef,
    Release: RecycleBin5_IRecycleBin5_Release,
    DeleteFile: RecycleBin5_IRecycleBin5_DeleteFile,
    EmptyRecycleBin: RecycleBin5_IRecycleBin5_EmptyRecycleBin,
    GetDirectory: RecycleBin5_IRecycleBin5_GetDirectory,
    Delete: RecycleBin5_IRecycleBin5_Delete,
    Restore: RecycleBin5_IRecycleBin5_Restore,
    RemoveFromDatabase: RecycleBin5_IRecycleBin5_RemoveFromDatabase,
    OnClosing: RecycleBin5_IRecycleBin5_OnClosing,
};

/// `new RecycleBin5` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBin5_new() -> *mut RecycleBin5 {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBin5>(),
    ) as *mut RecycleBin5;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBin5_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBIN5_IRECYCLEBIN5_VTBL);
    RecycleBin5_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `RecycleBin5_new`; freeing through any other allocator is a heap bug.
pub unsafe fn RecycleBin5_delete(this: *mut RecycleBin5) {
    if this.is_null() { return; }
    RecycleBin5_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Deterministic C++ support closure selected from source identifiers.

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const CREATE_ALWAYS: DWORD = 2;

pub const ERROR_ALREADY_EXISTS: DWORD = 183;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

pub const ERROR_INVALID_NAME: DWORD = 123;

pub const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;

pub const ERROR_NOT_SUPPORTED: DWORD = 50;

pub const ERROR_NO_MORE_FILES: DWORD = 18;

pub const ERROR_OUTOFMEMORY: DWORD = 14;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const ERROR_SHARING_VIOLATION: DWORD = 32;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

pub const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;

pub const FILE_END: DWORD = 2;

pub const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;

pub const FILE_PERSISTENT_ACLS: DWORD = 0x00000008;

pub const FO_MOVE: DWORD = 1;

pub const GENERIC_READ: DWORD = 0x80000000;

pub const GENERIC_WRITE: DWORD = 0x40000000;

pub const INT_MAX: DWORD = 2147483647;

pub const INVALID_FILE_SIZE: DWORD = 0xFFFFFFFF;

pub const NO_ERROR: DWORD = 0;

pub const OPEN_EXISTING: DWORD = 3;

pub const TokenUser: DWORD = 1;

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [WCHAR; 260],
    pub cAlternateFileName: [WCHAR; 14],
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct Folder {
    pub lpVtbl: *const FolderVtbl,
}
#[repr(C)]
pub struct FolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut Folder, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut Folder, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut Folder, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Title: unsafe extern "system" fn(This: *mut Folder, pbs: *mut BSTR) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_ParentFolder: unsafe extern "system" fn(This: *mut Folder, ppsf: *mut *mut Folder) -> HRESULT,
    pub Items: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut FolderItems) -> HRESULT,
    pub ParseName: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT,
    pub NewFolder: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, vOptions: VARIANT) -> HRESULT,
    pub MoveHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub CopyHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_Folder: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IDispatch {
    pub lpVtbl: *const IDispatchVtbl,
}
#[repr(C)]
pub struct IDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IDispatch: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeComp {
    pub lpVtbl: *const ITypeCompVtbl,
}
#[repr(C)]
pub struct ITypeCompVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeComp, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Bind: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, wFlags: WORD, ppTInfo: *mut *mut ITypeInfo, pDescKind: *mut DESCKIND, pBindPtr: *mut BINDPTR) -> HRESULT,
    pub BindType: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
}
extern "C" {
    pub static IID_ITypeComp: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeInfo {
    pub lpVtbl: *const ITypeInfoVtbl,
}
#[repr(C)]
pub struct ITypeInfoVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeInfo, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, ppTypeAttr: *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppFuncDesc: *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppVarDesc: *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, rgBstrNames: *mut BSTR, cMaxNames: UINT, pcNames: *mut UINT) -> HRESULT,
    pub GetRefTypeOfImplType: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pRefType: *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pImplTypeFlags: *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut ITypeInfo, rgszNames: *mut LPOLESTR, cNames: UINT, pMemId: *mut MEMBERID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut ITypeInfo, pvInstance: PVOID, memid: MEMBERID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, pBstrDllName: *mut BSTR, pBstrName: *mut BSTR, pwOrdinal: *mut WORD) -> HRESULT,
    pub GetRefTypeInfo: unsafe extern "system" fn(This: *mut ITypeInfo, hRefType: HREFTYPE, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, ppv: *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(This: *mut ITypeInfo, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObj: *mut PVOID) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrMops: *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib: unsafe extern "system" fn(This: *mut ITypeInfo, ppTLib: *mut *mut ITypeLib, pIndex: *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, pTypeAttr: *mut TYPEATTR) -> core::ffi::c_void,
    pub ReleaseFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pFuncDesc: *mut FUNCDESC) -> core::ffi::c_void,
    pub ReleaseVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pVarDesc: *mut VARDESC) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeInfo: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeLib {
    pub lpVtbl: *const ITypeLibVtbl,
}
#[repr(C)]
pub struct ITypeLibVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeLib, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut ITypeLib) -> UINT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, pTKind: *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid: unsafe extern "system" fn(This: *mut ITypeLib, guid: REFGUID, ppTinfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, ppTLibAttr: *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeLib, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeLib, index: INT, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub IsName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, pfName: *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, rgMemId: *mut MEMBERID, pcFound: *mut USHORT) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, pTLibAttr: *mut TLIBATTR) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeLib: IID;
}

#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANTARG,
    pub rgdispidNamedArgs: *mut DISPID,
    pub cArgs: UINT,
    pub cNamedArgs: UINT,
}

#[repr(C)]
pub struct IDLDESC {
    pub dwReserved: ULONG_PTR,
    pub wIDLFlags: USHORT,
}

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
}

extern "system" {
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn CreateFileW(name: LPCWSTR, access: DWORD, share: DWORD, security: LPVOID, creation: DWORD, flags: DWORD, template: HANDLE) -> HANDLE;
    fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;
    fn WriteFile(file: HANDLE, buffer: LPCVOID, bytes: DWORD, written: LPDWORD, overlapped: LPVOID) -> BOOL;
    fn GetLastError() -> DWORD;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn CoTaskMemFree(memory: LPVOID);
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR;
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
    static IID_IRecycleBin: GUID;
}

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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_8CC2401ABCDF: [WCHAR; 3] = [0x005C, 0x0044, 0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

pub unsafe fn SHELL_SingleFileOperation_body(hWnd: HWND, Op: UINT, pszFrom: LPCWSTR, pszTo: LPCWSTR, Flags: FILEOP_FLAGS) -> i32 {
    let mut szzFrom = CZZWStr::new();
    let mut szzTo = CZZWStr::new();
    if !szzFrom.Initialize(pszFrom) || !szzTo.Initialize(pszTo) {
        return ERROR_OUTOFMEMORY;
    }
    let mut fos: SHFILEOPSTRUCTW = core::mem::zeroed();
    fos.hwnd = hWnd;
    fos.wFunc = Op;
    fos.pFrom = szzFrom.c_str();
    fos.pTo = szzTo.c_str();
    fos.fFlags = Flags;
    SHFileOperationW(&mut fos)
}

pub unsafe fn IntDeleteRecursive_body(FullName: LPCWSTR) -> BOOL {
    let mut ret: i32 = FALSE;
    let mut hSearch: *mut core::ffi::c_void = INVALID_HANDLE_VALUE;
    let mut FullPath: *mut u16 = core::ptr::null_mut();

    'main: {
        let fa: u32 = GetFileAttributesW(FullName);
        if fa == INVALID_FILE_ATTRIBUTES {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                ret = TRUE;
            }
            break 'main;
        }
        if (fa & FILE_ATTRIBUTE_READONLY) != 0 {
            if SetFileAttributesW(FullName, fa & !FILE_ATTRIBUTE_READONLY) == 0 {
                break 'main;
            }
        }
        if (fa & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            let dwLength: u32 = wcslen(FullName);
            let heap = GetProcessHeap();
            let alloc_size = (dwLength
                .wrapping_add(1)
                .wrapping_add(MAX_PATH)
                .wrapping_add(1))
            .wrapping_mul(core::mem::size_of::<u16>() as u32);
            FullPath = HeapAlloc(heap, 0, alloc_size) as *mut u16;
            if FullPath.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'main;
            }
            wcscpy(FullPath, FullName);
            let mut dwLen = dwLength;
            if *FullPath.offset(dwLength as isize - 1) != '\\' as u16 {
                *FullPath.offset(dwLength as isize) = '\\' as u16;
                dwLen += 1;
            }
            let pFilePart: *mut u16 = FullPath.offset(dwLen as isize);
            wcscpy(pFilePart, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());
            let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
            hSearch = FindFirstFileW(FullPath, &mut FindData) as *mut core::ffi::c_void;
            if hSearch == INVALID_HANDLE_VALUE {
                break 'main;
            }
            loop {
                let c0 = FindData.cFileName[0 as usize];
                let c1 = FindData.cFileName[1 as usize];
                let c2 = FindData.cFileName[1 as usize];
                let c3 = FindData.cFileName[2 as usize];
                if !(c0 == '.' as u16 && (c1 == '\0' as u16 || (c2 == '.' as u16 && c3 == '\0' as u16))) {
                    wcscpy(pFilePart, FindData.cFileName.as_ptr());
                    if IntDeleteRecursive_body(FullPath as *const u16) == 0 {
                        FindClose(hSearch);
                        break 'main;
                    }
                }
                if FindNextFileW(hSearch, &mut FindData) == 0 {
                    break;
                }
            }
            FindClose(hSearch);
            if GetLastError() != ERROR_NO_MORE_FILES {
                break 'main;
            }
            if RemoveDirectoryW(FullName) == 0 {
                break 'main;
            }
        } else {
            if DeleteFileW(FullName) == 0 {
                break 'main;
            }
        }
        ret = TRUE;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        FullPath as *mut core::ffi::c_void,
    );
    ret
}

pub unsafe fn RecycleBin5_GetDirectory_body(this: *mut RecycleBin5, szPath: LPWSTR) -> HRESULT {
    if (*this).m_Folder[0] == 0 {
        return E_UNEXPECTED;
    }
    lstrcpynW(szPath, (*this).m_Folder, MAX_PATH);
    S_OK
}

pub unsafe fn RecycleBin5_QueryInterface_body(this: *mut RecycleBin5, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if cpp_com_iid_equal(riid, &IID_IUnknown) || cpp_com_iid_equal(riid, &IID_IRecycleBin) {
        *ppvObject = this as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, &IID_IRecycleBin5) {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    RecycleBin5_AddRef_body(this);
    S_OK
}

pub unsafe fn RecycleBin5_AddRef_body(this: *mut RecycleBin5) -> ULONG {
    ntdll_sys::nt::InterlockedIncrement(&mut (*this).m_ref) as u32
}

pub unsafe fn RecycleBin5_dtor_body(this: *mut RecycleBin5) -> () {
    if !(*this).m_hInfo.is_null() && (*this).m_hInfo != INVALID_HANDLE_VALUE {
        CloseHandle((*this).m_hInfo);
    }
    if !(*this).m_hInfoMapped.is_null() {
        CloseHandle((*this).m_hInfoMapped);
    }
}

pub unsafe fn RecycleBin5_Release_body(this: *mut RecycleBin5) -> ULONG {
    let refCount = InterlockedDecrement(&mut (*this).m_ref);
    if refCount == 0 {
        __rustconv_delete(this);
    }
    refCount
}

pub unsafe fn RecycleBin5_DeleteFile_body(this: *mut RecycleBin5, szFileName: IN LPCWSTR) -> HRESULT {
    let mut szFullName: LPWSTR = core::ptr::null_mut();
    let mut dwBufferLength: u32 = 0;
    let mut lpFilePart: LPWSTR = core::ptr::null_mut();
    let mut len: u32;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut pHeader: *mut INFO2_HEADER = core::ptr::null_mut();
    let mut pDeletedFile: *mut DELETED_FILE_RECORD;
    let mut FileSize: ULARGE_INTEGER;
    let mut dwAttributes: u32;
    let mut dwEntries: u32;
    let mut SystemTime: SYSTEMTIME;
    let mut hr: HRESULT = 0;
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut DeletedFileName: CStringW = CStringW::new();
    let mut szUniqueId: [u16; 64 as usize] = [0; 64 as usize];

    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION as LONG);
    }

    'main_body: loop {
        // Get full file name
        loop {
            len = GetFullPathNameW(
                szFileName,
                dwBufferLength,
                szFullName,
                core::ptr::addr_of_mut!(lpFilePart),
            );
            if len == 0 {
                if !szFullName.is_null() {
                    CoTaskMemFree(szFullName);
                    szFullName = core::ptr::null_mut();
                }
                hr = HResultFromWin32(GetLastError());
                break 'main_body;
            } else if len < dwBufferLength {
                break;
            }
            if !szFullName.is_null() {
                CoTaskMemFree(szFullName);
                szFullName = core::ptr::null_mut();
            }
            dwBufferLength = len;
            szFullName = CoTaskMemAlloc(
                (dwBufferLength as usize) * core::mem::size_of::<u16>(),
            ) as LPWSTR;
            if szFullName.is_null() {
                hr = HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY as LONG);
                break 'main_body;
            }
        }

        dwAttributes = GetFileAttributesW(szFullName);
        if dwAttributes == INVALID_FILE_ATTRIBUTES {
            CoTaskMemFree(szFullName);
            szFullName = core::ptr::null_mut();
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }

        if dwBufferLength < 2
            || *szFullName.add(1 as usize) != 0x003A_u16
        {
            CoTaskMemFree(szFullName);
            szFullName = core::ptr::null_mut();
            hr = HRESULT_FROM_WIN32(ERROR_INVALID_NAME as LONG);
            break 'main_body;
        }

        hFile = CreateFileW(
            szFullName,
            0,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                FILE_FLAG_BACKUP_SEMANTICS
            } else {
                0
            },
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }

        CloseHandle((*this).m_hInfoMapped);
        SetFilePointer(
            (*this).m_hInfo,
            core::mem::size_of::<DELETED_FILE_RECORD>() as i32,
            core::ptr::null_mut(),
            FILE_END,
        );
        SetEndOfFile((*this).m_hInfo);
        (*this).m_hInfoMapped = CreateFileMappingW(
            (*this).m_hInfo,
            core::ptr::null_mut(),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            0,
            core::ptr::null_mut(),
        );
        if (*this).m_hInfoMapped.is_null() {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }

        pHeader = MapViewOfFile(
            (*this).m_hInfoMapped,
            FILE_MAP_WRITE,
            0,
            0,
            0,
        ) as *mut INFO2_HEADER;
        if pHeader.is_null() {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }

        FileSize.u.__bindgen_anon_1.LowPart =
            GetFileSize((*this).m_hInfo, &mut FileSize.u.__bindgen_anon_1.HighPart);
        if FileSize.u.__bindgen_anon_1.LowPart < core::mem::size_of::<INFO2_HEADER>() as u32 {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        dwEntries = ((FileSize.QuadPart
            - core::mem::size_of::<INFO2_HEADER>() as u64)
            / core::mem::size_of::<DELETED_FILE_RECORD>() as u64)
            as u32
            - 1;
        pDeletedFile = (pHeader.add(1 as usize) as *mut DELETED_FILE_RECORD)
            .add(dwEntries as usize);

        FileSize.u.__bindgen_anon_1.LowPart =
            GetFileSize(hFile, &mut FileSize.u.__bindgen_anon_1.HighPart);
        if FileSize.u.__bindgen_anon_1.LowPart == INVALID_FILE_SIZE
            && GetLastError() != NO_ERROR
        {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        if FileSize.u.__bindgen_anon_1.HighPart != 0 {
            hr = HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED as LONG);
            break 'main_body;
        }
        (*pHeader).dwTotalLogicalSize += FileSize.u.__bindgen_anon_1.LowPart;

        let Extension: LPCWSTR = PathFindExtensionW(szFullName);
        ZeroMemory(
            pDeletedFile as *mut c_void,
            core::mem::size_of::<DELETED_FILE_RECORD>(),
        );
        if dwEntries == 0 {
            (*pDeletedFile).dwRecordUniqueId = 0;
        } else {
            let pLastDeleted = (pHeader.add(1 as usize)
                as *mut DELETED_FILE_RECORD)
                .add((dwEntries - 1) as usize);
            (*pDeletedFile).dwRecordUniqueId =
                (*pLastDeleted).dwRecordUniqueId + 1;
        }

        (*pDeletedFile).dwDriveNumber =
            tolower(*szFullName.add(0 as usize) as i32) - 0x61u32;
        _ultow(
            (*pDeletedFile).dwRecordUniqueId,
            szUniqueId.as_mut_ptr(),
            10,
        );

        DeletedFileName.assign_cstringw(&(*this).m_Folder);
        DeletedFileName.append_str(CPP_COM_WSTR_8CC2401ABCDF.as_ptr());
        let drive_char: u16 = 0x0061u16 + (*pDeletedFile).dwDriveNumber as u16;
        DeletedFileName.append_char(drive_char);
        DeletedFileName.append_str(szUniqueId.as_ptr());
        DeletedFileName.append_str(Extension);

        let mut SectorsPerCluster: u32 = 0;
        let mut BytesPerSector: u32 = 0;
        if GetDiskFreeSpaceW(
            (*this).m_VolumePath.as_ptr(),
            &mut SectorsPerCluster,
            &mut BytesPerSector,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        let ClusterSize = BytesPerSector * SectorsPerCluster;

        GetSystemTime(&mut SystemTime);
        if SystemTimeToFileTime(
            &SystemTime,
            &mut (*pDeletedFile).DeletionTime,
        ) == 0
        {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        (*pDeletedFile).dwPhysicalFileSize =
            ROUND_UP(FileSize.u.__bindgen_anon_1.LowPart, ClusterSize);

        wcscpy(
            (*pDeletedFile).FileNameW.as_mut_ptr(),
            szFullName,
        );
        if WideCharToMultiByte(
            CP_ACP,
            0,
            (*pDeletedFile).FileNameW.as_ptr(),
            -1,
            (*pDeletedFile).FileNameA.as_mut_ptr() as LPSTR,
            MAX_PATH,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32(ERROR_INVALID_NAME as LONG);
            SetLastError(ERROR_INVALID_NAME);
            break 'main_body;
        }

        wfd.dwFileAttributes = dwAttributes;
        wfd.nFileSizeLow = FileSize.u.__bindgen_anon_1.LowPart;
        GetFileTime(
            hFile,
            &mut wfd.ftCreationTime,
            &mut wfd.ftLastAccessTime,
            &mut wfd.ftLastWriteTime,
        );

        if MoveFileW(szFullName, DeletedFileName.as_ptr()) != 0 {
            hr = S_OK;
        } else {
            hr = HResultFromWin32(GetLastError());
        }

        if hr >= 0 {
            let ident = RECYCLEBINFILEIDENTITY {
                DeletionTime: (*pDeletedFile).DeletionTime,
                FileName: DeletedFileName.as_ptr(),
            };
            CRecycleBin_NotifyRecycled(szFullName, &wfd, &ident);
        }

        break 'main_body;
    }

    if !pHeader.is_null() {
        UnmapViewOfFile(pHeader as *const c_void);
    }
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    if !szFullName.is_null() {
        CoTaskMemFree(szFullName);
    }
    hr
}

pub unsafe fn RecycleBin5_EmptyRecycleBin_body(this: *mut RecycleBin5) -> HRESULT {
    loop {
        let mut prbel: *mut IRecycleBinEnumList = ::core::ptr::null_mut();
        let hr: HRESULT = RecycleBin5_EnumObjects_body(this, &mut prbel);
        if !(hr >= 0) {
            return hr;
        }

        let mut prbf: *mut IRecycleBinFile = ::core::ptr::null_mut();
        let hr: HRESULT = ((*(*prbel).lpVtbl).Next)(
            prbel,
            1,
            &mut prbf,
            ::core::ptr::null_mut(),
        );
        ((*(*prbel).lpVtbl).Release)(prbel);
        if hr == S_FALSE {
            return S_OK;
        }
        let hr: HRESULT = ((*(*prbf).lpVtbl).Delete)(prbf);
        ((*(*prbf).lpVtbl).Release)(prbf);
        if !(hr >= 0) {
            return hr;
        }
    }
}

pub unsafe fn RecycleBin5_EnumObjects_body(this: *mut RecycleBin5, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT {
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();

    // Extract the internal string pointer from the embedded CStringW field.
    // ATL CStringW typically stores the buffer pointer as its first (and only) member.
    let folder_ptr: *const u16 = *(&(*this).m_Folder as *const CStringW as *const *const u16);

    let mut hr = RecycleBin5Enum_Constructor(
        this,
        (*this).m_hInfo,
        (*this).m_hInfoMapped,
        folder_ptr,
        &mut pUnk,
    );

    if hr < 0 {
        return hr;
    }

    let mut prbel: *mut IRecycleBinEnumList = core::ptr::null_mut();
    hr = ((*(*pUnk).lpVtbl).QueryInterface)(
        pUnk,
        &IID_IRecycleBinEnumList as *const GUID,
        &mut prbel as *mut *mut _ as *mut *mut core::ffi::c_void,
    );

    if hr >= 0 {
        (*this).m_EnumeratorCount += 1;
        *ppEnumList = prbel;
    }

    ((*(*pUnk).lpVtbl).Release)(pUnk);
    hr
}

pub unsafe fn RecycleBin5_Delete_v1_body(this: *mut RecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let mut res: HRESULT = IntDeleteRecursive_body(pDeletedFileName) as HRESULT;
    if res == 0 {
        return HResultFromWin32(GetLastError());
    }
    res = RecycleBin5_RemoveFromDatabase_v1_body(this, pDeletedFileName, pDeletedFile);
    if res == 0 as HRESULT {
        SHUpdateRecycleBinIcon();
    }
    res
}

pub unsafe fn RecycleBin5_Restore_v1_body(this: *mut RecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let mut res = SHELL_SingleFileOperation_body(
        core::ptr::null_mut(),
        FO_MOVE,
        pDeletedFileName,
        (*pDeletedFile).FileNameW,
        0,
    );
    if res != 0 {
        return E_FAIL;
    }
    res = RecycleBin5_RemoveFromDatabase_v1_body(this, pDeletedFileName, pDeletedFile);
    if res == 0 {
        SHUpdateRecycleBinIcon();
    }
    res
}

pub unsafe fn RecycleBin5_RemoveFromDatabase_v1_body(this: *mut RecycleBin5, pDeletedFileName: IN LPCWSTR, pDeletedFile: *mut IN DELETED_FILE_RECORD) -> HRESULT {
    let mut FileSize: ULARGE_INTEGER = ULARGE_INTEGER::default();
    let pHeader: PINFO2_HEADER;
    let mut pRecord: *const DELETED_FILE_RECORD;
    let pLast: *const DELETED_FILE_RECORD;
    let dwEntries: DWORD;
    let mut i: DWORD;

    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION as LONG);
    }

    pHeader = MapViewOfFile(
        (*this).m_hInfoMapped,
        FILE_MAP_WRITE,
        0,
        0,
        0,
    ) as PINFO2_HEADER;
    if pHeader.is_null() {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    FileSize.u.__bindgen_anon_1.LowPart = GetFileSize((*this).m_hInfo, &mut FileSize.u.__bindgen_anon_1.HighPart);
    if FileSize.u.__bindgen_anon_1.LowPart == 0 {
        UnmapViewOfFile(pHeader as *mut c_void);
        return HRESULT_FROM_WIN32(GetLastError());
    }
    dwEntries = ((FileSize.QuadPart - core::mem::size_of::<INFO2_HEADER>() as u64)
        / core::mem::size_of::<DELETED_FILE_RECORD>() as u64) as DWORD;

    pRecord = (pHeader as *const u8)
        .add(core::mem::size_of::<INFO2_HEADER>() * 1 as usize)
        as *const DELETED_FILE_RECORD;

    i = 0;
    while i < dwEntries {
        if (*pRecord).dwRecordUniqueId == (*pDeletedFile).dwRecordUniqueId {
            // FIXME: The caller does not release pHeader!
            // Clear last entry in the file
            MoveMemory(
                (pRecord as *mut c_void).add(core::mem::size_of::<DELETED_FILE_RECORD>() * 1 as usize),
                pRecord as *const c_void,
                (dwEntries - i - 1) as usize * core::mem::size_of::<DELETED_FILE_RECORD>(),
            );
            pLast = (pRecord as *const u8).add(
                (dwEntries - i - 1) as usize
                    * core::mem::size_of::<DELETED_FILE_RECORD>(),
            ) as *const DELETED_FILE_RECORD;
            ZeroMemory(
                pLast as *mut c_void,
                core::mem::size_of::<DELETED_FILE_RECORD>(),
            );
            UnmapViewOfFile(pHeader as *mut c_void);

            // Resize file
            CloseHandle((*this).m_hInfoMapped);
            SetFilePointer(
                (*this).m_hInfo,
                -(core::mem::size_of::<DELETED_FILE_RECORD>() as i32),
                core::ptr::null_mut(),
                FILE_END,
            );
            SetEndOfFile((*this).m_hInfo);
            (*this).m_hInfoMapped = CreateFileMappingW(
                (*this).m_hInfo,
                core::ptr::null_mut(),
                PAGE_READWRITE | SEC_COMMIT,
                0,
                0,
                core::ptr::null_mut(),
            );
            if (*this).m_hInfoMapped.is_null() {
                return HResultFromWin32(GetLastError());
            }
            dwEntries -= 1;
            return if (dwEntries as i32) < 0 {
                INT_MAX as HRESULT
            } else {
                dwEntries as HRESULT
            };
        }
        pRecord = pRecord.add(1);
        i += 1;
    }

    UnmapViewOfFile(pHeader as *mut c_void);
    HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG)
}

pub unsafe fn RecycleBin5_OnClosing_body(this: *mut RecycleBin5, prbel: *mut IN IRecycleBinEnumList) -> HRESULT {
    (*this).m_EnumeratorCount -= 1;
    S_OK
}

pub unsafe fn RecycleBin5_Create_body(Folder: LPCWSTR, OwnerSid: PSID) -> HRESULT {
    let mut BufferName: *mut u16 = core::ptr::null_mut();
    let mut Separator: *mut u16;
    let mut FileName: *mut u16;
    let DesktopIniContents: *const u8 = b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n\0".as_ptr();
    let Info2Contents: [INFO2_HEADER; 1] = [INFO2_HEADER(5, 0, 0, 0x320, 0)];
    let mut BytesToWrite: u32;
    let mut BytesWritten: u32;
    let mut Needed: usize;
    let mut hFile: isize = INVALID_HANDLE_VALUE;
    let mut hr: HRESULT;

    'cleanup: loop {
        Needed = (wcslen(Folder) as usize
            + 1
            + max(wcslen(RECYCLE_BIN_FILE_NAME),
                  wcslen(CPP_COM_WSTR_A5DBFDEBD401.as_ptr()))
            as usize
            + 1)
            * 2;
        BufferName = HeapAlloc(GetProcessHeap(), 0, Needed) as *mut u16;
        if BufferName.is_null() {
            hr = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        wcscpy(BufferName, Folder);
        Separator = wcsstr(BufferName.add(3), CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        if !Separator.is_null() {
            *Separator = UNICODE_NULL;
        }
        if !CreateDirectoryW(BufferName, core::ptr::null_mut())
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);
        if !Separator.is_null() {
            *Separator = 0x005Cu16;
            if !CreateDirectoryW(BufferName, core::ptr::null_mut())
                && GetLastError() != ERROR_ALREADY_EXISTS
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'cleanup;
            }
        }

        // FIXME: Add ACL for OwnerSid if OwnerSid != NULL

        wcscat(BufferName, CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        FileName = BufferName.add(wcslen(BufferName));

        wcscpy(FileName, CPP_COM_WSTR_A5DBFDEBD401.as_ptr());
        hFile = CreateFileW(
            BufferName,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        BytesToWrite = strlen(DesktopIniContents) as u32;
        if !WriteFile(
            hFile,
            DesktopIniContents,
            BytesToWrite,
            &mut BytesWritten,
            core::ptr::null_mut(),
        ) {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        if BytesWritten != BytesToWrite {
            hr = E_FAIL;
            break 'cleanup;
        }
        CloseHandle(hFile);
        hFile = INVALID_HANDLE_VALUE;

        wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
        hFile = CreateFileW(
            BufferName,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        BytesToWrite = core::mem::size_of_val(&Info2Contents) as u32;
        let info2_ptr: *const u8 = &Info2Contents[0] as *const INFO2_HEADER as *const u8;
        if !WriteFile(hFile, info2_ptr, BytesToWrite, &mut BytesWritten, core::ptr::null_mut())
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup;
        }
        if BytesWritten == BytesToWrite {
            hr = S_OK;
        } else {
            hr = E_FAIL;
        }
        break 'cleanup;
    }

    HeapFree(GetProcessHeap(), 0, BufferName);
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    hr
}

pub unsafe fn RecycleBin5_ctor_body(this: *mut RecycleBin5) -> () {
    (*this).m_ref = 1;
    (*this).m_hInfo = core::ptr::null_mut();
    (*this).m_hInfoMapped = core::ptr::null_mut();
    (*this).m_EnumeratorCount = 0;
}

pub unsafe fn RecycleBin5_Init_body(this: *mut RecycleBin5, VolumePath: LPCWSTR) -> HRESULT {
    let mut hr: HRESULT;
    let mut tokenHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut TokenUserInfo: PTOKEN_USER = core::ptr::null_mut();
    let mut StringSid: LPWSTR = core::ptr::null_mut();
    let mut Needed: DWORD;
    let mut len: i32;

    let cleanup_hr: HRESULT = 'init: loop {
        RecycleBin5_cpp_com_assign_m_VolumePath(this, VolumePath);

        let mut FileSystemFlags: DWORD = 0;
        if GetVolumeInformationW(
            VolumePath,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut FileSystemFlags,
            core::ptr::null_mut(),
            0,
        ) == 0 {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'init hr;
        }

        let RecycleBinDirectory: LPCWSTR;
        if (FileSystemFlags & FILE_PERSISTENT_ACLS) == 0 {
            RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITHOUT_ACL;
        } else {
            RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITH_ACL;

            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut tokenHandle,
            ) == 0 {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'init hr;
            }
            if GetTokenInformation(
                tokenHandle,
                TokenUser,
                core::ptr::null_mut(),
                0,
                &mut Needed,
            ) != 0 {
                hr = E_FAIL;
                break 'init hr;
            }
            if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'init hr;
            }
            TokenUserInfo = HeapAlloc(
                GetProcessHeap(),
                0,
                Needed,
            );
            if TokenUserInfo.is_null() {
                hr = E_OUTOFMEMORY;
                break 'init hr;
            }
            if GetTokenInformation(
                tokenHandle,
                TokenUser,
                TokenUserInfo,
                Needed,
                &mut Needed,
            ) == 0 {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'init hr;
            }
            if ConvertSidToStringSidW((*TokenUserInfo).User.Sid, &mut StringSid) == 0 {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'init hr;
            }
        }

        RecycleBin5_cpp_com_assign_m_Folder(this, VolumePath);
        (*this).m_Folder += RecycleBinDirectory;
        if !StringSid.is_null() {
            (*this).m_Folder += 0x005Cu16;
            (*this).m_Folder += StringSid;
        }
        len = (*this).m_Folder.GetLength();
        (*this).m_Folder += CPP_COM_WSTR_B7FC02907B1F.as_ptr();
        (*this).m_Folder += RECYCLE_BIN_FILE_NAME;

        (*this).m_hInfo = CreateFileW(
            (*this).m_Folder.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );

        if (*this).m_hInfo == INVALID_HANDLE_VALUE
            && (GetLastError() == ERROR_PATH_NOT_FOUND
                || GetLastError() == ERROR_FILE_NOT_FOUND)
        {
            RecycleBin5_cpp_com_assign_m_Folder(this, (*this).m_Folder.Left(len).as_ptr());
            hr = RecycleBin5_Create_body(this, (*this).m_Folder.as_ptr(),
                if !TokenUserInfo.is_null() {
                    (*TokenUserInfo).User.Sid
                } else {
                    core::ptr::null_mut()
                },
            );
            (*this).m_Folder += CPP_COM_WSTR_B7FC02907B1F.as_ptr();
            (*this).m_Folder += RECYCLE_BIN_FILE_NAME;
            if (hr as i32) < 0 {
                break 'init hr;
            }
            (*this).m_hInfo = CreateFileW(
                (*this).m_Folder.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
        }

        if (*this).m_hInfo == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'init hr;
        }

        (*this).m_hInfoMapped = CreateFileMappingW(
            (*this).m_hInfo,
            core::ptr::null_mut(),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            0,
            core::ptr::null_mut(),
        );
        if (*this).m_hInfoMapped.is_null() {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'init hr;
        }

        RecycleBin5_cpp_com_assign_m_Folder(this, (*this).m_Folder.Left(len).as_ptr());
        hr = S_OK;
        break 'init hr;
    };

    if tokenHandle != INVALID_HANDLE_VALUE {
        CloseHandle(tokenHandle);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        TokenUserInfo,
    );
    if !StringSid.is_null() {
        LocalFree(StringSid);
    }
    cleanup_hr
}
