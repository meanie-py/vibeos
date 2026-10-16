//! rtl/ — main ntdll.dll rtl-glue translations.
//! Mirrors the C layout `dll/ntdll/rtl/` (the `rtl_um` OBJECT lib). These
//! land in ntdll.dll proper (NOT the ntdll_vista companion).

pub mod uilist;
pub mod version;
pub mod openkeyex;
pub mod ktm;
pub mod libsupp;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub mod wow64shim; // WOW64 host surface, arm64+amd64 (docs/engineering/wow64-plan.md Phase A/E)
