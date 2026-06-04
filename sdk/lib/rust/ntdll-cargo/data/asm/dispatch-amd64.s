.intel_syntax noprefix


.cfi_sections .debug_frame


.altmacro
.macro .PROC name







    .globl \name
    \name:



    .seh_proc \name
.endm



.macro .ENDP







    .seh_endproc
.endm



.macro PUBLIC symbol
    .global \symbol
.endm


.macro public symbol
    .global \symbol
.endm



.macro GLOBAL_LABEL label
    \label:
.endm


.macro ASSUME p1 p2 p3 p4 p5 p6 p7 p8
.endm


.macro .endcode16
.endm


.macro ALIGN x
    .align x
.endm

.macro align x
    .align x
.endm
.macro ljmp segment, offset
    .byte 0xEA
    .long offset
    .word segment
.endm

.macro ljmp16 segment, offset
    .byte 0xEA
    .word offset
    .word segment
.endm

.macro data32 opcode:vararg
    .byte 0x66
    opcode
.endm

.macro retf
    .byte 0xCB
.endm


.macro EXTERN name:vararg
.endm


.macro END
.endm

.macro end
.endm


.macro .MODEL model
.endm

.macro .code
    .text
.endm

.macro .const
    .section .rdata
.endm





.macro FPO cdwLocals, cdwParams, cbProlog, cbRegs, fUseBP, cbFrame
    .if (cbFrame == 1)
        .cfi_signal_frame
    .endif
.endm


.macro .allocstack size
    .seh_stackalloc \size
.endm

.macro .pushframe param

    .ifc \param, code
    .seh_pushframe @code
    .else
    .seh_pushframe
    .endif
.endm

.macro .pushreg reg
    .seh_pushreg \reg
.endm

.macro .savereg reg, offset
    .seh_savereg \reg, \offset
.endm

.macro .savexmm128 reg, offset
    .seh_savexmm \reg, \offset
.endm

.macro .setframe reg, offset
    .seh_setframe \reg, \offset
.endm

.macro .endprolog
    .seh_endprologue
.endm



.macro .ALLOCSTACK size
    .seh_stackalloc \size
.endm

.macro .PUSHFRAME param
    .ifc \param, code
    .seh_pushframe @code
    .else
    .seh_pushframe
    .endif
.endm

.macro .PUSHREG reg
    .seh_pushreg \reg
.endm

.macro .SAVEREG reg, offset
    .seh_savereg \reg, \offset
.endm

.macro .SAVEXMM128 reg, offset
    .seh_savexmm \reg, \offset
.endm

.macro .SETFRAME reg, offset
    .seh_setframe \reg, \offset
.endm

.macro .ENDPROLOG
    .seh_endprologue
.endm


.macro .seh_code
    .text
.endm


.macro absolute address
    __absolute__address__ = \address
.endm

.macro resb name, size
    \name = __absolute__address__
    __absolute__address__ = __absolute__address__ + \size
.endm

.macro UNIMPLEMENTED2 file, line, func
    jmp 4f
1: .ascii "Unimplemented %s (%s:%d)", "\r", "\n", "\0"
2: .asciz "\func"
3: .asciz "\file"
4:
    sub rsp, 0x20
    lea rcx, 1b[rip]
    lea rdx, 2b[rip]
    lea r8, 3b[rip]
    mov r9, \line
    call DbgPrint
    add rsp, 0x20
.endm







.macro push_reg, Reg
    push Reg
    .pushreg Reg
.endm

.macro rex_push_reg, Reg
    db 0x048 ; REX prefix
    push Reg
    .pushreg Reg
.endm

.macro push_eflags
    pushfq
    .allocstack 8
.endm

.macro rex_push_eflags
    db 0x048 ; REX prefix
    pushfq
    .allocstack 8
.endm

.macro rex_jmp_reg, Reg
    db 0x048 ; REX prefix / hint tail call to unwinder
    jmp Reg
.endm

.macro ret_zero
    ret
.endm

.macro alloc_stack, Size
    sub rsp, Size
    .allocstack Size
.endm

.macro save_reg, Reg, Offset
    mov [rsp + Offset], Reg
    .savereg Reg, Offset
.endm

.macro save_xmm128, Reg, Offset
    movaps [rsp + Offset], Reg
    .savexmm128 Reg, Offset
.endm

.macro push_frame, Code
    .pushframe Code
.endm

.macro set_frame, Reg, Offset
    .if Offset
        lea Reg, [rsp + Offset]
    .else
        mov Reg, rsp
    .endif
    .setframe Reg, Offset
.endm

.macro END_PROLOGUE
    .endprolog
.endm

.macro BEGIN_EPILOGUE
    .beginepilog
.endm

.macro LEAF_ENTRY, Name, Section, NoPad




    align 16
    public Name
.PROC Name
    END_PROLOGUE
.endm

.macro LEAF_ENTRY_ARG1, Name, Section, Arg1, NoPad




    align 16
    public Name
.PROC Name
    END_PROLOGUE
.endm

.macro LEAF_ENTRY_ARG2, Name, Section, Arg1, Arg2, NoPad




    align 16
    public Name
.PROC Name
    END_PROLOGUE
.endm

.macro LEAF_ENTRY_ARG3, Name, Section, Arg1, Arg2, Arg3, NoPad




    align 16
    public Name
.PROC Name
    END_PROLOGUE
.endm

.macro LEAF_END, Name, Section
.ENDP

.endm

.macro NESTED_ENTR, Name, Section, Handler, NoPad
    ifdef _CurrentSection_
        ifdif <Section>, _CurrentSection_
            .err <NESTED_ENTRY invoked for different sections within same module>
        .endif
    .endif
    _CurrentSection_ EQU <Section>
    Section segment para 'CODE'
    ifb <NoPad>
        db 6 dup (0cch)
    .endif
    align 16
    public Name
    ifb <Handler>
Name proc frame
    .else
Name proc frame:Handler
    .endif
.endm

.macro NESTED_END, Name, section
Name endp
Section ends
.endm

.macro ALTERNATE_ENTRY, Name
Name:
.endm

.macro Yield
    pause
.endm

.macro GENERATE_EXCEPTION_FRAME



    sub rsp, KEXCEPTION_FRAME_LENGTH - 8
    .allocstack (KEXCEPTION_FRAME_LENGTH - 8)


    mov [rsp + ExRbp], rbp
    .savereg rbp, ExRbp
    mov [rsp + ExRbx], rbx
    .savereg rbx, ExRbx
    mov [rsp +ExRdi], rdi
    .savereg rdi, ExRdi
    mov [rsp + ExRsi], rsi
    .savereg rsi, ExRsi
    mov [rsp + ExR12], r12
    .savereg r12, ExR12
    mov [rsp + ExR13], r13
    .savereg r13, ExR13
    mov [rsp + ExR14], r14
    .savereg r14, ExR14
    mov [rsp + ExR15], r15
    .savereg r15, ExR15
    movaps [rsp + ExXmm6], xmm6
    .savexmm128 xmm6, ExXmm6
    movaps [rsp + ExXmm7], xmm7
    .savexmm128 xmm7, ExXmm7
    movaps [rsp + ExXmm8], xmm8
    .savexmm128 xmm8, ExXmm8
    movaps [rsp + ExXmm9], xmm9
    .savexmm128 xmm9, ExXmm9
    movaps [rsp + ExXmm10], xmm10
    .savexmm128 xmm10, ExXmm10
    movaps [rsp + ExXmm11], xmm11
    .savexmm128 xmm11, ExXmm11
    movaps [rsp + ExXmm12], xmm12
    .savexmm128 xmm12, ExXmm12
    movaps [rsp + ExXmm13], xmm13
    .savexmm128 xmm13, ExXmm13
    movaps [rsp + ExXmm14], xmm14
    .savexmm128 xmm14, ExXmm14
    movaps [rsp + ExXmm15], xmm15
    .savexmm128 xmm15, ExXmm15
    .endprolog
.endm

