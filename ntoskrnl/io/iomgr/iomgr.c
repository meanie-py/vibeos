/*
 * COPYRIGHT:       See COPYING in the top level directory
 * PROJECT:         ReactOS Kernel
 * FILE:            ntoskrnl/io/iomgr/iomgr.c
 * PURPOSE:         I/O Manager Initialization and Misc Utility Functions
 *
 * PROGRAMMERS:     David Welch (welch@mcmail.com)
 */

/* INCLUDES ****************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Direct UART trace for ARM64 boot debugging — bypasses PCH limitation */
#ifdef _M_ARM64
extern volatile PVOID Arm64UartKernelBase;
#undef ARM64_UART_TRACE
#define ARM64_UART_TRACE(s) do { \
    volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase; \
    const char *_p = (s); \
    if (!_uart) break; \
    while (*_p) *_uart = (ULONG)*_p++; \
} while(0)
#endif

VOID
NTAPI
IopTimerDispatch(
    IN PKDPC Dpc,
    IN PVOID DeferredContext,
    IN PVOID SystemArgument1,
    IN PVOID SystemArgument2
);

BOOLEAN
NTAPI
WmiInitialize(
    VOID);

/* DATA ********************************************************************/

ULONG IopTraceLevel = 0;
BOOLEAN PnpSystemInit = FALSE;

POBJECT_TYPE IoDeviceObjectType = NULL;
POBJECT_TYPE IoFileObjectType = NULL;
extern POBJECT_TYPE IoControllerObjectType;
ULONG IopCaseInsensitive =
#if (NTDDI_VERSION >= NTDDI_WIN7) || (DLL_EXPORT_VERSION >= _WIN32_WINNT_WIN7)
    TRUE;
#else
    FALSE;
#endif
BOOLEAN IoCountOperations = TRUE;
ULONG IoReadOperationCount = 0;
LARGE_INTEGER IoReadTransferCount = {{0, 0}};
ULONG IoWriteOperationCount = 0;
LARGE_INTEGER IoWriteTransferCount = {{0, 0}};
ULONG IoOtherOperationCount = 0;
LARGE_INTEGER IoOtherTransferCount = {{0, 0}};
KSPIN_LOCK IoStatisticsLock = 0;
ULONG IopAutoReboot;
ULONG IopNumTriageDumpDataBlocks;
PVOID IopTriageDumpDataBlocks[64];

GENERIC_MAPPING IopFileMapping = {
    FILE_GENERIC_READ,
    FILE_GENERIC_WRITE,
    FILE_GENERIC_EXECUTE,
    FILE_ALL_ACCESS};

extern LIST_ENTRY ShutdownListHead;
extern LIST_ENTRY LastChanceShutdownListHead;
extern KSPIN_LOCK ShutdownListLock;
extern POBJECT_TYPE IoAdapterObjectType;
extern ERESOURCE IopDatabaseResource;
ERESOURCE IopSecurityResource;
extern ERESOURCE IopDriverLoadResource;
extern LIST_ENTRY IopDiskFileSystemQueueHead;
extern LIST_ENTRY IopCdRomFileSystemQueueHead;
extern LIST_ENTRY IopTapeFileSystemQueueHead;
extern LIST_ENTRY IopNetworkFileSystemQueueHead;
extern LIST_ENTRY DriverBootReinitListHead;
extern LIST_ENTRY DriverReinitListHead;
extern LIST_ENTRY IopFsNotifyChangeQueueHead;
extern LIST_ENTRY IopErrorLogListHead;
extern LIST_ENTRY IopTimerQueueHead;
extern KDPC IopTimerDpc;
extern KTIMER IopTimer;
extern KSPIN_LOCK IoStatisticsLock;
extern KSPIN_LOCK DriverReinitListLock;
extern KSPIN_LOCK DriverBootReinitListLock;
extern KSPIN_LOCK IopLogListLock;
extern KSPIN_LOCK IopTimerLock;

extern PDEVICE_OBJECT IopErrorLogObject;
extern BOOLEAN PnPBootDriversInitialized;

