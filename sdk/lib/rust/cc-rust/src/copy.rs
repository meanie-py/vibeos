use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _DEFERRED_WRITE, _FSRTL_COMMON_FCB_HEADER, _EPROCESS, _IO_STATUS_BLOCK, _KEVENT,
    PDEFERRED_WRITE, PFILE_OBJECT, PIO_STATUS_BLOCK, PLARGE_INTEGER,
    PFSRTL_COMMON_FCB_HEADER, PPRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP,
    PROS_VACB, PMDL, PPFN_NUMBER, PEXCEPTION_POINTERS, LARGE_INTEGER, LAZY_WRITER,
    _EVENT_TYPE_NotificationEvent, _KWAIT_REASON_Executive,
    IoAllocateMdl, IoFreeMdl, IoSynchronousPageWrite, IoIsFileOriginRemote,
    MmUnmapLockedPages, MmMakeSegmentDirty,
    KeInitializeEvent, KeSetEvent, KeWaitForSingleObject,
    CcRosGetVacb, CcRosRequestVacb, CcRosReleaseVacb, CcRosEnsureVacbResident,
    CcFlushCache, CcScheduleLazyWriteScan, KeBugCheck,
    PCC_POST_DEFERRED_WRITE, MMPFNLIST,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut CcDeferredWrites: LIST_ENTRY;
    static mut CcDeferredWriteSpinLock: KSPIN_LOCK;
    static mut CcTotalDirtyPages: ULONG;
    static mut CcDirtyPageThreshold: ULONG;
    // ULONG in Mm (mminit.c:407 / ARM3/mminit.rs) — declaring these PFN_NUMBER
    // (u64 on amd64) read 4 bytes of the neighboring variable into the high
    // half, making the throttle limits garbage-huge and permanently throttling
    // every cached write (CcCanIWrite blocked forever inside WriteFile).
    static mut MmThrottleTop: ULONG;
    static mut MmThrottleBottom: ULONG;
    static mut MmModifiedPageListHead: MMPFNLIST;
    static mut CcIdleDelay: LARGE_INTEGER;
    static mut LazyWriter: LAZY_WRITER;
    fn MiZeroPhysicalPage(PageFrameIndex: PFN_NUMBER);
    fn KeAcquireSpinLockAtDpcLevel(SpinLock: *mut KSPIN_LOCK);
    fn KeReleaseSpinLockFromDpcLevel(SpinLock: *mut KSPIN_LOCK);
    fn ExInterlockedInsertHeadList(ListHead: PLIST_ENTRY, ListEntry: PLIST_ENTRY, Lock: *mut KSPIN_LOCK) -> PLIST_ENTRY;
    fn ExInterlockedInsertTailList(ListHead: PLIST_ENTRY, ListEntry: PLIST_ENTRY, Lock: *mut KSPIN_LOCK) -> PLIST_ENTRY;
    fn ExInterlockedRemoveHeadList(ListHead: PLIST_ENTRY, Lock: *mut KSPIN_LOCK) -> PLIST_ENTRY;
    fn MmRequestPageMemoryConsumer(Consumer: ULONG, CanWait: UCHAR, AllocatedPage: *mut PFN_NUMBER) -> NTSTATUS;
}
extern "C" {
    fn MmGetMdlPfnArray(Mdl: PMDL) -> PPFN_NUMBER;
}

extern "C" {
    fn RtlLongLongAdd(a: LONGLONG, b: LONGLONG, result: *mut LONGLONG) -> NTSTATUS;
}

static mut CcZeroPage: PFN_NUMBER = 0;

