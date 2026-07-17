use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExAcquireResourceSharedLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, PCUNICODE_STRING, PUNICODE_STRING, PWCH, RtlCopyUnicodeString, PCSTR};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// FltObjectReference/Dereference are defined `extern "system"` in crate::Object;
// import the real defs. RtlAssert is the crate::glue no-op. DbgPrint stays the
// variadic cdecl ntoskrnl export.
use crate::Object::{FltObjectReference, FltObjectDereference};
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG (no value, skipped)

#[no_mangle]
pub unsafe extern "system" fn FltGetVolumeProperties(
    Volume: PFLT_VOLUME,
    VolumeProperties: PFLT_VOLUME_PROPERTIES,
    VolumePropertiesLength: ULONG,
    LengthReturned: PULONG,
) -> NTSTATUS {
    let mut BufferRequired: ULONG;
    let mut BytesWritten: ULONG;
    let mut Ptr: PCHAR;
    let Status: NTSTATUS;

    /* Calculate the required buffer size */
    BufferRequired = (core::mem::size_of::<FLT_VOLUME_PROPERTIES>() as usize)
        .wrapping_add((*Volume).CDODriverName.Length as usize)
        .wrapping_add((*Volume).DeviceName.Length as usize)
        .wrapping_add((*Volume).CDODeviceName.Length as usize) as ULONG;

    /* If we don't have enough buffer to fill in the fixed struct, return with the required size */
    if (VolumePropertiesLength as usize) < core::mem::size_of::<FLT_VOLUME_PROPERTIES>() {
        *LengthReturned = BufferRequired;
        return STATUS_BUFFER_TOO_SMALL;
    }

    /* Clear out the buffer */
    RtlZeroMemory(
        VolumeProperties as *mut core::ffi::c_void,
        core::mem::size_of::<FLT_VOLUME_PROPERTIES>(),
    );

    /* Fill in the fixed data */
    (*VolumeProperties).DeviceType = (*(*Volume).DeviceObject).DeviceType;
    (*VolumeProperties).DeviceObjectFlags = (*(*Volume).DeviceObject).Flags;
    (*VolumeProperties).AlignmentRequirement = (*(*Volume).DeviceObject).AlignmentRequirement;
    (*VolumeProperties).SectorSize = (*(*Volume).DeviceObject).SectorSize;
    if !(*Volume).DiskDeviceObject.is_null() {
        (*VolumeProperties).DeviceCharacteristics = (*(*Volume).DiskDeviceObject).Characteristics;
    } else {
        (*VolumeProperties).DeviceCharacteristics = (*(*Volume).DeviceObject).Characteristics;
    }

    /* So far we've written the fixed struct data */
    BytesWritten = core::mem::size_of::<FLT_VOLUME_PROPERTIES>() as ULONG;
    Ptr = VolumeProperties.offset(1) as PCHAR;

    /* Make sure we have enough room to add the dynamic data */
    if VolumePropertiesLength >= BufferRequired {
        /* Add the FS device name */
        (*VolumeProperties).FileSystemDeviceName.Length = 0;
        (*VolumeProperties).FileSystemDeviceName.MaximumLength = (*Volume).CDODeviceName.Length;
        (*VolumeProperties).FileSystemDeviceName.Buffer = Ptr as PWCH;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*VolumeProperties).FileSystemDeviceName),
            core::ptr::addr_of_mut!((*Volume).CDODeviceName) as PCUNICODE_STRING,
        );
        Ptr = Ptr.offset((*VolumeProperties).FileSystemDeviceName.Length as isize);

        /* Add the driver name */
        (*VolumeProperties).FileSystemDriverName.Length = 0;
        (*VolumeProperties).FileSystemDriverName.MaximumLength = (*Volume).CDODriverName.Length;
        (*VolumeProperties).FileSystemDriverName.Buffer = Ptr as PWCH;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*VolumeProperties).FileSystemDriverName),
            core::ptr::addr_of_mut!((*Volume).CDODriverName) as PCUNICODE_STRING,
        );
        Ptr = Ptr.offset((*VolumeProperties).FileSystemDriverName.Length as isize);

        /* Add the volume name */
        (*VolumeProperties).RealDeviceName.Length = 0;
        (*VolumeProperties).RealDeviceName.MaximumLength = (*Volume).DeviceName.Length;
        (*VolumeProperties).RealDeviceName.Buffer = Ptr as PWCH;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*VolumeProperties).RealDeviceName),
            core::ptr::addr_of_mut!((*Volume).DeviceName) as PCUNICODE_STRING,
        );

        BytesWritten = BufferRequired;

        Status = STATUS_SUCCESS;
    } else {
        Status = STATUS_BUFFER_OVERFLOW;
    }

    /* Set the number of bytes we wrote and return */
    *LengthReturned = BytesWritten;
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FltEnumerateVolumes(
    Filter: PFLT_FILTER,
    VolumeList: *mut PFLT_VOLUME,
    VolumeListSize: ULONG,
    NumberVolumesReturned: PULONG,
) -> NTSTATUS {
    const APC_LEVEL: ::core::ffi::c_int = 1;

    let mut i: ULONG;
    let mut Frame: PFLTP_FRAME;
    let mut Volume: PFLT_VOLUME;
    let mut ListEntry: PLIST_ENTRY;
    let mut NumberOfVolumes: ULONG = 0 as ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Volume.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                125 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    Frame = (*Filter).Frame;

    /* Lock the attached volumes list */
    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(
        core::ptr::addr_of_mut!((*Frame).AttachedVolumes.rLock),
        1 as BOOLEAN,
    );

    /* If it's not empty */
    if !IsListEmpty(core::ptr::addr_of_mut!((*Frame).AttachedVolumes.rList)) {
        /* Browse every entry */
        ListEntry = (*Frame).AttachedVolumes.rList.Flink as PLIST_ENTRY;
        while ListEntry != core::ptr::addr_of_mut!((*Frame).AttachedVolumes.rList) {
            /* Get the volume */
            Volume = (ListEntry as *mut u8).wrapping_sub(
                core::mem::offset_of!(FLT_VOLUME, Base.PrimaryLink),
            ) as *mut FLT_VOLUME as PFLT_VOLUME;

            /* If there's still room in the output buffer */
            if NumberOfVolumes < VolumeListSize {
                /* Reference the volume and return it */
                FltObjectReference(Volume as PVOID);
                *VolumeList.offset(NumberOfVolumes as isize) = Volume;
            }

            /* We returned one more volume */
            NumberOfVolumes = NumberOfVolumes.wrapping_add(1);

            ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
        }
    }

    /* Release the list */
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*Frame).AttachedVolumes.rLock));
    KeLeaveCriticalRegion();

    /* If we want to return more volumes than we can */
    if NumberOfVolumes > VolumeListSize {
        /* We will clear output */
        i = 0 as ULONG;
        while i < VolumeListSize {
            FltObjectDereference(*VolumeList.offset(i as isize) as PVOID);
            *VolumeList.offset(i as isize) = core::ptr::null_mut();
            i = i.wrapping_add(1);
        }

        /* And set failure status */
        Status = STATUS_BUFFER_TOO_SMALL;
    }

    /* Always return the max amount of volumes we want to return */
    *NumberVolumesReturned = NumberOfVolumes as ::core::ffi::c_ulong;

    /* Done */
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FltDetachVolume(
    mut Filter: PFLT_FILTER,
    mut Volume: PFLT_VOLUME,
    mut InstanceName: PCUNICODE_STRING,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"FltDetachVolume\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/filters/fltmgr/Volume.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        189 as ::core::ffi::c_int,
    );
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn FltAttachVolume(
    mut Filter: PFLT_FILTER,
    mut Volume: PFLT_VOLUME,
    mut InstanceName: PCUNICODE_STRING,
    mut RetInstance: *mut PFLT_INSTANCE,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"FltAttachVolume\0".as_ptr() as *const i8,
        b"drivers/filters/fltmgr/Volume.c\0".as_ptr() as *const i8,
        201 as i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn FltGetVolumeName(
    Volume: PFLT_VOLUME,
    VolumeName: PUNICODE_STRING,
    BufferSizeNeeded: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if VolumeName.is_null() {
        if BufferSizeNeeded.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        *BufferSizeNeeded = (*Volume).DeviceName.Length as u32;
        return STATUS_BUFFER_TOO_SMALL;
    }

    Status = STATUS_BUFFER_TOO_SMALL;

    if !BufferSizeNeeded.is_null() {
        *BufferSizeNeeded = (*Volume).DeviceName.Length as u32;
    }

    (*VolumeName).Length = 0 as u16;

    if (*VolumeName).MaximumLength as i32 >= (*Volume).DeviceName.Length as i32 {
        RtlCopyUnicodeString(
            VolumeName,
            core::ptr::addr_of_mut!((*Volume).DeviceName) as PCUNICODE_STRING,
        );
        Status = STATUS_SUCCESS;
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
// * FILE:            drivers/filters/fltmgr/Context.c
// * PURPOSE:         Contains routines for the volume
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
//
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
// NTSTATUS
// FLTAPI
// FltGetVolumeProperties(
//      PFLT_VOLUME Volume,
//      PFLT_VOLUME_PROPERTIES VolumeProperties,
//      ULONG VolumePropertiesLength,
//      PULONG LengthReturned
// )
// {
//     ULONG BufferRequired;
//     ULONG BytesWritten;
//     PCHAR Ptr;
//     NTSTATUS Status;
//
//     /* Calculate the required buffer size */
//     BufferRequired = sizeof(FLT_VOLUME_PROPERTIES) +
//                      Volume->CDODriverName.Length +
//                      Volume->DeviceName.Length +
//                      Volume->CDODeviceName.Length;
//
//     /* If we don't have enough buffer to fill in the fixed struct, return with the required size */
//     if (VolumePropertiesLength < sizeof(FLT_VOLUME_PROPERTIES))
//     {
//         *LengthReturned = BufferRequired;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Clear out the buffer */
//     RtlZeroMemory(VolumeProperties, sizeof(FLT_VOLUME_PROPERTIES));
//
//     /* Fill in the fixed data */
//     VolumeProperties->DeviceType = Volume->DeviceObject->DeviceType;
//     VolumeProperties->DeviceObjectFlags = Volume->DeviceObject->Flags;
//     VolumeProperties->AlignmentRequirement = Volume->DeviceObject->AlignmentRequirement;
//     VolumeProperties->SectorSize = Volume->DeviceObject->SectorSize;
//     if (Volume->DiskDeviceObject)
//     {
//         VolumeProperties->DeviceCharacteristics = Volume->DiskDeviceObject->Characteristics;
//     }
//     else
//     {
//         VolumeProperties->DeviceCharacteristics = Volume->DeviceObject->Characteristics;
//     }
//
//     /* So far we've written the fixed struct data */
//     BytesWritten = sizeof(FLT_VOLUME_PROPERTIES);
//     Ptr = (PCHAR)(VolumeProperties + 1);
//
//     /* Make sure we have enough room to add the dynamic data */
//     if (VolumePropertiesLength >= BufferRequired)
//     {
//         /* Add the FS device name */
//         VolumeProperties->FileSystemDeviceName.Length = 0;
//         VolumeProperties->FileSystemDeviceName.MaximumLength = Volume->CDODeviceName.Length;
//         VolumeProperties->FileSystemDeviceName.Buffer = (PWCH)Ptr;
//         RtlCopyUnicodeString(&VolumeProperties->FileSystemDeviceName, &Volume->CDODeviceName);
//         Ptr += VolumeProperties->FileSystemDeviceName.Length;
//
//         /* Add the driver name */
//         VolumeProperties->FileSystemDriverName.Length = 0;
//         VolumeProperties->FileSystemDriverName.MaximumLength = Volume->CDODriverName.Length;
//         VolumeProperties->FileSystemDriverName.Buffer = (PWCH)Ptr;
//         RtlCopyUnicodeString(&VolumeProperties->FileSystemDriverName, &Volume->CDODriverName);
//         Ptr += VolumeProperties->FileSystemDriverName.Length;
//
//         /* Add the volume name */
//         VolumeProperties->RealDeviceName.Length = 0;
//         VolumeProperties->RealDeviceName.MaximumLength = Volume->DeviceName.Length;
//         VolumeProperties->RealDeviceName.Buffer = (PWCH)Ptr;
//         RtlCopyUnicodeString(&VolumeProperties->RealDeviceName, &Volume->DeviceName);
//
//         BytesWritten = BufferRequired;
//
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Set the number of bytes we wrote and return */
//     *LengthReturned = BytesWritten;
//     return Status;
// }
//
//
// NTSTATUS
// FLTAPI
// FltEnumerateVolumes(
//      PFLT_FILTER Filter,
//      PFLT_VOLUME *VolumeList,
//      ULONG VolumeListSize,
//      PULONG NumberVolumesReturned)
// {
//     ULONG i;
//     PFLTP_FRAME Frame;
//     PFLT_VOLUME Volume;
//     PLIST_ENTRY ListEntry;
//     ULONG NumberOfVolumes = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     Frame = Filter->Frame;
//
//     /* Lock the attached volumes list */
//     KeEnterCriticalRegion();
//     ExAcquireResourceSharedLite(&Frame->AttachedVolumes.rLock, TRUE);
//
//     /* If it's not empty */
//     if (!IsListEmpty(&Frame->AttachedVolumes.rList))
//     {
//         /* Browse every entry */
//         for (ListEntry = Frame->AttachedVolumes.rList.Flink;
//              ListEntry != &Frame->AttachedVolumes.rList;
//              ListEntry = ListEntry->Flink)
//         {
//             /* Get the volume */
//             Volume = CONTAINING_RECORD(ListEntry, FLT_VOLUME, Base.PrimaryLink);
//
//             /* If there's still room in the output buffer */
//             if (NumberOfVolumes < VolumeListSize)
//             {
//                 /* Reference the volume and return it */
//                 FltObjectReference(Volume);
//                 VolumeList[NumberOfVolumes] = Volume;
//             }
//
//             /* We returned one more volume */
//             ++NumberOfVolumes;
//         }
//     }
//
//     /* Release the list */
//     ExReleaseResourceLite(&Frame->AttachedVolumes.rLock);
//     KeLeaveCriticalRegion();
//
//     /* If we want to return more volumes than we can */
//     if (NumberOfVolumes > VolumeListSize)
//     {
//         /* We will clear output */
//         for (i = 0; i < VolumeListSize; ++i)
//         {
//             FltObjectDereference(VolumeList[i]);
//             VolumeList[i] = NULL;
//         }
//
//         /* And set failure status */
//         Status = STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Always return the max amount of volumes we want to return */
//     *NumberVolumesReturned = NumberOfVolumes;
//
//     /* Done */
//     return Status;
// }
//
// NTSTATUS
// FLTAPI
// FltDetachVolume(
//      PFLT_FILTER Filter,
//      PFLT_VOLUME Volume,
//      PCUNICODE_STRING InstanceName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// FLTAPI
// FltAttachVolume(
//      PFLT_FILTER Filter,
//      PFLT_VOLUME Volume,
//      PCUNICODE_STRING InstanceName,
//      PFLT_INSTANCE *RetInstance)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// FLTAPI
// FltGetVolumeName(
//      PFLT_VOLUME Volume,
//      PUNICODE_STRING VolumeName,
//      PULONG BufferSizeNeeded)
// {
//     NTSTATUS Status;
//
//     /* Check if caller just probes for size */
//     if (VolumeName == NULL)
//     {
//         /* Totally broken call */
//         if (BufferSizeNeeded == NULL)
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Return the appropriate size and quit */
//         *BufferSizeNeeded = Volume->DeviceName.Length;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* We have an output buffer! Assume it's too small */
//     Status = STATUS_BUFFER_TOO_SMALL;
//
//     /* If we have output size, fill it */
//     if (BufferSizeNeeded != NULL)
//     {
//         *BufferSizeNeeded = Volume->DeviceName.Length;
//     }
//
//     /* Init that we didn't return a thing */
//     VolumeName->Length = 0;
//
//     /* If we have enough room, copy and return success */
//     if (VolumeName->MaximumLength >= Volume->DeviceName.Length)
//     {
//         RtlCopyUnicodeString(VolumeName, &Volume->DeviceName);
//         Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
