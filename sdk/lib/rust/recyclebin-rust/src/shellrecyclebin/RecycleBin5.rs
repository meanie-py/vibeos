use crate::prelude::*;
use crate::shellrecyclebin::recyclebin::EmptyRecycleBinW;
use crate::shellrecyclebin::recyclebin_v5_enumerator_free::RecycleBin5Enum_Constructor;
use crate::shellrecyclebin::recyclebin_support::wide_len;
// recyclebin_private.h:26-28 defines these three as `#define NAME L"..."` (WIDE strings), but
// the bindgen island runs in C mode and captured them as narrow `&[u8; N]` byte strings
// (types_amd64.rs:3937-3939) -- a nominal AND semantic split, not just layout. Explicit imports
// here shadow that glob-imported wrong copy with recyclebin_support's correct `LPCWSTR` ones
// (a plain `use crate::prelude::*;` would silently prefer the wrong byte-string constants).
use crate::shellrecyclebin::recyclebin_support::{
    RECYCLE_BIN_DIRECTORY_WITH_ACL, RECYCLE_BIN_DIRECTORY_WITHOUT_ACL, RECYCLE_BIN_FILE_NAME,
};
// IRecycleBin5: 11 slot(s) from the bindgen island, which agrees with recyclebin_v5.h.
// Not re-rendered here -- a second `IRecycleBin5` would be a distinct type with the same name.

// IRecycleBinEnumList: 6 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinEnumList` would be a distinct type with the same name.

// IRecycleBinFile: 14 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinFile` would be a distinct type with the same name.

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

