use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    CONTEXT, KTRAP_FRAME, NtCurrentTeb, PCONTEXT, PETHREAD, PKTRAP_FRAME, PVDM_TIB, 
    PsGetCurrentThread, VDM_TIB,
};
use ntkrnl_sys::ke::KeGetCurrentIrql;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static KeI386EFlagsAndMaskV86: ULONG;
    static KeI386EFlagsOrMaskV86: ULONG;
    static KeI386VirtualIntExtensions: ULONG;
    fn Ki386AdjustEsp0(TrapFrame: PKTRAP_FRAME);
}
extern "C" {
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

extern "C" {
    static mut VdmState: *mut ULONG;
}

/// InterlockedOr — atomic OR on LONG, returns old value.
#[inline(always)]
unsafe fn InterlockedOr(target: *mut LONG, value: LONG) -> LONG {
    let target = target as *mut core::sync::atomic::AtomicI32;
    (*target).fetch_or(value, core::sync::atomic::Ordering::SeqCst)
}

/// InterlockedAnd — atomic AND on LONG, returns old value.
#[inline(always)]
unsafe fn InterlockedAnd(target: *mut LONG, value: LONG) -> LONG {
    let target = target as *mut core::sync::atomic::AtomicI32;
    (*target).fetch_and(value, core::sync::atomic::Ordering::SeqCst)
}

const EFLAGS_V86_MASK: ULONG = 0x00020000;
const EFLAGS_INTERRUPT_MASK: ULONG = 0x00000200;
const EFLAGS_VIF: ULONG = 0x00080000;
const EFLAGS_VIP: ULONG = 0x00100000;
const EFLAGS_USER_SANITIZE: ULONG = 0x003F4DD7;
const EXCEPTION_CHAIN_END: ULONG = 0xFFFFFFFF;
const KGDT_R3_CODE: USHORT = 0x18;
const KGDT_R0_CODE: USHORT = 0x08;
const RPL_MASK: USHORT = 0x0003;
const FRAME_EDITED: USHORT = 0xFFF8;

const VdmBop: ULONG = 4; // header value; the hand copy said 0
const VdmMemAccess: ULONG = 2; // header value; the hand copy said 1

#[no_mangle]
pub static mut VdmBopCount: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn VdmpGetVdmTib(VdmTib: *mut PVDM_TIB) -> NTSTATUS {
    let VdmTib = opaque_ptr(VdmTib);
    *VdmTib = core::ptr::null_mut();

    let Tib = (*NtCurrentTeb()).Vdm as PVDM_TIB;
    if Tib.is_null() {
        return STATUS_INVALID_SYSTEM_SERVICE;
    }

    if (*Tib).Size != core::mem::size_of::<VDM_TIB>() as ULONG {
        return STATUS_INVALID_SYSTEM_SERVICE;
    }

    *VdmTib = Tib;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn VdmSwapContext(
    TrapFrame: PKTRAP_FRAME,
    OutContext: PCONTEXT,
    InContext: PCONTEXT,
) {
    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == APC_LEVEL);
    ntkrnl_sys::ASSERT!((*TrapFrame).DbgArgMark == 0xBADB0D00);

    if (*TrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        (*OutContext).SegGs = (*TrapFrame).V86Gs;
        (*OutContext).SegFs = (*TrapFrame).V86Fs;
        (*OutContext).SegEs = (*TrapFrame).V86Es;
        (*OutContext).SegDs = (*TrapFrame).V86Ds;
    } else if (*TrapFrame).SegCs != (KGDT_R3_CODE | RPL_MASK) as ULONG {
        (*OutContext).SegGs = (*TrapFrame).SegGs;
        (*OutContext).SegFs = (*TrapFrame).SegFs;
        (*OutContext).SegEs = (*TrapFrame).SegEs;
        (*OutContext).SegDs = (*TrapFrame).SegDs;
    }

    (*OutContext).SegCs = (*TrapFrame).SegCs;
    (*OutContext).SegSs = (*TrapFrame).HardwareSegSs;

    (*OutContext).Eax = (*TrapFrame).Eax;
    (*OutContext).Ebx = (*TrapFrame).Ebx;
    (*OutContext).Ecx = (*TrapFrame).Ecx;
    (*OutContext).Edx = (*TrapFrame).Edx;
    (*OutContext).Esi = (*TrapFrame).Esi;
    (*OutContext).Edi = (*TrapFrame).Edi;

    (*OutContext).Ebp = (*TrapFrame).Ebp;
    (*OutContext).Esp = (*TrapFrame).HardwareEsp;
    (*OutContext).Eip = (*TrapFrame).Eip;

    (*OutContext).EFlags = (*TrapFrame).EFlags;

    (*TrapFrame).SegCs = (*InContext).SegCs;
    (*TrapFrame).HardwareSegSs = (*InContext).SegSs;

    (*TrapFrame).Eax = (*InContext).Eax;
    (*TrapFrame).Ebx = (*InContext).Ebx;
    (*TrapFrame).Ecx = (*InContext).Ecx;
    (*TrapFrame).Edx = (*InContext).Edx;
    (*TrapFrame).Esi = (*InContext).Esi;
    (*TrapFrame).Edi = (*InContext).Edi;

    (*TrapFrame).Ebp = (*InContext).Ebp;
    (*TrapFrame).HardwareEsp = (*InContext).Esp;
    (*TrapFrame).Eip = (*InContext).Eip;

    let mut EFlags = (*InContext).EFlags;
    if EFlags & EFLAGS_V86_MASK != 0 {
        EFlags &= KeI386EFlagsAndMaskV86;
        EFlags |= KeI386EFlagsOrMaskV86;
    } else {
        (*TrapFrame).SegCs |= RPL_MASK as ULONG;
        (*TrapFrame).HardwareSegSs |= RPL_MASK as ULONG;

        if (*TrapFrame).SegCs < KGDT_R0_CODE as ULONG {
            (*TrapFrame).SegCs = (KGDT_R3_CODE | RPL_MASK) as ULONG;
        }

        EFlags &= EFLAGS_USER_SANITIZE;
        EFlags |= EFLAGS_INTERRUPT_MASK;
    }

    let OldEFlags = (*TrapFrame).EFlags;
    (*TrapFrame).EFlags = EFlags;

    if (OldEFlags ^ EFlags) & EFLAGS_V86_MASK != 0 {
        Ki386AdjustEsp0(TrapFrame);
    }

    if (*InContext).EFlags & EFLAGS_V86_MASK != 0 {
        (*TrapFrame).V86Gs = (*InContext).SegGs;
        (*TrapFrame).V86Fs = (*InContext).SegFs;
        (*TrapFrame).V86Es = (*InContext).SegEs;
        (*TrapFrame).V86Ds = (*InContext).SegDs;
    } else {
        (*TrapFrame).SegGs = (*InContext).SegGs;
        (*TrapFrame).SegFs = (*InContext).SegFs;
        (*TrapFrame).SegEs = (*InContext).SegEs;
        (*TrapFrame).SegDs = (*InContext).SegDs;
    }

    (*TrapFrame).ExceptionList = EXCEPTION_CHAIN_END as *mut _;
}

#[no_mangle]
pub unsafe extern "system" fn VdmpStartExecution() -> NTSTATUS {
    let Thread = PsGetCurrentThread();
    let VdmFrame = ((*Thread).Tcb.InitialStack as usize
        - core::mem::size_of::<FX_SAVE_AREA>()
        - core::mem::size_of::<KTRAP_FRAME>()) as PKTRAP_FRAME;
    
    let mut VdmTib: PVDM_TIB = core::ptr::null_mut();
    let Status = VdmpGetVdmTib(&mut VdmTib);
    if Status < 0 {
        return STATUS_INVALID_SYSTEM_SERVICE;
    }

    let mut OldIrql: KIRQL = 0;
    // [guard] KeRaiseIrql(APC_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };

    let Interrupts = ((*VdmTib).VdmContext.EFlags & EFLAGS_INTERRUPT_MASK) != 0;

    ntkrnl_sys::ASSERT!(*VdmState == 0);
    ntkrnl_sys::ASSERT!((*VdmTib).VdmContext.EFlags & EFLAGS_V86_MASK != 0);

    if KeI386VirtualIntExtensions != 0 && ((*VdmTib).VdmContext.EFlags & EFLAGS_V86_MASK != 0) {
        if Interrupts {
            (*VdmTib).VdmContext.EFlags |= EFLAGS_VIF;
        } else {
            (*VdmTib).VdmContext.EFlags &= !EFLAGS_VIF;
            (*VdmTib).VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
        }
    } else {
        if (*VdmTib).VdmContext.EFlags & EFLAGS_INTERRUPT_MASK != 0 {
            InterlockedOr(VdmState as *mut LONG, EFLAGS_INTERRUPT_MASK as LONG);
        } else {
            InterlockedAnd(VdmState as *mut LONG, !(EFLAGS_INTERRUPT_MASK as LONG));
        }

        (*VdmTib).VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
    }

    let VdmContext = &(*VdmTib).VdmContext as *const _ as PCONTEXT;
    if (*VdmContext).SegCs & FRAME_EDITED as ULONG == 0 {
        // [guard-drop] KeLowerIrql(OldIrql);
        return STATUS_INVALID_SYSTEM_SERVICE;
    }

    VdmSwapContext(VdmFrame, &mut (*VdmTib).MonitorContext, VdmContext);

    // [guard-drop] KeLowerIrql(OldIrql);
    (*VdmFrame).Eax as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn VdmEndExecution(TrapFrame: PKTRAP_FRAME, VdmTib: PVDM_TIB) {
    ntkrnl_sys::ASSERT!(((*TrapFrame).EFlags & EFLAGS_V86_MASK != 0)
            || ((*TrapFrame).SegCs != (KGDT_R3_CODE | RPL_MASK) as ULONG));

    let mut OldIrql: KIRQL = 0;
    // [guard] KeRaiseIrql(APC_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };

    (*VdmTib).MonitorContext.Eax = STATUS_SUCCESS as ULONG;

    let Context = &(*VdmTib).MonitorContext as *const _ as PCONTEXT;

    if ((*Context).EFlags & EFLAGS_V86_MASK != 0) || ((*Context).SegCs & FRAME_EDITED as ULONG != 0) {
        VdmSwapContext(TrapFrame, &mut (*VdmTib).VdmContext, Context);

        if KeI386VirtualIntExtensions != 0 && ((*VdmTib).VdmContext.EFlags & EFLAGS_V86_MASK != 0) {
            if (*VdmTib).VdmContext.EFlags & EFLAGS_VIF != 0 {
                (*VdmTib).VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
            } else {
                (*VdmTib).VdmContext.EFlags &= !EFLAGS_INTERRUPT_MASK;
            }

            (*TrapFrame).EFlags &= !(EFLAGS_VIP | EFLAGS_VIF);
            (*VdmTib).VdmContext.EFlags &= !(EFLAGS_VIP | EFLAGS_VIF);
        } else {
            (*VdmTib).VdmContext.EFlags = ((*VdmTib).VdmContext.EFlags & !EFLAGS_INTERRUPT_MASK)
                | (*VdmState & EFLAGS_INTERRUPT_MASK);
        }
    }

    drop(irql_guard);
}

#[no_mangle]
pub unsafe extern "system" fn VdmDispatchBop(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    if (*TrapFrame).EFlags & EFLAGS_V86_MASK != 0 {
        let Eip = (((*TrapFrame).Eip & 0xFFFF) + (((*TrapFrame).SegCs & 0xFFFF) << 4)) as *mut u8;

        if *(Eip as *const u16) == 0xC4C4 {
            if *Eip.offset(2) == 0x50 {
                ntkrnl_sys::ASSERT!(false);
            }

            VdmBopCount += 1;

            let VdmTib = (*NtCurrentTeb()).Vdm as PVDM_TIB;

            (*VdmTib).EventInfo.InstructionSize = 3;
            (*VdmTib).EventInfo.__bindgen_anon_1.BopNumber = *Eip.offset(2) as ULONG;
            (*VdmTib).EventInfo.Event = VdmBop;

            VdmEndExecution(TrapFrame, VdmTib);
        } else {
            return 0u8;
        }
    } else {
        ntkrnl_sys::ASSERT!(false);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn VdmDispatchPageFault(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    let mut VdmTib: PVDM_TIB = core::ptr::null_mut();
    let mut Status = VdmpGetVdmTib(&mut VdmTib);
    if Status < 0 {
        return 0u8;
    }

    ntkrnl_sys::ASSERT!((*TrapFrame).EFlags & EFLAGS_V86_MASK != 0);

    (*VdmTib).EventInfo.Event = VdmMemAccess;
    (*VdmTib).EventInfo.InstructionSize = 0;

    VdmEndExecution(TrapFrame, VdmTib);

    if Status >= 0 { 1u8 } else { 0u8 }
}

#[repr(C)]
struct FX_SAVE_AREA {
    _dummy: [u8; 528],
}

const STATUS_INVALID_SYSTEM_SERVICE: NTSTATUS = 0xC000001C_u32 as i32;


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/vdm/vdmexec.c
//  * PURPOSE:         Support for executing VDM code and context swapping.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// ULONG VdmBopCount;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// VdmpGetVdmTib(OUT PVDM_TIB *VdmTib)
// {
//     PVDM_TIB Tib;
//     PAGED_CODE();
//
//     /* Assume vailure */
//     *VdmTib = NULL;
//
//     /* Get the current TIB */
//     Tib = NtCurrentTeb()->Vdm;
//     if (!Tib) return STATUS_INVALID_SYSTEM_SERVICE;
//
//     /* Validate the size */
//     if (Tib->Size != sizeof(VDM_TIB)) return STATUS_INVALID_SYSTEM_SERVICE;
//
//     /* Return it */
//     *VdmTib = Tib;
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// VdmSwapContext(IN PKTRAP_FRAME TrapFrame,
//                IN PCONTEXT OutContext,
//                IN PCONTEXT InContext)
// {
//     ULONG EFlags, OldEFlags;
//
//     /* Make sure that we're at APC_LEVEL and that this is a valid frame */
//     ASSERT(KeGetCurrentIrql() == APC_LEVEL);
//     ASSERT(TrapFrame->DbgArgMark == 0xBADB0D00);
//
//     /* Check if this is a V86 frame */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Copy segment registers */
//         OutContext->SegGs = TrapFrame->V86Gs;
//         OutContext->SegFs = TrapFrame->V86Fs;
//         OutContext->SegEs = TrapFrame->V86Es;
//         OutContext->SegDs = TrapFrame->V86Ds;
//     }
//     else if (TrapFrame->SegCs != (KGDT_R3_CODE | RPL_MASK))
//     {
//         /* This was kernel mode, copy segment registers */
//         OutContext->SegGs = TrapFrame->SegGs;
//         OutContext->SegFs = TrapFrame->SegFs;
//         OutContext->SegEs = TrapFrame->SegEs;
//         OutContext->SegDs = TrapFrame->SegDs;
//     }
//
//     /* Copy CS and SS */
//     OutContext->SegCs = TrapFrame->SegCs;
//     OutContext->SegSs = TrapFrame->HardwareSegSs;
//
//     /* Copy general purpose registers */
//     OutContext->Eax = TrapFrame->Eax;
//     OutContext->Ebx = TrapFrame->Ebx;
//     OutContext->Ecx = TrapFrame->Ecx;
//     OutContext->Edx = TrapFrame->Edx;
//     OutContext->Esi = TrapFrame->Esi;
//     OutContext->Edi = TrapFrame->Edi;
//
//     /* Copy stack and counter */
//     OutContext->Ebp = TrapFrame->Ebp;
//     OutContext->Esp = TrapFrame->HardwareEsp;
//     OutContext->Eip = TrapFrame->Eip;
//
//     /* Finally the flags */
//     OutContext->EFlags = TrapFrame->EFlags;
//
//     /* Now copy from the in frame to the trap frame */
//     TrapFrame->SegCs = InContext->SegCs;
//     TrapFrame->HardwareSegSs = InContext->SegSs;
//
//     /* Copy the general purpose registers */
//     TrapFrame->Eax = InContext->Eax;
//     TrapFrame->Ebx = InContext->Ebx;
//     TrapFrame->Ecx = InContext->Ecx;
//     TrapFrame->Edx = InContext->Edx;
//     TrapFrame->Esi = InContext->Esi;
//     TrapFrame->Edi = InContext->Edi;
//
//     /* Copy the stack and counter */
//     TrapFrame->Ebp = InContext->Ebp;
//     TrapFrame->HardwareEsp = InContext->Esp;
//     TrapFrame->Eip = InContext->Eip;
//
//     /* Check if the context is from V86 */
//     EFlags = InContext->EFlags;
//     if (EFlags & EFLAGS_V86_MASK)
//     {
//         /* Sanitize the flags for V86 */
//         EFlags &= KeI386EFlagsAndMaskV86;
//         EFlags |= KeI386EFlagsOrMaskV86;
//     }
//     else
//     {
//         /* Add RPL_MASK to segments */
//         TrapFrame->SegCs |= RPL_MASK;
//         TrapFrame->HardwareSegSs |= RPL_MASK;
//
//         /* Check for bogus CS */
//         if (TrapFrame->SegCs < KGDT_R0_CODE)
//         {
//             /* Set user-mode */
//             TrapFrame->SegCs = KGDT_R3_CODE | RPL_MASK;
//         }
//
//         /* Sanitize flags and add interrupt mask */
//         EFlags &= EFLAGS_USER_SANITIZE;
//         EFlags |=EFLAGS_INTERRUPT_MASK;
//     }
//
//     /* Save the new eflags */
//     OldEFlags = TrapFrame->EFlags;
//     TrapFrame->EFlags = EFlags;
//
//     /* Check if we need to fixup ESP0 */
//     if ((OldEFlags ^ EFlags) & EFLAGS_V86_MASK)
//     {
//         /* Fix it up */
//         Ki386AdjustEsp0(TrapFrame);
//     }
//
//     /* Check if this is a V86 context */
//     if (InContext->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Copy VDM segments */
//         TrapFrame->V86Gs = InContext->SegGs;
//         TrapFrame->V86Fs = InContext->SegFs;
//         TrapFrame->V86Es = InContext->SegEs;
//         TrapFrame->V86Ds = InContext->SegDs;
//     }
//     else
//     {
//         /* Copy monitor segments */
//         TrapFrame->SegGs = InContext->SegGs;
//         TrapFrame->SegFs = InContext->SegFs;
//         TrapFrame->SegEs = InContext->SegEs;
//         TrapFrame->SegDs = InContext->SegDs;
//     }
//
//     /* Clear the exception list and return */
//     TrapFrame->ExceptionList = EXCEPTION_CHAIN_END;
// }
//
// NTSTATUS
// NTAPI
// VdmpStartExecution(VOID)
// {
//     PETHREAD Thread = PsGetCurrentThread();
//     PKTRAP_FRAME VdmFrame;
//     NTSTATUS Status;
//     PVDM_TIB VdmTib;
//     BOOLEAN Interrupts;
//     KIRQL OldIrql;
//     CONTEXT VdmContext;
//     PAGED_CODE();
//
//     /* Get the thread's VDM frame and TIB */
//     VdmFrame = (PVOID)((ULONG_PTR)Thread->Tcb.InitialStack -
//                                   sizeof(FX_SAVE_AREA) -
//                                   sizeof(KTRAP_FRAME));
//     Status = VdmpGetVdmTib(&VdmTib);
//     if (!NT_SUCCESS(Status)) return STATUS_INVALID_SYSTEM_SERVICE;
//
//     /* Go to APC level */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Check if interrupts are enabled */
//     Interrupts = (BOOLEAN)(VdmTib->VdmContext.EFlags & EFLAGS_INTERRUPT_MASK);
//
//     /* We don't support full VDM yet, this shouldn't happen */
//     ASSERT(*VdmState == 0);
//     ASSERT(VdmTib->VdmContext.EFlags & EFLAGS_V86_MASK);
//
//     /* Check if VME is supported and V86 mode was enabled */
//     if ((KeI386VirtualIntExtensions) &&
//         (VdmTib->VdmContext.EFlags & EFLAGS_V86_MASK))
//     {
//         /* Check if interrupts are enabled */
//         if (Interrupts)
//         {
//             /* Set fake IF flag */
//             VdmTib->VdmContext.EFlags |= EFLAGS_VIF;
//         }
//         else
//         {
//             /* Remove fake IF flag, turn on real IF flag */
//             VdmTib->VdmContext.EFlags &= ~EFLAGS_VIF;
//             VdmTib->VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
//         }
//     }
//     else
//     {
//         /* Set interrupt state in the VDM State */
//         if (VdmTib->VdmContext.EFlags & EFLAGS_INTERRUPT_MASK)
//         {
//             /* Enable them as well */
//             InterlockedOr((PLONG)VdmState, EFLAGS_INTERRUPT_MASK);
//         }
//         else
//         {
//             /* Disable them */
//             InterlockedAnd((PLONG)VdmState, ~EFLAGS_INTERRUPT_MASK);
//         }
//
//         /* Enable the interrupt flag */
//         VdmTib->VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
//     }
//
//     /*  Get the VDM context and make sure it's not an edited frame */
//     VdmContext = VdmTib->VdmContext;
//     if (!(VdmContext.SegCs & FRAME_EDITED))
//     {
//         /* Fail */
//         KeLowerIrql(OldIrql);
//         return STATUS_INVALID_SYSTEM_SERVICE;
//     }
//
//     /* Now do the VDM Swap */
//     VdmSwapContext(VdmFrame, &VdmTib->MonitorContext, &VdmContext);
//
//     /* Lower the IRQL and return EAX */
//     KeLowerIrql(OldIrql);
//     return VdmFrame->Eax;
// }
//
// VOID
// NTAPI
// VdmEndExecution(IN PKTRAP_FRAME TrapFrame,
//                 IN PVDM_TIB VdmTib)
// {
//     KIRQL OldIrql;
//     CONTEXT Context;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT((TrapFrame->EFlags & EFLAGS_V86_MASK) ||
//            (TrapFrame->SegCs != (KGDT_R3_CODE | RPL_MASK)));
//
//     /* Raise to APC_LEVEL */
//     KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Set success */
//     VdmTib->MonitorContext.Eax = STATUS_SUCCESS;
//
//     /* Make a copy of the monitor context */
//     Context = VdmTib->MonitorContext;
//
//     /* Check if V86 mode was enabled or the trap was edited */
//     if ((Context.EFlags & EFLAGS_V86_MASK) || (Context.SegCs & FRAME_EDITED))
//     {
//         /* Switch contexts */
//         VdmSwapContext(TrapFrame, &VdmTib->VdmContext, &Context);
//
//         /* Check if VME is supported and V86 mode was enabled */
//         if ((KeI386VirtualIntExtensions) &&
//             (VdmTib->VdmContext.EFlags & EFLAGS_V86_MASK))
//         {
//             /* Check for VIF (virtual interrupt) flag state */
//             if (VdmTib->VdmContext.EFlags & EFLAGS_VIF)
//             {
//                 /* Set real IF flag */
//                 VdmTib->VdmContext.EFlags |= EFLAGS_INTERRUPT_MASK;
//             }
//             else
//             {
//                 /* Remove real IF flag */
//                 VdmTib->VdmContext.EFlags &= ~EFLAGS_INTERRUPT_MASK;
//             }
//
//             /* Turn off VIP and VIF */
//             TrapFrame->EFlags &= ~(EFLAGS_VIP | EFLAGS_VIF);
//             VdmTib->VdmContext.EFlags &= ~(EFLAGS_VIP | EFLAGS_VIF);
//         }
//         else
//         {
//             /* Set the EFLAGS based on our software copy of EFLAGS */
//             VdmTib->VdmContext.EFlags = (VdmTib->VdmContext.EFlags & ~EFLAGS_INTERRUPT_MASK) |
//                                         (*VdmState & EFLAGS_INTERRUPT_MASK);
//         }
//     }
//
//     /* Lower IRQL and reutrn */
//     KeLowerIrql(OldIrql);
// }
//
// BOOLEAN
// NTAPI
// VdmDispatchBop(IN PKTRAP_FRAME TrapFrame)
// {
//     PUCHAR Eip;
//     PVDM_TIB VdmTib;
//
//     /* Check if this is from V86 mode */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Calculate flat EIP */
//         Eip = (PUCHAR)((TrapFrame->Eip & 0xFFFF) +
//                       ((TrapFrame->SegCs & 0xFFFF) << 4));
//
//         /* Check if this is a BOP */
//         if (*(PUSHORT)Eip == 0xC4C4)
//         {
//             /* Check sure its the DOS Bop */
//             if (Eip[2] == 0x50)
//             {
//                 /* FIXME: No VDM Support */
//                 ASSERT(FALSE);
//             }
//
//             /* Increase the number of BOP operations */
//             VdmBopCount++;
//
//             /* Get the TIB */
//             VdmTib = NtCurrentTeb()->Vdm;
//
//             /* Fill out a VDM Event */
//             VdmTib->EventInfo.InstructionSize = 3;
//             VdmTib->EventInfo.BopNumber = Eip[2];
//             VdmTib->EventInfo.Event = VdmBop;
//
//             /* End VDM Execution */
//             VdmEndExecution(TrapFrame, VdmTib);
//         }
//         else
//         {
//             /* Not a BOP */
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* FIXME: Shouldn't happen on ROS */
//         ASSERT(FALSE);
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// VdmDispatchPageFault(
//     _In_ PKTRAP_FRAME TrapFrame)
// {
//     NTSTATUS Status;
//     PVDM_TIB VdmTib;
//
//     PAGED_CODE();
//
//     /* Get the VDM TIB so we can terminate V86 execution */
//     Status = VdmpGetVdmTib(&VdmTib);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Not a proper VDM fault, keep looking */
//         DPRINT1("VdmDispatchPageFault: no VDM TIB, Vdm=%p\n", NtCurrentTeb()->Vdm);
//         return FALSE;
//     }
//
//     /* Must be coming from V86 code */
//     ASSERT(TrapFrame->EFlags & EFLAGS_V86_MASK);
//
//     _SEH2_TRY
//     {
//         /* Fill out a VDM Event */
//         VdmTib->EventInfo.Event = VdmMemAccess;
//         VdmTib->EventInfo.InstructionSize = 0;
//
//         /* End VDM Execution */
//         VdmEndExecution(TrapFrame, VdmTib);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Consider the exception handled if we succeeded */
//     DPRINT1("VdmDispatchPageFault EFlags %lx exit with 0x%lx\n", TrapFrame->EFlags, Status);
//     return NT_SUCCESS(Status);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
