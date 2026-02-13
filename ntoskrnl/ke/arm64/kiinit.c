/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/kiinit.c
 * PURPOSE:         ARM64 Kernel Initialization
 * PROGRAMMERS:     ReactOS ARM64 Port
 */

/* INCLUDES ******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

extern ULONG_PTR MainSSDT[];
extern UCHAR MainSSPT[];

/* Forward declarations */
DECLSPEC_NORETURN VOID NTAPI KiSystemStartupBootStack(VOID);
VOID KiSwitchToBootStack(ULONG_PTR InitialStack);

CODE_SEG("INIT")
VOID
NTAPI
KiInitializeKernel(
    _In_ PKPROCESS InitProcess,
    _In_ PKTHREAD InitThread,
    _In_ PVOID IdleStack,
    _In_ PKPRCB Prcb,
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock);

/* GLOBALS *******************************************************************/

/* Boot PCR for processor 0 */
KIPCR KiInitialPcr;

/* Boot and double-fault stack */
UCHAR DECLSPEC_ALIGN(16) KiP0BootStackData[KERNEL_STACK_SIZE] = {0};
UCHAR DECLSPEC_ALIGN(16) KiP0DoubleFaultStackData[KERNEL_STACK_SIZE] = {0};
PVOID KiP0BootStack = &KiP0BootStackData[KERNEL_STACK_SIZE];

/* Spinlock for freezing execution during init */
KSPIN_LOCK KiFreezeExecutionLock;

/* Function pointer for early debug prints (passed from FreeLDR) */
ULONG (*FrLdrDbgPrint)(const char *Format, ...);

/* Module list entries for core modules */
static LDR_DATA_TABLE_ENTRY LdrCoreEntries[3];

/* FUNCTIONS *****************************************************************/

static
VOID
KiInitializePcr(
    _Out_ PKIPCR Pcr,
    _In_ ULONG ProcessorNumber,
    _In_ PKTHREAD IdleThread)
{
    /* Zero out the PCR */
    RtlZeroMemory(Pcr, sizeof(KIPCR));

    /* Set self-pointer (KIPCR.Self maps to KPCR.Self) */
    Pcr->Self = (PKPCR)Pcr;

    /*
     * Set CurrentPrcb via the KPCR view.
     * KIPCR doesn't name it CurrentPrcb — it's PcrReserved0 at the same offset.
     * Cast to PKPCR to write the member that KeGetCurrentPrcb() reads.
     */
    ((PKPCR)Pcr)->CurrentPrcb = &Pcr->Prcb;

    /* Set the PCR Version */
    Pcr->MajorVersion = PCR_MAJOR_VERSION;
    Pcr->MinorVersion = PCR_MINOR_VERSION;

    /* Set the PRCB Version */
    Pcr->Prcb.MajorVersion = PRCB_MAJOR_VERSION;
    Pcr->Prcb.MinorVersion = PRCB_MINOR_VERSION;

    /* Set the Build Type */
    Pcr->Prcb.BuildType = 0;
#ifndef CONFIG_SMP
    Pcr->Prcb.BuildType |= PRCB_BUILD_UNIPROCESSOR;
#endif
#if DBG
    Pcr->Prcb.BuildType |= PRCB_BUILD_DEBUG;
#endif

    /* Set the Processor Number and current Processor Mask */
    Pcr->Prcb.Number = (UCHAR)ProcessorNumber;
    Pcr->Prcb.SetMember = 1ULL << ProcessorNumber;

    /* Set the Current Thread */
    Pcr->Prcb.CurrentThread = IdleThread;

    /* Start us out at PASSIVE_LEVEL */
    Pcr->CurrentIrql = PASSIVE_LEVEL;

    /* Setup the processor set */
    Pcr->Prcb.MultiThreadProcessorSet = Pcr->Prcb.SetMember;
}

CODE_SEG("INIT")
static
VOID
KiInitializeP0BootStructures(
    _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    /* Set the initial stack, idle thread and process for processor 0 */
    LoaderBlock->KernelStack = (ULONG_PTR)KiP0BootStack;
    LoaderBlock->Thread = (ULONG_PTR)&KiInitialThread;
    LoaderBlock->Process = (ULONG_PTR)&KiInitialProcess.Pcb;
    LoaderBlock->Prcb = (ULONG_PTR)&KiInitialPcr.Prcb;

    /* Initialize PCR for processor 0 */
    KiInitializePcr(&KiInitialPcr,
                    0,
                    &KiInitialThread.Tcb);
}