const MAX_ZERO_LENGTH: ULONG = 256 * 1024;
const IO_NO_INCREMENT: LONG = 0;
const FO_WRITE_THROUGH: u32 = 0x00000010;
const FSRTL_FLAG_LIMIT_MODIFIED_PAGES: u8 = 0x04;
const NODE_TYPE_DEFERRED_WRITE: i16 = 0x2FC;
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
const EXCEPTION_EXECUTE_HANDLER: i32 = 1;
const MDL_PAGES_LOCKED: i16 = 0x0002;
const MDL_MAPPED_TO_SYSTEM_VA: i16 = 0x0001;
const PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE: u32 = 0x0001;
const VACB_MAPPING_GRANULARITY: ULONG = 262144;
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005u32 as NTSTATUS;
const STATUS_PENDING: NTSTATUS = 0x00000103;
const MI_USAGE_CACHE: u32 = ntkrnl_sys::ddk::_MI_PFN_USAGES_MI_USAGE_CACHE as u32; // was 7
const MC_SYSTEM: u32 = 0;
const CACHE_MANAGER: u32 = 0x00000034;

const FirstTry: u8 = 0;
const RetryAllowRemote: u8 = 253;
const RetryForceCheckPerFile: u8 = 254;
const RetryMasterLocked: u8 = 255;
// KSPIN_LOCK_QUEUE_NUMBER values. Spelled out because the translation shipped bare
// integer literals here and they were the WRONG ones: 17 (LockQueueTimerTableLock)
// for the master lock and 6 (LockQueueNonPagedPoolLock) for the work-queue lock,
// so Cc took KiTimerTableLock[0] where the C takes CcMasterSpinLock. Never pass a
// literal to Ke*QueuedSpinLock.
const LockQueueMasterLock: i32 = 0x00000005;

#[no_mangle]
pub static mut CcRosTraceLevel: ULONG = 0;
#[no_mangle]
pub static mut CcFastMdlReadWait: ULONG = 0;
#[no_mangle]
pub static mut CcFastMdlReadNotPossible: ULONG = 0;
#[no_mangle]
pub static mut CcFastReadNotPossible: ULONG = 0;
#[no_mangle]
pub static mut CcFastReadWait: ULONG = 0;
#[no_mangle]
pub static mut CcFastReadNoWait: ULONG = 0;
#[no_mangle]
pub static mut CcFastReadResourceMiss: ULONG = 0;

#[no_mangle]
pub static mut CcDataPages: ULONG = 0;
#[no_mangle]
pub static mut CcDataFlushes: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn CcInitCacheZeroPage() {
    MI_SET_USAGE(MI_USAGE_CACHE);
    let mut page: PFN_NUMBER = 0;
    let status = MmRequestPageMemoryConsumer(MC_SYSTEM, 1u8, core::ptr::addr_of_mut!(page));
    CcZeroPage = page;
    if status < 0 {
        // crate::dbg_print!(b"Can't allocate CcZeroPage.\n\0");
        KeBugCheck(CACHE_MANAGER);
    }
    MiZeroPhysicalPage(CcZeroPage);
}

