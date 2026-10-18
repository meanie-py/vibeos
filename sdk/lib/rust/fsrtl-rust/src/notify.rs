use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _FILE_NOTIFY_INFORMATION,
    _IRP,
    _IO_STACK_LOCATION,
    _NOTIFY_CHANGE,
    _REAL_NOTIFY_SYNC,
    ExAcquireFastMutexUnsafe,
    ExFreePool,
    ExInitializeFastMutex,
    ExReleaseFastMutexUnsafe,
    FILE_NOTIFY_INFORMATION,
    IRP,
    IoAcquireCancelSpinLock,
    IoGetCurrentIrpStackLocation,
    IoMarkIrpPending,
    IoReleaseCancelSpinLock,
    IoSetCancelRoutine,
    NOTIFY_CHANGE,
    PCHECK_FOR_TRAVERSE_ACCESS,
    PDEVICE_OBJECT,
    PDRIVER_CANCEL,
    PEPROCESS,
    PFILE_NOTIFY_INFORMATION,
    PFILTER_REPORT_CHANGE,
    PIO_STACK_LOCATION,
    PIRP,
    PNOTIFY_CHANGE,
    PNOTIFY_SYNC,
    PREAL_NOTIFY_SYNC,
    PSECURITY_SUBJECT_CONTEXT,
    PSTR,
    PSTRING,
    PsReturnProcessPagedPoolQuota,
    REAL_NOTIFY_SYNC,
    RtlOemToUnicodeN,
    STRING,
    SeReleaseSubjectContext,
};

extern "system" {
    fn KeGetCurrentThread() -> ntkrnl_sys::ddk::PKTHREAD;
}

const TAG_FS_NOTIFICATIONS: u32 = 0x4E725346;
// fsrtl.h. The port numbered these 1/2/4/8, so NOTIFY_IMMEDIATELY shared bit 0x01
// with WATCH_TREE: every watch-tree notification started out "immediate".
const WATCH_TREE: u32 = 0x01;
const NOTIFY_IMMEDIATELY: u32 = 0x02;
const CLEANUP_IN_PROCESS: u32 = 0x04;
const NOTIFY_LATER: u32 = 0x08;
const WATCH_ROOT: u32 = 0x10;
const DELETE_IN_PROCESS: u32 = 0x20;

const IO_DISK_INCREMENT: i8 = 1;
const FO_CLEANUP_COMPLETE: u32 = 0x4000;
const IRP_BUFFERED_IO: u32 = 0x00000010;
const IRP_DEALLOCATE_BUFFER: u32 = 0x00000020;
const IRP_SYNCHRONOUS_PAGING_IO: u32 = 0x00000040;
const SL_PENDING_RETURNED: u8 = 0x01;
const FILE_ACTION_RENAMED_OLD_NAME: ULONG = 4;
const TAG_NOTIFY_SYNC: u32 = u32::from_le_bytes(*b"FSNS"); // 'SNSF'

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCompleteRequest(irp: PIRP, priority_boost: i8);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCompleteRequest(irp: PIRP, priority_boost: i8);
}

#[inline(always)]
unsafe fn IoCompleteRequest(irp: PIRP, priority_boost: i8) {
    IofCompleteRequest(irp, priority_boost);
}

extern "system" {
    fn RtlxOemStringToUnicodeSize(string: PSTRING) -> u32;
    fn PsChargeProcessPoolQuota(process: PEPROCESS, pool_type: POOL_TYPE, amount: SIZE_T) -> NTSTATUS;
}
extern "C" {
    fn MmGetSystemAddressForMdl(mdl: PVOID) -> PVOID;
}

unsafe fn FsRtlNotifyAcquireFastMutex(real_notify_sync: PREAL_NOTIFY_SYNC) {
    let current_thread = KeGetCurrentThread() as _;

    if (*real_notify_sync).OwningThread != current_thread {
        ExAcquireFastMutexUnsafe(core::ptr::addr_of_mut!((*real_notify_sync).FastMutex));
        (*real_notify_sync).OwningThread = current_thread;
    }
    (*real_notify_sync).OwnerCount += 1;
}

