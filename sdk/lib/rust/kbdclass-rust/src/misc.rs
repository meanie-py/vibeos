use crate::kbdclass_h::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation;
use ntkrnl_sys::ddk::{PCUNICODE_STRING, PDEVICE_OBJECT, PIRP, PUNICODE_STRING, _UNICODE_STRING};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let LowerDevice: PDEVICE_OBJECT =
        (*((*DeviceObject).DeviceExtension as PPORT_DEVICE_EXTENSION)).LowerDevice;

    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver(LowerDevice, Irp)
}

pub unsafe fn DuplicateUnicodeString(
    Flags: ULONG,
    SourceString: PCUNICODE_STRING,
    DestinationString: *mut _UNICODE_STRING,
) -> NTSTATUS {
    if SourceString.is_null()
        || DestinationString.is_null()
        || (*SourceString).Length > (*SourceString).MaximumLength
        || ((*SourceString).Length == 0
            && (*SourceString).MaximumLength > 0
            && (*SourceString).Buffer.is_null())
        || Flags == RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING
        || Flags >= 4
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (*SourceString).Length == 0
        && Flags
            != (RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE
                | RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING)
    {
        (*DestinationString).Length = 0;
        (*DestinationString).MaximumLength = 0;
        (*DestinationString).Buffer = core::ptr::null_mut();
    } else {
        let mut DestMaxLength: USHORT = (*SourceString).Length;

        if Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE != 0 {
            DestMaxLength += core::mem::size_of::<u16>() as USHORT;
        }

        (*DestinationString).Buffer =
            ExAllocatePoolWithTag(PagedPool, DestMaxLength as _, CLASS_TAG) as *mut WCHAR;
        if (*DestinationString).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyMemory(
            (*DestinationString).Buffer as *mut core::ffi::c_void,
            (*SourceString).Buffer as *const core::ffi::c_void,
            (*SourceString).Length as usize,
        );
        (*DestinationString).Length = (*SourceString).Length;
        (*DestinationString).MaximumLength = DestMaxLength;

        if Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE != 0 {
            *(*DestinationString).Buffer.add(
                (*DestinationString).Length as usize / core::mem::size_of::<WCHAR>(),
            ) = 0;
        }
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Keyboard class driver
//  * FILE:            drivers/input/kbdclass/misc.c
//  * PURPOSE:         Miscellaneous operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "kbdclass.h"
//
// #include <debug.h>
//
//
// NTSTATUS NTAPI
// ForwardIrpAndForget(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PDEVICE_OBJECT LowerDevice;
//
// 	ASSERT(!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO);
// 	LowerDevice = ((PPORT_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->LowerDevice;
//
// 	IoSkipCurrentIrpStackLocation(Irp);
// 	return IoCallDriver(LowerDevice, Irp);
// }
//
// NTSTATUS
// DuplicateUnicodeString(
// 	IN ULONG Flags,
// 	IN PCUNICODE_STRING SourceString,
// 	OUT PUNICODE_STRING DestinationString)
// {
// 	if (SourceString == NULL || DestinationString == NULL
// 	 || SourceString->Length > SourceString->MaximumLength
// 	 || (SourceString->Length == 0 && SourceString->MaximumLength > 0 && SourceString->Buffer == NULL)
// 	 || Flags == RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING || Flags >= 4)
// 	{
// 		return STATUS_INVALID_PARAMETER;
// 	}
//
//
// 	if ((SourceString->Length == 0)
// 	 && (Flags != (RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE |
// 	               RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING)))
// 	{
// 		DestinationString->Length = 0;
// 		DestinationString->MaximumLength = 0;
// 		DestinationString->Buffer = NULL;
// 	}
// 	else
// 	{
// 		USHORT DestMaxLength = SourceString->Length;
//
// 		if (Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE)
// 			DestMaxLength += sizeof(UNICODE_NULL);
//
// 		DestinationString->Buffer = ExAllocatePoolWithTag(PagedPool, DestMaxLength, CLASS_TAG);
// 		if (DestinationString->Buffer == NULL)
// 			return STATUS_NO_MEMORY;
//
// 		RtlCopyMemory(DestinationString->Buffer, SourceString->Buffer, SourceString->Length);
// 		DestinationString->Length = SourceString->Length;
// 		DestinationString->MaximumLength = DestMaxLength;
//
// 		if (Flags & RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE)
// 			DestinationString->Buffer[DestinationString->Length / sizeof(WCHAR)] = 0;
// 	}
//
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
