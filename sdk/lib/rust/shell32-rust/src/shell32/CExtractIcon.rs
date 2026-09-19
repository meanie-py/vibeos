// rustconv-assembly: class:CExtractIcon
use crate::prelude::*;
use crate::drv_def_ext::ShellObjectCreator;
use crate::atl::wide_len;
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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IExtractIconW {
    pub lpVtbl: *const IExtractIconWVtbl,
}
#[repr(C)]
pub struct IExtractIconWVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExtractIconW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExtractIconW) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExtractIconW) -> ULONG,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IExtractIconW, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT,
    pub Extract: unsafe extern "system" fn(This: *mut IExtractIconW, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IExtractIconW: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IExtractIconA {
    pub lpVtbl: *const IExtractIconAVtbl,
}
#[repr(C)]
pub struct IExtractIconAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExtractIconA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExtractIconA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExtractIconA) -> ULONG,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IExtractIconA, uFlags: UINT, szIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT,
    pub Extract: unsafe extern "system" fn(This: *mut IExtractIconA, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IExtractIconA: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT,
}
extern "C" {
    pub static IID_IPersist: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFile: IID;
}

#[repr(C)]
pub struct IconLocation {
    pub file: LPWSTR,
    pub index: UINT,
}

// Deterministic COM scaffold for CExtractIcon; method bodies are generated separately.
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
pub struct CExtractIcon {
    pub IDefaultExtractIconInit_iface: IDefaultExtractIconInit,
    pub IExtractIconW_iface: IExtractIconW,
    pub IExtractIconA_iface: IExtractIconA,
    pub IPersistFile_iface: IPersistFile,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub flags: UINT,
    pub defaultIcon: IconLocation,
    pub normalIcon: IconLocation,
    pub openIcon: IconLocation,
    pub shortcutIcon: IconLocation,
}

unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_QueryInterface(This: *mut IDefaultExtractIconInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_AddRef(This: *mut IDefaultExtractIconInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_AddRef(this)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_Release(This: *mut IDefaultExtractIconInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_Release(this)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetDefaultIcon(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetDefaultIcon_body(this, pszFile, iIcon)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetFlags(This: *mut IDefaultExtractIconInit, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetFlags_body(this, uFlags)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetKey(This: *mut IDefaultExtractIconInit, hkey: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetKey_body(this, hkey)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetNormalIcon(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetNormalIcon_body(this, pszFile, iIcon)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetOpenIcon(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetOpenIcon_body(this, pszFile, iIcon)
}
unsafe extern "system" fn CExtractIcon_IDefaultExtractIconInit_SetShortcutIcon(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IDefaultExtractIconInit_iface)) as *mut CExtractIcon;
    CExtractIcon_SetShortcutIcon_body(this, pszFile, iIcon)
}

pub static CEXTRACTICON_IDEFAULTEXTRACTICONINIT_VTBL: IDefaultExtractIconInitVtbl = IDefaultExtractIconInitVtbl {
    QueryInterface: CExtractIcon_IDefaultExtractIconInit_QueryInterface,
    AddRef: CExtractIcon_IDefaultExtractIconInit_AddRef,
    Release: CExtractIcon_IDefaultExtractIconInit_Release,
    SetDefaultIcon: CExtractIcon_IDefaultExtractIconInit_SetDefaultIcon,
    SetFlags: CExtractIcon_IDefaultExtractIconInit_SetFlags,
    SetKey: CExtractIcon_IDefaultExtractIconInit_SetKey,
    SetNormalIcon: CExtractIcon_IDefaultExtractIconInit_SetNormalIcon,
    SetOpenIcon: CExtractIcon_IDefaultExtractIconInit_SetOpenIcon,
    SetShortcutIcon: CExtractIcon_IDefaultExtractIconInit_SetShortcutIcon,
};

unsafe extern "system" fn CExtractIcon_IExtractIconW_QueryInterface(This: *mut IExtractIconW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconW_iface)) as *mut CExtractIcon;
    CExtractIcon_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CExtractIcon_IExtractIconW_AddRef(This: *mut IExtractIconW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconW_iface)) as *mut CExtractIcon;
    CExtractIcon_AddRef(this)
}
unsafe extern "system" fn CExtractIcon_IExtractIconW_Release(This: *mut IExtractIconW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconW_iface)) as *mut CExtractIcon;
    CExtractIcon_Release(this)
}
unsafe extern "system" fn CExtractIcon_IExtractIconW_GetIconLocation(This: *mut IExtractIconW, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconW_iface)) as *mut CExtractIcon;
    CExtractIcon_GetIconLocation_v1_body(this, uFlags, szIconFile, cchMax, piIndex, pwFlags)
}
unsafe extern "system" fn CExtractIcon_IExtractIconW_Extract(This: *mut IExtractIconW, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconW_iface)) as *mut CExtractIcon;
    CExtractIcon_Extract_v1_body(this, pszFile, nIconIndex, phiconLarge, phiconSmall, nIconSize)
}

