use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateIrp, IoAllocateMdl, IoFreeIrp, IoFreeMdl, KEVENT, KeInitializeEvent, KeWaitForSingleObject, MmProbeAndLockPages, MmUnlockPages, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, ULONG32};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `#define NDEBUG` has no value, so there is no constant to emit.
// `static IO_COMPLETION_ROUTINE SptiCompletionRoutine;` is a forward prototype only; its definition is emitted with the function body.



unsafe extern "C" fn SptiFreeIrpContext(IrpContext: PPASSTHROUGH_IRP_CONTEXT) {
    let mut Irp: PIRP = core::ptr::null_mut();
    PAGED_CODE!();
    ASSERT!(!IrpContext.is_null());
    Irp = (*IrpContext).Irp;
    if !Irp.is_null() {
        if !(*Irp).MdlAddress.is_null() {
            MmUnlockPages((*Irp).MdlAddress as PMDL);
            IoFreeMdl((*Irp).MdlAddress as PMDL);
        }
        (*Irp).MdlAddress = core::ptr::null_mut();
        IoFreeIrp(Irp);
    }
    if !(*IrpContext).Srb.SenseInfoBuffer.is_null() {
        ExFreePoolWithTag((*IrpContext).Srb.SenseInfoBuffer, TAG_SPTI as ULONG);
    }
    ExFreePoolWithTag(IrpContext as PVOID, TAG_SPTI as ULONG);
}

unsafe extern "system" fn SptiCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(DeviceObject);

    if (*Irp).PendingReturned != 0 {
        KeSetEvent(Context as PRKEVENT, IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
    }

    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe extern "C" fn SptiCreateIrpContext(
    DeviceObject: PDEVICE_OBJECT,
    OriginalIrp: PIRP,
    DataBuffer: PVOID,
    DataBufferLength: ULONG,
    IsDirectMemoryAccess: BOOLEAN,
    IsBufferReadAccess: BOOLEAN,
) -> PPASSTHROUGH_IRP_CONTEXT {
    let mut IrpContext: PPASSTHROUGH_IRP_CONTEXT = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    PAGED_CODE!();

    IrpContext = ExAllocatePoolZero(
        NonPagedPool,
        (core::mem::size_of::<_PASSTHROUGH_IRP_CONTEXT>() as SIZE_T) as _,
        TAG_SPTI,
    ) as PPASSTHROUGH_IRP_CONTEXT;
    if IrpContext.is_null() {
        return core::ptr::null_mut();
    }

    'cleanup: {
        Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);
        (*IrpContext).Irp = Irp;
        if Irp.is_null() {
            break 'cleanup;
        }

        (*Irp).Tail.Overlay.Thread = PsGetCurrentThread();

        if !DataBuffer.is_null() {
            if IoAllocateMdl(
                DataBuffer,
                DataBufferLength,
                FALSE,
                FALSE,
                Irp,
            )
            .is_null()
            {
                break 'cleanup;
            }
            ASSERT!(!(*Irp).MdlAddress.is_null());

            // The C wraps this in _SEH2_TRY and bails to Cleanup on an exception; the
            // non-raising export returns that failure instead, so a bad user buffer
            // fails the request rather than raising with no handler on the stack.
            if ntkrnl_sys::probe::MmTryProbeAndLockPages(
                (*Irp).MdlAddress as _,
                (if IsDirectMemoryAccess != 0 {
                    (*OriginalIrp).RequestorMode as i32
                } else {
                    0i32
                }) as _,
                (if IsBufferReadAccess != 0 {
                    IoReadAccess as i32
                } else {
                    IoWriteAccess as i32
                }) as _,
            ) < 0
            {
                break 'cleanup;
            }
        }

        (*IrpContext).Srb.Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
        (*IrpContext).Srb.Length =
            RTL_FIELD_SIZE!(PASSTHROUGH_IRP_CONTEXT, Srb) as USHORT;
        (*IrpContext).Srb.OriginalRequest = Irp as PVOID;
        (*IrpContext).Srb.DataBuffer = DataBuffer;
        (*IrpContext).Srb.DataTransferLength = DataBufferLength;
        (*IrpContext).Srb.SrbFlags = SRB_FLAGS_NO_QUEUE_FREEZE as ULONG;

        IoStack = IoGetNextIrpStackLocation(Irp);
        (*IoStack).MinorFunction = IRP_MN_SCSI_CLASS as UCHAR;
        (*IoStack).MajorFunction = IRP_MJ_SCSI as UCHAR;
        (*IoStack).Parameters.Scsi.Srb =
            core::ptr::addr_of_mut!((*IrpContext).Srb) as _;

        return IrpContext;
    }

    DPRINT1!("Failed to create IRP\n");
    SptiFreeIrpContext(IrpContext);
    return core::ptr::null_mut();
}

unsafe extern "C" fn SptiInitializeOutputBuffer(Irp: PIRP) {
    PAGED_CODE!();
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let OutputBufferLength: ULONG = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;
    let InputBufferLength: ULONG = (*IoStack).Parameters.DeviceIoControl.InputBufferLength;

    if OutputBufferLength > InputBufferLength {
        let BufferStart: ULONG_PTR = ((*Irp).AssociatedIrp.SystemBuffer as ULONG_PTR)
            .wrapping_add(InputBufferLength as ULONG_PTR);
        RtlZeroMemory!(
            BufferStart as PVOID,
            OutputBufferLength.wrapping_sub(InputBufferLength) as usize,
        );
    }
}

