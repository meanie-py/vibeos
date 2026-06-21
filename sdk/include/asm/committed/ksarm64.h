
/* Pointer size */
SizeofPointer = 0x8

/* Bug Check Codes */
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

/* Breakpoints */
BREAKPOINT_BREAK = 0x0
BREAKPOINT_PRINT = 0x1
BREAKPOINT_PROMPT = 0x2
BREAKPOINT_LOAD_SYMBOLS = 0x3
BREAKPOINT_UNLOAD_SYMBOLS = 0x4
BREAKPOINT_COMMAND_STRING = 0x5

/* Context Frame Flags */
CONTEXT_FULL = 0x400007
CONTEXT_CONTROL = 0x400001
CONTEXT_INTEGER = 0x400002
CONTEXT_FLOATING_POINT = 0x400004
CONTEXT_DEBUG_REGISTERS = 0x400008

/* Exception flags */
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

/* Exception types */
ExceptionContinueExecution = 0x0
ExceptionContinueSearch = 0x1
ExceptionNestedException = 0x2
ExceptionCollidedUnwind = 0x3

/* Fast Fail Constants */
FAST_FAIL_GUARD_ICALL_CHECK_FAILURE = 0xa

/* Interrupt object types */
InLevelSensitive = 0x0
InLatched = 0x1

/* IPI */
IPI_APC = 0x1
IPI_DPC = 0x2
IPI_FREEZE = 0x4
IPI_PACKET_READY = 0x6

/* IRQL */
PASSIVE_LEVEL = 0x0
APC_LEVEL = 0x1
DISPATCH_LEVEL = 0x2
IPI_LEVEL = 0xe
POWER_LEVEL = 0xe
PROFILE_LEVEL = 0xf
HIGH_LEVEL = 0xf
#ifndef CONFIG_SMP
SYNCH_LEVEL = 0x2
#else
SYNCH_LEVEL = 0xc
#endif

/* Lock Queue */
LOCK_QUEUE_WAIT = 0x1
LOCK_QUEUE_OWNER = 0x2
LockQueueDispatcherLock = 0x0

/* Process states */
ProcessInMemory = 0x0
ProcessOutOfMemory = 0x1
ProcessInTransition = 0x2

/* Processor mode */
KernelMode = 0x0
UserMode = 0x1

/* Service Table Constants */
NUMBER_SERVICE_TABLES = 0x2
SERVICE_NUMBER_MASK = 0xfff
SERVICE_TABLE_SHIFT = 0x7
SERVICE_TABLE_MASK = 0x20
SERVICE_TABLE_TEST = 0x20

/* Status codes */
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

/* TLS defines */
TLS_MINIMUM_AVAILABLE = 0x40
TLS_EXPANSION_SLOTS = 0x400

/* Thread states */
Initialized = 0x0
Ready = 0x1
Running = 0x2
Standby = 0x3
Terminated = 0x4
Waiting = 0x5

/* Wait type / reason */
WrExecutive = 0x7
WrMutex = 0x1d
WrDispatchInt = 0x1f
WrQuantumEnd = 0x1e
WrEventPair = 0xe
WaitAny = 0x1
WaitAll = 0x0

/* Stack sizes */
KERNEL_STACK_SIZE = 0x3000
KERNEL_LARGE_STACK_SIZE = 0xf000
KERNEL_LARGE_STACK_COMMIT = 0x3000
DOUBLE_FAULT_STACK_SIZE = 0x8000

/* Miscellaneous Definitions */
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

/* KAPC */
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

/* KAPC offsets (relative to NormalRoutine) */
ArNormalRoutine = 0x0
ArNormalContext = 0x8
ArSystemArgument1 = 0x10
ArSystemArgument2 = 0x18
ApcRecordLength = 0x20

/* KAPC_STATE */
AsApcListHead = 0x0
AsProcess = 0x20
AsKernelApcInProgress = 0x28
AsKernelApcPending = 0x29
AsUserApcPending = 0x2a

/* CLIENT_ID */
CidUniqueProcess = 0x0
CidUniqueThread = 0x8

/* RTL_CRITICAL_SECTION */
CsDebugInfo = 0x0
CsLockCount = 0x8
CsRecursionCount = 0xc
CsOwningThread = 0x10
CsLockSemaphore = 0x18
CsSpinCount = 0x20

/* RTL_CRITICAL_SECTION_DEBUG */
CsType = 0x0
CsCreatorBackTraceIndex = 0x2
CsCriticalSection = 0x8
CsProcessLocksList = 0x10
CsEntryCount = 0x20
CsContentionCount = 0x24

/* KDEVICE_QUEUE_ENTRY */
DeDeviceListEntry = 0x0
DeSortKey = 0x10
DeInserted = 0x14
DeviceQueueEntryLength = 0x18