pub static CEXTRACTICON_IEXTRACTICONW_VTBL: IExtractIconWVtbl = IExtractIconWVtbl {
    QueryInterface: CExtractIcon_IExtractIconW_QueryInterface,
    AddRef: CExtractIcon_IExtractIconW_AddRef,
    Release: CExtractIcon_IExtractIconW_Release,
    GetIconLocation: CExtractIcon_IExtractIconW_GetIconLocation,
    Extract: CExtractIcon_IExtractIconW_Extract,
};

unsafe extern "system" fn CExtractIcon_IExtractIconA_QueryInterface(This: *mut IExtractIconA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconA_iface)) as *mut CExtractIcon;
    CExtractIcon_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CExtractIcon_IExtractIconA_AddRef(This: *mut IExtractIconA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconA_iface)) as *mut CExtractIcon;
    CExtractIcon_AddRef(this)
}
unsafe extern "system" fn CExtractIcon_IExtractIconA_Release(This: *mut IExtractIconA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconA_iface)) as *mut CExtractIcon;
    CExtractIcon_Release(this)
}
unsafe extern "system" fn CExtractIcon_IExtractIconA_GetIconLocation(This: *mut IExtractIconA, uFlags: UINT, szIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconA_iface)) as *mut CExtractIcon;
    CExtractIcon_GetIconLocation_v2_body(this, uFlags, szIconFile, cchMax, piIndex, pwFlags)
}
unsafe extern "system" fn CExtractIcon_IExtractIconA_Extract(This: *mut IExtractIconA, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IExtractIconA_iface)) as *mut CExtractIcon;
    CExtractIcon_Extract_v2_body(this, pszFile, nIconIndex, phiconLarge, phiconSmall, nIconSize)
}

pub static CEXTRACTICON_IEXTRACTICONA_VTBL: IExtractIconAVtbl = IExtractIconAVtbl {
    QueryInterface: CExtractIcon_IExtractIconA_QueryInterface,
    AddRef: CExtractIcon_IExtractIconA_AddRef,
    Release: CExtractIcon_IExtractIconA_Release,
    GetIconLocation: CExtractIcon_IExtractIconA_GetIconLocation,
    Extract: CExtractIcon_IExtractIconA_Extract,
};

