use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcFlushCache, CcRosEnsureVacbResident, CcRosGetVacb, CcRosReleaseVacb, CcRosRequestVacb, CcScheduleLazyWriteScan, DEFERRED_WRITE, ExInterlockedInsertHeadList, ExInterlockedInsertTailList, ExInterlockedRemoveHeadList, _IO_STATUS_BLOCK, IoAllocateMdl, IoFreeMdl, IoIsFileOriginRemote, IoSynchronousPageWrite, KEVENT, KeAcquireSpinLockAtDpcLevel, KeBugCheck, KeInitializeEvent, KeReleaseSpinLockFromDpcLevel, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, MiZeroPhysicalPage, MmMakeSegmentDirty, MmRequestPageMemoryConsumer, MmUnmapLockedPages, PCC_POST_DEFERRED_WRITE, PDEFERRED_WRITE, PEXCEPTION_POINTERS, PFILE_OBJECT, PFSRTL_COMMON_FCB_HEADER, PIO_STATUS_BLOCK, PLARGE_INTEGER, PMDL, PPFN_NUMBER, PPRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP, PROS_VACB};
use core::ffi::c_void;

extern "C" {
    fn CCTRACE(level: u32, format: *const u8, ...);
    fn min(a: u32, b: u32) -> u32;
    fn MmGetMdlPfnArray(Mdl: PMDL) -> PPFN_NUMBER;
    static CcZeroPage: usize;
    fn DBG_UNREFERENCED_PARAMETER(x: u32);
    fn BooleanFlagOn(Flags: ULONG, Mask: ULONG) -> BOOLEAN;
}
extern "C" {
    fn RtlLongLongAdd(op1: i64, op2: i64, result: *mut i64) -> i32;
    fn _SEH2_AbnormalTermination() -> u8;
}

#[no_mangle]
pub unsafe extern "system" fn CcInitCacheZeroPage() {
    let mut Status: NTSTATUS;

    MI_SET_USAGE(MI_USAGE_CACHE);
    Status = MmRequestPageMemoryConsumer(MC_SYSTEM, 1u8, core::ptr::addr_of_mut!(CcZeroPage));
    if NT_SUCCESS(Status) == 0 {
        DbgPrint(b"Can't allocate CcZeroPage.\n\0".as_ptr() as PCHAR);
        KeBugCheck(CACHE_MANAGER);
    }
    MiZeroPhysicalPage(CcZeroPage);
}

pub unsafe extern "system" fn CcPostDeferredWrites() {
    let mut ToInsertBack: LIST_ENTRY = core::mem::zeroed();

    InitializeListHead(&mut ToInsertBack);

    loop {
        let mut DeferredWrite: PDEFERRED_WRITE;
        let mut ListEntry: PLIST_ENTRY;

        DeferredWrite = core::ptr::null_mut();

        ListEntry = ExInterlockedRemoveHeadList(&mut CcDeferredWrites, &mut CcDeferredWriteSpinLock);

        if ListEntry.is_null() {
            break;
        }

        DeferredWrite = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(DEFERRED_WRITE, DeferredWriteLinks) as isize)) as PDEFERRED_WRITE;

        if CcCanIWrite((*DeferredWrite).FileObject, (*DeferredWrite).BytesToWrite, 0u8, RetryForceCheckPerFile) != 0 {
            if !(*DeferredWrite).Event.is_null() {
                KeSetEvent((*DeferredWrite).Event, 0, 0u8);
            } else {
                ((*DeferredWrite).PostRoutine)((*DeferredWrite).Context1, (*DeferredWrite).Context2);
                ExFreePoolWithTag(DeferredWrite as *mut core::ffi::c_void, 0x43634477u32);
            }
            continue;
        }

        InsertHeadList(&mut ToInsertBack, &mut (*DeferredWrite).DeferredWriteLinks);

        if (*DeferredWrite).LimitModifiedPages == 0 {
            break;
        }
    }

    while IsListEmpty(&mut ToInsertBack) == 0 {
        let ListEntry = RemoveTailList(&mut ToInsertBack);
        ExInterlockedInsertHeadList(&mut CcDeferredWrites, ListEntry, &mut CcDeferredWriteSpinLock);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcPerformReadAhead(
    FileObject: PFILE_OBJECT
) {
    let mut Status: NTSTATUS;
    let mut CurrentOffset: i64;
    let mut OldIrql: KIRQL;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let mut Vacb: PROS_VACB;
    let mut PartialLength: u32;
    let mut Length: u32;
    let PrivateCacheMap: PPRIVATE_CACHE_MAP;
    let mut Locked: u8;
    let mut Success: u8;

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    OldIrql = KeAcquireQueuedSpinLock(17i32);
    let PrivateCacheMap = (*FileObject).PrivateCacheMap;
    if PrivateCacheMap.is_null() {
        KeReleaseQueuedSpinLock(17i32, OldIrql);
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return;
    } else {
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock));
        CurrentOffset = (*PrivateCacheMap).ReadAheadOffset[1].QuadPart;
        Length = (*PrivateCacheMap).ReadAheadLength[1];
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock));
    }
    KeReleaseQueuedSpinLock(17i32, OldIrql);

    if ((*(*SharedCacheMap).Callbacks).AcquireForReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext, 0u8) == 0 {
        Locked = 0u8;
        'Clear: {
            if Locked != 0 {
                ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
            }
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            return;
        }
    }

    Locked = 1u8;

    if CurrentOffset >= (*SharedCacheMap).FileSize.QuadPart {
        'Clear: {
            if Locked != 0 {
                ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
            }
            ObDereferenceObject(FileObject as *mut core::ffi::c_void);
            return;
        }
    }
    if CurrentOffset + Length as i64 > (*SharedCacheMap).FileSize.QuadPart {
        Length = ((*SharedCacheMap).FileSize.QuadPart - CurrentOffset) as u32;
    }

    PartialLength = (CurrentOffset % VACB_MAPPING_GRANULARITY as i64) as u32;
    if PartialLength != 0 {
        PartialLength = if Length < VACB_MAPPING_GRANULARITY - PartialLength {
            Length
        } else {
            VACB_MAPPING_GRANULARITY - PartialLength
        };
        Status = CcRosRequestVacb(SharedCacheMap,
                                  (CurrentOffset / VACB_MAPPING_GRANULARITY as i64) * VACB_MAPPING_GRANULARITY as i64,
                                  core::ptr::addr_of_mut!(Vacb));
        if !NT_SUCCESS(Status) {
            'Clear: {
                if Locked != 0 {
                    ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
                }
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return;
            }
        }

        Success = CcRosEnsureVacbResident(Vacb, 1u8, 0u8,
                (CurrentOffset % VACB_MAPPING_GRANULARITY as i64) as u32, PartialLength);

        if Success == 0 {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            'Clear: {
                if Locked != 0 {
                    ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
                }
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return;
            }
        }

        CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);

        Length -= PartialLength;
        CurrentOffset += PartialLength as i64;
    }

    while Length > 0 {
        PartialLength = if Length < VACB_MAPPING_GRANULARITY { Length } else { VACB_MAPPING_GRANULARITY };
        Status = CcRosRequestVacb(SharedCacheMap, CurrentOffset, core::ptr::addr_of_mut!(Vacb));
        if !NT_SUCCESS(Status) {
            'Clear: {
                if Locked != 0 {
                    ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
                }
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return;
            }
        }

        Success = CcRosEnsureVacbResident(Vacb, 1u8, 0u8, 0, PartialLength);

        if Success == 0 {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            'Clear: {
                if Locked != 0 {
                    ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
                }
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                return;
            }
        }

        CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);

        Length -= PartialLength;
        CurrentOffset += PartialLength as i64;
    }

    if Locked != 0 {
        ((*(*SharedCacheMap).Callbacks).ReleaseFromReadAhead.unwrap())((*SharedCacheMap).LazyWriteContext);
    }
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn CcCanIWrite(
    FileObject: PFILE_OBJECT,
    BytesToWrite: u32,
    Wait: u8,
    Retrying: u8,
) -> u8 {
    let mut OldIrql: KIRQL;
    let mut WaitEvent: KEVENT = core::mem::zeroed();
    let mut Length: u32;
    let mut Pages: u32;
    let mut PerFileDefer: u8;
    let mut Context: DEFERRED_WRITE = core::mem::zeroed();
    let Fcb: PFSRTL_COMMON_FCB_HEADER;
    let TryContext: i32;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    if ((*FileObject).Flags & 0x00000001) != 0 {
        return 1;
    }

    TryContext = Retrying as i32;
    if IoIsFileOriginRemote(FileObject) != 0 && TryContext < 2 {
        return 1;
    }

    Length = 0x8000;
    if BytesToWrite < 0x8000 {
        Length = BytesToWrite;
    }

    Pages = BYTES_TO_PAGES(Length as usize) as u32;

    PerFileDefer = 0;
    Fcb = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    if TryContext >= 3 || ((*Fcb).Flags & 0x00000040) != 0 {
        if TryContext != 4 {
            OldIrql = KeAcquireQueuedSpinLock(0);
        }

        if !(*FileObject).SectionObjectPointer.is_null() &&
           !(*(*FileObject).SectionObjectPointer).SharedCacheMap.is_null() {
            let SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
            if (*SharedCacheMap).DirtyPageThreshold != 0 &&
               (*SharedCacheMap).DirtyPages != 0 {
                if Pages + (*SharedCacheMap).DirtyPages > (*SharedCacheMap).DirtyPageThreshold {
                    PerFileDefer = 1;
                }
            }
        }

        if TryContext != 4 {
            KeReleaseQueuedSpinLock(0, OldIrql);
        }
    }

    if (TryContext != 0 || IsListEmpty(&mut CcDeferredWrites) != 0) &&
       CcTotalDirtyPages + Pages < CcDirtyPageThreshold &&
       (MmAvailablePages > MmThrottleTop ||
        (MmModifiedPageListHead.Total < 1000 && MmAvailablePages > MmThrottleBottom)) &&
       PerFileDefer == 0 {
        return 1;
    }

    if Wait == 0 {
        return 0;
    }

    if IsListEmpty(&mut CcDeferredWrites) != 0 {
        OldIrql = KeAcquireQueuedSpinLock(0);
        CcScheduleLazyWriteScan(1);
        KeReleaseQueuedSpinLock(0, OldIrql);
    }

    KeInitializeEvent(&mut WaitEvent, 0, 0);

    Context.NodeTypeCode = 0x02ca;
    Context.NodeByteSize = core::mem::size_of::<DEFERRED_WRITE>() as u16;
    Context.FileObject = FileObject;
    Context.BytesToWrite = BytesToWrite;
    Context.LimitModifiedPages = if ((*Fcb).Flags & 0x00000040) != 0 { 1 } else { 0 };
    Context.Event = &mut WaitEvent as *mut KEVENT;

    if Retrying != 0 {
        ExInterlockedInsertHeadList(
            &mut CcDeferredWrites,
            &mut Context.DeferredWriteLinks,
            &mut CcDeferredWriteSpinLock,
        );
    } else {
        ExInterlockedInsertTailList(
            &mut CcDeferredWrites,
            &mut Context.DeferredWriteLinks,
            &mut CcDeferredWriteSpinLock,
        );
    }

    loop {
        CcPostDeferredWrites();
        if KeWaitForSingleObject(
            &mut WaitEvent as *mut KEVENT as *mut core::ffi::c_void,
            0,
            0,
            0,
            &mut CcIdleDelay,
        ) == STATUS_SUCCESS {
            break;
        }
    }

    1
}

