use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLONG, va_list};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PSRBEX_DATA, PSRB_ALLOCATE_ROUTINE, PSTORAGE_REQUEST_BLOCK, PSTOR_ADDRESS, SRBEX_DATA_BIDIRECTIONAL, SRBEX_DATA_IO_INFO, SRBEX_DATA_PNP, SRBEX_DATA_POWER, SRBEX_DATA_SCSI_CDB16, SRBEX_DATA_SCSI_CDB32, SRBEX_DATA_SCSI_CDB_VAR, SRBEX_DATA_WMI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_SIGNATURE, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, STORAGE_REQUEST_BLOCK_VERSION_1, STOR_ADDRESS_TYPE_BTL8, STOR_ADDR_BTL8, STOR_ADDR_BTL8_ADDRESS_LENGTH, _SRBEXDATATYPE as SRBEXDATATYPE, _SRBEX_DATA, _STOR_ADDRESS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::driver_ddk::{DbgPrint};
#[no_mangle]
pub unsafe extern "C" fn DefaultStorageRequestBlockAllocateRoutine(ByteSize: CLONG) -> PVOID {
    ExAllocatePoolWithTag(NonPagedPoolNx, ByteSize as SIZE_T, 0x2B627273u32)
}
#[no_mangle]
pub unsafe extern "C" fn pInitializeStorageRequestBlock(
    Srb: PSTORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    ByteSize: ULONG,
    NumSrbExData: ULONG,
    mut ap: ::core::ffi::VaList,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut address: PSTOR_ADDRESS = core::ptr::null_mut::<_STOR_ADDRESS>();
    let mut srbExData: PSRBEX_DATA = core::ptr::null_mut::<_SRBEX_DATA>();
    let mut offset: ULONG = 0;
    let mut length: ULONG = !0u32;
    let mut type_0: SRBEXDATATYPE = 0;
    let mut srbExDataLength: ULONG = !0u32;
    let mut varLength: ULONG = 0;
    let mut i: ULONG = 0;

    if (ByteSize as usize) < ::core::mem::size_of::<STORAGE_REQUEST_BLOCK>() {
        return STATUS_BUFFER_OVERFLOW;
    }
    ::core::ptr::write_bytes(Srb as *mut u8, 0u8, ByteSize as usize);

    (*Srb).Length = 8 as USHORT;
    (*Srb).Function = (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8;
    (*Srb).Signature = SRB_SIGNATURE;
    (*Srb).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
    (*Srb).SrbLength = ByteSize;
    (*Srb).NumSrbExData = NumSrbExData;

    offset = ::core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
    if NumSrbExData > 0 {
        offset = offset.wrapping_add(
            (NumSrbExData.wrapping_sub(1) as usize).wrapping_mul(::core::mem::size_of::<ULONG>()) as ULONG
        );
        if offset as usize % ::core::mem::size_of::<PVOID>() != 0 {
            offset = offset.wrapping_add(
                (::core::mem::size_of::<PVOID>() - (offset as usize % ::core::mem::size_of::<PVOID>())) as ULONG
            );
        }
    }
    (*Srb).AddressOffset = offset;

    if AddressType as i32 == STORAGE_ADDRESS_TYPE_BTL8 as i32 {
        if ByteSize < offset ||
           (ByteSize as usize) < (offset as usize).wrapping_add(::core::mem::size_of::<STOR_ADDR_BTL8>())
        {
            return STATUS_BUFFER_OVERFLOW;
        }
        address = (Srb as *mut u8).offset(offset as isize) as PSTOR_ADDRESS;
        (*address).Type = STOR_ADDRESS_TYPE_BTL8 as USHORT;
        (*address).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
        offset = offset.wrapping_add(::core::mem::size_of::<STOR_ADDR_BTL8>() as ULONG);
    } else {
        status = STATUS_INVALID_PARAMETER;
    }

    i = 0;
    while i < NumSrbExData && status == STATUS_SUCCESS {
        if ByteSize <= offset {
            status = STATUS_BUFFER_OVERFLOW;
            break;
        }
        srbExData = (Srb as *mut u8).offset(offset as isize) as PSRBEX_DATA;
        *(&raw mut (*Srb).SrbExDataOffset as *mut ULONG).offset(i as isize) = offset;

        type_0 = ap.arg::<SRBEXDATATYPE>();

        match type_0 {
            1 => {
                length = ::core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as ULONG;
                srbExDataLength = SRBEX_DATA_BIDIRECTIONAL_LENGTH;
            }
            64 => {
                length = ::core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB16_LENGTH;
            }
            65 => {
                length = ::core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB32_LENGTH;
            }
            66 => {
                varLength = ap.arg::<ULONG>();
                length = (::core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as usize)
                    .wrapping_add(varLength as usize) as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB_VAR_LENGTH_MIN.wrapping_add(varLength as usize) as ULONG;
            }
            96 => {
                length = ::core::mem::size_of::<SRBEX_DATA_WMI>() as ULONG;
                srbExDataLength = SRBEX_DATA_WMI_LENGTH;
            }
            97 => {
                length = ::core::mem::size_of::<SRBEX_DATA_POWER>() as ULONG;
                srbExDataLength = SRBEX_DATA_POWER_LENGTH;
            }
            98 => {
                length = ::core::mem::size_of::<SRBEX_DATA_PNP>() as ULONG;
                srbExDataLength = SRBEX_DATA_PNP_LENGTH;
            }
            128 => {
                length = ::core::mem::size_of::<SRBEX_DATA_IO_INFO>() as ULONG;
                srbExDataLength = SRBEX_DATA_IO_INFO_LENGTH;
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }

        if status == STATUS_SUCCESS {
            if length == !0u32 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                    b"/home/curi/Projects/os-playground/vibeos/drivers/storage/class/classpnp/srblib.c\0".as_ptr() as *const i8,
                    186i32,
                    b"length != (ULONG)-1\0".as_ptr() as *const i8,
                );
                __int2c();
            }

            if ByteSize < offset.wrapping_add(length) {
                status = STATUS_BUFFER_OVERFLOW;
                break;
            }

            if srbExDataLength == !0u32 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                    b"/home/curi/Projects/os-playground/vibeos/drivers/storage/class/classpnp/srblib.c\0".as_ptr() as *const i8,
                    193i32,
                    b"srbExDataLength != (ULONG)-1\0".as_ptr() as *const i8,
                );
                __int2c();
            }

            (*srbExData).Type = type_0;
            (*srbExData).Length = srbExDataLength;
            offset = offset.wrapping_add(length);
        }
        i = i.wrapping_add(1);
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn InitializeStorageRequestBlock(
    Srb: PSTORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    ByteSize: ULONG,
    NumSrbExData: ULONG,
    args: ...,
) -> NTSTATUS {
    let mut ap: core::ffi::VaListImpl = args.clone();
    pInitializeStorageRequestBlock(Srb, AddressType, ByteSize, NumSrbExData, ap.as_va_list())
}
#[no_mangle]
pub unsafe extern "C" fn CreateStorageRequestBlock(
    Srb: *mut PSTORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    AllocateRoutine: PSRB_ALLOCATE_ROUTINE,
    ByteSize: *mut ULONG,
    NumSrbExData: ULONG,
    ...
) -> NTSTATUS {
    let mut sizeNeeded: ULONG = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as ULONG;
    if NumSrbExData > 0 {
        sizeNeeded = sizeNeeded.wrapping_add(
            ((NumSrbExData - 1) as usize * core::mem::size_of::<ULONG>()) as ULONG,
        );
        let align = core::mem::size_of::<PVOID>();
        if (sizeNeeded as usize) % align != 0 {
            sizeNeeded = sizeNeeded.wrapping_add((align - ((sizeNeeded as usize) % align)) as ULONG);
        }
    }
    let mut status = STATUS_SUCCESS;
    if AddressType as i32 == (STORAGE_ADDRESS_TYPE_BTL8) as i32 {
        sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<STOR_ADDR_BTL8>() as ULONG);
    } else {
        status = STATUS_INVALID_PARAMETER;
    }
    // first variadic pass: sizing
    let mut ap: core::ffi::VaList;
    va_start(&mut ap, NumSrbExData);
    let mut i: ULONG = 0;
    while i < NumSrbExData && status == STATUS_SUCCESS {
        let srbex_type: SRBEXDATATYPE = va_arg(&mut ap);
        match srbex_type {
            SrbExDataTypeBidirectional => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as ULONG);
            }
            SrbExDataTypeScsiCdb16 => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as ULONG);
            }
            SrbExDataTypeScsiCdb32 => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as ULONG);
            }
            SrbExDataTypeScsiCdbVar => {
                let extra: ULONG = va_arg(&mut ap);
                sizeNeeded = sizeNeeded.wrapping_add(
                    core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as ULONG + extra,
                );
            }
            SrbExDataTypeWmi => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_WMI>() as ULONG);
            }
            SrbExDataTypePower => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_POWER>() as ULONG);
            }
            SrbExDataTypePnP => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_PNP>() as ULONG);
            }
            SrbExDataTypeIoInfo => {
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_IO_INFO>() as ULONG);
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        i = i.wrapping_add(1);
    }
    va_end(&mut ap);
    if status == STATUS_SUCCESS {
        if !AllocateRoutine.is_none() {
            *Srb = AllocateRoutine(sizeNeeded as CLONG) as PSTORAGE_REQUEST_BLOCK;
            if (*Srb).is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        if !ByteSize.is_null() {
            *ByteSize = sizeNeeded;
        }
        if !(*Srb).is_null() {
            let mut ap2: core::ffi::VaList;
            va_start(&mut ap2, NumSrbExData);
            status = pInitializeStorageRequestBlock(*Srb, AddressType, sizeNeeded, NumSrbExData, ap2);
            va_end(&mut ap2);
        }
    }
    status
}
