use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::globals::KD_BREAKPOINT_TYPE;
use ntkrnl_sys::ke::*;
// ke globals referenced across the crate boundary as link symbols
// (all #[no_mangle] in ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeTimeIncrement: ntkrnl_sys::base::ULONG;
    static mut KiProcessorBlock: [ntkrnl_sys::ke::PKPRCB; ntkrnl_sys::constants::MAXIMUM_PROCESSORS as usize];
    static mut KeLoaderBlock: ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
}

use ntkrnl_sys::ddk::{
    _CONTEXT, _EPROCESS, _ETHREAD, _KAPC_STATE, _KPRCB,
    BREAKPOINT_ENTRY, DBGKD_DEBUG_DATA_HEADER64, DBGKD_GET_VERSION64,
    DBGKD_SIMULATION_NONE, KD_CONTEXT, KPCR, KPROCESS, KPROCESSOR_MODE,
    KPROCESSOR_STATE, KTHREAD, KTIMER, LARGE_INTEGER, LIST_ENTRY, LUID,
    PCONTEXT, PEXCEPTION_RECORD,
    PKDEBUG_ROUTINE, PKEXCEPTION_FRAME, PKEVENT, PKTRAP_FRAME, ULONG64,
    WORK_QUEUE_ITEM, KCALLOUT_FRAME,
    _DEVICE_POWER_STATE_PowerDeviceD0, _DEVICE_POWER_STATE_PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2, _DEVICE_POWER_STATE_PowerDeviceD3,
    _FIRMWARE_REENTRY_HalRebootRoutine, _LOCK_OPERATION_IoReadAccess,
    _LOCK_OPERATION_IoWriteAccess, _WORK_QUEUE_TYPE_DelayedWorkQueue,
};

extern "system" {
    fn RtlpBreakWithStatusInstruction();
    static mut PsLoadedModuleList: LIST_ENTRY;
    static mut PsActiveProcessHead: LIST_ENTRY;
    static mut PspCidTable: PVOID;
    static mut ExpSystemResourcesList: LIST_ENTRY;
    static mut ExpPagedPoolDescriptor: PVOID;
    static mut ExpNumberOfPagedPools: ULONG;
    static mut KeBugcheckCallbackListHead: LIST_ENTRY;
    static mut KiBugCheckData: [ULONG_PTR; 5];
    static mut IopErrorLogListHead: LIST_ENTRY;
    static mut ObpRootDirectoryObject: PVOID;
    static mut ObpTypeObjectType: PVOID;
    static mut MmSystemCacheStart: PVOID;
    static mut MmSystemCacheEnd: PVOID;
    static mut MmSystemCacheWs: PVOID;
    static mut MmPfnDatabase: PVOID;
    static mut MmSystemPtesStart: PVOID;
    static mut MmSystemPtesEnd: PVOID;
    static mut MmSubsectionBase: PVOID;
    static mut MmNumberOfPagingFiles: ULONG;
    static mut MmLowestPhysicalPage: ULONG_PTR;
    static mut MmHighestPhysicalPage: ULONG_PTR;
    static mut MmNumberOfPhysicalPages: ULONG_PTR;
    static mut MmMaximumNonPagedPoolInBytes: ULONG_PTR;
    static mut MmNonPagedSystemStart: PVOID;
    static mut MmNonPagedPoolStart: PVOID;
    static mut MmNonPagedPoolEnd: PVOID;
    static mut MmPagedPoolStart: PVOID;
    static mut MmPagedPoolEnd: PVOID;
    static mut MmPagedPoolInfo: PVOID;
    static mut MmSizeOfPagedPoolInBytes: SIZE_T;
    static mut MmTotalCommitLimit: ULONG_PTR;
    static mut MmTotalCommittedPages: ULONG_PTR;
    static mut MmSharedCommit: ULONG_PTR;
    static mut MmDriverCommit: ULONG_PTR;
    static mut MmProcessCommit: ULONG_PTR;
    static mut MmPagedPoolCommit: ULONG_PTR;
    static mut MmZeroedPageListHead: LIST_ENTRY;
    static mut MmFreePageListHead: LIST_ENTRY;
    static mut MmStandbyPageListHead: LIST_ENTRY;
    static mut MmModifiedPageListHead: LIST_ENTRY;
    static mut MmModifiedNoWritePageListHead: LIST_ENTRY;
    static mut MmAvailablePages: ULONG_PTR;
    static mut MmResidentAvailablePages: ULONG_PTR;
    static mut PoolTrackTable: PVOID;
    static mut NonPagedPoolDescriptor: PVOID;
    static mut MmHighestUserAddress: PVOID;
    static mut MmSystemRangeStart: PVOID;
    static mut MmUserProbeAddress: PVOID;
    static mut MmLoadedUserImageList: LIST_ENTRY;
    static mut NtBuildLab: PVOID;
    static mut MmUnloadedDrivers: PVOID;
    static mut MmLastUnloadedDrivers: PVOID;
    static mut MmTriageActionTaken: ULONG;
    static mut MmSpecialPoolTag: ULONG;
    static mut KernelVerifier: PVOID;
    static mut MmVerifierData: PVOID;
    static mut MmAllocatedNonPagedPool: ULONG_PTR;
    static mut MmPeakCommitment: ULONG_PTR;
    static mut MmtotalCommitLimitMaximum: ULONG_PTR;
    static mut CmNtCSDVersion: ULONG;
    static mut MmPhysicalMemoryBlock: PVOID;
    static mut MmSessionBase: PVOID;
    static mut MmSessionSize: SIZE_T;
    static mut IopNumTriageDumpDataBlocks: ULONG;
    static mut IopTriageDumpDataBlocks: PVOID;
}

extern "system" {
    fn KiCallUserMode(OutputBuffer: *mut PVOID, OutputLength: PULONG) -> NTSTATUS;
}
extern "system" {
    fn KdpStub(
        TrapFrame: PKTRAP_FRAME,
        ExceptionFrame: PKEXCEPTION_FRAME,
        ExceptionRecord: PEXCEPTION_RECORD,
        ContextRecord: PCONTEXT,
        PreviousMode: KPROCESSOR_MODE,
        SecondChanceException: BOOLEAN,
    ) -> BOOLEAN;
}

const KD_BREAKPOINT_MAX: usize = 32;
const KDP_MSG_BUFFER_SIZE: usize = 4096;
const KD_DEFAULT_LOG_BUFFER_SIZE: usize = 4096;
const MAX_KD_COMPONENT_TABLE_ENTRIES: usize = 156;

#[cfg(target_arch = "x86")]
const KPCR_SELF_PCR_OFFSET: ULONG = 0x1C;
#[cfg(target_arch = "x86")]
const KPCR_CURRENT_PRCB_OFFSET: ULONG = 0x20;
#[cfg(target_arch = "x86")]
const KPCR_CONTAINED_PRCB_OFFSET: ULONG = 0x120;
#[cfg(target_arch = "x86")]
const KPCR_INITIAL_STACK_OFFSET: ULONG = 0;
#[cfg(target_arch = "x86")]
const KPCR_STACK_LIMIT_OFFSET: ULONG = 0;
#[cfg(target_arch = "x86")]
const KPRCB_PCR_PAGE_OFFSET: ULONG = 0;

#[cfg(target_arch = "x86_64")]
const KPCR_SELF_PCR_OFFSET: ULONG = 0x18;
#[cfg(target_arch = "x86_64")]
const KPCR_CURRENT_PRCB_OFFSET: ULONG = 0x20;
#[cfg(target_arch = "x86_64")]
const KPCR_CONTAINED_PRCB_OFFSET: ULONG = 0x180;
#[cfg(target_arch = "x86_64")]
const KPCR_INITIAL_STACK_OFFSET: ULONG = 0;
#[cfg(target_arch = "x86_64")]
const KPCR_STACK_LIMIT_OFFSET: ULONG = 0;
#[cfg(target_arch = "x86_64")]
const KPRCB_PCR_PAGE_OFFSET: ULONG = 0;

#[cfg(target_arch = "arm")]
const KPCR_SELF_PCR_OFFSET: ULONG = 0;
#[cfg(target_arch = "arm")]
const KPCR_CURRENT_PRCB_OFFSET: ULONG = 0x4;
#[cfg(target_arch = "arm")]
const KPCR_CONTAINED_PRCB_OFFSET: ULONG = 0;
#[cfg(target_arch = "arm")]
const KPCR_INITIAL_STACK_OFFSET: ULONG = 0x20;
#[cfg(target_arch = "arm")]
const KPCR_STACK_LIMIT_OFFSET: ULONG = 0x24;
#[cfg(target_arch = "arm")]
const KPRCB_PCR_PAGE_OFFSET: ULONG = 0x1C;

#[cfg(target_arch = "aarch64")]
const KPCR_SELF_PCR_OFFSET: ULONG = 0x18;
#[cfg(target_arch = "aarch64")]
const KPCR_CURRENT_PRCB_OFFSET: ULONG = 0x20;
#[cfg(target_arch = "aarch64")]
const KPCR_CONTAINED_PRCB_OFFSET: ULONG = 0x180;
#[cfg(target_arch = "aarch64")]
const KPCR_INITIAL_STACK_OFFSET: ULONG = 0;
#[cfg(target_arch = "aarch64")]
const KPCR_STACK_LIMIT_OFFSET: ULONG = 0;
#[cfg(target_arch = "aarch64")]
const KPRCB_PCR_PAGE_OFFSET: ULONG = 0;

#[cfg(target_arch = "x86")]
const KGDT_R0_CODE: u16 = 0x08;
#[cfg(target_arch = "x86")]
const KGDT_R0_DATA: u16 = 0x10;
#[cfg(target_arch = "x86")]
const KGDT_R0_PCR: u16 = 0x30;
#[cfg(target_arch = "x86")]
const KGDT_R3_CODE: u16 = 0x18;
#[cfg(target_arch = "x86")]
const KGDT_R3_DATA: u16 = 0x20;
#[cfg(target_arch = "x86")]
const KGDT_R3_TEB: u16 = 0x38;
#[cfg(target_arch = "x86")]
const KGDT_LDT: u16 = 0x48;
#[cfg(target_arch = "x86")]
const KGDT_TSS: u16 = 0x28;

#[cfg(target_arch = "x86_64")]
const KGDT64_R0_CODE: u16 = 0x10;
#[cfg(target_arch = "x86_64")]
const KGDT64_R3_DATA: u16 = 0x2B;
#[cfg(target_arch = "x86_64")]
const KGDT64_R3_CODE: u16 = 0x33;
#[cfg(target_arch = "x86_64")]
const KGDT64_SYS_TSS: u16 = 0x40;

const DBGKD_64BIT_PROTOCOL_VERSION2: u16 = 6;
#[cfg(target_arch = "x86_64")]
const CURRENT_KD_SECONDARY_VERSION: u8 = 2; // KD_SECONDARY_VERSION_AMD64_CONTEXT
#[cfg(not(target_arch = "x86_64"))]
const CURRENT_KD_SECONDARY_VERSION: u8 = 0; // KD_SECONDARY_VERSION_DEFAULT
const DBGKD_VERS_FLAG_DATA: u16 = 0x0002;
#[cfg(target_pointer_width = "64")]
const DBGKD_VERS_FLAG_PTR64: u16 = 0x0004;
const IMAGE_FILE_MACHINE_NATIVE: u16 = 0x014C;
const PACKET_TYPE_MAX: u8 = 12;

#[cfg(target_arch = "x86")]
const IMAGE_FILE_MACHINE_VALUE: u16 = 0x014C;
#[cfg(target_arch = "x86_64")]
const IMAGE_FILE_MACHINE_VALUE: u16 = 0x8664;
#[cfg(target_arch = "arm")]
const IMAGE_FILE_MACHINE_VALUE: u16 = 0x01C0;
#[cfg(target_arch = "aarch64")]
const IMAGE_FILE_MACHINE_VALUE: u16 = 0xAA64;

#[repr(C)]
struct KDDEBUGGER_DATA64 {
    header: [u64; 4],
    kthread_next_processor: ULONG,
    kthread_teb: ULONG,
    kthread_kernel_stack: ULONG,
    kthread_initial_stack: ULONG,
    kthread_apc_process: ULONG,
    kthread_state: ULONG,
    eprocess_peb: ULONG,
    eprocess_parent_cid: ULONG,
    eprocess_directory_table_base: ULONG,
}

const PAGE_SIZE: ULONG = 4096;

#[cfg(target_arch = "aarch64")]
const KD_BREAKPOINT_VALUE: KD_BREAKPOINT_TYPE = 0xD4200000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const KD_BREAKPOINT_VALUE: KD_BREAKPOINT_TYPE = 0xCC;

