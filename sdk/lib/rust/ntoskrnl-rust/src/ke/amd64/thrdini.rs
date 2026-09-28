use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    HalRequestSoftwareInterrupt,
    KEXCEPTION_FRAME,
    KTRAP_FRAME,
    KeContextToTrapFrame,
    KeGetPcr,
    KiThreadStartup,
    PCONTEXT,
    PKIPCR,
    PKPROCESS,
    PKSTART_FRAME,
    PKSTART_ROUTINE,
    PKSWITCH_FRAME,
    PKSYSTEM_ROUTINE,
    PKTHREAD,
    PKTRAP_FRAME,
    PXSAVE_AREA,
    _XSAVE_FORMAT,
    _KEXCEPTION_FRAME,
    _KSTART_FRAME,
    _KSWITCH_FRAME,
    _KTRAP_FRAME,
    _XSAVE_AREA,
};

use crate::ke::krnlinit::KeFeatureBits;
use super::stubs::KeXStateLength;
extern "C" {
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
    fn KiInvalidSystemThreadStartupExit();
    fn KiUserThreadStartupExit();
    fn KiServiceExit3();
    fn KiSaveXState(SaveArea: *mut _XSAVE_AREA, Mask: u64);
    fn KiRestoreXState(SaveArea: *mut _XSAVE_AREA, Mask: u64);
    fn __writecr3(value: u64);
    fn __writemsr(msr: ULONG, value: u64);
}

// KUSER_SHARED_DATA: bindgen layout (a shadow struct here guessed XState at
// +0x2E8 — a third inconsistent location; see amd64/kiinit.rs note).
use ntkrnl_sys::ddk::_KUSER_SHARED_DATA as KUSER_SHARED_DATA;

#[repr(C)]
struct KUINIT_FRAME {
    CtxSwitchFrame: _KSWITCH_FRAME,
    StartFrame: _KSTART_FRAME,
    ExceptionFrame: _KEXCEPTION_FRAME,
    TrapFrame: _KTRAP_FRAME,
}

#[repr(C)]
struct KKINIT_FRAME {
    CtxSwitchFrame: _KSWITCH_FRAME,
    StartFrame: _KSTART_FRAME,
}

const INITIAL_MXCSR: ULONG = 0x1f80;
const INITIAL_FPCSR: USHORT = 0x27f;
// ndk/amd64/ketypes.h: the bits amd64 cpu.rs KiGetFeatureBits actually sets. These were
// 1 << 25 / 1 << 26, bits nothing sets, so a new thread never got an XSAVE-format FPU area.
const KF_XSTATE: u64 = 0x0080_0000;
const KF_XSAVES: u64 = 0x0000_0040_0000_0000;
const XSTATE_MASK_LEGACY_FLOATING_POINT: u64 = 1 << 0;
const RPL_MASK: USHORT = 3;
const CONTEXT_AMD64: ULONG = 0x00100000;
const CONTEXT_CONTROL: ULONG = 0x00100001;
const CONTEXT_DEBUG_REGISTERS: ULONG = 0x00100010;
const MSR_GS_SWAP: ULONG = 0xC0000102;
const ATTEMPTED_SWITCH_FROM_DPC: ULONG = 0xB8;

#[macro_export]
macro_rules! ALIGN_DOWN_POINTER_BY {
    ($ptr:expr, $align:expr) => {
        (($ptr as usize) & !($align - 1)) as *mut _
    };
}

