//! Idiomatic, slice-based, `no_std`, no-`unsafe` core. THE
//! implementation; `cabi` shims only adapt the C ABI to these.
//!
//! Contracts are faithful to ReactOS libcntpr (see cnt-manifest.toml),
//! expressed in honest Rust idiom (Option for search, plain for total,
//! a status flag where the C contract hides a fallible outcome).

pub mod casemap;
pub mod mbcp;
pub mod mem;
pub mod path;
pub mod radix;
pub mod scanf;
pub mod string;
pub mod strmut;
pub mod strtok;
pub mod wstring;
