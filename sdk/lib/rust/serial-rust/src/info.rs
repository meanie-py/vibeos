use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, PDEVICE_OBJECT, PFILE_POSITION_INFORMATION, PFILE_STANDARD_INFORMATION, PIO_STACK_LOCATION, PIRP};
// IoGetCurrentIrpStackLocation → crate::glue (tier1 FORCEINLINE)
// RtlAssert → crate::glue inline no-op
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: u32, Level: u32, Format: *const i8, ... ) -> u32;
}
unsafe extern "system" {
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SerialQueryInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;

    let mut Stack: PIO_STACK_LOCATION;
    let mut SystemBuffer: *mut core::ffi::c_void;
    let mut BufferLength: u32;
    let mut Information: usize = 0;
    let mut Status: NTSTATUS;

    Stack = IoGetCurrentIrpStackLocation(Irp);
    SystemBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    BufferLength = (*Stack).Parameters.QueryFile.Length;

    match (*Stack).Parameters.QueryFile.FileInformationClass as u32 {
        x if x == FileStandardInformation as u32 => {
            let StandardInfo: PFILE_STANDARD_INFORMATION =
                SystemBuffer as PFILE_STANDARD_INFORMATION;

            DbgPrintEx(
                DPFLTR_SERIAL_ID,
                DPFLTR_TRACE_LEVEL,
                b"(%s:%d) IRP_MJ_QUERY_INFORMATION / FileStandardInformation\n\0" as *const u8
                    as *const i8,
                b"info.c\0" as *const u8 as *const i8,
                35,
            );
            if (BufferLength as usize) < core::mem::size_of::<FILE_STANDARD_INFORMATION>() {
                Status = STATUS_BUFFER_OVERFLOW;
            } else if StandardInfo.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                (*StandardInfo).AllocationSize.QuadPart = 0;
                (*StandardInfo).EndOfFile.QuadPart = 0;
                (*StandardInfo).Directory = 0;
                (*StandardInfo).NumberOfLinks = 0;
                (*StandardInfo).DeletePending = 0;
                Information = core::mem::size_of::<FILE_STANDARD_INFORMATION>();
                Status = STATUS_SUCCESS;
            }
        }
        x if x == FilePositionInformation as u32 => {
            let PositionInfo: PFILE_POSITION_INFORMATION =
                SystemBuffer as PFILE_POSITION_INFORMATION;

            if PositionInfo.is_null() {
                RtlAssert(
                    b"PositionInfo\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                    b"info.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                    56,
                    core::ptr::null_mut(),
                );
            }

            DbgPrintEx(
                DPFLTR_SERIAL_ID,
                DPFLTR_TRACE_LEVEL,
                b"(%s:%d) IRP_MJ_QUERY_INFORMATION / FilePositionInformation\n\0" as *const u8
                    as *const i8,
                b"info.c\0" as *const u8 as *const i8,
                58,
            );
            if (BufferLength as usize) < core::mem::size_of::<FILE_POSITION_INFORMATION>() {
                Status = STATUS_BUFFER_OVERFLOW;
            } else if PositionInfo.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                (*PositionInfo).CurrentByteOffset.QuadPart = 0;
                Information = core::mem::size_of::<FILE_POSITION_INFORMATION>();
                Status = STATUS_SUCCESS;
            }
        }
        _ => {
            DbgPrintEx(
                DPFLTR_SERIAL_ID,
                DPFLTR_TRACE_LEVEL,
                b"(%s:%d) IRP_MJ_QUERY_INFORMATION: Unexpected file information class 0x%02x\n\0"
                    as *const u8 as *const i8,
                b"info.c\0" as *const u8 as *const i8,
                73,
                (*Stack).Parameters.QueryFile.FileInformationClass as u32,
            );
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/info.c
//  * PURPOSE:         Serial IRP_MJ_QUERY_INFORMATION operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialQueryInformation(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PVOID SystemBuffer;
// 	ULONG BufferLength;
// 	ULONG_PTR Information = 0;
// 	NTSTATUS Status;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	SystemBuffer = Irp->AssociatedIrp.SystemBuffer;
// 	BufferLength = Stack->Parameters.QueryFile.Length;
//
// 	switch (Stack->Parameters.QueryFile.FileInformationClass)
// 	{
// 		case FileStandardInformation:
// 		{
// 			PFILE_STANDARD_INFORMATION StandardInfo = (PFILE_STANDARD_INFORMATION)SystemBuffer;
//
// 			TRACE_!(SERIAL, "IRP_MJ_QUERY_INFORMATION / FileStandardInformation\n");
// 			if (BufferLength < sizeof(FILE_STANDARD_INFORMATION))
// 				Status = STATUS_BUFFER_OVERFLOW;
// 			else if (!StandardInfo)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				StandardInfo->AllocationSize.QuadPart = 0;
// 				StandardInfo->EndOfFile.QuadPart = 0;
// 				StandardInfo->Directory = FALSE;
// 				StandardInfo->NumberOfLinks = 0;
// 				StandardInfo->DeletePending = FALSE; /* FIXME: should be TRUE sometimes */
// 				Information = sizeof(FILE_STANDARD_INFORMATION);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case FilePositionInformation:
// 		{
// 			PFILE_POSITION_INFORMATION PositionInfo = (PFILE_POSITION_INFORMATION)SystemBuffer;
//
// 			ASSERT(PositionInfo);
//
// 			TRACE_!(SERIAL, "IRP_MJ_QUERY_INFORMATION / FilePositionInformation\n");
// 			if (BufferLength < sizeof(FILE_POSITION_INFORMATION))
// 				Status = STATUS_BUFFER_OVERFLOW;
// 			else if (!PositionInfo)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				PositionInfo->CurrentByteOffset.QuadPart = 0;
// 				Information = sizeof(FILE_POSITION_INFORMATION);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		default:
// 		{
// 			TRACE_!(SERIAL, "IRP_MJ_QUERY_INFORMATION: Unexpected file information class 0x%02x\n", Stack->Parameters.QueryFile.FileInformationClass);
// 			return ForwardIrpAndForget(DeviceObject, Irp);
// 		}
// 	}
//
// 	Irp->IoStatus.Information = Information;
// 	Irp->IoStatus.Status = Status;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
