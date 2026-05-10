//! ex subsystem — translated from C.

pub mod atom;
pub mod callback;
pub mod dbgctrl;
pub mod efi;
pub mod event;
pub mod evtpair;
pub mod exintrin;
pub mod fmutex;
pub mod handle;
pub mod harderr;
pub mod hdlsterm;
pub mod init;
pub mod interlocked;
pub mod keyedevt;
pub mod locale;
pub mod lookas;
pub mod mutant;
pub mod profile;
pub mod pushlock;
pub mod resource;
pub mod rundown;
pub mod sem;
pub mod shutdown;
pub mod sysinfo;
pub mod time;
pub mod timer;
pub mod uuid;
pub mod win32k;
pub mod work;
pub mod xipdisp;
pub mod zone;

// Folded back from standalone `pool` crate 2026-05-09.
pub mod pool;
