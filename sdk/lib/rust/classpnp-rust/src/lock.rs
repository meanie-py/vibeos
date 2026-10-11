use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{CLONG, KeInitializeSpinLock, PCSTR, PDEVICE_OBJECT, PEX_RUNDOWN_REF_CACHE_AWARE, PIRP, PRTL_GENERIC_TABLE, RTL_GENERIC_TABLE, RtlInitializeGenericTable, RtlLookupElementGenericTable, RtlNumberGenericTableElements, POOL_TYPE};
use ntkrnl_sys::ddk::{_RTL_GENERIC_COMPARE_RESULTS_GenericLessThan as GenericLessThan, _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan as GenericGreaterThan, _RTL_GENERIC_COMPARE_RESULTS_GenericEqual as GenericEqual};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_LEVEL_ERROR};
use crate::types::{COMMON_DEVICE_EXTENSION, PCOMMON_DEVICE_EXTENSION};
use ntkrnl_sys::ddk::{CCHAR, PSTR, RTL_GENERIC_COMPARE_RESULTS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_LOCK_TRACKING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_LOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
// resolve_imports.py: explicit kernel/private-type imports
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ExFreePool, RtlAssert};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut LockHighWatermark: LONG = 0 as LONG;

pub static mut LockLowWatermark: LONG = 0 as LONG;

pub static mut MaxLockedMinutes: LONG = 5 as LONG;

