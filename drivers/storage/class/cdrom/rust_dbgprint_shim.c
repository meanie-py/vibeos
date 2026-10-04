#include <ntddk.h>
#include "cdrom.h"
#include "ioctl.h"
#include "scratch.h"

ULONG CdromDbgPrint0(const char *Format);
ULONG CdromDbgPrint1(const char *Format, ULONG_PTR Arg1);
ULONG CdromDbgPrint2(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2);
ULONG CdromDbgPrint3(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2, ULONG_PTR Arg3);
ULONG CdromDbgPrint4(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2, ULONG_PTR Arg3, ULONG_PTR Arg4);
ULONG CdromDbgPrint5(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2, ULONG_PTR Arg3, ULONG_PTR Arg4, ULONG_PTR Arg5);

VOID
RequestCompletion(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    NTSTATUS Status,
    ULONG_PTR Information);

VOID
CdromRequestCompletion(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    NTSTATUS Status,
    ULONG_PTR Information);

NTSTATUS
__cdecl
CdromRustRequestValidateReadWrite(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters) __asm__("_RequestValidateReadWrite");

NTSTATUS
__cdecl
CdromRustRequestHandleReadWrite(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters) __asm__("_RequestHandleReadWrite");

VOID
__cdecl
NormalizeIoctl(
    PWDF_REQUEST_PARAMETERS RequestParameters);

VOID
CdromPrepareRequestContext(
    WDFREQUEST Request,
    PCDROM_DEVICE_EXTENSION DeviceExtension);

VOID
RequestProcessInternalDeviceControl(
    WDFREQUEST Request,
    PCDROM_DEVICE_EXTENSION DeviceExtension);

BOOLEAN
CdromTryHandleStorageControl(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters);

VOID
NTAPI
CdromRustScratchBufferResetItems(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
#if defined(_M_IX86)
    BOOLEAN ResetRequestHistory) __asm__("_ScratchBuffer_ResetItems@8");
#else
    BOOLEAN ResetRequestHistory) __asm__("ScratchBuffer_ResetItems");
#endif

BOOLEAN
NTAPI
CdromRustScratchBufferAllocate(
#if defined(_M_IX86)
    PCDROM_DEVICE_EXTENSION DeviceExtension) __asm__("_ScratchBuffer_Allocate@4");
#else
    PCDROM_DEVICE_EXTENSION DeviceExtension) __asm__("ScratchBuffer_Allocate");
#endif

EVT_WDF_REQUEST_COMPLETION_ROUTINE ScratchBuffer_ReadWriteCompletionRoutine;

NTSTATUS
NTAPI
CdromRustScratchBufferPerformNextReadWrite(
#if defined(_M_IX86)
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    BOOLEAN FirstTry) __asm__("_ScratchBuffer_PerformNextReadWrite@8");
#else
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    BOOLEAN FirstTry) __asm__("ScratchBuffer_PerformNextReadWrite");
#endif

NTSTATUS
RequestSend(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    WDFIOTARGET IoTarget,
    ULONG Flags,
    PBOOLEAN RequestSent);

NTSTATUS
CdromRustScratchAllocatePartialMdl(
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    NTSTATUS Status;
    ULONG TransferLength = 0;

    Status = RtlULongAdd(
        DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes,
        PAGE_SIZE,
        &TransferLength);
    if (!NT_SUCCESS(Status))
    {
        return STATUS_INTEGER_OVERFLOW;
    }

    DeviceExtension->ScratchContext.PartialMdlIsBuilt = FALSE;
    DeviceExtension->ScratchContext.PartialMdl =
        IoAllocateMdl(NULL, TransferLength, FALSE, FALSE, NULL);
    if (DeviceExtension->ScratchContext.PartialMdl == NULL)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    return STATUS_SUCCESS;
}

extern WDF_OBJECT_CONTEXT_TYPE_INFO CDROM_REQUEST_CONTEXT_TYPE_INFO;
extern WDF_OBJECT_CONTEXT_TYPE_INFO CDROM_DEVICE_EXTENSION_TYPE_INFO;
extern WDF_OBJECT_CONTEXT_TYPE_INFO FILE_OBJECT_CONTEXT_TYPE_INFO;

static
PCDROM_REQUEST_CONTEXT
CdromGetRequestContext(
    WDFREQUEST Request)
{
    return (PCDROM_REQUEST_CONTEXT)WdfObjectGetTypedContextWorker(
        Request,
        CDROM_REQUEST_CONTEXT_TYPE_INFO.UniqueType);
}

static
PCDROM_DEVICE_EXTENSION
CdromGetDeviceExtension(
    WDFDEVICE Device)
{
    return (PCDROM_DEVICE_EXTENSION)WdfObjectGetTypedContextWorker(
        Device,
        CDROM_DEVICE_EXTENSION_TYPE_INFO.UniqueType);
}

static
PFILE_OBJECT_CONTEXT
CdromGetFileObjectContext(
    WDFFILEOBJECT FileObject)
{
    return (PFILE_OBJECT_CONTEXT)WdfObjectGetTypedContextWorker(
        FileObject,
        FILE_OBJECT_CONTEXT_TYPE_INFO.UniqueType);
}

static
ULONG
CdromGetBlockLength(
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    return DeviceExtension->DiskGeometry.BytesPerSector != 0 ?
           DeviceExtension->DiskGeometry.BytesPerSector :
           2048;
}

static
ULONGLONG
CdromGetBlockCount(
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    ULONG BlockLength = CdromGetBlockLength(DeviceExtension);

    if (BlockLength == 0)
    {
        return 0;
    }

    return (ULONGLONG)(DeviceExtension->PartitionLength.QuadPart / BlockLength);
}

static
VOID
CdromStoreTrackAddress(
    PTRACK_DATA TrackData,
    ULONGLONG Lba,
    BOOLEAN Msf)
{
    RtlZeroMemory(TrackData->Address, sizeof(TrackData->Address));

    if (Msf)
    {
        ULONGLONG Frames = Lba + 150;
        TrackData->Address[1] = (UCHAR)(Frames / (60 * 75));
        TrackData->Address[2] = (UCHAR)((Frames / 75) % 60);
        TrackData->Address[3] = (UCHAR)(Frames % 75);
    }
    else
    {
        TrackData->Address[0] = (UCHAR)(Lba >> 24);
        TrackData->Address[1] = (UCHAR)(Lba >> 16);
        TrackData->Address[2] = (UCHAR)(Lba >> 8);
        TrackData->Address[3] = (UCHAR)Lba;
    }
}

static
VOID
CdromFillTrackData(
    PTRACK_DATA TrackData,
    UCHAR TrackNumber,
    ULONGLONG Lba,
    BOOLEAN Msf)
{
    RtlZeroMemory(TrackData, sizeof(*TrackData));
    TrackData->Adr = 1;
    TrackData->Control = 4;
    TrackData->TrackNumber = TrackNumber;
    CdromStoreTrackAddress(TrackData, Lba, Msf);
}

static
NTSTATUS
CdromCompleteToc(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    ULONG OutputBufferLength,
    BOOLEAN Msf,
    ULONG_PTR *Information)
{
    PCDROM_TOC Toc = NULL;
    NTSTATUS Status;
    ULONG_PTR Bytes = FIELD_OFFSET(CDROM_TOC, TrackData[2]);

    Status = WdfRequestRetrieveOutputBuffer(Request, Bytes, &Toc, NULL);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    if (OutputBufferLength < Bytes)
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    RtlZeroMemory(Toc, Bytes);
    Toc->Length[0] = 0;
    Toc->Length[1] = (UCHAR)(Bytes - RTL_FIELD_SIZE(CDROM_TOC, Length));
    Toc->FirstTrack = 1;
    Toc->LastTrack = 1;
    CdromFillTrackData(&Toc->TrackData[0], 1, 0, Msf);
    CdromFillTrackData(&Toc->TrackData[1], 0xAA, CdromGetBlockCount(DeviceExtension), Msf);

    *Information = Bytes;
    return STATUS_SUCCESS;
}

