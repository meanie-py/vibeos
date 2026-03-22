use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _HANDLE_TABLE_ENTRY,
    _OBJECT_HEADER,
    _OBJECT_TYPE,
    _ETHREAD,
    PEPROCESS,
    POBJECT_HEADER,
    POBJECT_TYPE,
    PHANDLE_TABLE,
    PHANDLE_TABLE_ENTRY,
    ACCESS_MASK,
    KPROCESSOR_MODE,
    HANDLE,
    OB_OPEN_REASON,
    ExAcquireFastMutex,
    ExReleaseFastMutex,
    ExInitializeFastMutex,
    ExGetPreviousMode,
    KeDelayExecutionThread,
    KeBugCheck,
    KeBugCheckEx,
    DbgkUnMapViewOfSection,
    CcFlushCache,
    LARGE_INTEGER,
    _KAPC_STATE,
    KEVENT,
    _IO_STATUS_BLOCK,
    IMAGE_DOS_HEADER,
    IMAGE_NT_HEADERS32,
    IMAGE_NT_HEADERS64,
    IMAGE_OPTIONAL_HEADER32,
    IMAGE_OPTIONAL_HEADER64,
    IMAGE_SECTION_HEADER,
    MEMORY_AREA,
    MEMORY_BASIC_INFORMATION,
    MM_SECTION_SEGMENT,
    MM_IMAGE_SECTION_OBJECT,
    MMSESSION,
    File,
    IoSetInformation,
    MmAlterRegion,
    MmCreateMemoryArea,
    MmCreatePageFileMapping,
    MmCreatePhysicalMapping,
    MmCreateVirtualMapping,
    MmDeletePageFileMapping,
    MmDeleteRmap,
    MmDeleteVirtualMapping,
    MmDereferenceSegment,
    MmFlushSegment,
    MmFreeMemoryArea,
    MmGetAddressSpaceOwner,
    MmGetKernelAddressSpace,
    MmGetPageEntrySectionSegment,
    MmLockSectionSegment,
    MmUnlockSectionSegment,
    MmProtectToValue,
};

static mut MmWaitPageEvent: KEVENT = unsafe { core::mem::zeroed() };
static mut TinyTime: LARGE_INTEGER = unsafe { core::mem::transmute(-1i64) };

#[repr(C)]
struct MM_SECTION_PAGEOUT_CONTEXT {
    MemoryArea: *mut MEMORY_AREA,
    Segment: *mut MM_SECTION_SEGMENT,
    Offset: LARGE_INTEGER,
    WasDirty: BOOLEAN,
    Private: BOOLEAN,
    CallingProcess: PEPROCESS,
    SectionEntry: ULONG_PTR,
}

static mut SectionCharacteristicsToProtect: [ULONG; 16] = [
    PAGE_NOACCESS, PAGE_NOACCESS, PAGE_EXECUTE, PAGE_EXECUTE,
    PAGE_READONLY, PAGE_READONLY, PAGE_EXECUTE_READ, PAGE_EXECUTE_READ,
    PAGE_READWRITE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_READWRITE,
    PAGE_READWRITE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_READWRITE,
];

#[cfg(not(feature = "NEWCC"))]
unsafe fn _MmLockSectionSegment(Segment: *mut MM_SECTION_SEGMENT, file: *const i8, line: i32) {
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
    (*Segment).Locked = 1u8;
}

#[cfg(not(feature = "NEWCC"))]
unsafe fn _MmUnlockSectionSegment(Segment: *mut MM_SECTION_SEGMENT, file: *const i8, line: i32) {
    (*Segment).Locked = 0u8;
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
}

unsafe fn MmAlterViewAttributes(
    AddressSpace: PMMSUPPORT,
    BaseAddress: PVOID,
    RegionSize: SIZE_T,
    OldType: ULONG,
    OldProtect: ULONG,
    NewType: ULONG,
    NewProtect: ULONG,
) {
    let mut DoCOW = 0u8;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, BaseAddress);
    let Segment = (*MemoryArea).SectionData.Segment;
    MmLockSectionSegment(Segment);

    if (*Segment).WriteCopy != 0 && (NewProtect == PAGE_READWRITE || NewProtect == PAGE_EXECUTE_READWRITE) {
        DoCOW = 1u8;
    }

    if OldProtect != NewProtect {
        let mut i = 0;
        while i < PAGE_ROUND_UP(RegionSize) / PAGE_SIZE {
            let mut SwapEntry = 0;
            let Address = (BaseAddress as ULONG_PTR + i * PAGE_SIZE) as PVOID;
            let mut Protect = NewProtect;

            loop {
                MmGetPageFileMapping(Process, Address, &mut SwapEntry);
                if SwapEntry != MM_WAIT_ENTRY { break; }
                MmUnlockSectionSegment(Segment);
                MmUnlockAddressSpace(AddressSpace);
                KeDelayExecutionThread(0i8, 0u8, &TinyTime);
                MmLockAddressSpace(AddressSpace);
                MmLockSectionSegment(Segment);
            }

            if DoCOW != 0 && (MmIsPagePresent(Process, Address) != 0 || MmIsDisabledPage(Process, Address) != 0) {
                let mut Offset = LARGE_INTEGER { QuadPart: 0 };
                Offset.QuadPart = (Address as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
                    + (*MemoryArea).SectionData.ViewOffset;
                let Entry = MmGetPageEntrySectionSegment(Segment, &mut Offset);
                let Page = MmGetPfnForProcess(Process, Address);

                Protect = if NewProtect == PAGE_EXECUTE_READWRITE {
                    PAGE_EXECUTE_READ
                } else {
                    PAGE_READONLY
                };

                if IS_SWAP_FROM_SSE(Entry) != 0 || PFN_FROM_SSE(Entry) != Page {
                    Protect = NewProtect;
                }
            }

            if MmIsPagePresent(Process, Address) != 0 || MmIsDisabledPage(Process, Address) != 0 {
                MmSetPageProtect(Process, Address, Protect);
            }
            i += 1;
        }
    }

    MmUnlockSectionSegment(Segment);
}

