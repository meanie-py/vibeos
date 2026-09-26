//! Wine JScript decoder, cohab-linked into jscript.dll.
//! Original source: dll/win32/jscript/decode.c. Other JScript units remain C.
#![no_std]

mod decode;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
