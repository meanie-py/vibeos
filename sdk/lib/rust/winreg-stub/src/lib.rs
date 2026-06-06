//! winreg RPC server stub — pure Rust, generated at build time from
//! `winreg.idl` (see build.rs). Exports `winreg_v1_0_s_ifspec`; the BaseReg* /
//! Open* server routines + `RPC_HKEY_rundown` resolve at link from
//! winlogon-rust, and `NdrServerCall2` / `MIDL_user_*` from rpcrt4 / the CRT.
#![no_std]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types, unused_unsafe)]

include!(env!("WINREG_STUB_RS"));

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