static
NTSTATUS
CdromCompleteLastSession(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    BOOLEAN Msf,
    ULONG_PTR *Information)
{
    PCDROM_TOC_SESSION_DATA SessionData = NULL;
    NTSTATUS Status;

    Status = WdfRequestRetrieveOutputBuffer(
        Request,
        sizeof(CDROM_TOC_SESSION_DATA),
        &SessionData,
        NULL);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    RtlZeroMemory(SessionData, sizeof(*SessionData));
    SessionData->Length[0] = 0;
    SessionData->Length[1] = sizeof(CDROM_TOC_SESSION_DATA) -
                             RTL_FIELD_SIZE(CDROM_TOC_SESSION_DATA, Length);
    SessionData->FirstCompleteSession = 1;
    SessionData->LastCompleteSession = 1;
    CdromFillTrackData(&SessionData->TrackData[0], 1, 0, Msf);

    UNREFERENCED_PARAMETER(DeviceExtension);
    *Information = sizeof(CDROM_TOC_SESSION_DATA);
    return STATUS_SUCCESS;
}

static
NTSTATUS
CdromCompleteMinimalTocEx(
    WDFREQUEST Request,
    ULONG_PTR *Information)
{
    PUCHAR OutputBuffer = NULL;
    NTSTATUS Status;

    Status = WdfRequestRetrieveOutputBuffer(
        Request,
        MINIMUM_CDROM_READ_TOC_EX_SIZE,
        &OutputBuffer,
        NULL);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    OutputBuffer[0] = 0;
    OutputBuffer[1] = 0;
    *Information = MINIMUM_CDROM_READ_TOC_EX_SIZE;
    return STATUS_SUCCESS;
}

NTSTATUS
CdromCRequestHandleMountQueryUniqueId(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    WDF_REQUEST_PARAMETERS RequestParameters,
    size_t *DataLength)
{
    NTSTATUS Status = STATUS_SUCCESS;
    PMOUNTDEV_UNIQUE_ID UniqueId = NULL;

    *DataLength = 0;

    if (DeviceExtension->MountedDeviceInterfaceName.Buffer == NULL)
    {
        Status = STATUS_INVALID_PARAMETER;
    }
    else if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
             sizeof(MOUNTDEV_UNIQUE_ID))
    {
        *DataLength = sizeof(MOUNTDEV_UNIQUE_ID);
        Status = STATUS_BUFFER_TOO_SMALL;
    }

    if (NT_SUCCESS(Status))
    {
        Status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.OutputBufferLength,
            &UniqueId,
            NULL);
    }

    if (NT_SUCCESS(Status))
    {
        RtlZeroMemory(
            UniqueId,
            RequestParameters.Parameters.DeviceIoControl.OutputBufferLength);

        UniqueId->UniqueIdLength = DeviceExtension->MountedDeviceInterfaceName.Length;

        if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
            (sizeof(USHORT) + DeviceExtension->MountedDeviceInterfaceName.Length))
        {
            *DataLength = sizeof(MOUNTDEV_UNIQUE_ID);
            Status = STATUS_BUFFER_OVERFLOW;
        }
    }

    if (NT_SUCCESS(Status))
    {
        RtlCopyMemory(
            UniqueId->UniqueId,
            DeviceExtension->MountedDeviceInterfaceName.Buffer,
            UniqueId->UniqueIdLength);

        *DataLength = sizeof(USHORT) + UniqueId->UniqueIdLength;
    }

    return Status;
}

NTSTATUS
CdromCRequestHandleMountQueryDeviceName(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    WDF_REQUEST_PARAMETERS RequestParameters,
    size_t *DataLength)
{
    NTSTATUS Status = STATUS_SUCCESS;
    PMOUNTDEV_NAME Name = NULL;

    *DataLength = 0;

    if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
        sizeof(MOUNTDEV_NAME))
    {
        Status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = sizeof(MOUNTDEV_NAME);
    }

    if (NT_SUCCESS(Status))
    {
        Status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.OutputBufferLength,
            &Name,
            NULL);
    }

    if (NT_SUCCESS(Status))
    {
        RtlZeroMemory(
            Name,
            RequestParameters.Parameters.DeviceIoControl.OutputBufferLength);
        Name->NameLength = DeviceExtension->DeviceName.Length;

        if (RequestParameters.Parameters.DeviceIoControl.OutputBufferLength <
            (sizeof(USHORT) + DeviceExtension->DeviceName.Length))
        {
            Status = STATUS_BUFFER_OVERFLOW;
            *DataLength = sizeof(MOUNTDEV_NAME);
        }
    }

    if (NT_SUCCESS(Status))
    {
        RtlCopyMemory(
            Name->Name,
            DeviceExtension->DeviceName.Buffer,
            Name->NameLength);

        *DataLength = sizeof(USHORT) + Name->NameLength;
    }

    return Status;
}

NTSTATUS
CdromCRequestHandleMountQuerySuggestedLinkName(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    WDF_REQUEST_PARAMETERS RequestParameters,
    size_t *DataLength)
{
    NTSTATUS Status = STATUS_NOT_FOUND;

    UNREFERENCED_PARAMETER(DeviceExtension);
    UNREFERENCED_PARAMETER(Request);
    UNREFERENCED_PARAMETER(RequestParameters);

    *DataLength = 0;
    return Status;
}

static
BOOLEAN
CdromTryHandleDirectDeviceControl(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters)
{
    NTSTATUS Status;
    size_t DataLength = 0;

    switch (RequestParameters->Parameters.DeviceIoControl.IoControlCode)
    {
        case IOCTL_MOUNTDEV_QUERY_UNIQUE_ID:
            Status = CdromCRequestHandleMountQueryUniqueId(
                DeviceExtension,
                Request,
                *RequestParameters,
                &DataLength);
            CdromRequestCompletion(DeviceExtension, Request, Status, DataLength);
            return TRUE;

        case IOCTL_MOUNTDEV_QUERY_DEVICE_NAME:
            Status = CdromCRequestHandleMountQueryDeviceName(
                DeviceExtension,
                Request,
                *RequestParameters,
                &DataLength);
            CdromRequestCompletion(DeviceExtension, Request, Status, DataLength);
            return TRUE;

        case IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME:
            Status = CdromCRequestHandleMountQuerySuggestedLinkName(
                DeviceExtension,
                Request,
                *RequestParameters,
                &DataLength);
            CdromRequestCompletion(DeviceExtension, Request, Status, DataLength);
            return TRUE;

        default:
            return FALSE;
    }
}

VOID
NTAPI
DeviceEvtIoInCallerContext(
    WDFDEVICE Device,
    WDFREQUEST Request)
{
    NTSTATUS Status;
    PCDROM_DEVICE_EXTENSION DeviceExtension = CdromGetDeviceExtension(Device);
    WDF_REQUEST_PARAMETERS RequestParameters;

    CdromPrepareRequestContext(Request, DeviceExtension);

    WDF_REQUEST_PARAMETERS_INIT(&RequestParameters);
    WdfRequestGetParameters(Request, &RequestParameters);

    if (RequestParameters.Type == WdfRequestTypeDeviceControl)
    {
        NormalizeIoctl(&RequestParameters);

        if (CdromTryHandleDirectDeviceControl(
                DeviceExtension,
                Request,
                &RequestParameters))
        {
            return;
        }

        if (CdromTryHandleStorageControl(
                DeviceExtension,
                Request,
                &RequestParameters))
        {
            return;
        }

        Status = WdfDeviceEnqueueRequest(Device, Request);
        if (!NT_SUCCESS(Status))
        {
            CdromRequestCompletion(
                DeviceExtension,
                Request,
                Status,
                WdfRequestGetInformation(Request));
        }
    }
    else if (RequestParameters.Type == WdfRequestTypeDeviceControlInternal)
    {
        RequestProcessInternalDeviceControl(Request, DeviceExtension);
    }
    else
    {
        Status = WdfDeviceEnqueueRequest(Device, Request);
        if (!NT_SUCCESS(Status))
        {
            CdromRequestCompletion(
                DeviceExtension,
                Request,
                Status,
                WdfRequestGetInformation(Request));
        }
    }
}

