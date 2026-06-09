use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLONG, va_list};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SRBEXDATATYPE as SRBEXDATATYPE};

extern "C" {
    fn NT_ASSERT(condition: i32)
    // va_start/va_end live in the C trampoline (rust_variadic_shim.c); these workers
    // receive the va_list already-started and pull args via the C va_arg helpers
    // (va_arg, like va_start, is a macro and cannot be expressed in stable Rust).
    #[link_name = "classpnp_va_arg_srbexdatatype"]
    fn va_arg_srbexdatatype(ap: *mut va_list) -> SRBEXDATATYPE;
    #[link_name = "classpnp_va_arg_ulong"]
    fn va_arg_ulong(ap: *mut va_list) -> ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn DefaultStorageRequestBlockAllocateRoutine(
    ByteSize: i32,
) -> *mut core::ffi::c_void {
    ExAllocatePoolWithTag(
        0x200, // NonPagedPoolNx
        ByteSize as usize,
        u32::from_le_bytes(*b"+brs"),
    )
}
#[no_mangle]
pub unsafe extern "C" fn pInitializeStorageRequestBlock(
    Srb: *mut STORAGE_REQUEST_BLOCK,
    AddressType: u16,
    ByteSize: u32,
    NumSrbExData: u32,
    mut ap: va_list,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let address: *mut STOR_ADDRESS;
    let srb_ex_data: *mut SRBEX_DATA;
    let mut offset: u32;
    let mut length: u32 = u32::MAX;
    let mut type_: SRBEXDATATYPE;
    let mut srb_ex_data_length: u32 = u32::MAX;
    let mut var_length: u32;
    let mut i: u32;

    if ByteSize < core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32 {
        return STATUS_BUFFER_OVERFLOW;
    }

    core::ptr::write_bytes(Srb as *mut u8, 0, ByteSize as usize);

    (*Srb).Length = core::mem::offset_of!(STORAGE_REQUEST_BLOCK, Signature) as u32;
    (*Srb).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
    (*Srb).Signature = SRB_SIGNATURE;
    (*Srb).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
    (*Srb).SrbLength = ByteSize;
    (*Srb).NumSrbExData = NumSrbExData;

    offset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
    if NumSrbExData > 0 {
        offset += (NumSrbExData - 1) * core::mem::size_of::<u32>() as u32;

        let pointer_size = core::mem::size_of::<*mut core::ffi::c_void>() as u32;
        if offset % pointer_size != 0 {
            offset += pointer_size - (offset % pointer_size);
        }
    }
    (*Srb).AddressOffset = offset;

    if AddressType == STORAGE_ADDRESS_TYPE_BTL8 {
        if ByteSize < offset || ByteSize < offset + core::mem::size_of::<STOR_ADDR_BTL8>() as u32 {
            return STATUS_BUFFER_OVERFLOW;
        }
        address = (Srb as *mut u8).add(offset as usize) as *mut STOR_ADDRESS;
        (*address).Type = STOR_ADDRESS_TYPE_BTL8;
        (*address).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
        offset += core::mem::size_of::<STOR_ADDR_BTL8>() as u32;
    } else {
        status = STATUS_INVALID_PARAMETER;
    }

    i = 0;
    while i < NumSrbExData && status == STATUS_SUCCESS {
        if ByteSize <= offset {
            status = STATUS_BUFFER_OVERFLOW;
            break;
        }
        srb_ex_data = (Srb as *mut u8).add(offset as usize) as *mut SRBEX_DATA;

        let offset_ptr = &raw mut (*Srb).SrbExDataOffset as *mut u32;
        *offset_ptr.add(i as usize) = offset;

        // va_arg(ap, SRBEXDATATYPE)
        let ap_p = &raw mut ap as *mut va_list as *mut u8;
        type_ = core::ptr::read_unaligned(ap_p as *const SRBEXDATATYPE);
        ap = ap_p.add(core::mem::size_of::<SRBEXDATATYPE>()) as va_list;

        match type_ {
            SrbExDataTypeBidirectional => {
                length = core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as u32;
                srb_ex_data_length = SRBEX_DATA_BIDIRECTIONAL_LENGTH;
            }
            SrbExDataTypeScsiCdb16 => {
                length = core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as u32;
                srb_ex_data_length = SRBEX_DATA_SCSI_CDB16_LENGTH;
            }
            SrbExDataTypeScsiCdb32 => {
                length = core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as u32;
                srb_ex_data_length = SRBEX_DATA_SCSI_CDB32_LENGTH;
            }
            SrbExDataTypeScsiCdbVar => {
                // va_arg(ap, ULONG)
                let ap_p = &raw mut ap as *mut va_list as *mut u8;
                var_length = core::ptr::read_unaligned(ap_p as *const u32);
                ap = ap_p.add(core::mem::size_of::<u32>()) as va_list;

                length = core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as u32 + var_length;
                srb_ex_data_length = SRBEX_DATA_SCSI_CDB_VAR_LENGTH_MIN + var_length;
            }
            SrbExDataTypeWmi => {
                length = core::mem::size_of::<SRBEX_DATA_WMI>() as u32;
                srb_ex_data_length = SRBEX_DATA_WMI_LENGTH;
            }
            SrbExDataTypePower => {
                length = core::mem::size_of::<SRBEX_DATA_POWER>() as u32;
                srb_ex_data_length = SRBEX_DATA_POWER_LENGTH;
            }
            SrbExDataTypePnP => {
                length = core::mem::size_of::<SRBEX_DATA_PNP>() as u32;
                srb_ex_data_length = SRBEX_DATA_PNP_LENGTH;
            }
            SrbExDataTypeIoInfo => {
                length = core::mem::size_of::<SRBEX_DATA_IO_INFO>() as u32;
                srb_ex_data_length = SRBEX_DATA_IO_INFO_LENGTH;
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }

        if status == STATUS_SUCCESS {
            NT_ASSERT((length != u32::MAX) as i32);

            if ByteSize < offset + length {
                status = STATUS_BUFFER_OVERFLOW;
                break;
            }

            NT_ASSERT((srb_ex_data_length != u32::MAX) as i32);

            (*srb_ex_data).Type = type_;
            (*srb_ex_data).Length = srb_ex_data_length;
            offset += length;
        }

        i += 1;
    }

    status
}
// The public variadic entry `InitializeStorageRequestBlock(..., ...)` lives in the C
// trampoline (rust_variadic_shim.c); it does va_start and calls the worker
// `pInitializeStorageRequestBlock` above. Nothing to define here.

// Worker for the public variadic `CreateStorageRequestBlock(..., ...)`, whose va_start
// trampoline is in rust_variadic_shim.c. Receives the already-started `ap`.
#[no_mangle]
pub unsafe extern "C" fn classpnp_CreateStorageRequestBlockWorker(
    Srb: *mut *mut STORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    AllocateRoutine: PSRB_ALLOCATE_ROUTINE,
    ByteSize: *mut ULONG,
    NumSrbExData: ULONG,
    mut ap: va_list,
) -> NTSTATUS {
    let mut sizeNeeded: ULONG = 0;
    let mut i: ULONG;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    sizeNeeded = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
    if NumSrbExData > 0 {
        sizeNeeded += (NumSrbExData - 1) * (core::mem::size_of::<ULONG>() as ULONG);
        if (sizeNeeded % (core::mem::size_of::<*mut core::ffi::c_void>() as ULONG)) != 0 {
            sizeNeeded += (core::mem::size_of::<*mut core::ffi::c_void>() as ULONG)
                - (sizeNeeded % (core::mem::size_of::<*mut core::ffi::c_void>() as ULONG));
        }
    }

    if AddressType == STORAGE_ADDRESS_TYPE_BTL8 {
        sizeNeeded += core::mem::size_of::<STOR_ADDR_BTL8>() as ULONG;
    } else {
        status = STATUS_INVALID_PARAMETER;
    }

    // ap is already va_start'd by the C trampoline. The C original va_start's twice
    // (once to size, once to fill); on all 3 Windows targets va_list is a plain pointer,
    // so we snapshot it and restore to re-traverse instead of a second va_start.
    let ap_start = ap;
    i = 0;
    while i < NumSrbExData && status == STATUS_SUCCESS {
        match va_arg_srbexdatatype(&mut ap) {
            SRBEXDATATYPE::SrbExDataTypeBidirectional => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypeScsiCdb16 => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypeScsiCdb32 => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypeScsiCdbVar => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as ULONG
                    + va_arg_ulong(&mut ap);
            }
            SRBEXDATATYPE::SrbExDataTypeWmi => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_WMI>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypePower => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_POWER>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypePnP => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_PNP>() as ULONG;
            }
            SRBEXDATATYPE::SrbExDataTypeIoInfo => {
                sizeNeeded += core::mem::size_of::<SRBEX_DATA_IO_INFO>() as ULONG;
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        i += 1;
    }

    if status == STATUS_SUCCESS {
        if !AllocateRoutine.is_null() {
            *Srb = AllocateRoutine(sizeNeeded);
            if (*Srb).is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        if !ByteSize.is_null() {
            *ByteSize = sizeNeeded;
        }

        if !(*Srb).is_null() {
            ap = ap_start; // re-traverse the same args (see snapshot above)
            // pInitializeStorageRequestBlock will set the SrbLength field
            status = pInitializeStorageRequestBlock(*Srb, AddressType, sizeNeeded, NumSrbExData, ap);
        }
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation 2010
//
// Module Name:
//
//     srblib.c
//
// Abstract:
//
//     Header for SRB utility functions
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
//
// #include "classp.h"
//
// PVOID
// DefaultStorageRequestBlockAllocateRoutine(
//      CLONG ByteSize
//     )
// /*++
//
// Routine Description:
//
//     Default allocation routine.
//
// Arguments:
//
//     ByteSize - SRB size in bytes.
//
// Return Value:
//
//     Pointer to the SRB buffer. NULL if SRB buffer could not be allocated.
//
// --*/
// {
//     return ExAllocatePoolWithTag(NonPagedPoolNx, ByteSize, '+brs');
// }
//
//
// NTSTATUS
// pInitializeStorageRequestBlock(
//      PSTORAGE_REQUEST_BLOCK Srb,
//      USHORT AddressType,
//      ULONG ByteSize,
//      ULONG NumSrbExData,
//      va_list ap
//     )
// /*++
//
// Routine Description:
//
//     Initialize a STORAGE_REQUEST_BLOCK.
//
// Arguments:
//
//     Srb - Pointer to STORAGE_REQUEST_BLOCK to initialize.
//
//     AddressType - Storage address type.
//
//     ByteSize - STORAGE_REQUEST_BLOCK size in bytes.
//
//     NumSrbExData - Number of SRB extended data.
//
//     ap - Variable argument list matching the SRB extended data in the
//          STORAGE_REQUEST_BLOCK.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//     PSTOR_ADDRESS address;
//     PSRBEX_DATA srbExData;
//     ULONG offset;
//     ULONG length = (ULONG)-1;
//     SRBEXDATATYPE type;
//     ULONG srbExDataLength = (ULONG)-1;
//     ULONG varLength;
//     ULONG i;
//
//     if (ByteSize < sizeof(STORAGE_REQUEST_BLOCK)) {
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     RtlZeroMemory(Srb, ByteSize);
//
//     Srb->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//     Srb->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//     Srb->Signature = SRB_SIGNATURE;
//     Srb->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//     Srb->SrbLength = ByteSize;
//     Srb->NumSrbExData = NumSrbExData;
//
//     offset = sizeof(STORAGE_REQUEST_BLOCK);
//     if (NumSrbExData > 0) {
//         offset += ((NumSrbExData - 1) * sizeof(ULONG));
//
//         // Ensure offset is pointer type aligned
//         if (offset % sizeof(PVOID)) {
//             offset += (sizeof(PVOID) - (offset % sizeof(PVOID)));
//         }
//     }
//     Srb->AddressOffset = offset;
//
//     if (AddressType == STORAGE_ADDRESS_TYPE_BTL8)
//     {
//         if ((ByteSize < offset) ||
//             (ByteSize < (offset + sizeof(STOR_ADDR_BTL8)))) {
//             return STATUS_BUFFER_OVERFLOW;
//         }
//         address = (PSTOR_ADDRESS)((PUCHAR)Srb + offset);
//         address->Type = STOR_ADDRESS_TYPE_BTL8;
//         address->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//         offset += sizeof(STOR_ADDR_BTL8);
//     } else
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     for (i = 0; i < NumSrbExData && status == STATUS_SUCCESS; i++)
//     {
//         if (ByteSize <= offset) {
//             status = STATUS_BUFFER_OVERFLOW;
//             break;
//         }
//         srbExData = (PSRBEX_DATA)((PUCHAR)Srb + offset);
//         Srb->SrbExDataOffset[i] = offset;
//
//         type = va_arg(ap, SRBEXDATATYPE);
//
//         switch (type)
//         {
//             case SrbExDataTypeBidirectional:
//                 length = sizeof(SRBEX_DATA_BIDIRECTIONAL);
//                 srbExDataLength = SRBEX_DATA_BIDIRECTIONAL_LENGTH;
//                 break;
//             case SrbExDataTypeScsiCdb16:
//                 length = sizeof(SRBEX_DATA_SCSI_CDB16);
//                 srbExDataLength = SRBEX_DATA_SCSI_CDB16_LENGTH;
//                 break;
//             case SrbExDataTypeScsiCdb32:
//                 length = sizeof(SRBEX_DATA_SCSI_CDB32);
//                 srbExDataLength = SRBEX_DATA_SCSI_CDB32_LENGTH;
//                 break;
//             case SrbExDataTypeScsiCdbVar:
//                 varLength = va_arg(ap, ULONG);
//                 length = sizeof(SRBEX_DATA_SCSI_CDB_VAR) + varLength;
//                 srbExDataLength = SRBEX_DATA_SCSI_CDB_VAR_LENGTH_MIN + varLength;
//                 break;
//             case SrbExDataTypeWmi:
//                 length = sizeof(SRBEX_DATA_WMI);
//                 srbExDataLength = SRBEX_DATA_WMI_LENGTH;
//                 break;
//             case SrbExDataTypePower:
//                 length = sizeof(SRBEX_DATA_POWER);
//                 srbExDataLength = SRBEX_DATA_POWER_LENGTH;
//                 break;
//             case SrbExDataTypePnP:
//                 length = sizeof(SRBEX_DATA_PNP);
//                 srbExDataLength = SRBEX_DATA_PNP_LENGTH;
//                 break;
//             case SrbExDataTypeIoInfo:
//                 length = sizeof(SRBEX_DATA_IO_INFO);
//                 srbExDataLength = SRBEX_DATA_IO_INFO_LENGTH;
//                 break;
//             default:
//                 status = STATUS_INVALID_PARAMETER;
//                 break;
//         }
//
//         if (status == STATUS_SUCCESS)
//         {
//             NT_ASSERT(length != (ULONG)-1 as i32)
//
//             if (ByteSize < (offset + length)) {
//                 status = STATUS_BUFFER_OVERFLOW;
//                 break;
//             }
//
//             NT_ASSERT(srbExDataLength != (ULONG)-1 as i32)
//
//             srbExData->Type = type;
//             srbExData->Length = srbExDataLength;
//             offset += length;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// InitializeStorageRequestBlock(
//      PSTORAGE_REQUEST_BLOCK Srb,
//      USHORT AddressType,
//      ULONG ByteSize,
//      ULONG NumSrbExData,
//     ...
//     )
// /*++
//
// Routine Description:
//
//     Initialize an extended SRB.
//
// Arguments:
//
//     Srb - Pointer to SRB buffer to initialize.
//
//     AddressType - Storage address type.
//
//     ByteSize - STORAGE_REQUEST_BLOCK size in bytes.
//
//     NumSrbExData - Number of SRB extended data.
//
//     ... - Variable argument list matching the SRB extended data in the
//           STORAGE_REQUEST_BLOCK.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status;
//     va_list ap;
//     va_start(ap, NumSrbExData);
//     status = pInitializeStorageRequestBlock(Srb, AddressType, ByteSize, NumSrbExData, ap);
//     va_end(ap);
//     return status;
// }
//
//
//
// NTSTATUS
// CreateStorageRequestBlock(
//      PSTORAGE_REQUEST_BLOCK *Srb,
//      USHORT AddressType,
//      PSRB_ALLOCATE_ROUTINE AllocateRoutine,
//      ULONG *ByteSize,
//      ULONG NumSrbExData,
//     ...
//     )
// /*++
//
// Routine Description:
//
//     Create an extended SRB.
//
// Arguments:
//
//     Srb - Pointer to buffer to store SRB pointer.
//
//     AddressType - Storage address type.
//
//     AllocateRoutine - Buffer allocation function (optional).
//
//     ByteSize - Pointer to ULONG to store size of SRB in bytes (optional).
//
//     NumSrbExData - Number of SRB extended data.
//
//     ... - Variable argument list matching the SRB extended data in the
//           STORAGE_REQUEST_BLOCK.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     ULONG sizeNeeded = 0;
//     va_list ap;
//     ULONG i;
//     NTSTATUS status = STATUS_SUCCESS;
//
//     // Ensure SrbExData offsets are pointer type aligned
//     sizeNeeded = sizeof(STORAGE_REQUEST_BLOCK);
//     if (NumSrbExData > 0) {
//         sizeNeeded += ((NumSrbExData - 1) * sizeof(ULONG));
//         if (sizeNeeded % sizeof(PVOID)) {
//             sizeNeeded += (sizeof(PVOID) - (sizeNeeded % sizeof(PVOID)));
//         }
//     }
//
//     if (AddressType == STORAGE_ADDRESS_TYPE_BTL8)
//     {
//         sizeNeeded += sizeof(STOR_ADDR_BTL8);
//     } else
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     va_start(ap, NumSrbExData);
//
//     for (i = 0; i < NumSrbExData && status == STATUS_SUCCESS; i++)
//     {
//         switch (va_arg(ap, SRBEXDATATYPE))
//         {
//             case SrbExDataTypeBidirectional:
//                 sizeNeeded += sizeof(SRBEX_DATA_BIDIRECTIONAL);
//                 break;
//             case SrbExDataTypeScsiCdb16:
//                 sizeNeeded += sizeof(SRBEX_DATA_SCSI_CDB16);
//                 break;
//             case SrbExDataTypeScsiCdb32:
//                 sizeNeeded += sizeof(SRBEX_DATA_SCSI_CDB32);
//                 break;
//             case SrbExDataTypeScsiCdbVar:
//                 sizeNeeded += sizeof(SRBEX_DATA_SCSI_CDB_VAR) + va_arg(ap, ULONG);
//                 break;
//             case SrbExDataTypeWmi:
//                 sizeNeeded += sizeof(SRBEX_DATA_WMI);
//                 break;
//             case SrbExDataTypePower:
//                 sizeNeeded += sizeof(SRBEX_DATA_POWER);
//                 break;
//             case SrbExDataTypePnP:
//                 sizeNeeded += sizeof(SRBEX_DATA_PNP);
//                 break;
//             case SrbExDataTypeIoInfo:
//                 sizeNeeded += sizeof(SRBEX_DATA_IO_INFO);
//                 break;
//             default:
//                 status = STATUS_INVALID_PARAMETER;
//                 break;
//         }
//     }
//     va_end(ap);
//
//     if (status == STATUS_SUCCESS)
//     {
//         if (AllocateRoutine)
//         {
//             *Srb = AllocateRoutine(sizeNeeded);
//             if (*Srb == NULL)
//             {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//         }
//
//         if (ByteSize != NULL)
//         {
//             *ByteSize = sizeNeeded;
//         }
//
//         if (*Srb)
//         {
//             va_start(ap, NumSrbExData);
// #ifdef _MSC_VER
//             #pragma prefast(suppress:26015, "pInitializeStorageRequestBlock will set the SrbLength field")
// #endif
//             status = pInitializeStorageRequestBlock(*Srb, AddressType, sizeNeeded, NumSrbExData, ap);
//             va_end(ap);
//         }
//
//     }
//
//     return status;
// }
//
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
