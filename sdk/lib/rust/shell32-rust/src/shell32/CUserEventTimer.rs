use crate::prelude::*;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IUserEventTimer {
    pub lpVtbl: *const IUserEventTimerVtbl,
}
#[repr(C)]
pub struct IUserEventTimerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUserEventTimer, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUserEventTimer) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUserEventTimer) -> ULONG,
    pub UserEventTimerProc: unsafe extern "system" fn(This: *mut IUserEventTimer, uUserEventTimerID: ULONG, uTimerElapse: UINT) -> HRESULT,
    pub SetUserEventTimer: unsafe extern "system" fn(This: *mut IUserEventTimer, hWnd: HWND, uCallbackMsg: UINT, uTimerElapse: UINT, pUserEventTimerCallback: *mut IUserEventTimerCallback, puUserEventTimerID: *mut ULONG) -> HRESULT,
    pub KillUserEventTimer: unsafe extern "system" fn(This: *mut IUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG) -> HRESULT,
    pub GetUserEventTimerElapsed: unsafe extern "system" fn(This: *mut IUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG, puTimerElapsed: *mut UINT) -> HRESULT,
    pub InitTimerTickInterval: unsafe extern "system" fn(This: *mut IUserEventTimer, uTimerTickIntervalMs: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IUserEventTimer: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IUserEventTimerCallback {
    pub lpVtbl: *const IUserEventTimerCallbackVtbl,
}
#[repr(C)]
pub struct IUserEventTimerCallbackVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUserEventTimerCallback, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUserEventTimerCallback) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUserEventTimerCallback) -> ULONG,
    pub UserEventTimerProc: unsafe extern "system" fn(This: *mut IUserEventTimerCallback, uUserEventTimerID: ULONG, uTimerElapse: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IUserEventTimerCallback: IID;
}

// Deterministic COM scaffold for CUserEventTimer; method bodies are generated separately.
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
pub struct CUserEventTimer {
    pub IUserEventTimer_iface: IUserEventTimer,
    pub ref_count: core::sync::atomic::AtomicI32,
}

unsafe extern "system" fn CUserEventTimer_IUserEventTimer_QueryInterface(This: *mut IUserEventTimer, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_AddRef(This: *mut IUserEventTimer) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_AddRef(this)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_Release(This: *mut IUserEventTimer) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_Release(this)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_UserEventTimerProc(This: *mut IUserEventTimer, uUserEventTimerID: ULONG, uTimerElapse: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_UserEventTimerProc_body(this, uUserEventTimerID, uTimerElapse)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_SetUserEventTimer(This: *mut IUserEventTimer, hWnd: HWND, uCallbackMsg: UINT, uTimerElapse: UINT, pUserEventTimerCallback: *mut IUserEventTimerCallback, puUserEventTimerID: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_SetUserEventTimer_body(this, hWnd, uCallbackMsg, uTimerElapse, pUserEventTimerCallback, puUserEventTimerID)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_KillUserEventTimer(This: *mut IUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_KillUserEventTimer_body(this, hWnd, uUserEventTimerID)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_GetUserEventTimerElapsed(This: *mut IUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG, puTimerElapsed: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_GetUserEventTimerElapsed_body(this, hWnd, uUserEventTimerID, puTimerElapsed)
}
unsafe extern "system" fn CUserEventTimer_IUserEventTimer_InitTimerTickInterval(This: *mut IUserEventTimer, uTimerTickIntervalMs: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserEventTimer, IUserEventTimer_iface)) as *mut CUserEventTimer;
    CUserEventTimer_InitTimerTickInterval_body(this, uTimerTickIntervalMs)
}

pub static CUSEREVENTTIMER_IUSEREVENTTIMER_VTBL: IUserEventTimerVtbl = IUserEventTimerVtbl {
    QueryInterface: CUserEventTimer_IUserEventTimer_QueryInterface,
    AddRef: CUserEventTimer_IUserEventTimer_AddRef,
    Release: CUserEventTimer_IUserEventTimer_Release,
    UserEventTimerProc: CUserEventTimer_IUserEventTimer_UserEventTimerProc,
    SetUserEventTimer: CUserEventTimer_IUserEventTimer_SetUserEventTimer,
    KillUserEventTimer: CUserEventTimer_IUserEventTimer_KillUserEventTimer,
    GetUserEventTimerElapsed: CUserEventTimer_IUserEventTimer_GetUserEventTimerElapsed,
    InitTimerTickInterval: CUserEventTimer_IUserEventTimer_InitTimerTickInterval,
};

unsafe fn CUserEventTimer_QueryInterface(
    this: *mut CUserEventTimer, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUserEventTimer)) {
        *ppv = core::ptr::addr_of_mut!((*this).IUserEventTimer_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CUserEventTimer_AddRef(this);
    S_OK
}

unsafe fn CUserEventTimer_AddRef(this: *mut CUserEventTimer) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CUserEventTimer_Release(this: *mut CUserEventTimer) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CUserEventTimer_dtor_body(this);
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

// Activation: CLSID_UserEventTimer currently maps to CUserEventTimer in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CUserEventTimer_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CUserEventTimer>()) as *mut CUserEventTimer;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CUserEventTimer {
        IUserEventTimer_iface: IUserEventTimer { lpVtbl: core::ptr::addr_of!(CUSEREVENTTIMER_IUSEREVENTTIMER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
    });
    Shell32RustObjectLock();
    CUserEventTimer_ctor_body(this);
    let hr = CUserEventTimer_QueryInterface(this, riid, ppv);
    CUserEventTimer_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_USEREVENTTIMER: UINT = 166;

extern "C" {
    static CLSID_UserEventTimer: GUID;
}

use core::ffi::c_void;

pub unsafe fn CUserEventTimer_ctor_body(this: *mut CUserEventTimer) -> () {}

pub unsafe fn CUserEventTimer_dtor_body(this: *mut CUserEventTimer) -> () {}

pub unsafe fn CUserEventTimer_UserEventTimerProc_body(this: *mut CUserEventTimer, uUserEventTimerID: ULONG, uTimerElapse: UINT) -> HRESULT { S_OK }

pub unsafe fn CUserEventTimer_SetUserEventTimer_body(this: *mut CUserEventTimer, hWnd: HWND, uCallbackMsg: UINT, uTimerElapse: UINT, pUserEventTimerCallback: *mut IUserEventTimerCallback, puUserEventTimerID: *mut ULONG) -> HRESULT { S_OK }

pub unsafe fn CUserEventTimer_KillUserEventTimer_body(this: *mut CUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG) -> HRESULT { S_OK }

pub unsafe fn CUserEventTimer_GetUserEventTimerElapsed_body(this: *mut CUserEventTimer, hWnd: HWND, uUserEventTimerID: ULONG, puTimerElapsed: *mut UINT) -> HRESULT { S_OK }

pub unsafe fn CUserEventTimer_InitTimerTickInterval_body(this: *mut CUserEventTimer, uTimerTickIntervalMs: UINT) -> HRESULT { S_OK }
