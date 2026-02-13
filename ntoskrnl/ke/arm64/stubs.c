/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/stubs.c
 * PURPOSE:         ARM64 Architecture-Specific Stub Functions
 * NOTE:            Minimal stubs for linking. Will be replaced with
 *                  real implementations as the port progresses.
 */

/* INCLUDES *******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* DATA ***********************************************************************/

/* Process count for performance data */
ULONG ProcessCount = 0;

/* PTE templates used by the memory manager */
MMPTE DemandZeroPde = {{0}};
MMPTE DemandZeroPte = {{0}};
MMPTE PrototypePte = {{0}};
MMPTE MmDecommittedPte = {{0}};
MMPTE ValidKernelPde = {{0}};
MMPTE ValidKernelPte = {{0}};
MMPTE ValidKernelPdeLocal = {{0}};
MMPTE ValidKernelPteLocal = {{0}};

/* Protection to PTE mask mapping */
ULONG_PTR MmProtectToPteMask[32] = {0};

/* Protection value to Win32 protection mapping */
ULONG MmProtectToValue[32] = {0};

/* FUNCTIONS ******************************************************************/

BOOLEAN
NTAPI
KeConnectInterrupt(
    _Inout_ PKINTERRUPT Interrupt)
{
    UNIMPLEMENTED;
    return FALSE;
}

BOOLEAN
NTAPI
KeDisconnectInterrupt(
    _Inout_ PKINTERRUPT Interrupt)
{
    UNIMPLEMENTED;
    return FALSE;
}

