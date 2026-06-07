use core::ffi::c_void;

extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn RosSymInit(callbacks: *const ROSSYM_CALLBACKS);
}

unsafe fn RosSymAllocMemUM(Size: usize) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0, Size)
}

unsafe fn RosSymFreeMemUM(Area: *mut core::ffi::c_void) {
    unsafe {
        HeapFree(GetProcessHeap(), 0, Area);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RosSymInitUserMode() {
    RosSymInit(core::ptr::addr_of!(KmCallbacks));
}
