/*
 * mm_sysldr_shims.c — C implementations for sysldr functions that are stubs
 * in the Rust translation (ntoskrnl-rust/src/ARM3/sysldr.rs).
 *
 * MmLoadSystemImage and its internal helpers are complex (~1000 lines total)
 * and depend on many kernel subsystems. Rather than translating them to Rust
 * immediately, we keep the proven C implementations and let the Rust code
 * handle the simpler sysldr functions (module list management, exports, etc.).
 *
 * Copied from ntoskrnl/mm/ARM3/sysldr.c with minimal modifications.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#define MODULE_INVOLVED_IN_ARM3
#include <mm/ARM3/miarm.h>

/* Globals provided by Rust sysldr.rs (#[no_mangle]) */
extern LIST_ENTRY PsLoadedModuleList;
extern LIST_ENTRY MmLoadedUserImageList;
extern KSPIN_LOCK PsLoadedModuleSpinLock;
extern ERESOURCE PsLoadedModuleResource;
extern ULONG_PTR PsNtosImageBase;
extern KMUTANT MmSystemLoadLock;
extern BOOLEAN MmEnforceWriteProtection;

/* Functions provided by Rust sysldr.rs (#[no_mangle]) */
NTSTATUS NTAPI MmCallDllInitialize(PLDR_DATA_TABLE_ENTRY, PLIST_ENTRY);
NTSTATUS NTAPI MmUnloadSystemImage(PVOID);
NTSTATUS NTAPI MmCheckSystemImage(HANDLE);

/* Forward declarations for static helpers in this file */
/* Not static — also called from driver.c LdrProcessDriverModule */
NTSTATUS NTAPI MiResolveImageReferences(PVOID, PUNICODE_STRING, PUNICODE_STRING, PCHAR *, PWCHAR *, PLOAD_IMPORTS *);
/* MiSnapThunk + NameToOrdinal: translated to Rust (sysldr.rs); externs supplied
 * under the same names by the kernel-rust archive. */
NTSTATUS NTAPI MiSnapThunk(PVOID, PVOID, PIMAGE_THUNK_DATA, PIMAGE_THUNK_DATA, PIMAGE_EXPORT_DIRECTORY, ULONG, BOOLEAN, PCHAR *);

/* ========== Small static stubs ========== */

static PVOID
NTAPI
MiCacheImageSymbols_shim(IN PVOID BaseAddress)
{
    ULONG DebugSize;
    PVOID DebugDirectory = NULL;
    PAGED_CODE();
    DebugDirectory = RtlImageDirectoryEntryToData(BaseAddress,
                                                   TRUE,
                                                   IMAGE_DIRECTORY_ENTRY_DEBUG,
                                                   &DebugSize);
    return DebugDirectory;
}

static VOID
NTAPI
MiProcessLoaderEntry_shim(IN PLDR_DATA_TABLE_ENTRY LdrEntry,
                          IN BOOLEAN Insert)
{
    KIRQL OldIrql;
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(&PsLoadedModuleResource, TRUE);
    OldIrql = KeAcquireSpinLockRaiseToSynch(&PsLoadedModuleSpinLock);
    if (Insert)
        InsertTailList(&PsLoadedModuleList, &LdrEntry->InLoadOrderLinks);
    else
        RemoveEntryList(&LdrEntry->InLoadOrderLinks);
    KeReleaseSpinLock(&PsLoadedModuleSpinLock, OldIrql);
    ExReleaseResourceLite(&PsLoadedModuleResource);
    KeLeaveCriticalRegion();
}

static LOGICAL
MiUseLargeDriverPage_shim(ULONG NumberOfPtes,
                          PVOID *ImageBaseAddress,
                          PUNICODE_STRING BaseImageName,
                          BOOLEAN BootDriver)
{
    /* Not implemented — large page optimization */
    return FALSE;
}

/* MiWriteProtectSystemImage_shim deleted — Rust impl
 * (sysldr.rs::MiWriteProtectSystemImage) is exported and called directly. */
VOID NTAPI MiWriteProtectSystemImage(PVOID ImageBase);

static PVOID
LdrpFetchAddressOfSecurityCookie_shim(PVOID BaseAddress, ULONG SizeOfImage)
{
    return NULL;
}

static PVOID
LdrpInitSecurityCookie_shim(PLDR_DATA_TABLE_ENTRY LdrEntry)
{
    return NULL;
}

static VOID
MiEnablePagingOfDriver_shim(PLDR_DATA_TABLE_ENTRY LdrEntry)
{
    /* Not implemented — driver paging */
}

