use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    HANDLE,
    _UNICODE_STRING,
    KEVENT,
    _DEVICE_OBJECT_POWER_EXTENSION,
    PDEVICE_OBJECT_POWER_EXTENSION,
    PDEVICE_OBJECT,
    PDEVOBJ_EXTENSION,
    PEXTENDED_DEVOBJ_EXTENSION,
    POBJECT_NAME_INFORMATION,
    _OBJECT_ATTRIBUTES,
    _IO_STATUS_BLOCK,
    OBJECT_NAME_INFORMATION,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    KeInitializeEvent,
    KeWaitForSingleObject,
    ObQueryNameString,
    PsCreateSystemThread,
    ZwClose,
    ZwCreateFile,
    ZwFlushBuffersFile,
    ZwFlushKey,
    ZwOpenKey,
};

extern "C" {
    // ntoskrnl-rust shim_exports: (address, type size, field offset). Declared here with
    // null for both, the offset was 0 and `Volume` is the LAST field of the extension, so
    // every volume walk read a DEVICE_OBJECT from inside the list links: the shutdown
    // flush bugchecked 0x1E (setup's reboot, the install smoke).
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );
    fn IoGetDevObjExtension(DeviceObject: PDEVICE_OBJECT) -> PEXTENDED_DEVOBJ_EXTENSION;
}

const TAG_PO_DOPE: ULONG = 0x45504F44;
const FILE_FLOPPY_DISKETTE: ULONG = 0x00000004;
const FILE_READ_ONLY_DEVICE: ULONG = 0x00000002;
const VPB_MOUNTED: u16 = 0x0001;
const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_OPEN: ULONG = 0x00000001;
const SYNCHRONIZE: u32 = 0x00100000;
const FILE_READ_DATA: u32 = 0x0001;
const FILE_WRITE_DATA: u32 = 0x0002;
const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const KEY_READ: u32 = 0x20019;
// 0x1F03FF = STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|0x3FF. xdk/pstypes.h picks this
// under `#if (NTDDI_VERSION >= NTDDI_VISTA) ... #else`, and this tree builds
// -D_WIN32_WINNT=0x502 (Server 2003), so the PRE-Vista branch is the live one. The
// Vista value 0x1FFFFF asks for 0xFC00 of thread rights the object's GENERIC_MAPPING
// does not grant, which an open would refuse rather than silently narrow.
const THREAD_ALL_ACCESS: u32 = 0x1F03FF;
const IO_NO_INCREMENT: i32 = 0;

const PowerSystemUnspecified: ULONG = 0;
const PowerDeviceUnspecified: ULONG = 0;

const KernelMode: i8 = 0;

#[repr(C)]
struct POP_FLUSH_VOLUME {
    List: LIST_ENTRY,
    Count: LONG,
    Wait: KEVENT,
}

#[no_mangle]
pub static mut PopFlushPolicy: ULONG = 0;

#[no_mangle]
pub static mut PopVolumeLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PopVolumeDevices: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut PopDopeGlobalLock: KSPIN_LOCK = 0;

#[no_mangle]
pub unsafe extern "system" fn PopGetDope(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT_POWER_EXTENSION {
    let DeviceObject = opaque_ptr(DeviceObject);
    let device_extension: PEXTENDED_DEVOBJ_EXTENSION;
    let mut dope: PDEVICE_OBJECT_POWER_EXTENSION;

    device_extension = IoGetDevObjExtension(DeviceObject);
    if !(*device_extension).Dope.is_null() {
        return (*device_extension).Dope as PDEVICE_OBJECT_POWER_EXTENSION;
    }

    dope = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_DEVICE_OBJECT_POWER_EXTENSION>(),
        TAG_PO_DOPE,
    ) as PDEVICE_OBJECT_POWER_EXTENSION;
    if dope.is_null() {
        return (*device_extension).Dope as PDEVICE_OBJECT_POWER_EXTENSION;
    }

    RtlZeroMemory(
        dope as PVOID,
        core::mem::size_of::<_DEVICE_OBJECT_POWER_EXTENSION>(),
    );
    (*dope).DeviceObject = DeviceObject;
    (*dope).State = PowerDeviceUnspecified;
    InitializeListHead(core::ptr::addr_of_mut!((*dope).IdleList) as PLIST_ENTRY);

    {
        // RAII: guard drops (releases PopDopeGlobalLock) at the end of THIS
        // block — deliberately scoped to exclude the ExFreePoolWithTag below,
        // which must run at the lowered IRQL, NOT while holding the spinlock.
        let _dope_lock = ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!(PopDopeGlobalLock) as ntkrnl_sys::ke::PKSPIN_LOCK,
        );
        if (*device_extension).Dope.is_null() {
            (*device_extension).Dope = dope;
            dope = core::ptr::null_mut();
        }
    }

    if !dope.is_null() {
        ExFreePoolWithTag(dope as PVOID, TAG_PO_DOPE);
    }

    (*device_extension).Dope as PDEVICE_OBJECT_POWER_EXTENSION
}

