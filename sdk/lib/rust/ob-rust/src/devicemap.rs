use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _DEVICE_MAP, _SECURITY_IMPERSONATION_LEVEL,
    PEPROCESS, POBJECT_DIRECTORY, POBJECT_HEADER, POBJECT_HEADER_NAME_INFO,
    PDEVICE_MAP, PETHREAD, PROCESS_DEVICEMAP_INFORMATION, PPROCESS_DEVICEMAP_INFORMATION,
    LUID, HANDLE, PACCESS_TOKEN, PLUID, PBOOLEAN,
    ObReferenceObjectByHandle, ObMakeTemporaryObject,
    PsReferenceImpersonationToken, PsReferencePrimaryToken, PsGetCurrentThread,
    SeQueryAuthenticationIdToken, SeGetLogonIdDeviceMap,
    ObpReferenceNameInfo, ObpDereferenceNameInfo, ObpEnterObjectTypeMutex, ObpLeaveObjectTypeMutex,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// NTSTATUS codes not in ntkrnl_sys::constants
const STATUS_NO_TOKEN: NTSTATUS = 0xC000007C_u32 as i32;
const STATUS_END_OF_FILE: NTSTATUS = 0xC0000011_u32 as i32;

// Device map constants (from obtypes.h, pstypes.h)
const PROCESS_LUID_DOSDEVICES_ONLY: ULONG = 0x00000001;
const DOSDEVICE_DRIVE_CALCULATE: UCHAR = 1;
const DOSDEVICE_DRIVE_REMOTE: UCHAR = 4;

#[no_mangle]
pub static mut ObSystemDeviceMap: PDEVICE_MAP = core::ptr::null_mut();
#[no_mangle]
pub static mut ObpDeviceMapLock: KGUARDED_MUTEX = KGUARDED_MUTEX::zeroed();

#[no_mangle]
pub static mut ObpLUIDDeviceMapsDisabled: ULONG = 0;
#[no_mangle]
pub static mut ObpLUIDDeviceMapsEnabled: ULONG = 0;

// Registry-controlled values owned by ob/devicemap. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static DEVICEMAP_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Session Manager", "LUIDDeviceMapsDisabled",
            core::ptr::addr_of_mut!(ObpLUIDDeviceMapsDisabled)),
    ] });

