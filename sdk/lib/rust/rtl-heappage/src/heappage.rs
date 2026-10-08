use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::InterlockedIncrement;
use ntkrnl_sys::ddk::{
    CLONG, EXCEPTION_RECORD, HEAP_LOCK, LARGE_INTEGER, LIST_ENTRY,
    MEMORY_BASIC_INFORMATION, PHEAP_LOCK, PRTL_AVL_TABLE, PRTL_HEAP_PARAMETERS, PTEB, RtlCreateHeap,
    RtlDeleteElementGenericTableAvl, RtlDestroyHeap,
    RtlEnumerateGenericTableAvl, RtlFreeHeap,
    RtlInitUnicodeString, RtlInitializeGenericTableAvl,
    RtlInsertElementGenericTableAvl, RtlLookupElementGenericTableAvl,
    RtlRaiseException, RTL_AVL_TABLE, RTL_BALANCED_LINKS,
    _RTL_GENERIC_COMPARE_RESULTS, _UNICODE_STRING, WORD,
    PRTL_TRACE_BLOCK, SINGLE_LIST_ENTRY, SLIST_HEADER,
    _MEMORY_INFORMATION_CLASS_MemoryBasicInformation,
    ZwAllocateVirtualMemory, ZwFreeVirtualMemory,
    ZwProtectVirtualMemory, ZwQueryPerformanceCounter,
    ZwQueryVirtualMemory,
};

// ---- missing type aliases ----
pub type RTL_GENERIC_COMPARE_RESULTS = _RTL_GENERIC_COMPARE_RESULTS;
pub type DPH_HEAP_BLOCK = _DPH_HEAP_BLOCK;
pub type DPH_BLOCK_INFORMATION = _DPH_BLOCK_INFORMATION;
pub type DPH_HEAP_ROOT = _DPH_HEAP_ROOT;
// PHEAP: opaque heap pointer (used to access Flags/ForceFlags via offset).
// The _HEAP struct is private to librtl; page-heap only needs the first-page
// Flags + ForceFlags fields which are past HEAP_SEGMENT_MEMBERS.
// Offset of _HEAP.Flags (= sizeof the HEAP_SEGMENT_MEMBERS prefix). Verified vs
// heap.h cross-checked against _HEAP.ProcessHeapsListIndex (128 i386 / 216 amd64):
// i386 = 64 (0x40); amd64/arm64 = 112 (0x70), NOT 96 — a wrong 64-bit offset reads
// ForceFlags from inside UCRSegmentList and breaks page-heap recognition on 64-bit.
#[cfg(target_arch = "x86")]
const HEAP_FLAGS_OFFSET: usize = 64;
#[cfg(not(target_arch = "x86"))]
const HEAP_FLAGS_OFFSET: usize = 112;

#[repr(C)]
pub struct _HEAP_METADATA {
    _pad: [u8; HEAP_FLAGS_OFFSET],
    pub Flags: ULONG,
    pub ForceFlags: ULONG,
}
pub type PHEAP = *mut _HEAP_METADATA;

// ---- Generic compare results (Enum values not directly in ddk) ----
pub const GenericLessThan: RTL_GENERIC_COMPARE_RESULTS = 0;
pub const GenericGreaterThan: RTL_GENERIC_COMPARE_RESULTS = 1;
pub const GenericEqual: RTL_GENERIC_COMPARE_RESULTS = 2;

// MemoryBasicInformation enum alias (use the bindgen constant directly via import)

// ---- Missing heap constants (from sdk/include/ndk/rtltypes.h) ----
pub const HEAP_NO_SERIALIZE: ULONG = 0x00000001;
pub const HEAP_GENERATE_EXCEPTIONS: ULONG = 0x00000004;
pub const HEAP_ZERO_MEMORY: ULONG = 0x00000008;
pub const HEAP_REALLOC_IN_PLACE_ONLY: ULONG = 0x00000010;
pub const HEAP_SETTABLE_USER_FLAGS: ULONG = 0x00000E00;
pub const HEAP_FLAG_PAGE_ALLOCS: ULONG = 0x01000000;
pub const HEAP_NO_ALIGNMENT: ULONG = 0x08000000;

// ---- Missing page protection constants (from sdk/include/xdk/mmtypes.h) ----
pub const PAGE_NOACCESS: ULONG = 0x01;
pub const PAGE_READONLY: ULONG = 0x02;

// ---- Missing APPLICATION_VERIFIER constants (from sdk/include/reactos/verifier.h) ----
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_HEADER: ULONG = 0x000D;
pub const APPLICATION_VERIFIER_DOUBLE_FREE: ULONG = 0x0007;
pub const APPLICATION_VERIFIER_CORRUPTED_FREED_HEAP_BLOCK: ULONG = 0x000E;
pub const APPLICATION_VERIFIER_SWITCHED_HEAP_HANDLE: ULONG = 0x0006;
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_SUFFIX: ULONG = 0x000F;
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_PREFIX: ULONG = 0x0012;
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_START_STAMP: ULONG = 0x0010;
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_END_STAMP: ULONG = 0x0011;
pub const APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_EXCEPTION_RAISED_FOR_HEADER: ULONG = 0x000B;
pub const APPLICATION_VERIFIER_DESTROY_PROCESS_HEAP: ULONG = 0x0009;

// ---- Missing extern declarations ----
extern "system" {
    fn RtlInitializeSListHead(ListHead: *mut SLIST_HEADER);
    fn RtlValidateHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
    fn RtlTryEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN) -> BOOLEAN;
    fn RtlEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN);
    fn RtlLeaveHeapLock(Lock: PHEAP_LOCK);
    fn RtlInitializeHeapLock(Lock: *mut PHEAP_LOCK) -> NTSTATUS;
    fn RtlDeleteHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS;
    fn RtlGetProcessHeap() -> HANDLE;
    fn RtlApplicationVerifierStop(
        Code: ULONG_PTR,
        Message: *const u8,
        Value1: PVOID,
        Description1: *const u8,
        Value2: PVOID,
        Description2: *const u8,
        Value3: PVOID,
        Description3: *const u8,
        Value4: PVOID,
        Description4: *const u8,
    );
    fn AVrfInternalHeapFreeNotification(
        AllocationBase: PVOID,
        AllocationSize: SIZE_T,
    );
    fn RtlFillMemoryUlong(Destination: PVOID, Length: SIZE_T, Pattern: ULONG);
}

// VERIFIER_STOP helper — translates the C variadic macro into a direct call.
unsafe fn VERIFIER_STOP(
    code: ULONG,
    msg: PCHAR,
    val1: PVOID,
    desc1: PCHAR,
    val2: PVOID,
    desc2: PCHAR,
    val3: PVOID,
    desc3: PCHAR,
    val4: PVOID,
    desc4: PCHAR,
) {
    RtlApplicationVerifierStop(
        code as ULONG_PTR,
        msg as *const u8,
        val1,
        desc1 as *const u8,
        val2,
        desc2 as *const u8,
        val3,
        desc3 as *const u8,
        val4,
        desc4 as *const u8,
    );
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _DPH_BLOCK_INFORMATION {
    pub StartStamp: ULONG,
    pub Heap: PVOID,
    pub RequestedSize: SIZE_T,
    pub ActualSize: SIZE_T,
    pub __bindgen_anon_1: _DPH_BLOCK_INFORMATION__bindgen_ty_1,
    pub StackTrace: PVOID,
    pub EndStamp: ULONG,
}
pub type PDPH_BLOCK_INFORMATION = *mut _DPH_BLOCK_INFORMATION;

#[repr(C)]
pub union _DPH_BLOCK_INFORMATION__bindgen_ty_1 {
    pub FreeQueue: LIST_ENTRY,
    pub FreePushList: SINGLE_LIST_ENTRY,
    pub TraceIndex: WORD,
}

#[repr(C)]
pub struct _DPH_HEAP_BLOCK {
    pub __bindgen_anon_1: _DPH_HEAP_BLOCK__bindgen_ty_1,
    pub pUserAllocation: PUCHAR,
    pub pVirtualBlock: PUCHAR,
    pub nVirtualBlockSize: SIZE_T,
    pub nVirtualAccessSize: SIZE_T,
    pub nUserRequestedSize: SIZE_T,
    pub nUserActualSize: SIZE_T,
    pub UserValue: PVOID,
    pub UserFlags: ULONG,
    pub StackTrace: PRTL_TRACE_BLOCK,
    pub AdjacencyEntry: LIST_ENTRY,
    pub pVirtualRegion: PUCHAR,
}
pub type PDPH_HEAP_BLOCK = *mut _DPH_HEAP_BLOCK;

#[repr(C)]
pub union _DPH_HEAP_BLOCK__bindgen_ty_1 {
    pub pNextAlloc: *mut _DPH_HEAP_BLOCK,
    pub AvailableEntry: LIST_ENTRY,
    pub TableLinks: RTL_BALANCED_LINKS,
}

#[repr(C)]
pub struct _DPH_HEAP_ROOT {
    pub Signature: ULONG,
    pub HeapFlags: ULONG,
    pub HeapCritSect: PHEAP_LOCK,
    pub nRemoteLockAcquired: ULONG,

    pub pVirtualStorageListHead: PDPH_HEAP_BLOCK,
    pub pVirtualStorageListTail: PDPH_HEAP_BLOCK,
    pub nVirtualStorageRanges: ULONG,
    pub nVirtualStorageBytes: SIZE_T,

    pub BusyNodesTable: RTL_AVL_TABLE,
    pub NodeToAllocate: PDPH_HEAP_BLOCK,
    pub nBusyAllocations: ULONG,
    pub nBusyAllocationBytesCommitted: SIZE_T,

    pub pFreeAllocationListHead: PDPH_HEAP_BLOCK,
    pub pFreeAllocationListTail: PDPH_HEAP_BLOCK,
    pub nFreeAllocations: ULONG,
    pub nFreeAllocationBytesCommitted: SIZE_T,

    pub AvailableAllocationHead: LIST_ENTRY,
    pub nAvailableAllocations: ULONG,
    pub nAvailableAllocationBytesCommitted: SIZE_T,

    pub pUnusedNodeListHead: PDPH_HEAP_BLOCK,
    pub pUnusedNodeListTail: PDPH_HEAP_BLOCK,
    pub nUnusedNodes: ULONG,
    pub nBusyAllocationBytesAccessible: SIZE_T,
    pub pNodePoolListHead: PDPH_HEAP_BLOCK,
    pub pNodePoolListTail: PDPH_HEAP_BLOCK,
    pub nNodePools: ULONG,
    pub nNodePoolBytes: SIZE_T,

    pub NextHeap: LIST_ENTRY,
    pub ExtraFlags: ULONG,
    pub Seed: ULONG,
    pub NormalHeap: PVOID,
    pub CreateStackTrace: PRTL_TRACE_BLOCK,
    pub FirstThread: PVOID,
}
pub type PDPH_HEAP_ROOT = *mut _DPH_HEAP_ROOT;

// NtCurrentTeb — ddk declares it out-of-line but no TU defines the symbol (the
// old C got it from a zombie thread.c.obj in the stale fat librtl.a); use the
// fs/gs/x18 intrinsic form (same as rtl-sync/src/shared.rs).
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}

