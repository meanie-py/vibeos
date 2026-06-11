#include "classp.h"
#include "debug.h"

// ---- RetryRequest ----
VOID
RetryRequest(
    PDEVICE_OBJECT DeviceObject,
    PIRP Irp,
    PSCSI_REQUEST_BLOCK Srb,
    BOOLEAN Associated,
    LONGLONG TimeDelta100ns
    )
{
    PIO_STACK_LOCATION currentIrpStack = IoGetCurrentIrpStackLocation(Irp);
    PIO_STACK_LOCATION nextIrpStack = IoGetNextIrpStackLocation(Irp);
    ULONG transferByteCount;
    ULONG dataTransferLength;
    PSTORAGE_REQUEST_BLOCK_HEADER srbHeader = (PSTORAGE_REQUEST_BLOCK_HEADER)Srb;

    // This function is obsolete but is still used by some of our class drivers.
    // TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "RetryRequest is OBSOLETE !"));

    //
    // Determine the transfer count of the request.  If this is a read or a
    // write then the transfer count is in the Irp stack.  Otherwise assume
    // the MDL contains the correct length.  If there is no MDL then the
    // transfer length must be zero.
    //

    dataTransferLength = SrbGetDataTransferLength(srbHeader);
    if (currentIrpStack->MajorFunction == IRP_MJ_READ ||
        currentIrpStack->MajorFunction == IRP_MJ_WRITE) {

        _Analysis_assume_(currentIrpStack->Parameters.Read.Length <= dataTransferLength);
        transferByteCount = currentIrpStack->Parameters.Read.Length;

    } else if (Irp->MdlAddress != NULL) {

        //
        // Note this assumes that only read and write requests are spilt and
        // other request do not need to be.  If the data buffer address in
        // the MDL and the SRB don't match then transfer length is most
        // likely incorrect.
        //

        NT_ASSERT(SrbGetDataBuffer(srbHeader) == MmGetMdlVirtualAddress(Irp->MdlAddress));
    _Analysis_assume_(Irp->MdlAddress->ByteCount <= dataTransferLength);
        transferByteCount = Irp->MdlAddress->ByteCount;

    } else {

        transferByteCount = 0;
    }

    //
    // this is a safety net.  this should not normally be hit, since we are
    // not guaranteed to be an fdoExtension
    //

    NT_ASSERT(!TEST_FLAG(SrbGetSrbFlags(srbHeader), SRB_FLAGS_FREE_SENSE_BUFFER));

    //
    // Reset byte count of transfer in SRB Extension.
    //

    SrbSetDataTransferLength(srbHeader, transferByteCount);

    //
    // Zero SRB statuses.
    //

    srbHeader->SrbStatus = 0;
    SrbSetScsiStatus(srbHeader, 0);

    //
    // If this is the last retry, then disable all the special flags.
    //

    if ( 0 == (ULONG)(ULONG_PTR)currentIrpStack->Parameters.Others.Argument4 ) {
        //
        // Set the no disconnect flag, disable synchronous data transfers and
        // disable tagged queuing. This fixes some errors.
        // NOTE: Cannot clear these flags, just add to them
        //

        SrbSetSrbFlags(srbHeader,
                          SRB_FLAGS_DISABLE_DISCONNECT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
        SrbClearSrbFlags(srbHeader, SRB_FLAGS_QUEUE_ACTION_ENABLE);

        SrbSetQueueTag(srbHeader, SP_UNTAGGED);
    }


    //
    // Set up major SCSI function.
    //

    nextIrpStack->MajorFunction = IRP_MJ_SCSI;

    //
    // Save SRB address in next stack for port driver.
    //

    nextIrpStack->Parameters.Scsi.Srb = Srb;

    if (Associated){
        IoSetCompletionRoutine(Irp, ClassIoCompleteAssociated, Srb, TRUE, TRUE, TRUE);
    }
    else {
        IoSetCompletionRoutine(Irp, ClassIoComplete, Srb, TRUE, TRUE, TRUE);
    }

    ClassRetryRequest(DeviceObject, Irp, TimeDelta100ns);
    return;
} // end RetryRequest()

// ---- InterpretSenseInfoWithoutHistory ----
BOOLEAN
InterpretSenseInfoWithoutHistory(
      PDEVICE_OBJECT Fdo,
      PIRP OriginalRequest,
      PSCSI_REQUEST_BLOCK Srb,
         UCHAR MajorFunctionCode,
         ULONG IoDeviceCode,
         ULONG PreviousRetryCount,
      NTSTATUS * Status,
       
         LONGLONG * RetryIn100nsUnits
    )
{
    PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
    PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
    LONGLONG tmpRetry = 0;
    BOOLEAN retry = FALSE;

    if (fdoData->InterpretSenseInfo != NULL)
    {
        SCSI_REQUEST_BLOCK tempSrb = {0};
        PSCSI_REQUEST_BLOCK srbPtr = Srb;

        // SAL annotations and ClassInitializeEx() both validate this
        NT_ASSERT(fdoData->InterpretSenseInfo->Interpret != NULL);

        //
        // If class driver does not support extended SRB and this is
        // an extended SRB, convert to legacy SRB and pass to class
        // driver.
        //
        if ((Srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) &&
            ((fdoExtension->CommonExtension.DriverExtension->SrbSupport &
              CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0)) {
            ClasspConvertToScsiRequestBlock(&tempSrb, (PSTORAGE_REQUEST_BLOCK)Srb);
            srbPtr = &tempSrb;
        }

        retry = fdoData->InterpretSenseInfo->Interpret(Fdo,
                                                       OriginalRequest,
                                                       srbPtr,
                                                       MajorFunctionCode,
                                                       IoDeviceCode,
                                                       PreviousRetryCount,
                                                       NULL,
                                                       Status,
                                                       &tmpRetry);
    }
    else
    {
        ULONG seconds = 0;

        retry = ClassInterpretSenseInfo(Fdo,
                                        Srb,
                                        MajorFunctionCode,
                                        IoDeviceCode,
                                        PreviousRetryCount,
                                        Status,
                                        &seconds);
        tmpRetry = ((LONGLONG)seconds) * 1000 * 1000 * 10;
    }


    if (RetryIn100nsUnits != NULL)
    {
        *RetryIn100nsUnits = tmpRetry;
    }
    return retry;
}

// ---- ClassFreeOrReuseSrb ----
VOID ClassFreeOrReuseSrb(     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
                                PSCSI_REQUEST_BLOCK Srb)


{
    PCOMMON_DEVICE_EXTENSION commonExt = &FdoExtension->CommonExtension;

    // This function is obsolete, but still called by DISK.SYS .
    // TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "ClassFreeOrReuseSrb is OBSOLETE !"));

    //
    // safety net.  this should never occur.  if it does, it's a potential
    // memory leak.
    //
    NT_ASSERT(!TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_FREE_SENSE_BUFFER));

    if (commonExt->IsSrbLookasideListInitialized){
        
        ClasspFreeSrb(FdoExtension, Srb);
    }
    else {
        TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,"ClassFreeOrReuseSrb: someone is trying to use an uninitialized SrbLookasideList !!!"));
        FREE_POOL(Srb);
    }
}

