use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

#[repr(C)]
struct _CACHE_SECTION_PAGE_TABLE {
    pub FileOffset: ntkrnl_sys::ddk::LARGE_INTEGER,
    pub Segment: ntkrnl_sys::ddk::PMM_SECTION_SEGMENT,
    pub Refcount: ULONG,
    pub PageEntries: [ULONG_PTR; 256],
}
type PCACHE_SECTION_PAGE_TABLE = *mut _CACHE_SECTION_PAGE_TABLE;

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _ETHREAD,
    _OBJECT_ATTRIBUTES,
    _UNICODE_STRING,
    _SECTION,
    _GENERIC_MAPPING,
    _IO_STATUS_BLOCK,
    _KAPC_STATE,
    _LARGE_INTEGER,
    _MEMORY_AREA,
    _MEMORY_BASIC_INFORMATION,
    _MM_IMAGE_SECTION_OBJECT,
    _MM_SECTION_SEGMENT,
    _MMSESSION,
    _RTL_AVL_TABLE,
    _FSRTL_COMMON_FCB_HEADER,
    _IMAGE_DOS_HEADER,
    _IMAGE_NT_HEADERS64,
    _IMAGE_OPTIONAL_HEADER64,
    _IMAGE_SECTION_HEADER,
    _OBJECT_TYPE_INITIALIZER,
    _FAST_MUTEX,
    _KEVENT,
    _MDL,
    _MMPTE,
    ACCESS_MASK,
    BOOLEAN,
    ExAcquireFastMutex,
    ExAllocatePoolZero,
    ExFreePool,
    ExFreePoolWithTag,
    ExGetPreviousMode,
    ExInitializeFastMutex,
    ExReleaseFastMutex,
    HANDLE,
    IoAllocateMdl,
    IoFreeMdl,
    IoGetTopLevelIrp,
    IoPageRead,
    IoSetInformation,
    IoSetTopLevelIrp,
    IoSynchronousPageWrite,
    KeBugCheck,
    KeBugCheckEx,
    KeDelayExecutionThread,
    KeInitializeEvent,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    KeWaitForSingleObject,
    KEVENT,
    KPROCESSOR_MODE,
    LARGE_INTEGER,
    MDL,
    MEMORY_AREA,
    MEMORY_BASIC_INFORMATION,
    MiDeleteARM3Section,
    MiIsRosSectionObject,
    MiMakeProtectionMask,
    MiMapPageInHyperSpace,
    MiUnmapPageInHyperSpace,
    MM_IMAGE_SECTION_OBJECT,
    MM_SECTION_SEGMENT,
    MMSESSION,
    MmDereferenceSegment,
    ObCloseHandle,
    ObCreateObject,
    ObCreateObjectType,
    ObInsertObject,
    ObReferenceObjectByHandle,
    PEPROCESS,
    PERESOURCE,
    PETHREAD,
    PFILE_OBJECT,
    PFSRTL_COMMON_FCB_HEADER,
    PIMAGE_DOS_HEADER,
    PIMAGE_NT_HEADERS64,
    PIMAGE_OPTIONAL_HEADER64,
    PIMAGE_SECTION_HEADER,
    PIO_STATUS_BLOCK,
    PIRP,
    PLARGE_INTEGER,
    PMEMORY_AREA,
    PMM_IMAGE_SECTION_OBJECT,
    PMM_REGION, _MM_REGION,
    PMM_SECTION_SEGMENT,
    POBJECT_ATTRIBUTES,
    POBJECT_TYPE,
    PSIZE_T,
    PSECTION,
    PSECTION_OBJECT_POINTERS,
    RTL_AVL_TABLE,
    SECTION_INHERIT,
    SECTION_INFORMATION_CLASS,
    SWAPENTRY,
    CcFlushCache,
    DbgkUnMapViewOfSection,
    RtlEnumerateGenericTable,
    RtlGetElementGenericTable,
    RtlNumberGenericTableElements,
};

// Types/functions not in bindgen — define locally
type _IMAGE_NT_HEADERS32 = _IMAGE_NT_HEADERS64; // stub alias
type PIMAGE_NT_HEADERS32 = *mut _IMAGE_NT_HEADERS32;
type _IMAGE_OPTIONAL_HEADER32 = _IMAGE_OPTIONAL_HEADER64; // stub alias
type PIMAGE_OPTIONAL_HEADER32 = *mut _IMAGE_OPTIONAL_HEADER32;

extern "system" {
    fn FsRtlAcquireFileExclusive(FileObject: PFILE_OBJECT);
    fn FsRtlAcquireFileForModWriteEx(
        FileObject: PFILE_OBJECT, EndingOffset: PLARGE_INTEGER,
        ResourceToRelease: *mut PERESOURCE,
    ) -> NTSTATUS;
    fn FsRtlGetFileSize(FileObject: PFILE_OBJECT, FileSize: PLARGE_INTEGER) -> NTSTATUS;
    fn FsRtlReleaseFile(FileObject: PFILE_OBJECT);
    fn FsRtlReleaseFileForModWrite(FileObject: PFILE_OBJECT, Resource: PERESOURCE);
    fn FsRtlAcquireToCreateMappedSection(FileObject: PFILE_OBJECT, SectionPageProtection: ULONG) -> NTSTATUS;
    fn ObDereferenceObject(Object: PVOID);
    fn MmFreePageTablesSectionSegment(
        Segment: PMM_SECTION_SEGMENT, FreePage: Option<unsafe extern "C" fn(PMM_SECTION_SEGMENT, *mut LARGE_INTEGER)>,
    );
    fn MiInitializeSectionPageTable(Segment: PMM_SECTION_SEGMENT);
    #[link_name = "MmGetPageEntrySectionSegment"]
    fn MmGetPageEntrySectionSegmentDdk(Segment: PVOID, Offset: *const u64) -> ULONG_PTR;
    fn MmBuildMdlFromPages(Mdl: *mut MDL, Pages: *const PFN_NUMBER);
    fn MmUnmapLockedPages(MappedSystemVa: PVOID, Mdl: *mut MDL);
    fn MmReleasePageMemoryConsumer(Type: ULONG, Page: ULONG_PTR) -> NTSTATUS;
    fn MmGetSavedSwapEntryPage(Page: ULONG_PTR) -> ULONG_PTR;
    fn MmSetSavedSwapEntryPage(Page: ULONG_PTR, SwapEntry: ULONG_PTR);
    fn MmFreeSwapPage(SwapEntry: ULONG_PTR);
}
extern "C" {
    fn ObReferenceObject(Object: PVOID) -> isize;
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

// Inline implementations for intrinsics/safe-math (no C symbols available)
#[inline]
unsafe fn InterlockedIncrement64(addend: *mut i64) -> i64 {
    let atom = &*(addend as *const core::sync::atomic::AtomicI64);
    atom.fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}

#[inline]
unsafe fn InterlockedDecrement64(addend: *mut i64) -> i64 {
    let atom = &*(addend as *const core::sync::atomic::AtomicI64);
    atom.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}

#[inline]
unsafe fn RtlLongLongSub(a: i64, b: i64, result: *mut i64) -> NTSTATUS {
    match a.checked_sub(b) {
        Some(v) => { *result = v; STATUS_SUCCESS }
        None => 0xC0000095_u32 as i32 // STATUS_INTEGER_OVERFLOW
    }
}

#[inline]
unsafe fn RtlLongLongAdd(a: i64, b: i64, result: *mut i64) -> NTSTATUS {
    match a.checked_add(b) {
        Some(v) => { *result = v; STATUS_SUCCESS }
        None => 0xC0000095_u32 as i32
    }
}

#[inline]
unsafe fn RtlLongLongToSIZET(a: i64, result: *mut usize) -> NTSTATUS {
    if a < 0 || a as u64 > usize::MAX as u64 {
        0xC0000095_u32 as i32
    } else {
        *result = a as usize;
        STATUS_SUCCESS
    }
}

// Extern declarations needed by tail functions (MmArePagesResident etc.)
// These are FORCEINLINE shims in mm_rust_shims.c — cdecl calling convention
extern "C" {
    fn MmGetKernelAddressSpace() -> PVOID;
    fn MmLockAddressSpace(AddressSpace: PVOID);
    fn MmUnlockAddressSpace(AddressSpace: PVOID);
}

#[inline]
fn PAGE_ROUND_UP(size: usize) -> usize {
    (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

extern "system" {
    fn MmLocateMemoryAreaByAddress(AddressSpace: PVOID, Address: PVOID) -> PMEMORY_AREA;
    fn MmAllocSwapPage() -> usize;
    fn MmWriteToSwapPage(SwapEntry: usize, Page: PFN_NUMBER) -> NTSTATUS;
}

unsafe fn _BitScanForward(index: *mut ULONG, mask: ULONG) -> u8 {
    if mask == 0 { return 0; }
    *index = mask.trailing_zeros();
    1
}

type PEXEFMT_CB_READ_FILE = unsafe extern "system" fn(
    PVOID, *mut LARGE_INTEGER, ULONG, *mut PVOID, *mut PVOID, *mut ULONG
) -> NTSTATUS;
type PEXEFMT_CB_ALLOCATE_SEGMENTS = unsafe extern "system" fn(ULONG) -> PMM_SECTION_SEGMENT;

extern "system" {
    fn PeFmtCreateSection( FileHeader: *const core::ffi::c_void, FileHeaderSize: usize, File: PVOID, ImageSectionObject: PMM_IMAGE_SECTION_OBJECT, Flags: *mut ULONG, ReadFileCb: PEXEFMT_CB_READ_FILE, AllocateSegmentsCb: PEXEFMT_CB_ALLOCATE_SEGMENTS ) -> NTSTATUS;
    fn MiSimpleRead( FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Buffer: PVOID, Length: ULONG, Paging: BOOLEAN, Iosb: *mut _IO_STATUS_BLOCK ) -> NTSTATUS;
    fn _MmSetPageEntrySectionSegment( Segment: PMM_SECTION_SEGMENT, Offset: PLARGE_INTEGER, Entry: ULONG_PTR, file: *const i8, line: i32 ) -> NTSTATUS;
    fn MmCreateArm3Section( SectionObject: *mut PVOID, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES, InputMaximumSize: PLARGE_INTEGER, SectionPageProtection: ULONG, AllocationAttributes: ULONG, FileHandle: HANDLE, FileObject: PFILE_OBJECT ) -> NTSTATUS;
    fn MmMapViewOfArm3Section( SectionObject: PVOID, Process: PEPROCESS, BaseAddress: *mut PVOID, ZeroBits: ULONG_PTR, CommitSize: SIZE_T, SectionOffset: PLARGE_INTEGER, ViewSize: PSIZE_T, InheritDisposition: SECTION_INHERIT, AllocationType: ULONG, Protect: ULONG ) -> NTSTATUS;
    static mut MmSession: MMSESSION;
    static mut MmSectionBasedRoot: RTL_AVL_TABLE;
    static mut MmSubsectionBase: ULONG_PTR;
    static mut MmHighestPhysicalPage: PFN_NUMBER;
    static mut IoFileObjectType: POBJECT_TYPE;
    static mut MmMakeFileAccess: [ACCESS_MASK; 8];
}
extern "C" {
    fn MiMapViewInSystemSpace( Section: PVOID, Session: PVOID, MappedBase: *mut PVOID, ViewSize: PSIZE_T, SectionOffset: PLARGE_INTEGER ) -> NTSTATUS;
}

// === MmFixupImageSectionSegments and helpers — Phase 4 mm batch (translated 2026-05-06) ===
//
// Pure-logic translation of the four `_Shim` helpers + MmFixupImageSectionSegments
// that previously lived in `ntoskrnl/mm/mm_rust_shims.c`. Sorts segments by VA,
// validates bounds, page-aligns them (merging when adjacent segments share a
// page), trims the segment array if the merge phase reduced count, then
// initializes the per-segment fast mutex, refcount, flags, page-table, and
// FileObject pointers. Called from MmCreateImageFileMap (line 2725).

const EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED: ULONG = 1 << 0;
const EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP: ULONG = 1 << 1;
const EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED: ULONG = 1 << 2;

const PAGE_IS_READABLE_FIXUP: ULONG = PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY
    | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;

const STATUS_INVALID_IMAGE_FORMAT_FIXUP: NTSTATUS = 0xC000007Bu32 as NTSTATUS;

unsafe fn MmspSortSegments(ImageSectionObject: PMM_IMAGE_SECTION_OBJECT, Flags: ULONG) {
    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED) != 0 { return; }
    let count = (*ImageSectionObject).NrSegments as usize;
    if count < 2 { return; }
    let base = (*ImageSectionObject).Segments;
    // Insertion sort by VirtualAddress. Avoids pulling in core's pdqsort
    // (which contains unreachable_unchecked → ud2 that scan_ud2 flags).
    // PE images have ≤16 sections in practice, so O(n²) is fine.
    for i in 1..count {
        let mut j = i;
        while j > 0 {
            let prev = base.add(j - 1);
            let cur = base.add(j);
            if (*prev).Image.VirtualAddress <= (*cur).Image.VirtualAddress {
                break;
            }
            // Swap: use a stack temporary; struct is bindgen Copy so this is fine.
            let tmp = *prev;
            *prev = *cur;
            *cur = tmp;
            j -= 1;
        }
    }
}

unsafe fn MmspCheckSegmentBounds(
    ImageSectionObject: PMM_IMAGE_SECTION_OBJECT,
    Flags: ULONG,
) -> BOOLEAN {
    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP) != 0 { return 1; }
    let segs = (*ImageSectionObject).Segments;
    let nr = (*ImageSectionObject).NrSegments as usize;
    ntkrnl_sys::ASSERT!(nr >= 1);

    for i in 0..nr {
        let s = segs.add(i);
        if (*s).Length.QuadPart == 0 { return 0; }
        if i > 0 {
            let prev = segs.add(i - 1);
            let prev_end = ((*prev).Image.VirtualAddress as u64)
                .wrapping_add((*prev).Length.QuadPart as u64);
            if prev_end != (*s).Image.VirtualAddress as u64 {
                return 0;
            }
        }
    }
    1
}

unsafe fn MmspPageAlignSegments(
    ImageSectionObject: PMM_IMAGE_SECTION_OBJECT,
    Flags: ULONG,
) -> BOOLEAN {
    static FLAGS_TO_PROTECTION: [ULONG; 16] = [
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
        PAGE_EXECUTE_WRITECOPY,
    ];

    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED) != 0 { return 1; }

    let segs = (*ImageSectionObject).Segments;
    let nr = (*ImageSectionObject).NrSegments as usize;
    let mut last_segment: usize = 0;
    let page_mask: u64 = PAGE_SIZE as u64 - 1;

    for i in 0..nr {
        if i == 0 {
            let eff = segs;
            let virtual_address = (*eff).Image.VirtualAddress as u64;
            let new_va = virtual_address & !page_mask;
            // `as _` lets the compiler pick u32 (i386) or u64 (amd64/arm64) — bindgen
            // names the field's type ULONG_PTR but its concrete arch-side alias differs
            // from the hand-written ntkrnl_sys::base::ULONG_PTR (always usize).
            (*eff).Image.VirtualAddress = new_va as _;
            let end = virtual_address.wrapping_add((*eff).Length.QuadPart as u64);
            let end_rounded = (end + page_mask) & !page_mask;
            (*eff).Length.QuadPart = (end_rounded - new_va) as i64;

            let virtual_offset = virtual_address - new_va;
            if (*eff).Image.FileOffset < virtual_offset {
                return 0;
            }
            (*eff).Image.FileOffset -= virtual_offset;
            (*eff).RawLength.QuadPart += virtual_offset as i64;
        } else {
            let cur = segs.add(i);
            let eff = segs.add(last_segment);

            let end_of_eff = ((*eff).Image.VirtualAddress as u64)
                + ((*eff).Length.QuadPart as u64);
            ntkrnl_sys::ASSERT!((end_of_eff & page_mask) == 0);

            if end_of_eff == (*cur).Image.VirtualAddress as u64 {
                last_segment += 1;
                ntkrnl_sys::ASSERT!(last_segment <= i);
                ntkrnl_sys::ASSERT!(last_segment < nr);

                let next_eff = segs.add(last_segment);
                if last_segment != i {
                    *next_eff = *cur;
                }
                ntkrnl_sys::ASSERT!(((*next_eff).Image.VirtualAddress as u64 & page_mask) == 0);
                let len = (*next_eff).Length.QuadPart as u64;
                let len_rounded = (len + page_mask) & !page_mask;
                (*next_eff).Length.QuadPart = len_rounded as i64;
            } else if end_of_eff > (*cur).Image.VirtualAddress as u64 {
                if (*cur).Image.FileOffset
                    != (*eff).Image.FileOffset + (*eff).RawLength.QuadPart as u64 {
                    return 0;
                }
                (*eff).RawLength.QuadPart += (*cur).RawLength.QuadPart;

                let cur_end_rounded = ((*cur).Image.VirtualAddress as u64
                    + (*cur).Length.QuadPart as u64
                    + page_mask) & !page_mask;
                ntkrnl_sys::ASSERT!(cur_end_rounded >= end_of_eff);
                (*eff).Length.QuadPart =
                    (cur_end_rounded - (*eff).Image.VirtualAddress as u64) as i64;

                (*eff).Protection |= (*cur).Protection;

                let mut protection_flags: u32 = 0;
                if ((*eff).Protection & PAGE_IS_READABLE_FIXUP) != 0 { protection_flags |= 1 << 0; }
                if ((*eff).Protection & PAGE_IS_WRITABLE)       != 0 { protection_flags |= 1 << 1; }
                if ((*eff).Protection & PAGE_IS_EXECUTABLE)     != 0 { protection_flags |= 1 << 2; }
                if ((*eff).Protection & PAGE_IS_WRITECOPY)      != 0 { protection_flags |= 1 << 3; }
                ntkrnl_sys::ASSERT!(protection_flags < 16);
                (*eff).Protection = FLAGS_TO_PROTECTION[protection_flags as usize];

                if ((*cur).Protection & PAGE_IS_WRITECOPY) == 0
                    && ((*eff).Protection & PAGE_IS_WRITECOPY) != 0 {
                    return 0;
                }
            } else {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }
    }
    (*ImageSectionObject).NrSegments = (last_segment + 1) as ULONG;
    1
}

#[no_mangle]
pub unsafe extern "system" fn MmFixupImageSectionSegments(
    ImageSectionObject: PMM_IMAGE_SECTION_OBJECT,
    Flags: ULONG,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    MmspSortSegments(ImageSectionObject, Flags);

    if MmspCheckSegmentBounds(ImageSectionObject, Flags) == 0 {
        return STATUS_INVALID_IMAGE_FORMAT_FIXUP;
    }

    let old_nr = (*ImageSectionObject).NrSegments;

    if MmspPageAlignSegments(ImageSectionObject, Flags) == 0 {
        return STATUS_INVALID_IMAGE_FORMAT_FIXUP;
    }

    if (*ImageSectionObject).NrSegments < old_nr {
        let count = (*ImageSectionObject).NrSegments as usize;
        let size = core::mem::size_of::<MM_SECTION_SEGMENT>() * count;
        let new_segs = ExAllocatePoolWithTag(
            PagedPool,
            size as SIZE_T,
            TAG_MM_SECTION_SEGMENT,
        ) as PMM_SECTION_SEGMENT;
        if new_segs.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping((*ImageSectionObject).Segments, new_segs, count);
        ExFreePool((*ImageSectionObject).Segments as PVOID);
        (*ImageSectionObject).Segments = new_segs;
    }

    let nr = (*ImageSectionObject).NrSegments as usize;
    for i in 0..nr {
        let seg = (*ImageSectionObject).Segments.add(i);
        ExInitializeFastMutex(core::ptr::addr_of_mut!((*seg).Lock));
        (*seg).ReferenceCount = core::ptr::addr_of_mut!((*ImageSectionObject).RefCount);
        (*seg).Flags = core::ptr::addr_of_mut!((*ImageSectionObject).SegFlags);
        MiInitializeSectionPageTable(seg);
        (*seg).FileObject = FileObject;
    }

    ntkrnl_sys::ASSERT!((*ImageSectionObject).RefCount > 0);
    (*ImageSectionObject).FileObject = FileObject;

    STATUS_SUCCESS
}

const _64K: usize = 64 * 1024;
const MM_SEGMENT_INCREATE: ULONG = 0x8;
const MM_SEGMENT_INDELETE: ULONG = 0x4;
const MM_PHYSICALMEMORY_SEGMENT: ULONG = 0x1;
const MM_DATAFILE_SEGMENT: ULONG = 0x2;
const TAG_MM_SECTION_SEGMENT: ULONG = 0x53534D4D;
const TAG_MM_REGION: ULONG = 0x4E47524D;
const MM_ALLOCATION_GRANULARITY: usize = 0x10000;
const MM_VIRTMEM_GRANULARITY: usize = 64 * 1024;
const MAX_SHARE_COUNT: ULONG_PTR = 0x1FF;
const VadImageMap: u32 = 2;
const MEM_IMAGE: ULONG = 0x1000000;
const MEM_MAPPED: ULONG = 0x40000;
const MEM_COMMIT_DUP: ULONG = 0x1000;
const ViewShare: SECTION_INHERIT = 1;
const STANDARD_RIGHTS_READ: ACCESS_MASK = 0x00020000;
const STANDARD_RIGHTS_WRITE: ACCESS_MASK = 0x00020000;
const STANDARD_RIGHTS_EXECUTE: ACCESS_MASK = 0x00020000;
const SECTION_MAP_READ: ACCESS_MASK = 0x0004;
const SECTION_MAP_WRITE: ACCESS_MASK = 0x0002;
const SECTION_MAP_EXECUTE: ACCESS_MASK = 0x0008;
const SECTION_QUERY: ACCESS_MASK = 0x0001;
const SECTION_ALL_ACCESS: ACCESS_MASK = 0x000F001F;
const OBJ_PERMANENT: ULONG = 0x00000010;
const OBJ_KERNEL_EXCLUSIVE: ULONG = 0x10000;
const SEC_FILE: ULONG = 0x800000;
const SEC_IMAGE: ULONG = 0x1000000;
const SEC_COMMIT: ULONG = 0x8000000;
const SEC_RESERVE: ULONG = 0x4000000;
const SEC_NO_CHANGE: ULONG = 0x400000;
const SEC_LARGE_PAGES: ULONG = 0x80000000;
const FO_DELETE_ON_CLOSE: ULONG = 0x00010000;
const FO_CLEANUP_COMPLETE: ULONG = 0x4000;
const FO_RANDOM_ACCESS: ULONG = 0x00100000;
const FSRTL_MOD_WRITE_TOP_LEVEL_IRP: ULONG_PTR = 0x03;
const IMAGE_FILE_DLL: u16 = 0x2000;
const IMAGE_DLLCHARACTERISTICS_NO_ISOLATION: u16 = 0x0200;
const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
const IMAGE_SCN_CNT_CODE: ULONG = 0x00000020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: ULONG = 0x00000040;
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: ULONG = 0x00000080;
const IMAGE_SCN_MEM_EXECUTE: ULONG = 0x20000000;
const IMAGE_SCN_MEM_READ: ULONG = 0x40000000;
const IMAGE_SCN_MEM_WRITE: ULONG = 0x80000000;
const IMAGE_SCN_MEM_SHARED: ULONG = 0x10000000;
const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x00004550;
const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x10b;
const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x20b;
const PAGE_FLAGS_VALID_FOR_SECTION: ULONG =
    PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY |
    PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY |
    PAGE_NOACCESS;
const PAGE_IS_WRITABLE: ULONG = PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
const PAGE_IS_EXECUTABLE: ULONG = PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
const PAGE_IS_WRITECOPY: ULONG = PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY;
const STATUS_MAPPED_FILE_SIZE_ZERO: NTSTATUS = 0xC000011E_u32 as NTSTATUS;
const STATUS_SECTION_NOT_EXTENDED: NTSTATUS = 0xC0000087_u32 as NTSTATUS;
const STATUS_NOT_MAPPED_VIEW: NTSTATUS = 0xC0000019_u32 as NTSTATUS;
const STATUS_GUARD_PAGE_VIOLATION: NTSTATUS = 0x80000001_u32 as NTSTATUS;
const STATUS_IMAGE_NOT_AT_BASE: NTSTATUS = 0x40000003_u32 as NTSTATUS;
const STATUS_SECTION_PROTECTION: NTSTATUS = 0xC000004E_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_NOT_MZ: NTSTATUS = 0xC000012F_u32 as NTSTATUS;
// sdk/include/reactos/exeformat.h:72. This was 0xC000FFFE — an invented value
// (§9.113). section_pe.rs PRODUCES the real 0xA0100001, so both comparisons
// below silently failed: the loader-retry loop always broke on the first pass,
// and the `-> STATUS_INVALID_IMAGE_NOT_MZ` conversion never fired, leaking the
// raw internal status to callers who open a non-PE file as an image section.
const STATUS_ROS_EXEFMT_UNKNOWN_FORMAT: NTSTATUS = 0xA0100001_u32 as NTSTATUS;
const STATUS_IN_PAGE_ERROR: NTSTATUS = 0xC0000006_u32 as NTSTATUS;
const STATUS_INTEGER_OVERFLOW: NTSTATUS = 0xC0000095_u32 as NTSTATUS;
const STATUS_INVALID_FILE_FOR_SECTION: NTSTATUS = 0xC0000020_u32 as NTSTATUS;
const STATUS_INVALID_VIEW_SIZE: NTSTATUS = 0xC000001F_u32 as NTSTATUS;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018_u32 as NTSTATUS;

static mut TinyTime: LARGE_INTEGER = unsafe { core::mem::transmute(-1i64) };

#[no_mangle]
pub static mut MmSectionObjectType: POBJECT_TYPE = core::ptr::null_mut();

static mut SectionCharacteristicsToProtect: [ULONG; 16] = [
    PAGE_NOACCESS, PAGE_NOACCESS, PAGE_EXECUTE, PAGE_EXECUTE,
    PAGE_READONLY, PAGE_READONLY, PAGE_EXECUTE_READ, PAGE_EXECUTE_READ,
    PAGE_READWRITE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_READWRITE,
    PAGE_READWRITE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_READWRITE,
];

static mut MmpSectionMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | SECTION_MAP_READ | SECTION_QUERY,
    GenericWrite: STANDARD_RIGHTS_WRITE | SECTION_MAP_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SECTION_MAP_EXECUTE,
    GenericAll: SECTION_ALL_ACCESS,
};

