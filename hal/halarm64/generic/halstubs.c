/*
 * PROJECT:         ReactOS HAL
 * LICENSE:         GPL-2.0-or-later
 * FILE:            hal/halarm64/generic/halstubs.c
 * PURPOSE:         ARM64 HAL Stub Functions
 * NOTE:            Minimal stubs to allow ntoskrnl to link.
 *                  These will be replaced with real implementations.
 */

/* INCLUDES *******************************************************************/

#include "hal.h"
#define NDEBUG
#include <debug.h>

/* GLOBALS ********************************************************************/

PUCHAR KdComPortInUse;

/* Dispatch tables - pointers filled in by the kernel */
PHAL_DISPATCH HalDispatchTable;
PHAL_PRIVATE_DISPATCH HalPrivateDispatchTable;

static ULONG HalpCurrentTimeIncrement = 100000;
static ULONG HalpNextTimeIncrement = 100000;
static ULONG HalpNextIntervalCount = 0;

/* Software interrupt state — used by IRQL management and HalRequestSoftwareInterrupt */
static KIRQL HalpCurrentIrql = PASSIVE_LEVEL;
static volatile ULONG HalpSoftwareInterruptPending = 0;
static volatile LONG HalpSoftIntDeliveryInProgress = 0;
static VOID HalpDispatchSoftwareInterrupts(KIRQL OldIrql, KIRQL NewIrql);

/* FUNCTIONS ******************************************************************/

/*
 * @implemented
 */
CODE_SEG("INIT")
BOOLEAN
NTAPI
HalInitSystem(
    _In_ ULONG BootPhase,
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    if (BootPhase == 0)
    {
        /* Setup time increments */
        HalpCurrentTimeIncrement = 100000;
        HalpNextTimeIncrement = 100000;
        HalpNextIntervalCount = 0;
        KeSetTimeIncrement(100000, 10000);
    }

    return TRUE;
}

CODE_SEG("INIT")
VOID
NTAPI
HalInitializeProcessor(
    _In_ ULONG ProcessorNumber,
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    /* Nothing to do yet */
}

BOOLEAN
NTAPI
HalAllProcessorsStarted(VOID)
{
    return TRUE;
}

VOID
NTAPI
HalAcquireDisplayOwnership(
    _In_ PHAL_RESET_DISPLAY_PARAMETERS ResetDisplayParameters)
{
    /* Nothing to do */
}

