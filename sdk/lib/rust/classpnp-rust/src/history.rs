use ntkrnl_sys::rtl::*;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PSRB_HISTORY, PSRB_HISTORY_ITEM, SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT, SCSI_SENSE_ERRORCODE_DESCRIPTOR_DEFERRED, SENSE_DATA, SRB_HISTORY, SRB_HISTORY_ITEM, _CLASS_PRIVATE_FDO_DATA, _FUNCTIONAL_DEVICE_EXTENSION, _SENSE_DATA, _SRB_HISTORY, _SRB_HISTORY_ITEM, _TRANSFER_PACKET};
use ntkrnl_sys::ddk::{BOOLEAN, PCSTR, PVOID, UCHAR, ULONG};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::driver_ddk::{DbgPrint};
#[no_mangle]
pub unsafe extern "C" fn HistoryInitializeRetryLogs(History: PSRB_HISTORY, HistoryCount: ULONG) {
    let tmp_size: ULONG = (HistoryCount as usize)
        .wrapping_mul(core::mem::size_of::<SRB_HISTORY_ITEM>())
        .wrapping_add(
            core::mem::size_of::<SRB_HISTORY>().wrapping_sub(core::mem::size_of::<SRB_HISTORY_ITEM>()),
        ) as ULONG;
    core::ptr::write_bytes(History as *mut u8, 0u8, tmp_size as usize);
    (*History).TotalHistoryCount = HistoryCount;
}
#[no_mangle]
pub unsafe extern "C" fn HistoryLogSendPacket(Pkt: *mut _TRANSFER_PACKET) {
    // assert Pkt->RetryHistory != NULL
    if (*Pkt).RetryHistory.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
            b"history.c\0".as_ptr() as *const core::ffi::c_char,
            50i32,
            b"Pkt->RetryHistory != NULL\0".as_ptr() as *const core::ffi::c_char,
        );
        __int2c();
    }

    let history: *mut _SRB_HISTORY = (*Pkt).RetryHistory;

    // assert UsedHistoryCount <= TotalHistoryCount
    if !((*history).UsedHistoryCount <= (*history).TotalHistoryCount) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
            b"history.c\0".as_ptr() as *const core::ffi::c_char,
            53i32,
            b"history->UsedHistoryCount <= history->TotalHistoryCount\0".as_ptr() as *const core::ffi::c_char,
        );
        __int2c();
    }

    // if full, request class driver to compress history
    if (*history).UsedHistoryCount == (*history).TotalHistoryCount {
        let fdoExtension = (*(*Pkt).Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
        let fdoData = (*fdoExtension).PrivateFdoData as *mut _CLASS_PRIVATE_FDO_DATA;
        if (*fdoData).InterpretSenseInfo.is_null() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                b"history.c\0".as_ptr() as *const core::ffi::c_char,
                60i32,
                b"fdoData->InterpretSenseInfo != NULL\0".as_ptr() as *const core::ffi::c_char,
            );
            __int2c();
        }
        if (*(*fdoData).InterpretSenseInfo).Compress.is_none() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                b"history.c\0".as_ptr() as *const core::ffi::c_char,
                61i32,
                b"fdoData->InterpretSenseInfo->Compress != NULL\0".as_ptr() as *const core::ffi::c_char,
            );
            __int2c();
        }
        let compress_fn = (*(*fdoData).InterpretSenseInfo).Compress
            .expect("non-null function pointer");
        compress_fn((*fdoExtension).DeviceObject, history);

        // assert UsedHistoryCount < TotalHistoryCount after compression
        if !((*history).UsedHistoryCount < (*history).TotalHistoryCount) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                b"history.c\0".as_ptr() as *const core::ffi::c_char,
                63i32,
                b"history->UsedHistoryCount < history->TotalHistoryCount\0".as_ptr() as *const core::ffi::c_char,
            );
            __int2c();
        }
    }

    // final ensure room
    if !((*history).UsedHistoryCount < (*history).TotalHistoryCount) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
            b"history.c\0".as_ptr() as *const core::ffi::c_char,
            67i32,
            b"history->UsedHistoryCount < history->TotalHistoryCount\0".as_ptr() as *const core::ffi::c_char,
        );
        __int2c();
    }

    (*history).UsedHistoryCount = (*history).UsedHistoryCount.wrapping_add(1);

    let item = &raw mut (*history).History as *mut _SRB_HISTORY_ITEM;
    let item = item.offset(
        (((*history).UsedHistoryCount as usize).wrapping_sub(1)) as isize
    );

    // zero the history item
    core::ptr::write_bytes(
        item as *mut u8,
        0,
        core::mem::size_of::<_SRB_HISTORY_ITEM>(),
    );

    // store tick count
    KeQueryTickCount(&raw mut (*item).TickCountSent as *mut i64);
}
#[no_mangle]
pub unsafe extern "C" fn HistoryLogReturnedPacket(Pkt: *mut _TRANSFER_PACKET) {
    if (*Pkt).RetryHistory.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos/drivers/storage/class/classpnp/history.c\0".as_ptr() as PCSTR,
            93i32,
            b"Pkt->RetryHistory != NULL\0".as_ptr() as PCSTR,
        );
        __int2c();
    }

    let history: PSRB_HISTORY = (*Pkt).RetryHistory;

    if !((*history).UsedHistoryCount <= (*history).TotalHistoryCount) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos/drivers/storage/class/classpnp/history.c\0".as_ptr() as PCSTR,
            95i32,
            b"history->UsedHistoryCount <= history->TotalHistoryCount\0".as_ptr() as PCSTR,
        );
        __int2c();
    }

    let item: PSRB_HISTORY_ITEM = ((*history).History.as_mut_ptr())
        .add((*history).UsedHistoryCount as usize - 1);

    KeQueryTickCount(&mut (*item).TickCountCompleted);
    (*item).SrbStatus = (*(*Pkt).Srb).SrbStatus;

    let mut senseInfoBuffer: PVOID = ClasspTransferPacketGetSenseInfoBuffer(Pkt);
    let mut senseInfoBufferLength: UCHAR = ClasspTransferPacketGetSenseInfoBufferLength(Pkt);

    let mut convertedSenseBuffer: SENSE_DATA = core::mem::zeroed();
    let mut validSense: BOOLEAN = 1u8;

    // Inline IsDescriptorSenseDataFormat check
    let byte = *(senseInfoBuffer as *const u8);
    if (byte & 0x7F) as i32 == SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT
        || (byte & 0x7F) as i32 == SCSI_SENSE_ERRORCODE_DESCRIPTOR_DEFERRED
    {
        validSense = ScsiConvertToFixedSenseFormat(
            senseInfoBuffer,
            senseInfoBufferLength,
            &mut convertedSenseBuffer as *mut _SENSE_DATA as PVOID,
            core::mem::size_of::<SENSE_DATA>() as UCHAR,
        );
        if validSense != 0 {
            senseInfoBuffer = &mut convertedSenseBuffer as *mut _SENSE_DATA as PVOID;
            senseInfoBufferLength = core::mem::size_of::<SENSE_DATA>() as UCHAR;
        }
    }

    core::ptr::write_bytes(
        &mut (*item).NormalizedSenseData as *mut _SENSE_DATA as *mut u8,
        0u8,
        core::mem::size_of::<SENSE_DATA>(),
    );

    if validSense != 0 {
        let mut senseSize: UCHAR;
        if ScsiGetTotalSenseByteCountIndicated(
            senseInfoBuffer,
            senseInfoBufferLength,
            &mut senseSize,
        ) == 0
        {
            senseSize = senseInfoBufferLength;
        }

        senseSize = if (senseSize as usize) < core::mem::size_of::<SENSE_DATA>() {
            senseSize
        } else {
            core::mem::size_of::<SENSE_DATA>() as UCHAR
        };

        core::ptr::copy_nonoverlapping(
            senseInfoBuffer as *const u8,
            &mut (*item).NormalizedSenseData as *mut _SENSE_DATA as *mut u8,
            senseSize as usize,
        );
    }
}
