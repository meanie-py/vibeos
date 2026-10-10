use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CcMapData, PLARGE_INTEGER, RtlOemToUnicodeN};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CD_NAME, COMPOUND_DIRENT, DIRENT, DIRENT_ENUM_CONTEXT, DIRENT_FLAG_ALLOC_BUFFER, FCB, FCB_INDEX, FILE_CONTEXT_MULTIPLE_DIRENTS, FILE_ENUM_CONTEXT, IRP_CONTEXT, PDIRENT, PIRP_CONTEXT, RIFF_HEADER, VCB, VCB_STATE_CDXA, VCB_STATE_JOLIET, XA_EXTENT_TYPE, _XA_EXTENT_TYPE_CDAudio as CDAudio, _XA_EXTENT_TYPE_Form1Data as Form1Data, _XA_EXTENT_TYPE_Mode2Form2Data as Mode2Form2Data};
use ntkrnl_sys::ddk::{_UNICODE_STRING, FSRTL_ADVANCED_FCB_HEADER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag};
use crate::glue::{CDFS_BUG_CHECK_DIRSUP, STATUS_FILE_CORRUPT_ERROR, TAG_DIRENT_NAME};
use crate::types::{CD_ATTRIBUTE_MULTI, RAW_DIRENT, SECTOR_SHIFT, SYSTEM_USE_XA, SYSTEM_USE_XA_DA, SYSTEM_USE_XA_FORM2, SYSTEM_XA_SIGNATURE, XA_SECTOR_SIZE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdUnicodeDirectoryNames};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CdPagedPool};

extern "C" {
    fn CdRawDirent( IrpContext: *mut IRP_CONTEXT, DirContext: *mut DIRENT_ENUM_CONTEXT, ) -> *mut RAW_DIRENT;
}
extern "C" {
    fn CdRawDirentFlags( IrpContext: *mut IRP_CONTEXT, RawDirent: *mut RAW_DIRENT, ) -> u32;
}
extern "C" {
    fn CopyUchar4( dest: *mut u32, src: *const u8, );
}
extern "C" {
    fn CdRaiseStatus( IrpContext: *mut IRP_CONTEXT, Status: i32, ) -> !;
}
extern "C" {
    fn WordAlign(offset: u32) -> u32;
}
extern "C" {
    fn SectorBlockOffset(Vcb: *mut VCB, Offset: ULONG) -> ULONG;
}
extern "C" {
    fn Int32x32To64(a: i32, b: i32) -> LONGLONG;
}
extern "C" {
    fn CdConvertBigToLittleEndian(IrpContext: PIRP_CONTEXT, Src: *const u8, SrcLen: ULONG, Dst: *mut u8);
}
extern "C" {
    fn CdConvertNameToCdName(IrpContext: PIRP_CONTEXT, FileName: *mut CD_NAME);
}
extern "C" {
    fn CdUpcaseName(IrpContext: PIRP_CONTEXT, ExactName: *const CD_NAME, UpcasedName: *mut CD_NAME);
}
extern "C" {
    fn CdIsLegalName(IrpContext: PIRP_CONTEXT, Name: *const _UNICODE_STRING) -> BOOLEAN;
}
extern "C" {
    fn CdFreePool(Pool: *mut *mut core::ffi::c_void);
}
extern "C" {
    fn NT_ASSERT(condition: u32);
}
extern "C" {
}
extern "C" {
    fn CdCleanupDirContext(IrpContext: *mut IRP_CONTEXT, DirContext: *mut DIRENT_ENUM_CONTEXT);
}
extern "C" {
    fn CdCleanupDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    fn CdVerifyOrCreateDirStreamFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdLookupInitialFileDirent(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, FileContext: *mut FILE_ENUM_CONTEXT, DirentOffset: u32); // def takes u32, not i64 (arg-width)
}
extern "C" {
    fn CdIsNameInExpression(IrpContext: *mut IRP_CONTEXT, Name: *mut CD_NAME, Expression: *mut CD_NAME, IgnoreWild: u32, IgnoreCase: u8) -> u8;
}
extern "C" {
    fn SectorTruncate(Offset: u32) -> u32; // defined -> u32; i64 decl leaks garbage EDX on i386
}
extern "C" {
    fn SectorOffset(Offset: u32) -> u32;
}
extern "C" {
    fn CdUnpinData(IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
}
extern "C" {
    fn CdShortNameDirentOffset(IrpContext: *mut IRP_CONTEXT, FileName: *mut _UNICODE_STRING) -> u32;
}
extern "C" {
    fn CdIs8dot3Name(IrpContext: *mut IRP_CONTEXT, Name: _UNICODE_STRING) -> i32;
}
extern "C" {
    fn CdGenerate8dot3Name(IrpContext: *mut IRP_CONTEXT, Source: *mut _UNICODE_STRING, Offset: u32, Buffer: *mut u16, Length: *mut u16);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_DIRSUP;

const MIN_RAW_DIRENT_LEN: u32 = 34;

const SHORT_NAME_SHIFT: u32 = 5;
const CD_ATTRIBUTE_ASSOC: u32 = 0x04;
const CD_ATTRIBUTE_DIRECTORY: u32 = 0x02;
const DIRENT_FLAG_CONSTANT_ENTRY: u32 = 0x02;
const DIRENT_FLAG_NOT_PERSISTENT_VAL: u32 = 0;

const SECTOR_SIZE: u32 = 2048;

// FsRtl FlagOn/SetFlag/ClearFlag are C macros (operate on the lvalue directly),
// not exported functions; inline them so they work on the actual field widths.
#[inline]
unsafe fn FlagOnU32(flags: u32, flag: u32) -> bool {
    (flags & flag) != 0
}
#[inline]
unsafe fn SetFlagU8(flags: *mut u8, flag: u32) {
    *flags |= flag as u8;
}
#[inline]
unsafe fn ClearFlagU8(flags: *mut u8, flag: u32) {
    *flags &= !(flag as u8);
}

// FCB field accessors. In the on-disk-mirroring bindgen layout, the FSRTL
// header lives in the leading anonymous union and the FCB_INDEX overlay
// (FileObject/StreamOffset) lives in the trailing anonymous union.
#[inline]
unsafe fn FcbHeader(Fcb: *mut FCB) -> *mut FSRTL_ADVANCED_FCB_HEADER {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_1) as *mut FSRTL_ADVANCED_FCB_HEADER
}
#[inline]
unsafe fn FcbIndex(Fcb: *mut FCB) -> *mut FCB_INDEX {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_INDEX
}

#[no_mangle]
pub unsafe extern "C" fn CdLookupDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    DirentOffset: u32,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
) {
    // PAGED_CODE();
    let mut BaseOffset: i64;

    (*DirContext).BaseOffset = SectorTruncate(DirentOffset) as u32;
    BaseOffset = (*DirContext).BaseOffset as i64;
    (*DirContext).DataLength = SECTOR_SIZE;
    (*DirContext).SectorOffset = SectorOffset(DirentOffset);

    if (*DirContext).DataLength as i64 > ((*FcbHeader(Fcb)).FileSize.QuadPart - BaseOffset)  {
        (*DirContext).DataLength = ((*FcbHeader(Fcb)).FileSize.QuadPart - BaseOffset) as u32;
    }

    CcMapData(
        (*FcbIndex(Fcb)).FileObject,
        &mut BaseOffset as *mut i64 as PLARGE_INTEGER,
        (*DirContext).DataLength,
        1, // TRUE
        core::ptr::addr_of_mut!((*DirContext).Bcb),
        core::ptr::addr_of_mut!((*DirContext).Sector),
    );

    if (*DirContext).Bcb.is_null() || (*DirContext).Sector.is_null()  {
        CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }

    (*DirContext).NextDirentOffset = CdCheckRawDirentBounds(IrpContext, DirContext);
}

