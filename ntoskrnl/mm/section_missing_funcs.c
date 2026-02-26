/*
 * Extracted from ntoskrnl/mm/section.c
 * Functions NOT yet translated to Rust (31 functions total)
 */

/* INCLUDES *****************************************************************/

#include <ntoskrnl.h>
#include <cache/newcc.h>
#include <cache/section/newmm.h>
#define NDEBUG
#include <debug.h>
#include <reactos/exeformat.h>

#include "ARM3/miarm.h"

#undef MmSetPageEntrySectionSegment
#define MmSetPageEntrySectionSegment(S,O,E) \
        _MmSetPageEntrySectionSegment((S),(O),(E),__FILE__,__LINE__)

extern MMSESSION MmSession;

static LARGE_INTEGER TinyTime = {{-1L, -1L}};

#ifndef NEWCC
KEVENT MmWaitPageEvent;

VOID
NTAPI
_MmLockSectionSegment(PMM_SECTION_SEGMENT Segment, const char *file, int line)
{
    //DPRINT("MmLockSectionSegment(%p,%s:%d)\n", Segment, file, line);
    ExAcquireFastMutex(&Segment->Lock);
    Segment->Locked = TRUE;
}

VOID
NTAPI
_MmUnlockSectionSegment(PMM_SECTION_SEGMENT Segment, const char *file, int line)
{
    ASSERT(Segment->Locked);
    Segment->Locked = FALSE;
    ExReleaseFastMutex(&Segment->Lock);
    //DPRINT("MmUnlockSectionSegment(%p,%s:%d)\n", Segment, file, line);
}
#endif

/* TYPES *********************************************************************/

typedef struct
{
    PMEMORY_AREA MemoryArea;
    PMM_SECTION_SEGMENT Segment;
    LARGE_INTEGER Offset;
    BOOLEAN WasDirty;
    BOOLEAN Private;
    PEPROCESS CallingProcess;
    ULONG_PTR SectionEntry;
}
MM_SECTION_PAGEOUT_CONTEXT;

/* GLOBALS *******************************************************************/

static ULONG SectionCharacteristicsToProtect[16] =
{
    PAGE_NOACCESS,          /* 0 = NONE */
    PAGE_NOACCESS,          /* 1 = SHARED */
    PAGE_EXECUTE,           /* 2 = EXECUTABLE */
    PAGE_EXECUTE,           /* 3 = EXECUTABLE, SHARED */
    PAGE_READONLY,          /* 4 = READABLE */
    PAGE_READONLY,          /* 5 = READABLE, SHARED */
    PAGE_EXECUTE_READ,      /* 6 = READABLE, EXECUTABLE */
    PAGE_EXECUTE_READ,      /* 7 = READABLE, EXECUTABLE, SHARED */
    PAGE_READWRITE,         /* 8 = WRITABLE */
    PAGE_READWRITE,         /* 9 = WRITABLE, SHARED */
    PAGE_EXECUTE_READWRITE, /* 10 = WRITABLE, EXECUTABLE */
    PAGE_EXECUTE_READWRITE, /* 11 = WRITABLE, EXECUTABLE, SHARED */
    PAGE_READWRITE,         /* 12 = WRITABLE, READABLE */
    PAGE_READWRITE,         /* 13 = WRITABLE, READABLE, SHARED */
    PAGE_EXECUTE_READWRITE, /* 14 = WRITABLE, READABLE, EXECUTABLE */
    PAGE_EXECUTE_READWRITE, /* 15 = WRITABLE, READABLE, EXECUTABLE, SHARED */
};

extern ACCESS_MASK MmMakeFileAccess[8];

/* Forward declarations */
NTSTATUS
MiMapViewInSystemSpace(IN PVOID Section,
                       IN PVOID Session,
                       OUT PVOID *MappedBase,
                       IN OUT PSIZE_T ViewSize,
                       IN PLARGE_INTEGER SectionOffset);

NTSTATUS
NTAPI
MmCreateArm3Section(OUT PVOID *SectionObject,
                    IN ACCESS_MASK DesiredAccess,
                    IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
                    IN PLARGE_INTEGER InputMaximumSize,
                    IN ULONG SectionPageProtection,
                    IN ULONG AllocationAttributes,
                    IN HANDLE FileHandle OPTIONAL,
                    IN PFILE_OBJECT FileObject OPTIONAL);

NTSTATUS
NTAPI
MmMapViewOfArm3Section(IN PVOID SectionObject,
                       IN PEPROCESS Process,
                       IN OUT PVOID *BaseAddress,
                       IN ULONG_PTR ZeroBits,
                       IN SIZE_T CommitSize,
                       IN OUT PLARGE_INTEGER SectionOffset OPTIONAL,
                       IN OUT PSIZE_T ViewSize,
                       IN SECTION_INHERIT InheritDisposition,
                       IN ULONG AllocationType,
                       IN ULONG Protect);

//
// PeFmtCreateSection depends on the following:
//
C_ASSERT(EXEFMT_LOAD_HEADER_SIZE >= sizeof(IMAGE_DOS_HEADER));
C_ASSERT(sizeof(IMAGE_NT_HEADERS32) <= sizeof(IMAGE_NT_HEADERS64));

C_ASSERT(TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) == TYPE_ALIGNMENT(IMAGE_NT_HEADERS64));
C_ASSERT(RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS32, FileHeader) == RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS64, FileHeader));
C_ASSERT(FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader) == FIELD_OFFSET(IMAGE_NT_HEADERS64, OptionalHeader));

C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, Magic));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, SectionAlignment));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, FileAlignment));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, Subsystem));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, MinorSubsystemVersion));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, MajorSubsystemVersion));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, AddressOfEntryPoint));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, SizeOfCode));
C_ASSERT(PEFMT_FIELDS_EQUAL(IMAGE_OPTIONAL_HEADER32, IMAGE_OPTIONAL_HEADER64, SizeOfHeaders));

/* FUNCTIONS *****************************************************************/

static VOID
MmAlterViewAttributes(PMMSUPPORT AddressSpace,
                      PVOID BaseAddress,
                      SIZE_T RegionSize,
                      ULONG OldType,
                      ULONG OldProtect,
                      ULONG NewType,
                      ULONG NewProtect)
{
    PMEMORY_AREA MemoryArea;
    PMM_SECTION_SEGMENT Segment;
    BOOLEAN DoCOW = FALSE;
    ULONG i;
    PEPROCESS Process = MmGetAddressSpaceOwner(AddressSpace);

    MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, BaseAddress);
    ASSERT(MemoryArea != NULL);
    Segment = MemoryArea->SectionData.Segment;
    MmLockSectionSegment(Segment);

    if ((Segment->WriteCopy) &&
            (NewProtect == PAGE_READWRITE || NewProtect == PAGE_EXECUTE_READWRITE))
    {
        DoCOW = TRUE;
    }

    if (OldProtect != NewProtect)
    {
        for (i = 0; i < PAGE_ROUND_UP(RegionSize) / PAGE_SIZE; i++)
        {
            SWAPENTRY SwapEntry;
            PVOID Address = (char*)BaseAddress + (i * PAGE_SIZE);
            ULONG Protect = NewProtect;

            /* Wait for a wait entry to disappear */
            do
            {
                MmGetPageFileMapping(Process, Address, &SwapEntry);
                if (SwapEntry != MM_WAIT_ENTRY)
                    break;
                MmUnlockSectionSegment(Segment);
                MmUnlockAddressSpace(AddressSpace);
                KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);
                MmLockAddressSpace(AddressSpace);
                MmLockSectionSegment(Segment);
            }
            while (TRUE);

            /*
             * If we doing COW for this segment then check if the page is
             * already private.
             */
            if (DoCOW && (MmIsPagePresent(Process, Address) || MmIsDisabledPage(Process, Address)))
            {
                LARGE_INTEGER Offset;
                ULONG_PTR Entry;
                PFN_NUMBER Page;

                Offset.QuadPart = (ULONG_PTR)Address - MA_GetStartingAddress(MemoryArea)
                                  + MemoryArea->SectionData.ViewOffset;
                Entry = MmGetPageEntrySectionSegment(Segment, &Offset);
                /*
                 * An MM_WAIT_ENTRY is ok in this case...  It'll just count as
                 * IS_SWAP_FROM_SSE and we'll do the right thing.
                 */
                Page = MmGetPfnForProcess(Process, Address);

                /* Choose protection based on what was requested */
                if (NewProtect == PAGE_EXECUTE_READWRITE)
                    Protect = PAGE_EXECUTE_READ;
                else
                    Protect = PAGE_READONLY;

                if (IS_SWAP_FROM_SSE(Entry) || PFN_FROM_SSE(Entry) != Page)
                {
                    Protect = NewProtect;
                }
            }

            if (MmIsPagePresent(Process, Address) || MmIsDisabledPage(Process, Address))
            {
                MmSetPageProtect(Process, Address,
                                 Protect);
            }
        }
    }

    MmUnlockSectionSegment(Segment);
}

NTSTATUS
NTAPI
MmNotPresentFaultSectionView(PMMSUPPORT AddressSpace,
                             MEMORY_AREA* MemoryArea,
                             PVOID Address,
                             BOOLEAN Locked)
{
    LARGE_INTEGER Offset;
    PFN_NUMBER Page;
    NTSTATUS Status;
    PMM_SECTION_SEGMENT Segment;
    ULONG_PTR Entry;
    ULONG_PTR Entry1;
    ULONG Attributes;
    PMM_REGION Region;
    BOOLEAN HasSwapEntry;
    PVOID PAddress;
    PEPROCESS Process = MmGetAddressSpaceOwner(AddressSpace);
    SWAPENTRY SwapEntry;

    ASSERT(Locked);

    /*
     * There is a window between taking the page fault and locking the
     * address space when another thread could load the page so we check
     * that.
     */
    if (MmIsPagePresent(Process, Address))
    {
        return STATUS_SUCCESS;
    }

    if (MmIsDisabledPage(Process, Address))
    {
        return STATUS_ACCESS_VIOLATION;
    }

    /*
     * Check for the virtual memory area being deleted.
     */
    if (MemoryArea->DeleteInProgress)
    {
        return STATUS_UNSUCCESSFUL;
    }

    PAddress = MM_ROUND_DOWN(Address, PAGE_SIZE);
    Offset.QuadPart = (ULONG_PTR)PAddress - MA_GetStartingAddress(MemoryArea)
                      + MemoryArea->SectionData.ViewOffset;

    Segment = MemoryArea->SectionData.Segment;
    Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
                          &MemoryArea->SectionData.RegionListHead,
                          Address, NULL);
    ASSERT(Region != NULL);

    /* Check for a NOACCESS mapping */
    if (Region->Protect & PAGE_NOACCESS)
    {
        return STATUS_ACCESS_VIOLATION;
    }

    if (Region->Protect & PAGE_GUARD)
    {
        /* Remove it */
        Status = MmAlterRegion(AddressSpace, (PVOID)MA_GetStartingAddress(MemoryArea),
                &MemoryArea->SectionData.RegionListHead,
                Address, PAGE_SIZE, Region->Type, Region->Protect & ~PAGE_GUARD,
                MmAlterViewAttributes);

        if (!NT_SUCCESS(Status))
        {
            DPRINT1("Removing PAGE_GUARD protection failed : 0x%08x.\n", Status);
        }

        return STATUS_GUARD_PAGE_VIOLATION;
    }

    HasSwapEntry = MmIsPageSwapEntry(Process, Address);

    /* See if we should use a private page */
    if (HasSwapEntry)
    {
        SWAPENTRY DummyEntry;

        MmGetPageFileMapping(Process, Address, &SwapEntry);
        if (SwapEntry == MM_WAIT_ENTRY)
        {
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        MI_SET_USAGE(MI_USAGE_SECTION);
        if (Process) MI_SET_PROCESS2(Process->ImageFileName);
        if (!Process) MI_SET_PROCESS2("Kernel Section");
        Status = MmRequestPageMemoryConsumer(MC_USER, TRUE, &Page);
        if (!NT_SUCCESS(Status))
        {
            return STATUS_NO_MEMORY;
        }

        /*
         * Must be private page we have swapped out.
         */

        /*
        * Sanity check
        */
        MmDeletePageFileMapping(Process, Address, &DummyEntry);
        ASSERT(DummyEntry == SwapEntry);

        /* Tell everyone else we are serving the fault. */
        MmCreatePageFileMapping(Process, Address, MM_WAIT_ENTRY);

        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry, Page);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("MmReadFromSwapPage failed, status = %x\n", Status);
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmLockAddressSpace(AddressSpace);
        MmDeletePageFileMapping(Process, PAddress, &DummyEntry);
        ASSERT(DummyEntry == MM_WAIT_ENTRY);

        Status = MmCreateVirtualMapping(Process,
                                        PAddress,
                                        Region->Protect,
                                        Page);
        if (!NT_SUCCESS(Status))
        {
            DPRINT("MmCreateVirtualMapping failed, not out of memory\n");
            KeBugCheck(MEMORY_MANAGEMENT);
            return Status;
        }

        /*
         * Store the swap entry for later use.
         */
        MmSetSavedSwapEntryPage(Page, SwapEntry);

        /*
         * Add the page to the process's working set
         */
        if (Process) MmInsertRmap(Page, Process, Address);
        /*
         * Finish the operation
         */
        DPRINT("Address 0x%p\n", Address);
        return STATUS_SUCCESS;
    }

    /*
     * Lock the segment
     */
    MmLockSectionSegment(Segment);

    /*
     * Satisfying a page fault on a map of /Device/PhysicalMemory is easy
     */
    if ((*Segment->Flags) & MM_PHYSICALMEMORY_SEGMENT)
    {
        MmUnlockSectionSegment(Segment);
        /*
         * Just map the desired physical page
         */
        Page = (PFN_NUMBER)(Offset.QuadPart >> PAGE_SHIFT);
        Status = MmCreatePhysicalMapping(Process,
                                         PAddress,
                                         Region->Protect,
                                         Page);
        if (!NT_SUCCESS(Status))
        {
            DPRINT("MmCreateVirtualMappingUnsafe failed, not out of memory\n");
            KeBugCheck(MEMORY_MANAGEMENT);
            return Status;
        }

        /*
         * Cleanup and release locks
         */
        DPRINT("Address 0x%p\n", Address);
        return STATUS_SUCCESS;
    }

    /*
     * Check if this page needs to be mapped COW
     */
    if ((Segment->WriteCopy) &&
        (Region->Protect == PAGE_READWRITE || Region->Protect == PAGE_EXECUTE_READWRITE))
    {
        Attributes = Region->Protect == PAGE_READWRITE ? PAGE_READONLY : PAGE_EXECUTE_READ;
    }
    else
    {
        Attributes = Region->Protect;
    }


    /*
     * Get the entry corresponding to the offset within the section
     */
    Entry = MmGetPageEntrySectionSegment(Segment, &Offset);
    if (Entry == 0)
    {
        /*
         * If the entry is zero, then we need to load the page.
         */
        if ((Offset.QuadPart >= (LONGLONG)PAGE_ROUND_UP(Segment->RawLength.QuadPart)) && (MemoryArea->VadNode.u.VadFlags.VadType == VadImageMap))
        {
            /* We are beyond the data which is on file. Just get a new page. */
            MI_SET_USAGE(MI_USAGE_SECTION);
            if (Process) MI_SET_PROCESS2(Process->ImageFileName);
            if (!Process) MI_SET_PROCESS2("Kernel Section");
            Status = MmRequestPageMemoryConsumer(MC_USER, FALSE, &Page);
            if (!NT_SUCCESS(Status))
            {
                MmUnlockSectionSegment(Segment);
                return STATUS_NO_MEMORY;
            }
            MmSetPageEntrySectionSegment(Segment, &Offset, MAKE_SSE(Page << PAGE_SHIFT, 1));
            MmUnlockSectionSegment(Segment);

            Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
            if (!NT_SUCCESS(Status))
            {
                DPRINT1("Unable to create virtual mapping\n");
                KeBugCheck(MEMORY_MANAGEMENT);
            }
            ASSERT(MmIsPagePresent(Process, PAddress));
            if (Process)
                MmInsertRmap(Page, Process, Address);

            DPRINT("Address 0x%p\n", Address);
            return STATUS_SUCCESS;
        }

        MmUnlockSectionSegment(Segment);
        MmUnlockAddressSpace(AddressSpace);

        /* The data must be paged in. Lock the file, so that the VDL doesn't get updated behind us. */
        FsRtlAcquireFileExclusive(Segment->FileObject);

        PFSRTL_COMMON_FCB_HEADER FcbHeader = Segment->FileObject->FsContext;

        Status = MmMakeSegmentResident(Segment, Offset.QuadPart, PAGE_SIZE, &FcbHeader->ValidDataLength, FALSE);

        FsRtlReleaseFile(Segment->FileObject);

        /* Lock address space again */
        MmLockAddressSpace(AddressSpace);
        if (!NT_SUCCESS(Status))
        {
            if (Status == STATUS_NO_MEMORY)
            {
                return Status;
            }
            /* Damn */
            DPRINT1("Failed to page data in!\n");
            return STATUS_IN_PAGE_ERROR;
        }

        /* Everything went fine. Restart the operation */
        return STATUS_MM_RESTART_OPERATION;
    }
    else if (IS_SWAP_FROM_SSE(Entry))
    {
        SWAPENTRY SwapEntry;

        SwapEntry = SWAPENTRY_FROM_SSE(Entry);

        /* See if a page op is running on this segment. */
        if (SwapEntry == MM_WAIT_ENTRY)
        {
            MmUnlockSectionSegment(Segment);
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        Status = MmRequestPageMemoryConsumer(MC_USER, TRUE, &Page);
        if (!NT_SUCCESS(Status))
        {
            MmUnlockSectionSegment(Segment);
            return STATUS_NO_MEMORY;
        }

        /*
        * Release all our locks and read in the page from disk
        */
        MmSetPageEntrySectionSegment(Segment, &Offset, MAKE_SWAP_SSE(MM_WAIT_ENTRY));
        MmUnlockSectionSegment(Segment);

        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry, Page);
        if (!NT_SUCCESS(Status))
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        /*
         * Relock the address space and segment
         */
        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment);

        /*
         * Check the entry. No one should change the status of a page
         * that has a pending page-in.
         */
        Entry1 = MmGetPageEntrySectionSegment(Segment, &Offset);
        if (Entry1 != MAKE_SWAP_SSE(MM_WAIT_ENTRY))
        {
            DPRINT1("Someone changed ppte entry while we slept (%x vs %x)\n", Entry, Entry1);
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        /*
         * Save the swap entry.
         */
        MmSetSavedSwapEntryPage(Page, SwapEntry);

        /* Map the page into the process address space */
        Status = MmCreateVirtualMapping(Process,
                                        PAddress,
                                        Attributes,
                                        Page);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("Unable to create virtual mapping\n");
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        if (Process)
            MmInsertRmap(Page, Process, Address);

        /*
         * Mark the offset within the section as having valid, in-memory
         * data
         */
        Entry = MAKE_SSE(Page << PAGE_SHIFT, 1);
        MmSetPageEntrySectionSegment(Segment, &Offset, Entry);
        MmUnlockSectionSegment(Segment);

        DPRINT("Address 0x%p\n", Address);
        return STATUS_SUCCESS;
    }
    else
    {
        /* We already have a page on this section offset. Map it into the process address space. */
        Page = PFN_FROM_SSE(Entry);

        Status = MmCreateVirtualMapping(Process,
                                        PAddress,
                                        Attributes,
                                        Page);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("Unable to create virtual mapping\n");
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        if (Process)
            MmInsertRmap(Page, Process, Address);

        /* Take a reference on it */
        MmSharePageEntrySectionSegment(Segment, &Offset);
        MmUnlockSectionSegment(Segment);

        DPRINT("Address 0x%p\n", Address);
        return STATUS_SUCCESS;
    }
}