/* KDPC */
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

/* KDEVICE_QUEUE */
DvType = 0x0
DvSize = 0x2
DvDeviceListHead = 0x8
DvSpinLock = 0x18
DvBusy = 0x20
DeviceQueueObjectLength = 0x28

/* EXCEPTION_RECORD */
ErExceptionCode = 0x0
ErExceptionFlags = 0x4
ErExceptionRecord = 0x8
ErExceptionAddress = 0x10
ErNumberParameters = 0x18
ErExceptionInformation = 0x20
ExceptionRecordLength = 0x98
EXCEPTION_RECORD_LENGTH = 0x98

/* EPROCESS */
EpDebugPort = 0x140
ExecutiveProcessObjectLength = 0x3e0

/* ETHREAD offsets */
EtCid = 0x340
ExecutiveThreadObjectLength = 0x3f8

/* KEVENT */
EvType = 0x0
EvSize = 0x2
EvSignalState = 0x4
EvWaitListHead = 0x8
EventObjectLength = 0x18

/* FIBER */
FbFiberData = 0x0
FbExceptionList = 0x8
FbStackBase = 0x10
FbStackLimit = 0x18
FbDeallocationStack = 0x20
FbFiberContext = 0x28
FbFlsData = 0x3c0
FbGuaranteedStackBytes = 0x3b8

/* FAST_MUTEX */
FmCount = 0x0
FmOwner = 0x8
FmContention = 0x10
FmOldIrql = 0x30

/* GETSETCONTEXT offsets */
GetSetCtxContextPtr = 0x78

/* KINTERRUPT */
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
InDispatchCode = 0x60
InterruptObjectLength = 0x208

/* KSPIN_LOCK_QUEUE */
LqNext = 0x0
LqLock = 0x8
LOCK_QUEUE_HEADER_SIZE = 0x10

/* KLOCK_QUEUE_HANDLE */
LqhLockQueue = 0x0
LqhNext = 0x0
LqhLock = 0x8
LqhOldIrql = 0x10

/* LARGE_INTEGER */
LiLowPart = 0x0
LiHighPart = 0x4

/* LOADER_PARAMETER_BLOCK (rel. to LoadOrderListHead) */
LpbKernelStack = 0x30
LpbPrcb = 0x38
LpbProcess = 0x40
LpbThread = 0x48

/* LIST_ENTRY */
LsFlink = 0x0
LsBlink = 0x8

/* PEB */
PeBeingDebugged = 0x2
PeProcessParameters = 0x20
PeKernelCallbackTable = 0x58
ProcessEnvironmentBlockLength = 0x358

/* KPROFILE */
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

/* PORT_MESSAGE */
PmLength = 0x0
PmZeroInit = 0x4
PmClientId = 0x8
PmProcess = 0x8
PmThread = 0x10
PmMessageId = 0x18
PmClientViewSize = 0x20
PortMessageLength = 0x28

/* KPROCESS */
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

/* KQUEUE */
QuType = 0x0
QuSize = 0x2
QuSignalState = 0x4
QuEntryListHead = 0x18
QuCurrentCount = 0x28
QuMaximumCount = 0x2c
QuThreadListHead = 0x30
QueueObjectLength = 0x40

/* KSERVICE_TABLE_DESCRIPTOR offsets */
SdBase = 0x0
SdCount = 0x8
SdLimit = 0x10
SdNumber = 0x18
SdLength = 0x20

/* STRING */
StrLength = 0x0
StrMaximumLength = 0x2
StrBuffer = 0x8

/* TEB */
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

/* TIME_FIELDS */
TfYear = 0x0
TfMonth = 0x2
TfDay = 0x4
TfHour = 0x6
TfMinute = 0x8
TfSecond = 0xa
TfMilliseconds = 0xc
TfWeekday = 0xe

/* KTHREAD */
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
KernelThreadObjectLength = 0x2f0

/* ETHREAD */

/* KTIMER */
TiType = 0x0
TiSize = 0x2
TiInserted = 0x3
TiSignalState = 0x4
TiDueTime = 0x18
TiTimerListEntry = 0x20
TiDpc = 0x30
TiPeriod = 0x38
TimerObjectLength = 0x40

/* TIME */
TmLowTime = 0x0
TmHighTime = 0x4

/* SYSTEM_CONTEXT_SWITCH_INFORMATION (relative to FindAny) */
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

/* KUSER_SHARED_DATA */
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

/* KWAIT_BLOCK offsets */
WbWaitListEntry = 0x0
WbThread = 0x10
WbObject = 0x18
WbNextWaitBlock = 0x20
WbWaitKey = 0x28
WbWaitType = 0x2a
