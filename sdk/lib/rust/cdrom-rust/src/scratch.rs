use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, IoAllocateMdl, IoBuildPartialMdl, IoFreeMdl, IoGetNextIrpStackLocation, KeCancelTimer, KeInitializeDpc, KeInitializeSpinLock, KeInitializeTimer, KeSetTimer, LARGE_INTEGER, MmBuildMdlForNonPagedPool, _KDPC};
use core::ffi::c_void;

extern "system" {
    fn NT_ASSERT(condition: i32);
    fn RequestGetContext(scratch_request: *mut core::ffi::c_void) -> *mut CDROM_REQUEST_CONTEXT;
    fn KeDelayExecutionThread( WaitMode: i32, Alertable: u8, Interval: *mut LARGE_INTEGER, ) -> i32;
    fn KeQueryPerformanceCounter(PerformanceFrequency: *mut i64) -> i64;
}
unsafe extern "C" {
}
unsafe extern "C" {
    fn RequestSenseInfoInterpretForScratchBuffer( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, timesAlreadyRetried: u32, status: *mut i32, retryIn100nsUnits: *mut i64, ) -> u8;
}
unsafe extern "system" {
    fn KeQueryTickCount(CurrentCount: *mut LARGE_INTEGER);
}
extern "C" {
    fn CdromDbgPrintForce2(format: *const i8, arg1: usize, arg2: usize) -> u32;
    fn RequestSend( device_ext: *mut CDROM_DEVICE_EXTENSION, request: *mut core::ffi::c_void, iotarget: *mut core::ffi::c_void, options: u32, timeout: *mut core::ffi::c_void, ) -> i32;
    fn CdromRustScratchCompleteOrContinueReadWrite(
        DeviceExtension: PCDROM_DEVICE_EXTENSION,
        OriginalRequest: WDFREQUEST,
        Status: i32,
        MayContinue: u8,
    );
    fn CdromRustScratchAllocatePartialMdl(DeviceExtension: PCDROM_DEVICE_EXTENSION) -> i32;
}

const TRACE_LEVEL_FATAL: u32 = 1; // storswtr.h:26-30
const STATUS_INTERNAL_ERROR: i32 = 0xC00000E5u32 as i32;
const STATUS_INTEGER_OVERFLOW: i32 = 0xC0000095u32 as i32;
extern "C" {
    fn CdromDbgPrintExHistory(
        component_id: u32,
        level: u32,
        format: *const i8,
        srb_status: usize,
        sense: usize,
        asc: usize,
        ascq: usize,
        index1_signed: usize,
        index1_hex: usize,
        index2_signed: usize,
        index2_hex: usize,
    ) -> u32;
    fn CdromDbgPrint1(format: *const i8, arg1: usize) -> u32;
    fn CdromDbgPrint2(format: *const i8, arg1: usize, arg2: usize) -> u32;
    fn CdromDbgPrint3(format: *const i8, arg1: usize, arg2: usize, arg3: usize) -> u32;
    fn CdromDbgPrint4(format: *const i8, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> u32;
    fn CdromDbgPrint5(format: *const i8, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> u32;
}
extern "system" {
    fn WDF_REQUEST_REUSE_PARAMS_INIT( params: *mut WDF_REQUEST_REUSE_PARAMS, flags: u32, status: NTSTATUS, );
    fn ScratchBuffer_EndUse(DeviceExtension: PCDROM_DEVICE_EXTENSION);
    fn SET_FLAG(flags: *mut u32, flag: u32);
}
unsafe extern "C" {
    fn RequestCompletion(DeviceExtension: PCDROM_DEVICE_EXTENSION, Request: WDFREQUEST, Status: i32, TransferedBytes: ULONG_PTR);
}
// C min() is a macro; extern "system" decl = i386 stdcall/cdecl mismatch vs
// ddk_shims' extern "C" def → stack corruption. Local wrapper instead.
#[inline] fn min(a: u32, b: u32) -> u32 { core::cmp::min(a, b) }
extern "C" {
    fn RequestIsRealtimeStreaming(Request: WDFREQUEST, IsRead: u8) -> u8; // BOOLEAN
    fn MmPrepareMdlForReuse(Mdl: *mut MDL);
    fn SRB_STATUS(status: u8) -> u32;
}

const WDF_REQUEST_REUSE_NO_FLAGS: u32 = 0;
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

const DPFLTR_ERROR_LEVEL: u32 = 0;
const DPFLTR_CDROM_ID: u32 = 7; // DPFLTR component id for cdrom

const IRP_MJ_SCSI: u32 = 0x0F;
const IOCTL_SCSI_EXECUTE_IN: u32 = 0x1b0011;  // was 0x0001D004 (LLM-invented); bindgen-correct
const IOCTL_SCSI_EXECUTE_OUT: u32 = 0x1b0012;  // was 0x0001D008 (LLM-invented); bindgen-correct
const SCSI_REQUEST_BLOCK_SIZE: u32 = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32; // = C SCSI_REQUEST_BLOCK_SIZE (arch-correct: i386 0x44 / amd64 0x68); 0x50 placeholder was i386-fatal (scsiport SRB_STATUS_BAD_SRB_BLOCK_LENGTH → STATUS_INVALID_DEVICE_REQUEST)
const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00;
const SRB_SIMPLE_TAG_REQUEST: u32 = 0x20;
const SENSE_BUFFER_SIZE: u32 = 18;
const SIZEOF_CDB_READ12: usize = 12;
const SIZEOF_CDB_WRITE12: usize = 12;
const SIZEOF_CDB_CDB10: usize = 10;
const SCSIOP_READ12: u8 = 0xA8;
const SCSIOP_WRITE12: u8 = 0xAA;
const SCSIOP_READ: u8 = 0x28;
const SCSIOP_WRITE: u8 = 0x2A;
const IRP_PAGING_IO: u32 = 0x02;
const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x40;
const SRB_CLASS_FLAGS_PAGING: u32 = 0x40000000;  // was 0x00000040 (LLM-invented); bindgen-correct
const SRB_FLAGS_DATA_IN: u32 = 0x00000040;
const SRB_FLAGS_DATA_OUT: u32 = 0x00000080;
const SRB_FLAGS_ADAPTER_CACHE_ENABLE: u32 = 0x200;  // was 0x00000100 (LLM-invented); bindgen-correct
const FDO_HACK_NO_STREAMING: u32 = 0x20; // placeholder  // was 0x00000001 (LLM-invented); bindgen-correct

const WDF_REQUEST_SEND_OPTION_SYNCHRONOUS: u32 = 0x0002;
const SRB_STATUS_ABORTED: u8 = 0x02;

const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const SRB_STATUS_SUCCESS: u32 = 0x1;  // was 0 (LLM-invented); bindgen-correct
const STATUS_SUCCESS: i32 = 0;
const TRUE: i32 = 1;
const FALSE: i32 = 0;
#[no_mangle]
pub unsafe extern "C" fn ScratchBuffer_Deallocate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    // PAGED_CODE();
    NT_ASSERT(((*DeviceExtension).ScratchContext.ScratchInUse.is_null()) as i32);
    if !(*DeviceExtension).ScratchContext.ScratchHistory.is_null() {
        ExFreePool((*DeviceExtension).ScratchContext.ScratchHistory as *mut core::ffi::c_void);
        (*DeviceExtension).ScratchContext.ScratchHistory = core::ptr::null_mut();
    }
    if !(*DeviceExtension).ScratchContext.ScratchSense.is_null() {
        ExFreePool((*DeviceExtension).ScratchContext.ScratchSense as *mut core::ffi::c_void);
        (*DeviceExtension).ScratchContext.ScratchSense = core::ptr::null_mut();
    }
    if !(*DeviceExtension).ScratchContext.ScratchSrb.is_null() {
        ExFreePool((*DeviceExtension).ScratchContext.ScratchSrb as *mut core::ffi::c_void);
        (*DeviceExtension).ScratchContext.ScratchSrb = core::ptr::null_mut();
    }
    if (*DeviceExtension).ScratchContext.ScratchBufferSize != 0 {
        (*DeviceExtension).ScratchContext.ScratchBufferSize = 0;
    }
    if !(*DeviceExtension).ScratchContext.ScratchBufferMdl.is_null() {
        IoFreeMdl((*DeviceExtension).ScratchContext.ScratchBufferMdl);
        (*DeviceExtension).ScratchContext.ScratchBufferMdl = core::ptr::null_mut();
    }
    if !(*DeviceExtension).ScratchContext.ScratchBuffer.is_null() {
        ExFreePool((*DeviceExtension).ScratchContext.ScratchBuffer as *mut core::ffi::c_void);
        (*DeviceExtension).ScratchContext.ScratchBuffer = core::ptr::null_mut();
    }
    if !(*DeviceExtension).ScratchContext.PartialMdl.is_null() {
        IoFreeMdl((*DeviceExtension).ScratchContext.PartialMdl);
        (*DeviceExtension).ScratchContext.PartialMdl = core::ptr::null_mut();
    }
    if !(*DeviceExtension).ScratchContext.ScratchRequest.is_null() {
        let irp: PIRP = WdfRequestWdmGetIrp((*DeviceExtension).ScratchContext.ScratchRequest);
        if !(*irp).MdlAddress.is_null() {
            (*irp).MdlAddress = core::ptr::null_mut();
        }
        WdfObjectDelete((*DeviceExtension).ScratchContext.ScratchRequest as *mut core::ffi::c_void);
        (*DeviceExtension).ScratchContext.ScratchRequest = core::ptr::null_mut();
    }
    return;
}

#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_Allocate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> u8 {
    // Constants used by this function
    const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
    const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    // cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NonPagedPoolNx: u32 = 0;
    const CDROM_TAG_SCRATCH: u32 = 0x42436353; // 'CScr'

    // Forward declarations for unknown symbols
    unsafe extern "system" {
        fn KeGetCurrentIrql() -> KIRQL;
        fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
            attributes: *mut WDF_OBJECT_ATTRIBUTES,
            context_type: *const core::ffi::c_void,
        ) -> ();
    }
    unsafe extern "C" {
        fn RtlULongAdd(ul1: u32, ul2: u32, result: *mut u32) -> NTSTATUS;
    }

    // WPP tracing macro (compiles to a no-op); accepts the variadic call shapes
    // used below without requiring an unstable C-variadic "system" fn.
    macro_rules! TracePrint {
        ($($arg:expr),* $(,)?) => {{ $( let _ = &$arg; )* }};
    }

    // validate max transfer already determined
    ntkrnl_sys::ASSERT!((*DeviceExtension)
            .DeviceAdditionalData
            .MaxPageAlignedTransferBytes
            != 0);

    // validate no partially-saved state
    ntkrnl_sys::ASSERT!((*DeviceExtension).ScratchContext.ScratchBuffer.is_null());
    ntkrnl_sys::ASSERT!((*DeviceExtension).ScratchContext.ScratchBufferMdl.is_null());
    ntkrnl_sys::ASSERT!((*DeviceExtension).ScratchContext.ScratchBufferSize == 0);
    ntkrnl_sys::ASSERT!((*DeviceExtension).ScratchContext.ScratchRequest.is_null());
    ntkrnl_sys::ASSERT!((*DeviceExtension).ScratchContext.PartialMdl.is_null());

    // limit the scratch buffer to between 4k and 64k (so data length fits into USHORT -- req'd for many commands)
    let max_transfer = (*DeviceExtension)
        .DeviceAdditionalData
        .MaxPageAlignedTransferBytes;
    let scratch_size: u32 = if max_transfer < (64 * 1024) {
        max_transfer
    } else {
        64 * 1024
    };
    (*DeviceExtension).ScratchContext.ScratchBufferSize = scratch_size;

    let mut status: NTSTATUS = STATUS_SUCCESS;
    CdromDbgPrint2(b"[SCRA] enter maxpat=%x scratch_size=%x\n\0".as_ptr() as *const i8,
        (*DeviceExtension).DeviceAdditionalData.MaxPageAlignedTransferBytes as usize,
        scratch_size as usize);

    // allocate the buffer
    if NT_SUCCESS(status) {
        (*DeviceExtension).ScratchContext.ScratchBuffer =
            ExAllocatePoolWithTag(
                NonPagedPoolNx,
                scratch_size as usize,
                CDROM_TAG_SCRATCH,
            ) as *mut core::ffi::c_void;
        if (*DeviceExtension).ScratchContext.ScratchBuffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate scratch buffer of %x bytes\n\0".as_ptr() as *const i8,
                scratch_size,
            );
        } else if ((*DeviceExtension).ScratchContext.ScratchBuffer as usize)
            & (PAGE_SIZE as usize - 1)
            != 0
        {
            status = STATUS_INTERNAL_ERROR;
            TracePrint!(
                TRACE_LEVEL_FATAL,
                TRACE_FLAG_INIT,
                b"Allocation of %x bytes non-paged pool was not allocated on page boundary?  STATUS_INTERNAL_ERROR\n\0"
                    .as_ptr() as *const i8,
                scratch_size,
            );
        }
    }

    CdromDbgPrint2(b"[SCRA] after buf: st=%x buf=%p\n\0".as_ptr() as *const i8,
        status as usize, (*DeviceExtension).ScratchContext.ScratchBuffer as usize);

    // allocate the MDL
    if NT_SUCCESS(status) {
        (*DeviceExtension).ScratchContext.ScratchBufferMdl = IoAllocateMdl(
            (*DeviceExtension).ScratchContext.ScratchBuffer,
            scratch_size,
            0,
            0,
            core::ptr::null_mut(),
        );
        if (*DeviceExtension).ScratchContext.ScratchBufferMdl.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate MDL for %x byte buffer\n\0".as_ptr() as *const i8,
                scratch_size,
            );
        } else {
            MmBuildMdlForNonPagedPool((*DeviceExtension).ScratchContext.ScratchBufferMdl);
        }
    }

    CdromDbgPrint2(b"[SCRA] after mdl: st=%x mdl=%p\n\0".as_ptr() as *const i8,
        status as usize, (*DeviceExtension).ScratchContext.ScratchBufferMdl as usize);

    // create the request
    if NT_SUCCESS(status) {
        let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
        WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
            &mut attributes,
            core::ptr::addr_of!(crate::ddk_shims::CDROM_REQUEST_CONTEXT) as *const core::ffi::c_void,
        ); // C passes CDROM_REQUEST_CONTEXT type-info; null here → NULL deref of UniqueType (0x1E)
        status = WdfRequestCreate(
            &mut attributes,
            (*DeviceExtension).IoTarget,
            &mut (*DeviceExtension).ScratchContext.ScratchRequest,
        );
        if !NT_SUCCESS(status)
            || (*DeviceExtension).ScratchContext.ScratchRequest.is_null()
        {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate scratch MDL \n\0".as_ptr() as *const i8,
            );
        }
    }

    CdromDbgPrint2(b"[SCRA] after reqcreate: st=%x req=%p\n\0".as_ptr() as *const i8,
        status as usize, (*DeviceExtension).ScratchContext.ScratchRequest as usize);

    // allocate the srb
    if NT_SUCCESS(status) {
        (*DeviceExtension).ScratchContext.ScratchSrb =
            ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                CDROM_TAG_SCRATCH,
            ) as *mut SCSI_REQUEST_BLOCK;
        if (*DeviceExtension).ScratchContext.ScratchSrb.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate scratch SRB\n\0".as_ptr() as *const i8,
            );
        }
    }

    // allocate the sense buffer
    if NT_SUCCESS(status) {
        (*DeviceExtension).ScratchContext.ScratchSense =
            ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<SENSE_DATA>(),
                CDROM_TAG_SCRATCH,
            ) as *mut SENSE_DATA;
        if (*DeviceExtension).ScratchContext.ScratchSense.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate scratch sense data\n\0".as_ptr() as *const i8,
            );
        }
    }

    CdromDbgPrint3(b"[SCRA] after srb+sense: st=%x srb=%p sense=%p\n\0".as_ptr() as *const i8,
        status as usize,
        (*DeviceExtension).ScratchContext.ScratchSrb as usize,
        (*DeviceExtension).ScratchContext.ScratchSense as usize);

    // allocate the SRB history data
    if NT_SUCCESS(status) {
        let allocation_size: usize = core::mem::size_of::<SRB_HISTORY>()
            - core::mem::size_of::<SRB_HISTORY_ITEM>()
            + 20 * core::mem::size_of::<SRB_HISTORY_ITEM>();
        (*DeviceExtension).ScratchContext.ScratchHistory =
            ExAllocatePoolWithTag(
                NonPagedPoolNx,
                allocation_size,
                CDROM_TAG_SCRATCH,
            ) as *mut SRB_HISTORY;
        if (*DeviceExtension).ScratchContext.ScratchHistory.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate scratch history buffer\n\0".as_ptr() as *const i8,
            );
        } else {
            // must be initialized here...
            core::ptr::write_bytes(
                ((*DeviceExtension).ScratchContext.ScratchHistory) as *mut u8,
                0u8,
                allocation_size);
            (*(*DeviceExtension).ScratchContext.ScratchHistory).TotalHistoryCount = 20;
        }
    }

    // allocate the MDL (PartialMdl)
    if NT_SUCCESS(status) {
        status = CdromRustScratchAllocatePartialMdl(DeviceExtension);
        if !NT_SUCCESS(status) {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to allocate MDL for %x byte buffer\n\0".as_ptr()
                    as *const i8,
                scratch_size,
            );
        }
    }

    CdromDbgPrint2(b"[SCRA] after history+partialmdl: st=%x partmdl=%p\n\0".as_ptr() as *const i8,
        status as usize, (*DeviceExtension).ScratchContext.PartialMdl as usize);

    // cleanup on failure
    if !NT_SUCCESS(status) {
        ScratchBuffer_Deallocate(DeviceExtension);
    }

    (NT_SUCCESS(status)) as u8
}
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ResetItems(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    ResetRequestHistory: u8,
) {
    let mut status: NTSTATUS;
    let mut reuseParams: WDF_REQUEST_REUSE_PARAMS = core::mem::zeroed();
    let irp: PIRP;


    // NT_ASSERT checks
    let scratch = &(*DeviceExtension).ScratchContext;
    NT_ASSERT(if scratch.ScratchHistory.is_null() { 0 } else { 1 });
    NT_ASSERT(if scratch.ScratchSense.is_null() { 0 } else { 1 });
    NT_ASSERT(if scratch.ScratchSrb.is_null() { 0 } else { 1 });
    NT_ASSERT(if scratch.ScratchRequest.is_null() { 0 } else { 1 });
    NT_ASSERT(if scratch.ScratchBufferSize != 0 { 1 } else { 0 });
    NT_ASSERT(if scratch.ScratchBuffer.is_null() { 0 } else { 1 });
    NT_ASSERT(if scratch.ScratchBufferMdl.is_null() { 0 } else { 1 });
    NT_ASSERT(if !scratch.ScratchInUse.is_null() { 1 } else { 0 });

    irp = WdfRequestWdmGetIrp(scratch.ScratchRequest);

    if ResetRequestHistory != 0 {
        let history: *mut SRB_HISTORY = scratch.ScratchHistory as *mut _;
        ntkrnl_sys::rtl::RtlZeroMemory(
            (*history).History.as_mut_ptr() as *mut _,
            core::mem::size_of::<SRB_HISTORY_ITEM>() * (*history).TotalHistoryCount as usize,
        );
        *((*history).ClassDriverUse.as_mut_ptr().offset(0)) = 0;
        *((*history).ClassDriverUse.as_mut_ptr().offset(1)) = 0;
        *((*history).ClassDriverUse.as_mut_ptr().offset(2)) = 0;
        *((*history).ClassDriverUse.as_mut_ptr().offset(3)) = 0;
        (*history).UsedHistoryCount = 0;
    }

    // deassign MdlAddress
    if !irp.is_null() && !(*irp).MdlAddress.is_null() {
        (*irp).MdlAddress = core::ptr::null_mut();
    }

    WDF_REQUEST_REUSE_PARAMS_INIT(
        &mut reuseParams as *mut _,
        WDF_REQUEST_REUSE_NO_FLAGS,
        ntkrnl_sys::constants::STATUS_NOT_SUPPORTED,
    );
    status = WdfRequestReuse(scratch.ScratchRequest, &mut reuseParams);
    if status >= 0 {
        WdfRequestSetCompletionRoutine(
            scratch.ScratchRequest,
            None,
            core::ptr::null_mut(),
        );
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*DeviceExtension).IoTarget,
            scratch.ScratchRequest,
            IOCTL_SCSI_EXECUTE_IN,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if status < 0 {
            let fmt = b"ScratchBuffer_ResetItems: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n\0"
                .as_ptr() as *const u8;
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, fmt, status);
        }
    }

    ntkrnl_sys::rtl::RtlZeroMemory(
        scratch.ScratchSense as *mut _,
        core::mem::size_of::<SENSE_DATA>(),
    );
    ntkrnl_sys::rtl::RtlZeroMemory(
        scratch.ScratchSrb as *mut _,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_PerformNextReadWrite(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FirstTry: i32,
) -> i32 {
    unsafe {
        let readWriteContext =
            core::ptr::addr_of_mut!((*DeviceExtension).ScratchContext.ScratchReadWriteContext);
        let requestContext = RequestGetContext((*DeviceExtension).ScratchContext.ScratchRequest as *mut core::ffi::c_void);
        let originalRequest = (*requestContext).OriginalRequest;
        let mut status: i32; // STATUS_SUCCESS
        CdromDbgPrint3(b"[PNRW] scratchreq=%p reqctx=%p origreq=%p\n\0".as_ptr() as *const i8,
            (*DeviceExtension).ScratchContext.ScratchRequest as usize,
            requestContext as usize, originalRequest as usize);

        let transferSize = min(
            (*readWriteContext).EntireXferLen.wrapping_sub((*readWriteContext).TransferedBytes),
            (*readWriteContext).MaxLength,
        );

        if FirstTry != 0 {
            (*DeviceExtension).ScratchContext.NumRetries = 0;
            CdromDbgPrint4(
                b"[READ] xfer=%lx done=%lx max=%lx pkts=%lx\n\0".as_ptr() as *const i8,
                (*readWriteContext).EntireXferLen as usize,
                (*readWriteContext).TransferedBytes as usize,
                (*readWriteContext).MaxLength as usize,
                (*readWriteContext).PacketsCount as usize,
            );
        }

        ScratchBuffer_ResetItems(DeviceExtension, 0); // FALSE

        let usePartialMdl = if ((*readWriteContext).PacketsCount > 1)
            || ((*readWriteContext).TransferedBytes > 0)
        {
            1
        } else {
            0
        };

        ScratchBuffer_SetupReadWriteSrb(
            DeviceExtension,
            originalRequest,
            (*readWriteContext).StartingOffset,
            transferSize,
            (*readWriteContext).DataBuffer,
            (*readWriteContext).IsRead as u8,
            usePartialMdl,
        );

        WdfRequestSetCompletionRoutine(
            (*DeviceExtension).ScratchContext.ScratchRequest,
            Some(core::mem::transmute(ScratchBuffer_ReadWriteCompletionRoutine as unsafe extern "system" fn(_, _, _, _))),
            DeviceExtension as *mut core::ffi::c_void,
        );

        let srbHistoryItemPtr = if FirstTry != 0 {
            core::ptr::addr_of_mut!((*readWriteContext).SrbHistoryItem)
        } else {
            core::ptr::null_mut()
        };

        SEND_QPC = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(QPC_FREQ));
        status = ScratchBuffer_SendSrb(DeviceExtension, 0, srbHistoryItemPtr); // FALSE

        status
    }
}