.macro RESTORE_EXCEPTION_STATE


    mov rbp, [rsp + ExRbp]
    mov rbx, [rsp + ExRbx]
    mov rdi, [rsp + ExRdi]
    mov rsi, [rsp + ExRsi]
    mov r12, [rsp + ExR12]
    mov r13, [rsp + ExR13]
    mov r14, [rsp + ExR14]
    mov r15, [rsp + ExR15]
    movaps xmm6, [rsp + ExXmm6]
    movaps xmm7, [rsp + ExXmm7]
    movaps xmm8, [rsp + ExXmm8]
    movaps xmm9, [rsp + ExXmm9]
    movaps xmm10, [rsp + ExXmm10]
    movaps xmm11, [rsp + ExXmm11]
    movaps xmm12, [rsp + ExXmm12]
    movaps xmm13, [rsp + ExXmm13]
    movaps xmm14, [rsp + ExXmm14]
    movaps xmm15, [rsp + ExXmm15]


    add rsp, KEXCEPTION_FRAME_LENGTH - 8

.endm
SizeofPointer = 0x8


CPU_AMD = 0x1
CPU_INTEL = 0x2
CPU_VIA = 0x3


CR0_PE = 0x1
CR0_MP = 0x2
CR0_EM = 0x4
CR0_TS = 0x8
CR0_ET = 0x10
CR0_NE = 0x20
CR0_WP = 0x10000
CR0_AM = 0x40000
CR0_NW = 0x20000000
CR0_CD = 0x40000000
CR0_PG = 0x80000000


CR4_VME = 0x1
CR4_PVI = 0x2
CR4_TSD = 0x4
CR4_DE = 0x8
CR4_PSE = 0x10
CR4_PAE = 0x20
CR4_MCE = 0x40
CR4_PGE = 0x80
CR4_FXSR = 0x200
CR4_XMMEXCPT = 0x400
CR4_CHANNELS = 0x800
CR4_XSAVE = 0x40000
DEBUG_ACTIVE_DR7 = 0x1
DEBUG_ACTIVE_INSTRUMENTED = 0x2
DEBUG_ACTIVE_DBG_INSTRUMENTED = 0x3
DEBUG_ACTIVE_MINIMAL_THREAD = 0x4
DEBUG_ACTIVE_PRIMARY_THREAD = 0x80
DEBUG_ACTIVE_PRIMARY_THREAD_BIT = 0x7
DEBUG_ACTIVE_PRIMARY_THREAD_LOCK_BIT = 0x1f
DEBUG_ACTIVE_SCHEDULED_THREAD = 0x40
DEBUG_ACTIVE_SCHEDULED_THREAD_BIT = 0x6
DEBUG_ACTIVE_SCHEDULED_THREAD_LOCK = 0x40000000
DEBUG_ACTIVE_SCHEDULED_THREAD_LOCK_BIT = 0x1e


DR7_LEGAL = 0xffff0355
DR7_ACTIVE = 0x355
DR7_TRACE_BRANCH = 0x200
DR7_LAST_BRANCH = 0x100


EFLAGS_TF_MASK = 0x100
EFLAGS_TF_SHIFT = 0x8
EFLAGS_IF_MASK = 0x200
EFLAGS_IF_SHIFT = 0x9
EFLAGS_ID_MASK = 0x200000
EFLAGS_IF_BIT = 0x9


EXCEPTION_DIVIDED_BY_ZERO = 0x0
EXCEPTION_DEBUG = 0x1
EXCEPTION_NMI = 0x2
EXCEPTION_INT3 = 0x3
EXCEPTION_BOUND_CHECK = 0x5
EXCEPTION_INVALID_OPCODE = 0x6
EXCEPTION_NPX_NOT_AVAILABLE = 0x7
EXCEPTION_DOUBLE_FAULT = 0x8
EXCEPTION_NPX_OVERRUN = 0x9
EXCEPTION_INVALID_TSS = 0xa
EXCEPTION_SEGMENT_NOT_PRESENT = 0xb
EXCEPTION_STACK_FAULT = 0xc
EXCEPTION_GP_FAULT = 0xd
EXCEPTION_RESERVED_TRAP = 0xf
EXCEPTION_NPX_ERROR = 0x10
EXCEPTION_ALIGNMENT_CHECK = 0x11


FSW_INVALID_OPERATION = 0x1
FSW_DENORMAL = 0x2
FSW_ZERO_DIVIDE = 0x4
FSW_OVERFLOW = 0x8
FSW_UNDERFLOW = 0x10
FSW_PRECISION = 0x20
FSW_STACK_FAULT = 0x40
FSW_ERROR_SUMMARY = 0x80
FSW_CONDITION_CODE_0 = 0x100
FSW_CONDITION_CODE_1 = 0x200
FSW_CONDITION_CODE_2 = 0x400
FSW_CONDITION_CODE_3 = 0x4000
FSW_ERROR_MASK = 0x3f


KEXCEPTION_ACTIVE_INTERRUPT_FRAME = 0x0
KEXCEPTION_ACTIVE_EXCEPTION_FRAME = 0x1
KEXCEPTION_ACTIVE_SERVICE_FRAME = 0x2


KF_RDTSC = 0x2
KF_CR4 = 0x4
KF_GLOBAL_PAGE = 0x10
KF_LARGE_PAGE = 0x20
KF_CMPXCHG8B = 0x80
KF_FAST_SYSCALL = 0x1000
KF_BRANCH = 0x20000
KF_XSTATE = 0x800000
KF_XSAVEOPT_BIT = 0xf
KF_XSTATE_BIT = 0x17
KF_RDWRFSGSBASE_BIT = 0x1c


KGDT64_NULL = 0x0
KGDT64_R0_CODE = 0x10
KGDT64_R0_DATA = 0x18
KGDT64_R3_CMCODE = 0x20
KGDT64_R3_DATA = 0x28
KGDT64_R3_CODE = 0x30
KGDT64_SYS_TSS = 0x40
KGDT64_R3_CMTEB = 0x50
KGDT64_R0_LDT = 0x60


MACHINE_TYPE_ISA = 0x0
MACHINE_TYPE_EISA = 0x1
MACHINE_TYPE_MCA = 0x2


MSR_EFER = 0xc0000080
MSR_STAR = 0xc0000081
MSR_LSTAR = 0xc0000082
MSR_CSTAR = 0xc0000083
MSR_SYSCALL_MASK = 0xc0000084
MSR_FS_BASE = 0xc0000100
MSR_GS_BASE = 0xc0000101
MSR_GS_SWAP = 0xc0000102
MSR_MCG_STATUS = 0x17a
MSR_AMD_ACCESS = 0x9c5a203a
MSR_IA32_MISC_ENABLE = 0x1a0
MSR_DEBUG_CTL = 0x1d9
MSR_LAST_BRANCH_FROM = 0x1db
MSR_LAST_BRANCH_TO = 0x1dc
MSR_LAST_EXCEPTION_FROM = 0x1dd
MSR_LAST_EXCEPTION_TO = 0x1de


MSR_LMA = 0x400
MSR_LME = 0x100
MSR_SCE = 0x1
MSR_NXE = 0x800
MSR_PAT = 0x277






P1Home = 0x8
P2Home = 0x10
P3Home = 0x18
P4Home = 0x20
XSTATE_MASK_LEGACY_FLOATING_POINT = 0x1
XSTATE_MASK_LEGACY_SSE = 0x2
XSTATE_MASK_LEGACY = 0x3
XSTATE_MASK_GSSE = 0x4


XSW_INVALID_OPERATION = 0x1
XSW_DENORMAL = 0x2
XSW_ZERO_DIVIDE = 0x4
XSW_OVERFLOW = 0x8
XSW_UNDERFLOW = 0x10
XSW_PRECISION = 0x20
XSW_ERROR_MASK = 0x3f
XSW_ERROR_SHIFT = 0x7
XCW_INVALID_OPERATION = 0x80
XCW_DENORMAL = 0x100
XCW_ZERO_DIVIDE = 0x200
XCW_OVERFLOW = 0x400
XCW_UNDERFLOW = 0x800
XCW_PRECISION = 0x1000
XCW_ROUND_CONTROL = 0x6000
XCW_FLUSH_ZERO = 0x8000
INITIAL_FPCSR = 0x27f
INITIAL_MXCSR = 0x1f80


