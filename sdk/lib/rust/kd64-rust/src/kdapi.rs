use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
// ke globals referenced across the crate boundary as link symbols
// (all #[no_mangle] in ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeNumberProcessors: core::ffi::c_char;
    static mut KiProcessorBlock: [ntkrnl_sys::ke::PKPRCB; ntkrnl_sys::constants::MAXIMUM_PROCESSORS as usize];
}
use crate::kdtrap::KdpStub;

use ntkrnl_sys::ddk::{
    CONTEXT,
    DBGKD_ANY_CONTROL_REPORT,
    DBGKD_ANY_WAIT_STATE_CHANGE,
    DBGKD_COMMAND_STRING,
    DBGKD_GET_VERSION64,
    DBGKD_MANIPULATE_STATE64,
    EXCEPTION_RECORD, EXCEPTION_RECORD64,
    ExAcquireTimeRefreshLock,
    ExGetPreviousMode,
    ExLockUserBuffer,
    ExQueueWorkItem,
    ExReleaseTimeRefreshLock,
    ExUnlockUserBuffer,
    ExUpdateSystemTimeFromCmos,
    HalReturnToFirmware,
    KDSTATUS,
    KPROCESSOR_MODE,
    KeBugCheck,
    KeFreezeExecution,
    KeGetCurrentProcessorNumber,
    KeInsertQueueDpc,
    KeQueryPerformanceCounter,
    KeRosDumpStackFrames,
    KeSetTimer,
    KeSweepICache,
    KeThawExecution,
    KeTryToAcquireSpinLockAtDpcLevel,
    LARGE_INTEGER,
    MmDbgCopyMemory,
    MmDumpArmPfnDatabase,
    MmIsSessionAddress,
    SeSinglePrivilegeCheck,
    PCONTEXT,
    PDBGKD_ANY_WAIT_STATE_CHANGE,
    PDBGKD_CONTEXT_EX,
    PDBGKD_GET_SET_BUS_DATA,
    PDBGKD_GET_VERSION64,
    PDBGKD_MANIPULATE_STATE64,
    PDBGKD_QUERY_MEMORY,
    PDBGKD_READ_MEMORY64,
    PDBGKD_READ_WRITE_IO64,
    PDBGKD_READ_WRITE_IO_EXTENDED64,
    PDBGKD_READ_WRITE_MSR,
    PDBGKD_RESTORE_BREAKPOINT,
    PDBGKD_WRITE_BREAKPOINT64,
    PDBGKD_WRITE_MEMORY64,
    PEXCEPTION_RECORD,
    PKDPC,
    PKDEBUG_ROUTINE,
    PKD_SYMBOLS_INFO,
    PKPRCB,
    PMDL,
    PSTRING,
    _WORK_QUEUE_TYPE_DelayedWorkQueue,
    _LOCK_OPERATION_IoReadAccess,
    _LOCK_OPERATION_IoWriteAccess,
    _DEVICE_POWER_STATE_PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2,
    _DEVICE_POWER_STATE_PowerDeviceD3,
    DEVICE_POWER_STATE,
    _FIRMWARE_REENTRY_HalRebootRoutine,
    SYSDBG_COMMAND,
    SYSDBG_BUS_DATA,
    SYSDBG_CONTROL_SPACE,
    SYSDBG_IO_SPACE,
    SYSDBG_MSR,
    SYSDBG_PHYSICAL,
    SYSDBG_VIRTUAL,
    PSYSDBG_VIRTUAL,
    PSYSDBG_PHYSICAL,
    PSYSDBG_CONTROL_SPACE,
    PSYSDBG_IO_SPACE,
    PSYSDBG_MSR,
    PSYSDBG_BUS_DATA,
    KD_OPTION,
    PULONG_PTR,
    PBOOLEAN,
    ULARGE_INTEGER,
    STRING,
    PCHAR,
    ULONG_PTR,
    SIZE_T,
    KTIMER,
    LUID,
    ULONG64,
    WORK_QUEUE_ITEM,
    _KPRCB,
    KPROCESSOR_STATE,
    LIST_ENTRY,
    KDPC,
    KD_CONTEXT,
    PKTRAP_FRAME,
    PKEXCEPTION_FRAME,
};

extern "C" {
    static mut KdpContext: KD_CONTEXT;
    static mut KdpMessageBuffer: [u8; 0];
    static mut KdpPathBuffer: [u8; 0];
    static mut KdDebuggerNotPresent: BOOLEAN;
    static mut KdVersionBlock: DBGKD_GET_VERSION64;
    static mut KdpContextSent: BOOLEAN;
    static mut KeProcessorLevel: USHORT;
    static mut KdDisableCount: ULONG;
    static mut KdPreviouslyEnabled: BOOLEAN;
    static mut KdDebuggerEnabled: BOOLEAN;
    static mut SharedUserData: *mut PVOID;
    static mut KdPitchDebugger: BOOLEAN;
    static mut KdBlockEnable: BOOLEAN;
    static mut KiDebugRoutine: PKDEBUG_ROUTINE;
    static mut KdpDebuggerLock: KSPIN_LOCK;
    static mut KdpPortLocked: BOOLEAN;
    static mut KdEnteredDebugger: BOOLEAN;
    static mut KiFreezeFlag: ULONG;
    static mut KdTimerStart: LARGE_INTEGER;
    static mut KdTimerStop: LARGE_INTEGER;
    static mut KdTimerDifference: LARGE_INTEGER;
    static mut KdpTimeSlipPending: LONG;
    static mut KdpTimeSlipEvent: PVOID;
    static mut KdpTimeSlipEventLock: KSPIN_LOCK;
    static mut KdpTimeSlipTimer: KTIMER;
    static mut KdpTimeSlipDpc: KDPC;
    static mut KdpTimeSlipWorkItem: WORK_QUEUE_ITEM;
    static mut KdpDebuggerStructuresInitialized: BOOLEAN;
    static mut KdComponentTableSize: ULONG;
    static mut KdComponentTable: [PULONG; 0];
    static mut Kd_WIN2000_Mask: ULONG;
    static mut Kd_DEFAULT_Mask: ULONG;
    static mut KdpNumInternalBreakpoints: ULONG;
    static mut TraceDataBuffer: [ULONG; 0];
    static mut TraceDataBufferPosition: ULONG;
    static mut MmHighestUserAddress: PVOID;
    static mut SeDebugPrivilege: LUID;
}