#[repr(C)]
pub struct _REMOVE_TRACKING_BLOCK {
    pub Tag: PVOID,
    pub TimeLocked: LARGE_INTEGER,
    pub File: PCSTR,
    pub Line: ULONG,
}
pub type REMOVE_TRACKING_BLOCK = _REMOVE_TRACKING_BLOCK;
pub type PREMOVE_TRACKING_BLOCK = *mut _REMOVE_TRACKING_BLOCK;
#[no_mangle]
pub unsafe extern "system" fn ClassCompleteRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    PriorityBoost: CCHAR,
) {
    // The whole tracking check is `#if DBG` in lock.c, and it has to be here too:
    // a release build never initializes the lock -- ClasspInitializeRemoveTracking
    // stores C's `(ULONG_PTR)-1` marker in it, which a spinlock reads as HELD -- so
    // acquiring it unconditionally spun forever at DISPATCH_LEVEL on the first
    // client IRP any classpnp disk completed. Every hard disk hung the boot in
    // DiskInitFdo's MBR read; smoke never attaches one, so nothing noticed.
    #[cfg(debug_assertions)]
    {
        let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
        let mut removeTrackingList: PRTL_GENERIC_TABLE = core::ptr::null_mut();
        let mut searchDataBlock: REMOVE_TRACKING_BLOCK = REMOVE_TRACKING_BLOCK {
            Tag: core::ptr::null_mut(),
            TimeLocked: LARGE_INTEGER { QuadPart: 0 },
            File: core::ptr::null(),
            Line: 0,
        };
        let mut foundTrackingBlock: PREMOVE_TRACKING_BLOCK = core::ptr::null_mut();
        let mut oldIrql: KIRQL = 0;

        KeAcquireSpinLock(
            (&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(),
            &raw mut oldIrql,
        );

        removeTrackingList = ((*commonExtension).RemoveTrackingList).cast();

        if !removeTrackingList.is_null() {
            searchDataBlock.Tag = Irp as PVOID;
            foundTrackingBlock = RtlLookupElementGenericTable(
                removeTrackingList,
                &raw mut searchDataBlock as PVOID,
            ) as PREMOVE_TRACKING_BLOCK;

            if !foundTrackingBlock.is_null() {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_LOCK,
                    b">>>>>ClassCompleteRequest: Irp %p completed while still holding the remove lock\n\0"
                        .as_ptr() as PCSTR,
                    Irp,
                );
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_LOCK,
                    b">>>>>ClassCompleteRequest: Lock acquired in file %s on line %d\n\0"
                        .as_ptr() as PCSTR,
                    (*foundTrackingBlock).File,
                    (*foundTrackingBlock).Line,
                );
                NT_ASSERT!(0);
            }
        }

        KeReleaseSpinLock((&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(), oldIrql);
    }
    #[cfg(not(debug_assertions))]
    let _ = DeviceObject;

    IoCompleteRequest(Irp, PriorityBoost);
}
#[no_mangle]
pub unsafe extern "system" fn RemoveTrackingCompareRoutine(
    Table: PRTL_GENERIC_TABLE,
    FirstStruct: PVOID,
    SecondStruct: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    let tag1 = (*(FirstStruct as PREMOVE_TRACKING_BLOCK)).Tag;
    let tag2 = (*(SecondStruct as PREMOVE_TRACKING_BLOCK)).Tag;
    if tag1 < tag2 {
        return GenericLessThan;
    } else if tag1 > tag2 {
        return GenericGreaterThan;
    }
    GenericEqual
}
#[no_mangle]
pub unsafe extern "system" fn RemoveTrackingAllocateRoutine(
    Table: PRTL_GENERIC_TABLE,
    ByteSize: CLONG,
) -> PVOID {
    ExAllocatePoolWithTag(
        NonPagedPoolNx as POOL_TYPE,
        ByteSize as usize,
        CLASS_TAG_LOCK_TRACKING,
    )
}
#[no_mangle]
pub unsafe extern "system" fn RemoveTrackingFreeRoutine(Table: PRTL_GENERIC_TABLE, Buffer: *mut core::ffi::c_void) {
    let _ = Table;
    if !Buffer.is_null() {
        ExFreePool(Buffer);
        // C source zeros the local pointer here (no-op after free)
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeRemoveTracking(DeviceObject: PDEVICE_OBJECT) {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    #[cfg(debug_assertions)]
    {
        KeInitializeSpinLock(&raw mut (*commonExtension).RemoveTrackingSpinlock);
        (*commonExtension).RemoveTrackingList = (ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<RTL_GENERIC_TABLE>() as SIZE_T,
            CLASS_TAG_LOCK_TRACKING as ULONG,
        ) as *mut RTL_GENERIC_TABLE).cast();
        if !(*commonExtension).RemoveTrackingList.is_null() {
            RtlInitializeGenericTable(
                ((*commonExtension).RemoveTrackingList).cast(),
                Some(abi_cast!(RemoveTrackingCompareRoutine)),
                Some(abi_cast!(RemoveTrackingAllocateRoutine)),
                Some(abi_cast!(RemoveTrackingFreeRoutine)),
                core::ptr::null_mut(),
            );
        }
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = DeviceObject;
        (*commonExtension).RemoveTrackingSpinlock = !0 as _;
        (*commonExtension).RemoveTrackingList = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspUninitializeRemoveTracking(DeviceObject: PDEVICE_OBJECT) {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let removeTrackingList = (*commonExtension).RemoveTrackingList;

    let condition = if (*commonExtension).RemoveTrackingUntrackedCount == 0 && !removeTrackingList.is_null() {
        RtlNumberGenericTableElements((removeTrackingList).cast()) == 0
    } else {
        true
    };

    if !condition {
        RtlAssert(
            b"commonExtension->RemoveTrackingUntrackedCount == 0 && removeTrackingList != NULL ? RtlNumberGenericTableElements(removeTrackingList) == 0 : TRUE\0".as_ptr() as PVOID,
            b"lock.c\0".as_ptr() as PVOID,
            0u32,
            b"Removing the device while still holding remove locks\0".as_ptr() as PSTR,
        );
    }

    if !removeTrackingList.is_null() {
        let mut oldIrql: KIRQL = 0;
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*commonExtension).RemoveTrackingSpinlock)).cast(),
            &mut oldIrql,
        );

        ExFreePool(removeTrackingList as *mut core::ffi::c_void);

        (*commonExtension).RemoveTrackingList = core::ptr::null_mut();

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*commonExtension).RemoveTrackingSpinlock)).cast(),
            oldIrql,
        );
    }
}
