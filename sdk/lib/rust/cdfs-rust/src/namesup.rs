use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{GENERATE_NAME_CONTEXT, OEM_STRING, PUNICODE_STRING, RtlCompareMemory, RtlFreeOemString, RtlGenerate8dot3Name, RtlUnicodeStringToCountedOemString, RtlUnicodeStringToOemString, RtlUpcaseUnicodeString, STRING};
use ntkrnl_sys::ddk::{FSRTL_COMPARISON_RESULT, _FSRTL_COMPARISON_RESULT_EqualTo as EqualTo, _FSRTL_COMPARISON_RESULT_GreaterThan as GreaterThan, _FSRTL_COMPARISON_RESULT_LessThan as LessThan};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CD_NAME, IRP_CONTEXT, PIRP_CONTEXT};
use ntkrnl_sys::ddk::{_UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlIsNameInExpression};
use crate::glue::{CDFS_BUG_CHECK_NAMESUP, STATUS_DISK_CORRUPT_ERROR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlIsFatDbcsLegal};

extern "C" {
    fn CdRaiseStatus(IrpContext: PIRP_CONTEXT, Status: u32);
}
extern "C" {
    fn FlagOn(Value: u32, Flags: u32) -> i32;
}
extern "C" {
    fn NT_ASSERT(condition: i32);
}
extern "C" {
    fn FsRtlIsAnsiCharacterLegalHpfs(c: u16, strict: i32) -> i32;
}
extern "C" {
    fn FsRtlIsLeadDbcsCharacter(Char: u8) -> u8;
}
extern "C" {
    fn RtlEqualMemory(Buffer1: *const c_void, Buffer2: *const c_void, Length: usize) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_NAMESUP;

const BYTE_COUNT_8_DOT_3: usize = 24;
const SHORT_NAME_SHIFT: u32 = 5;

const CCB_FLAG_ENUM_NAME_EXP_HAS_WILD: ULONG = 0x00010000;
const CCB_FLAG_ENUM_VERSION_MATCH_ALL: ULONG = 0x00080000;
const CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD: ULONG = 0x00020000;
#[no_mangle]
pub unsafe extern "C" fn CdConvertNameToCdName(
    IrpContext: *mut IRP_CONTEXT,
    CdName: *mut CD_NAME,
) {
    let mut NameLength: u32 = 0u32;
    let mut CurrentCharacter: *mut u16 = (*CdName).FileName.Buffer;

    PAGED_CODE!();

    // UNREFERENCED_PARAMETER( IrpContext );
    let _ = IrpContext;

    // Look for a separator character.
    while (NameLength < (*CdName).FileName.Length as u32)
        && (*CurrentCharacter != 0x3B_u16)
    {
        CurrentCharacter = CurrentCharacter.add(1);
        NameLength = NameLength.wrapping_add(2u32);
    }

    // If there is at least one more character after a possible separator then it
    // and all following characters are part of the version string.
    (*CdName).VersionString.Length = 0u16;
    if (NameLength + core::mem::size_of::<u16>() as u32) < (*CdName).FileName.Length as u32 {
        (*CdName).VersionString.MaximumLength = (((*CdName).FileName.Length as u32)
            .wrapping_sub(NameLength)
            .wrapping_sub(core::mem::size_of::<u16>() as u32)) as u16;
        (*CdName).VersionString.Length = (*CdName).VersionString.MaximumLength;

        // Add2Ptr( CdName->FileName.Buffer, NameLength + sizeof( WCHAR ), PWCHAR );
        let offset_bytes: u32 = NameLength + core::mem::size_of::<u16>() as u32;
        (*CdName).VersionString.Buffer = ((*CdName).FileName.Buffer as *mut u8)
            .add(offset_bytes as usize) as *mut u16;
    }

    // Now update the filename portion of the name.
    (*CdName).FileName.Length = NameLength as u16;
}
#[no_mangle]
pub unsafe extern "C" fn CdConvertBigToLittleEndian(
    IrpContext: PIRP_CONTEXT,
    BigEndian: *mut u8,
    ByteCount: u32,
    LittleEndian: *mut u8,
) {
    let mut remaining_byte_count: u32 = ByteCount;
    let mut source: *const u8 = BigEndian;
    let mut destination: *mut u8 = LittleEndian;

    PAGED_CODE!();

    if FlagOn(ByteCount, 1) != 0 {
        CdRaiseStatus(IrpContext, STATUS_DISK_CORRUPT_ERROR as u32);
    }

    core::ptr::copy_nonoverlapping(
        source.add(1),
        destination,
        (remaining_byte_count - 1) as usize,
    );

    destination = destination.add(1);

    while remaining_byte_count != 0 {
        *destination = *source;

        source = source.add(2);
        destination = destination.add(2);

        remaining_byte_count -= 2;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdUpcaseName(
    IrpContext: *mut core::ffi::c_void,
    Name:     *mut CD_NAME,
    UpcaseName: *mut CD_NAME,
) {
    // PAGED_CODE();
    let _ = IrpContext; // UNREFERENCED_PARAMETER

    let mut Status: i32;

    if Name != UpcaseName {
        (*UpcaseName).VersionString.Length = 0;

        if (*Name).VersionString.Length != 0 {
            (*UpcaseName).VersionString.MaximumLength = (*Name).VersionString.Length;
            (*UpcaseName).VersionString.Length = (*Name).VersionString.Length;

            // Add2Ptr(UpcaseName->FileName.Buffer, Name->FileName.Length, PWCHAR)
            let buffer = ((*UpcaseName).FileName.Buffer as *mut u8)
                .add((*Name).FileName.Length as usize) as *mut u16;
            (*UpcaseName).VersionString.Buffer = buffer;

            // Store separator and move to next character
            *((*UpcaseName).VersionString.Buffer) = ';' as u16;
            (*UpcaseName).VersionString.Buffer = ((*UpcaseName).VersionString.Buffer).add(1);
        }
    }

    Status = RtlUpcaseUnicodeString(
        core::ptr::addr_of_mut!((*UpcaseName).FileName),
        core::ptr::addr_of!((*Name).FileName),
        0, // FALSE → 0
    );

    NT_ASSERT((Status == STATUS_SUCCESS) as i32);

    // __analysis_assert( Status == STATUS_SUCCESS );

    if (*Name).VersionString.Length != 0 {
        Status = RtlUpcaseUnicodeString(
            core::ptr::addr_of_mut!((*UpcaseName).VersionString),
            core::ptr::addr_of!((*Name).VersionString),
            0,
        );

        NT_ASSERT((Status == STATUS_SUCCESS) as i32);
        // __analysis_assert( Status == STATUS_SUCCESS );
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdDissectName(
    IrpContext: *mut IRP_CONTEXT,
    RemainingName: *mut _UNICODE_STRING,
    FinalName: *mut _UNICODE_STRING,
) {
    // PAGED_CODE();  (stubbed out for translation)
    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    let mut name_length: u32 = 0;
    let mut next_wchar: *mut u16 = (*RemainingName).Buffer;

    while (name_length < (*RemainingName).Length as u32) && (*next_wchar != 0x5C) {
        name_length += 2; // sizeof(WCHAR)
        next_wchar = next_wchar.add(1);
    }

    (*FinalName).Buffer = (*RemainingName).Buffer;
    (*FinalName).MaximumLength = name_length as u16;
    (*FinalName).Length = name_length as u16;

    if name_length == (*RemainingName).Length as u32 {
        (*RemainingName).Length = 0;
    } else {
        let bytes_to_subtract = (name_length + 2) as u16;
        (*RemainingName).MaximumLength -= bytes_to_subtract;
        (*RemainingName).Length -= bytes_to_subtract;
        // Add2Ptr(RemainingName->Buffer, name_length + sizeof(WCHAR), PWCHAR)
        (*RemainingName).Buffer = ((*RemainingName).Buffer as *mut u8)
            .add((name_length + 2) as usize) as *mut u16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdIsLegalName(
    IrpContext: *mut core::ffi::c_void,
    FileName: *mut _UNICODE_STRING,
) -> i32 {
    // PAGED_CODE();
    let _ = &IrpContext; // UNREFERENCED_PARAMETER

    let mut Wchar = (*FileName).Buffer;
    let end_ptr = ((*FileName).Buffer as *mut u8).add((*FileName).Length as usize) as *mut u16;
    while Wchar < end_ptr {
        if (*Wchar as u32) < 0xff
            && FsRtlIsAnsiCharacterLegalHpfs(*Wchar, 0) == 0
            && *Wchar != 0x22
            && *Wchar != 0x3C
            && *Wchar != 0x3E
            && *Wchar != 0x7C
        {
            return 0; // FALSE
        }
        Wchar = Wchar.add(1);
    }

    return 1; // TRUE
}
#[no_mangle]
pub unsafe extern "C" fn CdIs8dot3Name(
    irp_context: *mut core::ffi::c_void,
    mut file_name: _UNICODE_STRING,
) -> i32 {
    const BYTE_COUNT_8_DOT_3: usize = 24;
    let mut dbcs_name_buffer: [i8; BYTE_COUNT_8_DOT_3] = [0i8; BYTE_COUNT_8_DOT_3];
    let mut dbcs_name: STRING = core::mem::zeroed();
    let mut next_wchar: *mut u16;
    let mut count: u32;
    let mut dot_count: u32 = 0;
    let mut last_char_dot: bool;

    UNREFERENCED_PARAMETER!(irp_context);

    NT_ASSERT((file_name.Length != 0) as i32);
    if file_name.Length as usize > BYTE_COUNT_8_DOT_3 {
        return 0;
    }

    next_wchar = file_name.Buffer;
    count = 0;

    loop {
        if *next_wchar == 0x0020u16 {
            return 0;
        }

        if *next_wchar == 0x002Eu16 {
            if dot_count > 0 || count > 16 {
                return 0;
            }
            dot_count += 1;
            last_char_dot = true;
        } else {
            last_char_dot = false;
        }

        count += 2;
        next_wchar = next_wchar.add(1);

        if count >= file_name.Length as u32 {
            break;
        }
    }

    if last_char_dot {
        file_name.Length -= core::mem::size_of::<u16>() as u16;
    }

    dbcs_name.MaximumLength = BYTE_COUNT_8_DOT_3 as u16;
    dbcs_name.Buffer = dbcs_name_buffer.as_mut_ptr();

    let status = RtlUnicodeStringToCountedOemString(
        &mut dbcs_name as *mut STRING,
        &mut file_name as *mut _UNICODE_STRING,
        0,
    );
    if !NT_SUCCESS(status) {
        return 0;
    }

    FsRtlIsFatDbcsLegal(dbcs_name, 0, 0, 0).into()
}
#[no_mangle]
pub unsafe extern "C" fn CdGenerate8dot3Name(
    IrpContext: *mut core::ffi::c_void,
    FileName: *mut _UNICODE_STRING,
    DirentOffset: u32,
    ShortFileName: *mut u16,
    ShortByteCount: *mut u16,
) {
    // PAGED_CODE();

    let Status: i32;
    let mut ShortName: _UNICODE_STRING = core::mem::zeroed();
    let mut BiasedShortName: _UNICODE_STRING = core::mem::zeroed();
    let mut ShortNameBuffer: [u16; BYTE_COUNT_8_DOT_3 / core::mem::size_of::<u16>()] = [0; BYTE_COUNT_8_DOT_3 / core::mem::size_of::<u16>()];
    let mut BiasedShortNameBuffer: [u16; BYTE_COUNT_8_DOT_3 / core::mem::size_of::<u16>()] = [0; BYTE_COUNT_8_DOT_3 / core::mem::size_of::<u16>()];

    let mut NameContext: GENERATE_NAME_CONTEXT = core::mem::zeroed();
    let mut BiasedDirentOffset: u32;
    let MaximumBaseBytes: u32;
    let mut BaseNameOffset: u32;
    let mut NextWchar: *mut u16;
    let mut ThisWchar: u16;
    let mut Length: u16;
    let mut FoundTilde: bool = false;

    let mut OemName: OEM_STRING = core::mem::zeroed();
    let mut OemNameOffset: u16 = 0;
    let mut OverflowBuffer: bool = false;

    // Initialize ShortName to use the input buffer.
    ShortName.Buffer = ShortNameBuffer.as_mut_ptr();
    ShortName.MaximumLength = BYTE_COUNT_8_DOT_3 as u16;
    ShortName.Length = 0;

    // Initialize the name context (zeroing).
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(NameContext).cast::<u8>(),
        0,
        core::mem::size_of::<GENERATE_NAME_CONTEXT>(),
    );

    // Generate the short name.
    RtlGenerate8dot3Name(FileName, 1, &mut NameContext, &mut ShortName);

    // Bias the dirent offset.
    BiasedDirentOffset = DirentOffset >> SHORT_NAME_SHIFT;

    // Point to the end of the local buffer for the offset string.
    NextWchar = (BiasedShortNameBuffer.as_mut_ptr() as *mut u8)
        .byte_add(BYTE_COUNT_8_DOT_3)
        as *mut u16;
    BiasedShortName.MaximumLength = BYTE_COUNT_8_DOT_3 as u16;
    BiasedShortName.Length = 0;
    BiasedShortName.Buffer = core::ptr::null_mut();

    // Generate OEM version of the string.
    Status = RtlUnicodeStringToOemString(&mut OemName, &mut ShortName, 1);

    if !NT_SUCCESS(Status) {
        NT_ASSERT((STATUS_INSUFFICIENT_RESOURCES == Status) as i32);
        CdRaiseStatus(IrpContext as PIRP_CONTEXT, Status as u32);
    }

    Length = 0;

    // Build the dirent offset string, least significant digit first.
    loop {
        NextWchar = NextWchar.offset(-1);
        ThisWchar = (BiasedDirentOffset & 0x0000000f) as u16;

        if ThisWchar <= 9 {
            *NextWchar = ThisWchar + b'0' as u16;
        } else {
            *NextWchar = ThisWchar + b'A' as u16 - 0xA;
        }

        Length += core::mem::size_of::<u16>() as u16;
        BiasedDirentOffset >>= 4;

        if BiasedDirentOffset == 0 {
            break;
        }
    }

    // Insert the tilde character.
    NextWchar = NextWchar.offset(-1);
    *NextWchar = b'~' as u16;
    Length += core::mem::size_of::<u16>() as u16;

    // Set up the biased short name string.
    BiasedShortName.Length = Length;
    BiasedShortName.Buffer = NextWchar;

    // Determine maximum bytes to copy from the base name.
    MaximumBaseBytes = 16 - Length as u32;
    BaseNameOffset = 0;

    NextWchar = ShortFileName;
    Length = 0;

    // Copy from the base name until '.' or '~' or end.
    while (BaseNameOffset < ShortName.Length as u32)
        && (*ShortName.Buffer.add((BaseNameOffset as usize) / 2) != b'.' as u16)
    {
        if *ShortName.Buffer.add((BaseNameOffset as usize) / 2) == b'~' as u16 {
            FoundTilde = true;
        }

        // Check for lead DBCS character in the OEM name.
        if FsRtlIsLeadDbcsCharacter(*OemName.Buffer.add(OemNameOffset as usize) as u8) != 0 {
            OemNameOffset += 2;
            if (OemNameOffset as u32 + (BiasedShortName.Length as u32 / core::mem::size_of::<u16>() as u32)) > 8 {
                OverflowBuffer = true;
            }
        } else {
            OemNameOffset += 1;
        }

        // Copy only if space remains.
        if !FoundTilde && !OverflowBuffer && (BaseNameOffset < MaximumBaseBytes) {
            *NextWchar = *ShortName.Buffer.add((BaseNameOffset as usize) / 2);
            Length += core::mem::size_of::<u16>() as u16;
            NextWchar = NextWchar.offset(1);
        }

        BaseNameOffset += 2;
    }

    RtlFreeOemString(&mut OemName);

    // Copy the dirent string into the biased name buffer.
    core::ptr::copy_nonoverlapping(
        BiasedShortName.Buffer as *const u8,
        NextWchar as *mut u8,
        BiasedShortName.Length as usize,
    );

    Length += BiasedShortName.Length;
    NextWchar = NextWchar.add((BiasedShortName.Length as usize) / core::mem::size_of::<u16>());

    // Copy any remaining bytes from the base name.
    if BaseNameOffset != ShortName.Length as u32 {
        let src = ShortName.Buffer.add((BaseNameOffset as usize) / 2) as *const u8;
        let dst = NextWchar as *mut u8;
        let count = (ShortName.Length as u32 - BaseNameOffset) as usize;
        core::ptr::copy_nonoverlapping(src, dst, count);

        Length += ShortName.Length as u16 - BaseNameOffset as u16 ;
    }

    // Store the final byte count.
    *ShortByteCount = Length;
}
#[no_mangle]
pub unsafe extern "C" fn CdIsNameInExpression(
    IrpContext: *mut IRP_CONTEXT,
    CurrentName: *mut CD_NAME,
    SearchExpression: *mut CD_NAME,
    WildcardFlags: ULONG,
    CheckVersion: BOOLEAN,
) -> BOOLEAN {
    let mut Match: BOOLEAN = 1;
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext as *mut c_void);

    if FlagOn(WildcardFlags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD) != 0 {
        Match = FsRtlIsNameInExpression(
            core::ptr::addr_of_mut!((*SearchExpression).FileName),
            core::ptr::addr_of_mut!((*CurrentName).FileName),
            0,
            core::ptr::null_mut(),
        );
    } else {
        if ((*CurrentName).FileName.Length != (*SearchExpression).FileName.Length)
            || (RtlEqualMemory(
                (*CurrentName).FileName.Buffer as *const c_void,
                (*SearchExpression).FileName.Buffer as *const c_void,
                (*CurrentName).FileName.Length as usize,
            ) == 0)
        {
            Match = 0;
        }
    }

    if Match != 0 && CheckVersion != 0 && (*SearchExpression).VersionString.Length != 0
        && FlagOn(WildcardFlags, CCB_FLAG_ENUM_VERSION_MATCH_ALL) == 0
    {
        if FlagOn(WildcardFlags, CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD) != 0 {
            Match = FsRtlIsNameInExpression(
                core::ptr::addr_of_mut!((*SearchExpression).VersionString),
                core::ptr::addr_of_mut!((*CurrentName).VersionString),
                0,
                core::ptr::null_mut(),
            );
        } else {
            if ((*CurrentName).VersionString.Length != (*SearchExpression).VersionString.Length)
                || (RtlEqualMemory(
                    (*CurrentName).VersionString.Buffer as *const c_void,
                    (*SearchExpression).VersionString.Buffer as *const c_void,
                    (*CurrentName).VersionString.Length as usize,
                ) == 0)
            {
                Match = 0;
            }
        }
    }

    Match
}
#[no_mangle]
pub unsafe extern "C" fn CdShortNameDirentOffset(
    IrpContext: *mut IRP_CONTEXT,
    Name: PUNICODE_STRING,
) -> u32 {

    let mut ResultOffset = MAXULONG;
    let mut RemainingByteCount;
    let mut FoundTilde: i32 = 0;
    let mut NextWchar: *mut u16;

    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext as *mut c_void);

    RemainingByteCount = (*Name).Length as u32;
    NextWchar = (*Name).Buffer;

    'outer: loop {
        if RemainingByteCount == 0 {
            break;
        }
        'increment: {
            if *NextWchar == b'.' as u16 {
                break 'outer;
            }
            if FoundTilde != 0 {
                if (*NextWchar < b'0' as u16)
                    || (*NextWchar > b'F' as u16)
                    || ((*NextWchar > b'9' as u16) && (*NextWchar < b'A' as u16))
                {
                    ResultOffset = MAXULONG;
                    break 'outer;
                }
                ResultOffset <<= 4;
                if *NextWchar < b'A' as u16 {
                    ResultOffset += (*NextWchar - (b'0' as u16)) as u32;
                } else {
                    ResultOffset += ((*NextWchar - (b'A' as u16)) + 10) as u32;
                }
                break 'increment;
            }
            if *NextWchar == b'~' as u16 {
                FoundTilde = 1;
                ResultOffset = 0;
            }
        }
        NextWchar = NextWchar.offset(1);
        RemainingByteCount -= core::mem::size_of::<u16>() as u32;
    }

    ResultOffset
}
#[no_mangle]
pub unsafe extern "C" fn CdFullCompareNames(
    _IrpContext: PIRP_CONTEXT,
    NameA: PUNICODE_STRING,
    NameB: PUNICODE_STRING,
) -> FSRTL_COMPARISON_RESULT {
    let i: u32;
    let mut MinLength: u16 = (*NameA).Length;
    let mut Result: FSRTL_COMPARISON_RESULT = LessThan;

    if (*NameA).Length > (*NameB).Length {
        MinLength = (*NameB).Length;
        Result = GreaterThan;
    } else if (*NameA).Length == (*NameB).Length {
        Result = EqualTo;
    }

    i = RtlCompareMemory(
        (*NameA).Buffer as *const core::ffi::c_void,
        (*NameB).Buffer as *const core::ffi::c_void,
        MinLength as _,
    ) as _;

    if i < MinLength as u32 {
        return if *((*NameA).Buffer as *const u16).add((i / 2) as usize)
            < *((*NameB).Buffer as *const u16).add((i / 2) as usize)
        {
            LessThan
        } else {
            GreaterThan
        };
    }

    Result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1991-2000 Microsoft Corporation
//
// Module Name:
//
//     NameSup.c
//
// Abstract:
//
//     This module implements the Cdfs Name support routines
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_NAMESUP)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdConvertBigToLittleEndian)
// #pragma alloc_text(PAGE, CdConvertNameToCdName)
// #pragma alloc_text(PAGE, CdDissectName)
// #pragma alloc_text(PAGE, CdGenerate8dot3Name)
// #pragma alloc_text(PAGE, CdFullCompareNames)
// #pragma alloc_text(PAGE, CdIsLegalName)
// #pragma alloc_text(PAGE, CdIs8dot3Name)
// #pragma alloc_text(PAGE, CdIsNameInExpression)
// #pragma alloc_text(PAGE, CdShortNameDirentOffset)
// #pragma alloc_text(PAGE, CdUpcaseName)
// #endif
//
//
// (_Old_(CdName->FileName.Length) >=
//                  CdName->FileName.Length + CdName->VersionString.Length)
// VOID
// CdConvertNameToCdName (
//      PIRP_CONTEXT IrpContext,
//      PCD_NAME CdName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to convert a string of bytes into a CdName.
//
//     The full name is already in the CdName structure in the FileName field.
//     We split this into the filename and version strings.
//
// Arguments:
//
//     CdName - Pointer to CdName structure to update.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG NameLength = 0;
//     PWCHAR CurrentCharacter = CdName->FileName.Buffer;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Look for a separator character.
//     //
//
//     while ((NameLength < CdName->FileName.Length) &&
//            (*CurrentCharacter != L';')) {
//
//         CurrentCharacter += 1;
//         NameLength += 2;
//     }
//
//     //
//     //  If there is at least one more character after a possible separator then it
//     //  and all following characters are part of the version string.
//     //
//
//     CdName->VersionString.Length = 0;
//     if (NameLength + sizeof( WCHAR ) < CdName->FileName.Length) {
//
//         CdName->VersionString.MaximumLength =
//         CdName->VersionString.Length = (USHORT) (CdName->FileName.Length - NameLength - sizeof( WCHAR ));
//         CdName->VersionString.Buffer = Add2Ptr( CdName->FileName.Buffer,
//                                                 NameLength + sizeof( WCHAR ),
//                                                 PWCHAR );
//     }
//
//     //
//     //  Now update the filename portion of the name.
//     //
//
//     CdName->FileName.Length = (USHORT) NameLength;
//
//     return;
// }
//
//
// VOID
// CdConvertBigToLittleEndian (
//      PIRP_CONTEXT IrpContext,
//      PCHAR BigEndian,
//      ULONG ByteCount,
//      PCHAR LittleEndian
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to convert a unicode string in big endian to
//     little endian.  We start by copying all of the source bytes except
//     the first.  This will put the low order bytes in the correct position.
//     We then copy each high order byte in its correct position.
//
// Arguments:
//
//     BigEndian - Pointer to the string of big endian characters.
//
//     ByteCount - Number of unicode characters in this string.
//
//     LittleEndian - Pointer to array to store the little endian characters.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG RemainingByteCount = ByteCount;
//
//     PCHAR Source = BigEndian;
//     PCHAR Destination = LittleEndian;
//
//     PAGED_CODE();
//
//     //
//     //  If the byte count isn't an even number then the disk is corrupt.
//     //
//
//     if (FlagOn( ByteCount, 1 )) {
//
//         CdRaiseStatus( IrpContext, STATUS_DISK_CORRUPT_ERROR );
//     }
//
//     //
//     //  Start by copy the low-order bytes into the correct position.  Do
//     //  this by skipping the first byte in the BigEndian string.
//     //
//
//     RtlCopyMemory( Destination,
//                    Source + 1,
//                    RemainingByteCount - 1 );
//
//     //
//     //  Now move the high-order bytes into position.
//     //
//
//     Destination += 1;
//
//     while (RemainingByteCount != 0) {
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(suppress:26014, "RemainingByteCount is even")
// #endif
//         *Destination = *Source;
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//         Source += 2;
//         Destination += 2;
//
//         RemainingByteCount -= 2;
//     }
//
//     return;
// }
//
//
// VOID
// CdUpcaseName (
//      PIRP_CONTEXT IrpContext,
//      PCD_NAME Name,
//      PCD_NAME UpcaseName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to upcase a CdName structure.  We will do both
//     the filename and version strings.
//
// Arguments:
//
//     Name - This is the mixed case version of the name.
//
//     UpcaseName - This is the destination for the upcase operation.
//
// Return Value:
//
//     None.  This routine will raise all errors.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If the name structures are different then initialize the different components.
//     //
//
//     if (Name != UpcaseName) {
//
//         //
//         //  Initialize the version string portion of the name.
//         //
//
//         UpcaseName->VersionString.Length = 0;
//
//         if (Name->VersionString.Length != 0) {
//
//             UpcaseName->VersionString.MaximumLength =
//             UpcaseName->VersionString.Length = Name->VersionString.Length;
//
//             //
//             //  Initially set the buffer to point to where we need to insert
//             //  the separator.
//             //
//
//             UpcaseName->VersionString.Buffer = Add2Ptr( UpcaseName->FileName.Buffer,
//                                                         Name->FileName.Length,
//                                                         PWCHAR );
//
//             //
//             //  We are currently pointing to the location to store the separator.
//             //  Store the ';' and then move to the next character to
//             //  copy the data.
//             //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:26015, "CD_NAME structures have two UNICODE_STRING structures pointing to the same allocation. there is no way to tell prefast this is the case and that the allocation is always big enough.");
// #endif
//             *(UpcaseName->VersionString.Buffer) = L';';
//
//             UpcaseName->VersionString.Buffer += 1;
//         }
//     }
//
//     //
//     //  Upcase the string using the correct upcase routine.
//     //
//
//     Status = RtlUpcaseUnicodeString( &UpcaseName->FileName,
//                                      &Name->FileName,
//                                      FALSE );
//
//     //
//     //  This should never fail.
//     //
//
//     NT_ASSERT( Status == STATUS_SUCCESS );
//     __analysis_assert( Status == STATUS_SUCCESS );
//
//     if (Name->VersionString.Length != 0) {
//
//         Status = RtlUpcaseUnicodeString( &UpcaseName->VersionString,
//                                          &Name->VersionString,
//                                          FALSE );
//
//         //
//         //  This should never fail.
//         //
//
//         NT_ASSERT( Status == STATUS_SUCCESS );
//         __analysis_assert( Status == STATUS_SUCCESS );
//     }
//
//     return;
// }
//
//
// VOID
// CdDissectName (
//      PIRP_CONTEXT IrpContext,
//      PUNICODE_STRING RemainingName,
//      PUNICODE_STRING FinalName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to strip off leading components of the name strings.  We search
//     for either the end of the string or separating characters.  The input remaining
//     name strings should have neither a trailing or leading backslash.
//
// Arguments:
//
//     RemainingName - Remaining name.
//
//     FinalName - Location to store next component of name.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG NameLength;
//     PWCHAR NextWchar;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Find the offset of the next component separators.
//     //
//
//     for (NameLength = 0, NextWchar = RemainingName->Buffer;
//          (NameLength < RemainingName->Length) && (*NextWchar != L'\\');
//          NameLength += sizeof( WCHAR) , NextWchar += 1);
//
//     //
//     //  Adjust all the strings by this amount.
//     //
//
//     FinalName->Buffer = RemainingName->Buffer;
//
//     FinalName->MaximumLength = FinalName->Length = (USHORT) NameLength;
//
//     //
//     //  If this is the last component then set the RemainingName lengths to zero.
//     //
//
//     if (NameLength == RemainingName->Length) {
//
//         RemainingName->Length = 0;
//
//     //
//     //  Otherwise we adjust the string by this amount plus the separating character.
//     //
//
//     } else {
//
//         RemainingName->MaximumLength -= (USHORT) (NameLength + sizeof( WCHAR ));
//         RemainingName->Length -= (USHORT) (NameLength + sizeof( WCHAR ));
//         RemainingName->Buffer = Add2Ptr( RemainingName->Buffer,
//                                          NameLength + sizeof( WCHAR ),
//                                          PWCHAR );
//     }
//
//     return;
// }
//
//
// BOOLEAN
// CdIsLegalName (
//      PIRP_CONTEXT IrpContext,
//      PUNICODE_STRING FileName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if the name is a legal ISO 9660 name.
//
// Arguments:
//
//     FileName - String of bytes containing the name.
//
// Return Value:
//
//     BOOLEAN - TRUE if this name is a legal, FALSE otherwise.
//
// --*/
//
// {
//     PWCHAR Wchar;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check if name corresponds to a legal file name.
//     //
//
//     for (Wchar = FileName->Buffer;
//          Wchar < Add2Ptr( FileName->Buffer, FileName->Length, PWCHAR );
//          Wchar++) {
//
//         if ((*Wchar < 0xff) &&
//             !FsRtlIsAnsiCharacterLegalHpfs( *Wchar, FALSE ) &&
//             (*Wchar != L'"') &&
//             (*Wchar != L'<') &&
//             (*Wchar != L'>') &&
//             (*Wchar != L'|')) {
//
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// BOOLEAN
// CdIs8dot3Name (
//      PIRP_CONTEXT IrpContext,
//      UNICODE_STRING FileName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if the name follows the 8.3 name conventions.  We check for
//     the name length and whether the characters are valid.
//
// Arguments:
//
//     FileName - String of bytes containing the name.
//
// Return Value:
//
//     BOOLEAN - TRUE if this name is a legal 8.3 name, FALSE otherwise.
//
// --*/
//
// {
//     CHAR DbcsNameBuffer[ BYTE_COUNT_8_DOT_3 ];
//     STRING DbcsName = {0};
//
//     PWCHAR NextWchar;
//     ULONG Count;
//
//     ULONG DotCount = 0;
//     BOOLEAN LastCharDot = FALSE;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  The length must be less than 24 bytes.
//     //
//
//     NT_ASSERT( FileName.Length != 0 );
//     if (FileName.Length > BYTE_COUNT_8_DOT_3) {
//
//         return FALSE;
//     }
//
//     //
//     //  Walk though and check for a space character.
//     //
//
//     NextWchar = FileName.Buffer;
//     Count = 0;
//
//     do {
//
//         //
//         //  No spaces allowed.
//         //
//
//         if (*NextWchar == L' ') { return FALSE; }
//
//         if (*NextWchar == L'.') {
//
//             //
//             //  Not an 8.3 name if more than 1 dot or more than 8 characters
//             //  remaining.  (It is legal for the dot to be in the ninth
//             //  position)
//             //
//
//             if ((DotCount > 0) ||
//                 (Count > 8 * sizeof( WCHAR ))) {
//
//                 return FALSE;
//             }
//
//             DotCount += 1;
//             LastCharDot = TRUE;
//
//         } else {
//
//             LastCharDot = FALSE;
//         }
//
//         Count += 2;
//         NextWchar += 1;
//
//     } while (Count < FileName.Length);
//
//     //
//     //  Go ahead and truncate the dot if at the end.
//     //
//
//     if (LastCharDot) {
//
//         FileName.Length -= sizeof( WCHAR );
//     }
//
//     //
//     //  Create an Oem name to use to check for a valid short name.
//     //
//
//     DbcsName.MaximumLength = BYTE_COUNT_8_DOT_3;
//     DbcsName.Buffer = DbcsNameBuffer;
//
//     if (!NT_SUCCESS( RtlUnicodeStringToCountedOemString( &DbcsName,
//                                                          &FileName,
//                                                          FALSE ))) {
//
//         return FALSE;
//     }
//
//     //
//     //  We have now initialized the Oem string.  Call the FsRtl package to check for a
//     //  valid FAT name.
//     //
//
//     return FsRtlIsFatDbcsLegal( DbcsName, FALSE, FALSE, FALSE );
// }
//
//
// VOID
// CdGenerate8dot3Name (
//      PIRP_CONTEXT IrpContext,
//      PUNICODE_STRING FileName,
//      ULONG DirentOffset,
//      PWCHAR ShortFileName,
//      PUSHORT ShortByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to generate a short name from the given long name.  We will
//     generate a short name from the given long name.
//
//     We go through the following steps to make this conversion.
//
//         1 - Generate the generic short name.  This will also be in unicode format.
//
//         2 - Build the string representation of the dirent offset.
//
//         3 - Build the biased short name string by combining the generic short name with
//             the dirent offset string.
//
//         4 - Copy the final unicode string back to our caller's buffer.
//
// Arguments:
//
//     FileName - String of bytes containing the name.
//
//     DirentOffset - Offset in the directory for this filename.  We incorporate the offset into
//         the short name by dividing this by 32 and prepending a tilde character to the
//         digit character.  We then append this to the base of the generated short name.
//
//     ShortFileName - Pointer to the buffer to store the short name into.
//
//     ShortByteCount - Address to store the number of bytes in the short name.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     UNICODE_STRING ShortName;
//     UNICODE_STRING BiasedShortName;
//     WCHAR ShortNameBuffer[ BYTE_COUNT_8_DOT_3 / sizeof( WCHAR ) ] = {0};
//     WCHAR BiasedShortNameBuffer[ BYTE_COUNT_8_DOT_3 / sizeof( WCHAR ) ];
//
//     GENERATE_NAME_CONTEXT NameContext;
//
//     ULONG BiasedDirentOffset;
//
//     ULONG MaximumBaseBytes;
//     ULONG BaseNameOffset;
//
//     PWCHAR NextWchar;
//     WCHAR ThisWchar;
//     USHORT Length;
//
//     BOOLEAN FoundTilde = FALSE;
//
//     OEM_STRING OemName = {0};
//     USHORT OemNameOffset = 0;
//     BOOLEAN OverflowBuffer = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the short string to use the input buffer.
//     //
//
//     ShortName.Buffer = ShortNameBuffer;
//     ShortName.MaximumLength = BYTE_COUNT_8_DOT_3;
//
//     //
//     //  Initialize the name context.
//     //
//
//     RtlZeroMemory( &NameContext, sizeof( GENERATE_NAME_CONTEXT ));
//
//     //
//     //  We now have the unicode name for the input string.  Go ahead and generate
//     //  the short name.
//     //
//
//     RtlGenerate8dot3Name( FileName, TRUE, &NameContext, &ShortName );
//
//     //
//     //  We now have the generic short name.  We want incorporate the dirent offset
//     //  into the name in order to reduce the chance of name conflicts.  We will use
//     //  a tilde character followed by a character representation of the dirent offset.
//     //  This will be the hexadecimal representation of the dirent offset in the directory.
//     //  It is actuall this offset divided by 32 since we don't need the full
//     //  granularity.
//     //
//
//     BiasedDirentOffset = DirentOffset >> SHORT_NAME_SHIFT;
//
//     //
//     //  Point to a local buffer to store the offset string.  We start
//     //  at the end of the buffer and work backwards.
//     //
//
//     NextWchar = Add2Ptr( BiasedShortNameBuffer,
//                          BYTE_COUNT_8_DOT_3,
//                          PWCHAR );
//
//     BiasedShortName.MaximumLength = BYTE_COUNT_8_DOT_3;
//
//     //
//     //  Generate an OEM version of the string so that we can check for double
//     //  byte characters.
//     //
//
//     Status = RtlUnicodeStringToOemString(&OemName, &ShortName, TRUE);
//
//     //
//     //  If this failed, bail out. Don't expect any problems other than no mem.
//     //
//
//     if (!NT_SUCCESS( Status)) {
//
//         NT_ASSERT( STATUS_INSUFFICIENT_RESOURCES == Status);
//         CdRaiseStatus( IrpContext, Status);
//     }
//
//     Length = 0;
//
//     //
//     //  Now add the characters for the dirent offset.  We need to start
//     //  from the least significant digit and work backwards.
//     //
//
//     do {
//
//         NextWchar -= 1;
//
//         ThisWchar = (WCHAR) (BiasedDirentOffset & 0x0000000f);
//
//         //
//         //  Store in the next character.  Bias against either '0' or 'A'
//         //
//
//         if (ThisWchar <= 9) {
//
//             *NextWchar = ThisWchar + L'0';
//
//         } else {
//
//             *NextWchar = ThisWchar + L'A' - 0xA;
//         }
//
//         Length += sizeof( WCHAR );
//
//         //
//         //  Shift out the low 4 bits of the offset.
//         //
//
//         BiasedDirentOffset >>= 4;
//
//     } while (BiasedDirentOffset != 0);
//
//     //
//     //  Now store in the tilde character.
//     //
//
//     NextWchar -= 1;
//     *NextWchar = L'~';
//     Length += sizeof( WCHAR );
//
//     //
//     //  Set the length of this string.
//     //
//
//     BiasedShortName.Length = Length;
//     BiasedShortName.Buffer = NextWchar;
//
//     //
//     //  Figure out the maximum number of characters we can copy of the base
//     //  name.  We subract the number of characters in the dirent string from 8.
//     //  We will copy this many characters or stop when we reach a '.' character
//     //  or a '~' character in the name.
//     //
//
//     MaximumBaseBytes = 16 - Length;
//
//     BaseNameOffset = 0;
//
//     //
//     //  Keep copying from the base name until we hit a '.', '~'  or the end of
//     //  the short name.
//     //
//
//     NextWchar = ShortFileName;
//     Length = 0;
//
//     while ((BaseNameOffset < ShortName.Length) &&
//            (ShortName.Buffer[BaseNameOffset / 2] != L'.')) {
//
//         //
//         //  Remember if we found a tilde character in the short name,
//         //  so we don't copy it or anything following it.
//         //
//
//         if (ShortName.Buffer[BaseNameOffset / 2] == L'~') {
//
//             FoundTilde = TRUE;
//         }
//
//         //
//         // We need to consider the DBCS code page,  because Unicode characters
//         // may use 2 bytes as DBCS characters.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(suppress:26014, "OemNameOffset <= BaseNameOffset throughout this loop; OemName buffer previously allocated based on ShortName's length.")
// #endif
//         if (FsRtlIsLeadDbcsCharacter(OemName.Buffer[OemNameOffset])) {
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//             OemNameOffset += 2;
//
//             if ((OemNameOffset + (BiasedShortName.Length / sizeof(WCHAR))) > 8)  {
//
//                 OverflowBuffer = TRUE;
//             }
//         }
//         else  {
//
//             OemNameOffset++;
//         }
//
//         //
//         //  Only copy the bytes if we still have space for the dirent string.
//         //
//
//         if (!FoundTilde && !OverflowBuffer && (BaseNameOffset < MaximumBaseBytes)) {
//
//             *NextWchar = ShortName.Buffer[BaseNameOffset / 2];
//             Length += sizeof( WCHAR );
//             NextWchar += 1;
//         }
//
//         BaseNameOffset += 2;
//     }
//
//     RtlFreeOemString(&OemName);
//
//     //
//     //  Now copy the dirent string into the biased name buffer.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #endif
//     RtlCopyMemory( NextWchar,
//                    BiasedShortName.Buffer,
//                    BiasedShortName.Length );
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//     Length += BiasedShortName.Length;
//     NextWchar += (BiasedShortName.Length / sizeof( WCHAR ));
//
//     //
//     //  Now copy any remaining bytes over to the biased short name.
//     //
//
//     if (BaseNameOffset != ShortName.Length) {
//
//         RtlCopyMemory( NextWchar,
//                        &ShortName.Buffer[BaseNameOffset / 2],
//                        ShortName.Length - BaseNameOffset );
//
//         Length += (ShortName.Length - (USHORT) BaseNameOffset);
//     }
//
//     //
//     //  The final short name is stored in the user's buffer.
//     //
//
//     *ShortByteCount = Length;
// }
//
//
// BOOLEAN
// CdIsNameInExpression (
//      PIRP_CONTEXT IrpContext,
//      PCD_NAME CurrentName,
//      PCD_NAME SearchExpression,
//      ULONG  WildcardFlags,
//      BOOLEAN CheckVersion
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will compare two CdName strings.  We assume that if this
//     is to be a case-insensitive search then they are already upcased.
//
//     We compare the filename portions of the name and if they match we
//     compare the version strings if requested.
//
// Arguments:
//
//     CurrentName - Filename from the disk.
//
//     SearchExpression - Filename expression to use for match.
//
//     WildcardFlags - Flags field which indicates which parts of the
//         search expression might have wildcards.  These flags are the
//         same as in the Ccb flags field.
//
//     CheckVersion - Indicates whether we should check both the name and the
//         version strings or just the name.
//
// Return Value:
//
//     BOOLEAN - TRUE if the expressions match, FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN Match = TRUE;
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If there are wildcards in the expression then we call the
//     //  appropriate FsRtlRoutine.
//     //
//
//     if (FlagOn( WildcardFlags, CCB_FLAG_ENUM_NAME_EXP_HAS_WILD )) {
//
//         Match = FsRtlIsNameInExpression( &SearchExpression->FileName,
//                                          &CurrentName->FileName,
//                                          FALSE,
//                                          NULL );
//
//     //
//     //  Otherwise do a direct memory comparison for the name string.
//     //
//
//     } else {
//
//         if ((CurrentName->FileName.Length != SearchExpression->FileName.Length) ||
//             (!RtlEqualMemory( CurrentName->FileName.Buffer,
//                               SearchExpression->FileName.Buffer,
//                               CurrentName->FileName.Length ))) {
//
//             Match = FALSE;
//         }
//     }
//
//     //
//     //  Check the version numbers if requested by the user and we have a
//     //  match on the name and the version number is present.
//     //
//
//     if (Match && CheckVersion && SearchExpression->VersionString.Length &&
//         !FlagOn( WildcardFlags, CCB_FLAG_ENUM_VERSION_MATCH_ALL )) {
//
//         //
//         //  If there are wildcards in the expression then call the
//         //  appropriate search expression.
//         //
//
//         if (FlagOn( WildcardFlags, CCB_FLAG_ENUM_VERSION_EXP_HAS_WILD )) {
//
//             Match = FsRtlIsNameInExpression( &SearchExpression->VersionString,
//                                              &CurrentName->VersionString,
//                                              FALSE,
//                                              NULL );
//
//         //
//         //  Otherwise do a direct memory comparison for the name string.
//         //
//
//         } else {
//
//             if ((CurrentName->VersionString.Length != SearchExpression->VersionString.Length) ||
//                 (!RtlEqualMemory( CurrentName->VersionString.Buffer,
//                                   SearchExpression->VersionString.Buffer,
//                                   CurrentName->VersionString.Length ))) {
//
//                 Match = FALSE;
//             }
//         }
//     }
//
//     return Match;
// }
//
//
// ULONG
// CdShortNameDirentOffset (
//      PIRP_CONTEXT IrpContext,
//      PUNICODE_STRING Name
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to examine a name to see if the dirent offset string is contained.
//     This consists of a tilde character followed by the offset represented as a hexadecimal
//     characters.  We don't do any other checks to see if this is a short name.  We
//     catch that later outside this routine.
//
// Arguments:
//
//     Name - This is the CdName to examine.
//
// Return Value:
//
//     ULONG - MAXULONG if there is no valid dirent offset string embedded, otherwise the
//         convert the value to numeric form.
//
// --*/
//
// {
//     ULONG ResultOffset = MAXULONG;
//     ULONG RemainingByteCount = Name->Length;
//
//     BOOLEAN FoundTilde = FALSE;
//
//     PWCHAR NextWchar;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Walk through the name until we either reach the end of the name
//     //  or find a tilde character.
//     //
//
//     for (NextWchar = Name->Buffer;
//          RemainingByteCount != 0;
//          NextWchar += 1, RemainingByteCount -= sizeof( WCHAR )) {
//
//         //
//         //  Check if this is a dot.  Stop constructing any string if
//         //  we found a dot.
//         //
//
//         if (*NextWchar == L'.') {
//
//             break;
//         }
//
//         //
//         //  If we already found a tilde then check this character as a
//         //  valid character.  It must be a digit or A to F.
//         //
//
//         if (FoundTilde) {
//
//             if ((*NextWchar < L'0') ||
//                 (*NextWchar > L'F') ||
//                 ((*NextWchar > L'9') && (*NextWchar < 'A'))) {
//
//                 ResultOffset = MAXULONG;
//                 break;
//             }
//
//             //
//             //  Shift the result by 4 bits and add in this new character.
//             //
//
//             ResultOffset <<= 4;
//
//             if (*NextWchar < L'A') {
//
//                 ResultOffset += *NextWchar - L'0';
//
//             } else {
//
//                 ResultOffset += (*NextWchar - L'A') + 10;
//             }
//
//             continue;
//         }
//
//         //
//         //  If this is a tilde then start building the dirent string.
//         //
//
//         if (*NextWchar == L'~') {
//
//             FoundTilde = TRUE;
//             ResultOffset = 0;
//         }
//     }
//
//     return ResultOffset;
// }
//
//
// //
// //  Local support routine
// //
//
// FSRTL_COMPARISON_RESULT
// CdFullCompareNames (
//      PIRP_CONTEXT IrpContext,
//      PUNICODE_STRING NameA,
//      PUNICODE_STRING NameB
//     )
//
// /*++
//
// Routine Description:
//
//     This function compares two names as fast as possible.  Note that since
//     this comparison is case sensitive we can do a direct memory comparison.
//
// Arguments:
//
//     NameA & NameB - The names to compare.
//
// Return Value:
//
//     COMPARISON - returns
//
//         LessThan    if NameA < NameB lexicalgraphically,
//         GreaterThan if NameA > NameB lexicalgraphically,
//         EqualTo     if NameA is equal to NameB
//
// --*/
//
// {
//     SIZE_T i;
//     ULONG MinLength = NameA->Length;
//     FSRTL_COMPARISON_RESULT Result = LessThan;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Figure out the minimum of the two lengths
//     //
//
//     if (NameA->Length > NameB->Length) {
//
//         MinLength = NameB->Length;
//         Result = GreaterThan;
//
//     } else if (NameA->Length == NameB->Length) {
//
//         Result = EqualTo;
//     }
//
//     //
//     //  Loop through looking at all of the characters in both strings
//     //  testing for equalilty, less than, and greater than
//     //
//
//     i = RtlCompareMemory( NameA->Buffer, NameB->Buffer, MinLength );
//
//     if (i < MinLength) {
//
//         //
//         //  We know the offset of the first character which is different.
//         //
//
//         return ((NameA->Buffer[ i / 2 ] < NameB->Buffer[ i / 2 ]) ?
//                  LessThan :
//                  GreaterThan);
//     }
//
//     //
//     //  The names match up to the length of the shorter string.
//     //  The shorter string lexically appears first.
//     //
//
//     return Result;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
