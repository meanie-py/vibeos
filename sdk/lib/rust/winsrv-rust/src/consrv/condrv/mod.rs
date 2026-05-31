//! consrv/condrv/ — UI-agnostic console "device driver" core.
//!
//! Console lifetime, screen-buffer (text + graphics) abstractions,
//! input-record queue, and the dummy terminal vtbl that consrv
//! falls back to before a real terminal frontend (TUI/GUI) attaches.
//!
//! Phase 5.3 landing — translations are integrated but cargo-check
//! parity is still in progress (see translation-philosophy: broken
//! Rust in tree beats hiding it from the build).

pub mod coninput;
pub mod conoutput;
pub mod console;
pub mod dummyterm;
pub mod graphics;
// text.rs landed in this commit too — pub-mod is wired by the same
// Phase 5.3 patch (see commit message). If text.rs is missing on
// build that's a corrupted checkout.
pub mod text;
