//! Extern declarations for C global variables needed by Rust translations.
//!
//! Grouped by subsystem (KD64, CC, FSRTL, KE, MM, EX, PS).
//! Types use bindgen names (via `ddk::`) where available.

use crate::base::*;
use crate::ddk;
use crate::ke::KSPIN_LOCK;

// ============================================================================
// KD64 subsystem globals
// (from ntoskrnl/include/internal/kd64.h, sdk/include/reactos/windbgkd.h)
// ============================================================================

/// KD_BREAKPOINT_MAX as usize — for use in array lengths within this module.
/// The public constant (ULONG) lives in constants.rs.
const KD_BREAKPOINT_MAX_USIZE: usize = 32;

/// KD_BREAKPOINT_TYPE — architecture-dependent breakpoint opcode type.
/// arm64: ULONG (4 bytes, BRK instruction)
/// amd64/i386: UCHAR (1 byte, INT3 = 0xCC)
#[cfg(target_arch = "aarch64")]
pub type KD_BREAKPOINT_TYPE = ULONG;
#[cfg(target_arch = "x86_64")]
pub type KD_BREAKPOINT_TYPE = UCHAR;
#[cfg(target_arch = "x86")]
pub type KD_BREAKPOINT_TYPE = UCHAR;

/// KD_BREAKPOINT_SIZE — size of a breakpoint instruction.
#[cfg(target_arch = "aarch64")]
pub const KD_BREAKPOINT_SIZE: usize = 4;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub const KD_BREAKPOINT_SIZE: usize = 1;

/// Breakpoint status flags.
pub const KD_BREAKPOINT_ACTIVE: ULONG = 0x01;
pub const KD_BREAKPOINT_PENDING: ULONG = 0x02;
pub const KD_BREAKPOINT_SUSPENDED: ULONG = 0x04;
pub const KD_BREAKPOINT_EXPIRED: ULONG = 0x08;

/// KDP_MSG_BUFFER_SIZE — size of KdpMessageBuffer and KdpPathBuffer.
pub const KDP_MSG_BUFFER_SIZE: usize = 4096;

/// KD_DEFAULT_LOG_BUFFER_SIZE — default circular log buffer size.
pub const KD_DEFAULT_LOG_BUFFER_SIZE: usize = 0x1000;

/// KD_HIGHEST_USER_BREAKPOINT_ADDRESS
#[cfg(target_arch = "aarch64")]
pub const KD_HIGHEST_USER_BREAKPOINT_ADDRESS: ULONG_PTR = 0x0000FFFFFFFFFFFF;
#[cfg(target_arch = "x86_64")]
pub const KD_HIGHEST_USER_BREAKPOINT_ADDRESS: ULONG_PTR = 0x0000FFFFFFFFFFFF;
#[cfg(target_arch = "x86")]
pub const KD_HIGHEST_USER_BREAKPOINT_ADDRESS: ULONG_PTR = 0x60000000;

/// MAX_KD_COMPONENT_TABLE_ENTRIES = DPFLTR_ENDOFTABLE_ID + 1 = 156
pub const MAX_KD_COMPONENT_TABLE_ENTRIES: usize = 156;

/// KdPacketReceived — return value from KdReceivePacket.
pub const KdPacketReceived: u32 = 0;
pub const KdPacketTimedOut: u32 = 1;
pub const KdPacketNeedsResend: u32 = 2;

/// KCONTINUE_STATUS values (from ndk/ketypes.h).
pub const ContinueError: u32 = 0;
pub const ContinueSuccess: u32 = 1;
pub const ContinueProcessorReselected: u32 = 2;
pub const ContinueNextProcessor: u32 = 3;

/// KDSTATUS values.
pub const KdPacketType_KdPollBreakIn: u32 = 8;
pub const PACKET_TYPE_KD_STATE_CHANGE64: u32 = 7;
pub const PACKET_TYPE_KD_STATE_MANIPULATE: u32 = 2;

/// DbgKd API IDs (not in constants.rs).
pub const DbgKdExceptionStateChange: u32 = 0x00003030;

/// KD_PRINT_MAX_BYTES
pub const KD_PRINT_MAX_BYTES: u32 = 512;

