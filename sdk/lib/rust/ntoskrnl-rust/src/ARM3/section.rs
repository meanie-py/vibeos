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
use ntkrnl_sys::macros::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use super::helpers::*;

use ntkrnl_sys::ddk::{
    _CONTROL_AREA,
    _EPROCESS,
    _ETHREAD,
    _FILE_OBJECT,
    _HANDLE_TABLE_ENTRY,
    _KAPC_STATE,
    _KGUARDED_MUTEX,
    _LARGE_INTEGER,
    _MMADDRESS_NODE,
    _MMPFN as DDK_MMPFN,
    _MMPTE as DDK_MMPTE,
    _MMSUPPORT,
    _MMVAD_LONG,
    _MMVIEW,
    _MM_AVL_TABLE,
    _MM_SESSION_SPACE,
    _MSUBSECTION,
    _OBJECT_ATTRIBUTES,
    _SECTION,
    _SECTION_IMAGE_INFORMATION,
    _SECTION_OBJECT_POINTERS,
    _SEGMENT,
    _SUBSECTION,
    _UNICODE_STRING,
    _OBJECT_NAME_INFORMATION,
    _MI_VAD_TYPE_VadImageMap,
    ACCESS_MASK,
    DbgkUnMapViewOfSection,
    ExFreePool,
    ExGetPreviousMode,
    IoSetTopLevelIrp,
    KPROCESSOR_MODE,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    MiCheckSecuredVad,
    MiDeleteVirtualAddresses,
    MiInsertBasedSection,
    MiInsertVadEx,
    MiIsRosSectionObject,
    MiLocateAddress,
    MiLocateVad,
    MiLockProcessWorkingSetUnsafe,
    MiRemoveNode,
    MiRosUnmapViewInSystemSpace,
    MiRosUnmapViewOfSection,
    MiUnlockProcessWorkingSetUnsafe,
    MmCreateSection,
    MmExtendSection,
    MmGetCurrentAddressSpace,
    MiInitializePfnForOtherProcess,
    MiRemoveZeroPage,
    MiDecrementShareCount,
    MiMakeSystemAddressValidPfn,
    MiUnlinkPageFromList,
    MiInsertPageInFreeList,
    ObOpenObjectByName,
    ObInsertObject,
    ObReferenceObjectByHandle,
    PsGetCurrentThread,
    MmLocateMemoryAreaByAddress,
    MmGetKernelAddressSpace,
    PMEMORY_AREA,
    MEMORY_AREA,
    OB_OPEN_REASON,
    PEPROCESS,
    PETHREAD,
    PFILE_OBJECT,
    PHANDLE_TABLE,
    PHANDLE_TABLE_ENTRY,
    POBJECT_ATTRIBUTES,
    PSUBSECTION,
    PSECTION,
    PSECTION_IMAGE_INFORMATION,
    PSECTION_OBJECT_POINTERS,
    PSEGMENT,
    SECTION_INHERIT,
    HANDLE,
    MMSESSION,
    PMM_IMAGE_SECTION_OBJECT,
    PMM_SECTION_SEGMENT,
    POBJECT_NAME_INFORMATION,
    PHYSICAL_ADDRESS,
    OBJECT_TYPE_INITIALIZER,
    RTL_AVL_TABLE,
    FSRTL_COMMON_FCB_HEADER,
    KGUARDED_MUTEX,
    RTL_BITMAP,
    DbgkMapViewOfSection,
    MiFindEmptyAddressRangeDownBasedTree,
    ObQueryNameString,
    RtlCreateUnicodeString,
    RtlCopyUnicodeString,
    RtlFreeUnicodeString,
    RtlInitEmptyUnicodeString,
    MmFreeSwapPage,
    MmDereferenceSegment,
    ObCreateObject,
    ObCreateObjectType,
    RtlEnumerateGenericTable,
};

// Type aliases for pointer types not in ddk
type PMMVAD = *mut _MMVAD_LONG;
type PMMSUPPORT = *mut _MMSUPPORT;
type PMMADDRESS_NODE = *mut _MMADDRESS_NODE;
type POBJECT_TYPE = PVOID;
type FAST_MUTEX = _KGUARDED_MUTEX;

// VadImageMap enum value
const VadImageMap: u32 = _MI_VAD_TYPE_VadImageMap;

// ============================================================================
// Extern globals not available in ntkrnl-sys
// ============================================================================

extern "system" {
    // NTAPI/stdcall per ntoskrnl.spec — matches the extern "system" shim def
    // in shim_exports.rs and the canonical mem-rust export.
    fn RtlFillMemoryUlong(Destination: PVOID, Length: SIZE_T, Pattern: ULONG);
}
extern "C" {
    pub static mut MmSectionObjectType: PVOID;
    pub static mut PsProcessType: PVOID;
    pub static mut IoFileObjectType: PVOID;
}

extern "system" {
    fn MmMapViewOfSection(
        section: PSECTION,
        process: PEPROCESS,
        base_address: *mut PVOID,
        zero_bits: ULONG_PTR,
        commit_size: SIZE_T,
        section_offset: *mut _LARGE_INTEGER,
        view_size: PSIZE_T,
        inherit_disposition: SECTION_INHERIT,
        allocation_type: ULONG,
        protect: ULONG,
    ) -> NTSTATUS;
}

// ============================================================================
// Constants not in ntkrnl-sys
// ============================================================================

const MEM_DOS_LIM: ULONG = 0x40000000;
const MEM_LARGE_PAGES: ULONG = 0x20000000;
const STATUS_MAPPED_ALIGNMENT: NTSTATUS = 0xC0000220_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_8: NTSTATUS = 0xC00000F6_u32 as NTSTATUS;
// mm.h:240-243 — these are BIT FLAGS, not a sequence. They were numbered 1,2
// as if they were ordinals; the real values are PHYSICALMEMORY 0x1,
// DATAFILE 0x2, INDELETE 0x4, INCREATE 0x8.
const MM_SEGMENT_INCREATE: ULONG_PTR = 0x8;
const MM_PHYSICALMEMORY_SEGMENT: ULONG_PTR = 0x1;
const TAG_MM_SECTION_SEGMENT: ULONG = 0x53534D4D;
const TAG_MM_REGION: ULONG = 0x4E47524D;
const PAGE_IS_WRITECOPY: ULONG = PAGE_WRITECOPY | PAGE_EXECUTE_WRITECOPY;
const PAGE_IS_WRITABLE: ULONG = PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
const PAGE_IS_EXECUTABLE: ULONG = PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
const PAGE_FLAGS_VALID_FOR_SECTION: ULONG =
    PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY |
    PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY |
    PAGE_NOACCESS;
const MM_ALLOCATION_GRANULARITY: usize = 64 * 1024;
const MAX_SHARE_COUNT: ULONG = 0x1FF; // ntoskrnl/include/internal/mm.h
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
const SECTION_MAP_EXECUTE_EXPLICIT: ULONG = 0x0020;
const STATUS_FILE_LOCKED_WITH_WRITERS: NTSTATUS = 0x012B;
const STATUS_INVALID_IMAGE_NOT_MZ: NTSTATUS = 0xC000012F_u32 as NTSTATUS;
const STATUS_ROS_EXEFMT_UNKNOWN_FORMAT: NTSTATUS = 0xA0100001_u32 as NTSTATUS;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018_u32 as NTSTATUS;
const STATUS_IMAGE_NOT_AT_BASE: NTSTATUS = 0x40000003_u32 as NTSTATUS;
const STATUS_GUARD_PAGE_VIOLATION: NTSTATUS = 0x80000001_u32 as NTSTATUS;
const STATUS_IN_PAGE_ERROR: NTSTATUS = 0xC0000006_u32 as NTSTATUS;
const STATUS_SECTION_PROTECTION: NTSTATUS = 0xC000004E_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_5: NTSTATUS = 0xC00000F3_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_10: NTSTATUS = 0xC00000F8_u32 as NTSTATUS;
const FILE_OFFSET: ULONG = 0;
const MAP_PROCESS: ULONG = 1;
const MAP_SYSTEM: ULONG = 2;

// ============================================================================
// Inline helper functions not in ntkrnl-sys or helpers.rs
// ============================================================================

#[inline]
unsafe fn RtlLongLongSub(a: i64, b: i64, result: *mut i64) -> NTSTATUS {
    // ntintsafe: overflow FAILS. A wrapping sub that always succeeded made the caller's
    // `!NT_SUCCESS(RtlLongLongSub(..))` check dead code.
    match a.checked_sub(b) {
        Some(v) => { *result = v; STATUS_SUCCESS }
        None => STATUS_INTEGER_OVERFLOW
    }
}

#[inline]
unsafe fn RtlSIZETAdd(a: usize, b: usize, result: *mut usize) -> NTSTATUS {
    match a.checked_add(b) {
        Some(v) => { *result = v; STATUS_SUCCESS }
        None => STATUS_INTEGER_OVERFLOW
    }
}

const STATUS_INTEGER_OVERFLOW: NTSTATUS = 0xC0000095_u32 as NTSTATUS;

#[inline]
unsafe fn MI_IS_PAGE_DIRTY(pte: &MMPTE) -> bool {
    super::helpers::MI_PTE_IS_DIRTY(pte.Long as u64)
}

#[inline]
unsafe fn MI_UPDATE_VALID_PTE(pte: PMMPTE, value: MMPTE) {
    (*pte).Long = value.Long;
}

#[inline]
fn FlagOn(flags: ULONG, test: ULONG) -> bool {
    (flags & test) != 0
}

#[inline]
fn SetFlag(flags: &mut ULONG, bit: ULONG) {
    *flags |= bit;
}

#[inline]
fn ClearFlag(flags: &mut ULONG, bit: ULONG) {
    *flags &= !bit;
}

#[inline]
fn BooleanFlagOn(flags: ULONG, test: ULONG) -> BOOLEAN {
    if (flags & test) != 0 { TRUE } else { FALSE }
}

#[inline]
fn MM_ROUND_DOWN(x: ULONG_PTR, align: ULONG_PTR) -> ULONG_PTR {
    x & !(align - 1)
}

// (No SSE helpers here: ARM3/section.c uses prototype PTEs, not section segment entries. A
// private set with an invented encoding -- swap bit inverted, share count at bit 2 -- sat here
// unused; the real ones are mm.h's, in mm/section.rs.)

#[inline]
unsafe fn InterlockedIncrement64(addend: *mut i64) -> i64 {
    let atomic = &*(addend as *const core::sync::atomic::AtomicI64);
    atomic.fetch_add(1, core::sync::atomic::Ordering::SeqCst).wrapping_add(1)
}

#[inline]
unsafe fn InterlockedDecrement64(addend: *mut i64) -> i64 {
    let atomic = &*(addend as *const core::sync::atomic::AtomicI64);
    atomic.fetch_sub(1, core::sync::atomic::Ordering::SeqCst).wrapping_sub(1)
}

#[inline]
fn MI_IS_MAPPED_PTE(pte_long: ULONG_PTR) -> bool {
    pte_long != 0
}

#[inline]
fn MI_MAX_ZERO_BITS() -> ULONG_PTR {
    #[cfg(target_pointer_width = "64")]
    { 21 }
    #[cfg(target_pointer_width = "32")]
    { 20 }
}

#[inline]
fn MI_GET_PROTOTYPE_PTE_FOR_VPN(vad: *const _MMVAD_LONG, vpn: ULONG_PTR) -> PMMPTE {
    unsafe {
        (*vad).FirstPrototypePte.add((vpn - (*vad).StartingVpn as ULONG_PTR) as usize) as PMMPTE
    }
}

#[inline]
fn MI_IS_ROSMM_VAD(_vad: PVOID) -> bool { false }

type PEXEFMT_CB_READ_FILE = PVOID;
type PEXEFMT_CB_ALLOCATE_SEGMENTS = PVOID;

#[repr(C)]
struct _MEMORY_SECTION_NAME {
    SectionFileName: _UNICODE_STRING,
}

extern "system" {
    fn MmSetPageEntrySectionSegment(
        Segment: PVOID, Offset: *const u64, Entry: ULONG_PTR,
    ) -> NTSTATUS;
    fn MmFreePageTablesSectionSegment(
        Segment: PVOID, FreeAllocFn: PVOID,
    );
    fn MiInitializeSectionPageTable(Segment: PVOID);
    fn MmFlushVirtualMemory(
        AddressSpace: PVOID, BaseAddress: *mut PVOID, RegionSize: PSIZE_T,
        IoStatusBlock: PVOID,
    ) -> NTSTATUS;
    fn MmMapViewInSystemSpace(
        Section: PVOID, MappedBase: *mut PVOID, ViewSize: PSIZE_T,
    ) -> NTSTATUS;
    fn FsRtlAcquireFileExclusive(FileObject: PFILE_OBJECT);
    fn FsRtlReleaseFile(FileObject: PFILE_OBJECT);
    fn FsRtlGetFileSize(FileObject: PFILE_OBJECT, FileSize: *mut _LARGE_INTEGER) -> NTSTATUS;
    fn MiProtectVirtualMemory(
        Process: PEPROCESS, BaseAddress: *mut PVOID, RegionSize: PSIZE_T,
        NewProtect: ULONG, OldProtect: *mut ULONG,
    ) -> NTSTATUS;
    fn MiSimpleRead(
        FileObject: PFILE_OBJECT, FileOffset: *mut _LARGE_INTEGER,
        Buffer: PVOID, Length: ULONG, Wait: BOOLEAN,
        Irp: *mut PVOID,
    ) -> NTSTATUS;
    fn FsRtlAcquireToCreateMappedSection(
        FileObject: PFILE_OBJECT, SectionPageProtection: ULONG,
    ) -> NTSTATUS;
    fn FsRtlAcquireFileForModWriteEx(
        FileObject: PFILE_OBJECT, EndingOffset: *mut _LARGE_INTEGER, ResourceToRelease: *mut PVOID,
    ) -> NTSTATUS;
    fn FsRtlReleaseFileForModWrite(
        FileObject: PFILE_OBJECT, ResourceToRelease: PVOID,
    );
}
extern "system" {

}
extern "C" {
    fn MA_GetStartingAddress(marea: PVOID) -> ULONG_PTR;
}
extern "C" {
    fn MA_GetEndingAddress(marea: PVOID) -> ULONG_PTR;
}

#[no_mangle]
pub static mut MmMakeSectionAccess: [ACCESS_MASK; 8] = [
    SECTION_MAP_READ,
    SECTION_MAP_READ,
    SECTION_MAP_EXECUTE,
    SECTION_MAP_EXECUTE | SECTION_MAP_READ,
    SECTION_MAP_WRITE,
    SECTION_MAP_READ,
    SECTION_MAP_EXECUTE | SECTION_MAP_WRITE,
    SECTION_MAP_EXECUTE | SECTION_MAP_READ,
];

#[no_mangle]
pub static mut MmMakeFileAccess: [ACCESS_MASK; 8] = [
    FILE_READ_DATA,
    FILE_READ_DATA,
    FILE_EXECUTE,
    FILE_EXECUTE | FILE_READ_DATA,
    FILE_WRITE_DATA | FILE_READ_DATA,
    FILE_READ_DATA,
    FILE_EXECUTE | FILE_WRITE_DATA | FILE_READ_DATA,
    FILE_EXECUTE | FILE_READ_DATA,
];

#[no_mangle]
pub static mut MmUserProtectionToMask1: [i8; 16] = [
    0,
    MM_NOACCESS as i8,
    MM_READONLY as i8,
    MM_INVALID_PROTECTION as i8,
    MM_READWRITE as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_WRITECOPY as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
];

#[no_mangle]
pub static mut MmUserProtectionToMask2: [i8; 16] = [
    0,
    MM_EXECUTE as i8,
    MM_EXECUTE_READ as i8,
    MM_INVALID_PROTECTION as i8,
    MM_EXECUTE_READWRITE as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_EXECUTE_WRITECOPY as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
    MM_INVALID_PROTECTION as i8,
];

#[no_mangle]
pub static mut MmCompatibleProtectionMask: [ULONG; 8] = [
    PAGE_NOACCESS,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY,
    PAGE_NOACCESS | PAGE_EXECUTE,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_EXECUTE | PAGE_EXECUTE_READ,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_READWRITE,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_READWRITE | PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY,
    PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_WRITECOPY,
];

#[no_mangle]
pub static mut MmSession: MMSESSION = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmSectionCommitMutex: _KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmSectionBasedRoot: _MM_AVL_TABLE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmSectionBasedMutex: _KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut MmHighSectionBase: PVOID = core::ptr::null_mut();

unsafe fn MiIsProtectionCompatible(
    section_page_protection: ULONG,
    new_section_page_protection: ULONG,
) -> BOOLEAN {
    let protection_mask = MiMakeProtectionMask(section_page_protection);
    if protection_mask == MM_INVALID_PROTECTION {
        return 0u8;
    }
    let compatible_mask = MmCompatibleProtectionMask[(protection_mask & 0x7) as usize]
        | PAGE_GUARD
        | PAGE_NOCACHE
        | PAGE_WRITECOMBINE;
    if (compatible_mask | new_section_page_protection) == compatible_mask {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiMakeProtectionMask(protect: ULONG) -> ULONG {
    if protect >= (PAGE_WRITECOMBINE * 2) {
        return MM_INVALID_PROTECTION;
    }
    let mask1 = protect & 0xF;
    let mask2 = (protect >> 4) & 0xF;
    let protect_mask = if mask1 == 0 {
        if mask2 == 0 {
            return MM_INVALID_PROTECTION;
        }
        MmUserProtectionToMask2[mask2 as usize] as ULONG
    } else {
        if mask2 != 0 {
            return MM_INVALID_PROTECTION;
        }
        MmUserProtectionToMask1[mask1 as usize] as ULONG
    };
    if protect_mask == MM_INVALID_PROTECTION {
        return MM_INVALID_PROTECTION;
    }
    let mut protect_mask = protect_mask;
    if (protect & PAGE_GUARD) != 0 {
        if (protect_mask == MM_NOACCESS) || ((protect & (PAGE_NOCACHE | PAGE_WRITECOMBINE)) != 0) {
            return MM_INVALID_PROTECTION;
        }
        protect_mask |= MM_GUARDPAGE;
    }
    if (protect & PAGE_NOCACHE) != 0 {
        if (protect_mask == MM_NOACCESS) || ((protect & PAGE_WRITECOMBINE) != 0) {
            return MM_INVALID_PROTECTION;
        }
        protect_mask |= MM_NOCACHE;
    }
    if (protect & PAGE_WRITECOMBINE) != 0 {
        if protect_mask == MM_NOACCESS {
            return MM_INVALID_PROTECTION;
        }
        protect_mask |= MM_NOACCESS;
    }
    protect_mask
}

unsafe fn MiInsertInSystemSpace(
    session: *mut MMSESSION,
    buckets: ULONG,
    control_area: *mut _CONTROL_AREA,
) -> PVOID {
    KeAcquireGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _);
    if ((*session).SystemSpaceHashEntries + 8) > (*session).SystemSpaceHashSize {
        let hash_size = (*session).SystemSpaceHashSize * 2;
        let old_table = (*session).SystemSpaceViewTable;
        (*session).SystemSpaceViewTable = ExAllocatePoolWithTag(
            if session as *mut u8 == &mut MmSession as *mut MMSESSION as *mut u8 {
                NonPagedPool
            } else {
                PagedPool
            },
            hash_size as usize * core::mem::size_of::<_MMVIEW>(),
            TAG_MM,
        ) as *mut _MMVIEW;
        if (*session).SystemSpaceViewTable.is_null() {
            (*session).SystemSpaceViewTable = old_table;
        } else {
            RtlZeroMemory(
                (*session).SystemSpaceViewTable as PVOID,
                hash_size as usize * core::mem::size_of::<_MMVIEW>(),
            );
            (*session).SystemSpaceHashSize = hash_size;
            (*session).SystemSpaceHashKey = (*session).SystemSpaceHashSize - 1;
            for i in 0..((*session).SystemSpaceHashSize / 2) {
                if (*old_table.add(i as usize)).Entry != 0 {
                    let entry_val = (*old_table.add(i as usize)).Entry >> 16;
                    let mut hash = (entry_val % (*session).SystemSpaceHashKey as ntkrnl_sys::ddk::ULONG_PTR) as u32;
                    while (*(*session).SystemSpaceViewTable.add(hash as usize)).Entry != 0 {
                        hash += 1;
                        if hash >= (*session).SystemSpaceHashSize {
                            hash = 0;
                        }
                    }
                    *(*session).SystemSpaceViewTable.add(hash as usize) = *old_table.add(i as usize);
                }
            }
            ExFreePool(old_table as PVOID);
        }
    }
    if (*session).SystemSpaceHashEntries == (*session).SystemSpaceHashSize {
        KeReleaseGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _);
        return core::ptr::null_mut();
    }
    let i = RtlFindClearBitsAndSet((*session).SystemSpaceBitMap as *mut _, buckets, 0);
    if i == 0xFFFFFFFF {
        (*session).BitmapFailures += 1;
        KeReleaseGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _);
        return core::ptr::null_mut();
    }
    let base = ((*session).SystemSpaceViewStart as ULONG_PTR + (i as ULONG_PTR * MI_SYSTEM_VIEW_BUCKET_SIZE)) as PVOID;
    let entry: ntkrnl_sys::ddk::ULONG_PTR = ((base as ULONG_PTR & !(MI_SYSTEM_VIEW_BUCKET_SIZE - 1)) + buckets as ULONG_PTR) as _;
    let mut hash = ((entry >> 16) % (*session).SystemSpaceHashKey as ntkrnl_sys::ddk::ULONG_PTR) as u32;
    while (*(*session).SystemSpaceViewTable.add(hash as usize)).Entry != 0 {
        hash += 1;
        if hash >= (*session).SystemSpaceHashSize {
            hash = 0;
        }
    }
    (*(*session).SystemSpaceViewTable.add(hash as usize)).Entry = entry;
    (*(*session).SystemSpaceViewTable.add(hash as usize)).ControlArea = control_area;
    (*session).SystemSpaceHashEntries += 1;
    KeReleaseGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _);
    base
}

unsafe fn MiAddMappedPtes(
    first_pte: PMMPTE,
    pte_count: PFN_NUMBER,
    control_area: *mut _CONTROL_AREA,
    section_offset: LONGLONG,
) -> NTSTATUS {
    let mut pointer_pte = first_pte;
    let last_pte = first_pte.add(pte_count as usize);
    let subsection = (control_area.add(1)) as PSUBSECTION;
    let mut proto_pte = (*subsection).SubsectionBase;
    let last_proto_pte = (*subsection).SubsectionBase.add((*subsection).PtesInSubsection as usize);
    while pointer_pte < last_pte {
        if proto_pte >= last_proto_pte {
            return STATUS_SUCCESS;
        }
        let mut temp_pte: MMPTE = core::mem::zeroed();
        MI_MAKE_PROTOTYPE_PTE(&mut temp_pte, proto_pte as PMMPTE);
        MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);
        pointer_pte = pointer_pte.add(1);
        proto_pte = proto_pte.add(1);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MiFillSystemPageDirectory(base: PVOID, number_of_bytes: SIZE_T) {
    let base = opaque_ptr(base);
    let mut pointer_pde = MiAddressToPde(base);
    let last_pde = MiAddressToPde((base as ULONG_PTR + number_of_bytes - 1) as PVOID);
    #[cfg(any(target_arch = "x86", target_arch = "arm"))]
    let mut system_map_pde = MmSystemPagePtes.add((pointer_pde as ULONG_PTR & (SYSTEM_PD_SIZE - 1)) / core::mem::size_of::<MMPTE>());
    #[cfg(not(any(target_arch = "x86", target_arch = "arm")))]
    let mut system_map_pde = pointer_pde;
    let mut temp_pde = ValidKernelPde;
    while pointer_pde <= last_pde {
        let old_irql = MiAcquirePfnLock();
        if (*system_map_pde).valid() == false {
            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_TABLE);
            ntkrnl_sys::MI_SET_PROCESS2!((*(PsGetCurrentProcess() as *mut _EPROCESS)).ImageFileName.as_ptr() as *const i8);
            let page_frame_index = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
            temp_pde.set_page_frame_number(page_frame_index as ULONG_PTR);
            #[cfg(any(target_arch = "x86", target_arch = "arm"))]
            // C: MmSystemPageDirectory[(PointerPde - MiAddressToPde(NULL)) / PDE_PER_PAGE]
            // C pointer subtraction gives element count
            let parent_page = MmSystemPageDirectory[
                pointer_pde.offset_from(MiAddressToPde(core::ptr::null_mut())) as usize
                / ntkrnl_sys::mm::PDE_PER_PAGE
            ];
            #[cfg(not(any(target_arch = "x86", target_arch = "arm")))]
            let parent_page = (*MiPdeToPpe(pointer_pde)).page_frame_number() as PFN_NUMBER;
            MiInitializePfnForOtherProcess(
                page_frame_index as _,
                pointer_pde as PVOID,
                parent_page as _,
            );
            MI_WRITE_VALID_PDE(system_map_pde, temp_pde);
            if (*pointer_pde).valid() == false {
                MI_WRITE_VALID_PDE(pointer_pde, temp_pde);
            }
        }
        MiReleasePfnLock(old_irql);
        system_map_pde = system_map_pde.add(1);
        pointer_pde = pointer_pde.add(1);
    }
}

