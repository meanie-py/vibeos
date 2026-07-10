//! svcctl (SCM) RPC server stub — pure Rust, generated at build time from
//! `svcctl.idl` (see build.rs). The generated module instantiates the MIDL
//! static graph against `midl-rt` and exports `svcctl_v2_0_s_ifspec`; the
//! server routines (`R*`) + rundowns it references resolve at link from
//! services-rust, and `NdrServerCall2` / `MIDL_user_*` from rpcrt4 / the CRT.
#![no_std]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types, unused_unsafe)]

include!(env!("SVCCTL_STUB_RS"));

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// services-rust; deduped at link by --allow-multiple-definition). No panics
// actually occur here — the stub is data + address tables.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
