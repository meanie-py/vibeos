use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    PKTRAP_FRAME,
    PKEXCEPTION_FRAME,
    KeDisableInterrupts,
    KeRaiseIrqlToDpcLevel,
    KeFlushCurrentTb,
    KeRestoreInterrupts,
};

static mut KiOldIrql: KIRQL = 0;
#[no_mangle]
pub static mut KiFreezeFlag: ULONG = 0;

// Per-arch freeze/thaw workers (#[no_mangle] in ke/{amd64,i386}/freeze.rs and
// ke/arm64/stubs.rs) — link-level externs, same-crate.
#[cfg(feature = "smp")]
extern "system" {
    fn KxFreezeExecution();
    fn KxThawExecution();
}

#[no_mangle]
pub unsafe extern "system" fn KeFreezeExecution(
    _TrapFrame: PKTRAP_FRAME,
    _ExceptionFrame: PKEXCEPTION_FRAME,
) -> BOOLEAN {
    let Enable: BOOLEAN;
    #[allow(unused_mut, unused_assignments)]
    let mut OldIrql: KIRQL = 0;

    Enable = KeDisableInterrupts();
    KiFreezeFlag = 4;

    #[cfg(not(feature = "smp"))]
    {
        OldIrql = KeGetCurrentIrql();
        let adjusted_irql = if OldIrql < DISPATCH_LEVEL {
            KeRaiseIrqlToDpcLevel()
        } else {
            OldIrql
        };
        KiOldIrql = adjusted_irql;
    }

    #[cfg(feature = "smp")]
    {
        KeRaiseIrql(HIGH_LEVEL, core::ptr::addr_of_mut!(OldIrql));
        KxFreezeExecution();
        KiOldIrql = OldIrql;
    }

    Enable
}

#[no_mangle]
pub unsafe extern "system" fn KeThawExecution(Enable: BOOLEAN) {
    #[cfg(feature = "smp")]
    KxThawExecution();

    KiFreezeFlag = 0;

    KeFlushCurrentTb();

    #[cfg(not(feature = "smp"))]
    {
        if KiOldIrql < DISPATCH_LEVEL {
            KeLowerIrql(KiOldIrql);
        }
    }

    #[cfg(feature = "smp")]
    KeLowerIrql(KiOldIrql);

    KeRestoreInterrupts(Enable);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/freeze.c
//  * PURPOSE:         Routines for freezing and unfreezing processors for
//  *                  kernel debugger synchronization.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// /* Freeze data */
// KIRQL KiOldIrql;
// ULONG KiFreezeFlag;
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// NTAPI
// KeFreezeExecution(IN PKTRAP_FRAME TrapFrame,
//                   IN PKEXCEPTION_FRAME ExceptionFrame)
// {
//     BOOLEAN Enable;
//     KIRQL OldIrql;
//
// #ifndef CONFIG_SMP
//     UNREFERENCED_PARAMETER(TrapFrame);
//     UNREFERENCED_PARAMETER(ExceptionFrame);
// #endif
//
//     /* Disable interrupts, get previous state and set the freeze flag */
//     Enable = KeDisableInterrupts();
//     KiFreezeFlag = 4;
//
// #ifndef CONFIG_SMP
//     /* Raise IRQL if we have to */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < DISPATCH_LEVEL)
//         OldIrql = KeRaiseIrqlToDpcLevel();
// #else
//     /* Raise IRQL to HIGH_LEVEL */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
// #endif
//
// #ifdef CONFIG_SMP
//     /* Architecture specific freeze code */
//     KxFreezeExecution();
// #endif
//
//     /* Save the old IRQL to be restored on unfreeze */
//     KiOldIrql = OldIrql;
//
//     /* Return whether interrupts were enabled */
//     return Enable;
// }
//
// VOID
// NTAPI
// KeThawExecution(IN BOOLEAN Enable)
// {
// #ifdef CONFIG_SMP
//     /* Architecture specific thaw code */
//     KxThawExecution();
// #endif
//
//     /* Clear the freeze flag */
//     KiFreezeFlag = 0;
//
//     /* Cleanup CPU caches */
//     KeFlushCurrentTb();
//
//     /* Restore the old IRQL */
// #ifndef CONFIG_SMP
//     if (KiOldIrql < DISPATCH_LEVEL)
// #endif
//     KeLowerIrql(KiOldIrql);
//
//     /* Re-enable interrupts */
//     KeRestoreInterrupts(Enable);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