NTSTATUS
NTAPI
MmAccessFaultSectionView(PMMSUPPORT AddressSpace,
                         MEMORY_AREA* MemoryArea,
                         PVOID Address,
                         BOOLEAN Locked)
{
    PMM_SECTION_SEGMENT Segment;
    PFN_NUMBER OldPage;
    PFN_NUMBER NewPage;
    PFN_NUMBER UnmappedPage;
    PVOID PAddress;
    LARGE_INTEGER Offset;
    PMM_REGION Region;
    ULONG_PTR Entry;
    PEPROCESS Process = MmGetAddressSpaceOwner(AddressSpace);
    BOOLEAN Cow = FALSE;
    ULONG NewProtect;
    BOOLEAN Unmapped;
    NTSTATUS Status;

    DPRINT("MmAccessFaultSectionView(%p, %p, %p)\n", AddressSpace, MemoryArea, Address);

    /* Get the region for this address */
    Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
                        &MemoryArea->SectionData.RegionListHead,
                        Address, NULL);
    ASSERT(Region != NULL);
    if (!(Region->Protect & PAGE_IS_WRITABLE))
        return STATUS_ACCESS_VIOLATION;

    /* Make sure we have a page mapping for this address.  */
    if (!MmIsPagePresent(Process, Address))
    {
        NTSTATUS Status = MmNotPresentFaultSectionView(AddressSpace, MemoryArea, Address, Locked);
        if (!NT_SUCCESS(Status))
        {
            /* This is invalid access ! */
            return Status;
        }
    }

    /*
     * Check if the page has already been set readwrite
     */
    if (MmGetPageProtect(Process, Address) & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE))
    {
        DPRINT("Address 0x%p\n", Address);
        return STATUS_SUCCESS;
    }

    /* Check if we are doing Copy-On-Write */
    Segment = MemoryArea->SectionData.Segment;
    Cow = Segment->WriteCopy || (Region->Protect & PAGE_IS_WRITECOPY);

    if (!Cow)
    {
        /* Simply update page protection and we're done */
        MmSetPageProtect(Process, Address, Region->Protect);
        return STATUS_SUCCESS;
    }

    /* Calculate the new protection & check if we should update the region */
    NewProtect = Region->Protect;
    if (NewProtect & PAGE_IS_WRITECOPY)
    {
        NewProtect &= ~PAGE_IS_WRITECOPY;
        if (Region->Protect & PAGE_IS_EXECUTABLE)
            NewProtect |= PAGE_EXECUTE_READWRITE;
        else
            NewProtect |= PAGE_READWRITE;
        MmAlterRegion(AddressSpace, (PVOID)MA_GetStartingAddress(MemoryArea),
                &MemoryArea->SectionData.RegionListHead,
                Address, PAGE_SIZE, Region->Type, NewProtect,
                MmAlterViewAttributes);
    }

    /*
     * Find the offset of the page
     */
    PAddress = MM_ROUND_DOWN(Address, PAGE_SIZE);
    Offset.QuadPart = (ULONG_PTR)PAddress - MA_GetStartingAddress(MemoryArea)
                      + MemoryArea->SectionData.ViewOffset;

    /* Get the page mapping this section offset. */
    MmLockSectionSegment(Segment);
    Entry = MmGetPageEntrySectionSegment(Segment, &Offset);

    /* Get the current page mapping for the process */
    ASSERT(MmIsPagePresent(Process, PAddress));
    OldPage = MmGetPfnForProcess(Process, PAddress);
    ASSERT(OldPage != 0);

    if (IS_SWAP_FROM_SSE(Entry) ||
            PFN_FROM_SSE(Entry) != OldPage)
    {
        MmUnlockSectionSegment(Segment);
        /* This is a private page. We must only change the page protection. */
        MmSetPageProtect(Process, PAddress, NewProtect);
        return STATUS_SUCCESS;
    }

    /*
     * Allocate a page
     */
    Status = MmRequestPageMemoryConsumer(MC_USER, TRUE, &NewPage);
    if (!NT_SUCCESS(Status))
    {
        MmUnlockSectionSegment(Segment);
        return STATUS_NO_MEMORY;
    }

    /*
     * Copy the old page
     */
    NT_VERIFY(NT_SUCCESS(MiCopyFromUserPage(NewPage, PAddress)));

    /*
     * Unshare the old page.
     */
    DPRINT("Swapping page (Old %x New %x)\n", OldPage, NewPage);
    Unmapped = MmDeleteVirtualMapping(Process, PAddress, NULL, &UnmappedPage);
    if (!Unmapped || (UnmappedPage != OldPage))
    {
        /* Uh , we had a page just before, but suddenly it changes. Someone corrupted us. */
        KeBugCheckEx(MEMORY_MANAGEMENT,
                    (ULONG_PTR)Process,
                    (ULONG_PTR)PAddress,
                    (ULONG_PTR)__FILE__,
                    __LINE__);
    }

    if (Process)
        MmDeleteRmap(OldPage, Process, PAddress);
    MmUnsharePageEntrySectionSegment(MemoryArea, Segment, &Offset, FALSE, FALSE, NULL);
    MmUnlockSectionSegment(Segment);

    /*
     * Set the PTE to point to the new page
     */
    if (!NT_SUCCESS(MmCreateVirtualMapping(Process, PAddress, NewProtect, NewPage)))
    {
        DPRINT1("MmCreateVirtualMapping failed, unable to create virtual mapping, not out of memory\n");
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if (Process)
        MmInsertRmap(NewPage, Process, PAddress);

    DPRINT("Address 0x%p\n", Address);
    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
MmProtectSectionView(PMMSUPPORT AddressSpace,
                     PMEMORY_AREA MemoryArea,
                     PVOID BaseAddress,
                     SIZE_T Length,
                     ULONG Protect,
                     PULONG OldProtect)
{
    PMM_REGION Region;
    NTSTATUS Status;
    ULONG_PTR MaxLength;

    ASSERT(MemoryArea->Type == MEMORY_AREA_SECTION_VIEW);

    if (MemoryArea->DeleteInProgress)
    {
        return STATUS_UNSUCCESSFUL;
    }

    MaxLength = MA_GetEndingAddress(MemoryArea) - (ULONG_PTR)BaseAddress;
    if (Length > MaxLength)
        Length = (ULONG)MaxLength;

    Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
                          &MemoryArea->SectionData.RegionListHead,
                          BaseAddress, NULL);
    ASSERT(Region != NULL);

    if ((MemoryArea->Flags & SEC_NO_CHANGE) &&
            Region->Protect != Protect)
    {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    if (OldProtect != NULL)
        *OldProtect = Region->Protect;
    Status = MmAlterRegion(AddressSpace, (PVOID)MA_GetStartingAddress(MemoryArea),
                           &MemoryArea->SectionData.RegionListHead,
                           BaseAddress, Length, Region->Type, Protect,
                           MmAlterViewAttributes);

    return Status;
}

NTSTATUS NTAPI
MmQuerySectionView(PMEMORY_AREA MemoryArea,
                   PVOID Address,
                   PMEMORY_BASIC_INFORMATION Info,
                   PSIZE_T ResultLength)
{
    PMM_REGION Region;
    PVOID RegionBaseAddress;
    PMM_SECTION_SEGMENT Segment;

    Region = MmFindRegion((PVOID)MA_GetStartingAddress(MemoryArea),
                          &MemoryArea->SectionData.RegionListHead,
                          Address, &RegionBaseAddress);
    if (Region == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    if (MemoryArea->VadNode.u.VadFlags.VadType == VadImageMap)
    {
        Segment = MemoryArea->SectionData.Segment;
        Info->AllocationBase = (PUCHAR)MA_GetStartingAddress(MemoryArea) - Segment->Image.VirtualAddress;
        Info->Type = MEM_IMAGE;
    }
    else
    {
        Info->AllocationBase = (PVOID)MA_GetStartingAddress(MemoryArea);
        Info->Type = MEM_MAPPED;
    }
    Info->BaseAddress = RegionBaseAddress;
    Info->AllocationProtect = MmProtectToValue[MemoryArea->VadNode.u.VadFlags.Protection];
    Info->RegionSize = Region->Length;
    Info->State = MEM_COMMIT;
    Info->Protect = Region->Protect;

    *ResultLength = sizeof(MEMORY_BASIC_INFORMATION);
    return STATUS_SUCCESS;
}

static
NTSTATUS
NTAPI
MmCreateDataFileSection(PSECTION *SectionObject,
                        ACCESS_MASK DesiredAccess,
                        POBJECT_ATTRIBUTES ObjectAttributes,
                        PLARGE_INTEGER UMaximumSize,
                        ULONG SectionPageProtection,
                        ULONG AllocationAttributes,
                        PFILE_OBJECT FileObject,
                        BOOLEAN GotFileHandle)
/*
 * Create a section backed by a data file
 */
{
    PSECTION Section;
    NTSTATUS Status;
    LARGE_INTEGER MaximumSize;
    PMM_SECTION_SEGMENT Segment;
    KIRQL OldIrql;

    /*
     * Create the section
     */
    Status = ObCreateObject(ExGetPreviousMode(),
                            MmSectionObjectType,
                            ObjectAttributes,
                            ExGetPreviousMode(),
                            NULL,
                            sizeof(*Section),
                            0,
                            0,
                            (PVOID*)&Section);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }
    /*
     * Initialize it
     */
    RtlZeroMemory(Section, sizeof(*Section));

    /* Mark this as a "ROS" section */
    Section->u.Flags.filler = 1;
    Section->InitialPageProtection = SectionPageProtection;
    Section->u.Flags.File = 1;

    if (AllocationAttributes & SEC_NO_CHANGE)
        Section->u.Flags.NoChange = 1;

    if (!GotFileHandle)
    {
        ASSERT(UMaximumSize != NULL);
        // ASSERT(UMaximumSize->QuadPart != 0);
        MaximumSize = *UMaximumSize;
    }
    else
    {
        LARGE_INTEGER FileSize;
        Status = FsRtlGetFileSize(FileObject, &FileSize);
        if (!NT_SUCCESS(Status))
        {
            ObDereferenceObject(Section);
            return Status;
        }

        /*
         * FIXME: Revise this once a locking order for file size changes is
         * decided
         */
        if ((UMaximumSize != NULL) && (UMaximumSize->QuadPart != 0))
        {
            MaximumSize = *UMaximumSize;
        }
        else
        {
            MaximumSize = FileSize;
            /* Mapping zero-sized files isn't allowed. */
            if (MaximumSize.QuadPart == 0)
            {
                ObDereferenceObject(Section);
                return STATUS_MAPPED_FILE_SIZE_ZERO;
            }
        }

        if (MaximumSize.QuadPart > FileSize.QuadPart)
        {
            Status = IoSetInformation(FileObject,
                                      FileEndOfFileInformation,
                                      sizeof(LARGE_INTEGER),
                                      &MaximumSize);
            if (!NT_SUCCESS(Status))
            {
                ObDereferenceObject(Section);
                return STATUS_SECTION_NOT_EXTENDED;
            }
        }
    }

    if (FileObject->SectionObjectPointer == NULL)
    {
        ObDereferenceObject(Section);
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    /*
     * Lock the file
     */
    Status = MmspWaitForFileLock(FileObject);
    if (Status != STATUS_SUCCESS)
    {
        ObDereferenceObject(Section);
        return Status;
    }

    /* Lock the PFN lock while messing with Section Object pointers */
grab_segment:
    OldIrql = MiAcquirePfnLock();
    Segment = FileObject->SectionObjectPointer->DataSectionObject;

    while (Segment && (Segment->SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)))
    {
        MiReleasePfnLock(OldIrql);
        KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);
        OldIrql = MiAcquirePfnLock();
        Segment = FileObject->SectionObjectPointer->DataSectionObject;
    }

    /*
     * If this file hasn't been mapped as a data file before then allocate a
     * section segment to describe the data file mapping
     */
    if (Segment == NULL)
    {
        /* Release the lock. ExAllocatePoolWithTag might acquire it */
        MiReleasePfnLock(OldIrql);

        Segment = ExAllocatePoolWithTag(NonPagedPool, sizeof(MM_SECTION_SEGMENT),
                                        TAG_MM_SECTION_SEGMENT);
        if (Segment == NULL)
        {
            //KeSetEvent((PVOID)&FileObject->Lock, IO_NO_INCREMENT, FALSE);
            ObDereferenceObject(Section);
            return STATUS_NO_MEMORY;
        }

        /* We are creating it */
        RtlZeroMemory(Segment, sizeof(*Segment));
        Segment->SegFlags = MM_DATAFILE_SEGMENT | MM_SEGMENT_INCREATE;
        Segment->RefCount = 1;

        /* Acquire lock again */
        OldIrql = MiAcquirePfnLock();

        if (FileObject->SectionObjectPointer->DataSectionObject != NULL)
        {
            /* Well that's bad luck. Restart it all over */
            MiReleasePfnLock(OldIrql);
            ExFreePoolWithTag(Segment, TAG_MM_SECTION_SEGMENT);
            goto grab_segment;
        }

        FileObject->SectionObjectPointer->DataSectionObject = Segment;

        /* We're safe to release the lock now */
        MiReleasePfnLock(OldIrql);

        Section->Segment = (PSEGMENT)Segment;

        /* Self-referencing segment */
        Segment->Flags = &Segment->SegFlags;
        Segment->ReferenceCount = &Segment->RefCount;

        Segment->SectionCount = 1;

        ExInitializeFastMutex(&Segment->Lock);
        Segment->FileObject = FileObject;
        ObReferenceObject(FileObject);

        Segment->Image.FileOffset = 0;
        Segment->Protection = SectionPageProtection;

        Segment->Image.Characteristics = 0;
        Segment->WriteCopy = (SectionPageProtection & (PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY));
        if (AllocationAttributes & SEC_RESERVE)
        {
            Segment->Length.QuadPart = Segment->RawLength.QuadPart = 0;
        }
        else
        {
            Segment->RawLength.QuadPart = MaximumSize.QuadPart;
            Segment->Length.QuadPart = PAGE_ROUND_UP(Segment->RawLength.QuadPart);
        }
        Segment->Image.VirtualAddress = 0;
        MiInitializeSectionPageTable(Segment);

        /* We're good to use it now */
        OldIrql = MiAcquirePfnLock();
        Segment->SegFlags &= ~MM_SEGMENT_INCREATE;
        MiReleasePfnLock(OldIrql);
    }
    else
    {
        Section->Segment = (PSEGMENT)Segment;
        InterlockedIncrement64(&Segment->RefCount);
        InterlockedIncrementUL(&Segment->SectionCount);

        MiReleasePfnLock(OldIrql);

        MmLockSectionSegment(Segment);

        if (MaximumSize.QuadPart > Segment->RawLength.QuadPart &&
                !(AllocationAttributes & SEC_RESERVE))
        {
            Segment->RawLength.QuadPart = MaximumSize.QuadPart;
            Segment->Length.QuadPart = PAGE_ROUND_UP(Segment->RawLength.QuadPart);
        }

        MmUnlockSectionSegment(Segment);
    }
    Section->SizeOfSection = MaximumSize;

    //KeSetEvent((PVOID)&FileObject->Lock, IO_NO_INCREMENT, FALSE);
    *SectionObject = Section;
    return STATUS_SUCCESS;
}

