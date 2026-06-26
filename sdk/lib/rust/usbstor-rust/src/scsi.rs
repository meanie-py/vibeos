use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateMdl, IoBuildPartialMdl, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine, PDEVICE_OBJECT, PIO_COMPLETION_ROUTINE, PIO_STACK_LOCATION, PIRP, PMDL};
use core::ffi::c_void;

// C macros / FORCEINLINE helpers not present in the bindgen ddk surface — defined
// locally per the sibling-driver (scsiport/ntoskrnl iofunc) pattern. Faithful to winnt.h /
// wdm.h: PtrToUlong truncates a pointer to ULONG; MmGetMdlVirtualAddress = StartVa + ByteOffset.
#[inline(always)]
unsafe fn PtrToUlong(p: *mut core::ffi::c_void) -> u32 {
    p as usize as u32
}
#[inline(always)]
unsafe fn MmGetMdlVirtualAddress(Mdl: PMDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}

extern "C" {
    fn USBSTOR_QueueResetPipe(FDODeviceExtension: PFDO_DEVICE_EXTENSION);
    fn USBSTOR_QueueResetDevice(FDODeviceExtension: PFDO_DEVICE_EXTENSION);
    fn USBSTOR_QueueTerminateRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
    fn SrbGetCdb(Srb: PSCSI_REQUEST_BLOCK) -> PCDB;
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR, );
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
    fn USBSTOR_QueueNextRequest(DeviceObject: PDEVICE_OBJECT);
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