// ---- ClasspValidateOffloadSupported ----
 
 
 
NTSTATUS
ClasspValidateOffloadSupported(
      PDEVICE_OBJECT DeviceObject,
      PIRP Irp
    )



{
    PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
    NTSTATUS status;

    PAGED_CODE();

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                "ClasspValidateOffloadSupported (%p): Entering function. Irp %p\n",
                DeviceObject,
                Irp));

    fdoExt = DeviceObject->DeviceExtension;
    status = STATUS_SUCCESS;

    //
    // For now this command is only supported by disk devices
    //
    if ((DeviceObject->DeviceType == FILE_DEVICE_DISK) &&
        (!TEST_FLAG(DeviceObject->Characteristics, FILE_FLOPPY_DISKETTE))) {

        if (!fdoExt->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits) {

            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        "ClasspValidateOffloadSupported (%p): Command not supported on this disk device.\n",
                        DeviceObject));

            status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
            goto __ClasspValidateOffloadSupported_Exit;
        }

        if (!NT_SUCCESS(fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus)) {

            status = fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus;

            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        "ClasspValidateOffloadSupported (%p): VPD retrieval failed with %x.\n",
                        DeviceObject,
                        status));

            goto __ClasspValidateOffloadSupported_Exit;
        }
    } else {

        TracePrint((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    "ClasspValidateOffloadSupported (%p): Suported only on Disk devices.\n",
                    DeviceObject));

        status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        goto __ClasspValidateOffloadSupported_Exit;
    }

