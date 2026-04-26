use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    KeRaiseIrqlToDpcLevel,
};

#[no_mangle]
pub unsafe extern "system" fn KxGetCurrentIrql() -> KIRQL {
    KeGetCurrentIrql()
}

#[no_mangle]
pub unsafe extern "system" fn KxLowerIrql(NewIrql: KIRQL) {
    KeLowerIrql(NewIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KxRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    KfRaiseIrql(NewIrql)
}

#[no_mangle]
pub unsafe extern "system" fn KxRaiseIrqlToDpcLevel() -> KIRQL {
    KeRaiseIrqlToDpcLevel()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * PURPOSE:         Routines for IRQL support
//  * PROGRAMMERS:     Timo Kreuzer
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// NTKERNELAPI
// KIRQL
// KxGetCurrentIrql(VOID)
// {
//     return KeGetCurrentIrql();
// }
//
// NTKERNELAPI
// VOID
// KxLowerIrql(IN KIRQL NewIrql)
// {
//     KeLowerIrql(NewIrql);
// }
//
// NTKERNELAPI
// KIRQL
// KxRaiseIrql(IN KIRQL NewIrql)
// {
//     return KfRaiseIrql(NewIrql);
// }
//
// NTKERNELAPI
// KIRQL
// KxRaiseIrqlToDpcLevel(VOID)
// {
//     return KeRaiseIrqlToDpcLevel();
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
