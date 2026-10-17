// Deterministic from emptyvc.h
#[repr(C)]
pub struct IEmptyVolumeCache2 {
    pub lpVtbl: *const IEmptyVolumeCache2Vtbl,
}
#[repr(C)]
pub struct IEmptyVolumeCache2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, hkRegKey: HKEY, pcwszVolume: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT,
    pub GetSpaceUsed: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, pdwlSpaceUsed: *mut DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT,
    pub Purge: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, dwlSpaceToFree: DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT,
    pub ShowProperties: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, hwnd: HWND) -> HRESULT,
    pub Deactivate: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, pdwFlags: *mut DWORD) -> HRESULT,
    pub InitializeEx: unsafe extern "system" fn(This: *mut IEmptyVolumeCache2, hkRegKey: HKEY, pcwszVolume: LPCWSTR, pcwszKeyName: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, ppwszBtnText: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IEmptyVolumeCache2: IID;
}

// Deterministic from emptyvc.h
#[repr(C)]
pub struct IEmptyVolumeCache {
    pub lpVtbl: *const IEmptyVolumeCacheVtbl,
}
#[repr(C)]
pub struct IEmptyVolumeCacheVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEmptyVolumeCache) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEmptyVolumeCache) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, hkRegKey: HKEY, pcwszVolume: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT,
    pub GetSpaceUsed: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, pdwlSpaceUsed: *mut DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT,
    pub Purge: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, dwlSpaceToFree: DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT,
    pub ShowProperties: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, hwnd: HWND) -> HRESULT,
    pub Deactivate: unsafe extern "system" fn(This: *mut IEmptyVolumeCache, pdwFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IEmptyVolumeCache: IID;
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

// Deterministic from emptyvc.h
#[repr(C)]
pub struct IEmptyVolumeCacheCallBack {
    pub lpVtbl: *const IEmptyVolumeCacheCallBackVtbl,
}
#[repr(C)]
pub struct IEmptyVolumeCacheCallBackVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack) -> ULONG,
    pub ScanProgress: unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack, dwlSpaceUsed: DWORDLONG, dwFlags: DWORD, pcwszStatus: LPCWSTR) -> HRESULT,
    pub PurgeProgress: unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack, dwlSpaceFreed: DWORDLONG, dwlSpaceToFree: DWORDLONG, dwFlags: DWORD, pcwszStatus: LPCWSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IEmptyVolumeCacheCallBack: IID;
}

// Deterministic COM scaffold for CRecycleBinCleaner; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}

#[repr(C)]
pub struct CRecycleBinCleaner {
    pub IEmptyVolumeCache2_iface: IEmptyVolumeCache2,
    pub IEmptyVolumeCache_iface: IEmptyVolumeCache,
    pub IUnknown_iface: IUnknown,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_wszVolume: [WCHAR; 4],
}

unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_QueryInterface(This: *mut IEmptyVolumeCache2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_AddRef(This: *mut IEmptyVolumeCache2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_AddRef(this)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_Release(This: *mut IEmptyVolumeCache2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Release(this)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_Initialize(This: *mut IEmptyVolumeCache2, hkRegKey: HKEY, pcwszVolume: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Initialize_body(this, hkRegKey, pcwszVolume, ppwszDisplayName, ppwszDescription, pdwFlags)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_GetSpaceUsed(This: *mut IEmptyVolumeCache2, pdwlSpaceUsed: *mut DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_GetSpaceUsed_body(this, pdwlSpaceUsed, picb)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_Purge(This: *mut IEmptyVolumeCache2, dwlSpaceToFree: DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Purge_body(this, dwlSpaceToFree, picb)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_ShowProperties(This: *mut IEmptyVolumeCache2, hwnd: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_ShowProperties_body(this, hwnd)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_Deactivate(This: *mut IEmptyVolumeCache2, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Deactivate_body(this, pdwFlags)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache2_InitializeEx(This: *mut IEmptyVolumeCache2, hkRegKey: HKEY, pcwszVolume: LPCWSTR, pcwszKeyName: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, ppwszBtnText: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache2_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_InitializeEx_body(this, hkRegKey, pcwszVolume, pcwszKeyName, ppwszDisplayName, ppwszDescription, ppwszBtnText, pdwFlags)
}

pub static CRECYCLEBINCLEANER_IEMPTYVOLUMECACHE2_VTBL: IEmptyVolumeCache2Vtbl = IEmptyVolumeCache2Vtbl {
    QueryInterface: CRecycleBinCleaner_IEmptyVolumeCache2_QueryInterface,
    AddRef: CRecycleBinCleaner_IEmptyVolumeCache2_AddRef,
    Release: CRecycleBinCleaner_IEmptyVolumeCache2_Release,
    Initialize: CRecycleBinCleaner_IEmptyVolumeCache2_Initialize,
    GetSpaceUsed: CRecycleBinCleaner_IEmptyVolumeCache2_GetSpaceUsed,
    Purge: CRecycleBinCleaner_IEmptyVolumeCache2_Purge,
    ShowProperties: CRecycleBinCleaner_IEmptyVolumeCache2_ShowProperties,
    Deactivate: CRecycleBinCleaner_IEmptyVolumeCache2_Deactivate,
    InitializeEx: CRecycleBinCleaner_IEmptyVolumeCache2_InitializeEx,
};

unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_QueryInterface(This: *mut IEmptyVolumeCache, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_AddRef(This: *mut IEmptyVolumeCache) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_AddRef(this)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_Release(This: *mut IEmptyVolumeCache) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Release(this)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_Initialize(This: *mut IEmptyVolumeCache, hkRegKey: HKEY, pcwszVolume: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Initialize_body(this, hkRegKey, pcwszVolume, ppwszDisplayName, ppwszDescription, pdwFlags)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_GetSpaceUsed(This: *mut IEmptyVolumeCache, pdwlSpaceUsed: *mut DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_GetSpaceUsed_body(this, pdwlSpaceUsed, picb)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_Purge(This: *mut IEmptyVolumeCache, dwlSpaceToFree: DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Purge_body(this, dwlSpaceToFree, picb)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_ShowProperties(This: *mut IEmptyVolumeCache, hwnd: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_ShowProperties_body(this, hwnd)
}
unsafe extern "system" fn CRecycleBinCleaner_IEmptyVolumeCache_Deactivate(This: *mut IEmptyVolumeCache, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IEmptyVolumeCache_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Deactivate_body(this, pdwFlags)
}

pub static CRECYCLEBINCLEANER_IEMPTYVOLUMECACHE_VTBL: IEmptyVolumeCacheVtbl = IEmptyVolumeCacheVtbl {
    QueryInterface: CRecycleBinCleaner_IEmptyVolumeCache_QueryInterface,
    AddRef: CRecycleBinCleaner_IEmptyVolumeCache_AddRef,
    Release: CRecycleBinCleaner_IEmptyVolumeCache_Release,
    Initialize: CRecycleBinCleaner_IEmptyVolumeCache_Initialize,
    GetSpaceUsed: CRecycleBinCleaner_IEmptyVolumeCache_GetSpaceUsed,
    Purge: CRecycleBinCleaner_IEmptyVolumeCache_Purge,
    ShowProperties: CRecycleBinCleaner_IEmptyVolumeCache_ShowProperties,
    Deactivate: CRecycleBinCleaner_IEmptyVolumeCache_Deactivate,
};

unsafe extern "system" fn CRecycleBinCleaner_IUnknown_QueryInterface(This: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IUnknown_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBinCleaner_IUnknown_AddRef(This: *mut IUnknown) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IUnknown_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_AddRef(this)
}
unsafe extern "system" fn CRecycleBinCleaner_IUnknown_Release(This: *mut IUnknown) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBinCleaner, IUnknown_iface)) as *mut CRecycleBinCleaner;
    CRecycleBinCleaner_Release(this)
}

pub static CRECYCLEBINCLEANER_IUNKNOWN_VTBL: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: CRecycleBinCleaner_IUnknown_QueryInterface,
    AddRef: CRecycleBinCleaner_IUnknown_AddRef,
    Release: CRecycleBinCleaner_IUnknown_Release,
};

unsafe fn CRecycleBinCleaner_QueryInterface(
    this: *mut CRecycleBinCleaner, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEmptyVolumeCache2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEmptyVolumeCache2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEmptyVolumeCache)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEmptyVolumeCache_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) {
        *ppv = core::ptr::addr_of_mut!((*this).IUnknown_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecycleBinCleaner_AddRef(this);
    S_OK
}

unsafe fn CRecycleBinCleaner_AddRef(this: *mut CRecycleBinCleaner) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecycleBinCleaner_Release(this: *mut CRecycleBinCleaner) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecycleBinCleaner_dtor_body(this);
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

// Activation: CLSID_RecycleBinCleaner currently maps to CRecycleBinCleaner in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CRecycleBinCleaner_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<CRecycleBinCleaner>()) as _) as *mut CRecycleBinCleaner;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecycleBinCleaner {
        IEmptyVolumeCache2_iface: IEmptyVolumeCache2 { lpVtbl: core::ptr::addr_of!(CRECYCLEBINCLEANER_IEMPTYVOLUMECACHE2_VTBL) },
        IEmptyVolumeCache_iface: IEmptyVolumeCache { lpVtbl: core::ptr::addr_of!(CRECYCLEBINCLEANER_IEMPTYVOLUMECACHE_VTBL) },
        IUnknown_iface: IUnknown { lpVtbl: core::ptr::addr_of!(CRECYCLEBINCLEANER_IUNKNOWN_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_wszVolume: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecycleBinCleaner_ctor_body(this);
    let hr = CRecycleBinCleaner_QueryInterface(this, riid, ppv);
    CRecycleBinCleaner_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_RECYCLEBINCLEANER: UINT = 162;

#[no_mangle]
pub static CLSID_RecycleBinCleaner: GUID = GUID {
    Data1: 0x5ef4af3a,
    Data2: 0xf726,
    Data3: 0x11d0,
    Data4: [0xb8, 0xa2, 0x00, 0xc0, 0x4f, 0xc3, 0x09, 0xa4],
};

pub const IDS_RECYCLE_CLEANER_BUTTON_TEXT: UINT = 10293;

pub const IDS_RECYCLE_CLEANER_DESCRIPTION: UINT = 10292;

pub const IDS_RECYCLE_CLEANER_DISPLAYNAME: UINT = 10291;

pub const CSIDL_BITBUCKET: INT = 0x000a;

pub const EVCCBF_LASTNOTIFICATION: DWORD = 0x0001;

pub const EVCF_HASSETTINGS: DWORD = 0x0001;

pub const SEE_MASK_IDLIST: ULONG = 0x0000_0004;

pub const SHERB_NOCONFIRMATION: DWORD = 0x0000_0001;

pub const SHERB_NOPROGRESSUI: DWORD = 0x0000_0002;

pub const SHERB_NOSOUND: DWORD = 0x0000_0004;

pub const SW_SHOWNORMAL: INT = 1;

pub type DWORDLONG = u64;

pub use crate::island::SHQUERYRBINFO;

#[repr(C)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn SHQueryRecycleBinW(root: LPCWSTR, info: *mut SHQUERYRBINFO) -> HRESULT;
}

extern "C" {
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if capacity == 0 || capacity > i32::MAX as usize { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    let copied = core::cmp::min(len, capacity - 1);
    let _ = memcpy(dst.cast(), src.cast(), copied * core::mem::size_of::<WCHAR>());
    *dst.add(copied) = 0;
    if len >= capacity { 0x8007007Au32 as HRESULT } else { S_OK }
}

extern "system" {
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut capacity = 256usize;
        loop {
            if capacity > i32::MAX as usize || !value.reserve(capacity) { break; }
            let count = LoadStringW(
                GetModuleHandleW(core::ptr::null()), id, value.buffer, capacity as INT,
            );
            if count <= 0 {
                value.len = 0;
                *value.buffer = 0;
                break;
            }
            value.len = count as usize;
            if value.len < capacity - 1 {
                break;
            }
            let Some(next) = capacity.checked_mul(2) else { break };
            capacity = next;
        }
        value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, (bytes) as _)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), (bytes) as SIZE_T)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len).cast(), text.cast(),
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer.cast()); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::recycle_bin_tu::SHEmptyRecycleBinW;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
use crate::shell32::shlexec::ShellExecuteExW;
// One declaration per external symbol: island.rs.
use crate::island::GetModuleHandleW;
use crate::island::HeapAlloc;
use crate::island::HeapReAlloc;
use crate::island::LoadStringW;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CRecycleBinCleaner_ctor_body(this: *mut CRecycleBinCleaner) -> () {}

pub unsafe fn CRecycleBinCleaner_dtor_body(this: *mut CRecycleBinCleaner) -> () {}

pub unsafe fn CRecycleBinCleaner_OutputResourceString_body(this: *mut CRecycleBinCleaner, dwResId: DWORD, ppwszOutput: *mut LPWSTR) -> () {
    let mut Tmp = CStringW::from_resource(dwResId as UINT);
    SHStrDupW(Tmp.as_ptr(), ppwszOutput);
}

pub unsafe fn CRecycleBinCleaner_Initialize_body(this: *mut CRecycleBinCleaner, hkRegKey: HKEY, pcwszVolume: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT {
    if pdwFlags.is_null() {
        return E_POINTER;
    }
    *pdwFlags = EVCF_HASSETTINGS;
    CRecycleBinCleaner_OutputResourceString_body(this, IDS_RECYCLE_CLEANER_DISPLAYNAME, ppwszDisplayName);
    CRecycleBinCleaner_OutputResourceString_body(this, IDS_RECYCLE_CLEANER_DESCRIPTION, ppwszDescription);
    StringCchCopyW((*this).m_wszVolume.as_mut_ptr(),
        (*this).m_wszVolume.len(),
        pcwszVolume,
    )
}

pub unsafe fn CRecycleBinCleaner_GetSpaceUsed_body(this: *mut CRecycleBinCleaner, pdwlSpaceUsed: *mut DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    if pdwlSpaceUsed.is_null() {
        return E_POINTER;
    }

    let mut bin: SHQUERYRBINFO = core::mem::zeroed();
    bin.cbSize = core::mem::size_of::<SHQUERYRBINFO>() as u32;

    let volume_ptr: *const u16 = (*this).m_wszVolume.as_ptr();
    let hr = SHQueryRecycleBinW(volume_ptr, &mut bin as *mut SHQUERYRBINFO);
    if ((hr) < 0) {
        bin.i64Size = 0;
    }
    *pdwlSpaceUsed = bin.i64Size as DWORDLONG;
    if !picb.is_null() {
        ((*(*picb).lpVtbl).ScanProgress)(
            picb,
            bin.i64Size as DWORDLONG,
            EVCCBF_LASTNOTIFICATION,
            core::ptr::null(),
        );
    }

    S_OK
}

pub unsafe fn CRecycleBinCleaner_Purge_body(this: *mut CRecycleBinCleaner, dwlSpaceToFree: DWORDLONG, picb: *mut IEmptyVolumeCacheCallBack) -> HRESULT {
    let _ = dwlSpaceToFree;
    let mut dwlPrevious: DWORDLONG = 0;
    CRecycleBinCleaner_GetSpaceUsed_body(this, &mut dwlPrevious, core::ptr::null_mut());
    SHEmptyRecycleBinW(core::ptr::null_mut(), (*this).m_wszVolume.as_ptr(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND);
    if !picb.is_null() {
        let vtbl = (*picb).lpVtbl;
        ((*vtbl).PurgeProgress)(picb, dwlPrevious, 0, EVCCBF_LASTNOTIFICATION, core::ptr::null_mut());
    }
    S_OK
}

pub unsafe fn CRecycleBinCleaner_ShowProperties_body(this: *mut CRecycleBinCleaner, hwnd: HWND) -> HRESULT {
    let mut pidl: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut hr: HRESULT;
    hr = SHGetSpecialFolderLocation(hwnd, CSIDL_BITBUCKET, &mut pidl);
    if ((hr) < 0) {
        return hr;
    }

    let mut seei: SHELLEXECUTEINFOW = core::mem::zeroed();
    seei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    seei.hwnd = hwnd;
    seei.lpVerb = CPP_COM_WSTR_82DB442E7A60.as_ptr();
    seei.nShow = SW_SHOWNORMAL;
    seei.fMask = SEE_MASK_IDLIST;
    seei.lpIDList = pidl as *mut _;

    ShellExecuteExW((&raw mut seei).cast());

    // RAII cleanup: CComHeapPtr destructor calls CoTaskMemFree
    CoTaskMemFree(pidl as *mut core::ffi::c_void);

    S_OK
}

pub unsafe fn CRecycleBinCleaner_Deactivate_body(this: *mut CRecycleBinCleaner, pdwFlags: *mut DWORD) -> HRESULT {
    if pdwFlags.is_null() {
        return E_POINTER;
    }
    *pdwFlags = 0;
    S_OK
}

pub unsafe fn CRecycleBinCleaner_InitializeEx_body(this: *mut CRecycleBinCleaner, hkRegKey: HKEY, pcwszVolume: LPCWSTR, pcwszKeyName: LPCWSTR, ppwszDisplayName: *mut LPWSTR, ppwszDescription: *mut LPWSTR, ppwszBtnText: *mut LPWSTR, pdwFlags: *mut DWORD) -> HRESULT {
    CRecycleBinCleaner_OutputResourceString_body(this, IDS_RECYCLE_CLEANER_BUTTON_TEXT, ppwszBtnText);
    CRecycleBinCleaner_Initialize_body(this, hkRegKey, pcwszVolume, ppwszDisplayName, ppwszDescription, pdwFlags)
}