static
VOID
KiInitModuleList(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    PLDR_DATA_TABLE_ENTRY LdrEntry;
    PLIST_ENTRY Entry;
    ULONG i;

    /* Initialize the list head */
    InitializeListHead(&PsLoadedModuleList);

    /* Loop the first 3 entries (ntoskrnl, hal, kdcom) */
    for (Entry = LoaderBlock->LoadOrderListHead.Flink, i = 0;
         Entry != &LoaderBlock->LoadOrderListHead && i < 3;
         Entry = Entry->Flink, i++)
    {
        /* Get the data table entry */
        LdrEntry = CONTAINING_RECORD(Entry,
                                     LDR_DATA_TABLE_ENTRY,
                                     InLoadOrderLinks);

        /* Copy the entry */
        LdrCoreEntries[i] = *LdrEntry;

        /* Insert the copy into the list */
        InsertTailList(&PsLoadedModuleList, &LdrCoreEntries[i].InLoadOrderLinks);
    }
}

CODE_SEG("INIT")
VOID
NTAPI
KiInitMachineDependent(VOID)
{
    /* ARM64-specific hardware initialization */
    DPRINT1("KiInitMachineDependent: ARM64\n");
}

CODE_SEG("INIT")
DECLSPEC_NORETURN
VOID
NTAPI
KiSystemStartup(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    CCHAR Cpu;
    PKTHREAD InitialThread;
    PKIPCR Pcr;

    /* Get the current CPU number */
    Cpu = KeNumberProcessors++;

    /* LoaderBlock initialization for Cpu 0 */
    if (Cpu == 0)
    {
        /* Save the loader block */
        KeLoaderBlock = LoaderBlock;

        /* Prepare LoaderBlock, PCR with the P0 boot data */
        KiInitializeP0BootStructures(LoaderBlock);
    }

    /* Get Pcr from loader block */
    Pcr = CONTAINING_RECORD(LoaderBlock->Prcb, KIPCR, Prcb);

    /* Write PCR pointer to TPIDR_EL1 for KeGetPcr() */
    __asm__ __volatile__("msr tpidr_el1, %0" :: "r"((ULONG64)Pcr));

    /* Set the PRCB for this Processor */
    KiProcessorBlock[Cpu] = &Pcr->Prcb;

    /* Save the initial thread */
    InitialThread = (PKTHREAD)LoaderBlock->Thread;

    /* Set us as the current process */
    InitialThread->ApcState.Process = (PVOID)LoaderBlock->Process;

    /* Initial setup for the boot CPU */
    if (Cpu == 0)
    {
        /* Set processor architecture (12 = ARM64) */
        KeProcessorArchitecture = 12;
        KeProcessorLevel = 8;   /* ARMv8 */
        KeProcessorRevision = 0;

        /* Initialize the module list (ntos, hal, kdcom) */
        KiInitModuleList(LoaderBlock);

        /* Initialize debugging system */
        KdInitSystem(0, KeLoaderBlock);

        /* Check for break-in */
        if (KdPollBreakIn()) DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
    }

    DPRINT1("ARM64 KiSystemStartup: Pcr=%p Prcb=%p Thread=%p\n",
            Pcr, &Pcr->Prcb, InitialThread);

    /* Initialize the Processor with HAL */
    HalInitializeProcessor(Cpu, KeLoaderBlock);

    /* Set processor as active */
    KeActiveProcessors |= 1ULL << Cpu;

    /* Raise to HIGH_LEVEL */
    KfRaiseIrql(HIGH_LEVEL);

    /* Machine specific kernel initialization */
    if (Cpu == 0)
        KiInitMachineDependent();

    DPRINT1("ARM64 KiSystemStartup: Entering KiSystemStartupBootStack\n");

    /* Switch to boot stack and continue initialization */
    KiSwitchToBootStack((ULONG_PTR)LoaderBlock->KernelStack);
}

