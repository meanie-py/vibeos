//! nt_0600/ — ntdll_vista.dll companion module.
//!
//! Mirrors the C layout `dll/ntdll/nt_0600/`. ntdll_vista.dll is a
//! *separate* PE module (own .spec, own DllMain entry) loaded by the
//! Vista+ compatibility layer.

pub mod dllmain;