#[no_mangle]
pub unsafe extern "system" fn PoVolumeDevice(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let dope: PDEVICE_OBJECT_POWER_EXTENSION;

    dope = PopGetDope(DeviceObject);
    if !dope.is_null() {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

        if (*dope).Volume.Flink.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!(PopVolumeDevices) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*dope).Volume) as PLIST_ENTRY,
            );
        }

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));
    }
}

#[no_mangle]
pub unsafe extern "system" fn PoRemoveVolumeDevice(DeviceObject: PDEVICE_OBJECT) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let dope: PDEVICE_OBJECT_POWER_EXTENSION;
    let device_extension: PEXTENDED_DEVOBJ_EXTENSION;

    device_extension = IoGetDevObjExtension(DeviceObject);
    if (*device_extension).Dope.is_null() {
        return;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

    dope = (*device_extension).Dope as PDEVICE_OBJECT_POWER_EXTENSION;

    if !(*dope).Volume.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*dope).Volume) as PLIST_ENTRY);
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

    {
        // RAII: release before the ExFreePoolWithTag (which must run outside
        // the spinlock, at the lowered IRQL).
        let _dope_lock = ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!(PopDopeGlobalLock) as ntkrnl_sys::ke::PKSPIN_LOCK,
        );
        (*device_extension).Dope = core::ptr::null_mut();
    }

    ExFreePoolWithTag(dope as PVOID, TAG_PO_DOPE);
}

#[no_mangle]
pub unsafe extern "system" fn PopFlushVolumeWorker(Context: PVOID) {
    let Context = opaque_ptr(Context);
    PopFlushVolumeWorkerInner(Context)
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopFlushVolumeWorkerInner(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let flush_context = Context as *mut POP_FLUSH_VOLUME;
    let mut dope: PDEVICE_OBJECT_POWER_EXTENSION;
    let mut next_entry: PLIST_ENTRY;
    let mut status: NTSTATUS;
    let mut buffer: [u8; core::mem::size_of::<OBJECT_NAME_INFORMATION>() + 512] =
        core::mem::zeroed();
    let name_info = buffer.as_mut_ptr() as POBJECT_NAME_INFORMATION;
    let mut length: ULONG = 0;
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut volume_handle: HANDLE = core::ptr::null_mut();
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

    while !IsListEmpty(core::ptr::addr_of_mut!((*flush_context).List) as PLIST_ENTRY) {
        next_entry = (*flush_context).List.Flink;
        RemoveEntryList(next_entry);

        InsertTailList(core::ptr::addr_of_mut!(PopVolumeDevices) as PLIST_ENTRY, next_entry);

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

        dope = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEVICE_OBJECT_POWER_EXTENSION>(),
            core::mem::offset_of!(_DEVICE_OBJECT_POWER_EXTENSION, Volume),
        ) as PDEVICE_OBJECT_POWER_EXTENSION;

        status = ObQueryNameString(
            (*dope).DeviceObject as PVOID,
            name_info,
            buffer.len() as ULONG,
            core::ptr::addr_of_mut!(length),
        );
        if status >= 0 && !(*name_info).Name.Buffer.is_null() {
            InitializeObjectAttributes(
                core::ptr::addr_of_mut!(object_attributes),
                core::ptr::addr_of_mut!((*name_info).Name),
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            status = ZwCreateFile(
                core::ptr::addr_of_mut!(volume_handle),
                SYNCHRONIZE | FILE_READ_DATA | FILE_WRITE_DATA,
                core::ptr::addr_of_mut!(object_attributes),
                core::ptr::addr_of_mut!(io_status_block),
                core::ptr::null_mut(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_OPEN,
                0,
                core::ptr::null_mut(),
                0,
            );
            if status >= 0 {
                ZwFlushBuffersFile(volume_handle, core::ptr::addr_of_mut!(io_status_block));
                ZwClose(volume_handle);
            }
        }

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));
    }

    (*flush_context).Count -= 1;
    if (*flush_context).Count == 0 {
        KeSetEvent(core::ptr::addr_of_mut!((*flush_context).Wait) as PVOID, IO_NO_INCREMENT, 0u8);
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));
}

