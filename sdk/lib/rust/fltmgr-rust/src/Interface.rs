use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue;
use ntkrnl_sys::ddk::{PFS_FILTER_CALLBACKS, ERESOURCE, ExAcquireFastMutex, ExInitializeResourceLite, ExQueueWorkItem, ExReleaseFastMutex, FAST_IO_DISPATCH, FS_FILTER_CALLBACKS, IoAttachDeviceToDeviceStackSafe, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoDetachDevice, IoEnumerateDeviceObjectList, IoGetAttachedDeviceReference, IoGetDeviceAttachmentBaseRef, IoGetDeviceObjectPointer, IoGetDiskDeviceObject, IoGetLowerDeviceObject, IoRegisterFsRegistrationChange, LIST_ENTRY, PCOMPRESSED_DATA_INFO, PDEVICE_OBJECT, PDRIVER_OBJECT, PFAST_IO_DISPATCH, PFILE_BASIC_INFORMATION, PFILE_NETWORK_OPEN_INFORMATION, PFILE_OBJECT, PFILE_STANDARD_INFORMATION, PFS_FILTER_CALLBACK_DATA, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER, PMDL, PUNICODE_STRING, PsGetCurrentProcessId, RtlCompareUnicodeString, RtlCopyUnicodeString, RtlInitUnicodeString, _UNICODE_STRING, PCSTR, CCHAR, _DEVICE_OBJECT, _FAST_IO_DISPATCH, _LIST_ENTRY, _FILE_OBJECT, WORK_QUEUE_ITEM, PWORKER_THREAD_ROUTINE};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// IofCallDriver / IofCompleteRequest are fastcall on i386 (ecx/edx), cdecl on
// amd64/arm64 — import from ddk (bindgen emits the correct per-arch ABI). RtlAssert
// is the crate::glue no-op. DbgPrint is the variadic cdecl ntoskrnl export.
use ntkrnl_sys::ddk::{IofCallDriver, IofCompleteRequest};

