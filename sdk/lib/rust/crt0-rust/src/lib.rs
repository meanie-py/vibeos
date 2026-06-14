//! startup/crt0_c.c + startup/crt0_w.c — the default `main`/`wmain` that
//! forward to a GUI image's WinMain/wWinMain. See Cargo.toml for why this is
//! its own crate. The `__mingw_winmain_*` globals are owned by
//! crt-startup-rust (entry.rs); WinMain/wWinMain are consumer-provided.
#![no_std]
#![allow(non_snake_case)]

use core::ffi::c_void;

type HINSTANCE = *mut c_void;
type DWORD = u32;

extern "C" {
    static __mingw_winmain_hInstance: HINSTANCE;
    static __mingw_winmain_lpCmdLine: *mut u8;
    static __mingw_winmain_nShowCmd: DWORD;
}

#[cfg(feature = "ansi")]
#[no_mangle]
pub unsafe extern "C" fn main(_flags: i32, _cmdline: *mut *mut u8, _inst: *mut *mut u8) -> i32 {
    extern "system" {
        fn WinMain(h: HINSTANCE, prev: HINSTANCE, cmd: *mut u8, show: i32) -> i32;
    }
    WinMain(
        __mingw_winmain_hInstance,
        core::ptr::null_mut(),
        __mingw_winmain_lpCmdLine,
        __mingw_winmain_nShowCmd as i32,
    )
}

#[cfg(feature = "wide")]
#[no_mangle]
pub unsafe extern "C" fn wmain(_flags: i32, _cmdline: *mut *mut u16, _inst: *mut *mut u16) -> i32 {
    extern "system" {
        fn wWinMain(h: HINSTANCE, prev: HINSTANCE, cmd: *mut u16, show: i32) -> i32;
    }
    wWinMain(
        __mingw_winmain_hInstance,
        core::ptr::null_mut(),
        __mingw_winmain_lpCmdLine as *mut u16,
        __mingw_winmain_nShowCmd as i32,
    )
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