#[no_mangle]
pub unsafe extern "system" fn PopFlushVolumes(ShuttingDown: BOOLEAN) {
    let mut flush_context: POP_FLUSH_VOLUME = core::mem::zeroed();
    let flush_policy: ULONG;
    // L"\\Registry" — UTF-16LE
    static REGISTRY_NAME: [u16; 10] = [
        0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, 0x00,
    ];
    let mut registry_name = _UNICODE_STRING {
        Length: 18,        // 9 chars * 2
        MaximumLength: 20, // 10 u16s * 2
        Buffer: REGISTRY_NAME.as_ptr() as *mut u16,
    };
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut registry_handle: HANDLE = core::ptr::null_mut();
    let mut next_entry: PLIST_ENTRY;
    let mut dope: PDEVICE_OBJECT_POWER_EXTENSION;
    let mut volume_count: ULONG = 0;
    let mut status: NTSTATUS;
    let mut thread_handle: HANDLE = core::ptr::null_mut();
    let mut thread_count: ULONG;

    InitializeListHead(core::ptr::addr_of_mut!(flush_context.List) as PLIST_ENTRY);
    KeInitializeEvent(core::ptr::addr_of_mut!(flush_context.Wait), _EVENT_TYPE_NotificationEvent, 0);

    flush_policy = if ShuttingDown != 0 {
        1 | 2
    } else {
        PopFlushPolicy
    };

    if (flush_policy & 1) != 0 {
        InitializeObjectAttributes(
            core::ptr::addr_of_mut!(object_attributes),
            core::ptr::addr_of_mut!(registry_name),
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        status = ZwOpenKey(
            core::ptr::addr_of_mut!(registry_handle),
            KEY_READ,
            core::ptr::addr_of_mut!(object_attributes),
        );
        if status >= 0 {
            ZwFlushKey(registry_handle);
            ZwClose(registry_handle);
        }
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

    next_entry = PopVolumeDevices.Flink;
    while next_entry != core::ptr::addr_of_mut!(PopVolumeDevices) as PLIST_ENTRY {
        dope = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEVICE_OBJECT_POWER_EXTENSION>(),
            core::mem::offset_of!(_DEVICE_OBJECT_POWER_EXTENSION, Volume),
        ) as PDEVICE_OBJECT_POWER_EXTENSION;

        next_entry = (*next_entry).Flink;

        if ((*(*(*dope).DeviceObject).Vpb).Flags & VPB_MOUNTED) == 0
            || ((*(*dope).DeviceObject).Characteristics & FILE_FLOPPY_DISKETTE) != 0
            || ((*(*dope).DeviceObject).Characteristics & FILE_READ_ONLY_DEVICE) != 0
            || (!(*(*(*dope).DeviceObject).Vpb).RealDevice.is_null()
                && ((*(*(*(*dope).DeviceObject).Vpb).RealDevice).Characteristics
                    & FILE_FLOPPY_DISKETTE)
                    != 0)
        {
            continue;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*dope).Volume) as PLIST_ENTRY);
        InsertTailList(
            core::ptr::addr_of_mut!(flush_context.List) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*dope).Volume) as PLIST_ENTRY,
        );

        volume_count += 1;
    }

    if (flush_policy & 2) == 0 {
        // UNIMPLEMENTED
    }

    if volume_count == 0 {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));
        return;
    }

    thread_count = if volume_count < 8 { volume_count } else { 8 };
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(object_attributes),
        core::ptr::null_mut(),
        OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    flush_context.Count = 1;
    thread_count -= 1;

    while thread_count > 0 {
        thread_count -= 1;
        status = PsCreateSystemThread(
            core::ptr::addr_of_mut!(thread_handle),
            THREAD_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(PopFlushVolumeWorker),
            core::ptr::addr_of_mut!(flush_context) as PVOID,
        );
        if status >= 0 {
            flush_context.Count += 1;
            ZwClose(thread_handle);
        }
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock));

    PopFlushVolumeWorker(core::ptr::addr_of_mut!(flush_context) as PVOID);

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(flush_context.Wait) as PVOID,
        _KWAIT_REASON_Executive,
        KernelMode,
        0u8,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn PoInitializeDeviceObject(
    DeviceObjectExtension: PDEVOBJ_EXTENSION,
) {
    let device_extension = DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;

    (*device_extension).PowerFlags = PowerSystemUnspecified & 0xF;
    (*device_extension).PowerFlags |= (PowerDeviceUnspecified << 4) & 0xF0;

    (*device_extension).Dope = core::ptr::null_mut();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/po/povolume.c
//  * PURPOSE:         Power Manager DOPE and Volume Management
//  * PROGRAMMERS:     ReactOS Portable Systems Group
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
// typedef struct _POP_FLUSH_VOLUME
// {
//     LIST_ENTRY List;
//     LONG Count;
//     KEVENT Wait;
// } POP_FLUSH_VOLUME, *PPOP_FLUSH_VOLUME;
//
// ULONG PopFlushPolicy = 0;
//
// KGUARDED_MUTEX PopVolumeLock;
// LIST_ENTRY PopVolumeDevices;
// KSPIN_LOCK PopDopeGlobalLock;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PDEVICE_OBJECT_POWER_EXTENSION
// NTAPI
// PopGetDope(IN PDEVICE_OBJECT DeviceObject)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT_POWER_EXTENSION Dope;
//     KIRQL OldIrql;
//     PAGED_CODE();
//
//     /* If the device already has the dope, return it */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//     if (DeviceExtension->Dope) goto Return;
//
//     /* Allocate some dope for the device */
//     Dope = ExAllocatePoolWithTag(NonPagedPool,
//                                  sizeof(DEVICE_OBJECT_POWER_EXTENSION),
//                                  TAG_PO_DOPE);
//     if (!Dope) goto Return;
//
//     /* Initialize the initial contents of the dope */
//     RtlZeroMemory(Dope, sizeof(DEVICE_OBJECT_POWER_EXTENSION));
//     Dope->DeviceObject = DeviceObject;
//     Dope->State = PowerDeviceUnspecified;
//     InitializeListHead(&Dope->IdleList);
//
//     /* Make sure only one caller can assign dope to a device */
//     KeAcquireSpinLock(&PopDopeGlobalLock, &OldIrql);
//
//     /* Make sure the device still has no dope */
//     if (!DeviceExtension->Dope)
//     {
//         /* Give the local dope to this device, and remember we won the race */
//         DeviceExtension->Dope = (PVOID)Dope;
//         Dope = NULL;
//     }
//
//     /* Allow other dope transactions now */
//     KeReleaseSpinLock(&PopDopeGlobalLock, OldIrql);
//
//     /* Check if someone other than us already assigned the dope, so free ours */
//     if (Dope) ExFreePoolWithTag(Dope, TAG_PO_DOPE);
//
//     /* Return the dope to the caller */
// Return:
//     return (PDEVICE_OBJECT_POWER_EXTENSION)DeviceExtension->Dope;
// }
//
// VOID
// NTAPI
// PoVolumeDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_OBJECT_POWER_EXTENSION Dope;
//     PAGED_CODE();
//
//     /* Get dope from the device (if the device has no dope, it will receive some) */
//     Dope = PopGetDope(DeviceObject);
//     if (Dope)
//     {
//         /* Make sure we can flush safely */
//         KeAcquireGuardedMutex(&PopVolumeLock);
//
//         /* Add this volume into the list of power-manager volumes */
//         if (!Dope->Volume.Flink) InsertTailList(&PopVolumeDevices, &Dope->Volume);
//
//         /* Allow flushes to go through */
//         KeReleaseGuardedMutex(&PopVolumeLock);
//     }
// }
//
// VOID
// NTAPI
// PoRemoveVolumeDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_OBJECT_POWER_EXTENSION Dope;
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     KIRQL OldIrql;
//     PAGED_CODE();
//
//     /* If the device already has the dope, return it */
//     DeviceExtension = IoGetDevObjExtension(DeviceObject);
//     if (!DeviceExtension->Dope)
//     {
//         /* no dope */
//         return;
//     }
//
//     /* Make sure we can flush safely */
//     KeAcquireGuardedMutex(&PopVolumeLock);
//
//     /* Get dope from device */
//     Dope = (PDEVICE_OBJECT_POWER_EXTENSION)DeviceExtension->Dope;
//
//     if (Dope->Volume.Flink)
//     {
//         /* Remove from volume from list */
//         RemoveEntryList(&Dope->Volume);
//     }
//
//     /* Allow flushes to go through */
//     KeReleaseGuardedMutex(&PopVolumeLock);
//
//     /* Now remove dope from device object */
//     KeAcquireSpinLock(&PopDopeGlobalLock, &OldIrql);
//
//     /* remove from dev obj */
//     DeviceExtension->Dope = NULL;
//
//     /* Release lock */
//     KeReleaseSpinLock(&PopDopeGlobalLock, OldIrql);
//
//     /* Free dope */
//     ExFreePoolWithTag(Dope, TAG_PO_DOPE);
// }
//
// VOID
// NTAPI
// PopFlushVolumeWorker(IN PVOID Context)
// {
//     PPOP_FLUSH_VOLUME FlushContext = (PPOP_FLUSH_VOLUME)Context;
//     PDEVICE_OBJECT_POWER_EXTENSION Dope;
//     PLIST_ENTRY NextEntry;
//     NTSTATUS Status;
//     UCHAR Buffer[sizeof(OBJECT_NAME_INFORMATION) + 512];
//     POBJECT_NAME_INFORMATION NameInfo = (PVOID)Buffer;
//     ULONG Length;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE VolumeHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Acquire the flush lock since we're messing with the list */
//     KeAcquireGuardedMutex(&PopVolumeLock);
//
//     /* Loop the flush list */
//     while (!IsListEmpty(&FlushContext->List))
//     {
//         /* Grab the next (ie: current) entry and remove it */
//         NextEntry = FlushContext->List.Flink;
//         RemoveEntryList(NextEntry);
//
//         /* Add it back on the volume list */
//         InsertTailList(&PopVolumeDevices, NextEntry);
//
//         /* Done touching the volume list */
//         KeReleaseGuardedMutex(&PopVolumeLock);
//
//         /* Get the dope from the volume link */
//         Dope = CONTAINING_RECORD(NextEntry, DEVICE_OBJECT_POWER_EXTENSION, Volume);
//
//         /* Get the name */
//         Status = ObQueryNameString(Dope->DeviceObject,
//                                    NameInfo,
//                                    sizeof(Buffer),
//                                    &Length);
//         if ((NT_SUCCESS(Status)) && (NameInfo->Name.Buffer))
//         {
//             /* Open the volume */
//             DPRINT("Opening: %wZ\n", &NameInfo->Name);
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &NameInfo->Name,
//                                        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                        0,
//                                        0);
//             Status = ZwCreateFile(&VolumeHandle,
//                                   SYNCHRONIZE | FILE_READ_DATA | FILE_WRITE_DATA,
//                                   &ObjectAttributes,
//                                   &IoStatusBlock,
//                                   NULL,
//                                   GENERIC_READ | GENERIC_WRITE,
//                                   FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                   FILE_OPEN,
//                                   0,
//                                   NULL,
//                                   0);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Flush it and close it */
//                 DPRINT("Sending flush to: %p\n", VolumeHandle);
//                 ZwFlushBuffersFile(VolumeHandle, &IoStatusBlock);
//                 ZwClose(VolumeHandle);
//             }
//         }
//
//         /* Acquire the flush lock again since we'll touch the list */
//         KeAcquireGuardedMutex(&PopVolumeLock);
//     }
//
//     /* One more flush completed... if it was the last, signal the caller */
//     if (!--FlushContext->Count) KeSetEvent(&FlushContext->Wait, IO_NO_INCREMENT, FALSE);
//
//     /* Serialize with flushers */
//     KeReleaseGuardedMutex(&PopVolumeLock);
// }
//
// VOID
// NTAPI
// PopFlushVolumes(IN BOOLEAN ShuttingDown)
// {
//     POP_FLUSH_VOLUME FlushContext = {{0}};
//     ULONG FlushPolicy;
//     UNICODE_STRING RegistryName = RTL_CONSTANT_STRING(L"\\Registry");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE RegistryHandle;
//     PLIST_ENTRY NextEntry;
//     PDEVICE_OBJECT_POWER_EXTENSION  Dope;
//     ULONG VolumeCount = 0;
//     NTSTATUS Status;
//     HANDLE ThreadHandle;
//     ULONG ThreadCount;
//
//     /* Setup the flush context */
//     InitializeListHead(&FlushContext.List);
//     KeInitializeEvent(&FlushContext.Wait, NotificationEvent, FALSE);
//
//     /* What to flush */
//     FlushPolicy = ShuttingDown ? 1 | 2 : PopFlushPolicy;
//     if ((FlushPolicy & 1))
//     {
//         /* Registry flush requested, so open it */
//         DPRINT("Opening registry\n");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &RegistryName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//         Status = ZwOpenKey(&RegistryHandle, KEY_READ, &ObjectAttributes);
//         if (NT_SUCCESS(Status))
//         {
//             /* Flush the registry */
//             DPRINT("Flushing registry\n");
//             ZwFlushKey(RegistryHandle);
//             ZwClose(RegistryHandle);
//         }
//     }
//
//     /* Serialize with other flushes */
//     KeAcquireGuardedMutex(&PopVolumeLock);
//
//     /* Scan the volume list */
//     NextEntry = PopVolumeDevices.Flink;
//     while (NextEntry != &PopVolumeDevices)
//     {
//         /* Get the dope from the link */
//         Dope = CONTAINING_RECORD(NextEntry, DEVICE_OBJECT_POWER_EXTENSION, Volume);
//
//         /* Grab the next entry now, since we'll be modifying the list */
//         NextEntry = NextEntry->Flink;
//
//         /* Make sure the object is mounted, writable, exists, and is not a floppy */
//         if (!(Dope->DeviceObject->Vpb->Flags & VPB_MOUNTED) ||
//             (Dope->DeviceObject->Characteristics & FILE_FLOPPY_DISKETTE) ||
//             (Dope->DeviceObject->Characteristics & FILE_READ_ONLY_DEVICE) ||
//             ((Dope->DeviceObject->Vpb->RealDevice) &&
//              (Dope->DeviceObject->Vpb->RealDevice->Characteristics & FILE_FLOPPY_DISKETTE)))
//         {
//             /* Not flushable */
//             continue;
//         }
//
//         /* Remove it from the dope and add it to the flush context list */
//         RemoveEntryList(&Dope->Volume);
//         InsertTailList(&FlushContext.List, &Dope->Volume);
//
//         /* Next */
//         VolumeCount++;
//     }
//
//     /* Check if we should skip non-removable devices */
//     if (!(FlushPolicy & 2))
//     {
//         /* ReactOS only implements this routine for shutdown, which requires it */
//         UNIMPLEMENTED;
//     }
//
//     /* Check if there were no volumes at all */
//     if (!VolumeCount)
//     {
//         /* Nothing to do */
//         KeReleaseGuardedMutex(&PopVolumeLock);
//         return;
//     }
//
//     /* Allocate up to 8 flusher threads */
//     ThreadCount = min(VolumeCount, 8);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* We will ourselves become a flusher thread */
//     FlushContext.Count = 1;
//     ThreadCount--;
//
//     /* Look for any extra ones we might need */
//     while (ThreadCount > 0)
//     {
//         /* Create a new one */
//         ThreadCount--;
//         DPRINT("Creating flush thread\n");
//         Status = PsCreateSystemThread(&ThreadHandle,
//                                       THREAD_ALL_ACCESS,
//                                       &ObjectAttributes,
//                                       0L,
//                                       NULL,
//                                       PopFlushVolumeWorker,
//                                       &FlushContext);
//         if (NT_SUCCESS(Status))
//         {
//             /* One more created... */
//             FlushContext.Count++;
//             ZwClose(ThreadHandle);
//         }
//     }
//
//     /* Allow flushes to go through */
//     KeReleaseGuardedMutex(&PopVolumeLock);
//
//     /* Enter the flush work */
//     DPRINT("Local flush\n");
//     PopFlushVolumeWorker(&FlushContext);
//
//     /* Wait for all flushes to be over */
//     DPRINT("Waiting for flushes\n");
//     KeWaitForSingleObject(&FlushContext.Wait, Executive, KernelMode, FALSE, NULL);
//     DPRINT("Flushes have completed\n");
// }
//
// VOID
// NTAPI
// PoInitializeDeviceObject(IN OUT PDEVOBJ_EXTENSION DeviceObjectExtension)
// {
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension = (PVOID)DeviceObjectExtension;
//     PAGED_CODE();
//
//     /* Initialize the power flags */
//     DeviceExtension->PowerFlags = PowerSystemUnspecified & 0xF;
//     DeviceExtension->PowerFlags |= ((PowerDeviceUnspecified << 4) & 0xF0);
//
//     /* The device object is not on drugs yet */
//     DeviceExtension->Dope = NULL;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<POP_FLUSH_VOLUME>() == 48);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, List) == 0);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, Count) == 16);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, Wait) == 24);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<POP_FLUSH_VOLUME>() == 28);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, List) == 0);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, Count) == 8);
    assert!(core::mem::offset_of!(POP_FLUSH_VOLUME, Wait) == 12);
};
// END OFFSET CHECKS
