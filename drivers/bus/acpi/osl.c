/*******************************************************************************
*                                                                              *
* ACPI Component Architecture Operating System Layer (OSL) for ReactOS         *
*                                                                              *
*******************************************************************************/

#include "precomp.h"

#include <arc/arc.h>
#include <reactos/drivers/acpi/acpi.h>
#include <pseh/pseh2.h>

#define NDEBUG
#include <debug.h>

#define TAG_ACPI_LOADER_RSDP_TABLE   'RpcA'

static PKINTERRUPT AcpiInterrupt;
static BOOLEAN AcpiInterruptHandlerRegistered = FALSE;
static ACPI_OSD_HANDLER AcpiIrqHandler = NULL;
static PVOID AcpiIrqContext = NULL;
static ULONG AcpiIrqNumber = 0;
/* TODO: Replace these local declarations with <ndk/kefuncs.h> once the
 * acpi.sys build context can consume the required NDK dependencies cleanly.
 */
extern NTSYSAPI PLOADER_PARAMETER_BLOCK KeLoaderBlock;
extern NTSYSAPI
PCONFIGURATION_COMPONENT_DATA
NTAPI
KeFindConfigurationNextEntry(
    _In_ PCONFIGURATION_COMPONENT_DATA Child,
    _In_ CONFIGURATION_CLASS Class,
    _In_ CONFIGURATION_TYPE Type,
    _In_opt_ PULONG ComponentKey,
    _Inout_ PCONFIGURATION_COMPONENT_DATA *NextLink);

static ACPI_TABLE_RSDP *AcpiLoaderRsdp = NULL;

static
UCHAR
AcpiChecksumBuffer(
    _In_reads_bytes_(Length) const UCHAR *Buffer,
    _In_ ULONG Length)
{
    ULONG Index;
    UCHAR Sum;

    Sum = 0;
    for (Index = 0; Index < Length; ++Index)
        Sum = (UCHAR)(Sum + Buffer[Index]);

    return (UCHAR)(0 - Sum);
}

static
PACPI_BIOS_MULTI_NODE
AcpiGetLoaderAcpiBiosNode(VOID)
{
    PCONFIGURATION_COMPONENT_DATA ComponentEntry = NULL;
    PCONFIGURATION_COMPONENT_DATA Next = NULL;
    PCM_PARTIAL_RESOURCE_LIST ResourceList;

    if (!KeLoaderBlock || !KeLoaderBlock->ConfigurationRoot)
        return NULL;

    ComponentEntry = KeFindConfigurationNextEntry(KeLoaderBlock->ConfigurationRoot,
                                                  AdapterClass,
                                                  MultiFunctionAdapter,
                                                  NULL,
                                                  &Next);
    while (ComponentEntry)
    {
        if ((ComponentEntry->ComponentEntry.Identifier != NULL) &&
            !_stricmp(ComponentEntry->ComponentEntry.Identifier, "ACPI BIOS"))
        {
            break;
        }

        Next = ComponentEntry;
        ComponentEntry = KeFindConfigurationNextEntry(KeLoaderBlock->ConfigurationRoot,
                                                      AdapterClass,
                                                      MultiFunctionAdapter,
                                                      NULL,
                                                      &Next);
    }

    if (!ComponentEntry)
        return NULL;

    ResourceList = ComponentEntry->ConfigurationData;
    if (!ResourceList ||
        (ComponentEntry->ComponentEntry.ConfigurationDataLength <
         FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) +
         FIELD_OFFSET(ACPI_BIOS_MULTI_NODE, E820Entry)) ||
        (ResourceList->Count < 1) ||
        (ResourceList->PartialDescriptors[0].Type != CmResourceTypeDeviceSpecific))
    {
        DPRINT1("Loader ACPI BIOS node is missing valid device-specific data\n");
        return NULL;
    }

    return (PACPI_BIOS_MULTI_NODE)(ResourceList + 1);
}

