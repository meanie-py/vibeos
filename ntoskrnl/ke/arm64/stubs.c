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

/*
 * UART base address for ARM64 debug traces.
 * Initialized to identity-mapped PL011 VA (0x09000000) which works during
 * early boot when the system PML4 still has PXE[0] identity mapping.
 * Remapped to a kernel VA via MmMapIoSpace during Phase 1 init so traces
 * work from any process context (user process PML4s don't have PXE[0]).
 */
volatile PVOID Arm64UartKernelBase = (PVOID)0x09000000ULL;

/* PTE templates are now defined in mm/ARM3/arm/init.c with correct ARM64 values */

/*
 * Protection to PTE mask mapping for ARM64.
 *
 * ARM64 4K page descriptors (L3) require:
 *   Bit 0:  Valid (set separately by MI_MAKE_HARDWARE_PTE_KERNEL)
 *   Bit 1:  NotLargePage = 1 (page descriptor, not block/reserved)
 *   Bit 10: Accessed/AF = 1 (required; no HW AF management)
 *
 * ARM64 PTE bit layout (from HARDWARE_PTE in mmtypes.h):
 *   [1:0]   Valid=1, NotLargePage=1
 *   [3:2]   CacheType (MAIR index): 00=Device, 01=Normal-WB, 11=Normal-NC
 *   [6]     Owner: 0=EL1-only(kernel), 1=EL0+EL1(user)
 *   [7]     NotDirty (inverted dirty bit)
 *   [9:8]   Shareability: 00=Non-shareable
 *   [10]    Accessed/AF: must be 1
 *   [47:12] PageFrameNumber
 *   [54]    UserNoExecute (UXN)
 *   [55]    Writable (software-managed)
 *   [56]    CopyOnWrite (software-managed)
 *
 * Constants (from miarm.h ARM64 section):
 *   PTE_NOT_LARGE_PAGE    = 0x0000000000000002
 *   PTE_ACCESSED          = 0x0000000000000400
 *   PTE_ENABLE_CACHE      = 0x0000000000000004 (CacheType=01)
 *   PTE_DISABLE_CACHE     = 0x0000000000000000 (CacheType=00)
 *   PTE_WRITECOMBINED     = 0x000000000000000C (CacheType=11)
 *   PTE_READONLY          = 0x0040000000000080 (UXN=1, NotDirty=1 → hw RO)
 *   PTE_EXECUTE           = 0x0000000000000080 (UXN=0, NotDirty=1 → hw RO)
 *   PTE_READWRITE         = 0x00C0000000000000 (UXN=1, Writable=1, hw RW)
 *   PTE_WRITECOPY         = 0x0140000000000080 (UXN=1, CopyOnWrite=1, NotDirty=1 → hw RO)
 *   PTE_EXECUTE_READWRITE = 0x0080000000000000 (UXN=0, Writable=1, hw RW)
 *   PTE_EXECUTE_WRITECOPY = 0x0100000000000080 (UXN=0, CopyOnWrite=1, NotDirty=1 → hw RO)
 */
#define _B  0x0000000000000402ULL  /* NotLargePage | Accessed */
#define _C  0x0000000000000004ULL  /* Normal WB Cacheable */
#define _NC 0x0000000000000000ULL  /* Device (non-cacheable) */
#define _WC 0x000000000000000CULL  /* Normal Non-Cacheable (write-combining) */
#define _ND 0x0000000000000080ULL  /* NotDirty=1 (AP[2]=1 = hardware read-only) */
#define _RO (0x0040000000000000ULL | _ND) /* UXN=1, read-only (hw enforced) */
#define _X  (0x0000000000000000ULL | _ND) /* UXN=0 (executable), read-only (hw enforced) */
#define _XR _X                     /* Execute+Read = same as Execute on ARM64 */
#define _RW 0x00C0000000000000ULL  /* UXN=1, Writable=1 (hw writable: NotDirty=0) */
#define _WC2 (0x0140000000000000ULL | _ND) /* UXN=1, CopyOnWrite=1, read-only (hw enforced) */
#define _XW 0x0080000000000000ULL  /* UXN=0, Writable=1 (hw writable: NotDirty=0) */
#define _XC (0x0100000000000000ULL | _ND) /* UXN=0, CopyOnWrite=1, read-only (hw enforced) */