__ClasspValidateOffloadSupported_Exit:
    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                "ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n",
                DeviceObject,
                Irp,
                status));

    return status;
}

// ---- ClassRetrieveDeviceRelations ----
NTSTATUS
ClassRetrieveDeviceRelations(
      PDEVICE_OBJECT Fdo,
      DEVICE_RELATION_TYPE RelationType,
      PDEVICE_RELATIONS *DeviceRelations
    )
{
    PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;

    ULONG count = 0;
    ULONG i;

    PPHYSICAL_DEVICE_EXTENSION nextChild;

    ULONG relationsSize;
    PDEVICE_RELATIONS deviceRelations = NULL;
    PDEVICE_RELATIONS oldRelations = *DeviceRelations;

    NTSTATUS status;

    UNREFERENCED_PARAMETER(RelationType);

    PAGED_CODE();

    ClassAcquireChildLock(fdoExtension);

    nextChild = fdoExtension->CommonExtension.ChildList;

    //
    // Count the number of PDO's attached to this disk
    //

    while (nextChild != NULL) {
        PCOMMON_DEVICE_EXTENSION commonExtension;

        commonExtension = &(nextChild->CommonExtension);

        NT_ASSERTMSG("ClassPnp internal error: missing child on active list\n",
                  (nextChild->IsMissing == FALSE));

        nextChild = commonExtension->ChildList;

        count++;
    };

    //
    // If relations already exist in the QDR, adjust the current count
    // to include the previous list.
    //

    if (oldRelations) {
        count += oldRelations->Count;
    }

    relationsSize = (sizeof(DEVICE_RELATIONS) +
                     (count * sizeof(PDEVICE_OBJECT)));

    deviceRelations = ExAllocatePoolWithTag(PagedPool, relationsSize, '5BcS');

    if (deviceRelations == NULL) {

        TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_ENUM,  "ClassRetrieveDeviceRelations: unable to allocate "
                       "%d bytes for device relations\n", relationsSize));

        ClassReleaseChildLock(fdoExtension);

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(deviceRelations, relationsSize);

    if (oldRelations) {

        //
        // Copy the old relations to the new list and free the old list.
        //

        for (i = 0; i < oldRelations->Count; i++) {
            deviceRelations->Objects[i] = oldRelations->Objects[i];
        }

        FREE_POOL(oldRelations);
    }

    nextChild = fdoExtension->CommonExtension.ChildList;
    i = count;

    while (nextChild != NULL) {
        PCOMMON_DEVICE_EXTENSION commonExtension;

        commonExtension = &(nextChild->CommonExtension);

        NT_ASSERTMSG("ClassPnp internal error: missing child on active list\n",
                  (nextChild->IsMissing == FALSE));

    _Analysis_assume_(i >= 1);
    deviceRelations->Objects[--i] = nextChild->DeviceObject;

        status = ObReferenceObjectByPointer(
                    nextChild->DeviceObject,
                    0,
                    NULL,
                    KernelMode);
        if (!NT_SUCCESS(status)) {
            NT_ASSERT(!"Error referencing child device by pointer");
            TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_ENUM,
                        "ClassRetrieveDeviceRelations: Error referencing child "
                        "device %p by pointer\n", nextChild->DeviceObject));

        }
        nextChild->IsEnumerated = TRUE;
        nextChild = commonExtension->ChildList;
    }

    NT_ASSERTMSG("Child list has changed: ", i == 0);

    deviceRelations->Count = count;
    *DeviceRelations = deviceRelations;

    ClassReleaseChildLock(fdoExtension);
    return STATUS_SUCCESS;
} // end ClassRetrieveDeviceRelations()

