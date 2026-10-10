use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

pub static mut ContextListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

pub type Pwgl_context = *mut wgl_context;

#[repr(C)]
pub union wgl_dc_data_owner {
    pub hwnd: HWND,
    pub hdc: HDC,
    pub u: HANDLE,
}

pub type Pwgl_dc_data = *mut wgl_dc_data;
unsafe extern "C" fn IntGetCurrentDispatchTable() -> *const GLDISPATCHTABLE {
    return (*NtCurrentTeb()).glTable as *const GLDISPATCHTABLE;
}

unsafe extern "C" fn IntSetCurrentDispatchTable(table: *const GLDISPATCHTABLE) {
    (*NtCurrentTeb()).glTable = table as *mut core::ffi::c_void;
}

unsafe extern "C" fn IntMakeCurrent(mut hglrc: HGLRC, mut hdc: HDC, mut dc_data: *mut wgl_dc_data) {
    let mut CurrentTeb: *mut TEB = NtCurrentTeb() as *mut _TEB;
    (*CurrentTeb).glCurrentRC = hglrc as PVOID;
    (*CurrentTeb).glReserved2 = hdc as PVOID;
    (*CurrentTeb).glSectionInfo = dc_data as PVOID;
}

unsafe extern "C" fn IntGetCurrentRC() -> HGLRC {
    return (*NtCurrentTeb()).glCurrentRC as HGLRC;
}

unsafe extern "C" fn IntGetCurrentDC() -> HDC {
    return (*NtCurrentTeb()).glReserved2 as HDC;
}

unsafe extern "C" fn IntGetCurrentDcData() -> *mut wgl_dc_data {
    return (*NtCurrentTeb()).glSectionInfo as *mut wgl_dc_data;
}

unsafe extern "C" fn IntSetCurrentICDPrivate(value: *mut core::ffi::c_void) {
    (*NtCurrentTeb()).glContext = value;
}

unsafe extern "C" fn IntGetCurrentICDPrivate() -> *mut ::core::ffi::c_void {
    return (*NtCurrentTeb()).glContext as *mut ::core::ffi::c_void;
}

unsafe extern "C" fn IntGetCurrentDHGLRC() -> DHGLRC {
    let mut ctx: *mut wgl_context = IntGetCurrentRC() as *mut wgl_context;
    if ctx.is_null() {
        return core::ptr::null_mut();
    }
    return (*ctx).dhglrc;
}
