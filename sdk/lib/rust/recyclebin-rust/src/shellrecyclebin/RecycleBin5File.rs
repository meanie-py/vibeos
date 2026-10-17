use crate::prelude::*;
// One declaration per external symbol: types.rs.
use crate::types::CreateFileW;
use crate::shellrecyclebin::recyclebin_generic_free::RecycleBinGeneric_IsEqualFileIdentity;
// IRecycleBinFile: 14 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinFile` would be a distinct type with the same name.

// Deterministic COM scaffold for RecycleBin5File; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: SIZE_T) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size() as SIZE_T)
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
    QueryInterface: Some(RecycleBin5File_IRecycleBinFile_QueryInterface),
    AddRef: Some(RecycleBin5File_IRecycleBinFile_AddRef),
    Release: Some(RecycleBin5File_IRecycleBinFile_Release),
    IsEqualIdentity: Some(RecycleBin5File_IRecycleBinFile_IsEqualIdentity),
    GetInfo: Some(RecycleBin5File_IRecycleBinFile_GetInfo),
    GetLastModificationTime: Some(RecycleBin5File_IRecycleBinFile_GetLastModificationTime),
    GetDeletionTime: Some(RecycleBin5File_IRecycleBinFile_GetDeletionTime),
    GetFileSize: Some(RecycleBin5File_IRecycleBinFile_GetFileSize),
    GetPhysicalFileSize: Some(RecycleBin5File_IRecycleBinFile_GetPhysicalFileSize),
    GetAttributes: Some(RecycleBin5File_IRecycleBinFile_GetAttributes),
    GetFileName: Some(RecycleBin5File_IRecycleBinFile_GetFileName),
    Delete: Some(RecycleBin5File_IRecycleBinFile_Delete),
    Restore: Some(RecycleBin5File_IRecycleBinFile_Restore),
    RemoveFromDatabase: Some(RecycleBin5File_IRecycleBinFile_RemoveFromDatabase),
};

/// `new RecycleBin5File` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBin5File_new() -> *mut RecycleBin5File {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBin5File>(),
    ) as *mut RecycleBin5File;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBinFile_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBIN5FILE_IRECYCLEBINFILE_VTBL).cast_mut();
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

// NOTE: no local `ULARGE_INTEGER` here -- the real bindgen union (ntdll_sys::nt::_ULARGE_INTEGER,
// glob-imported via `crate::prelude::*` -> `crate::types::*` -> `types_arch::*`) is what
// IRecycleBinFileVtbl::GetFileSize/GetPhysicalFileSize actually declare. A same-named local
// struct here would silently SHADOW it and make every body a distinct, incompatible nominal
// type from the vtable's fn-pointer signature (this is what broke the build originally).

// Deterministic from shldisp.h
// FolderItems/FolderItem are forward-declared COM interfaces in shldisp.h with no member of
// this file ever calling through them (`Folder` here is dead deterministic scaffold -- the
// `Folder` *parameter* of RecycleBin5File_Init_body below is an unrelated LPCWSTR, just a
// same-named value). Opaque stubs match a C forward declaration used only as a pointer target.
#[repr(C)]
pub struct FolderItems { _unused: [u8; 0] }
#[repr(C)]
pub struct FolderItem { _unused: [u8; 0] }

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
        // recyclebin_v5_enumerator.cpp:33 -- aggregate-inits { DeletionTime, m_FullName },
        // and the struct's 2nd member is `RecycledFullPath` (there is no `FullName` field).
        DeletionTime: (*this).m_deletedFile.DeletionTime,
        RecycledFullPath: (*this).m_FullName,
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
    // InterlockedIncrement returns LONG (i32); recyclebin_v5_enumerator.cpp:78 returns it
    // straight out of an `ULONG AddRef()` -- same reinterpretation C performs implicitly.
    InterlockedIncrement(&raw mut (*this).m_ref) as ULONG
}

pub unsafe fn RecycleBin5File_dtor_body(this: *mut RecycleBin5File) -> () {
    // recyclebin_v5_enumerator.cpp:84 -- m_recycleBin->Release(); vtable slots are Option<fn>.
    ((*(*(*this).m_recycleBin).lpVtbl).Release.unwrap_unchecked())((*this).m_recycleBin);
    SHFree(((*this).m_FullName) as LPVOID);
}