#[inline]
unsafe fn MmSetPageEntrySectionSegment(
    Segment: PMM_SECTION_SEGMENT, Offset: *mut LARGE_INTEGER, Entry: ULONG_PTR
) -> NTSTATUS {
    _MmSetPageEntrySectionSegment(Segment, Offset, Entry, b"section.rs\0".as_ptr() as *const i8, 0)
}

#[inline]
unsafe fn MmGetPageEntrySectionSegment(Segment: *mut core::ffi::c_void, Offset: *const i64) -> ULONG_PTR {
    MmGetPageEntrySectionSegmentDdk(Segment, Offset as *const u64)
}

#[inline]
fn MAKE_SSE(page_addr: ULONG_PTR, share_count: ULONG_PTR) -> ULONG_PTR {
    page_addr | (share_count << 3)
}

#[inline]
fn DIRTY_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    sse | 2
}

#[inline]
fn MAKE_SWAP_SSE(swap_entry: ULONG_PTR) -> ULONG_PTR {
    (swap_entry << 1) | 0x1
}

#[inline]
fn SHARE_COUNT_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    (sse & 0x00000FFC) >> 3
}

#[inline]
fn PAGE_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    #[cfg(target_pointer_width = "64")]
    { sse & 0xFFFFFFFFF000 }
    #[cfg(target_pointer_width = "32")]
    { sse & 0xFFFFF000 }
}

#[inline]
fn PFN_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    PAGE_FROM_SSE(sse) >> PAGE_SHIFT
}

#[inline]
fn BUMPREF_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    PAGE_FROM_SSE(sse) | ((SHARE_COUNT_FROM_SSE(sse) + 1) << 3) | (sse & 0x7)
}

#[inline]
fn DECREF_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    PAGE_FROM_SSE(sse) | ((SHARE_COUNT_FROM_SSE(sse) - 1) << 3) | (sse & 0x7)
}

#[inline]
fn SWAPENTRY_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    sse >> 1
}

#[inline]
fn IS_SWAP_FROM_SSE(sse: ULONG_PTR) -> bool {
    (sse & 0x1) != 0
}

#[inline]
fn IS_DIRTY_SSE(sse: ULONG_PTR) -> bool {
    (sse & 0x2) != 0
}

#[inline]
fn IS_WRITE_SSE(sse: ULONG_PTR) -> bool {
    (sse & 0x4) != 0
}

#[inline]
fn WRITE_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    sse | 0x4
}

#[no_mangle]
pub unsafe extern "C" fn MA_GetStartingAddress(ma: PMEMORY_AREA) -> ULONG_PTR {
    ((*ma).VadNode.StartingVpn as ULONG_PTR) << PAGE_SHIFT
}

#[no_mangle]
pub unsafe extern "C" fn MA_GetEndingAddress(ma: PMEMORY_AREA) -> ULONG_PTR {
    (((*ma).VadNode.EndingVpn as ULONG_PTR) + 1) << PAGE_SHIFT
}

#[inline]
unsafe fn MmInitializeMdl(mdl: *mut MDL, base_va: PVOID, length: usize) {
    (*mdl).Next = core::ptr::null_mut();
    (*mdl).Size = (core::mem::size_of::<MDL>() + core::mem::size_of::<PFN_NUMBER>() * ((length + PAGE_SIZE - 1) >> PAGE_SHIFT)) as i16;
    (*mdl).MdlFlags = 0;
    (*mdl).StartVa = (base_va as ULONG_PTR & !(PAGE_SIZE - 1)) as PVOID;
    (*mdl).ByteOffset = (base_va as ULONG_PTR & (PAGE_SIZE - 1)) as ULONG;
    (*mdl).ByteCount = length as ULONG;
}

#[inline]
unsafe fn MmGetMdlPfnArray(mdl: *mut MDL) -> *mut PFN_NUMBER {
    (mdl as *mut u8).add(core::mem::size_of::<MDL>()) as *mut PFN_NUMBER
}

unsafe fn MiGrabDataSection(SectionObjectPointer: PSECTION_OBJECT_POINTERS) -> PMM_SECTION_SEGMENT {
    let mut OldIrql = MiAcquirePfnLock();
    let mut Segment: PMM_SECTION_SEGMENT;
    // Per-CALL, matching the C's local GrabSpinCount: a boot that spins here
    // briefly a few times is normal, a boot WEDGED here is the thing we cannot
    // currently see. Rate-limited 1:1000 because each turn of the loop already
    // costs a KeDelayExecutionThread(TinyTime).
    #[cfg(target_arch = "aarch64")]
    let mut GrabSpinCount: u32 = 0;

    loop {
        Segment = (*SectionObjectPointer).DataSectionObject as PMM_SECTION_SEGMENT;
        if Segment.is_null() {
            break;
        }

        if (*Segment).SegFlags & (MM_SEGMENT_INCREATE | MM_SEGMENT_INDELETE) != 0 {
            #[cfg(target_arch = "aarch64")]
            {
                if GrabSpinCount % 1000 == 0 {
                    mm_guard_trip_atomic!(
                        "section.MiGrabDataSection INCREATE/INDELETE spin", u32::MAX,
                        "SegFlags" => (*Segment).SegFlags,
                        "SpinCount" => GrabSpinCount,
                    );
                }
                GrabSpinCount = GrabSpinCount.wrapping_add(1);
            }
            MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            OldIrql = MiAcquirePfnLock();
            continue;
        }

        InterlockedIncrement64((*Segment).ReferenceCount as *mut i64);
        break;
    }

    MiReleasePfnLock(OldIrql);
    Segment
}

unsafe fn ImageSectionObjectFromSegment(Segment: PMM_SECTION_SEGMENT) -> PMM_IMAGE_SECTION_OBJECT {
    // C: CONTAINING_RECORD(Segment->ReferenceCount, MM_IMAGE_SECTION_OBJECT, RefCount)
    // Segment->ReferenceCount is a PLONG64 pointing INTO the parent ImageSectionObject's
    // RefCount field. We offset backwards from that pointer to get the parent struct.
    ((*Segment).ReferenceCount as *const u8).offset(
        -(core::mem::offset_of!(MM_IMAGE_SECTION_OBJECT, RefCount) as isize)
    ) as PMM_IMAGE_SECTION_OBJECT
}

unsafe fn MiWritePage(Segment: PMM_SECTION_SEGMENT, SegOffset: i64, Page: PFN_NUMBER) -> NTSTATUS {
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut MdlBase: [u8; core::mem::size_of::<MDL>() + core::mem::size_of::<PFN_NUMBER>()] = core::mem::zeroed();
    let Mdl = MdlBase.as_mut_ptr() as *mut MDL;
    let FileObject = (*Segment).FileObject;
    let mut FileOffset = LARGE_INTEGER { QuadPart: (*Segment).Image.FileOffset as i64 + SegOffset };

    MmInitializeMdl(Mdl, core::ptr::null_mut(), PAGE_SIZE);
    MmBuildMdlFromPages(Mdl, core::ptr::addr_of!(Page));
    (*Mdl).MdlFlags |= 0x02; // MDL_PAGES_LOCKED — caller pre-locked the page

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);
    let Status = IoSynchronousPageWrite(FileObject, Mdl, core::ptr::addr_of_mut!(FileOffset), core::ptr::addr_of_mut!(Event), core::ptr::addr_of_mut!(IoStatus));
    let Status = if Status == STATUS_PENDING {
        KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0i8, 0u8, core::ptr::null_mut());
        IoStatus.__bindgen_anon_1.Status
    } else { Status };

    if (*Mdl).MdlFlags & 0x01 != 0 {
        MmUnmapLockedPages((*Mdl).MappedSystemVa, Mdl);
    }

    if !NT_SUCCESS(Status) {
        ntkrnl_sys::driver_ddk::DbgPrint(c"[MM] MiWritePage FAILED st=%08lx off=%I64x fo=%p\n".as_ptr(),
            Status, FileOffset.QuadPart, FileObject);
    }

    Status
}

