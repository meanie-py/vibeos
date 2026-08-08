use crate::prelude::*;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_Init_body;
use crate::shellrecyclebin::RecycleBin5File::RecycleBin5File_dtor_body;
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

// Deterministic COM scaffold for RecycleBin5Enum; method bodies are generated separately.
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
pub struct RecycleBin5Enum {
    pub IRecycleBinEnumList_iface: IRecycleBinEnumList,
    pub m_ref: LONG,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_hInfo: HANDLE,
    pub m_pInfo: *mut INFO2_HEADER,
    pub m_dwCurrent: DWORD,
    pub m_pszPrefix: LPWSTR,
}

unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_QueryInterface(This: *mut IRecycleBinEnumList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_QueryInterface_body(this, riid, ppvObject)
}
unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_AddRef(This: *mut IRecycleBinEnumList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_AddRef_body(this)
}
unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_Release(This: *mut IRecycleBinEnumList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_Release_body(this)
}
unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_Next(This: *mut IRecycleBinEnumList, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_Skip(This: *mut IRecycleBinEnumList, celt: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_Skip_body(this, celt)
}
unsafe extern "system" fn RecycleBin5Enum_IRecycleBinEnumList_Reset(This: *mut IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBin5Enum, IRecycleBinEnumList_iface)) as *mut RecycleBin5Enum;
    RecycleBin5Enum_Reset_body(this)
}

pub static RECYCLEBIN5ENUM_IRECYCLEBINENUMLIST_VTBL: IRecycleBinEnumListVtbl = IRecycleBinEnumListVtbl {
    QueryInterface: RecycleBin5Enum_IRecycleBinEnumList_QueryInterface,
    AddRef: RecycleBin5Enum_IRecycleBinEnumList_AddRef,
    Release: RecycleBin5Enum_IRecycleBinEnumList_Release,
    Next: RecycleBin5Enum_IRecycleBinEnumList_Next,
    Skip: RecycleBin5Enum_IRecycleBinEnumList_Skip,
    Reset: RecycleBin5Enum_IRecycleBinEnumList_Reset,
};

/// `new RecycleBin5Enum` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBin5Enum_new() -> *mut RecycleBin5Enum {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBin5Enum>(),
    ) as *mut RecycleBin5Enum;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBinEnumList_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBIN5ENUM_IRECYCLEBINENUMLIST_VTBL);
    RecycleBin5Enum_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `RecycleBin5Enum_new`; freeing through any other allocator is a heap bug.
pub unsafe fn RecycleBin5Enum_delete(this: *mut RecycleBin5Enum) {
    if this.is_null() { return; }
    RecycleBin5Enum_dtor_body(this);
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

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

pub unsafe fn RecycleBin5File_Constructor_body(prb: *mut IRecycleBin5, Folder: LPCWSTR, pDeletedFile: PDELETED_FILE_RECORD, ppFile: *mut *mut IRecycleBinFile) -> HRESULT {
    if ppFile.is_null() {
        return E_POINTER;
    }
    *ppFile = core::ptr::null_mut();

    let pThis: *mut RecycleBin5File = RecycleBin5File();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr = RecycleBin5File_Init_body(pThis, prb, Folder, pDeletedFile);
    if hr < 0 {
        RecycleBin5File_dtor_body(pThis);
        free(pThis as *mut u8);
        return hr;
    }

    *ppFile = pThis as *mut IRecycleBinFile;
    S_OK
}

pub unsafe fn RecycleBin5Enum_QueryInterface_body(this: *mut RecycleBin5Enum, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if cpp_com_iid_equal(riid, &IID_IUnknown) || cpp_com_iid_equal(riid, &IID_IRecycleBinEnumList) {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    RecycleBin5Enum_AddRef_body(this);
    return S_OK;
}

pub unsafe fn RecycleBin5Enum_AddRef_body(this: *mut RecycleBin5Enum) -> ULONG {
    InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref))
}

pub unsafe fn RecycleBin5Enum_dtor_body(this: *mut RecycleBin5Enum) -> () {
    let recycle_bin = (*this).m_recycleBin;
    ((*(*recycle_bin).lpVtbl).OnClosing)(recycle_bin, this);
    UnmapViewOfFile((*this).m_pInfo);
    ((*(*recycle_bin).lpVtbl).Release)(recycle_bin);
    SHFree(((*this).m_pszPrefix) as LPVOID);
}