/// DPFLTR_MASK — for debug filter state (WIN2000, DEFAULT).
pub const DPFLTR_MASK: u32 = 0x80000000;

/// KDBG_TAG — pool tag for KD allocations.
pub const KDBG_TAG: u32 = u32::from_le_bytes(*b"KDBg");

/// KD_OPTION values.
pub const _KD_OPTION_KD_OPTION_SET_BLOCK_ENABLE: u32 = 0;

/// IRQL_NOT_LESS_OR_EQUAL bugcheck code.
pub const IRQL_NOT_LESS_OR_EQUAL: u32 = 0x0000000A;

/// FLG_STOP_ON_EXCEPTION flag.
pub const FLG_STOP_ON_EXCEPTION: u32 = 0x00000001;

/// STATUS codes needed by kd64 (unique to globals — not in constants.rs).
pub const STATUS_WAKE_SYSTEM_DEBUGGER: NTSTATUS = 0x80000007_u32 as i32;

/// Isa bus type (INTERFACE_TYPE enum value).
pub const Isa: u32 = 1;

/// KERNEL_VERSION string constants — defined as C globals in the C build.
/// The Rust code should reference the C symbols or define them locally.
/// These are placeholders; the actual values come from the C build system.
pub const KERNEL_VERSION_STR: &[u8] = b"0.0\0";
pub const KERNEL_VERSION_BUILD_STR: &[u8] = b"0\0";
pub const KERNEL_VERSION_COMMIT_HASH: &[u8] = b"unknown\0";

