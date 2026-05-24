//! Idiomatic, slice-based, `no_std`, no-`unsafe` core. THE
//! implementation; `cabi` shims only adapt the NT counted-string ABI
//! to these. Faithful to ReactOS `sdk/lib/rtl/unicode.c`.

pub mod acl;
pub mod conv;
pub mod dos8dot3;
pub mod misc;
pub mod nls;
pub mod sd;
pub mod sid;
pub mod ustr;
pub mod utf8;
