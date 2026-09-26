use crate::prelude::*;
use crate::reg_folder::CreateRegItem_2_body;
use crate::reg_folder::CreateRegItem_body;
use crate::shell32::shlfolder::SHELL32_CompareDetails;
use crate::shell32::shlfolder::SHELL_IncludeItemInFolderEnum;
use crate::drv_def_ext::ShellObjectCreator;
use crate::wine::pidl::_ILGetGUIDPointer;
use crate::wine::pidl::_ILIsSpecialFolder;
use crate::drv_def_ext::size_t;
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

// Deterministic COM scaffold for CRegFolderEnum; method bodies are generated separately.
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
pub struct CRegFolderEnum {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pInfo: *const REGFOLDERINFO,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
    pub m_SHCTF: SHCONTF,
}

unsafe extern "system" fn CRegFolderEnum_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_AddRef(this)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_Release(this)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_Skip_body(this, celt)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_Reset_body(this)
}
unsafe extern "system" fn CRegFolderEnum_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolderEnum, IEnumIDList_iface)) as *mut CRegFolderEnum;
    CRegFolderEnum_Clone_body(this, ppenum)
}

pub static CREGFOLDERENUM_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CRegFolderEnum_IEnumIDList_QueryInterface,
    AddRef: CRegFolderEnum_IEnumIDList_AddRef,
    Release: CRegFolderEnum_IEnumIDList_Release,
    Next: CRegFolderEnum_IEnumIDList_Next,
    Skip: CRegFolderEnum_IEnumIDList_Skip,
    Reset: CRegFolderEnum_IEnumIDList_Reset,
    Clone: CRegFolderEnum_IEnumIDList_Clone,
};

unsafe fn CRegFolderEnum_QueryInterface(
    this: *mut CRegFolderEnum, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRegFolderEnum_AddRef(this);
    S_OK
}

unsafe fn CRegFolderEnum_AddRef(this: *mut CRegFolderEnum) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRegFolderEnum_Release(this: *mut CRegFolderEnum) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRegFolderEnum_dtor_body(this);
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
pub unsafe fn CRegFolderEnum_CreateInstance(
    pInfo: *const REGFOLDERINFO,
    pSF: *mut IShellFolder,
    dwFlags: DWORD,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRegFolderEnum>()) as *mut CRegFolderEnum;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRegFolderEnum {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CREGFOLDERENUM_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pInfo: core::mem::zeroed(),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
        m_SHCTF: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRegFolderEnum_ctor_body(this);
    let mut hr = CRegFolderEnum_Initialize_body(this, pInfo, pSF, dwFlags);
    if hr >= 0 {
        hr = CRegFolderEnum_QueryInterface(this, riid, ppv);
    }
    CRegFolderEnum_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDI_SHELL_DESKTOP: UINT = 35;

pub const IDI_SHELL_DOCUMENT: UINT = 1;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SHCNE_UPDATEITEM: DWORD = 0x00002000;

pub const SHCNF_IDLIST: DWORD = 0x0000;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHDID_ROOT_REGITEM: DWORD = 1;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const STGM_CREATE: DWORD = 0x00001000;

// shtypes.idl STRRET_WSTR = 0; local per-TU redefinition, same pattern as
// folders/CRegFolder.rs / desktop_folder.rs / drives_folder.rs etc.
pub const STRRET_WSTR: UINT = 0;

// winerror.h SHCONTF_INCLUDEHIDDEN / SHCONTF_INCLUDESUPERHIDDEN; not part of the bindgen
// island under these names (only the tagSHCONTF_* variants are), so redefined locally like
// every other TU in this crate that touches SHELL_CreateFolderEnumItemAttributeQuery.
pub const SHCONTF_INCLUDEHIDDEN: DWORD = 0x80;
pub const SHCONTF_INCLUDESUPERHIDDEN: DWORD = 0x10000;

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

#[repr(C)]
pub struct SHCOLUMNID {
    pub fmtid: GUID,
    pub pid: DWORD,
}

#[repr(C, packed)]
pub struct SHITEMID {
    pub cb: WORD,
    pub abID: [BYTE; 1],
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDefaultExtractIconInit {
    pub lpVtbl: *const IDefaultExtractIconInitVtbl,
}
#[repr(C)]
pub struct IDefaultExtractIconInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub SetDefaultIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetFlags: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, uFlags: UINT) -> HRESULT,
    pub SetKey: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, hkey: HKEY) -> HRESULT,
    pub SetNormalIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetOpenIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetShortcutIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
}
extern "C" {
    pub static IID_IDefaultExtractIconInit: IID;
}

