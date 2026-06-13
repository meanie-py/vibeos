use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;

// Use ddk::LARGE_INTEGER (= _LARGE_INTEGER) for bindgen-generated struct compatibility.
use ntkrnl_sys::ddk::LARGE_INTEGER as DdkLargeInteger;
use ntkrnl_sys::ddk::{
    CPU_INFO, ExInitPoolLookasidePointers, ExInitializeFastMutex, ExpInitializeExecutive,
    KI_SAMPLE_MAP, KeInitializeDpc, KeInitializeEvent, KeInitializeProcess,
    KeInitializeSpinLock, KeInitializeThread, KeQueryPerformanceCounter, KeSetPriorityThread,
    KeSetTargetProcessorDpc, KeStallExecutionProcessor, KiAcquirePrcbLock, KiComputeReciprocal,
    KiIdleLoop, KiInitSpinLocks, KiReleasePrcbLock, KiSaveProcessorControlState,
    KiTimerExpiration, MmCreateKernelStack, PKI_SAMPLE_MAP, PKPROCESS, PKTHREAD,
    PLOADER_PARAMETER_BLOCK, _KEVENT, _KDPC, _FAST_MUTEX,
    _KTIMER_TABLE_ENTRY, _KNODE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Most globals now come via ke_defs re-exports (KiTimerExpireDpc, KiProfileLock, etc.)
// Only declare symbols NOT already in ke_defs or from C-only sources
use crate::ke::clock::KeMaximumIncrement;
use crate::ke::timerobj::{KiTimeIncrementReciprocal, KiTimeIncrementShiftCount};
use crate::ke::procobj::KiProcessListHead;
use crate::ke::krnlinit::{KiServiceLimit, KeFeatureBits};
use crate::ke::thrdschd::KiIdleSummary;
use crate::ke::bug::{KeBugcheckCallbackListHead, KeBugcheckReasonCallbackListHead, BugCheckCallbackLock};
use crate::ke::krnlinit::{KeLoaderBlock, KeNodeBlock};
use crate::ke::ssdt::{MainSSDT, MainSSPT};
extern "C" {
    static mut RtlpUse16ByteSLists: BOOLEAN;
    fn KiCpuId(CpuInfo: *mut CPU_INFO, function: ULONG);
    fn __rdtsc() -> u64;
    fn KiGetCacheInformation();
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn _enable();
}
extern "system" {
    fn PoInitializePrcb(Prcb: PKPRCB);
}

const KF_RDTSC: ULONG = 0x00000002;
const KF_CMPXCHG16B: ULONG = 0x100000;
const HIGH_PRIORITY: i32 = 31;
const Running: i32 = 2;
const MAXCHAR: i8 = 127;
const TIMER_TABLE_SIZE: usize = 512;

// CODE_SEG("INIT")
unsafe fn KiCalculateCpuFrequency(Prcb: PKPRCB) {
    if (*Prcb).FeatureBits & KF_RDTSC != 0 {
        let mut Sample: ULONG = 0;
        let mut CpuInfo: CPU_INFO = core::mem::zeroed();
        let mut Samples: [KI_SAMPLE_MAP; 10] = core::mem::zeroed();
        let mut CurrentSample: PKI_SAMPLE_MAP = Samples.as_mut_ptr();

        loop {
            KiCpuId(&mut CpuInfo, 0);

            (*CurrentSample).PerfStart = KeQueryPerformanceCounter(core::ptr::null_mut());
            (*CurrentSample).TSCStart = __rdtsc() as i64;
            (*CurrentSample).PerfFreq.QuadPart = -50000;

            KeStallExecutionProcessor(((*CurrentSample).PerfFreq.QuadPart * -1 / 10) as ULONG);

            KiCpuId(&mut CpuInfo, 0);

            (*CurrentSample).PerfEnd = KeQueryPerformanceCounter(
                core::ptr::addr_of_mut!((*CurrentSample).PerfFreq),
            );
            (*CurrentSample).TSCEnd = __rdtsc() as i64;

            (*CurrentSample).PerfDelta =
                (*CurrentSample).PerfEnd.QuadPart - (*CurrentSample).PerfStart.QuadPart;
            (*CurrentSample).TSCDelta = (*CurrentSample).TSCEnd - (*CurrentSample).TSCStart;

            (*CurrentSample).MHz = (((*CurrentSample).TSCDelta as i64
                * (*CurrentSample).PerfFreq.QuadPart
                + 500000)
                / ((*CurrentSample).PerfDelta * 1000000)) as ULONG;

            if Sample != 0 {
                if (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz
                    || (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz + 1
                    || (*CurrentSample).MHz == (*CurrentSample.offset(-1)).MHz - 1
                {
                    break;
                }
            }

            CurrentSample = CurrentSample.offset(1);
            Sample += 1;

            if Sample == 10 {
                let mut TotalMHz: ULONG = 0;
                let mut i = Sample;
                while i > 0 {
                    i -= 1;
                    TotalMHz += Samples[i as usize].MHz;
                }
                (*CurrentSample.offset(-1)).MHz = TotalMHz / 10;
                break;
            }
        }

        (*Prcb).MHz = (*CurrentSample.offset(-1)).MHz;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiInitializeHandBuiltThread(
    Thread: PKTHREAD,
    Process: PKPROCESS,
    Stack: PVOID,
) {
    let Thread = opaque_ptr(Thread);
    let Process = opaque_ptr(Process);
    let Stack = opaque_ptr(Stack);
    let Prcb = KeGetCurrentPrcb();

    KeInitializeThread(
        Process,
        Thread,
        None,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Stack,
    );

    (*Thread).__bindgen_anon_1.__bindgen_anon_1.NextProcessor = (*Prcb).Number;
    (*Thread).IdealProcessor = (*Prcb).Number;
    (*Thread).Priority = HIGH_PRIORITY as i8;
    (*Thread).__bindgen_anon_5.__bindgen_anon_6.State = Running as u8;
    (*Thread).Affinity = (1u64) << (*Prcb).Number;
    (*Thread).__bindgen_anon_5.__bindgen_anon_6.WaitIrql = DISPATCH_LEVEL;
    (*Process).ActiveProcessors |= (1u64) << (*Prcb).Number;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiSystemStartupBootStack() -> ! {
    let LoaderBlock = KeLoaderBlock;
    let Prcb = KeGetCurrentPrcb();
    let Thread = (*KeLoaderBlock).Thread as PKTHREAD;
    let Process = (*Thread).__bindgen_anon_1.ApcState.Process;
    let KernelStack = (*KeLoaderBlock).KernelStack as PVOID;

    // Prcb->ParentNode = KeNodeBlock[0];
    (*Prcb).ParentNode = *(core::ptr::addr_of!(KeNodeBlock) as *const *mut _KNODE);
    // Prcb->ParentNode->ProcessorMask |= Prcb->SetMember;
    (*(*Prcb).ParentNode).ProcessorMask |= (*Prcb).SetMember;

    PoInitializePrcb(Prcb);

    KiSaveProcessorControlState(core::ptr::addr_of_mut!((*Prcb).ProcessorState));

    KiGetCacheInformation();

    KiInitSpinLocks(Prcb, (*Prcb).Number as i8);

    (*Prcb).CurrentThread = Thread;
    (*Prcb).NextThread = core::ptr::null_mut();
    (*Prcb).IdleThread = Thread;

    ExInitPoolLookasidePointers();

    KeLowerIrql(APC_LEVEL);

    if (*Prcb).Number == 0 {
        KiInitializeKernel(Process, Thread, KernelStack, Prcb, LoaderBlock);
    } else {
        KiInitializeHandBuiltThread(Thread, Process, KernelStack);
    }

    KiCalculateCpuFrequency(Prcb);

    KfRaiseIrql(DISPATCH_LEVEL);

    KeSetPriorityThread(Thread, 0);

    KiAcquirePrcbLock(Prcb);
    if (*Prcb).NextThread.is_null() {
        KiIdleSummary |= (1u64) << (*Prcb).Number;
    }
    KiReleasePrcbLock(Prcb);

    KfRaiseIrql(HIGH_LEVEL);
    (*LoaderBlock).Prcb = 0;

    let Thread = PsGetCurrentThread();
    (*Thread).Priority = 0;

    _enable();
    KeLowerIrql(DISPATCH_LEVEL);

    (*Thread).__bindgen_anon_5.__bindgen_anon_6.WaitIrql = DISPATCH_LEVEL;

    KiIdleLoop();
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeKernel(
    InitProcess: PKPROCESS,
    InitThread: PKTHREAD,
    IdleStack: PVOID,
    Prcb: PKPRCB,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let InitProcess = opaque_ptr(InitProcess);
    let InitThread = opaque_ptr(InitThread);
    let IdleStack = opaque_ptr(IdleStack);
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut PageDirectory: [u64; 2] = [0, 0];
    let DpcStack: PVOID;

    RtlpUse16ByteSLists = if KeFeatureBits & KF_CMPXCHG16B as u64 != 0 { 1u8 } else { 0u8 };

    (*Prcb).MultiThreadSetMaster = Prcb;

    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckCallbackListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KeBugcheckReasonCallbackListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(BugCheckCallbackLock) as *mut _);

    KeInitializeDpc(
        core::ptr::addr_of_mut!(KiTimerExpireDpc),
        Some(KiTimerExpiration),
        core::ptr::null_mut(),
    );
    KeSetTargetProcessorDpc(core::ptr::addr_of_mut!(KiTimerExpireDpc), 0);

    KeInitializeSpinLock(core::ptr::addr_of_mut!(KiProfileLock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProfileSourceListHead));

    for i in 0..TIMER_TABLE_SIZE {
        let tte = (core::ptr::addr_of_mut!(KiTimerTableListHead) as *mut _KTIMER_TABLE_ENTRY)
            .add(i);
        InitializeListHead(core::ptr::addr_of_mut!((*tte).Entry));
        (*tte).Time.u.HighPart = 0xFFFFFFFFu32;
        (*tte).Time.u.LowPart = 0;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(KiSwapEvent),
        1u32, // SynchronizationEvent
        0u8,
    );
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessInSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiProcessOutSwapListHead));
    InitializeListHead(core::ptr::addr_of_mut!(KiStackInSwapListHead));

    ExInitializeFastMutex(core::ptr::addr_of_mut!(KiGenericCallDpcMutex));

    KeServiceDescriptorTable[0].Base = core::ptr::addr_of_mut!(MainSSDT) as *mut _;
    KeServiceDescriptorTable[0].Count = core::ptr::null_mut();
    KeServiceDescriptorTable[0].Limit = KiServiceLimit;
    KeServiceDescriptorTable[1].Limit = 0;
    KeServiceDescriptorTable[0].Number = core::ptr::addr_of_mut!(MainSSPT) as *mut _;

    RtlCopyMemory(
        core::ptr::addr_of_mut!(KeServiceDescriptorTableShadow) as PVOID,
        core::ptr::addr_of!(KeServiceDescriptorTable) as PCVOID,
        core::mem::size_of::<[ntkrnl_sys::ddk::_KSERVICE_TABLE_DESCRIPTOR; 2]>(),
    );

    InitializeListHead(core::ptr::addr_of_mut!(KiProcessListHead));
    PageDirectory[0] = 0;
    PageDirectory[1] = 0;
    KeInitializeProcess(
        InitProcess,
        0,
        u64::MAX,
        PageDirectory.as_mut_ptr(),
        0u8,
    );
    (*InitProcess).QuantumReset = MAXCHAR;

    KiInitializeHandBuiltThread(InitThread, InitProcess, IdleStack);

    ExpInitializeExecutive(0, LoaderBlock);

    KiTimeIncrementReciprocal = KiComputeReciprocal(
        KeMaximumIncrement as i32,
        core::ptr::addr_of_mut!(KiTimeIncrementShiftCount),
    );

    (*Prcb).MaximumDpcQueueDepth = KiMaximumDpcQueueDepth as i32;
    (*Prcb).MinimumDpcRate = KiMinimumDpcRate;
    (*Prcb).AdjustDpcThreshold = KiAdjustDpcThreshold;

    DpcStack = MmCreateKernelStack(0u8, 0);
    if DpcStack.is_null() {
        KeBugCheckEx(NO_PAGES_AVAILABLE, 1, 0, 0, 0);
    }
    (*Prcb).DpcStack = DpcStack;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/amd64/krnlinit.c
//  * PURPOSE:         Portable part of kernel initialization
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern ULONG_PTR MainSSDT[];
// extern UCHAR MainSSPT[];
//
// extern BOOLEAN RtlpUse16ByteSLists;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernel(IN PKPROCESS InitProcess,
//                    IN PKTHREAD InitThread,
//                    IN PVOID IdleStack,
//                    IN PKPRCB Prcb,
//                    IN PLOADER_PARAMETER_BLOCK LoaderBlock);
//
//
// CODE_SEG("INIT")
// VOID
// KiCalculateCpuFrequency(
//     IN PKPRCB Prcb)
// {
//     if (Prcb->FeatureBits & KF_RDTSC)
//     {
//         ULONG Sample = 0;
//         CPU_INFO CpuInfo;
//         KI_SAMPLE_MAP Samples[10];
//         PKI_SAMPLE_MAP CurrentSample = Samples;
//
//         /* Start sampling loop */
//         for (;;)
//         {
//             /* Do a dummy CPUID to start the sample */
//             KiCpuId(&CpuInfo, 0);
//
//             /* Fill out the starting data */
//             CurrentSample->PerfStart = KeQueryPerformanceCounter(NULL);
//             CurrentSample->TSCStart = __rdtsc();
//             CurrentSample->PerfFreq.QuadPart = -50000;
//
//             /* Sleep for this sample */
//             KeStallExecutionProcessor(CurrentSample->PerfFreq.QuadPart * -1 / 10);
//
//             /* Do another dummy CPUID */
//             KiCpuId(&CpuInfo, 0);
//
//             /* Fill out the ending data */
//             CurrentSample->PerfEnd =
//                 KeQueryPerformanceCounter(&CurrentSample->PerfFreq);
//             CurrentSample->TSCEnd = __rdtsc();
//
//             /* Calculate the differences */
//             CurrentSample->PerfDelta = CurrentSample->PerfEnd.QuadPart -
//                                        CurrentSample->PerfStart.QuadPart;
//             CurrentSample->TSCDelta = CurrentSample->TSCEnd -
//                                       CurrentSample->TSCStart;
//
//             /* Compute CPU Speed */
//             CurrentSample->MHz = (ULONG)((CurrentSample->TSCDelta *
//                                           CurrentSample->
//                                           PerfFreq.QuadPart + 500000) /
//                                          (CurrentSample->PerfDelta *
//                                           1000000));
//
//             /* Check if this isn't the first sample */
//             if (Sample)
//             {
//                 /* Check if we got a good precision within 1MHz */
//                 if ((CurrentSample->MHz == CurrentSample[-1].MHz) ||
//                     (CurrentSample->MHz == CurrentSample[-1].MHz + 1) ||
//                     (CurrentSample->MHz == CurrentSample[-1].MHz - 1))
//                 {
//                     /* We did, stop sampling */
//                     break;
//                 }
//             }
//
//             /* Move on */
//             CurrentSample++;
//             Sample++;
//
//             if (Sample == RTL_NUMBER_OF(Samples))
//             {
//                 /* No luck. Average the samples and be done */
//                 ULONG TotalMHz = 0;
//                 while (Sample--)
//                 {
//                     TotalMHz += Samples[Sample].MHz;
//                 }
//                 CurrentSample[-1].MHz = TotalMHz / RTL_NUMBER_OF(Samples);
//                 DPRINT1("Sampling CPU frequency failed. Using average of %lu MHz\n", CurrentSample[-1].MHz);
//                 break;
//             }
//         }
//
//         /* Save the CPU Speed */
//         Prcb->MHz = CurrentSample[-1].MHz;
//     }
// }
//
// VOID
// NTAPI
// KiInitializeHandBuiltThread(
//     IN PKTHREAD Thread,
//     IN PKPROCESS Process,
//     IN PVOID Stack)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//
//     /* Setup the Thread */
//     KeInitializeThread(Process, Thread, NULL, NULL, NULL, NULL, NULL, Stack);
//
//     Thread->NextProcessor = Prcb->Number;
//     Thread->IdealProcessor = Prcb->Number;
//     Thread->Priority = HIGH_PRIORITY;
//     Thread->State = Running;
//     Thread->Affinity = (ULONG_PTR)1 << Prcb->Number;
//     Thread->WaitIrql = DISPATCH_LEVEL;
//     Process->ActiveProcessors |= (ULONG_PTR)1 << Prcb->Number;
//
// }
//
// CODE_SEG("INIT")
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiSystemStartupBootStack(VOID)
// {
//     PLOADER_PARAMETER_BLOCK LoaderBlock = KeLoaderBlock; // hack
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PKTHREAD Thread = (PKTHREAD)KeLoaderBlock->Thread;
//     PKPROCESS Process = Thread->ApcState.Process;
//     PVOID KernelStack = (PVOID)KeLoaderBlock->KernelStack;
//
//     /* Set Node Data */
//     Prcb->ParentNode = KeNodeBlock[0];
//     Prcb->ParentNode->ProcessorMask |= Prcb->SetMember;
//
//     /* Initialize the Power Management Support for this PRCB */
//     PoInitializePrcb(Prcb);
//
//     /* Save CPU state */
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//
//     /* Get cache line information for this CPU */
//     KiGetCacheInformation();
//
//     /* Initialize spinlocks and DPC data */
//     KiInitSpinLocks(Prcb, Prcb->Number);
//
//     /* Set up the thread-related fields in the PRCB */
//     Prcb->CurrentThread = Thread;
//     Prcb->NextThread = NULL;
//     Prcb->IdleThread = Thread;
//
//     /* Initialize PRCB pool lookaside pointers */
//     ExInitPoolLookasidePointers();
//
//     /* Lower to APC_LEVEL */
//     KeLowerIrql(APC_LEVEL);
//
//     /* Check if this is the boot cpu */
//     if (Prcb->Number == 0)
//     {
//         /* Initialize the kernel */
//         KiInitializeKernel(Process, Thread, KernelStack, Prcb, LoaderBlock);
//     }
//     else
//     {
//         /* Initialize the startup thread */
//         KiInitializeHandBuiltThread(Thread, Process, KernelStack);
//     }
//
//     /* Calculate the CPU frequency */
//     KiCalculateCpuFrequency(Prcb);
//
//     /* Raise to Dispatch */
//     KfRaiseIrql(DISPATCH_LEVEL);
//
//     /* Set the Idle Priority to 0. This will jump into Phase 1 */
//     KeSetPriorityThread(Thread, 0);
//
//     /* If there's no thread scheduled, put this CPU in the Idle summary */
//     KiAcquirePrcbLock(Prcb);
//     if (!Prcb->NextThread) KiIdleSummary |= (ULONG_PTR)1 << Prcb->Number;
//     KiReleasePrcbLock(Prcb);
//
//     /* Raise back to HIGH_LEVEL and clear the PRCB for the loader block */
//     KfRaiseIrql(HIGH_LEVEL);
//     LoaderBlock->Prcb = 0;
//
//     /* Set the priority of this thread to 0 */
//     Thread = KeGetCurrentThread();
//     Thread->Priority = 0;
//
//     /* Force interrupts enabled and lower IRQL back to DISPATCH_LEVEL */
//     _enable();
//     KeLowerIrql(DISPATCH_LEVEL);
//
//     /* Set the right wait IRQL */
//     Thread->WaitIrql = DISPATCH_LEVEL;
//
//     /* Jump into the idle loop */
//     KiIdleLoop();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeKernel(IN PKPROCESS InitProcess,
//                    IN PKTHREAD InitThread,
//                    IN PVOID IdleStack,
//                    IN PKPRCB Prcb,
//                    IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG_PTR PageDirectory[2];
//     PVOID DpcStack;
//     ULONG i;
//
//     /* Initialize 8/16 bit SList support */
//     RtlpUse16ByteSLists = (KeFeatureBits & KF_CMPXCHG16B) ? TRUE : FALSE;
//
//     /* Set the current MP Master KPRCB to the Boot PRCB */
//     Prcb->MultiThreadSetMaster = Prcb;
//
//     /* Initialize Bugcheck Callback data */
//     InitializeListHead(&KeBugcheckCallbackListHead);
//     InitializeListHead(&KeBugcheckReasonCallbackListHead);
//     KeInitializeSpinLock(&BugCheckCallbackLock);
//
//     /* Initialize the Timer Expiration DPC */
//     KeInitializeDpc(&KiTimerExpireDpc, KiTimerExpiration, NULL);
//     KeSetTargetProcessorDpc(&KiTimerExpireDpc, 0);
//
//     /* Initialize Profiling data */
//     KeInitializeSpinLock(&KiProfileLock);
//     InitializeListHead(&KiProfileListHead);
//     InitializeListHead(&KiProfileSourceListHead);
//
//     /* Loop the timer table */
//     for (i = 0; i < TIMER_TABLE_SIZE; i++)
//     {
//         /* Initialize the list and entries */
//         InitializeListHead(&KiTimerTableListHead[i].Entry);
//         KiTimerTableListHead[i].Time.HighPart = 0xFFFFFFFF;
//         KiTimerTableListHead[i].Time.LowPart = 0;
//     }
//
//     /* Initialize the Swap event and all swap lists */
//     KeInitializeEvent(&KiSwapEvent, SynchronizationEvent, FALSE);
//     InitializeListHead(&KiProcessInSwapListHead);
//     InitializeListHead(&KiProcessOutSwapListHead);
//     InitializeListHead(&KiStackInSwapListHead);
//
//     /* Initialize the mutex for generic DPC calls */
//     ExInitializeFastMutex(&KiGenericCallDpcMutex);
//
//     /* Initialize the syscall table */
//     KeServiceDescriptorTable[0].Base = MainSSDT;
//     KeServiceDescriptorTable[0].Count = NULL;
//     KeServiceDescriptorTable[0].Limit = KiServiceLimit;
//     KeServiceDescriptorTable[1].Limit = 0;
//     KeServiceDescriptorTable[0].Number = MainSSPT;
//
//     /* Copy the the current table into the shadow table for win32k */
//     RtlCopyMemory(KeServiceDescriptorTableShadow,
//                   KeServiceDescriptorTable,
//                   sizeof(KeServiceDescriptorTable));
//
//     /* Initialize the Idle Process and the Process Listhead */
//     InitializeListHead(&KiProcessListHead);
//     PageDirectory[0] = 0;
//     PageDirectory[1] = 0;
//     KeInitializeProcess(InitProcess,
//                         0,
//                         MAXULONG_PTR,
//                         PageDirectory,
//                         FALSE);
//     InitProcess->QuantumReset = MAXCHAR;
//
//     /* Initialize the startup thread */
//     KiInitializeHandBuiltThread(InitThread, InitProcess, IdleStack);
//
//     /* Initialize the Kernel Executive */
//     ExpInitializeExecutive(0, LoaderBlock);
//
//     /* Calculate the time reciprocal */
//     KiTimeIncrementReciprocal =
//         KiComputeReciprocal(KeMaximumIncrement,
//                             &KiTimeIncrementShiftCount);
//
//     /* Update DPC Values in case they got updated by the executive */
//     Prcb->MaximumDpcQueueDepth = KiMaximumDpcQueueDepth;
//     Prcb->MinimumDpcRate = KiMinimumDpcRate;
//     Prcb->AdjustDpcThreshold = KiAdjustDpcThreshold;
//
//     /* Allocate the DPC Stack */
//     DpcStack = MmCreateKernelStack(FALSE, 0);
//     if (!DpcStack) KeBugCheckEx(NO_PAGES_AVAILABLE, 1, 0, 0, 0);
//     Prcb->DpcStack = DpcStack;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
