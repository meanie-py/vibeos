//! Shared substrate for the rtl-diag leaves.
//!
//! `NtCurrentTeb` — ntkrnl-sys::ddk declares it out-of-line (`_NtCurrentTeb`)
//! but it's really an fs/gs/x18 intrinsic with no defined symbol, so the leaves
//! that read the TEB import this inline-asm version instead.
use ntkrnl_sys::ddk::PTEB;

#[cfg(target_arch = "x86")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}
