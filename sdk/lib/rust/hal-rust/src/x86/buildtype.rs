//! HAL build-type constants used by halinit's PRCB version check
//! (mismatched-HAL bugcheck) and the SYNCH_LEVEL macro in halp.h.
//!
//! Translated from `hal/halx86/generic/buildtype.c`. Default (UP) build:
//! UNIPROCESSOR bit set, SYNCH_LEVEL == DISPATCH_LEVEL. The `smp` feature
//! (VIBEOS_SMP_HAL=1, `vibeos-cli build --smp-kernel`) flips both for the
//! MP-kernel pairing — see smp-bringup-quest.md.

use ntkrnl_sys::hal_ddk::{KIRQL, USHORT};

const PRCB_BUILD_DEBUG: u32 = 1;
const PRCB_BUILD_UNIPROCESSOR: u32 = 2;
const DISPATCH_LEVEL: u32 = 2;

/// Per-arch HAL build-type bitmap.  halinit.c compares it against
/// the kernel's PRCB->BuildType at HalInitSystem time and bug-checks
/// MISMATCHED_HAL on mismatch (exact equality).  UP build: DEBUG +
/// UNIPROCESSOR (pairs with ntoskrnl.exe).  smp build: DEBUG only
/// (pairs with ntkrnlmp.exe, whose kiinit drops the UNIPROCESSOR bit).
#[cfg(not(feature = "smp"))]
#[unsafe(no_mangle)]
pub static HalpBuildType: USHORT = (PRCB_BUILD_DEBUG | PRCB_BUILD_UNIPROCESSOR) as USHORT;
#[cfg(feature = "smp")]
#[unsafe(no_mangle)]
pub static HalpBuildType: USHORT = PRCB_BUILD_DEBUG as USHORT;

/// IRQL at which the HAL synchronizes (SYNCH_LEVEL macro in halp.h
/// expands to the static below).  DISPATCH_LEVEL on UP, IPI_LEVEL-2 on SMP
/// (amd64 IPI_LEVEL=14 → 12; i386 IPI_LEVEL=0x1D → 27; must agree with
/// the kernel-side smp SYNCH_LEVEL in ntoskrnl-rust ke_defs / ntkrnl-sys).
/// Rust HAL code uses this const; C HAL code reads the HalpIrqlSynchLevel
/// static (halp.h `#define SYNCH_LEVEL HalpIrqlSynchLevel`) — same value.
#[cfg(not(feature = "smp"))]
pub const SYNCH_LEVEL: KIRQL = DISPATCH_LEVEL as KIRQL;
#[cfg(all(feature = "smp", target_arch = "x86_64"))]
pub const SYNCH_LEVEL: KIRQL = (14 - 2) as KIRQL;
#[cfg(all(feature = "smp", target_arch = "x86"))]
pub const SYNCH_LEVEL: KIRQL = (0x1D - 2) as KIRQL;

#[unsafe(no_mangle)]
pub static mut HalpIrqlSynchLevel: KIRQL = SYNCH_LEVEL;