extern "C" {
    // ------------------------------------------------------------------
    // KD state and version (kd64.h)
    // ------------------------------------------------------------------

    /// DBGKD_GET_VERSION64 KdVersionBlock
    pub static mut KdVersionBlock: ddk::_DBGKD_GET_VERSION64;

    /// KDDEBUGGER_DATA64 KdDebuggerDataBlock
    pub static mut KdDebuggerDataBlock: ddk::_KDDEBUGGER_DATA64;

    /// LIST_ENTRY KdpDebuggerDataListHead
    pub static mut KdpDebuggerDataListHead: ddk::_LIST_ENTRY;

    /// KSPIN_LOCK KdpDataSpinLock
    pub static mut KdpDataSpinLock: KSPIN_LOCK;

    /// LARGE_INTEGER KdPerformanceCounterRate
    pub static mut KdPerformanceCounterRate: ddk::LARGE_INTEGER;

    /// LARGE_INTEGER KdTimerStart
    pub static mut KdTimerStart: ddk::LARGE_INTEGER;

    /// LARGE_INTEGER KdTimerStop
    pub static mut KdTimerStop: ddk::LARGE_INTEGER;

    /// LARGE_INTEGER KdTimerDifference
    pub static mut KdTimerDifference: ddk::LARGE_INTEGER;

    /// ULONG KdDisableCount
    pub static mut KdDisableCount: ULONG;

    /// KD_CONTEXT KdpContext
    pub static mut KdpContext: ddk::_KD_CONTEXT;

    // ------------------------------------------------------------------
    // KD boolean flags (kd64.h)
    // ------------------------------------------------------------------

    /// BOOLEAN KdPitchDebugger — TRUE if debugger should be pitched (disabled).
    pub static mut KdPitchDebugger: BOOLEAN;

    /// BOOLEAN KdDebuggerNotPresent
    pub static mut KdDebuggerNotPresent: BOOLEAN;

    /// BOOLEAN KdBreakAfterSymbolLoad
    pub static mut KdBreakAfterSymbolLoad: BOOLEAN;

    /// BOOLEAN KdAutoEnableOnEvent
    pub static mut KdAutoEnableOnEvent: BOOLEAN;

    /// BOOLEAN KdBlockEnable
    pub static mut KdBlockEnable: BOOLEAN;

    /// BOOLEAN KdIgnoreUmExceptions
    pub static mut KdIgnoreUmExceptions: BOOLEAN;

    /// BOOLEAN KdPreviouslyEnabled
    pub static mut KdPreviouslyEnabled: BOOLEAN;

    /// BOOLEAN KdpDebuggerStructuresInitialized
    pub static mut KdpDebuggerStructuresInitialized: BOOLEAN;

    /// BOOLEAN KdEnteredDebugger
    pub static mut KdEnteredDebugger: BOOLEAN;

    /// BOOLEAN KdpPortLocked
    pub static mut KdpPortLocked: BOOLEAN;

    /// BOOLEAN KdpControlCPressed
    pub static mut KdpControlCPressed: BOOLEAN;

    /// BOOLEAN KdpContextSent
    pub static mut KdpContextSent: BOOLEAN;

    /// BOOLEAN BreakpointsSuspended
    pub static mut BreakpointsSuspended: BOOLEAN;

    // ------------------------------------------------------------------
    // KD DPC / timer / work item (kd64.h)
    // ------------------------------------------------------------------

    /// KDPC KdpTimeSlipDpc
    pub static mut KdpTimeSlipDpc: ddk::_KDPC;

    /// KTIMER KdpTimeSlipTimer
    pub static mut KdpTimeSlipTimer: ddk::_KTIMER;

    /// WORK_QUEUE_ITEM KdpTimeSlipWorkItem
    pub static mut KdpTimeSlipWorkItem: ddk::_WORK_QUEUE_ITEM;

    /// LONG KdpTimeSlipPending
    pub static mut KdpTimeSlipPending: LONG;

    /// PKEVENT KdpTimeSlipEvent
    pub static mut KdpTimeSlipEvent: ddk::PKEVENT;

    /// KSPIN_LOCK KdpTimeSlipEventLock
    pub static mut KdpTimeSlipEventLock: KSPIN_LOCK;

    /// KSPIN_LOCK KdpDebuggerLock
    pub static mut KdpDebuggerLock: KSPIN_LOCK;

    // ------------------------------------------------------------------
    // KD message buffers (kd64.h)
    // ------------------------------------------------------------------

    /// CHAR KdpMessageBuffer[KDP_MSG_BUFFER_SIZE]
    pub static mut KdpMessageBuffer: [i8; KDP_MSG_BUFFER_SIZE];

    /// CHAR KdpPathBuffer[KDP_MSG_BUFFER_SIZE]
    pub static mut KdpPathBuffer: [i8; KDP_MSG_BUFFER_SIZE];

    // ------------------------------------------------------------------
    // KD print / log (kd64.h)
    // ------------------------------------------------------------------

    /// CHAR KdPrintDefaultCircularBuffer[KD_DEFAULT_LOG_BUFFER_SIZE]
    pub static mut KdPrintDefaultCircularBuffer: [i8; KD_DEFAULT_LOG_BUFFER_SIZE];

    /// PCHAR KdPrintWritePointer
    pub static mut KdPrintWritePointer: PCHAR;

    /// ULONG KdPrintRolloverCount
    pub static mut KdPrintRolloverCount: ULONG;

    /// PCHAR KdPrintCircularBuffer
    pub static mut KdPrintCircularBuffer: PCHAR;

    /// ULONG KdPrintBufferSize
    pub static mut KdPrintBufferSize: ULONG;

    /// KSPIN_LOCK KdpPrintSpinLock
    pub static mut KdpPrintSpinLock: KSPIN_LOCK;

    // ------------------------------------------------------------------
    // KD breakpoints (kd64.h)
    // ------------------------------------------------------------------

    /// BREAKPOINT_ENTRY KdpBreakpointTable[KD_BREAKPOINT_MAX]
    pub static mut KdpBreakpointTable: [ddk::_BREAKPOINT_ENTRY; KD_BREAKPOINT_MAX_USIZE];

    /// KD_BREAKPOINT_TYPE KdpBreakpointInstruction
    /// Architecture-dependent: ULONG on arm64, UCHAR on amd64/i386.
    pub static mut KdpBreakpointInstruction: KD_BREAKPOINT_TYPE;

    /// BOOLEAN KdpOweBreakpoint
    pub static mut KdpOweBreakpoint: BOOLEAN;

    /// ULONG KdpNumInternalBreakpoints
    pub static mut KdpNumInternalBreakpoints: ULONG;

    // ------------------------------------------------------------------
    // KD trace data (kd64.h)
    // ------------------------------------------------------------------

    /// ULONG TraceDataBuffer[40]
    pub static mut TraceDataBuffer: [ULONG; 40];

    /// ULONG TraceDataBufferPosition
    pub static mut TraceDataBufferPosition: ULONG;

    // ------------------------------------------------------------------
    // KD component filter table (kd64.h)
    // ------------------------------------------------------------------

    /// ULONG KdComponentTableSize
    pub static mut KdComponentTableSize: ULONG;

    /// PULONG KdComponentTable[MAX_KD_COMPONENT_TABLE_ENTRIES]
    pub static mut KdComponentTable: [PULONG; MAX_KD_COMPONENT_TABLE_ENTRIES];

    // ------------------------------------------------------------------
    // KD debug filter masks (kd64.h) — only the commonly used ones
    // ------------------------------------------------------------------

    /// ULONG Kd_WIN2000_Mask
    pub static mut Kd_WIN2000_Mask: ULONG;

    /// ULONG Kd_DEFAULT_Mask
    pub static mut Kd_DEFAULT_Mask: ULONG;

    // ------------------------------------------------------------------
    // KD debug routine (kd64.h)
    // ------------------------------------------------------------------

    /// PKDEBUG_ROUTINE KiDebugRoutine — the active debugger dispatch function.
    pub static mut KiDebugRoutine: ddk::PKDEBUG_ROUTINE;

    // ------------------------------------------------------------------
    // KE globals needed by KD64 (ke.h)
    // ------------------------------------------------------------------

    /// PKPRCB KiProcessorBlock[MAXIMUM_PROCESSORS] — must be the REAL size, not the
    /// C-idiom `[; 0]` unsized-extern placeholder: Rust bounds-checks indexing against
    /// the DECLARED length, so `KiProcessorBlock[i]` through a `[; 0]` decl panics
    /// ("len is 0") on the first use. This is what hung i386+amd64 boots at ksecdd's
    /// NtQuerySystemInformation after the ex crate-split (ex-rust/sysinfo.rs indexes
    /// it 7×; pre-split it resolved to the in-crate sized def and was fine). Matches
    /// the #[no_mangle] def in ntoskrnl-rust ke/krnlinit.rs (same constant, per-arch).
    pub static mut KiProcessorBlock: [ddk::PKPRCB; crate::constants::MAXIMUM_PROCESSORS as usize];

    /// volatile CCHAR KeNumberProcessors
    pub static mut KeNumberProcessors: i8;

    /// ULONG KiFreezeFlag
    pub static mut KiFreezeFlag: ULONG;

    /// PVOID KeUserCallbackDispatcher
    pub static mut KeUserCallbackDispatcher: PVOID;

    /// PKDBG_PRESERVICEHOOK KeWin32PreServiceHook
    /// C: `typedef void (NTAPI *PKDBG_PRESERVICEHOOK)(ULONG, PULONG_PTR);`
    pub static mut KeWin32PreServiceHook: Option<unsafe extern "system" fn(ULONG, *mut ULONG_PTR)>;

    /// PKDBG_POSTSERVICEHOOK KeWin32PostServiceHook
    /// C: `typedef ULONG_PTR (NTAPI *PKDBG_POSTSERVICEHOOK)(ULONG, ULONG_PTR);`
    pub static mut KeWin32PostServiceHook: Option<
        unsafe extern "system" fn(ULONG, ULONG_PTR) -> ULONG_PTR,
    >;

    /// PLOADER_PARAMETER_BLOCK KeLoaderBlock
    pub static mut KeLoaderBlock: ddk::PLOADER_PARAMETER_BLOCK;

    // ------------------------------------------------------------------
    // EX globals needed by KD64 / CC / FSRTL
    // ------------------------------------------------------------------

    /// ULONG NtBuildNumber
    pub static mut NtBuildNumber: ULONG;

    /// ULONG NtGlobalFlag
    pub static mut NtGlobalFlag: ULONG;

    /// ULONG ExCriticalWorkerThreads
    pub static mut ExCriticalWorkerThreads: ULONG;

    // ------------------------------------------------------------------
    // MM globals needed by KD64 / CC
    // ------------------------------------------------------------------

    /// PFN_NUMBER MmHighestPhysicalPage — already in mm.rs but re-exporting
    /// for convenience; consumers should use ntkrnl_sys::mm::MmHighestPhysicalPage.

    /// PVOID MmHighestUserAddress
    pub static mut MmHighestUserAddress: PVOID;

    /// ULONG MmThrottleTop
    pub static mut MmThrottleTop: ULONG;

    /// ULONG MmThrottleBottom
    pub static mut MmThrottleBottom: ULONG;

    /// MMPFNLIST MmModifiedPageListHead
    pub static mut MmModifiedPageListHead: ddk::_MMPFNLIST;

    // ------------------------------------------------------------------
    // PS globals needed by KD64
    // ------------------------------------------------------------------

    /// ULONG_PTR PsNtosImageBase
    pub static mut PsNtosImageBase: ULONG_PTR;

    /// LIST_ENTRY PsLoadedModuleList
    pub static mut PsLoadedModuleList: ddk::_LIST_ENTRY;

    // ------------------------------------------------------------------
    // SE globals needed by KD64
    // ------------------------------------------------------------------

    /// LUID SeDebugPrivilege
    pub static mut SeDebugPrivilege: ddk::LUID;

    // ------------------------------------------------------------------
    // RTL globals
    // ------------------------------------------------------------------

    /// BOOLEAN NlsMbOemCodePageTag
    pub static mut NlsMbOemCodePageTag: BOOLEAN;

    // ------------------------------------------------------------------
    // SharedUserData — C macro, not a real global.
    // On each arch: `((KUSER_SHARED_DATA * const) KI_USER_SHARED_DATA)`
    // Rust code must use the address constant directly.
    // ------------------------------------------------------------------

    /// SharedUserData — provided as a symbol from kd64 C shims or cast from
    /// the fixed kernel VA. Declared as `*mut _KUSER_SHARED_DATA` for FFI.
    pub static mut SharedUserData: *mut ddk::_KUSER_SHARED_DATA;
}