static
ACPI_PHYSICAL_ADDRESS
AcpiBuildLoaderRootPointer(VOID)
{
    PACPI_BIOS_MULTI_NODE NodeData;
    ACPI_TABLE_HEADER *RootTable;
    PHYSICAL_ADDRESS PhysicalAddress;

    if (AcpiLoaderRsdp != NULL)
    {
        PhysicalAddress = MmGetPhysicalAddress(AcpiLoaderRsdp);
        return (ACPI_PHYSICAL_ADDRESS)PhysicalAddress.QuadPart;
    }

    NodeData = AcpiGetLoaderAcpiBiosNode();
    if (!NodeData || (NodeData->RsdtAddress.QuadPart == 0))
        goto Failure;

    RootTable = MmMapIoSpace(NodeData->RsdtAddress, sizeof(*RootTable), MmNonCached);
    if (!RootTable)
    {
        DPRINT1("Unable to map loader ACPI root table at 0x%I64x\n",
                NodeData->RsdtAddress.QuadPart);
        goto Failure;
    }

    AcpiLoaderRsdp = ExAllocatePoolZero(NonPagedPool,
                                        sizeof(*AcpiLoaderRsdp),
                                        TAG_ACPI_LOADER_RSDP_TABLE);
    if (!AcpiLoaderRsdp)
    {
        DPRINT1("Unable to allocate synthetic RSDP for loader ACPI tables\n");
        goto Failure;
    }

    RtlCopyMemory(AcpiLoaderRsdp->Signature,
                  ACPI_SIG_RSDP,
                  sizeof(AcpiLoaderRsdp->Signature));
    RtlCopyMemory(AcpiLoaderRsdp->OemId, "ROS   ", sizeof(AcpiLoaderRsdp->OemId));

    if (RtlEqualMemory(RootTable->Signature, ACPI_SIG_XSDT, ACPI_NAMESEG_SIZE))
    {
        AcpiLoaderRsdp->Revision = 2;
        AcpiLoaderRsdp->Length = sizeof(*AcpiLoaderRsdp);
        AcpiLoaderRsdp->XsdtPhysicalAddress = NodeData->RsdtAddress.QuadPart;
    }
    else if (RtlEqualMemory(RootTable->Signature, ACPI_SIG_RSDT, ACPI_NAMESEG_SIZE))
    {
        if (NodeData->RsdtAddress.QuadPart > MAXULONG)
        {
            DPRINT1("Loader RSDT address does not fit in 32 bits: 0x%I64x\n",
                    NodeData->RsdtAddress.QuadPart);
            goto Failure;
        }

        AcpiLoaderRsdp->Revision = 0;
        AcpiLoaderRsdp->RsdtPhysicalAddress = NodeData->RsdtAddress.LowPart;
    }
    else
    {
        DPRINT1("Loader ACPI root table has unexpected signature '%.4s'\n",
                RootTable->Signature);
        goto Failure;
    }

    AcpiLoaderRsdp->Checksum = AcpiChecksumBuffer((const UCHAR *)AcpiLoaderRsdp,
                                                  ACPI_RSDP_CHECKSUM_LENGTH);
    if (AcpiLoaderRsdp->Revision >= 2)
    {
        AcpiLoaderRsdp->ExtendedChecksum =
            AcpiChecksumBuffer((const UCHAR *)AcpiLoaderRsdp,
                               sizeof(*AcpiLoaderRsdp));
    }

    MmUnmapIoSpace(RootTable, sizeof(*RootTable));
    PhysicalAddress = MmGetPhysicalAddress(AcpiLoaderRsdp);
    return (ACPI_PHYSICAL_ADDRESS)PhysicalAddress.QuadPart;

Failure:
    if (AcpiLoaderRsdp)
    {
        ExFreePoolWithTag(AcpiLoaderRsdp, TAG_ACPI_LOADER_RSDP_TABLE);
        AcpiLoaderRsdp = NULL;
    }
    if (RootTable)
        MmUnmapIoSpace(RootTable, sizeof(*RootTable));
    return 0;
}

ACPI_STATUS
AcpiOsInitialize (void)
{
    DPRINT("AcpiOsInitialize called\n");

#ifndef NDEBUG
    /* Verboseness level of the acpica core */
    AcpiDbgLevel = 0x00FFFFFF;
    AcpiDbgLayer = 0xFFFFFFFF;
#endif

    (VOID)AcpiBuildLoaderRootPointer();

    return AE_OK;
}

ACPI_STATUS
AcpiOsTerminate(void)
{
    DPRINT("AcpiOsTerminate() called\n");

    /* Release the synthetic RSDP built from the loader-provided ACPI tables.
     * ACPICA has already torn down its table mappings (AcpiUtSubsystemShutdown)
     * by the time it calls us, so nothing references this buffer anymore. */
    if (AcpiLoaderRsdp != NULL)
    {
        ExFreePoolWithTag(AcpiLoaderRsdp, TAG_ACPI_LOADER_RSDP_TABLE);
        AcpiLoaderRsdp = NULL;
    }

    return AE_OK;
}

/*
 * UEFI fallback for AcpiOsGetRootPointer.
 *
 * Under UEFI boot the legacy BIOS scan (AcpiFindRootPointer) fails because the
 * RSDP does not live in the 0xE0000-0xFFFFF area.  FreeLDR records the RSDT/XSDT
 * physical address in the ARC "ACPI BIOS" multifunction-adapter configuration
 * data.  We read that address from the registry and synthesize a valid
 * ACPI_TABLE_RSDP (in a stable nonpaged-data static) that points ACPICA at the
 * XSDT.
 */
static ACPI_TABLE_RSDP AcpipSyntheticRsdp;
static ACPI_PHYSICAL_ADDRESS AcpipCachedRootPointer = 0;
static BOOLEAN AcpipRootPointerComputed = FALSE;

/*
 * Read the RSDT/XSDT physical address from the freeldr-registered
 * "ACPI BIOS" multifunction-adapter Configuration Data in the registry.
 */