#[no_mangle]
pub static mut KdpContext: KD_CONTEXT = unsafe { core::mem::zeroed() };

// Registry-controlled value owned by kd64/kddata. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
// (KD_FILTER_CONTROL_VECTOR for debug-print masks lives further down with the
// Kd_*_Mask statics.)
#[no_mangle]
pub static KDDATA_TUNABLES_CV: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager", "Debugger Retries",
            core::ptr::addr_of_mut!(KdpContext.KdpDefaultRetries)),
    ] });

#[no_mangle]
pub static mut KdpPortLocked: BOOLEAN = 0;

#[no_mangle]
pub static mut KdpDebuggerLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KdpControlCPressed: BOOLEAN = 0;

#[no_mangle]
pub static mut KdpContextSent: BOOLEAN = 0;

// Initialized to KdpStub at runtime by KdInitSystem (abi_cast! can't run in const).
#[no_mangle]
pub static mut KiDebugRoutine: PKDEBUG_ROUTINE = None;

#[no_mangle]
pub static mut KdBreakAfterSymbolLoad: BOOLEAN = 0;

#[no_mangle]
pub static mut KdPitchDebugger: BOOLEAN = 0;

#[no_mangle]
pub static mut KdDebuggerNotPresent: BOOLEAN = 0;

#[no_mangle]
pub static mut KdDebuggerEnabled: BOOLEAN = 0;

#[no_mangle]
pub static mut KdAutoEnableOnEvent: BOOLEAN = 0;

#[no_mangle]
pub static mut KdBlockEnable: BOOLEAN = 0;

#[no_mangle]
pub static mut KdIgnoreUmExceptions: BOOLEAN = 0;

#[no_mangle]
pub static mut KdPreviouslyEnabled: BOOLEAN = 0;

#[no_mangle]
pub static mut KdpDebuggerStructuresInitialized: BOOLEAN = 0;

#[no_mangle]
pub static mut KdEnteredDebugger: BOOLEAN = 0;

#[no_mangle]
pub static mut KdDisableCount: ULONG = 0;

#[no_mangle]
pub static mut KdPerformanceCounterRate: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpBreakpointTable: [BREAKPOINT_ENTRY; KD_BREAKPOINT_MAX] = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpBreakpointInstruction: KD_BREAKPOINT_TYPE = KD_BREAKPOINT_VALUE;

#[no_mangle]
pub static mut KdpOweBreakpoint: BOOLEAN = 0;

#[no_mangle]
pub static mut BreakpointsSuspended: BOOLEAN = 0;

#[no_mangle]
pub static mut KdpNumInternalBreakpoints: ULONG = 0;

#[no_mangle]
pub static mut KdpCurrentSymbolStart: ULONG_PTR = 0;

#[no_mangle]
pub static mut KdpCurrentSymbolEnd: ULONG_PTR = 0;

#[no_mangle]
pub static mut TraceDataBuffer: [ULONG; 40] = [0; 40];

#[no_mangle]
pub static mut TraceDataBufferPosition: ULONG = 1;

#[no_mangle]
pub static mut KdpTimeSlipDpc: KDPC = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpTimeSlipTimer: KTIMER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpTimeSlipWorkItem: WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpTimeSlipPending: LONG = 1;

#[no_mangle]
pub static mut KdpTimeSlipEvent: PKEVENT = core::ptr::null_mut();

#[no_mangle]
pub static mut KdpTimeSlipEventLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KdTimerStop: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdTimerStart: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdTimerDifference: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpMessageBuffer: [i8; KDP_MSG_BUFFER_SIZE] = [0; KDP_MSG_BUFFER_SIZE];

#[no_mangle]
pub static mut KdpPathBuffer: [i8; KDP_MSG_BUFFER_SIZE] = [0; KDP_MSG_BUFFER_SIZE];

#[no_mangle]
pub static mut KdPrintDefaultCircularBuffer: [i8; KD_DEFAULT_LOG_BUFFER_SIZE] = [0; KD_DEFAULT_LOG_BUFFER_SIZE];

#[no_mangle]
pub static mut KdPrintWritePointer: *mut i8 = core::ptr::addr_of_mut!(KdPrintDefaultCircularBuffer) as *mut i8;

#[no_mangle]
pub static mut KdPrintRolloverCount: ULONG = 0;

#[no_mangle]
pub static mut KdPrintCircularBuffer: *mut i8 = core::ptr::addr_of_mut!(KdPrintDefaultCircularBuffer) as *mut i8;

#[no_mangle]
pub static mut KdPrintBufferSize: ULONG = KD_DEFAULT_LOG_BUFFER_SIZE as ULONG;

#[no_mangle]
pub static mut KdPrintBufferChanges: ULONG = 0;

#[no_mangle]
pub static mut KdpPrintSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut Kd_WIN2000_Mask: ULONG = 1;

#[no_mangle]
pub static mut Kd_SYSTEM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SMSS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SETUP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_NTFS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FSTUB_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CRASHDUMP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CDAUDIO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CDROM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CLASSPNP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DISK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_REDBOOK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_STORPROP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SCSIPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SCSIMINIPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CONFIG_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_I8042PRT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SERMOUSE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_LSERMOUS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_KBDHID_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MOUHID_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_KBDCLASS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MOUCLASS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TWOTRACK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WMILIB_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_ACPI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_AMLI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_HALIA64_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VIDEO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SVCHOST_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VIDEOPRT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TCPIP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DMSYNTH_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_NTOSPNP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FASTFAT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SAMSS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PNPMGR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_NETAPI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SCSERVER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SCCLIENT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SERIAL_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SERENUM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_UHCD_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_RPCPROXY_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_AUTOCHK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DCOMSS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_UNIMODEM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SIS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FLTMGR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WMICORE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_BURNENG_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IMAPI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SXS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FUSION_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IDLETASK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SOFTPCI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TAPE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MCHGR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IDEP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PCIIDE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FLOPPY_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FDC_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TERMSRV_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_W32TIME_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PREFETCHER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_RSFILTER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FCPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PCI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DMIO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DMCONFIG_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DMADMIN_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WSOCKTRANSPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VSS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PNPMEM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PROCESSOR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DMSERVER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_INFINIBAND_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVDRIVER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVVIDEO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVAUDIO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVNETWORK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVSTREAMING_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IHVBUS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_HPS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_RTLTHREADPOOL_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_LDR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TCPIP6_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_ISAPNP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SHPC_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_STORPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_STORMINIPORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PRINTSPOOLER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VSSDYNDISK_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VERIFIER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDSBAS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDSDYN_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDSDYNDR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDSLDR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VDSUTIL_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DFRGIFC_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DEFAULT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DFSC_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WOW64_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_ALPC_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WDI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PERFLIB_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_KTM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_IOSTRESS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_HEAP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WHEA_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_USERGDI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MMCSS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TPM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_THREADORDER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_ENVIRON_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_EMS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WDT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FVEVOL_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_NDIS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_NVCTRACE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_LUAFV_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_APPCOMPAT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_USBSTOR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SBP2PORT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_COVERAGE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CACHEMGR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MOUNTMGR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CFR_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_TXF_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_KSECDD_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FLTREGRESS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MPIO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_MSDSM_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_UDFS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_PSHED_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_STORVSP_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_LSASS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SSPICLI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CNG_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_EXFAT_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FILETRACE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_XSAVE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_SE_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_DRIVEEXTENDER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_POWER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CRASHDUMPXHCI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_GPIO_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_REFS_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_WER_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_CAPIMG_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_VPCI_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_STORAGECLASSMEMORY_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_FSLIB_Mask: ULONG = 0;

#[no_mangle]
pub static mut Kd_ENDOFTABLE_Mask: ULONG = 0;

// ============================================================================
// CmControlVector entries for the KD debug-print filter masks.
// Each entry maps a registry key under "Session Manager\Debug Print Filter"
// to one of the Kd_*_Mask globals above. Concatenated into CmControlVector
// by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
// ============================================================================

// Local sugar — saves repeating the long key path 156 times. Local-only
// (not exported) since kd64 is the sole producer of these entries.
macro_rules! kd_filter {
    ($name:literal, $sym:ident) => {
        ntkrnl_sys::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: ntkrnl_sys::w!("Session Manager\\Debug Print Filter").as_ptr()
                as ntkrnl_sys::ddk::PWCHAR,
            ValueName: ntkrnl_sys::w!($name).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            Buffer: core::ptr::addr_of_mut!($sym) as ntkrnl_sys::ddk::PVOID,
            BufferLength: core::ptr::null_mut(),
            Type: core::ptr::null_mut(),
        }
    };
}