static
NTSTATUS
CdromEnsurePartialMdl(
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    ULONG TransferLength;

    if (DeviceExtension->ScratchContext.PartialMdl != NULL)
    {
        return STATUS_SUCCESS;
    }

    if (DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes >
        (MAXULONG - PAGE_SIZE))
    {
        return STATUS_INTEGER_OVERFLOW;
    }

    TransferLength =
        DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes + PAGE_SIZE;

    DeviceExtension->ScratchContext.PartialMdlIsBuilt = FALSE;
    DeviceExtension->ScratchContext.PartialMdl = IoAllocateMdl(
        NULL,
        TransferLength,
        FALSE,
        FALSE,
        NULL);

    DbgPrint("[SCR-C] ensure partial len=%x mdl=%p\n",
             TransferLength,
             DeviceExtension->ScratchContext.PartialMdl);

    return DeviceExtension->ScratchContext.PartialMdl != NULL ?
           STATUS_SUCCESS :
           STATUS_INSUFFICIENT_RESOURCES;
}

static
NTSTATUS
CdromScratchBufferBeginUse(
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    PVOID Tmp;
    PIRP Irp;
    WDF_REQUEST_REUSE_PARAMS ReuseParams;
    NTSTATUS Status;

    if (DeviceExtension->ScratchContext.ScratchRequest == NULL)
    {
        BOOLEAN Allocated = CdromRustScratchBufferAllocate(DeviceExtension);
        DbgPrint("[SCR-C] allocate=%x req=%p buf=%p srb=%p\n",
                 Allocated,
                 DeviceExtension->ScratchContext.ScratchRequest,
                 DeviceExtension->ScratchContext.ScratchBuffer,
                 DeviceExtension->ScratchContext.ScratchSrb);
        if (!Allocated)
        {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = CdromEnsurePartialMdl(DeviceExtension);
        if (!NT_SUCCESS(Status))
        {
            return Status;
        }
    }

    Tmp = InterlockedCompareExchangePointer(
        (PVOID)&DeviceExtension->ScratchContext.ScratchInUse,
        (PVOID)-1,
        NULL);
    NT_ASSERT(Tmp == NULL);
    UNREFERENCED_PARAMETER(Tmp);

    DeviceExtension->ScratchContext.ScratchInUseFileName = __FILE__;
    DeviceExtension->ScratchContext.ScratchInUseLineNumber = __LINE__;

    Irp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);

    if (DeviceExtension->ScratchContext.ScratchHistory != NULL)
    {
        PSRB_HISTORY History = DeviceExtension->ScratchContext.ScratchHistory;
        RtlZeroMemory(
            History->History,
            sizeof(SRB_HISTORY_ITEM) * History->TotalHistoryCount);
        RtlZeroMemory(History->ClassDriverUse, sizeof(History->ClassDriverUse));
        History->UsedHistoryCount = 0;
    }

    if (Irp != NULL)
    {
        Irp->MdlAddress = NULL;
    }

    WDF_REQUEST_REUSE_PARAMS_INIT(
        &ReuseParams,
        WDF_REQUEST_REUSE_NO_FLAGS,
        STATUS_NOT_SUPPORTED);

    Status = WdfRequestReuse(
        DeviceExtension->ScratchContext.ScratchRequest,
        &ReuseParams);

    if (NT_SUCCESS(Status))
    {
        WdfRequestSetCompletionRoutine(
            DeviceExtension->ScratchContext.ScratchRequest,
            NULL,
            NULL);

        Status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            DeviceExtension->IoTarget,
            DeviceExtension->ScratchContext.ScratchRequest,
            IOCTL_SCSI_EXECUTE_IN,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL);
    }

    if (!NT_SUCCESS(Status))
    {
        DbgPrint("[SCR-C] reset failed st=%x\n", Status);
        InterlockedExchangePointer(
            (PVOID)&DeviceExtension->ScratchContext.ScratchInUse,
            NULL);
        return Status;
    }

    if (DeviceExtension->ScratchContext.ScratchSense == NULL ||
        DeviceExtension->ScratchContext.ScratchSrb == NULL)
    {
        InterlockedExchangePointer(
            (PVOID)&DeviceExtension->ScratchContext.ScratchInUse,
            NULL);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = CdromEnsurePartialMdl(DeviceExtension);
    if (!NT_SUCCESS(Status))
    {
        InterlockedExchangePointer(
            (PVOID)&DeviceExtension->ScratchContext.ScratchInUse,
            NULL);
        return Status;
    }

    RtlZeroMemory(DeviceExtension->ScratchContext.ScratchSense, sizeof(SENSE_DATA));
    RtlZeroMemory(DeviceExtension->ScratchContext.ScratchSrb, sizeof(SCSI_REQUEST_BLOCK));

    {
        PCDROM_REQUEST_CONTEXT RequestContext =
            CdromGetRequestContext(DeviceExtension->ScratchContext.ScratchRequest);
        if (RequestContext != NULL)
        {
            RequestContext->TimeSentDownFirstTime.QuadPart = 0;
            RequestContext->TimeSentDownLasttTime.QuadPart = 0;
        }
    }

    return STATUS_SUCCESS;
}