static
NTSTATUS
AcpipGetRsdtAddressFromRegistry(
    PPHYSICAL_ADDRESS Out)
{
    static const WCHAR McfgPath[] =
        L"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter";
    static const WCHAR AcpiBiosId[] = L"ACPI BIOS";

    UNICODE_STRING KeyName;
    UNICODE_STRING ValueName;
    OBJECT_ATTRIBUTES ObjectAttributes;
    HANDLE RootKey = NULL;
    HANDLE SubKey = NULL;
    NTSTATUS Status;
    ULONG Index;
    PVOID Buffer = NULL;
    ULONG BufferSize = 0;
    ULONG ResultLength;
    NTSTATUS RetStatus = STATUS_NOT_FOUND;

    Out->QuadPart = 0;

    RtlInitUnicodeString(&KeyName, McfgPath);
    InitializeObjectAttributes(&ObjectAttributes,
                               &KeyName,
                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);

    Status = ZwOpenKey(&RootKey, KEY_READ, &ObjectAttributes);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("AcpiOsGetRootPointer: ZwOpenKey(MultifunctionAdapter) failed 0x%08lx\n", Status);
        return Status;
    }

    /* Walk the numbered subkeys "0", "1", "2", ... */
    for (Index = 0; ; Index++)
    {
        PKEY_BASIC_INFORMATION BasicInfo;
        WCHAR SubKeyNameBuffer[16];
        UNICODE_STRING SubKeyName;
        UCHAR BasicInfoBuffer[sizeof(KEY_BASIC_INFORMATION) + 16 * sizeof(WCHAR)];

        BasicInfo = (PKEY_BASIC_INFORMATION)BasicInfoBuffer;
        Status = ZwEnumerateKey(RootKey,
                                Index,
                                KeyBasicInformation,
                                BasicInfo,
                                sizeof(BasicInfoBuffer),
                                &ResultLength);
        if (Status == STATUS_NO_MORE_ENTRIES)
        {
            break;
        }
        if (!NT_SUCCESS(Status))
        {
            /* A too-small subkey name is unexpected here; just stop. */
            DPRINT1("AcpiOsGetRootPointer: ZwEnumerateKey(%lu) failed 0x%08lx\n", Index, Status);
            break;
        }

        if (BasicInfo->NameLength == 0 ||
            BasicInfo->NameLength >= sizeof(SubKeyNameBuffer))
        {
            continue;
        }

        RtlCopyMemory(SubKeyNameBuffer, BasicInfo->Name, BasicInfo->NameLength);
        SubKeyNameBuffer[BasicInfo->NameLength / sizeof(WCHAR)] = UNICODE_NULL;
        RtlInitUnicodeString(&SubKeyName, SubKeyNameBuffer);

        InitializeObjectAttributes(&ObjectAttributes,
                                   &SubKeyName,
                                   OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                                   RootKey,
                                   NULL);

        Status = ZwOpenKey(&SubKey, KEY_READ, &ObjectAttributes);
        if (!NT_SUCCESS(Status))
        {
            continue;
        }

        /* Query "Identifier" and check it equals "ACPI BIOS". */
        RtlInitUnicodeString(&ValueName, L"Identifier");
        Status = ZwQueryValueKey(SubKey,
                                 &ValueName,
                                 KeyValuePartialInformation,
                                 NULL,
                                 0,
                                 &ResultLength);
        if (Status == STATUS_BUFFER_TOO_SMALL || Status == STATUS_BUFFER_OVERFLOW)
        {
            PKEY_VALUE_PARTIAL_INFORMATION IdInfo;

            IdInfo = ExAllocatePoolWithTag(NonPagedPool, ResultLength, 'pcaA');
            if (IdInfo != NULL)
            {
                Status = ZwQueryValueKey(SubKey,
                                         &ValueName,
                                         KeyValuePartialInformation,
                                         IdInfo,
                                         ResultLength,
                                         &ResultLength);
                if (NT_SUCCESS(Status) &&
                    IdInfo->Type == REG_SZ &&
                    IdInfo->DataLength >= sizeof(AcpiBiosId))
                {
                    PWCHAR IdStr = (PWCHAR)IdInfo->Data;

                    /* AcpiBiosId includes its terminating NUL; this matches the
                     * REG_SZ data which is NUL-terminated as well. */
                    if (RtlCompareMemory(IdStr, AcpiBiosId, sizeof(AcpiBiosId)) ==
                        sizeof(AcpiBiosId))
                    {
                        ExFreePoolWithTag(IdInfo, 'pcaA');

                        /* Found the ACPI BIOS adapter. Read Configuration Data. */
                        RtlInitUnicodeString(&ValueName, L"Configuration Data");
                        Status = ZwQueryValueKey(SubKey,
                                                 &ValueName,
                                                 KeyValuePartialInformation,
                                                 NULL,
                                                 0,
                                                 &ResultLength);
                        if (Status == STATUS_BUFFER_TOO_SMALL ||
                            Status == STATUS_BUFFER_OVERFLOW)
                        {
                            BufferSize = ResultLength;
                            Buffer = ExAllocatePoolWithTag(NonPagedPool, BufferSize, 'pcaA');
                            if (Buffer != NULL)
                            {
                                Status = ZwQueryValueKey(SubKey,
                                                         &ValueName,
                                                         KeyValuePartialInformation,
                                                         Buffer,
                                                         BufferSize,
                                                         &ResultLength);
                                if (NT_SUCCESS(Status))
                                {
                                    RetStatus = STATUS_SUCCESS;
                                }
                            }
                        }

                        ZwClose(SubKey);
                        SubKey = NULL;
                        break;
                    }
                }

                ExFreePoolWithTag(IdInfo, 'pcaA');
            }
        }

        ZwClose(SubKey);
        SubKey = NULL;
    }

    ZwClose(RootKey);

    if (!NT_SUCCESS(RetStatus) || Buffer == NULL)
    {
        if (Buffer != NULL)
        {
            ExFreePoolWithTag(Buffer, 'pcaA');
        }
        return STATUS_NOT_FOUND;
    }

    /*
     * Parse the Configuration Data blob:
     *   KEY_VALUE_PARTIAL_INFORMATION.Data -> CM_FULL_RESOURCE_DESCRIPTOR
     *   { InterfaceType; BusNumber; PartialResourceList { ...; Count; Descriptors[] } }
     * Find the CmResourceTypeDeviceSpecific descriptor; the freeldr ACPI_BIOS_DATA
     * begins immediately after it, with PHYSICAL_ADDRESS RSDTAddress as its first
     * field.
     */
    {
        PKEY_VALUE_PARTIAL_INFORMATION ValueInfo = (PKEY_VALUE_PARTIAL_INFORMATION)Buffer;
        PUCHAR DataStart;
        PUCHAR DataEnd;
        PCM_FULL_RESOURCE_DESCRIPTOR FullDesc;
        PCM_PARTIAL_RESOURCE_LIST PartialList;
        ULONG i;
        ULONG DataLength;

        RetStatus = STATUS_NOT_FOUND;

        DataLength = ValueInfo->DataLength;
        DataStart = (PUCHAR)ValueInfo->Data;
        DataEnd = DataStart + DataLength;

        if (DataLength < sizeof(CM_FULL_RESOURCE_DESCRIPTOR))
        {
            DPRINT1("AcpiOsGetRootPointer: Configuration Data too small (%lu)\n", DataLength);
            ExFreePoolWithTag(Buffer, 'pcaA');
            return STATUS_NOT_FOUND;
        }

        FullDesc = (PCM_FULL_RESOURCE_DESCRIPTOR)DataStart;
        PartialList = &FullDesc->PartialResourceList;

        for (i = 0; i < PartialList->Count; i++)
        {
            PCM_PARTIAL_RESOURCE_DESCRIPTOR Desc = &PartialList->PartialDescriptors[i];
            PUCHAR DescBytes = (PUCHAR)Desc;

            /* Ensure the descriptor itself is fully inside the blob. */
            if (DescBytes + sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR) > DataEnd)
            {
                break;
            }

            if (Desc->Type == CmResourceTypeDeviceSpecific)
            {
                PUCHAR AcpiBiosData = DescBytes + sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);

                /* The first field of freeldr ACPI_BIOS_DATA is
                 * PHYSICAL_ADDRESS RSDTAddress (8 bytes). */
                if (AcpiBiosData + sizeof(PHYSICAL_ADDRESS) <= DataEnd)
                {
                    PHYSICAL_ADDRESS RsdtAddress;

                    RtlCopyMemory(&RsdtAddress, AcpiBiosData, sizeof(PHYSICAL_ADDRESS));
                    *Out = RsdtAddress;
                    RetStatus = STATUS_SUCCESS;
                }
                break;
            }
        }
    }

    ExFreePoolWithTag(Buffer, 'pcaA');
    return RetStatus;
}