CONTEXT_XSTATE = 0x100040
EVENT_INCREMENT = 0x1
PF_COMPARE_EXCHANGE128 = 0xe


CpuEax = 0x0
CpuEbx = 0x4
CpuEcx = 0x8
CpuEdx = 0xc


CkBuffer = 0x20
CkLength = 0x28
CkApiNumber = 0x2c
CkRsp = 0x48
CkRip = 0x30
CalloutFrameLength = 0x58


CuTrapFrame = 0xd0
CuOutputBuffer = 0xe0
CuOutputLength = 0xe8


CxP1Home = 0x0
CxP2Home = 0x8
CxP3Home = 0x10
CxP4Home = 0x18
CxP5Home = 0x20
CxP6Home = 0x28
CxContextFlags = 0x30
CxMxCsr = 0x34
CxSegCs = 0x38
CxSegDs = 0x3a
CxSegEs = 0x3c
CxSegFs = 0x3e
CxSegGs = 0x40
CxSegSs = 0x42
CxEFlags = 0x44
CxDr0 = 0x48
CxDr1 = 0x50
CxDr2 = 0x58
CxDr3 = 0x60
CxDr6 = 0x68
CxDr7 = 0x70
CxRax = 0x78
CxRcx = 0x80
CxRdx = 0x88
CxRbx = 0x90
CxRsp = 0x98
CxRbp = 0xa0
CxRsi = 0xa8
CxRdi = 0xb0
CxR8 = 0xb8
CxR9 = 0xc0
CxR10 = 0xc8
CxR11 = 0xd0
CxR12 = 0xd8
CxR13 = 0xe0
CxR14 = 0xe8
CxR15 = 0xf0
CxRip = 0xf8
CxFltSave = 0x100
CxXmm0 = 0x1a0
CxXmm1 = 0x1b0
CxXmm2 = 0x1c0
CxXmm3 = 0x1d0
CxXmm4 = 0x1e0
CxXmm5 = 0x1f0
CxXmm6 = 0x200
CxXmm7 = 0x210
CxXmm8 = 0x220
CxXmm9 = 0x230
CxXmm10 = 0x240
CxXmm11 = 0x250
CxXmm12 = 0x260
CxXmm13 = 0x270
CxXmm14 = 0x280
CxXmm15 = 0x290
CxDebugControl = 0x4a8
CxLastBranchToRip = 0x4b0
CxLastBranchFromRip = 0x4b8
CxLastExceptionToRip = 0x4c0
CxLastExceptionFromRip = 0x4c8
CxVectorControl = 0x4a0
CxVectorRegister = 0x300
CONTEXT_FRAME_LENGTH = 0x4d0


DcControlPc = 0x0
DcImageBase = 0x8
DcFunctionEntry = 0x10
DcEstablisherFrame = 0x18
DcTargetIp = 0x20
DcContextRecord = 0x28
DcLanguageHandler = 0x30
DcHandlerData = 0x38
DcHistoryTable = 0x40
DcScopeIndex = 0x48


ExP1Home = 0x0
ExP2Home = 0x8
ExP3Home = 0x10
ExP4Home = 0x18
ExP5 = 0x20
ExXmm6 = 0x30
ExXmm7 = 0x40
ExXmm8 = 0x50
ExXmm9 = 0x60
ExXmm10 = 0x70
ExXmm11 = 0x80
ExXmm12 = 0x90
ExXmm13 = 0xa0
ExXmm14 = 0xb0
ExXmm15 = 0xc0
ExOutputBuffer = 0xe0
ExOutputLength = 0xe8
ExMxCsr = 0xf0
ExRbp = 0xf8
ExRbx = 0x100
ExRdi = 0x108
ExRsi = 0x110
ExR12 = 0x118
ExR13 = 0x120
ExR14 = 0x128
ExR15 = 0x130
ExReturn = 0x138
KEXCEPTION_FRAME_LENGTH = 0x140


JbFrame = 0x0
JbRbx = 0x8
JbRsp = 0x10
JbRbp = 0x18
JbRsi = 0x20
JbRdi = 0x28
JbR12 = 0x30
JbR13 = 0x38
JbR14 = 0x40
JbR15 = 0x48
JbRip = 0x50
JbMxCsr = 0x58
JbFpCsr = 0x5c
JbSpare = 0x5e
JbXmm6 = 0x60
JbXmm7 = 0x70
JbXmm8 = 0x80
JbXmm9 = 0x90
JbXmm10 = 0xa0
JbXmm11 = 0xb0
JbXmm12 = 0xc0
JbXmm13 = 0xd0
JbXmm14 = 0xe0
JbXmm15 = 0xf0


LfControlWord = 0x0
LfStatusWord = 0x2
LfTagWord = 0x4
LfErrorOpcode = 0x6
LfErrorOffset = 0x8
LfErrorSelector = 0xc
LfDataOffset = 0x10
LfDataSelector = 0x14
LfMxCsr = 0x18
LfMxCsr_Mask = 0x1c
LfFloatRegisters = 0x20
LfXmmRegisters = 0xa0


KgdtBaseLow = 0x2
KgdtBaseMiddle = 0x4
KgdtBaseHigh = 0x7
KgdtBaseUpper = 0x8
KgdtLimitHigh = 0x6
KgdtLimitLow = 0x0


MfRip = 0x0
MfSegCs = 0x8
MfEFlags = 0x10
MfRsp = 0x18
MfSegSs = 0x20
MachineFrameLength = 0x28


PbMxCsr = 0x0
PbNumber = 0x4
PbInterruptRequest = 0x6
PbIdleHalt = 0x7
PbCurrentThread = 0x8
PbNextThread = 0x10
PbIdleThread = 0x18
PbNestingLevel = 0x5
PbRspBase = 0x28
PbPrcbLock = 0x30
PbSetMember = 0x38
PbProcessorState = 0x40
PbCpuType = 0x5f0
PbCpuID = 0x5f1
PbCpuStep = 0x5f2
PbHalReserved = 0x5f8
PbMinorVersion = 0x638
PbMajorVersion = 0x63a
PbBuildType = 0x63c
PbCpuVendor = 0x63d
PbApicMask = 0x640
PbCFlushSize = 0x644
PbAcpiReserved = 0x648
PbInitialApicId = 0x63e
PbLockQueue = 0x670
PbPPLookasideList = 0x880
PbPPNPagedLookasideList = 0x980
PbPPPagedLookasideList = 0xb80
PbPacketBarrier = 0xd80
PbDeferredReadyListHead = 0xd88
PbLookasideIrpFloat = 0xdc4
PbTargetSet = 0xe00
PbIpiFrozen = 0xe08
PbRequestMailbox = 0xe80
PbSenderSummary = 0x2680
PbDpcStack = 0x2740
PbMaximumDpcQueueDepth = 0x2750
PbDpcRequestRate = 0x2754
PbMinimumDpcRate = 0x2758
PbDpcInterruptRequested = 0x275c
PbDpcThreadRequested = 0x275d
PbDpcRoutineActive = 0x275e
PbDpcThreadActive = 0x275f
PbTimerHand = 0x2760
PbTimerRequest = 0x2768
PbTickOffset = 0x2770
PbMasterOffset = 0x2774
PbDpcLastCount = 0x2778
PbQuantumEnd = 0x277d
PbDpcSetEventRequest = 0x2780
PbIdleSchedule = 0x277f
PbReadySummary = 0x2818
PbDispatcherReadyListHead = 0x2820
PbInterruptCount = 0x2a20
PbKernelTime = 0x2a24
PbUserTime = 0x2a28
PbDpcTime = 0x2a2c
PbInterruptTime = 0x2a30
PbAdjustDpcThreshold = 0x2a34
PbSkipTick = 0x2a38
PbPollSlot = 0x2a3a
PbParentNode = 0x2a48
PbMultiThreadProcessorSet = 0x2a50
PbMultiThreadSetMaster = 0x2a58
PbPageColor = 0x2a6c
PbNodeColor = 0x2a70
PbNodeShiftedColor = 0x2a74
PbSecondaryColorMask = 0x2a78
PbSleeping = 0x2a60
PbVendorString = 0x2abc
PbPowerState = 0x2ad8
ProcessorBlockLength = 0x2c90


