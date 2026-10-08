use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KeInitializeSpinLock, KeQueryTimeIncrement, LARGE_INTEGER, NTSTRSAFE_PSTR, PCCHAR, RtlCaptureStackBackTrace, va_list};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING};
use crate::types::{DBG_NUM_PACKET_LOG_ENTRIES, PCDB, PCLASS_PRIVATE_FDO_DATA, PFUNCTIONAL_DEVICE_EXTENSION, PSTORAGE_REQUEST_BLOCK_HEADER, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_DATA_OVERRUN, SRB_STATUS_INTERNAL_ERROR, SRB_STATUS_PENDING, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SUCCESS, TRANSFER_PACKET, _CLASS_DEBUG_LEVEL as CLASS_DEBUG_LEVEL, _CLASS_PRIVATE_FDO_DATA, SCSI_ADSENSE_COPY_PROTECTION_FAILURE, SCSI_ADSENSE_ILLEGAL_BLOCK, SCSI_ADSENSE_INVALID_MEDIA, SCSI_ADSENSE_LUN_NOT_READY, SCSI_ADSENSE_NO_SENSE, SCSI_ADSENSE_OPERATOR_REQUEST, SCSI_ADSENSE_POSITION_ERROR, SCSI_SENSEQ_AUTHENTICATION_FAILURE, SCSI_SENSEQ_BECOMING_READY, SCSI_SENSEQ_BEGINNING_OF_MEDIA_DETECTED, SCSI_SENSEQ_CAUSE_NOT_REPORTABLE, SCSI_SENSEQ_CLEANING_CARTRIDGE_INSTALLED, SCSI_SENSEQ_DESTINATION_FULL, SCSI_SENSEQ_END_OF_MEDIA_DETECTED, SCSI_SENSEQ_FILEMARK_DETECTED, SCSI_SENSEQ_FORMAT_IN_PROGRESS, SCSI_SENSEQ_ILLEGAL_ELEMENT_ADDR, SCSI_SENSEQ_INCOMPATIBLE_FORMAT, SCSI_SENSEQ_INCOMPATIBLE_MEDIA_INSTALLED, SCSI_SENSEQ_INIT_COMMAND_REQUIRED, SCSI_SENSEQ_KEY_NOT_ESTABLISHED, SCSI_SENSEQ_KEY_NOT_PRESENT, SCSI_SENSEQ_LOGICAL_UNIT_RESET_COUNT_ERROR, SCSI_SENSEQ_LONG_WRITE_IN_PROGRESS, SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED, SCSI_SENSEQ_MEDIA_CODE_MISMATCHED_TO_LOGICAL_UNIT, SCSI_SENSEQ_MEDIUM_REMOVAL, SCSI_SENSEQ_OPERATION_IN_PROGRESS, SCSI_SENSEQ_READ_OF_SCRAMBLED_SECTOR_WITHOUT_AUTHENTICATION, SCSI_SENSEQ_REBUILD_IN_PROGRESS, SCSI_SENSEQ_RECALCULATION_IN_PROGRESS, SCSI_SENSEQ_SETMARK_DETECTED, SCSI_SENSEQ_SOURCE_EMPTY, SCSI_SENSEQ_STATE_CHANGE_INPUT, SCSI_SENSEQ_UNKNOWN_FORMAT, SCSI_SENSEQ_WRITE_PROTECT_DISABLE, SCSI_SENSEQ_WRITE_PROTECT_ENABLE};
use ntkrnl_sys::ddk::{PCSTR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_INIT, TRACE_FLAG_RW};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DISK_SPINUP_BACKTRACE_LENGTH, NUMBER_OF_DISK_SPINUP_TRACES};
use crate::types::{SCSIOP_MEDIUM_REMOVAL, SCSIOP_MODE_SENSE, SCSIOP_READ, SCSIOP_READ16, SCSIOP_READ_CAPACITY, SCSIOP_READ_CAPACITY16, SCSIOP_START_STOP_UNIT, SCSIOP_WRITE, SCSIOP_WRITE16};
use ntkrnl_sys::ddk::{_LARGE_INTEGER};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::driver_ddk::{DbgPrint};
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetDataBuffer, SrbGetDataTransferLength, SrbGetOriginalRequest, SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbGetSystemStatus};

