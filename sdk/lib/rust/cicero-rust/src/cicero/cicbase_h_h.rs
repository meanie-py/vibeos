use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations (types, #defines, enums, or tables) in this preamble.
// The only item is the static function `cicGetFN__T0`, which belongs to the per-function pass.
// It references `T_FN` and `cicLoadSystemLibrary`, which are not declared here.




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
pub unsafe extern "C" fn cicIsNullPtr(ptr: LPCVOID) -> bool {
    return ptr.is_null();
}

pub unsafe extern "system" fn cicGetFN__T1(hinstDLL: *mut HINSTANCE, r#fn: *mut *mut core::ffi::c_void, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if !(*r#fn).is_null() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *r#fn = GetProcAddress(*hinstDLL, pszFuncName) as *mut core::ffi::c_void;
    return if (*r#fn).is_null() { 0 } else { 1 };
}
pub unsafe extern "system" fn cicGetFN__T2(hinstDLL: &mut HINSTANCE, fnp: &mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *const tagRECT, *const tagRECT) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> i32 {
    if fnp.is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *fnp = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return fnp.is_some() as i32;
}
pub unsafe extern "system" fn cicGetFN__T3(hinstDLL: *mut HINSTANCE, r#fn: *mut Option<unsafe extern "system" fn(*mut HWND__, *const HDC__, *mut tagRECT) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if (*r#fn).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return ((*r#fn).is_some()) as BOOL;
}

pub unsafe extern "system" fn cicGetFN__T4(hinstDLL: *mut HINSTANCE, r#fn: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *const u16, i32, u32, u32, *const tagRECT) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> i32 {
    if (*r#fn).is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return ((*r#fn).is_some()) as i32;
}
pub unsafe extern "system" fn cicGetFN__T5(
    hinstDLL: *mut HINSTANCE,
    r#fn: *mut Option<
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *mut HDC__,
            i32,
            i32,
            *const tagRECT,
            *mut _IMAGELIST,
            i32,
        ) -> i32,
    >,
    pszDllName: LPCTSTR,
    pszFuncName: LPCSTR,
) -> BOOL {
    if (*r#fn).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*r#fn).is_some() as BOOL;
}

pub unsafe extern "system" fn cicGetFN__T6(hinstDLL: *mut HINSTANCE, fn_: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *const tagRECT, *mut tagRECT) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> i32 {
    if (*fn_).is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *fn_ = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*fn_).is_some() as i32;
}
pub unsafe extern "system" fn cicGetFN__T7(hinstDLL: &mut HINSTANCE, fn_: &mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *const u16, i32, u32, *const tagRECT, *mut tagRECT) -> i64>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if fn_.is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *fn_ = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return fn_.is_some() as BOOL;
}
pub unsafe extern "system" fn cicGetFN__T8(hinstDLL: *mut HINSTANCE, fn_: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *mut tagRECT, THEMESIZE, *mut tagSIZE) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> i32 {
    if (*fn_).is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *fn_ = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*fn_).is_some() as i32;
}

pub unsafe extern "system" fn cicGetFN__T9(
    hinstDLL: *mut HINSTANCE,
    r#fn: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, *const tagRECT, u32, u32, *mut tagRECT) -> i32>,
    pszDllName: LPCTSTR,
    pszFuncName: LPCSTR,
) -> i32 {
    if (*r#fn).is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return ((*r#fn).is_some()) as i32;
}
pub unsafe extern "system" fn cicGetFN__T10(hinstDLL: *mut HINSTANCE, r#fn: *mut FARPROC, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if (*r#fn).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    // The prelude declares GetProcAddress as returning `*mut c_void`; FARPROC is its
    // nullable fn-pointer spelling, the same bits.
    *r#fn = core::mem::transmute::<*mut c_void, FARPROC>(GetProcAddress(*hinstDLL, pszFuncName));
    return (*r#fn).is_some() as BOOL;
}

pub unsafe extern "system" fn cicGetFN__T11(hinstDLL: *mut HINSTANCE, r#fn: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, i32, *mut tagRECT, *mut _MARGINS) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> i32 {
    if (*r#fn).is_some() {
        return (TRUE as i32);
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE as i32);
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*r#fn).is_some() as i32;
}

pub unsafe extern "system" fn cicGetFN__T12(hinstDLL: &mut HINSTANCE, r#fn: &mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, *mut HDC__, i32, i32, i32, *mut tagLOGFONTW) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if (*r#fn).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *r#fn = Some(core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName)));
    return (*r#fn).is_some() as BOOL;
}
// The cache slot is `Option`: C keeps it NULL until the first lookup, and a bare `fn` pointer
// that holds NULL is undefined behaviour (and its null test compiles to `false`).
pub unsafe extern "system" fn cicGetFN__T13(hinstDLL: *mut HINSTANCE, r#fn: *mut Option<unsafe extern "system" fn(LPVOID, i32) -> u32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if (*r#fn).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *r#fn = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*r#fn).is_some() as BOOL;
}

pub unsafe extern "system" fn cicGetFN__T14(hinstDLL: *mut HINSTANCE, fn_: *mut Option<unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> i32>, pszDllName: LPCTSTR, pszFuncName: LPCSTR) -> BOOL {
    if (*fn_).is_some() {
        return (TRUE) as BOOL;
    }
    if (*hinstDLL).is_null() {
        *hinstDLL = cicLoadSystemLibrary(pszDllName, (FALSE) as BOOL);
    }
    if (*hinstDLL).is_null() {
        return (FALSE) as BOOL;
    }
    *fn_ = core::mem::transmute(GetProcAddress(*hinstDLL, pszFuncName));
    return (*fn_).is_some() as BOOL;
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