PcUserRsp = 0x10
PcSelf = 0x18
PcCurrentPrcb = 0x20
PcLockArray = 0x28
PcIrql = 0x50
PcStallScaleFactor = 0x64
PcHalReserved = 0xc0
PcMxCsr = 0x180
PcCurrentThread = 0x188
PcRspBase = 0x1a8
PcSetMember = 0x1b8
PcCpuType = 0x770
PcCpuID = 0x771
PcCpuStep = 0x772
PcCpuVendor = 0x7bd
PcCFlushSize = 0x7c4
PcDeferredReadyListHead = 0xf08
PcSystemCalls = 0xf48
PcDpcRoutineActive = 0x28de
PcInterruptCount = 0x2ba0
PcDebuggerSavedIRQL = 0x2bb9
PcTickOffset = 0x28f0
PcMasterOffset = 0x28f4
PcSkipTick = 0x2bb8
ProcessorControlRegisterLength = 0x2e20


PsbCompletionFlag = 0x0
PsbFlags = 0x4
PsbGdt32 = 0x8
PsbIdt32 = 0xc
PsbGdt = 0x10
PsbTiledMemoryMap = 0x18
PsbPmTarget = 0x20
PsbLmIdentityTarget = 0x26
PsbLmTarget = 0x30
PsbSelfMap = 0x38
PsbMsrPat = 0x40
PsbMsrEFER = 0x48
PsbProcessorState = 0x50
ProcessorStartBlockLength = 0x600
PROCESSOR_START_FLAG_FORCE_ENABLE_NX = 0x1


PsSpecialRegisters = 0x0
PsCr0 = 0x0
PsCr2 = 0x8
PsCr3 = 0x10
PsCr4 = 0x18
PsKernelDr0 = 0x20
PsKernelDr1 = 0x28
PsKernelDr2 = 0x30
PsKernelDr3 = 0x38
PsKernelDr6 = 0x40
PsKernelDr7 = 0x48
PsGdtr = 0x50
PsIdtr = 0x60
PsTr = 0x70
PsLdtr = 0x72
PsMxCsr = 0x74
PsContextFrame = 0xe0
PsDebugControl = 0x78
PsLastBranchToRip = 0x80
PsLastBranchFromRip = 0x88
PsLastExceptionToRip = 0x90
PsLastExceptionFromRip = 0x98
PsCr8 = 0xa0
ProcessorStateLength = 0x5b0


SfP1Home = 0x0
SfP2Home = 0x8
SfP3Home = 0x10
SfP4Home = 0x18
SfReturn = 0x28
KSTART_FRAME_LENGTH = 0x30


SrKernelDr0 = 0x20
SrKernelDr1 = 0x28
SrKernelDr2 = 0x30
SrKernelDr3 = 0x38
SrKernelDr6 = 0x40
SrKernelDr7 = 0x48
SrGdtr = 0x50
SrIdtr = 0x60
SrTr = 0x70
SrMxCsr = 0x74
SrMsrGsBase = 0xa8
SrMsrGsSwap = 0xb0
SrMsrStar = 0xb8
SrMsrLStar = 0xc0
SrMsrCStar = 0xc8
SrMsrSyscallMask = 0xd0


StLowTime = 0x0
StHigh1Time = 0x4
StHigh2Time = 0x8


SwP5Home = 0x20
SwApcBypass = 0x28
SwRbp = 0x30
SwReturn = 0x38
SwitchFrameLength = 0x40
KSWITCH_FRAME_LENGTH = 0x40
KTRAP_FRAME_LENGTH = 0x190


TssRsp0 = 0x4
TssRsp1 = 0xc
TssRsp2 = 0x14
TssPanicStack = 0x24
TssMcaStack = 0x2c
TssNmiStack = 0x34
TssIoMapBase = 0x66
TssLength = 0x68


XcfgEnabledFeatures = 0x0


XctxMask = 0x0
XctxLength = 0x8
XctxArea = 0x10


XsaHeader = 0x200
XsaHeaderLength = 0x40




CONTEXT_P1Home = 0x0
CONTEXT_P2Home = 0x8
CONTEXT_P3Home = 0x10
CONTEXT_P4Home = 0x18
CONTEXT_P5Home = 0x20
CONTEXT_P6Home = 0x28
CONTEXT_ContextFlags = 0x30
CONTEXT_MxCsr = 0x34
CONTEXT_SegCs = 0x38
CONTEXT_SegDs = 0x3a
CONTEXT_SegEs = 0x3c
CONTEXT_SegFs = 0x3e
CONTEXT_SegGs = 0x40
CONTEXT_SegSs = 0x42
CONTEXT_EFlags = 0x44
CONTEXT_Dr0 = 0x48
CONTEXT_Dr1 = 0x50
CONTEXT_Dr2 = 0x58
CONTEXT_Dr3 = 0x60
CONTEXT_Dr6 = 0x68
CONTEXT_Dr7 = 0x70
CONTEXT_Rax = 0x78
CONTEXT_Rcx = 0x80
CONTEXT_Rdx = 0x88
CONTEXT_Rbx = 0x90
CONTEXT_Rsp = 0x98
CONTEXT_Rbp = 0xa0
CONTEXT_Rsi = 0xa8
CONTEXT_Rdi = 0xb0
CONTEXT_R8 = 0xb8
CONTEXT_R9 = 0xc0
CONTEXT_R10 = 0xc8
CONTEXT_R11 = 0xd0
CONTEXT_R12 = 0xd8
CONTEXT_R13 = 0xe0
CONTEXT_R14 = 0xe8
CONTEXT_R15 = 0xf0
CONTEXT_Rip = 0xf8
CONTEXT_FltSave = 0x100
CONTEXT_Xmm0 = 0x1a0
CONTEXT_Xmm1 = 0x1b0
CONTEXT_Xmm2 = 0x1c0
CONTEXT_Xmm3 = 0x1d0
CONTEXT_Xmm4 = 0x1e0
CONTEXT_Xmm5 = 0x1f0
CONTEXT_Xmm6 = 0x200
CONTEXT_Xmm7 = 0x210
CONTEXT_Xmm8 = 0x220
CONTEXT_Xmm9 = 0x230
CONTEXT_Xmm10 = 0x240
CONTEXT_Xmm11 = 0x250
CONTEXT_Xmm12 = 0x260
CONTEXT_Xmm13 = 0x270
CONTEXT_Xmm14 = 0x280
CONTEXT_Xmm15 = 0x290
CONTEXT_DebugControl = 0x4a8
CONTEXT_LastBranchToRip = 0x4b0
CONTEXT_LastBranchFromRip = 0x4b8
CONTEXT_LastExceptionToRip = 0x4c0
CONTEXT_LastExceptionFromRip = 0x4c8
CONTEXT_VectorControl = 0x4a0
CONTEXT_VectorRegister = 0x300
CONTEXT_FRAME_LENGTH = 0x4d0


KEXCEPTION_FRAME_P1Home = 0x0
KEXCEPTION_FRAME_P2Home = 0x8
KEXCEPTION_FRAME_P3Home = 0x10
KEXCEPTION_FRAME_P4Home = 0x18
KEXCEPTION_FRAME_P5 = 0x20
KEXCEPTION_FRAME_Xmm6 = 0x30
KEXCEPTION_FRAME_Xmm7 = 0x40
KEXCEPTION_FRAME_Xmm8 = 0x50
KEXCEPTION_FRAME_Xmm9 = 0x60
KEXCEPTION_FRAME_Xmm10 = 0x70
KEXCEPTION_FRAME_Xmm11 = 0x80
KEXCEPTION_FRAME_Xmm12 = 0x90
KEXCEPTION_FRAME_Xmm13 = 0xa0
KEXCEPTION_FRAME_Xmm14 = 0xb0
KEXCEPTION_FRAME_Xmm15 = 0xc0
KEXCEPTION_FRAME_MxCsr = 0xf0
KEXCEPTION_FRAME_Rbp = 0xf8
KEXCEPTION_FRAME_Rbx = 0x100
KEXCEPTION_FRAME_Rdi = 0x108
KEXCEPTION_FRAME_Rsi = 0x110
KEXCEPTION_FRAME_R12 = 0x118
KEXCEPTION_FRAME_R13 = 0x120
KEXCEPTION_FRAME_R14 = 0x128
KEXCEPTION_FRAME_R15 = 0x130
KEXCEPTION_FRAME_Return = 0x138
KEXCEPTION_FRAME_TrapFrame = 0xd0
KEXCEPTION_FRAME_OutputBuffer = 0xe0
KEXCEPTION_FRAME_OutputLength = 0xe8
KEXCEPTION_FRAME_LENGTH = 0x140