#[no_mangle]
pub unsafe extern "system" fn ObSetDeviceMap(
    Process: PEPROCESS,
    DirectoryHandle: HANDLE,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let DirectoryHandle = opaque_ptr(DirectoryHandle);
    let mut directory_object: POBJECT_DIRECTORY = core::ptr::null_mut();
    let mut device_map: PDEVICE_MAP = core::ptr::null_mut();
    let mut new_device_map: PDEVICE_MAP = core::ptr::null_mut();
    let old_device_map: PDEVICE_MAP;
    let mut status: NTSTATUS;
    let work_process: PEPROCESS;
    let mut make_permanent = 0u8;

    status = ObReferenceObjectByHandle(
        DirectoryHandle,
        DIRECTORY_TRAVERSE,
        ObpDirectoryObjectType,
        KeGetPreviousMode(),
        core::ptr::addr_of_mut!(directory_object) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    device_map = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_DEVICE_MAP>(),
        u32::from_ne_bytes(*b"ObDm"),
    ) as PDEVICE_MAP;
    if device_map.is_null() {
        ObDereferenceObject(directory_object as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(device_map as *mut u8, 0, core::mem::size_of::<_DEVICE_MAP>());
    (*device_map).ReferenceCount = 1;
    (*device_map).DosDevicesDirectory = directory_object;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if (*directory_object).DeviceMap.is_null() {
        (*directory_object).DeviceMap = device_map;
    } else {
        new_device_map = device_map;
        device_map = (*directory_object).DeviceMap;
        (*device_map).ReferenceCount += 1;
    }

    if !Process.is_null() {
        work_process = Process as PEPROCESS;
    } else {
        work_process = PsGetCurrentProcess() as PEPROCESS;
        ObSystemDeviceMap = device_map;
    }

    if directory_object != (*ObSystemDeviceMap).DosDevicesDirectory {
        (*device_map).GlobalDosDevicesDirectory = (*ObSystemDeviceMap).DosDevicesDirectory;
        make_permanent = 1u8;
    }

    old_device_map = (*(work_process as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP;
    (*(work_process as *mut _EPROCESS)).DeviceMap = device_map as PVOID;

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if make_permanent != 0 {
        let object_header: POBJECT_HEADER = OBJECT_TO_OBJECT_HEADER(directory_object as PVOID);
        let header_name_info: POBJECT_HEADER_NAME_INFO = ObpReferenceNameInfo(object_header);

        ObpEnterObjectTypeMutex((*object_header).Type);
        if !header_name_info.is_null() && !(*header_name_info).Directory.is_null() {
            (*object_header).Flags |= OB_FLAG_PERMANENT;
        }
        ObpLeaveObjectTypeMutex((*object_header).Type);

        if !header_name_info.is_null() {
            ObpDereferenceNameInfo(header_name_info);
        }
    }

    if !new_device_map.is_null() {
        ObDereferenceObject(directory_object as PVOID);
        ExFreePoolWithTag(new_device_map as PVOID, u32::from_ne_bytes(*b"ObDm"));
    }

    if !old_device_map.is_null() {
        ObfDereferenceDeviceMap(old_device_map);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ObSetDirectoryDeviceMap(
    DeviceMap: *mut PDEVICE_MAP,
    DirectoryHandle: HANDLE,
) -> NTSTATUS {
    let DeviceMap = opaque_ptr(DeviceMap);
    let DirectoryHandle = opaque_ptr(DirectoryHandle);
    let mut directory_object: POBJECT_DIRECTORY = core::ptr::null_mut();
    let mut local_map: PDEVICE_MAP = core::ptr::null_mut();
    let mut new_device_map: PDEVICE_MAP = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let object_header: POBJECT_HEADER;
    let header_name_info: POBJECT_HEADER_NAME_INFO;

    status = ObReferenceObjectByHandle(
        DirectoryHandle,
        DIRECTORY_TRAVERSE,
        ObpDirectoryObjectType,
        0i8,
        core::ptr::addr_of_mut!(directory_object) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    local_map = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_DEVICE_MAP>(),
        u32::from_ne_bytes(*b"ObDm"),
    ) as PDEVICE_MAP;
    if local_map.is_null() {
        ObDereferenceObject(directory_object as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(local_map as *mut u8, 0, core::mem::size_of::<_DEVICE_MAP>());
    (*local_map).ReferenceCount = 1;
    (*local_map).DosDevicesDirectory = directory_object;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if (*directory_object).DeviceMap.is_null() {
        (*directory_object).DeviceMap = local_map;
    } else {
        new_device_map = local_map;
        local_map = (*directory_object).DeviceMap;
        (*local_map).ReferenceCount += 1;
    }

    if directory_object != (*ObSystemDeviceMap).DosDevicesDirectory {
        (*local_map).GlobalDosDevicesDirectory = (*ObSystemDeviceMap).DosDevicesDirectory;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if !DeviceMap.is_null() {
        *DeviceMap = local_map;
    }

    object_header = OBJECT_TO_OBJECT_HEADER(directory_object as PVOID);
    header_name_info = ObpReferenceNameInfo(object_header);

    ObpEnterObjectTypeMutex((*object_header).Type);
    if !header_name_info.is_null() && !(*header_name_info).Directory.is_null() {
        (*object_header).Flags |= OB_FLAG_PERMANENT;
    }
    ObpLeaveObjectTypeMutex((*object_header).Type);

    if !header_name_info.is_null() {
        ObpDereferenceNameInfo(header_name_info);
    }

    if !new_device_map.is_null() {
        ObDereferenceObject(directory_object as PVOID);
        ExFreePoolWithTag(new_device_map as PVOID, u32::from_ne_bytes(*b"ObDm"));
    }

    status
}

unsafe fn ObpSetCurrentProcessDeviceMap() -> NTSTATUS {
    let token: PVOID;
    let mut logon_id: LUID = core::mem::zeroed();
    let mut status: NTSTATUS;
    let current_process: PEPROCESS;
    let system_luid = LUID { LowPart: 0x3e7, HighPart: 0 };
    let mut device_map: PDEVICE_MAP = core::ptr::null_mut();
    let old_device_map: PDEVICE_MAP;

    current_process = PsGetCurrentProcess() as PEPROCESS;
    token = PsReferencePrimaryToken(current_process) as PVOID;
    if token.is_null() {
        return STATUS_NO_TOKEN;
    }

    status = SeQueryAuthenticationIdToken(token as PACCESS_TOKEN, core::ptr::addr_of_mut!(logon_id) as PLUID);
    if status < 0 {
        ObDereferenceObject(token);
        return status;
    }

    if logon_id.LowPart == system_luid.LowPart
        && logon_id.HighPart == system_luid.HighPart
    {
        device_map = ObSystemDeviceMap;
    } else {
        status = SeGetLogonIdDeviceMap(core::ptr::addr_of_mut!(logon_id) as PLUID, core::ptr::addr_of_mut!(device_map));
        if status < 0 {
            status = STATUS_OBJECT_PATH_INVALID;
            ObDereferenceObject(token);
            return status;
        }
    }

    if device_map.is_null() {
        status = STATUS_OBJECT_PATH_INVALID;
        ObDereferenceObject(token);
        return status;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    old_device_map = (*current_process).DeviceMap as PDEVICE_MAP;

    (*device_map).ReferenceCount += 1;
    (*current_process).DeviceMap = device_map as PVOID;

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if !old_device_map.is_null() {
        ObfDereferenceDeviceMap(old_device_map);
    }

    ObDereferenceObject(token);

    status
}

#[no_mangle]
pub unsafe extern "system" fn ObpReferenceDeviceMap() -> PDEVICE_MAP {
    let mut logon_id: LUID = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut token: PVOID = core::ptr::null_mut();
    let mut device_map: PDEVICE_MAP = core::ptr::null_mut();
    let current_thread: PETHREAD;
    let mut looking_for_system: BOOLEAN;
    let system_luid = LUID { LowPart: 0x3e7, HighPart: 0 };
    let mut copy_on_open: BOOLEAN = 0;
    let mut effective_only: BOOLEAN = 0;
    let mut impersonation_level: _SECURITY_IMPERSONATION_LEVEL = 0;

    looking_for_system = 0u8;

    if ObpLUIDDeviceMapsEnabled != 0 {
        current_thread = PsGetCurrentThread();
        if (*current_thread).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
            token = PsReferenceImpersonationToken(
                current_thread,
                core::ptr::addr_of_mut!(copy_on_open) as PBOOLEAN,
                core::ptr::addr_of_mut!(effective_only) as PBOOLEAN,
                core::ptr::addr_of_mut!(impersonation_level),
            ) as PVOID;
            if !token.is_null() {
                status = SeQueryAuthenticationIdToken(token as PACCESS_TOKEN, core::ptr::addr_of_mut!(logon_id) as PLUID);
            } else {
                status = STATUS_NO_TOKEN;
            }

            if status >= 0 {
                if !(logon_id.LowPart == system_luid.LowPart
                    && logon_id.HighPart == system_luid.HighPart)
                {
                    status = SeGetLogonIdDeviceMap(core::ptr::addr_of_mut!(logon_id) as PLUID, core::ptr::addr_of_mut!(device_map));
                    if status >= 0 {
                        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

                        if !device_map.is_null() {
                            (*device_map).ReferenceCount += 1;
                        }

                        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

                        if !device_map.is_null() {
                            ObDereferenceObject(token);

                            return device_map;
                        }
                    }
                } else {
                    looking_for_system = 1u8;
                }
            }
        }

        if (*(PsGetCurrentProcess() as *mut _EPROCESS)).DeviceMap.is_null()
            && ObpSetCurrentProcessDeviceMap() < 0
        {
            if !token.is_null() {
                ObDereferenceObject(token);
            }

            return core::ptr::null_mut();
        }
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if looking_for_system != 0 {
        device_map = ObSystemDeviceMap;
    } else {
        device_map = (*(PsGetCurrentProcess() as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP;
    }

    if !device_map.is_null() {
        (*device_map).ReferenceCount += 1;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if !token.is_null() {
        ObDereferenceObject(token);
    }

    device_map
}

#[no_mangle]
pub unsafe extern "system" fn ObDereferenceDeviceMap(Process: PEPROCESS) {
    let Process = opaque_ptr(Process);
    let device_map: PDEVICE_MAP;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);
    device_map = (*(Process as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP;
    (*(Process as *mut _EPROCESS)).DeviceMap = core::ptr::null_mut();
    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if !device_map.is_null() {
        ObfDereferenceDeviceMap(device_map);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObfDereferenceDeviceMap(DeviceMap: PDEVICE_MAP) {
    ObfDereferenceDeviceMapImpl(DeviceMap);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObfDereferenceDeviceMap(DeviceMap: PDEVICE_MAP) {
    ObfDereferenceDeviceMapImpl(DeviceMap);
}

unsafe fn ObfDereferenceDeviceMapImpl(device_map: PDEVICE_MAP) {
    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    (*device_map).ReferenceCount -= 1;

    if (*device_map).ReferenceCount != 0 {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);
        return;
    }

    (*(*device_map).DosDevicesDirectory).DeviceMap = core::ptr::null_mut();

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    ObMakeTemporaryObject((*device_map).DosDevicesDirectory as PVOID);
    ObDereferenceObject((*device_map).DosDevicesDirectory as PVOID);
    ExFreePoolWithTag(device_map as PVOID, u32::from_ne_bytes(*b"ObDm"));
}

#[no_mangle]
pub unsafe extern "system" fn ObInheritDeviceMap(Parent: PEPROCESS, Process: PEPROCESS) {
    let Parent = opaque_ptr(Parent);
    let Process = opaque_ptr(Process);
    let device_map: PDEVICE_MAP;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    device_map = if !Parent.is_null() {
        (*(Parent as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP
    } else {
        ObSystemDeviceMap
    };

    if !device_map.is_null() {
        (*device_map).ReferenceCount += 1;

        (*(Process as *mut _EPROCESS)).DeviceMap = device_map as PVOID;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);
}

#[no_mangle]
pub unsafe extern "system" fn ObQueryDeviceMapInformation(
    Process: PEPROCESS,
    DeviceMapInfo: PPROCESS_DEVICEMAP_INFORMATION,
    Flags: ULONG,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let mut device_map: PDEVICE_MAP = core::ptr::null_mut();
    let mut global_device_map: PDEVICE_MAP;
    let mut dereference: BOOLEAN;
    let mut map_info: PROCESS_DEVICEMAP_INFORMATION = core::mem::zeroed();
    let mut bit_mask: ULONG;
    let mut i: ULONG;
    let return_any: BOOLEAN;
    let mut status: NTSTATUS;

    if (Flags & !PROCESS_LUID_DOSDEVICES_ONLY) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    dereference = 0u8;
    return_any = if (Flags & PROCESS_LUID_DOSDEVICES_ONLY) != 0 { 0u8 } else { 1u8 };

    if ObpLUIDDeviceMapsEnabled != 0 {
        if !Process.is_null() && Process as PVOID != PsGetCurrentProcess() as PVOID {
            return STATUS_INVALID_PARAMETER;
        }

        device_map = ObpReferenceDeviceMap();
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if !device_map.is_null() {
        dereference = 1u8;
    } else {
        device_map = if !Process.is_null() {
            (*(Process as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP
        } else {
            ObSystemDeviceMap
        };
    }

    if device_map.is_null() {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);
        return STATUS_END_OF_FILE;
    }

    status = STATUS_SUCCESS;

    global_device_map = device_map;
    if !(*device_map).GlobalDosDevicesDirectory.is_null() {
        if !(*(*device_map).GlobalDosDevicesDirectory).DeviceMap.is_null() {
            global_device_map = (*(*device_map).GlobalDosDevicesDirectory).DeviceMap;
        }
    }

    map_info.__bindgen_anon_1.Query.DriveMap = (*device_map).DriveMap;
    i = 0;
    bit_mask = 1;
    while i < 32 {
        map_info.__bindgen_anon_1.Query.DriveType[i as usize] = (*device_map).DriveType[i as usize];

        if (map_info.__bindgen_anon_1.Query.DriveMap & bit_mask) == 0 && return_any != 0 {
            if ObpLUIDDeviceMapsEnabled != 0
                || ((*global_device_map).DriveType[i as usize] != DOSDEVICE_DRIVE_REMOTE
                    && (*global_device_map).DriveType[i as usize] != DOSDEVICE_DRIVE_CALCULATE)
            {
                map_info.__bindgen_anon_1.Query.DriveType[i as usize] = (*global_device_map).DriveType[i as usize];
                map_info.__bindgen_anon_1.Query.DriveMap |= bit_mask & (*global_device_map).DriveMap;
            }
        }

        i += 1;
        bit_mask *= 2;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    if dereference != 0 {
        ObfDereferenceDeviceMapImpl(device_map);
    }

    // Only the Query member: callers pass a buffer of exactly its size (GetLogicalDrives,
    // and NtQueryInformationProcess insists on it). The C copies the whole union, which on
    // 64-bit is 40 bytes (Set's HANDLE pads it), 4 past a 36-byte Query; special pool
    // caught it on the OptiPlex (BUGCHECK 0xC1 at the bounce buffer's free).
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!(map_info.__bindgen_anon_1.Query) as *const u8,
        core::ptr::addr_of_mut!((*DeviceMapInfo).__bindgen_anon_1.Query) as *mut u8,
        core::mem::size_of_val(&map_info.__bindgen_anon_1.Query),
    );

    status
}

#[no_mangle]
pub unsafe extern "system" fn ObIsLUIDDeviceMapsEnabled() -> ULONG {
    ObpLUIDDeviceMapsEnabled
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/devicemap.c
//  * PURPOSE:         Device map implementation
//  * PROGRAMMERS:     Eric Kohl (eric.kohl@reactos.org)
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// ULONG ObpLUIDDeviceMapsDisabled;
// ULONG ObpLUIDDeviceMapsEnabled;
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// NTSTATUS
// NTAPI
// ObSetDeviceMap(IN PEPROCESS Process,
//                IN HANDLE DirectoryHandle)
// {
//     POBJECT_DIRECTORY DirectoryObject = NULL;
//     PDEVICE_MAP DeviceMap = NULL, NewDeviceMap = NULL, OldDeviceMap;
//     NTSTATUS Status;
//     PEPROCESS WorkProcess;
//     BOOLEAN MakePermanant = FALSE;
//
//     Status = ObReferenceObjectByHandle(DirectoryHandle,
//                                        DIRECTORY_TRAVERSE,
//                                        ObpDirectoryObjectType,
//                                        KeGetPreviousMode(),
//                                        (PVOID*)&DirectoryObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ObReferenceObjectByHandle() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Allocate and initialize a new device map */
//     DeviceMap = ExAllocatePoolWithTag(PagedPool,
//                                       sizeof(*DeviceMap),
//                                       'mDbO');
//     if (DeviceMap == NULL)
//     {
//         ObDereferenceObject(DirectoryObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize the device map */
//     RtlZeroMemory(DeviceMap, sizeof(*DeviceMap));
//     DeviceMap->ReferenceCount = 1;
//     DeviceMap->DosDevicesDirectory = DirectoryObject;
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Attach the device map to the directory object */
//     if (DirectoryObject->DeviceMap == NULL)
//     {
//         DirectoryObject->DeviceMap = DeviceMap;
//     }
//     else
//     {
//         NewDeviceMap = DeviceMap;
//
//         /* There's already a device map,
//            so reuse it */
//         DeviceMap = DirectoryObject->DeviceMap;
//         ++DeviceMap->ReferenceCount;
//     }
//
//     /* Caller gave a process, use it */
//     if (Process != NULL)
//     {
//         WorkProcess = Process;
//     }
//     /* If no process given, use current and
//      * set system device map */
//     else
//     {
//         WorkProcess = PsGetCurrentProcess();
//         ObSystemDeviceMap = DeviceMap;
//     }
//
//     /* If current object isn't system one, save system one in current
//      * device map */
//     if (DirectoryObject != ObSystemDeviceMap->DosDevicesDirectory)
//     {
//         /* We also need to make the object permanant */
//         DeviceMap->GlobalDosDevicesDirectory = ObSystemDeviceMap->DosDevicesDirectory;
//         MakePermanant = TRUE;
//     }
//
//     /* Save old process device map */
//     OldDeviceMap = WorkProcess->DeviceMap;
//     /* Attach the device map to the process */
//     WorkProcess->DeviceMap = DeviceMap;
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* If we have to make the object permamant, do it now */
//     if (MakePermanant)
//     {
//         POBJECT_HEADER ObjectHeader;
//         POBJECT_HEADER_NAME_INFO HeaderNameInfo;
//
//         ObjectHeader = OBJECT_TO_OBJECT_HEADER(DirectoryObject);
//         HeaderNameInfo = ObpReferenceNameInfo(ObjectHeader);
//
//         ObpEnterObjectTypeMutex(ObjectHeader->Type);
//         if (HeaderNameInfo != NULL && HeaderNameInfo->Directory != NULL)
//         {
//             ObjectHeader->Flags |= OB_FLAG_PERMANENT;
//         }
//         ObpLeaveObjectTypeMutex(ObjectHeader->Type);
//
//         if (HeaderNameInfo != NULL)
//         {
//             ObpDereferenceNameInfo(HeaderNameInfo);
//         }
//     }
//
//     /* Release useless device map if required */
//     if (NewDeviceMap != NULL)
//     {
//         ObDereferenceObject(DirectoryObject);
//         ExFreePoolWithTag(NewDeviceMap, 'mDbO');
//     }
//
//     /* And dereference previous process device map */
//     if (OldDeviceMap != NULL)
//     {
//         ObfDereferenceDeviceMap(OldDeviceMap);
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// ObSetDirectoryDeviceMap(OUT PDEVICE_MAP * DeviceMap,
//                         IN HANDLE DirectoryHandle)
// {
//     POBJECT_DIRECTORY DirectoryObject = NULL;
//     PDEVICE_MAP LocalMap = NULL, NewDeviceMap = NULL;
//     NTSTATUS Status;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_HEADER_NAME_INFO HeaderNameInfo;
//
//     Status = ObReferenceObjectByHandle(DirectoryHandle,
//                                        DIRECTORY_TRAVERSE,
//                                        ObpDirectoryObjectType,
//                                        KernelMode,
//                                        (PVOID*)&DirectoryObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ObReferenceObjectByHandle() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Allocate and initialize a new device map */
//     LocalMap = ExAllocatePoolWithTag(PagedPool,
//                                      sizeof(*LocalMap),
//                                      'mDbO');
//     if (LocalMap == NULL)
//     {
//         ObDereferenceObject(DirectoryObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize the device map */
//     RtlZeroMemory(LocalMap, sizeof(*LocalMap));
//     LocalMap->ReferenceCount = 1;
//     LocalMap->DosDevicesDirectory = DirectoryObject;
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Attach the device map to the directory object */
//     if (DirectoryObject->DeviceMap == NULL)
//     {
//         DirectoryObject->DeviceMap = LocalMap;
//     }
//     else
//     {
//         NewDeviceMap = LocalMap;
//
//         /* There's already a device map,
//            so reuse it */
//         LocalMap = DirectoryObject->DeviceMap;
//         ++LocalMap->ReferenceCount;
//     }
//
//     /* If current object isn't system one, save system one in current
//      * device map */
//     if (DirectoryObject != ObSystemDeviceMap->DosDevicesDirectory)
//     {
//         /* We also need to make the object permanant */
//         LocalMap->GlobalDosDevicesDirectory = ObSystemDeviceMap->DosDevicesDirectory;
//     }
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     if (DeviceMap != NULL)
//     {
//         *DeviceMap = LocalMap;
//     }
//
//     /* Caller expects us to make the object permanant, so do it! */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(DirectoryObject);
//     HeaderNameInfo = ObpReferenceNameInfo(ObjectHeader);
//
//     ObpEnterObjectTypeMutex(ObjectHeader->Type);
//     if (HeaderNameInfo != NULL && HeaderNameInfo->Directory != NULL)
//     {
//         ObjectHeader->Flags |= OB_FLAG_PERMANENT;
//     }
//     ObpLeaveObjectTypeMutex(ObjectHeader->Type);
//
//     if (HeaderNameInfo != NULL)
//     {
//         ObpDereferenceNameInfo(HeaderNameInfo);
//     }
//
//     /* Release useless device map if required */
//     if (NewDeviceMap != NULL)
//     {
//         ObDereferenceObject(DirectoryObject);
//         ExFreePoolWithTag(NewDeviceMap, 'mDbO');
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// ObpSetCurrentProcessDeviceMap(VOID)
// {
//     PTOKEN Token;
//     LUID LogonId;
//     NTSTATUS Status;
//     PEPROCESS CurrentProcess;
//     LUID SystemLuid = SYSTEM_LUID;
//     PDEVICE_MAP DeviceMap, OldDeviceMap;
//
//     /* Get our impersonation token */
//     CurrentProcess = PsGetCurrentProcess();
//     Token = PsReferencePrimaryToken(CurrentProcess);
//     if (Token == NULL)
//     {
//         return STATUS_NO_TOKEN;
//     }
//
//     /* Query the Logon ID */
//     Status = SeQueryAuthenticationIdToken(Token, &LogonId);
//     if (!NT_SUCCESS(Status))
//     {
//         goto done;
//     }
//
//     /* If that's system, then use system device map */
//     if (RtlEqualLuid(&LogonId, &SystemLuid))
//     {
//         DeviceMap = ObSystemDeviceMap;
//     }
//     /* Otherwise ask Se for the device map */
//     else
//     {
//         Status = SeGetLogonIdDeviceMap(&LogonId, &DeviceMap);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Normalize failure status */
//             Status = STATUS_OBJECT_PATH_INVALID;
//             goto done;
//         }
//     }
//
//     /* Fail if no device map */
//     if (DeviceMap == NULL)
//     {
//         Status = STATUS_OBJECT_PATH_INVALID;
//         goto done;
//     }
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Save old device map attached to the process */
//     OldDeviceMap = CurrentProcess->DeviceMap;
//
//     /* Set new device map & reference it */
//     ++DeviceMap->ReferenceCount;
//     CurrentProcess->DeviceMap = DeviceMap;
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* If we had a device map, dereference it */
//     if (OldDeviceMap != NULL)
//     {
//         ObfDereferenceDeviceMap(OldDeviceMap);
//     }
//
// done:
//     /* We're done with the token! */
//     ObDereferenceObject(Token);
//
//     return Status;
// }
//
//
// PDEVICE_MAP
// NTAPI
// ObpReferenceDeviceMap(VOID)
// {
//     LUID LogonId;
//     NTSTATUS Status;
//     PTOKEN Token = NULL;
//     PDEVICE_MAP DeviceMap;
//     PETHREAD CurrentThread;
//     BOOLEAN LookingForSystem;
//     LUID SystemLuid = SYSTEM_LUID;
//     BOOLEAN CopyOnOpen, EffectiveOnly;
//     SECURITY_IMPERSONATION_LEVEL ImpersonationLevel;
//
//     LookingForSystem = FALSE;
//
//     /* If LUID mapping is enable, try to get appropriate device map */
//     if (ObpLUIDDeviceMapsEnabled != 0)
//     {
//         /* In case of impersonation, we've got a bit of work to do */
//         CurrentThread = PsGetCurrentThread();
//         if (CurrentThread->ActiveImpersonationInfo)
//         {
//             /* Get impersonation token */
//             Token = PsReferenceImpersonationToken(CurrentThread,
//                                                   &CopyOnOpen,
//                                                   &EffectiveOnly,
//                                                   &ImpersonationLevel);
//             /* Get logon LUID */
//             if (Token != NULL)
//             {
//                 Status = SeQueryAuthenticationIdToken(Token, &LogonId);
//             }
//             else
//             {
//                 /* Force failure */
//                 Status = STATUS_NO_TOKEN;
//             }
//
//             /* If we got logon LUID */
//             if (NT_SUCCESS(Status))
//             {
//                 /*
//                  * Check it's not system, system is easy to handle,
//                  * we just need to return ObSystemDeviceMap
//                  */
//                 if (!RtlEqualLuid(&LogonId, &SystemLuid))
//                 {
//                     /* Ask Se for the device  map */
//                     Status = SeGetLogonIdDeviceMap(&LogonId, &DeviceMap);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* Acquire the device map lock */
//                         KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//                         /* Reference the device map if any */
//                         if (DeviceMap != NULL)
//                         {
//                             ++DeviceMap->ReferenceCount;
//                         }
//
//                         /* Release the device map lock */
//                         KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//                         /* If we got the device map, we're done! */
//                         if (DeviceMap != NULL)
//                         {
//                             ObDereferenceObject(Token);
//
//                             return DeviceMap;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     LookingForSystem = TRUE;
//                 }
//             }
//         }
//
//         /*
//          * Fall back case of the LUID mapping, make sure there's a
//          * a device map attached to the current process
//          */
//         if (PsGetCurrentProcess()->DeviceMap == NULL &&
//             !NT_SUCCESS(ObpSetCurrentProcessDeviceMap()))
//         {
//             /* We may have failed after we got impersonation token */
//             if (Token != NULL)
//             {
//                 ObDereferenceObject(Token);
//             }
//
//             return NULL;
//         }
//     }
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* If we're looking for system map, use it */
//     if (LookingForSystem)
//     {
//         DeviceMap = ObSystemDeviceMap;
//     }
//     /* Otherwise, use current process device map */
//     else
//     {
//         DeviceMap = PsGetCurrentProcess()->DeviceMap;
//     }
//
//     /* If we got one, reference it */
//     if (DeviceMap != NULL)
//     {
//         ++DeviceMap->ReferenceCount;
//     }
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* We may have impersonation token (if we failed in impersonation branch) */
//     if (Token != NULL)
//     {
//         ObDereferenceObject(Token);
//     }
//
//     /* Return the potentially found device map */
//     return DeviceMap;
// }
//
//
// VOID
// NTAPI
// ObDereferenceDeviceMap(IN PEPROCESS Process)
// {
//     PDEVICE_MAP DeviceMap;
//
//     DPRINT("ObDereferenceDeviceMap()\n");
//
//     /* Get the pointer to this process devicemap and reset it
//        holding the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//     DeviceMap = Process->DeviceMap;
//     Process->DeviceMap = NULL;
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* Continue only if there is a device map */
//     if (DeviceMap != NULL)
//         ObfDereferenceDeviceMap(DeviceMap);
// }
//
//
// VOID
// FASTCALL
// ObfDereferenceDeviceMap(IN PDEVICE_MAP DeviceMap)
// {
//     DPRINT("ObfDereferenceDeviceMap()\n");
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Decrement the reference counter */
//     DeviceMap->ReferenceCount--;
//     DPRINT("ReferenceCount: %lu\n", DeviceMap->ReferenceCount);
//
//     /* Leave, if there are still references to this device map */
//     if (DeviceMap->ReferenceCount != 0)
//     {
//         /* Release the device map lock and leave */
//         KeReleaseGuardedMutex(&ObpDeviceMapLock);
//         return;
//     }
//
//     /* Nobody is referencing it anymore, unlink the DOS directory */
//     DeviceMap->DosDevicesDirectory->DeviceMap = NULL;
//
//     /* Release the devicemap lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* Dereference the DOS Devices Directory and free the Device Map */
//     ObMakeTemporaryObject(DeviceMap->DosDevicesDirectory);
//     ObDereferenceObject(DeviceMap->DosDevicesDirectory);
//     ExFreePoolWithTag(DeviceMap, 'mDbO');
// }
//
//
// VOID
// NTAPI
// ObInheritDeviceMap(IN PEPROCESS Parent,
//                    IN PEPROCESS Process)
// {
//     PDEVICE_MAP DeviceMap;
//
//     DPRINT("ObInheritDeviceMap()\n");
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Get the parent process device map or the system device map */
//     DeviceMap = (Parent != NULL) ? Parent->DeviceMap : ObSystemDeviceMap;
//     if (DeviceMap != NULL)
//     {
//         /* Reference the device map and attach it to the new process */
//         DeviceMap->ReferenceCount++;
//         DPRINT("ReferenceCount: %lu\n", DeviceMap->ReferenceCount);
//
//         Process->DeviceMap = DeviceMap;
//     }
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
// }
//
//
// NTSTATUS
// NTAPI
// ObQueryDeviceMapInformation(
//     _In_opt_ PEPROCESS Process,
//     _Out_ PPROCESS_DEVICEMAP_INFORMATION DeviceMapInfo,
//     _In_ ULONG Flags)
// {
//     PDEVICE_MAP DeviceMap = NULL, GlobalDeviceMap;
//     BOOLEAN Dereference;
//     PROCESS_DEVICEMAP_INFORMATION MapInfo;
//     ULONG BitMask, i;
//     BOOLEAN ReturnAny;
//     NTSTATUS Status;
//
//     /* Validate flags */
//     if (Flags & ~PROCESS_LUID_DOSDEVICES_ONLY)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Dereference = FALSE;
//     /* Do we want to return anything? */
//     ReturnAny = ~Flags & PROCESS_LUID_DOSDEVICES_ONLY;
//
//     /* If LUID mappings are enabled... */
//     if (ObpLUIDDeviceMapsEnabled != 0)
//     {
//         /* Check for process parameter validness */
//         if (Process != NULL && Process != PsGetCurrentProcess())
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* And get the device map */
//         DeviceMap = ObpReferenceDeviceMap();
//     }
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /*
//      * If we had a device map, if because of LUID mappings,
//      * we'll have to dereference it afterwards
//      */
//     if (DeviceMap != NULL)
//     {
//         Dereference = TRUE;
//     }
//     else
//     {
//         /* Get the process device map or the system device map */
//         DeviceMap = (Process != NULL) ? Process->DeviceMap : ObSystemDeviceMap;
//     }
//
//     /* Fail if no device map */
//     if (DeviceMap == NULL)
//     {
//         KeReleaseGuardedMutex(&ObpDeviceMapLock);
//         return STATUS_END_OF_FILE;
//     }
//
//     /* At that point, assume success */
//     Status = STATUS_SUCCESS;
//
//     /* Try to get the global device map if any */
//     GlobalDeviceMap = DeviceMap;
//     if (DeviceMap->GlobalDosDevicesDirectory != NULL)
//     {
//         if (DeviceMap->GlobalDosDevicesDirectory->DeviceMap != NULL)
//         {
//             GlobalDeviceMap = DeviceMap->GlobalDosDevicesDirectory->DeviceMap;
//         }
//     }
//
//     /* Now, setup our device map info, especially drive types */
//     MapInfo.Query.DriveMap = DeviceMap->DriveMap;
//     /* Browse every device */
//     for (i = 0, BitMask = 1; i < 32; ++i, BitMask *= 2)
//     {
//         /* Set the type given current device map */
//         MapInfo.Query.DriveType[i] = DeviceMap->DriveType[i];
//
//         /*
//          * If device is not existing and we're asked to return
//          * more than just LUID mapped, get the entry
//          * from global device map if not remote
//          */
//         if (!(MapInfo.Query.DriveMap & BitMask) && ReturnAny)
//         {
//             if (ObpLUIDDeviceMapsEnabled != 0 ||
//                 (GlobalDeviceMap->DriveType[i] != DOSDEVICE_DRIVE_REMOTE &&
//                  GlobalDeviceMap->DriveType[i] != DOSDEVICE_DRIVE_CALCULATE))
//             {
//                 MapInfo.Query.DriveType[i] = GlobalDeviceMap->DriveType[i];
//                 MapInfo.Query.DriveMap |= BitMask & GlobalDeviceMap->DriveMap;
//             }
//         }
//     }
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     /* Dereference LUID device map */
//     if (Dereference)
//     {
//         ObfDereferenceDeviceMap(DeviceMap);
//     }
//
//     /* Copy back data */
//     _SEH2_TRY
//     {
//         RtlCopyMemory(DeviceMapInfo, &MapInfo, sizeof(PROCESS_DEVICEMAP_INFORMATION));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
//
// ULONG
// NTAPI
// ObIsLUIDDeviceMapsEnabled(VOID)
// {
//     return ObpLUIDDeviceMapsEnabled;
// }
//
//
// #if 0
// NTSTATUS
// NTAPI
// ObIsDosDeviceLocallyMapped(
//     IN ULONG Index,
//     OUT PUCHAR DosDeviceState)
// {
//     /* Check the index */
//     if (Index < 1 || Index > 26)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Get drive mapping status */
//     *DosDeviceState = (ObSystemDeviceMap->DriveMap & (1 << Index)) != 0;
//
//     /* Release the device map lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     return STATUS_SUCCESS;
// }
// #endif
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
