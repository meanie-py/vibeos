//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/hivewrt.c. Risk: HIGH.
//! Behavior: 7 real bugs found, 2 critical (wrong constants corrupt on-disk data / abort
//! writes), 1 critical (HvpWriteLog drops half its work — sequence2 bump + second flush
//! missing), 3 ABI bugs (extern "C" vs "system" on i386, FileFlush called with wrong
//! signature, FileWrite/Allocate transmuted with u32 for SIZE_T on 64-bit).
//!
//! **BUG 1 — CRITICAL: HV_HBLOCK_SIGNATURE wrong value (hivewrt.rs:22)**
//!   Rust: `0xbee0bee0` — C: `0x66676572` ("regf", hivedata.h:63).
//!   (0xbee0bee0 is HV_HHIVE_SIGNATURE, not the on-disk base block signature.)
//!   Used only in ASSERTs here so the bug is dormant at runtime but still wrong.
//!
//! **BUG 2 — CRITICAL: HV_LOG_HEADER_SIZE wrong value (hivewrt.rs:26)**
//!   Rust: `4096` — C: `FIELD_OFFSET(HBASE_BLOCK, Reserved2)` = 512 = 0x200 (hivedata.h:46).
//!   The Rust uses 4096 (= HBLOCK_SIZE). This causes the log header copy to be 8× too
//!   large, corrupts the dirty-block bitmap placement, and the first FileWrite sends
//!   wrong data to disk. Affects every code path through HvpWriteLog.
//!
//! **BUG 3 — CRITICAL: HV_LOG_DIRTY_SIGNATURE wrong value (hivewrt.rs:28)**
//!   Rust: `0x4c445646` — C: `0x54524944` ("DIRT", hivedata.h:57).
//!   The wrong magic "VLDF" is written to the log dirty-block bitmap header; a log
//!   replayed by the bootloader will fail to recognise the dirty region and cannot
//!   recover a damaged hive.
//!
//! **BUG 4 — CRITICAL: HV_LOG_DIRTY_BLOCK wrong value (hivewrt.rs:29)**
//!   Rust: `1` — C: `0xFF` (hivedata.h:56).
//!   Each dirty-block slot is tagged with 1 instead of 0xFF; again mismatches the
//!   bootloader's expected pattern, breaking log-based recovery.
//!
//! **BUG 5 — CRITICAL: HvpWriteLog drops secondary-sequence write + second flush (missing body)**
//!   After the dirty-block write loop, the C does: Sequence2++, recompute checksum,
//!   write BaseBlock again at FileOffset=0, flush again, then return TRUE/FALSE on
//!   each step. The Rust (line 151-152) calls FileFlush once and immediately returns 1.
//!   The entire secondary-sequence commitment step is missing. The log is left
//!   permanently with Sequence1 != Sequence2, so the kernel will treat every log as
//!   incomplete and never apply it during crash recovery.
//!
//! **BUG 6 — ABI: extern "C" for CMAPI (stdcall on i386) (lines 11, 32, 155, 215, 227, 246, 261)**
//!   `HvpValidateBaseHeader`, `HvpHiveHeaderChecksum`, `HvpWriteHive` are declared
//!   `extern "C"` (cdecl) but CMAPI = NTAPI = stdcall on i386. Callee-pops mismatch →
//!   stack corruption on every call on i386. All `#[no_mangle] pub unsafe extern "C" fn`
//!   public symbols have the same defect — they must be `extern "system"` to match the C
//!   callers (see lib.rs:13 which already documents this requirement).
//!
//! **BUG 7 — ABI: FileFlush transmuted to wrong signature, FileWrite/Allocate SIZE_T truncated**
//!   C `PFILE_FLUSH_ROUTINE` is `fn(HHIVE*, ULONG FileType, PLARGE_INTEGER, ULONG) -> BOOLEAN`
//!   (hivedata.h:274-279). Rust (line 151) transmutes to `fn(*mut HHIVE) -> i32` — three
//!   args are dropped, so random stack garbage is passed as FileType/offset/length.
//!   `PFILE_WRITE_ROUTINE` has `SIZE_T BufferLength` (pointer-sized); Rust transmutes it as
//!   `u32` (line 103/131) — silently truncates on amd64/arm64 if buffer > 4 GB (unlikely
//!   in practice for registry blocks, so LOW practical impact, but still ABI-wrong).
//!   `PALLOCATE_ROUTINE` has `SIZE_T Size`; transmuted as `u32` (line 60) — same issue.
//!
//! Unverified: `ROUND_UP` macro semantics (Rust `round_up` inline differs from glue.rs
//! usage; BitmapSize calc uses `SizeOfBitMap` directly in bits, matching the C — OK).
//! The bootloader `bldr` feature mirrors the C `_BLDR_` build by gating the
//! kernel-only hard-error and timestamp calls.

