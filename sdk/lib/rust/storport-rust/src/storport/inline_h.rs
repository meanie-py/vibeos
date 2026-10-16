use crate::prelude::*;
use ntkrnl_sys::ddk::{KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeReleaseInStackQueuedSpinLock, PIRP};
use core::ffi::c_void;
pub unsafe extern "C" fn StorpCompleteRequest(
    Irp: PIRP,
    SrbStatus: UCHAR,
    IrpStatus: NTSTATUS,
) {
    (*((*IoGetCurrentIrpStackLocation(Irp)).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK)).SrbStatus = SrbStatus;
    (*Irp).IoStatus.__bindgen_anon_1.Status = IrpStatus;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
}

pub unsafe extern "C" fn StorpGetMiniportFdo(HwDeviceExtension: PVOID) -> PFDO_DEVICE_EXTENSION {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION;

    NT_ASSERT!(!HwDeviceExtension.is_null());

    MiniportExtension = (HwDeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as *mut MINIPORT_DEVICE_EXTENSION as PMINIPORT_DEVICE_EXTENSION;

    return (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;
}

pub unsafe extern "C" fn StorpInterlockedInsertHeadListCounted(
    SpinLock: PKSPIN_LOCK,
    ListHead: PLIST_ENTRY,
    Entry: PLIST_ENTRY,
    Counter: PULONG,
) -> ULONG {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KeAcquireInStackQueuedSpinLock(SpinLock, core::ptr::addr_of_mut!(LockHandle));
    InsertHeadList(ListHead, Entry);
    *Counter = (*Counter).wrapping_add(1);
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return *Counter;
}

unsafe extern "C" fn StorpInterlockedInsertTailListCounted(
    SpinLock: PKSPIN_LOCK,
    ListHead: PLIST_ENTRY,
    Entry: PLIST_ENTRY,
    Counter: PULONG,
) -> ULONG {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    KeAcquireInStackQueuedSpinLock(SpinLock, core::ptr::addr_of_mut!(LockHandle));
    InsertTailList(ListHead, Entry);
    *Counter = (*Counter).wrapping_add(1);
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return *Counter;
}

pub unsafe extern "C" fn StorpInterlockedRemoveEntryListCounted(
    mut SpinLock: PKSPIN_LOCK,
    mut Entry: PLIST_ENTRY,
    mut Counter: PULONG,
) -> BOOLEAN {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Empty: BOOLEAN = 0;

    KeAcquireInStackQueuedSpinLock(SpinLock, core::ptr::addr_of_mut!(LockHandle));
    Empty = RemoveEntryList(Entry);
    *Counter = (*Counter).wrapping_sub(1);

    NT_ASSERT!(
        (Empty as i32) == ((*Counter == 0) as i32)
    );

    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return Empty;
}

unsafe extern "C" fn StorpInterlockedRemoveHeadListCounted(
    SpinLock: PKSPIN_LOCK,
    ListHead: PLIST_ENTRY,
    Counter: PULONG,
) -> PLIST_ENTRY {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Entry: PLIST_ENTRY;

    KeAcquireInStackQueuedSpinLock(SpinLock, core::ptr::addr_of_mut!(LockHandle));

    NT_ASSERT!(
        (IsListEmpty(ListHead as *const LIST_ENTRY) as i32)
            == ((*Counter == 0) as i32),
    );

    Entry = RemoveHeadList(ListHead);
    *Counter = (*Counter).wrapping_sub(1);
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return Entry;
}

unsafe extern "C" fn StorpInterlockedRemoveTailListCounted(
    mut SpinLock: PKSPIN_LOCK,
    mut ListHead: PLIST_ENTRY,
    mut Counter: PULONG,
) -> PLIST_ENTRY {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();

    KeAcquireInStackQueuedSpinLock(SpinLock, core::ptr::addr_of_mut!(LockHandle));

    NT_ASSERT!(
        (IsListEmpty(ListHead as *const LIST_ENTRY) as i32)
            == ((*Counter == 0) as i32),
    );

    Entry = RemoveTailList(ListHead);
    *Counter = (*Counter).wrapping_sub(1);
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return Entry;
}

pub unsafe extern "C" fn StorpCheckFlowControl(FdoExtension: PFDO_DEVICE_EXTENSION) {
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut PdoExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PdoFlowControl: PPDO_IO_FLOW_CONTROL = core::ptr::null_mut();
    let mut FdoFlowControl: PFDO_IO_FLOW_CONTROL = core::ptr::null_mut();
    let mut PdoEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PdoOutstandingSum: ULONG = 0;
    let mut PdoStrongOrderedSum: ULONG = 0;

    FdoFlowControl = &raw mut (*FdoExtension).FlowControl;

    KeAcquireInStackQueuedSpinLock(&raw mut (*FdoExtension).PdoListLock, &raw mut LockHandle);
    PdoEntry = (*FdoExtension).PdoListHead.Flink;

    while PdoEntry != &raw mut (*FdoExtension).PdoListHead {
        /* Obtain PDO extension and flow control */
        PdoExtension = (PdoEntry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(PDO_DEVICE_EXTENSION, PdoListEntry) as ULONG_PTR)
            as PPDO_DEVICE_EXTENSION;
        PdoFlowControl = &raw mut (*PdoExtension).FlowControl;

        /* Accumulate important values and compare to FDO record */
        PdoOutstandingSum = PdoOutstandingSum.wrapping_add((*PdoFlowControl).OutstandingRequestCount);
        PdoStrongOrderedSum = PdoStrongOrderedSum.wrapping_add((*PdoFlowControl).StrongOrderedCount);

        /* Next element */
        PdoEntry = (*PdoEntry).Flink;
    }

    UNREFERENCED_PARAMETER!(PdoStrongOrderedSum); /* Don't know what to do with it */

    /* Compare */
    NT_ASSERT!(PdoOutstandingSum == (*FdoFlowControl).OutstandingRequestCount);

    KeReleaseInStackQueuedSpinLock(&raw mut LockHandle);
}
