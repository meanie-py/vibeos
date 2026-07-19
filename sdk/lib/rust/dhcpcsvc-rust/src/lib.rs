//! dhcpcsvc.dll — Rust translation of base/services/dhcpcsvc (the DHCP client service DLL).
//! staticlib → libdhcpcsvc_rust.a, cohab-linked into the cmake-built dhcpcsvc.dll with the
//! translated .c commented out (per-file flip, like the winsock-family crates).
//!
//! The DhcpC* client API + a BSD-derived dhclient state machine (dhcp/) + a single MIDL
//! interface (dhcpcsvc.idl, Client_/Server_ stubs). The rosdhcp/dhcpd/dhcpcsvc struct layouts
//! are bindgen-generated per arch into `types` (gen_driver_types.sh profile: dhcpcsvc).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // rosdhcp/dhcpd/dhcpcsvc struct layouts (bindgen, per arch) + re-exported nt base
pub mod prelude; // the ONE import surface: types glob + CRT/registry/winsock/Rtl/MIDL externs

#[allow(clippy::module_inception)]
pub mod dhcp {
    pub mod adapter;  // adapter enumeration + config
    pub mod alloc;    // small allocator helpers
    pub mod api;      // the DhcpC* public API
    pub mod compat;   // win32 compat shims
    pub mod dhclient; // the BSD dhclient state machine (the big one)
    pub mod dispatch; // packet dispatch loop
    pub mod hash;     // the option/lease hash table
    pub mod options;  // DHCP option parse/format + the option tables
    pub mod socket;   // raw/UDP socket I/O
    pub mod tables;   // the DHCP option universe tables
    // tree.c (the ISC parse-tree evaluator) is NOT in base/services/dhcpcsvc/CMakeLists.txt SOURCE —
    // it is dead/excluded upstream source. DeepSeek translated it anyway (src/dhcp/tree.rs kept on
    // disk), but it references a fuller ISC `struct tree`/`struct tree_cache.tree`/`TREE_*`/
    // `dns_host_entry` that do NOT exist in ReactOS's tree.h (hence absent from the bindgen island),
    // plus the file-static `do_host_lookup` which DeepSeek dropped untranslated. It therefore cannot
    // compile against the real types. Excluded here to mirror the C build's file set (NOT to hide a
    // live regression — no other module references tree.rs's symbols). See integrator note.
    // pub mod tree;  // the config tree — excluded: tree.c not built (see above)
    pub mod util;     // misc helpers (iaddr, print, error)
}

#[allow(clippy::module_inception)]
pub mod dhcpcsvc {
    pub mod dhcpcsvc; // the service entry (ServiceMain) + RPC server glue
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