// ---- ClasspGetBlockDeviceTokenLimitsInfo ----
 
NTSTATUS
ClasspGetBlockDeviceTokenLimitsInfo(
      PDEVICE_OBJECT DeviceObject
    )



{
    PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
    NTSTATUS status;
    PSCSI_REQUEST_BLOCK srb = NULL;
    ULONG srbSize = 0;
    USHORT pageLength = 0;
    USHORT descriptorLength = 0;
    PVOID dataBuffer = NULL;
    UCHAR bufferLength = VPD_MAX_BUFFER_SIZE;
    ULONG allocationBufferLength = bufferLength;
    PCDB cdb;
    ULONG dataTransferLength = 0;
    PVPD_THIRD_PARTY_COPY_PAGE operatingParameters = NULL;
    PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR blockDeviceTokenLimits = NULL;

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                "ClasspGetBlockDeviceTokenLimitsInfo (%p): Entering function.\n",
                DeviceObject));

    //
    // Allocate an SRB for querying the device for LBP-related info.
    //
    if ((fdoExtension->AdapterDescriptor != NULL) &&
        (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = sizeof(SCSI_REQUEST_BLOCK);
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, CLASSPNP_POOL_TAG_SRB);

    if (!srb) {

        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate SRB.\n",
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
    }

#if defined(_ARM_) || defined(_ARM64_)
    //
    // ARM has specific alignment requirements, although this will not have a functional impact on x86 or amd64
    // based platforms. We are taking the conservative approach here.
    //
    allocationBufferLength = ALIGN_UP_BY(allocationBufferLength, KeGetRecommendedSharedDataAlignment());
    dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, allocationBufferLength, CLASSPNP_POOL_TAG_VPD);
#else
    dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength, CLASSPNP_POOL_TAG_VPD);
