use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _KPROCESS, _KTHREAD, _XSAVE_FORMAT,
    PKPROCESS, PKTHREAD, PKTRAP_FRAME, PLONG64, XSAVE_FORMAT,
    HalClearSoftwareInterrupt, KeGetPcr, KiRetireDpcList, KiSwapContext,
    PKIPCR,
};

#[no_mangle]
pub static mut ProcessCount: ULONG = 0;

#[no_mangle]
pub static mut KeXStateLength: SIZE_T = core::mem::size_of::<XSAVE_FORMAT>();

extern "system" {
    fn KiSwitchKernelStackHelper(StackOffset: isize, OldStackBase: PVOID) -> PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn KiSwitchKernelStack(
    StackBase: PVOID,
    StackLimit: PVOID,
) -> PVOID {
    let StackBase = opaque_ptr(StackBase);
    let StackLimit = opaque_ptr(StackLimit);
    let CurrentThread = PsGetCurrentThread();
    let OldStackBase = (*CurrentThread).StackBase;
    let StackSize = (*CurrentThread).StackBase as usize - (*CurrentThread).StackLimit as usize;
    
    let StackOffset = StackBase as isize - (*CurrentThread).StackBase as isize;
    
    RtlCopyMemory(
        (StackBase as *mut u8).sub(StackSize) as PVOID,
        (*CurrentThread).StackLimit as PVOID,
        StackSize,
    );
    
    // Disable interrupts (equivalent to _disable() in C)
    core::arch::asm!("cli");
    
    (*CurrentThread).TrapFrame = ((*CurrentThread).TrapFrame as isize + StackOffset) as PKTRAP_FRAME;
    (*CurrentThread).InitialStack = ((*CurrentThread).InitialStack as isize + StackOffset) as PVOID;
    (*CurrentThread).StateSaveArea = ((*CurrentThread).StateSaveArea as isize + StackOffset) as *mut _XSAVE_FORMAT;
    (*CurrentThread).StackBase = StackBase;
    (*CurrentThread).StackLimit = StackLimit as u64;
    (*CurrentThread).__bindgen_anon_5.__bindgen_anon_4.LargeStack = 1u8;
    
    let Pcr = KeGetPcr() as PKIPCR;
    (*Pcr).Prcb.RspBase = ((*Pcr).Prcb.RspBase as isize + StackOffset) as u64;
    (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0 = ((*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0 as isize + StackOffset) as u64;
    
    OldStackBase
}

#[no_mangle]
pub unsafe extern "system" fn KiIdleLoop() -> ! {
    let Prcb = KeGetCurrentPrcb();
    
    loop {
        core::arch::asm!("sti");
        core::hint::spin_loop();
        core::hint::spin_loop();
        core::arch::asm!("cli");
        
        if ((*Prcb).DpcData[0].DpcQueueDepth != 0)
            || ((*Prcb).TimerRequest != 0)
            || (!(*Prcb).DeferredReadyListHead.Next.is_null())
        {
            HalClearSoftwareInterrupt(DISPATCH_LEVEL);
            KiRetireDpcList(Prcb);
        }
        
        if !(*Prcb).NextThread.is_null() {
            core::arch::asm!("sti");
            
            let OldThread = (*Prcb).CurrentThread;
            let NewThread = (*Prcb).NextThread;
            
            (*Prcb).NextThread = core::ptr::null_mut();
            (*Prcb).CurrentThread = NewThread;
            
            (*(NewThread as *mut _KTHREAD)).__bindgen_anon_5.__bindgen_anon_6.State = 2u8;
            
            #[cfg(feature = "smp")]
            {
                // C: KfRaiseIrql(SYNCH_LEVEL) — do the swap at SYNCH_LEVEL.
                crate::ke::ke_defs::KfRaiseIrql(crate::ke::ke_defs::SYNCH_LEVEL);
            }

            KiSwapContext(APC_LEVEL, OldThread);

            #[cfg(feature = "smp")]
            {
                // C: KeLowerIrql(DISPATCH_LEVEL) — back to DISPATCH_LEVEL.
                crate::ke::ke_defs::KeLowerIrql(DISPATCH_LEVEL);
            }
        } else {
            ((*Prcb).PowerState.IdleFunction.unwrap())(&mut (*Prcb).PowerState as *mut _);
        }
    }
}

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
        // C: InterlockedXor64(&Process->ActiveProcessors, Pcr->Prcb.SetMember)
        // (Prcb is a direct embedded field of amd64 KIPCR).
        use core::sync::atomic::{AtomicU64, Ordering};
        let set_member = (*Pcr).Prcb.SetMember as u64;
        AtomicU64::from_ptr(
            core::ptr::addr_of_mut!((*(NewProcess as *mut _KPROCESS)).ActiveProcessors) as *mut u64,
        )
        .fetch_xor(set_member, Ordering::SeqCst);
        AtomicU64::from_ptr(
            core::ptr::addr_of_mut!((*(OldProcess as *mut _KPROCESS)).ActiveProcessors) as *mut u64,
        )
        .fetch_xor(set_member, Ordering::SeqCst);
    }
    
    core::arch::x86_64::_mm_mfence();
    core::arch::asm!("mov cr3, {}", in(reg) (*(NewProcess as *mut _KPROCESS)).DirectoryTableBase[0]);
    
    (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).IoMapBase = (*(NewProcess as *mut _KPROCESS)).IopmOffset;
}

#[no_mangle]
pub unsafe extern "system" fn NtSetLdtEntries(
    Selector1: ULONG,
    LdtEntry1: u64,
    Selector2: ULONG,
    LdtEntry2: u64,
) -> NTSTATUS {
    core::arch::asm!("int3");
    ntkrnl_sys::constants::STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn NtVdmControl(
    ControlCode: ULONG,
    ControlData: PVOID,
) -> NTSTATUS {
    let ControlData = opaque_ptr(ControlData);
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * PURPOSE:         stubs
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// ULONG ProcessCount;
// SIZE_T KeXStateLength = sizeof(XSAVE_FORMAT);
//
// PVOID
// KiSwitchKernelStackHelper(
//     LONG_PTR StackOffset,
//     PVOID OldStackBase);
//
// /*
//  * Kernel stack layout (example pointers):
//  * 0xFFFFFC0F'2D008000 KTHREAD::StackBase
//  *    [XSAVE_AREA size == KeXStateLength = 0x440]
//  * 0xFFFFFC0F'2D007BC0 KTHREAD::StateSaveArea _XSAVE_FORMAT
//  * 0xFFFFFC0F'2D007B90 KTHREAD::InitialStack
//  *    [0x190 bytes KTRAP_FRAME]
//  * 0xFFFFFC0F'2D007A00 KTHREAD::TrapFrame
//  *    [KSTART_FRAME] or ...
//  *    [KSWITCH_FRAME]
//  * 0xFFFFFC0F'2D007230 KTHREAD::KernelStack
//  */
//
// PVOID
// NTAPI
// KiSwitchKernelStack(PVOID StackBase, PVOID StackLimit)
// {
//     PKTHREAD CurrentThread;
//     PVOID OldStackBase;
//     LONG_PTR StackOffset;
//     SIZE_T StackSize;
//     PKIPCR Pcr;
//     ULONG Eflags;
//
//     /* Get the current thread */
//     CurrentThread = PsGetCurrentThread();
//
//     /* Save the old stack base */
//     OldStackBase = CurrentThread->StackBase;
//
//     /* Get the size of the current stack */
//     StackSize = (ULONG_PTR)CurrentThread->StackBase - CurrentThread->StackLimit;
//     ASSERT(StackSize <= (ULONG_PTR)StackBase - (ULONG_PTR)StackLimit);
//
//     /* Copy the current stack contents to the new stack */
//     RtlCopyMemory((PUCHAR)StackBase - StackSize,
//                   (PVOID)CurrentThread->StackLimit,
//                   StackSize);
//
//     /* Calculate the offset between the old and the new stack */
//     StackOffset = (PUCHAR)StackBase - (PUCHAR)CurrentThread->StackBase;
//
//     /* Disable interrupts while messing with the stack */
//     Eflags = __readeflags();
//     _disable();
//
//     /* Set the new trap frame */
//     CurrentThread->TrapFrame = (PKTRAP_FRAME)Add2Ptr(CurrentThread->TrapFrame,
//                                                      StackOffset);
//
//     /* Set the new initial stack */
//     CurrentThread->InitialStack = Add2Ptr(CurrentThread->InitialStack,
//                                           StackOffset);
//
//     /* Switch StateSaveArea */
//     CurrentThread->StateSaveArea = Add2Ptr(CurrentThread->StateSaveArea,
//                                            StackOffset);
//
//     /* Set the new stack limits */
//     CurrentThread->StackBase = StackBase;
//     CurrentThread->StackLimit = (ULONG_PTR)StackLimit;
//     CurrentThread->LargeStack = TRUE;
//
//     /* Adjust RspBase in the PCR */
//     Pcr = (PKIPCR)KeGetPcr();
//     Pcr->Prcb.RspBase += StackOffset;
//
//     /* Adjust Rsp0 in the TSS */
//     Pcr->TssBase->Rsp0 += StackOffset;
//
//     /* Restore interrupts */
//     __writeeflags(Eflags);
//
//     return OldStackBase;
// }
//
// DECLSPEC_NORETURN
// VOID
// KiIdleLoop(VOID)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD OldThread, NewThread;
//
//     /* Now loop forever */
//     while (TRUE)
//     {
//         /* Start of the idle loop: disable interrupts */
//         _enable();
//         YieldProcessor();
//         YieldProcessor();
//         _disable();
//
//         /* Check for pending timers, pending DPCs, or pending ready threads */
//         if ((Prcb->DpcData[0].DpcQueueDepth) ||
//             (Prcb->TimerRequest) ||
//             (Prcb->DeferredReadyListHead.Next))
//         {
//             /* Quiesce the DPC software interrupt */
//             HalClearSoftwareInterrupt(DISPATCH_LEVEL);
//
//             /* Handle it */
//             KiRetireDpcList(Prcb);
//         }
//
//         /* Check if a new thread is scheduled for execution */
//         if (Prcb->NextThread)
//         {
//             /* Enable interrupts */
//             _enable();
//
//             /* Capture current thread data */
//             OldThread = Prcb->CurrentThread;
//             NewThread = Prcb->NextThread;
//
//             /* Set new thread data */
//             Prcb->NextThread = NULL;
//             Prcb->CurrentThread = NewThread;
//
//             /* The thread is now running */
//             NewThread->State = Running;
//
// #ifdef CONFIG_SMP
//             /* Do the swap at SYNCH_LEVEL */
//             KfRaiseIrql(SYNCH_LEVEL);
// #endif
//
//             /* Switch away from the idle thread */
//             KiSwapContext(APC_LEVEL, OldThread);
//
// #ifdef CONFIG_SMP
//             /* Go back to DISPATCH_LEVEL */
//             KeLowerIrql(DISPATCH_LEVEL);
// #endif
//         }
//         else
//         {
//             /* Continue staying idle. Note the HAL returns with interrupts on */
//             Prcb->PowerState.IdleFunction(&Prcb->PowerState);
//         }
//     }
// }
//
// VOID
// NTAPI
// KiSwapProcess(IN PKPROCESS NewProcess,
//               IN PKPROCESS OldProcess)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//
// #ifdef CONFIG_SMP
//     /* Update active processor mask */
//     InterlockedXor64((PLONG64)&NewProcess->ActiveProcessors, Pcr->Prcb.SetMember);
//     InterlockedXor64((PLONG64)&OldProcess->ActiveProcessors, Pcr->Prcb.SetMember);
// #endif
//
//     /* Update CR3 */
//     __writecr3(NewProcess->DirectoryTableBase[0]);
//
//     /* Update IOPM offset */
//     Pcr->TssBase->IoMapBase = NewProcess->IopmOffset;
// }
//
// NTSTATUS
// NTAPI
// NtSetLdtEntries(ULONG Selector1, LDT_ENTRY LdtEntry1, ULONG Selector2, LDT_ENTRY LdtEntry2)
// {
//     UNIMPLEMENTED;
//     __debugbreak();
//     return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// NTAPI
// NtVdmControl(IN ULONG ControlCode,
//              IN PVOID ControlData)
// {
//     /* Not supported */
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
