//! netapi32.dll — Rust translation of dll/win32/netapi32 (the LAN Manager API DLL).
//! staticlib → libnetapi32_rust.a, cohab-linked into the cmake-built netapi32.dll with the
//! translated .c commented out (per-file flip, like the winsock-family crates).
//!
//! netapi32 implements the Net* LAN Manager APIs (NetUser/Group/LocalGroup/Share/Server/Wksta/
//! Dfs/Alert/Message) + the NetBIOS (nb30) surface. The lm* struct layouts are bindgen-generated
//! per arch into `types` (profile: netapi32). Translated with DeepSeek-V4-pro + c2rust anchors.
//!
//! Only the 14 c2rust-anchorable .c are flipped here; the 7 RPC-stub files (browser/dssetup/
//! misc/netlogon/schedule/srvsvc/wksta_new) keep their C bodies (owe a pure-LLM pass).
//!
//! Per-file cohab flip: each translated .c becomes a module here and its #[no_mangle]
//! symbols take over from the commented-out C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // lm*/nb30 struct layouts (bindgen, per arch) + re-exported nt base types
pub mod types_rpc; // MIDL wire types: srvsvc/wkssvc/atsvc/dssetup/netlogon (*_c.h, per arch)
pub mod types_browser; // MIDL wire types: browser_c.h — its own island (redeclares SERVER_INFO_100)
pub mod rpc_stubs; // MIDL client-stub PROTOTYPES for the types_rpc island (*_c.c stay C)
pub mod browser_stubs; // MIDL client-stub PROTOTYPES for the types_browser island
pub mod prelude_common; // island-agnostic half: wine-debug/CRT/winsock externs, consts, aliases
pub mod prelude_rpc; // prelude_common + types_rpc  + rpc_stubs (srvsvc/wkssvc/atsvc/dssetup/netlogon)
pub mod prelude_browser; // prelude_common + types_browser + browser_stubs (browser.rs only)
pub mod prelude; // the ONE import surface: types glob + enum aliases + skipped consts + bridges

// The 14 c2rust-anchored translation units. `netapi32` mirrors the C layout (dll/win32/netapi32/).
#[allow(clippy::module_inception)]
pub mod netapi32 {
    pub mod apibuf; // NetApiBufferAllocate/Free/Reallocate/Size
    pub mod dfs; // NetDfs* (the one RPC call NetrDfsAdd + DFS_INFO marshalling)
    pub mod display; // NetQueryDisplayInformation
    pub mod group_new; // NetGroup* (global groups)
    pub mod local_group; // NetLocalGroup* (local groups + members)
    pub mod nbcmdqueue; // the NetBIOS async command queue
    pub mod nbnamecache; // the NetBIOS name cache
    pub mod nbt; // NetBIOS-over-TCP transport helpers
    pub mod netapi32; // NETAPI_IsLocalComputer + the module glue
    pub mod netbios; // the Netbios() NCB dispatch
    pub mod obsolete; // the obsolete/compat Net* entry points
    pub mod user; // NetUser* (accounts) — the biggest unit
    pub mod utils; // Netp*/wprint_* internal helpers
    pub mod wksta; // NetWksta*/NetWkstaTransportEnum

    // The 7 RPC-stub tail units (pure-LLM translated — c2rust cannot anchor a MIDL client).
    // Their wire types come from the *_c.h WIDL stubs, which do NOT coexist with the <lm.h>
    // master header the `types` island above is built from (both declare AT_INFO/
    // CONNECTION_INFO_*/SERVER_INFO_*/WKSTA_INFO_*). They therefore import `types_rpc` /
    // `types_browser` instead — see the gen_driver_types.sh netapi32-rpc profile.
    pub mod browser; // I_BrowserrQueryOtherDomains  (types_browser: browser_c.h)
    pub mod dssetup; // DsRoleGetPrimaryDomainInformation
    pub mod misc; // NetShutdown*/NetRemoteTOD/NetServer* (srvsvc + wkssvc)
    pub mod netlogon; // NetrLogon*/DsGetDcName — the biggest tail unit
    pub mod schedule; // NetSchedule* (atsvc)
    pub mod srvsvc; // NetShare*/NetSession*/NetConnection*/NetFile*
    pub mod wksta_new; // NetWkstaGetInfo/NetWkstaUserEnum (wkssvc)
}

// staticlib (libnetapi32_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