GENERAL_LOOKASIDE IoLargeIrpLookaside;
GENERAL_LOOKASIDE IoSmallIrpLookaside;
GENERAL_LOOKASIDE IopMdlLookasideList;
extern GENERAL_LOOKASIDE IoCompletionPacketLookaside;

PLOADER_PARAMETER_BLOCK IopLoaderBlock;

/* INIT FUNCTIONS ************************************************************/

CODE_SEG("INIT")
VOID
NTAPI
IopInitLookasideLists(VOID)
{
    ULONG LargeIrpSize, SmallIrpSize, MdlSize;
    LONG i;
    PKPRCB Prcb;
    PGENERAL_LOOKASIDE CurrentList = NULL;

    /* Calculate the sizes */
    LargeIrpSize = sizeof(IRP) + (8 * sizeof(IO_STACK_LOCATION));
    SmallIrpSize = sizeof(IRP) + sizeof(IO_STACK_LOCATION);
    MdlSize = sizeof(MDL) + (23 * sizeof(PFN_NUMBER));

    /* Initialize the Lookaside List for I\O Completion */
    ExInitializeSystemLookasideList(&IoCompletionPacketLookaside,
                                    NonPagedPool,
                                    sizeof(IOP_MINI_COMPLETION_PACKET),
                                    IOC_TAG1,
                                    32,
                                    &ExSystemLookasideListHead);

    /* Initialize the Lookaside List for Large IRPs */
    ExInitializeSystemLookasideList(&IoLargeIrpLookaside,
                                    NonPagedPool,
                                    LargeIrpSize,
                                    IO_LARGEIRP,
                                    64,
                                    &ExSystemLookasideListHead);


    /* Initialize the Lookaside List for Small IRPs */
    ExInitializeSystemLookasideList(&IoSmallIrpLookaside,
                                    NonPagedPool,
                                    SmallIrpSize,
                                    IO_SMALLIRP,
                                    32,
                                    &ExSystemLookasideListHead);

    /* Initialize the Lookaside List for MDLs */
    ExInitializeSystemLookasideList(&IopMdlLookasideList,
                                    NonPagedPool,
                                    MdlSize,
                                    TAG_MDL,
                                    128,
                                    &ExSystemLookasideListHead);

    /* Allocate the global lookaside list buffer */
    CurrentList = ExAllocatePoolWithTag(NonPagedPool,
                                        4 * KeNumberProcessors *
                                        sizeof(GENERAL_LOOKASIDE),
                                        TAG_IO);

    /* Loop all processors */
    for (i = 0; i < KeNumberProcessors; i++)
    {
        /* Get the PRCB for this CPU */
        Prcb = KiProcessorBlock[i];
        DPRINT("Setting up lookaside for CPU: %x, PRCB: %p\n", i, Prcb);

        /* Write IRP credit limit */
        Prcb->LookasideIrpFloat = 512 / KeNumberProcessors;

        /* Set the I/O Completion List */
        Prcb->PPLookasideList[LookasideCompletionList].L = &IoCompletionPacketLookaside;
        if (CurrentList)
        {
            /* Initialize the Lookaside List for mini-packets */
            ExInitializeSystemLookasideList(CurrentList,
                                            NonPagedPool,
                                            sizeof(IOP_MINI_COMPLETION_PACKET),
                                            IO_SMALLIRP_CPU,
                                            32,
                                            &ExSystemLookasideListHead);
            Prcb->PPLookasideList[LookasideCompletionList].P = CurrentList;
            CurrentList++;

        }
        else
        {
            Prcb->PPLookasideList[LookasideCompletionList].P = &IoCompletionPacketLookaside;
        }

        /* Set the Large IRP List */
        Prcb->PPLookasideList[LookasideLargeIrpList].L = &IoLargeIrpLookaside;
        if (CurrentList)
        {
            /* Initialize the Lookaside List for Large IRPs */
            ExInitializeSystemLookasideList(CurrentList,
                                            NonPagedPool,
                                            LargeIrpSize,
                                            IO_LARGEIRP_CPU,
                                            64,
                                            &ExSystemLookasideListHead);
            Prcb->PPLookasideList[LookasideLargeIrpList].P = CurrentList;
            CurrentList++;

        }
        else
        {
            Prcb->PPLookasideList[LookasideLargeIrpList].P = &IoLargeIrpLookaside;
        }

        /* Set the Small IRP List */
        Prcb->PPLookasideList[LookasideSmallIrpList].L = &IoSmallIrpLookaside;
        if (CurrentList)
        {
            /* Initialize the Lookaside List for Small IRPs */
            ExInitializeSystemLookasideList(CurrentList,
                                            NonPagedPool,
                                            SmallIrpSize,
                                            IO_SMALLIRP_CPU,
                                            32,
                                            &ExSystemLookasideListHead);
            Prcb->PPLookasideList[LookasideSmallIrpList].P = CurrentList;
            CurrentList++;

        }
        else
        {
            Prcb->PPLookasideList[LookasideSmallIrpList].P = &IoSmallIrpLookaside;
        }

        /* Set the MDL Completion List */
        Prcb->PPLookasideList[LookasideMdlList].L = &IopMdlLookasideList;
        if (CurrentList)
        {
            /* Initialize the Lookaside List for MDLs */
            ExInitializeSystemLookasideList(CurrentList,
                                            NonPagedPool,
                                            SmallIrpSize,
                                            TAG_MDL,
                                            128,
                                            &ExSystemLookasideListHead);

            Prcb->PPLookasideList[LookasideMdlList].P = CurrentList;
            CurrentList++;

        }
        else
        {
            Prcb->PPLookasideList[LookasideMdlList].P = &IopMdlLookasideList;
        }
    }
}

