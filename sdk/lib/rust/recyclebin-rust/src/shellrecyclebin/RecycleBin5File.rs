use crate::prelude::*;
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

// Deterministic COM scaffold for RecycleBin5File; method bodies are generated separately.
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
pub struct RecycleBin5File {
    pub IRecycleBinFile_iface: IRecycleBinFile,
    pub m_ref: LONG,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_deletedFile: DELETED_FILE_RECORD,
    pub m_FullName: LPWSTR,
}

unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_QueryInterface(This: *mut IRecycleBinFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_QueryInterface_body(this, riid, ppvObject)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_AddRef(This: *mut IRecycleBinFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_AddRef_body(this)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_Release(This: *mut IRecycleBinFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_Release_body(this)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_IsEqualIdentity(This: *mut IRecycleBinFile, pFI: *const RECYCLEBINFILEIDENTITY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_IsEqualIdentity_body(this, pFI)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetInfo(This: *mut IRecycleBinFile, pInfo: PDELETED_FILE_INFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetInfo_body(this, pInfo)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetLastModificationTime(This: *mut IRecycleBinFile, pLastModificationTime: *mut FILETIME) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetLastModificationTime_body(this, pLastModificationTime)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetDeletionTime(This: *mut IRecycleBinFile, pDeletionTime: *mut FILETIME) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetDeletionTime_body(this, pDeletionTime)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetFileSize(This: *mut IRecycleBinFile, pFileSize: *mut ULARGE_INTEGER) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetFileSize_body(this, pFileSize)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetPhysicalFileSize(This: *mut IRecycleBinFile, pPhysicalFileSize: *mut ULARGE_INTEGER) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetPhysicalFileSize_body(this, pPhysicalFileSize)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetAttributes(This: *mut IRecycleBinFile, pAttributes: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetAttributes_body(this, pAttributes)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_GetFileName(This: *mut IRecycleBinFile, BufferSize: SIZE_T, Buffer: LPWSTR, RequiredSize: *mut SIZE_T) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_GetFileName_body(this, BufferSize, Buffer, RequiredSize)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_Delete(This: *mut IRecycleBinFile) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_Delete_body(this)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_Restore(This: *mut IRecycleBinFile) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_Restore_body(this)
}
unsafe extern "system" fn RecycleBin5File_IRecycleBinFile_RemoveFromDatabase(This: *mut IRecycleBinFile) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5File, IRecycleBinFile_iface)) as *mut RecycleBin5File;
    RecycleBin5File_RemoveFromDatabase_body(this)
}

pub static RECYCLEBIN5FILE_IRECYCLEBINFILE_VTBL: IRecycleBinFileVtbl = IRecycleBinFileVtbl {
    QueryInterface: RecycleBin5File_IRecycleBinFile_QueryInterface,
    AddRef: RecycleBin5File_IRecycleBinFile_AddRef,
    Release: RecycleBin5File_IRecycleBinFile_Release,
    IsEqualIdentity: RecycleBin5File_IRecycleBinFile_IsEqualIdentity,
    GetInfo: RecycleBin5File_IRecycleBinFile_GetInfo,
    GetLastModificationTime: RecycleBin5File_IRecycleBinFile_GetLastModificationTime,
    GetDeletionTime: RecycleBin5File_IRecycleBinFile_GetDeletionTime,
    GetFileSize: RecycleBin5File_IRecycleBinFile_GetFileSize,
    GetPhysicalFileSize: RecycleBin5File_IRecycleBinFile_GetPhysicalFileSize,
    GetAttributes: RecycleBin5File_IRecycleBinFile_GetAttributes,
    GetFileName: RecycleBin5File_IRecycleBinFile_GetFileName,
    Delete: RecycleBin5File_IRecycleBinFile_Delete,
    Restore: RecycleBin5File_IRecycleBinFile_Restore,
    RemoveFromDatabase: RecycleBin5File_IRecycleBinFile_RemoveFromDatabase,
};

/// `new RecycleBin5File` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBin5File_new() -> *mut RecycleBin5File {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBin5File>(),
    ) as *mut RecycleBin5File;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBinFile_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBIN5FILE_IRECYCLEBINFILE_VTBL);
    RecycleBin5File_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `RecycleBin5File_new`; freeing through any other allocator is a heap bug.