/*
 TODO: not that great (declaring loaders statically, having to declare all of
 them, having to keep them extern, etc.), will fix in the future
*/
extern NTSTATUS NTAPI PeFmtCreateSection
(
    IN CONST VOID * FileHeader,
    IN SIZE_T FileHeaderSize,
    IN PVOID File,
    OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    OUT PULONG Flags,
    IN PEXEFMT_CB_READ_FILE ReadFileCb,
    IN PEXEFMT_CB_ALLOCATE_SEGMENTS AllocateSegmentsCb
);

extern NTSTATUS NTAPI ElfFmtCreateSection
(
    IN CONST VOID * FileHeader,
    IN SIZE_T FileHeaderSize,
    IN PVOID File,
    OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    OUT PULONG Flags,
    IN PEXEFMT_CB_READ_FILE ReadFileCb,
    IN PEXEFMT_CB_ALLOCATE_SEGMENTS AllocateSegmentsCb
);

static PEXEFMT_LOADER ExeFmtpLoaders[] =
{
    PeFmtCreateSection,
#ifdef __ELF
    ElfFmtCreateSection
#endif
};

static
PMM_SECTION_SEGMENT
NTAPI
ExeFmtpAllocateSegments(IN ULONG NrSegments)
{
    SIZE_T SizeOfSegments;
    PMM_SECTION_SEGMENT Segments;

    /* TODO: check for integer overflow */
    SizeOfSegments = sizeof(MM_SECTION_SEGMENT) * NrSegments;

    Segments = ExAllocatePoolWithTag(NonPagedPool,
                                     SizeOfSegments,
                                     TAG_MM_SECTION_SEGMENT);

    if(Segments)
        RtlZeroMemory(Segments, SizeOfSegments);

    return Segments;
}
static
NTSTATUS
NTAPI
ExeFmtpReadFile(IN PVOID File,
                IN PLARGE_INTEGER Offset,
                IN ULONG Length,
                OUT PVOID * Data,
                OUT PVOID * AllocBase,
                OUT PULONG ReadSize)
{
    NTSTATUS Status;
    LARGE_INTEGER FileOffset;
    ULONG AdjustOffset;
    ULONG OffsetAdjustment;
    ULONG BufferSize;
    ULONG UsedSize;
    PVOID Buffer;
    PFILE_OBJECT FileObject = File;
    IO_STATUS_BLOCK Iosb;

    ASSERT_IRQL_LESS(DISPATCH_LEVEL);

    if(Length == 0)
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    FileOffset = *Offset;

    /* Negative/special offset: it cannot be used in this context */
    if(FileOffset.u.HighPart < 0)
    {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    AdjustOffset = PAGE_ROUND_DOWN(FileOffset.u.LowPart);
    OffsetAdjustment = FileOffset.u.LowPart - AdjustOffset;
    FileOffset.u.LowPart = AdjustOffset;

    BufferSize = Length + OffsetAdjustment;
    BufferSize = PAGE_ROUND_UP(BufferSize);

    /*
     * It's ok to use paged pool, because this is a temporary buffer only used in
     * the loading of executables. The assumption is that MmCreateSection is
     * always called at low IRQLs and that these buffers don't survive a brief
     * initialization phase
     */
    Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize, 'rXmM');
    if (!Buffer)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ARM64_UART_TRACE("[MM] ExeFmtpReadFile: MiSimpleRead\r\n");
    Status = MiSimpleRead(FileObject, &FileOffset, Buffer, BufferSize, TRUE, &Iosb);
    ARM64_UART_TRACE("[MM] ExeFmtpReadFile: MiSimpleRead done\r\n");

    UsedSize = (ULONG)Iosb.Information;

    if(NT_SUCCESS(Status) && UsedSize < OffsetAdjustment)
    {
        Status = STATUS_IN_PAGE_ERROR;
        ASSERT(!NT_SUCCESS(Status));
    }

    if(NT_SUCCESS(Status))
    {
        *Data = (PVOID)((ULONG_PTR)Buffer + OffsetAdjustment);
        *AllocBase = Buffer;
        *ReadSize = UsedSize - OffsetAdjustment;
    }
    else
    {
        ExFreePoolWithTag(Buffer, 'rXmM');
    }

    return Status;
}

#ifdef NASSERT
# define MmspAssertSegmentsSorted(OBJ_) ((void)0)
# define MmspAssertSegmentsNoOverlap(OBJ_) ((void)0)
# define MmspAssertSegmentsPageAligned(OBJ_) ((void)0)
#else
static
VOID
NTAPI
MmspAssertSegmentsSorted(IN PMM_IMAGE_SECTION_OBJECT ImageSectionObject)
{
    ULONG i;

    for( i = 1; i < ImageSectionObject->NrSegments; ++ i )
    {
        ASSERT(ImageSectionObject->Segments[i].Image.VirtualAddress >=
               ImageSectionObject->Segments[i - 1].Image.VirtualAddress);
    }
}

static
VOID
NTAPI
MmspAssertSegmentsNoOverlap(IN PMM_IMAGE_SECTION_OBJECT ImageSectionObject)
{
    ULONG i;

    MmspAssertSegmentsSorted(ImageSectionObject);

    for( i = 0; i < ImageSectionObject->NrSegments; ++ i )
    {
        ASSERT(ImageSectionObject->Segments[i].Length.QuadPart > 0);

        if(i > 0)
        {
            ASSERT(ImageSectionObject->Segments[i].Image.VirtualAddress >=
                   (ImageSectionObject->Segments[i - 1].Image.VirtualAddress +
                    ImageSectionObject->Segments[i - 1].Length.QuadPart));
        }
    }
}

static
VOID
NTAPI
MmspAssertSegmentsPageAligned(IN PMM_IMAGE_SECTION_OBJECT ImageSectionObject)
{
    ULONG i;

    for( i = 0; i < ImageSectionObject->NrSegments; ++ i )
    {
        ASSERT((ImageSectionObject->Segments[i].Image.VirtualAddress % PAGE_SIZE) == 0);
        ASSERT((ImageSectionObject->Segments[i].Length.QuadPart % PAGE_SIZE) == 0);
    }
}
#endif

static
int
__cdecl
MmspCompareSegments(const void * x,
                    const void * y)
{
    const MM_SECTION_SEGMENT *Segment1 = (const MM_SECTION_SEGMENT *)x;
    const MM_SECTION_SEGMENT *Segment2 = (const MM_SECTION_SEGMENT *)y;

    if (Segment1->Image.VirtualAddress > Segment2->Image.VirtualAddress)
        return 1;
    else if (Segment1->Image.VirtualAddress < Segment2->Image.VirtualAddress)
        return -1;
    else
        return 0;
}

/*
 * Ensures an image section's segments are sorted in memory
 */
static
VOID
NTAPI
MmspSortSegments(IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
                 IN ULONG Flags)
{
    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED)
    {
        MmspAssertSegmentsSorted(ImageSectionObject);
    }
    else
    {
        qsort(ImageSectionObject->Segments,
              ImageSectionObject->NrSegments,
              sizeof(ImageSectionObject->Segments[0]),
              MmspCompareSegments);
    }
}


/*
 * Ensures an image section's segments don't overlap in memory and don't have
 * gaps and don't have a null size. We let them map to overlapping file regions,
 * though - that's not necessarily an error
 */
static
BOOLEAN
NTAPI
MmspCheckSegmentBounds
(
    IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    IN ULONG Flags
)
{
    ULONG i;

    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP)
    {
        MmspAssertSegmentsNoOverlap(ImageSectionObject);
        return TRUE;
    }

    ASSERT(ImageSectionObject->NrSegments >= 1);

    for ( i = 0; i < ImageSectionObject->NrSegments; ++ i )
    {
        if(ImageSectionObject->Segments[i].Length.QuadPart == 0)
        {
            return FALSE;
        }

        if(i > 0)
        {
            /*
             * TODO: relax the limitation on gaps. For example, gaps smaller than a
             * page could be OK (Windows seems to be OK with them), and larger gaps
             * could lead to image sections spanning several discontiguous regions
             * (NtMapViewOfSection could then refuse to map them, and they could
             * e.g. only be allowed as parameters to NtCreateProcess, like on UNIX)
             */
            if ((ImageSectionObject->Segments[i - 1].Image.VirtualAddress +
                    ImageSectionObject->Segments[i - 1].Length.QuadPart) !=
                    ImageSectionObject->Segments[i].Image.VirtualAddress)
            {
                return FALSE;
            }
        }
    }

    return TRUE;
}

/*
 * Merges and pads an image section's segments until they all are page-aligned
 * and have a size that is a multiple of the page size
 */
