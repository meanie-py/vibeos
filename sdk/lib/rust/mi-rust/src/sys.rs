//! What mi.dll imports from the system: the process heap, threads and events (kernel32),
//! COM activation (ole32) and BSTR/VARIANT/SAFEARRAY (oleaut32). Plus the handful of GUIDs
//! and VARTYPEs the WMI side needs; the island's `extern static` GUIDs would drag in
//! wbemuuid, so they are spelled here instead.

use crate::types::*;
use core::ffi::c_void;

extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut c_void;
    pub fn HeapFree(heap: HANDLE, flags: DWORD, mem: *mut c_void) -> BOOL;
    pub fn CreateThread(
        attrs: *mut c_void,
        stack: usize,
        start: Option<unsafe extern "system" fn(*mut c_void) -> DWORD>,
        param: *mut c_void,
        flags: DWORD,
        tid: *mut DWORD,
    ) -> HANDLE;
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn CreateEventW(attrs: *mut c_void, manual: BOOL, initial: BOOL, name: *const u16) -> HANDLE;
    pub fn SetEvent(h: HANDLE) -> BOOL;
    pub fn ResetEvent(h: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(h: HANDLE, ms: DWORD) -> DWORD;
    pub fn InitializeCriticalSection(cs: *mut CRITICAL_SECTION);
    pub fn EnterCriticalSection(cs: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(cs: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(cs: *mut CRITICAL_SECTION);
    pub fn GetComputerNameW(buf: *mut u16, len: *mut DWORD) -> BOOL;

    pub fn CoInitializeEx(reserved: *mut c_void, coinit: DWORD) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(
        clsid: *const GUID,
        outer: *mut c_void,
        ctx: DWORD,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> HRESULT;

    pub fn CoSetProxyBlanket(
        proxy: *mut IUnknown,
        authn_svc: DWORD,
        authz_svc: DWORD,
        principal: *mut u16,
        authn_level: DWORD,
        imp_level: DWORD,
        auth_info: *mut c_void,
        capabilities: DWORD,
    ) -> HRESULT;

    pub fn SysAllocString(s: *const u16) -> BSTR;
    pub fn SysFreeString(s: BSTR);
    pub fn SysStringLen(s: BSTR) -> UINT;
    pub fn VariantClear(v: *mut VARIANT) -> HRESULT;
    pub fn SafeArrayGetLBound(sa: *mut SAFEARRAY, dim: UINT, lb: *mut LONG) -> HRESULT;
    pub fn SafeArrayGetUBound(sa: *mut SAFEARRAY, dim: UINT, ub: *mut LONG) -> HRESULT;
    pub fn SafeArrayAccessData(sa: *mut SAFEARRAY, data: *mut *mut c_void) -> HRESULT;
    pub fn SafeArrayUnaccessData(sa: *mut SAFEARRAY) -> HRESULT;
}

pub const COINIT_MULTITHREADED: DWORD = 0;
pub const CLSCTX_INPROC_SERVER: DWORD = 1;
pub const RPC_C_AUTHN_WINNT: DWORD = 10;
pub const RPC_C_AUTHZ_NONE: DWORD = 0;
pub const RPC_C_AUTHN_LEVEL_CALL: DWORD = 3;
pub const RPC_C_IMP_LEVEL_IMPERSONATE: DWORD = 3;
pub const EOAC_NONE: DWORD = 0;

/// Let WMI impersonate us on calls through `proxy`. Windows' winmgmt runs out of process
/// and refuses (WBEM_E_ACCESS_DENIED) a proxy at the default identify level; an in-process
/// provider (wbemprox here) has no proxy, and the call fails harmlessly.
pub unsafe fn impersonate<T>(proxy: *mut T) {
    CoSetProxyBlanket(
        proxy as *mut IUnknown,
        RPC_C_AUTHN_WINNT,
        RPC_C_AUTHZ_NONE,
        core::ptr::null_mut(),
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        core::ptr::null_mut(),
        EOAC_NONE,
    );
}

pub const VT_EMPTY: u16 = 0;
pub const VT_NULL: u16 = 1;
pub const VT_I2: u16 = 2;
pub const VT_I4: u16 = 3;
pub const VT_R4: u16 = 4;
pub const VT_R8: u16 = 5;
pub const VT_BSTR: u16 = 8;
pub const VT_DISPATCH: u16 = 9;
pub const VT_BOOL: u16 = 11;
pub const VT_UNKNOWN: u16 = 13;
pub const VT_I1: u16 = 16;
pub const VT_UI1: u16 = 17;
pub const VT_UI2: u16 = 18;
pub const VT_UI4: u16 = 19;
pub const VT_I8: u16 = 20;
pub const VT_UI8: u16 = 21;
pub const VT_INT: u16 = 22;
pub const VT_UINT: u16 = 23;
pub const VT_ARRAY: u16 = 0x2000;
pub const VT_TYPEMASK: u16 = 0x0fff;

const fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> GUID {
    GUID { Data1: d1, Data2: d2, Data3: d3, Data4: d4 }
}
pub const CLSID_WBEM_LOCATOR: GUID =
    guid(0x4590f811, 0x1d3a, 0x11d0, [0x89, 0x1f, 0x00, 0xaa, 0x00, 0x4b, 0x2e, 0x24]);
pub const IID_IWBEM_LOCATOR: GUID =
    guid(0xdc12a687, 0x737f, 0x11cf, [0x88, 0x4d, 0x00, 0xaa, 0x00, 0x4b, 0x2e, 0x24]);
pub const IID_IWBEM_CLASS_OBJECT: GUID =
    guid(0xdc12a681, 0x737f, 0x11cf, [0x88, 0x4d, 0x00, 0xaa, 0x00, 0x4b, 0x2e, 0x24]);

/// A COM interface pointer released on drop.
pub struct ComPtr<T>(pub *mut T);
impl<T> ComPtr<T> {
    pub fn null() -> Self {
        ComPtr(core::ptr::null_mut())
    }
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // Every COM interface starts with IUnknown's vtable.
            unsafe {
                let unk = self.0 as *mut IUnknown;
                if let Some(release) = (*(*unk).lpVtbl).Release {
                    release(unk);
                }
            }
        }
    }
}

/// A BSTR freed on drop.
pub struct Bstr(pub BSTR);
impl Bstr {
    pub fn new(s: &[u16]) -> Self {
        // `s` is NUL-terminated (every wide string here is).
        Bstr(unsafe { SysAllocString(s.as_ptr()) })
    }
}
impl Drop for Bstr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { SysFreeString(self.0) }
        }
    }
}
