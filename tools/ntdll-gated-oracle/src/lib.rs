//! Compiles the IN-TREE bodies, unmodified, for a Windows target so tests.rs can diff them
//! against the real ntdll. Every module below is the shipping file, by path.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

// rtl-strings provides the tree's RtlIntegerToUnicodeString / RtlUpcaseUnicodeChar /
// RtlInitUnicodeString, so locale.rs and name.rs are tested against OUR helpers, not
// Windows'. (tests::helpers_are_ours proves the binding.)
extern crate rtl_strings;

#[path = "../../../sdk/lib/rust/ntdll-rust/src/crypt.rs"]
pub mod crypt;
#[path = "../../../sdk/lib/rust/ntdll-rust/src/rtl/procnum.rs"]
pub mod procnum;
#[cfg(target_arch = "x86")]
#[path = "../../../sdk/lib/rust/ntdll-rust/src/rtl/slistex.rs"]
pub mod slistex;

/// rtl-sync's own `shared` substrate, which locale.rs imports as `crate::shared`.
#[path = "../../../sdk/lib/rust/rtl-sync/src/shared.rs"]
pub mod shared;
#[path = "../../../sdk/lib/rust/rtl-sync/src/locale.rs"]
pub mod locale;
#[path = "../../../sdk/lib/rust/rtl-sync/src/name.rs"]
pub mod name;
#[path = "../../../sdk/lib/rust/rtl-misc/src/memres.rs"]
pub mod memres;

#[cfg(test)]
mod tests;