use ntkrnl_sys::mm::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::hiveinit::DbgPrint;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{HBASE_BLOCK, HHIVE, PHBASE_BLOCK, PHHIVE, RtlFindSetBits, HSTORAGE_TYPE_Stable as Stable};
#[cfg(not(feature = "bldr"))]
use ntkrnl_sys::ddk::IoSetThreadHardErrorMode;
use crate::glue::{HCELL_NIL, HBLOCK_SIZE, TAG_CM, HV_HHIVE_SIGNATURE, HFILE_TYPE_ALTERNATE, HFILE_TYPE_LOG, HFILE_TYPE_PRIMARY, HIVE_VOLATILE};
use core::ffi::c_void;
use core::ptr::addr_of_mut;

extern "system" {
    fn HvpHiveHeaderChecksum(BaseBlock: PHBASE_BLOCK) -> u32;
}
unsafe extern "C" {
}
unsafe extern "system" {
    // KeQuerySystemTime is NTAPI (stdcall on i386 → _KeQuerySystemTime@4).
    // Provided by ntoskrnl in the kernel; by the loader stub in uefildr/rosload.
    fn KeQuerySystemTime(SystemTime: *mut i64);
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

const HV_HBLOCK_SIGNATURE: u32 = 0x66676572;
const HBASE_FORMAT_MEMORY: u32 = 1;
const HSYS_MAJOR: u32 = 1;

const HV_LOG_HEADER_SIZE: u32 = 512;
const HSECTOR_SIZE: u32 = 512;
const HV_LOG_DIRTY_SIGNATURE: u32 = 0x54524944;
const HV_LOG_DIRTY_BLOCK: u32 = 0xFF;
const HV_CLEAN_BLOCK: u32 = 0;

/// Validates the base block header of a primary hive for consistency.
/// C: `static VOID HvpValidateBaseHeader(...)` — the three checks are ASSERTs,
/// which are no-ops in the NDEBUG cmlib build.
unsafe fn HvpValidateBaseHeader(RegistryHive: *mut HHIVE) {
    let BaseBlock = (*RegistryHive).BaseBlock;
    ntkrnl_sys::ASSERT!((*BaseBlock).Signature == HV_HBLOCK_SIGNATURE);
    ntkrnl_sys::ASSERT!((*BaseBlock).Format == HBASE_FORMAT_MEMORY);
    ntkrnl_sys::ASSERT!((*BaseBlock).Major == HSYS_MAJOR);
}

/// Writes data (dirty or non) to a primary hive during syncing operation.
/// C: `static BOOLEAN CMAPI HvpWriteHive(...)`.
unsafe fn HvpWriteHive(RegistryHive: *mut HHIVE, OnlyDirty: u8, FileType: u32) -> u8 {
    let mut Success: u8;
    let mut FileOffset: u32;
    let mut BlockIndex: u32;
    let mut LastIndex: u32;
    let mut Block: *mut c_void;

    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);
    ntkrnl_sys::ASSERT!((*(*RegistryHive).BaseBlock).Length
            == (*RegistryHive).Storage[Stable as usize].Length * HBLOCK_SIZE);
    ntkrnl_sys::ASSERT!((*(*RegistryHive).BaseBlock).RootCell != HCELL_NIL);

    /* Validate the base header before we go further */
    HvpValidateBaseHeader(RegistryHive);

    /*
     * The sequences can diverge during a forced system shutdown. In such
     * cases the hive needs a repair.
     */
    if (*(*RegistryHive).BaseBlock).Sequence1 != (*(*RegistryHive).BaseBlock).Sequence2 {
        return 0u8;
    }

    /*
     * Update the primary sequence number and write the base block to hive.
     */
    (*(*RegistryHive).BaseBlock).Type = HFILE_TYPE_PRIMARY;
    (*(*RegistryHive).BaseBlock).Sequence1 += 1;
    (*(*RegistryHive).BaseBlock).CheckSum = HvpHiveHeaderChecksum((*RegistryHive).BaseBlock);

    /* Write hive block */
    FileOffset = 0;
    Success = (*RegistryHive).FileWrite.unwrap()(
        RegistryHive,
        FileType,
        &mut FileOffset,
        (*RegistryHive).BaseBlock as *mut c_void,
        core::mem::size_of::<HBASE_BLOCK>() as _,
    );
    if Success == 0 {
        return 0u8;
    }

    /* Write the whole primary hive, block by block */
    BlockIndex = 0;
    while BlockIndex < (*RegistryHive).Storage[Stable as usize].Length {
        /*
         * If we have to synchronize the registry hive we want to look for
         * dirty blocks. Otherwise just write all the blocks.
         */
        if OnlyDirty != 0 {
            LastIndex = BlockIndex;
            BlockIndex = RtlFindSetBits(
                core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
                1,
                BlockIndex,
            );
            if BlockIndex == !HV_CLEAN_BLOCK || BlockIndex < LastIndex {
                break;
            }
        }

        /* Get the block and offset position */
        Block = (*(*RegistryHive).Storage[Stable as usize]
            .BlockList
            .add(BlockIndex as usize))
        .BlockAddress as *mut c_void;
        FileOffset = (BlockIndex + 1) * HBLOCK_SIZE;

        /* Now write this block to primary hive file */
        Success = (*RegistryHive).FileWrite.unwrap()(
            RegistryHive,
            FileType,
            &mut FileOffset,
            Block,
            HBLOCK_SIZE as _,
        );
        if Success == 0 {
            return 0u8;
        }

        /* Go to the next block */
        BlockIndex += 1;
    }

    /*
     * We wrote all the hive contents to the file, we must flush the changes
     * to disk now.
     */
    Success = (*RegistryHive).FileFlush.unwrap()(
        RegistryHive,
        FileType,
        core::ptr::null_mut(),
        0,
    );
    if Success == 0 {
        return 0u8;
    }

    /*
     * Increment the secondary sequence number and update the checksum.
     */
    (*(*RegistryHive).BaseBlock).Sequence2 += 1;
    (*(*RegistryHive).BaseBlock).CheckSum = HvpHiveHeaderChecksum((*RegistryHive).BaseBlock);

    /* Write hive block */
    FileOffset = 0;
    Success = (*RegistryHive).FileWrite.unwrap()(
        RegistryHive,
        FileType,
        &mut FileOffset,
        (*RegistryHive).BaseBlock as *mut c_void,
        core::mem::size_of::<HBASE_BLOCK>() as _,
    );
    if Success == 0 {
        return 0u8;
    }

    /* Flush the hive immediately */
    Success = (*RegistryHive).FileFlush.unwrap()(
        RegistryHive,
        FileType,
        core::ptr::null_mut(),
        0,
    );
    if Success == 0 {
        return 0u8;
    }

    1u8
}