ACPI_PHYSICAL_ADDRESS
AcpiOsGetRootPointer (
    void)
{
    ACPI_PHYSICAL_ADDRESS pa = 0;
    PHYSICAL_ADDRESS RsdtAddress;
    PHYSICAL_ADDRESS PhysicalAddress;
    NTSTATUS Status;
    PUCHAR Bytes;
    ULONG Sum;
    ULONG i;

    DPRINT("AcpiOsGetRootPointer\n");

    /* Return the cached result once computed. */
    if (AcpipRootPointerComputed)
    {
        return AcpipCachedRootPointer;
    }

    /* 0) If freeldr already synthesized an RSDP for us (e.g. UEFI boot with no
     *    legacy BIOS area to scan), it is the most reliable source: use it directly. */
    if (AcpiLoaderRsdp != NULL)
    {
        PhysicalAddress = MmGetPhysicalAddress(AcpiLoaderRsdp);
        AcpipCachedRootPointer = (ACPI_PHYSICAL_ADDRESS)PhysicalAddress.QuadPart;
        AcpipRootPointerComputed = TRUE;
        return AcpipCachedRootPointer;
    }

    /* 1) Legacy BIOS path: scan the BIOS area. Unchanged behavior. */
    AcpiFindRootPointer(&pa);
    if (pa != 0)
    {
        DPRINT1("AcpiOsGetRootPointer: BIOS scan found RSDP at 0x%I64x\n",
                (UINT64)pa);
        AcpipCachedRootPointer = pa;
        AcpipRootPointerComputed = TRUE;
        return pa;
    }

    /* 2) UEFI path: BIOS scan failed. Get the RSDT/XSDT physical address from
     *    the freeldr "ACPI BIOS" registry entry and synthesize an RSDP. */
    Status = AcpipGetRsdtAddressFromRegistry(&RsdtAddress);
    if (!NT_SUCCESS(Status) || RsdtAddress.QuadPart == 0)
    {
        DPRINT1("AcpiOsGetRootPointer: no RSDP from scan or registry!\n");
        AcpipCachedRootPointer = 0;
        AcpipRootPointerComputed = TRUE;
        return 0;
    }

    /* Synthesize a valid ACPI 2.0+ RSDP pointing at the XSDT. */
    RtlZeroMemory(&AcpipSyntheticRsdp, sizeof(AcpipSyntheticRsdp));
    RtlCopyMemory(AcpipSyntheticRsdp.Signature, "RSD PTR ", 8);   /* no NUL */
    RtlCopyMemory(AcpipSyntheticRsdp.OemId, "REACTS", ACPI_OEM_ID_SIZE);
    AcpipSyntheticRsdp.Revision = 2;
    AcpipSyntheticRsdp.Length = sizeof(ACPI_TABLE_RSDP);   /* 36 */
    AcpipSyntheticRsdp.RsdtPhysicalAddress = 0;            /* point at XSDT */
    AcpipSyntheticRsdp.XsdtPhysicalAddress = (UINT64)RsdtAddress.QuadPart;

    /* ACPI 1.0 checksum: unsigned 8-bit sum of first 20 bytes must be 0. */
    AcpipSyntheticRsdp.Checksum = 0;
    Bytes = (PUCHAR)&AcpipSyntheticRsdp;
    Sum = 0;
    for (i = 0; i < 20; i++)
    {
        Sum += Bytes[i];
    }
    AcpipSyntheticRsdp.Checksum = (UINT8)(0 - (Sum & 0xFF));

    /* Extended checksum: unsigned 8-bit sum of all 36 bytes must be 0. */
    AcpipSyntheticRsdp.ExtendedChecksum = 0;
    Sum = 0;
    for (i = 0; i < sizeof(ACPI_TABLE_RSDP); i++)
    {
        Sum += Bytes[i];
    }
    AcpipSyntheticRsdp.ExtendedChecksum = (UINT8)(0 - (Sum & 0xFF));

    pa = MmGetPhysicalAddress(&AcpipSyntheticRsdp).QuadPart;

    DPRINT1("AcpiOsGetRootPointer: BIOS scan failed, using registry RSDT/XSDT 0x%I64x, synthetic RSDP\n",
            (UINT64)RsdtAddress.QuadPart);

    AcpipCachedRootPointer = pa;
    AcpipRootPointerComputed = TRUE;
    return pa;
}

