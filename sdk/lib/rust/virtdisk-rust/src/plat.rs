//! Platform seam: the two kernel32 queries virtdisk makes, faked under host tests so lib.rs's
//! validation order can be exercised without Windows.
#[cfg(not(test))]
mod imp {
    use core::ffi::c_void;
    extern "system" {
        fn GetHandleInformation(h: *mut c_void, flags: *mut u32) -> i32;
        fn CreateFileW(
            name: *const u16, access: u32, share: u32, sa: *mut c_void, disp: u32, attrs: u32,
            template: *mut c_void,
        ) -> *mut c_void;
        fn CloseHandle(h: *mut c_void) -> i32;
        fn GetLastError() -> u32;
    }

    /// True when `h` names an open kernel object in this process. NULL and INVALID_HANDLE_VALUE
    /// both fail GetHandleInformation with ERROR_INVALID_HANDLE, which is exactly the set
    /// Windows' virtdisk answers 6 for.
    pub unsafe fn handle_valid(h: *mut c_void) -> bool {
        let mut f = 0u32;
        GetHandleInformation(h, &mut f) != 0
    }

    /// Open `path` read-only with full sharing (so a mounted/in-use image is not refused for
    /// our own sake). Returns 0 when it opened, else the Win32 error of the failed open
    /// (2 not found, 3 path not found, 5 denied, 32 sharing violation, ...).
    pub unsafe fn probe_file(path: *const u16) -> u32 {
        const GENERIC_READ: u32 = 0x8000_0000;
        const SHARE_ALL: u32 = 7; // READ | WRITE | DELETE
        const OPEN_EXISTING: u32 = 3;
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
        let h = CreateFileW(
            path, GENERIC_READ, SHARE_ALL, core::ptr::null_mut(), OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut(),
        );
        if h as isize == -1 {
            return GetLastError();
        }
        CloseHandle(h);
        0
    }
}
#[cfg(test)]
mod imp {
    use core::ffi::c_void;
    /// Fake: every handle except NULL and INVALID_HANDLE_VALUE is open.
    pub unsafe fn handle_valid(h: *mut c_void) -> bool {
        !h.is_null() && h as isize != -1
    }
    fn s(p: *const u16) -> String {
        let mut v = Vec::new();
        let mut i = 0;
        unsafe {
            while *p.add(i) != 0 {
                v.push(*p.add(i));
                i += 1;
            }
        }
        String::from_utf16(&v).unwrap()
    }
    /// Fake file system: three files exist / fail in distinct ways, anything else is missing.
    pub unsafe fn probe_file(path: *const u16) -> u32 {
        match s(path).as_str() {
            "C:\\exists.vhd" => 0,
            "C:\\nodir\\x.vhd" => 3,
            "C:\\locked.vhd" => 32,
            _ => 2,
        }
    }
}
pub use imp::*;