static
BOOLEAN
NTAPI
MmspPageAlignSegments
(
    IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    IN ULONG Flags
)
{
    ULONG i;
    ULONG LastSegment;
    PMM_SECTION_SEGMENT EffectiveSegment;

    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED)
    {
        MmspAssertSegmentsPageAligned(ImageSectionObject);
        return TRUE;
    }

    LastSegment = 0;
    EffectiveSegment = &ImageSectionObject->Segments[LastSegment];

    for ( i = 0; i < ImageSectionObject->NrSegments; ++ i )
    {
        /*
         * The first segment requires special handling
         */
        if (i == 0)
        {
            ULONG_PTR VirtualAddress;
            ULONG_PTR VirtualOffset;

            VirtualAddress = EffectiveSegment->Image.VirtualAddress;

            /* Round down the virtual address to the nearest page */
            EffectiveSegment->Image.VirtualAddress = PAGE_ROUND_DOWN(VirtualAddress);

            /* Round up the virtual size to the nearest page */
            EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(VirtualAddress + EffectiveSegment->Length.QuadPart) -
                                                EffectiveSegment->Image.VirtualAddress;

            /* Adjust the raw address and size */
            VirtualOffset = VirtualAddress - EffectiveSegment->Image.VirtualAddress;

            if (EffectiveSegment->Image.FileOffset < VirtualOffset)
            {
                return FALSE;
            }

            /*
             * Garbage in, garbage out: unaligned base addresses make the file
             * offset point in curious and odd places, but that's what we were
             * asked for
             */
            EffectiveSegment->Image.FileOffset -= VirtualOffset;
            EffectiveSegment->RawLength.QuadPart += VirtualOffset;
        }
        else
        {
            PMM_SECTION_SEGMENT Segment = &ImageSectionObject->Segments[i];
            ULONG_PTR EndOfEffectiveSegment;

            EndOfEffectiveSegment = (ULONG_PTR)(EffectiveSegment->Image.VirtualAddress + EffectiveSegment->Length.QuadPart);
            ASSERT((EndOfEffectiveSegment % PAGE_SIZE) == 0);

            /*
             * The current segment begins exactly where the current effective
             * segment ended, therefore beginning a new effective segment
             */
            if (EndOfEffectiveSegment == Segment->Image.VirtualAddress)
            {
                LastSegment ++;
                ASSERT(LastSegment <= i);
                ASSERT(LastSegment < ImageSectionObject->NrSegments);

                EffectiveSegment = &ImageSectionObject->Segments[LastSegment];

                if (LastSegment != i)
                {
                    /*
                     * Copy the current segment. If necessary, the effective segment
                     * will be expanded later
                     */
                    *EffectiveSegment = *Segment;
                }

                /*
                 * Page-align the virtual size. We know for sure the virtual address
                 * already is
                 */
                ASSERT((EffectiveSegment->Image.VirtualAddress % PAGE_SIZE) == 0);
                EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(EffectiveSegment->Length.QuadPart);
            }
            /*
             * The current segment is still part of the current effective segment:
             * extend the effective segment to reflect this
             */
            else if (EndOfEffectiveSegment > Segment->Image.VirtualAddress)
            {
                static const ULONG FlagsToProtection[16] =
                {
                    PAGE_NOACCESS,
                    PAGE_READONLY,
                    PAGE_READWRITE,
                    PAGE_READWRITE,
                    PAGE_EXECUTE_READ,
                    PAGE_EXECUTE_READ,
                    PAGE_EXECUTE_READWRITE,
                    PAGE_EXECUTE_READWRITE,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY
                };

                unsigned ProtectionFlags;

                /*
                 * Extend the file size
                 */

                /* Unaligned segments must be contiguous within the file */
                if (Segment->Image.FileOffset != (EffectiveSegment->Image.FileOffset +
                                                  EffectiveSegment->RawLength.QuadPart))
                {
                    return FALSE;
                }

                EffectiveSegment->RawLength.QuadPart += Segment->RawLength.QuadPart;

                /*
                 * Extend the virtual size
                 */
                ASSERT(PAGE_ROUND_UP(Segment->Image.VirtualAddress + Segment->Length.QuadPart) >= EndOfEffectiveSegment);

                EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(Segment->Image.VirtualAddress + Segment->Length.QuadPart) -
                                                    EffectiveSegment->Image.VirtualAddress;

                /*
                 * Merge the protection
                 */
                EffectiveSegment->Protection |= Segment->Protection;

                /* Clean up redundance */
                ProtectionFlags = 0;

                if(EffectiveSegment->Protection & PAGE_IS_READABLE)
                    ProtectionFlags |= 1 << 0;

                if(EffectiveSegment->Protection & PAGE_IS_WRITABLE)
                    ProtectionFlags |= 1 << 1;

                if(EffectiveSegment->Protection & PAGE_IS_EXECUTABLE)
                    ProtectionFlags |= 1 << 2;

                if(EffectiveSegment->Protection & PAGE_IS_WRITECOPY)
                    ProtectionFlags |= 1 << 3;

                ASSERT(ProtectionFlags < 16);
                EffectiveSegment->Protection = FlagsToProtection[ProtectionFlags];

                /* If a segment was required to be shared and cannot, fail */
                if(!(Segment->Protection & PAGE_IS_WRITECOPY) &&
                        EffectiveSegment->Protection & PAGE_IS_WRITECOPY)
                {
                    return FALSE;
                }
            }
            /*
             * We assume no holes between segments at this point
             */
            else
            {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }
    }
    ImageSectionObject->NrSegments = LastSegment + 1;

    return TRUE;
}

NTSTATUS
ExeFmtpCreateImageSection(PFILE_OBJECT FileObject,
                          PMM_IMAGE_SECTION_OBJECT ImageSectionObject)
{
    LARGE_INTEGER Offset;
    PVOID FileHeader;
    PVOID FileHeaderBuffer;
    ULONG FileHeaderSize;
    ULONG Flags;
    ULONG OldNrSegments;
    NTSTATUS Status;
    ULONG i;

    /*
     * Read the beginning of the file (2 pages). Should be enough to contain
     * all (or most) of the headers
     */
    Offset.QuadPart = 0;

    Status = ExeFmtpReadFile (FileObject,
                              &Offset,
                              PAGE_SIZE * 2,
                              &FileHeader,
                              &FileHeaderBuffer,
                              &FileHeaderSize);

    if (!NT_SUCCESS(Status))
        return Status;

    if (FileHeaderSize == 0)
    {
        ExFreePool(FileHeaderBuffer);
        return STATUS_UNSUCCESSFUL;
    }

    /*
     * Look for a loader that can handle this executable
     */
    for (i = 0; i < RTL_NUMBER_OF(ExeFmtpLoaders); ++ i)
    {
        Flags = 0;

        Status = ExeFmtpLoaders[i](FileHeader,
                                   FileHeaderSize,
                                   FileObject,
                                   ImageSectionObject,
                                   &Flags,
                                   ExeFmtpReadFile,
                                   ExeFmtpAllocateSegments);

        if (!NT_SUCCESS(Status))
        {
            if (ImageSectionObject->Segments)
            {
                ExFreePool(ImageSectionObject->Segments);
                ImageSectionObject->Segments = NULL;
            }
        }

        if (Status != STATUS_ROS_EXEFMT_UNKNOWN_FORMAT)
            break;
    }

    ExFreePoolWithTag(FileHeaderBuffer, 'rXmM');

    /*
     * No loader handled the format
     */
    if (Status == STATUS_ROS_EXEFMT_UNKNOWN_FORMAT)
    {
        Status = STATUS_INVALID_IMAGE_NOT_MZ;
        ASSERT(!NT_SUCCESS(Status));
    }

    if (!NT_SUCCESS(Status))
        return Status;

    ASSERT(ImageSectionObject->Segments != NULL);
    ASSERT(ImageSectionObject->RefCount > 0);

    /*
     * Some defaults
     */
    /* FIXME? are these values platform-dependent? */
    if (ImageSectionObject->ImageInformation.MaximumStackSize == 0)
        ImageSectionObject->ImageInformation.MaximumStackSize = 0x40000;

    if(ImageSectionObject->ImageInformation.CommittedStackSize == 0)
        ImageSectionObject->ImageInformation.CommittedStackSize = 0x1000;

    if(ImageSectionObject->BasedAddress == NULL)
    {
        if(ImageSectionObject->ImageInformation.ImageCharacteristics & IMAGE_FILE_DLL)
            ImageSectionObject->BasedAddress = (PVOID)0x10000000;
        else
            ImageSectionObject->BasedAddress = (PVOID)0x00400000;
    }

    /*
     * And now the fun part: fixing the segments
     */

    /* Sort them by virtual address */
    MmspSortSegments(ImageSectionObject, Flags);

    /* Ensure they don't overlap in memory */
    if (!MmspCheckSegmentBounds(ImageSectionObject, Flags))
        return STATUS_INVALID_IMAGE_FORMAT;

    /* Ensure they are aligned */
    OldNrSegments = ImageSectionObject->NrSegments;

    if (!MmspPageAlignSegments(ImageSectionObject, Flags))
        return STATUS_INVALID_IMAGE_FORMAT;

    /* Trim them if the alignment phase merged some of them */
    if (ImageSectionObject->NrSegments < OldNrSegments)
    {
        PMM_SECTION_SEGMENT Segments;
        SIZE_T SizeOfSegments;

        SizeOfSegments = sizeof(MM_SECTION_SEGMENT) * ImageSectionObject->NrSegments;

        Segments = ExAllocatePoolWithTag(PagedPool,
                                         SizeOfSegments,
                                         TAG_MM_SECTION_SEGMENT);

        if (Segments == NULL)
            return STATUS_INSUFFICIENT_RESOURCES;

        RtlCopyMemory(Segments, ImageSectionObject->Segments, SizeOfSegments);
        ExFreePool(ImageSectionObject->Segments);
        ImageSectionObject->Segments = Segments;
    }

    /* And finish their initialization */
    for ( i = 0; i < ImageSectionObject->NrSegments; ++ i )
    {
        ExInitializeFastMutex(&ImageSectionObject->Segments[i].Lock);
        ImageSectionObject->Segments[i].ReferenceCount = &ImageSectionObject->RefCount;
        ImageSectionObject->Segments[i].Flags = &ImageSectionObject->SegFlags;
        MiInitializeSectionPageTable(&ImageSectionObject->Segments[i]);
        ImageSectionObject->Segments[i].FileObject = FileObject;
    }

    ASSERT(ImageSectionObject->RefCount > 0);

    ImageSectionObject->FileObject = FileObject;

    ASSERT(NT_SUCCESS(Status));
    return Status;
}

NTSTATUS
MmCreateImageSection(PSECTION *SectionObject,
                     ACCESS_MASK DesiredAccess,
                     POBJECT_ATTRIBUTES ObjectAttributes,
                     PLARGE_INTEGER UMaximumSize,
                     ULONG SectionPageProtection,
                     ULONG AllocationAttributes,
                     PFILE_OBJECT FileObject)
{
    PSECTION Section;
    NTSTATUS Status;
    PMM_IMAGE_SECTION_OBJECT ImageSectionObject;
    KIRQL OldIrql;


    if (FileObject == NULL)
        return STATUS_INVALID_FILE_FOR_SECTION;

    if (FileObject->SectionObjectPointer == NULL)
    {
        DPRINT1("Denying section creation due to missing cache initialization\n");
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    /*
     * Create the section
     */
    Status = ObCreateObject (ExGetPreviousMode(),
                             MmSectionObjectType,
                             ObjectAttributes,
                             ExGetPreviousMode(),
                             NULL,
                             sizeof(*Section),
                             0,
                             0,
                             (PVOID*)(PVOID)&Section);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    /*
     * Initialize it
     */
    RtlZeroMemory(Section, sizeof(*Section));

    /* Mark this as a "ROS" Section */
    Section->u.Flags.filler = 1;

    Section->InitialPageProtection = SectionPageProtection;
    Section->SizeOfSection = *UMaximumSize;
    Section->u.Flags.File = 1;
    Section->u.Flags.Image = 1;
    if (AllocationAttributes & SEC_NO_CHANGE)
        Section->u.Flags.NoChange = 1;

grab_image_section_object:
    OldIrql = MiAcquirePfnLock();

    /* Wait for it to be properly created or deleted */
    ImageSectionObject = FileObject->SectionObjectPointer->ImageSectionObject;
    while(ImageSectionObject && (ImageSectionObject->SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)))
    {
        MiReleasePfnLock(OldIrql);

        KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);

        OldIrql = MiAcquirePfnLock();
        ImageSectionObject = FileObject->SectionObjectPointer->ImageSectionObject;
    }

    if (ImageSectionObject == NULL)
    {
        NTSTATUS StatusExeFmt;

        /* Release the lock because ExAllocatePoolWithTag could need to acquire it */
        MiReleasePfnLock(OldIrql);

        ImageSectionObject = ExAllocatePoolZero(NonPagedPool, sizeof(MM_IMAGE_SECTION_OBJECT), TAG_MM_SECTION_SEGMENT);
        if (ImageSectionObject == NULL)
        {
            ObDereferenceObject(Section);
            return STATUS_NO_MEMORY;
        }

        ImageSectionObject->SegFlags = MM_SEGMENT_INCREATE;
        ImageSectionObject->RefCount = 1;
        ImageSectionObject->SectionCount = 1;

        OldIrql = MiAcquirePfnLock();
        if (FileObject->SectionObjectPointer->ImageSectionObject != NULL)
        {
            MiReleasePfnLock(OldIrql);
            /* Bad luck. Start over */
            ExFreePoolWithTag(ImageSectionObject, TAG_MM_SECTION_SEGMENT);
            goto grab_image_section_object;
        }

        FileObject->SectionObjectPointer->ImageSectionObject = ImageSectionObject;

        MiReleasePfnLock(OldIrql);

        /* Purge the cache */
        ARM64_UART_TRACE("[MM] CreateImageSection: CcFlushCache\r\n");
        CcFlushCache(FileObject->SectionObjectPointer, NULL, 0, NULL);

        ARM64_UART_TRACE("[MM] CreateImageSection: ExeFmtpCreateImageSection\r\n");
        StatusExeFmt = ExeFmtpCreateImageSection(FileObject, ImageSectionObject);
        ARM64_UART_TRACE("[MM] CreateImageSection: ExeFmtpCreateImageSection done\r\n");
        ARM64_UART_TRACE_HEX("[MM]   StatusExeFmt=", (ULONG_PTR)StatusExeFmt);

        if (!NT_SUCCESS(StatusExeFmt))
        {
            /* Unset */
            OldIrql = MiAcquirePfnLock();
            FileObject->SectionObjectPointer->ImageSectionObject = NULL;
            MiReleasePfnLock(OldIrql);

            if(ImageSectionObject->Segments != NULL)
                ExFreePool(ImageSectionObject->Segments);

            /*
             * If image file is empty, then return that the file is invalid for section
             */
            Status = StatusExeFmt;
            if (StatusExeFmt == STATUS_END_OF_FILE)
            {
                Status = STATUS_INVALID_FILE_FOR_SECTION;
            }

            ExFreePoolWithTag(ImageSectionObject, TAG_MM_SECTION_SEGMENT);
            ObDereferenceObject(Section);
            return Status;
        }

        Section->Segment = (PSEGMENT)ImageSectionObject;
        ASSERT(ImageSectionObject->Segments);
        ASSERT(ImageSectionObject->RefCount > 0);

        /*
         * Lock the file
         */
        Status = MmspWaitForFileLock(FileObject);
        if (!NT_SUCCESS(Status))
        {
            /* Unset */
            OldIrql = MiAcquirePfnLock();
            FileObject->SectionObjectPointer->ImageSectionObject = NULL;
            MiReleasePfnLock(OldIrql);

            ExFreePool(ImageSectionObject->Segments);
            ExFreePool(ImageSectionObject);
            ObDereferenceObject(Section);
            return Status;
        }

        OldIrql = MiAcquirePfnLock();
        ImageSectionObject->SegFlags &= ~MM_SEGMENT_INCREATE;

        /* Take a ref on the file on behalf of the newly created structure */
        ObReferenceObject(FileObject);

        MiReleasePfnLock(OldIrql);

        Status = StatusExeFmt;
    }
    else
    {
        /* If FS driver called for delete, tell them it's not possible anymore. */
        ImageSectionObject->SegFlags &= ~MM_IMAGE_SECTION_FLUSH_DELETE;

        /* Take one ref */
        InterlockedIncrement64(&ImageSectionObject->RefCount);
        ImageSectionObject->SectionCount++;

        MiReleasePfnLock(OldIrql);

        Section->Segment = (PSEGMENT)ImageSectionObject;

        Status = STATUS_SUCCESS;
    }

    if (Section->SizeOfSection.QuadPart == 0)
    {
        Section->SizeOfSection.QuadPart = ImageSectionObject->ImageInformation.ImageFileSize;
    }

    //KeSetEvent((PVOID)&FileObject->Lock, IO_NO_INCREMENT, FALSE);
    *SectionObject = Section;
    ASSERT(ImageSectionObject->RefCount > 0);

    ARM64_UART_TRACE("[MM] CreateImageSection: returning\r\n");
    return Status;
}

