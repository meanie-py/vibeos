#[allow(unused_imports)] use crate::{TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_CACHE; // cache.c: DBG_DEFAULT_CHANNEL(CACHE)
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;
use crate::frldr::{
    _CACHE_BLOCK, PCACHE_BLOCK, PCACHE_DRIVE, _GEOMETRY,
    TAG_CACHE_BLOCK, TAG_CACHE_DATA, MM_PAGE_SIZE, TEMP_HEAP_SIZE, min,
    CacheManagerDrive, CacheManagerInitialized, CacheManagerDataInvalid,
    CacheBlockCount, CacheSizeLimit, CacheSizeCurrent,
    FrLdrTempFree, MachDiskGetDriveGeometry, MachDiskGetCacheableBlockCount,
    TotalPagesInLookupTable,
};
// CacheInternal* live in crate::blocklist (same staticlib).
use crate::blocklist::{CacheInternalGetBlockPointer, CacheInternalFreeBlock};

// TRACE: DBG_DEFAULT_CHANNEL macro in C (no real symbol) — kept variadic for
// call typecheck, satisfied at link by the cdecl no-op in crate::frldr.
extern "C" {
}

#[no_mangle]
pub unsafe extern "C" fn CacheInitializeDrive(DriveNumber: u8) -> u8 {
    let mut NextCacheBlock: *mut _CACHE_BLOCK;
    let mut DriveGeometry: _GEOMETRY = core::mem::zeroed();

    if (CacheManagerInitialized != 0) &&
        (DriveNumber == CacheManagerDrive.DriveNumber) &&
        (DriveNumber >= 0x80) &&
        (CacheManagerDataInvalid == 0)
    {
        return 1u8;
    }

    CacheManagerDataInvalid = 0u8;

    if CacheManagerInitialized != 0
    {
        CacheManagerInitialized = 0u8;

        TRACE!("CacheBlockCount: %d\n", CacheBlockCount);
        TRACE!("CacheSizeLimit: %d\n", CacheSizeLimit);
        TRACE!("CacheSizeCurrent: %d\n", CacheSizeCurrent);

        while !IsListEmpty(core::ptr::addr_of_mut!(CacheManagerDrive.CacheBlockHead))
        {
            NextCacheBlock = ((RemoveHeadList(core::ptr::addr_of_mut!(CacheManagerDrive.CacheBlockHead)) as *mut u8)
                .offset(-(core::mem::offset_of!(_CACHE_BLOCK, ListEntry) as isize))) as *mut _CACHE_BLOCK;

            FrLdrTempFree((*NextCacheBlock).BlockData as *mut core::ffi::c_void, TAG_CACHE_DATA);
            FrLdrTempFree(NextCacheBlock as *mut core::ffi::c_void, TAG_CACHE_BLOCK);
        }
    }

    core::ptr::write_bytes(core::ptr::addr_of_mut!(CacheManagerDrive), 0, 1);
    InitializeListHead(core::ptr::addr_of_mut!(CacheManagerDrive.CacheBlockHead));
    CacheManagerDrive.DriveNumber = DriveNumber;
    if MachDiskGetDriveGeometry(DriveNumber, core::ptr::addr_of_mut!(DriveGeometry)) == 0
    {
        return 0u8;
    }
    CacheManagerDrive.BytesPerSector = DriveGeometry.BytesPerSector;

    CacheManagerDrive.BlockSize = MachDiskGetCacheableBlockCount(DriveNumber);

    CacheBlockCount = 0;
    CacheSizeCurrent = 0;
    CacheSizeLimit = TotalPagesInLookupTable / 8 * MM_PAGE_SIZE;
    CacheSizeLimit = min(CacheSizeLimit, TEMP_HEAP_SIZE - (128 * 1024));

    CacheManagerInitialized = 1u8;

    TRACE!("Initializing BIOS drive 0x%x.\n", DriveNumber as u32);
    TRACE!("BytesPerSector: %d.\n", CacheManagerDrive.BytesPerSector);
    TRACE!("BlockSize: %d.\n", CacheManagerDrive.BlockSize);
    TRACE!("CacheSizeLimit: %d.\n", CacheSizeLimit);

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn CacheInvalidateCacheData() {
    CacheManagerDataInvalid = 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn CacheReadDiskSectors(
    DiskNumber: UCHAR,
    StartSector: ULONGLONG,
    SectorCount: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {
    let mut CacheBlock: PCACHE_BLOCK;
    let StartBlock: ULONG;
    let SectorOffsetInStartBlock: ULONG;
    let CopyLengthInStartBlock: ULONG;
    let EndBlock: ULONG;
    let SectorOffsetInEndBlock: ULONG;
    let mut BlockCount: ULONG;
    let mut Idx: ULONG;
    let mut Buffer = Buffer;

    TRACE!("CacheReadDiskSectors() DiskNumber: 0x%x StartSector: %I64u SectorCount: %u Buffer: 0x%x\n", DiskNumber as u32, StartSector, SectorCount, Buffer);

    if CacheManagerInitialized == 0u8 {
        return 0u8;
    }

    StartBlock = (StartSector / CacheManagerDrive.BlockSize as ULONGLONG) as ULONG;
    SectorOffsetInStartBlock = (StartSector % CacheManagerDrive.BlockSize as ULONGLONG) as ULONG;
    CopyLengthInStartBlock = if SectorCount > (CacheManagerDrive.BlockSize - SectorOffsetInStartBlock) {
        CacheManagerDrive.BlockSize - SectorOffsetInStartBlock
    } else {
        SectorCount
    };
    EndBlock = ((StartSector + (SectorCount - 1) as ULONGLONG) / CacheManagerDrive.BlockSize as ULONGLONG) as ULONG;
    SectorOffsetInEndBlock = (1 + (StartSector + (SectorCount - 1) as ULONGLONG) % CacheManagerDrive.BlockSize as ULONGLONG) as ULONG;
    BlockCount = (EndBlock - StartBlock) + 1;
    TRACE!("StartBlock: %d SectorOffsetInStartBlock: %d CopyLengthInStartBlock: %d EndBlock: %d SectorOffsetInEndBlock: %d BlockCount: %d\n", StartBlock, SectorOffsetInStartBlock, CopyLengthInStartBlock, EndBlock, SectorOffsetInEndBlock, BlockCount);

    if BlockCount > 0 {
        CacheBlock = CacheInternalGetBlockPointer(core::ptr::addr_of_mut!(CacheManagerDrive), StartBlock);
        if CacheBlock.is_null() {
            return 0u8;
        }

        RtlCopyMemory(
            Buffer,
            ((*CacheBlock).BlockData as ULONG_PTR + (SectorOffsetInStartBlock * CacheManagerDrive.BytesPerSector) as ULONG_PTR) as PVOID,
            (CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector) as SIZE_T,
        );
        TRACE!("1 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, (*CacheBlock).BlockData as ULONG_PTR + (SectorOffsetInStartBlock * CacheManagerDrive.BytesPerSector) as ULONG_PTR, CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector);

        Buffer = (Buffer as ULONG_PTR + (CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector) as ULONG_PTR) as PVOID;

        BlockCount -= 1;
    }

    Idx = StartBlock + 1;
    while BlockCount > 1 {
        CacheBlock = CacheInternalGetBlockPointer(core::ptr::addr_of_mut!(CacheManagerDrive), Idx);
        if CacheBlock.is_null() {
            return 0u8;
        }

        RtlCopyMemory(
            Buffer,
            (*CacheBlock).BlockData,
            (CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector) as SIZE_T,
        );
        TRACE!("2 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, (*CacheBlock).BlockData, CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector);

        Buffer = (Buffer as ULONG_PTR + (CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector) as ULONG_PTR) as PVOID;

        BlockCount -= 1;
        Idx += 1;
    }

    if BlockCount > 0 {
        CacheBlock = CacheInternalGetBlockPointer(core::ptr::addr_of_mut!(CacheManagerDrive), EndBlock);
        if CacheBlock.is_null() {
            return 0u8;
        }

        RtlCopyMemory(
            Buffer,
            (*CacheBlock).BlockData,
            (SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector) as SIZE_T,
        );
        TRACE!("3 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, (*CacheBlock).BlockData, SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector);

        Buffer = (Buffer as ULONG_PTR + (SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector) as ULONG_PTR) as PVOID;

        BlockCount -= 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CacheForceDiskSectorsIntoCache(
    DiskNumber: UCHAR,
    StartSector: ULONGLONG,
    SectorCount: ULONG,
) -> BOOLEAN {
    let mut CacheBlock: PCACHE_BLOCK;
    let StartBlock: ULONG;
    let EndBlock: ULONG;
    let BlockCount: ULONG;
    let mut Idx: ULONG;

    TRACE!("CacheForceDiskSectorsIntoCache() DiskNumber: 0x%x StartSector: %d SectorCount: %d\n", DiskNumber as u32, StartSector, SectorCount);

    if CacheManagerInitialized == 0u8 {
        return 0u8;
    }

    StartBlock = (StartSector / (*core::ptr::addr_of!(CacheManagerDrive)).BlockSize as ULONGLONG) as ULONG;
    EndBlock = ((StartSector + SectorCount as ULONGLONG) / (*core::ptr::addr_of!(CacheManagerDrive)).BlockSize as ULONGLONG) as ULONG;
    BlockCount = (EndBlock - StartBlock) + 1;

    Idx = StartBlock;
    while Idx < (StartBlock + BlockCount) {
        CacheBlock = CacheInternalGetBlockPointer(core::ptr::addr_of_mut!(CacheManagerDrive), Idx);
        if CacheBlock == core::ptr::null_mut() {
            return 0u8;
        }

        (*CacheBlock).LockedInCache = 1u8;
        Idx += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CacheReleaseMemory(MinimumAmountToRelease: ULONG) -> BOOLEAN {
    let mut AmountReleased: ULONG;

    TRACE!("CacheReleaseMemory() MinimumAmountToRelease = %d\n", MinimumAmountToRelease);

    if CacheManagerInitialized == 0u8 {
        return 0u8;
    }

    AmountReleased = 0;
    while AmountReleased < MinimumAmountToRelease {
        if CacheInternalFreeBlock(core::ptr::addr_of_mut!(CacheManagerDrive)) == 0u8 {
            break;
        }

        AmountReleased += CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector;
    }

    if AmountReleased >= MinimumAmountToRelease { 1u8 } else { 0u8 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(CACHE);
//
// ///////////////////////////////////////////////////////////////////////////////////////
// //
// // Internal data
// //
// ///////////////////////////////////////////////////////////////////////////////////////
// CACHE_DRIVE        CacheManagerDrive;
// BOOLEAN            CacheManagerInitialized = FALSE;
// BOOLEAN            CacheManagerDataInvalid = FALSE;
// ULONG            CacheBlockCount = 0;
// SIZE_T            CacheSizeLimit = 0;
// SIZE_T            CacheSizeCurrent = 0;
//
// BOOLEAN CacheInitializeDrive(UCHAR DriveNumber)
// {
//     PCACHE_BLOCK    NextCacheBlock;
//     GEOMETRY    DriveGeometry;
//
//     // If we already have a cache for this drive then
//     // by all means lets keep it, unless it is a removable
//     // drive, in which case we'll invalidate the cache
//     if ((CacheManagerInitialized) &&
//         (DriveNumber == CacheManagerDrive.DriveNumber) &&
//         (DriveNumber >= 0x80) &&
//         (!CacheManagerDataInvalid))
//     {
//         return TRUE;
//     }
//
//     CacheManagerDataInvalid = FALSE;
//
//     //
//     // If we have already been initialized then free
//     // the old data
//     //
//     if (CacheManagerInitialized)
//     {
//         CacheManagerInitialized = FALSE;
//
//         TRACE("CacheBlockCount: %d\n", CacheBlockCount);
//         TRACE("CacheSizeLimit: %d\n", CacheSizeLimit);
//         TRACE("CacheSizeCurrent: %d\n", CacheSizeCurrent);
//         //
//         // Loop through and free the cache blocks
//         //
//         while (!IsListEmpty(&CacheManagerDrive.CacheBlockHead))
//         {
//             NextCacheBlock = CONTAINING_RECORD(RemoveHeadList(&CacheManagerDrive.CacheBlockHead),
//                                                CACHE_BLOCK,
//                                                ListEntry);
//
//             FrLdrTempFree(NextCacheBlock->BlockData, TAG_CACHE_DATA);
//             FrLdrTempFree(NextCacheBlock, TAG_CACHE_BLOCK);
//         }
//     }
//
//     // Initialize the structure
//     RtlZeroMemory(&CacheManagerDrive, sizeof(CACHE_DRIVE));
//     InitializeListHead(&CacheManagerDrive.CacheBlockHead);
//     CacheManagerDrive.DriveNumber = DriveNumber;
//     if (!MachDiskGetDriveGeometry(DriveNumber, &DriveGeometry))
//     {
//         return FALSE;
//     }
//     CacheManagerDrive.BytesPerSector = DriveGeometry.BytesPerSector;
//
//     // Get the number of sectors in each cache block
//     CacheManagerDrive.BlockSize = MachDiskGetCacheableBlockCount(DriveNumber);
//
//     CacheBlockCount = 0;
//     CacheSizeCurrent = 0;
//     CacheSizeLimit = TotalPagesInLookupTable / 8 * MM_PAGE_SIZE;
//     CacheSizeLimit = min(CacheSizeLimit, TEMP_HEAP_SIZE - (128 * 1024));
//
//     CacheManagerInitialized = TRUE;
//
//     TRACE("Initializing BIOS drive 0x%x.\n", DriveNumber);
//     TRACE("BytesPerSector: %d.\n", CacheManagerDrive.BytesPerSector);
//     TRACE("BlockSize: %d.\n", CacheManagerDrive.BlockSize);
//     TRACE("CacheSizeLimit: %d.\n", CacheSizeLimit);
//
//     return TRUE;
// }
//
// VOID CacheInvalidateCacheData(VOID)
// {
//     CacheManagerDataInvalid = TRUE;
// }
//
// BOOLEAN CacheReadDiskSectors(UCHAR DiskNumber, ULONGLONG StartSector, ULONG SectorCount, PVOID Buffer)
// {
//     PCACHE_BLOCK    CacheBlock;
//     ULONG                StartBlock;
//     ULONG                SectorOffsetInStartBlock;
//     ULONG                CopyLengthInStartBlock;
//     ULONG                EndBlock;
//     ULONG                SectorOffsetInEndBlock;
//     ULONG                BlockCount;
//     ULONG                Idx;
//
//     TRACE("CacheReadDiskSectors() DiskNumber: 0x%x StartSector: %I64u SectorCount: %u Buffer: 0x%x\n", DiskNumber, StartSector, SectorCount, Buffer);
//
//     // If we aren't initialized yet then they can't do this
//     if (CacheManagerInitialized == FALSE)
//     {
//         return FALSE;
//     }
//
//     //
//     // Calculate which blocks we must cache
//     //
//     StartBlock = (ULONG)(StartSector / CacheManagerDrive.BlockSize);
//     SectorOffsetInStartBlock = (ULONG)(StartSector % CacheManagerDrive.BlockSize);
//     CopyLengthInStartBlock = (ULONG)((SectorCount > (CacheManagerDrive.BlockSize - SectorOffsetInStartBlock)) ? (CacheManagerDrive.BlockSize - SectorOffsetInStartBlock) : SectorCount);
//     EndBlock = (ULONG)((StartSector + (SectorCount - 1)) / CacheManagerDrive.BlockSize);
//     SectorOffsetInEndBlock = (ULONG)(1 + (StartSector + (SectorCount - 1)) % CacheManagerDrive.BlockSize);
//     BlockCount = (EndBlock - StartBlock) + 1;
//     TRACE("StartBlock: %d SectorOffsetInStartBlock: %d CopyLengthInStartBlock: %d EndBlock: %d SectorOffsetInEndBlock: %d BlockCount: %d\n", StartBlock, SectorOffsetInStartBlock, CopyLengthInStartBlock, EndBlock, SectorOffsetInEndBlock, BlockCount);
//
//     //
//     // Read the first block into the buffer
//     //
//     if (BlockCount > 0)
//     {
//         //
//         // Get cache block pointer (this forces the disk sectors into the cache memory)
//         //
//         CacheBlock = CacheInternalGetBlockPointer(&CacheManagerDrive, StartBlock);
//         if (CacheBlock == NULL)
//         {
//             return FALSE;
//         }
//
//         //
//         // Copy the portion requested into the buffer
//         //
//         RtlCopyMemory(Buffer,
//             (PVOID)((ULONG_PTR)CacheBlock->BlockData + (SectorOffsetInStartBlock * CacheManagerDrive.BytesPerSector)),
//             (CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector));
//         TRACE("1 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, ((ULONG_PTR)CacheBlock->BlockData + (SectorOffsetInStartBlock * CacheManagerDrive.BytesPerSector)), (CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector));
//
//         //
//         // Update the buffer address
//         //
//         Buffer = (PVOID)((ULONG_PTR)Buffer + (CopyLengthInStartBlock * CacheManagerDrive.BytesPerSector));
//
//         //
//         // Update the block count
//         //
//         BlockCount--;
//     }
//
//     //
//     // Loop through the middle blocks and read them into the buffer
//     //
//     for (Idx=StartBlock+1; BlockCount>1; Idx++)
//     {
//         //
//         // Get cache block pointer (this forces the disk sectors into the cache memory)
//         //
//         CacheBlock = CacheInternalGetBlockPointer(&CacheManagerDrive, Idx);
//         if (CacheBlock == NULL)
//         {
//             return FALSE;
//         }
//
//         //
//         // Copy the portion requested into the buffer
//         //
//         RtlCopyMemory(Buffer,
//             CacheBlock->BlockData,
//             CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector);
//         TRACE("2 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, CacheBlock->BlockData, CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector);
//
//         //
//         // Update the buffer address
//         //
//         Buffer = (PVOID)((ULONG_PTR)Buffer + (CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector));
//
//         //
//         // Update the block count
//         //
//         BlockCount--;
//     }
//
//     //
//     // Read the last block into the buffer
//     //
//     if (BlockCount > 0)
//     {
//         //
//         // Get cache block pointer (this forces the disk sectors into the cache memory)
//         //
//         CacheBlock = CacheInternalGetBlockPointer(&CacheManagerDrive, EndBlock);
//         if (CacheBlock == NULL)
//         {
//             return FALSE;
//         }
//
//         //
//         // Copy the portion requested into the buffer
//         //
//         RtlCopyMemory(Buffer,
//             CacheBlock->BlockData,
//             SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector);
//         TRACE("3 - RtlCopyMemory(0x%x, 0x%x, %d)\n", Buffer, CacheBlock->BlockData, SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector);
//
//         //
//         // Update the buffer address
//         //
//         Buffer = (PVOID)((ULONG_PTR)Buffer + (SectorOffsetInEndBlock * CacheManagerDrive.BytesPerSector));
//
//         //
//         // Update the block count
//         //
//         BlockCount--;
//     }
//
//     return TRUE;
// }
//
// #if 0
// BOOLEAN CacheForceDiskSectorsIntoCache(UCHAR DiskNumber, ULONGLONG StartSector, ULONG SectorCount)
// {
//     PCACHE_BLOCK    CacheBlock;
//     ULONG                StartBlock;
//     ULONG                EndBlock;
//     ULONG                BlockCount;
//     ULONG                Idx;
//
//     TRACE("CacheForceDiskSectorsIntoCache() DiskNumber: 0x%x StartSector: %d SectorCount: %d\n", DiskNumber, StartSector, SectorCount);
//
//     // If we aren't initialized yet then they can't do this
//     if (CacheManagerInitialized == FALSE)
//     {
//         return FALSE;
//     }
//
//     //
//     // Calculate which blocks we must cache
//     //
//     StartBlock = StartSector / CacheManagerDrive.BlockSize;
//     EndBlock = (StartSector + SectorCount) / CacheManagerDrive.BlockSize;
//     BlockCount = (EndBlock - StartBlock) + 1;
//
//     //
//     // Loop through and cache them
//     //
//     for (Idx=StartBlock; Idx<(StartBlock+BlockCount); Idx++)
//     {
//         //
//         // Get cache block pointer (this forces the disk sectors into the cache memory)
//         //
//         CacheBlock = CacheInternalGetBlockPointer(&CacheManagerDrive, Idx);
//         if (CacheBlock == NULL)
//         {
//             return FALSE;
//         }
//
//         //
//         // Lock the sectors into the cache
//         //
//         CacheBlock->LockedInCache = TRUE;
//     }
//
//     return TRUE;
// }
// #endif
//
// BOOLEAN CacheReleaseMemory(ULONG MinimumAmountToRelease)
// {
//     ULONG                AmountReleased;
//
//     TRACE("CacheReleaseMemory() MinimumAmountToRelease = %d\n", MinimumAmountToRelease);
//
//     // If we aren't initialized yet then they can't do this
//     if (CacheManagerInitialized == FALSE)
//     {
//         return FALSE;
//     }
//
//     // Loop through and try to free the requested amount of memory
//     for (AmountReleased=0; AmountReleased<MinimumAmountToRelease; )
//     {
//         // Try to free a block
//         // If this fails then break out of the loop
//         if (!CacheInternalFreeBlock(&CacheManagerDrive))
//         {
//             break;
//         }
//
//         // It succeeded so increment the amount of memory we have freed
//         AmountReleased += CacheManagerDrive.BlockSize * CacheManagerDrive.BytesPerSector;
//     }
//
//     // Return status
//     return (AmountReleased >= MinimumAmountToRelease);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
