//! i386/amd64 arch-specific MM code — translated from C.

// page.c is shared between i386 and amd64
pub mod page;

// procsup.c and ARM3/i386/init.c are i386 only
#[cfg(target_arch = "x86")]
pub mod procsup;
#[cfg(target_arch = "x86")]
pub mod init;