unsafe fn MmspWaitForFileLock(_File: PFILE_OBJECT) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe fn MmpFreePageFileSegment(Segment: PMM_SECTION_SEGMENT) {
    let Length = PAGE_ROUND_UP((*Segment).Length.QuadPart as usize);
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };

    extern "system" {
    fn MmLockSectionSegment(Segment: PVOID);
    fn MmUnlockSectionSegment(Segment: PVOID);
    fn MmFreeSwapPage(SwapEntry: ULONG_PTR);
    fn MmGetSavedSwapEntryPage(Page: ULONG_PTR) -> ULONG_PTR;
    fn MmSetSavedSwapEntryPage(Page: ULONG_PTR, SwapEntry: ULONG_PTR);
    fn MmReleasePageMemoryConsumer(Type: ULONG, Page: ULONG_PTR) -> NTSTATUS;
}

    MmLockSectionSegment(Segment as PVOID);
    while Offset.QuadPart < Length as i64 {
        let Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
        if Entry != 0 {
            MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset), 0);
            if IS_SWAP_FROM_SSE(Entry) {
                MmFreeSwapPage(SWAPENTRY_FROM_SSE(Entry));
            } else {
                let Page = PFN_FROM_SSE(Entry);
                let SavedSwapEntry = MmGetSavedSwapEntryPage(Page);
                if SavedSwapEntry != 0 {
                    MmSetSavedSwapEntryPage(Page, 0);
                    MmFreeSwapPage(SavedSwapEntry);
                }
                MmReleasePageMemoryConsumer(MC_USER, Page);
            }
        }
        Offset.QuadPart += PAGE_SIZE as i64;
    }
    MmUnlockSectionSegment(Segment as PVOID);
}

unsafe extern "C" fn FreeSegmentPage(Segment: PMM_SECTION_SEGMENT, Offset: PLARGE_INTEGER) {
    MmLockSectionSegment(Segment as PVOID);
    let Entry = MmGetPageEntrySectionSegment(Segment as PVOID, Offset as *const i64);
    MmUnlockSectionSegment(Segment as PVOID);
    let Page = PFN_FROM_SSE(Entry);
    if IS_DIRTY_SSE(Entry) {
        MiWritePage(Segment, (*Offset).QuadPart, Page as PFN_NUMBER);
    }
    MmReleasePageMemoryConsumer(MC_USER, Page);
}

#[no_mangle]
pub unsafe extern "system" fn MmDereferenceSegmentWithLock(
    Segment: PMM_SECTION_SEGMENT, mut OldIrql: KIRQL
) {
    if OldIrql == MM_NOIRQL as KIRQL {
        OldIrql = MiAcquirePfnLock();
    }
    if InterlockedDecrement64((*Segment).ReferenceCount as *mut i64) > 0 {
        MiReleasePfnLock(OldIrql);
        return;
    }
    *(*Segment).Flags |= MM_SEGMENT_INDELETE;
    if *(*Segment).Flags & MM_DATAFILE_SEGMENT != 0 {
        MiReleasePfnLock(OldIrql);
        MmFreePageTablesSectionSegment(Segment, Some(FreeSegmentPage));
        OldIrql = MiAcquirePfnLock();
        (*(*(*Segment).FileObject).SectionObjectPointer).DataSectionObject = core::ptr::null_mut();
        MiReleasePfnLock(OldIrql);
        ObDereferenceObject((*Segment).FileObject as PVOID);
        ExFreePoolWithTag(Segment as PVOID, TAG_MM_SECTION_SEGMENT);
    } else {
        let ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        (*(*(*ImageSectionObject).FileObject).SectionObjectPointer).ImageSectionObject = core::ptr::null_mut();
        MiReleasePfnLock(OldIrql);
        ObDereferenceObject((*ImageSectionObject).FileObject as PVOID);
        let NrSegments = (*ImageSectionObject).NrSegments;
        let SectionSegments = (*ImageSectionObject).Segments;
        for i in 0..NrSegments {
            if (*SectionSegments.add(i as usize)).Image.Characteristics & IMAGE_SCN_MEM_SHARED != 0 {
                MmpFreePageFileSegment(&mut *SectionSegments.add(i as usize));
            }
            MmFreePageTablesSectionSegment(&mut *SectionSegments.add(i as usize), None);
        }
        ExFreePoolWithTag(SectionSegments as PVOID, TAG_MM_SECTION_SEGMENT);
        ExFreePoolWithTag(ImageSectionObject as PVOID, TAG_MM_SECTION_SEGMENT);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmSharePageEntrySectionSegment(
    Segment: PMM_SECTION_SEGMENT, Offset: PLARGE_INTEGER
) {
    let Offset = opaque_ptr(Offset);
    let Entry = MmGetPageEntrySectionSegment(Segment as PVOID, Offset as *const i64);
    if Entry == 0 || SHARE_COUNT_FROM_SSE(Entry) == MAX_SHARE_COUNT || IS_SWAP_FROM_SSE(Entry) {
        KeBugCheck(MEMORY_MANAGEMENT);
    }
    MmSetPageEntrySectionSegment(Segment, Offset, BUMPREF_SSE(Entry));
}

#[no_mangle]
pub unsafe extern "system" fn MmUnsharePageEntrySectionSegment(
    _MemoryArea: PMEMORY_AREA, Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER, Dirty: BOOLEAN, PageOut: BOOLEAN, InEntry: *mut ULONG_PTR
) -> BOOLEAN {
    let Offset = opaque_ptr(Offset);
    let InEntry = opaque_ptr(InEntry);
    let mut Entry = if !InEntry.is_null() { *InEntry } else { MmGetPageEntrySectionSegment(Segment as PVOID, Offset as *const i64) };
    let Page = PFN_FROM_SSE(Entry);
    let IsDataMap = (*Segment).SegFlags & MM_DATAFILE_SEGMENT != 0;

    if Entry == 0 || SHARE_COUNT_FROM_SSE(Entry) == 0 || IS_SWAP_FROM_SSE(Entry) {
        KeBugCheck(MEMORY_MANAGEMENT);
    }
    Entry = DECREF_SSE(Entry);
    if Dirty != 0 { Entry = DIRTY_SSE(Entry); }

    if SHARE_COUNT_FROM_SSE(Entry) > 0 || PageOut != 0 {
        MmSetPageEntrySectionSegment(Segment, Offset, Entry);
        return 0u8;
    }
    if IsDataMap {
        MmSetPageEntrySectionSegment(Segment, Offset, Entry);
        return 0u8;
    }
    if (*Segment).Image.Characteristics & IMAGE_SCN_MEM_SHARED == 0 {
        MmSetPageEntrySectionSegment(Segment, Offset, Entry);
        return 0u8;
    }
    let SwapEntry = MmGetSavedSwapEntryPage(Page);
    if SwapEntry == 0 && !IS_DIRTY_SSE(Entry) {
        MmSetPageEntrySectionSegment(Segment, Offset, Entry);
        return 0u8;
    }
    if SwapEntry != 0 {
        MmSetSavedSwapEntryPage(Page, 0);
        MmFreeSwapPage(SwapEntry);
    }
    MmSetPageEntrySectionSegment(Segment, Offset, 0);
    MmReleasePageMemoryConsumer(MC_USER, Page);
    1u8
}

unsafe fn MiCopyFromUserPage(DestPage: PFN_NUMBER, SrcAddress: *const core::ffi::c_void) -> NTSTATUS {
    let Process = PsGetCurrentProcess();
    let mut Irql: KIRQL = 0;
    let DestAddress = MiMapPageInHyperSpace(Process as PEPROCESS, DestPage as ntkrnl_sys::ddk::PFN_NUMBER, core::ptr::addr_of_mut!(Irql));
    if DestAddress.is_null() {
        return STATUS_NO_MEMORY;
    }
    core::ptr::copy_nonoverlapping(SrcAddress as *const u8, DestAddress as *mut u8, PAGE_SIZE);
    MiUnmapPageInHyperSpace(Process as PEPROCESS, DestAddress, Irql);
    STATUS_SUCCESS
}

unsafe fn MmMakeSegmentResident(
    Segment: PMM_SECTION_SEGMENT, Offset: i64, Length: ULONG,
    _ValidDataLength: PLARGE_INTEGER, SetDirty: BOOLEAN
) -> NTSTATUS {
    extern "system" {
    fn MmRequestPageMemoryConsumer(Type: ULONG, CanWait: BOOLEAN, Page: *mut ULONG_PTR) -> NTSTATUS;
    fn MmBuildMdlFromPages(Mdl: *mut MDL, Pages: *const PFN_NUMBER);
    fn MmUnmapLockedPages(MappedSystemVa: PVOID, Mdl: *mut MDL);
}

    // Use 16K readahead chunks instead of 64K (workaround for IoPageRead hang with >=32K reads on i386 CDFS)
    const READAHEAD_CHUNK: usize = 16 * 1024;
    let mut RangeStart = Offset - (Offset % READAHEAD_CHUNK as i64);
    let mut RangeEnd = Offset + Length as i64;
    if RangeEnd % READAHEAD_CHUNK as i64 != 0 {
        RangeEnd += READAHEAD_CHUNK as i64 - (RangeEnd % READAHEAD_CHUNK as i64);
    }
    if (*Segment).SegFlags & MM_DATAFILE_SEGMENT == 0 {
        if RangeEnd > (*Segment).RawLength.QuadPart {
            RangeEnd = (*Segment).RawLength.QuadPart;
        }
    }

    while RangeStart < RangeEnd {
        let mut ToReadPageBits: ULONG = 0;
        let ChunkEnd = core::cmp::min(RangeStart + READAHEAD_CHUNK as i64, RangeEnd);
        MmLockSectionSegment(Segment as PVOID);

        let mut ChunkOffset = RangeStart;
        while ChunkOffset < ChunkEnd {
            let mut CurrentOffset = LARGE_INTEGER { QuadPart: ChunkOffset };
            let mut Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(CurrentOffset.QuadPart));
            while MM_IS_WAIT_PTE(Entry) {
                MmUnlockSectionSegment(Segment as PVOID);
                KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
                MmLockSectionSegment(Segment as PVOID);
                Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(CurrentOffset.QuadPart));
            }
            if Entry != 0 {
                if SetDirty != 0 && !IS_SWAP_FROM_SSE(Entry) {
                    MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), DIRTY_SSE(Entry));
                }
                ChunkOffset += PAGE_SIZE as i64;
                continue;
            }
            ToReadPageBits |= 1u32 << ((ChunkOffset - RangeStart) >> PAGE_SHIFT);
            let Status = MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), MAKE_SWAP_SSE(MM_WAIT_ENTRY));
            if !NT_SUCCESS(Status) {
                let mut RollbackOffset = LARGE_INTEGER { QuadPart: ChunkOffset };
                while RollbackOffset.QuadPart > RangeStart {
                    RollbackOffset.QuadPart -= PAGE_SIZE as i64;
                    if MM_IS_WAIT_PTE(MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(RollbackOffset.QuadPart))) {
                        MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(RollbackOffset), 0);
                    }
                }
                MmUnlockSectionSegment(Segment as PVOID);
                return Status;
            }
            ChunkOffset += PAGE_SIZE as i64;
        }
        MmUnlockSectionSegment(Segment as PVOID);

        if ToReadPageBits == 0 {
            RangeStart += READAHEAD_CHUNK as i64;
            continue;
        }

        ChunkOffset = RangeStart;
        while ChunkOffset < ChunkEnd {
            let mut BitSet: ULONG = 0;
            if _BitScanForward(core::ptr::addr_of_mut!(BitSet), ToReadPageBits) == 0 {
                break;
            }
            ToReadPageBits >>= BitSet;
            ChunkOffset += (BitSet as usize * PAGE_SIZE) as i64;
            _BitScanForward(core::ptr::addr_of_mut!(BitSet), !ToReadPageBits);
            let ReadLength: ULONG = core::cmp::min(BitSet * PAGE_SIZE as ULONG, (ChunkEnd - ChunkOffset) as ULONG);

            let Mdl = IoAllocateMdl(core::ptr::null_mut(), ReadLength, 0u8, 0u8, core::ptr::null_mut());
            if Mdl.is_null() {
                MmLockSectionSegment(Segment as PVOID);
                while ChunkOffset < ChunkEnd {
                    if ToReadPageBits & 1 != 0 {
                        let mut CurrentOffset = LARGE_INTEGER { QuadPart: ChunkOffset };
                        MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), 0);
                    }
                    ToReadPageBits >>= 1;
                    ChunkOffset += PAGE_SIZE as i64;
                }
                MmUnlockSectionSegment(Segment as PVOID);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            let Pages = MmGetMdlPfnArray(Mdl);
            // FIX: write_bytes<T> writes count*sizeof(T) bytes, so pass element count only.
            // Was: write_bytes(Pages, 0, N * sizeof(PFN_NUMBER)) — double-counted sizeof(T)!
            core::ptr::write_bytes(Pages, 0, BYTES_TO_PAGES(ReadLength as usize));
            for i in 0..BYTES_TO_PAGES(ReadLength as usize) {
                let Status = MmRequestPageMemoryConsumer(MC_USER, 0u8, Pages.add(i) as *mut ULONG_PTR);
                if !NT_SUCCESS(Status) {
                    for j in 0..i {
                        MmReleasePageMemoryConsumer(MC_USER, *Pages.add(j) as ULONG_PTR);
                    }
                    MmLockSectionSegment(Segment as PVOID);
                    while ChunkOffset < ChunkEnd {
                        if ToReadPageBits & 1 != 0 {
                            let mut CurrentOffset = LARGE_INTEGER { QuadPart: ChunkOffset };
                            MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), 0);
                        }
                        ToReadPageBits >>= 1;
                        ChunkOffset += PAGE_SIZE as i64;
                    }
                    MmUnlockSectionSegment(Segment as PVOID);
                    IoFreeMdl(Mdl);
                    return Status;
                }
            }
            (*Mdl).MdlFlags |= 0x0002 | 0x0040; // MDL_PAGES_LOCKED | MDL_IO_PAGE_READ
            let mut FileOffset = LARGE_INTEGER { QuadPart: (*Segment).Image.FileOffset as i64 + ChunkOffset };
            let mut Event: KEVENT = core::mem::zeroed();
            KeInitializeEvent(core::ptr::addr_of_mut!(Event), 0, 0u8);
            let mut OldIrql: KIRQL = 0;
            KeRaiseIrql(APC_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
            let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut Status = IoPageRead((*Segment).FileObject, Mdl, core::ptr::addr_of_mut!(FileOffset), core::ptr::addr_of_mut!(Event), core::ptr::addr_of_mut!(Iosb));
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID, 0, 0i8, 0u8, core::ptr::null_mut());
                Status = Iosb.__bindgen_anon_1.Status;
            }
            // ARM's instruction fetch is not coherent with its data cache. The
            // read above filled these pages through the D-cache, so until the
            // lines are cleaned to the point of unification an instruction fetch
            // from them reads whatever the RAM held before - on a Raspberry Pi 4,
            // smss died with STATUS_ILLEGAL_INSTRUCTION on the first `stp` of
            // RtlSetUnhandledExceptionFilter, a page of zeros decoding as UDF.
            // QEMU models no caches, so only hardware shows it. Clean through a
            // system mapping (A72 caches are physically indexed, so any alias
            // reaches the lines) and invalidate the I-cache, which may hold stale
            // instructions for a recycled physical page even when the driver
            // wrote by DMA. Image segments only: data-file pages are never
            // executed from this path. The existing cleanup below unmaps
            // whatever mapping is present, including one made here.
            #[cfg(target_arch = "aarch64")]
            if NT_SUCCESS(Status) && (*Segment).SegFlags & MM_DATAFILE_SEGMENT == 0 {
                if (*Mdl).MdlFlags & 0x01 == 0 {
                    use ntkrnl_sys::ddk::{
                        MmMapLockedPagesSpecifyCache, _MEMORY_CACHING_TYPE_MmCached,
                        _MM_PAGE_PRIORITY_HighPagePriority, _MODE_KernelMode,
                    };
                    // A CACHED alias, so the clean below reaches the same lines the
                    // read dirtied. Bugcheck on failure (the 1) rather than skip it.
                    MmMapLockedPagesSpecifyCache(
                        Mdl as _, _MODE_KernelMode as _, _MEMORY_CACHING_TYPE_MmCached,
                        core::ptr::null_mut(), 1, _MM_PAGE_PRIORITY_HighPagePriority as ULONG,
                    );
                }
                ntkrnl_sys::inline::ke::KeSweepICache((*Mdl).MappedSystemVa, ReadLength as ULONG_PTR);
            }
            if (*Mdl).MdlFlags & 0x01 != 0 {
                MmUnmapLockedPages((*Mdl).MappedSystemVa, Mdl);
            }
            KeLowerIrql(OldIrql);
            if Status == STATUS_END_OF_FILE {
                Status = STATUS_SUCCESS;
            }
            if !NT_SUCCESS(Status) {
                for i in 0..BYTES_TO_PAGES(ReadLength as usize) {
                    MmReleasePageMemoryConsumer(MC_USER, *Pages.add(i) as ULONG_PTR);
                }
                MmLockSectionSegment(Segment as PVOID);
                while ChunkOffset < ChunkEnd {
                    if ToReadPageBits & 1 != 0 {
                        let mut CurrentOffset = LARGE_INTEGER { QuadPart: ChunkOffset };
                        MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), 0);
                    }
                    ToReadPageBits >>= 1;
                    ChunkOffset += PAGE_SIZE as i64;
                }
                MmUnlockSectionSegment(Segment as PVOID);
                IoFreeMdl(Mdl);
                return Status;
            }
            MmLockSectionSegment(Segment as PVOID);
            for i in 0..BYTES_TO_PAGES(ReadLength as usize) {
                let mut Entry = MAKE_SSE((*Pages.add(i) as usize) << PAGE_SHIFT, 0);
                let mut CurrentOffset = LARGE_INTEGER { QuadPart: ChunkOffset + (i * PAGE_SIZE) as i64 };
                if SetDirty != 0 {
                    Entry = DIRTY_SSE(Entry);
                }
                MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(CurrentOffset), Entry);
            }
            MmUnlockSectionSegment(Segment as PVOID);
            IoFreeMdl(Mdl);
            ToReadPageBits >>= BitSet;
            ChunkOffset += (BitSet as usize * PAGE_SIZE) as i64;
        }
        RangeStart += READAHEAD_CHUNK as i64;
    }
    STATUS_SUCCESS
}

