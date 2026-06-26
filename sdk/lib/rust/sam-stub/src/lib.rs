//! samr RPC server stub — pure Rust, generated at build time from `sam.idl`
//! (see build.rs). The generated module instantiates the MIDL static graph
//! against `midl-rt` and exports `samr_v1_0_s_ifspec`; the server routines
//! (`Samr*`) + rundowns it references resolve at link from the samsrv bodies
//! (C cohab today, Rust as files flip), and `NdrServerCall2` / `MIDL_user_*`
//! from rpcrt4 / the CRT.
#![no_std]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types, unused_unsafe)]

include!(env!("SAM_STUB_RS"));

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role as
// lsa-stub; deduped at link by --allow-multiple-definition when another Rust
// staticlib provides them too). No panics actually occur here — the stub is data
// + address tables.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