CODE_SEG("INIT")
BOOLEAN
NTAPI
IopCreateObjectTypes(VOID)
{
    OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
    UNICODE_STRING Name;

    /* Initialize default settings */
    RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
    ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
    ObjectTypeInitializer.PoolType = NonPagedPool;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = TRUE;
    ObjectTypeInitializer.GenericMapping = IopFileMapping;

    /* Do the Adapter Type */
    RtlInitUnicodeString(&Name, L"Adapter");
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoAdapterObjectType))) return FALSE;

    /* Do the Controller Type */
    RtlInitUnicodeString(&Name, L"Controller");
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(CONTROLLER_OBJECT);
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoControllerObjectType))) return FALSE;

    /* Do the Device Type */
    RtlInitUnicodeString(&Name, L"Device");
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(DEVICE_OBJECT);
    ObjectTypeInitializer.DeleteProcedure = IopDeleteDevice;
    ObjectTypeInitializer.ParseProcedure = IopParseDevice;
    ObjectTypeInitializer.SecurityProcedure = IopGetSetSecurityObject;
    ObjectTypeInitializer.CaseInsensitive = TRUE;
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoDeviceObjectType))) return FALSE;

    /* Initialize the Driver object type */
    RtlInitUnicodeString(&Name, L"Driver");
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(DRIVER_OBJECT);
    ObjectTypeInitializer.DeleteProcedure = IopDeleteDriver;
    ObjectTypeInitializer.ParseProcedure = NULL;
    ObjectTypeInitializer.SecurityProcedure = NULL;
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoDriverObjectType))) return FALSE;

    /* Initialize the I/O Completion object type */
    RtlInitUnicodeString(&Name, L"IoCompletion");
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KQUEUE);
    ObjectTypeInitializer.ValidAccessMask = IO_COMPLETION_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes |= OBJ_PERMANENT;
    ObjectTypeInitializer.GenericMapping = IopCompletionMapping;
    ObjectTypeInitializer.DeleteProcedure = IopDeleteIoCompletion;
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoCompletionType))) return FALSE;

    /* Initialize the File object type  */
    RtlInitUnicodeString(&Name, L"File");
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(FILE_OBJECT);
    ObjectTypeInitializer.InvalidAttributes |= OBJ_EXCLUSIVE;
    ObjectTypeInitializer.MaintainHandleCount = TRUE;
    ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
    ObjectTypeInitializer.GenericMapping = IopFileMapping;
    ObjectTypeInitializer.CloseProcedure = IopCloseFile;
    ObjectTypeInitializer.DeleteProcedure = IopDeleteFile;
    ObjectTypeInitializer.SecurityProcedure = IopGetSetSecurityObject;
    ObjectTypeInitializer.QueryNameProcedure = IopQueryName;
    ObjectTypeInitializer.ParseProcedure = IopParseFile;
    ObjectTypeInitializer.UseDefaultObject = FALSE;
    if (!NT_SUCCESS(ObCreateObjectType(&Name,
                                       &ObjectTypeInitializer,
                                       NULL,
                                       &IoFileObjectType))) return FALSE;

    /* Success */
    return TRUE;
}

