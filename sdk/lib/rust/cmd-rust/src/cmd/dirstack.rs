use crate::cmd::internal::SetRootPath;
use crate::cmd::error::error_out_of_memory;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct tagDIRENTRY {
    pub prev: *mut tagDIRENTRY,
    pub next: *mut tagDIRENTRY,
    pub szPath: [TCHAR; 1],
}

pub type DIRENTRY = tagDIRENTRY;
pub type LPDIRENTRY = *mut DIRENTRY;

#[allow(non_upper_case_globals)]
pub static mut nStackDepth: INT = 0;
#[allow(non_upper_case_globals)]
pub static mut lpStackTop: LPDIRENTRY = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut lpStackBottom: LPDIRENTRY = core::ptr::null_mut();

unsafe extern "C" fn PushDirectory(pszPath: LPTSTR) -> INT {
    let lpDir: *mut DIRENTRY = cmd_alloc!(FIELD_OFFSET!(DIRENTRY, szPath[_tcslen(pszPath) + 1])) as *mut DIRENTRY;
    if lpDir.is_null() {
        error_out_of_memory();
        return -1;
    }

    (*lpDir).prev = core::ptr::null_mut();
    (*lpDir).next = lpStackTop;
    if lpStackTop.is_null() {
        lpStackBottom = lpDir;
    } else {
        (*lpStackTop).prev = lpDir;
    }
    lpStackTop = lpDir;

    _tcscpy(core::ptr::addr_of_mut!((*lpDir).szPath) as LPTSTR, pszPath);

    nStackDepth += 1;

    nErrorLevel = 0;
    return nErrorLevel;
}

unsafe extern "C" fn PopDirectory() {
    let mut lpDir: LPDIRENTRY = lpStackTop;
    lpStackTop = (*lpDir).next;
    if lpStackTop != core::ptr::null_mut() {
        (*lpStackTop).prev = core::ptr::null_mut();
    } else {
        lpStackBottom = core::ptr::null_mut();
    }

    cmd_free!(lpDir as LPVOID);

    nStackDepth -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn InitDirectoryStack() {
    nStackDepth = 0;
    lpStackTop = core::ptr::null_mut();
    lpStackBottom = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn DestroyDirectoryStack() {
    while nStackDepth != 0 {
        PopDirectory();
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetDirectoryStackDepth() -> INT {
    return nStackDepth;
}
#[no_mangle]
pub unsafe extern "C" fn CommandPushd(rest: LPTSTR) -> INT {
    let mut curPath: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];

    if _tcsncmp(rest, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPuts!(STRING_DIRSTACK_HELP1);
        return 0 as i32;
    }

    GetCurrentDirectory(MAX_PATH, curPath.as_mut_ptr());

    if *rest.offset(0 as isize) != ('\0' as u16) {
        if SetRootPath(core::ptr::null_mut(), rest) == 0 {
            return 1 as i32;
        }
    }

    return PushDirectory(curPath.as_mut_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn CommandPopd(rest: LPTSTR) -> INT {
    let mut ret: INT = 0;
    if _tcsncmp(rest, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPuts!(STRING_DIRSTACK_HELP2);
        return 0;
    }

    if nStackDepth == 0 {
        return 1;
    }

    ret = (_tchdir(core::ptr::addr_of!((*lpStackTop).szPath) as *const TCHAR) != 0) as INT;
    PopDirectory();

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CommandDirs(rest: LPTSTR) -> INT {
    let mut lpDir: LPDIRENTRY = core::ptr::null_mut();

    if _tcsncmp(rest, (['/' as u16, '?' as u16, 0].as_ptr()), 2) == 0
    {
        ConOutResPuts!(STRING_DIRSTACK_HELP3);
        return 0;
    }

    nErrorLevel = 0;

    lpDir = lpStackBottom;

    if lpDir.is_null()
    {
        ConOutResPuts!(STRING_DIRSTACK_HELP4);
        return 0;
    }

    while !lpDir.is_null()
    {
        ConOutPuts!((*lpDir).szPath.as_mut_ptr());
        lpDir = (*lpDir).prev;
    }

    return 0;
}
