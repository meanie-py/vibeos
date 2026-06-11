use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    
    
    fn FREE_POOL(ptr: PVOID);
    
    fn NT_ASSERTMSG(msg: PCSZ, expr: BOOLEAN);
    fn TracePrint(level_flags: u64, format: PCSZ, ...);
    fn UNREFERENCED_PARAMETER(p: PVOID);
    fn _Analysis_assume_(expr: BOOLEAN);
    
    fn ClasspLogSystemEventWithDeviceNumber(DeviceObject: PDEVICE_OBJECT, ErrorCode: NTSTATUS) -> NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn RetryRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
    Associated: BOOLEAN,
    TimeDelta100ns: LONGLONG,
) {
    let currentIrpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let nextIrpStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    let transferByteCount: ULONG;
    let dataTransferLength: ULONG;
    let srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER = Srb as PSTORAGE_REQUEST_BLOCK_HEADER;

    dataTransferLength = SrbGetDataTransferLength(srbHeader);
    if (*currentIrpStack).MajorFunction == IRP_MJ_READ as u8 ||
       (*currentIrpStack).MajorFunction == IRP_MJ_WRITE as u8 {

        _Analysis_assume_((*currentIrpStack).Parameters.Read.Length <= dataTransferLength);
        transferByteCount = (*currentIrpStack).Parameters.Read.Length;

    } else if (*Irp).MdlAddress != core::ptr::null_mut() {

        NT_ASSERT(SrbGetDataBuffer(srbHeader) == MmGetMdlVirtualAddress((*Irp).MdlAddress));
        _Analysis_assume_((*(*Irp).MdlAddress).ByteCount <= dataTransferLength);
        transferByteCount = (*(*Irp).MdlAddress).ByteCount;

    } else {

        transferByteCount = 0;
    }

    NT_ASSERT(!TEST_FLAG(SrbGetSrbFlags(srbHeader), SRB_FLAGS_FREE_SENSE_BUFFER));

    SrbSetDataTransferLength(srbHeader, transferByteCount);

    (*srbHeader).SrbStatus = 0;
    SrbSetScsiStatus(srbHeader, 0);

    if 0 == (*currentIrpStack).Parameters.Others.Argument4 as ULONG {
        SrbSetSrbFlags(srbHeader,
                      SRB_FLAGS_DISABLE_DISCONNECT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
        SrbClearSrbFlags(srbHeader, SRB_FLAGS_QUEUE_ACTION_ENABLE);

        SrbSetQueueTag(srbHeader, SP_UNTAGGED);
    }

    (*nextIrpStack).MajorFunction = IRP_MJ_SCSI as u8;

    (*nextIrpStack).Parameters.Scsi.Srb = Srb as *mut core::ffi::c_void;

    if Associated != 0 {
        IoSetCompletionRoutine(Irp, Some(ClassIoCompleteAssociated), Srb as PVOID, 1, 1, 1);
    } else {
        IoSetCompletionRoutine(Irp, Some(ClassIoComplete), Srb as PVOID, 1, 1, 1);
    }

    ClassRetryRequest(DeviceObject, Irp, TimeDelta100ns);
}
#[no_mangle]
pub unsafe extern "C" fn InterpretSenseInfoWithoutHistory(
    Fdo: PDEVICE_OBJECT,
    OriginalRequest: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
    MajorFunctionCode: UCHAR,
    IoDeviceCode: ULONG,
    PreviousRetryCount: ULONG,
    Status: *mut NTSTATUS,
    RetryIn100nsUnits: *mut LONGLONG,
) -> BOOLEAN {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
    let mut tmpRetry: LONGLONG = 0;
    let mut retry: BOOLEAN = 0u8;

    if !(*fdoData).InterpretSenseInfo.is_null() {
        let mut tempSrb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
        let mut srbPtr: PSCSI_REQUEST_BLOCK = Srb;

        NT_ASSERT(!(*(*fdoData).InterpretSenseInfo).Interpret.is_null());

        if (*Srb).Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK &&
            ((*(*fdoExtension).CommonExtension.DriverExtension).SrbSupport &
             CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0 {
            ClasspConvertToScsiRequestBlock(&mut tempSrb, Srb as PSTORAGE_REQUEST_BLOCK);
            srbPtr = &mut tempSrb;
        }

        retry = ((*(*fdoData).InterpretSenseInfo).Interpret)(
            Fdo,
            OriginalRequest,
            srbPtr,
            MajorFunctionCode,
            IoDeviceCode,
            PreviousRetryCount,
            core::ptr::null_mut(),
            Status,
            &mut tmpRetry,
        );
    } else {
        let mut seconds: ULONG = 0;

        retry = ClassInterpretSenseInfo(
            Fdo,
            Srb,
            MajorFunctionCode,
            IoDeviceCode,
            PreviousRetryCount,
            Status,
            &mut seconds,
        );
        tmpRetry = (seconds as LONGLONG) * 1000 * 1000 * 10;
    }

    if !RetryIn100nsUnits.is_null() {
        *RetryIn100nsUnits = tmpRetry;
    }
    retry
}
#[no_mangle]
pub unsafe extern "C" fn ClassFreeOrReuseSrb(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    let commonExt = core::ptr::addr_of_mut!((*FdoExtension).CommonExtension);

    NT_ASSERT(TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_FREE_SENSE_BUFFER) == 0);

    if (*commonExt).IsSrbLookasideListInitialized != 0 {
        ClasspFreeSrb(FdoExtension, Srb);
    } else {
        TracePrint((
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassFreeOrReuseSrb: someone is trying to use an uninitialized SrbLookasideList !!!\0".as_ptr() as PCHAR,
        ));
        FREE_POOL(Srb as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspValidateOffloadSupported(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION;
    let status: NTSTATUS;

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadSupported (%p): Entering function. Irp %p\n\0".as_ptr() as PCSZ,
                DeviceObject,
                Irp));

    fdoExt = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    status = STATUS_SUCCESS;

    if ((*DeviceObject).DeviceType == FILE_DEVICE_DISK) &&
        (!TEST_FLAG((*DeviceObject).Characteristics, FILE_FLOPPY_DISKETTE)) {

        if (*(*fdoExt).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits == 0 {

            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): Command not supported on this disk device.\n\0".as_ptr() as PCSZ,
                        DeviceObject));

            return STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }

        if !NT_SUCCESS((*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus) {

            let status = (*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus;

            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): VPD retrieval failed with %x.\n\0".as_ptr() as PCSZ,
                        DeviceObject,
                        status));

            TracePrint((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                        DeviceObject,
                        Irp,
                        status));

            return status;
        }
    } else {

        TracePrint((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadSupported (%p): Suported only on Disk devices.\n\0".as_ptr() as PCSZ,
                    DeviceObject));

        TracePrint((TRACE_LEVEL_VERBOSE,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                    DeviceObject,
                    Irp,
                    STATUS_DEVICE_FEATURE_NOT_SUPPORTED));

        return STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
    }

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                DeviceObject,
                Irp,
                status));

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassRetrieveDeviceRelations(
    Fdo: PDEVICE_OBJECT,
    RelationType: DEVICE_RELATION_TYPE,
    DeviceRelations: *mut PDEVICE_RELATIONS,
) -> NTSTATUS {

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    let mut count: ULONG = 0;
    let mut i: ULONG;

    let mut nextChild: PPHYSICAL_DEVICE_EXTENSION;

    let relationsSize: ULONG;
    let mut deviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let oldRelations: PDEVICE_RELATIONS = *DeviceRelations;

    let status: NTSTATUS;

    UNREFERENCED_PARAMETER(core::ptr::addr_of_mut!(RelationType) as PVOID);

    ClassAcquireChildLock(fdoExtension);

    nextChild = (*fdoExtension).CommonExtension.ChildList as PPHYSICAL_DEVICE_EXTENSION;

    while !nextChild.is_null() {
        let commonExtension: PCOMMON_DEVICE_EXTENSION;

        commonExtension = core::ptr::addr_of_mut!((*nextChild).CommonExtension);

        NT_ASSERTMSG(
            b"ClassPnp internal error: missing child on active list\n\0".as_ptr() as PCSZ,
            if (*nextChild).IsMissing == 0u8 { 1u8 } else { 0u8 }
        );

        nextChild = (*commonExtension).ChildList as PPHYSICAL_DEVICE_EXTENSION;

        count += 1;
    }

    if !oldRelations.is_null() {
        count += (*oldRelations).Count;
    }

    relationsSize = core::mem::size_of::<DEVICE_RELATIONS>() as ULONG +
                    (count * core::mem::size_of::<PDEVICE_OBJECT>() as ULONG);

    deviceRelations = ExAllocatePoolWithTag(PagedPool, relationsSize as usize, 0x35426353) as PDEVICE_RELATIONS;

    if deviceRelations.is_null() {
        TracePrint(
            0x0002_0000u64,
            b"ClassRetrieveDeviceRelations: unable to allocate %d bytes for device relations\n\0".as_ptr() as PCSZ,
            relationsSize
        );

        ClassReleaseChildLock(fdoExtension);

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(deviceRelations as PVOID, relationsSize as usize);

    if !oldRelations.is_null() {
        i = 0;
        while i < (*oldRelations).Count {
            (*deviceRelations).Objects[i as usize] = (*oldRelations).Objects[i as usize];
            i += 1;
        }

        FREE_POOL(oldRelations as PVOID);
    }

    nextChild = (*fdoExtension).CommonExtension.ChildList as PPHYSICAL_DEVICE_EXTENSION;
    i = count;

    while !nextChild.is_null() {
        let commonExtension: PCOMMON_DEVICE_EXTENSION;

        commonExtension = core::ptr::addr_of_mut!((*nextChild).CommonExtension);

        NT_ASSERTMSG(
            b"ClassPnp internal error: missing child on active list\n\0".as_ptr() as PCSZ,
            if (*nextChild).IsMissing == 0u8 { 1u8 } else { 0u8 }
        );

        _Analysis_assume_(if i >= 1 { 1u8 } else { 0u8 });
        i -= 1;
        (*deviceRelations).Objects[i as usize] = (*nextChild).DeviceObject;

        let status = ObReferenceObjectByPointer(
            (*nextChild).DeviceObject as PVOID,
            0,
            core::ptr::null_mut(),
            0i32
        );
        if !NT_SUCCESS(status) {
            NT_ASSERT(0u8);
            TracePrint(
                0x0001_0000u64,
                b"ClassRetrieveDeviceRelations: Error referencing child device %p by pointer\n\0".as_ptr() as PCSZ,
                (*nextChild).DeviceObject
            );
        }
        (*nextChild).IsEnumerated = 1u8;
        nextChild = (*commonExtension).ChildList as PPHYSICAL_DEVICE_EXTENSION;
    }

    NT_ASSERTMSG(
        b"Child list has changed: \0".as_ptr() as PCSZ,
        if i == 0 { 1u8 } else { 0u8 }
    );

    (*deviceRelations).Count = count;
    *DeviceRelations = deviceRelations;

    ClassReleaseChildLock(fdoExtension);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetBlockDeviceTokenLimitsInfo(
    DeviceObject: PDEVICE_OBJECT
) -> NTSTATUS {
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut status: NTSTATUS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG = 0;
    let mut pageLength: USHORT = 0;
    let mut descriptorLength: USHORT = 0;
    let mut dataBuffer: PVOID = core::ptr::null_mut();
    let bufferLength: UCHAR = VPD_MAX_BUFFER_SIZE;
    let mut allocationBufferLength: ULONG = bufferLength as ULONG;
    let mut cdb: PCDB;
    let mut dataTransferLength: ULONG = 0;
    let mut operatingParameters: PVPD_THIRD_PARTY_COPY_PAGE = core::ptr::null_mut();
    let mut blockDeviceTokenLimits: PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR = core::ptr::null_mut();

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Entering function.\n\0".as_ptr() as PCHAR,
                DeviceObject));

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, CLASSPNP_POOL_TAG_SRB) as PSCSI_REQUEST_BLOCK;

    if srb.is_null() {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate SRB.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        'exit: loop {
            FREE_POOL(dataBuffer);
            FREE_POOL(srb as PVOID);
            (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

            TracePrint((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        status));

            return status;
        }
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        allocationBufferLength = ALIGN_UP_BY(allocationBufferLength, KeGetRecommendedSharedDataAlignment());
        dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, allocationBufferLength, CLASSPNP_POOL_TAG_VPD);
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength as ULONG, CLASSPNP_POOL_TAG_VPD);
    }

    if dataBuffer.is_null() {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate dataBuffer.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        'exit: loop {
            FREE_POOL(dataBuffer);
            FREE_POOL(srb as PVOID);
            (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

            TracePrint((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        status));

            return status;
        }
    }

    operatingParameters = dataBuffer as PVPD_THIRD_PARTY_COPY_PAGE;

    RtlZeroMemory(dataBuffer, allocationBufferLength);

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        status = InitializeStorageRequestBlock(srb as PSTORAGE_REQUEST_BLOCK,
                                               STORAGE_ADDRESS_TYPE_BTL8,
                                               CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
                                               1,
                                               SrbExDataTypeScsiCdb16);
        if NT_SUCCESS(status) {
            (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        } else {
            NT_ASSERT(0);

            TracePrint((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Falling back to using SRB (instead of SRB_EX).\n\0".as_ptr() as PCHAR,
                        DeviceObject));

            RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
            (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
            (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI;
        }
    } else {
        RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI;
    }

    SrbSetTimeOutValue(srb as PVOID, (*fdoExtension).TimeOutValue);
    SrbSetRequestTag(srb as PVOID, SP_UNTAGGED);
    SrbSetRequestAttribute(srb as PVOID, SRB_SIMPLE_TAG_REQUEST);
    SrbAssignSrbFlags(srb as PVOID, (*fdoExtension).SrbFlags);

    SrbSetCdbLength(srb as PVOID, 6);

    cdb = SrbGetCdb(srb as PVOID) as PCDB;
    (*cdb).CDB6INQUIRY3.OperationCode = SCSIOP_INQUIRY;
    (*cdb).CDB6INQUIRY3.EnableVitalProductData = 1;
    (*cdb).CDB6INQUIRY3.PageCode = VPD_THIRD_PARTY_COPY;
    (*cdb).CDB6INQUIRY3.AllocationLength = bufferLength;

    status = ClassSendSrbSynchronous((*fdoExtension).DeviceObject,
                                     srb,
                                     dataBuffer,
                                     allocationBufferLength,
                                     0u8);

    dataTransferLength = SrbGetDataTransferLength(srb as PVOID);

    if status == STATUS_DATA_OVERRUN && dataTransferLength < bufferLength as ULONG {
        status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(status) {
        REVERSE_BYTES_SHORT(core::ptr::addr_of_mut!(pageLength), core::ptr::addr_of!((*operatingParameters).PageLength));
    } else {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Inquiry for TPC VPD failed with %x.\n\0".as_ptr() as PCHAR,
                    DeviceObject,
                    status));
    }

    if NT_SUCCESS(status) &&
        pageLength >= core::mem::size_of::<WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR>() as USHORT {

        let mut descriptorType: USHORT = 0;

        blockDeviceTokenLimits = (*operatingParameters).ThirdPartyCopyDescriptors.as_ptr() as PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR;
        REVERSE_BYTES_SHORT(core::ptr::addr_of_mut!(descriptorType), core::ptr::addr_of!((*blockDeviceTokenLimits).DescriptorType));
        REVERSE_BYTES_SHORT(core::ptr::addr_of_mut!(descriptorLength), core::ptr::addr_of!((*blockDeviceTokenLimits).DescriptorLength));

        if descriptorType == BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR_TYPE_WINDOWS &&
            VPD_PAGE_HEADER_SIZE + descriptorLength as ULONG == core::mem::size_of::<WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR>() as ULONG {

            (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits = 1u8;

            REVERSE_BYTES_SHORT(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumRangeDescriptors), core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumRangeDescriptors));
            REVERSE_BYTES(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumInactivityTimer), core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumInactivityTimer));
            REVERSE_BYTES(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.DefaultInactivityTimer), core::ptr::addr_of!((*blockDeviceTokenLimits).DefaultInactivityTimer));
            REVERSE_BYTES_QUAD(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize), core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumTokenTransferSize));
            REVERSE_BYTES_QUAD(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.OptimalTransferCount), core::ptr::addr_of!((*blockDeviceTokenLimits).OptimalTransferCount));

            TracePrint((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): %s %s (rev %s) reported following parameters: \n\t\t\tMaxRangeDescriptors: %u\n\t\t\tMaxIAT: %u\n\t\t\tDefaultIAT: %u \n\t\t\tMaxTokenTransferSize: %I64u\n\t\t\tOptimalTransferCount: %I64u\n\t\t\tOptimalTransferLengthGranularity: %u \n\t\t\tOptimalTransferLength: %u\n\t\t\tMaxTransferLength: %u\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).VendorIdOffset as isize) as PCSZ,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).ProductIdOffset as isize) as PCSZ,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).ProductRevisionOffset as isize) as PCSZ,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumRangeDescriptors,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumInactivityTimer,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.DefaultInactivityTimer,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.OptimalTransferCount,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.OptimalTransferLengthGranularity,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.OptimalTransferLength,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.MaximumTransferLength));
        } else {
            TracePrint((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): ThirdPartyCopy VPD data doesn't have Windows OffloadDataTransfer descriptor.\n\0".as_ptr() as PCHAR,
                        DeviceObject));

            (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits = 0u8;
            status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }
    } else {
        TracePrint((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): TPC VPD data didn't return TPC descriptors of interest.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits = 0u8;
        status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
    }

    FREE_POOL(dataBuffer);
    FREE_POOL(srb as PVOID);
    (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                DeviceObject,
                status));

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspRefreshFunctionSupportInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    ForceQuery: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG;
    let mut blockLimitsDataNew: CLASS_VPD_B0_DATA = core::mem::zeroed();
    let mut blockLimitsDataOriginal: PCLASS_VPD_B0_DATA;
    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let generationCount: ULONG;
    let changeRequestCount: ULONG;

    generationCount = (*(*FdoExtension).FunctionSupportInfo).GenerationCount;
    changeRequestCount = (*(*FdoExtension).FunctionSupportInfo).ChangeRequestCount;
    if ForceQuery == 0 && generationCount == changeRequestCount {
        return STATUS_SUCCESS;
    }

    if !(*FdoExtension).AdapterDescriptor.is_null() &&
        (*(*FdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize as SIZE_T, 0x31446353) as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ClasspDeviceGetBlockLimitsVPDPage(
        FdoExtension,
        srb,
        srbSize,
        &mut blockLimitsDataNew,
    );

    if NT_SUCCESS(status) {
        KeAcquireInStackQueuedSpinLock(
            &mut (*(*FdoExtension).FunctionSupportInfo).SyncLock,
            &mut lockHandle,
        );

        if generationCount == (*(*FdoExtension).FunctionSupportInfo).GenerationCount {
            blockLimitsDataOriginal = &mut (*(*FdoExtension).FunctionSupportInfo).BlockLimitsData;
            if (*blockLimitsDataOriginal).CommandStatus == -1i32 as UCHAR {
                *blockLimitsDataOriginal = blockLimitsDataNew;
            } else {
                (*blockLimitsDataOriginal).UGAVALID = blockLimitsDataNew.UGAVALID;
                (*blockLimitsDataOriginal).UnmapGranularityAlignment = blockLimitsDataNew.UnmapGranularityAlignment;
                (*blockLimitsDataOriginal).OptimalUnmapGranularity = blockLimitsDataNew.OptimalUnmapGranularity;
            }
            (*(*FdoExtension).FunctionSupportInfo).GenerationCount = changeRequestCount;
        }

        KeReleaseInStackQueuedSpinLock(&mut lockHandle);
    }

    FREE_POOL(srb as PVOID);
    status
}
#[no_mangle]
pub unsafe extern "system" fn ClasspMediaChangeRegistryCallBack(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID
) -> NTSTATUS {
    let valueFound: *mut ULONG;
    let deviceString: PUNICODE_STRING;
    let keyValue: PWSTR;
    let mut ValueLength = ValueLength;

    if ValueData.is_null() || Context.is_null() || EntryContext.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    valueFound = EntryContext as *mut ULONG;
    if (*valueFound) != 0 {
        return STATUS_SUCCESS;
    }

    if ValueLength == core::mem::size_of::<WCHAR>() as ULONG {
        return STATUS_SUCCESS;
    }

    if ValueType != REG_SZ {
        return STATUS_SUCCESS;
    }

    deviceString = Context as PUNICODE_STRING;
    keyValue = ValueData as PWSTR;
    ValueLength -= core::mem::size_of::<WCHAR>() as ULONG;

    if ValueLength > (*deviceString).Length as ULONG {
        ValueLength = (*deviceString).Length as ULONG;
    }

    if RtlCompareMemory((*deviceString).Buffer as PVOID, keyValue as PVOID, ValueLength as SIZE_T) == ValueLength as SIZE_T {
        (*valueFound) = 1;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ClassLogResourceExhaustionEvent(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID
) {

    let workItem = Context as PIO_WORKITEM;

    if NT_SUCCESS(ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_ERROR_DISK_RESOURCES_EXHAUSTED)) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClassLogResourceExhaustionEvent: DO (%p), Permanent resource exhaustion logged.\n\0".as_ptr() as PCHAR,
            DeviceObject
        );
    } else {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassLogResourceExhaustionEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as PCHAR,
            DeviceObject
        );
    }

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);

    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassLogThresholdEvent(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID
) {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let workItem: PIO_WORKITEM = Context as PIO_WORKITEM;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut resources: STORAGE_LB_PROVISIONING_MAP_RESOURCES = core::mem::zeroed();
    let resourcesSize: ULONG = core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as ULONG;
    let mut errorLogEntry: PIO_ERROR_LOG_PACKET = core::ptr::null_mut();
    let mut logEntrySize: ULONG = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as ULONG;
    let mut stringIndex: PWCHAR = core::ptr::null_mut();
    let mut stringSize: LONG = 0;
    let srbSize: ULONG;

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*(*fdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx,
                                srbSize,
                                0x41436353) as PSCSI_REQUEST_BLOCK;
    if !srb.is_null() {

        ClassGetLBProvisioningResources(DeviceObject,
                                        srb,
                                        resourcesSize,
                                        &mut resources);

        logEntrySize = (core::mem::size_of::<IO_ERROR_LOG_PACKET>() + (11 * core::mem::size_of::<WCHAR>()) + (2 * (21 * core::mem::size_of::<WCHAR>()))) as ULONG;
        logEntrySize = min(logEntrySize, ERROR_LOG_MAXIMUM_SIZE);
        logEntrySize = min(logEntrySize, MAXUCHAR as ULONG);

        errorLogEntry = IoAllocateErrorLogEntry(DeviceObject as PVOID, logEntrySize as UCHAR) as PIO_ERROR_LOG_PACKET;
        if !errorLogEntry.is_null() {

            RtlZeroMemory(errorLogEntry as PVOID, logEntrySize);
            (*errorLogEntry).StringOffset = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as USHORT;

            stringIndex = (errorLogEntry as ULONG_PTR + core::mem::size_of::<IO_ERROR_LOG_PACKET>()) as PWCHAR;
            stringSize = (logEntrySize - core::mem::size_of::<IO_ERROR_LOG_PACKET>() as ULONG) as LONG;

            status = RtlStringCbPrintfW(stringIndex, stringSize as usize, b"L\0%\0d\0\0".as_ptr() as PWSTR, (*fdoExtension).DeviceNumber);

            if NT_SUCCESS(status) {
                (*errorLogEntry).NumberOfStrings += 1;

                if (*resources).UsedMappingResourcesValid != 0 &&
                    (*resources).AvailableMappingResourcesValid != 0 {

                    stringIndex = stringIndex.offset((wcslen(stringIndex) + 1) as isize);
                    stringSize -= ((wcslen(stringIndex.offset(-((wcslen(stringIndex.offset(-1)) + 1) as isize))) + 1) * core::mem::size_of::<WCHAR>()) as LONG;

                    status = RtlStringCbPrintfW(stringIndex, stringSize as usize, b"L\0%\0I\06\04\0u\0\0".as_ptr() as PWSTR, (*resources).UsedMappingResources);

                    if NT_SUCCESS(status) {
                        (*errorLogEntry).NumberOfStrings += 1;

                        stringIndex = stringIndex.offset((wcslen(stringIndex) + 1) as isize);
                        stringSize -= ((wcslen(stringIndex.offset(-((wcslen(stringIndex.offset(-1)) + 1) as isize))) + 1) * core::mem::size_of::<WCHAR>()) as LONG;

                        status = RtlStringCbPrintfW(stringIndex, stringSize as usize, b"L\0%\0I\06\04\0u\0\0".as_ptr() as PWSTR, (*resources).AvailableMappingResources);

                        if NT_SUCCESS(status) {
                            (*errorLogEntry).NumberOfStrings += 1;
                        }
                    }
                } else {
                    TracePrint((TRACE_LEVEL_WARNING,
                                TRACE_FLAG_GENERAL,
                                b"ClassLogThresholdEvent: DO (%p), Used and available mapping resources were unavailable.\n\0".as_ptr() as PCSZ,
                                DeviceObject));
                }
            }

            if (*errorLogEntry).NumberOfStrings == 3 {
                if (*resources).UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
                    (*resources).AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN;

                } else if (*resources).UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
                           (*resources).AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL;

                } else if (*resources).UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
                           (*resources).AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN;

                } else if (*resources).UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
                           (*resources).AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL;

                } else {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX;
                }
            } else {
                (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED;
            }

            IoWriteErrorLogEntry(errorLogEntry as PVOID);

            TracePrint((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_GENERAL,
                        b"ClassLogThresholdEvent: DO (%p), Soft threshold notification logged.\n\0".as_ptr() as PCSZ,
                        DeviceObject));
        } else {
            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        b"ClassLogThresholdEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as PCSZ,
                        DeviceObject));
        }
    } else {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClassLogThresholdEvent: DO (%p), Failed to allocate memory for SRB.\n\0".as_ptr() as PCSZ,
                    DeviceObject));
    }

    core::sync::atomic::AtomicU32::from_mut(&mut (*(*(*fdoExtension).FunctionSupportInfo).LBProvisioningData).SoftThresholdEventPending)
        .store(0, core::sync::atomic::Ordering::SeqCst);

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);

    FREE_POOL(srb as PVOID);

    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "classp.h"
