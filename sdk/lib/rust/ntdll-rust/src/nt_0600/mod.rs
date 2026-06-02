//! nt_0600/ — ntdll_vista.dll companion module.
//!
//! Mirrors the C layout `dll/ntdll/nt_0600/`. ntdll_vista.dll is a
//! *separate* PE module (own .spec, own DllMain entry) loaded by the
//! Vista+ compatibility layer.

// DllMain is ntdll_vista.dll's ENTRY point — vista-only, must not land in
// ntdll.dll (whose CRT/entry never references it).
#[cfg(feature = "vista")]
pub mod dllmain;
// LdrpHeap (ldr/ldrinit.rs) is referenced by the MAIN ntdll.dll loader too
// (it resolved from ntdll_vista_static historically), so it stays UNgated —
// compiled into both the default and vista builds.
pub mod ldr;