#[no_mangle]
pub unsafe extern "system" fn KiInitializeContextThread(
    Thread: PKTHREAD,
    SystemRoutine: PKSYSTEM_ROUTINE,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
    Context: PCONTEXT,
) {
    let Thread = opaque_ptr(Thread);
    let StartContext = opaque_ptr(StartContext);
    let mut InitialStack: PVOID;

    InitialStack = ((*Thread).InitialStack as *mut u8).sub(KeXStateLength as usize) as PVOID;
    InitialStack = ALIGN_DOWN_POINTER_BY!(InitialStack, 64);
    (*Thread).InitialStack = InitialStack;

    (*Thread).StateSaveArea = InitialStack as *mut _XSAVE_FORMAT;
    RtlZeroMemory((*Thread).StateSaveArea as PVOID, KeXStateLength as usize);
    (*(*Thread).StateSaveArea).MxCsr = INITIAL_MXCSR;
    (*(*Thread).StateSaveArea).ControlWord = INITIAL_FPCSR;

    if (KeFeatureBits & KF_XSTATE) != 0 {
        let XSaveArea = (*Thread).StateSaveArea as *mut _XSAVE_AREA;
        (*XSaveArea).Header.Mask |= XSTATE_MASK_LEGACY_FLOATING_POINT;

        if (KeFeatureBits & KF_XSAVES) != 0 {
            (*XSaveArea).Header.CompactionMask |= 0x8000000000000000u64 |
                                                  XSTATE_MASK_LEGACY_FLOATING_POINT;
        }
    }

    if !Context.is_null() {
        let InitFrame = ((*Thread).InitialStack as *mut KUINIT_FRAME).sub(1);
        let StartFrame = core::ptr::addr_of_mut!((*InitFrame).StartFrame);
        let CtxSwitchFrame = core::ptr::addr_of_mut!((*InitFrame).CtxSwitchFrame);

        (*Thread).KernelStack = InitFrame as PVOID;
        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 1i8;

        (*Thread).__bindgen_anon_5.__bindgen_anon_6.NpxState = (*SharedUserData).XState.EnabledFeatures as u8;
        (*Thread).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.NpxIrql = PASSIVE_LEVEL as u8;

        let ContextFlags = (*Context).ContextFlags & !CONTEXT_DEBUG_REGISTERS;
        let TrapFrame = core::ptr::addr_of_mut!((*InitFrame).TrapFrame);

        RtlZeroMemory(TrapFrame as PVOID, core::mem::size_of::<_KTRAP_FRAME>());
        RtlZeroMemory(
            core::ptr::addr_of_mut!((*InitFrame).ExceptionFrame) as PVOID,
            core::mem::size_of::<_KEXCEPTION_FRAME>(),
        );

        KeContextToTrapFrame(
            Context,
            core::ptr::addr_of_mut!((*InitFrame).ExceptionFrame),
            TrapFrame,
            CONTEXT_AMD64 | ContextFlags,
            1i8,
        );

        (*TrapFrame).SegSs |= RPL_MASK;
        (*TrapFrame).SegDs |= RPL_MASK;
        (*TrapFrame).SegEs |= RPL_MASK;
        (*TrapFrame).Dr7 = 0;
        (*TrapFrame).PreviousMode = 1i8;
        (*TrapFrame).__bindgen_anon_4.ExceptionFrame = 0;

        (*StartFrame).Return = KiUserThreadStartupExit as u64;
        (*InitFrame).ExceptionFrame.Return = KiServiceExit3 as u64;
    } else {
        let InitFrame = ((*Thread).InitialStack as *mut KKINIT_FRAME).sub(1);
        let StartFrame = core::ptr::addr_of_mut!((*InitFrame).StartFrame);
        let CtxSwitchFrame = core::ptr::addr_of_mut!((*InitFrame).CtxSwitchFrame);

        (*Thread).KernelStack = InitFrame as PVOID;
        (*Thread).__bindgen_anon_5.__bindgen_anon_2.PreviousMode = 0i8;
        (*Thread).__bindgen_anon_5.__bindgen_anon_6.NpxState = 0;

        (*StartFrame).Return = KiInvalidSystemThreadStartupExit as u64;
    }

    let CtxSwitchFrame = if !Context.is_null() {
        core::ptr::addr_of_mut!((*((*Thread).KernelStack as *mut KUINIT_FRAME)).CtxSwitchFrame)
    } else {
        core::ptr::addr_of_mut!((*((*Thread).KernelStack as *mut KKINIT_FRAME)).CtxSwitchFrame)
    };
    let StartFrame = if !Context.is_null() {
        core::ptr::addr_of_mut!((*((*Thread).KernelStack as *mut KUINIT_FRAME)).StartFrame)
    } else {
        core::ptr::addr_of_mut!((*((*Thread).KernelStack as *mut KKINIT_FRAME)).StartFrame)
    };

    (*CtxSwitchFrame).Return = KiThreadStartup as u64;
    (*CtxSwitchFrame).ApcBypass = 1u8;

    (*StartFrame).P1Home = StartRoutine.map(|f| f as u64).unwrap_or(0);
    (*StartFrame).P2Home = StartContext as u64;
    (*StartFrame).P3Home = 0;
    (*StartFrame).P4Home = SystemRoutine.map(|f| f as u64).unwrap_or(0);
    (*StartFrame).Reserved = 0;

    // cswitchdep birth brand: claim the shadow slot so a pool-reused KTHREAD
    // address never inherits a dead thread's record, and so the resume tap can
    // tell a first-run from a lost suspend record.
    #[cfg(feature = "cswitchdep")]
    crate::ke::cswitchdep::cswitch_thread_init(Thread as usize);
}