#[no_mangle]
pub unsafe extern "system" fn MmNotPresentFaultSectionView(
    AddressSpace: PMMSUPPORT,
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    Locked: BOOLEAN,
) -> NTSTATUS {
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let mut Page: PFN_NUMBER = 0;
    let mut Status: NTSTATUS;
    let mut Entry: ULONG_PTR;
    let mut Entry1: ULONG_PTR;
    let Attributes: ULONG;
    let Region: PMM_REGION;
    let HasSwapEntry: BOOLEAN;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let mut SwapEntry: SWAPENTRY = 0;

    if MmIsPagePresent(Process, Address) != 0 {
        return STATUS_SUCCESS;
    }

    if MmIsDisabledPage(Process, Address) != 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    if (*MemoryArea).DeleteInProgress != 0 {
        return STATUS_UNSUCCESSFUL;
    }

    let PAddress = MM_ROUND_DOWN(Address as ULONG_PTR, PAGE_SIZE) as PVOID;
    Offset.QuadPart = (PAddress as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
        + (*MemoryArea).SectionData.ViewOffset;

    let Segment = (*MemoryArea).SectionData.Segment;
    Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        Address,
        core::ptr::null_mut(),
    );

    if ((*Region).Protect & PAGE_GUARD) != 0 {
        Status = MmAlterRegion(
            AddressSpace,
            MA_GetStartingAddress(MemoryArea) as PVOID,
            core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
            Address,
            PAGE_SIZE,
            (*Region).Type,
            (*Region).Protect & !PAGE_GUARD,
            MmAlterViewAttributes,
        );
        return STATUS_GUARD_PAGE_VIOLATION;
    }

    HasSwapEntry = MmIsPageSwapEntry(Process, Address);

    if HasSwapEntry != 0 {
        let mut DummyEntry: SWAPENTRY = 0;
        MmGetPageFileMapping(Process, Address, &mut SwapEntry);
        if SwapEntry == MM_WAIT_ENTRY {
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(0i8, 0u8, &TinyTime);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        MI_SET_USAGE(MI_USAGE_SECTION);
        if !Process.is_null() { MI_SET_PROCESS2((*(Process as *mut _EPROCESS)).ImageFileName.as_ptr() as *const i8); }
        if Process.is_null() { MI_SET_PROCESS2(b"Kernel Section\0".as_ptr() as *const i8); }
        Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut Page);
        if Status < 0 {
            return STATUS_NO_MEMORY;
        }

        MmDeletePageFileMapping(Process, Address, &mut DummyEntry);
        MmCreatePageFileMapping(Process, Address, MM_WAIT_ENTRY);
        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmLockAddressSpace(AddressSpace);
        MmDeletePageFileMapping(Process, PAddress, &mut DummyEntry);

        Status = MmCreateVirtualMapping(Process, PAddress, (*Region).Protect, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmSetSavedSwapEntryPage(Page, SwapEntry);
        if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
        return STATUS_SUCCESS;
    }

    MmLockSectionSegment(Segment);

    if ((*(*Segment).Flags) & MM_PHYSICALMEMORY_SEGMENT) != 0 {
        MmUnlockSectionSegment(Segment);
        Page = (Offset.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
        Status = MmCreatePhysicalMapping(Process, PAddress, (*Region).Protect, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        return STATUS_SUCCESS;
    }

    let Attributes = if (*Segment).WriteCopy != 0 && ((*Region).Protect == PAGE_READWRITE || (*Region).Protect == PAGE_EXECUTE_READWRITE) {
        if (*Region).Protect == PAGE_READWRITE { PAGE_READONLY } else { PAGE_EXECUTE_READ }
    } else {
        (*Region).Protect
    };

    Entry = MmGetPageEntrySectionSegment(Segment, &mut Offset);
    if Entry == 0 {
        if (Offset.QuadPart >= PAGE_ROUND_UP((*Segment).RawLength.QuadPart) as i64)
            && ((*MemoryArea).VadNode.u.VadFlags.VadType == VadImageMap) {
            MI_SET_USAGE(MI_USAGE_SECTION);
            if !Process.is_null() { MI_SET_PROCESS2((*(Process as *mut _EPROCESS)).ImageFileName.as_ptr() as *const i8); }
            if Process.is_null() { MI_SET_PROCESS2(b"Kernel Section\0".as_ptr() as *const i8); }
            Status = MmRequestPageMemoryConsumer(MC_USER, 0u8, &mut Page);
            if Status < 0 {
                MmUnlockSectionSegment(Segment);
                return STATUS_NO_MEMORY;
            }
            MmSetPageEntrySectionSegment(Segment, &mut Offset, MAKE_SSE(Page << PAGE_SHIFT, 1));
            MmUnlockSectionSegment(Segment);

            Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
            if Status < 0 {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
            if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
            return STATUS_SUCCESS;
        }

        MmUnlockSectionSegment(Segment);
        MmUnlockAddressSpace(AddressSpace);

        FsRtlAcquireFileExclusive((*Segment).FileObject);
        let FcbHeader = (*(*Segment).FileObject).FsContext as *mut PFSRTL_COMMON_FCB_HEADER;
        Status = MmMakeSegmentResident(Segment, Offset.QuadPart as ULONG_PTR, PAGE_SIZE, &mut (*FcbHeader).ValidDataLength, 0u8);
        FsRtlReleaseFile((*Segment).FileObject);

        MmLockAddressSpace(AddressSpace);
        if Status < 0 {
            if Status == STATUS_END_OF_FILE {
                return STATUS_INVALID_FILE_FOR_SECTION;
            }
            return STATUS_IN_PAGE_ERROR;
        }
        return STATUS_MM_RESTART_OPERATION;
    } else if IS_SWAP_FROM_SSE(Entry) != 0 {
        let SwapEntry = SWAPENTRY_FROM_SSE(Entry);

        if SwapEntry == MM_WAIT_ENTRY {
            MmUnlockSectionSegment(Segment);
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(0i8, 0u8, &TinyTime);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut Page);
        if Status < 0 {
            MmUnlockSectionSegment(Segment);
            return STATUS_NO_MEMORY;
        }

        MmSetPageEntrySectionSegment(Segment, &mut Offset, MAKE_SWAP_SSE(MM_WAIT_ENTRY));
        MmUnlockSectionSegment(Segment);
        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment);

        Entry1 = MmGetPageEntrySectionSegment(Segment, &mut Offset);
        if Entry1 != MAKE_SWAP_SSE(MM_WAIT_ENTRY) {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmSetSavedSwapEntryPage(Page, SwapEntry);

        Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        if !Process.is_null() { MmInsertRmap(Page, Process, Address); }

        Entry = MAKE_SSE(Page << PAGE_SHIFT, 1);
        MmSetPageEntrySectionSegment(Segment, &mut Offset, Entry);
        MmUnlockSectionSegment(Segment);
        return STATUS_SUCCESS;
    } else {
        Page = PFN_FROM_SSE(Entry);

        Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
        MmSharePageEntrySectionSegment(Segment, &mut Offset);
        MmUnlockSectionSegment(Segment);
        return STATUS_SUCCESS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmAccessFaultSectionView(
    AddressSpace: PMMSUPPORT,
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    Locked: BOOLEAN,
) -> NTSTATUS {
    let Segment: *mut MM_SECTION_SEGMENT;
    let mut OldPage: PFN_NUMBER = 0;
    let mut NewPage: PFN_NUMBER = 0;
    let mut UnmappedPage: PFN_NUMBER = 0;
    let mut PAddress: PVOID;
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let Region: PMM_REGION;
    let mut Entry: ULONG_PTR;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let mut Cow = 0u8;
    let mut NewProtect: ULONG;
    let Unmapped: BOOLEAN;
    let mut Status: NTSTATUS;

    Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        Address,
        core::ptr::null_mut(),
    );

    if ((*Region).Protect & PAGE_IS_WRITABLE) == 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    if MmIsPagePresent(Process, Address) == 0 {
        Status = MmNotPresentFaultSectionView(AddressSpace, MemoryArea, Address, Locked);
        if Status < 0 {
            return Status;
        }
    }

    if (MmGetPageProtect(Process, Address) & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) != 0 {
        return STATUS_SUCCESS;
    }

    Segment = (*MemoryArea).SectionData.Segment;
    Cow = if (*Segment).WriteCopy != 0 || ((*Region).Protect & PAGE_IS_WRITECOPY) != 0 { 1u8 } else { 0u8 };

    if Cow == 0 {
        MmSetPageProtect(Process, Address, (*Region).Protect);
        return STATUS_SUCCESS;
    }

    NewProtect = (*Region).Protect;
    if (NewProtect & PAGE_IS_WRITECOPY) != 0 {
        NewProtect &= !PAGE_IS_WRITECOPY;
        if ((*Region).Protect & PAGE_IS_EXECUTABLE) != 0 {
            NewProtect |= PAGE_EXECUTE_READWRITE;
        } else {
            NewProtect |= PAGE_READWRITE;
        }
        MmAlterRegion(
            AddressSpace,
            MA_GetStartingAddress(MemoryArea) as PVOID,
            core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
            Address,
            PAGE_SIZE,
            (*Region).Type,
            NewProtect,
            MmAlterViewAttributes,
        );
    }

    PAddress = MM_ROUND_DOWN(Address as ULONG_PTR, PAGE_SIZE) as PVOID;
    Offset.QuadPart = (PAddress as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
        + (*MemoryArea).SectionData.ViewOffset;

    MmLockSectionSegment(Segment);
    Entry = MmGetPageEntrySectionSegment(Segment, &mut Offset);

    OldPage = MmGetPfnForProcess(Process, PAddress);

    if IS_SWAP_FROM_SSE(Entry) != 0 || PFN_FROM_SSE(Entry) != OldPage {
        MmUnlockSectionSegment(Segment);
        MmSetPageProtect(Process, PAddress, NewProtect);
        return STATUS_SUCCESS;
    }

    Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut NewPage);
    if Status < 0 {
        MmUnlockSectionSegment(Segment);
        return STATUS_NO_MEMORY;
    }

    MiCopyFromUserPage(NewPage, PAddress);

    Unmapped = MmDeleteVirtualMapping(Process, PAddress, core::ptr::null_mut(), &mut UnmappedPage);
    if Unmapped == 0 || UnmappedPage != OldPage {
        KeBugCheckEx(
            MEMORY_MANAGEMENT,
            Process as ULONG_PTR,
            PAddress as ULONG_PTR,
            file!().as_ptr() as ULONG_PTR,
            line!() as ULONG_PTR,
        );
    }

    if !Process.is_null() { MmDeleteRmap(OldPage, Process, PAddress); }
    MmUnsharePageEntrySectionSegment(MemoryArea, Segment, &mut Offset, 0u8, 0u8, core::ptr::null_mut());
    MmUnlockSectionSegment(Segment);

    Status = MmCreateVirtualMapping(Process, PAddress, NewProtect, NewPage);
    if Status < 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if !Process.is_null() { MmInsertRmap(NewPage, Process, PAddress); }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MmProtectSectionView(
    AddressSpace: PMMSUPPORT,
    MemoryArea: *mut MEMORY_AREA,
    BaseAddress: PVOID,
    Length: SIZE_T,
    Protect: ULONG,
    OldProtect: *mut ULONG,
) -> NTSTATUS {
    let Region: PMM_REGION;
    let mut Status: NTSTATUS;
    let mut MaxLength: ULONG_PTR;
    let mut Length = Length;

    if (*MemoryArea).DeleteInProgress != 0 {
        return STATUS_UNSUCCESSFUL;
    }

    MaxLength = MA_GetEndingAddress(MemoryArea) - (BaseAddress as ULONG_PTR);
    if Length > MaxLength {
        Length = MaxLength as SIZE_T;
    }

    Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        BaseAddress,
        core::ptr::null_mut(),
    );

    if ((*MemoryArea).Flags & SEC_NO_CHANGE) != 0 && (*Region).Protect != Protect {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    if !OldProtect.is_null() {
        core::ptr::write_volatile(OldProtect, (*Region).Protect);
    }
    Status = MmAlterRegion(
        AddressSpace,
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        BaseAddress,
        Length,
        (*Region).Type,
        Protect,
        MmAlterViewAttributes,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmQuerySectionView(
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    Info: *mut MEMORY_BASIC_INFORMATION,
    ResultLength: *mut SIZE_T,
) -> NTSTATUS {
    let mut RegionBaseAddress: PVOID = core::ptr::null_mut();
    let Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        Address,
        &mut RegionBaseAddress,
    );
    if Region.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*MemoryArea).VadNode.u.VadFlags.VadType == VadImageMap {
        let Segment = (*MemoryArea).SectionData.Segment;
        let ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        let SectionSegments = (*ImageSectionObject).Segments;
        (*Info).AllocationBase = ((MA_GetStartingAddress(MemoryArea) as ULONG_PTR)
            - (*SectionSegments).Image.VirtualAddress) as PVOID;
        (*Info).Type = MEM_IMAGE;
    } else {
        (*Info).AllocationBase = MA_GetStartingAddress(MemoryArea) as PVOID;
        (*Info).Type = MEM_MAPPED;
    }
    (*Info).BaseAddress = RegionBaseAddress;
    (*Info).AllocationProtect = MmProtectToValue[(*MemoryArea).VadNode.u.VadFlags.Protection as usize];
    (*Info).RegionSize = (*Region).Length;
    (*Info).State = MEM_COMMIT;
    (*Info).Protect = (*Region).Protect;

    core::ptr::write_volatile(ResultLength, core::mem::size_of::<MEMORY_BASIC_INFORMATION>());
    STATUS_SUCCESS
}

static unsafe fn MmCreateDataFileSection(
    SectionObject: *mut *mut SECTION,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    UMaximumSize: *mut LARGE_INTEGER,
    SectionPageProtection: ULONG,
    AllocationAttributes: ULONG,
    FileObject: *mut FILE_OBJECT,
    GotFileHandle: BOOLEAN,
) -> NTSTATUS {
    let mut Section: *mut SECTION;
    let mut Status: NTSTATUS;
    let mut MaximumSize = LARGE_INTEGER { QuadPart: 0 };
    let mut Segment: *mut MM_SECTION_SEGMENT;
    let mut OldIrql: KIRQL;

    Status = ObCreateObject(
        ExGetPreviousMode(),
        MmSectionObjectType,
        ObjectAttributes,
        ExGetPreviousMode(),
        core::ptr::null_mut(),
        core::mem::size_of::<SECTION>() as ULONG,
        0,
        0,
        &mut Section as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    RtlZeroMemory(Section as PVOID, core::mem::size_of::<SECTION>());
    (*Section).u.Flags.filler = 1;
    (*Section).InitialPageProtection = SectionPageProtection;
    (*Section).u.Flags.File = 1;

    if (AllocationAttributes & SEC_NO_CHANGE) != 0 {
        (*Section).u.Flags.NoChange = 1;
    }

    if GotFileHandle == 0 {
        MaximumSize = *UMaximumSize;
    } else {
        let mut FileSize = LARGE_INTEGER { QuadPart: 0 };
        Status = FsRtlGetFileSize(FileObject, &mut FileSize);
        if Status < 0 {
            ObDereferenceObject(Section as PVOID);
            return Status;
        }

        if !UMaximumSize.is_null() && (*UMaximumSize).QuadPart != 0 {
            MaximumSize = *UMaximumSize;
        } else {
            MaximumSize = FileSize;
            if MaximumSize.QuadPart == 0 {
                ObDereferenceObject(Section as PVOID);
                return STATUS_MAPPED_FILE_SIZE_ZERO;
            }
        }

        if MaximumSize.QuadPart > FileSize.QuadPart {
            Status = IoSetInformation(
                FileObject,
                FileEndOfFileInformation,
                core::mem::size_of::<LARGE_INTEGER>() as ULONG,
                &mut MaximumSize as *mut _ as PVOID,
            );
            if Status < 0 {
                ObDereferenceObject(Section as PVOID);
                return STATUS_SECTION_NOT_EXTENDED;
            }
        }
    }

    if (*FileObject).SectionObjectPointer.is_null() {
        ObDereferenceObject(Section as PVOID);
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    Status = MmspWaitForFileLock(FileObject);
    if Status != STATUS_SUCCESS {
        ObDereferenceObject(Section as PVOID);
        return Status;
    }

    'grab_segment: loop {
        // [guard] OldIrql = MiAcquirePfnLock();
        let mut pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        Segment = (*(*FileObject).SectionObjectPointer).DataSectionObject;

        while !Segment.is_null() && ((*Segment).SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)) != 0 {
            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, &TinyTime);
            // [guard] OldIrql = MiAcquirePfnLock();
            pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            Segment = (*(*FileObject).SectionObjectPointer).DataSectionObject;
        }

        if Segment.is_null() {
            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);

            Segment = ExAllocatePoolWithTag(
                NonPagedPool,
                core::mem::size_of::<MM_SECTION_SEGMENT>() as SIZE_T,
                TAG_MM_SECTION_SEGMENT,
            ) as *mut MM_SECTION_SEGMENT;
            if Segment.is_null() {
                ObDereferenceObject(Section as PVOID);
                return STATUS_NO_MEMORY;
            }

            RtlZeroMemory(Segment as PVOID, core::mem::size_of::<MM_SECTION_SEGMENT>());
            (*Segment).SegFlags = MM_DATAFILE_SEGMENT | MM_SEGMENT_INCREATE;
            (*Segment).RefCount = 1;

            // [guard] OldIrql = MiAcquirePfnLock();
            let pfn_guard2 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

            if !(*(*FileObject).SectionObjectPointer).DataSectionObject.is_null() {
                drop(pfn_guard2); // [guard] MiReleasePfnLock(OldIrql);
                ExFreePoolWithTag(Segment as PVOID, TAG_MM_SECTION_SEGMENT);
                continue 'grab_segment;
            }

            (*(*FileObject).SectionObjectPointer).DataSectionObject = Segment;
            drop(pfn_guard2); // [guard] MiReleasePfnLock(OldIrql);

            (*Section).Segment = Segment as PSEGMENT;
            (*Segment).Flags = core::ptr::addr_of_mut!((*Segment).SegFlags);
            (*Segment).ReferenceCount = core::ptr::addr_of_mut!((*Segment).RefCount);
            (*Segment).SectionCount = 1;

            ExInitializeFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
            (*Segment).FileObject = FileObject;
            ObReferenceObject(FileObject as PVOID);

            (*Segment).Image.FileOffset = 0;
            (*Segment).Protection = SectionPageProtection;
            (*Segment).Image.Characteristics = 0;
            (*Segment).WriteCopy = if (SectionPageProtection & (PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY)) != 0 { 1u8 } else { 0u8 };

            if (AllocationAttributes & SEC_RESERVE) != 0 {
                (*Segment).Length.QuadPart = 0;
                (*Segment).RawLength.QuadPart = 0;
            } else {
                (*Segment).RawLength.QuadPart = MaximumSize.QuadPart;
                (*Segment).Length.QuadPart = PAGE_ROUND_UP((*Segment).RawLength.QuadPart as ULONG_PTR) as i64;
            }
            (*Segment).Image.VirtualAddress = 0;
            MiInitializeSectionPageTable(Segment);

            // [guard] OldIrql = MiAcquirePfnLock();
            let pfn_guard3 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            (*Segment).SegFlags &= !MM_SEGMENT_INCREATE;
            drop(pfn_guard3); // [guard] MiReleasePfnLock(OldIrql);
            break;
        } else {
            (*Section).Segment = Segment as PSEGMENT;
            InterlockedIncrement64(core::ptr::addr_of_mut!((*Segment).RefCount) as *mut i64);
            InterlockedIncrementUL(core::ptr::addr_of_mut!((*Segment).SectionCount));

            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);

            MmLockSectionSegment(Segment);

            if MaximumSize.QuadPart > (*Segment).RawLength.QuadPart
                && (AllocationAttributes & SEC_RESERVE) == 0 {
                (*Segment).RawLength.QuadPart = MaximumSize.QuadPart;
                (*Segment).Length.QuadPart = PAGE_ROUND_UP((*Segment).RawLength.QuadPart as ULONG_PTR) as i64;
            }

            MmUnlockSectionSegment(Segment);
            break;
        }
    }

    (*Section).SizeOfSection = MaximumSize;
    *SectionObject = Section;
    STATUS_SUCCESS
}

extern "system" {
    fn PeFmtCreateSection( FileHeader: *const VOID, FileHeaderSize: SIZE_T, File: PVOID, ImageSectionObject: *mut MM_IMAGE_SECTION_OBJECT, Flags: *mut ULONG, ReadFileCb: PEXEFMT_CB_READ_FILE, AllocateSegmentsCb: PEXEFMT_CB_ALLOCATE_SEGMENTS, ) -> NTSTATUS;
}

static mut ExeFmtpLoaders: [unsafe extern "system" fn(*const VOID, SIZE_T, PVOID, *mut MM_IMAGE_SECTION_OBJECT, *mut ULONG, PEXEFMT_CB_READ_FILE, PEXEFMT_CB_ALLOCATE_SEGMENTS) -> NTSTATUS; 1] = [
    PeFmtCreateSection,
];

static unsafe fn ExeFmtpAllocateSegments(NrSegments: ULONG) -> *mut MM_SECTION_SEGMENT {
    let SizeOfSegments = (core::mem::size_of::<MM_SECTION_SEGMENT>() * NrSegments as usize) as SIZE_T;
    let Segments = ExAllocatePoolWithTag(
        NonPagedPool,
        SizeOfSegments,
        TAG_MM_SECTION_SEGMENT,
    ) as *mut MM_SECTION_SEGMENT;

    if !Segments.is_null() {
        RtlZeroMemory(Segments as PVOID, SizeOfSegments);
    }

    Segments
}

static unsafe fn ExeFmtpReadFile(
    File: PVOID,
    Offset: *mut LARGE_INTEGER,
    Length: ULONG,
    Data: *mut PVOID,
    AllocBase: *mut PVOID,
    ReadSize: *mut ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut FileOffset = *Offset;
    let AdjustOffset: ULONG;
    let OffsetAdjustment: ULONG;
    let mut BufferSize: ULONG;
    let UsedSize: ULONG;
    let Buffer: PVOID;
    let FileObject = File as *mut FILE_OBJECT;
    let mut Iosb = _IO_STATUS_BLOCK { __bindgen_anon_1: unsafe { core::mem::zeroed() }, Information: 0 };

    if Length == 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if FileOffset.u.__bindgen_anon_1.HighPart < 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    AdjustOffset = PAGE_ROUND_DOWN(FileOffset.u.__bindgen_anon_1.LowPart);
    OffsetAdjustment = FileOffset.u.__bindgen_anon_1.LowPart - AdjustOffset;
    FileOffset.u.__bindgen_anon_1.LowPart = AdjustOffset;

    BufferSize = Length + OffsetAdjustment;
    BufferSize = PAGE_ROUND_UP(BufferSize);

    Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize as SIZE_T, 0x724D6D4D);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = MiSimpleRead(FileObject, &mut FileOffset, Buffer, BufferSize, 1u8, &mut Iosb);
    let UsedSize = Iosb.Information as ULONG;

    if Status >= 0 && UsedSize < OffsetAdjustment {
        Status = STATUS_IN_PAGE_ERROR;
    }

    if Status >= 0 {
        *Data = ((Buffer as ULONG_PTR) + OffsetAdjustment as ULONG_PTR) as PVOID;
        *AllocBase = Buffer;
        *ReadSize = UsedSize - OffsetAdjustment;
    } else {
        ExFreePoolWithTag(Buffer, 0x724D6D4D);
    }

    Status
}

unsafe fn ExeFmtpCreateImageSection(
    FileObject: *mut FILE_OBJECT,
    ImageSectionObject: *mut MM_IMAGE_SECTION_OBJECT,
) -> NTSTATUS {
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let mut FileHeader: PVOID = core::ptr::null_mut();
    let mut FileHeaderBuffer: PVOID = core::ptr::null_mut();
    let mut FileHeaderSize: ULONG = 0;
    let mut Flags: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    Offset.QuadPart = 0;

    Status = ExeFmtpReadFile(
        FileObject as PVOID,
        &mut Offset,
        PAGE_SIZE * 2,
        &mut FileHeader,
        &mut FileHeaderBuffer,
        &mut FileHeaderSize,
    );

    if Status < 0 {
        return Status;
    }

    if FileHeaderSize == 0 {
        ExFreePool(FileHeaderBuffer);
        return STATUS_UNSUCCESSFUL;
    }

    i = 0;
    Status = STATUS_ROS_EXEFMT_UNKNOWN_FORMAT;
    while i < ExeFmtpLoaders.len() as ULONG {
        Flags = 0;

        Status = ExeFmtpLoaders[i as usize](
            FileHeader,
            FileHeaderSize as SIZE_T,
            FileObject as PVOID,
            ImageSectionObject,
            &mut Flags,
            ExeFmtpReadFile,
            ExeFmtpAllocateSegments,
        );

        if Status < 0 {
            if !(*ImageSectionObject).Segments.is_null() {
                ExFreePool((*ImageSectionObject).Segments as PVOID);
                (*ImageSectionObject).Segments = core::ptr::null_mut();
            }
        }

        if Status != STATUS_ROS_EXEFMT_UNKNOWN_FORMAT {
            break;
        }
        i += 1;
    }

    ExFreePoolWithTag(FileHeaderBuffer, 0x724D6D4D);

    if Status == STATUS_ROS_EXEFMT_UNKNOWN_FORMAT {
        Status = STATUS_INVALID_IMAGE_NOT_MZ;
    }

    if Status < 0 {
        return Status;
    }

    if (*ImageSectionObject).ImageInformation.MaximumStackSize == 0 {
        (*ImageSectionObject).ImageInformation.MaximumStackSize = 0x40000;
    }

    if (*ImageSectionObject).ImageInformation.CommittedStackSize == 0 {
        (*ImageSectionObject).ImageInformation.CommittedStackSize = 0x1000;
    }

    if (*ImageSectionObject).BasedAddress.is_null() {
        if ((*ImageSectionObject).ImageInformation.ImageCharacteristics & IMAGE_FILE_DLL) != 0 {
            (*ImageSectionObject).BasedAddress = 0x10000000 as PVOID;
        } else {
            (*ImageSectionObject).BasedAddress = 0x00400000 as PVOID;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn MmCreateImageSection(
    SectionObject: *mut *mut SECTION,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    UMaximumSize: *mut LARGE_INTEGER,
    SectionPageProtection: ULONG,
    AllocationAttributes: ULONG,
    FileObject: *mut FILE_OBJECT,
) -> NTSTATUS {
    let mut Section: *mut SECTION;
    let mut Status: NTSTATUS;
    let mut ImageSectionObject: *mut MM_IMAGE_SECTION_OBJECT;
    let mut OldIrql: KIRQL;

    if FileObject.is_null() {
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    if (*FileObject).SectionObjectPointer.is_null() {
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    Status = ObCreateObject(
        ExGetPreviousMode(),
        MmSectionObjectType,
        ObjectAttributes,
        ExGetPreviousMode(),
        core::ptr::null_mut(),
        core::mem::size_of::<SECTION>() as ULONG,
        0,
        0,
        &mut Section as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    RtlZeroMemory(Section as PVOID, core::mem::size_of::<SECTION>());
    (*Section).u.Flags.filler = 1;
    (*Section).InitialPageProtection = SectionPageProtection;
    (*Section).SizeOfSection = *UMaximumSize;
    (*Section).u.Flags.File = 1;
    (*Section).u.Flags.Image = 1;
    if (AllocationAttributes & SEC_NO_CHANGE) != 0 {
        (*Section).u.Flags.NoChange = 1;
    }

    'grab_image_section_object: loop {
        // [guard] OldIrql = MiAcquirePfnLock();
        let mut pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

        ImageSectionObject = (*(*FileObject).SectionObjectPointer).ImageSectionObject;
        while !ImageSectionObject.is_null() && ((*ImageSectionObject).SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)) != 0 {
            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, &TinyTime);
            // [guard] OldIrql = MiAcquirePfnLock();
            pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            ImageSectionObject = (*(*FileObject).SectionObjectPointer).ImageSectionObject;
        }

        if ImageSectionObject.is_null() {
            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);

            ImageSectionObject = ExAllocatePoolZero(
                NonPagedPool,
                core::mem::size_of::<MM_IMAGE_SECTION_OBJECT>() as SIZE_T,
                TAG_MM_SECTION_SEGMENT,
            ) as *mut MM_IMAGE_SECTION_OBJECT;
            if ImageSectionObject.is_null() {
                ObDereferenceObject(Section as PVOID);
                return STATUS_NO_MEMORY;
            }

            (*ImageSectionObject).SegFlags = MM_SEGMENT_INCREATE;
            (*ImageSectionObject).RefCount = 1;
            (*ImageSectionObject).SectionCount = 1;

            // [guard] OldIrql = MiAcquirePfnLock();
            let pfn_guard2 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            if !(*(*FileObject).SectionObjectPointer).ImageSectionObject.is_null() {
                drop(pfn_guard2); // [guard] MiReleasePfnLock(OldIrql);
                ExFreePoolWithTag(ImageSectionObject as PVOID, TAG_MM_SECTION_SEGMENT);
                continue 'grab_image_section_object;
            }

            (*(*FileObject).SectionObjectPointer).ImageSectionObject = ImageSectionObject;
            drop(pfn_guard2); // [guard] MiReleasePfnLock(OldIrql);

            CcFlushCache((*FileObject).SectionObjectPointer, core::ptr::null_mut(), 0, core::ptr::null_mut());

            let StatusExeFmt = ExeFmtpCreateImageSection(FileObject, ImageSectionObject);

            if StatusExeFmt < 0 {
                // [guard] OldIrql = MiAcquirePfnLock();
                let pfn_guard3 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
                (*(*FileObject).SectionObjectPointer).ImageSectionObject = core::ptr::null_mut();
                drop(pfn_guard3); // [guard] MiReleasePfnLock(OldIrql);

                if !(*ImageSectionObject).Segments.is_null() {
                    ExFreePool((*ImageSectionObject).Segments as PVOID);
                }

                Status = if StatusExeFmt == STATUS_END_OF_FILE {
                    STATUS_INVALID_FILE_FOR_SECTION
                } else {
                    StatusExeFmt
                };

                ExFreePoolWithTag(ImageSectionObject as PVOID, TAG_MM_SECTION_SEGMENT);
                ObDereferenceObject(Section as PVOID);
                return Status;
            }

            (*Section).Segment = ImageSectionObject as PSEGMENT;

            Status = MmspWaitForFileLock(FileObject);
            if Status < 0 {
                // [guard] OldIrql = MiAcquirePfnLock();
                let pfn_guard4 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
                (*(*FileObject).SectionObjectPointer).ImageSectionObject = core::ptr::null_mut();
                drop(pfn_guard4); // [guard] MiReleasePfnLock(OldIrql);

                ExFreePool((*ImageSectionObject).Segments as PVOID);
                ExFreePool(ImageSectionObject as PVOID);
                ObDereferenceObject(Section as PVOID);
                return Status;
            }

            // [guard] OldIrql = MiAcquirePfnLock();
            let pfn_guard5 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            (*ImageSectionObject).SegFlags &= !MM_SEGMENT_INCREATE;
            ObReferenceObject(FileObject as PVOID);
            drop(pfn_guard5); // [guard] MiReleasePfnLock(OldIrql);

            Status = StatusExeFmt;
            break;
        } else {
            (*ImageSectionObject).SegFlags &= !MM_IMAGE_SECTION_FLUSH_DELETE;
            InterlockedIncrement64(core::ptr::addr_of_mut!((*ImageSectionObject).RefCount) as *mut i64);
            (*ImageSectionObject).SectionCount += 1;

            drop(pfn_guard); // [guard] MiReleasePfnLock(OldIrql);

            (*Section).Segment = ImageSectionObject as PSEGMENT;
            Status = STATUS_SUCCESS;
            break;
        }
    }

    if (*Section).SizeOfSection.QuadPart == 0 {
        (*Section).SizeOfSection.QuadPart = (*ImageSectionObject).ImageInformation.ImageFileSize as i64;
    }

    core::ptr::write_volatile(SectionObject, Section);
    Status
}

static unsafe fn MmMapViewOfSegment(
    AddressSpace: PMMSUPPORT,
    AsImage: BOOLEAN,
    Segment: *mut MM_SECTION_SEGMENT,
    BaseAddress: *mut PVOID,
    ViewSize: SIZE_T,
    Protect: ULONG,
    ViewOffset: i64,
    AllocationType: ULONG,
) -> NTSTATUS {
    let mut MArea: *mut MEMORY_AREA;
    let mut Status: NTSTATUS;
    let Granularity: ULONG;
    let mut Protect = Protect;

    if (*Segment).WriteCopy != 0 {
        if Protect == PAGE_WRITECOPY {
            Protect = PAGE_READWRITE;
        } else if Protect == PAGE_EXECUTE_WRITECOPY {
            Protect = PAGE_EXECUTE_READWRITE;
        }
    }

    Granularity = if (*BaseAddress).is_null() {
        MM_ALLOCATION_GRANULARITY
    } else {
        PAGE_SIZE
    };

    Status = MmCreateMemoryArea(
        AddressSpace,
        MEMORY_AREA_SECTION_VIEW,
        BaseAddress,
        ViewSize,
        Protect,
        &mut MArea,
        AllocationType,
        Granularity,
    );
    if Status < 0 {
        return Status;
    }

    InterlockedIncrement64((*Segment).ReferenceCount as *mut i64);

    (*MArea).SectionData.Segment = Segment;
    (*MArea).SectionData.ViewOffset = ViewOffset;
    if AsImage != 0 {
        (*MArea).VadNode.u.VadFlags.VadType = VadImageMap;
    }

    MmInitializeRegion(
        core::ptr::addr_of_mut!((*MArea).SectionData.RegionListHead),
        ViewSize,
        0,
        Protect,
    );

    STATUS_SUCCESS
}

static unsafe fn MmFreeSectionPage(
    Context: PVOID,
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    Page: PFN_NUMBER,
    SwapEntry: SWAPENTRY,
    Dirty: BOOLEAN,
) {
    let mut Entry: ULONG_PTR;
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let mut SavedSwapEntry: SWAPENTRY;
    let Segment: *mut MM_SECTION_SEGMENT;
    let AddressSpace = Context as PMMSUPPORT;
    let Process = MmGetAddressSpaceOwner(AddressSpace);

    let Address = PAGE_ROUND_DOWN(Address as ULONG_PTR) as PVOID;

    Offset.QuadPart = ((Address as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64)
        + (*MemoryArea).SectionData.ViewOffset;

    Segment = (*MemoryArea).SectionData.Segment;

    Entry = MmGetPageEntrySectionSegment(Segment, &mut Offset);
    while Entry != 0 && MM_IS_WAIT_PTE(Entry) {
        MmUnlockSectionSegment(Segment);
        MmUnlockAddressSpace(AddressSpace);
        KeDelayExecutionThread(0i8, 0u8, &TinyTime);
        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment);
        Entry = MmGetPageEntrySectionSegment(Segment, &mut Offset);
    }

    if SwapEntry != 0 {
        MmFreeSwapPage(SwapEntry);
    } else if Page != 0 {
        if IS_SWAP_FROM_SSE(Entry) != 0 || Page != PFN_FROM_SSE(Entry) {
            SavedSwapEntry = MmGetSavedSwapEntryPage(Page);
            if SavedSwapEntry != 0 {
                MmFreeSwapPage(SavedSwapEntry);
                MmSetSavedSwapEntryPage(Page, 0);
            }
            MmDeleteRmap(Page, Process, Address);
            MmReleasePageMemoryConsumer(MC_USER, Page);
        } else {
            if !Process.is_null() {
                MmDeleteRmap(Page, Process, Address);
            }
            MmUnsharePageEntrySectionSegment(MemoryArea, Segment, &mut Offset, if !Process.is_null() { Dirty } else { 0u8 }, 0u8, core::ptr::null_mut());
        }
    }
}

static unsafe fn MmUnmapViewOfSegment(
    AddressSpace: PMMSUPPORT,
    BaseAddress: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let MemoryArea: *mut MEMORY_AREA;
    let Segment: *mut MM_SECTION_SEGMENT;
    let mut CurrentEntry: PLIST_ENTRY;
    let CurrentRegion: *mut MM_REGION;
    let RegionListHead: PLIST_ENTRY;

    MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, BaseAddress);
    if MemoryArea.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Segment = (*MemoryArea).SectionData.Segment;

    (*MemoryArea).DeleteInProgress = 1u8;

    MmLockSectionSegment(Segment);

    RegionListHead = core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead);
    while IsListEmpty(RegionListHead) == 0 {
        CurrentEntry = RemoveHeadList(RegionListHead);
        CurrentRegion = CurrentEntry as *mut MM_REGION;
        ExFreePoolWithTag(CurrentRegion as PVOID, TAG_MM_REGION);
    }

    if ((*(*Segment).Flags) & MM_PHYSICALMEMORY_SEGMENT) != 0 {
        Status = MmFreeMemoryArea(AddressSpace, MemoryArea, core::ptr::null_mut(), core::ptr::null_mut());
    } else {
        Status = MmFreeMemoryArea(AddressSpace, MemoryArea, MmFreeSectionPage, AddressSpace as PVOID);
    }
    MmUnlockSectionSegment(Segment);
    MmDereferenceSegment(Segment);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiRosUnmapViewOfSection(
    Process: PEPROCESS,
    MemoryArea: *mut MEMORY_AREA,
    BaseAddress: PVOID,
    SkipDebuggerNotify: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let AddressSpace: PMMSUPPORT;
    let mut ImageBaseAddress: PVOID = core::ptr::null_mut();

    AddressSpace = &mut (*(Process as *mut _EPROCESS)).Vm;

    if MemoryArea.is_null() || (*MemoryArea).DeleteInProgress != 0 {
        return STATUS_NOT_MAPPED_VIEW;
    }

    if (*MemoryArea).VadNode.u.VadFlags.VadType == VadImageMap {
        let mut i: ULONG;
        let NrSegments: ULONG;
        let ImageSectionObject: *mut MM_IMAGE_SECTION_OBJECT;
        let SectionSegments: *mut MM_SECTION_SEGMENT;
        let Segment: *mut MM_SECTION_SEGMENT;

        Segment = (*MemoryArea).SectionData.Segment;
        ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        SectionSegments = (*ImageSectionObject).Segments;
        NrSegments = (*ImageSectionObject).NrSegments;

        (*MemoryArea).DeleteInProgress = 1u8;

        i = 0;
        while i < NrSegments {
            if Segment == &mut (*SectionSegments.add(i as usize)) {
                ImageBaseAddress = ((BaseAddress as ULONG_PTR) - (*SectionSegments.add(i as usize)).Image.VirtualAddress) as PVOID;
                break;
            }
            i += 1;
        }

        i = 0;
        while i < NrSegments {
            let SBaseAddress = ((ImageBaseAddress as ULONG_PTR) + (*SectionSegments.add(i as usize)).Image.VirtualAddress) as PVOID;

            Status = MmUnmapViewOfSegment(AddressSpace, SBaseAddress);
            if Status < 0 {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
            i += 1;
        }

        let MapCount = InterlockedDecrement(core::ptr::addr_of_mut!((*ImageSectionObject).MapCount));
        if MapCount != 0 {
            ImageBaseAddress = core::ptr::null_mut();
        }
    } else {
        let Segment = (*MemoryArea).SectionData.Segment;
        InterlockedIncrement64((*Segment).ReferenceCount as *mut i64);

        Status = MmUnmapViewOfSegment(AddressSpace, BaseAddress);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        if !FlagOn((*(*Segment).Flags), MM_PHYSICALMEMORY_SEGMENT) {
            let FileObject = (*Segment).FileObject;
            FsRtlAcquireFileExclusive(FileObject);
            FsRtlReleaseFile(FileObject);
        }

        MmDereferenceSegment(Segment);
    }

    if !ImageBaseAddress.is_null() && SkipDebuggerNotify == 0 {
        DbgkUnMapViewOfSection(ImageBaseAddress);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySection(
    SectionHandle: HANDLE,
    SectionInformationClass: SECTION_INFORMATION_CLASS,
    SectionInformation: PVOID,
    SectionInformationLength: SIZE_T,
    ResultLength: *mut SIZE_T,
) -> NTSTATUS {
    let mut Section: *mut SECTION;
    let PreviousMode: KPROCESSOR_MODE;
    let mut Status: NTSTATUS;

    PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0i8 {
        ProbeForWrite(SectionInformation, SectionInformationLength as ULONG, 4);
        if !ResultLength.is_null() {
            ProbeForWrite(ResultLength as PVOID, core::mem::size_of::<SIZE_T>() as ULONG, core::mem::align_of::<SIZE_T>() as ULONG);
        }
    }

    if SectionInformationClass == SectionBasicInformation {
        if SectionInformationLength < core::mem::size_of::<SECTION_BASIC_INFORMATION>() {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    } else if SectionInformationClass == SectionImageInformation {
        if SectionInformationLength < core::mem::size_of::<SECTION_IMAGE_INFORMATION>() {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
    } else {
        return STATUS_INVALID_INFO_CLASS;
    }

    Status = ObReferenceObjectByHandle(
        SectionHandle,
        SECTION_QUERY,
        MmSectionObjectType,
        PreviousMode,
        &mut Section as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    match SectionInformationClass {
        SectionBasicInformation => {
            let mut Sbi = SECTION_BASIC_INFORMATION { Size: LARGE_INTEGER { QuadPart: 0 }, BaseAddress: core::ptr::null_mut(), Attributes: 0 };

            Sbi.Size = (*Section).SizeOfSection;
            Sbi.BaseAddress = core::ptr::null_mut();

            if (*Section).u.Flags.File != 0 { Sbi.Attributes |= SEC_FILE; }
            if (*Section).u.Flags.Image != 0 { Sbi.Attributes |= SEC_IMAGE; }
            if (*Section).u.Flags.Commit != 0 { Sbi.Attributes |= SEC_COMMIT; }
            if (*Section).u.Flags.Reserve != 0 { Sbi.Attributes |= SEC_RESERVE; }
            if (*Section).u.Flags.Based != 0 { Sbi.Attributes |= SEC_BASED; }

            *(SectionInformation as *mut SECTION_BASIC_INFORMATION) = Sbi;
            if !ResultLength.is_null() {
                core::ptr::write_volatile(ResultLength, core::mem::size_of::<SECTION_BASIC_INFORMATION>());
            }
        }
        SectionImageInformation => {
            if (*Section).u.Flags.Image == 0 {
                Status = STATUS_SECTION_NOT_IMAGE;
            } else if MiIsRosSectionObject(Section as PVOID) != 0 {
                let ImageSectionObject = (*Section).Segment as *mut MM_IMAGE_SECTION_OBJECT;
                let Sii = SectionInformation as *mut SECTION_IMAGE_INFORMATION;
                *Sii = (*ImageSectionObject).ImageInformation;
                if !ResultLength.is_null() {
                    core::ptr::write_volatile(ResultLength, core::mem::size_of::<SECTION_IMAGE_INFORMATION>());
                }
            }
        }
        _ => Status = STATUS_NOT_SUPPORTED,
    }

    ObDereferenceObject(Section as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmMapViewOfSection(
    SectionObject: PVOID,
    Process: PEPROCESS,
    BaseAddress: *mut PVOID,
    ZeroBits: ULONG_PTR,
    CommitSize: SIZE_T,
    SectionOffset: *mut LARGE_INTEGER,
    ViewSize: *mut SIZE_T,
    InheritDisposition: SECTION_INHERIT,
    AllocationType: ULONG,
    Protect: ULONG,
) -> NTSTATUS {
    let Section: *mut SECTION;
    let AddressSpace: PMMSUPPORT;
    let mut Status = STATUS_SUCCESS;
    let mut NotAtBase = 0u8;
    let mut IsAttached = 0u8;
    let mut ApcState = _KAPC_STATE { ApcListHead: [LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() }; 2], Process: core::ptr::null_mut(), InProgressFlags: 0, KernelApcInProgress: 0, KernelApcPending: 0, UserApcPending: 0 };

    if MiIsRosSectionObject(SectionObject) == 0 {
        return MmMapViewOfArm3Section(
            SectionObject, Process, BaseAddress, ZeroBits, CommitSize,
            SectionOffset, ViewSize, InheritDisposition, AllocationType, Protect,
        );
    }

    if Protect == 0 || (Protect & !PAGE_FLAGS_VALID_FOR_SECTION) != 0 {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    if PsGetCurrentProcess() != Process {
        KeStackAttachProcess(&mut (*(Process as *mut _EPROCESS)).Pcb, &mut ApcState);
        IsAttached = 1u8;
    }

    Section = SectionObject as *mut SECTION;
    AddressSpace = &mut (*(Process as *mut _EPROCESS)).Vm;

    MmLockAddressSpace(AddressSpace);

    if (*Section).u.Flags.Image != 0 {
        let ImageSectionObject = (*Section).Segment as *mut MM_IMAGE_SECTION_OBJECT;
        let SectionSegments = (*ImageSectionObject).Segments;
        let NrSegments = (*ImageSectionObject).NrSegments;

        let mut ImageBase = (*BaseAddress) as ULONG_PTR;
        if ImageBase == 0 {
            ImageBase = (*ImageSectionObject).BasedAddress as ULONG_PTR;
        }

        let mut ImageSize: SIZE_T = 0;
        let mut i = 0;
        while i < NrSegments {
            let MaxExtent = (*SectionSegments.add(i as usize)).Image.VirtualAddress + (*SectionSegments.add(i as usize)).Length.QuadPart as ULONG_PTR;
            if MaxExtent > ImageSize { ImageSize = MaxExtent; }
            i += 1;
        }

        (*ImageSectionObject).ImageInformation.ImageFileSize = ImageSize as ULONG;

        if ((ImageBase + ImageSize) > MM_HIGHEST_VAD_ADDRESS as ULONG_PTR) || ((ImageBase + ImageSize) < ImageSize) {
            ImageBase = ALIGN_DOWN_BY(MM_HIGHEST_VAD_ADDRESS as ULONG_PTR - ImageSize, MM_VIRTMEM_GRANULARITY);
            NotAtBase = 1u8;
        }

        i = 0;
        while i < NrSegments {
            let SBaseAddress = (ImageBase + (*SectionSegments.add(i as usize)).Image.VirtualAddress) as PVOID;
            MmLockSectionSegment(&mut (*SectionSegments.add(i as usize)));
            Status = MmMapViewOfSegment(
                AddressSpace, 1u8, &mut (*SectionSegments.add(i as usize)),
                &mut (SBaseAddress as *mut PVOID) as *mut PVOID, (*SectionSegments.add(i as usize)).Length.QuadPart as SIZE_T,
                (*SectionSegments.add(i as usize)).Protection, 0, 0,
            );
            MmUnlockSectionSegment(&mut (*SectionSegments.add(i as usize)));
            if Status < 0 { break; }
            i += 1;
        }

        core::ptr::write_volatile(BaseAddress, ImageBase as PVOID);
        core::ptr::write_volatile(ViewSize, ImageSize);

        InterlockedIncrement(core::ptr::addr_of_mut!((*ImageSectionObject).MapCount));
    } else {
        let Segment = (*Section).Segment as *mut MM_SECTION_SEGMENT;
        let ViewOffset = if SectionOffset.is_null() { 0 } else { (*SectionOffset).QuadPart };

        // Handle ViewSize == 0: expand to section size minus offset
        if *ViewSize == 0 {
            let calculated_size = (*Section).SizeOfSection.QuadPart as u64 - ViewOffset as u64;
            if calculated_size > usize::MAX as u64 {
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(&mut ApcState);
                }
                return STATUS_INVALID_VIEW_SIZE;
            }
            core::ptr::write_volatile(ViewSize, calculated_size as SIZE_T);
        }

        // Page-round the view size
        core::ptr::write_volatile(ViewSize, ((*ViewSize + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE);

        MmLockSectionSegment(Segment);
        Status = MmMapViewOfSegment(AddressSpace, 0u8, Segment, BaseAddress, *ViewSize, Protect, ViewOffset, AllocationType);
        MmUnlockSectionSegment(Segment);
    }

    if NotAtBase != 0 { Status = STATUS_IMAGE_NOT_AT_BASE; }

    MmUnlockAddressSpace(AddressSpace);

    if IsAttached != 0 {
        KeUnstackDetachProcess(&mut ApcState);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiRosUnmapViewInSystemSpace(MappedBase: PVOID) -> NTSTATUS {
    MmUnmapViewOfSegment(MmGetKernelAddressSpace(), MappedBase)
}