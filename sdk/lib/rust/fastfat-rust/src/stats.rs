//! The per-processor performance counters behind FSCTL_FILESYSTEM_GET_STATISTICS.
//!
//! create.c, read.c and deviosup.c define these as macros. The cohab glue used to
//! stand in for them with empty C functions, so creates, cached reads and disk I/O
//! were never counted. (write.c's CollectWriteStats was already expanded inline in
//! write.rs.) `FatUpdateIOCountersPCW` stays in the glue as a no-op: below
//! NTDDI_WIN8 deviosup.c defines it as nothing.
#![allow(non_snake_case)]

use crate::types::{
    FatData, _FILE_SYSTEM_STATISTICS, _TYPE_OF_OPEN, _TYPE_OF_OPEN_DirectoryFile,
    _TYPE_OF_OPEN_UserFileOpen, _TYPE_OF_OPEN_VirtualVolumeFile, VCB,
};
use ntkrnl_sys::ddk::KeGetCurrentProcessorNumber;

const IRP_MJ_WRITE: u32 = 0x04;

/// `&(VCB)->Statistics[KeGetCurrentProcessorNumber() % FatData.NumberProcessors]`
#[inline(always)]
unsafe fn current_stats(Vcb: *mut VCB) -> *mut _FILE_SYSTEM_STATISTICS {
    let index = KeGetCurrentProcessorNumber() as usize % FatData.NumberProcessors as usize;
    (*Vcb).Statistics.add(index)
}

/// create.c `CollectCreateHitStatistics`
pub unsafe fn CollectCreateHitStatistics(Vcb: *mut VCB) {
    let stats = current_stats(Vcb);
    (*stats).Fat.CreateHits = (*stats).Fat.CreateHits.wrapping_add(1);
}

/// create.c `CollectCreateStatistics`
pub unsafe fn CollectCreateStatistics(Vcb: *mut VCB, Status: i32) {
    let stats = current_stats(Vcb);
    if Status == 0 {
        (*stats).Fat.SuccessfulCreates = (*stats).Fat.SuccessfulCreates.wrapping_add(1);
    } else {
        (*stats).Fat.FailedCreates = (*stats).Fat.FailedCreates.wrapping_add(1);
    }
}

/// read.c `CollectReadStats`
pub unsafe fn CollectReadStats(Vcb: *mut VCB, OpenType: _TYPE_OF_OPEN, ByteCount: u32) {
    let stats = &mut (*current_stats(Vcb)).Common;
    if OpenType == _TYPE_OF_OPEN_UserFileOpen {
        stats.UserFileReads = stats.UserFileReads.wrapping_add(1);
        stats.UserFileReadBytes = stats.UserFileReadBytes.wrapping_add(ByteCount);
    } else if OpenType == _TYPE_OF_OPEN_VirtualVolumeFile || OpenType == _TYPE_OF_OPEN_DirectoryFile {
        stats.MetaDataReads = stats.MetaDataReads.wrapping_add(1);
        stats.MetaDataReadBytes = stats.MetaDataReadBytes.wrapping_add(ByteCount);
    }
}

/// deviosup.c `CollectDiskIoStats`
pub unsafe fn CollectDiskIoStats(Vcb: *mut VCB, Function: u32, IsUserIo: u8, Count: u32) {
    let stats = &mut (*current_stats(Vcb)).Common;
    let is_write = Function == IRP_MJ_WRITE;
    match (IsUserIo != 0, is_write) {
        (true, true) => stats.UserDiskWrites = stats.UserDiskWrites.wrapping_add(Count),
        (true, false) => stats.UserDiskReads = stats.UserDiskReads.wrapping_add(Count),
        (false, true) => stats.MetaDataDiskWrites = stats.MetaDataDiskWrites.wrapping_add(Count),
        (false, false) => stats.MetaDataDiskReads = stats.MetaDataDiskReads.wrapping_add(Count),
    }
}