static NTSTATUS
MmMapViewOfSegment(
    PMMSUPPORT AddressSpace,
    BOOLEAN AsImage,
    PMM_SECTION_SEGMENT Segment,
    PVOID* BaseAddress,
    SIZE_T ViewSize,
    ULONG Protect,
    LONGLONG ViewOffset,
    ULONG AllocationType)
{
    PMEMORY_AREA MArea;
    NTSTATUS Status;
    ULONG Granularity;

    ASSERT(ViewSize != 0);

    if (Segment->WriteCopy)
    {
        /* We have to do this because the not present fault
         * and access fault handlers depend on the protection
         * that should be granted AFTER the COW fault takes
         * place to be in Region->Protect. The not present fault
         * handler changes this to the correct protection for COW when
         * mapping the pages into the process's address space. If a COW
         * fault takes place, the access fault handler sets the page protection
         * to these values for the newly copied pages
         */
        if (Protect == PAGE_WRITECOPY)
            Protect = PAGE_READWRITE;
        else if (Protect == PAGE_EXECUTE_WRITECOPY)
            Protect = PAGE_EXECUTE_READWRITE;
    }

    if (*BaseAddress == NULL)
        Granularity = MM_ALLOCATION_GRANULARITY;
    else
        Granularity = PAGE_SIZE;

#ifdef NEWCC
    if (Segment->Flags & MM_DATAFILE_SEGMENT)
    {
        LARGE_INTEGER FileOffset;
        FileOffset.QuadPart = ViewOffset;
        ObReferenceObject(Section);
        return _MiMapViewOfSegment(AddressSpace, Segment, BaseAddress, ViewSize, Protect, &FileOffset, AllocationType, __FILE__, __LINE__);
    }
#endif
    Status = MmCreateMemoryArea(AddressSpace,
                                MEMORY_AREA_SECTION_VIEW,
                                BaseAddress,
                                ViewSize,
                                Protect,
                                &MArea,
                                AllocationType,
                                Granularity);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Mapping between 0x%p and 0x%p failed (%X).\n",
                (*BaseAddress), (char*)(*BaseAddress) + ViewSize, Status);
        return Status;
    }

    InterlockedIncrement64(Segment->ReferenceCount);

    MArea->SectionData.Segment = Segment;
    MArea->SectionData.ViewOffset = ViewOffset;
    if (AsImage)
    {
        MArea->VadNode.u.VadFlags.VadType = VadImageMap;
    }

    MmInitializeRegion(&MArea->SectionData.RegionListHead,
                       ViewSize, 0, Protect);

    return STATUS_SUCCESS;
}


static VOID
MmFreeSectionPage(PVOID Context, MEMORY_AREA* MemoryArea, PVOID Address,
                  PFN_NUMBER Page, SWAPENTRY SwapEntry, BOOLEAN Dirty)
{
    ULONG_PTR Entry;
    LARGE_INTEGER Offset;
    SWAPENTRY SavedSwapEntry;
    PMM_SECTION_SEGMENT Segment;
    PMMSUPPORT AddressSpace;
    PEPROCESS Process;

    AddressSpace = (PMMSUPPORT)Context;
    Process = MmGetAddressSpaceOwner(AddressSpace);

    Address = (PVOID)PAGE_ROUND_DOWN(Address);

    Offset.QuadPart = ((ULONG_PTR)Address - MA_GetStartingAddress(MemoryArea)) +
                      MemoryArea->SectionData.ViewOffset;

    Segment = MemoryArea->SectionData.Segment;

    Entry = MmGetPageEntrySectionSegment(Segment, &Offset);
    while (Entry && MM_IS_WAIT_PTE(Entry))
    {
        MmUnlockSectionSegment(Segment);
        MmUnlockAddressSpace(AddressSpace);

        KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);

        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment);
        Entry = MmGetPageEntrySectionSegment(Segment, &Offset);
    }

    /*
     * For a dirty, datafile, non-private page, there shoulkd be no swap entry
     */
    if (*Segment->Flags & MM_DATAFILE_SEGMENT)
    {
        if (Page == PFN_FROM_SSE(Entry) && Dirty)
        {
            ASSERT(SwapEntry == 0);
        }
    }

    if (SwapEntry != 0)
    {
        /*
         * Sanity check
         */
        MmFreeSwapPage(SwapEntry);
    }
    else if (Page != 0)
    {
        if (IS_SWAP_FROM_SSE(Entry) ||
                Page != PFN_FROM_SSE(Entry))
        {
            ASSERT(Process != NULL);

            /*
             * Just dereference private pages
             */
            SavedSwapEntry = MmGetSavedSwapEntryPage(Page);
            if (SavedSwapEntry != 0)
            {
                MmFreeSwapPage(SavedSwapEntry);
                MmSetSavedSwapEntryPage(Page, 0);
            }
            MmDeleteRmap(Page, Process, Address);
            MmReleasePageMemoryConsumer(MC_USER, Page);
        }
        else
        {
            if (Process)
            {
                MmDeleteRmap(Page, Process, Address);
            }

            /* We don't dirtify for System Space Maps. We let Cc manage that */
            MmUnsharePageEntrySectionSegment(MemoryArea, Segment, &Offset, Process ? Dirty : FALSE, FALSE, NULL);
        }
    }
}

static NTSTATUS
MmUnmapViewOfSegment(PMMSUPPORT AddressSpace,
                     PVOID BaseAddress)
{
    NTSTATUS Status;
    PMEMORY_AREA MemoryArea;
    PMM_SECTION_SEGMENT Segment;
    PLIST_ENTRY CurrentEntry;
    PMM_REGION CurrentRegion;
    PLIST_ENTRY RegionListHead;

    MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace,
                 BaseAddress);
    if (MemoryArea == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    Segment = MemoryArea->SectionData.Segment;

#ifdef NEWCC
    if (Segment->Flags & MM_DATAFILE_SEGMENT)
    {
        MmUnlockAddressSpace(AddressSpace);
        Status = MmUnmapViewOfCacheSegment(AddressSpace, BaseAddress);
        MmLockAddressSpace(AddressSpace);

        return Status;
    }
#endif

    MemoryArea->DeleteInProgress = TRUE;

    MmLockSectionSegment(Segment);

    RegionListHead = &MemoryArea->SectionData.RegionListHead;
    while (!IsListEmpty(RegionListHead))
    {
        CurrentEntry = RemoveHeadList(RegionListHead);
        CurrentRegion = CONTAINING_RECORD(CurrentEntry, MM_REGION, RegionListEntry);
        ExFreePoolWithTag(CurrentRegion, TAG_MM_REGION);
    }

    if ((*Segment->Flags) & MM_PHYSICALMEMORY_SEGMENT)
    {
        Status = MmFreeMemoryArea(AddressSpace,
                                  MemoryArea,
                                  NULL,
                                  NULL);
    }
    else
    {
        Status = MmFreeMemoryArea(AddressSpace,
                                  MemoryArea,
                                  MmFreeSectionPage,
                                  AddressSpace);
    }
    MmUnlockSectionSegment(Segment);
    MmDereferenceSegment(Segment);
    return Status;
}

/* This functions must be called with a locked address space */
NTSTATUS
NTAPI
MiRosUnmapViewOfSection(
    _In_ PEPROCESS Process,
    _In_ PMEMORY_AREA MemoryArea,
    _In_ PVOID BaseAddress,
    _In_ BOOLEAN SkipDebuggerNotify)
{
    NTSTATUS Status;
    PMMSUPPORT AddressSpace;
    PVOID ImageBaseAddress = 0;

    DPRINT("Opening memory area Process %p BaseAddress %p\n",
           Process, BaseAddress);

    ASSERT(Process);
    ASSERT(MemoryArea);

    AddressSpace = &Process->Vm;

    if (MemoryArea == NULL ||
#ifdef NEWCC
            ((MemoryArea->Type != MEMORY_AREA_SECTION_VIEW) && (MemoryArea->Type != MEMORY_AREA_CACHE)) ||
#else
            (MemoryArea->Type != MEMORY_AREA_SECTION_VIEW) ||
#endif
            MemoryArea->DeleteInProgress)

    {
        if (MemoryArea) ASSERT(MemoryArea->Type != MEMORY_AREA_OWNED_BY_ARM3);

        DPRINT1("Unable to find memory area at address %p.\n", BaseAddress);
        return STATUS_NOT_MAPPED_VIEW;
    }

    if (MemoryArea->VadNode.u.VadFlags.VadType == VadImageMap)
    {
        ULONG i;
        ULONG NrSegments;
        PMM_IMAGE_SECTION_OBJECT ImageSectionObject;
        PMM_SECTION_SEGMENT SectionSegments;
        PMM_SECTION_SEGMENT Segment;
        ULONG MapCount;

        Segment = MemoryArea->SectionData.Segment;
        ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        SectionSegments = ImageSectionObject->Segments;
        NrSegments = ImageSectionObject->NrSegments;

        MemoryArea->DeleteInProgress = TRUE;

        /* Search for the current segment within the section segments
         * and calculate the image base address */
        for (i = 0; i < NrSegments; i++)
        {
            if (Segment == &SectionSegments[i])
            {
                ImageBaseAddress = (char*)BaseAddress - (ULONG_PTR)SectionSegments[i].Image.VirtualAddress;
                break;
            }
        }
        if (i >= NrSegments)
        {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        for (i = 0; i < NrSegments; i++)
        {
            PVOID SBaseAddress = (PVOID)
                                 ((char*)ImageBaseAddress + (ULONG_PTR)SectionSegments[i].Image.VirtualAddress);

            Status = MmUnmapViewOfSegment(AddressSpace, SBaseAddress);
            if (!NT_SUCCESS(Status))
            {
                DPRINT1("MmUnmapViewOfSegment failed for %p (Process %p) with %lx\n",
                        SBaseAddress, Process, Status);
                ASSERT(NT_SUCCESS(Status));
            }
        }
        DPRINT("One mapping less for %p\n", ImageSectionObject->FileObject->SectionObjectPointer);
        MapCount = InterlockedDecrement(&ImageSectionObject->MapCount);
        if (MapCount != 0)
            ImageBaseAddress = NULL;
    }
    else
    {
        PMM_SECTION_SEGMENT Segment = MemoryArea->SectionData.Segment;
        PMMVAD Vad = &MemoryArea->VadNode;
        PCONTROL_AREA ControlArea  = Vad->ControlArea;
        PFILE_OBJECT FileObject;
        SIZE_T ViewSize;
        LARGE_INTEGER ViewOffset;
        ViewOffset.QuadPart = MemoryArea->SectionData.ViewOffset;

        InterlockedIncrement64(Segment->ReferenceCount);

        ViewSize = PAGE_SIZE + ((Vad->EndingVpn - Vad->StartingVpn) << PAGE_SHIFT);

        Status = MmUnmapViewOfSegment(AddressSpace, BaseAddress);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("MmUnmapViewOfSegment failed for %p (Process %p) with %lx\n",
                    BaseAddress, Process, Status);
            ASSERT(NT_SUCCESS(Status));
        }

        /* These might be deleted now */
        Vad = NULL;
        MemoryArea = NULL;

        if (FlagOn(*Segment->Flags, MM_PHYSICALMEMORY_SEGMENT))
        {
            /* Don't bother */
            MmDereferenceSegment(Segment);
            return STATUS_SUCCESS;
        }
        ASSERT(FlagOn(*Segment->Flags, MM_DATAFILE_SEGMENT));

        FileObject = Segment->FileObject;
        FsRtlAcquireFileExclusive(FileObject);

        /* Don't bother for auto-delete closed file. */
        if (FlagOn(FileObject->Flags, FO_DELETE_ON_CLOSE) && FlagOn(FileObject->Flags, FO_CLEANUP_COMPLETE))
        {
            FsRtlReleaseFile(FileObject);
            MmDereferenceSegment(Segment);
            return STATUS_SUCCESS;
        }

        /*
         * Flush only when last mapping is deleted.
         * FIXME: Why ControlArea == NULL? Or rather: is ControlArea ever not NULL here?
         */
        if (ControlArea == NULL || ControlArea->NumberOfMappedViews == 1)
        {
            while (ViewSize > 0)
            {
                ULONG FlushSize = min(ViewSize, PAGE_ROUND_DOWN(MAXULONG));
                MmFlushSegment(FileObject->SectionObjectPointer,
                               &ViewOffset,
                               FlushSize,
                               NULL);
                ViewSize -= FlushSize;
                ViewOffset.QuadPart += FlushSize;
            }
        }

        FsRtlReleaseFile(FileObject);
        MmDereferenceSegment(Segment);
    }

    /* Notify debugger */
    if (ImageBaseAddress && !SkipDebuggerNotify) DbgkUnMapViewOfSection(ImageBaseAddress);

    return STATUS_SUCCESS;
}

/**
 * Queries the information of a section object.
 *
 * @param SectionHandle
 *        Handle to the section object. It must be opened with SECTION_QUERY
 *        access.
 * @param SectionInformationClass
 *        Index to a certain information structure. Can be either
 *        SectionBasicInformation or SectionImageInformation. The latter
 *        is valid only for sections that were created with the SEC_IMAGE
 *        flag.
 * @param SectionInformation
 *        Caller supplies storage for resulting information.
 * @param Length
 *        Size of the supplied storage.
 * @param ResultLength
 *        Data written.
 *
 * @return Status.
 *
 * @implemented
 */