#[no_mangle]
pub unsafe extern "C" fn CdLookupNextDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    CurrentDirContext: *mut DIRENT_ENUM_CONTEXT,
    NextDirContext: *mut DIRENT_ENUM_CONTEXT,
) -> i32 {
    let mut CurrentBaseOffset: i64 = (*CurrentDirContext).BaseOffset as i64;
    let mut TempUlong: u32;
    let mut FoundDirent: i32 = 0;

    // PAGED_CODE(); (ignored as no-op)

    if ((*CurrentDirContext).BaseOffset != (*NextDirContext).BaseOffset) ||
       ((*NextDirContext).Bcb).is_null()
    {
        // Unpin the current target Bcb and map the next sector.
        CdUnpinData(IrpContext, &raw mut (*NextDirContext).Bcb as *mut *mut core::ffi::c_void);

        CcMapData(
            (*FcbIndex(Fcb)).FileObject,
            &raw mut CurrentBaseOffset as PLARGE_INTEGER,
            (*CurrentDirContext).DataLength,
            1u32, // TRUE
            &raw mut (*NextDirContext).Bcb as *mut *mut core::ffi::c_void,
            &raw mut (*NextDirContext).Sector as *mut *mut core::ffi::c_void,
        );

        // Verify that the mapping succeeded and we got a valid sector pointer.
        if ((*NextDirContext).Bcb.is_null()) || ((*NextDirContext).Sector.is_null()) {
            CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        // Copy the data length and sector offset.
        (*NextDirContext).DataLength = (*CurrentDirContext).DataLength;
        (*NextDirContext).BaseOffset = (*CurrentDirContext).BaseOffset;
    }

    // Now move to the same offset in the sector.
    (*NextDirContext).SectorOffset = (*CurrentDirContext).SectorOffset;

    // If the value is zero then unmap the current sector and set up
    // the base offset to the beginning of the next sector.
    if (*CurrentDirContext).NextDirentOffset == 0 {
        CurrentBaseOffset = (*NextDirContext).BaseOffset as i64 + (*NextDirContext).DataLength as i64;

        // Unmap the current sector. We test the value of the Bcb in the
        // loop below to see if we need to read in another sector.
        CdUnpinData(IrpContext, &raw mut (*NextDirContext).Bcb as *mut *mut core::ffi::c_void);
    } else {
        // There is another possible dirent in the current sector. Update the
        // enumeration context to reflect this.
        (*NextDirContext).SectorOffset += (*CurrentDirContext).NextDirentOffset;
    }

    // Now loop until we find the next possible dirent or walk off the directory.
    loop {
        // If we don't currently have a sector mapped then map the
        // directory at the current offset.
        if (*NextDirContext).Bcb.is_null() {
            TempUlong = SECTOR_SIZE;

            if TempUlong > ((*FcbHeader(Fcb)).FileSize.QuadPart as i64 - CurrentBaseOffset) as u32 {
                TempUlong = ((*FcbHeader(Fcb)).FileSize.QuadPart as i64 - CurrentBaseOffset) as u32;

                // If the length is zero then there is no dirent.
                if TempUlong == 0 {
                    break;
                }
            }

            CcMapData(
                (*FcbIndex(Fcb)).FileObject,
                &raw mut CurrentBaseOffset as PLARGE_INTEGER,
                TempUlong,
                1u32, // TRUE
                &raw mut (*NextDirContext).Bcb as *mut *mut core::ffi::c_void,
                &raw mut (*NextDirContext).Sector as *mut *mut core::ffi::c_void,
            );

            // Verify that the mapping succeeded and we got a valid sector pointer.
            if ((*NextDirContext).Bcb.is_null()) || ((*NextDirContext).Sector.is_null()) {
                CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
            }

            (*NextDirContext).BaseOffset = CurrentBaseOffset as u32;
            (*NextDirContext).SectorOffset = 0;
            (*NextDirContext).DataLength = TempUlong;
        }

        // The CDFS spec allows for sectors in a directory to contain all zeroes.
        // In this case we need to move to the next sector. So look at the
        // current potential dirent for a zero length. Move to the next
        // dirent if length is zero.
        let raw_dirent_ptr = CdRawDirent(IrpContext, NextDirContext);
        if (raw_dirent_ptr as *const u8).read() != 0 {
            FoundDirent = 1; // TRUE
            break;
        }

        CurrentBaseOffset = (*NextDirContext).BaseOffset as i64 + (*NextDirContext).DataLength as i64;
        CdUnpinData(IrpContext, &raw mut (*NextDirContext).Bcb as *mut *mut core::ffi::c_void);
    }

    // Check the dirent bounds if we found a dirent.
    if FoundDirent != 0 {
        (*NextDirContext).NextDirentOffset = CdCheckRawDirentBounds(IrpContext, NextDirContext);
    }

    FoundDirent
}

#[no_mangle]
pub unsafe extern "C" fn CdUpdateDirentFromRawDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
    Dirent: *mut DIRENT,
) {
    // PAGED_CODE();
    let _ = Fcb; // UNREFERENCED_PARAMETER( Fcb );

    let RawDirent = CdRawDirent(IrpContext, DirContext);

    // Clear all current state flags except the flag indicating that
    // we allocated a name string.
    (*Dirent).Flags &= !(DIRENT_FLAG_NOT_PERSISTENT_VAL as u8);

    // The dirent offset is the sum of the start of the sector and the
    // sector offset.
    (*Dirent).DirentOffset = (*DirContext).BaseOffset + (*DirContext).SectorOffset;

    // Copy the dirent length from the raw dirent.
    (*Dirent).DirentLength = (*RawDirent).DirLen as u32;

    // The starting offset on disk is computed by finding the starting
    // logical block and stepping over the Xar block.
    CopyUchar4(
        core::ptr::addr_of_mut!((*Dirent).StartingOffset),
        &raw const (*RawDirent).FileLoc as *const u8,
    );
    (*Dirent).StartingOffset += (*RawDirent).XarLen as u32;

    // Do a safe copy to get the data length.
    CopyUchar4(
        core::ptr::addr_of_mut!((*Dirent).DataLength),
        &raw const (*RawDirent).DataLen as *const u8,
    );

    // Save a pointer to the time stamps.
    (*Dirent).CdTime = (*RawDirent).RecordTime.as_ptr() as *mut i8;

    // Copy the dirent flags.
    (*Dirent).DirentFlags = CdRawDirentFlags(IrpContext, RawDirent) as u8;

    // For both the file unit and interleave skip we want to take the
    // logical block count.
    (*Dirent).FileUnitSize = 0;
    (*Dirent).InterleaveGapSize = 0;

    if (*RawDirent).IntLeaveSize != 0 {
        (*Dirent).FileUnitSize = (*RawDirent).IntLeaveSize as u32;
        (*Dirent).InterleaveGapSize = (*RawDirent).IntLeaveSkip as u32;
    }

    // Get the name length and remember a pointer to the start of the
    // name string.
    // Check that the name length is non-zero.
    if (*RawDirent).FileIdLen == 0 {
        CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }

    (*Dirent).FileNameLen = (*RawDirent).FileIdLen as u32;
    (*Dirent).FileName = (*RawDirent).FileId.as_ptr() as *mut i8;

    // If there are any remaining bytes at the end of the dirent then
    // there may be a system use area.
    (*Dirent).XAAttributes = 0;
    (*Dirent).XAFileNumber = 0;
    (*Dirent).ExtentType = Form1Data;
    (*Dirent).SystemUseOffset = 0;

    let file_id_offset = core::mem::offset_of!(RAW_DIRENT, FileId) as u32;
    let name_len = (*Dirent).FileNameLen as u32;
    let required_length = file_id_offset + name_len + 1;

    if !FlagOnU32((*Dirent).DirentFlags as u32, CD_ATTRIBUTE_DIRECTORY)
        && (*Dirent).DirentLength > required_length
    {
        (*Dirent).SystemUseOffset = WordAlign(file_id_offset + name_len);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdUpdateDirentName(
    IrpContext: PIRP_CONTEXT,
    Dirent: PDIRENT,
    IgnoreCase: ULONG,
) -> () {
    unsafe {
        let DirectoryValue: UCHAR;
        let mut Length: ULONG;
        let Status: NTSTATUS;

        PAGED_CODE!();

        if ((*Dirent).FileNameLen == 1) && FlagOnU32((*Dirent).DirentFlags as u32, CD_ATTRIBUTE_DIRECTORY) {
            DirectoryValue = *((*Dirent).FileName as *const u8);

            if (DirectoryValue == 0) || (DirectoryValue == 1) {
                if FlagOnU32((*Dirent).Flags as u32, DIRENT_FLAG_ALLOC_BUFFER) {
                    CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }

                (*Dirent).CdFileName.FileName = CdUnicodeDirectoryNames[DirectoryValue as usize];

                (*Dirent).CdFileName.VersionString.Length = 0;

                (*Dirent).CdCaseFileName = core::ptr::read(&(*Dirent).CdFileName as *const CD_NAME);

                SetFlagU8(&mut (*Dirent).Flags, DIRENT_FLAG_CONSTANT_ENTRY);

                return;
            }
        }

        ClearFlagU8(&mut (*Dirent).Flags, DIRENT_FLAG_CONSTANT_ENTRY);

        Length = (*Dirent).FileNameLen;

        if IgnoreCase != 0 {
            Length = Length.wrapping_mul(2);
        }

        if !FlagOnU32((*(*IrpContext).Vcb).VcbState, VCB_STATE_JOLIET) {
            Length = Length.wrapping_mul(core::mem::size_of::<WCHAR>() as ULONG);
        }

        if !FlagOnU32((*Dirent).Flags as u32, DIRENT_FLAG_ALLOC_BUFFER)
            && (Length <= core::mem::size_of::<[u16; 32]>() as ULONG)
        {
            (*Dirent).CdFileName.FileName.MaximumLength = core::mem::size_of::<[u16; 32]>() as USHORT;
            (*Dirent).CdFileName.FileName.Buffer = (*Dirent).NameBuffer.as_mut_ptr();
        } else {
            if Length > (*Dirent).CdFileName.FileName.MaximumLength as ULONG {
                if FlagOnU32((*Dirent).Flags as u32, DIRENT_FLAG_ALLOC_BUFFER) {
                    CdFreePool(&mut (*Dirent).CdFileName.FileName.Buffer as *mut _ as *mut _);
                    ClearFlagU8(&mut (*Dirent).Flags, DIRENT_FLAG_ALLOC_BUFFER);
                }

                (*Dirent).CdFileName.FileName.Buffer = FsRtlAllocatePoolWithTag(
                    CdPagedPool,
                    Length,
                    TAG_DIRENT_NAME,
                ) as *mut WCHAR;

                SetFlagU8(&mut (*Dirent).Flags, DIRENT_FLAG_ALLOC_BUFFER);

                (*Dirent).CdFileName.FileName.MaximumLength = Length as USHORT;
            }
        }

        if !FlagOnU32((*(*IrpContext).Vcb).VcbState, VCB_STATE_JOLIET) {
            Status = RtlOemToUnicodeN(
                (*Dirent).CdFileName.FileName.Buffer,
                (*Dirent).CdFileName.FileName.MaximumLength as u32,
                &mut Length,
                (*Dirent).FileName as *const i8,
                (*Dirent).FileNameLen,
            );

            ntkrnl_sys::__analysis_assert!(((Status == STATUS_SUCCESS) as BOOLEAN) as _);
            NT_ASSERT(((Status == STATUS_SUCCESS) as BOOLEAN) as _);
            (*Dirent).CdFileName.FileName.Length = Length as USHORT;
        } else {
            CdConvertBigToLittleEndian(
                IrpContext,
                (*Dirent).FileName as *const u8,
                (*Dirent).FileNameLen,
                (*Dirent).CdFileName.FileName.Buffer as *mut u8,
            );

            (*Dirent).CdFileName.FileName.Length = (*Dirent).FileNameLen as USHORT;
        }

        CdConvertNameToCdName(IrpContext, &mut (*Dirent).CdFileName);

        if (*Dirent).CdFileName.FileName.Length == 0 {
            CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        // if the filename ends with a period, back up one character
        if (*Dirent)
            .CdFileName
            .FileName
            .Buffer
            .add(
                ((*Dirent).CdFileName.FileName.Length as usize - core::mem::size_of::<WCHAR>()) / 2,
            )
            .read()
            == 0x2Eu16
        {
            if (*Dirent).CdFileName.VersionString.Length != 0 {
                let NewVersion: *mut WCHAR = (*Dirent)
                    .CdFileName
                    .FileName
                    .Buffer
                    .add((*Dirent).CdFileName.FileName.Length as usize / 2);

                // RtlMoveMemory(NewVersion - 1, NewVersion, VersionString.Length + sizeof(WCHAR))
                let count = (*Dirent).CdFileName.VersionString.Length as usize
                    + core::mem::size_of::<WCHAR>();
                let src: *const u8 = NewVersion as *const u8;
                let dst: *mut u8 = (NewVersion as *mut u8).sub(core::mem::size_of::<WCHAR>());

                // Overlapping move: ptr::copy is memmove. A hand-rolled byte
                // loop is the shape LLVM folds back into a memmove call.
                core::ptr::copy(src, dst, count);

                (*Dirent).CdFileName.VersionString.Buffer = NewVersion;
            }

            (*Dirent).CdFileName.FileName.Length -= core::mem::size_of::<WCHAR>() as USHORT;
        }

        if CdIsLegalName(IrpContext, &(*Dirent).CdFileName.FileName) == 0 {
            CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        if IgnoreCase == 0 {
            (*Dirent).CdCaseFileName = core::ptr::read(&(*Dirent).CdFileName as *const CD_NAME);
        } else {
            // C uses Add2Ptr (BYTE arithmetic): Buffer + MaximumLength/2 *bytes*.
            // Buffer is *mut u16, so .add() scales by 2 — cast to *mut u8 first or
            // we advance MaximumLength bytes (2x too far), spilling the upcased name
            // one full NameBuffer past its end onto Dirents[n+1].DirContext.Bcb.
            (*Dirent).CdCaseFileName.FileName.Buffer = ((*Dirent)
                .CdFileName
                .FileName
                .Buffer as *mut u8)
                .add((*Dirent).CdFileName.FileName.MaximumLength as usize / 2)
                as *mut u16;
            (*Dirent).CdCaseFileName.FileName.MaximumLength =
                (*Dirent).CdFileName.FileName.MaximumLength / 2;

            CdUpcaseName(
                IrpContext,
                &(*Dirent).CdFileName,
                &mut (*Dirent).CdCaseFileName,
            );
        }

        return;
    }
}

#[no_mangle]
pub unsafe extern "C" fn CdFindFile(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Name: *mut CD_NAME,
    IgnoreCase: i32,
    FileContext: *mut FILE_ENUM_CONTEXT,
    MatchingName: *mut *mut CD_NAME,
) -> i32 {
    let mut Dirent: *mut DIRENT;
    let ShortNameDirentOffset: u32;
    let mut Found: i32 = 0; // FALSE

    // PAGED_CODE(); // omitted, debug only

    CdVerifyOrCreateDirStreamFile(IrpContext, Fcb);

    ShortNameDirentOffset = CdShortNameDirentOffset(
        IrpContext,
        core::ptr::addr_of_mut!((*Name).FileName),
    );

    CdLookupInitialFileDirent(
        IrpContext,
        Fcb,
        FileContext,
        (*FcbIndex(Fcb)).StreamOffset as u32,
    );

    loop {
        Dirent = core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent);

        if !FlagOnU32((*Dirent).DirentFlags as u32, CD_ATTRIBUTE_ASSOC | CD_ATTRIBUTE_DIRECTORY) {
            CdUpdateDirentName(IrpContext, Dirent, IgnoreCase as u32);

            if FlagOnU32((*Dirent).Flags as u32, DIRENT_FLAG_CONSTANT_ENTRY) {
                if CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext) == 0 {
                    break;
                }
                continue;
            }

            if CdIsNameInExpression(
                IrpContext,
                core::ptr::addr_of_mut!((*Dirent).CdCaseFileName) as *mut CD_NAME,
                Name,
                0,
                1,
            ) != 0 {
                *MatchingName = core::ptr::addr_of_mut!((*Dirent).CdCaseFileName) as *mut CD_NAME;
                Found = 1;
                break;
            }

            if ((*Dirent).DirentOffset >> SHORT_NAME_SHIFT) as u32 == ShortNameDirentOffset
                && (*Name).VersionString.Length as u32 == 0
                && CdIs8dot3Name(
                    IrpContext,
                    (*Dirent).CdFileName.FileName,
                ) == 0
            {
                CdGenerate8dot3Name(
                    IrpContext,
                    core::ptr::addr_of_mut!((*Dirent).CdCaseFileName.FileName),
                    (*Dirent).DirentOffset,
                    (*FileContext).ShortName.FileName.Buffer,
                    core::ptr::addr_of_mut!((*FileContext).ShortName.FileName.Length),
                );

                if CdIsNameInExpression(
                    IrpContext,
                    core::ptr::addr_of_mut!((*FileContext).ShortName) as *mut CD_NAME,
                    Name,
                    0,
                    0,
                ) != 0 {
                    *MatchingName = core::ptr::addr_of_mut!((*FileContext).ShortName) as *mut CD_NAME;
                    Found = 1;
                    break;
                }
            }
        }

        if CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext) == 0 {
            break;
        }
    }

    if Found != 0 {
        CdLookupLastFileDirent(IrpContext, Fcb, FileContext);
    }

    Found
}
#[no_mangle]
pub unsafe extern "C" fn CdFindDirectory(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Name: *mut CD_NAME,
    IgnoreCase: u8,
    FileContext: *mut FILE_ENUM_CONTEXT,
) -> u8 {
    let mut found: u8 = 0;

    CdVerifyOrCreateDirStreamFile(IrpContext, Fcb);

    CdLookupInitialFileDirent(IrpContext, Fcb, FileContext, (*FcbIndex(Fcb)).StreamOffset as u32);

    loop {
        let dirent: *mut DIRENT = core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent);

        if FlagOnU32((*dirent).DirentFlags as u32, CD_ATTRIBUTE_DIRECTORY) {
            CdUpdateDirentName(IrpContext, dirent, IgnoreCase as u32);

            if FlagOnU32((*dirent).Flags as u32, DIRENT_FLAG_CONSTANT_ENTRY) {
                // C is a do/while: `continue` jumps to the while-condition which ADVANCES
                // the enumeration. A bare `continue` here re-reads the SAME InitialDirent
                // (no advance) -> infinite loop on every "."/".." entry. Advance first,
                // mirroring CdFindFile.
                if CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext) == 0 {
                    break;
                }
                continue;
            }

            if CdIsNameInExpression(
                IrpContext,
                core::ptr::addr_of_mut!((*dirent).CdCaseFileName),
                Name,
                0,
                1u8,
            ) != 0 {
                found = 1;
                break;
            }
        }

        if CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext) == 0 {
            break;
        }
    }

    found
}

