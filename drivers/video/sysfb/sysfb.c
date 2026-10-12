/*
 * sysfb.sys — Standalone ARM64 video device driver for QEMU ramfb/GOP framebuffer
 *
 * Reads framebuffer info from ARC config tree (populated by FreeLDR from UEFI GOP),
 * creates \Device\Video0, writes DEVICEMAP\VIDEO registry entries, and handles
 * video IOCTLs needed by framebuf.dll display driver.
 *
 * No videoprt.sys dependency — fully standalone kernel driver.
 */

#include <ntifs.h>
#include <ntstrsafe.h>
#include <ntddvdeo.h>
#include <reactos/drivers/bootvid/framebuf.h>

#define DPRINT1 DbgPrint

/* Device extension */
typedef struct _SYSFB_DEVICE_EXTENSION {
    PHYSICAL_ADDRESS FrameBufferPhysical;
    ULONG            FrameBufferSize;
    ULONG            ScreenWidth;
    ULONG            ScreenHeight;
    ULONG            PixelsPerScanLine;
    ULONG            BitsPerPixel;
    ULONG            ScreenStride;
    ULONG            RedMask;
    ULONG            GreenMask;
    ULONG            BlueMask;

    /* Mapped framebuffer VA (from MmMapIoSpace) */
    PVOID            FrameBufferMapped;
    ULONG            FrameBufferMappedSize;

    /* Win32k callback info */
    PVOID            PhysDisp;
    PVOID            Callout;
} SYSFB_DEVICE_EXTENSION, *PSYSFB_DEVICE_EXTENSION;

/* Forward declarations */
static NTSTATUS NTAPI SysFbCreateClose(PDEVICE_OBJECT DeviceObject, PIRP Irp);
static NTSTATUS NTAPI SysFbDeviceControl(PDEVICE_OBJECT DeviceObject, PIRP Irp);
static NTSTATUS SysFbDiscoverFramebuffer(PSYSFB_DEVICE_EXTENSION DevExt);
static NTSTATUS SysFbWriteDeviceMap(void);
static NTSTATUS SysFbWriteServiceConfig(PSYSFB_DEVICE_EXTENSION DevExt);

/* Helper: set REG_SZ value */
static NTSTATUS
RegSetSz(HANDLE KeyHandle, PCWSTR ValueName, PCWSTR Data)
{
    UNICODE_STRING Name;
    RtlInitUnicodeString(&Name, ValueName);
    return ZwSetValueKey(KeyHandle, &Name, 0, REG_SZ,
                         (PVOID)Data,
                         (ULONG)((wcslen(Data) + 1) * sizeof(WCHAR)));
}

/* Helper: set REG_MULTI_SZ value */
static NTSTATUS
RegSetMultiSz(HANDLE KeyHandle, PCWSTR ValueName, PCWSTR Data, ULONG DataSize)
{
    UNICODE_STRING Name;
    RtlInitUnicodeString(&Name, ValueName);
    return ZwSetValueKey(KeyHandle, &Name, 0, REG_MULTI_SZ, (PVOID)Data, DataSize);
}

/* Helper: set REG_DWORD value */
static NTSTATUS
RegSetDword(HANDLE KeyHandle, PCWSTR ValueName, ULONG Value)
{
    UNICODE_STRING Name;
    RtlInitUnicodeString(&Name, ValueName);
    return ZwSetValueKey(KeyHandle, &Name, 0, REG_DWORD, &Value, sizeof(ULONG));
}

/* Helper: create/open a registry key */
static NTSTATUS
RegCreateKey(HANDLE ParentKey, PCWSTR SubKeyName, PHANDLE OutHandle)
{
    OBJECT_ATTRIBUTES ObjectAttributes;
    UNICODE_STRING Name;
    ULONG Disposition;

    RtlInitUnicodeString(&Name, SubKeyName);
    InitializeObjectAttributes(&ObjectAttributes, &Name,
                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               ParentKey, NULL);
    return ZwCreateKey(OutHandle, KEY_ALL_ACCESS, &ObjectAttributes,
                       0, NULL, REG_OPTION_VOLATILE, &Disposition);
}

