//! No-op wine-debug formatter shims. mlang.dll does not link libwine, so the
//! translated TRACE paths' `debugstr_*` refs must resolve locally. Returning a
//! static empty string is the established no-op TRACE shim behavior (see
//! ole32-rust/src/debug_shims.rs).

#[no_mangle]
pub extern "C" fn debugstr_a(_s: *const i8) -> *const i8 {
    b"\0".as_ptr() as *const i8
}

#[no_mangle]
pub extern "C" fn debugstr_w(_s: *const u16) -> *const i8 {
    b"\0".as_ptr() as *const i8
}

#[no_mangle]
pub extern "C" fn debugstr_an(_s: *const i8, _n: i32) -> *const i8 {
    b"\0".as_ptr() as *const i8
}

#[no_mangle]
pub extern "C" fn debugstr_wn(_s: *const u16, _n: i32) -> *const i8 {
    b"\0".as_ptr() as *const i8
}

// mlang.dll does not link libwine; DllRegisterServer's resource registration
// becomes a no-op (S_OK), same effective behavior as the previously-dead path.
#[no_mangle]
pub extern "C" fn __wine_register_resources(_h: *mut core::ffi::c_void) -> i32 { 0 }

#[no_mangle]
pub extern "C" fn __wine_unregister_resources(_h: *mut core::ffi::c_void) -> i32 { 0 }