#[no_mangle]
pub unsafe extern "C" fn CcPostDeferredWrites() {
    let mut to_insert_back: LIST_ENTRY = core::mem::zeroed();
    InitializeListHead(core::ptr::addr_of_mut!(to_insert_back));

    loop {
        let list_entry = ExInterlockedRemoveHeadList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );

        if list_entry.is_null() {
            break;
        }

        let deferred_write = (list_entry as usize - core::mem::offset_of!(_DEFERRED_WRITE, DeferredWriteLinks)) as PDEFERRED_WRITE;

        if CcCanIWrite(
            (*deferred_write).FileObject,
            (*deferred_write).BytesToWrite,
            0u8,
            RetryForceCheckPerFile,
        ) != 0 {
            if !(*deferred_write).Event.is_null() {
                KeSetEvent((*deferred_write).Event, IO_NO_INCREMENT, 0u8);
            } else {
                ((*deferred_write).PostRoutine.unwrap())(
                    (*deferred_write).Context1,
                    (*deferred_write).Context2,
                );
                ExFreePoolWithTag(deferred_write as PVOID, u32::from_ne_bytes(*b"CcDw"));
            }
            continue;
        }

        InsertHeadList(
            core::ptr::addr_of_mut!(to_insert_back),
            core::ptr::addr_of_mut!((*deferred_write).DeferredWriteLinks) as PLIST_ENTRY,
        );

        if (*deferred_write).LimitModifiedPages == 0 {
            break;
        }
    }

    while !IsListEmpty(core::ptr::addr_of_mut!(to_insert_back)) {
        let list_entry = RemoveTailList(core::ptr::addr_of_mut!(to_insert_back));
        ExInterlockedInsertHeadList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            list_entry,
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn CcPerformReadAhead(file_object: PFILE_OBJECT) {
    let file_object = opaque_ptr(file_object);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
    let mut locked = 0u8;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    let private_cache_map = (*file_object).PrivateCacheMap as PPRIVATE_CACHE_MAP;
    let (current_offset, length) = if private_cache_map.is_null() {
        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
        ObDereferenceObject(file_object as PVOID);
        return;
    } else {
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _);
        let offset = (*private_cache_map).ReadAheadOffset[1].QuadPart;
        let len = (*private_cache_map).ReadAheadLength[1];
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _);
        (offset, len)
    };
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    let mut current_offset = current_offset;
    let mut length = length;

    if ((*(*shared_cache_map).Callbacks).AcquireForReadAhead.unwrap())(
        (*shared_cache_map).LazyWriteContext,
        0u8,
    ) == 0 {
        locked = 0;
    } else {
        locked = 1;

        if current_offset >= (*shared_cache_map).FileSize.QuadPart {
        } else {
            if current_offset + length as LONGLONG > (*shared_cache_map).FileSize.QuadPart {
                length = ((*shared_cache_map).FileSize.QuadPart - current_offset) as ULONG;
            }

            let mut partial_length = (current_offset % VACB_MAPPING_GRANULARITY as LONGLONG) as ULONG;
            if partial_length != 0 {
                partial_length = core::cmp::min(length, VACB_MAPPING_GRANULARITY - partial_length);
                let mut vacb: PROS_VACB = core::ptr::null_mut();
                let status = CcRosRequestVacb(
                    shared_cache_map,
                    (current_offset / VACB_MAPPING_GRANULARITY as LONGLONG) * VACB_MAPPING_GRANULARITY as LONGLONG,
                    core::ptr::addr_of_mut!(vacb),
                );
                if status >= 0 {
                    let success = CcRosEnsureVacbResident(
                        vacb,
                        1u8,
                        0u8,
                        (current_offset % VACB_MAPPING_GRANULARITY as LONGLONG) as ULONG,
                        partial_length,
                    );

                    CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);

                    if success != 0 {
                        length -= partial_length;
                        current_offset += partial_length as LONGLONG;
                    }
                }
            }

            while length > 0 {
                partial_length = core::cmp::min(VACB_MAPPING_GRANULARITY, length);
                let mut vacb: PROS_VACB = core::ptr::null_mut();
                let status = CcRosRequestVacb(shared_cache_map, current_offset, core::ptr::addr_of_mut!(vacb));
                if status >= 0 {
                    let success = CcRosEnsureVacbResident(vacb, 1u8, 0u8, 0, partial_length);

                    CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);

                    if success != 0 {
                        length -= partial_length;
                        current_offset += partial_length as LONGLONG;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    let private_cache_map = (*file_object).PrivateCacheMap as PPRIVATE_CACHE_MAP;
    if !private_cache_map.is_null() {
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _);
        core::sync::atomic::AtomicU32::from_ptr(
            core::ptr::addr_of_mut!((*private_cache_map).__bindgen_anon_1.UlongFlags)
        ).fetch_and(!PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE, core::sync::atomic::Ordering::SeqCst);
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _);
    }
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    if locked != 0 {
        ((*(*shared_cache_map).Callbacks).ReleaseFromReadAhead.unwrap())((*shared_cache_map).LazyWriteContext);
    }

    ObDereferenceObject(file_object as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CcCanIWrite(
    file_object: PFILE_OBJECT,
    bytes_to_write: ULONG,
    wait: BOOLEAN,
    retrying: BOOLEAN,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    if ((*file_object).Flags & FO_WRITE_THROUGH) != 0 {
        return 1;
    }

    let try_context = retrying;
    if IoIsFileOriginRemote(file_object) != 0 && try_context < RetryAllowRemote {
        return 1;
    }

    let length = if bytes_to_write < MAX_ZERO_LENGTH {
        bytes_to_write
    } else {
        MAX_ZERO_LENGTH
    };

    let pages = BYTES_TO_PAGES(length as usize);

    let mut per_file_defer = 0u8;
    let fcb = (*file_object).FsContext as PFSRTL_COMMON_FCB_HEADER;
    if try_context >= RetryForceCheckPerFile || ((*fcb).Flags & FSRTL_FLAG_LIMIT_MODIFIED_PAGES) != 0 {
        let old_irql = if try_context != RetryMasterLocked {
            KeAcquireQueuedSpinLock(LockQueueMasterLock as _)
        } else {
            0
        };

        if !(*file_object).SectionObjectPointer.is_null() &&
           !(*(*file_object).SectionObjectPointer).SharedCacheMap.is_null() {
            let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
            if (*shared_cache_map).DirtyPageThreshold != 0 && (*shared_cache_map).DirtyPages != 0 {
                if pages + (*shared_cache_map).DirtyPages as usize > (*shared_cache_map).DirtyPageThreshold as usize {
                    per_file_defer = 1;
                }
            }
        }

        if try_context != RetryMasterLocked {
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
        }
    }

    if (try_context != FirstTry || IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites))) &&
       (CcTotalDirtyPages as usize + pages) < CcDirtyPageThreshold as usize &&
       (MmAvailablePages > MmThrottleTop as PFN_NUMBER ||
        ((*core::ptr::addr_of!(MmModifiedPageListHead)).Total < 1000 && MmAvailablePages > MmThrottleBottom as PFN_NUMBER)) &&
       per_file_defer == 0 {
        return 1;
    }

    {
        extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
        DbgPrint(
            b"[CC] CanIWrite THROTTLED: dirty=%u thresh=%u avail=%u top=%u bot=%u modtot=%u perfile=%u try=%u wait=%u\n\0".as_ptr() as *const i8,
            CcTotalDirtyPages as u32, CcDirtyPageThreshold as u32,
            MmAvailablePages as u32, MmThrottleTop as u32, MmThrottleBottom as u32,
            (*core::ptr::addr_of!(MmModifiedPageListHead)).Total as u32,
            per_file_defer as u32, try_context as u32, wait as u32,
        );
    }

    if wait == 0 {
        return 0;
    }

    if IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites)) {
        let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
        CcScheduleLazyWriteScan(1u8);
        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
    }

    let mut wait_event: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(
        core::ptr::addr_of_mut!(wait_event),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );

    let mut context: _DEFERRED_WRITE = core::mem::zeroed();
    context.NodeTypeCode = NODE_TYPE_DEFERRED_WRITE;
    context.NodeByteSize = core::mem::size_of::<_DEFERRED_WRITE>() as i16;
    context.FileObject = file_object;
    context.BytesToWrite = bytes_to_write;
    context.LimitModifiedPages = if ((*fcb).Flags & FSRTL_FLAG_LIMIT_MODIFIED_PAGES) != 0 { 1 } else { 0 };
    context.Event = core::ptr::addr_of_mut!(wait_event);

    if retrying != 0 {
        ExInterlockedInsertHeadList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            core::ptr::addr_of_mut!(context.DeferredWriteLinks) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );
    } else {
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            core::ptr::addr_of_mut!(context.DeferredWriteLinks) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );
    }

    loop {
        CcPostDeferredWrites();
        if KeWaitForSingleObject(
            core::ptr::addr_of_mut!(wait_event) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::addr_of_mut!(CcIdleDelay),
        ) == STATUS_SUCCESS {
            break;
        }
    }

    1
}

