use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define cicAssert assert  (macro alias for assert; not emitted)

// #define cicNoThrow CicNoThrow{}
pub const cicNoThrow: CicNoThrow = CicNoThrow { _address: 0 }; // bindgen sizes the empty tag type to one byte


pub type PCicSystemModulePath = *mut CicSystemModulePath;

unsafe extern "C" fn cicMemAlloc(size: SIZE_T) -> LPVOID {
    return LocalAlloc(0, size);
}

unsafe extern "C" fn cicMemAllocClear(size: usize) -> LPVOID {
    LocalAlloc(LMEM_ZEROINIT, (size) as SIZE_T)
}

unsafe extern "C" fn cicMemFree(ptr: LPVOID) {
    if !ptr.is_null() {
        LocalFree(ptr);
    }
}
unsafe extern "C" fn cicMemReAlloc(ptr: LPVOID, newSize: usize) -> LPVOID {
    if newSize == 0 {
        cicMemFree(ptr);
        return core::ptr::null_mut();
    }
    if ptr.is_null() {
        return LocalAlloc(LMEM_ZEROINIT, (newSize) as SIZE_T);
    }
    return LocalReAlloc(ptr, (newSize) as SIZE_T, LMEM_ZEROINIT | LMEM_MOVEABLE);
}

unsafe extern "C" fn cicIsNullPtr(ptr: LPCVOID) -> bool {
    ptr.is_null()
}

/// Generic, so a Rust fn (an `extern "C"` one may not be generic in a useful way, and the
/// C++ template has no symbol to keep). `hinstDLL`/`fn` are the C++ references.
pub unsafe fn cicGetFN<T_FN: Copy>(
    hinstDLL: *mut HINSTANCE,
    r#fn: *mut T_FN,
    pszDllName: LPCTSTR,
    pszFuncName: LPCSTR,
) -> BOOL {
    if core::ptr::read(r#fn as *const usize) != 0 {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary((pszDllName).cast(), (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    let p = GetProcAddress(*hinstDLL, pszFuncName);
    core::ptr::write(r#fn, core::ptr::read(core::ptr::addr_of!(p) as *const T_FN));
    return (core::ptr::read(r#fn as *const usize) != 0) as BOOL;
}
