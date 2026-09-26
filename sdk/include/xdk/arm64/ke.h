$if (_WDMDDK_)
/** Kernel definitions for ARM64 **/

/* Interrupt request levels */
#define PASSIVE_LEVEL           0
#define LOW_LEVEL               0
#define APC_LEVEL               1
#define DISPATCH_LEVEL          2
#define CLOCK_LEVEL             13
#define IPI_LEVEL               14
#define DRS_LEVEL               14
#define POWER_LEVEL             14
#define PROFILE_LEVEL           15
#define HIGH_LEVEL              15

/* Kernel-mode shared user data address (same as AMD64) */
#define KI_USER_SHARED_DATA     0xFFFFF78000000000ULL

#define SharedUserData          ((KUSER_SHARED_DATA * const)KI_USER_SHARED_DATA)

/* Shared system/tick time offsets (same as AMD64) */
#define SharedInterruptTime     (KI_USER_SHARED_DATA + 0x8)
#define SharedSystemTime        (KI_USER_SHARED_DATA + 0x14)
#define SharedTickCount         (KI_USER_SHARED_DATA + 0x320)

#define KeQueryInterruptTime() \
    (*(volatile ULONG64*)SharedInterruptTime)

#define KeQuerySystemTime(CurrentCount) \
    *(ULONG64*)(CurrentCount) = *(volatile ULONG64*)SharedSystemTime

#define KeQueryTickCount(CurrentCount) \
    *(ULONG64*)(CurrentCount) = *(volatile ULONG64*)SharedTickCount

#define KeGetDcacheFillSize() 1L

/* ARM64 yield instruction for spin loops */
#define YieldProcessor __yield

#define PAGE_SIZE               0x1000
#define PAGE_SHIFT              12L

#define PAUSE_PROCESSOR YieldProcessor();

/* arm64 is 64-bit: use amd64's stack sizing, not i386's 12288 (the original
 * port copied the i386 value here). Deep Rust kernel stacks + no dedicated arm64
 * exception stack + no GUI large-stack switch on arm64 make an undersized base
 * stack overflow intermittently on the thread-exit teardown path. */
#define KERNEL_STACK_SIZE                   0xC000  /* 48KB, matches amd64 */
#define KERNEL_LARGE_STACK_SIZE             0x12000 /* 72KB, matches amd64 */
#define KERNEL_LARGE_STACK_COMMIT KERNEL_STACK_SIZE

#define EXCEPTION_READ_FAULT    0
#define EXCEPTION_WRITE_FAULT   1
#define EXCEPTION_EXECUTE_FAULT 8

NTSYSAPI
PKTHREAD
NTAPI
KeGetCurrentThread(VOID);

typedef UCHAR KFLOATING_SAVE, *PKFLOATING_SAVE;

_Always_(_Post_satisfies_(return<=0))
_Must_inspect_result_
_IRQL_requires_max_(DISPATCH_LEVEL)
_Kernel_float_saved_
_At_(*FloatSave, _Kernel_requires_resource_not_held_(FloatState) _Kernel_acquires_resource_(FloatState))
FORCEINLINE
NTSTATUS
KeSaveFloatingPointState(
    _Out_ PKFLOATING_SAVE FloatSave)
{
    UNREFERENCED_PARAMETER(FloatSave);
    return STATUS_SUCCESS;
}

_Success_(1)
_Kernel_float_restored_
_At_(*FloatSave, _Kernel_requires_resource_held_(FloatState) _Kernel_releases_resource_(FloatState))
FORCEINLINE
NTSTATUS
KeRestoreFloatingPointState(
    _In_ PKFLOATING_SAVE FloatSave)
{
    UNREFERENCED_PARAMETER(FloatSave);
    return STATUS_SUCCESS;
}

#if (NTDDI_VERSION >= NTDDI_WIN7)
FORCEINLINE
ULONG
KeGetCurrentProcessorIndex(VOID)
{
    /* FIXME: Read from KPCR when properly implemented */
    return 0;
}
#endif

/* VOID
 * KeFlushIoBuffers(
 *   IN PMDL Mdl,
 *   IN BOOLEAN ReadOperation,
 *   IN BOOLEAN DmaOperation)
 */
#define KeFlushIoBuffers(_Mdl, _ReadOperation, _DmaOperation) \
    _DataSynchronizationBarrier()

#define DbgRaiseAssertionFailure() __break(0xf001)

FORCEINLINE
VOID
KeMemoryBarrier(VOID)
{
    _ReadWriteBarrier();
    MemoryBarrier();
}

#define KeMemoryBarrierWithoutFence() _ReadWriteBarrier()

_IRQL_requires_max_(HIGH_LEVEL)
_IRQL_saves_
NTHALAPI
KIRQL
NTAPI
KeGetCurrentIrql(
    VOID);

_IRQL_requires_max_(HIGH_LEVEL)
NTHALAPI
VOID
FASTCALL
KfLowerIrql(
    _In_ _IRQL_restores_ _Notliteral_ KIRQL NewIrql);
