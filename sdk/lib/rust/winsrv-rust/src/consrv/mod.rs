//! consrv/ — Console subsystem server (CSR server DLL surface for
//! kernel32 console APIs + console host process integration).
//!
//! `condrv/` is the UI-agnostic core (buffers, input queue, lifetime,
//! dummy terminal vtbl). Frontends (TUI/GUI terminals) plug in via
//! `TERMINAL_VTBL`. Phase 5.3 lands `condrv/`; Phase 5.4 lands the
//! consrv top + frontends dispatch (alias/coninput/conoutput/console/
//! frontendctl/handle/history/init/lineinput/popup/settings/shutdown/
//! subsysreg + `frontends/{input,terminal,wcwidth}`).

pub mod condrv;

// Phase 5.4: consrv top
pub mod alias;
pub mod coninput;
pub mod conoutput;
pub mod console;
pub mod frontendctl;
pub mod handle;
pub mod history;
pub mod init;
pub mod lineinput;
pub mod popup;
pub mod settings;
pub mod shutdown;
pub mod subsysreg;

// Phase 5.4: frontends dispatch
pub mod frontends;
