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

// Win32 predefined consts, macros, and the WLSession/hAppInstance globals
// (owned by winlogon.rs, re-exported here) the leaf modules share.
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod preamble;

// winlogon.c — the main: WinMain bootstrap (winsta/desktop, services+lsass
// spawn, GINA init, explorer spawn) + the WLSession/hAppInstance globals +
// WlStrDup/DisplayStatusMessage/RemoveStatusMessage. Second GINA/SAS/RPC core.
pub mod winlogon;

// setup.c — GetSetupType / RunSetup (setup-mode shell launch). The spike.
pub mod setup;

// environment.c — CreateUserEnvironment / BuildVolatileEnvironment. On the
// autologon boot path (SAS calls it post-logon); reads WLSession->UserToken
// + ->Profile, so it exercises the full WLSESSION layout at runtime.
pub mod environment;

// shutdown.c — StartSystemShutdown / TerminateSystemShutdown (RPC-invoked)
// + the countdown shutdown dialog. Link-checked (not on the boot path).
pub mod shutdown;

// screensaver.c — InitializeScreenSaver / StartScreenSaver + LL hook procs
// + the idle monitor thread. Link-checked (runs on real desktop idle).
pub mod screensaver;

// security.c — winsta/desktop security descriptors + per-session access
// grants. On the boot/logon path (wlx.c startup + sas.c logon).
pub mod security;

// notify.c — logon notification DLL registry walker + dispatcher
// (CallNotificationDlls). On the boot/logon path. Owns the (now-Rust)
// CallNotificationDlls that the other leaves call via the preamble.
pub mod notify;

// rpcserver.c — the winreg RPC server (StartRpcServer + 36 BaseReg*/Open*
// manager stubs + MIDL allocator glue + 2 shutdown forwarders). First of the
// 4 GINA/SAS/RPC cores flipped; the stub handlers feed the winreg-stub crate's
// SERVER_ROUTINES dispatch table.
pub mod rpcserver;

// wlx.c — the GINA interface: the WLX_DISPATCH_VERSION_1_4 function table given
// to msgina.dll, the GINA loader (GinaInit/LoadGina), the dialog tracking list,
// and CreateWindowStationAndDesktops. Third GINA/SAS/RPC core.
pub mod wlx;

// sas.c — the Secure Attention Sequence state machine (logon/logoff/shutdown,
// the SAS window proc, hotkeys, sound). Fourth/last GINA/SAS/RPC core.
pub mod sas;

// USER-MODE no_std staticlib → needs its own panic handler + eh_personality
// (same role as userinit-rust / smss-csrss-rust). Must NOT use privileged
// instructions.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
