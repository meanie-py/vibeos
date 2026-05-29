//! ldr/ — main ntdll.dll loader translations.
//! Mirrors the C layout `dll/ntdll/ldr/`. Lands in ntdll.dll proper.
//! (Distinct from `nt_0600::ldr`, the ntdll_vista companion loader glue.)

pub mod verifier;
