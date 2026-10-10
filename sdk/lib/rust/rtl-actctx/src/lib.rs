//! rtl-actctx: Rust translation of ReactOS `sdk/lib/rtl/actctx.c` — the
//! side-by-side (SxS) activation-context engine: the Wine-derived XML
//! manifest parser, the section-data builders, and the exported
//! `Rtl*ActivationContext*` NTAPI surface.
//!
//! `types.rs` holds the byte-exact mirrors of the private structs the
//! engine walks — both the in-memory bookkeeping (`struct assembly`,
//! `ACTIVATION_CONTEXT`, …) and the on-disk section-data layouts
//! (`strsection_header`, `string_index`, `*redirect_data`, …) that get
//! serialized into the activation-context blob. The size assertions in
//! `types.rs` are load-bearing: the blob layout is an ABI contract.
//!
//! Semantics track ReactOS rtl, not textbook.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
// The activation-context globals (system/process/implicit_actctx) are
// `static mut`, matching the C file-statics; access is from `unsafe` fns.
// rustconv-style transliteration keeps some explicit casts / dead locals.
#![allow(unused_unsafe, unused_parens)]

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate (only the `cabi`
// modules log).
#[cfg(feature = "cabi")]
#[macro_use]
extern crate winedebug_rust;

pub mod types;

#[cfg(feature = "cabi")]
#[macro_use]
pub mod shim;

#[cfg(feature = "cabi")]
pub mod strings;

#[cfg(feature = "cabi")]
pub mod actctx;