unsafe extern "system" fn RecycleBin5_IRecycleBin5_QueryInterface(This: *mut IRecycleBin5, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
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
unsafe extern "system" fn RecycleBin5_IRecycleBin5_DeleteFile(This: *mut IRecycleBin5, szFileName: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_DeleteFile_body(this, szFileName)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_EmptyRecycleBin(This: *mut IRecycleBin5) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_EmptyRecycleBin_body(this)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_EnumObjects(This: *mut IRecycleBin5, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_EnumObjects_body(this, ppEnumList)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_GetDirectory(This: *mut IRecycleBin5, szPath: LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_GetDirectory_body(this, szPath)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_Delete(This: *mut IRecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_Delete_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_Restore(This: *mut IRecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_Restore_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_RemoveFromDatabase(This: *mut IRecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_RemoveFromDatabase_v1_body(this, pDeletedFileName, pDeletedFile)
}
unsafe extern "system" fn RecycleBin5_IRecycleBin5_OnClosing(This: *mut IRecycleBin5, prbel: *mut IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5, IRecycleBin5_iface)) as *mut RecycleBin5;
    RecycleBin5_OnClosing_body(this, prbel)
}

pub static RECYCLEBIN5_IRECYCLEBIN5_VTBL: IRecycleBin5Vtbl = IRecycleBin5Vtbl {
    QueryInterface: Some(RecycleBin5_IRecycleBin5_QueryInterface),
    AddRef: Some(RecycleBin5_IRecycleBin5_AddRef),
    Release: Some(RecycleBin5_IRecycleBin5_Release),
    DeleteFileW: Some(RecycleBin5_IRecycleBin5_DeleteFile),
    EmptyRecycleBinW: Some(RecycleBin5_IRecycleBin5_EmptyRecycleBin),
    EnumObjects: Some(RecycleBin5_IRecycleBin5_EnumObjects),
    GetDirectory: Some(RecycleBin5_IRecycleBin5_GetDirectory),
    Delete: Some(RecycleBin5_IRecycleBin5_Delete),
    Restore: Some(RecycleBin5_IRecycleBin5_Restore),
    RemoveFromDatabase: Some(RecycleBin5_IRecycleBin5_RemoveFromDatabase),
    OnClosing: Some(RecycleBin5_IRecycleBin5_OnClosing),
};

/// `new RecycleBin5` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBin5_new() -> *mut RecycleBin5 {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBin5>(),
    ) as *mut RecycleBin5;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBin5_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBIN5_IRECYCLEBIN5_VTBL) as *mut IRecycleBin5Vtbl;
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

// winnt.h `_ULARGE_INTEGER`: a union of a `{ DWORD LowPart; DWORD HighPart; }` struct (named
// `u` here -- winnt.h's OTHER anonymous-struct member and NONAMELESSUNION's `s` are unused by
// this TU) and `QuadPart`. The C++ reads/writes `FileSize.u.LowPart` directly -- one level of
// nesting, not two -- so this must NOT gain a bindgen-style `__bindgen_anon_1` layer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ULARGE_INTEGER_U {
    pub LowPart: DWORD,
    pub HighPart: DWORD,
}
#[repr(C)]
pub union ULARGE_INTEGER {
    pub u: ULARGE_INTEGER_U,
    pub QuadPart: ULONGLONG,
}
impl Default for ULARGE_INTEGER {
    fn default() -> Self {
        // SAFETY: all-zero is a valid bit pattern for every union member (two DWORDs / one
        // ULONGLONG), matching `ULARGE_INTEGER FileSize;` (C++ leaves it default-initialized,
        // but every call site sets `.u.LowPart`/`.u.HighPart` or `.QuadPart` before reading).
        unsafe { core::mem::zeroed() }
    }
}

/// `recyclebin_private.h:31` `#define ROUND_UP(N, S) (((N) + (S) - 1) / (S) * (S))`. A macro
/// there (double-evaluates `S`), but both call sites here pass a plain local, so a function is
/// faithful and avoids the double-eval footgun for any future caller.
#[inline]
fn ROUND_UP(n: DWORD, s: DWORD) -> DWORD {
    (n + s - 1) / s * s
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

// Deterministic from shldisp.h. `Folder`/`FolderVtbl` below are dead scaffolding -- no function
// body in this file calls through them -- but shldisp.h forward-declares `FolderItems` and
// `FolderItem` as opaque COM interfaces (their own vtables live in shell32's IDL closure, not
// this module's island), so the vtable's `*mut *mut FolderItems`/`FolderItem` slots need SOME
// nominal type to name. Opaque zero-sized structs, exactly as a C `struct FolderItems;` forward
// declaration would be: only ever used behind a pointer, never dereferenced here.
#[repr(C)]
pub struct FolderItems { _private: [u8; 0] }
#[repr(C)]
pub struct FolderItem { _private: [u8; 0] }

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
        // recyclebin_v5.cpp:42 `return ERROR_OUTOFMEMORY;` -- an `int` return smuggling a
        // Win32 error code, per the comment there ("Not one of the DE errors"). ERROR_OUTOFMEMORY
        // is DWORD; the function's declared return is `i32` (matching `int SHELL_SingleFileOperation`).
        return ERROR_OUTOFMEMORY as i32;
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
    let removable_attributes: u32 = FILE_ATTRIBUTE_READONLY;
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed::<WIN32_FIND_DATAW>();
    let mut h_search: HANDLE = INVALID_HANDLE_VALUE;
    let mut full_path: *mut u16 = core::ptr::null_mut();
    let mut ret: BOOL = FALSE as BOOL;

    'cleanup: {
        let file_attributes: u32 = GetFileAttributesW(FullName);
        if file_attributes == INVALID_FILE_ATTRIBUTES {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                ret = TRUE as BOOL;
            }
            break 'cleanup;
        }
        if (file_attributes & removable_attributes) != 0 {
            if SetFileAttributesW(FullName, file_attributes & !removable_attributes) == 0 {
                break 'cleanup;
            }
        }
        if (file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            // Directory branch
            let mut dw_length: usize = wcslen(FullName) as usize;
            let alloc_size: usize = (dw_length
                + 1 as usize
                + MAX_PATH as usize
                + 1 as usize)
                * core::mem::size_of::<u16>();

            full_path = HeapAlloc(
                GetProcessHeap(),
                0,
                alloc_size,
            ) as *mut u16;

            if full_path.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            wcscpy(full_path, FullName);

            // Append a backslash if not present
            if *full_path.add(dw_length - 1 as usize) != '\\' as u16 {
                *full_path.add(dw_length) = '\\' as u16;
                dw_length += 1;
            }
            let p_file_part: *mut u16 = full_path.add(dw_length);
            wcscpy(p_file_part, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());

            h_search = FindFirstFileW(full_path, (&mut find_data as *mut WIN32_FIND_DATAW).cast());
            if h_search == INVALID_HANDLE_VALUE {
                break 'cleanup;
            }

            loop {
                let skip = {
                    let c = &find_data.cFileName;
                    c[0 as usize] == '.' as u16
                        && (c[1 as usize] == 0u16
                            || (c[1 as usize] == '.' as u16
                                && c[2 as usize] == 0u16))
                };

                if !skip {
                    wcscpy(p_file_part, find_data.cFileName.as_ptr());
                    if IntDeleteRecursive_body(full_path) == 0 {
                        FindClose(h_search);
                        break 'cleanup;
                    }
                }

                if FindNextFileW(h_search, (&mut find_data as *mut WIN32_FIND_DATAW).cast()) == 0 {
                    break;
                }
            }

            FindClose(h_search);
            h_search = INVALID_HANDLE_VALUE;

            if GetLastError() != ERROR_NO_MORE_FILES {
                break 'cleanup;
            }
            if RemoveDirectoryW(FullName) == 0 {
                break 'cleanup;
            }
        } else {
            // File branch
            if DeleteFileW(FullName) == 0 {
                break 'cleanup;
            }
        }

        ret = TRUE as BOOL;
    }

    // cleanup
    if !full_path.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            full_path as *mut _,
        );
    }

    ret
}