#define KeLowerIrql(a) KfLowerIrql(a)

_IRQL_requires_max_(HIGH_LEVEL)
_IRQL_raises_(NewIrql)
_IRQL_saves_
NTHALAPI
KIRQL
FASTCALL
KfRaiseIrql(
    _In_ KIRQL NewIrql);
#define KeRaiseIrql(a,b) *(b) = KfRaiseIrql(a)

_IRQL_requires_max_(DISPATCH_LEVEL)
_IRQL_saves_
_IRQL_raises_(DISPATCH_LEVEL)
NTHALAPI
KIRQL
NTAPI
KeRaiseIrqlToDpcLevel(VOID);

NTHALAPI
KIRQL
NTAPI
KeRaiseIrqlToSynchLevel(VOID);

$endif (_WDMDDK_)
$if (_NTDDK_)

#define ARM64_MAX_BREAKPOINTS 8
#define ARM64_MAX_WATCHPOINTS 2

#define CONTEXT_ARM64           0x400000L
#define CONTEXT_CONTROL         (CONTEXT_ARM64 | 0x00000001L)
#define CONTEXT_INTEGER         (CONTEXT_ARM64 | 0x00000002L)
#define CONTEXT_FLOATING_POINT  (CONTEXT_ARM64 | 0x00000004L)
#define CONTEXT_DEBUG_REGISTERS (CONTEXT_ARM64 | 0x00000008L)
#define CONTEXT_X18             (CONTEXT_ARM64 | 0x00000010L)
#define CONTEXT_FULL            (CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_FLOATING_POINT)

typedef union NEON128 {
    struct {
        ULONGLONG Low;
        LONGLONG High;
    } DUMMYSTRUCTNAME;
    double D[2];
    float  S[4];
    USHORT H[8];
    UCHAR  B[16];
} NEON128, *PNEON128;
typedef NEON128 NEON128, *PNEON128;

typedef struct _CONTEXT {

    //
    // Control flags.
    //

    ULONG ContextFlags;

    //
    // Integer registers
    //

    ULONG Cpsr;
    union {
        struct {
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
    		ULONG64 Fp;
            ULONG64 Lr;
        } DUMMYSTRUCTNAME;
        ULONG64 X[31];
    } DUMMYUNIONNAME;

    ULONG64 Sp;
    ULONG64 Pc;

    //
    // Floating Point/NEON Registers
    //

    NEON128 V[32];
    ULONG Fpcr;
    ULONG Fpsr;

    //
    // Debug registers
    //

    ULONG Bcr[ARM64_MAX_BREAKPOINTS];
    ULONG64 Bvr[ARM64_MAX_BREAKPOINTS];
    ULONG Wcr[ARM64_MAX_WATCHPOINTS];
    ULONG64 Wvr[ARM64_MAX_WATCHPOINTS];

} CONTEXT, *PCONTEXT;

#define PCR_MINOR_VERSION 1
#define PCR_MAJOR_VERSION 1

typedef struct _KPCR
{
    _ANONYMOUS_UNION union
    {
        NT_TIB NtTib;
        _ANONYMOUS_STRUCT struct
        {
            ULONG64 TibPad0[2];
            PVOID Spare1;
            struct _KPCR *Self;
            struct _KPRCB *CurrentPrcb;
            PKSPIN_LOCK_QUEUE LockArray;
            PVOID Used_Self;
        };
    };
    KIRQL CurrentIrql;
    UCHAR SecondLevelCacheAssociativity;
    ULONG Unused0[3];
    USHORT MajorVersion;
    USHORT MinorVersion;
    ULONG StallScaleFactor;
    PVOID Unused1[3];
    ULONG KernelReserved[15];
    ULONG SecondLevelCacheSize;
    _ANONYMOUS_UNION union
    {
        USHORT SoftwareInterruptPending;
        struct
        {
            UCHAR ApcInterrupt;
            UCHAR DispatchInterrupt;
        };
    };
    USHORT InterruptPad;
    ULONG HalReserved[32];
    PVOID KdVersionBlock;
    PVOID Unused3;
    ULONG PcrAlign1[8];
} KPCR, *PKPCR;

/* ARM64 uses TPIDR_EL1 for PCR.  All kernel-mode code (ntoskrnl, HAL,
 * and boot drivers) runs at EL1 and can read TPIDR_EL1. */
FORCEINLINE
PKPCR
KeGetPcr(VOID)
{
    ULONG64 Pcr;
    __asm__ __volatile__("mrs %0, tpidr_el1" : "=r"(Pcr));
    return (PKPCR)Pcr;
}

#if (NTDDI_VERSION < NTDDI_WIN7) || !defined(NT_PROCESSOR_GROUPS)
FORCEINLINE
ULONG
KeGetCurrentProcessorNumber(VOID)
{
    /* FIXME: Implement proper ARM64 processor number access */
    return 0;
}
#endif /* (NTDDI_VERSION < NTDDI_WIN7) || !defined(NT_PROCESSOR_GROUPS) */

$endif
