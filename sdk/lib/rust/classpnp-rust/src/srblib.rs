use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLONG};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PSRBEX_DATA, PSRB_ALLOCATE_ROUTINE, PSTORAGE_REQUEST_BLOCK, PSTOR_ADDRESS, SRBEX_DATA_BIDIRECTIONAL, SRBEX_DATA_IO_INFO, SRBEX_DATA_PNP, SRBEX_DATA_POWER, SRBEX_DATA_SCSI_CDB16, SRBEX_DATA_SCSI_CDB32, SRBEX_DATA_SCSI_CDB_VAR, SRBEX_DATA_WMI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_SIGNATURE, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, STORAGE_REQUEST_BLOCK_VERSION_1, STOR_ADDRESS_TYPE_BTL8, STOR_ADDR_BTL8, STOR_ADDR_BTL8_ADDRESS_LENGTH, _SRBEXDATATYPE as SRBEXDATATYPE, _SRBEX_DATA, _STOR_ADDRESS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::driver_ddk::{DbgPrint};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{SRB_FUNCTION_EXECUTE_SCSI};
// ---------------------------------------------------------------------------
// srbhelper.h getters/setters (faithful hand-translation of the __inline accessors;
// bindgen can't emit inline fns and the LLM pass left them undefined). Each dispatches
// on Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK: extended STORAGE_REQUEST_BLOCK vs
// the classic SCSI_REQUEST_BLOCK layout. ReactOS uses the classic SRB in practice but
// the extended branch is preserved for fidelity.
use crate::types::{PSCSI_REQUEST_BLOCK, PSRBEX_DATA_SCSI_CDB16, PSRBEX_DATA_SCSI_CDB32,
                   PSRBEX_DATA_SCSI_CDB_VAR};

#[inline]
unsafe fn srb_is_ex(srb: PSTORAGE_REQUEST_BLOCK) -> bool {
    (*srb).Function as u32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetDataTransferLength(Srb: PVOID) -> ULONG {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).DataTransferLength }
    else { (*(Srb as PSCSI_REQUEST_BLOCK)).DataTransferLength }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetDataBuffer(Srb: PVOID) -> PVOID {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).DataBuffer }
    else { (*(Srb as PSCSI_REQUEST_BLOCK)).DataBuffer }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetSrbFlags(Srb: PVOID) -> ULONG {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).SrbFlags }
    else { (*(Srb as PSCSI_REQUEST_BLOCK)).SrbFlags }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetOriginalRequest(Srb: PVOID) -> PVOID {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).OriginalRequest }
    else { (*(Srb as PSCSI_REQUEST_BLOCK)).OriginalRequest }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetSystemStatus(Srb: PVOID) -> ULONG {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) { (*srb).SystemStatus }
    else { (*(Srb as PSCSI_REQUEST_BLOCK)).__bindgen_anon_1.bindgen_union_field }
}