NTSTATUS
NTAPI
DriverEntry(
    PDRIVER_OBJECT DriverObject,
    PUNICODE_STRING RegistryPath)
{
    NTSTATUS Status;
    PDEVICE_OBJECT DeviceObject;
    UNICODE_STRING DeviceName, SymlinkName;
    PSYSFB_DEVICE_EXTENSION DevExt;

    DPRINT1("[sysfb] DriverEntry\n");

    /* Set up dispatch routines */
    DriverObject->MajorFunction[IRP_MJ_CREATE] = SysFbCreateClose;
    DriverObject->MajorFunction[IRP_MJ_CLOSE] = SysFbCreateClose;
    DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = SysFbDeviceControl;

    /* Create \Device\Video0 */
    RtlInitUnicodeString(&DeviceName, L"\\Device\\Video0");
    Status = IoCreateDevice(DriverObject, sizeof(SYSFB_DEVICE_EXTENSION),
                            &DeviceName, FILE_DEVICE_VIDEO, 0, FALSE,
                            &DeviceObject);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] IoCreateDevice failed: 0x%08lx\n", Status);
        return Status;
    }

    /* Create \??\DISPLAY1 symlink */
    RtlInitUnicodeString(&SymlinkName, L"\\??\\DISPLAY1");
    Status = IoCreateSymbolicLink(&SymlinkName, &DeviceName);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] IoCreateSymbolicLink failed: 0x%08lx\n", Status);
        /* Non-fatal — continue */
    }

    /* Initialize device extension */
    DevExt = (PSYSFB_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
    RtlZeroMemory(DevExt, sizeof(SYSFB_DEVICE_EXTENSION));

    /* Discover framebuffer from ARC config tree in registry */
    Status = SysFbDiscoverFramebuffer(DevExt);
    if (!NT_SUCCESS(Status))
    {
        /* No GOP/linear framebuffer in the ARC config tree. This is the
         * normal case on a legacy-BIOS boot (no UEFI GOP) where the VGA
         * miniport owns the display. We MUST bail here rather than register
         * a bogus framebuffer: doing so would steal \Device\Video0 from VGA
         * and point framebuf.dll at invalid memory, blanking the screen on
         * BIOS. On a UEFI boot the framebuffer is always present, so this
         * path is only taken when there is genuinely nothing for us to drive.
         */
        DPRINT1("[sysfb] No ARC framebuffer (BIOS/no-GOP boot): 0x%08lx — "
                "bailing, leaving display to VGA\n", Status);
        IoDeleteSymbolicLink(&SymlinkName);
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    DPRINT1("[sysfb] Framebuffer: %ldx%ld %ld BPP, phys=0x%I64x, size=0x%lx, stride=%ld\n",
            DevExt->ScreenWidth, DevExt->ScreenHeight, DevExt->BitsPerPixel,
            DevExt->FrameBufferPhysical.QuadPart, DevExt->FrameBufferSize,
            DevExt->ScreenStride);

    /* Write DEVICEMAP\VIDEO registry entries */
    Status = SysFbWriteDeviceMap();
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Failed to write DEVICEMAP\\VIDEO: 0x%08lx\n", Status);
        /* Non-fatal — win32k may not find us but driver still works */
    }

    /* Write service config (InstalledDisplayDrivers etc.) */
    Status = SysFbWriteServiceConfig(DevExt);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Failed to write service config: 0x%08lx\n", Status);
    }

    DPRINT1("[sysfb] DriverEntry SUCCESS\n");
    return STATUS_SUCCESS;
}

/*
 * IRP_MJ_CREATE / IRP_MJ_CLOSE — just succeed
 */
static NTSTATUS NTAPI
SysFbCreateClose(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    Irp->IoStatus.Status = STATUS_SUCCESS;
    Irp->IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return STATUS_SUCCESS;
}

/*
 * IRP_MJ_DEVICE_CONTROL — handle video IOCTLs
 */
