// Brings i386 KxFreezeExecution/KxThawExecution/KxSwitchKdProcessor to parity
// with the real amd64 implementation
// (sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/freeze.rs). Same protocol, same
// statics, same IPI_FROZEN state machine — only the KAFFINITY width differs.
// `ntkrnl_sys::ddk::KAFFINITY` (the type of both `KeActiveProcessors` and
// KPRCB.SetMember, and of KiIpiSend's TargetSet param) is bindgen-local
// ULONG_PTR: `c_ulonglong` (u64) on amd64/arm64 but `c_ulong` (u32) on
// i386 (verified against bindgen_i386.rs / bindgen_amd64.rs — these are
// bindgen's own per-arch ULONG_PTR, distinct from base.rs's `usize` alias
// of the same name used elsewhere in the tree). So amd64/freeze.rs's
// `KeActiveProcessors as u64` is a same-width no-op cast there; on i386 the
// operand is already u32-width KAFFINITY end to end, so this file omits
// the cast entirely rather than mis-casting it to u64.
//
// No shared-static hoisting is required: ke/mod.rs cfg-gates the per-arch
// freeze modules mutually exclusively —
//     #[cfg(target_arch = "x86")]     pub mod i386;
//     #[cfg(target_arch = "x86_64")]  pub mod amd64;
//     #[cfg(target_arch = "aarch64")] pub mod arm64;
// — so only one of amd64::freeze / i386::freeze / arm64::stubs is ever
// compiled into a given target. A `static mut KiFreezeOwner` re-declared
// identically in i386/freeze.rs cannot collide with amd64's. See NOTES.md
// for the reasoning in full plus the one open question (KiProcessorFreezeHandler
// wiring into the i386 trap/IPI dispatch path).

use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    KeGetCurrentProcessorNumber, KeMemoryBarrier, KiIpiSend, KiRestoreProcessorState,
    KiSaveProcessorState, PKEXCEPTION_FRAME, PKTRAP_FRAME,
};

type KCONTINUE_STATUS = u32;

static mut KiFreezeOwner: PKPRCB = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn KiProcessorFreezeHandler(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
) -> BOOLEAN {
    let CurrentPrcb = KeGetCurrentPrcb();

    if (*CurrentPrcb).IpiFrozen != IPI_FROZEN_STATE_TARGET_FREEZE {
        return 0u8;
    }

    (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_FROZEN;

    KiSaveProcessorState(TrapFrame, ExceptionFrame);

    while (*CurrentPrcb).IpiFrozen != IPI_FROZEN_STATE_THAW {
        if ((*CurrentPrcb).IpiFrozen & IPI_FROZEN_FLAG_ACTIVE) != 0 {
            let ContinueStatus = KdReportProcessorChange();

            (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_FROZEN;

            if ContinueStatus == ContinueSuccess {
                (*KiFreezeOwner).IpiFrozen = IPI_FROZEN_STATE_THAW;
            }
        }

        YieldProcessor();
        KeMemoryBarrier();
    }

    KiRestoreProcessorState(TrapFrame, ExceptionFrame);

    (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_RUNNING;

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KxFreezeExecution() {
    let CurrentPrcb = KeGetCurrentPrcb();

    if KiFreezeOwner == CurrentPrcb {
        return;
    }

    // Try to acquire the freeze owner. Someone else may be faster; we
    // expect an NMI/IPI to freeze us any time, so spin here until the
    // freeze owner slot is available again.
    while InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(KiFreezeOwner) as *mut PVOID,
        CurrentPrcb as PVOID,
        core::ptr::null_mut(),
    ) != core::ptr::null_mut()
    {
        while KiFreezeOwner != core::ptr::null_mut() {
            YieldProcessor();
            KeMemoryBarrier();
        }
    }

    (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_OWNER | IPI_FROZEN_FLAG_ACTIVE;

    for i in 0..KeNumberProcessors {
        let TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i as usize);
        if TargetPrcb != CurrentPrcb {
            (*TargetPrcb).IpiFrozen = IPI_FROZEN_STATE_TARGET_FREEZE;
        }
    }

    // No width cast needed here (see file header): KAFFINITY is u32 on
    // i386 end to end, unlike amd64/freeze.rs's `as u64` no-op cast.
    KiIpiSend(KeActiveProcessors & !(*CurrentPrcb).SetMember, IPI_FREEZE);

    for i in 0..KeNumberProcessors {
        let TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i as usize);
        if TargetPrcb != CurrentPrcb {
            while (*TargetPrcb).IpiFrozen != IPI_FROZEN_STATE_FROZEN {
                YieldProcessor();
                KeMemoryBarrier();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KxThawExecution() {
    let CurrentPrcb = KeGetCurrentPrcb();

    for i in 0..KeNumberProcessors {
        let TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i as usize);
        if TargetPrcb != CurrentPrcb {
            (*TargetPrcb).IpiFrozen = IPI_FROZEN_STATE_THAW;
        }
    }

    for i in 0..KeNumberProcessors {
        let TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(i as usize);
        if TargetPrcb != CurrentPrcb {
            while (*TargetPrcb).IpiFrozen != IPI_FROZEN_STATE_RUNNING {
                YieldProcessor();
                KeMemoryBarrier();
            }
        }
    }

    (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_RUNNING;

    InterlockedExchangePointer(
        core::ptr::addr_of_mut!(KiFreezeOwner) as *mut PVOID,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn KxSwitchKdProcessor(ProcessorIndex: ULONG) -> KCONTINUE_STATUS {
    let CurrentPrcb = KeGetCurrentPrcb();

    (*CurrentPrcb).IpiFrozen &= !IPI_FROZEN_FLAG_ACTIVE;

    let TargetPrcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(ProcessorIndex as usize);
    (*TargetPrcb).IpiFrozen |= IPI_FROZEN_FLAG_ACTIVE;

    if KiFreezeOwner != CurrentPrcb {
        return ContinueNextProcessor;
    }

    while (*CurrentPrcb).IpiFrozen == IPI_FROZEN_STATE_OWNER {
        YieldProcessor();
        KeMemoryBarrier();
    }

    if (*CurrentPrcb).IpiFrozen == IPI_FROZEN_STATE_THAW {
        crate::kdp_dprintf!(b"[%u] KxSwitchKdProcessor: ContinueSuccess\n\0",
            KeGetCurrentProcessorNumber(),
        );
        (*CurrentPrcb).IpiFrozen = IPI_FROZEN_STATE_OWNER | IPI_FROZEN_FLAG_ACTIVE;
        return ContinueSuccess;
    }

    ContinueProcessorReselected
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Processor freeze support for i386
//  * COPYRIGHT:
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// KxFreezeExecution(
//     VOID)
// {
//     UNIMPLEMENTED;
// }
//
// VOID
// NTAPI
// KxThawExecution(
//     VOID)
// {
//     UNIMPLEMENTED;
// }
//
// KCONTINUE_STATUS
// NTAPI
// KxSwitchKdProcessor(
//     _In_ ULONG ProcessorIndex)
// {
//     UNIMPLEMENTED;
//     return ContinueError;
// }
//
// NOTE: the original i386 C source was itself just UNIMPLEMENTED stubs — this
// port does not have a "real" upstream i386 C implementation to translate
// faithfully. amd64/freeze.c (see ke/amd64/freeze.rs's ORIGINAL C SOURCE
// footer) is authoritative for the actual freeze/thaw protocol; this file
// applies that same protocol to i386's KPRCB/KAFFINITY shapes.
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