unsafe fn FsRtlNotifyReleaseFastMutex(real_notify_sync: PREAL_NOTIFY_SYNC) {
    (*real_notify_sync).OwnerCount -= 1;
    if (*real_notify_sync).OwnerCount == 0 {
        ExReleaseFastMutexUnsafe(core::ptr::addr_of_mut!((*real_notify_sync).FastMutex));
        (*real_notify_sync).OwningThread = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCancelNotify(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
) {
    let _device_object = opaque_ptr(_device_object);
    let irp = opaque_ptr(irp);
    let mut buffer: PVOID;
    let mut notify_irp: PIRP;
    let mut buffer_length: ULONG;
    let stack: PIO_STACK_LOCATION;
    let notify_change: PNOTIFY_CHANGE;
    let real_notify_sync: PREAL_NOTIFY_SYNC;
    let mut subject_context: PSECURITY_SUBJECT_CONTEXT = core::ptr::null_mut();

    notify_change = (*irp).IoStatus.Information as PNOTIFY_CHANGE;
    (*irp).IoStatus.Information = 0;
    IoSetCancelRoutine(irp, None);
    IoReleaseCancelSpinLock((*irp).CancelIrql);
    real_notify_sync = (*notify_change).NotifySync as PREAL_NOTIFY_SYNC;

    FsRtlNotifyAcquireFastMutex(real_notify_sync);

    RemoveEntryList(core::ptr::addr_of_mut!((*irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY);
    IoMarkIrpPending(irp);

    if !(*notify_change).Buffer.is_null() && (*notify_change).AllocatedBuffer.is_null() &&
       ((!(*irp).MdlAddress.is_null() && MmGetSystemAddressForMdl((*irp).MdlAddress as PVOID) == (*notify_change).Buffer) ||
        (*notify_change).Buffer == (*irp).AssociatedIrp.SystemBuffer)
    {
        buffer = core::ptr::null_mut();
        buffer_length = 0;

        if IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
            if (*notify_change).BufferLength >= (*notify_change).DataLength {
                buffer_length = (*notify_change).BufferLength;
            }
        } else {
            notify_irp = ((core::ptr::addr_of!((*notify_change).NotifyIrps) as *const LIST_ENTRY).offset(0) as *const u8).offset(-(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            stack = IoGetCurrentIrpStackLocation(notify_irp);

            if (*stack).Parameters.NotifyDirectory.Length >= (*notify_change).BufferLength {
                if (*notify_irp).AssociatedIrp.SystemBuffer.is_null() {
                    if !(*notify_irp).MdlAddress.is_null() {
                        buffer = MmGetSystemAddressForMdl((*notify_irp).MdlAddress as PVOID);
                    }
                } else {
                    buffer = (*notify_irp).AssociatedIrp.SystemBuffer;
                }

                buffer_length = (*stack).Parameters.NotifyDirectory.Length;
                if buffer_length > (*notify_change).BufferLength {
                    buffer_length = (*notify_change).BufferLength;
                }
            }
        }

        if buffer_length == 0 {
            (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
        } else {
            if buffer.is_null() {
                // The C's _SEH2_TRY: a failed charge or allocation means notify
                // immediately (see notify_charge_paged_quota).
                if !notify_charge_paged_quota((*notify_change).OwningProcess, buffer_length) {
                    (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
                } else {
                    buffer = ExAllocatePoolWithTag(PagedPool, buffer_length as usize, TAG_FS_NOTIFICATIONS);
                    if buffer.is_null() {
                        PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, buffer_length as _);
                        (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
                    } else {
                        (*notify_change).AllocatedBuffer = buffer;
                        core::ptr::copy_nonoverlapping((*notify_change).Buffer as *const u8, buffer as *mut u8, (*notify_change).DataLength as usize);
                        (*notify_change).ThisBufferLength = buffer_length;
                        (*notify_change).Buffer = buffer;
                    }
                }
            }
        }

        if ((*notify_change).Flags & NOTIFY_IMMEDIATELY as u16) != 0 {
            (*notify_change).Buffer = core::ptr::null_mut();
            (*notify_change).AllocatedBuffer = core::ptr::null_mut();
            (*notify_change).LastEntry = 0;
            (*notify_change).DataLength = 0;
            (*notify_change).ThisBufferLength = 0;
        }
    }

    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(irp, IO_DISK_INCREMENT);

    if InterlockedDecrement(core::ptr::addr_of_mut!((*notify_change).ReferenceCount) as *mut LONG) == 0 {
        if !(*notify_change).AllocatedBuffer.is_null() {
            PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, (*notify_change).ThisBufferLength as _);
            ExFreePoolWithTag((*notify_change).AllocatedBuffer, TAG_FS_NOTIFICATIONS);
        }

        if !(*notify_change).FullDirectoryName.is_null() {
            subject_context = (*notify_change).SubjectContext;
        }

        ExFreePoolWithTag(notify_change as PVOID, 0);
    }

    FsRtlNotifyReleaseFastMutex(real_notify_sync);

    if !subject_context.is_null() {
        SeReleaseSubjectContext(subject_context);
        ExFreePool(subject_context as PVOID);
    }
}

unsafe fn FsRtlCheckNotifyForDelete(notify_list: PLIST_ENTRY, fs_context: PVOID) {
    let mut next_entry: PLIST_ENTRY;
    let mut notify_change: PNOTIFY_CHANGE;

    if !IsListEmpty(notify_list) {
        next_entry = (*notify_list).Flink;
        while next_entry != notify_list {
            notify_change = (next_entry as *const u8).offset(-(core::mem::offset_of!(_NOTIFY_CHANGE, NotifyList) as isize)) as PNOTIFY_CHANGE;
            if (*notify_change).FsContext == fs_context && !IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
                FsRtlNotifyCompleteIrpList(notify_change, STATUS_DELETE_PENDING);
            }
            next_entry = (*next_entry).Flink;
        }
    }
}

unsafe fn FsRtlIsNotifyOnList(notify_list: PLIST_ENTRY, fs_context: PVOID) -> PNOTIFY_CHANGE {
    let mut next_entry: PLIST_ENTRY;
    let mut notify_change: PNOTIFY_CHANGE;

    if !IsListEmpty(notify_list) {
        next_entry = (*notify_list).Flink;
        while next_entry != notify_list {
            notify_change = (next_entry as *const u8).offset(-(core::mem::offset_of!(_NOTIFY_CHANGE, NotifyList) as isize)) as PNOTIFY_CHANGE;
            if (*notify_change).FsContext == fs_context {
                return notify_change;
            }
            next_entry = (*next_entry).Flink;
        }
    }
    core::ptr::null_mut()
}

unsafe fn FsRtlNotifyCompleteIrp(
    irp: PIRP,
    notify_change: PNOTIFY_CHANGE,
    data_length: ULONG,
    status: NTSTATUS,
    skip_completion: BOOLEAN,
) {
    let mut buffer: PVOID;
    let stack: PIO_STACK_LOCATION;

    if FsRtlNotifySetCancelRoutine(irp, notify_change) == 0 && skip_completion != 0 {
        return;
    }

    if status != STATUS_SUCCESS {
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return;
    }

    stack = IoGetCurrentIrpStackLocation(irp);
    if data_length == 0 || (*stack).Parameters.NotifyDirectory.Length < data_length {
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOTIFY_ENUM_DIR;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return;
    }

    if (*notify_change).AllocatedBuffer.is_null() {
        (*irp).IoStatus.Information = data_length as _;
        (*notify_change).Buffer = core::ptr::null_mut();
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return;
    }

    if !(*irp).AssociatedIrp.SystemBuffer.is_null() {
        buffer = (*irp).AssociatedIrp.SystemBuffer;
        core::ptr::copy_nonoverlapping((*notify_change).AllocatedBuffer as *const u8, buffer as *mut u8, data_length as usize);
    } else if !(*irp).MdlAddress.is_null() {
        buffer = MmGetSystemAddressForMdl((*irp).MdlAddress as PVOID);
        core::ptr::copy_nonoverlapping((*notify_change).AllocatedBuffer as *const u8, buffer as *mut u8, data_length as usize);
    } else if ((*stack).Control & SL_PENDING_RETURNED) == 0 {
        buffer = (*irp).UserBuffer;
        core::ptr::copy_nonoverlapping((*notify_change).AllocatedBuffer as *const u8, buffer as *mut u8, data_length as usize);
    } else {
        (*irp).Flags |= IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_SYNCHRONOUS_PAGING_IO;
        (*irp).AssociatedIrp.SystemBuffer = (*notify_change).AllocatedBuffer;
        PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, (*notify_change).ThisBufferLength as _);
        (*notify_change).AllocatedBuffer = core::ptr::null_mut();
        (*notify_change).ThisBufferLength = 0;
        (*irp).IoStatus.Information = data_length as _;
        (*notify_change).Buffer = core::ptr::null_mut();
        IoMarkIrpPending(irp);
        (*irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(irp, IO_DISK_INCREMENT);
        return;
    }

    PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, (*notify_change).ThisBufferLength as _);

    if (*notify_change).AllocatedBuffer != (*irp).AssociatedIrp.SystemBuffer &&
       !(*notify_change).AllocatedBuffer.is_null()
    {
        ExFreePoolWithTag((*notify_change).AllocatedBuffer, 0);
    }

    (*notify_change).AllocatedBuffer = core::ptr::null_mut();
    (*notify_change).ThisBufferLength = 0;
    (*irp).IoStatus.Information = data_length as _;
    (*notify_change).Buffer = core::ptr::null_mut();

    IoMarkIrpPending(irp);
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(irp, IO_DISK_INCREMENT);
}

unsafe fn FsRtlNotifyCompleteIrpList(notify_change: PNOTIFY_CHANGE, status: NTSTATUS) {
    let mut irp: PIRP;
    let data_length: ULONG;
    let mut next_entry: PLIST_ENTRY;

    data_length = (*notify_change).DataLength;

    (*notify_change).Flags &= (NOTIFY_IMMEDIATELY | WATCH_TREE) as u16;
    (*notify_change).DataLength = 0;
    (*notify_change).LastEntry = 0;

    while !IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
        next_entry = RemoveHeadList(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY);
        irp = (next_entry as *const u8).offset(-(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
        FsRtlNotifyCompleteIrp(irp, notify_change, data_length, status, 1);
        if status == STATUS_SUCCESS {
            break;
        }
    }
}

unsafe fn FsRtlNotifySetCancelRoutine(
    irp: PIRP,
    notify_change: PNOTIFY_CHANGE,
) -> BOOLEAN {
    let cancel_routine: PDRIVER_CANCEL;

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*irp).CancelIrql));

    if !notify_change.is_null() {
        cancel_routine = IoSetCancelRoutine(irp, None);
        (*irp).IoStatus.Information = 0;
        IoReleaseCancelSpinLock((*irp).CancelIrql);
        if cancel_routine.is_some() {
            InterlockedDecrement(core::ptr::addr_of_mut!((*notify_change).ReferenceCount) as *mut LONG);
            return 1;
        }
    } else {
        if (*irp).Cancel != 0 {
            FsRtlCancelNotify(core::ptr::null_mut(), irp);
        } else {
            IoSetCancelRoutine(irp, Some(abi_cast!(FsRtlCancelNotify)));
            IoReleaseCancelSpinLock((*irp).CancelIrql);
        }
    }

    0
}

unsafe fn FsRtlNotifyUpdateBuffer(
    output_buffer: PFILE_NOTIFY_INFORMATION,
    action: ULONG,
    parent_name: PSTRING,
    target_name: PSTRING,
    stream_name: PSTRING,
    is_unicode: BOOLEAN,
    data_length: ULONG,
) -> BOOLEAN {
    let mut succeed = 1u8;
    let mut already_written: ULONG = 0;
    let mut result_size: ULONG = 0;

    (*output_buffer).NextEntryOffset = 0;
    (*output_buffer).Action = action;
    (*output_buffer).FileNameLength = data_length - core::mem::offset_of!(_FILE_NOTIFY_INFORMATION, FileName) as u32;
    
    if is_unicode != 0 {
        if (*parent_name).Length != 0 {
            core::ptr::copy_nonoverlapping((*parent_name).Buffer as *const u8, (*output_buffer).FileName.as_mut_ptr() as *mut u8, (*parent_name).Length as usize);
            ((*output_buffer).FileName.as_mut_ptr() as *mut u16).add(((*parent_name).Length / 2) as usize).write(b'\\' as u16);
            already_written = (*parent_name).Length as ULONG + 2;
        }
        core::ptr::copy_nonoverlapping((*target_name).Buffer as *const u8, ((*output_buffer).FileName.as_mut_ptr() as *mut u8).add(already_written as usize), (*target_name).Length as usize);
        if !stream_name.is_null() {
            already_written += (*target_name).Length as ULONG;
            ((*output_buffer).FileName.as_mut_ptr() as *mut u16).add((already_written / 2) as usize).write(b':' as u16);
            core::ptr::copy_nonoverlapping((*stream_name).Buffer as *const u8, ((*output_buffer).FileName.as_mut_ptr() as *mut u8).add(already_written as usize + 2), (*stream_name).Length as usize);
        }
    } else {
        if (*parent_name).Length == 0 {
            core::ptr::copy_nonoverlapping((*stream_name).Buffer as *const u8, (*output_buffer).FileName.as_mut_ptr() as *mut u8, (*stream_name).Length as usize);
        } else {
            RtlOemToUnicodeN((*output_buffer).FileName.as_mut_ptr() as *mut u16, (*output_buffer).FileNameLength, core::ptr::addr_of_mut!(result_size), (*parent_name).Buffer as *const i8, (*parent_name).Length as ULONG);
            ((*output_buffer).FileName.as_mut_ptr() as *mut u16).add((result_size / 2) as usize).write(b'\\' as u16);
            already_written = result_size + 2;

            RtlOemToUnicodeN(((*output_buffer).FileName.as_mut_ptr() as *mut u8).add(already_written as usize) as *mut u16, (*output_buffer).FileNameLength, core::ptr::addr_of_mut!(result_size), (*target_name).Buffer as *const i8, (*target_name).Length as ULONG);

            if !stream_name.is_null() {
                already_written += result_size;
                ((*output_buffer).FileName.as_mut_ptr() as *mut u16).add((already_written / 2) as usize).write(b':' as u16);
                RtlOemToUnicodeN(((*output_buffer).FileName.as_mut_ptr() as *mut u8).add(already_written as usize + 2) as *mut u16, (*output_buffer).FileNameLength, core::ptr::addr_of_mut!(result_size), (*stream_name).Buffer as *const i8, (*stream_name).Length as ULONG);
            }
        }
    }

    succeed
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyChangeDirectory(
    notify_sync: PNOTIFY_SYNC,
    fs_context: PVOID,
    full_directory_name: PSTRING,
    notify_list: PLIST_ENTRY,
    watch_tree: BOOLEAN,
    completion_filter: ULONG,
    notify_irp: PIRP,
) {
    let fs_context = opaque_ptr(fs_context);
    let notify_list = opaque_ptr(notify_list);
    let notify_irp = opaque_ptr(notify_irp);
    FsRtlNotifyFilterChangeDirectory(
        notify_sync,
        notify_list,
        fs_context,
        full_directory_name,
        watch_tree,
        1,
        completion_filter,
        notify_irp,
        None,
        core::ptr::null_mut(),
        None,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyCleanup(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    fs_context: PVOID,
) {
    let notify_list = opaque_ptr(notify_list);
    let fs_context = opaque_ptr(fs_context);
    let notify_change: PNOTIFY_CHANGE;
    let real_notify_sync: PREAL_NOTIFY_SYNC;
    let mut subject_context: PSECURITY_SUBJECT_CONTEXT = core::ptr::null_mut();

    real_notify_sync = notify_sync as PREAL_NOTIFY_SYNC;

    FsRtlNotifyAcquireFastMutex(real_notify_sync);

    notify_change = FsRtlIsNotifyOnList(notify_list, fs_context);
    if !notify_change.is_null() {
        (*notify_change).Flags |= CLEANUP_IN_PROCESS as u16;

        if !IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
            FsRtlNotifyCompleteIrpList(notify_change, STATUS_NOTIFY_CLEANUP);
        }

        if InterlockedDecrement(core::ptr::addr_of_mut!((*notify_change).ReferenceCount) as *mut LONG) == 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*notify_change).NotifyList) as PLIST_ENTRY);

            if !(*notify_change).AllocatedBuffer.is_null() {
                PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, (*notify_change).ThisBufferLength as _);
                ExFreePool((*notify_change).AllocatedBuffer);
            }

            if !(*notify_change).FullDirectoryName.is_null() {
                subject_context = (*notify_change).SubjectContext;
            }

            ExFreePoolWithTag(notify_change as PVOID, TAG_FS_NOTIFICATIONS);
        }
    }

    FsRtlNotifyReleaseFastMutex(real_notify_sync);

    if !subject_context.is_null() {
        SeReleaseSubjectContext(subject_context);
        ExFreePool(subject_context as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyFilterChangeDirectory(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    fs_context: PVOID,
    full_directory_name: PSTRING,
    watch_tree: BOOLEAN,
    ignore_buffer: BOOLEAN,
    completion_filter: ULONG,
    notify_irp: PIRP,
    traverse_callback: PCHECK_FOR_TRAVERSE_ACCESS,
    subject_context: PSECURITY_SUBJECT_CONTEXT,
    filter_callback: PFILTER_REPORT_CHANGE,
) {
    let notify_list = opaque_ptr(notify_list);
    let fs_context = opaque_ptr(fs_context);
    let notify_irp = opaque_ptr(notify_irp);
    let subject_context = opaque_ptr(subject_context);
    let mut notify_change: PNOTIFY_CHANGE = core::ptr::null_mut();
    let real_notify_sync: PREAL_NOTIFY_SYNC = notify_sync as PREAL_NOTIFY_SYNC;

    FsRtlNotifyAcquireFastMutex(real_notify_sync);

    // The C's _SEH2_TRY body; `break 'body` is its _SEH2_LEAVE, the end of the
    // block its _SEH2_FINALLY.
    'body: {
        // No IRP: the FSD is cleaning up, so complete for delete.
        if notify_irp.is_null() {
            FsRtlCheckNotifyForDelete(notify_list, fs_context);
            break 'body;
        }

        (*notify_irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*notify_irp).IoStatus.Information = 0;

        let stack = IoGetCurrentIrpStackLocation(notify_irp);
        if ((*(*stack).FileObject).Flags & FO_CLEANUP_COMPLETE) != 0 {
            IoMarkIrpPending(notify_irp);
            (*notify_irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOTIFY_CLEANUP;
            IoCompleteRequest(notify_irp, IO_DISK_INCREMENT);
            break 'body;
        }

        notify_change = FsRtlIsNotifyOnList(notify_list, fs_context);
        if !notify_change.is_null() {
            if ((*notify_change).Flags & CLEANUP_IN_PROCESS as u16) != 0 {
                IoMarkIrpPending(notify_irp);
                (*notify_irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOTIFY_CLEANUP;
                IoCompleteRequest(notify_irp, IO_DISK_INCREMENT);
            } else if ((*notify_change).Flags & DELETE_IN_PROCESS as u16) != 0 {
                IoMarkIrpPending(notify_irp);
                (*notify_irp).IoStatus.__bindgen_anon_1.Status = STATUS_DELETE_PENDING;
                IoCompleteRequest(notify_irp, IO_DISK_INCREMENT);
            }
            if ((*notify_change).Flags & NOTIFY_IMMEDIATELY as u16) != 0
                && ((*notify_change).Flags & NOTIFY_LATER as u16) == 0
            {
                (*notify_change).Flags &= !(NOTIFY_IMMEDIATELY as u16);
                IoMarkIrpPending(notify_irp);
                (*notify_irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOTIFY_ENUM_DIR;
                IoCompleteRequest(notify_irp, IO_DISK_INCREMENT);
                break 'body;
            } else if (*notify_change).DataLength == 0
                || ((*notify_change).Flags & NOTIFY_LATER as u16) != 0
            {
                // C: goto HandleIRP. The port left this a comment and returned, so a
                // second watch on a handle with nothing reported yet left its IRP
                // pending on no list and with no cancel routine.
            } else {
                let saved_length = (*notify_change).DataLength;
                (*notify_change).DataLength = 0;
                FsRtlNotifyCompleteIrp(notify_irp, notify_change, saved_length, STATUS_SUCCESS, 0);
                break 'body;
            }
        } else {
            notify_change = ExAllocatePoolWithTag(
                PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
                core::mem::size_of::<_NOTIFY_CHANGE>(),
                TAG_FS_NOTIFICATIONS,
            ) as PNOTIFY_CHANGE;
            core::ptr::write_bytes(notify_change as *mut u8, 0, core::mem::size_of::<_NOTIFY_CHANGE>());

            (*notify_change).NotifySync = notify_sync as PREAL_NOTIFY_SYNC;
            (*notify_change).FsContext = fs_context;
            (*notify_change).StreamID = (*(*stack).FileObject).FsContext;
            (*notify_change).TraverseCallback = traverse_callback;
            (*notify_change).SubjectContext = subject_context;
            (*notify_change).FullDirectoryName = full_directory_name;
            (*notify_change).FilterCallback = filter_callback;
            InitializeListHead(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY);

            if watch_tree != 0 {
                (*notify_change).Flags |= WATCH_TREE as u16;
            }

            // An empty name counts as ANSI; one that cannot hold a WCHAR, or whose
            // second byte is set, is ANSI too.
            if (*full_directory_name).Length == 0 {
                (*notify_change).CharacterSize = 1;
            } else {
                if (*full_directory_name).Length < 2 || *((*full_directory_name).Buffer as *const i8).add(1) != 0 {
                    (*notify_change).CharacterSize = 1;
                } else {
                    (*notify_change).CharacterSize = 2;
                }
                if (*full_directory_name).Length == (*notify_change).CharacterSize as u16 {
                    (*notify_change).Flags |= WATCH_ROOT as u16;
                }
            }

            (*notify_change).CompletionFilter = completion_filter;
            if ignore_buffer == 0 {
                (*notify_change).BufferLength = (*stack).Parameters.NotifyDirectory.Length;
            }
            (*notify_change).OwningProcess = (*(*notify_irp).Tail.Overlay.Thread).ThreadsProcess;
            InsertTailList(notify_list, core::ptr::addr_of_mut!((*notify_change).NotifyList) as PLIST_ENTRY);
            (*notify_change).ReferenceCount = 1;
        }

        // HandleIRP:
        (*notify_irp).IoStatus.Information = notify_change as _;
        IoMarkIrpPending(notify_irp);
        InsertTailList(
            core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*notify_irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY,
        );
        InterlockedIncrement(core::ptr::addr_of_mut!((*notify_change).ReferenceCount) as *mut LONG);
        FsRtlNotifySetCancelRoutine(notify_irp, core::ptr::null_mut());
    }

    FsRtlNotifyReleaseFastMutex(real_notify_sync);

    // A captured subject context is ours to free unless a new notification kept it.
    if !subject_context.is_null() && (notify_change.is_null() || !(*notify_change).FullDirectoryName.is_null()) {
        SeReleaseSubjectContext(subject_context);
        ExFreePool(subject_context as PVOID);
    }
}

/// C: `RtlOemStringToCountedUnicodeSize`, a macro over `NLS_MB_OEM_CODE_PAGE_TAG`.
unsafe fn oem_string_to_counted_unicode_size(string: PSTRING) -> ULONG {
    let size = if NlsMbOemCodePageTag != 0 {
        RtlxOemStringToUnicodeSize(string)
    } else {
        ((*string).Length as ULONG + 1) * 2
    };
    size - 2
}

/// The C charges with `PsChargePoolQuota`, which raises, inside an `_SEH2_TRY`
/// whose handler treats a failure as "notify immediately". This kernel has no
/// SEH, so take the status instead; the system process is exempt as there.
unsafe fn notify_charge_paged_quota(process: PEPROCESS, amount: ULONG) -> bool {
    process as PVOID == PsInitialSystemProcess as PVOID
        || PsChargeProcessPoolQuota(process, PagedPool, amount as _) >= 0
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyFilterReportChange(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    full_target_name: PSTRING,
    target_name_offset: USHORT,
    stream_name: PSTRING,
    normalized_parent_name: PSTRING,
    filter_match: ULONG,
    action: ULONG,
    target_context: PVOID,
    filter_context: PVOID,
) {
    let notify_list = opaque_ptr(notify_list);
    let target_context = opaque_ptr(target_context);
    let filter_context = opaque_ptr(filter_context);
    let empty = STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut target_directory = empty;
    let mut target_name = empty;
    let mut parent_name = empty;
    let mut int_normalized_parent_name = empty;
    let mut normalized_parent_name = normalized_parent_name;
    let mut is_stream = false;

    // A full name needs the offset of its last part.
    if target_name_offset == 0 && !full_target_name.is_null() {
        return;
    }

    let real_notify_sync = notify_sync as PREAL_NOTIFY_SYNC;
    FsRtlNotifyAcquireFastMutex(real_notify_sync);

    let mut next_entry = (*notify_list).Flink;
    while next_entry != notify_list {
        let notify_change = (next_entry as *mut u8)
            .sub(core::mem::offset_of!(_NOTIFY_CHANGE, NotifyList)) as PNOTIFY_CHANGE;
        next_entry = (*next_entry).Flink;
        let flags = move |f: u32| (*notify_change).Flags & f as u16 != 0;
        let char_size = (*notify_change).CharacterSize as u32;
        let is_parent: bool;

        if !full_target_name.is_null() {
            let full_dir = (*notify_change).FullDirectoryName;
            ntkrnl_sys::ASSERT!(!full_dir.is_null());
            if (*full_dir).Length == 0 || filter_match & (*notify_change).CompletionFilter == 0 {
                continue;
            }

            // Without a normalized parent name, take the directory part of the target.
            if normalized_parent_name.is_null() {
                int_normalized_parent_name.Buffer = (*full_target_name).Buffer;
                let len = if target_name_offset as u32 != char_size {
                    target_name_offset - char_size as u16
                } else {
                    target_name_offset
                };
                int_normalized_parent_name.Length = len;
                int_normalized_parent_name.MaximumLength = len;
                normalized_parent_name = &mut int_normalized_parent_name;
            }

            // A watched directory longer than the changed file's parent cannot match.
            if (*normalized_parent_name).Length < (*full_dir).Length {
                continue;
            }
            if (*normalized_parent_name).Length == (*full_dir).Length {
                is_parent = true;
            } else if !flags(WATCH_TREE) {
                continue;
            } else {
                // Deeper changes must sit under a `\` right after the watched name.
                if !flags(WATCH_ROOT) {
                    let sep = if char_size == 1 {
                        *((*normalized_parent_name).Buffer as *const u8).add((*full_dir).Length as usize) as u16
                    } else {
                        *((*normalized_parent_name).Buffer as *const u16).add((*full_dir).Length as usize / 2)
                    };
                    if sep != b'\\' as u16 {
                        continue;
                    }
                }
                is_parent = false;
            }

            // C: RtlEqualMemory, a macro.
            let len = (*full_dir).Length as usize;
            if ntkrnl_sys::ddk::RtlCompareMemory(
                (*normalized_parent_name).Buffer as PVOID,
                (*full_dir).Buffer as PVOID,
                len as _,
            ) != len as _
            {
                continue;
            }

            if !is_parent {
                if let Some(traverse) = (*notify_change).TraverseCallback {
                    if traverse((*notify_change).FsContext, target_context, (*notify_change).SubjectContext) == 0 {
                        continue;
                    }
                }
            }
            if let Some(filter) = (*notify_change).FilterCallback {
                if !filter_context.is_null() && filter((*notify_change).FsContext, filter_context) == 0 {
                    continue;
                }
            }
        } else {
            // A stream notification: matched by context, no directory name.
            ntkrnl_sys::ASSERT!((*notify_change).FullDirectoryName.is_null());
            if target_context != (*notify_change).SubjectContext as PVOID {
                continue;
            }
            parent_name.Buffer = core::ptr::null_mut();
            parent_name.Length = 0;
            is_stream = true;
            is_parent = false;
        }

        if !flags(NOTIFY_IMMEDIATELY) && (*notify_change).BufferLength != 0 {
            // How much room the output has: this buffer, an IRP's, or the default.
            let mut irp: PIRP = core::ptr::null_mut();
            let number_of_bytes: ULONG = if (*notify_change).ThisBufferLength == 0 {
                if IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
                    (*notify_change).BufferLength
                } else {
                    irp = ((*notify_change).NotifyIrps.Flink as *mut u8)
                        .sub(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
                    (*IoGetCurrentIrpStackLocation(irp)).Parameters.NotifyDirectory.Length
                }
            } else {
                (*notify_change).ThisBufferLength
            };

            if is_parent {
                // The watched directory itself changed: no parent part to report.
                parent_name.Length = 0;
            } else if !is_stream {
                let full_dir = (*notify_change).FullDirectoryName;
                let parent_name_offset: ULONG;
                let mut parent_name_length: ULONG;
                if flags(WATCH_ROOT) || (*normalized_parent_name).Buffer != (*full_target_name).Buffer {
                    if target_directory.Buffer.is_null() {
                        target_directory.Buffer = (*full_target_name).Buffer;
                        target_directory.Length = target_name_offset;
                        if target_name_offset as u32 != char_size {
                            target_directory.Length = target_name_offset - char_size as u16;
                        }
                        target_directory.MaximumLength = target_directory.Length;
                    }
                    // Find where the target directory passes the watched name's last
                    // part, by counting separators (FsRtlNotifyGetLastPartOffset).
                    // The C leaves LastPartOffset unset when watching the root;
                    // there the parent starts right after the leading `\`, offset 0.
                    let mut last_part_offset: ULONG = 0;
                    if !flags(WATCH_ROOT) {
                        let mut full_parts: ULONG = 1;
                        let mut target_parts: ULONG = 0;
                        let (full_len, targ_len) = if char_size == 1 {
                            ((*full_dir).Length as ULONG, target_directory.Length as ULONG)
                        } else {
                            ((*full_dir).Length as ULONG / 2, target_directory.Length as ULONG / 2)
                        };
                        let at = |buf: PCHAR, i: ULONG| -> u16 {
                            if char_size == 1 {
                                *(buf as *const u8).add(i as usize) as u16
                            } else {
                                *(buf as *const u16).add(i as usize)
                            }
                        };
                        for pos in 0..full_len {
                            if at((*full_dir).Buffer, pos) == b'\\' as u16 {
                                full_parts += 1;
                            }
                        }
                        while last_part_offset < targ_len {
                            if at(target_directory.Buffer, last_part_offset) == b'\\' as u16 {
                                target_parts += 1;
                                if target_parts == full_parts {
                                    break;
                                }
                            }
                            last_part_offset += 1;
                        }
                        last_part_offset *= char_size;
                    }
                    parent_name_offset = char_size + last_part_offset;
                    parent_name.Buffer = target_directory.Buffer.add(parent_name_offset as usize);
                    parent_name_length = target_directory.Length as ULONG;
                } else {
                    parent_name.Buffer = (*normalized_parent_name).Buffer.add((*full_dir).Length as usize + char_size as usize);
                    parent_name_length = ((*normalized_parent_name).Length - (*full_dir).Length) as ULONG;
                    parent_name_offset = char_size;
                }
                parent_name_length -= parent_name_offset;
                parent_name.Length = parent_name_length as USHORT;
                parent_name.MaximumLength = parent_name_length as USHORT;
            }

            // Size of the record: header, then parent `\` target `:` stream, in WCHARs.
            let mut data_length = core::mem::offset_of!(_FILE_NOTIFY_INFORMATION, FileName) as ULONG;
            if is_stream {
                ntkrnl_sys::ASSERT!(!stream_name.is_null());
                data_length += (*stream_name).Length as ULONG;
            } else {
                if !is_parent {
                    data_length += if char_size == 1 {
                        oem_string_to_counted_unicode_size(&mut parent_name)
                    } else {
                        parent_name.Length as ULONG
                    };
                    data_length += 2;
                }
                if target_name.Buffer.is_null() {
                    target_name.Buffer = (*full_target_name).Buffer.add(target_name_offset as usize);
                    target_name.Length = (*full_target_name).Length - target_name_offset;
                    target_name.MaximumLength = target_name.Length;
                }
                data_length += if char_size == 1 {
                    oem_string_to_counted_unicode_size(&mut target_name)
                } else {
                    target_name.Length as ULONG
                };
                if !stream_name.is_null() {
                    // The C sizes an OEM stream by TargetName plus a CHAR, but
                    // FsRtlNotifyUpdateBuffer writes the converted stream after a
                    // WCHAR ':'; size what is written.
                    data_length += if char_size == 2 {
                        (*stream_name).Length as ULONG
                    } else {
                        oem_string_to_counted_unicode_size(stream_name)
                    } + 2;
                }
            }

            // Records are ULONG-aligned; one that does not fit means re-enumerate.
            let aligned_data_length = ((*notify_change).DataLength + 3) & !3;
            if data_length > number_of_bytes || aligned_data_length + data_length > number_of_bytes {
                (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
            } else {
                let mut output_buffer: PVOID = core::ptr::null_mut();
                if !(*notify_change).Buffer.is_null() {
                    // Chain the previous record to the one about to be written.
                    let last = ((*notify_change).Buffer as *mut u8).add((*notify_change).LastEntry as usize)
                        as PFILE_NOTIFY_INFORMATION;
                    (*last).NextEntryOffset = aligned_data_length - (*notify_change).LastEntry;
                    (*notify_change).LastEntry = aligned_data_length;
                    output_buffer = ((*notify_change).Buffer as *mut u8).add(aligned_data_length as usize) as PVOID;
                } else if !irp.is_null() {
                    if !(*irp).AssociatedIrp.SystemBuffer.is_null() {
                        output_buffer = (*irp).AssociatedIrp.SystemBuffer;
                    } else if !(*irp).MdlAddress.is_null() {
                        output_buffer = MmGetSystemAddressForMdl((*irp).MdlAddress as PVOID);
                    }
                    (*notify_change).Buffer = output_buffer;
                    (*notify_change).ThisBufferLength = number_of_bytes;
                }

                if (*notify_change).Buffer.is_null() {
                    (*notify_change).ThisBufferLength = number_of_bytes;
                    if notify_charge_paged_quota((*notify_change).OwningProcess, number_of_bytes) {
                        output_buffer = ExAllocatePoolWithTag(PagedPool, number_of_bytes as usize, TAG_FS_NOTIFICATIONS);
                        if output_buffer.is_null() {
                            PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, number_of_bytes as _);
                            (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
                        }
                        (*notify_change).Buffer = output_buffer;
                        (*notify_change).AllocatedBuffer = output_buffer;
                    } else {
                        (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
                    }
                }

                if !output_buffer.is_null() {
                    if FsRtlNotifyUpdateBuffer(
                        output_buffer as PFILE_NOTIFY_INFORMATION,
                        action,
                        &mut parent_name,
                        &mut target_name,
                        stream_name,
                        (char_size == 2) as BOOLEAN,
                        data_length,
                    ) != 0
                    {
                        (*notify_change).DataLength = data_length + aligned_data_length;
                    } else {
                        (*notify_change).Flags |= NOTIFY_IMMEDIATELY as u16;
                    }
                }
            }

            // Notifying immediately means re-enumerate: drop whatever was buffered.
            if flags(NOTIFY_IMMEDIATELY) && !(*notify_change).Buffer.is_null() {
                if !(*notify_change).AllocatedBuffer.is_null() {
                    PsReturnProcessPagedPoolQuota((*notify_change).OwningProcess, (*notify_change).ThisBufferLength as _);
                    ExFreePoolWithTag((*notify_change).AllocatedBuffer, TAG_FS_NOTIFICATIONS);
                }
                (*notify_change).Buffer = core::ptr::null_mut();
                (*notify_change).AllocatedBuffer = core::ptr::null_mut();
                (*notify_change).LastEntry = 0;
                (*notify_change).DataLength = 0;
                (*notify_change).ThisBufferLength = 0;
            }
        }

        // An old name waits for its new name before anything completes.
        if action == FILE_ACTION_RENAMED_OLD_NAME {
            (*notify_change).Flags |= NOTIFY_LATER as u16;
        } else {
            (*notify_change).Flags &= !(NOTIFY_LATER as u16);
            if !IsListEmpty(core::ptr::addr_of_mut!((*notify_change).NotifyIrps) as PLIST_ENTRY) {
                FsRtlNotifyCompleteIrpList(notify_change, STATUS_SUCCESS);
            }
        }
    }

    FsRtlNotifyReleaseFastMutex(real_notify_sync);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyFullChangeDirectory(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    fs_context: PVOID,
    full_directory_name: PSTRING,
    watch_tree: BOOLEAN,
    ignore_buffer: BOOLEAN,
    completion_filter: ULONG,
    notify_irp: PIRP,
    traverse_callback: PCHECK_FOR_TRAVERSE_ACCESS,
    subject_context: PSECURITY_SUBJECT_CONTEXT,
) {
    let notify_list = opaque_ptr(notify_list);
    let fs_context = opaque_ptr(fs_context);
    let notify_irp = opaque_ptr(notify_irp);
    let subject_context = opaque_ptr(subject_context);
    FsRtlNotifyFilterChangeDirectory(
        notify_sync,
        notify_list,
        fs_context,
        full_directory_name,
        watch_tree,
        ignore_buffer,
        completion_filter,
        notify_irp,
        traverse_callback,
        subject_context,
        None,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyFullReportChange(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    full_target_name: PSTRING,
    target_name_offset: USHORT,
    stream_name: PSTRING,
    normalized_parent_name: PSTRING,
    filter_match: ULONG,
    action: ULONG,
    target_context: PVOID,
) {
    let notify_list = opaque_ptr(notify_list);
    let target_context = opaque_ptr(target_context);
    FsRtlNotifyFilterReportChange(
        notify_sync,
        notify_list,
        full_target_name,
        target_name_offset,
        stream_name,
        normalized_parent_name,
        filter_match,
        action,
        target_context,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyInitializeSync(notify_sync: *mut PNOTIFY_SYNC) {
    let notify_sync = opaque_ptr(notify_sync);
    let real_notify_sync: PREAL_NOTIFY_SYNC;

    *notify_sync = core::ptr::null_mut();

    real_notify_sync = ExAllocatePoolWithTag(NonPagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE, core::mem::size_of::<_REAL_NOTIFY_SYNC>(), TAG_NOTIFY_SYNC) as PREAL_NOTIFY_SYNC;
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*real_notify_sync).FastMutex));
    (*real_notify_sync).OwningThread = 0;
    (*real_notify_sync).OwnerCount = 0;

    *notify_sync = real_notify_sync as PNOTIFY_SYNC;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyReportChange(
    notify_sync: PNOTIFY_SYNC,
    notify_list: PLIST_ENTRY,
    full_target_name: PSTRING,
    file_name_part_length: *mut USHORT,
    filter_match: ULONG,
) {
    let notify_list = opaque_ptr(notify_list);
    let file_name_part_length = opaque_ptr(file_name_part_length);
    FsRtlNotifyFilterReportChange(
        notify_sync,
        notify_list,
        full_target_name,
        (*full_target_name).Length - *file_name_part_length,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        filter_match,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyUninitializeSync(notify_sync: *mut PNOTIFY_SYNC) {
    let notify_sync = opaque_ptr(notify_sync);
    if !(*notify_sync).is_null() {
        ExFreePoolWithTag(*notify_sync, TAG_NOTIFY_SYNC);
        *notify_sync = core::ptr::null_mut();
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/notify.c
//  * PURPOSE:         Change Notifications and Sync for File System Drivers
//  * PROGRAMMERS:     Pierre Schweitzer
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* INLINED FUNCTIONS *********************************************************/
//
// /*
//  * @implemented
//  */
// FORCEINLINE
// VOID
// FsRtlNotifyAcquireFastMutex(IN PREAL_NOTIFY_SYNC RealNotifySync)
// {
//     ULONG_PTR CurrentThread = (ULONG_PTR)KeGetCurrentThread();
//
//     /* Only acquire fast mutex if it's not already acquired by the current thread */
//     if (RealNotifySync->OwningThread != CurrentThread)
//     {
//         ExAcquireFastMutexUnsafe(&(RealNotifySync->FastMutex));
//         RealNotifySync->OwningThread = CurrentThread;
//     }
//     /* Whatever the case, keep trace of the attempt to acquire fast mutex */
//     RealNotifySync->OwnerCount++;
// }
//
// /*
//  * @implemented
//  */
// FORCEINLINE
// VOID
// FsRtlNotifyReleaseFastMutex(IN PREAL_NOTIFY_SYNC RealNotifySync)
// {
//     RealNotifySync->OwnerCount--;
//     /* Release the fast mutex only if no other instance needs it */
//     if (!RealNotifySync->OwnerCount)
//     {
//         ExReleaseFastMutexUnsafe(&(RealNotifySync->FastMutex));
//         RealNotifySync->OwningThread = (ULONG_PTR)0;
//     }
// }
//
// #define FsRtlNotifyGetLastPartOffset(FullLen, TargLen, Type, Chr)                 \
//     for (FullPosition = 0; FullPosition < FullLen; ++FullPosition)                \
//         if (((Type)NotifyChange->FullDirectoryName->Buffer)[FullPosition] == Chr) \
//             ++FullNumberOfParts;                                                  \
//     for (LastPartOffset = 0; LastPartOffset < TargLen; ++LastPartOffset) {        \
//         if ( ((Type)TargetDirectory.Buffer)[LastPartOffset] == Chr) {             \
//             ++TargetNumberOfParts;                                                \
//         if (TargetNumberOfParts == FullNumberOfParts)                             \
//             break;                                                                \
//         }                                                                         \
//     }
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// FsRtlNotifyCompleteIrpList(IN PNOTIFY_CHANGE NotifyChange,
//                            IN NTSTATUS Status);
//
// BOOLEAN
// FsRtlNotifySetCancelRoutine(IN PIRP Irp,
//                             IN PNOTIFY_CHANGE NotifyChange OPTIONAL);
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlCancelNotify(IN PDEVICE_OBJECT DeviceObject,
//                   IN PIRP Irp)
// {
//     PVOID Buffer;
//     PIRP NotifyIrp;
//     ULONG BufferLength;
//     PIO_STACK_LOCATION Stack;
//     PNOTIFY_CHANGE NotifyChange;
//     PREAL_NOTIFY_SYNC RealNotifySync;
//     BOOLEAN PoolQuotaCharged;
//     PSECURITY_SUBJECT_CONTEXT _SEH2_VOLATILE SubjectContext = NULL;
//
//     /* Get the NOTIFY_CHANGE struct and reset it */
//     NotifyChange = (PNOTIFY_CHANGE)Irp->IoStatus.Information;
//     Irp->IoStatus.Information = 0;
//     /* Reset the cancel routine */
//     IoSetCancelRoutine(Irp, NULL);
//     /* And release lock */
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//     /* Get REAL_NOTIFY_SYNC struct */
//     RealNotifySync = NotifyChange->NotifySync;
//
//     FsRtlNotifyAcquireFastMutex(RealNotifySync);
//
//    _SEH2_TRY
//    {
//        /* Remove the IRP from the notifications list and mark it pending */
//        RemoveEntryList(&(Irp->Tail.Overlay.ListEntry));
//        IoMarkIrpPending(Irp);
//
//        /* Now, the tricky part - let's find a buffer big enough to hold the return data */
//        if (NotifyChange->Buffer && NotifyChange->AllocatedBuffer == NULL &&
//            ((Irp->MdlAddress && MmGetSystemAddressForMdl(Irp->MdlAddress) == NotifyChange->Buffer) ||
//            NotifyChange->Buffer == Irp->AssociatedIrp.SystemBuffer))
//        {
//            /* Assume we didn't find any */
//            Buffer = NULL;
//            BufferLength = 0;
//
//            /* If we don't have IRPs, check if current buffer is big enough */
//            if (IsListEmpty(&NotifyChange->NotifyIrps))
//            {
//                if (NotifyChange->BufferLength >= NotifyChange->DataLength)
//                {
//                    BufferLength = NotifyChange->BufferLength;
//                }
//            }
//            else
//            {
//                /* Otherwise, try to look at next IRP available */
//                NotifyIrp = CONTAINING_RECORD(NotifyChange->NotifyIrps.Flink, IRP, Tail.Overlay.ListEntry);
//                Stack = IoGetCurrentIrpStackLocation(NotifyIrp);
//
//                /* If its buffer is big enough, get it */
//                if (Stack->Parameters.NotifyDirectory.Length >= NotifyChange->BufferLength)
//                {
//                    /* Is it MDL? */
//                    if (NotifyIrp->AssociatedIrp.SystemBuffer == NULL)
//                    {
//                        if (NotifyIrp->MdlAddress != NULL)
//                        {
//                            Buffer = MmGetSystemAddressForMdl(NotifyIrp->MdlAddress);
//                        }
//                    }
//                    else
//                    {
//                        Buffer = NotifyIrp->AssociatedIrp.MasterIrp;
//                    }
//
//                    /* Backup our accepted buffer length */
//                    BufferLength = Stack->Parameters.NotifyDirectory.Length;
//                    if (BufferLength > NotifyChange->BufferLength)
//                    {
//                        BufferLength = NotifyChange->BufferLength;
//                    }
//                }
//            }
//
//            /* At that point, we *may* have a buffer */
//
//            /* If it has null length, then note that we won't use it */
//            if (BufferLength == 0)
//            {
//                NotifyChange->Flags |= NOTIFY_IMMEDIATELY;
//            }
//            else
//            {
//                /* If we have a buffer length, but no buffer then allocate one */
//                if (Buffer == NULL)
//                {
//                    /* Assume we haven't charged quotas */
//                    PoolQuotaCharged = FALSE;
//
//                    _SEH2_TRY
//                    {
//                        /* Charge quotas */
//                        PsChargePoolQuota(NotifyChange->OwningProcess, PagedPool, BufferLength as _);
//                        PoolQuotaCharged = TRUE;
//
//                        /* Allocate buffer */
//                        Buffer = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE, BufferLength, TAG_FS_NOTIFICATIONS);
//                        NotifyChange->AllocatedBuffer = Buffer;
//
//                        /* Copy data in that buffer */
//                        RtlCopyMemory(Buffer, NotifyChange->Buffer, NotifyChange->DataLength);
//                        NotifyChange->ThisBufferLength = BufferLength;
//                        NotifyChange->Buffer = Buffer;
//                    }
//                    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                    {
//                        /* Something went wrong, have we charged quotas? */
//                        if (PoolQuotaCharged)
//                        {
//                            /* We did, return quotas */
//                            PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, BufferLength as _);
//                        }
//
//                        /* And notify immediately */
//                        NotifyChange->Flags |= NOTIFY_IMMEDIATELY;
//                    }
//                    _SEH2_END;
//                }
//            }
//
//            /* If we have to notify immediately, ensure that any buffer is 0-ed out */
//            if (NotifyChange->Flags & NOTIFY_IMMEDIATELY)
//            {
//                NotifyChange->Buffer = 0;
//                NotifyChange->AllocatedBuffer = 0;
//                NotifyChange->LastEntry = 0;
//                NotifyChange->DataLength = 0;
//                NotifyChange->ThisBufferLength = 0;
//            }
//        }
//
//        /* It's now time to complete - data are ready */
//
//        /* Set appropriate status and complete */
//        Irp->IoStatus.Status = STATUS_CANCELLED;
//        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
//
//        /* If that notification isn't referenced any longer, drop it */
//        if (!InterlockedDecrement((PLONG)&(NotifyChange->ReferenceCount)))
//        {
//            /* If it had an allocated buffer, delete */
//            if (NotifyChange->AllocatedBuffer)
//            {
//                PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, NotifyChange->ThisBufferLength as _);
//                ExFreePoolWithTag(NotifyChange->AllocatedBuffer, TAG_FS_NOTIFICATIONS);
//            }
//
//            /* In case of full name, remember subject context for later deletion */
//            if (NotifyChange->FullDirectoryName)
//            {
//                SubjectContext = NotifyChange->SubjectContext;
//            }
//
//            /* We mustn't have ANY change left anymore */
//            ASSERT(NotifyChange->NotifyList.Flink == NULL);
//            ExFreePoolWithTag(NotifyChange, 0);
//        }
//     }
//     _SEH2_FINALLY
//     {
//         FsRtlNotifyReleaseFastMutex(RealNotifySync);
//
//         /* If the subject security context was captured, release and free it */
//         if (SubjectContext)
//         {
//             SeReleaseSubjectContext(SubjectContext);
//             ExFreePool(SubjectContext);
//         }
//     }
//     _SEH2_END;
// }
//
// /*
//  * @implemented
//  */
// VOID
// FsRtlCheckNotifyForDelete(IN PLIST_ENTRY NotifyList,
//                           IN PVOID FsContext)
// {
//     PLIST_ENTRY NextEntry;
//     PNOTIFY_CHANGE NotifyChange;
//
//     if (!IsListEmpty(NotifyList))
//     {
//         /* Browse the notifications list to find the matching entry */
//         for (NextEntry = NotifyList->Flink;
//              NextEntry != NotifyList;
//              NextEntry = NextEntry->Flink)
//         {
//             NotifyChange = CONTAINING_RECORD(NextEntry, NOTIFY_CHANGE, NotifyList);
//             /* If the current record matches with the given context, it's the good one */
//             if (NotifyChange->FsContext == FsContext && !IsListEmpty(&(NotifyChange->NotifyIrps)))
//             {
//                 FsRtlNotifyCompleteIrpList(NotifyChange, STATUS_DELETE_PENDING);
//             }
//         }
//     }
// }
//
// /*
//  *@implemented
//  */
// PNOTIFY_CHANGE
// FsRtlIsNotifyOnList(IN PLIST_ENTRY NotifyList,
//                     IN PVOID FsContext)
// {
//     PLIST_ENTRY NextEntry;
//     PNOTIFY_CHANGE NotifyChange;
//
//     if (!IsListEmpty(NotifyList))
//     {
//         /* Browse the notifications list to find the matching entry */
//         for (NextEntry = NotifyList->Flink;
//              NextEntry != NotifyList;
//              NextEntry = NextEntry->Flink)
//         {
//             NotifyChange = CONTAINING_RECORD(NextEntry, NOTIFY_CHANGE, NotifyList);
//             /* If the current record matches with the given context, it's the good one */
//             if (NotifyChange->FsContext == FsContext)
//             {
//                 return NotifyChange;
//             }
//         }
//     }
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// VOID
// FsRtlNotifyCompleteIrp(IN PIRP Irp,
//                        IN PNOTIFY_CHANGE NotifyChange,
//                        IN ULONG DataLength,
//                        IN NTSTATUS Status,
//                        IN BOOLEAN SkipCompletion)
// {
//     PVOID Buffer;
//     PIO_STACK_LOCATION Stack;
//
//     PAGED_CODE();
//
//     /* Check if we need to complete */
//     if (!FsRtlNotifySetCancelRoutine(Irp, NotifyChange) && SkipCompletion)
//     {
//         return;
//     }
//
//     /* No succes => no data to return just complete */
//     if (Status != STATUS_SUCCESS)
//     {
//         goto Completion;
//     }
//
//     /* Ensure there's something to return */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     if (!DataLength || Stack->Parameters.NotifyDirectory.Length < DataLength)
//     {
//         Status = STATUS_NOTIFY_ENUM_DIR;
//         goto Completion;
//     }
//
//     /* Ensture there's a buffer where to find data */
//     if (!NotifyChange->AllocatedBuffer)
//     {
//         Irp->IoStatus.Information = DataLength;
//         NotifyChange->Buffer = NULL;
//         goto Completion;
//     }
//
//     /* Now, browse all the way to return data
//      * and find the one that will work. We will
//      * return data whatever happens
//      */
//     if (Irp->AssociatedIrp.SystemBuffer)
//     {
//         Buffer = Irp->AssociatedIrp.SystemBuffer;
//         goto CopyAndComplete;
//     }
//
//     if (Irp->MdlAddress)
//     {
//         Buffer = MmGetSystemAddressForMdl(Irp->MdlAddress);
//         goto CopyAndComplete;
//     }
//
//     if (!(Stack->Control & SL_PENDING_RETURNED))
//     {
//         Buffer = Irp->UserBuffer;
//         goto CopyAndComplete;
//     }
//
//     Irp->Flags |= (IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_SYNCHRONOUS_PAGING_IO);
//     Irp->AssociatedIrp.SystemBuffer = NotifyChange->AllocatedBuffer;
//     /* Nothing to copy */
//     goto ReleaseAndComplete;
//
// CopyAndComplete:
//     _SEH2_TRY
//     {
//         RtlCopyMemory(Buffer, NotifyChange->AllocatedBuffer, DataLength);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Do nothing */
//     }
//     _SEH2_END;
//
// ReleaseAndComplete:
//     PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, NotifyChange->ThisBufferLength as _);
//
//     /* Release buffer UNLESS it's used */
//     if (NotifyChange->AllocatedBuffer != Irp->AssociatedIrp.SystemBuffer &&
//         NotifyChange->AllocatedBuffer)
//     {
//         ExFreePoolWithTag(NotifyChange->AllocatedBuffer, 0);
//     }
//
//     /* Prepare for return */
//     NotifyChange->AllocatedBuffer = 0;
//     NotifyChange->ThisBufferLength = 0;
//     Irp->IoStatus.Information = DataLength;
//     NotifyChange->Buffer = NULL;
//
//     /* Finally complete */
// Completion:
//     IoMarkIrpPending(Irp);
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_DISK_INCREMENT);
// }
//
// /*
//  * @implemented
//  */
// VOID
// FsRtlNotifyCompleteIrpList(IN PNOTIFY_CHANGE NotifyChange,
//                            IN NTSTATUS Status)
// {
//     PIRP Irp;
//     ULONG DataLength;
//     PLIST_ENTRY NextEntry;
//
//     DataLength = NotifyChange->DataLength;
//
//     NotifyChange->Flags &= (NOTIFY_IMMEDIATELY | WATCH_TREE);
//     NotifyChange->DataLength = 0;
//     NotifyChange->LastEntry = 0;
//
//     while (!IsListEmpty(&(NotifyChange->NotifyIrps)))
//     {
//         /* We take the first entry */
//         NextEntry = RemoveHeadList(&(NotifyChange->NotifyIrps));
//         Irp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//         /* We complete it */
//         FsRtlNotifyCompleteIrp(Irp, NotifyChange, DataLength, Status, TRUE);
//         /* If we're notifying success, just notify first one */
//         if (Status == STATUS_SUCCESS)
//             break;
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// FsRtlNotifySetCancelRoutine(IN PIRP Irp,
//                             IN PNOTIFY_CHANGE NotifyChange OPTIONAL)
// {
//     PDRIVER_CANCEL CancelRoutine;
//
//     /* Acquire cancel lock */
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//
//     /* If NotifyChange was given */
//     if (NotifyChange)
//     {
//         /* First get cancel routine */
//         CancelRoutine = IoSetCancelRoutine(Irp, NULL);
//         Irp->IoStatus.Information = 0;
//         /* Release cancel lock */
//         IoReleaseCancelSpinLock(Irp->CancelIrql);
//         /* If there was a cancel routine */
//         if (CancelRoutine)
//         {
//             /* Decrease reference count */
//             InterlockedDecrement((PLONG)&NotifyChange->ReferenceCount);
//             /* Notify that we removed cancel routine */
//             return TRUE;
//         }
//     }
//     else
//     {
//         /* If IRP is cancel, call FsRtl cancel routine */
//         if (Irp->Cancel)
//         {
//              FsRtlCancelNotify(NULL, Irp);
//         }
//         else
//         {
//             /* Otherwise, define FsRtl cancel routine as IRP cancel routine */
//             IoSetCancelRoutine(Irp, FsRtlCancelNotify);
//             /* Release lock */
//             IoReleaseCancelSpinLock(Irp->CancelIrql);
//         }
//     }
//
//     /* Return that we didn't removed cancel routine */
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// FsRtlNotifyUpdateBuffer(OUT PFILE_NOTIFY_INFORMATION OutputBuffer,
//                         IN ULONG Action,
//                         IN PSTRING ParentName,
//                         IN PSTRING TargetName,
//                         IN PSTRING StreamName,
//                         IN BOOLEAN IsUnicode,
//                         IN ULONG DataLength)
// {
//     /* Unless there's an issue with buffers, there's no reason to fail */
//     BOOLEAN Succeed = TRUE;
//     ULONG AlreadyWritten = 0, ResultSize;
//
//     PAGED_CODE();
//
//     /* Update user buffer with the change that occured
//      * First copy parent name if any
//      * Then copy target name, there's always one
//      * And finally, copy stream name if any
//      * If these names aren't unicode, then convert first
//      */
//     _SEH2_TRY
//     {
//         OutputBuffer->NextEntryOffset = 0;
//         OutputBuffer->Action = Action;
//         OutputBuffer->FileNameLength = DataLength - FIELD_OFFSET(FILE_NOTIFY_INFORMATION, FileName);
//         if (IsUnicode)
//         {
//             if (ParentName->Length)
//             {
//                 RtlCopyMemory(OutputBuffer->FileName, ParentName->Buffer, ParentName->Length);
//                 OutputBuffer->FileName[ParentName->Length / sizeof(WCHAR)] = L'\\';
//                 AlreadyWritten = ParentName->Length + sizeof(WCHAR);
//             }
//             RtlCopyMemory((PVOID)((ULONG_PTR)OutputBuffer->FileName + AlreadyWritten),
//                           TargetName->Buffer, TargetName->Length);
//             if (StreamName)
//             {
//                 AlreadyWritten += TargetName->Length;
//                 OutputBuffer->FileName[AlreadyWritten / sizeof(WCHAR)] = L':';
//                 RtlCopyMemory((PVOID)((ULONG_PTR)OutputBuffer->FileName + AlreadyWritten + sizeof(WCHAR)),
//                               StreamName->Buffer, StreamName->Length);
//             }
//         }
//         else
//         {
//             if (!ParentName->Length)
//             {
//                 ASSERT(StreamName);
//                 RtlCopyMemory(OutputBuffer->FileName, StreamName->Buffer, StreamName->Length);
//             }
//             else
//             {
//                 RtlOemToUnicodeN(OutputBuffer->FileName, OutputBuffer->FileNameLength,
//                                  &ResultSize, ParentName->Buffer,
//                                  ParentName->Length);
//                 OutputBuffer->FileName[ResultSize / sizeof(WCHAR)] = L'\\';
//                 AlreadyWritten = ResultSize + sizeof(WCHAR);
//
//                 RtlOemToUnicodeN((PVOID)((ULONG_PTR)OutputBuffer->FileName + AlreadyWritten),
//                                  OutputBuffer->FileNameLength, &ResultSize,
//                                  TargetName->Buffer, TargetName->Length);
//
//                 if (StreamName)
//                 {
//                     AlreadyWritten += ResultSize;
//                     OutputBuffer->FileName[AlreadyWritten / sizeof(WCHAR)] = L':';
//                     RtlOemToUnicodeN((PVOID)((ULONG_PTR)OutputBuffer->FileName + AlreadyWritten + sizeof(WCHAR)),
//                                      OutputBuffer->FileNameLength, &ResultSize,
//                                      StreamName->Buffer, StreamName->Length);
//                 }
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Succeed = FALSE;
//     }
//     _SEH2_END;
//
//     return Succeed;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlNotifyChangeDirectory
//  * @implemented
//  *
//  * Lets FSD know if changes occures in the specified directory.
//  * Directory will be reenumerated.
//  *
//  * @param NotifySync
//  *        Synchronization object pointer
//  *
//  * @param FsContext
//  *        Used to identify the notify structure
//  *
//  * @param FullDirectoryName
//  *        String (A or W) containing the full directory name
//  *
//  * @param NotifyList
//  *        Notify list pointer (to head)
//  *
//  * @param WatchTree
//  *        True to notify changes in subdirectories too
//  *
//  * @param CompletionFilter
//  *        Used to define types of changes to notify
//  *
//  * @param NotifyIrp
//  *        IRP pointer to complete notify operation. It can be null
//  *
//  * @return None
//  *
//  * @remarks This function only redirects to FsRtlNotifyFilterChangeDirectory.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyChangeDirectory(IN PNOTIFY_SYNC NotifySync,
//                            IN PVOID FsContext,
//                            IN PSTRING FullDirectoryName,
//                            IN PLIST_ENTRY NotifyList,
//                            IN BOOLEAN WatchTree,
//                            IN ULONG CompletionFilter,
//                            IN PIRP NotifyIrp)
// {
//     FsRtlNotifyFilterChangeDirectory(NotifySync,
//                                      NotifyList,
//                                      FsContext,
//                                      FullDirectoryName,
//                                      WatchTree,
//                                      TRUE,
//                                      CompletionFilter,
//                                      NotifyIrp,
//                                      NULL,
//                                      NULL,
//                                      NULL);
// }
//
// /*++
//  * @name FsRtlNotifyCleanup
//  * @implemented
//  *
//  * Called by FSD when all handles to FileObject (identified by FsContext) are closed
//  *
//  * @param NotifySync
//  *        Synchronization object pointer
//  *
//  * @param NotifyList
//  *        Notify list pointer (to head)
//  *
//  * @param FsContext
//  *        Used to identify the notify structure
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyCleanup(IN PNOTIFY_SYNC NotifySync,
//                    IN PLIST_ENTRY NotifyList,
//                    IN PVOID FsContext)
// {
//     PNOTIFY_CHANGE NotifyChange;
//     PREAL_NOTIFY_SYNC RealNotifySync;
//     PSECURITY_SUBJECT_CONTEXT _SEH2_VOLATILE SubjectContext = NULL;
//
//     /* Get real structure hidden behind the opaque pointer */
//     RealNotifySync = (PREAL_NOTIFY_SYNC)NotifySync;
//
//     /* Acquire the fast mutex */
//     FsRtlNotifyAcquireFastMutex(RealNotifySync);
//
//     _SEH2_TRY
//     {
//         /* Find if there's a matching notification with the FsContext */
//         NotifyChange = FsRtlIsNotifyOnList(NotifyList, FsContext);
//         if (NotifyChange)
//         {
//             /* Mark it as to know that cleanup is in process */
//             NotifyChange->Flags |= CLEANUP_IN_PROCESS;
//
//             /* If there are pending IRPs, complete them using the STATUS_NOTIFY_CLEANUP status */
//             if (!IsListEmpty(&NotifyChange->NotifyIrps))
//             {
//                 FsRtlNotifyCompleteIrpList(NotifyChange, STATUS_NOTIFY_CLEANUP);
//             }
//
//             /* Decrease reference number and if 0 is reached, it's time to do complete cleanup */
//             if (!InterlockedDecrement((PLONG)&(NotifyChange->ReferenceCount)))
//             {
//                 /* Remove it from the notifications list */
//                 RemoveEntryList(&NotifyChange->NotifyList);
//
//                 /* In case there was an allocated buffer, free it */
//                 if (NotifyChange->AllocatedBuffer)
//                 {
//                     PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, NotifyChange->ThisBufferLength as _);
//                     ExFreePool(NotifyChange->AllocatedBuffer);
//                 }
//
//                 /* In case there the string was set, get the captured subject security context */
//                 if (NotifyChange->FullDirectoryName)
//                 {
//                     SubjectContext = NotifyChange->SubjectContext;
//                 }
//
//                 /* Finally, free the notification, as it's not needed anymore */
//                 ExFreePoolWithTag(NotifyChange, 'NrSF');
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         /* Release fast mutex */
//         FsRtlNotifyReleaseFastMutex(RealNotifySync);
//
//         /* If the subject security context was captured, release and free it */
//         if (SubjectContext)
//         {
//             SeReleaseSubjectContext(SubjectContext);
//             ExFreePool(SubjectContext);
//         }
//     }
//     _SEH2_END;
// }
//
// /*++
//  * @name FsRtlNotifyFilterChangeDirectory
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param NotifySync
//  *        FILLME
//  *
//  * @param NotifyList
//  *        FILLME
//  *
//  * @param FsContext
//  *        FILLME
//  *
//  * @param FullDirectoryName
//  *        FILLME
//  *
//  * @param WatchTree
//  *        FILLME
//  *
//  * @param IgnoreBuffer
//  *        FILLME
//  *
//  * @param CompletionFilter
//  *        FILLME
//  *
//  * @param NotifyIrp
//  *        FILLME
//  *
//  * @param TraverseCallback
//  *        FILLME
//  *
//  * @param SubjectContext
//  *        FILLME
//  *
//  * @param FilterCallback
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyFilterChangeDirectory(IN PNOTIFY_SYNC NotifySync,
//                                  IN PLIST_ENTRY NotifyList,
//                                  IN PVOID FsContext,
//                                  IN PSTRING FullDirectoryName,
//                                  IN BOOLEAN WatchTree,
//                                  IN BOOLEAN IgnoreBuffer,
//                                  IN ULONG CompletionFilter,
//                                  IN PIRP NotifyIrp,
//                                  IN PCHECK_FOR_TRAVERSE_ACCESS TraverseCallback OPTIONAL,
//                                  IN PSECURITY_SUBJECT_CONTEXT SubjectContext OPTIONAL,
//                                  IN PFILTER_REPORT_CHANGE FilterCallback OPTIONAL)
// {
//     ULONG SavedLength;
//     PIO_STACK_LOCATION Stack;
//     PNOTIFY_CHANGE NotifyChange = NULL;
//     PREAL_NOTIFY_SYNC RealNotifySync;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlNotifyFilterChangeDirectory(): %p, %p, %p, %wZ, %u, %u, %u, %p, %p, %p, %p\n",
//     NotifySync, NotifyList, FsContext, FullDirectoryName, WatchTree, IgnoreBuffer, CompletionFilter, NotifyIrp,
//     TraverseCallback, SubjectContext, FilterCallback);
//
//     /* Get real structure hidden behind the opaque pointer */
//     RealNotifySync = (PREAL_NOTIFY_SYNC)NotifySync;
//
//     /* Acquire the fast mutex */
//     FsRtlNotifyAcquireFastMutex(RealNotifySync);
//
//     _SEH2_TRY
//     {
//         /* If we have no IRP, FSD is performing a cleanup */
//         if (!NotifyIrp)
//         {
//             /* So, we delete */
//             FsRtlCheckNotifyForDelete(NotifyList, FsContext);
//             _SEH2_LEAVE;
//         }
//
//         NotifyIrp->IoStatus.Status = STATUS_SUCCESS;
//         NotifyIrp->IoStatus.Information = (ULONG_PTR)NULL;
//
//         Stack = IoGetCurrentIrpStackLocation(NotifyIrp);
//         /* If FileObject's been cleaned up, just return */
//         if (Stack->FileObject->Flags & FO_CLEANUP_COMPLETE)
//         {
//             IoMarkIrpPending(NotifyIrp);
//             NotifyIrp->IoStatus.Status = STATUS_NOTIFY_CLEANUP;
//             IoCompleteRequest(NotifyIrp, IO_DISK_INCREMENT);
//             _SEH2_LEAVE;
//         }
//
//         /* Try to find a matching notification has been already registered */
//         NotifyChange = FsRtlIsNotifyOnList(NotifyList, FsContext);
//         if (NotifyChange)
//         {
//             /* If it's been found, and is cleaned up, immediatly complete */
//             if (NotifyChange->Flags & CLEANUP_IN_PROCESS)
//             {
//                 IoMarkIrpPending(NotifyIrp);
//                 NotifyIrp->IoStatus.Status = STATUS_NOTIFY_CLEANUP;
//                 IoCompleteRequest(NotifyIrp, IO_DISK_INCREMENT);
//             }
//             /* Or if it's about to be deleted, complete */
//             else if (NotifyChange->Flags & DELETE_IN_PROCESS)
//             {
//                 IoMarkIrpPending(NotifyIrp);
//                 NotifyIrp->IoStatus.Status = STATUS_DELETE_PENDING;
//                 IoCompleteRequest(NotifyIrp, IO_DISK_INCREMENT);
//             }
//             /* Complete now if asked to (and not asked to notify later on) */
//             if ((NotifyChange->Flags & NOTIFY_IMMEDIATELY) && !(NotifyChange->Flags & NOTIFY_LATER))
//             {
//                 NotifyChange->Flags &= ~NOTIFY_IMMEDIATELY;
//                 IoMarkIrpPending(NotifyIrp);
//                 NotifyIrp->IoStatus.Status = STATUS_NOTIFY_ENUM_DIR;
//                 IoCompleteRequest(NotifyIrp, IO_DISK_INCREMENT);
//             }
//             /* If no data yet, or asked to notify later on, handle */
//             else if (NotifyChange->DataLength == 0 || (NotifyChange->Flags & NOTIFY_LATER))
//             {
//                 goto HandleIRP;
//             }
//             /* Else, just complete with we have */
//             else
//             {
//                 SavedLength = NotifyChange->DataLength;
//                 NotifyChange->DataLength = 0;
//                 FsRtlNotifyCompleteIrp(NotifyIrp, NotifyChange, SavedLength, STATUS_SUCCESS, FALSE);
//             }
//
//             _SEH2_LEAVE;
//         }
//
//         /* Allocate new notification */
//         NotifyChange = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                              sizeof(NOTIFY_CHANGE), 'NrSF');
//         RtlZeroMemory(NotifyChange, sizeof(NOTIFY_CHANGE));
//
//         /* Set basic information */
//         NotifyChange->NotifySync = NotifySync;
//         NotifyChange->FsContext = FsContext;
//         NotifyChange->StreamID = Stack->FileObject->FsContext;
//         NotifyChange->TraverseCallback = TraverseCallback;
//         NotifyChange->SubjectContext = SubjectContext;
//         NotifyChange->FullDirectoryName = FullDirectoryName;
//         NotifyChange->FilterCallback = FilterCallback;
//         InitializeListHead(&(NotifyChange->NotifyIrps));
//
//         /* Keep trace of WatchTree */
//         if (WatchTree)
//         {
//             NotifyChange->Flags |= WATCH_TREE;
//         }
//
//         /* If string is empty, faulty to ANSI */
//         if (FullDirectoryName->Length == 0)
//         {
//             NotifyChange->CharacterSize = sizeof(CHAR);
//         }
//         else
//         {
//             /* If it can't contain WCHAR, it's ANSI */
//             if (FullDirectoryName->Length < sizeof(WCHAR) || ((CHAR*)FullDirectoryName->Buffer)[1] != 0)
//             {
//                 NotifyChange->CharacterSize = sizeof(CHAR);
//             }
//             else
//             {
//                 NotifyChange->CharacterSize = sizeof(WCHAR);
//             }
//
//             /* Now, check is user is willing to watch root */
//             if (FullDirectoryName->Length == NotifyChange->CharacterSize)
//             {
//                 NotifyChange->Flags |= WATCH_ROOT;
//             }
//         }
//
//         NotifyChange->CompletionFilter = CompletionFilter;
//
//         /* In case we have not to ignore buffer , keep its length */
//         if (!IgnoreBuffer)
//         {
//             NotifyChange->BufferLength = Stack->Parameters.NotifyDirectory.Length;
//         }
//
//         NotifyChange->OwningProcess = NotifyIrp->Tail.Overlay.Thread->ThreadsProcess;
//
//         /* Insert the notification into the notification list */
//         InsertTailList(NotifyList, &(NotifyChange->NotifyList));
//
//         NotifyChange->ReferenceCount = 1;
//
// HandleIRP:
//         /* Associate the notification to the IRP */
//         NotifyIrp->IoStatus.Information = (ULONG_PTR)NotifyChange;
//         /* The IRP is pending */
//         IoMarkIrpPending(NotifyIrp);
//         /* Insert the IRP in the IRP list */
//         InsertTailList(&(NotifyChange->NotifyIrps), &(NotifyIrp->Tail.Overlay.ListEntry));
//         /* Increment reference count */
//         InterlockedIncrement((PLONG)&(NotifyChange->ReferenceCount));
//         /* Set cancel routine to FsRtl one */
//         FsRtlNotifySetCancelRoutine(NotifyIrp, NULL);
//     }
//     _SEH2_FINALLY
//     {
//         /* Release fast mutex */
//         FsRtlNotifyReleaseFastMutex(RealNotifySync);
//
//         /* If the subject security context was captured and there's no notify */
//         if (SubjectContext && (!NotifyChange || NotifyChange->FullDirectoryName))
//         {
//             SeReleaseSubjectContext(SubjectContext);
//             ExFreePool(SubjectContext);
//         }
//     }
//     _SEH2_END;
// }
//
// /*++
//  * @name FsRtlNotifyFilterReportChange
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param NotifySync
//  *        FILLME
//  *
//  * @param NotifyList
//  *        FILLME
//  *
//  * @param FullTargetName
//  *        FILLME
//  *
//  * @param TargetNameOffset
//  *        FILLME
//  *
//  * @param StreamName
//  *        FILLME
//  *
//  * @param NormalizedParentName
//  *        FILLME
//  *
//  * @param FilterMatch
//  *        FILLME
//  *
//  * @param Action
//  *        FILLME
//  *
//  * @param TargetContext
//  *        FILLME
//  *
//  * @param FilterContext
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyFilterReportChange(IN PNOTIFY_SYNC NotifySync,
//                               IN PLIST_ENTRY NotifyList,
//                               IN PSTRING FullTargetName,
//                               IN USHORT TargetNameOffset,
//                               IN PSTRING StreamName OPTIONAL,
//                               IN PSTRING NormalizedParentName OPTIONAL,
//                               IN ULONG FilterMatch,
//                               IN ULONG Action,
//                               IN PVOID TargetContext,
//                               IN PVOID FilterContext)
// {
//     PIRP Irp;
//     PVOID OutputBuffer;
//     USHORT FullPosition;
//     PLIST_ENTRY NextEntry;
//     PIO_STACK_LOCATION Stack;
//     PNOTIFY_CHANGE NotifyChange;
//     PREAL_NOTIFY_SYNC RealNotifySync;
//     PFILE_NOTIFY_INFORMATION FileNotifyInfo;
//     BOOLEAN IsStream, IsParent, PoolQuotaCharged;
//     STRING TargetDirectory, TargetName, ParentName, IntNormalizedParentName;
//     ULONG NumberOfBytes, TargetNumberOfParts, FullNumberOfParts, LastPartOffset, ParentNameOffset, ParentNameLength;
//     ULONG DataLength, AlignedDataLength;
//
//     TargetDirectory.Length = 0;
//     TargetDirectory.MaximumLength = 0;
//     TargetDirectory.Buffer = NULL;
//     TargetName.Length = 0;
//     TargetName.MaximumLength = 0;
//     TargetName.Buffer = NULL;
//     ParentName.Length = 0;
//     ParentName.MaximumLength = 0;
//     ParentName.Buffer = NULL;
//     IsStream = FALSE;
//
//     PAGED_CODE();
//
//     DPRINT("FsRtlNotifyFilterReportChange(%p, %p, %p, %u, %p, %p, %p, %x, %x, %p, %p)\n",
//            NotifySync, NotifyList, FullTargetName, TargetNameOffset, StreamName, NormalizedParentName,
//            FilterMatch, Action, TargetContext, FilterContext);
//
//     /* We need offset in name */
//     if (!TargetNameOffset && FullTargetName)
//     {
//         return;
//     }
//
//     /* Get real structure hidden behind the opaque pointer */
//     RealNotifySync = (PREAL_NOTIFY_SYNC)NotifySync;
//     /* Acquire lock - will be released in finally block */
//     FsRtlNotifyAcquireFastMutex(RealNotifySync);
//     _SEH2_TRY
//     {
//         /* Browse all the registered notifications we have */
//         for (NextEntry = NotifyList->Flink; NextEntry != NotifyList;
//              NextEntry = NextEntry->Flink)
//         {
//             /* Try to find an entry matching our change */
//             NotifyChange = CONTAINING_RECORD(NextEntry, NOTIFY_CHANGE, NotifyList);
//             if (FullTargetName != NULL)
//             {
//                 ASSERT(NotifyChange->FullDirectoryName != NULL);
//                 if (!NotifyChange->FullDirectoryName->Length)
//                 {
//                     continue;
//                 }
//
//                 if (!(FilterMatch & NotifyChange->CompletionFilter))
//                 {
//                     continue;
//                 }
//
//                 /* If no normalized name provided, construct it from full target name */
//                 if (NormalizedParentName == NULL)
//                 {
//                     IntNormalizedParentName.Buffer = FullTargetName->Buffer;
//                     if (TargetNameOffset != NotifyChange->CharacterSize)
//                     {
//                         IntNormalizedParentName.MaximumLength =
//                         IntNormalizedParentName.Length = TargetNameOffset - NotifyChange->CharacterSize;
//                     }
//                     else
//                     {
//                         IntNormalizedParentName.MaximumLength =
//                         IntNormalizedParentName.Length = TargetNameOffset;
//                     }
//                     NormalizedParentName = &IntNormalizedParentName;
//                 }
//
//                 /* heh? Watched directory bigger than changed file? */
//                 if (NormalizedParentName->Length < NotifyChange->FullDirectoryName->Length)
//                 {
//                     continue;
//                 }
//
//                 /* Same len => parent */
//                 if (NormalizedParentName->Length == NotifyChange->FullDirectoryName->Length)
//                 {
//                     IsParent = TRUE;
//                 }
//                 /* If not, then, we have to be watching the tree, otherwise we don't have to report such changes */
//                 else if (!(NotifyChange->Flags & WATCH_TREE))
//                 {
//                     continue;
//                 }
//                 /* And finally, we've to check we're properly \-terminated */
//                 else
//                 {
//                     if (!(NotifyChange->Flags & WATCH_ROOT))
//                     {
//                         if (NotifyChange->CharacterSize == sizeof(CHAR))
//                         {
//                             if (((PSTR)NormalizedParentName->Buffer)[NotifyChange->FullDirectoryName->Length] != '\\')
//                             {
//                                 continue;
//                             }
//                         }
//                         else
//                         {
//                             if (((PWSTR)NormalizedParentName->Buffer)[NotifyChange->FullDirectoryName->Length / sizeof (WCHAR)] != L'\\')
//                             {
//                                 continue;
//                             }
//                         }
//                     }
//
//                     IsParent = FALSE;
//                 }
//
//                 /* If len matches, then check that both name are equal */
//                 if (!RtlEqualMemory(NormalizedParentName->Buffer, NotifyChange->FullDirectoryName->Buffer,
//                                     NotifyChange->FullDirectoryName->Length))
//                 {
//                     continue;
//                 }
//
//                 /* Call traverse callback (only if we have to traverse ;-)) */
//                 if (!IsParent
//                     && NotifyChange->TraverseCallback != NULL
//                     && !NotifyChange->TraverseCallback(NotifyChange->FsContext,
//                                                        TargetContext,
//                                                        NotifyChange->SubjectContext))
//                 {
//                     continue;
//                 }
//
//                 /* And then, filter callback if provided */
//                 if (NotifyChange->FilterCallback != NULL
//                     && FilterContext != NULL
//                     && !NotifyChange->FilterCallback(NotifyChange->FsContext, FilterContext))
//                 {
//                     continue;
//                 }
//             }
//             /* We have a stream! */
//             else
//             {
//                 ASSERT(NotifyChange->FullDirectoryName == NULL);
//                 if (TargetContext != NotifyChange->SubjectContext)
//                 {
//                     continue;
//                 }
//
//                 ParentName.Buffer = NULL;
//                 ParentName.Length = 0;
//                 IsStream = TRUE;
//                 IsParent = FALSE;
//             }
//
//             /* If we don't have to notify immediately, prepare for output */
//             if (!(NotifyChange->Flags & NOTIFY_IMMEDIATELY))
//             {
//                 /* If we have something to output... */
//                 if (NotifyChange->BufferLength)
//                 {
//                     /* Get size of the output */
//                     NumberOfBytes = 0;
//                     Irp = NULL;
//                     if (!NotifyChange->ThisBufferLength)
//                     {
//                         if (IsListEmpty(&NotifyChange->NotifyIrps))
//                         {
//                             NumberOfBytes = NotifyChange->BufferLength;
//                         }
//                         else
//                         {
//                             Irp = CONTAINING_RECORD(NotifyChange->NotifyIrps.Flink, IRP, Tail.Overlay.ListEntry);
//                             Stack = IoGetCurrentIrpStackLocation(Irp);
//                             NumberOfBytes = Stack->Parameters.NotifyDirectory.Length;
//                         }
//                     }
//                     else
//                     {
//                         NumberOfBytes = NotifyChange->ThisBufferLength;
//                     }
//
//                     /* If we're matching parent, we don't care about parent (redundant) */
//                     if (IsParent)
//                     {
//                         ParentName.Length = 0;
//                     }
//                     else
//                     {
//                         /* If we don't deal with streams, some more work is required */
//                         if (!IsStream)
//                         {
//                             if (NotifyChange->Flags & WATCH_ROOT ||
//                                 (NormalizedParentName->Buffer != FullTargetName->Buffer))
//                             {
//                                 /* Construct TargetDirectory if we don't have it yet */
//                                 if (TargetDirectory.Buffer == NULL)
//                                 {
//                                     TargetDirectory.Buffer = FullTargetName->Buffer;
//                                     TargetDirectory.Length = TargetNameOffset;
//                                     if (TargetNameOffset != NotifyChange->CharacterSize)
//                                     {
//                                         TargetDirectory.Length = TargetNameOffset - NotifyChange->CharacterSize;
//                                     }
//                                     TargetDirectory.MaximumLength = TargetDirectory.Length;
//                                 }
//                                 /* Now, we start looking for matching parts (unless we watch root) */
//                                 TargetNumberOfParts = 0;
//                                 if (!(NotifyChange->Flags & WATCH_ROOT))
//                                 {
//                                     FullNumberOfParts = 1;
//                                     if (NotifyChange->CharacterSize == sizeof(CHAR))
//                                     {
//                                         FsRtlNotifyGetLastPartOffset(NotifyChange->FullDirectoryName->Length,
//                                                                      TargetDirectory.Length, PSTR, '\\');
//                                     }
//                                     else
//                                     {
//                                         FsRtlNotifyGetLastPartOffset(NotifyChange->FullDirectoryName->Length / sizeof(WCHAR),
//                                                                      TargetDirectory.Length / sizeof(WCHAR), PWSTR, L'\\');
//                                         LastPartOffset *= NotifyChange->CharacterSize;
//                                     }
//                                 }
//
//                                 /* Then, we can construct proper parent name */
//                                 ParentNameOffset = NotifyChange->CharacterSize + LastPartOffset;
//                                 ParentName.Buffer = &TargetDirectory.Buffer[ParentNameOffset];
//                                 ParentNameLength = TargetDirectory.Length;
//                             }
//                             else
//                             {
//                                 /* Construct parent name even for streams */
//                                 ParentName.Buffer = &NormalizedParentName->Buffer[NotifyChange->FullDirectoryName->Length] + NotifyChange->CharacterSize;
//                                 ParentNameLength = NormalizedParentName->Length - NotifyChange->FullDirectoryName->Length;
//                                 ParentNameOffset = NotifyChange->CharacterSize;
//                             }
//                             ParentNameLength -= ParentNameOffset;
//                             ParentName.Length = ParentNameLength;
//                             ParentName.MaximumLength = ParentNameLength;
//                         }
//                     }
//
//                     /* Start to count amount of data to write, we've first the structure itself */
//                     DataLength = FIELD_OFFSET(FILE_NOTIFY_INFORMATION, FileName);
//
//                     /* If stream, we'll just append stream name */
//                     if (IsStream)
//                     {
//                         ASSERT(StreamName != NULL);
//                         DataLength += StreamName->Length;
//                     }
//                     else
//                     {
//                         /* If not parent, we've to append parent name */
//                         if (!IsParent)
//                         {
//                             if (NotifyChange->CharacterSize == sizeof(CHAR))
//                             {
//                                 DataLength += RtlOemStringToCountedUnicodeSize(&ParentName);
//                             }
//                             else
//                             {
//                                 DataLength += ParentName.Length;
//                             }
//                             DataLength += sizeof(WCHAR);
//                         }
//
//                         /* Look for target name & construct it, if required */
//                         if (TargetName.Buffer == NULL)
//                         {
//                             TargetName.Buffer = &FullTargetName->Buffer[TargetNameOffset];
//                             TargetName.Length =
//                             TargetName.MaximumLength = FullTargetName->Length - TargetNameOffset;
//                         }
//
//                         /* Then, we will append it as well */
//                         if (NotifyChange->CharacterSize == sizeof(CHAR))
//                         {
//                             DataLength += RtlOemStringToCountedUnicodeSize(&TargetName);
//                         }
//                         else
//                         {
//                             DataLength += TargetName.Length;
//                         }
//
//                         /* If we also had a stream name, then we can append it as well */
//                         if (StreamName != NULL)
//                         {
//                             if (NotifyChange->CharacterSize == sizeof(WCHAR))
//                             {
//                                 DataLength += StreamName->Length + sizeof(WCHAR);
//                             }
//                             else
//                             {
//                                 DataLength = DataLength + RtlOemStringToCountedUnicodeSize(&TargetName) + sizeof(CHAR);
//                             }
//                         }
//                     }
//
//                     /* Get the position where we can put our data (aligned!) */
//                     AlignedDataLength = ROUND_UP(NotifyChange->DataLength, sizeof(ULONG));
//                     /* If it's higher than buffer length, then, bail out without outputing */
//                     if (DataLength > NumberOfBytes || AlignedDataLength + DataLength > NumberOfBytes)
//                     {
//                         NotifyChange->Flags |= NOTIFY_IMMEDIATELY;
//                     }
//                     else
//                     {
//                         OutputBuffer = NULL;
//                         FileNotifyInfo = NULL;
//                         /* If we already had a buffer, update last entry position */
//                         if (NotifyChange->Buffer != NULL)
//                         {
//                             FileNotifyInfo = (PVOID)((ULONG_PTR)NotifyChange->Buffer + NotifyChange->LastEntry);
//                             FileNotifyInfo->NextEntryOffset = AlignedDataLength - NotifyChange->LastEntry;
//                             NotifyChange->LastEntry = AlignedDataLength;
//                             /* And get our output buffer */
//                             OutputBuffer = (PVOID)((ULONG_PTR)NotifyChange->Buffer + AlignedDataLength);
//                         }
//                         /* If we hadn't buffer, try to find one */
//                         else if (Irp != NULL)
//                         {
//                             if (Irp->AssociatedIrp.SystemBuffer != NULL)
//                             {
//                                 OutputBuffer = Irp->AssociatedIrp.SystemBuffer;
//                             }
//                             else if (Irp->MdlAddress != NULL)
//                             {
//                                 OutputBuffer = MmGetSystemAddressForMdl(Irp->MdlAddress);
//                             }
//
//                             NotifyChange->Buffer = OutputBuffer;
//                             NotifyChange->ThisBufferLength = NumberOfBytes;
//                         }
//
//                         /* If we couldn't find one, then allocate one */
//                         if (NotifyChange->Buffer == NULL)
//                         {
//                             /* Assign the length buffer */
//                             NotifyChange->ThisBufferLength = NumberOfBytes;
//
//                             /* Assume we have not charged quotas */
//                             PoolQuotaCharged = FALSE;
//                             _SEH2_TRY
//                             {
//                                 /* And charge quotas */
//                                 PsChargePoolQuota(NotifyChange->OwningProcess, PagedPool, NotifyChange->ThisBufferLength as _);
//                                 PoolQuotaCharged = TRUE;
//                                 OutputBuffer = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                                                      NumberOfBytes, TAG_FS_NOTIFICATIONS);
//                                 NotifyChange->Buffer = OutputBuffer;
//                                 NotifyChange->AllocatedBuffer = OutputBuffer;
//                             }
//                             /* If something went wrong during allocation, notify immediately instead of outputing */
//                             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                             {
//                                 if (PoolQuotaCharged)
//                                 {
//                                     PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, NotifyChange->ThisBufferLength as _);
//                                 }
//                                 NotifyChange->Flags |= NOTIFY_IMMEDIATELY;
//                             }
//                             _SEH2_END;
//                         }
//
//                         /* Finally, if we have a buffer, fill it in! */
//                         if (OutputBuffer != NULL)
//                         {
//                             if (FsRtlNotifyUpdateBuffer((FILE_NOTIFY_INFORMATION *)OutputBuffer,
//                                                          Action, &ParentName, &TargetName,
//                                                          StreamName, NotifyChange->CharacterSize == sizeof(WCHAR),
//                                                          DataLength))
//                             {
//                                 NotifyChange->DataLength = DataLength + AlignedDataLength;
//                             }
//                             /* If it failed, notify immediately */
//                             else
//                             {
//                                 NotifyChange->Flags |= NOTIFY_IMMEDIATELY;
//                             }
//                         }
//                     }
//
//                     /* If we have to notify right now (something went wrong?) */
//                     if (NotifyChange->Flags & NOTIFY_IMMEDIATELY)
//                     {
//                         /* Ensure that all our buffers are NULL */
//                         if (NotifyChange->Buffer != NULL)
//                         {
//                             if (NotifyChange->AllocatedBuffer != NULL)
//                             {
//                                 PsReturnProcessPagedPoolQuota(NotifyChange->OwningProcess, NotifyChange->ThisBufferLength as _);
//                                 ExFreePoolWithTag(NotifyChange->AllocatedBuffer, TAG_FS_NOTIFICATIONS);
//                             }
//
//                             NotifyChange->Buffer = NULL;
//                             NotifyChange->AllocatedBuffer = NULL;
//                             NotifyChange->LastEntry = 0;
//                             NotifyChange->DataLength = 0;
//                             NotifyChange->ThisBufferLength = 0;
//                         }
//                     }
//                 }
//             }
//
//             /* If asking for old name in case of a rename, notify later on,
//              * so that we can wait for new name.
//              * https://learn.microsoft.com/en-us/openspecs/main/ms-openspeclp/3589baea-5b22-48f2-9d43-f5bea4960ddb
//              */
//             if (Action == FILE_ACTION_RENAMED_OLD_NAME)
//             {
//                 NotifyChange->Flags |= NOTIFY_LATER;
//             }
//             else
//             {
//                 NotifyChange->Flags &= ~NOTIFY_LATER;
//                 if (!IsListEmpty(&NotifyChange->NotifyIrps))
//                 {
//                     FsRtlNotifyCompleteIrpList(NotifyChange, STATUS_SUCCESS);
//                 }
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         FsRtlNotifyReleaseFastMutex(RealNotifySync);
//     }
//     _SEH2_END;
// }
//
// /*++
//  * @name FsRtlNotifyFullChangeDirectory
//  * @implemented
//  *
//  * Lets FSD know if changes occures in the specified directory.
//  *
//  * @param NotifySync
//  *        Synchronization object pointer
//  *
//  * @param NotifyList
//  *        Notify list pointer (to head)
//  *
//  * @param FsContext
//  *        Used to identify the notify structure
//  *
//  * @param FullDirectoryName
//  *        String (A or W) containing the full directory name
//  *
//  * @param WatchTree
//  *        True to notify changes in subdirectories too
//  *
//  * @param IgnoreBuffer
//  *        True to reenumerate directory. It's ignored it NotifyIrp is null
//  *
//  * @param CompletionFilter
//  *        Used to define types of changes to notify
//  *
//  * @param NotifyIrp
//  *        IRP pointer to complete notify operation. It can be null
//  *
//  * @param TraverseCallback
//  *        Pointer to a callback function. It's called each time a change is
//  *        done in a subdirectory of the main directory. It's ignored it NotifyIrp
//  *        is null
//  *
//  * @param SubjectContext
//  *        Pointer to pass to SubjectContext member of TraverseCallback.
//  *        It's freed after use. It's ignored it NotifyIrp is null
//  *
//  * @return None
//  *
//  * @remarks This function only redirects to FsRtlNotifyFilterChangeDirectory.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyFullChangeDirectory(IN PNOTIFY_SYNC NotifySync,
//                                IN PLIST_ENTRY NotifyList,
//                                IN PVOID FsContext,
//                                IN PSTRING FullDirectoryName,
//                                IN BOOLEAN WatchTree,
//                                IN BOOLEAN IgnoreBuffer,
//                                IN ULONG CompletionFilter,
//                                IN PIRP NotifyIrp,
//                                IN PCHECK_FOR_TRAVERSE_ACCESS TraverseCallback OPTIONAL,
//                                IN PSECURITY_SUBJECT_CONTEXT SubjectContext OPTIONAL)
// {
//     FsRtlNotifyFilterChangeDirectory(NotifySync,
//                                      NotifyList,
//                                      FsContext,
//                                      FullDirectoryName,
//                                      WatchTree,
//                                      IgnoreBuffer,
//                                      CompletionFilter,
//                                      NotifyIrp,
//                                      TraverseCallback,
//                                      SubjectContext,
//                                      NULL);
// }
//
// /*++
//  * @name FsRtlNotifyFullReportChange
//  * @implemented
//  *
//  * Complets the pending notify IRPs.
//  *
//  * @param NotifySync
//  *        Synchronization object pointer
//  *
//  * @param NotifyList
//  *        Notify list pointer (to head)
//  *
//  * @param FullTargetName
//  *        String (A or W) containing the full directory name that changed
//  *
//  * @param TargetNameOffset
//  *        Offset, in FullTargetName, of the final component that is in the changed directory
//  *
//  * @param StreamName
//  *        String (A or W) containing a stream name
//  *
//  * @param NormalizedParentName
//  *        String (A or W) containing the full directory name that changed with long names
//  *
//  * @param FilterMatch
//  *        Flags that will be compared to the completion filter
//  *
//  * @param Action
//  *        Action code to store in user's buffer
//  *
//  * @param TargetContext
//  *        Pointer to a callback function. It's called each time a change is
//  *        done in a subdirectory of the main directory.
//  *
//  * @return None
//  *
//  * @remarks This function only redirects to FsRtlNotifyFilterReportChange.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyFullReportChange(IN PNOTIFY_SYNC NotifySync,
//                             IN PLIST_ENTRY NotifyList,
//                             IN PSTRING FullTargetName,
//                             IN USHORT TargetNameOffset,
//                             IN PSTRING StreamName OPTIONAL,
//                             IN PSTRING NormalizedParentName OPTIONAL,
//                             IN ULONG FilterMatch,
//                             IN ULONG Action,
//                             IN PVOID TargetContext)
// {
//     FsRtlNotifyFilterReportChange(NotifySync,
//                                   NotifyList,
//                                   FullTargetName,
//                                   TargetNameOffset,
//                                   StreamName,
//                                   NormalizedParentName,
//                                   FilterMatch,
//                                   Action,
//                                   TargetContext,
//                                   NULL);
// }
//
// /*++
//  * @name FsRtlNotifyInitializeSync
//  * @implemented
//  *
//  * Allocates the internal structure associated with notifications.
//  *
//  * @param NotifySync
//  *        Opaque pointer. It will receive the address of the allocated internal structure.
//  *
//  * @return None
//  *
//  * @remarks This function raise an exception in case of a failure.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyInitializeSync(IN PNOTIFY_SYNC *NotifySync)
// {
//     PREAL_NOTIFY_SYNC RealNotifySync;
//
//     *NotifySync = NULL;
//
//     RealNotifySync = ExAllocatePoolWithTag(NonPagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                            sizeof(REAL_NOTIFY_SYNC), 'SNSF');
//     ExInitializeFastMutex(&(RealNotifySync->FastMutex));
//     RealNotifySync->OwningThread = 0;
//     RealNotifySync->OwnerCount = 0;
//
//     *NotifySync = RealNotifySync;
// }
//
// /*++
//  * @name FsRtlNotifyReportChange
//  * @implemented
//  *
//  * Complets the pending notify IRPs.
//  *
//  * @param NotifySync
//  *        Synchronization object pointer
//  *
//  * @param NotifyList
//  *        Notify list pointer (to head)
//  *
//  * @param FullTargetName
//  *        String (A or W) containing the full directory name that changed
//  *
//  * @param FileNamePartLength
//  *        Length of the final component that is in the changed directory
//  *
//  * @param FilterMatch
//  *        Flags that will be compared to the completion filter
//  *
//  * @return None
//  *
//  * @remarks This function only redirects to FsRtlNotifyFilterReportChange.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyReportChange(IN PNOTIFY_SYNC NotifySync,
//                         IN PLIST_ENTRY NotifyList,
//                         IN PSTRING FullTargetName,
//                         IN PUSHORT FileNamePartLength,
//                         IN ULONG FilterMatch)
// {
//       FsRtlNotifyFilterReportChange(NotifySync,
//                                     NotifyList,
//                                     FullTargetName,
//                                     FullTargetName->Length - *FileNamePartLength,
//                                     NULL,
//                                     NULL,
//                                     FilterMatch,
//                                     0,
//                                     NULL,
//                                     NULL);
// }
//
// /*++
//  * @name FsRtlNotifyUninitializeSync
//  * @implemented
//  *
//  * Uninitialize a NOTIFY_SYNC object
//  *
//  * @param NotifySync
//  *        Address of a pointer to a PNOTIFY_SYNC object previously
//  *        initialized by FsRtlNotifyInitializeSync()
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlNotifyUninitializeSync(IN PNOTIFY_SYNC *NotifySync)
// {
//     if (*NotifySync)
//     {
//         ExFreePoolWithTag(*NotifySync, 'SNSF');
//         *NotifySync = NULL;
//     }
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