NTSTATUS
NTAPI
NtQuerySection(
    _In_ HANDLE SectionHandle,
    _In_ SECTION_INFORMATION_CLASS SectionInformationClass,
    _Out_ PVOID SectionInformation,
    _In_ SIZE_T SectionInformationLength,
    _Out_opt_ PSIZE_T ResultLength)
{
    PSECTION Section;
    KPROCESSOR_MODE PreviousMode;
    NTSTATUS Status;
    PAGED_CODE();

    PreviousMode = ExGetPreviousMode();
    if (PreviousMode != KernelMode)
    {
        _SEH2_TRY
        {
            ProbeForWrite(SectionInformation,
                          SectionInformationLength,
                          __alignof(ULONG));
            if (ResultLength != NULL)
            {
                ProbeForWrite(ResultLength,
                              sizeof(*ResultLength),
                              __alignof(SIZE_T));
            }
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            _SEH2_YIELD(return _SEH2_GetExceptionCode());
        }
        _SEH2_END;
    }

    if (SectionInformationClass == SectionBasicInformation)
    {
        if (SectionInformationLength < sizeof(SECTION_BASIC_INFORMATION))
        {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    }
    else if (SectionInformationClass == SectionImageInformation)
    {
        if (SectionInformationLength < sizeof(SECTION_IMAGE_INFORMATION))
        {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    }
    else
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    Status = ObReferenceObjectByHandle(SectionHandle,
                                       SECTION_QUERY,
                                       MmSectionObjectType,
                                       PreviousMode,
                                       (PVOID*)(PVOID)&Section,
                                       NULL);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Failed to reference section: 0x%lx\n", Status);
        return Status;
    }

    switch(SectionInformationClass)
    {
        case SectionBasicInformation:
        {
            SECTION_BASIC_INFORMATION Sbi = { 0 };

            Sbi.Size = Section->SizeOfSection;
            Sbi.BaseAddress = NULL;

            if (Section->u.Flags.File)
                Sbi.Attributes |= SEC_FILE;
            if (Section->u.Flags.Image)
                Sbi.Attributes |= SEC_IMAGE;
            if (Section->u.Flags.Commit)
                Sbi.Attributes |= SEC_COMMIT;
            if (Section->u.Flags.Reserve)
                Sbi.Attributes |= SEC_RESERVE;
            if (Section->u.Flags.Based)
                Sbi.Attributes |= SEC_BASED;

            if (Section->u.Flags.Image)
            {
                if (!MiIsRosSectionObject(Section))
                {
                    /* Not supported yet */
                    ASSERT(FALSE);
                }
            }
            else if (MiIsRosSectionObject(Section))
            {
                Sbi.BaseAddress = (PVOID)((PMM_SECTION_SEGMENT)Section->Segment)->Image.VirtualAddress;
            }
            else
            {
                Sbi.BaseAddress = Section->Segment->BasedAddress;
            }

            _SEH2_TRY
            {
                *((SECTION_BASIC_INFORMATION*)SectionInformation) = Sbi;
                if (ResultLength != NULL)
                {
                    *ResultLength = sizeof(Sbi);
                }
            }
            _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            {
                Status = _SEH2_GetExceptionCode();
            }
            _SEH2_END;
            break;
        }
        case SectionImageInformation:
        {
            if (!Section->u.Flags.Image)
            {
                Status = STATUS_SECTION_NOT_IMAGE;
            }
            else if (MiIsRosSectionObject(Section))
            {
                PMM_IMAGE_SECTION_OBJECT ImageSectionObject = ((PMM_IMAGE_SECTION_OBJECT)Section->Segment);

                _SEH2_TRY
                {
                    PSECTION_IMAGE_INFORMATION Sii = (PSECTION_IMAGE_INFORMATION)SectionInformation;
                    *Sii = ImageSectionObject->ImageInformation;
                    if (ResultLength != NULL)
                    {
                        *ResultLength = sizeof(*Sii);
                    }
                }
                _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                {
                    Status = _SEH2_GetExceptionCode();
                }
                _SEH2_END;
            }
            else
            {
                _SEH2_TRY
                {
                    PSECTION_IMAGE_INFORMATION Sii = (PSECTION_IMAGE_INFORMATION)SectionInformation;
                    *Sii = *Section->Segment->u2.ImageInformation;
                    if (ResultLength != NULL)
                        *ResultLength = sizeof(*Sii);
                }
                _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                {
                    Status = _SEH2_GetExceptionCode();
                }
                _SEH2_END;
            }
            break;
        }
        default:
            DPRINT1("Unknown SectionInformationClass: %d\n", SectionInformationClass);
            Status = STATUS_NOT_SUPPORTED;
    }

    ObDereferenceObject(Section);

    return Status;
}

/**********************************************************************
 * NAME       EXPORTED
 * MmMapViewOfSection
 *
 * DESCRIPTION
 * Maps a view of a section into the virtual address space of a
 * process.
 *
 * @implemented
 */
NTSTATUS NTAPI
MmMapViewOfSection(IN PVOID SectionObject,
                   IN PEPROCESS Process,
                   IN OUT PVOID *BaseAddress,
                   IN ULONG_PTR ZeroBits,
                   IN SIZE_T CommitSize,
                   IN OUT PLARGE_INTEGER SectionOffset OPTIONAL,
                   IN OUT PSIZE_T ViewSize,
                   IN SECTION_INHERIT InheritDisposition,
                   IN ULONG AllocationType,
                   IN ULONG Protect)
{
    PSECTION Section;
    PMMSUPPORT AddressSpace;
    NTSTATUS Status = STATUS_SUCCESS;
    BOOLEAN NotAtBase = FALSE;
    BOOLEAN IsAttached = FALSE;
    KAPC_STATE ApcState;

    if (MiIsRosSectionObject(SectionObject) == FALSE)
    {
        DPRINT("Mapping ARM3 section into %s\n", Process->ImageFileName);
        return MmMapViewOfArm3Section(SectionObject,
                                      Process,
                                      BaseAddress,
                                      ZeroBits,
                                      CommitSize,
                                      SectionOffset,
                                      ViewSize,
                                      InheritDisposition,
                                      AllocationType,
                                      Protect);
    }

    ASSERT(Process);

    if (!Protect || Protect & ~PAGE_FLAGS_VALID_FOR_SECTION)
    {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    if (PsGetCurrentProcess() != Process)
    {
        KeStackAttachProcess(&Process->Pcb, &ApcState);
        IsAttached = TRUE;
    }

    Section = SectionObject;
    AddressSpace = &Process->Vm;

    if (Section->u.Flags.NoChange)
        AllocationType |= SEC_NO_CHANGE;

    MmLockAddressSpace(AddressSpace);

    if (Section->u.Flags.Image)
    {
        ULONG i;
        ULONG NrSegments;
        ULONG_PTR ImageBase;
        SIZE_T ImageSize;
        PMM_IMAGE_SECTION_OBJECT ImageSectionObject;
        PMM_SECTION_SEGMENT SectionSegments;

        ImageSectionObject = ((PMM_IMAGE_SECTION_OBJECT)Section->Segment);
        SectionSegments = ImageSectionObject->Segments;
        NrSegments = ImageSectionObject->NrSegments;

        ASSERT(ImageSectionObject->RefCount > 0);

        ImageBase = (ULONG_PTR)*BaseAddress;
        if (ImageBase == 0)
        {
            ImageBase = (ULONG_PTR)ImageSectionObject->BasedAddress;
        }

        ImageSize = 0;
        for (i = 0; i < NrSegments; i++)
        {
            ULONG_PTR MaxExtent;
            MaxExtent = (ULONG_PTR)(SectionSegments[i].Image.VirtualAddress +
                                    SectionSegments[i].Length.QuadPart);
            ImageSize = max(ImageSize, MaxExtent);
        }

        ImageSectionObject->ImageInformation.ImageFileSize = (ULONG)ImageSize;

        /* Check for an illegal base address */
        if (((ImageBase + ImageSize) > (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS) ||
                ((ImageBase + ImageSize) < ImageSize))
        {
            ASSERT(*BaseAddress == NULL);
            ImageBase = ALIGN_DOWN_BY((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS - ImageSize,
                                      MM_VIRTMEM_GRANULARITY);
            NotAtBase = TRUE;
        }
        else if (ImageBase != ALIGN_DOWN_BY(ImageBase, MM_VIRTMEM_GRANULARITY))
        {
            ASSERT(*BaseAddress == NULL);
            ImageBase = ALIGN_DOWN_BY(ImageBase, MM_VIRTMEM_GRANULARITY);
            NotAtBase = TRUE;
        }

        /* Check there is enough space to map the section at that point. */
        if (!MmIsAddressRangeFree(AddressSpace, (PVOID)ImageBase, PAGE_ROUND_UP(ImageSize)))
        {
            /* Fail if the user requested a fixed base address. */
            if ((*BaseAddress) != NULL)
            {
                Status = STATUS_CONFLICTING_ADDRESSES;
                goto Exit;
            }
            /* Otherwise find a gap to map the image. */
            ImageBase = (ULONG_PTR)MmFindGap(AddressSpace, PAGE_ROUND_UP(ImageSize), MM_VIRTMEM_GRANULARITY, FALSE);
            if (ImageBase == 0)
            {
                Status = STATUS_CONFLICTING_ADDRESSES;
                goto Exit;
            }
            /* Remember that we loaded image at a different base address */
            NotAtBase = TRUE;
        }

        for (i = 0; i < NrSegments; i++)
        {
            PVOID SBaseAddress = (PVOID)
                                 ((char*)ImageBase + (ULONG_PTR)SectionSegments[i].Image.VirtualAddress);
            MmLockSectionSegment(&SectionSegments[i]);
            Status = MmMapViewOfSegment(AddressSpace,
                                        TRUE,
                                        &SectionSegments[i],
                                        &SBaseAddress,
                                        SectionSegments[i].Length.QuadPart,
                                        SectionSegments[i].Protection,
                                        0,
                                        0);
            MmUnlockSectionSegment(&SectionSegments[i]);
            if (!NT_SUCCESS(Status))
            {
                /* roll-back */
                while (i--)
                {
                    SBaseAddress =  ((char*)ImageBase + (ULONG_PTR)SectionSegments[i].Image.VirtualAddress);
                    MmLockSectionSegment(&SectionSegments[i]);
                    MmUnmapViewOfSegment(AddressSpace, SBaseAddress);
                    MmUnlockSectionSegment(&SectionSegments[i]);
                }

                goto Exit;
            }
        }

        *BaseAddress = (PVOID)ImageBase;
        *ViewSize = ImageSize;

        DPRINT("Mapped %p for section pointer %p\n", ImageSectionObject, ImageSectionObject->FileObject->SectionObjectPointer);

        /* One more map */
        InterlockedIncrement(&ImageSectionObject->MapCount);
    }
    else
    {
        PMM_SECTION_SEGMENT Segment = (PMM_SECTION_SEGMENT)Section->Segment;
        LONGLONG ViewOffset;

        ASSERT(Segment->RefCount > 0);

        /* check for write access */
        if ((Protect & (PAGE_READWRITE|PAGE_EXECUTE_READWRITE)) &&
                !(Section->InitialPageProtection & (PAGE_READWRITE|PAGE_EXECUTE_READWRITE)))
        {
            Status = STATUS_SECTION_PROTECTION;
            goto Exit;
        }
        /* check for read access */
        if ((Protect & (PAGE_READONLY|PAGE_WRITECOPY|PAGE_EXECUTE_READ|PAGE_EXECUTE_WRITECOPY)) &&
                !(Section->InitialPageProtection & (PAGE_READONLY|PAGE_READWRITE|PAGE_WRITECOPY|PAGE_EXECUTE_READ|PAGE_EXECUTE_READWRITE|PAGE_EXECUTE_WRITECOPY)))
        {
            Status = STATUS_SECTION_PROTECTION;
            goto Exit;
        }
        /* check for execute access */
        if ((Protect & (PAGE_EXECUTE|PAGE_EXECUTE_READ|PAGE_EXECUTE_READWRITE|PAGE_EXECUTE_WRITECOPY)) &&
                !(Section->InitialPageProtection & (PAGE_EXECUTE|PAGE_EXECUTE_READ|PAGE_EXECUTE_READWRITE|PAGE_EXECUTE_WRITECOPY)))
        {
            Status = STATUS_SECTION_PROTECTION;
            goto Exit;
        }

        if (SectionOffset == NULL)
        {
            ViewOffset = 0;
        }
        else
        {
            SectionOffset->QuadPart &= ~(PAGE_SIZE - 1);
            ViewOffset = SectionOffset->QuadPart;
        }

        /* Check if the offset and size would cause an overflow */
        if (((ULONG64)ViewOffset + *ViewSize) < (ULONG64)ViewOffset)
        {
            DPRINT1("Section offset overflows\n");
            Status = STATUS_INVALID_VIEW_SIZE;
            goto Exit;
        }

        /* Check if the offset and size are bigger than the section itself */
        if (((ULONG64)ViewOffset + *ViewSize) > (ULONG64)Section->SizeOfSection.QuadPart)
        {
            /* This is allowed for physical memory sections and kernel mode callers */
            if (!Section->u.Flags.PhysicalMemory || (ExGetPreviousMode() == UserMode))
            {
                DPRINT1("Section offset and size are larger than section\n");
                Status = STATUS_INVALID_VIEW_SIZE;
                goto Exit;
            }
        }

        if ((*ViewSize) == 0)
        {
            /* Calculate a view size and make sure it doesn't overflow a SIZE_T */
            ULONG64 CalculatedSize = Section->SizeOfSection.QuadPart - ViewOffset;
            if (CalculatedSize > SIZE_T_MAX)
            {
                DPRINT1("ViewSize is larger than SIZE_T_MAX\n");
                Status = STATUS_INVALID_VIEW_SIZE;
                goto Exit;
            }

            *ViewSize = (SIZE_T)CalculatedSize;
        }
        else if ((ExGetPreviousMode() == UserMode) &&
            (((*ViewSize)+ViewOffset) > Section->SizeOfSection.QuadPart) &&
            (!Section->u.Flags.Reserve))
        {
            /* Dubious */
            (*ViewSize) = MIN(Section->SizeOfSection.QuadPart - ViewOffset, SIZE_T_MAX - PAGE_SIZE);
        }

        *ViewSize = PAGE_ROUND_UP(*ViewSize);

        MmLockSectionSegment(Segment);
        Status = MmMapViewOfSegment(AddressSpace,
                                    FALSE,
                                    Segment,
                                    BaseAddress,
                                    *ViewSize,
                                    Protect,
                                    ViewOffset,
                                    AllocationType & (MEM_TOP_DOWN|SEC_NO_CHANGE));
        MmUnlockSectionSegment(Segment);
        if (!NT_SUCCESS(Status))
        {
            goto Exit;
        }
    }

    if (NotAtBase)
        Status = STATUS_IMAGE_NOT_AT_BASE;
    else
        Status = STATUS_SUCCESS;

Exit:

    MmUnlockAddressSpace(AddressSpace);

    if (IsAttached)
    {
        KeUnstackDetachProcess(&ApcState);
    }

    return Status;
}

/* This function must be called with address space lock held */
NTSTATUS
NTAPI
MiRosUnmapViewInSystemSpace(IN PVOID MappedBase)
{
    DPRINT("MmUnmapViewInSystemSpace() called\n");

    return MmUnmapViewOfSegment(MmGetKernelAddressSpace(), MappedBase);
}

/* This function is not used. It is left for future use, when per-process
 * address space is considered. */
#if 0
NTSTATUS
NTAPI
MmMakePagesDirty(
    _In_opt_ PEPROCESS Process,
    _In_ PVOID Address,
    _In_ ULONG Length)
{
    PMEMORY_AREA MemoryArea;
    PMM_SECTION_SEGMENT Segment;
    LARGE_INTEGER SegmentOffset, RangeEnd;
    PMMSUPPORT AddressSpace = Process ? &Process->Vm : MmGetKernelAddressSpace();

    MmLockAddressSpace(AddressSpace);

    MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, Address);
    if (MemoryArea == NULL)
    {
        DPRINT1("Unable to find memory area at address %p.\n", Address);
        MmUnlockAddressSpace(AddressSpace);
        return STATUS_NOT_MAPPED_VIEW;
    }

    /* Only supported in old Mm for now */
    ASSERT(MemoryArea->Type == MEMORY_AREA_SECTION_VIEW);
    /* For file mappings */
    ASSERT(MemoryArea->VadNode.u.VadFlags.VadType != VadImageMap);

    Segment = MemoryArea->SectionData.Segment;
    MmLockSectionSegment(Segment);

    SegmentOffset.QuadPart = PAGE_ROUND_DOWN(Address) - MA_GetStartingAddress(MemoryArea)
            + MemoryArea->SectionData.ViewOffset;
    RangeEnd.QuadPart = PAGE_ROUND_UP((ULONG_PTR)Address + Length) - MA_GetStartingAddress(MemoryArea)
            + MemoryArea->SectionData.ViewOffset;

    DPRINT("MmMakePagesResident: Segment %p, 0x%I64x -> 0x%I64x\n", Segment, SegmentOffset.QuadPart, RangeEnd.QuadPart);

    while (SegmentOffset.QuadPart < RangeEnd.QuadPart)
    {
        ULONG_PTR Entry = MmGetPageEntrySectionSegment(Segment, &SegmentOffset);

        /* Let any pending read proceed */
        while (MM_IS_WAIT_PTE(Entry))
        {
            MmUnlockSectionSegment(Segment);
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(KernelMode, FALSE, &TinyTime);
            MmLockAddressSpace(AddressSpace);
            MmLockSectionSegment(Segment);
            Entry = MmGetPageEntrySectionSegment(Segment, &SegmentOffset);
        }

        /* We are called from Cc, this can't be backed by the page files */
        ASSERT(!IS_SWAP_FROM_SSE(Entry));

        /* If there is no page there, there is nothing to make dirty */
        if (Entry != 0)
        {
            /* Dirtify the entry */
            MmSetPageEntrySectionSegment(Segment, &SegmentOffset, DIRTY_SSE(Entry));
        }

        SegmentOffset.QuadPart += PAGE_SIZE;
    }

    MmUnlockSectionSegment(Segment);

    MmUnlockAddressSpace(AddressSpace);
    return STATUS_SUCCESS;
}
#endif

/*
 References:
  [1] Microsoft Corporation, "Microsoft Portable Executable and Common Object
      File Format Specification", revision 6.0 (February 1999)
*/
NTSTATUS NTAPI PeFmtCreateSection(IN CONST VOID * FileHeader,
                                  IN SIZE_T FileHeaderSize,
                                  IN PVOID File,
                                  OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
                                  OUT PULONG Flags,
                                  IN PEXEFMT_CB_READ_FILE ReadFileCb,
                                  IN PEXEFMT_CB_ALLOCATE_SEGMENTS AllocateSegmentsCb)
{
    NTSTATUS nStatus;
    ULONG cbFileHeaderOffsetSize = 0;
    ULONG cbSectionHeadersOffset = 0;
    ULONG cbSectionHeadersSize;
    ULONG cbSectionHeadersOffsetSize = 0;
    ULONG cbOptHeaderSize;
    ULONG cbHeadersSize = 0;
    ULONG nSectionAlignment;
    ULONG nFileAlignment;
    ULONG_PTR ImageBase = 0;
    const IMAGE_DOS_HEADER * pidhDosHeader;
    const IMAGE_NT_HEADERS32 * pinhNtHeader;
    const IMAGE_OPTIONAL_HEADER32 * piohOptHeader;
    const IMAGE_SECTION_HEADER * pishSectionHeaders;
    PMM_SECTION_SEGMENT pssSegments;
    LARGE_INTEGER lnOffset;
    PVOID pBuffer;
    SIZE_T nPrevVirtualEndOfSegment = 0;
    ULONG nFileSizeOfHeaders = 0;
    ULONG i;
    ULONG AlignedLength;

    ASSERT(FileHeader);
    ASSERT(FileHeaderSize > 0);
    ASSERT(File);
    ASSERT(ImageSectionObject);
    ASSERT(ReadFileCb);
    ASSERT(AllocateSegmentsCb);

    ASSERT(Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize));

    ASSERT(((UINT_PTR)FileHeader % TYPE_ALIGNMENT(IMAGE_DOS_HEADER)) == 0);

#define DIE(ARGS_) { DPRINT ARGS_; goto l_Return; }

    pBuffer = NULL;
    pidhDosHeader = FileHeader;

    /* DOS HEADER */
    nStatus = STATUS_ROS_EXEFMT_UNKNOWN_FORMAT;

    /* image too small to be an MZ executable */
    if(FileHeaderSize < sizeof(IMAGE_DOS_HEADER))
        DIE(("Too small to be an MZ executable, size is %lu\n", FileHeaderSize));

    /* no MZ signature */
    if(pidhDosHeader->e_magic != IMAGE_DOS_SIGNATURE)
        DIE(("No MZ signature found, e_magic is %hX\n", pidhDosHeader->e_magic));

    /* NT HEADER */
    nStatus = STATUS_INVALID_IMAGE_PROTECT;

    /* not a Windows executable */
    if(pidhDosHeader->e_lfanew <= 0)
        DIE(("Not a Windows executable, e_lfanew is %d\n", pidhDosHeader->e_lfanew));

    if(!Intsafe_AddULong32(&cbFileHeaderOffsetSize, pidhDosHeader->e_lfanew, RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS32, FileHeader)))
        DIE(("The DOS stub is too large, e_lfanew is %X\n", pidhDosHeader->e_lfanew));

    if(FileHeaderSize < cbFileHeaderOffsetSize)
        pinhNtHeader = NULL;
    else
    {
        /*
         * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
         * and FileHeaderSize >= cbFileHeaderOffsetSize, so this holds true too
         */
        ASSERT(Intsafe_CanOffsetPointer(FileHeader, pidhDosHeader->e_lfanew));
        pinhNtHeader = (PVOID)((UINT_PTR)FileHeader + pidhDosHeader->e_lfanew);
    }

    /*
     * the buffer doesn't contain the NT file header, or the alignment is wrong: we
     * need to read the header from the file
     */
    if(FileHeaderSize < cbFileHeaderOffsetSize ||
            (UINT_PTR)pinhNtHeader % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) != 0)
    {
        ULONG cbNtHeaderSize;
        ULONG cbReadSize;
        PVOID pData;

l_ReadHeaderFromFile:
        cbNtHeaderSize = 0;
        lnOffset.QuadPart = pidhDosHeader->e_lfanew;

        /* read the header from the file */
        nStatus = ReadFileCb(File, &lnOffset, sizeof(IMAGE_NT_HEADERS64), &pData, &pBuffer, &cbReadSize);

        if(!NT_SUCCESS(nStatus))
        {
            NTSTATUS ReturnedStatus = nStatus;

            /* If it attempted to read past the end of the file, it means e_lfanew is invalid */
            if (ReturnedStatus == STATUS_END_OF_FILE) nStatus = STATUS_INVALID_IMAGE_PROTECT;

            DIE(("ReadFile failed, status %08X\n", ReturnedStatus));
        }

        ASSERT(pData);
        ASSERT(pBuffer);
        ASSERT(cbReadSize > 0);

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        /* the buffer doesn't contain the file header */
        if(cbReadSize < RTL_SIZEOF_THROUGH_FIELD(IMAGE_NT_HEADERS32, FileHeader))
            DIE(("The file doesn't contain the PE file header\n"));

        pinhNtHeader = pData;

        /* object still not aligned: copy it to the beginning of the buffer */
        if((UINT_PTR)pinhNtHeader % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) != 0)
        {
            ASSERT((UINT_PTR)pBuffer % TYPE_ALIGNMENT(IMAGE_NT_HEADERS32) == 0);
            RtlMoveMemory(pBuffer, pData, cbReadSize);
            pinhNtHeader = pBuffer;
        }

        /* invalid NT header */
        nStatus = STATUS_INVALID_IMAGE_PROTECT;

        if(pinhNtHeader->Signature != IMAGE_NT_SIGNATURE)
            DIE(("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature));

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        if(!Intsafe_AddULong32(&cbNtHeaderSize, pinhNtHeader->FileHeader.SizeOfOptionalHeader, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
            DIE(("The full NT header is too large\n"));

        /* the buffer doesn't contain the whole NT header */
        if(cbReadSize < cbNtHeaderSize)
            DIE(("The file doesn't contain the full NT header\n"));
    }
    else
    {
        ULONG cbOptHeaderOffsetSize = 0;

        nStatus = STATUS_INVALID_IMAGE_PROTECT;

        /* don't trust an invalid NT header */
        if(pinhNtHeader->Signature != IMAGE_NT_SIGNATURE)
            DIE(("The file isn't a PE executable, Signature is %X\n", pinhNtHeader->Signature));

        if(!Intsafe_AddULong32(&cbOptHeaderOffsetSize, pidhDosHeader->e_lfanew, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
            DIE(("The DOS stub is too large, e_lfanew is %X\n", pidhDosHeader->e_lfanew));

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        if(!Intsafe_AddULong32(&cbOptHeaderOffsetSize, cbOptHeaderOffsetSize, pinhNtHeader->FileHeader.SizeOfOptionalHeader))
            DIE(("The NT header is too large, SizeOfOptionalHeader is %X\n", pinhNtHeader->FileHeader.SizeOfOptionalHeader));

        /* the buffer doesn't contain the whole NT header: read it from the file */
        if(cbOptHeaderOffsetSize > FileHeaderSize)
            goto l_ReadHeaderFromFile;
    }

    /* read information from the NT header */
    piohOptHeader = &pinhNtHeader->OptionalHeader;
    cbOptHeaderSize = pinhNtHeader->FileHeader.SizeOfOptionalHeader;

    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    if(!RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, Magic))
        DIE(("The optional header doesn't contain the Magic field, SizeOfOptionalHeader is %X\n", cbOptHeaderSize));

    /* ASSUME: RtlZeroMemory(ImageSectionObject, sizeof(*ImageSectionObject)); */

    switch(piohOptHeader->Magic)
    {
        case IMAGE_NT_OPTIONAL_HDR64_MAGIC:
#ifndef _WIN64
            nStatus = STATUS_INVALID_IMAGE_WIN_64;
            DIE(("Win64 optional header, unsupported\n"));
#else
            // Fall through.
#endif
        case IMAGE_NT_OPTIONAL_HDR32_MAGIC:
            break;
        default:
            DIE(("Unrecognized optional header, Magic is %X\n", piohOptHeader->Magic));
    }

    if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SectionAlignment) &&
            RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, FileAlignment))
    {
        /* See [1], section 3.4.2 */
        if(piohOptHeader->SectionAlignment < PAGE_SIZE)
        {
            if(piohOptHeader->FileAlignment != piohOptHeader->SectionAlignment)
                DIE(("Sections aren't page-aligned and the file alignment isn't the same\n"));
        }
        else if(piohOptHeader->SectionAlignment < piohOptHeader->FileAlignment)
            DIE(("The section alignment is smaller than the file alignment\n"));

        nSectionAlignment = piohOptHeader->SectionAlignment;
        nFileAlignment = piohOptHeader->FileAlignment;

        if(!IsPowerOf2(nSectionAlignment) || !IsPowerOf2(nFileAlignment))
            DIE(("The section alignment (%u) and file alignment (%u) aren't both powers of 2\n", nSectionAlignment, nFileAlignment));
    }
    else
    {
        nSectionAlignment = PAGE_SIZE;
        nFileAlignment = PAGE_SIZE;
    }

    ASSERT(IsPowerOf2(nSectionAlignment));
    ASSERT(IsPowerOf2(nFileAlignment));

    switch(piohOptHeader->Magic)
    {
    /* PE32 */
    case IMAGE_NT_OPTIONAL_HDR32_MAGIC:
    {
        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, ImageBase))
            ImageBase = piohOptHeader->ImageBase;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfImage))
            ImageSectionObject->ImageInformation.ImageFileSize = piohOptHeader->SizeOfImage;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfStackReserve))
            ImageSectionObject->ImageInformation.MaximumStackSize = piohOptHeader->SizeOfStackReserve;

        if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfStackCommit))
            ImageSectionObject->ImageInformation.CommittedStackSize = piohOptHeader->SizeOfStackCommit;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, Subsystem))
        {
            ImageSectionObject->ImageInformation.SubSystemType = piohOptHeader->Subsystem;

            if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, MinorSubsystemVersion) &&
                    RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, MajorSubsystemVersion))
            {
                ImageSectionObject->ImageInformation.SubSystemMinorVersion = piohOptHeader->MinorSubsystemVersion;
                ImageSectionObject->ImageInformation.SubSystemMajorVersion = piohOptHeader->MajorSubsystemVersion;
            }
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            ImageSectionObject->ImageInformation.TransferAddress = (PVOID) (ImageBase +
                    piohOptHeader->AddressOfEntryPoint);
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfCode))
            ImageSectionObject->ImageInformation.ImageContainsCode = piohOptHeader->SizeOfCode != 0;
        else
            ImageSectionObject->ImageInformation.ImageContainsCode = TRUE;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            if (piohOptHeader->AddressOfEntryPoint == 0)
            {
                ImageSectionObject->ImageInformation.ImageContainsCode = FALSE;
            }
        }

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, LoaderFlags))
            ImageSectionObject->ImageInformation.LoaderFlags = piohOptHeader->LoaderFlags;

        if (RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, DllCharacteristics))
        {
            ImageSectionObject->ImageInformation.DllCharacteristics = piohOptHeader->DllCharacteristics;

            /*
             * Since we don't really implement SxS yet and LD doesn't supoprt /ALLOWISOLATION:NO, hard-code
             * this flag here, which will prevent the loader and other code from doing any .manifest or SxS
             * magic to any binary.
             *
             * This will break applications that depend on SxS when running with real Windows Kernel32/SxS/etc
             * but honestly that's not tested. It will also break them when running no ReactOS once we implement
             * the SxS support -- at which point, duh, this should be removed.
             *
             * But right now, any app depending on SxS is already broken anyway, so this flag only helps.
             */
            ImageSectionObject->ImageInformation.DllCharacteristics |= IMAGE_DLLCHARACTERISTICS_NO_ISOLATION;
        }

        break;
    }