static
NTSTATUS
CdromValidateReadWrite(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters)
{
    NTSTATUS Status = STATUS_SUCCESS;
    PCDROM_DATA CdData = &DeviceExtension->DeviceAdditionalData;
    BOOLEAN IsValid = TRUE;
    LONGLONG StartingOffset;
    size_t TransferByteCount;
    FEATURE_NUMBER Schema;

    if (RequestParameters->Type == WdfRequestTypeRead)
    {
        StartingOffset = RequestParameters->Parameters.Read.DeviceOffset;
        TransferByteCount = RequestParameters->Parameters.Read.Length;
    }
    else
    {
        StartingOffset = RequestParameters->Parameters.Write.DeviceOffset;
        TransferByteCount = RequestParameters->Parameters.Write.Length;
    }

    if (DeviceExtension->DiskGeometry.BytesPerSector == 0)
    {
        DeviceExtension->DiskGeometry.BytesPerSector = 2048;
    }

    if (DeviceExtension->SectorShift == 0)
    {
        DeviceExtension->SectorShift = 11;
    }

    if (DeviceExtension->PartitionLength.QuadPart == 0)
    {
        DeviceExtension->PartitionLength.QuadPart = (LONGLONG)0x7fffffff;
    }

    if (((ULONGLONG)StartingOffset & (DeviceExtension->DiskGeometry.BytesPerSector - 1)) ||
        ((ULONGLONG)TransferByteCount & (DeviceExtension->DiskGeometry.BytesPerSector - 1)))
    {
        Status = STATUS_INVALID_DEVICE_REQUEST;
    }

    if (NT_SUCCESS(Status))
    {
        Schema = CdData->Mmc.ValidationSchema;

        if (RequestParameters->Type == WdfRequestTypeRead)
        {
            if (!CdData->Mmc.WriteAllowed)
            {
                Schema = FeatureRandomWritable;
            }
            else if (Schema != FeatureIncrementalStreamingWritable)
            {
                Schema = FeatureRandomWritable;
            }
        }

        if (RequestParameters->Type == WdfRequestTypeWrite && !CdData->Mmc.WriteAllowed)
        {
            IsValid = FALSE;
        }

        if (IsValid)
        {
            switch (Schema)
            {
                case FeatureDefectManagement:
                case FeatureRandomWritable:
                    if (!CdData->Mmc.IsWriter)
                    {
                        if (StartingOffset < 0 ||
                            StartingOffset >= DeviceExtension->PartitionLength.QuadPart ||
                            (ULONGLONG)TransferByteCount >
                                (ULONGLONG)(DeviceExtension->PartitionLength.QuadPart - StartingOffset))
                        {
                            IsValid = FALSE;
                        }
                    }
                    break;

                case FeatureIncrementalStreamingWritable:
                    break;

                case FeatureRigidRestrictedOverwrite:
                    if (((TransferByteCount >> DeviceExtension->SectorShift) %
                         CdData->Mmc.Blocking) != 0)
                    {
                        IsValid = FALSE;
                    }
                    /* fall through */
                case FeatureRestrictedOverwrite:
                    if ((Int64ShrlMod32(StartingOffset, DeviceExtension->SectorShift) %
                         CdData->Mmc.Blocking) != 0)
                    {
                        IsValid = FALSE;
                    }
                    break;

                default:
                    IsValid = FALSE;
                    break;
            }
        }

        if (!IsValid)
        {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    UNREFERENCED_PARAMETER(Request);

    return Status;
}

static
VOID
CdromScratchBufferSetupReadWriteSrb(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST OriginalRequest,
    LARGE_INTEGER StartingOffset,
    ULONG RequiredLength,
    PUCHAR DataBuffer,
    BOOLEAN IsReadRequest,
    BOOLEAN UsePartialMdl)
{
    PSCSI_REQUEST_BLOCK Srb = DeviceExtension->ScratchContext.ScratchSrb;
    PCDB Cdb = (PCDB)Srb->Cdb;
    LARGE_INTEGER LogicalBlockAddr;
    ULONG NumTransferBlocks;
    PIRP OriginalIrp = WdfRequestWdmGetIrp(OriginalRequest);
    PIRP Irp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);
    PIO_STACK_LOCATION IrpStack;
    PCDROM_REQUEST_CONTEXT RequestContext =
        CdromGetRequestContext(DeviceExtension->ScratchContext.ScratchRequest);

    if (RequestContext != NULL)
    {
        RequestContext->OriginalRequest = OriginalRequest;
    }

    LogicalBlockAddr.QuadPart =
        (LONGLONG)((ULONGLONG)StartingOffset.QuadPart >> DeviceExtension->SectorShift);
    NumTransferBlocks = RequiredLength >> DeviceExtension->SectorShift;

    IrpStack = IoGetNextIrpStackLocation(Irp);
    IrpStack->MajorFunction = IRP_MJ_SCSI;
    IrpStack->Parameters.DeviceIoControl.IoControlCode =
        IsReadRequest ? IOCTL_SCSI_EXECUTE_IN : IOCTL_SCSI_EXECUTE_OUT;
    IrpStack->Parameters.Scsi.Srb = Srb;

    Srb->Length = SCSI_REQUEST_BLOCK_SIZE;
    Srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
    Srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
    Srb->SrbStatus = 0;
    Srb->ScsiStatus = 0;
    Srb->NextSrb = NULL;
    Srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
    Srb->SenseInfoBuffer = DeviceExtension->ScratchContext.ScratchSense;
    Srb->DataBuffer = DataBuffer;
    Srb->DataTransferLength = RequiredLength;
    Srb->QueueSortKey = LogicalBlockAddr.LowPart;
    if (LogicalBlockAddr.QuadPart > 0xFFFFFFFFLL)
    {
        Srb->QueueSortKey = 0xFFFFFFFF;
    }
    Srb->OriginalRequest = Irp;
    Srb->TimeOutValue = DeviceExtension->TimeOutValue;

    RtlZeroMemory(&Cdb->CDB10, sizeof(Cdb->CDB10));
    Cdb->CDB10.LogicalBlockByte0 = ((PFOUR_BYTE)&LogicalBlockAddr.LowPart)->Byte3;
    Cdb->CDB10.LogicalBlockByte1 = ((PFOUR_BYTE)&LogicalBlockAddr.LowPart)->Byte2;
    Cdb->CDB10.LogicalBlockByte2 = ((PFOUR_BYTE)&LogicalBlockAddr.LowPart)->Byte1;
    Cdb->CDB10.LogicalBlockByte3 = ((PFOUR_BYTE)&LogicalBlockAddr.LowPart)->Byte0;
    Cdb->CDB10.TransferBlocksMsb = ((PFOUR_BYTE)&NumTransferBlocks)->Byte1;
    Cdb->CDB10.TransferBlocksLsb = ((PFOUR_BYTE)&NumTransferBlocks)->Byte0;
    Cdb->CDB10.OperationCode = IsReadRequest ? SCSIOP_READ : SCSIOP_WRITE;
    Srb->CdbLength = sizeof(Cdb->CDB10);

    Srb->SrbFlags = DeviceExtension->SrbFlags;
    if (TEST_FLAG(OriginalIrp->Flags, IRP_PAGING_IO) ||
        TEST_FLAG(OriginalIrp->Flags, IRP_SYNCHRONOUS_PAGING_IO))
    {
        SET_FLAG(Srb->SrbFlags, SRB_CLASS_FLAGS_PAGING);
    }
    SET_FLAG(Srb->SrbFlags, IsReadRequest ? SRB_FLAGS_DATA_IN : SRB_FLAGS_DATA_OUT);
    SET_FLAG(Srb->SrbFlags, SRB_FLAGS_ADAPTER_CACHE_ENABLE);

    if (!UsePartialMdl)
    {
        Irp->MdlAddress = OriginalIrp->MdlAddress;
    }
    else
    {
        if (DeviceExtension->ScratchContext.PartialMdlIsBuilt)
        {
            MmPrepareMdlForReuse(DeviceExtension->ScratchContext.PartialMdl);
        }
        IoBuildPartialMdl(
            OriginalIrp->MdlAddress,
            DeviceExtension->ScratchContext.PartialMdl,
            Srb->DataBuffer,
            Srb->DataTransferLength);
        DeviceExtension->ScratchContext.PartialMdlIsBuilt = TRUE;
        Irp->MdlAddress = DeviceExtension->ScratchContext.PartialMdl;
    }

    Srb->SrbExtension = OriginalIrp;
}

static
NTSTATUS
CdromScratchBufferSendSrbSynchronously(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    PSRB_HISTORY_ITEM *SrbHistoryItem)
{
    NTSTATUS Status;
    PSCSI_REQUEST_BLOCK Srb = DeviceExtension->ScratchContext.ScratchSrb;
    PSRB_HISTORY History = DeviceExtension->ScratchContext.ScratchHistory;
    PSRB_HISTORY_ITEM Item = NULL;
    ULONG SenseSize;

    Srb->InternalStatus = 0;
    Srb->SrbStatus = 0;

    if (SrbHistoryItem != NULL)
    {
        if (History->UsedHistoryCount == History->TotalHistoryCount)
        {
            CompressSrbHistoryData(History);
        }

        if (History->UsedHistoryCount < History->TotalHistoryCount)
        {
            History->UsedHistoryCount++;
            Item = &History->History[History->UsedHistoryCount - 1];
            *SrbHistoryItem = Item;
            RtlZeroMemory(Item, sizeof(*Item));
            KeQueryTickCount(&Item->TickCountSent);
        }
    }

    {
        static ULONGLONG s_totalSendTicks = 0;
        static ULONG s_sendCount = 0;
        LARGE_INTEGER _t0, _t1, _freq;
        _t0 = KeQueryPerformanceCounter(&_freq);
        Status = RequestSend(
            DeviceExtension,
            DeviceExtension->ScratchContext.ScratchRequest,
            DeviceExtension->IoTarget,
            WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
            NULL);
        _t1 = KeQueryPerformanceCounter(NULL);
        s_totalSendTicks += (ULONGLONG)(_t1.QuadPart - _t0.QuadPart);
        s_sendCount++;
        if ((s_sendCount & 0x7F) == 0)
        {
            ULONGLONG _avgUs = (s_totalSendTicks * 1000000ULL) / ((ULONGLONG)_freq.QuadPart * s_sendCount);
            CdromDbgPrint3("[SENDPERF] count=%x avg_us=%x freq=%I64x\n",
                (ULONG_PTR)s_sendCount, (ULONG_PTR)_avgUs, (ULONG_PTR)_freq.QuadPart);
        }
    }

    if (Item != NULL)
    {
        KeQueryTickCount(&Item->TickCountCompleted);
        Item->SrbStatus = Srb->SrbStatus;

        if (Srb->SenseInfoBufferLength >= RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength))
        {
            PSENSE_DATA Sense = (PSENSE_DATA)Srb->SenseInfoBuffer;
            SenseSize = RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength) +
                        Sense->AdditionalSenseLength;
            SenseSize = min(SenseSize, sizeof(SENSE_DATA));
        }
        else
        {
            SenseSize = Srb->SenseInfoBufferLength;
        }

        RtlZeroMemory(&Item->NormalizedSenseData, sizeof(SENSE_DATA));
        RtlCopyMemory(&Item->NormalizedSenseData, Srb->SenseInfoBuffer, SenseSize);
    }

    return Status;
}

