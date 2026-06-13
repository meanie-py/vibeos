use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _KIDTENTRY,
    _KGDTENTRY,
    _KIPCR,
    _ETHREAD,
    _KTSS,
    _KPROCESSOR_STATE,
    PKPROCESSOR_STATE,
    PKTHREAD,
    ExAllocatePoolZero,
    HalStartNextProcessor,
    KeMemoryBarrier,
    KiSystemStartup,
    MmCreateKernelStack,
    MmDeleteKernelStack,
};

const DOUBLE_FAULT_STACK_SIZE: usize = 0x3000;

#[repr(C, align(4096))]
struct APINFO {
    Idt: [_KIDTENTRY; 256],
    _pad_after_idt: [u8; PAGE_SIZE - (core::mem::size_of::<_KIDTENTRY>() * 256)],
    Gdt: [_KGDTENTRY; 128],
    _pad_after_gdt: [u8; PAGE_SIZE - (core::mem::size_of::<_KGDTENTRY>() * 128)],
    NMIStackData: core::mem::MaybeUninit<[u8; DOUBLE_FAULT_STACK_SIZE]>,
    Pcr: _KIPCR,
    Thread: _ETHREAD,
    Tss: _KTSS,
    TssDoubleFault: _KTSS,
    TssNMI: _KTSS,
}

#[repr(C)]
struct AP_SETUP_STACK {
    ReturnAddr: PVOID,
    KxLoaderBlock: PVOID,
}

use crate::ke::krnlinit::KeLoaderBlock;
use crate::ke::processor::{KeMaximumProcessors, KeNumprocSpecified, KeBootprocSpecified};
extern "system" {
    fn KiInitializePcr( ProcessorNumber: ULONG, Pcr: *mut _KIPCR, Idt: *mut _KIDTENTRY, Gdt: *mut _KGDTENTRY, Tss: *mut _KTSS, Thread: PKTHREAD, DpcStack: PVOID, );
    fn __readeflags() -> ULONG_PTR;
}
extern "C" {
    fn __sgdt(Descriptor: *mut u16);
    fn __sidt(Descriptor: *mut u16);
    fn __readcr0() -> ULONG_PTR;
    fn __readcr3() -> ULONG_PTR;
    fn __readcr4() -> ULONG_PTR;
    fn YieldProcessor();
    fn KiGetGdtEntry(Gdt: *mut _KGDTENTRY, Selector: USHORT) -> *mut _KGDTENTRY;
    fn KiSetGdtDescriptorBase(Entry: *mut _KGDTENTRY, Base: ULONG_PTR);
}

const KGDT_R0_CODE: USHORT = 0x08;
const KGDT_R0_DATA: USHORT = 0x10;
const KGDT_R3_DATA: USHORT = 0x1B;
const KGDT_R0_PCR: USHORT = 0x30;
const KGDT_TSS: USHORT = 0x28;
const KGDT_DF_TSS: USHORT = 0x50;
const KGDT_NMI_TSS: USHORT = 0x58;
const I386_TSS: u8 = 0x09;
const EFLAGS_INTERRUPT_MASK: ULONG_PTR = 0x200;