extern "system" {
    fn RegCloseKey(key: HKEY) -> LONG;
    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
}

extern "C" {
    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;
    // _ILIsSpecialFolder and SHELL32_CompareDetails are already brought into scope by the
    // `use` imports at the top of this file (crate::wine::pidl / crate::shell32::shlfolder);
    // redeclaring them here as extern fns duplicated those names (E0255).
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

extern "C" {
    static CLSID_MyComputer: GUID;
    static CLSID_MyDocuments: GUID;
    static CLSID_NetworkConnections: GUID;
    static CLSID_Printers: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IShellFolder: GUID;
    static IID_IShellFolder2: GUID;
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

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    // STRRET's union is `__bindgen_anon_1` here (bindgen names it, not `u`); same pattern
    // as folders/CDrivesFolderEnum.rs:595.
    (*out).uType = STRRET_WSTR; *(*out).__bindgen_anon_1.pOleStr.as_mut() = buffer; S_OK
}

// shfldr.h:85-92 — `static inline`, so every TU that includes the header (this one included)
// carries its own copy; not an extern symbol to import. Deterministic bitwise combination.
unsafe fn SHELL_CreateFolderEnumItemAttributeQuery(Flags: SHCONTF, ForRegItem: BOOL) -> SFGAOF {
    let mut query: SFGAOF = SFGAO_FOLDER | (if ForRegItem != 0 { SFGAO_NONENUMERATED } else { 0 });
    if Flags & SHCONTF_INCLUDEHIDDEN == 0 {
        query |= SFGAO_HIDDEN;
    }
    if Flags & SHCONTF_INCLUDESUPERHIDDEN == 0 {
        query |= SFGAO_HIDDEN | SFGAO_SYSTEM;
    }
    query
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH as usize];
    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 { return E_FAIL; }
    cpp_com_SHSetStrRetW(out, buffer.as_ptr())
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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static CPP_COM_WSTR_A3FDF2CDA91F: [WCHAR; 15] = [0x0053, 0x006F, 0x0072, 0x0074, 0x004F, 0x0072, 0x0064, 0x0065, 0x0072, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_B23CFF6BE226: [WCHAR; 64] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0025, 0x0073, 0x005C, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x0070, 0x0061, 0x0063, 0x0065, 0x0000];

use core::ffi::c_void;

pub unsafe fn CRegFolderEnum_ctor_body(this: *mut CRegFolderEnum) -> () {}

pub unsafe fn CRegFolderEnum_dtor_body(this: *mut CRegFolderEnum) -> () {
    CRegFolderEnum_DeleteList_body(this);
}

pub unsafe fn SHELL_QueryCLSIDValue_body(clsid: REFCLSID, SubKey: LPCWSTR, Value: LPCWSTR, pData: PVOID, pSize: PDWORD) -> DWORD {
    let cchGuid: UINT = CHARS_IN_GUID - 1;
    let cchClsidSlash: UINT = CPP_COM_WSTR_6E342AB1A0F1.len() as UINT - 1;
    let mut Path = [0u16; 200 as usize];

    wcscpy(Path.as_mut_ptr(), CPP_COM_WSTR_6E342AB1A0F1.as_ptr());
    StringFromGUID2(
        // REFCLSID here is `*const com_sys::IID`; StringFromGUID2's REFGUID island param is
        // `*const ntdll_sys::nt::_GUID` -- same nominal-type-split as elsewhere in this file.
        clsid.cast(),
        Path.as_mut_ptr().add(6 as usize),
        CHARS_IN_GUID as INT
    );

    if !SubKey.is_null() {
        let offset = (cchClsidSlash + cchGuid) as usize;
        *Path.as_mut_ptr().add(offset) = 0x005Cu16;
        wcscpy(
            Path.as_mut_ptr().add(offset + 1 as usize),
            SubKey
        );
    }

    RegGetValueW(
        HKEY_CLASSES_ROOT,
        Path.as_mut_ptr(),
        Value,
        RRF_RT_ANY,
        core::ptr::null_mut(),
        pData.cast(),
        pSize
    ) as DWORD
}

pub unsafe fn CRegFolderEnum_AddToEnumList_body(this: *mut CRegFolderEnum, pidl: LPITEMIDLIST) -> BOOL {
    let pNew: *mut ENUMLIST;

    if pidl.is_null() {
        return FALSE;
    }

    pNew = SHAlloc(::core::mem::size_of::<ENUMLIST>() as SIZE_T) as *mut ENUMLIST;
    if !pNew.is_null() {
        (*pNew).pNext = ::core::ptr::null_mut();
        (*pNew).pidl = pidl;

        if (*this).mpFirst.is_null() {
            (*this).mpFirst = pNew;
            (*this).mpCurrent = pNew;
        }

        if !(*this).mpLast.is_null() {
            (*(*this).mpLast).pNext = pNew;
        }

        (*this).mpLast = pNew;

        return TRUE;
    }
    return FALSE;
}

pub unsafe fn GetRegItemCLSIDOffset_body(r#type: PIDLTYPE) -> UINT {
    // PT_CONTROLS_NEWREGITEM comes through as a bindgen `u32` const while PIDLTYPE = BYTE;
    // widen rather than narrow, matching CRegFolder.rs:649's identical fix for the sibling
    // free function of the same name in this TU.
    if u32::from(r#type) == PT_CONTROLS_NEWREGITEM {
        14
    } else {
        4
    }
}

pub unsafe fn GetRegItemOrder_body(clsid: REFCLSID) -> BYTE {
    let mut dwOrder_storage = core::mem::MaybeUninit::<DWORD>::uninit();
    let mut cb: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    if SHELL_QueryCLSIDValue_body(
        clsid,
        core::ptr::null(),
        CPP_COM_WSTR_A3FDF2CDA91F.as_ptr(),
        dwOrder_storage.as_mut_ptr() as PVOID,
        &mut cb as PDWORD,
    ) != 0
        || cb != core::mem::size_of::<DWORD>() as DWORD
    {
        return REGITEMORDER_DEFAULT as BYTE;
    }
    dwOrder_storage.assume_init() as BYTE
}

pub unsafe fn CRegFolderEnum_DeleteList_body(this: *mut CRegFolderEnum) -> BOOL {
    while !((*this).mpFirst).is_null() {
        let pDelete = (*this).mpFirst;
        (*this).mpFirst = (*pDelete).pNext;
        SHFree((*pDelete).pidl);
        SHFree(pDelete);
    }
    (*this).mpFirst = core::ptr::null_mut();
    (*this).mpLast = core::ptr::null_mut();
    (*this).mpCurrent = core::ptr::null_mut();
    return TRUE;
}

pub unsafe fn CRegFolderEnum_InitializeFolderInfo_body(this: *mut CRegFolderEnum, pInfo: *const REGFOLDERINFO) -> () {
    (*this).m_pInfo = pInfo;
}

pub unsafe fn CRegFolderEnum_AppendItemsFromEnumerator_body(this: *mut CRegFolderEnum, pEnum: *mut IEnumIDList) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;

    if pEnum.is_null() {
        return E_INVALIDARG;
    }

    let _ = unsafe { ((*(*pEnum).lpVtbl).Reset)(pEnum) };

    while (S_OK == unsafe { ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched) }) && dwFetched != 0 {
        let _ = unsafe { CRegFolderEnum_AddToEnumList_body(this, pidl) };
    }

    return S_OK;
}