unsafe fn CcpCheckInvalidUserBuffer(
    except: PEXCEPTION_POINTERS,
    buffer: PVOID,
    length: ULONG,
) -> i32 {
    let begin_address = buffer as usize;
    let end_address = begin_address + length as usize;

    if (*(*except).ExceptionRecord).ExceptionCode != STATUS_ACCESS_VIOLATION {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    if (*(*except).ExceptionRecord).NumberParameters < 2 {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    let exception_address = (*(*except).ExceptionRecord).ExceptionInformation[1] as usize;
    if exception_address >= begin_address && exception_address < end_address {
        return EXCEPTION_EXECUTE_HANDLER;
    }

    EXCEPTION_CONTINUE_SEARCH
}

#[no_mangle]
pub unsafe extern "system" fn CcCopyRead(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    wait: BOOLEAN,
    buffer: PVOID,
    io_status: PIO_STATUS_BLOCK,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let buffer = opaque_ptr(buffer);
    let io_status = opaque_ptr(io_status);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (shared_cache_map as PVOID).is_null() {
        return 0;
    }

    let read_end = (*file_offset).QuadPart + length as LONGLONG;
    let mut read_length = 0u32;
    let mut current_offset = (*file_offset).QuadPart;

    while current_offset < read_end {
        let mut vacb: PROS_VACB = core::ptr::null_mut();
        let status = CcRosGetVacb(shared_cache_map, current_offset, core::ptr::addr_of_mut!(vacb));
        if status < 0 {
            // DUMMY_PSEH: ExRaiseStatus would be caught by caller's SEH — return FALSE instead
            return 0;
        }

        let vacb_offset = (current_offset % VACB_MAPPING_GRANULARITY as LONGLONG) as ULONG;
        let vacb_length = core::cmp::min(length - read_length, VACB_MAPPING_GRANULARITY - vacb_offset);

        if CcRosEnsureVacbResident(vacb, wait, 0u8, vacb_offset, vacb_length) == 0 {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            ((*vacb).BaseAddress as usize + vacb_offset as usize) as *const u8,
            (buffer as usize + read_length as usize) as *mut u8,
            vacb_length as usize,
        );

        read_length += vacb_length;
        current_offset += vacb_length as LONGLONG;

        CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
    }

    (*io_status).__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*io_status).Information = read_length as _;

    1
}

#[no_mangle]
pub unsafe extern "system" fn CcCopyWrite(
    file_object: PFILE_OBJECT,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    wait: BOOLEAN,
    buffer: PVOID,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let file_offset = opaque_ptr(file_offset);
    let buffer = opaque_ptr(buffer);
    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (shared_cache_map as PVOID).is_null() {
        return 0;
    }

    let mut write_end = 0i64;
    let status = RtlLongLongAdd((*file_offset).QuadPart, length as LONGLONG, core::ptr::addr_of_mut!(write_end));
    if status < 0 {
        // DUMMY_PSEH: return FALSE instead of raising
        return 0;
    }

    let mut current_offset = (*file_offset).QuadPart;
    let mut buffer_ptr = buffer;

    while current_offset < write_end {
        let vacb_offset = (current_offset % VACB_MAPPING_GRANULARITY as LONGLONG) as ULONG;
        let vacb_length = core::cmp::min((write_end - current_offset) as ULONG, VACB_MAPPING_GRANULARITY - vacb_offset);

        let mut vacb: PROS_VACB = core::ptr::null_mut();
        let status = CcRosGetVacb(shared_cache_map, current_offset, core::ptr::addr_of_mut!(vacb));
        if status < 0 {
            // DUMMY_PSEH: return FALSE instead of raising
            return 0;
        }

        let resident = CcRosEnsureVacbResident(vacb, wait, 0u8, vacb_offset, vacb_length);
        if resident == 0 {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            buffer_ptr as *const u8,
            (((*vacb).BaseAddress as usize) + vacb_offset as usize) as *mut u8,
            vacb_length as usize,
        );

        buffer_ptr = (buffer_ptr as usize + vacb_length as usize) as PVOID;
        current_offset += vacb_length as LONGLONG;

        let status = MmMakeSegmentDirty(
            (*file_object).SectionObjectPointer,
            (*vacb).FileOffset.QuadPart + vacb_offset as LONGLONG,
            vacb_length,
        );

        CcRosReleaseVacb(shared_cache_map, vacb, 1u8, 0u8);

        if status < 0 {
            // DUMMY_PSEH: return FALSE instead of raising
            return 0;
        }
    }

    if ((*file_object).Flags & FO_WRITE_THROUGH) != 0 {
        CcFlushCache((*file_object).SectionObjectPointer, file_offset, length, core::ptr::null_mut());
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CcDeferWrite(
    file_object: PFILE_OBJECT,
    post_routine: PCC_POST_DEFERRED_WRITE,
    context1: PVOID,
    context2: PVOID,
    bytes_to_write: ULONG,
    retrying: BOOLEAN,
) {
    let file_object = opaque_ptr(file_object);
    let context1 = opaque_ptr(context1);
    let context2 = opaque_ptr(context2);
    let context = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_DEFERRED_WRITE>(), u32::from_ne_bytes(*b"CcDw")) as PDEFERRED_WRITE;
    if context.is_null() {
        (post_routine.unwrap())(context1, context2);
        return;
    }

    let fcb = (*file_object).FsContext as PFSRTL_COMMON_FCB_HEADER;

    core::ptr::write_bytes(context as *mut u8, 0, core::mem::size_of::<_DEFERRED_WRITE>());
    (*context).NodeTypeCode = NODE_TYPE_DEFERRED_WRITE;
    (*context).NodeByteSize = core::mem::size_of::<_DEFERRED_WRITE>() as i16;
    (*context).FileObject = file_object;
    (*context).PostRoutine = post_routine;
    (*context).Context1 = context1;
    (*context).Context2 = context2;
    (*context).BytesToWrite = bytes_to_write;
    (*context).LimitModifiedPages = if ((*fcb).Flags & FSRTL_FLAG_LIMIT_MODIFIED_PAGES) != 0 { 1 } else { 0 };

    if retrying != 0 {
        ExInterlockedInsertHeadList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            core::ptr::addr_of_mut!((*context).DeferredWriteLinks) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );
    } else {
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!(CcDeferredWrites),
            core::ptr::addr_of_mut!((*context).DeferredWriteLinks) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(CcDeferredWriteSpinLock) as *mut _,
        );
    }

    CcPostDeferredWrites();

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    if (*core::ptr::addr_of!(LazyWriter)).ScanActive == 0 {
        CcScheduleLazyWriteScan(0u8);
    }
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn CcFastCopyRead(
    file_object: PFILE_OBJECT,
    file_offset: ULONG,
    length: ULONG,
    page_count: ULONG,
    buffer: PVOID,
    io_status: PIO_STATUS_BLOCK,
) {
    let file_object = opaque_ptr(file_object);
    let buffer = opaque_ptr(buffer);
    let io_status = opaque_ptr(io_status);
    let mut large_file_offset: LARGE_INTEGER = core::mem::zeroed();
    large_file_offset.QuadPart = file_offset as i64;
    CcCopyRead(file_object, core::ptr::addr_of_mut!(large_file_offset), length, 1u8, buffer, io_status);
}

#[no_mangle]
pub unsafe extern "system" fn CcFastCopyWrite(
    file_object: PFILE_OBJECT,
    file_offset: ULONG,
    length: ULONG,
    buffer: PVOID,
) {
    let file_object = opaque_ptr(file_object);
    let buffer = opaque_ptr(buffer);
    let mut large_file_offset: LARGE_INTEGER = core::mem::zeroed();
    large_file_offset.QuadPart = file_offset as i64;
    CcCopyWrite(file_object, core::ptr::addr_of_mut!(large_file_offset), length, 1u8, buffer);
}

#[no_mangle]
pub unsafe extern "system" fn CcZeroData(
    file_object: PFILE_OBJECT,
    start_offset: PLARGE_INTEGER,
    end_offset: PLARGE_INTEGER,
    wait: BOOLEAN,
) -> BOOLEAN {
    let file_object = opaque_ptr(file_object);
    let start_offset = opaque_ptr(start_offset);
    let end_offset = opaque_ptr(end_offset);
    let mut length = (*end_offset).QuadPart - (*start_offset).QuadPart;
    let mut write_offset: LARGE_INTEGER = *start_offset;

    let shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    if (shared_cache_map as PVOID).is_null() {
        let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut event: _KEVENT = core::mem::zeroed();

        let mdl = IoAllocateMdl(
            core::ptr::null_mut(),
            core::cmp::min(length as ULONG, MAX_ZERO_LENGTH),
            0u8,
            0u8,
            core::ptr::null_mut(),
        );
        if mdl.is_null() {
            // DUMMY_PSEH: return FALSE instead of raising
            return 0;
        }

        let pfn_array = MmGetMdlPfnArray(mdl);
        for i in 0..BYTES_TO_PAGES((*mdl).ByteCount as usize) {
            *pfn_array.add(i) = CcZeroPage as _;
        }
        (*mdl).MdlFlags |= MDL_PAGES_LOCKED as i16;

        while length > 0 {
            let current_length = core::cmp::min(length as ULONG, MAX_ZERO_LENGTH);
            (*mdl).ByteCount = current_length;

            KeInitializeEvent(core::ptr::addr_of_mut!(event), _EVENT_TYPE_NotificationEvent, 0u8);
            let mut status = IoSynchronousPageWrite(
                file_object,
                mdl,
                core::ptr::addr_of_mut!(write_offset),
                core::ptr::addr_of_mut!(event),
                core::ptr::addr_of_mut!(iosb),
            );
            if status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(event) as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
                status = iosb.__bindgen_anon_1.Status;
            }
            if ((*mdl).MdlFlags & MDL_MAPPED_TO_SYSTEM_VA) != 0 {
                MmUnmapLockedPages((*mdl).MappedSystemVa, mdl);
            }
            if status < 0 {
                IoFreeMdl(mdl);
                // DUMMY_PSEH: return FALSE instead of raising
                return 0;
            }
            write_offset.QuadPart += current_length as LONGLONG;
            length -= current_length as LONGLONG;
        }

        IoFreeMdl(mdl);
        return 1;
    }

    if (*start_offset).QuadPart < (*shared_cache_map).ValidDataLength.QuadPart &&
       (*end_offset).QuadPart >= (*shared_cache_map).ValidDataLength.QuadPart {
        (*shared_cache_map).ValidDataLength = *start_offset;
        return 1;
    }

    while write_offset.QuadPart < (*end_offset).QuadPart {
        let vacb_offset = (write_offset.QuadPart % VACB_MAPPING_GRANULARITY as LONGLONG) as ULONG;
        let vacb_length = core::cmp::min(length as ULONG, VACB_MAPPING_GRANULARITY - vacb_offset);

        let mut vacb: PROS_VACB = core::ptr::null_mut();
        let status = CcRosGetVacb(shared_cache_map, write_offset.QuadPart, core::ptr::addr_of_mut!(vacb));
        if status < 0 {
            // DUMMY_PSEH: return FALSE instead of raising
            return 0;
        }

        let resident = CcRosEnsureVacbResident(vacb, wait, 0u8, vacb_offset, vacb_length);
        if resident == 0 {
            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
            return 0;
        }

        core::ptr::write_bytes(
            (((*vacb).BaseAddress as usize) + vacb_offset as usize) as *mut u8,
            0,
            vacb_length as usize,
        );

        write_offset.QuadPart += vacb_length as LONGLONG;
        length -= vacb_length as LONGLONG;

        let status = MmMakeSegmentDirty(
            (*file_object).SectionObjectPointer,
            (*vacb).FileOffset.QuadPart + vacb_offset as LONGLONG,
            vacb_length,
        );

        CcRosReleaseVacb(shared_cache_map, vacb, 1u8, 0u8);

        if status < 0 {
            // DUMMY_PSEH: return FALSE instead of raising
            return 0;
        }
    }

    if ((*file_object).Flags & FO_WRITE_THROUGH) != 0 {
        CcFlushCache(
            (*file_object).SectionObjectPointer,
            start_offset,
            ((*end_offset).QuadPart - (*start_offset).QuadPart) as ULONG,
            core::ptr::null_mut(),
        );
    }

    1
}