// #include "debug.h"
//
// // ---- RetryRequest ----
// VOID
// RetryRequest(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PSCSI_REQUEST_BLOCK Srb,
//     BOOLEAN Associated,
//     LONGLONG TimeDelta100ns
//     )
// {
//     PIO_STACK_LOCATION currentIrpStack = IoGetCurrentIrpStackLocation(Irp);
//     PIO_STACK_LOCATION nextIrpStack = IoGetNextIrpStackLocation(Irp);
//     ULONG transferByteCount;
//     ULONG dataTransferLength;
//     PSTORAGE_REQUEST_BLOCK_HEADER srbHeader = (PSTORAGE_REQUEST_BLOCK_HEADER)Srb;
//
//     // This function is obsolete but is still used by some of our class drivers.
//     // TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "RetryRequest is OBSOLETE !"));
//
//     //
//     // Determine the transfer count of the request.  If this is a read or a
//     // write then the transfer count is in the Irp stack.  Otherwise assume
//     // the MDL contains the correct length.  If there is no MDL then the
//     // transfer length must be zero.
//     //
//
//     dataTransferLength = SrbGetDataTransferLength(srbHeader);
//     if (currentIrpStack->MajorFunction == IRP_MJ_READ ||
//         currentIrpStack->MajorFunction == IRP_MJ_WRITE) {
//
//         _Analysis_assume_(currentIrpStack->Parameters.Read.Length <= dataTransferLength);
//         transferByteCount = currentIrpStack->Parameters.Read.Length;
//
//     } else if (Irp->MdlAddress != NULL) {
//
//         //
//         // Note this assumes that only read and write requests are spilt and
//         // other request do not need to be.  If the data buffer address in
//         // the MDL and the SRB don't match then transfer length is most
//         // likely incorrect.
//         //
//
//         NT_ASSERT(SrbGetDataBuffer(srbHeader) == MmGetMdlVirtualAddress(Irp->MdlAddress));
//     _Analysis_assume_(Irp->MdlAddress->ByteCount <= dataTransferLength);
//         transferByteCount = Irp->MdlAddress->ByteCount;
//
//     } else {
//
//         transferByteCount = 0;
//     }
//
//     //
//     // this is a safety net.  this should not normally be hit, since we are
//     // not guaranteed to be an fdoExtension
//     //
//
//     NT_ASSERT(!TEST_FLAG(SrbGetSrbFlags(srbHeader), SRB_FLAGS_FREE_SENSE_BUFFER));
//
//     //
//     // Reset byte count of transfer in SRB Extension.
//     //
//
//     SrbSetDataTransferLength(srbHeader, transferByteCount);
//
//     //
//     // Zero SRB statuses.
//     //
//
//     srbHeader->SrbStatus = 0;
//     SrbSetScsiStatus(srbHeader, 0);
//
//     //
//     // If this is the last retry, then disable all the special flags.
//     //
//
//     if ( 0 == (ULONG)(ULONG_PTR)currentIrpStack->Parameters.Others.Argument4 ) {
//         //
//         // Set the no disconnect flag, disable synchronous data transfers and
//         // disable tagged queuing. This fixes some errors.
//         // NOTE: Cannot clear these flags, just add to them
//         //
//
//         SrbSetSrbFlags(srbHeader,
//                           SRB_FLAGS_DISABLE_DISCONNECT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//         SrbClearSrbFlags(srbHeader, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//         SrbSetQueueTag(srbHeader, SP_UNTAGGED);
//     }
//
//
//     //
//     // Set up major SCSI function.
//     //
//
//     nextIrpStack->MajorFunction = IRP_MJ_SCSI;
//
//     //
//     // Save SRB address in next stack for port driver.
//     //
//
//     nextIrpStack->Parameters.Scsi.Srb = Srb;
//
//     if (Associated){
//         IoSetCompletionRoutine(Irp, ClassIoCompleteAssociated, Srb, TRUE, TRUE, TRUE);
//     }
//     else {
//         IoSetCompletionRoutine(Irp, ClassIoComplete, Srb, TRUE, TRUE, TRUE);
//     }
//
//     ClassRetryRequest(DeviceObject, Irp, TimeDelta100ns);
//     return;
// } // end RetryRequest()
//
// // ---- InterpretSenseInfoWithoutHistory ----
// BOOLEAN
// InterpretSenseInfoWithoutHistory(
//       PDEVICE_OBJECT Fdo,
//       PIRP OriginalRequest,
//       PSCSI_REQUEST_BLOCK Srb,
//          UCHAR MajorFunctionCode,
//          ULONG IoDeviceCode,
//          ULONG PreviousRetryCount,
//       NTSTATUS * Status,
//        
//          LONGLONG * RetryIn100nsUnits
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     LONGLONG tmpRetry = 0;
//     BOOLEAN retry = FALSE;
//
//     if (fdoData->InterpretSenseInfo != NULL)
//     {
//         SCSI_REQUEST_BLOCK tempSrb = {0};
//         PSCSI_REQUEST_BLOCK srbPtr = Srb;
//
//         // SAL annotations and ClassInitializeEx() both validate this
//         NT_ASSERT(fdoData->InterpretSenseInfo->Interpret != NULL);
//
//         //
//         // If class driver does not support extended SRB and this is
//         // an extended SRB, convert to legacy SRB and pass to class
//         // driver.
//         //
//         if ((Srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) &&
//             ((fdoExtension->CommonExtension.DriverExtension->SrbSupport &
//               CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0)) {
//             ClasspConvertToScsiRequestBlock(&tempSrb, (PSTORAGE_REQUEST_BLOCK)Srb);
//             srbPtr = &tempSrb;
//         }
//
//         retry = fdoData->InterpretSenseInfo->Interpret(Fdo,
//                                                        OriginalRequest,
//                                                        srbPtr,
//                                                        MajorFunctionCode,
//                                                        IoDeviceCode,
//                                                        PreviousRetryCount,
//                                                        NULL,
//                                                        Status,
//                                                        &tmpRetry);
//     }
//     else
//     {
//         ULONG seconds = 0;
//
//         retry = ClassInterpretSenseInfo(Fdo,
//                                         Srb,
//                                         MajorFunctionCode,
//                                         IoDeviceCode,
//                                         PreviousRetryCount,
//                                         Status,
//                                         &seconds);
//         tmpRetry = ((LONGLONG)seconds) * 1000 * 1000 * 10;
//     }
//
//
//     if (RetryIn100nsUnits != NULL)
//     {
//         *RetryIn100nsUnits = tmpRetry;
//     }
//     return retry;
// }
//
// // ---- ClassFreeOrReuseSrb ----
// VOID ClassFreeOrReuseSrb(     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//                                 PSCSI_REQUEST_BLOCK Srb)
//
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExt = &FdoExtension->CommonExtension;
//
//     // This function is obsolete, but still called by DISK.SYS .
//     // TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "ClassFreeOrReuseSrb is OBSOLETE !"));
//
//     //
//     // safety net.  this should never occur.  if it does, it's a potential
//     // memory leak.
//     //
//     NT_ASSERT(!TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_FREE_SENSE_BUFFER));
//
//     if (commonExt->IsSrbLookasideListInitialized){
//         
//         ClasspFreeSrb(FdoExtension, Srb);
//     }
//     else {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,"ClassFreeOrReuseSrb: someone is trying to use an uninitialized SrbLookasideList !!!"));
//         FREE_POOL(Srb);
//     }
// }
//
// // ---- ClasspValidateOffloadSupported ----
//  
//  
//  
// NTSTATUS
// ClasspValidateOffloadSupported(
//       PDEVICE_OBJECT DeviceObject,
//       PIRP Irp
//     )
//
//
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspValidateOffloadSupported (%p): Entering function. Irp %p\n",
//                 DeviceObject,
//                 Irp));
//
//     fdoExt = DeviceObject->DeviceExtension;
//     status = STATUS_SUCCESS;
//
//     //
//     // For now this command is only supported by disk devices
//     //
//     if ((DeviceObject->DeviceType == FILE_DEVICE_DISK) &&
//         (!TEST_FLAG(DeviceObject->Characteristics, FILE_FLOPPY_DISKETTE))) {
//
//         if (!fdoExt->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits) {
//
//             TracePrint((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_IOCTL,
//                         "ClasspValidateOffloadSupported (%p): Command not supported on this disk device.\n",
//                         DeviceObject));
//
//             status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//             goto __ClasspValidateOffloadSupported_Exit;
//         }
//
//         if (!NT_SUCCESS(fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus)) {
//
//             status = fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus;
//
//             TracePrint((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_IOCTL,
//                         "ClasspValidateOffloadSupported (%p): VPD retrieval failed with %x.\n",
//                         DeviceObject,
//                         status));
//
//             goto __ClasspValidateOffloadSupported_Exit;
//         }
//     } else {
//
//         TracePrint((TRACE_LEVEL_WARNING,
//                     TRACE_FLAG_IOCTL,
//                     "ClasspValidateOffloadSupported (%p): Suported only on Disk devices.\n",
//                     DeviceObject));
//
//         status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//         goto __ClasspValidateOffloadSupported_Exit;
//     }
//
// __ClasspValidateOffloadSupported_Exit:
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n",
//                 DeviceObject,
//                 Irp,
//                 status));
//
//     return status;
// }
//
// // ---- ClassRetrieveDeviceRelations ----
// NTSTATUS
// ClassRetrieveDeviceRelations(
//       PDEVICE_OBJECT Fdo,
//       DEVICE_RELATION_TYPE RelationType,
//       PDEVICE_RELATIONS *DeviceRelations
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//
//     ULONG count = 0;
//     ULONG i;
//
//     PPHYSICAL_DEVICE_EXTENSION nextChild;
//
//     ULONG relationsSize;
//     PDEVICE_RELATIONS deviceRelations = NULL;
//     PDEVICE_RELATIONS oldRelations = *DeviceRelations;
//
//     NTSTATUS status;
//
//     UNREFERENCED_PARAMETER(RelationType);
//
//     PAGED_CODE();
//
//     ClassAcquireChildLock(fdoExtension);
//
//     nextChild = fdoExtension->CommonExtension.ChildList;
//
//     //
//     // Count the number of PDO's attached to this disk
//     //
//
//     while (nextChild != NULL) {
//         PCOMMON_DEVICE_EXTENSION commonExtension;
//
//         commonExtension = &(nextChild->CommonExtension);
//
//         NT_ASSERTMSG("ClassPnp internal error: missing child on active list\n",
//                   (nextChild->IsMissing == FALSE));
//
//         nextChild = commonExtension->ChildList;
//
//         count++;
//     };
//
//     //
//     // If relations already exist in the QDR, adjust the current count
//     // to include the previous list.
//     //
//
//     if (oldRelations) {
//         count += oldRelations->Count;
//     }
//
//     relationsSize = (sizeof(DEVICE_RELATIONS) +
//                      (count * sizeof(PDEVICE_OBJECT)));
//
//     deviceRelations = ExAllocatePoolWithTag(PagedPool, relationsSize, '5BcS');
//
//     if (deviceRelations == NULL) {
//
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_ENUM,  "ClassRetrieveDeviceRelations: unable to allocate "
//                        "%d bytes for device relations\n", relationsSize));
//
//         ClassReleaseChildLock(fdoExtension);
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(deviceRelations, relationsSize);
//
//     if (oldRelations) {
//
//         //
//         // Copy the old relations to the new list and free the old list.
//         //
//
//         for (i = 0; i < oldRelations->Count; i++) {
//             deviceRelations->Objects[i] = oldRelations->Objects[i];
//         }
//
//         FREE_POOL(oldRelations);
//     }
//
//     nextChild = fdoExtension->CommonExtension.ChildList;
//     i = count;
//
//     while (nextChild != NULL) {
//         PCOMMON_DEVICE_EXTENSION commonExtension;
//
//         commonExtension = &(nextChild->CommonExtension);
//
//         NT_ASSERTMSG("ClassPnp internal error: missing child on active list\n",
//                   (nextChild->IsMissing == FALSE));
//
//     _Analysis_assume_(i >= 1);
//     deviceRelations->Objects[--i] = nextChild->DeviceObject;
//
//         status = ObReferenceObjectByPointer(
//                     nextChild->DeviceObject,
//                     0,
//                     NULL,
//                     KernelMode);
//         if (!NT_SUCCESS(status)) {
//             NT_ASSERT(!"Error referencing child device by pointer");
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_ENUM,
//                         "ClassRetrieveDeviceRelations: Error referencing child "
//                         "device %p by pointer\n", nextChild->DeviceObject));
//
//         }
//         nextChild->IsEnumerated = TRUE;
//         nextChild = commonExtension->ChildList;
//     }
//
//     NT_ASSERTMSG("Child list has changed: ", i == 0);
//
//     deviceRelations->Count = count;
//     *DeviceRelations = deviceRelations;
//
//     ClassReleaseChildLock(fdoExtension);
//     return STATUS_SUCCESS;
// } // end ClassRetrieveDeviceRelations()
//
// // ---- ClasspGetBlockDeviceTokenLimitsInfo ----
//  
// NTSTATUS
// ClasspGetBlockDeviceTokenLimitsInfo(
//       PDEVICE_OBJECT DeviceObject
//     )
//
//
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     NTSTATUS status;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     ULONG srbSize = 0;
//     USHORT pageLength = 0;
//     USHORT descriptorLength = 0;
//     PVOID dataBuffer = NULL;
//     UCHAR bufferLength = VPD_MAX_BUFFER_SIZE;
//     ULONG allocationBufferLength = bufferLength;
//     PCDB cdb;
//     ULONG dataTransferLength = 0;
//     PVPD_THIRD_PARTY_COPY_PAGE operatingParameters = NULL;
//     PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR blockDeviceTokenLimits = NULL;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_PNP,
//                 "ClasspGetBlockDeviceTokenLimitsInfo (%p): Entering function.\n",
//                 DeviceObject));
//
//     //
//     // Allocate an SRB for querying the device for LBP-related info.
//     //
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, CLASSPNP_POOL_TAG_SRB);
//
//     if (!srb) {
//
//         TracePrint((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate SRB.\n",
//                     DeviceObject));
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
//     }
//
// #if defined(_ARM_) || defined(_ARM64_)
//     //
//     // ARM has specific alignment requirements, although this will not have a functional impact on x86 or amd64
//     // based platforms. We are taking the conservative approach here.
//     //
//     allocationBufferLength = ALIGN_UP_BY(allocationBufferLength, KeGetRecommendedSharedDataAlignment());
//     dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, allocationBufferLength, CLASSPNP_POOL_TAG_VPD);
// #else
//     dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength, CLASSPNP_POOL_TAG_VPD);
// #endif
//
//     if (!dataBuffer) {
//
//         TracePrint((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate dataBuffer.\n",
//                     DeviceObject));
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
//     }
//
//     operatingParameters = (PVPD_THIRD_PARTY_COPY_PAGE)dataBuffer;
//
//     RtlZeroMemory(dataBuffer, allocationBufferLength);
//
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         status = InitializeStorageRequestBlock((PSTORAGE_REQUEST_BLOCK)srb,
//                                                STORAGE_ADDRESS_TYPE_BTL8,
//                                                CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
//                                                1,
//                                                SrbExDataTypeScsiCdb16);
//         if (NT_SUCCESS(status)) {
//
//             ((PSTORAGE_REQUEST_BLOCK)srb)->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//
//         } else {
//
//             //
//             // Should not occur. Revert to legacy SRB.
//             //
//             NT_ASSERT(FALSE);
//
//             TracePrint((TRACE_LEVEL_WARNING,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): Falling back to using SRB (instead of SRB_EX).\n",
//                         DeviceObject));
//
//             RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//             srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//             srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         }
//     } else {
//
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//         srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//     }
//
//     SrbSetTimeOutValue(srb, fdoExtension->TimeOutValue);
//     SrbSetRequestTag(srb, SP_UNTAGGED);
//     SrbSetRequestAttribute(srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbAssignSrbFlags(srb, fdoExtension->SrbFlags);
//
//     SrbSetCdbLength(srb, 6);
//
//     cdb = SrbGetCdb(srb);
//     cdb->CDB6INQUIRY3.OperationCode = SCSIOP_INQUIRY;
//     cdb->CDB6INQUIRY3.EnableVitalProductData = 1;
//     cdb->CDB6INQUIRY3.PageCode = VPD_THIRD_PARTY_COPY;
//     cdb->CDB6INQUIRY3.AllocationLength = bufferLength;
//
//     status = ClassSendSrbSynchronous(fdoExtension->DeviceObject,
//                                      srb,
//                                      dataBuffer,
//                                      allocationBufferLength,
//                                      FALSE);
//
//     //
//     // Handle the case where we get back STATUS_DATA_OVERRUN because the input
//     // buffer was larger than necessary.
//     //
//     dataTransferLength = SrbGetDataTransferLength(srb);
//
//     if (status == STATUS_DATA_OVERRUN && dataTransferLength < bufferLength) {
//
//         status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(status)) {
//
//         REVERSE_BYTES_SHORT(&pageLength, &operatingParameters->PageLength);
//
//     } else {
//         TracePrint((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Inquiry for TPC VPD failed with %x.\n",
//                     DeviceObject,
//                     status));
//     }
//
//     if ((NT_SUCCESS(status)) &&
//         (pageLength >= sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {
//
//         USHORT descriptorType;
//
//         blockDeviceTokenLimits = (PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR)operatingParameters->ThirdPartyCopyDescriptors;
//         REVERSE_BYTES_SHORT(&descriptorType, &blockDeviceTokenLimits->DescriptorType);
//         REVERSE_BYTES_SHORT(&descriptorLength, &blockDeviceTokenLimits->DescriptorLength);
//
//         if ((descriptorType == BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR_TYPE_WINDOWS) &&
//             (VPD_PAGE_HEADER_SIZE + descriptorLength == sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {
//
//             fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = TRUE;
//
//             REVERSE_BYTES_SHORT(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors, &blockDeviceTokenLimits->MaximumRangeDescriptors);
//             REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer, &blockDeviceTokenLimits->MaximumInactivityTimer);
//             REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer, &blockDeviceTokenLimits->DefaultInactivityTimer);
//             REVERSE_BYTES_QUAD(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize, &blockDeviceTokenLimits->MaximumTokenTransferSize);
//             REVERSE_BYTES_QUAD(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount, &blockDeviceTokenLimits->OptimalTransferCount);
//
//             TracePrint((TRACE_LEVEL_INFORMATION,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): %s %s (rev %s) reported following parameters: \
//                         \n\t\t\tMaxRangeDescriptors: %u\n\t\t\tMaxIAT: %u\n\t\t\tDefaultIAT: %u \
//                         \n\t\t\tMaxTokenTransferSize: %I64u\n\t\t\tOptimalTransferCount: %I64u\n\t\t\tOptimalTransferLengthGranularity: %u \
//                         \n\t\t\tOptimalTransferLength: %u\n\t\t\tMaxTransferLength: %u\n",
//                         DeviceObject,
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->VendorIdOffset),
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductIdOffset),
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductRevisionOffset),
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLengthGranularity,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLength,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.MaximumTransferLength));
//         } else {
//
//             TracePrint((TRACE_LEVEL_WARNING,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): ThirdPartyCopy VPD data doesn't have Windows OffloadDataTransfer descriptor.\n",
//                         DeviceObject));
//
//             fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = FALSE;
//             status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//         }
//     } else {
//
//         TracePrint((TRACE_LEVEL_WARNING,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): TPC VPD data didn't return TPC descriptors of interest.\n",
//                     DeviceObject));
//
//         fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = FALSE;
//         status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//     }
//
// __ClasspGetBlockDeviceTokenLimitsInfo_Exit:
//
//     FREE_POOL(dataBuffer);
//     FREE_POOL(srb);
//     fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus = status;
//
//     TracePrint((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_PNP,
//                 "ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n",
//                 DeviceObject,
//                 status));
//
//     return status;
// }
//
// // ---- ClasspRefreshFunctionSupportInfo ----
// NTSTATUS
// ClasspRefreshFunctionSupportInfo(
//       PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//       BOOLEAN ForceQuery
//     )
//
// {
//     NTSTATUS            status;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     ULONG               srbSize;
//     CLASS_VPD_B0_DATA   blockLimitsDataNew;
//     PCLASS_VPD_B0_DATA  blockLimitsDataOriginal;
//     KLOCK_QUEUE_HANDLE  lockHandle;
//     ULONG               generationCount;
//     ULONG               changeRequestCount;
//
//     //
//     // ChangeRequestCount is incremented every time we get an unit attention with
//     // SCSI_ADSENSE_OPERATING_CONDITIONS_CHANGED.  GenerationCount will be set to
//     // ChangeRequestCount after CLASS_FUNCTION_SUPPORT_INFO is refreshed with the latest
//     // VPD data.  i.e.  if both values are the same, data in
//     // CLASS_FUNCTION_SUPPORT_INFO is current
//     //
//
//     generationCount = FdoExtension->FunctionSupportInfo->GenerationCount;
//     changeRequestCount = FdoExtension->FunctionSupportInfo->ChangeRequestCount;
//     if (!ForceQuery && generationCount == changeRequestCount) {
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // Allocate an SRB for querying the device for LBP-related info if either
//     // the Logical Block Provisioning (0xB2) or Block Limits (0xB0) VPD page
//     // exists.
//     //
//     if ((FdoExtension->AdapterDescriptor != NULL) &&
//         (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, '1DcS');
//     if (srb == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     status = ClasspDeviceGetBlockLimitsVPDPage(FdoExtension,
//                                                srb,
//                                                srbSize,
//                                                &blockLimitsDataNew);
//
//     if (NT_SUCCESS(status)) {
//
//         KeAcquireInStackQueuedSpinLock(&FdoExtension->FunctionSupportInfo->SyncLock, &lockHandle);
//
//         //
//         // If the generationCount didn't change since we looked at it last time, it means
//         // no one has tried to update the CLASS_FUNCTION_SUPPORT_INFO data; otherwise, someone
//         // else has beat us to it.
//         //
//         if (generationCount == FdoExtension->FunctionSupportInfo->GenerationCount) {
//
//             blockLimitsDataOriginal = &FdoExtension->FunctionSupportInfo->BlockLimitsData;
//             if (blockLimitsDataOriginal->CommandStatus == -1) {
//                 //
//                 // CommandStatus == -1 means this is the first time we have
//                 // gotten the block limits data.
//                 //
//                 *blockLimitsDataOriginal = blockLimitsDataNew;
//             } else {
//                 //
//                 // We only expect the Optimal Unmap Granularity (and alignment)
//                 // to change, so those are the only parameters we update.
//                 //
//                 blockLimitsDataOriginal->UGAVALID = blockLimitsDataNew.UGAVALID;
//                 blockLimitsDataOriginal->UnmapGranularityAlignment = blockLimitsDataNew.UnmapGranularityAlignment;
//                 blockLimitsDataOriginal->OptimalUnmapGranularity = blockLimitsDataNew.OptimalUnmapGranularity;
//             }
//             FdoExtension->FunctionSupportInfo->GenerationCount = changeRequestCount;
//         }
//
//         KeReleaseInStackQueuedSpinLock(&lockHandle);
//     }
//
//     FREE_POOL(srb);
//     return status;
// }
//
// // ---- ClasspMediaChangeRegistryCallBack ----
//  
//  
//  
// NTSTATUS
// NTAPI 
// ClasspMediaChangeRegistryCallBack(
//       PWSTR ValueName,
//       ULONG ValueType,
//       PVOID ValueData,
//       ULONG ValueLength,
//       PVOID Context,
//       PVOID EntryContext
//     )
// {
//     PULONG valueFound;
//     PUNICODE_STRING deviceString;
//     PWSTR keyValue;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(ValueName);
//
//     if (ValueData == NULL ||
//         Context == NULL ||
//         EntryContext == NULL) {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // if we have already set the value to true, exit
//     //
//
//     valueFound = EntryContext;
//     if ((*valueFound) != 0) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspMcnRegCB: already set to true\n"));
//         return STATUS_SUCCESS;
//     }
//
//     if (ValueLength == sizeof(WCHAR)) {
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN, "ClasspMcnRegCB: NULL string should "
//                     "never be passed to registry call-back!\n"));
//         return STATUS_SUCCESS;
//     }
//
//
//     //
//     // if the data is not a terminated string, exit
//     //
//
//     if (ValueType != REG_SZ) {
//         return STATUS_SUCCESS;
//     }
//
//     deviceString = Context;
//     keyValue = ValueData;
//     ValueLength -= sizeof(WCHAR); // ignore the null character
//
//     //
//     // do not compare more memory than is in deviceString
//     //
//
//     if (ValueLength > deviceString->Length) {
//         ValueLength = deviceString->Length;
//     }
//
//     //
//     // if the strings match, disable autorun
//     //
//
//     if (RtlCompareMemory(deviceString->Buffer, keyValue, ValueLength) == ValueLength) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: Match found\n"));
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: DeviceString at %p\n",
//                     deviceString->Buffer));
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: KeyValue at %p\n",
//                     keyValue));
//         (*valueFound) = TRUE;
//     }
//
//     return STATUS_SUCCESS;
// } // end ClasspMediaChangeRegistryCallBack()
//
// // ---- ClassLogResourceExhaustionEvent ----
//  
//  
//  
// VOID
// NTAPI 
// ClassLogResourceExhaustionEvent(
//       PDEVICE_OBJECT DeviceObject,
//       PVOID Context
//     )
//
// {
//     PIO_WORKITEM workItem = (PIO_WORKITEM)Context;
//
//     if (NT_SUCCESS(ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_ERROR_DISK_RESOURCES_EXHAUSTED))) {
//
//         TracePrint((TRACE_LEVEL_INFORMATION,
//                     TRACE_FLAG_GENERAL,
//                     "ClassLogResourceExhaustionEvent: DO (%p), Permanent resource exhaustion logged.\n",
//                     DeviceObject));
//     } else {
//         TracePrint((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_GENERAL,
//                     "ClassLogResourceExhaustionEvent: DO (%p), Failed to allocate memory for error log entry.\n",
//                     DeviceObject));
//     }
//
//
//     ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);
//
//     if (workItem != NULL) {
//         IoFreeWorkItem(workItem);
//     }
// }
//
// // ---- ClassLogThresholdEvent ----
//  
//  
//  
// VOID
// NTAPI 
// ClassLogThresholdEvent(
//       PDEVICE_OBJECT DeviceObject,
//       PVOID Context
//     )
//
// {
//     NTSTATUS status = STATUS_SUCCESS;
//     PIO_WORKITEM workItem = (PIO_WORKITEM)Context;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     STORAGE_LB_PROVISIONING_MAP_RESOURCES resources = {0};
//     ULONG resourcesSize = sizeof(STORAGE_LB_PROVISIONING_MAP_RESOURCES);
//     PIO_ERROR_LOG_PACKET errorLogEntry = NULL;
//     ULONG logEntrySize = sizeof(IO_ERROR_LOG_PACKET);
//     PWCHAR stringIndex = NULL;
//     LONG stringSize = 0;
//     ULONG srbSize;
//
//     //
//     // Allocate an SRB for getting the LBP log page.
//     //
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 'ACcS');
//     if (srb != NULL) {
//
//         //
//         // Try to get the LBP resources from the device so we can report them in
//         // the system event log.
//         //
//         ClassGetLBProvisioningResources(DeviceObject,
//                                         srb,
//                                         resourcesSize,
//                                         &resources);
//
//         //
//         // We need to allocate enough space for 3 insertion strings:
//         // The first is a ULONG representing the disk number in decimal, which means
//         // a max of 10 digits, plus one for the NULL character.
//         // The second and third are ULONGLONGs representing the used and available
//         // bytes, which means a max of 20 digits, plus one for the NULL character.
//         // Make sure we do not exceed the max error log size or the max size of a
//         // UCHAR since the size gets truncated to a UCHAR when we pass it to
//         // IoAllocateErrorLogEntry().
//         //
//         logEntrySize = sizeof(IO_ERROR_LOG_PACKET) + (11 * sizeof(WCHAR)) + (2 * (21 * sizeof(WCHAR)));
//         logEntrySize = min(logEntrySize, ERROR_LOG_MAXIMUM_SIZE);
//         logEntrySize = min(logEntrySize, MAXUCHAR);
//
//         errorLogEntry = (PIO_ERROR_LOG_PACKET)IoAllocateErrorLogEntry(DeviceObject, (UCHAR)logEntrySize);
//         if (errorLogEntry != NULL)
//         {
//             //
//             // There are two event IDs we can use here.  Both use the disk number,
//             // but one reports the available and used bytes while the other does not.
//             // We fall back on the latter if we failed to obtain the available and
//             // used byte counts from the LBP log page.
//             //
//             // The event insertion strings need to be in this order:
//             // 1. The disk number. (Both event IDs use this.)
//             // 2. Bytes used.
//             // 3. Bytes available.
//             //
//
//             RtlZeroMemory(errorLogEntry, logEntrySize);
//             errorLogEntry->StringOffset = sizeof(IO_ERROR_LOG_PACKET);
//
//             stringIndex = (PWCHAR)((ULONG_PTR)errorLogEntry + sizeof(IO_ERROR_LOG_PACKET));
//             stringSize = logEntrySize - sizeof(IO_ERROR_LOG_PACKET);
//
//             //
//             // Add the disk number to the insertion strings.
//             //
//             status = RtlStringCbPrintfW(stringIndex, stringSize, L"%d", fdoExtension->DeviceNumber);
//
//             if (NT_SUCCESS(status) )
//             {
//                 errorLogEntry->NumberOfStrings++;
//
//                 if (resources.UsedMappingResourcesValid &&
//                     resources.AvailableMappingResourcesValid)
//                 {
//                     //
//                     // Add the used mapping resources to the insertion strings.
//                     //
//                     stringIndex += (wcslen(stringIndex) + 1);
//                     stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);
//
//                     status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.UsedMappingResources);
//
//                     if (NT_SUCCESS(status))
//                     {
//                         errorLogEntry->NumberOfStrings++;
//
//                         //
//                         // Add the available mapping resources to the insertion strings.
//                         //
//                         stringIndex += (wcslen(stringIndex) + 1);
//                         stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);
//
//                         status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.AvailableMappingResources);
//
//                         if (NT_SUCCESS(status))
//                         {
//                             errorLogEntry->NumberOfStrings++;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     TracePrint((TRACE_LEVEL_WARNING,
//                                 TRACE_FLAG_GENERAL,
//                                 "ClassLogThresholdEvent: DO (%p), Used and available mapping resources were unavailable.\n",
//                                 DeviceObject));
//                 }
//             }
//
//             //
//             // If we were able to successfully assemble all 3 insertion strings,
//             // then we can use one of the "extended" event IDs.  Otherwise, use the basic
//             // event ID, which only requires the disk number.
//             //
//             if (errorLogEntry->NumberOfStrings == 3)
//             {
//                 if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
//                     resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL;
//
//                 } else {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX;
//                 }
//             }
//             else
//             {
//                 errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED;
//             }
//
//             //
//             // Write the error log packet to the system error logging thread.
//             // It will be freed automatically.
//             //
//             IoWriteErrorLogEntry(errorLogEntry);
//
//             TracePrint((TRACE_LEVEL_INFORMATION,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Soft threshold notification logged.\n",
//                         DeviceObject));
//         }
//         else
//         {
//             TracePrint((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for error log entry.\n",
//                         DeviceObject));
//         }
//     } else {
//         TracePrint((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for SRB.\n",
//                         DeviceObject));
//     }
//
//
//     //
//     // Clear the soft threshold event pending flag so that another can be queued.
//     //
//     InterlockedExchange((PLONG)&(fdoExtension->FunctionSupportInfo->LBProvisioningData.SoftThresholdEventPending), 0);
//
//     ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);
//
//     FREE_POOL(srb);
//
//     if (workItem != NULL) {
//         IoFreeWorkItem(workItem);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "system" {
    fn ClassAcquireChildLock(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION);
    fn ClassReleaseChildLock(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION);
    fn NT_ASSERT(expr: BOOLEAN);
    fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PIRP);
}