static
NTSTATUS
CdromScratchBufferPerformReadWriteSynchronously(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    BOOLEAN FirstTry)
{
    PCDROM_SCRATCH_READ_WRITE_CONTEXT ReadWriteContext =
        &DeviceExtension->ScratchContext.ScratchReadWriteContext;
    PCDROM_REQUEST_CONTEXT RequestContext =
        CdromGetRequestContext(DeviceExtension->ScratchContext.ScratchRequest);
    WDFREQUEST OriginalRequest;
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG TransferSize;
    BOOLEAN UsePartialMdl;
    BOOLEAN ShouldRetry;
    LONGLONG RetryIn100nsUnits;
    LARGE_INTEGER Delay;

    if (RequestContext == NULL || RequestContext->OriginalRequest == NULL)
    {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    OriginalRequest = RequestContext->OriginalRequest;
    TransferSize = min(
        ReadWriteContext->EntireXferLen - ReadWriteContext->TransferedBytes,
        ReadWriteContext->MaxLength);

    if (FirstTry)
    {
        DeviceExtension->ScratchContext.NumRetries = 0;
    }

    while (ReadWriteContext->PacketsCount > 0)
    {
        TransferSize = min(
            ReadWriteContext->EntireXferLen - ReadWriteContext->TransferedBytes,
            ReadWriteContext->MaxLength);

        CdromRustScratchBufferResetItems(DeviceExtension, FALSE);

        UsePartialMdl =
            (ReadWriteContext->PacketsCount > 1 || ReadWriteContext->TransferedBytes > 0);

        CdromScratchBufferSetupReadWriteSrb(
            DeviceExtension,
            OriginalRequest,
            ReadWriteContext->StartingOffset,
            TransferSize,
            ReadWriteContext->DataBuffer,
            ReadWriteContext->IsRead,
            UsePartialMdl);

        WdfRequestSetCompletionRoutine(
            DeviceExtension->ScratchContext.ScratchRequest,
            NULL,
            NULL);

        Status = CdromScratchBufferSendSrbSynchronously(
            DeviceExtension,
            FirstTry ? &ReadWriteContext->SrbHistoryItem : NULL);
        FirstTry = FALSE;

        if ((DeviceExtension->ScratchContext.ScratchSrb->SrbStatus == SRB_STATUS_ABORTED) &&
            (DeviceExtension->ScratchContext.ScratchSrb->InternalStatus == STATUS_CANCELLED))
        {
            Status = STATUS_CANCELLED;
            break;
        }

        if (NT_SUCCESS(Status) &&
            SRB_STATUS(DeviceExtension->ScratchContext.ScratchSrb->SrbStatus) == SRB_STATUS_SUCCESS)
        {
            ReadWriteContext->DataBuffer +=
                DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
            ReadWriteContext->StartingOffset.QuadPart +=
                DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
            ReadWriteContext->TransferedBytes +=
                DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
            ReadWriteContext->PacketsCount--;
            DeviceExtension->ScratchContext.NumRetries = 0;
            continue;
        }

        RetryIn100nsUnits = 0;
        ShouldRetry = RequestSenseInfoInterpretForScratchBuffer(
            DeviceExtension,
            DeviceExtension->ScratchContext.NumRetries,
            &Status,
            &RetryIn100nsUnits);

        CdromDbgPrint5("[RHRW] nonsucc srbst=%x dtl=%x retry=%x st=%x nr=%x\n",
            (ULONG_PTR)DeviceExtension->ScratchContext.ScratchSrb->SrbStatus,
            (ULONG_PTR)DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength,
            (ULONG_PTR)ShouldRetry, (ULONG_PTR)Status,
            (ULONG_PTR)DeviceExtension->ScratchContext.NumRetries);

        if (!ShouldRetry)
        {
            break;
        }

        DeviceExtension->ScratchContext.NumRetries++;
        if (RetryIn100nsUnits != 0)
        {
            Delay.QuadPart = -RetryIn100nsUnits;
            KeDelayExecutionThread(KernelMode, FALSE, &Delay);
        }
    }

    ScratchBuffer_EndUse(DeviceExtension);
    RequestCompletion(
        DeviceExtension,
        OriginalRequest,
        Status,
        ReadWriteContext->TransferedBytes);

    return Status;
}

VOID
CdromRustScratchCompleteOrContinueReadWrite(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST OriginalRequest,
    NTSTATUS Status,
    BOOLEAN MayContinue)
{
    PCDROM_SCRATCH_READ_WRITE_CONTEXT ReadWriteContext =
        &DeviceExtension->ScratchContext.ScratchReadWriteContext;

    if (MayContinue && ReadWriteContext->PacketsCount > 0)
    {
        Status = CdromRustScratchBufferPerformNextReadWrite(DeviceExtension, TRUE);
        if (NT_SUCCESS(Status))
        {
            return;
        }
    }

    ScratchBuffer_EndUse(DeviceExtension);
    RequestCompletion(
        DeviceExtension,
        OriginalRequest,
        Status,
        ReadWriteContext->TransferedBytes);
}

static
NTSTATUS
CdromHandleReadWrite(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters)
{
    NTSTATUS Status = STATUS_SUCCESS;
    PCDROM_DATA CdData = &DeviceExtension->DeviceAdditionalData;
    size_t TransferByteCount;
    PIRP Irp;
    PIO_STACK_LOCATION CurrentStack;
    PUCHAR DataBuffer;
    ULONG EntireXferLen;
    ULONG MaxLength;
    ULONG PacketsCount;
    PCDROM_SCRATCH_READ_WRITE_CONTEXT ReadWriteContext;
    PCDROM_REQUEST_CONTEXT RequestContext;
    PCDROM_REQUEST_CONTEXT OriginalRequestContext;

    Irp = WdfRequestWdmGetIrp(Request);
    CurrentStack = IoGetCurrentIrpStackLocation(Irp);
    DataBuffer = MmGetMdlVirtualAddress(Irp->MdlAddress);

    if (RequestParameters->Type == WdfRequestTypeRead)
    {
        TransferByteCount = RequestParameters->Parameters.Read.Length;
    }
    else
    {
        TransferByteCount = RequestParameters->Parameters.Write.Length;
    }

    if (TransferByteCount == 0)
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_SUCCESS, 0);
        return STATUS_SUCCESS;
    }

    CurrentStack->Parameters.Read.ByteOffset.QuadPart +=
        DeviceExtension->StartingOffset.QuadPart;

    if (RequestParameters->Type == WdfRequestTypeRead)
    {
        RequestParameters->Parameters.Read.DeviceOffset =
            CurrentStack->Parameters.Read.ByteOffset.QuadPart;
    }
    else
    {
        RequestParameters->Parameters.Write.DeviceOffset =
            CurrentStack->Parameters.Write.ByteOffset.QuadPart;
    }

    EntireXferLen = CurrentStack->Parameters.Read.Length;
    if (((ULONG_PTR)DataBuffer & (PAGE_SIZE - 1)) == 0)
    {
        MaxLength = CdData->MaxPageAlignedTransferBytes;
    }
    else
    {
        MaxLength = CdData->MaxUnalignedTransferBytes;
    }

    if (MaxLength == 0)
    {
        MaxLength = EntireXferLen;
    }

    PacketsCount = EntireXferLen / MaxLength;
    if ((EntireXferLen % MaxLength) != 0)
    {
        PacketsCount++;
    }

    OriginalRequestContext = CdromGetRequestContext(Request);
    if (OriginalRequestContext == NULL)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = CdromScratchBufferBeginUse(DeviceExtension);
    if (!NT_SUCCESS(Status))
    {
        CdromRequestCompletion(DeviceExtension, Request, Status, 0);
        return Status;
    }

    ReadWriteContext = &DeviceExtension->ScratchContext.ScratchReadWriteContext;
    RequestContext = CdromGetRequestContext(DeviceExtension->ScratchContext.ScratchRequest);
    if (RequestContext == NULL)
    {
        InterlockedExchangePointer(
            (PVOID)&DeviceExtension->ScratchContext.ScratchInUse,
            NULL);
        CdromRequestCompletion(DeviceExtension, Request, STATUS_INVALID_DEVICE_REQUEST, 0);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    ReadWriteContext->PacketsCount = PacketsCount;
    ReadWriteContext->EntireXferLen = EntireXferLen;
    ReadWriteContext->MaxLength = MaxLength;
    ReadWriteContext->StartingOffset = CurrentStack->Parameters.Read.ByteOffset;
    ReadWriteContext->DataBuffer = DataBuffer;
    ReadWriteContext->TransferedBytes = 0;
    ReadWriteContext->IsRead = RequestParameters->Type == WdfRequestTypeRead;

    RequestContext->OriginalRequest = Request;
    RequestContext->DeviceExtension = DeviceExtension;

    OriginalRequestContext->ReadWriteIsCompleted = FALSE;
    OriginalRequestContext->ReadWriteRetryInitialized = FALSE;
    OriginalRequestContext->DeviceExtension = DeviceExtension;

    return CdromScratchBufferPerformReadWriteSynchronously(DeviceExtension, TRUE);
}