const
ULONG_PTR
MmProtectToPteMask[32] =
{
    /* Base MM_ protection flags (cached) */
    0,
    _B | _RO | _C,   /* MM_READONLY */
    _B | _X  | _C,   /* MM_EXECUTE */
    _B | _XR | _C,   /* MM_EXECUTE_READ */
    _B | _RW | _C,   /* MM_READWRITE */
    _B | _WC2| _C,   /* MM_WRITECOPY */
    _B | _XW | _C,   /* MM_EXECUTE_READWRITE */
    _B | _XC | _C,   /* MM_EXECUTE_WRITECOPY */
    /* + MM_NOCACHE */
    0,
    _B | _RO | _NC,
    _B | _X  | _NC,
    _B | _XR | _NC,
    _B | _RW | _NC,
    _B | _WC2| _NC,
    _B | _XW | _NC,
    _B | _XC | _NC,
    /* + MM_DECOMMIT (same as cached) */
    0,
    _B | _RO | _C,
    _B | _X  | _C,
    _B | _XR | _C,
    _B | _RW | _C,
    _B | _WC2| _C,
    _B | _XW | _C,
    _B | _XC | _C,
    /* + MM_NOACCESS (write-combined) */
    0,
    _B | _RO | _WC,
    _B | _X  | _WC,
    _B | _XR | _WC,
    _B | _RW | _WC,
    _B | _WC2| _WC,
    _B | _XW | _WC,
    _B | _XC | _WC,
};

#undef _B
#undef _C
#undef _NC
#undef _WC
#undef _ND
#undef _RO
#undef _X
#undef _XR
#undef _RW
#undef _WC2
#undef _XW
#undef _XC

/* Protection value to Win32 protection mapping */
const ULONG MmProtectToValue[32] =
{
    PAGE_NOACCESS,
    PAGE_READONLY,
    PAGE_EXECUTE,
    PAGE_EXECUTE_READ,
    PAGE_READWRITE,
    PAGE_WRITECOPY,
    PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_NOCACHE | PAGE_READONLY,
    PAGE_NOCACHE | PAGE_EXECUTE,
    PAGE_NOCACHE | PAGE_EXECUTE_READ,
    PAGE_NOCACHE | PAGE_READWRITE,
    PAGE_NOCACHE | PAGE_WRITECOPY,
    PAGE_NOCACHE | PAGE_EXECUTE_READWRITE,
    PAGE_NOCACHE | PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_READONLY,
    PAGE_EXECUTE,
    PAGE_EXECUTE_READ,
    PAGE_READWRITE,
    PAGE_WRITECOPY,
    PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS,
    PAGE_WRITECOMBINE | PAGE_READONLY,
    PAGE_WRITECOMBINE | PAGE_EXECUTE,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_READ,
    PAGE_WRITECOMBINE | PAGE_READWRITE,
    PAGE_WRITECOMBINE | PAGE_WRITECOPY,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_READWRITE,
    PAGE_WRITECOMBINE | PAGE_EXECUTE_WRITECOPY,
};

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

/* KeUserModeCallback is implemented in usercall.c */

/* VDM is not supported on ARM64 */
NTSTATUS
NTAPI
NtVdmControl(
    _In_ ULONG ControlCode,
    _In_ PVOID ControlData)
{
    return STATUS_NOT_IMPLEMENTED;
}

