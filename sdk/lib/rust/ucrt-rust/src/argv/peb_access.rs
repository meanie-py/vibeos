//! `internal/peb_access.cpp`: two PEB probes through `NtCurrentTeb()`.
//!
//! Offsets are read off Microsoft's compiled object on each target, not from
//! a header (`llvm-objdump -d peb_access.cpp.obj`):
//!
//! | field                                  | i386   | amd64 / arm64 |
//! |----------------------------------------|--------|---------------|
//! | `TEB::ProcessEnvironmentBlock`         | `0x30` | `0x60`        |
//! | `PEB::NtGlobalFlag`                    | `0x68` | `0xBC`        |
//! | `PEB::ProcessParameters`               | `0x10` | `0x20`        |
//! | `RTL_USER_PROCESS_PARAMETERS::Flags`   | `0x08` | `0x08`        |
//!
//! The TEB itself is `fs:[0x18]` (i386, `NtTib.Self`), `gs:[0x30]` (amd64,
//! `NtTib.Self`), and `x18` (arm64).

use core::arch::asm;

/// `FLG_APPLICATION_VERIFIER` (ntexapi.h)
const FLG_APPLICATION_VERIFIER: u32 = 0x0000_0100;
/// `RTL_USER_PROC_SECURE_PROCESS` (the high bit of `Flags`)
const RTL_USER_PROC_SECURE_PROCESS: u32 = 0x8000_0000;

/// Per-target offsets (see the table above); one expression each, so no
/// constant name carries two values.
mod off {
    const X86: bool = cfg!(target_arch = "x86");
    pub const TEB_PEB: usize = if X86 { 0x30 } else { 0x60 };
    pub const PEB_NT_GLOBAL_FLAG: usize = if X86 { 0x68 } else { 0xBC };
    pub const PEB_PROCESS_PARAMETERS: usize = if X86 { 0x10 } else { 0x20 };
}
const PARAMETERS_FLAGS: usize = 0x08;

/// `NtCurrentTeb()`
#[inline(always)]
unsafe fn nt_current_teb() -> *const u8 {
    let teb: *const u8;
    #[cfg(target_arch = "x86")]
    asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, readonly, preserves_flags));
    #[cfg(target_arch = "x86_64")]
    asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, readonly, preserves_flags));
    #[cfg(target_arch = "aarch64")]
    asm!("mov {}, x18", out(reg) teb, options(nostack, nomem, preserves_flags));
    teb
}

/// `NtCurrentTeb()->ProcessEnvironmentBlock`
#[inline(always)]
unsafe fn peb() -> *const u8 {
    *(nt_current_teb().add(off::TEB_PEB) as *const *const u8)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_app_verifier_enabled() -> bool {
    let nt_global_flag = *(peb().add(off::PEB_NT_GLOBAL_FLAG) as *const u32);
    (nt_global_flag & FLG_APPLICATION_VERIFIER) != 0
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_is_secure_process() -> bool {
    let parameters = *(peb().add(off::PEB_PROCESS_PARAMETERS) as *const *const u8);
    let flags = *(parameters.add(PARAMETERS_FLAGS) as *const u32);
    (flags & RTL_USER_PROC_SECURE_PROCESS) != 0
}
