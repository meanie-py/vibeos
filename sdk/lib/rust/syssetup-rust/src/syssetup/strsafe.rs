//! strsafe.h's bounded string ops — now the shared `strsafe-rust` crate.
//!
//! This file used to hold syssetup's own port of the header — the FIRST of four (syssetup, desk,
//! msgina, and sysdm about to be a fifth), and a strict subset: no Cat-by-bytes, no Length, no
//! PrintfEx. `STRSAFEAPI` is `static __inline`, so no object file in the tree defines these and
//! every flipped module had to port the header again; one of those copies had silently dropped
//! its bound (see the roadmap's bug registry for msgina's stack overrun).
//!
//! Kept as a re-export rather than deleted so every existing `crate::syssetup::strsafe::X` path
//! keeps resolving. Nothing was lost — `strsafe-rust` is a superset of all the copies.
//!
//! One convention change came with the move: the `*_v` twins now take a POINTER to the caller's
//! `va_list` (`&ap`), matching `varargs-rust`'s documented shape, so
//! `dll/win32/syssetup/rust_varargs_shim.c` passes `(void *)&ap` for the strsafe entry point.
pub use strsafe_rust::*;
