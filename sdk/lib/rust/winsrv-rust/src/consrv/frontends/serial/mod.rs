//! consrv/frontends/serial/ — escape-sequence serial console backend.
//!
//! arm64-only via `SERIALTERM_COMPILE` in `consrv.cmake`. The Rust
//! translation is unconditional at the module level; the cmake cohab
//! flip will gate inclusion per arch (mirrors the C build).

pub mod serialterm;