#ifdef _WIN64
    /* PE64 */
    case IMAGE_NT_OPTIONAL_HDR64_MAGIC:
    {
        const IMAGE_OPTIONAL_HEADER64 * pioh64OptHeader;

        pioh64OptHeader = (const IMAGE_OPTIONAL_HEADER64 *)piohOptHeader;

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, ImageBase))
        {
            ImageBase = pioh64OptHeader->ImageBase;
            if(pioh64OptHeader->ImageBase > MAXULONG_PTR)
                DIE(("ImageBase exceeds the address space\n"));
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfImage))
        {
            if(pioh64OptHeader->SizeOfImage > MAXULONG_PTR)
                DIE(("SizeOfImage exceeds the address space\n"));

            ImageSectionObject->ImageInformation.ImageFileSize = pioh64OptHeader->SizeOfImage;
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfStackReserve))
        {
            if(pioh64OptHeader->SizeOfStackReserve > MAXULONG_PTR)
                DIE(("SizeOfStackReserve exceeds the address space\n"));

            ImageSectionObject->ImageInformation.MaximumStackSize = (ULONG_PTR) pioh64OptHeader->SizeOfStackReserve;
        }

        if(RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfStackCommit))
        {
            if(pioh64OptHeader->SizeOfStackCommit > MAXULONG_PTR)
                DIE(("SizeOfStackCommit exceeds the address space\n"));

            ImageSectionObject->ImageInformation.CommittedStackSize = (ULONG_PTR) pioh64OptHeader->SizeOfStackCommit;
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, Subsystem))
        {
            ImageSectionObject->ImageInformation.SubSystemType = pioh64OptHeader->Subsystem;

            if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, MinorSubsystemVersion) &&
                    RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, MajorSubsystemVersion))
            {
                ImageSectionObject->ImageInformation.SubSystemMinorVersion = pioh64OptHeader->MinorSubsystemVersion;
                ImageSectionObject->ImageInformation.SubSystemMajorVersion = pioh64OptHeader->MajorSubsystemVersion;
            }
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            ImageSectionObject->ImageInformation.TransferAddress = (PVOID) (ImageBase +
                    pioh64OptHeader->AddressOfEntryPoint);
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, SizeOfCode))
            ImageSectionObject->ImageInformation.ImageContainsCode = pioh64OptHeader->SizeOfCode != 0;
        else
            ImageSectionObject->ImageInformation.ImageContainsCode = TRUE;

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, AddressOfEntryPoint))
        {
            if (pioh64OptHeader->AddressOfEntryPoint == 0)
            {
                ImageSectionObject->ImageInformation.ImageContainsCode = FALSE;
            }
        }

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, LoaderFlags))
            ImageSectionObject->ImageInformation.LoaderFlags = pioh64OptHeader->LoaderFlags;

        if (RTL_CONTAINS_FIELD(pioh64OptHeader, cbOptHeaderSize, DllCharacteristics))
            ImageSectionObject->ImageInformation.DllCharacteristics = pioh64OptHeader->DllCharacteristics;

        break;
    }