pub unsafe fn CRegFolderEnum_IsRegItem_body(this: *mut CRegFolderEnum, pidl: LPCITEMIDLIST) -> *mut CLSID {
    if !pidl.is_null() && (*pidl).mkid.cb as usize >= core::mem::size_of::<WORD>() + 1 + 1 + core::mem::size_of::<GUID>()
    {
        if *((*pidl).mkid.abID.as_ptr().add(0 as usize)) == (*(*this).m_pInfo).PidlType
        {
            // `SIZE_T(pidl)` in the C is a reinterpret-cast (SIZE_T -> pointer), not a call;
            // SIZE_T is a type alias here, not a function. Same fix as CRegFolder.rs:741-743
            // for the twin IsRegItem in this TU.
            return (pidl as *mut u8).add(CRegFolderEnum_GetCLSIDOffset_body(this) as usize) as *mut CLSID;
        }
        if u32::from(*((*pidl).mkid.abID.as_ptr().add(0 as usize))) == PT_CONTROLS_OLDREGITEM
        {
            return (pidl as *mut u8).add(GetRegItemCLSIDOffset_body(PT_CONTROLS_OLDREGITEM as PIDLTYPE) as usize) as *mut CLSID;
        }
    }
    // _ILGetGUIDPointer returns `*mut shellcom::IID`; this fn works in `*const IID` — same
    // cast as CRegFolder.rs:756-757.
    let pIID: *const IID = _ILGetGUIDPointer(pidl) as *const IID;
    if !pIID.is_null()
    {
        // FIXME: Remove this when all folders have been fixed
        return pIID as *mut CLSID;
    }
    return core::ptr::null_mut();
}