KTRAP_FRAME_P1Home = 0x0
KTRAP_FRAME_P2Home = 0x8
KTRAP_FRAME_P3Home = 0x10
KTRAP_FRAME_P4Home = 0x18
KTRAP_FRAME_P5 = 0x20
KTRAP_FRAME_PreviousMode = 0x28
KTRAP_FRAME_PreviousIrql = 0x29
KTRAP_FRAME_FaultIndicator = 0x2a
KTRAP_FRAME_ExceptionActive = 0x2b
KTRAP_FRAME_MxCsr = 0x2c
KTRAP_FRAME_Rax = 0x30
KTRAP_FRAME_Rcx = 0x38
KTRAP_FRAME_Rdx = 0x40
KTRAP_FRAME_R8 = 0x48
KTRAP_FRAME_R9 = 0x50
KTRAP_FRAME_R10 = 0x58
KTRAP_FRAME_R11 = 0x60
KTRAP_FRAME_GsBase = 0x68
KTRAP_FRAME_GsSwap = 0x68
KTRAP_FRAME_Xmm0 = 0x70
KTRAP_FRAME_Xmm1 = 0x80
KTRAP_FRAME_Xmm2 = 0x90
KTRAP_FRAME_Xmm3 = 0xa0
KTRAP_FRAME_Xmm4 = 0xb0
KTRAP_FRAME_Xmm5 = 0xc0
KTRAP_FRAME_FaultAddress = 0xd0
KTRAP_FRAME_TimeStampCKCL = 0xd0
KTRAP_FRAME_Dr0 = 0xd8
KTRAP_FRAME_Dr1 = 0xe0
KTRAP_FRAME_Dr2 = 0xe8
KTRAP_FRAME_Dr3 = 0xf0
KTRAP_FRAME_Dr6 = 0xf8
KTRAP_FRAME_Dr7 = 0x100
KTRAP_FRAME_DebugControl = 0x108
KTRAP_FRAME_LastBranchToRip = 0x110
KTRAP_FRAME_LastBranchFromRip = 0x118
KTRAP_FRAME_LastExceptionToRip = 0x120
KTRAP_FRAME_LastExceptionFromRip = 0x128
KTRAP_FRAME_LastBranchControl = 0x108
KTRAP_FRAME_LastBranchMSR = 0x110
KTRAP_FRAME_SegDs = 0x130
KTRAP_FRAME_SegEs = 0x132
KTRAP_FRAME_SegFs = 0x134
KTRAP_FRAME_SegGs = 0x136
KTRAP_FRAME_TrapFrame = 0x138
KTRAP_FRAME_Rbx = 0x140
KTRAP_FRAME_Rdi = 0x148
KTRAP_FRAME_Rsi = 0x150
KTRAP_FRAME_Rbp = 0x158
KTRAP_FRAME_ErrorCode = 0x160
KTRAP_FRAME_ExceptionFrame = 0x160
KTRAP_FRAME_TimeStampKlog = 0x160
KTRAP_FRAME_Rip = 0x168
KTRAP_FRAME_SegCs = 0x170
KTRAP_FRAME_Logging = 0x173
KTRAP_FRAME_EFlags = 0x178
KTRAP_FRAME_Rsp = 0x180
KTRAP_FRAME_SegSs = 0x188
KTRAP_FRAME_CodePatchCycle = 0x18c
KTRAP_FRAME_LENGTH = 0x190


EXCEPTION_RECORD_ExceptionCode = 0x0
EXCEPTION_RECORD_ExceptionFlags = 0x4
EXCEPTION_RECORD_ExceptionRecord = 0x8
EXCEPTION_RECORD_ExceptionAddress = 0x10
EXCEPTION_RECORD_NumberParameters = 0x18
EXCEPTION_RECORD_ExceptionInformation = 0x20


KTHREAD_WAIT_IRQL = 0x156
KTHREAD_TrapFrame = 0x1c8
KTHREAD_PreviousMode = 0x153
KTHREAD_KernelStack = 0x38
KTHREAD_UserApcPending = 0x72
KTHREAD_LargeStack = 0x1b3


KINTERRUPT_Type = 0x0
KINTERRUPT_Size = 0x2
KINTERRUPT_InterruptListEntry = 0x8
KINTERRUPT_ServiceRoutine = 0x18
KINTERRUPT_ServiceContext = 0x20
KINTERRUPT_SpinLock = 0x28
KINTERRUPT_TickCount = 0x30
KINTERRUPT_ActualLock = 0x38
KINTERRUPT_DispatchAddress = 0x40
KINTERRUPT_Vector = 0x48
KINTERRUPT_Irql = 0x4c
KINTERRUPT_SynchronizeIrql = 0x4d
KINTERRUPT_FloatingSave = 0x4e
KINTERRUPT_Connected = 0x4f
KINTERRUPT_Number = 0x50
KINTERRUPT_ShareVector = 0x51
KINTERRUPT_Mode = 0x54
KINTERRUPT_ServiceCount = 0x58
KINTERRUPT_DispatchCount = 0x5c
KINTERRUPT_TrapFrame = 0x60
KINTERRUPT_DispatchCode = 0x70


MAX_SYSCALL_PARAM_SIZE = 0x80


SizeofPointer = 0x8


APC_INDEX_MISMATCH = 0x1
INVALID_AFFINITY_SET = 0x3
INVALID_DATA_ACCESS_TRAP = 0x4
IRQL_NOT_GREATER_OR_EQUAL = 0x9
IRQL_NOT_LESS_OR_EQUAL = 0xa
NO_USER_MODE_CONTEXT = 0xe
SPIN_LOCK_ALREADY_OWNED = 0xf
SPIN_LOCK_NOT_OWNED = 0x10
THREAD_NOT_MUTEX_OWNER = 0x11
TRAP_CAUSE_UNKNOWN = 0x12
KMODE_EXCEPTION_NOT_HANDLED = 0x1e
KERNEL_APC_PENDING_DURING_EXIT = 0x20
PANIC_STACK_SWITCH = 0x2b
DATA_BUS_ERROR = 0x2e
INSTRUCTION_BUS_ERROR = 0x2f
SYSTEM_EXIT_OWNED_MUTEX = 0x39
PAGE_FAULT_WITH_INTERRUPTS_OFF = 0x49
IRQL_GT_ZERO_AT_SYSTEM_SERVICE = 0x4a
DATA_COHERENCY_EXCEPTION = 0x55
INSTRUCTION_COHERENCY_EXCEPTION = 0x56
HAL1_INITIALIZATION_FAILED = 0x61
UNEXPECTED_KERNEL_MODE_TRAP = 0x7f
NMI_HARDWARE_FAILURE = 0x80
SPIN_LOCK_INIT_FAILURE = 0x81
ATTEMPTED_SWITCH_FROM_DPC = 0xb8
KERNEL_SECURITY_CHECK_FAILURE = 0x139


BREAKPOINT_BREAK = 0x0
BREAKPOINT_PRINT = 0x1
BREAKPOINT_PROMPT = 0x2
BREAKPOINT_LOAD_SYMBOLS = 0x3
BREAKPOINT_UNLOAD_SYMBOLS = 0x4
BREAKPOINT_COMMAND_STRING = 0x5


CONTEXT_FULL = 0x10000b
CONTEXT_CONTROL = 0x100001
CONTEXT_INTEGER = 0x100002
CONTEXT_FLOATING_POINT = 0x100008
CONTEXT_DEBUG_REGISTERS = 0x100010
CONTEXT_SEGMENTS = 0x100004


