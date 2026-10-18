//! ATL's `CComBSTR` (sdk/lib/atl/atlcomcli.h): an owned `BSTR`, freed with `SysFreeString`.
//!
//! A BSTR is a `WCHAR*` (with its length prefix before it) on every target, so this lives in
//! the shared crate. Feature `bstr`: oleaut32, which a module without BSTRs never imports.
//! Ported members are the ones the cluster's C++ uses (acppage: `LoadString`, `Detach`,
//! `Attach`, the destructor) plus the trivial core; anything else is added when a module
//! names it, not guessed at.

use core::ffi::c_void;

pub type BSTR = *mut u16;

extern "system" {
    fn SysAllocString(psz: *const u16) -> BSTR;
    fn SysAllocStringLen(str_in: *const u16, ui: u32) -> BSTR;
    fn SysFreeString(bstr: BSTR);
    fn SysStringLen(bstr: BSTR) -> u32;
    fn LoadStringW(h: *mut c_void, id: u32, buf: *mut u16, n: i32) -> i32;
}

#[repr(C)]
pub struct CComBSTR {
    pub m_str: BSTR,
}

impl CComBSTR {
    /// `CComBSTR()`.
    pub const fn new() -> Self {
        CComBSTR { m_str: core::ptr::null_mut() }
    }

    /// `CComBSTR(LPCOLESTR pSrc)`: NULL stays NULL.
    pub unsafe fn from_ptr(src: *const u16) -> Self {
        CComBSTR { m_str: if src.is_null() { core::ptr::null_mut() } else { SysAllocString(src) } }
    }

    /// `operator BSTR() const`.
    pub fn as_ptr(&self) -> BSTR {
        self.m_str
    }

    /// `BSTR* operator&()`: for a callee that fills it.
    pub fn as_out(&mut self) -> *mut BSTR {
        &mut self.m_str
    }

    /// `void Attach(BSTR)`: frees the current string, then owns `bstr`.
    pub unsafe fn Attach(&mut self, bstr: BSTR) {
        SysFreeString(self.m_str);
        self.m_str = bstr;
    }

    /// `BSTR Detach()`: gives up ownership.
    pub fn Detach(&mut self) -> BSTR {
        core::mem::replace(&mut self.m_str, core::ptr::null_mut())
    }

    /// `bool LoadString(HMODULE module, DWORD uID)`: `LoadStringW` with a zero buffer size
    /// returns a read-only pointer into the resource itself, which is copied.
    pub unsafe fn LoadString(&mut self, module: *mut c_void, uid: u32) -> bool {
        SysFreeString(self.m_str);
        self.m_str = core::ptr::null_mut();
        let mut ptr: *const u16 = core::ptr::null();
        let len = LoadStringW(module, uid, &mut ptr as *mut *const u16 as *mut u16, 0);
        if len != 0 {
            self.m_str = SysAllocStringLen(ptr, len as u32);
        }
        !self.m_str.is_null()
    }

    /// `unsigned int Length() const`.
    pub unsafe fn Length(&self) -> u32 {
        SysStringLen(self.m_str)
    }
}

impl Default for CComBSTR {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CComBSTR {
    /// `~CComBSTR()`.
    fn drop(&mut self) {
        unsafe { SysFreeString(self.m_str) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static FREED: Mutex<Vec<usize>> = Mutex::new(Vec::new());
    static RES: [u16; 6] = [b'T' as u16, b'i' as u16, b't' as u16, b'l' as u16, b'e' as u16, 0];

    // Host stand-ins: a BSTR is a leaked Vec with a NUL, enough to check ownership.
    #[no_mangle]
    extern "system" fn SysAllocStringLen(s: *const u16, n: u32) -> BSTR {
        let mut v: Vec<u16> = unsafe { core::slice::from_raw_parts(s, n as usize) }.to_vec();
        v.push(0);
        Box::leak(v.into_boxed_slice()).as_mut_ptr()
    }
    #[no_mangle]
    extern "system" fn SysAllocString(s: *const u16) -> BSTR {
        let mut n = 0;
        while unsafe { *s.add(n) } != 0 {
            n += 1;
        }
        SysAllocStringLen(s, n as u32)
    }
    #[no_mangle]
    extern "system" fn SysFreeString(b: BSTR) {
        if !b.is_null() {
            FREED.lock().unwrap().push(b as usize);
        }
    }
    #[no_mangle]
    extern "system" fn SysStringLen(b: BSTR) -> u32 {
        let mut n = 0;
        while !b.is_null() && unsafe { *b.add(n) } != 0 {
            n += 1;
        }
        n as u32
    }
    #[test]
    fn detach_attach_and_drop_free_exactly_once() {
        unsafe {
            let mut title = CComBSTR::from_ptr(RES.as_ptr());
            assert_eq!(title.Length(), 5);
            // acppage's constructor: the title goes into the PROPSHEETPAGE, owned by nobody...
            let raw = title.Detach();
            assert!(title.as_ptr().is_null());
            drop(title);
            assert!(!FREED.lock().unwrap().contains(&(raw as usize)));
            // ...and its destructor takes it back, so the drop frees it.
            let mut back = CComBSTR::new();
            back.Attach(raw);
            drop(back);
            assert_eq!(FREED.lock().unwrap().iter().filter(|p| **p == raw as usize).count(), 1);
            assert!(CComBSTR::from_ptr(core::ptr::null()).as_ptr().is_null());
        }
    }

    #[test]
    fn a_missing_string_loads_as_null() {
        // lib.rs's host LoadStringW shim finds nothing: ATL's LoadString then holds NULL.
        unsafe {
            let mut s = CComBSTR::from_ptr(RES.as_ptr());
            assert!(!s.LoadString(core::ptr::null_mut(), 8));
            assert!(s.as_ptr().is_null());
        }
    }
}
