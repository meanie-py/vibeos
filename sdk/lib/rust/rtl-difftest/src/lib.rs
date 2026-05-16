//! Differential harness scaffold for the rtl-strings track. The real
//! cross-checks live in `tests/diff.rs`; build.rs cc-compiles the
//! genuine isolated `sdk/lib/rtl/unicode.c` with public symbols
//! renamed `ref_*`. This lib exists so the workspace resolves.
