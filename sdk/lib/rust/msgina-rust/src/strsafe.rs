//! strsafe.h's bounded string ops — now the shared `strsafe-rust` crate.
//!
//! This file used to hold msgina's own port, and it was an APPROXIMATION rather than a port:
//! `StringCbCopyNExW` dropped the fill paths and the `STRSAFE_IGNORE_NULLS` handling and
//! computed `pcbRemaining` differently from the header's worker. The printf family was worse —
//! see the `extern` block in `lib.rs` for the stack overrun it caused. All of it is replaced by
//! the shared crate's faithful port of the header.
//!
//! Re-exported by NAME rather than with a glob: six files do `use crate::strsafe::*` alongside
//! `use crate::prelude::*`, and `strsafe-rust` publishes its own `WCHAR`/`HRESULT`/`DWORD`
//! aliases (it has to — it cannot depend on any module's island). Two globs offering the same
//! name is an ambiguity error at every use site, so only the API comes through here.
pub use strsafe_rust::{
    StringCbCopyNExW, StringCbCopyW, STRSAFE_E_INSUFFICIENT_BUFFER, STRSAFE_E_INVALID_PARAMETER,
};
