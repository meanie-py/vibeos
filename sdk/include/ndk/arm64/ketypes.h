

#ifndef _ARM64_KETYPES_H
#define _ARM64_KETYPES_H

#ifdef __cplusplus
extern "C" {
#endif

/* Interrupt request levels */
#define PASSIVE_LEVEL           0
#define LOW_LEVEL               0
#define APC_LEVEL               1
#define DISPATCH_LEVEL          2
#define CMCI_LEVEL              5
#define CLOCK_LEVEL             13
#define IPI_LEVEL               14
#define DRS_LEVEL               14
#define POWER_LEVEL             14
#define PROFILE_LEVEL           15
#define HIGH_LEVEL              15

//
// Synchronization-level IRQL
//
#ifndef CONFIG_SMP
#define SYNCH_LEVEL             DISPATCH_LEVEL
#else
#define SYNCH_LEVEL             (IPI_LEVEL - 2)
#endif

//
// IPI Types
//
#define IPI_APC                 1
#define IPI_DPC                 2
#define IPI_FREEZE              4
#define IPI_PACKET_READY        6
#define IPI_SYNCH_REQUEST       16

//
// PRCB Flags
//
#define PRCB_MAJOR_VERSION      1
#define PRCB_MINOR_VERSION      1
#define PRCB_BUILD_DEBUG        1
#define PRCB_BUILD_UNIPROCESSOR 2

//
// No LDTs on ARM64
//
#define LDT_ENTRY              ULONG

//
// Pool Lookaside List count
//
#define NUMBER_POOL_LOOKASIDE_LISTS 32

//
// On AMD64 and ARM64, GENERAL_LOOKASIDE_POOL is the same as PP_LOOKASIDE_LIST
//
#define GENERAL_LOOKASIDE_POOL PP_LOOKASIDE_LIST


//
// HAL Variables
//
#define INITIAL_STALL_COUNT     100
#define MM_HAL_VA_START         0xFFFFFFFFFFC00000ULL
#define MM_HAL_VA_END           0xFFFFFFFFFFFFFFFFULL

//
// Double fault stack size
//
#define DOUBLE_FAULT_STACK_SIZE 0x8000

//
// Static Kernel-Mode Address start (use MM_KSEG0_BASE for actual)
//
#define KSEG0_BASE 0xFFFF800000000000ULL

//
// Address of the user-shared data (same location as AMD64)
//
#define KI_USER_SHARED_DATA 0xFFFFF78000000000ULL

//
// Structure for CPUID info
//
typedef union _CPU_INFO
{
    ULONG dummy;
} CPU_INFO, *PCPU_INFO;

typedef struct _KTRAP_FRAME
{
    UCHAR ExceptionActive;
    UCHAR ContextFromKFramesUnwound;
    UCHAR DebugRegistersValid;
    union
    {
        struct
        {
            CHAR PreviousMode;
            UCHAR PreviousIrql;
        };
    };
    ULONG Reserved;
    union
    {
        struct
        {
            ULONG64 FaultAddress;
            ULONG64 TrapFrame;
        };
    };
    //struct PKARM64_VFP_STATE VfpState;
    ULONG VfpState;
    ULONG Bcr[8];
    ULONG64 Bvr[8];
    ULONG Wcr[2];
    ULONG64 Wvr[2];
    ULONG Spsr;
    ULONG Esr;
    ULONG64 Sp;
    union
    {
        ULONG64 X[19];
        struct
        {
            ULONG64 X0;
            ULONG64 X1;
            ULONG64 X2;
            ULONG64 X3;
            ULONG64 X4;
            ULONG64 X5;
            ULONG64 X6;
            ULONG64 X7;
            ULONG64 X8;
            ULONG64 X9;
            ULONG64 X10;
            ULONG64 X11;
            ULONG64 X12;
            ULONG64 X13;
            ULONG64 X14;
            ULONG64 X15;
            ULONG64 X16;
            ULONG64 X17;
            ULONG64 X18;
        };
    };
    ULONG64 Lr;
    ULONG64 Fp;
    ULONG64 Pc;
} KTRAP_FRAME, *PKTRAP_FRAME;

/*
 * ARM64 Exception Frame - callee-saved registers preserved during context switch.
 * Saved/restored by KiSwapContext. No SIMD (d8-d15) for now.
 */
typedef struct _KEXCEPTION_FRAME
{
    ULONG64 X19;
    ULONG64 X20;
    ULONG64 X21;
    ULONG64 X22;
    ULONG64 X23;
    ULONG64 X24;
    ULONG64 X25;
    ULONG64 X26;
    ULONG64 X27;
    ULONG64 X28;
    ULONG64 Fp;       /* X29 - frame pointer */
    ULONG64 Return;   /* X30 - link register (return address) */
} KEXCEPTION_FRAME, *PKEXCEPTION_FRAME;
/* Size: 96 bytes (0x60), 16-byte aligned */

/*
 * ARM64 Callout Frame — saved by KiCallUserMode (setjmp), restored by
 * KiCallbackReturn (longjmp).  Lives on the kernel stack during a
 * KeUserModeCallback round-trip.
 */
typedef struct _KCALLOUT_FRAME
{
    /* Callee-saved registers (saved/restored by assembly) */
    ULONG64 X19;            // +0x00
    ULONG64 X20;            // +0x08
    ULONG64 X21;            // +0x10
    ULONG64 X22;            // +0x18
    ULONG64 X23;            // +0x20
    ULONG64 X24;            // +0x28
    ULONG64 X25;            // +0x30
    ULONG64 X26;            // +0x38
    ULONG64 X27;            // +0x40
    ULONG64 X28;            // +0x48
    ULONG64 Fp;             // +0x50  (X29)
    ULONG64 Lr;             // +0x58  (X30 — return address to KeUserModeCallback)
    /* Callback-specific state (saved/restored by C code) */
    ULONG64 OutputBuffer;   // +0x60  (pointer to Result output pointer)
    ULONG64 OutputLength;   // +0x68  (pointer to ResultLength output pointer)
    ULONG64 TrapFrame;      // +0x70  (saved Thread->TrapFrame)
    ULONG64 CallbackStack;  // +0x78  (saved Thread->CallbackStack)
    ULONG64 InitialStack;   // +0x80  (saved Thread->InitialStack)
    ULONG64 Reserved;       // +0x88  (padding for 16-byte SP alignment)
} KCALLOUT_FRAME, *PKCALLOUT_FRAME;
/* Size: 0x90 (144 bytes), 16-byte aligned */

/*
 * ARM64 Context Switch Frame - minimal frame saved by KiSwapContextInternal.
 * Sits at the top of the saved stack (lowest address = Thread->KernelStack).
 */
typedef struct _KSWITCH_FRAME
{
    ULONG64 ApcBypass;  /* WaitIrql / APC bypass flag */
    ULONG64 Return;     /* Return address for KiSwapContextInternal */
} KSWITCH_FRAME, *PKSWITCH_FRAME;
/* Size: 16 bytes (0x10), 16-byte aligned */

/*
 * ARM64 Thread Start Frame - set up by KiInitializeContextThread.
 * KiThreadStartup reads parameters from this frame.
 */
typedef struct _KSTART_FRAME
{
    ULONG64 P1Home;     /* StartRoutine */
    ULONG64 P2Home;     /* StartContext */
    ULONG64 P3Home;     /* SystemRoutine */
    ULONG64 Return;     /* Return address (KiInvalidSystemThreadStartupExit) */
} KSTART_FRAME, *PKSTART_FRAME;

#ifndef NTOS_MODE_USER

typedef struct _TRAPFRAME_LOG_ENTRY
{
    ULONG64 Thread;
    UCHAR CpuNumber;
    UCHAR TrapType;
    USHORT Padding;
    ULONG Cpsrl;
    ULONG64 X0;
    ULONG64 X1;
    ULONG64 X2;
    ULONG64 X3;
    ULONG64 X4;
    ULONG64 X5;
    ULONG64 X6;
    ULONG64 X7;
    ULONG64 Fp;
    ULONG64 Lr;
    ULONG64 Sp;
    ULONG64 Pc;
    ULONG64 Far;
    ULONG Esr;
    ULONG Reserved1;
} TRAPFRAME_LOG_ENTRY, *PTRAPFRAME_LOG_ENTRY;

//
// Special Registers Structure (outside of CONTEXT)
// Based on WoA symbols
//
typedef struct _KSPECIAL_REGISTERS
{
    ULONG64 Elr_El1;
    UINT32  Spsr_El1;
    ULONG64 Tpidr_El0;
    ULONG64 Tpidrro_El0;
    ULONG64 Tpidr_El1;
    ULONG64 KernelBvr[8];
    ULONG   KernelBcr[8];
    ULONG64 KernelWvr[2];
    ULONG   KernelWcr[2];
} KSPECIAL_REGISTERS, *PKSPECIAL_REGISTERS;

//
// ARM64 Architecture State
// Based on WoA symbols
//
typedef struct _KARM64_ARCH_STATE
{
    ULONG64 Midr_El1;
    ULONG64 Sctlr_El1;
    ULONG64 Actlr_El1;
    ULONG64 Cpacr_El1;
    ULONG64 Tcr_El1;
    ULONG64 Ttbr0_El1;
    ULONG64 Ttbr1_El1;
    ULONG64 Esr_El1;
    ULONG64 Far_El1;
    ULONG64 Pmcr_El0;
    ULONG64 Pmcntenset_El0;
    ULONG64 Pmccntr_El0;
    ULONG64 Pmxevcntr_El0[31];
    ULONG64 Pmxevtyper_El0[31];
    ULONG64 Pmovsclr_El0;
    ULONG64 Pmselr_El0;
    ULONG64 Pmuserenr_El0;
    ULONG64 Mair_El1;
    ULONG64 Vbar_El1;
} KARM64_ARCH_STATE, *PKARM64_ARCH_STATE;

typedef struct _KPROCESSOR_STATE
{
    KSPECIAL_REGISTERS SpecialRegisters; // 0
    KARM64_ARCH_STATE ArchState;         // 160
    CONTEXT ContextFrame;                // 800
} KPROCESSOR_STATE, *PKPROCESSOR_STATE;

//
// Processor Region Control Block
// Based on Windows on ARM64
//
typedef struct _KPRCB
{
    UCHAR LegacyNumber;
    UCHAR ReservedMustBeZero;
    UCHAR IdleHalt;
    PKTHREAD CurrentThread;
    PKTHREAD NextThread;
    PKTHREAD IdleThread;
    UCHAR NestingLevel;
    UCHAR ClockOwner;
    union
    {
        UCHAR PendingTickFlags;
        struct
        {
            UCHAR PendingTick : 1;
            UCHAR PendingBackupTick : 1;
        };
    };
    UCHAR PrcbPad00[1];
    ULONG Number;
    ULONG PrcbLock;
    PCHAR PriorityState;
    KPROCESSOR_STATE ProcessorState;
    USHORT ProcessorModel;
    USHORT ProcessorRevision;
    ULONG MHz;
    UINT64 CycleCounterFrequency;
    ULONG HalReserved[15];
    USHORT MinorVersion;
    USHORT MajorVersion;
    UCHAR BuildType;
    UCHAR CpuVendor;
    UCHAR CoresPerPhysicalProcessor;
    UCHAR LogicalProcessorsPerCore;
    PVOID AcpiReserved;
    ULONG GroupSetMember;
    KAFFINITY SetMember;
    UCHAR Group;
    UCHAR GroupIndex;
    UCHAR PrcbPad01[2];
    KSPIN_LOCK_QUEUE DECLSPEC_ALIGN(128) LockQueue[LockQueueMaximumLock];
    UCHAR ProcessorVendorString[2];
    UCHAR PrcbPad02[0x2];
    ULONG FeatureBits;
    ULONG MaxBreakpoints;
    ULONG MaxWatchpoints;
    PCONTEXT Context;
    ULONG ContextFlagsInit;
    UCHAR PrcbPad03[0x60];
    PP_LOOKASIDE_LIST DECLSPEC_ALIGN(128) PPLookasideList[16];
    LONG PacketBarrier;
    SINGLE_LIST_ENTRY DeferredReadyListHead;
    LONG MmPageFaultCount;
    LONG MmCopyOnWriteCount;
    LONG MmTransitionCount;
    LONG MmDemandZeroCount;
    LONG MmPageReadCount;
    LONG MmPageReadIoCount;
    LONG MmDirtyPagesWriteCount;
    LONG MmDirtyWriteIoCount;
    LONG MmMappedPagesWriteCount;
    LONG MmMappedWriteIoCount;
    ULONG KeSystemCalls;
    ULONG KeContextSwitches;
    ULONG CcFastReadNoWait;
    ULONG CcFastReadWait;
    ULONG CcFastReadNotPossible;
    ULONG CcCopyReadNoWait;
    ULONG CcCopyReadWait;
    ULONG CcCopyReadNoWaitMiss;
    LONG LookasideIrpFloat;
    LONG IoReadOperationCount;
    LONG IoWriteOperationCount;
    LONG IoOtherOperationCount;
    LARGE_INTEGER IoReadTransferCount;
    LARGE_INTEGER IoWriteTransferCount;
    LARGE_INTEGER IoOtherTransferCount;
    UCHAR PrcbPad04[0x8];
    PVOID CurrentPacket[3];
    PVOID WorkerRoutine;
    UINT64 TargetSet;
    struct _REQUEST_MAILBOX* Mailbox;
    LONG TargetCount;
    ULONG IpiFrozen;
    ULONG RequestSummary;
    KDPC_DATA DpcData[2];
    PVOID DpcStack;
    PVOID SpBase;
    LONG MaximumDpcQueueDepth;
    ULONG DpcRequestRate;
    ULONG MinimumDpcRate;
    ULONG DpcLastCount;
    UCHAR ThreadDpcEnable;
    UCHAR QuantumEnd;
    UCHAR DpcRoutineActive;
    UCHAR IdleSchedule;
    union
    {
        LONG DpcRequestSummary;
        SHORT DpcRequestSlot[2];
        struct
        {
            SHORT NormalDpcState;
            SHORT ThreadDpcState;
        };
        struct
        {
            ULONG DpcNormalProcessingActive : 1;
            ULONG DpcNormalProcessingRequested : 1;
            ULONG DpcNormalThreadSignal : 1;
            ULONG DpcNormalTimerExpiration : 1;
            ULONG DpcNormalDpcPresent : 1;
            ULONG DpcNormalLocalInterrupt : 1;
            ULONG DpcNormalSpare : 10;
            ULONG DpcThreadActive : 1;
            ULONG DpcThreadRequested : 1;
            ULONG DpcThreadSpare : 14;
        };
    };
    ULONG LastTimerHand;
    ULONG LastTick;
    ULONG ClockInterrupts;
    ULONG ReadyScanTick;
    ULONG PrcbPad10[1];
    ULONG InterruptLastCount;
    ULONG InterruptRate;
    ULONG InterruptCount;
    UCHAR PrcbPad11[0x4];
    KGATE DpcGate;
    KDPC CallDpc;
    LONG ClockKeepAlive;
    UCHAR ClockCheckSlot;
    UCHAR ClockPollCycle;
    UCHAR PrcbPad12[0x2];
    LONG DpcWatchdogPeriod;
    LONG DpcWatchdogCount;
    LONG KeSpinLockOrdering;
    UCHAR PrcbPad13[0x38];
    LIST_ENTRY WaitListHead;
    ULONG WaitLock;
    ULONG ReadySummary;
    ULONG QueueIndex;
    LIST_ENTRY DispatcherReadyListHead[32];
    ULONG InterruptObjectPoolPad[2];
    SINGLE_LIST_ENTRY DeferredWorkList;
    UCHAR PrcbPad14[32];
    KDPC_WATCHDOG_INFORMATION DpcWatchdogDpc;
    KTIMER DpcWatchdogTimer;

    /* Timing/scheduling fields */
    ULONG KernelTime;
    ULONG UserTime;
    ULONG DpcTime;
    ULONG InterruptTime;
    ULONG AdjustDpcThreshold;
    UCHAR SkipTick;
    UCHAR DebuggerSavedIRQL;
    UCHAR PrcbPad20[2];
    ULONG DebugDpcTime;

    /* Timer fields */
    UINT64 TimerHand;
    UINT64 TimerRequest;
    LONG DpcSetEventRequest;
    UCHAR PrcbPad21[4];
    KEVENT DpcEvent;

    /* Pool lookaside lists */
    GENERAL_LOOKASIDE_POOL PPNPagedLookasideList[NUMBER_POOL_LOOKASIDE_LISTS];
    GENERAL_LOOKASIDE_POOL PPPagedLookasideList[NUMBER_POOL_LOOKASIDE_LISTS];

    /* Processor topology */
    struct _KNODE *ParentNode;
    UINT64 MultiThreadProcessorSet;
    struct _KPRCB *MultiThreadSetMaster;

    /* Thread scheduling */
    volatile struct _KPRCB *SignalDone;
    LONG Sleeping;

    /* Memory manager */
    ULONG SecondaryColorMask;

    /* Power management */
    PROCESSOR_POWER_STATE PowerState;

    /* Performance counters */
    ULONG KeAlignmentFixupCount;
    ULONG KeExceptionDispatchCount;
    ULONG KeFirstLevelTbFills;
    ULONG KeFloatingEmulationCount;
    ULONG KeSecondLevelTbFills;

    /* CPU identification */
    CHAR CpuType;
    UCHAR VendorString[13];
    UCHAR DpcInterruptRequested;
    UCHAR PrcbPad22[1];
} KPRCB, *PKPRCB;

//
// Processor Control Region
// Based on WoA
//
typedef struct _KIPCR
{
    union
    {
        struct
        {
            ULONG TibPad0[2];
            PVOID Spare1;
            struct _KPCR *Self;
            PVOID  PcrReserved0;
            struct _KSPIN_LOCK_QUEUE* LockArray;
            PVOID Used_Self;
        };
    };
    KIRQL CurrentIrql;
    UCHAR SecondLevelCacheAssociativity;
    UCHAR Pad1[2];
    USHORT MajorVersion;
    USHORT MinorVersion;
    ULONG StallScaleFactor;
    ULONG SecondLevelCacheSize;
    struct
    {
        UCHAR ApcInterrupt;
        UCHAR DispatchInterrupt;
    };
    USHORT InterruptPad;
    UCHAR BtiMitigation;
    struct
    {
        UCHAR SsbMitigationFirmware:1;
        UCHAR SsbMitigationDynamic:1;
        UCHAR SsbMitigationKernel:1;
        UCHAR SsbMitigationUser:1;
        UCHAR SsbMitigationReserved:4;
    };
    UCHAR Pad2[2];
    ULONG64 PanicStorage[6];
    PVOID KdVersionBlock;
    PVOID HalReserved[134];
    PVOID KvaUserModeTtbr1;

    /* Private members, not in ntddk.h */
    PVOID Idt[256];
    PVOID* IdtExt;
    PVOID PcrAlign[15];
    KPRCB Prcb;
} KIPCR, *PKIPCR;

//
// Macro to get current KPRCB
//
FORCEINLINE
struct _KPRCB *
KeGetCurrentPrcb(VOID)
{
    return KeGetPcr()->CurrentPrcb;
}

//
// Just read it from the PCR
// Note: Don't redefine KeGetCurrentIrql if WDM headers declared it as a function
//
#ifndef _WDMDDK_
#define KeGetCurrentIrql()             KeGetPcr()->CurrentIrql
#endif
#define _KeGetCurrentThread()          KeGetCurrentPrcb()->CurrentThread
#define _KeGetPreviousMode()           KeGetCurrentPrcb()->CurrentThread->PreviousMode
#define _KeIsExecutingDpc()            (KeGetCurrentPrcb()->DpcRoutineActive != 0)
#define KeGetCurrentThread()           _KeGetCurrentThread()
#define KeGetPreviousMode()            _KeGetPreviousMode()

#endif // !NTOS_MODE_USER

#ifdef __cplusplus
}; // extern "C"
#endif

#endif // !_ARM64_KETYPES_H
