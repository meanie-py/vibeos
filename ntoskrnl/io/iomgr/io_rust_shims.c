/*
 * io_rust_shims.c — C functions not yet translated to Rust for the IO subsystem.
 *
 * IopInitializeDriverModule is a complex function that was not translated in
 * the IO Tier 1 batch. It remains in C and links against both C and Rust symbols.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* ========================================================================
 * Extern declarations for Rust-defined symbols used by IopInitializeDriverModule
 * ======================================================================== */

extern UNICODE_STRING IopHardwareDatabaseKey;
extern BOOLEAN PnpSystemInit;
extern BOOLEAN PnPBootDriversLoaded;

NTSTATUS NTAPI IopInvalidDeviceRequest(PDEVICE_OBJECT DeviceObject, PIRP Irp);
VOID NTAPI IopReadyDeviceObjects(PDRIVER_OBJECT Driver);
VOID NTAPI IopReinitializeDrivers(VOID);

/* ========================================================================
 * FORCEINLINE/macro shims
 * ======================================================================== */

/*
 * HalInitPnpDriver — macro in haltypes.h that calls HALDISPATCH->HalInitPnpDriver
 * Rust can't expand this macro, needs a C shim.
 */
NTSTATUS
HalInitPnpDriver_shim(VOID)
{
    return HalInitPnpDriver();
}


/* ========================================================================
 * IopInitializeDriverModule — from driver.c
 * Creates a driver object, initializes it, and calls DriverEntry.
 * ======================================================================== */