CODE_SEG("INIT")
BOOLEAN
NTAPI
IopCreateRootDirectories(VOID)
{
    OBJECT_ATTRIBUTES ObjectAttributes;
    UNICODE_STRING DirName;
    HANDLE Handle;
    NTSTATUS Status;

    /* Create the '\Driver' object directory */
    RtlInitUnicodeString(&DirName, L"\\Driver");
    InitializeObjectAttributes(&ObjectAttributes,
                               &DirName,
                               OBJ_PERMANENT,
                               NULL,
                               NULL);
    Status = NtCreateDirectoryObject(&Handle,
                                     DIRECTORY_ALL_ACCESS,
                                     &ObjectAttributes);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Failed to create \\Driver directory: 0x%lx\n", Status);
        return FALSE;
    }
    NtClose(Handle);

    /* Create the '\FileSystem' object directory */
    RtlInitUnicodeString(&DirName, L"\\FileSystem");
    InitializeObjectAttributes(&ObjectAttributes,
                               &DirName,
                               OBJ_PERMANENT,
                               NULL,
                               NULL);
    Status = NtCreateDirectoryObject(&Handle,
                                     DIRECTORY_ALL_ACCESS,
                                     &ObjectAttributes);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Failed to create \\FileSystem directory: 0x%lx\n", Status);
        return FALSE;
    }
    NtClose(Handle);

    /* Create the '\FileSystem' object directory */
    RtlInitUnicodeString(&DirName, L"\\FileSystem\\Filters");
    InitializeObjectAttributes(&ObjectAttributes,
                               &DirName,
                               OBJ_PERMANENT,
                               NULL,
                               NULL);
    Status = NtCreateDirectoryObject(&Handle,
                                     DIRECTORY_ALL_ACCESS,
                                     &ObjectAttributes);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Failed to create \\FileSystem\\Filters directory: 0x%lx\n", Status);
        return FALSE;
    }
    NtClose(Handle);

    /* Return success */
    return TRUE;
}

CODE_SEG("INIT")
BOOLEAN
NTAPI
IopMarkBootPartition(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    OBJECT_ATTRIBUTES ObjectAttributes;
    STRING DeviceString;
    CHAR Buffer[256];
    UNICODE_STRING DeviceName;
    NTSTATUS Status;
    HANDLE FileHandle;
    IO_STATUS_BLOCK IoStatusBlock;
    PFILE_OBJECT FileObject;

    /* Build the ARC device name */
    sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
    RtlInitAnsiString(&DeviceString, Buffer);
    Status = RtlAnsiStringToUnicodeString(&DeviceName, &DeviceString, TRUE);
    if (!NT_SUCCESS(Status)) return FALSE;

    /* Open it */
    InitializeObjectAttributes(&ObjectAttributes,
                               &DeviceName,
                               OBJ_CASE_INSENSITIVE,
                               NULL,
                               NULL);
    Status = ZwOpenFile(&FileHandle,
                        FILE_READ_ATTRIBUTES,
                        &ObjectAttributes,
                        &IoStatusBlock,
                        0,
                        FILE_NON_DIRECTORY_FILE);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_ARM64
        /* On ARM64 UEFI boot without disk drivers, boot device may not exist.
         * Return FALSE instead of bugchecking to allow boot to continue. */
        RtlFreeUnicodeString(&DeviceName);
        return FALSE;
#else
        /* MININT/WinPE boots may lack a real boot device (e.g. LiveCD when the
         * ATAPI driver cannot probe the CD-ROM).  Return FALSE to let the
         * caller decide whether to continue. */
        if (InitIsWinPEMode)
        {
            RtlFreeUnicodeString(&DeviceName);
            return FALSE;
        }
        /* Fail */
        KeBugCheckEx(INACCESSIBLE_BOOT_DEVICE,
                     (ULONG_PTR)&DeviceName,
                     Status,
                     0,
                     0);
#endif
    }

    /* Get the DO */
    Status = ObReferenceObjectByHandle(FileHandle,
                                       0,
                                       IoFileObjectType,
                                       KernelMode,
                                       (PVOID *)&FileObject,
                                       NULL);
    if (!NT_SUCCESS(Status))
    {
        /* Fail */
        RtlFreeUnicodeString(&DeviceName);
        return FALSE;
    }

    /* Mark it as the boot partition */
    FileObject->DeviceObject->Flags |= DO_SYSTEM_BOOT_PARTITION;

    /* Save a copy of the DO for the I/O Error Logger */
    ObReferenceObject(FileObject->DeviceObject);
    IopErrorLogObject = FileObject->DeviceObject;

    /* Cleanup and return success */
    RtlFreeUnicodeString(&DeviceName);
    NtClose(FileHandle);
    ObDereferenceObject(FileObject);
    return TRUE;
}