extern "system" {
    fn KdSendPacket(packet_type: ULONG, header: PSTRING, data: PSTRING, context: *mut KD_CONTEXT);
    fn KdReceivePacket(packet_type: ULONG, header: PSTRING, data: PSTRING, length: *mut ULONG, context: *mut KD_CONTEXT) -> KDSTATUS;
    fn KdpAddBreakpoint(address: PVOID) -> ULONG;
    fn KdpDeleteBreakpoint(handle: ULONG) -> BOOLEAN;
    fn KdpDeleteBreakpointRange(start: PVOID, end: PVOID) -> BOOLEAN;
    fn KdpSysReadControlSpace(processor: ULONG, address: ULONG64, buffer: PVOID, size: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysWriteControlSpace(processor: ULONG, address: ULONG64, buffer: PVOID, size: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysReadMsr(msr: ULONG, value: *mut ULONGLONG) -> NTSTATUS;
    fn KdpSysWriteMsr(msr: ULONG, value: *mut ULONGLONG) -> NTSTATUS;
    fn KdpSysReadBusData(bus_type: ULONG, bus_number: ULONG, slot_number: ULONG, offset: ULONG, buffer: PVOID, length: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysWriteBusData(bus_type: ULONG, bus_number: ULONG, slot_number: ULONG, offset: ULONG, buffer: PVOID, length: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysReadIoSpace(interface_type: ULONG, bus_number: ULONG, address_space: ULONG, address: ULONG64, buffer: PVOID, size: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysWriteIoSpace(interface_type: ULONG, bus_number: ULONG, address_space: ULONG, address: ULONG64, buffer: PVOID, size: ULONG, actual: *mut ULONG) -> NTSTATUS;
    fn KdpSysCheckLowMemory(flags: ULONG) -> NTSTATUS;
    fn KdpGetStateChange(state: PDBGKD_MANIPULATE_STATE64, context: PCONTEXT);
    fn KdpSetContextState(state: PDBGKD_ANY_WAIT_STATE_CHANGE, context: PCONTEXT);
    fn KdSave(flag: BOOLEAN);
    fn KdRestore(flag: BOOLEAN);
    fn KdpPortUnlock();
    fn KdInitSystem(phase: ULONG, loader_block: PVOID) -> BOOLEAN;
    fn KdpRestoreAllBreakpoints();
    fn KdpAllowDisable() -> NTSTATUS;
    fn KdpSuspendAllBreakPoints();
    fn KdD0Transition();
    fn KdD3Transition();
    fn KxSwitchKdProcessor(index: ULONG) -> ULONG;
    fn KeGetCurrentThread() -> PVOID;
    fn KdpPortLock();
}
extern "C" {
    fn ExceptionRecord32To64(ex32: *mut PVOID, ex64: *mut EXCEPTION_RECORD64);
    // ke.h takes the trap frame by ONE pointer (a macro on arm64, FORCEINLINE
    // elsewhere). This file declared it, KdpQueryPerformanceCounter and
    // KdEnterDebugger all one level too deep, consistently — which is why it
    // compiled and why nothing but a decl-vs-def comparison could see it.
    fn KeGetTrapFrameInterruptState(trap_frame: PVOID) -> BOOLEAN;
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}
extern "C" {
    fn KeGetContextPc(context: PCONTEXT) -> ULONG_PTR;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
extern "system" {
    fn PspDumpThreads(system: BOOLEAN);
}

const PACKET_TYPE_KD_STATE_MANIPULATE: ULONG = 2;
const PACKET_TYPE_KD_DEBUG_IO: ULONG = 3;
const PACKET_TYPE_KD_STATE_CHANGE64: ULONG = 7;
const PACKET_MAX_SIZE: ULONG = 4000;
const DBGKD_MAXSTREAM: ULONG = 16;
const MMDBG_COPY_UNSAFE: ULONG = 0x00000001;
const MMDBG_COPY_WRITE: ULONG = 0x00000002;
const MMDBG_COPY_PHYSICAL: ULONG = 0x00000004;
const MMDBG_COPY_CACHED: ULONG = 0x00000008;
const MMDBG_COPY_UNCACHED: ULONG = 0x00000010;
const MMDBG_COPY_WRITE_COMBINED: ULONG = 0x00000020;
const MMDBG_COPY_MAX_SIZE: ULONG = 4096;

const DBGKD_QUERY_MEMORY_VIRTUAL: ULONG = 0;
const DBGKD_QUERY_MEMORY_PROCESS: ULONG = 0;
const DBGKD_QUERY_MEMORY_SESSION: ULONG = 1;
const DBGKD_QUERY_MEMORY_KERNEL: ULONG = 2;
const DBGKD_QUERY_MEMORY_READ: ULONG = 1;
const DBGKD_QUERY_MEMORY_WRITE: ULONG = 2;
const DBGKD_QUERY_MEMORY_EXECUTE: ULONG = 4;

const DBGKD_CACHING_CACHED: ULONG = 1;
const DBGKD_CACHING_UNCACHED: ULONG = 2;
const DBGKD_CACHING_WRITE_COMBINED: ULONG = 3;

const DbgKdReadVirtualMemoryApi: ULONG = 0x00003130;
const DbgKdWriteVirtualMemoryApi: ULONG = 0x00003131;
const DbgKdGetContextApi: ULONG = 0x00003132;
const DbgKdSetContextApi: ULONG = 0x00003133;
const DbgKdWriteBreakPointApi: ULONG = 0x00003134;
const DbgKdRestoreBreakPointApi: ULONG = 0x00003135;
const DbgKdContinueApi: ULONG = 0x00003136;
const DbgKdReadControlSpaceApi: ULONG = 0x00003137;
const DbgKdWriteControlSpaceApi: ULONG = 0x00003138;
const DbgKdReadIoSpaceApi: ULONG = 0x00003139;
const DbgKdWriteIoSpaceApi: ULONG = 0x0000313A;
const DbgKdRebootApi: ULONG = 0x0000313B;
const DbgKdContinueApi2: ULONG = 0x0000313C;
const DbgKdReadPhysicalMemoryApi: ULONG = 0x0000313D;
const DbgKdWritePhysicalMemoryApi: ULONG = 0x0000313E;
const DbgKdQuerySpecialCallsApi: ULONG = 0x0000313F;
const DbgKdSetSpecialCallApi: ULONG = 0x00003140;
const DbgKdClearSpecialCallsApi: ULONG = 0x00003141;
const DbgKdSetInternalBreakPointApi: ULONG = 0x00003142;
const DbgKdGetInternalBreakPointApi: ULONG = 0x00003143;
const DbgKdReadIoSpaceExtendedApi: ULONG = 0x00003144;
const DbgKdWriteIoSpaceExtendedApi: ULONG = 0x00003145;
const DbgKdGetVersionApi: ULONG = 0x00003146;
const DbgKdWriteBreakPointExApi: ULONG = 0x00003147;
const DbgKdRestoreBreakPointExApi: ULONG = 0x00003148;
const DbgKdCauseBugCheckApi: ULONG = 0x00003149;
const DbgKdSwitchProcessor: ULONG = 0x00003150;
const DbgKdPageInApi: ULONG = 0x00003151;
const DbgKdReadMachineSpecificRegister: ULONG = 0x00003152;
const DbgKdWriteMachineSpecificRegister: ULONG = 0x00003153;
const DbgKdSearchMemoryApi: ULONG = 0x00003156;
const DbgKdGetBusDataApi: ULONG = 0x00003157;
const DbgKdSetBusDataApi: ULONG = 0x00003158;
const DbgKdCheckLowMemoryApi: ULONG = 0x00003159;
const DbgKdClearAllInternalBreakpointsApi: ULONG = 0x0000315A;
const DbgKdFillMemoryApi: ULONG = 0x0000315B;
const DbgKdQueryMemoryApi: ULONG = 0x0000315C;
const DbgKdSwitchPartition: ULONG = 0x0000315D;
const DbgKdWriteCustomBreakpointApi: ULONG = 0x0000315E;
const DbgKdGetContextExApi: ULONG = 0x0000315F;
const DbgKdSetContextExApi: ULONG = 0x00003160;

const DbgKdExceptionStateChange: ULONG = 0x00003030;
const DbgKdLoadSymbolsStateChange: ULONG = 0x00003031;
const DbgKdCommandStringStateChange: ULONG = 0x00003032;

const ContinueError: ULONG = 0;
const ContinueSuccess: ULONG = 1;
const ContinueProcessorReselected: ULONG = 2;

const KdPacketReceived: KDSTATUS = 0;
const KdPacketTimedOut: KDSTATUS = 1;
const KdPacketNeedsResend: KDSTATUS = 2;

const MANUALLY_INITIATED_CRASH: ULONG = 0xE2;

const SysDbgQueryVersion: ULONG = 7; // header value; the hand copy said 0
const SysDbgReadVirtual: ULONG = 8;
const SysDbgWriteVirtual: ULONG = 9;
const SysDbgReadPhysical: ULONG = 10;
const SysDbgWritePhysical: ULONG = 11;
const SysDbgReadControlSpace: ULONG = 12;
const SysDbgWriteControlSpace: ULONG = 13;
const SysDbgReadIoSpace: ULONG = 14;
const SysDbgWriteIoSpace: ULONG = 15;
const SysDbgReadMsr: ULONG = 16;
const SysDbgWriteMsr: ULONG = 17;
const SysDbgReadBusData: ULONG = 18;
const SysDbgWriteBusData: ULONG = 19;
const SysDbgCheckLowMemory: ULONG = 20;

const ROS_INTERNAL_COMMAND: ULONG = u32::from_be_bytes(*b" soR");
const ROS_DUMP_STACK_FRAMES_COMMAND: ULONG = u32::from_be_bytes(*b"DsoR");

const KD_OPTION_SET_BLOCK_ENABLE: ULONG = 0;

const Isa: ULONG = 1;

pub(crate) unsafe fn KdpMoveMemory(destination: PVOID, source: PVOID, length: usize) {
    let mut dest_bytes = destination as *mut u8;
    let mut src_bytes = source as *mut u8;
    let mut len = length;
    while len > 0 {
        *dest_bytes = *src_bytes;
        dest_bytes = dest_bytes.add(1);
        src_bytes = src_bytes.add(1);
        len -= 1;
    }
}

unsafe fn KdpZeroMemory(destination: PVOID, length: usize) {
    let mut dest_bytes = destination as *mut u8;
    let mut len = length;
    while len > 0 {
        *dest_bytes = 0;
        dest_bytes = dest_bytes.add(1);
        len -= 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpCopyMemoryChunks(
    address: ULONG64,
    buffer: PVOID,
    total_size: ULONG,
    chunk_size: ULONG,
    flags: ULONG,
    actual_size: *mut ULONG,
) -> NTSTATUS {
    let buffer = opaque_ptr(buffer);
    let actual_size = opaque_ptr(actual_size);
    let mut chunk = chunk_size;
    if chunk == 0 {
        chunk = 4;
    } else if chunk > MMDBG_COPY_MAX_SIZE {
        chunk = MMDBG_COPY_MAX_SIZE;
    }

    let mut remaining = total_size;
    let mut copy_chunk = 1u32;
    let mut addr = address;
    let mut buf = buffer;
    let mut status = STATUS_SUCCESS;

    while remaining > 0 {
        while (copy_chunk * 2) <= remaining
            && copy_chunk < chunk
            && ((addr & ((copy_chunk * 2) as u64 - 1)) == 0)
        {
            copy_chunk *= 2;
        }

        while copy_chunk > remaining {
            copy_chunk /= 2;
        }

        status = MmDbgCopyMemory(addr, buf, copy_chunk, flags);
        if status < 0 {
            break;
        }

        addr = addr + copy_chunk as u64;
        buf = (buf as ULONG_PTR + copy_chunk as ULONG_PTR) as PVOID;
        remaining = remaining - copy_chunk;
    }

    KeSweepICache(addr as PVOID, total_size as SIZE_T);

    if !actual_size.is_null() {
        *actual_size = total_size - remaining;
    }
    if remaining == 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

unsafe fn KdpQueryMemory(state: PDBGKD_MANIPULATE_STATE64, _context: PCONTEXT) {
    let memory = core::ptr::addr_of_mut!((*state).u.QueryMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut status = STATUS_SUCCESS;

    if (*memory).AddressSpace == DBGKD_QUERY_MEMORY_VIRTUAL {
        if ((*memory).Address as PVOID) < MmHighestUserAddress {
            (*memory).AddressSpace = DBGKD_QUERY_MEMORY_PROCESS;
        } else {
            if MmIsSessionAddress((*memory).Address as PVOID) != 0 {
                (*memory).AddressSpace = DBGKD_QUERY_MEMORY_SESSION;
            } else {
                (*memory).AddressSpace = DBGKD_QUERY_MEMORY_KERNEL;
            }
        }

        (*memory).Flags = DBGKD_QUERY_MEMORY_READ | DBGKD_QUERY_MEMORY_WRITE | DBGKD_QUERY_MEMORY_EXECUTE;
    } else {
        status = STATUS_INVALID_PARAMETER;
    }

    (*state).ReturnStatus = status;
    (*memory).Reserved = 0;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpSearchMemory(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // crate::kdp_dprintf!(b"Memory Search support is unimplemented!\n\0");

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;
    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpFillMemory(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // crate::kdp_dprintf!(b"Memory Fill support is unimplemented!\n\0");

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;
    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpWriteBreakpoint(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let breakpoint = core::ptr::addr_of_mut!((*state).u.WriteBreakPoint);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*breakpoint).BreakPointHandle = KdpAddBreakpoint((*breakpoint).BreakPointAddress as PVOID);
    if (*breakpoint).BreakPointHandle == 0 {
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    } else {
        (*state).ReturnStatus = STATUS_SUCCESS;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpRestoreBreakpoint(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let restore_bp = core::ptr::addr_of_mut!((*state).u.RestoreBreakPoint);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if KdpDeleteBreakpoint((*restore_bp).BreakPointHandle) != 0 {
        (*state).ReturnStatus = STATUS_SUCCESS;
    } else {
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpWriteBreakPointEx(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) -> NTSTATUS {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // crate::kdp_dprintf!(b"Extended Breakpoint Write support is unimplemented!\n\0");

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;
    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
    STATUS_UNSUCCESSFUL
}

unsafe fn KdpRestoreBreakPointEx(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // crate::kdp_dprintf!(b"Extended Breakpoint Restore support is unimplemented!\n\0");

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;
    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpWriteCustomBreakpoint(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    // crate::kdp_dprintf!(b"Custom Breakpoint Write is unimplemented\n\0");

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;
    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn DumpTraceData(trace_data: PSTRING) {
    let buffer_ptr = core::ptr::addr_of!(TraceDataBuffer) as *mut ULONG;
    *buffer_ptr = TraceDataBufferPosition;

    (*trace_data).Length = (TraceDataBufferPosition * core::mem::size_of::<ULONG>() as u32) as u16;
    (*trace_data).Buffer = buffer_ptr as PCHAR;

    TraceDataBufferPosition = 1;
}

unsafe fn KdpSetCommonState(
    new_state: ULONG,
    context: PCONTEXT,
    wait_state_change: PDBGKD_ANY_WAIT_STATE_CHANGE,
) {
    let mut instruction_count = 0u32;

    (*wait_state_change).NewState = new_state;
    (*wait_state_change).ProcessorLevel = KeProcessorLevel;
    (*wait_state_change).Processor = (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as u16;
    (*wait_state_change).NumberProcessors = KeNumberProcessors as ULONG;
    (*wait_state_change).Thread = KeGetCurrentThread() as u64;
    (*wait_state_change).ProgramCounter = KeGetContextPc(context) as u64;

    KdpZeroMemory(
        core::ptr::addr_of_mut!((*wait_state_change).__bindgen_anon_1.AnyControlReport) as PVOID,
        core::mem::size_of::<DBGKD_ANY_CONTROL_REPORT>(),
    );

    KdpCopyMemoryChunks(
        (*wait_state_change).ProgramCounter,
        core::ptr::addr_of_mut!((*wait_state_change).__bindgen_anon_1.ControlReport.InstructionStream) as PVOID,
        DBGKD_MAXSTREAM,
        0,
        MMDBG_COPY_UNSAFE,
        &mut instruction_count,
    );
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*wait_state_change).__bindgen_anon_1.ControlReport.InstructionCount = instruction_count as u16; }
    #[cfg(target_arch = "aarch64")]
    { (*wait_state_change).__bindgen_anon_1.ControlReport.InstructionCount = instruction_count; }

    let had_breakpoints = KdpDeleteBreakpointRange(
        (*wait_state_change).ProgramCounter as PVOID,
        ((*wait_state_change).ProgramCounter + (*wait_state_change).__bindgen_anon_1.ControlReport.InstructionCount as u64 - 1) as PVOID,
    );
    if had_breakpoints != 0 {
        KdpCopyMemoryChunks(
            (*wait_state_change).ProgramCounter,
            core::ptr::addr_of_mut!((*wait_state_change).__bindgen_anon_1.ControlReport.InstructionStream) as PVOID,
            instruction_count,
            0,
            MMDBG_COPY_UNSAFE,
            core::ptr::null_mut(),
        );
    }
}

unsafe fn KdpSysGetVersion(version: PDBGKD_GET_VERSION64) {
    KdpMoveMemory(
        version as PVOID,
        &raw const KdVersionBlock as *const DBGKD_GET_VERSION64 as PVOID,
        core::mem::size_of::<DBGKD_GET_VERSION64>(),
    );
}

unsafe fn KdpGetVersion(state: PDBGKD_MANIPULATE_STATE64) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    KdpSysGetVersion(core::ptr::addr_of_mut!((*state).u.GetVersion64));

    (*state).ApiNumber = DbgKdGetVersionApi;
    (*state).ReturnStatus = STATUS_SUCCESS;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpReadVirtualMemory(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let read_memory = core::ptr::addr_of_mut!((*state).u.ReadMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut length = (*read_memory).TransferCount;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if length > (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32) {
        length = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32;
    }

    (*state).ReturnStatus = KdpCopyMemoryChunks(
        (*read_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        length,
        0,
        MMDBG_COPY_UNSAFE,
        &mut length,
    );

    (*read_memory).ActualBytesRead = length;
    (*data).Length = length as u16;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpWriteVirtualMemory(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let write_memory = core::ptr::addr_of_mut!((*state).u.WriteMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpCopyMemoryChunks(
        (*write_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        (*data).Length as u32,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        &mut (*write_memory).ActualBytesWritten,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpReadPhysicalMemory(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let read_memory = core::ptr::addr_of_mut!((*state).u.ReadMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut length = (*read_memory).TransferCount;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if length > (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32) {
        length = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32;
    }

    let mut flags = MMDBG_COPY_UNSAFE | MMDBG_COPY_PHYSICAL;

    let cache_flags = (*read_memory).ActualBytesRead;
    if cache_flags == DBGKD_CACHING_CACHED {
        flags |= MMDBG_COPY_CACHED;
    } else if cache_flags == DBGKD_CACHING_UNCACHED {
        flags |= MMDBG_COPY_UNCACHED;
    } else if cache_flags == DBGKD_CACHING_WRITE_COMBINED {
        flags |= MMDBG_COPY_WRITE_COMBINED;
    }

    (*state).ReturnStatus = KdpCopyMemoryChunks(
        (*read_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        length,
        0,
        flags,
        &mut length,
    );

    (*read_memory).ActualBytesRead = length;
    (*data).Length = length as u16;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpWritePhysicalMemory(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let write_memory = core::ptr::addr_of_mut!((*state).u.WriteMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    let mut flags = MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE | MMDBG_COPY_PHYSICAL;

    let cache_flags = (*write_memory).ActualBytesWritten;
    if cache_flags == DBGKD_CACHING_CACHED {
        flags |= MMDBG_COPY_CACHED;
    } else if cache_flags == DBGKD_CACHING_UNCACHED {
        flags |= MMDBG_COPY_UNCACHED;
    } else if cache_flags == DBGKD_CACHING_WRITE_COMBINED {
        flags |= MMDBG_COPY_WRITE_COMBINED;
    }

    (*state).ReturnStatus = KdpCopyMemoryChunks(
        (*write_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        (*data).Length as u32,
        0,
        flags,
        &mut (*write_memory).ActualBytesWritten,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpReadControlSpace(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let read_memory = core::ptr::addr_of_mut!((*state).u.ReadMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut length = (*read_memory).TransferCount;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if length > (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32) {
        length = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32;
    }

    (*state).ReturnStatus = KdpSysReadControlSpace(
        (*state).Processor as ULONG,
        (*read_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        length,
        &mut length,
    );

    (*read_memory).ActualBytesRead = length;
    (*data).Length = length as u16;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpWriteControlSpace(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let write_memory = core::ptr::addr_of_mut!((*state).u.WriteMemory);
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysWriteControlSpace(
        (*state).Processor as ULONG,
        (*write_memory).TargetBaseAddress,
        (*data).Buffer as PVOID,
        (*data).Length as u32,
        &mut (*write_memory).ActualBytesWritten,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpGetContext(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if ((*state).Processor as ULONG) < KeNumberProcessors as ULONG {
        let target_context = if (*state).Processor == (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as u16 {
            context
        } else {
            let prcb_ptr = (core::ptr::addr_of!(KiProcessorBlock) as *const *mut KPRCB).add((*state).Processor as usize);
            core::ptr::addr_of_mut!((*((*prcb_ptr) as *mut _KPRCB)).ProcessorState.ContextFrame)
        };

        KdpMoveMemory((*data).Buffer as PVOID, target_context as PVOID, core::mem::size_of::<CONTEXT>());
        (*data).Length = core::mem::size_of::<CONTEXT>() as u16;

        KdpContextSent = 1u8;

        (*state).ReturnStatus = STATUS_SUCCESS;
    } else {
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpSetContext(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if ((*state).Processor as ULONG) < KeNumberProcessors as ULONG && KdpContextSent != 0 {
        let target_context = if (*state).Processor == (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as u16 {
            context
        } else {
            let prcb_ptr = (core::ptr::addr_of!(KiProcessorBlock) as *const *mut KPRCB).add((*state).Processor as usize);
            core::ptr::addr_of_mut!((*((*prcb_ptr) as *mut _KPRCB)).ProcessorState.ContextFrame)
        };

        KdpMoveMemory(target_context as PVOID, (*data).Buffer as PVOID, core::mem::size_of::<CONTEXT>());

        (*state).ReturnStatus = STATUS_SUCCESS;
    } else {
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpGetContextEx(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let context_ex = core::ptr::addr_of_mut!((*state).u.ContextEx);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if ((*state).Processor as ULONG) < KeNumberProcessors as ULONG
        && ((*context_ex).Offset + (*context_ex).ByteCount) <= core::mem::size_of::<CONTEXT>() as u32
    {
        let target_context = if (*state).Processor == (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as u16 {
            context
        } else {
            let prcb_ptr = (core::ptr::addr_of!(KiProcessorBlock) as *const *mut KPRCB).add((*state).Processor as usize);
            core::ptr::addr_of_mut!((*((*prcb_ptr) as *mut _KPRCB)).ProcessorState.ContextFrame)
        };

        KdpMoveMemory(
            (*data).Buffer as PVOID,
            ((target_context as ULONG_PTR) + (*context_ex).Offset as ULONG_PTR) as PVOID,
            (*context_ex).ByteCount as usize,
        );

        (*data).Length = (*context_ex).ByteCount as u16;
        (*context_ex).BytesCopied = (*context_ex).ByteCount;

        KdpContextSent = 1u8;

        (*state).ReturnStatus = STATUS_SUCCESS;
    } else {
        (*context_ex).BytesCopied = 0;
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpSetContextEx(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let context_ex = core::ptr::addr_of_mut!((*state).u.ContextEx);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if ((*state).Processor as ULONG) < KeNumberProcessors as ULONG
        && (((*context_ex).Offset + (*context_ex).ByteCount) <= core::mem::size_of::<CONTEXT>() as u32)
        && (KdpContextSent != 0)
    {
        let target_context = if (*state).Processor == (*(KeGetCurrentPrcb() as *mut _KPRCB)).Number as u16 {
            context
        } else {
            let prcb_ptr = (core::ptr::addr_of!(KiProcessorBlock) as *const *mut KPRCB).add((*state).Processor as usize);
            core::ptr::addr_of_mut!((*((*prcb_ptr) as *mut _KPRCB)).ProcessorState.ContextFrame)
        };

        KdpMoveMemory(
            ((target_context as ULONG_PTR) + (*context_ex).Offset as ULONG_PTR) as PVOID,
            (*data).Buffer as PVOID,
            (*context_ex).ByteCount as usize,
        );

        (*context_ex).BytesCopied = (*context_ex).ByteCount;

        (*state).ReturnStatus = STATUS_SUCCESS;
    } else {
        (*context_ex).BytesCopied = 0;
        (*state).ReturnStatus = STATUS_UNSUCCESSFUL;
    }

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpCauseBugCheck(_state: PDBGKD_MANIPULATE_STATE64) {
    KeBugCheck(MANUALLY_INITIATED_CRASH);
}

unsafe fn KdpReadMachineSpecificRegister(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let read_msr = core::ptr::addr_of_mut!((*state).u.ReadWriteMsr);
    let mut msr_value = core::mem::zeroed::<ULARGE_INTEGER>();

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysReadMsr((*read_msr).Msr, core::ptr::addr_of_mut!(msr_value.QuadPart));

    (*read_msr).DataValueLow = msr_value.u.LowPart;
    (*read_msr).DataValueHigh = msr_value.u.HighPart;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpWriteMachineSpecificRegister(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let write_msr = core::ptr::addr_of_mut!((*state).u.ReadWriteMsr);
    let mut msr_value = core::mem::zeroed::<ULARGE_INTEGER>();

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    msr_value.u.LowPart = (*write_msr).DataValueLow;
    msr_value.u.HighPart = (*write_msr).DataValueHigh;
    (*state).ReturnStatus = KdpSysWriteMsr((*write_msr).Msr, core::ptr::addr_of_mut!(msr_value.QuadPart));

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpGetBusData(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let get_bus_data = core::ptr::addr_of_mut!((*state).u.GetSetBusData);
    let mut length = (*get_bus_data).Length;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    if length > (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32) {
        length = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u32;
    }

    (*state).ReturnStatus = KdpSysReadBusData(
        (*get_bus_data).BusDataType,
        (*get_bus_data).BusNumber,
        (*get_bus_data).SlotNumber,
        (*get_bus_data).Offset,
        (*data).Buffer as PVOID,
        length,
        &mut length,
    );

    (*get_bus_data).Length = length;
    (*data).Length = length as u16;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, data, &raw mut KdpContext);
}

unsafe fn KdpSetBusData(state: PDBGKD_MANIPULATE_STATE64, data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let set_bus_data = core::ptr::addr_of_mut!((*state).u.GetSetBusData);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysWriteBusData(
        (*set_bus_data).BusDataType,
        (*set_bus_data).BusNumber,
        (*set_bus_data).SlotNumber,
        (*set_bus_data).Offset,
        (*data).Buffer as PVOID,
        (*set_bus_data).Length,
        &mut (*set_bus_data).Length,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpReadIoSpace(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let read_io = core::ptr::addr_of_mut!((*state).u.ReadWriteIo);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*read_io).DataValue = 0;

    (*state).ReturnStatus = KdpSysReadIoSpace(
        Isa,
        0,
        1,
        (*read_io).IoAddress,
        core::ptr::addr_of_mut!((*read_io).DataValue) as PVOID,
        (*read_io).DataSize,
        &mut (*read_io).DataSize,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpWriteIoSpace(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let write_io = core::ptr::addr_of_mut!((*state).u.ReadWriteIo);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysWriteIoSpace(
        Isa,
        0,
        1,
        (*write_io).IoAddress,
        core::ptr::addr_of_mut!((*write_io).DataValue) as PVOID,
        (*write_io).DataSize,
        &mut (*write_io).DataSize,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpReadIoSpaceExtended(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let read_io_extended = core::ptr::addr_of_mut!((*state).u.ReadWriteIoExtended);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*read_io_extended).DataValue = 0;

    (*state).ReturnStatus = KdpSysReadIoSpace(
        (*read_io_extended).InterfaceType,
        (*read_io_extended).BusNumber,
        (*read_io_extended).AddressSpace,
        (*read_io_extended).IoAddress,
        core::ptr::addr_of_mut!((*read_io_extended).DataValue) as PVOID,
        (*read_io_extended).DataSize,
        &mut (*read_io_extended).DataSize,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpWriteIoSpaceExtended(state: PDBGKD_MANIPULATE_STATE64, _data: PSTRING, _context: PCONTEXT) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let write_io_extended = core::ptr::addr_of_mut!((*state).u.ReadWriteIoExtended);

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysWriteIoSpace(
        (*write_io_extended).InterfaceType,
        (*write_io_extended).BusNumber,
        (*write_io_extended).AddressSpace,
        (*write_io_extended).IoAddress,
        core::ptr::addr_of_mut!((*write_io_extended).DataValue) as PVOID,
        (*write_io_extended).DataSize,
        &mut (*write_io_extended).DataSize,
    );

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpCheckLowMemory(state: PDBGKD_MANIPULATE_STATE64) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    (*state).ReturnStatus = KdpSysCheckLowMemory(MMDBG_COPY_UNSAFE);

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpNotSupported(state: PDBGKD_MANIPULATE_STATE64) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    (*state).ReturnStatus = STATUS_UNSUCCESSFUL;

    header.Length = core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16;
    header.Buffer = state as PCHAR;

    KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, core::ptr::null_mut(), &raw mut KdpContext);
}

unsafe fn KdpSwitchProcessor(processor_index: USHORT) -> ULONG {
    if processor_index as u32 >= KeNumberProcessors as u32 {
        // crate::kdp_dprintf!(b"%u is not a valid processor number\n\0");
        return ContinueProcessorReselected;
    }

    if processor_index as u32 == KeGetCurrentProcessorNumber() {
        return ContinueProcessorReselected;
    }

    KxSwitchKdProcessor(processor_index as ULONG)
}

unsafe fn KdpSendWaitContinue(
    packet_type: ULONG,
    send_header: PSTRING,
    send_data: PSTRING,
    context: PCONTEXT,
) -> ULONG {
    let mut data = STRING {
        Length: 0,
        MaximumLength: 4096u16,
        Buffer: core::ptr::addr_of!(KdpMessageBuffer) as PCHAR,
    };
    let mut header = STRING {
        Length: 0,
        MaximumLength: core::mem::size_of::<DBGKD_MANIPULATE_STATE64>() as u16,
        Buffer: core::ptr::null_mut(),
    };
    let mut manipulate_state = core::mem::zeroed::<DBGKD_MANIPULATE_STATE64>();
    let mut length = 0u32;

    header.Buffer = core::ptr::addr_of_mut!(manipulate_state) as PCHAR;

    KdpContextSent = 0u8;

    // C control flow (kdapi.c `SendPacket:` label): the state-change packet is
    // sent ONCE, then an inner infinite loop receives + dispatches manipulate
    // requests; ONLY KdPacketNeedsResend jumps back to re-send the original
    // packet. (An earlier translation put the dispatch inside the send loop, so
    // every serviced API re-sent the state change with a fresh packet id —
    // WinDbg saw a new state change per request, desynced, and its memory reads
    // timed out ["Unable to read head of debugger data list, error 0n56"]; once
    // it stopped acking, exception-typed sends retried forever = boot wedge.)
    'send_packet: loop {
        KdSendPacket(packet_type, send_header, send_data, &raw mut KdpContext);

        if KdDebuggerNotPresent != 0 {
            return ContinueSuccess;
        }

        // Main processing loop: receive one manipulate request, dispatch it
        // (each handler sends its own response), receive the next.
        loop {
        let mut recv_code;
        loop {
            recv_code = KdReceivePacket(
                PACKET_TYPE_KD_STATE_MANIPULATE,
                &mut header,
                &mut data,
                &mut length,
                &raw mut KdpContext,
            );

            if recv_code == KdPacketNeedsResend {
                continue 'send_packet;
            }
            if recv_code != KdPacketTimedOut {
                break;
            }
        }

        match manipulate_state.ApiNumber {
            DbgKdReadVirtualMemoryApi => {
                KdpReadVirtualMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteVirtualMemoryApi => {
                KdpWriteVirtualMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdGetContextApi => {
                KdpGetContext(&mut manipulate_state, &mut data, context);
            }
            DbgKdSetContextApi => {
                KdpSetContext(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteBreakPointApi => {
                KdpWriteBreakpoint(&mut manipulate_state, &mut data, context);
            }
            DbgKdRestoreBreakPointApi => {
                KdpRestoreBreakpoint(&mut manipulate_state, &mut data, context);
            }
            DbgKdContinueApi => {
                // C: if (NT_SUCCESS(ContinueStatus)) return ContinueSuccess; else return ContinueError;
                // NT_SUCCESS is true for status >= 0
                return if manipulate_state.u.Continue.ContinueStatus >= 0 {
                    ContinueSuccess
                } else {
                    ContinueError
                };
            }
            DbgKdReadControlSpaceApi => {
                KdpReadControlSpace(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteControlSpaceApi => {
                KdpWriteControlSpace(&mut manipulate_state, &mut data, context);
            }
            DbgKdReadIoSpaceApi => {
                KdpReadIoSpace(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteIoSpaceApi => {
                KdpWriteIoSpace(&mut manipulate_state, &mut data, context);
            }
            DbgKdRebootApi => {
                HalReturnToFirmware(_FIRMWARE_REENTRY_HalRebootRoutine);
            }
            DbgKdContinueApi2 => {
                if manipulate_state.u.Continue2.ContinueStatus >= 0 {
                    KdpGetStateChange(&mut manipulate_state, context);
                    return ContinueSuccess;
                } else {
                    return ContinueError;
                }
            }
            DbgKdReadPhysicalMemoryApi => {
                KdpReadPhysicalMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdWritePhysicalMemoryApi => {
                KdpWritePhysicalMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdQuerySpecialCallsApi | DbgKdSetSpecialCallApi | DbgKdClearSpecialCallsApi => {
                // crate::kdp_dprintf!(b"Special Call support is unimplemented!\n\0");
                KdpNotSupported(&mut manipulate_state);
            }
            DbgKdSetInternalBreakPointApi | DbgKdGetInternalBreakPointApi => {
                // crate::kdp_dprintf!(b"Internal Breakpoint support is unimplemented!\n\0");
                KdpNotSupported(&mut manipulate_state);
            }
            DbgKdReadIoSpaceExtendedApi => {
                KdpReadIoSpaceExtended(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteIoSpaceExtendedApi => {
                KdpWriteIoSpaceExtended(&mut manipulate_state, &mut data, context);
            }
            DbgKdGetVersionApi => {
                KdpGetVersion(&mut manipulate_state);
            }
            DbgKdWriteBreakPointExApi => {
                if KdpWriteBreakPointEx(&mut manipulate_state, &mut data, context) < 0 {
                    return ContinueError;
                }
            }
            DbgKdRestoreBreakPointExApi => {
                KdpRestoreBreakPointEx(&mut manipulate_state, &mut data, context);
            }
            DbgKdCauseBugCheckApi => {
                KdpCauseBugCheck(&mut manipulate_state);
            }
            DbgKdSwitchProcessor => {
                return KdpSwitchProcessor(manipulate_state.Processor);
            }
            DbgKdPageInApi => {
                // crate::kdp_dprintf!(b"DbgKdPageInApi is obsolete!\n\0");
                KdpNotSupported(&mut manipulate_state);
            }
            DbgKdReadMachineSpecificRegister => {
                KdpReadMachineSpecificRegister(&mut manipulate_state, &mut data, context);
            }
            DbgKdWriteMachineSpecificRegister => {
                KdpWriteMachineSpecificRegister(&mut manipulate_state, &mut data, context);
            }
            DbgKdSearchMemoryApi => {
                KdpSearchMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdGetBusDataApi => {
                KdpGetBusData(&mut manipulate_state, &mut data, context);
            }
            DbgKdSetBusDataApi => {
                KdpSetBusData(&mut manipulate_state, &mut data, context);
            }
            DbgKdCheckLowMemoryApi => {
                KdpCheckLowMemory(&mut manipulate_state);
            }
            DbgKdClearAllInternalBreakpointsApi => {
                KdpNumInternalBreakpoints = 0;
            }
            DbgKdFillMemoryApi => {
                KdpFillMemory(&mut manipulate_state, &mut data, context);
            }
            DbgKdQueryMemoryApi => {
                KdpQueryMemory(&mut manipulate_state, context);
            }
            DbgKdSwitchPartition => {
                // crate::kdp_dprintf!(b"Partition Switch support is unimplemented!\n\0");
                KdpNotSupported(&mut manipulate_state);
            }
            DbgKdWriteCustomBreakpointApi => {
                KdpWriteCustomBreakpoint(&mut manipulate_state, &mut data, context);
            }
            DbgKdGetContextExApi => {
                KdpGetContextEx(&mut manipulate_state, &mut data, context);
            }
            DbgKdSetContextExApi => {
                KdpSetContextEx(&mut manipulate_state, &mut data, context);
            }
            _ => {
                // crate::kdp_dprintf!(b"Received Unrecognized API 0x%lx\n\0");

                data.Length = 0;
                manipulate_state.ReturnStatus = STATUS_UNSUCCESSFUL;

                KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE, &mut header, &mut data, &raw mut KdpContext);
            }
        }
        } // main processing loop
    }
}


#[no_mangle]
pub unsafe extern "system" fn KdpReportLoadSymbolsStateChange(
    path_name: PSTRING,
    symbol_info: PKD_SYMBOLS_INFO,
    unload: BOOLEAN,
    context: PCONTEXT,
) {
    let symbol_info = opaque_ptr(symbol_info);
    let mut extra_data: PSTRING;
    let mut data = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut wait_state_change = core::mem::zeroed::<DBGKD_ANY_WAIT_STATE_CHANGE>();
    let mut path_name_length = 0u32;
    let mut status;

    loop {
        KdpSetCommonState(DbgKdLoadSymbolsStateChange, context, &mut wait_state_change);

        KdpSetContextState(&mut wait_state_change, context);

        wait_state_change.u.LoadSymbols.UnloadSymbols = unload;
        wait_state_change.u.LoadSymbols.BaseOfDll = (*symbol_info).BaseOfDll as ULONG_PTR as ULONG64;
        wait_state_change.u.LoadSymbols.ProcessId = (*symbol_info).ProcessId as ULONG64;
        wait_state_change.u.LoadSymbols.CheckSum = (*symbol_info).CheckSum;
        wait_state_change.u.LoadSymbols.SizeOfImage = (*symbol_info).SizeOfImage;

        if !path_name.is_null() {
            KdpCopyMemoryChunks(
                (*path_name).Buffer as ULONG_PTR as u64,
                core::ptr::addr_of!(KdpPathBuffer) as PVOID,
                (*path_name).Length as u32,
                0,
                MMDBG_COPY_UNSAFE,
                &mut path_name_length,
            );

            *(core::ptr::addr_of!(KdpPathBuffer) as *mut u8).add(path_name_length as usize) = 0;
            path_name_length += 1;

            wait_state_change.u.LoadSymbols.PathNameLength = path_name_length;

            data.Buffer = core::ptr::addr_of!(KdpPathBuffer) as PCHAR;
            data.Length = path_name_length as u16;
            extra_data = &mut data;
        } else {
            wait_state_change.u.LoadSymbols.PathNameLength = 0;
            extra_data = core::ptr::null_mut();
        }

        header.Length = core::mem::size_of::<DBGKD_ANY_WAIT_STATE_CHANGE>() as u16;
        header.Buffer = core::ptr::addr_of_mut!(wait_state_change) as PCHAR;

        status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64, &mut header, extra_data, context);

        if status != ContinueProcessorReselected {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpReportCommandStringStateChange(
    name_string: PSTRING,
    command_string: PSTRING,
    context: PCONTEXT,
) {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut data = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut wait_state_change = core::mem::zeroed::<DBGKD_ANY_WAIT_STATE_CHANGE>();
    let mut length;
    let mut actual_length = 0u32;
    let mut total_length;
    let mut status;

    loop {
        KdpSetCommonState(DbgKdCommandStringStateChange, context, &mut wait_state_change);

        KdpSetContextState(&mut wait_state_change, context);

        KdpZeroMemory(
            core::ptr::addr_of_mut!(wait_state_change.u.CommandString) as PVOID,
            core::mem::size_of::<DBGKD_COMMAND_STRING>(),
        );

        length = if (*name_string).Length < 127 {
            (*name_string).Length as u32
        } else {
            127
        };

        KdpCopyMemoryChunks(
            (*name_string).Buffer as ULONG_PTR as u64,
            core::ptr::addr_of!(KdpMessageBuffer) as PVOID,
            length,
            0,
            MMDBG_COPY_UNSAFE,
            &mut actual_length,
        );

        total_length = actual_length;
        *(core::ptr::addr_of!(KdpMessageBuffer) as *mut u8).add(total_length as usize) = 0;
        total_length += 1;

        length = (*command_string).Length as u32;
        if length
            > (PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_ANY_WAIT_STATE_CHANGE>() as u32 - total_length)
        {
            length = PACKET_MAX_SIZE - core::mem::size_of::<DBGKD_ANY_WAIT_STATE_CHANGE>() as u32 - total_length;
        }

        KdpCopyMemoryChunks(
            (*command_string).Buffer as ULONG_PTR as u64,
            (core::ptr::addr_of!(KdpMessageBuffer) as *mut u8).add(total_length as usize) as PVOID,
            length,
            0,
            MMDBG_COPY_UNSAFE,
            &mut actual_length,
        );

        total_length += actual_length;
        *(core::ptr::addr_of!(KdpMessageBuffer) as *mut u8).add(total_length as usize) = 0;
        total_length += 1;

        header.Length = core::mem::size_of::<DBGKD_ANY_WAIT_STATE_CHANGE>() as u16;
        header.Buffer = core::ptr::addr_of_mut!(wait_state_change) as PCHAR;
        data.Length = total_length as u16;
        data.Buffer = core::ptr::addr_of!(KdpMessageBuffer) as PCHAR;

        status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64, &mut header, &mut data, context);

        if status != ContinueProcessorReselected {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpReportExceptionStateChange(
    exception_record: PEXCEPTION_RECORD,
    context: PCONTEXT,
    second_chance_exception: BOOLEAN,
) -> BOOLEAN {
    let mut header = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut data = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut wait_state_change = core::mem::zeroed::<DBGKD_ANY_WAIT_STATE_CHANGE>();
    let mut status;

    loop {
        KdpSetCommonState(DbgKdExceptionStateChange, context, &mut wait_state_change);

        #[cfg(not(target_pointer_width = "64"))]
        {
            ExceptionRecord32To64(
                exception_record as *mut PVOID,
                core::ptr::addr_of_mut!(wait_state_change.u.Exception.ExceptionRecord),
            );
        }

        #[cfg(target_pointer_width = "64")]
        {
            KdpMoveMemory(
                core::ptr::addr_of_mut!(wait_state_change.u.Exception.ExceptionRecord) as PVOID,
                exception_record as PVOID,
                core::mem::size_of::<EXCEPTION_RECORD>(),
            );
        }

        wait_state_change.u.Exception.FirstChance = if second_chance_exception != 0 { 0 } else { 1 };

        KdpSetContextState(&mut wait_state_change, context);

        header.Length = core::mem::size_of::<DBGKD_ANY_WAIT_STATE_CHANGE>() as u16;
        header.Buffer = core::ptr::addr_of_mut!(wait_state_change) as PCHAR;

        DumpTraceData(&mut data);

        status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64, &mut header, &mut data, context);

        if status != ContinueProcessorReselected {
            break;
        }
    }

    // vibeos robustness: if the send BAILED because no debugger is listening
    // (KdDebuggerNotPresent latched by KdSendPacket; a live exchange clears it
    // on the first received leader), we must NOT claim this exception handled —
    // "handled" resumes at the faulting instruction, which re-faults and
    // re-reports forever (observed: an endless DbgKdExceptionStateChange
    // stream at one PC). Returning FALSE lets normal dispatch proceed: kernel
    // SEH gets its shot, else KMODE_EXCEPTION_NOT_HANDLED — exactly what the
    // same exception does on an unarmed boot.
    if KdDebuggerNotPresent != 0 {
        return 0;
    }

    // C returns BOOLEAN: ContinueSuccess(1)→TRUE, ContinueError(0)→FALSE
    status as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn KdReportProcessorChange() -> ULONG {
    let current_prcb = KeGetCurrentPrcb();
    let context_record = core::ptr::addr_of_mut!((*(current_prcb as *mut _KPRCB)).ProcessorState.ContextFrame);
    let mut exception_record = core::mem::zeroed::<EXCEPTION_RECORD>();

    KdSave(0u8);

    exception_record.ExceptionAddress = KeGetContextPc(context_record) as PVOID;
    exception_record.ExceptionCode = 0x80000007u32 as i32;

    let status = KdpReportExceptionStateChange(&mut exception_record, context_record, 0u8);

    KdRestore(0u8);

    status as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn KdpTimeSlipDpcRoutine(
    _dpc: PKDPC,
    _deferred_context: PVOID,
    _system_argument1: PVOID,
    _system_argument2: PVOID,
) {
    let _dpc = opaque_ptr(_dpc);
    let _deferred_context = opaque_ptr(_deferred_context);
    let _system_argument1 = opaque_ptr(_system_argument1);
    let _system_argument2 = opaque_ptr(_system_argument2);
    let mut pending_slip = KdpTimeSlipPending;
    let mut new_slip: LONG = 0;
    loop {
        let old_slip = pending_slip;
        new_slip = if old_slip > 1 { 1 } else { 0 };

        let result = InterlockedCompareExchange(
            core::ptr::addr_of_mut!(KdpTimeSlipPending),
            new_slip,
            old_slip,
        );
        if result == old_slip {
            break;
        }
        pending_slip = result;
    }

    if new_slip != 0 {
        ExQueueWorkItem(core::ptr::addr_of_mut!(KdpTimeSlipWorkItem), _WORK_QUEUE_TYPE_DelayedWorkQueue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpTimeSlipWork(_context: PVOID) {
    let _context = opaque_ptr(_context);
    let mut old_irql = PASSIVE_LEVEL;

    ExAcquireTimeRefreshLock(0u8);
    ExUpdateSystemTimeFromCmos(0u8, 0);
    ExReleaseTimeRefreshLock();

    // [guard] KeAcquireSpinLock(core::ptr::addr_of_mut!(KdpTimeSlipEventLock), &mut old_irql);
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(core::ptr::addr_of_mut!(KdpTimeSlipEventLock)) };
    if !KdpTimeSlipEvent.is_null() {
        KeSetEvent(KdpTimeSlipEvent, 0, 0u8);
    }
    drop(spin_guard);

    let mut due_time = core::mem::zeroed::<LARGE_INTEGER>();
    due_time.QuadPart = -1800000000i64;
    KeSetTimer(
        core::ptr::addr_of_mut!(KdpTimeSlipTimer),
        due_time,
        core::ptr::addr_of_mut!(KdpTimeSlipDpc) as PKDPC,
    );
}

// kdapi.c: `KdpQueryPerformanceCounter(IN PKTRAP_FRAME TrapFrame)` — one
// pointer. Declared `*mut PVOID` here, matching KdEnterDebugger's equally
// over-deep parameter, which is how the two stayed consistent with each other
// and wrong against the three declarations of KdEnterDebugger elsewhere.
unsafe fn KdpQueryPerformanceCounter(trap_frame: PVOID) -> LARGE_INTEGER {
    let null = core::mem::zeroed::<LARGE_INTEGER>();

    if KeGetTrapFrameInterruptState(trap_frame) == 0 {
        return null;
    }

    KeQueryPerformanceCounter(core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn KdEnterDebugger(trap_frame: PVOID, exception_frame: PVOID) -> BOOLEAN {
    let trap_frame = opaque_ptr(trap_frame);
    let exception_frame = opaque_ptr(exception_frame);
    let enable;

    if !trap_frame.is_null() {
        KdTimerStop = KdpQueryPerformanceCounter(trap_frame);
        KdTimerDifference.QuadPart = KdTimerStop.QuadPart - KdTimerStart.QuadPart;
    } else {
        KdTimerStop.QuadPart = 0;
    }

    (*(KeGetCurrentPrcb() as *mut _KPRCB)).DebuggerSavedIRQL = KeGetCurrentIrql();

    enable = KeFreezeExecution(trap_frame as PKTRAP_FRAME, exception_frame as PKEXCEPTION_FRAME);

    KdpPortLocked = KeTryToAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KdpDebuggerLock) as *mut _);
    KdSave(0u8);
    KdEnteredDebugger = 1u8;

    if (KiFreezeFlag & 1) != 0 {
        // crate::kdp_dprintf!(b"FreezeLock was jammed!  Backup SpinLock was used!\n\0");
    }

    if (KiFreezeFlag & 2) != 0 {
        // crate::kdp_dprintf!(b"Some processors not frozen in debugger!\n\0");
    }

    if KdpPortLocked == 0 {
        // crate::kdp_dprintf!(b"Port lock was not acquired!\n\0");
    }

    enable
}

#[no_mangle]
pub unsafe extern "system" fn KdExitDebugger(enable: BOOLEAN) {
    KdEnteredDebugger = 0u8;
    KdRestore(0u8);
    if KdpPortLocked != 0 {
        KdpPortUnlock();
    }

    KeThawExecution(enable);

    if KdTimerStop.QuadPart == 0 {
        KdTimerStart = KdTimerStop;
    } else {
        KdTimerStart = KeQueryPerformanceCounter(core::ptr::null_mut());
    }

    // KdpTimeSlipDpc/Timer/WorkItem are only KeInitializeDpc'd on the enable_kd
    // path (kdinit.rs). The vibeos "KD forced off" config (the COM2 gate) disables
    // KD WITHOUT setting KdPitchDebugger, so KdRefreshDebuggerNotPresent (which only
    // early-outs on KdPitchDebugger) still reaches here with a zeroed, NULL-routine
    // KdpTimeSlipDpc. Queueing it makes KiRetireDpcList jump through the NULL routine
    // (C ke/dpc.c:633 / the Rust DeferredRoutine.unwrap() panic — the kmtest
    // KdSystemDebugControl quarantine). Only arm the time slip when the structures
    // exist; upstream relies on KdPitchDebugger to gate the same path.
    if KdpDebuggerStructuresInitialized != 0 {
        let time_slip = InterlockedIncrement(core::ptr::addr_of_mut!(KdpTimeSlipPending));
        if time_slip == 1 {
            InterlockedIncrement(core::ptr::addr_of_mut!(KdpTimeSlipPending));
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!(KdpTimeSlipDpc) as PKDPC,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdEnableDebuggerWithLock(need_lock: BOOLEAN) -> NTSTATUS {
    let mut old_irql = PASSIVE_LEVEL;

    if KdBlockEnable != 0 {
        return STATUS_ACCESS_DENIED;
    }

    if need_lock != 0 {
        KeRaiseIrql(DISPATCH_LEVEL, &mut old_irql);
        KdpPortLock();
    }

    if KdDisableCount == 0 {
        if need_lock != 0 {
            KdpPortUnlock();
            KeLowerIrql(old_irql);

            return STATUS_INVALID_PARAMETER;
        } else {
            KdInitSystem(0, core::ptr::null_mut());

            return STATUS_SUCCESS;
        }
    }

    KdDisableCount -= 1;
    if KdDisableCount == 0 {
        if KdPreviouslyEnabled != 0 {
            KdInitSystem(0, core::ptr::null_mut());
            KdpRestoreAllBreakpoints();
        }
    }

    if need_lock != 0 {
        KdpPortUnlock();
        KeLowerIrql(old_irql);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdDisableDebuggerWithLock(need_lock: BOOLEAN) -> NTSTATUS {
    let mut old_irql = PASSIVE_LEVEL;

    if KdBlockEnable != 0 {
        return STATUS_ACCESS_DENIED;
    }

    if need_lock != 0 {
        KeRaiseIrql(DISPATCH_LEVEL, &mut old_irql);
        KdpPortLock();
    }

    if KdDisableCount == 0 {
        if KdDebuggerEnabled == 0 && KdPitchDebugger == 0 {
            KdPreviouslyEnabled = 0u8;
        } else {
            KdPreviouslyEnabled = 1u8;
        }

        if need_lock != 0 && KdPreviouslyEnabled != 0 {
            let status = KdpAllowDisable();
            if status < 0 {
                KdpPortUnlock();
                KeLowerIrql(old_irql);
                return status;
            }
        }

        if KdDebuggerEnabled != 0 {
            KdpSuspendAllBreakPoints();
            KiDebugRoutine = Some(abi_cast!(KdpStub));

            KdDebuggerEnabled = 0u8;
            // C: SharedUserData->KdDebuggerEnabled = FALSE. Was a raw +0x2d8 poke —
            // KdDebuggerEnabled is at 0x2d4 (ketypes.h), so the old code cleared a
            // neighboring field and left the user-visible flag stale (found by
            // scan_magic_offsets audit). Use the typed field.
            (*(SharedUserData as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA)).KdDebuggerEnabled = 0;
        }
    }

    KdDisableCount += 1;

    if need_lock != 0 {
        KdpPortUnlock();
        KeLowerIrql(old_irql);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdEnableDebugger() -> NTSTATUS {
    KdEnableDebuggerWithLock(1u8)
}

#[no_mangle]
pub unsafe extern "system" fn KdDisableDebugger() -> NTSTATUS {
    KdDisableDebuggerWithLock(1u8)
}

#[no_mangle]
pub unsafe extern "system" fn KdSystemDebugControl(
    command: SYSDBG_COMMAND,
    input_buffer: PVOID,
    input_buffer_length: ULONG,
    output_buffer: PVOID,
    output_buffer_length: ULONG,
    return_length: PULONG,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let input_buffer = opaque_ptr(input_buffer);
    let output_buffer = opaque_ptr(output_buffer);
    let return_length = opaque_ptr(return_length);
    let mut status;
    let mut length = 0u32;

    match command as u32 {
        ROS_INTERNAL_COMMAND => {
            match input_buffer as ULONG_PTR {
                0x21 => PspDumpThreads(1u8),
                0x22 => PspDumpThreads(0u8),
                0x24 => MmDumpArmPfnDatabase(0u8),
                _ => {}
            }
            return STATUS_SUCCESS;
        }
        ROS_DUMP_STACK_FRAMES_COMMAND => {
            KeRosDumpStackFrames(input_buffer as PULONG_PTR, input_buffer_length);
        }
        _ => {}
    }

    match command as u32 {
        SysDbgQueryVersion => {
            if output_buffer_length != core::mem::size_of::<DBGKD_GET_VERSION64>() as u32 {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                // Build it here, then one fault-tolerant store: KdpSysGetVersion's
                // KdpMoveMemory is a raw copy, and this is the caller's buffer.
                let mut version: DBGKD_GET_VERSION64 = core::mem::zeroed();
                KdpSysGetVersion(&mut version);
                if let Err(s) = ntkrnl_sys::probe::write_back(output_buffer as PDBGKD_GET_VERSION64, version) {
                    return s;
                }
                status = STATUS_SUCCESS;
            }
        }
        SysDbgReadVirtual | SysDbgWriteVirtual => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_VIRTUAL>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_VIRTUAL) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    if command as u32 == SysDbgReadVirtual {
                        _LOCK_OPERATION_IoWriteAccess
                    } else {
                        _LOCK_OPERATION_IoReadAccess
                    },
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpCopyMemoryChunks(
                        request.Address as ULONG_PTR as ULONG64,
                        request.Buffer,
                        request.Request,
                        0,
                        if command as u32 == SysDbgReadVirtual {
                            0
                        } else {
                            MMDBG_COPY_WRITE
                        },
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgReadPhysical | SysDbgWritePhysical => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_PHYSICAL>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_PHYSICAL) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    if command as u32 == SysDbgReadVirtual {
                        _LOCK_OPERATION_IoWriteAccess
                    } else {
                        _LOCK_OPERATION_IoReadAccess
                    },
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpCopyMemoryChunks(
                        request.Address.QuadPart as ULONG64,
                        request.Buffer,
                        request.Request,
                        0,
                        MMDBG_COPY_PHYSICAL
                            | if command as u32 == SysDbgReadVirtual {
                                0
                            } else {
                                MMDBG_COPY_WRITE
                            },
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgReadControlSpace => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_CONTROL_SPACE>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_CONTROL_SPACE) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoWriteAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysReadControlSpace(
                        request.Processor,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgWriteControlSpace => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_CONTROL_SPACE>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_CONTROL_SPACE) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoReadAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysWriteControlSpace(
                        request.Processor,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgReadIoSpace => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_IO_SPACE>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_IO_SPACE) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoWriteAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysReadIoSpace(
                        request.InterfaceType as ULONG,
                        request.BusNumber,
                        request.AddressSpace,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgWriteIoSpace => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_IO_SPACE>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_IO_SPACE) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoReadAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysWriteIoSpace(
                        request.InterfaceType as ULONG,
                        request.BusNumber,
                        request.AddressSpace,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgReadMsr => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_MSR>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                // Worked on a captured copy; Data goes back with one fault-tolerant store.
                let mut request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_MSR) {
                    Ok(v) => v,
                    Err(s) => return s,
                };
                status = KdpSysReadMsr(request.Address, core::ptr::addr_of_mut!(request.Data));
                // Unconditional, as the C's in-place call leaves whatever it stored;
                // the local started as the caller's own value.
                let data = core::ptr::addr_of_mut!((*(input_buffer as PSYSDBG_MSR)).Data);
                if let Err(s) = ntkrnl_sys::probe::write_back(data, request.Data) {
                    return s;
                }
            }
        }
        SysDbgWriteMsr => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_MSR>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let mut request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_MSR) {
                    Ok(v) => v,
                    Err(s) => return s,
                };
                status = KdpSysWriteMsr(request.Address, core::ptr::addr_of_mut!(request.Data));
            }
        }
        SysDbgReadBusData => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_BUS_DATA>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_BUS_DATA) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoWriteAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysReadBusData(
                        request.BusDataType as ULONG,
                        request.BusNumber,
                        request.SlotNumber,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgWriteBusData => {
            if input_buffer_length != core::mem::size_of::<SYSDBG_BUS_DATA>() as ULONG {
                status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                let request = match ntkrnl_sys::probe::read_back(input_buffer as *const SYSDBG_BUS_DATA) {
                    Ok(v) => v,
                    Err(s) => return s, // C: the caller's __except returns the code
                };
                let mut locked_buffer = core::ptr::null_mut();
                let mut lock_variable = core::ptr::null_mut();

                status = ExLockUserBuffer(
                    request.Buffer,
                    request.Request,
                    previous_mode,
                    _LOCK_OPERATION_IoReadAccess,
                    &mut locked_buffer,
                    &mut lock_variable,
                );
                if status >= 0 {
                    status = KdpSysWriteBusData(
                        request.BusDataType as ULONG,
                        request.BusNumber,
                        request.SlotNumber,
                        request.Address,
                        locked_buffer,
                        request.Request,
                        &mut length,
                    );
                    ExUnlockUserBuffer(lock_variable);
                }
            }
        }
        SysDbgCheckLowMemory => {
            status = KdpSysCheckLowMemory(0);
        }
        _ => {
            status = STATUS_INVALID_INFO_CLASS;
        }
    }

    if !return_length.is_null() {
        return_length.write_volatile(length);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn KdChangeOption(
    option: ULONG,
    in_buffer_bytes: ULONG,
    in_buffer: PVOID,
    out_buffer_bytes: ULONG,
    out_buffer: PVOID,
    out_buffer_needed: PULONG,
) -> NTSTATUS {
    let in_buffer = opaque_ptr(in_buffer);
    let out_buffer = opaque_ptr(out_buffer);
    let out_buffer_needed = opaque_ptr(out_buffer_needed);
    if KdPitchDebugger != 0 {
        return 0xC0000354u32 as NTSTATUS;
    }

    if option != KD_OPTION_SET_BLOCK_ENABLE {
        if !out_buffer_needed.is_null() {
            out_buffer_needed.write_volatile(0);
        }
        return STATUS_INVALID_INFO_CLASS;
    }

    if in_buffer_bytes != 1 || out_buffer_bytes != 0 || !out_buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (KdBlockEnable & 0x80) != 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    // NtSystemDebugControl passes the caller's buffer straight through; the C reads
    // it inside that syscall's __try, whose handler returns the code.
    KdBlockEnable = match ntkrnl_sys::probe::read_back(in_buffer as *const u8) {
        Ok(v) => v,
        Err(s) => return s,
    };

    if !out_buffer_needed.is_null() {
        out_buffer_needed.write_volatile(0);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdPowerTransition(new_state: DEVICE_POWER_STATE) -> NTSTATUS {
    if new_state == _DEVICE_POWER_STATE_PowerDeviceD0 {
        KdD0Transition();
        STATUS_SUCCESS
    } else if new_state == _DEVICE_POWER_STATE_PowerDeviceD1 || new_state == _DEVICE_POWER_STATE_PowerDeviceD2 || new_state == _DEVICE_POWER_STATE_PowerDeviceD3 {
        KdD3Transition();
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_PARAMETER_1
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdRefreshDebuggerNotPresent() -> BOOLEAN {
    if KdPitchDebugger != 0 {
        return 1u8;
    }

    let enable = KdEnterDebugger(core::ptr::null_mut(), core::ptr::null_mut());

    crate::kdp_dprintf!(b"KDTARGET: Refreshing KD connection\n\0");

    let debugger_not_present = KdDebuggerNotPresent;

    KdExitDebugger(enable);
    debugger_not_present
}

const DPFLTR_MASK: ULONG = 0x80000000;
const MAXULONG: ULONG = 0xFFFFFFFF;
const STATUS_INVALID_PARAMETER_1: NTSTATUS = 0xC00000EF_u32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn NtQueryDebugFilterState(
    ComponentId: ULONG,
    Level: ULONG,
) -> NTSTATUS {
    let mask: PULONG;

    /* Check if the ID fits in the component table */
    if ComponentId < KdComponentTableSize {
        /* It does, so get the mask from there */
        mask = *(core::ptr::addr_of!(KdComponentTable) as *const PULONG).add(ComponentId as usize);
    } else if ComponentId == MAXULONG {
        /* Internal ID used for DbgPrint messages without ID and Level */
        mask = core::ptr::addr_of_mut!(Kd_WIN2000_Mask);
    } else {
        /* Use the default component ID */
        mask = core::ptr::addr_of_mut!(Kd_DEFAULT_Mask);
    }

    /* Convert Level to bit field if required */
    let mut level = Level;
    if level < 32 {
        level = 1 << level;
    }
    level &= !DPFLTR_MASK;

    /* Determine if this Level is filtered out */
    if (Kd_WIN2000_Mask & level) != 0 || (*mask & level) != 0 {
        /* This mask will get through to the debugger */
        1 as NTSTATUS
    } else {
        /* This mask is filtered out */
        0 as NTSTATUS
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtSetDebugFilterState(
    ComponentId: ULONG,
    Level: ULONG,
    State: BOOLEAN,
) -> NTSTATUS {
    /* Modifying debug filters requires the debug privilege */
    if SeSinglePrivilegeCheck(SeDebugPrivilege, ExGetPreviousMode()) == 0 {
        return STATUS_ACCESS_DENIED;
    }

    let mask: PULONG;

    /* Check if the ID fits in the component table */
    if ComponentId < KdComponentTableSize {
        mask = *(core::ptr::addr_of!(KdComponentTable) as *const PULONG).add(ComponentId as usize);
    } else if ComponentId == MAXULONG {
        mask = core::ptr::addr_of_mut!(Kd_WIN2000_Mask);
    } else {
        mask = core::ptr::addr_of_mut!(Kd_DEFAULT_Mask);
    }

    /* Convert Level to bit field if required */
    let mut level = Level;
    if level < 32 {
        level = 1 << level;
    }
    level &= !DPFLTR_MASK;

    /* Set or remove the Level */
    if State != 0 {
        *mask |= level;
    } else {
        *mask &= !level;
    }

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdapi.c
//  * PURPOSE:         KD64 Public Routines and Internal Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Stefan Ginsberg (stefan.ginsberg@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #ifdef KDBG
// #include <kdbg/kdb.h>
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// VOID NTAPI PspDumpThreads(BOOLEAN SystemThreads);
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KdpMoveMemory(
//     _In_ PVOID Destination,
//     _In_ PVOID Source,
//     _In_ SIZE_T Length)
// {
//     PCHAR DestinationBytes, SourceBytes;
//
//     /* Copy the buffers 1 byte at a time */
//     DestinationBytes = Destination;
//     SourceBytes = Source;
//     while (Length--) *DestinationBytes++ = *SourceBytes++;
// }
//
// VOID
// NTAPI
// KdpZeroMemory(
//     _In_ PVOID Destination,
//     _In_ SIZE_T Length)
// {
//     PCHAR DestinationBytes;
//
//     /* Zero the buffer 1 byte at a time */
//     DestinationBytes = Destination;
//     while (Length--) *DestinationBytes++ = 0;
// }
//
// NTSTATUS
// NTAPI
// KdpCopyMemoryChunks(
//     _In_ ULONG64 Address,
//     _In_ PVOID Buffer,
//     _In_ ULONG TotalSize,
//     _In_ ULONG ChunkSize,
//     _In_ ULONG Flags,
//     _Out_opt_ PULONG ActualSize)
// {
//     NTSTATUS Status;
//     ULONG RemainingLength, CopyChunk;
//
//     /* Check if we didn't get a chunk size or if it is too big */
//     if (ChunkSize == 0)
//     {
//         /* Default to 4 byte chunks */
//         ChunkSize = 4;
//     }
//     else if (ChunkSize > MMDBG_COPY_MAX_SIZE)
//     {
//         /* Normalize to maximum size */
//         ChunkSize = MMDBG_COPY_MAX_SIZE;
//     }
//
//     /* Copy the whole range in aligned chunks */
//     RemainingLength = TotalSize;
//     CopyChunk = 1;
//     while (RemainingLength > 0)
//     {
//         /*
//          * Determine the best chunk size for this round.
//          * The ideal size is aligned, isn't larger than the
//          * the remaining length and respects the chunk limit.
//          */
//         while (((CopyChunk * 2) <= RemainingLength) &&
//                (CopyChunk < ChunkSize) &&
//                ((Address & ((CopyChunk * 2) - 1)) == 0))
//         {
//             /* Increase it */
//             CopyChunk *= 2;
//         }
//
//         /*
//          * The chunk size can be larger than the remaining size if this
//          * isn't the first round, so check if we need to shrink it back.
//          */
//         while (CopyChunk > RemainingLength)
//         {
//             /* Shrink it */
//             CopyChunk /= 2;
//         }
//
//         /* Do the copy */
//         Status = MmDbgCopyMemory(Address, Buffer, CopyChunk, Flags);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Copy failed, break out */
//             break;
//         }
//
//         /* Update pointers and length for the next run */
//         Address = Address + CopyChunk;
//         Buffer = (PVOID)((ULONG_PTR)Buffer + CopyChunk);
//         RemainingLength = RemainingLength - CopyChunk;
//     }
//
//     /* We may have modified executable code, flush the instruction cache */
//     KeSweepICache((PVOID)(ULONG_PTR)Address, TotalSize);
//
//     /*
//      * Return the size we managed to copy and return
//      * success if we could copy the whole range.
//      */
//     if (ActualSize) *ActualSize = TotalSize - RemainingLength;
//     return RemainingLength == 0 ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
// }
//
// VOID
// NTAPI
// KdpQueryMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                IN PCONTEXT Context)
// {
//     PDBGKD_QUERY_MEMORY Memory = &State->u.QueryMemory;
//     STRING Header;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Validate the address space */
//     if (Memory->AddressSpace == DBGKD_QUERY_MEMORY_VIRTUAL)
//     {
//         /* Check if this is process memory */
//         if ((PVOID)(ULONG_PTR)Memory->Address < MmHighestUserAddress)
//         {
//             /* It is */
//             Memory->AddressSpace = DBGKD_QUERY_MEMORY_PROCESS;
//         }
//         else
//         {
//             /* Check if it's session space */
//             if (MmIsSessionAddress((PVOID)(ULONG_PTR)Memory->Address))
//             {
//                 /* It is */
//                 Memory->AddressSpace = DBGKD_QUERY_MEMORY_SESSION;
//             }
//             else
//             {
//                 /* Not session space but some other kernel memory */
//                 Memory->AddressSpace = DBGKD_QUERY_MEMORY_KERNEL;
//             }
//         }
//
//         /* Set flags */
//         Memory->Flags = DBGKD_QUERY_MEMORY_READ |
//                         DBGKD_QUERY_MEMORY_WRITE |
//                         DBGKD_QUERY_MEMORY_EXECUTE;
//     }
//     else
//     {
//         /* Invalid */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* Return structure */
//     State->ReturnStatus = Status;
//     Memory->Reserved = 0;
//
//     /* Build header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpSearchMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                 IN PSTRING Data,
//                 IN PCONTEXT Context)
// {
//     //PDBGKD_SEARCH_MEMORY SearchMemory = &State->u.SearchMemory;
//     STRING Header;
//
//     /* TODO */
//     KdpDprintf("Memory Search support is unimplemented!\n");
//
//     /* Send a failure packet */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpFillMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//               IN PSTRING Data,
//               IN PCONTEXT Context)
// {
//     //PDBGKD_FILL_MEMORY FillMemory = &State->u.FillMemory;
//     STRING Header;
//
//     /* TODO */
//     KdpDprintf("Memory Fill support is unimplemented!\n");
//
//     /* Send a failure packet */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteBreakpoint(IN PDBGKD_MANIPULATE_STATE64 State,
//                    IN PSTRING Data,
//                    IN PCONTEXT Context)
// {
//     PDBGKD_WRITE_BREAKPOINT64 Breakpoint = &State->u.WriteBreakPoint;
//     STRING Header;
//
//     /* Build header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Create the breakpoint */
//     Breakpoint->BreakPointHandle =
//         KdpAddBreakpoint((PVOID)(ULONG_PTR)Breakpoint->BreakPointAddress);
//     if (!Breakpoint->BreakPointHandle)
//     {
//         /* We failed */
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//     else
//     {
//         /* Success! */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpRestoreBreakpoint(IN PDBGKD_MANIPULATE_STATE64 State,
//                      IN PSTRING Data,
//                      IN PCONTEXT Context)
// {
//     PDBGKD_RESTORE_BREAKPOINT RestoreBp = &State->u.RestoreBreakPoint;
//     STRING Header;
//
//     /* Fill out the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Get the version block */
//     if (KdpDeleteBreakpoint(RestoreBp->BreakPointHandle))
//     {
//         /* We're all good */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* We failed */
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// NTSTATUS
// NTAPI
// KdpWriteBreakPointEx(IN PDBGKD_MANIPULATE_STATE64 State,
//                      IN PSTRING Data,
//                      IN PCONTEXT Context)
// {
//     //PDBGKD_BREAKPOINTEX = &State->u.BreakPointEx;
//     STRING Header;
//
//     /* TODO */
//     KdpDprintf("Extended Breakpoint Write support is unimplemented!\n");
//
//     /* Send a failure packet */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
//     return STATUS_UNSUCCESSFUL;
// }
//
// VOID
// NTAPI
// KdpRestoreBreakPointEx(IN PDBGKD_MANIPULATE_STATE64 State,
//                        IN PSTRING Data,
//                        IN PCONTEXT Context)
// {
//     //PDBGKD_BREAKPOINTEX = &State->u.BreakPointEx;
//     STRING Header;
//
//     /* TODO */
//     KdpDprintf("Extended Breakpoint Restore support is unimplemented!\n");
//
//     /* Send a failure packet */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteCustomBreakpoint(IN PDBGKD_MANIPULATE_STATE64 State,
//                          IN PSTRING Data,
//                          IN PCONTEXT Context)
// {
//     //PDBGKD_WRITE_CUSTOM_BREAKPOINT = &State->u.WriteCustomBreakpoint;
//     STRING Header;
//
//     /* Not supported */
//     KdpDprintf("Custom Breakpoint Write is unimplemented\n");
//
//     /* Send a failure packet */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// DumpTraceData(IN PSTRING TraceData)
// {
//     /* Update the buffer */
//     TraceDataBuffer[0] = TraceDataBufferPosition;
//
//     /* Setup the trace data */
//     TraceData->Length = (USHORT)(TraceDataBufferPosition * sizeof(ULONG));
//     TraceData->Buffer = (PCHAR)TraceDataBuffer;
//
//     /* Reset the buffer location */
//     TraceDataBufferPosition = 1;
// }
//
// VOID
// NTAPI
// KdpSetCommonState(IN ULONG NewState,
//                   IN PCONTEXT Context,
//                   IN PDBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange)
// {
//     ULONG InstructionCount;
//     BOOLEAN HadBreakpoints;
//
//     /* Setup common stuff available for all CPU architectures */
//     WaitStateChange->NewState = NewState;
//     WaitStateChange->ProcessorLevel = KeProcessorLevel;
//     WaitStateChange->Processor = (USHORT)KeGetCurrentPrcb()->Number;
//     WaitStateChange->NumberProcessors = (ULONG)KeNumberProcessors;
//     WaitStateChange->Thread = (ULONG64)(LONG_PTR)KeGetCurrentThread();
//     WaitStateChange->ProgramCounter = (ULONG64)(LONG_PTR)KeGetContextPc(Context);
//
//     /* Zero out the entire Control Report */
//     KdpZeroMemory(&WaitStateChange->AnyControlReport,
//                   sizeof(DBGKD_ANY_CONTROL_REPORT));
//
//     /* Now copy the instruction stream and set the count */
//     KdpCopyMemoryChunks((ULONG_PTR)WaitStateChange->ProgramCounter,
//                         &WaitStateChange->ControlReport.InstructionStream[0],
//                         DBGKD_MAXSTREAM,
//                         0,
//                         MMDBG_COPY_UNSAFE,
//                         &InstructionCount);
//     WaitStateChange->ControlReport.InstructionCount = (USHORT)InstructionCount;
//
//     /* Clear all the breakpoints in this region */
//     HadBreakpoints =
//         KdpDeleteBreakpointRange((PVOID)(ULONG_PTR)WaitStateChange->ProgramCounter,
//                                  (PVOID)((ULONG_PTR)WaitStateChange->ProgramCounter +
//                                          WaitStateChange->ControlReport.InstructionCount - 1));
//     if (HadBreakpoints)
//     {
//         /* Copy the instruction stream again, this time without breakpoints */
//         KdpCopyMemoryChunks((ULONG_PTR)WaitStateChange->ProgramCounter,
//                             &WaitStateChange->ControlReport.InstructionStream[0],
//                             InstructionCount,
//                             0,
//                             MMDBG_COPY_UNSAFE,
//                             NULL);
//     }
// }
//
// VOID
// NTAPI
// KdpSysGetVersion(
//     _Out_ PDBGKD_GET_VERSION64 Version)
// {
//     /* Copy the version block */
//     KdpMoveMemory(Version,
//                   &KdVersionBlock,
//                   sizeof(DBGKD_GET_VERSION64));
// }
//
// VOID
// NTAPI
// KdpGetVersion(IN PDBGKD_MANIPULATE_STATE64 State)
// {
//     STRING Header;
//
//     /* Fill out the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Get the version block */
//     KdpSysGetVersion(&State->u.GetVersion64);
//
//     /* Fill out the state */
//     State->ApiNumber = DbgKdGetVersionApi;
//     State->ReturnStatus = STATUS_SUCCESS;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpReadVirtualMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                      IN PSTRING Data,
//                      IN PCONTEXT Context)
// {
//     PDBGKD_READ_MEMORY64 ReadMemory = &State->u.ReadMemory;
//     STRING Header;
//     ULONG Length = ReadMemory->TransferCount;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Validate length */
//     if (Length > (PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64)))
//     {
//         /* Overflow, set it to maximum possible */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64);
//     }
//
//     /* Do the read */
//     State->ReturnStatus = KdpCopyMemoryChunks(ReadMemory->TargetBaseAddress,
//                                               Data->Buffer,
//                                               Length,
//                                               0,
//                                               MMDBG_COPY_UNSAFE,
//                                               &Length);
//
//     /* Return the actual length read */
//     ReadMemory->ActualBytesRead = Length;
//     Data->Length = (USHORT)Length;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteVirtualMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                       IN PSTRING Data,
//                       IN PCONTEXT Context)
// {
//     PDBGKD_WRITE_MEMORY64 WriteMemory = &State->u.WriteMemory;
//     STRING Header;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Do the write */
//     State->ReturnStatus = KdpCopyMemoryChunks(WriteMemory->TargetBaseAddress,
//                                               Data->Buffer,
//                                               Data->Length,
//                                               0,
//                                               MMDBG_COPY_UNSAFE |
//                                               MMDBG_COPY_WRITE,
//                                               &WriteMemory->ActualBytesWritten);
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpReadPhysicalMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                       IN PSTRING Data,
//                       IN PCONTEXT Context)
// {
//     PDBGKD_READ_MEMORY64 ReadMemory = &State->u.ReadMemory;
//     STRING Header;
//     ULONG Length = ReadMemory->TransferCount;
//     ULONG Flags, CacheFlags;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Validate length */
//     if (Length > (PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64)))
//     {
//         /* Overflow, set it to maximum possible */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64);
//     }
//
//     /* Start with the default flags */
//     Flags = MMDBG_COPY_UNSAFE | MMDBG_COPY_PHYSICAL;
//
//     /* Get the caching flags and check if a type is specified */
//     CacheFlags = ReadMemory->ActualBytesRead;
//     if (CacheFlags == DBGKD_CACHING_CACHED)
//     {
//         /* Cached */
//         Flags |= MMDBG_COPY_CACHED;
//     }
//     else if (CacheFlags == DBGKD_CACHING_UNCACHED)
//     {
//         /* Uncached */
//         Flags |= MMDBG_COPY_UNCACHED;
//     }
//     else if (CacheFlags == DBGKD_CACHING_WRITE_COMBINED)
//     {
//         /* Write Combined */
//         Flags |= MMDBG_COPY_WRITE_COMBINED;
//     }
//
//     /* Do the read */
//     State->ReturnStatus = KdpCopyMemoryChunks(ReadMemory->TargetBaseAddress,
//                                               Data->Buffer,
//                                               Length,
//                                               0,
//                                               Flags,
//                                               &Length);
//
//     /* Return the actual length read */
//     ReadMemory->ActualBytesRead = Length;
//     Data->Length = (USHORT)Length;
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWritePhysicalMemory(IN PDBGKD_MANIPULATE_STATE64 State,
//                        IN PSTRING Data,
//                        IN PCONTEXT Context)
// {
//     PDBGKD_WRITE_MEMORY64 WriteMemory = &State->u.WriteMemory;
//     STRING Header;
//     ULONG Flags, CacheFlags;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Start with the default flags */
//     Flags = MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE | MMDBG_COPY_PHYSICAL;
//
//     /* Get the caching flags and check if a type is specified */
//     CacheFlags = WriteMemory->ActualBytesWritten;
//     if (CacheFlags == DBGKD_CACHING_CACHED)
//     {
//         /* Cached */
//         Flags |= MMDBG_COPY_CACHED;
//     }
//     else if (CacheFlags == DBGKD_CACHING_UNCACHED)
//     {
//         /* Uncached */
//         Flags |= MMDBG_COPY_UNCACHED;
//     }
//     else if (CacheFlags == DBGKD_CACHING_WRITE_COMBINED)
//     {
//         /* Write Combined */
//         Flags |= MMDBG_COPY_WRITE_COMBINED;
//     }
//
//     /* Do the write */
//     State->ReturnStatus = KdpCopyMemoryChunks(WriteMemory->TargetBaseAddress,
//                                               Data->Buffer,
//                                               Data->Length,
//                                               0,
//                                               Flags,
//                                               &WriteMemory->ActualBytesWritten);
//
//     /* Send the packet */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpReadControlSpace(IN PDBGKD_MANIPULATE_STATE64 State,
//                     IN PSTRING Data,
//                     IN PCONTEXT Context)
// {
//     PDBGKD_READ_MEMORY64 ReadMemory = &State->u.ReadMemory;
//     STRING Header;
//     ULONG Length;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Check the length requested */
//     Length = ReadMemory->TransferCount;
//     if (Length > (PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64)))
//     {
//         /* Use maximum allowed */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64);
//     }
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysReadControlSpace(State->Processor,
//                                                  ReadMemory->TargetBaseAddress,
//                                                  Data->Buffer,
//                                                  Length,
//                                                  &Length);
//
//     /* Return the actual length read */
//     ReadMemory->ActualBytesRead = Length;
//     Data->Length = (USHORT)Length;
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteControlSpace(IN PDBGKD_MANIPULATE_STATE64 State,
//                      IN PSTRING Data,
//                      IN PCONTEXT Context)
// {
//     PDBGKD_WRITE_MEMORY64 WriteMemory = &State->u.WriteMemory;
//     STRING Header;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysWriteControlSpace(State->Processor,
//                                                   WriteMemory->TargetBaseAddress,
//                                                   Data->Buffer,
//                                                   Data->Length,
//                                                   &WriteMemory->ActualBytesWritten);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpGetContext(IN PDBGKD_MANIPULATE_STATE64 State,
//               IN PSTRING Data,
//               IN PCONTEXT Context)
// {
//     STRING Header;
//     PCONTEXT TargetContext;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Make sure that this is a valid request */
//     if (State->Processor < KeNumberProcessors)
//     {
//         /* Check if the request is for this CPU */
//         if (State->Processor == KeGetCurrentPrcb()->Number)
//         {
//             /* We're just copying our own context */
//             TargetContext = Context;
//         }
//         else
//         {
//             /* Get the context from the PRCB array */
//             TargetContext = &KiProcessorBlock[State->Processor]->
//                             ProcessorState.ContextFrame;
//         }
//
//         /* Copy it over to the debugger */
//         KdpMoveMemory(Data->Buffer,
//                       TargetContext,
//                       sizeof(CONTEXT));
//         Data->Length = sizeof(CONTEXT);
//
//         /* Let the debugger set the context now */
//         KdpContextSent = TRUE;
//
//         /* Finish up */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid request */
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpSetContext(IN PDBGKD_MANIPULATE_STATE64 State,
//               IN PSTRING Data,
//               IN PCONTEXT Context)
// {
//     STRING Header;
//     PCONTEXT TargetContext;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == sizeof(CONTEXT));
//
//     /* Make sure that this is a valid request */
//     if ((State->Processor < KeNumberProcessors) &&
//         (KdpContextSent))
//     {
//         /* Check if the request is for this CPU */
//         if (State->Processor == KeGetCurrentPrcb()->Number)
//         {
//             /* We're just copying our own context */
//             TargetContext = Context;
//         }
//         else
//         {
//             /* Get the context from the PRCB array */
//             TargetContext = &KiProcessorBlock[State->Processor]->
//                             ProcessorState.ContextFrame;
//         }
//
//         /* Copy the new context to it */
//         KdpMoveMemory(TargetContext,
//                       Data->Buffer,
//                       sizeof(CONTEXT));
//
//         /* Finish up */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid request */
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpGetContextEx(IN PDBGKD_MANIPULATE_STATE64 State,
//                 IN PSTRING Data,
//                 IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_CONTEXT_EX ContextEx;
//     PCONTEXT TargetContext;
//     ASSERT(Data->Length == 0);
//
//     /* Get our struct */
//     ContextEx = &State->u.ContextEx;
//
//     /* Set up the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Make sure that this is a valid request */
//     if ((State->Processor < KeNumberProcessors) &&
//         (ContextEx->Offset + ContextEx->ByteCount) <= sizeof(CONTEXT))
//     {
//         /* Check if the request is for this CPU */
//         if (State->Processor == KeGetCurrentPrcb()->Number)
//         {
//             /* We're just copying our own context */
//             TargetContext = Context;
//         }
//         else
//         {
//             /* Get the context from the PRCB array */
//             TargetContext = &KiProcessorBlock[State->Processor]->
//                             ProcessorState.ContextFrame;
//         }
//
//         /* Copy what is requested */
//         KdpMoveMemory(Data->Buffer,
//                       (PVOID)((ULONG_PTR)TargetContext + ContextEx->Offset),
//                       ContextEx->ByteCount);
//
//         /* KD copies all */
//         Data->Length = ContextEx->BytesCopied = ContextEx->ByteCount;
//
//         /* Let the debugger set the context now */
//         KdpContextSent = TRUE;
//
//         /* Finish up */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid request */
//         ContextEx->BytesCopied = 0;
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpSetContextEx(IN PDBGKD_MANIPULATE_STATE64 State,
//                 IN PSTRING Data,
//                 IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_CONTEXT_EX ContextEx;
//     PCONTEXT TargetContext;
//
//     /* Get our struct */
//     ContextEx = &State->u.ContextEx;
//     ASSERT(Data->Length == ContextEx->ByteCount);
//
//     /* Set up the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Make sure that this is a valid request */
//     if ((State->Processor < KeNumberProcessors) &&
//         ((ContextEx->Offset + ContextEx->ByteCount) <= sizeof(CONTEXT)) &&
//         (KdpContextSent))
//     {
//         /* Check if the request is for this CPU */
//         if (State->Processor == KeGetCurrentPrcb()->Number)
//         {
//             /* We're just copying our own context */
//             TargetContext = Context;
//         }
//         else
//         {
//             /* Get the context from the PRCB array */
//             TargetContext = &KiProcessorBlock[State->Processor]->
//                             ProcessorState.ContextFrame;
//         }
//
//         /* Copy what is requested */
//         KdpMoveMemory((PVOID)((ULONG_PTR)TargetContext + ContextEx->Offset),
//                       Data->Buffer,
//                       ContextEx->ByteCount);
//
//         /* KD copies all */
//         ContextEx->BytesCopied = ContextEx->ByteCount;
//
//         /* Finish up */
//         State->ReturnStatus = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid request */
//         ContextEx->BytesCopied = 0;
//         State->ReturnStatus = STATUS_UNSUCCESSFUL;
//     }
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpCauseBugCheck(IN PDBGKD_MANIPULATE_STATE64 State)
// {
//     /* Crash with the special code */
//     KeBugCheck(MANUALLY_INITIATED_CRASH);
// }
//
// VOID
// NTAPI
// KdpReadMachineSpecificRegister(IN PDBGKD_MANIPULATE_STATE64 State,
//                                IN PSTRING Data,
//                                IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_MSR ReadMsr = &State->u.ReadWriteMsr;
//     ULARGE_INTEGER MsrValue;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysReadMsr(ReadMsr->Msr, &MsrValue.QuadPart);
//
//     /* Return the data */
//     ReadMsr->DataValueLow = MsrValue.LowPart;
//     ReadMsr->DataValueHigh = MsrValue.HighPart;
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteMachineSpecificRegister(IN PDBGKD_MANIPULATE_STATE64 State,
//                                 IN PSTRING Data,
//                                 IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_MSR WriteMsr = &State->u.ReadWriteMsr;
//     ULARGE_INTEGER MsrValue;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Call the internal routine */
//     MsrValue.LowPart = WriteMsr->DataValueLow;
//     MsrValue.HighPart = WriteMsr->DataValueHigh;
//     State->ReturnStatus = KdpSysWriteMsr(WriteMsr->Msr, &MsrValue.QuadPart);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpGetBusData(IN PDBGKD_MANIPULATE_STATE64 State,
//               IN PSTRING Data,
//               IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_GET_SET_BUS_DATA GetBusData = &State->u.GetSetBusData;
//     ULONG Length;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Check the length requested */
//     Length = GetBusData->Length;
//     if (Length > (PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64)))
//     {
//         /* Use maximum allowed */
//         Length = PACKET_MAX_SIZE - sizeof(DBGKD_MANIPULATE_STATE64);
//     }
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysReadBusData(GetBusData->BusDataType,
//                                             GetBusData->BusNumber,
//                                             GetBusData->SlotNumber,
//                                             GetBusData->Offset,
//                                             Data->Buffer,
//                                             Length,
//                                             &Length);
//
//     /* Return the actual length read */
//     GetBusData->Length = Length;
//     Data->Length = (USHORT)Length;
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  Data,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpSetBusData(IN PDBGKD_MANIPULATE_STATE64 State,
//               IN PSTRING Data,
//               IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_GET_SET_BUS_DATA SetBusData = &State->u.GetSetBusData;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysWriteBusData(SetBusData->BusDataType,
//                                              SetBusData->BusNumber,
//                                              SetBusData->SlotNumber,
//                                              SetBusData->Offset,
//                                              Data->Buffer,
//                                              SetBusData->Length,
//                                              &SetBusData->Length);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpReadIoSpace(IN PDBGKD_MANIPULATE_STATE64 State,
//                IN PSTRING Data,
//                IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_IO64 ReadIo = &State->u.ReadWriteIo;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /*
//      * Clear the value so 1 or 2 byte reads
//      * don't leave the higher bits unmodified
//      */
//     ReadIo->DataValue = 0;
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysReadIoSpace(Isa,
//                                             0,
//                                             1,
//                                             ReadIo->IoAddress,
//                                             &ReadIo->DataValue,
//                                             ReadIo->DataSize,
//                                             &ReadIo->DataSize);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteIoSpace(IN PDBGKD_MANIPULATE_STATE64 State,
//                 IN PSTRING Data,
//                 IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_IO64 WriteIo = &State->u.ReadWriteIo;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysWriteIoSpace(Isa,
//                                              0,
//                                              1,
//                                              WriteIo->IoAddress,
//                                              &WriteIo->DataValue,
//                                              WriteIo->DataSize,
//                                              &WriteIo->DataSize);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpReadIoSpaceExtended(IN PDBGKD_MANIPULATE_STATE64 State,
//                        IN PSTRING Data,
//                        IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_IO_EXTENDED64 ReadIoExtended = &State->u.
//                                                       ReadWriteIoExtended;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /*
//      * Clear the value so 1 or 2 byte reads
//      * don't leave the higher bits unmodified
//      */
//     ReadIoExtended->DataValue = 0;
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysReadIoSpace(ReadIoExtended->InterfaceType,
//                                             ReadIoExtended->BusNumber,
//                                             ReadIoExtended->AddressSpace,
//                                             ReadIoExtended->IoAddress,
//                                             &ReadIoExtended->DataValue,
//                                             ReadIoExtended->DataSize,
//                                             &ReadIoExtended->DataSize);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpWriteIoSpaceExtended(IN PDBGKD_MANIPULATE_STATE64 State,
//                         IN PSTRING Data,
//                         IN PCONTEXT Context)
// {
//     STRING Header;
//     PDBGKD_READ_WRITE_IO_EXTENDED64 WriteIoExtended = &State->u.
//                                                        ReadWriteIoExtended;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//     ASSERT(Data->Length == 0);
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysWriteIoSpace(WriteIoExtended->InterfaceType,
//                                              WriteIoExtended->BusNumber,
//                                              WriteIoExtended->AddressSpace,
//                                              WriteIoExtended->IoAddress,
//                                              &WriteIoExtended->DataValue,
//                                              WriteIoExtended->DataSize,
//                                              &WriteIoExtended->DataSize);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpCheckLowMemory(IN PDBGKD_MANIPULATE_STATE64 State)
// {
//     STRING Header;
//
//     /* Setup the header */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Call the internal routine */
//     State->ReturnStatus = KdpSysCheckLowMemory(MMDBG_COPY_UNSAFE);
//
//     /* Send the reply */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// VOID
// NTAPI
// KdpNotSupported(IN PDBGKD_MANIPULATE_STATE64 State)
// {
//     STRING Header;
//
//     /* Set failure */
//     State->ReturnStatus = STATUS_UNSUCCESSFUL;
//
//     /* Setup the packet */
//     Header.Length = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)State;
//
//     /* Send it */
//     KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                  &Header,
//                  NULL,
//                  &KdpContext);
// }
//
// static
// KCONTINUE_STATUS
// KdpSwitchProcessor(
//     _In_ USHORT ProcessorIndex)
// {
//     /* Make sure that the processor index is valid */
//     if (ProcessorIndex >= KeNumberProcessors)
//     {
//         KdpDprintf("%u is not a valid processor number\n", ProcessorIndex);
//         return ContinueProcessorReselected;
//     }
//
//     /* If the new processor is the current one, there is nothing to do */
//     if (ProcessorIndex == KeGetCurrentProcessorNumber())
//     {
//         return ContinueProcessorReselected;
//     }
//
//     /* Call the architecture specific Ke routine */
//     return KxSwitchKdProcessor(ProcessorIndex);
// }
//
// KCONTINUE_STATUS
// NTAPI
// KdpSendWaitContinue(IN ULONG PacketType,
//                     IN PSTRING SendHeader,
//                     IN PSTRING SendData OPTIONAL,
//                     IN OUT PCONTEXT Context)
// {
//     STRING Data, Header;
//     DBGKD_MANIPULATE_STATE64 ManipulateState;
//     ULONG Length;
//     KDSTATUS RecvCode;
//
//     /* Setup the Manipulate State structure */
//     Header.MaximumLength = sizeof(DBGKD_MANIPULATE_STATE64);
//     Header.Buffer = (PCHAR)&ManipulateState;
//     Data.MaximumLength = sizeof(KdpMessageBuffer);
//     Data.Buffer = KdpMessageBuffer;
//
//     /*
//      * Reset the context state to ensure the debugger has received
//      * the current context before it sets it.
//      */
//     KdpContextSent = FALSE;
//
// SendPacket:
//     /* Send the Packet */
//     KdSendPacket(PacketType, SendHeader, SendData, &KdpContext);
//
//     /* If the debugger isn't present anymore, just return success */
//     if (KdDebuggerNotPresent) return ContinueSuccess;
//
//     /* Main processing Loop */
//     for (;;)
//     {
//         /* Receive Loop */
//         do
//         {
//             /* Wait to get a reply to our packet */
//             RecvCode = KdReceivePacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                                        &Header,
//                                        &Data,
//                                        &Length,
//                                        &KdpContext);
//
//             /* If we got a resend request, do it */
//             if (RecvCode == KdPacketNeedsResend) goto SendPacket;
//         } while (RecvCode == KdPacketTimedOut);
//
//         /* Now check what API we got */
//         switch (ManipulateState.ApiNumber)
//         {
//             case DbgKdReadVirtualMemoryApi:
//
//                 /* Read virtual memory */
//                 KdpReadVirtualMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteVirtualMemoryApi:
//
//                 /* Write virtual memory */
//                 KdpWriteVirtualMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdGetContextApi:
//
//                 /* Get the current context */
//                 KdpGetContext(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdSetContextApi:
//
//                 /* Set a new context */
//                 KdpSetContext(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteBreakPointApi:
//
//                 /* Write the breakpoint */
//                 KdpWriteBreakpoint(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdRestoreBreakPointApi:
//
//                 /* Restore the breakpoint */
//                 KdpRestoreBreakpoint(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdContinueApi:
//
//                 /* Simply continue */
//                 return NT_SUCCESS(ManipulateState.u.Continue.ContinueStatus);
//
//             case DbgKdReadControlSpaceApi:
//
//                 /* Read control space */
//                 KdpReadControlSpace(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteControlSpaceApi:
//
//                 /* Write control space */
//                 KdpWriteControlSpace(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdReadIoSpaceApi:
//
//                 /* Read I/O Space */
//                 KdpReadIoSpace(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteIoSpaceApi:
//
//                 /* Write I/O Space */
//                 KdpWriteIoSpace(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdRebootApi:
//
//                 /* Reboot the system */
//                 HalReturnToFirmware(_FIRMWARE_REENTRY_HalRebootRoutine);
//                 break;
//
//             case DbgKdContinueApi2:
//
//                 /* Check if caller reports success */
//                 if (NT_SUCCESS(ManipulateState.u.Continue2.ContinueStatus))
//                 {
//                     /* Update the state */
//                     KdpGetStateChange(&ManipulateState, Context);
//                     return ContinueSuccess;
//                 }
//                 else
//                 {
//                     /* Return an error */
//                     return ContinueError;
//                 }
//
//             case DbgKdReadPhysicalMemoryApi:
//
//                 /* Read  physical memory */
//                 KdpReadPhysicalMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWritePhysicalMemoryApi:
//
//                 /* Write  physical memory */
//                 KdpWritePhysicalMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdQuerySpecialCallsApi:
//             case DbgKdSetSpecialCallApi:
//             case DbgKdClearSpecialCallsApi:
//
//                 /* TODO */
//                 KdpDprintf("Special Call support is unimplemented!\n");
//                 KdpNotSupported(&ManipulateState);
//                 break;
//
//             case DbgKdSetInternalBreakPointApi:
//             case DbgKdGetInternalBreakPointApi:
//
//                 /* TODO */
//                 KdpDprintf("Internal Breakpoint support is unimplemented!\n");
//                 KdpNotSupported(&ManipulateState);
//                 break;
//
//             case DbgKdReadIoSpaceExtendedApi:
//
//                 /* Read I/O Space */
//                 KdpReadIoSpaceExtended(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteIoSpaceExtendedApi:
//
//                 /* Write I/O Space */
//                 KdpWriteIoSpaceExtended(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdGetVersionApi:
//
//                 /* Get version data */
//                 KdpGetVersion(&ManipulateState);
//                 break;
//
//             case DbgKdWriteBreakPointExApi:
//
//                 /* Write the breakpoint and check if it failed */
//                 if (!NT_SUCCESS(KdpWriteBreakPointEx(&ManipulateState,
//                                                      &Data,
//                                                      Context)))
//                 {
//                     /* Return an error */
//                     return ContinueError;
//                 }
//                 break;
//
//             case DbgKdRestoreBreakPointExApi:
//
//                 /* Restore the breakpoint */
//                 KdpRestoreBreakPointEx(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdCauseBugCheckApi:
//
//                 /* Crash the system */
//                 KdpCauseBugCheck(&ManipulateState);
//                 break;
//
//             case DbgKdSwitchProcessor:
//
//                 /* Switch the processor and return */
//                 return KdpSwitchProcessor(ManipulateState.Processor);
//
//             case DbgKdPageInApi:
//
//                 /* This API, introduced in NT4, has been obsoleted in NT5. It is
//                  * replaced by ExpDebuggerPageIn support in ExpDebuggerWorker(). */
//                 KdpDprintf("DbgKdPageInApi is obsolete!\n");
//                 KdpNotSupported(&ManipulateState);
//                 break;
//
//             case DbgKdReadMachineSpecificRegister:
//
//                 /* Read from the specified MSR */
//                 KdpReadMachineSpecificRegister(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdWriteMachineSpecificRegister:
//
//                 /* Write to the specified MSR */
//                 KdpWriteMachineSpecificRegister(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdSearchMemoryApi:
//
//                 /* Search memory */
//                 KdpSearchMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdGetBusDataApi:
//
//                 /* Read from the bus */
//                 KdpGetBusData(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdSetBusDataApi:
//
//                 /* Write to the bus */
//                 KdpSetBusData(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdCheckLowMemoryApi:
//
//                 /* Check for memory corruption in the lower 4 GB */
//                 KdpCheckLowMemory(&ManipulateState);
//                 break;
//
//             case DbgKdClearAllInternalBreakpointsApi:
//
//                 /* Just clear the counter */
//                 KdpNumInternalBreakpoints = 0;
//                 break;
//
//             case DbgKdFillMemoryApi:
//
//                 /* Fill memory */
//                 KdpFillMemory(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdQueryMemoryApi:
//
//                 /* Query memory */
//                 KdpQueryMemory(&ManipulateState, Context);
//                 break;
//
//             case DbgKdSwitchPartition:
//
//                 /* TODO */
//                 KdpDprintf("Partition Switch support is unimplemented!\n");
//                 KdpNotSupported(&ManipulateState);
//                 break;
//
//             case DbgKdWriteCustomBreakpointApi:
//
//                 /* Write the customized breakpoint */
//                 KdpWriteCustomBreakpoint(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdGetContextExApi:
//
//                 /* Extended Context Get */
//                 KdpGetContextEx(&ManipulateState, &Data, Context);
//                 break;
//
//             case DbgKdSetContextExApi:
//
//                 /* Extended Context Set */
//                 KdpSetContextEx(&ManipulateState, &Data, Context);
//                 break;
//
//             /* Unsupported Messages */
//             default:
//
//                 /* Send warning */
//                 KdpDprintf("Received Unrecognized API 0x%lx\n", ManipulateState.ApiNumber);
//
//                 /* Setup an empty message, with failure */
//                 Data.Length = 0;
//                 ManipulateState.ReturnStatus = STATUS_UNSUCCESSFUL;
//
//                 /* Send it */
//                 KdSendPacket(PACKET_TYPE_KD_STATE_MANIPULATE,
//                              &Header,
//                              &Data,
//                              &KdpContext);
//                 break;
//         }
//     }
// }
//
// VOID
// NTAPI
// KdpReportLoadSymbolsStateChange(IN PSTRING PathName,
//                                 IN PKD_SYMBOLS_INFO SymbolInfo,
//                                 IN BOOLEAN Unload,
//                                 IN OUT PCONTEXT Context)
// {
//     PSTRING ExtraData;
//     STRING Data, Header;
//     DBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange;
//     ULONG PathNameLength;
//     KCONTINUE_STATUS Status;
//
//     /* Start wait loop */
//     do
//     {
//         /* Build the architecture common parts of the message */
//         KdpSetCommonState(DbgKdLoadSymbolsStateChange,
//                           Context,
//                           &WaitStateChange);
//
//         /* Now finish creating the structure */
//         KdpSetContextState(&WaitStateChange, Context);
//
//         /* Fill out load data */
//         WaitStateChange.u.LoadSymbols.UnloadSymbols = Unload;
//         WaitStateChange.u.LoadSymbols.BaseOfDll = (ULONG64)(LONG_PTR)SymbolInfo->BaseOfDll;
//         WaitStateChange.u.LoadSymbols.ProcessId = SymbolInfo->ProcessId;
//         WaitStateChange.u.LoadSymbols.CheckSum = SymbolInfo->CheckSum;
//         WaitStateChange.u.LoadSymbols.SizeOfImage = SymbolInfo->SizeOfImage;
//
//         /* Check if we have a path name */
//         if (PathName)
//         {
//             /* Copy it to the path buffer */
//             KdpCopyMemoryChunks((ULONG_PTR)PathName->Buffer,
//                                 KdpPathBuffer,
//                                 PathName->Length,
//                                 0,
//                                 MMDBG_COPY_UNSAFE,
//                                 &PathNameLength);
//
//             /* Null terminate */
//             KdpPathBuffer[PathNameLength++] = ANSI_NULL;
//
//             /* Set the path length */
//             WaitStateChange.u.LoadSymbols.PathNameLength = PathNameLength;
//
//             /* Set up the data */
//             Data.Buffer = KdpPathBuffer;
//             Data.Length = (USHORT)PathNameLength;
//             ExtraData = &Data;
//         }
//         else
//         {
//             /* No name */
//             WaitStateChange.u.LoadSymbols.PathNameLength = 0;
//             ExtraData = NULL;
//         }
//
//         /* Setup the header */
//         Header.Length = sizeof(DBGKD_ANY_WAIT_STATE_CHANGE);
//         Header.Buffer = (PCHAR)&WaitStateChange;
//
//         /* Send the packet */
//         Status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64,
//                                      &Header,
//                                      ExtraData,
//                                      Context);
//     } while (Status == ContinueProcessorReselected);
// }
//
// VOID
// NTAPI
// KdpReportCommandStringStateChange(IN PSTRING NameString,
//                                   IN PSTRING CommandString,
//                                   IN OUT PCONTEXT Context)
// {
//     STRING Header, Data;
//     DBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange;
//     ULONG Length, ActualLength, TotalLength;
//     KCONTINUE_STATUS Status;
//
//     /* Start wait loop */
//     do
//     {
//         /* Build the architecture common parts of the message */
//         KdpSetCommonState(DbgKdCommandStringStateChange,
//                           Context,
//                           &WaitStateChange);
//
//         /* Set the context */
//         KdpSetContextState(&WaitStateChange, Context);
//
//         /* Clear the command string structure */
//         KdpZeroMemory(&WaitStateChange.u.CommandString,
//                       sizeof(DBGKD_COMMAND_STRING));
//
//         /* Normalize name string to max */
//         Length = min(128 - 1, NameString->Length);
//
//         /* Copy it to the message buffer */
//         KdpCopyMemoryChunks((ULONG_PTR)NameString->Buffer,
//                             KdpMessageBuffer,
//                             Length,
//                             0,
//                             MMDBG_COPY_UNSAFE,
//                             &ActualLength);
//
//         /* Null terminate and calculate the total length */
//         TotalLength = ActualLength;
//         KdpMessageBuffer[TotalLength++] = ANSI_NULL;
//
//         /* Check if the command string is too long */
//         Length = CommandString->Length;
//         if (Length > (PACKET_MAX_SIZE -
//                       sizeof(DBGKD_ANY_WAIT_STATE_CHANGE) - TotalLength))
//         {
//             /* Use maximum possible size */
//             Length = (PACKET_MAX_SIZE -
//                       sizeof(DBGKD_ANY_WAIT_STATE_CHANGE) - TotalLength);
//         }
//
//         /* Copy it to the message buffer */
//         KdpCopyMemoryChunks((ULONG_PTR)CommandString->Buffer,
//                             KdpMessageBuffer + TotalLength,
//                             Length,
//                             0,
//                             MMDBG_COPY_UNSAFE,
//                             &ActualLength);
//
//         /* Null terminate and calculate the total length */
//         TotalLength += ActualLength;
//         KdpMessageBuffer[TotalLength++] = ANSI_NULL;
//
//         /* Now set up the header and the data */
//         Header.Length = sizeof(DBGKD_ANY_WAIT_STATE_CHANGE);
//         Header.Buffer = (PCHAR)&WaitStateChange;
//         Data.Length = (USHORT)TotalLength;
//         Data.Buffer = KdpMessageBuffer;
//
//         /* Send State Change packet and wait for a reply */
//         Status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64,
//                                      &Header,
//                                      &Data,
//                                      Context);
//     } while (Status == ContinueProcessorReselected);
// }
//
// BOOLEAN
// NTAPI
// KdpReportExceptionStateChange(IN PEXCEPTION_RECORD ExceptionRecord,
//                               IN OUT PCONTEXT Context,
//                               IN BOOLEAN SecondChanceException)
// {
//     STRING Header, Data;
//     DBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange;
//     KCONTINUE_STATUS Status;
//
//     /* Start report loop */
//     do
//     {
//         /* Build the architecture common parts of the message */
//         KdpSetCommonState(DbgKdExceptionStateChange, Context, &WaitStateChange);
//
// #if !defined(_WIN64)
//
//         /* Convert it and copy it over */
//         ExceptionRecord32To64((PEXCEPTION_RECORD32)ExceptionRecord,
//                               &WaitStateChange.u.Exception.ExceptionRecord);
//
// #else
//
//         /* Just copy it directly, no need to convert */
//         KdpMoveMemory(&WaitStateChange.u.Exception.ExceptionRecord,
//                       ExceptionRecord,
//                       sizeof(EXCEPTION_RECORD));
//
// #endif
//
//         /* Set the First Chance flag */
//         WaitStateChange.u.Exception.FirstChance = !SecondChanceException;
//
//         /* Now finish creating the structure */
//         KdpSetContextState(&WaitStateChange, Context);
//
//         /* Setup the actual header to send to KD */
//         Header.Length = sizeof(DBGKD_ANY_WAIT_STATE_CHANGE);
//         Header.Buffer = (PCHAR)&WaitStateChange;
//
//         /* Setup the trace data */
//         DumpTraceData(&Data);
//
//         /* Send State Change packet and wait for a reply */
//         Status = KdpSendWaitContinue(PACKET_TYPE_KD_STATE_CHANGE64,
//                                      &Header,
//                                      &Data,
//                                      Context);
//     } while (Status == ContinueProcessorReselected);
//
//     /* Return */
//     return Status;
// }
//
// KCONTINUE_STATUS
// NTAPI
// KdReportProcessorChange(
//     VOID)
// {
//     PKPRCB CurrentPrcb = KeGetCurrentPrcb();
//     PCONTEXT ContextRecord = &CurrentPrcb->ProcessorState.ContextFrame;
//     EXCEPTION_RECORD ExceptionRecord = {0};
//     KCONTINUE_STATUS Status;
//
//     /* Save the port data */
//     KdSave(FALSE);
//
//     ExceptionRecord.ExceptionAddress = (PVOID)KeGetContextPc(ContextRecord);
//     ExceptionRecord.ExceptionCode = STATUS_WAKE_SYSTEM_DEBUGGER;
//
//     /* Report the new state */
//     Status = KdpReportExceptionStateChange(&ExceptionRecord,
//                                            ContextRecord,
//                                            FALSE);
//
//     /* Restore the port data */
//     KdRestore(FALSE);
//
//     return Status;
// }
//
// VOID
// NTAPI
// KdpTimeSlipDpcRoutine(IN PKDPC Dpc,
//                       IN PVOID DeferredContext,
//                       IN PVOID SystemArgument1,
//                       IN PVOID SystemArgument2)
// {
//     LONG OldSlip, NewSlip, PendingSlip;
//
//     /* Get the current pending slip */
//     PendingSlip = KdpTimeSlipPending;
//     do
//     {
//         /* Save the old value and either disable or enable it now. */
//         OldSlip = PendingSlip;
//         NewSlip = OldSlip > 1 ? 1 : 0;
//
//         /* Try to change the value */
//     } while (InterlockedCompareExchange(&KdpTimeSlipPending,
//                                         NewSlip,
//                                         OldSlip) != OldSlip);
//
//     /* If the New Slip value is 1, then do the Time Slipping */
//     if (NewSlip) ExQueueWorkItem(&KdpTimeSlipWorkItem, DelayedWorkQueue);
// }
//
// VOID
// NTAPI
// KdpTimeSlipWork(IN PVOID Context)
// {
//     KIRQL OldIrql;
//     LARGE_INTEGER DueTime;
//
//     /* Update the System time from the CMOS */
//     ExAcquireTimeRefreshLock(FALSE);
//     ExUpdateSystemTimeFromCmos(FALSE, 0);
//     ExReleaseTimeRefreshLock();
//
//     /* Check if we have a registered Time Slip Event and signal it */
//     KeAcquireSpinLock(&KdpTimeSlipEventLock, &OldIrql);
//     if (KdpTimeSlipEvent) KeSetEvent(KdpTimeSlipEvent, 0, FALSE);
//     KeReleaseSpinLock(&KdpTimeSlipEventLock, OldIrql);
//
//     /* Delay the DPC until it runs next time */
//     DueTime.QuadPart = -1800000000;
//     KeSetTimer(&KdpTimeSlipTimer, DueTime, &KdpTimeSlipDpc);
// }
//
// LARGE_INTEGER
// NTAPI
// KdpQueryPerformanceCounter(IN PKTRAP_FRAME TrapFrame)
// {
//     LARGE_INTEGER Null = {{0}};
//
//     /* Check if interrupts were disabled */
//     if (!KeGetTrapFrameInterruptState(TrapFrame))
//     {
//         /* Nothing to return */
//         return Null;
//     }
//
//     /* Otherwise, do the call */
//     return KeQueryPerformanceCounter(NULL);
// }
//
// BOOLEAN
// NTAPI
// KdEnterDebugger(IN PKTRAP_FRAME TrapFrame,
//                 IN PKEXCEPTION_FRAME ExceptionFrame)
// {
//     BOOLEAN Enable;
//
//     /* Check if we have a trap frame */
//     if (TrapFrame)
//     {
//         /* Calculate the time difference for the enter */
//         KdTimerStop = KdpQueryPerformanceCounter(TrapFrame);
//         KdTimerDifference.QuadPart = KdTimerStop.QuadPart -
//                                      KdTimerStart.QuadPart;
//     }
//     else
//     {
//         /* No trap frame, so can't calculate */
//         KdTimerStop.QuadPart = 0;
//     }
//
//     /* Save the current IRQL */
//     KeGetCurrentPrcb()->DebuggerSavedIRQL = KeGetCurrentIrql();
//
//     /* Freeze all CPUs, raising also the IRQL to HIGH_LEVEL */
//     Enable = KeFreezeExecution(TrapFrame, ExceptionFrame);
//
//     /* Lock the port, save its state and set the debugger entered flag */
//     KdpPortLocked = KeTryToAcquireSpinLockAtDpcLevel(&KdpDebuggerLock);
//     KdSave(FALSE);
//     KdEnteredDebugger = TRUE;
//
//     /* Check freeze flag */
//     if (KiFreezeFlag & 1)
//     {
//         /* Print out errror */
//         KdpDprintf("FreezeLock was jammed!  Backup SpinLock was used!\n");
//     }
//
//     /* Check processor state */
//     if (KiFreezeFlag & 2)
//     {
//         /* Print out errror */
//         KdpDprintf("Some processors not frozen in debugger!\n");
//     }
//
//     /* Make sure we acquired the port */
//     if (!KdpPortLocked) KdpDprintf("Port lock was not acquired!\n");
//
//     /* Return if interrupts needs to be re-enabled */
//     return Enable;
// }
//
// VOID
// NTAPI
// KdExitDebugger(IN BOOLEAN Enable)
// {
//     ULONG TimeSlip;
//
//     /* Reset the debugger entered flag, restore the port state and unlock it */
//     KdEnteredDebugger = FALSE;
//     KdRestore(FALSE);
//     if (KdpPortLocked) KdpPortUnlock();
//
//     /* Unfreeze the CPUs, restoring also the IRQL */
//     KeThawExecution(Enable);
//
//     /* Compare time with the one from KdEnterDebugger */
//     if (!KdTimerStop.QuadPart)
//     {
//         /* We didn't get a trap frame earlier in so never got the time */
//         KdTimerStart = KdTimerStop;
//     }
//     else
//     {
//         /* Query the timer */
//         KdTimerStart = KeQueryPerformanceCounter(NULL);
//     }
//
//     /* Check if a Time Slip was on queue */
//     TimeSlip = InterlockedIncrement(&KdpTimeSlipPending);
//     if (TimeSlip == 1)
//     {
//         /* Queue a DPC for the time slip */
//         InterlockedIncrement(&KdpTimeSlipPending);
//         KeInsertQueueDpc(&KdpTimeSlipDpc, NULL, NULL); // FIXME: this can trigger context switches!
//     }
// }
//
// NTSTATUS
// NTAPI
// KdEnableDebuggerWithLock(IN BOOLEAN NeedLock)
// {
//     KIRQL OldIrql;
//
// #if defined(__GNUC__)
//     /* Make gcc happy */
//     OldIrql = PASSIVE_LEVEL;
// #endif
//
//     /* Check if enabling the debugger is blocked */
//     if (KdBlockEnable)
//     {
//         /* It is, fail the enable */
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Check if we need to acquire the lock */
//     if (NeedLock)
//     {
//         /* Lock the port */
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//         KdpPortLock();
//     }
//
//     /* Check if we're not disabled */
//     if (!KdDisableCount)
//     {
//         /* Check if we had locked the port before */
//         if (NeedLock)
//         {
//             /* Do the unlock */
//             KdpPortUnlock();
//             KeLowerIrql(OldIrql);
//
//             /* Fail: We're already enabled */
//             return STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             /*
//              * This can only happen if we are called from a bugcheck
//              * and were never initialized, so initialize the debugger now.
//              */
//             KdInitSystem(0, NULL);
//
//             /* Return success since we initialized */
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Decrease the disable count */
//     if (!(--KdDisableCount))
//     {
//         /* We're now enabled again! Were we enabled before, too? */
//         if (KdPreviouslyEnabled)
//         {
//             /* Reinitialize the Debugger */
//             KdInitSystem(0, NULL);
//             KdpRestoreAllBreakpoints();
//         }
//     }
//
//     /* Check if we had locked the port before */
//     if (NeedLock)
//     {
//         /* Yes, now unlock it */
//         KdpPortUnlock();
//         KeLowerIrql(OldIrql);
//     }
//
//     /* We're done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KdDisableDebuggerWithLock(IN BOOLEAN NeedLock)
// {
//     KIRQL OldIrql;
//     NTSTATUS Status;
//
// #if defined(__GNUC__)
//     /* Make gcc happy */
//     OldIrql = PASSIVE_LEVEL;
// #endif
//
//     /*
//      * If enabling the debugger is blocked
//      * then there is nothing to disable (duh)
//      */
//     if (KdBlockEnable)
//     {
//         /* Fail */
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Check if we need to acquire the lock */
//     if (NeedLock)
//     {
//         /* Lock the port */
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//         KdpPortLock();
//     }
//
//     /* Check if we're not disabled */
//     if (!KdDisableCount)
//     {
//         /* Check if the debugger was never actually initialized */
//         if (!(KdDebuggerEnabled) && !(KdPitchDebugger))
//         {
//             /* It wasn't, so don't re-enable it later */
//             KdPreviouslyEnabled = FALSE;
//         }
//         else
//         {
//             /* It was, so we will re-enable it later */
//             KdPreviouslyEnabled = TRUE;
//         }
//
//         /* Check if we were called from the exported API and are enabled */
//         if ((NeedLock) && (KdPreviouslyEnabled))
//         {
//             /* Check if it is safe to disable the debugger */
//             Status = KdpAllowDisable();
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Release the lock and fail */
//                 KdpPortUnlock();
//                 KeLowerIrql(OldIrql);
//                 return Status;
//             }
//         }
//
//         /* Only disable the debugger if it is enabled */
//         if (KdDebuggerEnabled)
//         {
//             /*
//              * Disable the debugger; suspend breakpoints
//              * and reset the debug stub
//              */
//             KdpSuspendAllBreakPoints();
//             KiDebugRoutine = KdpStub;
//
//             /* We are disabled now */
//             KdDebuggerEnabled = FALSE;
//             SharedUserData->KdDebuggerEnabled = FALSE;
//         }
//      }
//
//     /* Increment the disable count */
//     KdDisableCount++;
//
//     /* Check if we had locked the port before */
//     if (NeedLock)
//     {
//         /* Yes, now unlock it */
//         KdpPortUnlock();
//         KeLowerIrql(OldIrql);
//     }
//
//     /* We're done */
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KdEnableDebugger(VOID)
// {
//     /* Use the internal routine */
//     return KdEnableDebuggerWithLock(TRUE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KdDisableDebugger(VOID)
// {
//     /* Use the internal routine */
//     return KdDisableDebuggerWithLock(TRUE);
// }
//
// /**
//  * @brief
//  * Perform various queries to the kernel debugger.
//  *
//  * @param[in]   Command
//  * A SYSDBG_COMMAND value describing the kernel debugger command to perform.
//  *
//  * @param[in]   InputBuffer
//  * Pointer to a user-provided input command-specific buffer, whose length
//  * is given by InputBufferLength.
//  *
//  * @param[in]   InputBufferLength
//  * The size (in bytes) of the buffer pointed by InputBuffer.
//  *
//  * @param[out]  OutputBuffer
//  * Pointer to a user-provided command-specific output buffer, whose length
//  * is given by OutputBufferLength.
//  *
//  * @param[in]   OutputBufferLength
//  * The size (in bytes) of the buffer pointed by OutputBuffer.
//  *
//  * @param[out]  ReturnLength
//  * Optional pointer to a ULONG variable that receives the actual length of
//  * data written written in the output buffer. It is always zero, except for
//  * the live dump commands where an actual non-zero length is returned.
//  *
//  * @param[in]   PreviousMode
//  * The processor mode (KernelMode or UserMode) in which the command is being executed.
//  *
//  * @return
//  * STATUS_SUCCESS in case of success, or a proper error code otherwise.
//  *
//  * @remarks
//  * - This is a kernel-mode function, accessible only by kernel-mode drivers.
//  *
//  * @note
//  * See: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2004-2339
//  *
//  * @see NtSystemDebugControl()
//  **/
// NTSTATUS
// NTAPI
// KdSystemDebugControl(
//     _In_ SYSDBG_COMMAND Command,
//     _In_reads_bytes_(InputBufferLength) PVOID InputBuffer,
//     _In_ ULONG InputBufferLength,
//     _Out_writes_bytes_(OutputBufferLength) PVOID OutputBuffer,
//     _In_ ULONG OutputBufferLength,
//     _Out_opt_ PULONG ReturnLength,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     NTSTATUS Status;
//     ULONG Length = 0;
//
//     /* Handle some internal commands */
//     switch ((ULONG)Command)
//     {
// #if DBG
//         case ' soR': /* ROS-INTERNAL */
//         {
//             switch ((ULONG_PTR)InputBuffer)
//             {
//                 case 0x21: // DumpAllThreads:
//                     PspDumpThreads(TRUE);
//                     break;
//
//                 case 0x22: // DumpUserThreads:
//                     PspDumpThreads(FALSE);
//                     break;
//
//                 case 0x24: // KdSpare3:
//                     MmDumpArmPfnDatabase(FALSE);
//                     break;
//
//                 default:
//                     break;
//             }
//             return STATUS_SUCCESS;
//         }
//
// #if defined(_M_IX86) && !defined(_WINKD_) // See ke/i386/traphdlr.c
//         /* Register a debug callback */
//         case 'CsoR':
//         {
//             switch (InputBufferLength)
//             {
//                 case ID_Win32PreServiceHook:
//                     KeWin32PreServiceHook = InputBuffer;
//                     break;
//
//                 case ID_Win32PostServiceHook:
//                     KeWin32PostServiceHook = InputBuffer;
//                     break;
//
//             }
//             break;
//         }
// #endif
//
//         /* Special case for stack frame dumps */
//         case 'DsoR':
//         {
//             KeRosDumpStackFrames((PULONG_PTR)InputBuffer, InputBufferLength);
//             break;
//         }
// #ifdef KDBG
//         /* Register KDBG CLI callback */
//         case 'RbdK':
//         {
//             return KdbRegisterCliCallback(InputBuffer, InputBufferLength);
//         }
// #endif // KDBG
// #endif
//         default:
//             break;
//     }
//
//     switch (Command)
//     {
//         case SysDbgQueryVersion:
//             if (OutputBufferLength != sizeof(DBGKD_GET_VERSION64))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 KdpSysGetVersion((PDBGKD_GET_VERSION64)OutputBuffer);
//                 Status = STATUS_SUCCESS;
//             }
//             break;
//
//         case SysDbgReadVirtual:
//         case SysDbgWriteVirtual:
//             if (InputBufferLength != sizeof(SYSDBG_VIRTUAL))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_VIRTUAL Request = *(PSYSDBG_VIRTUAL)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           Command == SysDbgReadVirtual ? IoWriteAccess : IoReadAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpCopyMemoryChunks((ULONG64)(ULONG_PTR)Request.Address,
//                                                  Request.Buffer,
//                                                  Request.Request,
//                                                  0,
//                                                  Command == SysDbgReadVirtual ? 0 : MMDBG_COPY_WRITE,
//                                                  &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgReadPhysical:
//         case SysDbgWritePhysical:
//             if (InputBufferLength != sizeof(SYSDBG_PHYSICAL))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_PHYSICAL Request = *(PSYSDBG_PHYSICAL)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           Command == SysDbgReadVirtual ? IoWriteAccess : IoReadAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpCopyMemoryChunks(Request.Address.QuadPart,
//                                                  Request.Buffer,
//                                                  Request.Request,
//                                                  0,
//                                                  MMDBG_COPY_PHYSICAL | (Command == SysDbgReadVirtual ? 0 : MMDBG_COPY_WRITE),
//                                                  &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgReadControlSpace:
//             if (InputBufferLength != sizeof(SYSDBG_CONTROL_SPACE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_CONTROL_SPACE Request = *(PSYSDBG_CONTROL_SPACE)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoWriteAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysReadControlSpace(Request.Processor,
//                                                     Request.Address,
//                                                     LockedBuffer,
//                                                     Request.Request,
//                                                     &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgWriteControlSpace:
//             if (InputBufferLength != sizeof(SYSDBG_CONTROL_SPACE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_CONTROL_SPACE Request = *(PSYSDBG_CONTROL_SPACE)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoReadAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysWriteControlSpace(Request.Processor,
//                                                      Request.Address,
//                                                      LockedBuffer,
//                                                      Request.Request,
//                                                      &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgReadIoSpace:
//             if (InputBufferLength != sizeof(SYSDBG_IO_SPACE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_IO_SPACE Request = *(PSYSDBG_IO_SPACE)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoWriteAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysReadIoSpace(Request.InterfaceType,
//                                                Request.BusNumber,
//                                                Request.AddressSpace,
//                                                Request.Address,
//                                                LockedBuffer,
//                                                Request.Request,
//                                                &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgWriteIoSpace:
//             if (InputBufferLength != sizeof(SYSDBG_IO_SPACE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_IO_SPACE Request = *(PSYSDBG_IO_SPACE)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoReadAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysWriteIoSpace(Request.InterfaceType,
//                                                 Request.BusNumber,
//                                                 Request.AddressSpace,
//                                                 Request.Address,
//                                                 LockedBuffer,
//                                                 Request.Request,
//                                                 &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgReadMsr:
//             if (InputBufferLength != sizeof(SYSDBG_MSR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 PSYSDBG_MSR Request = (PSYSDBG_MSR)InputBuffer;
//                 Status = KdpSysReadMsr(Request->Address, &Request->Data);
//             }
//             break;
//
//         case SysDbgWriteMsr:
//             if (InputBufferLength != sizeof(SYSDBG_MSR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 PSYSDBG_MSR Request = (PSYSDBG_MSR)InputBuffer;
//                 Status = KdpSysWriteMsr(Request->Address, &Request->Data);
//             }
//             break;
//
//         case SysDbgReadBusData:
//             if (InputBufferLength != sizeof(SYSDBG_BUS_DATA))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_BUS_DATA Request = *(PSYSDBG_BUS_DATA)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoWriteAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysReadBusData(Request.BusDataType,
//                                                Request.BusNumber,
//                                                Request.SlotNumber,
//                                                Request.Address,
//                                                LockedBuffer,
//                                                Request.Request,
//                                                &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgWriteBusData:
//             if (InputBufferLength != sizeof(SYSDBG_BUS_DATA))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             else
//             {
//                 SYSDBG_BUS_DATA Request = *(PSYSDBG_BUS_DATA)InputBuffer;
//                 PVOID LockedBuffer;
//                 PMDL LockVariable;
//
//                 Status = ExLockUserBuffer(Request.Buffer,
//                                           Request.Request,
//                                           PreviousMode,
//                                           IoReadAccess,
//                                           &LockedBuffer,
//                                           &LockVariable);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = KdpSysWriteBusData(Request.BusDataType,
//                                                 Request.BusNumber,
//                                                 Request.SlotNumber,
//                                                 Request.Address,
//                                                 LockedBuffer,
//                                                 Request.Request,
//                                                 &Length);
//                     ExUnlockUserBuffer(LockVariable);
//                 }
//             }
//             break;
//
//         case SysDbgCheckLowMemory:
//             Status = KdpSysCheckLowMemory(0);
//             break;
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
//     if (ReturnLength)
//         *ReturnLength = Length;
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KdChangeOption(IN KD_OPTION Option,
//                IN ULONG InBufferBytes OPTIONAL,
//                IN PVOID InBuffer,
//                IN ULONG OutBufferBytes OPTIONAL,
//                OUT PVOID OutBuffer,
//                OUT PULONG OutBufferNeeded OPTIONAL)
// {
//     /* Fail if there is no debugger */
//     if (KdPitchDebugger)
//     {
//         /* No debugger, no options */
//         return STATUS_DEBUGGER_INACTIVE;
//     }
//
//     /* Do we recognize this option? */
//     if (Option != KD_OPTION_SET_BLOCK_ENABLE)
//     {
//         /* We don't, clear the output length and fail */
//         if (OutBufferNeeded) *OutBufferNeeded = 0;
//         return STATUS_INVALID_INFO_CLASS;
//     }
//
//     /* Verify parameters */
//     if ((InBufferBytes != sizeof(BOOLEAN)) ||
//         (OutBufferBytes != 0) ||
//         (OutBuffer != NULL))
//     {
//         /* Invalid parameters for this option, fail */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /*
//      * Check if the high bit is set, meaning we don't
//      * allow the debugger to be enabled
//      */
//     if (KdBlockEnable & 0x80)
//     {
//         /* Fail regardless of what state the caller tried to set */
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Set the new block enable state */
//     KdBlockEnable = *(PBOOLEAN)InBuffer;
//
//     /* No output buffer required for this option */
//     if (OutBufferNeeded) *OutBufferNeeded = 0;
//
//     /* We are done */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KdPowerTransition(IN DEVICE_POWER_STATE NewState)
// {
//     /* Check what power state this is */
//     if (NewState == PowerDeviceD0)
//     {
//         /* Wake up the debug port */
//         KdD0Transition();
//         return STATUS_SUCCESS;
//     }
//     else if ((NewState == PowerDeviceD1) ||
//              (NewState == PowerDeviceD2) ||
//              (NewState == PowerDeviceD3))
//     {
//         /* Power down the debug port */
//         KdD3Transition();
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Invalid state! */
//         return STATUS_INVALID_PARAMETER_1;
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KdRefreshDebuggerNotPresent(VOID)
// {
//     BOOLEAN Enable, DebuggerNotPresent;
//
//     /* Check if the debugger is completely disabled */
//     if (KdPitchDebugger)
//     {
//         /* Don't try to refresh then, fail early */
//         return TRUE;
//     }
//
//     /* Enter the debugger */
//     Enable = KdEnterDebugger(NULL, NULL);
//
//     /*
//      * Attempt to send a string to the debugger
//      * to refresh the connection state.
//      */
//     KdpDprintf("KDTARGET: Refreshing KD connection\n");
//
//     /* Save the state while we are holding the lock */
//     DebuggerNotPresent = KdDebuggerNotPresent;
//
//     /* Exit the debugger and return the state */
//     KdExitDebugger(Enable);
//     return DebuggerNotPresent;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryDebugFilterState(
//     _In_ ULONG ComponentId,
//     _In_ ULONG Level)
// {
//     PULONG Mask;
//
//     /* Check if the ID fits in the component table */
//     if (ComponentId < KdComponentTableSize)
//     {
//         /* It does, so get the mask from there */
//         Mask = KdComponentTable[ComponentId];
//     }
//     else if (ComponentId == MAXULONG)
//     {
//         /*
//          * This is the internal ID used for DbgPrint messages without ID
//          * and Level. Use the system-wide mask for those.
//          */
//         Mask = &Kd_WIN2000_Mask;
//     }
//     else
//     {
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//         /* Use the default component ID */
//         Mask = &Kd_DEFAULT_Mask;
//         // Level = DPFLTR_INFO_LEVEL; // Override the Level.
// #else
//         /* Invalid ID, fail */
//         return STATUS_INVALID_PARAMETER_1;
// #endif
//     }
//
//     /* Convert Level to bit field if required */
//     if (Level < 32) Level = 1 << Level;
//     Level &= ~DPFLTR_MASK;
//
//     /* Determine if this Level is filtered out */
//     if ((Kd_WIN2000_Mask & Level) || (*Mask & Level))
//     {
//         /* This mask will get through to the debugger */
//         return (NTSTATUS)TRUE;
//     }
//     else
//     {
//         /* This mask is filtered out */
//         return (NTSTATUS)FALSE;
//     }
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetDebugFilterState(
//     _In_ ULONG ComponentId,
//     _In_ ULONG Level,
//     _In_ BOOLEAN State)
// {
//     PULONG Mask;
//
//     /* Modifying debug filters requires the debug privilege */
//     if (!SeSinglePrivilegeCheck(SeDebugPrivilege, ExGetPreviousMode()))
//     {
//         /* Fail */
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Check if the ID fits in the component table */
//     if (ComponentId < KdComponentTableSize)
//     {
//         /* It does, so get the mask from there */
//         Mask = KdComponentTable[ComponentId];
//     }
//     else if (ComponentId == MAXULONG)
//     {
//         /*
//          * This is the internal ID used for DbgPrint messages without ID
//          * and Level. Use the system-wide mask for those.
//          */
//         Mask = &Kd_WIN2000_Mask;
//     }
//     else
//     {
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//         /* Use the default component ID */
//         Mask = &Kd_DEFAULT_Mask;
// #else
//         /* Invalid ID, fail */
//         return STATUS_INVALID_PARAMETER_1;
// #endif
//     }
//
//     /* Convert Level to bit field if required */
//     if (Level < 32) Level = 1 << Level;
//     Level &= ~DPFLTR_MASK;
//
//     /* Set or remove the Level */
//     if (State)
//         *Mask |= Level;
//     else
//         *Mask &= ~Level;
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