unsafe extern "system" fn CExtractIcon_IPersistFile_QueryInterface(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_AddRef(This: *mut IPersistFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_AddRef(this)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_Release(This: *mut IPersistFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_Release(this)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_GetClassID(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_IsDirty(This: *mut IPersistFile) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_IsDirty_body(this)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_Load(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_Load_body(this, pszFileName, dwMode)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_Save(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_Save_body(this, pszFileName, fRemember)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_SaveCompleted(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_SaveCompleted_body(this, pszFileName)
}
unsafe extern "system" fn CExtractIcon_IPersistFile_GetCurFile(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CExtractIcon, IPersistFile_iface)) as *mut CExtractIcon;
    CExtractIcon_GetCurFile_body(this, ppszFileName)
}

pub static CEXTRACTICON_IPERSISTFILE_VTBL: IPersistFileVtbl = IPersistFileVtbl {
    QueryInterface: CExtractIcon_IPersistFile_QueryInterface,
    AddRef: CExtractIcon_IPersistFile_AddRef,
    Release: CExtractIcon_IPersistFile_Release,
    GetClassID: CExtractIcon_IPersistFile_GetClassID,
    IsDirty: CExtractIcon_IPersistFile_IsDirty,
    Load: CExtractIcon_IPersistFile_Load,
    Save: CExtractIcon_IPersistFile_Save,
    SaveCompleted: CExtractIcon_IPersistFile_SaveCompleted,
    GetCurFile: CExtractIcon_IPersistFile_GetCurFile,
};

unsafe fn CExtractIcon_QueryInterface(
    this: *mut CExtractIcon, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDefaultExtractIconInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDefaultExtractIconInit_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconW)) {
        *ppv = core::ptr::addr_of_mut!((*this).IExtractIconW_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconA)) {
        *ppv = core::ptr::addr_of_mut!((*this).IExtractIconA_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFile_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFile)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFile_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CExtractIcon_AddRef(this);
    S_OK
}

unsafe fn CExtractIcon_AddRef(this: *mut CExtractIcon) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CExtractIcon_Release(this: *mut CExtractIcon) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CExtractIcon_dtor_body(this);
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
pub unsafe fn CExtractIcon_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CExtractIcon>()) as *mut CExtractIcon;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CExtractIcon {
        IDefaultExtractIconInit_iface: IDefaultExtractIconInit { lpVtbl: core::ptr::addr_of!(CEXTRACTICON_IDEFAULTEXTRACTICONINIT_VTBL) },
        IExtractIconW_iface: IExtractIconW { lpVtbl: core::ptr::addr_of!(CEXTRACTICON_IEXTRACTICONW_VTBL) },
        IExtractIconA_iface: IExtractIconA { lpVtbl: core::ptr::addr_of!(CEXTRACTICON_IEXTRACTICONA_VTBL) },
        IPersistFile_iface: IPersistFile { lpVtbl: core::ptr::addr_of!(CEXTRACTICON_IPERSISTFILE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        flags: core::mem::zeroed(),
        defaultIcon: core::mem::zeroed(),
        normalIcon: core::mem::zeroed(),
        openIcon: core::mem::zeroed(),
        shortcutIcon: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CExtractIcon_ctor_body(this);
    let hr = CExtractIcon_QueryInterface(this, riid, ppv);
    CExtractIcon_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const GIL_DEFAULTICON: DWORD = 0x0040;

pub const GIL_FORSHORTCUT: DWORD = 0x0080;

pub const GIL_OPENICON: DWORD = 0x0001;

pub const SHGFI_ICONLOCATION: DWORD = 4096;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

extern "system" {
    fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;
    #[link_name = "GUID_NULL"]
    static GUID_NULL: GUID;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn CoTaskMemFree(memory: LPVOID);
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
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
// `DuplicateString` is a file-scope helper of this same C++ TU (CExtractIcon.cpp:72).
// Splitting the TU into a class half and a free half turned that intra-TU call into a
// cross-module one, and the class half was translated BEFORE the free half existed, so
// the import resolver had nothing to resolve it against.
use crate::shell32::extracticon_free::DuplicateString;

pub unsafe fn CExtractIcon_ctor_body(this: *mut CExtractIcon) -> () {
    (*this).flags = 0;
    memset(
        core::ptr::addr_of_mut!((*this).defaultIcon).cast(),
        0,
        core::mem::size_of_val(&(*this).defaultIcon),
    );
    memset(
        core::ptr::addr_of_mut!((*this).normalIcon).cast(),
        0,
        core::mem::size_of_val(&(*this).normalIcon),
    );
    memset(
        core::ptr::addr_of_mut!((*this).openIcon).cast(),
        0,
        core::mem::size_of_val(&(*this).openIcon),
    );
    memset(
        core::ptr::addr_of_mut!((*this).shortcutIcon).cast(),
        0,
        core::mem::size_of_val(&(*this).shortcutIcon),
    );
}

pub unsafe fn CExtractIcon_dtor_body(this: *mut CExtractIcon) -> () {
    if !(*this).defaultIcon.file.is_null() {
        CoTaskMemFree((*this).defaultIcon.file.cast());
    }
    if !(*this).normalIcon.file.is_null() {
        CoTaskMemFree((*this).normalIcon.file.cast());
    }
    if !(*this).openIcon.file.is_null() {
        CoTaskMemFree((*this).openIcon.file.cast());
    }
    if !(*this).shortcutIcon.file.is_null() {
        CoTaskMemFree((*this).shortcutIcon.file.cast());
    }
}

pub unsafe fn CExtractIcon_SetDefaultIcon_body(this: *mut CExtractIcon, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    DuplicateString(pszFile, core::ptr::addr_of_mut!((*this).defaultIcon.file));
    if (*this).defaultIcon.file.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).defaultIcon.index = iIcon as UINT;
    return S_OK;
}

pub unsafe fn CExtractIcon_SetFlags_body(this: *mut CExtractIcon, uFlags: UINT) -> HRESULT {
    (*this).flags = uFlags;
    return S_OK;
}

pub unsafe fn CExtractIcon_SetKey_body(this: *mut CExtractIcon, hkey: HKEY) -> HRESULT { E_NOTIMPL }

pub unsafe fn CExtractIcon_SetNormalIcon_body(this: *mut CExtractIcon, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    DuplicateString(pszFile, &mut (*this).normalIcon.file);
    if (*this).normalIcon.file.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).normalIcon.index = iIcon as UINT;
    return S_OK;
}

pub unsafe fn CExtractIcon_SetOpenIcon_body(this: *mut CExtractIcon, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let _ = DuplicateString(pszFile, &mut (*this).openIcon.file);
    if (*this).openIcon.file.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).openIcon.index = iIcon as UINT;
    S_OK
}

pub unsafe fn CExtractIcon_SetShortcutIcon_body(this: *mut CExtractIcon, pszFile: LPCWSTR, iIcon: i32) -> HRESULT {
    let _ = DuplicateString(pszFile, &mut (*this).shortcutIcon.file);
    if (*this).shortcutIcon.file.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).shortcutIcon.index = iIcon as UINT;
    return S_OK;
}

pub unsafe fn CExtractIcon_GetIconLocation_v1_body(this: *mut CExtractIcon, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    if piIndex.is_null() || pwFlags.is_null() {
        return E_POINTER;
    }

    let icon = if (uFlags & GIL_DEFAULTICON) != 0 {
        if unsafe { !(*this).defaultIcon.file.is_null() } {
            unsafe { core::ptr::addr_of!((*this).defaultIcon) }
        } else {
            unsafe { core::ptr::addr_of!((*this).normalIcon) }
        }
    } else if (uFlags & GIL_FORSHORTCUT) != 0 {
        if unsafe { !(*this).shortcutIcon.file.is_null() } {
            unsafe { core::ptr::addr_of!((*this).shortcutIcon) }
        } else {
            unsafe { core::ptr::addr_of!((*this).normalIcon) }
        }
    } else if (uFlags & GIL_OPENICON) != 0 {
        if unsafe { !(*this).openIcon.file.is_null() } {
            unsafe { core::ptr::addr_of!((*this).openIcon) }
        } else {
            unsafe { core::ptr::addr_of!((*this).normalIcon) }
        }
    } else {
        unsafe { core::ptr::addr_of!((*this).normalIcon) }
    };

    if unsafe { (*icon).file.is_null() } {
        return E_FAIL;
    }

    let cb = unsafe { wcslen((*icon).file) } + 1;
    if cchMax < cb as UINT {
        return E_FAIL;
    }

    unsafe {
        CopyMemory(szIconFile, (*icon).file, cb * core::mem::size_of::<u16>());
        *piIndex = (*icon).index as INT;
        *pwFlags = (*this).flags;
    }

    return S_OK;
}

pub unsafe fn CExtractIcon_Extract_v1_body(this: *mut CExtractIcon, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT { S_FALSE }

pub unsafe fn CExtractIcon_GetIconLocation_v2_body(this: *mut CExtractIcon, uFlags: UINT, szIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let mut szIconFileW: LPWSTR = core::ptr::null_mut();
    let mut hr: HRESULT;
    if cchMax > 0 {
        szIconFileW = CoTaskMemAlloc(((cchMax as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if szIconFileW.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    hr = CExtractIcon_GetIconLocation_v1_body(this, uFlags, szIconFileW, cchMax, piIndex, pwFlags);
    if (hr >= 0) && (cchMax > 0) {
        if 0 == WideCharToMultiByte(
            CP_ACP,
            0,
            szIconFileW,
            cchMax as INT,
            szIconFile,
            cchMax as INT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) {
            hr = E_FAIL;
        }
    }
    if !szIconFileW.is_null() {
        CoTaskMemFree(szIconFileW as *mut core::ffi::c_void);
    }
    hr
}

pub unsafe fn CExtractIcon_Extract_v2_body(this: *mut CExtractIcon, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let mut pszFileW: LPWSTR = core::ptr::null_mut();
    let hr: HRESULT;

    if !pszFile.is_null() {
        let nLength: i32 = MultiByteToWideChar(
            CP_ACP,
            0,
            pszFile,
            -1,
            core::ptr::null_mut(),
            0
        );

        if nLength == 0 {
            return E_FAIL;
        }

        pszFileW = CoTaskMemAlloc(
            (((nLength as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T)
        ) as LPWSTR;

        if pszFileW.is_null() {
            return E_OUTOFMEMORY;
        }

        if MultiByteToWideChar(
            CP_ACP,
            0,
            pszFile,
            nLength,
            pszFileW,
            nLength
        ) == 0 {
            CoTaskMemFree(pszFileW.cast());
            return E_FAIL;
        }
    }

    hr = CExtractIcon_Extract_v1_body(this, pszFileW, nIconIndex, phiconLarge, phiconSmall, nIconSize);

    if !pszFileW.is_null() {
        CoTaskMemFree(pszFileW.cast());
    }

    hr
}

pub unsafe fn CExtractIcon_GetClassID_body(this: *mut CExtractIcon, pClassID: *mut CLSID) -> HRESULT {
    if pClassID.is_null() {
        return E_POINTER;
    }

    unsafe {
        // `GUID_NULL` is rendered by the COM island, `*pClassID` by the ntdll one: same
        // 16 bytes, two nominal spellings. A pointer cast, not a layout change.
        core::ptr::write(pClassID.cast::<GUID>(), GUID_NULL);
    }

    S_OK
}

pub unsafe fn CExtractIcon_IsDirty_body(this: *mut CExtractIcon) -> HRESULT { E_NOTIMPL }

pub unsafe fn CExtractIcon_Load_body(this: *mut CExtractIcon, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CExtractIcon_Save_body(this: *mut CExtractIcon, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CExtractIcon_SaveCompleted_body(this: *mut CExtractIcon, pszFileName: LPCOLESTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CExtractIcon_GetCurFile_body(this: *mut CExtractIcon, ppszFileName: *mut LPOLESTR) -> HRESULT { E_NOTIMPL }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Registry namespace extension
//  * FILE:            dll/win32/shell32/extracticon.c
//  * PURPOSE:         Icon extraction
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// struct IconLocation
// {
//     LPWSTR file;
//     UINT index;
// };
//
// class CExtractIcon :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IDefaultExtractIconInit,
//     public IExtractIconW,
//     public IExtractIconA,
//     public IPersistFile
// {
// private:
//     UINT flags;
//     struct IconLocation defaultIcon;
//     struct IconLocation normalIcon;
//     struct IconLocation openIcon;
//     struct IconLocation shortcutIcon;
// public:
//     CExtractIcon();
//     ~CExtractIcon();
//
//     // IDefaultExtractIconInit
//     STDMETHOD(SetDefaultIcon)(LPCWSTR pszFile, int iIcon) override;
//     STDMETHOD(SetFlags)(UINT uFlags) override;
//     STDMETHOD(SetKey)(HKEY hkey) override;
//     STDMETHOD(SetNormalIcon)(LPCWSTR pszFile, int iIcon) override;
//     STDMETHOD(SetOpenIcon)(LPCWSTR pszFile, int iIcon) override;
//     STDMETHOD(SetShortcutIcon)(LPCWSTR pszFile, int iIcon) override;
//
//     // IExtractIconW
//     STDMETHOD(GetIconLocation)(UINT uFlags, LPWSTR szIconFile, UINT cchMax, int *piIndex, UINT *pwFlags) override;
//     STDMETHOD(Extract)(LPCWSTR pszFile, UINT nIconIndex, HICON *phiconLarge, HICON *phiconSmall, UINT nIconSize) override;
//
//     // IExtractIconA
//     STDMETHOD(GetIconLocation)(UINT uFlags, LPSTR szIconFile, UINT cchMax, int *piIndex, UINT *pwFlags) override;
//     STDMETHOD(Extract)(LPCSTR pszFile, UINT nIconIndex, HICON *phiconLarge, HICON *phiconSmall, UINT nIconSize) override;
//
//     // IPersist
//     STDMETHOD(GetClassID)(CLSID *pClassID) override;
//     STDMETHOD(IsDirty)() override;
//
//     // IPersistFile
//     STDMETHOD(Load)(LPCOLESTR pszFileName, DWORD dwMode) override;
//     STDMETHOD(Save)(LPCOLESTR pszFileName, BOOL fRemember) override;
//     STDMETHOD(SaveCompleted)(LPCOLESTR pszFileName) override;
//     STDMETHOD(GetCurFile)(LPOLESTR *ppszFileName) override;
//
// BEGIN_COM_MAP(CExtractIcon)
//     COM_INTERFACE_ENTRY_IID(IID_IDefaultExtractIconInit, IDefaultExtractIconInit)
//     COM_INTERFACE_ENTRY_IID(IID_IExtractIconW, IExtractIconW)
//     COM_INTERFACE_ENTRY_IID(IID_IExtractIconA, IExtractIconA)
//     COM_INTERFACE_ENTRY_IID(IID_IPersist, IPersist)
//     COM_INTERFACE_ENTRY_IID(IID_IPersistFile, IPersistFile)
// END_COM_MAP()
// };
//
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetDefaultIcon(
//     LPCWSTR pszFile,
//     int iIcon)
// {
//     TRACE("(%p, %s, %d)\n", this, debugstr_w(pszFile), iIcon);
//
//     DuplicateString(pszFile, &defaultIcon.file);
//     if (!defaultIcon.file)
//         return E_OUTOFMEMORY;
//     defaultIcon.index = iIcon;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetFlags(
//     UINT uFlags)
// {
//     TRACE("(%p, 0x%x)\n", this, uFlags);
//
//     flags = uFlags;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetKey(
//     HKEY hkey)
// {
//     FIXME("(%p, %p)\n", this, hkey);
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetNormalIcon(
//     LPCWSTR pszFile,
//     int iIcon)
// {
//     TRACE("(%p, %s, %d)\n", this, debugstr_w(pszFile), iIcon);
//
//     DuplicateString(pszFile, &normalIcon.file);
//     if (!normalIcon.file)
//         return E_OUTOFMEMORY;
//     normalIcon.index = iIcon;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetOpenIcon(
//     LPCWSTR pszFile,
//     int iIcon)
// {
//     TRACE("(%p, %s, %d)\n", this, debugstr_w(pszFile), iIcon);
//
//     DuplicateString(pszFile, &openIcon.file);
//     if (!openIcon.file)
//         return E_OUTOFMEMORY;
//     openIcon.index = iIcon;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SetShortcutIcon(
//     LPCWSTR pszFile,
//     int iIcon)
// {
//     TRACE("(%p, %s, %d)\n", this, debugstr_w(pszFile), iIcon);
//
//     DuplicateString(pszFile, &shortcutIcon.file);
//     if (!shortcutIcon.file)
//         return E_OUTOFMEMORY;
//     shortcutIcon.index = iIcon;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::GetClassID(
//     CLSID *pClassID)
// {
//     TRACE("(%p, %p)\n", this, pClassID);
//
//     if (!pClassID)
//         return E_POINTER;
//
//     *pClassID = GUID_NULL;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::IsDirty()
// {
//     FIXME("(%p)\n", this);
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::Load(
//     LPCOLESTR pszFileName,
//     DWORD dwMode)
// {
//     FIXME("(%p, %s, %u)\n", this, debugstr_w(pszFileName), dwMode);
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::Save(
//     LPCOLESTR pszFileName,
//     BOOL fRemember)
// {
//     FIXME("(%p, %s, %d)\n", this, debugstr_w(pszFileName), fRemember);
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::SaveCompleted(
//     LPCOLESTR pszFileName)
// {
//     FIXME("(%p, %s)\n", this, debugstr_w(pszFileName));
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CExtractIcon::GetCurFile(
//     LPOLESTR *ppszFileName)
// {
//     FIXME("(%p, %p)\n", this, ppszFileName);
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