unsafe extern "C" fn SptiCallDriver(
    DeviceObject: PDEVICE_OBJECT,
    IrpContext: PPASSTHROUGH_IRP_CONTEXT,
) -> NTSTATUS {
    let Irp: PIRP = (*IrpContext).Irp;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    SptiInitializeOutputBuffer(Irp);

    // TODO: Send the IRP in an asynchronous way (do not block the user app thread)
    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    IoSetCompletionRoutine(
        Irp,
        Some(SptiCompletionRoutine),
        core::ptr::addr_of_mut!(Event) as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}

unsafe extern "C" fn SptiSrbAppendSenseBuffer(
    Srb: PSCSI_REQUEST_BLOCK,
    BufferSize: ULONG,
) -> NTSTATUS {
    PAGED_CODE!();

    if BufferSize == 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_DISABLE_AUTOSENSE;
        return STATUS_SUCCESS;
    }

    (*Srb).SenseInfoBuffer = ExAllocatePoolUninitialized(
        NonPagedPoolCacheAligned,
        (BufferSize as SIZE_T) as _,
        TAG_SPTI,
    );
    if (*Srb).SenseInfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Srb).SenseInfoBufferLength = (BufferSize) as u8;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn SptiTranslateTaskFileToCdb(
    Cdb: *mut CDB,
    TaskFile: *mut UCHAR,
    AtaFlags: USHORT,
) {
    let mut Protocol: UCHAR = 0;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"sdk/lib/drivers/sptilib/sptilib.c\0".as_ptr() as PVOID,
            236,
            core::ptr::null_mut(),
        );
    }

    (*Cdb).ATA_PASSTHROUGH16.OperationCode = (SCSIOP_ATA_PASSTHROUGH16 as u8);

    (*Cdb).ATA_PASSTHROUGH16.Features7_0 =
        *TaskFile.offset((8 + 0) as isize);
    (*Cdb).ATA_PASSTHROUGH16.SectorCount7_0 =
        *TaskFile.offset((8 + 1) as isize);
    (*Cdb).ATA_PASSTHROUGH16.LbaLow7_0 =
        *TaskFile.offset((8 + 2) as isize);
    (*Cdb).ATA_PASSTHROUGH16.LbaMid7_0 =
        *TaskFile.offset((8 + 3) as isize);
    (*Cdb).ATA_PASSTHROUGH16.LbaHigh7_0 =
        *TaskFile.offset((8 + 4) as isize);
    (*Cdb).ATA_PASSTHROUGH16.Device =
        *TaskFile.offset((8 + 5) as isize);
    (*Cdb).ATA_PASSTHROUGH16.Command =
        *TaskFile.offset((8 + 6) as isize);

    if AtaFlags as i32 & (ATA_FLAGS_48BIT_COMMAND as i32) != 0 {
        (*Cdb).ATA_PASSTHROUGH16.Features15_8 =
            *TaskFile.offset(0 as isize);
        (*Cdb).ATA_PASSTHROUGH16.SectorCount15_8 =
            *TaskFile.offset(1 as isize);
        (*Cdb).ATA_PASSTHROUGH16.LbaLow15_8 =
            *TaskFile.offset(2 as isize);
        (*Cdb).ATA_PASSTHROUGH16.LbaMid15_8 =
            *TaskFile.offset(3 as isize);
        (*Cdb).ATA_PASSTHROUGH16.LbaHigh15_8 =
            *TaskFile.offset(4 as isize);

        (*Cdb).ATA_PASSTHROUGH16.set_Extend(1 as UCHAR);
    }

    /* Enable the check condition to get ATA fields from the device */
    (*Cdb).ATA_PASSTHROUGH16.set_CkCond(1 as UCHAR);

    if AtaFlags as i32 & ((ATA_FLAGS_DATA_IN | ATA_FLAGS_DATA_OUT) as i32) != 0 {
        (*Cdb).ATA_PASSTHROUGH16.set_TLength(3 as UCHAR);

        if AtaFlags as i32 & (ATA_FLAGS_DATA_IN as i32) != 0 {
            (*Cdb).ATA_PASSTHROUGH16.set_TDir(1 as UCHAR);
        }
    }

    if AtaFlags as i32 & (ATA_FLAGS_USE_DMA as i32) != 0 {
        Protocol = ATA_PASSTHROUGH_PROTOCOL_DMA as UCHAR;
    } else {
        if AtaFlags as i32 & ((ATA_FLAGS_DATA_IN | ATA_FLAGS_DATA_OUT) as i32) != 0 {
            if AtaFlags as i32 & (ATA_FLAGS_DATA_IN as i32) != 0 {
                Protocol = ATA_PASSTHROUGH_PROTOCOL_PIO_DATA_IN as UCHAR;
            } else {
                Protocol = ATA_PASSTHROUGH_PROTOCOL_PIO_DATA_OUT as UCHAR;
            }
        } else {
            Protocol = ATA_PASSTHROUGH_PROTOCOL_NON_DATA as UCHAR;
        }
    }
    (*Cdb).ATA_PASSTHROUGH16.set_Protocol(Protocol as UCHAR);
}

