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
static NTSTATUS NTAPI MiSnapThunk(PVOID, PVOID, PIMAGE_THUNK_DATA, PIMAGE_THUNK_DATA, PIMAGE_EXPORT_DIRECTORY, ULONG, BOOLEAN, PCHAR *);
static USHORT NTAPI NameToOrdinal(PCSTR, PVOID, ULONG, PULONG, PUSHORT);

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

/* ========== MiLoadImageSection ========== */

static NTSTATUS
NTAPI
MiLoadImageSection_shim(_Inout_ PSECTION *SectionPtr,
                        _Out_ PVOID *ImageBase,
                        _In_ PUNICODE_STRING FileName,
                        _In_ BOOLEAN SessionLoad,
                        _In_ PLDR_DATA_TABLE_ENTRY LdrEntry)
{
    PSECTION Section = *SectionPtr;
    NTSTATUS Status;
    PFN_COUNT PteCount;
    PMMPTE PointerPte, LastPte;
    PVOID DriverBase;
    MMPTE TempPte;
    KIRQL OldIrql;
    PFN_NUMBER PageFrameIndex;
    PEPROCESS Process;
    PVOID Base = NULL;
    SIZE_T ViewSize = 0;
    KAPC_STATE ApcState;
    LARGE_INTEGER SectionOffset = {{0, 0}};
    BOOLEAN LoadSymbols = FALSE;
    PAGED_CODE();

    /* Detect session load */
    if (SessionLoad)
    {
        return STATUS_NOT_IMPLEMENTED;
    }

    /* Not session load, shouldn't have an entry */
    ASSERT(LdrEntry == NULL);

    /* Reserve system PTEs needed */
    PteCount = ROUND_TO_PAGES(((PMM_IMAGE_SECTION_OBJECT)Section->Segment)->ImageInformation.ImageFileSize) >> PAGE_SHIFT;
    PointerPte = MiReserveSystemPtes(PteCount, SystemPteSpace);
    if (!PointerPte)
    {
        DPRINT1("MiReserveSystemPtes failed\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* New driver base */
    LastPte = PointerPte + PteCount;
    DriverBase = MiPteToAddress(PointerPte);
    *ImageBase = DriverBase;

    /* Lock the PFN database */
    OldIrql = MiAcquirePfnLock();

    /* Loop the new driver PTEs */
    TempPte = ValidKernelPte;
    while (PointerPte < LastPte)
    {
        ASSERT(PointerPte->u.Hard.Valid == 0);
        MI_SET_USAGE(MI_USAGE_DRIVER_PAGE);
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        MiInitializePfn(PageFrameIndex, PointerPte, TRUE);
        TempPte.u.Hard.PageFrameNumber = PageFrameIndex;
        MI_WRITE_VALID_PTE(PointerPte, TempPte);
        PointerPte++;
    }

    MiReleasePfnLock(OldIrql);

    /* Attach to the system process */
    KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);

    /* Check if we need to load symbols */
    if (NtGlobalFlag & FLG_ENABLE_KDEBUG_SYMBOL_LOAD)
    {
        LoadSymbols = TRUE;
        NtGlobalFlag &= ~FLG_ENABLE_KDEBUG_SYMBOL_LOAD;
    }

    /* Map the driver */
    Process = PsGetCurrentProcess();
    Status = MmMapViewOfSection(Section,
                                Process,
                                &Base,
                                0,
                                0,
                                &SectionOffset,
                                &ViewSize,
                                ViewUnmap,
                                0,
                                PAGE_EXECUTE);

    /* Re-enable the flag */
    if (LoadSymbols) NtGlobalFlag |= FLG_ENABLE_KDEBUG_SYMBOL_LOAD;

    /* Check if we failed with distinguished status code */
    if (Status == STATUS_IMAGE_MACHINE_TYPE_MISMATCH)
    {
        Status = STATUS_INVALID_IMAGE_FORMAT;
    }

    if (!NT_SUCCESS(Status))
    {
        DPRINT1("MmMapViewOfSection failed with status 0x%x\n", Status);
        KeUnstackDetachProcess(&ApcState);
        return Status;
    }

    /* Clear stale PTEs in the section view range before reading.
     * On amd64, FreeLDR's identity-mapped page table pages can be reused
     * for user-space page tables with stale Valid PTEs (e.g., PFN=0).
     * Additionally, MmUnmapViewOfSection doesn't always clear PTEs from
     * previous section views. These stale PTEs prevent page faults from
     * firing, so the section data never gets demand-paged in. Zeroing them
     * forces proper fault resolution from the section. */
#ifdef _M_AMD64
    {
        PMMPTE StartPte = MiAddressToPte(Base);
        PMMPTE EndPte = MiAddressToPte((PVOID)((ULONG_PTR)Base + (PteCount << PAGE_SHIFT) - 1));
        PMMPTE Pte;
        ULONG Cleared = 0;
        for (Pte = StartPte; Pte <= EndPte; Pte++)
        {
            if (Pte->u.Long != 0)
            {
                Pte->u.Long = 0;
                __invlpg(MiPteToAddress(Pte));
                Cleared++;
            }
        }
        if (Cleared > 0)
        {
            /* Fix up UsedPageTableEntries to match the cleared count.
             * The fault handler incremented this counter when it populated
             * PTEs during previous section loads, but MmUnmapViewOfSection
             * didn't decrement it when unmapping. Correct it now. */
            PMMPDE Pde = MiAddressToPde(Base);
            if (Pde->u.Hard.Valid)
            {
                PMMPFN PtPagePfn = MiGetPfnEntry(Pde->u.Hard.PageFrameNumber);
                USHORT OldCount = PtPagePfn->OriginalPte.u.Soft.UsedPageTableEntries;
                if (OldCount >= Cleared)
                    PtPagePfn->OriginalPte.u.Soft.UsedPageTableEntries = (USHORT)(OldCount - Cleared);
                else
                    PtPagePfn->OriginalPte.u.Soft.UsedPageTableEntries = 0;
            }
        }
    }
#endif

    /* Copy image from section view to kernel address */
    RtlCopyMemory(DriverBase, Base, PteCount << PAGE_SHIFT);

    /* Now unmap the view */
    Status = MmUnmapViewOfSection(Process, Base);
    ASSERT(NT_SUCCESS(Status));

    /* Detach and return status */
    KeUnstackDetachProcess(&ApcState);

    return Status;
}

/* ========== NameToOrdinal ========== */

#ifndef RVA
#define RVA(m, b) ((PVOID)((ULONG_PTR)(b) + (ULONG_PTR)(m)))
#endif

static USHORT
NTAPI
NameToOrdinal(
    _In_ PCSTR ExportName,
    _In_ PVOID ImageBase,
    _In_ ULONG NumberOfNames,
    _In_ PULONG NameTable,
    _In_ PUSHORT OrdinalTable)
{
    LONG Low, Mid, High, Ret;

    if (!NumberOfNames)
        return -1;

    Low = Mid = 0;
    High = NumberOfNames - 1;
    while (High >= Low)
    {
        Mid = (Low + High) >> 1;
        Ret = strcmp(ExportName, (PCHAR)RVA(ImageBase, NameTable[Mid]));
        if (Ret < 0)
            High = Mid - 1;
        else if (Ret > 0)
            Low = Mid + 1;
        else
            break;
    }

    if (High < Low)
        return -1;

    return OrdinalTable[Mid];
}

/* ========== MiSnapThunk ========== */

static NTSTATUS
NTAPI
MiSnapThunk(IN PVOID DllBase,
            IN PVOID ImageBase,
            IN PIMAGE_THUNK_DATA Name,
            IN PIMAGE_THUNK_DATA Address,
            IN PIMAGE_EXPORT_DIRECTORY ExportDirectory,
            IN ULONG ExportSize,
            IN BOOLEAN SnapForwarder,
            OUT PCHAR *MissingApi)
{
    BOOLEAN IsOrdinal;
    USHORT Ordinal;
    PULONG NameTable;
    PUSHORT OrdinalTable;
    PIMAGE_IMPORT_BY_NAME NameImport;
    USHORT Hint;
    NTSTATUS Status;
    PCHAR MissingForwarder;
    CHAR NameBuffer[MAXIMUM_FILENAME_LENGTH];
    PULONG ExportTable;
    ANSI_STRING DllName;
    UNICODE_STRING ForwarderName;
    PLIST_ENTRY NextEntry;
    PLDR_DATA_TABLE_ENTRY LdrEntry;
    ULONG ForwardExportSize;
    PIMAGE_EXPORT_DIRECTORY ForwardExportDirectory;
    PIMAGE_IMPORT_BY_NAME ForwardName;
    SIZE_T ForwardLength;
    IMAGE_THUNK_DATA ForwardThunk;

    PAGED_CODE();

    /* Check if this is an ordinal */
    IsOrdinal = IMAGE_SNAP_BY_ORDINAL(Name->u1.Ordinal);
    if ((IsOrdinal) && !(SnapForwarder))
    {
        Ordinal = (USHORT)(IMAGE_ORDINAL(Name->u1.Ordinal) -
                           ExportDirectory->Base);
        *MissingApi = (PCHAR)(ULONG_PTR)Ordinal;
    }
    else
    {
        /* Get the VA if we don't have to snap */
        if (!SnapForwarder) Name->u1.AddressOfData += (ULONG_PTR)ImageBase;
        NameImport = (PIMAGE_IMPORT_BY_NAME)Name->u1.AddressOfData;

        /* Copy the procedure name */
        RtlStringCbCopyA(*MissingApi,
                         MAXIMUM_FILENAME_LENGTH,
                         (PCHAR)NameImport->Name);

        /* Setup name tables */
        NameTable = (PULONG)((ULONG_PTR)DllBase +
                             ExportDirectory->AddressOfNames);
        OrdinalTable = (PUSHORT)((ULONG_PTR)DllBase +
                                 ExportDirectory->AddressOfNameOrdinals);

        /* Get the hint and check if it's valid */
        Hint = NameImport->Hint;
        if ((Hint < ExportDirectory->NumberOfNames) &&
            !(strcmp((PCHAR)NameImport->Name, (PCHAR)DllBase + NameTable[Hint])))
        {
            Ordinal = OrdinalTable[Hint];
        }
        else
        {
            /* Do a binary search */
            Ordinal = NameToOrdinal((PCHAR)NameImport->Name,
                                    DllBase,
                                    ExportDirectory->NumberOfNames,
                                    NameTable,
                                    OrdinalTable);
            if (Ordinal == (USHORT)-1)
            {
                DPRINT1("Warning: Driver failed to load, %s not found\n", NameImport->Name);
                return STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
            }
        }
    }

    /* Check if the ordinal is valid */
    if (Ordinal >= ExportDirectory->NumberOfFunctions)
    {
        Status = STATUS_DRIVER_ORDINAL_NOT_FOUND;
    }
    else
    {
        /* In case the forwarder is missing */
        MissingForwarder = NameBuffer;

        /* Resolve the address and write it */
        ExportTable = (PULONG)((ULONG_PTR)DllBase +
                               ExportDirectory->AddressOfFunctions);
        Address->u1.Function = (ULONG_PTR)DllBase + ExportTable[Ordinal];

        /* Assume success from now on */
        Status = STATUS_SUCCESS;

        /* Check if the function is actually a forwarder */
        if ((Address->u1.Function > (ULONG_PTR)ExportDirectory) &&
            (Address->u1.Function < (ULONG_PTR)ExportDirectory + ExportSize))
        {
            /* Now assume failure in case the forwarder doesn't exist */
            Status = STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;

            /* Build the forwarder name */
            DllName.Buffer = (PCHAR)Address->u1.Function;

            /* lld (llvm linker) prepends '_' to forwarded export DLL names
             * on i386. Strip it so module lookup succeeds. */
            if (DllName.Buffer[0] == '_' && strchr(DllName.Buffer + 1, '.'))
                DllName.Buffer++;

            DllName.Length = (USHORT)(strchr(DllName.Buffer, '.') -
                                      DllName.Buffer) +
                                      sizeof(ANSI_NULL);
            DllName.MaximumLength = DllName.Length;

            /* Convert it */
            if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&ForwarderName,
                                                         &DllName,
                                                         TRUE)))
            {
                return Status;
            }

            /* Loop the module list */
            NextEntry = PsLoadedModuleList.Flink;
            while (NextEntry != &PsLoadedModuleList)
            {
                LdrEntry = CONTAINING_RECORD(NextEntry,
                                             LDR_DATA_TABLE_ENTRY,
                                             InLoadOrderLinks);

                if (RtlPrefixUnicodeString(&ForwarderName,
                                           &LdrEntry->BaseDllName,
                                           TRUE))
                {
                    ForwardExportDirectory =
                        RtlImageDirectoryEntryToData(LdrEntry->DllBase,
                                                     TRUE,
                                                     IMAGE_DIRECTORY_ENTRY_EXPORT,
                                                     &ForwardExportSize);
                    if (!ForwardExportDirectory) break;

                    ForwardLength = strlen(DllName.Buffer + DllName.Length) +
                                    sizeof(ANSI_NULL);
                    ForwardName = ExAllocatePoolWithTag(PagedPool,
                                                        sizeof(*ForwardName) +
                                                        ForwardLength,
                                                        TAG_LDR_WSTR);
                    if (!ForwardName) break;

                    RtlCopyMemory(&ForwardName->Name[0],
                                  DllName.Buffer + DllName.Length,
                                  ForwardLength);
                    ForwardName->Hint = 0;

                    ForwardThunk.u1.AddressOfData = (ULONG_PTR)ForwardName;

                    Status = MiSnapThunk(LdrEntry->DllBase,
                                         ImageBase,
                                         &ForwardThunk,
                                         &ForwardThunk,
                                         ForwardExportDirectory,
                                         ForwardExportSize,
                                         TRUE,
                                         &MissingForwarder);

                    ExFreePoolWithTag(ForwardName, TAG_LDR_WSTR);
                    Address->u1 = ForwardThunk.u1;
                    break;
                }

                NextEntry = NextEntry->Flink;
            }

            RtlFreeUnicodeString(&ForwarderName);
        }
    }

    return Status;
}