EXCEPTION_NONCONTINUABLE = 0x1
EXCEPTION_UNWINDING = 0x2
EXCEPTION_EXIT_UNWIND = 0x4
EXCEPTION_STACK_INVALID = 0x8
EXCEPTION_NESTED_CALL = 0x10
EXCEPTION_TARGET_UNWIND = 0x20
EXCEPTION_COLLIDED_UNWIND = 0x40
EXCEPTION_UNWIND = 0x66
EXCEPTION_EXECUTE_HANDLER = 0x1
EXCEPTION_CONTINUE_SEARCH = 0x0
EXCEPTION_CONTINUE_EXECUTION = 0xffffffff


ExceptionContinueExecution = 0x0
ExceptionContinueSearch = 0x1
ExceptionNestedException = 0x2
ExceptionCollidedUnwind = 0x3


FAST_FAIL_GUARD_ICALL_CHECK_FAILURE = 0xa


InLevelSensitive = 0x0
InLatched = 0x1




PASSIVE_LEVEL = 0x0
APC_LEVEL = 0x1
DISPATCH_LEVEL = 0x2
CLOCK_LEVEL = 0xd
IPI_LEVEL = 0xe
POWER_LEVEL = 0xe
PROFILE_LEVEL = 0xf
HIGH_LEVEL = 0xf

SYNCH_LEVEL = 0x2





LOCK_QUEUE_WAIT = 0x1
LOCK_QUEUE_OWNER = 0x2
LockQueueDispatcherLock = 0x0


ProcessInMemory = 0x0
ProcessOutOfMemory = 0x1
ProcessInTransition = 0x2


KernelMode = 0x0
UserMode = 0x1


NUMBER_SERVICE_TABLES = 0x2
SERVICE_NUMBER_MASK = 0xfff
SERVICE_TABLE_SHIFT = 0x7
SERVICE_TABLE_MASK = 0x20
SERVICE_TABLE_TEST = 0x20


STATUS_ACCESS_VIOLATION = 0xc0000005
STATUS_ASSERTION_FAILURE = 0xc0000420
STATUS_ARRAY_BOUNDS_EXCEEDED = 0xc000008c
STATUS_BAD_COMPRESSION_BUFFER = 0xc0000242
STATUS_BREAKPOINT = 0x80000003
STATUS_CALLBACK_POP_STACK = 0xc0000423
STATUS_DATATYPE_MISALIGNMENT = 0x80000002
STATUS_FLOAT_DENORMAL_OPERAND = 0xc000008d
STATUS_FLOAT_DIVIDE_BY_ZERO = 0xc000008e
STATUS_FLOAT_INEXACT_RESULT = 0xc000008f
STATUS_FLOAT_INVALID_OPERATION = 0xc0000090
STATUS_FLOAT_OVERFLOW = 0xc0000091
STATUS_FLOAT_STACK_CHECK = 0xc0000092
STATUS_FLOAT_UNDERFLOW = 0xc0000093
STATUS_FLOAT_MULTIPLE_FAULTS = 0xc00002b4
STATUS_FLOAT_MULTIPLE_TRAPS = 0xc00002b5
STATUS_GUARD_PAGE_VIOLATION = 0x80000001
STATUS_ILLEGAL_FLOAT_CONTEXT = 0xc000014a
STATUS_ILLEGAL_INSTRUCTION = 0xc000001d
STATUS_INSTRUCTION_MISALIGNMENT = 0xc00000aa
STATUS_INVALID_HANDLE = 0xc0000008
STATUS_INVALID_LOCK_SEQUENCE = 0xc000001e
STATUS_INVALID_OWNER = 0xc000005a
STATUS_INVALID_PARAMETER = 0xc000000d
STATUS_INVALID_PARAMETER_1 = 0xc00000ef
STATUS_INVALID_SYSTEM_SERVICE = 0xc000001c
STATUS_INTEGER_DIVIDE_BY_ZERO = 0xc0000094
STATUS_INTEGER_OVERFLOW = 0xc0000095
STATUS_IN_PAGE_ERROR = 0xc0000006
STATUS_KERNEL_APC = 0x100
STATUS_LONGJUMP = 0x80000026
STATUS_NO_CALLBACK_ACTIVE = 0xc0000258
STATUS_NO_EVENT_PAIR = 0xc000014e
STATUS_PRIVILEGED_INSTRUCTION = 0xc0000096
STATUS_SINGLE_STEP = 0x80000004
STATUS_STACK_BUFFER_OVERRUN = 0xc0000409
STATUS_STACK_OVERFLOW = 0xc00000fd
STATUS_SUCCESS = 0x0
STATUS_THREAD_IS_TERMINATING = 0xc000004b
STATUS_TIMEOUT = 0x102
STATUS_UNWIND = 0xc0000027
STATUS_UNWIND_CONSOLIDATE = 0x80000029
STATUS_USER_APC = 0xc0
STATUS_WAKE_SYSTEM = 0x40000294
STATUS_WAKE_SYSTEM_DEBUGGER = 0x80000007


TLS_MINIMUM_AVAILABLE = 0x40
TLS_EXPANSION_SLOTS = 0x400


Initialized = 0x0
Ready = 0x1
Running = 0x2
Standby = 0x3
Terminated = 0x4
Waiting = 0x5


WrExecutive = 0x7
WrMutex = 0x1d
WrDispatchInt = 0x1f
WrQuantumEnd = 0x1e
WrEventPair = 0xe
WaitAny = 0x1
WaitAll = 0x0


KERNEL_STACK_SIZE = 0xc000
KERNEL_LARGE_STACK_SIZE = 0x12000
KERNEL_LARGE_STACK_COMMIT = 0xc000
DOUBLE_FAULT_STACK_SIZE = 0x2000
KERNEL_MCA_EXCEPTION_STACK_SIZE = 0x2000
NMI_STACK_SIZE = 0x2000
ISR_STACK_SIZE = 0x6000


TRUE = 0x1
FALSE = 0x0
PAGE_SIZE = 0x1000
Executive = 0x0
LOW_REALTIME_PRIORITY = 0x10
CLOCK_QUANTUM_DECREMENT = 0x3
WAIT_QUANTUM_DECREMENT = 0x1
MAXIMUM_PROCESSORS = 0x40
INITIAL_STALL_COUNT = 0x64
KI_EXCEPTION_ACCESS_VIOLATION = 0x10000004
DBG_STATUS_CONTROL_C = 0x1
MAXIMUM_IDTVECTOR = 0xff
PRIMARY_VECTOR_BASE = 0x30
RPL_MASK = 0x3
MODE_MASK = 0x1


ApType = 0x0
ApSize = 0x2
ApThread = 0x8
ApApcListEntry = 0x10
ApKernelRoutine = 0x20
ApRundownRoutine = 0x28
ApNormalRoutine = 0x30
ApNormalContext = 0x38
ApSystemArgument1 = 0x40
ApSystemArgument2 = 0x48
ApApcStateIndex = 0x50
ApApcMode = 0x51
ApInserted = 0x52
ApcObjectLength = 0x58


ArNormalRoutine = 0x0
ArNormalContext = 0x8
ArSystemArgument1 = 0x10
ArSystemArgument2 = 0x18
ApcRecordLength = 0x20


AsApcListHead = 0x0
AsProcess = 0x20
AsKernelApcInProgress = 0x28
AsKernelApcPending = 0x29
AsUserApcPending = 0x2a


CidUniqueProcess = 0x0
CidUniqueThread = 0x8


CsDebugInfo = 0x0
CsLockCount = 0x8
CsRecursionCount = 0xc
CsOwningThread = 0x10
CsLockSemaphore = 0x18
CsSpinCount = 0x20


CsType = 0x0
CsCreatorBackTraceIndex = 0x2
CsCriticalSection = 0x8
CsProcessLocksList = 0x10
CsEntryCount = 0x20
CsContentionCount = 0x24


DeDeviceListEntry = 0x0
DeSortKey = 0x10
DeInserted = 0x14
DeviceQueueEntryLength = 0x18