unsafe extern "C" fn SptiTranslateAtaStatusReturnToTaskFile(
    Srb: *mut SCSI_REQUEST_BLOCK,
    SenseData: *mut DESCRIPTOR_SENSE_DATA,
    TaskFile: *mut UCHAR,
) -> BOOLEAN {
    const APC_LEVEL: i32 = 1;

    let mut WasCheckCondition: BOOLEAN = FALSE as BOOLEAN;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"sdk/lib/drivers/sptilib/sptilib.c\0".as_ptr() as *mut core::ffi::c_void,
                301,
                core::ptr::null_mut(),
            );
        } else {
        }
    }

    if ((*Srb).SrbStatus as i32 & (SRB_STATUS_AUTOSENSE_VALID as i32)) == 0 {
        return WasCheckCondition;
    }

    if (*SenseData).ErrorCode() as i32 == (SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT as i32) {
        let mut Descriptor: PSCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN = core::ptr::null_mut();

        Descriptor = (core::ptr::addr_of_mut!((*SenseData).DescriptorBuffer) as *mut UCHAR)
            .offset(0 as isize) as *mut UCHAR
            as PSCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN;

        if (*Descriptor).Header.DescriptorType as i32 == (SCSI_SENSE_DESCRIPTOR_TYPE_ATA_STATUS_RETURN as i32) {
            *TaskFile.offset((8 + 0) as isize) =
                (*Descriptor).Error;
            *TaskFile.offset((8 + 1) as isize) =
                (*Descriptor).SectorCount7_0;
            *TaskFile.offset((8 + 2) as isize) =
                (*Descriptor).LbaLow7_0;
            *TaskFile.offset((8 + 3) as isize) =
                (*Descriptor).LbaMid7_0;
            *TaskFile.offset((8 + 4) as isize) =
                (*Descriptor).LbaHigh7_0;
            *TaskFile.offset((8 + 5) as isize) =
                (*Descriptor).Device;
            *TaskFile.offset((8 + 6) as isize) =
                (*Descriptor).Status;

            if (*Descriptor).Extend() != 0 {
                *TaskFile.offset(1 as isize) = (*Descriptor).SectorCount15_8;
                *TaskFile.offset(2 as isize) = (*Descriptor).LbaLow15_8;
                *TaskFile.offset(3 as isize) = (*Descriptor).LbaMid15_8;
                *TaskFile.offset(4 as isize) = (*Descriptor).LbaHigh15_8;
            }
        }
    }

    /*
     * The check condition bit will cause the APT command to return an error upon completion.
     * We should determine if there was a real error or not.
     */
    if ((*Srb).ScsiStatus as i32 == (SCSISTAT_CHECK_CONDITION as i32))
        && ((*SenseData).SenseKey() as i32 == (SCSI_SENSE_RECOVERED_ERROR as i32))
        && ((*SenseData).AdditionalSenseCode as i32 == (SCSI_ADSENSE_NO_SENSE as i32))
        && ((*SenseData).AdditionalSenseCodeQualifier as i32 == (SCSI_SENSEQ_ATA_PASS_THROUGH_INFORMATION_AVAILABLE as i32))
    {
        WasCheckCondition = TRUE as BOOLEAN;
    }

    return WasCheckCondition;
}

unsafe extern "C" fn SptiTranslateAptToSrb(
    Srb: *mut SCSI_REQUEST_BLOCK,
    Apt: *mut ATA_PASS_THROUGH_EX,
    TaskFile: PUCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"sdk/lib/drivers/sptilib/sptilib.c\0" as *const u8 as *const i8 as PVOID,
                358 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    Status = SptiSrbAppendSenseBuffer(
        Srb as PSCSI_REQUEST_BLOCK,
        FIELD_OFFSET!(DESCRIPTOR_SENSE_DATA, DescriptorBuffer) as ULONG
            + core::mem::size_of::<SCSI_SENSE_DESCRIPTOR_ATA_STATUS_RETURN>() as ULONG,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Srb).CdbLength = RTL_FIELD_SIZE!(CDB, ATA_PASSTHROUGH16) as UCHAR;
    (*Srb).TimeOutValue = (*Apt).TimeOutValue;

    (*Srb).PathId = (*Apt).PathId;
    (*Srb).TargetId = (*Apt).TargetId;
    (*Srb).Lun = (*Apt).Lun;

    if (*Apt).AtaFlags as i32 & (ATA_FLAGS_DATA_IN as i32) != 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_DATA_IN as ULONG;
    }
    if (*Apt).AtaFlags as i32 & (ATA_FLAGS_DATA_OUT as i32) != 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_DATA_OUT as ULONG;
    }

    SptiTranslateTaskFileToCdb(
        &raw mut (*Srb).Cdb as *mut UCHAR as PCDB,
        TaskFile,
        (*Apt).AtaFlags,
    );
    return STATUS_SUCCESS;
}

