use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _LARGE_INTEGER, _ULARGE_INTEGER, _IRP, _FILE_LOCK, _FILE_LOCK_INFO,
    _FILE_OBJECT, _EPROCESS, _IO_STACK_LOCATION, _LIST_ENTRY, _IO_CSQ,
    _RTL_GENERIC_TABLE, _FILE_SHARED_LOCK_ENTRY, _FILE_EXCLUSIVE_LOCK_ENTRY,
    _PAGED_LOOKASIDE_LIST,
    _IRP__bindgen_ty_3, _IRP__bindgen_ty_3__bindgen_ty_1,
    _IRP__bindgen_ty_3__bindgen_ty_1__bindgen_ty_2,
    _IRP__bindgen_ty_3__bindgen_ty_1__bindgen_ty_2__bindgen_ty_1,
    PFILE_LOCK, PFILE_LOCK_INFO, PFILE_OBJECT, PIRP, PIO_STATUS_BLOCK, PLARGE_INTEGER,
    PEPROCESS, PCOMPLETE_LOCK_IRP_ROUTINE, PUNLOCK_ROUTINE, PIO_STACK_LOCATION,
    PRTL_GENERIC_TABLE, PIO_CSQ, PKIRQL, LARGE_INTEGER, ULARGE_INTEGER, _IO_STATUS_BLOCK,
    CLONG, PNTSTATUS, RTL_GENERIC_COMPARE_RESULTS,
    IoCsqInitializeEx, IoCsqInsertIrpEx, IoCsqRemoveNextIrp, IoGetCurrentIrpStackLocation,
    IoGetRequestorProcess, IoMarkIrpPending, KeInitializeSpinLock,
    RtlInitializeGenericTable, RtlInsertElementGenericTable, RtlLookupElementGenericTable,
    RtlDeleteElementGenericTable, RtlEnumerateGenericTable, RtlGetElementGenericTable,
    RtlNumberGenericTableElements,
    IofCompleteRequest,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[inline(always)]
unsafe fn FsRtlCompleteRequest(Irp: ntkrnl_sys::ddk::PIRP, Status: NTSTATUS) {
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IofCompleteRequest(Irp, 1); // IO_DISK_INCREMENT
}

extern "system" {
    fn ExAllocateFromPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_PAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(Lookaside: *mut ntkrnl_sys::ddk::_PAGED_LOOKASIDE_LIST, Entry: PVOID);
}

extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
}

const TAG_TABLE: ULONG = 0x4241544C; // 'able'
const TAG_FLOCK: ULONG = 0x4B434C46; // 'lock'
const TAG_RANGE: ULONG = 0x41525346; // 'ange'

// ndk/rtltypes.h: GenericLessThan, GenericGreaterThan, GenericEqual = 0, 1, 2. A private copy
// here had Equal and GreaterThan SWAPPED, so every overlapping range compared "greater":
// conflicts went undetected, RtlDeleteElementGenericTable never found a lock, and
// FsRtlUninitializeFileLock spun forever inside fastfat's close holding the volume resource.
use ntkrnl_sys::ddk::{
    _RTL_GENERIC_COMPARE_RESULTS_GenericEqual, _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan,
    _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan,
};

const IRP_MJ_LOCK_CONTROL: u8 = 0x11;
const IRP_MN_LOCK: u8 = 0x01;
const IRP_MN_UNLOCK_SINGLE: u8 = 0x02;
const IRP_MN_UNLOCK_ALL: u8 = 0x03;
const IRP_MN_UNLOCK_ALL_BY_KEY: u8 = 0x04;

const SL_FAIL_IMMEDIATELY: u8 = 0x01;
const SL_EXCLUSIVE_LOCK: u8 = 0x02;

#[no_mangle]
pub static mut FsRtlFileLockLookasideList: _PAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

#[repr(C)]
#[derive(Copy, Clone)]
union COMBINED_LOCK_ELEMENT {
    _dummy: [u8; core::mem::size_of::<_FILE_EXCLUSIVE_LOCK_ENTRY>()],
    Exclusive: _FILE_EXCLUSIVE_LOCK_ENTRY,
}

type PCOMBINED_LOCK_ELEMENT = *mut COMBINED_LOCK_ELEMENT;

#[repr(C)]
struct LOCK_INFORMATION {
    RangeTable: _RTL_GENERIC_TABLE,
    Csq: _IO_CSQ,
    CsqLock: KSPIN_LOCK,
    CsqList: _LIST_ENTRY,
    BelongsTo: PFILE_LOCK,
    SharedLocks: _LIST_ENTRY,
    Generation: ULONG,
}

type PLOCK_INFORMATION = *mut LOCK_INFORMATION;

#[repr(C)]
struct LOCK_SHARED_RANGE {
    Entry: _LIST_ENTRY,
    Start: LARGE_INTEGER,
    End: LARGE_INTEGER,
    Key: ULONG,
    ProcessId: PVOID,
    // Not in the C: Windows owns a byte-range lock by (file object, process, key),
    // so unlocking through another handle must not release it.
    FileObject: PFILE_OBJECT,
}

type PLOCK_SHARED_RANGE = *mut LOCK_SHARED_RANGE;

unsafe extern "system" fn LockAllocate(Table: PRTL_GENERIC_TABLE, Bytes: CLONG) -> PVOID {
    let result = ExAllocatePoolWithTag(NonPagedPool, Bytes as usize, TAG_TABLE);
    result
}

unsafe extern "system" fn LockFree(Table: PRTL_GENERIC_TABLE, Buffer: PVOID) {
    ExFreePoolWithTag(Buffer, TAG_TABLE);
}

unsafe extern "system" fn LockCompare(
    Table: PRTL_GENERIC_TABLE,
    PtrA: PVOID,
    PtrB: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let A = PtrA as PCOMBINED_LOCK_ELEMENT;
    let B = PtrB as PCOMBINED_LOCK_ELEMENT;

    if (((*A).Exclusive.FileLock.StartingByte.QuadPart < (*B).Exclusive.FileLock.EndingByte.QuadPart)
        && ((*A).Exclusive.FileLock.StartingByte.QuadPart >= (*B).Exclusive.FileLock.StartingByte.QuadPart))
        || (((*B).Exclusive.FileLock.StartingByte.QuadPart < (*A).Exclusive.FileLock.EndingByte.QuadPart)
            && ((*B).Exclusive.FileLock.StartingByte.QuadPart >= (*A).Exclusive.FileLock.StartingByte.QuadPart))
    {
        return _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
    }

    if (*A).Exclusive.FileLock.StartingByte.QuadPart < (*B).Exclusive.FileLock.StartingByte.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan
    } else if (*A).Exclusive.FileLock.StartingByte.QuadPart > (*B).Exclusive.FileLock.StartingByte.QuadPart {
        _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan
    } else {
        _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
    }
}

unsafe extern "system" fn LockInsertIrpEx(Csq: PIO_CSQ, Irp: PIRP, InsertContext: PVOID) -> NTSTATUS {
    let lock_info_offset = core::mem::offset_of!(LOCK_INFORMATION, Csq);
    let LockInfo = CONTAINING_RECORD(Csq as PVOID, core::mem::size_of::<LOCK_INFORMATION>(), lock_info_offset) as PLOCK_INFORMATION;
    InsertTailList(
        core::ptr::addr_of_mut!((*LockInfo).CsqList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY,
    );
    STATUS_SUCCESS
}

unsafe extern "system" fn LockRemoveIrp(Csq: PIO_CSQ, Irp: PIRP) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY);
}

unsafe extern "system" fn LockPeekNextIrp(Csq: PIO_CSQ, Irp: PIRP, PeekContext: PVOID) -> PIRP {
    let mut LockElement: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let WhereUnlock = PeekContext as PCOMBINED_LOCK_ELEMENT;
    let lock_info_offset = core::mem::offset_of!(LOCK_INFORMATION, Csq);
    let LockInfo = CONTAINING_RECORD(Csq as PVOID, core::mem::size_of::<LOCK_INFORMATION>(), lock_info_offset) as PLOCK_INFORMATION;

    let mut Following: PLIST_ENTRY;
    if Irp.is_null() {
        Following = (*LockInfo).CsqList.Flink;
    } else {
        Following = (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink;
    }

    while Following != core::ptr::addr_of_mut!((*LockInfo).CsqList) as PLIST_ENTRY {
        let entry_offset = core::mem::offset_of!(_IRP, Tail) + core::mem::offset_of!(_IRP__bindgen_ty_3, Overlay)
            + core::mem::offset_of!(_IRP__bindgen_ty_3__bindgen_ty_1, __bindgen_anon_2)
            + core::mem::offset_of!(_IRP__bindgen_ty_3__bindgen_ty_1__bindgen_ty_2, ListEntry);
        let current_irp = CONTAINING_RECORD(Following as PVOID, core::mem::size_of::<_IRP>(), entry_offset) as PIRP;
        let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(current_irp);
        LockElement.Exclusive.FileLock.StartingByte = (*IoStack).Parameters.LockControl.ByteOffset;
        LockElement.Exclusive.FileLock.EndingByte.QuadPart = LockElement.Exclusive.FileLock.StartingByte.QuadPart
            + (*(*IoStack).Parameters.LockControl.Length).QuadPart;

        let Matching = if !WhereUnlock.is_null() {
            LockCompare(
                core::ptr::addr_of_mut!((*LockInfo).RangeTable),
                &mut LockElement as *mut _ as PVOID,
                WhereUnlock as PVOID,
            ) != _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
        } else {
            false
        };

        if !Matching {
            return current_irp;
        }

        Following = (*Following).Flink;
    }
    core::ptr::null_mut()
}

unsafe extern "system" fn LockAcquireQueueLock(Csq: PIO_CSQ, Irql: PKIRQL) {
    let lock_info_offset = core::mem::offset_of!(LOCK_INFORMATION, Csq);
    let LockInfo = CONTAINING_RECORD(Csq as PVOID, core::mem::size_of::<LOCK_INFORMATION>(), lock_info_offset) as PLOCK_INFORMATION;
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*LockInfo).CsqLock), Irql);
}

unsafe extern "system" fn LockReleaseQueueLock(Csq: PIO_CSQ, Irql: KIRQL) {
    let lock_info_offset = core::mem::offset_of!(LOCK_INFORMATION, Csq);
    let LockInfo = CONTAINING_RECORD(Csq as PVOID, core::mem::size_of::<LOCK_INFORMATION>(), lock_info_offset) as PLOCK_INFORMATION;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*LockInfo).CsqLock), Irql);
}

unsafe extern "system" fn LockCompleteCanceledIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let mut Status: NTSTATUS = 0;
    let lock_info_offset = core::mem::offset_of!(LOCK_INFORMATION, Csq);
    let LockInfo = CONTAINING_RECORD(Csq as PVOID, core::mem::size_of::<LOCK_INFORMATION>(), lock_info_offset) as PLOCK_INFORMATION;
    FsRtlCompleteLockIrpReal(
        (*(*LockInfo).BelongsTo).CompleteLockIrpRoutine,
        core::ptr::null_mut(),
        Irp,
        STATUS_CANCELLED,
        &mut Status as PNTSTATUS,
        core::ptr::null_mut(),
    );
}

