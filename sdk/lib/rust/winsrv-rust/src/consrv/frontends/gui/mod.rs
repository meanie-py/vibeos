//! consrv/frontends/gui/ — Win32 GUI frontend (conwnd window class).
//!
//! Phase 5.6: Translates the GUI terminal frontend (the default
//! GraphicalUserInterface in non-headless boots). 6 files:
//! conwnd, fullscreen, graphics, guisettings, guiterm, text.
//!
//! conwnd is added once its translate completes — leave commented
//! here in the interim.

pub mod conwnd;
pub mod fullscreen;
pub mod graphics;
pub mod guisettings;
pub mod guiterm;
pub mod text;