unsafe fn USBSTOR_SrbStatusToNtStatus(mut Srb: PSCSI_REQUEST_BLOCK) -> NTSTATUS {
    const SRB_STATUS_AUTOSENSE_VALID: ::core::ffi::c_int = 0x80;
    const SRB_STATUS_QUEUE_FROZEN: ::core::ffi::c_int = 0x40;
    const SRB_STATUS_SUCCESS: ::core::ffi::c_int = 0x01;
    const SRB_STATUS_DATA_OVERRUN: ::core::ffi::c_int = 0x12;
    const SRB_STATUS_BAD_FUNCTION: ::core::ffi::c_int = 0x22;
    const SRB_STATUS_BAD_SRB_BLOCK_LENGTH: ::core::ffi::c_int = 0x24;
    const SRB_STATUS_INVALID_LUN: ::core::ffi::c_int = 0x20;
    const SRB_STATUS_INVALID_TARGET_ID: ::core::ffi::c_int = 0x1F;
    const SRB_STATUS_NO_HBA: ::core::ffi::c_int = 0x11;
    const SRB_STATUS_NO_DEVICE: ::core::ffi::c_int = 0x08;
    const SRB_STATUS_TIMEOUT: ::core::ffi::c_int = 0x09;
    const SRB_STATUS_BUS_RESET: ::core::ffi::c_int = 0x0E;
    const SRB_STATUS_COMMAND_TIMEOUT: ::core::ffi::c_int = 0x0B;
    const SRB_STATUS_SELECTION_TIMEOUT: ::core::ffi::c_int = 0x0A;
    const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0u32 as NTSTATUS;
    const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as NTSTATUS;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;

    let mut SrbStatus: UCHAR = 0;
    SrbStatus = ((*Srb).SrbStatus as ::core::ffi::c_int
        & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as UCHAR;
    match SrbStatus as ::core::ffi::c_int {
        SRB_STATUS_SUCCESS => return STATUS_SUCCESS,
        SRB_STATUS_DATA_OVERRUN => return STATUS_BUFFER_OVERFLOW,
        SRB_STATUS_BAD_FUNCTION | SRB_STATUS_BAD_SRB_BLOCK_LENGTH => {
            return STATUS_INVALID_DEVICE_REQUEST;
        }
        SRB_STATUS_INVALID_LUN
        | SRB_STATUS_INVALID_TARGET_ID
        | SRB_STATUS_NO_HBA
        | SRB_STATUS_NO_DEVICE => return STATUS_DEVICE_DOES_NOT_EXIST,
        SRB_STATUS_TIMEOUT => return STATUS_IO_TIMEOUT,
        SRB_STATUS_BUS_RESET | SRB_STATUS_COMMAND_TIMEOUT | SRB_STATUS_SELECTION_TIMEOUT => {
            return STATUS_DEVICE_NOT_CONNECTED;
        }
        _ => return STATUS_IO_DEVICE_ERROR,
    };
}
unsafe fn USBSTOR_IssueBulkOrInterruptRequest(
    mut FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut PipeHandle: USBD_PIPE_HANDLE,
    mut TransferFlags: ULONG,
    mut TransferBufferLength: ULONG,
    mut TransferBuffer: PVOID,
    mut TransferBufferMDL: PMDL,
    mut CompletionRoutine: PIO_COMPLETION_ROUTINE,
) -> NTSTATUS {
    let mut NextStack: PIO_STACK_LOCATION;
    let mut Context: PIRP_CONTEXT = core::ptr::addr_of_mut!((*FDODeviceExtension).CurrentIrpContext);

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*Context).Urb) as PVOID,
        core::mem::size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>(),
    );

    const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003;

    (*Context).Urb.__bindgen_anon_1.UrbHeader.as_mut().Length =
        core::mem::size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>() as USHORT;
    (*Context).Urb.__bindgen_anon_1.UrbHeader.as_mut().Function = URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER as USHORT;

    (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_mut().PipeHandle = PipeHandle;
    (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_mut().TransferFlags = TransferFlags;
    (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_mut().TransferBufferLength = TransferBufferLength;
    (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_mut().TransferBuffer = TransferBuffer;
    (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_mut().TransferBufferMDL = TransferBufferMDL;

    NextStack = IoGetNextIrpStackLocation(Irp);
    (*NextStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as UCHAR;
    (*NextStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB as ULONG;
    (*NextStack).Parameters.Others.Argument1 = core::ptr::addr_of_mut!((*Context).Urb) as PVOID;

    IoSetCompletionRoutine(
        Irp,
        CompletionRoutine,
        FDODeviceExtension as PVOID,
        1 as BOOLEAN,
        1 as BOOLEAN,
        1 as BOOLEAN,
    );

    return IoCallDriver((*FDODeviceExtension).LowerDeviceObject, Irp);
}
unsafe fn USBSTOR_IsCSWValid(mut Context: PIRP_CONTEXT) -> BOOLEAN {
    if (*Context).__bindgen_anon_1.csw.as_ref().Signature != CSW_SIGNATURE as core::ffi::c_ulong {
        return 0 as BOOLEAN;
    }

    if (*Context).__bindgen_anon_1.csw.as_ref().Tag != PtrToUlong((*Context).Irp as *mut core::ffi::c_void) {
        return 0 as BOOLEAN;
    }

    return 1 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_CSWCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Ctx: PVOID,
) -> NTSTATUS {

    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
    const STATUS_CONTINUE_COMPLETION: NTSTATUS = 0x00000000;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;
    const USBD_STATUS_STALL_PID: USBD_STATUS = 0xc0000004u32 as USBD_STATUS;
    const CSW_STATUS_PHASE_ERROR: i32 = 0x02;
    const CSW_STATUS_COMMAND_PASSED: i32 = 0x00;
    const CSW_STATUS_COMMAND_FAILED: i32 = 0x01;
    const SRB_STATUS_AUTOSENSE_VALID: i32 = 0x80;
    const SRB_STATUS_ERROR: i32 = 0x04;
    const SRB_STATUS_BUS_RESET: i32 = 0x0E;
    const SRB_FLAGS_DISABLE_AUTOSENSE: u32 = 0x00000020;

    let mut Context: PIRP_CONTEXT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Request: PSCSI_REQUEST_BLOCK;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    PDODeviceExtension = (*(*IoStack).DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    FDODeviceExtension = Ctx as PFDO_DEVICE_EXTENSION;
    Context = core::ptr::addr_of_mut!((*FDODeviceExtension).CurrentIrpContext) as PIRP_CONTEXT;
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    if Request.is_null() {
        RtlAssert(
            b"Request\0".as_ptr() as PVOID,
            b"scsi.c\0".as_ptr() as PVOID,
            147 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // first check for Irp errors
    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        if ((*Context).Urb.__bindgen_anon_1.UrbHeader.as_ref().Status as core::ffi::c_ulong & 0xfffffff)
            == (USBD_STATUS_STALL_PID as USBD_STATUS as core::ffi::c_ulong & 0xfffffff)
        {
            if (*Context).StallRetryCount < 2 {
                (*Context).StallRetryCount = (*Context).StallRetryCount.wrapping_add(1);

                // clear stall and resend cbw
                USBSTOR_QueueResetPipe(FDODeviceExtension);

                return STATUS_MORE_PROCESSING_REQUIRED;
            }
        }

        // goto ResetRecovery;
        Request = (*FDODeviceExtension).ActiveSrb;
        (*IoStack).Parameters.Scsi.Srb = Request as *mut _;
        (*Irp).IoStatus.Information = 0 as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
        (*Request).SrbStatus = SRB_STATUS_BUS_RESET as UCHAR;

        USBSTOR_QueueTerminateRequest((*PDODeviceExtension).LowerDeviceObject, Irp);
        USBSTOR_QueueResetDevice(FDODeviceExtension);

        return STATUS_CONTINUE_COMPLETION;
    }

    // now check the CSW packet validity
    if USBSTOR_IsCSWValid(Context) == 0
        || (*Context).__bindgen_anon_1.csw.as_ref().Status as i32 == CSW_STATUS_PHASE_ERROR
    {
        // goto ResetRecovery;
        Request = (*FDODeviceExtension).ActiveSrb;
        (*IoStack).Parameters.Scsi.Srb = Request as *mut _;
        (*Irp).IoStatus.Information = 0 as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
        (*Request).SrbStatus = SRB_STATUS_BUS_RESET as UCHAR;

        USBSTOR_QueueTerminateRequest((*PDODeviceExtension).LowerDeviceObject, Irp);
        USBSTOR_QueueResetDevice(FDODeviceExtension);

        return STATUS_CONTINUE_COMPLETION;
    }

    // finally check for CSW errors
    if (*Context).__bindgen_anon_1.csw.as_ref().Status as i32 == CSW_STATUS_COMMAND_PASSED {
        // should happen only when a sense request was sent
        if Request != (*FDODeviceExtension).ActiveSrb {
            if (*IoStack).Parameters.Scsi.Srb != core::ptr::addr_of_mut!((*Context).SenseSrb) as *mut _ {
                RtlAssert(
                    b"IoStack->Parameters.Scsi.Srb == &Context->SenseSrb\0".as_ptr() as PVOID,
                    b"scsi.c\0".as_ptr() as PVOID,
                    184 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            }
            (*(*FDODeviceExtension).ActiveSrb).SenseInfoBufferLength =
                (*Request).DataTransferLength as UCHAR;
            Request = (*FDODeviceExtension).ActiveSrb;
            (*IoStack).Parameters.Scsi.Srb = Request as *mut _;
            (*Request).SrbStatus =
                ((*Request).SrbStatus as i32 | SRB_STATUS_AUTOSENSE_VALID) as UCHAR;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = USBSTOR_SrbStatusToNtStatus(Request);
    } else if (*Context).__bindgen_anon_1.csw.as_ref().Status as i32 == CSW_STATUS_COMMAND_FAILED {
        // the command is correct but with failed status - issue request sense

        if (*FDODeviceExtension).ActiveSrb != Request {
            RtlAssert(
                b"FDODeviceExtension->ActiveSrb == Request\0".as_ptr() as PVOID,
                b"scsi.c\0".as_ptr() as PVOID,
                198 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }

        // setting a generic error status, additional information
        // should be read by higher-level driver from SenseInfoBuffer
        (*Request).SrbStatus = SRB_STATUS_ERROR as UCHAR;
        (*Request).ScsiStatus = 2 as UCHAR;
        (*Request).DataTransferLength = 0 as ULONG;

        if (*Request).SrbFlags as core::ffi::c_ulong
            & SRB_FLAGS_DISABLE_AUTOSENSE as core::ffi::c_ulong
            == 0
            && (*Request).SenseInfoBufferLength as i32 != 0
            && !(*Request).SenseInfoBuffer.is_null()
        {
            USBSTOR_IssueRequestSense(FDODeviceExtension, Irp);
            return STATUS_MORE_PROCESSING_REQUIRED;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
    }

    (*Irp).IoStatus.Information = (*Request).DataTransferLength as _;

    // terminate current request
    USBSTOR_QueueTerminateRequest((*PDODeviceExtension).LowerDeviceObject, Irp);
    USBSTOR_QueueNextRequest((*PDODeviceExtension).LowerDeviceObject);

    return STATUS_CONTINUE_COMPLETION;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_SendCSWRequest(
    mut FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    return USBSTOR_IssueBulkOrInterruptRequest(
        FDODeviceExtension,
        Irp,
        (*((&raw mut (*(*FDODeviceExtension).InterfaceInformation).Pipes)
            as *mut USBD_PIPE_INFORMATION)
            .offset((*FDODeviceExtension).BulkInPipeIndex as isize))
        .PipeHandle,
        USBD_TRANSFER_DIRECTION_IN as ULONG,
        core::mem::size_of::<CSW>() as ULONG,
        (*FDODeviceExtension).CurrentIrpContext.__bindgen_anon_1.csw.as_mut() as *mut CSW as PVOID,
        core::ptr::null_mut(),
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(
            USBSTOR_CSWCompletionRoutine as usize,
        ),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_DataCompletionRoutine(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut Ctx: PVOID,
) -> NTSTATUS {
    const SRB_STATUS_DATA_OVERRUN: UCHAR = 0x12;
    const SRB_STATUS_SUCCESS: UCHAR = 0x01;
    const SRB_STATUS_BUS_RESET: UCHAR = 0x0E;
    const USBD_STATUS_STALL_PID: USBD_STATUS = 0xc0000004u32 as USBD_STATUS;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;
    const STATUS_CONTINUE_COMPLETION: NTSTATUS = STATUS_SUCCESS;
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;

    let mut Context: PIRP_CONTEXT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Request: PSCSI_REQUEST_BLOCK;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;

    FDODeviceExtension = Ctx as PFDO_DEVICE_EXTENSION;
    Context = core::ptr::addr_of_mut!((*FDODeviceExtension).CurrentIrpContext) as PIRP_CONTEXT;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    PDODeviceExtension = (*(*IoStack).DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if Request == (*FDODeviceExtension).ActiveSrb
        && (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().TransferBufferMDL != (*Irp).MdlAddress
    {
        IoFreeMdl((*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().TransferBufferMDL);
    }

    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        if (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().TransferBufferLength
            < (*Request).DataTransferLength
        {
            (*Request).SrbStatus = SRB_STATUS_DATA_OVERRUN;
        } else {
            (*Request).SrbStatus = SRB_STATUS_SUCCESS;
        }

        (*Request).DataTransferLength =
            (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().TransferBufferLength;
        USBSTOR_SendCSWRequest(FDODeviceExtension, Irp);
    } else if ((*Context).Urb.__bindgen_anon_1.UrbHeader.as_ref().Status as USBD_STATUS) & 0xfffffff
        == (USBD_STATUS_STALL_PID) & 0xfffffff
    {
        (*Context).StallRetryCount = (*Context).StallRetryCount.wrapping_add(1);

        (*Request).SrbStatus = SRB_STATUS_DATA_OVERRUN;
        (*Request).DataTransferLength =
            (*Context).Urb.__bindgen_anon_1.UrbBulkOrInterruptTransfer.as_ref().TransferBufferLength;

        USBSTOR_QueueResetPipe(FDODeviceExtension);
    } else {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
        (*Request).SrbStatus = SRB_STATUS_BUS_RESET;

        USBSTOR_QueueTerminateRequest((*PDODeviceExtension).LowerDeviceObject, Irp);
        USBSTOR_QueueResetDevice(FDODeviceExtension);

        return STATUS_CONTINUE_COMPLETION;
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_CBWCompletionRoutine(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut Ctx: PVOID,
) -> NTSTATUS {

    const SRB_STATUS_SUCCESS: UCHAR = 0x01;
    const SRB_STATUS_BUS_RESET: UCHAR = 0x0E;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;
    const STATUS_CONTINUE_COMPLETION: NTSTATUS = STATUS_SUCCESS;
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
    const SRB_FLAGS_DATA_IN: u32 = 0x00000040;
    const SRB_FLAGS_DATA_OUT: u32 = 0x00000080;
    const SRB_FLAGS_UNSPECIFIED_DIRECTION: u32 = SRB_FLAGS_DATA_IN | SRB_FLAGS_DATA_OUT;

    let mut current_block: u64;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Request: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PipeHandle: USBD_PIPE_HANDLE = core::ptr::null_mut();
    let mut TransferFlags: ULONG = 0;
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut TransferBuffer: PVOID = core::ptr::null_mut();

    FDODeviceExtension = Ctx as PFDO_DEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    PDODeviceExtension = (*(*IoStack).DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if (*Irp).IoStatus.__bindgen_anon_1.Status >= 0 as core::ffi::c_long {
        if (*Irp).MdlAddress.is_null() && Request == (*FDODeviceExtension).ActiveSrb {
            (*Request).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
            USBSTOR_SendCSWRequest(FDODeviceExtension, Irp);
            return STATUS_MORE_PROCESSING_REQUIRED;
        }

        if (*Request).SrbFlags as core::ffi::c_ulong
            & SRB_FLAGS_UNSPECIFIED_DIRECTION as core::ffi::c_ulong
            == SRB_FLAGS_DATA_IN as core::ffi::c_ulong
        {
            PipeHandle = (*(&raw mut (*(*FDODeviceExtension).InterfaceInformation).Pipes
                as *mut USBD_PIPE_INFORMATION)
                .offset((*FDODeviceExtension).BulkInPipeIndex as isize))
            .PipeHandle;
            TransferFlags = (USBD_TRANSFER_DIRECTION_IN | USBD_SHORT_TRANSFER_OK) as ULONG;
            current_block = 17478428563724192186;
        } else if (*Request).SrbFlags as core::ffi::c_ulong
            & SRB_FLAGS_UNSPECIFIED_DIRECTION as core::ffi::c_ulong
            == SRB_FLAGS_DATA_OUT as core::ffi::c_ulong
        {
            PipeHandle = (*(&raw mut (*(*FDODeviceExtension).InterfaceInformation).Pipes
                as *mut USBD_PIPE_INFORMATION)
                .offset((*FDODeviceExtension).BulkOutPipeIndex as isize))
            .PipeHandle;
            TransferFlags = USBD_TRANSFER_DIRECTION_OUT as ULONG;
            current_block = 17478428563724192186;
        } else {
            // DPRINT1("Warning: shouldn't be here\n");
            current_block = 110718756212904686;
        }

        match current_block {
            110718756212904686 => {}
            _ => {
                if Request == (*FDODeviceExtension).ActiveSrb {
                    if MmGetMdlVirtualAddress((*Irp).MdlAddress) == (*Request).DataBuffer {
                        Mdl = (*Irp).MdlAddress as PMDL;
                    } else {
                        Mdl = IoAllocateMdl(
                            (*Request).DataBuffer,
                            (*Request).DataTransferLength,
                            0 as BOOLEAN,
                            0 as BOOLEAN,
                            core::ptr::null_mut(),
                        );
                        if !Mdl.is_null() {
                            IoBuildPartialMdl(
                                (*Irp).MdlAddress as PMDL,
                                Mdl,
                                (*Request).DataBuffer,
                                (*Request).DataTransferLength,
                            );
                        }
                    }
                    if Mdl.is_null() {
                        // DPRINT1("USBSTOR_CBWCompletionRoutine: Mdl - %p\n", Mdl);
                        current_block = 110718756212904686;
                    } else {
                        current_block = 9007357115414505193;
                    }
                } else {
                    // ASSERT(Request->DataBuffer);
                    TransferBuffer = (*Request).DataBuffer;
                    current_block = 9007357115414505193;
                }

                match current_block {
                    110718756212904686 => {}
                    _ => {
                        USBSTOR_IssueBulkOrInterruptRequest(
                            FDODeviceExtension,
                            Irp,
                            PipeHandle,
                            TransferFlags,
                            (*Request).DataTransferLength,
                            TransferBuffer,
                            Mdl,
                            core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(
                                USBSTOR_DataCompletionRoutine as usize,
                            ),
                        );
                        return STATUS_MORE_PROCESSING_REQUIRED;
                    }
                }
            }
        }
    }

    Request = (*FDODeviceExtension).ActiveSrb;
    (*IoStack).Parameters.Scsi.Srb = Request as *mut _;
    (*Irp).IoStatus.Information = 0 as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_DEVICE_ERROR;
    (*Request).SrbStatus = SRB_STATUS_BUS_RESET as UCHAR;

    USBSTOR_QueueTerminateRequest((*PDODeviceExtension).LowerDeviceObject, Irp);
    USBSTOR_QueueResetDevice(FDODeviceExtension);

    return STATUS_CONTINUE_COMPLETION;
}

#[no_mangle]
pub unsafe extern "C" fn DumpCBW(mut Block: PUCHAR) {
}
unsafe fn USBSTOR_SendCBWRequest(
    mut FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    const SRB_FLAGS_DATA_IN: i32 = 0x00000040;
    const SRB_FLAGS_DATA_OUT: i32 = 0x00000080;
    const SRB_FLAGS_UNSPECIFIED_DIRECTION: i32 = SRB_FLAGS_DATA_IN | SRB_FLAGS_DATA_OUT;

    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Request: PSCSI_REQUEST_BLOCK;
    let mut Context: PIRP_CONTEXT = &mut (*FDODeviceExtension).CurrentIrpContext;

    RtlZeroMemory(
        (*Context).__bindgen_anon_1.cbw.as_mut() as *mut CBW as PVOID,
        core::mem::size_of::<CBW>() as _,
    );
    RtlZeroMemory(
        &mut (*Context).Urb as *mut _ as PVOID,
        core::mem::size_of::<URB>() as _,
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    PDODeviceExtension = (*(*IoStack).DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    (*Context).__bindgen_anon_1.cbw.as_mut().Signature = CBW_SIGNATURE as ULONG;
    (*Context).__bindgen_anon_1.cbw.as_mut().Tag = PtrToUlong(Irp as *mut core::ffi::c_void) as ULONG;
    (*Context).__bindgen_anon_1.cbw.as_mut().DataTransferLength = (*Request).DataTransferLength;
    (*Context).__bindgen_anon_1.cbw.as_mut().Flags = (((*Request).SrbFlags as UCHAR as i32
        & SRB_FLAGS_UNSPECIFIED_DIRECTION)
        << 1) as UCHAR;

    (*Context).__bindgen_anon_1.cbw.as_mut().LUN = ((*PDODeviceExtension).LUN as i32 & 0x0F) as UCHAR;
    (*Context).__bindgen_anon_1.cbw.as_mut().CommandBlockLength = (*Request).CdbLength;

    RtlCopyMemory(
        &mut (*Context).__bindgen_anon_1.cbw.as_mut().CommandBlock as *mut _ as PVOID,
        &mut (*Request).Cdb as *mut UCHAR as PVOID,
        (*Request).CdbLength as _,
    );

    DumpCBW((*Context).__bindgen_anon_1.cbw.as_mut() as *mut CBW as PUCHAR);

    (*Context).Irp = Irp;
    (*Context).StallRetryCount = 0;

    return USBSTOR_IssueBulkOrInterruptRequest(
        FDODeviceExtension,
        Irp,
        (*(&mut (*(*FDODeviceExtension).InterfaceInformation).Pipes
            as *mut USBD_PIPE_INFORMATION)
            .offset((*FDODeviceExtension).BulkOutPipeIndex as isize))
        .PipeHandle,
        USBD_TRANSFER_DIRECTION_OUT as ULONG,
        core::mem::size_of::<CBW>() as ULONG,
        (*Context).__bindgen_anon_1.cbw.as_mut() as *mut CBW as PVOID,
        core::ptr::null_mut(),
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(
            USBSTOR_CBWCompletionRoutine as usize,
        ),
    );
}
unsafe fn USBSTOR_IssueRequestSense(
    FDODeviceExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const SRB_FUNCTION_EXECUTE_SCSI: i32 = 0x00;
    const SRB_FLAGS_DATA_IN: u32 = 0x00000040;
    const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x00000100;
    const SRB_FLAGS_DISABLE_AUTOSENSE: u32 = 0x00000020;

    let mut IoStack: PIO_STACK_LOCATION;
    let CurrentSrb: PSCSI_REQUEST_BLOCK;
    let SenseSrb: PSCSI_REQUEST_BLOCK;

    CurrentSrb = (*FDODeviceExtension).ActiveSrb;
    SenseSrb = core::ptr::addr_of_mut!((*FDODeviceExtension).CurrentIrpContext.SenseSrb)
        as PSCSI_REQUEST_BLOCK;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    (*IoStack).Parameters.Scsi.Srb = SenseSrb as *mut _;

    RtlZeroMemory(
        SenseSrb as *mut core::ffi::c_void,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
    );

    (*SenseSrb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*SenseSrb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
    (*SenseSrb).CdbLength = CDB6GENERIC_LENGTH as UCHAR;
    (*SenseSrb).SrbFlags =
        (SRB_FLAGS_DATA_IN | SRB_FLAGS_NO_QUEUE_FREEZE | SRB_FLAGS_DISABLE_AUTOSENSE) as ULONG;

    if (*CurrentSrb).SenseInfoBufferLength == 0 {
        RtlAssert(
            b"CurrentSrb->SenseInfoBufferLength\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/scsi.c\0"
                .as_ptr() as *const i8 as PVOID,
            528 as ULONG,
            core::ptr::null_mut::<CHAR>() as PCHAR,
        );
    }

    if (*CurrentSrb).SenseInfoBuffer.is_null() {
        RtlAssert(
            b"CurrentSrb->SenseInfoBuffer\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/scsi.c\0"
                .as_ptr() as *const i8 as PVOID,
            529 as ULONG,
            core::ptr::null_mut::<CHAR>() as PCHAR,
        );
    }

    (*SenseSrb).DataTransferLength = (*CurrentSrb).SenseInfoBufferLength as ULONG;
    (*SenseSrb).DataBuffer = (*CurrentSrb).SenseInfoBuffer;

    (*SrbGetCdb(SenseSrb)).CDB6GENERIC.as_mut().OperationCode = SCSIOP_REQUEST_SENSE as UCHAR;
    (*SrbGetCdb(SenseSrb)).AsByte.as_mut()[4 as usize] = (*CurrentSrb).SenseInfoBufferLength as UCHAR;

    return USBSTOR_SendCBWRequest(FDODeviceExtension, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_HandleExecuteSCSI(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Request: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if !((*PDODeviceExtension).Common.IsFDO as i32 == 0 as i32) {
        RtlAssert(
            b"PDODeviceExtension->Common.IsFDO == FALSE\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/scsi.c\0"
                .as_ptr() as *const i8 as PVOID,
            550 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    if (*SrbGetCdb(Request)).CDB10.as_ref().LogicalUnitNumber() as i32
        != (*PDODeviceExtension).LUN as i32
    {
        if DbgPrint(
            b"(%s:%d) USBSTOR_HandleExecuteSCSI: CDB LUN %lu != PDO LUN %lu (using CBW LUN)\n\0"
                .as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/scsi.c\0"
                .as_ptr() as *const i8,
            570 as i32,
            (*SrbGetCdb(Request)).CDB10.as_ref().LogicalUnitNumber() as ULONG,
            (*PDODeviceExtension).LUN as ULONG,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/scsi.c\0"
                    .as_ptr() as *const i8,
                570 as i32,
            );
        }
    }

    return USBSTOR_SendCBWRequest(
        (*(*PDODeviceExtension).LowerDeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION,
        Irp,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Storage Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     USB block storage device driver.
//  * COPYRIGHT:   2005-2006 James Tabor
//  *              2011-2012 Michael Martin (michael.martin@reactos.org)
//  *              2011-2013 Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              2017 Vadim Galyant
//  *              2019 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// static
// NTSTATUS
// USBSTOR_SrbStatusToNtStatus(
//     IN PSCSI_REQUEST_BLOCK Srb)
// {
//     UCHAR SrbStatus;
//
//     SrbStatus = SRB_STATUS(Srb->SrbStatus);
//
//     switch (SrbStatus)
//     {
//         case SRB_STATUS_SUCCESS:
//             return STATUS_SUCCESS;
//
//         case SRB_STATUS_DATA_OVERRUN:
//             return STATUS_BUFFER_OVERFLOW;
//
//         case SRB_STATUS_BAD_FUNCTION:
//         case SRB_STATUS_BAD_SRB_BLOCK_LENGTH:
//             return STATUS_INVALID_DEVICE_REQUEST;
//
//         case SRB_STATUS_INVALID_LUN:
//         case SRB_STATUS_INVALID_TARGET_ID:
//         case SRB_STATUS_NO_HBA:
//         case SRB_STATUS_NO_DEVICE:
//             return STATUS_DEVICE_DOES_NOT_EXIST;
//
//         case SRB_STATUS_TIMEOUT:
//             return STATUS_IO_TIMEOUT;
//
//         case SRB_STATUS_BUS_RESET:
//         case SRB_STATUS_COMMAND_TIMEOUT:
//         case SRB_STATUS_SELECTION_TIMEOUT:
//             return STATUS_DEVICE_NOT_CONNECTED;
//
//         default:
//             return STATUS_IO_DEVICE_ERROR;
//     }
// }
//
// static
// NTSTATUS
// USBSTOR_IssueBulkOrInterruptRequest(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PIRP Irp,
//     IN USBD_PIPE_HANDLE PipeHandle,
//     IN ULONG TransferFlags,
//     IN ULONG TransferBufferLength,
//     IN PVOID TransferBuffer,
//     IN PMDL TransferBufferMDL,
//     IN PIO_COMPLETION_ROUTINE CompletionRoutine)
// {
//     PIO_STACK_LOCATION NextStack;
//     PIRP_CONTEXT Context = &FDODeviceExtension->CurrentIrpContext;
//
//     RtlZeroMemory(&Context->Urb, sizeof(struct _URB_BULK_OR_INTERRUPT_TRANSFER));
//
//     Context->Urb.UrbHeader.Length = sizeof(struct _URB_BULK_OR_INTERRUPT_TRANSFER);
//     Context->Urb.UrbHeader.Function = URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER;
//
//     Context->Urb.UrbBulkOrInterruptTransfer.PipeHandle = PipeHandle;
//     Context->Urb.UrbBulkOrInterruptTransfer.TransferFlags = TransferFlags;
//     Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferLength = TransferBufferLength;
//     Context->Urb.UrbBulkOrInterruptTransfer.TransferBuffer = TransferBuffer;
//     Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferMDL = TransferBufferMDL;
//
//     NextStack = IoGetNextIrpStackLocation(Irp);
//     NextStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     NextStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//     NextStack->Parameters.Others.Argument1 = &Context->Urb;
//
//     IoSetCompletionRoutine(Irp,
//                            CompletionRoutine,
//                            FDODeviceExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     return IoCallDriver(FDODeviceExtension->LowerDeviceObject, Irp);
// }
//
// static
// BOOLEAN
// USBSTOR_IsCSWValid(
//     PIRP_CONTEXT Context)
// {
//     if (Context->csw.Signature != CSW_SIGNATURE)
//     {
//         DPRINT1("[USBSTOR] Expected Signature %x but got %x\n", CSW_SIGNATURE, Context->csw.Signature);
//         return FALSE;
//     }
//
//     if (Context->csw.Tag != PtrToUlong(Context->Irp))
//     {
//         DPRINT1("[USBSTOR] Expected Tag %Ix but got %x\n", PtrToUlong(Context->Irp), Context->csw.Tag);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static
// NTSTATUS
// USBSTOR_IssueRequestSense(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PIRP Irp);
//
// IO_COMPLETION_ROUTINE USBSTOR_CSWCompletionRoutine;
//
// NTSTATUS
// NTAPI
// USBSTOR_CSWCompletionRoutine(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Ctx)
// {
//     PIRP_CONTEXT Context;
//     PIO_STACK_LOCATION IoStack;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PSCSI_REQUEST_BLOCK Request;
//
//     DPRINT("USBSTOR_CSWCompletionRoutine Irp %p Ctx %p Status %x\n", Irp, Ctx, Irp->IoStatus.Status);
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)Ctx;
//     Context = &FDODeviceExtension->CurrentIrpContext;
//     Request = IoStack->Parameters.Scsi.Srb;
//     ASSERT(Request);
//
//     // first check for Irp errors
//     if (!NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         if (USBD_STATUS(Context->Urb.UrbHeader.Status) == USBD_STATUS(USBD_STATUS_STALL_PID))
//         {
//             if (Context->StallRetryCount < 2)
//             {
//                 ++Context->StallRetryCount;
//
//                 // clear stall and resend cbw
//                 USBSTOR_QueueResetPipe(FDODeviceExtension);
//
//                 return STATUS_MORE_PROCESSING_REQUIRED;
//             }
//         }
//         else
//         {
//             DPRINT1("USBSTOR_CSWCompletionRoutine: Urb.Hdr.Status - %x\n", Context->Urb.UrbHeader.Status);
//         }
//
//         goto ResetRecovery;
//     }
//
//     // now check the CSW packet validity
//     if (!USBSTOR_IsCSWValid(Context) || Context->csw.Status == CSW_STATUS_PHASE_ERROR)
//     {
//         goto ResetRecovery;
//     }
//
//     // finally check for CSW errors
//     if (Context->csw.Status == CSW_STATUS_COMMAND_PASSED)
//     {
//         // should happen only when a sense request was sent
//         if (Request != FDODeviceExtension->ActiveSrb)
//         {
//             ASSERT(IoStack->Parameters.Scsi.Srb == &Context->SenseSrb);
//             FDODeviceExtension->ActiveSrb->SenseInfoBufferLength = Request->DataTransferLength;
//             Request = FDODeviceExtension->ActiveSrb;
//             IoStack->Parameters.Scsi.Srb = Request;
//             Request->SrbStatus |= SRB_STATUS_AUTOSENSE_VALID;
//         }
//
//         Irp->IoStatus.Status = USBSTOR_SrbStatusToNtStatus(Request);
//     }
//     else if (Context->csw.Status == CSW_STATUS_COMMAND_FAILED)
//     {
//         // the command is correct but with failed status - issue request sense
//         DPRINT("USBSTOR_CSWCompletionRoutine: CSW_STATUS_COMMAND_FAILED\n");
//
//         ASSERT(FDODeviceExtension->ActiveSrb == Request);
//
//         // setting a generic error status, additional information
//         // should be read by higher-level driver from SenseInfoBuffer
//         Request->SrbStatus = SRB_STATUS_ERROR;
//         Request->ScsiStatus = 2;
//         Request->DataTransferLength = 0;
//
//         DPRINT("Flags: %x SBL: %x, buf: %p\n", Request->SrbFlags, Request->SenseInfoBufferLength, Request->SenseInfoBuffer);
//
//         if (!(Request->SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE) &&
//               Request->SenseInfoBufferLength &&
//               Request->SenseInfoBuffer)
//         {
//             USBSTOR_IssueRequestSense(FDODeviceExtension, Irp);
//             return STATUS_MORE_PROCESSING_REQUIRED;
//         }
//
//         Irp->IoStatus.Status = STATUS_IO_DEVICE_ERROR;
//     }
//
//     Irp->IoStatus.Information = Request->DataTransferLength;
//
//     // terminate current request
//     USBSTOR_QueueTerminateRequest(PDODeviceExtension->LowerDeviceObject, Irp);
//     USBSTOR_QueueNextRequest(PDODeviceExtension->LowerDeviceObject);
//
//     return STATUS_CONTINUE_COMPLETION;
//
// ResetRecovery:
//
//     Request = FDODeviceExtension->ActiveSrb;
//     IoStack->Parameters.Scsi.Srb = Request;
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = STATUS_IO_DEVICE_ERROR;
//     Request->SrbStatus = SRB_STATUS_BUS_RESET;
//
//     USBSTOR_QueueTerminateRequest(PDODeviceExtension->LowerDeviceObject, Irp);
//     USBSTOR_QueueResetDevice(FDODeviceExtension);
//
//     return STATUS_CONTINUE_COMPLETION;
// }
//
// NTSTATUS
// USBSTOR_SendCSWRequest(
//     PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     PIRP Irp)
// {
//     return USBSTOR_IssueBulkOrInterruptRequest(FDODeviceExtension,
//                                                Irp,
//                                                FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkInPipeIndex].PipeHandle,
//                                                USBD_TRANSFER_DIRECTION_IN,
//                                                sizeof(CSW),
//                                                &FDODeviceExtension->CurrentIrpContext.csw,
//                                                NULL,
//                                                USBSTOR_CSWCompletionRoutine);
// }
//
// IO_COMPLETION_ROUTINE USBSTOR_DataCompletionRoutine;
//
// NTSTATUS
// NTAPI
// USBSTOR_DataCompletionRoutine(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Ctx)
// {
//     PIRP_CONTEXT Context;
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//
//     DPRINT("USBSTOR_DataCompletionRoutine Irp %p Ctx %p Status %x\n", Irp, Ctx, Irp->IoStatus.Status);
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)Ctx;
//     Context = &FDODeviceExtension->CurrentIrpContext;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = IoStack->Parameters.Scsi.Srb;
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//
//     // for Sense Request a partial MDL was already freed (if existed)
//     if (Request == FDODeviceExtension->ActiveSrb &&
//         Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferMDL != Irp->MdlAddress)
//     {
//         IoFreeMdl(Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferMDL);
//     }
//
//     if (NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         if (Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferLength < Request->DataTransferLength)
//         {
//             Request->SrbStatus = SRB_STATUS_DATA_OVERRUN;
//         }
//         else
//         {
//             Request->SrbStatus = SRB_STATUS_SUCCESS;
//         }
//
//         Request->DataTransferLength = Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferLength;
//         USBSTOR_SendCSWRequest(FDODeviceExtension, Irp);
//     }
//     else if (USBD_STATUS(Context->Urb.UrbHeader.Status) == USBD_STATUS(USBD_STATUS_STALL_PID))
//     {
//         ++Context->StallRetryCount;
//
//         Request->SrbStatus = SRB_STATUS_DATA_OVERRUN;
//         Request->DataTransferLength = Context->Urb.UrbBulkOrInterruptTransfer.TransferBufferLength;
//
//         // clear stall and resend cbw
//         USBSTOR_QueueResetPipe(FDODeviceExtension);
//     }
//     else
//     {
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = STATUS_IO_DEVICE_ERROR;
//         Request->SrbStatus = SRB_STATUS_BUS_RESET;
//
//         USBSTOR_QueueTerminateRequest(PDODeviceExtension->LowerDeviceObject, Irp);
//         USBSTOR_QueueResetDevice(FDODeviceExtension);
//
//         return STATUS_CONTINUE_COMPLETION;
//     }
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// IO_COMPLETION_ROUTINE USBSTOR_CBWCompletionRoutine;
//
// NTSTATUS
// NTAPI
// USBSTOR_CBWCompletionRoutine(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Ctx)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     USBD_PIPE_HANDLE PipeHandle;
//     ULONG TransferFlags;
//     PMDL Mdl = NULL;
//     PVOID TransferBuffer = NULL;
//
//     DPRINT("USBSTOR_CBWCompletionRoutine Irp %p Ctx %p Status %x\n", Irp, Ctx, Irp->IoStatus.Status);
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)Ctx;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = IoStack->Parameters.Scsi.Srb;
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//
//     if (!NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         goto ResetRecovery;
//     }
//
//     // a request without the buffer AND not a sense request
//     // for a sense request we provide just a TransferBuffer, an Mdl will be allocated by usbport (see below)
//     if (!Irp->MdlAddress && Request == FDODeviceExtension->ActiveSrb)
//     {
//         Request->SrbStatus = SRB_STATUS_SUCCESS;
//         USBSTOR_SendCSWRequest(FDODeviceExtension, Irp);
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
//     // a request with the data buffer
//
//     if ((Request->SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION) == SRB_FLAGS_DATA_IN)
//     {
//         PipeHandle = FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkInPipeIndex].PipeHandle;
//         TransferFlags = USBD_TRANSFER_DIRECTION_IN | USBD_SHORT_TRANSFER_OK;
//     }
//     else if ((Request->SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION) == SRB_FLAGS_DATA_OUT)
//     {
//         PipeHandle = FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkOutPipeIndex].PipeHandle;
//         TransferFlags = USBD_TRANSFER_DIRECTION_OUT;
//     }
//     else
//     {
//         // we check the validity of a request in disk.c so we should never be here
//         DPRINT1("Warning: shouldn't be here\n");
//         goto ResetRecovery;
//     }
//
//     // if it is not a Sense Request
//     if (Request == FDODeviceExtension->ActiveSrb)
//     {
//         if (MmGetMdlVirtualAddress(Irp->MdlAddress) == Request->DataBuffer)
//         {
//             Mdl = Irp->MdlAddress;
//         }
//         else
//         {
//             Mdl = IoAllocateMdl(Request->DataBuffer,
//                                 Request->DataTransferLength,
//                                 FALSE,
//                                 FALSE,
//                                 NULL);
//
//             if (Mdl)
//             {
//                 IoBuildPartialMdl(Irp->MdlAddress,
//                                   Mdl,
//                                   Request->DataBuffer,
//                                   Request->DataTransferLength);
//             }
//         }
//
//         if (!Mdl)
//         {
//             DPRINT1("USBSTOR_CBWCompletionRoutine: Mdl - %p\n", Mdl);
//             goto ResetRecovery;
//         }
//     }
//     else
//     {
//         ASSERT(Request->DataBuffer);
//         TransferBuffer = Request->DataBuffer;
//     }
//
//     USBSTOR_IssueBulkOrInterruptRequest(FDODeviceExtension,
//                                         Irp,
//                                         PipeHandle,
//                                         TransferFlags,
//                                         Request->DataTransferLength,
//                                         TransferBuffer,
//                                         Mdl,
//                                         USBSTOR_DataCompletionRoutine);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
//
// ResetRecovery:
//     Request = FDODeviceExtension->ActiveSrb;
//     IoStack->Parameters.Scsi.Srb = Request;
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = STATUS_IO_DEVICE_ERROR;
//     Request->SrbStatus = SRB_STATUS_BUS_RESET;
//
//     USBSTOR_QueueTerminateRequest(PDODeviceExtension->LowerDeviceObject, Irp);
//     USBSTOR_QueueResetDevice(FDODeviceExtension);
//
//     return STATUS_CONTINUE_COMPLETION;
// }
//
// VOID
// DumpCBW(
//     PUCHAR Block)
// {
//     DPRINT("%02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x\n",
//         Block[0] & 0xFF, Block[1] & 0xFF, Block[2] & 0xFF, Block[3] & 0xFF, Block[4] & 0xFF, Block[5] & 0xFF, Block[6] & 0xFF, Block[7] & 0xFF, Block[8] & 0xFF, Block[9] & 0xFF,
//         Block[10] & 0xFF, Block[11] & 0xFF, Block[12] & 0xFF, Block[13] & 0xFF, Block[14] & 0xFF, Block[15] & 0xFF, Block[16] & 0xFF, Block[17] & 0xFF, Block[18] & 0xFF, Block[19] & 0xFF,
//         Block[20] & 0xFF, Block[21] & 0xFF, Block[22] & 0xFF, Block[23] & 0xFF, Block[24] & 0xFF, Block[25] & 0xFF, Block[26] & 0xFF, Block[27] & 0xFF, Block[28] & 0xFF, Block[29] & 0xFF,
//         Block[30] & 0xFF);
// }
//
// static
// NTSTATUS
// USBSTOR_SendCBWRequest(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PIRP Irp)
// {
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PIRP_CONTEXT Context = &FDODeviceExtension->CurrentIrpContext;
//
//     RtlZeroMemory(&Context->cbw, sizeof(CBW));
//     RtlZeroMemory(&Context->Urb, sizeof(URB));
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PDODeviceExtension = IoStack->DeviceObject->DeviceExtension;
//     Request = IoStack->Parameters.Scsi.Srb;
//
//     // Make a CBW structure from SCSI request block
//     Context->cbw.Signature = CBW_SIGNATURE;
//     Context->cbw.Tag = PtrToUlong(Irp);
//     Context->cbw.DataTransferLength = Request->DataTransferLength;
//     Context->cbw.Flags = ((UCHAR)Request->SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION) << 1;
//
//     // Per Bulk Only Transfer, LUN is 4 bits; clamp just in case
//     Context->cbw.LUN = (UCHAR)(PDODeviceExtension->LUN & 0x0F);
//     Context->cbw.CommandBlockLength = Request->CdbLength;
//
//     RtlCopyMemory(&Context->cbw.CommandBlock, Request->Cdb, Request->CdbLength);
//
//     DPRINT("CBW for IRP %p\n", Irp);
//     DumpCBW((PUCHAR)&Context->cbw);
//
//     // initialize rest of context
//     Context->Irp = Irp;
//     Context->StallRetryCount = 0;
//
//     return USBSTOR_IssueBulkOrInterruptRequest(
//         FDODeviceExtension,
//         Irp,
//         FDODeviceExtension->InterfaceInformation->Pipes[FDODeviceExtension->BulkOutPipeIndex].PipeHandle,
//         USBD_TRANSFER_DIRECTION_OUT,
//         sizeof(CBW),
//         &Context->cbw,
//         NULL,
//         USBSTOR_CBWCompletionRoutine);
// }
//
// static
// NTSTATUS
// USBSTOR_IssueRequestSense(
//     IN PFDO_DEVICE_EXTENSION FDODeviceExtension,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK CurrentSrb;
//     PSCSI_REQUEST_BLOCK SenseSrb;
//
//     DPRINT("USBSTOR_IssueRequestSense: \n");
//
//     CurrentSrb = FDODeviceExtension->ActiveSrb;
//     SenseSrb = &FDODeviceExtension->CurrentIrpContext.SenseSrb;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     IoStack->Parameters.Scsi.Srb = SenseSrb;
//
//     RtlZeroMemory(SenseSrb, sizeof(*SenseSrb));
//
//     SenseSrb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//     SenseSrb->Length = sizeof(*SenseSrb);
//     SenseSrb->CdbLength = CDB6GENERIC_LENGTH;
//     SenseSrb->SrbFlags = SRB_FLAGS_DATA_IN |
//                          SRB_FLAGS_NO_QUEUE_FREEZE |
//                          SRB_FLAGS_DISABLE_AUTOSENSE;
//
//     ASSERT(CurrentSrb->SenseInfoBufferLength);
//     ASSERT(CurrentSrb->SenseInfoBuffer);
//     DPRINT("SenseInfoBuffer %x, SenseInfoBufferLength %x\n", CurrentSrb->SenseInfoBuffer, CurrentSrb->SenseInfoBufferLength);
//
//     SenseSrb->DataTransferLength = CurrentSrb->SenseInfoBufferLength;
//     SenseSrb->DataBuffer = CurrentSrb->SenseInfoBuffer;
//
//     SrbGetCdb(SenseSrb)->CDB6GENERIC.OperationCode = SCSIOP_REQUEST_SENSE;
//     SrbGetCdb(SenseSrb)->AsByte[4] = CurrentSrb->SenseInfoBufferLength;
//
//     return USBSTOR_SendCBWRequest(FDODeviceExtension, Irp);
// }
//
// NTSTATUS
// USBSTOR_HandleExecuteSCSI(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension = (PPDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = IoStack->Parameters.Scsi.Srb;
//
//     DPRINT("USBSTOR_HandleExecuteSCSI Operation Code %x, Length %lu\n", SrbGetCdb(Request)->CDB10.OperationCode, Request->DataTransferLength);
//
//     /*
//      * Some stacks/devices don’t propagate the LUN in the CDB10 header bits.
//      * USB BOT uses CBW.bCBWLUN for addressing.
//      *
//      * TODO: There's actually a bigger bug here though,
//      * while the above CAN HAPPPEN:
//      * Card Readers multiplex their card slots for example, but we can deal with
//      * that as we put more effort into fixing the USB stack.
//      */
//     if (SrbGetCdb(Request)->CDB10.LogicalUnitNumber != PDODeviceExtension->LUN)
//     {
//         DPRINT1("USBSTOR_HandleExecuteSCSI: CDB LUN %lu != PDO LUN %lu (using CBW LUN)\n",
//                 (ULONG)SrbGetCdb(Request)->CDB10.LogicalUnitNumber,
//                 (ULONG)PDODeviceExtension->LUN);
//     }
//
//     return USBSTOR_SendCBWRequest(PDODeviceExtension->LowerDeviceObject->DeviceExtension, Irp);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================