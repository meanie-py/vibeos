//! wldap32 — Rust translation of dll/win32/wldap32.
//! staticlib → libwldap32_rust.a, cohab-linked into the cmake-built wldap32
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 20 TU(s)
//! the build actually compiles. Generated sources (wldap32_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// TWO islands. The 19 PCH'd TUs compile against "winldap_private.h"; ber.c (the
// add_pch SKIP source) compiles against <winldap.h>, and the two declare ~30 ldap_*
// prototypes and LDAPControlA/LDAPModA/LDAPSortKeyA/LDAPAPIInfoA (+W) INCOMPATIBLY.
// There is no legal merged island (§9.108) — so NEVER glob both into one scope. The
// prelude globs `types`; ber.rs imports from `types_1` explicitly.
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod types; // bindgen island, per arch (gen_driver_types.sh wldap32)
pub mod types_1; // ber.c's island (TYPES_STEM=types_1 gen_driver_types.sh wldap32-types_1)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod wldap32 {
    // wldap32.h's 62 `static inline` marshalling helpers (strAtoW/strfreeW/
    // controlarray*/modarray*/sortkey*). In C every TU inlined a private copy, so no .c
    // defined them; here they are one shared module the other 20 call ~300 times.
    pub mod helpers;
    pub mod add;
    pub mod bind;
    pub mod compare;
    pub mod control;
    pub mod delete;
    pub mod dn;
    pub mod error;
    pub mod extended;
    pub mod init;
    pub mod main;
    pub mod misc;
    pub mod modify;
    pub mod modrdn;
    pub mod option;
    pub mod page;
    pub mod parse;
    pub mod rename;
    pub mod search;
    pub mod value;
    pub mod ber;
}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
