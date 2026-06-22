//! misc/ — user32 miscellaneous client APIs (dllmain, timer, object, exit, desktop, ...).
//! Wave 0 leaf: timer.rs.

pub mod timer;
pub mod display;
pub mod desktop;

// Wave 4 — secondary surface (misc/).
pub mod dllmain;
pub mod exit;
pub mod exticon;
pub mod imm;
pub mod logon;
pub mod misc;
pub mod object;
pub mod resources;
pub mod rtlstr;
pub mod stubs;
pub mod usrapihk;
pub mod winhelp;
pub mod winsta;
