//! consrv/frontends/tui/ — text-mode console (BlueScreen driver) backend.
//!
//! Targets `\Device\BlueScreen` (drivers/base/blue) when freeldr boots the
//! kernel without a GUI subsystem. The Rust translation is unconditional at
//! the module level; the cmake cohab flip will gate inclusion per arch
//! (mirrors the C `TUITERM_COMPILE` build define).

pub mod tuiterm;
