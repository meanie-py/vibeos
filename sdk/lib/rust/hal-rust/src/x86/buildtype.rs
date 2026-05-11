//! HAL build-type constants used by halinit's PRCB version check
//! (mismatched-HAL bugcheck) and the SYNCH_LEVEL macro in halp.h.
//!
//! Translated from `hal/halx86/generic/buildtype.c`. We ship UP only
//! after the SMP prune, so HalpIrqlSynchLevel is always DISPATCH_LEVEL
//! and HalpBuildType always has the UNIPROCESSOR bit set.

use ntkrnl_sys::hal_ddk::{KIRQL, USHORT};

const PRCB_BUILD_DEBUG: u32 = 1;
const PRCB_BUILD_UNIPROCESSOR: u32 = 2;
const DISPATCH_LEVEL: u32 = 2;

/// Per-arch HAL build-type bitmap.  halinit.c compares it against
/// the kernel's PRCB->BuildType at HalInitSystem time and bug-checks
/// MISMATCHED_HAL on mismatch.  We're a UP build, so always include
/// PRCB_BUILD_UNIPROCESSOR; DBG=1 in our build sets the debug bit too.
#[unsafe(no_mangle)]
pub static HalpBuildType: USHORT = (PRCB_BUILD_DEBUG | PRCB_BUILD_UNIPROCESSOR) as USHORT;

/// IRQL at which the HAL synchronizes (SYNCH_LEVEL macro in halp.h
/// expands to this).  DISPATCH_LEVEL on UP, IPI_LEVEL-2 on SMP.
#[unsafe(no_mangle)]
pub static mut HalpIrqlSynchLevel: KIRQL = DISPATCH_LEVEL as KIRQL;
