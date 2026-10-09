use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
}

// ---- file-scope preamble (translated) ----
pub static mut hApplet: HINSTANCE = core::ptr::null_mut();

pub const ADDON_GECKO: addon_t = 0;
pub const ADDON_MONO: addon_t = 1;

pub static mut hInst: HINSTANCE = core::ptr::null_mut();

pub static mut __wine_dbch_appwiz: __wine_debug_channel = __wine_debug_channel {
    flags: 0xff,
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"appwiz\0\0\0\0\0\0\0\0\0") },
};

unsafe extern "C" fn heap_alloc(len: usize) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), 0 as DWORD, len as SIZE_T) as *mut core::ffi::c_void;
}

unsafe extern "C" fn heap_realloc(mut mem: *mut ::core::ffi::c_void, mut len: size_t) -> *mut ::core::ffi::c_void {
    return HeapReAlloc(GetProcessHeap(), 0, mem as PVOID, len as SIZE_T) as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn heap_free(mut mem: *mut ::core::ffi::c_void) -> BOOL {
    return HeapFree(GetProcessHeap(), 0 as DWORD, mem as PVOID);
}

unsafe extern "C" fn heap_strdupAtoW(str: *const core::ffi::c_char) -> *mut u16 {
    let mut ret: *mut u16 = core::ptr::null_mut();
    if !str.is_null() {
        let mut len: usize = 0;
        len = MultiByteToWideChar(CP_ACP as u32, 0 as u32, str as *const i8, -(1 as i32), core::ptr::null_mut(), 0 as i32) as usize;
        ret = heap_alloc(len.wrapping_mul(core::mem::size_of::<u16>())) as *mut u16;
        if !ret.is_null() {
            MultiByteToWideChar(CP_ACP as u32, 0 as u32, str as *const i8, -(1 as i32), ret, len as i32);
        }
    }
    return ret;
}