unsafe fn CcpCheckInvalidUserBuffer(Except: PEXCEPTION_POINTERS, Buffer: PVOID, Length: ULONG) -> i32 {
    let ExceptionAddress: ULONG_PTR;
    let BeginAddress: ULONG_PTR = Buffer as ULONG_PTR;
    let EndAddress: ULONG_PTR = (Buffer as ULONG_PTR).wrapping_add(Length as ULONG_PTR);

    if (*(*Except).ExceptionRecord).ExceptionCode != STATUS_ACCESS_VIOLATION {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    if (*(*Except).ExceptionRecord).NumberParameters < 2 {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    ExceptionAddress = (*(*Except).ExceptionRecord).ExceptionInformation[1];
    if (ExceptionAddress >= BeginAddress) && (ExceptionAddress < EndAddress) {
        return 1;
    }

    return EXCEPTION_CONTINUE_SEARCH;
}

#[no_mangle]
pub unsafe extern "system" fn CcCopyRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Wait: u8,
    Buffer: *mut core::ffi::c_void,
    IoStatus: PIO_STATUS_BLOCK,
) -> u8 {
    let SharedCacheMap: PROS_SHARED_CACHE_MAP = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    let mut Status: i32;
    let mut CurrentOffset: i64;
    let ReadEnd: i64 = (*FileOffset).QuadPart + Length as i64;
    let mut ReadLength: u32 = 0;

    CCTRACE(
        1,
        b"FileObject=%p FileOffset=%I64d Length=%lu Wait=%d\n\0".as_ptr(),
        FileObject,
        (*FileOffset).QuadPart,
        Length,
        Wait,
    );

    if SharedCacheMap.is_null() {
        return 0u8;
    }

    CurrentOffset = (*FileOffset).QuadPart;
    while CurrentOffset < ReadEnd {
        let mut Vacb: PROS_VACB = core::ptr::null_mut();
        Status = CcRosGetVacb(SharedCacheMap, CurrentOffset, &mut Vacb);
        if !NT_SUCCESS(Status) {
            ExRaiseStatus(Status);
            return 0u8;
        }

        let VacbOffset: u32 = (CurrentOffset % 262144) as u32;
        let VacbLength: u32 = min(Length, 262144 - VacbOffset);
        let CopyLength: usize = VacbLength as usize;

        if CcRosEnsureVacbResident(Vacb, Wait, 0u8, VacbOffset, VacbLength) == 0 {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            return 0u8;
        }

        RtlCopyMemory(
            Buffer,
            ((*Vacb).BaseAddress as *mut u8).add(VacbOffset as usize) as *mut core::ffi::c_void,
            CopyLength,
        );

        ReadLength += VacbLength;

        let Buffer_new = (Buffer as usize + VacbLength as usize) as *mut core::ffi::c_void;
        CurrentOffset += VacbLength as i64;
        let Length_new = Length - VacbLength;

        CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);

        Buffer = Buffer_new;
        Length = Length_new;
    }

    (*IoStatus).__bindgen_anon_1.Status = 0i32;
    (*IoStatus).__bindgen_anon_1.Information = ReadLength as usize;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CcCopyWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Wait: u8,
    Buffer: *mut core::ffi::c_void,
) -> u8 {
    let SharedCacheMap: PROS_SHARED_CACHE_MAP = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    let mut Status: i32;
    let mut CurrentOffset: i64;
    let WriteEnd: i64;

    const CC_API_DEBUG: u32 = 0;
    const VACB_MAPPING_GRANULARITY: i64 = 262144;
    const STATUS_INVALID_USER_BUFFER: i32 = 0xC00000E8u32 as i32;
    const FO_WRITE_THROUGH: u32 = 0x00000002;

    if SharedCacheMap.is_null() {
        return 0u8;
    }

    let mut write_end_temp: i64 = 0;
    Status = RtlLongLongAdd((*FileOffset).QuadPart, Length as i64, &mut write_end_temp);
    if !NT_SUCCESS(Status) {
        ExRaiseStatus(Status);
    }
    let WriteEnd = write_end_temp;

    CurrentOffset = (*FileOffset).QuadPart;
    while CurrentOffset < WriteEnd {
        let VacbOffset: u32 = (CurrentOffset % VACB_MAPPING_GRANULARITY) as u32;
        let remaining = WriteEnd - CurrentOffset;
        let max_len = (VACB_MAPPING_GRANULARITY - VacbOffset as i64) as u32;
        let VacbLength: u32 = if remaining < max_len as i64 { remaining as u32 } else { max_len };

        let mut Vacb: PROS_VACB = core::ptr::null_mut();
        Status = CcRosGetVacb(SharedCacheMap, CurrentOffset, &mut Vacb);
        if !NT_SUCCESS(Status) {
            ExRaiseStatus(Status);
            return 0u8;
        }

        if CcRosEnsureVacbResident(Vacb, Wait, 0u8, VacbOffset, VacbLength) == 0 {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            return 0u8;
        }

        core::ptr::copy_nonoverlapping(
            Buffer as *const u8,
            ((*Vacb).BaseAddress as usize + VacbOffset as usize) as *mut u8,
            VacbLength as usize,
        );

        let Buffer = (Buffer as usize + VacbLength as usize) as *mut core::ffi::c_void;
        CurrentOffset += VacbLength as i64;

        Status = MmMakeSegmentDirty(
            (*FileObject).SectionObjectPointer,
            (*Vacb).FileOffset.QuadPart + VacbOffset as i64,
            VacbLength,
        );
        if !NT_SUCCESS(Status) {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            ExRaiseStatus(Status);
        }

        CcRosReleaseVacb(SharedCacheMap, Vacb, 1u8, 0u8);
    }

    if (*FileObject).Flags & FO_WRITE_THROUGH != 0 {
        CcFlushCache((*FileObject).SectionObjectPointer, FileOffset, Length, core::ptr::null_mut());
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CcDeferWrite(
    FileObject: PFILE_OBJECT,
    PostRoutine: PCC_POST_DEFERRED_WRITE,
    Context1: PVOID,
    Context2: PVOID,
    BytesToWrite: ULONG,
    Retrying: BOOLEAN,
) {
    let mut OldIrql: KIRQL;
    let Context: PDEFERRED_WRITE;
    let Fcb: PFSRTL_COMMON_FCB_HEADER;

    CCTRACE(
        CC_API_DEBUG,
        b"FileObject=%p PostRoutine=%p Context1=%p Context2=%p BytesToWrite=%lu Retrying=%d\n\0".as_ptr() as PCHAR,
        FileObject,
        PostRoutine,
        Context1,
        Context2,
        BytesToWrite,
        Retrying,
    );

    Context = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DEFERRED_WRITE>() as SIZE_T,
        0x43634477,
    ) as PDEFERRED_WRITE;

    if Context.is_null() {
        PostRoutine.unwrap()(Context1, Context2);
        return;
    }

    Fcb = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;

    core::ptr::write_bytes(Context, 0, 1);
    (*Context).NodeTypeCode = NODE_TYPE_DEFERRED_WRITE;
    (*Context).NodeByteSize = core::mem::size_of::<DEFERRED_WRITE>() as USHORT;
    (*Context).FileObject = FileObject;
    (*Context).PostRoutine = PostRoutine;
    (*Context).Context1 = Context1;
    (*Context).Context2 = Context2;
    (*Context).BytesToWrite = BytesToWrite;
    (*Context).LimitModifiedPages = BooleanFlagOn((*Fcb).Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES);

    if Retrying != 0 {
        ExInterlockedInsertHeadList(
            &mut CcDeferredWrites,
            &mut (*Context).DeferredWriteLinks,
            &mut CcDeferredWriteSpinLock,
        );
    } else {
        ExInterlockedInsertTailList(
            &mut CcDeferredWrites,
            &mut (*Context).DeferredWriteLinks,
            &mut CcDeferredWriteSpinLock,
        );
    }

    CcPostDeferredWrites();

    OldIrql = KeAcquireQueuedSpinLock(17i32);
    if (*core::ptr::addr_of!(LazyWriter)).ScanActive == 0 {
        CcScheduleLazyWriteScan(0u8);
    }
    KeReleaseQueuedSpinLock(17i32, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn CcFastCopyRead(
    FileObject: PFILE_OBJECT,
    FileOffset: u32,
    Length: u32,
    PageCount: u32,
    Buffer: *mut core::ffi::c_void,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let mut LargeFileOffset: LARGE_INTEGER = core::mem::zeroed();
    let Success: u8;

    CCTRACE(
        0,
        b"FileObject=%p FileOffset=%lu Length=%lu PageCount=%lu Buffer=%p\n\0".as_ptr() as *const i8,
        FileObject,
        FileOffset,
        Length,
        PageCount,
        Buffer,
    );

    DBG_UNREFERENCED_PARAMETER(PageCount);

    LargeFileOffset.QuadPart = FileOffset as i64;
    Success = CcCopyRead(
        FileObject,
        &mut LargeFileOffset,
        Length,
        1u8,
        Buffer,
        IoStatus,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CcFastCopyWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: u32,
    Length: u32,
    Buffer: *mut core::ffi::c_void,
) {
    let mut LargeFileOffset: LARGE_INTEGER = core::mem::zeroed();
    let Success: u8;

    CCTRACE(
        0x00000040,
        b"FileObject=%p FileOffset=%lu Length=%lu Buffer=%p\n\0".as_ptr(),
        FileObject,
        FileOffset,
        Length,
        Buffer,
    );

    *LargeFileOffset.QuadPart_mut() = FileOffset as i64;
    Success = CcCopyWrite(
        FileObject,
        &mut LargeFileOffset,
        Length,
        1u8,
        Buffer,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CcZeroData(
    FileObject: PFILE_OBJECT,
    StartOffset: PLARGE_INTEGER,
    EndOffset: PLARGE_INTEGER,
    Wait: u8,
) -> u8 {
    let mut Status: i32;
    let mut WriteOffset: LARGE_INTEGER;
    let mut Length: i64;
    let mut Vacb: PROS_VACB;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    const CC_API_DEBUG: u32 = 1;
    const MAX_ZERO_LENGTH: u32 = 262144;
    const VACB_MAPPING_GRANULARITY: i64 = 262144;
    const MDL_PAGES_LOCKED: u16 = 0x0002;
    const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;
    const FO_WRITE_THROUGH: u32 = 0x00000002;
    const NotificationEvent: u32 = 0;
    const Executive: u32 = 0;
    const 0i32: i8 = 0;
    const STATUS_PENDING: i32 = 0x103u32 as i32;

    CCTRACE(CC_API_DEBUG, b"FileObject=%p StartOffset=%I64u EndOffset=%I64u Wait=%d\n\0".as_ptr(), FileObject, (*StartOffset).QuadPart, (*EndOffset).QuadPart, Wait);

    Length = (*EndOffset).QuadPart - (*StartOffset).QuadPart;
    WriteOffset.QuadPart = (*StartOffset).QuadPart;

    if SharedCacheMap.is_null() {
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut Event: KEVENT = core::mem::zeroed();
        let mut Mdl: PMDL;
        let mut i: u32;
        let mut CurrentLength: u32;
        let PfnArray: PPFN_NUMBER;

        let min_len = if Length < MAX_ZERO_LENGTH as i64 { Length as u32 } else { MAX_ZERO_LENGTH };
        Mdl = IoAllocateMdl(core::ptr::null_mut(), min_len, 0u8, 0u8, core::ptr::null_mut());
        if Mdl.is_null() {
            ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
        }

        PfnArray = MmGetMdlPfnArray(Mdl);
        i = 0;
        while i < BYTES_TO_PAGES((*Mdl).ByteCount) {
            *PfnArray.offset(i as isize) = CcZeroPage;
            i += 1;
        }
        (*Mdl).MdlFlags |= MDL_PAGES_LOCKED;

        while Length > 0 {
            CurrentLength = if Length < MAX_ZERO_LENGTH as i64 { Length as u32 } else { MAX_ZERO_LENGTH };

            (*Mdl).ByteCount = CurrentLength;

            KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
            Status = IoSynchronousPageWrite(FileObject, Mdl, core::ptr::addr_of_mut!(WriteOffset), core::ptr::addr_of_mut!(Event), core::ptr::addr_of_mut!(Iosb));
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void, _KWAIT_REASON_Executive, 0i32, 0u8, core::ptr::null_mut());
                Status = Iosb.__bindgen_anon_1.Status;
            }
            if ((*Mdl).MdlFlags & MDL_MAPPED_TO_SYSTEM_VA) != 0 {
                MmUnmapLockedPages((*Mdl).MappedSystemVa, Mdl);
            }
            if !NT_SUCCESS(Status) {
                IoFreeMdl(Mdl);
                ExRaiseStatus(Status);
            }
            WriteOffset.QuadPart += CurrentLength as i64;
            Length -= CurrentLength as i64;
        }

        IoFreeMdl(Mdl);

        return 1u8;
    }

    if ((*StartOffset).QuadPart < (*SharedCacheMap).ValidDataLength.QuadPart) && ((*EndOffset).QuadPart >= (*SharedCacheMap).ValidDataLength.QuadPart) {
        (*SharedCacheMap).ValidDataLength = *StartOffset;
        return 1u8;
    }

    while WriteOffset.QuadPart < (*EndOffset).QuadPart {
        let VacbOffset: u32 = (WriteOffset.QuadPart % VACB_MAPPING_GRANULARITY) as u32;
        let remaining: i64 = VACB_MAPPING_GRANULARITY - VacbOffset as i64;
        let VacbLength: u32 = if Length < remaining { Length as u32 } else { remaining as u32 };

        Status = CcRosGetVacb(SharedCacheMap, WriteOffset.QuadPart, core::ptr::addr_of_mut!(Vacb));
        if !NT_SUCCESS(Status) {
            ExRaiseStatus(Status);
            return 0u8;
        }

        if CcRosEnsureVacbResident(Vacb, Wait, 0u8, VacbOffset, VacbLength) == 0 {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            return 0u8;
        }

        RtlZeroMemory(((*Vacb).BaseAddress as usize + VacbOffset as usize) as *mut core::ffi::c_void, VacbLength as usize);

        WriteOffset.QuadPart += VacbLength as i64;
        Length -= VacbLength as i64;

        Status = MmMakeSegmentDirty((*FileObject).SectionObjectPointer, (*Vacb).FileOffset.QuadPart + VacbOffset as i64, VacbLength);
        if !NT_SUCCESS(Status) {
            CcRosReleaseVacb(SharedCacheMap, Vacb, 0u8, 0u8);
            ExRaiseStatus(Status);
        }

        CcRosReleaseVacb(SharedCacheMap, Vacb, 1u8, 0u8);
    }

    if ((*FileObject).Flags & FO_WRITE_THROUGH) != 0 {
        CcFlushCache((*FileObject).SectionObjectPointer, StartOffset, (*EndOffset).QuadPart - (*StartOffset).QuadPart, core::ptr::null_mut());
    }

    return 1u8;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/copy.c
//  * PURPOSE:         Implements cache managers copy interface
//  *
//  * PROGRAMMERS:     Some people?
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// static PFN_NUMBER CcZeroPage = 0;
//
// #define MAX_ZERO_LENGTH    (256 * 1024)
//
// typedef enum _CC_CAN_WRITE_RETRY
// {
//     FirstTry = 0,
//     RetryAllowRemote = 253,
//     RetryForceCheckPerFile = 254,
//     RetryMasterLocked = 255,
// } CC_CAN_WRITE_RETRY;
//
// ULONG CcRosTraceLevel = CC_API_DEBUG;
// ULONG CcFastMdlReadWait;
// ULONG CcFastMdlReadNotPossible;
// ULONG CcFastReadNotPossible;
// ULONG CcFastReadWait;
// ULONG CcFastReadNoWait;
// ULONG CcFastReadResourceMiss;
//
// /* Counters:
//  * - Amount of pages flushed to the disk
//  * - Number of flush operations
//  */
// ULONG CcDataPages = 0;
// ULONG CcDataFlushes = 0;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// MiZeroPhysicalPage (
//     IN PFN_NUMBER PageFrameIndex
// );
//
// VOID
// NTAPI
// CcInitCacheZeroPage (
//     VOID)
// {
//     NTSTATUS Status;
//
//     MI_SET_USAGE(MI_USAGE_CACHE);
//     //MI_SET_PROCESS2(PsGetCurrentProcess()->ImageFileName);
//     Status = MmRequestPageMemoryConsumer(MC_SYSTEM, TRUE, &CcZeroPage);
//     if (!NT_SUCCESS(Status))
//     {
//         DbgPrint("Can't allocate CcZeroPage.\n");
//         KeBugCheck(CACHE_MANAGER);
//     }
//     MiZeroPhysicalPage(CcZeroPage);
// }
//
// VOID
// CcPostDeferredWrites(VOID)
// {
//     LIST_ENTRY ToInsertBack;
//
//     InitializeListHead(&ToInsertBack);
//
//     /* We'll try to write as much as we can */
//     while (TRUE)
//     {
//         PDEFERRED_WRITE DeferredWrite;
//         PLIST_ENTRY ListEntry;
//
//         DeferredWrite = NULL;
//
//         ListEntry = ExInterlockedRemoveHeadList(&CcDeferredWrites, &CcDeferredWriteSpinLock);
//
//         if (!ListEntry)
//             break;
//
//         DeferredWrite = CONTAINING_RECORD(ListEntry, DEFERRED_WRITE, DeferredWriteLinks);
//
//         /* Check if we can write */
//         if (CcCanIWrite(DeferredWrite->FileObject, DeferredWrite->BytesToWrite, FALSE, RetryForceCheckPerFile))
//         {
//             /* If we have an event, set it and go along */
//             if (DeferredWrite->Event)
//             {
//                 KeSetEvent(DeferredWrite->Event, IO_NO_INCREMENT, FALSE);
//             }
//             /* Otherwise, call the write routine and free the context */
//             else
//             {
//                 DeferredWrite->PostRoutine(DeferredWrite->Context1, DeferredWrite->Context2);
//                 ExFreePoolWithTag(DeferredWrite, 'CcDw');
//             }
//             continue;
//         }
//
//         /* Keep it for later */
//         InsertHeadList(&ToInsertBack, &DeferredWrite->DeferredWriteLinks);
//
//         /* If we don't accept modified pages, stop here */
//         if (!DeferredWrite->LimitModifiedPages)
//         {
//             break;
//         }
//     }
//
//     /* Insert what we couldn't write back in the list */
//     while (!IsListEmpty(&ToInsertBack))
//     {
//         PLIST_ENTRY ListEntry = RemoveTailList(&ToInsertBack);
//         ExInterlockedInsertHeadList(&CcDeferredWrites, ListEntry, &CcDeferredWriteSpinLock);
//     }
// }
//
// VOID
// CcPerformReadAhead(
//     IN PFILE_OBJECT FileObject)
// {
//     NTSTATUS Status;
//     LONGLONG CurrentOffset;
//     KIRQL OldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     PROS_VACB Vacb;
//     ULONG PartialLength;
//     ULONG Length;
//     PPRIVATE_CACHE_MAP PrivateCacheMap;
//     BOOLEAN Locked;
//     BOOLEAN Success;
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//     /* Critical:
//      * PrivateCacheMap might disappear in-between if the handle
//      * to the file is closed (private is attached to the handle not to
//      * the file), so we need to lock the master lock while we deal with
//      * it. It won't disappear without attempting to lock such lock.
//      */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     PrivateCacheMap = FileObject->PrivateCacheMap;
//     /* If the handle was closed since the read ahead was scheduled, just quit */
//     if (PrivateCacheMap == NULL)
//     {
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//         ObDereferenceObject(FileObject);
//         return;
//     }
//     /* Otherwise, extract read offset and length and release private map */
//     else
//     {
//         KeAcquireSpinLockAtDpcLevel(&PrivateCacheMap->ReadAheadSpinLock);
//         CurrentOffset = PrivateCacheMap->ReadAheadOffset[1].QuadPart;
//         Length = PrivateCacheMap->ReadAheadLength[1];
//         KeReleaseSpinLockFromDpcLevel(&PrivateCacheMap->ReadAheadSpinLock);
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /* Time to go! */
//     DPRINT("Doing ReadAhead for %p\n", FileObject);
//     /* Lock the file, first */
//     if (!SharedCacheMap->Callbacks->AcquireForReadAhead(SharedCacheMap->LazyWriteContext, FALSE))
//     {
//         Locked = FALSE;
//         goto Clear;
//     }
//
//     /* Remember it's locked */
//     Locked = TRUE;
//
//     /* Don't read past the end of the file */
//     if (CurrentOffset >= SharedCacheMap->FileSize.QuadPart)
//     {
//         goto Clear;
//     }
//     if (CurrentOffset + Length > SharedCacheMap->FileSize.QuadPart)
//     {
//         Length = SharedCacheMap->FileSize.QuadPart - CurrentOffset;
//     }
//
//     /* Next of the algorithm will lock like CcCopyData with the slight
//      * difference that we don't copy data back to an user-backed buffer
//      * We just bring data into Cc
//      */
//     PartialLength = CurrentOffset % VACB_MAPPING_GRANULARITY;
//     if (PartialLength != 0)
//     {
//         PartialLength = min(Length, VACB_MAPPING_GRANULARITY - PartialLength);
//         Status = CcRosRequestVacb(SharedCacheMap,
//                                   ROUND_DOWN(CurrentOffset, VACB_MAPPING_GRANULARITY),
//                                   &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to request VACB: %lx!\n", Status);
//             goto Clear;
//         }
//
//         _SEH2_TRY
//         {
//             Success = CcRosEnsureVacbResident(Vacb, TRUE, FALSE,
//                     CurrentOffset % VACB_MAPPING_GRANULARITY, PartialLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Success = FALSE;
//         }
//         _SEH2_END
//
//         if (!Success)
//         {
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//             DPRINT1("Failed to read data: %lx!\n", Status);
//             goto Clear;
//         }
//
//         CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//
//         Length -= PartialLength;
//         CurrentOffset += PartialLength;
//     }
//
//     while (Length > 0)
//     {
//         ASSERT(CurrentOffset % VACB_MAPPING_GRANULARITY == 0);
//         PartialLength = min(VACB_MAPPING_GRANULARITY, Length);
//         Status = CcRosRequestVacb(SharedCacheMap,
//                                   CurrentOffset,
//                                   &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to request VACB: %lx!\n", Status);
//             goto Clear;
//         }
//
//         _SEH2_TRY
//         {
//             Success = CcRosEnsureVacbResident(Vacb, TRUE, FALSE, 0, PartialLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Success = FALSE;
//         }
//         _SEH2_END
//
//         if (!Success)
//         {
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//             DPRINT1("Failed to read data: %lx!\n", Status);
//             goto Clear;
//         }
//
//         CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//
//         Length -= PartialLength;
//         CurrentOffset += PartialLength;
//     }
//
// Clear:
//     /* See previous comment about private cache map */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     PrivateCacheMap = FileObject->PrivateCacheMap;
//     if (PrivateCacheMap != NULL)
//     {
//         /* Mark read ahead as unactive */
//         KeAcquireSpinLockAtDpcLevel(&PrivateCacheMap->ReadAheadSpinLock);
//         InterlockedAnd((volatile long *)&PrivateCacheMap->UlongFlags, ~PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE);
//         KeReleaseSpinLockFromDpcLevel(&PrivateCacheMap->ReadAheadSpinLock);
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /* If file was locked, release it */
//     if (Locked)
//     {
//         SharedCacheMap->Callbacks->ReleaseFromReadAhead(SharedCacheMap->LazyWriteContext);
//     }
//
//     /* And drop our extra reference (See: CcScheduleReadAhead) */
//     ObDereferenceObject(FileObject);
//
//     return;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcCanIWrite (
//     IN PFILE_OBJECT FileObject,
//     IN ULONG BytesToWrite,
//     IN BOOLEAN Wait,
//     IN BOOLEAN Retrying)
// {
//     KIRQL OldIrql;
//     KEVENT WaitEvent;
//     ULONG Length, Pages;
//     BOOLEAN PerFileDefer;
//     DEFERRED_WRITE Context;
//     PFSRTL_COMMON_FCB_HEADER Fcb;
//     CC_CAN_WRITE_RETRY TryContext;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p BytesToWrite=%lu Wait=%d Retrying=%d\n",
//         FileObject, BytesToWrite, Wait, Retrying);
//
//     /* Write through is always OK */
//     if (BooleanFlagOn(FileObject->Flags, FO_WRITE_THROUGH))
//     {
//         return TRUE;
//     }
//
//     TryContext = Retrying;
//     /* Allow remote file if not from posted */
//     if (IoIsFileOriginRemote(FileObject) && TryContext < RetryAllowRemote)
//     {
//         return TRUE;
//     }
//
//     /* Don't exceed max tolerated size */
//     Length = MAX_ZERO_LENGTH;
//     if (BytesToWrite < MAX_ZERO_LENGTH)
//     {
//         Length = BytesToWrite;
//     }
//
//     Pages = BYTES_TO_PAGES(Length);
//
//     /* By default, assume limits per file won't be hit */
//     PerFileDefer = FALSE;
//     Fcb = FileObject->FsContext;
//     /* Do we have to check for limits per file? */
//     if (TryContext >= RetryForceCheckPerFile ||
//         BooleanFlagOn(Fcb->Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES))
//     {
//         /* If master is not locked, lock it now */
//         if (TryContext != RetryMasterLocked)
//         {
//             OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//         }
//
//         /* Let's not assume the file is cached... */
//         if (FileObject->SectionObjectPointer != NULL &&
//             FileObject->SectionObjectPointer->SharedCacheMap != NULL)
//         {
//             SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//             /* Do we have limits per file set? */
//             if (SharedCacheMap->DirtyPageThreshold != 0 &&
//                 SharedCacheMap->DirtyPages != 0)
//             {
//                 /* Yes, check whether they are blocking */
//                 if (Pages + SharedCacheMap->DirtyPages > SharedCacheMap->DirtyPageThreshold)
//                 {
//                     PerFileDefer = TRUE;
//                 }
//             }
//         }
//
//         /* And don't forget to release master */
//         if (TryContext != RetryMasterLocked)
//         {
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//         }
//     }
//
//     /* So, now allow write if:
//      * - Not the first try or we have no throttling yet
//      * AND:
//      * - We don't exceed threshold!
//      * - We don't exceed what Mm can allow us to use
//      *   + If we're above top, that's fine
//      *   + If we're above bottom with limited modified pages, that's fine
//      *   + Otherwise, throttle!
//      */
//     if ((TryContext != FirstTry || IsListEmpty(&CcDeferredWrites)) &&
//         CcTotalDirtyPages + Pages < CcDirtyPageThreshold &&
//         (MmAvailablePages > MmThrottleTop ||
//          (MmModifiedPageListHead.Total < 1000 && MmAvailablePages > MmThrottleBottom)) &&
//         !PerFileDefer)
//     {
//         return TRUE;
//     }
//
//     /* If we can wait, we'll start the wait loop for waiting till we can
//      * write for real
//      */
//     if (!Wait)
//     {
//         return FALSE;
//     }
//
//     /* Otherwise, if there are no deferred writes yet, start the lazy writer */
//     if (IsListEmpty(&CcDeferredWrites))
//     {
//         KIRQL OldIrql;
//
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//         CcScheduleLazyWriteScan(TRUE);
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//     }
//
//     /* Initialize our wait event */
//     KeInitializeEvent(&WaitEvent, NotificationEvent, FALSE);
//
//     /* And prepare a dummy context */
//     Context.NodeTypeCode = NODE_TYPE_DEFERRED_WRITE;
//     Context.NodeByteSize = sizeof(DEFERRED_WRITE);
//     Context.FileObject = FileObject;
//     Context.BytesToWrite = BytesToWrite;
//     Context.LimitModifiedPages = BooleanFlagOn(Fcb->Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES);
//     Context.Event = &WaitEvent;
//
//     /* And queue it */
//     if (Retrying)
//     {
//         /* To the top, if that's a retry */
//         ExInterlockedInsertHeadList(&CcDeferredWrites,
//                                     &Context.DeferredWriteLinks,
//                                     &CcDeferredWriteSpinLock);
//     }
//     else
//     {
//         /* To the bottom, if that's a first time */
//         ExInterlockedInsertTailList(&CcDeferredWrites,
//                                     &Context.DeferredWriteLinks,
//                                     &CcDeferredWriteSpinLock);
//     }
//
// #if DBG
//     DPRINT1("Actively deferring write for: %p\n", FileObject);
//     DPRINT1("Because:\n");
//     if (CcTotalDirtyPages + Pages >= CcDirtyPageThreshold)
//         DPRINT1("    There are too many cache dirty pages: %x + %x >= %x\n", CcTotalDirtyPages, Pages, CcDirtyPageThreshold);
//     if (MmAvailablePages <= MmThrottleTop)
//         DPRINT1("    Available pages are below throttle top: %lx <= %lx\n", MmAvailablePages, MmThrottleTop);
//     if (MmModifiedPageListHead.Total >= 1000)
//         DPRINT1("    There are too many modified pages: %lu >= 1000\n", MmModifiedPageListHead.Total);
//     if (MmAvailablePages <= MmThrottleBottom)
//         DPRINT1("    Available pages are below throttle bottom: %lx <= %lx\n", MmAvailablePages, MmThrottleBottom);
// #endif
//     /* Now, we'll loop until our event is set. When it is set, it means that caller
//      * can immediately write, and has to
//      */
//     do
//     {
//         CcPostDeferredWrites();
//     } while (KeWaitForSingleObject(&WaitEvent, Executive, KernelMode, FALSE, &CcIdleDelay) != STATUS_SUCCESS);
//
//     return TRUE;
// }
//
// static
// int
// CcpCheckInvalidUserBuffer(PEXCEPTION_POINTERS Except, PVOID Buffer, ULONG Length)
// {
//     ULONG_PTR ExceptionAddress;
//     ULONG_PTR BeginAddress = (ULONG_PTR)Buffer;
//     ULONG_PTR EndAddress = (ULONG_PTR)Buffer + Length;
//
//     if (Except->ExceptionRecord->ExceptionCode != STATUS_ACCESS_VIOLATION)
//         return EXCEPTION_CONTINUE_SEARCH;
//     if (Except->ExceptionRecord->NumberParameters < 2)
//         return EXCEPTION_CONTINUE_SEARCH;
//
//     ExceptionAddress = Except->ExceptionRecord->ExceptionInformation[1];
//     if ((ExceptionAddress >= BeginAddress) && (ExceptionAddress < EndAddress))
//         return EXCEPTION_EXECUTE_HANDLER;
//
//     return EXCEPTION_CONTINUE_SEARCH;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// CcCopyRead (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN BOOLEAN Wait,
//     OUT PVOID Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus)
// {
//     PROS_VACB Vacb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     NTSTATUS Status;
//     LONGLONG CurrentOffset;
//     LONGLONG ReadEnd = FileOffset->QuadPart + Length;
//     ULONG ReadLength = 0;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%I64d Length=%lu Wait=%d\n",
//         FileObject, FileOffset->QuadPart, Length, Wait);
//
//     DPRINT("CcCopyRead(FileObject 0x%p, FileOffset %I64x, "
//            "Length %lu, Wait %u, Buffer 0x%p, IoStatus 0x%p)\n",
//            FileObject, FileOffset->QuadPart, Length, Wait,
//            Buffer, IoStatus);
//
//     if (!SharedCacheMap)
//         return FALSE;
//
//     /* Documented to ASSERT, but KMTests test this case... */
//     // ASSERT((FileOffset->QuadPart + Length) <= SharedCacheMap->FileSize.QuadPart);
//
//     CurrentOffset = FileOffset->QuadPart;
//     while(CurrentOffset < ReadEnd)
//     {
//         Status = CcRosGetVacb(SharedCacheMap, CurrentOffset, &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             ExRaiseStatus(Status);
//             return FALSE;
//         }
//
//         _SEH2_TRY
//         {
//             ULONG VacbOffset = CurrentOffset % VACB_MAPPING_GRANULARITY;
//             ULONG VacbLength = min(Length, VACB_MAPPING_GRANULARITY - VacbOffset);
//             SIZE_T CopyLength = VacbLength;
//
//             if (!CcRosEnsureVacbResident(Vacb, Wait, FALSE, VacbOffset, VacbLength))
//                 return FALSE;
//
//             _SEH2_TRY
//             {
//                 RtlCopyMemory(Buffer, (PUCHAR)Vacb->BaseAddress + VacbOffset, CopyLength);
//             }
//             _SEH2_EXCEPT(CcpCheckInvalidUserBuffer(_SEH2_GetExceptionInformation(), Buffer, VacbLength))
//             {
//                 ExRaiseStatus(STATUS_INVALID_USER_BUFFER);
//             }
//             _SEH2_END;
//
//             ReadLength += VacbLength;
//
//             Buffer = (PVOID)((ULONG_PTR)Buffer + VacbLength);
//             CurrentOffset += VacbLength;
//             Length -= VacbLength;
//         }
//         _SEH2_FINALLY
//         {
//             CcRosReleaseVacb(SharedCacheMap, Vacb, FALSE, FALSE);
//         }
//         _SEH2_END;
//     }
//
//     IoStatus->Status = STATUS_SUCCESS;
//     IoStatus->Information = ReadLength;
//
// #if 0
//     /* If that was a successful sync read operation, let's handle read ahead */
//     if (Length == 0 && Wait)
//     {
//         PPRIVATE_CACHE_MAP PrivateCacheMap = FileObject->PrivateCacheMap;
//
//         /* If file isn't random access and next read may get us cross VACB boundary,
//          * schedule next read
//          */
//         if (!BooleanFlagOn(FileObject->Flags, FO_RANDOM_ACCESS) &&
//             (CurrentOffset - 1) / VACB_MAPPING_GRANULARITY != (CurrentOffset + ReadLength - 1) / VACB_MAPPING_GRANULARITY)
//         {
//             CcScheduleReadAhead(FileObject, FileOffset, ReadLength);
//         }
//
//         /* And update read history in private cache map */
//         PrivateCacheMap->FileOffset1.QuadPart = PrivateCacheMap->FileOffset2.QuadPart;
//         PrivateCacheMap->BeyondLastByte1.QuadPart = PrivateCacheMap->BeyondLastByte2.QuadPart;
//         PrivateCacheMap->FileOffset2.QuadPart = FileOffset->QuadPart;
//         PrivateCacheMap->BeyondLastByte2.QuadPart = FileOffset->QuadPart + ReadLength;
//     }
// #endif
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// CcCopyWrite (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN BOOLEAN Wait,
//     IN PVOID Buffer)
// {
//     PROS_VACB Vacb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     NTSTATUS Status;
//     LONGLONG CurrentOffset;
//     LONGLONG WriteEnd;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%I64d Length=%lu Wait=%d Buffer=%p\n",
//         FileObject, FileOffset->QuadPart, Length, Wait, Buffer);
//
//     DPRINT("CcCopyWrite(FileObject 0x%p, FileOffset %I64x, "
//            "Length %lu, Wait %u, Buffer 0x%p)\n",
//            FileObject, FileOffset->QuadPart, Length, Wait, Buffer);
//
//     if (!SharedCacheMap)
//         return FALSE;
//
//     Status = RtlLongLongAdd(FileOffset->QuadPart, Length, &WriteEnd);
//     if (!NT_SUCCESS(Status))
//         ExRaiseStatus(Status);
//
//     ASSERT(WriteEnd <= SharedCacheMap->SectionSize.QuadPart);
//
//     CurrentOffset = FileOffset->QuadPart;
//     while(CurrentOffset < WriteEnd)
//     {
//         ULONG VacbOffset = CurrentOffset % VACB_MAPPING_GRANULARITY;
//         ULONG VacbLength = min(WriteEnd - CurrentOffset, VACB_MAPPING_GRANULARITY - VacbOffset);
//
//         Status = CcRosGetVacb(SharedCacheMap, CurrentOffset, &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             ExRaiseStatus(Status);
//             return FALSE;
//         }
//
//         _SEH2_TRY
//         {
//             if (!CcRosEnsureVacbResident(Vacb, Wait, FALSE, VacbOffset, VacbLength))
//             {
//                 return FALSE;
//             }
//
//             _SEH2_TRY
//             {
//                 RtlCopyMemory((PVOID)((ULONG_PTR)Vacb->BaseAddress + VacbOffset), Buffer, VacbLength);
//             }
//             _SEH2_EXCEPT(CcpCheckInvalidUserBuffer(_SEH2_GetExceptionInformation(), Buffer, VacbLength))
//             {
//                 ExRaiseStatus(STATUS_INVALID_USER_BUFFER);
//             }
//             _SEH2_END;
//
//             Buffer = (PVOID)((ULONG_PTR)Buffer + VacbLength);
//             CurrentOffset += VacbLength;
//
//             /* Tell Mm */
//             Status = MmMakeSegmentDirty(FileObject->SectionObjectPointer,
//                                         Vacb->FileOffset.QuadPart + VacbOffset,
//                                         VacbLength);
//             if (!NT_SUCCESS(Status))
//                 ExRaiseStatus(Status);
//         }
//         _SEH2_FINALLY
//         {
//             /* Do not mark the VACB as dirty if an exception was raised */
//             CcRosReleaseVacb(SharedCacheMap, Vacb, !_SEH2_AbnormalTermination(), FALSE);
//         }
//         _SEH2_END;
//     }
//
//     /* Flush if needed */
//     if (FileObject->Flags & FO_WRITE_THROUGH)
//         CcFlushCache(FileObject->SectionObjectPointer, FileOffset, Length, NULL);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcDeferWrite (
//     IN PFILE_OBJECT FileObject,
//     IN PCC_POST_DEFERRED_WRITE PostRoutine,
//     IN PVOID Context1,
//     IN PVOID Context2,
//     IN ULONG BytesToWrite,
//     IN BOOLEAN Retrying)
// {
//     KIRQL OldIrql;
//     PDEFERRED_WRITE Context;
//     PFSRTL_COMMON_FCB_HEADER Fcb;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p PostRoutine=%p Context1=%p Context2=%p BytesToWrite=%lu Retrying=%d\n",
//         FileObject, PostRoutine, Context1, Context2, BytesToWrite, Retrying);
//
//     /* Try to allocate a context for queueing the write operation */
//     Context = ExAllocatePoolWithTag(NonPagedPool, sizeof(DEFERRED_WRITE), 'CcDw');
//     /* If it failed, immediately execute the operation! */
//     if (Context == NULL)
//     {
//         PostRoutine(Context1, Context2);
//         return;
//     }
//
//     Fcb = FileObject->FsContext;
//
//     /* Otherwise, initialize the context */
//     RtlZeroMemory(Context, sizeof(DEFERRED_WRITE));
//     Context->NodeTypeCode = NODE_TYPE_DEFERRED_WRITE;
//     Context->NodeByteSize = sizeof(DEFERRED_WRITE);
//     Context->FileObject = FileObject;
//     Context->PostRoutine = PostRoutine;
//     Context->Context1 = Context1;
//     Context->Context2 = Context2;
//     Context->BytesToWrite = BytesToWrite;
//     Context->LimitModifiedPages = BooleanFlagOn(Fcb->Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES);
//
//     /* And queue it */
//     if (Retrying)
//     {
//         /* To the top, if that's a retry */
//         ExInterlockedInsertHeadList(&CcDeferredWrites,
//                                     &Context->DeferredWriteLinks,
//                                     &CcDeferredWriteSpinLock);
//     }
//     else
//     {
//         /* To the bottom, if that's a first time */
//         ExInterlockedInsertTailList(&CcDeferredWrites,
//                                     &Context->DeferredWriteLinks,
//                                     &CcDeferredWriteSpinLock);
//     }
//
//     /* Try to execute the posted writes */
//     CcPostDeferredWrites();
//
//     /* Schedule a lazy writer run to handle deferred writes */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     if (!LazyWriter.ScanActive)
//     {
//         CcScheduleLazyWriteScan(FALSE);
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcFastCopyRead (
//     IN PFILE_OBJECT FileObject,
//     IN ULONG FileOffset,
//     IN ULONG Length,
//     IN ULONG PageCount,
//     OUT PVOID Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus)
// {
//     LARGE_INTEGER LargeFileOffset;
//     BOOLEAN Success;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%lu Length=%lu PageCount=%lu Buffer=%p\n",
//         FileObject, FileOffset, Length, PageCount, Buffer);
//
//     DBG_UNREFERENCED_PARAMETER(PageCount);
//
//     LargeFileOffset.QuadPart = FileOffset;
//     Success = CcCopyRead(FileObject,
//                          &LargeFileOffset,
//                          Length,
//                          TRUE,
//                          Buffer,
//                          IoStatus);
//     ASSERT(Success == TRUE);
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcFastCopyWrite (
//     IN PFILE_OBJECT FileObject,
//     IN ULONG FileOffset,
//     IN ULONG Length,
//     IN PVOID Buffer)
// {
//     LARGE_INTEGER LargeFileOffset;
//     BOOLEAN Success;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%lu Length=%lu Buffer=%p\n",
//         FileObject, FileOffset, Length, Buffer);
//
//     LargeFileOffset.QuadPart = FileOffset;
//     Success = CcCopyWrite(FileObject,
//                           &LargeFileOffset,
//                           Length,
//                           TRUE,
//                           Buffer);
//     ASSERT(Success == TRUE);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// CcZeroData (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER StartOffset,
//     IN PLARGE_INTEGER EndOffset,
//     IN BOOLEAN Wait)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER WriteOffset;
//     LONGLONG Length;
//     PROS_VACB Vacb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p StartOffset=%I64u EndOffset=%I64u Wait=%d\n",
//         FileObject, StartOffset->QuadPart, EndOffset->QuadPart, Wait);
//
//     DPRINT("CcZeroData(FileObject 0x%p, StartOffset %I64x, EndOffset %I64x, "
//            "Wait %u)\n", FileObject, StartOffset->QuadPart, EndOffset->QuadPart,
//            Wait);
//
//     Length = EndOffset->QuadPart - StartOffset->QuadPart;
//     WriteOffset.QuadPart = StartOffset->QuadPart;
//
//     if (!SharedCacheMap)
//     {
//         /* Make this a non-cached write */
//         _IO_STATUS_BLOCK Iosb;
//         KEVENT Event;
//         PMDL Mdl;
//         ULONG i;
//         ULONG CurrentLength;
//         PPFN_NUMBER PfnArray;
//
//         /* Setup our Mdl */
//         Mdl = IoAllocateMdl(NULL, min(Length, MAX_ZERO_LENGTH), FALSE, FALSE, NULL);
//         if (!Mdl)
//             ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//
//         PfnArray = MmGetMdlPfnArray(Mdl);
//         for (i = 0; i < BYTES_TO_PAGES(Mdl->ByteCount); i++)
//             PfnArray[i] = CcZeroPage;
//         Mdl->MdlFlags |= MDL_PAGES_LOCKED;
//
//         /* Perform the write sequencially */
//         while (Length > 0)
//         {
//             CurrentLength = min(Length, MAX_ZERO_LENGTH);
//
//             Mdl->ByteCount = CurrentLength;
//
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//             Status = IoSynchronousPageWrite(FileObject, Mdl, &WriteOffset, &Event, &Iosb);
//             if (Status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 Status = Iosb.Status;
//             }
//             if (Mdl->MdlFlags & MDL_MAPPED_TO_SYSTEM_VA)
//             {
//                 MmUnmapLockedPages(Mdl->MappedSystemVa, Mdl);
//             }
//             if (!NT_SUCCESS(Status))
//             {
//                 IoFreeMdl(Mdl);
//                 ExRaiseStatus(Status);
//             }
//             WriteOffset.QuadPart += CurrentLength;
//             Length -= CurrentLength;
//         }
//
//         IoFreeMdl(Mdl);
//
//         return TRUE;
//     }
//
//     /* See if we should simply truncate the valid data length */
//     if ((StartOffset->QuadPart < SharedCacheMap->ValidDataLength.QuadPart) && (EndOffset->QuadPart >= SharedCacheMap->ValidDataLength.QuadPart))
//     {
//         DPRINT1("Truncating VDL.\n");
//         SharedCacheMap->ValidDataLength = *StartOffset;
//         return TRUE;
//     }
//
//     ASSERT(EndOffset->QuadPart <= SharedCacheMap->SectionSize.QuadPart);
//
//     while(WriteOffset.QuadPart < EndOffset->QuadPart)
//     {
//         ULONG VacbOffset = WriteOffset.QuadPart % VACB_MAPPING_GRANULARITY;
//         ULONG VacbLength = min(Length, VACB_MAPPING_GRANULARITY - VacbOffset);
//
//         Status = CcRosGetVacb(SharedCacheMap, WriteOffset.QuadPart, &Vacb);
//         if (!NT_SUCCESS(Status))
//         {
//             ExRaiseStatus(Status);
//             return FALSE;
//         }
//
//         _SEH2_TRY
//         {
//             if (!CcRosEnsureVacbResident(Vacb, Wait, FALSE, VacbOffset, VacbLength))
//             {
//                 return FALSE;
//             }
//
//             RtlZeroMemory((PVOID)((ULONG_PTR)Vacb->BaseAddress + VacbOffset), VacbLength);
//
//             WriteOffset.QuadPart += VacbLength;
//             Length -= VacbLength;
//
//             /* Tell Mm */
//             Status = MmMakeSegmentDirty(FileObject->SectionObjectPointer,
//                                         Vacb->FileOffset.QuadPart + VacbOffset,
//                                         VacbLength);
//             if (!NT_SUCCESS(Status))
//                 ExRaiseStatus(Status);
//         }
//         _SEH2_FINALLY
//         {
//             /* Do not mark the VACB as dirty if an exception was raised */
//             CcRosReleaseVacb(SharedCacheMap, Vacb, !_SEH2_AbnormalTermination(), FALSE);
//         }
//         _SEH2_END;
//     }
//
//     /* Flush if needed */
//     if (FileObject->Flags & FO_WRITE_THROUGH)
//         CcFlushCache(FileObject->SectionObjectPointer, StartOffset, EndOffset->QuadPart - StartOffset->QuadPart, NULL);
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