// ObfDereferenceObject is fastcall on i386. Declared locally (cfg-gated) rather
// than imported from ddk so the return type stays the crate-local LONG_PTR
// (isize) the call sites expect — ddk's LONG_PTR is c_long (i32) on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfDereferenceObject(Object: PVOID) -> LONG_PTR;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ObfDereferenceObject(Object: PVOID) -> LONG_PTR;
}

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}
// FsRtlRegisterFileSystemFilterCallbacks is an ntoskrnl NTAPI export not emitted
// by bindgen → declare it `extern "system"` (stdcall/`_Name@N` on i386). The
// Fltp* helpers are crate-internal cdecl (def + decl agree), left extern "C".
extern "system" {
    fn FsRtlRegisterFileSystemFilterCallbacks(FilterDriverObject: PDRIVER_OBJECT, Callbacks: PFS_FILTER_CALLBACKS) -> NTSTATUS;
}
extern "C" {
    fn FltpMsgCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn FltpGetBaseDeviceObjectName(DeviceObject: PDEVICE_OBJECT, ObjectName: PUNICODE_STRING) -> NTSTATUS;
    fn FltpGetObjectName(Object: *mut core::ffi::c_void, ObjectName: PUNICODE_STRING) -> NTSTATUS;
    fn FltpFreeUnicodeString(String: PUNICODE_STRING);
    fn FltpSetupCommunicationObjects(DriverObject: PDRIVER_OBJECT) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// C: `DRIVER_DATA DriverData;` — file-scope global, zero-initialized in BSS.
// (c2rust spelled out the FAST_MUTEX/KEVENT zero-init with its own anonymous-union
// type names which don't match the bindgen layout; zeroed() is identical semantics.)
pub static mut DriverData: DRIVER_DATA = unsafe { core::mem::zeroed() };

pub static mut CommsDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
pub static mut FilterList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut FilterListLock: ERESOURCE = unsafe { core::mem::zeroed() };

#[repr(C)]
pub struct _DETACH_DEVICE_WORK_ITEM {
    pub WorkItem: WORK_QUEUE_ITEM,
    pub SourceDevice: PDEVICE_OBJECT,
    pub TargetDevice: PDEVICE_OBJECT,
}
pub type DETACH_DEVICE_WORK_ITEM = _DETACH_DEVICE_WORK_ITEM;
pub type PDETACH_DEVICE_WORK_ITEM = *mut _DETACH_DEVICE_WORK_ITEM;

const APC_LEVEL: i32 = 1;

const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 0x00000008;
const FM_TAG_REGISTRY_DATA: u32 = 0x67655266; // 'fReg' or similar
const FM_TAG_DISPATCH_TABLE: u32 = 0x70736944; // 'Disp'
const STATUS_DEVICE_ALREADY_ATTACHED: NTSTATUS = 0xC0000038u32 as NTSTATUS;
unsafe fn FltpCleanupDeviceObject(mut DeviceObject: PDEVICE_OBJECT) {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();
    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if !DeviceExtension.is_null() {
        // cleanup device extension
    }
}
unsafe fn FltpAttachDeviceObject(
    mut SourceDevice: PDEVICE_OBJECT,
    mut TargetDevice: PDEVICE_OBJECT,
    mut AttachedToDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    const APC_LEVEL: i32 = 1;
    const DO_SYSTEM_BOOT_PARTITION: u32 = 0x100;

    let mut Status: NTSTATUS;

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                69 as u32,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    /* Before attaching, copy the flags from the device we're going to attach to */
    if (*SourceDevice).Flags & DO_BUFFERED_IO != 0 {
        (*TargetDevice).Flags |= DO_BUFFERED_IO;
    }
    if (*SourceDevice).Flags & DO_DIRECT_IO != 0 {
        (*TargetDevice).Flags |= DO_DIRECT_IO;
    }
    if (*SourceDevice).Flags & DO_SYSTEM_BOOT_PARTITION != 0 {
        (*TargetDevice).Characteristics |= FILE_DEVICE_SECURE_OPEN;
    }

    /* Attach this device to the top of the driver stack */
    Status = IoAttachDeviceToDeviceStackSafe(SourceDevice, TargetDevice, AttachedToDeviceObject)
        as NTSTATUS;

    return Status;
}
unsafe fn FltpIsAttachedToDevice(
    DeviceObject: PDEVICE_OBJECT,
    AttachedDeviceObject: *mut PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: i32 = 1;

    let mut CurrentDeviceObject: PDEVICE_OBJECT;
    let mut NextDeviceObject: PDEVICE_OBJECT;

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                102,
                core::ptr::null_mut(),
            );
        } else {
        }
    }

    /* Initialize the return pointer */
    if !AttachedDeviceObject.is_null() {
        *AttachedDeviceObject = core::ptr::null_mut();
    }

    /* Start by getting the top level device in the chain */
    CurrentDeviceObject = IoGetAttachedDeviceReference(DeviceObject);

    /* Loop while there are attached devices */
    while !CurrentDeviceObject.is_null() {
        /* Check if this device driver matches ours */
        if (*CurrentDeviceObject).DriverObject == DriverData.DriverObject {
            if (*CurrentDeviceObject).DeviceExtension.is_null() {
                RtlAssert(
                    b"CurrentDeviceObject->DeviceExtension != ((void *)0)\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                    b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                    116,
                    core::ptr::null_mut(),
                );
            } else {
            }

            /* We're attached, return the device object if the caller asked for it */
            if !AttachedDeviceObject.is_null() {
                *AttachedDeviceObject = CurrentDeviceObject;
            } else {
                /* We aren't returning the reference, so decrement the count */
                ObDereferenceObject(CurrentDeviceObject as *mut core::ffi::c_void);
            }

            return 1;
        }

        /* Get the next device in the chain */
        NextDeviceObject = IoGetLowerDeviceObject(CurrentDeviceObject);

        /* Decrement the count on the last device before we update the pointer */
        ObDereferenceObject(CurrentDeviceObject as *mut core::ffi::c_void);
        CurrentDeviceObject = NextDeviceObject;
    }

    return 0;
}
unsafe fn FltpEnumerateFileSystemVolumes(mut DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {

    const APC_LEVEL: i32 = 1;
    const STATUS_DEVICE_ALREADY_ATTACHED: NTSTATUS = 0xC0000038u32 as i32;
    const FM_TAG_DEV_OBJ_PTRS: u32 = 0x6F4D6C46; // 'FlMo' style tag

    let mut NewDeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut BaseDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut NewDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceList: *mut PDEVICE_OBJECT = core::ptr::null_mut();
    let mut StorageStackDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut NumDevices: ULONG = 0;
    let mut i: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                158 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    BaseDeviceObject = IoGetDeviceAttachmentBaseRef(DeviceObject);

    Status = IoEnumerateDeviceObjectList(
        (*BaseDeviceObject).DriverObject as PDRIVER_OBJECT,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(NumDevices),
    ) as NTSTATUS;
    if Status != STATUS_BUFFER_TOO_SMALL {
        return Status;
    }

    NumDevices = (NumDevices as core::ffi::c_ulong).wrapping_add(4 as core::ffi::c_ulong) as ULONG;

    DeviceList = ExAllocatePoolWithTag(
        NonPagedPool,
        (NumDevices as usize).wrapping_mul(core::mem::size_of::<PDEVICE_OBJECT>()),
        FM_TAG_DEV_OBJ_PTRS as ULONG,
    ) as *mut PDEVICE_OBJECT;
    if DeviceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = IoEnumerateDeviceObjectList(
        (*BaseDeviceObject).DriverObject as PDRIVER_OBJECT,
        DeviceList,
        (NumDevices as usize).wrapping_mul(core::mem::size_of::<PDEVICE_OBJECT>()) as ULONG,
        core::ptr::addr_of_mut!(NumDevices),
    ) as NTSTATUS;
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(DeviceList as *mut core::ffi::c_void, FM_TAG_DEV_OBJ_PTRS as ULONG);
        return Status;
    }

    i = 0 as ULONG;
    while i < NumDevices {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceName), core::ptr::null());
        StorageStackDeviceObject = core::ptr::null_mut();
        NewDeviceObject = core::ptr::null_mut();

        if !(*DeviceList.offset(i as isize) == BaseDeviceObject
            || (**DeviceList.offset(i as isize)).DeviceType != (*BaseDeviceObject).DeviceType)
        {
            if !(FltpIsAttachedToDevice(
                *DeviceList.offset(i as isize),
                core::ptr::null_mut(),
            ) as i32
                == 0)
            {
                FltpGetBaseDeviceObjectName(*DeviceList.offset(i as isize), core::ptr::addr_of_mut!(DeviceName));
                if !(NT_SUCCESS(Status) && DeviceName.Length as i32 > 0 as i32) {
                    Status = IoGetDiskDeviceObject(
                        *DeviceList.offset(i as isize),
                        core::ptr::addr_of_mut!(StorageStackDeviceObject),
                    ) as NTSTATUS;
                    if NT_SUCCESS(Status) {
                        Status = IoCreateDevice(
                            DriverData.DriverObject,
                            core::mem::size_of::<FLTMGR_DEVICE_EXTENSION>() as ULONG,
                            core::ptr::null_mut(),
                            (**DeviceList.offset(i as isize)).DeviceType,
                            0 as ULONG,
                            0 as BOOLEAN,
                            core::ptr::addr_of_mut!(NewDeviceObject),
                        ) as NTSTATUS;
                        if NT_SUCCESS(Status) {
                            NewDeviceExtension =
                                (*NewDeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
                            (*NewDeviceExtension).StorageStackDeviceObject = StorageStackDeviceObject;
                            RtlInitEmptyUnicodeString(
                                core::ptr::addr_of_mut!((*NewDeviceExtension).DeviceName),
                                core::ptr::addr_of_mut!((*NewDeviceExtension).DeviceNameBuffer) as *mut u16,
                                core::mem::size_of::<[u16; 64]>() as u16,
                            );
                            FltpGetObjectName(
                                StorageStackDeviceObject as *mut core::ffi::c_void,
                                core::ptr::addr_of_mut!((*NewDeviceExtension).DeviceName),
                            );
                            ExAcquireFastMutex(core::ptr::addr_of_mut!(DriverData.FilterAttachLock));
                            if FltpIsAttachedToDevice(
                                *DeviceList.offset(i as isize),
                                core::ptr::null_mut(),
                            ) as i32
                                == 0
                            {
                                if !((*NewDeviceObject).DriverObject == DriverData.DriverObject) {
                                    RtlAssert(
                                        b"NewDeviceObject->DriverObject == DriverData.DriverObject\0"
                                            as *const u8 as *const i8 as *mut core::ffi::c_void,
                                        b"drivers/filters/fltmgr/Interface.c\0"
                                            as *const u8 as *const i8 as *mut core::ffi::c_void,
                                        270 as ULONG,
                                        core::ptr::null_mut(),
                                    );
                                } else {
                                };
                                Status = FltpAttachDeviceObject(
                                    *DeviceList.offset(i as isize),
                                    NewDeviceObject,
                                    core::ptr::addr_of_mut!((*NewDeviceExtension).AttachedToDeviceObject),
                                );
                                if NT_SUCCESS(Status) {
                                    (*NewDeviceObject).Flags &= !(0x80 as i32) as core::ffi::c_ulong;
                                }
                            } else {
                                Status = STATUS_DEVICE_ALREADY_ATTACHED;
                            }
                            ExReleaseFastMutex(core::ptr::addr_of_mut!(DriverData.FilterAttachLock));
                        }
                    }
                }
            }
        }

        if !NT_SUCCESS(Status) {
            if !NewDeviceObject.is_null() {
                FltpCleanupDeviceObject(NewDeviceObject);
                IoDeleteDevice(NewDeviceObject);
            }
        }

        if !StorageStackDeviceObject.is_null() {
            ObfDereferenceObject(StorageStackDeviceObject as *mut core::ffi::c_void);
        }

        ObfDereferenceObject(*DeviceList.offset(i as isize) as *mut core::ffi::c_void);

        FltpFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));

        i = i.wrapping_add(1);
    }

    ExFreePoolWithTag(DeviceList as *mut core::ffi::c_void, FM_TAG_DEV_OBJ_PTRS as ULONG);

    return STATUS_SUCCESS;
}
unsafe fn FltpAttachToFileSystemDevice(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceName: PUNICODE_STRING,
) -> NTSTATUS {

    const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 0x00000008;
    const FILE_DEVICE_CD_ROM_FILE_SYSTEM: u32 = 0x00000003;
    const FILE_DEVICE_NETWORK_FILE_SYSTEM: u32 = 0x00000014;
    const MAX_DEVNAME_LENGTH: usize = 64;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut NewDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Buffer: [u16; MAX_DEVNAME_LENGTH] = [0; MAX_DEVNAME_LENGTH];
    let mut FileSystemDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FsRecDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    /* Only handle device types we're interested in */
    if (*DeviceObject).DeviceType != FILE_DEVICE_DISK_FILE_SYSTEM
        && (*DeviceObject).DeviceType != FILE_DEVICE_CD_ROM_FILE_SYSTEM
        && (*DeviceObject).DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM
    {
        return STATUS_SUCCESS;
    }

    /* Setup the buffer to hold the device name */
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(FileSystemDeviceName),
        core::ptr::addr_of_mut!(Buffer) as *mut u16,
        (MAX_DEVNAME_LENGTH * core::mem::size_of::<u16>()) as u16,
    );

    /* Get the the name of the file system device */
    Status = FltpGetObjectName(
        (*DeviceObject).DriverObject as PVOID,
        core::ptr::addr_of_mut!(FileSystemDeviceName),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Build up the name of the file system recognizer device */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(FsRecDeviceName),
        core::mem::transmute::<&[u16; 19], *const u16>(&[
            0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
            0x006D, 0x005C, 0x0046, 0x0073, 0x005F, 0x0052, 0x0065, 0x0063, 0x0000,
        ]),
    );

    /* We don't attach to recognizer devices, so bail if this is one */
    if RtlCompareUnicodeString(
        core::ptr::addr_of_mut!(FileSystemDeviceName),
        core::ptr::addr_of_mut!(FsRecDeviceName),
        1,
    ) == 0
    {
        return STATUS_SUCCESS;
    }

    /* Create a device object which we can attach to this file system */
    Status = IoCreateDevice(
        DriverData.DriverObject,
        core::mem::size_of::<FLTMGR_DEVICE_EXTENSION>() as u32,
        core::ptr::null_mut(),
        (*DeviceObject).DeviceType,
        0,
        0,
        core::ptr::addr_of_mut!(NewDeviceObject),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Cast the device extension to something we understand */
    DeviceExtension = (*NewDeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    /* Attach this device to the top of the driver stack and store the DO we attached to in the DE */
    Status = FltpAttachDeviceObject(
        NewDeviceObject,
        DeviceObject,
        core::ptr::addr_of_mut!((*DeviceExtension).AttachedToDeviceObject),
    );
    if NT_SUCCESS(Status) {
        /* DPRINT("Attached to %wZ\n", &FileSystemDeviceName); */

        /* Setup the unicode string buffer and copy the device name to the device extension */
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceName),
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceNameBuffer) as *mut u16,
            (MAX_DEVNAME_LENGTH * core::mem::size_of::<u16>()) as u16,
        );
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceName),
            DeviceName,
        );

        /* We're done, remove the initializing flag */
        (*NewDeviceObject).Flags &= !(0x80u32);

        /* Look for existing mounted devices for this file system */
        Status = FltpEnumerateFileSystemVolumes(DeviceObject);
        if !NT_SUCCESS(Status) {
            IoDetachDevice((*DeviceExtension).AttachedToDeviceObject);
        }
    } else {
        /* DPRINT1("Failed to attach to the driver stack : 0x%X\n", Status); */
    }

    // Cleanup:
    if !NT_SUCCESS(Status) {
        IoDeleteDevice(NewDeviceObject);
    }

    return Status;
}
unsafe fn FltpDetachFromFileSystemDevice(mut DeviceObject: PDEVICE_OBJECT) -> LONG_PTR {
    let mut AttachedDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut NextDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Count: LONG_PTR = 0;

    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                431 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* Get the top device in the chain and increment the ref count on it */
    AttachedDevice = IoGetAttachedDeviceReference(DeviceObject);

    /* Loop all attached devices looking for our file system driver */
    while (*AttachedDevice).DriverObject != DriverData.DriverObject {
        if AttachedDevice.is_null() {
            RtlAssert(
                b"AttachedDevice != ((void *)0)\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                439 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };

        /* Get the next lower device object. This adds a ref on NextDevice */
        NextDevice = IoGetLowerDeviceObject(AttachedDevice);

        /* Remove the reference we added */
        Count = ObfDereferenceObject(AttachedDevice as PVOID);

        /* Bail if this is the last one */
        if NextDevice.is_null() {
            return Count;
        }

        /* Try the next one */
        AttachedDevice = NextDevice;
    }

    DeviceExtension = (*AttachedDevice).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    /* Detach the device from the chain and delete the object */
    IoDetachDevice(DeviceObject);
    IoDeleteDevice(AttachedDevice);

    /* Remove the reference we added so the delete can complete */
    return ObfDereferenceObject(AttachedDevice as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn FltpPreFsFilterOperation(
    Data: PFS_FILTER_CALLBACK_DATA,
    CompletionContext: *mut PVOID,
) -> NTSTATUS {
    let _ = Data;
    let _ = CompletionContext;
    __debugbreak();
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FltpPostFsFilterOperation(
    mut Data: PFS_FILTER_CALLBACK_DATA,
    mut OperationStatus: NTSTATUS,
    mut CompletionContext: PVOID,
) {
    let _ = Data;
    let _ = OperationStatus;
    let _ = CompletionContext;
    __debugbreak();
}

#[no_mangle]
pub unsafe extern "system" fn FltpDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut StackPtr: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    if DeviceObject == DriverData.DeviceObject {
        if !((*DeviceObject).DriverObject == DriverData.DriverObject) {
            RtlAssert(
                b"DeviceObject->DriverObject == DriverData.DriverObject\0".as_ptr() as PVOID,
                b"Interface.c\0".as_ptr() as PVOID,
                511 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
        if !DeviceExtension.is_null() {
            RtlAssert(
                b"DeviceExtension == ((void *)0)\0".as_ptr() as PVOID,
                b"Interface.c\0".as_ptr() as PVOID,
                512 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }

        Status = FltpDispatchHandler(DeviceObject, Irp);
        if Status != STATUS_REPARSE {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            (*Irp).IoStatus.Information = 0;
            IofCompleteRequest(Irp, 0 as CCHAR);
        }
        return Status;
    }

    if DeviceObject == CommsDeviceObject {
        return FltpMsgDispatch(DeviceObject, Irp);
    }

    if !(!DeviceExtension.is_null() && !(*DeviceExtension).AttachedToDeviceObject.is_null()) {
        RtlAssert(
            b"DeviceExtension && DeviceExtension->AttachedToDeviceObject\0".as_ptr() as PVOID,
            b"Interface.c\0".as_ptr() as PVOID,
            533 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    StackPtr = IoGetCurrentIrpStackLocation(Irp);
    let _ = (*StackPtr).MajorFunction as core::ffi::c_int == IRP_MJ_SHUTDOWN as core::ffi::c_int;

    if DbgPrint(
        b"(%s:%d) Received %X from %wZ\n\0".as_ptr() as PCSTR,
        b"Interface.c\0".as_ptr() as *const core::ffi::c_char,
        541 as core::ffi::c_int,
        (*StackPtr).MajorFunction as core::ffi::c_int,
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceName),
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"Interface.c\0".as_ptr() as *const core::ffi::c_char,
            541 as core::ffi::c_int,
        );
    }

    IoSkipCurrentIrpStackLocation(Irp);
    return IofCallDriver((*DeviceExtension).AttachedToDeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "system" fn FltpCreate(mut DeviceObject: PDEVICE_OBJECT, mut Irp: PIRP) -> NTSTATUS {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                556 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    if DeviceObject == DriverData.DeviceObject {
        if !((*DeviceObject).DriverObject == DriverData.DriverObject) {
            RtlAssert(
                b"DeviceObject->DriverObject == DriverData.DriverObject\0" as *const u8
                    as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                563 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
        if !DeviceExtension.is_null() {
            RtlAssert(
                b"DeviceExtension == ((void *)0)\0" as *const u8 as *const ::core::ffi::c_char
                    as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                564 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS as NTSTATUS;
        (*Irp).IoStatus.Information = 0 as _;
        IofCompleteRequest(Irp, 0 as CCHAR);
        return STATUS_SUCCESS;
    }

    if DeviceObject == CommsDeviceObject {
        return FltpMsgCreate(DeviceObject, Irp);
    }

    if !(!DeviceExtension.is_null() && !(*DeviceExtension).AttachedToDeviceObject.is_null()) {
        RtlAssert(
            b"DeviceExtension && DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            581 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if DbgPrint(
        b"(%s:%d) Received create from %wZ (%lu)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/filters/fltmgr/Interface.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        583 as ::core::ffi::c_int,
        ::core::ptr::addr_of_mut!((*DeviceExtension).DeviceName),
        PsGetCurrentProcessId(),
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            583 as ::core::ffi::c_int,
        );
    }

    IoSkipCurrentIrpStackLocation(Irp);
    return IofCallDriver((*DeviceExtension).AttachedToDeviceObject, Irp) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFsControl(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as PVOID,
                598 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if DeviceObject == DriverData.DeviceObject {
        (*Irp).IoStatus.Information = 0 as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        IofCompleteRequest(Irp, 0 as CCHAR);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    if !(!DeviceExtension.is_null() && !(*DeviceExtension).AttachedToDeviceObject.is_null()) {
        RtlAssert(
            b"DeviceExtension && DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as PVOID,
            613 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*DeviceExtension).AttachedToDeviceObject, Irp) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn FltpDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let Status: NTSTATUS;

    /* Check if the request was meant for us */
    if DeviceObject == DriverData.DeviceObject {
        Status = FltpDeviceControlHandler(DeviceObject, Irp);
        if Status != STATUS_REPARSE {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, 0);
        }

        return Status;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;

    FLT_ASSERT(
        (!DeviceExtension.is_null() && !(*DeviceExtension).AttachedToDeviceObject.is_null()) as i32,
    );

    /* Just pass the IRP down the stack */
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver((*DeviceExtension).AttachedToDeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoCheckIfPossible(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut Wait: BOOLEAN,
    mut LockKey: ULONG,
    mut CheckForReadOperation: BOOLEAN,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut::<_DEVICE_OBJECT>();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = ::core::ptr::null_mut::<_FAST_IO_DISPATCH>();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                673 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            685 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoCheckIfPossible.is_some() {
        return (*FastIoDispatch)
            .FastIoCheckIfPossible
            .expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            Wait,
            LockKey,
            CheckForReadOperation,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            706 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoRead(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut Wait: BOOLEAN,
    mut LockKey: ULONG,
    mut Buffer: PVOID,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as PVOID,
                726 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8 as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as PVOID,
            738 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoRead.is_some() {
        return (*FastIoDispatch).FastIoRead.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            Wait,
            LockKey,
            Buffer,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as PVOID,
            759 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoWrite(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut Wait: BOOLEAN,
    mut LockKey: ULONG,
    mut Buffer: PVOID,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut::<_DEVICE_OBJECT>();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = ::core::ptr::null_mut::<_FAST_IO_DISPATCH>();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                779 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            791 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoWrite.is_some() {
        return (*FastIoDispatch)
            .FastIoWrite
            .expect("non-null function pointer")(
            FileObject as *mut _FILE_OBJECT,
            FileOffset,
            Length,
            Wait,
            LockKey,
            Buffer,
            IoStatus,
            AttachedDeviceObject as *mut _DEVICE_OBJECT,
        );
    }

    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            812 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoQueryBasicInfo(
    mut FileObject: PFILE_OBJECT,
    mut Wait: BOOLEAN,
    mut Buffer: PFILE_BASIC_INFORMATION,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                829u32,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the call */
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        (*IoStatus).Information = 0;
        return 1;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            841u32,
            core::ptr::null_mut(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoQueryBasicInfo.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoQueryBasicInfo.expect("non-null function pointer")(
            FileObject,
            Wait,
            Buffer,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            859u32,
            core::ptr::null_mut(),
        );
    } else {
    };
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoQueryStandardInfo(
    FileObject: PFILE_OBJECT,
    Wait: BOOLEAN,
    Buffer: PFILE_STANDARD_INFORMATION,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                876 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            888 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoQueryStandardInfo.is_some() {
        return (*FastIoDispatch).FastIoQueryStandardInfo.expect("non-null function pointer")(
            FileObject,
            Wait,
            Buffer,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            906 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    return 0 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn FltpFastIoLock(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: PLARGE_INTEGER,
    mut ProcessId: PEPROCESS,
    mut Key: ULONG,
    mut FailImmediately: BOOLEAN,
    mut ExclusiveLock: BOOLEAN,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = ::core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = ::core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    .as_ptr() as *const ::core::ffi::c_char as PVOID,
                927 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the call */
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            939 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoLock.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoLock.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            ProcessId as *mut _,
            Key,
            FailImmediately,
            ExclusiveLock,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            961 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoUnlockSingle(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                980 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        }
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        (*IoStatus).Information = 0;
        return 1;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            992 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    }

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoUnlockSingle.is_some() {
        return (*FastIoDispatch).FastIoUnlockSingle.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            ProcessId as *mut _,
            Key,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1012 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoUnlockAll(
    FileObject: PFILE_OBJECT,
    ProcessId: PEPROCESS,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT;
    let mut FastIoDispatch: PFAST_IO_DISPATCH;

    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                1029 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the call */
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return TRUE as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            1041 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoUnlockAll.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoUnlockAll.expect("non-null function pointer")(
            FileObject,
            ProcessId as *mut _,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            1058 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return FALSE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoUnlockAllByKey(
    FileObject: PFILE_OBJECT,
    ProcessId: PVOID,
    Key: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
                1075 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the call */
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
            1087 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoUnlockAllByKey.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoUnlockAllByKey.expect("non-null function pointer")(
            FileObject,
            ProcessId,
            Key,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
            1105 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoDeviceControl(
    mut FileObject: PFILE_OBJECT,
    mut Wait: BOOLEAN,
    mut InputBuffer: PVOID,
    mut InputBufferLength: ULONG,
    mut OutputBuffer: PVOID,
    mut OutputBufferLength: ULONG,
    mut IoControlCode: ULONG,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: i32 = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
                1126 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the request, send it down the slow path */
        return 0 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
            1136 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoDeviceControl.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoDeviceControl.expect("non-null function pointer")(
            FileObject,
            Wait,
            InputBuffer,
            InputBufferLength,
            OutputBuffer,
            OutputBufferLength,
            IoControlCode,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
            1158 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoDetachDeviceWorker(mut Parameter: PVOID) {
    let mut DetachDeviceWorkItem: PDETACH_DEVICE_WORK_ITEM = Parameter as PDETACH_DEVICE_WORK_ITEM;

    /* Run any cleanup routines */
    FltpCleanupDeviceObject((*DetachDeviceWorkItem).SourceDevice);

    /* Detach from the target device */
    IoDetachDevice((*DetachDeviceWorkItem).TargetDevice);

    /* Delete the source */
    IoDeleteDevice((*DetachDeviceWorkItem).SourceDevice);

    /* Free the pool we allocated in FltpFastIoDetachDevice */
    ExFreePoolWithTag(DetachDeviceWorkItem as PVOID, 0x1234 as ULONG);
}
#[no_mangle]
pub unsafe extern "system" fn FltpFastIoDetachDevice(
    mut SourceDevice: PDEVICE_OBJECT,
    mut TargetDevice: PDEVICE_OBJECT,
) {
    const APC_LEVEL: i32 = 1;

    let mut DetachDeviceWorkItem: PDETACH_DEVICE_WORK_ITEM =
        core::ptr::null_mut::<_DETACH_DEVICE_WORK_ITEM>();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as PVOID,
                1189 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    DetachDeviceWorkItem = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DETACH_DEVICE_WORK_ITEM>() as SIZE_T,
        0x1234 as ULONG,
    ) as PDETACH_DEVICE_WORK_ITEM;
    if !DetachDeviceWorkItem.is_null() {
        (*DetachDeviceWorkItem).WorkItem.WorkerRoutine =
            core::mem::transmute::<_, PWORKER_THREAD_ROUTINE>(
                FltpFastIoDetachDeviceWorker as unsafe extern "system" fn(PVOID) -> (),
            );
        core::ptr::write_volatile(
            &mut (*DetachDeviceWorkItem).WorkItem.Parameter as *mut PVOID,
            DetachDeviceWorkItem as PVOID,
        );
        (*DetachDeviceWorkItem).WorkItem.List.Flink = core::ptr::null_mut::<_LIST_ENTRY>();
        ExQueueWorkItem(&raw mut (*DetachDeviceWorkItem).WorkItem, DelayedWorkQueue);
    } else {
        FltpCleanupDeviceObject(SourceDevice);
        IoDetachDevice(TargetDevice);
        IoDeleteDevice(SourceDevice);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoQueryNetworkOpenInfo(
    mut FileObject: PFILE_OBJECT,
    mut Wait: BOOLEAN,
    mut Buffer: PFILE_NETWORK_OPEN_INFORMATION,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_FLTMGR_DEVICE_EXTENSION>();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = ::core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                1235 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1247 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoQueryNetworkOpenInfo.is_some() {
        return (*FastIoDispatch)
            .FastIoQueryNetworkOpenInfo
            .expect("non-null function pointer")(
            FileObject,
            Wait,
            Buffer,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1265 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoMdlRead(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut LockKey: ULONG,
    mut MdlChain: *mut PMDL,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                1284 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the call */
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1296 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).MdlRead.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).MdlRead.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            LockKey,
            MdlChain,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1316 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn FltpFastIoMdlReadComplete(
    mut FileObject: PFILE_OBJECT,
    mut MdlChain: PMDL,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: i32 = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *mut core::ffi::c_void,
                1332 as u32,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the request, send it down the slow path */
        return 0;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *mut core::ffi::c_void,
            1342 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).MdlReadComplete.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).MdlReadComplete.expect("non-null function pointer")(
            FileObject,
            MdlChain,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *mut core::ffi::c_void,
            1358 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoPrepareMdlWrite(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut LockKey: ULONG,
    mut MdlChain: *mut PMDL,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                1377 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST as NTSTATUS;
        (*IoStatus).Information = 0 as _;
        return 1 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            1389 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).PrepareMdlWrite.is_some() {
        return (*FastIoDispatch).PrepareMdlWrite.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            LockKey,
            MdlChain,
            IoStatus,
            AttachedDeviceObject,
        );
    }

    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            1409 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoMdlWriteComplete(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut MdlChain: PMDL,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: core::ffi::c_int = 1;
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();
    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                1425 as u32,
                core::ptr::null_mut(),
            );
        } else {
        };
    }
    if (*DeviceObject).DeviceExtension.is_null() {
        return 0;
    }
    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1435 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;
    if !FastIoDispatch.is_null() && (*FastIoDispatch).MdlWriteComplete.is_some() {
        return (*FastIoDispatch).MdlWriteComplete.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            MdlChain,
            AttachedDeviceObject,
        );
    }
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1452 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoReadCompressed(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    LockKey: ULONG,
    Buffer: PVOID,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
    CompressedDataInfo: PCOMPRESSED_DATA_INFO,
    CompressedDataInfoLength: ULONG,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as PVOID,
                1474 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the request, send it down the slow path */
        return 0 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8 as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as PVOID,
            1484 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoReadCompressed.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).FastIoReadCompressed.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            LockKey,
            Buffer,
            MdlChain,
            IoStatus,
            CompressedDataInfo,
            CompressedDataInfoLength,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as PVOID,
            1507 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoWriteCompressed(
    mut FileObject: PFILE_OBJECT,
    mut FileOffset: PLARGE_INTEGER,
    mut Length: ULONG,
    mut LockKey: ULONG,
    mut Buffer: PVOID,
    mut MdlChain: *mut PMDL,
    mut IoStatus: PIO_STATUS_BLOCK,
    mut CompressedDataInfo: PCOMPRESSED_DATA_INFO,
    mut CompressedDataInfoLength: ULONG,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION =
        ::core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = ::core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = ::core::ptr::null_mut();

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                1529 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        /* Fail the request, send it down the slow path */
        return 0 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1539 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoWriteCompressed.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch)
            .FastIoWriteCompressed
            .expect("non-null function pointer")(
            FileObject,
            FileOffset,
            Length,
            LockKey,
            Buffer,
            MdlChain,
            IoStatus,
            CompressedDataInfo,
            CompressedDataInfoLength,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1562 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoMdlReadCompleteCompressed(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    const APC_LEVEL: i32 = 1;

    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                1577 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        }
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        return 0;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1586 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    }

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).MdlReadCompleteCompressed.is_some() {
        /* Forward the call onto the device we attached to */
        return (*FastIoDispatch).MdlReadCompleteCompressed.expect("non-null function pointer")(
            FileObject,
            MdlChain,
            AttachedDeviceObject,
        );
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 == 0 {
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1602 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoMdlWriteCompleteCompressed(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                1618 as u32,
                core::ptr::null_mut(),
            );
        } else {
        };
    }

    if (*DeviceObject).DeviceExtension.is_null() {
        return 0;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            1627 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };

    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    if !FastIoDispatch.is_null() && (*FastIoDispatch).MdlWriteCompleteCompressed.is_some() {
        return (*FastIoDispatch).MdlWriteCompleteCompressed.expect("non-null function pointer")(
            FileObject,
            FileOffset,
            MdlChain,
            AttachedDeviceObject,
        );
    }

    if 0 == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            1644 as u32,
            core::ptr::null_mut(),
        );
    } else {
    };
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn FltpFastIoQueryOpen(
    mut Irp: PIRP,
    mut NetworkInformation: PFILE_NETWORK_OPEN_INFORMATION,
    mut DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut DeviceExtension: PFLTMGR_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut AttachedDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FastIoDispatch: PFAST_IO_DISPATCH = core::ptr::null_mut();
    let mut Success: BOOLEAN = 0;

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Interface.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                1660 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* If it doesn't have a device extension, then it's not our device object */
    if (*DeviceObject).DeviceExtension.is_null() {
        return 0 as BOOLEAN;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFLTMGR_DEVICE_EXTENSION;
    if (*DeviceExtension).AttachedToDeviceObject.is_null() {
        RtlAssert(
            b"DeviceExtension->AttachedToDeviceObject\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1669 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    /* Get the device that we attached to */
    AttachedDeviceObject = (*DeviceExtension).AttachedToDeviceObject;
    FastIoDispatch = (*(*AttachedDeviceObject).DriverObject).FastIoDispatch as PFAST_IO_DISPATCH;

    /* Make sure our FastIo table is valid */
    if !FastIoDispatch.is_null() && (*FastIoDispatch).FastIoQueryOpen.is_some() {
        let mut StackPtr: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

        /* Update the stack to contain the correct device for the next filter */
        (*StackPtr).DeviceObject = AttachedDeviceObject;

        /* Now forward the call */
        Success = (*FastIoDispatch)
            .FastIoQueryOpen
            .expect("non-null function pointer")(
            Irp,
            NetworkInformation,
            AttachedDeviceObject,
        );

        /* Restore the DeviceObject as we found it */
        (*StackPtr).DeviceObject = DeviceObject;
        return Success;
    }

    /* We failed to handle the request, send it down the slow path */
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Interface.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            1694 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return 0 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn FltpFsNotification(mut DeviceObject: PDEVICE_OBJECT, mut FsActive: BOOLEAN) {
    const APC_LEVEL: core::ffi::c_int = 1;

    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null::<u16>() as *mut u16,
    };
    let mut Status: NTSTATUS;

    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filters/fltmgr/Interface.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                1710 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }

    /* Set an empty string */
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceName), core::ptr::null::<u16>());

    /* Get the name of the lowest device object on the stack */
    Status = FltpGetBaseDeviceObjectName(DeviceObject, core::ptr::addr_of_mut!(DeviceName));
    if Status >= 0 as core::ffi::c_long {
        /* Check if it's attaching or detaching */
        if FsActive != 0 {
            /* Run the attach routine */
            FltpAttachToFileSystemDevice(DeviceObject, core::ptr::addr_of_mut!(DeviceName));
        } else {
            /* Run the detach routine */
            FltpDetachFromFileSystemDevice(DeviceObject);
        }

        /* Free the buffer which FltpGetBaseDeviceObjectName allocated */
        FltpFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetupDispatchAndCallbacksTables(DriverObject: PDRIVER_OBJECT) -> NTSTATUS {
    let mut FastIoDispatch: PFAST_IO_DISPATCH;
    let mut Callbacks: FS_FILTER_CALLBACKS = core::mem::zeroed();
    let mut i: ULONG;

    /* Plug all the IRPs */
    i = 0 as ULONG;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as ULONG {
        (*DriverObject).MajorFunction[i as usize] =
            core::mem::transmute(FltpDispatch as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        i = i.wrapping_add(1);
    }

    /* Override the ones we're interested in */
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute(FltpCreate as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_NAMED_PIPE as usize] =
        core::mem::transmute(FltpCreate as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_MAILSLOT as usize] =
        core::mem::transmute(FltpCreate as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL as usize] =
        core::mem::transmute(FltpFsControl as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        core::mem::transmute(FltpDeviceControl as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);

    /* The FastIo dispatch table is stored in the pool along with a tag */
    FastIoDispatch = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<FAST_IO_DISPATCH>() as SIZE_T,
        FM_TAG_DISPATCH_TABLE as ULONG,
    ) as PFAST_IO_DISPATCH;
    if FastIoDispatch.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Fill out the FastIo table */
    RtlZeroMemory(
        FastIoDispatch as *mut core::ffi::c_void,
        core::mem::size_of::<FAST_IO_DISPATCH>(),
    );
    (*FastIoDispatch).SizeOfFastIoDispatch = core::mem::size_of::<FAST_IO_DISPATCH>() as ULONG;
    (*FastIoDispatch).FastIoCheckIfPossible = core::mem::transmute(FltpFastIoCheckIfPossible as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                BOOLEAN,
                ULONG,
                BOOLEAN,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoRead = core::mem::transmute(FltpFastIoRead as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                BOOLEAN,
                ULONG,
                PVOID,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoWrite = core::mem::transmute(FltpFastIoWrite as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                BOOLEAN,
                ULONG,
                PVOID,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoQueryBasicInfo = core::mem::transmute(FltpFastIoQueryBasicInfo as unsafe extern "system" fn(
                PFILE_OBJECT,
                BOOLEAN,
                PFILE_BASIC_INFORMATION,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoQueryStandardInfo = core::mem::transmute(FltpFastIoQueryStandardInfo as unsafe extern "system" fn(
                PFILE_OBJECT,
                BOOLEAN,
                PFILE_STANDARD_INFORMATION,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoLock = core::mem::transmute(FltpFastIoLock as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                PLARGE_INTEGER,
                PEPROCESS,
                ULONG,
                BOOLEAN,
                BOOLEAN,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoUnlockSingle = core::mem::transmute(FltpFastIoUnlockSingle as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                PLARGE_INTEGER,
                PEPROCESS,
                ULONG,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoUnlockAll = core::mem::transmute(FltpFastIoUnlockAll as unsafe extern "system" fn(
                PFILE_OBJECT,
                PEPROCESS,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoUnlockAllByKey = core::mem::transmute(FltpFastIoUnlockAllByKey as unsafe extern "system" fn(
                PFILE_OBJECT,
                PVOID,
                ULONG,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoDeviceControl = core::mem::transmute(FltpFastIoDeviceControl as unsafe extern "system" fn(
                PFILE_OBJECT,
                BOOLEAN,
                PVOID,
                ULONG,
                PVOID,
                ULONG,
                ULONG,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoDetachDevice =
        core::mem::transmute(FltpFastIoDetachDevice as unsafe extern "system" fn(PDEVICE_OBJECT, PDEVICE_OBJECT) -> ());
    (*FastIoDispatch).FastIoQueryNetworkOpenInfo = core::mem::transmute(FltpFastIoQueryNetworkOpenInfo as unsafe extern "system" fn(
                PFILE_OBJECT,
                BOOLEAN,
                PFILE_NETWORK_OPEN_INFORMATION,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).MdlRead = core::mem::transmute(FltpFastIoMdlRead as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                ULONG,
                *mut PMDL,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).MdlReadComplete = core::mem::transmute(FltpFastIoMdlReadComplete as unsafe extern "system" fn(PFILE_OBJECT, PMDL, PDEVICE_OBJECT) -> BOOLEAN,
    );
    (*FastIoDispatch).PrepareMdlWrite = core::mem::transmute(FltpFastIoPrepareMdlWrite as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                ULONG,
                *mut PMDL,
                PIO_STATUS_BLOCK,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).MdlWriteComplete = core::mem::transmute(FltpFastIoMdlWriteComplete as unsafe extern "system" fn(PFILE_OBJECT, PLARGE_INTEGER, PMDL, PDEVICE_OBJECT) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoReadCompressed = core::mem::transmute(FltpFastIoReadCompressed as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                ULONG,
                PVOID,
                *mut PMDL,
                PIO_STATUS_BLOCK,
                PCOMPRESSED_DATA_INFO,
                ULONG,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoWriteCompressed = core::mem::transmute(FltpFastIoWriteCompressed as unsafe extern "system" fn(
                PFILE_OBJECT,
                PLARGE_INTEGER,
                ULONG,
                ULONG,
                PVOID,
                *mut PMDL,
                PIO_STATUS_BLOCK,
                PCOMPRESSED_DATA_INFO,
                ULONG,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );
    (*FastIoDispatch).MdlReadCompleteCompressed = core::mem::transmute(FltpFastIoMdlReadCompleteCompressed as unsafe extern "system" fn(PFILE_OBJECT, PMDL, PDEVICE_OBJECT) -> BOOLEAN,
    );
    (*FastIoDispatch).MdlWriteCompleteCompressed = core::mem::transmute(FltpFastIoMdlWriteCompleteCompressed as unsafe extern "system" fn(PFILE_OBJECT, PLARGE_INTEGER, PMDL, PDEVICE_OBJECT) -> BOOLEAN,
    );
    (*FastIoDispatch).FastIoQueryOpen = core::mem::transmute(FltpFastIoQueryOpen as unsafe extern "system" fn(
                PIRP,
                PFILE_NETWORK_OPEN_INFORMATION,
                PDEVICE_OBJECT,
            ) -> BOOLEAN,
    );

    /* Store the FastIo table for internal and our access */
    (*DriverObject).FastIoDispatch = FastIoDispatch;
    DriverData.FastIoDispatch = FastIoDispatch;

    /* Initialize the callback table */
    Callbacks.SizeOfFsFilterCallbacks = core::mem::size_of::<FS_FILTER_CALLBACKS>() as ULONG;
    Callbacks.PreAcquireForSectionSynchronization = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostAcquireForSectionSynchronization = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );
    Callbacks.PreReleaseForSectionSynchronization = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostReleaseForSectionSynchronization = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );
    Callbacks.PreAcquireForCcFlush = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostAcquireForCcFlush = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );
    Callbacks.PreReleaseForCcFlush = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostReleaseForCcFlush = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );
    Callbacks.PreAcquireForModifiedPageWriter = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostAcquireForModifiedPageWriter = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );
    Callbacks.PreReleaseForModifiedPageWriter = core::mem::transmute(FltpPreFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, *mut PVOID) -> NTSTATUS,
    );
    Callbacks.PostReleaseForModifiedPageWriter = core::mem::transmute(FltpPostFsFilterOperation as unsafe extern "system" fn(PFS_FILTER_CALLBACK_DATA, NTSTATUS, PVOID) -> (),
    );

    /* Register our callbacks */
    return FsRtlRegisterFileSystemFilterCallbacks(
        DriverObject,
        core::ptr::addr_of_mut!(Callbacks),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    static DEVICE_NAME_BUF: [u16; 27] = [
        0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065,
        0x006D, 0x005C, 0x0046, 0x0069, 0x006C, 0x0074, 0x0065, 0x0072, 0x0073, 0x005C,
        0x0046, 0x006C, 0x0074, 0x004D, 0x0067, 0x0072, 0x0000,
    ];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 27]>()
            - core::mem::size_of::<u16>()) as u16,
        MaximumLength: (core::mem::size_of::<[u16; 27]>()
            / core::mem::size_of::<u8>()) as u16,
        Buffer: DEVICE_NAME_BUF.as_ptr() as *mut u16,
    };
    let mut RawDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut RawFileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut SymLink: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(DriverData) as *mut core::ffi::c_void,
        core::mem::size_of::<DRIVER_DATA>(),
    );
    DriverData.DriverObject = DriverObject;

    DriverData.ServiceKey.Length = (*RegistryPath).Length;
    DriverData.ServiceKey.MaximumLength = (*RegistryPath).MaximumLength;
    DriverData.ServiceKey.Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (*RegistryPath).MaximumLength as usize,
        FM_TAG_REGISTRY_DATA,
    ) as *mut u16;
    if DriverData.ServiceKey.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!(DriverData.ServiceKey),
        RegistryPath,
    );

    ExInitializeFastMutex(core::ptr::addr_of_mut!(DriverData.FilterAttachLock) as ntkrnl_sys::ddk::PFAST_MUTEX);

    Status = IoCreateDevice(
        DriverObject,
        0,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_DISK_FILE_SYSTEM,
        FILE_DEVICE_SECURE_OPEN,
        0,
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) fltmgr IoCreateDevice failed.  Status = %X\n\0" as *const u8 as *const i8,
            b"Interface.c\0" as *const u8 as *const i8,
            1852,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"Interface.c\0" as *const u8 as *const i8,
                1852,
            );
        }
    } else {
        DriverData.DeviceObject = DeviceObject;

        static SYMLINK_BUF: [u16; 11] = [
            0x005C, 0x003F, 0x003F, 0x005C, 0x0046, 0x006C, 0x0074, 0x004D, 0x0067, 0x0072,
            0x0000,
        ];
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(SymLink),
            SYMLINK_BUF.as_ptr(),
        );
        Status = IoCreateSymbolicLink(core::ptr::addr_of_mut!(SymLink), core::ptr::addr_of_mut!(DeviceName));
        if NT_SUCCESS(Status) {
            Status = SetupDispatchAndCallbacksTables(DriverObject);
            if NT_SUCCESS(Status) {
                Status = FltpSetupCommunicationObjects(DriverObject);
                if NT_SUCCESS(Status) {
                    Status = IoRegisterFsRegistrationChange(
                        DriverObject,
                        core::mem::transmute(FltpFsNotification as unsafe extern "system" fn(_, _)),
                    );
                    if !(Status != STATUS_DEVICE_ALREADY_ATTACHED) {
                        RtlAssert(
                            b"Status != ((NTSTATUS)0xC0000038)\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                            b"Interface.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                            1874,
                            core::ptr::null_mut(),
                        );
                    } else {
                    }
                    if NT_SUCCESS(Status) {
                        InitializeListHead(core::ptr::addr_of_mut!(FilterList));
                        ExInitializeResourceLite(core::ptr::addr_of_mut!(FilterListLock));

                        static RAWDISK_BUF: [u16; 16] = [
                            0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
                            0x0052, 0x0061, 0x0077, 0x0044, 0x0069, 0x0073, 0x006B, 0x0000,
                        ];
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(ObjectName),
                            RAWDISK_BUF.as_ptr(),
                        );
                        Status = IoGetDeviceObjectPointer(
                            core::ptr::addr_of_mut!(ObjectName),
                            FILE_READ_ATTRIBUTES,
                            core::ptr::addr_of_mut!(RawFileObject),
                            core::ptr::addr_of_mut!(RawDeviceObject),
                        );
                        if NT_SUCCESS(Status) {
                            FltpFsNotification(RawDeviceObject, 1);
                            ObDereferenceObject(RawFileObject as *mut core::ffi::c_void);
                        }

                        static RAWCDROM_BUF: [u16; 17] = [
                            0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
                            0x0052, 0x0061, 0x0077, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D,
                            0x0000,
                        ];
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(ObjectName),
                            RAWCDROM_BUF.as_ptr(),
                        );
                        Status = IoGetDeviceObjectPointer(
                            core::ptr::addr_of_mut!(ObjectName),
                            FILE_READ_ATTRIBUTES,
                            core::ptr::addr_of_mut!(RawFileObject),
                            core::ptr::addr_of_mut!(RawDeviceObject),
                        );
                        if NT_SUCCESS(Status) {
                            FltpFsNotification(RawDeviceObject, 1);
                            ObDereferenceObject(RawFileObject as *mut core::ffi::c_void);
                        }

                        (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as u32);
                        Status = STATUS_SUCCESS;
                    }
                }
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !DriverData.FastIoDispatch.is_null() {
            (*DriverObject).FastIoDispatch = core::ptr::null_mut();
            ExFreePoolWithTag(
                DriverData.FastIoDispatch as *mut core::ffi::c_void,
                FM_TAG_DISPATCH_TABLE,
            );
        }
        IoDeleteSymbolicLink(core::ptr::addr_of_mut!(SymLink));
        if !DeviceObject.is_null() {
            IoDeleteDevice(DeviceObject);
        }
        if !DriverData.ServiceKey.Buffer.is_null() {
            ExFreePoolWithTag(
                DriverData.ServiceKey.Buffer as *mut core::ffi::c_void,
                FM_TAG_REGISTRY_DATA,
            );
        }
    }

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/interface.c
// * PURPOSE:         Implements the driver interface
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// //#define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// #define VALID_FAST_IO_DISPATCH_HANDLER(_FastIoDispatchPtr, _FieldName) \
//     (((_FastIoDispatchPtr) != NULL) && \
//      (((_FastIoDispatchPtr)->SizeOfFastIoDispatch) >= \
//             (FIELD_OFFSET(FAST_IO_DISPATCH, _FieldName) + sizeof(void *))) && \
//      ((_FastIoDispatchPtr)->_FieldName != NULL))
//
// #define IS_MY_DEVICE_OBJECT(_devObj) \
//     (((_devObj) != NULL) && \
//     ((_devObj)->DriverObject == Dispatcher::DriverObject) && \
//       ((_devObj)->DeviceExtension != NULL))
//
// extern PDEVICE_OBJECT CommsDeviceObject;
// extern LIST_ENTRY FilterList;
// extern ERESOURCE FilterListLock;
//
// DRIVER_DATA DriverData;
//
// typedef struct _DETACH_DEVICE_WORK_ITEM
// {
//     WORK_QUEUE_ITEM WorkItem;
//     PDEVICE_OBJECT SourceDevice;
//     PDEVICE_OBJECT TargetDevice;
//
// } DETACH_DEVICE_WORK_ITEM, *PDETACH_DEVICE_WORK_ITEM;
//
// /* LOCAL FUNCTIONS ****************************************/
//
// static
// VOID
// FltpCleanupDeviceObject( PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     if (DeviceExtension)
//     {
//         // cleanup device extension
//     }
// }
//
// CODE_SEG("PAGE")
// static
// NTSTATUS
// FltpAttachDeviceObject( PDEVICE_OBJECT SourceDevice,
//                         PDEVICE_OBJECT TargetDevice,
//                         PDEVICE_OBJECT *AttachedToDeviceObject)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Before attaching, copy the flags from the device we're going to attach to */
//     if (FlagOn(SourceDevice->Flags, DO_BUFFERED_IO))
//     {
//         SetFlag(TargetDevice->Flags, DO_BUFFERED_IO);
//     }
//     if (FlagOn(SourceDevice->Flags, DO_DIRECT_IO))
//     {
//         SetFlag(TargetDevice->Flags, DO_DIRECT_IO);
//     }
//     if (FlagOn(SourceDevice->Flags, DO_SYSTEM_BOOT_PARTITION))
//     {
//         SetFlag(TargetDevice->Characteristics, FILE_DEVICE_SECURE_OPEN);
//     }
//
//     /* Attach this device to the top of the driver stack */
//     Status = IoAttachDeviceToDeviceStackSafe(SourceDevice,
//                                              TargetDevice,
//                                              AttachedToDeviceObject);
//
//     return Status;
// }
//
// CODE_SEG("PAGE")
// static
// BOOLEAN
// FltpIsAttachedToDevice( PDEVICE_OBJECT DeviceObject,
//                         PDEVICE_OBJECT *AttachedDeviceObject)
// {
//     PDEVICE_OBJECT CurrentDeviceObject;
//     PDEVICE_OBJECT NextDeviceObject;
//
//     PAGED_CODE();
//
//     /* Initialize the return pointer */
//     if (AttachedDeviceObject) *AttachedDeviceObject = NULL;
//
//     /* Start by getting the top level device in the chain */
//     CurrentDeviceObject = IoGetAttachedDeviceReference(DeviceObject);
//
//     /* Loop while there are attached devices */
//     while (CurrentDeviceObject)
//     {
//         /* Check if this device driver matches ours */
//         if (CurrentDeviceObject->DriverObject == DriverData.DriverObject)
//         {
//             FLT_ASSERT(CurrentDeviceObject->DeviceExtension != NULL);
//
//             /* We're attached, return the device object if the caller asked for it */
//             if (AttachedDeviceObject)
//             {
//                 *AttachedDeviceObject = CurrentDeviceObject;
//             }
//             else
//             {
//                 /* We aren't returning the reference, so decrement the count */
//                 ObDereferenceObject(CurrentDeviceObject);
//             }
//
//             return TRUE;
//         }
//
//         /* Get the next device in the chain */
//         NextDeviceObject = IoGetLowerDeviceObject(CurrentDeviceObject);
//
//         /* Decrement the count on the last device before we update the pointer */
//         ObDereferenceObject(CurrentDeviceObject);
//         CurrentDeviceObject = NextDeviceObject;
//     }
//
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// static
// NTSTATUS
// FltpEnumerateFileSystemVolumes( PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION NewDeviceExtension;
//     PDEVICE_OBJECT BaseDeviceObject;
//     PDEVICE_OBJECT NewDeviceObject;
//     PDEVICE_OBJECT *DeviceList;
//     PDEVICE_OBJECT StorageStackDeviceObject;
//     UNICODE_STRING DeviceName;
//     ULONG NumDevices;
//     ULONG i;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Get the base device */
//     BaseDeviceObject = IoGetDeviceAttachmentBaseRef(DeviceObject);
//
//     /* get the number of device object linked to the base file system */
//     Status = IoEnumerateDeviceObjectList(BaseDeviceObject->DriverObject,
//                                          NULL,
//                                          0,
//                                          &NumDevices);
//     if (Status != STATUS_BUFFER_TOO_SMALL) return Status;
//
//     /* Add a few more slots in case the size changed between calls and allocate some memory to hold the pointers */
//     NumDevices += 4;
//     DeviceList = ExAllocatePoolWithTag(NonPagedPool,
//                                        (NumDevices * sizeof(PDEVICE_OBJECT)),
//                                        FM_TAG_DEV_OBJ_PTRS);
//     if (DeviceList == NULL) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Now get all the device objects that this base driver has created */
//     Status = IoEnumerateDeviceObjectList(BaseDeviceObject->DriverObject,
//                                          DeviceList,
//                                          (NumDevices * sizeof(PDEVICE_OBJECT)),
//                                          &NumDevices);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(DeviceList, FM_TAG_DEV_OBJ_PTRS);
//         return Status;
//     }
//
//     /* Loop through all the devices looking for ones to attach to */
//     for (i = 0; i < NumDevices; i++)
//     {
//         RtlInitUnicodeString(&DeviceName, NULL);
//         StorageStackDeviceObject = NULL;
//         NewDeviceObject = NULL;
//
//         /* Ignore the device we passed in, and devices of the wrong type */
//         if ((DeviceList[i] == BaseDeviceObject) ||
//             (DeviceList[i]->DeviceType != BaseDeviceObject->DeviceType))
//         {
//             goto CleanupAndNext;
//         }
//
//         /* Ignore this device if we're already attached to it */
//         if (FltpIsAttachedToDevice(DeviceList[i], NULL) == FALSE)
//         {
//             goto CleanupAndNext;
//         }
//
//
//         /*
//          * If the device has a name, it must be a control device.
//          * This handles drivers with more then one control device (like FastFat)
//          */
//         FltpGetBaseDeviceObjectName(DeviceList[i], &DeviceName);
//         if (NT_SUCCESS(Status) && DeviceName.Length > 0)
//         {
//             goto CleanupAndNext;
//         }
//
//         /*
//          * Try to get the storage stack (disk) device object associated with
//          * this file system device object. Ignore the device if we don't have one
//          */
//         Status = IoGetDiskDeviceObject(DeviceList[i],
//                                        &StorageStackDeviceObject);
//         if (!NT_SUCCESS(Status))
//         {
//             goto CleanupAndNext;
//         }
//
//
//         /*
//          * TODO: Don't attach to shadow copy volumes,
//          * ros doesn't have any so it's not an issues yet
//          */
//
//         /*
//          * We're far enough to be ready to attach, create a device
//          * object which we'll use to do so
//          */
//         Status = IoCreateDevice(DriverData.DriverObject,
//                                 sizeof(FLTMGR_DEVICE_EXTENSION),
//                                 NULL,
//                                 DeviceList[i]->DeviceType,
//                                 0,
//                                 FALSE,
//                                 &NewDeviceObject);
//         if (!NT_SUCCESS(Status))
//         {
//             goto CleanupAndNext;
//         }
//
//         /* Get the device extension for this new object and store our disk object there */
//         NewDeviceExtension = NewDeviceObject->DeviceExtension;
//         NewDeviceExtension->StorageStackDeviceObject = StorageStackDeviceObject;
//
//         /* Lookup and store the device name for the storage stack */
//         RtlInitEmptyUnicodeString(&NewDeviceExtension->DeviceName,
//                                   NewDeviceExtension->DeviceNameBuffer,
//                                   sizeof(NewDeviceExtension->DeviceNameBuffer));
//         FltpGetObjectName(StorageStackDeviceObject,
//                           &NewDeviceExtension->DeviceName);
//
//
//         /* Grab the attach lock before we attempt to attach */
//         ExAcquireFastMutex(&DriverData.FilterAttachLock);
//
//         /* Check again that we aren't already attached. It may have changed since our last check */
//         if (FltpIsAttachedToDevice(DeviceList[i], NULL) == FALSE)
//         {
//             FLT_ASSERT(NewDeviceObject->DriverObject == DriverData.DriverObject);
//
//             /* Finally, attach to the volume */
//             Status = FltpAttachDeviceObject(DeviceList[i],
//                                             NewDeviceObject,
//                                             &NewDeviceExtension->AttachedToDeviceObject);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Clean the initializing flag so other filters can attach to our device object */
//                 ClearFlag(NewDeviceObject->Flags, DO_DEVICE_INITIALIZING);
//             }
//         }
//         else
//         {
//             /* We're already attached. Just cleanup */
//             Status = STATUS_DEVICE_ALREADY_ATTACHED;
//         }
//
//         ExReleaseFastMutex(&DriverData.FilterAttachLock);
//
// CleanupAndNext:
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (NewDeviceObject)
//             {
//                 FltpCleanupDeviceObject(NewDeviceObject);
//                 IoDeleteDevice(NewDeviceObject);
//             }
//         }
//
//         if (StorageStackDeviceObject)
//         {
//             /* A ref was added for us when we attached, so we can deref ours now */
//             ObDereferenceObject(StorageStackDeviceObject);
//         }
//
//         /* Remove the ref which was added by IoEnumerateDeviceObjectList */
//         ObDereferenceObject(DeviceList[i]);
//
//         /* Free the buffer that FltpGetBaseDeviceObjectName added */
//         FltpFreeUnicodeString(&DeviceName);
//
//     }
//
//     /* Free the memory we allocated for the list */
//     ExFreePoolWithTag(DeviceList, FM_TAG_DEV_OBJ_PTRS);
//
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("PAGE")
// static
// NTSTATUS
// FltpAttachToFileSystemDevice( PDEVICE_OBJECT DeviceObject,
//                               PUNICODE_STRING DeviceName)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT NewDeviceObject;
//     WCHAR Buffer[MAX_DEVNAME_LENGTH];
//     UNICODE_STRING FileSystemDeviceName;
//     UNICODE_STRING FsRecDeviceName;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Only handle device types we're interested in */
//     if (DeviceObject->DeviceType != FILE_DEVICE_DISK_FILE_SYSTEM &&
//         DeviceObject->DeviceType != FILE_DEVICE_CD_ROM_FILE_SYSTEM &&
//         DeviceObject->DeviceType != FILE_DEVICE_NETWORK_FILE_SYSTEM)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Setup the buffer to hold the device name */
//     RtlInitEmptyUnicodeString(&FileSystemDeviceName,
//                               Buffer,
//                               MAX_DEVNAME_LENGTH * sizeof(WCHAR));
//
//     /* Get the the name of the file system device */
//     Status = FltpGetObjectName(DeviceObject->DriverObject, &FileSystemDeviceName);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     DPRINT("Found device %wZ, checking if we need to attach...\n", &FileSystemDeviceName);
//
//     /* Build up the name of the file system recognizer device */
//     RtlInitUnicodeString(&FsRecDeviceName, L"\\FileSystem\\Fs_Rec");
//
//     /* We don't attach to recognizer devices, so bail if this is one */
//     if (RtlCompareUnicodeString(&FileSystemDeviceName, &FsRecDeviceName, TRUE) == 0)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     /* Create a device object which we can attach to this file system */
//     Status = IoCreateDevice(DriverData.DriverObject,
//                             sizeof(FLTMGR_DEVICE_EXTENSION),
//                             NULL,
//                             DeviceObject->DeviceType,
//                             0,
//                             FALSE,
//                             &NewDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create a DO for attaching to a FS : 0x%X\n", Status);
//         return Status;
//     }
//
//     /* Cast the device extension to something we understand */
//     DeviceExtension = NewDeviceObject->DeviceExtension;
//
//     /* Attach this device to the top of the driver stack and store the DO we attached to in the DE */
//     Status = FltpAttachDeviceObject(NewDeviceObject,
//                                     DeviceObject,
//                                     &DeviceExtension->AttachedToDeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         DPRINT("Attached to %wZ\n", &FileSystemDeviceName);
//     }
//     else
//     {
//         DPRINT1("Failed to attach to the driver stack : 0x%X\n", Status);
//         goto Cleanup;
//     }
//
//     /* Setup the unicode string buffer and copy the device name to the device extension */
//     RtlInitEmptyUnicodeString(&DeviceExtension->DeviceName,
//                               DeviceExtension->DeviceNameBuffer,
//                               MAX_DEVNAME_LENGTH * sizeof(WCHAR));
//     RtlCopyUnicodeString(&DeviceExtension->DeviceName, DeviceName);
//
//     /* We're done, remove the initializing flag */
//     ClearFlag(NewDeviceObject->Flags, DO_DEVICE_INITIALIZING);
//
//     /* Look for existing mounted devices for this file system */
//     Status = FltpEnumerateFileSystemVolumes(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to enumerate file system volumes for this file system : 0x%X\n", Status);
//         IoDetachDevice(DeviceExtension->AttachedToDeviceObject);
//     }
//
// Cleanup:
//
//     if (!NT_SUCCESS(Status))
//     {
//         IoDeleteDevice(NewDeviceObject);
//     }
//
//     return Status;
// }
//
// CODE_SEG("PAGE")
// static
// LONG_PTR
// FltpDetachFromFileSystemDevice( PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_OBJECT AttachedDevice, NextDevice;
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     LONG_PTR Count;
//
//     PAGED_CODE();
//
//     /* Get the top device in the chain and increment the ref count on it */
//     AttachedDevice = IoGetAttachedDeviceReference(DeviceObject);
//
//     /* Loop all attached devices looking for our file system driver */
//     while (AttachedDevice->DriverObject != DriverData.DriverObject)
//     {
//         FLT_ASSERT(AttachedDevice != NULL);
//
//         /* Get the next lower device object. This adds a ref on NextDevice */
//         NextDevice = IoGetLowerDeviceObject(AttachedDevice);
//
//         /* Remove the reference we added */
//         Count = ObDereferenceObject(AttachedDevice);
//
//         /* Bail if this is the last one */
//         if (NextDevice == NULL) return Count;
//
//         /* Try the next one */
//         AttachedDevice = NextDevice;
//     }
//
//
//     DeviceExtension = AttachedDevice->DeviceExtension;
//     if (DeviceExtension)
//     {
//         //
//         // FIXME: Put any device extension cleanup code here
//         //
//     }
//
//     /* Detach the device from the chain and delete the object */
//     IoDetachDevice(DeviceObject);
//     IoDeleteDevice(AttachedDevice);
//
//     /* Remove the reference we added so the delete can complete */
//     return ObDereferenceObject(AttachedDevice);
// }
//
//
// /* DISPATCH ROUTINES **********************************************/
//
// NTSTATUS
// NTAPI
// FltpPreFsFilterOperation( PFS_FILTER_CALLBACK_DATA Data,
//                           PVOID *CompletionContext)
// {
//     UNREFERENCED_PARAMETER(Data);
//     UNREFERENCED_PARAMETER(CompletionContext);
//     __debugbreak();
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// FltpPostFsFilterOperation( PFS_FILTER_CALLBACK_DATA Data,
//                            NTSTATUS OperationStatus,
//                            PVOID CompletionContext)
// {
//     UNREFERENCED_PARAMETER(Data);
//     UNREFERENCED_PARAMETER(OperationStatus);
//     UNREFERENCED_PARAMETER(CompletionContext);
//     __debugbreak();
// }
//
// NTSTATUS
// NTAPI
// FltpDispatch( PDEVICE_OBJECT DeviceObject,
//               PIRP Irp)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION StackPtr;
//     NTSTATUS Status;
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* Check if this is a request for us */
//     if (DeviceObject == DriverData.DeviceObject)
//     {
//         FLT_ASSERT(DeviceObject->DriverObject == DriverData.DriverObject);
//         FLT_ASSERT(DeviceExtension == NULL);
//
//         /* Hand it off to our internal handler */
//         Status = FltpDispatchHandler(DeviceObject, Irp);
//         if (Status != STATUS_REPARSE)
//         {
//             Irp->IoStatus.Status = Status;
//             Irp->IoStatus.Information = 0;
//             IoCompleteRequest(Irp, 0);
//         }
//         return Status;
//     }
//
//     /* Check if this is a request for a the messaging device */
//     if (DeviceObject == CommsDeviceObject)
//     {
//         /* Hand off to our internal routine */
//         return FltpMsgDispatch(DeviceObject, Irp);
//     }
//
//     FLT_ASSERT(DeviceExtension &&
//                DeviceExtension->AttachedToDeviceObject);
//
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//     if (StackPtr->MajorFunction == IRP_MJ_SHUTDOWN)
//     {
//         // handle shutdown request
//     }
//
//     DPRINT1("Received %X from %wZ\n", StackPtr->MajorFunction, &DeviceExtension->DeviceName);
//
//     /* Just pass the IRP down the stack */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(DeviceExtension->AttachedToDeviceObject, Irp);
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// FltpCreate( PDEVICE_OBJECT DeviceObject,
//             PIRP Irp)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//
//     PAGED_CODE();
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     /* Check if this is a request for us */
//     if (DeviceObject == DriverData.DeviceObject)
//     {
//         FLT_ASSERT(DeviceObject->DriverObject == DriverData.DriverObject);
//         FLT_ASSERT(DeviceExtension == NULL);
//
//         /* Someone wants a handle to the fltmgr, allow it */
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = 0;
//         IofCompleteRequest(Irp, 0);
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if this is a request for a the new comms connection */
//     if (DeviceObject == CommsDeviceObject)
//     {
//         /* Hand off to our internal routine */
//         return FltpMsgCreate(DeviceObject, Irp);
//     }
//
//     FLT_ASSERT(DeviceExtension &&
//                DeviceExtension->AttachedToDeviceObject);
//
//     DPRINT1("Received create from %wZ (%lu)\n", &DeviceExtension->DeviceName, PsGetCurrentProcessId());
//
//     /* Just pass the IRP down the stack */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(DeviceExtension->AttachedToDeviceObject, Irp);
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// FltpFsControl( PDEVICE_OBJECT DeviceObject,
//                PIRP Irp)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//
//     PAGED_CODE();
//
//     /* Check if this is a request for us */
//     if (DeviceObject == DriverData.DeviceObject)
//     {
//         /* We don't handle this request */
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//         IofCompleteRequest(Irp, 0);
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     FLT_ASSERT(DeviceExtension &&
//                DeviceExtension->AttachedToDeviceObject);
//
//     /* Just pass the IRP down the stack */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(DeviceExtension->AttachedToDeviceObject, Irp);
// }
//
// NTSTATUS
// NTAPI
// FltpDeviceControl( PDEVICE_OBJECT DeviceObject,
//                    PIRP Irp)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//
//     /* Check if the request was meant for us */
//     if (DeviceObject == DriverData.DeviceObject)
//     {
//         Status = FltpDeviceControlHandler(DeviceObject, Irp);
//         if (Status != STATUS_REPARSE)
//         {
//             Irp->IoStatus.Status = Status;
//             Irp->IoStatus.Information = 0;
//             IoCompleteRequest(Irp, 0);
//         }
//
//         return Status;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//
//     FLT_ASSERT(DeviceExtension &&
//                DeviceExtension->AttachedToDeviceObject);
//
//     /* Just pass the IRP down the stack */
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(DeviceExtension->AttachedToDeviceObject, Irp);
// }
//
//
//
// /* FASTIO ROUTINES ************************************************/
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoCheckIfPossible( PFILE_OBJECT FileObject,
//                            PLARGE_INTEGER FileOffset,
//                            ULONG Length,
//                            BOOLEAN Wait,
//                            ULONG LockKey,
//                            BOOLEAN CheckForReadOperation,
//                            PIO_STATUS_BLOCK IoStatus,
//                            PDEVICE_OBJECT DeviceObject)
//
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoCheckIfPossible)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                      FileOffset,
//                                                      Length,
//                                                      Wait,
//                                                      LockKey,
//                                                      CheckForReadOperation,
//                                                      IoStatus,
//                                                      AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoRead( PFILE_OBJECT FileObject,
//                 PLARGE_INTEGER FileOffset,
//                 ULONG Length,
//                 BOOLEAN Wait,
//                 ULONG LockKey,
//                 PVOID Buffer,
//                 PIO_STATUS_BLOCK IoStatus,
//                 PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoRead)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoRead(FileObject,
//                                           FileOffset,
//                                           Length,
//                                           Wait,
//                                           LockKey,
//                                           Buffer,
//                                           IoStatus,
//                                           AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoWrite( PFILE_OBJECT FileObject,
//                  PLARGE_INTEGER FileOffset,
//                   ULONG Length,
//                   BOOLEAN Wait,
//                   ULONG LockKey,
//                   PVOID Buffer,
//                   PIO_STATUS_BLOCK IoStatus,
//                   PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoWrite)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoWrite(FileObject,
//                                            FileOffset,
//                                            Length,
//                                            Wait,
//                                            LockKey,
//                                            Buffer,
//                                            IoStatus,
//                                            AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoQueryBasicInfo( PFILE_OBJECT FileObject,
//                           BOOLEAN Wait,
//                           PFILE_BASIC_INFORMATION Buffer,
//                           PIO_STATUS_BLOCK IoStatus,
//                           PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoQueryBasicInfo)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoQueryBasicInfo(FileObject,
//                                                     Wait,
//                                                     Buffer,
//                                                     IoStatus,
//                                                     AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoQueryStandardInfo( PFILE_OBJECT FileObject,
//                              BOOLEAN Wait,
//                              PFILE_STANDARD_INFORMATION Buffer,
//                              PIO_STATUS_BLOCK IoStatus,
//                              PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoQueryStandardInfo)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoQueryStandardInfo(FileObject,
//                                                        Wait,
//                                                        Buffer,
//                                                        IoStatus,
//                                                        AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoLock( PFILE_OBJECT FileObject,
//                 PLARGE_INTEGER FileOffset,
//                 PLARGE_INTEGER Length,
//                 PEPROCESS ProcessId,
//                 ULONG Key,
//                 BOOLEAN FailImmediately,
//                 BOOLEAN ExclusiveLock,
//                 PIO_STATUS_BLOCK IoStatus,
//                 PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoLock)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoLock(FileObject,
//                                           FileOffset,
//                                           Length,
//                                           ProcessId,
//                                           Key,
//                                           FailImmediately,
//                                           ExclusiveLock,
//                                           IoStatus,
//                                           AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoUnlockSingle( PFILE_OBJECT FileObject,
//                         PLARGE_INTEGER FileOffset,
//                         PLARGE_INTEGER Length,
//                         PEPROCESS ProcessId,
//                         ULONG Key,
//                         PIO_STATUS_BLOCK IoStatus,
//                         PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoUnlockSingle)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoUnlockSingle(FileObject,
//                                                   FileOffset,
//                                                   Length,
//                                                   ProcessId,
//                                                   Key,
//                                                   IoStatus,
//                                                   AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoUnlockAll( PFILE_OBJECT FileObject,
//                      PEPROCESS ProcessId,
//                      PIO_STATUS_BLOCK IoStatus,
//                      PDEVICE_OBJECT DeviceObject)
//
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoUnlockAll)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoUnlockAll(FileObject,
//                                                ProcessId,
//                                                IoStatus,
//                                                AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoUnlockAllByKey( PFILE_OBJECT FileObject,
//                           PVOID ProcessId,
//                           ULONG Key,
//                           PIO_STATUS_BLOCK IoStatus,
//                           PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoUnlockAllByKey)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoUnlockAllByKey(FileObject,
//                                                     ProcessId,
//                                                     Key,
//                                                     IoStatus,
//                                                     AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoDeviceControl( PFILE_OBJECT FileObject,
//                          BOOLEAN Wait,
//                          PVOID InputBuffer,
//                          ULONG InputBufferLength,
//                          PVOID OutputBuffer,
//                          ULONG OutputBufferLength,
//                          ULONG IoControlCode,
//                          PIO_STATUS_BLOCK IoStatus,
//                          PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the request, send it down the slow path */
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoDeviceControl)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoDeviceControl(FileObject,
//                                                    Wait,
//                                                    InputBuffer,
//                                                    InputBufferLength,
//                                                    OutputBuffer,
//                                                    OutputBufferLength,
//                                                    IoControlCode,
//                                                    IoStatus,
//                                                    AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// VOID
// NTAPI
// FltpFastIoDetachDeviceWorker( PVOID Parameter)
// {
//     PDETACH_DEVICE_WORK_ITEM DetachDeviceWorkItem = Parameter;
//
//     /* Run any cleanup routines */
//     FltpCleanupDeviceObject(DetachDeviceWorkItem->SourceDevice);
//
//     /* Detach from the target device */
//     IoDetachDevice(DetachDeviceWorkItem->TargetDevice);
//
//     /* Delete the source */
//     IoDeleteDevice(DetachDeviceWorkItem->SourceDevice);
//
//     /* Free the pool we allocated in FltpFastIoDetachDevice */
//     ExFreePoolWithTag(DetachDeviceWorkItem, 0x1234);
// }
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// FltpFastIoDetachDevice( PDEVICE_OBJECT SourceDevice,
//                       PDEVICE_OBJECT TargetDevice)
// {
//     PDETACH_DEVICE_WORK_ITEM DetachDeviceWorkItem;
//
//     PAGED_CODE();
//
//     /*
//      * Detaching and deleting devices is a lot of work and takes too long
//      * to be a worthwhile FastIo candidate, so we defer this call to speed
//      * it up. There's no return value so we're okay to do this.
//      */
//
//     /* Allocate the work item and it's corresponding data */
//     DetachDeviceWorkItem = ExAllocatePoolWithTag(NonPagedPool,
//                                                  sizeof(DETACH_DEVICE_WORK_ITEM),
//                                                  0x1234);
//     if (DetachDeviceWorkItem)
//     {
//         /* Initialize the work item */
//         ExInitializeWorkItem(&DetachDeviceWorkItem->WorkItem,
//                              FltpFastIoDetachDeviceWorker,
//                              DetachDeviceWorkItem);
//
//         /* Queue the work item and return the call */
//         ExQueueWorkItem(&DetachDeviceWorkItem->WorkItem,
//                         DelayedWorkQueue);
//     }
//     else
//     {
//         /* We failed to defer, just cleanup here */
//         FltpCleanupDeviceObject(SourceDevice);
//         IoDetachDevice(TargetDevice);
//         IoDeleteDevice(SourceDevice);
//     }
//
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoQueryNetworkOpenInfo( PFILE_OBJECT FileObject,
//                                 BOOLEAN Wait,
//                                 PFILE_NETWORK_OPEN_INFORMATION Buffer,
//                                 PIO_STATUS_BLOCK IoStatus,
//                                 PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoQueryNetworkOpenInfo)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoQueryNetworkOpenInfo(FileObject,
//                                                           Wait,
//                                                           Buffer,
//                                                           IoStatus,
//                                                           AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoMdlRead( PFILE_OBJECT FileObject,
//                    PLARGE_INTEGER FileOffset,
//                    ULONG Length,
//                    ULONG LockKey,
//                    PMDL *MdlChain,
//                    PIO_STATUS_BLOCK IoStatus,
//                    PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->MdlRead)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->MdlRead(FileObject,
//                                        FileOffset,
//                                        Length,
//                                        LockKey,
//                                        MdlChain,
//                                        IoStatus,
//                                        AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoMdlReadComplete( PFILE_OBJECT FileObject,
//                            PMDL MdlChain,
//                            PDEVICE_OBJECT DeviceObject)
//
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the request, send it down the slow path */
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->MdlReadComplete)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->MdlReadComplete(FileObject,
//                                                MdlChain,
//                                                AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoPrepareMdlWrite( PFILE_OBJECT FileObject,
//                            PLARGE_INTEGER FileOffset,
//                            ULONG Length,
//                            ULONG LockKey,
//                            PMDL *MdlChain,
//                            PIO_STATUS_BLOCK IoStatus,
//                            PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the call */
//         IoStatus->Status = STATUS_INVALID_DEVICE_REQUEST;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->PrepareMdlWrite)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->PrepareMdlWrite(FileObject,
//                                                FileOffset,
//                                                Length,
//                                                LockKey,
//                                                MdlChain,
//                                                IoStatus,
//                                                AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoMdlWriteComplete( PFILE_OBJECT FileObject,
//                             PLARGE_INTEGER FileOffset,
//                             PMDL MdlChain,
//                             PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the request, send it down the slow path */
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->MdlWriteComplete)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->MdlWriteComplete(FileObject,
//                                                FileOffset,
//                                                MdlChain,
//                                                AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoReadCompressed( PFILE_OBJECT FileObject,
//                           PLARGE_INTEGER FileOffset,
//                           ULONG Length,
//                           ULONG LockKey,
//                           PVOID Buffer,
//                           PMDL *MdlChain,
//                           PIO_STATUS_BLOCK IoStatus,
//                           PCOMPRESSED_DATA_INFO CompressedDataInfo,
//                           ULONG CompressedDataInfoLength,
//                           PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the request, send it down the slow path */
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoReadCompressed)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoReadCompressed(FileObject,
//                                                     FileOffset,
//                                                     Length,
//                                                     LockKey,
//                                                     Buffer,
//                                                     MdlChain,
//                                                     IoStatus,
//                                                     CompressedDataInfo,
//                                                     CompressedDataInfoLength,
//                                                     AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoWriteCompressed( PFILE_OBJECT FileObject,
//                            PLARGE_INTEGER FileOffset,
//                            ULONG Length,
//                            ULONG LockKey,
//                            PVOID Buffer,
//                            PMDL *MdlChain,
//                            PIO_STATUS_BLOCK IoStatus,
//                            PCOMPRESSED_DATA_INFO CompressedDataInfo,
//                            ULONG CompressedDataInfoLength,
//                            PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         /* Fail the request, send it down the slow path */
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoWriteCompressed)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->FastIoWriteCompressed(FileObject,
//                                                      FileOffset,
//                                                      Length,
//                                                      LockKey,
//                                                      Buffer,
//                                                      MdlChain,
//                                                      IoStatus,
//                                                      CompressedDataInfo,
//                                                      CompressedDataInfoLength,
//                                                      AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoMdlReadCompleteCompressed( PFILE_OBJECT FileObject,
//                                      PMDL MdlChain,
//                                      PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->MdlReadCompleteCompressed)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->MdlReadCompleteCompressed(FileObject,
//                                                          MdlChain,
//                                                          AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoMdlWriteCompleteCompressed( PFILE_OBJECT FileObject,
//                                       PLARGE_INTEGER FileOffset,
//                                       PMDL MdlChain,
//                                       PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->MdlWriteCompleteCompressed)
//     {
//         /* Forward the call onto the device we attached to */
//         return FastIoDispatch->MdlWriteCompleteCompressed(FileObject,
//                                                           FileOffset,
//                                                           MdlChain,
//                                                           AttachedDeviceObject);
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// FltpFastIoQueryOpen( PIRP Irp,
//                      PFILE_NETWORK_OPEN_INFORMATION NetworkInformation,
//                      PDEVICE_OBJECT DeviceObject)
// {
//     PFLTMGR_DEVICE_EXTENSION DeviceExtension;
//     PDEVICE_OBJECT AttachedDeviceObject;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     BOOLEAN Success;
//
//     PAGED_CODE();
//
//     /* If it doesn't have a device extension, then it's not our device object */
//     if (DeviceObject->DeviceExtension == NULL)
//     {
//         return FALSE;
//     }
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     FLT_ASSERT(DeviceExtension->AttachedToDeviceObject);
//
//     /* Get the device that we attached to */
//     AttachedDeviceObject = DeviceExtension->AttachedToDeviceObject;
//     FastIoDispatch = AttachedDeviceObject->DriverObject->FastIoDispatch;
//
//     /* Make sure our FastIo table is valid */
//     if (FastIoDispatch && FastIoDispatch->FastIoQueryOpen)
//     {
//         PIO_STACK_LOCATION StackPtr = IoGetCurrentIrpStackLocation(Irp);
//
//         /* Update the stack to contain the correct device for the next filter */
//         StackPtr->DeviceObject = AttachedDeviceObject;
//
//         /* Now forward the call */
//         Success = FastIoDispatch->FastIoQueryOpen(Irp,
//                                                   NetworkInformation,
//                                                   AttachedDeviceObject);
//
//         /* Restore the DeviceObject as we found it */
//         StackPtr->DeviceObject = DeviceObject;
//         return Success;
//     }
//
//     /* We failed to handle the request, send it down the slow path */
//     FLT_ASSERT(FALSE);
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// DRIVER_FS_NOTIFICATION FltpFsNotification;
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// FltpFsNotification( PDEVICE_OBJECT DeviceObject,
//                     BOOLEAN FsActive)
// {
//     UNICODE_STRING DeviceName;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Set an empty string */
//     RtlInitUnicodeString(&DeviceName, NULL);
//
//     /* Get the name of the lowest device object on the stack */
//     Status = FltpGetBaseDeviceObjectName(DeviceObject, &DeviceName);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if it's attaching or detaching */
//         if (FsActive)
//         {
//             /* Run the attach routine */
//             FltpAttachToFileSystemDevice(DeviceObject, &DeviceName);
//         }
//         else
//         {
//             /* Run the detach routine */
//             FltpDetachFromFileSystemDevice(DeviceObject);
//         }
//
//         /* Free the buffer which FltpGetBaseDeviceObjectName allocated */
//         FltpFreeUnicodeString(&DeviceName);
//     }
// }
//
// static
// CODE_SEG("INIT")
// NTSTATUS
// SetupDispatchAndCallbacksTables( PDRIVER_OBJECT DriverObject)
// {
//     PFAST_IO_DISPATCH FastIoDispatch;
//     FS_FILTER_CALLBACKS Callbacks;
//     ULONG i;
//
//     /* Plug all the IRPs */
//     for (i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
//     {
//         DriverObject->MajorFunction[i] = FltpDispatch;
//     }
//
//     /* Override the ones we're interested in */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = FltpCreate;
//     DriverObject->MajorFunction[IRP_MJ_CREATE_NAMED_PIPE] = FltpCreate;
//     DriverObject->MajorFunction[IRP_MJ_CREATE_MAILSLOT] = FltpCreate;
//     DriverObject->MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL] = FltpFsControl;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = FltpDeviceControl;
//
//     /* The FastIo dispatch table is stored in the pool along with a tag */
//     FastIoDispatch = ExAllocatePoolWithTag(NonPagedPool, sizeof(FAST_IO_DISPATCH), FM_TAG_DISPATCH_TABLE);
//     if (FastIoDispatch == NULL) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Fill out the FastIo table  */
//     RtlZeroMemory(FastIoDispatch, sizeof(FAST_IO_DISPATCH));
//     FastIoDispatch->SizeOfFastIoDispatch = sizeof(FAST_IO_DISPATCH);
//     FastIoDispatch->FastIoCheckIfPossible = FltpFastIoCheckIfPossible;
//     FastIoDispatch->FastIoRead = FltpFastIoRead;
//     FastIoDispatch->FastIoWrite = FltpFastIoWrite;
//     FastIoDispatch->FastIoQueryBasicInfo = FltpFastIoQueryBasicInfo;
//     FastIoDispatch->FastIoQueryStandardInfo = FltpFastIoQueryStandardInfo;
//     FastIoDispatch->FastIoLock = FltpFastIoLock;
//     FastIoDispatch->FastIoUnlockSingle = FltpFastIoUnlockSingle;
//     FastIoDispatch->FastIoUnlockAll = FltpFastIoUnlockAll;
//     FastIoDispatch->FastIoUnlockAllByKey = FltpFastIoUnlockAllByKey;
//     FastIoDispatch->FastIoDeviceControl = FltpFastIoDeviceControl;
//     FastIoDispatch->FastIoDetachDevice = FltpFastIoDetachDevice;
//     FastIoDispatch->FastIoQueryNetworkOpenInfo = FltpFastIoQueryNetworkOpenInfo;
//     FastIoDispatch->MdlRead = FltpFastIoMdlRead;
//     FastIoDispatch->MdlReadComplete = FltpFastIoMdlReadComplete;
//     FastIoDispatch->PrepareMdlWrite = FltpFastIoPrepareMdlWrite;
//     FastIoDispatch->MdlWriteComplete = FltpFastIoMdlWriteComplete;
//     FastIoDispatch->FastIoReadCompressed = FltpFastIoReadCompressed;
//     FastIoDispatch->FastIoWriteCompressed = FltpFastIoWriteCompressed;
//     FastIoDispatch->MdlReadCompleteCompressed = FltpFastIoMdlReadCompleteCompressed;
//     FastIoDispatch->MdlWriteCompleteCompressed = FltpFastIoMdlWriteCompleteCompressed;
//     FastIoDispatch->FastIoQueryOpen = FltpFastIoQueryOpen;
//
//     /* Store the FastIo table for internal and our access */
//     DriverObject->FastIoDispatch = FastIoDispatch;
//     DriverData.FastIoDispatch = FastIoDispatch;
//
//     /* Initialize the callback table */
//     Callbacks.SizeOfFsFilterCallbacks = sizeof(FS_FILTER_CALLBACKS);
//     Callbacks.PreAcquireForSectionSynchronization = FltpPreFsFilterOperation;
//     Callbacks.PostAcquireForSectionSynchronization = FltpPostFsFilterOperation;
//     Callbacks.PreReleaseForSectionSynchronization = FltpPreFsFilterOperation;
//     Callbacks.PostReleaseForSectionSynchronization = FltpPostFsFilterOperation;
//     Callbacks.PreAcquireForCcFlush = FltpPreFsFilterOperation;
//     Callbacks.PostAcquireForCcFlush = FltpPostFsFilterOperation;
//     Callbacks.PreReleaseForCcFlush = FltpPreFsFilterOperation;
//     Callbacks.PostReleaseForCcFlush = FltpPostFsFilterOperation;
//     Callbacks.PreAcquireForModifiedPageWriter = FltpPreFsFilterOperation;
//     Callbacks.PostAcquireForModifiedPageWriter = FltpPostFsFilterOperation;
//     Callbacks.PreReleaseForModifiedPageWriter = FltpPreFsFilterOperation;
//     Callbacks.PostReleaseForModifiedPageWriter = FltpPostFsFilterOperation;
//
//     /* Register our callbacks */
//     return FsRtlRegisterFileSystemFilterCallbacks(DriverObject, &Callbacks);
// }
//
// CODE_SEG("INIT") DRIVER_INITIALIZE DriverEntry;
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// DriverEntry( PDRIVER_OBJECT DriverObject,
//              PUNICODE_STRING RegistryPath)
// {
//     UNICODE_STRING DeviceName = RTL_CONSTANT_STRING(L"\\FileSystem\\Filters\\"DRIVER_NAME);
//     PDEVICE_OBJECT RawDeviceObject;
//     PDEVICE_OBJECT DeviceObject;
//     PFILE_OBJECT RawFileObject;
//     UNICODE_STRING ObjectName;
//     UNICODE_STRING SymLink;
//
//     NTSTATUS Status;
//
//     RtlZeroMemory(&DriverData, sizeof(DRIVER_DATA));
//     DriverData.DriverObject = DriverObject;
//
//     /* Save the registry key for this driver */
//     DriverData.ServiceKey.Length = RegistryPath->Length;
//     DriverData.ServiceKey.MaximumLength = RegistryPath->MaximumLength;
//     DriverData.ServiceKey.Buffer = (PWCHAR)ExAllocatePoolWithTag(NonPagedPool,
//                                                                  RegistryPath->MaximumLength,
//                                                                  FM_TAG_REGISTRY_DATA);
//     if (!DriverData.ServiceKey.Buffer) return STATUS_INSUFFICIENT_RESOURCES;
//     RtlCopyUnicodeString(&DriverData.ServiceKey, RegistryPath);
//
//     /* Do some initialization */
//     ExInitializeFastMutex(&DriverData.FilterAttachLock);
//
//     /* Create the main filter manager device object */
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &DeviceName,
//                             FILE_DEVICE_DISK_FILE_SYSTEM,
//                             FILE_DEVICE_SECURE_OPEN,
//                             FALSE,
//                             &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("fltmgr IoCreateDevice failed.  Status = %X\n", Status);
//         goto Cleanup;
//     }
//
//     /* Store a global reference so we can access from callbacks */
//     DriverData.DeviceObject = DeviceObject;
//
//     /* Generate the symbolic link name */
//     RtlInitUnicodeString(&SymLink, L"\\??\\"DRIVER_NAME);
//     Status = IoCreateSymbolicLink(&SymLink, &DeviceName);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Create the callbacks for the dispatch table, FastIo and FS callbacks */
//     Status = SetupDispatchAndCallbacksTables(DriverObject);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Initialize the comms objects */
//     Status = FltpSetupCommunicationObjects(DriverObject);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Register for notifications when a new file system is loaded. This also enumerates any existing file systems */
//     Status = IoRegisterFsRegistrationChange(DriverObject, FltpFsNotification);
//     FLT_ASSERT(Status != STATUS_DEVICE_ALREADY_ATTACHED); // Windows checks for this, I'm not sure how it can happen. Needs investigation??
//     if (!NT_SUCCESS(Status))  goto Cleanup;
//
//     InitializeListHead(&FilterList);
//     ExInitializeResourceLite(&FilterListLock);
//
//     /* IoRegisterFsRegistrationChange isn't notified about the raw  file systems, so we attach to them manually */
//     RtlInitUnicodeString(&ObjectName, L"\\Device\\RawDisk");
//     Status = IoGetDeviceObjectPointer(&ObjectName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &RawFileObject,
//                                       &RawDeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         FltpFsNotification(RawDeviceObject, TRUE);
//         ObDereferenceObject(RawFileObject);
//     }
//
//     RtlInitUnicodeString(&ObjectName, L"\\Device\\RawCdRom");
//     Status = IoGetDeviceObjectPointer(&ObjectName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &RawFileObject,
//                                       &RawDeviceObject);
//     if (NT_SUCCESS(Status))
//     {
//         FltpFsNotification(RawDeviceObject, TRUE);
//         ObDereferenceObject(RawFileObject);
//     }
//
//     /* We're done, clear the initializing flag */
//     ClearFlag(DeviceObject->Flags, DO_DEVICE_INITIALIZING);
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (DriverData.FastIoDispatch)
//         {
//             DriverObject->FastIoDispatch = NULL;
//             ExFreePoolWithTag(DriverData.FastIoDispatch, FM_TAG_DISPATCH_TABLE);
//         }
//
//         IoDeleteSymbolicLink(&SymLink);
//
//         if (DeviceObject)
//             IoDeleteDevice(DeviceObject);
//
//         if (DriverData.ServiceKey.Buffer)
//             ExFreePoolWithTag(DriverData.ServiceKey.Buffer, FM_TAG_REGISTRY_DATA);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