static NTSTATUS NTAPI
SysFbDeviceControl(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
    PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation(Irp);
    PSYSFB_DEVICE_EXTENSION DevExt = DeviceObject->DeviceExtension;
    ULONG IoControlCode = IrpSp->Parameters.DeviceIoControl.IoControlCode;
    PVOID InputBuffer = Irp->AssociatedIrp.SystemBuffer;
    PVOID OutputBuffer = Irp->AssociatedIrp.SystemBuffer;
    ULONG InputLength = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
    ULONG OutputLength = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG Information = 0;

    switch (IoControlCode)
    {
    case IOCTL_VIDEO_INIT_WIN32K_CALLBACKS:
    {
        PVIDEO_WIN32K_CALLBACKS pCallbacks;

        DPRINT1("[sysfb] IOCTL_VIDEO_INIT_WIN32K_CALLBACKS\n");

        if (InputLength < sizeof(VIDEO_WIN32K_CALLBACKS) ||
            OutputLength < sizeof(VIDEO_WIN32K_CALLBACKS))
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        pCallbacks = (PVIDEO_WIN32K_CALLBACKS)InputBuffer;

        /* Store the callback info */
        DevExt->PhysDisp = pCallbacks->PhysDisp;
        DevExt->Callout = (PVOID)pCallbacks->Callout;

        /* Fill output */
        pCallbacks->bACPI = FALSE;
        pCallbacks->pPhysDeviceObject = DeviceObject;
        pCallbacks->DualviewFlags = 0;

        Information = sizeof(VIDEO_WIN32K_CALLBACKS);
        break;
    }

    case IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES:
    {
        PVIDEO_NUM_MODES pModes;

        DPRINT1("[sysfb] IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES\n");

        if (OutputLength < sizeof(VIDEO_NUM_MODES))
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        pModes = (PVIDEO_NUM_MODES)OutputBuffer;
        pModes->NumModes = 1;
        pModes->ModeInformationLength = sizeof(VIDEO_MODE_INFORMATION);
        Information = sizeof(VIDEO_NUM_MODES);
        break;
    }

    case IOCTL_VIDEO_QUERY_AVAIL_MODES:
    case IOCTL_VIDEO_QUERY_CURRENT_MODE:
    {
        PVIDEO_MODE_INFORMATION pModeInfo;

        DPRINT1("[sysfb] IOCTL_VIDEO_QUERY_%s_MODE(S)\n",
                IoControlCode == IOCTL_VIDEO_QUERY_AVAIL_MODES ? "AVAIL" : "CURRENT");

        if (OutputLength < sizeof(VIDEO_MODE_INFORMATION))
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        pModeInfo = (PVIDEO_MODE_INFORMATION)OutputBuffer;
        RtlZeroMemory(pModeInfo, sizeof(VIDEO_MODE_INFORMATION));

        pModeInfo->Length = sizeof(VIDEO_MODE_INFORMATION);
        pModeInfo->ModeIndex = 0;
        pModeInfo->VisScreenWidth = DevExt->ScreenWidth;
        pModeInfo->VisScreenHeight = DevExt->ScreenHeight;
        pModeInfo->ScreenStride = DevExt->ScreenStride;
        pModeInfo->NumberOfPlanes = 1;
        pModeInfo->BitsPerPlane = DevExt->BitsPerPixel;
        pModeInfo->Frequency = 60;
        pModeInfo->XMillimeter = 320;
        pModeInfo->YMillimeter = 240;
        pModeInfo->NumberRedBits = 8;
        pModeInfo->NumberGreenBits = 8;
        pModeInfo->NumberBlueBits = 8;
        pModeInfo->RedMask = DevExt->RedMask;
        pModeInfo->GreenMask = DevExt->GreenMask;
        pModeInfo->BlueMask = DevExt->BlueMask;
        pModeInfo->AttributeFlags = VIDEO_MODE_COLOR | VIDEO_MODE_GRAPHICS | VIDEO_MODE_LINEAR;
        pModeInfo->VideoMemoryBitmapWidth = DevExt->ScreenWidth;
        pModeInfo->VideoMemoryBitmapHeight = DevExt->ScreenHeight;
        pModeInfo->DriverSpecificAttributeFlags = 0;

        Information = sizeof(VIDEO_MODE_INFORMATION);
        break;
    }

    case IOCTL_VIDEO_SET_CURRENT_MODE:
    {
        DPRINT1("[sysfb] IOCTL_VIDEO_SET_CURRENT_MODE\n");
        /* Mode already set by UEFI GOP — nothing to do */
        Status = STATUS_SUCCESS;
        break;
    }

    case IOCTL_VIDEO_MAP_VIDEO_MEMORY:
    {
        PVIDEO_MEMORY_INFORMATION pVideoMemoryInfo;

        DPRINT1("[sysfb] IOCTL_VIDEO_MAP_VIDEO_MEMORY\n");

        if (InputLength < sizeof(VIDEO_MEMORY) ||
            OutputLength < sizeof(VIDEO_MEMORY_INFORMATION))
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        pVideoMemoryInfo = (PVIDEO_MEMORY_INFORMATION)OutputBuffer;

        if (DevExt->FrameBufferPhysical.QuadPart == 0)
        {
            DPRINT1("[sysfb] No framebuffer physical address!\n");
            Status = STATUS_UNSUCCESSFUL;
            break;
        }

        /* Map the framebuffer */
        if (!DevExt->FrameBufferMapped)
        {
            DevExt->FrameBufferMappedSize = DevExt->FrameBufferSize;
            DevExt->FrameBufferMapped = MmMapIoSpace(
                DevExt->FrameBufferPhysical,
                DevExt->FrameBufferMappedSize,
                MmNonCached);

            if (!DevExt->FrameBufferMapped)
            {
                DPRINT1("[sysfb] MmMapIoSpace failed for 0x%I64x size 0x%lx\n",
                        DevExt->FrameBufferPhysical.QuadPart, DevExt->FrameBufferMappedSize);
                Status = STATUS_UNSUCCESSFUL;
                break;
            }

            DPRINT1("[sysfb] Framebuffer mapped at %p (size 0x%lx)\n",
                    DevExt->FrameBufferMapped, DevExt->FrameBufferMappedSize);
        }

        pVideoMemoryInfo->VideoRamBase = DevExt->FrameBufferMapped;
        pVideoMemoryInfo->VideoRamLength = DevExt->FrameBufferMappedSize;
        pVideoMemoryInfo->FrameBufferBase = DevExt->FrameBufferMapped;
        pVideoMemoryInfo->FrameBufferLength = DevExt->FrameBufferMappedSize;

        Information = sizeof(VIDEO_MEMORY_INFORMATION);
        break;
    }

    case IOCTL_VIDEO_UNMAP_VIDEO_MEMORY:
    {
        DPRINT1("[sysfb] IOCTL_VIDEO_UNMAP_VIDEO_MEMORY\n");

        if (DevExt->FrameBufferMapped)
        {
            MmUnmapIoSpace(DevExt->FrameBufferMapped, DevExt->FrameBufferMappedSize);
            DevExt->FrameBufferMapped = NULL;
            DevExt->FrameBufferMappedSize = 0;
        }
        break;
    }

    case IOCTL_VIDEO_RESET_DEVICE:
        DPRINT1("[sysfb] IOCTL_VIDEO_RESET_DEVICE\n");
        /* No-op */
        break;

    case IOCTL_VIDEO_SET_COLOR_REGISTERS:
        DPRINT1("[sysfb] IOCTL_VIDEO_SET_COLOR_REGISTERS (no-op for 32bpp)\n");
        /* No-op for 32bpp direct color */
        break;

    case IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES:
    {
        PVIDEO_COLOR_CAPABILITIES pColorCaps;

        DPRINT1("[sysfb] IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES\n");

        if (OutputLength < sizeof(VIDEO_COLOR_CAPABILITIES))
        {
            Status = STATUS_BUFFER_TOO_SMALL;
            break;
        }

        pColorCaps = (PVIDEO_COLOR_CAPABILITIES)OutputBuffer;
        RtlZeroMemory(pColorCaps, sizeof(VIDEO_COLOR_CAPABILITIES));
        pColorCaps->Length = sizeof(VIDEO_COLOR_CAPABILITIES);
        pColorCaps->AttributeFlags = VIDEO_DEVICE_COLOR;
        pColorCaps->RedPhosphoreDecay = 0;
        pColorCaps->GreenPhosphoreDecay = 0;
        pColorCaps->BluePhosphoreDecay = 0;
        pColorCaps->WhiteChromaticity_x = 3127;
        pColorCaps->WhiteChromaticity_y = 3290;
        pColorCaps->WhiteChromaticity_Y = 0;
        pColorCaps->RedChromaticity_x = 6700;
        pColorCaps->RedChromaticity_y = 3300;
        pColorCaps->GreenChromaticity_x = 2100;
        pColorCaps->GreenChromaticity_y = 7100;
        pColorCaps->BlueChromaticity_x = 1400;
        pColorCaps->BlueChromaticity_y = 800;
        pColorCaps->WhiteGamma = 20000;
        pColorCaps->RedGamma = 20000;
        pColorCaps->GreenGamma = 20000;
        pColorCaps->BlueGamma = 20000;

        Information = sizeof(VIDEO_COLOR_CAPABILITIES);
        break;
    }

    default:
        DPRINT1("[sysfb] Unhandled IOCTL 0x%08lx\n", IoControlCode);
        Status = STATUS_NOT_IMPLEMENTED;
        break;
    }

    Irp->IoStatus.Status = Status;
    Irp->IoStatus.Information = Information;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}