#[no_mangle]
pub unsafe extern "C" fn CdLookupLastFileDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FileContext: *mut FILE_ENUM_CONTEXT,
) {

    let mut ExtentType: XA_EXTENT_TYPE = Form1Data;
    let mut CurrentCompoundDirent: *mut COMPOUND_DIRENT;
    let mut CurrentDirent: *mut DIRENT;
    let mut FirstPass: i32 = 1;
    let mut FoundDirent: i32;

    CurrentCompoundDirent = (*FileContext).InitialDirent;

    loop {
        CurrentDirent = &raw mut (*CurrentCompoundDirent).Dirent;

        if ((*CurrentDirent).SystemUseOffset != 0) &&
           (((*((*Fcb).Vcb)).VcbState & VCB_STATE_CDXA) != 0) &&
           {
               let raw_dirent: *mut RAW_DIRENT = {
                   let dc = &(*CurrentCompoundDirent).DirContext;
                   (dc.Sector as *mut u8).add(dc.SectorOffset as usize) as *mut RAW_DIRENT
               };
               (CdCheckForXAExtent(IrpContext, raw_dirent, CurrentDirent) as i32) != 0
           }
        {
            if FirstPass == 0 && ExtentType != (*CurrentDirent).ExtentType {
                CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
            }

            if (*((*Fcb).Vcb)).BlockSize != SECTOR_SIZE {
                if (SectorBlockOffset((*Fcb).Vcb, (*CurrentDirent).StartingOffset) != 0) ||
                   (SectorBlockOffset((*Fcb).Vcb, (*CurrentDirent).DataLength) != 0)
                {
                    CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }

                if ((*CurrentDirent).FileUnitSize != 0) &&
                   (SectorBlockOffset((*Fcb).Vcb, (*CurrentDirent).FileUnitSize) != 0 ||
                    SectorBlockOffset((*Fcb).Vcb, (*CurrentDirent).InterleaveGapSize) != 0)
                {
                    CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }
            }

            if FirstPass != 0 {
                (*FileContext).FileSize = core::mem::size_of::<RIFF_HEADER>() as LONGLONG;
            }

            (*FileContext).FileSize += Int32x32To64(
                ((*CurrentDirent).DataLength >> SECTOR_SHIFT) as i32,
                XA_SECTOR_SIZE as i32,
            );
        } else {
            if FirstPass == 0 && ExtentType != (*CurrentDirent).ExtentType {
                CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
            }

            (*FileContext).FileSize += (*CurrentDirent).DataLength as LONGLONG;
        }

        if !FlagOnU32((*CurrentDirent).DirentFlags as u32, CD_ATTRIBUTE_MULTI) {
            break;
        }

        ExtentType = (*CurrentDirent).ExtentType;

        let next_dir_context = &raw mut (*((*FileContext).CurrentDirent)).DirContext;
        FoundDirent = CdLookupNextDirent(
            IrpContext,
            Fcb,
            &raw mut (*CurrentCompoundDirent).DirContext,
            next_dir_context,
        );

        if FoundDirent == 0 {
            CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        CurrentCompoundDirent = (*FileContext).CurrentDirent;
        FirstPass = 0;

        CdUpdateDirentFromRawDirent(
            IrpContext,
            Fcb,
            &raw mut (*CurrentCompoundDirent).DirContext,
            &raw mut (*CurrentCompoundDirent).Dirent,
        );

        (*FileContext).Flags |= FILE_CONTEXT_MULTIPLE_DIRENTS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdCleanupFileContext(
    IrpContext: *mut IRP_CONTEXT,
    FileContext: *mut FILE_ENUM_CONTEXT,
) {
    let mut Count: u32 = 2;
    let mut CurrentCompoundDirent: *mut COMPOUND_DIRENT;

    PAGED_CODE!();

    UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);

    loop {
        CurrentCompoundDirent = (core::ptr::addr_of_mut!((*FileContext).Dirents) as *mut COMPOUND_DIRENT).add(Count as usize);

        CdCleanupDirContext(IrpContext, core::ptr::addr_of_mut!((*CurrentCompoundDirent).DirContext));
        CdCleanupDirent(IrpContext, core::ptr::addr_of_mut!((*CurrentCompoundDirent).Dirent));

        let old_count = Count;
        Count = Count.wrapping_sub(1);
        if old_count == 0 {
            break;
        }
    }
    return;
}
#[no_mangle]
pub unsafe extern "C" fn CdCheckRawDirentBounds(
    IrpContext: *mut IRP_CONTEXT,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
) -> u32 {
    let mut NextDirentOffset: u32;
    let RawDirent: *mut RAW_DIRENT;

    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);

    NT_ASSERT(((((*DirContext).DataLength - (*DirContext).SectorOffset) >= 1) as BOOLEAN) as _);

    if ((*DirContext).SectorOffset >= (*DirContext).DataLength) ||
        ((*DirContext).Sector.is_null())
    {
        CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }

    RawDirent = CdRawDirent(IrpContext, DirContext);

    if (*RawDirent).DirLen != 0 {
        if ((*RawDirent).DirLen as u32 > ((*DirContext).DataLength - (*DirContext).SectorOffset)) ||
            (((*RawDirent).DirLen as u32) < MIN_RAW_DIRENT_LEN) ||
            (((*RawDirent).DirLen as u32) < (MIN_RAW_DIRENT_LEN - 1 + (*RawDirent).FileIdLen as u32))
        {
            CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        NextDirentOffset = (*RawDirent).DirLen as u32;

        if NextDirentOffset == ((*DirContext).DataLength - (*DirContext).SectorOffset) {
            NextDirentOffset = 0;
        }
    } else {
        NextDirentOffset = 0;
    }

    return NextDirentOffset;
}
#[no_mangle]
pub unsafe extern "C" fn CdCheckForXAExtent(
    IrpContext: *mut IRP_CONTEXT,
    RawDirent: *mut RAW_DIRENT,
    Dirent: *mut DIRENT,
) -> XA_EXTENT_TYPE {
    let _ = IrpContext; // UNREFERENCED_PARAMETER

    let mut ExtentType: XA_EXTENT_TYPE = Form1Data;
    let DirentLen = (*Dirent).DirentLength;
    let SystemUseOffset = (*Dirent).SystemUseOffset;

    if DirentLen - SystemUseOffset >= core::mem::size_of::<SYSTEM_USE_XA>() as u32 {
        let SystemUseArea = (RawDirent as *mut u8).add(SystemUseOffset as usize) as *mut SYSTEM_USE_XA;

        if (*SystemUseArea).Signature == SYSTEM_XA_SIGNATURE as u16 {
            if ((*SystemUseArea).Attributes & SYSTEM_USE_XA_DA as u16) != 0 {
                ExtentType = CDAudio;
            } else if ((*SystemUseArea).Attributes & SYSTEM_USE_XA_FORM2 as u16) != 0 {
                ExtentType = Mode2Form2Data;
            }

            (*Dirent).XAAttributes = (*SystemUseArea).Attributes;
            (*Dirent).XAFileNumber = (*SystemUseArea).FileNumber;
        }
    }

    (*Dirent).ExtentType = ExtentType;
    ExtentType
}

#[no_mangle]
pub unsafe extern "C" fn CdFindFileByShortName(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Name: *mut CD_NAME,
    IgnoreCase: u8,
    ShortNameDirentOffset: u32,
    FileContext: *mut FILE_ENUM_CONTEXT,
) -> u8 {
    let mut Found: u8 = 0; // FALSE
    let mut Dirent: *mut DIRENT;

    let mut ThisShortNameDirentOffset: u32;

    // PAGED_CODE();

    //
    //  Make sure there is a stream file for this Fcb.
    //

    CdVerifyOrCreateDirStreamFile(IrpContext, Fcb);

    //
    //  Position ourselves at the start of the directory and update
    //

    CdLookupInitialFileDirent(IrpContext, Fcb, FileContext, (*FcbIndex(Fcb)).StreamOffset as u32);

    //
    //  Loop until we have found the entry or are beyond this dirent.
    //

    loop {
        //
        //  Compute the short name dirent offset for the current dirent.
        //

        Dirent = core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent);
        ThisShortNameDirentOffset = (*Dirent).DirentOffset >> SHORT_NAME_SHIFT;

        //
        //  If beyond the target then exit.
        //

        if ThisShortNameDirentOffset > ShortNameDirentOffset {
            break;
        }

        //
        //  If equal to the target then check if we have a name match.
        //  We will either match or fail here.
        //

        if ThisShortNameDirentOffset == ShortNameDirentOffset {
            //
            //  If this is an associated file then get out.
            //

            if FlagOnU32((*Dirent).DirentFlags as u32, CD_ATTRIBUTE_ASSOC) {
                break;
            }

            //
            //  Update the name in the dirent and check if it is not
            //  an 8.3 name.
            //

            CdUpdateDirentName(IrpContext, Dirent, IgnoreCase as u32);

            if CdIs8dot3Name(
                IrpContext,
                (*Dirent).CdFileName.FileName,
            ) != 0 {
                break;
            }

            //
            //  Generate the 8.3 name see if it matches our input name.
            //

            CdGenerate8dot3Name(
                IrpContext,
                core::ptr::addr_of_mut!((*Dirent).CdCaseFileName.FileName),
                (*Dirent).DirentOffset,
                (*FileContext).ShortName.FileName.Buffer,
                core::ptr::addr_of_mut!((*FileContext).ShortName.FileName.Length),
            );

            //
            //  Check if this name matches.
            //

            if CdIsNameInExpression(
                IrpContext,
                Name,
                core::ptr::addr_of_mut!((*FileContext).ShortName) as *mut CD_NAME,
                0,
                0, // FALSE
            ) != 0 {
                //
                //  Let our caller know we found an entry.
                //

                Found = 1; // TRUE
            }

            //
            //  Break out of the loop.
            //

            break;
        }

        //
        //  Continue until there are no more entries.
        //

        if CdLookupNextInitialFileDirent(IrpContext, Fcb, FileContext) == 0 {
            break;
        }
    }

    //
    //  If we find the file then collect all of the dirents.
    //

    if Found != 0 {
        CdLookupLastFileDirent(IrpContext, Fcb, FileContext);
    }

    Found
}

#[no_mangle]
pub unsafe extern "C" fn CdLookupNextInitialFileDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FileContext: *mut FILE_ENUM_CONTEXT,
) -> u8 {
    let mut RawDirent: *mut RAW_DIRENT;

    let mut CurrentDirContext: *mut DIRENT_ENUM_CONTEXT;
    let TargetDirContext: *mut DIRENT_ENUM_CONTEXT;
    let TempDirent: *mut COMPOUND_DIRENT;

    let mut FoundDirent: u8; // FALSE
    let mut FoundLastDirent: bool;

    // PAGED_CODE();

    //
    //  Start by saving the initial dirent of the current file as the
    //  previous file.
    //

    TempDirent = (*FileContext).PriorDirent;
    (*FileContext).PriorDirent = (*FileContext).InitialDirent;
    (*FileContext).InitialDirent = TempDirent;

    //
    //  We will use the initial dirent of the prior file unless the
    //  previous search returned multiple extents.
    //

    CurrentDirContext = core::ptr::addr_of_mut!((*(*FileContext).PriorDirent).DirContext);

    if FlagOnU32((*FileContext).Flags, FILE_CONTEXT_MULTIPLE_DIRENTS) {
        CurrentDirContext = core::ptr::addr_of_mut!((*(*FileContext).CurrentDirent).DirContext);
    }

    //
    //  Clear all of the flags and file size for the next file.
    //

    (*FileContext).Flags = 0;
    (*FileContext).FileSize = 0;

    (*FileContext).ShortName.FileName.Length = 0;

    //
    //  We always want to store the result into the updated initial dirent
    //  context.
    //

    TargetDirContext = core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).DirContext);

    //
    //  Loop until we find the first dirent after the last dirent of the
    //  current file.  We may not be at the last dirent for the current file yet
    //  so we may walk forward looking for the last and then find the
    //  initial dirent for the next file after that.
    //

    loop {
        //
        //  Remember if the last dirent we visited was the last dirent for
        //  a file.
        //

        RawDirent = CdRawDirent(IrpContext, CurrentDirContext);

        FoundLastDirent = !FlagOnU32(CdRawDirentFlags(IrpContext, RawDirent), CD_ATTRIBUTE_MULTI);

        //
        //  Try to find another dirent.
        //

        FoundDirent = CdLookupNextDirent(IrpContext, Fcb, CurrentDirContext, TargetDirContext) as u8;

        //
        //  Exit the loop if no entry found.
        //

        if FoundDirent == 0 {
            break;
        }

        //
        //  Update the in-memory dirent.
        //

        CdUpdateDirentFromRawDirent(
            IrpContext,
            Fcb,
            TargetDirContext,
            core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent),
        );

        //
        //  Exit the loop if we had the end for the previous file.
        //

        if FoundLastDirent {
            break;
        }

        //
        //  Always use a single dirent from this point on.
        //

        CurrentDirContext = TargetDirContext;
    }

    FoundDirent
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     DirSup.c
//
// Abstract:
//
//     This module implements the dirent support routines for Cdfs.
//
//     Directories on a CD consist of a number of contiguous sectors on
//     the disk.  File descriptors consist of one or more directory entries
//     (dirents) within a directory.  Files may contain version numbers.  If
//     present all like-named files will be ordered contiguously in the
//     directory by decreasing version numbers.  We will only return the
//     first of these on a directory query unless the user explicitly
//     asks for version numbers.  Finally dirents will not span sector
//     boundaries.  Unused bytes at the end of a sector will be zero
//     filled.
//
//     Directory sector:                                                   Offset
//                                                                         2048
//         +---------------------------------------------------------------+
//         |            |          |          |           |          |     |
//         | foo;4      | foo;4    | foo;3    |  hat      |  zebra   | Zero|
//         |            |          |          |           |          | Fill|
//         |            |  final   |  single  |           |          |     |
//         |            |  extent  |   extent |           |          |     |
//         +---------------------------------------------------------------+
//
//     Dirent operations:
//
//         - Position scan at known offset in directory.  Dirent at this
//             offset must exist and is valid.  Used when scanning a directory
//             from the beginning when the self entry is known to be valid.
//             Used when positioning at the first dirent for an open
//             file to scan the allocation information.  Used when resuming
//             a directory enumeration from a valid directory entry.
//
//         - Position scan at known offset in directory.  Dirent is known to
//             start at this position but must be checked for validity.
//             Used to read the self-directory entry.
//
//         - Move to the next dirent within a directory.
//
//         - Given a known starting dirent, collect all the dirents for
//             that file.  Scan will finish positioned at the last dirent
//             for the file.  We will accumulate the extent lengths to
//             find the size of the file.
//
//         - Given a known starting dirent, position the scan for the first
//             dirent of the following file.  Used when not interested in
//             all of the details for the current file and are looking for
//             the next file.
//
//         - Update a common dirent structure with the details of the on-disk
//             structure.  This is used to smooth out the differences
//
//         - Build the filename (name and version strings) out of the stream
//             of bytes in the file name on disk.  For Joliet disks we will have
//             to convert to little endian.
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_DIRSUP)
//
// //
// //  Local macros
// //
//
// //
// //  PRAW_DIRENT
// //  CdRawDirent (
// //       PIRP_CONTEXT IrpContext,
// //       PDIR_ENUM_CONTEXT DirContext
// //      );
// //
//
// #define CdRawDirent(IC,DC)                                      \
//     Add2Ptr( (DC)->Sector, (DC)->SectorOffset, PRAW_DIRENT )
//
// //
// //  Local support routines
// //
//
// ULONG
// CdCheckRawDirentBounds (
//      PIRP_CONTEXT IrpContext,
//      PDIRENT_ENUM_CONTEXT DirContext
//     );
//
// XA_EXTENT_TYPE
// CdCheckForXAExtent (
//      PIRP_CONTEXT IrpContext,
//      PRAW_DIRENT RawDirent,
//      PDIRENT Dirent
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCheckForXAExtent)
// #pragma alloc_text(PAGE, CdCheckRawDirentBounds)
// #pragma alloc_text(PAGE, CdCleanupFileContext)
// #pragma alloc_text(PAGE, CdFindFile)
// #pragma alloc_text(PAGE, CdFindDirectory)
// #pragma alloc_text(PAGE, CdFindFileByShortName)
// #pragma alloc_text(PAGE, CdLookupDirent)
// #pragma alloc_text(PAGE, CdLookupLastFileDirent)
// #pragma alloc_text(PAGE, CdLookupNextDirent)
// #pragma alloc_text(PAGE, CdLookupNextInitialFileDirent)
// #pragma alloc_text(PAGE, CdUpdateDirentFromRawDirent)
// #pragma alloc_text(PAGE, CdUpdateDirentName)
// #endif
//
//
// VOID
// CdLookupDirent (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG DirentOffset,
//      PDIRENT_ENUM_CONTEXT DirContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initiate a walk through a directory.  We will
//     position ourselves in the directory at offset DirentOffset.  We know that
//     a dirent begins at this boundary but may have to verify the dirent bounds.
//     We will call this routine when looking up the first entry of a known
//     file or verifying the self entry of a directory.
//
// Arguments:
//
//     Fcb - Fcb for the directory being traversed.
//
//     DirentOffset - This is our target point in the directory.  We will map the
//         page containing this entry and possibly verify the dirent bounds at
//         this location.
//
//     DirContext - This is the dirent context for this scan.  We update it with
//         the location of the dirent we found.  This structure has been initialized
//         outside of this call.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     LONGLONG BaseOffset;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the offset of the first dirent we want to map.
//     //
//
//     DirContext->BaseOffset = SectorTruncate( DirentOffset );
//     BaseOffset = DirContext->BaseOffset;
//
//     DirContext->DataLength = SECTOR_SIZE;
//
//     DirContext->SectorOffset = SectorOffset( DirentOffset );
//
//     //
//     //  Truncate the data length if we are at the end of the file.
//     //
//
//     if (DirContext->DataLength > (Fcb->FileSize.QuadPart - BaseOffset)) {
//
//         DirContext->DataLength = (ULONG) (Fcb->FileSize.QuadPart - BaseOffset);
//     }
//
//     //
//     //  Now map the data at this offset.
//     //
//
//     CcMapData( Fcb->FileObject,
//                (PLARGE_INTEGER) &BaseOffset,
//                DirContext->DataLength,
//                TRUE,
//                &DirContext->Bcb,
//                &DirContext->Sector );
//
//     //
//     //  Verify that the mapping succeeded and we got a valid sector pointer.
//     //
//
//     if ((DirContext->Bcb == NULL) || (DirContext->Sector == NULL)) {
//
//         CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     //
//     //  Verify the dirent bounds.
//     //
//
//     DirContext->NextDirentOffset = CdCheckRawDirentBounds( IrpContext,
//                                                            DirContext );
//
//     return;
// }
//
//
// BOOLEAN
// CdLookupNextDirent (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PDIRENT_ENUM_CONTEXT CurrentDirContext,
//      PDIRENT_ENUM_CONTEXT NextDirContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to find the next dirent in the directory.  The
//     current position is given and we look for the next.  We leave the context
//     for the starting position untouched and update the context for the
//     dirent we found.  The target context may already be initialized so we
//     may already have the sector in memory.
//
//     This routine will position the enumeration context for the next dirent and
//     verify the dirent bounds.
//
//     NOTE - This routine can be called with CurrentDirContext and NextDirContext
//         pointing to the same enumeration context.
//
// Arguments:
//
//     Fcb - Fcb for the directory being traversed.
//
//     CurrentDirContext - This is the dirent context for this scan.  We update
//         it with the location of the dirent we found.  This is currently
//         pointing to a dirent location.  The dirent bounds at this location
//         have already been verified.
//
//     NextDirContext - This is the dirent context to update with the dirent we
//         find.  This may already point to a dirent so we need to check if
//         we are in the same sector and unmap any buffer as necessary.
//
//         This dirent is left in an indeterminant state if we don't find a dirent.
//
// Return Value:
//
//     BOOLEAN - TRUE if we find a location for the next dirent, FALSE otherwise.
//         This routine can cause a raise if the directory is corrupt.
//
// --*/
//
// {
//     LONGLONG CurrentBaseOffset = CurrentDirContext->BaseOffset;
//     ULONG TempUlong;
//
//     BOOLEAN FoundDirent = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Check if a different sector is mapped.  If so then move our target
//     //  enumeration context to the same sector.
//     //
//
//     if ((CurrentDirContext->BaseOffset != NextDirContext->BaseOffset) ||
//         (NextDirContext->Bcb == NULL)) {
//
//         //
//         //  Unpin the current target Bcb and map the next sector.
//         //
//
//         CdUnpinData( IrpContext, &NextDirContext->Bcb );
//
//         CcMapData( Fcb->FileObject,
//                    (PLARGE_INTEGER) &CurrentBaseOffset,
//                    CurrentDirContext->DataLength,
//                    TRUE,
//                    &NextDirContext->Bcb,
//                    &NextDirContext->Sector );
//
//         //
//         //  Verify that the mapping succeeded and we got a valid sector pointer.
//         //
//
//         if ((NextDirContext->Bcb == NULL) || (NextDirContext->Sector == NULL)) {
//
//             CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//         }
//
//         //
//         //  Copy the data length and sector offset.
//         //
//
//         NextDirContext->DataLength = CurrentDirContext->DataLength;
//         NextDirContext->BaseOffset = CurrentDirContext->BaseOffset;
//     }
//
//     //
//     //  Now move to the same offset in the sector.
//     //
//
//     NextDirContext->SectorOffset = CurrentDirContext->SectorOffset;
//
//     //
//     //  If the value is zero then unmap the current sector and set up
//     //  the base offset to the beginning of the next sector.
//     //
//
//     if (CurrentDirContext->NextDirentOffset == 0) {
//
//         CurrentBaseOffset = NextDirContext->BaseOffset + NextDirContext->DataLength;
//
//         //
//         //  Unmap the current sector.  We test the value of the Bcb in the
//         //  loop below to see if we need to read in another sector.
//         //
//
//         CdUnpinData( IrpContext, &NextDirContext->Bcb );
//
//     //
//     //  There is another possible dirent in the current sector.  Update the
//     //  enumeration context to reflect this.
//     //
//
//     } else {
//
//         NextDirContext->SectorOffset += CurrentDirContext->NextDirentOffset;
//     }
//
//     //
//     //  Now loop until we find the next possible dirent or walk off the directory.
//     //
//
//     while (TRUE) {
//
//         //
//         //  If we don't currently have a sector mapped then map the
//         //  directory at the current offset.
//         //
//
//         if (NextDirContext->Bcb == NULL) {
//
//             TempUlong = SECTOR_SIZE;
//
//             if (TempUlong > (ULONG) (Fcb->FileSize.QuadPart - CurrentBaseOffset)) {
//
//                 TempUlong = (ULONG) (Fcb->FileSize.QuadPart - CurrentBaseOffset);
//
//                 //
//                 //  If the length is zero then there is no dirent.
//                 //
//
//                 if (TempUlong == 0) {
//
//                     break;
//                 }
//             }
//
//             CcMapData( Fcb->FileObject,
//                        (PLARGE_INTEGER) &CurrentBaseOffset,
//                        TempUlong,
//                        TRUE,
//                        &NextDirContext->Bcb,
//                        &NextDirContext->Sector );
//
//             //
//             //  Verify that the mapping succeeded and we got a valid sector pointer.
//             //
//
//             if ((NextDirContext->Bcb == NULL) || (NextDirContext->Sector == NULL)) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             NextDirContext->BaseOffset = (ULONG) CurrentBaseOffset;
//             NextDirContext->SectorOffset = 0;
//             NextDirContext->DataLength = TempUlong;
//         }
//
//         //
//         //  The CDFS spec allows for sectors in a directory to contain all zeroes.
//         //  In this case we need to move to the next sector.  So look at the
//         //  current potential dirent for a zero length.  Move to the next
//         //  dirent if length is zero.
//         //
//
//         if (*((PCHAR) CdRawDirent( IrpContext, NextDirContext )) != 0) {
//
//             FoundDirent = TRUE;
//             break;
//         }
//
//         CurrentBaseOffset = NextDirContext->BaseOffset + NextDirContext->DataLength;
//         CdUnpinData( IrpContext, &NextDirContext->Bcb );
//     }
//
//     //
//     //  Check the dirent bounds if we found a dirent.
//     //
//
//     if (FoundDirent) {
//
//         NextDirContext->NextDirentOffset = CdCheckRawDirentBounds( IrpContext,
//                                                                    NextDirContext );
//     }
//
//     return FoundDirent;
// }
//
//
//
// VOID
// CdUpdateDirentFromRawDirent (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PDIRENT_ENUM_CONTEXT DirContext,
//      PDIRENT Dirent
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to safely copy the data from the dirent on disk
//     to the in-memory dirent.  The fields on disk are unaligned so we
//     need to safely copy them to our structure.
//
// Arguments:
//
//     Fcb - Fcb for the directory being scanned.
//
//     DirContext - Enumeration context for the raw disk dirent.
//
//     Dirent - In-memory dirent to update.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PRAW_DIRENT RawDirent = CdRawDirent( IrpContext, DirContext );
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Fcb );
//
//     //
//     //  Clear all of the current state flags except the flag indicating that
//     //  we allocated a name string.
//     //
//
//     ClearFlag( Dirent->Flags, DIRENT_FLAG_NOT_PERSISTENT );
//
//     //
//     //  The dirent offset is the sum of the start of the sector and the
//     //  sector offset.
//     //
//
//     Dirent->DirentOffset = DirContext->BaseOffset + DirContext->SectorOffset;
//
//     //
//     //  Copy the dirent length from the raw dirent.
//     //
//
//     Dirent->DirentLength = RawDirent->DirLen;
//
//     //
//     //  The starting offset on disk is computed by finding the starting
//     //  logical block and stepping over the Xar block.
//     //
//
//     CopyUchar4( &Dirent->StartingOffset, RawDirent->FileLoc );
//
//     Dirent->StartingOffset += RawDirent->XarLen;
//
//     //
//     //  Do a safe copy to get the data length.
//     //
//
//     CopyUchar4( &Dirent->DataLength, RawDirent->DataLen );
//
//     //
//     //  Save a pointer to the time stamps.
//     //
//
//     Dirent->CdTime = (PCHAR)RawDirent->RecordTime;
//
//     //
//     //  Copy the dirent flags.
//     //
//
//     Dirent->DirentFlags = CdRawDirentFlags( IrpContext, RawDirent );
//
//     //
//     //  For both the file unit and interleave skip we want to take the
//     //  logical block count.
//     //
//
//     Dirent->FileUnitSize =
//     Dirent->InterleaveGapSize = 0;
//
//     if (RawDirent->IntLeaveSize != 0) {
//
//         Dirent->FileUnitSize = RawDirent->IntLeaveSize;
//         Dirent->InterleaveGapSize = RawDirent->IntLeaveSkip;
//     }
//
//     //
//     //  Get the name length and remember a pointer to the start of the
//     //  name string.  We don't do any processing on the name at this
//     //  point.
//     //
//     //  Check that the name length is non-zero.
//     //
//
//     if (RawDirent->FileIdLen == 0) {
//
//         CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     Dirent->FileNameLen = RawDirent->FileIdLen;
//     Dirent->FileName = (PCHAR)RawDirent->FileId;
//
//     //
//     //  If there are any remaining bytes at the end of the dirent then
//     //  there may be a system use area.  We protect ourselves from
//     //  disks which don't pad the dirent entries correctly by using
//     //  a fudge factor of one.  All system use areas must have a length
//     //  greater than one.  Don't bother with the system use area
//     //  if this is a directory.
//     //
//
//     Dirent->XAAttributes = 0;
//     Dirent->XAFileNumber = 0;
//     Dirent->ExtentType = Form1Data;
//     Dirent->SystemUseOffset = 0;
//
//     if (!FlagOn( Dirent->DirentFlags, CD_ATTRIBUTE_DIRECTORY ) &&
//         (Dirent->DirentLength > ((FIELD_OFFSET( RAW_DIRENT, FileId ) + Dirent->FileNameLen) + 1))) {
//
//         Dirent->SystemUseOffset = WordAlign( FIELD_OFFSET( RAW_DIRENT, FileId ) + Dirent->FileNameLen );
//     }
//
//     return;
// }
//
//
// VOID
// CdUpdateDirentName (
//      PIRP_CONTEXT IrpContext,
//      PDIRENT Dirent,
//      ULONG IgnoreCase
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to update the name in the dirent with the name
//     from the disk.  We will look for the special case of the self and
//     parent entries and also construct the Unicode name for the Joliet disk
//     in order to work around the BigEndian on-disk structure.
//
// Arguments:
//
//     Dirent - Pointer to the in-memory dirent structure.
//
//     IgnoreCase - TRUE if we should build the upcased version.  Otherwise we
//         use the exact case name.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     UCHAR DirectoryValue;
//     ULONG Length;
//
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     //
//     //  Check if this is a self or parent entry.  There is no version number
//     //  in these cases.  We use a fixed string for these.
//     //
//     //      Self-Entry - Length is 1, value is 0.
//     //      Parent-Entry - Length is 1, value is 1.
//     //
//
//     if ((Dirent->FileNameLen == 1) &&
//         FlagOn( Dirent->DirentFlags, CD_ATTRIBUTE_DIRECTORY )) {
//
//         DirectoryValue = *((PCHAR) Dirent->FileName);
//
//         if ((DirectoryValue == 0) || (DirectoryValue == 1)) {
//
//             //
//             //  We should not have allocated a name by the time we see these cases.
//             //  If we have, this means that the image is in violation of ISO 9660 7.6.2,
//             //  which states that the ./.. entries must be the first two in the directory.
//             //
//
//             if (FlagOn( Dirent->Flags, DIRENT_FLAG_ALLOC_BUFFER )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             //
//             //  Now use one of the hard coded directory names.
//             //
//
//             Dirent->CdFileName.FileName = CdUnicodeDirectoryNames[DirectoryValue];
//
//             //
//             //  Show that there is no version number.
//             //
//
//             Dirent->CdFileName.VersionString.Length = 0;
//
//             //
//             //  The case name is the same as the exact name.
//             //
//
//             Dirent->CdCaseFileName = Dirent->CdFileName;
//
//             //
//             //  Mark this as a constant value entry.
//             //
//
//             SetFlag( Dirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY );
//
//             //
//             //  Return now.
//             //
//
//             return;
//         }
//     }
//
//     //
//     //  Mark this as a non-constant value entry.
//     //
//
//     ClearFlag( Dirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY );
//
//     //
//     //  Compute how large a buffer we will need.  If this is an ignore
//     //  case operation then we will want a double size buffer.  If the disk is not
//     //  a Joliet disk then we might need two bytes for each byte in the name.
//     //
//
//     Length = Dirent->FileNameLen;
//
//     if (IgnoreCase) {
//
//         Length *= 2;
//     }
//
//     if (!FlagOn( IrpContext->Vcb->VcbState, VCB_STATE_JOLIET )) {
//
//         Length *= sizeof( WCHAR );
//     }
//
//     //
//     //  Now decide if we need to allocate a new buffer.  We will if
//     //  this name won't fit in the embedded name buffer and it is
//     //  larger than the current allocated buffer.  We always use the
//     //  allocated buffer if present.
//     //
//     //  If we haven't allocated a buffer then use the embedded buffer if the data
//     //  will fit.  This is the typical case.
//     //
//
//     if (!FlagOn( Dirent->Flags, DIRENT_FLAG_ALLOC_BUFFER ) &&
//         (Length <= sizeof( Dirent->NameBuffer ))) {
//
//         Dirent->CdFileName.FileName.MaximumLength = sizeof( Dirent->NameBuffer );
//         Dirent->CdFileName.FileName.Buffer = Dirent->NameBuffer;
//
//     } else {
//
//         //
//         //  We need to use an allocated buffer.  Check if the current buffer
//         //  is large enough.
//         //
//
//         if (Length > Dirent->CdFileName.FileName.MaximumLength) {
//
//             //
//             //  Free any allocated buffer.
//             //
//
//             if (FlagOn( Dirent->Flags, DIRENT_FLAG_ALLOC_BUFFER )) {
//
//                 CdFreePool( &Dirent->CdFileName.FileName.Buffer );
//                 ClearFlag( Dirent->Flags, DIRENT_FLAG_ALLOC_BUFFER );
//             }
//
//             Dirent->CdFileName.FileName.Buffer = FsRtlAllocatePoolWithTag( CdPagedPool,
//                                                                             Length,
//                                                                             TAG_DIRENT_NAME );
//
//             SetFlag( Dirent->Flags, DIRENT_FLAG_ALLOC_BUFFER );
//
//             Dirent->CdFileName.FileName.MaximumLength = (USHORT) Length;
//         }
//     }
//
//     //
//     //  We now have a buffer for the name.  We need to either convert the on-disk bigendian
//     //  to little endian or covert the name to Unicode.
//     //
//
//     if (!FlagOn( IrpContext->Vcb->VcbState, VCB_STATE_JOLIET )) {
//
//         Status = RtlOemToUnicodeN( Dirent->CdFileName.FileName.Buffer,
//                                    Dirent->CdFileName.FileName.MaximumLength,
//                                    &Length,
//                                    Dirent->FileName,
//                                    Dirent->FileNameLen );
//
//         __analysis_assert( Status == STATUS_SUCCESS );
//         NT_ASSERT( Status == STATUS_SUCCESS );
//         Dirent->CdFileName.FileName.Length = (USHORT) Length;
//
//     } else {
//
//         //
//         //  Convert this string to little endian.
//         //
//
//         CdConvertBigToLittleEndian( IrpContext,
//                                     Dirent->FileName,
//                                     Dirent->FileNameLen,
//                                     (PCHAR) Dirent->CdFileName.FileName.Buffer );
//
//         Dirent->CdFileName.FileName.Length = (USHORT) Dirent->FileNameLen;
//     }
//
//     //
//     //  Split the name into name and version strings.
//     //
//
//     CdConvertNameToCdName( IrpContext,
//                            &Dirent->CdFileName );
//
//     //
//     //  The name length better be non-zero.
//     //
//
//     if (Dirent->CdFileName.FileName.Length == 0) {
//
//         CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     //
//     //  If the filename ends with a period then back up one character.
//     //
//
//     if (Dirent->CdFileName.FileName.Buffer[(Dirent->CdFileName.FileName.Length - sizeof( WCHAR )) / 2] == L'.') {
//
//         //
//         //  Slide the version string down.
//         //
//
//         if (Dirent->CdFileName.VersionString.Length != 0) {
//
//             PWCHAR NewVersion;
//
//             //
//             //  Start from the position currently containing the separator.
//             //
//
//             NewVersion = Add2Ptr( Dirent->CdFileName.FileName.Buffer,
//                                   Dirent->CdFileName.FileName.Length,
//                                   PWCHAR );
//
//             //
//             //  Now overwrite the period.
//             //
//
//             RtlMoveMemory( NewVersion - 1,
//                            NewVersion,
//                            Dirent->CdFileName.VersionString.Length + sizeof( WCHAR ));
//
//             //
//             //  Now point to the new version string.
//             //
//
//             Dirent->CdFileName.VersionString.Buffer = NewVersion;
//         }
//
//         //
//         //  Shrink the filename length.
//         //
//
//         Dirent->CdFileName.FileName.Length -= sizeof( WCHAR );
//     }
//
//     if (!CdIsLegalName( IrpContext, &Dirent->CdFileName.FileName )) {
//
//         CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     //
//     //  If this an exact case operation then use the filename exactly.
//     //
//
//     if (!IgnoreCase) {
//
//         Dirent->CdCaseFileName = Dirent->CdFileName;
//
//     //
//     //  Otherwise perform our upcase operation.  We already have guaranteed the buffers are
//     //  there.
//     //
//
//     } else {
//
//         Dirent->CdCaseFileName.FileName.Buffer = Add2Ptr( Dirent->CdFileName.FileName.Buffer,
//                                                           Dirent->CdFileName.FileName.MaximumLength / 2,
//                                                           PWCHAR);
//
//         Dirent->CdCaseFileName.FileName.MaximumLength = Dirent->CdFileName.FileName.MaximumLength / 2;
//
//         CdUpcaseName( IrpContext,
//                       &Dirent->CdFileName,
//                       &Dirent->CdCaseFileName );
//     }
//
//     return;
// }
//
//
//  BOOLEAN
// CdFindFile (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCD_NAME Name,
//      BOOLEAN IgnoreCase,
//      PFILE_ENUM_CONTEXT FileContext,
//      PCD_NAME *MatchingName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to search a dirctory for a file matching the input
//     name.  This name has been upcased at this point if this a case-insensitive
//     search.  The name has been separated into separate name and version strings.
//     We look for an exact match in the name and only consider the version if
//     there is a version specified in the search name.
//
// Arguments:
//
//     Fcb - Fcb for the directory being scanned.
//
//     Name - Name to search for.
//
//     IgnoreCase - Indicates the case of the search.
//
//     FileContext - File context to use for the search.  This has already been
//         initialized.
//
//     MatchingName - Pointer to buffer containing matching name.  We need this
//         in case we don't match the name in the directory but match the
//         short name instead.
//
// Return Value:
//
//     BOOLEAN - TRUE if matching entry is found, FALSE otherwise.
//
// --*/
//
// {
//     PDIRENT Dirent;
//     ULONG ShortNameDirentOffset;
//
//     BOOLEAN Found = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Make sure there is a stream file for this Fcb.
//     //
//
//     CdVerifyOrCreateDirStreamFile( IrpContext, Fcb);
//
//     //
//     //  Check to see whether we need to check for a possible short name.
//     //
//
//     ShortNameDirentOffset = CdShortNameDirentOffset( IrpContext, &Name->FileName );
//
//     //
//     //  Position ourselves at the first entry.
//     //
//
//     CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, Fcb->StreamOffset );
//
//     //
//     //  Loop while there are more entries in this directory.
//     //
//
//     do {
//
//         Dirent = &FileContext->InitialDirent->Dirent;
//
//         //
//         //  We only consider files which don't have the associated bit set.
//         //  We also only look for files.  All directories would already
//         //  have been found.
//         //
//
//         if (!FlagOn( Dirent->DirentFlags, CD_ATTRIBUTE_ASSOC | CD_ATTRIBUTE_DIRECTORY )) {
//
//             //
//             //  Update the name in the current dirent.
//             //
//
//             CdUpdateDirentName( IrpContext, Dirent, IgnoreCase );
//
//             //
//             //  Don't bother with constant entries.
//             //
//
//             if (FlagOn( Dirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY )) {
//
//                 continue;
//             }
//
//             //
//             //  Now check whether we have a name match.
//             //  We exit the loop if we have a match.
//             //
//
//             if (CdIsNameInExpression( IrpContext,
//                                       &Dirent->CdCaseFileName,
//                                       Name,
//                                       0,
//                                       TRUE )) {
//
//                 *MatchingName = &Dirent->CdCaseFileName;
//                 Found = TRUE;
//                 break;
//             }
//
//             //
//             //  The names didn't match.  If the input name is a possible short
//             //  name and we are at the correct offset in the directory then
//             //  check if the short names match.
//             //
//
//             if (((Dirent->DirentOffset >> SHORT_NAME_SHIFT) == ShortNameDirentOffset) &&
//                 (Name->VersionString.Length == 0) &&
//                 !CdIs8dot3Name( IrpContext,
//                                 Dirent->CdFileName.FileName )) {
//
//                 //
//                 //  Create the short name and check for a match.
//                 //
//
//                 CdGenerate8dot3Name( IrpContext,
//                                      &Dirent->CdCaseFileName.FileName,
//                                      Dirent->DirentOffset,
//                                      FileContext->ShortName.FileName.Buffer,
//                                      &FileContext->ShortName.FileName.Length );
//
//                 //
//                 //  Now check whether we have a name match.
//                 //  We exit the loop if we have a match.
//                 //
//
//                 if (CdIsNameInExpression( IrpContext,
//                                           &FileContext->ShortName,
//                                           Name,
//                                           0,
//                                           FALSE )) {
//
//                     *MatchingName = &FileContext->ShortName,
//                     Found = TRUE;
//                     break;
//                 }
//             }
//         }
//
//         //
//         //  Go to the next initial dirent for a file.
//         //
//
//     } while (CdLookupNextInitialFileDirent( IrpContext, Fcb, FileContext ));
//
//     //
//     //  If we find the file then collect all of the dirents.
//     //
//
//     if (Found) {
//
//         CdLookupLastFileDirent( IrpContext, Fcb, FileContext );
//
//     }
//
//     return Found;
// }
//
//
// BOOLEAN
// CdFindDirectory (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCD_NAME Name,
//      BOOLEAN IgnoreCase,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to search a dirctory for a directory matching the input
//     name.  This name has been upcased at this point if this a case-insensitive
//     search.  We look for an exact match in the name and do not look for shortname
//     equivalents.
//
// Arguments:
//
//     Fcb - Fcb for the directory being scanned.
//
//     Name - Name to search for.
//
//     IgnoreCase - Indicates the case of the search.
//
//     FileContext - File context to use for the search.  This has already been
//         initialized.
//
// Return Value:
//
//     BOOLEAN - TRUE if matching entry is found, FALSE otherwise.
//
// --*/
//
// {
//     PDIRENT Dirent;
//
//     BOOLEAN Found = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Make sure there is a stream file for this Fcb.
//     //
//
//     CdVerifyOrCreateDirStreamFile( IrpContext, Fcb);
//
//     //
//     //  Position ourselves at the first entry.
//     //
//
//     CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, Fcb->StreamOffset );
//
//     //
//     //  Loop while there are more entries in this directory.
//     //
//
//     do {
//
//         Dirent = &FileContext->InitialDirent->Dirent;
//
//         //
//         //  We only look for directories.  Directories cannot have the
//         //  associated bit set.
//         //
//
//         if (FlagOn( Dirent->DirentFlags, CD_ATTRIBUTE_DIRECTORY )) {
//
//             //
//             //  Update the name in the current dirent.
//             //
//
//             CdUpdateDirentName( IrpContext, Dirent, IgnoreCase );
//
//             //
//             //  Don't bother with constant entries.
//             //
//
//             if (FlagOn( Dirent->Flags, DIRENT_FLAG_CONSTANT_ENTRY )) {
//
//                 continue;
//             }
//
//             //
//             //  Now check whether we have a name match.
//             //  We exit the loop if we have a match.
//             //
//
//             if (CdIsNameInExpression( IrpContext,
//                                       &Dirent->CdCaseFileName,
//                                       Name,
//                                       0,
//                                       TRUE )) {
//
//                 Found = TRUE;
//                 break;
//             }
//         }
//
//         //
//         //  Go to the next initial dirent.
//         //
//
//     } while (CdLookupNextInitialFileDirent( IrpContext, Fcb, FileContext ));
//
//     return Found;
// }
//
//
// (FileContext->ShortName.FileName.MaximumLength, )
// BOOLEAN
// CdFindFileByShortName (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PCD_NAME Name,
//      BOOLEAN IgnoreCase,
//      ULONG ShortNameDirentOffset,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to find the file name entry whose short name
//     is defined by the input DirentOffset.  The dirent offset here is
//     multiplied by 32 and we look for the dirent begins in this 32 byte offset in
//     directory.  The minimum dirent length is 34 so we are guaranteed that only
//     one dirent can begin in each 32 byte block in the directory.
//
// Arguments:
//
//     Fcb - Fcb for the directory being scanned.
//
//     Name - Name we are trying to match.  We know this contains the tilde
//         character followed by decimal characters.
//
//     IgnoreCase - Indicates whether we need to upcase the long name and
//         generated short name.
//
//     ShortNameDirentOffset - This is the shifted value for the offset of the
//         name in the directory.
//
//     FileContext - This is the initialized file context to use for the search.
//
// Return Value:
//
//     BOOLEAN - TRUE if a matching name was found, FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN Found = FALSE;
//     PDIRENT Dirent;
//
//     ULONG ThisShortNameDirentOffset;
//
//     PAGED_CODE();
//
//     //
//     //  Make sure there is a stream file for this Fcb.
//     //
//
//     CdVerifyOrCreateDirStreamFile( IrpContext, Fcb);
//
//     //
//     //  Position ourselves at the start of the directory and update
//     //
//     //
//
//     CdLookupInitialFileDirent( IrpContext, Fcb, FileContext, Fcb->StreamOffset );
//
//     //
//     //  Loop until we have found the entry or are beyond this dirent.
//     //
//
//     do {
//
//         //
//         //  Compute the short name dirent offset for the current dirent.
//         //
//
//         Dirent = &FileContext->InitialDirent->Dirent;
//         ThisShortNameDirentOffset = Dirent->DirentOffset >> SHORT_NAME_SHIFT;
//
//         //
//         //  If beyond the target then exit.
//         //
//
//         if (ThisShortNameDirentOffset > ShortNameDirentOffset) {
//
//             break;
//         }
//
//         //
//         //  If equal to the target then check if we have a name match.
//         //  We will either match or fail here.
//         //
//
//         if (ThisShortNameDirentOffset == ShortNameDirentOffset) {
//
//             //
//             //  If this is an associated file then get out.
//             //
//
//             if (FlagOn( Dirent->DirentFlags, CD_ATTRIBUTE_ASSOC )) {
//
//                 break;
//             }
//
//             //
//             //  Update the name in the dirent and check if it is not
//             //  an 8.3 name.
//             //
//
//             CdUpdateDirentName( IrpContext, Dirent, IgnoreCase );
//
//             if (CdIs8dot3Name( IrpContext,
//                                Dirent->CdFileName.FileName )) {
//
//                 break;
//             }
//
//             //
//             //  Generate the 8.3 name see if it matches our input name.
//             //
//
//             CdGenerate8dot3Name( IrpContext,
//                                  &Dirent->CdCaseFileName.FileName,
//                                  Dirent->DirentOffset,
//                                  FileContext->ShortName.FileName.Buffer,
//                                  &FileContext->ShortName.FileName.Length );
//
//             //
//             //  Check if this name matches.
//             //
//
//             if (CdIsNameInExpression( IrpContext,
//                                       Name,
//                                       &FileContext->ShortName,
//                                       0,
//                                       FALSE )) {
//
//                 //
//                 //  Let our caller know we found an entry.
//                 //
//
//                 Found = TRUE;
//             }
//
//             //
//             //  Break out of the loop.
//             //
//
//             break;
//         }
//
//         //
//         //  Continue until there are no more entries.
//         //
//
//     } while (CdLookupNextInitialFileDirent( IrpContext, Fcb, FileContext ));
//
//     //
//     //  If we find the file then collect all of the dirents.
//     //
//
//     if (Found) {
//
//         CdLookupLastFileDirent( IrpContext, Fcb, FileContext );
//
//     }
//
//     return Found;
// }
//
//
// BOOLEAN
// CdLookupNextInitialFileDirent (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to walk through the directory until we find the
//     first possible dirent for file.  We are positioned at some point described
//     by the FileContext.  We will walk through any remaing dirents for the
//     current file until we find the first dirent for some subsequent file.
//
//     We can be called when we have found just one dirent for a file or all
//     of them.  We first check the CurrentDirContext.  In the typical
//     single-extent case this is unused.  Then we look to the InitialDirContext
//     which must be initialized.
//
//     This routine will save the initial DirContext to the PriorDirContext and
//     clean up any existing DirContext for the Prior or Current positions in
//     the enumeration context.
//
// Arguments:
//
//     Fcb - This is the directory to scan.
//
//     FileContext - This is the file enumeration context.  It is currently pointing
//         at some file in the directory.
//
// Return Value:
//
// --*/
//
// {
//     PRAW_DIRENT RawDirent;
//
//     PDIRENT_ENUM_CONTEXT CurrentDirContext;
//     PDIRENT_ENUM_CONTEXT TargetDirContext;
//     PCOMPOUND_DIRENT TempDirent;
//
//     BOOLEAN FoundDirent = FALSE;
//     BOOLEAN FoundLastDirent;
//
//     PAGED_CODE();
//
//     //
//     //  Start by saving the initial dirent of the current file as the
//     //  previous file.
//     //
//
//     TempDirent = FileContext->PriorDirent;
//     FileContext->PriorDirent = FileContext->InitialDirent;
//     FileContext->InitialDirent = TempDirent;
//
//     //
//     //  We will use the initial dirent of the prior file unless the
//     //  previous search returned multiple extents.
//     //
//
//     CurrentDirContext = &FileContext->PriorDirent->DirContext;
//
//     if (FlagOn( FileContext->Flags, FILE_CONTEXT_MULTIPLE_DIRENTS )) {
//
//         CurrentDirContext = &FileContext->CurrentDirent->DirContext;
//     }
//
//     //
//     //  Clear all of the flags and file size for the next file.
//     //
//
//     FileContext->Flags = 0;
//     FileContext->FileSize = 0;
//
//     FileContext->ShortName.FileName.Length = 0;
//
//     //
//     //  We always want to store the result into the updated initial dirent
//     //  context.
//     //
//
//     TargetDirContext = &FileContext->InitialDirent->DirContext;
//
//     //
//     //  Loop until we find the first dirent after the last dirent of the
//     //  current file.  We may not be at the last dirent for the current file yet
//     //  so we may walk forward looking for the last and then find the
//     //  initial dirent for the next file after that.
//     //
//
//     while (TRUE) {
//
//         //
//         //  Remember if the last dirent we visited was the last dirent for
//         //  a file.
//         //
//
//         RawDirent = CdRawDirent( IrpContext, CurrentDirContext );
//
//         FoundLastDirent = !FlagOn( CdRawDirentFlags( IrpContext, RawDirent ), CD_ATTRIBUTE_MULTI );
//
//         //
//         //  Try to find another dirent.
//         //
//
//         FoundDirent = CdLookupNextDirent( IrpContext,
//                                           Fcb,
//                                           CurrentDirContext,
//                                           TargetDirContext );
//
//         //
//         //  Exit the loop if no entry found.
//         //
//
//         if (!FoundDirent) {
//
//             break;
//
//         }
//
//         //
//         //  Update the in-memory dirent.
//         //
//
//         CdUpdateDirentFromRawDirent( IrpContext,
//                                      Fcb,
//                                      TargetDirContext,
//                                      &FileContext->InitialDirent->Dirent );
//
//         //
//         //  Exit the loop if we had the end for the previous file.
//         //
//
//         if (FoundLastDirent) {
//
//             break;
//         }
//
//         //
//         //  Always use a single dirent from this point on.
//         //
//
//         CurrentDirContext = TargetDirContext;
//     }
//
//     return FoundDirent;
// }
//
//
// VOID
// CdLookupLastFileDirent (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when we've found the matching initial dirent for
//     a file.  Now we want to find all of the dirents for a file as well as
//     compute the running total for the file size.
//
//     We also go out to the system use area and check whether this is an
//     XA sector.  In that case we will compute the real file size.
//
//     The dirent in the initial compound dirent has been updated from the
//     raw dirent when this routine is called.
//
// Arguments:
//
//     Fcb - Directory containing the entries for the file.
//
//     FileContext - Enumeration context for this search.  It currently points
//         to the first dirent of the file and the in-memory dirent has been
//         updated.
//
// Return Value:
//
//     None.  This routine may raise STATUS_FILE_CORRUPT.
//
// --*/
//
// {
//     XA_EXTENT_TYPE ExtentType = Form1Data;
//     PCOMPOUND_DIRENT CurrentCompoundDirent;
//     PDIRENT CurrentDirent = NULL;
//
//     BOOLEAN FirstPass = TRUE;
//     BOOLEAN FoundDirent;
//
//     PAGED_CODE();
//
//     //
//     //  The current dirent to look at is the initial dirent for the file.
//     //
//
//     CurrentCompoundDirent = FileContext->InitialDirent;
//
//     //
//     //  Loop until we reach the last dirent for the file.
//     //
//
//     while (TRUE) {
//
//         CurrentDirent = &CurrentCompoundDirent->Dirent;
//
//         //
//         //  Check if this extent has XA sectors.
//         //
//
//         if ((CurrentDirent->SystemUseOffset != 0) &&
//             FlagOn( Fcb->Vcb->VcbState, VCB_STATE_CDXA ) &&
//             CdCheckForXAExtent( IrpContext,
//                                 CdRawDirent( IrpContext, &CurrentCompoundDirent->DirContext ),
//                                 CurrentDirent )) {
//
//             //
//             //  Any previous dirent must describe XA sectors as well.
//             //
//
//             if (!FirstPass && (ExtentType != CurrentDirent->ExtentType)) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             //
//             //  If there are XA sectors then the data on the disk must
//             //  be correctly aligned on sectors and be an integral number of
//             //  sectors.  Only an issue if the logical block size is not
//             //  2048.
//             //
//
//             if (Fcb->Vcb->BlockSize != SECTOR_SIZE) {
//
//                 //
//                 //  We will do the following checks.
//                 //
//                 //      Data must start on a sector boundary.
//                 //      Data length must be integral number of sectors.
//                 //
//
//                 if ((SectorBlockOffset( Fcb->Vcb, CurrentDirent->StartingOffset ) != 0) ||
//                     (SectorBlockOffset( Fcb->Vcb, CurrentDirent->DataLength ) != 0)) {
//
//                     CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                 }
//
//                 //
//                 //  If interleaved then both the file unit and interleave
//                 //  gap must be integral number of sectors.
//                 //
//
//                 if ((CurrentDirent->FileUnitSize != 0) &&
//                     ((SectorBlockOffset( Fcb->Vcb, CurrentDirent->FileUnitSize ) != 0) ||
//                      (SectorBlockOffset( Fcb->Vcb, CurrentDirent->InterleaveGapSize ) != 0))) {
//
//                     CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                 }
//             }
//
//             //
//             //  If this is the first dirent then add the bytes for the RIFF
//             //  header.
//             //
//
//             if (FirstPass) {
//
//                 FileContext->FileSize = sizeof( RIFF_HEADER );
//             }
//
//             //
//             //  Add the size of the mode2-form2 sector for each sector
//             //  we have here.
//             //
//
//             FileContext->FileSize += Int32x32To64( CurrentDirent->DataLength >> SECTOR_SHIFT,
//                                                    XA_SECTOR_SIZE);
//
//         } else {
//
//             //
//             //  This extent does not have XA sectors.  Any previous dirent
//             //  better not have XA sectors.
//             //
//
//             if (!FirstPass && (ExtentType != CurrentDirent->ExtentType)) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             //
//             //  Add these bytes to the file size.
//             //
//
//             FileContext->FileSize += CurrentDirent->DataLength;
//         }
//
//         //
//         //  If we are at the last dirent then exit.
//         //
//
//         if (!FlagOn( CurrentDirent->DirentFlags, CD_ATTRIBUTE_MULTI )) {
//
//             break;
//         }
//
//         //
//         //  Remember the extent type of the current extent.
//         //
//
//         ExtentType = CurrentDirent->ExtentType;
//
//         //
//         //  Look for the next dirent of the file.
//         //
//
//         FoundDirent = CdLookupNextDirent( IrpContext,
//                                           Fcb,
//                                           &CurrentCompoundDirent->DirContext,
//                                           &FileContext->CurrentDirent->DirContext );
//
//         //
//         //  If we didn't find the entry then this is a corrupt directory.
//         //
//
//         if (!FoundDirent) {
//
//             CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//         }
//
//         //
//         //  Remember the dirent we just found.
//         //
//
//         CurrentCompoundDirent = FileContext->CurrentDirent;
//         FirstPass = FALSE;
//
//         //
//         //  Look up all of the dirent information for the given dirent.
//         //
//
//         CdUpdateDirentFromRawDirent( IrpContext,
//                                      Fcb,
//                                      &CurrentCompoundDirent->DirContext,
//                                      &CurrentCompoundDirent->Dirent );
//
//         //
//         //  Set flag to show there were multiple extents.
//         //
//
//         SetFlag( FileContext->Flags, FILE_CONTEXT_MULTIPLE_DIRENTS );
//     }
//
//     return;
// }
//
//
// VOID
// CdCleanupFileContext (
//      PIRP_CONTEXT IrpContext,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to cleanup the enumeration context for a file
//     search in a directory.  We will unpin any remaining Bcbs and free
//     any allocated buffers.
//
// Arguments:
//
//     FileContext - Enumeration context for the file search.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PCOMPOUND_DIRENT CurrentCompoundDirent;
//     ULONG Count = 2;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Cleanup the individual compound dirents.
//     //
//
//     do {
//
//         CurrentCompoundDirent = &FileContext->Dirents[ Count ];
//         CdCleanupDirContext( IrpContext, &CurrentCompoundDirent->DirContext );
//         CdCleanupDirent( IrpContext, &CurrentCompoundDirent->Dirent );
//
//     } while (Count--);
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// ULONG
// CdCheckRawDirentBounds (
//      PIRP_CONTEXT IrpContext,
//      PDIRENT_ENUM_CONTEXT DirContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes a Dirent enumeration context and computes the offset
//     to the next dirent.  A non-zero value indicates the offset within this
//     sector.  A zero value indicates to move to the next sector.  If the
//     current dirent does not fit within the sector then we will raise
//     STATUS_CORRUPT.
//
// Arguments:
//
//     DirContext - Enumeration context indicating the current position in
//         the sector.
//
// Return Value:
//
//     ULONG - Offset to the next dirent in this sector or zero if the
//         next dirent is in the next sector.
//
//     This routine will raise on a dirent which does not fit into the
//     described data buffer.
//
// --*/
//
// {
//     ULONG NextDirentOffset;
//     PRAW_DIRENT RawDirent;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  We should always have at least a byte still available in the
//     //  current buffer.
//     //
//
//     NT_ASSERT( (DirContext->DataLength - DirContext->SectorOffset) >= 1 );
//
//     //
//     //  Validate sector bounds before dereferencing.
//     //  SectorOffset must be within the mapped data length.
//     //
//
//     if ((DirContext->SectorOffset >= DirContext->DataLength) ||
//         (DirContext->Sector == NULL)) {
//
//         CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     //
//     //  Get a pointer to the current dirent.
//     //
//
//     RawDirent = CdRawDirent( IrpContext, DirContext );
//
//     //
//     //  If the dirent length is non-zero then look at the current dirent.
//     //
//
//     if (RawDirent->DirLen != 0) {
//
//         //
//         //  Check the following bound for the dirent length.
//         //
//         //      - Fits in the available bytes in the sector.
//         //      - Is at least the minimal dirent size.
//         //      - Is large enough to hold the file name.
//         //
//
//         if ((RawDirent->DirLen > (DirContext->DataLength - DirContext->SectorOffset)) ||
//             (RawDirent->DirLen < MIN_RAW_DIRENT_LEN) ||
//             (RawDirent->DirLen < (MIN_RAW_DIRENT_LEN - 1 + RawDirent->FileIdLen))) {
//
//             CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//         }
//
//         //
//         //  Copy the dirent length field.
//         //
//
//         NextDirentOffset = RawDirent->DirLen;
//
//         //
//         //  If we are exactly at the next sector then tell our caller by
//         //  returning zero.
//         //
//
//         if (NextDirentOffset == (DirContext->DataLength - DirContext->SectorOffset)) {
//
//             NextDirentOffset = 0;
//         }
//
//     } else {
//
//         NextDirentOffset = 0;
//     }
//
//     return NextDirentOffset;
// }
//
//
// //
// //  Local support routine
// //
//
// XA_EXTENT_TYPE
// CdCheckForXAExtent (
//      PIRP_CONTEXT IrpContext,
//      PRAW_DIRENT RawDirent,
//      PDIRENT Dirent
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to scan through the system use area to test if
//     the current dirent has the XA bit set.  The bit in the in-memory
//     dirent will be set as appropriate.
//
// Arguments:
//
//     RawDirent - Pointer to the on-disk dirent.
//
//     Dirent - Pointer to the in-memory dirent.  We will update this with the
//         appropriate XA flag.
//
// Return Value:
//
//     XA_EXTENT_TYPE - Type of physical extent for this on disk dirent.
//
// --*/
//
// {
//     XA_EXTENT_TYPE ExtentType = Form1Data;
//     PSYSTEM_USE_XA SystemUseArea;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check if there is enough space for the XA system use area.
//     //
//
//     if (Dirent->DirentLength - Dirent->SystemUseOffset >= sizeof( SYSTEM_USE_XA )) {
//
//         SystemUseArea = Add2Ptr( RawDirent, Dirent->SystemUseOffset, PSYSTEM_USE_XA );
//
//         //
//         //  Check for a valid signature.
//         //
//
//         if (SystemUseArea->Signature == SYSTEM_XA_SIGNATURE) {
//
//             //
//             //  Check for an audio track.
//             //
//
//             if (FlagOn( SystemUseArea->Attributes, SYSTEM_USE_XA_DA )) {
//
//                 ExtentType = CDAudio;
//
//             } else if (FlagOn( SystemUseArea->Attributes, SYSTEM_USE_XA_FORM2 )) {
//
//                 //
//                 //  Check for XA data.  Note that a number of discs (video CDs)
//                 //  have files marked as type XA Mode 2 Form 1 (2048 bytes of
//                 //  user data),  but actually record these sectors as Mode2 Form 2
//                 //  (2352). We will fail to read these files,  since for M2F1,
//                 //  a normal read CD command is issued (as per SCSI specs).
//                 //
//
//                 ExtentType = Mode2Form2Data;
//             }
//
//             Dirent->XAAttributes = SystemUseArea->Attributes;
//             Dirent->XAFileNumber = SystemUseArea->FileNumber;
//         }
//     }
//
//     Dirent->ExtentType = ExtentType;
//     return ExtentType;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
