use crate::prelude::*;
use core::ffi::c_void;



pub unsafe extern "C" fn cicMemAlloc(size: usize) -> LPVOID {
    return LocalAlloc(0, (size) as SIZE_T) as LPVOID;
}

pub unsafe extern "C" fn cicMemAllocClear(size: SIZE_T) -> LPVOID {
    return LocalAlloc(LMEM_ZEROINIT, size);
}

pub unsafe extern "C" fn cicMemFree(ptr: LPVOID) {
    if !ptr.is_null() {
        LocalFree(ptr);
    }
}

pub unsafe extern "C" fn cicMemReAlloc(ptr: LPVOID, newSize: usize) -> LPVOID {
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
    return ptr.is_null();
}

impl CicNoThrow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CicSystemModulePath {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            (*this_).m_szPath[0] = UNICODE_NULL;
            (*this_).m_cchPath = 0;
        }
    }
}