/*
 * Discover framebuffer from ARC config tree in registry.
 * FreeLDR stores: HKLM\HARDWARE\DESCRIPTION\System\MultifunctionAdapter\0\DisplayController\0
 *   - "Configuration Data" = REG_FULL_RESOURCE_DESCRIPTOR with:
 *     [0] CmResourceTypeMemory: framebuffer physical address + size
 *     [1] CmResourceTypeDeviceSpecific: CM_FRAMEBUF_DEVICE_DATA
 */
static NTSTATUS
SysFbDiscoverFramebuffer(PSYSFB_DEVICE_EXTENSION DevExt)
{
    NTSTATUS Status;
    HANDLE hDisplay = NULL;
    OBJECT_ATTRIBUTES ObjectAttributes;
    UNICODE_STRING KeyName, ValueName;
    ULONG ResultLength;
    UCHAR *Buffer = NULL;
    ULONG BufferSize;
    PKEY_VALUE_PARTIAL_INFORMATION ValueInfo;
    PCM_FULL_RESOURCE_DESCRIPTOR FullResDesc;
    PCM_PARTIAL_RESOURCE_LIST PartialList;
    PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDesc;
    PCM_FRAMEBUF_DEVICE_DATA FramebufData;
    ULONG i;
    WCHAR PathBuf[128];
    ULONG AdapterIdx, InstanceIdx;
    BOOLEAN Found = FALSE;

    /*
     * ARC config tree registry structure:
     *   HARDWARE\DESCRIPTION\System\MultifunctionAdapter\<N>\DisplayController\<M>
     * Try adapter indices 0..3, instance 0 for each.
     */
    for (AdapterIdx = 0; AdapterIdx < 4 && !Found; AdapterIdx++)
    {
        for (InstanceIdx = 0; InstanceIdx < 2 && !Found; InstanceIdx++)
        {
            RtlStringCchPrintfW(PathBuf, sizeof(PathBuf) / sizeof(WCHAR),
                L"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter\\%lu\\DisplayController\\%lu",
                AdapterIdx, InstanceIdx);

            RtlInitUnicodeString(&KeyName, PathBuf);
            InitializeObjectAttributes(&ObjectAttributes, &KeyName,
                                       OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                                       NULL, NULL);
            Status = ZwOpenKey(&hDisplay, KEY_READ, &ObjectAttributes);
            if (NT_SUCCESS(Status))
            {
                DPRINT1("[sysfb] Found DisplayController at adapter %lu instance %lu\n",
                        AdapterIdx, InstanceIdx);
                Found = TRUE;
            }
        }
    }

    if (!Found)
    {
        DPRINT1("[sysfb] DisplayController not found in ARC config tree\n");
        return STATUS_NOT_FOUND;
    }

    /* Read "Configuration Data" value */
    RtlInitUnicodeString(&ValueName, L"Configuration Data");
    Status = ZwQueryValueKey(hDisplay, &ValueName, KeyValuePartialInformation,
                             NULL, 0, &ResultLength);
    if (Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW)
    {
        DPRINT1("[sysfb] No Configuration Data value: 0x%08lx\n", Status);
        ZwClose(hDisplay);
        return STATUS_NOT_FOUND;
    }

    BufferSize = ResultLength;
    Buffer = (UCHAR *)ExAllocatePoolWithTag(PagedPool, BufferSize, 'diVA');
    if (!Buffer)
    {
        ZwClose(hDisplay);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(hDisplay, &ValueName, KeyValuePartialInformation,
                             Buffer, BufferSize, &ResultLength);
    ZwClose(hDisplay);

    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Failed to read Configuration Data: 0x%08lx\n", Status);
        ExFreePoolWithTag(Buffer, 'diVA');
        return Status;
    }

    /* Parse the CM_FULL_RESOURCE_DESCRIPTOR */
    ValueInfo = (PKEY_VALUE_PARTIAL_INFORMATION)Buffer;

    if (ValueInfo->DataLength < sizeof(CM_FULL_RESOURCE_DESCRIPTOR))
    {
        DPRINT1("[sysfb] Configuration Data too small: %lu bytes\n", ValueInfo->DataLength);
        ExFreePoolWithTag(Buffer, 'diVA');
        return STATUS_INVALID_PARAMETER;
    }

    FullResDesc = (PCM_FULL_RESOURCE_DESCRIPTOR)ValueInfo->Data;
    PartialList = &FullResDesc->PartialResourceList;

    DPRINT1("[sysfb] Configuration Data: %lu partial descriptors\n", PartialList->Count);

    /* Walk partial descriptors looking for Memory + DeviceSpecific */
    for (i = 0; i < PartialList->Count; i++)
    {
        PartialDesc = &PartialList->PartialDescriptors[i];

        if (PartialDesc->Type == CmResourceTypeMemory)
        {
            DevExt->FrameBufferPhysical.QuadPart = PartialDesc->u.Memory.Start.QuadPart;
            DevExt->FrameBufferSize = PartialDesc->u.Memory.Length;
            DPRINT1("[sysfb] Memory resource: phys=0x%I64x, size=0x%lx\n",
                    DevExt->FrameBufferPhysical.QuadPart, DevExt->FrameBufferSize);
        }
        else if (PartialDesc->Type == CmResourceTypeDeviceSpecific)
        {
            /* The CM_FRAMEBUF_DEVICE_DATA follows immediately after this descriptor */
            FramebufData = (PCM_FRAMEBUF_DEVICE_DATA)(PartialDesc + 1);
            DevExt->ScreenWidth = FramebufData->ScreenWidth;
            DevExt->ScreenHeight = FramebufData->ScreenHeight;
            DevExt->PixelsPerScanLine = FramebufData->PixelsPerScanLine;
            DevExt->BitsPerPixel = FramebufData->BitsPerPixel;
            DevExt->RedMask = FramebufData->PixelMasks.RedMask;
            DevExt->GreenMask = FramebufData->PixelMasks.GreenMask;
            DevExt->BlueMask = FramebufData->PixelMasks.BlueMask;

            /* Calculate stride */
            DevExt->ScreenStride = DevExt->PixelsPerScanLine * (DevExt->BitsPerPixel / 8);

            DPRINT1("[sysfb] Display: %lux%lu, %lu BPP, stride=%lu, R=0x%lx G=0x%lx B=0x%lx\n",
                    DevExt->ScreenWidth, DevExt->ScreenHeight, DevExt->BitsPerPixel,
                    DevExt->ScreenStride, DevExt->RedMask, DevExt->GreenMask, DevExt->BlueMask);
        }
    }

    ExFreePoolWithTag(Buffer, 'diVA');

    /* Validate we got both memory and display info */
    if (DevExt->FrameBufferPhysical.QuadPart == 0 || DevExt->ScreenWidth == 0)
    {
        DPRINT1("[sysfb] Incomplete framebuffer info from ARC tree\n");
        return STATUS_NOT_FOUND;
    }

    /* If FrameBufferSize wasn't set by Memory resource, calculate it */
    if (DevExt->FrameBufferSize == 0)
    {
        DevExt->FrameBufferSize = DevExt->ScreenStride * DevExt->ScreenHeight;
    }

    return STATUS_SUCCESS;
}

