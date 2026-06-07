//! Shared substrate for the rtl-misc leaves: helpers ntkrnl-sys doesn't surface
//! directly, that the per-file translations reference.
//!
//! `NtCurrentTeb` — ntkrnl-sys::ddk declares it out-of-line (`_NtCurrentTeb`)
//! but it's really an fs/gs/x18 intrinsic with no defined symbol, so the leaves
//! that read the TEB / process heap import this inline-asm version (a plain ddk
//! import compiles but link-fails — invisible to `cargo check`).
use core::ffi::c_void;
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

// C `#define RtlGetProcessHeap() NtCurrentPeb()->ProcessHeap` — crate-private inline
// (NOT #[no_mangle]; ntdll-rust owns the RtlGetProcessHeap export symbol).
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn RtlGetProcessHeap() -> *mut c_void {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap as *mut c_void
}

// C `#define GetProcessHeap() RtlGetProcessHeap()`
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetProcessHeap() -> *mut c_void {
    RtlGetProcessHeap()
}