#[no_mangle]
pub unsafe extern "C" fn SrbSetScsiStatus(Srb: PVOID, ScsiStatus: UCHAR) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) {
        // extended SRB: write ScsiStatus into the EXECUTE_SCSI SrbExData entry (mirrors
        // srbhelper.h SrbSetScsiData, narrowed to the ScsiStatus output we need).
        if (*srb).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI {
            let offsets = &raw const (*srb).SrbExDataOffset as *const ULONG;
            let mut i: ULONG = 0;
            while i < (*srb).NumSrbExData {
                let off = *offsets.offset(i as isize);
                if (off as usize) >= ::core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
                    && off < (*srb).SrbLength {
                    let d = (srb as *mut u8).offset(off as isize) as PSRBEX_DATA;
                    match (*d).Type {
                        64 => (*(d as PSRBEX_DATA_SCSI_CDB16)).ScsiStatus = ScsiStatus,
                        65 => (*(d as PSRBEX_DATA_SCSI_CDB32)).ScsiStatus = ScsiStatus,
                        66 => (*(d as PSRBEX_DATA_SCSI_CDB_VAR)).ScsiStatus = ScsiStatus,
                        _ => {}
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    } else {
        (*(Srb as PSCSI_REQUEST_BLOCK)).ScsiStatus = ScsiStatus;
    }
}

#[no_mangle]
pub unsafe extern "C" fn SrbSetQueueSortKey(Srb: PVOID, QueueSortKey: ULONG) {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if !srb_is_ex(srb) {
        (*(Srb as PSCSI_REQUEST_BLOCK)).__bindgen_anon_1.bindgen_union_field = QueueSortKey;
    }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetSenseInfoBuffer(Srb: PVOID) -> PVOID {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) {
        let mut p: PVOID = core::ptr::null_mut();
        SrbGetScsiData(srb, core::ptr::null_mut(), core::ptr::null_mut(),
                       core::ptr::null_mut(), &mut p as *mut PVOID, core::ptr::null_mut());
        p
    } else {
        (*(Srb as PSCSI_REQUEST_BLOCK)).SenseInfoBuffer
    }
}

#[no_mangle]
pub unsafe extern "C" fn SrbGetSenseInfoBufferLength(Srb: PVOID) -> UCHAR {
    let srb = Srb as PSTORAGE_REQUEST_BLOCK;
    if srb_is_ex(srb) {
        let mut len: UCHAR = 0;
        SrbGetScsiData(srb, core::ptr::null_mut(), core::ptr::null_mut(),
                       core::ptr::null_mut(), core::ptr::null_mut(), &mut len as *mut UCHAR);
        len
    } else {
        (*(Srb as PSCSI_REQUEST_BLOCK)).SenseInfoBufferLength
    }
}

/// Faithful translation of srbhelper.h SrbGetScsiData — walks the SrbExData entries of an
/// extended SRB to extract the CDB/ScsiStatus/SenseInfo for the EXECUTE_SCSI request.
#[no_mangle]
pub unsafe extern "C" fn SrbGetScsiData(
    SrbEx: PSTORAGE_REQUEST_BLOCK,
    CdbLength8: PUCHAR,
    CdbLength32: PULONG,
    ScsiStatus: PUCHAR,
    SenseInfoBuffer: *mut PVOID,
    SenseInfoBufferLength: PUCHAR,
) -> PVOID {
    let mut cdb: PVOID = core::ptr::null_mut();
    if !((*SrbEx).Function as u32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK
        && (*SrbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI) {
        return cdb;
    }
    let offsets = &raw const (*SrbEx).SrbExDataOffset as *const ULONG;
    let mut i: ULONG = 0;
    while i < (*SrbEx).NumSrbExData {
        let off = *offsets.offset(i as isize);
        if (off as usize) < ::core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
            || off >= (*SrbEx).SrbLength {
            i = i.wrapping_add(1);
            continue;
        }
        let srb_ex_data = (SrbEx as *mut u8).offset(off as isize) as PSRBEX_DATA;
        match (*srb_ex_data).Type {
            64 => { // SrbExDataTypeScsiCdb16
                if off as usize + ::core::mem::size_of::<crate::types::SRBEX_DATA_SCSI_CDB16>()
                    <= (*SrbEx).SrbLength as usize {
                    let d = srb_ex_data as PSRBEX_DATA_SCSI_CDB16;
                    if !CdbLength8.is_null() { *CdbLength8 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw const (*d).Cdb) as PVOID; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            65 => { // SrbExDataTypeScsiCdb32
                if off as usize + ::core::mem::size_of::<crate::types::SRBEX_DATA_SCSI_CDB32>()
                    <= (*SrbEx).SrbLength as usize {
                    let d = srb_ex_data as PSRBEX_DATA_SCSI_CDB32;
                    if !CdbLength8.is_null() { *CdbLength8 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw const (*d).Cdb) as PVOID; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            66 => { // SrbExDataTypeScsiCdbVar
                if off as usize + ::core::mem::size_of::<crate::types::SRBEX_DATA_SCSI_CDB_VAR>()
                    <= (*SrbEx).SrbLength as usize {
                    let d = srb_ex_data as PSRBEX_DATA_SCSI_CDB_VAR;
                    if !CdbLength32.is_null() { *CdbLength32 = (*d).CdbLength; }
                    if (*d).CdbLength > 0 { cdb = (&raw const (*d).Cdb) as PVOID; }
                    if !ScsiStatus.is_null() { *ScsiStatus = (*d).ScsiStatus; }
                    if !SenseInfoBuffer.is_null() { *SenseInfoBuffer = (*d).SenseInfoBuffer; }
                    if !SenseInfoBufferLength.is_null() { *SenseInfoBufferLength = (*d).SenseInfoBufferLength; }
                }
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    cdb
}

#[no_mangle]
pub unsafe extern "C" fn DefaultStorageRequestBlockAllocateRoutine(ByteSize: CLONG) -> PVOID {
    ExAllocatePoolWithTag(NonPagedPoolNx, ByteSize as SIZE_T, 0x2B627273u32)
}
// SRBEX size constants (from srb.h; sizeof(ULONG)=4, sizeof(UCHAR)=1, sizeof(PVOID) is arch-dependent)
const SRBEX_DATA_BIDIRECTIONAL_LENGTH: usize = 2 * core::mem::size_of::<ULONG>() + core::mem::size_of::<PVOID>();
const SRBEX_DATA_SCSI_CDB16_LENGTH: usize = 20 * core::mem::size_of::<UCHAR>() + core::mem::size_of::<ULONG>() + core::mem::size_of::<PVOID>();
const SRBEX_DATA_SCSI_CDB32_LENGTH: usize = 36 * core::mem::size_of::<UCHAR>() + core::mem::size_of::<ULONG>() + core::mem::size_of::<PVOID>();
const SRBEX_DATA_SCSI_CDB_VAR_LENGTH_MIN: usize = 4 * core::mem::size_of::<UCHAR>() + 3 * core::mem::size_of::<ULONG>() + core::mem::size_of::<PVOID>();
const SRBEX_DATA_WMI_LENGTH: usize = 4 * core::mem::size_of::<UCHAR>() + core::mem::size_of::<ULONG>() + core::mem::size_of::<PVOID>();
const SRBEX_DATA_POWER_LENGTH: usize = 4 * core::mem::size_of::<UCHAR>() + 2 * core::mem::size_of::<u32>();  // STOR_DEVICE_POWER_STATE + STOR_POWER_ACTION = 2 u32
const SRBEX_DATA_PNP_LENGTH: usize = 4 * core::mem::size_of::<UCHAR>() + core::mem::size_of::<u32>() + 2 * core::mem::size_of::<ULONG>(); // STOR_PNP_ACTION + 2×ULONG
const SRBEX_DATA_IO_INFO_LENGTH: usize = 5 * core::mem::size_of::<ULONG>() + 4 * core::mem::size_of::<UCHAR>();

// NOTE: pInitializeStorageRequestBlock / InitializeStorageRequestBlock /
// CreateStorageRequestBlock require the `c_variadic` feature (core::ffi::VaList
// and `...` in fn definitions). This feature is nightly-only on Rust 1.92.
// The function bodies are preserved intact below so no translation work is lost;
// enabling the feature in the crate root (lib.rs) would make them compile.
// NEEDS SHARED SUBSTRATE: c_variadic feature gate in crate root.
#[no_mangle]
pub unsafe extern "C" fn pInitializeStorageRequestBlock(
    Srb: PSTORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    ByteSize: ULONG,
    NumSrbExData: ULONG,
    // De-variadicized: classpnp only ever builds 0 or 1 SrbExData entry (NumSrbExData in {0,1},
    // always SrbExDataTypeScsiCdb16, never ScsiCdbVar) — so the single tag replaces the C va_list.
    SrbExType: SRBEXDATATYPE,
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

        type_0 = SrbExType;

        match type_0 {
            1 => {
                length = ::core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as ULONG;
                srbExDataLength = SRBEX_DATA_BIDIRECTIONAL_LENGTH as ULONG;
            }
            64 => {
                length = ::core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB16_LENGTH as ULONG;
            }
            65 => {
                length = ::core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB32_LENGTH as ULONG;
            }
            66 => {
                varLength = 0; // ScsiCdbVar (var-length CDB) is never used by classpnp
                length = (::core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as usize)
                    .wrapping_add(varLength as usize) as ULONG;
                srbExDataLength = SRBEX_DATA_SCSI_CDB_VAR_LENGTH_MIN.wrapping_add(varLength as usize) as ULONG;
            }
            96 => {
                length = ::core::mem::size_of::<SRBEX_DATA_WMI>() as ULONG;
                srbExDataLength = SRBEX_DATA_WMI_LENGTH as ULONG;
            }
            97 => {
                length = ::core::mem::size_of::<SRBEX_DATA_POWER>() as ULONG;
                srbExDataLength = SRBEX_DATA_POWER_LENGTH as ULONG;
            }
            98 => {
                length = ::core::mem::size_of::<SRBEX_DATA_PNP>() as ULONG;
                srbExDataLength = SRBEX_DATA_PNP_LENGTH as ULONG;
            }
            128 => {
                length = ::core::mem::size_of::<SRBEX_DATA_IO_INFO>() as ULONG;
                srbExDataLength = SRBEX_DATA_IO_INFO_LENGTH as ULONG;
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }

        if status == STATUS_SUCCESS {
            if length == !0u32 {
                DbgPrint(
                    (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
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
                    (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
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
    SrbExType: SRBEXDATATYPE,
) -> NTSTATUS {
    pInitializeStorageRequestBlock(Srb, AddressType, ByteSize, NumSrbExData, SrbExType)
}
#[no_mangle]
pub unsafe extern "C" fn CreateStorageRequestBlock(
    Srb: *mut PSTORAGE_REQUEST_BLOCK,
    AddressType: USHORT,
    AllocateRoutine: PSRB_ALLOCATE_ROUTINE,
    ByteSize: *mut ULONG,
    NumSrbExData: ULONG,
    SrbExType: SRBEXDATATYPE,
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
    // De-variadicized sizing pass: classpnp passes NumSrbExData in {0,1} with a single SrbExType
    // tag (always ScsiCdb16, never ScsiCdbVar). The loop adds the one entry's size when present.
    let mut i: ULONG = 0;
    while i < NumSrbExData && status == STATUS_SUCCESS {
        match SrbExType {
            1 => { // SrbExDataTypeBidirectional
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_BIDIRECTIONAL>() as ULONG);
            }
            64 => { // SrbExDataTypeScsiCdb16
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>() as ULONG);
            }
            65 => { // SrbExDataTypeScsiCdb32
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB32>() as ULONG);
            }
            66 => { // SrbExDataTypeScsiCdbVar (var length unused by classpnp)
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_SCSI_CDB_VAR>() as ULONG);
            }
            96 => { // SrbExDataTypeWmi
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_WMI>() as ULONG);
            }
            97 => { // SrbExDataTypePower
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_POWER>() as ULONG);
            }
            98 => { // SrbExDataTypePnP
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_PNP>() as ULONG);
            }
            128 => { // SrbExDataTypeIoInfo
                sizeNeeded = sizeNeeded.wrapping_add(core::mem::size_of::<SRBEX_DATA_IO_INFO>() as ULONG);
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        i = i.wrapping_add(1);
    }
    if status == STATUS_SUCCESS {
        if !AllocateRoutine.is_none() {
            *Srb = AllocateRoutine.unwrap()(sizeNeeded as CLONG) as PSTORAGE_REQUEST_BLOCK;
            if (*Srb).is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        if !ByteSize.is_null() {
            *ByteSize = sizeNeeded;
        }
        if !(*Srb).is_null() {
            status = pInitializeStorageRequestBlock(*Srb, AddressType, sizeNeeded, NumSrbExData, SrbExType);
        }
    }
    status
}