ACPI_STATUS
AcpiOsPredefinedOverride(
    const ACPI_PREDEFINED_NAMES *PredefinedObject,
    ACPI_STRING                 *NewValue)
{
    if (!PredefinedObject || !NewValue)
    {
        DPRINT1("Invalid parameter\n");
        return AE_BAD_PARAMETER;
    }

    /* No override */
    *NewValue = NULL;

    return AE_OK;
}

ACPI_STATUS
AcpiOsTableOverride(
    ACPI_TABLE_HEADER *ExistingTable,
    ACPI_TABLE_HEADER **NewTable)
{
    if (!ExistingTable || !NewTable)
    {
        DPRINT1("Invalid parameter\n");
        return AE_BAD_PARAMETER;
    }

    /* No override */
    *NewTable = NULL;

    return AE_OK;
}

ACPI_STATUS
AcpiOsPhysicalTableOverride(
    ACPI_TABLE_HEADER       *ExistingTable,
    ACPI_PHYSICAL_ADDRESS   *NewAddress,
    UINT32                  *NewTableLength)
{
    if (!ExistingTable || !NewAddress || !NewTableLength)
    {
        DPRINT1("Invalid parameter\n");
        return AE_BAD_PARAMETER;
    }

    /* No override */
    *NewAddress     = 0;
    *NewTableLength = 0;

    return AE_OK;
}

void *
AcpiOsMapMemory (
    ACPI_PHYSICAL_ADDRESS   phys,
    ACPI_SIZE               length)
{
    PHYSICAL_ADDRESS Address;
    PVOID Ptr;

    DPRINT("AcpiOsMapMemory(phys 0x%p  size 0x%X)\n", phys, length);

    Address.QuadPart = (ULONGLONG)phys;
    Ptr = MmMapIoSpace(Address, length, MmNonCached);
    if (!Ptr)
    {
        DPRINT1("Mapping failed\n");
    }

    return Ptr;
}

void
AcpiOsUnmapMemory (
    void                    *virt,
    ACPI_SIZE               length)
{
    DPRINT("AcpiOsMapMemory(phys 0x%p  size 0x%X)\n", virt, length);

    ASSERT(virt);

    MmUnmapIoSpace(virt, length);
}

ACPI_STATUS
AcpiOsGetPhysicalAddress(
    void *LogicalAddress,
    ACPI_PHYSICAL_ADDRESS *PhysicalAddress)
{
    PHYSICAL_ADDRESS PhysAddr;

    if (!LogicalAddress || !PhysicalAddress)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    PhysAddr = MmGetPhysicalAddress(LogicalAddress);

    *PhysicalAddress = (ACPI_PHYSICAL_ADDRESS)PhysAddr.QuadPart;

    return AE_OK;
}

void *
AcpiOsAllocate (ACPI_SIZE size)
{
    DPRINT("AcpiOsAllocate size %d\n",size);
    return ExAllocatePoolWithTag(NonPagedPool, size, 'ipcA');
}

void
AcpiOsFree(void *ptr)
{
    if (!ptr)
        DPRINT1("Attempt to free null pointer!!!\n");
    ExFreePoolWithTag(ptr, 'ipcA');
}

BOOLEAN
AcpiOsReadable(
    void *Memory,
    ACPI_SIZE Length)
{
    BOOLEAN Ret = FALSE;

    _SEH2_TRY
    {
        ProbeForRead(Memory, Length, sizeof(UCHAR));
        Ret = TRUE;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Ret = FALSE;
    }
    _SEH2_END;

    return Ret;
}

BOOLEAN
AcpiOsWritable(
    void *Memory,
    ACPI_SIZE Length)
{
    BOOLEAN Ret = FALSE;

    _SEH2_TRY
    {
        ProbeForWrite(Memory, Length, sizeof(UCHAR));
        Ret = TRUE;
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Ret = FALSE;
    }
    _SEH2_END;

    return Ret;
}

ACPI_THREAD_ID
AcpiOsGetThreadId (void)
{
    /* Thread ID must be non-zero */
    return (ULONG_PTR)PsGetCurrentThreadId() + 1;
}

ACPI_STATUS
AcpiOsExecute (
    ACPI_EXECUTE_TYPE       Type,
    ACPI_OSD_EXEC_CALLBACK  Function,
    void                    *Context)
{
    HANDLE ThreadHandle;
    OBJECT_ATTRIBUTES ObjectAttributes;
    NTSTATUS Status;

    DPRINT("AcpiOsExecute\n");

    InitializeObjectAttributes(&ObjectAttributes,
                               NULL,
                               OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);

    Status = PsCreateSystemThread(&ThreadHandle,
                                  THREAD_ALL_ACCESS,
                                  &ObjectAttributes,
                                  NULL,
                                  NULL,
                                  (PKSTART_ROUTINE)Function,
                                  Context);
    if (!NT_SUCCESS(Status))
        return AE_ERROR;

    ZwClose(ThreadHandle);

    return AE_OK;
}

void
AcpiOsSleep (UINT64 milliseconds)
{
    DPRINT("AcpiOsSleep %d\n", milliseconds);
    KeStallExecutionProcessor(milliseconds*1000);
}

void
AcpiOsStall (UINT32 microseconds)
{
    DPRINT("AcpiOsStall %d\n",microseconds);
    KeStallExecutionProcessor(microseconds);
}