pub unsafe fn RecycleBin5File_Release_body(this: *mut RecycleBin5File) -> ULONG {
    let ref_count = InterlockedDecrement(&mut (*this).m_ref as *mut LONG) as ULONG;
    if ref_count == 0 {
        // delete this: run destructor and free allocation
        RecycleBin5File_delete(this);
    }
    ref_count
}

pub unsafe fn RecycleBin5File_GetInfo_body(this: *mut RecycleBin5File, pInfo: PDELETED_FILE_INFO) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    // recyclebin_v5_enumerator.cpp:100 -- `ULARGE_INTEGER uli;` is left uninitialized in C++
    // (only read after GetFileSize succeeds and fills it); zeroing is a safe superset.
    let mut uli: ULARGE_INTEGER = core::mem::zeroed();

    let last_mod_hr = RecycleBin5File_GetLastModificationTime_body(this, &mut (*pInfo).LastModification);
    if last_mod_hr < 0 {
        ZeroMemory(&mut (*pInfo).LastModification as *mut FILETIME as *mut core::ffi::c_void, core::mem::size_of::<FILETIME>());
    }

    (*pInfo).DeletionTime = (*this).m_deletedFile.DeletionTime;

    // recyclebin_v5_enumerator.cpp:104 -- C_ASSERT is a compile-time-only macro (no runtime
    // code); pInfo->FileSize is RECYCLEBINFILESIZETYPE (== UINT), so this is the Rust
    // equivalent static check, not a lowered/no-op body.
    const _: () = assert!(core::mem::size_of::<RECYCLEBINFILESIZETYPE>() <= core::mem::size_of::<u32>());

    let file_size_hr = RecycleBin5File_GetFileSize_body(this, &mut uli);
    (*pInfo).FileSize = if file_size_hr < 0 {
        INVALID_FILE_SIZE
    } else {
        uli.u.LowPart
    };

    hr = RecycleBin5File_GetAttributes_body(this, &mut (*pInfo).Attributes);
    if hr < 0 {
        (*pInfo).Attributes = 0;
    }

    InitializeRecycleBinStringRef(&mut (*pInfo).OriginalFullPath, core::ptr::addr_of!((*this).m_deletedFile.FileNameW).cast::<u16>());
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
    // recyclebin_v5_enumerator.cpp:165 -- `::GetFileSize(hFile, &pFileSize->u.HighPart)`, the
    // Win32 function (2 args: HANDLE, PDWORD); no `this`. The real ULARGE_INTEGER union names
    // this member `u` directly (winnt.h uses `u`/`DUMMYSTRUCTNAME`, not `__bindgen_anon_1`).
    (*pFileSize).u.LowPart = GetFileSize(
        hFile,
        core::ptr::addr_of_mut!((*pFileSize).u.HighPart),
    );
    let hr;
    if (*pFileSize).u.LowPart != INVALID_FILE_SIZE {
        hr = S_OK;
    } else {
        hr = HRESULT_FROM_WIN32(GetLastError());
    }
    CloseHandle(hFile);
    hr
}

pub unsafe fn RecycleBin5File_GetPhysicalFileSize_body(this: *mut RecycleBin5File, pPhysicalFileSize: *mut ULARGE_INTEGER) -> HRESULT {
    (*pPhysicalFileSize).u.HighPart = 0;
    (*pPhysicalFileSize).u.LowPart = (*this).m_deletedFile.dwPhysicalFileSize;
    S_OK
}

pub unsafe fn RecycleBin5File_GetAttributes_body(this: *mut RecycleBin5File, pAttributes: *mut DWORD) -> HRESULT {
    // recyclebin_v5_enumerator.cpp:191 -- `GetFileAttributesW(m_FullName)`; m_FullName is
    // already LPWSTR, which coerces to LPCWSTR on its own -- `.as_ptr()` doesn't exist on a
    // raw pointer and isn't needed.
    let dwAttributes: u32 = GetFileAttributesW((*this).m_FullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HResultFromWin32(GetLastError());
    }
    *pAttributes = dwAttributes;
    S_OK
}

