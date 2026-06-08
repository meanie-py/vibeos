use crate::types::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;

extern "system" {
    fn NT_ASSERT(condition: i32);
    fn ClasspTransferPacketGetSenseInfoBuffer(Pkt: *mut TRANSFER_PACKET) -> *mut core::ffi::c_void;
    fn ClasspTransferPacketGetSenseInfoBufferLength(Pkt: *mut TRANSFER_PACKET) -> u8;
    fn IsDescriptorSenseDataFormat(buffer: *mut core::ffi::c_void) -> u8;
    fn ScsiConvertToFixedSenseFormat( senseInfoBuffer: *mut core::ffi::c_void, senseInfoBufferLength: u8, convertedSenseBuffer: *mut core::ffi::c_void, size: u32, ) -> u8;
    fn ScsiGetTotalSenseByteCountIndicated( senseInfoBuffer: *mut core::ffi::c_void, senseInfoBufferLength: u8, senseSize: *mut u8, ) -> u8;
    fn min(a: u8, b: u8) -> u8;
}
#[no_mangle]
pub unsafe extern "C" fn HistoryInitializeRetryLogs(
    History: *mut SRB_HISTORY,
    HistoryCount: u32,
) {
    let mut tmpSize: u32 = (HistoryCount as usize * core::mem::size_of::<SRB_HISTORY_ITEM>()) as u32;
    tmpSize += (core::mem::size_of::<SRB_HISTORY>() - core::mem::size_of::<SRB_HISTORY_ITEM>()) as u32;
    core::ptr::write_bytes(History, 0u8, tmpSize as usize);
    (*History).TotalHistoryCount = HistoryCount;
}
#[no_mangle]
pub unsafe extern "C" fn HistoryLogSendPacket(Pkt: *mut TRANSFER_PACKET) {
    let history: *mut SRB_HISTORY;
    let item: *mut SRB_HISTORY_ITEM;

    NT_ASSERT((!(*Pkt).RetryHistory.is_null()) as i32);
    history = (*Pkt).RetryHistory;
    // sending a packet implies a new history unit is to be used.
    NT_ASSERT(((*history).UsedHistoryCount <= (*history).TotalHistoryCount) as i32);

    if (*history).UsedHistoryCount == (*history).TotalHistoryCount {
        let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = (*(*Pkt).Fdo).DeviceExtension;
        let fdoData: *mut CLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
        NT_ASSERT((!(*fdoData).InterpretSenseInfo.is_null()) as i32);
        NT_ASSERT((!((*(*fdoData).InterpretSenseInfo).Compress).is_null()) as i32);
        ((*(*fdoData).InterpretSenseInfo).Compress)((*fdoExtension).DeviceObject, history);
        NT_ASSERT(((*history).UsedHistoryCount < (*history).TotalHistoryCount) as i32);
    }

    // thus, since we are about to increment the count, it must now be less...
    NT_ASSERT(((*history).UsedHistoryCount < (*history).TotalHistoryCount) as i32);

    // increment the number of history units in use
    (*history).UsedHistoryCount += 1;

    // determine index to use
    let base: *mut SRB_HISTORY_ITEM =
        core::ptr::addr_of_mut!((*history).History) as *mut SRB_HISTORY_ITEM;
    item = base.offset(((*history).UsedHistoryCount - 1) as isize);

    // zero out the history item
    RtlZeroMemory(
        item as *mut core::ffi::c_void,
        core::mem::size_of::<SRB_HISTORY_ITEM>(),
    );

    // Query the tick count and store in the history
    KeQueryTickCount(core::ptr::addr_of_mut!((*item).TickCountSent));
    return;
}
#[no_mangle]
pub unsafe extern "C" fn HistoryLogReturnedPacket(Pkt: *mut TRANSFER_PACKET) {
    NT_ASSERT((!(*Pkt).RetryHistory.is_null()) as i32);

    let history = (*Pkt).RetryHistory;

    NT_ASSERT(((*history).UsedHistoryCount <= (*history).TotalHistoryCount) as i32);

    let index = ((*history).UsedHistoryCount as usize).wrapping_sub(1);
    let item_base_ptr = (history as *mut u8).add(core::mem::offset_of!(SRB_HISTORY, History));
    let item = &mut *((item_base_ptr as *mut SRB_HISTORY_ITEM).add(index));

    KeQueryTickCount(&mut item.TickCountCompleted as *mut i64);

    item.SrbStatus = (*(*Pkt).Srb).SrbStatus;

    let mut senseInfoBuffer = ClasspTransferPacketGetSenseInfoBuffer(Pkt);
    let mut senseInfoBufferLength = ClasspTransferPacketGetSenseInfoBufferLength(Pkt);

    let mut convertedSenseBuffer: SENSE_DATA = core::mem::zeroed();
    let mut validSense: u8 = 1;

    if IsDescriptorSenseDataFormat(senseInfoBuffer) != 0 {
        validSense = ScsiConvertToFixedSenseFormat(
            senseInfoBuffer,
            senseInfoBufferLength,
            &mut convertedSenseBuffer as *mut SENSE_DATA as *mut core::ffi::c_void,
            core::mem::size_of::<SENSE_DATA>() as u32,
        );

        if validSense != 0 {
            senseInfoBuffer = &mut convertedSenseBuffer as *mut SENSE_DATA as *mut core::ffi::c_void;
            senseInfoBufferLength = core::mem::size_of::<SENSE_DATA>() as u8;
        }
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(item.NormalizedSenseData) as *mut core::ffi::c_void,
        core::mem::size_of_val(&item.NormalizedSenseData),
    );

    if validSense != 0 {
        let mut senseSize: u8 = 0;
        if ScsiGetTotalSenseByteCountIndicated(
            senseInfoBuffer,
            senseInfoBufferLength,
            &mut senseSize,
        ) == 0
        {
            senseSize = senseInfoBufferLength;
        }

        senseSize = min(
            senseSize,
            core::mem::size_of_val(&item.NormalizedSenseData) as u8,
        );

        RtlCopyMemory(
            core::ptr::addr_of_mut!(item.NormalizedSenseData) as *mut core::ffi::c_void,
            senseInfoBuffer,
            senseSize as usize,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
//
// Copyright (C) Microsoft Corporation, 1991 - 1999
//
// Module Name:
//
//     history.c
//
// Abstract:
//
//     Packet history routines for CLASSPNP
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
// #include "classp.h"
// #include "debug.h"
//
// #ifdef DEBUG_USE_WPP
// #include "history.tmh"
// #endif
//
// //#ifdef ALLOC_PRAGMA
// //    #pragma alloc_text(PAGE, InitializeTransferPackets)
// //#endif
//
// VOID HistoryInitializeRetryLogs( PSRB_HISTORY History, ULONG HistoryCount) {
//     ULONG tmpSize = HistoryCount * sizeof(SRB_HISTORY_ITEM);
//     tmpSize += sizeof(SRB_HISTORY) - sizeof(SRB_HISTORY_ITEM);
//     RtlZeroMemory(History, tmpSize);
//     History->TotalHistoryCount = HistoryCount;
//     return;
// }
//
//
// VOID HistoryLogSendPacket(TRANSFER_PACKET * Pkt) {
//
//     PSRB_HISTORY history;
//     PSRB_HISTORY_ITEM item;
//
//     NT_ASSERT( Pkt->RetryHistory != NULL );
//     history = Pkt->RetryHistory;
//     // sending a packet implies a new history unit is to be used.
//     NT_ASSERT( history->UsedHistoryCount <= history->TotalHistoryCount );
//
//     // if already all used up, request class driver to remove at least one history unit
//     if (history->UsedHistoryCount == history->TotalHistoryCount )
//     {
//         PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Pkt->Fdo->DeviceExtension;
//         PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//         NT_ASSERT( fdoData->InterpretSenseInfo != NULL );
//         NT_ASSERT( fdoData->InterpretSenseInfo->Compress != NULL );
//         fdoData->InterpretSenseInfo->Compress( fdoExtension->DeviceObject, history );
//         NT_ASSERT( history->UsedHistoryCount < history->TotalHistoryCount );
//     }
//
//     // thus, since we are about to increment the count, it must now be less...
//     NT_ASSERT( history->UsedHistoryCount < history->TotalHistoryCount );
//
//     // increment the number of history units in use
//     history->UsedHistoryCount++;
//
//     // determine index to use
//     item = &( history->History[ history->UsedHistoryCount-1 ] );
//
//     // zero out the history item
//     RtlZeroMemory(item, sizeof(SRB_HISTORY_ITEM));
//
//     // Query the tick count and store in the history
//     KeQueryTickCount(&item->TickCountSent);
//     return;
// }
//
// VOID HistoryLogReturnedPacket(TRANSFER_PACKET *Pkt) {
//
//     PSRB_HISTORY history;
//     PSRB_HISTORY_ITEM item;
//     UCHAR senseSize;
//     PVOID senseInfoBuffer;
//     UCHAR senseInfoBufferLength;
//     SENSE_DATA convertedSenseBuffer = {0};
//     BOOLEAN validSense = TRUE;
//
//     NT_ASSERT( Pkt->RetryHistory != NULL );
//     history = Pkt->RetryHistory;
//     NT_ASSERT( history->UsedHistoryCount <= history->TotalHistoryCount );
//     item = &( history->History[ history->UsedHistoryCount-1 ] );
//
//     // Query the tick count and store in the history
//     KeQueryTickCount(&item->TickCountCompleted);
//
//     // Copy the SRB Status...
//     item->SrbStatus = Pkt->Srb->SrbStatus;
//
//     //
//     // Process sense data
//     //
//
//     senseInfoBuffer = ClasspTransferPacketGetSenseInfoBuffer(Pkt);
//     senseInfoBufferLength = ClasspTransferPacketGetSenseInfoBufferLength(Pkt);
//
//     if (IsDescriptorSenseDataFormat(senseInfoBuffer)) {
//
//         validSense = ScsiConvertToFixedSenseFormat(senseInfoBuffer,
//                                                    senseInfoBufferLength,
//                                                    (PVOID)&convertedSenseBuffer,
//                                                    sizeof(convertedSenseBuffer));
//
//         if (validSense) {
//             senseInfoBuffer = (PVOID)&convertedSenseBuffer;
//             senseInfoBufferLength = sizeof(convertedSenseBuffer);
//         }
//     }
//
//     RtlZeroMemory(&(item->NormalizedSenseData), sizeof(item->NormalizedSenseData));
//
//     if (validSense) {
//
//         // Determine the amount of valid sense data
//
//         if (!ScsiGetTotalSenseByteCountIndicated(senseInfoBuffer,
//                                                  senseInfoBufferLength,
//                                                  &senseSize)) {
//             senseSize = senseInfoBufferLength;
//         }
//
//         // Normalize the sense data copy in the history
//         senseSize = min(senseSize, sizeof(item->NormalizedSenseData));
//         RtlCopyMemory(&(item->NormalizedSenseData),
//                       senseInfoBuffer,
//                       senseSize
//                       );
//     }
//
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