/* MCN poll instrumentation: counts media-change polls between reads. */
volatile LONG g_cdromMcnSinceRead = 0;
VOID CdromMcnPollMark(void) { g_cdromMcnSinceRead++; }

/* Gap profiler: prints only inter-read gaps > 5ms to locate boot stalls. */
VOID
CdromGapLog(void)
{
    static LARGE_INTEGER s_last = {0};
    static ULONG s_seq = 0;
    LARGE_INTEGER now, freq;
    LONG mcn = g_cdromMcnSinceRead;
    g_cdromMcnSinceRead = 0;
    now = KeQueryPerformanceCounter(&freq);
    s_seq++;
    if (s_last.QuadPart != 0)
    {
        ULONGLONG gap_us = ((ULONGLONG)(now.QuadPart - s_last.QuadPart) * 1000000ULL) / (ULONGLONG)freq.QuadPart;
        if (gap_us > 1000000)
        {
            DbgPrint("[GAP] read#%lu gap=%lu us mcnPolls=%ld\n", (ULONG)s_seq, (ULONG)gap_us, mcn);
        }
    }
    s_last = now;
}

/* Per-read trace gate. 0 = silent (measure real boot speed); 1 = verbose. */
volatile LONG g_cdromTrace = 0; /* 0 = silent (production); 1 = verbose per-read tracing */

/* Ungated forced trace for rare (>500ms) stall events — never floods. */
ULONG CdromDbgPrintForce2(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2)
{
    return DbgPrint(Format, Arg1, Arg2);
}

ULONG
CdromDbgPrint0(const char *Format)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint("%s", Format);
}

ULONG
CdromDbgPrint1(const char *Format, ULONG_PTR Arg1)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(Format, Arg1);
}

ULONG
CdromDbgPrint2(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(Format, Arg1, Arg2);
}

ULONG
CdromDbgPrint3(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2, ULONG_PTR Arg3)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(Format, Arg1, Arg2, Arg3);
}

ULONG
CdromDbgPrint4(const char *Format, ULONG_PTR Arg1, ULONG_PTR Arg2, ULONG_PTR Arg3, ULONG_PTR Arg4)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(Format, Arg1, Arg2, Arg3, Arg4);
}

ULONG
CdromDbgPrint5(
    const char *Format,
    ULONG_PTR Arg1,
    ULONG_PTR Arg2,
    ULONG_PTR Arg3,
    ULONG_PTR Arg4,
    ULONG_PTR Arg5)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(Format, Arg1, Arg2, Arg3, Arg4, Arg5);
}

ULONG
CdromDbgPrintReadWriteValidate(
    const char *Format,
    ULONG_PTR Type,
    ULONGLONG StartingOffset,
    ULONG_PTR TransferByteCount,
    ULONG_PTR BytesPerSector,
    ULONGLONG PartitionLength,
    ULONG_PTR ValidationSchema,
    ULONG_PTR IsValid,
    ULONG_PTR Status)
{
    if (!g_cdromTrace) return 0;
    return DbgPrint(
        Format,
        Type,
        StartingOffset,
        TransferByteCount,
        BytesPerSector,
        PartitionLength,
        ValidationSchema,
        IsValid,
        Status);
}

ULONG
CdromDbgPrintExHistory(
    ULONG ComponentId,
    ULONG Level,
    const char *Format,
    ULONG_PTR SrbStatus,
    ULONG_PTR Sense,
    ULONG_PTR Asc,
    ULONG_PTR Ascq,
    ULONG_PTR Index1Signed,
    ULONG_PTR Index1Hex,
    ULONG_PTR Index2Signed,
    ULONG_PTR Index2Hex)
{
    return DbgPrintEx(
        ComponentId,
        Level,
        Format,
        SrbStatus,
        Sense,
        Asc,
        Ascq,
        Index1Signed,
        Index1Hex,
        Index2Signed,
        Index2Hex);
}

VOID
CdromWdfDeviceSetAlignmentRequirement(WDFDEVICE Device, ULONG AlignmentRequirement)
{
    WdfDeviceSetAlignmentRequirement(Device, AlignmentRequirement);
}