#[no_mangle]
pub static KD_FILTER_CONTROL_VECTOR: ntkrnl_sys::SubVector<156> =
    ntkrnl_sys::SubVector({ [
    kd_filter!("WIN2000", Kd_WIN2000_Mask),
    kd_filter!("SYSTEM", Kd_SYSTEM_Mask),
    kd_filter!("SMSS", Kd_SMSS_Mask),
    kd_filter!("SETUP", Kd_SETUP_Mask),
    kd_filter!("NTFS", Kd_NTFS_Mask),
    kd_filter!("FSTUB", Kd_FSTUB_Mask),
    kd_filter!("CRASHDUMP", Kd_CRASHDUMP_Mask),
    kd_filter!("CDAUDIO", Kd_CDAUDIO_Mask),
    kd_filter!("CDROM", Kd_CDROM_Mask),
    kd_filter!("CLASSPNP", Kd_CLASSPNP_Mask),
    kd_filter!("DISK", Kd_DISK_Mask),
    kd_filter!("REDBOOK", Kd_REDBOOK_Mask),
    kd_filter!("STORPROP", Kd_STORPROP_Mask),
    kd_filter!("SCSIPORT", Kd_SCSIPORT_Mask),
    kd_filter!("SCSIMINIPORT", Kd_SCSIMINIPORT_Mask),
    kd_filter!("CONFIG", Kd_CONFIG_Mask),
    kd_filter!("I8042PRT", Kd_I8042PRT_Mask),
    kd_filter!("SERMOUSE", Kd_SERMOUSE_Mask),
    kd_filter!("LSERMOUS", Kd_LSERMOUS_Mask),
    kd_filter!("KBDHID", Kd_KBDHID_Mask),
    kd_filter!("MOUHID", Kd_MOUHID_Mask),
    kd_filter!("KBDCLASS", Kd_KBDCLASS_Mask),
    kd_filter!("MOUCLASS", Kd_MOUCLASS_Mask),
    kd_filter!("TWOTRACK", Kd_TWOTRACK_Mask),
    kd_filter!("WMILIB", Kd_WMILIB_Mask),
    kd_filter!("ACPI", Kd_ACPI_Mask),
    kd_filter!("AMLI", Kd_AMLI_Mask),
    kd_filter!("HALIA64", Kd_HALIA64_Mask),
    kd_filter!("VIDEO", Kd_VIDEO_Mask),
    kd_filter!("SVCHOST", Kd_SVCHOST_Mask),
    kd_filter!("VIDEOPRT", Kd_VIDEOPRT_Mask),
    kd_filter!("TCPIP", Kd_TCPIP_Mask),
    kd_filter!("DMSYNTH", Kd_DMSYNTH_Mask),
    kd_filter!("NTOSPNP", Kd_NTOSPNP_Mask),
    kd_filter!("FASTFAT", Kd_FASTFAT_Mask),
    kd_filter!("SAMSS", Kd_SAMSS_Mask),
    kd_filter!("PNPMGR", Kd_PNPMGR_Mask),
    kd_filter!("NETAPI", Kd_NETAPI_Mask),
    kd_filter!("SCSERVER", Kd_SCSERVER_Mask),
    kd_filter!("SCCLIENT", Kd_SCCLIENT_Mask),
    kd_filter!("SERIAL", Kd_SERIAL_Mask),
    kd_filter!("SERENUM", Kd_SERENUM_Mask),
    kd_filter!("UHCD", Kd_UHCD_Mask),
    kd_filter!("RPCPROXY", Kd_RPCPROXY_Mask),
    kd_filter!("AUTOCHK", Kd_AUTOCHK_Mask),
    kd_filter!("DCOMSS", Kd_DCOMSS_Mask),
    kd_filter!("UNIMODEM", Kd_UNIMODEM_Mask),
    kd_filter!("SIS", Kd_SIS_Mask),
    kd_filter!("FLTMGR", Kd_FLTMGR_Mask),
    kd_filter!("WMICORE", Kd_WMICORE_Mask),
    kd_filter!("BURNENG", Kd_BURNENG_Mask),
    kd_filter!("IMAPI", Kd_IMAPI_Mask),
    kd_filter!("SXS", Kd_SXS_Mask),
    kd_filter!("FUSION", Kd_FUSION_Mask),
    kd_filter!("IDLETASK", Kd_IDLETASK_Mask),
    kd_filter!("SOFTPCI", Kd_SOFTPCI_Mask),
    kd_filter!("TAPE", Kd_TAPE_Mask),
    kd_filter!("MCHGR", Kd_MCHGR_Mask),
    kd_filter!("IDEP", Kd_IDEP_Mask),
    kd_filter!("PCIIDE", Kd_PCIIDE_Mask),
    kd_filter!("FLOPPY", Kd_FLOPPY_Mask),
    kd_filter!("FDC", Kd_FDC_Mask),
    kd_filter!("TERMSRV", Kd_TERMSRV_Mask),
    kd_filter!("W32TIME", Kd_W32TIME_Mask),
    kd_filter!("PREFETCHER", Kd_PREFETCHER_Mask),
    kd_filter!("RSFILTER", Kd_RSFILTER_Mask),
    kd_filter!("FCPORT", Kd_FCPORT_Mask),
    kd_filter!("PCI", Kd_PCI_Mask),
    kd_filter!("DMIO", Kd_DMIO_Mask),
    kd_filter!("DMCONFIG", Kd_DMCONFIG_Mask),
    kd_filter!("DMADMIN", Kd_DMADMIN_Mask),
    kd_filter!("WSOCKTRANSPORT", Kd_WSOCKTRANSPORT_Mask),
    kd_filter!("VSS", Kd_VSS_Mask),
    kd_filter!("PNPMEM", Kd_PNPMEM_Mask),
    kd_filter!("PROCESSOR", Kd_PROCESSOR_Mask),
    kd_filter!("DMSERVER", Kd_DMSERVER_Mask),
    kd_filter!("SR", Kd_SR_Mask),
    kd_filter!("INFINIBAND", Kd_INFINIBAND_Mask),
    kd_filter!("IHVDRIVER", Kd_IHVDRIVER_Mask),
    kd_filter!("IHVVIDEO", Kd_IHVVIDEO_Mask),
    kd_filter!("IHVAUDIO", Kd_IHVAUDIO_Mask),
    kd_filter!("IHVNETWORK", Kd_IHVNETWORK_Mask),
    kd_filter!("IHVSTREAMING", Kd_IHVSTREAMING_Mask),
    kd_filter!("IHVBUS", Kd_IHVBUS_Mask),
    kd_filter!("HPS", Kd_HPS_Mask),
    kd_filter!("RTLTHREADPOOL", Kd_RTLTHREADPOOL_Mask),
    kd_filter!("LDR", Kd_LDR_Mask),
    kd_filter!("TCPIP6", Kd_TCPIP6_Mask),
    kd_filter!("ISAPNP", Kd_ISAPNP_Mask),
    kd_filter!("SHPC", Kd_SHPC_Mask),
    kd_filter!("STORPORT", Kd_STORPORT_Mask),
    kd_filter!("STORMINIPORT", Kd_STORMINIPORT_Mask),
    kd_filter!("PRINTSPOOLER", Kd_PRINTSPOOLER_Mask),
    kd_filter!("VSSDYNDISK", Kd_VSSDYNDISK_Mask),
    kd_filter!("VERIFIER", Kd_VERIFIER_Mask),
    kd_filter!("VDS", Kd_VDS_Mask),
    kd_filter!("VDSBAS", Kd_VDSBAS_Mask),
    kd_filter!("VDSDYN", Kd_VDSDYN_Mask),
    kd_filter!("VDSDYNDR", Kd_VDSDYNDR_Mask),
    kd_filter!("VDSLDR", Kd_VDSLDR_Mask),
    kd_filter!("VDSUTIL", Kd_VDSUTIL_Mask),
    kd_filter!("DFRGIFC", Kd_DFRGIFC_Mask),
    kd_filter!("DEFAULT", Kd_DEFAULT_Mask),
    kd_filter!("MM", Kd_MM_Mask),
    kd_filter!("DFSC", Kd_DFSC_Mask),
    kd_filter!("WOW64", Kd_WOW64_Mask),
    kd_filter!("ALPC", Kd_ALPC_Mask),
    kd_filter!("WDI", Kd_WDI_Mask),
    kd_filter!("PERFLIB", Kd_PERFLIB_Mask),
    kd_filter!("KTM", Kd_KTM_Mask),
    kd_filter!("IOSTRESS", Kd_IOSTRESS_Mask),
    kd_filter!("HEAP", Kd_HEAP_Mask),
    kd_filter!("WHEA", Kd_WHEA_Mask),
    kd_filter!("USERGDI", Kd_USERGDI_Mask),
    kd_filter!("MMCSS", Kd_MMCSS_Mask),
    kd_filter!("TPM", Kd_TPM_Mask),
    kd_filter!("THREADORDER", Kd_THREADORDER_Mask),
    kd_filter!("ENVIRON", Kd_ENVIRON_Mask),
    kd_filter!("EMS", Kd_EMS_Mask),
    kd_filter!("WDT", Kd_WDT_Mask),
    kd_filter!("FVEVOL", Kd_FVEVOL_Mask),
    kd_filter!("NDIS", Kd_NDIS_Mask),
    kd_filter!("NVCTRACE", Kd_NVCTRACE_Mask),
    kd_filter!("LUAFV", Kd_LUAFV_Mask),
    kd_filter!("APPCOMPAT", Kd_APPCOMPAT_Mask),
    kd_filter!("USBSTOR", Kd_USBSTOR_Mask),
    kd_filter!("SBP2PORT", Kd_SBP2PORT_Mask),
    kd_filter!("COVERAGE", Kd_COVERAGE_Mask),
    kd_filter!("CACHEMGR", Kd_CACHEMGR_Mask),
    kd_filter!("MOUNTMGR", Kd_MOUNTMGR_Mask),
    kd_filter!("CFR", Kd_CFR_Mask),
    kd_filter!("TXF", Kd_TXF_Mask),
    kd_filter!("KSECDD", Kd_KSECDD_Mask),
    kd_filter!("FLTREGRESS", Kd_FLTREGRESS_Mask),
    kd_filter!("MPIO", Kd_MPIO_Mask),
    kd_filter!("MSDSM", Kd_MSDSM_Mask),
    kd_filter!("UDFS", Kd_UDFS_Mask),
    kd_filter!("PSHED", Kd_PSHED_Mask),
    kd_filter!("STORVSP", Kd_STORVSP_Mask),
    kd_filter!("LSASS", Kd_LSASS_Mask),
    kd_filter!("SSPICLI", Kd_SSPICLI_Mask),
    kd_filter!("CNG", Kd_CNG_Mask),
    kd_filter!("EXFAT", Kd_EXFAT_Mask),
    kd_filter!("FILETRACE", Kd_FILETRACE_Mask),
    kd_filter!("XSAVE", Kd_XSAVE_Mask),
    kd_filter!("SE", Kd_SE_Mask),
    kd_filter!("DRIVEEXTENDER", Kd_DRIVEEXTENDER_Mask),
    kd_filter!("POWER", Kd_POWER_Mask),
    kd_filter!("CRASHDUMPXHCI", Kd_CRASHDUMPXHCI_Mask),
    kd_filter!("GPIO", Kd_GPIO_Mask),
    kd_filter!("REFS", Kd_REFS_Mask),
    kd_filter!("WER", Kd_WER_Mask),
    kd_filter!("CAPIMG", Kd_CAPIMG_Mask),
    kd_filter!("VPCI", Kd_VPCI_Mask),
    kd_filter!("STORAGECLASSMEMORY", Kd_STORAGECLASSMEMORY_Mask),
    kd_filter!("FSLIB", Kd_FSLIB_Mask),
] });