ACPI_STATUS
AcpiOsCreateMutex(
    ACPI_MUTEX *OutHandle)
{
    PFAST_MUTEX Mutex;

    if (!OutHandle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    Mutex = ExAllocatePoolWithTag(NonPagedPool, sizeof(FAST_MUTEX), 'LpcA');
    if (!Mutex) return AE_NO_MEMORY;

    ExInitializeFastMutex(Mutex);

    *OutHandle = (ACPI_MUTEX)Mutex;

    return AE_OK;
}

void
AcpiOsDeleteMutex(
    ACPI_MUTEX Handle)
{
    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return;
    }

    ExFreePoolWithTag(Handle, 'LpcA');
}

ACPI_STATUS
AcpiOsAcquireMutex(
    ACPI_MUTEX Handle,
    UINT16 Timeout)
{
    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    /* Check what the caller wants us to do */
    if (Timeout == ACPI_DO_NOT_WAIT)
    {
        /* Try to acquire without waiting */
        if (!ExTryToAcquireFastMutex((PFAST_MUTEX)Handle))
            return AE_TIME;
    }
    else
    {
        /* Block until we get it */
        ExAcquireFastMutex((PFAST_MUTEX)Handle);
    }

    return AE_OK;
}

void
AcpiOsReleaseMutex(
    ACPI_MUTEX Handle)
{
    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return;
    }

    ExReleaseFastMutex((PFAST_MUTEX)Handle);
}

typedef struct _ACPI_SEM {
    UINT32 CurrentUnits;
    KEVENT Event;
    KSPIN_LOCK Lock;
} ACPI_SEM, *PACPI_SEM;