pub unsafe fn RecycleBin5File_delete(this: *mut RecycleBin5File) {
    if this.is_null() { return; }
    RecycleBin5File_dtor_body(this);
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

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;

pub const FILE_SHARE_DELETE: DWORD = 0x00000004;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const GENERIC_READ: DWORD = 0x80000000;

pub const INVALID_FILE_SIZE: DWORD = 0xFFFFFFFF;

pub const OPEN_EXISTING: DWORD = 3;

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
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
    static IID_IRecycleBinEnumList: GUID;
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
pub static CPP_COM_WSTR_F3C6A8F2AC47: [WCHAR; 4] = [0x0025, 0x006C, 0x0075, 0x0000];

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

pub unsafe fn RecycleBin5File_IsEqualIdentity_body(this: *mut RecycleBin5File, pFI: *const RECYCLEBINFILEIDENTITY) -> HRESULT {
    let self_value = RECYCLEBINFILEIDENTITY {
        DeletionTime: (*this).m_deletedFile.DeletionTime,
        FullName: (*this).m_FullName,
    };
    let is_equal = RecycleBinGeneric_IsEqualFileIdentity(pFI, &self_value);
    if is_equal != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn RecycleBin5File_QueryInterface_body(this: *mut RecycleBin5File, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if cpp_com_iid_equal(riid, &IID_IUnknown as *const _)
        || cpp_com_iid_equal(riid, &IID_IRecycleBinFile as *const _)
    {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    RecycleBin5File_AddRef_body(this);
    S_OK
}

pub unsafe fn RecycleBin5File_AddRef_body(this: *mut RecycleBin5File) -> ULONG {
    InterlockedIncrement(&raw mut (*this).m_ref)
}

pub unsafe fn RecycleBin5File_dtor_body(this: *mut RecycleBin5File) -> () {
    (*(*(*this).m_recycleBin).lpVtbl).Release((*this).m_recycleBin);
    SHFree(((*this).m_FullName) as LPVOID);
}

pub unsafe fn RecycleBin5File_Release_body(this: *mut RecycleBin5File) -> ULONG {
    let ref_count = InterlockedDecrement(&mut (*this).m_ref as *mut LONG) as ULONG;
    if ref_count == 0 {
        // delete this: run destructor and free allocation
        drop(Box::from_raw(this));
    }
    ref_count
}

pub unsafe fn RecycleBin5File_GetInfo_body(this: *mut RecycleBin5File, pInfo: PDELETED_FILE_INFO) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut uli: ULARGE_INTEGER = ULARGE_INTEGER::default();

    let last_mod_hr = RecycleBin5File_GetLastModificationTime_body(this, &mut (*pInfo).LastModification);
    if last_mod_hr < 0 {
        ZeroMemory(&mut (*pInfo).LastModification as *mut FILETIME as *mut core::ffi::c_void, core::mem::size_of::<FILETIME>());
    }

    (*pInfo).DeletionTime = (*this).m_deletedFile.DeletionTime;

    C_ASSERT(core::mem::size_of_val(&(*pInfo).FileSize) <= core::mem::size_of::<u32>());

    let file_size_hr = RecycleBin5File_GetFileSize_body(this, &mut uli);
    (*pInfo).FileSize = if file_size_hr < 0 {
        INVALID_FILE_SIZE
    } else {
        uli.__bindgen_anon_1.LowPart
    };

    hr = RecycleBin5File_GetAttributes_body(this, &mut (*pInfo).Attributes);
    if hr < 0 {
        (*pInfo).Attributes = 0;
    }

    InitializeRecycleBinStringRef(&mut (*pInfo).OriginalFullPath, (*this).m_deletedFile.FileNameW);
    InitializeRecycleBinStringRef(&mut (*pInfo).RecycledFullPath, (*this).m_FullName);

    hr
}

pub unsafe fn RecycleBin5File_GetLastModificationTime_body(this: *mut RecycleBin5File, pLastModificationTime: *mut FILETIME) -> HRESULT {
    let dwAttributes: DWORD = GetFileAttributesW((*this).m_FullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HResultFromWin32(GetLastError());
    }
    let mut hFile: HANDLE;
    hFile = CreateFileW(
        (*this).m_FullName,
        GENERIC_READ,
        FILE_SHARE_READ
            | if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                FILE_SHARE_WRITE | FILE_SHARE_DELETE
            } else {
                0
            },
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
        return HResultFromWin32(GetLastError());
    }
    let hr: HRESULT;
    if GetFileTime(
        hFile,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pLastModificationTime,
    ) != 0
    {
        hr = S_OK;
    } else {
        hr = HResultFromWin32(GetLastError());
    }
    CloseHandle(hFile);
    hr
}

pub unsafe fn RecycleBin5File_GetDeletionTime_body(this: *mut RecycleBin5File, pDeletionTime: *mut FILETIME) -> HRESULT {
    *pDeletionTime = (*this).m_deletedFile.DeletionTime;
    S_OK
}

pub unsafe fn RecycleBin5File_GetFileSize_body(this: *mut RecycleBin5File, pFileSize: *mut ULARGE_INTEGER) -> HRESULT {
    let dwAttributes = GetFileAttributesW((*this).m_FullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        (*pFileSize).QuadPart = 0;
        return S_OK;
    }
    let hFile = CreateFileW(
        (*this).m_FullName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    (*pFileSize).u.__bindgen_anon_1.LowPart = GetFileSize(this,
        hFile,
        core::ptr::addr_of_mut!((*pFileSize).u.__bindgen_anon_1.HighPart),
    );
    let hr;
    if (*pFileSize).u.__bindgen_anon_1.LowPart != INVALID_FILE_SIZE {
        hr = S_OK;
    } else {
        hr = HRESULT_FROM_WIN32(GetLastError());
    }
    CloseHandle(hFile);
    hr
}

pub unsafe fn RecycleBin5File_GetPhysicalFileSize_body(this: *mut RecycleBin5File, pPhysicalFileSize: *mut ULARGE_INTEGER) -> HRESULT {
    (*pPhysicalFileSize).u.__bindgen_anon_1.HighPart = 0;
    (*pPhysicalFileSize).u.__bindgen_anon_1.LowPart = (*this).m_deletedFile.dwPhysicalFileSize;
    S_OK
}

pub unsafe fn RecycleBin5File_GetAttributes_body(this: *mut RecycleBin5File, pAttributes: *mut DWORD) -> HRESULT {
    let dwAttributes: u32 = GetFileAttributesW((*this).m_FullName.as_ptr());
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HResultFromWin32(GetLastError());
    }
    *pAttributes = dwAttributes;
    S_OK
}

pub unsafe fn RecycleBin5File_GetFileName_body(this: *mut RecycleBin5File, BufferSize: SIZE_T, Buffer: LPWSTR, RequiredSize: *mut SIZE_T) -> HRESULT {
    let file_name_ptr = (*this).m_deletedFile.FileNameW;
    let len = wcslen(file_name_ptr) + 1;
    let dw_required = len.wrapping_mul(core::mem::size_of::<u16>());

    if !RequiredSize.is_null() {
        *RequiredSize = dw_required;
    }

    if BufferSize == 0 && Buffer.is_null() {
        return S_OK;
    }

    if BufferSize < dw_required {
        return E_OUTOFMEMORY;
    }

    CopyMemory(
        Buffer as *mut core::ffi::c_void,
        file_name_ptr as *const core::ffi::c_void,
        dw_required,
    );

    S_OK
}

pub unsafe fn RecycleBin5File_Delete_body(this: *mut RecycleBin5File) -> HRESULT {
    ((*(*(*this).m_recycleBin).lpVtbl).Delete)(
        (*this).m_recycleBin,
        (*this).m_FullName,
        &(*this).m_deletedFile,
    )
}

pub unsafe fn RecycleBin5File_Restore_body(this: *mut RecycleBin5File) -> HRESULT {
    ((*(*(*this).m_recycleBin).lpVtbl).Restore)(
        (*this).m_recycleBin,
        (*this).m_FullName,
        &raw const (*this).m_deletedFile,
    )
}

pub unsafe fn RecycleBin5File_RemoveFromDatabase_body(this: *mut RecycleBin5File) -> HRESULT {
    let p_recycleBin = (*this).m_recycleBin;
    ((*(*p_recycleBin).lpVtbl).RemoveFromDatabase)(
        p_recycleBin,
        (*this).m_FullName,
        &raw const (*this).m_deletedFile as *mut DELETED_FILE_RECORD,
    )
}

pub unsafe fn RecycleBin5File_ctor_body(this: *mut RecycleBin5File) -> () {
    (*this).m_ref = 1;
    (*this).m_recycleBin = core::ptr::null_mut();
    (*this).m_FullName = core::ptr::null_mut();
    let size = core::mem::size_of_val(&(*this).m_deletedFile);
    ZeroMemory((&raw mut (*this).m_deletedFile) as *mut _, size);
}

pub unsafe fn RecycleBin5File_Init_body(this: *mut RecycleBin5File, prb: *mut IRecycleBin5, Folder: LPCWSTR, pDeletedFile: PDELETED_FILE_RECORD) -> HRESULT {
    // 1. set recycle bin pointer and addref
    (*this).m_recycleBin = prb;
    (*(*prb).lpVtbl).AddRef(prb);

    // 2. copy the deleted file record
    let deleted: DELETED_FILE_RECORD = *pDeletedFile;
    (*this).m_deletedFile = deleted;

    // 3. build unique id string
    let mut szUniqueId: [WCHAR; 32] = [0u16; 32];
    StringCchPrintfW(
        szUniqueId.as_mut_ptr(),
        szUniqueId.len(),
        CPP_COM_WSTR_F3C6A8F2AC47.as_ptr(),
        deleted.dwRecordUniqueId,
    );

    // 4. build full path with CStringW
    let mut strFullName = CStringW::from_ptr(Folder);
    strFullName.append_str(CPP_COM_WSTR_8CC2401ABCDF.as_ptr());
    strFullName.append_char((0x0061u16 + deleted.dwDriveNumber as u16) as u16);
    strFullName.append_str(szUniqueId.as_ptr());
    let ext_ptr = PathFindExtensionW(deleted.FileNameW);
    strFullName.append_str(ext_ptr);

    // 5. check if the file exists (attribute check)
    if GetFileAttributesW(strFullName.as_ptr()) == INVALID_FILE_ATTRIBUTES {
        return E_FAIL;
    }

    // 6. duplicate the full path into m_FullName
    SHStrDup(strFullName.as_ptr(), &mut (*this).m_FullName)
}