static NTSTATUS
MiDereferenceImports_shim(PLOAD_IMPORTS ImportList)
{
    /* LoadedImports field handling elided */
    return STATUS_SUCCESS;
}

/* MiLoadImageSection: translated to Rust (sysldr.rs); exported under the
 * unsuffixed name. NameToOrdinal + MiSnapThunk also live in Rust. */
NTSTATUS NTAPI MiLoadImageSection(PSECTION *SectionPtr, PVOID *ImageBase,
                                  PUNICODE_STRING FileName, BOOLEAN SessionLoad,
                                  PLDR_DATA_TABLE_ENTRY LdrEntry);

/* MiResolveImageReferences: translated to Rust (sysldr.rs); externs at top. */

/* ========== MmLoadSystemImage ========== */

NTSTATUS
NTAPI
MmLoadSystemImage(IN PUNICODE_STRING FileName,
                  IN PUNICODE_STRING NamePrefix OPTIONAL,
                  IN PUNICODE_STRING LoadedName OPTIONAL,
                  IN ULONG Flags,
                  OUT PVOID *ModuleObject,
                  OUT PVOID *ImageBaseAddress)
{
    PVOID ModuleLoadBase = NULL;
    NTSTATUS Status;
    HANDLE FileHandle = NULL;
    OBJECT_ATTRIBUTES ObjectAttributes;
    IO_STATUS_BLOCK IoStatusBlock;
    PIMAGE_NT_HEADERS NtHeader;
    UNICODE_STRING BaseName, BaseDirectory, PrefixName;
    PLDR_DATA_TABLE_ENTRY LdrEntry = NULL;
    ULONG EntrySize, DriverSize;
    PLOAD_IMPORTS LoadedImports = MM_SYSLDR_NO_IMPORTS;
    PCHAR MissingApiName, Buffer;
    PWCHAR MissingDriverName, PrefixedBuffer = NULL;
    HANDLE SectionHandle;
    ACCESS_MASK DesiredAccess;
    PSECTION Section = NULL;
    BOOLEAN LockOwned = FALSE;
    PLIST_ENTRY NextEntry;
    IMAGE_INFO ImageInfo;

    PAGED_CODE();

    /* Detect session-load */
    if (Flags)
    {
        ASSERT(NamePrefix == NULL);
        ASSERT(LoadedName == NULL);
        if (!PsGetCurrentProcess()->ProcessInSession) return STATUS_NO_MEMORY;
    }

    /* Allocate a buffer we'll use for names */
    Buffer = ExAllocatePoolWithTag(NonPagedPool,
                                   MAXIMUM_FILENAME_LENGTH,
                                   TAG_LDR_WSTR);
    if (!Buffer) return STATUS_INSUFFICIENT_RESOURCES;

    /* Check for a separator */
    if (FileName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
    {
        PWCHAR p;
        ULONG BaseLength;

        p = &FileName->Buffer[FileName->Length / sizeof(WCHAR)];
        while (*(p - 1) != OBJ_NAME_PATH_SEPARATOR) p--;

        BaseLength = (ULONG)(&FileName->Buffer[FileName->Length / sizeof(WCHAR)] - p);
        BaseLength *= sizeof(WCHAR);

        BaseName.Length = (USHORT)BaseLength;
        BaseName.Buffer = p;
    }
    else
    {
        BaseName.Length = FileName->Length;
        BaseName.Buffer = FileName->Buffer;
    }

    BaseName.MaximumLength = BaseName.Length;

    BaseDirectory = *FileName;
    BaseDirectory.Length -= BaseName.Length;
    BaseDirectory.MaximumLength = BaseDirectory.Length;

    PrefixName = *FileName;

    /* Check if we have a prefix */
    if (NamePrefix)
    {
        Status = RtlUShortAdd(BaseDirectory.Length,
                              NamePrefix->Length,
                              &PrefixName.MaximumLength);
        if (!NT_SUCCESS(Status))
        {
            Status = STATUS_INVALID_PARAMETER;
            goto Quickie;
        }

        Status = RtlUShortAdd(PrefixName.MaximumLength,
                              BaseName.Length,
                              &PrefixName.MaximumLength);
        if (!NT_SUCCESS(Status))
        {
            Status = STATUS_INVALID_PARAMETER;
            goto Quickie;
        }

        PrefixedBuffer = ExAllocatePoolWithTag(PagedPool,
                                               PrefixName.MaximumLength,
                                               TAG_LDR_WSTR);
        if (!PrefixedBuffer)
        {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            goto Quickie;
        }

        PrefixName.Buffer = PrefixedBuffer;
        PrefixName.Length = 0;

        RtlAppendUnicodeStringToString(&PrefixName, &BaseDirectory);
        RtlAppendUnicodeStringToString(&PrefixName, NamePrefix);
        RtlAppendUnicodeStringToString(&PrefixName, &BaseName);

        BaseName.Buffer = &(PrefixName.Buffer[BaseDirectory.Length / sizeof(WCHAR)]);
        BaseName.Length += NamePrefix->Length;
        BaseName.MaximumLength = (PrefixName.MaximumLength - BaseDirectory.Length);
    }

    /* Check if we already have a name, use it instead */
    if (LoadedName) BaseName = *LoadedName;

    /* Check for loader snap debugging */
    if (NtGlobalFlag & FLG_SHOW_LDR_SNAPS)
    {
        DPRINT1("MM:SYSLDR Loading %wZ (%wZ) %s\n",
                &PrefixName, &BaseName, Flags ? "in session space" : "");
    }

    /* Acquire the load lock */
LoaderScan:
    ASSERT(LockOwned == FALSE);
    LockOwned = TRUE;
    KeEnterCriticalRegion();
    KeWaitForSingleObject(&MmSystemLoadLock,
                          WrVirtualMemory,
                          KernelMode,
                          FALSE,
                          NULL);

    /* Scan the module list */
    NextEntry = PsLoadedModuleList.Flink;
    while (NextEntry != &PsLoadedModuleList)
    {
        LdrEntry = CONTAINING_RECORD(NextEntry,
                                     LDR_DATA_TABLE_ENTRY,
                                     InLoadOrderLinks);
        if (RtlEqualUnicodeString(&PrefixName, &LdrEntry->FullDllName, TRUE))
        {
            break;
        }
        NextEntry = NextEntry->Flink;
    }
    /* Check if we found the image */
    if (NextEntry != &PsLoadedModuleList)
    {
        if (Section)
        {
            ObDereferenceObject(Section);
            Section = NULL;
        }

        if (!Flags)
        {
            *ModuleObject = LdrEntry;
            *ImageBaseAddress = LdrEntry->DllBase;
            Status = STATUS_IMAGE_ALREADY_LOADED;
        }
        else
        {
                Status = STATUS_NOT_IMPLEMENTED;
        }

        goto Quickie;
    }
    else if (!Section)
    {
        KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
        KeLeaveCriticalRegion();
        LockOwned = FALSE;

        LdrEntry = NULL;

        InitializeObjectAttributes(&ObjectAttributes,
                                   FileName,
                                   OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                                   NULL,
                                   NULL);


        Status = ZwOpenFile(&FileHandle,
                            FILE_EXECUTE,
                            &ObjectAttributes,
                            &IoStatusBlock,
                            FILE_SHARE_READ | FILE_SHARE_DELETE,
                            0);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("MmLoadSystemImage: ZwOpenFile(%wZ) failed: 0x%x\n", FileName, Status);
            goto Quickie;
        }
        /* Validate it */
        Status = MmCheckSystemImage(FileHandle);
        if ((Status == STATUS_IMAGE_CHECKSUM_MISMATCH) ||
            (Status == STATUS_IMAGE_MP_UP_MISMATCH) ||
            (Status == STATUS_INVALID_IMAGE_PROTECT))
        {
            DPRINT1("MmLoadSystemImage: MmCheckSystemImage(%wZ) failed: 0x%x\n", FileName, Status);
            goto Quickie;
        }

        /* Check if this is a session-load */
        if (Flags)
        {
            DesiredAccess = SECTION_MAP_READ | SECTION_MAP_EXECUTE;
        }
        else
        {
            DesiredAccess = SECTION_ALL_ACCESS;
        }

        InitializeObjectAttributes(&ObjectAttributes,
                                   NULL,
                                   OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                                   NULL,
                                   NULL);

        Status = ZwCreateSection(&SectionHandle,
                                 DesiredAccess,
                                 &ObjectAttributes,
                                 NULL,
                                 PAGE_EXECUTE,
                                 SEC_IMAGE,
                                 FileHandle);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("MmLoadSystemImage: ZwCreateSection(%wZ) failed: 0x%x\n", FileName, Status);
            goto Quickie;
        }
        Status = ObReferenceObjectByHandle(SectionHandle,
                                           SECTION_MAP_EXECUTE,
                                           MmSectionObjectType,
                                           KernelMode,
                                           (PVOID*)&Section,
                                           NULL);
        ZwClose(SectionHandle);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("MmLoadSystemImage: ObReferenceObjectByHandle(%wZ) failed: 0x%x\n", FileName, Status);
            goto Quickie;
        }

        if (Flags)
        {
            goto Quickie;
        }

        goto LoaderScan;
    }
    else
    {
        LdrEntry = NULL;
    }

    /* Load the image */
    Status = MiLoadImageSection(&Section,
                                     &ModuleLoadBase,
                                     FileName,
                                     FALSE,
                                     NULL);
    ASSERT(Status != STATUS_ALREADY_COMMITTED);

    /* Get the size of the driver */
    DriverSize = ((PMM_IMAGE_SECTION_OBJECT)Section->Segment)->ImageInformation.ImageFileSize;

    /* Make sure we're not being loaded into session space */
    if (!Flags)
    {
        if (NT_SUCCESS(Status))
        {
            MiUseLargeDriverPage_shim(DriverSize / PAGE_SIZE,
                                      &ModuleLoadBase,
                                      &BaseName,
                                      TRUE);
        }

        ObDereferenceObject(Section);
        Section = NULL;
    }

    if (!NT_SUCCESS(Status))
    {
        DPRINT1("MiLoadImageSection failed with status 0x%x\n", Status);
        goto Quickie;
    }

    /* Relocate the driver */
    Status = LdrRelocateImageWithBias(ModuleLoadBase,
                                      0,
                                      "SYSLDR",
                                      STATUS_SUCCESS,
                                      STATUS_CONFLICTING_ADDRESSES,
                                      STATUS_INVALID_IMAGE_FORMAT);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("LdrRelocateImageWithBias failed with status 0x%x\n", Status);
        goto Quickie;
    }

    /* Get the NT Header */
    NtHeader = RtlImageNtHeader(ModuleLoadBase);

    /* Calculate the size we'll need for the entry and allocate it */
    EntrySize = sizeof(LDR_DATA_TABLE_ENTRY) +
                BaseName.Length +
                sizeof(UNICODE_NULL);

    LdrEntry = ExAllocatePoolWithTag(NonPagedPool, EntrySize, TAG_MODULE_OBJECT);
    if (!LdrEntry)
    {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        goto Quickie;
    }

    /* Setup the entry */
    LdrEntry->Flags = LDRP_LOAD_IN_PROGRESS;
    LdrEntry->LoadCount = 1;
    LdrEntry->LoadedImports = LoadedImports;
    LdrEntry->PatchInformation = NULL;

    if ((NtHeader->OptionalHeader.MajorOperatingSystemVersion >= 5) &&
        (NtHeader->OptionalHeader.MajorImageVersion >= 5))
    {
        LdrEntry->Flags |= LDRP_ENTRY_NATIVE;
    }

    LdrEntry->DllBase = ModuleLoadBase;
    LdrEntry->EntryPoint = (PVOID)((ULONG_PTR)ModuleLoadBase +
                                   NtHeader->OptionalHeader.AddressOfEntryPoint);
    LdrEntry->SizeOfImage = DriverSize;
    LdrEntry->CheckSum = NtHeader->OptionalHeader.CheckSum;
    LdrEntry->SectionPointer = Section;

    /* Now write the DLL name */
    LdrEntry->BaseDllName.Buffer = (PVOID)(LdrEntry + 1);
    LdrEntry->BaseDllName.Length = BaseName.Length;
    LdrEntry->BaseDllName.MaximumLength = BaseName.Length;

    RtlCopyMemory(LdrEntry->BaseDllName.Buffer,
                  BaseName.Buffer,
                  BaseName.Length);
    LdrEntry->BaseDllName.Buffer[BaseName.Length / sizeof(WCHAR)] = UNICODE_NULL;

    /* Now allocate the full name */
    LdrEntry->FullDllName.Buffer = ExAllocatePoolWithTag(PagedPool,
                                                         PrefixName.Length +
                                                         sizeof(UNICODE_NULL),
                                                         TAG_LDR_WSTR);
    if (!LdrEntry->FullDllName.Buffer)
    {
        LdrEntry->FullDllName.Length = 0;
        LdrEntry->FullDllName.MaximumLength = 0;
    }
    else
    {
        LdrEntry->FullDllName.Length = PrefixName.Length;
        LdrEntry->FullDllName.MaximumLength = PrefixName.Length;
        RtlCopyMemory(LdrEntry->FullDllName.Buffer,
                      PrefixName.Buffer,
                      PrefixName.Length);
        LdrEntry->FullDllName.Buffer[PrefixName.Length / sizeof(WCHAR)] = UNICODE_NULL;
    }

    /* Add the entry */
    MiProcessLoaderEntry_shim(LdrEntry, TRUE);

    /* Resolve imports */
    MissingApiName = Buffer;
    MissingDriverName = NULL;
    Status = MiResolveImageReferences(ModuleLoadBase,
                                      &BaseDirectory,
                                      NULL,
                                      &MissingApiName,
                                      &MissingDriverName,
                                      &LoadedImports);
    if (!NT_SUCCESS(Status))
    {
        BOOLEAN NeedToFreeString = FALSE;

        if (*(ULONG_PTR*)&MissingDriverName & 1)
        {
            NeedToFreeString = TRUE;
            *(ULONG_PTR*)&MissingDriverName &= ~1;
        }

        DPRINT1("MiResolveImageReferences failed with status 0x%x\n", Status);
        DPRINT1(" Missing driver '%ls', missing API '%s'\n",
                MissingDriverName, MissingApiName);

        if (NeedToFreeString)
        {
            ExFreePoolWithTag(MissingDriverName, TAG_LDR_WSTR);
        }

        MiProcessLoaderEntry_shim(LdrEntry, FALSE);

        if (LdrEntry->FullDllName.Buffer)
        {
            ExFreePoolWithTag(LdrEntry->FullDllName.Buffer, TAG_LDR_WSTR);
        }

        ExFreePoolWithTag(LdrEntry, TAG_MODULE_OBJECT);
        LdrEntry = NULL;
        goto Quickie;
    }

    /* Update the loader entry */
    LdrEntry->Flags |= (LDRP_SYSTEM_MAPPED |
                        LDRP_ENTRY_PROCESSED |
                        LDRP_MM_LOADED);
    LdrEntry->Flags &= ~LDRP_LOAD_IN_PROGRESS;
    LdrEntry->LoadedImports = LoadedImports;

    /* Write-protect the system image */
    MiWriteProtectSystemImage(LdrEntry->DllBase);

    /* Initialize the security cookie */
    LdrpInitSecurityCookie_shim(LdrEntry);

    /* Check if notifications are enabled */
    if (PsImageNotifyEnabled)
    {
        ImageInfo.Properties = 0;
        ImageInfo.ImageAddressingMode = IMAGE_ADDRESSING_MODE_32BIT;
        ImageInfo.SystemModeImage = TRUE;
        ImageInfo.ImageSize = LdrEntry->SizeOfImage;
        ImageInfo.ImageBase = LdrEntry->DllBase;
        ImageInfo.ImageSectionNumber = ImageInfo.ImageSelector = 0;

        PspRunLoadImageNotifyRoutines(FileName, NULL, &ImageInfo);
    }