NTSTATUS
NTAPI
HalAdjustResourceList(
    _Inout_ PIO_RESOURCE_REQUIREMENTS_LIST *Requirements)
{
    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
HalAllocateAdapterChannel(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ PWAIT_CONTEXT_BLOCK Wcb,
    _In_ ULONG NumberOfMapRegisters,
    _In_ PDRIVER_CONTROL ExecutionRoutine)
{
    return STATUS_NOT_IMPLEMENTED;
}

PVOID
NTAPI
HalAllocateCommonBuffer(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ ULONG Length,
    _Out_ PPHYSICAL_ADDRESS LogicalAddress,
    _In_ BOOLEAN CacheEnabled)
{
    return NULL;
}

PVOID
NTAPI
HalAllocateCrashDumpRegisters(
    _In_ PADAPTER_OBJECT AdapterObject,
    _Inout_ PULONG NumberOfMapRegisters)
{
    return NULL;
}

NTSTATUS
NTAPI
HalAssignSlotResources(
    _In_ PUNICODE_STRING RegistryPath,
    _In_ PUNICODE_STRING DriverClassName,
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PDEVICE_OBJECT DeviceObject,
    _In_ INTERFACE_TYPE BusType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _Inout_ PCM_RESOURCE_LIST *AllocatedResources)
{
    return STATUS_NOT_IMPLEMENTED;
}

VOID
NTAPI
HalCalibratePerformanceCounter(
    _Inout_ volatile PLONG Count,
    _In_ ULONGLONG NewCount,
    _In_ ULONGLONG Unused)
{
    /* Nothing to do */
}

VOID
FASTCALL
HalClearSoftwareInterrupt(
    _In_ KIRQL Request)
{
    /* Nothing to do yet */
}

VOID
NTAPI
HalDisableSystemInterrupt(
    _In_ ULONG Vector,
    _In_ KIRQL Irql)
{
    /* Nothing to do */
}

VOID
NTAPI
HalDisplayString(
    _In_ PCSTR String)
{
    /* Nothing to do yet */
}

BOOLEAN
NTAPI
HalEnableSystemInterrupt(
    _In_ ULONG Vector,
    _In_ KIRQL Irql,
    _In_ KINTERRUPT_MODE InterruptMode)
{
    return TRUE;
}

BOOLEAN
NTAPI
HalFlushCommonBuffer(
    _In_ ULONG Unknown1,
    _In_ ULONG Unknown2,
    _In_ ULONG Unknown3,
    _In_ ULONG Unknown4,
    _In_ ULONG Unknown5)
{
    return TRUE;
}

VOID
NTAPI
HalFreeCommonBuffer(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ ULONG Length,
    _In_ PHYSICAL_ADDRESS LogicalAddress,
    _In_ PVOID VirtualAddress,
    _In_ BOOLEAN CacheEnabled)
{
    /* Nothing to do */
}

PADAPTER_OBJECT
NTAPI
HalGetAdapter(
    _In_ PDEVICE_DESCRIPTION DeviceDescription,
    _Out_ PULONG NumberOfMapRegisters)
{
    return NULL;
}

ULONG
NTAPI
HalGetBusData(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _Out_ PVOID Buffer,
    _In_ ULONG Length)
{
    return 0;
}

ULONG
NTAPI
HalGetBusDataByOffset(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _Out_ PVOID Buffer,
    _In_ ULONG Offset,
    _In_ ULONG Length)
{
    return 0;
}

ARC_STATUS
NTAPI
HalGetEnvironmentVariable(
    _In_ PCH Name,
    _In_ USHORT ValueLength,
    _Out_ PCH Value)
{
    return ESUCCESS;
}

ULONG
NTAPI
HalGetInterruptVector(
    _In_ INTERFACE_TYPE InterfaceType,
    _In_ ULONG BusNumber,
    _In_ ULONG BusInterruptLevel,
    _In_ ULONG BusInterruptVector,
    _Out_ PKIRQL Irql,
    _Out_ PKAFFINITY Affinity)
{
    return 0;
}

BOOLEAN
NTAPI
HalMakeBeep(
    _In_ ULONG Frequency)
{
    return FALSE;
}

VOID
NTAPI
HalProcessorIdle(VOID)
{
    /* ARM64 WFI instruction */
    __asm__ __volatile__("wfi");
}

VOID
NTAPI
HalQueryDisplayParameters(
    _Out_ PULONG WidthInCharacters,
    _Out_ PULONG HeightInLines,
    _Out_ PULONG CursorColumn,
    _Out_ PULONG CursorRow)
{
    if (WidthInCharacters) *WidthInCharacters = 80;
    if (HeightInLines) *HeightInLines = 25;
    if (CursorColumn) *CursorColumn = 0;
    if (CursorRow) *CursorRow = 0;
}

BOOLEAN
NTAPI
HalQueryRealTimeClock(
    _Out_ PTIME_FIELDS Time)
{
    /* Return a stub time */
    Time->Year = 2025;
    Time->Month = 1;
    Time->Day = 1;
    Time->Hour = 0;
    Time->Minute = 0;
    Time->Second = 0;
    Time->Milliseconds = 0;
    Time->Weekday = 0;
    return TRUE;
}

ULONG
NTAPI
HalReadDmaCounter(
    _In_ PADAPTER_OBJECT AdapterObject)
{
    return 0;
}

VOID
NTAPI
HalReportResourceUsage(VOID)
{
    /* Nothing to do */
}

VOID
NTAPI
HalRequestIpi(
    _In_ KAFFINITY TargetProcessors)
{
    /* Nothing to do yet */
}

VOID
FASTCALL
HalRequestSoftwareInterrupt(
    _In_ KIRQL Request)
{
    /* Record the pending software interrupt */
    HalpSoftwareInterruptPending |= (1 << Request);

    /* If we're already below the requested level, try to dispatch now */
    if (HalpCurrentIrql < APC_LEVEL && Request == APC_LEVEL)
    {
        HalpDispatchSoftwareInterrupts(APC_LEVEL, HalpCurrentIrql);
    }
}

VOID
NTAPI
HalReturnToFirmware(
    _In_ FIRMWARE_REENTRY Action)
{
    /* FIXME: Implement PSCI call */
    while (TRUE);
}

ULONG
NTAPI
HalSetBusData(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _In_ PVOID Buffer,
    _In_ ULONG Length)
{
    return 0;
}

ULONG
NTAPI
HalSetBusDataByOffset(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _In_ PVOID Buffer,
    _In_ ULONG Offset,
    _In_ ULONG Length)
{
    return 0;
}

VOID
NTAPI
HalSetDisplayParameters(
    _In_ ULONG CursorColumn,
    _In_ ULONG CursorRow)
{
    /* Nothing to do */
}

ARC_STATUS
NTAPI
HalSetEnvironmentVariable(
    _In_ PCH Name,
    _In_ PCH Value)
{
    return ESUCCESS;
}

ULONG_PTR
NTAPI
HalSetProfileInterval(
    _In_ ULONG_PTR Interval)
{
    return Interval;
}

BOOLEAN
NTAPI
HalSetRealTimeClock(
    _In_ PTIME_FIELDS Time)
{
    return TRUE;
}

ULONG
NTAPI
HalSetTimeIncrement(
    _In_ ULONG Increment)
{
    HalpNextTimeIncrement = Increment;
    return HalpCurrentTimeIncrement;
}

BOOLEAN
NTAPI
HalStartNextProcessor(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock,
    _In_ PKPROCESSOR_STATE ProcessorState)
{
    return FALSE;
}

VOID
NTAPI
HalStartProfileInterrupt(
    _In_ KPROFILE_SOURCE ProfileSource)
{
    /* Nothing to do */
}

VOID
NTAPI
HalStopProfileInterrupt(
    _In_ KPROFILE_SOURCE ProfileSource)
{
    /* Nothing to do */
}

UCHAR
FASTCALL
HalSystemVectorDispatchEntry(
    _In_ ULONG Vector,
    _Out_ PKINTERRUPT_ROUTINE **FlatDispatch,
    _Out_ PKINTERRUPT_ROUTINE *NoConnection)
{
    return 0;
}

BOOLEAN
NTAPI
HalTranslateBusAddress(
    _In_ INTERFACE_TYPE InterfaceType,
    _In_ ULONG BusNumber,
    _In_ PHYSICAL_ADDRESS BusAddress,
    _In_ PULONG AddressSpace,
    _Out_ PPHYSICAL_ADDRESS TranslatedAddress)
{
    /* Identity-map for now */
    TranslatedAddress->QuadPart = BusAddress.QuadPart;
    return TRUE;
}

BOOLEAN
NTAPI
IoFlushAdapterBuffers(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ PMDL Mdl,
    _In_ PVOID MapRegisterBase,
    _In_ PVOID CurrentVa,
    _In_ ULONG Length,
    _In_ BOOLEAN WriteToDevice)
{
    return TRUE;
}

VOID
NTAPI
IoFreeAdapterChannel(
    _In_ PADAPTER_OBJECT AdapterObject)
{
    /* Nothing to do */
}

VOID
NTAPI
IoFreeMapRegisters(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ PVOID MapRegisterBase,
    _In_ ULONG NumberOfMapRegisters)
{
    /* Nothing to do */
}

PHYSICAL_ADDRESS
NTAPI
IoMapTransfer(
    _In_ PADAPTER_OBJECT AdapterObject,
    _In_ PMDL Mdl,
    _In_ PVOID MapRegisterBase,
    _In_ PVOID CurrentVa,
    _Inout_ PULONG Length,
    _In_ BOOLEAN WriteToDevice)
{
    PHYSICAL_ADDRESS Address;
    Address.QuadPart = 0;
    return Address;
}

VOID
NTAPI
KeFlushWriteBuffer(VOID)
{
    /* ARM64 DSB SY instruction */
    __asm__ __volatile__("dsb sy" ::: "memory");
}

LARGE_INTEGER
NTAPI
KeQueryPerformanceCounter(
    _Out_opt_ PLARGE_INTEGER PerformanceFrequency)
{
    LARGE_INTEGER Result;
    ULONG64 Count, Freq;

    /* Read ARM64 virtual counter (CNTVCT_EL0) */
    __asm__ __volatile__("mrs %0, cntvct_el0" : "=r"(Count));
    Result.QuadPart = Count;

    if (PerformanceFrequency)
    {
        /* Read counter frequency (CNTFRQ_EL0) */
        __asm__ __volatile__("mrs %0, cntfrq_el0" : "=r"(Freq));
        PerformanceFrequency->QuadPart = Freq;
    }

    return Result;
}

VOID
NTAPI
KeStallExecutionProcessor(
    _In_ ULONG MicroSeconds)
{
    ULONG64 Freq, Start, Target;

    /* Read counter frequency and current count */
    __asm__ __volatile__("mrs %0, cntfrq_el0" : "=r"(Freq));
    __asm__ __volatile__("mrs %0, cntvct_el0" : "=r"(Start));
    Target = Start + (Freq * MicroSeconds / 1000000);

    while (1)
    {
        ULONG64 Current;
        __asm__ __volatile__("mrs %0, cntvct_el0" : "=r"(Current));
        if (Current >= Target) break;
        __asm__ __volatile__("yield");
    }
}

/* Undef macros so we can define the actual exported functions */
#undef KeLowerIrql
#undef KeRaiseIrql
#undef KeAcquireSpinLock
#undef KeReleaseSpinLock

/* IRQL Management ************************************************************/

KIRQL
NTAPI
KeGetCurrentIrql(VOID)
{
    return HalpCurrentIrql;
}

/*
 * HalpDispatchSoftwareInterrupts
 *
 * On ARM64, there is no hardware software-interrupt mechanism.
 * We emulate it by checking for pending software interrupts
 * whenever IRQL is lowered. This is critical for APC delivery.
 */
static
VOID
HalpDispatchSoftwareInterrupts(
    _In_ KIRQL OldIrql,
    _In_ KIRQL NewIrql)
{
    /* Prevent reentrancy */
    if (_InterlockedCompareExchange(&HalpSoftIntDeliveryInProgress, 1, 0) != 0)
        return;

    /* Check for pending APC software interrupt */
    if (NewIrql < APC_LEVEL && (HalpSoftwareInterruptPending & (1 << APC_LEVEL)))
    {
        PKTHREAD Thread = KeGetCurrentThread();
        if (Thread != NULL &&
            Thread->ApcState.KernelApcPending &&
            !Thread->SpecialApcDisable)
        {
            /* Clear pending APC request */
            HalpSoftwareInterruptPending &= ~(1 << APC_LEVEL);

            /* Deliver kernel APCs at APC_LEVEL */
            HalpCurrentIrql = APC_LEVEL;
            KiDeliverApc(KernelMode, NULL, NULL);
            HalpCurrentIrql = NewIrql;
        }
    }

    HalpSoftIntDeliveryInProgress = 0;
}

VOID
FASTCALL
KfLowerIrql(
    _In_ KIRQL NewIrql)
{
    KIRQL OldIrql = HalpCurrentIrql;
    HalpCurrentIrql = NewIrql;

    /* Check for pending software interrupts when lowering IRQL */
    if (OldIrql > NewIrql && HalpSoftwareInterruptPending)
    {
        HalpDispatchSoftwareInterrupts(OldIrql, NewIrql);
    }
}

KIRQL
FASTCALL
KfRaiseIrql(
    _In_ KIRQL NewIrql)
{
    KIRQL OldIrql = HalpCurrentIrql;
    HalpCurrentIrql = NewIrql;
    return OldIrql;
}

VOID
NTAPI
KeLowerIrql(
    _In_ KIRQL NewIrql)
{
    KfLowerIrql(NewIrql);
}

VOID
NTAPI
KeRaiseIrql(
    _In_ KIRQL NewIrql,
    _Out_ PKIRQL OldIrql)
{
    *OldIrql = KfRaiseIrql(NewIrql);
}

KIRQL
NTAPI
KeRaiseIrqlToDpcLevel(VOID)
{
    return KfRaiseIrql(DISPATCH_LEVEL);
}

KIRQL
NTAPI
KeRaiseIrqlToSynchLevel(VOID)
{
    return KfRaiseIrql(SYNCH_LEVEL);
}

BOOLEAN
NTAPI
HalBeginSystemInterrupt(
    _In_ KIRQL Irql,
    _In_ ULONG Vector,
    _Out_ PKIRQL OldIrql)
{
    *OldIrql = KfRaiseIrql(Irql);
    return TRUE;
}

VOID
NTAPI
HalEndSystemInterrupt(
    _In_ KIRQL Irql,
    _In_ PKTRAP_FRAME TrapFrame)
{
    KfLowerIrql(Irql);
}

/* Spinlock Stubs *************************************************************/

/*
 * ARM64 uniprocessor spinlock stubs.
 * On UP, spinlocks are implemented by raising IRQL only.
 * However, in DBG builds, the kernel's KxReleaseSpinLock checks that the lock
 * value matches (KeGetCurrentThread() | 1), and KeReleaseInStackQueuedSpinLock-
 * FromDpcLevel reads LockHandle->LockQueue.Lock.  We must set these values
 * during acquire to avoid DBG bugchecks and null pointer dereferences.
 */

FORCEINLINE
VOID
HalpAcquireSpinLockDbg(
    _Inout_ PKSPIN_LOCK SpinLock)
{
#if DBG
    *SpinLock = (KSPIN_LOCK)KeGetCurrentThread() | 1;
#endif
}

FORCEINLINE
VOID
HalpReleaseSpinLockDbg(
    _Inout_ PKSPIN_LOCK SpinLock)
{
#if DBG
    *SpinLock = 0;
#endif
}

KIRQL
FASTCALL
KfAcquireSpinLock(
    _Inout_ PKSPIN_LOCK SpinLock)
{
    KIRQL OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
    HalpAcquireSpinLockDbg(SpinLock);
    return OldIrql;
}

VOID
FASTCALL
KfReleaseSpinLock(
    _Inout_ PKSPIN_LOCK SpinLock,
    _In_ KIRQL NewIrql)
{
    HalpReleaseSpinLockDbg(SpinLock);
    KfLowerIrql(NewIrql);
}

VOID
NTAPI
KeAcquireSpinLock(
    _Inout_ PKSPIN_LOCK SpinLock,
    _Out_ PKIRQL OldIrql)
{
    *OldIrql = KfAcquireSpinLock(SpinLock);
}

VOID
NTAPI
KeReleaseSpinLock(
    _Inout_ PKSPIN_LOCK SpinLock,
    _In_ KIRQL NewIrql)
{
    KfReleaseSpinLock(SpinLock, NewIrql);
}

KIRQL
FASTCALL
KeAcquireSpinLockRaiseToSynch(
    _Inout_ PKSPIN_LOCK SpinLock)
{
    KIRQL OldIrql = KfRaiseIrql(SYNCH_LEVEL);
    HalpAcquireSpinLockDbg(SpinLock);
    return OldIrql;
}

VOID
FASTCALL
KeAcquireInStackQueuedSpinLock(
    _Inout_ PKSPIN_LOCK SpinLock,
    _Out_ PKLOCK_QUEUE_HANDLE LockHandle)
{
    LockHandle->OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
    LockHandle->LockQueue.Lock = SpinLock;
    HalpAcquireSpinLockDbg(SpinLock);
}

VOID
FASTCALL
KeAcquireInStackQueuedSpinLockRaiseToSynch(
    _Inout_ PKSPIN_LOCK SpinLock,
    _Out_ PKLOCK_QUEUE_HANDLE LockHandle)
{
    LockHandle->OldIrql = KfRaiseIrql(SYNCH_LEVEL);
    LockHandle->LockQueue.Lock = SpinLock;
    HalpAcquireSpinLockDbg(SpinLock);
}

VOID
FASTCALL
KeReleaseInStackQueuedSpinLock(
    _In_ PKLOCK_QUEUE_HANDLE LockHandle)
{
    HalpReleaseSpinLockDbg(LockHandle->LockQueue.Lock);
    KfLowerIrql(LockHandle->OldIrql);
}

KIRQL
FASTCALL
KeAcquireQueuedSpinLock(
    _In_ KSPIN_LOCK_QUEUE_NUMBER Number)
{
    return KfRaiseIrql(DISPATCH_LEVEL);
}

KIRQL
FASTCALL
KeAcquireQueuedSpinLockRaiseToSynch(
    _In_ KSPIN_LOCK_QUEUE_NUMBER LockNumber)
{
    return KfRaiseIrql(SYNCH_LEVEL);
}

VOID
FASTCALL
KeReleaseQueuedSpinLock(
    _In_ KSPIN_LOCK_QUEUE_NUMBER Number,
    _In_ KIRQL OldIrql)
{
    KfLowerIrql(OldIrql);
}

LOGICAL
FASTCALL
KeTryToAcquireQueuedSpinLock(
    _In_ KSPIN_LOCK_QUEUE_NUMBER Number,
    _Out_ PKIRQL OldIrql)
{
    *OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
    return TRUE;
}

BOOLEAN
FASTCALL
KeTryToAcquireQueuedSpinLockRaiseToSynch(
    _In_ KSPIN_LOCK_QUEUE_NUMBER LockNumber,
    _In_ PKIRQL OldIrql)
{
    *OldIrql = KfRaiseIrql(SYNCH_LEVEL);
    return TRUE;
}

/* FastMutex Stubs ************************************************************/

VOID
FASTCALL
ExAcquireFastMutex(
    _Inout_ PFAST_MUTEX FastMutex)
{
    FastMutex->OldIrql = KfRaiseIrql(APC_LEVEL);
    InterlockedDecrement(&FastMutex->Count);
}

VOID
FASTCALL
ExReleaseFastMutex(
    _Inout_ PFAST_MUTEX FastMutex)
{
    KIRQL OldIrql = (KIRQL)FastMutex->OldIrql;
    InterlockedIncrement(&FastMutex->Count);
    KfLowerIrql(OldIrql);
}

BOOLEAN
FASTCALL
ExTryToAcquireFastMutex(
    _Inout_ PFAST_MUTEX FastMutex)
{
    FastMutex->OldIrql = KfRaiseIrql(APC_LEVEL);
    if (InterlockedCompareExchange(&FastMutex->Count, 0, 1) == 1)
    {
        return TRUE;
    }
    KfLowerIrql((KIRQL)FastMutex->OldIrql);
    return FALSE;
}

/* Port I/O Stubs (memory-mapped on ARM64) ************************************/

UCHAR
NTAPI
READ_PORT_UCHAR(
    _In_ PUCHAR Port)
{
    return *(volatile UCHAR *)Port;
}

USHORT
NTAPI
READ_PORT_USHORT(
    _In_ PUSHORT Port)
{
    return *(volatile USHORT *)Port;
}

ULONG
NTAPI
READ_PORT_ULONG(
    _In_ PULONG Port)
{
    return *(volatile ULONG *)Port;
}

VOID
NTAPI
READ_PORT_BUFFER_UCHAR(
    _In_ PUCHAR Port,
    _Out_ PUCHAR Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *Buffer++ = *(volatile UCHAR *)Port;
}

VOID
NTAPI
READ_PORT_BUFFER_USHORT(
    _In_ PUSHORT Port,
    _Out_ PUSHORT Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *Buffer++ = *(volatile USHORT *)Port;
}

VOID
NTAPI
READ_PORT_BUFFER_ULONG(
    _In_ PULONG Port,
    _Out_ PULONG Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *Buffer++ = *(volatile ULONG *)Port;
}

VOID
NTAPI
WRITE_PORT_UCHAR(
    _In_ PUCHAR Port,
    _In_ UCHAR Value)
{
    *(volatile UCHAR *)Port = Value;
}

VOID
NTAPI
WRITE_PORT_USHORT(
    _In_ PUSHORT Port,
    _In_ USHORT Value)
{
    *(volatile USHORT *)Port = Value;
}

VOID
NTAPI
WRITE_PORT_ULONG(
    _In_ PULONG Port,
    _In_ ULONG Value)
{
    *(volatile ULONG *)Port = Value;
}

VOID
NTAPI
WRITE_PORT_BUFFER_UCHAR(
    _In_ PUCHAR Port,
    _In_ PUCHAR Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *(volatile UCHAR *)Port = *Buffer++;
}

VOID
NTAPI
WRITE_PORT_BUFFER_USHORT(
    _In_ PUSHORT Port,
    _In_ PUSHORT Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *(volatile USHORT *)Port = *Buffer++;
}

VOID
NTAPI
WRITE_PORT_BUFFER_ULONG(
    _In_ PULONG Port,
    _In_ PULONG Buffer,
    _In_ ULONG Count)
{
    while (Count--)
        *(volatile ULONG *)Port = *Buffer++;
}

/* EOF */