CODE_SEG("INIT")
BOOLEAN
NTAPI
IoInitSystem(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    LARGE_INTEGER ExpireTime;
    NTSTATUS Status;
    CHAR Buffer[256];
    ANSI_STRING NtBootPath, RootString;

    /* Initialize empty NT Boot Path */
    RtlInitEmptyAnsiString(&NtBootPath, Buffer, sizeof(Buffer));

    /* Initialize the lookaside lists */
    ARM64_UART_TRACE("[IO] IopInitLookasideLists\r\n");
    IopInitLookasideLists();

    /* Initialize all locks and lists */
    ExInitializeResourceLite(&IopDatabaseResource);
    ExInitializeResourceLite(&IopSecurityResource);
    ExInitializeResourceLite(&IopDriverLoadResource);
    InitializeListHead(&IopDiskFileSystemQueueHead);
    InitializeListHead(&IopCdRomFileSystemQueueHead);
    InitializeListHead(&IopTapeFileSystemQueueHead);
    InitializeListHead(&IopNetworkFileSystemQueueHead);
    InitializeListHead(&DriverBootReinitListHead);
    InitializeListHead(&DriverReinitListHead);
    InitializeListHead(&ShutdownListHead);
    InitializeListHead(&LastChanceShutdownListHead);
    InitializeListHead(&IopFsNotifyChangeQueueHead);
    InitializeListHead(&IopErrorLogListHead);
    KeInitializeSpinLock(&IoStatisticsLock);
    KeInitializeSpinLock(&DriverReinitListLock);
    KeInitializeSpinLock(&DriverBootReinitListLock);
    KeInitializeSpinLock(&ShutdownListLock);
    KeInitializeSpinLock(&IopLogListLock);

    /* Initialize PnP notifications */
    PiInitializeNotifications();

    /* Initialize the reserve IRP */
    if (!IopInitializeReserveIrp(&IopReserveIrpAllocator))
    {
        DPRINT1("IopInitializeReserveIrp failed!\n");
        return FALSE;
    }

    /* Initialize Timer List Lock */
    KeInitializeSpinLock(&IopTimerLock);

    /* Initialize Timer List */
    InitializeListHead(&IopTimerQueueHead);

    /* Initialize the DPC/Timer which will call the other Timer Routines */
    ExpireTime.QuadPart = -10000000;
    KeInitializeDpc(&IopTimerDpc, IopTimerDispatch, NULL);
    KeInitializeTimerEx(&IopTimer, SynchronizationTimer);
    KeSetTimerEx(&IopTimer, ExpireTime, 1000, &IopTimerDpc);

    /* Create Object Types */
    ARM64_UART_TRACE("[IO] IopCreateObjectTypes\r\n");
    if (!IopCreateObjectTypes())
    {
        DPRINT1("IopCreateObjectTypes failed!\n");
        return FALSE;
    }

    /* Create Object Directories */
    ARM64_UART_TRACE("[IO] IopCreateRootDirectories\r\n");
    if (!IopCreateRootDirectories())
    {
        DPRINT1("IopCreateRootDirectories failed!\n");
        return FALSE;
    }

    /* Initialize PnP manager */
    ARM64_UART_TRACE("[IO] IopInitializePlugPlayServices\r\n");
    IopInitializePlugPlayServices();

    /* Initialize SHIM engine */
    ApphelpCacheInitialize();

    /* Initialize WMI */
    WmiInitialize();

    /* Initialize HAL Root Bus Driver */
    ARM64_UART_TRACE("[IO] HalInitPnpDriver\r\n");
    HalInitPnpDriver();

    /* Reenumerate what HAL has added (synchronously)
     * This function call should eventually become a 2nd stage of the PnP initialization */
    ARM64_UART_TRACE("[IO] PiQueueDeviceAction\r\n");
    PiQueueDeviceAction(IopRootDeviceNode->PhysicalDeviceObject,
                        PiActionEnumRootDevices,
                        NULL,
                        NULL);

    /* Make loader block available for the whole kernel */
    IopLoaderBlock = LoaderBlock;

    /* Load boot start drivers */
    ARM64_UART_TRACE("[IO] IopInitializeBootDrivers\r\n");
    IopInitializeBootDrivers();

    /* Call back drivers that asked for */
    ARM64_UART_TRACE("[IO] IopReinitializeBootDrivers\r\n");
    IopReinitializeBootDrivers();

    /* Check if this was a ramdisk boot */
    ARM64_UART_TRACE("[IO] Checking ArcBootDeviceName for ramdisk...\r\n");
    if (!_strnicmp(LoaderBlock->ArcBootDeviceName, "ramdisk(0)", 10))
    {
        /* Initialize the ramdisk driver */
        ARM64_UART_TRACE("[IO] IopStartRamdisk\r\n");
        IopStartRamdisk(LoaderBlock);
        ARM64_UART_TRACE("[IO] IopStartRamdisk done\r\n");

#ifdef _M_ARM64
        /* Test: try to open a file on the ramdisk to verify filesystem access */
        {
            OBJECT_ATTRIBUTES TestOa;
            IO_STATUS_BLOCK TestIosb;
            HANDLE TestHandle;
            UNICODE_STRING TestPath;
            NTSTATUS TestStatus;

            ARM64_UART_TRACE("[IO] Testing ramdisk filesystem access...\r\n");
            RtlInitUnicodeString(&TestPath, L"\\ArcName\\ramdisk(0)\\reactos\\system32\\ntdll.dll");
            InitializeObjectAttributes(&TestOa, &TestPath, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
            ARM64_UART_TRACE("[IO] ZwOpenFile(ntdll.dll) start\r\n");
            TestStatus = ZwOpenFile(&TestHandle, FILE_READ_ACCESS, &TestOa, &TestIosb, FILE_SHARE_READ, 0);
            if (NT_SUCCESS(TestStatus))
            {
                ARM64_UART_TRACE("[IO] ZwOpenFile(ntdll.dll) SUCCESS!\r\n");
                ZwClose(TestHandle);
            }
            else
            {
                volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
                char _buf[64];
                const char *_p;
                int _len = sprintf(_buf, "[IO] ZwOpenFile(ntdll.dll) failed: 0x%lx\r\n", (unsigned long)TestStatus);
                _p = _buf;
                while (*_p) { *_u = (ULONG)*_p++; }
            }
        }
#endif
    }
    else
    {
        ARM64_UART_TRACE("[IO] Not a ramdisk boot\r\n");
    }

    /* No one should need loader block any longer */
    IopLoaderBlock = NULL;

    /* Create ARC names for boot devices */
    ARM64_UART_TRACE("[IO] IopCreateArcNames\r\n");
    Status = IopCreateArcNames(LoaderBlock);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("IopCreateArcNames failed: %lx\n", Status);
#if !defined(_M_ARM64)
        /* MININT/WinPE boots may lack a real boot disk (e.g. LiveCD when the
         * ATAPI driver cannot probe the CD-ROM device).  Allow boot to continue
         * so that ramdisk or other drivers can still provide the volume. */
        if (!InitIsWinPEMode)
            return FALSE;
#endif
    }

    /* Mark the system boot partition */
    ARM64_UART_TRACE("[IO] IopMarkBootPartition\r\n");
    if (!IopMarkBootPartition(LoaderBlock))
    {
        ARM64_UART_TRACE("[IO] IopMarkBootPartition failed (continuing on ARM64/MININT)\r\n");
#if !defined(_M_ARM64)
        if (!InitIsWinPEMode)
            return FALSE;
#endif
    }
    else
    {
        ARM64_UART_TRACE("[IO] IopMarkBootPartition done\r\n");
    }

    /* The disk subsystem is initialized here and the SystemRoot is set too.
     * We can finally load other drivers from the boot volume. */
    PnPBootDriversInitialized = TRUE;

    /* Load system start drivers */
    ARM64_UART_TRACE("[IO] IopInitializeSystemDrivers\r\n");
    IopInitializeSystemDrivers();
    PnpSystemInit = TRUE;
    ARM64_UART_TRACE("[IO] IopInitializeSystemDrivers done\r\n");

    /* Reinitialize drivers that requested it */
    IopReinitializeDrivers();

    /* Convert SystemRoot from ARC to NT path */
    ARM64_UART_TRACE("[IO] IopReassignSystemRoot\r\n");
    Status = IopReassignSystemRoot(LoaderBlock, &NtBootPath);
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_ARM64
        ARM64_UART_TRACE("[IO] IopReassignSystemRoot failed (continuing on ARM64)\r\n");
        return FALSE;
#else
        ARM64_UART_TRACE("[IO] IopReassignSystemRoot failed\r\n");
        return FALSE;
#endif
    }
    ARM64_UART_TRACE("[IO] IopReassignSystemRoot done\r\n");

    /* Set the ANSI_STRING for the root path */
    RootString.MaximumLength = NtSystemRoot.MaximumLength / sizeof(WCHAR);
    RootString.Length = 0;
    RootString.Buffer = ExAllocatePoolWithTag(PagedPool,
                                              RootString.MaximumLength,
                                              TAG_IO);

    /* Convert the path into the ANSI_STRING */
    Status = RtlUnicodeStringToAnsiString(&RootString, &NtSystemRoot, FALSE);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("RtlUnicodeStringToAnsiString failed: %lx\n", Status);
        return FALSE;
    }

    /* Assign drive letters */
    ARM64_UART_TRACE("[IO] IoAssignDriveLetters\r\n");
    IoAssignDriveLetters(LoaderBlock,
                         &NtBootPath,
                         (PUCHAR)RootString.Buffer,
                         &RootString);
    ARM64_UART_TRACE("[IO] IoAssignDriveLetters done\r\n");

    /* Update system root */
    Status = RtlAnsiStringToUnicodeString(&NtSystemRoot, &RootString, FALSE);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("RtlAnsiStringToUnicodeString failed: %lx\n", Status);
        return FALSE;
    }

    /* Load the System DLL and its entrypoints */
    ARM64_UART_TRACE("[IO] PsLocateSystemDll\r\n");
    Status = PsLocateSystemDll();
    if (!NT_SUCCESS(Status))
    {
#ifdef _M_ARM64
        /* On ARM64, if ntdll.dll can't be loaded (no filesystem),
         * continue without it — Phase 1 will enter idle instead of SMSS. */
        ARM64_UART_TRACE("[IO] PsLocateSystemDll failed (continuing on ARM64)\r\n");
        DPRINT1("PsLocateSystemDll failed: %lx (ARM64, continuing)\n", Status);
#else
        DPRINT1("PsLocateSystemDll failed: %lx\n", Status);
        return FALSE;
#endif
    }

    /* Return success */
    ARM64_UART_TRACE("[IO] IoInitSystem returning TRUE\r\n");
    return TRUE;
}

BOOLEAN
NTAPI
IoInitializeCrashDump(IN HANDLE PageFileHandle)
{
    UNIMPLEMENTED;
    return FALSE;
}

/* EOF */
