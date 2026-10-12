//! The `FatGenerateFileId*` family — file IDs derived from a dirent's on-disk location.
//!
//! These are MACROS in the C (`fatprocs.h:3045-3085`, `fat.h:559`), never functions.
//! The translation emitted `extern "system" fn` declarations for them, so they became
//! unresolved symbols, and the cohab stub generator "resolved" them into
//! `KeBugCheckEx(0x23, 0xDEAD0000)` — a crash-on-first-call wearing the costume of a
//! link fix. arm64 boots a FAT32 ramdisk (fastfat IS the system volume there), so this
//! was live, not latent: the kmtest `IoFilesystem` test bugchecked the instant it asked
//! for FileInternalInformation, and the directory-enumeration twin below sat one
//! `FindFirstFile` behind it.
//!
//! The lesson generalises past fastfat: an `extern` declaration in translated Rust is a
//! CLAIM that a symbol exists, and nothing checks it. Grep the headers for
//! `#define <name>(` before trusting one — and treat a generated bugcheck stub as an
//! unimplemented function, not as a resolved reference.

use crate::types::*;
// One declaration per external symbol: externs.rs.
use crate::externs::FatIsFat32;
use core::ffi::c_void;

extern "system" {
}

/// First 4 bytes of the 8.3 name for `.` and `..`, as they sit on disk
/// (`fatprocs.h:3074`). FAT pads short names with spaces, so `.` is `". "`.
const FATDOT: u32 = 0x2020202E;
const FATDOTDOT: u32 = 0x20202E2E;

/// `FatGetLboFromIndex` (`fat.h:559`) — cluster index to logical byte offset.
///
/// The `- 2` is done in ULONG width on purpose: FAT reserves indices 0 and 1, and the C
/// subtracts before widening, so an index below 2 wraps to a huge positive LBO rather
/// than going negative. `wrapping_sub` keeps that; a plain `as i64 - 2` would not.
#[inline]
unsafe fn fat_get_lbo_from_index(Vcb: PVCB, FatIndex: u32) -> LBO {
    let alloc = &(*Vcb).AllocationSupport;
    alloc.FileAreaLbo + ((FatIndex.wrapping_sub(2) as LBO) << (alloc.LogOfBytesPerCluster as u32))
}

/// `FatGenerateFileIdFromDirentOffset` (`fatprocs.h:3055`).
///
/// A NULL parent commutes to 0 — that only happens for an illegal root `..` entry.
#[inline]
pub unsafe fn fat_generate_file_id_from_dirent_offset(ParentDcb: PDCB, DirentOffset: VBO) -> LBO {
    if ParentDcb.is_null() {
        return 0;
    }

    let Vcb = (*ParentDcb).Vcb;

    // NodeType(Ptr) is `((PNODE_TYPE_CODE)Ptr)->NodeTypeCode` — the first field of the
    // FCB header. Read it directly rather than through the C `NodeType` shim, whose
    // return is CSHORT while the crate's extern declarations call it u32; comparing a
    // 16-bit result against FAT_NTC_ROOT_DCB through a widened return leaves the upper
    // bits undefined.
    let node_type = *(ParentDcb as *const u16);

    let base = if node_type != crate::glue::FAT_NTC_ROOT_DCB as u16 || FatIsFat32(Vcb) != 0 {
        fat_get_lbo_from_index(Vcb, (*ParentDcb).FirstClusterOfFile)
    } else {
        // A non-FAT32 root directory lives at a fixed Lbo, outside the cluster area.
        (*Vcb).AllocationSupport.RootDirectoryLbo
    };

    base + DirentOffset as LBO
}

/// `FatGenerateFileIdFromFcb` (`fatprocs.h:3067`).
#[inline]
pub unsafe fn fat_generate_file_id_from_fcb(Fcb: PFCB) -> LBO {
    fat_generate_file_id_from_dirent_offset(
        (*Fcb).ParentDcb,
        (*Fcb).DirentOffsetWithinDirectory,
    )
}

/// `FatGenerateFileIdFromDirentAndOffset` (`fatprocs.h:3078`) — the enumeration form,
/// which has to special-case `.` and `..` so they report the same ID as the directory
/// they alias.
#[inline]
pub unsafe fn fat_generate_file_id_from_dirent_and_offset(
    Dcb: PDCB,
    Dirent: *const c_void,
    DirentOffset: VBO,
) -> LBO {
    // DIRENT.FileName is at offset 0 and DIRENTs are quadword-aligned (fat.h:304-326),
    // so the C's `*(PULONG)Dirent->FileName` is a plain aligned 4-byte read.
    let name0 = *(Dirent as *const u32);

    if name0 == FATDOT {
        fat_generate_file_id_from_fcb(Dcb)
    } else if name0 == FATDOTDOT {
        let parent = (*Dcb).ParentDcb;
        if parent.is_null() {
            0
        } else {
            fat_generate_file_id_from_fcb(parent)
        }
    } else {
        fat_generate_file_id_from_dirent_offset(Dcb, DirentOffset)
    }
}
