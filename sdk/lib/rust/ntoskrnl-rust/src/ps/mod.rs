//! ps subsystem — translated from C.

#[cfg(target_arch = "x86")]
pub mod i386;
#[cfg(target_arch = "x86_64")]
pub mod amd64;

pub mod apphelp;
pub mod debug;
pub mod job;
pub mod kill;
pub mod process;
pub mod psmgr;
pub mod psnotify;
pub mod query;
pub mod quota;
pub mod security;
pub mod state;
pub mod thread;
pub mod win32;