pub unsafe fn CRegFolderEnum_Next_body(this: *mut CRegFolderEnum, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let mut i: ULONG;
    let mut hr: HRESULT = S_OK;

    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }

    *rgelt = core::ptr::null_mut();

    if celt > 1 && pceltFetched.is_null() {
        return E_INVALIDARG;
    }

    if celt > 0 && (*this).mpCurrent.is_null() {
        return S_FALSE;
    }

    i = 0;
    while i < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }

        let temp: PITEMID_CHILD = ILClone((*(*this).mpCurrent).pidl);
        if temp.is_null() {
            hr = if i != 0 { S_FALSE } else { E_OUTOFMEMORY };
            break;
        }

        *rgelt.add(i as usize) = temp;
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        i += 1;
    }

    if !pceltFetched.is_null() {
        *pceltFetched = i;
    }

    return hr;
}

pub unsafe fn CRegFolderEnum_CreateItem_body(this: *mut CRegFolderEnum, i: size_t) -> LPITEMIDLIST {
    let item: &REQUIREDREGITEM = CRegFolderEnum_GetAt_body(this, i);
    // item.clsid is already `*const CLSID` (REQUIREDREGITEM::clsid); `&item.clsid` would take
    // the address of that pointer field itself (a pointer-to-pointer), not the pointer's
    // value it stores -- a real bug, not just a type mismatch, since it compiled (coerced to
    // `*const *const CLSID` then bit-cast) but read the wrong memory. Fixed to `.cast()`,
    // matching the twin CreateItem in CRegFolder.rs:769.
    CreateRegItem_body(CRegFolderEnum_GetPidlType_body(this), item.clsid.cast(), item.Order as i32)
}

pub unsafe fn CRegFolderEnum_GetRegItemOrder_body(this: *mut CRegFolderEnum, pidl: LPCITEMIDLIST) -> WORD {
    let p_clsid = CRegFolderEnum_IsRegItem_body(this, pidl);
    if p_clsid.is_null() {
        0xffff as WORD
    } else {
        GetRegItemOrder_body(p_clsid as REFCLSID) as WORD
    }
}

pub unsafe fn CRegFolderEnum_GetParsingPath_body(this: *mut CRegFolderEnum) -> LPCWSTR {
    (*(*this).m_pInfo).pszParsingPath
}

pub unsafe fn CRegFolderEnum_GetCLSIDOffset_body(this: *mut CRegFolderEnum) -> UINT {
    GetRegItemCLSIDOffset_body((*(*this).m_pInfo).PidlType)
}

pub unsafe fn CRegFolderEnum_GetPidlType_body(this: *mut CRegFolderEnum) -> PIDLTYPE {
    (*(*this).m_pInfo).PidlType
}

pub unsafe fn CRegFolderEnum_GetRequiredItemsCount_body(this: *mut CRegFolderEnum) -> UINT {
    (*(*this).m_pInfo).Count as UINT
}

// m_pInfo always points at a `static` REGFOLDERINFO (e.g. g_RegFolderInfo in
// CDesktopFolder.rs/CDrivesFolder.rs/CControlPanelFolder.rs), so the borrow this returns is
// valid for the program's lifetime, matching the C++ `const REQUIREDREGITEM&` return.
pub unsafe fn CRegFolderEnum_GetAt_body(this: *mut CRegFolderEnum, i: size_t) -> &'static mut REQUIREDREGITEM {
    &mut *(((*(*this).m_pInfo).Items).cast::<REQUIREDREGITEM>().add(i) as *mut REQUIREDREGITEM)
}

pub unsafe fn CRegFolderEnum_Skip_body(this: *mut CRegFolderEnum, celt: ULONG) -> HRESULT {
    let mut dwIndex: u32 = 0;
    let mut hr: HRESULT = S_OK;

    while dwIndex < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        dwIndex += 1;
    }

    hr
}

pub unsafe fn CRegFolderEnum_Reset_body(this: *mut CRegFolderEnum) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    S_OK
}