unsafe extern "C" fn SptiInitializeApt(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
    IsDirectMemoryAccess: BOOLEAN,
    MaximumTransferLength: ULONG,
    MaximumPhysicalPages: ULONG,
    Apt: PATA_PASS_THROUGH_EX,
    AptData: PPASSTHROUGH_DATA,
    TaskFile: *mut PUCHAR,
    DataBuffer: *mut PVOID,
) -> NTSTATUS {
    let mut StructSize: ULONG = 0;
    let mut IsNativeStructSize: BOOLEAN = 0;

    PAGED_CODE!();

    // The C's `#if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)` arm (32-bit callers on a
    // 64-bit kernel) is compiled on no arch here -- nothing defines BUILD_WOW64_ENABLED -- so
    // only its #else is ported. It needs IoIs32bitProcess and the *32 structs, neither of which
    // i386 has.
    StructSize = core::mem::size_of::<ATA_PASS_THROUGH_EX>() as ULONG;
    *TaskFile = (((Apt as ULONG_PTR)
        .wrapping_add(core::mem::offset_of!(ATA_PASS_THROUGH_EX, PreviousTaskFile) as ULONG_PTR)) as _);
    IsNativeStructSize = TRUE;
    

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < StructSize {
        DPRINT1!(
            "Buffer too small %lu/%lu\n",
            (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
            StructSize
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*Apt).Length as ULONG != StructSize {
        DPRINT1!("Unknown structure size %lu\n", StructSize);
        return STATUS_REVISION_MISMATCH;
    }

    if (*Apt).DataTransferLength > MaximumTransferLength {
        DPRINT1!(
            "Too large transfer %lu/%lu\n",
            (*Apt).DataTransferLength,
            MaximumTransferLength
        );
        return STATUS_INVALID_PARAMETER;
    }

    if BYTES_TO_PAGES((*Apt).DataTransferLength) > MaximumPhysicalPages {
        DPRINT1!(
            "Too large transfer %lu/%lu pages\n",
            BYTES_TO_PAGES((*Apt).DataTransferLength),
            MaximumPhysicalPages
        );
        return STATUS_INVALID_PARAMETER;
    }

    /* Retrieve the data buffer or the data buffer offset */
    // The C's `#if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)` arm (32-bit callers on a
    // 64-bit kernel) is compiled on no arch here -- nothing defines BUILD_WOW64_ENABLED -- so
    // only its #else is ported. It needs IoIs32bitProcess and the *32 structs, neither of which
    // i386 has.
    DBG_UNREFERENCED_LOCAL_VARIABLE!(IsNativeStructSize);

    (*AptData).BufferOffset = (*Apt).DataBufferOffset;
    

    if (*Apt).DataTransferLength != 0 {
        if IsDirectMemoryAccess != 0 {
            *DataBuffer = (*AptData).Buffer as _;
        } else {
            *DataBuffer = ((Apt as ULONG_PTR).wrapping_add((*AptData).BufferOffset)) as _;
        }
    } else {
        *DataBuffer = core::ptr::null_mut();
    }

    // FIXME: Validate the command opcode

    if (*(DataBuffer as *mut ULONG_PTR) & (*DeviceObject).AlignmentRequirement as ULONG_PTR) != 0 {
        DPRINT1!(
            "Unaligned data buffer %p:%lx\n",
            *(DataBuffer as *mut ULONG_PTR) as PVOID,
            (*DeviceObject).AlignmentRequirement
        );
        return STATUS_INVALID_PARAMETER;
    }

    if ((*Apt).DataTransferLength & (*DeviceObject).AlignmentRequirement) != 0 {
        DPRINT1!(
            "Unaligned data transfer length %lx:%lx\n",
            (*Apt).DataTransferLength,
            (*DeviceObject).AlignmentRequirement
        );
        return STATUS_INVALID_PARAMETER;
    }

    if (((*Apt).AtaFlags as i32 & (ATA_FLAGS_DATA_IN | ATA_FLAGS_DATA_OUT) as i32) != 0)
        && ((*Apt).DataTransferLength == 0)
    {
        DPRINT1!("Data buffer too small\n");
        return STATUS_BUFFER_TOO_SMALL;
    }

    if (*Apt).TimeOutValue < PASSTHROUGH_CMD_TIMEOUT_MIN_SEC
        || (*Apt).TimeOutValue > PASSTHROUGH_CMD_TIMEOUT_MAX_SEC
    {
        DPRINT1!("Invalid timeout value %lu\n", (*Apt).TimeOutValue);
        return STATUS_INVALID_PARAMETER;
    }

    if IsDirectMemoryAccess == 0 {
        let mut DataBufferEnd: ULONG_PTR = 0;
        let mut Status: NTSTATUS = 0;

        Status = RtlULongPtrAdd(
            (*AptData).BufferOffset,
            (*Apt).DataTransferLength as ULONG_PTR,
            core::ptr::addr_of_mut!(DataBufferEnd),
        );
        if !(Status >= 0) {
            DPRINT1!("Invalid data buffer offset\n");
            return Status;
        }

        if ((*Apt).Length as ULONG_PTR > (*AptData).BufferOffset)
            && ((*Apt).DataTransferLength != 0)
        {
            DPRINT1!("Data buffer overlaps APT\n");
            return STATUS_INVALID_PARAMETER;
        }

        if (((*Apt).AtaFlags as i32 & (ATA_FLAGS_DATA_IN as i32)) != 0)
            && (DataBufferEnd > (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as ULONG_PTR)
        {
            DPRINT1!("Data buffer outside of available space\n");
            return STATUS_INVALID_PARAMETER;
        }

        if (((*Apt).AtaFlags as i32 & (ATA_FLAGS_DATA_OUT as i32)) != 0)
            && (DataBufferEnd > (*IoStack).Parameters.DeviceIoControl.InputBufferLength as ULONG_PTR)
        {
            DPRINT1!("Data buffer outside of available space\n");
            return STATUS_INVALID_PARAMETER;
        }
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn SptiTranslateSptToSrb(
    Srb: *mut SCSI_REQUEST_BLOCK,
    Spt: *mut SCSI_PASS_THROUGH,
    Cdb: PUCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"sdk/lib/drivers/sptilib/sptilib.c\0"
                    .as_ptr() as *const i8 as PVOID,
                543 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    Status = SptiSrbAppendSenseBuffer(Srb as PSCSI_REQUEST_BLOCK, (*Spt).SenseInfoLength as ULONG);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Srb).CdbLength = (*Spt).CdbLength;
    (*Srb).TimeOutValue = (*Spt).TimeOutValue;

    (*Srb).PathId = (*Spt).PathId;
    (*Srb).TargetId = (*Spt).TargetId;
    (*Srb).Lun = (*Spt).Lun;

    if (*Spt).DataTransferLength != 0 {
        match ((*Spt).DataIn as i32) as u32 {
            SCSI_IOCTL_DATA_IN => {
                (*Srb).SrbFlags |= SRB_FLAGS_DATA_IN as ULONG;
            }
            SCSI_IOCTL_DATA_OUT => {
                (*Srb).SrbFlags |= SRB_FLAGS_DATA_OUT as ULONG;
            }
            _ => {
                (*Srb).SrbFlags |= SRB_FLAGS_UNSPECIFIED_DIRECTION as ULONG;
            }
        }
    }

    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*Srb).Cdb) as *mut UCHAR as *mut core::ffi::c_void,
        Cdb as *const core::ffi::c_void,
        (*Srb).CdbLength as usize,
    );

    return STATUS_SUCCESS;
}

unsafe extern "C" fn SptiInitializeSpt(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
    IsDirectMemoryAccess: BOOLEAN,
    MaximumTransferLength: ULONG,
    MaximumPhysicalPages: ULONG,
    Spt: PSCSI_PASS_THROUGH,
    SptData: PPASSTHROUGH_DATA,
    SenseInfoOffset: PULONG,
    Cdb: *mut PUCHAR,
    DataBuffer: *mut PVOID,
) -> NTSTATUS {
    let mut StructSize: ULONG = 0;
    let mut IsNativeStructSize: BOOLEAN = 0;
    let mut SenseInfoOffsetPtr: PULONG = core::ptr::null_mut();

    PAGED_CODE!();

    // The C's `#if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)` arm (32-bit callers on a
    // 64-bit kernel) is compiled on no arch here -- nothing defines BUILD_WOW64_ENABLED -- so
    // only its #else is ported. It needs IoIs32bitProcess and the *32 structs, neither of which
    // i386 has.
    StructSize = core::mem::size_of::<SCSI_PASS_THROUGH>() as ULONG;
    SenseInfoOffsetPtr = ((Spt as ULONG_PTR)
        .wrapping_add(core::mem::offset_of!(SCSI_PASS_THROUGH, SenseInfoOffset) as ULONG_PTR))
        as PVOID as PULONG;
    *Cdb = ((Spt as ULONG_PTR)
        .wrapping_add(core::mem::offset_of!(SCSI_PASS_THROUGH, Cdb) as ULONG_PTR))
        as PVOID as PUCHAR;
    IsNativeStructSize = TRUE;
    

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < StructSize {
        DPRINT1!(
            "Buffer too small %lu/%lu\n",
            (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
            StructSize
        );
        return STATUS_BUFFER_TOO_SMALL;
    }

    if ((*Spt).Length as ULONG) != StructSize {
        DPRINT1!("Unknown structure size %lu\n", StructSize);
        return STATUS_REVISION_MISMATCH;
    }

    if (*Spt).DataTransferLength > MaximumTransferLength {
        DPRINT1!(
            "Too large transfer %lu/%lu\n",
            (*Spt).DataTransferLength,
            MaximumTransferLength
        );
        return STATUS_INVALID_PARAMETER;
    }

    if BYTES_TO_PAGES((*Spt).DataTransferLength) > MaximumPhysicalPages {
        DPRINT1!(
            "Too large transfer %lu/%lu pages\n",
            BYTES_TO_PAGES((*Spt).DataTransferLength),
            MaximumPhysicalPages
        );
        return STATUS_INVALID_PARAMETER;
    }

    /* Retrieve the data buffer or the data buffer offset */
    // The C's `#if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)` arm (32-bit callers on a
    // 64-bit kernel) is compiled on no arch here -- nothing defines BUILD_WOW64_ENABLED -- so
    // only its #else is ported. It needs IoIs32bitProcess and the *32 structs, neither of which
    // i386 has.
    DBG_UNREFERENCED_LOCAL_VARIABLE!(IsNativeStructSize);

    (*SptData).BufferOffset = (*Spt).DataBufferOffset;
    
    *SenseInfoOffset = *SenseInfoOffsetPtr;

    if (*Spt).DataTransferLength != 0 {
        if IsDirectMemoryAccess != 0 {
            *DataBuffer = (*SptData).Buffer;
        } else {
            *DataBuffer = ((Spt as ULONG_PTR).wrapping_add((*SptData).BufferOffset) as PVOID)
                as PVOID;
        }
    } else {
        *DataBuffer = core::ptr::null_mut() as PVOID;
    }

    // FIXME: Validate the command opcode

    if (*(DataBuffer as PULONG_PTR) & (*DeviceObject).AlignmentRequirement as ULONG_PTR) != 0 {
        DPRINT1!(
            "Unaligned data buffer %p:%lx\n",
            (*(DataBuffer as PULONG_PTR)) as PVOID,
            (*DeviceObject).AlignmentRequirement
        );
        return STATUS_INVALID_PARAMETER;
    }

    if ((*Spt).DataTransferLength & (*DeviceObject).AlignmentRequirement) != 0 {
        DPRINT1!(
            "Unaligned data transfer length %lx:%lx\n",
            (*Spt).DataTransferLength,
            (*DeviceObject).AlignmentRequirement
        );
        return STATUS_INVALID_PARAMETER;
    }

    if (*Spt).TimeOutValue < PASSTHROUGH_CMD_TIMEOUT_MIN_SEC
        || (*Spt).TimeOutValue > PASSTHROUGH_CMD_TIMEOUT_MAX_SEC
    {
        DPRINT1!("Invalid timeout value %lu\n", (*Spt).TimeOutValue);
        return STATUS_INVALID_PARAMETER;
    }

    if ((*Spt).CdbLength as usize) > (RTL_FIELD_SIZE!(SCSI_REQUEST_BLOCK, Cdb) as usize) {
        DPRINT1!("Invalid CDB size %u\n", (*Spt).CdbLength as core::ffi::c_uint);
        return STATUS_INVALID_PARAMETER;
    }

    if (*Spt).DataIn > (SCSI_IOCTL_DATA_UNSPECIFIED as u8) {
        DPRINT1!("Unknown DataIn value %u\n", (*Spt).DataIn as core::ffi::c_uint);
        return STATUS_INVALID_PARAMETER;
    }

    if (*Spt).SenseInfoLength != 0 {
        let mut SenseBufferEnd: ULONG = 0;
        let mut Status: NTSTATUS = 0;

        Status = RtlULongAdd(
            *SenseInfoOffset,
            (*Spt).SenseInfoLength as ULONG,
            core::ptr::addr_of_mut!(SenseBufferEnd),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("Invalid sense buffer offset\n");
            return Status;
        }

        if ((*Spt).Length as ULONG) > *SenseInfoOffset {
            DPRINT1!("Sense buffer overlaps SPT\n");
            return STATUS_INVALID_PARAMETER;
        }

        if IsDirectMemoryAccess == 0 {
            if ((SenseBufferEnd as ULONG_PTR) > (*SptData).BufferOffset)
                && ((*Spt).DataTransferLength != 0)
            {
                DPRINT1!("Sense buffer overlaps data buffer\n");
                return STATUS_INVALID_PARAMETER;
            }

            if SenseBufferEnd > (*IoStack).Parameters.DeviceIoControl.OutputBufferLength {
                DPRINT1!("Sense buffer outside of available space\n");
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    if IsDirectMemoryAccess == 0 {
        let mut DataBufferEnd: ULONG_PTR = 0;
        let mut Status: NTSTATUS = 0;

        Status = RtlULongPtrAdd(
            (*SptData).BufferOffset,
            (*Spt).DataTransferLength as ULONG_PTR,
            core::ptr::addr_of_mut!(DataBufferEnd),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("Invalid data buffer offset\n");
            return Status;
        }

        if (((*Spt).Length as ULONG_PTR) > (*SptData).BufferOffset)
            && ((*Spt).DataTransferLength != 0)
        {
            DPRINT1!("Data buffer inside of structure bounds\n");
            return STATUS_INVALID_PARAMETER;
        }

        if (*Spt).DataIn == (SCSI_IOCTL_DATA_IN as u8) || (*Spt).DataIn == (SCSI_IOCTL_DATA_UNSPECIFIED as u8) {
            if DataBufferEnd
                > ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as ULONG_PTR)
            {
                DPRINT1!("Data buffer outside of available space\n");
                return STATUS_INVALID_PARAMETER;
            }
        }

        if (*Spt).DataIn == (SCSI_IOCTL_DATA_OUT as u8) || (*Spt).DataIn == (SCSI_IOCTL_DATA_UNSPECIFIED as u8) {
            if DataBufferEnd
                > ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as ULONG_PTR)
            {
                DPRINT1!("Data buffer outside of available space\n");
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SptiHandleAtaPassthru(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MaximumTransferLength: ULONG,
    MaximumPhysicalPages: ULONG,
) -> NTSTATUS {
    let mut Apt: PATA_PASS_THROUGH_EX = (*Irp).AssociatedIrp.SystemBuffer as PATA_PASS_THROUGH_EX;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut IsDirectMemoryAccess: BOOLEAN = 0;
    let mut AptData: PASSTHROUGH_DATA = core::mem::zeroed();
    let mut TaskFile: PUCHAR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut DataBuffer: PVOID = core::ptr::null_mut();
    let mut IrpContext: PPASSTHROUGH_IRP_CONTEXT = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ASSERT!(
        GET_IOCTL!(IoStack) == IOCTL_ATA_PASS_THROUGH
            || GET_IOCTL!(IoStack) == IOCTL_ATA_PASS_THROUGH_DIRECT
    );

    IsDirectMemoryAccess = (GET_IOCTL!(IoStack) == IOCTL_ATA_PASS_THROUGH_DIRECT) as BOOLEAN;

    Status = SptiInitializeApt(
        DeviceObject,
        Irp,
        IoStack,
        IsDirectMemoryAccess,
        MaximumTransferLength,
        MaximumPhysicalPages,
        Apt,
        core::ptr::addr_of_mut!(AptData),
        core::ptr::addr_of_mut!(TaskFile),
        core::ptr::addr_of_mut!(DataBuffer),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    DPRINT!(
        "APT: Flags %x DTL %lu\n",
        (*Apt).AtaFlags,
        (*Apt).DataTransferLength
    );
    DPRINT!(
        "APT: Curr %02x:%02x:%02x:%02x:%02x:%02x:%02x\n",
        *TaskFile.add((8 + 0) as usize),
        *TaskFile.add((8 + 1) as usize),
        *TaskFile.add((8 + 2) as usize),
        *TaskFile.add((8 + 3) as usize),
        *TaskFile.add((8 + 4) as usize),
        *TaskFile.add((8 + 5) as usize),
        *TaskFile.add((8 + 6) as usize),
        (*Apt).AtaFlags,
        (*Apt).DataTransferLength
    );
    if ((*Apt).AtaFlags & (ATA_FLAGS_48BIT_COMMAND as u16)) != 0 {
        DPRINT!(
            "APT: Prev %02x:%02x:%02x:%02x:%02x:%02x:%02x\n",
            *TaskFile.add(0 as usize),
            *TaskFile.add(1 as usize),
            *TaskFile.add(2 as usize),
            *TaskFile.add(3 as usize),
            *TaskFile.add(4 as usize),
            *TaskFile.add(5 as usize),
            *TaskFile.add(6 as usize)
        );
    }

    IrpContext = SptiCreateIrpContext(
        DeviceObject,
        Irp,
        DataBuffer,
        (*Apt).DataTransferLength,
        IsDirectMemoryAccess,
        (((*Apt).AtaFlags & (ATA_FLAGS_DATA_OUT as u16)) != 0) as BOOLEAN,
    );
    if IrpContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'Cleanup: {
        Status = SptiTranslateAptToSrb(
            core::ptr::addr_of_mut!((*IrpContext).Srb),
            Apt,
            TaskFile,
        );
        if !NT_SUCCESS!(Status) {
            break 'Cleanup;
        }

        Status = SptiCallDriver(DeviceObject, IrpContext);

        Srb = core::ptr::addr_of_mut!((*IrpContext).Srb);

        if SptiTranslateAtaStatusReturnToTaskFile(Srb, ((*Srb).SenseInfoBuffer).cast(), TaskFile) != 0 {
            Status = STATUS_SUCCESS;
        }

        /* Update the output buffer size */
        (*Apt).DataTransferLength = (*Srb).DataTransferLength;
        if IsDirectMemoryAccess != 0 {
            (*Irp).IoStatus.Information = (*Apt).Length as ULONG_PTR;
        } else {
            if ((*Apt).AtaFlags & (ATA_FLAGS_DATA_IN as u16)) != 0
                && (AptData.BufferOffset != 0)
            {
                (*Irp).IoStatus.Information =
                    AptData.BufferOffset + (*Apt).DataTransferLength as ULONG_PTR;
            } else {
                (*Irp).IoStatus.Information = (*Apt).Length as ULONG_PTR;
            }
        }

        DPRINT!(
            "APT: Return %lx, DTL %lu, %lu bytes\n",
            Status,
            (*Apt).DataTransferLength,
            (*Irp).IoStatus.Information as ULONG
        );
        DPRINT!(
            "APT: Return curr %02x:%02x:%02x:%02x:%02x:%02x:%02x\n",
            *TaskFile.add((8 + 0) as usize),
            *TaskFile.add((8 + 1) as usize),
            *TaskFile.add((8 + 2) as usize),
            *TaskFile.add((8 + 3) as usize),
            *TaskFile.add((8 + 4) as usize),
            *TaskFile.add((8 + 5) as usize),
            *TaskFile.add((8 + 6) as usize)
        );
        if ((*Apt).AtaFlags & (ATA_FLAGS_48BIT_COMMAND as u16)) != 0 {
            DPRINT!(
                "APT: Return prev %02x:%02x:%02x:%02x:%02x:%02x:%02x\n",
                *TaskFile.add(0 as usize),
                *TaskFile.add(1 as usize),
                *TaskFile.add(2 as usize),
                *TaskFile.add(3 as usize),
                *TaskFile.add(4 as usize),
                *TaskFile.add(5 as usize),
                *TaskFile.add(6 as usize)
            );
        }
    }

    SptiFreeIrpContext(IrpContext);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn SptiHandleScsiPassthru(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MaximumTransferLength: ULONG,
    MaximumPhysicalPages: ULONG,
) -> NTSTATUS {
    let mut Spt: PSCSI_PASS_THROUGH = (*Irp).AssociatedIrp.SystemBuffer as PSCSI_PASS_THROUGH;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut IsDirectMemoryAccess: BOOLEAN = 0;
    let mut SptData: PASSTHROUGH_DATA = core::mem::zeroed();
    let mut SenseInfoOffset: ULONG = 0;
    let mut Cdb: PUCHAR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut DataBuffer: PVOID = core::ptr::null_mut();
    let mut IrpContext: PPASSTHROUGH_IRP_CONTEXT = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ASSERT!(
        GET_IOCTL!(IoStack) == IOCTL_SCSI_PASS_THROUGH
            || GET_IOCTL!(IoStack) == IOCTL_SCSI_PASS_THROUGH_DIRECT
    );

    IsDirectMemoryAccess = (GET_IOCTL!(IoStack) == IOCTL_SCSI_PASS_THROUGH_DIRECT) as BOOLEAN;

    Status = SptiInitializeSpt(
        DeviceObject,
        Irp,
        IoStack,
        IsDirectMemoryAccess,
        MaximumTransferLength,
        MaximumPhysicalPages,
        Spt,
        core::ptr::addr_of_mut!(SptData),
        core::ptr::addr_of_mut!(SenseInfoOffset),
        core::ptr::addr_of_mut!(Cdb),
        core::ptr::addr_of_mut!(DataBuffer),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    DPRINT!(
        "SPT: Flags %u, DTL %lu, sense len %u\n",
        (*Spt).DataIn,
        (*Spt).DataTransferLength,
        (*Spt).SenseInfoLength
    );
    DPRINT!(
        "SPT: CDB %02x:%02x:%02x:%02x:%02x:%02x:%02x:%02x:%02x:%02x\n",
        *Cdb.add(0 as usize),
        *Cdb.add(1 as usize),
        *Cdb.add(2 as usize),
        *Cdb.add(3 as usize),
        *Cdb.add(4 as usize),
        *Cdb.add(5 as usize),
        *Cdb.add(6 as usize),
        *Cdb.add(7 as usize),
        *Cdb.add(8 as usize),
        *Cdb.add(9 as usize)
    );

    IrpContext = SptiCreateIrpContext(
        DeviceObject,
        Irp,
        DataBuffer,
        (*Spt).DataTransferLength,
        IsDirectMemoryAccess,
        ((*Spt).DataIn == (SCSI_IOCTL_DATA_OUT as u8)) as BOOLEAN,
    );
    if IrpContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'cleanup: {
        Status = SptiTranslateSptToSrb(
            core::ptr::addr_of_mut!((*IrpContext).Srb),
            Spt,
            Cdb,
        );
        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        Status = SptiCallDriver(DeviceObject, IrpContext);

        Srb = core::ptr::addr_of_mut!((*IrpContext).Srb);

        /*
         * Low-level storage drivers map SRB_STATUS_DATA_OVERRUN to STATUS_BUFFER_OVERFLOW.
         * This SRB status is usually set on an underrun and means success in that case.
         */
        if SRB_STATUS!((*Srb).SrbStatus) == (SRB_STATUS_DATA_OVERRUN as i32) {
            Status = STATUS_SUCCESS;
        }

        if ((*Srb).SrbStatus & (SRB_STATUS_AUTOSENSE_VALID as u8)) != 0 {
            ASSERT!((*Srb).SenseInfoBufferLength <= (*Spt).SenseInfoLength);

            (*Spt).SenseInfoLength = (*Srb).SenseInfoBufferLength;

            /* Copy the sense buffer back */
            RtlCopyMemory!(
                ((Spt as ULONG_PTR).wrapping_add(SenseInfoOffset as ULONG_PTR)) as PVOID,
                (*Srb).SenseInfoBuffer,
                (*Srb).SenseInfoBufferLength,
            );
        } else {
            (*Spt).SenseInfoLength = 0;
        }

        (*Spt).ScsiStatus = (*Srb).ScsiStatus;

        /* Update the output buffer size */
        (*Spt).DataTransferLength = (*Srb).DataTransferLength;
        if IsDirectMemoryAccess != 0 {
            if (*Spt).SenseInfoLength != 0 {
                (*Irp).IoStatus.Information =
                    SenseInfoOffset.wrapping_add((*Spt).SenseInfoLength as ULONG) as ULONG_PTR;
            } else {
                (*Irp).IoStatus.Information = (*Spt).Length as ULONG_PTR;
            }
        } else {
            if ((*Srb).SrbFlags & SRB_FLAGS_DATA_IN) != 0
                && (SptData.BufferOffset != 0)
            {
                (*Irp).IoStatus.Information = SptData
                    .BufferOffset
                    .wrapping_add((*Spt).DataTransferLength as ULONG_PTR);
            } else if (*Spt).SenseInfoLength != 0 {
                (*Irp).IoStatus.Information =
                    SenseInfoOffset.wrapping_add((*Spt).SenseInfoLength as ULONG) as ULONG_PTR;
            } else {
                (*Irp).IoStatus.Information = (*Spt).Length as ULONG_PTR;
            }
        }

        DPRINT!(
            "SPT: Return %lx, DTL %lu, Srb status %02x, SCSI status %02x, sense len %u, %lu bytes\n",
            Status,
            (*Spt).DataTransferLength,
            (*Srb).SrbStatus,
            (*Spt).ScsiStatus,
            (*Spt).SenseInfoLength,
            (*Irp).IoStatus.Information as ULONG
        );
    }

    SptiFreeIrpContext(IrpContext);
    return Status;
}
