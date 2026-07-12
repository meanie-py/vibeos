//! ole32-rust — Rust translation of dll/win32/ole32's runtime core, cohab-linked
//! into the cmake-built ole32.dll (advapi32-rust pattern: staticlib first in the
//! link, translated .c TUs commented out in the CMakeLists, lld first-match).
//!
//! Coverage (2026-07-11): compobj.c (COM runtime: CoInitialize/apartments/
//! activation/DllMain) + the marshaling cluster (marshal.c, stubmanager.c,
//! rpc.c, ftmarshal.c, git.c). The mechanical ring (ifs/streams/monikers/
//! clipboard/...) arrives via the rustconv track; storage32/OLE-embedding are a
//! later pass. Lane ledgers: ~/vibeos-rust-staging/ole32-lane-{a,b}/MANIFEST.md;
//! strategy: docs/engineering/active-lanes/ole-com-translation-audit.md.
#![no_std]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

// USER-MODE no_std staticlib → own panic handler + eh_personality (deduped at
// link by --allow-multiple-definition, same as advapi32-rust/lsasrv-rust).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

#[path = "compobj/mod.rs"]
pub mod compobj;

pub mod marshal;