#[no_mangle]
pub static mut KdComponentTable: [*mut ULONG; MAX_KD_COMPONENT_TABLE_ENTRIES] = {
    [
        core::ptr::addr_of_mut!(Kd_SYSTEM_Mask),
        core::ptr::addr_of_mut!(Kd_SMSS_Mask),
        core::ptr::addr_of_mut!(Kd_SETUP_Mask),
        core::ptr::addr_of_mut!(Kd_NTFS_Mask),
        core::ptr::addr_of_mut!(Kd_FSTUB_Mask),
        core::ptr::addr_of_mut!(Kd_CRASHDUMP_Mask),
        core::ptr::addr_of_mut!(Kd_CDAUDIO_Mask),
        core::ptr::addr_of_mut!(Kd_CDROM_Mask),
        core::ptr::addr_of_mut!(Kd_CLASSPNP_Mask),
        core::ptr::addr_of_mut!(Kd_DISK_Mask),
        core::ptr::addr_of_mut!(Kd_REDBOOK_Mask),
        core::ptr::addr_of_mut!(Kd_STORPROP_Mask),
        core::ptr::addr_of_mut!(Kd_SCSIPORT_Mask),
        core::ptr::addr_of_mut!(Kd_SCSIMINIPORT_Mask),
        core::ptr::addr_of_mut!(Kd_CONFIG_Mask),
        core::ptr::addr_of_mut!(Kd_I8042PRT_Mask),
        core::ptr::addr_of_mut!(Kd_SERMOUSE_Mask),
        core::ptr::addr_of_mut!(Kd_LSERMOUS_Mask),
        core::ptr::addr_of_mut!(Kd_KBDHID_Mask),
        core::ptr::addr_of_mut!(Kd_MOUHID_Mask),
        core::ptr::addr_of_mut!(Kd_KBDCLASS_Mask),
        core::ptr::addr_of_mut!(Kd_MOUCLASS_Mask),
        core::ptr::addr_of_mut!(Kd_TWOTRACK_Mask),
        core::ptr::addr_of_mut!(Kd_WMILIB_Mask),
        core::ptr::addr_of_mut!(Kd_ACPI_Mask),
        core::ptr::addr_of_mut!(Kd_AMLI_Mask),
        core::ptr::addr_of_mut!(Kd_HALIA64_Mask),
        core::ptr::addr_of_mut!(Kd_VIDEO_Mask),
        core::ptr::addr_of_mut!(Kd_SVCHOST_Mask),
        core::ptr::addr_of_mut!(Kd_VIDEOPRT_Mask),
        core::ptr::addr_of_mut!(Kd_TCPIP_Mask),
        core::ptr::addr_of_mut!(Kd_DMSYNTH_Mask),
        core::ptr::addr_of_mut!(Kd_NTOSPNP_Mask),
        core::ptr::addr_of_mut!(Kd_FASTFAT_Mask),
        core::ptr::addr_of_mut!(Kd_SAMSS_Mask),
        core::ptr::addr_of_mut!(Kd_PNPMGR_Mask),
        core::ptr::addr_of_mut!(Kd_NETAPI_Mask),
        core::ptr::addr_of_mut!(Kd_SCSERVER_Mask),
        core::ptr::addr_of_mut!(Kd_SCCLIENT_Mask),
        core::ptr::addr_of_mut!(Kd_SERIAL_Mask),
        core::ptr::addr_of_mut!(Kd_SERENUM_Mask),
        core::ptr::addr_of_mut!(Kd_UHCD_Mask),
        core::ptr::addr_of_mut!(Kd_RPCPROXY_Mask),
        core::ptr::addr_of_mut!(Kd_AUTOCHK_Mask),
        core::ptr::addr_of_mut!(Kd_DCOMSS_Mask),
        core::ptr::addr_of_mut!(Kd_UNIMODEM_Mask),
        core::ptr::addr_of_mut!(Kd_SIS_Mask),
        core::ptr::addr_of_mut!(Kd_FLTMGR_Mask),
        core::ptr::addr_of_mut!(Kd_WMICORE_Mask),
        core::ptr::addr_of_mut!(Kd_BURNENG_Mask),
        core::ptr::addr_of_mut!(Kd_IMAPI_Mask),
        core::ptr::addr_of_mut!(Kd_SXS_Mask),
        core::ptr::addr_of_mut!(Kd_FUSION_Mask),
        core::ptr::addr_of_mut!(Kd_IDLETASK_Mask),
        core::ptr::addr_of_mut!(Kd_SOFTPCI_Mask),
        core::ptr::addr_of_mut!(Kd_TAPE_Mask),
        core::ptr::addr_of_mut!(Kd_MCHGR_Mask),
        core::ptr::addr_of_mut!(Kd_IDEP_Mask),
        core::ptr::addr_of_mut!(Kd_PCIIDE_Mask),
        core::ptr::addr_of_mut!(Kd_FLOPPY_Mask),
        core::ptr::addr_of_mut!(Kd_FDC_Mask),
        core::ptr::addr_of_mut!(Kd_TERMSRV_Mask),
        core::ptr::addr_of_mut!(Kd_W32TIME_Mask),
        core::ptr::addr_of_mut!(Kd_PREFETCHER_Mask),
        core::ptr::addr_of_mut!(Kd_RSFILTER_Mask),
        core::ptr::addr_of_mut!(Kd_FCPORT_Mask),
        core::ptr::addr_of_mut!(Kd_PCI_Mask),
        core::ptr::addr_of_mut!(Kd_DMIO_Mask),
        core::ptr::addr_of_mut!(Kd_DMCONFIG_Mask),
        core::ptr::addr_of_mut!(Kd_DMADMIN_Mask),
        core::ptr::addr_of_mut!(Kd_WSOCKTRANSPORT_Mask),
        core::ptr::addr_of_mut!(Kd_VSS_Mask),
        core::ptr::addr_of_mut!(Kd_PNPMEM_Mask),
        core::ptr::addr_of_mut!(Kd_PROCESSOR_Mask),
        core::ptr::addr_of_mut!(Kd_DMSERVER_Mask),
        core::ptr::addr_of_mut!(Kd_SR_Mask),
        core::ptr::addr_of_mut!(Kd_INFINIBAND_Mask),
        core::ptr::addr_of_mut!(Kd_IHVDRIVER_Mask),
        core::ptr::addr_of_mut!(Kd_IHVVIDEO_Mask),
        core::ptr::addr_of_mut!(Kd_IHVAUDIO_Mask),
        core::ptr::addr_of_mut!(Kd_IHVNETWORK_Mask),
        core::ptr::addr_of_mut!(Kd_IHVSTREAMING_Mask),
        core::ptr::addr_of_mut!(Kd_IHVBUS_Mask),
        core::ptr::addr_of_mut!(Kd_HPS_Mask),
        core::ptr::addr_of_mut!(Kd_RTLTHREADPOOL_Mask),
        core::ptr::addr_of_mut!(Kd_LDR_Mask),
        core::ptr::addr_of_mut!(Kd_TCPIP6_Mask),
        core::ptr::addr_of_mut!(Kd_ISAPNP_Mask),
        core::ptr::addr_of_mut!(Kd_SHPC_Mask),
        core::ptr::addr_of_mut!(Kd_STORPORT_Mask),
        core::ptr::addr_of_mut!(Kd_STORMINIPORT_Mask),
        core::ptr::addr_of_mut!(Kd_PRINTSPOOLER_Mask),
        core::ptr::addr_of_mut!(Kd_VSSDYNDISK_Mask),
        core::ptr::addr_of_mut!(Kd_VERIFIER_Mask),
        core::ptr::addr_of_mut!(Kd_VDS_Mask),
        core::ptr::addr_of_mut!(Kd_VDSBAS_Mask),
        core::ptr::addr_of_mut!(Kd_VDSDYN_Mask),
        core::ptr::addr_of_mut!(Kd_VDSDYNDR_Mask),
        core::ptr::addr_of_mut!(Kd_VDSLDR_Mask),
        core::ptr::addr_of_mut!(Kd_VDSUTIL_Mask),
        core::ptr::addr_of_mut!(Kd_DFRGIFC_Mask),
        core::ptr::addr_of_mut!(Kd_DEFAULT_Mask),
        core::ptr::addr_of_mut!(Kd_MM_Mask),
        core::ptr::addr_of_mut!(Kd_DFSC_Mask),
        core::ptr::addr_of_mut!(Kd_WOW64_Mask),
        core::ptr::addr_of_mut!(Kd_ALPC_Mask),
        core::ptr::addr_of_mut!(Kd_WDI_Mask),
        core::ptr::addr_of_mut!(Kd_PERFLIB_Mask),
        core::ptr::addr_of_mut!(Kd_KTM_Mask),
        core::ptr::addr_of_mut!(Kd_IOSTRESS_Mask),
        core::ptr::addr_of_mut!(Kd_HEAP_Mask),
        core::ptr::addr_of_mut!(Kd_WHEA_Mask),
        core::ptr::addr_of_mut!(Kd_USERGDI_Mask),
        core::ptr::addr_of_mut!(Kd_MMCSS_Mask),
        core::ptr::addr_of_mut!(Kd_TPM_Mask),
        core::ptr::addr_of_mut!(Kd_THREADORDER_Mask),
        core::ptr::addr_of_mut!(Kd_ENVIRON_Mask),
        core::ptr::addr_of_mut!(Kd_EMS_Mask),
        core::ptr::addr_of_mut!(Kd_WDT_Mask),
        core::ptr::addr_of_mut!(Kd_FVEVOL_Mask),
        core::ptr::addr_of_mut!(Kd_NDIS_Mask),
        core::ptr::addr_of_mut!(Kd_NVCTRACE_Mask),
        core::ptr::addr_of_mut!(Kd_LUAFV_Mask),
        core::ptr::addr_of_mut!(Kd_APPCOMPAT_Mask),
        core::ptr::addr_of_mut!(Kd_USBSTOR_Mask),
        core::ptr::addr_of_mut!(Kd_SBP2PORT_Mask),
        core::ptr::addr_of_mut!(Kd_COVERAGE_Mask),
        core::ptr::addr_of_mut!(Kd_CACHEMGR_Mask),
        core::ptr::addr_of_mut!(Kd_MOUNTMGR_Mask),
        core::ptr::addr_of_mut!(Kd_CFR_Mask),
        core::ptr::addr_of_mut!(Kd_TXF_Mask),
        core::ptr::addr_of_mut!(Kd_KSECDD_Mask),
        core::ptr::addr_of_mut!(Kd_FLTREGRESS_Mask),
        core::ptr::addr_of_mut!(Kd_MPIO_Mask),
        core::ptr::addr_of_mut!(Kd_MSDSM_Mask),
        core::ptr::addr_of_mut!(Kd_UDFS_Mask),
        core::ptr::addr_of_mut!(Kd_PSHED_Mask),
        core::ptr::addr_of_mut!(Kd_STORVSP_Mask),
        core::ptr::addr_of_mut!(Kd_LSASS_Mask),
        core::ptr::addr_of_mut!(Kd_SSPICLI_Mask),
        core::ptr::addr_of_mut!(Kd_CNG_Mask),
        core::ptr::addr_of_mut!(Kd_EXFAT_Mask),
        core::ptr::addr_of_mut!(Kd_FILETRACE_Mask),
        core::ptr::addr_of_mut!(Kd_XSAVE_Mask),
        core::ptr::addr_of_mut!(Kd_SE_Mask),
        core::ptr::addr_of_mut!(Kd_DRIVEEXTENDER_Mask),
        core::ptr::addr_of_mut!(Kd_POWER_Mask),
        core::ptr::addr_of_mut!(Kd_CRASHDUMPXHCI_Mask),
        core::ptr::addr_of_mut!(Kd_GPIO_Mask),
        core::ptr::addr_of_mut!(Kd_REFS_Mask),
        core::ptr::addr_of_mut!(Kd_WER_Mask),
        core::ptr::addr_of_mut!(Kd_CAPIMG_Mask),
        core::ptr::addr_of_mut!(Kd_VPCI_Mask),
        core::ptr::addr_of_mut!(Kd_STORAGECLASSMEMORY_Mask),
        core::ptr::addr_of_mut!(Kd_FSLIB_Mask),
        core::ptr::addr_of_mut!(Kd_ENDOFTABLE_Mask),
    ]
};

#[no_mangle]
pub static mut KdComponentTableSize: ULONG = MAX_KD_COMPONENT_TABLE_ENTRIES as ULONG;

#[no_mangle]
pub static mut KdpDebuggerDataListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut KdpDataSpinLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KdVersionBlock: DBGKD_GET_VERSION64 = DBGKD_GET_VERSION64 {
    MajorVersion: 0,
    MinorVersion: 0,
    ProtocolVersion: DBGKD_64BIT_PROTOCOL_VERSION2 as u8,
    KdSecondaryVersion: CURRENT_KD_SECONDARY_VERSION,
    #[cfg(target_pointer_width = "64")]
    Flags: (DBGKD_VERS_FLAG_DATA | DBGKD_VERS_FLAG_PTR64),
    #[cfg(target_pointer_width = "32")]
    Flags: DBGKD_VERS_FLAG_DATA,
    MachineType: IMAGE_FILE_MACHINE_VALUE,
    MaxPacketType: PACKET_TYPE_MAX,
    MaxStateChange: 0,
    MaxManipulate: 0,
    Simulation: DBGKD_SIMULATION_NONE as u8,
    Unused: [0],
    KernBase: 0,
    PsLoadedModuleList: 0,
    DebuggerDataList: 0,
};

#[repr(C)]
#[allow(non_snake_case)]
pub struct KDDEBUGGER_DATA64_FULL {
    Header: DBGKD_DEBUG_DATA_HEADER64,
    KernBase: u64,
    BreakpointWithStatus: u64,
    SavedContext: u64,
    ThCallbackStack: u16,
    NextCallback: u16,
    FramePointer: u16,
    PaeEnabled: u8,
    KiCallUserMode: u64,
    KeUserCallbackDispatcher: u64,
    PsLoadedModuleList: u64,
    PsActiveProcessHead: u64,
    PspCidTable: u64,
    ExpSystemResourcesList: u64,
    ExpPagedPoolDescriptor: u64,
    ExpNumberOfPagedPools: u64,
    KeTimeIncrement: u64,
    KeBugCheckCallbackListHead: u64,
    KiBugCheckData: u64,
    IopErrorLogListHead: u64,
    ObpRootDirectoryObject: u64,
    ObpTypeObjectType: u64,
    MmSystemCacheStart: u64,
    MmSystemCacheEnd: u64,
    MmSystemCacheWs: u64,
    MmPfnDatabase: u64,
    MmSystemPtesStart: u64,
    MmSystemPtesEnd: u64,
    MmSubsectionBase: u64,
    MmNumberOfPagingFiles: u64,
    MmLowestPhysicalPage: u64,
    MmHighestPhysicalPage: u64,
    MmNumberOfPhysicalPages: u64,
    MmMaximumNonPagedPoolInBytes: u64,
    MmNonPagedSystemStart: u64,
    MmNonPagedPoolStart: u64,
    MmNonPagedPoolEnd: u64,
    MmPagedPoolStart: u64,
    MmPagedPoolEnd: u64,
    MmPagedPoolInfo: u64,
    PageSize: u64,
    MmSizeOfPagedPoolInBytes: u64,
    MmTotalCommitLimit: u64,
    MmTotalCommittedPages: u64,
    MmSharedCommit: u64,
    MmDriverCommit: u64,
    MmProcessCommit: u64,
    MmPagedPoolCommit: u64,
    MmExtendedCommit: u64,
    MmZeroedPageListHead: u64,
    MmFreePageListHead: u64,
    MmStandbyPageListHead: u64,
    MmModifiedPageListHead: u64,
    MmModifiedNoWritePageListHead: u64,
    MmAvailablePages: u64,
    MmResidentAvailablePages: u64,
    PoolTrackTable: u64,
    NonPagedPoolDescriptor: u64,
    MmHighestUserAddress: u64,
    MmSystemRangeStart: u64,
    MmUserProbeAddress: u64,
    KdPrintCircularBuffer: u64,
    KdPrintCircularBufferEnd: u64,
    KdPrintWritePointer: u64,
    KdPrintRolloverCount: u64,
    MmLoadedUserImageList: u64,
    NtBuildLab: u64,
    KiNormalSystemCall: u64,
    KiProcessorBlock: u64,
    MmUnloadedDrivers: u64,
    MmLastUnloadedDrivers: u64,
    MmTriageActionTaken: u64,
    MmSpecialPoolTag: u64,
    KernelVerifier: u64,
    MmVerifierData: u64,
    MmAllocatedNonPagedPool: u64,
    MmPeakCommitment: u64,
    MmtotalCommitLimitMaximum: u64,
    CmNtCSDVersion: u64,
    MmPhysicalMemoryBlock: u64,
    MmSessionBase: u64,
    MmSessionSize: u64,
    MmSystemParentTablePage: u64,
    MmVirtualTranslationBase: u64,
    OffsetKThreadNextProcessor: u16,
    OffsetKThreadTeb: u16,
    OffsetKThreadKernelStack: u16,
    OffsetKThreadInitialStack: u16,
    OffsetKThreadApcProcess: u16,
    OffsetKThreadState: u16,
    OffsetKThreadBStore: u16,
    OffsetKThreadBStoreLimit: u16,
    SizeEProcess: u16,
    OffsetEprocessPeb: u16,
    OffsetEprocessParentCID: u16,
    OffsetEprocessDirectoryTableBase: u16,
    SizePrcb: u16,
    OffsetPrcbDpcRoutine: u16,
    OffsetPrcbCurrentThread: u16,
    OffsetPrcbMhz: u16,
    OffsetPrcbCpuType: u16,
    OffsetPrcbVendorString: u16,
    OffsetPrcbProcStateContext: u16,
    OffsetPrcbNumber: u16,
    SizeEThread: u16,
    KdPrintCircularBufferPtr: u64,
    KdPrintBufferSize: u64,
    KeLoaderBlock: u64,
    SizePcr: u16,
    OffsetPcrSelfPcr: u16,
    OffsetPcrCurrentPrcb: u16,
    OffsetPcrContainedPrcb: u16,
    OffsetPcrInitialBStore: u16,
    OffsetPcrBStoreLimit: u16,
    OffsetPcrInitialStack: u16,
    OffsetPcrStackLimit: u16,
    OffsetPrcbPcrPage: u16,
    OffsetPrcbProcStateSpecialReg: u16,
    GdtR0Code: u16,
    GdtR0Data: u16,
    GdtR0Pcr: u16,
    GdtR3Code: u16,
    GdtR3Data: u16,
    GdtR3Teb: u16,
    GdtLdt: u16,
    GdtTss: u16,
    Gdt64R3CmCode: u16,
    Gdt64R3CmTeb: u16,
    IopNumTriageDumpDataBlocks: u64,
    IopTriageDumpDataBlocks: u64,
}