BOOLEAN
CdromTryHandleStorageControl(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters)
{
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG_PTR Information = 0;
    ULONG IoControlCode = RequestParameters->Parameters.DeviceIoControl.IoControlCode;
    PZERO_POWER_ODD_INFO ZpoddInfo;
    BOOLEAN InZeroPowerState;

    ZpoddInfo = DeviceExtension->ZeroPowerODDInfo;
    InZeroPowerState = (ZpoddInfo != NULL && ZpoddInfo->InZeroPowerState != FALSE);

    switch (IoControlCode)
    {
        case IOCTL_STORAGE_CHECK_VERIFY:
        case IOCTL_STORAGE_CHECK_VERIFY2:
            if ((RequestParameters->Parameters.DeviceIoControl.OutputBufferLength > 0) &&
                (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength < sizeof(ULONG)))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = sizeof(ULONG);
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:
        case IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX:
            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                FIELD_OFFSET(DISK_GEOMETRY_EX, Data))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = FIELD_OFFSET(DISK_GEOMETRY_EX, Data);
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_DISK_GET_DRIVE_GEOMETRY:
        case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(DISK_GEOMETRY))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = sizeof(DISK_GEOMETRY);
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_STORAGE_READ_CAPACITY:
            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(STORAGE_READ_CAPACITY))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = sizeof(STORAGE_READ_CAPACITY);
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_CDROM_READ_TOC_EX:
            if (RequestParameters->Parameters.DeviceIoControl.InputBufferLength <
                sizeof(CDROM_READ_TOC_EX))
            {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            }
            else if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                     MINIMUM_CDROM_READ_TOC_EX_SIZE)
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = MINIMUM_CDROM_READ_TOC_EX_SIZE;
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_CDROM_READ_TOC:
            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(CDROM_TOC))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = sizeof(CDROM_TOC);
            }
            else
            {
                Status = InZeroPowerState ? STATUS_NO_MEDIA_IN_DEVICE : STATUS_SUCCESS;
            }
            break;

        case IOCTL_CDROM_GET_LAST_SESSION:
            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(CDROM_TOC_SESSION_DATA))
            {
                Status = STATUS_BUFFER_TOO_SMALL;
                Information = sizeof(CDROM_TOC_SESSION_DATA);
            }
            break;

        case IOCTL_STORAGE_FIND_NEW_DEVICES:
            IoInvalidateDeviceRelations(DeviceExtension->LowerPdo, BusRelations);
            Status = STATUS_SUCCESS;
            break;

        case IOCTL_STORAGE_GET_DEVICE_NUMBER:
        {
            PSTORAGE_DEVICE_NUMBER DeviceNumber = NULL;

            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(STORAGE_DEVICE_NUMBER))
            {
                CdromRequestCompletion(
                    DeviceExtension,
                    Request,
                    STATUS_BUFFER_TOO_SMALL,
                    sizeof(STORAGE_DEVICE_NUMBER));
                return TRUE;
            }

            Status = WdfRequestRetrieveOutputBuffer(
                Request,
                sizeof(STORAGE_DEVICE_NUMBER),
                &DeviceNumber,
                NULL);
            if (NT_SUCCESS(Status))
            {
                DeviceNumber->DeviceType = DeviceExtension->DeviceObject->DeviceType;
                DeviceNumber->DeviceNumber = DeviceExtension->DeviceNumber;
                DeviceNumber->PartitionNumber = (ULONG)-1;
                Information = sizeof(STORAGE_DEVICE_NUMBER);
            }

            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;
        }

        case IOCTL_SCSI_GET_CAPABILITIES:
        {
            PIO_SCSI_CAPABILITIES Capabilities = NULL;

            if (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength <
                sizeof(IO_SCSI_CAPABILITIES))
            {
                CdromRequestCompletion(
                    DeviceExtension,
                    Request,
                    STATUS_BUFFER_TOO_SMALL,
                    sizeof(IO_SCSI_CAPABILITIES));
                return TRUE;
            }

            Status = WdfRequestRetrieveOutputBuffer(
                Request,
                sizeof(IO_SCSI_CAPABILITIES),
                &Capabilities,
                NULL);
            if (NT_SUCCESS(Status))
            {
                RtlZeroMemory(Capabilities, sizeof(*Capabilities));
                Capabilities->Length = sizeof(*Capabilities);
                if (DeviceExtension->AdapterDescriptor != NULL)
                {
                    Capabilities->MaximumTransferLength =
                        DeviceExtension->AdapterDescriptor->MaximumTransferLength;
                    Capabilities->MaximumPhysicalPages =
                        DeviceExtension->AdapterDescriptor->MaximumPhysicalPages;
                    Capabilities->AlignmentMask =
                        DeviceExtension->AdapterDescriptor->AlignmentMask;
                    Capabilities->AdapterUsesPio =
                        DeviceExtension->AdapterDescriptor->AdapterUsesPio;
                }
                Information = sizeof(*Capabilities);
            }

            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;
        }

        case IOCTL_MOUNTDEV_QUERY_STABLE_GUID:
            CdromRequestCompletion(DeviceExtension, Request, STATUS_NOT_SUPPORTED, 0);
            return TRUE;

        case IOCTL_MOUNTDEV_LINK_CREATED:
        case IOCTL_MOUNTDEV_LINK_DELETED:
            CdromRequestCompletion(DeviceExtension, Request, STATUS_SUCCESS, 0);
            return TRUE;

        default:
            return FALSE;
    }

    if (NT_SUCCESS(Status))
    {
        Status = WdfDeviceEnqueueRequest(DeviceExtension->Device, Request);
    }

    if (!NT_SUCCESS(Status))
    {
        CdromRequestCompletion(DeviceExtension, Request, Status, Information);
    }

    return TRUE;
}

BOOLEAN
CdromTryHandleSerializedStorageControl(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PWDF_REQUEST_PARAMETERS RequestParameters)
{
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG_PTR Information = 0;
    ULONG IoControlCode = RequestParameters->Parameters.DeviceIoControl.IoControlCode;
    PULONG OutputBuffer = NULL;
    PVOID GenericOutputBuffer = NULL;

    switch (IoControlCode)
    {
        case IOCTL_CDROM_READ_TOC:
            Status = CdromCompleteToc(
                DeviceExtension,
                Request,
                RequestParameters->Parameters.DeviceIoControl.OutputBufferLength,
                TRUE,
                &Information);
            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        case IOCTL_CDROM_GET_LAST_SESSION:
            Status = CdromCompleteLastSession(DeviceExtension, Request, TRUE, &Information);
            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        case IOCTL_CDROM_READ_TOC_EX:
        {
            PCDROM_READ_TOC_EX InputBuffer = NULL;

            Status = WdfRequestRetrieveInputBuffer(
                Request,
                sizeof(CDROM_READ_TOC_EX),
                &InputBuffer,
                NULL);
            if (NT_SUCCESS(Status))
            {
                if (InputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_TOC)
                {
                    Status = CdromCompleteToc(
                        DeviceExtension,
                        Request,
                        RequestParameters->Parameters.DeviceIoControl.OutputBufferLength,
                        InputBuffer->Msf != 0,
                        &Information);
                }
                else if (InputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_SESSION)
                {
                    Status = CdromCompleteLastSession(
                        DeviceExtension,
                        Request,
                        InputBuffer->Msf != 0,
                        &Information);
                }
                else
                {
                    Status = CdromCompleteMinimalTocEx(Request, &Information);
                }
            }

            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;
        }

        case IOCTL_DISK_GET_DRIVE_GEOMETRY:
        case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
            Status = WdfRequestRetrieveOutputBuffer(
                Request,
                sizeof(DISK_GEOMETRY),
                &GenericOutputBuffer,
                NULL);
            if (NT_SUCCESS(Status))
            {
                *(PDISK_GEOMETRY)GenericOutputBuffer = DeviceExtension->DiskGeometry;
                Information = sizeof(DISK_GEOMETRY);
            }
            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:
        case IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX:
            Status = WdfRequestRetrieveOutputBuffer(
                Request,
                FIELD_OFFSET(DISK_GEOMETRY_EX, Data),
                &GenericOutputBuffer,
                NULL);
            if (NT_SUCCESS(Status))
            {
                PDISK_GEOMETRY_EX GeometryEx = (PDISK_GEOMETRY_EX)GenericOutputBuffer;
                GeometryEx->Geometry = DeviceExtension->DiskGeometry;
                GeometryEx->DiskSize = DeviceExtension->PartitionLength;
                Information = FIELD_OFFSET(DISK_GEOMETRY_EX, Data);
            }
            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        case IOCTL_STORAGE_READ_CAPACITY:
            Status = WdfRequestRetrieveOutputBuffer(
                Request,
                sizeof(STORAGE_READ_CAPACITY),
                &GenericOutputBuffer,
                NULL);
            if (NT_SUCCESS(Status))
            {
                PSTORAGE_READ_CAPACITY ReadCapacity = (PSTORAGE_READ_CAPACITY)GenericOutputBuffer;
                RtlZeroMemory(ReadCapacity, sizeof(*ReadCapacity));
                ReadCapacity->Version = sizeof(STORAGE_READ_CAPACITY);
                ReadCapacity->Size = sizeof(STORAGE_READ_CAPACITY);
                ReadCapacity->BlockLength = CdromGetBlockLength(DeviceExtension);
                ReadCapacity->NumberOfBlocks.QuadPart = CdromGetBlockCount(DeviceExtension);
                ReadCapacity->DiskLength = DeviceExtension->PartitionLength;
                Information = sizeof(STORAGE_READ_CAPACITY);
            }
            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        case IOCTL_STORAGE_CHECK_VERIFY:
        case IOCTL_STORAGE_CHECK_VERIFY2:
            if ((IoControlCode == IOCTL_STORAGE_CHECK_VERIFY) &&
                (RequestParameters->Parameters.DeviceIoControl.OutputBufferLength != 0))
            {
                Status = WdfRequestRetrieveOutputBuffer(
                    Request,
                    RequestParameters->Parameters.DeviceIoControl.OutputBufferLength,
                    &OutputBuffer,
                    NULL);

                if (NT_SUCCESS(Status) && OutputBuffer != NULL)
                {
                    *OutputBuffer = DeviceExtension->MediaChangeCount;
                    Information = sizeof(ULONG);
                }
            }

            CdromRequestCompletion(DeviceExtension, Request, Status, Information);
            return TRUE;

        default:
            return FALSE;
    }
}

VOID
CdromRequestCompletion(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    NTSTATUS Status,
    ULONG_PTR Information)
{
    /* Every call site this stands in for is a RequestCompletion() in the C
     * (cdrom.c, ioctl.c). A bare WdfRequestCompleteWithInformation skipped its
     * two side effects: IoSetHardErrorOrVerifyDevice for user-induced errors
     * (no-media / verify-required never prompted or forced a re-mount) and the
     * surprise-removed -> STATUS_DEVICE_DOES_NOT_EXIST mapping IMAPI relies on. */
    RequestCompletion(DeviceExtension, Request, Status, Information);
}