CODE_SEG("INIT")
DECLSPEC_NORETURN
VOID
NTAPI
KiSystemStartupBootStack(VOID)
{
    PLOADER_PARAMETER_BLOCK LoaderBlock = KeLoaderBlock;
    PKPRCB Prcb = KeGetCurrentPrcb();
    PKTHREAD Thread = (PKTHREAD)KeLoaderBlock->Thread;
    PKPROCESS Process = Thread->ApcState.Process;
    PVOID KernelStack = (PVOID)KeLoaderBlock->KernelStack;

    DPRINT1("ARM64 KiSystemStartupBootStack: Prcb=%p Thread=%p\n", Prcb, Thread);

    /* Set Node Data */
    Prcb->ParentNode = KeNodeBlock[0];
    Prcb->ParentNode->ProcessorMask |= Prcb->SetMember;

    /* Save CPU state (stub on ARM64) */
    KiSaveProcessorControlState(&Prcb->ProcessorState);

    /* Initialize spinlocks and DPC data */
    KiInitSpinLocks(Prcb, Prcb->Number);

    /* Set up the thread-related fields in the PRCB */
    Prcb->CurrentThread = Thread;
    Prcb->NextThread = NULL;
    Prcb->IdleThread = Thread;

    /* Lower to APC_LEVEL */
    KeLowerIrql(APC_LEVEL);

    /* Check if this is the boot cpu */
    if (Prcb->Number == 0)
    {
        DPRINT1("ARM64: Calling KiInitializeKernel\n");

        /* Initialize the kernel */
        KiInitializeKernel(Process, Thread, KernelStack, Prcb, LoaderBlock);
    }

    /* If we get here, KiInitializeKernel returned (unlikely in M2) */
    DPRINT1("ARM64: KiInitializeKernel returned or wasn't called\n");

    /* Raise back to HIGH_LEVEL */
    KfRaiseIrql(HIGH_LEVEL);
    LoaderBlock->Prcb = 0;

    /* Won't reach here in this milestone */
    KeBugCheck(0);
    while (TRUE);
}

CODE_SEG("INIT")
VOID
NTAPI
KiInitializeKernel(
    _In_ PKPROCESS InitProcess,
    _In_ PKTHREAD InitThread,
    _In_ PVOID IdleStack,
    _In_ PKPRCB Prcb,
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    ULONG_PTR PageDirectory[2];
    ULONG i;

    DPRINT1("ARM64 KiInitializeKernel: Process=%p Thread=%p Stack=%p Prcb=%p\n",
            InitProcess, InitThread, IdleStack, Prcb);

    /* Set the current MP Master KPRCB to the Boot PRCB */
    Prcb->MultiThreadSetMaster = Prcb;

    /* Initialize Bugcheck Callback data */
    InitializeListHead(&KeBugcheckCallbackListHead);
    InitializeListHead(&KeBugcheckReasonCallbackListHead);
    KeInitializeSpinLock(&BugCheckCallbackLock);

    /* Initialize the Timer Expiration DPC */
    KeInitializeDpc(&KiTimerExpireDpc, KiTimerExpiration, NULL);
    KeSetTargetProcessorDpc(&KiTimerExpireDpc, 0);

    /* Initialize Profiling data */
    KeInitializeSpinLock(&KiProfileLock);
    InitializeListHead(&KiProfileListHead);
    InitializeListHead(&KiProfileSourceListHead);

    /* Loop the timer table */
    for (i = 0; i < TIMER_TABLE_SIZE; i++)
    {
        /* Initialize the list and entries */
        InitializeListHead(&KiTimerTableListHead[i].Entry);
        KiTimerTableListHead[i].Time.HighPart = 0xFFFFFFFF;
        KiTimerTableListHead[i].Time.LowPart = 0;
    }

    /* Initialize the Swap event and all swap lists */
    KeInitializeEvent(&KiSwapEvent, SynchronizationEvent, FALSE);
    InitializeListHead(&KiProcessInSwapListHead);
    InitializeListHead(&KiProcessOutSwapListHead);
    InitializeListHead(&KiStackInSwapListHead);

    /* Initialize the mutex for generic DPC calls */
    ExInitializeFastMutex(&KiGenericCallDpcMutex);

    /* Initialize the syscall table */
    KeServiceDescriptorTable[0].Base = MainSSDT;
    KeServiceDescriptorTable[0].Count = NULL;
    KeServiceDescriptorTable[0].Limit = KiServiceLimit;
    KeServiceDescriptorTable[1].Limit = 0;
    KeServiceDescriptorTable[0].Number = MainSSPT;

    /* Copy the current table into the shadow table for win32k */
    RtlCopyMemory(KeServiceDescriptorTableShadow,
                  KeServiceDescriptorTable,
                  sizeof(KeServiceDescriptorTable));

    /* Initialize the Idle Process and the Process Listhead */
    InitializeListHead(&KiProcessListHead);
    PageDirectory[0] = 0;
    PageDirectory[1] = 0;
    KeInitializeProcess(InitProcess,
                        0,
                        MAXULONG_PTR,
                        PageDirectory,
                        FALSE);
    InitProcess->QuantumReset = MAXCHAR;

    DPRINT1("ARM64 KiInitializeKernel: Calling ExpInitializeExecutive\n");

    /* Initialize the Kernel Executive */
    ExpInitializeExecutive(0, LoaderBlock);

    /* If we get here, Phase 0 init succeeded! */
    DPRINT1("ARM64 KiInitializeKernel: ExpInitializeExecutive returned!\n");
}

/* EOF */