unsafe fn HvpWriteLog(RegistryHive: PHHIVE) -> u8 {
    let mut Success: u8;
    let mut FileOffset: u32;
    let mut BlockIndex: u32;
    let mut LastIndex: u32;
    let mut Block: *mut c_void;
    let BitmapSize: u32;
    let BufferSize: u32;
    let HeaderBuffer: *mut u8;
    let mut Ptr: *mut u8;

    HvpValidateBaseHeader(RegistryHive);

    if (*(*RegistryHive).BaseBlock).Sequence1 != (*(*RegistryHive).BaseBlock).Sequence2 {
        return 0u8;
    }

    BitmapSize = ROUND_UP(
        (core::mem::size_of::<u32>() + (*RegistryHive).DirtyVector.SizeOfBitMap as usize) as usize,
        HSECTOR_SIZE as usize,
    ) as u32;
    BufferSize = HV_LOG_HEADER_SIZE + BitmapSize;

    HeaderBuffer = (*RegistryHive).Allocate.unwrap()(BufferSize as _, 1u8, TAG_CM) as *mut u8;
    if HeaderBuffer.is_null() {
        return 0u8;
    }

    RtlZeroMemory(HeaderBuffer as *mut c_void, BufferSize as usize);

    (*(*RegistryHive).BaseBlock).Type = HFILE_TYPE_LOG;
    (*(*RegistryHive).BaseBlock).Sequence1 += 1;
    (*(*RegistryHive).BaseBlock).CheckSum = HvpHiveHeaderChecksum((*RegistryHive).BaseBlock);

    RtlCopyMemory(
        HeaderBuffer as *mut c_void,
        (*RegistryHive).BaseBlock as *const c_void,
        HV_LOG_HEADER_SIZE as usize,
    );
    Ptr = HeaderBuffer.add(HV_LOG_HEADER_SIZE as usize);

    *(Ptr as *mut u32) = HV_LOG_DIRTY_SIGNATURE;
    Ptr = Ptr.add(core::mem::size_of::<u32>());

    BlockIndex = 0;
    while BlockIndex < (*RegistryHive).Storage[Stable as usize].Length {
        LastIndex = BlockIndex;
        BlockIndex = RtlFindSetBits(
            core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
            1,
            BlockIndex,
        );
        if BlockIndex == !HV_CLEAN_BLOCK || BlockIndex < LastIndex {
            break;
        }

        *Ptr.add(BlockIndex as usize) = HV_LOG_DIRTY_BLOCK as u8;
        BlockIndex += 1;
    }

    FileOffset = 0;
    Success = (*RegistryHive).FileWrite.unwrap()(
        RegistryHive,
        HFILE_TYPE_LOG,
        &mut FileOffset,
        HeaderBuffer as *mut c_void,
        BufferSize as _,
    );
    (*RegistryHive).Free.unwrap()(HeaderBuffer as *mut c_void, 0);
    if Success == 0 {
        return 0u8;
    }

    FileOffset = BufferSize;
    BlockIndex = 0;
    while BlockIndex < (*RegistryHive).Storage[Stable as usize].Length {
        LastIndex = BlockIndex;
        BlockIndex = RtlFindSetBits(
            core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
            1,
            BlockIndex,
        );
        if BlockIndex == !HV_CLEAN_BLOCK || BlockIndex < LastIndex {
            break;
        }

        Block = (*(*RegistryHive).Storage[Stable as usize].BlockList.add(BlockIndex as usize)).BlockAddress as *mut c_void;

        Success = (*RegistryHive).FileWrite.unwrap()(
            RegistryHive,
            HFILE_TYPE_LOG,
            &mut FileOffset,
            Block,
            HBLOCK_SIZE as _,
        );
        if Success == 0 {
            return 0u8;
        }

        BlockIndex += 1;
        FileOffset += HBLOCK_SIZE;
    }

    Success = (*RegistryHive).FileFlush.unwrap()(
        RegistryHive,
        HFILE_TYPE_LOG,
        core::ptr::null_mut(),
        0,
    );
    if Success == 0 {
        return 0u8;
    }

    (*(*RegistryHive).BaseBlock).Sequence2 += 1;
    (*(*RegistryHive).BaseBlock).CheckSum = HvpHiveHeaderChecksum((*RegistryHive).BaseBlock);

    FileOffset = 0;
    Success = (*RegistryHive).FileWrite.unwrap()(
        RegistryHive,
        HFILE_TYPE_LOG,
        &mut FileOffset,
        (*RegistryHive).BaseBlock as *mut c_void,
        HV_LOG_HEADER_SIZE as _,
    );
    if Success == 0 {
        return 0u8;
    }

    Success = (*RegistryHive).FileFlush.unwrap()(
        RegistryHive,
        HFILE_TYPE_LOG,
        core::ptr::null_mut(),
        0,
    );
    if Success == 0 {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn HvSyncHive(RegistryHive: *mut HHIVE) -> u8 {
    #[cfg(not(feature = "bldr"))]
    let HardErrors: u8;

    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);
    ntkrnl_sys::ASSERT!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

    if (*RegistryHive).HiveFlags & HIVE_VOLATILE != 0 {
        DbgPrint(
            b"Hive 0x%p is volatile\n\0".as_ptr() as *const i8,
            RegistryHive as *const core::ffi::c_void,
        );
        return 1u8;
    }

    if RtlFindSetBits(core::ptr::addr_of_mut!((*RegistryHive).DirtyVector), 1, 0) == !HV_CLEAN_BLOCK {
        DbgPrint(
            b"The dirty vector has clean data, nothing to do\n\0".as_ptr() as *const i8,
        );
        return 1u8;
    }

    #[cfg(not(feature = "bldr"))]
    {
        HardErrors = IoSetThreadHardErrorMode(0u8);
    }

    #[cfg(not(feature = "bldr"))]
    { let _ts = core::ptr::addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp); KeQuerySystemTime(_ts as *mut _) };

    if (*RegistryHive).Log != 0 {
        if HvpWriteLog(RegistryHive) == 0 {
            DbgPrint(
                b"Failed to write a log whilst syncing the hive\n\0".as_ptr() as *const i8,
            );
            #[cfg(not(feature = "bldr"))]
            IoSetThreadHardErrorMode(HardErrors);
            return 0u8;
        }
    }

    if HvpWriteHive(RegistryHive, 1u8, HFILE_TYPE_PRIMARY) == 0 {
        DbgPrint(
            b"Failed to write the primary hive\n\0".as_ptr() as *const i8,
        );
        #[cfg(not(feature = "bldr"))]
        IoSetThreadHardErrorMode(HardErrors);
        return 0u8;
    }

    if (*RegistryHive).Alternate != 0 {
        if HvpWriteHive(RegistryHive, 1u8, HFILE_TYPE_ALTERNATE) == 0 {
            DbgPrint(
                b"Failed to write the alternate hive\n\0".as_ptr() as *const i8,
            );
            #[cfg(not(feature = "bldr"))]
            IoSetThreadHardErrorMode(HardErrors);
            return 0u8;
        }
    }

    RtlClearAllBits(core::ptr::addr_of_mut!((*RegistryHive).DirtyVector) as *mut RTL_BITMAP);
    (*RegistryHive).DirtyCount = 0;

    #[cfg(not(feature = "bldr"))]
    IoSetThreadHardErrorMode(HardErrors);
    return 1u8;
}
#[no_mangle]
pub unsafe extern "system" fn HvHiveWillShrink(
    _RegistryHive: *mut HHIVE,
) -> i32 {
    // No shrinking yet
    DbgPrint(b"HvHiveWillShrink is unimplemented\n\0".as_ptr() as *const i8);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn HvWriteHive(RegistryHive: *mut HHIVE) -> i32 {
    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);
    ntkrnl_sys::ASSERT!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

    #[cfg(not(feature = "bldr"))]
    {
        // Update hive header modification time
        KeQuerySystemTime({ let _ts = addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp); _ts as *mut _ });
    }

    // Update hive file
    if HvpWriteHive(RegistryHive, 0, HFILE_TYPE_PRIMARY) == 0 {
        DbgPrint(b"Failed to write the hive\n\0".as_ptr() as *const i8);
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn HvWriteAlternateHive(RegistryHive: *mut HHIVE) -> i32 {
    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);
    ntkrnl_sys::ASSERT!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);
    ntkrnl_sys::ASSERT!((*RegistryHive).Alternate != 0);
    #[cfg(not(feature = "bldr"))]
    {
        { let _ts = core::ptr::addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp); KeQuerySystemTime(_ts as *mut _) };
    }
    if HvpWriteHive(RegistryHive, 0, HFILE_TYPE_ALTERNATE) == 0 {
        DbgPrint(b"Failed to write the alternate hive\n\0".as_ptr() as *const i8);
        return 0;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn HvSyncHiveFromRecover(RegistryHive: *mut HHIVE) -> i32 {
    unsafe {
        ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);
        ntkrnl_sys::ASSERT!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

        HvpWriteHive(RegistryHive, 1u8, HFILE_TYPE_PRIMARY) as i32
    }
}