/*
 * Write HARDWARE\DEVICEMAP\VIDEO registry entries.
 * This is what EngpUpdateGraphicsDeviceList reads to discover video devices.
 */
static NTSTATUS
SysFbWriteDeviceMap(void)
{
    NTSTATUS Status;
    HANDLE hDeviceMap = NULL, hVideo = NULL;

    /* Create HKLM\HARDWARE\DEVICEMAP\VIDEO */
    Status = RegCreateKey(NULL, L"\\Registry\\Machine\\HARDWARE\\DEVICEMAP", &hDeviceMap);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Cannot create DEVICEMAP key: 0x%08lx\n", Status);
        return Status;
    }

    Status = RegCreateKey(hDeviceMap, L"VIDEO", &hVideo);
    ZwClose(hDeviceMap);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Cannot create VIDEO key: 0x%08lx\n", Status);
        return Status;
    }

    /* \Device\Video0 → registry path for driver service Device0 key */
    Status = RegSetSz(hVideo, L"\\Device\\Video0",
                      L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\sysfb\\Device0");
    if (!NT_SUCCESS(Status))
        DPRINT1("[sysfb] Failed to write \\Device\\Video0 value: 0x%08lx\n", Status);

    /* MaxObjectNumber = 0 (one device) */
    Status = RegSetDword(hVideo, L"MaxObjectNumber", 0);
    if (!NT_SUCCESS(Status))
        DPRINT1("[sysfb] Failed to write MaxObjectNumber: 0x%08lx\n", Status);

    ZwClose(hVideo);

    DPRINT1("[sysfb] DEVICEMAP\\VIDEO written\n");
    return STATUS_SUCCESS;
}