/* ========== MiResolveImageReferences ========== */

NTSTATUS
NTAPI
MiResolveImageReferences(IN PVOID ImageBase,
                         IN PUNICODE_STRING ImageFileDirectory,
                         IN PUNICODE_STRING NamePrefix OPTIONAL,
                         OUT PCHAR *MissingApi,
                         OUT PWCHAR *MissingDriver,
                         OUT PLOAD_IMPORTS *LoadImports)
{
    static UNICODE_STRING DriversFolderName = RTL_CONSTANT_STRING(L"drivers\\");
    PCHAR MissingApiBuffer = *MissingApi, ImportName;
    PIMAGE_IMPORT_DESCRIPTOR ImportDescriptor, CurrentImport;
    ULONG ImportSize, ImportCount = 0, LoadedImportsSize, ExportSize;
    PLOAD_IMPORTS LoadedImports, NewImports;
    ULONG i;
    BOOLEAN GdiLink, NormalLink;
    BOOLEAN ReferenceNeeded, Loaded;
    ANSI_STRING TempString;
    UNICODE_STRING NameString, DllName;
    PLDR_DATA_TABLE_ENTRY LdrEntry = NULL, DllEntry, ImportEntry = NULL;
    PVOID ImportBase, DllBase;
    PLIST_ENTRY NextEntry;
    PIMAGE_EXPORT_DIRECTORY ExportDirectory;
    NTSTATUS Status;
    PIMAGE_THUNK_DATA OrigThunk, FirstThunk;

    PAGED_CODE();

    /* No name string buffer yet */
    NameString.Buffer = NULL;

    /* Assume no imports */
    *LoadImports = MM_SYSLDR_NO_IMPORTS;

    /* Get the import descriptor */
    ImportDescriptor = RtlImageDirectoryEntryToData(ImageBase,
                                                    TRUE,
                                                    IMAGE_DIRECTORY_ENTRY_IMPORT,
                                                    &ImportSize);
    if (!ImportDescriptor) return STATUS_SUCCESS;

    /* Loop all imports to count them */
    for (CurrentImport = ImportDescriptor;
         (CurrentImport->Name) && (CurrentImport->OriginalFirstThunk);
         CurrentImport++)
    {
        ImportCount++;
    }

    /* Make sure we have non-zero imports */
    if (ImportCount)
    {
        LoadedImportsSize = ImportCount * sizeof(PVOID) + sizeof(SIZE_T);
        LoadedImports = ExAllocatePoolWithTag(PagedPool,
                                              LoadedImportsSize,
                                              TAG_LDR_IMPORTS);
        if (LoadedImports)
        {
            RtlZeroMemory(LoadedImports, LoadedImportsSize);
            LoadedImports->Count = ImportCount;
        }
    }
    else
    {
        LoadedImports = NULL;
    }

    /* Reset the import count and loop descriptors again */
    GdiLink = NormalLink = FALSE;
    ImportCount = 0;
    while ((ImportDescriptor->Name) && (ImportDescriptor->OriginalFirstThunk))
    {
        /* Get the name */
        ImportName = (PCHAR)((ULONG_PTR)ImageBase + ImportDescriptor->Name);

        /* Check if this is a GDI driver */
        GdiLink = GdiLink ||
                  !(_strnicmp(ImportName, "win32k", sizeof("win32k") - 1));

        NormalLink = NormalLink ||
                     ((_strnicmp(ImportName, "win32k", sizeof("win32k") - 1)) &&
                      (_strnicmp(ImportName, "dxapi", sizeof("dxapi") - 1)) &&
                      (_strnicmp(ImportName, "coverage", sizeof("coverage") - 1)) &&
                      (_strnicmp(ImportName, "irt", sizeof("irt") - 1)));

        if (GdiLink && NormalLink)
        {
            Status = STATUS_PROCEDURE_NOT_FOUND;
            goto Failure;
        }

        /* Check for user-mode drivers */
        if (!(_strnicmp(ImportName, "ntdll", sizeof("ntdll") - 1)) ||
            !(_strnicmp(ImportName, "winsrv", sizeof("winsrv") - 1)) ||
            !(_strnicmp(ImportName, "advapi32", sizeof("advapi32") - 1)) ||
            !(_strnicmp(ImportName, "kernel32", sizeof("kernel32") - 1)) ||
            !(_strnicmp(ImportName, "user32", sizeof("user32") - 1)) ||
            !(_strnicmp(ImportName, "gdi32", sizeof("gdi32") - 1)))
        {
            Status = STATUS_PROCEDURE_NOT_FOUND;
            goto Failure;
        }

        /* Check if this is a "core" import */
        if (!(_strnicmp(ImportName, "ntoskrnl", sizeof("ntoskrnl") - 1)) ||
            !(_strnicmp(ImportName, "win32k", sizeof("win32k") - 1)) ||
            !(_strnicmp(ImportName, "hal", sizeof("hal") - 1)))
        {
            ReferenceNeeded = FALSE;
        }
        else
        {
            ReferenceNeeded = TRUE;
        }

        /* Setup a unicode string for the import */
        RtlInitAnsiString(&TempString, ImportName);
        Status = RtlAnsiStringToUnicodeString(&NameString, &TempString, TRUE);
        if (!NT_SUCCESS(Status))
        {
            goto Failure;
        }

        if (NamePrefix) DPRINT1("Name Prefix not yet supported!\n");

    CheckDllState:
        Loaded = FALSE;
        ImportBase = NULL;

        /* Loop the driver list */
        NextEntry = PsLoadedModuleList.Flink;
        while (NextEntry != &PsLoadedModuleList)
        {
            LdrEntry = CONTAINING_RECORD(NextEntry,
                                         LDR_DATA_TABLE_ENTRY,
                                         InLoadOrderLinks);
            if (RtlEqualUnicodeString(&NameString,
                                      &LdrEntry->BaseDllName,
                                      TRUE))
            {
                ImportBase = LdrEntry->DllBase;
                if (!(Loaded) && (ReferenceNeeded))
                {
                    if (!(LdrEntry->Flags & LDRP_LOAD_IN_PROGRESS))
                    {
                        LdrEntry->LoadCount++;
                    }
                }
                break;
            }
            NextEntry = NextEntry->Flink;
        }

        /* Check if we haven't loaded the import yet */
        if (!ImportBase)
        {
            DllName.MaximumLength = NameString.Length +
                                    ImageFileDirectory->Length +
                                    sizeof(UNICODE_NULL);
            DllName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
                                                   DllName.MaximumLength,
                                                   TAG_LDR_WSTR);
            if (!DllName.Buffer)
            {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                goto Failure;
            }

            RtlCopyUnicodeString(&DllName, ImageFileDirectory);
            RtlAppendUnicodeStringToString(&DllName, &NameString);

            /* Load the image */
            Status = MmLoadSystemImage(&DllName,
                                       NamePrefix,
                                       NULL,
                                       FALSE,
                                       (PVOID *)&DllEntry,
                                       &DllBase);

            /* win32k / GDI drivers can also import from system32 folder */
            if ((Status == STATUS_OBJECT_NAME_NOT_FOUND) &&
                (MI_IS_SESSION_ADDRESS(ImageBase) || 1)) // HACK
            {
                ExFreePoolWithTag(DllName.Buffer, TAG_LDR_WSTR);
                DllName.MaximumLength += DriversFolderName.Length;
                DllName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
                                                       DllName.MaximumLength,
                                                       TAG_LDR_WSTR);
                if (!DllName.Buffer)
                {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    goto Failure;
                }

                RtlCopyUnicodeString(&DllName, ImageFileDirectory);
                RtlAppendUnicodeStringToString(&DllName, &DriversFolderName);
                RtlAppendUnicodeStringToString(&DllName, &NameString);

                Status = MmLoadSystemImage(&DllName,
                                           NamePrefix,
                                           NULL,
                                           FALSE,
                                           (PVOID *)&DllEntry,
                                           &DllBase);
            }

            if (!NT_SUCCESS(Status))
            {
                *MissingDriver = DllName.Buffer;
                *(PULONG)MissingDriver |= 1;
                *MissingApi = NULL;
                DPRINT1("Failed to load dependency: %wZ\n", &DllName);
                DllName.Buffer = NULL;
                goto Failure;
            }

            ExFreePoolWithTag(DllName.Buffer, TAG_LDR_WSTR);
            DllName.Buffer = NULL;

            Loaded = TRUE;
            ASSERT(DllBase == DllEntry->DllBase);

            Status = MmCallDllInitialize(DllEntry, &PsLoadedModuleList);
            if (!NT_SUCCESS(Status))
            {
                MmUnloadSystemImage(DllEntry);
                DPRINT1("MmCallDllInitialize failed with status 0x%x\n", Status);
                Loaded = FALSE;
            }

            goto CheckDllState;
        }

        /* Check if we're supposed to reference this import */
        if ((ReferenceNeeded) && (LoadedImports))
        {
            if (!(LdrEntry->Flags & LDRP_LOAD_IN_PROGRESS))
            {
                LoadedImports->Entry[ImportCount] = LdrEntry;
                ImportCount++;
            }
        }

        RtlFreeUnicodeString(&NameString);

        *MissingDriver = LdrEntry->BaseDllName.Buffer;
        ExportDirectory = RtlImageDirectoryEntryToData(ImportBase,
                                                       TRUE,
                                                       IMAGE_DIRECTORY_ENTRY_EXPORT,
                                                       &ExportSize);
        if (!ExportDirectory)
        {
            DPRINT1("Warning: Driver failed to load, %S not found\n", *MissingDriver);
            Status = STATUS_DRIVER_ENTRYPOINT_NOT_FOUND;
            goto Failure;
        }

        /* Make sure we have an IAT */
        if (ImportDescriptor->OriginalFirstThunk)
        {
            OrigThunk = (PVOID)((ULONG_PTR)ImageBase +
                                ImportDescriptor->OriginalFirstThunk);
            FirstThunk = (PVOID)((ULONG_PTR)ImageBase +
                                 ImportDescriptor->FirstThunk);

            while (OrigThunk->u1.AddressOfData)
            {
                Status = MiSnapThunk(ImportBase,
                                     ImageBase,
                                     OrigThunk++,
                                     FirstThunk++,
                                     ExportDirectory,
                                     ExportSize,
                                     FALSE,
                                     MissingApi);
                if (!NT_SUCCESS(Status))
                {
                    goto Failure;
                }

                *MissingApi = MissingApiBuffer;
            }
        }

        ImportDescriptor++;
    }

    /* Check if we have an import list */
    if (LoadedImports)
    {
        ImportCount = 0;
        for (i = 0; i < LoadedImports->Count; i++)
        {
            if (LoadedImports->Entry[i])
            {
                ImportEntry = (PVOID)((ULONG_PTR)LoadedImports->Entry[i] |
                                      MM_SYSLDR_SINGLE_ENTRY);
                ImportCount++;
            }
        }

        if (!ImportCount)
        {
            ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
            LoadedImports = MM_SYSLDR_NO_IMPORTS;
        }
        else if (ImportCount == 1)
        {
            ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
            LoadedImports = (PLOAD_IMPORTS)ImportEntry;
        }
        else if (ImportCount != LoadedImports->Count)
        {
            LoadedImportsSize = ImportCount * sizeof(PVOID) + sizeof(SIZE_T);
            NewImports = ExAllocatePoolWithTag(PagedPool,
                                               LoadedImportsSize,
                                               TAG_LDR_IMPORTS);
            if (NewImports)
            {
                NewImports->Count = 0;
                for (i = 0; i < LoadedImports->Count; i++)
                {
                    if (LoadedImports->Entry[i])
                    {
                        NewImports->Entry[NewImports->Count] = LoadedImports->Entry[i];
                        NewImports->Count++;
                    }
                }
                ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
                LoadedImports = NewImports;
            }
        }

        *LoadImports = LoadedImports;
    }

    return STATUS_SUCCESS;

Failure:
    RtlFreeUnicodeString(&NameString);
    if (LoadedImports)
    {
        MiDereferenceImports_shim(LoadedImports);
        ExFreePoolWithTag(LoadedImports, TAG_LDR_IMPORTS);
    }
    return Status;
}

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
    Status = MiLoadImageSection_shim(&Section,
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