pub unsafe fn RecycleBin5_GetDirectory_body(this: *mut RecycleBin5, szPath: LPWSTR) -> HRESULT {
    if (*this).m_Folder.GetAt(0) == 0 {
        return E_UNEXPECTED;
    }
    lstrcpynW(szPath, (*this).m_Folder.as_ptr(), MAX_PATH as i32);
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
    InterlockedIncrement(&mut (*this).m_ref) as u32
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
        RecycleBin5_delete(this);
    }
    refCount as ULONG
}

pub unsafe fn RecycleBin5_DeleteFile_body(this: *mut RecycleBin5, szFileName: LPCWSTR) -> HRESULT {
    let mut szFullName: LPWSTR = core::ptr::null_mut();
    let mut dwBufferLength: u32 = 0;
    let mut lpFilePart: LPWSTR = core::ptr::null_mut();
    let mut len: u32;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut pHeader: *mut INFO2_HEADER = core::ptr::null_mut();
    let mut pDeletedFile: *mut DELETED_FILE_RECORD;
    // Declared uninitialized in the C++ (`ULARGE_INTEGER FileSize;`), but every use here is
    // `&mut FileSize.u.LowPart`/`.HighPart` -- an FFI out-param write the Rust definite-init
    // checker cannot see as initializing the union. Zeroed to satisfy it; every read site
    // happens only after the corresponding write.
    let mut FileSize: ULARGE_INTEGER = ULARGE_INTEGER::default();
    let mut dwAttributes: u32;
    let mut dwEntries: u32;
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut DeletedFileName: CStringW = CStringW::new();
    let mut szUniqueId: [u16; 64 as usize] = [0; 64 as usize];

    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION);
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
                    CoTaskMemFree(szFullName.cast());
                    szFullName = core::ptr::null_mut();
                }
                hr = HResultFromWin32(GetLastError());
                break 'main_body;
            } else if len < dwBufferLength {
                break;
            }
            if !szFullName.is_null() {
                CoTaskMemFree(szFullName.cast());
                szFullName = core::ptr::null_mut();
            }
            dwBufferLength = len;
            szFullName = CoTaskMemAlloc(
                (dwBufferLength as usize) as SIZE_T * core::mem::size_of::<u16>() as SIZE_T,
            ) as LPWSTR;
            if szFullName.is_null() {
                hr = HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
                break 'main_body;
            }
        }

        dwAttributes = GetFileAttributesW(szFullName);
        if dwAttributes == INVALID_FILE_ATTRIBUTES {
            CoTaskMemFree(szFullName.cast());
            szFullName = core::ptr::null_mut();
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }

        if dwBufferLength < 2
            || *szFullName.add(1 as usize) != 0x003A_u16
        {
            CoTaskMemFree(szFullName.cast());
            szFullName = core::ptr::null_mut();
            hr = HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
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

        FileSize.u.LowPart =
            GetFileSize((*this).m_hInfo, &mut FileSize.u.HighPart);
        if FileSize.u.LowPart < core::mem::size_of::<INFO2_HEADER>() as u32 {
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

        FileSize.u.LowPart =
            GetFileSize(hFile, &mut FileSize.u.HighPart);
        if FileSize.u.LowPart == INVALID_FILE_SIZE
            && GetLastError() != NO_ERROR
        {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        if FileSize.u.HighPart != 0 {
            hr = HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
            break 'main_body;
        }
        (*pHeader).dwTotalLogicalSize += FileSize.u.LowPart;

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
            (tolower(*szFullName.add(0 as usize) as i32) - 0x61i32) as DWORD;
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
        // DELETED_FILE_RECORD is #[repr(C, packed)]; `&mut (*pDeletedFile).DeletionTime` would
        // be an unaligned reference (E0793). Write through a local FILETIME and store it back
        // with `write_unaligned`, which is exactly what packed-field writes need.
        let mut deletion_time: FILETIME = core::mem::zeroed();
        if SystemTimeToFileTime(
            &SystemTime,
            &mut deletion_time,
        ) == 0
        {
            hr = HResultFromWin32(GetLastError());
            break 'main_body;
        }
        core::ptr::addr_of_mut!((*pDeletedFile).DeletionTime).write_unaligned(deletion_time);
        (*pDeletedFile).dwPhysicalFileSize =
            ROUND_UP(FileSize.u.LowPart, ClusterSize);

        // DELETED_FILE_RECORD is #[repr(C, packed)] (types_amd64.rs:16611) -- `.as_ptr()` /
        // `.as_mut_ptr()` on a field array would borrow `&(*pDeletedFile).FileNameW`, an
        // unaligned reference (AGENTS.md / repair brief rule on packed fields). `addr_of!`
        // takes the address without ever forming a reference.
        wcscpy(
            core::ptr::addr_of_mut!((*pDeletedFile).FileNameW).cast::<u16>(),
            szFullName,
        );
        if WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of!((*pDeletedFile).FileNameW).cast::<u16>(),
            -1,
            core::ptr::addr_of_mut!((*pDeletedFile).FileNameA) as LPSTR,
            MAX_PATH as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
            SetLastError(ERROR_INVALID_NAME);
            break 'main_body;
        }

        wfd.dwFileAttributes = dwAttributes;
        wfd.nFileSizeLow = FileSize.u.LowPart;
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
                RecycledFullPath: DeletedFileName.as_ptr(),
            };
            CRecycleBin_NotifyRecycled(szFullName, (&wfd as *const WIN32_FIND_DATAW).cast(), &ident);
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
        CoTaskMemFree(szFullName.cast());
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
        let hr: HRESULT = ((*(*prbel).lpVtbl).Next.unwrap_unchecked())(
            prbel,
            1,
            &mut prbf,
            ::core::ptr::null_mut(),
        );
        ((*(*prbel).lpVtbl).Release.unwrap_unchecked())(prbel);
        if hr == S_FALSE {
            return S_OK;
        }
        let hr: HRESULT = ((*(*prbf).lpVtbl).Delete.unwrap_unchecked())(prbf);
        ((*(*prbf).lpVtbl).Release.unwrap_unchecked())(prbf);
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

    // `this` (RecycleBin5*) implicitly upcasts to IRecycleBin5* in the C++ call
    // (`RecycleBin5Enum_Constructor(this, ...)`, recyclebin_v5.cpp:437) -- valid because
    // `IRecycleBin5_iface` is RecycleBin5's first field, so the address is identical.
    let mut hr = RecycleBin5Enum_Constructor(
        core::ptr::addr_of_mut!((*this).IRecycleBin5_iface),
        (*this).m_hInfo,
        (*this).m_hInfoMapped,
        folder_ptr,
        &mut pUnk,
    );

    if hr < 0 {
        return hr;
    }

    let mut prbel: *mut IRecycleBinEnumList = core::ptr::null_mut();
    hr = ((*(*pUnk).lpVtbl).QueryInterface.unwrap_unchecked())(
        pUnk,
        (&IID_IRecycleBinEnumList as *const GUID).cast(),
        &mut prbel as *mut *mut _ as *mut *mut core::ffi::c_void,
    );

    if hr >= 0 {
        (*this).m_EnumeratorCount += 1;
        *ppEnumList = prbel;
    }

    ((*(*pUnk).lpVtbl).Release.unwrap_unchecked())(pUnk);
    hr
}