/*
 * Write driver service config:
 *   HKLM\SYSTEM\CurrentControlSet\Services\sysfb\Device0\
 *     InstalledDisplayDrivers = REG_MULTI_SZ "framebuf\0"
 *     Device Description = REG_SZ "QEMU ARM64 Framebuffer"
 *     DefaultSettings.* = display settings
 */
static NTSTATUS
SysFbWriteServiceConfig(PSYSFB_DEVICE_EXTENSION DevExt)
{
    NTSTATUS Status;
    HANDLE hService = NULL, hDevice0 = NULL;
    /* "framebuf\0\0" — double-null terminated MULTI_SZ */
    static const WCHAR MultiSzFramebuf[] = L"framebuf\0";

    /* Open Services key */
    Status = RegCreateKey(NULL, L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\sysfb", &hService);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Cannot open/create sysfb service key: 0x%08lx\n", Status);
        return Status;
    }

    /* Create Device0 subkey */
    Status = RegCreateKey(hService, L"Device0", &hDevice0);
    ZwClose(hService);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("[sysfb] Cannot create Device0 key: 0x%08lx\n", Status);
        return Status;
    }

    /* InstalledDisplayDrivers = "framebuf" (MULTI_SZ) */
    Status = RegSetMultiSz(hDevice0, L"InstalledDisplayDrivers",
                           MultiSzFramebuf, sizeof(MultiSzFramebuf));

    /* Device Description */
    RegSetSz(hDevice0, L"Device Description", L"QEMU ARM64 Framebuffer");

    /* Default settings from discovered framebuffer */
    RegSetDword(hDevice0, L"DefaultSettings.BitsPerPel", DevExt->BitsPerPixel);
    RegSetDword(hDevice0, L"DefaultSettings.XResolution", DevExt->ScreenWidth);
    RegSetDword(hDevice0, L"DefaultSettings.YResolution", DevExt->ScreenHeight);
    RegSetDword(hDevice0, L"DefaultSettings.VRefresh", 60);
    RegSetDword(hDevice0, L"DefaultSettings.Flags", 0);
    RegSetDword(hDevice0, L"DefaultSettings.XPanning", 0);
    RegSetDword(hDevice0, L"DefaultSettings.YPanning", 0);

    ZwClose(hDevice0);

    DPRINT1("[sysfb] Service config written (framebuf, %lux%lu %lubpp)\n",
            DevExt->ScreenWidth, DevExt->ScreenHeight, DevExt->BitsPerPixel);
    return STATUS_SUCCESS;
}