// ============================================================================
// CC (Cache Manager) subsystem globals
// (from ntoskrnl/include/internal/cc.h, ntoskrnl/cc/*.c)
// ============================================================================

extern "C" {
    // ------------------------------------------------------------------
    // Global Cc lists and locks (cc.h)
    // ------------------------------------------------------------------

    /// LIST_ENTRY DirtyVacbListHead
    pub static mut DirtyVacbListHead: ddk::_LIST_ENTRY;

    /// ULONG CcDirtyPageThreshold
    pub static mut CcDirtyPageThreshold: ULONG;

    /// ULONG CcTotalDirtyPages
    pub static mut CcTotalDirtyPages: ULONG;

    /// LIST_ENTRY CcDeferredWrites
    pub static mut CcDeferredWrites: ddk::_LIST_ENTRY;

    /// KSPIN_LOCK CcDeferredWriteSpinLock
    pub static mut CcDeferredWriteSpinLock: KSPIN_LOCK;

    /// ULONG CcNumberWorkerThreads
    pub static mut CcNumberWorkerThreads: ULONG;

    /// LIST_ENTRY CcIdleWorkerThreadList
    pub static mut CcIdleWorkerThreadList: ddk::_LIST_ENTRY;

    /// LIST_ENTRY CcExpressWorkQueue
    pub static mut CcExpressWorkQueue: ddk::_LIST_ENTRY;

    /// LIST_ENTRY CcRegularWorkQueue
    pub static mut CcRegularWorkQueue: ddk::_LIST_ENTRY;

    /// LIST_ENTRY CcPostTickWorkQueue
    pub static mut CcPostTickWorkQueue: ddk::_LIST_ENTRY;

    /// NPAGED_LOOKASIDE_LIST CcTwilightLookasideList
    pub static mut CcTwilightLookasideList: ddk::_NPAGED_LOOKASIDE_LIST;

    /// LARGE_INTEGER CcIdleDelay
    pub static mut CcIdleDelay: ddk::LARGE_INTEGER;

    // ------------------------------------------------------------------
    // CC counters (cc.h)
    // ------------------------------------------------------------------

    /// ULONG CcLazyWritePages
    pub static mut CcLazyWritePages: ULONG;

    /// ULONG CcLazyWriteIos
    pub static mut CcLazyWriteIos: ULONG;

    /// ULONG CcMapDataWait
    pub static mut CcMapDataWait: ULONG;

    /// ULONG CcMapDataNoWait
    pub static mut CcMapDataNoWait: ULONG;

    /// ULONG CcFastReadNoWait (copy.c)
    pub static mut CcFastReadNoWait: ULONG;

    /// ULONG CcFastReadResourceMiss (copy.c)
    pub static mut CcFastReadResourceMiss: ULONG;

    // ------------------------------------------------------------------
    // CC prefetcher globals (cacheman.c)
    // ------------------------------------------------------------------

    /// PFSN_PREFETCHER_GLOBALS CcPfGlobals (declared as struct, not pointer)
    pub static mut CcPfGlobals: ddk::_PFSN_PREFETCHER_GLOBALS;

    /// MM_SYSTEMSIZE CcCapturedSystemSize
    pub static mut CcCapturedSystemSize: ddk::MM_SYSTEMSIZE;

    // ------------------------------------------------------------------
    // CC lazy writer (cc.h, lazywrite.c)
    // ------------------------------------------------------------------

    /// LAZY_WRITER LazyWriter
    pub static mut LazyWriter: ddk::_LAZY_WRITER;

    /// BOOLEAN CcQueueThrottle (lazywrite.c)
    pub static mut CcQueueThrottle: BOOLEAN;

    /// ULONG CcNumberActiveWorkerThreads (lazywrite.c)
    pub static mut CcNumberActiveWorkerThreads: ULONG;

    /// LARGE_INTEGER CcFirstDelay (lazywrite.c)
    pub static mut CcFirstDelay: ddk::LARGE_INTEGER;

    /// LARGE_INTEGER CcNoDelay (lazywrite.c)
    pub static mut CcNoDelay: ddk::LARGE_INTEGER;

    // ------------------------------------------------------------------
    // CC view globals (view.c) — some are static in C but needed by Rust
    // ------------------------------------------------------------------

    /// LIST_ENTRY VacbLruListHead (static in view.c, but Rust view.rs references it)
    pub static mut VacbLruListHead: ddk::_LIST_ENTRY;

    /// ULONG CcRosTraceLevel (cc.h)
    pub static mut CcRosTraceLevel: ULONG;

    // ------------------------------------------------------------------
    // CC pin globals (pin.c)
    // ------------------------------------------------------------------

    /// NPAGED_LOOKASIDE_LIST iBcbLookasideList
    pub static mut iBcbLookasideList: ddk::_NPAGED_LOOKASIDE_LIST;
}