pub unsafe fn RecycleBin5Enum_Release_body(this: *mut RecycleBin5Enum) -> ULONG {
    let refCount = InterlockedDecrement(&raw mut (*this).m_ref);
    if refCount == 0 {
        RecycleBin5Enum_dtor_body(this);
    free(this as *mut core::ffi::c_void);
    }
    refCount
}

pub unsafe fn RecycleBin5Enum_Next_body(this: *mut RecycleBin5Enum, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT {
    if rgelt.is_null() {
        return E_POINTER;
    }
    if pceltFetched.is_null() && celt > 1 {
        return E_INVALIDARG;
    }

    let mut FileSize: ULARGE_INTEGER = core::mem::zeroed();
    FileSize.u.__bindgen_anon_1.LowPart = GetFileSize((*this).m_hInfo, &mut FileSize.u.__bindgen_anon_1.HighPart);
    if FileSize.u.__bindgen_anon_1.LowPart == 0 {
        return HResultFromWin32(GetLastError());
    }

    let header_size = core::mem::size_of::<INFO2_HEADER>() as u64;
    let record_size = core::mem::size_of::<DELETED_FILE_RECORD>() as u64;
    let dwEntries = ((FileSize.QuadPart - header_size) / record_size) as u32;

    let mut iEntry = (*this).m_dwCurrent;
    let mut fetched: u32 = 0;
    let pInfo_ptr = (*this).m_pInfo;
    let mut pDeletedFile = (pInfo_ptr.add(1) as *mut DELETED_FILE_RECORD).add(iEntry as usize);

    while iEntry < dwEntries && fetched < celt {
        let hr = RecycleBin5File_Constructor_body(
            (*this).m_recycleBin,
            (*this).m_pszPrefix,
            pDeletedFile,
            &mut *(*rgelt).add(fetched as usize),
        );
        if hr >= 0 {
            fetched += 1;
        }
        iEntry += 1;
        pDeletedFile = pDeletedFile.add(1);
    }

    (*this).m_dwCurrent = iEntry;
    if !pceltFetched.is_null() {
        *pceltFetched = fetched;
    }
    if fetched == celt {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn RecycleBin5Enum_Skip_body(this: *mut RecycleBin5Enum, celt: DWORD) -> HRESULT {
    (*this).m_dwCurrent += celt;
    S_OK
}

pub unsafe fn RecycleBin5Enum_Reset_body(this: *mut RecycleBin5Enum) -> HRESULT {
    (*this).m_dwCurrent = 0;
    S_OK
}

pub unsafe fn RecycleBin5Enum_ctor_body(this: *mut RecycleBin5Enum) -> () {
    (*this).m_ref = 1;
    (*this).m_recycleBin = core::ptr::null_mut();
    (*this).m_hInfo = core::ptr::null_mut();
    (*this).m_pInfo = core::ptr::null_mut();
    (*this).m_dwCurrent = 0;
    (*this).m_pszPrefix = core::ptr::null_mut();
}

pub unsafe fn RecycleBin5Enum_Init_body(this: *mut RecycleBin5Enum, prb: *mut IRecycleBin5, hInfo: HANDLE, hInfoMapped: HANDLE, pszPrefix: LPCWSTR) -> HRESULT {
    // m_recycleBin = prb; AddRef
    (*this).m_recycleBin = prb;
    (*((*(*((*this).m_recycleBin))).lpVtbl).AddRef)((*this).m_recycleBin);

    // Duplicate the prefix string
    let hr: HRESULT = SHStrDup(pszPrefix, &mut (*this).m_pszPrefix);
    if hr < 0 {
        return hr;
    }

    // Store the info file handle
    (*this).m_hInfo = hInfo;

    // Map view of the info file
    let p_info: *mut INFO2_HEADER = MapViewOfFile(
        hInfoMapped,
        FILE_MAP_READ,
        0,
        0,
        0,
    ) as *mut INFO2_HEADER;
    (*this).m_pInfo = p_info;

    if p_info.is_null() {
        return HResultFromWin32(GetLastError());
    }

    // Validate header version and record size
    if (*p_info).dwVersion != 5
        || (*p_info).dwRecordSize != core::mem::size_of::<DELETED_FILE_RECORD>()
    {
        return E_FAIL;
    }

    S_OK
}
