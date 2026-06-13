//! Leaf helpers + small internal SRB builders from atapi.c (the functions defined
//! at the bottom of the C file). Pure data manipulation — no SCSIPORT port I/O.
//!
//! All keep their C `NTAPI` (stdcall on i386, cdecl on amd64/arm64) ABI via
//! `extern "system"`, matching the C declarations; they are internal (not exports)
//! and called directly from the device/IO-core module.

use core::mem::size_of;
use ntkrnl_sys::ddk::{CHAR, LONG, PCHAR, PUCHAR, PVOID, UCHAR, ULONG, USHORT};

use crate::types::{
    MECHANICAL_STATUS_INFORMATION_HEADER, PCDB, PHW_DEVICE_EXTENSION, PSCSI_REQUEST_BLOCK,
    SCSIOP_MECHANISM_STATUS, SCSIOP_REQUEST_SENSE, SCSI_REQUEST_BLOCK, SENSE_DATA,
    SRB_FLAGS_DATA_IN, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI,
};

/// atapi.c `AtapiStringCmp` — case-insensitive, bounded compare; returns
/// `first - last` at the first mismatch (after lower-casing), else 0.
pub unsafe extern "system" fn AtapiStringCmp(
    mut FirstStr: PCHAR,
    mut SecondStr: PCHAR,
    mut Count: ULONG,
) -> LONG {
    let mut first: UCHAR;
    let mut last: UCHAR;

    if Count != 0 {
        loop {
            // Get next char.
            first = *FirstStr as UCHAR;
            FirstStr = FirstStr.add(1);
            last = *SecondStr as UCHAR;
            SecondStr = SecondStr.add(1);

            if first != last {
                // If no match, try lower-casing.
                if first >= b'A' && first <= b'Z' {
                    first = first - b'A' + b'a';
                }
                if last >= b'A' && last <= b'Z' {
                    last = last - b'A' + b'a';
                }
                if first != last {
                    // No match.
                    return first as LONG - last as LONG;
                }
            }

            Count -= 1;
            if !(Count != 0 && first != 0) {
                break;
            }
        }
    }

    0
}

/// atapi.c `AtapiZeroMemory` — byte-fill `Count` bytes of `Buffer` with 0.
pub unsafe extern "system" fn AtapiZeroMemory(Buffer: PUCHAR, Count: ULONG) {
    let mut i: ULONG = 0;
    while i < Count {
        *Buffer.add(i as usize) = 0;
        i += 1;
    }
}

/// atapi.c `AtapiHexToString` — format low 16 bits of `Value` as 4 hex digits at
/// `*Buffer`, NUL-terminate, advancing nothing (digits written then reversed).
pub unsafe extern "system" fn AtapiHexToString(mut Value: ULONG, Buffer: *mut PCHAR) {
    let mut string: PCHAR = *Buffer;
    let mut firstdig: PCHAR = string;

    let mut i: ULONG = 0;
    while i < 4 {
        let digval: USHORT = (Value % 16) as USHORT;
        Value /= 16;

        // Convert to ascii and store (digits land reversed).
        if digval > 9 {
            *string = (digval - 10 + b'a' as USHORT) as CHAR;
        } else {
            *string = (digval + b'0' as USHORT) as CHAR;
        }
        string = string.add(1);
        i += 1;
    }

    // *string-- = '\0';
    *string = 0;
    string = string.offset(-1);

    // Reverse the digits.
    loop {
        let temp: CHAR = *string;
        *string = *firstdig;
        *firstdig = temp;
        string = string.offset(-1);
        firstdig = firstdig.add(1);
        if !(firstdig < string) {
            break;
        }
    }
}

/// atapi.c `BuildMechanismStatusSrb` — fill the device extension's internal SRB
/// with a 6-byte MECHANISM STATUS CDB targeting `MechStatusData`.
pub unsafe extern "system" fn BuildMechanismStatusSrb(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
    TargetId: ULONG,
) -> PSCSI_REQUEST_BLOCK {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let srb: PSCSI_REQUEST_BLOCK = &raw mut (*deviceExtension).InternalSrb;

    AtapiZeroMemory(srb as PUCHAR, size_of::<SCSI_REQUEST_BLOCK>() as ULONG);

    (*srb).PathId = PathId as UCHAR;
    (*srb).TargetId = TargetId as UCHAR;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*srb).Length = size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    // Disable synchronous negotiation.
    (*srb).SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

    // Timeout (2 seconds in the original comment; value 4).
    (*srb).TimeOutValue = 4;

    (*srb).CdbLength = 6;
    (*srb).DataBuffer = &raw mut (*deviceExtension).MechStatusData as PVOID;
    (*srb).DataTransferLength = size_of::<MECHANICAL_STATUS_INFORMATION_HEADER>() as ULONG;

    let cdb: PCDB = (*srb).Cdb.as_mut_ptr() as PCDB;
    (*cdb).MECH_STATUS.as_mut().OperationCode = SCSIOP_MECHANISM_STATUS as UCHAR;
    (*cdb).MECH_STATUS.as_mut().AllocationLength[1] =
        size_of::<MECHANICAL_STATUS_INFORMATION_HEADER>() as UCHAR;

    srb
}

/// atapi.c `BuildRequestSenseSrb` — fill the internal SRB with a 6-byte REQUEST
/// SENSE CDB targeting `MechStatusSense`.
pub unsafe extern "system" fn BuildRequestSenseSrb(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
    TargetId: ULONG,
) -> PSCSI_REQUEST_BLOCK {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let srb: PSCSI_REQUEST_BLOCK = &raw mut (*deviceExtension).InternalSrb;

    AtapiZeroMemory(srb as PUCHAR, size_of::<SCSI_REQUEST_BLOCK>() as ULONG);

    (*srb).PathId = PathId as UCHAR;
    (*srb).TargetId = TargetId as UCHAR;
    (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
    (*srb).Length = size_of::<SCSI_REQUEST_BLOCK>() as USHORT;

    // Disable synchronous negotiation.
    (*srb).SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

    // Timeout.
    (*srb).TimeOutValue = 4;

    (*srb).CdbLength = 6;
    (*srb).DataBuffer = &raw mut (*deviceExtension).MechStatusSense as PVOID;
    (*srb).DataTransferLength = size_of::<SENSE_DATA>() as ULONG;

    let cdb: PCDB = (*srb).Cdb.as_mut_ptr() as PCDB;
    (*cdb).CDB6INQUIRY.as_mut().OperationCode = SCSIOP_REQUEST_SENSE as UCHAR;
    (*cdb).CDB6INQUIRY.as_mut().AllocationLength = size_of::<SENSE_DATA>() as UCHAR;

    srb
}