// ============================================================================
// FSRTL (File System Runtime Library) subsystem globals
// (from ntoskrnl/include/internal/fsrtl.h, ntoskrnl/fsrtl/*.c)
// ============================================================================

/// FsRtlpDRD — anonymous struct for UNC redirector data.
/// C definition (from unc.c):
/// ```c
/// struct {
///     HANDLE MupHandle;
///     HANDLE NullHandle;
///     UNICODE_STRING RedirectorDeviceName;
///     BOOLEAN MailslotsSupported;
/// } FsRtlpDRD;
/// ```
#[repr(C)]
pub struct FsRtlpDrdStruct {
    pub MupHandle: PVOID,   // HANDLE
    pub NullHandle: PVOID,  // HANDLE
    pub RedirectorDeviceName: ddk::_UNICODE_STRING,
    pub MailslotsSupported: BOOLEAN,
}

/// MAXIMUM_PAIR_COUNT — max pair count per MCB.
pub const MAXIMUM_PAIR_COUNT: usize = 15;

/// FSRTLP_MAX_QUEUES — number of FsRtl worker queues (paging + non-paging).
pub const FSRTLP_MAX_QUEUES: usize = 2;

/// STACK_OVERFLOW_WORK_ITEM — internal struct for FsRtlPostStackOverflow.
/// C definition (from stackovf.c):
/// ```c
/// typedef struct _STACK_OVERFLOW_WORK_ITEM {
///     WORK_QUEUE_ITEM WorkItem;
///     PFSRTL_STACK_OVERFLOW_ROUTINE Routine;
///     PVOID Context;
///     PKEVENT Event;
/// } STACK_OVERFLOW_WORK_ITEM;
/// ```
#[repr(C)]
pub struct _STACK_OVERFLOW_WORK_ITEM {
    pub WorkItem: ddk::_WORK_QUEUE_ITEM,
    pub Routine: ddk::PFSRTL_STACK_OVERFLOW_ROUTINE,
    pub Context: PVOID,
    pub Event: ddk::PKEVENT,
}

