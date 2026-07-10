//! rtl/ — main ntdll.dll rtl-glue translations.
//! Mirrors the C layout `dll/ntdll/rtl/` (the `rtl_um` OBJECT lib). These
//! land in ntdll.dll proper (NOT the ntdll_vista companion).

pub mod uilist;
pub mod version;
pub mod libsupp;
#[cfg(target_arch = "aarch64")]
pub mod wow64shim; // WOW64/FEX host surface (docs/engineering/wow64-plan.md Phase A)
