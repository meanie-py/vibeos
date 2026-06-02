//! rtl/ — main ntdll.dll rtl-glue translations.
//! Mirrors the C layout `dll/ntdll/rtl/` (the `rtl_um` OBJECT lib). These
//! land in ntdll.dll proper (NOT the ntdll_vista companion).

pub mod uilist;
pub mod version;
pub mod libsupp;