DpType = 0x0
DpImportance = 0x1
DpNumber = 0x2
DpDpcListEntry = 0x8
DpDeferredRoutine = 0x18
DpDeferredContext = 0x20
DpSystemArgument1 = 0x28
DpSystemArgument2 = 0x30
DpDpcData = 0x38
DpcObjectLength = 0x40


DvType = 0x0
DvSize = 0x2
DvDeviceListHead = 0x8
DvSpinLock = 0x18
DvBusy = 0x20
DeviceQueueObjectLength = 0x28


ErExceptionCode = 0x0
ErExceptionFlags = 0x4
ErExceptionRecord = 0x8
ErExceptionAddress = 0x10
ErNumberParameters = 0x18
ErExceptionInformation = 0x20
ExceptionRecordLength = 0x98
EXCEPTION_RECORD_LENGTH = 0x98


EpDebugPort = 0x140
EpWow64Process = 0x2a0
ExecutiveProcessObjectLength = 0x3d8


EtCid = 0x378
ExecutiveThreadObjectLength = 0x430


EvType = 0x0
EvSize = 0x2
EvSignalState = 0x4
EvWaitListHead = 0x8
EventObjectLength = 0x18


FbFiberData = 0x0
FbExceptionList = 0x8
FbStackBase = 0x10
FbStackLimit = 0x18
FbDeallocationStack = 0x20
FbFiberContext = 0x30
FbFlsData = 0x508
FbGuaranteedStackBytes = 0x500


FmCount = 0x0
FmOwner = 0x8
FmContention = 0x10
FmOldIrql = 0x30


GetSetCtxContextPtr = 0x80


InType = 0x0
InSize = 0x2
InInterruptListEntry = 0x8
InServiceRoutine = 0x18
InServiceContext = 0x20
InSpinLock = 0x28
InTickCount = 0x30
InActualLock = 0x38
InDispatchAddress = 0x40
InVector = 0x48
InIrql = 0x4c
InSynchronizeIrql = 0x4d
InFloatingSave = 0x4e
InConnected = 0x4f
InNumber = 0x50
InShareVector = 0x51
InMode = 0x54
InServiceCount = 0x58
InDispatchCount = 0x5c
InDispatchCode = 0x70
InterruptObjectLength = 0x80


IoStatus = 0x0
IoPointer = 0x0
IoInformation = 0x8


LqNext = 0x0
LqLock = 0x8
LOCK_QUEUE_HEADER_SIZE = 0x10


LqhLockQueue = 0x0
LqhNext = 0x0
LqhLock = 0x8
LqhOldIrql = 0x10


LiLowPart = 0x0
LiHighPart = 0x4


LpbKernelStack = 0x30
LpbPrcb = 0x38
LpbProcess = 0x40
LpbThread = 0x48


LsFlink = 0x0
LsBlink = 0x8


PeBeingDebugged = 0x2
PeProcessParameters = 0x20
PeKernelCallbackTable = 0x58
ProcessEnvironmentBlockLength = 0x358


PfType = 0x0
PfSize = 0x2
PfProfileListEntry = 0x8
PfProcess = 0x18
PfRangeBase = 0x20
PfRangeLimit = 0x28
PfBucketShift = 0x30
PfBuffer = 0x38
PfSegment = 0x40
PfAffinity = 0x48
PfSource = 0x50
PfStarted = 0x54
ProfileObjectLength = 0x58


PmLength = 0x0
PmZeroInit = 0x4
PmClientId = 0x8
PmProcess = 0x8
PmThread = 0x10
PmMessageId = 0x18
PmClientViewSize = 0x20
PortMessageLength = 0x28


PrType = 0x0
PrSize = 0x2
PrSignalState = 0x4
PrProfileListHead = 0x18
PrDirectoryTableBase = 0x28
PrThreadListHead = 0x70
PrAffinity = 0x88
PrReadyListHead = 0x50
PrSwapListEntry = 0x60
PrActiveProcessors = 0x40
PrProcessFlags = 0x90
PrBasePriority = 0x94
PrQuantumReset = 0x95
PrKernelTime = 0x48
PrUserTime = 0x4c
KernelProcessObjectLength = 0xb0


QuType = 0x0
QuSize = 0x2
QuSignalState = 0x4
QuEntryListHead = 0x18
QuCurrentCount = 0x28
QuMaximumCount = 0x2c
QuThreadListHead = 0x30
QueueObjectLength = 0x40


SdBase = 0x0
SdCount = 0x8
SdLimit = 0x10
SdNumber = 0x18
SdLength = 0x20


StrLength = 0x0
StrMaximumLength = 0x2
StrBuffer = 0x8


TeCmTeb = 0x0
TeStackBase = 0x8
TeStackLimit = 0x10
TeFiberData = 0x20
TeSelf = 0x30
TeEnvironmentPointer = 0x38
TeClientId = 0x40
TeActiveRpcHandle = 0x50
TeThreadLocalStoragePointer = 0x58
TePeb = 0x60
TeLastErrorValue = 0x68
TeCountOfOwnedCriticalSections = 0x6c
TeCsrClientThread = 0x70
TeWOW32Reserved = 0x100
TeExceptionCode = 0x2c0
TeActivationContextStackPointer = 0x2c8
TeGdiClientPID = 0x7f0
TeGdiClientTID = 0x7f4
TeGdiThreadLocalInfo = 0x7f8
TeglDispatchTable = 0x9f0
TeglReserved1 = 0x1138
TeglReserved2 = 0x1220
TeglSectionInfo = 0x1228
TeglSection = 0x1230
TeglTable = 0x1238
TeglCurrentRC = 0x1240
TeglContext = 0x1248
TeDeallocationStack = 0x1478
TeTlsSlots = 0x1480
TeVdm = 0x1690
TeInstrumentation = 0x16b8
TeGdiBatchCount = 0x1740
TeGuaranteedStackBytes = 0x1748
TeTlsExpansionSlots = 0x1780
TeFlsData = 0x17c8
ThreadEnvironmentBlockLength = 0x17d8


TfYear = 0x0
TfMonth = 0x2
TfDay = 0x4
TfHour = 0x6
TfMinute = 0x8
TfSecond = 0xa
TfMilliseconds = 0xc
TfWeekday = 0xe


ThType = 0x0
ThLock = 0x0
ThSize = 0x2
ThThreadControlFlags = 0x2
ThDebugActive = 0x3
ThSignalState = 0x4
ThInitialStack = 0x28
ThStackLimit = 0x30
ThStackBase = 0x258
ThThreadLock = 0x40
ThKernelStack = 0x38
ThAlerted = 0x96
ThThreadFlags = 0xf4
ThTrapFrame = 0x1c8
ThApcState = 0x48
ThPriority = 0x93
ThSwapBusy = 0x95
ThContextSwitches = 0x124
ThState = 0x154
ThProcess = 0x200
ThNpxState = 0x155
ThWaitIrql = 0x156
ThWaitMode = 0x157
ThTeb = 0xb0
ThTimer = 0xb8
ThWin32Thread = 0x250
ThWaitTime = 0x184
ThCombinedApcDisable = 0x1b4
ThKernelApcDisable = 0x1b4
ThSpecialApcDisable = 0x1b6
ThNextProcessor = 0x74
ThPreviousMode = 0x153
ThPriorityDecrement = 0x1f6
ThAdjustReason = 0x76
ThAdjustIncrement = 0x77
ThAffinity = 0x208
ThApcStateIndex = 0x1e4
ThIdealProcessor = 0x1e5
ThApcStatePointer = 0x210
ThSavedApcState = 0x220
ThWaitReason = 0x92
ThSaturation = 0x1f7
ThLegoData = 0x2a8
KernelThreadObjectLength = 0x328




TiType = 0x0
TiSize = 0x2
TiInserted = 0x3
TiSignalState = 0x4
TiDueTime = 0x18
TiTimerListEntry = 0x20
TiDpc = 0x30
TiPeriod = 0x38
TimerObjectLength = 0x40


TmLowTime = 0x0
TmHighTime = 0x4