// ---- local struct definitions (from debug.h, not in generated types) ----
#[repr(C)]
pub struct CLASSPNP_GLOBALS {
    pub BreakOnLostIrps: ULONG,
    pub SecondsToWaitForIrps: ULONG,
    pub UseBufferedDebugPrint: ULONG,
    pub UseDelayedRetry: ULONG,
    pub Index: ULONG,
    pub SpinLock: KSPIN_LOCK,
    pub Buffer: *mut UCHAR,
    pub NumberOfBuffers: ULONG,
    pub EachBufferSize: SIZE_T,
    pub Initializing: LONG,
    pub Initialized: LONG,
}
unsafe impl Sync for CLASSPNP_GLOBALS {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DISK_SPINUP_TRACES {
    pub TimeStamp: LARGE_INTEGER,
    pub StackTrace: [PVOID; 24],
}
unsafe impl Sync for DISK_SPINUP_TRACES {}
pub type _DISK_SPINUP_TRACES = DISK_SPINUP_TRACES;

extern "C" {
    pub fn ClasspTransferPacketGetCdb(Pkt: *mut TRANSFER_PACKET) -> PCDB;
    pub fn DbgGetSrbStatusStr(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
    pub fn DbgGetSenseCodeStr(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
    pub fn DbgGetAdditionalSenseCodeStr(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
    pub fn RtlULongAdd(Augend: ULONG, Addend: ULONG, pResult: *mut ULONG) -> NTSTATUS;
    
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut ClasspnpGlobals: CLASSPNP_GLOBALS = CLASSPNP_GLOBALS {
    BreakOnLostIrps: 0,
    SecondsToWaitForIrps: 0,
    UseBufferedDebugPrint: 0,
    UseDelayedRetry: 0,
    Index: 0,
    SpinLock: 0,
    Buffer: core::ptr::null_mut(),
    NumberOfBuffers: 0,
    EachBufferSize: 0,
    Initializing: 0,
    Initialized: 0,
};

pub static mut ClassDebug: LONG = 0 as LONG;

pub static mut DebugTrapOnWarn: BOOLEAN = FALSE as BOOLEAN;

pub static mut DiskSpinupIndex: ULONG = 0 as ULONG;

pub static mut DiskSpinupTraces: [DISK_SPINUP_TRACES; 16] = [DISK_SPINUP_TRACES {
    TimeStamp: LARGE_INTEGER { QuadPart: 0 },
    StackTrace: [core::ptr::null_mut(); 24],
}; 16];

pub const CLASS_GLOBAL_BREAK_ON_LOST_IRPS: u32 = 0;
pub const CLASS_GLOBAL_SECONDS_TO_WAIT_FOR_SYNCHRONOUS_SRB: u32 = 300;
pub const CLASS_GLOBAL_BUFFERED_DEBUG_PRINT: u32 = 0;
pub const CLASS_GLOBAL_BUFFERED_DEBUG_PRINT_BUFFER_SIZE: u32 = 512;
pub const CLASS_GLOBAL_BUFFERED_DEBUG_PRINT_BUFFERS: u32 = 512;
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeDebugGlobals() {
    let mut irql: u8 = 0; // KIRQL
    let initializing_ptr = core::ptr::addr_of_mut!(ClasspnpGlobals.Initializing);
    let cmp_result: i32;

    // InterlockedCompareExchange(Initializing, 1, 0) — one-time init guard.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!(
            "lock cmpxchg [{dest}], {src}",
            dest = in(reg) initializing_ptr,
            src = in(reg) 1i32,
            inout("eax") 0i32 => cmp_result,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let atomic = &*(initializing_ptr as *const core::sync::atomic::AtomicI32);
        cmp_result = match atomic.compare_exchange(
            0, 1,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(v) => v,
            Err(v) => v,
        };
    }

    if cmp_result == 0 {
        KeInitializeSpinLock(core::ptr::addr_of_mut!(ClasspnpGlobals.SpinLock).cast());
        KeAcquireSpinLock(core::ptr::addr_of_mut!(ClasspnpGlobals.SpinLock).cast(), &mut irql);
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, b"CLASSPNP.SYS => Initializing ClasspnpGlobals...\n\0".as_ptr() as *const i8);
        ClasspnpGlobals.Buffer = core::ptr::null_mut::<u8>();
        ClasspnpGlobals.Index = !0u32; // (ULONG)-1
        ClasspnpGlobals.BreakOnLostIrps = 0u32;
        ClasspnpGlobals.EachBufferSize = 512usize;
        ClasspnpGlobals.NumberOfBuffers = 512u32;
        ClasspnpGlobals.SecondsToWaitForIrps = 300u32;
        ClasspnpGlobals.UseBufferedDebugPrint = 0u32;
        KeReleaseSpinLock(core::ptr::addr_of_mut!(ClasspnpGlobals.SpinLock), irql);

        // InterlockedExchange(Initialized, 1)
        let initialized_ptr = core::ptr::addr_of_mut!(ClasspnpGlobals.Initialized);
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            core::arch::asm!(
                "lock xchg [{dest}], {src}",
                dest = in(reg) initialized_ptr,
                src = inout(reg) 1i32 => _,
                options(nostack, preserves_flags)
            );
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            (&*(initialized_ptr as *const core::sync::atomic::AtomicI32))
                .swap(1, core::sync::atomic::Ordering::SeqCst);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DbgCheckReturnedPkt(Pkt: *mut TRANSFER_PACKET) {
    let pCdb: PCDB = ClasspTransferPacketGetCdb(Pkt);

    // NT_ASSERT macros expanded
    if !(SrbGetOriginalRequest((*Pkt).Srb as *mut core::ffi::c_void)
        == (*Pkt).Irp as *mut core::ffi::c_void)
    {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                as PCSTR,
            240i32,
            b"SrbGetOriginalRequest(Pkt->Srb) == Pkt->Irp\0".as_ptr() as PCSTR,
        );
        __int2c();
    };
    if !(SrbGetDataBuffer((*Pkt).Srb as *mut core::ffi::c_void)
        == (*Pkt).BufPtrCopy as *mut core::ffi::c_void)
    {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                as PCSTR,
            241i32,
            b"SrbGetDataBuffer(Pkt->Srb) == Pkt->BufPtrCopy\0".as_ptr() as PCSTR,
        );
        __int2c();
    };
    if !(SrbGetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void) <= (*Pkt).BufLenCopy) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                as PCSTR,
            242i32,
            b"SrbGetDataTransferLength(Pkt->Srb) <= Pkt->BufLenCopy\0".as_ptr() as PCSTR,
        );
        __int2c();
    };
    if (*(*Pkt).Irp).CancelRoutine.is_some() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                as PCSTR,
            243i32,
            b"!Pkt->Irp->CancelRoutine\0".as_ptr() as PCSTR,
        );
        __int2c();
    };

    // Extract masked SRB status (SRB_STATUS macro)
    let srb_status: i32 = (*(*Pkt).Srb).SrbStatus as i32
        & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32;

    if srb_status == (SRB_STATUS_PENDING) as i32 {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_RW,
            b"SRB completed with status PENDING in packet %ph: (op=%s srbstat=%s(%xh), irpstat=%xh)\0".as_ptr()
                as PCCHAR,
            Pkt,
            DbgGetScsiOpStr(((*Pkt).Srb).cast()),
            DbgGetSrbStatusStr((*Pkt).Srb),
            (*(*Pkt).Srb).SrbStatus as ULONG,
            (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
        );
    } else if srb_status == (SRB_STATUS_SUCCESS) as i32 {
        // Make sure SRB and IRP status match
        if !((*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status >= 0) {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"SRB and IRP status don't match in packet %ph: (op=%s srbstat=%s(%xh), irpstat=%xh)\0".as_ptr()
                    as PCCHAR,
                Pkt,
                DbgGetScsiOpStr(((*Pkt).Srb).cast()),
                DbgGetSrbStatusStr((*Pkt).Srb),
                (*(*Pkt).Srb).SrbStatus as ULONG,
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
            );
        }
        if (*(*Pkt).Irp).IoStatus.Information
            != (((SrbGetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void) as u64) as u32) as u64) as _
        {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"SRB and IRP result transfer lengths don't match in succeeded packet %ph: (op=%s, SrbStatus=%s, Srb.DataTransferLength=%xh, Irp->IoStatus.Information=%Ixh).\0".as_ptr()
                    as PCCHAR,
                Pkt,
                DbgGetScsiOpStr(((*Pkt).Srb).cast()),
                DbgGetSrbStatusStr((*Pkt).Srb),
                SrbGetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void),
                (*(*Pkt).Irp).IoStatus.Information,
            );
        }
    } else {
        if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status >= 0 {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_RW,
                b"SRB and IRP status don't match in packet %ph: (op=%s srbstat=%s(%xh), irpstat=%xh)\0".as_ptr()
                    as PCCHAR,
                Pkt,
                DbgGetScsiOpStr(((*Pkt).Srb).cast()),
                DbgGetSrbStatusStr((*Pkt).Srb),
                (*(*Pkt).Srb).SrbStatus as ULONG,
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
            );
        }
        {
            // Loud numeric twin of the (arg-less) TracePrint below: the stub only
            // prints the format string, which localizes but doesn't identify the op.
            unsafe extern "C" { fn DbgPrint(Format: *const i8, ...) -> u32; }
            let cdb = crate::ddk_shims::SrbGetCdb(((*Pkt).Srb).cast());
            DbgPrint(
                c"[CLASSPNP] pkt FAIL op=0x%02x srbstat=0x%02x irpstat=0x%x\n".as_ptr(),
                if cdb.is_null() { 0xFFu32 } else { (*(cdb as *const u8)) as u32 },
                (*(*Pkt).Srb).SrbStatus as ULONG,
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
            );
        }
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_RW,
            b"Packet %ph failed (op=%s srbstat=%s(%xh), irpstat=%xh, sense=%s/%s/%s)\0".as_ptr()
                as PCCHAR,
            Pkt,
            DbgGetScsiOpStr(((*Pkt).Srb).cast()),
            DbgGetSrbStatusStr((*Pkt).Srb),
            (*(*Pkt).Srb).SrbStatus as ULONG,
            (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
            DbgGetSenseCodeStr((*Pkt).Srb),
            DbgGetAdditionalSenseCodeStr((*Pkt).Srb),
            DbgGetAdditionalSenseCodeQualifierStr((*Pkt).Srb),
        );

        // If SRB failed with underrun/overrun, check transferred length
        if srb_status == (SRB_STATUS_DATA_OVERRUN) as i32
            && (*(*Pkt).Irp).IoStatus.Information
                != (((SrbGetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void) as u64) as u32) as u64) as _
        {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"SRB and IRP result transfer lengths don't match in failed packet %ph: (op=%s, SrbStatus=%s, Srb.DataTransferLength=%xh, Irp->IoStatus.Information=%Ixh).\0".as_ptr()
                    as PCCHAR,
                Pkt,
                DbgGetScsiOpStr(((*Pkt).Srb).cast()),
                DbgGetSrbStatusStr((*Pkt).Srb),
                SrbGetDataTransferLength((*Pkt).Srb as *mut core::ffi::c_void),
                (*(*Pkt).Irp).IoStatus.Information,
            );
        }
    }

    // Check STATUS_INSUFFICIENT_RESOURCES correlation
    if (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status as i32 == STATUS_INSUFFICIENT_RESOURCES as i32 {
        if !(srb_status == (SRB_STATUS_INTERNAL_ERROR) as i32) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                    as PCSTR,
                315i32,
                b"SRB_STATUS(Pkt->Srb->SrbStatus) == SRB_STATUS_INTERNAL_ERROR\0".as_ptr()
                    as PCSTR,
            );
            __int2c();
        };
        if !(SrbGetSystemStatus((*Pkt).Srb as *mut core::ffi::c_void)
            == STATUS_INSUFFICIENT_RESOURCES as u32)
        {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/debug.c\0".as_ptr()
                    as PCSTR,
                316i32,
                b"SrbGetSystemStatus(Pkt->Srb) == STATUS_INSUFFICIENT_RESOURCES\0".as_ptr()
                    as PCSTR,
            );
            __int2c();
        };
    }

    // Check miniport did not change SCSI operation code
    match (*pCdb).CDB10.as_ref().OperationCode as u32 {
        SCSIOP_MEDIUM_REMOVAL
        | SCSIOP_MODE_SENSE
        | SCSIOP_READ_CAPACITY
        | SCSIOP_READ
        | SCSIOP_WRITE
        | SCSIOP_START_STOP_UNIT
        | SCSIOP_READ_CAPACITY16
        | SCSIOP_READ16
        | SCSIOP_WRITE16 => {}
        _ => {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"Miniport illegally changed Srb.Cdb.OperationCode in packet %ph failed (op=%s srbstat=%s(%xh), irpstat=%xh, sense=%s/%s/%s)\0".as_ptr()
                    as PCCHAR,
                Pkt,
                DbgGetScsiOpStr(((*Pkt).Srb).cast()),
                DbgGetSrbStatusStr((*Pkt).Srb),
                (*(*Pkt).Srb).SrbStatus as ULONG,
                (*(*Pkt).Irp).IoStatus.__bindgen_anon_1.Status,
                DbgGetSenseCodeStr((*Pkt).Srb),
                DbgGetAdditionalSenseCodeStr((*Pkt).Srb),
                DbgGetAdditionalSenseCodeQualifierStr((*Pkt).Srb),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DbgLogSendPacket(Pkt: *mut TRANSFER_PACKET) {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = ((*Pkt).Fdo.read().DeviceExtension).cast();
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;

    if !(*Pkt).OriginalIrp.is_null() {
        (*Pkt).DbgOriginalIrpCopy = *(*Pkt).OriginalIrp ;
        if !(*(*Pkt).OriginalIrp).MdlAddress.is_null() {
            (*Pkt).DbgMdlCopy = *(*(*Pkt).OriginalIrp).MdlAddress ;
        }
    }

    KeQueryTickCount(&raw mut (*Pkt).DbgTimeSent as *mut i64);
    (*Pkt).DbgTimeReturned.QuadPart = 0;

    KeAcquireSpinLock((&raw mut (*fdoData).SpinLock).cast(), &raw mut oldIrql);
    let idx = (*fdoData).DbgPacketLogNextIndex as usize;
    core::ptr::copy_nonoverlapping(Pkt, &mut (*fdoData).DbgPacketLogs[idx], 1);
    (*fdoData).DbgPacketLogNextIndex = (*fdoData).DbgPacketLogNextIndex.wrapping_add(1);
    (*fdoData).DbgPacketLogNextIndex = (*fdoData).DbgPacketLogNextIndex.wrapping_rem(DBG_NUM_PACKET_LOG_ENTRIES);
    KeReleaseSpinLock((&raw mut (*fdoData).SpinLock).cast(), oldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn DbgLogReturnPacket(Pkt: *mut TRANSFER_PACKET) {
    let fdoExt = (*(*Pkt).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;

    KeQueryTickCount(core::ptr::addr_of_mut!((*Pkt).DbgTimeReturned.QuadPart));

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), core::ptr::addr_of_mut!(oldIrql));
    let idx = (*fdoData).DbgPacketLogNextIndex as usize;
    core::ptr::copy_nonoverlapping(Pkt, &mut (*fdoData).DbgPacketLogs[idx], 1);
    (*fdoData).DbgPacketLogNextIndex = (*fdoData).DbgPacketLogNextIndex.wrapping_add(1);
    (*fdoData).DbgPacketLogNextIndex = ((*fdoData).DbgPacketLogNextIndex as usize)
        .wrapping_rem((DBG_NUM_PACKET_LOG_ENTRIES) as usize) as ULONG;
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
}

unsafe fn DbgSafeInc(pValue: *mut u32) {
    let mut incrementResult: u32 = 0;
    if NT_SUCCESS(RtlULongAdd(*pValue, 1, &mut incrementResult)) {
        *pValue = incrementResult;
    }
}

#[no_mangle]
pub unsafe extern "C" fn DbgLogFlushInfo(
    FdoData: *mut _CLASS_PRIVATE_FDO_DATA,
    IsIO: u8,
    IsFUA: u8,
    IsFlush: u8,
) {
    if (*FdoData).DbgInitFlushLogging != 0 {
        (*FdoData).DbgNumIORequests = 0;
        (*FdoData).DbgNumFUAs = 0;
        (*FdoData).DbgNumFlushes = 0;
        (*FdoData).DbgIOsSinceFUA = 0;
        (*FdoData).DbgIOsSinceFlush = 0;
        (*FdoData).DbgAveIOsToFUA = 0;
        (*FdoData).DbgAveIOsToFlush = 0;
        (*FdoData).DbgMaxIOsToFUA = 0;
        (*FdoData).DbgMaxIOsToFlush = 0;
        (*FdoData).DbgMinIOsToFUA = 0xffffffff;
        (*FdoData).DbgMinIOsToFlush = 0xffffffff;
        (*FdoData).DbgInitFlushLogging = 0u8;
    }

    if IsIO != 0 {
        DbgSafeInc(core::ptr::addr_of_mut!((*FdoData).DbgNumIORequests));
        DbgSafeInc(core::ptr::addr_of_mut!((*FdoData).DbgIOsSinceFlush));

        if IsFUA != 0 {
            if (*FdoData).DbgNumFUAs > 0 {
                (*FdoData).DbgMinIOsToFUA = ::core::cmp::min(
                    (*FdoData).DbgMinIOsToFUA,
                    (*FdoData).DbgIOsSinceFUA,
                );
            }
            DbgSafeInc(core::ptr::addr_of_mut!((*FdoData).DbgNumFUAs));
            (*FdoData).DbgAveIOsToFUA = (*FdoData).DbgNumIORequests.wrapping_div((*FdoData).DbgNumFUAs);
            (*FdoData).DbgIOsSinceFUA = 0;
        } else {
            DbgSafeInc(core::ptr::addr_of_mut!((*FdoData).DbgIOsSinceFUA));
            (*FdoData).DbgMaxIOsToFUA = ::core::cmp::max(
                (*FdoData).DbgMaxIOsToFUA,
                (*FdoData).DbgIOsSinceFUA,
            );
        }

        (*FdoData).DbgMaxIOsToFlush = ::core::cmp::max(
            (*FdoData).DbgMaxIOsToFlush,
            (*FdoData).DbgIOsSinceFlush,
        );
    } else if IsFlush != 0 {
        if (*FdoData).DbgNumFlushes > 0 {
            (*FdoData).DbgMinIOsToFlush = ::core::cmp::min(
                (*FdoData).DbgMinIOsToFlush,
                (*FdoData).DbgIOsSinceFlush,
            );
        }
        DbgSafeInc(core::ptr::addr_of_mut!((*FdoData).DbgNumFlushes));
        (*FdoData).DbgAveIOsToFlush = (*FdoData).DbgNumIORequests.wrapping_div((*FdoData).DbgNumFlushes);
        (*FdoData).DbgIOsSinceFlush = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SnapDiskStartup() {
    let mut index: ULONG;
    let entry: *mut _DISK_SPINUP_TRACES;
    let mut spin_up_time: LARGE_INTEGER = core::mem::zeroed();

    index = InterlockedIncrement(&raw mut DiskSpinupIndex as *mut ::core::ffi::c_long) as ULONG;
    index = index & ((NUMBER_OF_DISK_SPINUP_TRACES - 1) as ULONG);
    entry = &raw mut DiskSpinupTraces[index as usize];

    KeQueryTickCount(&raw mut spin_up_time.QuadPart as *mut i64);
    spin_up_time.QuadPart =
        (spin_up_time.QuadPart as i64 * KeQueryTimeIncrement() as i64) / (10000000i64);

    (*entry).TimeStamp.QuadPart = spin_up_time.QuadPart;
    core::ptr::write_bytes(
        (*entry).StackTrace.as_mut_ptr() as *mut u8,
        0u8,
        (DISK_SPINUP_BACKTRACE_LENGTH as usize).wrapping_mul(core::mem::size_of::<PVOID>()),
    );
    RtlCaptureStackBackTrace(
        5u32,
        DISK_SPINUP_BACKTRACE_LENGTH as ULONG,
        (*entry).StackTrace.as_mut_ptr(),
        &raw mut index,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ClassDebugPrint(
    _DebugPrintLevel: CLASS_DEBUG_LEVEL,
    _DebugMessage: PCCHAR,
    // NOTE: stable Rust cannot define C-variadic fns; this retail stub ignores all args.
    // cdecl caller cleans up any extra args so the ABI is safe.
) {
    // Retail version: does nothing.
}

pub unsafe fn DbgGetAdditionalSenseCodeQualifierStr(srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR {
    let mut ad_sense_code_qual_str: PCHAR = b"?\0".as_ptr() as PCHAR;
    if ((*srb).SrbStatus & (SRB_STATUS_AUTOSENSE_VALID) as u8) != 0 {
        let sense_data: PVOID = SrbGetSenseInfoBuffer((srb).cast());
        ntkrnl_sys::ASSERT!(!sense_data.is_null());
        let mut ad_sense_code: u8 = 0;
        let mut ad_sense_code_qual: u8 = 0;
        let valid_sense = ScsiGetSenseKeyAndCodes(
            sense_data,
            SrbGetSenseInfoBufferLength((srb).cast()),
            SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(ad_sense_code),
            core::ptr::addr_of_mut!(ad_sense_code_qual),
        ) != 0;
        if valid_sense {
            match ad_sense_code as u32 {
                SCSI_ADSENSE_LUN_NOT_READY => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_CAUSE_NOT_REPORTABLE => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_CAUSE_NOT_REPORTABLE\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_BECOMING_READY => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_BECOMING_READY\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_INIT_COMMAND_REQUIRED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_INIT_COMMAND_REQUIRED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_FORMAT_IN_PROGRESS => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_FORMAT_IN_PROGRESS\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_REBUILD_IN_PROGRESS => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_REBUILD_IN_PROGRESS\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_RECALCULATION_IN_PROGRESS => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_RECALCULATION_IN_PROGRESS\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_OPERATION_IN_PROGRESS => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_OPERATION_IN_PROGRESS\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_LONG_WRITE_IN_PROGRESS => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_LONG_WRITE_IN_PROGRESS\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_NO_SENSE => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_FILEMARK_DETECTED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_FILEMARK_DETECTED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_END_OF_MEDIA_DETECTED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_END_OF_MEDIA_DETECTED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_SETMARK_DETECTED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_SETMARK_DETECTED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_BEGINNING_OF_MEDIA_DETECTED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_BEGINNING_OF_MEDIA_DETECTED\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_ILLEGAL_BLOCK => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_ILLEGAL_ELEMENT_ADDR => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_ILLEGAL_ELEMENT_ADDR\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_POSITION_ERROR => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_DESTINATION_FULL => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_DESTINATION_FULL\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_SOURCE_EMPTY => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_SOURCE_EMPTY\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_INVALID_MEDIA => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_INCOMPATIBLE_MEDIA_INSTALLED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_INCOMPATIBLE_MEDIA_INSTALLED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_UNKNOWN_FORMAT => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_UNKNOWN_FORMAT\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_INCOMPATIBLE_FORMAT => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_INCOMPATIBLE_FORMAT\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_CLEANING_CARTRIDGE_INSTALLED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_CLEANING_CARTRIDGE_INSTALLED\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_OPERATOR_REQUEST => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_STATE_CHANGE_INPUT => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_STATE_CHANGE_INPUT\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_MEDIUM_REMOVAL => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_MEDIUM_REMOVAL\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_WRITE_PROTECT_ENABLE => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_WRITE_PROTECT_ENABLE\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_WRITE_PROTECT_DISABLE => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_WRITE_PROTECT_DISABLE\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                SCSI_ADSENSE_COPY_PROTECTION_FAILURE => match ad_sense_code_qual as u32 {
                    SCSI_SENSEQ_AUTHENTICATION_FAILURE => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_AUTHENTICATION_FAILURE\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_KEY_NOT_PRESENT => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_KEY_NOT_PRESENT\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_KEY_NOT_ESTABLISHED => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_KEY_NOT_ESTABLISHED\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_READ_OF_SCRAMBLED_SECTOR_WITHOUT_AUTHENTICATION => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_READ_OF_SCRAMBLED_SECTOR_WITHOUT_AUTHENTICATION\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_MEDIA_CODE_MISMATCHED_TO_LOGICAL_UNIT => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_MEDIA_CODE_MISMATCHED_TO_LOGICAL_UNIT\0".as_ptr() as PCHAR;
                    }
                    SCSI_SENSEQ_LOGICAL_UNIT_RESET_COUNT_ERROR => {
                        ad_sense_code_qual_str = b"SCSI_SENSEQ_LOGICAL_UNIT_RESET_COUNT_ERROR\0".as_ptr() as PCHAR;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    ad_sense_code_qual_str
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
}
extern "system" {
    pub fn ScsiGetSenseKeyAndCodes(
        SenseBuffer: PVOID,
        SenseBufferLength: UCHAR,
        Options: ULONG,
        SenseKey: *mut UCHAR,
        AdditionalSenseCode: *mut UCHAR,
        AdditionalSenseCodeQualifier: *mut UCHAR,
    ) -> BOOLEAN;
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn KeQueryTickCount(TickCount: *mut i64);
}