#[repr(C)]
struct KDESCRIPTOR {
    Limit: USHORT,
    Base: ULONG_PTR,
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KeStartAllProcessors() {
    let mut KernelStack: PVOID = core::ptr::null_mut();
    let mut DPCStack: PVOID = core::ptr::null_mut();
    let mut APInfo: *mut APINFO = core::ptr::null_mut();
    let mut ProcessorCount: ULONG;
    let mut MaximumProcessors: ULONG;

    MaximumProcessors = KeMaximumProcessors;

    if KeNumprocSpecified != 0 {
        MaximumProcessors = core::cmp::min(MaximumProcessors, KeNumprocSpecified);
    }

    if KeBootprocSpecified != 0 {
        MaximumProcessors = core::cmp::min(MaximumProcessors, KeBootprocSpecified);
    }

    ProcessorCount = 1;
    while ProcessorCount < MaximumProcessors {
        KernelStack = core::ptr::null_mut();
        DPCStack = core::ptr::null_mut();

        APInfo = ExAllocatePoolZero(
            NonPagedPool,
            core::mem::size_of::<APINFO>() as u32,
            TAG_KERNEL,
        ) as *mut APINFO;
        if APInfo.is_null() {
            break;
        }

        KernelStack = MmCreateKernelStack(0u8, 0);
        if KernelStack.is_null() {
            break;
        }

        DPCStack = MmCreateKernelStack(0u8, 0);
        if DPCStack.is_null() {
            break;
        }

        KiInitializePcr(
            ProcessorCount,
            core::ptr::addr_of_mut!((*APInfo).Pcr),
            core::ptr::addr_of_mut!((*APInfo).Idt[0]),
            core::ptr::addr_of_mut!((*APInfo).Gdt[0]),
            core::ptr::addr_of_mut!((*APInfo).Tss),
            core::ptr::addr_of_mut!((*APInfo).Thread) as PKTHREAD,
            DPCStack,
        );

        let mut bspGdt = core::mem::zeroed::<KDESCRIPTOR>();
        let mut bspIdt = core::mem::zeroed::<KDESCRIPTOR>();
        __sgdt(core::ptr::addr_of_mut!(bspGdt.Limit));
        __sidt(core::ptr::addr_of_mut!(bspIdt.Limit));
        
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*APInfo).Gdt) as PVOID,
            bspGdt.Base as PVOID,
            (bspGdt.Limit as usize) + 1,
        );
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*APInfo).Idt) as PVOID,
            bspIdt.Base as PVOID,
            (bspIdt.Limit as usize) + 1,
        );

        KiSetGdtDescriptorBase(
            KiGetGdtEntry(core::ptr::addr_of_mut!((*APInfo).Gdt[0]), KGDT_R0_PCR),
            core::ptr::addr_of_mut!((*APInfo).Pcr) as ULONG_PTR,
        );
        KiSetGdtDescriptorBase(
            KiGetGdtEntry(core::ptr::addr_of_mut!((*APInfo).Gdt[0]), KGDT_DF_TSS),
            core::ptr::addr_of_mut!((*APInfo).TssDoubleFault) as ULONG_PTR,
        );
        KiSetGdtDescriptorBase(
            KiGetGdtEntry(core::ptr::addr_of_mut!((*APInfo).Gdt[0]), KGDT_NMI_TSS),
            core::ptr::addr_of_mut!((*APInfo).TssNMI) as ULONG_PTR,
        );

        KiSetGdtDescriptorBase(
            KiGetGdtEntry(core::ptr::addr_of_mut!((*APInfo).Gdt[0]), KGDT_TSS),
            core::ptr::addr_of_mut!((*APInfo).Tss) as ULONG_PTR,
        );
        
        (*KiGetGdtEntry(core::ptr::addr_of_mut!((*APInfo).Gdt[0]), KGDT_TSS))
            .HighWord
            .Bits
            .set_Type(I386_TSS as ULONG);

        (*APInfo).TssDoubleFault.Esp0 = (*APInfo).NMIStackData.as_ptr() as ULONG_PTR as ULONG;
        (*APInfo).TssDoubleFault.Esp = (*APInfo).NMIStackData.as_ptr() as ULONG_PTR as ULONG;

        (*APInfo).TssNMI.Esp0 = (*APInfo).NMIStackData.as_ptr() as ULONG_PTR as ULONG;
        (*APInfo).TssNMI.Esp = (*APInfo).NMIStackData.as_ptr() as ULONG_PTR as ULONG;

        let ProcessorState = core::ptr::addr_of_mut!((*(*APInfo).Pcr.Prcb).ProcessorState) as PKPROCESSOR_STATE;
        RtlZeroMemory(
            ProcessorState as PVOID,
            core::mem::size_of::<_KPROCESSOR_STATE>(),
        );

        (*ProcessorState).SpecialRegisters.Cr0 = __readcr0() as ULONG;
        (*ProcessorState).SpecialRegisters.Cr3 = __readcr3() as ULONG;
        (*ProcessorState).SpecialRegisters.Cr4 = __readcr4() as ULONG;

        (*ProcessorState).ContextFrame.SegCs = KGDT_R0_CODE as ULONG;
        (*ProcessorState).ContextFrame.SegDs = KGDT_R3_DATA as ULONG;
        (*ProcessorState).ContextFrame.SegEs = KGDT_R3_DATA as ULONG;
        (*ProcessorState).ContextFrame.SegSs = KGDT_R0_DATA as ULONG;
        (*ProcessorState).ContextFrame.SegFs = KGDT_R0_PCR as ULONG;

        (*ProcessorState).SpecialRegisters.Gdtr.Base = core::ptr::addr_of_mut!((*APInfo).Gdt) as ULONG_PTR as ULONG;
        (*ProcessorState).SpecialRegisters.Gdtr.Limit = (core::mem::size_of_val(&(*APInfo).Gdt) - 1) as USHORT;
        (*ProcessorState).SpecialRegisters.Idtr.Base = core::ptr::addr_of_mut!((*APInfo).Idt) as ULONG_PTR as ULONG;
        (*ProcessorState).SpecialRegisters.Idtr.Limit = (core::mem::size_of_val(&(*APInfo).Idt) - 1) as USHORT;

        (*ProcessorState).SpecialRegisters.Tr = KGDT_TSS;

        (*ProcessorState).ContextFrame.Esp = KernelStack as ULONG_PTR as ULONG;
        (*ProcessorState).ContextFrame.Eip = KiSystemStartup as ULONG_PTR as ULONG;
        (*ProcessorState).ContextFrame.EFlags = (__readeflags() & !EFLAGS_INTERRUPT_MASK) as ULONG;

        (*ProcessorState).ContextFrame.Esp = ((*ProcessorState).ContextFrame.Esp as usize 
            - core::mem::size_of::<AP_SETUP_STACK>()) as ULONG;
        let ApStack = (*ProcessorState).ContextFrame.Esp as *mut AP_SETUP_STACK;
        (*ApStack).KxLoaderBlock = KeLoaderBlock as PVOID;
        (*ApStack).ReturnAddr = core::ptr::null_mut();

        *(KeLoaderBlock as *mut ULONG_PTR).offset(0) = KernelStack as ULONG_PTR;
        *(KeLoaderBlock as *mut ULONG_PTR).offset(1) = (*APInfo).Pcr.Prcb as ULONG_PTR;
        *(KeLoaderBlock as *mut ULONG_PTR).offset(2) = (*(*APInfo).Pcr.Prcb).IdleThread as ULONG_PTR;

        if HalStartNextProcessor(KeLoaderBlock as *mut _, ProcessorState) == 0 {
            break;
        }

        while *(KeLoaderBlock as *mut ULONG_PTR).offset(1) != 0 {
            KeMemoryBarrier();
            YieldProcessor();
        }

        ProcessorCount += 1;
    }

    ProcessorCount -= 1;

    if !APInfo.is_null() {
        ExFreePoolWithTag(APInfo as PVOID, TAG_KERNEL);
    }
    if !KernelStack.is_null() {
        MmDeleteKernelStack(KernelStack, 0u8);
    }
    if !DPCStack.is_null() {
        MmDeleteKernelStack(DPCStack, 0u8);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Architecture specific source file to hold multiprocessor functions
//  * COPYRIGHT:   Copyright 2023 Justin Miller <justin.miller@reactos.org>
//  *              Copyright 2023 Victor Perevertkin <victor.perevertkin@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _APINFO
// {
//     DECLSPEC_ALIGN(PAGE_SIZE) KIDTENTRY Idt[256];
//     DECLSPEC_ALIGN(PAGE_SIZE) KGDTENTRY Gdt[128];
//     DECLSPEC_ALIGN(16) UINT8 NMIStackData[DOUBLE_FAULT_STACK_SIZE];
//     KIPCR Pcr;
//     ETHREAD Thread;
//     KTSS Tss;
//     KTSS TssDoubleFault;
//     KTSS TssNMI;
// } APINFO, *PAPINFO;
//
// typedef struct _AP_SETUP_STACK
// {
//     PVOID ReturnAddr;
//     PVOID KxLoaderBlock;
// } AP_SETUP_STACK, *PAP_SETUP_STACK; // Note: expected layout only for 32-bit x86
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KeStartAllProcessors(VOID)
// {
//     PVOID KernelStack, DPCStack;
//     PAPINFO APInfo;
//     ULONG ProcessorCount;
//     ULONG MaximumProcessors;
//
//     /* NOTE: NT6+ HAL exports HalEnumerateProcessors() and
//      * HalQueryMaximumProcessorCount() that help determining
//      * the number of detected processors on the system. */
//     MaximumProcessors = KeMaximumProcessors;
//
//     /* Limit the number of processors we can start at run-time */
//     if (KeNumprocSpecified)
//         MaximumProcessors = min(MaximumProcessors, KeNumprocSpecified);
//
//     /* Limit also the number of processors we can start during boot-time */
//     if (KeBootprocSpecified)
//         MaximumProcessors = min(MaximumProcessors, KeBootprocSpecified);
//
//     // TODO: Support processor nodes
//
//     /* Start ProcessorCount at 1 because we already have the boot CPU */
//     for (ProcessorCount = 1; ProcessorCount < MaximumProcessors; ++ProcessorCount)
//     {
//         KernelStack = NULL;
//         DPCStack = NULL;
//
//         // Allocate structures for a new CPU.
//         APInfo = ExAllocatePoolZero(NonPagedPool, sizeof(*APInfo), TAG_KERNEL);
//         if (!APInfo)
//             break;
//         ASSERT(ALIGN_DOWN_POINTER_BY(APInfo, PAGE_SIZE) == APInfo);
//
//         KernelStack = MmCreateKernelStack(FALSE, 0);
//         if (!KernelStack)
//             break;
//
//         DPCStack = MmCreateKernelStack(FALSE, 0);
//         if (!DPCStack)
//             break;
//
//         // Initalize a new PCR for the specific AP
//         KiInitializePcr(ProcessorCount,
//                         &APInfo->Pcr,
//                         &APInfo->Idt[0],
//                         &APInfo->Gdt[0],
//                         &APInfo->Tss,
//                         (PKTHREAD)&APInfo->Thread,
//                         DPCStack);
//
//         // Prepare descriptor tables
//         KDESCRIPTOR bspGdt, bspIdt;
//         __sgdt(&bspGdt.Limit);
//         __sidt(&bspIdt.Limit);
//         RtlCopyMemory(&APInfo->Gdt, (PVOID)bspGdt.Base, bspGdt.Limit + 1);
//         RtlCopyMemory(&APInfo->Idt, (PVOID)bspIdt.Base, bspIdt.Limit + 1);
//
//         KiSetGdtDescriptorBase(KiGetGdtEntry(&APInfo->Gdt, KGDT_R0_PCR), (ULONG_PTR)&APInfo->Pcr);
//         KiSetGdtDescriptorBase(KiGetGdtEntry(&APInfo->Gdt, KGDT_DF_TSS), (ULONG_PTR)&APInfo->TssDoubleFault);
//         KiSetGdtDescriptorBase(KiGetGdtEntry(&APInfo->Gdt, KGDT_NMI_TSS), (ULONG_PTR)&APInfo->TssNMI);
//
//         KiSetGdtDescriptorBase(KiGetGdtEntry(&APInfo->Gdt, KGDT_TSS), (ULONG_PTR)&APInfo->Tss);
//         // Clear TSS Busy flag (aka set the type to "TSS (Available)")
//         KiGetGdtEntry(&APInfo->Gdt, KGDT_TSS)->HighWord.Bits.Type = I386_TSS;
//
//         APInfo->TssDoubleFault.Esp0 = (ULONG_PTR)&APInfo->NMIStackData;
//         APInfo->TssDoubleFault.Esp = (ULONG_PTR)&APInfo->NMIStackData;
//
//         APInfo->TssNMI.Esp0 = (ULONG_PTR)&APInfo->NMIStackData;
//         APInfo->TssNMI.Esp = (ULONG_PTR)&APInfo->NMIStackData;
//
//         // Fill the processor state
//         PKPROCESSOR_STATE ProcessorState = &APInfo->Pcr.Prcb->ProcessorState;
//         RtlZeroMemory(ProcessorState, sizeof(*ProcessorState));
//
//         ProcessorState->SpecialRegisters.Cr0 = __readcr0();
//         ProcessorState->SpecialRegisters.Cr3 = __readcr3();
//         ProcessorState->SpecialRegisters.Cr4 = __readcr4();
//
//         ProcessorState->ContextFrame.SegCs = KGDT_R0_CODE;
//         ProcessorState->ContextFrame.SegDs = KGDT_R3_DATA;
//         ProcessorState->ContextFrame.SegEs = KGDT_R3_DATA;
//         ProcessorState->ContextFrame.SegSs = KGDT_R0_DATA;
//         ProcessorState->ContextFrame.SegFs = KGDT_R0_PCR;
//
//         ProcessorState->SpecialRegisters.Gdtr.Base = (ULONG_PTR)APInfo->Gdt;
//         ProcessorState->SpecialRegisters.Gdtr.Limit = sizeof(APInfo->Gdt) - 1;
//         ProcessorState->SpecialRegisters.Idtr.Base = (ULONG_PTR)APInfo->Idt;
//         ProcessorState->SpecialRegisters.Idtr.Limit = sizeof(APInfo->Idt) - 1;
//
//         ProcessorState->SpecialRegisters.Tr = KGDT_TSS;
//
//         ProcessorState->ContextFrame.Esp = (ULONG_PTR)KernelStack;
//         ProcessorState->ContextFrame.Eip = (ULONG_PTR)KiSystemStartup;
//         ProcessorState->ContextFrame.EFlags = __readeflags() & ~EFLAGS_INTERRUPT_MASK;
//
//         ProcessorState->ContextFrame.Esp = (ULONG)((ULONG_PTR)ProcessorState->ContextFrame.Esp - sizeof(AP_SETUP_STACK));
//         PAP_SETUP_STACK ApStack = (PAP_SETUP_STACK)ProcessorState->ContextFrame.Esp;
//         ApStack->KxLoaderBlock = KeLoaderBlock;
//         ApStack->ReturnAddr = NULL;
//
//         // Update the LOADER_PARAMETER_BLOCK structure for the new processor
//         KeLoaderBlock->KernelStack = (ULONG_PTR)KernelStack;
//         KeLoaderBlock->Prcb = (ULONG_PTR)&APInfo->Pcr.Prcb;
//         KeLoaderBlock->Thread = (ULONG_PTR)&APInfo->Pcr.Prcb->IdleThread;
//
//         // Start the CPU
//         DPRINT("Attempting to Start a CPU with number: %lu\n", ProcessorCount);
//         if (!HalStartNextProcessor(KeLoaderBlock, ProcessorState))
//         {
//             break;
//         }
//
//         // And wait for it to start
//         while (KeLoaderBlock->Prcb != 0)
//         {
//             //TODO: Add a time out so we don't wait forever
//             KeMemoryBarrier();
//             YieldProcessor();
//         }
//     }
//
//     // The last CPU didn't start - clean the data
//     ProcessorCount--;
//
//     if (APInfo)
//         ExFreePoolWithTag(APInfo, TAG_KERNEL);
//     if (KernelStack)
//         MmDeleteKernelStack(KernelStack, FALSE);
//     if (DPCStack)
//         MmDeleteKernelStack(DPCStack, FALSE);
//
//     DPRINT1("KeStartAllProcessors: Successful AP startup count is %lu\n", ProcessorCount);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