#endif // _WIN64
    }

    /* [1], section 3.4.2 */
    if((ULONG_PTR)ImageBase % 0x10000)
        DIE(("ImageBase is not aligned on a 64KB boundary"));

    ImageSectionObject->ImageInformation.ImageCharacteristics = pinhNtHeader->FileHeader.Characteristics;
    ImageSectionObject->ImageInformation.Machine = pinhNtHeader->FileHeader.Machine;
    ImageSectionObject->ImageInformation.GpValue = 0;
    ImageSectionObject->ImageInformation.ZeroBits = 0;
    ImageSectionObject->BasedAddress = (PVOID)ImageBase;

    /* SECTION HEADERS */
    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    /* see [1], section 3.3 */
    if(pinhNtHeader->FileHeader.NumberOfSections > 96)
        DIE(("Too many sections, NumberOfSections is %u\n", pinhNtHeader->FileHeader.NumberOfSections));

    /*
     * the additional segment is for the file's headers. They need to be present for
     * the benefit of the dynamic loader (to locate exports, defaults for thread
     * parameters, resources, etc.)
     */
    ImageSectionObject->NrSegments = pinhNtHeader->FileHeader.NumberOfSections + 1;

    /* file offset for the section headers */
    if(!Intsafe_AddULong32(&cbSectionHeadersOffset, pidhDosHeader->e_lfanew, FIELD_OFFSET(IMAGE_NT_HEADERS32, OptionalHeader)))
        DIE(("Offset overflow\n"));

    if(!Intsafe_AddULong32(&cbSectionHeadersOffset, cbSectionHeadersOffset, pinhNtHeader->FileHeader.SizeOfOptionalHeader))
        DIE(("Offset overflow\n"));

    /* size of the section headers */
    ASSERT(Intsafe_CanMulULong32(pinhNtHeader->FileHeader.NumberOfSections, sizeof(IMAGE_SECTION_HEADER)));
    cbSectionHeadersSize = pinhNtHeader->FileHeader.NumberOfSections * sizeof(IMAGE_SECTION_HEADER);

    if(!Intsafe_AddULong32(&cbSectionHeadersOffsetSize, cbSectionHeadersOffset, cbSectionHeadersSize))
        DIE(("Section headers too large\n"));

    /* size of the executable's headers */
    if(RTL_CONTAINS_FIELD(piohOptHeader, cbOptHeaderSize, SizeOfHeaders))
    {
//        if(!IsAligned(piohOptHeader->SizeOfHeaders, nFileAlignment))
//            DIE(("SizeOfHeaders is not aligned\n"));

        if(cbSectionHeadersSize > piohOptHeader->SizeOfHeaders)
            DIE(("The section headers overflow SizeOfHeaders\n"));

        cbHeadersSize = piohOptHeader->SizeOfHeaders;
    }
    else if(!AlignUp(&cbHeadersSize, cbSectionHeadersOffsetSize, nFileAlignment))
        DIE(("Overflow aligning the size of headers\n"));

    if(pBuffer)
    {
        ExFreePool(pBuffer);
        pBuffer = NULL;
    }
    /* WARNING: pinhNtHeader IS NO LONGER USABLE */
    /* WARNING: piohOptHeader IS NO LONGER USABLE */
    /* WARNING: pioh64OptHeader IS NO LONGER USABLE */

    if(FileHeaderSize < cbSectionHeadersOffsetSize)
        pishSectionHeaders = NULL;
    else
    {
        /*
         * we already know that Intsafe_CanOffsetPointer(FileHeader, FileHeaderSize),
         * and FileHeaderSize >= cbSectionHeadersOffsetSize, so this holds true too
         */
        ASSERT(Intsafe_CanOffsetPointer(FileHeader, cbSectionHeadersOffset));
        pishSectionHeaders = (PVOID)((UINT_PTR)FileHeader + cbSectionHeadersOffset);
    }

    /*
     * the buffer doesn't contain the section headers, or the alignment is wrong:
     * read the headers from the file
     */
    if(FileHeaderSize < cbSectionHeadersOffsetSize ||
            (UINT_PTR)pishSectionHeaders % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) != 0)
    {
        PVOID pData;
        ULONG cbReadSize;

        lnOffset.QuadPart = cbSectionHeadersOffset;

        /* read the header from the file */
        nStatus = ReadFileCb(File, &lnOffset, cbSectionHeadersSize, &pData, &pBuffer, &cbReadSize);

        if(!NT_SUCCESS(nStatus))
            DIE(("ReadFile failed with status %08X\n", nStatus));

        ASSERT(pData);
        ASSERT(pBuffer);
        ASSERT(cbReadSize > 0);

        nStatus = STATUS_INVALID_IMAGE_FORMAT;

        /* the buffer doesn't contain all the section headers */
        if(cbReadSize < cbSectionHeadersSize)
            DIE(("The file doesn't contain all of the section headers\n"));

        pishSectionHeaders = pData;

        /* object still not aligned: copy it to the beginning of the buffer */
        if((UINT_PTR)pishSectionHeaders % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) != 0)
        {
            ASSERT((UINT_PTR)pBuffer % TYPE_ALIGNMENT(IMAGE_SECTION_HEADER) == 0);
            RtlMoveMemory(pBuffer, pData, cbReadSize);
            pishSectionHeaders = pBuffer;
        }
    }

    /* SEGMENTS */
    /* allocate the segments */
    nStatus = STATUS_INSUFFICIENT_RESOURCES;
    ImageSectionObject->Segments = AllocateSegmentsCb(ImageSectionObject->NrSegments);

    if(ImageSectionObject->Segments == NULL)
        DIE(("AllocateSegments failed\n"));

    /* initialize the headers segment */
    pssSegments = ImageSectionObject->Segments;

//  ASSERT(IsAligned(cbHeadersSize, nFileAlignment));

    if(!AlignUp(&nFileSizeOfHeaders, cbHeadersSize, nFileAlignment))
        DIE(("Cannot align the size of the section headers\n"));

    nPrevVirtualEndOfSegment = ALIGN_UP_BY(cbHeadersSize, nSectionAlignment);
    if (nPrevVirtualEndOfSegment < cbHeadersSize)
        DIE(("Cannot align the size of the section headers\n"));

    pssSegments[0].Image.FileOffset = 0;
    pssSegments[0].Protection = PAGE_READONLY;
    pssSegments[0].Length.QuadPart = nPrevVirtualEndOfSegment;
    pssSegments[0].RawLength.QuadPart = nFileSizeOfHeaders;
    pssSegments[0].Image.VirtualAddress = 0;
    pssSegments[0].Image.Characteristics = 0;
    pssSegments[0].WriteCopy = TRUE;

    /* skip the headers segment */
    ++ pssSegments;

    nStatus = STATUS_INVALID_IMAGE_FORMAT;

    ASSERT(ImageSectionObject->RefCount > 0);

    /* convert the executable sections into segments. See also [1], section 4 */
    for(i = 0; i < ImageSectionObject->NrSegments - 1; ++ i)
    {
        ULONG nCharacteristics;

        /* validate the alignment */
        if(!IsAligned(pishSectionHeaders[i].VirtualAddress, nSectionAlignment))
            DIE(("Image.VirtualAddress[%u] is not aligned\n", i));

        /* sections must be contiguous, ordered by base address and non-overlapping */
        if(pishSectionHeaders[i].VirtualAddress != nPrevVirtualEndOfSegment)
            DIE(("Memory gap between section %u and the previous\n", i));

        /* ignore explicit BSS sections */
        if(pishSectionHeaders[i].PointerToRawData != 0 && pishSectionHeaders[i].SizeOfRawData != 0)
        {
            /* validate the alignment */
#if 0
            /* Yes, this should be a multiple of FileAlignment, but there's
             * stuff out there that isn't. We can cope with that
             */
            if(!IsAligned(pishSectionHeaders[i].SizeOfRawData, nFileAlignment))
                DIE(("SizeOfRawData[%u] is not aligned\n", i));
#endif

//            if(!IsAligned(pishSectionHeaders[i].PointerToRawData, nFileAlignment))
//                DIE(("PointerToRawData[%u] is not aligned\n", i));

            if(!Intsafe_CanAddULong32(pishSectionHeaders[i].PointerToRawData, pishSectionHeaders[i].SizeOfRawData))
                DIE(("SizeOfRawData[%u] too large\n", i));

            /* conversion */
            pssSegments[i].Image.FileOffset = pishSectionHeaders[i].PointerToRawData;
            pssSegments[i].RawLength.QuadPart = pishSectionHeaders[i].SizeOfRawData;
        }
        else
        {
            /* FIXME: Should reset PointerToRawData to 0 in the image mapping */
            ASSERT(pssSegments[i].Image.FileOffset == 0);
            ASSERT(pssSegments[i].RawLength.QuadPart == 0);
        }

        ASSERT(Intsafe_CanAddLong64(pssSegments[i].Image.FileOffset, pssSegments[i].RawLength.QuadPart));

        nCharacteristics = pishSectionHeaders[i].Characteristics;

        /* no explicit protection */
        if((nCharacteristics & (IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE)) == 0)
        {
            if(nCharacteristics & IMAGE_SCN_CNT_CODE)
                nCharacteristics |= IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ;

            if(nCharacteristics & IMAGE_SCN_CNT_INITIALIZED_DATA)
                nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;

            if(nCharacteristics & IMAGE_SCN_CNT_UNINITIALIZED_DATA)
                nCharacteristics |= IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
        }

        /* see table above */
        pssSegments[i].Protection = SectionCharacteristicsToProtect[nCharacteristics >> 28];
        pssSegments[i].WriteCopy = !(nCharacteristics & IMAGE_SCN_MEM_SHARED);

        if(pishSectionHeaders[i].Misc.VirtualSize == 0)
            pssSegments[i].Length.QuadPart = pishSectionHeaders[i].SizeOfRawData;
        else
            pssSegments[i].Length.QuadPart = pishSectionHeaders[i].Misc.VirtualSize;

        AlignedLength = ALIGN_UP_BY(pssSegments[i].Length.LowPart, nSectionAlignment);
        if(AlignedLength < pssSegments[i].Length.LowPart)
            DIE(("Cannot align the virtual size of section %u\n", i));

        pssSegments[i].Length.LowPart = AlignedLength;

        if(pssSegments[i].Length.QuadPart == 0)
            DIE(("Virtual size of section %u is null\n", i));

        pssSegments[i].Image.VirtualAddress = pishSectionHeaders[i].VirtualAddress;
        pssSegments[i].Image.Characteristics = pishSectionHeaders[i].Characteristics;

        /* ensure the memory image is no larger than 4GB */
        nPrevVirtualEndOfSegment = (ULONG_PTR)(pssSegments[i].Image.VirtualAddress + pssSegments[i].Length.QuadPart);
        if (nPrevVirtualEndOfSegment < pssSegments[i].Image.VirtualAddress)
            DIE(("The image is too large\n"));
    }

    if(nSectionAlignment >= PAGE_SIZE)
        *Flags |= EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED;

    /* Success */
    nStatus = STATUS_SUCCESS;// STATUS_ROS_EXEFMT_LOADED_FORMAT | EXEFMT_LOADED_PE32;

l_Return:
    if(pBuffer)
        ExFreePool(pBuffer);

    return nStatus;
}
