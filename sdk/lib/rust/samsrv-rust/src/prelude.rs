//! Crate-wide prelude of the universal symbols the per-file translations expect
//! in scope: the NT base substrate (ntdll_sys::nt), the SAMPR_*/RPC_* marshalled
//! types (samsrv-sys), shared LSA SD/SID/policy types (lsasrv-sys), CRT + Wine
//! wide-string helpers, MIDL allocators, and no-op Wine debug shims. Each
//! translated file adds `use crate::prelude::*;`. Grown as files flip (mirror
//! lsasrv-rust/src/prelude.rs). Curated re-exports (not globs) where samsrv-sys /
//! lsasrv-sys base types would collide with ntdll_sys::nt.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;

// samsrv consts + crate-wide helpers grow here during integration.
pub use crate::consts::*;

// MIDL allocators (RPC out-param marshalling). __RPC_USER = stdcall on i386.
extern "system" {
    pub fn MIDL_user_allocate(size: usize) -> *mut core::ffi::c_void;
    pub fn MIDL_user_free(ptr: *mut core::ffi::c_void);
}
