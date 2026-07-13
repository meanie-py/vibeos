//! oleaut32-rust — Rust translation of dll/win32/oleaut32, cohab-linked into
//! the cmake-built oleaut32.dll (advapi32-rust pattern).
//!
//! Modules: hash/oleaut/safearray/variant/vartype/usrmarshal come from the
//! rustconv track (use crate::prelude::* + debug_shims); olesys/dispatch/
//! typelib come from the fable lane (self-contained hand-mirrored surface).
//! Strategy doc: docs/engineering/active-lanes/ole-com-translation-audit.md.
#![no_std]
#![feature(c_variadic)]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

// USER-MODE no_std staticlib → own panic handler + eh_personality (deduped at
// link by --allow-multiple-definition, same as advapi32-rust/ole32-rust).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

pub mod prelude;
pub mod debug_shims;
pub mod consts;
// Tier-D constants (harvested from the SDK headers, cited) + the IID/CLSID externs.
pub mod tierd_consts;
pub mod iids;
pub mod winemacros;
pub mod winemacros_gen;
pub mod commacros_gen;
// Non-COM Win32 ABI island (wincodec/olectl/prsht/wingdi) — bindgen, per-arch.
pub mod win_sys;

// rustconv track
pub mod hash;
pub mod oleaut;
pub mod safearray;
pub mod variant;
pub mod usrmarshal;
pub mod vartype;

// fable lane (Lane A/A2)
pub mod olesys;
pub mod dispatch;
pub mod typelib;

// rustconv track (Tier-D) — the last six C TUs of oleaut32
pub mod connpt;
pub mod olefont;
pub mod olepicture;
pub mod olepropframe;
pub mod recinfo;
pub mod varformat;
