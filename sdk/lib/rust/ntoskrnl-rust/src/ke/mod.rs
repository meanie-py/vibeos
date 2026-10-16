//! ke subsystem — translated from C.

pub mod ke_defs;
pub mod apc;
pub mod balmgr;
pub mod bug;
pub mod clock;
pub mod config;
pub mod devqueue;
pub mod dpc;
pub mod eventobj;
pub mod except;
pub mod freeze;
pub mod gate;
pub mod gmutex;
pub mod ipi;
pub mod krnlinit;
pub mod mutex;
pub mod ssdt;
pub mod processor;
pub mod procobj;
pub mod profobj;
pub mod queue;
pub mod semphobj;
pub mod spinlock;
pub mod flightrec;
pub mod synctrace;
pub mod census;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86x64;
#[cfg(feature = "lockdep")]
pub mod lockdep;
#[cfg(feature = "cswitchdep")]
pub mod cswitchdep;
pub mod waitdep;
pub mod thrdobj;
pub mod thrdschd;
pub mod time;
pub mod timerobj;
pub mod wait;

// Arch-specific ke modules
#[cfg(target_arch = "x86")]
pub mod i386;
#[cfg(target_arch = "x86_64")]
pub mod amd64;
#[cfg(target_arch = "aarch64")]
pub mod arm64;

