//! Idiomatic, slice-based, `no_std`, no-`unsafe` core. THE
//! implementation; `cabi` shims only adapt the NT counted-string ABI
//! to these. Faithful to ReactOS `sdk/lib/rtl/unicode.c`.

pub mod ustr;
