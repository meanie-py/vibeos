#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]
#![allow(dangerous_implicit_autorefs)]

//! usbport.sys — USB host-controller PORT framework. Faithful C→Rust cohab (rustconv opus;
//! 11 files c2rust-anchored, 3 (roothub/urb/device) pure-LLM where c2rust AST was unsupported).
//! The miniport substrate ehci/ohci/uhci register WITH; owns URB queue, endpoint/transfer
//! state, PnP FDO/PDO, roothub PDO, USB2 bandwidth scheduler. Exports USBPORT_GetHciMn /
//! USBPORT_RegisterUSBPortDriver (usbport.spec). NOT boot-loaded in smoke -> link-clean x3 bar.
// ASSERT is debug_assert! (ntkrnl_sys::dbgassert): checked in a debug build, never
// evaluated in release -- the kernel's convention. It was an empty fn here, which
// never checked yet still evaluated its argument on every call.
macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// Fold-proof DPRINT/TRACE shims — see drvtrace-shim/src/lib.rs (§9.99).
#[macro_use]
extern crate drvtrace_shim;

// Coerce an `extern "system"` fn item to a bindgen NTAPI callback slot
// (`Option<unsafe extern "stdcall" fn ...>` on i386; `Option<unsafe extern "C" fn ...>`
// on amd64/arm64). `extern "system"` IS that per-arch ABI, so the fn-pointer bit pattern
// is identical; only the nominal Rust type differs. The slot type is inferred from the
// assignment/argument context. (Same pattern as usbhub.) Crate-root so all mods see it.
macro_rules! cb {
    ($f:expr) => {
        core::mem::transmute($f as usize)
    };
}
pub(crate) use cb;

// Compile-time L"..." wide-string literal -> *const u16 (NUL-terminated static array).
// Replaces the pure-LLM `"x\0".encode_utf16().collect::<Vec<u16>>().as_ptr()` (no alloc in
// the kernel). ASCII-only, which all the usbport registry/symlink literals are.
macro_rules! wstr {
    ($s:literal) => {{
        const BYTES: &[u8] = $s.as_bytes();
        const N: usize = BYTES.len();
        const ARR: [u16; N] = {
            let mut arr = [0u16; N];
            let mut i = 0;
            while i < N {
                arr[i] = BYTES[i] as u16;
                i += 1;
            }
            arr
        };
        ARR.as_ptr() as *mut u16
    }};
}
pub(crate) use wstr;

pub mod types;
pub mod glue;
pub mod debug;
pub mod device;
pub mod endpoint;
pub mod iface;
pub mod ioctl;
pub mod iso;
pub mod msi;
pub mod pnp;
pub mod power;
pub mod queue;
pub mod roothub;
pub mod trfsplit;
pub mod urb;
pub mod usb2;
pub mod usbport;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