#[no_mangle] // cross-crate C name (ntdll ldr, rtl-heap)
pub static mut RtlpPageHeapEnabled: BOOLEAN = 0u8;
#[no_mangle] // cross-crate C name (ntdll ldr)
pub static mut RtlpDphGlobalFlags: ULONG = 0;
pub static mut RtlpPageHeapSizeRangeStart: ULONG = 0;
pub static mut RtlpPageHeapSizeRangeEnd: ULONG = 0;
pub static mut RtlpPageHeapDllRangeStart: ULONG = 0;
pub static mut RtlpPageHeapDllRangeEnd: ULONG = 0;
pub static mut RtlpDphTargetDlls: [WCHAR; 512] = [0; 512];
pub static mut RtlpDphPageHeapList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut RtlpDphPageHeapListInitialized: BOOLEAN = 0u8;
pub static mut _RtlpDphPageHeapListLock: HEAP_LOCK = unsafe { core::mem::zeroed() };
pub static mut RtlpDphPageHeapListLock: PHEAP_LOCK =
    unsafe { core::ptr::addr_of_mut!(_RtlpDphPageHeapListLock) };
pub static mut RtlpDphPageHeapListLength: ULONG = 0;
pub static mut RtlpDphTargetDllsUnicode: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut _RtlpDphDelayedFreeQueueLock: HEAP_LOCK = unsafe { core::mem::zeroed() };
pub static mut RtlpDphDelayedFreeQueueLock: PHEAP_LOCK =
    unsafe { core::ptr::addr_of_mut!(_RtlpDphDelayedFreeQueueLock) };
pub static mut RtlpDphDelayedFreeQueue: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut RtlpDphDelayedTemporaryPushList: SLIST_HEADER = unsafe { core::mem::zeroed() };
pub static mut RtlpDphMemoryUsedByDelayedFreeBlocks: SIZE_T = 0;
pub static mut RtlpDphNumberOfDelayedFreeBlocks: ULONG = 0;
pub static mut RtlpDphCounter: LONG = 0;
pub static mut RtlpDphAllocFails: LONG = 0;
pub static mut RtlpDphReleaseFails: LONG = 0;
pub static mut RtlpDphFreeFails: LONG = 0;
pub static mut RtlpDphProtectFails: LONG = 0;

pub const DPH_RESERVE_SIZE: ULONG = 0x100000;
pub const DPH_POOL_SIZE: ULONG = 0x4000;
pub const DPH_FREE_LIST_MINIMUM: ULONG = 8;
pub const DPH_BREAK_ON_RESERVE_FAIL: ULONG = 0x01;
pub const DPH_BREAK_ON_COMMIT_FAIL: ULONG = 0x02;
pub const DPH_BREAK_ON_RELEASE_FAIL: ULONG = 0x04;
pub const DPH_BREAK_ON_FREE_FAIL: ULONG = 0x08;
pub const DPH_BREAK_ON_PROTECT_FAIL: ULONG = 0x10;
pub const DPH_BREAK_ON_NULL_FREE: ULONG = 0x80;
pub const DPH_DEBUG_INTERNAL_VALIDATE: ULONG = 0x01;
pub const DPH_DEBUG_VERBOSE: ULONG = 0x04;
pub const DPH_EXTRA_LOG_STACK_TRACES: ULONG = 0x02;
pub const DPH_EXTRA_CHECK_UNDERRUN: ULONG = 0x10;
pub const DPH_FILL: ULONG = 0xEEEEEEEE;
pub const DPH_FILL_START_STAMP_1: ULONG = 0xABCDBBBB;
pub const DPH_FILL_START_STAMP_2: ULONG = 0xABCDBBBA;
pub const DPH_FILL_END_STAMP_1: ULONG = 0xDCBABBBB;
pub const DPH_FILL_END_STAMP_2: ULONG = 0xDCBABBBA;
pub const DPH_FILL_SUFFIX: ULONG = 0xD0;
pub const DPH_FILL_INFIX: ULONG = 0xC0;
pub const DPH_VALINFO_BAD_START_STAMP: ULONG = 0x01;
pub const DPH_VALINFO_BAD_END_STAMP: ULONG = 0x02;
pub const DPH_VALINFO_BAD_POINTER: ULONG = 0x04;
pub const DPH_VALINFO_BAD_PREFIX_PATTERN: ULONG = 0x08;
pub const DPH_VALINFO_BAD_SUFFIX_PATTERN: ULONG = 0x10;
pub const DPH_VALINFO_EXCEPTION: ULONG = 0x20;
pub const DPH_VALINFO_1: ULONG = 0x40;
pub const DPH_VALINFO_BAD_INFIX_PATTERN: ULONG = 0x80;
pub const DPH_VALINFO_ALREADY_FREED: ULONG = 0x100;
pub const DPH_VALINFO_CORRUPTED_AFTER_FREE: ULONG = 0x200;
pub const DPH_SIGNATURE: ULONG = 0xFFEEDDCC;