pub unsafe fn RecycleBin5File_GetFileName_body(this: *mut RecycleBin5File, BufferSize: SIZE_T, Buffer: LPWSTR, RequiredSize: *mut SIZE_T) -> HRESULT {
    // recyclebin_v5_enumerator.cpp:205: `dwRequired = (DWORD)(wcslen(m_deletedFile.FileNameW) + 1) * sizeof(WCHAR);`
    // FileNameW is a [WCHAR;260] field of a #[repr(C, packed)] struct: copy it out first
    // (a value copy is not a reference, so no unaligned-field UB), THEN take .as_ptr() on the
    // now-ordinary local array. dwRequired is DWORD in the C++, and the (DWORD) cast happens
    // BEFORE the *sizeof(WCHAR) multiply -- preserved here, not folded into a wider type.
    let file_name_ptr = (*this).m_deletedFile.FileNameW;
    let len = wcslen(file_name_ptr.as_ptr());
    let dw_required: DWORD = ((len + 1) as DWORD).wrapping_mul(core::mem::size_of::<u16>() as DWORD);

    if !RequiredSize.is_null() {
        *RequiredSize = dw_required as SIZE_T;
    }

    if BufferSize == 0 && Buffer.is_null() {
        return S_OK;
    }

    if BufferSize < dw_required as SIZE_T {
        return E_OUTOFMEMORY;
    }

    CopyMemory(
        Buffer as *mut core::ffi::c_void,
        file_name_ptr.as_ptr() as *const core::ffi::c_void,
        dw_required,
    );

    S_OK
}

pub unsafe fn RecycleBin5File_Delete_body(this: *mut RecycleBin5File) -> HRESULT {
    // recyclebin_v5_enumerator.cpp:221 -- vtable slots are Option<fn>, need .unwrap_unchecked(); the
    // callee's declared 3rd param is `*mut DELETED_FILE_RECORD` (non-const), matching `&m_deletedFile`.
    ((*(*(*this).m_recycleBin).lpVtbl).Delete.unwrap_unchecked())(
        (*this).m_recycleBin,
        (*this).m_FullName,
        &mut (*this).m_deletedFile,
    )
}

pub unsafe fn RecycleBin5File_Restore_body(this: *mut RecycleBin5File) -> HRESULT {
    ((*(*(*this).m_recycleBin).lpVtbl).Restore.unwrap_unchecked())(
        (*this).m_recycleBin,
        (*this).m_FullName,
        &mut (*this).m_deletedFile,
    )
}

pub unsafe fn RecycleBin5File_RemoveFromDatabase_body(this: *mut RecycleBin5File) -> HRESULT {
    let p_recycleBin = (*this).m_recycleBin;
    ((*(*p_recycleBin).lpVtbl).RemoveFromDatabase.unwrap_unchecked())(
        p_recycleBin,
        (*this).m_FullName,
        &mut (*this).m_deletedFile,
    )
}

pub unsafe fn RecycleBin5File_ctor_body(this: *mut RecycleBin5File) -> () {
    (*this).m_ref = 1;
    (*this).m_recycleBin = core::ptr::null_mut();
    (*this).m_FullName = core::ptr::null_mut();
    let size = core::mem::size_of_val(&(*this).m_deletedFile);
    ZeroMemory((&raw mut (*this).m_deletedFile) as *mut _, size);
}

// StringCchPrintfW is strsafe_rust::StringCchPrintfW!, which calls msvcrt's variadic
// `_snwprintf` directly. It used to bind to shell32's C trampoline (rust_strsafe_shim.c).

pub unsafe fn RecycleBin5File_Init_body(this: *mut RecycleBin5File, prb: *mut IRecycleBin5, Folder: LPCWSTR, pDeletedFile: PDELETED_FILE_RECORD) -> HRESULT {
    // 1. set recycle bin pointer and addref
    (*this).m_recycleBin = prb;
    ((*(*prb).lpVtbl).AddRef.unwrap_unchecked())(prb);

    // 2. copy the deleted file record
    let deleted: DELETED_FILE_RECORD = *pDeletedFile;
    (*this).m_deletedFile = deleted;

    // 3. build unique id string
    let mut szUniqueId: [WCHAR; 32] = [0u16; 32];
    strsafe_rust::StringCchPrintfW!(
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
    // deleted.FileNameW is [WCHAR;260] on a #[repr(C, packed)] type; addr_of! avoids forming
    // a reference to a possibly-unaligned field (recyclebin_v5_enumerator.cpp:261).
    let ext_ptr = PathFindExtensionW(core::ptr::addr_of!(deleted.FileNameW).cast::<u16>());
    strFullName.append_str(ext_ptr);

    // 5. check if the file exists (attribute check)
    if GetFileAttributesW(strFullName.as_ptr()) == INVALID_FILE_ATTRIBUTES {
        return E_FAIL;
    }

    // 6. duplicate the full path into m_FullName
    SHStrDup(strFullName.as_ptr(), &mut (*this).m_FullName)
}
