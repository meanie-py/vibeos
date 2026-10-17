use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    ANSI_STRING, PANSI_STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use core::ptr;

const FSRTL_FAT_LEGAL: u8 = 0x01;
const FSRTL_HPFS_LEGAL: u8 = 0x02;
const FSRTL_WILD_CHARACTER: u8 = 0x08;

extern "system" {
    static NlsMbOemCodePageTag: BOOLEAN;
    static NlsOemLeadByteInfo: *const USHORT;
}

#[inline(always)]
unsafe fn FsRtlIsLeadDbcsCharacter(c: u8) -> bool {
    // fsrtlfuncs.h:1637 opens with `(UCHAR)(DBCS_CHAR) < 0x80 ? FALSE : ...`.
    // The lead-byte table is only meaningful above 0x7F, and skipping the guard
    // let an ASCII byte be classified as a lead byte if the table happened to
    // carry anything below 0x80.
    c >= 0x80 && NlsMbOemCodePageTag != 0 && *NlsOemLeadByteInfo.add(c as usize) != 0
}

/// `FsRtlTestAnsiCharacter` (fsrtlfuncs.h:1632).
///
/// The `(SCHAR)(C) < 0` short-circuit is the whole point of this helper, and it
/// was missing from all three wrappers below: LEGAL_ANSI_CHARACTER_ARRAY has
/// **128** entries, so indexing it with a byte >= 0x80 read up to 128 bytes past
/// the end of a static — real out-of-bounds UB, not merely a wrong answer. The C
/// never gets there: a high byte is DBCS and returns `default_ret` unexamined.
#[inline(always)]
unsafe fn FsRtlTestAnsiCharacter(c: u8, default_ret: bool, wild_ok: bool, flags_mask: u8) -> bool {
    if c >= 0x80 {
        return default_ret;
    }
    let flags = *super::fsrtlpc::LEGAL_ANSI_CHARACTER_ARRAY_DATA.as_ptr().add(c as usize);
    (flags & flags_mask != 0) || (wild_ok && (flags & FSRTL_WILD_CHARACTER != 0))
}

#[inline(always)]
unsafe fn FsRtlIsAnsiCharacterLegalFat(c: u8, wild_ok: bool) -> bool {
    FsRtlTestAnsiCharacter(c, true, wild_ok, FSRTL_FAT_LEGAL)
}

#[inline(always)]
unsafe fn FsRtlIsAnsiCharacterWild(c: u8) -> bool {
    FsRtlTestAnsiCharacter(c, false, false, FSRTL_WILD_CHARACTER)
}

#[inline(always)]
unsafe fn FsRtlIsAnsiCharacterLegalHpfs(c: u8, wild_ok: bool) -> bool {
    FsRtlTestAnsiCharacter(c, true, wild_ok, FSRTL_HPFS_LEGAL)
}

