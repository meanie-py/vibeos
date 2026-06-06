//! Idiomatic, slice-based, `no_std`, no-`unsafe` core. THE
//! implementation; `cabi` shims only adapt the C ABI to these.
//!
//! Contracts are faithful to ReactOS libcntpr (see cnt-manifest.toml),
//! expressed in honest Rust idiom (Option for search, plain for total,
//! a status flag where the C contract hides a fallible outcome).

pub mod casemap;
pub mod env;
// fb64/fb32 bit→float shims — the one audited unsafe carve-out, used by
// floatcvt to compile no-SSE on i386 (see floatbits.rs).
pub mod floatbits;
// floatcvt is no-SSE-safe (extern "C" returns + floatbits shims) → built
// unconditionally on all 3 arches, no longer behind `float`.
pub mod floatcvt;
pub mod mbcp;
pub mod mem;
pub mod path;
pub mod radix;
pub mod rng;
#[cfg(feature = "float")]
pub mod scanf;
pub mod string;
pub mod strmut;
// strtod/atof — pure f64 parser, no-SSE-safe (returns (f64,usize) via
// tuple sret, no by-value f64 register return), no unsafe.
pub mod strtod;
pub mod strtok;
pub mod wstring;