TwFindAny = 0x0
TwFindIdeal = 0x8
TwFindLast = 0x4
TwIdleAny = 0xc
TwIdleCurrent = 0x10
TwIdleIdeal = 0x18
TwIdleLast = 0x14
TwPreemptAny = 0x1c
TwPreemptCurrent = 0x20
TwPreemptLast = 0x24
TwSwitchToIdle = 0x28


UsTickCountMultiplier = 0x4
UsInterruptTime = 0x8
UsSystemTime = 0x14
UsTimeZoneBias = 0x20
UsImageNumberLow = 0x2c
UsImageNumberHigh = 0x2e
UsNtSystemRoot = 0x30
UsMaxStackTraceDepth = 0x238
UsCryptoExponent = 0x23c
UsTimeZoneId = 0x240
UsLargePageMinimum = 0x244
UsReserved2 = 0x248
UsNtProductType = 0x264
UsProductTypeIsValid = 0x268
UsNtMajorVersion = 0x26c
UsNtMinorVersion = 0x270
UsProcessorFeatures = 0x274
UsReserved1 = 0x2b4
UsReserved3 = 0x2b8
UsTimeSlip = 0x2bc
UsAlternativeArchitecture = 0x2c0
UsSystemExpirationDate = 0x2c8
UsSuiteMask = 0x2d0
UsKdDebuggerEnabled = 0x2d4
UsActiveConsoleId = 0x2d8
UsDismountCount = 0x2dc
UsComPlusPackage = 0x2e0
UsLastSystemRITEventTickCount = 0x2e4
UsNumberOfPhysicalPages = 0x2e8
UsSafeBootMode = 0x2ec
UsTestRetInstruction = 0x2f8
UsSystemCall = 0x300
UsSystemCallReturn = 0x304
UsSystemCallPad = 0x308
UsTickCount = 0x320
UsTickCountQuad = 0x320
UsWow64SharedInformation = 0x334


WbWaitListEntry = 0x0
WbThread = 0x10
WbObject = 0x18
WbNextWaitBlock = 0x20
WbWaitKey = 0x28
WbWaitType = 0x2a
KSTART_FRAME_LENGTH = 0x30

EXTERN NtContinue:PROC
EXTERN LdrpInit:PROC
EXTERN ZwCallbackReturn:PROC
EXTERN RtlRaiseStatus:PROC
.data

Wow64PrepareForException:
    .quad 0

.code

PUBLIC LdrInitializeThunk
.PROC LdrInitializeThunk
    mov rbp, 0
    .setframe rbp, 0
    .endprolog


    mov rcx, r9
    jmp LdrpInit

.ENDP

PUBLIC KiUserApcDispatcher
.PROC KiUserApcDispatcher


    .PUSHFRAME
    .ALLOCSTACK CONTEXT_FRAME_LENGTH



    .SAVEREG rbx, CxRbx
    .SAVEREG rbp, CxRbp
    .SAVEREG rsi, CxRsi
    .SAVEREG rdi, CxRdi
    .SAVEREG r12, CxR12
    .SAVEREG r13, CxR13
    .SAVEREG r14, CxR14
    .SAVEREG r15, CxR15
    .SAVEXMM128 xmm6, CxXmm6
    .SAVEXMM128 xmm7, CxXmm7
    .SAVEXMM128 xmm8, CxXmm8
    .SAVEXMM128 xmm9, CxXmm9
    .SAVEXMM128 xmm10, CxXmm10
    .SAVEXMM128 xmm11, CxXmm11
    .SAVEXMM128 xmm12, CxXmm12
    .SAVEXMM128 xmm13, CxXmm13
    .SAVEXMM128 xmm14, CxXmm14
    .SAVEXMM128 xmm15, CxXmm15

    .endprolog


    mov rcx, [rsp + CONTEXT_P1Home]
    mov rdx, [rsp + CONTEXT_P2Home]
    mov r8, [rsp + CONTEXT_P3Home]
    lea r9, [rsp]
    call qword ptr [rsp + CONTEXT_P4Home]


    lea rcx, [rsp]
    mov dl, 1
    call NtContinue

    nop
    int 3
.ENDP


PUBLIC KiRaiseUserExceptionDispatcher
.PROC KiRaiseUserExceptionDispatcher
    .endprolog
    int 3

.ENDP

PUBLIC KiUserCallbackDispatcher
.PROC KiUserCallbackDispatcher



    .PUSHFRAME


    .ALLOCSTACK (6 * 8)
    .ENDPROLOG



    test rsp, 15
    jz AlignmentOk
    int 0x2C
AlignmentOk:



    mov rcx, [rsp + CkBuffer]
    mov edx, [rsp + CkLength]
    mov r8d, [rsp + CkApiNumber]


    mov rax, gs:[TePeb]
    mov r9, [rax + PeKernelCallbackTable]


    test r9, r9
    jz CallbackInvalid


    mov rax, [r9 + r8 * 8]


    test rax, rax
    jz CallbackInvalid
    cmp rax, -1
    je CallbackInvalid


    call rax


    xor ecx, ecx
    xor edx, edx
    mov r8d, eax
    call ZwCallbackReturn


    mov esi, eax


StatusRaise:
    mov ecx, esi
    call RtlRaiseStatus
    jmp StatusRaise

CallbackInvalid:

    xor ecx, ecx
    xor edx, edx
    mov r8d, 0xC0000002
    call ZwCallbackReturn
    jmp StatusRaise

.ENDP
EXTERN RtlDispatchException:PROC
EXTERN ZwContinue:PROC
EXTERN ZwRaiseException:PROC







EXTERN RtlRaiseException:PROC
PUBLIC KiUserExceptionDispatcher
.PROC KiUserExceptionDispatcher



    .PUSHFRAME


    .ALLOCSTACK 8 + EXCEPTION_RECORD_LENGTH + CONTEXT_FRAME_LENGTH
    .ENDPROLOG


    cld


    mov rax, qword ptr Wow64PrepareForException[rip]
    test rax, rax
    jz .NoWow64


    lea rcx, [rsp + CONTEXT_FRAME_LENGTH]
    lea rdx, [rsp]
    call rax

.NoWow64:


    lea rcx, [rsp + CONTEXT_FRAME_LENGTH]
    lea rdx, [rsp]
    call RtlDispatchException


    or al, al
    jz RaiseException


    lea rcx, [rsp]
    mov dl, 0
    call ZwContinue


    jmp Exit

RaiseException:







    lea rax, [rsp + CONTEXT_FRAME_LENGTH]
    test dword ptr [rax + ErExceptionFlags], EXCEPTION_NONCONTINUABLE
    jnz _DummyPseh_RaiseSecondChance






    cmp dword ptr [rax + ErExceptionCode], 0xC0000000
    jae _DummyPseh_RaiseSecondChance





    cmp dword ptr [rax + ErExceptionCode], 0x80000003
    jne _DummyPseh_NoBreakpointAdjust
    inc qword ptr [rsp + CxRip]
_DummyPseh_NoBreakpointAdjust:
    lea rcx, [rsp]
    mov dl, 0
    call ZwContinue
    jmp Exit

_DummyPseh_RaiseSecondChance:



    lea rcx, [rsp + CONTEXT_FRAME_LENGTH]
    lea rdx, [rsp]
    xor r8, r8
    call ZwRaiseException

Exit:
    lea rcx, [rsp + CONTEXT_FRAME_LENGTH]
    mov rdx, rax
    call KiUserExceptionDispatcherNested
    ret

.ENDP
.PROC KiUserExceptionDispatcherNested

    sub rsp, EXCEPTION_RECORD_LENGTH
    .ALLOCSTACK EXCEPTION_RECORD_LENGTH
    .ENDPROLOG


    mov dword ptr [rsp + ErNumberParameters], 0
    mov dword ptr [rsp + ErExceptionFlags], EXCEPTION_NONCONTINUABLE
    mov [rsp + ErExceptionRecord], rcx
    mov [rsp + ErExceptionCode], edx


    mov rcx, rsp
    call RtlRaiseException


    add rsp, EXCEPTION_RECORD_LENGTH
    ret
.ENDP

END