#[inline(always)]
unsafe fn RtlEqualMemory(a: PVOID, b: PVOID, len: usize) -> bool {
    ntkrnl_sys::ddk::RtlCompareMemory(a, b, len as _) == len as _
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDissectDbcs(
    Name: ANSI_STRING,
    FirstPart: PANSI_STRING,
    RemainingPart: PANSI_STRING,
) {
    let FirstPart = opaque_ptr(FirstPart);
    let RemainingPart = opaque_ptr(RemainingPart);
    let mut first_position: USHORT;
    let mut i: USHORT;
    let mut skip_first_slash: USHORT = 0;

    core::ptr::write_bytes(FirstPart as *mut u8, 0, core::mem::size_of::<ANSI_STRING>());
    core::ptr::write_bytes(RemainingPart as *mut u8, 0, core::mem::size_of::<ANSI_STRING>());

    if Name.Length == 0 {
        return;
    }

    first_position = Name.Length;
    i = 0;
    while i < Name.Length {
        if FsRtlIsLeadDbcsCharacter(*Name.Buffer.add(i as usize) as u8) {
            i += 1;
        } else if *Name.Buffer.add(i as usize) as u8 == b'\\' {
            if i == 0 {
                skip_first_slash = 1;
            } else {
                first_position = i;
                break;
            }
        }
        i += 1;
    }

    (*FirstPart).Buffer = Name.Buffer.add(skip_first_slash as usize);
    (*FirstPart).Length = first_position - skip_first_slash;
    (*FirstPart).MaximumLength = (*FirstPart).Length;

    if first_position < Name.Length {
        (*RemainingPart).Buffer = Name.Buffer.add((first_position + 1) as usize);
        (*RemainingPart).Length = Name.Length - (first_position + 1);
        (*RemainingPart).MaximumLength = (*RemainingPart).Length;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDoesDbcsContainWildCards(Name: PANSI_STRING) -> BOOLEAN {
    let Name = opaque_ptr(Name);
    let mut i: USHORT = 0;

    while i < (*Name).Length {
        if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.add(i as usize) as u8) {
            i += 1;
        } else if FsRtlIsAnsiCharacterWild(*(*Name).Buffer.add(i as usize) as u8) {
            return 1u8;
        }
        i += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsDbcsInExpression(
    Expression: PANSI_STRING,
    Name: PANSI_STRING,
) -> BOOLEAN {
    let Expression = opaque_ptr(Expression);
    let Name = opaque_ptr(Name);
    let mut offset: USHORT;
    let mut position: USHORT;
    let mut backtracking_position: USHORT;
    let mut old_backtracking_position: USHORT;
    let mut backtracking_buffer: [USHORT; 16] = [0; 16];
    let mut old_backtracking_buffer: [USHORT; 16] = [0; 16];
    let mut backtracking_swap: *mut USHORT;
    let mut backtracking: *mut USHORT = backtracking_buffer.as_mut_ptr();
    let mut old_backtracking: *mut USHORT = old_backtracking_buffer.as_mut_ptr();
    let mut backtracking_buffer_size: ULONG = 16;
    let mut allocated_buffer: PVOID = core::ptr::null_mut();
    let mut expression_position: USHORT;
    let mut name_position: USHORT = 0;
    let mut matching_chars: USHORT = 1;
    let mut name_char: USHORT = 0;
    let mut expression_char: USHORT;
    let mut end_of_name: BOOLEAN = 0u8;
    let result: BOOLEAN;
    let mut alloc_failed = false;
    let mut dont_skip_dot: BOOLEAN;

    if (*Name).Length == 0 || (*Expression).Length == 0 {
        if (*Name).Length == 0 && (*Expression).Length == 0 {
            return 1u8;
        } else {
            return 0u8;
        }
    }

    if (*Expression).Length == core::mem::size_of::<i8>() as USHORT {
        if *(*Expression).Buffer == b'*' as i8 {
            return 1u8;
        }
    }

    if *(*Expression).Buffer == b'*' as i8 {
        let mut int_expression = *Expression;

        int_expression.Buffer = int_expression.Buffer.add(1);
        int_expression.Length -= core::mem::size_of::<i8>() as USHORT;

        if FsRtlDoesDbcsContainWildCards(&mut int_expression) == 0 {
            if (*Name).Length < ((*Expression).Length - core::mem::size_of::<i8>() as USHORT) {
                return 0u8;
            }

            name_position = ((*Name).Length - int_expression.Length) / core::mem::size_of::<i8>() as USHORT;

            if NlsMbOemCodePageTag != 0 {
                matching_chars = 0;

                while matching_chars < name_position {
                    matching_chars += if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.add(matching_chars as usize) as u8) {
                        2
                    } else {
                        1
                    };
                }

                if matching_chars > name_position {
                    return 0u8;
                }
            }

            return RtlEqualMemory(
                int_expression.Buffer as PVOID,
                (*Name).Buffer.add(name_position as usize) as PVOID,
                int_expression.Length as usize,
            ) as BOOLEAN;
        }
    }

    'outer: loop {
        old_backtracking_position = 0;
        backtracking_position = 0;

        if name_position >= (*Name).Length {
            end_of_name = 1u8;
            if matching_chars != 0 && *old_backtracking.add((matching_chars - 1) as usize) == (*Expression).Length * 2 {
                break;
            }
        } else {
            if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.add(name_position as usize) as u8) {
                name_char = (*(*Name).Buffer.add(name_position as usize) as u8 as USHORT) +
                    (0x100 * (*(*Name).Buffer.add((name_position + 1) as usize) as USHORT));
                name_position += core::mem::size_of::<USHORT>() as USHORT;
            } else {
                name_char = *(*Name).Buffer.add(name_position as usize) as u8 as USHORT;
                name_position += core::mem::size_of::<u8>() as USHORT;
            }
        }

        while matching_chars > old_backtracking_position {
            expression_position = (*old_backtracking.add(old_backtracking_position as usize) + 1) / 2;
            old_backtracking_position += 1;

            offset = 0;
            while expression_position < (*Expression).Length {
                expression_position += offset;

                if expression_position == (*Expression).Length {
                    *backtracking.add(backtracking_position as usize) = (*Expression).Length * 2;
                    backtracking_position += 1;
                    break;
                }

                if backtracking_position > backtracking_buffer_size as USHORT - 3 {
                    backtracking_buffer_size = ((*Expression).Length as ULONG) * 2 + 1;

                    allocated_buffer = ExAllocatePoolWithTag(
                        PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
                        2 * backtracking_buffer_size as usize * core::mem::size_of::<USHORT>(),
                        u32::from_ne_bytes(*b"nrSF"),
                    );
                    if allocated_buffer.is_null() {
                        alloc_failed = true;
                        break 'outer;
                    }

                    core::ptr::copy_nonoverlapping(
                        backtracking as *const u8,
                        allocated_buffer as *mut u8,
                        16 as usize * core::mem::size_of::<USHORT>(),
                    );

                    backtracking = allocated_buffer as *mut USHORT;

                    core::ptr::copy_nonoverlapping(
                        old_backtracking as *const u8,
                        (backtracking as *mut u8).add((backtracking_buffer_size as usize * core::mem::size_of::<USHORT>()) as usize),
                        16 as usize * core::mem::size_of::<USHORT>(),
                    );

                    old_backtracking = backtracking.add(backtracking_buffer_size as usize);
                }

                if FsRtlIsLeadDbcsCharacter(*(*Expression).Buffer.add(expression_position as usize) as u8) {
                    expression_char = (*(*Expression).Buffer.add(expression_position as usize) as u8 as USHORT) +
                        (0x100 * (*(*Expression).Buffer.add((expression_position + 1) as usize) as USHORT));
                    offset = core::mem::size_of::<USHORT>() as USHORT;
                } else {
                    expression_char = *(*Expression).Buffer.add(expression_position as usize) as u8 as USHORT;
                    offset = core::mem::size_of::<u8>() as USHORT;
                }

                if expression_char == name_char && end_of_name == 0 {
                    *backtracking.add(backtracking_position as usize) = (expression_position + offset) * 2;
                    backtracking_position += 1;
                } else if expression_char == b'?' as USHORT && end_of_name == 0 {
                    *backtracking.add(backtracking_position as usize) = (expression_position + offset) * 2;
                    backtracking_position += 1;
                } else if expression_char == b'*' as USHORT {
                    *backtracking.add(backtracking_position as usize) = expression_position * 2;
                    backtracking_position += 1;
                    *backtracking.add(backtracking_position as usize) = (expression_position * 2) + 1;
                    backtracking_position += 1;
                    continue;
                } else if expression_char == ANSI_DOS_STAR as USHORT {
                    dont_skip_dot = 1u8;
                    if end_of_name == 0 && name_char == b'.' as USHORT {
                        position = name_position;
                        while position < (*Name).Length {
                            if !FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.add(position as usize) as u8) {
                                if *(*Name).Buffer.add(position as usize) as u8 == b'.' {
                                    dont_skip_dot = 0u8;
                                    break;
                                }

                                position += core::mem::size_of::<u8>() as USHORT;
                            } else {
                                position += core::mem::size_of::<USHORT>() as USHORT;
                            }
                        }
                    }

                    if end_of_name != 0 || name_char != b'.' as USHORT || dont_skip_dot == 0 {
                        *backtracking.add(backtracking_position as usize) = expression_position * 2;
                        backtracking_position += 1;
                    }

                    *backtracking.add(backtracking_position as usize) = (expression_position * 2) + 1;
                    backtracking_position += 1;
                    continue;
                } else if expression_char == ANSI_DOS_DOT as USHORT {
                    if end_of_name != 0 {
                        continue;
                    }

                    if name_char == b'.' as USHORT {
                        *backtracking.add(backtracking_position as usize) = (expression_position + offset) * 2;
                        backtracking_position += 1;
                    }
                } else if expression_char == ANSI_DOS_QM as USHORT {
                    if end_of_name != 0 || name_char == b'.' as USHORT {
                        continue;
                    }

                    *backtracking.add(backtracking_position as usize) = (expression_position + offset) * 2;
                    backtracking_position += 1;
                }

                break;
            }

            position = 0;
            while matching_chars > old_backtracking_position && position < backtracking_position {
                while matching_chars > old_backtracking_position &&
                    *backtracking.add(position as usize) > *old_backtracking.add(old_backtracking_position as usize)
                {
                    old_backtracking_position += 1;
                }
                position += 1;
            }
        }

        backtracking_swap = backtracking;
        backtracking = old_backtracking;
        old_backtracking = backtracking_swap;

        // The C's `for (; !EndOfName; MatchingChars = BackTrackingPosition)` takes the count
        // BEFORE testing EndOfName, so the end-of-name pass (a trailing `*`, DOS_STAR or
        // DOS_DOT reaching the end of the expression) counts. Breaking first dropped those
        // matches: `F0_*.*` did not match `F0_.` (FsRtlExpression.c:233).
        matching_chars = backtracking_position;

        if end_of_name != 0 {
            break;
        }
    }

    // The C jumps past this to Exit on an allocation failure.
    result = if !alloc_failed && matching_chars != 0 && *old_backtracking.add((matching_chars - 1) as usize) == (*Expression).Length * 2 {
        1u8
    } else {
        0u8
    };

    'Exit: {
        if !allocated_buffer.is_null() {
            ExFreePoolWithTag(allocated_buffer, u32::from_ne_bytes(*b"nrSF"));
        }
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsFatDbcsLegal(
    DbcsName: ANSI_STRING,
    WildCardsPermissible: BOOLEAN,
    PathNamePermissible: BOOLEAN,
    LeadingBackslashPermissible: BOOLEAN,
) -> BOOLEAN {
    let mut first_part: ANSI_STRING = core::mem::zeroed();
    let mut remaining_part: ANSI_STRING = core::mem::zeroed();
    let mut last_dot: BOOLEAN;
    let mut i: USHORT;
    let mut dbcs_name = DbcsName;

    if dbcs_name.Length == 0 {
        return 0u8;
    }

    if WildCardsPermissible != 0 && (dbcs_name.Length == 1 || dbcs_name.Length == 2) && *dbcs_name.Buffer == b'.' as i8 {
        if dbcs_name.Length == 2 {
            if *dbcs_name.Buffer.add(1) as u8 == b'.' {
                return 1u8;
            }
        } else {
            return 1u8;
        }
    }

    if LeadingBackslashPermissible == 0 && *dbcs_name.Buffer == b'\\' as i8 {
        return 0u8;
    } else if LeadingBackslashPermissible != 0 && *dbcs_name.Buffer == b'\\' as i8 {
        dbcs_name.Buffer = dbcs_name.Buffer.add(1);
        dbcs_name.Length = dbcs_name.Length - 1;
        dbcs_name.MaximumLength = dbcs_name.MaximumLength - 1;
    }

    if PathNamePermissible != 0 {
        remaining_part.Buffer = dbcs_name.Buffer;
        remaining_part.Length = dbcs_name.Length;
        remaining_part.MaximumLength = dbcs_name.MaximumLength;

        while remaining_part.Length > 0 {
            if *remaining_part.Buffer == b'\\' as i8 {
                return 0u8;
            }

            FsRtlDissectDbcs(remaining_part, &mut first_part, &mut remaining_part);

            if FsRtlIsFatDbcsLegal(first_part, WildCardsPermissible, 0u8, 0u8) == 0 {
                return 0u8;
            }
        }

        return 1u8;
    }

    if WildCardsPermissible != 0 && FsRtlDoesDbcsContainWildCards(&mut dbcs_name) != 0 {
        i = 0;
        while i < dbcs_name.Length {
            if FsRtlIsLeadDbcsCharacter(*dbcs_name.Buffer.add(i as usize) as u8) {
                i += 1;
            } else if !FsRtlIsAnsiCharacterLegalFat(*dbcs_name.Buffer.add(i as usize) as u8, true) {
                return 0u8;
            }
            i += 1;
        }

        return 1u8;
    }

    if dbcs_name.Length > 12 {
        return 0u8;
    }

    last_dot = 0u8;

    i = 0;
    while i < dbcs_name.Length {
        if FsRtlIsLeadDbcsCharacter(*dbcs_name.Buffer.add(i as usize) as u8) {
            if last_dot == 0 && i >= 7 {
                return 0u8;
            }

            if i == (dbcs_name.Length - 1) {
                return 0u8;
            }

            i += 1;
            i += 1;
            continue;
        } else if !FsRtlIsAnsiCharacterLegalFat(*dbcs_name.Buffer.add(i as usize) as u8, WildCardsPermissible != 0) {
            return 0u8;
        } else if *dbcs_name.Buffer.add(i as usize) as u8 == b'.' {
            if last_dot != 0 {
                return 0u8;
            }

            last_dot = 1u8;

            if i == (dbcs_name.Length - 1) || i == 0 {
                return 0u8;
            }

            if (dbcs_name.Length - 1) - i > 3 {
                return 0u8;
            }

            if i > 0 && *dbcs_name.Buffer.add((i - 1) as usize) == b' ' as i8 {
                return 0u8;
            }
        } else if *dbcs_name.Buffer.add(i as usize) as u8 == b' ' && i == (dbcs_name.Length - 1) {
            return 0u8;
        }

        if last_dot == 0 && i >= 8 {
            return 0u8;
        }

        i += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsHpfsDbcsLegal(
    DbcsName: ANSI_STRING,
    WildCardsPermissible: BOOLEAN,
    PathNamePermissible: BOOLEAN,
    LeadingBackslashPermissible: BOOLEAN,
) -> BOOLEAN {
    let mut first_part: ANSI_STRING = core::mem::zeroed();
    let mut remaining_part: ANSI_STRING = core::mem::zeroed();
    let mut i: USHORT;
    let mut dbcs_name = DbcsName;

    if dbcs_name.Length == 0 {
        return 0u8;
    }

    if WildCardsPermissible != 0 && (dbcs_name.Length == 1 || dbcs_name.Length == 2) && *dbcs_name.Buffer == b'.' as i8 {
        if dbcs_name.Length == 2 {
            if *dbcs_name.Buffer.add(1) as u8 == b'.' {
                return 1u8;
            }
        } else {
            return 1u8;
        }
    }

    if LeadingBackslashPermissible == 0 && *dbcs_name.Buffer == b'\\' as i8 {
        return 0u8;
    } else if LeadingBackslashPermissible != 0 && *dbcs_name.Buffer == b'\\' as i8 {
        dbcs_name.Buffer = dbcs_name.Buffer.add(1);
        dbcs_name.Length = dbcs_name.Length - 1;
        dbcs_name.MaximumLength = dbcs_name.MaximumLength - 1;
    }

    if PathNamePermissible != 0 {
        remaining_part.Buffer = dbcs_name.Buffer;
        remaining_part.Length = dbcs_name.Length;
        remaining_part.MaximumLength = dbcs_name.MaximumLength;

        while remaining_part.Length > 0 {
            if *remaining_part.Buffer == b'\\' as i8 {
                return 0u8;
            }

            FsRtlDissectDbcs(remaining_part, &mut first_part, &mut remaining_part);

            if FsRtlIsHpfsDbcsLegal(first_part, WildCardsPermissible, 0u8, 0u8) == 0 {
                return 0u8;
            }
        }

        return 1u8;
    }

    if dbcs_name.Length > 255 {
        return 0u8;
    }

    i = 0;
    while i < dbcs_name.Length {
        if FsRtlIsLeadDbcsCharacter(*dbcs_name.Buffer.add(i as usize) as u8) {
            if i == (dbcs_name.Length - 1) {
                return 0u8;
            }
            i += 1;
        } else if !FsRtlIsAnsiCharacterLegalHpfs(*dbcs_name.Buffer.add(i as usize) as u8, WildCardsPermissible != 0) {
            return 0u8;
        } else if (*dbcs_name.Buffer.add(i as usize) as u8 == b' ' || *dbcs_name.Buffer.add(i as usize) as u8 == b'.') && i == (dbcs_name.Length - 1) {
            return 0u8;
        }
        i += 1;
    }

    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/dbcsname.c
//  * PURPOSE:         Provides DBCS parsing and other support routines for FSDs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlDissectDbcs
//  * @implemented
//  *
//  * Dissects a given path name into first and remaining part.
//  *
//  * @param Name
//  *        ANSI string to dissect.
//  *
//  * @param FirstPart
//  *        Pointer to user supplied ANSI_STRING, that will later point
//  *        to the first part of the original name.
//  *
//  * @param RemainingPart
//  *        Pointer to user supplied ANSI_STRING, that will later point
//  *        to the remaining part of the original name.
//  *
//  * @return None
//  *
//  * @remarks Example:
//  *          Name:           \test1\test2\test3
//  *          FirstPart:      test1
//  *          RemainingPart:  test2\test3
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlDissectDbcs(IN ANSI_STRING Name,
//                  OUT PANSI_STRING FirstPart,
//                  OUT PANSI_STRING RemainingPart)
// {
//     USHORT FirstPosition, i;
//     USHORT SkipFirstSlash = 0;
//     PAGED_CODE();
//
//     /* Zero the strings before continuing */
//     RtlZeroMemory(FirstPart, sizeof(ANSI_STRING));
//     RtlZeroMemory(RemainingPart, sizeof(ANSI_STRING));
//
//     /* Just quit if the string is empty */
//     if (!Name.Length) return;
//
//     /* Find first backslash */
//     FirstPosition = Name.Length;
//     for (i = 0; i < Name.Length; i++)
//     {
//         /* First make sure the character it's not the Lead DBCS */
//         if (FsRtlIsLeadDbcsCharacter(Name.Buffer[i]))
//         {
//             i++;
//         }
//         /* If we found one... */
//         else if (Name.Buffer[i] == '\\')
//         {
//             /* If it begins string, just notice it and continue */
//             if (i == 0)
//             {
//                 SkipFirstSlash = 1;
//             }
//             else
//             {
//                 /* Else, save its position and break out of the loop */
//                 FirstPosition = i;
//                 break;
//             }
//         }
//     }
//
//     /* Set up the first result string */
//     FirstPart->Buffer = Name.Buffer + SkipFirstSlash;
//     FirstPart->Length = (FirstPosition - SkipFirstSlash);
//     FirstPart->MaximumLength = FirstPart->Length;
//
//     /* And second one, if necessary */
//     if (FirstPosition < (Name.Length))
//     {
//         RemainingPart->Buffer = Name.Buffer + FirstPosition + 1;
//         RemainingPart->Length = Name.Length - (FirstPosition + 1);
//         RemainingPart->MaximumLength = RemainingPart->Length;
//     }
// }
//
// /*++
//  * @name FsRtlDoesDbcsContainWildCards
//  * @implemented
//  *
//  * Returns TRUE if the given DbcsName contains wildcards such as *, ?,
//  * ANSI_DOS_STAR, ANSI_DOS_DOT, and ANSI_DOS_QM
//  *
//  * @param Name
//  *        The Name to check
//  *
//  * @return TRUE if there are wildcards, FALSE otherwise
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlDoesDbcsContainWildCards(IN PANSI_STRING Name)
// {
//     USHORT i;
//     PAGED_CODE();
//
//     /* Check every character */
//     for (i = 0; i < Name->Length; i++)
//     {
//         /* First make sure it's not the Lead DBCS */
//         if (FsRtlIsLeadDbcsCharacter(Name->Buffer[i]))
//         {
//             i++;
//         }
//         else if (FsRtlIsAnsiCharacterWild(Name->Buffer[i]))
//         {
//             /* Now return if it has a wildcard */
//             return TRUE;
//         }
//     }
//
//     /* We didn't return above...so none found */
//     return FALSE;
// }
//
// /*++
//  * @name FsRtlIsDbcsInExpression
//  * @implemented
//  *
//  * Check if the Name string is in the Expression string.
//  *
//  * @param Expression
//  *        The string in which we've to find Name. It can contains wildcards
//  *
//  * @param Name
//  *        The string to find. It cannot contain wildcards.
//  *
//  * @return TRUE if Name is found in Expression, FALSE otherwise
//  *
//  * @remarks
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlIsDbcsInExpression(IN PANSI_STRING Expression,
//                         IN PANSI_STRING Name)
// {
//     USHORT Offset, Position, BackTrackingPosition, OldBackTrackingPosition;
//     USHORT BackTrackingBuffer[16], OldBackTrackingBuffer[16] = {0};
//     PUSHORT BackTrackingSwap, BackTracking = BackTrackingBuffer, OldBackTracking = OldBackTrackingBuffer;
//     ULONG BackTrackingBufferSize = RTL_NUMBER_OF(BackTrackingBuffer);
//     PVOID AllocatedBuffer = NULL;
//     USHORT ExpressionPosition, NamePosition = 0, MatchingChars = 1;
//     USHORT NameChar = 0, ExpressionChar;
//     BOOLEAN EndOfName = FALSE;
//     BOOLEAN Result;
//     BOOLEAN DontSkipDot;
//     PAGED_CODE();
//
//     ASSERT(Name->Length);
//     ASSERT(Expression->Length);
//     ASSERT(!FsRtlDoesDbcsContainWildCards(Name));
//
//     /* Check if we were given strings at all */
//     if (!Name->Length || !Expression->Length)
//     {
//         /* Return TRUE if both strings are empty, otherwise FALSE */
//         if (Name->Length == 0 && Expression->Length == 0)
//             return TRUE;
//         else
//             return FALSE;
//     }
//
//     /* Check for a shortcut: just one wildcard */
//     if (Expression->Length == sizeof(CHAR))
//     {
//         if (Expression->Buffer[0] == '*')
//             return TRUE;
//     }
//
//     //ASSERT(FsRtlDoesDbcsContainWildCards(Expression));
//
//     /* Another shortcut, wildcard followed by some string */
//     if (Expression->Buffer[0] == '*')
//     {
//         /* Copy Expression to our local variable */
//         ANSI_STRING IntExpression = *Expression;
//
//         /* Skip the first char */
//         IntExpression.Buffer++;
//         IntExpression.Length -= sizeof(CHAR);
//
//         /* Continue only if the rest of the expression does NOT contain
//            any more wildcards */
//         if (!FsRtlDoesDbcsContainWildCards(&IntExpression))
//         {
//             /* Check for a degenerate case */
//             if (Name->Length < (Expression->Length - sizeof(CHAR)))
//                 return FALSE;
//
//             /* Calculate position */
//             NamePosition = (Name->Length - IntExpression.Length) / sizeof(CHAR);
//
//             /* Check whether we are breaking a two chars char (DBCS) */
//             if (NlsMbOemCodePageTag)
//             {
//                 MatchingChars = 0;
//
//                 while (MatchingChars < NamePosition)
//                 {
//                     /* Check if current char is DBCS lead char, if so, jump by two chars */
//                     MatchingChars += FsRtlIsLeadDbcsCharacter(Name->Buffer[MatchingChars]) ? 2 : 1;
//                 }
//
//                 /* If so, deny */
//                 if (MatchingChars > NamePosition)
//                     return FALSE;
//             }
//
//             /* Compare */
//             return RtlEqualMemory(IntExpression.Buffer,
//                                   (Name->Buffer + NamePosition),
//                                   IntExpression.Length);
//         }
//     }
//
//     /* Name parsing loop */
//     for (; !EndOfName; MatchingChars = BackTrackingPosition)
//     {
//         /* Reset positions */
//         OldBackTrackingPosition = BackTrackingPosition = 0;
//
//         if (NamePosition >= Name->Length)
//         {
//             EndOfName = TRUE;
//             if (MatchingChars && OldBackTracking[MatchingChars - 1] == Expression->Length * 2)
//                 break;
//         }
//         else
//         {
//             /* If lead byte present */
//             if (FsRtlIsLeadDbcsCharacter(Name->Buffer[NamePosition]))
//             {
//                 NameChar = Name->Buffer[NamePosition] +
//                            (0x100 * Name->Buffer[NamePosition + 1]);
//                 NamePosition += sizeof(USHORT);
//             }
//             else
//             {
//                 NameChar = Name->Buffer[NamePosition];
//                 NamePosition += sizeof(UCHAR);
//             }
//         }
//
//         while (MatchingChars > OldBackTrackingPosition)
//         {
//             ExpressionPosition = (OldBackTracking[OldBackTrackingPosition++] + 1) / 2;
//
//             /* Expression parsing loop */
//             for (Offset = 0; ExpressionPosition < Expression->Length; )
//             {
//                 ExpressionPosition += Offset;
//
//                 if (ExpressionPosition == Expression->Length)
//                 {
//                     BackTracking[BackTrackingPosition++] = Expression->Length * 2;
//                     break;
//                 }
//
//                 /* If buffer too small */
//                 if (BackTrackingPosition > BackTrackingBufferSize - 3)
//                 {
//                     /* We should only ever get here once! */
//                     ASSERT(AllocatedBuffer == NULL);
//                     ASSERT((BackTracking == BackTrackingBuffer) || (BackTracking == OldBackTrackingBuffer));
//                     ASSERT((OldBackTracking == BackTrackingBuffer) || (OldBackTracking == OldBackTrackingBuffer));
//
//                     /* Calculate buffer size */
//                     BackTrackingBufferSize = Expression->Length * 2 + 1;
//
//                     /* Allocate memory for both back-tracking buffers */
//                     AllocatedBuffer = ExAllocatePoolWithTag(PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                                             2 * BackTrackingBufferSize * sizeof(USHORT),
//                                                             'nrSF');
//                     if (AllocatedBuffer == NULL)
//                     {
//                         DPRINT1("Failed to allocate BackTracking buffer. BackTrackingBufferSize = =x%lx\n",
//                                 BackTrackingBufferSize);
//                         Result = FALSE;
//                         goto Exit;
//                     }
//
//                     /* Copy BackTracking content. Note that it can point to either BackTrackingBuffer or OldBackTrackingBuffer */
//                     RtlCopyMemory(AllocatedBuffer,
//                                   BackTracking,
//                                   RTL_NUMBER_OF(BackTrackingBuffer) * sizeof(USHORT));
//
//                     /* Place current Backtracking is at the start of the new buffer */
//                     BackTracking = AllocatedBuffer;
//
//                     /* Copy OldBackTracking content */
//                     RtlCopyMemory(&BackTracking[BackTrackingBufferSize],
//                                   OldBackTracking,
//                                   RTL_NUMBER_OF(OldBackTrackingBuffer) * sizeof(USHORT));
//
//                     /* Place current OldBackTracking after current BackTracking in the buffer */
//                     OldBackTracking = &BackTracking[BackTrackingBufferSize];
//                 }
//
//                 /* If lead byte present */
//                 if (FsRtlIsLeadDbcsCharacter(Expression->Buffer[ExpressionPosition]))
//                 {
//                     ExpressionChar = Expression->Buffer[ExpressionPosition] +
//                                      (0x100 * Expression->Buffer[ExpressionPosition + 1]);
//                     Offset = sizeof(USHORT);
//                 }
//                 else
//                 {
//                     ExpressionChar = Expression->Buffer[ExpressionPosition];
//                     Offset = sizeof(UCHAR);
//                 }
//
//                 /* Basic check to test if chars are equal */
//                 if (ExpressionChar == NameChar && !EndOfName)
//                 {
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + Offset) * 2;
//                 }
//                 /* Check cases that eat one char */
//                 else if (ExpressionChar == '?' && !EndOfName)
//                 {
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + Offset) * 2;
//                 }
//                 /* Test star */
//                 else if (ExpressionChar == '*')
//                 {
//                     BackTracking[BackTrackingPosition++] = ExpressionPosition * 2;
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition * 2) + 1;
//                     continue;
//                 }
//                 /* Check DOS_STAR */
//                 else if (ExpressionChar == ANSI_DOS_STAR)
//                 {
//                     /* Look for last dot */
//                     DontSkipDot = TRUE;
//                     if (!EndOfName && NameChar == '.')
//                     {
//                         for (Position = NamePosition; Position < Name->Length; )
//                         {
//                             /* If lead byte not present */
//                             if (!FsRtlIsLeadDbcsCharacter(Name->Buffer[Position]))
//                             {
//                                 if (Name->Buffer[Position] == '.')
//                                 {
//                                     DontSkipDot = FALSE;
//                                     break;
//                                 }
//
//                                 Position += sizeof(UCHAR);
//                             }
//                             else
//                             {
//                                 Position += sizeof(USHORT);
//                             }
//                         }
//                     }
//
//                     if (EndOfName || NameChar != '.' || !DontSkipDot)
//                         BackTracking[BackTrackingPosition++] = ExpressionPosition * 2;
//
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition * 2) + 1;
//                     continue;
//                 }
//                 /* Check DOS_DOT */
//                 else if (ExpressionChar == ANSI_DOS_DOT)
//                 {
//                     if (EndOfName) continue;
//
//                     if (NameChar == '.')
//                         BackTracking[BackTrackingPosition++] = (ExpressionPosition + Offset) * 2;
//                 }
//                 /* Check DOS_QM */
//                 else if (ExpressionChar == ANSI_DOS_QM)
//                 {
//                     if (EndOfName || NameChar == '.') continue;
//
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + Offset) * 2;
//                 }
//
//                 /* Leave from loop */
//                 break;
//             }
//
//             for (Position = 0; MatchingChars > OldBackTrackingPosition && Position < BackTrackingPosition; Position++)
//             {
//                 while (MatchingChars > OldBackTrackingPosition &&
//                        BackTracking[Position] > OldBackTracking[OldBackTrackingPosition])
//                 {
//                     ++OldBackTrackingPosition;
//                 }
//             }
//         }
//
//         /* Swap pointers */
//         BackTrackingSwap = BackTracking;
//         BackTracking = OldBackTracking;
//         OldBackTracking = BackTrackingSwap;
//     }
//
//     /* Store result value */
//     Result = MatchingChars && (OldBackTracking[MatchingChars - 1] == Expression->Length * 2);
//
// Exit:
//
//     /* Frees the memory if necessary */
//     if (AllocatedBuffer != NULL)
//     {
//         ExFreePoolWithTag(AllocatedBuffer, 'nrSF');
//     }
//
//     return Result;
// }
//
// /*++
//  * @name FsRtlIsFatDbcsLegal
//  * @implemented
//  *
//  * Returns TRUE if the given DbcsName is a valid FAT filename (in 8.3)
//  *
//  * @param DbcsName
//  *        The filename to check. It can also contains pathname.
//  *
//  * @param WildCardsPermissible
//  *        If this is set to FALSE and if filename contains wildcard, the function
//  *        will fail
//  *
//  * @param PathNamePermissible
//  *        If this is set to FALSE and if the filename comes with a pathname, the
//  *        function will fail
//  *
//  * @param LeadingBackslashPermissible
//  *        If this is set to FALSE and if the filename starts with a backslash, the
//  *        function will fail
//  *
//  * @return TRUE if the DbcsName is legal, FALSE otherwise
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlIsFatDbcsLegal(IN ANSI_STRING DbcsName,
//                     IN BOOLEAN WildCardsPermissible,
//                     IN BOOLEAN PathNamePermissible,
//                     IN BOOLEAN LeadingBackslashPermissible)
// {
//     ANSI_STRING FirstPart, RemainingPart;
//     BOOLEAN LastDot;
//     USHORT i;
//     PAGED_CODE();
//
//     /* Just quit if the string is empty */
//     if (!DbcsName.Length)
//         return FALSE;
//
//     /* Accept special filename if wildcards are allowed */
//     if (WildCardsPermissible && (DbcsName.Length == 1 || DbcsName.Length == 2) && DbcsName.Buffer[0] == '.')
//     {
//         if (DbcsName.Length == 2)
//         {
//             if (DbcsName.Buffer[1] == '.')
//                 return TRUE;
//         }
//         else
//         {
//             return TRUE;
//         }
//     }
//
//     /* DbcsName wasn't supposed to be started with \ */
//     if (!LeadingBackslashPermissible && DbcsName.Buffer[0] == '\\')
//         return FALSE;
//     /* DbcsName was allowed to be started with \, but now, remove it */
//     else if (LeadingBackslashPermissible && DbcsName.Buffer[0] == '\\')
//     {
//         DbcsName.Buffer = DbcsName.Buffer + 1;
//         DbcsName.Length = DbcsName.Length - 1;
//         DbcsName.MaximumLength = DbcsName.MaximumLength - 1;
//     }
//
//     if (PathNamePermissible)
//     {
//         /* We copy the buffer for FsRtlDissectDbcs call */
//         RemainingPart.Buffer = DbcsName.Buffer;
//         RemainingPart.Length = DbcsName.Length;
//         RemainingPart.MaximumLength = DbcsName.MaximumLength;
//
//         while (RemainingPart.Length > 0)
//         {
//             if (RemainingPart.Buffer[0] == '\\')
//                 return FALSE;
//
//             /* Call once again our dissect function */
//             FsRtlDissectDbcs(RemainingPart, &FirstPart, &RemainingPart);
//
//             if (!FsRtlIsFatDbcsLegal(FirstPart,
//                                      WildCardsPermissible,
//                                      FALSE,
//                                      FALSE))
//             {
//                 return FALSE;
//             }
//         }
//
//         return TRUE;
//     }
//
//     if (WildCardsPermissible && FsRtlDoesDbcsContainWildCards(&DbcsName))
//     {
//         for (i = 0; i < DbcsName.Length; i++)
//         {
//             /* First make sure the character it's not the Lead DBCS */
//             if (FsRtlIsLeadDbcsCharacter(DbcsName.Buffer[i]))
//             {
//                 i++;
//             }
//             /* Then check for bad characters */
//             else if (!FsRtlIsAnsiCharacterLegalFat(DbcsName.Buffer[i], TRUE))
//             {
//                 return FALSE;
//             }
//         }
//
//         return TRUE;
//     }
//
//     /* Filename must be 8.3 filename */
//     if (DbcsName.Length > 12)
//         return FALSE;
//
//     /* Reset dots count */
//     LastDot = FALSE;
//
//     for (i = 0; i < DbcsName.Length; i++)
//     {
//         /* First make sure the character it's not the Lead DBCS */
//         if (FsRtlIsLeadDbcsCharacter(DbcsName.Buffer[i]))
//         {
//             if (!LastDot && (i >= 7))
//                 return FALSE;
//
//             if (i == (DbcsName.Length - 1))
//                 return FALSE;
//
//             i++;
//             continue;
//         }
//         /* Then check for bad characters */
//         else if (!FsRtlIsAnsiCharacterLegalFat(DbcsName.Buffer[i], WildCardsPermissible \!= 0))
//         {
//             return FALSE;
//         }
//         else if (DbcsName.Buffer[i] == '.')
//         {
//             /* Filename can only contain one dot */
//             if (LastDot)
//                 return FALSE;
//
//             LastDot = TRUE;
//
//             /* We mustn't have spaces before dot or at the end of the filename
//              * and no dot at the beginning of the filename */
//             if (i == (DbcsName.Length - 1) || i == 0)
//                 return FALSE;
//
//             /* Filename must be 8.3 filename and not 3.8 filename */
//             if ((DbcsName.Length - 1) - i > 3)
//                 return FALSE;
//
//             if ((i > 0) && DbcsName.Buffer[i - 1] == ' ')
//                 return FALSE;
//         }
//         /* Filename mustn't finish with a space */
//         else if (DbcsName.Buffer[i] == ' ' && i == (DbcsName.Length - 1))
//         {
//             return FALSE;
//         }
//
//         if (!LastDot && (i >= 8))
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*++
//  * @name FsRtlIsHpfsDbcsLegal
//  * @implemented
//  *
//  * Returns TRUE if the given DbcsName is a valid HPFS filename
//  *
//  * @param DbcsName
//  *        The filename to check. It can also contains pathname.
//  *
//  * @param WildCardsPermissible
//  *        If this is set to FALSE and if filename contains wildcard, the function
//  *        will fail
//  *
//  * @param PathNamePermissible
//  *        If this is set to FALSE and if the filename comes with a pathname, the
//  *        function will fail
//  *
//  * @param LeadingBackslashPermissible
//  *        If this is set to FALSE and if the filename starts with a backslash, the
//  *        function will fail
//  *
//  * @return TRUE if the DbcsName is legal, FALSE otherwise
//  *
//  * @remarks None
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlIsHpfsDbcsLegal(IN ANSI_STRING DbcsName,
//                      IN BOOLEAN WildCardsPermissible,
//                      IN BOOLEAN PathNamePermissible,
//                      IN BOOLEAN LeadingBackslashPermissible)
// {
//     ANSI_STRING FirstPart, RemainingPart;
//     USHORT i;
//     PAGED_CODE();
//
//     /* Just quit if the string is empty */
//     if (!DbcsName.Length)
//         return FALSE;
//
//     /* Accept special filename if wildcards are allowed */
//     if (WildCardsPermissible && (DbcsName.Length == 1 || DbcsName.Length == 2) && DbcsName.Buffer[0] == '.')
//     {
//         if (DbcsName.Length == 2)
//         {
//             if (DbcsName.Buffer[1] == '.')
//                 return TRUE;
//         }
//         else
//         {
//             return TRUE;
//         }
//     }
//
//     /* DbcsName wasn't supposed to be started with \ */
//     if (!LeadingBackslashPermissible && DbcsName.Buffer[0] == '\\')
//         return FALSE;
//     /* DbcsName was allowed to be started with \, but now, remove it */
//     else if (LeadingBackslashPermissible && DbcsName.Buffer[0] == '\\')
//     {
//         DbcsName.Buffer = DbcsName.Buffer + 1;
//         DbcsName.Length = DbcsName.Length - 1;
//         DbcsName.MaximumLength = DbcsName.MaximumLength - 1;
//     }
//
//     if (PathNamePermissible)
//     {
//         /* We copy the buffer for FsRtlDissectDbcs call */
//         RemainingPart.Buffer = DbcsName.Buffer;
//         RemainingPart.Length = DbcsName.Length;
//         RemainingPart.MaximumLength = DbcsName.MaximumLength;
//
//         while (RemainingPart.Length > 0)
//         {
//             if (RemainingPart.Buffer[0] == '\\')
//                 return FALSE;
//
//             /* Call once again our dissect function */
//             FsRtlDissectDbcs(RemainingPart, &FirstPart, &RemainingPart);
//
//             if (!FsRtlIsHpfsDbcsLegal(FirstPart,
//                                       WildCardsPermissible,
//                                       FALSE,
//                                       FALSE))
//             {
//                 return FALSE;
//             }
//         }
//
//         return TRUE;
//     }
//
//     if (DbcsName.Length > 255)
//         return FALSE;
//
//     for (i = 0; i < DbcsName.Length; i++)
//     {
//         /* First make sure the character it's not the Lead DBCS */
//         if (FsRtlIsLeadDbcsCharacter(DbcsName.Buffer[i]))
//         {
//             if (i == (DbcsName.Length - 1))
//                 return FALSE;
//             i++;
//         }
//         /* Then check for bad characters */
//         else if (!FsRtlIsAnsiCharacterLegalHpfs(DbcsName.Buffer[i], WildCardsPermissible \!= 0))
//         {
//             return FALSE;
//         }
//         /* Filename mustn't finish with a space or a dot */
//         else if ((DbcsName.Buffer[i] == ' ' || DbcsName.Buffer[i] == '.') && i == (DbcsName.Length - 1))
//         {
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