pub static mut RtlpDphBreakOptions: ULONG = 0;
pub static mut RtlpDphDebugOptions: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn RtlpDphRaiseException(Status: NTSTATUS) {
    let mut Exception: EXCEPTION_RECORD = core::mem::zeroed();

    Exception.ExceptionCode = Status;
    Exception.ExceptionAddress = RtlpDphRaiseException as PVOID;
    Exception.ExceptionFlags = 0;
    Exception.ExceptionRecord = core::ptr::null_mut();
    Exception.NumberParameters = 0;

    RtlRaiseException(&mut Exception);
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphPointerFromHandle(Handle: PVOID) -> PVOID {
    let NormalHeap = Handle as PHEAP;
    let DphHeap = (Handle as PUCHAR).add(PAGE_SIZE as usize) as PDPH_HEAP_ROOT;

    if (*NormalHeap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        if (*DphHeap).Signature == DPH_SIGNATURE {
            return DphHeap as PVOID;
        }
    }

    DbgBreakPoint();
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphHeapFromPointer(DphHeap: PDPH_HEAP_ROOT) -> PVOID {
    (DphHeap as PUCHAR).sub(PAGE_SIZE as usize) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphGetBlockSizeFromCorruptedBlock(Block: PVOID) -> ULONG {
    let BlockInfo: PDPH_BLOCK_INFORMATION = (Block as PDPH_BLOCK_INFORMATION).offset(-1);

    if (*BlockInfo).StartStamp != 0xABCDBBBB && (*BlockInfo).StartStamp != 0xABCDBBBA {
        return 0;
    }

    (*BlockInfo).RequestedSize as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphEnterCriticalSection(DphRoot: PDPH_HEAP_ROOT, Flags: ULONG) {
    if Flags & HEAP_NO_SERIALIZE != 0 {
        if RtlTryEnterHeapLock((*DphRoot).HeapCritSect, 1u8) == 0 {
            if (*DphRoot).nRemoteLockAcquired == 0 {
                DbgBreakPoint();

                (*DphRoot).HeapFlags &= !HEAP_NO_SERIALIZE;
            }

            RtlEnterHeapLock((*DphRoot).HeapCritSect, 1u8);
        }
    } else {
        RtlEnterHeapLock((*DphRoot).HeapCritSect, 1u8);
    }
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphLeaveCriticalSection(DphRoot: PDPH_HEAP_ROOT) {
    RtlLeaveHeapLock((*DphRoot).HeapCritSect);
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPreProcessing(DphRoot: PDPH_HEAP_ROOT, Flags: ULONG) {
    RtlpDphEnterCriticalSection(DphRoot, Flags);
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPostProcessing(DphRoot: PDPH_HEAP_ROOT) {
    if DphRoot.is_null() {
        return;
    }

    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
        /* FIXME: Validate integrity, internal lists if necessary */
    }

    RtlpDphLeaveCriticalSection(DphRoot);
}
#[no_mangle]
pub unsafe extern "system" fn RtlpSecMemFreeVirtualMemory(
    Process: HANDLE,
    Base: *mut PVOID,
    Size: PSIZE_T,
    Type: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS = ZwFreeVirtualMemory(Process, Base, Size.cast(), Type);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphAllocateVm(
    Base: *mut PVOID,
    mut Size: SIZE_T,
    Type: ULONG,
    Protection: ULONG,
) -> NTSTATUS {
    let Status = ZwAllocateVirtualMemory(
        (-1isize) as HANDLE,
        Base,
        0,
        core::ptr::addr_of_mut!(Size).cast(),
        Type,
        Protection,
    );

    if !NT_SUCCESS(Status) {
        if Type == MEM_RESERVE {
            InterlockedIncrement(core::ptr::addr_of_mut!(RtlpDphCounter));
            if (RtlpDphBreakOptions & DPH_BREAK_ON_RESERVE_FAIL) != 0 {
                DbgBreakPoint();
                return Status;
            }
        } else {
            InterlockedIncrement(core::ptr::addr_of_mut!(RtlpDphAllocFails));
            if (RtlpDphBreakOptions & DPH_BREAK_ON_COMMIT_FAIL) != 0 {
                DbgBreakPoint();
                return Status;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphFreeVm(
    mut Base: PVOID,
    mut Size: SIZE_T,
    Type: ULONG,
) -> NTSTATUS {
    let Status = RtlpSecMemFreeVirtualMemory(
        -1isize as HANDLE,
        &mut Base,
        &mut Size,
        Type,
    );

    if !NT_SUCCESS(Status) {
        if Type == MEM_RELEASE {
            InterlockedIncrement(core::ptr::addr_of_mut!(RtlpDphReleaseFails));
            if RtlpDphBreakOptions & DPH_BREAK_ON_RELEASE_FAIL != 0 {
                DbgBreakPoint();
                return Status;
            }
        } else {
            InterlockedIncrement(core::ptr::addr_of_mut!(RtlpDphFreeFails));
            if RtlpDphBreakOptions & DPH_BREAK_ON_FREE_FAIL != 0 {
                DbgBreakPoint();
                return Status;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphProtectVm(
    mut Base: PVOID,
    mut Size: SIZE_T,
    Protection: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut OldProtection: ULONG = 0;

    Status = ZwProtectVirtualMemory(
        -1isize as HANDLE,
        &mut Base,
        core::ptr::addr_of_mut!(Size).cast(),
        Protection,
        &mut OldProtection,
    );

    if !NT_SUCCESS(Status) {
        InterlockedIncrement(core::ptr::addr_of_mut!(RtlpDphProtectFails));
        if RtlpDphBreakOptions & DPH_BREAK_ON_PROTECT_FAIL != 0 {
            DbgBreakPoint();
            return Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphWritePageHeapBlockInformation(
    DphRoot: PDPH_HEAP_ROOT,
    UserAllocation: PVOID,
    Size: SIZE_T,
    UserSize: SIZE_T,
) -> BOOLEAN {
    let BlockInfo: PDPH_BLOCK_INFORMATION =
        (UserAllocation as PDPH_BLOCK_INFORMATION).offset(-1);

    (*BlockInfo).Heap = DphRoot as PVOID;
    (*BlockInfo).ActualSize = UserSize;
    (*BlockInfo).RequestedSize = Size;
    (*BlockInfo).StartStamp = DPH_FILL_START_STAMP_1 as ULONG;
    (*BlockInfo).EndStamp = DPH_FILL_END_STAMP_1 as ULONG;

    let FillPtr: PUCHAR = (UserAllocation as PUCHAR).add(Size as usize);
    let len = ROUND_UP(FillPtr as ULONG_PTR, PAGE_SIZE as ULONG_PTR) - (FillPtr as ULONG_PTR);
    core::ptr::write_bytes(FillPtr, DPH_FILL_SUFFIX as u8, len as usize);

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPlaceOnBusyList(DphRoot: PDPH_HEAP_ROOT, DphNode: PDPH_HEAP_BLOCK) {
    let mut NewElement: u8 = 0;

    (*DphRoot).NodeToAllocate = DphNode;
    let _AddressUserData = RtlInsertElementGenericTableAvl(
        core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable),
        core::ptr::addr_of_mut!((*DphNode).pUserAllocation) as *mut core::ffi::c_void,
        core::mem::size_of::<usize>() as u32,
        &mut NewElement,
    );

    (*DphRoot).nBusyAllocations += 1;
    (*DphRoot).nBusyAllocationBytesAccessible += (*DphNode).nVirtualAccessSize;
    (*DphRoot).nBusyAllocationBytesCommitted += (*DphNode).nVirtualBlockSize;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPlaceOnFreeList(DphRoot: PDPH_HEAP_ROOT, Node: PDPH_HEAP_BLOCK) {
    (*Node).__bindgen_anon_1.pNextAlloc = core::ptr::null_mut();

    if !(*DphRoot).pFreeAllocationListTail.is_null() {
        (*(*DphRoot).pFreeAllocationListTail).__bindgen_anon_1.pNextAlloc = Node;
    } else {
        (*DphRoot).pFreeAllocationListHead = Node;
    }
    (*DphRoot).pFreeAllocationListTail = Node;

    (*DphRoot).nFreeAllocations += 1;
    (*DphRoot).nFreeAllocationBytesCommitted += (*Node).nVirtualBlockSize;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPlaceOnPoolList(DphRoot: PDPH_HEAP_ROOT, Node: PDPH_HEAP_BLOCK) {
    (*Node).__bindgen_anon_1.pNextAlloc = core::ptr::null_mut();

    if !(*DphRoot).pNodePoolListTail.is_null() {
        (*(*DphRoot).pNodePoolListTail).__bindgen_anon_1.pNextAlloc = Node;
    } else {
        (*DphRoot).pNodePoolListHead = Node;
    }
    (*DphRoot).pNodePoolListTail = Node;

    (*DphRoot).nNodePools += 1;
    (*DphRoot).nNodePoolBytes += (*Node).nVirtualBlockSize;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphPlaceOnVirtualList(DphRoot: PDPH_HEAP_ROOT, Node: PDPH_HEAP_BLOCK) {
    (*Node).__bindgen_anon_1.pNextAlloc = (*DphRoot).pVirtualStorageListHead;
    if (*DphRoot).pVirtualStorageListHead.is_null() {
        (*DphRoot).pVirtualStorageListTail = Node;
    }
    (*DphRoot).pVirtualStorageListHead = Node;

    (*DphRoot).nVirtualStorageRanges += 1;
    (*DphRoot).nVirtualStorageBytes += (*Node).nVirtualBlockSize;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphTakeNodeFromUnusedList(DphRoot: PDPH_HEAP_ROOT) -> PDPH_HEAP_BLOCK {
    let Node: PDPH_HEAP_BLOCK = (*DphRoot).pUnusedNodeListHead;
    let Next: PDPH_HEAP_BLOCK;

    if Node.is_null() {
        return core::ptr::null_mut();
    }

    Next = (*Node).__bindgen_anon_1.pNextAlloc;
    if (*DphRoot).pUnusedNodeListHead == Node {
        (*DphRoot).pUnusedNodeListHead = Next;
    }
    if (*DphRoot).pUnusedNodeListTail == Node {
        (*DphRoot).pUnusedNodeListTail = core::ptr::null_mut();
    }

    (*DphRoot).nUnusedNodes -= 1;

    Node
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphReturnNodeToUnusedList(
    DphRoot: PDPH_HEAP_ROOT,
    Node: PDPH_HEAP_BLOCK,
) {
    (*Node).__bindgen_anon_1.pNextAlloc = (*DphRoot).pUnusedNodeListHead;
    if (*DphRoot).pUnusedNodeListHead.is_null() {
        (*DphRoot).pUnusedNodeListTail = Node;
    }
    (*DphRoot).pUnusedNodeListHead = Node;

    (*DphRoot).nUnusedNodes += 1;
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphRemoveFromAvailableList(
    DphRoot: PDPH_HEAP_ROOT,
    Node: PDPH_HEAP_BLOCK,
) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Node).__bindgen_anon_1.AvailableEntry) as PLIST_ENTRY);

    (*DphRoot).nAvailableAllocations -= 1;
    (*DphRoot).nAvailableAllocationBytesCommitted -= (*Node).nVirtualBlockSize;

    (*Node).AdjacencyEntry.Flink =
        ((*Node).AdjacencyEntry.Flink as usize & !(1usize)) as PLIST_ENTRY;
    (*Node).AdjacencyEntry.Blink =
        ((*Node).AdjacencyEntry.Blink as usize & !(1usize)) as PLIST_ENTRY;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphRemoveFromBusyList(
    DphRoot: PDPH_HEAP_ROOT,
    Node: PDPH_HEAP_BLOCK,
) {
    let _ElementPresent = RtlDeleteElementGenericTableAvl(
        core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable),
        core::ptr::addr_of_mut!((*Node).pUserAllocation) as *mut _,
    );

    (*DphRoot).nBusyAllocations -= 1;
    (*DphRoot).nBusyAllocationBytesCommitted -= (*Node).nVirtualBlockSize;
    (*DphRoot).nBusyAllocationBytesAccessible -= (*Node).nVirtualAccessSize;
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphRemoveFromFreeList(
    DphRoot: PDPH_HEAP_ROOT,
    Node: PDPH_HEAP_BLOCK,
    Prev: PDPH_HEAP_BLOCK,
) {
    let Next: PDPH_HEAP_BLOCK = (*Node).__bindgen_anon_1.pNextAlloc;

    if (*DphRoot).pFreeAllocationListHead == Node {
        (*DphRoot).pFreeAllocationListHead = Next;
    }
    if (*DphRoot).pFreeAllocationListTail == Node {
        (*DphRoot).pFreeAllocationListTail = Prev;
    }
    if !Prev.is_null() {
        (*Prev).__bindgen_anon_1.pNextAlloc = Next;
    }

    (*DphRoot).nFreeAllocations -= 1;
    (*DphRoot).nFreeAllocationBytesCommitted -= (*Node).nVirtualBlockSize;

    (*Node).StackTrace = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphCoalesceNodeIntoAvailable(
    DphRoot: PDPH_HEAP_ROOT,
    mut Node: PDPH_HEAP_BLOCK,
) {
    let mut PrevNode: PDPH_HEAP_BLOCK = core::ptr::null_mut();

    /* Update heap counters */
    (*DphRoot).nAvailableAllocationBytesCommitted += (*Node).nVirtualBlockSize;
    (*DphRoot).nAvailableAllocations += 1;

    /* Find where to put this node according to its virtual address */
    let AvailListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*DphRoot).AvailableAllocationHead);

    /* Find a point where to insert an available node */
    let mut CurEntry: PLIST_ENTRY = (*AvailListHead).Flink;

    while CurEntry != AvailListHead {
        let NodeEntry: PDPH_HEAP_BLOCK = (CurEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_DPH_HEAP_BLOCK, __bindgen_anon_1) as isize))
            as PDPH_HEAP_BLOCK;
        if (*NodeEntry).pVirtualBlock >= (*Node).pVirtualBlock {
            PrevNode = NodeEntry;
            break;
        }
        CurEntry = (*CurEntry).Flink;
    }

    if PrevNode.is_null() {
        /* That means either this list is empty, or we should add to the head of it */
        InsertHeadList(AvailListHead, core::ptr::addr_of_mut!((*Node).__bindgen_anon_1.AvailableEntry));
    } else {
        /* Check the previous node and merge if possible */
        if (*PrevNode).pVirtualBlock.add((*PrevNode).nVirtualBlockSize) == (*Node).pVirtualBlock {
            let mut MemoryBasicInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

            let _Status = ZwQueryVirtualMemory(
                -1isize as *mut core::ffi::c_void,
                (*Node).pVirtualBlock as *mut core::ffi::c_void,
                _MEMORY_INFORMATION_CLASS_MemoryBasicInformation,
                core::ptr::addr_of_mut!(MemoryBasicInfo) as *mut core::ffi::c_void,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
                core::ptr::null_mut(),
            );

            if (MemoryBasicInfo.AllocationBase as *mut u8) <= (*PrevNode).pVirtualBlock {
                /* They are adjacent, and from the same VM region. - merge! */
                (*PrevNode).nVirtualBlockSize += (*Node).nVirtualBlockSize;
                RtlpDphReturnNodeToUnusedList(DphRoot, Node);
                (*DphRoot).nAvailableAllocations -= 1;

                Node = PrevNode;
            } else {
                /* Insert after PrevNode */
                InsertTailList(
                    core::ptr::addr_of_mut!((*PrevNode).__bindgen_anon_1.AvailableEntry),
                    core::ptr::addr_of_mut!((*Node).__bindgen_anon_1.AvailableEntry),
                );
            }
        } else {
            /* Insert after PrevNode */
            InsertTailList(
                core::ptr::addr_of_mut!((*PrevNode).__bindgen_anon_1.AvailableEntry),
                core::ptr::addr_of_mut!((*Node).__bindgen_anon_1.AvailableEntry),
            );
        }

        /* Now check the next entry after our one */
        if (*Node).__bindgen_anon_1.AvailableEntry.Flink != AvailListHead {
            let NextNode: PDPH_HEAP_BLOCK = ((*Node).__bindgen_anon_1.AvailableEntry.Flink as *mut u8)
                .offset(-(core::mem::offset_of!(_DPH_HEAP_BLOCK, __bindgen_anon_1) as isize))
                as PDPH_HEAP_BLOCK;
            /* Node is not at the tail of the list, check if it's adjacent */
            if (*Node).pVirtualBlock.add((*Node).nVirtualBlockSize) == (*NextNode).pVirtualBlock {
                let mut MemoryBasicInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

                let _Status = ZwQueryVirtualMemory(
                    -1isize as *mut core::ffi::c_void,
                    (*NextNode).pVirtualBlock as *mut core::ffi::c_void,
                    _MEMORY_INFORMATION_CLASS_MemoryBasicInformation,
                    core::ptr::addr_of_mut!(MemoryBasicInfo) as *mut core::ffi::c_void,
                    core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
                    core::ptr::null_mut(),
                );

                if (MemoryBasicInfo.AllocationBase as *mut u8) <= (*Node).pVirtualBlock {
                    /* They are adjacent - merge! */
                    (*Node).nVirtualBlockSize += (*NextNode).nVirtualBlockSize;

                    /* Remove next entry from the list and put it into unused entries list */
                    RemoveEntryList(core::ptr::addr_of_mut!((*NextNode).__bindgen_anon_1.AvailableEntry));
                    RtlpDphReturnNodeToUnusedList(DphRoot, NextNode);
                    (*DphRoot).nAvailableAllocations -= 1;
                }
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphCoalesceFreeIntoAvailable(
    DphRoot: PDPH_HEAP_ROOT,
    LeaveOnFreeList: ULONG,
) {
    let mut Node: PDPH_HEAP_BLOCK = (*DphRoot).pFreeAllocationListHead;
    let mut Next: PDPH_HEAP_BLOCK;
    let mut FreeAllocations: SIZE_T = (*DphRoot).nFreeAllocations as SIZE_T;

    while !Node.is_null() {
        FreeAllocations = FreeAllocations.wrapping_sub(1);
        if FreeAllocations < LeaveOnFreeList as SIZE_T {
            break;
        }

        Next = (*Node).__bindgen_anon_1.pNextAlloc;

        RtlpDphRemoveFromFreeList(DphRoot, Node, core::ptr::null_mut());

        RtlpDphCoalesceNodeIntoAvailable(DphRoot, Node);

        Node = Next;
    }
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphAddNewPool(
    DphRoot: PDPH_HEAP_ROOT,
    _NodeBlock: PDPH_HEAP_BLOCK,
    Virtual: *mut core::ffi::c_void,
    Size: SIZE_T,
    PlaceOnPool: BOOLEAN,
) {
    let NodeCount: ULONG = (Size / core::mem::size_of::<DPH_HEAP_BLOCK>()) as ULONG;
    let DphStartNode: PDPH_HEAP_BLOCK = Virtual as PDPH_HEAP_BLOCK;

    let mut DphNode: PDPH_HEAP_BLOCK = Virtual as PDPH_HEAP_BLOCK;
    let mut i: ULONG = NodeCount - 1;
    while i > 0 {
        (*DphNode).__bindgen_anon_1.pNextAlloc = DphNode.add(1);
        DphNode = (*DphNode).__bindgen_anon_1.pNextAlloc;
        i -= 1;
    }

    (*DphNode).__bindgen_anon_1.pNextAlloc = core::ptr::null_mut();

    if !(*DphRoot).pUnusedNodeListTail.is_null() {
        (*(*DphRoot).pUnusedNodeListTail).__bindgen_anon_1.pNextAlloc = DphStartNode;
    } else {
        (*DphRoot).pUnusedNodeListHead = DphStartNode;
    }

    (*DphRoot).pUnusedNodeListTail = DphNode;

    (*DphRoot).nUnusedNodes += NodeCount;

    if PlaceOnPool != 0 {
        DphNode = RtlpDphTakeNodeFromUnusedList(DphRoot);

        (*DphNode).pVirtualBlock = Virtual as PUCHAR;
        (*DphNode).nVirtualBlockSize = Size;

        RtlpDphPlaceOnPoolList(DphRoot, DphNode);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphSearchAvailableMemoryListForBestFit(
    DphRoot: PDPH_HEAP_ROOT,
    Size: SIZE_T,
) -> PDPH_HEAP_BLOCK {
    let mut NodeFound: PDPH_HEAP_BLOCK = core::ptr::null_mut();

    let mut CurEntry: PLIST_ENTRY = (*DphRoot).AvailableAllocationHead.Flink;

    while CurEntry != core::ptr::addr_of_mut!((*DphRoot).AvailableAllocationHead) {
        let Node = (CurEntry as usize
            - core::mem::offset_of!(_DPH_HEAP_BLOCK, __bindgen_anon_1))
            as PDPH_HEAP_BLOCK;

        if (*Node).nVirtualBlockSize >= Size {
            NodeFound = Node;
            break;
        }

        CurEntry = (*CurEntry).Flink;
    }

    NodeFound
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphFindAvailableMemory(
    DphRoot: PDPH_HEAP_ROOT,
    Size: SIZE_T,
    Grow: BOOLEAN,
) -> PDPH_HEAP_BLOCK {
    let mut Node: PDPH_HEAP_BLOCK;
    let mut NewSize: ULONG;

    Node = RtlpDphSearchAvailableMemoryListForBestFit(DphRoot, Size);

    while Node.is_null() {
        if (*DphRoot).nFreeAllocations <= DPH_FREE_LIST_MINIMUM {
            break;
        }

        NewSize = (*DphRoot).nFreeAllocations >> 2;
        if NewSize < DPH_FREE_LIST_MINIMUM {
            NewSize = DPH_FREE_LIST_MINIMUM;
        }

        RtlpDphCoalesceFreeIntoAvailable(DphRoot, NewSize);

        Node = RtlpDphSearchAvailableMemoryListForBestFit(DphRoot, Size);
    }

    if Node.is_null() && Grow != 0 {
        if RtlpDphGrowVirtual(DphRoot, Size) == 0 {
            return core::ptr::null_mut();
        }

        Node = RtlpDphSearchAvailableMemoryListForBestFit(DphRoot, Size);

        if Node.is_null() {
            if (*DphRoot).nFreeAllocationBytesCommitted
                + (*DphRoot).nAvailableAllocationBytesCommitted
                >= Size
            {
                RtlpDphCoalesceFreeIntoAvailable(DphRoot, 0);

                Node = RtlpDphSearchAvailableMemoryListForBestFit(DphRoot, Size);
            }
        }
    }

    Node
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphFindBusyMemory(
    DphRoot: PDPH_HEAP_ROOT,
    pUserMem: PVOID,
) -> PDPH_HEAP_BLOCK {
    let mut pUserMemLocal = pUserMem;
    let Ptr = RtlLookupElementGenericTableAvl(
        core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable),
        core::ptr::addr_of_mut!(pUserMemLocal) as PVOID,
    );
    if Ptr.is_null() {
        return core::ptr::null_mut();
    }

    let Node = (Ptr as PUCHAR)
        .offset(-(core::mem::offset_of!(_DPH_HEAP_BLOCK, pUserAllocation) as isize))
        as PDPH_HEAP_BLOCK;
    Node
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphSetProtectionBeforeUse(
    DphRoot: PDPH_HEAP_ROOT,
    VirtualBlock: PUCHAR,
    UserSize: ULONG,
) -> NTSTATUS {
    let Protection: ULONG;
    let Base: PVOID;

    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN != 0 {
        Base = VirtualBlock.add(PAGE_SIZE as usize) as PVOID;
    } else {
        Base = VirtualBlock as PVOID;
    }

    Protection = PAGE_READWRITE;

    RtlpDphProtectVm(Base, UserSize as SIZE_T, Protection)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphSetProtectionAfterUse(
    DphRoot: PDPH_HEAP_ROOT,
    _Node: PDPH_HEAP_BLOCK,
) -> NTSTATUS {
    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN != 0 {
    } else {
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphAllocateNode(DphRoot: PDPH_HEAP_ROOT) -> PDPH_HEAP_BLOCK {
    let mut Node: PDPH_HEAP_BLOCK;
    let mut Status: NTSTATUS;
    let mut Size: SIZE_T = DPH_POOL_SIZE as SIZE_T;
    let mut SizeVirtual: SIZE_T = 0;
    let mut Ptr: PVOID = core::ptr::null_mut();

    /* Check for the easy case */
    if !(*DphRoot).pUnusedNodeListHead.is_null() {
        /* Just take a node from this list */
        Node = RtlpDphTakeNodeFromUnusedList(DphRoot);
        return Node;
    }

    /* There is a need to make free space */
    Node = RtlpDphFindAvailableMemory(DphRoot, DPH_POOL_SIZE as SIZE_T, 0u8);

    if (*DphRoot).pUnusedNodeListHead.is_null() && Node.is_null() {
        /* Retry with a smaller request */
        Size = PAGE_SIZE as SIZE_T;
        Node = RtlpDphFindAvailableMemory(DphRoot, PAGE_SIZE as SIZE_T, 0u8);
    }

    if (*DphRoot).pUnusedNodeListHead.is_null() {
        if !Node.is_null() {
            RtlpDphRemoveFromAvailableList(DphRoot, Node);
            Ptr = (*Node).pVirtualBlock as PVOID;
            SizeVirtual = (*Node).nVirtualBlockSize;
        } else {
            /* No free space, need to alloc a new VM block */
            Size = DPH_POOL_SIZE as SIZE_T;
            SizeVirtual = DPH_RESERVE_SIZE as SIZE_T;
            Status = RtlpDphAllocateVm(&mut Ptr, SizeVirtual, MEM_COMMIT, PAGE_NOACCESS);

            if !NT_SUCCESS(Status) {
                /* Retry with a smaller size */
                SizeVirtual = 0x10000;
                Status = RtlpDphAllocateVm(&mut Ptr, SizeVirtual, MEM_COMMIT, PAGE_NOACCESS);
                if !NT_SUCCESS(Status) {
                    return core::ptr::null_mut();
                }
            }
        }

        /* VM is allocated at this point, set protection */
        Status = RtlpDphProtectVm(Ptr, Size, PAGE_READWRITE);
        if !NT_SUCCESS(Status) {
            if !Node.is_null() {
                RtlpDphCoalesceNodeIntoAvailable(DphRoot, Node);
            } else {
                //RtlpDphFreeVm();
            }

            return core::ptr::null_mut();
        }

        /* Zero the memory */
        if !Node.is_null() {
            RtlZeroMemory(Ptr, Size);
        }

        /* Add a new pool based on this VM */
        RtlpDphAddNewPool(DphRoot, Node, Ptr, Size, 1u8);

        if !Node.is_null() {
            if (*Node).nVirtualBlockSize > Size {
                (*Node).pVirtualBlock = (*Node).pVirtualBlock.add(Size as usize);
                (*Node).nVirtualBlockSize -= Size;

                RtlpDphCoalesceNodeIntoAvailable(DphRoot, Node);
            } else {
                RtlpDphReturnNodeToUnusedList(DphRoot, Node);
            }
        } else {
            /* The new VM block was just allocated a few code lines ago,
               so initialize it */
            Node = RtlpDphTakeNodeFromUnusedList(DphRoot);
            (*Node).pVirtualBlock = Ptr as PUCHAR;
            (*Node).nVirtualBlockSize = SizeVirtual;
            RtlpDphPlaceOnVirtualList(DphRoot, Node);

            Node = RtlpDphTakeNodeFromUnusedList(DphRoot);
            (*Node).pVirtualBlock = (Ptr as PUCHAR).add(Size as usize);
            (*Node).nVirtualBlockSize = SizeVirtual - Size;
            RtlpDphPlaceOnVirtualList(DphRoot, Node);

            /* Coalesce them into available list */
            RtlpDphCoalesceNodeIntoAvailable(DphRoot, Node);
        }
    }

    RtlpDphTakeNodeFromUnusedList(DphRoot)
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphGrowVirtual(
    DphRoot: PDPH_HEAP_ROOT,
    mut Size: SIZE_T,
) -> BOOLEAN {
    let mut Base: PVOID = core::ptr::null_mut();
    let mut VirtualSize: SIZE_T;
    let mut Status: NTSTATUS;

    let Node = RtlpDphAllocateNode(DphRoot);
    if Node.is_null() {
        return 0u8;
    }

    let AvailableNode = RtlpDphAllocateNode(DphRoot);
    if AvailableNode.is_null() {
        RtlpDphReturnNodeToUnusedList(DphRoot, Node);
        return 0u8;
    }

    Size = ROUND_UP(Size as _, 0xFFFF as _) as SIZE_T;
    VirtualSize = Size;
    if Size < DPH_RESERVE_SIZE as SIZE_T {
        VirtualSize = DPH_RESERVE_SIZE as SIZE_T;
    }

    Status = RtlpDphAllocateVm(&mut Base, VirtualSize, MEM_COMMIT, PAGE_NOACCESS);
    if !NT_SUCCESS(Status) {
        VirtualSize = Size;
        Status = RtlpDphAllocateVm(&mut Base, VirtualSize, MEM_COMMIT, PAGE_NOACCESS);
        if !NT_SUCCESS(Status) {
            RtlpDphReturnNodeToUnusedList(DphRoot, Node);
            RtlpDphReturnNodeToUnusedList(DphRoot, AvailableNode);
            return 0u8;
        }
    }

    (*Node).pVirtualBlock = Base as PUCHAR;
    (*Node).nVirtualBlockSize = VirtualSize;
    (*AvailableNode).pVirtualBlock = Base as PUCHAR;
    (*AvailableNode).nVirtualBlockSize = VirtualSize;

    RtlpDphPlaceOnVirtualList(DphRoot, Node);
    RtlpDphCoalesceNodeIntoAvailable(DphRoot, AvailableNode);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphCompareNodeForTable(
    _Table: PRTL_AVL_TABLE,
    FirstStruct: *mut core::ffi::c_void,
    SecondStruct: *mut core::ffi::c_void,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let FirstBlock: usize = *(FirstStruct as *mut usize);
    let SecondBlock: usize = *(SecondStruct as *mut usize);

    if FirstBlock < SecondBlock {
        return GenericLessThan;
    } else if FirstBlock > SecondBlock {
        return GenericGreaterThan;
    }

    GenericEqual
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphAllocateNodeForTable(
    Table: PRTL_AVL_TABLE,
    ByteSize: CLONG,
) -> PVOID {
    let _ = ByteSize;

    let DphRoot = (Table as *mut u8)
        .offset(-(core::mem::offset_of!(_DPH_HEAP_ROOT, BusyNodesTable) as isize))
        as *mut _DPH_HEAP_ROOT;

    let pBlock = (*DphRoot).NodeToAllocate;

    (*DphRoot).NodeToAllocate = core::ptr::null_mut();

    core::ptr::addr_of_mut!((*pBlock).__bindgen_anon_1.TableLinks) as PVOID
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphFreeNodeForTable(_Table: PRTL_AVL_TABLE, _Buffer: *mut core::ffi::c_void) {
    /* Nothing */
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphInitializeDelayedFreeQueue() -> NTSTATUS {
    let Status: NTSTATUS;

    Status = RtlInitializeHeapLock(core::ptr::addr_of_mut!(RtlpDphDelayedFreeQueueLock));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    InitializeListHead(core::ptr::addr_of_mut!(RtlpDphDelayedFreeQueue));
    RtlInitializeSListHead(core::ptr::addr_of_mut!(RtlpDphDelayedTemporaryPushList));

    RtlpDphMemoryUsedByDelayedFreeBlocks = 0;
    RtlpDphNumberOfDelayedFreeBlocks = 0;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphFreeDelayedBlocksFromHeap(
    DphRoot: PDPH_HEAP_ROOT,
    NormalHeap: PVOID,
) {
    let mut Current: PLIST_ENTRY;
    let mut Next: PLIST_ENTRY;
    let mut BlockInfo: PDPH_BLOCK_INFORMATION;
    let mut ValidationInfo: ULONG = 0;

    RtlEnterHeapLock(RtlpDphDelayedFreeQueueLock, 1u8);

    Current = RtlpDphDelayedFreeQueue.Flink;
    while Current != core::ptr::addr_of_mut!(RtlpDphDelayedFreeQueue) {
        Next = (*Current).Flink;

        BlockInfo = (Current as *mut u8).offset(
            -(core::mem::offset_of!(_DPH_BLOCK_INFORMATION, __bindgen_anon_1) as isize),
        ) as PDPH_BLOCK_INFORMATION;

        if (*BlockInfo).Heap == DphRoot as PVOID {
            RemoveEntryList(Current);

            (*BlockInfo).Heap = core::ptr::null_mut();

            if RtlpDphIsNormalFreeHeapBlock(
                BlockInfo.offset(1) as PVOID,
                &mut ValidationInfo,
                1u8,
            ) == 0
            {
                RtlpDphReportCorruptedBlock(
                    DphRoot,
                    10,
                    BlockInfo.offset(1) as PVOID,
                    ValidationInfo,
                );
            }

            RtlpDphMemoryUsedByDelayedFreeBlocks -= (*BlockInfo).ActualSize;
            RtlpDphNumberOfDelayedFreeBlocks -= 1;

            RtlFreeHeap(NormalHeap, 0, BlockInfo as PVOID);
        }

        Current = Next;
    }

    RtlLeaveHeapLock(RtlpDphDelayedFreeQueueLock);
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphTargetDllsLogicInitialize() -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphInternalValidatePageHeap(_DphRoot: PDPH_HEAP_ROOT, _Address: PVOID, _Value: ULONG) {
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphVerifyIntegrity(_DphRoot: PDPH_HEAP_ROOT) {
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphReportCorruptedBlock(
    DphRoot: PDPH_HEAP_ROOT,
    _Reserved: ULONG,
    Block: PVOID,
    ValidationInfo: ULONG,
) {
    let Size: PVOID = RtlpDphGetBlockSizeFromCorruptedBlock(Block) as usize as PVOID;
    let mut SafeInfo: DPH_BLOCK_INFORMATION = core::mem::zeroed();

    {
        let BlockInfo = (Block as PDPH_BLOCK_INFORMATION).offset(-1);
        core::ptr::copy_nonoverlapping(
            BlockInfo,
            &mut SafeInfo,
            1,
        );
    }

    if ValidationInfo & DPH_VALINFO_CORRUPTED_AFTER_FREE != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_HEADER,
            b"block corrupted after having been freed\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_ALREADY_FREED != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_DOUBLE_FREE,
            b"block already freed\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_INFIX_PATTERN != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_FREED_HEAP_BLOCK,
            b"corrupted infix pattern for freed block\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_POINTER != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_SWITCHED_HEAP_HANDLE,
            b"corrupted heap pointer or using wrong heap\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            SafeInfo.Heap,
            b"Actual heap handle\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_SUFFIX_PATTERN != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_SUFFIX,
            b"corrupted suffix pattern\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_PREFIX_PATTERN != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_PREFIX,
            b"corrupted prefix pattern\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_START_STAMP != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_START_STAMP,
            b"corrupted start stamp\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            SafeInfo.StartStamp as usize as PVOID,
            b"Corrupted start stamp\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_BAD_END_STAMP != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_END_STAMP,
            b"corrupted end stamp\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            SafeInfo.EndStamp as usize as PVOID,
            b"Corrupted end stamp\0".as_ptr() as PCHAR,
        );
    }

    if ValidationInfo & DPH_VALINFO_EXCEPTION != 0 {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_CORRUPTED_HEAP_BLOCK_EXCEPTION_RAISED_FOR_HEADER,
            b"exception raised while verifying block\0".as_ptr() as PCHAR,
            RtlpDphHeapFromPointer(DphRoot),
            b"Heap handle used\0".as_ptr() as PCHAR,
            Block,
            b"Heap block\0".as_ptr() as PCHAR,
            Size,
            b"Block size\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphIsPageHeapBlock(
    DphRoot: PDPH_HEAP_ROOT,
    Block: PVOID,
    ValidationInformation: PULONG,
    CheckFillers: BOOLEAN,
) -> BOOLEAN {
    let BlockInfo: PDPH_BLOCK_INFORMATION;
    let mut SomethingWrong: BOOLEAN = 0u8;
    let mut Byte: PUCHAR;
    let Start: PUCHAR;
    let End: PUCHAR;

    core::ptr::write_volatile(ValidationInformation, 0);

    BlockInfo = (Block as PDPH_BLOCK_INFORMATION).offset(-1);

    /* Check stamps */
    if (*BlockInfo).StartStamp != DPH_FILL_START_STAMP_1 {
        *ValidationInformation |= DPH_VALINFO_BAD_START_STAMP;
        SomethingWrong = 1u8;

        /* Check if it has an alloc/free mismatch */
        if (*BlockInfo).StartStamp == DPH_FILL_START_STAMP_2 {
            core::ptr::write_volatile(ValidationInformation, 0x101);
        }
    }

    if (*BlockInfo).EndStamp != DPH_FILL_END_STAMP_1 {
        *ValidationInformation |= DPH_VALINFO_BAD_END_STAMP;
        SomethingWrong = 1u8;
    }

    /* Check root heap pointer */
    if (*BlockInfo).Heap != DphRoot as PVOID {
        *ValidationInformation |= DPH_VALINFO_BAD_POINTER;
        SomethingWrong = 1u8;
    }

    /* Check other fillers if requested */
    if CheckFillers != 0 {
        Start = (Block as PUCHAR).offset((*BlockInfo).RequestedSize as isize);
        End = ROUND_UP(Start as usize, PAGE_SIZE as usize) as PUCHAR;
        Byte = Start;
        while Byte < End {
            if *Byte != DPH_FILL_SUFFIX as UCHAR {
                *ValidationInformation |= DPH_VALINFO_BAD_SUFFIX_PATTERN;
                SomethingWrong = 1u8;
                break;
            }
            Byte = Byte.offset(1);
        }
    }

    (SomethingWrong == 0u8) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphIsNormalFreeHeapBlock(
    _Block: PVOID,
    ValidationInformation: PULONG,
    _CheckFillers: BOOLEAN,
) -> BOOLEAN {
    core::ptr::write_volatile(ValidationInformation, 0);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDphProcessStartupInitialization() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let _Teb: PTEB = NtCurrentTeb();

    InitializeListHead(core::ptr::addr_of_mut!(RtlpDphPageHeapList));
    Status = RtlInitializeHeapLock(core::ptr::addr_of_mut!(RtlpDphPageHeapListLock));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlpDphInitializeDelayedFreeQueue();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(RtlpDphTargetDllsUnicode),
        (&raw mut RtlpDphTargetDlls).cast::<WCHAR>(),
    );
    Status = RtlpDphTargetDllsLogicInitialize();

    RtlpDphPageHeapListInitialized = 1u8;

    Status
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphShouldAllocateInPageHeap(
    _DphRoot: PDPH_HEAP_ROOT,
    _Size: SIZE_T,
) -> BOOLEAN {
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapCreate(
    Flags: ULONG,
    Addr: PVOID,
    TotalSize: SIZE_T,
    CommitSize: SIZE_T,
    Lock: PVOID,
    Parameters: PRTL_HEAP_PARAMETERS,
) -> HANDLE {
    let mut Base: PVOID = core::ptr::null_mut();
    let HeapPtr: PHEAP;
    let DphRoot: PDPH_HEAP_ROOT;
    let mut DphNode: PDPH_HEAP_BLOCK;
    let MemSize: ULONG;
    let mut Status: NTSTATUS;
    let mut PerfCounter: LARGE_INTEGER = core::mem::zeroed();

    /* Check for a DPH bypass flag */
    if Parameters as ULONG_PTR == ULONG_PTR::MAX {
        return core::ptr::null_mut();
    }

    /* Make sure no user-allocated stuff was provided */
    if !Addr.is_null() || !Lock.is_null() {
        return core::ptr::null_mut();
    }

    /* Allocate minimum amount of virtual memory */
    MemSize = DPH_RESERVE_SIZE;
    Status = RtlpDphAllocateVm(&mut Base, MemSize as SIZE_T, MEM_COMMIT, PAGE_NOACCESS);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    /* Set protection */
    Status = RtlpDphProtectVm(
        Base,
        (2 * PAGE_SIZE + DPH_POOL_SIZE as usize) as SIZE_T,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    /* Start preparing the 1st page. Fill it with the default filler */
    RtlFillMemoryUlong(Base, PAGE_SIZE as _, DPH_FILL);

    /* Set flags in the "HEAP" structure */
    HeapPtr = Base as PHEAP;
    (*HeapPtr).Flags = Flags | HEAP_FLAG_PAGE_ALLOCS;
    (*HeapPtr).ForceFlags = Flags | HEAP_FLAG_PAGE_ALLOCS;

    /* Set 1st page to read only now */
    Status = RtlpDphProtectVm(Base, PAGE_SIZE as SIZE_T, PAGE_READONLY);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    /* 2nd page is the real DPH root block */
    DphRoot = (Base as PCHAR).add(PAGE_SIZE as usize) as PDPH_HEAP_ROOT;

    /* Initialize the DPH root */
    (*DphRoot).Signature = DPH_SIGNATURE;
    (*DphRoot).HeapFlags = Flags;
    (*DphRoot).HeapCritSect = (DphRoot as PCHAR).add(DPH_POOL_SIZE as usize) as PHEAP_LOCK;
    (*DphRoot).ExtraFlags = RtlpDphGlobalFlags;

    ZwQueryPerformanceCounter(&mut PerfCounter, core::ptr::null_mut());
    (*DphRoot).Seed = PerfCounter.__bindgen_anon_1.LowPart;

    RtlInitializeHeapLock(core::ptr::addr_of_mut!((*DphRoot).HeapCritSect));
    InitializeListHead(core::ptr::addr_of_mut!((*DphRoot).AvailableAllocationHead));

    /* Create a normal heap for this paged heap */
    (*DphRoot).NormalHeap = RtlCreateHeap(
        Flags,
        core::ptr::null_mut(),
        TotalSize as _,
        CommitSize as _,
        core::ptr::null_mut(),
        (usize::MAX as *mut core::ffi::c_void) as PRTL_HEAP_PARAMETERS,
    );
    if (*DphRoot).NormalHeap.is_null() {
        return core::ptr::null_mut();
    }

    /* 3rd page: a pool for DPH allocations */
    RtlpDphAddNewPool(
        DphRoot,
        core::ptr::null_mut(),
        DphRoot.add(1) as *mut core::ffi::c_void,
        (DPH_POOL_SIZE as usize - core::mem::size_of::<DPH_HEAP_ROOT>()) as SIZE_T,
        0u8,
    );

    /* Allocate internal heap blocks. For the root */
    DphNode = RtlpDphAllocateNode(DphRoot);
    (*DphNode).pVirtualBlock = DphRoot as PUCHAR;
    (*DphNode).nVirtualBlockSize = DPH_POOL_SIZE as SIZE_T;
    RtlpDphPlaceOnPoolList(DphRoot, DphNode);

    /* For the memory we allocated as a whole */
    DphNode = RtlpDphAllocateNode(DphRoot);
    (*DphNode).pVirtualBlock = Base as PUCHAR;
    (*DphNode).nVirtualBlockSize = MemSize as SIZE_T;
    RtlpDphPlaceOnVirtualList(DphRoot, DphNode);

    /* For the remaining part */
    DphNode = RtlpDphAllocateNode(DphRoot);
    (*DphNode).pVirtualBlock = (Base as PUCHAR).add(2 * PAGE_SIZE + DPH_POOL_SIZE as usize);
    (*DphNode).nVirtualBlockSize = (MemSize as usize - (2 * PAGE_SIZE + DPH_POOL_SIZE as usize)) as SIZE_T;
    RtlpDphCoalesceNodeIntoAvailable(DphRoot, DphNode);

    /* Initialize AVL-based busy nodes table */
    RtlInitializeGenericTableAvl(
        core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable),
        Some(unsafe { core::mem::transmute(RtlpDphCompareNodeForTable as *const ()) }),
        Some(unsafe { core::mem::transmute(RtlpDphAllocateNodeForTable as *const ()) }),
        Some(unsafe { core::mem::transmute(RtlpDphFreeNodeForTable as *const ()) }),
        core::ptr::null_mut(),
    );

    /* Initialize per-process startup info */
    if RtlpDphPageHeapListInitialized == 0 {
        RtlpDphProcessStartupInitialization();
    }

    /* Acquire the heap list lock */
    RtlEnterHeapLock(RtlpDphPageHeapListLock, 1u8);

    /* Insert this heap to the tail of the global list */
    InsertTailList(
        core::ptr::addr_of_mut!(RtlpDphPageHeapList),
        core::ptr::addr_of_mut!((*DphRoot).NextHeap),
    );

    /* Note we increased the size of the list */
    RtlpDphPageHeapListLength += 1;

    /* Release the heap list lock */
    RtlLeaveHeapLock(RtlpDphPageHeapListLock);

    /* Perform internal validation if required */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    (DphRoot as PUCHAR).sub(PAGE_SIZE as usize) as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapDestroy(HeapPtr: HANDLE) -> PVOID {
    let DphRoot: PDPH_HEAP_ROOT;
    let mut Ptr: PVOID;
    let mut Node: PDPH_HEAP_BLOCK;
    let mut Next: PDPH_HEAP_BLOCK;
    let NormalHeap: PVOID;
    let mut Value: ULONG = 0;

    /* Check if it's not a process heap */
    if HeapPtr == RtlGetProcessHeap() {
        VERIFIER_STOP(
            APPLICATION_VERIFIER_DESTROY_PROCESS_HEAP,
            b"attempt to destroy process heap\0".as_ptr() as PCHAR,
            HeapPtr,
            b"Heap handle\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
            core::ptr::null_mut(),
            b"\0".as_ptr() as PCHAR,
        );
        return core::ptr::null_mut();
    }

    /* Get pointer to the heap root */
    DphRoot = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return core::ptr::null_mut();
    }

    RtlpDphPreProcessing(DphRoot, (*DphRoot).HeapFlags);

    /* Get the pointer to the normal heap */
    NormalHeap = (*DphRoot).NormalHeap;

    /* Free the delayed-free blocks */
    RtlpDphFreeDelayedBlocksFromHeap(DphRoot, NormalHeap);

    /* Go through the busy blocks */
    Ptr = RtlEnumerateGenericTableAvl(core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable), 1u8);

    while !Ptr.is_null() {
        Node = (Ptr as *mut u8).offset(
            -(core::mem::offset_of!(_DPH_HEAP_BLOCK, pUserAllocation) as isize),
        ) as PDPH_HEAP_BLOCK;

        if ((*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN) == 0 {
            if RtlpDphIsPageHeapBlock(
                DphRoot,
                (*Node).pUserAllocation as PVOID,
                &mut Value,
                1u8,
            ) == 0
            {
                RtlpDphReportCorruptedBlock(DphRoot, 3, (*Node).pUserAllocation as PVOID, Value);
            }
        }

        AVrfInternalHeapFreeNotification(
            (*Node).pUserAllocation as PVOID,
            (*Node).nUserRequestedSize,
        );

        /* Go to the next node */
        Ptr = RtlEnumerateGenericTableAvl(core::ptr::addr_of_mut!((*DphRoot).BusyNodesTable), 0u8);
    }

    /* Acquire the global heap list lock */
    RtlEnterHeapLock(RtlpDphPageHeapListLock, 1u8);

    /* Remove the entry and decrement the global counter */
    RemoveEntryList(core::ptr::addr_of_mut!((*DphRoot).NextHeap));
    RtlpDphPageHeapListLength -= 1;

    /* Release the global heap list lock */
    RtlLeaveHeapLock(RtlpDphPageHeapListLock);

    /* Leave and delete this heap's critical section */
    RtlLeaveHeapLock((*DphRoot).HeapCritSect);
    RtlDeleteHeapLock((*DphRoot).HeapCritSect);

    /* Now go through all virtual list nodes and release the VM */
    Node = (*DphRoot).pVirtualStorageListHead;
    while !Node.is_null() {
        Next = (*Node).__bindgen_anon_1.pNextAlloc;
        /* Release the memory without checking result */
        RtlpDphFreeVm((*Node).pVirtualBlock as PVOID, 0, MEM_RELEASE);
        Node = Next;
    }

    /* Destroy the normal heap */
    RtlDestroyHeap(NormalHeap);

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapAllocate(
    HeapPtr: PVOID,
    Flags: ULONG,
    Size: SIZE_T,
) -> PVOID {
    let mut HeapPtr = HeapPtr;
    let mut Flags = Flags;
    let DphRoot: PDPH_HEAP_ROOT;
    let AvailableNode: PDPH_HEAP_BLOCK;
    let BusyNode: PDPH_HEAP_BLOCK;
    let mut Biased: BOOLEAN = 0u8;
    let AllocateSize: ULONG;
    let AccessSize: ULONG;
    let Status: NTSTATUS;
    let UserActualSize: SIZE_T;
    let Ptr: PVOID;

    /* Check requested size */
    if Size > 0x7FF00000 {
        /* Generate an exception if needed */
        if Flags & HEAP_GENERATE_EXCEPTIONS != 0 {
            RtlpDphRaiseException(STATUS_NO_MEMORY);
        }

        return core::ptr::null_mut();
    }

    /* Unbias the pointer if necessary */
    if (HeapPtr as ULONG_PTR) & 1 != 0 {
        HeapPtr = ((HeapPtr as ULONG_PTR) & !(1 as ULONG_PTR)) as PVOID;
        Biased = 1u8;
    }

    /* Get a pointer to the heap root */
    DphRoot = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return core::ptr::null_mut();
    }

    /* Acquire the heap lock */
    RtlpDphPreProcessing(DphRoot, Flags);

    /* Perform internal validation if specified by flags */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 && Biased == 0 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    /* Add heap flags */
    Flags |= (*DphRoot).HeapFlags;

    if Biased == 0 && RtlpDphShouldAllocateInPageHeap(DphRoot, Size) == 0 {
        /* Perform allocation from a normal heap */
    }

    /* Perform heap integrity check if specified by flags */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
        RtlpDphVerifyIntegrity(DphRoot);
    }

    /* Calculate sizes */
    AccessSize = ROUND_UP(
        Size + core::mem::size_of::<DPH_BLOCK_INFORMATION>(),
        PAGE_SIZE as SIZE_T,
    ) as ULONG;
    AllocateSize = (AccessSize as usize + PAGE_SIZE) as ULONG;

    AvailableNode = RtlpDphFindAvailableMemory(DphRoot, AllocateSize as SIZE_T, 1u8);
    if AvailableNode.is_null() {
        DbgBreakPoint();

        /* Release the lock */
        RtlpDphPostProcessing(DphRoot);

        return core::ptr::null_mut();
    }

    /* Set protection */
    Status = RtlpDphSetProtectionBeforeUse(
        DphRoot,
        (*AvailableNode).pVirtualBlock,
        AccessSize,
    );
    if !NT_SUCCESS(Status) {
    }

    /* Save available node pointer */
    Ptr = (*AvailableNode).pVirtualBlock as PVOID;

    /* Check node's size */
    if (*AvailableNode).nVirtualBlockSize > AllocateSize as SIZE_T {
        /* The block contains too much free space, reduce it */
        (*AvailableNode).pVirtualBlock = (*AvailableNode).pVirtualBlock.add(AllocateSize as usize);
        (*AvailableNode).nVirtualBlockSize -= AllocateSize as SIZE_T;
        (*DphRoot).nAvailableAllocationBytesCommitted -= AllocateSize as SIZE_T;

        /* Allocate a new node which will be our busy node */
        BusyNode = RtlpDphAllocateNode(DphRoot);
        (*BusyNode).pVirtualBlock = Ptr as PUCHAR;
        (*BusyNode).nVirtualBlockSize = AllocateSize as SIZE_T;
    } else {
        /* The block's size fits exactly */
        RtlpDphRemoveFromAvailableList(DphRoot, AvailableNode);
        BusyNode = AvailableNode;
    }

    /* Calculate actual user size  */
    if (*DphRoot).HeapFlags & HEAP_NO_ALIGNMENT != 0 {
        UserActualSize = Size;
    } else {
        UserActualSize = ROUND_UP(Size, 8 as SIZE_T) as SIZE_T;
    }

    /* Set up the block */
    (*BusyNode).nVirtualAccessSize = AccessSize as SIZE_T;
    (*BusyNode).nUserActualSize = UserActualSize;
    (*BusyNode).nUserRequestedSize = Size;

    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN != 0 {
        (*BusyNode).pUserAllocation = (*BusyNode).pVirtualBlock.add(PAGE_SIZE as usize);
    } else {
        (*BusyNode).pUserAllocation = (*BusyNode)
            .pVirtualBlock
            .add((*BusyNode).nVirtualAccessSize as usize)
            .sub(UserActualSize as usize);
    }

    (*BusyNode).UserValue = core::ptr::null_mut();
    (*BusyNode).UserFlags = Flags & HEAP_SETTABLE_USER_FLAGS;

    (*BusyNode).StackTrace = core::ptr::null_mut();

    /* Place it on busy list */
    RtlpDphPlaceOnBusyList(DphRoot, BusyNode);

    /* Zero or patter-fill memory depending on flags */
    if Flags & HEAP_ZERO_MEMORY != 0 {
        RtlZeroMemory((*BusyNode).pUserAllocation as PVOID, Size);
    } else {
        core::ptr::write_bytes((*BusyNode).pUserAllocation, DPH_FILL_INFIX as u8, Size as usize);
    }

    /* Write DPH info */
    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN == 0 {
        RtlpDphWritePageHeapBlockInformation(
            DphRoot,
            (*BusyNode).pUserAllocation as PVOID,
            Size,
            AccessSize as SIZE_T,
        );
    }

    /* Finally allocation is done, perform validation again if required */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 && Biased == 0 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    /* Release the lock */
    RtlpDphPostProcessing(DphRoot);

    /* Return pointer to user allocation */
    (*BusyNode).pUserAllocation as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapFree(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT;
    let Node: PDPH_HEAP_BLOCK;
    let mut ValidationInfo: ULONG = 0;
    let Info: PDPH_BLOCK_INFORMATION;

    /* Check for a NULL pointer freeing */
    if Ptr.is_null() {
        if RtlpDphBreakOptions & DPH_BREAK_ON_NULL_FREE != 0 {
            DbgBreakPoint();
        }
        return 1u8;
    }

    /* Get a pointer to the heap root */
    DphRoot = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    /* Acquire the heap lock */
    RtlpDphPreProcessing(DphRoot, Flags);

    /* Perform internal validation if specified by flags */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    /* Add heap flags */
    Flags |= (*DphRoot).HeapFlags;

    /* Find busy memory */
    Node = RtlpDphFindBusyMemory(DphRoot, Ptr);

    if Node.is_null() {
        /* This block was not found in page heap, try a normal heap instead */
    }

    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN == 0 {
        /* Check and report corrupted block */
        if RtlpDphIsPageHeapBlock(DphRoot, Ptr, &mut ValidationInfo, 1u8) == 0 {
            RtlpDphReportCorruptedBlock(DphRoot, 1, Ptr, ValidationInfo);
        }

        if (*Node).nVirtualAccessSize != 0 {
            /* Set stamps */
            Info = ((*Node).pUserAllocation as PDPH_BLOCK_INFORMATION).offset(-1);
            (*Info).StartStamp = DPH_FILL_START_STAMP_2;
            (*Info).EndStamp = DPH_FILL_END_STAMP_2;

            RtlpDphProtectVm(
                (*Node).pVirtualBlock as PVOID,
                (*Node).nVirtualAccessSize,
                PAGE_NOACCESS,
            );
        }
    } else {
        if (*Node).nVirtualAccessSize != 0 {
            RtlpDphProtectVm(
                (*Node).pVirtualBlock.add(PAGE_SIZE as usize) as PVOID,
                (*Node).nVirtualAccessSize,
                PAGE_NOACCESS,
            );
        }
    }

    /* Remove it from the list of busy nodes */
    RtlpDphRemoveFromBusyList(DphRoot, Node);

    /* And put it into the list of free nodes */
    RtlpDphPlaceOnFreeList(DphRoot, Node);

    (*Node).StackTrace = core::ptr::null_mut();

    /* Leave the heap lock */
    RtlpDphPostProcessing(DphRoot);

    /* Return success */
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapReAllocate(
    mut HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
    Size: SIZE_T,
) -> PVOID {
    let DphRoot: PDPH_HEAP_ROOT;
    let mut Node: PDPH_HEAP_BLOCK = core::ptr::null_mut();
    let AllocatedNode: PDPH_HEAP_BLOCK;
    let mut Biased: BOOLEAN = 0u8;
    let mut UseNormalHeap: BOOLEAN = 0u8;
    let mut OldBlockPageHeap: BOOLEAN = 1u8;
    let mut ValidationInfo: ULONG = 0;
    let DataSize: SIZE_T;
    let mut NewAlloc: PVOID = core::ptr::null_mut();

    /* Check requested size */
    if Size > 0x7FF00000 {
        /* Generate an exception if needed */
        if Flags & HEAP_GENERATE_EXCEPTIONS != 0 {
            RtlpDphRaiseException(STATUS_NO_MEMORY);
        }

        return core::ptr::null_mut();
    }

    /* Unbias the pointer if necessary */
    if (HeapPtr as ULONG_PTR) & 1 != 0 {
        HeapPtr = ((HeapPtr as ULONG_PTR) & !(1 as ULONG_PTR)) as HANDLE;
        Biased = 1u8;
    }

    /* Get a pointer to the heap root */
    DphRoot = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return core::ptr::null_mut();
    }

    /* Acquire the heap lock */
    RtlpDphPreProcessing(DphRoot, Flags);

    /* Perform internal validation if specified by flags */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    /* Add heap flags */
    Flags |= (*DphRoot).HeapFlags;

    /* Exit with NULL right away if inplace is specified */
    if Flags & HEAP_REALLOC_IN_PLACE_ONLY != 0 {
        /* Release the lock */
        RtlpDphPostProcessing(DphRoot);

        /* Generate an exception if needed */
        if Flags & HEAP_GENERATE_EXCEPTIONS != 0 {
            RtlpDphRaiseException(STATUS_NO_MEMORY);
        }

        return core::ptr::null_mut();
    }

    /* Try to get node of the allocated block */
    AllocatedNode = RtlpDphFindBusyMemory(DphRoot, Ptr);

    if AllocatedNode.is_null() {
        /* This block was not found in page heap, try a normal heap instead */
        ntkrnl_sys::ASSERT!(false);
        OldBlockPageHeap = 0u8;
    }

    /* Check the block */
    if (*DphRoot).ExtraFlags & DPH_EXTRA_CHECK_UNDERRUN == 0 {
        if RtlpDphIsPageHeapBlock(
            DphRoot,
            (*AllocatedNode).pUserAllocation as PVOID,
            &mut ValidationInfo,
            1u8,
        ) == 0
        {
            RtlpDphReportCorruptedBlock(
                DphRoot,
                3,
                (*AllocatedNode).pUserAllocation as PVOID,
                ValidationInfo,
            );
        }
    }

    /* Remove old one from the busy list */
    RtlpDphRemoveFromBusyList(DphRoot, AllocatedNode);

    if Biased == 0u8 && RtlpDphShouldAllocateInPageHeap(DphRoot, Size) == 0u8 {
        UseNormalHeap = 1u8;
    } else {
        /* Now do a trick: bias the pointer and call our allocate routine */
        NewAlloc = RtlpPageHeapAllocate(
            ((HeapPtr as ULONG_PTR) | 1) as PVOID,
            Flags,
            Size,
        );
    }

    if NewAlloc.is_null() {
        /* New allocation failed, put the block back (if it was found in page heap) */
        RtlpDphPlaceOnBusyList(DphRoot, AllocatedNode);

        /* Release the lock */
        RtlpDphPostProcessing(DphRoot);

        /* Perform validation again if required */
        if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 {
            RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
        }

        /* Generate an exception if needed */
        if Flags & HEAP_GENERATE_EXCEPTIONS != 0 {
            RtlpDphRaiseException(STATUS_NO_MEMORY);
        }

        return core::ptr::null_mut();
    }

    /* Copy contents of the old block */
    if (*AllocatedNode).nUserRequestedSize > Size {
        DataSize = Size;
    } else {
        DataSize = (*AllocatedNode).nUserRequestedSize;
    }

    if DataSize != 0 {
        core::ptr::copy_nonoverlapping(Ptr as *const u8, NewAlloc as *mut u8, DataSize as usize);
    }

    /* Copy user flags and values */
    if UseNormalHeap == 0u8 {
        /* Get the node of the new block */
        Node = RtlpDphFindBusyMemory(DphRoot, NewAlloc);

        /* Set its values/flags */
        (*Node).UserValue = (*AllocatedNode).UserValue;
        if Flags & HEAP_SETTABLE_USER_FLAGS != 0 {
            (*Node).UserFlags = Flags & HEAP_SETTABLE_USER_FLAGS;
        } else {
            (*Node).UserFlags = (*AllocatedNode).UserFlags;
        }
    }

    if OldBlockPageHeap == 0 {
        /* Weird scenario, investigate */
        ntkrnl_sys::ASSERT!(false);
    }

    /* Mark the old block as no access */
    if (*AllocatedNode).nVirtualAccessSize != 0 {
        RtlpDphProtectVm(
            (*AllocatedNode).pVirtualBlock as PVOID,
            (*AllocatedNode).nVirtualAccessSize,
            PAGE_NOACCESS,
        );
    }

    /* And place it on the free list */
    RtlpDphPlaceOnFreeList(DphRoot, AllocatedNode);

    (*AllocatedNode).StackTrace = core::ptr::null_mut();

    /* Finally allocation is done, perform validation again if required */
    if RtlpDphDebugOptions & DPH_DEBUG_INTERNAL_VALIDATE != 0 && Biased == 0u8 {
        RtlpDphInternalValidatePageHeap(DphRoot, core::ptr::null_mut(), 0);
    }

    /* Release the lock */
    RtlpDphPostProcessing(DphRoot);

    /* Return pointer to user allocation */
    NewAlloc
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapGetUserInfo(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: *mut PVOID,
    UserFlags: PULONG,
) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT;
    let Node: PDPH_HEAP_BLOCK;

    DphRoot = RtlpDphPointerFromHandle(HeapHandle) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    Flags |= (*DphRoot).HeapFlags;

    RtlpDphPreProcessing(DphRoot, Flags);

    Node = RtlpDphFindBusyMemory(DphRoot, BaseAddress);

    if Node.is_null() {
        return 0u8;
    }

    if !UserValue.is_null() {
        core::ptr::write_volatile(UserValue, (*Node).UserValue);
    }
    if !UserFlags.is_null() {
        core::ptr::write_volatile(UserFlags, (*Node).UserFlags);
    }

    RtlpDphPostProcessing(DphRoot);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapSetUserValue(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: PVOID,
) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT;
    let Node: PDPH_HEAP_BLOCK;

    DphRoot = RtlpDphPointerFromHandle(HeapHandle) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    Flags |= (*DphRoot).HeapFlags;

    RtlpDphPreProcessing(DphRoot, Flags);

    Node = RtlpDphFindBusyMemory(DphRoot, BaseAddress);

    if Node.is_null() {
        return 0u8;
    }

    (*Node).UserValue = UserValue;

    RtlpDphPostProcessing(DphRoot);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapSetUserFlags(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserFlagsReset: ULONG,
    UserFlagsSet: ULONG,
) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT;
    let Node: PDPH_HEAP_BLOCK;

    DphRoot = RtlpDphPointerFromHandle(HeapHandle) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    Flags |= (*DphRoot).HeapFlags;

    RtlpDphPreProcessing(DphRoot, Flags);

    Node = RtlpDphFindBusyMemory(DphRoot, BaseAddress);

    if Node.is_null() {
        return 0u8;
    }

    (*Node).UserFlags &= !UserFlagsReset;
    (*Node).UserFlags |= UserFlagsSet;

    RtlpDphPostProcessing(DphRoot);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapSize(
    HeapHandle: HANDLE,
    mut Flags: ULONG,
    BaseAddress: PVOID,
) -> SIZE_T {
    let DphRoot: PDPH_HEAP_ROOT;
    let Node: PDPH_HEAP_BLOCK;
    let Size: SIZE_T;

    DphRoot = RtlpDphPointerFromHandle(HeapHandle) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return -1i32 as SIZE_T;
    }

    Flags |= (*DphRoot).HeapFlags;

    RtlpDphPreProcessing(DphRoot, Flags);

    Node = RtlpDphFindBusyMemory(DphRoot, BaseAddress);

    if Node.is_null() {
        return -1i32 as SIZE_T;
    }

    Size = (*Node).nUserRequestedSize;

    RtlpDphPostProcessing(DphRoot);

    Size
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDebugPageHeapValidate(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT;
    let mut Node: PDPH_HEAP_BLOCK = core::ptr::null_mut();
    let mut Valid: BOOLEAN = 0u8;

    DphRoot = RtlpDphPointerFromHandle(HeapHandle) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0xFFu8 as BOOLEAN; // C: `return -1;` — BOOLEAN(UCHAR) sentinel 0xFF (truthy)
    }

    Flags |= (*DphRoot).HeapFlags;

    RtlpDphPreProcessing(DphRoot, Flags);

    if !BaseAddress.is_null() {
        Node = RtlpDphFindBusyMemory(DphRoot, BaseAddress);
    }

    if Node.is_null() {
        Valid = RtlpDphNormalHeapValidate(DphRoot, Flags, BaseAddress);
    }

    RtlpDphPostProcessing(DphRoot);

    if !BaseAddress.is_null() && Node.is_null() {
        return Valid;
    }

    if BaseAddress.is_null() || !Node.is_null() {
        Valid = 1u8;
    }

    Valid
}
#[no_mangle]
pub unsafe extern "system" fn RtlpDphNormalHeapValidate(
    DphRoot: PDPH_HEAP_ROOT,
    Flags: ULONG,
    BaseAddress: PVOID,
) -> BOOLEAN {
    let BlockInfo = (BaseAddress as PDPH_BLOCK_INFORMATION).offset(-1);
    if BaseAddress.is_null() {
        return RtlValidateHeap((*DphRoot).NormalHeap, Flags, core::ptr::null_mut());
    }

    RtlValidateHeap((*DphRoot).NormalHeap, Flags, BlockInfo as PVOID)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapLock(HeapPtr: HANDLE) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    RtlpDphEnterCriticalSection(DphRoot, (*DphRoot).HeapFlags);
    1u8
}
#[no_mangle]
pub unsafe extern "system" fn RtlpPageHeapUnlock(HeapPtr: HANDLE) -> BOOLEAN {
    let DphRoot: PDPH_HEAP_ROOT = RtlpDphPointerFromHandle(HeapPtr) as PDPH_HEAP_ROOT;
    if DphRoot.is_null() {
        return 0u8;
    }

    RtlpDphLeaveCriticalSection(DphRoot);
    1u8
}
