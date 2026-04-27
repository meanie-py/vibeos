//! i386/amd64 arch-specific MM code — translated from C.

// page.c is shared between i386 and amd64
pub mod page;

// procsup.c and ARM3/i386/init.c are i386 only
#[cfg(target_arch = "x86")]
pub mod procsup;
#[cfg(target_arch = "x86")]
pub mod init;

// kd/i386/kdserial.c — used by BOTH i386 and amd64 (per ntos.cmake which fed
// the same .c file into both arches). Shared x86 COM-port glue.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod kdserial;
