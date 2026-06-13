//! The trivial scalar CRT globals scattered across the mingw startup
//! sources. Names/decoration match the C (verified vs nm of libmsvcrtex.a).

// startup/mingw_helpers.c — 0:console, 1:windows.
#[no_mangle]
pub static mut mingw_app_type: i32 = 0;

// startup/_newmode.c
#[no_mangle]
pub static mut _newmode: i32 = 0;

// startup/wildcard.c — command-line globbing disabled by default.
#[no_mangle]
pub static mut _dowildcard: i32 = 0;

// startup/reactos.c
#[no_mangle]
pub static mut _MINGW_INSTALL_DEBUG_MATHERR: i32 = 0;

// misc/fltused.c — referenced by the linker when the image uses floats.
#[no_mangle]
#[used]
pub static _fltused: i32 = 0x9875;

// startup/charmax.c
#[no_mangle]
pub static mut mingw_initcharmax: i32 = 0;
#[no_mangle]
pub static mut _charmax: i32 = 255;

// startup/tlsmcrt.c — 2 = do TLS cleanup at runtime (static libgcc).
#[no_mangle]
pub static mut _CRT_MT: i32 = 2;

// startup/tlssup.c — "force linked" flags toggled by check_managed_app.
#[no_mangle]
pub static mut mingw_initltsdrot_force: i32 = 0;
#[no_mangle]
pub static mut mingw_initltsdyn_force: i32 = 0;
#[no_mangle]
pub static mut mingw_initltssuo_force: i32 = 0;

// startup/tlssup.c — TLS index slot, filled in by the loader.
#[no_mangle]
pub static mut _tls_index: u32 = 0;