unsafe extern "system" fn MmpDeleteSection(ObjectBody: PVOID) {
    MmpDeleteSectionInner(ObjectBody);
}
#[inline(always)]
unsafe fn MmpDeleteSectionInner(ObjectBody: PVOID) {
    let Section = ObjectBody as PSECTION;
    if MiIsRosSectionObject(Section) == 0 {
        MiDeleteARM3Section(ObjectBody);
        return;
    }
    if (*Section).Segment.is_null() {
        return;
    }
    if (*Section).u.Flags.Image() != 0 {
        let ImageSectionObject = (*Section).Segment as PMM_IMAGE_SECTION_OBJECT;
        let mut OldIrql = MiAcquirePfnLock();
        (*ImageSectionObject).SectionCount -= 1;
        MmDereferenceSegmentWithLock((*ImageSectionObject).Segments, OldIrql);
    } else {
        let Segment = (*Section).Segment as PMM_SECTION_SEGMENT;
        let mut OldIrql = MiAcquirePfnLock();
        (*Segment).SectionCount -= 1;
        MmDereferenceSegmentWithLock(Segment, OldIrql);
    }
}

unsafe extern "system" fn MmpCloseSection(
    _Process: PEPROCESS, _Object: PVOID, _GrantedAccess: ACCESS_MASK,
    _ProcessHandleCount: ntkrnl_sys::ddk::ULONG_PTR, _SystemHandleCount: ntkrnl_sys::ddk::ULONG_PTR
) {}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePhysicalMemorySection() -> NTSTATUS {
    let mut SectionSize = LARGE_INTEGER {
        QuadPart: (MmHighestPhysicalPage * PAGE_SIZE) as i64,
    };
    // L"\\Device\\PhysicalMemory" — UTF-16LE
    static PHYS_MEM_NAME: [u16; 23] = [
        0x5C, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5C, // \Device\backslash
        0x50, 0x68, 0x79, 0x73, 0x69, 0x63, 0x61, 0x6C, // Physical
        0x4D, 0x65, 0x6D, 0x6F, 0x72, 0x79, 0x00,        // Memory\0
    ];
    let Name = _UNICODE_STRING {
        Length: 44,       // 22 chars * 2
        MaximumLength: 46, // 23 u16s * 2
        Buffer: PHYS_MEM_NAME.as_ptr() as *mut u16,
    };
    let mut Obj: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    Obj.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    Obj.RootDirectory = core::ptr::null_mut();
    Obj.ObjectName = core::ptr::addr_of!(Name) as *mut _;
    Obj.Attributes = OBJ_PERMANENT | OBJ_KERNEL_EXCLUSIVE;
    Obj.SecurityDescriptor = core::ptr::null_mut();
    Obj.SecurityQualityOfService = core::ptr::null_mut();

    let mut PhysSection: PSECTION = core::ptr::null_mut();
    let mut Status = ObCreateObject(
        0i32 as KPROCESSOR_MODE,
        MmSectionObjectType,
        core::ptr::addr_of_mut!(Obj),
        ExGetPreviousMode(),
        core::ptr::null_mut(),
        core::mem::size_of::<_SECTION>() as u32,
        0,
        0,
        core::ptr::addr_of_mut!(PhysSection) as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    core::ptr::write_bytes(PhysSection as *mut u8, 0, core::mem::size_of::<_SECTION>());
    (*PhysSection).u.Flags.set_filler(1);
    (*PhysSection).InitialPageProtection = PAGE_EXECUTE_READWRITE;
    (*PhysSection).u.Flags.set_PhysicalMemory(1);
    (*PhysSection).SizeOfSection = SectionSize;
    let Segment = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MM_SECTION_SEGMENT>(),
        TAG_MM_SECTION_SEGMENT,
    ) as PMM_SECTION_SEGMENT;
    if Segment.is_null() {
        ObDereferenceObject(PhysSection as PVOID);
        return STATUS_NO_MEMORY;
    }
    core::ptr::write_bytes(Segment as *mut u8, 0, core::mem::size_of::<MM_SECTION_SEGMENT>());
    (*PhysSection).Segment = Segment as *mut _;
    (*Segment).RefCount = 1;
    (*Segment).ReferenceCount = core::ptr::addr_of_mut!((*Segment).RefCount);
    (*Segment).Flags = core::ptr::addr_of_mut!((*Segment).SegFlags);
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
    (*Segment).Image.FileOffset = 0;
    (*Segment).Protection = PAGE_EXECUTE_READWRITE;
    (*Segment).RawLength = SectionSize;
    (*Segment).Length = SectionSize;
    (*Segment).SegFlags = MM_PHYSICALMEMORY_SEGMENT;
    (*Segment).WriteCopy = 0u8;
    (*Segment).Image.VirtualAddress = 0;
    (*Segment).Image.Characteristics = 0;
    MiInitializeSectionPageTable(Segment);

    let mut Handle: HANDLE = core::ptr::null_mut();
    Status = ObInsertObject(
        PhysSection as PVOID,
        core::ptr::null_mut(),
        SECTION_ALL_ACCESS,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Handle),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    ObCloseHandle(Handle, 0i32 as KPROCESSOR_MODE);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmInitSectionImplementation() -> NTSTATUS {
    let mut ObjectTypeInitializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    // L"Section" — UTF-16LE
    static SECTION_NAME: [u16; 8] = [
        0x53, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x00, // Section\0
    ];
    let Name = _UNICODE_STRING {
        Length: 14,        // 7 chars * 2
        MaximumLength: 16, // 8 u16s * 2
        Buffer: SECTION_NAME.as_ptr() as *mut u16,
    };

    MmSectionBasedRoot.BalancedRoot.Parent =
        core::ptr::addr_of_mut!(MmSectionBasedRoot.BalancedRoot);

    core::ptr::write_bytes(core::ptr::addr_of_mut!(ObjectTypeInitializer), 0, 1);
    ObjectTypeInitializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultPagedPoolCharge = core::mem::size_of::<_SECTION>() as ULONG;
    ObjectTypeInitializer.PoolType = PagedPool;
    ObjectTypeInitializer.UseDefaultObject = 1u8;
    ObjectTypeInitializer.GenericMapping = MmpSectionMapping;
    ObjectTypeInitializer.DeleteProcedure = Some(MmpDeleteSection);
    ObjectTypeInitializer.CloseProcedure = Some(MmpCloseSection);
    ObjectTypeInitializer.ValidAccessMask = SECTION_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    let Status = ObCreateObjectType(
        core::ptr::addr_of!(Name) as *mut _,
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(MmSectionObjectType),
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx( // nolint: bugcheck-code (0xDEAD0010 = deliberate diagnostic marker, not a stop code)
            0xDEAD0010,
            Status as _,
            ObjectTypeInitializer.Length as _,
            MmSectionObjectType as _,
            0,
        );
    }

    MmCreatePhysicalMemorySection();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCanFileBeTruncated(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    NewFileSize: PLARGE_INTEGER,
) -> BOOLEAN {
    let NewFileSize = opaque_ptr(NewFileSize);
    if !(*SectionObjectPointer).ImageSectionObject.is_null() {
        return 0u8;
    }
    let Segment = MiGrabDataSection(SectionObjectPointer);
    if Segment.is_null() {
        return 1u8;
    }
    MmLockSectionSegment(Segment as PVOID);
    let Ret = if (*Segment).SectionCount == 0
        || ((*Segment).SectionCount == 1 && !(*SectionObjectPointer).SharedCacheMap.is_null())
    {
        1u8
    } else if !NewFileSize.is_null() {
        if (*NewFileSize).QuadPart >= (*Segment).RawLength.QuadPart { 1u8 } else { 0u8 }
    } else {
        0u8
    };
    MmUnlockSectionSegment(Segment as PVOID);
    MmDereferenceSegment(Segment);
    Ret
}

unsafe fn MiPurgeImageSegment(Segment: PMM_SECTION_SEGMENT) -> BOOLEAN {
    MmLockSectionSegment(Segment as PVOID);
    let PageTable = RtlEnumerateGenericTable(core::ptr::addr_of_mut!((*Segment).PageTable), 1u8);
    if PageTable.is_null() {
        MmUnlockSectionSegment(Segment as PVOID);
        return 1u8;
    }
    MmUnlockSectionSegment(Segment as PVOID);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmFlushImageSection(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    FlushType: u32,
) -> BOOLEAN {
    if FlushType == 0 || FlushType == 1 {
        let mut OldIrql = MiAcquirePfnLock();
        let mut ImageSectionObject = (*SectionObjectPointer).ImageSectionObject as PMM_IMAGE_SECTION_OBJECT;

        while !ImageSectionObject.is_null()
            && ((*ImageSectionObject).SegFlags & (MM_SEGMENT_INCREATE | MM_SEGMENT_INDELETE) != 0)
        {
            MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            OldIrql = MiAcquirePfnLock();
            ImageSectionObject = (*SectionObjectPointer).ImageSectionObject as PMM_IMAGE_SECTION_OBJECT;
        }

        if ImageSectionObject.is_null() {
            MiReleasePfnLock(OldIrql);
            return 1u8;
        }

        if (*ImageSectionObject).SectionCount != 0 || (*ImageSectionObject).MapCount != 0 {
            MiReleasePfnLock(OldIrql);
            return 0u8;
        }

        (*ImageSectionObject).SegFlags |= 0x80000000;
        InterlockedIncrement64(core::ptr::addr_of_mut!((*ImageSectionObject).RefCount) as *mut i64);
        MiReleasePfnLock(OldIrql);

        for i in 0..(*ImageSectionObject).NrSegments {
            if MiPurgeImageSegment(&mut *(*ImageSectionObject).Segments.add(i as usize)) == 0 {
                break;
            }
        }

        OldIrql = MiAcquirePfnLock();
        if (*ImageSectionObject).SegFlags & 0x80000000 == 0 {
            MmDereferenceSegmentWithLock(&mut *(*ImageSectionObject).Segments, OldIrql);
            return 0u8;
        }
        MmDereferenceSegmentWithLock(&mut *(*ImageSectionObject).Segments, OldIrql);
        return 1u8;
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmMapViewInSystemSpace(
    SectionObject: PVOID,
    MappedBase: *mut PVOID,
    ViewSize: PSIZE_T,
) -> NTSTATUS {
    let SectionObject = opaque_ptr(SectionObject);
    let MappedBase = opaque_ptr(MappedBase);
    let ViewSize = opaque_ptr(ViewSize);
    let mut SectionOffset = LARGE_INTEGER { QuadPart: 0 };
    MmMapViewInSystemSpaceEx(SectionObject, MappedBase, ViewSize, core::ptr::addr_of_mut!(SectionOffset), 0)
}

#[no_mangle]
pub unsafe extern "system" fn MmMapViewInSystemSpaceEx(
    SectionObject: PVOID,
    MappedBase: *mut PVOID,
    ViewSize: PSIZE_T,
    SectionOffset: PLARGE_INTEGER,
    _Flags: ULONG_PTR,
) -> NTSTATUS {
    let SectionObject = opaque_ptr(SectionObject);
    let MappedBase = opaque_ptr(MappedBase);
    let ViewSize = opaque_ptr(ViewSize);
    let SectionOffset = opaque_ptr(SectionOffset);
    extern "C" {
    fn MmGetKernelAddressSpace() -> *mut core::ffi::c_void;
    fn MmLockAddressSpace(AddressSpace: *mut core::ffi::c_void);
    fn MmUnlockAddressSpace(AddressSpace: *mut core::ffi::c_void);
}
    let Section = SectionObject as PSECTION;
    if MiIsRosSectionObject(Section) == 0 {
        return MiMapViewInSystemSpace(
            SectionObject,
            core::ptr::addr_of_mut!(MmSession) as *mut core::ffi::c_void,
            MappedBase,
            ViewSize,
            SectionOffset,
        );
    }
    let Segment = (*Section).Segment as PMM_SECTION_SEGMENT;
    if *ViewSize == 0 {
        (*SectionOffset).__bindgen_anon_1.LowPart = PAGE_ROUND_DOWN((*SectionOffset).__bindgen_anon_1.LowPart as usize) as u32;
        let mut MapSizeLL: i64 = 0;
        if !NT_SUCCESS(RtlLongLongSub((*Section).SizeOfSection.QuadPart, (*SectionOffset).QuadPart, core::ptr::addr_of_mut!(MapSizeLL))) {
            return STATUS_INVALID_VIEW_SIZE;
        }
        if !NT_SUCCESS(RtlLongLongToSIZET(MapSizeLL, ViewSize as *mut usize)) {
            return STATUS_INVALID_VIEW_SIZE;
        }
    } else {
        let mut HelperLL: i64 = 0;
        if !NT_SUCCESS(RtlLongLongAdd((*SectionOffset).QuadPart, *ViewSize as i64, core::ptr::addr_of_mut!(HelperLL))) {
            return STATUS_INVALID_VIEW_SIZE;
        }
        if HelperLL % PAGE_SIZE as i64 != 0 {
            if !NT_SUCCESS(RtlLongLongAdd(HelperLL, PAGE_SIZE as i64 - (HelperLL % PAGE_SIZE as i64), core::ptr::addr_of_mut!(HelperLL))) {
                return STATUS_INVALID_VIEW_SIZE;
            }
        }
        (*SectionOffset).__bindgen_anon_1.LowPart = PAGE_ROUND_DOWN((*SectionOffset).__bindgen_anon_1.LowPart as usize) as u32;
        if !NT_SUCCESS(RtlLongLongSub(HelperLL, (*SectionOffset).QuadPart, core::ptr::addr_of_mut!(HelperLL))) {
            return STATUS_INVALID_VIEW_SIZE;
        }
        if !NT_SUCCESS(RtlLongLongToSIZET(HelperLL, ViewSize as *mut usize)) {
            return STATUS_INVALID_VIEW_SIZE;
        }
    }
    let AddressSpace = MmGetKernelAddressSpace();
    MmLockAddressSpace(AddressSpace);
    MmLockSectionSegment(Segment as PVOID);
    let Status = MmMapViewOfSegment(
        AddressSpace,
        (*Section).u.Flags.Image() as u8,
        Segment,
        MappedBase,
        *ViewSize as usize,
        PAGE_READWRITE,
        (*SectionOffset).QuadPart,
        SEC_RESERVE,
    );
    MmUnlockSectionSegment(Segment as PVOID);
    MmUnlockAddressSpace(AddressSpace);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateSection(
    Section: *mut PVOID,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaximumSize: PLARGE_INTEGER,
    SectionPageProtection: ULONG,
    mut AllocationAttributes: ULONG,
    FileHandle: HANDLE,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    let Section = opaque_ptr(Section);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let MaximumSize = opaque_ptr(MaximumSize);
    let FileHandle = opaque_ptr(FileHandle);
    let FileObject = opaque_ptr(FileObject);
    if AllocationAttributes & SEC_LARGE_PAGES != 0 {
        return 0xC000000D_u32 as i32;
    }

    if AllocationAttributes & SEC_IMAGE == 0 {
        if FileObject.is_null() && FileHandle.is_null() {
            return MmCreateArm3Section(
                Section,
                DesiredAccess,
                ObjectAttributes,
                MaximumSize,
                SectionPageProtection,
                AllocationAttributes & !1,
                FileHandle,
                FileObject,
            );
        }
    }

    let Protection = MiMakeProtectionMask(SectionPageProtection);
    if Protection == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    let mut FileObject = FileObject;
    let mut FileLock = 0u8;
    let mut HaveFileObject = 0u8;

    if !FileHandle.is_null() || !FileObject.is_null() {
        if AllocationAttributes & SEC_LARGE_PAGES != 0 {
            return 0xC000000D_u32 as i32;
        }

        if !FileObject.is_null() {
            ObReferenceObject(FileObject as PVOID);
            HaveFileObject = 1u8;
        } else {
            let Status = ObReferenceObjectByHandle(
                FileHandle,
                MmMakeFileAccess[Protection as usize],
                IoFileObjectType,
                ExGetPreviousMode(),
                core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            let Status = FsRtlAcquireToCreateMappedSection(FileObject, SectionPageProtection);
            if !NT_SUCCESS(Status) {
                ObDereferenceObject(FileObject as PVOID);
                return Status;
            }
            FileLock = 1u8;
        }
    } else {
        if AllocationAttributes & SEC_IMAGE != 0 {
            return STATUS_INVALID_FILE_FOR_SECTION;
        }
    }

    if FileObject.is_null() {
        if FileLock != 0 {
            FsRtlReleaseFile(FileObject);
        }
        if !FileObject.is_null() {
            ObDereferenceObject(FileObject as PVOID);
        }
        return STATUS_INVALID_FILE_FOR_SECTION;
    }

    let mut Status = if AllocationAttributes & SEC_IMAGE != 0 {
        MmCreateImageSection(
            Section as *mut PSECTION,
            DesiredAccess,
            ObjectAttributes,
            MaximumSize,
            SectionPageProtection,
            AllocationAttributes,
            FileObject,
        )
    } else {
        STATUS_NOT_IMPLEMENTED
    };

    if !NT_SUCCESS(Status) && HaveFileObject != 0 {
        AllocationAttributes &= !SEC_IMAGE;
    }

    if AllocationAttributes & SEC_IMAGE == 0 {
        Status = MmCreateDataFileSection(
            Section as *mut PSECTION,
            DesiredAccess,
            ObjectAttributes,
            MaximumSize,
            SectionPageProtection,
            AllocationAttributes,
            FileObject,
            (FileHandle != core::ptr::null_mut()) as u8,
        );
    }

    if FileLock != 0 {
        FsRtlReleaseFile(FileObject);
    }
    if !FileObject.is_null() {
        ObDereferenceObject(FileObject as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmArePagesResident(
    Process: *mut _EPROCESS,
    Address: PVOID,
    Length: ULONG,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let address_space = if !Process.is_null() {
        core::ptr::addr_of_mut!((*Process).Vm) as PVOID
    } else {
        MmGetKernelAddressSpace()
    };

    MmLockAddressSpace(address_space);

    let memory_area = MmLocateMemoryAreaByAddress(address_space, Address);
    if memory_area.is_null() {
        MmUnlockAddressSpace(address_space);
        return 0u8;
    }

    let segment = (*memory_area).SectionData.Segment;
    MmLockSectionSegment(segment as PVOID);

    let mut segment_offset = LARGE_INTEGER { QuadPart: 0 };
    segment_offset.QuadPart = PAGE_ROUND_DOWN(Address as usize) as i64
        - MA_GetStartingAddress(memory_area) as i64
        + (*memory_area).SectionData.ViewOffset as i64;

    let mut range_end = LARGE_INTEGER { QuadPart: 0 };
    range_end.QuadPart = ((Address as usize + Length as usize + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)) as i64
        - MA_GetStartingAddress(memory_area) as i64
        + (*memory_area).SectionData.ViewOffset as i64;

    let mut ret = 1u8;
    while segment_offset.QuadPart < range_end.QuadPart {
        let entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(segment_offset.QuadPart));
        if entry == 0 || IS_SWAP_FROM_SSE(entry) {
            ret = 0u8;
            break;
        }
        segment_offset.QuadPart += PAGE_SIZE as i64;
    }

    MmUnlockSectionSegment(segment as PVOID);
    MmUnlockAddressSpace(address_space);
    ret
}

static mut TINY_TIME: LARGE_INTEGER = LARGE_INTEGER { QuadPart: -1 };

#[no_mangle]
pub unsafe extern "system" fn MmPurgeSegment(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    Offset: PLARGE_INTEGER,
    Length: ULONG,
) -> BOOLEAN {
    let Offset = opaque_ptr(Offset);
    let mut purge_start = LARGE_INTEGER { QuadPart: 0 };
    let mut purge_end = LARGE_INTEGER { QuadPart: 0 };

    purge_start.QuadPart = if !Offset.is_null() {
        (*Offset).QuadPart
    } else {
        0i64
    };

    if Length != 0 && !Offset.is_null() {
        let mut end_val: i64 = 0;
        if (purge_start.QuadPart).checked_add(Length as i64).is_none() {
            return 0u8;
        }
        purge_end.QuadPart = purge_start.QuadPart + Length as i64;
    }

    let segment = MiGrabDataSection(SectionObjectPointer);
    if segment.is_null() {
        return 1u8;
    }

    MmLockSectionSegment(segment as PVOID);

    if Length == 0 || Offset.is_null() {
        let elem_count = RtlNumberGenericTableElements(
            core::ptr::addr_of_mut!((*segment).PageTable)
        );
        if elem_count == 0 {
            MmUnlockSectionSegment(segment as PVOID);
            MmDereferenceSegment(segment);
            return 1u8;
        }

        let page_table = RtlGetElementGenericTable(
            core::ptr::addr_of_mut!((*segment).PageTable),
            elem_count - 1,
        ) as PCACHE_SECTION_PAGE_TABLE;
        purge_end.QuadPart = (*page_table).FileOffset.QuadPart
            + (core::mem::size_of::<[usize; 256]>() / core::mem::size_of::<usize>()) as i64 * PAGE_SIZE as i64;
    }

    purge_start.QuadPart = (purge_start.QuadPart as u64 & !(PAGE_SIZE as u64 - 1)) as i64;

    while purge_start.QuadPart < purge_end.QuadPart {
        let entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(purge_start.QuadPart));

        if entry == 0 {
            purge_start.QuadPart += PAGE_SIZE as i64;
            continue;
        }

        if IS_SWAP_FROM_SSE(entry) {
            MmUnlockSectionSegment(segment as PVOID);
            MmDereferenceSegment(segment);
            return 0u8;
        }

        if IS_WRITE_SSE(entry) {
            MmUnlockSectionSegment(segment as PVOID);
            KeDelayExecutionThread(0i8, 0u8, &mut TINY_TIME);
            MmLockSectionSegment(segment as PVOID);
            continue;
        }

        if SHARE_COUNT_FROM_SSE(entry) > 0 {
            MmUnlockSectionSegment(segment as PVOID);
            MmDereferenceSegment(segment);
            return 0u8;
        }

        MmSetPageEntrySectionSegment(segment, &mut purge_start, 0);
        MmReleasePageMemoryConsumer(MC_USER, PFN_FROM_SSE(entry));

        purge_start.QuadPart += PAGE_SIZE as i64;
    }

    MmUnlockSectionSegment(segment as PVOID);
    MmDereferenceSegment(segment);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn MmIsDataSectionResident(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    Offset: i64,
    Length: ULONG,
) -> BOOLEAN {
    let mut range_start = LARGE_INTEGER { QuadPart: Offset };
    let mut range_end = LARGE_INTEGER { QuadPart: 0 };

    if (range_start.QuadPart).checked_add(Length as i64).is_none() {
        return 0u8;
    }
    range_end.QuadPart = range_start.QuadPart + Length as i64;

    let segment = MiGrabDataSection(SectionObjectPointer);
    if segment.is_null() {
        return 0u8;
    }

    range_start.QuadPart = (range_start.QuadPart as u64 & !(PAGE_SIZE as u64 - 1)) as i64;

    MmLockSectionSegment(segment as PVOID);

    let mut ret = 1u8;
    while range_start.QuadPart < range_end.QuadPart {
        let entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(range_start.QuadPart));
        if entry == 0 || IS_SWAP_FROM_SSE(entry) {
            ret = 0u8;
            break;
        }
        range_start.QuadPart += PAGE_SIZE as i64;
    }

    MmUnlockSectionSegment(segment as PVOID);
    MmDereferenceSegment(segment);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn MmMakeDataSectionResident(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    Offset: i64,
    Length: ULONG,
    ValidDataLength: PLARGE_INTEGER,
) -> NTSTATUS {
    let ValidDataLength = opaque_ptr(ValidDataLength);
    let segment = MiGrabDataSection(SectionObjectPointer);

    let status = MmMakeSegmentResident(segment, Offset, Length, ValidDataLength, 0u8);
    MmDereferenceSegment(segment);
    status
}

#[no_mangle]
pub unsafe extern "system" fn MmMakeSegmentDirty(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    Offset: i64,
    Length: ULONG,
) -> NTSTATUS {
    let mut range_start = LARGE_INTEGER { QuadPart: Offset };
    let mut range_end = LARGE_INTEGER { QuadPart: 0 };

    if (range_start.QuadPart).checked_add(Length as i64).is_none() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    range_end.QuadPart = range_start.QuadPart + Length as i64;

    let segment = MiGrabDataSection(SectionObjectPointer);
    if segment.is_null() {
        return 0xC0000019u32 as i32;
    }

    range_start.QuadPart = (range_start.QuadPart as u64 & !(PAGE_SIZE as u64 - 1)) as i64;

    MmLockSectionSegment(segment as PVOID);

    while range_start.QuadPart < range_end.QuadPart {
        let mut entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(range_start.QuadPart));

        while MM_IS_WAIT_PTE(entry) {
            MmUnlockSectionSegment(segment as PVOID);
            KeDelayExecutionThread(0i8, 0u8, &mut TINY_TIME);
            MmLockSectionSegment(segment as PVOID);
            entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(range_start.QuadPart));
        }

        if entry != 0 {
            MmSetPageEntrySectionSegment(segment, &mut range_start, DIRTY_SSE(entry));
        }

        range_start.QuadPart += PAGE_SIZE as i64;
    }

    MmUnlockSectionSegment(segment as PVOID);
    MmDereferenceSegment(segment);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmFlushSegment(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    Offset: PLARGE_INTEGER,
    Length: ULONG,
    Iosb: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let Offset = opaque_ptr(Offset);
    let Iosb = opaque_ptr(Iosb);
    let mut flush_start = LARGE_INTEGER { QuadPart: 0 };
    let mut flush_end = LARGE_INTEGER { QuadPart: 0 };

    if !Offset.is_null() {
        flush_start = *Offset;
        if (flush_start.QuadPart).checked_add(Length as i64).is_none() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        flush_end.QuadPart = flush_start.QuadPart + Length as i64;
    }

    if !Iosb.is_null() {
        (*Iosb).Information = 0;
    }

    let segment = MiGrabDataSection(SectionObjectPointer);
    if segment.is_null() {
        if !Iosb.is_null() {
            (*Iosb).__bindgen_anon_1.Status = STATUS_SUCCESS;
        }
        return STATUS_SUCCESS;
    }

    MmLockSectionSegment(segment as PVOID);

    if Offset.is_null() {
        flush_start.QuadPart = 0;

        let elem_count = RtlNumberGenericTableElements(
            core::ptr::addr_of_mut!((*segment).PageTable)
        );
        if elem_count == 0 {
            MmUnlockSectionSegment(segment as PVOID);
            MmDereferenceSegment(segment);
            if !Iosb.is_null() {
                (*Iosb).__bindgen_anon_1.Status = STATUS_SUCCESS;
            }
            return STATUS_SUCCESS;
        }

        let page_table = RtlGetElementGenericTable(
            core::ptr::addr_of_mut!((*segment).PageTable),
            elem_count - 1,
        ) as PCACHE_SECTION_PAGE_TABLE;
        flush_end.QuadPart = (*page_table).FileOffset.QuadPart
            + (core::mem::size_of::<[usize; 256]>() / core::mem::size_of::<usize>()) as i64 * PAGE_SIZE as i64;
    }

    flush_start.QuadPart = (flush_start.QuadPart as u64 & !(PAGE_SIZE as u64 - 1)) as i64;

    while flush_start.QuadPart < flush_end.QuadPart {
        let entry = MmGetPageEntrySectionSegment(segment as PVOID, core::ptr::addr_of!(flush_start.QuadPart));

        if IS_DIRTY_SSE(entry) {
            MmCheckDirtySegment(segment, &mut flush_start, 0u8, 0u8);

            if !Iosb.is_null() {
                (*Iosb).Information += PAGE_SIZE as ntkrnl_sys::ddk::ULONG_PTR;
            }
        }

        flush_start.QuadPart += PAGE_SIZE as i64;
    }

    MmUnlockSectionSegment(segment as PVOID);
    MmDereferenceSegment(segment);

    if !Iosb.is_null() {
        (*Iosb).__bindgen_anon_1.Status = STATUS_SUCCESS;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCheckDirtySegment(
    Segment: PMM_SECTION_SEGMENT,
    Offset: PLARGE_INTEGER,
    ForceDirty: BOOLEAN,
    PageOut: BOOLEAN,
) -> BOOLEAN {
    let Offset = opaque_ptr(Offset);
    let mut entry = MmGetPageEntrySectionSegment(Segment as PVOID, Offset as *const i64);
    if entry == 0 {
        return 0u8;
    }

    let page = PFN_FROM_SSE(entry);
    if IS_DIRTY_SSE(entry) || ForceDirty != 0 {
        entry = MAKE_SSE(PAGE_FROM_SSE(entry), SHARE_COUNT_FROM_SSE(entry) + 1);
        entry = WRITE_SSE(entry);
        MmSetPageEntrySectionSegment(Segment, Offset, entry);

        MmUnlockSectionSegment(Segment as PVOID);

        let mut status: NTSTATUS;
        if (*(*Segment).Flags & MM_DATAFILE_SEGMENT) != 0 {
            let mut resource_to_release: PERESOURCE = core::ptr::null_mut();
            let mut old_irql: KIRQL = 0;

            if PageOut != 0 {
                let mut end_offset = *Offset;
                end_offset.QuadPart += PAGE_SIZE as i64;

                KeRaiseIrql(APC_LEVEL, &mut old_irql);

                status = FsRtlAcquireFileForModWriteEx(
                    (*Segment).FileObject,
                    &mut end_offset,
                    &mut resource_to_release,
                );
                if status >= 0 {
                    IoSetTopLevelIrp(0x80000000usize as PIRP);
                } else {
                    resource_to_release = core::ptr::null_mut();
                }
            } else {
                status = STATUS_SUCCESS;
            }

            if status >= 0 {
                status = MiWritePage(Segment, (*Offset).QuadPart, page);
            } else {
                static REPORTED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                if REPORTED.fetch_add(1, core::sync::atomic::Ordering::Relaxed) < 8 {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        c"[MM] page-out of %wZ at %I64x skipped: FsRtlAcquireFileForModWriteEx %08lx\n".as_ptr(),
                        core::ptr::addr_of!((*(*Segment).FileObject).FileName),
                        (*Offset).QuadPart,
                        status,
                    );
                }
            }

            if PageOut != 0 {
                IoSetTopLevelIrp(core::ptr::null_mut());
                if !resource_to_release.is_null() {
                    FsRtlReleaseFileForModWrite((*Segment).FileObject, resource_to_release);
                }
                KeLowerIrql(old_irql);
            }
        } else {
            let mut swap_entry = MmGetSavedSwapEntryPage(page);
            if swap_entry == 0 {
                swap_entry = MmAllocSwapPage();
            }

            if swap_entry != 0 {
                status = MmWriteToSwapPage(swap_entry, page);
                if status >= 0 {
                    MmSetSavedSwapEntryPage(page, swap_entry);
                } else {
                    MmFreeSwapPage(swap_entry);
                }
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        MmLockSectionSegment(Segment as PVOID);

        entry = MmGetPageEntrySectionSegment(Segment as PVOID, Offset as *const i64);

        let dirty_again = if status < 0 { 1u8 } else { IS_DIRTY_SSE(entry) as u8 };

        entry = MAKE_SSE(page << PAGE_SHIFT, SHARE_COUNT_FROM_SSE(entry) - 1);
        if dirty_again != 0 {
            entry = DIRTY_SSE(entry);
        }
        MmSetPageEntrySectionSegment(Segment, Offset, entry);
    }

    if !IS_DIRTY_SSE(entry) && SHARE_COUNT_FROM_SSE(entry) == 0 && PageOut != 0 {
        let mut new_entry: usize = 0;
        if (*(*Segment).Flags & MM_DATAFILE_SEGMENT) == 0 {
            let swap_entry = MmGetSavedSwapEntryPage(page);
            if swap_entry != 0 {
                new_entry = MAKE_SWAP_SSE(swap_entry);
            }
        }

        MmSetPageEntrySectionSegment(Segment, Offset, new_entry);
        MmReleasePageMemoryConsumer(MC_USER, page);
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmExtendSection(
    _Section: PVOID,
    NewSize: PLARGE_INTEGER,
) -> NTSTATUS {
    let _Section = opaque_ptr(_Section);
    let NewSize = opaque_ptr(NewSize);
    let section = _Section as PSECTION;

    if (*section).u.Flags.Image() != 0 {
        return 0xC0000087u32 as i32;
    }

    if (*section).u.Flags.File() == 0 {
        return 0xC0000087u32 as i32;
    }

    if MiIsRosSectionObject(section) == 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    if (*NewSize).QuadPart > (*section).SizeOfSection.QuadPart {
        let segment = (*section).Segment as PMM_SECTION_SEGMENT;
        (*section).SizeOfSection = *NewSize;

        if (*section).u.Flags.Reserve() == 0 {
            MmLockSectionSegment(segment as PVOID);
            if (*segment).RawLength.QuadPart < (*NewSize).QuadPart {
                (*segment).RawLength = *NewSize;
                (*segment).Length.QuadPart = ((*NewSize).QuadPart + PAGE_SIZE as i64 - 1) & !((PAGE_SIZE as i64) - 1);
            }
            MmUnlockSectionSegment(segment as PVOID);
        }
    }

    STATUS_SUCCESS
}

extern "system" {
    // DUPLICATE: fn MiGrabDataSection(SectionObjectPointer: PSECTION_OBJECT_POINTERS) -> PMM_SECTION_SEGMENT;
    // DUPLICATE: fn MmMakeSegmentResident( Segment: PMM_SECTION_SEGMENT, Offset: i64, Length: ULONG, ValidDataLength: PLARGE_INTEGER, CacheFriendly: BOOLEAN, ) -> NTSTATUS;
    // DUPLICATE: fn MiWritePage(Segment: PMM_SECTION_SEGMENT, Offset: i64, Page: PFN_NUMBER) -> NTSTATUS;
    // DUPLICATE: fn FsRtlAcquireFileForModWriteEx( FileObject: PVOID, EndingOffset: PLARGE_INTEGER, ResourceToRelease: *mut PERESOURCE, ) -> NTSTATUS;
    // DUPLICATE: fn FsRtlReleaseFileForModWrite(FileObject: PVOID, ResourceToRelease: PERESOURCE);
    // DUPLICATE: fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    // DUPLICATE: fn KeLowerIrql(NewIrql: KIRQL);
}

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<_CACHE_SECTION_PAGE_TABLE>() == 2072);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, FileOffset) == 0);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, Segment) == 8);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, Refcount) == 16);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, PageEntries) == 24);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<_CACHE_SECTION_PAGE_TABLE>() == 1040);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, FileOffset) == 0);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, Segment) == 8);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, Refcount) == 12);
    assert!(core::mem::offset_of!(_CACHE_SECTION_PAGE_TABLE, PageEntries) == 16);
};
// END OFFSET CHECKS