#ifdef __ROS_ROSSYM__
    if (TRUE)
#else
    if (MiCacheImageSymbols_shim(LdrEntry->DllBase))
#endif
    {
        UNICODE_STRING UnicodeTemp;
        STRING AnsiTemp;

        if ((PrefixName.Length > (11 * sizeof(WCHAR))) &&
            !(_wcsnicmp(PrefixName.Buffer, L"\\SystemRoot", 11)))
        {
            UnicodeTemp = PrefixName;
            UnicodeTemp.Buffer += 11;
            UnicodeTemp.Length -= (11 * sizeof(WCHAR));
            RtlStringCbPrintfA(Buffer,
                               MAXIMUM_FILENAME_LENGTH,
                               "%ws%wZ",
                               &SharedUserData->NtSystemRoot[2],
                               &UnicodeTemp);
        }
        else
        {
            RtlStringCbPrintfA(Buffer, MAXIMUM_FILENAME_LENGTH,
                               "%wZ", &BaseName);
        }

        RtlInitString(&AnsiTemp, Buffer);

        DbgLoadImageSymbols(&AnsiTemp,
                            LdrEntry->DllBase,
                            (ULONG_PTR)PsGetCurrentProcessId());
        LdrEntry->Flags |= LDRP_DEBUG_SYMBOLS_LOADED;
    }

    /* Page the driver */
    ASSERT(Section == NULL);
    MiEnablePagingOfDriver_shim(LdrEntry);

    /* Return pointers */
    *ModuleObject = LdrEntry;
    *ImageBaseAddress = LdrEntry->DllBase;

Quickie:
    if (LockOwned)
    {
        KeReleaseMutant(&MmSystemLoadLock, MUTANT_INCREMENT, FALSE, FALSE);
        KeLeaveCriticalRegion();
        LockOwned = FALSE;
    }

    if (FileHandle) ZwClose(FileHandle);
    if (PrefixedBuffer) ExFreePoolWithTag(PrefixedBuffer, TAG_LDR_WSTR);
    ExFreePoolWithTag(Buffer, TAG_LDR_WSTR);
    return Status;
}

/* MiLookupDataTableEntry — translated to Rust (sysldr.rs::MiLookupDataTableEntry)
 * and exported under the same name; C body deleted. */