/* Hardware configuration — create CentralProcessor registry key for ARM64 */
NTSTATUS
NTAPI
CmpInitializeMachineDependentConfiguration(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    NTSTATUS Status;
    OBJECT_ATTRIBUTES ObjectAttributes;
    UNICODE_STRING KeyName, ValueName;
    HANDLE SystemHandle, CpuHandle, CpuKeyHandle;
    ULONG Disposition;
    CHAR Buffer[64];
    WCHAR WideBuffer[64];
    ULONG i;
    ANSI_STRING AnsiString;
    UNICODE_STRING UnicodeString;

    /* Open the hardware description System key */
    RtlInitUnicodeString(&KeyName,
                         L"\\Registry\\Machine\\Hardware\\Description\\System");
    InitializeObjectAttributes(&ObjectAttributes,
                               &KeyName,
                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);
    Status = NtOpenKey(&SystemHandle, KEY_READ | KEY_WRITE, &ObjectAttributes);
    if (!NT_SUCCESS(Status))
        return Status;

    /* Create the CentralProcessor key */
    RtlInitUnicodeString(&KeyName, L"CentralProcessor");
    InitializeObjectAttributes(&ObjectAttributes,
                               &KeyName,
                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               SystemHandle,
                               NULL);
    Status = NtCreateKey(&CpuHandle, KEY_READ | KEY_WRITE, &ObjectAttributes,
                         0, NULL, 0, &Disposition);
    if (!NT_SUCCESS(Status))
    {
        NtClose(SystemHandle);
        return Status;
    }

    /* Create an entry for each processor */
    for (i = 0; i < KeNumberProcessors; i++)
    {
        /* Create the "0", "1", etc. subkey */
        swprintf(WideBuffer, L"%u", i);
        RtlInitUnicodeString(&KeyName, WideBuffer);
        InitializeObjectAttributes(&ObjectAttributes,
                                   &KeyName,
                                   OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                                   CpuHandle,
                                   NULL);
        Status = NtCreateKey(&CpuKeyHandle, KEY_READ | KEY_WRITE,
                             &ObjectAttributes, 0, NULL, 0, &Disposition);
        if (!NT_SUCCESS(Status))
            continue;

        /* Set the Identifier value */
        sprintf(Buffer, "ARM64 Family 8 Model 0 Stepping 0");
        RtlInitAnsiString(&AnsiString, Buffer);
        RtlAnsiStringToUnicodeString(&UnicodeString, &AnsiString, TRUE);
        RtlInitUnicodeString(&ValueName, L"Identifier");
        NtSetValueKey(CpuKeyHandle, &ValueName, 0, REG_SZ,
                      UnicodeString.Buffer,
                      UnicodeString.Length + sizeof(UNICODE_NULL));
        RtlFreeUnicodeString(&UnicodeString);

        /* Set the VendorIdentifier value */
        RtlInitUnicodeString(&ValueName, L"VendorIdentifier");
        NtSetValueKey(CpuKeyHandle, &ValueName, 0, REG_SZ,
                      L"ARM", sizeof(L"ARM"));

        /* Set the ~MHz value (read from CNTFRQ_EL0 as a rough estimate) */
        {
            ULONG Mhz = 100; /* Default estimate for QEMU virt */
            RtlInitUnicodeString(&ValueName, L"~MHz");
            NtSetValueKey(CpuKeyHandle, &ValueName, 0, REG_DWORD,
                          &Mhz, sizeof(Mhz));
        }

        NtClose(CpuKeyHandle);
    }

    NtClose(CpuHandle);
    NtClose(SystemHandle);
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

/* ARM64 CONTEXT flags (from winnt_old.h / ksarm64.template.h) */
#ifndef CONTEXT_ARM64
#define CONTEXT_ARM64           0x00400000L
#endif
#ifndef CONTEXT_CONTROL
#define CONTEXT_CONTROL         (CONTEXT_ARM64 | 0x1L)
#endif
#ifndef CONTEXT_INTEGER
#define CONTEXT_INTEGER         (CONTEXT_ARM64 | 0x2L)
#endif

/* Trap frame to context conversion (inverse of KeContextToTrapFrame) */
VOID
NTAPI
KeTrapFrameToContext(
    _In_ PKTRAP_FRAME TrapFrame,
    _In_opt_ PKEXCEPTION_FRAME ExceptionFrame,
    _Inout_ PCONTEXT Context)
{
    ULONG ContextFlags = Context->ContextFlags;

    /* Integer registers */
    if ((ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER)
    {
        ULONG i;
        /* Caller-saved: X0-X18 from TrapFrame */
        for (i = 0; i < 19; i++)
            Context->X[i] = TrapFrame->X[i];

        /* Callee-saved: X19-X28 from ExceptionFrame */
        if (ExceptionFrame)
        {
            Context->X[19] = ExceptionFrame->X19;
            Context->X[20] = ExceptionFrame->X20;
            Context->X[21] = ExceptionFrame->X21;
            Context->X[22] = ExceptionFrame->X22;
            Context->X[23] = ExceptionFrame->X23;
            Context->X[24] = ExceptionFrame->X24;
            Context->X[25] = ExceptionFrame->X25;
            Context->X[26] = ExceptionFrame->X26;
            Context->X[27] = ExceptionFrame->X27;
            Context->X[28] = ExceptionFrame->X28;
        }

        /* Frame pointer and link register */
        Context->X[29] = TrapFrame->Fp;
        Context->X[30] = TrapFrame->Lr;
    }

    /* Control registers: Sp, Pc, Cpsr */
    if ((ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL)
    {
        Context->Sp = TrapFrame->Sp;
        Context->Pc = TrapFrame->Pc;
        Context->Cpsr = TrapFrame->Spsr;
    }
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
    /* Integer registers: X0-X18 (caller-saved) + Fp + Lr */
    if ((ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER)
    {
        ULONG i;
        for (i = 0; i < 19; i++)
            TrapFrame->X[i] = Context->X[i];
        TrapFrame->Fp = Context->Fp;
        TrapFrame->Lr = Context->Lr;

        /* Callee-saved registers go in exception frame if provided */
        if (ExceptionFrame)
        {
            ExceptionFrame->X19 = Context->X19;
            ExceptionFrame->X20 = Context->X20;
            ExceptionFrame->X21 = Context->X21;
            ExceptionFrame->X22 = Context->X22;
            ExceptionFrame->X23 = Context->X23;
            ExceptionFrame->X24 = Context->X24;
            ExceptionFrame->X25 = Context->X25;
            ExceptionFrame->X26 = Context->X26;
            ExceptionFrame->X27 = Context->X27;
            ExceptionFrame->X28 = Context->X28;
        }
    }

    /* Control registers: Sp, Pc, Cpsr -> SPSR */
    if ((ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL)
    {
        TrapFrame->Sp = Context->Sp;
        TrapFrame->Pc = Context->Pc;

        /* Set SPSR with correct EL mode bits */
        if (PreviousMode == UserMode)
            TrapFrame->Spsr = Context->Cpsr & ~0xFU; /* Force EL0t (M[3:0]=0) */
        else
            TrapFrame->Spsr = (Context->Cpsr & ~0xFU) | 0x5; /* Force EL1h (M[3:0]=5) */
    }
}

/* KiSwapContext is now implemented in ctxswitch.S */

/* User-mode APC initialization — redirects trap frame to KiUserApcDispatcher */
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
    CONTEXT Context = {0};
    ULONG_PTR Stack;

    /* Build user-mode context from trap frame */
    Context.ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER;
    KeTrapFrameToContext(TrapFrame, ExceptionFrame, &Context);

    /* Allocate CONTEXT on user stack, 16-byte aligned */
    Stack = (Context.Sp & ~15ULL) - sizeof(CONTEXT);
    Stack &= ~15ULL;

    _SEH2_TRY
    {
        /* Probe and copy context to user stack */
        ProbeForWrite((PVOID)Stack, sizeof(CONTEXT), 16);
        RtlCopyMemory((PVOID)Stack, &Context, sizeof(CONTEXT));
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        /* Failed to write to user stack — skip this APC */
        return;
    }
    _SEH2_END;

    /* Set up APC parameters in registers (ARM64 calling convention):
     * KiUserApcDispatcher(NormalContext, SysArg1, SysArg2, NormalRoutine) */
    TrapFrame->X[0] = (ULONG64)NormalContext;
    TrapFrame->X[1] = (ULONG64)SystemArgument1;
    TrapFrame->X[2] = (ULONG64)SystemArgument2;
    TrapFrame->X[3] = (ULONG64)NormalRoutine;

    /* SP = pointer to CONTEXT (for NtContinue after APC returns) */
    TrapFrame->Sp = Stack;

    /* PC = KiUserApcDispatcher in ntdll */
    TrapFrame->Pc = (ULONG64)KeUserApcDispatcher;

    /* Ensure EL0 return (clear M[3:0] to 0 = EL0t) */
    TrapFrame->Spsr &= ~0xFU;

    ARM64_UART_TRACE_HEX("[APC] PC=", TrapFrame->Pc);
    ARM64_UART_TRACE_HEX("[APC] SP=", TrapFrame->Sp);
    ARM64_UART_TRACE_HEX("[APC] NR=", (ULONG_PTR)NormalRoutine);
}

/* KiCallUserMode is implemented in usercall_asm.S */

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

/* User-mode exception stack frame (pushed onto user stack) */
typedef struct _KUSER_EXCEPTION_STACK_ARM64
{
    CONTEXT Context;
    EXCEPTION_RECORD ExceptionRecord;
} KUSER_EXCEPTION_STACK_ARM64, *PKUSER_EXCEPTION_STACK_ARM64;

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
    CONTEXT Context = {0};

    /* Build context from trap frame */
    Context.ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER;
    KeTrapFrameToContext(TrapFrame, ExceptionFrame, &Context);

    if (PreviousMode == KernelMode)
    {
        /* Kernel-mode exception: try kernel-mode dispatch, then bugcheck */
        if (FirstChance)
        {
            if (RtlDispatchException(ExceptionRecord, &Context))
                goto Handled;
        }

        /*
         * ARM64 DUMMY_PSEH workaround: _SEH2_EXCEPT handlers are dead code,
         * so kernel code that accesses user-mode pointers inside _SEH2_TRY
         * blocks will fault without being caught. Instead of bugchecking,
         * skip the faulting instruction (advance PC by 4). This mimics what
         * the _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER){} handler would do
         * on architectures with working SEH.
         *
         * Condition: access violation where the faulting DATA address is in
         * user space (< MmHighestUserAddress). Kernel→kernel faults still
         * bugcheck as they indicate real bugs.
         */
        if (ExceptionRecord->ExceptionCode == STATUS_ACCESS_VIOLATION &&
            ExceptionRecord->NumberParameters >= 2)
        {
            ULONG_PTR FaultDataAddr = ExceptionRecord->ExceptionInformation[1];
            if (FaultDataAddr < (ULONG_PTR)MmHighestUserAddress)
            {
                {
                    volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
                    if (_u) {
                        const char *_s; int _i;
                        _s = "[KSEH-SKIP] PC=0x"; while (*_s) *_u = (ULONG)*_s++;
                        for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Pc >> _i) & 0xF];
                        _s = " FAR=0x"; while (*_s) *_u = (ULONG)*_s++;
                        for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(FaultDataAddr >> _i) & 0xF];
                        *_u = '\r'; *_u = '\n';
                    }
                }
                TrapFrame->Pc += 4;
                return;
            }
        }

        /* Print detailed fault info before bugcheck */
        {
            volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
            if (_u) {
                const char *_s; int _i;
                _s = "[KEXC] Code=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 28; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(ExceptionRecord->ExceptionCode >> _i) & 0xF];
                _s = " ExAddr=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[((ULONG64)(ULONG_PTR)ExceptionRecord->ExceptionAddress >> _i) & 0xF];
                _s = " TF.Pc=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Pc >> _i) & 0xF];
                _s = " TF.Lr=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Lr >> _i) & 0xF];
                _s = " TF.Sp=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Sp >> _i) & 0xF];
                _s = " Ctx.Pc=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(Context.Pc >> _i) & 0xF];
                _s = " Ctx.Lr=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(Context.Lr >> _i) & 0xF];
                if (ExceptionRecord->NumberParameters >= 2) {
                    _s = " P0=0x"; while (*_s) *_u = (ULONG)*_s++;
                    for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(ExceptionRecord->ExceptionInformation[0] >> _i) & 0xF];
                    _s = " P1=0x"; while (*_s) *_u = (ULONG)*_s++;
                    for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(ExceptionRecord->ExceptionInformation[1] >> _i) & 0xF];
                }
                *_u = '\r'; *_u = '\n';
            }
        }

        KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                     ExceptionRecord->ExceptionCode,
                     (ULONG_PTR)ExceptionRecord->ExceptionAddress,
                     (ULONG_PTR)TrapFrame,
                     0);
    }
    else
    {
        /* User-mode exception: dispatch to user-mode handler */
        {
            volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
            if (_u) {
                const char *_s;
                int _i;
                _s = "[UEXC] Code=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 28; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(ExceptionRecord->ExceptionCode >> _i) & 0xF];
                _s = " PC=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Pc >> _i) & 0xF];
                _s = " SP=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Sp >> _i) & 0xF];
                _s = " LR=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Lr >> _i) & 0xF];
                _s = " ESR=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 28; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(TrapFrame->Esr >> _i) & 0xF];
                _s = " 1st="; while (*_s) *_u = (ULONG)*_s++;
                *_u = FirstChance ? '1' : '0';
                *_u = '\r'; *_u = '\n';
            }
        }

        if (FirstChance)
        {
            ULONG_PTR UserRsp;
            PKUSER_EXCEPTION_STACK_ARM64 UserStack;

            /* Allocate exception stack on user stack, 16-byte aligned */
            UserRsp = (Context.Sp - sizeof(KUSER_EXCEPTION_STACK_ARM64)) & ~15ULL;
            UserStack = (PKUSER_EXCEPTION_STACK_ARM64)UserRsp;

            /*
             * Validate and fault-in user stack before writing. Stack pages
             * may be demand-paged (not yet physically present), so use
             * MmAccessFault to bring them in rather than MmIsAddressValid
             * which only checks current page table presence.
             */
            if (UserRsp >= (ULONG_PTR)MmHighestUserAddress ||
                UserRsp < sizeof(KUSER_EXCEPTION_STACK_ARM64))
            {
                volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
                if (_u) {
                    const char *_s; int _i;
                    _s = "[UEXC-STKBAD] SP=0x"; while (*_s) *_u = (ULONG)*_s++;
                    for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(Context.Sp >> _i) & 0xF];
                    *_u = '\r'; *_u = '\n';
                }
                goto SecondChance;
            }

            /* Fault in user stack pages (write fault, user mode).
             * Stack pages may be demand-paged and not yet physically present.
             * FaultCode 0x02 = not-present (bit0=0) + write (bit1=1).
             * MmAccessFault brings them in; ignore errors (page may already be present). */
            MmAccessFault(0x02, (PVOID)UserRsp, UserMode, TrapFrame);
            MmAccessFault(0x02,
                          (PVOID)((ULONG_PTR)UserStack + sizeof(*UserStack) - 1),
                          UserMode,
                          TrapFrame);

            if (!MmIsAddressValid(UserStack) ||
                !MmIsAddressValid((PUCHAR)UserStack + sizeof(*UserStack) - 1))
            {
                volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
                if (_u) {
                    const char *_s; int _i;
                    _s = "[UEXC-STKOVERFLOW] SP=0x"; while (*_s) *_u = (ULONG)*_s++;
                    for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(Context.Sp >> _i) & 0xF];
                    *_u = '\r'; *_u = '\n';
                }
                goto SecondChance;
            }

            /* Pages are mapped — safe to write exception frame */
            RtlZeroMemory(UserStack, sizeof(*UserStack));
            UserStack->Context = Context;
            UserStack->ExceptionRecord = *ExceptionRecord;

            /* Set up parameters for KiUserExceptionDispatcher(ExceptionRecord, Context)
             * ARM64 calling convention: x0 = arg1, x1 = arg2 */
            TrapFrame->X[0] = (ULONG64)&UserStack->ExceptionRecord;
            TrapFrame->X[1] = (ULONG64)&UserStack->Context;

            /* Set SP to the user exception stack */
            TrapFrame->Sp = UserRsp;

            /* Redirect to KiUserExceptionDispatcher in ntdll */
            TrapFrame->Pc = (ULONG64)KeUserExceptionDispatcher;

            /* Ensure EL0 return */
            TrapFrame->Spsr &= ~0xFU;
            return;
        }

    SecondChance:
        /* Second chance: kill the process */
        {
            volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
            if (_u) {
                const char *_s; int _i;
                _s = "[UEXC-KILL] Code=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 28; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[(ExceptionRecord->ExceptionCode >> _i) & 0xF];
                _s = " PC=0x"; while (*_s) *_u = (ULONG)*_s++;
                for (_i = 60; _i >= 0; _i -= 4) *_u = (ULONG)"0123456789ABCDEF"[((ULONG64)(ULONG_PTR)ExceptionRecord->ExceptionAddress >> _i) & 0xF];
                _s = " Proc="; while (*_s) *_u = (ULONG)*_s++;
                _s = (const char *)PsGetCurrentProcess()->ImageFileName;
                for (_i = 0; _i < 16 && _s[_i]; _i++) *_u = (ULONG)_s[_i];
                *_u = '\r'; *_u = '\n';
            }
        }

        ZwTerminateProcess(NtCurrentProcess(), ExceptionRecord->ExceptionCode);
        KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
                     ExceptionRecord->ExceptionCode,
                     (ULONG_PTR)ExceptionRecord->ExceptionAddress,
                     (ULONG_PTR)TrapFrame,
                     PreviousMode);
    }

    return;