unsafe fn FsRtlCompleteLockIrpReal(
    CompleteRoutine: PCOMPLETE_LOCK_IRP_ROUTINE,
    Context: PVOID,
    Irp: PIRP,
    Status: NTSTATUS,
    NewStatus: PNTSTATUS,
    FileObject: PFILE_OBJECT,
) {
    (*Irp).IoStatus.Information = 0;
    if let Some(routine) = CompleteRoutine {
        if !FileObject.is_null() {
            (*FileObject).LastLock = core::ptr::null_mut();
        }
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        *NewStatus = routine(Context, Irp);
    } else {
        FsRtlCompleteRequest(Irp, Status);
        *NewStatus = Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetNextFileLock(
    FileLock: PFILE_LOCK,
    Restart: BOOLEAN,
) -> PFILE_LOCK_INFO {
    if (*FileLock).LockInformation.is_null() {
        return core::ptr::null_mut();
    }
    let Entry = RtlEnumerateGenericTable((*FileLock).LockInformation as PRTL_GENERIC_TABLE, Restart) as PCOMBINED_LOCK_ELEMENT;
    if Entry.is_null() {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!((*Entry).Exclusive.FileLock) as PFILE_LOCK_INFO
    }
}

unsafe fn FsRtlpExpandLockElement(ToExpand: PCOMBINED_LOCK_ELEMENT, Conflict: PCOMBINED_LOCK_ELEMENT) {
    if (*ToExpand).Exclusive.FileLock.StartingByte.QuadPart > (*Conflict).Exclusive.FileLock.StartingByte.QuadPart {
        (*ToExpand).Exclusive.FileLock.StartingByte = (*Conflict).Exclusive.FileLock.StartingByte;
    }
    if (*ToExpand).Exclusive.FileLock.EndingByte.QuadPart < (*Conflict).Exclusive.FileLock.EndingByte.QuadPart {
        (*ToExpand).Exclusive.FileLock.EndingByte = (*Conflict).Exclusive.FileLock.EndingByte;
    }
}

unsafe fn FsRtlpRebuildSharedLockRange(
    FileLock: PFILE_LOCK,
    LockInfo: PLOCK_INFORMATION,
    Conflict: PCOMBINED_LOCK_ELEMENT,
) -> PCOMBINED_LOCK_ELEMENT {
    let mut InsertedNew = 0u8;
    let mut NewElement = *Conflict;
    let mut Entry: PCOMBINED_LOCK_ELEMENT;

    loop {
        Entry = RtlLookupElementGenericTable(
            (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
            &mut NewElement as *mut _ as PVOID,
        ) as PCOMBINED_LOCK_ELEMENT;
        if Entry.is_null() {
            break;
        }
        FsRtlpExpandLockElement(&mut NewElement, Entry);
        let RemovedOld = RtlDeleteElementGenericTable(
            core::ptr::addr_of_mut!((*LockInfo).RangeTable),
            Entry as PVOID,
        );
    }

    let result = RtlInsertElementGenericTable(
        core::ptr::addr_of_mut!((*LockInfo).RangeTable),
        &mut NewElement as *mut _ as PVOID,
        core::mem::size_of::<COMBINED_LOCK_ELEMENT>() as ULONG,
        &mut InsertedNew as *mut u8,
    ) as PCOMBINED_LOCK_ELEMENT;
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPrivateLock(
    FileLock: PFILE_LOCK,
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Process: PEPROCESS,
    Key: ULONG,
    FailImmediately: BOOLEAN,
    ExclusiveLock: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
    Irp: PIRP,
    Context: PVOID,
    AlreadySynchronized: BOOLEAN,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let Length = opaque_ptr(Length);
    let Process = opaque_ptr(Process);
    let IoStatus = opaque_ptr(IoStatus);
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let mut Status: NTSTATUS = 0;
    let mut ToInsert: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let mut Conflict: PCOMBINED_LOCK_ELEMENT;
    let mut LockInfo: PLOCK_INFORMATION;
    let mut NewSharedRange: PLOCK_SHARED_RANGE;
    let mut InsertedNew: u8 = 0;
    let mut UnsignedStart: ULARGE_INTEGER = core::mem::zeroed();
    let mut UnsignedEnd: ULARGE_INTEGER = core::mem::zeroed();

    UnsignedStart.QuadPart = (*FileOffset).QuadPart as u64;
    UnsignedEnd.QuadPart = ((*FileOffset).QuadPart + (*Length).QuadPart) as u64;

    if UnsignedEnd.QuadPart < UnsignedStart.QuadPart {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        if !Irp.is_null() {
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                (*IoStatus).__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                FileObject,
            );
        }
        return 0u8;
    }

    if (*FileLock).LockInformation.is_null() {
        LockInfo = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<LOCK_INFORMATION>(), TAG_FLOCK) as PLOCK_INFORMATION;
        if LockInfo.is_null() {
            (*IoStatus).__bindgen_anon_1.Status = STATUS_NO_MEMORY;
            return 0u8;
        }
        (*FileLock).LockInformation = LockInfo as PVOID;

        (*LockInfo).BelongsTo = FileLock;
        InitializeListHead(core::ptr::addr_of_mut!((*LockInfo).SharedLocks) as PLIST_ENTRY);

        RtlInitializeGenericTable(
            core::ptr::addr_of_mut!((*LockInfo).RangeTable),
            Some(abi_cast!(LockCompare)),
            Some(abi_cast!(LockAllocate)),
            Some(abi_cast!(LockFree)),
            core::ptr::null_mut(),
        );

        KeInitializeSpinLock(core::ptr::addr_of_mut!((*LockInfo).CsqLock) as *mut _);
        InitializeListHead(core::ptr::addr_of_mut!((*LockInfo).CsqList) as PLIST_ENTRY);

        IoCsqInitializeEx(
            core::ptr::addr_of_mut!((*LockInfo).Csq),
            Some(abi_cast!(LockInsertIrpEx)),
            Some(abi_cast!(LockRemoveIrp)),
            Some(abi_cast!(LockPeekNextIrp)),
            Some(abi_cast!(LockAcquireQueueLock)),
            Some(abi_cast!(LockReleaseQueueLock)),
            Some(abi_cast!(LockCompleteCanceledIrp)),
        );
    }

    LockInfo = (*FileLock).LockInformation as PLOCK_INFORMATION;
    ToInsert.Exclusive.FileLock.FileObject = FileObject;
    ToInsert.Exclusive.FileLock.StartingByte = *FileOffset;
    ToInsert.Exclusive.FileLock.EndingByte.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
    ToInsert.Exclusive.FileLock.ProcessId = Process as PVOID;
    ToInsert.Exclusive.FileLock.Key = Key;
    ToInsert.Exclusive.FileLock.ExclusiveLock = ExclusiveLock;

    Conflict = RtlInsertElementGenericTable(
        (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
        &mut ToInsert as *mut _ as PVOID,
        core::mem::size_of::<COMBINED_LOCK_ELEMENT>() as ULONG,
        &mut InsertedNew as *mut u8,
    ) as PCOMBINED_LOCK_ELEMENT;

    if !Conflict.is_null() && InsertedNew == 0 {
        if (*Conflict).Exclusive.FileLock.ExclusiveLock != 0 || ExclusiveLock != 0 {
            if FailImmediately != 0 {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_FILE_LOCK_CONFLICT;
                if !Irp.is_null() {
                    FsRtlCompleteLockIrpReal(
                        (*FileLock).CompleteLockIrpRoutine,
                        Context,
                        Irp,
                        (*IoStatus).__bindgen_anon_1.Status,
                        &mut Status as PNTSTATUS,
                        FileObject,
                    );
                }
                return 0u8;
            } else {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_PENDING;
                if !Irp.is_null() {
                    (*Irp).IoStatus.Information = (*LockInfo).Generation as _;
                    IoMarkIrpPending(Irp);
                    IoCsqInsertIrpEx(
                        core::ptr::addr_of_mut!((*LockInfo).Csq),
                        Irp,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                }
            }
            return 0u8;
        } else {
            let num_elements = RtlNumberGenericTableElements(core::ptr::addr_of_mut!((*LockInfo).RangeTable)) as ULONG;
            for i in 0..num_elements {
                Conflict = RtlGetElementGenericTable(core::ptr::addr_of_mut!((*LockInfo).RangeTable), i) as PCOMBINED_LOCK_ELEMENT;

                if !Conflict.is_null()
                    && LockCompare(
                        core::ptr::addr_of_mut!((*LockInfo).RangeTable),
                        Conflict as PVOID,
                        &mut ToInsert as *mut _ as PVOID,
                    ) == _RTL_GENERIC_COMPARE_RESULTS_GenericEqual
                {
                    if (*Conflict).Exclusive.FileLock.ExclusiveLock != 0 {
                        if FailImmediately != 0 {
                            (*IoStatus).__bindgen_anon_1.Status = STATUS_FILE_LOCK_CONFLICT;
                            if !Irp.is_null() {
                                FsRtlCompleteLockIrpReal(
                                    (*FileLock).CompleteLockIrpRoutine,
                                    Context,
                                    Irp,
                                    (*IoStatus).__bindgen_anon_1.Status,
                                    &mut Status as PNTSTATUS,
                                    FileObject,
                                );
                            }
                        } else {
                            (*IoStatus).__bindgen_anon_1.Status = STATUS_PENDING;
                            if !Irp.is_null() {
                                IoMarkIrpPending(Irp);
                                IoCsqInsertIrpEx(
                                    core::ptr::addr_of_mut!((*LockInfo).Csq),
                                    Irp,
                                    core::ptr::null_mut(),
                                    core::ptr::null_mut(),
                                );
                            }
                        }
                        return 0u8;
                    }
                }
            }

            Conflict = FsRtlpRebuildSharedLockRange(FileLock, LockInfo, &mut ToInsert);
            if Conflict.is_null() {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_NO_MEMORY;
                if !Irp.is_null() {
                    FsRtlCompleteLockIrpReal(
                        (*FileLock).CompleteLockIrpRoutine,
                        Context,
                        Irp,
                        (*IoStatus).__bindgen_anon_1.Status,
                        &mut Status as PNTSTATUS,
                        FileObject,
                    );
                }
            }

            NewSharedRange = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<LOCK_SHARED_RANGE>(), TAG_RANGE) as PLOCK_SHARED_RANGE;
            if NewSharedRange.is_null() {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_NO_MEMORY;
                if !Irp.is_null() {
                    FsRtlCompleteLockIrpReal(
                        (*FileLock).CompleteLockIrpRoutine,
                        Context,
                        Irp,
                        (*IoStatus).__bindgen_anon_1.Status,
                        &mut Status as PNTSTATUS,
                        FileObject,
                    );
                }
                return 0u8;
            }
            (*NewSharedRange).Start = *FileOffset;
            (*NewSharedRange).End.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
            (*NewSharedRange).Key = Key;
            (*NewSharedRange).ProcessId = ToInsert.Exclusive.FileLock.ProcessId;
            (*NewSharedRange).FileObject = FileObject;
            InsertTailList(
                core::ptr::addr_of_mut!((*LockInfo).SharedLocks) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*NewSharedRange).Entry) as PLIST_ENTRY,
            );

            (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
            if !Irp.is_null() {
                FsRtlCompleteLockIrpReal(
                    (*FileLock).CompleteLockIrpRoutine,
                    Context,
                    Irp,
                    (*IoStatus).__bindgen_anon_1.Status,
                    &mut Status as PNTSTATUS,
                    FileObject,
                );
            }
            return 1u8;
        }
    } else if Conflict.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        if !Irp.is_null() {
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                (*IoStatus).__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                FileObject,
            );
        }
        return 0u8;
    } else {
        if ExclusiveLock == 0 {
            NewSharedRange = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<LOCK_SHARED_RANGE>(), TAG_RANGE) as PLOCK_SHARED_RANGE;
            if NewSharedRange.is_null() {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_NO_MEMORY;
                if !Irp.is_null() {
                    FsRtlCompleteLockIrpReal(
                        (*FileLock).CompleteLockIrpRoutine,
                        Context,
                        Irp,
                        (*IoStatus).__bindgen_anon_1.Status,
                        &mut Status as PNTSTATUS,
                        FileObject,
                    );
                }
                return 0u8;
            }
            (*NewSharedRange).Start = *FileOffset;
            (*NewSharedRange).End.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
            (*NewSharedRange).Key = Key;
            (*NewSharedRange).ProcessId = Process as PVOID;
            (*NewSharedRange).FileObject = FileObject;
            InsertTailList(
                core::ptr::addr_of_mut!((*LockInfo).SharedLocks) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*NewSharedRange).Entry) as PLIST_ENTRY,
            );
        }

        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;

        if !Irp.is_null() {
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                (*IoStatus).__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                FileObject,
            );
            (*IoStatus).__bindgen_anon_1.Status = Status;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCheckLockForReadAccess(FileLock: PFILE_LOCK, Irp: PIRP) -> BOOLEAN {
    let Irp = opaque_ptr(Irp);
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut ToFind: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let Found: PCOMBINED_LOCK_ELEMENT;

    if (*FileLock).LockInformation.is_null() {
        return 1u8;
    }
    ToFind.Exclusive.FileLock.StartingByte = (*IoStack).Parameters.Read.ByteOffset;
    ToFind.Exclusive.FileLock.EndingByte.QuadPart =
        ToFind.Exclusive.FileLock.StartingByte.QuadPart + (*IoStack).Parameters.Read.Length as i64;
    Found = RtlLookupElementGenericTable(
        (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
        &mut ToFind as *mut _ as PVOID,
    ) as PCOMBINED_LOCK_ELEMENT;
    if Found.is_null() {
        return 1u8;
    }
    let result = if (*Found).Exclusive.FileLock.ExclusiveLock == 0
        || (*IoStack).Parameters.Read.Key == (*Found).Exclusive.FileLock.Key
    {
        1u8
    } else {
        0u8
    };
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCheckLockForWriteAccess(FileLock: PFILE_LOCK, Irp: PIRP) -> BOOLEAN {
    let Irp = opaque_ptr(Irp);
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut ToFind: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let Found: PCOMBINED_LOCK_ELEMENT;
    let Process = (*(*Irp).Tail.Overlay.Thread).Tcb.Process as PEPROCESS;

    if (*FileLock).LockInformation.is_null() {
        return 1u8;
    }
    ToFind.Exclusive.FileLock.StartingByte = (*IoStack).Parameters.Write.ByteOffset;
    ToFind.Exclusive.FileLock.EndingByte.QuadPart =
        ToFind.Exclusive.FileLock.StartingByte.QuadPart + (*IoStack).Parameters.Write.Length as i64;
    Found = RtlLookupElementGenericTable(
        (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
        &mut ToFind as *mut _ as PVOID,
    ) as PCOMBINED_LOCK_ELEMENT;
    if Found.is_null() {
        return 1u8;
    }
    if Process as PVOID == (*Found).Exclusive.FileLock.ProcessId {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFastCheckLockForRead(
    FileLock: PFILE_LOCK,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Key: ULONG,
    FileObject: PFILE_OBJECT,
    Process: PVOID,
) -> BOOLEAN {
    let FileOffset = opaque_ptr(FileOffset);
    let Length = opaque_ptr(Length);
    let FileObject = opaque_ptr(FileObject);
    let Process = opaque_ptr(Process);
    let EProcess = Process as PEPROCESS;
    let mut ToFind: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let Found: PCOMBINED_LOCK_ELEMENT;

    ToFind.Exclusive.FileLock.StartingByte = *FileOffset;
    ToFind.Exclusive.FileLock.EndingByte.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
    if (*FileLock).LockInformation.is_null() {
        return 1u8;
    }
    Found = RtlLookupElementGenericTable(
        (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
        &mut ToFind as *mut _ as PVOID,
    ) as PCOMBINED_LOCK_ELEMENT;
    if Found.is_null() || (*Found).Exclusive.FileLock.ExclusiveLock == 0 {
        return 1u8;
    }
    if (*Found).Exclusive.FileLock.Key == Key && (*Found).Exclusive.FileLock.ProcessId == EProcess as PVOID {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFastCheckLockForWrite(
    FileLock: PFILE_LOCK,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Key: ULONG,
    FileObject: PFILE_OBJECT,
    Process: PVOID,
) -> BOOLEAN {
    let FileOffset = opaque_ptr(FileOffset);
    let Length = opaque_ptr(Length);
    let FileObject = opaque_ptr(FileObject);
    let Process = opaque_ptr(Process);
    let EProcess = Process as PEPROCESS;
    let mut ToFind: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let Found: PCOMBINED_LOCK_ELEMENT;

    ToFind.Exclusive.FileLock.StartingByte = *FileOffset;
    ToFind.Exclusive.FileLock.EndingByte.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
    if (*FileLock).LockInformation.is_null() {
        return 1u8;
    }
    Found = RtlLookupElementGenericTable(
        (*FileLock).LockInformation as PRTL_GENERIC_TABLE,
        &mut ToFind as *mut _ as PVOID,
    ) as PCOMBINED_LOCK_ELEMENT;
    if Found.is_null() {
        return 1u8;
    }
    if (*Found).Exclusive.FileLock.Key == Key && (*Found).Exclusive.FileLock.ProcessId == EProcess as PVOID {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFastUnlockSingle(
    FileLock: PFILE_LOCK,
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    Process: PEPROCESS,
    Key: ULONG,
    Context: PVOID,
    AlreadySynchronized: BOOLEAN,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let Length = opaque_ptr(Length);
    let Process = opaque_ptr(Process);
    let Context = opaque_ptr(Context);
    let mut FoundShared = 0u8;
    let mut SharedEntry: PLIST_ENTRY;
    let mut SharedRange: PLOCK_SHARED_RANGE = core::ptr::null_mut();
    let mut Find: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
    let mut Entry: PCOMBINED_LOCK_ELEMENT;
    let mut NextMatchingLockIrp: PIRP;
    let InternalInfo = (*FileLock).LockInformation as PLOCK_INFORMATION;

    Find.Exclusive.FileLock.StartingByte = *FileOffset;
    Find.Exclusive.FileLock.EndingByte.QuadPart = (*FileOffset).QuadPart + (*Length).QuadPart;
    if InternalInfo.is_null() {
        return STATUS_RANGE_NOT_LOCKED;
    }
    Entry = RtlLookupElementGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), &mut Find as *mut _ as PVOID) as PCOMBINED_LOCK_ELEMENT;
    if Entry.is_null() {
        return STATUS_RANGE_NOT_LOCKED;
    }

    if (*Entry).Exclusive.FileLock.ExclusiveLock != 0 {
        // The C omits the file object; Windows requires it (the olewinetest storage
        // probe unlocks every byte through its own handle and released ole32's locks).
        if (*Entry).Exclusive.FileLock.Key != Key
            || (*Entry).Exclusive.FileLock.ProcessId != Process as PVOID
            || (*Entry).Exclusive.FileLock.FileObject != FileObject
            || (*Entry).Exclusive.FileLock.StartingByte.QuadPart != (*FileOffset).QuadPart
            || (*Entry).Exclusive.FileLock.EndingByte.QuadPart != (*FileOffset).QuadPart + (*Length).QuadPart
        {
            return STATUS_RANGE_NOT_LOCKED;
        }
        core::ptr::copy_nonoverlapping(Entry as *const u8, &mut Find as *mut _ as *mut u8, core::mem::size_of::<COMBINED_LOCK_ELEMENT>());
        RtlDeleteElementGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), Entry as PVOID);
    } else {
        SharedEntry = (*InternalInfo).SharedLocks.Flink;
        while SharedEntry != core::ptr::addr_of_mut!((*InternalInfo).SharedLocks) as PLIST_ENTRY {
            let range_offset = core::mem::offset_of!(LOCK_SHARED_RANGE, Entry);
            SharedRange = CONTAINING_RECORD(SharedEntry as PVOID, core::mem::size_of::<LOCK_SHARED_RANGE>(), range_offset) as PLOCK_SHARED_RANGE;
            if (*SharedRange).Start.QuadPart == (*FileOffset).QuadPart
                && (*SharedRange).End.QuadPart == (*FileOffset).QuadPart + (*Length).QuadPart
                && (*SharedRange).Key == Key
                && (*SharedRange).ProcessId == Process as PVOID
                && (*SharedRange).FileObject == FileObject
            {
                FoundShared = 1u8;
                break;
            }
            SharedEntry = (*SharedEntry).Flink;
        }
        if FoundShared != 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*SharedRange).Entry) as PLIST_ENTRY);
            ExFreePoolWithTag(SharedRange as PVOID, TAG_RANGE);

            Find = *Entry;
            RtlDeleteElementGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), &mut Find as *mut _ as PVOID);

            SharedEntry = (*InternalInfo).SharedLocks.Flink;
            while SharedEntry != core::ptr::addr_of_mut!((*InternalInfo).SharedLocks) as PLIST_ENTRY {
                let mut LockElement: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
                let range_offset = core::mem::offset_of!(LOCK_SHARED_RANGE, Entry);
                SharedRange = CONTAINING_RECORD(SharedEntry as PVOID, core::mem::size_of::<LOCK_SHARED_RANGE>(), range_offset) as PLOCK_SHARED_RANGE;
                LockElement.Exclusive.FileLock.FileObject = FileObject;
                LockElement.Exclusive.FileLock.StartingByte = (*SharedRange).Start;
                LockElement.Exclusive.FileLock.EndingByte = (*SharedRange).End;
                LockElement.Exclusive.FileLock.ProcessId = (*SharedRange).ProcessId;
                LockElement.Exclusive.FileLock.Key = (*SharedRange).Key;
                LockElement.Exclusive.FileLock.ExclusiveLock = 0u8;

                if LockCompare(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), &mut Find as *mut _ as PVOID, &mut LockElement as *mut _ as PVOID) != _RTL_GENERIC_COMPARE_RESULTS_GenericEqual {
                    SharedEntry = (*SharedEntry).Flink;
                    continue;
                }
                FsRtlpRebuildSharedLockRange(FileLock, InternalInfo, &mut LockElement);
                SharedEntry = (*SharedEntry).Flink;
            }
        } else {
            return STATUS_RANGE_NOT_LOCKED;
        }
    }

    (*InternalInfo).Generation = (*InternalInfo).Generation.wrapping_add(1);
    loop {
        NextMatchingLockIrp = IoCsqRemoveNextIrp(core::ptr::addr_of_mut!((*InternalInfo).Csq), &mut Find as *mut _ as PVOID);
        if NextMatchingLockIrp.is_null() {
            break;
        }
        if (*NextMatchingLockIrp).IoStatus.Information == (*InternalInfo).Generation as _ {
            IoCsqInsertIrpEx(
                core::ptr::addr_of_mut!((*InternalInfo).Csq),
                NextMatchingLockIrp,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            break;
        }
        let Status = FsRtlProcessFileLock((*InternalInfo).BelongsTo, NextMatchingLockIrp, core::ptr::null_mut());
        if Status < 0 {
            return Status;
        }
    }

    STATUS_SUCCESS
}

