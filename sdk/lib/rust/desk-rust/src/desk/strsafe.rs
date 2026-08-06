//! strsafe.h's bounded string ops — now the shared `strsafe-rust` crate.
//!
//! This file used to hold desk's own port of the header. It was the SECOND of four (msgina,
//! syssetup, desk, and sysdm about to be a fifth), which is the detection event the crate exists
//! for: `STRSAFEAPI` is `static __inline`, so no object file in the tree defines these and every
//! flipped module had to port the header again. One of those copies had silently dropped its
//! bound — see the roadmap's bug registry for msgina's stack overrun.
//!
//! Kept as a re-export rather than deleted so every existing `crate::desk::strsafe::X` path — and
//! the prelude's TCHAR aliases built on them — keeps resolving. Nothing was lost: `strsafe-rust`
//! carries this file's full contents plus the Copy-N-Ex family the other copies needed.
//!
//! One convention change came with the move: the `*_v` twins now take a POINTER to the caller's
//! `va_list` (`&ap`), matching `varargs-rust`'s documented shape, so `dll/cpl/desk/
//! rust_varargs_shim.c` passes `(void *)&ap`. `va_list` is a single `char*` on all three of our
//! Windows targets but an array-of-struct on SysV x86-64, and the by-pointer form is correct on
//! both.
pub use strsafe_rust::*;