ACPI_STATUS
AcpiOsCreateSemaphore(
    UINT32 MaxUnits,
    UINT32 InitialUnits,
    ACPI_SEMAPHORE *OutHandle)
{
    PACPI_SEM Sem;

    if (!OutHandle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    Sem = ExAllocatePoolWithTag(NonPagedPool, sizeof(ACPI_SEM), 'LpcA');
    if (!Sem) return AE_NO_MEMORY;

    Sem->CurrentUnits = InitialUnits;
    KeInitializeEvent(&Sem->Event, SynchronizationEvent, Sem->CurrentUnits != 0);
    KeInitializeSpinLock(&Sem->Lock);

    *OutHandle = (ACPI_SEMAPHORE)Sem;

    return AE_OK;
}

ACPI_STATUS
AcpiOsDeleteSemaphore(
    ACPI_SEMAPHORE Handle)
{
    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    ExFreePoolWithTag(Handle, 'LpcA');

    return AE_OK;
}

ACPI_STATUS
AcpiOsWaitSemaphore(
    ACPI_SEMAPHORE Handle,
    UINT32 Units,
    UINT16 Timeout)
{
    PACPI_SEM Sem = Handle;
    KIRQL OldIrql;

    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    KeAcquireSpinLock(&Sem->Lock, &OldIrql);

    /* Make sure we can wait if we have fewer units than we need */
    if ((Timeout == ACPI_DO_NOT_WAIT) && (Sem->CurrentUnits < Units))
    {
        /* We can't so we must bail now */
        KeReleaseSpinLock(&Sem->Lock, OldIrql);
        return AE_TIME;
    }

    /* Time to block until we get enough units */
    while (Sem->CurrentUnits < Units)
    {
        KeReleaseSpinLock(&Sem->Lock, OldIrql);
        KeWaitForSingleObject(&Sem->Event,
                              Executive,
                              KernelMode,
                              FALSE,
                              NULL);
        KeAcquireSpinLock(&Sem->Lock, &OldIrql);
    }

    Sem->CurrentUnits -= Units;

    if (Sem->CurrentUnits != 0) KeSetEvent(&Sem->Event, IO_NO_INCREMENT, FALSE);

    KeReleaseSpinLock(&Sem->Lock, OldIrql);

    return AE_OK;
}

ACPI_STATUS
AcpiOsSignalSemaphore(
    ACPI_SEMAPHORE Handle,
    UINT32 Units)
{
    PACPI_SEM Sem = Handle;
    KIRQL OldIrql;

    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    KeAcquireSpinLock(&Sem->Lock, &OldIrql);

    Sem->CurrentUnits += Units;
    KeSetEvent(&Sem->Event, IO_NO_INCREMENT, FALSE);

    KeReleaseSpinLock(&Sem->Lock, OldIrql);

    return AE_OK;
}

ACPI_STATUS
AcpiOsCreateLock(
    ACPI_SPINLOCK *OutHandle)
{
    PKSPIN_LOCK SpinLock;

    if (!OutHandle)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    SpinLock = ExAllocatePoolWithTag(NonPagedPool, sizeof(KSPIN_LOCK), 'LpcA');
    if (!SpinLock) return AE_NO_MEMORY;

    KeInitializeSpinLock(SpinLock);

    *OutHandle = (ACPI_SPINLOCK)SpinLock;

    return AE_OK;
}

void
AcpiOsDeleteLock(
    ACPI_SPINLOCK Handle)
{
    if (!Handle)
    {
        DPRINT1("Bad parameter\n");
        return;
    }

    ExFreePoolWithTag(Handle, 'LpcA');
}

ACPI_CPU_FLAGS
AcpiOsAcquireLock(
    ACPI_SPINLOCK Handle)
{
    KIRQL OldIrql;

    if ((OldIrql = KeGetCurrentIrql()) >= DISPATCH_LEVEL)
    {
        KeAcquireSpinLockAtDpcLevel((PKSPIN_LOCK)Handle);
    }
    else
    {
        KeAcquireSpinLock((PKSPIN_LOCK)Handle, &OldIrql);
    }

    return (ACPI_CPU_FLAGS)OldIrql;
}

void
AcpiOsReleaseLock(
    ACPI_SPINLOCK Handle,
    ACPI_CPU_FLAGS Flags)
{
    KIRQL OldIrql = (KIRQL)Flags;

    if (OldIrql >= DISPATCH_LEVEL)
    {
        KeReleaseSpinLockFromDpcLevel((PKSPIN_LOCK)Handle);
    }
    else
    {
        KeReleaseSpinLock((PKSPIN_LOCK)Handle, OldIrql);
    }
}

BOOLEAN NTAPI
OslIsrStub(
  PKINTERRUPT Interrupt,
  PVOID ServiceContext)
{
  INT32 Status;

  Status = (*AcpiIrqHandler)(AcpiIrqContext);

  if (Status == ACPI_INTERRUPT_HANDLED)
    return TRUE;
  else
    return FALSE;
}

UINT32
AcpiOsInstallInterruptHandler (
    UINT32                  InterruptNumber,
    ACPI_OSD_HANDLER        ServiceRoutine,
    void                    *Context)
{
    ULONG Vector;
    KIRQL DIrql;
    KAFFINITY Affinity;
    NTSTATUS Status;

    if (AcpiInterruptHandlerRegistered)
    {
        DPRINT1("Reregister interrupt attempt failed\n");
        return AE_ALREADY_EXISTS;
    }

    if (!ServiceRoutine)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    DPRINT("AcpiOsInstallInterruptHandler()\n");
    Vector = HalGetInterruptVector(
        Internal,
        0,
        InterruptNumber,
        InterruptNumber,
        &DIrql,
        &Affinity);

    AcpiIrqNumber = InterruptNumber;
    AcpiIrqHandler = ServiceRoutine;
    AcpiIrqContext = Context;
    AcpiInterruptHandlerRegistered = TRUE;

    Status = IoConnectInterrupt(
        &AcpiInterrupt,
        OslIsrStub,
        NULL,
        NULL,
        Vector,
        DIrql,
        DIrql,
        LevelSensitive,
        TRUE,
        Affinity,
        FALSE);

    if (!NT_SUCCESS(Status))
    {
        DPRINT("Could not connect to interrupt %d\n", Vector);
        return AE_ERROR;
    }
    return AE_OK;
}

ACPI_STATUS
AcpiOsRemoveInterruptHandler (
    UINT32                  InterruptNumber,
    ACPI_OSD_HANDLER        ServiceRoutine)
{
    DPRINT("AcpiOsRemoveInterruptHandler()\n");

    if (!ServiceRoutine)
    {
        DPRINT1("Bad parameter\n");
        return AE_BAD_PARAMETER;
    }

    if (AcpiInterruptHandlerRegistered)
    {
        IoDisconnectInterrupt(AcpiInterrupt);
        AcpiInterrupt = NULL;
        AcpiInterruptHandlerRegistered = FALSE;
    }
    else
    {
        DPRINT1("Trying to remove non-existing interrupt handler\n");
        return AE_NOT_EXIST;
    }

    return AE_OK;
}

ACPI_STATUS
AcpiOsReadMemory (
    ACPI_PHYSICAL_ADDRESS   Address,
    UINT64                  *Value,
    UINT32                  Width)
{
    DPRINT("AcpiOsReadMemory %p\n", Address);
    switch (Width)
    {
    case 8:
        *Value = (*(PUCHAR)(ULONG_PTR)Address);
        break;

    case 16:
        *Value = (*(PUSHORT)(ULONG_PTR)Address);
        break;

    case 32:
        *Value = (*(PULONG)(ULONG_PTR)Address);
        break;

    case 64:
        *Value = (*(PULONGLONG)(ULONG_PTR)Address);
        break;

    default:
        DPRINT1("AcpiOsReadMemory got bad width: %d\n",Width);
        return (AE_BAD_PARAMETER);
        break;
    }
    return (AE_OK);
}

ACPI_STATUS
AcpiOsWriteMemory (
    ACPI_PHYSICAL_ADDRESS   Address,
    UINT64                  Value,
    UINT32                  Width)
{
    DPRINT("AcpiOsWriteMemory %p\n", Address);
    switch (Width)
    {
    case 8:
        *(PUCHAR)(ULONG_PTR)Address = Value;
        break;

    case 16:
        *(PUSHORT)(ULONG_PTR)Address = Value;
        break;

    case 32:
        *(PULONG)(ULONG_PTR)Address = Value;
        break;

    case 64:
        *(PULONGLONG)(ULONG_PTR)Address = Value;
        break;

    default:
        DPRINT1("AcpiOsWriteMemory got bad width: %d\n",Width);
        return (AE_BAD_PARAMETER);
        break;
    }

    return (AE_OK);
}

ACPI_STATUS
AcpiOsReadPort (
    ACPI_IO_ADDRESS         Address,
    UINT32                  *Value,
    UINT32                  Width)
{
    DPRINT("AcpiOsReadPort %p, width %d\n",Address,Width);

    switch (Width)
    {
    case 8:
        *Value = READ_PORT_UCHAR((PUCHAR)(ULONG_PTR)Address);
        break;

    case 16:
        *Value = READ_PORT_USHORT((PUSHORT)(ULONG_PTR)Address);
        break;

    case 32:
        *Value = READ_PORT_ULONG((PULONG)(ULONG_PTR)Address);
        break;

    default:
        DPRINT1("AcpiOsReadPort got bad width: %d\n",Width);
        return (AE_BAD_PARAMETER);
        break;
    }
    return (AE_OK);
}

ACPI_STATUS
AcpiOsWritePort (
    ACPI_IO_ADDRESS         Address,
    UINT32                  Value,
    UINT32                  Width)
{
    DPRINT("AcpiOsWritePort %p, width %d\n",Address,Width);
    switch (Width)
    {
    case 8:
        WRITE_PORT_UCHAR((PUCHAR)(ULONG_PTR)Address, Value);
        break;

    case 16:
        WRITE_PORT_USHORT((PUSHORT)(ULONG_PTR)Address, Value);
        break;

    case 32:
        WRITE_PORT_ULONG((PULONG)(ULONG_PTR)Address, Value);
        break;

    default:
        DPRINT1("AcpiOsWritePort got bad width: %d\n",Width);
        return (AE_BAD_PARAMETER);
        break;
    }
    return (AE_OK);
}

BOOLEAN
OslIsPciDevicePresent(ULONG BusNumber, ULONG SlotNumber)
{
    UINT32 ReadLength;
    PCI_COMMON_CONFIG PciConfig;

    /* Detect device presence by reading the PCI configuration space */

    ReadLength = HalGetBusDataByOffset(PCIConfiguration,
                                       BusNumber,
                                       SlotNumber,
                                       &PciConfig,
                                       0,
                                       sizeof(PciConfig));
    if (ReadLength == 0)
    {
        DPRINT("PCI device is not present\n");
        return FALSE;
    }

    ASSERT(ReadLength >= 2);

    if (PciConfig.VendorID == PCI_INVALID_VENDORID)
    {
        DPRINT("Invalid vendor ID in PCI configuration space\n");
        return FALSE;
    }

    DPRINT("PCI device is present\n");

    return TRUE;
}

ACPI_STATUS
AcpiOsReadPciConfiguration (
    ACPI_PCI_ID             *PciId,
    UINT32                  Reg,
    UINT64                  *Value,
    UINT32                  Width)
{
    PCI_SLOT_NUMBER slot;

    slot.u.AsULONG = 0;
    slot.u.bits.DeviceNumber = PciId->Device;
    slot.u.bits.FunctionNumber = PciId->Function;

    DPRINT("AcpiOsReadPciConfiguration, slot=0x%X, func=0x%X\n", slot.u.AsULONG, Reg);

    if (!OslIsPciDevicePresent(PciId->Bus, slot.u.AsULONG))
        return AE_NOT_FOUND;

    /* Width is in BITS */
    HalGetBusDataByOffset(PCIConfiguration,
        PciId->Bus,
        slot.u.AsULONG,
        Value,
        Reg,
        (Width >> 3));

    return AE_OK;
}

ACPI_STATUS
AcpiOsWritePciConfiguration (
    ACPI_PCI_ID              *PciId,
    UINT32                   Reg,
    UINT64                   Value,
    UINT32                   Width)
{
    ULONG buf = Value;
    PCI_SLOT_NUMBER slot;

    slot.u.AsULONG = 0;
    slot.u.bits.DeviceNumber = PciId->Device;
    slot.u.bits.FunctionNumber = PciId->Function;

    DPRINT("AcpiOsWritePciConfiguration, slot=0x%x\n", slot.u.AsULONG);
    if (!OslIsPciDevicePresent(PciId->Bus, slot.u.AsULONG))
        return AE_NOT_FOUND;

    /* Width is in BITS */
    HalSetBusDataByOffset(PCIConfiguration,
        PciId->Bus,
        slot.u.AsULONG,
        &buf,
        Reg,
        (Width >> 3));

    return AE_OK;
}

void ACPI_INTERNAL_VAR_XFACE
AcpiOsPrintf (
    const char              *Fmt,
    ...)
{
    va_list                 Args;
    va_start (Args, Fmt);

    AcpiOsVprintf (Fmt, Args);

    va_end (Args);
    return;
}

void
AcpiOsVprintf (
    const char              *Fmt,
    va_list                 Args)
{
#ifndef NDEBUG
    vDbgPrintEx (-1, DPFLTR_ERROR_LEVEL, Fmt, Args);
#endif
    return;
}

void
AcpiOsRedirectOutput(
    void *Destination)
{
    /* No-op */
    DPRINT1("Output redirection not supported\n");
}

UINT64
AcpiOsGetTimer(
    void)
{
    LARGE_INTEGER CurrentTime;

    KeQuerySystemTime(&CurrentTime);
    return CurrentTime.QuadPart;
}

void
AcpiOsWaitEventsComplete(void)
{
    /*
     * Wait for all asynchronous events to complete.
     * This implementation does nothing.
     */
    return;
}

ACPI_STATUS
AcpiOsSignal (
    UINT32                  Function,
    void                    *Info)
{
    ACPI_SIGNAL_FATAL_INFO *FatalInfo = Info;

    switch (Function)
    {
    case ACPI_SIGNAL_FATAL:
        if (Info)
            DPRINT1 ("AcpiOsBreakpoint: %d %d %d ****\n", FatalInfo->Type, FatalInfo->Code, FatalInfo->Argument);
        else
            DPRINT1 ("AcpiOsBreakpoint ****\n");
        break;
    case ACPI_SIGNAL_BREAKPOINT:
        if (Info)
            DPRINT1 ("AcpiOsBreakpoint: %s ****\n", Info);
        else
            DPRINT1 ("AcpiOsBreakpoint ****\n");
        break;
    }

    ASSERT(FALSE);

    return (AE_OK);
}

ACPI_STATUS
AcpiOsEnterSleep(
    UINT8 SleepState,
    UINT32 RegaValue,
    UINT32 RegbValue)
{
    DPRINT1("Entering sleep state S%u.\n", SleepState);
    return AE_OK;
}

ACPI_STATUS
AcpiOsGetLine(
    char *Buffer,
    UINT32 BufferLength,
    UINT32 *BytesRead)
{
    DPRINT1("File reading not supported\n");
    return AE_ERROR;
}