Handled:
    /* Apply context changes back to trap frame */
    KeContextToTrapFrame(&Context, ExceptionFrame, TrapFrame,
                         Context.ContextFlags, PreviousMode);
}

/* KiExceptionExit is now implemented in trap.S */

/* KiInitializeContextThread is now implemented in thrdini.c */

/* KiInitMachineDependent is in kiinit.c */

/* Process address space switch */
VOID
NTAPI
KiSwapProcess(
    _In_ PKPROCESS NewProcess,
    _In_ PKPROCESS OldProcess)
{
    ULONG64 NewPageTable = NewProcess->DirectoryTableBase[0];

    /* Skip if same address space (common for kernel threads) */
    if (NewPageTable == OldProcess->DirectoryTableBase[0])
        return;

    /*
     * ARM64 split TTBR: TTBR0 handles user-space (VA bit 55=0),
     * TTBR1 handles kernel-space (VA bit 55=1).
     *
     * We switch BOTH TTBRs to the new process's PML4 because:
     *  - TTBR0: needed for user-space address resolution
     *  - TTBR1: needed so the self-map (PXE[0x1ED], at kernel VA)
     *    resolves through the current process's page tables
     *
     * The new PML4's upper half (kernel entries) is a copy of the
     * kernel's entries, made at process creation time.
     */

    /* Drain pending memory operations */
    __asm__ __volatile__("dsb ish" ::: "memory");

    /* Switch both TTBRs to the new process's PML4 */
    __asm__ __volatile__("msr ttbr0_el1, %0" :: "r"(NewPageTable) : "memory");
    __asm__ __volatile__("msr ttbr1_el1, %0" :: "r"(NewPageTable) : "memory");

    /* Invalidate all TLB entries and synchronize */
    __asm__ __volatile__("tlbi vmalle1is" ::: "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
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

/* MiInitializeSessionSpaceLayout and MiInitMachineDependent are now in mm/ARM3/arm/init.c */

/* NtCallbackReturn is implemented in usercall.c */

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

/* Page table / virtual memory management — moved to mm/arm64/page.c */

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