VOID
CdromPrepareRequestContext(
    WDFREQUEST Request,
    PCDROM_DEVICE_EXTENSION DeviceExtension)
{
    LARGE_INTEGER TickCount;
    PCDROM_REQUEST_CONTEXT RequestContext = CdromGetRequestContext(Request);

    if (RequestContext == NULL)
    {
        return;
    }

    RequestContext->DeviceExtension = DeviceExtension;
    KeQueryTickCount(&TickCount);
    RequestContext->TimeReceived = TickCount;
}

VOID
CdromGetReadWriteVerifyState(
    PCDROM_DEVICE_EXTENSION DeviceExtension,
    WDFREQUEST Request,
    PULONG DeviceFlags,
    PUCHAR MinorFunction,
    PUCHAR IrpFlags)
{
    PIRP Irp = WdfRequestWdmGetIrp(Request);
    PIO_STACK_LOCATION IrpStack = IoGetCurrentIrpStackLocation(Irp);

    *DeviceFlags = DeviceExtension->DeviceObject->Flags;
    *MinorFunction = IrpStack->MinorFunction;
    *IrpFlags = IrpStack->Flags;
}

VOID
CdromMarkVolumeVerifyChecked(
    WDFREQUEST Request)
{
    PIRP Irp = WdfRequestWdmGetIrp(Request);
    PIO_STACK_LOCATION IrpStack = IoGetCurrentIrpStackLocation(Irp);

    IrpStack->MinorFunction = CDROM_VOLUME_VERIFY_CHECKED;
}

VOID
CdromGetReadWriteFields(
    WDFREQUEST Request,
    PULONG RequestType,
    PLONGLONG DeviceOffset,
    PULONG_PTR TransferLength)
{
    WDF_REQUEST_PARAMETERS RequestParameters;

    WDF_REQUEST_PARAMETERS_INIT(&RequestParameters);
    WdfRequestGetParameters(Request, &RequestParameters);

    *RequestType = RequestParameters.Type;
    if (RequestParameters.Type == WdfRequestTypeRead)
    {
        *DeviceOffset = RequestParameters.Parameters.Read.DeviceOffset;
        *TransferLength = RequestParameters.Parameters.Read.Length;
    }
    else
    {
        *DeviceOffset = RequestParameters.Parameters.Write.DeviceOffset;
        *TransferLength = RequestParameters.Parameters.Write.Length;
    }
}

NTSTATUS
CdromInvokeSyncCallback(
    PCDROM_REQUEST_CONTEXT RequestContext,
    WDFDEVICE Device,
    WDFREQUEST Request)
{
    if (RequestContext->SyncCallback == NULL)
    {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    return RequestContext->SyncCallback(Device, Request);
}

VOID
CdromDeviceEvtFileClose(
    WDFFILEOBJECT FileObject)
{
    NTSTATUS Status;
    WDFDEVICE Device;
    PCDROM_DEVICE_EXTENSION DeviceExtension;
    PFILE_OBJECT_CONTEXT FileObjectContext;
    PCDROM_DATA CdData;

    if (FileObject == NULL)
    {
        return;
    }

    Device = WdfFileObjectGetDevice(FileObject);
    DeviceExtension = CdromGetDeviceExtension(Device);
    FileObjectContext = CdromGetFileObjectContext(FileObject);
    if (DeviceExtension == NULL || FileObjectContext == NULL)
    {
        return;
    }
    CdData = &DeviceExtension->DeviceAdditionalData;

    Status = DeviceCleanupProtectedLocks(DeviceExtension, FileObjectContext);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("Failed to cleanup protected locks for WDFDEVICE %p, 0x%08lx\n", Device, Status);
    }

    Status = DeviceCleanupDisableMcn(DeviceExtension, FileObjectContext);
    if (!NT_SUCCESS(Status))
    {
        DbgPrint("Failed to disable MCN for WDFDEVICE %p, 0x%08lx\n", Device, Status);
    }

    if (CdData->ExclusiveOwner == FileObject)
    {
        Status = DeviceUnlockExclusive(DeviceExtension, FileObject, FALSE);
        if (!NT_SUCCESS(Status))
        {
            DbgPrint("Failed to release exclusive lock for WDFDEVICE %p, 0x%08lx\n", Device, Status);
        }
    }
}

VOID
NTAPI
CdromSequentialQueueEvtIoReadWrite(
    WDFQUEUE Queue,
    WDFREQUEST Request,
    size_t Length)
{
    NTSTATUS Status = STATUS_SUCCESS;
    WDFDEVICE Device = WdfIoQueueGetDevice(Queue);
    PCDROM_DEVICE_EXTENSION DeviceExtension = CdromGetDeviceExtension(Device);
    WDF_REQUEST_PARAMETERS RequestParameters;
    PIRP Irp;
    CdromGapLog();
    Irp = WdfRequestWdmGetIrp(Request);
    PIO_STACK_LOCATION CurrentIrpStack = IoGetCurrentIrpStackLocation(Irp);
    PCDROM_DATA CdData;

    if (DeviceExtension == NULL)
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_INVALID_DEVICE_REQUEST, 0);
        return;
    }

    CdData = &DeviceExtension->DeviceAdditionalData;

    WDF_REQUEST_PARAMETERS_INIT(&RequestParameters);
    WdfRequestGetParameters(Request, &RequestParameters);

    if ((DeviceExtension->DeviceObject->Flags & DO_VERIFY_VOLUME) &&
        (RequestParameters.MinorFunction != CDROM_VOLUME_VERIFY_CHECKED) &&
        !(CurrentIrpStack->Flags & SL_OVERRIDE_VERIFY_VOLUME))
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_VERIFY_REQUIRED, 0);
        return;
    }

    CurrentIrpStack->MinorFunction = CDROM_VOLUME_VERIFY_CHECKED;

    if (DeviceExtension->DeviceAdditionalData.PlayActive)
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_DEVICE_BUSY, 0);
        return;
    }

    if (CdData->ExclusiveOwner != NULL &&
        CdData->ExclusiveOwner != WdfRequestGetFileObject(Request))
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_ACCESS_DENIED, 0);
        return;
    }

    if (Length == 0)
    {
        CdromRequestCompletion(DeviceExtension, Request, STATUS_SUCCESS, Length);
        return;
    }

    if (!DeviceExtension->DeviceAdditionalData.Mmc.WriteAllowed &&
        RequestParameters.Type == WdfRequestTypeWrite)
    {
        DeviceExtension->DeviceAdditionalData.Mmc.UpdateState = CdromMmcUpdateRequired;
    }

    if (DeviceIsMmcUpdateRequired(DeviceExtension->Device))
    {
        DeviceExtension->ReadWriteWorkItemContext.OriginalRequest = Request;
        WdfWorkItemEnqueue(DeviceExtension->ReadWriteWorkItem);
    }
    else
    {
        static ULONGLONG s_totalRwTicks = 0;
        static ULONG s_rwCount = 0;
        LARGE_INTEGER _r0, _r1, _rfreq;
        _r0 = KeQueryPerformanceCounter(&_rfreq);
        Status = CdromValidateReadWrite(DeviceExtension, Request, &RequestParameters);
        if (NT_SUCCESS(Status))
        {
            Status = CdromHandleReadWrite(DeviceExtension, Request, &RequestParameters);
        }
        _r1 = KeQueryPerformanceCounter(NULL);
        s_totalRwTicks += (ULONGLONG)(_r1.QuadPart - _r0.QuadPart);
        s_rwCount++;
        if ((s_rwCount & 0x7F) == 0)
        {
            ULONGLONG _avgUs = (s_totalRwTicks * 1000000ULL) / ((ULONGLONG)_rfreq.QuadPart * s_rwCount);
            CdromDbgPrint2("[RWPERF] count=%x avg_us=%x\n", (ULONG_PTR)s_rwCount, (ULONG_PTR)_avgUs);
        }
    }

    if (!NT_SUCCESS(Status))
    {
        CdromRequestCompletion(DeviceExtension, Request, Status, 0);
    }
}