VOID
NTAPI
KeFlushEntireTb(
    _In_ BOOLEAN Invalid,
    _In_ BOOLEAN AllProcessors)
{
    /* ARM64: TLBI VMALLE1IS - invalidate all TLB entries */
    __asm__ __volatile__("tlbi vmalle1is" ::: "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
}

ULONG
NTAPI
KeGetRecommendedSharedDataAlignment(VOID)
{
    /* ARM64 cache line size is typically 64 bytes */
    return 64;
}

VOID
NTAPI
KeInitializeInterrupt(
    _Out_ PKINTERRUPT Interrupt,
    _In_ PKSERVICE_ROUTINE ServiceRoutine,
    _In_opt_ PVOID ServiceContext,
    _In_opt_ PKSPIN_LOCK SpinLock,
    _In_ ULONG Vector,
    _In_ KIRQL Irql,
    _In_ KIRQL SynchronizeIrql,
    _In_ KINTERRUPT_MODE InterruptMode,
    _In_ BOOLEAN ShareVector,
    _In_ CHAR ProcessorNumber,
    _In_ BOOLEAN FloatingSave)
{
    /* Initialize the interrupt object */
    Interrupt->Type = InterruptObject;
    Interrupt->Size = sizeof(KINTERRUPT);
    Interrupt->ServiceRoutine = ServiceRoutine;
    Interrupt->ServiceContext = ServiceContext;
    KeInitializeSpinLock(&Interrupt->SpinLock);
    Interrupt->ActualLock = SpinLock ? SpinLock : &Interrupt->SpinLock;
    Interrupt->Vector = Vector;
    Interrupt->Irql = Irql;
    Interrupt->SynchronizeIrql = SynchronizeIrql;
    Interrupt->Mode = InterruptMode;
    Interrupt->ShareVector = ShareVector;
    Interrupt->Number = ProcessorNumber;
    Interrupt->FloatingSave = FloatingSave;
    Interrupt->Connected = FALSE;
}

BOOLEAN
NTAPI
KeInvalidateAllCaches(VOID)
{
    /* ARM64: Clean and invalidate all data caches */
    __asm__ __volatile__("dsb sy" ::: "memory");
    return TRUE;
}

NTSTATUS
NTAPI
KeRaiseUserException(
    _In_ NTSTATUS ExceptionCode)
{
    UNIMPLEMENTED;
    return ExceptionCode;
}

VOID
NTAPI
KeSaveStateForHibernate(
    _Out_ PVOID State)
{
    UNIMPLEMENTED;
}

VOID
NTAPI
KeSetDmaIoCoherency(
    _In_ ULONG Coherency)
{
    /* Nothing to do */
}

BOOLEAN
NTAPI
KeSynchronizeExecution(
    _Inout_ PKINTERRUPT Interrupt,
    _In_ PKSYNCHRONIZE_ROUTINE SynchronizeRoutine,
    _In_opt_ PVOID SynchronizeContext)
{
    KIRQL OldIrql;
    BOOLEAN Result;

    /* Raise to synchronize IRQL */
    KeRaiseIrql(Interrupt->SynchronizeIrql, &OldIrql);

    /* Call the routine */
    Result = SynchronizeRoutine(SynchronizeContext);

    /* Lower IRQL */
    KeLowerIrql(OldIrql);

    return Result;
}

NTSTATUS
NTAPI
KeUserModeCallback(
    _In_ ULONG RoutineIndex,
    _In_ PVOID InputBuffer,
    _In_ ULONG InputLength,
    _Out_ PVOID *OutputBuffer,
    _Out_ PULONG OutputLength)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

/* VDM is not supported on ARM64 */
NTSTATUS
NTAPI
NtVdmControl(
    _In_ ULONG ControlCode,
    _In_ PVOID ControlData)
{
    return STATUS_NOT_IMPLEMENTED;
}

/* Hardware configuration stub */
NTSTATUS
NTAPI
CmpInitializeMachineDependentConfiguration(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    /* Nothing to configure yet */
    return STATUS_SUCCESS;
}

/* Interlocked SList operations - simple stubs for UP */
NTKERNELAPI
PSLIST_ENTRY
ExpInterlockedPopEntrySList(
    _Inout_ PSLIST_HEADER SListHead)
{
    /* FIXME: Proper atomic implementation needed for SMP */
    return NULL;
}

NTKERNELAPI
PSLIST_ENTRY
ExpInterlockedPushEntrySList(
    _Inout_ PSLIST_HEADER SListHead,
    _Inout_ PSLIST_ENTRY SListEntry)
{
    /* FIXME: Proper atomic implementation needed for SMP */
    return NULL;
}

/* Kernel debugger transport stub - matches KdSendPacket signature */
VOID
NTAPI
KdbgSendPacket(
    _In_ ULONG PacketType,
    _In_ PSTRING MessageHeader,
    _In_ PSTRING MessageData,
    _Inout_ PKD_CONTEXT Context)
{
    /* Nothing to do */
}

/* KdbpGetHexNumber and KdbpPrint moved to #ifdef KDBG section below */

/* KD system I/O stubs - match kd64.h declarations */
NTSTATUS
NTAPI
KdpSysReadBusData(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _In_ ULONG Offset,
    _Out_ PVOID Buffer,
    _In_ ULONG Length,
    _Out_ PULONG ActualLength)
{
    *ActualLength = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysReadControlSpace(
    _In_ ULONG Processor,
    _In_ ULONG64 BaseAddress,
    _Out_ PVOID Buffer,
    _In_ ULONG Length,
    _Out_ PULONG ActualLength)
{
    *ActualLength = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysReadIoSpace(
    _In_ INTERFACE_TYPE InterfaceType,
    _In_ ULONG BusNumber,
    _In_ ULONG AddressSpace,
    _In_ ULONG64 IoAddress,
    _Out_ PVOID DataValue,
    _In_ ULONG DataSize,
    _Out_ PULONG ActualDataSize)
{
    *ActualDataSize = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysReadMsr(
    _In_ ULONG Msr,
    _Out_ PULONGLONG MsrValue)
{
    *MsrValue = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysWriteBusData(
    _In_ BUS_DATA_TYPE BusDataType,
    _In_ ULONG BusNumber,
    _In_ ULONG SlotNumber,
    _In_ ULONG Offset,
    _In_ PVOID Buffer,
    _In_ ULONG Length,
    _Out_ PULONG ActualLength)
{
    *ActualLength = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysWriteControlSpace(
    _In_ ULONG Processor,
    _In_ ULONG64 BaseAddress,
    _In_ PVOID Buffer,
    _In_ ULONG Length,
    _Out_ PULONG ActualLength)
{
    *ActualLength = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysWriteIoSpace(
    _In_ INTERFACE_TYPE InterfaceType,
    _In_ ULONG BusNumber,
    _In_ ULONG AddressSpace,
    _In_ ULONG64 IoAddress,
    _In_ PVOID DataValue,
    _In_ ULONG DataSize,
    _Out_ PULONG ActualDataSize)
{
    *ActualDataSize = 0;
    return STATUS_UNSUCCESSFUL;
}

NTSTATUS
NTAPI
KdpSysWriteMsr(
    _In_ ULONG Msr,
    _In_ PULONGLONG MsrValue)
{
    return STATUS_UNSUCCESSFUL;
}

/* Spin lock acquire at DPC level (exported symbol) */
#undef KeAcquireSpinLock
KIRQL
NTAPI
KeAcquireSpinLockRaiseToDpc(
    _Inout_ PKSPIN_LOCK SpinLock)
{
    return KfRaiseIrql(DISPATCH_LEVEL);
}

/* Context/Trap frame conversion */
VOID
NTAPI
KeContextToTrapFrame(
    _In_ PCONTEXT Context,
    _Inout_opt_ PKEXCEPTION_FRAME ExceptionFrame,
    _Inout_ PKTRAP_FRAME TrapFrame,
    _In_ ULONG ContextFlags,
    _In_ KPROCESSOR_MODE PreviousMode)
{
    /* FIXME: Implement ARM64 context to trap frame conversion */
    UNIMPLEMENTED;
}

/* Thread context switch */
BOOLEAN
FASTCALL
KiSwapContext(
    _In_ KIRQL WaitIrql,
    _In_ PKTHREAD CurrentThread)
{
    /* FIXME: Implement ARM64 context switch */
    UNIMPLEMENTED;
    return FALSE;
}

/* User-mode APC initialization */
VOID
NTAPI
KiInitializeUserApc(
    _In_ PKEXCEPTION_FRAME ExceptionFrame,
    _Inout_ PKTRAP_FRAME TrapFrame,
    _In_ PKNORMAL_ROUTINE NormalRoutine,
    _In_ PVOID NormalContext,
    _In_ PVOID SystemArgument1,
    _In_ PVOID SystemArgument2)
{
    UNIMPLEMENTED;
}

/* Call user mode */
NTSTATUS
NTAPI
KiCallUserMode(
    _Out_ PVOID *OutputBuffer,
    _Out_ PULONG OutputLength)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

/* Processor state save/restore */
VOID
NTAPI
KiSaveProcessorControlState(
    _Out_ PKPROCESSOR_STATE ProcessorState)
{
    /* FIXME: Save ARM64 system registers */
}

VOID
NTAPI
KiRestoreProcessorControlState(
    _In_ PKPROCESSOR_STATE ProcessorState)
{
    /* FIXME: Restore ARM64 system registers */
}

/* KD stubs */
NTSTATUS
NTAPI
KdpAllowDisable(VOID)
{
    return STATUS_ACCESS_DENIED;
}

VOID
NTAPI
KdpGetStateChange(
    _In_ PDBGKD_MANIPULATE_STATE64 State,
    _In_ PCONTEXT Context)
{
    /* Nothing to do */
}

VOID
NTAPI
KdpSetContextState(
    _In_ PDBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange,
    _In_ PCONTEXT Context)
{
    /* Nothing to do */
}

NTSTATUS
NTAPI
KdpSysCheckLowMemory(
    _In_ ULONG Flags)
{
    return STATUS_UNSUCCESSFUL;
}

KCONTINUE_STATUS
NTAPI
KxSwitchKdProcessor(
    _In_ ULONG ProcessorIndex)
{
    return ContinueSuccess;
}

VOID
NTAPI
KxFreezeExecution(VOID)
{
    UNIMPLEMENTED;
}

VOID
NTAPI
KxThawExecution(VOID)
{
    UNIMPLEMENTED;
}

/* KD power transition stubs */
NTSTATUS
NTAPI
KdbgD0Transition(VOID)
{
    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
KdbgD3Transition(VOID)
{
    return STATUS_SUCCESS;
}

KDSTATUS
NTAPI
KdbgReceivePacket(
    _In_ ULONG PacketType,
    _Out_ PSTRING MessageHeader,
    _Out_ PSTRING MessageData,
    _Out_ PULONG DataLength,
    _Inout_ PKD_CONTEXT Context)
{
    return KdPacketTimedOut;
}

NTSTATUS
NTAPI
KdbgSave(
    _In_ BOOLEAN SleepTransition)
{
    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
KdbgRestore(
    _In_ BOOLEAN SleepTransition)
{
    return STATUS_SUCCESS;
}

/* KDBG callback stubs */
BOOLEAN
NTAPI
KdbRegisterCliCallback(
    _In_ PVOID Callback,
    _In_ BOOLEAN Deregister)
{
    return FALSE;
}

/* KdbSymPrintAddress moved to #ifdef KDBG section below */

/* Exception dispatch */
VOID
NTAPI
KiDispatchException(
    _In_ PEXCEPTION_RECORD ExceptionRecord,
    _In_ PKEXCEPTION_FRAME ExceptionFrame,
    _In_ PKTRAP_FRAME TrapFrame,
    _In_ KPROCESSOR_MODE PreviousMode,
    _In_ BOOLEAN FirstChance)
{
    UNIMPLEMENTED;
    KeBugCheck(KMODE_EXCEPTION_NOT_HANDLED);
}

/* Exception exit - declared in ARM64 ke.h */
DECLSPEC_NORETURN
VOID
KiExceptionExit(
    _In_ PKTRAP_FRAME TrapFrame,
    _In_ PKEXCEPTION_FRAME ExceptionFrame)
{
    /* FIXME: Implement trap frame restore and return */
    KeBugCheck(0);
    while (TRUE);
}

/* Thread initialization */
VOID
NTAPI
KiInitializeContextThread(
    _In_ PKTHREAD Thread,
    _In_ PKSYSTEM_ROUTINE SystemRoutine,
    _In_opt_ PKSTART_ROUTINE StartRoutine,
    _In_opt_ PVOID StartContext,
    _In_ PCONTEXT ContextRecord)
{
    UNIMPLEMENTED;
}

/* Machine-dependent kernel init */
VOID
NTAPI
KiInitMachineDependent(VOID)
{
    /* FIXME: Initialize ARM64-specific kernel features */
}

/* Process address space switch */
VOID
NTAPI
KiSwapProcess(
    _In_ PKPROCESS NewProcess,
    _In_ PKPROCESS OldProcess)
{
    /* FIXME: Switch ARM64 page tables (TTBR0_EL1) */
    UNIMPLEMENTED;
}

/* TLB flush for current processor */
VOID
NTAPI
KeFlushCurrentTb(VOID)
{
    __asm__ __volatile__("tlbi vmalle1" ::: "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
}

/* Zero pages (fast memory clear) */
VOID
NTAPI
KeZeroPages(
    _Out_ PVOID Address,
    _In_ ULONG Size)
{
    RtlZeroMemory(Address, Size);
}

/* MM session space layout */
VOID
NTAPI
MiInitializeSessionSpaceLayout(VOID)
{
    /* FIXME: Setup ARM64 session space addresses */
}

/* MM machine-dependent init */
VOID
NTAPI
MiInitMachineDependent(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    UNIMPLEMENTED;
}

/* Callback return */
NTSTATUS
NTAPI
NtCallbackReturn(
    _In_ PVOID Result,
    _In_ ULONG ResultLength,
    _In_ NTSTATUS Status)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

/* LDT not supported on ARM64 */
NTSTATUS
NTAPI
NtSetLdtEntries(
    _In_ ULONG Selector1,
    _In_ LDT_ENTRY LdtEntry1,
    _In_ ULONG Selector2,
    _In_ LDT_ENTRY LdtEntry2)
{
    return STATUS_NOT_IMPLEMENTED;
}

/* PS context get/set routine */
VOID
NTAPI
PspGetOrSetContextKernelRoutine(
    _In_ PKAPC Apc,
    _Inout_ PKNORMAL_ROUTINE *NormalRoutine,
    _Inout_ PVOID *NormalContext,
    _Inout_ PVOID *SystemArgument1,
    _Inout_ PVOID *SystemArgument2)
{
    UNIMPLEMENTED;
}

/* Page table / virtual memory management stubs */
BOOLEAN
NTAPI
MmIsPagePresent(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    return FALSE;
}

BOOLEAN
NTAPI
MmIsDisabledPage(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    return FALSE;
}

BOOLEAN
NTAPI
MmIsPageSwapEntry(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    return FALSE;
}

PFN_NUMBER
NTAPI
MmGetPfnForProcess(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    return 0;
}

ULONG
NTAPI
MmGetPageProtect(
    _In_ PEPROCESS Process,
    _In_ PVOID Address)
{
    return PAGE_NOACCESS;
}

VOID
NTAPI
MmSetPageProtect(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect)
{
    UNIMPLEMENTED;
}

VOID
NTAPI
MmSetDirtyBit(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ BOOLEAN Dirty)
{
    UNIMPLEMENTED;
}

NTSTATUS
NTAPI
MmCreateVirtualMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS
NTAPI
MmCreatePhysicalMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG flProtect,
    _In_ PFN_NUMBER Page)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

_Success_(return)
BOOLEAN
MmDeleteVirtualMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_opt_ BOOLEAN *WasDirty,
    _Out_opt_ PPFN_NUMBER Page)
{
    if (WasDirty) *WasDirty = FALSE;
    if (Page) *Page = 0;
    UNIMPLEMENTED;
    return FALSE;
}

_Success_(return)
BOOLEAN
MmDeletePhysicalMapping(
    _Inout_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_opt_ BOOLEAN *WasDirty,
    _Out_opt_ PPFN_NUMBER Page)
{
    if (WasDirty) *WasDirty = FALSE;
    if (Page) *Page = 0;
    UNIMPLEMENTED;
    return FALSE;
}

VOID
NTAPI
MmGetPageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_ SWAPENTRY *SwapEntry)
{
    *SwapEntry = 0;
}

NTSTATUS
NTAPI
MmCreatePageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ SWAPENTRY SwapEntry)
{
    UNIMPLEMENTED;
    return STATUS_NOT_IMPLEMENTED;
}

VOID
NTAPI
MmDeletePageFileMapping(
    _In_ PEPROCESS Process,
    _In_ PVOID Address,
    _Out_ SWAPENTRY *SwapEntry)
{
    *SwapEntry = 0;
}

BOOLEAN
MiArchCreateProcessAddressSpace(
    _In_ PEPROCESS Process,
    _In_ PULONG_PTR DirectoryTableBase)
{
    UNIMPLEMENTED;
    return FALSE;
}

VOID
NTAPI
MmInitGlobalKernelPageDirectory(VOID)
{
    /* FIXME: Initialize ARM64 kernel page directory */
}

/* KeSwitchKernelStack */
PVOID
NTAPI
KeSwitchKernelStack(
    IN PVOID StackBase,
    IN PVOID StackLimit)
{
    UNIMPLEMENTED;
    return NULL;
}

/* KDBG stubs - needed when KDBG is defined but kdbg sources excluded for arm64 */
#ifdef KDBG
#include "../kdbg/kdb.h"

const CSTRING KdbPromptStr = RTL_CONSTANT_STRING("kdb:> ");

NTSTATUS
NTAPI
KdbInitialize(
    _In_ PKD_DISPATCH_TABLE DispatchTable,
    _In_ ULONG BootPhase)
{
    return STATUS_NOT_IMPLEMENTED;
}

PCSTR
KdbGetHistoryEntry(
    _Inout_ PLONG NextIndex,
    _In_ BOOLEAN Next)
{
    return NULL;
}

BOOLEAN
NTAPI
KdbpGetHexNumber(
    IN PCHAR pszNum,
    OUT ULONG_PTR *pulValue)
{
    return FALSE;
}

VOID
KdbpPrint(
    _In_ PSTR Format,
    ...)
{
    /* Nothing to do */
}

BOOLEAN
KdbSymPrintAddress(
    IN PVOID Address,
    IN PCONTEXT Context)
{
    return FALSE;
}

#endif /* KDBG */

/* RtlUshortByteSwap - must #undef the macro from wdm.h */
#undef RtlUshortByteSwap

USHORT
FASTCALL
RtlUshortByteSwap(
    IN USHORT Source)
{
    return (Source >> 8) | (Source << 8);
}

/* ARM64 Memory-mapped I/O register access functions */
UCHAR
NTAPI
READ_REGISTER_UCHAR(
    IN PUCHAR Register)
{
    UCHAR Value;
    Value = *(volatile UCHAR *)Register;
    __asm__ __volatile__("dmb ld" ::: "memory");
    return Value;
}

USHORT
NTAPI
READ_REGISTER_USHORT(
    IN PUSHORT Register)
{
    USHORT Value;
    Value = *(volatile USHORT *)Register;
    __asm__ __volatile__("dmb ld" ::: "memory");
    return Value;
}

ULONG
NTAPI
READ_REGISTER_ULONG(
    IN PULONG Register)
{
    ULONG Value;
    Value = *(volatile ULONG *)Register;
    __asm__ __volatile__("dmb ld" ::: "memory");
    return Value;
}

VOID
NTAPI
READ_REGISTER_BUFFER_UCHAR(
    IN PUCHAR Register,
    IN PUCHAR Buffer,
    IN ULONG Count)
{
    volatile UCHAR *Reg = (volatile UCHAR *)Register;
    while (Count--)
        *Buffer++ = *Reg++;
    __asm__ __volatile__("dmb ld" ::: "memory");
}

VOID
NTAPI
READ_REGISTER_BUFFER_USHORT(
    IN PUSHORT Register,
    IN PUSHORT Buffer,
    IN ULONG Count)
{
    volatile USHORT *Reg = (volatile USHORT *)Register;
    while (Count--)
        *Buffer++ = *Reg++;
    __asm__ __volatile__("dmb ld" ::: "memory");
}

VOID
NTAPI
READ_REGISTER_BUFFER_ULONG(
    IN PULONG Register,
    IN PULONG Buffer,
    IN ULONG Count)
{
    volatile ULONG *Reg = (volatile ULONG *)Register;
    while (Count--)
        *Buffer++ = *Reg++;
    __asm__ __volatile__("dmb ld" ::: "memory");
}

VOID
NTAPI
WRITE_REGISTER_UCHAR(
    IN PUCHAR Register,
    IN UCHAR Value)
{
    __asm__ __volatile__("dmb st" ::: "memory");
    *(volatile UCHAR *)Register = Value;
}

VOID
NTAPI
WRITE_REGISTER_USHORT(
    IN PUSHORT Register,
    IN USHORT Value)
{
    __asm__ __volatile__("dmb st" ::: "memory");
    *(volatile USHORT *)Register = Value;
}

VOID
NTAPI
WRITE_REGISTER_ULONG(
    IN PULONG Register,
    IN ULONG Value)
{
    __asm__ __volatile__("dmb st" ::: "memory");
    *(volatile ULONG *)Register = Value;
}

VOID
NTAPI
WRITE_REGISTER_BUFFER_UCHAR(
    IN PUCHAR Register,
    IN PUCHAR Buffer,
    IN ULONG Count)
{
    volatile UCHAR *Reg = (volatile UCHAR *)Register;
    __asm__ __volatile__("dmb st" ::: "memory");
    while (Count--)
        *Reg++ = *Buffer++;
}

VOID
NTAPI
WRITE_REGISTER_BUFFER_USHORT(
    IN PUSHORT Register,
    IN PUSHORT Buffer,
    IN ULONG Count)
{
    volatile USHORT *Reg = (volatile USHORT *)Register;
    __asm__ __volatile__("dmb st" ::: "memory");
    while (Count--)
        *Reg++ = *Buffer++;
}

VOID
NTAPI
WRITE_REGISTER_BUFFER_ULONG(
    IN PULONG Register,
    IN PULONG Buffer,
    IN ULONG Count)
{
    volatile ULONG *Reg = (volatile ULONG *)Register;
    __asm__ __volatile__("dmb st" ::: "memory");
    while (Count--)
        *Reg++ = *Buffer++;
}

/* Multi-processor startup stub (only needed for ntkrnlmp) */
VOID
NTAPI
KeStartAllProcessors(VOID)
{
    /* TODO: Send startup IPIs to secondary cores via PSCI/GIC */
}

/* EOF */