#endif

    if (!dataBuffer) {

        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate dataBuffer.\n",
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
    }

    operatingParameters = (PVPD_THIRD_PARTY_COPY_PAGE)dataBuffer;

    RtlZeroMemory(dataBuffer, allocationBufferLength);

    if ((fdoExtension->AdapterDescriptor != NULL) &&
        (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
        status = InitializeStorageRequestBlock((PSTORAGE_REQUEST_BLOCK)srb,
                                               STORAGE_ADDRESS_TYPE_BTL8,
                                               CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
                                               1,
                                               SrbExDataTypeScsiCdb16);
        if (NT_SUCCESS(status)) {

            ((PSTORAGE_REQUEST_BLOCK)srb)->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;

        } else {

            //
            // Should not occur. Revert to legacy SRB.
            //
            NT_ASSERT(FALSE);

            TracePrint((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        "ClasspGetBlockDeviceTokenLimitsInfo (%p): Falling back to using SRB (instead of SRB_EX).\n",
                        DeviceObject));

            RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
            srb->Length = sizeof(SCSI_REQUEST_BLOCK);
            srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
        }
    } else {

        RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
        srb->Length = sizeof(SCSI_REQUEST_BLOCK);
        srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
    }

    SrbSetTimeOutValue(srb, fdoExtension->TimeOutValue);
    SrbSetRequestTag(srb, SP_UNTAGGED);
    SrbSetRequestAttribute(srb, SRB_SIMPLE_TAG_REQUEST);
    SrbAssignSrbFlags(srb, fdoExtension->SrbFlags);

    SrbSetCdbLength(srb, 6);

    cdb = SrbGetCdb(srb);
    cdb->CDB6INQUIRY3.OperationCode = SCSIOP_INQUIRY;
    cdb->CDB6INQUIRY3.EnableVitalProductData = 1;
    cdb->CDB6INQUIRY3.PageCode = VPD_THIRD_PARTY_COPY;
    cdb->CDB6INQUIRY3.AllocationLength = bufferLength;

    status = ClassSendSrbSynchronous(fdoExtension->DeviceObject,
                                     srb,
                                     dataBuffer,
                                     allocationBufferLength,
                                     FALSE);

    //
    // Handle the case where we get back STATUS_DATA_OVERRUN because the input
    // buffer was larger than necessary.
    //
    dataTransferLength = SrbGetDataTransferLength(srb);

    if (status == STATUS_DATA_OVERRUN && dataTransferLength < bufferLength) {

        status = STATUS_SUCCESS;
    }

    if (NT_SUCCESS(status)) {

        REVERSE_BYTES_SHORT(&pageLength, &operatingParameters->PageLength);

    } else {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    "ClasspGetBlockDeviceTokenLimitsInfo (%p): Inquiry for TPC VPD failed with %x.\n",
                    DeviceObject,
                    status));
    }

    if ((NT_SUCCESS(status)) &&
        (pageLength >= sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {

        USHORT descriptorType;

        blockDeviceTokenLimits = (PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR)operatingParameters->ThirdPartyCopyDescriptors;
        REVERSE_BYTES_SHORT(&descriptorType, &blockDeviceTokenLimits->DescriptorType);
        REVERSE_BYTES_SHORT(&descriptorLength, &blockDeviceTokenLimits->DescriptorLength);

        if ((descriptorType == BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR_TYPE_WINDOWS) &&
            (VPD_PAGE_HEADER_SIZE + descriptorLength == sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {

            fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = TRUE;

            REVERSE_BYTES_SHORT(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors, &blockDeviceTokenLimits->MaximumRangeDescriptors);
            REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer, &blockDeviceTokenLimits->MaximumInactivityTimer);
            REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer, &blockDeviceTokenLimits->DefaultInactivityTimer);
            REVERSE_BYTES_QUAD(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize, &blockDeviceTokenLimits->MaximumTokenTransferSize);
            REVERSE_BYTES_QUAD(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount, &blockDeviceTokenLimits->OptimalTransferCount);

            TracePrint((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_PNP,
                        "ClasspGetBlockDeviceTokenLimitsInfo (%p): %s %s (rev %s) reported following parameters: \
                        \n\t\t\tMaxRangeDescriptors: %u\n\t\t\tMaxIAT: %u\n\t\t\tDefaultIAT: %u \
                        \n\t\t\tMaxTokenTransferSize: %I64u\n\t\t\tOptimalTransferCount: %I64u\n\t\t\tOptimalTransferLengthGranularity: %u \
                        \n\t\t\tOptimalTransferLength: %u\n\t\t\tMaxTransferLength: %u\n",
                        DeviceObject,
                        (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->VendorIdOffset),
                        (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductIdOffset),
                        (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductRevisionOffset),
                        fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors,
                        fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer,
                        fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer,
                        fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize,
                        fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount,
                        fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLengthGranularity,
                        fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLength,
                        fdoExtension->FunctionSupportInfo->BlockLimitsData.MaximumTransferLength));
        } else {

            TracePrint((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        "ClasspGetBlockDeviceTokenLimitsInfo (%p): ThirdPartyCopy VPD data doesn't have Windows OffloadDataTransfer descriptor.\n",
                        DeviceObject));

            fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = FALSE;
            status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }
    } else {

        TracePrint((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_PNP,
                    "ClasspGetBlockDeviceTokenLimitsInfo (%p): TPC VPD data didn't return TPC descriptors of interest.\n",
                    DeviceObject));

        fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits = FALSE;
        status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
    }

__ClasspGetBlockDeviceTokenLimitsInfo_Exit:

    FREE_POOL(dataBuffer);
    FREE_POOL(srb);
    fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus = status;

    TracePrint((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                "ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n",
                DeviceObject,
                status));

    return status;
}

// ---- ClasspRefreshFunctionSupportInfo ----
NTSTATUS
ClasspRefreshFunctionSupportInfo(
      PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
      BOOLEAN ForceQuery
    )

{
    NTSTATUS            status;
    PSCSI_REQUEST_BLOCK srb = NULL;
    ULONG               srbSize;
    CLASS_VPD_B0_DATA   blockLimitsDataNew;
    PCLASS_VPD_B0_DATA  blockLimitsDataOriginal;
    KLOCK_QUEUE_HANDLE  lockHandle;
    ULONG               generationCount;
    ULONG               changeRequestCount;

    //
    // ChangeRequestCount is incremented every time we get an unit attention with
    // SCSI_ADSENSE_OPERATING_CONDITIONS_CHANGED.  GenerationCount will be set to
    // ChangeRequestCount after CLASS_FUNCTION_SUPPORT_INFO is refreshed with the latest
    // VPD data.  i.e.  if both values are the same, data in
    // CLASS_FUNCTION_SUPPORT_INFO is current
    //

    generationCount = FdoExtension->FunctionSupportInfo->GenerationCount;
    changeRequestCount = FdoExtension->FunctionSupportInfo->ChangeRequestCount;
    if (!ForceQuery && generationCount == changeRequestCount) {
        return STATUS_SUCCESS;
    }

    //
    // Allocate an SRB for querying the device for LBP-related info if either
    // the Logical Block Provisioning (0xB2) or Block Limits (0xB0) VPD page
    // exists.
    //
    if ((FdoExtension->AdapterDescriptor != NULL) &&
        (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = sizeof(SCSI_REQUEST_BLOCK);
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, '1DcS');
    if (srb == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ClasspDeviceGetBlockLimitsVPDPage(FdoExtension,
                                               srb,
                                               srbSize,
                                               &blockLimitsDataNew);

    if (NT_SUCCESS(status)) {

        KeAcquireInStackQueuedSpinLock(&FdoExtension->FunctionSupportInfo->SyncLock, &lockHandle);

        //
        // If the generationCount didn't change since we looked at it last time, it means
        // no one has tried to update the CLASS_FUNCTION_SUPPORT_INFO data; otherwise, someone
        // else has beat us to it.
        //
        if (generationCount == FdoExtension->FunctionSupportInfo->GenerationCount) {

            blockLimitsDataOriginal = &FdoExtension->FunctionSupportInfo->BlockLimitsData;
            if (blockLimitsDataOriginal->CommandStatus == -1) {
                //
                // CommandStatus == -1 means this is the first time we have
                // gotten the block limits data.
                //
                *blockLimitsDataOriginal = blockLimitsDataNew;
            } else {
                //
                // We only expect the Optimal Unmap Granularity (and alignment)
                // to change, so those are the only parameters we update.
                //
                blockLimitsDataOriginal->UGAVALID = blockLimitsDataNew.UGAVALID;
                blockLimitsDataOriginal->UnmapGranularityAlignment = blockLimitsDataNew.UnmapGranularityAlignment;
                blockLimitsDataOriginal->OptimalUnmapGranularity = blockLimitsDataNew.OptimalUnmapGranularity;
            }
            FdoExtension->FunctionSupportInfo->GenerationCount = changeRequestCount;
        }

        KeReleaseInStackQueuedSpinLock(&lockHandle);
    }

    FREE_POOL(srb);
    return status;
}

// ---- ClasspMediaChangeRegistryCallBack ----
 
 
 
NTSTATUS
NTAPI 
ClasspMediaChangeRegistryCallBack(
      PWSTR ValueName,
      ULONG ValueType,
      PVOID ValueData,
      ULONG ValueLength,
      PVOID Context,
      PVOID EntryContext
    )
{
    PULONG valueFound;
    PUNICODE_STRING deviceString;
    PWSTR keyValue;

    PAGED_CODE();
    UNREFERENCED_PARAMETER(ValueName);

    if (ValueData == NULL ||
        Context == NULL ||
        EntryContext == NULL) {
        return STATUS_INVALID_PARAMETER;
    }

    //
    // if we have already set the value to true, exit
    //

    valueFound = EntryContext;
    if ((*valueFound) != 0) {
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspMcnRegCB: already set to true\n"));
        return STATUS_SUCCESS;
    }

    if (ValueLength == sizeof(WCHAR)) {
        TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN, "ClasspMcnRegCB: NULL string should "
                    "never be passed to registry call-back!\n"));
        return STATUS_SUCCESS;
    }


    //
    // if the data is not a terminated string, exit
    //

    if (ValueType != REG_SZ) {
        return STATUS_SUCCESS;
    }

    deviceString = Context;
    keyValue = ValueData;
    ValueLength -= sizeof(WCHAR); // ignore the null character

    //
    // do not compare more memory than is in deviceString
    //

    if (ValueLength > deviceString->Length) {
        ValueLength = deviceString->Length;
    }

    //
    // if the strings match, disable autorun
    //

    if (RtlCompareMemory(deviceString->Buffer, keyValue, ValueLength) == ValueLength) {
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: Match found\n"));
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: DeviceString at %p\n",
                    deviceString->Buffer));
        TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: KeyValue at %p\n",
                    keyValue));
        (*valueFound) = TRUE;
    }

    return STATUS_SUCCESS;
} // end ClasspMediaChangeRegistryCallBack()

