//! consrv/ — Console subsystem server (CSR server DLL surface for
//! kernel32 console APIs + console host process integration).
//!
//! `condrv/` is the UI-agnostic core (buffers, input queue, lifetime,
//! dummy terminal vtbl). Frontends (TUI/GUI terminals) plug in via
//! `TERMINAL_VTBL`. Phase 5.3 lands `condrv/`; later phases bring up
//! `frontends/`, `init.c`, and the bulk of the CSR server entry
//! points.

pub mod condrv;
