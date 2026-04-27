//! kd subsystem — kernel-debug I/O surface.
//! Translated from ntoskrnl/kd/*.c via per-function rustconv translate.
//! Arch-specific kdserial.c lives in the existing arm/, arm64/, i386/ modules.

pub mod kdio;
pub mod kdmain;
pub mod kdprompt;
pub mod kdps2kbd;
pub mod kdserial;
pub mod kdterminal;
