//! nt_0600/ldr/ — ntdll_vista.dll loader-support translations.
//! Mirrors the C layout `dll/ntdll/nt_0600/ldr/`. These are compiled into
//! `ntdll_vista_static` (linked into ntdll_vista.dll), NOT the main ntdll.dll
//! loader.

pub mod ldrinit;