static mut SEND_QPC: i64 = 0;
static mut QPC_FREQ: i64 = 0;

#[inline(never)]
unsafe fn ScratchBuffer_RequestIsCanceled(request: WDFREQUEST) -> u8 {
    WdfRequestIsCanceled(request)
}

#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ReadWriteTimerRoutine(
    Dpc: *mut _KDPC,
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {

    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION;
    let mut readWriteContext: *mut CDROM_SCRATCH_READ_WRITE_CONTEXT;
    let originalRequest: *mut core::ffi::c_void;
    let requestContext: *mut CDROM_REQUEST_CONTEXT;
    let mut status: NTSTATUS;
    let mut oldIrql: KIRQL = 0;

    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    if DeferredContext.is_null() {
        return;
    }

    originalRequest = DeferredContext;
    requestContext = RequestGetContext(originalRequest);

    KeAcquireSpinLock(&raw mut (*requestContext).ReadWriteCancelSpinLock as *mut usize, &raw mut oldIrql);

    if (*requestContext).ReadWriteIsCompleted == 0 {
        status = WdfRequestUnmarkCancelable(originalRequest as WDFREQUEST);
    } else {
        status = STATUS_CANCELLED;
    }

    KeReleaseSpinLock(&raw mut (*requestContext).ReadWriteCancelSpinLock as *mut usize, oldIrql);

    if status != STATUS_CANCELLED {
        deviceExtension = (*requestContext).DeviceExtension;
        readWriteContext = &raw mut (*deviceExtension).ScratchContext.ScratchReadWriteContext;

        status = ScratchBuffer_PerformNextReadWrite(deviceExtension, 0i32);

        if !NT_SUCCESS(status) {
            ScratchBuffer_EndUse(deviceExtension);
            RequestCompletion(
                deviceExtension,
                originalRequest as WDFREQUEST,
                status,
                ((*readWriteContext).TransferedBytes) as usize,
            );
        }
    }

    WdfObjectDereference(originalRequest);
}
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ReadWriteEvtRequestCancel(Request: WDFREQUEST) {
    let requestContext: PCDROM_REQUEST_CONTEXT = RequestGetContext(Request as *mut core::ffi::c_void);
    let deviceExtension: PCDROM_DEVICE_EXTENSION = (*requestContext).DeviceExtension;
    let readWriteContext: PCDROM_SCRATCH_READ_WRITE_CONTEXT = core::ptr::addr_of_mut!((*deviceExtension).ScratchContext.ScratchReadWriteContext);
    let mut oldIrql: ntkrnl_sys::base::KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*requestContext).ReadWriteCancelSpinLock) as *mut usize, core::ptr::addr_of_mut!(oldIrql));

    if KeCancelTimer(core::ptr::addr_of_mut!((*requestContext).ReadWriteTimer)) != 0 {
        WdfObjectDereference(Request as *mut core::ffi::c_void);
    } else {
        // Timer will run and drop the reference but it won't complete the request
        // because we set IsCompleted to TRUE
    }

    (*requestContext).ReadWriteIsCompleted = 1; // TRUE

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*requestContext).ReadWriteCancelSpinLock) as *mut usize, oldIrql);

    ScratchBuffer_EndUse(deviceExtension);

    // If WdfTimerStop returned TRUE, it means this request was scheduled for a retry
    // and the retry has not happened yet. We just need to cancel it and release the scratch buffer.
    RequestCompletion(deviceExtension, Request, STATUS_CANCELLED, ((*readWriteContext).TransferedBytes) as usize);
}
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ReadWriteCompletionRoutine(
    Request: WDFREQUEST,
    Target: WDFIOTARGET,
    Params: *mut WDF_REQUEST_COMPLETION_PARAMS,
    Context: WDFCONTEXT,
) {
    let device_extension: *mut CDROM_DEVICE_EXTENSION = Context as *mut CDROM_DEVICE_EXTENSION;
    let read_write_context: *mut CDROM_SCRATCH_READ_WRITE_CONTEXT =
        core::ptr::addr_of_mut!((*device_extension).ScratchContext.ScratchReadWriteContext);
    let mut status: i32 = STATUS_SUCCESS;
    let request_context: *mut CDROM_REQUEST_CONTEXT =
        RequestGetContext((*device_extension).ScratchContext.ScratchRequest as *mut core::ffi::c_void);
    let original_request: WDFREQUEST = (*request_context).OriginalRequest;

    {
        let now = KeQueryPerformanceCounter(core::ptr::null_mut());
        let us = if QPC_FREQ != 0 { (now - SEND_QPC).wrapping_mul(1_000_000) / QPC_FREQ } else { 0 };
        if us > 100_000 {
            CdromDbgPrint2(
                b"[IOLAT] send->compl us=%lx srbst=%lx\n\0".as_ptr() as *const i8,
                us as usize,
                (*(*device_extension).ScratchContext.ScratchSrb).SrbStatus as usize,
            );
        }
    }

    if !NT_SUCCESS(WdfRequestGetStatus(Request)) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"WdfRequestSend: %lx\n\0".as_ptr() as *const u8,
            WdfRequestGetStatus(Request),
        );
    }

    // UNREFERENCED_PARAMETER
    let _ = Params;
    let _ = Target;

    if (*(*device_extension).ScratchContext.ScratchSrb).SrbStatus == SRB_STATUS_ABORTED
        && *(*(*device_extension).ScratchContext.ScratchSrb).__bindgen_anon_1.InternalStatus.as_ref() == STATUS_CANCELLED as u32
    {
        // The request has been cancelled, just need to complete it
    } else if SRB_STATUS((*(*device_extension).ScratchContext.ScratchSrb).SrbStatus) != SRB_STATUS_SUCCESS {
        // The SCSI command that we sent down has failed, retry it if necessary
        let mut should_retry: i32;
        let mut retry_in_100ns_units: i64 = 0;

        should_retry = RequestSenseInfoInterpretForScratchBuffer(
            device_extension,
            (*device_extension).ScratchContext.NumRetries,
            &mut status,
            &mut retry_in_100ns_units,
        ) as i32;

        {
            let srb = (*device_extension).ScratchContext.ScratchSrb;
            let sense = (*srb).SenseInfoBuffer as *const u8;
            let (sk, asc, ascq) = if !sense.is_null() {
                (*sense.add(2) & 0x0f, *sense.add(12), *sense.add(13))
            } else { (0xff, 0xff, 0xff) };
            CdromDbgPrint5(
                b"[RETRY] srbst=%lx retry=%lx delay100ns=%lx numretry=%lx sense=%lx\n\0".as_ptr() as *const i8,
                (*srb).SrbStatus as usize,
                should_retry as usize,
                retry_in_100ns_units as usize,
                (*device_extension).ScratchContext.NumRetries as usize,
                ((sk as usize) << 16) | ((asc as usize) << 8) | (ascq as usize),
            );
        }

        if should_retry != 0 {
            (*device_extension).ScratchContext.NumRetries += 1;

            if retry_in_100ns_units == 0 {
                // We take a shortcut here by calling ScratchBuffer_PerformNextReadWrite directly:
                // this helps to avoid unnecessary context switch.
                status = ScratchBuffer_PerformNextReadWrite(device_extension, FALSE);

                if NT_SUCCESS(status) {
                    // We're not done with the request yet, no need to complete it now
                    return;
                }
            } else {
                let original_request_context: *mut CDROM_REQUEST_CONTEXT =
                    RequestGetContext(original_request as *mut core::ffi::c_void);
                let mut old_irql: KIRQL = 0;

                //
                // Initialize the spin lock and timer local to the original request.
                //
                if (*original_request_context).ReadWriteRetryInitialized == 0 {
                    KeInitializeSpinLock(core::ptr::addr_of_mut!(
                        (*original_request_context).ReadWriteCancelSpinLock
                    ) as ntkrnl_sys::ddk::PKSPIN_LOCK);
                    KeInitializeTimer(core::ptr::addr_of_mut!(
                        (*original_request_context).ReadWriteTimer
                    ));
                    KeInitializeDpc(
                        core::ptr::addr_of_mut!((*original_request_context).ReadWriteDpc),
                        Some(core::mem::transmute(ScratchBuffer_ReadWriteTimerRoutine as unsafe extern "system" fn(_, _, _, _))),
                        original_request as *mut core::ffi::c_void,
                    );
                    (*original_request_context).ReadWriteRetryInitialized = TRUE as u8;
                }

                KeAcquireSpinLock(
                    core::ptr::addr_of_mut!((*request_context).ReadWriteCancelSpinLock) as *mut usize,
                    &mut old_irql,
                );

                // Mark cancelable before setting the timer so we can't race with
                // cancellation callbacks.
                status = WdfRequestMarkCancelableEx(
                    original_request,
                    Some(core::mem::transmute(ScratchBuffer_ReadWriteEvtRequestCancel as unsafe extern "system" fn(_))),
                );

                if status == STATUS_CANCELLED {
                    (*request_context).ReadWriteIsCompleted = TRUE as u8;

                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*request_context).ReadWriteCancelSpinLock) as *mut usize,
                        old_irql,
                    );
                } else {
                    let mut t: LARGE_INTEGER = core::mem::zeroed();
                    t.QuadPart = -retry_in_100ns_units;

                    WdfObjectReference(original_request as *mut core::ffi::c_void);

                    // Use negative time to indicate that we want a relative delay
                    KeSetTimer(
                        core::ptr::addr_of_mut!((*original_request_context).ReadWriteTimer),
                        t,
                        core::ptr::addr_of_mut!((*original_request_context).ReadWriteDpc),
                    );

                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*request_context).ReadWriteCancelSpinLock) as *mut usize,
                        old_irql,
                    );

                    return;
                }
            }
        }
    } else {
        // The SCSI command has succeeded
        let data_transfer_length =
            (*(*device_extension).ScratchContext.ScratchSrb).DataTransferLength;
        (*read_write_context).DataBuffer =
            (*read_write_context).DataBuffer.add(data_transfer_length as usize);
        (*read_write_context).StartingOffset.QuadPart += data_transfer_length as i64;
        (*read_write_context).TransferedBytes += data_transfer_length;
        let packets_count = core::ptr::read_volatile(core::ptr::addr_of!((*read_write_context).PacketsCount));
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*read_write_context).PacketsCount),
            packets_count.wrapping_sub(1),
        );

        // Update the SRB history item
        if !(*read_write_context).SrbHistoryItem.is_null() {
            let mut sense_size: u32;

            // Query the tick count and store in the history
            KeQueryTickCount(core::ptr::addr_of_mut!(
                (*(*read_write_context).SrbHistoryItem).TickCountCompleted
            ));

            // Copy the SRB Status...
            (*(*read_write_context).SrbHistoryItem).SrbStatus =
                (*(*device_extension).ScratchContext.ScratchSrb).SrbStatus;

            // Determine the amount of valid sense data
            let sense_info_buffer_length =
                (*(*device_extension).ScratchContext.ScratchSrb).SenseInfoBufferLength as u32;

            if sense_info_buffer_length
                >= (core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength)
                    + core::mem::size_of::<u8>()) as u32
            {
                let sense: *const SENSE_DATA =
                    (*(*device_extension).ScratchContext.ScratchSrb).SenseInfoBuffer
                        as *const SENSE_DATA;
                sense_size = (core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength)
                    + core::mem::size_of::<u8>()
                    + (*sense).AdditionalSenseLength as usize) as u32;
                sense_size = min(sense_size, core::mem::size_of::<SENSE_DATA>() as u32);
            } else {
                sense_size = sense_info_buffer_length;
            }

            // Normalize the sense data copy in the history
            core::ptr::write_bytes(
                (core::ptr::addr_of_mut!(
                    (*(*read_write_context).SrbHistoryItem).NormalizedSenseData
                )) as *mut u8,
                0,
                core::mem::size_of::<SENSE_DATA>());
            core::ptr::copy_nonoverlapping(
                (*(*device_extension).ScratchContext.ScratchSrb).SenseInfoBuffer as *const u8,
                core::ptr::addr_of_mut!(
                    (*(*read_write_context).SrbHistoryItem).NormalizedSenseData
                ) as *mut u8,
                sense_size as usize,
            );
        }

        // Check whether we need to send more SCSI commands to complete the request
        CdromRustScratchCompleteOrContinueReadWrite(
            device_extension,
            original_request,
            status,
            TRUE as u8,
        );
        return;
    }

    CdromRustScratchCompleteOrContinueReadWrite(
        device_extension,
        original_request,
        status,
        FALSE as u8,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ScratchBuffer_SetupSrb(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    MaximumTransferLength: u32,
    GetDataFromDevice: u8,
) {
    let mut request: WDFREQUEST;
    let mut irp: PIRP;
    let mut srb: PSCSI_REQUEST_BLOCK;
    let mut irpStack: PIO_STACK_LOCATION;
    let mut requestContext: PCDROM_REQUEST_CONTEXT;

    request = (*DeviceExtension).ScratchContext.ScratchRequest;
    irp = WdfRequestWdmGetIrp(request);
    srb = (*DeviceExtension).ScratchContext.ScratchSrb;
    irpStack = IoGetNextIrpStackLocation(irp);
    requestContext = RequestGetContext(request as *mut core::ffi::c_void);

    PAGED_CODE!();

    core::ptr::write_volatile(core::ptr::addr_of_mut!((*requestContext).OriginalRequest), OriginalRequest);

    // set to use the full scratch buffer via the scratch SRB
    irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;
    if MaximumTransferLength == 0 {
        (*irpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_NONE;
    } else if GetDataFromDevice != 0 {
        (*irpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN;
    } else {
        (*irpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_OUT;
    }
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*irpStack).Parameters)
            as *mut *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK,
        srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK,
    );

    if MaximumTransferLength > 0 {
        // the Irp must show the MDL's address for the transfer
        (*irp).MdlAddress = (*DeviceExtension).ScratchContext.ScratchBufferMdl;
        (*srb).DataBuffer = (*DeviceExtension).ScratchContext.ScratchBuffer;
    }

    // prepare the SRB with default values
    (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
    (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
    (*srb).SrbStatus = 0;
    (*srb).ScsiStatus = 0;
    (*srb).NextSrb = core::ptr::null_mut();
    (*srb).OriginalRequest = irp as *mut core::ffi::c_void;
    (*srb).SenseInfoBufferLength = SENSE_BUFFER_SIZE as u8;
    (*srb).SenseInfoBuffer = (*DeviceExtension).ScratchContext.ScratchSense as *mut core::ffi::c_void;
    (*srb).CdbLength = 16; // to cause failures if not set correctly -- CD devices limited to 12 bytes for now...
    (*srb).DataTransferLength = min((*DeviceExtension).ScratchContext.ScratchBufferSize, MaximumTransferLength);
    (*srb).TimeOutValue = (*DeviceExtension).TimeOutValue;
    (*srb).SrbFlags = (*DeviceExtension).SrbFlags;
    SET_FLAG(core::ptr::addr_of_mut!((*srb).SrbFlags), SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
    SET_FLAG(core::ptr::addr_of_mut!((*srb).SrbFlags), SRB_FLAGS_NO_QUEUE_FREEZE);

    if MaximumTransferLength == 0 {
        SET_FLAG(core::ptr::addr_of_mut!((*srb).SrbFlags), SRB_FLAGS_NO_DATA_TRANSFER);
    } else if GetDataFromDevice != 0 {
        SET_FLAG(core::ptr::addr_of_mut!((*srb).SrbFlags), SRB_FLAGS_DATA_IN);
    } else {
        SET_FLAG(core::ptr::addr_of_mut!((*srb).SrbFlags), SRB_FLAGS_DATA_OUT);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScratchBuffer_SendSrb(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    SynchronousSrb: u8,
    SrbHistoryItem: *mut *mut SRB_HISTORY_ITEM,
) -> i32 {
    let mut status: i32;
    let srb = (*DeviceExtension).ScratchContext.ScratchSrb;
    let history = (*DeviceExtension).ScratchContext.ScratchHistory;
    let mut item: *mut SRB_HISTORY_ITEM = core::ptr::null_mut();
    let mut request_cancelled: u8 = 0;

    *(*srb).__bindgen_anon_1.InternalStatus.as_mut() = 0;
    (*srb).SrbStatus = 0;

    if SynchronousSrb != 0 || !SrbHistoryItem.is_null() {
        NT_ASSERT(((*history).UsedHistoryCount <= (*history).TotalHistoryCount) as i32);

        if (*history).UsedHistoryCount == (*history).TotalHistoryCount {
            CompressSrbHistoryData(history);
            NT_ASSERT(((*history).UsedHistoryCount < (*history).TotalHistoryCount) as i32);
        }

        NT_ASSERT(((*history).UsedHistoryCount < (*history).TotalHistoryCount) as i32);

        (*history).UsedHistoryCount += 1;

        let history_array_ptr: *mut SRB_HISTORY_ITEM =
            core::ptr::addr_of!((*history).History) as *mut SRB_HISTORY_ITEM;
        item = history_array_ptr.add((*history).UsedHistoryCount as usize - 1);

        if !SrbHistoryItem.is_null() {
            *SrbHistoryItem = item;
        }

        RtlZeroMemory(
            item as *mut core::ffi::c_void,
            core::mem::size_of::<SRB_HISTORY_ITEM>(),
        );

        let tick_ptr: *mut LARGE_INTEGER = &raw mut (*item).TickCountSent;
        KeQueryTickCount(tick_ptr);
    }

    let request_context =
        RequestGetContext((*DeviceExtension).ScratchContext.ScratchRequest as *mut core::ffi::c_void);
    let original_request = core::ptr::read_volatile(core::ptr::addr_of!((*request_context).OriginalRequest));
    if !original_request.is_null() {
        request_cancelled = ScratchBuffer_RequestIsCanceled(original_request as WDFREQUEST);
    }

    if request_cancelled == 0 {
        let send_options = if SynchronousSrb != 0 {
            WDF_REQUEST_SEND_OPTION_SYNCHRONOUS
        } else {
            0
        };
        status = RequestSend(
            DeviceExtension,
            (*DeviceExtension).ScratchContext.ScratchRequest as *mut core::ffi::c_void,
            (*DeviceExtension).IoTarget as *mut core::ffi::c_void,
            send_options,
            core::ptr::null_mut(),
        );
        CdromDbgPrint5(
            b"[SCR] send st=%x srbst=%x dtl=%x sync=%x req=%p\n\0".as_ptr() as *const i8,
            status as usize,
            (*srb).SrbStatus as usize,
            (*srb).DataTransferLength as usize,
            SynchronousSrb as usize,
            (*DeviceExtension).ScratchContext.ScratchRequest as usize,
        );

        if SynchronousSrb != 0 {
            let mut sense_size: usize;

            let tick_ptr2: *mut LARGE_INTEGER = &raw mut (*item).TickCountCompleted;
            KeQueryTickCount(tick_ptr2);

            (*item).SrbStatus = (*srb).SrbStatus;

            let sense_size_offset = core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength) + 1;
            if (*srb).SenseInfoBufferLength as usize >= sense_size_offset {
                let sense = (*srb).SenseInfoBuffer as *const SENSE_DATA;
                let additional_sense_length = (*sense).AdditionalSenseLength as usize;
                sense_size = sense_size_offset + additional_sense_length;
                if sense_size > core::mem::size_of::<SENSE_DATA>() {
                    sense_size = core::mem::size_of::<SENSE_DATA>();
                }
            } else {
                sense_size = (*srb).SenseInfoBufferLength as usize;
            }

            RtlZeroMemory(
                &raw mut (*item).NormalizedSenseData as *mut core::ffi::c_void,
                core::mem::size_of::<SENSE_DATA>(),
            );
            RtlCopyMemory(
                &raw mut (*item).NormalizedSenseData as *mut core::ffi::c_void,
                (*srb).SenseInfoBuffer,
                sense_size,
            );
        }
    } else {
        (*(*DeviceExtension).ScratchContext.ScratchSrb).SrbStatus = SRB_STATUS_ABORTED;
        *(*(*DeviceExtension).ScratchContext.ScratchSrb).__bindgen_anon_1.InternalStatus.as_mut() = STATUS_CANCELLED as u32;
        status = STATUS_CANCELLED;
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn CompressSrbHistoryData(RequestHistory: *mut SRB_HISTORY) {
    let mut i: u32;
    NT_ASSERT(((*RequestHistory).UsedHistoryCount == (*RequestHistory).TotalHistoryCount) as i32);
    ValidateSrbHistoryDataPresumptions(RequestHistory as *const SRB_HISTORY);

    i = 0;
    while i < (*RequestHistory).UsedHistoryCount {
        // History is a FLEXIBLE array -- classpnp.h declares it
        // `_Field_size_part_(TotalHistoryCount, UsedHistoryCount) History[1]` and the
        // allocator adds 20 * sizeof(SRB_HISTORY_ITEM), so bindgen's `[T; 1]` makes
        // every index >= 1 a panic. Walk it by pointer.
        let toMatch: *mut SRB_HISTORY_ITEM = (&raw mut (*RequestHistory).History).cast::<SRB_HISTORY_ITEM>().add(i as usize);
        let srbStatus: u8 = (*toMatch).SrbStatus;
        let sense: u8 = (*toMatch).NormalizedSenseData.SenseKey();
        let asc: u8 = (*toMatch).NormalizedSenseData.AdditionalSenseCode;
        let ascq: u8 = (*toMatch).NormalizedSenseData.AdditionalSenseCodeQualifier;
        let mut j: u32;

        j = i + 1;
        while ((*toMatch).ClassDriverUse != 0xFF) && j < (*RequestHistory).UsedHistoryCount {
            let found: *mut SRB_HISTORY_ITEM = (&raw mut (*RequestHistory).History).cast::<SRB_HISTORY_ITEM>().add(j as usize);
            if (srbStatus == (*found).SrbStatus) &&
                (sense == (*found).NormalizedSenseData.SenseKey()) &&
                (asc == (*found).NormalizedSenseData.AdditionalSenseCode) &&
                (ascq == (*found).NormalizedSenseData.AdditionalSenseCodeQualifier)
            {
                if ((*toMatch).MillisecondsDelayOnRetry).wrapping_add((*found).MillisecondsDelayOnRetry) < (*toMatch).MillisecondsDelayOnRetry {
                    (*toMatch).MillisecondsDelayOnRetry = MAXULONG;
                } else {
                    (*toMatch).MillisecondsDelayOnRetry = ((*toMatch).MillisecondsDelayOnRetry).wrapping_add((*found).MillisecondsDelayOnRetry);
                }
                NT_ASSERT(((*found).ClassDriverUse == 0) as i32);
                (*toMatch).ClassDriverUse += 1;

                if j != (*RequestHistory).UsedHistoryCount - 1 {
                    let mut remainingBytes: usize;
                    remainingBytes = ((*RequestHistory).UsedHistoryCount as usize).wrapping_sub(1).wrapping_sub(j as usize);
                    remainingBytes = remainingBytes.wrapping_mul(core::mem::size_of::<SRB_HISTORY_ITEM>());
                    // C: RtlMoveMemory(found, found+1, remainingBytes) — a BYTE
                    // count. core::ptr::copy::<SRB_HISTORY_ITEM> takes an ELEMENT
                    // count, so copy as bytes to match (else a size_of× over-copy
                    // → heap corruption). Found by `rustconv semdiff`.
                    core::ptr::copy(found.add(1) as *const u8, found as *mut u8, remainingBytes);
                    (*RequestHistory).UsedHistoryCount -= 1;
                    j -= 1;
                }
            }
            j += 1;
        }
        i += 1;
    }

    if (*RequestHistory).UsedHistoryCount == (*RequestHistory).TotalHistoryCount {
        let item: *mut SRB_HISTORY_ITEM =
            (&raw mut (*RequestHistory).History).cast::<SRB_HISTORY_ITEM>().add((*RequestHistory).TotalHistoryCount as usize - 1);
        (*RequestHistory).ClassDriverUse[0] = ((*RequestHistory).ClassDriverUse[0]).wrapping_add((*item).ClassDriverUse as _);
        (*RequestHistory).UsedHistoryCount -= 1;
    }

    NT_ASSERT(((*RequestHistory).UsedHistoryCount != (*RequestHistory).TotalHistoryCount) as i32);
    {
        let mut bytesToZero: usize;
        bytesToZero = ((*RequestHistory).TotalHistoryCount as usize).wrapping_sub((*RequestHistory).UsedHistoryCount as usize);
        bytesToZero = bytesToZero.wrapping_mul(core::mem::size_of::<SRB_HISTORY_ITEM>());
        let start_ptr: *mut SRB_HISTORY_ITEM =
            (&raw mut (*RequestHistory).History).cast::<SRB_HISTORY_ITEM>().add((*RequestHistory).UsedHistoryCount as usize);
        core::ptr::write_bytes(start_ptr as *mut u8, 0u8, bytesToZero);
    }

    ValidateSrbHistoryDataPresumptions(RequestHistory as *const SRB_HISTORY);
}
#[no_mangle]
pub unsafe extern "C" fn ValidateSrbHistoryDataPresumptions(
    RequestHistory: *const SRB_HISTORY,
) {
    if cfg!(debug_assertions) {
        let used_count = (*RequestHistory).UsedHistoryCount as usize;
        let history_base = core::ptr::addr_of!((*RequestHistory).History) as *const SRB_HISTORY_ITEM;

        for i in 0..used_count {
            let to_match = history_base.add(i);
            let srb_status = (*to_match).SrbStatus;
            let sense = (*to_match).NormalizedSenseData.SenseKey();
            let asc = (*to_match).NormalizedSenseData.AdditionalSenseCode;
            let ascq = (*to_match).NormalizedSenseData.AdditionalSenseCodeQualifier;

            let mut found_partially_compressed_item: bool =
                ((*to_match).ClassDriverUse != 0) && ((*to_match).ClassDriverUse != 0xFF);
            let mut found_uncompressed_item: bool = (*to_match).ClassDriverUse == 0;

            for j in (i + 1)..used_count {
                let found = history_base.add(j);
                if (srb_status == (*found).SrbStatus)
                    && (sense == (*found).NormalizedSenseData.SenseKey())
                    && (asc == (*found).NormalizedSenseData.AdditionalSenseCode)
                    && (ascq == (*found).NormalizedSenseData.AdditionalSenseCodeQualifier)
                {
                    if found_uncompressed_item && ((*found).ClassDriverUse != 0) {
                        CdromDbgPrintExHistory(
                            DPFLTR_CDROM_ID,
                            DPFLTR_ERROR_LEVEL,
                            b"History data has compressed history following uncompressed history for srbstatus/sense/asc/ascq of %02x/%02x/%02x/%02x at indices %d (%08x) and %d (%08x)\n\0"
                                .as_ptr() as *const i8,
                            srb_status as usize,
                            sense as usize,
                            asc as usize,
                            ascq as usize,
                            i,
                            i,
                            j,
                            j,
                        );
                        ntkrnl_sys::ASSERT!(false);
                    } else if found_partially_compressed_item && ((*found).ClassDriverUse == 0xFF) {
                        CdromDbgPrintExHistory(
                            DPFLTR_CDROM_ID,
                            DPFLTR_ERROR_LEVEL,
                            b"History data has fully compressed history following partially compressed history for srbstatus/sense/asc/ascq of %02x/%02x/%02x/%02x at indices %d (%08x) and %d (%08x)\n\0"
                                .as_ptr() as *const i8,
                            srb_status as usize,
                            sense as usize,
                            asc as usize,
                            ascq as usize,
                            i,
                            i,
                            j,
                            j,
                        );
                        ntkrnl_sys::ASSERT!(false);
                    }

                    if (*found).ClassDriverUse == 0 {
                        found_uncompressed_item = true;
                    } else if (*found).ClassDriverUse != 0xFF {
                        found_partially_compressed_item = true;
                    }
                }
            }
        }
    } else {
        let _ = RequestHistory;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScratchBuffer_SetupReadWriteSrb(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    StartingOffset: LARGE_INTEGER,
    RequiredLength: u32,
    DataBuffer: *mut u8,
    IsReadRequest: u8,
    UsePartialMdl: u8,
) {
    let scratch_context = core::ptr::addr_of_mut!((*DeviceExtension).ScratchContext);
    let srb: *mut SCSI_REQUEST_BLOCK =
        core::ptr::read_unaligned(core::ptr::addr_of!((*scratch_context).ScratchSrb));
    let scratch_request =
        core::ptr::read_unaligned(core::ptr::addr_of!((*scratch_context).ScratchRequest));
    let scratch_sense =
        core::ptr::read_unaligned(core::ptr::addr_of!((*scratch_context).ScratchSense));
    let partial_mdl =
        core::ptr::read_unaligned(core::ptr::addr_of!((*scratch_context).PartialMdl));
    let srb_bytes = srb.cast::<u8>();
    let cdb_bytes: *mut u8 =
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, Cdb));
    let mut logicalBlockAddr: LARGE_INTEGER = core::mem::zeroed();
    let numTransferBlocks: u32;

    let originalIrp: PIRP = WdfRequestWdmGetIrp(OriginalRequest);
    let irp: PIRP = WdfRequestWdmGetIrp(scratch_request);
    let mut irpStack: *mut IO_STACK_LOCATION;
    let requestContext: *mut CDROM_REQUEST_CONTEXT = RequestGetContext(scratch_request as *mut core::ffi::c_void);

    (*requestContext).OriginalRequest = OriginalRequest;

    // Int64ShrlMod32 expanded
    let sector_shift = core::ptr::read_unaligned(core::ptr::addr_of!((*DeviceExtension).SectorShift));
    logicalBlockAddr.QuadPart = ((StartingOffset.QuadPart as u64) >> sector_shift as u64) as i64;
    numTransferBlocks = RequiredLength >> sector_shift;

    irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;
    if IsReadRequest != 0 {
        (*irpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN;
    } else {
        (*irpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_OUT;
    }
    core::ptr::write_unaligned(
        core::ptr::addr_of_mut!((*irpStack).Parameters)
            as *mut *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK,
        srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK,
    );

    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, Length)).cast::<u16>(),
        SCSI_REQUEST_BLOCK_SIZE as u16,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, Function)).cast::<u8>(),
        SRB_FUNCTION_EXECUTE_SCSI as u8,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, QueueAction)).cast::<u8>(),
        SRB_SIMPLE_TAG_REQUEST as u8,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, SrbStatus)).cast::<u8>(),
        0,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, ScsiStatus)).cast::<u8>(),
        0,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, NextSrb)).cast::<*mut SCSI_REQUEST_BLOCK>(),
        core::ptr::null_mut(),
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, SenseInfoBufferLength)).cast::<u8>(),
        SENSE_BUFFER_SIZE as u8,
    );
    core::ptr::write_volatile(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, SenseInfoBuffer)).cast::<*mut core::ffi::c_void>(),
        scratch_sense as *mut core::ffi::c_void,
    );
    core::ptr::write_volatile(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, DataBuffer)).cast::<*mut core::ffi::c_void>(),
        DataBuffer as *mut core::ffi::c_void,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, DataTransferLength)).cast::<u32>(),
        RequiredLength,
    );

    let queue_sort_key = srb_bytes
        .wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, __bindgen_anon_1))
        .cast::<u32>();
    core::ptr::write_unaligned(queue_sort_key, logicalBlockAddr.__bindgen_anon_1.LowPart as u32);
    if logicalBlockAddr.QuadPart > 0xFFFFFFFF_u64 as i64 {
        core::ptr::write_unaligned(queue_sort_key, 0xFFFFFFFF);
    }

    core::ptr::write_volatile(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, OriginalRequest)).cast::<*mut core::ffi::c_void>(),
        irp as *mut core::ffi::c_void,
    );
    core::ptr::write_unaligned(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, TimeOutValue)).cast::<u32>(),
        core::ptr::read_unaligned(core::ptr::addr_of!((*DeviceExtension).TimeOutValue)),
    );

    if RequestIsRealtimeStreaming(OriginalRequest, IsReadRequest) != 0
        && ((*core::ptr::read_unaligned(core::ptr::addr_of!((*DeviceExtension).PrivateFdoData))).HackFlags & FDO_HACK_NO_STREAMING as ntkrnl_sys::ddk::ULONG_PTR) == 0
    {
        if IsReadRequest != 0 {
            RtlZeroMemory(cdb_bytes as *mut core::ffi::c_void, SIZEOF_CDB_READ12);
            // REVERSE_BYTES expanded
            let src_val: u32 = core::ptr::read(core::ptr::addr_of!(logicalBlockAddr.__bindgen_anon_1.LowPart) as *const u32);
            core::ptr::write_unaligned(cdb_bytes.wrapping_add(2) as *mut u32, src_val.swap_bytes());
            let src_val: u32 = core::ptr::read(core::ptr::addr_of!(numTransferBlocks) as *const u32);
            core::ptr::write_unaligned(cdb_bytes.wrapping_add(6) as *mut u32, src_val.swap_bytes());
            *cdb_bytes.wrapping_add(10) |= 0x80;
            *cdb_bytes = SCSIOP_READ12;
            core::ptr::write_unaligned(
                srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, CdbLength)).cast::<u8>(),
                SIZEOF_CDB_READ12 as u8,
            );
        } else {
            RtlZeroMemory(cdb_bytes as *mut core::ffi::c_void, SIZEOF_CDB_WRITE12);
            let src_val: u32 = core::ptr::read(core::ptr::addr_of!(logicalBlockAddr.__bindgen_anon_1.LowPart) as *const u32);
            core::ptr::write_unaligned(cdb_bytes.wrapping_add(2) as *mut u32, src_val.swap_bytes());
            let src_val: u32 = core::ptr::read(core::ptr::addr_of!(numTransferBlocks) as *const u32);
            core::ptr::write_unaligned(cdb_bytes.wrapping_add(6) as *mut u32, src_val.swap_bytes());
            *cdb_bytes.wrapping_add(10) |= 0x80;
            *cdb_bytes = SCSIOP_WRITE12;
            core::ptr::write_unaligned(
                srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, CdbLength)).cast::<u8>(),
                SIZEOF_CDB_WRITE12 as u8,
            );
        }
    } else {
        RtlZeroMemory(cdb_bytes as *mut core::ffi::c_void, SIZEOF_CDB_CDB10);
        let fp: *const FOUR_BYTE = core::ptr::addr_of!(logicalBlockAddr.__bindgen_anon_1.LowPart) as *const FOUR_BYTE;
        *cdb_bytes.wrapping_add(2) = (*fp).__bindgen_anon_1.as_ref().Byte3;
        *cdb_bytes.wrapping_add(3) = (*fp).__bindgen_anon_1.as_ref().Byte2;
        *cdb_bytes.wrapping_add(4) = (*fp).__bindgen_anon_1.as_ref().Byte1;
        *cdb_bytes.wrapping_add(5) = (*fp).__bindgen_anon_1.as_ref().Byte0;
        let fp_num: *const FOUR_BYTE = core::ptr::addr_of!(numTransferBlocks) as *const FOUR_BYTE;
        *cdb_bytes.wrapping_add(7) = (*fp_num).__bindgen_anon_1.as_ref().Byte1;
        *cdb_bytes.wrapping_add(8) = (*fp_num).__bindgen_anon_1.as_ref().Byte0;
        *cdb_bytes = if IsReadRequest != 0 { SCSIOP_READ } else { SCSIOP_WRITE };
        core::ptr::write_unaligned(
            srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, CdbLength)).cast::<u8>(),
            SIZEOF_CDB_CDB10 as u8,
        );
    }

    let srb_flags = srb_bytes
        .wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, SrbFlags))
        .cast::<u32>();
    core::ptr::write_unaligned(
        srb_flags,
        core::ptr::read_unaligned(core::ptr::addr_of!((*DeviceExtension).SrbFlags)),
    );
    if ((*originalIrp).Flags & IRP_PAGING_IO) != 0 || ((*originalIrp).Flags & IRP_SYNCHRONOUS_PAGING_IO) != 0 {
        core::ptr::write_unaligned(
            srb_flags,
            core::ptr::read_unaligned(srb_flags) | SRB_CLASS_FLAGS_PAGING,
        );
    }

    core::ptr::write_unaligned(
        srb_flags,
        core::ptr::read_unaligned(srb_flags)
            | if IsReadRequest != 0 { SRB_FLAGS_DATA_IN } else { SRB_FLAGS_DATA_OUT },
    );
    core::ptr::write_unaligned(
        srb_flags,
        core::ptr::read_unaligned(srb_flags) | SRB_FLAGS_ADAPTER_CACHE_ENABLE,
    );

    if UsePartialMdl == 0 {
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*irp).MdlAddress),
                core::ptr::read_volatile(core::ptr::addr_of!((*originalIrp).MdlAddress)),
            );
    } else {
        if core::ptr::read_unaligned(core::ptr::addr_of!((*scratch_context).PartialMdlIsBuilt)) != 0 {
            MmPrepareMdlForReuse(partial_mdl);
        }
        IoBuildPartialMdl(
            (*originalIrp).MdlAddress,
            partial_mdl,
            core::ptr::read_unaligned(
                srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, DataBuffer)).cast::<*mut core::ffi::c_void>(),
            ),
            core::ptr::read_unaligned(
                srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, DataTransferLength)).cast::<u32>(),
            ),
        );
        core::ptr::write_unaligned(core::ptr::addr_of_mut!((*scratch_context).PartialMdlIsBuilt), 1);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*irp).MdlAddress), partial_mdl);
    }

    core::ptr::write_volatile(
        srb_bytes.wrapping_add(core::mem::offset_of!(SCSI_REQUEST_BLOCK, SrbExtension)).cast::<*mut core::ffi::c_void>(),
        originalIrp as *mut core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ExecuteCdbEx(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    OriginalRequest: *mut core::ffi::c_void,
    TransferSize: u32,
    GetDataFromDevice: u8,
    Cdb: *mut CDB,
    OprationLength: u8,
    TimeoutValue: u32,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let srb: *mut SCSI_REQUEST_BLOCK = (*DeviceExtension).ScratchContext.ScratchSrb;
    let cdb: *mut CDB = (*srb).Cdb.as_mut_ptr() as *mut CDB;

    let mut shouldRetry: u8 = 1;
    let mut timesAlreadyRetried: u32 = 0;
    let mut retryIn100nsUnits: i64 = 0;

    PAGED_CODE!();

    let mut ecx_freq: i64 = 0;
    let ecx_t0 = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(ecx_freq));

    while shouldRetry != 0 {
        ScratchBuffer_SetupSrb(DeviceExtension, OriginalRequest as WDFREQUEST, TransferSize, GetDataFromDevice);

        core::ptr::copy_nonoverlapping(Cdb as *const CDB, cdb, 1usize);

        (*srb).CdbLength = OprationLength;

        if TimeoutValue > 0 {
            (*srb).TimeOutValue = TimeoutValue;
        }

        ScratchBuffer_SendSrb(DeviceExtension, 1u8, core::ptr::null_mut());

        if (*srb).SrbStatus == SRB_STATUS_ABORTED && *(*srb).__bindgen_anon_1.InternalStatus.as_ref() == STATUS_CANCELLED as u32 {
            shouldRetry = 0u8;
            status = STATUS_CANCELLED;
        } else {
            shouldRetry = RequestSenseInfoInterpretForScratchBuffer(
                DeviceExtension,
                timesAlreadyRetried,
                core::ptr::addr_of_mut!(status),
                core::ptr::addr_of_mut!(retryIn100nsUnits),
            );
            if shouldRetry != 0 {
                let mut t: LARGE_INTEGER = core::mem::zeroed();
                t.QuadPart = -retryIn100nsUnits;
                timesAlreadyRetried += 1;
                KeDelayExecutionThread(0i32, 0u8, core::ptr::addr_of_mut!(t));
                ScratchBuffer_ResetItems(DeviceExtension, 0u8);
            }
        }
    }

    {
        let ecx_t1 = KeQueryPerformanceCounter(core::ptr::null_mut());
        let us = if ecx_freq != 0 { (ecx_t1 - ecx_t0).wrapping_mul(1_000_000) / ecx_freq } else { 0 };
        if us > 500_000 {
            let op = *(Cdb as *const u8);
            CdromDbgPrintForce2(
                b"[CDBEX] blocked us=%lx op_retries_srbst=%lx\n\0".as_ptr() as *const i8,
                us as usize,
                ((op as usize) << 24) | ((timesAlreadyRetried as usize) << 8) | ((*srb).SrbStatus as usize),
            );
        }
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*--
//
// Copyright (C) Microsoft Corporation. All rights reserved.
//
// Module Name:
//
//     scratch.c
//
// Abstract:
//
//     Functions for using common scratch buffer
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "stddef.h"
// #include "string.h"
//
// #include "ntddk.h"
// #include "ntddstor.h"
// #include "cdrom.h"
// #include "ioctl.h"
// #include "scratch.h"
// #include "mmc.h"
//
// #ifdef DEBUG_USE_WPP
// #include "scratch.tmh"
// #endif
//
// // Forward declarations
// EVT_WDF_REQUEST_COMPLETION_ROUTINE  ScratchBuffer_ReadWriteCompletionRoutine;
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, ScratchBuffer_Deallocate)
// #pragma alloc_text(PAGE, ScratchBuffer_Allocate)
// #pragma alloc_text(PAGE, ScratchBuffer_SetupSrb)
// #pragma alloc_text(PAGE, ScratchBuffer_ExecuteCdbEx)
//
// #endif
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// ScratchBuffer_Deallocate(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     release all resources allocated for scratch.
//
// Arguments:
//
//     DeviceExtension - device extension
//
// Return Value:
//
//     none
//
// --*/
// {
//     PAGED_CODE ();
//
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchInUse == 0);
//
//     if (DeviceExtension->ScratchContext.ScratchHistory != NULL)
//     {
//         ExFreePool(DeviceExtension->ScratchContext.ScratchHistory);
//         DeviceExtension->ScratchContext.ScratchHistory = NULL;
//     }
//     if (DeviceExtension->ScratchContext.ScratchSense != NULL)
//     {
//         ExFreePool(DeviceExtension->ScratchContext.ScratchSense);
//         DeviceExtension->ScratchContext.ScratchSense = NULL;
//     }
//     if (DeviceExtension->ScratchContext.ScratchSrb != NULL)
//     {
//         ExFreePool(DeviceExtension->ScratchContext.ScratchSrb);
//         DeviceExtension->ScratchContext.ScratchSrb = NULL;
//     }
//     if (DeviceExtension->ScratchContext.ScratchBufferSize != 0)
//     {
//         DeviceExtension->ScratchContext.ScratchBufferSize = 0;
//     }
//     if (DeviceExtension->ScratchContext.ScratchBufferMdl != NULL)
//     {
//         IoFreeMdl(DeviceExtension->ScratchContext.ScratchBufferMdl);
//         DeviceExtension->ScratchContext.ScratchBufferMdl = NULL;
//     }
//     if (DeviceExtension->ScratchContext.ScratchBuffer != NULL)
//     {
//         ExFreePool(DeviceExtension->ScratchContext.ScratchBuffer);
//         DeviceExtension->ScratchContext.ScratchBuffer = NULL;
//     }
//
//     if (DeviceExtension->ScratchContext.PartialMdl != NULL)
//     {
//         IoFreeMdl(DeviceExtension->ScratchContext.PartialMdl);
//         DeviceExtension->ScratchContext.PartialMdl = NULL;
//     }
//
//     if (DeviceExtension->ScratchContext.ScratchRequest != NULL)
//     {
//         PIRP irp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);
//         if (irp->MdlAddress)
//         {
//             irp->MdlAddress = NULL;
//         }
//         WdfObjectDelete(DeviceExtension->ScratchContext.ScratchRequest);
//         DeviceExtension->ScratchContext.ScratchRequest = NULL;
//     }
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// BOOLEAN
// ScratchBuffer_Allocate(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     allocate resources allocated for scratch.
//
// Arguments:
//
//     DeviceExtension - device extension
//
// Return Value:
//
//     none
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchInUse == 0);
//
//     // quick-exit if already allocated
//     if ((DeviceExtension->ScratchContext.ScratchBuffer     != NULL) &&
//         (DeviceExtension->ScratchContext.ScratchBufferMdl  != NULL) &&
//         (DeviceExtension->ScratchContext.ScratchBufferSize != 0)    &&
//         (DeviceExtension->ScratchContext.ScratchRequest    != NULL) &&
//         (DeviceExtension->ScratchContext.ScratchSrb        != NULL) &&
//         (DeviceExtension->ScratchContext.ScratchHistory    != NULL) &&
//         (DeviceExtension->ScratchContext.PartialMdl  != NULL)
//         )
//     {
//         return TRUE;
//     }
//
//     // validate max transfer already determined
//     NT_ASSERT(DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes != 0);
//
//     // validate no partially-saved state
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBuffer     == NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBufferMdl  == NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBufferSize == 0);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchRequest    == NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.PartialMdl == NULL);
//
//     // limit the scratch buffer to between 4k and 64k (so data length fits into USHORT -- req'd for many commands)
//     DeviceExtension->ScratchContext.ScratchBufferSize = min(DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes, (64*1024));
//
//     // allocate the buffer
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->ScratchContext.ScratchBuffer = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                                               DeviceExtension->ScratchContext.ScratchBufferSize,
//                                                                               CDROM_TAG_SCRATCH);
//         if (DeviceExtension->ScratchContext.ScratchBuffer == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate scratch buffer of %x bytes\n",
//                         DeviceExtension->ScratchContext.ScratchBufferSize
//                         ));
//         }
//         else if (BYTE_OFFSET(DeviceExtension->ScratchContext.ScratchBuffer) != 0)
//         {
//             status = STATUS_INTERNAL_ERROR;
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_INIT,
//                         "Allocation of %x bytes non-paged pool was not "
//                         "allocated on page boundary?  STATUS_INTERNAL_ERROR\n",
//                         DeviceExtension->ScratchContext.ScratchBufferSize
//                         ));
//         }
//     }
//
//     // allocate the MDL
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->ScratchContext.ScratchBufferMdl = IoAllocateMdl(DeviceExtension->ScratchContext.ScratchBuffer,
//                                                                          DeviceExtension->ScratchContext.ScratchBufferSize,
//                                                                          FALSE, FALSE, NULL);
//         if (DeviceExtension->ScratchContext.ScratchBufferMdl == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate MDL for %x byte buffer\n",
//                         DeviceExtension->ScratchContext.ScratchBufferSize
//                         ));
//         }
//         else
//         {
//             MmBuildMdlForNonPagedPool(DeviceExtension->ScratchContext.ScratchBufferMdl);
//         }
//     }
//
//     // create the request
//     if (NT_SUCCESS(status))
//     {
//         WDF_OBJECT_ATTRIBUTES attributes;
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//
//         status =  WdfRequestCreate(&attributes,
//                                    DeviceExtension->IoTarget,
//                                    &DeviceExtension->ScratchContext.ScratchRequest);
//
//         if ((!NT_SUCCESS(status)) ||
//             (DeviceExtension->ScratchContext.ScratchRequest == NULL))
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate scratch MDL \n"));
//         }
//     }
//
//     // allocate the srb
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->ScratchContext.ScratchSrb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                                            sizeof(SCSI_REQUEST_BLOCK),
//                                                                            CDROM_TAG_SCRATCH);
//
//         if (DeviceExtension->ScratchContext.ScratchSrb == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate scratch SRB\n"));
//         }
//     }
//
//     // allocate the sense buffer
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->ScratchContext.ScratchSense = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                                              sizeof(SENSE_DATA),
//                                                                              CDROM_TAG_SCRATCH);
//
//         if (DeviceExtension->ScratchContext.ScratchSense == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate scratch sense data\n"
//                         ));
//         }
//     }
//
//     // allocate the SRB history data
//     if (NT_SUCCESS(status))
//     {
//         size_t allocationSize = sizeof(SRB_HISTORY) - sizeof(SRB_HISTORY_ITEM);
//         allocationSize += 20 * sizeof(SRB_HISTORY_ITEM);
//
//         DeviceExtension->ScratchContext.ScratchHistory = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                                                allocationSize,
//                                                                                CDROM_TAG_SCRATCH);
//         if (DeviceExtension->ScratchContext.ScratchHistory == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to allocate scratch history buffer\n"
//                         ));
//         }
//         else
//         {
//             // must be initialized here...
//             RtlZeroMemory(DeviceExtension->ScratchContext.ScratchHistory, allocationSize);
//             DeviceExtension->ScratchContext.ScratchHistory->TotalHistoryCount = 20;
//         }
//     }
//
//     // allocate the MDL
//     if (NT_SUCCESS(status))
//     {
//         ULONG transferLength = 0;
//
//         status = RtlULongAdd(DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes, PAGE_SIZE, &transferLength);
//         if (NT_SUCCESS(status))
//         {
//             DeviceExtension->ScratchContext.PartialMdlIsBuilt = FALSE;
//             DeviceExtension->ScratchContext.PartialMdl = IoAllocateMdl(NULL,
//                                                                        transferLength,
//                                                                        FALSE,
//                                                                        FALSE,
//                                                                        NULL);
//             if (DeviceExtension->ScratchContext.PartialMdl == NULL)
//             {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                             "Failed to allocate MDL for %x byte buffer\n",
//                             DeviceExtension->ScratchContext.ScratchBufferSize
//                             ));
//             }
//             else
//             {
//                 NT_ASSERT(DeviceExtension->ScratchContext.PartialMdl->Size >=
//                        (CSHORT)(sizeof(MDL) + BYTES_TO_PAGES(DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes) * sizeof(PFN_NUMBER)));
//             }
//         }
//         else
//         {
//             status = STATUS_INTEGER_OVERFLOW;
//         }
//     }
//
//     // cleanup on failure
//     if (!NT_SUCCESS(status))
//     {
//         ScratchBuffer_Deallocate(DeviceExtension);
//     }
//
//     return NT_SUCCESS(status);
// }
//
//
// VOID
// ScratchBuffer_ResetItems(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                    ResetRequestHistory
//     )
// /*++
//
// Routine Description:
//
//     reset scratch items for reuse.
//
// Arguments:
//
//     DeviceExtension - device extension
//     ResetRequestHistory - reset history fields or not
//
// Return Value:
//
//     none
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     WDF_REQUEST_REUSE_PARAMS reuseParams;
//     PIRP                     irp = NULL;
//
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchHistory    != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchSense      != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchSrb        != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchRequest    != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBufferSize != 0);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBuffer     != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchBufferMdl  != NULL);
//     NT_ASSERT(DeviceExtension->ScratchContext.ScratchInUse      != 0);
//
//     irp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);
//
//     if (ResetRequestHistory)
//     {
//         PSRB_HISTORY history = DeviceExtension->ScratchContext.ScratchHistory;
//         RtlZeroMemory(history->History, sizeof(SRB_HISTORY_ITEM) * history->TotalHistoryCount);
//         history->ClassDriverUse[0] = 0;
//         history->ClassDriverUse[1] = 0;
//         history->ClassDriverUse[2] = 0;
//         history->ClassDriverUse[3] = 0;
//         history->UsedHistoryCount  = 0;
//     }
//
//     // re-use the KMDF request object
//
//     // deassign the MdlAddress, this is the value we assign explicitly.
//     // this is to prevent WdfRequestReuse to release the Mdl unexpectly.
//     if (irp->MdlAddress)
//     {
//         irp->MdlAddress = NULL;
//     }
//
//     WDF_REQUEST_REUSE_PARAMS_INIT(&reuseParams, WDF_REQUEST_REUSE_NO_FLAGS, STATUS_NOT_SUPPORTED);
//     status = WdfRequestReuse(DeviceExtension->ScratchContext.ScratchRequest, &reuseParams);
//     // WDF request to format the request befor sending it
//     if (NT_SUCCESS(status))
//     {
//         // clean up completion routine.
//         WdfRequestSetCompletionRoutine(DeviceExtension->ScratchContext.ScratchRequest, NULL, NULL);
//
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                                 DeviceExtension->ScratchContext.ScratchRequest,
//                                                                 IOCTL_SCSI_EXECUTE_IN,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "ScratchBuffer_ResetItems: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n",
//                        status));
//         }
//     }
//
//     RtlZeroMemory(DeviceExtension->ScratchContext.ScratchSense, sizeof(SENSE_DATA));
//     RtlZeroMemory(DeviceExtension->ScratchContext.ScratchSrb, sizeof(SCSI_REQUEST_BLOCK));
//
//     return;
// }
//
//
// NTSTATUS
// ScratchBuffer_PerformNextReadWrite(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      BOOLEAN                  FirstTry
//     )
// /*++
//
// Routine Description:
//
//     This function asynchronously sends the next read/write SRB down the stack.
//
// Arguments:
//
//     DeviceExtension - Device extension
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_SCRATCH_READ_WRITE_CONTEXT   readWriteContext = &DeviceExtension->ScratchContext.ScratchReadWriteContext;
//     PCDROM_REQUEST_CONTEXT              requestContext = RequestGetContext(DeviceExtension->ScratchContext.ScratchRequest);
//     WDFREQUEST                          originalRequest = requestContext->OriginalRequest;
//     NTSTATUS                            status = STATUS_SUCCESS;
//
//     ULONG       transferSize;
//     BOOLEAN     usePartialMdl;
//
//     transferSize = min((readWriteContext->EntireXferLen - readWriteContext->TransferedBytes), readWriteContext->MaxLength);
//
//     if (FirstTry)
//     {
//         DeviceExtension->ScratchContext.NumRetries = 0;
//     }
//
//     ScratchBuffer_ResetItems(DeviceExtension, FALSE);
//
//     usePartialMdl = (readWriteContext->PacketsCount > 1 || readWriteContext->TransferedBytes > 0);
//
//     ScratchBuffer_SetupReadWriteSrb(DeviceExtension,
//                                     originalRequest,
//                                     readWriteContext->StartingOffset,
//                                     transferSize,
//                                     readWriteContext->DataBuffer,
//                                     readWriteContext->IsRead,
//                                     usePartialMdl
//                                     );
//
//     WdfRequestSetCompletionRoutine(DeviceExtension->ScratchContext.ScratchRequest,
//             ScratchBuffer_ReadWriteCompletionRoutine, DeviceExtension);
//
//     status = ScratchBuffer_SendSrb(DeviceExtension, FALSE, (FirstTry ? &readWriteContext->SrbHistoryItem : NULL));
//
//     return status;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ScratchBuffer_ReadWriteTimerRoutine(
//     struct _KDPC *Dpc,
//     PVOID         DeferredContext,
//     PVOID         SystemArgument1,
//     PVOID         SystemArgument2
//     )
// /*++
//
// Routine Description:
//
//     Timer routine for retrying read and write requests.
//
// Arguments:
//
//     Timer - WDF timer
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION             deviceExtension = NULL;
//     PCDROM_SCRATCH_READ_WRITE_CONTEXT   readWriteContext = NULL;
//     WDFREQUEST                          originalRequest = NULL;
//     PCDROM_REQUEST_CONTEXT              requestContext = NULL;
//     NTSTATUS                            status = STATUS_SUCCESS;
//     KIRQL                               oldIrql;
//
//     UNREFERENCED_PARAMETER(Dpc);
//     UNREFERENCED_PARAMETER(SystemArgument1);
//     UNREFERENCED_PARAMETER(SystemArgument2);
//
//     if (DeferredContext == NULL)
//     {
//         // This is impossible, but definition of KDEFERRED_ROUTINE allows optional argument,
//         // and thus OACR will complain.
//
//         return;
//     }
//
//     originalRequest = (WDFREQUEST) DeferredContext;
//     requestContext = RequestGetContext(originalRequest);
//
//     KeAcquireSpinLock(&requestContext->ReadWriteCancelSpinLock, &oldIrql);
//
//     if (!requestContext->ReadWriteIsCompleted)
//     {
//         // As the first step, unregister the cancellation routine
//         status = WdfRequestUnmarkCancelable(originalRequest);
//     }
//     else
//     {
//         status = STATUS_CANCELLED;
//     }
//
//     KeReleaseSpinLock(&requestContext->ReadWriteCancelSpinLock, oldIrql);
//
//     if (status != STATUS_CANCELLED)
//     {
//         deviceExtension = requestContext->DeviceExtension;
//         readWriteContext = &deviceExtension->ScratchContext.ScratchReadWriteContext;
//
//         // We use timer only for retries, that's why the second parameter is always FALSE
//         status = ScratchBuffer_PerformNextReadWrite(deviceExtension, FALSE);
//
//         if (!NT_SUCCESS(status))
//         {
//             ScratchBuffer_EndUse(deviceExtension);
//             RequestCompletion(deviceExtension, originalRequest, status, readWriteContext->TransferedBytes);
//         }
//     }
//
//     //
//     // Drop the extra reference
//     //
//     WdfObjectDereference(originalRequest);
// }
//
//
// EVT_WDF_REQUEST_CANCEL  ScratchBuffer_ReadWriteEvtRequestCancel;
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ScratchBuffer_ReadWriteEvtRequestCancel(
//      WDFREQUEST  Request
//     )
// /*++
//
// Routine Description:
//
//     Cancels a request waiting for the read/write timer to expire. This function does not
//     support cancellation of requests that have already been sent down.
//
// Arguments:
//
//     Request - WDF request
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_REQUEST_CONTEXT             requestContext = RequestGetContext(Request);
//     PCDROM_DEVICE_EXTENSION            deviceExtension = requestContext->DeviceExtension;
//     PCDROM_SCRATCH_READ_WRITE_CONTEXT  readWriteContext = &deviceExtension->ScratchContext.ScratchReadWriteContext;
//     KIRQL                              oldIrql;
//
//     KeAcquireSpinLock(&requestContext->ReadWriteCancelSpinLock, &oldIrql);
//
//     if (KeCancelTimer(&requestContext->ReadWriteTimer))
//     {
//        //
//        // Timer is canceled, we own the request.  Drop the reference we took before
//        // queueing the timer.
//        //
//        WdfObjectDereference(Request);
//     }
//     else
//     {
//         //
//         // Timer will run and drop the reference but it won't complete the request
//         // because we set IsCompleted to TRUE
//         //
//     }
//
//     requestContext->ReadWriteIsCompleted = TRUE;
//
//     KeReleaseSpinLock(&requestContext->ReadWriteCancelSpinLock, oldIrql);
//
//     ScratchBuffer_EndUse(deviceExtension);
//
//     // If WdfTimerStop returned TRUE, it means this request was scheduled for a retry
//     // and the retry has not happened yet. We just need to cancel it and release the scratch buffer.
//     RequestCompletion(deviceExtension, Request, STATUS_CANCELLED, readWriteContext->TransferedBytes);
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ScratchBuffer_ReadWriteCompletionRoutine(
//      WDFREQUEST  Request,
//      WDFIOTARGET  Target,
//      PWDF_REQUEST_COMPLETION_PARAMS  Params,
//      WDFCONTEXT  Context
//     )
// /*++
//
// Routine Description:
//
//     Read/write request completion routine.
//
// Arguments:
//     Request - WDF request
//     Target - The IO target the request was completed by.
//     Params - the request completion parameters
//     Context - context
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION             deviceExtension = (PCDROM_DEVICE_EXTENSION) Context;
//     PCDROM_SCRATCH_READ_WRITE_CONTEXT   readWriteContext = &deviceExtension->ScratchContext.ScratchReadWriteContext;
//     NTSTATUS                            status = STATUS_SUCCESS;
//     PCDROM_REQUEST_CONTEXT              requestContext = RequestGetContext(deviceExtension->ScratchContext.ScratchRequest);
//     WDFREQUEST                          originalRequest = requestContext->OriginalRequest;
//
//     if (!NT_SUCCESS(WdfRequestGetStatus(Request)))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                    "WdfRequestSend: %lx\n",
//                     WdfRequestGetStatus(Request)
//                     ));
//     }
//
//     UNREFERENCED_PARAMETER(Params);
//     UNREFERENCED_PARAMETER(Target);
//
//     // We are not calling ScratchBuffer_BeginUse / ScratchBuffer_EndUse in this function, because we already own
//     // the scratch buffer if this function is being called.
//
//     if ((deviceExtension->ScratchContext.ScratchSrb->SrbStatus == SRB_STATUS_ABORTED) &&
//         (deviceExtension->ScratchContext.ScratchSrb->InternalStatus == STATUS_CANCELLED))
//     {
//         // The request has been cancelled, just need to complete it
//     }
//     else if (SRB_STATUS(deviceExtension->ScratchContext.ScratchSrb->SrbStatus) != SRB_STATUS_SUCCESS)
//     {
//         // The SCSI command that we sent down has failed, retry it if necessary
//         BOOLEAN shouldRetry = TRUE;
//         LONGLONG retryIn100nsUnits = 0;
//
//         shouldRetry = RequestSenseInfoInterpretForScratchBuffer(deviceExtension,
//                                                                 deviceExtension->ScratchContext.NumRetries,
//                                                                 &status,
//                                                                 &retryIn100nsUnits);
//
//         if (shouldRetry)
//         {
//             deviceExtension->ScratchContext.NumRetries++;
//
//             if (retryIn100nsUnits == 0)
//             {
//                 // We take a shortcut here by calling ScratchBuffer_PerformNextReadWrite directly:
//                 // this helps to avoid unnecessary context switch.
//                 status = ScratchBuffer_PerformNextReadWrite(deviceExtension, FALSE);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     // We're not done with the request yet, no need to complete it now
//                     return;
//                 }
//             }
//             else
//             {
//                 PCDROM_REQUEST_CONTEXT originalRequestContext = RequestGetContext(originalRequest);
//                 KIRQL                  oldIrql;
//
//                 //
//                 // Initialize the spin lock and timer local to the original request.
//                 //
//                 if (!originalRequestContext->ReadWriteRetryInitialized)
//                 {
//                     KeInitializeSpinLock(&originalRequestContext->ReadWriteCancelSpinLock);
//                     KeInitializeTimer(&originalRequestContext->ReadWriteTimer);
//                     KeInitializeDpc(&originalRequestContext->ReadWriteDpc, ScratchBuffer_ReadWriteTimerRoutine, originalRequest);
//                     originalRequestContext->ReadWriteRetryInitialized = TRUE;
//                 }
//
//                 KeAcquireSpinLock(&requestContext->ReadWriteCancelSpinLock, &oldIrql);
//
//                 status = WdfRequestMarkCancelableEx(originalRequest, ScratchBuffer_ReadWriteEvtRequestCancel);
//
//                 if (status == STATUS_CANCELLED)
//                 {
//                     requestContext->ReadWriteIsCompleted = TRUE;
//
//                     KeReleaseSpinLock(&requestContext->ReadWriteCancelSpinLock, oldIrql);
//                 }
//                 else
//                 {
//                     LARGE_INTEGER t;
//
//                     t.QuadPart = -retryIn100nsUnits;
//
//                     WdfObjectReference(originalRequest);
//
//                     // Use negative time to indicate that we want a relative delay
//                     KeSetTimer(&originalRequestContext->ReadWriteTimer,
//                                t,
//                                &originalRequestContext->ReadWriteDpc
//                                );
//
//                     KeReleaseSpinLock(&requestContext->ReadWriteCancelSpinLock, oldIrql);
//
//                     return;
//                 }
//             }
//         }
//     }
//     else
//     {
//         // The SCSI command has succeeded
//         readWriteContext->DataBuffer += deviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//         readWriteContext->StartingOffset.QuadPart += deviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//         readWriteContext->TransferedBytes += deviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//         readWriteContext->PacketsCount--;
//
//         // Update the SRB history item
//         if (readWriteContext->SrbHistoryItem)
//         {
//             ULONG senseSize;
//
//             // Query the tick count and store in the history
//             KeQueryTickCount(&readWriteContext->SrbHistoryItem->TickCountCompleted);
//
//             // Copy the SRB Status...
//             readWriteContext->SrbHistoryItem->SrbStatus = deviceExtension->ScratchContext.ScratchSrb->SrbStatus;
//
//             // Determine the amount of valid sense data
//             if (deviceExtension->ScratchContext.ScratchSrb->SenseInfoBufferLength >=
//                     RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength))
//             {
//                 PSENSE_DATA sense = (PSENSE_DATA)deviceExtension->ScratchContext.ScratchSrb->SenseInfoBuffer;
//                 senseSize = RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength) +
//                             sense->AdditionalSenseLength;
//                 senseSize = min(senseSize, sizeof(SENSE_DATA));
//             }
//             else
//             {
//                 senseSize = deviceExtension->ScratchContext.ScratchSrb->SenseInfoBufferLength;
//             }
//
//             // Normalize the sense data copy in the history
//             RtlZeroMemory(&(readWriteContext->SrbHistoryItem->NormalizedSenseData), sizeof(SENSE_DATA));
//             RtlCopyMemory(&(readWriteContext->SrbHistoryItem->NormalizedSenseData),
//                     deviceExtension->ScratchContext.ScratchSrb->SenseInfoBuffer, senseSize);
//         }
//
//         // Check whether we need to send more SCSI commands to complete the request
//         if (readWriteContext->PacketsCount > 0)
//         {
//             status = ScratchBuffer_PerformNextReadWrite(deviceExtension, TRUE);
//
//             if (NT_SUCCESS(status))
//             {
//                 // We're not done with the request yet, no need to complete it now
//                 return;
//             }
//         }
//     }
//
//     ScratchBuffer_EndUse(deviceExtension);
//
//
//     RequestCompletion(deviceExtension, originalRequest, status, readWriteContext->TransferedBytes);
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// ScratchBuffer_SetupSrb(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      WDFREQUEST             OriginalRequest,
//      ULONG                      MaximumTransferLength,
//      BOOLEAN                    GetDataFromDevice
//     )
// /*++
//
// Routine Description:
//
//     setup scratch SRB for sending out.
//
// Arguments:
//
//     DeviceExtension - device extension
//     OriginalRequest - original request delivered by WDF
//     MaximumTransferLength - transfer length
//     GetDataFromDevice - TRUE (get data from device); FALSE (send data to device)
//
// Return Value:
//
//     none
//
// --*/
// {
//     WDFREQUEST              request = DeviceExtension->ScratchContext.ScratchRequest;
//     PIRP                    irp = WdfRequestWdmGetIrp(request);
//     PSCSI_REQUEST_BLOCK     srb = DeviceExtension->ScratchContext.ScratchSrb;
//     PIO_STACK_LOCATION      irpStack = NULL;
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(request);
//
//     PAGED_CODE ();
//
//     requestContext->OriginalRequest = OriginalRequest;
//
//     // set to use the full scratch buffer via the scratch SRB
//     irpStack = IoGetNextIrpStackLocation(irp);
//     irpStack->MajorFunction = IRP_MJ_SCSI;
//     if (MaximumTransferLength == 0)
//     {
//         irpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_NONE;
//     }
//     else if (GetDataFromDevice)
//     {
//         irpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN;
//     }
//     else
//     {
//         irpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_OUT;
//     }
//     irpStack->Parameters.Scsi.Srb = srb;
//
//     if (MaximumTransferLength > 0)
//     {
//         // the Irp must show the MDL's address for the transfer
//         irp->MdlAddress = DeviceExtension->ScratchContext.ScratchBufferMdl;
//
//         srb->DataBuffer = DeviceExtension->ScratchContext.ScratchBuffer;
//     }
//
//     // prepare the SRB with default values
//     srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//     srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//     srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//     srb->SrbStatus = 0;
//     srb->ScsiStatus = 0;
//     srb->NextSrb = NULL;
//     srb->OriginalRequest = irp;
//     srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//     srb->SenseInfoBuffer = DeviceExtension->ScratchContext.ScratchSense;
//
//     srb->CdbLength = 16; // to cause failures if not set correctly -- CD devices limited to 12 bytes for now...
//
//     srb->DataTransferLength = min(DeviceExtension->ScratchContext.ScratchBufferSize, MaximumTransferLength);
//     srb->TimeOutValue = DeviceExtension->TimeOutValue;
//     srb->SrbFlags = DeviceExtension->SrbFlags;
//     SET_FLAG(srb->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//     SET_FLAG(srb->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     if (MaximumTransferLength == 0)
//     {
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_NO_DATA_TRANSFER);
//     }
//     else if (GetDataFromDevice)
//     {
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_DATA_IN);
//     }
//     else
//     {
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_DATA_OUT);
//     }
// }
//
//
// NTSTATUS
// ScratchBuffer_SendSrb(
//          PCDROM_DEVICE_EXTENSION DeviceExtension,
//             BOOLEAN                 SynchronousSrb,
//     
//     
//                 PSRB_HISTORY_ITEM       *SrbHistoryItem
//     )
// /*++
//
// Routine Description:
//
//     Send the command from the scratch SRB to lower driver and retry if necessary.
//
// Arguments:
//
//     DeviceExtension - device extension
//     SynchronousSrb - indicates whether the SRB needs to be sent synchronously or nor
//     SrbHistoryItem - storage for SRB history item, if this is an asynchronous request
//
// Return Value:
//
//     none
//
// --*/
// {
//     NTSTATUS                 status  = STATUS_SUCCESS;
//     PSCSI_REQUEST_BLOCK      srb = DeviceExtension->ScratchContext.ScratchSrb;
//     PSRB_HISTORY             history = DeviceExtension->ScratchContext.ScratchHistory;
//     PSRB_HISTORY_ITEM        item = NULL;
//     BOOLEAN                  requestCancelled = FALSE;
//
//     srb->InternalStatus = 0;
//     srb->SrbStatus = 0;
//
//     // allocate/update history pre-command, if it is a synchronous request or we were supplied
//     // with a storage for the history item
//     if (SynchronousSrb || SrbHistoryItem != NULL)
//     {
//         // sending a packet implies a new history unit is to be used.
//         NT_ASSERT( history->UsedHistoryCount <= history->TotalHistoryCount );
//
//         // if already all used up, remove at least one history unit
//         if (history->UsedHistoryCount == history->TotalHistoryCount )
//         {
//             CompressSrbHistoryData(history);
//             NT_ASSERT( history->UsedHistoryCount < history->TotalHistoryCount );
//         }
//
//         // thus, since we are about to increment the count, it must now be less...
//         NT_ASSERT( history->UsedHistoryCount < history->TotalHistoryCount );
//
//         // increment the number of history units in use
//         history->UsedHistoryCount++;
//
//         // determine index to use
//         item = &( history->History[ history->UsedHistoryCount-1 ] );
//
//         if (SrbHistoryItem != NULL)
//         {
//             *SrbHistoryItem = item;
//         }
//
//         // zero out the history item
//         RtlZeroMemory(item, sizeof(SRB_HISTORY_ITEM));
//
//         // Query the tick count and store in the history
//         KeQueryTickCount(&item->TickCountSent);
//     }
//
//     // get cancellation status;
//     {
//         PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(DeviceExtension->ScratchContext.ScratchRequest);
//
//         if (requestContext->OriginalRequest != NULL)
//         {
//             requestCancelled = WdfRequestIsCanceled(requestContext->OriginalRequest);
//         }
//     }
//
//     if (!requestCancelled)
//     {
//         status = RequestSend(DeviceExtension,
//                              DeviceExtension->ScratchContext.ScratchRequest,
//                              DeviceExtension->IoTarget,
//                              SynchronousSrb ? WDF_REQUEST_SEND_OPTION_SYNCHRONOUS : 0,
//                              NULL);
//
//         // If this is a synchronous request, update the history item immediately, including "normalized" sense data
//         if (SynchronousSrb)
//         {
//             ULONG senseSize;
//
//             // Query the tick count and store in the history
//             KeQueryTickCount(&item->TickCountCompleted);
//
//             // Copy the SRB Status
//             item->SrbStatus = srb->SrbStatus;
//
//             // Determine the amount of valid sense data
//             if (srb->SenseInfoBufferLength >= RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength))
//             {
//                 PSENSE_DATA sense = (PSENSE_DATA)srb->SenseInfoBuffer;
//                 senseSize = RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength) +
//                             sense->AdditionalSenseLength;
//                 senseSize = min(senseSize, sizeof(SENSE_DATA));
//             }
//             else
//             {
//                 senseSize = srb->SenseInfoBufferLength;
//             }
//
//             // Normalize the sense data copy in the history
//             RtlZeroMemory(&(item->NormalizedSenseData), sizeof(SENSE_DATA));
//             RtlCopyMemory(&(item->NormalizedSenseData), srb->SenseInfoBuffer, senseSize);
//         }
//     }
//     else
//     {
//         DeviceExtension->ScratchContext.ScratchSrb->SrbStatus = SRB_STATUS_ABORTED;
//         DeviceExtension->ScratchContext.ScratchSrb->InternalStatus = (ULONG)STATUS_CANCELLED;
//         status = STATUS_CANCELLED;
//     }
//
//     return status;
// }
//
// VOID
// CompressSrbHistoryData(
//       PSRB_HISTORY   RequestHistory
//     )
// /*++
//
// Routine Description:
//
//     compress the SRB history data.
//
// Arguments:
//
//     RequestHistory - SRB history data
//
// Return Value:
//
//     RequestHistory - compressed history data
//
// --*/
// {
//     ULONG i;
//     NT_ASSERT( RequestHistory->UsedHistoryCount == RequestHistory->TotalHistoryCount );
//     ValidateSrbHistoryDataPresumptions(RequestHistory);
//
//     for (i=0; i < RequestHistory->UsedHistoryCount; i++)
//     {
//         // for each item...
//         PSRB_HISTORY_ITEM toMatch = &( RequestHistory->History[i] );
//         // hint: read const qualifiers backwards.  i.e. srbstatus is a const UCHAR
//         // so, "UCHAR const * const x" is read "x is a const pointer to a const UCHAR"
//         // unfortunately, "const UCHAR" is equivalent to "UCHAR const", which causes
//         // people no end of confusion due to its widespread use.
//         UCHAR const srbStatus = toMatch->SrbStatus;
//         UCHAR const sense     = toMatch->NormalizedSenseData.SenseKey;
//         UCHAR const asc       = toMatch->NormalizedSenseData.AdditionalSenseCode;
//         UCHAR const ascq      = toMatch->NormalizedSenseData.AdditionalSenseCodeQualifier;
//         ULONG j;
//
//         // see if there are any at higher indices with identical Sense/ASC/ASCQ
//         for (j = i+1; (toMatch->ClassDriverUse != 0xFF) && (j < RequestHistory->UsedHistoryCount); j++)
//         {
//             PSRB_HISTORY_ITEM found = &( RequestHistory->History[j] );
//             // close enough match?
//             if ((srbStatus == found->SrbStatus) &&
//                 (sense     == found->NormalizedSenseData.SenseKey) &&
//                 (asc       == found->NormalizedSenseData.AdditionalSenseCode) &&
//                 (ascq      == found->NormalizedSenseData.AdditionalSenseCodeQualifier)) {
//
//                 // add the fields to keep reasonable track of delay times.
//                 if (toMatch->MillisecondsDelayOnRetry + found->MillisecondsDelayOnRetry < toMatch->MillisecondsDelayOnRetry) {
//                     toMatch->MillisecondsDelayOnRetry = MAXULONG;
//                 } else {
//                     toMatch->MillisecondsDelayOnRetry += found->MillisecondsDelayOnRetry;
//                 }
//
//                 // this found item cannot contain any compressed entries because
//                 // the first entry with a given set of sense/asc/ascq will always
//                 // either be full (0xFF) or be the only partially-full entry with
//                 // that sense/asc/ascq.
//                 NT_ASSERT(found->ClassDriverUse == 0);
//                 // add the counts so we still know how many retries total
//                 toMatch->ClassDriverUse++;
//
//
//                 // if not the last entry, need to move later entries earlier in the array
//                 if (j != RequestHistory->UsedHistoryCount-1) {
//                     // how many entries remain?
//                     SIZE_T remainingBytes = RequestHistory->UsedHistoryCount - 1 - j;
//                     remainingBytes *= sizeof(SRB_HISTORY_ITEM);
//
//                     // note that MOVE is required due to overlapping entries
//                     RtlMoveMemory(found, found+1, remainingBytes);
//
//                     // Finally, decrement the number of used history count and
//                     // decrement j to rescan the current location again
//                     --RequestHistory->UsedHistoryCount;
//                     --j;
//                 } // end moving of array elements around
//             } // end of close enough match
//         } // end j loop
//     } // end i loop
//
//     // unable to compress duplicate sense/asc/ascq, so just lose the most recent data
//     if (RequestHistory->UsedHistoryCount == RequestHistory->TotalHistoryCount)
//     {
//         PSRB_HISTORY_ITEM item = &( RequestHistory->History[ RequestHistory->TotalHistoryCount-1 ] );
//         RequestHistory->ClassDriverUse[0] += item->ClassDriverUse; // how many did we "lose"?
//         RequestHistory->UsedHistoryCount--;
//     }
//
//     // finally, zero any that are no longer in use
//     NT_ASSERT( RequestHistory->UsedHistoryCount != RequestHistory->TotalHistoryCount);
//     {
//         SIZE_T bytesToZero = RequestHistory->TotalHistoryCount - RequestHistory->UsedHistoryCount;
//         bytesToZero *= sizeof(SRB_HISTORY_ITEM);
//         RtlZeroMemory(&(RequestHistory->History[RequestHistory->UsedHistoryCount]), bytesToZero);
//     }
//
//     ValidateSrbHistoryDataPresumptions(RequestHistory);
//     return;
// }
//
// VOID
// ValidateSrbHistoryDataPresumptions(
//          SRB_HISTORY const * RequestHistory
//     )
// {
// #if DBG
//     // validate that all fully-compressed items are before any non-fully-compressed items of any particular sense/asc/ascq
//     // validate that there is at most one partially-compressed item of any particular sense/asc/ascq
//     // validate that all items of any particular sense/asc/ascq that are uncompressed are at the end
//     // THUS: A(255) A(255) A( 40) A(  0) A(  0) is legal for all types with A as sense/asc/ascq
//     //       A(0)   B(255) A(  0) B( 17) B(  0) is also legal because A/B are different types of error
//
//     ULONG i;
//     for (i = 0; i < RequestHistory->UsedHistoryCount; i++)
//     {
//         SRB_HISTORY_ITEM const * toMatch = &( RequestHistory->History[i] );
//         UCHAR const srbStatus = toMatch->SrbStatus;
//         UCHAR const sense     = toMatch->NormalizedSenseData.SenseKey;
//         UCHAR const asc       = toMatch->NormalizedSenseData.AdditionalSenseCode;
//         UCHAR const ascq      = toMatch->NormalizedSenseData.AdditionalSenseCodeQualifier;
//         ULONG j;
//
//         BOOLEAN foundPartiallyCompressedItem =
//             (toMatch->ClassDriverUse !=    0) &&
//             (toMatch->ClassDriverUse != 0xFF) ;
//         BOOLEAN foundUncompressedItem =
//             (toMatch->ClassDriverUse ==    0) ;
//
//         for (j = i+1; j < RequestHistory->UsedHistoryCount; j++)
//         {
//             SRB_HISTORY_ITEM const * found = &( RequestHistory->History[j] );
//             if ((srbStatus == found->SrbStatus) &&
//                 (sense     == found->NormalizedSenseData.SenseKey) &&
//                 (asc       == found->NormalizedSenseData.AdditionalSenseCode) &&
//                 (ascq      == found->NormalizedSenseData.AdditionalSenseCodeQualifier)
//                 )
//             {
//                 // found a matching type, so validate ordering rules
//                 if (foundUncompressedItem && (found->ClassDriverUse != 0))
//                 {
//                     DbgPrintEx(DPFLTR_CDROM_ID, DPFLTR_ERROR_LEVEL,
//                                "History data has compressed history following uncompressed history "
//                                "for srbstatus/sense/asc/ascq of %02x/%02x/%02x/%02x at indices %d (%08x) and %d (%08x)\n",
//                                srbStatus, sense, asc, ascq,
//                                i,i, j,j
//                                );
//                     NT_ASSERT(FALSE);
//                 }
//                 else if (foundPartiallyCompressedItem && (found->ClassDriverUse == 0xFF))
//                 {
//                     DbgPrintEx(DPFLTR_CDROM_ID, DPFLTR_ERROR_LEVEL,
//                                "History data has fully compressed history following partially compressed history "
//                                "for srbstatus/sense/asc/ascq of %02x/%02x/%02x/%02x at indices %d (%08x) and %d (%08x)\n",
//                                srbStatus, sense, asc, ascq,
//                                i,i, j,j
//                                );
//                     NT_ASSERT(FALSE);
//                 }
//
//                 // update if we have now found partially compressed and/or uncompressed items
//                 if (found->ClassDriverUse == 0)
//                 {
//                     foundUncompressedItem = TRUE;
//                 }
//                 else if (found->ClassDriverUse != 0xFF)
//                 {
//                     foundPartiallyCompressedItem = TRUE;
//                 }
//             } // end match of (toMatch,found)
//         } // end loop j
//     } // end loop i
// #else
//     UNREFERENCED_PARAMETER(RequestHistory);
// #endif
//     return;
// }
//
// VOID
// ScratchBuffer_SetupReadWriteSrb(
//      PCDROM_DEVICE_EXTENSION     DeviceExtension,
//         WDFREQUEST                  OriginalRequest,
//         LARGE_INTEGER               StartingOffset,
//         ULONG                       RequiredLength,
//      UCHAR* DataBuffer,
//         BOOLEAN                     IsReadRequest,
//         BOOLEAN                     UsePartialMdl
//     )
// /*++
//
// Routine Description:
//
//     setup SRB for read/write request.
//
// Arguments:
//
//     DeviceExtension - device extension
//     OriginalRequest - read/write request
//     StartingOffset - read/write starting offset
//     DataBuffer - buffer for read/write
//     IsReadRequest - TRUE (read); FALSE (write)
//
// Return Value:
//
//     none
//
// --*/
// {
//     //NOTE: R/W request not use the ScratchBuffer, instead, it uses the buffer associated with IRP.
//
//     PSCSI_REQUEST_BLOCK srb = DeviceExtension->ScratchContext.ScratchSrb;
//     PCDB                cdb = (PCDB)srb->Cdb;
//     LARGE_INTEGER       logicalBlockAddr;
//     ULONG               numTransferBlocks;
//
//     PIRP                originalIrp = WdfRequestWdmGetIrp(OriginalRequest);
//
//     PIRP                irp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);
//     PIO_STACK_LOCATION  irpStack = NULL;
//
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(DeviceExtension->ScratchContext.ScratchRequest);
//
//     requestContext->OriginalRequest = OriginalRequest;
//
//
//     logicalBlockAddr.QuadPart = Int64ShrlMod32(StartingOffset.QuadPart, DeviceExtension->SectorShift);
//     numTransferBlocks = RequiredLength >> DeviceExtension->SectorShift;
//
//     // set to use the full scratch buffer via the scratch SRB
//     irpStack = IoGetNextIrpStackLocation(irp);
//     irpStack->MajorFunction = IRP_MJ_SCSI;
//     if (IsReadRequest)
//     {
//         irpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN;
//     }
//     else
//     {
//         irpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_OUT;
//     }
//     irpStack->Parameters.Scsi.Srb = srb;
//
//     // prepare the SRB with default values
//     srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//     srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//     srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//     srb->SrbStatus = 0;
//     srb->ScsiStatus = 0;
//     srb->NextSrb = NULL;
//     srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//     srb->SenseInfoBuffer = DeviceExtension->ScratchContext.ScratchSense;
//
//     srb->DataBuffer = DataBuffer;
//     srb->DataTransferLength = RequiredLength;
//
//     srb->QueueSortKey = logicalBlockAddr.LowPart;
//     if (logicalBlockAddr.QuadPart > 0xFFFFFFFF)
//     {
//         //
//         // If the requested LBA is more than max ULONG set the
//         // QueueSortKey to the maximum value, so that these
//         // requests can be added towards the end of the queue.
//         //
//         srb->QueueSortKey = 0xFFFFFFFF;
//     }
//
//     srb->OriginalRequest = irp;
//     srb->TimeOutValue = DeviceExtension->TimeOutValue;
//
//     if (RequestIsRealtimeStreaming(OriginalRequest, IsReadRequest) &&
//         !TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags, FDO_HACK_NO_STREAMING))
//     {
//         if (IsReadRequest)
//         {
//             RtlZeroMemory(&cdb->READ12, sizeof(cdb->READ12));
//             REVERSE_BYTES(&cdb->READ12.LogicalBlock, &logicalBlockAddr.LowPart);
//             REVERSE_BYTES(&cdb->READ12.TransferLength, &numTransferBlocks);
//             cdb->READ12.Streaming = 1;
//             cdb->READ12.OperationCode = SCSIOP_READ12;
//             srb->CdbLength = sizeof(cdb->READ12);
//         }
//         else
//         {
//             RtlZeroMemory(&cdb->WRITE12, sizeof(cdb->WRITE12));
//             REVERSE_BYTES(&cdb->WRITE12.LogicalBlock, &logicalBlockAddr.LowPart);
//             REVERSE_BYTES(&cdb->WRITE12.TransferLength, &numTransferBlocks);
//             cdb->WRITE12.Streaming = 1;
//             cdb->WRITE12.OperationCode = SCSIOP_WRITE12;
//             srb->CdbLength = sizeof(cdb->WRITE12);
//         }
//     }
//     else
//     {
//         RtlZeroMemory(&cdb->CDB10, sizeof(cdb->CDB10));
//         cdb->CDB10.LogicalBlockByte0 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte3;
//         cdb->CDB10.LogicalBlockByte1 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte2;
//         cdb->CDB10.LogicalBlockByte2 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte1;
//         cdb->CDB10.LogicalBlockByte3 = ((PFOUR_BYTE)&logicalBlockAddr.LowPart)->Byte0;
//         cdb->CDB10.TransferBlocksMsb = ((PFOUR_BYTE)&numTransferBlocks)->Byte1;
//         cdb->CDB10.TransferBlocksLsb = ((PFOUR_BYTE)&numTransferBlocks)->Byte0;
//         cdb->CDB10.OperationCode = (IsReadRequest) ? SCSIOP_READ : SCSIOP_WRITE;
//         srb->CdbLength = sizeof(cdb->CDB10);
//     }
//
//     //  Set SRB and IRP flags
//     srb->SrbFlags = DeviceExtension->SrbFlags;
//     if (TEST_FLAG(originalIrp->Flags, IRP_PAGING_IO) ||
//         TEST_FLAG(originalIrp->Flags, IRP_SYNCHRONOUS_PAGING_IO))
//     {
//         SET_FLAG(srb->SrbFlags, SRB_CLASS_FLAGS_PAGING);
//     }
//
//     SET_FLAG(srb->SrbFlags, (IsReadRequest) ? SRB_FLAGS_DATA_IN : SRB_FLAGS_DATA_OUT);
//     SET_FLAG(srb->SrbFlags, SRB_FLAGS_ADAPTER_CACHE_ENABLE);
//
//     //
//     // If the request is not split, we can use the original IRP MDL.  If the
//     // request needs to be split, we need to use a partial MDL.  The partial MDL
//     // is needed because more than one driver might be mapping the same MDL
//     // and this causes problems.
//     //
//     if (UsePartialMdl == FALSE)
//     {
//         irp->MdlAddress = originalIrp->MdlAddress;
//     }
//     else
//     {
//         if (DeviceExtension->ScratchContext.PartialMdlIsBuilt != FALSE)
//         {
//             MmPrepareMdlForReuse(DeviceExtension->ScratchContext.PartialMdl);
//         }
//
//         IoBuildPartialMdl(originalIrp->MdlAddress, DeviceExtension->ScratchContext.PartialMdl, srb->DataBuffer, srb->DataTransferLength);
//         DeviceExtension->ScratchContext.PartialMdlIsBuilt = TRUE;
//         irp->MdlAddress = DeviceExtension->ScratchContext.PartialMdl;
//     }
//
//     //DBGLOGSENDPACKET(Pkt);
//     //HISTORYLOGSENDPACKET(Pkt);
//
//     //
//     // Set the original irp here for SFIO.
//     //
//     srb->SrbExtension = (PVOID)(originalIrp);
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// ScratchBuffer_ExecuteCdbEx(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      WDFREQUEST             OriginalRequest,
//      ULONG                      TransferSize,
//      BOOLEAN                    GetDataFromDevice,
//      PCDB                       Cdb,
//      UCHAR                      OprationLength,
//      ULONG                      TimeoutValue
//     )
// /*++
//
// Routine Description:
//
//     Use Scratch buffer to send the Cdb, check error and retry if necessary.
//
// Arguments:
//
//     DeviceExtension - device context
//     OriginalRequest - original request that requires this CDB operation
//     TransferSize - Data transfer size required
//     GetFromDevice - TRUE if getting data from device.
//     Cdb - SCSI command
//     OprationLength - SCSI command length: 6, 10 or 12
//     TimeoutValue - if > 0, use it as timeout value for command
//                    if 0, use the default device timeout value
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PSCSI_REQUEST_BLOCK srb = DeviceExtension->ScratchContext.ScratchSrb;
//     PCDB                cdb = (PCDB)(srb->Cdb);
//
//     BOOLEAN             shouldRetry = TRUE;
//     ULONG               timesAlreadyRetried = 0;
//     LONGLONG            retryIn100nsUnits = 0;
//
//     PAGED_CODE ();
//
//     while (shouldRetry)
//     {
//         ScratchBuffer_SetupSrb(DeviceExtension, OriginalRequest, TransferSize, GetDataFromDevice);
//
//         // Set up the SRB/CDB
//         RtlCopyMemory(cdb, Cdb, sizeof(CDB));
//
//         srb->CdbLength = OprationLength;
//
//         if (TimeoutValue > 0)
//         {
//             srb->TimeOutValue = TimeoutValue;
//         }
//
//         ScratchBuffer_SendSrb(DeviceExtension, TRUE, NULL);
//
//         if ((DeviceExtension->ScratchContext.ScratchSrb->SrbStatus == SRB_STATUS_ABORTED) &&
//             (DeviceExtension->ScratchContext.ScratchSrb->InternalStatus == STATUS_CANCELLED))
//         {
//             shouldRetry = FALSE;
//             status = STATUS_CANCELLED;
//         }
//         else
//         {
//             shouldRetry = RequestSenseInfoInterpretForScratchBuffer(DeviceExtension,
//                                                                     timesAlreadyRetried,
//                                                                     &status,
//                                                                     &retryIn100nsUnits);
//             if (shouldRetry)
//             {
//                 LARGE_INTEGER t;
//                 t.QuadPart = -retryIn100nsUnits;
//                 timesAlreadyRetried++;
//                 KeDelayExecutionThread(KernelMode, FALSE, &t);
//                 // keep items clean
//                 ScratchBuffer_ResetItems(DeviceExtension, FALSE);
//             }
//         }
//     }
//
//     return status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