#[cfg(target_pointer_width = "32")]
#[inline]
fn ptr_to_ul64(x: *const core::ffi::c_void) -> u64 {
    x as usize as u64
}

#[cfg(target_pointer_width = "64")]
#[inline]
fn ptr_to_ul64(x: *const core::ffi::c_void) -> u64 {
    x as u64
}

#[no_mangle]
pub static mut KdDebuggerDataBlock: KDDEBUGGER_DATA64_FULL = unsafe { core::mem::zeroed() };

/// Initialize KdDebuggerDataBlock with pointer and constant values.
/// Must be called before KdRegisterDebuggerDataBlock in KdInitSystem.
#[no_mangle]
pub unsafe extern "system" fn KdpInitDebuggerDataBlock() {
    let d = &mut *(&raw mut KdDebuggerDataBlock);
    d.BreakpointWithStatus = RtlpBreakWithStatusInstruction as u64;
    d.ThCallbackStack = core::mem::offset_of!(KTHREAD, CallbackStack) as u16;
    #[cfg(not(any(target_arch = "arm", target_arch = "x86_64", target_arch = "aarch64")))]
    { d.NextCallback = core::mem::offset_of!(KCALLOUT_FRAME, CallbackStack) as u16; }
    #[cfg(not(any(target_arch = "arm", target_arch = "x86_64", target_arch = "aarch64")))]
    { d.FramePointer = core::mem::offset_of!(KCALLOUT_FRAME, Ebp) as u16; }
    d.KiCallUserMode = ptr_to_ul64(KiCallUserMode as *const _);
    d.PsLoadedModuleList = ptr_to_ul64(core::ptr::addr_of!(PsLoadedModuleList) as *const _);
    d.PsActiveProcessHead = ptr_to_ul64(core::ptr::addr_of!(PsActiveProcessHead) as *const _);
    d.PspCidTable = ptr_to_ul64(core::ptr::addr_of!(PspCidTable) as *const _);
    d.ExpSystemResourcesList = ptr_to_ul64(core::ptr::addr_of!(ExpSystemResourcesList) as *const _);
    d.ExpPagedPoolDescriptor = ptr_to_ul64(core::ptr::addr_of!(ExpPagedPoolDescriptor) as *const _);
    d.ExpNumberOfPagedPools = ptr_to_ul64(core::ptr::addr_of!(ExpNumberOfPagedPools) as *const _);
    d.KeTimeIncrement = ptr_to_ul64(core::ptr::addr_of!(KeTimeIncrement) as *const _);
    d.KeBugCheckCallbackListHead = ptr_to_ul64(core::ptr::addr_of!(KeBugcheckCallbackListHead) as *const _);
    d.KiBugCheckData = ptr_to_ul64(core::ptr::addr_of!(KiBugCheckData) as *const _);
    d.IopErrorLogListHead = ptr_to_ul64(core::ptr::addr_of!(IopErrorLogListHead) as *const _);
    d.ObpRootDirectoryObject = ptr_to_ul64(core::ptr::addr_of!(ObpRootDirectoryObject) as *const _);
    d.ObpTypeObjectType = ptr_to_ul64(core::ptr::addr_of!(ObpTypeObjectType) as *const _);
    d.MmSystemCacheStart = ptr_to_ul64(core::ptr::addr_of!(MmSystemCacheStart) as *const _);
    d.MmSystemCacheEnd = ptr_to_ul64(core::ptr::addr_of!(MmSystemCacheEnd) as *const _);
    d.MmSystemCacheWs = ptr_to_ul64(core::ptr::addr_of!(MmSystemCacheWs) as *const _);
    d.MmPfnDatabase = ptr_to_ul64(core::ptr::addr_of!(MmPfnDatabase) as *const _);
    d.MmSystemPtesStart = ptr_to_ul64(core::ptr::addr_of!(MmSystemPtesStart) as *const _);
    d.MmSystemPtesEnd = ptr_to_ul64(core::ptr::addr_of!(MmSystemPtesEnd) as *const _);
    d.MmSubsectionBase = ptr_to_ul64(core::ptr::addr_of!(MmSubsectionBase) as *const _);
    d.MmNumberOfPagingFiles = ptr_to_ul64(core::ptr::addr_of!(MmNumberOfPagingFiles) as *const _);
    d.MmLowestPhysicalPage = ptr_to_ul64(core::ptr::addr_of!(MmLowestPhysicalPage) as *const _);
    d.MmHighestPhysicalPage = ptr_to_ul64(core::ptr::addr_of!(MmHighestPhysicalPage) as *const _);
    d.MmNumberOfPhysicalPages = ptr_to_ul64(core::ptr::addr_of!(MmNumberOfPhysicalPages) as *const _);
    d.MmMaximumNonPagedPoolInBytes = ptr_to_ul64(core::ptr::addr_of!(MmMaximumNonPagedPoolInBytes) as *const _);
    d.MmNonPagedSystemStart = ptr_to_ul64(core::ptr::addr_of!(MmNonPagedSystemStart) as *const _);
    d.MmNonPagedPoolStart = ptr_to_ul64(core::ptr::addr_of!(MmNonPagedPoolStart) as *const _);
    d.MmNonPagedPoolEnd = ptr_to_ul64(core::ptr::addr_of!(MmNonPagedPoolEnd) as *const _);
    d.MmPagedPoolStart = ptr_to_ul64(core::ptr::addr_of!(MmPagedPoolStart) as *const _);
    d.MmPagedPoolEnd = ptr_to_ul64(core::ptr::addr_of!(MmPagedPoolEnd) as *const _);
    d.MmPagedPoolInfo = ptr_to_ul64(core::ptr::addr_of!(MmPagedPoolInfo) as *const _);
    d.PageSize = PAGE_SIZE as u64;
    d.MmSizeOfPagedPoolInBytes = ptr_to_ul64(core::ptr::addr_of!(MmSizeOfPagedPoolInBytes) as *const _);
    d.MmTotalCommitLimit = ptr_to_ul64(core::ptr::addr_of!(MmTotalCommitLimit) as *const _);
    d.MmTotalCommittedPages = ptr_to_ul64(core::ptr::addr_of!(MmTotalCommittedPages) as *const _);
    d.MmSharedCommit = ptr_to_ul64(core::ptr::addr_of!(MmSharedCommit) as *const _);
    d.MmDriverCommit = ptr_to_ul64(core::ptr::addr_of!(MmDriverCommit) as *const _);
    d.MmProcessCommit = ptr_to_ul64(core::ptr::addr_of!(MmProcessCommit) as *const _);
    d.MmPagedPoolCommit = ptr_to_ul64(core::ptr::addr_of!(MmPagedPoolCommit) as *const _);
    d.MmZeroedPageListHead = ptr_to_ul64(core::ptr::addr_of!(MmZeroedPageListHead) as *const _);
    d.MmFreePageListHead = ptr_to_ul64(core::ptr::addr_of!(MmFreePageListHead) as *const _);
    d.MmStandbyPageListHead = ptr_to_ul64(core::ptr::addr_of!(MmStandbyPageListHead) as *const _);
    d.MmModifiedPageListHead = ptr_to_ul64(core::ptr::addr_of!(MmModifiedPageListHead) as *const _);
    d.MmModifiedNoWritePageListHead = ptr_to_ul64(core::ptr::addr_of!(MmModifiedNoWritePageListHead) as *const _);
    d.MmAvailablePages = ptr_to_ul64(core::ptr::addr_of!(MmAvailablePages) as *const _);
    d.MmResidentAvailablePages = ptr_to_ul64(core::ptr::addr_of!(MmResidentAvailablePages) as *const _);
    d.PoolTrackTable = ptr_to_ul64(core::ptr::addr_of!(PoolTrackTable) as *const _);
    d.NonPagedPoolDescriptor = ptr_to_ul64(core::ptr::addr_of!(NonPagedPoolDescriptor) as *const _);
    d.MmHighestUserAddress = ptr_to_ul64(core::ptr::addr_of!(MmHighestUserAddress) as *const _);
    d.MmSystemRangeStart = ptr_to_ul64(core::ptr::addr_of!(MmSystemRangeStart) as *const _);
    d.MmUserProbeAddress = ptr_to_ul64(core::ptr::addr_of!(MmUserProbeAddress) as *const _);
    d.KdPrintCircularBuffer = ptr_to_ul64(core::ptr::addr_of!(KdPrintDefaultCircularBuffer) as *const _);
    d.KdPrintCircularBufferEnd = ptr_to_ul64((core::ptr::addr_of!(KdPrintDefaultCircularBuffer) as *const u8).add(KD_DEFAULT_LOG_BUFFER_SIZE) as *const _); // nolint: magic-offset (buffer size, not a field offset)
    d.KdPrintWritePointer = ptr_to_ul64(core::ptr::addr_of!(KdPrintWritePointer) as *const _);
    d.KdPrintRolloverCount = ptr_to_ul64(core::ptr::addr_of!(KdPrintRolloverCount) as *const _);
    d.MmLoadedUserImageList = ptr_to_ul64(core::ptr::addr_of!(MmLoadedUserImageList) as *const _);
    d.NtBuildLab = ptr_to_ul64(core::ptr::addr_of!(NtBuildLab) as *const _);
    d.KiProcessorBlock = ptr_to_ul64(core::ptr::addr_of!(KiProcessorBlock) as *const _);
    d.MmUnloadedDrivers = ptr_to_ul64(core::ptr::addr_of!(MmUnloadedDrivers) as *const _);
    d.MmLastUnloadedDrivers = ptr_to_ul64(core::ptr::addr_of!(MmLastUnloadedDrivers) as *const _);
    d.MmTriageActionTaken = ptr_to_ul64(core::ptr::addr_of!(MmTriageActionTaken) as *const _);
    d.MmSpecialPoolTag = ptr_to_ul64(core::ptr::addr_of!(MmSpecialPoolTag) as *const _);
    d.KernelVerifier = ptr_to_ul64(core::ptr::addr_of!(KernelVerifier) as *const _);
    d.MmVerifierData = ptr_to_ul64(core::ptr::addr_of!(MmVerifierData) as *const _);
    d.MmAllocatedNonPagedPool = ptr_to_ul64(core::ptr::addr_of!(MmAllocatedNonPagedPool) as *const _);
    d.MmPeakCommitment = ptr_to_ul64(core::ptr::addr_of!(MmPeakCommitment) as *const _);
    d.MmtotalCommitLimitMaximum = ptr_to_ul64(core::ptr::addr_of!(MmtotalCommitLimitMaximum) as *const _);
    d.CmNtCSDVersion = ptr_to_ul64(core::ptr::addr_of!(CmNtCSDVersion) as *const _);
    d.MmPhysicalMemoryBlock = ptr_to_ul64(core::ptr::addr_of!(MmPhysicalMemoryBlock) as *const _);
    d.MmSessionBase = ptr_to_ul64(core::ptr::addr_of!(MmSessionBase) as *const _);
    d.MmSessionSize = ptr_to_ul64(core::ptr::addr_of!(MmSessionSize) as *const _);
    d.OffsetKThreadNextProcessor = core::mem::offset_of!(KTHREAD, __bindgen_anon_1.__bindgen_anon_1.NextProcessor) as u16;
    d.OffsetKThreadTeb = core::mem::offset_of!(KTHREAD, Teb) as u16;
    d.OffsetKThreadKernelStack = core::mem::offset_of!(KTHREAD, KernelStack) as u16;
    d.OffsetKThreadInitialStack = core::mem::offset_of!(KTHREAD, InitialStack) as u16;
    d.OffsetKThreadApcProcess = (core::mem::offset_of!(KTHREAD, __bindgen_anon_1.ApcState) + core::mem::offset_of!(_KAPC_STATE, Process)) as u16;
    #[cfg(not(target_arch = "x86"))]
    { d.OffsetKThreadState = core::mem::offset_of!(KTHREAD, __bindgen_anon_5.__bindgen_anon_6.State) as u16; }
    #[cfg(target_arch = "x86")]
    { d.OffsetKThreadState = core::mem::offset_of!(KTHREAD, State) as u16; }
    d.SizeEProcess = core::mem::size_of::<_EPROCESS>() as u16;
    d.OffsetEprocessPeb = core::mem::offset_of!(_EPROCESS, Peb) as u16;
    d.OffsetEprocessParentCID = core::mem::offset_of!(_EPROCESS, InheritedFromUniqueProcessId) as u16;
    d.OffsetEprocessDirectoryTableBase = (core::mem::offset_of!(_EPROCESS, Pcb) + core::mem::offset_of!(KPROCESS, DirectoryTableBase)) as u16;
    d.SizePrcb = core::mem::size_of::<_KPRCB>() as u16;
    d.OffsetPrcbDpcRoutine = core::mem::offset_of!(_KPRCB, DpcRoutineActive) as u16;
    d.OffsetPrcbCurrentThread = core::mem::offset_of!(_KPRCB, CurrentThread) as u16;
    d.OffsetPrcbMhz = core::mem::offset_of!(_KPRCB, MHz) as u16;
    d.OffsetPrcbCpuType = core::mem::offset_of!(_KPRCB, CpuType) as u16;
    d.OffsetPrcbVendorString = core::mem::offset_of!(_KPRCB, VendorString) as u16;
    d.OffsetPrcbProcStateContext = (core::mem::offset_of!(_KPRCB, ProcessorState) + core::mem::offset_of!(KPROCESSOR_STATE, ContextFrame)) as u16;
    d.OffsetPrcbNumber = core::mem::offset_of!(_KPRCB, Number) as u16;
    d.SizeEThread = core::mem::size_of::<_ETHREAD>() as u16;
    d.KdPrintCircularBufferPtr = ptr_to_ul64(core::ptr::addr_of!(KdPrintCircularBuffer) as *const _);
    d.KdPrintBufferSize = ptr_to_ul64(core::ptr::addr_of!(KdPrintBufferSize) as *const _);
    d.KeLoaderBlock = ptr_to_ul64(core::ptr::addr_of!(KeLoaderBlock) as *const _);
    d.SizePcr = core::mem::size_of::<KPCR>() as u16;
    d.OffsetPcrSelfPcr = KPCR_SELF_PCR_OFFSET as u16;
    d.OffsetPcrCurrentPrcb = KPCR_CURRENT_PRCB_OFFSET as u16;
    d.OffsetPcrContainedPrcb = KPCR_CONTAINED_PRCB_OFFSET as u16;
    #[cfg(target_arch = "arm")]
    { d.OffsetPcrInitialStack = KPCR_INITIAL_STACK_OFFSET as u16; }
    #[cfg(target_arch = "arm")]
    { d.OffsetPcrStackLimit = KPCR_STACK_LIMIT_OFFSET as u16; }
    #[cfg(target_arch = "arm")]
    { d.OffsetPrcbPcrPage = KPRCB_PCR_PAGE_OFFSET as u16; }
    d.OffsetPrcbProcStateSpecialReg = (core::mem::offset_of!(_KPRCB, ProcessorState) + core::mem::offset_of!(KPROCESSOR_STATE, SpecialRegisters)) as u16;
    #[cfg(target_arch = "x86")]
    { d.GdtR0Code = KGDT_R0_CODE; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR0Code = KGDT64_R0_CODE; }
    #[cfg(target_arch = "x86")]
    { d.GdtR0Data = KGDT_R0_DATA; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR0Data = KGDT64_R3_DATA; }
    #[cfg(target_arch = "x86")]
    { d.GdtR0Pcr = KGDT_R0_PCR; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR0Pcr = KGDT64_R3_DATA; }
    #[cfg(target_arch = "x86")]
    { d.GdtR3Code = KGDT_R3_CODE; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR3Code = KGDT64_R3_CODE; }
    #[cfg(target_arch = "x86")]
    { d.GdtR3Data = KGDT_R3_DATA; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR3Data = KGDT64_R3_DATA; }
    #[cfg(target_arch = "x86")]
    { d.GdtR3Teb = KGDT_R3_TEB; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtR3Teb = KGDT64_R3_DATA; }
    #[cfg(target_arch = "x86")]
    { d.GdtLdt = KGDT_LDT; }
    #[cfg(target_arch = "x86")]
    { d.GdtTss = KGDT_TSS; }
    #[cfg(target_arch = "x86_64")]
    { d.GdtTss = KGDT64_SYS_TSS; }
    d.IopNumTriageDumpDataBlocks = ptr_to_ul64(core::ptr::addr_of!(IopNumTriageDumpDataBlocks) as *const _);
    d.IopTriageDumpDataBlocks = ptr_to_ul64(core::ptr::addr_of!(IopTriageDumpDataBlocks) as *const _);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kddata.c
//  * PURPOSE:         Contains all global variables and settings for KD64
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include <mm/ARM3/miarm.h>
// #undef MmSystemRangeStart
//
// VOID NTAPI RtlpBreakWithStatusInstruction(VOID);
//
// //
// // Apply the KIPCR WDK workaround for x86 and AMD64
// //
// #if defined(_M_IX86) || defined(_M_AMD64)
// #define KPCR KIPCR
// #endif
//
// #if defined(_M_IX86)
//
// #define KPCR_SELF_PCR_OFFSET           FIELD_OFFSET(KPCR, SelfPcr)
// #define KPCR_CURRENT_PRCB_OFFSET       FIELD_OFFSET(KPCR, Prcb)
// #define KPCR_CONTAINED_PRCB_OFFSET     FIELD_OFFSET(KPCR, PrcbData)
// #define KPCR_INITIAL_STACK_OFFSET      0
// #define KPCR_STACK_LIMIT_OFFSET        0
// #define KPRCB_PCR_PAGE_OFFSET          0
// #define CBSTACK_FRAME_POINTER          Ebp
//
// #elif defined(_M_AMD64)
//
// #define KPCR_SELF_PCR_OFFSET           FIELD_OFFSET(KPCR, Self)
// #define KPCR_CURRENT_PRCB_OFFSET       FIELD_OFFSET(KPCR, CurrentPrcb)
// #define KPCR_CONTAINED_PRCB_OFFSET     FIELD_OFFSET(KPCR, Prcb)
// #define KPCR_INITIAL_STACK_OFFSET      0
// #define KPCR_STACK_LIMIT_OFFSET        0
// #define KPRCB_PCR_PAGE_OFFSET          0
// #define CBSTACK_FRAME_POINTER          Rbp
//
// #elif defined(_M_ARM)
//
// #define KPCR_SELF_PCR_OFFSET           0
// #define KPCR_CURRENT_PRCB_OFFSET       FIELD_OFFSET(KIPCR, Prcb)
// #define KPCR_CONTAINED_PRCB_OFFSET     0
// #define KPCR_INITIAL_STACK_OFFSET      FIELD_OFFSET(KPCR, InitialStack)
// #define KPCR_STACK_LIMIT_OFFSET        FIELD_OFFSET(KPCR, StackLimit)
// #define KPRCB_PCR_PAGE_OFFSET          FIELD_OFFSET(KPRCB, PcrPage)
// #define CBSTACK_FRAME_POINTER          DummyFramePointer
//
// #elif defined(_M_ARM64)
//
// #define KPCR_SELF_PCR_OFFSET           FIELD_OFFSET(KPCR, Self)
// #define KPCR_CURRENT_PRCB_OFFSET       FIELD_OFFSET(KPCR, CurrentPrcb)
// #define KPCR_CONTAINED_PRCB_OFFSET     FIELD_OFFSET(KIPCR, Prcb)
// #define KPCR_INITIAL_STACK_OFFSET      0
// #define KPCR_STACK_LIMIT_OFFSET        0
// #define KPRCB_PCR_PAGE_OFFSET          0
// #define CBSTACK_FRAME_POINTER          Fp
//
// #else
// #error Unsupported Architecture
// #endif
//
// /* GLOBALS *******************************************************************/
//
// //
// // Debugger State
// //
// KD_CONTEXT KdpContext;
// BOOLEAN KdpPortLocked;
// KSPIN_LOCK KdpDebuggerLock;
// BOOLEAN KdpControlCPressed;
// BOOLEAN KdpContextSent;
//
// //
// // Debug Trap Handlers
// //
// PKDEBUG_ROUTINE KiDebugRoutine = KdpStub;
//
// //
// // Debugger Configuration Settings
// //
// BOOLEAN KdBreakAfterSymbolLoad;
// BOOLEAN KdPitchDebugger;
// BOOLEAN KdDebuggerNotPresent;
// BOOLEAN KdDebuggerEnabled;
// BOOLEAN KdAutoEnableOnEvent;
// BOOLEAN KdBlockEnable;
// BOOLEAN KdIgnoreUmExceptions;
// BOOLEAN KdPreviouslyEnabled;
// BOOLEAN KdpDebuggerStructuresInitialized;
// BOOLEAN KdEnteredDebugger;
// ULONG KdDisableCount;
// LARGE_INTEGER KdPerformanceCounterRate;
//
// //
// // Breakpoint Data
// //
// BREAKPOINT_ENTRY KdpBreakpointTable[KD_BREAKPOINT_MAX];
// KD_BREAKPOINT_TYPE KdpBreakpointInstruction = KD_BREAKPOINT_VALUE;
// BOOLEAN KdpOweBreakpoint;
// BOOLEAN BreakpointsSuspended;
// ULONG KdpNumInternalBreakpoints;
//
// //
// // Symbol Data
// //
// ULONG_PTR KdpCurrentSymbolStart, KdpCurrentSymbolEnd;
//
// //
// // Tracepoint Data
// //
// ULONG TraceDataBuffer[40];
// ULONG TraceDataBufferPosition = 1;
//
// //
// // Time Slip Support
// //
// KDPC KdpTimeSlipDpc;
// KTIMER KdpTimeSlipTimer;
// WORK_QUEUE_ITEM KdpTimeSlipWorkItem;
// LONG KdpTimeSlipPending = 1;
// PKEVENT KdpTimeSlipEvent;
// KSPIN_LOCK KdpTimeSlipEventLock;
// LARGE_INTEGER KdTimerStop, KdTimerStart, KdTimerDifference;
//
// //
// // Buffers
// //
// CHAR KdpMessageBuffer[KDP_MSG_BUFFER_SIZE];
// CHAR KdpPathBuffer[KDP_MSG_BUFFER_SIZE];
//
// //
// // KdPrint Buffers
// //
// CHAR KdPrintDefaultCircularBuffer[KD_DEFAULT_LOG_BUFFER_SIZE];
// PCHAR KdPrintWritePointer = KdPrintDefaultCircularBuffer;
// ULONG KdPrintRolloverCount;
// PCHAR KdPrintCircularBuffer = KdPrintDefaultCircularBuffer;
// ULONG KdPrintBufferSize = sizeof(KdPrintDefaultCircularBuffer);
// ULONG KdPrintBufferChanges = 0;
// KSPIN_LOCK KdpPrintSpinLock;
//
// //
// // Debug Filter Masks
// //
// ULONG Kd_WIN2000_Mask = 1;
// ULONG Kd_SYSTEM_Mask;
// ULONG Kd_SMSS_Mask;
// ULONG Kd_SETUP_Mask;
// ULONG Kd_NTFS_Mask;
// ULONG Kd_FSTUB_Mask;
// ULONG Kd_CRASHDUMP_Mask;
// ULONG Kd_CDAUDIO_Mask;
// ULONG Kd_CDROM_Mask;
// ULONG Kd_CLASSPNP_Mask;
// ULONG Kd_DISK_Mask;
// ULONG Kd_REDBOOK_Mask;
// ULONG Kd_STORPROP_Mask;
// ULONG Kd_SCSIPORT_Mask;
// ULONG Kd_SCSIMINIPORT_Mask;
// ULONG Kd_CONFIG_Mask;
// ULONG Kd_I8042PRT_Mask;
// ULONG Kd_SERMOUSE_Mask;
// ULONG Kd_LSERMOUS_Mask;
// ULONG Kd_KBDHID_Mask;
// ULONG Kd_MOUHID_Mask;
// ULONG Kd_KBDCLASS_Mask;
// ULONG Kd_MOUCLASS_Mask;
// ULONG Kd_TWOTRACK_Mask;
// ULONG Kd_WMILIB_Mask;
// ULONG Kd_ACPI_Mask;
// ULONG Kd_AMLI_Mask;
// ULONG Kd_HALIA64_Mask;
// ULONG Kd_VIDEO_Mask;
// ULONG Kd_SVCHOST_Mask;
// ULONG Kd_VIDEOPRT_Mask;
// ULONG Kd_TCPIP_Mask;
// ULONG Kd_DMSYNTH_Mask;
// ULONG Kd_NTOSPNP_Mask;
// ULONG Kd_FASTFAT_Mask;
// ULONG Kd_SAMSS_Mask;
// ULONG Kd_PNPMGR_Mask;
// ULONG Kd_NETAPI_Mask;
// ULONG Kd_SCSERVER_Mask;
// ULONG Kd_SCCLIENT_Mask;
// ULONG Kd_SERIAL_Mask;
// ULONG Kd_SERENUM_Mask;
// ULONG Kd_UHCD_Mask;
// ULONG Kd_RPCPROXY_Mask;
// ULONG Kd_AUTOCHK_Mask;
// ULONG Kd_DCOMSS_Mask;
// ULONG Kd_UNIMODEM_Mask;
// ULONG Kd_SIS_Mask;
// ULONG Kd_FLTMGR_Mask;
// ULONG Kd_WMICORE_Mask;
// ULONG Kd_BURNENG_Mask;
// ULONG Kd_IMAPI_Mask;
// ULONG Kd_SXS_Mask;
// ULONG Kd_FUSION_Mask;
// ULONG Kd_IDLETASK_Mask;
// ULONG Kd_SOFTPCI_Mask;
// ULONG Kd_TAPE_Mask;
// ULONG Kd_MCHGR_Mask;
// ULONG Kd_IDEP_Mask;
// ULONG Kd_PCIIDE_Mask;
// ULONG Kd_FLOPPY_Mask;
// ULONG Kd_FDC_Mask;
// ULONG Kd_TERMSRV_Mask;
// ULONG Kd_W32TIME_Mask;
// ULONG Kd_PREFETCHER_Mask;
// ULONG Kd_RSFILTER_Mask;
// ULONG Kd_FCPORT_Mask;
// ULONG Kd_PCI_Mask;
// ULONG Kd_DMIO_Mask;
// ULONG Kd_DMCONFIG_Mask;
// ULONG Kd_DMADMIN_Mask;
// ULONG Kd_WSOCKTRANSPORT_Mask;
// ULONG Kd_VSS_Mask;
// ULONG Kd_PNPMEM_Mask;
// ULONG Kd_PROCESSOR_Mask;
// ULONG Kd_DMSERVER_Mask;
// ULONG Kd_SR_Mask;
// ULONG Kd_INFINIBAND_Mask;
// ULONG Kd_IHVDRIVER_Mask;
// ULONG Kd_IHVVIDEO_Mask;
// ULONG Kd_IHVAUDIO_Mask;
// ULONG Kd_IHVNETWORK_Mask;
// ULONG Kd_IHVSTREAMING_Mask;
// ULONG Kd_IHVBUS_Mask;
// ULONG Kd_HPS_Mask;
// ULONG Kd_RTLTHREADPOOL_Mask;
// ULONG Kd_LDR_Mask;
// ULONG Kd_TCPIP6_Mask;
// ULONG Kd_ISAPNP_Mask;
// ULONG Kd_SHPC_Mask;
// ULONG Kd_STORPORT_Mask;
// ULONG Kd_STORMINIPORT_Mask;
// ULONG Kd_PRINTSPOOLER_Mask;
// ULONG Kd_VSSDYNDISK_Mask;
// ULONG Kd_VERIFIER_Mask;
// ULONG Kd_VDS_Mask;
// ULONG Kd_VDSBAS_Mask;
// ULONG Kd_VDSDYN_Mask;   // Specified in Vista+
// ULONG Kd_VDSDYNDR_Mask;
// ULONG Kd_VDSLDR_Mask;   // Specified in Vista+
// ULONG Kd_VDSUTIL_Mask;
// ULONG Kd_DFRGIFC_Mask;
// ULONG Kd_DEFAULT_Mask;
// ULONG Kd_MM_Mask;
// ULONG Kd_DFSC_Mask;
// ULONG Kd_WOW64_Mask;
// //
// // Components specified in Vista+, some of which we also use in ReactOS
// //
// ULONG Kd_ALPC_Mask;
// ULONG Kd_WDI_Mask;
// ULONG Kd_PERFLIB_Mask;
// ULONG Kd_KTM_Mask;
// ULONG Kd_IOSTRESS_Mask;
// ULONG Kd_HEAP_Mask;
// ULONG Kd_WHEA_Mask;
// ULONG Kd_USERGDI_Mask;
// ULONG Kd_MMCSS_Mask;
// ULONG Kd_TPM_Mask;
// ULONG Kd_THREADORDER_Mask;
// ULONG Kd_ENVIRON_Mask;
// ULONG Kd_EMS_Mask;
// ULONG Kd_WDT_Mask;
// ULONG Kd_FVEVOL_Mask;
// ULONG Kd_NDIS_Mask;
// ULONG Kd_NVCTRACE_Mask;
// ULONG Kd_LUAFV_Mask;
// ULONG Kd_APPCOMPAT_Mask;
// ULONG Kd_USBSTOR_Mask;
// ULONG Kd_SBP2PORT_Mask;
// ULONG Kd_COVERAGE_Mask;
// ULONG Kd_CACHEMGR_Mask;
// ULONG Kd_MOUNTMGR_Mask;
// ULONG Kd_CFR_Mask;
// ULONG Kd_TXF_Mask;
// ULONG Kd_KSECDD_Mask;
// ULONG Kd_FLTREGRESS_Mask;
// ULONG Kd_MPIO_Mask;
// ULONG Kd_MSDSM_Mask;
// ULONG Kd_UDFS_Mask;
// ULONG Kd_PSHED_Mask;
// ULONG Kd_STORVSP_Mask;
// ULONG Kd_LSASS_Mask;
// ULONG Kd_SSPICLI_Mask;
// ULONG Kd_CNG_Mask;
// ULONG Kd_EXFAT_Mask;
// ULONG Kd_FILETRACE_Mask;
// ULONG Kd_XSAVE_Mask;
// ULONG Kd_SE_Mask;
// ULONG Kd_DRIVEEXTENDER_Mask;
// //
// // Components specified in Windows 8
// //
// ULONG Kd_POWER_Mask;
// ULONG Kd_CRASHDUMPXHCI_Mask;
// ULONG Kd_GPIO_Mask;
// ULONG Kd_REFS_Mask;
// ULONG Kd_WER_Mask;
// //
// // Components specified in Windows 10
// //
// ULONG Kd_CAPIMG_Mask;
// ULONG Kd_VPCI_Mask;
// ULONG Kd_STORAGECLASSMEMORY_Mask;
// ULONG Kd_FSLIB_Mask;
// // End Mask
// ULONG Kd_ENDOFTABLE_Mask;
//
// //
// // Debug Filter Component Table
// //
// PULONG KdComponentTable[MAX_KD_COMPONENT_TABLE_ENTRIES] =
// {
//     &Kd_SYSTEM_Mask,
//     &Kd_SMSS_Mask,
//     &Kd_SETUP_Mask,
//     &Kd_NTFS_Mask,
//     &Kd_FSTUB_Mask,
//     &Kd_CRASHDUMP_Mask,
//     &Kd_CDAUDIO_Mask,
//     &Kd_CDROM_Mask,
//     &Kd_CLASSPNP_Mask,
//     &Kd_DISK_Mask,
//     &Kd_REDBOOK_Mask,
//     &Kd_STORPROP_Mask,
//     &Kd_SCSIPORT_Mask,
//     &Kd_SCSIMINIPORT_Mask,
//     &Kd_CONFIG_Mask,
//     &Kd_I8042PRT_Mask,
//     &Kd_SERMOUSE_Mask,
//     &Kd_LSERMOUS_Mask,
//     &Kd_KBDHID_Mask,
//     &Kd_MOUHID_Mask,
//     &Kd_KBDCLASS_Mask,
//     &Kd_MOUCLASS_Mask,
//     &Kd_TWOTRACK_Mask,
//     &Kd_WMILIB_Mask,
//     &Kd_ACPI_Mask,
//     &Kd_AMLI_Mask,
//     &Kd_HALIA64_Mask,
//     &Kd_VIDEO_Mask,
//     &Kd_SVCHOST_Mask,
//     &Kd_VIDEOPRT_Mask,
//     &Kd_TCPIP_Mask,
//     &Kd_DMSYNTH_Mask,
//     &Kd_NTOSPNP_Mask,
//     &Kd_FASTFAT_Mask,
//     &Kd_SAMSS_Mask,
//     &Kd_PNPMGR_Mask,
//     &Kd_NETAPI_Mask,
//     &Kd_SCSERVER_Mask,
//     &Kd_SCCLIENT_Mask,
//     &Kd_SERIAL_Mask,
//     &Kd_SERENUM_Mask,
//     &Kd_UHCD_Mask,
//     &Kd_RPCPROXY_Mask,
//     &Kd_AUTOCHK_Mask,
//     &Kd_DCOMSS_Mask,
//     &Kd_UNIMODEM_Mask,
//     &Kd_SIS_Mask,
//     &Kd_FLTMGR_Mask,
//     &Kd_WMICORE_Mask,
//     &Kd_BURNENG_Mask,
//     &Kd_IMAPI_Mask,
//     &Kd_SXS_Mask,
//     &Kd_FUSION_Mask,
//     &Kd_IDLETASK_Mask,
//     &Kd_SOFTPCI_Mask,
//     &Kd_TAPE_Mask,
//     &Kd_MCHGR_Mask,
//     &Kd_IDEP_Mask,
//     &Kd_PCIIDE_Mask,
//     &Kd_FLOPPY_Mask,
//     &Kd_FDC_Mask,
//     &Kd_TERMSRV_Mask,
//     &Kd_W32TIME_Mask,
//     &Kd_PREFETCHER_Mask,
//     &Kd_RSFILTER_Mask,
//     &Kd_FCPORT_Mask,
//     &Kd_PCI_Mask,
//     &Kd_DMIO_Mask,
//     &Kd_DMCONFIG_Mask,
//     &Kd_DMADMIN_Mask,
//     &Kd_WSOCKTRANSPORT_Mask,
//     &Kd_VSS_Mask,
//     &Kd_PNPMEM_Mask,
//     &Kd_PROCESSOR_Mask,
//     &Kd_DMSERVER_Mask,
//     &Kd_SR_Mask,
//     &Kd_INFINIBAND_Mask,
//     &Kd_IHVDRIVER_Mask,
//     &Kd_IHVVIDEO_Mask,
//     &Kd_IHVAUDIO_Mask,
//     &Kd_IHVNETWORK_Mask,
//     &Kd_IHVSTREAMING_Mask,
//     &Kd_IHVBUS_Mask,
//     &Kd_HPS_Mask,
//     &Kd_RTLTHREADPOOL_Mask,
//     &Kd_LDR_Mask,
//     &Kd_TCPIP6_Mask,
//     &Kd_ISAPNP_Mask,
//     &Kd_SHPC_Mask,
//     &Kd_STORPORT_Mask,
//     &Kd_STORMINIPORT_Mask,
//     &Kd_PRINTSPOOLER_Mask,
//     &Kd_VSSDYNDISK_Mask,
//     &Kd_VERIFIER_Mask,
//     &Kd_VDS_Mask,
//     &Kd_VDSBAS_Mask,
//     &Kd_VDSDYN_Mask,    // Specified in Vista+
//     &Kd_VDSDYNDR_Mask,
//     &Kd_VDSLDR_Mask,    // Specified in Vista+
//     &Kd_VDSUTIL_Mask,
//     &Kd_DFRGIFC_Mask,
//     &Kd_DEFAULT_Mask,
//     &Kd_MM_Mask,
//     &Kd_DFSC_Mask,
//     &Kd_WOW64_Mask,
// //
// // Components specified in Vista+, some of which we also use in ReactOS
// //
//     &Kd_ALPC_Mask,
//     &Kd_WDI_Mask,
//     &Kd_PERFLIB_Mask,
//     &Kd_KTM_Mask,
//     &Kd_IOSTRESS_Mask,
//     &Kd_HEAP_Mask,
//     &Kd_WHEA_Mask,
//     &Kd_USERGDI_Mask,
//     &Kd_MMCSS_Mask,
//     &Kd_TPM_Mask,
//     &Kd_THREADORDER_Mask,
//     &Kd_ENVIRON_Mask,
//     &Kd_EMS_Mask,
//     &Kd_WDT_Mask,
//     &Kd_FVEVOL_Mask,
//     &Kd_NDIS_Mask,
//     &Kd_NVCTRACE_Mask,
//     &Kd_LUAFV_Mask,
//     &Kd_APPCOMPAT_Mask,
//     &Kd_USBSTOR_Mask,
//     &Kd_SBP2PORT_Mask,
//     &Kd_COVERAGE_Mask,
//     &Kd_CACHEMGR_Mask,
//     &Kd_MOUNTMGR_Mask,
//     &Kd_CFR_Mask,
//     &Kd_TXF_Mask,
//     &Kd_KSECDD_Mask,
//     &Kd_FLTREGRESS_Mask,
//     &Kd_MPIO_Mask,
//     &Kd_MSDSM_Mask,
//     &Kd_UDFS_Mask,
//     &Kd_PSHED_Mask,
//     &Kd_STORVSP_Mask,
//     &Kd_LSASS_Mask,
//     &Kd_SSPICLI_Mask,
//     &Kd_CNG_Mask,
//     &Kd_EXFAT_Mask,
//     &Kd_FILETRACE_Mask,
//     &Kd_XSAVE_Mask,
//     &Kd_SE_Mask,
//     &Kd_DRIVEEXTENDER_Mask,
// //
// // Components specified in Windows 8
// //
//     &Kd_POWER_Mask,
//     &Kd_CRASHDUMPXHCI_Mask,
//     &Kd_GPIO_Mask,
//     &Kd_REFS_Mask,
//     &Kd_WER_Mask,
// //
// // Components specified in Windows 10
// //
//     &Kd_CAPIMG_Mask,
//     &Kd_VPCI_Mask,
//     &Kd_STORAGECLASSMEMORY_Mask,
//     &Kd_FSLIB_Mask,
// // End Mask
//     &Kd_ENDOFTABLE_Mask,
// };
//
// ULONG KdComponentTableSize = RTL_NUMBER_OF(KdComponentTable);
//
// //
// // Debugger Data
// //
// LIST_ENTRY KdpDebuggerDataListHead;
// KSPIN_LOCK KdpDataSpinLock;
//
// //
// // Debugger Version and Data Block
// //
// DBGKD_GET_VERSION64 KdVersionBlock =
// {
//     0,
//     0,
//     DBGKD_64BIT_PROTOCOL_VERSION2,
//     CURRENT_KD_SECONDARY_VERSION,
// #if defined(_M_AMD64) || defined(_M_ARM64)
//     DBGKD_VERS_FLAG_DATA | DBGKD_VERS_FLAG_PTR64,
// #else
//     DBGKD_VERS_FLAG_DATA,
// #endif
//     IMAGE_FILE_MACHINE_NATIVE,
//     PACKET_TYPE_MAX,
//     0,
//     0,
//     DBGKD_SIMULATION_NONE,
//     {0},
//     0,
//     0,
//     0
// };
//
// #if (NTDDI_VERSION >= NTDDI_WS03)
// C_ASSERT(sizeof(KDDEBUGGER_DATA64) >= 0x318);
// #endif
//
// #if !defined(_WIN64) && (defined(__GNUC__) || defined(__clang__))
// /* Minimal hackery for GCC/Clang, see commit b9cd3f2d9 (r25845) and de81021ba */
// #define PtrToUL64(x)    ((ULPTR64)(ULONG_PTR)(x))
// #else
// #define PtrToUL64(x)    ((ULPTR64)(x))
// #endif
// KDDEBUGGER_DATA64 KdDebuggerDataBlock =
// {
//     {{0}},
//     0,
//     PtrToUL64(RtlpBreakWithStatusInstruction),
//     0,
//     FIELD_OFFSET(KTHREAD, CallbackStack),
// #if defined(_M_ARM) || defined(_M_AMD64) || defined(_M_ARM64)
//     0,
//     0,
// #else
//     FIELD_OFFSET(KCALLOUT_FRAME, CallbackStack),
//     FIELD_OFFSET(KCALLOUT_FRAME, CBSTACK_FRAME_POINTER),
// #endif
//     FALSE,
//     PtrToUL64(KiCallUserMode),
//     0,
//     PtrToUL64(&PsLoadedModuleList),
//     PtrToUL64(&PsActiveProcessHead),
//     PtrToUL64(&PspCidTable),
//     PtrToUL64(&ExpSystemResourcesList),
//     PtrToUL64(ExpPagedPoolDescriptor),
//     PtrToUL64(&ExpNumberOfPagedPools),
//     PtrToUL64(&KeTimeIncrement),
//     PtrToUL64(&KeBugcheckCallbackListHead),
//     PtrToUL64(KiBugCheckData),
//     PtrToUL64(&IopErrorLogListHead),
//     PtrToUL64(&ObpRootDirectoryObject),
//     PtrToUL64(&ObpTypeObjectType),
//     PtrToUL64(&MmSystemCacheStart),
//     PtrToUL64(&MmSystemCacheEnd),
//     PtrToUL64(&MmSystemCacheWs),
//     PtrToUL64(&MmPfnDatabase),
//     PtrToUL64(MmSystemPtesStart),
//     PtrToUL64(MmSystemPtesEnd),
//     PtrToUL64(&MmSubsectionBase),
//     PtrToUL64(&MmNumberOfPagingFiles),
//     PtrToUL64(&MmLowestPhysicalPage),
//     PtrToUL64(&MmHighestPhysicalPage),
//     PtrToUL64(&MmNumberOfPhysicalPages),
//     PtrToUL64(&MmMaximumNonPagedPoolInBytes),
//     PtrToUL64(&MmNonPagedSystemStart),
//     PtrToUL64(&MmNonPagedPoolStart),
//     PtrToUL64(&MmNonPagedPoolEnd),
//     PtrToUL64(&MmPagedPoolStart),
//     PtrToUL64(&MmPagedPoolEnd),
//     PtrToUL64(&MmPagedPoolInfo),
//     PAGE_SIZE,
//     PtrToUL64(&MmSizeOfPagedPoolInBytes),
//     PtrToUL64(&MmTotalCommitLimit),
//     PtrToUL64(&MmTotalCommittedPages),
//     PtrToUL64(&MmSharedCommit),
//     PtrToUL64(&MmDriverCommit),
//     PtrToUL64(&MmProcessCommit),
//     PtrToUL64(&MmPagedPoolCommit),
//     PtrToUL64(0),
//     PtrToUL64(&MmZeroedPageListHead),
//     PtrToUL64(&MmFreePageListHead),
//     PtrToUL64(&MmStandbyPageListHead),
//     PtrToUL64(&MmModifiedPageListHead),
//     PtrToUL64(&MmModifiedNoWritePageListHead),
//     PtrToUL64(&MmAvailablePages),
//     PtrToUL64(&MmResidentAvailablePages),
//     PtrToUL64(&PoolTrackTable),
//     PtrToUL64(&NonPagedPoolDescriptor),
//     PtrToUL64(&MmHighestUserAddress),
//     PtrToUL64(&MmSystemRangeStart),
//     PtrToUL64(&MmUserProbeAddress),
//     PtrToUL64(KdPrintDefaultCircularBuffer),
//     PtrToUL64(KdPrintDefaultCircularBuffer + sizeof(KdPrintDefaultCircularBuffer)),
//     PtrToUL64(&KdPrintWritePointer),
//     PtrToUL64(&KdPrintRolloverCount),
//     PtrToUL64(&MmLoadedUserImageList),
//     PtrToUL64(&NtBuildLab),
//     PtrToUL64(0),
//     PtrToUL64(KiProcessorBlock),
//     PtrToUL64(&MmUnloadedDrivers),
//     PtrToUL64(&MmLastUnloadedDrivers),
//     PtrToUL64(&MmTriageActionTaken),
//     PtrToUL64(&MmSpecialPoolTag),
//     PtrToUL64(&KernelVerifier),
//     PtrToUL64(&MmVerifierData),
//     PtrToUL64(&MmAllocatedNonPagedPool),
//     PtrToUL64(&MmPeakCommitment),
//     PtrToUL64(&MmtotalCommitLimitMaximum),
//     PtrToUL64(&CmNtCSDVersion),
//     PtrToUL64(&MmPhysicalMemoryBlock),
//     PtrToUL64(&MmSessionBase),
//     PtrToUL64(&MmSessionSize),
//     PtrToUL64(0),
//     PtrToUL64(0),
//     FIELD_OFFSET(KTHREAD, NextProcessor),
//     FIELD_OFFSET(KTHREAD, Teb),
//     FIELD_OFFSET(KTHREAD, KernelStack),
//     FIELD_OFFSET(KTHREAD, InitialStack),
//     FIELD_OFFSET(KTHREAD, ApcState.Process),
//     FIELD_OFFSET(KTHREAD, State),
//     0,
//     0,
//     sizeof(_EPROCESS),
//     FIELD_OFFSET(_EPROCESS, Peb),
//     FIELD_OFFSET(_EPROCESS, InheritedFromUniqueProcessId),
//     FIELD_OFFSET(_EPROCESS, Pcb.DirectoryTableBase),
//     sizeof(KPRCB),
//     FIELD_OFFSET(KPRCB, DpcRoutineActive),
//     FIELD_OFFSET(KPRCB, CurrentThread),
//     FIELD_OFFSET(KPRCB, MHz),
//     FIELD_OFFSET(KPRCB, CpuType),
//     FIELD_OFFSET(KPRCB, VendorString),
//     FIELD_OFFSET(KPRCB, ProcessorState.ContextFrame),
//     FIELD_OFFSET(KPRCB, Number),
//     sizeof(_ETHREAD),
//     PtrToUL64(&KdPrintCircularBuffer),
//     PtrToUL64(&KdPrintBufferSize),
//     PtrToUL64(&KeLoaderBlock),
//     sizeof(KPCR),
//     KPCR_SELF_PCR_OFFSET,
//     KPCR_CURRENT_PRCB_OFFSET,
//     KPCR_CONTAINED_PRCB_OFFSET,
//     0,
//     0,
// #if defined(_M_ARM)
//     _WARN("KPCR_INITIAL_STACK_OFFSET, KPCR_STACK_LIMIT_OFFSET and KPRCB_PCR_PAGE_OFFSET not properly defined on ARM")
//     0,
//     0,
//     0,
// #else
//     KPCR_INITIAL_STACK_OFFSET,
//     KPCR_STACK_LIMIT_OFFSET,
//     KPRCB_PCR_PAGE_OFFSET,
// #endif
//     FIELD_OFFSET(KPRCB, ProcessorState.SpecialRegisters),
// #if defined(_M_IX86)
//     //
//     // x86 GDT/LDT/TSS constants
//     //
//     KGDT_R0_CODE,
//     KGDT_R0_DATA,
//     KGDT_R0_PCR,
//     KGDT_R3_CODE,
//     KGDT_R3_DATA,
//     KGDT_R3_TEB,
//     KGDT_LDT,
//     KGDT_TSS,
//     0,
//     0,
// #elif defined(_M_AMD64)
//     //
//     // AMD64 GDT/LDT/TSS constants
//     //
//     KGDT64_R0_CODE,
//     KGDT64_R3_DATA,
//     KGDT64_R3_DATA,
//     KGDT64_R3_CODE,
//     KGDT64_R3_DATA,
//     KGDT64_R3_DATA,
//     0,
//     KGDT64_SYS_TSS,
//     0,
//     0,
// #else
//     //
//     // No GDT/LDT/TSS on other architectures
//     //
//     0,
//     0,
//     0,
//     0,
//     0,
//     0,
//     0,
//     0,
//     0,
//     0,
// #endif
//     PtrToUL64(&IopNumTriageDumpDataBlocks),
//     PtrToUL64(IopTriageDumpDataBlocks),
//
// #if (NTDDI_VERSION >= NTDDI_LONGHORN)
// #error KdDebuggerDataBlock requires other fields for this NT version!
// #endif
// };
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