pub unsafe fn RecycleBin5_Delete_v1_body(this: *mut RecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
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

pub unsafe fn RecycleBin5_Restore_v1_body(this: *mut RecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
    let mut res = SHELL_SingleFileOperation_body(
        core::ptr::null_mut(),
        FO_MOVE,
        pDeletedFileName,
        core::ptr::addr_of!((*pDeletedFile).FileNameW).cast::<u16>(),
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

pub unsafe fn RecycleBin5_RemoveFromDatabase_v1_body(this: *mut RecycleBin5, pDeletedFileName: LPCWSTR, pDeletedFile: *mut DELETED_FILE_RECORD) -> HRESULT {
    let mut FileSize: ULARGE_INTEGER = ULARGE_INTEGER::default();
    let pHeader: PINFO2_HEADER;
    let mut pRecord: *const DELETED_FILE_RECORD;
    let pLast: *const DELETED_FILE_RECORD;
    let mut dwEntries: DWORD;
    let mut i: DWORD;

    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION);
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

    FileSize.u.LowPart = GetFileSize((*this).m_hInfo, &mut FileSize.u.HighPart);
    if FileSize.u.LowPart == 0 {
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
            // recyclebin_v5.cpp:719 `MoveMemory(pRecord, pRecord + 1, ...)` -- dest is the
            // record being removed, src is the record AFTER it (shift the tail down by one
            // slot). The prior translation had dest/src swapped (`pRecord.add(1)` as dest,
            // `pRecord` as src), which corrupts the INFO2 database by shifting the wrong way
            // instead of deleting the entry. `MoveMemory` is also a MACRO (prelude.rs), not a
            // function -- called here as `MoveMemory!`.
            MoveMemory!(
                pRecord as *mut c_void,
                (pRecord as *const u8).add(core::mem::size_of::<DELETED_FILE_RECORD>() * 1 as usize),
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
    HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND)
}

pub unsafe fn RecycleBin5_OnClosing_body(this: *mut RecycleBin5, prbel: *mut IRecycleBinEnumList) -> HRESULT {
    (*this).m_EnumeratorCount -= 1;
    S_OK
}

pub unsafe fn RecycleBin5_Create_body(Folder: LPCWSTR, OwnerSid: PSID) -> HRESULT {
    let mut hr: HRESULT;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut buffer_name: LPWSTR = core::ptr::null_mut();

    // ---------- Main logic with goto-cleanup simulation ----------
    'cleanup_main: loop {
        // Allocate buffer for path manipulation
        let needed: usize = (wcslen(Folder)
            + 1
            + max(
                wcslen(RECYCLE_BIN_FILE_NAME),
                wcslen(CPP_COM_WSTR_A5DBFDEBD401.as_ptr()),
            )
            + 1)
            * core::mem::size_of::<u16>();

        buffer_name = HeapAlloc(
            GetProcessHeap(),
            0,
            needed,
        ) as LPWSTR;
        if buffer_name.is_null() {
            // recyclebin_v5.cpp:557 `hr = ERROR_NOT_ENOUGH_MEMORY; goto cleanup;` -- the C++
            // itself stores the raw Win32 error into `hr` here (unlike every other failure
            // path in this function, which wraps via HRESULT_FROM_WIN32). Preserved as-is.
            hr = ERROR_NOT_ENOUGH_MEMORY as HRESULT;
            break 'cleanup_main;
        }

        // Copy Folder path into buffer
        wcscpy(buffer_name, Folder);

        // Find a possible separator substring in the buffer (offset given by FACT_0012)
        let mut separator: LPWSTR =
            wcsstr(
                buffer_name.wrapping_add(3 as usize),
                CPP_COM_WSTR_B7FC02907B1F.as_ptr(),
            );
        if !separator.is_null() {
            *separator = UNICODE_NULL;
        }

        // Create main directory (without separator)
        if CreateDirectoryW(buffer_name, core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup_main;
        }

        // Set attributes on the directory
        SetFileAttributesW(
            buffer_name,
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
        );

        // If we had a separator (e.g. backslash), create the subdirectory too
        if !separator.is_null() {
            *separator = 0x005Cu16;
            if CreateDirectoryW(buffer_name, core::ptr::null_mut()) == 0
                && GetLastError() != ERROR_ALREADY_EXISTS
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'cleanup_main;
            }
        }

        // Owner SID handling (FIXME – stub)
        if !OwnerSid.is_null() {
            // FIXME: ACL setup not yet implemented
        }

        // Append a trailing separator (fact string) to build final file paths
        wcscat(buffer_name, CPP_COM_WSTR_B7FC02907B1F.as_ptr());

        // Pointer to the end of the current buffer (where we will concat file names)
        let file_name: LPWSTR =
            buffer_name.wrapping_add(wcslen(buffer_name) as usize);

        // ---------- Create desktop.ini ----------
        wcscpy(file_name, CPP_COM_WSTR_A5DBFDEBD401.as_ptr());

        hFile = CreateFileW(
            buffer_name,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup_main;
        }

        // recyclebin_v5.cpp:552 `LPCSTR DesktopIniContents = "[.ShellClassInfo]\r\nCLSID=
        // {645FF040-5081-101B-9F08-00AA002F954E}\r\n";` -- a distinct ASCII literal. The prior
        // translation dropped it entirely and reused `RECYCLE_BIN_FILE_NAME` ("INFO2") as the
        // desktop.ini payload instead, which would write the wrong 5 bytes into every
        // newly-created recycle bin's desktop.ini. Restored verbatim against the C++.
        const DESKTOP_INI_CONTENTS: &[u8] =
            b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n";
        let bytes_to_write: u32 = DESKTOP_INI_CONTENTS.len() as u32;
        let mut bytes_written: u32 = 0;
        if WriteFile(
            hFile,
            DESKTOP_INI_CONTENTS.as_ptr().cast(),
            bytes_to_write,
            &mut bytes_written,
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup_main;
        }
        if bytes_written != bytes_to_write {
            hr = E_FAIL;
            break 'cleanup_main;
        }
        CloseHandle(hFile);
        hFile = INVALID_HANDLE_VALUE;

        // ---------- Create empty INFO2 file ----------
        wcscpy(file_name, RECYCLE_BIN_FILE_NAME);

        hFile = CreateFileW(
            buffer_name,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup_main;
        }

        let info2_contents: [u32; 5] = [
            5 as u32,
            0 as u32,
            0 as u32,
            0x320 as u32,
            0 as u32,
        ];
        let bytes_to_write = core::mem::size_of_val(&info2_contents) as u32;
        bytes_written = 0;
        if WriteFile(
            hFile,
            info2_contents.as_ptr().cast(),
            bytes_to_write,
            &mut bytes_written,
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
            break 'cleanup_main;
        }
        if bytes_written == bytes_to_write {
            hr = S_OK;
        } else {
            hr = E_FAIL;
        }
        break 'cleanup_main;
    }

    // ---------- Cleanup block ----------
    HeapFree(
        GetProcessHeap(),
        0,
        buffer_name as *mut core::ffi::c_void,
    );
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
    let mut Needed: DWORD = 0;
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
            ) as PTOKEN_USER;
            if TokenUserInfo.is_null() {
                hr = E_OUTOFMEMORY;
                break 'init hr;
            }
            if GetTokenInformation(
                tokenHandle,
                TokenUser,
                TokenUserInfo.cast(),
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
        (*this).m_Folder.append_str(RecycleBinDirectory);
        if !StringSid.is_null() {
            (*this).m_Folder.append_char(0x005Cu16);
            (*this).m_Folder.append_str(StringSid);
        }
        len = (*this).m_Folder.GetLength();
        (*this).m_Folder.append_str(CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        (*this).m_Folder.append_str(RECYCLE_BIN_FILE_NAME);

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
            // recyclebin_v5.cpp:752: `RecycleBin5_Create` is the free function above, not a
            // method -- it takes no `this`.
            hr = RecycleBin5_Create_body((*this).m_Folder.as_ptr(),
                if !TokenUserInfo.is_null() {
                    (*TokenUserInfo).User.Sid
                } else {
                    core::ptr::null_mut()
                },
            );
            (*this).m_Folder.append_str(CPP_COM_WSTR_B7FC02907B1F.as_ptr());
            (*this).m_Folder.append_str(RECYCLE_BIN_FILE_NAME);
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
        LocalFree(StringSid.cast());
    }
    cleanup_hr
}