pub unsafe fn CRegFolderEnum_Clone_body(this: *mut CRegFolderEnum, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRegFolderEnum_GetPidlClsid_body(this: *mut CRegFolderEnum, pidl: PCUITEMID_CHILD) -> *mut CLSID {
    return CRegFolderEnum_IsRegItem_body(this, pidl);
}

pub unsafe fn CRegFolderEnum_HasItemWithCLSID_body(this: *mut CRegFolderEnum, pidl: LPCITEMIDLIST) -> BOOL {
    if !pidl.is_null() && (*pidl).mkid.cb >= (core::mem::size_of::<CLSID>() + core::mem::size_of::<u16>()) as u16 {
        1
    } else {
        0
    }
}

pub unsafe fn CRegFolderEnum_Initialize_body(this: *mut CRegFolderEnum, pInfo: *const REGFOLDERINFO, pSF: *mut IShellFolder, dwFlags: DWORD) -> HRESULT {
    CRegFolderEnum_InitializeFolderInfo_body(this, pInfo);
    (*this).m_SHCTF = dwFlags as SHCONTF;
    if dwFlags & SHCONTF_FOLDERS == 0 {
        return S_OK;
    }

    let mut KeyName = [0u16; MAX_PATH as usize];
    let count = KeyName.len();
    let hr: HRESULT = strsafe_rust::StringCchPrintfW!(
        KeyName.as_mut_ptr(),
        count,
        CPP_COM_WSTR_B23CFF6BE226.as_ptr(),
        (*pInfo).pszEnumKeyName,
    );
    if ((hr) < 0) {
        return hr;
    }

    let query: SFGAOF = SHELL_CreateFolderEnumItemAttributeQuery((*this).m_SHCTF, TRUE);
    let mut i = 0 as usize;
    while i < CRegFolderEnum_GetRequiredItemsCount_body(this) as usize {
        let pidl: LPITEMIDLIST = CRegFolderEnum_CreateItem_body(this, i);
        if !pidl.is_null() && SHELL_IncludeItemInFolderEnum(pSF, pidl, query, (*this).m_SHCTF) != 0 {
            CRegFolderEnum_AddToEnumList_body(this, pidl);
        } else {
            ILFree(pidl);
        }
        i = i + 1;
    }

    CRegFolderEnum_AddItemsFromKey_body(this, pSF, HKEY_LOCAL_MACHINE, KeyName.as_ptr());
    CRegFolderEnum_AddItemsFromKey_body(this, pSF, HKEY_CURRENT_USER, KeyName.as_ptr());
    return S_OK;
}

pub unsafe fn CRegFolderEnum_AddItemsFromKey_body(this: *mut CRegFolderEnum, pSF: *mut IShellFolder, hkey_root: HKEY, szRepPath: LPCWSTR) -> HRESULT {
    unsafe {
        let mut name = [0u16; MAX_PATH as usize];
        let mut hkey: HKEY = core::ptr::null_mut();

        if RegOpenKeyW(hkey_root, szRepPath, &mut hkey) != ERROR_SUCCESS as LONG {
            return S_FALSE;
        }

        let mut idx: i32 = 0;
        loop {
            if RegEnumKeyW(
                hkey,
                idx as u32,
                name.as_mut_ptr(),
                MAX_PATH as u32,
            ) != ERROR_SUCCESS as LONG
            {
                break;
            }

            if name[0 as usize] != 0x007Bu16 {
                let mut dwSize: u32 = core::mem::size_of_val(&name) as u32;
                RegGetValueW(
                    hkey,
                    name.as_mut_ptr(),
                    core::ptr::null(),
                    RRF_RT_REG_SZ as u32,
                    core::ptr::null_mut(),
                    name.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut dwSize,
                );
            }

            if name[0 as usize] == '{' as u16 {
                let pidl = CreateRegItem_2_body(
                    CRegFolderEnum_GetPidlType_body(this),
                    name.as_mut_ptr(),
                );
                if !pidl.is_null() {
                    let query = SHELL_CreateFolderEnumItemAttributeQuery((*this).m_SHCTF, TRUE);
                    if SHELL_IncludeItemInFolderEnum(pSF, pidl, query, (*this).m_SHCTF) != 0
                        && CRegFolderEnum_HasItemWithCLSID_body(this, pidl) == 0
                    {
                        CRegFolderEnum_AddToEnumList_body(this, pidl);
                    } else {
                        ILFree(pidl);
                    }
                }
            }

            idx += 1;
        }

        RegCloseKey(hkey);

        return S_OK;
    }
}