NTSTATUS
IopInitializeDriverModule(
    _In_ PLDR_DATA_TABLE_ENTRY ModuleObject,
    _In_ HANDLE ServiceHandle,
    _Out_ PDRIVER_OBJECT *OutDriverObject,
    _Out_ NTSTATUS *DriverEntryStatus)
{
    UNICODE_STRING DriverName, RegistryPath, ServiceName;
    NTSTATUS Status;

    PAGED_CODE();

    Status = IopGetDriverNames(ServiceHandle, &DriverName, &ServiceName);
    if (!NT_SUCCESS(Status))
    {
        MmUnloadSystemImage(ModuleObject);
        return Status;
    }

    DPRINT("Driver name: '%wZ'\n", &DriverName);

    PIMAGE_NT_HEADERS NtHeaders = RtlImageNtHeader(ModuleObject->DllBase);
    ASSERT(NtHeaders);
    ASSERT(ModuleObject->SizeOfImage == ROUND_TO_PAGES(NtHeaders->OptionalHeader.SizeOfImage));
    ASSERT(ModuleObject->EntryPoint == RVA(ModuleObject->DllBase, NtHeaders->OptionalHeader.AddressOfEntryPoint));

    /* Obtain the registry path for the DriverInit routine */
    PKEY_NAME_INFORMATION nameInfo;
    ULONG infoLength;
    Status = ZwQueryKey(ServiceHandle, KeyNameInformation, NULL, 0, &infoLength);
    if (Status == STATUS_BUFFER_TOO_SMALL)
    {
        nameInfo = ExAllocatePoolWithTag(NonPagedPool, infoLength, TAG_IO);
        if (nameInfo)
        {
            Status = ZwQueryKey(ServiceHandle,
                                KeyNameInformation,
                                nameInfo,
                                infoLength,
                                &infoLength);
            if (NT_SUCCESS(Status))
            {
                RegistryPath.Length = nameInfo->NameLength;
                RegistryPath.MaximumLength = nameInfo->NameLength;
                RegistryPath.Buffer = nameInfo->Name;
            }
            else
            {
                ExFreePoolWithTag(nameInfo, TAG_IO);
            }
        }
        else
        {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }
    else
    {
        Status = NT_SUCCESS(Status) ? STATUS_UNSUCCESSFUL : Status;
    }

    if (!NT_SUCCESS(Status))
    {
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        MmUnloadSystemImage(ModuleObject);
        return Status;
    }

    /* Create the driver object */
    ULONG ObjectSize = sizeof(DRIVER_OBJECT) + sizeof(EXTENDED_DRIVER_EXTENSION);
    OBJECT_ATTRIBUTES objAttrs;
    PDRIVER_OBJECT driverObject;
    InitializeObjectAttributes(&objAttrs,
                               &DriverName,
                               OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);

    Status = ObCreateObject(KernelMode,
                            IoDriverObjectType,
                            &objAttrs,
                            KernelMode,
                            NULL,
                            ObjectSize,
                            0,
                            0,
                            (PVOID*)&driverObject);
    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        MmUnloadSystemImage(ModuleObject);
        DPRINT1("Error while creating driver object \"%wZ\" status %x\n", &DriverName, Status);
        return Status;
    }

    DPRINT("Created driver object 0x%p for \"%wZ\"\n", driverObject, &DriverName);

    RtlZeroMemory(driverObject, ObjectSize);
    driverObject->Type = IO_TYPE_DRIVER;
    driverObject->Size = sizeof(DRIVER_OBJECT);

    /* Set the legacy flag if this is not a WDM driver */
    if (!(NtHeaders->OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_WDM_DRIVER))
        driverObject->Flags |= DRVO_LEGACY_DRIVER;

    driverObject->DriverSection = ModuleObject;
    driverObject->DriverStart = ModuleObject->DllBase;
    driverObject->DriverSize = ModuleObject->SizeOfImage;
    driverObject->DriverInit = ModuleObject->EntryPoint;
    driverObject->HardwareDatabase = &IopHardwareDatabaseKey;
    driverObject->DriverExtension = (PDRIVER_EXTENSION)(driverObject + 1);
    driverObject->DriverExtension->DriverObject = driverObject;

    /* Loop all Major Functions */
    for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
    {
        driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
    }

    /* Add the Object and get its handle */
    HANDLE hDriver;
    Status = ObInsertObject(driverObject, NULL, FILE_READ_DATA, 0, NULL, &hDriver);
    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return Status;
    }

    /* Now reference it */
    Status = ObReferenceObjectByHandle(hDriver,
                                       0,
                                       IoDriverObjectType,
                                       KernelMode,
                                       (PVOID*)&driverObject,
                                       NULL);

    /* Close the extra handle */
    ZwClose(hDriver);

    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return Status;
    }

    /* Set up the service key name buffer */
    UNICODE_STRING serviceKeyName;
    serviceKeyName.Length = 0;
    serviceKeyName.MaximumLength = ServiceName.MaximumLength + sizeof(UNICODE_NULL);
    serviceKeyName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
                                                  serviceKeyName.MaximumLength,
                                                  TAG_IO);
    if (!serviceKeyName.Buffer)
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Copy the name and set it in the driver extension */
    RtlCopyUnicodeString(&serviceKeyName, &ServiceName);
    RtlFreeUnicodeString(&ServiceName);
    driverObject->DriverExtension->ServiceKeyName = serviceKeyName;

    /* Make a copy of the driver name to store in the driver object */
    UNICODE_STRING driverNamePaged;
    driverNamePaged.Length = 0;
    driverNamePaged.MaximumLength = DriverName.MaximumLength + sizeof(UNICODE_NULL);
    driverNamePaged.Buffer = ExAllocatePoolWithTag(PagedPool,
                                                   driverNamePaged.MaximumLength,
                                                   TAG_IO);
    if (!driverNamePaged.Buffer)
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(&driverNamePaged, &DriverName);
    driverObject->DriverName = driverNamePaged;

    /* Finally, call its init function */
    DPRINT("Calling DriverInit for '%wZ'\n", &DriverName);
    Status = driverObject->DriverInit(driverObject, &RegistryPath);
    *DriverEntryStatus = Status;
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("'%wZ' initialization failed, status (0x%08lx)\n", &DriverName, Status);
        Status = STATUS_FAILED_DRIVER_ENTRY;
    }

    /* HACK: We're going to say if we don't have any DOs from DriverEntry, then we're not legacy. */
    if (!driverObject->DeviceObject)
    {
        driverObject->Flags &= ~DRVO_LEGACY_DRIVER;
    }

    /* Windows does this fixup, keep it for compatibility */
    for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
    {
        if (!driverObject->MajorFunction[i])
        {
            DPRINT1("Driver <%wZ> set DriverObject->MajorFunction[%lu] to NULL!\n",
                    &driverObject->DriverName, i);
            driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
        }
    }

    ExFreePoolWithTag(nameInfo, TAG_IO);
    RtlFreeUnicodeString(&DriverName);

    if (!NT_SUCCESS(Status))
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        return Status;
    }

    *OutDriverObject = driverObject;

    MmFreeDriverInitialization((PLDR_DATA_TABLE_ENTRY)driverObject->DriverSection);

    /* Set the driver as initialized */
    IopReadyDeviceObjects(driverObject);

    if (PnpSystemInit) IopReinitializeDrivers();

    return STATUS_SUCCESS;
}