// ============================================================
// New functions integrated from section_stitched_chunks.rs
// ============================================================

use ntkrnl_sys::ddk::{
    _SECTION_BASIC_INFORMATION,
    _SECTION_IMAGE_INFORMATION,
    SECTION_IMAGE_INFORMATION,
    SECTION_BASIC_INFORMATION,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const SectionBasicInformation: SECTION_INFORMATION_CLASS = 0;
const SectionImageInformation: SECTION_INFORMATION_CLASS = 1;
const MM_IMAGE_SECTION_FLUSH_DELETE: ULONG = 0x10;

// These are FORCEINLINE shims — cdecl calling convention (no NTAPI)
extern "C" {
    fn MmGetAddressSpaceOwner(AddressSpace: PVOID) -> PEPROCESS;
}

extern "C" {
    fn InterlockedIncrementUL(Addend: *mut ULONG) -> ULONG;
}

extern "system" {
    fn MmGetPageFileMapping(Process: PEPROCESS, Address: PVOID, SwapEntry: *mut SWAPENTRY);
    fn MmIsPagePresent(Process: PEPROCESS, Address: PVOID) -> BOOLEAN;
    fn MmIsDisabledPage(Process: PEPROCESS, Address: PVOID) -> BOOLEAN;
    fn MmIsPageSwapEntry(Process: PEPROCESS, Address: PVOID) -> BOOLEAN;
    fn MmDeletePageFileMapping(Process: PEPROCESS, Address: PVOID, SwapEntry: *mut SWAPENTRY);
    fn MmCreatePageFileMapping(Process: PEPROCESS, Address: PVOID, SwapEntry: SWAPENTRY) -> NTSTATUS;
    fn MmRequestPageMemoryConsumer(Type: ULONG, CanWait: BOOLEAN, Page: *mut PFN_NUMBER) -> NTSTATUS;
    fn MmReadFromSwapPage(SwapEntry: ULONG_PTR, Page: PFN_NUMBER) -> NTSTATUS;
    fn MmCreateVirtualMapping(Process: PEPROCESS, Address: PVOID, Protect: ULONG, Page: PFN_NUMBER) -> NTSTATUS;
    fn MmCreatePhysicalMapping(Process: PEPROCESS, Address: PVOID, Protect: ULONG, Page: PFN_NUMBER) -> NTSTATUS;
    fn MmInsertRmap(Page: PFN_NUMBER, Process: PEPROCESS, Address: PVOID);
    fn MmFindRegion(BaseAddress: PVOID, RegionListHead: *mut LIST_ENTRY, Address: PVOID, RegionBaseAddress: *mut PVOID) -> PMM_REGION;
    fn MmAlterRegion(AddressSpace: PVOID, BaseAddress: PVOID, RegionListHead: *mut LIST_ENTRY, Address: PVOID, Length: usize, Type: ULONG, Protect: ULONG, AlterFunc: *const core::ffi::c_void) -> NTSTATUS;
    fn MmGetPageProtect(Process: PEPROCESS, Address: PVOID) -> ULONG;
    fn MmSetPageProtect(Process: PEPROCESS, Address: PVOID, Protect: ULONG);
    fn MmGetPfnForProcess(Process: PEPROCESS, Address: PVOID) -> PFN_NUMBER;
    fn MmDeleteRmap(Page: PFN_NUMBER, Process: PEPROCESS, Address: PVOID);
    fn MmCreateMemoryArea(AddressSpace: PVOID, Type: ULONG, BaseAddress: *mut PVOID, Length: SIZE_T, Protect: ULONG, MemoryArea: *mut PMEMORY_AREA, AllocationType: ULONG, Granularity: ULONG) -> NTSTATUS;
    fn MmInitializeRegion(RegionListHead: *mut LIST_ENTRY, Length: SIZE_T, Type: ULONG, Protect: ULONG);
    fn MmFreeMemoryArea(AddressSpace: PVOID, MemoryArea: PMEMORY_AREA, FreePage: Option<unsafe extern "C" fn(PVOID, PMEMORY_AREA, PVOID, PFN_NUMBER, ULONG_PTR, BOOLEAN)>, FreePageContext: PVOID) -> NTSTATUS;
    fn MmIsAddressRangeFree(AddressSpace: PVOID, BaseAddress: PVOID, Length: usize) -> BOOLEAN;
    fn MmFindGap(AddressSpace: PVOID, Length: usize, Alignment: usize, TopDown: BOOLEAN) -> PVOID;
}

// MmDeleteVirtualMapping is NTAPI/stdcall on i386 (extern "system"), NOT cdecl — the no-NTAPI
// decl in internal/mm.h is misleading; the working ABI is stdcall (@16), matching the def and
// the VirtFn transmute-call in marea.rs. extern "C" here desyncs ESP → exec-at-0 (field manual §9.99).
extern "system" {
    fn MmDeleteVirtualMapping(Process: PEPROCESS, Address: PVOID, WasDirty: *mut BOOLEAN, Page: *mut PFN_NUMBER) -> BOOLEAN;
}

#[inline]
fn PAGE_ROUND_DOWN_USIZE(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

#[inline]
fn MM_ROUND_DOWN_ULONG(addr: ULONG_PTR, size: usize) -> ULONG_PTR {
    addr & !(size as ULONG_PTR - 1)
}

#[inline]
unsafe fn IsListEmpty_inline(list: PLIST_ENTRY) -> bool {
    (*list).Flink == list
}

#[inline]
unsafe fn RemoveHeadList_inline(list: PLIST_ENTRY) -> PLIST_ENTRY {
    let entry = (*list).Flink;
    let flink = (*entry).Flink;
    (*list).Flink = flink;
    (*flink).Blink = list;
    entry
}

#[no_mangle]
#[cfg(not(feature = "NEWCC"))]
pub unsafe extern "system" fn _MmLockSectionSegment(Segment: PMM_SECTION_SEGMENT, _file: *const i8, _line: i32) {
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
    (*Segment).Locked = 1u8;
}

#[no_mangle]
#[cfg(not(feature = "NEWCC"))]
pub unsafe extern "system" fn _MmUnlockSectionSegment(Segment: PMM_SECTION_SEGMENT, _file: *const i8, _line: i32) {
    (*Segment).Locked = 0u8;
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Segment).Lock));
}

