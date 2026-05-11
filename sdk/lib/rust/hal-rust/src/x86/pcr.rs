//! KeGetPcr / KeGetCurrentPrcb — local replicas of the FORCEINLINE
//! kernel macros.  Same instruction sequence as
//! `ntkrnl_sys::inline::ke::{KeGetPcr,KeGetCurrentPrcb}`; replicated
//! here because cross-crate `#[inline]` on inline-asm bodies doesn't
//! reliably fire at our workspace opt-level (= 1) — see
//! `feedback_cross_crate_inline_asm.md`.  The two-line duplicate is
//! cheaper than a missing-symbol link error.
//!
//! - i386: `fs:[0x1C]` → `KPCR.SelfPcr` (self-pointer)
//! - amd64: `gs:[0x18]` → `KPCR.Self_` (self-pointer, nested in NtTib union)

use ntkrnl_sys::hal_ddk::{_KPCR, _KPRCB};

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> *mut _KPCR {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "mov {:e}, dword ptr fs:[0x1C]",
            out(reg) result,
            options(nostack, nomem, preserves_flags),
        );
    }
    result as *mut _KPCR
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeGetPcr() -> *mut _KPCR {
    let result: *mut _KPCR;
    unsafe {
        core::arch::asm!(
            "mov {}, gs:[0x18]",
            out(reg) result,
            options(nostack, nomem, preserves_flags),
        );
    }
    result
}

/// KeGetCurrentPrcb — i386: `KPCR.Prcb` is a plain field (offset 0x20).
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeGetCurrentPrcb() -> *mut _KPRCB {
    unsafe { (*KeGetPcr()).Prcb }
}

/// KeGetCurrentPrcb — amd64: `KPCR.CurrentPrcb` lives in the nested
/// anonymous union under NtTib.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeGetCurrentPrcb() -> *mut _KPRCB {
    unsafe { (*KeGetPcr()).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb }
}