// ---- ClassLogResourceExhaustionEvent ----
 
 
 
VOID
NTAPI 
ClassLogResourceExhaustionEvent(
      PDEVICE_OBJECT DeviceObject,
      PVOID Context
    )

{
    PIO_WORKITEM workItem = (PIO_WORKITEM)Context;

    if (NT_SUCCESS(ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_ERROR_DISK_RESOURCES_EXHAUSTED))) {

        TracePrint((TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_GENERAL,
                    "ClassLogResourceExhaustionEvent: DO (%p), Permanent resource exhaustion logged.\n",
                    DeviceObject));
    } else {
        TracePrint((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    "ClassLogResourceExhaustionEvent: DO (%p), Failed to allocate memory for error log entry.\n",
                    DeviceObject));
    }


    ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);

    if (workItem != NULL) {
        IoFreeWorkItem(workItem);
    }
}

// ---- ClassLogThresholdEvent ----
 
 
 
VOID
NTAPI 
ClassLogThresholdEvent(
      PDEVICE_OBJECT DeviceObject,
      PVOID Context
    )

{
    NTSTATUS status = STATUS_SUCCESS;
    PIO_WORKITEM workItem = (PIO_WORKITEM)Context;
    PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
    PSCSI_REQUEST_BLOCK srb = NULL;
    STORAGE_LB_PROVISIONING_MAP_RESOURCES resources = {0};
    ULONG resourcesSize = sizeof(STORAGE_LB_PROVISIONING_MAP_RESOURCES);
    PIO_ERROR_LOG_PACKET errorLogEntry = NULL;
    ULONG logEntrySize = sizeof(IO_ERROR_LOG_PACKET);
    PWCHAR stringIndex = NULL;
    LONG stringSize = 0;
    ULONG srbSize;

    //
    // Allocate an SRB for getting the LBP log page.
    //
    if ((fdoExtension->AdapterDescriptor != NULL) &&
        (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = sizeof(SCSI_REQUEST_BLOCK);
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx,
                                srbSize,
                                'ACcS');
    if (srb != NULL) {

        //
        // Try to get the LBP resources from the device so we can report them in
        // the system event log.
        //
        ClassGetLBProvisioningResources(DeviceObject,
                                        srb,
                                        resourcesSize,
                                        &resources);

        //
        // We need to allocate enough space for 3 insertion strings:
        // The first is a ULONG representing the disk number in decimal, which means
        // a max of 10 digits, plus one for the NULL character.
        // The second and third are ULONGLONGs representing the used and available
        // bytes, which means a max of 20 digits, plus one for the NULL character.
        // Make sure we do not exceed the max error log size or the max size of a
        // UCHAR since the size gets truncated to a UCHAR when we pass it to
        // IoAllocateErrorLogEntry().
        //
        logEntrySize = sizeof(IO_ERROR_LOG_PACKET) + (11 * sizeof(WCHAR)) + (2 * (21 * sizeof(WCHAR)));
        logEntrySize = min(logEntrySize, ERROR_LOG_MAXIMUM_SIZE);
        logEntrySize = min(logEntrySize, MAXUCHAR);

        errorLogEntry = (PIO_ERROR_LOG_PACKET)IoAllocateErrorLogEntry(DeviceObject, (UCHAR)logEntrySize);
        if (errorLogEntry != NULL)
        {
            //
            // There are two event IDs we can use here.  Both use the disk number,
            // but one reports the available and used bytes while the other does not.
            // We fall back on the latter if we failed to obtain the available and
            // used byte counts from the LBP log page.
            //
            // The event insertion strings need to be in this order:
            // 1. The disk number. (Both event IDs use this.)
            // 2. Bytes used.
            // 3. Bytes available.
            //

            RtlZeroMemory(errorLogEntry, logEntrySize);
            errorLogEntry->StringOffset = sizeof(IO_ERROR_LOG_PACKET);

            stringIndex = (PWCHAR)((ULONG_PTR)errorLogEntry + sizeof(IO_ERROR_LOG_PACKET));
            stringSize = logEntrySize - sizeof(IO_ERROR_LOG_PACKET);

            //
            // Add the disk number to the insertion strings.
            //
            status = RtlStringCbPrintfW(stringIndex, stringSize, L"%d", fdoExtension->DeviceNumber);

            if (NT_SUCCESS(status) )
            {
                errorLogEntry->NumberOfStrings++;

                if (resources.UsedMappingResourcesValid &&
                    resources.AvailableMappingResourcesValid)
                {
                    //
                    // Add the used mapping resources to the insertion strings.
                    //
                    stringIndex += (wcslen(stringIndex) + 1);
                    stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);

                    status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.UsedMappingResources);

                    if (NT_SUCCESS(status))
                    {
                        errorLogEntry->NumberOfStrings++;

                        //
                        // Add the available mapping resources to the insertion strings.
                        //
                        stringIndex += (wcslen(stringIndex) + 1);
                        stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);

                        status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.AvailableMappingResources);

                        if (NT_SUCCESS(status))
                        {
                            errorLogEntry->NumberOfStrings++;
                        }
                    }
                }
                else
                {
                    TracePrint((TRACE_LEVEL_WARNING,
                                TRACE_FLAG_GENERAL,
                                "ClassLogThresholdEvent: DO (%p), Used and available mapping resources were unavailable.\n",
                                DeviceObject));
                }
            }

            //
            // If we were able to successfully assemble all 3 insertion strings,
            // then we can use one of the "extended" event IDs.  Otherwise, use the basic
            // event ID, which only requires the disk number.
            //
            if (errorLogEntry->NumberOfStrings == 3)
            {
                if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
                    resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {

                    errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN;

                } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
                           resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {

                    errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL;

                } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
                           resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {

                    errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN;

                } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
                           resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {

                    errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL;

                } else {

                    errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX;
                }
            }
            else
            {
                errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED;
            }

            //
            // Write the error log packet to the system error logging thread.
            // It will be freed automatically.
            //
            IoWriteErrorLogEntry(errorLogEntry);

            TracePrint((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_GENERAL,
                        "ClassLogThresholdEvent: DO (%p), Soft threshold notification logged.\n",
                        DeviceObject));
        }
        else
        {
            TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for error log entry.\n",
                        DeviceObject));
        }
    } else {
        TracePrint((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for SRB.\n",
                        DeviceObject));
    }


    //
    // Clear the soft threshold event pending flag so that another can be queued.
    //
    InterlockedExchange((PLONG)&(fdoExtension->FunctionSupportInfo->LBProvisioningData.SoftThresholdEventPending), 0);

    ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);

    FREE_POOL(srb);

    if (workItem != NULL) {
        IoFreeWorkItem(workItem);
    }
}
