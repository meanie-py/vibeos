//! winlogon-rust — Rust translation of winlogon.exe's GINA-independent
//! leaf files (the logon manager: winlogon → userinit → explorer).
//!
//! PARTIAL cohabitation flip: only the leaf files
//! (setup/environment/shutdown/screensaver/security) move to Rust; the
//! GINA core (wlx.c), SAS state machine (sas.c), main (winlogon.c) and
//! RPC server (rpcserver.c) stay C. The still-C core owns the `WLSession`
//! global, which the flipped leaves read through the winlogon-sys
//! substrate (WLSESSION layout size-asserted ×3). Each `.c` is commented
//! out of base/system/winlogon/CMakeLists.txt once Rust parity holds; the
//! C stays on disk (translation philosophy: never delete the C).
//!
//! See docs/engineering/smss-csrss-translation-audit.md + memory
//! project_winlogon_track.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// Win32 predefined consts, macros, and the extern `WLSession` global the
// per-file translation can't resolve (not in winlogon-sys: they're macros
// / a C-core-owned global). Shared by the flipped leaf modules.
pub mod preamble;

// setup.c — GetSetupType / RunSetup (setup-mode shell launch). The spike.
pub mod setup;

// USER-MODE no_std staticlib → needs its own panic handler + eh_personality
// (same role as userinit-rust / smss-csrss-rust). Must NOT use privileged
// instructions.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