unsafe fn MiCheckPurgeAndUpMapCount(
    control_area: *mut _CONTROL_AREA,
    fail_if_system_views: BOOLEAN,
) -> NTSTATUS {
    let old_irql = MiAcquirePfnLock();
    (*control_area).NumberOfMappedViews += 1;
    (*control_area).NumberOfUserReferences += 1;
    MiReleasePfnLock(old_irql);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MiLocateSubsection(vad: *mut _MMVAD_LONG, vpn: ULONG_PTR) -> PSUBSECTION {
    let vad = opaque_ptr(vad);
    let control_area = (*vad).ControlArea;
    let subsection = (control_area.add(1)) as PSUBSECTION;
    subsection
}

unsafe fn MiSegmentDelete(segment: PSEGMENT) {
    let control_area = (*segment).ControlArea;
    let subsection = (control_area.add(1)) as PSUBSECTION;
    let mut pointer_pte = (*subsection).SubsectionBase as PMMPTE;
    let last_pte = pointer_pte.add((*segment).NonExtendedPtes as usize);
    let old_irql = MiAcquirePfnLock();
    let mut pte_for_proto = MiAddressToPte(pointer_pte as PVOID);
    if (*pte_for_proto).valid() == false {
        MiMakeSystemAddressValidPfn(pointer_pte as PVOID, old_irql);  // nolint: irql-balance (drops the PFN lock via OldIrql before it faults or waits, retakes it after)
    }
    while pointer_pte < last_pte {
        if MiIsPteOnPdeBoundary(pointer_pte) && pointer_pte != (*subsection).SubsectionBase as PMMPTE {
            pte_for_proto = MiAddressToPte(pointer_pte as PVOID);
            if (*pte_for_proto).valid() == false {
                MiMakeSystemAddressValidPfn(pointer_pte as PVOID, old_irql);  // nolint: irql-balance (drops the PFN lock via OldIrql before it faults or waits, retakes it after)
            }
        }
        let temp_pte = *pointer_pte;
        if ((*control_area).u.Flags.Image() == 0) && ((*control_area).FilePointer.is_null()) {
            if mmpte_software_transition(&temp_pte) {
                let page_frame_index = PFN_FROM_PTE(&temp_pte as *const MMPTE as *mut MMPTE);
                let pfn1 = MiGetPfnEntry(page_frame_index);
                MiUnlinkPageFromList(pfn1 as *mut _ as *mut DDK_MMPFN);
                (*pfn1).set_page_location(ActiveAndValid as u16);
                MI_SET_PFN_DELETED(pfn1);
                MiInsertPageInFreeList(page_frame_index as _);
            }
        }
        (*pointer_pte).Long = 0;
        pointer_pte = pointer_pte.add(1);
    }
    MiReleasePfnLock(old_irql);
    // A pagefile-backed segment's commit charge (commit.rs) dies with it.
    if ((*control_area).u.Flags.Image() == 0) && ((*control_area).FilePointer.is_null()) {
        super::commit::MiReturnCommitment((*segment).NumberOfCommittedPages as usize, core::ptr::null_mut());
    }
    ExFreePool(control_area as PVOID);
    ExFreePool(segment as PVOID);
}

unsafe fn MiCheckControlArea(control_area: *mut _CONTROL_AREA, old_irql: KIRQL) {
    let mut delete_segment = false;
    if ((*control_area).NumberOfMappedViews == 0) && ((*control_area).NumberOfSectionReferences == 0) {
        (*control_area).u.Flags.set_BeingDeleted(1);
        delete_segment = true;
    }
    MiReleasePfnLock(old_irql);
    if delete_segment {
        MiSegmentDelete((*control_area).Segment);
    }
}

unsafe fn MiDereferenceControlArea(control_area: *mut _CONTROL_AREA) {
    let old_irql = MiAcquirePfnLock();
    (*control_area).NumberOfMappedViews -= 1;
    (*control_area).NumberOfUserReferences -= 1;
    MiCheckControlArea(control_area, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn MiRemoveMappedView(current_process: PEPROCESS, vad: *mut _MMVAD_LONG) {
    let current_process = opaque_ptr(current_process);
    let vad = opaque_ptr(vad);
    let control_area = (*vad).ControlArea;
    let current_thread = PsGetCurrentThread();
    MiDeleteVirtualAddresses(
        ((*vad).StartingVpn << PAGE_SHIFT) as _,
        (((*vad).EndingVpn << PAGE_SHIFT) as ULONG_PTR | (PAGE_SIZE - 1)) as _,
        vad as *mut _ as ntkrnl_sys::ddk::PMMVAD,
    );
    MiUnlockProcessWorkingSetUnsafe(current_process, current_thread);
    let old_irql = MiAcquirePfnLock();
    (*control_area).NumberOfMappedViews -= 1;
    (*control_area).NumberOfUserReferences -= 1;
    MiCheckControlArea(control_area, old_irql);
}

unsafe fn MiUnmapViewOfSection(
    process: PEPROCESS,
    base_address: PVOID,
    flags: ULONG,
) -> NTSTATUS {
    let mut attached = false;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let current_thread = PsGetCurrentThread();
    let current_process = PsGetCurrentProcess();
    if flags == 0 {
        MmLockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
    }
    let vad = MiLocateVad(&mut (*(process as *mut _EPROCESS)).VadRoot, base_address) as *mut _MMVAD_LONG;
    if vad.is_null() || ((*vad).u.VadFlags.PrivateMemory() != 0) {
        if flags == 0 {
            MmUnlockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
        }
        return STATUS_NOT_MAPPED_VIEW;
    }
    if MI_IS_MEMORY_AREA_VAD(vad as *mut c_void) {
        let status = MiRosUnmapViewOfSection(
            process,
            vad as *mut MEMORY_AREA,
            base_address,
            (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessExiting() as BOOLEAN,
        );
        if flags == 0 {
            MmUnlockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
        }
        return status;
    }
    if current_process as PVOID != process as PVOID {
        KeStackAttachProcess(&mut (*(process as *mut _EPROCESS)).Pcb, &mut apc_state);
        attached = true;
    }
    if (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        if flags == 0 {
            MmUnlockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
        }
        if attached {
            KeUnstackDetachProcess(&mut apc_state);
        }
        return STATUS_PROCESS_IS_TERMINATING;
    }
    let db_base = if (*vad).u.VadFlags.VadType() == VadImageMap as _ {
        ((*vad).StartingVpn >> PAGE_SHIFT) as PVOID
    } else {
        core::ptr::null_mut()
    };
    let region_size = PAGE_SIZE as ULONG_PTR + (((*vad).EndingVpn - (*vad).StartingVpn) << PAGE_SHIFT) as ULONG_PTR;
    let mut status = STATUS_SUCCESS;
    if (*vad).u.VadFlags.NoChange() == 1 {
        status = MiCheckSecuredVad(
            vad as *mut _ as ntkrnl_sys::ddk::PMMVAD,
            ((*vad).StartingVpn >> PAGE_SHIFT) as PVOID,
            region_size as _,
            MM_DELETE_CHECK,
        );
        if status < 0 {
            if flags == 0 {
                MmUnlockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
            }
            if !db_base.is_null() {
                DbgkUnMapViewOfSection(db_base);
            }
            if attached {
                KeUnstackDetachProcess(&mut apc_state);
            }
            return status;
        }
    }
    MiLockProcessWorkingSetUnsafe(process, current_thread);
    MiRemoveNode(vad as PMMADDRESS_NODE, &mut (*(process as *mut _EPROCESS)).VadRoot);
    PsReturnProcessNonPagedPoolQuota(process as PVOID, core::mem::size_of::<_MMVAD_LONG>());
    MiRemoveMappedView(process, vad);
    (*(process as *mut _EPROCESS)).VirtualSize -= region_size as ntkrnl_sys::ddk::SIZE_T;
    if flags == 0 {
        MmUnlockAddressSpace(&mut (*(process as *mut _EPROCESS)).Vm as *mut _ as PVOID);
    }
    ExFreePool(vad as PVOID);
    if !db_base.is_null() {
        DbgkUnMapViewOfSection(db_base);
    }
    if attached {
        KeUnstackDetachProcess(&mut apc_state);
    }
    STATUS_SUCCESS
}

unsafe fn MiSessionCommitPageTables(start_va: PVOID, end_va: PVOID) -> NTSTATUS {
    let session_space = MmSessionSpace as *mut _MM_SESSION_SPACE;
    let mut page_count: PFN_NUMBER = 0;
    let mut actual_pages: PFN_NUMBER = 0;
    let mut start_pde = MiAddressToPde(start_va);
    let end_pde = MiAddressToPde((end_va as ULONG_PTR - 1) as PVOID);
    #[cfg(target_pointer_width = "64")]
    {
        while start_pde <= end_pde {
            if (*start_pde).valid() == false {
                page_count += 1;
            }
            start_pde = start_pde.add(1);
        }
        if page_count == 0 {
            return STATUS_SUCCESS;
        }
        start_pde = MiAddressToPde(start_va);
        while start_pde <= end_pde {
            if (*start_pde).valid() == false {
                let old_irql = MiAcquirePfnLock();
                ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_TABLE);
                ntkrnl_sys::MI_SET_PROCESS2!((*(PsGetCurrentProcess() as *mut _EPROCESS)).ImageFileName.as_ptr() as *const i8);
                let color = {
                    (*session_space).Color += 1;
                    (*session_space).Color & MmSecondaryColorMask
                };
                let page_frame_number = MiRemoveZeroPage(color);
                let mut temp_pde = ValidKernelPdeLocal;
                temp_pde.set_page_frame_number(page_frame_number as ULONG_PTR);
                MI_WRITE_VALID_PDE(start_pde, temp_pde);
                MiInitializePfnForOtherProcess(
                    page_frame_number as _,
                    start_pde as PVOID,
                    (*session_space).SessionPageDirectoryIndex as _,
                );
                MiReleasePfnLock(old_irql);
                actual_pages += 1;
            }
            start_pde = start_pde.add(1);
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut index = ((start_va as ULONG_PTR - MmSessionBase as ULONG_PTR) >> 22) as usize;
        while start_pde <= end_pde {
            if (*((*session_space).PageTables as PMMPTE).add(index)).Long == 0 {
                page_count += 1;
            }
            start_pde = start_pde.add(1);
            index += 1;
        }
        if page_count == 0 {
            return STATUS_SUCCESS;
        }
        start_pde = MiAddressToPde(start_va);
        index = ((start_va as ULONG_PTR - MmSessionBase as ULONG_PTR) >> 22) as usize;
        while start_pde <= end_pde {
            if (*((*session_space).PageTables as PMMPTE).add(index)).Long == 0 {
                let old_irql = MiAcquirePfnLock();
                ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_TABLE);
                ntkrnl_sys::MI_SET_PROCESS2!((*(PsGetCurrentProcess() as *mut _EPROCESS)).ImageFileName.as_ptr() as *const i8);
                let color = {
                    (*session_space).Color += 1;
                    (*session_space).Color & MmSecondaryColorMask
                };
                let page_frame_number = MiRemoveZeroPage(color);
                let mut temp_pde = ValidKernelPdeLocal;
                temp_pde.set_page_frame_number(page_frame_number as ULONG_PTR);
                MI_WRITE_VALID_PDE(start_pde, temp_pde);
                *((*session_space).PageTables as PMMPTE).add(index) = temp_pde;
                MiInitializePfnForOtherProcess(
                    page_frame_number as _,
                    start_pde as PVOID,
                    (*session_space).SessionPageDirectoryIndex as _,
                );
                MiReleasePfnLock(old_irql);
                actual_pages += 1;
            }
            start_pde = start_pde.add(1);
            index += 1;
        }
    }
    if actual_pages != 0 {
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*session_space).NonPageablePages) as *mut isize,
            actual_pages as isize,
        );
        InterlockedExchangeAddSizeT(
            core::ptr::addr_of_mut!((*session_space).CommittedPages) as *mut isize,
            actual_pages as isize,
        );
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn MiMapViewInSystemSpace(
    section: PVOID,
    session: *mut MMSESSION,
    mapped_base: *mut PVOID,
    view_size: PSIZE_T,
    section_offset: *mut _LARGE_INTEGER,
) -> NTSTATUS {
    let section = opaque_ptr(section);
    let session = opaque_ptr(session);
    let mapped_base = opaque_ptr(mapped_base);
    let view_size = opaque_ptr(view_size);
    let section_offset = opaque_ptr(section_offset);
    let control_area = (*(*((section as PSECTION) as *mut _SECTION)).Segment).ControlArea;
    let mut status = MiCheckPurgeAndUpMapCount(control_area, 0u8);
    let section_size = (*((section as PSECTION) as *mut _SECTION)).SizeOfSection.QuadPart;
    if *view_size == 0 {
        if (section_size - (*section_offset).QuadPart) > isize::MAX as i64 {
            MiDereferenceControlArea(control_area);
            return STATUS_INVALID_VIEW_SIZE;
        }
        *view_size = (section_size - (*section_offset).QuadPart) as SIZE_T;
    }
    if ((*section_offset).QuadPart + *view_size as i64) < (*section_offset).QuadPart {
        MiDereferenceControlArea(control_area);
        return STATUS_INVALID_VIEW_SIZE;
    }
    if ((*section_offset).QuadPart + *view_size as i64) > section_size {
        MiDereferenceControlArea(control_area);
        return STATUS_INVALID_VIEW_SIZE;
    }
    let mut buckets = (*view_size / MI_SYSTEM_VIEW_BUCKET_SIZE) as ULONG;
    if (*view_size & (MI_SYSTEM_VIEW_BUCKET_SIZE - 1)) != 0 {
        buckets += 1;
    }
    if buckets >= MI_SYSTEM_VIEW_BUCKET_SIZE as ULONG {
        MiDereferenceControlArea(control_area);
        return STATUS_INVALID_VIEW_SIZE;
    }
    let base = MiInsertInSystemSpace(session, buckets, control_area);
    if base.is_null() {
        MiDereferenceControlArea(control_area);
        return STATUS_NO_MEMORY;
    }
    if session as *mut u8 == &mut MmSession as *mut MMSESSION as *mut u8 {
        MiFillSystemPageDirectory(base, buckets as SIZE_T * MI_SYSTEM_VIEW_BUCKET_SIZE);
        status = STATUS_SUCCESS;
    } else {
        status = MiSessionCommitPageTables(
            base,
            (base as ULONG_PTR + buckets as ULONG_PTR * MI_SYSTEM_VIEW_BUCKET_SIZE) as PVOID,
        );
    }
    status = MiAddMappedPtes(
        MiAddressToPte(base),
        BYTES_TO_PAGES(*view_size),
        control_area,
        (*section_offset).QuadPart,
    );
    *mapped_base = base;
    STATUS_SUCCESS
}

unsafe fn MiMapViewOfDataSection(
    control_area: *mut _CONTROL_AREA,
    process: PEPROCESS,
    base_address: *mut PVOID,
    section_offset: *mut _LARGE_INTEGER,
    view_size: PSIZE_T,
    section: PSECTION,
    inherit_disposition: SECTION_INHERIT,
    protection_mask: ULONG,
    commit_size: SIZE_T,
    zero_bits: ULONG_PTR,
    allocation_type: ULONG,
) -> NTSTATUS {
    let segment = (*control_area).Segment;
    let mut granularity = MM_VIRTMEM_GRANULARITY;
    #[cfg(target_arch = "x86")]
    {
        if (allocation_type & MEM_DOS_LIM) != 0 {
            granularity = PAGE_SIZE;
        }
    }
    if ((allocation_type & MEM_RESERVE) != 0) && ((*control_area).FilePointer.is_null()) {
        return STATUS_INVALID_PARAMETER_9;
    }
    let mut status = MiCheckPurgeAndUpMapCount(control_area, 0u8);
    if status < 0 {
        return status;
    }
    // section.c: the offset is aligned BEFORE the size is derived from it, the derived size
    // must be non-negative and fit SIZE_T (`RtlLongLongToSIZET`), and both branches reject
    // a view over MAXLONG_PTR. This had the alignment after the subtraction (a view 0..64K
    // short) and neither range check, so an offset past the end became a huge SIZE_T.
    const MAXLONG_PTR: SIZE_T = (SIZE_T::MAX) >> 1;
    if *view_size == 0 {
        let mut view_size_ll: LONGLONG = 0;
        (*section_offset).__bindgen_anon_1.LowPart &= !(_64K as u32 - 1);
        if !NT_SUCCESS(RtlLongLongSub(
            (*((section as PSECTION) as *mut _SECTION)).SizeOfSection.QuadPart,
            (*section_offset).QuadPart,
            &mut view_size_ll,
        )) || view_size_ll < 0
            || view_size_ll as u64 > MAXLONG_PTR as u64
        {
            MiDereferenceControlArea(control_area);
            return STATUS_INVALID_VIEW_SIZE;
        }
        *view_size = view_size_ll as SIZE_T;
    } else {
        if !NT_SUCCESS(RtlSIZETAdd(*view_size, ((*section_offset).__bindgen_anon_1.LowPart & (_64K as u32 - 1)) as SIZE_T, view_size))
            || *view_size > MAXLONG_PTR
        {
            MiDereferenceControlArea(control_area);
            return STATUS_INVALID_VIEW_SIZE;
        }
        (*section_offset).__bindgen_anon_1.LowPart &= !(_64K as u32 - 1);
    }
    let subsection = (control_area.add(1)) as PSUBSECTION;
    let pte_offset = ((*section_offset).QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    let mut quota_charge = 0;
    if (commit_size != 0) && ((*segment).NumberOfCommittedPages < (*segment).TotalNumberOfPtes) {
        quota_charge = BYTES_TO_PAGES(commit_size) as ULONG;
    }
    let view_size_in_pages = BYTES_TO_PAGES(*view_size);
    let vad = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_MMVAD_LONG>(), 0x6C646156) as *mut _MMVAD_LONG;
    if vad.is_null() {
        MiDereferenceControlArea(control_area);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlZeroMemory(vad as PVOID, core::mem::size_of::<_MMVAD_LONG>());
    (*vad).u4.Banked = 0xDEADBABEDEADBABEu64 as PVOID;
    (*vad).ControlArea = control_area;
    (*vad).u.VadFlags.set_CommitCharge(0);
    (*vad).u.VadFlags.set_Protection(protection_mask as _);
    (*vad).u2.VadFlags2.set_FileOffset(((*section_offset).QuadPart >> 16) as ULONG);
    (*vad).u2.VadFlags2.set_Inherit(if inherit_disposition == ViewShare { 1 } else { 0 });
    if ((allocation_type & SEC_NO_CHANGE) != 0) || ((*((section as PSECTION) as *mut _SECTION)).u.Flags.NoChange() != 0) {
        (*vad).u.VadFlags.set_NoChange(1);
        (*vad).u2.VadFlags2.set_SecNoChange(1);
    }
    (*vad).FirstPrototypePte = (*subsection).SubsectionBase.add(pte_offset as usize);
    let pte_offset = pte_offset + view_size_in_pages as PFN_NUMBER - 1;
    (*vad).LastContiguousPte = (*subsection).SubsectionBase.add(pte_offset as usize);
    status = STATUS_SUCCESS;
    if quota_charge != 0 {
        // Charged whole before any prototype PTE changes, the excess handed back
        // after (commit.rs); the section, not the mapper, holds it.
        if !super::commit::MiChargeCommitment(quota_charge as usize, core::ptr::null_mut()) {
            ExFreePoolWithTag(vad as PVOID, 0x6C646156);
            MiDereferenceControlArea(control_area);
            return super::commit::STATUS_COMMITMENT_LIMIT;
        }
        let mut pointer_pte = (*vad).FirstPrototypePte as PMMPTE;
        let last_pte = pointer_pte.add(BYTES_TO_PAGES(commit_size) as usize);
        let temp_pte = *(core::ptr::addr_of!((*segment).SegmentPteTemplate) as *const MMPTE);
        KeAcquireGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        let mut quota_excess = 0;
        while pointer_pte < last_pte {
            if (*pointer_pte).Long == 0 {
                MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);
            } else {
                quota_excess += 1;
            }
            pointer_pte = pointer_pte.add(1);
        }
        quota_charge -= quota_excess;
        (*segment).NumberOfCommittedPages += quota_charge;
        KeReleaseGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        super::commit::MiReturnCommitment(quota_excess as usize, core::ptr::null_mut());
    }
    let mut start_address = if !(*base_address).is_null() {
        ALIGN_DOWN_BY((*base_address) as ULONG_PTR, granularity)
    } else if (*((section as PSECTION) as *mut _SECTION)).Address.StartingVpn != 0 {
        (*((section as PSECTION) as *mut _SECTION)).Address.StartingVpn as ULONG_PTR + (*section_offset).__bindgen_anon_1.LowPart as ULONG_PTR
    } else {
        0
    };
    status = PsChargeProcessNonPagedPoolQuota(process as PVOID, core::mem::size_of::<_MMVAD_LONG>());
    if status < 0 {
        ExFreePoolWithTag(vad as PVOID, 0x6C646156);
        MiDereferenceControlArea(control_area);
        KeAcquireGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        (*segment).NumberOfCommittedPages -= quota_charge;
        KeReleaseGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        super::commit::MiReturnCommitment(quota_charge as usize, core::ptr::null_mut());
        return status;
    }
    // ZeroBits mask form (64-bit, value > 32): the value IS the address
    // ceiling; shift form: classic high-zero-bits count. Matches the
    // NtMapViewOfSection/NtAllocateVirtualMemory mask semantics.
    let highest_addr: ULONG_PTR =
        if core::mem::size_of::<ULONG_PTR>() == 8 && zero_bits > 32 {
            zero_bits
        } else {
            MAXULONG_PTR >> zero_bits
        };
    status = MiInsertVadEx(
        vad as *mut _ as ntkrnl_sys::ddk::PMMVAD,
        &mut start_address as *mut usize as *mut _,
        (view_size_in_pages * PAGE_SIZE) as _,
        highest_addr as _,
        granularity as _,
        allocation_type,
    );
    if status < 0 {
        ExFreePoolWithTag(vad as PVOID, 0x6C646156);
        MiDereferenceControlArea(control_area);
        KeAcquireGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        (*segment).NumberOfCommittedPages -= quota_charge;
        KeReleaseGuardedMutex(&mut MmSectionCommitMutex as *mut _ as PKGUARDED_MUTEX);
        super::commit::MiReturnCommitment(quota_charge as usize, core::ptr::null_mut());
        PsReturnProcessNonPagedPoolQuota(PsGetCurrentProcess() as PVOID, core::mem::size_of::<_MMVAD_LONG>());
        return status;
    }
    if (*segment).u2.FirstMappedVa.is_null() {
        (*segment).u2.FirstMappedVa = start_address as PVOID;
    }
    *view_size = view_size_in_pages * PAGE_SIZE;
    *base_address = start_address as PVOID;
    STATUS_SUCCESS
}

unsafe fn MiCreateDataFileMap(
    file: PFILE_OBJECT,
    segment: *mut PSEGMENT,
    maximum_size: *mut _LARGE_INTEGER,
    section_page_protection: ULONG,
    allocation_attributes: ULONG,
    ignore_file_sizing: ULONG,
) -> NTSTATUS {
    *segment = core::ptr::null_mut();
    STATUS_NOT_IMPLEMENTED
}

unsafe fn MiCreatePagingFileMap(
    segment: *mut PSEGMENT,
    maximum_size: ULONG64,
    protection_mask: ULONG,
    allocation_attributes: ULONG,
) -> NTSTATUS {
    if maximum_size == 0 {
        return STATUS_INVALID_PARAMETER_4;
    }
    let size_limit = (MmSizeOfPagedPoolInBytes - core::mem::size_of::<_SEGMENT>()) / core::mem::size_of::<MMPTE>();
    let size_limit = (size_limit as u64) << PAGE_SHIFT;
    if maximum_size > size_limit {
        return STATUS_SECTION_TOO_BIG;
    }
    let pte_count = ((maximum_size + PAGE_SIZE as u64 - 1) >> PAGE_SHIFT) as PFN_COUNT;
    // A SEC_COMMIT section is committed whole at creation, so it is charged whole
    // here (no process: it outlives its creator) and given back by MiSegmentDelete;
    // SEC_RESERVE pages are charged as views commit them. Upstream left this empty.
    let commit_charge = if (allocation_attributes & SEC_COMMIT) != 0 { pte_count as usize } else { 0 };
    if !super::commit::MiChargeCommitment(commit_charge, core::ptr::null_mut()) {
        return super::commit::STATUS_COMMITMENT_LIMIT;
    }
    let new_segment = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_SEGMENT>() + core::mem::size_of::<MMPTE>() * (pte_count as usize - 1),
        0x744D6D4D,
    ) as PSEGMENT;
    if new_segment.is_null() {
        super::commit::MiReturnCommitment(commit_charge, core::ptr::null_mut());
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    *segment = new_segment;
    let control_area = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_CONTROL_AREA>() + core::mem::size_of::<_SUBSECTION>(),
        0x74436D4D,
    ) as *mut _CONTROL_AREA;
    if control_area.is_null() {
        ExFreePoolWithTag(new_segment as PVOID, 0x744D6D4D);
        super::commit::MiReturnCommitment(commit_charge, core::ptr::null_mut());
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlZeroMemory(control_area as PVOID, core::mem::size_of::<_CONTROL_AREA>() + core::mem::size_of::<_SUBSECTION>());
    (*control_area).Segment = new_segment;
    (*control_area).NumberOfSectionReferences = 1;
    (*control_area).NumberOfUserReferences = 1;
    if (allocation_attributes & SEC_BASED) != 0 {
        (*control_area).u.Flags.set_Based(1);
    }
    if (allocation_attributes & SEC_RESERVE) != 0 {
        (*control_area).u.Flags.set_Reserve(1);
    }
    if (allocation_attributes & SEC_COMMIT) != 0 {
        (*control_area).u.Flags.set_Commit(1);
    }
    (*control_area).u.Flags.set_BeingCreated(1);
    let subsection = (control_area.add(1)) as PSUBSECTION;
    (*subsection).ControlArea = control_area;
    (*subsection).PtesInSubsection = pte_count;
    (*subsection).u.SubsectionFlags.set_Protection(protection_mask);
    let pointer_pte = core::ptr::addr_of_mut!((*new_segment).ThePtes[0]) as PMMPTE;
    RtlZeroMemory(new_segment as PVOID, core::mem::size_of::<_SEGMENT>());
    (*new_segment).PrototypePte = pointer_pte as *mut DDK_MMPTE;
    (*new_segment).ControlArea = control_area;
    (*new_segment).u1.CreatingProcess = PsGetCurrentProcess() as *mut _EPROCESS;
    (*new_segment).SizeOfSegment = (pte_count as ULONGLONG) * PAGE_SIZE as ULONGLONG;
    (*new_segment).TotalNumberOfPtes = pte_count;
    (*new_segment).NonExtendedPtes = pte_count;
    (*subsection).SubsectionBase = pointer_pte as *mut DDK_MMPTE;
    let mut temp_pte: MMPTE = core::mem::zeroed();
    if (allocation_attributes & SEC_COMMIT) != 0 {
        temp_pte.set_soft_protection(protection_mask);
        (*new_segment).NumberOfCommittedPages = pte_count;
    }
    (*(core::ptr::addr_of_mut!((*new_segment).SegmentPteTemplate) as *mut MMPTE)).set_soft_protection(protection_mask);
    #[cfg(target_pointer_width = "64")]
    {
        for i in 0..pte_count {
            *pointer_pte.add(i as usize) = temp_pte;
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        RtlFillMemoryUlong(pointer_pte as PVOID, pte_count as SIZE_T * core::mem::size_of::<MMPTE>(), temp_pte.Long as u32);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmGetFileObjectForSection(section_object: PVOID) -> PFILE_OBJECT {
    let section_object = opaque_ptr(section_object);
    let section = section_object as PSECTION;
    if MiIsRosSectionObject(section_object as *mut _SECTION) == 0 {
        return (*(*(*((section as PSECTION) as *mut _SECTION)).Segment).ControlArea).FilePointer;
    }
    (*((*((section as PSECTION) as *mut _SECTION)).Segment as PMM_SECTION_SEGMENT)).FileObject
}

unsafe fn MiGetFileObjectForVad(vad: *mut _MMVAD_LONG) -> PFILE_OBJECT {
    if MI_IS_MEMORY_AREA_VAD(vad as *mut c_void) {
        let memory_area = vad as *mut MEMORY_AREA;
        if (*memory_area).Type == MEMORY_AREA_SECTION_VIEW {
            return (*(*memory_area).SectionData.Segment).FileObject;
        } else {
            return core::ptr::null_mut();
        }
    } else {
        if (*vad).u.VadFlags.PrivateMemory() == 1 {
            return core::ptr::null_mut();
        }
        let control_area = (*vad).ControlArea;
        if control_area.is_null() || (*control_area).u.Flags.Image() == 0 {
            return core::ptr::null_mut();
        }
        return (*control_area).FilePointer;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmGetImageInformation(image_information: PSECTION_IMAGE_INFORMATION) {
    let section_object = (*(PsGetCurrentProcess() as *mut _EPROCESS)).SectionObject;
    if (*((section_object as PSECTION) as *mut _SECTION)).u.Flags.Image() == 0 {
        RtlZeroMemory(image_information as PVOID, core::mem::size_of::<_SECTION_IMAGE_INFORMATION>());
        return;
    }
    *image_information = (*((*((section_object as PSECTION) as *mut _SECTION)).Segment as PMM_IMAGE_SECTION_OBJECT)).ImageInformation;
}

unsafe fn MmGetFileNameForFileObject(
    file_object: PFILE_OBJECT,
    module_name: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let object_name_info = ExAllocatePoolWithTag(PagedPool, 1024, TAG_MM) as POBJECT_NAME_INFORMATION;
    if object_name_info.is_null() {
        return STATUS_NO_MEMORY;
    }
    let mut return_length: ULONG = 0;
    let status = ObQueryNameString(
        file_object as *mut c_void,
        object_name_info,
        1024,
        &mut return_length,
    );
    if status < 0 {
        ExFreePoolWithTag(object_name_info as PVOID, TAG_MM);
        *module_name = core::ptr::null_mut();
        return status;
    }
    *module_name = object_name_info;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmGetFileNameForSection(
    section: PVOID,
    module_name: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let section = opaque_ptr(section);
    let module_name = opaque_ptr(module_name);
    let section_object = section as PSECTION;
    if (*((section_object as PSECTION) as *mut _SECTION)).u.Flags.Image() == 0 {
        return STATUS_SECTION_NOT_IMAGE;
    }
    let file_object = MmGetFileObjectForSection(section);
    MmGetFileNameForFileObject(file_object, module_name)
}

#[no_mangle]
pub unsafe extern "system" fn MmGetFileNameForAddress(
    address: PVOID,
    module_name: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let address = opaque_ptr(address);
    let module_name = opaque_ptr(module_name);
    let address_space = MmGetCurrentAddressSpace();
    MmLockAddressSpace(address_space as PVOID);
    let vad = MiLocateAddress(address) as *mut _MMVAD_LONG;
    if vad.is_null() {
        MmUnlockAddressSpace(address_space as PVOID);
        return STATUS_INVALID_ADDRESS;
    }
    let file_object = MiGetFileObjectForVad(vad);
    if file_object.is_null() {
        MmUnlockAddressSpace(address_space as PVOID);
        return STATUS_SECTION_NOT_IMAGE;
    }
    ObReferenceObject(file_object as *mut c_void);
    MmUnlockAddressSpace(address_space as PVOID);
    let mut module_name_information: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let status = MmGetFileNameForFileObject(file_object, &mut module_name_information);
    ObDereferenceObject(file_object as *mut c_void);
    if status >= 0 {
        if RtlCreateUnicodeString(module_name, (*module_name_information).Name.Buffer) == 0 {
            ExFreePoolWithTag(module_name_information as PVOID, TAG_MM);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        ExFreePoolWithTag(module_name_information as PVOID, TAG_MM);
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn MiQueryMemorySectionName(
    process_handle: HANDLE,
    base_address: PVOID,
    memory_information: PVOID,
    memory_information_length: SIZE_T,
    return_length: PSIZE_T,
) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let base_address = opaque_ptr(base_address);
    let memory_information = opaque_ptr(memory_information);
    let return_length = opaque_ptr(return_length);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let previous_mode = ExGetPreviousMode();
    let mut status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_QUERY_INFORMATION,
        core::ptr::null_mut(),
        previous_mode,
        &mut process as *mut PEPROCESS as *mut *mut c_void,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }
    let mut module_file_name: _UNICODE_STRING = core::mem::zeroed();
    status = MmGetFileNameForAddress(base_address, &mut module_file_name);
    if status >= 0 {
        let section_name = memory_information as *mut _MEMORY_SECTION_NAME;
        if previous_mode != 0i8 {
            RtlInitEmptyUnicodeString(
                &mut (*section_name).SectionFileName,
                (section_name.add(1)) as PWSTR,
                (memory_information_length - core::mem::size_of::<_MEMORY_SECTION_NAME>()) as USHORT,
            );
            RtlCopyUnicodeString(&mut (*section_name).SectionFileName, &module_file_name);
            if !return_length.is_null() {
                *return_length = module_file_name.Length as SIZE_T + core::mem::size_of::<_MEMORY_SECTION_NAME>();
            }
        } else {
            RtlInitEmptyUnicodeString(
                &mut (*section_name).SectionFileName,
                (section_name.add(1)) as PWSTR,
                (memory_information_length - core::mem::size_of::<_MEMORY_SECTION_NAME>()) as USHORT,
            );
            RtlCopyUnicodeString(&mut (*section_name).SectionFileName, &module_file_name);
            if !return_length.is_null() {
                *return_length = module_file_name.Length as SIZE_T + core::mem::size_of::<_MEMORY_SECTION_NAME>();
            }
        }
        RtlFreeUnicodeString(&mut module_file_name);
    }
    ObDereferenceObject(process as *mut c_void);
    status
}

#[no_mangle]
pub unsafe extern "system" fn MiFlushTbAndCapture(
    found_vad: *mut _MMVAD_LONG,
    pointer_pte: PMMPTE,
    mut protection_mask: ULONG,
    pfn1: PMMPFN,
    update_dirty: BOOLEAN,
) {
    let found_vad = opaque_ptr(found_vad);
    let pointer_pte = opaque_ptr(pointer_pte);
    let pfn1 = opaque_ptr(pfn1);
    let previous_pte = *pointer_pte;
    let mut temp_pte: MMPTE = core::mem::zeroed();
    MI_MAKE_HARDWARE_PTE_USER(
        &mut temp_pte,
        pointer_pte,
        protection_mask as ULONG_PTR,
        previous_pte.page_frame_number() as PFN_NUMBER,
    );
    let old_irql = MiAcquirePfnLock();
    let mut rebuild_pte = false;
    if (*pfn1).cache_attribute() == MiCached as u16 {
        if (protection_mask & (MM_NOCACHE | MM_NOACCESS)) != 0 {
            rebuild_pte = true;
            // C-exact: REASSIGN (C `ProtectionMask &= ~(...)`) — the
            // translation shadowed with a block-local `let`, so the rebuilt
            // PTE below reused the unmodified mask (branch was a no-op).
            protection_mask &= !(MM_NOCACHE | MM_NOACCESS);
        }
    } else if (*pfn1).cache_attribute() == MiNonCached as u16 {
        if (protection_mask & (MM_NOCACHE | MM_NOACCESS)) != MM_NOCACHE {
            rebuild_pte = true;
            protection_mask = (protection_mask & !MM_NOACCESS) | MM_NOCACHE; // same: reassign, not shadow
        }
    }
    if rebuild_pte {
        MI_MAKE_HARDWARE_PTE_USER(
            &mut temp_pte,
            pointer_pte,
            protection_mask as ULONG_PTR,
            previous_pte.page_frame_number() as PFN_NUMBER,
        );
    }
    MI_UPDATE_VALID_PTE(pointer_pte, temp_pte);
    // All CPUs, not the C's KeFlushCurrentTb: see virtual.rs MiProcessValidPteList.
    // arm64 invalidates just this VA: `tlbi vae1is` broadcasts in hardware and every PTE
    // here is global, so it reaches every CPU's copy of this one entry. The full
    // `vmalle1is` ran once per page of every NtProtectVirtualMemory, and under TCG cost
    // about 12 ms a call: FEX re-protects guest code pages constantly (cpl-console made
    // 312 such calls and timed out under appstorm64).
    #[cfg(target_arch = "aarch64")]
    ntkrnl_sys::inline::mm::KeInvalidateTlbEntry(MiPteToAddress(pointer_pte));
    #[cfg(not(target_arch = "aarch64"))]
    ntkrnl_sys::inline::mm::KeFlushProcessTb();
    #[cfg(target_arch = "aarch64")]
    {
        if (update_dirty != 0) && !MI_IS_PAGE_DIRTY(&previous_pte) {
            if !(*pfn1).modified() {}
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        if (update_dirty != 0) && MI_IS_PAGE_DIRTY(&previous_pte) {
            if !(*pfn1).modified() {}
        }
    }
    MiReleasePfnLock(old_irql);
}

unsafe fn MiRemoveMappedPtes(
    base_address: PVOID,
    number_of_ptes: ULONG,
    control_area: *mut _CONTROL_AREA,
    ws: *mut c_void,
) {
    let mut pointer_pte = MiAddressToPte(base_address);
    let mut number_of_ptes = number_of_ptes;
    while number_of_ptes != 0 {
        let pte_contents = *pointer_pte;
        if pte_contents.valid() {
            let pfn1 = MiGetPfnEntry(PFN_FROM_PTE(&pte_contents as *const MMPTE as *mut MMPTE));
            let pointer_pde = MiPteToPde(pointer_pte);
            let old_irql = MiAcquirePfnLock();
            if MI_IS_PAGE_DIRTY(&pte_contents) {
                (*pfn1).set_modified(true);
            }
            if (*pointer_pde).Long == 0 {
                #[cfg(any(target_arch = "x86", target_arch = "arm"))]
                {
                    let system_map_pde = &mut *MmSystemPagePtes.add(((pointer_pde as ULONG_PTR & (SYSTEM_PD_SIZE - 1)) / core::mem::size_of::<MMPTE>()) as usize);
                    MI_WRITE_VALID_PDE(pointer_pde, *system_map_pde);
                }
            }
            let pfn2 = MiGetPfnEntry(PFN_FROM_PTE(pointer_pde));
            MiDecrementShareCount(pfn2 as *mut _ as *mut DDK_MMPFN, PFN_FROM_PTE(pointer_pde) as _);
            MiDecrementShareCount(pfn1 as *mut _ as *mut DDK_MMPFN, PFN_FROM_PTE(&pte_contents as *const MMPTE as *mut MMPTE) as _);
            MiReleasePfnLock(old_irql);
        } else {
            if mmpte_software_prototype(&pte_contents) {
                let _proto_pte = MiProtoPteToPte(&pte_contents as *const MMPTE as PMMPTE);
            }
        }
        (*pointer_pte).Long = 0;
        pointer_pte = pointer_pte.add(1);
        number_of_ptes -= 1;
    }
    // All CPUs, not the C's KeFlushCurrentTb: see virtual.rs MiProcessValidPteList.
    ntkrnl_sys::inline::mm::KeFlushProcessTb();
    let old_irql = MiAcquirePfnLock();
    (*control_area).NumberOfUserReferences -= 1;
    (*control_area).NumberOfMappedViews -= 1;
    MiCheckControlArea(control_area, old_irql);
}

unsafe fn MiRemoveFromSystemSpace(
    session: *mut MMSESSION,
    base: PVOID,
    control_area: *mut *mut _CONTROL_AREA,
) -> ULONG {
    let entry = (base as ULONG_PTR) >> 16;
    let mut hash = (entry % (*session).SystemSpaceHashKey as ULONG_PTR) as usize;
    let mut count = 0;
    while ((*(*session).SystemSpaceViewTable.add(hash)).Entry >> 16) != entry as _ {
        hash += 1;
        if hash >= (*session).SystemSpaceHashSize as usize {
            hash = 0;
            count += 1;
            if count == 2 {
                KeBugCheckEx(
                    DRIVER_UNMAPPING_INVALID_VIEW,
                    base as ULONG_PTR,
                    1,
                    0,
                    0,
                );
            }
        }
    }
    (*session).SystemSpaceHashEntries -= 1;
    let size = ((*(*session).SystemSpaceViewTable.add(hash)).Entry & 0xFFFF) as ULONG;
    (*(*session).SystemSpaceViewTable.add(hash)).Entry = 0;
    *control_area = (*(*session).SystemSpaceViewTable.add(hash)).ControlArea;
    size
}

unsafe fn MiUnmapViewInSystemSpace(session: *mut MMSESSION, mapped_base: PVOID) -> NTSTATUS {
    KeAcquireGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _ as PKGUARDED_MUTEX);
    let mut control_area: *mut _CONTROL_AREA = core::ptr::null_mut();
    let size = MiRemoveFromSystemSpace(session, mapped_base, &mut control_area);
    RtlClearBits(
        (*session).SystemSpaceBitMap as *mut _ as PRTL_BITMAP,
        ((mapped_base as ULONG_PTR - (*session).SystemSpaceViewStart as ULONG_PTR) >> 16) as ULONG,
        size,
    );
    let size = size * (_64K >> PAGE_SHIFT) as ULONG;
    MiRemoveMappedPtes(mapped_base, size, control_area, core::ptr::null_mut());
    KeReleaseGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _ as PKGUARDED_MUTEX);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateArm3Section(
    section_object: *mut PVOID,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    input_maximum_size: *mut _LARGE_INTEGER,
    section_page_protection: ULONG,
    allocation_attributes: ULONG,
    file_handle: HANDLE,
    file_object: PFILE_OBJECT,
) -> NTSTATUS {
    let section_object = opaque_ptr(section_object);
    let object_attributes = opaque_ptr(object_attributes);
    let input_maximum_size = opaque_ptr(input_maximum_size);
    let file_handle = opaque_ptr(file_handle);
    let file_object = opaque_ptr(file_object);
    let mut section: _SECTION = core::mem::zeroed();
    let previous_mode = ExGetPreviousMode();
    let mut file_lock = false;
    let kernel_call = false;
    let mut user_ref_incremented = false;
    let mut new_segment: PSEGMENT = core::ptr::null_mut();
    let mut control_area: *mut _CONTROL_AREA = core::ptr::null_mut();
    let mut file: PFILE_OBJECT = core::ptr::null_mut();
    let section_page_protection = if (allocation_attributes & SEC_NOCACHE) != 0 {
        section_page_protection | PAGE_NOCACHE
    } else {
        section_page_protection
    };
    let protection_mask = MiMakeProtectionMask(section_page_protection);
    if protection_mask == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }
    if (file_handle != core::ptr::null_mut()) || (!file_object.is_null()) {
        if (allocation_attributes & SEC_LARGE_PAGES) != 0 {
            return STATUS_INVALID_PARAMETER_6;
        }
        let mut status = ObReferenceObjectByHandle(
            file_handle,
            MmMakeFileAccess[(protection_mask & 0x7) as usize],
            IoFileObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
            previous_mode,
            &mut file as *mut PFILE_OBJECT as *mut *mut c_void,
            core::ptr::null_mut(),
        );
        if status < 0 {
            return status;
        }
        if (*file).SectionObjectPointer.is_null() {
            ObDereferenceObject(file as *mut c_void);
            return STATUS_INVALID_FILE_FOR_SECTION;
        }
        let control_area_size = core::mem::size_of::<_CONTROL_AREA>() + core::mem::size_of::<_MSUBSECTION>();
        control_area = ExAllocatePoolWithTag(NonPagedPool, control_area_size, 0x61436D4D) as *mut _CONTROL_AREA;
        if control_area.is_null() {
            ObDereferenceObject(file as *mut c_void);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        RtlZeroMemory(control_area as PVOID, control_area_size);
        if file_handle != core::ptr::null_mut() {
            IoSetTopLevelIrp(0x01usize as ntkrnl_sys::ddk::PIRP); // FSRTL_FSP_TOP_LEVEL_IRP
            file_lock = true;
        }
        let old_irql = MiAcquirePfnLock();
        (*control_area).u.Flags.set_BeingCreated(1);
        let _previous_section_pointer = (*file).SectionObjectPointer;
        (*(*file).SectionObjectPointer).DataSectionObject = control_area as *mut c_void;
        let mut segment: PSEGMENT = core::ptr::null_mut();
        MiReleasePfnLock(old_irql);
        status = MiCreateDataFileMap(
            file,
            &mut segment,
            input_maximum_size,
            section_page_protection,
            allocation_attributes,
            kernel_call as ULONG,
        );
        if status < 0 {
            let old_irql = MiAcquirePfnLock();
            (*control_area).WaitingForDeletion = core::ptr::null_mut();
            (*control_area).u.Flags.set_FilePointerNull(1);
            (*(*file).SectionObjectPointer).DataSectionObject = core::ptr::null_mut();
            (*control_area).u.Flags.set_BeingCreated(0);
            MiReleasePfnLock(old_irql);
            if file_lock {
                IoSetTopLevelIrp(core::ptr::null_mut());
            }
            ExFreePool(control_area as PVOID);
            ObDereferenceObject(file as *mut c_void);
            return status;
        }
        new_segment = segment;
        if (*input_maximum_size).QuadPart == 0 {
            section.SizeOfSection.QuadPart = (*new_segment).SizeOfSegment as LONGLONG;
        } else {
            section.SizeOfSection.QuadPart = (*input_maximum_size).QuadPart;
        }
        // Clear BeingCreated on the control area
        let old_irql = MiAcquirePfnLock();
        (*control_area).WaitingForDeletion = core::ptr::null_mut();
        (*control_area).u.Flags.set_BeingCreated(0);
        MiReleasePfnLock(old_irql);
    } else {
        if (allocation_attributes & SEC_IMAGE) != 0 {
            return STATUS_INVALID_FILE_FOR_SECTION;
        }
        let status = MiCreatePagingFileMap(
            &mut new_segment,
            (*input_maximum_size).QuadPart as ULONG64,
            protection_mask,
            allocation_attributes,
        );
        if status < 0 {
            return status;
        }
        section.SizeOfSection.QuadPart = (*new_segment).SizeOfSegment as LONGLONG;
        control_area = (*new_segment).ControlArea;
        user_ref_incremented = true;
    }

    // Check if we locked the file earlier
    if file_lock {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    // Set the initial section object data
    section.InitialPageProtection = section_page_protection;

    // The mapping created a control area and segment, save the flags
    section.Segment = new_segment;
    section.u.LongFlags = (*control_area).u.LongFlags;

    // Check if this is a user-mode read-write non-image file mapping
    if file_object.is_null()
        && (section_page_protection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) != 0
        && (*control_area).u.Flags.Image() == 0
        && !(*control_area).FilePointer.is_null()
    {
        section.u.Flags.set_UserWritable(1);
        InterlockedIncrement(
            &mut (*control_area).WritableUserReferences as *mut ULONG as *mut LONG,
        );
    }

    // Check for image mappings or page file mappings
    let (paged_charge, size) =
        if (*control_area).u.Flags.Image() != 0 || (*control_area).FilePointer.is_null() {
            // Charge the segment size, and allocate a subsection
            (
                core::mem::size_of::<_SECTION>() as ULONG
                    + (*new_segment).TotalNumberOfPtes
                        * core::mem::size_of::<DDK_MMPTE>() as ULONG,
                core::mem::size_of::<_SUBSECTION>() as ULONG,
            )
        } else {
            // Charge nothing, and allocate a mapped subsection
            (0u32, core::mem::size_of::<_MSUBSECTION>() as ULONG)
        };

    // Charge only a CA, and the subsection is right after
    let non_paged_charge = core::mem::size_of::<_CONTROL_AREA>() as ULONG + size;

    // Create the actual section object, with enough space for the prototype PTEs
    let mut new_section: PSECTION = core::ptr::null_mut();
    let mut status = ObCreateObject(
        previous_mode,
        MmSectionObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
        object_attributes,
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_SECTION>() as ULONG,
        paged_charge,
        non_paged_charge,
        &mut new_section as *mut PSECTION as *mut PVOID,
    );
    if status < 0 {
        // Check if this is a user-mode read-write non-image file mapping
        if file_object.is_null()
            && (section_page_protection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) != 0
            && (*control_area).u.Flags.Image() == 0
            && !(*control_area).FilePointer.is_null()
        {
            InterlockedDecrement(
                &mut (*control_area).WritableUserReferences as *mut ULONG as *mut LONG,
            );
        }
        // Check if a user reference was added
        if user_ref_incremented {
            let old_irql = MiAcquirePfnLock();
            (*control_area).NumberOfSectionReferences -= 1;
            (*control_area).NumberOfUserReferences -= 1;
            MiCheckControlArea(control_area, old_irql);
        }
        return status;
    }

    // NOTE: Past this point, all failures will be handled by Ob upon ref->0

    // Now copy the local section object from the stack into this new object
    core::ptr::copy_nonoverlapping(&section as *const _SECTION, new_section, 1);
    (*new_section).Address.StartingVpn = 0;

    // For now, only user calls are supported
    (*new_section).u.Flags.set_UserReference(1);

    // Is this a "based" allocation, in which all mappings are identical?
    if (allocation_attributes & SEC_BASED) != 0 {
        KeAcquireGuardedMutex(&mut MmSectionBasedMutex as *mut _ as PKGUARDED_MUTEX);
        if (*control_area).u.Flags.BeingCreated() == 1 {
            status = MiFindEmptyAddressRangeDownBasedTree(
                (*(*control_area).Segment).SizeOfSegment as _,
                MmHighSectionBase as _,
                _64K as _,
                &mut MmSectionBasedRoot as *mut _ as *mut ntkrnl_sys::ddk::_MM_AVL_TABLE,
                &mut (*(*control_area).Segment).BasedAddress as *mut PVOID as *mut _,
            );
            if status < 0 {
                KeReleaseGuardedMutex(&mut MmSectionBasedMutex as *mut _ as PKGUARDED_MUTEX);
                (*control_area).u.Flags.set_Based(0);
                (*new_section).u.Flags.set_Based(0);
                ObDereferenceObject(new_section as *mut c_void);
                return status;
            }
            #[cfg(target_pointer_width = "64")]
            {
                (*new_section).Address.StartingVpn =
                    (*(*control_area).Segment).BasedAddress as u64;
                (*new_section).Address.EndingVpn = (*new_section).Address.StartingVpn
                    + (*new_section).SizeOfSection.QuadPart as u64
                    - 1;
            }
            #[cfg(target_pointer_width = "32")]
            {
                (*new_section).Address.StartingVpn =
                    (*(*control_area).Segment).BasedAddress as u32;
                (*new_section).Address.EndingVpn = (*new_section).Address.StartingVpn
                    + (*new_section).SizeOfSection.QuadPart as u32
                    - 1;
            }
            MiInsertBasedSection(new_section);
        }
        KeReleaseGuardedMutex(&mut MmSectionBasedMutex as *mut _ as PKGUARDED_MUTEX);
    }

    // The control area is not being created anymore
    if (*control_area).u.Flags.BeingCreated() == 1 {
        let old_irql = MiAcquirePfnLock();
        (*control_area).u.Flags.set_BeingCreated(0);
        (*new_section).u.Flags.set_BeingCreated(0);
        MiReleasePfnLock(old_irql);
    }

    // Migrate the attribute into a flag
    if (allocation_attributes & SEC_NO_CHANGE) != 0 {
        (*new_section).u.Flags.set_NoChange(1);
    }

    // If R/W access is not requested, this might eventually become a CoW mapping
    if (section_page_protection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) == 0 {
        (*new_section).u.Flags.set_CopyOnWrite(1);
    }

    // Write down if this was a kernel call
    (*control_area)
        .u
        .Flags
        .set_WasPurged((*control_area).u.Flags.WasPurged() | kernel_call as ULONG);

    // Make sure the segment and the section are the same size, or the section is smaller
    ntkrnl_sys::ASSERT!((*new_section).SizeOfSection.QuadPart as u64 <= (*new_section).Segment.read().SizeOfSegment);

    // Return the object and the creation status
    *section_object = new_section as PVOID;
    status
}

#[no_mangle]
pub unsafe extern "system" fn MmMapViewOfArm3Section(
    section_object: PVOID,
    process: PEPROCESS,
    base_address: *mut PVOID,
    zero_bits: ULONG_PTR,
    commit_size: SIZE_T,
    section_offset: *mut _LARGE_INTEGER,
    view_size: PSIZE_T,
    inherit_disposition: SECTION_INHERIT,
    allocation_type: ULONG,
    protect: ULONG,
) -> NTSTATUS {
    let section_object = opaque_ptr(section_object);
    let process = opaque_ptr(process);
    let base_address = opaque_ptr(base_address);
    let section_offset = opaque_ptr(section_offset);
    let view_size = opaque_ptr(view_size);
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut attached = false;

    // Get the segment and control area
    let section = section_object as *mut _SECTION;
    let control_area = (*(*section).Segment).ControlArea;

    // FIXME
    if (allocation_type & MEM_RESERVE) != 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    // Check if the mapping protection is compatible with the create
    if MiIsProtectionCompatible((*section).InitialPageProtection, protect) == 0 {
        return STATUS_SECTION_PROTECTION;
    }

    // Check if the offset and size would cause an overflow
    if ((*section_offset).QuadPart as u64).wrapping_add(*view_size as u64)
        < (*section_offset).QuadPart as u64
    {
        return STATUS_INVALID_VIEW_SIZE;
    }

    // Check if the offset and size are bigger than the section itself
    if ((*section_offset).QuadPart as u64).wrapping_add(*view_size as u64)
        > (*section).SizeOfSection.QuadPart as u64
    {
        return STATUS_INVALID_VIEW_SIZE;
    }

    // Check if the caller did not specify a view size
    if *view_size == 0 {
        let calculated_view_size = (*section).SizeOfSection.QuadPart as u64
            - (*section_offset).QuadPart as u64;
        // On 32-bit, check for overflow; on 64-bit, ULONGLONG == SIZE_T
        #[cfg(target_pointer_width = "32")]
        {
            if calculated_view_size > u32::MAX as u64 {
                return STATUS_INVALID_VIEW_SIZE;
            }
        }
        *view_size = calculated_view_size as SIZE_T;
        if (MM_HIGHEST_VAD_ADDRESS as ULONG_PTR - *base_address as ULONG_PTR) < *view_size {
            return STATUS_INVALID_VIEW_SIZE;
        }
    }

    // Check if the commit size is larger than the view size
    if commit_size > *view_size {
        return STATUS_INVALID_PARAMETER_5;
    }

    // Check if the view size is larger than the section
    if (*view_size as u64) > (*section).SizeOfSection.QuadPart as u64 {
        return STATUS_INVALID_VIEW_SIZE;
    }

    // Compute and validate the protection mask
    let protection_mask = MiMakeProtectionMask(protect);
    if protection_mask == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    // We only handle pagefile-backed sections, which cannot be writecombined
    if (protect & PAGE_WRITECOMBINE) != 0 {
        return STATUS_INVALID_PARAMETER_10;
    }

    // Start by attaching to the current process if needed
    if PsGetCurrentProcess() as PVOID != process as PVOID {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*process).Pcb),
            core::ptr::addr_of_mut!(apc_state),
        );
        attached = true;
    }

    // Do the actual mapping
    let status = MiMapViewOfDataSection(
        control_area,
        process,
        base_address,
        section_offset,
        view_size,
        section as PSECTION,
        inherit_disposition,
        protection_mask,
        commit_size,
        zero_bits,
        allocation_type,
    );

    // Detach if needed, then return status
    if attached {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(apc_state));
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn MmDisableModifiedWriteOfSection(
    section_object_pointer: PSECTION_OBJECT_POINTERS,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmForceSectionClosed(
    section_object_pointer: PSECTION_OBJECT_POINTERS,
    delay_close: BOOLEAN,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn MmMapViewInSessionSpace(
    section: PVOID,
    mapped_base: *mut PVOID,
    view_size: PSIZE_T,
) -> NTSTATUS {
    let section = opaque_ptr(section);
    let mapped_base = opaque_ptr(mapped_base);
    let view_size = opaque_ptr(view_size);
    let is_ros = MiIsRosSectionObject(section as *mut _SECTION);
    if is_ros != 0 {
        let status = MmMapViewInSystemSpace(section, mapped_base, view_size);
        return status;
    }
    let in_session = (*(PsGetCurrentProcess() as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessInSession();
    if in_session == 0 {
        return STATUS_NOT_MAPPED_VIEW;
    }
    let mut section_offset: _LARGE_INTEGER = core::mem::zeroed();
    section_offset.QuadPart = 0;
    let session_space = MmSessionSpace as *mut _MM_SESSION_SPACE;
    MiMapViewInSystemSpace(section, &mut (*session_space).Session, mapped_base, view_size, &mut section_offset)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapViewInSessionSpace(mapped_base: PVOID) -> NTSTATUS {
    let mapped_base = opaque_ptr(mapped_base);
    if !MI_IS_SESSION_ADDRESS(mapped_base) {
        return MmUnmapViewInSystemSpace(mapped_base);
    }
    if (*(PsGetCurrentProcess() as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessInSession() == 0 {
        return STATUS_NOT_MAPPED_VIEW;
    }
    let session_space = MmSessionSpace as *mut _MM_SESSION_SPACE;
    MiUnmapViewInSystemSpace(&mut (*session_space).Session, mapped_base)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapViewOfSection(process: PEPROCESS, base_address: PVOID) -> NTSTATUS {
    let process = opaque_ptr(process);
    let base_address = opaque_ptr(base_address);
    MiUnmapViewOfSection(process, base_address, 0)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapViewInSystemSpace(mapped_base: PVOID) -> NTSTATUS {
    let mapped_base = opaque_ptr(mapped_base);
    MmLockAddressSpace(MmGetKernelAddressSpace() as PVOID);
    let memory_area = MmLocateMemoryAreaByAddress(MmGetKernelAddressSpace(), mapped_base);
    if (!memory_area.is_null()) && ((*memory_area).Type != MEMORY_AREA_OWNED_BY_ARM3) {
        let status = MiRosUnmapViewInSystemSpace(mapped_base);
        MmUnlockAddressSpace(MmGetKernelAddressSpace() as PVOID);
        return status;
    }
    MmUnlockAddressSpace(MmGetKernelAddressSpace() as PVOID);
    MiUnmapViewInSystemSpace(&mut MmSession, mapped_base)
}

#[no_mangle]
pub unsafe extern "system" fn MmCommitSessionMappedView(
    mapped_base: PVOID,
    view_size: SIZE_T,
) -> NTSTATUS {
    let mapped_base = opaque_ptr(mapped_base);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn MiDeleteARM3Section(object_body: PVOID) {
    let object_body = opaque_ptr(object_body);
    let section_object = object_body as PSECTION;
    if (*((section_object as PSECTION) as *mut _SECTION)).u.Flags.Based() == 1 {
        KeAcquireGuardedMutex(&mut MmSectionBasedMutex as *mut _ as PKGUARDED_MUTEX);
        MiRemoveNode(&mut (*((section_object as PSECTION) as *mut _SECTION)).Address as *mut _ as *mut _MMADDRESS_NODE, &mut MmSectionBasedRoot);
        KeReleaseGuardedMutex(&mut MmSectionBasedMutex as *mut _ as PKGUARDED_MUTEX);
    }
    let old_irql = MiAcquirePfnLock();
    let control_area = (*(*((section_object as PSECTION) as *mut _SECTION)).Segment).ControlArea;
    (*control_area).NumberOfSectionReferences -= 1;
    (*control_area).NumberOfUserReferences -= 1;
    MiCheckControlArea(control_area, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn MmDoesFileHaveUserWritableReferences(
    section_pointer: PSECTION_OBJECT_POINTERS,
) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtAreMappedFilesTheSame(
    file1_mapped_as_an_image: PVOID,
    file2_mapped_as_file: PVOID,
) -> NTSTATUS {
    let file1_mapped_as_an_image = opaque_ptr(file1_mapped_as_an_image);
    let file2_mapped_as_file = opaque_ptr(file2_mapped_as_file);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateSection(
    section_handle: *mut HANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    maximum_size: *mut _LARGE_INTEGER,
    section_page_protection: ULONG,
    allocation_attributes: ULONG,
    file_handle: HANDLE,
) -> NTSTATUS {
    let section_handle = opaque_ptr(section_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let maximum_size = opaque_ptr(maximum_size);
    let file_handle = opaque_ptr(file_handle);
    let mut safe_maximum_size: _LARGE_INTEGER = core::mem::zeroed();
    let previous_mode = ExGetPreviousMode();
    if (allocation_attributes & !(SEC_COMMIT | SEC_RESERVE | SEC_BASED | SEC_LARGE_PAGES | SEC_IMAGE | SEC_NOCACHE | SEC_NO_CHANGE)) != 0 {
        if (allocation_attributes & 1) == 0 {
            return STATUS_INVALID_PARAMETER_6;
        }
    }
    if (allocation_attributes & (SEC_COMMIT | SEC_RESERVE | SEC_IMAGE)) == 0 {
        return STATUS_INVALID_PARAMETER_6;
    }
    if ((allocation_attributes & SEC_IMAGE) != 0) && ((allocation_attributes & (SEC_COMMIT | SEC_RESERVE | SEC_LARGE_PAGES | SEC_NOCACHE | SEC_NO_CHANGE)) != 0) {
        return STATUS_INVALID_PARAMETER_6;
    }
    if ((allocation_attributes & SEC_COMMIT) != 0) && ((allocation_attributes & SEC_RESERVE) != 0) {
        return STATUS_INVALID_PARAMETER_6;
    }
    if ((section_page_protection & PAGE_NOCACHE) != 0) || ((section_page_protection & PAGE_WRITECOMBINE) != 0) || ((section_page_protection & PAGE_GUARD) != 0) || ((section_page_protection & PAGE_NOACCESS) != 0) {
        return STATUS_INVALID_PAGE_PROTECTION;
    }
    safe_maximum_size.QuadPart = 0;
    if previous_mode != 0i8 {
        // Faithful to the C's ProbeForReadLargeInteger(MaximumSize) — validate the
        // user pointer (range only: the typed macro checks no alignment) and capture
        // it fault-tolerantly. C: inside _SEH2_TRY, handler returns the exception
        // code. (conformance: kmtest NtCreateSection deliberately passes a bad
        // MaximumSize.)
        if !maximum_size.is_null() {
            safe_maximum_size = match ntkrnl_sys::probe::probe_read_generic(maximum_size as *const _LARGE_INTEGER, 1, previous_mode) {
                Ok(v) => v,
                Err(status) => return status,
            };
        }
        // C: inside _SEH2_TRY, handler returns the exception code. No SEH here, so the
        // Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(section_handle, 1, previous_mode) {
            return status;
        }
    } else {
        if !maximum_size.is_null() {
            safe_maximum_size = match ntkrnl_sys::probe::read_back(maximum_size) { Ok(v) => v, Err(s) => return s };
        }
    }
    if (file_handle == core::ptr::null_mut()) && (safe_maximum_size.QuadPart == 0) {
        return STATUS_INVALID_PARAMETER_4;
    }
    let mut section_object: PVOID = core::ptr::null_mut();
    let mut status = MmCreateSection(
        &mut section_object,
        desired_access,
        object_attributes,
        &mut safe_maximum_size,
        section_page_protection,
        allocation_attributes,
        file_handle,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }
    let mut handle: HANDLE = core::ptr::null_mut();
    status = ObInsertObject(
        section_object,
        core::ptr::null_mut(),
        desired_access,
        0,
        core::ptr::null_mut(),
        &mut handle,
    );
    if status >= 0 {
        // C: _SEH2_TRY { *SectionHandle = Handle; } _SEH2_EXCEPT { /* Nothing here */ }
        let _ = ntkrnl_sys::probe::safe_copy_raw(
            section_handle as PVOID,
            &handle as *const HANDLE as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
        );
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenSection(
    section_handle: *mut HANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let section_handle = opaque_ptr(section_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let previous_mode = ExGetPreviousMode();
    if previous_mode != 0i8 {
        // C: inside _SEH2_TRY, handler returns the exception code. No SEH here, so the
        // Result probe returns it instead of raising unhandled.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(section_handle, 1, previous_mode) {
            return status;
        }
    }
    let mut handle: HANDLE = core::ptr::null_mut();
    let status = ObOpenObjectByName(
        object_attributes,
        MmSectionObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
        previous_mode,
        core::ptr::null_mut(),
        desired_access,
        core::ptr::null_mut(),
        &mut handle,
    );
    // C: _SEH2_TRY { *SectionHandle = Handle; } _SEH2_EXCEPT { /* Nothing here */ }
    let _ = ntkrnl_sys::probe::safe_copy_raw(
        section_handle as PVOID,
        &handle as *const HANDLE as *const core::ffi::c_void,
        core::mem::size_of::<HANDLE>(),
    );
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtMapViewOfSection(
    section_handle: HANDLE,
    process_handle: HANDLE,
    base_address: *mut PVOID,
    zero_bits: ULONG_PTR,
    commit_size: SIZE_T,
    section_offset: *mut _LARGE_INTEGER,
    view_size: PSIZE_T,
    inherit_disposition: SECTION_INHERIT,
    allocation_type: ULONG,
    protect: ULONG,
) -> NTSTATUS {
    let section_handle = opaque_ptr(section_handle);
    let process_handle = opaque_ptr(process_handle);
    let base_address = opaque_ptr(base_address);
    let section_offset = opaque_ptr(section_offset);
    let view_size = opaque_ptr(view_size);
    extern "C" {
        static mut MmHighestPhysicalPage: PFN_NUMBER;
    }

    let previous_mode = ExGetPreviousMode();

    // Valid allocation types
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    const VALID_ALLOC_TYPE: ULONG = 0x100000 | MEM_LARGE_PAGES | MEM_DOS_LIM | SEC_NO_CHANGE | MEM_RESERVE;
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    const VALID_ALLOC_TYPE: ULONG = 0x100000 | MEM_LARGE_PAGES | SEC_NO_CHANGE | MEM_RESERVE;

    // Check for invalid inherit disposition
    if (inherit_disposition > ViewUnmap) || (inherit_disposition < ViewShare) {
        return STATUS_INVALID_PARAMETER_8;
    }

    // Allow only valid allocation types
    if (allocation_type & !VALID_ALLOC_TYPE) != 0 {
        return STATUS_INVALID_PARAMETER_9;
    }

    // Convert the protection mask, and validate it
    let protection_mask = MiMakeProtectionMask(protect);
    if protection_mask == MM_INVALID_PROTECTION {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    // Now convert the protection mask into desired section access mask
    let desired_access = MmMakeSectionAccess[(protection_mask & 0x7) as usize];

    // Assume no section offset
    let mut safe_section_offset = _LARGE_INTEGER { QuadPart: 0 };

    // Check for unsafe parameters
    if previous_mode != 0i8 {
        // Probe the parameters. C: inside _SEH2_TRY, handler returns the exception
        // code. No SEH here, so the Result probe returns it instead of raising.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(base_address, 1, previous_mode) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(view_size, 1, previous_mode) {
            return status;
        }
    }

    // Check if a section offset was given. C: this and the captures below are in
    // the same _SEH2_TRY (for every mode), handler returns the exception code.
    if !section_offset.is_null() {
        // Check for unsafe parameters and capture section offset
        // (ProbeForWriteLargeInteger: range only, no alignment).
        if previous_mode != 0i8 {
            if let Err(status) = ntkrnl_sys::probe::probe_write_generic(section_offset, 1, previous_mode) {
                return status;
            }
        }
        safe_section_offset = match ntkrnl_sys::probe::probe_read_generic(section_offset as *const _LARGE_INTEGER, 1, previous_mode) {
            Ok(v) => v,
            Err(status) => return status,
        };
    }

    // Capture the other parameters
    let safe_base_address = match ntkrnl_sys::probe::probe_read_generic(base_address as *const PVOID, 1, previous_mode) {
        Ok(v) => v,
        Err(status) => return status,
    };
    let safe_view_size = match ntkrnl_sys::probe::probe_read_generic(view_size as *const SIZE_T, 1, previous_mode) {
        Ok(v) => v,
        Err(status) => return status,
    };

    // Check for kernel-mode address
    if safe_base_address > MM_HIGHEST_VAD_ADDRESS as PVOID {
        return STATUS_INVALID_PARAMETER_3;
    }

    // Check for range entering kernel-mode
    if (MM_HIGHEST_VAD_ADDRESS as ULONG_PTR).wrapping_sub(safe_base_address as ULONG_PTR) < safe_view_size {
        return STATUS_INVALID_PARAMETER_3;
    }

    // Check for invalid zero bits. 64-bit NT semantics (same as
    // NtAllocateVirtualMemory in virtual.rs): a ZeroBits above 32 is not a
    // shift count but an address MASK — the view must land at or below it
    // (WOW64 passes (1<<31)-1 to keep views guest-addressable).
    let zero_bits_is_mask =
        core::mem::size_of::<ULONG_PTR>() == 8 && zero_bits > 32;
    if zero_bits != 0 {
        if zero_bits_is_mask {
            if (safe_base_address as ULONG_PTR).wrapping_add(safe_view_size) > zero_bits {
                return STATUS_INVALID_PARAMETER_4;
            }
        } else {
            if zero_bits > MI_MAX_ZERO_BITS() {
                return STATUS_INVALID_PARAMETER_4;
            }

            if ((safe_base_address as ULONG_PTR) << zero_bits) >> zero_bits != safe_base_address as ULONG_PTR {
                return STATUS_INVALID_PARAMETER_4;
            }

            if (((safe_base_address as ULONG_PTR).wrapping_add(safe_view_size)) << zero_bits) >> zero_bits
                != (safe_base_address as ULONG_PTR).wrapping_add(safe_view_size)
            {
                return STATUS_INVALID_PARAMETER_4;
            }
        }
    }

    // Reference the process
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_VM_OPERATION,
        PsProcessType as ntkrnl_sys::ddk::POBJECT_TYPE,
        previous_mode,
        &mut process as *mut PEPROCESS as *mut *mut c_void,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    // Reference the section
    let mut section: PSECTION = core::ptr::null_mut();
    status = ObReferenceObjectByHandle(
        section_handle,
        desired_access,
        MmSectionObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
        previous_mode,
        &mut section as *mut PSECTION as *mut *mut c_void,
        core::ptr::null_mut(),
    );
    if status < 0 {
        ObDereferenceObject(process as *mut c_void);
        return status;
    }

    if (*section).u.Flags.PhysicalMemory() != 0 {
        if previous_mode == 1i8
            && safe_section_offset.QuadPart as u64 + safe_view_size as u64
                > (MmHighestPhysicalPage as u64) << PAGE_SHIFT
        {
            ObDereferenceObject(section as *mut c_void);
            ObDereferenceObject(process as *mut c_void);
            return STATUS_INVALID_PARAMETER_6;
        }
    } else if (allocation_type & MEM_DOS_LIM) == 0 {
        // Check for non-allocation-granularity-aligned BaseAddress
        if safe_base_address as ULONG_PTR != ALIGN_DOWN_BY(safe_base_address as ULONG_PTR, MM_VIRTMEM_GRANULARITY as ULONG_PTR) {
            ObDereferenceObject(section as *mut c_void);
            ObDereferenceObject(process as *mut c_void);
            return STATUS_MAPPED_ALIGNMENT;
        }

        // Do the same for the section offset
        if safe_section_offset.__bindgen_anon_1.LowPart != ALIGN_DOWN_BY(safe_section_offset.__bindgen_anon_1.LowPart as ULONG_PTR, MM_VIRTMEM_GRANULARITY as ULONG_PTR) as u32 {
            ObDereferenceObject(section as *mut c_void);
            ObDereferenceObject(process as *mut c_void);
            return STATUS_MAPPED_ALIGNMENT;
        }
    }

    // Now do the actual mapping
    let mut safe_base_address = safe_base_address;
    let mut safe_view_size = safe_view_size;
    status = MmMapViewOfSection(
        section,
        process,
        &mut safe_base_address,
        zero_bits,
        commit_size,
        &mut safe_section_offset,
        &mut safe_view_size,
        inherit_disposition,
        allocation_type,
        protect,
    );

    // Return data only on success
    if status >= 0 {
        // Check if this is an image for the current process
        if (*section).u.Flags.Image() != 0
            && process == PsGetCurrentProcess() as PEPROCESS
            && status != STATUS_IMAGE_NOT_AT_BASE
        {
            // Notify the debugger
            DbgkMapViewOfSection(
                section as PVOID,
                safe_base_address,
                safe_section_offset.__bindgen_anon_1.LowPart,
                safe_view_size as _,
            );
        }

        // Return parameters to user
        let _ = ntkrnl_sys::probe::write_back(base_address, safe_base_address);
        let _ = ntkrnl_sys::probe::write_back(view_size, safe_view_size);
        if !section_offset.is_null() {
            let _ = ntkrnl_sys::probe::write_back(section_offset, safe_section_offset);
        }
    }

    // Dereference all objects and return status
    ObDereferenceObject(section as *mut c_void);
    ObDereferenceObject(process as *mut c_void);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtUnmapViewOfSection(
    process_handle: HANDLE,
    base_address: PVOID,
) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let base_address = opaque_ptr(base_address);
    let previous_mode = ExGetPreviousMode();
    if (previous_mode == 1i8) && (base_address > MM_HIGHEST_USER_ADDRESS as PVOID) {
        return STATUS_NOT_MAPPED_VIEW;
    }
    let mut process: PEPROCESS = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_VM_OPERATION,
        PsProcessType as ntkrnl_sys::ddk::POBJECT_TYPE,
        previous_mode,
        &mut process as *mut PEPROCESS as *mut *mut c_void,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }
    let status = MiUnmapViewOfSection(process, base_address, 0);
    ObDereferenceObject(process as *mut c_void);
    status
}

#[no_mangle]
// The C probes new_maximum_size (ProbeForWriteLargeInteger) before reading it. This body
// is an unimplemented stub that dereferences nothing, so there is no access to guard —
// restore the probe together with the implementation, not before it.
pub unsafe extern "system" fn NtExtendSection(  // nolint: dropped-probe (stub, derefs nothing)
    section_handle: HANDLE,
    new_maximum_size: *mut _LARGE_INTEGER,
) -> NTSTATUS {
    let section_handle = opaque_ptr(section_handle);
    let new_maximum_size = opaque_ptr(new_maximum_size);
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/section.c
//  * PURPOSE:         ARM Memory Manager Section Support
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
//
// /* GLOBALS ********************************************************************/
//
// ACCESS_MASK MmMakeSectionAccess[8] =
// {
//     SECTION_MAP_READ,
//     SECTION_MAP_READ,
//     SECTION_MAP_EXECUTE,
//     SECTION_MAP_EXECUTE | SECTION_MAP_READ,
//     SECTION_MAP_WRITE,
//     SECTION_MAP_READ,
//     SECTION_MAP_EXECUTE | SECTION_MAP_WRITE,
//     SECTION_MAP_EXECUTE | SECTION_MAP_READ
// };
//
// ACCESS_MASK MmMakeFileAccess[8] =
// {
//     FILE_READ_DATA,
//     FILE_READ_DATA,
//     FILE_EXECUTE,
//     FILE_EXECUTE | FILE_READ_DATA,
//     FILE_WRITE_DATA | FILE_READ_DATA,
//     FILE_READ_DATA,
//     FILE_EXECUTE | FILE_WRITE_DATA | FILE_READ_DATA,
//     FILE_EXECUTE | FILE_READ_DATA
// };
//
// CHAR MmUserProtectionToMask1[16] =
// {
//     0,
//     MM_NOACCESS,
//     MM_READONLY,
//     (CHAR)MM_INVALID_PROTECTION,
//     MM_READWRITE,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     MM_WRITECOPY,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION
// };
//
// CHAR MmUserProtectionToMask2[16] =
// {
//     0,
//     MM_EXECUTE,
//     MM_EXECUTE_READ,
//     (CHAR)MM_INVALID_PROTECTION,
//     MM_EXECUTE_READWRITE,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     MM_EXECUTE_WRITECOPY,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION,
//     (CHAR)MM_INVALID_PROTECTION
// };
//
// ULONG MmCompatibleProtectionMask[8] =
// {
//     PAGE_NOACCESS,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY,
//
//     PAGE_NOACCESS | PAGE_EXECUTE,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_EXECUTE |
//     PAGE_EXECUTE_READ,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_READWRITE,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_READWRITE |
//     PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE |
//     PAGE_EXECUTE_WRITECOPY,
//
//     PAGE_NOACCESS | PAGE_READONLY | PAGE_WRITECOPY | PAGE_EXECUTE |
//     PAGE_EXECUTE_READ | PAGE_EXECUTE_WRITECOPY
// };
//
// MMSESSION MmSession;
// KGUARDED_MUTEX MmSectionCommitMutex;
// MM_AVL_TABLE MmSectionBasedRoot;
// KGUARDED_MUTEX MmSectionBasedMutex;
// PVOID MmHighSectionBase;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// BOOLEAN
// MiIsProtectionCompatible(IN ULONG SectionPageProtection,
//                          IN ULONG NewSectionPageProtection)
// {
//     ULONG ProtectionMask, CompatibleMask;
//
//     /* Calculate the protection mask and make sure it's valid */
//     ProtectionMask = MiMakeProtectionMask(SectionPageProtection);
//     if (ProtectionMask == MM_INVALID_PROTECTION)
//     {
//         DPRINT1("Invalid protection mask\n");
//         return FALSE;
//     }
//
//     /* Calculate the compatible mask */
//     CompatibleMask = MmCompatibleProtectionMask[ProtectionMask & 0x7] |
//                      PAGE_GUARD | PAGE_NOCACHE | PAGE_WRITECOMBINE;
//
//     /* See if the mapping protection is compatible with the create protection */
//     return ((CompatibleMask | NewSectionPageProtection) == CompatibleMask);
// }
//
// ULONG
// NTAPI
// MiMakeProtectionMask(IN ULONG Protect)
// {
//     ULONG Mask1, Mask2, ProtectMask;
//
//     /* PAGE_EXECUTE_WRITECOMBINE is theoretically the maximum */
//     if (Protect >= (PAGE_WRITECOMBINE * 2)) return MM_INVALID_PROTECTION;
//
//     /*
//      * Windows API protection mask can be understood as two bitfields, differing
//      * by whether or not execute rights are being requested
//      */
//     Mask1 = Protect & 0xF;
//     Mask2 = (Protect >> 4) & 0xF;
//
//     /* Check which field is there */
//     if (!Mask1)
//     {
//         /* Mask2 must be there, use it to determine the PTE protection */
//         if (!Mask2) return MM_INVALID_PROTECTION;
//         ProtectMask = MmUserProtectionToMask2[Mask2];
//     }
//     else
//     {
//         /* Mask2 should not be there, use Mask1 to determine the PTE mask */
//         if (Mask2) return MM_INVALID_PROTECTION;
//         ProtectMask = MmUserProtectionToMask1[Mask1];
//     }
//
//     /* Make sure the final mask is a valid one */
//     if (ProtectMask == MM_INVALID_PROTECTION) return MM_INVALID_PROTECTION;
//
//     /* Check for PAGE_GUARD option */
//     if (Protect & PAGE_GUARD)
//     {
//         /* It's not valid on no-access, nocache, or writecombine pages */
//         if ((ProtectMask == MM_NOACCESS) ||
//             (Protect & (PAGE_NOCACHE | PAGE_WRITECOMBINE)))
//         {
//             /* Fail such requests */
//             return MM_INVALID_PROTECTION;
//         }
//
//         /* This actually turns on guard page in this scenario! */
//         ProtectMask |= MM_GUARDPAGE;
//     }
//
//     /* Check for nocache option */
//     if (Protect & PAGE_NOCACHE)
//     {
//         /* The earlier check should've eliminated this possibility */
//         ASSERT((Protect & PAGE_GUARD) == 0);
//
//         /* Check for no-access page or write combine page */
//         if ((ProtectMask == MM_NOACCESS) || (Protect & PAGE_WRITECOMBINE))
//         {
//             /* Such a request is invalid */
//             return MM_INVALID_PROTECTION;
//         }
//
//         /* Add the PTE flag */
//         ProtectMask |= MM_NOCACHE;
//     }
//
//     /* Check for write combine option */
//     if (Protect & PAGE_WRITECOMBINE)
//     {
//         /* The two earlier scenarios should've caught this */
//         ASSERT((Protect & (PAGE_GUARD | PAGE_NOACCESS)) == 0);
//
//         /* Don't allow on no-access pages */
//         if (ProtectMask == MM_NOACCESS) return MM_INVALID_PROTECTION;
//
//         /* This actually turns on write-combine in this scenario! */
//         ProtectMask |= MM_NOACCESS;
//     }
//
//     /* Return the final MM PTE protection mask */
//     return ProtectMask;
// }
//
#[no_mangle]
pub unsafe extern "system" fn MiInitializeSystemSpaceMap(
    InputSession: *mut MMSESSION,
) -> BOOLEAN {
    let InputSession = opaque_ptr(InputSession);
    let session: *mut MMSESSION;
    let view_start: PVOID;
    let size: SIZE_T;

    /* Check if this a session or system space */
    if !InputSession.is_null() {
        session = InputSession;
        view_start = MiSessionViewStart;
        size = super::mminit::MmSessionViewSize;
    } else {
        session = &mut MmSession as *mut MMSESSION;
        view_start = MiSystemViewStart;
        size = super::mminit::MmSystemViewSize;
    }

    /* Initialize the system space lock */
    (*session).SystemSpaceViewLockPointer = &mut (*session).SystemSpaceViewLock as *mut _ as *mut _;
    KeInitializeGuardedMutex((*session).SystemSpaceViewLockPointer as *mut _);

    /* Set the start address */
    (*session).SystemSpaceViewStart = view_start as *mut i8;

    /* Create a bitmap to describe system space */
    let bitmap_size: SIZE_T = core::mem::size_of::<ntkrnl_sys::ddk::_RTL_BITMAP>()
        + ((((size / MI_SYSTEM_VIEW_BUCKET_SIZE) + 31) / 32) * core::mem::size_of::<ULONG>());
    (*session).SystemSpaceBitMap = ExAllocatePoolWithTag(
        NonPagedPool,
        bitmap_size,
        TAG_MM,
    ) as *mut _;
    ASSERT!((*session).SystemSpaceBitMap as usize != 0);
    RtlInitializeBitMap(
        (*session).SystemSpaceBitMap as *mut _,
        ((*session).SystemSpaceBitMap as *mut u8)
            .add(core::mem::size_of::<ntkrnl_sys::ddk::_RTL_BITMAP>()) as PULONG,
        (size / MI_SYSTEM_VIEW_BUCKET_SIZE) as ULONG,
    );

    /* Set system space fully empty to begin with */
    RtlClearAllBits((*session).SystemSpaceBitMap as *mut _);

    /* Set default hash flags */
    (*session).SystemSpaceHashSize = 31;
    (*session).SystemSpaceHashKey = (*session).SystemSpaceHashSize - 1;
    (*session).SystemSpaceHashEntries = 0;

    /* Calculate how much space for the hash views we'll need */
    let alloc_size: SIZE_T =
        core::mem::size_of::<_MMVIEW>() * (*session).SystemSpaceHashSize as usize;
    ASSERT!(alloc_size < PAGE_SIZE);

    /* Allocate and zero the view table */
    (*session).SystemSpaceViewTable = ExAllocatePoolWithTag(
        if session as *mut u8 == &mut MmSession as *mut MMSESSION as *mut u8 {
            NonPagedPool
        } else {
            PagedPool
        },
        alloc_size,
        TAG_MM,
    ) as *mut _MMVIEW;
    ASSERT!(!(*session).SystemSpaceViewTable.is_null());
    RtlZeroMemory((*session).SystemSpaceViewTable as PVOID, alloc_size);

    /* Success */
    TRUE
}
//
// static
// PVOID
// MiInsertInSystemSpace(IN PMMSESSION Session,
//                       IN ULONG Buckets,
//                       IN PCONTROL_AREA ControlArea)
// {
//     PVOID Base;
//     ULONG Entry, Hash, i, HashSize;
//     PMMVIEW OldTable;
//     PAGED_CODE();
//
//     /* Stay within 4GB */
//     ASSERT(Buckets < MI_SYSTEM_VIEW_BUCKET_SIZE);
//
//     /* Lock system space */
//     KeAcquireGuardedMutex(Session->SystemSpaceViewLockPointer);
//
//     /* Check if we're going to exhaust hash entries */
//     if ((Session->SystemSpaceHashEntries + 8) > Session->SystemSpaceHashSize)
//     {
//         /* Double the hash size */
//         HashSize = Session->SystemSpaceHashSize * 2;
//
//         /* Save the old table and allocate a new one */
//         OldTable = Session->SystemSpaceViewTable;
//         Session->SystemSpaceViewTable = ExAllocatePoolWithTag(Session ==
//                                                               &MmSession ?
//                                                               NonPagedPool :
//                                                               PagedPool,
//                                                               HashSize *
//                                                               sizeof(MMVIEW),
//                                                               TAG_MM);
//         if (!Session->SystemSpaceViewTable)
//         {
//             /* Failed to allocate a new table, keep the old one for now */
//             Session->SystemSpaceViewTable = OldTable;
//         }
//         else
//         {
//             /* Clear the new table and set the new ahsh and key */
//             RtlZeroMemory(Session->SystemSpaceViewTable, HashSize * sizeof(MMVIEW));
//             Session->SystemSpaceHashSize = HashSize;
//             Session->SystemSpaceHashKey = Session->SystemSpaceHashSize - 1;
//
//             /* Loop the old table */
//             for (i = 0; i < Session->SystemSpaceHashSize / 2; i++)
//             {
//                 /* Check if the entry was valid */
//                 if (OldTable[i].Entry)
//                 {
//                     /* Re-hash the old entry and search for space in the new table */
//                     Hash = (OldTable[i].Entry >> 16) % Session->SystemSpaceHashKey;
//                     while (Session->SystemSpaceViewTable[Hash].Entry)
//                     {
//                         /* Loop back at the beginning if we had an overflow */
//                         if (++Hash >= Session->SystemSpaceHashSize) Hash = 0;
//                     }
//
//                     /* Write the old entry in the new table */
//                     Session->SystemSpaceViewTable[Hash] = OldTable[i];
//                 }
//             }
//
//             /* Free the old table */
//             ExFreePool(OldTable);
//         }
//     }
//
//     /* Check if we ran out */
//     if (Session->SystemSpaceHashEntries == Session->SystemSpaceHashSize)
//     {
//         DPRINT1("Ran out of system view hash entries\n");
//         KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//         return NULL;
//     }
//
//     /* Find space where to map this view */
//     i = RtlFindClearBitsAndSet(Session->SystemSpaceBitMap, Buckets, 0);
//     if (i == 0xFFFFFFFF)
//     {
//         /* Out of space, fail */
//         Session->BitmapFailures++;
//         DPRINT1("Out of system view space\n");
//         KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//         return NULL;
//     }
//
//     /* Compute the base address */
//     Base = (PVOID)((ULONG_PTR)Session->SystemSpaceViewStart + (i * MI_SYSTEM_VIEW_BUCKET_SIZE));
//
//     /* Get the hash entry for this allocation */
//     Entry = ((ULONG_PTR)Base & ~(MI_SYSTEM_VIEW_BUCKET_SIZE - 1)) + Buckets;
//     Hash = (Entry >> 16) % Session->SystemSpaceHashKey;
//
//     /* Loop hash entries until a free one is found */
//     while (Session->SystemSpaceViewTable[Hash].Entry)
//     {
//         /* Unless we overflow, in which case loop back at hash o */
//         if (++Hash >= Session->SystemSpaceHashSize) Hash = 0;
//     }
//
//     /* Add this entry into the hash table */
//     Session->SystemSpaceViewTable[Hash].Entry = Entry;
//     Session->SystemSpaceViewTable[Hash].ControlArea = ControlArea;
//
//     /* Hash entry found, increment total and return the base address */
//     Session->SystemSpaceHashEntries++;
//     KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//     return Base;
// }
//
// static
// NTSTATUS
// MiAddMappedPtes(IN PMMPTE FirstPte,
//                 IN PFN_NUMBER PteCount,
//                 IN PCONTROL_AREA ControlArea,
//                 IN LONGLONG SectionOffset)
// {
//     MMPTE TempPte;
//     PMMPTE PointerPte, ProtoPte, LastProtoPte, LastPte;
//     PSUBSECTION Subsection;
//
//     /* Mapping at offset not supported yet */
//     ASSERT(SectionOffset == 0);
//
//     /* ARM3 doesn't support this yet */
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//     ASSERT(ControlArea->FilePointer == NULL);
//
//     /* Sanity checks */
//     ASSERT(PteCount != 0);
//     ASSERT(ControlArea->NumberOfMappedViews >= 1);
//     ASSERT(ControlArea->NumberOfUserReferences >= 1);
//     ASSERT(ControlArea->NumberOfSectionReferences != 0);
//     ASSERT(ControlArea->u.Flags.BeingCreated == 0);
//     ASSERT(ControlArea->u.Flags.BeingDeleted == 0);
//     ASSERT(ControlArea->u.Flags.BeingPurged == 0);
//
//     /* Get the PTEs for the actual mapping */
//     PointerPte = FirstPte;
//     LastPte = FirstPte + PteCount;
//
//     /* Get the prototype PTEs that desribe the section mapping in the subsection */
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//     ProtoPte = Subsection->SubsectionBase;
//     LastProtoPte = &Subsection->SubsectionBase[Subsection->PtesInSubsection];
//
//     /* Loop the PTEs for the mapping */
//     while (PointerPte < LastPte)
//     {
//         /* We may have run out of prototype PTEs in this subsection */
//         if (ProtoPte >= LastProtoPte)
//         {
//             /* But we don't handle this yet */
//             ASSERT(FALSE);
//         }
//
//         /* The PTE should be completely clear */
//         ASSERT(PointerPte->u.Long == 0);
//
//         /* Build the prototype PTE and write it */
//         MI_MAKE_PROTOTYPE_PTE(&TempPte, ProtoPte);
//         MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//
//         /* Keep going */
//         PointerPte++;
//         ProtoPte++;
//     }
//
//     /* No failure path */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiFillSystemPageDirectory(IN PVOID Base,
//                           IN SIZE_T NumberOfBytes)
// {
//     PMMPDE PointerPde, LastPde, SystemMapPde;
//     MMPDE TempPde;
//     PFN_NUMBER PageFrameIndex, ParentPage;
//     KIRQL OldIrql;
//     PAGED_CODE();
//
//     /* Find the PDEs needed for this mapping */
//     PointerPde = MiAddressToPde(Base);
//     LastPde = MiAddressToPde((PVOID)((ULONG_PTR)Base + NumberOfBytes - 1));
//
// #if (_MI_PAGING_LEVELS == 2)
//     /* Find the system double-mapped PDE that describes this mapping */
//     SystemMapPde = &MmSystemPagePtes[((ULONG_PTR)PointerPde & (SYSTEM_PD_SIZE - 1)) / sizeof(MMPTE)];
// #else
//     /* We don't have a double mapping */
//     SystemMapPde = PointerPde;
// #endif
//
//     /* Use the PDE template and loop the PDEs */
//     TempPde = ValidKernelPde;
//     while (PointerPde <= LastPde)
//     {
//         /* Lock the PFN database */
//         OldIrql = MiAcquirePfnLock();
//
//         /* Check if we don't already have this PDE mapped */
//         if (SystemMapPde->u.Hard.Valid == 0)
//         {
//             /* Grab a page for it */
//             MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//             MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//             PageFrameIndex = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
//             ASSERT(PageFrameIndex);
//             TempPde.u.Hard.PageFrameNumber = PageFrameIndex;
//
// #if (_MI_PAGING_LEVELS == 2)
//             ParentPage = MmSystemPageDirectory[(PointerPde - MiAddressToPde(NULL)) / PDE_PER_PAGE];
// #else
//             ParentPage = MiPdeToPpe(PointerPde)->u.Hard.PageFrameNumber;
// #endif
//             /* Initialize its PFN entry, with the parent system page directory page table */
//             MiInitializePfnForOtherProcess(PageFrameIndex,
//                                            (PMMPTE)PointerPde,
//                                            ParentPage);
//
//             /* Make the system PDE entry valid */
//             MI_WRITE_VALID_PDE(SystemMapPde, TempPde);
//
//             /* The system PDE entry might be the PDE itself, so check for this */
//             if (PointerPde->u.Hard.Valid == 0)
//             {
//                 /* It's different, so make the real PDE valid too */
//                 MI_WRITE_VALID_PDE(PointerPde, TempPde);
//             }
//         }
//
//         /* Release the lock and keep going with the next PDE */
//         MiReleasePfnLock(OldIrql);
//         SystemMapPde++;
//         PointerPde++;
//     }
// }
//
// static
// NTSTATUS
// MiCheckPurgeAndUpMapCount(IN PCONTROL_AREA ControlArea,
//                           IN BOOLEAN FailIfSystemViews)
// {
//     KIRQL OldIrql;
//
//     /* Flag not yet supported */
//     ASSERT(FailIfSystemViews == FALSE);
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* State not yet supported */
//     ASSERT(ControlArea->u.Flags.BeingPurged == 0);
//
//     /* Increase the reference counts */
//     ControlArea->NumberOfMappedViews++;
//     ControlArea->NumberOfUserReferences++;
//     ASSERT(ControlArea->NumberOfSectionReferences != 0);
//
//     /* Release the PFN lock and return success */
//     MiReleasePfnLock(OldIrql);
//     return STATUS_SUCCESS;
// }
//
// PSUBSECTION
// NTAPI
// MiLocateSubsection(IN PMMVAD Vad,
//                    IN ULONG_PTR Vpn)
// {
//     PSUBSECTION Subsection;
//     PCONTROL_AREA ControlArea;
//     ULONG_PTR PteOffset;
//
//     /* Get the control area */
//     ControlArea = Vad->ControlArea;
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//     ASSERT(ControlArea->u.Flags.Image == 0);
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//
//     /* Get the subsection */
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//
//     /* We only support single-subsection segments */
//     ASSERT(Subsection->SubsectionBase != NULL);
//     ASSERT(Vad->FirstPrototypePte >= Subsection->SubsectionBase);
//     ASSERT(Vad->FirstPrototypePte < &Subsection->SubsectionBase[Subsection->PtesInSubsection]);
//
//     /* Compute the PTE offset */
//     PteOffset = Vpn - Vad->StartingVpn;
//     PteOffset += Vad->FirstPrototypePte - Subsection->SubsectionBase;
//
//     /* Again, we only support single-subsection segments */
//     ASSERT(PteOffset < 0xF0000000);
//     ASSERT(PteOffset < Subsection->PtesInSubsection);
//
//     /* Return the subsection */
//     return Subsection;
// }
//
// static
// VOID
// MiSegmentDelete(IN PSEGMENT Segment)
// {
//     PCONTROL_AREA ControlArea;
//     SEGMENT_FLAGS SegmentFlags;
//     PSUBSECTION Subsection;
//     PMMPTE PointerPte, LastPte, PteForProto;
//     PMMPFN Pfn1;
//     PFN_NUMBER PageFrameIndex;
//     MMPTE TempPte;
//     KIRQL OldIrql;
//
//     /* Capture data */
//     SegmentFlags = Segment->SegmentFlags;
//     ControlArea = Segment->ControlArea;
//
//     /* Make sure control area is on the right delete path */
//     ASSERT(ControlArea->u.Flags.BeingDeleted == 1);
//     ASSERT(ControlArea->WritableUserReferences == 0);
//
//     /* These things are not supported yet */
//     ASSERT(ControlArea->DereferenceList.Flink == NULL);
//     ASSERT(!(ControlArea->u.Flags.Image) && !(ControlArea->u.Flags.File));
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//
//     /* Get the subsection and PTEs for this segment */
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//     PointerPte = Subsection->SubsectionBase;
//     LastPte = PointerPte + Segment->NonExtendedPtes;
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Check if the master PTE is invalid */
//     PteForProto = MiAddressToPte(PointerPte);
//     if (!PteForProto->u.Hard.Valid)
//     {
//         /* Fault it in */
//         MiMakeSystemAddressValidPfn(PointerPte, OldIrql);
//     }
//
//     /* Loop all the segment PTEs */
//     while (PointerPte < LastPte)
//     {
//         /* Check if it's time to switch master PTEs if we passed a PDE boundary */
//         if (MiIsPteOnPdeBoundary(PointerPte) &&
//             (PointerPte != Subsection->SubsectionBase))
//         {
//             /* Check if the master PTE is invalid */
//             PteForProto = MiAddressToPte(PointerPte);
//             if (!PteForProto->u.Hard.Valid)
//             {
//                 /* Fault it in */
//                 MiMakeSystemAddressValidPfn(PointerPte, OldIrql);
//             }
//         }
//
//         /* This should be a prototype PTE */
//         TempPte = *PointerPte;
//         ASSERT(SegmentFlags.LargePages == 0);
//         ASSERT(TempPte.valid() == false);
//
//         /* See if we should clean things up */
//         if (!(ControlArea->u.Flags.Image) && !(ControlArea->u.Flags.File))
//         {
//             /*
//              * This is a section backed by the pagefile. Now that it doesn't exist anymore,
//              * we can give everything back to the system.
//              */
//             ASSERT(TempPte.u.Soft.Prototype == 0);
//
//             if (TempPte.u.Soft.Transition == 1)
//             {
//                 /* We can give the page back for other use */
//                 DPRINT("Releasing page for transition PTE %p\n", PointerPte);
//                 PageFrameIndex = PFN_FROM_PTE(&TempPte);
//                 Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//
//                 /* As this is a paged-backed section, nobody should reference it anymore (no cache or whatever) */
//                 ASSERT(Pfn1->u3.ReferenceCount == 0);
//
//                 /* And it should be in standby or modified list */
//                 ASSERT((Pfn1->u3.e1.PageLocation == ModifiedPageList) || (Pfn1->u3.e1.PageLocation == StandbyPageList));
//
//                 /* Unlink it and put it back in free list */
//                 MiUnlinkPageFromList(Pfn1);
//
//                 /* Temporarily mark this as active and make it free again */
//                 Pfn1->u3.e1.PageLocation = ActiveAndValid;
//                 MI_SET_PFN_DELETED(Pfn1);
//
//                 MiInsertPageInFreeList(PageFrameIndex);
//             }
//             else if (TempPte.u.Soft.PageFileHigh != 0)
//             {
//                 /* Should not happen for now */
//                 ASSERT(FALSE);
//             }
//         }
//         else
//         {
//             /* unsupported for now */
//             ASSERT(FALSE);
//
//             /* File-backed section must have prototype PTEs */
//             ASSERT(TempPte.u.Soft.Prototype == 1);
//         }
//
//         /* Zero the PTE and keep going */
//         PointerPte->u.Long = 0;
//         PointerPte++;
//     }
//
//     /* Release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Free the structures */
//     ExFreePool(ControlArea);
//     ExFreePool(Segment);
// }
//
// static
// VOID
// MiCheckControlArea(IN PCONTROL_AREA ControlArea,
//                    IN KIRQL OldIrql)
// {
//     BOOLEAN DeleteSegment = FALSE;
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Check if this is the last reference or view */
//     if (!(ControlArea->NumberOfMappedViews) &&
//         !(ControlArea->NumberOfSectionReferences))
//     {
//         /* There should be no more user references either */
//         ASSERT(ControlArea->NumberOfUserReferences == 0);
//
//         /* Not yet supported */
//         ASSERT(ControlArea->FilePointer == NULL);
//
//         /* The control area is being destroyed */
//         ControlArea->u.Flags.BeingDeleted = TRUE;
//         DeleteSegment = TRUE;
//     }
//
//     /* Release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Delete the segment if needed */
//     if (DeleteSegment)
//     {
//         /* No more user write references at all */
//         ASSERT(ControlArea->WritableUserReferences == 0);
//         MiSegmentDelete(ControlArea->Segment);
//     }
// }
//
// static
// VOID
// MiDereferenceControlArea(IN PCONTROL_AREA ControlArea)
// {
//     KIRQL OldIrql;
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Drop reference counts */
//     ControlArea->NumberOfMappedViews--;
//     ControlArea->NumberOfUserReferences--;
//
//     /* Check if it's time to delete the CA. This releases the lock */
//     MiCheckControlArea(ControlArea, OldIrql);
// }
//
// VOID
// NTAPI
// MiRemoveMappedView(IN PEPROCESS CurrentProcess,
//                    IN PMMVAD Vad)
// {
//     KIRQL OldIrql;
//     PCONTROL_AREA ControlArea;
//     PETHREAD CurrentThread = PsGetCurrentThread();
//
//     /* Get the control area */
//     ControlArea = Vad->ControlArea;
//
//     /* We only support non-extendable, non-image, pagefile-backed regular sections */
//     ASSERT(Vad->u.VadFlags.VadType == VadNone);
//     ASSERT(Vad->u2.VadFlags2.ExtendableFile == FALSE);
//     ASSERT(ControlArea);
//     ASSERT(ControlArea->FilePointer == NULL);
//     ASSERT(!MI_IS_MEMORY_AREA_VAD(Vad));
//
//     /* Delete the actual virtual memory pages */
//     MiDeleteVirtualAddresses(Vad->StartingVpn << PAGE_SHIFT,
//                              (Vad->EndingVpn << PAGE_SHIFT) | (PAGE_SIZE - 1),
//                              Vad);
//
//     /* Release the working set */
//     MiUnlockProcessWorkingSetUnsafe(CurrentProcess, CurrentThread);
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Remove references */
//     ControlArea->NumberOfMappedViews--;
//     ControlArea->NumberOfUserReferences--;
//
//     /* Check if it should be destroyed */
//     MiCheckControlArea(ControlArea, OldIrql);
// }
//
// static
// NTSTATUS
// MiUnmapViewOfSection(IN PEPROCESS Process,
//                      IN PVOID BaseAddress,
//                      IN ULONG Flags)
// {
//     BOOLEAN Attached = FALSE;
//     KAPC_STATE ApcState;
//     PMMVAD Vad;
//     PVOID DbgBase = NULL;
//     SIZE_T RegionSize;
//     NTSTATUS Status;
//     PETHREAD CurrentThread = PsGetCurrentThread();
//     PEPROCESS CurrentProcess = PsGetCurrentProcess();
//     PAGED_CODE();
//
//     /* Check if we need to lock the address space */
//     if (!Flags) MmLockAddressSpace(&Process->Vm);
//
//     /* Find the VAD for the address and make sure it's a section VAD */
//     Vad = MiLocateVad(&Process->VadRoot, BaseAddress);
//     if (!(Vad) || (Vad->u.VadFlags.PrivateMemory))
//     {
//         /* Couldn't find it, or invalid VAD, fail */
//         DPRINT1("No VAD or invalid VAD\n");
//         if (!Flags) MmUnlockAddressSpace(&Process->Vm);
//         return STATUS_NOT_MAPPED_VIEW;
//     }
//
//     /* Check for RosMm memory area */
//     if (MI_IS_MEMORY_AREA_VAD(Vad))
//     {
//         /* Call Mm API */
//         ASSERT(MI_IS_ROSMM_VAD(Vad));
//         Status = MiRosUnmapViewOfSection(Process, (PMEMORY_AREA)Vad, BaseAddress, Process->ProcessExiting);
//         if (!Flags) MmUnlockAddressSpace(&Process->Vm);
//         return Status;
//     }
//
//     /* Check if we should attach to the process */
//     if (CurrentProcess != Process)
//     {
//         /* The process is different, do an attach */
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//         Attached = TRUE;
//     }
//
//     /* Check if the process is already dead */
//     if (Process->VmDeleted)
//     {
//         /* Fail the call */
//         DPRINT1("Process died!\n");
//         if (!Flags) MmUnlockAddressSpace(&Process->Vm);
//         Status = STATUS_PROCESS_IS_TERMINATING;
//         goto Quickie;
//     }
//
//     /* We should be attached */
//     ASSERT(Process == PsGetCurrentProcess());
//
//     /* We need the base address for the debugger message on image-backed VADs */
//     if (Vad->u.VadFlags.VadType == VadImageMap)
//     {
//         DbgBase = (PVOID)(Vad->StartingVpn >> PAGE_SHIFT);
//     }
//
//     /* Compute the size of the VAD region */
//     RegionSize = PAGE_SIZE + ((Vad->EndingVpn - Vad->StartingVpn) << PAGE_SHIFT);
//
//     /* For SEC_NO_CHANGE sections, we need some extra checks */
//     if (Vad->u.VadFlags.NoChange == 1)
//     {
//         /* Are we allowed to mess with this VAD? */
//         Status = MiCheckSecuredVad(Vad,
//                                    (PVOID)(Vad->StartingVpn >> PAGE_SHIFT),
//                                    RegionSize,
//                                    MM_DELETE_CHECK);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed */
//             DPRINT1("Trying to unmap protected VAD!\n");
//             if (!Flags) MmUnlockAddressSpace(&Process->Vm);
//             goto Quickie;
//         }
//     }
//
//     /* Not currently supported */
//     ASSERT(Vad->u.VadFlags.VadType != VadRotatePhysical);
//
//     /* FIXME: Remove VAD charges */
//
//     /* Lock the working set */
//     MiLockProcessWorkingSetUnsafe(Process, CurrentThread);
//
//     /* Remove the VAD */
//     ASSERT(Process->VadRoot.NumberGenericTableElements >= 1);
//     MiRemoveNode((PMMADDRESS_NODE)Vad, &Process->VadRoot);
//     PsReturnProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//
//     /* Remove the PTEs for this view, which also releases the working set lock */
//     MiRemoveMappedView(Process, Vad);
//
//     /* FIXME: Remove commitment */
//
//     /* Update performance counter and release the lock */
//     Process->VirtualSize -= RegionSize;
//     if (!Flags) MmUnlockAddressSpace(&Process->Vm);
//
//     /* Destroy the VAD and return success */
//     ExFreePool(Vad);
//     Status = STATUS_SUCCESS;
//
//     /* Failure and success case -- send debugger message, detach, and return */
// Quickie:
//     if (DbgBase) DbgkUnMapViewOfSection(DbgBase);
//     if (Attached) KeUnstackDetachProcess(&ApcState);
//     return Status;
// }
//
// static
// NTSTATUS
// MiSessionCommitPageTables(IN PVOID StartVa,
//                           IN PVOID EndVa)
// {
//     KIRQL OldIrql;
//     ULONG Color;
//     PMMPDE StartPde, EndPde;
//     MMPDE TempPde = ValidKernelPdeLocal;
//     PMMPFN Pfn1;
//     PFN_NUMBER PageCount = 0, ActualPages = 0, PageFrameNumber;
// #if (_MI_PAGING_LEVELS < 3)
//     ULONG Index;
// #endif
//
//     /* Windows sanity checks */
//     ASSERT(StartVa >= (PVOID)MmSessionBase);
//     ASSERT(EndVa < (PVOID)MiSessionSpaceEnd);
//     ASSERT(PAGE_ALIGN(EndVa) == EndVa);
//
//     /* Get the start and end PDE, then loop each one */
//     StartPde = MiAddressToPde(StartVa);
//     EndPde = MiAddressToPde((PVOID)((ULONG_PTR)EndVa - 1));
// #if (_MI_PAGING_LEVELS >= 3)
//     /* On 4-level paging, check PDEs directly via self-map */
//     while (StartPde <= EndPde)
//     {
//         if (StartPde->u.Hard.Valid == 0) PageCount++;
//         StartPde++;
//     }
//
//     if (PageCount == 0) return STATUS_SUCCESS;
//
//     StartPde = MiAddressToPde(StartVa);
//     while (StartPde <= EndPde)
//     {
//         if (StartPde->u.Hard.Valid == 0)
//         {
//             ASSERT(MmAvailablePages >= 32);
//             OldIrql = MiAcquirePfnLock();
//             MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//             MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//             Color = (++MmSessionSpace->Color) & MmSecondaryColorMask;
//             PageFrameNumber = MiRemoveZeroPage(Color);
//             TempPde.u.Hard.PageFrameNumber = PageFrameNumber;
//             MI_WRITE_VALID_PDE(StartPde, TempPde);
//             MiInitializePfnForOtherProcess(PageFrameNumber,
//                                            StartPde,
//                                            MmSessionSpace->SessionPageDirectoryIndex);
//             MiReleasePfnLock(OldIrql);
//             Pfn1 = MI_PFN_ELEMENT(PageFrameNumber);
//             ASSERT(Pfn1->u1.Event == NULL);
//             ActualPages++;
//         }
//         StartPde++;
//     }
// #else
//     Index = ((ULONG_PTR)StartVa - (ULONG_PTR)MmSessionBase) >> 22;
//     while (StartPde <= EndPde)
//     {
//         /* If we don't already have a page table for it, increment count */
//         if (MmSessionSpace->PageTables[Index].Long == 0) PageCount++;
//         /* Move to the next one */
//         StartPde++;
//         Index++;
//     }
//
//     /* If there's no page tables to create, bail out */
//     if (PageCount == 0) return STATUS_SUCCESS;
//
//     /* Reset the start PDE and index */
//     StartPde = MiAddressToPde(StartVa);
//     Index = ((ULONG_PTR)StartVa - (ULONG_PTR)MmSessionBase) >> 22;
//
//     /* Loop each PDE while holding the working set lock */
// //  MiLockWorkingSet(PsGetCurrentThread(),
// //                   &MmSessionSpace->GlobalVirtualAddress->Vm);
//     while (StartPde <= EndPde)
//     {
//         /* Check if we already have a page table */
//         if (MmSessionSpace->PageTables[Index].Long == 0)
//         {
//             /* We don't, so the PDE shouldn't be ready yet */
//             ASSERT(StartPde->u.Hard.Valid == 0);
//
//             /* ReactOS check to avoid MiEnsureAvailablePageOrWait */
//             ASSERT(MmAvailablePages >= 32);
//
//             /* Acquire the PFN lock and grab a zero page */
//             OldIrql = MiAcquirePfnLock();
//             MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//             MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//             Color = (++MmSessionSpace->Color) & MmSecondaryColorMask;
//             PageFrameNumber = MiRemoveZeroPage(Color);
//             TempPde.u.Hard.PageFrameNumber = PageFrameNumber;
//             MI_WRITE_VALID_PDE(StartPde, TempPde);
//
//             /* Write the page table in session space structure */
//             ASSERT(MmSessionSpace->PageTables[Index].Long == 0);
//             MmSessionSpace->PageTables[Index] = TempPde;
//
//             /* Initialize the PFN */
//             MiInitializePfnForOtherProcess(PageFrameNumber,
//                                            StartPde,
//                                            MmSessionSpace->SessionPageDirectoryIndex);
//
//             /* And now release the lock */
//             MiReleasePfnLock(OldIrql);
//
//             /* Get the PFN entry and make sure there's no event for it */
//             Pfn1 = MI_PFN_ELEMENT(PageFrameNumber);
//             ASSERT(Pfn1->u1.Event == NULL);
//
//             /* Increment the number of pages */
//             ActualPages++;
//         }
//
//         /* Move to the next PDE */
//         StartPde++;
//         Index++;
//     }
// #endif
//
//     /* Make sure we didn't do more pages than expected */
//     ASSERT(ActualPages <= PageCount);
//
//     /* Release the working set lock */
// //  MiUnlockWorkingSet(PsGetCurrentThread(),
// //                     &MmSessionSpace->GlobalVirtualAddress->Vm);
//
//
//     /* If we did at least one page... */
//     if (ActualPages)
//     {
//         /* Update the performance counters! */
//         InterlockedExchangeAddSizeT(&MmSessionSpace->NonPageablePages, ActualPages);
//         InterlockedExchangeAddSizeT(&MmSessionSpace->CommittedPages, ActualPages);
//     }
//
//     /* Return status */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// MiMapViewInSystemSpace(
//     _In_ PVOID Section,
//     _In_ PMMSESSION Session,
//     _Outptr_result_bytebuffer_ (*ViewSize) PVOID *MappedBase,
//     _Inout_ PSIZE_T ViewSize,
//     _Inout_ PLARGE_INTEGER SectionOffset)
// {
//     PVOID Base;
//     PCONTROL_AREA ControlArea;
//     ULONG Buckets;
//     LONGLONG SectionSize;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the control area, check for any flags ARM3 doesn't yet support */
//     ControlArea = ((PSECTION)Section)->Segment->ControlArea;
//     ASSERT(ControlArea->u.Flags.Image == 0);
//     ASSERT(ControlArea->FilePointer == NULL);
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//     ASSERT(ControlArea->u.Flags.WasPurged == 0);
//
//     /* Increase the reference and map count on the control area, no purges yet */
//     Status = MiCheckPurgeAndUpMapCount(ControlArea, FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Get the section size at creation time */
//     SectionSize = ((PSECTION)Section)->SizeOfSection.QuadPart;
//
//     /* If the caller didn't specify a view size, assume until the end of the section */
//     if (!(*ViewSize))
//     {
//         /* Check for overflow first */
//         if ((SectionSize - SectionOffset->QuadPart) > SIZE_T_MAX)
//         {
//             DPRINT1("Section end is too far away from the specified offset.\n");
//             MiDereferenceControlArea(ControlArea);
//             return STATUS_INVALID_VIEW_SIZE;
//         }
//         *ViewSize = SectionSize - SectionOffset->QuadPart;
//     }
//
//     /* Check overflow */
//     if ((SectionOffset->QuadPart + *ViewSize) < SectionOffset->QuadPart)
//     {
//         DPRINT1("Integer overflow between size & offset!\n");
//         MiDereferenceControlArea(ControlArea);
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Check if the caller wanted a larger section than the view */
//     if (SectionOffset->QuadPart + *ViewSize > SectionSize)
//     {
//         /* Fail */
//         DPRINT1("View is too large\n");
//         MiDereferenceControlArea(ControlArea);
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Get the number of 64K buckets required for this mapping */
//     Buckets = (ULONG)(*ViewSize / MI_SYSTEM_VIEW_BUCKET_SIZE);
//     if (*ViewSize & (MI_SYSTEM_VIEW_BUCKET_SIZE - 1)) Buckets++;
//
//     /* Check if the view is more than 4GB large */
//     if (Buckets >= MI_SYSTEM_VIEW_BUCKET_SIZE)
//     {
//         /* Fail */
//         DPRINT1("View is too large\n");
//         MiDereferenceControlArea(ControlArea);
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Insert this view into system space and get a base address for it */
//     Base = MiInsertInSystemSpace(Session, Buckets, ControlArea);
//     if (!Base)
//     {
//         /* Fail */
//         DPRINT1("Out of system space\n");
//         MiDereferenceControlArea(ControlArea);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* What's the underlying session? */
//     if (Session == &MmSession)
//     {
//         /* Create the PDEs needed for this mapping, and double-map them if needed */
//         MiFillSystemPageDirectory(Base, Buckets * MI_SYSTEM_VIEW_BUCKET_SIZE);
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Create the PDEs needed for this mapping */
//         Status = MiSessionCommitPageTables(Base,
//                                            (PVOID)((ULONG_PTR)Base +
//                                            Buckets * MI_SYSTEM_VIEW_BUCKET_SIZE));
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Create the actual prototype PTEs for this mapping */
//     Status = MiAddMappedPtes(MiAddressToPte(Base),
//                              BYTES_TO_PAGES(*ViewSize),
//                              ControlArea,
//                              SectionOffset->QuadPart);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Return the base address of the mapping and success */
//     *MappedBase = Base;
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// MiMapViewOfDataSection(IN PCONTROL_AREA ControlArea,
//                        IN PEPROCESS Process,
//                        IN PVOID *BaseAddress,
//                        IN PLARGE_INTEGER SectionOffset,
//                        IN PSIZE_T ViewSize,
//                        IN PSECTION Section,
//                        IN SECTION_INHERIT InheritDisposition,
//                        IN ULONG ProtectionMask,
//                        IN SIZE_T CommitSize,
//                        IN ULONG_PTR ZeroBits,
//                        IN ULONG AllocationType)
// {
//     PMMVAD_LONG Vad;
//     ULONG_PTR StartAddress;
//     ULONG_PTR ViewSizeInPages;
//     PSUBSECTION Subsection;
//     PSEGMENT Segment;
//     PFN_NUMBER PteOffset;
//     NTSTATUS Status;
//     ULONG QuotaCharge = 0, QuotaExcess = 0;
//     PMMPTE PointerPte, LastPte;
//     MMPTE TempPte;
//     ULONG Granularity = MM_VIRTMEM_GRANULARITY;
//
//     DPRINT("Mapping ARM3 data section\n");
//
//     /* Get the segment for this section */
//     Segment = ControlArea->Segment;
//
// #ifdef _M_IX86
//     /* ALlow being less restrictive on x86. */
//     if (AllocationType & MEM_DOS_LIM)
//         Granularity = PAGE_SIZE;
// #endif
//
//     /* One can only reserve a file-based mapping, not shared memory! */
//     if ((AllocationType & MEM_RESERVE) && !(ControlArea->FilePointer))
//     {
//         return STATUS_INVALID_PARAMETER_9;
//     }
//
//     /* First, increase the map count. No purging is supported yet */
//     Status = MiCheckPurgeAndUpMapCount(ControlArea, FALSE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if the caller specified the view size */
//     if (!(*ViewSize))
//     {
//         LONGLONG ViewSizeLL;
//
//         /* The caller did not, so pick a 64K aligned view size based on the offset */
//         SectionOffset->LowPart &= ~(_64K - 1);
//
//         /* Calculate size and make sure this fits */
//         if (!NT_SUCCESS(RtlLongLongSub(Section->SizeOfSection.QuadPart, SectionOffset->QuadPart, &ViewSizeLL))
//             || !NT_SUCCESS(RtlLongLongToSIZET(ViewSizeLL, ViewSize))
//             || (*ViewSize > MAXLONG_PTR))
//         {
//             MiDereferenceControlArea(ControlArea);
//             return STATUS_INVALID_VIEW_SIZE;
//         }
//     }
//     else
//     {
//         /* A size was specified, align it to a 64K boundary
//          * and check for overflow or huge value. */
//         if (!NT_SUCCESS(RtlSIZETAdd(*ViewSize, SectionOffset->LowPart & (_64K - 1), ViewSize))
//             || (*ViewSize > MAXLONG_PTR))
//         {
//             MiDereferenceControlArea(ControlArea);
//             return STATUS_INVALID_VIEW_SIZE;
//         }
//
//         /* Align the offset as well to make this an aligned map */
//         SectionOffset->LowPart &= ~((ULONG)_64K - 1);
//     }
//
//     /* We must be dealing with a 64KB aligned offset. This is a Windows ASSERT */
//     ASSERT((SectionOffset->LowPart & ((ULONG)_64K - 1)) == 0);
//
//     /* Windows ASSERTs for this flag */
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//
//     /* Get the subsection. We don't support LARGE_CONTROL_AREA in ARM3 */
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//
//     /* Sections with extended segments are not supported in ARM3 */
//     ASSERT(Segment->SegmentFlags.TotalNumberOfPtes4132 == 0);
//
//     /* Within this section, figure out which PTEs will describe the view */
//     PteOffset = (PFN_NUMBER)(SectionOffset->QuadPart >> PAGE_SHIFT);
//
//     /* The offset must be in this segment's PTE chunk and it must be valid. Windows ASSERTs */
//     ASSERT(PteOffset < Segment->TotalNumberOfPtes);
//     ASSERT(((SectionOffset->QuadPart + *ViewSize + PAGE_SIZE - 1) >> PAGE_SHIFT) >= PteOffset);
//
//     /* In ARM3, only one subsection is used for now. It must contain these PTEs */
//     ASSERT(PteOffset < Subsection->PtesInSubsection);
//
//     /* In ARM3, only page-file backed sections (shared memory) are supported now */
//     ASSERT(ControlArea->FilePointer == NULL);
//
//     /* Windows ASSERTs for this too -- there must be a subsection base address */
//     ASSERT(Subsection->SubsectionBase != NULL);
//
//     /* Compute how much commit space the segment will take */
//     if ((CommitSize) && (Segment->NumberOfCommittedPages < Segment->TotalNumberOfPtes))
//     {
//         /* Charge for the maximum pages */
//         QuotaCharge = BYTES_TO_PAGES(CommitSize);
//     }
//
//     /* ARM3 does not currently support large pages */
//     ASSERT(Segment->SegmentFlags.LargePages == 0);
//
//     /* Calculate how many pages the region spans */
//     ViewSizeInPages = BYTES_TO_PAGES(*ViewSize);
//
//     /* A VAD can now be allocated. Do so and zero it out */
//     /* FIXME: we are allocating a LONG VAD for ReactOS compatibility only */
//     ASSERT((AllocationType & MEM_RESERVE) == 0); /* ARM3 does not support this */
//     Vad = ExAllocatePoolWithTag(NonPagedPool, sizeof(MMVAD_LONG), 'ldaV');
//     if (!Vad)
//     {
//         MiDereferenceControlArea(ControlArea);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Vad, sizeof(MMVAD_LONG));
//     Vad->u4.Banked = (PVOID)(ULONG_PTR)0xDEADBABEDEADBABEULL;
//
//     /* Write all the data required in the VAD for handling a fault */
//     Vad->ControlArea = ControlArea;
//     Vad->u.VadFlags.CommitCharge = 0;
//     Vad->u.VadFlags.Protection = ProtectionMask;
//     Vad->u2.VadFlags2.FileOffset = (ULONG)(SectionOffset->QuadPart >> 16);
//     Vad->u2.VadFlags2.Inherit = (InheritDisposition == ViewShare);
//     if ((AllocationType & SEC_NO_CHANGE) || (Section->u.Flags.NoChange))
//     {
//         /* This isn't really implemented yet, but handle setting the flag */
//         Vad->u.VadFlags.NoChange = 1;
//         Vad->u2.VadFlags2.SecNoChange = 1;
//     }
//
//     /* Finally, write down the first and last prototype PTE */
//     Vad->FirstPrototypePte = &Subsection->SubsectionBase[PteOffset];
//     PteOffset += ViewSizeInPages - 1;
//     ASSERT(PteOffset < Subsection->PtesInSubsection);
//     Vad->LastContiguousPte = &Subsection->SubsectionBase[PteOffset];
//
//     /* Make sure the prototype PTE ranges make sense, this is a Windows ASSERT */
//     ASSERT(Vad->FirstPrototypePte <= Vad->LastContiguousPte);
//
//     /* FIXME: Should setup VAD bitmap */
//     Status = STATUS_SUCCESS;
//
//     /* Check if anything was committed */
//     if (QuotaCharge)
//     {
//         /* Set the start and end PTE addresses, and pick the template PTE */
//         PointerPte = Vad->FirstPrototypePte;
//         LastPte = PointerPte + BYTES_TO_PAGES(CommitSize);
//         TempPte = Segment->SegmentPteTemplate;
//
//         /* Acquire the commit lock and loop all prototype PTEs to be committed */
//         KeAcquireGuardedMutex(&MmSectionCommitMutex);
//         while (PointerPte < LastPte)
//         {
//             /* Make sure the PTE is already invalid */
//             if (PointerPte->u.Long == 0)
//             {
//                 /* And write the invalid PTE */
//                 MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//             }
//             else
//             {
//                 /* The PTE is valid, so skip it */
//                 QuotaExcess++;
//             }
//
//             /* Move to the next PTE */
//             PointerPte++;
//         }
//
//         /* Now check how many pages exactly we committed, and update accounting */
//         ASSERT(QuotaCharge >= QuotaExcess);
//         QuotaCharge -= QuotaExcess;
//         Segment->NumberOfCommittedPages += QuotaCharge;
//         ASSERT(Segment->NumberOfCommittedPages <= Segment->TotalNumberOfPtes);
//
//         /* Now that we're done, release the lock */
//         KeReleaseGuardedMutex(&MmSectionCommitMutex);
//     }
//
//     /* Is it SEC_BASED, or did the caller manually specify an address? */
//     if (*BaseAddress != NULL)
//     {
//         /* Just align what the caller gave us */
//         StartAddress = ALIGN_DOWN_BY((ULONG_PTR)*BaseAddress, Granularity);
//     }
//     else if (Section->Address.StartingVpn != 0)
//     {
//         /* It is a SEC_BASED mapping, use the address that was generated */
//         StartAddress = Section->Address.StartingVpn + SectionOffset->LowPart;
//     }
//     else
//     {
//         StartAddress = 0;
//     }
//
//     Status = PsChargeProcessNonPagedPoolQuota(Process, sizeof(MMVAD_LONG));
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Vad, 'ldaV');
//         MiDereferenceControlArea(ControlArea);
//
//         KeAcquireGuardedMutex(&MmSectionCommitMutex);
//         Segment->NumberOfCommittedPages -= QuotaCharge;
//         KeReleaseGuardedMutex(&MmSectionCommitMutex);
//         return Status;
//     }
//
//     /* Insert the VAD */
//     Status = MiInsertVadEx((PMMVAD)Vad,
//                            &StartAddress,
//                            ViewSizeInPages * PAGE_SIZE,
//                            MAXULONG_PTR >> ZeroBits,
//                            Granularity,
//                            AllocationType);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Vad, 'ldaV');
//         MiDereferenceControlArea(ControlArea);
//
//         KeAcquireGuardedMutex(&MmSectionCommitMutex);
//         Segment->NumberOfCommittedPages -= QuotaCharge;
//         KeReleaseGuardedMutex(&MmSectionCommitMutex);
//
//         PsReturnProcessNonPagedPoolQuota(PsGetCurrentProcess(), sizeof(MMVAD_LONG));
//         return Status;
//     }
//
//     /* Windows stores this for accounting purposes, do so as well */
//     if (!Segment->u2.FirstMappedVa) Segment->u2.FirstMappedVa = (PVOID)StartAddress;
//
//     /* Finally, let the caller know where, and for what size, the view was mapped */
//     *ViewSize = ViewSizeInPages * PAGE_SIZE;
//     *BaseAddress = (PVOID)StartAddress;
//     DPRINT("Start and region: 0x%p, 0x%p\n", *BaseAddress, *ViewSize);
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// MiCreateDataFileMap(IN PFILE_OBJECT File,
//                     OUT PSEGMENT *Segment,
//                     IN PLARGE_INTEGER MaximumSize,
//                     IN ULONG SectionPageProtection,
//                     IN ULONG AllocationAttributes,
//                     IN ULONG IgnoreFileSizing)
// {
//     /* Not yet implemented */
//     ASSERT(FALSE);
//     *Segment = NULL;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// static
// NTSTATUS
// MiCreatePagingFileMap(OUT PSEGMENT *Segment,
//                       IN ULONG64 MaximumSize,
//                       IN ULONG ProtectionMask,
//                       IN ULONG AllocationAttributes)
// {
//     ULONGLONG SizeLimit;
//     PFN_COUNT PteCount;
//     PMMPTE PointerPte;
//     MMPTE TempPte;
//     PCONTROL_AREA ControlArea;
//     PSEGMENT NewSegment;
//     PSUBSECTION Subsection;
//     PAGED_CODE();
//
//     /* No large pages in ARM3 yet */
//     ASSERT((AllocationAttributes & SEC_LARGE_PAGES) == 0);
//
//     /* Pagefile-backed sections need a known size */
//     if (MaximumSize == 0)
//         return STATUS_INVALID_PARAMETER_4;
//
//     /* Calculate the maximum size possible, given the Prototype PTEs we'll need */
//     SizeLimit = MmSizeOfPagedPoolInBytes - sizeof(SEGMENT);
//     SizeLimit /= sizeof(MMPTE);
//     SizeLimit <<= PAGE_SHIFT;
//
//     /* Fail if this size is too big */
//     if (MaximumSize > SizeLimit)
//     {
//         return STATUS_SECTION_TOO_BIG;
//     }
//
//     /* Calculate how many Prototype PTEs will be needed */
//     PteCount = (PFN_COUNT)((MaximumSize + PAGE_SIZE - 1) >> PAGE_SHIFT);
//
//     /* For commited memory, we must have a valid protection mask */
//     if (AllocationAttributes & SEC_COMMIT) ASSERT(ProtectionMask != 0);
//
//     /* The segment contains all the Prototype PTEs, allocate it in paged pool */
//     NewSegment = ExAllocatePoolWithTag(PagedPool,
//                                        sizeof(SEGMENT) +
//                                        sizeof(MMPTE) * (PteCount - 1),
//                                        'tSmM');
//     if (!NewSegment)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     *Segment = NewSegment;
//
//     /* Now allocate the control area, which has the subsection structure */
//     ControlArea = ExAllocatePoolWithTag(NonPagedPool,
//                                         sizeof(CONTROL_AREA) + sizeof(SUBSECTION),
//                                         'tCmM');
//     if (!ControlArea)
//     {
//         ExFreePoolWithTag(Segment, 'tSmM');
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* And zero it out, filling the basic segmnet pointer and reference fields */
//     RtlZeroMemory(ControlArea, sizeof(CONTROL_AREA) + sizeof(SUBSECTION));
//     ControlArea->Segment = NewSegment;
//     ControlArea->NumberOfSectionReferences = 1;
//     ControlArea->NumberOfUserReferences = 1;
//
//     /* Convert allocation attributes to control area flags */
//     if (AllocationAttributes & SEC_BASED) ControlArea->u.Flags.Based = 1;
//     if (AllocationAttributes & SEC_RESERVE) ControlArea->u.Flags.Reserve = 1;
//     if (AllocationAttributes & SEC_COMMIT) ControlArea->u.Flags.Commit = 1;
//
//     /* We just allocated it */
//     ControlArea->u.Flags.BeingCreated = 1;
//
//     /* The subsection follows, write the mask, PTE count and point back to the CA */
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//     Subsection->ControlArea = ControlArea;
//     Subsection->PtesInSubsection = PteCount;
//     Subsection->u.SubsectionFlags.Protection = ProtectionMask;
//
//     /* Zero out the segment's prototype PTEs, and link it with the control area */
//     PointerPte = &NewSegment->ThePtes[0];
//     RtlZeroMemory(NewSegment, sizeof(SEGMENT));
//     NewSegment->PrototypePte = PointerPte;
//     NewSegment->ControlArea = ControlArea;
//
//     /* Save some extra accounting data for the segment as well */
//     NewSegment->u1.CreatingProcess = PsGetCurrentProcess();
//     NewSegment->SizeOfSegment = ((ULONGLONG)PteCount) * PAGE_SIZE;
//     NewSegment->TotalNumberOfPtes = PteCount;
//     NewSegment->NonExtendedPtes = PteCount;
//
//     /* The subsection's base address is the first Prototype PTE in the segment */
//     Subsection->SubsectionBase = PointerPte;
//
//     /* Start with an empty PTE, unless this is a commit operation */
//     TempPte.Long = 0;
//     if (AllocationAttributes & SEC_COMMIT)
//     {
//         /* In which case, write down the protection mask in the Prototype PTEs */
//         TempPte.u.Soft.Protection = ProtectionMask;
//
//         /* For accounting, also mark these pages as being committed */
//         NewSegment->NumberOfCommittedPages = PteCount;
//     }
//
//     /* The template PTE itself for the segment should also have the mask set */
//     NewSegment->SegmentPteTemplate.u.Soft.Protection = ProtectionMask;
//
//     /* Write out the prototype PTEs, for now they're simply demand zero */
// #ifdef _WIN64
//     RtlFillMemoryUlonglong(PointerPte, PteCount * sizeof(MMPTE), TempPte.Long);
// #else
//     RtlFillMemoryUlong(PointerPte, PteCount * sizeof(MMPTE), TempPte.Long);
// #endif
//     return STATUS_SUCCESS;
// }
//
// PFILE_OBJECT
// NTAPI
// MmGetFileObjectForSection(IN PVOID SectionObject)
// {
//     PSECTION Section = SectionObject;
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//     ASSERT(SectionObject != NULL);
//
//     /* Check if it's an ARM3, or ReactOS section */
//     if (MiIsRosSectionObject(SectionObject) == FALSE)
//     {
//         /* Return the file pointer stored in the control area */
//         return Section->Segment->ControlArea->FilePointer;
//     }
//
//     /* Return the file object */
//     return ((PMM_SECTION_SEGMENT)Section->Segment)->FileObject;
// }
//
// static
// PFILE_OBJECT
// MiGetFileObjectForVad(
//     _In_ PMMVAD Vad)
// {
//     PCONTROL_AREA ControlArea;
//     PFILE_OBJECT FileObject;
//
//     /* Check if this is a RosMm memory area */
//     if (MI_IS_MEMORY_AREA_VAD(Vad))
//     {
//         PMEMORY_AREA MemoryArea = (PMEMORY_AREA)Vad;
//
//         /* We do not expect ARM3 memory areas here, those are kernel only */
//         ASSERT(MI_IS_ROSMM_VAD(Vad));
//
//         /* Check if it's a section view (RosMm section) */
//         if (MemoryArea->Type == MEMORY_AREA_SECTION_VIEW)
//         {
//             /* Get the section pointer to the SECTION_OBJECT */
//             FileObject = MemoryArea->SectionData.Segment->FileObject;
//         }
//         else
//         {
// #ifdef NEWCC
//             ASSERT(MemoryArea->Type == MEMORY_AREA_CACHE);
//             DPRINT1("VAD is a cache section!\n");
// #else
//             ASSERT(FALSE);
// #endif
//             return NULL;
//         }
//     }
//     else
//     {
//         /* Make sure it's not a VM VAD */
//         if (Vad->u.VadFlags.PrivateMemory == 1)
//         {
//             DPRINT1("VAD is not a section\n");
//             return NULL;
//         }
//
//         /* Get the control area */
//         ControlArea = Vad->ControlArea;
//         if ((ControlArea == NULL) || !ControlArea->u.Flags.Image)
//         {
//             DPRINT1("Address is not a section\n");
//             return NULL;
//         }
//
//         /* Get the file object */
//         FileObject = ControlArea->FilePointer;
//     }
//
//     /* Return the file object */
//     return FileObject;
// }
//
// VOID
// NTAPI
// MmGetImageInformation (OUT PSECTION_IMAGE_INFORMATION ImageInformation)
// {
//     PSECTION SectionObject;
//
//     /* Get the section object of this process*/
//     SectionObject = PsGetCurrentProcess()->SectionObject;
//     ASSERT(SectionObject != NULL);
//     ASSERT(MiIsRosSectionObject(SectionObject) == TRUE);
//
//     if (SectionObject->u.Flags.Image == 0)
//     {
//         RtlZeroMemory(ImageInformation, sizeof(*ImageInformation));
//         return;
//     }
//
//     /* Return the image information */
//     *ImageInformation = ((PMM_IMAGE_SECTION_OBJECT)SectionObject->Segment)->ImageInformation;
// }
//
// static
// NTSTATUS
// MmGetFileNameForFileObject(IN PFILE_OBJECT FileObject,
//                            OUT POBJECT_NAME_INFORMATION *ModuleName)
// {
//     POBJECT_NAME_INFORMATION ObjectNameInfo;
//     NTSTATUS Status;
//     ULONG ReturnLength;
//
//     /* Allocate memory for our structure */
//     ObjectNameInfo = ExAllocatePoolWithTag(PagedPool, 1024, TAG_MM);
//     if (!ObjectNameInfo) return STATUS_NO_MEMORY;
//
//     /* Query the name */
//     Status = ObQueryNameString(FileObject,
//                                ObjectNameInfo,
//                                1024,
//                                &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, free memory */
//         DPRINT1("Name query failed\n");
//         ExFreePoolWithTag(ObjectNameInfo, TAG_MM);
//         *ModuleName = NULL;
//         return Status;
//     }
//
//     /* Success */
//     *ModuleName = ObjectNameInfo;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MmGetFileNameForSection(IN PVOID Section,
//                         OUT POBJECT_NAME_INFORMATION *ModuleName)
// {
//     PFILE_OBJECT FileObject;
//     PSECTION SectionObject = Section;
//
//     /* Make sure it's an image section */
//     if (SectionObject->u.Flags.Image == 0)
//     {
//         /* It's not, fail */
//         DPRINT1("Not an image section\n");
//         return STATUS_SECTION_NOT_IMAGE;
//     }
//
//     /* Get the file object */
//     FileObject = MmGetFileObjectForSection(Section);
//     return MmGetFileNameForFileObject(FileObject, ModuleName);
// }
//
// NTSTATUS
// NTAPI
// MmGetFileNameForAddress(IN PVOID Address,
//                         OUT PUNICODE_STRING ModuleName)
// {
//     POBJECT_NAME_INFORMATION ModuleNameInformation;
//     PVOID AddressSpace;
//     NTSTATUS Status;
//     PMMVAD Vad;
//     PFILE_OBJECT FileObject = NULL;
//
//     /* Lock address space */
//     AddressSpace = MmGetCurrentAddressSpace();
//     MmLockAddressSpace(AddressSpace);
//
//     /* Get the VAD */
//     Vad = MiLocateAddress(Address);
//     if (Vad == NULL)
//     {
//         /* Fail, the address does not exist */
//         DPRINT1("No VAD at address %p\n", Address);
//         MmUnlockAddressSpace(AddressSpace);
//         return STATUS_INVALID_ADDRESS;
//     }
//
//     /* Get the file object pointer for the VAD */
//     FileObject = MiGetFileObjectForVad(Vad);
//     if (FileObject == NULL)
//     {
//         DPRINT1("Failed to get file object for Address %p\n", Address);
//         MmUnlockAddressSpace(AddressSpace);
//         return STATUS_SECTION_NOT_IMAGE;
//     }
//
//     /* Reference the file object */
//     ObReferenceObject(FileObject);
//
//     /* Unlock address space */
//     MmUnlockAddressSpace(AddressSpace);
//
//     /* Get the filename of the file object */
//     Status = MmGetFileNameForFileObject(FileObject, &ModuleNameInformation);
//
//     /* Dereference the file object */
//     ObDereferenceObject(FileObject);
//
//     /* Check if we were able to get the file object name */
//     if (NT_SUCCESS(Status))
//     {
//         /* Init modulename */
//         if (!RtlCreateUnicodeString(ModuleName, ModuleNameInformation->Name.Buffer))
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Free temp taged buffer from MmGetFileNameForFileObject() */
//         ExFreePoolWithTag(ModuleNameInformation, TAG_MM);
//
//         DPRINT("Found ModuleName %wZ by address %p\n", ModuleName, Address);
//     }
//
//    /* Return status */
//    return Status;
// }
//
// NTSTATUS
// NTAPI
// MiQueryMemorySectionName(IN HANDLE ProcessHandle,
//                          IN PVOID BaseAddress,
//                          OUT PVOID MemoryInformation,
//                          IN SIZE_T MemoryInformationLength,
//                          OUT PSIZE_T ReturnLength)
// {
//     PEPROCESS Process;
//     NTSTATUS Status;
//     UNICODE_STRING ModuleFileName;
//     PMEMORY_SECTION_NAME SectionName = NULL;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_QUERY_INFORMATION,
//                                        NULL,
//                                        PreviousMode,
//                                        (PVOID*)(&Process),
//                                        NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("MiQueryMemorySectionName: ObReferenceObjectByHandle returned %x\n",Status);
//         return Status;
//     }
//
//     Status = MmGetFileNameForAddress(BaseAddress, &ModuleFileName);
//
//     if (NT_SUCCESS(Status))
//     {
//         SectionName = MemoryInformation;
//         if (PreviousMode != KernelMode)
//         {
//             _SEH2_TRY
//             {
//                 RtlInitEmptyUnicodeString(&SectionName->SectionFileName,
//                                           (PWSTR)(SectionName + 1),
//                                           MemoryInformationLength - sizeof(MEMORY_SECTION_NAME));
//                 RtlCopyUnicodeString(&SectionName->SectionFileName, &ModuleFileName);
//
//                 if (ReturnLength) *ReturnLength = ModuleFileName.Length + sizeof(MEMORY_SECTION_NAME);
//
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             RtlInitEmptyUnicodeString(&SectionName->SectionFileName,
//                                       (PWSTR)(SectionName + 1),
//                                       MemoryInformationLength - sizeof(MEMORY_SECTION_NAME));
//             RtlCopyUnicodeString(&SectionName->SectionFileName, &ModuleFileName);
//
//             if (ReturnLength) *ReturnLength = ModuleFileName.Length + sizeof(MEMORY_SECTION_NAME);
//
//         }
//
//         RtlFreeUnicodeString(&ModuleFileName);
//     }
//     ObDereferenceObject(Process);
//     return Status;
// }
//
// VOID
// NTAPI
// MiFlushTbAndCapture(IN PMMVAD FoundVad,
//                     IN PMMPTE PointerPte,
//                     IN ULONG ProtectionMask,
//                     IN PMMPFN Pfn1,
//                     IN BOOLEAN UpdateDirty)
// {
//     MMPTE TempPte, PreviousPte;
//     KIRQL OldIrql;
//     BOOLEAN RebuildPte = FALSE;
//
//     //
//     // User for sanity checking later on
//     //
//     PreviousPte = *PointerPte;
//
//     //
//     // Build the PTE and acquire the PFN lock
//     //
//     MI_MAKE_HARDWARE_PTE_USER(&TempPte,
//                               PointerPte,
//                               ProtectionMask,
//                               PreviousPte.u.Hard.PageFrameNumber);
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // We don't support I/O mappings in this path yet
//     //
//     ASSERT(Pfn1 != NULL);
//     ASSERT(Pfn1->u3.e1.CacheAttribute != MiWriteCombined);
//
//     //
//     // Make sure new protection mask doesn't get in conflict and fix it if it does
//     //
//     if (Pfn1->u3.e1.CacheAttribute == MiCached)
//     {
//         //
//         // This is a cached PFN
//         //
//         if (ProtectionMask & (MM_NOCACHE | MM_NOACCESS))
//         {
//             RebuildPte = TRUE;
//             ProtectionMask &= ~(MM_NOCACHE | MM_NOACCESS);
//         }
//     }
//     else if (Pfn1->u3.e1.CacheAttribute == MiNonCached)
//     {
//         //
//         // This is a non-cached PFN
//         //
//         if ((ProtectionMask & (MM_NOCACHE | MM_NOACCESS)) != MM_NOCACHE)
//         {
//             RebuildPte = TRUE;
//             ProtectionMask &= ~MM_NOACCESS;
//             ProtectionMask |= MM_NOCACHE;
//         }
//     }
//
//     if (RebuildPte)
//     {
//         MI_MAKE_HARDWARE_PTE_USER(&TempPte,
//                                   PointerPte,
//                                   ProtectionMask,
//                                   PreviousPte.u.Hard.PageFrameNumber);
//     }
//
//     //
//     // Write the new PTE, making sure we are only changing the bits
//     //
//     MI_UPDATE_VALID_PTE(PointerPte, TempPte);
//
//     //
//     // Flush the TLB
//     //
//     ASSERT(PreviousPte.u.Hard.Valid == 1);
//     KeFlushCurrentTb();
//     ASSERT(PreviousPte.u.Hard.Valid == 1);
//
//     //
//     // Windows updates the relevant PFN1 information, we currently don't.
//     //
// #ifdef _M_ARM64
//     if (UpdateDirty && !PreviousPte.u.Hard.NotDirty)
// #else
//     if (UpdateDirty && PreviousPte.u.Hard.Dirty)
// #endif
//     {
//         if (!Pfn1->u3.e1.Modified)
//         {
//             DPRINT1("FIXME: Mark PFN as dirty\n");
//         }
//     }
//
//     //
//     // Not supported in ARM3
//     //
//     ASSERT(FoundVad->u.VadFlags.VadType != VadWriteWatch);
//
//     //
//     // Release the PFN lock, we are done
//     //
//     MiReleasePfnLock(OldIrql);
// }
//
// static
// VOID
// MiRemoveMappedPtes(IN PVOID BaseAddress,
//                    IN ULONG NumberOfPtes,
//                    IN PCONTROL_AREA ControlArea,
//                    IN PMMSUPPORT Ws)
// {
//     PMMPTE PointerPte, ProtoPte;//, FirstPte;
//     PMMPDE PointerPde, SystemMapPde;
//     PMMPFN Pfn1, Pfn2;
//     MMPTE PteContents;
//     KIRQL OldIrql;
//     DPRINT("Removing mapped view at: 0x%p\n", BaseAddress);
//
//     ASSERT(Ws == NULL);
//
//     /* Get the PTE and loop each one */
//     PointerPte = MiAddressToPte(BaseAddress);
//     //FirstPte = PointerPte;
//     while (NumberOfPtes)
//     {
//         /* Check if the PTE is already valid */
//         PteContents = *PointerPte;
//         if (PteContents.u.Hard.Valid == 1)
//         {
//             /* Get the PFN entry */
//             Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(&PteContents));
//
//             /* Get the PTE */
//             PointerPde = MiPteToPde(PointerPte);
//
//             /* Lock the PFN database and make sure this isn't a mapped file */
//             OldIrql = MiAcquirePfnLock();
//             ASSERT(((Pfn1->u3.e1.PrototypePte) && (Pfn1->OriginalPte.u.Soft.Prototype)) == 0);
//
//             /* Mark the page as modified accordingly */
//             if (MI_IS_PAGE_DIRTY(&PteContents))
//                 Pfn1->u3.e1.Modified = 1;
//
//             /* Was the PDE invalid */
//             if (PointerPde->u.Long == 0)
//             {
// #if (_MI_PAGING_LEVELS == 2)
//                 /* Find the system double-mapped PDE that describes this mapping */
//                 SystemMapPde = &MmSystemPagePtes[((ULONG_PTR)PointerPde & (SYSTEM_PD_SIZE - 1)) / sizeof(MMPTE)];
//
//                 /* Make it valid */
//                 ASSERT(SystemMapPde->u.Hard.Valid == 1);
//                 MI_WRITE_VALID_PDE(PointerPde, *SystemMapPde);
// #else
//                 DBG_UNREFERENCED_LOCAL_VARIABLE(SystemMapPde);
//                 ASSERT(FALSE);
// #endif
//             }
//
//             /* Dereference the PDE and the PTE */
//             Pfn2 = MiGetPfnEntry(PFN_FROM_PTE(PointerPde));
//             MiDecrementShareCount(Pfn2, PFN_FROM_PTE(PointerPde));
//             DBG_UNREFERENCED_LOCAL_VARIABLE(Pfn2);
//             MiDecrementShareCount(Pfn1, PFN_FROM_PTE(&PteContents));
//
//             /* Release the PFN lock */
//             MiReleasePfnLock(OldIrql);
//         }
//         else
//         {
//             /* Windows ASSERT */
//             ASSERT((PteContents.Long == 0) || (PteContents.u.Soft.Prototype == 1));
//
//             /* Check if this is a prototype pointer PTE */
//             if (PteContents.u.Soft.Prototype == 1)
//             {
//                 /* Get the prototype PTE */
//                 ProtoPte = MiProtoPteToPte(&PteContents);
//
//                 /* We don't support anything else atm */
//                 ASSERT(ProtoPte->u.Long == 0);
//             }
//         }
//
//         /* Make the PTE into a zero PTE */
//         PointerPte->u.Long = 0;
//
//         /* Move to the next PTE */
//         PointerPte++;
//         NumberOfPtes--;
//     }
//
//     /* Flush the TLB */
//     KeFlushCurrentTb();
//
//     /* Acquire the PFN lock */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Decrement the accounting counters */
//     ControlArea->NumberOfUserReferences--;
//     ControlArea->NumberOfMappedViews--;
//
//     /* Check if we should destroy the CA and release the lock */
//     MiCheckControlArea(ControlArea, OldIrql);
// }
//
// static
// ULONG
// MiRemoveFromSystemSpace(IN PMMSESSION Session,
//                         IN PVOID Base,
//                         OUT PCONTROL_AREA *ControlArea)
// {
//     ULONG Hash, Size, Count = 0;
//     ULONG_PTR Entry;
//     PAGED_CODE();
//
//     /* Compute the hash for this entry and loop trying to find it */
//     Entry = (ULONG_PTR)Base >> 16;
//     Hash = Entry % Session->SystemSpaceHashKey;
//     while ((Session->SystemSpaceViewTable[Hash].Entry >> 16) != Entry)
//     {
//         /* Check if we overflew past the end of the hash table */
//         if (++Hash >= Session->SystemSpaceHashSize)
//         {
//             /* Reset the hash to zero and keep searching from the bottom */
//             Hash = 0;
//             if (++Count == 2)
//             {
//                 /* But if we overflew twice, then this is not a real mapping */
//                 KeBugCheckEx(DRIVER_UNMAPPING_INVALID_VIEW,
//                              (ULONG_PTR)Base,
//                              1,
//                              0,
//                              0);
//             }
//         }
//     }
//
//     /* One less entry */
//     Session->SystemSpaceHashEntries--;
//
//     /* Extract the size and clear the entry */
//     Size = Session->SystemSpaceViewTable[Hash].Entry & 0xFFFF;
//     Session->SystemSpaceViewTable[Hash].Entry = 0;
//
//     /* Return the control area and the size */
//     *ControlArea = Session->SystemSpaceViewTable[Hash].ControlArea;
//     return Size;
// }
//
// static
// NTSTATUS
// MiUnmapViewInSystemSpace(IN PMMSESSION Session,
//                          IN PVOID MappedBase)
// {
//     ULONG Size;
//     PCONTROL_AREA ControlArea;
//     PAGED_CODE();
//
//     /* Remove this mapping */
//     KeAcquireGuardedMutex(Session->SystemSpaceViewLockPointer);
//     Size = MiRemoveFromSystemSpace(Session, MappedBase, &ControlArea);
//
//     /* Clear the bits for this mapping */
//     RtlClearBits(Session->SystemSpaceBitMap,
//                  (ULONG)(((ULONG_PTR)MappedBase - (ULONG_PTR)Session->SystemSpaceViewStart) >> 16),
//                  Size);
//
//     /* Convert the size from a bit size into the actual size */
//     Size = Size * (_64K >> PAGE_SHIFT);
//
//     /* Remove the PTEs now */
//     MiRemoveMappedPtes(MappedBase, Size, ControlArea, NULL);
//     KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmCreateArm3Section(OUT PVOID *SectionObject,
//                     IN ACCESS_MASK DesiredAccess,
//                     IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                     IN PLARGE_INTEGER InputMaximumSize,
//                     IN ULONG SectionPageProtection,
//                     IN ULONG AllocationAttributes,
//                     IN HANDLE FileHandle OPTIONAL,
//                     IN PFILE_OBJECT FileObject OPTIONAL)
// {
//     SECTION Section;
//     PSECTION NewSection;
//     PSUBSECTION Subsection;
//     PSEGMENT NewSegment, Segment;
//     NTSTATUS Status;
//     PCONTROL_AREA ControlArea;
//     ULONG ProtectionMask, ControlAreaSize, Size, NonPagedCharge, PagedCharge;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     BOOLEAN FileLock = FALSE, KernelCall = FALSE;
//     KIRQL OldIrql;
//     PFILE_OBJECT File;
//     BOOLEAN UserRefIncremented = FALSE;
//     PVOID PreviousSectionPointer;
//
//     /* Make the same sanity checks that the Nt interface should've validated */
//     ASSERT((AllocationAttributes & ~(SEC_COMMIT | SEC_RESERVE | SEC_BASED |
//                                      SEC_LARGE_PAGES | SEC_IMAGE | SEC_NOCACHE |
//                                      SEC_NO_CHANGE)) == 0);
//     ASSERT((AllocationAttributes & (SEC_COMMIT | SEC_RESERVE | SEC_IMAGE)) != 0);
//     ASSERT(!((AllocationAttributes & SEC_IMAGE) &&
//              (AllocationAttributes & (SEC_COMMIT | SEC_RESERVE |
//                                       SEC_NOCACHE | SEC_NO_CHANGE))));
//     ASSERT(!((AllocationAttributes & SEC_COMMIT) && (AllocationAttributes & SEC_RESERVE)));
//     ASSERT(!((SectionPageProtection & PAGE_NOCACHE) ||
//              (SectionPageProtection & PAGE_WRITECOMBINE) ||
//              (SectionPageProtection & PAGE_GUARD) ||
//              (SectionPageProtection & PAGE_NOACCESS)));
//
//     /* Convert section flag to page flag */
//     if (AllocationAttributes & SEC_NOCACHE)
//         SectionPageProtection |= PAGE_NOCACHE;
//
//     /* Check to make sure the protection is correct. Nt* does this already */
//     ProtectionMask = MiMakeProtectionMask(SectionPageProtection);
//     if (ProtectionMask == MM_INVALID_PROTECTION)
//     {
//         DPRINT1("Invalid protection mask\n");
//         return STATUS_INVALID_PAGE_PROTECTION;
//     }
//
//     /* Check if this is going to be a data or image backed file section */
//     if ((FileHandle) || (FileObject))
//     {
//         /* These cannot be mapped with large pages */
//         if (AllocationAttributes & SEC_LARGE_PAGES) return STATUS_INVALID_PARAMETER_6;
//
//         /* For now, only support the mechanism through a file handle */
//         ASSERT(FileObject == NULL);
//
//         /* Reference the file handle to get the object */
//         Status = ObReferenceObjectByHandle(FileHandle,
//                                            MmMakeFileAccess[ProtectionMask],
//                                            IoFileObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&File,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Make sure Cc has been doing its job */
//         if (!File->SectionObjectPointer)
//         {
//             /* This is not a valid file system-based file, fail */
//             ObDereferenceObject(File);
//             return STATUS_INVALID_FILE_FOR_SECTION;
//         }
//
//         /* Image-file backed sections are not yet supported */
//         ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//
//         /* Compute the size of the control area, and allocate it */
//         ControlAreaSize = sizeof(CONTROL_AREA) + sizeof(MSUBSECTION);
//         ControlArea = ExAllocatePoolWithTag(NonPagedPool, ControlAreaSize, 'aCmM');
//         if (!ControlArea)
//         {
//             ObDereferenceObject(File);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Zero it out */
//         RtlZeroMemory(ControlArea, ControlAreaSize);
//
//         /* Did we get a handle, or an object? */
//         if (FileHandle)
//         {
//             /* We got a file handle so we have to lock down the file */
// #if 0
//             Status = FsRtlAcquireToCreateMappedSection(File, SectionPageProtection);
//             if (!NT_SUCCESS(Status))
//             {
//                 ExFreePool(ControlArea);
//                 ObDereferenceObject(File);
//                 return Status;
//             }
// #else
//             /* ReactOS doesn't support this API yet, so do nothing */
//             UNIMPLEMENTED;
//             Status = STATUS_SUCCESS;
// #endif
//             /* Update the top-level IRP so that drivers know what's happening */
//             IoSetTopLevelIrp((PIRP)FSRTL_FSP_TOP_LEVEL_IRP);
//             FileLock = TRUE;
//         }
//
//         /* Lock the PFN database while we play with the section pointers */
//         OldIrql = MiAcquirePfnLock();
//
//         /* Image-file backed sections are not yet supported */
//         ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//
//         /* There should not already be a control area for this file */
//         ASSERT(File->SectionObjectPointer->DataSectionObject == NULL);
//         NewSegment = NULL;
//
//         /* Write down that this CA is being created, and set it */
//         ControlArea->u.Flags.BeingCreated = TRUE;
//         ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//         PreviousSectionPointer = File->SectionObjectPointer;
//         File->SectionObjectPointer->DataSectionObject = ControlArea;
//
//         /* We can release the PFN lock now */
//         MiReleasePfnLock(OldIrql);
//
//         /* We don't support previously-mapped file */
//         ASSERT(NewSegment == NULL);
//
//         /* Image-file backed sections are not yet supported */
//         ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//
//         /* So we always create a data file map */
//         Status = MiCreateDataFileMap(File,
//                                      &Segment,
//                                      InputMaximumSize,
//                                      SectionPageProtection,
//                                      AllocationAttributes,
//                                      KernelCall);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Lock the PFN database while we play with the section pointers */
//             OldIrql = MiAcquirePfnLock();
//
//             /* Reset the waiting-for-deletion event */
//             ASSERT(ControlArea->WaitingForDeletion == NULL);
//             ControlArea->WaitingForDeletion = NULL;
//
//             /* Set the file pointer NULL flag */
//             ASSERT(ControlArea->u.Flags.FilePointerNull == 0);
//             ControlArea->u.Flags.FilePointerNull = TRUE;
//
//             /* Delete the data section object */
//             ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//             File->SectionObjectPointer->DataSectionObject = NULL;
//
//             /* No longer being created */
//             ControlArea->u.Flags.BeingCreated = FALSE;
//
//             /* We can release the PFN lock now */
//             MiReleasePfnLock(OldIrql);
//
//             /* Check if we locked and set the IRP */
//             if (FileLock)
//             {
//                 /* Undo */
//                 IoSetTopLevelIrp(NULL);
//                 //FsRtlReleaseFile(File);
//             }
//
//             /* Free the control area and de-ref the file object */
//             ExFreePool(ControlArea);
//             ObDereferenceObject(File);
//
//             /* All done */
//             return Status;
//         }
//
//         /* On success, we expect this */
//         ASSERT(PreviousSectionPointer == File->SectionObjectPointer);
//
//         /* Check if a maximum size was specified */
//         if (!InputMaximumSize->QuadPart)
//         {
//             /* Nope, use the segment size */
//             Section.SizeOfSection.QuadPart = (LONGLONG)Segment->SizeOfSegment;
//         }
//         else
//         {
//             /* Yep, use the entered size */
//             Section.SizeOfSection.QuadPart = InputMaximumSize->QuadPart;
//         }
//     }
//     else
//     {
//         /* A handle must be supplied with SEC_IMAGE, as this is the no-handle path */
//         if (AllocationAttributes & SEC_IMAGE) return STATUS_INVALID_FILE_FOR_SECTION;
//
//         /* Not yet supported */
//         ASSERT((AllocationAttributes & SEC_LARGE_PAGES) == 0);
//
//         /* So this must be a pagefile-backed section, create the mappings needed */
//         Status = MiCreatePagingFileMap(&NewSegment,
//                                        InputMaximumSize->QuadPart,
//                                        ProtectionMask,
//                                        AllocationAttributes);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Set the size here, and read the control area */
//         Section.SizeOfSection.QuadPart = NewSegment->SizeOfSegment;
//         ControlArea = NewSegment->ControlArea;
//
//         /* MiCreatePagingFileMap increments user references */
//         UserRefIncremented = TRUE;
//     }
//
//     /* Did we already have a segment? */
//     if (!NewSegment)
//     {
//         /* This must be the file path and we created a segment */
//         NewSegment = Segment;
//         ASSERT(File != NULL);
//
//         /* Acquire the PFN lock while we set control area flags */
//         OldIrql = MiAcquirePfnLock();
//
//         /* We don't support this race condition yet, so assume no waiters */
//         ASSERT(ControlArea->WaitingForDeletion == NULL);
//         ControlArea->WaitingForDeletion = NULL;
//
//         /* Image-file backed sections are not yet supported, nor ROM images */
//         ASSERT((AllocationAttributes & SEC_IMAGE) == 0);
//         ASSERT(Segment->ControlArea->u.Flags.Rom == 0);
//
//         /* Take off the being created flag, and then release the lock */
//         ControlArea->u.Flags.BeingCreated = FALSE;
//         MiReleasePfnLock(OldIrql);
//     }
//
//     /* Check if we locked the file earlier */
//     if (FileLock)
//     {
//         /* Reset the top-level IRP and release the lock */
//         IoSetTopLevelIrp(NULL);
//         //FsRtlReleaseFile(File);
//         FileLock = FALSE;
//     }
//
//     /* Set the initial section object data */
//     Section.InitialPageProtection = SectionPageProtection;
//
//     /* The mapping created a control area and segment, save the flags */
//     Section.Segment = NewSegment;
//     Section.LongFlags = ControlArea->u.LongFlags;
//
//     /* Check if this is a user-mode read-write non-image file mapping */
//     if (!(FileObject) &&
//         (SectionPageProtection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) &&
//         !(ControlArea->u.Flags.Image) &&
//         (ControlArea->FilePointer))
//     {
//         /* Add a reference and set the flag */
//         Section.u.Flags.UserWritable = TRUE;
//         InterlockedIncrement((volatile LONG*)&ControlArea->WritableUserReferences);
//     }
//
//     /* Check for image mappings or page file mappings */
//     if ((ControlArea->u.Flags.Image) || !(ControlArea->FilePointer))
//     {
//         /* Charge the segment size, and allocate a subsection */
//         PagedCharge = sizeof(SECTION) + NewSegment->TotalNumberOfPtes * sizeof(MMPTE);
//         Size = sizeof(SUBSECTION);
//     }
//     else
//     {
//         /* Charge nothing, and allocate a mapped subsection */
//         PagedCharge = 0;
//         Size = sizeof(MSUBSECTION);
//     }
//
//     /* Check if this is a normal CA */
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//
//     /* Charge only a CA, and the subsection is right after */
//     NonPagedCharge = sizeof(CONTROL_AREA);
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//
//     /* We only support single-subsection mappings */
//     NonPagedCharge += Size;
//     ASSERT(Subsection->NextSubsection == NULL);
//
//     /* Create the actual section object, with enough space for the prototype PTEs */
//     Status = ObCreateObject(PreviousMode,
//                             MmSectionObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(SECTION),
//                             PagedCharge,
//                             NonPagedCharge,
//                             (PVOID*)&NewSection);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Check if this is a user-mode read-write non-image file mapping */
//         if (!(FileObject) &&
//             (SectionPageProtection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)) &&
//             !(ControlArea->u.Flags.Image) &&
//             (ControlArea->FilePointer))
//         {
//             /* Remove a reference and check the flag */
//             ASSERT(Section.u.Flags.UserWritable == 1);
//             InterlockedDecrement((volatile LONG*)&ControlArea->WritableUserReferences);
//         }
//
//         /* Check if a user reference was added */
//         if (UserRefIncremented)
//         {
//             /* Acquire the PFN lock while we change counters */
//             OldIrql = MiAcquirePfnLock();
//
//             /* Decrement the accounting counters */
//             ControlArea->NumberOfSectionReferences--;
//             ASSERT((LONG)ControlArea->NumberOfUserReferences > 0);
//             ControlArea->NumberOfUserReferences--;
//
//             /* Check if we should destroy the CA and release the lock */
//             MiCheckControlArea(ControlArea, OldIrql);
//         }
//
//         /* Return the failure code */
//         return Status;
//     }
//
//     /* NOTE: Past this point, all failures will be handled by Ob upon ref->0 */
//
//     /* Now copy the local section object from the stack into this new object */
//     RtlCopyMemory(NewSection, &Section, sizeof(SECTION));
//     NewSection->Address.StartingVpn = 0;
//
//     /* For now, only user calls are supported */
//     ASSERT(KernelCall == FALSE);
//     NewSection->u.Flags.UserReference = TRUE;
//
//     /* Is this a "based" allocation, in which all mappings are identical? */
//     if (AllocationAttributes & SEC_BASED)
//     {
//         /* Lock the VAD tree during the search */
//         KeAcquireGuardedMutex(&MmSectionBasedMutex);
//
//         /* Is it a brand new ControArea ? */
//         if (ControlArea->u.Flags.BeingCreated == 1)
//         {
//             ASSERT(ControlArea->u.Flags.Based == 1);
//             /* Then we must find a global address, top-down */
//             Status = MiFindEmptyAddressRangeDownBasedTree((SIZE_T)ControlArea->Segment->SizeOfSegment,
//                                                           (ULONG_PTR)MmHighSectionBase,
//                                                           _64K,
//                                                           &MmSectionBasedRoot,
//                                                           (ULONG_PTR*)&ControlArea->Segment->BasedAddress);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 /* No way to find a valid range. */
//                 KeReleaseGuardedMutex(&MmSectionBasedMutex);
//                 ControlArea->u.Flags.Based = 0;
//                 NewSection->u.Flags.Based = 0;
//                 ObDereferenceObject(NewSection);
//                 return Status;
//             }
//
//             /* Compute the ending address and insert it into the VAD tree */
//             NewSection->Address.StartingVpn = (ULONG_PTR)ControlArea->Segment->BasedAddress;
//             NewSection->Address.EndingVpn = NewSection->Address.StartingVpn + NewSection->SizeOfSection.LowPart - 1;
//             MiInsertBasedSection(NewSection);
//         }
//         else
//         {
//             /* FIXME : Should we deny section creation if SEC_BASED is not set ? Can we have two different section objects on the same based address ? Investigate !*/
//             ASSERT(FALSE);
//         }
//
//         KeReleaseGuardedMutex(&MmSectionBasedMutex);
//     }
//
//     /* The control area is not being created anymore */
//     if (ControlArea->u.Flags.BeingCreated == 1)
//     {
//         /* Acquire the PFN lock while we set control area flags */
//         OldIrql = MiAcquirePfnLock();
//
//         /* Take off the being created flag, and then release the lock */
//         ControlArea->u.Flags.BeingCreated = 0;
//         NewSection->u.Flags.BeingCreated = 0;
//
//         MiReleasePfnLock(OldIrql);
//     }
//
//     /* Migrate the attribute into a flag */
//     if (AllocationAttributes & SEC_NO_CHANGE) NewSection->u.Flags.NoChange = TRUE;
//
//     /* If R/W access is not requested, this might eventually become a CoW mapping */
//     if (!(SectionPageProtection & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE)))
//     {
//         NewSection->u.Flags.CopyOnWrite = TRUE;
//     }
//
//     /* Write down if this was a kernel call */
//     ControlArea->u.Flags.WasPurged |= KernelCall;
//     ASSERT(ControlArea->u.Flags.WasPurged == FALSE);
//
//     /* Make sure the segment and the section are the same size, or the section is smaller */
//     ASSERT((ULONG64)NewSection->SizeOfSection.QuadPart <= NewSection->Segment->SizeOfSegment);
//
//     /* Return the object and the creation status */
//     *SectionObject = (PVOID)NewSection;
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmMapViewOfArm3Section(IN PVOID SectionObject,
//                        IN PEPROCESS Process,
//                        IN OUT PVOID *BaseAddress,
//                        IN ULONG_PTR ZeroBits,
//                        IN SIZE_T CommitSize,
//                        IN OUT PLARGE_INTEGER SectionOffset OPTIONAL,
//                        IN OUT PSIZE_T ViewSize,
//                        IN SECTION_INHERIT InheritDisposition,
//                        IN ULONG AllocationType,
//                        IN ULONG Protect)
// {
//     KAPC_STATE ApcState;
//     BOOLEAN Attached = FALSE;
//     PSECTION Section;
//     PCONTROL_AREA ControlArea;
//     ULONG ProtectionMask;
//     NTSTATUS Status;
//     ULONG64 CalculatedViewSize;
//     PAGED_CODE();
//
//     /* Get the segment and control area */
//     Section = (PSECTION)SectionObject;
//     ControlArea = Section->Segment->ControlArea;
//
//     /* These flags/states are not yet supported by ARM3 */
//     ASSERT(Section->u.Flags.Image == 0);
//     ASSERT(Section->u.Flags.NoCache == 0);
//     ASSERT(Section->u.Flags.WriteCombined == 0);
//     ASSERT(ControlArea->u.Flags.PhysicalMemory == 0);
//
//     /* FIXME */
//     if ((AllocationType & MEM_RESERVE) != 0)
//     {
//         DPRINT1("MmMapViewOfArm3Section called with MEM_RESERVE, this is not implemented yet!!!\n");
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* Check if the mapping protection is compatible with the create */
//     if (!MiIsProtectionCompatible(Section->InitialPageProtection, Protect))
//     {
//         DPRINT1("Mapping protection is incompatible\n");
//         return STATUS_SECTION_PROTECTION;
//     }
//
//     /* Check if the offset and size would cause an overflow */
//     if (((ULONG64)SectionOffset->QuadPart + *ViewSize) <
//          (ULONG64)SectionOffset->QuadPart)
//     {
//         DPRINT1("Section offset overflows\n");
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Check if the offset and size are bigger than the section itself */
//     if (((ULONG64)SectionOffset->QuadPart + *ViewSize) >
//          (ULONG64)Section->SizeOfSection.QuadPart)
//     {
//         DPRINT1("Section offset is larger than section\n");
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Check if the caller did not specify a view size */
//     if (!(*ViewSize))
//     {
//         /* Compute it for the caller */
//         CalculatedViewSize = Section->SizeOfSection.QuadPart -
//                              SectionOffset->QuadPart;
//
//         /* Check if it's larger than 4GB or overflows into kernel-mode */
//         if (!NT_SUCCESS(RtlULongLongToSIZET(CalculatedViewSize, ViewSize)) ||
//             (((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS - (ULONG_PTR)*BaseAddress) < CalculatedViewSize))
//         {
//             DPRINT1("Section view won't fit\n");
//             return STATUS_INVALID_VIEW_SIZE;
//         }
//     }
//
//     /* Check if the commit size is larger than the view size */
//     if (CommitSize > *ViewSize)
//     {
//         DPRINT1("Attempting to commit more than the view itself\n");
//         return STATUS_INVALID_PARAMETER_5;
//     }
//
//     /* Check if the view size is larger than the section */
//     if (*ViewSize > (ULONG64)Section->SizeOfSection.QuadPart)
//     {
//         DPRINT1("The view is larger than the section\n");
//         return STATUS_INVALID_VIEW_SIZE;
//     }
//
//     /* Compute and validate the protection mask */
//     ProtectionMask = MiMakeProtectionMask(Protect);
//     if (ProtectionMask == MM_INVALID_PROTECTION)
//     {
//         DPRINT1("The protection is invalid\n");
//         return STATUS_INVALID_PAGE_PROTECTION;
//     }
//
//     /* We only handle pagefile-backed sections, which cannot be writecombined */
//     if (Protect & PAGE_WRITECOMBINE)
//     {
//         DPRINT1("Cannot write combine a pagefile-backed section\n");
//         return STATUS_INVALID_PARAMETER_10;
//     }
//
//     /* Start by attaching to the current process if needed */
//     if (PsGetCurrentProcess() != Process)
//     {
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//         Attached = TRUE;
//     }
//
//     /* Do the actual mapping */
//     Status = MiMapViewOfDataSection(ControlArea,
//                                     Process,
//                                     BaseAddress,
//                                     SectionOffset,
//                                     ViewSize,
//                                     Section,
//                                     InheritDisposition,
//                                     ProtectionMask,
//                                     CommitSize,
//                                     ZeroBits,
//                                     AllocationType);
//
//     /* Detach if needed, then return status */
//     if (Attached) KeUnstackDetachProcess(&ApcState);
//     return Status;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// MmDisableModifiedWriteOfSection(IN PSECTION_OBJECT_POINTERS SectionObjectPointer)
// {
//    UNIMPLEMENTED;
//    return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// MmForceSectionClosed(IN PSECTION_OBJECT_POINTERS SectionObjectPointer,
//                      IN BOOLEAN DelayClose)
// {
//    UNIMPLEMENTED;
//    return FALSE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmMapViewInSessionSpace(IN PVOID Section,
//                         OUT PVOID *MappedBase,
//                         IN OUT PSIZE_T ViewSize)
// {
//     PAGED_CODE();
//     LARGE_INTEGER SectionOffset;
//
//     // HACK
//     if (MiIsRosSectionObject(Section))
//     {
//         return MmMapViewInSystemSpace(Section, MappedBase, ViewSize);
//     }
//
//     /* Process must be in a session */
//     if (PsGetCurrentProcess()->ProcessInSession == FALSE)
//     {
//         DPRINT1("Process is not in session\n");
//         return STATUS_NOT_MAPPED_VIEW;
//     }
//
//     /* Use the system space API, but with the session view instead */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == TRUE);
//     SectionOffset.QuadPart = 0;
//     return MiMapViewInSystemSpace(Section,
//                                   &MmSessionSpace->Session,
//                                   MappedBase,
//                                   ViewSize,
//                                   &SectionOffset);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmUnmapViewInSessionSpace(IN PVOID MappedBase)
// {
//     PAGED_CODE();
//
//     // HACK
//     if (!MI_IS_SESSION_ADDRESS(MappedBase))
//     {
//         return MmUnmapViewInSystemSpace(MappedBase);
//     }
//
//     /* Process must be in a session */
//     if (PsGetCurrentProcess()->ProcessInSession == FALSE)
//     {
//         DPRINT1("Proess is not in session\n");
//         return STATUS_NOT_MAPPED_VIEW;
//     }
//
//     /* Use the system space API, but with the session view instead */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == TRUE);
//     return MiUnmapViewInSystemSpace(&MmSessionSpace->Session,
//                                     MappedBase);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmUnmapViewOfSection(IN PEPROCESS Process,
//                      IN PVOID BaseAddress)
// {
//     return MiUnmapViewOfSection(Process, BaseAddress, 0);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmUnmapViewInSystemSpace(IN PVOID MappedBase)
// {
//     PMEMORY_AREA MemoryArea;
//     PAGED_CODE();
//
//     /* Was this mapped by RosMm? */
//     MmLockAddressSpace(MmGetKernelAddressSpace());
//     MemoryArea = MmLocateMemoryAreaByAddress(MmGetKernelAddressSpace(), MappedBase);
//     if ((MemoryArea) && (MemoryArea->Type != MEMORY_AREA_OWNED_BY_ARM3))
//     {
//         NTSTATUS Status = MiRosUnmapViewInSystemSpace(MappedBase);
//         MmUnlockAddressSpace(MmGetKernelAddressSpace());
//         return Status;
//     }
//     MmUnlockAddressSpace(MmGetKernelAddressSpace());
//
//     /* It was not, call the ARM3 routine */
//     return MiUnmapViewInSystemSpace(&MmSession, MappedBase);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MmCommitSessionMappedView(IN PVOID MappedBase,
//                           IN SIZE_T ViewSize)
// {
//     ULONG_PTR StartAddress, EndingAddress, Base;
//     ULONG Hash, Count = 0, Size, QuotaCharge;
//     PMMSESSION Session;
//     PMMPTE LastProtoPte, PointerPte, ProtoPte;
//     PCONTROL_AREA ControlArea;
//     PSEGMENT Segment;
//     PSUBSECTION Subsection;
//     MMPTE TempPte;
//     PAGED_CODE();
//
//     /* Make sure the base isn't past the session view range */
//     if ((MappedBase < MiSessionViewStart) ||
//         (MappedBase >= (PVOID)((ULONG_PTR)MiSessionViewStart + MmSessionViewSize)))
//     {
//         DPRINT1("Base outside of valid range\n");
//         return STATUS_INVALID_PARAMETER_1;
//     }
//
//     /* Make sure the size isn't past the session view range */
//     if (((ULONG_PTR)MiSessionViewStart + MmSessionViewSize -
//         (ULONG_PTR)MappedBase) < ViewSize)
//     {
//         DPRINT1("Size outside of valid range\n");
//         return STATUS_INVALID_PARAMETER_2;
//     }
//
//     /* Sanity check */
//     ASSERT(ViewSize != 0);
//
//     /* Process must be in a session */
//     if (PsGetCurrentProcess()->ProcessInSession == FALSE)
//     {
//         DPRINT1("Process is not in session\n");
//         return STATUS_NOT_MAPPED_VIEW;
//     }
//
//     /* Compute the correctly aligned base and end addresses */
//     StartAddress = (ULONG_PTR)PAGE_ALIGN(MappedBase);
//     EndingAddress = ((ULONG_PTR)MappedBase + ViewSize - 1) | (PAGE_SIZE - 1);
//
//     /* Sanity check and grab the session */
//     ASSERT(MmIsAddressValid(MmSessionSpace) == TRUE);
//     Session = &MmSessionSpace->Session;
//
//     /* Get the hash entry for this allocation */
//     Hash = (StartAddress >> 16) % Session->SystemSpaceHashKey;
//
//     /* Lock system space */
//     KeAcquireGuardedMutex(Session->SystemSpaceViewLockPointer);
//
//     /* Loop twice so we can try rolling over if needed */
//     while (TRUE)
//     {
//         /* Extract the size and base addresses from the entry */
//         Base = Session->SystemSpaceViewTable[Hash].Entry & ~0xFFFF;
//         Size = Session->SystemSpaceViewTable[Hash].Entry & 0xFFFF;
//
//         /* Convert the size to bucket chunks */
//         Size *= MI_SYSTEM_VIEW_BUCKET_SIZE;
//
//         /* Bail out if this entry fits in here */
//         if ((StartAddress >= Base) && (EndingAddress < (Base + Size))) break;
//
//         /* Check if we overflew past the end of the hash table */
//         if (++Hash >= Session->SystemSpaceHashSize)
//         {
//             /* Reset the hash to zero and keep searching from the bottom */
//             Hash = 0;
//             if (++Count == 2)
//             {
//                 /* But if we overflew twice, then this is not a real mapping */
//                 KeBugCheckEx(DRIVER_UNMAPPING_INVALID_VIEW,
//                              Base,
//                              2,
//                              0,
//                              0);
//             }
//         }
//     }
//
//     /* Make sure the view being mapped is not file-based */
//     ControlArea = Session->SystemSpaceViewTable[Hash].ControlArea;
//     if (ControlArea->FilePointer != NULL)
//     {
//         /* It is, so we have to bail out */
//         DPRINT1("Only page-filed backed sections can be commited\n");
//         KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//         return STATUS_ALREADY_COMMITTED;
//     }
//
//     /* Get the subsection. We don't support LARGE_CONTROL_AREA in ARM3 */
//     ASSERT(ControlArea->u.Flags.GlobalOnlyPerSession == 0);
//     ASSERT(ControlArea->u.Flags.Rom == 0);
//     Subsection = (PSUBSECTION)(ControlArea + 1);
//
//     /* Get the start and end PTEs -- make sure the end PTE isn't past the end */
//     ProtoPte = Subsection->SubsectionBase + ((StartAddress - Base) >> PAGE_SHIFT);
//     QuotaCharge = MiAddressToPte(EndingAddress) - MiAddressToPte(StartAddress) + 1;
//     LastProtoPte = ProtoPte + QuotaCharge;
//     if (LastProtoPte >= Subsection->SubsectionBase + Subsection->PtesInSubsection)
//     {
//         DPRINT1("PTE is out of bounds\n");
//         KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//         return STATUS_INVALID_PARAMETER_2;
//     }
//
//     /* Acquire the commit lock and count all the non-committed PTEs */
//     KeAcquireGuardedMutexUnsafe(&MmSectionCommitMutex);
//     PointerPte = ProtoPte;
//     while (PointerPte < LastProtoPte)
//     {
//         if (PointerPte->u.Long) QuotaCharge--;
//         PointerPte++;
//     }
//
//     /* Was everything committed already? */
//     if (!QuotaCharge)
//     {
//         /* Nothing to do! */
//         KeReleaseGuardedMutexUnsafe(&MmSectionCommitMutex);
//         KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//         return STATUS_SUCCESS;
//     }
//
//     /* Pick the segment and template PTE */
//     Segment = ControlArea->Segment;
//     TempPte = Segment->SegmentPteTemplate;
//     ASSERT(TempPte.Long != 0);
//
//     /* Loop all prototype PTEs to be committed */
//     PointerPte = ProtoPte;
//     while (PointerPte < LastProtoPte)
//     {
//         /* Make sure the PTE is already invalid */
//         if (PointerPte->u.Long == 0)
//         {
//             /* And write the invalid PTE */
//             MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//         }
//
//         /* Move to the next PTE */
//         PointerPte++;
//     }
//
//     /* Check if we had at least one page charged */
//     if (QuotaCharge)
//     {
//         /* Update the accounting data */
//         Segment->NumberOfCommittedPages += QuotaCharge;
//         InterlockedExchangeAddSizeT(&MmSharedCommit, QuotaCharge);
//     }
//
//     /* Release all */
//     KeReleaseGuardedMutexUnsafe(&MmSectionCommitMutex);
//     KeReleaseGuardedMutex(Session->SystemSpaceViewLockPointer);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiDeleteARM3Section(PVOID ObjectBody)
// {
//     PSECTION SectionObject;
//     PCONTROL_AREA ControlArea;
//     KIRQL OldIrql;
//
//     SectionObject = (PSECTION)ObjectBody;
//
//     if (SectionObject->u.Flags.Based == 1)
//     {
//         /* Remove the node from the global section address tree */
//         KeAcquireGuardedMutex(&MmSectionBasedMutex);
//         MiRemoveNode(&SectionObject->Address, &MmSectionBasedRoot);
//         KeReleaseGuardedMutex(&MmSectionBasedMutex);
//     }
//
//     /* Lock the PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     ASSERT(SectionObject->Segment);
//     ASSERT(SectionObject->Segment->ControlArea);
//
//     ControlArea = SectionObject->Segment->ControlArea;
//
//     /* Dereference */
//     ControlArea->NumberOfSectionReferences--;
//     ControlArea->NumberOfUserReferences--;
//
//     ASSERT(ControlArea->u.Flags.BeingDeleted == 0);
//
//     /* Check it. It will delete it if there is no more reference to it */
//     MiCheckControlArea(ControlArea, OldIrql);
// }
//
// ULONG
// NTAPI
// MmDoesFileHaveUserWritableReferences(IN PSECTION_OBJECT_POINTERS SectionPointer)
// {
//     UNIMPLEMENTED_ONCE;
//     return 0;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// NTSTATUS
// NTAPI
// NtAreMappedFilesTheSame(IN PVOID File1MappedAsAnImage,
//                         IN PVOID File2MappedAsFile)
// {
//     PVOID AddressSpace;
//     PMMVAD Vad1, Vad2;
//     PFILE_OBJECT FileObject1, FileObject2;
//     NTSTATUS Status;
//
//     /* Lock address space */
//     AddressSpace = MmGetCurrentAddressSpace();
//     MmLockAddressSpace(AddressSpace);
//
//     /* Get the VAD for Address 1 */
//     Vad1 = MiLocateAddress(File1MappedAsAnImage);
//     if (Vad1 == NULL)
//     {
//         /* Fail, the address does not exist */
//         DPRINT1("No VAD at address 1 %p\n", File1MappedAsAnImage);
//         Status = STATUS_INVALID_ADDRESS;
//         goto Exit;
//     }
//
//     /* Get the VAD for Address 2 */
//     Vad2 = MiLocateAddress(File2MappedAsFile);
//     if (Vad2 == NULL)
//     {
//         /* Fail, the address does not exist */
//         DPRINT1("No VAD at address 2 %p\n", File2MappedAsFile);
//         Status = STATUS_INVALID_ADDRESS;
//         goto Exit;
//     }
//
//     /* Get the file object pointer for VAD 1 */
//     FileObject1 = MiGetFileObjectForVad(Vad1);
//     if (FileObject1 == NULL)
//     {
//         DPRINT1("Failed to get file object for Address 1 %p\n", File1MappedAsAnImage);
//         Status = STATUS_CONFLICTING_ADDRESSES;
//         goto Exit;
//     }
//
//     /* Get the file object pointer for VAD 2 */
//     FileObject2 = MiGetFileObjectForVad(Vad2);
//     if (FileObject2 == NULL)
//     {
//         DPRINT1("Failed to get file object for Address 2 %p\n", File2MappedAsFile);
//         Status = STATUS_CONFLICTING_ADDRESSES;
//         goto Exit;
//     }
//
//     /* Make sure Vad1 is an image mapping */
//     if (Vad1->u.VadFlags.VadType != VadImageMap)
//     {
//         DPRINT1("Address 1 (%p) is not an image mapping\n", File1MappedAsAnImage);
//         Status = STATUS_NOT_SAME_DEVICE;
//         goto Exit;
//     }
//
//     /* SectionObjectPointer is equal if the files are equal */
//     if (FileObject1->SectionObjectPointer == FileObject2->SectionObjectPointer)
//     {
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         Status = STATUS_NOT_SAME_DEVICE;
//     }
//
// Exit:
//     /* Unlock address space */
//     MmUnlockAddressSpace(AddressSpace);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateSection(OUT PHANDLE SectionHandle,
//                 IN ACCESS_MASK DesiredAccess,
//                 IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                 IN PLARGE_INTEGER MaximumSize OPTIONAL,
//                 IN ULONG SectionPageProtection OPTIONAL,
//                 IN ULONG AllocationAttributes,
//                 IN HANDLE FileHandle OPTIONAL)
// {
//     LARGE_INTEGER SafeMaximumSize;
//     PVOID SectionObject;
//     HANDLE Handle;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check for non-existing flags */
//     if ((AllocationAttributes & ~(SEC_COMMIT | SEC_RESERVE | SEC_BASED |
//                                   SEC_LARGE_PAGES | SEC_IMAGE | SEC_NOCACHE |
//                                   SEC_NO_CHANGE)))
//     {
//         if (!(AllocationAttributes & 1))
//         {
//             DPRINT1("Bogus allocation attribute: %lx\n", AllocationAttributes);
//             return STATUS_INVALID_PARAMETER_6;
//         }
//     }
//
//     /* Check for no allocation type */
//     if (!(AllocationAttributes & (SEC_COMMIT | SEC_RESERVE | SEC_IMAGE)))
//     {
//         DPRINT1("Missing allocation type in allocation attributes\n");
//         return STATUS_INVALID_PARAMETER_6;
//     }
//
//     /* Check for image allocation with invalid attributes */
//     if ((AllocationAttributes & SEC_IMAGE) &&
//         (AllocationAttributes & (SEC_COMMIT | SEC_RESERVE | SEC_LARGE_PAGES |
//                                  SEC_NOCACHE | SEC_NO_CHANGE)))
//     {
//         DPRINT1("Image allocation with invalid attributes\n");
//         return STATUS_INVALID_PARAMETER_6;
//     }
//
//     /* Check for allocation type is both commit and reserve */
//     if ((AllocationAttributes & SEC_COMMIT) && (AllocationAttributes & SEC_RESERVE))
//     {
//         DPRINT1("Commit and reserve in the same time\n");
//         return STATUS_INVALID_PARAMETER_6;
//     }
//
//     /* Now check for valid protection */
//     if ((SectionPageProtection & PAGE_NOCACHE) ||
//         (SectionPageProtection & PAGE_WRITECOMBINE) ||
//         (SectionPageProtection & PAGE_GUARD) ||
//         (SectionPageProtection & PAGE_NOACCESS))
//     {
//         DPRINT1("Sections don't support these protections\n");
//         return STATUS_INVALID_PAGE_PROTECTION;
//     }
//
//     /* Use a maximum size of zero, if none was specified */
//     SafeMaximumSize.QuadPart = 0;
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Safely check user-mode parameters */
//             if (MaximumSize) SafeMaximumSize = ProbeForReadLargeInteger(MaximumSize);
//             MaximumSize = &SafeMaximumSize;
//             ProbeForWriteHandle(SectionHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else if (!MaximumSize) MaximumSize = &SafeMaximumSize;
//
//     /* Check that MaximumSize is valid if backed by paging file */
//     if ((!FileHandle) && (!MaximumSize->QuadPart))
//         return STATUS_INVALID_PARAMETER_4;
//
//     /* ARM64 SEC_IMAGE trace — disabled for clean console output */
//
//     /* Create the section */
//     Status = MmCreateSection(&SectionObject,
//                              DesiredAccess,
//                              ObjectAttributes,
//                              MaximumSize,
//                              SectionPageProtection,
//                              AllocationAttributes,
//                              FileHandle,
//                              NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* FIXME: Should zero last page for a file mapping */
//
//     /* Now insert the object */
//     Status = ObInsertObject(SectionObject,
//                             NULL,
//                             DesiredAccess,
//                             0,
//                             NULL,
//                             &Handle);
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Return the handle safely */
//             *SectionHandle = Handle;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Nothing here */
//         }
//         _SEH2_END;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtOpenSection(OUT PHANDLE SectionHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PAGED_CODE();
//
//     /* Check for user-mode caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Safely check user-mode parameters */
//             ProbeForWriteHandle(SectionHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Try opening the object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 MmSectionObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &Handle);
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Return the handle safely */
//         *SectionHandle = Handle;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Nothing here */
//     }
//     _SEH2_END;
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtMapViewOfSection(IN HANDLE SectionHandle,
//                    IN HANDLE ProcessHandle,
//                    IN OUT PVOID* BaseAddress,
//                    IN ULONG_PTR ZeroBits,
//                    IN SIZE_T CommitSize,
//                    IN OUT PLARGE_INTEGER SectionOffset OPTIONAL,
//                    IN OUT PSIZE_T ViewSize,
//                    IN SECTION_INHERIT InheritDisposition,
//                    IN ULONG AllocationType,
//                    IN ULONG Protect)
// {
//     PVOID SafeBaseAddress;
//     LARGE_INTEGER SafeSectionOffset;
//     SIZE_T SafeViewSize;
//     PSECTION Section;
//     PEPROCESS Process;
//     NTSTATUS Status;
//     ACCESS_MASK DesiredAccess;
//     ULONG ProtectionMask;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
// #if defined(_M_IX86) || defined(_M_AMD64)
//     static const ULONG ValidAllocationType = (MEM_TOP_DOWN | MEM_LARGE_PAGES |
//             MEM_DOS_LIM | SEC_NO_CHANGE | MEM_RESERVE);
// #else
//     static const ULONG ValidAllocationType = (MEM_TOP_DOWN | MEM_LARGE_PAGES |
//             SEC_NO_CHANGE | MEM_RESERVE);
// #endif
//
//     /* Check for invalid inherit disposition */
//     if ((InheritDisposition > ViewUnmap) || (InheritDisposition < ViewShare))
//     {
//         DPRINT1("Invalid inherit disposition\n");
//         return STATUS_INVALID_PARAMETER_8;
//     }
//
//     /* Allow only valid allocation types */
//     if (AllocationType & ~ValidAllocationType)
//     {
//         DPRINT1("Invalid allocation type\n");
//         return STATUS_INVALID_PARAMETER_9;
//     }
//
//     /* Convert the protection mask, and validate it */
//     ProtectionMask = MiMakeProtectionMask(Protect);
//     if (ProtectionMask == MM_INVALID_PROTECTION)
//     {
//         DPRINT1("Invalid page protection\n");
//         return STATUS_INVALID_PAGE_PROTECTION;
//     }
//
//     /* Now convert the protection mask into desired section access mask */
//     DesiredAccess = MmMakeSectionAccess[ProtectionMask & 0x7];
//
//     /* Assume no section offset */
//     SafeSectionOffset.QuadPart = 0;
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Check for unsafe parameters */
//         if (PreviousMode != KernelMode)
//         {
//             /* Probe the parameters */
//             ProbeForWritePointer(BaseAddress);
//             ProbeForWriteSize_t(ViewSize);
//         }
//
//         /* Check if a section offset was given */
//         if (SectionOffset)
//         {
//             /* Check for unsafe parameters and capture section offset */
//             if (PreviousMode != KernelMode) ProbeForWriteLargeInteger(SectionOffset);
//             SafeSectionOffset = *SectionOffset;
//         }
//
//         /* Capture the other parameters */
//         SafeBaseAddress = *BaseAddress;
//         SafeViewSize = *ViewSize;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Return the exception code */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Check for kernel-mode address */
//     if (SafeBaseAddress > MM_HIGHEST_VAD_ADDRESS)
//     {
//         DPRINT1("Kernel base not allowed\n");
//         return STATUS_INVALID_PARAMETER_3;
//     }
//
//     /* Check for range entering kernel-mode */
//     if (((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS - (ULONG_PTR)SafeBaseAddress) < SafeViewSize)
//     {
//         DPRINT1("Overflowing into kernel base not allowed\n");
//         return STATUS_INVALID_PARAMETER_3;
//     }
//
//     /* Check for invalid zero bits */
//     if (ZeroBits)
//     {
//         if (ZeroBits > MI_MAX_ZERO_BITS)
//         {
//             DPRINT1("Invalid zero bits\n");
//             return STATUS_INVALID_PARAMETER_4;
//         }
//
//         if ((((ULONG_PTR)SafeBaseAddress << ZeroBits) >> ZeroBits) != (ULONG_PTR)SafeBaseAddress)
//         {
//             DPRINT1("Invalid zero bits\n");
//             return STATUS_INVALID_PARAMETER_4;
//         }
//
//         if (((((ULONG_PTR)SafeBaseAddress + SafeViewSize) << ZeroBits) >> ZeroBits) != ((ULONG_PTR)SafeBaseAddress + SafeViewSize))
//         {
//             DPRINT1("Invalid zero bits\n");
//             return STATUS_INVALID_PARAMETER_4;
//         }
//     }
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_VM_OPERATION,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Reference the section */
//     Status = ObReferenceObjectByHandle(SectionHandle,
//                                        DesiredAccess,
//                                        MmSectionObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Section,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ObDereferenceObject(Process);
//         return Status;
//     }
//
//     if (Section->u.Flags.PhysicalMemory)
//     {
//         if (PreviousMode == UserMode &&
//             SafeSectionOffset.QuadPart + SafeViewSize > MmHighestPhysicalPage << PAGE_SHIFT)
//         {
//             DPRINT1("Denying map past highest physical page.\n");
//             ObDereferenceObject(Section);
//             ObDereferenceObject(Process);
//             return STATUS_INVALID_PARAMETER_6;
//         }
//     }
//     else if (!(AllocationType & MEM_DOS_LIM))
//     {
//         /* Check for non-allocation-granularity-aligned BaseAddress */
//         if (SafeBaseAddress != ALIGN_DOWN_POINTER_BY(SafeBaseAddress, MM_VIRTMEM_GRANULARITY))
//         {
//             DPRINT("BaseAddress is not at 64-kilobyte address boundary.\n");
//             ObDereferenceObject(Section);
//             ObDereferenceObject(Process);
//             return STATUS_MAPPED_ALIGNMENT;
//         }
//
//         /* Do the same for the section offset */
//         if (SafeSectionOffset.LowPart != ALIGN_DOWN_BY(SafeSectionOffset.LowPart, MM_VIRTMEM_GRANULARITY))
//         {
//             DPRINT("SectionOffset is not at 64-kilobyte address boundary.\n");
//             ObDereferenceObject(Section);
//             ObDereferenceObject(Process);
//             return STATUS_MAPPED_ALIGNMENT;
//         }
//     }
//
//     /* Now do the actual mapping */
//     Status = MmMapViewOfSection(Section,
//                                 Process,
//                                 &SafeBaseAddress,
//                                 ZeroBits,
//                                 CommitSize,
//                                 &SafeSectionOffset,
//                                 &SafeViewSize,
//                                 InheritDisposition,
//                                 AllocationType,
//                                 Protect);
//
//     /* Return data only on success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if this is an image for the current process */
//         if ((Section->u.Flags.Image) &&
//             (Process == PsGetCurrentProcess()) &&
//             (Status != STATUS_IMAGE_NOT_AT_BASE))
//         {
//             /* Notify the debugger */
//             DbgkMapViewOfSection(Section,
//                                  SafeBaseAddress,
//                                  SafeSectionOffset.LowPart,
//                                  SafeViewSize);
//         }
//
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Return parameters to user */
//             *BaseAddress = SafeBaseAddress;
//             *ViewSize = SafeViewSize;
//             if (SectionOffset) *SectionOffset = SafeSectionOffset;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Nothing to do */
//         }
//         _SEH2_END;
//     }
//
//     /* Dereference all objects and return status */
//     ObDereferenceObject(Section);
//     ObDereferenceObject(Process);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtUnmapViewOfSection(IN HANDLE ProcessHandle,
//                      IN PVOID BaseAddress)
// {
//     PEPROCESS Process;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     /* Don't allowing mapping kernel views */
//     if ((PreviousMode == UserMode) && (BaseAddress > MM_HIGHEST_USER_ADDRESS))
//     {
//         DPRINT1("Trying to unmap a kernel view\n");
//         return STATUS_NOT_MAPPED_VIEW;
//     }
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_VM_OPERATION,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Unmap the view */
//     Status = MiUnmapViewOfSection(Process, BaseAddress, 0);
//
//     /* Dereference the process and return status */
//     ObDereferenceObject(Process);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtExtendSection(IN HANDLE SectionHandle,
//                 IN OUT PLARGE_INTEGER NewMaximumSize)
// {
//     LARGE_INTEGER SafeNewMaximumSize;
//     PSECTION Section;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     /* Check for user-mode parameters */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Probe and capture the maximum size, it's both read and write */
//             ProbeForWriteLargeInteger(NewMaximumSize);
//             SafeNewMaximumSize = *NewMaximumSize;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Just read the size directly */
//         SafeNewMaximumSize = *NewMaximumSize;
//     }
//
//     /* Reference the section */
//     Status = ObReferenceObjectByHandle(SectionHandle,
//                                        SECTION_EXTEND_SIZE,
//                                        MmSectionObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Section,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     Status = MmExtendSection(Section, &SafeNewMaximumSize);
//
//     /* Dereference the section */
//     ObDereferenceObject(Section);
//
//     if (NT_SUCCESS(Status))
//     {
//         _SEH2_TRY
//         {
//             /* Write back the new size */
//             *NewMaximumSize = SafeNewMaximumSize;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return the status */
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<_MEMORY_SECTION_NAME>() == 16);
    assert!(core::mem::offset_of!(_MEMORY_SECTION_NAME, SectionFileName) == 0);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<_MEMORY_SECTION_NAME>() == 8);
    assert!(core::mem::offset_of!(_MEMORY_SECTION_NAME, SectionFileName) == 0);
};
// END OFFSET CHECKS