unsafe fn MmAlterViewAttributes(
    AddressSpace: PVOID,
    BaseAddress: PVOID,
    RegionSize: SIZE_T,
    _OldType: ULONG,
    OldProtect: ULONG,
    _NewType: ULONG,
    NewProtect: ULONG,
) {
    let mut DoCOW = 0u8;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, BaseAddress);
    let Segment = (*MemoryArea).SectionData.Segment;
    MmLockSectionSegment(Segment as PVOID);

    if (*Segment).WriteCopy != 0 && (NewProtect == PAGE_READWRITE || NewProtect == PAGE_EXECUTE_READWRITE) {
        DoCOW = 1u8;
    }

    if OldProtect != NewProtect {
        let mut i = 0;
        while i < PAGE_ROUND_UP(RegionSize) / PAGE_SIZE {
            let mut SwapEntry: SWAPENTRY = 0;
            let Address = (BaseAddress as ULONG_PTR + i * PAGE_SIZE) as PVOID;
            let mut Protect = NewProtect;

            loop {
                MmGetPageFileMapping(Process, Address, &mut SwapEntry);
                if SwapEntry != MM_WAIT_ENTRY as SWAPENTRY { break; }
                MmUnlockSectionSegment(Segment as PVOID);
                MmUnlockAddressSpace(AddressSpace);
                KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
                MmLockAddressSpace(AddressSpace);
                MmLockSectionSegment(Segment as PVOID);
            }

            if DoCOW != 0 && (MmIsPagePresent(Process, Address) != 0 || MmIsDisabledPage(Process, Address) != 0) {
                let mut Offset = LARGE_INTEGER { QuadPart: 0 };
                Offset.QuadPart = (Address as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
                    + (*MemoryArea).SectionData.ViewOffset;
                let Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
                let Page = MmGetPfnForProcess(Process, Address);

                Protect = if NewProtect == PAGE_EXECUTE_READWRITE {
                    PAGE_EXECUTE_READ
                } else {
                    PAGE_READONLY
                };

                if IS_SWAP_FROM_SSE(Entry) || PFN_FROM_SSE(Entry) != Page {
                    Protect = NewProtect;
                }
            }

            if MmIsPagePresent(Process, Address) != 0 || MmIsDisabledPage(Process, Address) != 0 {
                MmSetPageProtect(Process, Address, Protect);
            }
            i += 1;
        }
    }

    MmUnlockSectionSegment(Segment as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn MmNotPresentFaultSectionView(
    AddressSpace: PVOID,
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    Locked: BOOLEAN,
) -> NTSTATUS {
    let AddressSpace = opaque_ptr(AddressSpace);
    let MemoryArea = opaque_ptr(MemoryArea);
    let Address = opaque_ptr(Address);
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let mut Page: PFN_NUMBER = 0;
    let mut Status: NTSTATUS;
    let mut Entry: ULONG_PTR;
    let mut Entry1: ULONG_PTR;
    let mut Region: PMM_REGION;
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

    let PAddress = MM_ROUND_DOWN_ULONG(Address as ULONG_PTR, PAGE_SIZE) as PVOID;
    Offset.QuadPart = (PAddress as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
        + (*MemoryArea).SectionData.ViewOffset;

    let Segment = (*MemoryArea).SectionData.Segment;
    Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        Address,
        core::ptr::null_mut(),
    );

    /* Check for a NOACCESS mapping */
    if ((*Region).Protect & PAGE_NOACCESS) != 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    if ((*Region).Protect & PAGE_GUARD) != 0 {
        MmAlterRegion(
            AddressSpace,
            MA_GetStartingAddress(MemoryArea) as PVOID,
            core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
            Address,
            PAGE_SIZE,
            (*Region).Type,
            (*Region).Protect & !PAGE_GUARD,
            MmAlterViewAttributes as *const core::ffi::c_void,
        );
        return STATUS_GUARD_PAGE_VIOLATION;
    }

    HasSwapEntry = MmIsPageSwapEntry(Process, Address);

    if HasSwapEntry != 0 {
        let mut DummyEntry: SWAPENTRY = 0;
        MmGetPageFileMapping(Process, Address, &mut SwapEntry);
        if SwapEntry == MM_WAIT_ENTRY as SWAPENTRY {
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        MI_SET_USAGE(MI_USAGE_SECTION);
        MI_SET_PROCESS2("");
        Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut Page);
        if Status < 0 {
            return STATUS_NO_MEMORY;
        }

        MmDeletePageFileMapping(Process, Address, &mut DummyEntry);
        MmCreatePageFileMapping(Process, Address, MM_WAIT_ENTRY as SWAPENTRY);
        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry as ULONG_PTR, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmLockAddressSpace(AddressSpace);
        MmDeletePageFileMapping(Process, PAddress, &mut DummyEntry);

        Status = MmCreateVirtualMapping(Process, PAddress, (*Region).Protect, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmSetSavedSwapEntryPage(Page, SwapEntry as ULONG_PTR);
        if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
        return STATUS_SUCCESS;
    }

    MmLockSectionSegment(Segment as PVOID);

    if ((*(*Segment).Flags) & MM_PHYSICALMEMORY_SEGMENT) != 0 {
        MmUnlockSectionSegment(Segment as PVOID);
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

    Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
    if Entry == 0 {
        if (Offset.QuadPart >= PAGE_ROUND_UP((*Segment).RawLength.QuadPart as usize) as i64)
            && ((*MemoryArea).VadNode.u.VadFlags.VadType() as u32 == VadImageMap) {
            MI_SET_USAGE(MI_USAGE_SECTION);
            MI_SET_PROCESS2("");
            Status = MmRequestPageMemoryConsumer(MC_USER, 0u8, &mut Page);
            if Status < 0 {
                MmUnlockSectionSegment(Segment as PVOID);
                return STATUS_NO_MEMORY;
            }
            MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset), MAKE_SSE(Page << PAGE_SHIFT, 1));
            MmUnlockSectionSegment(Segment as PVOID);

            Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
            if Status < 0 {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
            if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
            return STATUS_SUCCESS;
        }

        MmUnlockSectionSegment(Segment as PVOID);
        MmUnlockAddressSpace(AddressSpace);

        FsRtlAcquireFileExclusive((*Segment).FileObject);
        let FcbHeader = (*(*Segment).FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
        Status = MmMakeSegmentResident(Segment, Offset.QuadPart as i64, PAGE_SIZE as ULONG, core::ptr::addr_of_mut!((*FcbHeader).ValidDataLength), 0u8);
        FsRtlReleaseFile((*Segment).FileObject);

        MmLockAddressSpace(AddressSpace);
        if Status < 0 {
            // The C passes STATUS_NO_MEMORY through: MmAccessFault answers it by
            // rebalancing memory and retrying the fault. The translation tested
            // STATUS_END_OF_FILE here instead, so a page-in that only lacked a free
            // page under load became a hard STATUS_IN_PAGE_ERROR -- an amd64 modrun
            // boot bugchecked 0x1E on it, reading a new process's image header.
            if Status == STATUS_NO_MEMORY {
                return Status;
            }
            ntkrnl_sys::driver_ddk::DbgPrint(
                c"[MM] Failed to page data in! st=%08lx off=%I64x fo=%p\n".as_ptr(),
                Status,
                Offset.QuadPart,
                (*Segment).FileObject,
            );
            return STATUS_IN_PAGE_ERROR;
        }
        return STATUS_MM_RESTART_OPERATION;
    } else if IS_SWAP_FROM_SSE(Entry) {
        let SwapEntry = SWAPENTRY_FROM_SSE(Entry);

        if SwapEntry == MM_WAIT_ENTRY {
            MmUnlockSectionSegment(Segment as PVOID);
            MmUnlockAddressSpace(AddressSpace);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            MmLockAddressSpace(AddressSpace);
            return STATUS_MM_RESTART_OPERATION;
        }

        Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut Page);
        if Status < 0 {
            MmUnlockSectionSegment(Segment as PVOID);
            return STATUS_NO_MEMORY;
        }

        MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset), MAKE_SWAP_SSE(MM_WAIT_ENTRY));
        MmUnlockSectionSegment(Segment as PVOID);
        MmUnlockAddressSpace(AddressSpace);

        Status = MmReadFromSwapPage(SwapEntry, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment as PVOID);

        Entry1 = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
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
        MmSetPageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset), Entry);
        MmUnlockSectionSegment(Segment as PVOID);
        return STATUS_SUCCESS;
    } else {
        Page = PFN_FROM_SSE(Entry);

        Status = MmCreateVirtualMapping(Process, PAddress, Attributes, Page);
        if Status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        if !Process.is_null() { MmInsertRmap(Page, Process, Address); }
        MmSharePageEntrySectionSegment(Segment, core::ptr::addr_of_mut!(Offset));
        MmUnlockSectionSegment(Segment as PVOID);
        return STATUS_SUCCESS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmAccessFaultSectionView(
    AddressSpace: PVOID,
    MemoryArea: *mut MEMORY_AREA,
    Address: PVOID,
    _Locked: BOOLEAN,
) -> NTSTATUS {
    let AddressSpace = opaque_ptr(AddressSpace);
    let MemoryArea = opaque_ptr(MemoryArea);
    let Address = opaque_ptr(Address);
    let Segment: PMM_SECTION_SEGMENT;
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
    let _Unmapped: BOOLEAN;
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
        Status = MmNotPresentFaultSectionView(AddressSpace, MemoryArea, Address, 1u8);
        if !NT_SUCCESS(Status) {
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
            MmAlterViewAttributes as *const core::ffi::c_void,
        );
    }

    PAddress = MM_ROUND_DOWN_ULONG(Address as ULONG_PTR, PAGE_SIZE) as PVOID;
    Offset.QuadPart = (PAddress as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64
        + (*MemoryArea).SectionData.ViewOffset;

    MmLockSectionSegment(Segment as PVOID);
    Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));

    OldPage = MmGetPfnForProcess(Process, PAddress);

    if IS_SWAP_FROM_SSE(Entry) || PFN_FROM_SSE(Entry) != OldPage {
        MmUnlockSectionSegment(Segment as PVOID);
        MmSetPageProtect(Process, PAddress, NewProtect);
        return STATUS_SUCCESS;
    }

    Status = MmRequestPageMemoryConsumer(MC_USER, 1u8, &mut NewPage);
    if !NT_SUCCESS(Status) {
        MmUnlockSectionSegment(Segment as PVOID);
        return STATUS_NO_MEMORY;
    }

    let _ = MiCopyFromUserPage(NewPage, PAddress);

    _Unmapped = MmDeleteVirtualMapping(Process, PAddress, core::ptr::null_mut(), &mut UnmappedPage);

    if !Process.is_null() {
        MmDeleteRmap(OldPage, Process, PAddress);
    }
    MmUnsharePageEntrySectionSegment(MemoryArea, Segment, core::ptr::addr_of_mut!(Offset), 0u8, 0u8, core::ptr::null_mut());
    MmUnlockSectionSegment(Segment as PVOID);

    if !NT_SUCCESS(MmCreateVirtualMapping(Process, PAddress, NewProtect, NewPage)) {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if !Process.is_null() {
        MmInsertRmap(NewPage, Process, PAddress);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmProtectSectionView(
    AddressSpace: PVOID,
    MemoryArea: PMEMORY_AREA,
    BaseAddress: PVOID,
    Length: SIZE_T,
    Protect: ULONG,
    OldProtect: *mut ULONG,
) -> NTSTATUS {
    let AddressSpace = opaque_ptr(AddressSpace);
    let BaseAddress = opaque_ptr(BaseAddress);
    let OldProtect = opaque_ptr(OldProtect);
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
        MmAlterViewAttributes as *const core::ffi::c_void,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmQuerySectionView(
    MemoryArea: PMEMORY_AREA,
    Address: PVOID,
    Info: *mut MEMORY_BASIC_INFORMATION,
    ResultLength: *mut SIZE_T,
) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    let Info = opaque_ptr(Info);
    let ResultLength = opaque_ptr(ResultLength);
    let mut RegionBaseAddress: PVOID = core::ptr::null_mut();
    let Region = MmFindRegion(
        MA_GetStartingAddress(MemoryArea) as PVOID,
        core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead),
        Address,
        core::ptr::addr_of_mut!(RegionBaseAddress),
    );
    if Region.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*MemoryArea).VadNode.u.VadFlags.VadType() as u32 == VadImageMap {
        let Segment = (*MemoryArea).SectionData.Segment;
        let ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        let SectionSegments = (*ImageSectionObject).Segments;
        (*Info).AllocationBase = ((MA_GetStartingAddress(MemoryArea) as ULONG_PTR)
            - (*SectionSegments).Image.VirtualAddress as ULONG_PTR) as PVOID;
        (*Info).Type = MEM_IMAGE;
    } else {
        (*Info).AllocationBase = MA_GetStartingAddress(MemoryArea) as PVOID;
        (*Info).Type = MEM_MAPPED;
    }
    (*Info).BaseAddress = RegionBaseAddress;
    (*Info).AllocationProtect = PAGE_READONLY;
    (*Info).RegionSize = (*Region).Length;
    (*Info).State = MEM_COMMIT;
    (*Info).Protect = (*Region).Protect;

    core::ptr::write_volatile(ResultLength, core::mem::size_of::<MEMORY_BASIC_INFORMATION>());
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateDataFileSection(
    SectionObject: *mut PSECTION,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    UMaximumSize: PLARGE_INTEGER,
    SectionPageProtection: ULONG,
    AllocationAttributes: ULONG,
    FileObject: PFILE_OBJECT,
    GotFileHandle: BOOLEAN,
) -> NTSTATUS {
    let SectionObject = opaque_ptr(SectionObject);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let UMaximumSize = opaque_ptr(UMaximumSize);
    let FileObject = opaque_ptr(FileObject);
    let mut Section: PSECTION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut MaximumSize = LARGE_INTEGER { QuadPart: 0 };
    let mut Segment: PMM_SECTION_SEGMENT;
    let mut OldIrql: KIRQL;

    Status = ObCreateObject(
        ExGetPreviousMode() as KPROCESSOR_MODE,
        MmSectionObjectType,
        ObjectAttributes,
        ExGetPreviousMode(),
        core::ptr::null_mut(),
        core::mem::size_of::<_SECTION>() as u32,
        0,
        0,
        core::ptr::addr_of_mut!(Section) as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    core::ptr::write_bytes(Section as *mut u8, 0, core::mem::size_of::<_SECTION>());
    (*Section).u.Flags.set_filler(1);
    (*Section).InitialPageProtection = SectionPageProtection;
    (*Section).u.Flags.set_File(1);

    if (AllocationAttributes & SEC_NO_CHANGE) != 0 {
        (*Section).u.Flags.set_NoChange(1);
    }

    if GotFileHandle == 0 {
        MaximumSize = *UMaximumSize;
    } else {
        let mut FileSize = LARGE_INTEGER { QuadPart: 0 };
        Status = FsRtlGetFileSize(FileObject, core::ptr::addr_of_mut!(FileSize));
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
            // Exceeds the C, which extends the file whatever the protection. NT grows a file
            // only for a section that could WRITE it; a read-only one larger than the file is
            // STATUS_SECTION_TOO_BIG (kmtest ZwCreateSection.c:380). Without this, the extend
            // below -- broken until its info class was fixed -- grew kmtest's file by a byte and
            // every later size check against it failed (ZwCreateSection.c:490).
            if (SectionPageProtection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) == 0 {
                ObDereferenceObject(Section as PVOID);
                return STATUS_SECTION_TOO_BIG;
            }
            // FileEndOfFileInformation (20). A literal 2 (FileFullDirectoryInformation) made
            // every extend fail, so a section larger than its file was NOT_EXTENDED: setup's
            // cabinet extraction lost all 964 files.
            Status = IoSetInformation(
                FileObject,
                ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileEndOfFileInformation as u32,
                core::mem::size_of::<LARGE_INTEGER>() as u32,
                core::ptr::addr_of_mut!(MaximumSize) as PVOID,
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
        OldIrql = MiAcquirePfnLock();
        Segment = (*(*FileObject).SectionObjectPointer).DataSectionObject as PMM_SECTION_SEGMENT;

        while !Segment.is_null() && ((*Segment).SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)) != 0 {
            MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            OldIrql = MiAcquirePfnLock();
            Segment = (*(*FileObject).SectionObjectPointer).DataSectionObject as PMM_SECTION_SEGMENT;
        }

        if Segment.is_null() {
            MiReleasePfnLock(OldIrql);

            Segment = ExAllocatePoolWithTag(
                NonPagedPool,
                core::mem::size_of::<MM_SECTION_SEGMENT>(),
                TAG_MM_SECTION_SEGMENT,
            ) as PMM_SECTION_SEGMENT;
            if Segment.is_null() {
                ObDereferenceObject(Section as PVOID);
                return STATUS_NO_MEMORY;
            }

            core::ptr::write_bytes(Segment as *mut u8, 0, core::mem::size_of::<MM_SECTION_SEGMENT>());
            (*Segment).SegFlags = MM_DATAFILE_SEGMENT | MM_SEGMENT_INCREATE;
            (*Segment).RefCount = 1;

            OldIrql = MiAcquirePfnLock();

            if !(*(*FileObject).SectionObjectPointer).DataSectionObject.is_null() {
                MiReleasePfnLock(OldIrql);
                ExFreePoolWithTag(Segment as PVOID, TAG_MM_SECTION_SEGMENT);
                continue 'grab_segment;
            }

            (*(*FileObject).SectionObjectPointer).DataSectionObject = Segment as *mut core::ffi::c_void;
            MiReleasePfnLock(OldIrql);

            (*Section).Segment = Segment as *mut _;
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
                (*Segment).Length.QuadPart = PAGE_ROUND_UP((*Segment).RawLength.QuadPart as usize) as i64;
            }
            (*Segment).Image.VirtualAddress = 0;
            MiInitializeSectionPageTable(Segment);

            OldIrql = MiAcquirePfnLock();
            (*Segment).SegFlags &= !MM_SEGMENT_INCREATE;
            MiReleasePfnLock(OldIrql);
            break;
        } else {
            (*Section).Segment = Segment as *mut _;
            InterlockedIncrement64(core::ptr::addr_of_mut!((*Segment).RefCount) as *mut i64);
            InterlockedIncrementUL(core::ptr::addr_of_mut!((*Segment).SectionCount));

            MiReleasePfnLock(OldIrql);

            MmLockSectionSegment(Segment as PVOID);

            if MaximumSize.QuadPart > (*Segment).RawLength.QuadPart
                && (AllocationAttributes & SEC_RESERVE) == 0
            {
                (*Segment).RawLength.QuadPart = MaximumSize.QuadPart;
                (*Segment).Length.QuadPart = PAGE_ROUND_UP((*Segment).RawLength.QuadPart as usize) as i64;
            }

            MmUnlockSectionSegment(Segment as PVOID);
            break;
        }
    }
    (*Section).SizeOfSection = MaximumSize;

    core::ptr::write_volatile(SectionObject, Section);
    STATUS_SUCCESS
}

unsafe extern "system" fn ExeFmtpAllocateSegments(NrSegments: ULONG) -> PMM_SECTION_SEGMENT {
    let SizeOfSegments = core::mem::size_of::<MM_SECTION_SEGMENT>() * NrSegments as usize;
    let Segments = ExAllocatePoolWithTag(
        NonPagedPool,
        SizeOfSegments,
        TAG_MM_SECTION_SEGMENT,
    ) as PMM_SECTION_SEGMENT;

    if !Segments.is_null() {
        core::ptr::write_bytes(Segments as *mut u8, 0, SizeOfSegments);
    }

    Segments
}

unsafe extern "system" fn ExeFmtpReadFile(
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
    let Buffer: PVOID;
    let FileObject = File as PFILE_OBJECT;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    if Length == 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if FileOffset.__bindgen_anon_1.HighPart < 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    AdjustOffset = PAGE_ROUND_DOWN_USIZE(FileOffset.__bindgen_anon_1.LowPart as usize) as ULONG;
    OffsetAdjustment = FileOffset.__bindgen_anon_1.LowPart - AdjustOffset;
    FileOffset.__bindgen_anon_1.LowPart = AdjustOffset;

    BufferSize = Length + OffsetAdjustment;
    BufferSize = PAGE_ROUND_UP(BufferSize as usize) as ULONG;

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
    FileObject: PFILE_OBJECT,
    ImageSectionObject: PMM_IMAGE_SECTION_OBJECT,
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
        PAGE_SIZE as ULONG * 2,
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
    while i < 1 {
        Flags = 0;

        Status = PeFmtCreateSection(
            FileHeader,
            FileHeaderSize as usize,
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

    // Sort, bounds-check, page-align, trim, and initialize segment locks/refcounts/flags/pagetable/FileObject.
    // This block was missing from the original Rust translation — without it, image section segments
    // have uninitialized FAST_MUTEX locks, causing BUGCHECK in MmLockSectionSegment.
    let FixupStatus = MmFixupImageSectionSegments(ImageSectionObject, Flags, FileObject);
    if !NT_SUCCESS(FixupStatus) {
        return FixupStatus;
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn MmCreateImageSection(
    SectionObject: *mut PSECTION,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    UMaximumSize: PLARGE_INTEGER,
    SectionPageProtection: ULONG,
    AllocationAttributes: ULONG,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    let SectionObject = opaque_ptr(SectionObject);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let UMaximumSize = opaque_ptr(UMaximumSize);
    let FileObject = opaque_ptr(FileObject);
    let mut Section: PSECTION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ImageSectionObject: PMM_IMAGE_SECTION_OBJECT;
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
        core::mem::size_of::<_SECTION>() as u32,
        0,
        0,
        &mut Section as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    core::ptr::write_bytes(Section as *mut u8, 0, core::mem::size_of::<_SECTION>());
    (*Section).u.Flags.set_filler(1);
    (*Section).InitialPageProtection = SectionPageProtection;
    (*Section).SizeOfSection = *UMaximumSize;
    (*Section).u.Flags.set_File(1);
    (*Section).u.Flags.set_Image(1);
    if (AllocationAttributes & SEC_NO_CHANGE) != 0 {
        (*Section).u.Flags.set_NoChange(1);
    }

    'grab_image_section_object: loop {
        OldIrql = MiAcquirePfnLock();

        ImageSectionObject = (*(*FileObject).SectionObjectPointer).ImageSectionObject as PMM_IMAGE_SECTION_OBJECT;
        while !ImageSectionObject.is_null() && ((*ImageSectionObject).SegFlags & (MM_SEGMENT_INDELETE | MM_SEGMENT_INCREATE)) != 0 {
            MiReleasePfnLock(OldIrql);
            KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
            OldIrql = MiAcquirePfnLock();
            ImageSectionObject = (*(*FileObject).SectionObjectPointer).ImageSectionObject as PMM_IMAGE_SECTION_OBJECT;
        }

        if ImageSectionObject.is_null() {
            MiReleasePfnLock(OldIrql);

            ImageSectionObject = ExAllocatePoolZero(
                NonPagedPool,
                core::mem::size_of::<MM_IMAGE_SECTION_OBJECT>() as _,
                TAG_MM_SECTION_SEGMENT,
            ) as PMM_IMAGE_SECTION_OBJECT;
            if ImageSectionObject.is_null() {
                ObDereferenceObject(Section as PVOID);
                return STATUS_NO_MEMORY;
            }

            (*ImageSectionObject).SegFlags = MM_SEGMENT_INCREATE;
            (*ImageSectionObject).RefCount = 1;
            (*ImageSectionObject).SectionCount = 1;

            OldIrql = MiAcquirePfnLock();
            if !(*(*FileObject).SectionObjectPointer).ImageSectionObject.is_null() {
                MiReleasePfnLock(OldIrql);
                ExFreePoolWithTag(ImageSectionObject as PVOID, TAG_MM_SECTION_SEGMENT);
                continue 'grab_image_section_object;
            }

            (*(*FileObject).SectionObjectPointer).ImageSectionObject = ImageSectionObject as *mut _;
            MiReleasePfnLock(OldIrql);

            CcFlushCache((*FileObject).SectionObjectPointer, core::ptr::null_mut(), 0, core::ptr::null_mut());

            let StatusExeFmt = ExeFmtpCreateImageSection(FileObject, ImageSectionObject);

            if StatusExeFmt < 0 {
                OldIrql = MiAcquirePfnLock();
                (*(*FileObject).SectionObjectPointer).ImageSectionObject = core::ptr::null_mut();
                MiReleasePfnLock(OldIrql);

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

            (*Section).Segment = ImageSectionObject as *mut _;

            Status = MmspWaitForFileLock(FileObject);
            if Status < 0 {
                OldIrql = MiAcquirePfnLock();
                (*(*FileObject).SectionObjectPointer).ImageSectionObject = core::ptr::null_mut();
                MiReleasePfnLock(OldIrql);

                ExFreePool((*ImageSectionObject).Segments as PVOID);
                ExFreePool(ImageSectionObject as PVOID);
                ObDereferenceObject(Section as PVOID);
                return Status;
            }

            OldIrql = MiAcquirePfnLock();
            (*ImageSectionObject).SegFlags &= !MM_SEGMENT_INCREATE;

            ObReferenceObject(FileObject as PVOID);

            MiReleasePfnLock(OldIrql);

            Status = StatusExeFmt;
        } else {
            (*ImageSectionObject).SegFlags &= !MM_IMAGE_SECTION_FLUSH_DELETE;

            InterlockedIncrement64(core::ptr::addr_of_mut!((*ImageSectionObject).RefCount) as *mut i64);
            (*ImageSectionObject).SectionCount += 1;

            MiReleasePfnLock(OldIrql);

            (*Section).Segment = ImageSectionObject as *mut _;

            Status = STATUS_SUCCESS;
        }

        if (*Section).SizeOfSection.QuadPart == 0 {
            (*Section).SizeOfSection.QuadPart = (*ImageSectionObject).ImageInformation.ImageFileSize as i64;
        }

        core::ptr::write_volatile(SectionObject, Section);

        return Status;
    }
}

unsafe fn MmMapViewOfSegment(
    AddressSpace: PVOID,
    AsImage: BOOLEAN,
    Segment: PMM_SECTION_SEGMENT,
    BaseAddress: *mut PVOID,
    ViewSize: SIZE_T,
    Protect: ULONG,
    ViewOffset: i64,
    AllocationType: ULONG,
) -> NTSTATUS {
    let mut MArea: PMEMORY_AREA = core::ptr::null_mut();
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
        MM_ALLOCATION_GRANULARITY as ULONG
    } else {
        PAGE_SIZE as ULONG
    };

    Status = MmCreateMemoryArea(
        AddressSpace,
        MEMORY_AREA_SECTION_VIEW,
        BaseAddress,
        ViewSize,
        Protect,
        core::ptr::addr_of_mut!(MArea),
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
        (*MArea).VadNode.u.VadFlags.set_VadType(VadImageMap as _);
    }

    MmInitializeRegion(
        core::ptr::addr_of_mut!((*MArea).SectionData.RegionListHead),
        ViewSize,
        0,
        Protect,
    );

    STATUS_SUCCESS
}

unsafe extern "C" fn MmFreeSectionPage(
    Context: PVOID,
    MemoryArea: PMEMORY_AREA,
    Address: PVOID,
    Page: PFN_NUMBER,
    SwapEntry: ULONG_PTR,
    Dirty: BOOLEAN,
) {
    let mut Entry: ULONG_PTR;
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let mut SavedSwapEntry: ULONG_PTR;
    let Segment: PMM_SECTION_SEGMENT;
    let AddressSpace = Context as PVOID;
    let Process = MmGetAddressSpaceOwner(AddressSpace);

    let Address = PAGE_ROUND_DOWN_USIZE(Address as usize) as PVOID;

    Offset.QuadPart = ((Address as ULONG_PTR - MA_GetStartingAddress(MemoryArea)) as i64)
        + (*MemoryArea).SectionData.ViewOffset;

    Segment = (*MemoryArea).SectionData.Segment;

    Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
    while Entry != 0 && MM_IS_WAIT_PTE(Entry) {
        MmUnlockSectionSegment(Segment as PVOID);
        MmUnlockAddressSpace(AddressSpace);
        KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of!(TinyTime) as *mut _);
        MmLockAddressSpace(AddressSpace);
        MmLockSectionSegment(Segment as PVOID);
        Entry = MmGetPageEntrySectionSegment(Segment as PVOID, core::ptr::addr_of!(Offset.QuadPart));
    }

    if SwapEntry != 0 {
        MmFreeSwapPage(SwapEntry);
    } else if Page != 0 {
        if IS_SWAP_FROM_SSE(Entry) || Page != PFN_FROM_SSE(Entry) {
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
            MmUnsharePageEntrySectionSegment(MemoryArea, Segment, core::ptr::addr_of_mut!(Offset), if !Process.is_null() { Dirty } else { 0u8 }, 0u8, core::ptr::null_mut());
        }
    }
}

unsafe fn MmUnmapViewOfSegment(
    AddressSpace: PVOID,
    BaseAddress: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let MemoryArea: PMEMORY_AREA;
    let Segment: PMM_SECTION_SEGMENT;
    let mut CurrentEntry: PLIST_ENTRY;
    let RegionListHead: PLIST_ENTRY;

    MemoryArea = MmLocateMemoryAreaByAddress(AddressSpace, BaseAddress);
    if MemoryArea.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Segment = (*MemoryArea).SectionData.Segment;

    (*MemoryArea).DeleteInProgress = 1u8;

    MmLockSectionSegment(Segment as PVOID);

    RegionListHead = core::ptr::addr_of_mut!((*MemoryArea).SectionData.RegionListHead);
    while !IsListEmpty_inline(RegionListHead) {
        CurrentEntry = RemoveHeadList_inline(RegionListHead);
        let CurrentRegion = (CurrentEntry as *mut u8)
            .sub(core::mem::offset_of!(_MM_REGION, RegionListEntry)) as PMM_REGION;
        ExFreePoolWithTag(CurrentRegion as PVOID, TAG_MM_REGION);
    }

    if ((*(*Segment).Flags) & MM_PHYSICALMEMORY_SEGMENT) != 0 {
        Status = MmFreeMemoryArea(AddressSpace, MemoryArea, None, core::ptr::null_mut());
    } else {
        Status = MmFreeMemoryArea(AddressSpace, MemoryArea, Some(MmFreeSectionPage), AddressSpace);
    }
    MmUnlockSectionSegment(Segment as PVOID);
    MmDereferenceSegment(Segment);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiRosUnmapViewOfSection(
    Process: PEPROCESS,
    MemoryArea: PMEMORY_AREA,
    BaseAddress: PVOID,
    SkipDebuggerNotify: BOOLEAN,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let BaseAddress = opaque_ptr(BaseAddress);
    let mut Status: NTSTATUS;
    let AddressSpace: PVOID;
    let mut ImageBaseAddress: PVOID = core::ptr::null_mut();

    AddressSpace = core::ptr::addr_of_mut!((*Process).Vm) as PVOID;

    if MemoryArea.is_null() || (*MemoryArea).DeleteInProgress != 0 {
        return STATUS_NOT_MAPPED_VIEW;
    }

    if (*MemoryArea).VadNode.u.VadFlags.VadType() as u32 == VadImageMap {
        let mut i: ULONG;
        let NrSegments: ULONG;
        let ImageSectionObject: PMM_IMAGE_SECTION_OBJECT;
        let SectionSegments: PMM_SECTION_SEGMENT;
        let Segment: PMM_SECTION_SEGMENT;

        Segment = (*MemoryArea).SectionData.Segment;
        ImageSectionObject = ImageSectionObjectFromSegment(Segment);
        SectionSegments = (*ImageSectionObject).Segments;
        NrSegments = (*ImageSectionObject).NrSegments;

        (*MemoryArea).DeleteInProgress = 1u8;

        i = 0;
        while i < NrSegments {
            if Segment == core::ptr::addr_of_mut!((*SectionSegments.add(i as usize))) {
                ImageBaseAddress = ((BaseAddress as ULONG_PTR) - (*SectionSegments.add(i as usize)).Image.VirtualAddress as ULONG_PTR) as PVOID;
                break;
            }
            i += 1;
        }

        i = 0;
        while i < NrSegments {
            let SBaseAddress = ((ImageBaseAddress as ULONG_PTR) + (*SectionSegments.add(i as usize)).Image.VirtualAddress as ULONG_PTR) as PVOID;

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

        if ((*(*Segment).Flags) & MM_PHYSICALMEMORY_SEGMENT) == 0 {
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
    let SectionHandle = opaque_ptr(SectionHandle);
    let SectionInformation = opaque_ptr(SectionInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Section: PSECTION = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE;
    let mut Status: NTSTATUS;

    PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0i8 {
        // C: inside _SEH2_TRY; the handler returns the exception code. (The old
        // raising extern also truncated the SIZE_T length to ULONG.)
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
            SectionInformation,
            SectionInformationLength as usize,
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return status;
        }
        if !ResultLength.is_null() {
            if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                ResultLength as PVOID,
                core::mem::size_of::<SIZE_T>(),
                core::mem::align_of::<SIZE_T>(),
                PreviousMode,
            ) {
                return status;
            }
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
        core::ptr::addr_of_mut!(Section) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    Status = handle_query_section_match(Section, SectionInformationClass, SectionInformation, ResultLength);

    ObDereferenceObject(Section as PVOID);
    Status
}

/// The C writes the result inside `_SEH2_TRY` and its handler returns the code. The
/// pointers were range-probed on entry, so write through the fault-tolerant copy.
unsafe fn write_section_result<T>(dst: PVOID, value: &T) -> Result<(), NTSTATUS> {
    ntkrnl_sys::probe::safe_copy_raw(
        dst,
        value as *const T as *const core::ffi::c_void,
        core::mem::size_of::<T>(),
    )
}

// Helper for NtQuerySection match branches
pub unsafe fn handle_query_section_match(
    Section: PSECTION,
    SectionInformationClass: SECTION_INFORMATION_CLASS,
    SectionInformation: PVOID,
    ResultLength: *mut usize,
) -> NTSTATUS {
    if SectionInformationClass == SectionBasicInformation {
        let mut Sbi = _SECTION_BASIC_INFORMATION {
            Size: LARGE_INTEGER { QuadPart: 0 },
            BaseAddress: core::ptr::null_mut(),
            Attributes: 0,
        };

        Sbi.Size = (*Section).SizeOfSection;
        Sbi.BaseAddress = core::ptr::null_mut();

        if (*Section).u.Flags.File() != 0 {
            Sbi.Attributes |= SEC_FILE;
        }
        if (*Section).u.Flags.Image() != 0 {
            Sbi.Attributes |= SEC_IMAGE;
        }
        if (*Section).u.Flags.Commit() != 0 {
            Sbi.Attributes |= SEC_COMMIT;
        }
        if (*Section).u.Flags.Reserve() != 0 {
            Sbi.Attributes |= SEC_RESERVE;
        }
        if (*Section).u.Flags.Based() != 0 {
            Sbi.Attributes |= SEC_BASED;
        }

        if let Err(status) = write_section_result(SectionInformation, &Sbi) {
            return status;
        }
        if !ResultLength.is_null() {
            let length = core::mem::size_of::<_SECTION_BASIC_INFORMATION>();
            if let Err(status) = write_section_result(ResultLength as PVOID, &length) {
                return status;
            }
        }
        STATUS_SUCCESS
    } else if SectionInformationClass == SectionImageInformation {
        if (*Section).u.Flags.Image() == 0 {
            STATUS_SECTION_NOT_IMAGE
        } else if MiIsRosSectionObject(Section) != 0 {
            let ImageSectionObject = (*Section).Segment as PMM_IMAGE_SECTION_OBJECT;
            if let Err(status) =
                write_section_result(SectionInformation, &(*ImageSectionObject).ImageInformation)
            {
                return status;
            }
            if !ResultLength.is_null() {
                let length = core::mem::size_of::<SECTION_IMAGE_INFORMATION>();
                if let Err(status) = write_section_result(ResultLength as PVOID, &length) {
                    return status;
                }
            }
            STATUS_SUCCESS
        } else {
            STATUS_NOT_SUPPORTED
        }
    } else {
        STATUS_NOT_SUPPORTED
    }
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
    let SectionObject = opaque_ptr(SectionObject);
    let Process = opaque_ptr(Process);
    let BaseAddress = opaque_ptr(BaseAddress);
    let SectionOffset = opaque_ptr(SectionOffset);
    let ViewSize = opaque_ptr(ViewSize);
    if MiIsRosSectionObject(SectionObject as PSECTION) == 0 {
        return MmMapViewOfArm3Section(
            SectionObject, Process as ntkrnl_sys::ddk::PEPROCESS, BaseAddress, ZeroBits, CommitSize,
            SectionOffset as PLARGE_INTEGER, ViewSize as PSIZE_T, InheritDisposition, AllocationType, Protect,
        );
    }

    if Protect == 0 || (Protect & !PAGE_FLAGS_VALID_FOR_SECTION) != 0 {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    let mut IsAttached = 0u8;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

    if PsGetCurrentProcess() as PVOID != Process as PVOID {
        KeStackAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb), core::ptr::addr_of_mut!(ApcState));
        IsAttached = 1u8;
    }

    let Section: PSECTION = SectionObject as PSECTION;
    let AddressSpace: PVOID = core::ptr::addr_of_mut!((*Process).Vm) as PVOID;

    MmLockAddressSpace(AddressSpace);

    let mut Status = STATUS_SUCCESS;
    let mut NotAtBase = 0u8;

    if (*Section).u.Flags.Image() != 0 {
        let ImageSectionObject = (*Section).Segment as PMM_IMAGE_SECTION_OBJECT;
        let SectionSegments = (*ImageSectionObject).Segments;
        let NrSegments = (*ImageSectionObject).NrSegments;

        let mut ImageBase = (*BaseAddress) as ULONG_PTR;
        if ImageBase == 0 {
            ImageBase = (*ImageSectionObject).BasedAddress as ULONG_PTR;
        }

        let mut ImageSize: SIZE_T = 0;
        for i in 0..NrSegments {
            let MaxExtent = (*SectionSegments.add(i as usize)).Image.VirtualAddress as ULONG_PTR
                + (*SectionSegments.add(i as usize)).Length.QuadPart as ULONG_PTR;
            if MaxExtent > ImageSize {
                ImageSize = MaxExtent;
            }
        }

        (*ImageSectionObject).ImageInformation.ImageFileSize = ImageSize as ULONG;

        if ((ImageBase + ImageSize) > MM_HIGHEST_VAD_ADDRESS as ULONG_PTR)
            || ((ImageBase + ImageSize) < ImageSize)
        {
            let aligned = (MM_HIGHEST_VAD_ADDRESS as ULONG_PTR - ImageSize) & !(MM_VIRTMEM_GRANULARITY as ULONG_PTR - 1);
            ImageBase = aligned;
            NotAtBase = 1u8;
        } else if ImageBase != ImageBase & !(MM_VIRTMEM_GRANULARITY as ULONG_PTR - 1) {
            ImageBase = ImageBase & !(MM_VIRTMEM_GRANULARITY as ULONG_PTR - 1);
            NotAtBase = 1u8;
        }

        // Check there is enough space to map the section at that point.
        if MmIsAddressRangeFree(AddressSpace, ImageBase as PVOID, PAGE_ROUND_UP(ImageSize)) == 0 {
            // Fail if the user requested a fixed base address.
            if !(*BaseAddress).is_null() {
                Status = STATUS_CONFLICTING_ADDRESSES;
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                return Status;
            }
            // Otherwise find a gap to map the image.
            let Gap = MmFindGap(AddressSpace, PAGE_ROUND_UP(ImageSize), MM_VIRTMEM_GRANULARITY, 0u8);
            if Gap.is_null() {
                Status = STATUS_CONFLICTING_ADDRESSES;
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                return Status;
            }
            ImageBase = Gap as ULONG_PTR;
            NotAtBase = 1u8;
        }

        for i in 0..NrSegments {
            let mut SBaseAddress =
                (ImageBase + (*SectionSegments.add(i as usize)).Image.VirtualAddress as ULONG_PTR) as PVOID;
            MmLockSectionSegment(SectionSegments.add(i as usize) as PVOID);
            Status = MmMapViewOfSegment(
                AddressSpace,
                1u8,
                SectionSegments.add(i as usize),
                &mut SBaseAddress,
                (*SectionSegments.add(i as usize)).Length.QuadPart as SIZE_T,
                (*SectionSegments.add(i as usize)).Protection,
                0,
                0,
            );
            MmUnlockSectionSegment(SectionSegments.add(i as usize) as PVOID);
            if Status < 0 {
                // Roll back already-mapped segments
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let RollbackAddr =
                        (ImageBase + (*SectionSegments.add(j as usize)).Image.VirtualAddress as ULONG_PTR) as PVOID;
                    MmLockSectionSegment(SectionSegments.add(j as usize) as PVOID);
                    MmUnmapViewOfSegment(AddressSpace, RollbackAddr);
                    MmUnlockSectionSegment(SectionSegments.add(j as usize) as PVOID);
                }
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                return Status;
            }
        }

        core::ptr::write_volatile(BaseAddress, ImageBase as PVOID);
        core::ptr::write_volatile(ViewSize, ImageSize);

        InterlockedIncrementUL(core::ptr::addr_of_mut!((*ImageSectionObject).MapCount) as *mut ULONG);
    } else {
        let Segment = (*Section).Segment as PMM_SECTION_SEGMENT;
        let mut ViewOffset: i64 = if SectionOffset.is_null() {
            0
        } else {
            (*SectionOffset).QuadPart
        };

        // Check if the offset and size would cause an overflow
        let mut current_view_size = core::ptr::read_volatile(ViewSize);
        if (ViewOffset as u64).wrapping_add(current_view_size as u64) < ViewOffset as u64 {
            MmUnlockAddressSpace(AddressSpace);
            if IsAttached != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
            }
            return STATUS_INVALID_VIEW_SIZE;
        }

        // Check if the offset and size are bigger than the section itself
        if (ViewOffset as u64).wrapping_add(current_view_size as u64) > (*Section).SizeOfSection.QuadPart as u64 {
            if (*Section).u.Flags.PhysicalMemory() == 0 || ExGetPreviousMode() != 0i8 {
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                return STATUS_INVALID_VIEW_SIZE;
            }
        }

        // Handle ViewSize == 0: expand to section size minus offset
        if current_view_size == 0 {
            let calculated_size = (*Section).SizeOfSection.QuadPart as u64 - ViewOffset as u64;
            if calculated_size > usize::MAX as u64 {
                MmUnlockAddressSpace(AddressSpace);
                if IsAttached != 0 {
                    KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
                }
                return STATUS_INVALID_VIEW_SIZE;
            }
            current_view_size = calculated_size as SIZE_T;
            core::ptr::write_volatile(ViewSize, current_view_size);
        }

        // Page-round the view size
        current_view_size = PAGE_ROUND_UP(current_view_size);
        core::ptr::write_volatile(ViewSize, current_view_size);

        MmLockSectionSegment(Segment as PVOID);
        Status = MmMapViewOfSegment(
            AddressSpace,
            0u8,
            Segment,
            BaseAddress,
            current_view_size,
            Protect,
            ViewOffset,
            AllocationType,
        );
        MmUnlockSectionSegment(Segment as PVOID);
    }

    if NotAtBase != 0 {
        Status = STATUS_IMAGE_NOT_AT_BASE;
    }

    MmUnlockAddressSpace(AddressSpace);

    if IsAttached != 0 {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState));
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MiRosUnmapViewInSystemSpace(MappedBase: PVOID) -> NTSTATUS {
    let MappedBase = opaque_ptr(MappedBase);
    MmUnmapViewOfSegment(MmGetKernelAddressSpace(), MappedBase)
}
