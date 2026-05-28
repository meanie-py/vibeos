//! consrv/frontends/ — Terminal frontends dispatch layer.
//!
//! The frontends plug into the UI-agnostic `condrv/` core via
//! `TERMINAL_VTBL`. `terminal.rs` owns the dispatch tables and
//! ConSrvTerm* shim functions; `input.rs` provides Unicode/ANSI
//! input event conversion; `wcwidth.rs` is the wide-character
//! width table used by the TUI/GUI frontends.
//!
//! Phase 5.4 lands `input.rs`, `terminal.rs`, `wcwidth.rs`.
//! Phase 5.5 lands `tui/tuiterm.rs` and `serial/serialterm.rs`.
//! Phase 5.6 lands `gui/` (conwnd, fullscreen, graphics, guisettings,
//! guiterm, text).

pub mod gui;
pub mod input;
pub mod serial;
pub mod terminal;
pub mod tui;
pub mod wcwidth;
