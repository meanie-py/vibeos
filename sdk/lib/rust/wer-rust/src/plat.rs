//! Platform seam: heap + the excluded-application registry key + the report-table lock.
//! Host tests swap in std allocation and an in-memory "registry".
use core::sync::atomic::{AtomicBool, Ordering};

/// Tiny spin lock guarding the report table. Critical sections are a handful of pointer
/// writes, and it needs no kernel32 init (the DLL has no DllMain).
pub struct Lock(AtomicBool);
impl Lock {
    pub const fn new() -> Self {
        Lock(AtomicBool::new(false))
    }
    pub fn lock(&self) -> Guard<'_> {
        while self.0.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        Guard(self)
    }
}
pub struct Guard<'a>(&'a Lock);
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        (self.0).0.store(false, Ordering::Release);
    }
}

#[cfg(not(test))]
mod imp {
    use core::ffi::c_void;
    extern "system" {
        fn GetProcessHeap() -> *mut c_void;
        fn HeapAlloc(h: *mut c_void, flags: u32, size: usize) -> *mut c_void;
        fn HeapFree(h: *mut c_void, flags: u32, p: *mut c_void) -> i32;
        fn RegCreateKeyW(root: *mut c_void, sub: *const u16, out: *mut *mut c_void) -> i32;
        fn RegSetValueExW(k: *mut c_void, name: *const u16, res: u32, ty: u32, data: *const u8, cb: u32) -> i32;
        fn RegDeleteValueW(k: *mut c_void, name: *const u16) -> i32;
        fn RegCloseKey(k: *mut c_void) -> i32;
    }
    const HEAP_ZERO_MEMORY: u32 = 8;
    const REG_DWORD: u32 = 4;
    // HKEY_LOCAL_MACHINE / HKEY_CURRENT_USER are sign-extended 32-bit constants.
    const HKLM: *mut c_void = 0x8000_0002u32 as i32 as isize as *mut c_void;
    const HKCU: *mut c_void = 0x8000_0001u32 as i32 as isize as *mut c_void;

    /// Zeroed block; null on failure.
    pub unsafe fn alloc_zeroed(size: usize) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size) as *mut u8
    }
    pub unsafe fn free(p: *mut u8, _size: usize) {
        HeapFree(GetProcessHeap(), 0, p as *mut c_void);
    }

    unsafe fn open(all_users: bool, path: *const u16) -> Option<*mut c_void> {
        let mut k = core::ptr::null_mut();
        if RegCreateKeyW(if all_users { HKLM } else { HKCU }, path, &mut k) == 0 { Some(k) } else { None }
    }
    /// `None` = the key could not be created (E_ACCESSDENIED).
    pub unsafe fn exclude_set(all_users: bool, path: *const u16, name: *const u16) -> Option<()> {
        let k = open(all_users, path)?;
        let value: u32 = 1;
        RegSetValueExW(k, name, 0, REG_DWORD, &value as *const u32 as *const u8, 4);
        RegCloseKey(k);
        Some(())
    }
    /// `None` = key unavailable; `Some(true)` = value deleted; `Some(false)` = no such value.
    pub unsafe fn exclude_del(all_users: bool, path: *const u16, name: *const u16) -> Option<bool> {
        let k = open(all_users, path)?;
        let r = RegDeleteValueW(k, name);
        RegCloseKey(k);
        Some(r == 0)
    }
}
#[cfg(test)]
mod imp {
    use std::sync::Mutex;
    pub static REG: Mutex<Vec<(bool, Vec<u16>)>> = Mutex::new(Vec::new());
    /// Set to simulate an unwritable key.
    pub static DENY: Mutex<bool> = Mutex::new(false);

    pub unsafe fn alloc_zeroed(size: usize) -> *mut u8 {
        std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align(size, 16).unwrap())
    }
    pub unsafe fn free(p: *mut u8, size: usize) {
        std::alloc::dealloc(p, std::alloc::Layout::from_size_align(size, 16).unwrap())
    }
    unsafe fn name(p: *const u16) -> Vec<u16> {
        let mut v = Vec::new();
        let mut i = 0;
        while *p.add(i) != 0 {
            v.push(*p.add(i));
            i += 1;
        }
        v
    }
    pub unsafe fn exclude_set(all: bool, _path: *const u16, n: *const u16) -> Option<()> {
        if *DENY.lock().unwrap() {
            return None;
        }
        let n = name(n);
        let mut r = REG.lock().unwrap();
        if !r.iter().any(|(a, x)| *a == all && *x == n) {
            r.push((all, n));
        }
        Some(())
    }
    pub unsafe fn exclude_del(all: bool, _path: *const u16, n: *const u16) -> Option<bool> {
        if *DENY.lock().unwrap() {
            return None;
        }
        let n = name(n);
        let mut r = REG.lock().unwrap();
        let before = r.len();
        r.retain(|(a, x)| !(*a == all && *x == n));
        Some(r.len() != before)
    }
}
pub use imp::*;