#[no_mangle]
pub unsafe extern "system" fn KiSwapContextResume(
    ApcBypass: BOOLEAN,
    OldThread: PKTHREAD,
    NewThread: PKTHREAD,
) -> BOOLEAN {
    let OldThread = opaque_ptr(OldThread);
    let NewThread = opaque_ptr(NewThread);
    let Pcr = KeGetPcr() as PKIPCR;
    let OldProcess: PKPROCESS;
    let NewProcess: PKPROCESS;

    (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0 = (*NewThread).InitialStack as u64;
    (*Pcr).Prcb.RspBase = (*(*Pcr).__bindgen_anon_1.__bindgen_anon_1.TssBase).Rsp0;

    // xstate save/restore — using forced fxsave64/fxrstor64 in C shim
    // to avoid xsaves64/xrstors64 crash on KVM
    if (*OldThread).__bindgen_anon_5.__bindgen_anon_6.NpxState != 0 {
        KiSaveXState((*OldThread).StateSaveArea as *mut _XSAVE_AREA, (*OldThread).__bindgen_anon_5.__bindgen_anon_6.NpxState as u64);
    }
    if (*NewThread).__bindgen_anon_5.__bindgen_anon_6.NpxState != 0 {
        KiRestoreXState((*NewThread).StateSaveArea as *mut _XSAVE_AREA, (*NewThread).__bindgen_anon_5.__bindgen_anon_6.NpxState as u64);
    }

    OldProcess = (*OldThread).__bindgen_anon_1.ApcState.Process;
    NewProcess = (*NewThread).__bindgen_anon_1.ApcState.Process;
    if OldProcess != NewProcess {
        __writecr3((*NewProcess).DirectoryTableBase[0]);
    }

    (*Pcr).__bindgen_anon_1.NtTib.Self_ = (*NewThread).Teb as *mut ntkrnl_sys::ddk::_NT_TIB;
    if !(*NewThread).Teb.is_null() {
        __writemsr(MSR_GS_SWAP, (*NewThread).Teb as u64);

        // WoW64 (Phase E, amd64): a 32-bit guest reads its TEB32 through
        // `fs` (selector KGDT64_R3_CMTEB = 0x53). That descriptor's base is
        // per-CPU and shared, so it must track the incoming thread's TEB32 on
        // every switch. TEB32 lives at TEB64 + 0x2000 (MmCreateTeb's combined
        // low region). Only the base bytes are rewritten — the boot GDT's
        // 32-bit-data/ring-3 type flags (Flags1/Flags2) stay intact.
        let eproc = NewProcess as *mut ntkrnl_sys::ddk::_EPROCESS;
        if !(*eproc).Wow64Process.is_null() {
            const KGDT64_R3_CMTEB: usize = 0x50; // selector == GDT byte offset
            let teb32 =
                (*NewThread).Teb as usize + ntkrnl_sys::wow64::WOW64_TEB32_OFFSET;
            let gdt = (*Pcr).__bindgen_anon_1.__bindgen_anon_1.GdtBase as usize;
            let cmteb = (gdt + KGDT64_R3_CMTEB) as *mut ntkrnl_sys::ddk::_KGDTENTRY64;
            (*cmteb).__bindgen_anon_1.BaseLow = (teb32 & 0xFFFF) as u16;
            (*cmteb).__bindgen_anon_1.__bindgen_anon_1.Bytes.BaseMiddle =
                ((teb32 >> 16) & 0xFF) as u8;
            (*cmteb).__bindgen_anon_1.__bindgen_anon_1.Bytes.BaseHigh =
                ((teb32 >> 24) & 0xFF) as u8;
            (*cmteb).__bindgen_anon_1.BaseUpper = 0; // TEB32 < 4GB
        }
    }

    (*Pcr).ContextSwitches += 1;
    (*NewThread).__bindgen_anon_5.__bindgen_anon_5.ContextSwitches += 1;

    // M3 SMP visibility: one-shot "[SMP] CPUn scheduling live" markers when
    // an AP switches onto a real (non-idle) thread — consumed by the -smp
    // smoke milestone check.
    #[cfg(feature = "smp")]
    {
        if (*Pcr).Prcb.Number != 0 && NewThread != (*Pcr).Prcb.IdleThread {
            crate::ke::ipi::KiSmpNoteApSwitch((*Pcr).Prcb.Number as u32);
        }
    }

    // cswitchdep tap 2 (post-swap, on the NEW thread's stack): IRQL still at
    // the switching level, CurrentThread coherence, SP within the new stack.
    // Runs BEFORE the ATTEMPTED_SWITCH_FROM_DPC BugCheck below so the record
    // reaches the table + serial even when the BugCheck then fires.
    #[cfg(feature = "cswitchdep")]
    {
        let sp_probe = 0u8;
        crate::ke::cswitchdep::cswitch_swap_resume(
            OldThread as usize,
            NewThread as usize,
            (*Pcr).Prcb.CurrentThread as usize,
            core::ptr::addr_of!(sp_probe) as usize,
            (*NewThread).StackLimit as usize,
            (*NewThread).InitialStack as usize,
            (*Pcr).Prcb.DpcRoutineActive != 0,
            NewThread as usize == (*Pcr).Prcb.IdleThread as usize,
            crate::code_addr!(),
        );
    }

    if (*Pcr).Prcb.DpcRoutineActive != 0 {
        KeBugCheckEx(
            ATTEMPTED_SWITCH_FROM_DPC,
            OldThread as ULONG_PTR,
            NewThread as ULONG_PTR,
            (*OldThread).InitialStack as ULONG_PTR,
            0,
        );
    }

    (*OldThread).SwapBusy = 0u8;

    if (*NewThread).__bindgen_anon_1.ApcState.KernelApcPending != 0 {
        if (*NewThread).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable == 0 && ApcBypass == 0 {
            return 1u8;
        }

        HalRequestSoftwareInterrupt(APC_LEVEL as u8);
    }

    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ke/amd64/thrdini.c
//  * PURPOSE:         amd64 Thread Context Creation
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern void KiInvalidSystemThreadStartupExit(void);
// extern void KiUserThreadStartupExit(void);
// extern void KiServiceExit3(void);
//
// typedef struct _KUINIT_FRAME
// {
//     KSWITCH_FRAME CtxSwitchFrame;
//     KSTART_FRAME StartFrame;
//     KEXCEPTION_FRAME ExceptionFrame;
//     KTRAP_FRAME TrapFrame;
//     //FX_SAVE_AREA FxSaveArea;
// } KUINIT_FRAME, *PKUINIT_FRAME;
//
// typedef struct _KKINIT_FRAME
// {
//     KSWITCH_FRAME CtxSwitchFrame;
//     KSTART_FRAME StartFrame;
//     //FX_SAVE_AREA FxSaveArea;
// } KKINIT_FRAME, *PKKINIT_FRAME;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// KiInitializeContextThread(IN PKTHREAD Thread,
//                            IN PKSYSTEM_ROUTINE SystemRoutine,
//                            IN PKSTART_ROUTINE StartRoutine,
//                            IN PVOID StartContext,
//                            IN PCONTEXT Context)
// {
//     PKSTART_FRAME StartFrame;
//     PKSWITCH_FRAME CtxSwitchFrame;
//     PKTRAP_FRAME TrapFrame;
//     ULONG ContextFlags;
//     PVOID InitialStack;
//
//     /* Allocate space on the stack for the XSAVE area */
//     InitialStack = (PUCHAR)Thread->InitialStack - KeXStateLength;
//     InitialStack = ALIGN_DOWN_POINTER_BY(InitialStack, 64);
//     Thread->InitialStack = InitialStack;
//
//     /* Initialize the state save area */
//     Thread->StateSaveArea = InitialStack;
//     RtlZeroMemory(Thread->StateSaveArea, KeXStateLength);
//     Thread->StateSaveArea->MxCsr = INITIAL_MXCSR;
//     Thread->StateSaveArea->ControlWord = INITIAL_FPCSR;
//
//     /* Check if we use XSAVE */
//     if (KeFeatureBits & KF_XSTATE)
//     {
//         /* Enable the mask for legacy floating point state */
//         PXSAVE_AREA XSaveArea = (PXSAVE_AREA)Thread->StateSaveArea;
//         XSaveArea->Header.Mask |= XSTATE_MASK_LEGACY_FLOATING_POINT;
//
//         /* Special initialization for XSAVES */
//         if (KeFeatureBits & KF_XSAVES)
//         {
//             /* Set bit 63 in XCOMP_BV to mark the area as compacted.
//                XRSTORS requires this and will #GP otherwise.
//                Also mark legacy FP as compacted. */
//             XSaveArea->Header.CompactionMask |= 0x8000000000000000ULL |
//                                                 XSTATE_MASK_LEGACY_FLOATING_POINT;
//         }
//     }
//
//     /* Check if this is a With-Context Thread */
//     if (Context)
//     {
//         PKUINIT_FRAME InitFrame;
//
//         /* Set up the Initial Frame */
//         InitFrame = ((PKUINIT_FRAME)Thread->InitialStack) - 1;
//         StartFrame = &InitFrame->StartFrame;
//         CtxSwitchFrame = &InitFrame->CtxSwitchFrame;
//
//         /* Save back the new value of the kernel stack. */
//         Thread->KernelStack = (PVOID)InitFrame;
//
//         /* Tell the thread it will run in User Mode */
//         Thread->PreviousMode = UserMode;
//
//         /* Set the Thread's NPX State */
//         Thread->NpxState = SharedUserData->XState.EnabledFeatures;
//         Thread->Header.NpxIrql = PASSIVE_LEVEL;
//
//         /* Make sure, we have control registers, disable debug registers */
//         ASSERT((Context->ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL);
//         ContextFlags = Context->ContextFlags & ~CONTEXT_DEBUG_REGISTERS;
//
//         /* Setup the Trap Frame */
//         TrapFrame = &InitFrame->TrapFrame;
//
//         /* Zero out the trap frame */
//         RtlZeroMemory(TrapFrame, sizeof(KTRAP_FRAME));
//         RtlZeroMemory(&InitFrame->ExceptionFrame, sizeof(KEXCEPTION_FRAME));
//
//         /* Set up a trap frame from the context. */
//         KeContextToTrapFrame(Context,
//                              &InitFrame->ExceptionFrame,
//                              TrapFrame,
//                              CONTEXT_AMD64 | ContextFlags,
//                              UserMode);
//
//         /* Set SS, DS, ES's RPL Mask properly */
//         TrapFrame->SegSs |= RPL_MASK;
//         TrapFrame->SegDs |= RPL_MASK;
//         TrapFrame->SegEs |= RPL_MASK;
//         TrapFrame->Dr7 = 0;
//
//         /* Set the previous mode as user */
//         TrapFrame->PreviousMode = UserMode;
//
//         /* Terminate the Exception Handler List */
//         TrapFrame->ExceptionFrame = 0;
//
//         /* KiThreadStartup returns to KiUserThreadStartupExit */
//         StartFrame->Return = (ULONG64)KiUserThreadStartupExit;
//
//         /* KiUserThreadStartupExit returns to KiServiceExit3 */
//         InitFrame->ExceptionFrame.Return = (ULONG64)KiServiceExit3;
//     }
//     else
//     {
//         PKKINIT_FRAME InitFrame;
//
//         /* Set up the Initial Frame for the system thread */
//         InitFrame = ((PKKINIT_FRAME)Thread->InitialStack) - 1;
//         StartFrame = &InitFrame->StartFrame;
//         CtxSwitchFrame = &InitFrame->CtxSwitchFrame;
//
//         /* Save back the new value of the kernel stack. */
//         Thread->KernelStack = (PVOID)InitFrame;
//
//         /* Tell the thread it will run in Kernel Mode */
//         Thread->PreviousMode = KernelMode;
//
//         /* No NPX State */
//         Thread->NpxState = 0;
//
//         /* This must never return! */
//         StartFrame->Return = (ULONG64)KiInvalidSystemThreadStartupExit;
//     }
//
//     /* Set up the Context Switch Frame */
//     CtxSwitchFrame->Return = (ULONG64)KiThreadStartup;
//     CtxSwitchFrame->ApcBypass = TRUE;
//
//     StartFrame->P1Home = (ULONG64)StartRoutine;
//     StartFrame->P2Home = (ULONG64)StartContext;
//     StartFrame->P3Home = 0;
//     StartFrame->P4Home = (ULONG64)SystemRoutine;
//     StartFrame->Reserved = 0;
// }
//
// BOOLEAN
// KiSwapContextResume(
//     _In_ BOOLEAN ApcBypass,
//     _In_ PKTHREAD OldThread,
//     _In_ PKTHREAD NewThread)
// {
//     PKIPCR Pcr = (PKIPCR)KeGetPcr();
//     PKPROCESS OldProcess, NewProcess;
//
//     /* Setup ring 0 stack pointer */
//     Pcr->TssBase->Rsp0 = (ULONG64)NewThread->InitialStack;
//     Pcr->Prcb.RspBase = Pcr->TssBase->Rsp0;
//
//     /* Save old thread's extended state */
//     if (OldThread->NpxState != 0)
//     {
//         KiSaveXState(OldThread->StateSaveArea, OldThread->NpxState);
//     }
//
//     /* Load new thread's extended state */
//     if (NewThread->NpxState != 0)
//     {
//         KiRestoreXState(NewThread->StateSaveArea, NewThread->NpxState);
//     }
//
//     /* Now we are the new thread. Check if it's in a new process */
//     OldProcess = OldThread->ApcState.Process;
//     NewProcess = NewThread->ApcState.Process;
//     if (OldProcess != NewProcess)
//     {
//         /* Switch address space and flush TLB */
//         __writecr3(NewProcess->DirectoryTableBase[0]);
//
//         /* Set new TSS fields */
//         //Pcr->TssBase->IoMapBase = NewProcess->IopmOffset;
//     }
//
//     /* Set TEB pointer and GS base */
//     Pcr->NtTib.Self = (PVOID)NewThread->Teb;
//     if (NewThread->Teb)
//     {
//        /* This will switch the usermode gs */
//        __writemsr(MSR_GS_SWAP, (ULONG64)NewThread->Teb);
//     }
//
//     /* Increase context switch count */
//     Pcr->ContextSwitches++;
//     NewThread->ContextSwitches++;
//
//     /* DPCs shouldn't be active */
//     if (Pcr->Prcb.DpcRoutineActive)
//     {
//         /* Crash the machine */
//         KeBugCheckEx(ATTEMPTED_SWITCH_FROM_DPC,
//                      (ULONG_PTR)OldThread,
//                      (ULONG_PTR)NewThread,
//                      (ULONG_PTR)OldThread->InitialStack,
//                      0);
//     }
//
//     /* Old thread os no longer busy */
//     OldThread->SwapBusy = FALSE;
//
//     /* Kernel APCs may be pending */
//     if (NewThread->ApcState.KernelApcPending)
//     {
//         /* Are APCs enabled? */
//         if ((NewThread->SpecialApcDisable == 0) &&
//             (ApcBypass == 0))
//         {
//             /* Return TRUE to indicate that we want APCs to be delivered */
//             return TRUE;
//         }
//
//         /* Request an APC interrupt to be delivered later */
//         HalRequestSoftwareInterrupt(APC_LEVEL);
//     }
//
//     /* Return stating that no kernel APCs are pending*/
//     return FALSE;
// }
//
// /* EOF */
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