extern "C" {
    // ------------------------------------------------------------------
    // FSRTL UNC globals (unc.c, fsrtlpc.c)
    // ------------------------------------------------------------------

    /// struct { HANDLE MupHandle; ... } FsRtlpDRD
    pub static mut FsRtlpDRD: FsRtlpDrdStruct;

    /// ULONG FsRtlpRedirs — count of registered redirectors
    pub static mut FsRtlpRedirs: ULONG;

    /// KSEMAPHORE FsRtlpUncSemaphore
    pub static mut FsRtlpUncSemaphore: ddk::_KSEMAPHORE;

    // ------------------------------------------------------------------
    // FSRTL worker queues (stackovf.c)
    // ------------------------------------------------------------------

    /// KQUEUE FsRtlWorkerQueues[FSRTLP_MAX_QUEUES]
    pub static mut FsRtlWorkerQueues: [ddk::_KQUEUE; FSRTLP_MAX_QUEUES];

    /// KEVENT StackOverflowFallbackSerialEvent
    pub static mut StackOverflowFallbackSerialEvent: ddk::_KEVENT;

    /// STACK_OVERFLOW_WORK_ITEM StackOverflowFallback
    pub static mut StackOverflowFallback: _STACK_OVERFLOW_WORK_ITEM;

    // ------------------------------------------------------------------
    // FSRTL paging IO resources (fsrtl.h, fsrtlpc.c)
    // ------------------------------------------------------------------

    /// PERESOURCE FsRtlPagingIoResources — pointer to array of ERESOURCE
    pub static mut FsRtlPagingIoResources: ddk::PERESOURCE;

    /// ULONG FsRtlPagingIoResourceSelector — round-robin selector
    pub static mut FsRtlPagingIoResourceSelector: ULONG;

    // ------------------------------------------------------------------
    // FSRTL file lock lookaside (fsrtl.h)
    // ------------------------------------------------------------------

    /// PAGED_LOOKASIDE_LIST FsRtlFileLockLookasideList
    pub static mut FsRtlFileLockLookasideList: ddk::_PAGED_LOOKASIDE_LIST;

    // ------------------------------------------------------------------
    // FSRTL MCB lookaside lists (largemcb.c)
    // ------------------------------------------------------------------

    /// PAGED_LOOKASIDE_LIST FsRtlFirstMappingLookasideList
    pub static mut FsRtlFirstMappingLookasideList: ddk::_PAGED_LOOKASIDE_LIST;

    /// NPAGED_LOOKASIDE_LIST FsRtlFastMutexLookasideList
    pub static mut FsRtlFastMutexLookasideList: ddk::_NPAGED_LOOKASIDE_LIST;

    // ------------------------------------------------------------------
    // FSRTL tunnel globals (tunnel.c)
    // ------------------------------------------------------------------

    /// ULONG TunnelMaxEntries — max tunnel cache entries (default 256)
    pub static mut TunnelMaxEntries: ULONG;

    /// ULONG TunnelMaxAge — max tunnel entry age in seconds (default 15)
    pub static mut TunnelMaxAge: ULONG;

    /// PAGED_LOOKASIDE_LIST TunnelLookasideList
    pub static mut TunnelLookasideList: ddk::_PAGED_LOOKASIDE_LIST;

    // ------------------------------------------------------------------
    // FSRTL ANSI character array (fsrtlpc.c)
    // ------------------------------------------------------------------

    /// const UCHAR * const FsRtlLegalAnsiCharacterArray
    pub static FsRtlLegalAnsiCharacterArray: *const UCHAR;
}
