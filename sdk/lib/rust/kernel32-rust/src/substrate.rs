//! substrate.rs — the FFI surface kernel32-rust builds on.
//!
//! Most types/functions come straight from ntdll-sys (the same substrate
//! ntdll-rust / smss-csrss-rust use): the Native API (Nt*), Rtl*, Ldr*, Dbg*,
//! the Csr client, PEB / RTL_USER_PROCESS_PARAMETERS / STARTUPINFOW /
//! PROCESS_BASIC_INFORMATION / CLIENT_ID, the BASE_API_MESSAGE csr message
//! union (BASE_CREATE_PROCESS etc.) and the STATUS_* constants. They live under
//! `ntdll_sys::nt` (NOT the crate root).
//!
//! A handful of kernel32-INTERNAL types live only in
//! dll/win32/kernel32/include/{kernel32.h,vdm.h} and are mirrored in
//! `crate::k32_types`. The cohab-imported `Base*` helpers (defined in the OTHER
//! kernel32 .c files that stay C) and the cross-file Win32 exports proc.c calls
//! are declared in `crate::k32_externs`. proc.c's own globals live in
//! `crate::k32_globals`.

pub use ntdll_sys::nt::*;

pub use crate::k32_externs::*;
pub use crate::k32_globals::*;
pub use crate::k32_types::*;

// bindgen emits `BOOL = i32` but `TRUE`/`FALSE` as `u32` — so `return FALSE`
// from a `-> BOOL` fn mismatches. These explicit BOOL-typed constants shadow the
// glob-imported u32 ones for every module that `use crate::substrate::*`.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
