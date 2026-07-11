use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    KeSetGdtSelector,
    PKIPCR,
    PKPROCESS,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const KGDT_LDT: u16 = 0x48;

#[no_mangle]
pub unsafe extern "system" fn KiSwapProcess(
    NewProcess: PKPROCESS,
    OldProcess: PKPROCESS,
) {
    let NewProcess = opaque_ptr(NewProcess);
    let OldProcess = opaque_ptr(OldProcess);
    let Pcr = KeGetPcr() as PKIPCR;
    
    #[cfg(feature = "smp")]
    {
        // C: InterlockedXor(&Process->ActiveProcessors, Pcr->SetMember).
        use core::sync::atomic::{AtomicI32, Ordering};
        let SetMember = (*Pcr).SetMember as LONG;
        AtomicI32::from_ptr(core::ptr::addr_of_mut!((*NewProcess).ActiveProcessors) as *mut LONG)
            .fetch_xor(SetMember, Ordering::SeqCst);
        AtomicI32::from_ptr(core::ptr::addr_of_mut!((*OldProcess).ActiveProcessors) as *mut LONG)
            .fetch_xor(SetMember, Ordering::SeqCst);
    }

    if (*NewProcess).LdtDescriptor.LimitLow != (*OldProcess).LdtDescriptor.LimitLow {
        if (*NewProcess).LdtDescriptor.LimitLow != 0 {
            let desc_ptr = core::ptr::addr_of!((*NewProcess).LdtDescriptor) as *const ULONG;
            KeSetGdtSelector(
                KGDT_LDT as ULONG,
                *desc_ptr,
                *desc_ptr.add(1),
            );
            Ke386SetLocalDescriptorTable(KGDT_LDT);
        } else {
            Ke386SetLocalDescriptorTable(0);
        }
    }

    core::arch::asm!("mov cr3, {0:e}", in(reg) (*NewProcess).DirectoryTableBase[0], options(nostack, nomem));

    Ke386SetGs(0);

    (*(*Pcr).TSS).IoMapBase = (*NewProcess).IopmOffset;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ke/i386/context.c
//  * PURPOSE:         Context Switching Related Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
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
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// KiSwapProcess(IN PKPROCESS NewProcess,
//               IN PKPROCESS OldProcess)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
// #ifdef CONFIG_SMP
//     LONG SetMember;
//
//     /* Update active processor mask */
//     SetMember = (LONG)Pcr->SetMember;
//     InterlockedXor((PLONG)&NewProcess->ActiveProcessors, SetMember);
//     InterlockedXor((PLONG)&OldProcess->ActiveProcessors, SetMember);
// #endif
//
//     /* Check for new LDT */
//     if (NewProcess->LdtDescriptor.LimitLow != OldProcess->LdtDescriptor.LimitLow)
//     {
//         if (NewProcess->LdtDescriptor.LimitLow)
//         {
//             KeSetGdtSelector(KGDT_LDT,
//                              ((PULONG)&NewProcess->LdtDescriptor)[0],
//                              ((PULONG)&NewProcess->LdtDescriptor)[1]);
//             Ke386SetLocalDescriptorTable(KGDT_LDT);
//         }
//         else
//         {
//             Ke386SetLocalDescriptorTable(0);
//         }
//     }
//
//     /* Update CR3 */
//     __writecr3(NewProcess->DirectoryTableBase[0]);
//
//     /* Clear GS */
//     Ke386SetGs(0);
//
//     /* Update IOPM offset */
//     Pcr->TSS->IoMapBase = NewProcess->IopmOffset;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