/// Release every exclusive lock owned by (`FileObject`, `Process`[, `Key`]).
///
/// The C walked the whole table: FsRtlFastUnlockAll's translation had also lost its
/// `ProcessId != Process` filter, so closing ANY handle to a file dropped every
/// lock on it, and neither version checked the file object Windows owns locks by.
/// A delete invalidates the enumeration cursor, so the walk restarts after each one.
unsafe fn FsRtlpUnlockOwnedExclusive(
    FileLock: PFILE_LOCK,
    InternalInfo: PLOCK_INFORMATION,
    FileObject: PFILE_OBJECT,
    Process: PEPROCESS,
    Key: Option<ULONG>,
    Context: PVOID,
) {
    let Table = core::ptr::addr_of_mut!((*InternalInfo).RangeTable);
    let mut Entry = RtlEnumerateGenericTable(Table, 1u8) as PCOMBINED_LOCK_ELEMENT;
    while !Entry.is_null() {
        let Lock = core::ptr::addr_of_mut!((*Entry).Exclusive.FileLock);
        let Owned = (*Lock).ExclusiveLock != 0
            && (*Lock).ProcessId == Process as PVOID
            && (*Lock).FileObject == FileObject
            && Key.is_none_or(|k| (*Lock).Key == k);
        if Owned {
            let mut Start = (*Lock).StartingByte;
            let mut Length: LARGE_INTEGER = core::mem::zeroed();
            Length.QuadPart = (*Lock).EndingByte.QuadPart - Start.QuadPart;
            let Status = FsRtlFastUnlockSingle(
                FileLock,
                FileObject,
                &mut Start,
                &mut Length,
                Process,
                (*Lock).Key,
                Context,
                1u8,
            );
            if Status >= 0 {
                Entry = RtlEnumerateGenericTable(Table, 1u8) as PCOMBINED_LOCK_ELEMENT;
                continue;
            }
        }
        Entry = RtlEnumerateGenericTable(Table, 0u8) as PCOMBINED_LOCK_ELEMENT;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFastUnlockAll(
    FileLock: PFILE_LOCK,
    FileObject: PFILE_OBJECT,
    Process: PEPROCESS,
    Context: PVOID,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let Process = opaque_ptr(Process);
    let Context = opaque_ptr(Context);
    let mut ListEntry: PLIST_ENTRY;
    let InternalInfo = (*FileLock).LockInformation as PLOCK_INFORMATION;

    if (*FileLock).LockInformation.is_null() {
        return STATUS_RANGE_NOT_LOCKED;
    }
    ListEntry = (*InternalInfo).SharedLocks.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*InternalInfo).SharedLocks) as PLIST_ENTRY {
        let mut Length: LARGE_INTEGER = core::mem::zeroed();
        let range_offset = core::mem::offset_of!(LOCK_SHARED_RANGE, Entry);
        let Range = CONTAINING_RECORD(ListEntry as PVOID, core::mem::size_of::<LOCK_SHARED_RANGE>(), range_offset) as PLOCK_SHARED_RANGE;
        Length.QuadPart = (*Range).End.QuadPart - (*Range).Start.QuadPart;
        ListEntry = (*ListEntry).Flink;
        if (*Range).ProcessId != Process as PVOID || (*Range).FileObject != FileObject {
            continue;
        }
        FsRtlFastUnlockSingle(
            FileLock,
            FileObject,
            core::ptr::addr_of_mut!((*Range).Start),
            &mut Length as PLARGE_INTEGER,
            (*Range).ProcessId as PEPROCESS,
            (*Range).Key,
            Context,
            1u8,
        );
    }
    FsRtlpUnlockOwnedExclusive(FileLock, InternalInfo, FileObject, Process, None, Context);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFastUnlockAllByKey(
    FileLock: PFILE_LOCK,
    FileObject: PFILE_OBJECT,
    Process: PEPROCESS,
    Key: ULONG,
    Context: PVOID,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let Process = opaque_ptr(Process);
    let Context = opaque_ptr(Context);
    let mut ListEntry: PLIST_ENTRY;
    let InternalInfo = (*FileLock).LockInformation as PLOCK_INFORMATION;

    if (*FileLock).LockInformation.is_null() {
        return STATUS_RANGE_NOT_LOCKED;
    }
    ListEntry = (*InternalInfo).SharedLocks.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*InternalInfo).SharedLocks) as PLIST_ENTRY {
        let range_offset = core::mem::offset_of!(LOCK_SHARED_RANGE, Entry);
        let Range = CONTAINING_RECORD(ListEntry as PVOID, core::mem::size_of::<LOCK_SHARED_RANGE>(), range_offset) as PLOCK_SHARED_RANGE;
        let mut Length: LARGE_INTEGER = core::mem::zeroed();
        Length.QuadPart = (*Range).End.QuadPart - (*Range).Start.QuadPart;
        ListEntry = (*ListEntry).Flink;
        if (*Range).ProcessId != Process as PVOID
            || (*Range).Key != Key
            || (*Range).FileObject != FileObject
        {
            continue;
        }
        FsRtlFastUnlockSingle(
            FileLock,
            FileObject,
            core::ptr::addr_of_mut!((*Range).Start),
            &mut Length as PLARGE_INTEGER,
            (*Range).ProcessId as PEPROCESS,
            (*Range).Key,
            Context,
            1u8,
        );
    }
    FsRtlpUnlockOwnedExclusive(FileLock, InternalInfo, FileObject, Process, Some(Key), Context);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlProcessFileLock(
    FileLock: PFILE_LOCK,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let IoStackLocation: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    IoStatusBlock.Information = 0;

    match (*IoStackLocation).MinorFunction {
        IRP_MN_LOCK => {
            let Result = FsRtlPrivateLock(
                FileLock,
                (*IoStackLocation).FileObject,
                core::ptr::addr_of_mut!((*IoStackLocation).Parameters.LockControl.ByteOffset),
                (*IoStackLocation).Parameters.LockControl.Length,
                IoGetRequestorProcess(Irp),
                (*IoStackLocation).Parameters.LockControl.Key,
                if (*IoStackLocation).Flags & SL_FAIL_IMMEDIATELY != 0 { 1u8 } else { 0u8 },
                if (*IoStackLocation).Flags & SL_EXCLUSIVE_LOCK != 0 { 1u8 } else { 0u8 },
                &mut IoStatusBlock as PIO_STATUS_BLOCK,
                Irp,
                Context,
                0u8,
            );
            IoStatusBlock.__bindgen_anon_1.Status
        }
        IRP_MN_UNLOCK_SINGLE => {
            IoStatusBlock.__bindgen_anon_1.Status = FsRtlFastUnlockSingle(
                FileLock,
                (*IoStackLocation).FileObject,
                core::ptr::addr_of_mut!((*IoStackLocation).Parameters.LockControl.ByteOffset),
                (*IoStackLocation).Parameters.LockControl.Length,
                IoGetRequestorProcess(Irp),
                (*IoStackLocation).Parameters.LockControl.Key,
                Context,
                0u8,
            );
            let mut Status: NTSTATUS = 0;
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                IoStatusBlock.__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                core::ptr::null_mut(),
            );
            IoStatusBlock.__bindgen_anon_1.Status
        }
        IRP_MN_UNLOCK_ALL => {
            IoStatusBlock.__bindgen_anon_1.Status = FsRtlFastUnlockAll(FileLock, (*IoStackLocation).FileObject, IoGetRequestorProcess(Irp), Context);
            let mut Status: NTSTATUS = 0;
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                IoStatusBlock.__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                core::ptr::null_mut(),
            );
            IoStatusBlock.__bindgen_anon_1.Status
        }
        IRP_MN_UNLOCK_ALL_BY_KEY => {
            IoStatusBlock.__bindgen_anon_1.Status = FsRtlFastUnlockAllByKey(
                FileLock,
                (*IoStackLocation).FileObject,
                IoGetRequestorProcess(Irp),
                (*IoStackLocation).Parameters.LockControl.Key,
                Context,
            );
            let mut Status: NTSTATUS = 0;
            FsRtlCompleteLockIrpReal(
                (*FileLock).CompleteLockIrpRoutine,
                Context,
                Irp,
                IoStatusBlock.__bindgen_anon_1.Status,
                &mut Status as PNTSTATUS,
                core::ptr::null_mut(),
            );
            IoStatusBlock.__bindgen_anon_1.Status
        }
        _ => {
            FsRtlCompleteRequest(Irp, STATUS_INVALID_DEVICE_REQUEST);
            STATUS_INVALID_DEVICE_REQUEST
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeFileLock(
    FileLock: PFILE_LOCK,
    CompleteLockIrpRoutine: PCOMPLETE_LOCK_IRP_ROUTINE,
    UnlockRoutine: PUNLOCK_ROUTINE,
) {
    core::ptr::write_bytes(FileLock as *mut u8, 0, core::mem::size_of::<_FILE_LOCK>());
    (*FileLock).FastIoIsQuestionable = 0u8;
    (*FileLock).CompleteLockIrpRoutine = CompleteLockIrpRoutine;
    (*FileLock).UnlockRoutine = UnlockRoutine;
    (*FileLock).LockInformation = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlUninitializeFileLock(FileLock: PFILE_LOCK) {
    if !(*FileLock).LockInformation.is_null() {
        let mut Irp: PIRP;
        let InternalInfo = (*FileLock).LockInformation as PLOCK_INFORMATION;
        let mut Entry: PCOMBINED_LOCK_ELEMENT;
        let mut SharedEntry: PLIST_ENTRY;
        let mut SharedRange: PLOCK_SHARED_RANGE;

        SharedEntry = (*InternalInfo).SharedLocks.Flink;
        while SharedEntry != core::ptr::addr_of_mut!((*InternalInfo).SharedLocks) as PLIST_ENTRY {
            let range_offset = core::mem::offset_of!(LOCK_SHARED_RANGE, Entry);
            SharedRange = CONTAINING_RECORD(SharedEntry as PVOID, core::mem::size_of::<LOCK_SHARED_RANGE>(), range_offset) as PLOCK_SHARED_RANGE;
            SharedEntry = (*SharedEntry).Flink;
            RemoveEntryList(core::ptr::addr_of_mut!((*SharedRange).Entry) as PLIST_ENTRY);
            ExFreePoolWithTag(SharedRange as PVOID, TAG_RANGE);
        }
        loop {
            Entry = RtlGetElementGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), 0) as PCOMBINED_LOCK_ELEMENT;
            if Entry.is_null() {
                break;
            }
            RtlDeleteElementGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), Entry as PVOID);
        }
        loop {
            Irp = IoCsqRemoveNextIrp(core::ptr::addr_of_mut!((*InternalInfo).Csq), core::ptr::null_mut());
            if Irp.is_null() {
                break;
            }
            let Status = FsRtlProcessFileLock(FileLock, Irp, core::ptr::null_mut());
        }
        ExFreePoolWithTag(InternalInfo as PVOID, TAG_FLOCK);
        (*FileLock).LockInformation = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAllocateFileLock(
    CompleteLockIrpRoutine: PCOMPLETE_LOCK_IRP_ROUTINE,
    UnlockRoutine: PUNLOCK_ROUTINE,
) -> PFILE_LOCK {
    let FileLock = ExAllocateFromPagedLookasideList(core::ptr::addr_of_mut!(FsRtlFileLockLookasideList)) as PFILE_LOCK;
    if !FileLock.is_null() {
        FsRtlInitializeFileLock(FileLock, CompleteLockIrpRoutine, UnlockRoutine);
    }
    FileLock
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlFreeFileLock(FileLock: PFILE_LOCK) {
    FsRtlUninitializeFileLock(FileLock);
    ExFreeToPagedLookasideList(core::ptr::addr_of_mut!(FsRtlFileLockLookasideList), FileLock as PVOID);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/filelock.c
//  * PURPOSE:         File Locking implementation for File System Drivers
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
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
// _PAGED_LOOKASIDE_LIST FsRtlFileLockLookasideList;
//
// /* Note: this aligns the two types of lock entry structs so we can access the
//    FILE_LOCK_INFO part in common.  Add elements after Shared if new stuff is needed.
// */
// typedef union _COMBINED_LOCK_ELEMENT
// {
//     struct
//     {
//         LIST_ENTRY dummy;
//         FILE_SHARED_LOCK_ENTRY Shared;
//     };
//     FILE_EXCLUSIVE_LOCK_ENTRY Exclusive;
// }
//     COMBINED_LOCK_ELEMENT, *PCOMBINED_LOCK_ELEMENT;
//
// typedef struct _LOCK_INFORMATION
// {
//     RTL_GENERIC_TABLE RangeTable;
//     IO_CSQ Csq;
//     KSPIN_LOCK CsqLock;
//     LIST_ENTRY CsqList;
//     PFILE_LOCK BelongsTo;
//     LIST_ENTRY SharedLocks;
//     ULONG Generation;
// }
//     LOCK_INFORMATION, *PLOCK_INFORMATION;
//
// typedef struct _LOCK_SHARED_RANGE
// {
//     LIST_ENTRY Entry;
//     LARGE_INTEGER Start, End;
//     ULONG Key;
//     PVOID ProcessId;
// }
//     LOCK_SHARED_RANGE, *PLOCK_SHARED_RANGE;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// FsRtlCompleteLockIrpReal(IN PCOMPLETE_LOCK_IRP_ROUTINE CompleteRoutine,
//                          IN PVOID Context,
//                          IN PIRP Irp,
//                          IN NTSTATUS Status,
//                          OUT PNTSTATUS NewStatus,
//                          IN PFILE_OBJECT FileObject OPTIONAL);
//
// /* Generic table methods */
//
// static PVOID NTAPI LockAllocate(PRTL_GENERIC_TABLE Table, CLONG Bytes)
// {
//     PVOID Result;
//     Result = ExAllocatePoolWithTag(NonPagedPool, Bytes, TAG_TABLE);
//     DPRINT("LockAllocate(%lu) => %p\n", Bytes, Result);
//     return Result;
// }
//
// static VOID NTAPI LockFree(PRTL_GENERIC_TABLE Table, PVOID Buffer)
// {
//     DPRINT("LockFree(%p)\n", Buffer);
//     ExFreePoolWithTag(Buffer, TAG_TABLE);
// }
//
// static RTL_GENERIC_COMPARE_RESULTS NTAPI LockCompare
// (PRTL_GENERIC_TABLE Table, PVOID PtrA, PVOID PtrB)
// {
//     PCOMBINED_LOCK_ELEMENT A = PtrA, B = PtrB;
//     RTL_GENERIC_COMPARE_RESULTS Result;
// #if 0
//     DPRINT("Starting to compare element %x to element %x\n", PtrA, PtrB);
// #endif
//     /* Match if we overlap */
//     if (((A->Exclusive.FileLock.StartingByte.QuadPart <
//           B->Exclusive.FileLock.EndingByte.QuadPart) &&
//          (A->Exclusive.FileLock.StartingByte.QuadPart >=
//           B->Exclusive.FileLock.StartingByte.QuadPart)) ||
//         ((B->Exclusive.FileLock.StartingByte.QuadPart <
//           A->Exclusive.FileLock.EndingByte.QuadPart) &&
//          (B->Exclusive.FileLock.StartingByte.QuadPart >=
//           A->Exclusive.FileLock.StartingByte.QuadPart)))
//         return _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//     /* Otherwise, key on the starting byte */
//     Result =
//         (A->Exclusive.FileLock.StartingByte.QuadPart <
//          B->Exclusive.FileLock.StartingByte.QuadPart) ? _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan :
//         (A->Exclusive.FileLock.StartingByte.QuadPart >
//          B->Exclusive.FileLock.StartingByte.QuadPart) ? _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan :
//         _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
// #if 0
//     DPRINT("Compare(%x:%x) %x-%x to %x-%x => %d\n",
//            A,B,
//            A->Exclusive.FileLock.StartingByte.LowPart,
//            A->Exclusive.FileLock.EndingByte.LowPart,
//            B->Exclusive.FileLock.StartingByte.LowPart,
//            B->Exclusive.FileLock.EndingByte.LowPart,
//            Result);
// #endif
//     return Result;
// }
//
// /* CSQ methods */
//
// static NTSTATUS NTAPI LockInsertIrpEx
// (PIO_CSQ Csq,
//  PIRP Irp,
//  PVOID InsertContext)
// {
//     PLOCK_INFORMATION LockInfo = CONTAINING_RECORD(Csq, LOCK_INFORMATION, Csq);
//     InsertTailList(&LockInfo->CsqList, &Irp->Tail.Overlay.ListEntry);
//     return STATUS_SUCCESS;
// }
//
// static VOID NTAPI LockRemoveIrp(PIO_CSQ Csq, PIRP Irp)
// {
//     RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
// }
//
// static PIRP NTAPI LockPeekNextIrp(PIO_CSQ Csq, PIRP Irp, PVOID PeekContext)
// {
//     // Context will be a COMBINED_LOCK_ELEMENT.  We're looking for a
//     // lock that can be acquired, now that the lock matching PeekContext
//     // has been removed.
//     COMBINED_LOCK_ELEMENT LockElement;
//     PCOMBINED_LOCK_ELEMENT WhereUnlock = PeekContext;
//     PLOCK_INFORMATION LockInfo = CONTAINING_RECORD(Csq, LOCK_INFORMATION, Csq);
//     PLIST_ENTRY Following;
//     DPRINT("PeekNextIrp(IRP %p, Context %p)\n", Irp, PeekContext);
//     if (!Irp)
//     {
//         Following = LockInfo->CsqList.Flink;
//     }
//     else
//         Following = Irp->Tail.Overlay.ListEntry.Flink;
//
//     DPRINT("ListEntry %p Head %p\n", Following, &LockInfo->CsqList);
//     for (;
//          Following != &LockInfo->CsqList;
//          Following = Following->Flink)
//     {
//         PIO_STACK_LOCATION IoStack;
//         BOOLEAN Matching;
//         Irp = CONTAINING_RECORD(Following, IRP, Tail.Overlay.ListEntry);
//         DPRINT("Irp %p\n", Irp);
//         IoStack = IoGetCurrentIrpStackLocation(Irp);
//         LockElement.Exclusive.FileLock.StartingByte =
//             IoStack->Parameters.LockControl.ByteOffset;
//         LockElement.Exclusive.FileLock.EndingByte.QuadPart =
//             LockElement.Exclusive.FileLock.StartingByte.QuadPart +
//             IoStack->Parameters.LockControl.Length->QuadPart;
//         /* If a context was specified, it's a range to check to unlock */
//         if (WhereUnlock)
//         {
//             Matching = LockCompare
//                 (&LockInfo->RangeTable, &LockElement, WhereUnlock) != _RTL_GENERIC_COMPARE_RESULTS_GenericEqual;
//         }
//         /* Else get any completable IRP */
//         else
//         {
//             Matching = FALSE;
//         }
//         if (!Matching)
//         {
//             // This IRP is fine...
//             DPRINT("Returning the IRP %p\n", Irp);
//             return Irp;
//         }
//     }
//     DPRINT("Return NULL\n");
//     return NULL;
// }
//
// static VOID NTAPI
// LockAcquireQueueLock(PIO_CSQ Csq, PKIRQL Irql)
// {
//     PLOCK_INFORMATION LockInfo = CONTAINING_RECORD(Csq, LOCK_INFORMATION, Csq);
//     KeAcquireSpinLock(&LockInfo->CsqLock, Irql);
// }
//
// static VOID NTAPI
// LockReleaseQueueLock(PIO_CSQ Csq, KIRQL Irql)
// {
//     PLOCK_INFORMATION LockInfo = CONTAINING_RECORD(Csq, LOCK_INFORMATION, Csq);
//     KeReleaseSpinLock(&LockInfo->CsqLock, Irql);
// }
//
// static VOID NTAPI
// LockCompleteCanceledIrp(PIO_CSQ Csq, PIRP Irp)
// {
//     NTSTATUS Status;
//     PLOCK_INFORMATION LockInfo = CONTAINING_RECORD(Csq, LOCK_INFORMATION, Csq);
//     DPRINT("Complete cancelled IRP %p Status %x\n", Irp, STATUS_CANCELLED);
//     FsRtlCompleteLockIrpReal
//         (LockInfo->BelongsTo->CompleteLockIrpRoutine,
//          NULL,
//          Irp,
//          STATUS_CANCELLED,
//          &Status,
//          NULL);
// }
//
// VOID
// NTAPI
// FsRtlCompleteLockIrpReal(IN PCOMPLETE_LOCK_IRP_ROUTINE CompleteRoutine,
//                          IN PVOID Context,
//                          IN PIRP Irp,
//                          IN NTSTATUS Status,
//                          OUT PNTSTATUS NewStatus,
//                          IN PFILE_OBJECT FileObject OPTIONAL)
// {
//     /* Check if we have a complete routine */
//     Irp->IoStatus.Information = 0;
//     if (CompleteRoutine)
//     {
//         /* Check if we have a file object */
//         if (FileObject) FileObject->LastLock = NULL;
//
//         /* Set the I/O Status and do completion */
//         Irp->IoStatus.Status = Status;
//         DPRINT("Calling completion routine %p Status %x\n", Irp, Status);
//         *NewStatus = CompleteRoutine(Context, Irp);
//     }
//     else
//     {
//         /* Otherwise do a normal I/O complete request */
//         DPRINT("Completing IRP %p Status %x\n", Irp, Status);
//         FsRtlCompleteRequest(Irp, Status);
//         *NewStatus = Status;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// PFILE_LOCK_INFO
// NTAPI
// FsRtlGetNextFileLock(IN PFILE_LOCK FileLock,
//                      IN BOOLEAN Restart)
// {
//     PCOMBINED_LOCK_ELEMENT Entry;
//     if (!FileLock->LockInformation) return NULL;
//     Entry = RtlEnumerateGenericTable(FileLock->LockInformation, Restart);
//     if (!Entry) return NULL;
//     else return &Entry->Exclusive.FileLock;
// }
//
// VOID
// NTAPI
// FsRtlpExpandLockElement
// (PCOMBINED_LOCK_ELEMENT ToExpand,
//  PCOMBINED_LOCK_ELEMENT Conflict)
// {
//     if (ToExpand->Exclusive.FileLock.StartingByte.QuadPart >
//         Conflict->Exclusive.FileLock.StartingByte.QuadPart)
//     {
//         ToExpand->Exclusive.FileLock.StartingByte =
//             Conflict->Exclusive.FileLock.StartingByte;
//     }
//     if (ToExpand->Exclusive.FileLock.EndingByte.QuadPart <
//         Conflict->Exclusive.FileLock.EndingByte.QuadPart)
//     {
//         ToExpand->Exclusive.FileLock.EndingByte =
//             Conflict->Exclusive.FileLock.EndingByte;
//     }
// }
//
// /* This function expands the conflicting range Conflict by removing and reinserting it,
//    then adds a shared range of the same size */
// PCOMBINED_LOCK_ELEMENT
// NTAPI
// FsRtlpRebuildSharedLockRange
// (PFILE_LOCK FileLock,
//  PLOCK_INFORMATION LockInfo,
//  PCOMBINED_LOCK_ELEMENT Conflict)
// {
//     /* Starting at Conflict->StartingByte and going to Conflict->EndingByte
//      * capture and expand a shared range from the shared range list.
//      * Finish when we've incorporated all overlapping shared regions.
//      */
//     BOOLEAN InsertedNew = FALSE, RemovedOld;
//     COMBINED_LOCK_ELEMENT NewElement = *Conflict;
//     PCOMBINED_LOCK_ELEMENT Entry;
//     while ((Entry = RtlLookupElementGenericTable
//             (FileLock->LockInformation, &NewElement)))
//     {
//         FsRtlpExpandLockElement(&NewElement, Entry);
//         RemovedOld = RtlDeleteElementGenericTable
//             (&LockInfo->RangeTable,
//              Entry);
//         ASSERT(RemovedOld);
//     }
//     Conflict = RtlInsertElementGenericTable
//         (&LockInfo->RangeTable,
//          &NewElement,
//          sizeof(NewElement),
//          &InsertedNew);
//     ASSERT(InsertedNew);
//     return Conflict;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlPrivateLock(IN PFILE_LOCK FileLock,
//                  IN PFILE_OBJECT FileObject,
//                  IN PLARGE_INTEGER FileOffset,
//                  IN PLARGE_INTEGER Length,
//                  IN PEPROCESS Process,
//                  IN ULONG Key,
//                  IN BOOLEAN FailImmediately,
//                  IN BOOLEAN ExclusiveLock,
//                  OUT PIO_STATUS_BLOCK IoStatus,
//                  IN PIRP Irp OPTIONAL,
//                  IN PVOID Context OPTIONAL,
//                  IN BOOLEAN AlreadySynchronized)
// {
//     NTSTATUS Status;
//     COMBINED_LOCK_ELEMENT ToInsert;
//     PCOMBINED_LOCK_ELEMENT Conflict;
//     PLOCK_INFORMATION LockInfo;
//     PLOCK_SHARED_RANGE NewSharedRange;
//     BOOLEAN InsertedNew;
//     ULARGE_INTEGER UnsignedStart;
//     ULARGE_INTEGER UnsignedEnd;
//
//     DPRINT("FsRtlPrivateLock(%wZ, Offset %08x%08x (%d), Length %08x%08x (%d), Key %x, FailImmediately %u, Exclusive %u)\n",
//            &FileObject->FileName,
//            FileOffset->HighPart,
//            FileOffset->LowPart,
//            (int)FileOffset->QuadPart,
//            Length->HighPart,
//            Length->LowPart,
//            (int)Length->QuadPart,
//            Key,
//            FailImmediately,
//            ExclusiveLock);
//
//     UnsignedStart.QuadPart = FileOffset->QuadPart;
//     UnsignedEnd.QuadPart = FileOffset->QuadPart + Length->QuadPart;
//
//     if (UnsignedEnd.QuadPart < UnsignedStart.QuadPart)
//     {
//         DPRINT("File offset out of range\n");
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         if (Irp)
//         {
//             DPRINT("Complete lock %p Status %x\n", Irp, IoStatus->Status);
//             FsRtlCompleteLockIrpReal
//                 (FileLock->CompleteLockIrpRoutine,
//                  Context,
//                  Irp,
//                  IoStatus->Status,
//                  &Status,
//                  FileObject);
//         }
//         return FALSE;
//     }
//
//     /* Initialize the lock, if necessary */
//     if (!FileLock->LockInformation)
//     {
//         LockInfo = ExAllocatePoolWithTag(NonPagedPool, sizeof(LOCK_INFORMATION), TAG_FLOCK);
//         if (!LockInfo)
//         {
//             IoStatus->Status = STATUS_NO_MEMORY;
//             return FALSE;
//         }
//         FileLock->LockInformation = LockInfo;
//
//         LockInfo->BelongsTo = FileLock;
//         InitializeListHead(&LockInfo->SharedLocks);
//
//         RtlInitializeGenericTable
//             (&LockInfo->RangeTable,
//              LockCompare,
//              LockAllocate,
//              LockFree,
//              NULL);
//
//         KeInitializeSpinLock(&LockInfo->CsqLock);
//         InitializeListHead(&LockInfo->CsqList);
//
//         IoCsqInitializeEx
//             (&LockInfo->Csq,
//              LockInsertIrpEx,
//              LockRemoveIrp,
//              LockPeekNextIrp,
//              LockAcquireQueueLock,
//              LockReleaseQueueLock,
//              LockCompleteCanceledIrp);
//     }
//
//     LockInfo = FileLock->LockInformation;
//     ToInsert.Exclusive.FileLock.FileObject = FileObject;
//     ToInsert.Exclusive.FileLock.StartingByte = *FileOffset;
//     ToInsert.Exclusive.FileLock.EndingByte.QuadPart = FileOffset->QuadPart + Length->QuadPart;
//     ToInsert.Exclusive.FileLock.ProcessId = Process;
//     ToInsert.Exclusive.FileLock.Key = Key;
//     ToInsert.Exclusive.FileLock.ExclusiveLock = ExclusiveLock;
//
//     Conflict = RtlInsertElementGenericTable
//         (FileLock->LockInformation,
//          &ToInsert,
//          sizeof(ToInsert),
//          &InsertedNew);
//
//     if (Conflict && !InsertedNew)
//     {
//         if (Conflict->Exclusive.FileLock.ExclusiveLock || ExclusiveLock)
//         {
//             DPRINT("Conflict %08x%08x:%08x%08x Exc %u (Want Exc %u)\n",
//                    Conflict->Exclusive.FileLock.StartingByte.HighPart,
//                    Conflict->Exclusive.FileLock.StartingByte.LowPart,
//                    Conflict->Exclusive.FileLock.EndingByte.HighPart,
//                    Conflict->Exclusive.FileLock.EndingByte.LowPart,
//                    Conflict->Exclusive.FileLock.ExclusiveLock,
//                    ExclusiveLock);
//             if (FailImmediately)
//             {
//                 DPRINT("STATUS_FILE_LOCK_CONFLICT\n");
//                 IoStatus->Status = STATUS_FILE_LOCK_CONFLICT;
//                 if (Irp)
//                 {
//                     DPRINT("STATUS_FILE_LOCK_CONFLICT: Complete\n");
//                     FsRtlCompleteLockIrpReal
//                         (FileLock->CompleteLockIrpRoutine,
//                          Context,
//                          Irp,
//                          IoStatus->Status,
//                          &Status,
//                          FileObject);
//                 }
//                 return FALSE;
//             }
//             else
//             {
//                 IoStatus->Status = STATUS_PENDING;
//                 if (Irp)
//                 {
//                     Irp->IoStatus.Information = LockInfo->Generation;
//                     IoMarkIrpPending(Irp);
//                     IoCsqInsertIrpEx
//                         (&LockInfo->Csq,
//                          Irp,
//                          NULL,
//                          NULL);
//                 }
//             }
//             return FALSE;
//         }
//         else
//         {
//             ULONG i;
//             /* We know of at least one lock in range that's shared.  We need to
//              * find out if any more exist and any are exclusive. */
//             for (i = 0; i < RtlNumberGenericTableElements(&LockInfo->RangeTable); i++)
//             {
//                 Conflict = RtlGetElementGenericTable(&LockInfo->RangeTable, i);
//
//                 /* The first argument will be inserted as a shared range */
//                 if (Conflict && (LockCompare(&LockInfo->RangeTable, Conflict, &ToInsert) == _RTL_GENERIC_COMPARE_RESULTS_GenericEqual))
//                 {
//                     if (Conflict->Exclusive.FileLock.ExclusiveLock)
//                     {
//                         /* Found an exclusive match */
//                         if (FailImmediately)
//                         {
//                             IoStatus->Status = STATUS_FILE_LOCK_CONFLICT;
//                             DPRINT("STATUS_FILE_LOCK_CONFLICT\n");
//                             if (Irp)
//                             {
//                                 DPRINT("STATUS_FILE_LOCK_CONFLICT: Complete\n");
//                                 FsRtlCompleteLockIrpReal
//                                     (FileLock->CompleteLockIrpRoutine,
//                                      Context,
//                                      Irp,
//                                      IoStatus->Status,
//                                      &Status,
//                                      FileObject);
//                             }
//                         }
//                         else
//                         {
//                             IoStatus->Status = STATUS_PENDING;
//                             if (Irp)
//                             {
//                                 IoMarkIrpPending(Irp);
//                                 IoCsqInsertIrpEx
//                                     (&LockInfo->Csq,
//                                      Irp,
//                                      NULL,
//                                      NULL);
//                             }
//                         }
//                         return FALSE;
//                     }
//                 }
//             }
//
//             DPRINT("Overlapping shared lock %wZ %08x%08x %08x%08x\n",
//                    &FileObject->FileName,
//                    Conflict->Exclusive.FileLock.StartingByte.HighPart,
//                    Conflict->Exclusive.FileLock.StartingByte.LowPart,
//                    Conflict->Exclusive.FileLock.EndingByte.HighPart,
//                    Conflict->Exclusive.FileLock.EndingByte.LowPart);
//             Conflict = FsRtlpRebuildSharedLockRange(FileLock,
//                                                     LockInfo,
//                                                     &ToInsert);
//             if (!Conflict)
//             {
//                 IoStatus->Status = STATUS_NO_MEMORY;
//                 if (Irp)
//                 {
//                     FsRtlCompleteLockIrpReal
//                         (FileLock->CompleteLockIrpRoutine,
//                          Context,
//                          Irp,
//                          IoStatus->Status,
//                          &Status,
//                          FileObject);
//                 }
//             }
//
//             /* We got here because there were only overlapping shared locks */
//             /* A shared lock is both a range *and* a list entry.  Insert the
//                entry here. */
//
//             DPRINT("Adding shared lock %wZ\n", &FileObject->FileName);
//             NewSharedRange =
//                 ExAllocatePoolWithTag(NonPagedPool, sizeof(*NewSharedRange), TAG_RANGE);
//             if (!NewSharedRange)
//             {
//                 IoStatus->Status = STATUS_NO_MEMORY;
//                 if (Irp)
//                 {
//                     FsRtlCompleteLockIrpReal
//                         (FileLock->CompleteLockIrpRoutine,
//                          Context,
//                          Irp,
//                          IoStatus->Status,
//                          &Status,
//                          FileObject);
//                 }
//                 return FALSE;
//             }
//             DPRINT("Adding shared lock %wZ\n", &FileObject->FileName);
//             NewSharedRange->Start = *FileOffset;
//             NewSharedRange->End.QuadPart = FileOffset->QuadPart + Length->QuadPart;
//             NewSharedRange->Key = Key;
//             NewSharedRange->ProcessId = ToInsert.Exclusive.FileLock.ProcessId;
//             InsertTailList(&LockInfo->SharedLocks, &NewSharedRange->Entry);
//
//             DPRINT("Acquired shared lock %wZ %08x%08x %08x%08x\n",
//                    &FileObject->FileName,
//                    Conflict->Exclusive.FileLock.StartingByte.HighPart,
//                    Conflict->Exclusive.FileLock.StartingByte.LowPart,
//                    Conflict->Exclusive.FileLock.EndingByte.HighPart,
//                    Conflict->Exclusive.FileLock.EndingByte.LowPart);
//             IoStatus->Status = STATUS_SUCCESS;
//             if (Irp)
//             {
//                 FsRtlCompleteLockIrpReal
//                     (FileLock->CompleteLockIrpRoutine,
//                      Context,
//                      Irp,
//                      IoStatus->Status,
//                      &Status,
//                      FileObject);
//             }
//             return TRUE;
//         }
//     }
//     else if (!Conflict)
//     {
//         /* Conflict here is (or would be) the newly inserted element, but we ran
//          * out of space probably. */
//         IoStatus->Status = STATUS_NO_MEMORY;
//         if (Irp)
//         {
//             FsRtlCompleteLockIrpReal
//                 (FileLock->CompleteLockIrpRoutine,
//                  Context,
//                  Irp,
//                  IoStatus->Status,
//                  &Status,
//                  FileObject);
//         }
//         return FALSE;
//     }
//     else
//     {
//         DPRINT("Inserted new lock %wZ %08x%08x %08x%08x exclusive %u\n",
//                &FileObject->FileName,
//                Conflict->Exclusive.FileLock.StartingByte.HighPart,
//                Conflict->Exclusive.FileLock.StartingByte.LowPart,
//                Conflict->Exclusive.FileLock.EndingByte.HighPart,
//                Conflict->Exclusive.FileLock.EndingByte.LowPart,
//                Conflict->Exclusive.FileLock.ExclusiveLock);
//         if (!ExclusiveLock)
//         {
//             NewSharedRange =
//                 ExAllocatePoolWithTag(NonPagedPool, sizeof(*NewSharedRange), TAG_RANGE);
//             if (!NewSharedRange)
//             {
//                 IoStatus->Status = STATUS_NO_MEMORY;
//                 if (Irp)
//                 {
//                     FsRtlCompleteLockIrpReal
//                         (FileLock->CompleteLockIrpRoutine,
//                          Context,
//                          Irp,
//                          IoStatus->Status,
//                          &Status,
//                          FileObject);
//                 }
//                 return FALSE;
//             }
//             DPRINT("Adding shared lock %wZ\n", &FileObject->FileName);
//             NewSharedRange->Start = *FileOffset;
//             NewSharedRange->End.QuadPart = FileOffset->QuadPart + Length->QuadPart;
//             NewSharedRange->Key = Key;
//             NewSharedRange->ProcessId = Process;
//             InsertTailList(&LockInfo->SharedLocks, &NewSharedRange->Entry);
//         }
//
//         /* Assume all is cool, and lock is set */
//         IoStatus->Status = STATUS_SUCCESS;
//
//         if (Irp)
//         {
//             /* Complete the request */
//             FsRtlCompleteLockIrpReal(FileLock->CompleteLockIrpRoutine,
//                                      Context,
//                                      Irp,
//                                      IoStatus->Status,
//                                      &Status,
//                                      FileObject);
//
//             /* Update the status */
//             IoStatus->Status = Status;
//         }
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlCheckLockForReadAccess(IN PFILE_LOCK FileLock,
//                             IN PIRP Irp)
// {
//     BOOLEAN Result;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//     COMBINED_LOCK_ELEMENT ToFind;
//     PCOMBINED_LOCK_ELEMENT Found;
//     DPRINT("CheckLockForReadAccess(%wZ, Offset %08x%08x, Length %x)\n",
//            &IoStack->FileObject->FileName,
//            IoStack->Parameters.Read.ByteOffset.HighPart,
//            IoStack->Parameters.Read.ByteOffset.LowPart,
//            IoStack->Parameters.Read.Length);
//     if (!FileLock->LockInformation) {
//         DPRINT("CheckLockForReadAccess(%wZ) => TRUE\n", &IoStack->FileObject->FileName);
//         return TRUE;
//     }
//     ToFind.Exclusive.FileLock.StartingByte = IoStack->Parameters.Read.ByteOffset;
//     ToFind.Exclusive.FileLock.EndingByte.QuadPart =
//         ToFind.Exclusive.FileLock.StartingByte.QuadPart +
//         IoStack->Parameters.Read.Length;
//     Found = RtlLookupElementGenericTable
//         (FileLock->LockInformation,
//          &ToFind);
//     if (!Found) {
//         DPRINT("CheckLockForReadAccess(%wZ) => TRUE\n", &IoStack->FileObject->FileName);
//         return TRUE;
//     }
//     Result = !Found->Exclusive.FileLock.ExclusiveLock ||
//         IoStack->Parameters.Read.Key == Found->Exclusive.FileLock.Key;
//     DPRINT("CheckLockForReadAccess(%wZ) => %s\n", &IoStack->FileObject->FileName, Result ? "TRUE" : "FALSE");
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlCheckLockForWriteAccess(IN PFILE_LOCK FileLock,
//                              IN PIRP Irp)
// {
//     BOOLEAN Result;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//     COMBINED_LOCK_ELEMENT ToFind;
//     PCOMBINED_LOCK_ELEMENT Found;
//     PEPROCESS Process = Irp->Tail.Overlay.Thread->ThreadsProcess;
//     DPRINT("CheckLockForWriteAccess(%wZ, Offset %08x%08x, Length %x)\n",
//            &IoStack->FileObject->FileName,
//            IoStack->Parameters.Write.ByteOffset.HighPart,
//            IoStack->Parameters.Write.ByteOffset.LowPart,
//            IoStack->Parameters.Write.Length);
//     if (!FileLock->LockInformation) {
//         DPRINT("CheckLockForWriteAccess(%wZ) => TRUE\n", &IoStack->FileObject->FileName);
//         return TRUE;
//     }
//     ToFind.Exclusive.FileLock.StartingByte = IoStack->Parameters.Write.ByteOffset;
//     ToFind.Exclusive.FileLock.EndingByte.QuadPart =
//         ToFind.Exclusive.FileLock.StartingByte.QuadPart +
//         IoStack->Parameters.Write.Length;
//     Found = RtlLookupElementGenericTable
//         (FileLock->LockInformation,
//          &ToFind);
//     if (!Found) {
//         DPRINT("CheckLockForWriteAccess(%wZ) => TRUE\n", &IoStack->FileObject->FileName);
//         return TRUE;
//     }
//     Result = Process == Found->Exclusive.FileLock.ProcessId;
//     DPRINT("CheckLockForWriteAccess(%wZ) => %s\n", &IoStack->FileObject->FileName, Result ? "TRUE" : "FALSE");
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlFastCheckLockForRead(IN PFILE_LOCK FileLock,
//                           IN PLARGE_INTEGER FileOffset,
//                           IN PLARGE_INTEGER Length,
//                           IN ULONG Key,
//                           IN PFILE_OBJECT FileObject,
//                           IN PVOID Process)
// {
//     PEPROCESS EProcess = Process;
//     COMBINED_LOCK_ELEMENT ToFind;
//     PCOMBINED_LOCK_ELEMENT Found;
//     DPRINT("FsRtlFastCheckLockForRead(%wZ, Offset %08x%08x, Length %08x%08x, Key %x)\n",
//            &FileObject->FileName,
//            FileOffset->HighPart,
//            FileOffset->LowPart,
//            Length->HighPart,
//            Length->LowPart,
//            Key);
//     ToFind.Exclusive.FileLock.StartingByte = *FileOffset;
//     ToFind.Exclusive.FileLock.EndingByte.QuadPart =
//         FileOffset->QuadPart + Length->QuadPart;
//     if (!FileLock->LockInformation) return TRUE;
//     Found = RtlLookupElementGenericTable
//         (FileLock->LockInformation,
//          &ToFind);
//     if (!Found || !Found->Exclusive.FileLock.ExclusiveLock) return TRUE;
//     return Found->Exclusive.FileLock.Key == Key &&
//         Found->Exclusive.FileLock.ProcessId == EProcess;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlFastCheckLockForWrite(IN PFILE_LOCK FileLock,
//                            IN PLARGE_INTEGER FileOffset,
//                            IN PLARGE_INTEGER Length,
//                            IN ULONG Key,
//                            IN PFILE_OBJECT FileObject,
//                            IN PVOID Process)
// {
//     BOOLEAN Result;
//     PEPROCESS EProcess = Process;
//     COMBINED_LOCK_ELEMENT ToFind;
//     PCOMBINED_LOCK_ELEMENT Found;
//     DPRINT("FsRtlFastCheckLockForWrite(%wZ, Offset %08x%08x, Length %08x%08x, Key %x)\n",
//            &FileObject->FileName,
//            FileOffset->HighPart,
//            FileOffset->LowPart,
//            Length->HighPart,
//            Length->LowPart,
//            Key);
//     ToFind.Exclusive.FileLock.StartingByte = *FileOffset;
//     ToFind.Exclusive.FileLock.EndingByte.QuadPart =
//         FileOffset->QuadPart + Length->QuadPart;
//     if (!FileLock->LockInformation) {
//         DPRINT("CheckForWrite(%wZ) => TRUE\n", &FileObject->FileName);
//         return TRUE;
//     }
//     Found = RtlLookupElementGenericTable
//         (FileLock->LockInformation,
//          &ToFind);
//     if (!Found) {
//         DPRINT("CheckForWrite(%wZ) => TRUE\n", &FileObject->FileName);
//         return TRUE;
//     }
//     Result = Found->Exclusive.FileLock.Key == Key &&
//         Found->Exclusive.FileLock.ProcessId == EProcess;
//     DPRINT("CheckForWrite(%wZ) => %s\n", &FileObject->FileName, Result ? "TRUE" : "FALSE");
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlFastUnlockSingle(IN PFILE_LOCK FileLock,
//                       IN PFILE_OBJECT FileObject,
//                       IN PLARGE_INTEGER FileOffset,
//                       IN PLARGE_INTEGER Length,
//                       IN PEPROCESS Process,
//                       IN ULONG Key,
//                       IN PVOID Context OPTIONAL,
//                       IN BOOLEAN AlreadySynchronized)
// {
//     BOOLEAN FoundShared = FALSE;
//     PLIST_ENTRY SharedEntry;
//     PLOCK_SHARED_RANGE SharedRange = NULL;
//     COMBINED_LOCK_ELEMENT Find;
//     PCOMBINED_LOCK_ELEMENT Entry;
//     PIRP NextMatchingLockIrp;
//     PLOCK_INFORMATION InternalInfo = FileLock->LockInformation;
//     DPRINT("FsRtlFastUnlockSingle(%wZ, Offset %08x%08x (%d), Length %08x%08x (%d), Key %x)\n",
//            &FileObject->FileName,
//            FileOffset->HighPart,
//            FileOffset->LowPart,
//            (int)FileOffset->QuadPart,
//            Length->HighPart,
//            Length->LowPart,
//            (int)Length->QuadPart,
//            Key);
//     // The region to unlock must correspond exactly to a previously locked region
//     // -- msdn
//     // But Windows 2003 doesn't assert on it and simply ignores that parameter
//     // ASSERT(AlreadySynchronized);
//     Find.Exclusive.FileLock.StartingByte = *FileOffset;
//     Find.Exclusive.FileLock.EndingByte.QuadPart =
//         FileOffset->QuadPart + Length->QuadPart;
//     if (!InternalInfo) {
//         DPRINT("File not previously locked (ever)\n");
//         return STATUS_RANGE_NOT_LOCKED;
//     }
//     Entry = RtlLookupElementGenericTable(&InternalInfo->RangeTable, &Find);
//     if (!Entry) {
//         DPRINT("Range not locked %wZ\n", &FileObject->FileName);
//         return STATUS_RANGE_NOT_LOCKED;
//     }
//
//     DPRINT("Found lock entry: Exclusive %u %08x%08x:%08x%08x %wZ\n",
//            Entry->Exclusive.FileLock.ExclusiveLock,
//            Entry->Exclusive.FileLock.StartingByte.HighPart,
//            Entry->Exclusive.FileLock.StartingByte.LowPart,
//            Entry->Exclusive.FileLock.EndingByte.HighPart,
//            Entry->Exclusive.FileLock.EndingByte.LowPart,
//            &FileObject->FileName);
//
//     if (Entry->Exclusive.FileLock.ExclusiveLock)
//     {
//         if (Entry->Exclusive.FileLock.Key != Key ||
//             Entry->Exclusive.FileLock.ProcessId != Process ||
//             Entry->Exclusive.FileLock.StartingByte.QuadPart != FileOffset->QuadPart ||
//             Entry->Exclusive.FileLock.EndingByte.QuadPart !=
//             FileOffset->QuadPart + Length->QuadPart)
//         {
//             DPRINT("Range not locked %wZ\n", &FileObject->FileName);
//             return STATUS_RANGE_NOT_LOCKED;
//         }
//         RtlCopyMemory(&Find, Entry, sizeof(Find));
//         // Remove the old exclusive lock region
//         RtlDeleteElementGenericTable(&InternalInfo->RangeTable, Entry);
//     }
//     else
//     {
//         DPRINT("Shared lock %wZ Start %08x%08x End %08x%08x\n",
//                &FileObject->FileName,
//                Entry->Exclusive.FileLock.StartingByte.HighPart,
//                Entry->Exclusive.FileLock.StartingByte.LowPart,
//                Entry->Exclusive.FileLock.EndingByte.HighPart,
//                Entry->Exclusive.FileLock.EndingByte.LowPart);
//         for (SharedEntry = InternalInfo->SharedLocks.Flink;
//              SharedEntry != &InternalInfo->SharedLocks;
//              SharedEntry = SharedEntry->Flink)
//         {
//             SharedRange = CONTAINING_RECORD(SharedEntry, LOCK_SHARED_RANGE, Entry);
//             if (SharedRange->Start.QuadPart == FileOffset->QuadPart &&
//                 SharedRange->End.QuadPart == FileOffset->QuadPart + Length->QuadPart &&
//                 SharedRange->Key == Key &&
//                 SharedRange->ProcessId == Process)
//             {
//                 FoundShared = TRUE;
//                 DPRINT("Found shared element to delete %wZ Start %08x%08x End %08x%08x Key %x\n",
//                        &FileObject->FileName,
//                        SharedRange->Start.HighPart,
//                        SharedRange->Start.LowPart,
//                        SharedRange->End.HighPart,
//                        SharedRange->End.LowPart,
//                        SharedRange->Key);
//                 break;
//             }
//         }
//         if (FoundShared)
//         {
//             /* Remove the found range from the shared range lists */
//             RemoveEntryList(&SharedRange->Entry);
//             ExFreePoolWithTag(SharedRange, TAG_RANGE);
//             /* We need to rebuild the list of shared ranges. */
//             DPRINT("Removing the lock entry %wZ (%08x%08x:%08x%08x)\n",
//                    &FileObject->FileName,
//                    Entry->Exclusive.FileLock.StartingByte.HighPart,
//                    Entry->Exclusive.FileLock.StartingByte.LowPart,
//                    Entry->Exclusive.FileLock.EndingByte.HighPart,
//                    Entry->Exclusive.FileLock.EndingByte.LowPart);
//
//             /* Remember what was in there and remove it from the table */
//             Find = *Entry;
//             RtlDeleteElementGenericTable(&InternalInfo->RangeTable, &Find);
//             /* Put shared locks back in place */
//             for (SharedEntry = InternalInfo->SharedLocks.Flink;
//                  SharedEntry != &InternalInfo->SharedLocks;
//                  SharedEntry = SharedEntry->Flink)
//             {
//                 COMBINED_LOCK_ELEMENT LockElement;
//                 SharedRange = CONTAINING_RECORD(SharedEntry, LOCK_SHARED_RANGE, Entry);
//                 LockElement.Exclusive.FileLock.FileObject = FileObject;
//                 LockElement.Exclusive.FileLock.StartingByte = SharedRange->Start;
//                 LockElement.Exclusive.FileLock.EndingByte = SharedRange->End;
//                 LockElement.Exclusive.FileLock.ProcessId = SharedRange->ProcessId;
//                 LockElement.Exclusive.FileLock.Key = SharedRange->Key;
//                 LockElement.Exclusive.FileLock.ExclusiveLock = FALSE;
//
//                 if (LockCompare(&InternalInfo->RangeTable, &Find, &LockElement) != _RTL_GENERIC_COMPARE_RESULTS_GenericEqual)
//                 {
//                     DPRINT("Skipping range %08x%08x:%08x%08x\n",
//                            LockElement.Exclusive.FileLock.StartingByte.HighPart,
//                            LockElement.Exclusive.FileLock.StartingByte.LowPart,
//                            LockElement.Exclusive.FileLock.EndingByte.HighPart,
//                            LockElement.Exclusive.FileLock.EndingByte.LowPart);
//                     continue;
//                 }
//                 DPRINT("Re-creating range %08x%08x:%08x%08x\n",
//                        LockElement.Exclusive.FileLock.StartingByte.HighPart,
//                        LockElement.Exclusive.FileLock.StartingByte.LowPart,
//                        LockElement.Exclusive.FileLock.EndingByte.HighPart,
//                        LockElement.Exclusive.FileLock.EndingByte.LowPart);
//                 FsRtlpRebuildSharedLockRange(FileLock, InternalInfo, &LockElement);
//             }
//         }
//         else
//         {
//             return STATUS_RANGE_NOT_LOCKED;
//         }
//     }
//
// #ifndef NDEBUG
//     DPRINT("Lock still has:\n");
//     for (SharedEntry = InternalInfo->SharedLocks.Flink;
//          SharedEntry != &InternalInfo->SharedLocks;
//          SharedEntry = SharedEntry->Flink)
//     {
//         SharedRange = CONTAINING_RECORD(SharedEntry, LOCK_SHARED_RANGE, Entry);
//         DPRINT("Shared element %wZ Offset %08x%08x Length %08x%08x Key %x\n",
//                &FileObject->FileName,
//                SharedRange->Start.HighPart,
//                SharedRange->Start.LowPart,
//                SharedRange->End.HighPart,
//                SharedRange->End.LowPart,
//                SharedRange->Key);
//     }
// #endif
//
//     // this is definitely the thing we want
//     InternalInfo->Generation++;
//     while ((NextMatchingLockIrp = IoCsqRemoveNextIrp(&InternalInfo->Csq, &Find)))
//     {
//         NTSTATUS Status;
//         if (NextMatchingLockIrp->IoStatus.Information == InternalInfo->Generation)
//         {
//             // We've already looked at this one, meaning that we looped.
//             // Put it back and exit.
//             IoCsqInsertIrpEx
//                 (&InternalInfo->Csq,
//                  NextMatchingLockIrp,
//                  NULL,
//                  NULL);
//             break;
//         }
//         // Got a new lock irp... try to do the new lock operation
//         // Note that we pick an operation that would succeed at the time
//         // we looked, but can't guarantee that it won't just be re-queued
//         // because somebody else snatched part of the range in a new thread.
//         DPRINT("Locking another IRP %p for %p %wZ\n",
//                NextMatchingLockIrp, FileLock, &FileObject->FileName);
//         Status = FsRtlProcessFileLock(InternalInfo->BelongsTo, NextMatchingLockIrp, NULL);
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     DPRINT("Success %wZ\n", &FileObject->FileName);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlFastUnlockAll(IN PFILE_LOCK FileLock,
//                    IN PFILE_OBJECT FileObject,
//                    IN PEPROCESS Process,
//                    IN PVOID Context OPTIONAL)
// {
//     PLIST_ENTRY ListEntry;
//     PCOMBINED_LOCK_ELEMENT Entry;
//     PLOCK_INFORMATION InternalInfo = FileLock->LockInformation;
//     DPRINT("FsRtlFastUnlockAll(%wZ)\n", &FileObject->FileName);
//     // XXX Synchronize somehow
//     if (!FileLock->LockInformation) {
//         DPRINT("Not locked %wZ\n", &FileObject->FileName);
//         return STATUS_RANGE_NOT_LOCKED; // no locks
//     }
//     for (ListEntry = InternalInfo->SharedLocks.Flink;
//          ListEntry != &InternalInfo->SharedLocks;)
//     {
//         LARGE_INTEGER Length;
//         PLOCK_SHARED_RANGE Range = CONTAINING_RECORD(ListEntry, LOCK_SHARED_RANGE, Entry);
//         Length.QuadPart = Range->End.QuadPart - Range->Start.QuadPart;
//         ListEntry = ListEntry->Flink;
//         if (Range->ProcessId != Process)
//             continue;
//         FsRtlFastUnlockSingle
//             (FileLock,
//              FileObject,
//              &Range->Start,
//              &Length,
//              Range->ProcessId,
//              Range->Key,
//              Context,
//              TRUE);
//     }
//     for (Entry = RtlEnumerateGenericTable(&InternalInfo->RangeTable, TRUE);
//          Entry;
//          Entry = RtlEnumerateGenericTable(&InternalInfo->RangeTable, FALSE))
//     {
//         LARGE_INTEGER Length;
//         // We'll take the first one to be the list head, and free the others first...
//         Length.QuadPart =
//             Entry->Exclusive.FileLock.EndingByte.QuadPart -
//             Entry->Exclusive.FileLock.StartingByte.QuadPart;
//         FsRtlFastUnlockSingle
//             (FileLock,
//              Entry->Exclusive.FileLock.FileObject,
//              &Entry->Exclusive.FileLock.StartingByte,
//              &Length,
//              Entry->Exclusive.FileLock.ProcessId,
//              Entry->Exclusive.FileLock.Key,
//              Context,
//              TRUE);
//     }
//     DPRINT("Done %wZ\n", &FileObject->FileName);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlFastUnlockAllByKey(IN PFILE_LOCK FileLock,
//                         IN PFILE_OBJECT FileObject,
//                         IN PEPROCESS Process,
//                         IN ULONG Key,
//                         IN PVOID Context OPTIONAL)
// {
//     PLIST_ENTRY ListEntry;
//     PCOMBINED_LOCK_ELEMENT Entry;
//     PLOCK_INFORMATION InternalInfo = FileLock->LockInformation;
//
//     DPRINT("FsRtlFastUnlockAllByKey(%wZ,Key %x)\n", &FileObject->FileName, Key);
//
//     // XXX Synchronize somehow
//     if (!FileLock->LockInformation) return STATUS_RANGE_NOT_LOCKED; // no locks
//     for (ListEntry = InternalInfo->SharedLocks.Flink;
//          ListEntry != &InternalInfo->SharedLocks;)
//     {
//         PLOCK_SHARED_RANGE Range = CONTAINING_RECORD(ListEntry, LOCK_SHARED_RANGE, Entry);
//         LARGE_INTEGER Length;
//         Length.QuadPart = Range->End.QuadPart - Range->Start.QuadPart;
//         ListEntry = ListEntry->Flink;
//         if (Range->ProcessId != Process ||
//             Range->Key != Key)
//             continue;
//         FsRtlFastUnlockSingle
//             (FileLock,
//              FileObject,
//              &Range->Start,
//              &Length,
//              Range->ProcessId,
//              Range->Key,
//              Context,
//              TRUE);
//     }
//     for (Entry = RtlEnumerateGenericTable(&InternalInfo->RangeTable, TRUE);
//          Entry;
//          Entry = RtlEnumerateGenericTable(&InternalInfo->RangeTable, FALSE))
//     {
//         LARGE_INTEGER Length;
//         // We'll take the first one to be the list head, and free the others first...
//         Length.QuadPart =
//             Entry->Exclusive.FileLock.EndingByte.QuadPart -
//             Entry->Exclusive.FileLock.StartingByte.QuadPart;
//         if (Entry->Exclusive.FileLock.Key == Key &&
//             Entry->Exclusive.FileLock.ProcessId == Process)
//         {
//             FsRtlFastUnlockSingle
//                 (FileLock,
//                  Entry->Exclusive.FileLock.FileObject,
//                  &Entry->Exclusive.FileLock.StartingByte,
//                  &Length,
//                  Entry->Exclusive.FileLock.ProcessId,
//                  Entry->Exclusive.FileLock.Key,
//                  Context,
//                  TRUE);
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlProcessFileLock(IN PFILE_LOCK FileLock,
//                      IN PIRP Irp,
//                      IN PVOID Context OPTIONAL)
// {
//     PIO_STACK_LOCATION IoStackLocation;
//     NTSTATUS Status;
//     _IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Get the I/O Stack location */
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//     ASSERT(IoStackLocation->MajorFunction == IRP_MJ_LOCK_CONTROL);
//
//     /* Clear the I/O status block and check what function this is */
//     IoStatusBlock.Information = 0;
//
//     DPRINT("FsRtlProcessFileLock(%wZ, MinorFunction %x)\n",
//            &IoStackLocation->FileObject->FileName,
//            IoStackLocation->MinorFunction);
//
//     switch(IoStackLocation->MinorFunction)
//     {
//         /* A lock */
//     case IRP_MN_LOCK:
//     {
//         /* Call the private lock routine */
//         BOOLEAN Result = FsRtlPrivateLock(FileLock,
//                                           IoStackLocation->FileObject,
//                                           &IoStackLocation->Parameters.LockControl.ByteOffset,
//                                           IoStackLocation->Parameters.LockControl.Length,
//                                           IoGetRequestorProcess(Irp),
//                                           IoStackLocation->Parameters.LockControl.Key,
//                                           IoStackLocation->Flags & SL_FAIL_IMMEDIATELY,
//                                           IoStackLocation->Flags & SL_EXCLUSIVE_LOCK,
//                                           &IoStatusBlock,
//                                           Irp,
//                                           Context,
//                                           FALSE);
//         /* FsRtlPrivateLock has _Must_inspect_result_. Just check this is consistent on debug builds */
//         NT_ASSERT(Result == NT_SUCCESS(IoStatusBlock.Status));
//         (void)Result;
//         return IoStatusBlock.Status;
//     }
//         /* A single unlock */
//     case IRP_MN_UNLOCK_SINGLE:
//
//         /* Call fast unlock */
//         IoStatusBlock.Status =
//             FsRtlFastUnlockSingle(FileLock,
//                                   IoStackLocation->FileObject,
//                                   &IoStackLocation->Parameters.LockControl.
//                                   ByteOffset,
//                                   IoStackLocation->Parameters.LockControl.
//                                   Length,
//                                   IoGetRequestorProcess(Irp),
//                                   IoStackLocation->Parameters.LockControl.
//                                   Key,
//                                   Context,
//                                   FALSE);
//         break;
//
//         /* Total unlock */
//     case IRP_MN_UNLOCK_ALL:
//
//         /* Do a fast unlock */
//         IoStatusBlock.Status = FsRtlFastUnlockAll(FileLock,
//                                                   IoStackLocation->
//                                                   FileObject,
//                                                   IoGetRequestorProcess(Irp),
//                                                   Context);
//         break;
//
//         /* Unlock by key */
//     case IRP_MN_UNLOCK_ALL_BY_KEY:
//
//         /* Do it */
//         IoStatusBlock.Status =
//             FsRtlFastUnlockAllByKey(FileLock,
//                                     IoStackLocation->FileObject,
//                                     IoGetRequestorProcess(Irp),
//                                     IoStackLocation->Parameters.
//                                     LockControl.Key,
//                                     Context);
//         break;
//
//         /* Invalid request */
//     default:
//
//         /* Complete it */
//         FsRtlCompleteRequest(Irp, STATUS_INVALID_DEVICE_REQUEST);
//         IoStatusBlock.Status = STATUS_INVALID_DEVICE_REQUEST;
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Return the status */
//     DPRINT("Lock IRP %p %x\n", Irp, IoStatusBlock.Status);
//     FsRtlCompleteLockIrpReal
//         (FileLock->CompleteLockIrpRoutine,
//          Context,
//          Irp,
//          IoStatusBlock.Status,
//          &Status,
//          NULL);
//     return IoStatusBlock.Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlInitializeFileLock (IN PFILE_LOCK FileLock,
//                          IN PCOMPLETE_LOCK_IRP_ROUTINE CompleteLockIrpRoutine OPTIONAL,
//                          IN PUNLOCK_ROUTINE UnlockRoutine OPTIONAL)
// {
//     /* Setup the lock */
//     RtlZeroMemory(FileLock, sizeof(*FileLock));
//     FileLock->FastIoIsQuestionable = FALSE;
//     FileLock->CompleteLockIrpRoutine = CompleteLockIrpRoutine;
//     FileLock->UnlockRoutine = UnlockRoutine;
//     FileLock->LockInformation = NULL;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlUninitializeFileLock(IN PFILE_LOCK FileLock)
// {
//     if (FileLock->LockInformation)
//     {
//         PIRP Irp;
//         PLOCK_INFORMATION InternalInfo = FileLock->LockInformation;
//         PCOMBINED_LOCK_ELEMENT Entry;
//         PLIST_ENTRY SharedEntry;
//         PLOCK_SHARED_RANGE SharedRange;
//         // MSDN: this completes any remaining lock IRPs
//         for (SharedEntry = InternalInfo->SharedLocks.Flink;
//              SharedEntry != &InternalInfo->SharedLocks;)
//         {
//             SharedRange = CONTAINING_RECORD(SharedEntry, LOCK_SHARED_RANGE, Entry);
//             SharedEntry = SharedEntry->Flink;
//             RemoveEntryList(&SharedRange->Entry);
//             ExFreePoolWithTag(SharedRange, TAG_RANGE);
//         }
//         while ((Entry = RtlGetElementGenericTable(&InternalInfo->RangeTable, 0)) != NULL)
//         {
//             RtlDeleteElementGenericTable(&InternalInfo->RangeTable, Entry);
//         }
//         while ((Irp = IoCsqRemoveNextIrp(&InternalInfo->Csq, NULL)) != NULL)
//         {
//             NTSTATUS Status = FsRtlProcessFileLock(FileLock, Irp, NULL);
//             /* FsRtlProcessFileLock has _Must_inspect_result_ */
//             NT_ASSERT(NT_SUCCESS(Status));
//             (void)Status;
//         }
//         ExFreePoolWithTag(InternalInfo, TAG_FLOCK);
//         FileLock->LockInformation = NULL;
//     }
// }
//
// /*
//  * @implemented
//  */
// PFILE_LOCK
// NTAPI
// FsRtlAllocateFileLock(IN PCOMPLETE_LOCK_IRP_ROUTINE CompleteLockIrpRoutine OPTIONAL,
//                       IN PUNLOCK_ROUTINE UnlockRoutine OPTIONAL)
// {
//     PFILE_LOCK FileLock;
//
//     /* Try to allocate it */
//     FileLock = ExAllocateFromPagedLookasideList(&FsRtlFileLockLookasideList);
//     if (FileLock)
//     {
//         /* Initialize it */
//         FsRtlInitializeFileLock(FileLock,
//                                 CompleteLockIrpRoutine,
//                                 UnlockRoutine);
//     }
//
//     /* Return the lock */
//     return FileLock;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlFreeFileLock(IN PFILE_LOCK FileLock)
// {
//     /* Uninitialize and free the lock */
//     FsRtlUninitializeFileLock(FileLock);
//     ExFreeToPagedLookasideList(&FsRtlFileLockLookasideList, FileLock);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<LOCK_INFORMATION>() == 192);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, RangeTable) == 0);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, Csq) == 72);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, CsqLock) == 136);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, CsqList) == 144);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, BelongsTo) == 160);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, SharedLocks) == 168);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, Generation) == 184);
    // LOCK_SHARED_RANGE is private to this file; FileObject is our addition to the C.
    assert!(core::mem::size_of::<LOCK_SHARED_RANGE>() == 56);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Entry) == 0);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Start) == 16);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, End) == 24);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Key) == 32);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, ProcessId) == 40);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, FileObject) == 48);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<LOCK_INFORMATION>() == 100);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, RangeTable) == 0);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, Csq) == 40);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, CsqLock) == 72);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, CsqList) == 76);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, BelongsTo) == 84);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, SharedLocks) == 88);
    assert!(core::mem::offset_of!(LOCK_INFORMATION, Generation) == 96);
    assert!(core::mem::size_of::<LOCK_SHARED_RANGE>() == 40);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Entry) == 0);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Start) == 8);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, End) == 16);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, Key) == 24);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, ProcessId) == 28);
    assert!(core::mem::offset_of!(LOCK_SHARED_RANGE, FileObject) == 32);
};
// END OFFSET CHECKS
