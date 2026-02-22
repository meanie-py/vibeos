use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, PANSI_STRING};
use core::ffi::c_void;

extern "system" {
    fn FsRtlIsLeadDbcsCharacter(Character: u8) -> u8;
    fn FsRtlIsAnsiCharacterWild(Character: u8) -> u8;
    fn FsRtlIsAnsiCharacterLegalHpfs(Character: u8, WildCardsPermissible: u8) -> u8;
    fn RtlEqualMemory(Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: usize) -> u8;
    fn FsRtlIsAnsiCharacterLegalFat(Character: u8, WildCardsPermissible: u8) -> u8;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDissectDbcs(
    Name: ANSI_STRING,
    FirstPart: PANSI_STRING,
    RemainingPart: PANSI_STRING,
) {
    let mut FirstPosition: u16;
    let mut i: u16;
    let mut SkipFirstSlash: u16 = 0;

    RtlZeroMemory(FirstPart as *mut core::ffi::c_void, core::mem::size_of::<ANSI_STRING>());
    RtlZeroMemory(RemainingPart as *mut core::ffi::c_void, core::mem::size_of::<ANSI_STRING>());

    if Name.Length == 0 {
        return;
    }

    FirstPosition = Name.Length;
    i = 0;
    while i < Name.Length {
        if FsRtlIsLeadDbcsCharacter(*Name.Buffer.offset(i as isize)) != 0 {
            i += 1;
        } else if *Name.Buffer.offset(i as isize) == b'\\' {
            if i == 0 {
                SkipFirstSlash = 1;
            } else {
                FirstPosition = i;
                break;
            }
        }
        i += 1;
    }

    (*FirstPart).Buffer = Name.Buffer.offset(SkipFirstSlash as isize);
    (*FirstPart).Length = FirstPosition - SkipFirstSlash;
    (*FirstPart).MaximumLength = (*FirstPart).Length;

    if FirstPosition < Name.Length {
        (*RemainingPart).Buffer = Name.Buffer.offset((FirstPosition + 1) as isize);
        (*RemainingPart).Length = Name.Length - (FirstPosition + 1);
        (*RemainingPart).MaximumLength = (*RemainingPart).Length;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDoesDbcsContainWildCards(Name: PANSI_STRING) -> u8 {

    let mut i: u16 = 0;

    while i < (*Name).Length {
        if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.offset(i as isize)) != 0 {
            i += 1;
        } else if FsRtlIsAnsiCharacterWild(*(*Name).Buffer.offset(i as isize)) != 0 {
            return 1u8;
        }
        i += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsDbcsInExpression(
    Expression: PANSI_STRING,
    Name: PANSI_STRING
) -> u8 {

    let mut BackTrackingBuffer: [u16; 16] = [0; 16];
    let mut OldBackTrackingBuffer: [u16; 16] = [0; 16];
    let mut BackTracking: *mut u16 = BackTrackingBuffer.as_mut_ptr();
    let mut OldBackTracking: *mut u16 = OldBackTrackingBuffer.as_mut_ptr();
    let mut BackTrackingBufferSize: u32 = 16;
    let mut AllocatedBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut NamePosition: u16 = 0;
    let mut MatchingChars: u16 = 1;
    let mut NameChar: u16 = 0;
    let mut EndOfName: u8 = 0;
    let Result: u8;

    if (*Name).Length == 0 || (*Expression).Length == 0 {
        if (*Name).Length == 0 && (*Expression).Length == 0 {
            return 1u8;
        } else {
            return 0u8;
        }
    }

    if (*Expression).Length == core::mem::size_of::<u8>() as u16 {
        if *(*Expression).Buffer.offset(0) == b'*' {
            return 1u8;
        }
    }

    if *(*Expression).Buffer.offset(0) == b'*' {
        let mut IntExpression: ANSI_STRING = *Expression;
        IntExpression.Buffer = IntExpression.Buffer.offset(1);
        IntExpression.Length -= core::mem::size_of::<u8>() as u16;

        if FsRtlDoesDbcsContainWildCards(core::ptr::addr_of_mut!(IntExpression)) == 0 {
            if (*Name).Length < ((*Expression).Length - core::mem::size_of::<u8>() as u16) {
                return 0u8;
            }

            NamePosition = ((*Name).Length - IntExpression.Length) / core::mem::size_of::<u8>() as u16;

            if NlsMbOemCodePageTag != 0 {
                MatchingChars = 0;

                while MatchingChars < NamePosition {
                    MatchingChars += if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.offset(MatchingChars as isize)) != 0 { 2 } else { 1 };
                }

                if MatchingChars > NamePosition {
                    return 0u8;
                }
            }

            return RtlEqualMemory(
                IntExpression.Buffer as *const core::ffi::c_void,
                (*Name).Buffer.offset(NamePosition as isize) as *const core::ffi::c_void,
                IntExpression.Length as usize
            );
        }
    }

    'name_loop: loop {
        let mut OldBackTrackingPosition: u16 = 0;
        let mut BackTrackingPosition: u16 = 0;

        if NamePosition >= (*Name).Length {
            EndOfName = 1;
            if MatchingChars != 0 && *OldBackTracking.offset((MatchingChars - 1) as isize) == (*Expression).Length * 2 {
                break 'name_loop;
            }
        } else {
            if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.offset(NamePosition as isize)) != 0 {
                NameChar = *(*Name).Buffer.offset(NamePosition as isize) as u16 +
                           (0x100u16 * (*(*Name).Buffer.offset((NamePosition + 1) as isize) as u16));
                NamePosition += core::mem::size_of::<u16>() as u16;
            } else {
                NameChar = *(*Name).Buffer.offset(NamePosition as isize) as u16;
                NamePosition += core::mem::size_of::<u8>() as u16;
            }
        }

        while MatchingChars > OldBackTrackingPosition {
            let mut ExpressionPosition: u16 = (*OldBackTracking.offset(OldBackTrackingPosition as isize) + 1) / 2;
            OldBackTrackingPosition += 1;

            let mut Offset: u16 = 0;
            'expression_loop: loop {
                ExpressionPosition += Offset;

                if ExpressionPosition == (*Expression).Length {
                    *BackTracking.offset(BackTrackingPosition as isize) = (*Expression).Length * 2;
                    BackTrackingPosition += 1;
                    break 'expression_loop;
                }

                if BackTrackingPosition > BackTrackingBufferSize as u16 - 3 {
                    BackTrackingBufferSize = (*Expression).Length as u32 * 2 + 1;

                    AllocatedBuffer = ExAllocatePoolWithTag(
                        PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
                        2 * BackTrackingBufferSize as usize * core::mem::size_of::<u16>(),
                        0x6e725346u32
                    );
                    if AllocatedBuffer.is_null() {
                        Result = 0u8;
                        break 'name_loop;
                    }

                    core::ptr::copy_nonoverlapping(
                        BackTracking,
                        AllocatedBuffer as *mut u16,
                        16
                    );

                    BackTracking = AllocatedBuffer as *mut u16;

                    core::ptr::copy_nonoverlapping(
                        OldBackTracking,
                        (BackTracking as *mut u16).offset(BackTrackingBufferSize as isize),
                        16
                    );

                    OldBackTracking = (BackTracking as *mut u16).offset(BackTrackingBufferSize as isize);
                }

                let ExpressionChar: u16;
                if FsRtlIsLeadDbcsCharacter(*(*Expression).Buffer.offset(ExpressionPosition as isize)) != 0 {
                    ExpressionChar = *(*Expression).Buffer.offset(ExpressionPosition as isize) as u16 +
                                    (0x100u16 * (*(*Expression).Buffer.offset((ExpressionPosition + 1) as isize) as u16));
                    Offset = core::mem::size_of::<u16>() as u16;
                } else {
                    ExpressionChar = *(*Expression).Buffer.offset(ExpressionPosition as isize) as u16;
                    Offset = core::mem::size_of::<u8>() as u16;
                }

                if ExpressionChar == NameChar && EndOfName == 0 {
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + Offset) * 2;
                    BackTrackingPosition += 1;
                } else if ExpressionChar == b'?' as u16 && EndOfName == 0 {
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + Offset) * 2;
                    BackTrackingPosition += 1;
                } else if ExpressionChar == b'*' as u16 {
                    *BackTracking.offset(BackTrackingPosition as isize) = ExpressionPosition * 2;
                    BackTrackingPosition += 1;
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition * 2) + 1;
                    BackTrackingPosition += 1;
                    continue 'expression_loop;
                } else if ExpressionChar == 0x3Cu16 {
                    let mut DontSkipDot: u8 = 1;
                    if EndOfName == 0 && NameChar == b'.' as u16 {
                        let mut Position: u16 = NamePosition;
                        while Position < (*Name).Length {
                            if FsRtlIsLeadDbcsCharacter(*(*Name).Buffer.offset(Position as isize)) == 0 {
                                if *(*Name).Buffer.offset(Position as isize) == b'.' {
                                    DontSkipDot = 0;
                                    break;
                                }
                                Position += core::mem::size_of::<u8>() as u16;
                            } else {
                                Position += core::mem::size_of::<u16>() as u16;
                            }
                        }
                    }

                    if EndOfName != 0 || NameChar != b'.' as u16 || DontSkipDot == 0 {
                        *BackTracking.offset(BackTrackingPosition as isize) = ExpressionPosition * 2;
                        BackTrackingPosition += 1;
                    }

                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition * 2) + 1;
                    BackTrackingPosition += 1;
                    continue 'expression_loop;
                } else if ExpressionChar == 0x3Eu16 {
                    if EndOfName == 0 {
                        if NameChar == b'.' as u16 {
                            *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + Offset) * 2;
                            BackTrackingPosition += 1;
                        }
                    }
                } else if ExpressionChar == 0x3Fu16 {
                    if EndOfName == 0 && NameChar != b'.' as u16 {
                        *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + Offset) * 2;
                        BackTrackingPosition += 1;
                    }
                }

                break 'expression_loop;
            }

            let mut Position: u16 = 0;
            while MatchingChars > OldBackTrackingPosition && Position < BackTrackingPosition {
                while MatchingChars > OldBackTrackingPosition &&
                      *BackTracking.offset(Position as isize) > *OldBackTracking.offset(OldBackTrackingPosition as isize) {
                    OldBackTrackingPosition += 1;
                }
                Position += 1;
            }
        }

        let BackTrackingSwap = BackTracking;
        BackTracking = OldBackTracking;
        OldBackTracking = BackTrackingSwap;

        MatchingChars = BackTrackingPosition;

        if EndOfName != 0 {
            break 'name_loop;
        }
    }

    Result = if MatchingChars != 0 && *OldBackTracking.offset((MatchingChars - 1) as isize) == (*Expression).Length * 2 { 1u8 } else { 0u8 };

    if !AllocatedBuffer.is_null() {
        ExFreePoolWithTag(AllocatedBuffer, 0x6e725346u32);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsFatDbcsLegal(
    mut DbcsName: ANSI_STRING,
    WildCardsPermissible: u8,
    PathNamePermissible: u8,
    LeadingBackslashPermissible: u8,
) -> u8 {

    let mut FirstPart: ANSI_STRING = core::mem::zeroed();
    let mut RemainingPart: ANSI_STRING;
    let mut LastDot: u8;
    let mut i: u16;

    if DbcsName.Length == 0 {
        return 0u8;
    }

    if WildCardsPermissible != 0 && (DbcsName.Length == 1 || DbcsName.Length == 2) && *DbcsName.Buffer.offset(0) == b'.' {
        if DbcsName.Length == 2 {
            if *DbcsName.Buffer.offset(1) == b'.' {
                return 1u8;
            }
        } else {
            return 1u8;
        }
    }

    if LeadingBackslashPermissible == 0 && *DbcsName.Buffer.offset(0) == b'\\' {
        return 0u8;
    } else if LeadingBackslashPermissible != 0 && *DbcsName.Buffer.offset(0) == b'\\' {
        DbcsName.Buffer = DbcsName.Buffer.offset(1);
        DbcsName.Length = DbcsName.Length - 1;
        DbcsName.MaximumLength = DbcsName.MaximumLength - 1;
    }

    if PathNamePermissible != 0 {
        RemainingPart.Buffer = DbcsName.Buffer;
        RemainingPart.Length = DbcsName.Length;
        RemainingPart.MaximumLength = DbcsName.MaximumLength;

        while RemainingPart.Length > 0 {
            if *RemainingPart.Buffer.offset(0) == b'\\' {
                return 0u8;
            }

            FsRtlDissectDbcs(RemainingPart, &mut FirstPart, &mut RemainingPart);

            if FsRtlIsFatDbcsLegal(FirstPart, WildCardsPermissible, 0u8, 0u8) == 0 {
                return 0u8;
            }
        }

        return 1u8;
    }

    if WildCardsPermissible != 0 && FsRtlDoesDbcsContainWildCards(&DbcsName) != 0 {
        i = 0;
        while i < DbcsName.Length {
            if FsRtlIsLeadDbcsCharacter(*DbcsName.Buffer.offset(i as isize)) != 0 {
                i += 1;
            } else if FsRtlIsAnsiCharacterLegalFat(*DbcsName.Buffer.offset(i as isize), 1u8) == 0 {
                return 0u8;
            }
            i += 1;
        }

        return 1u8;
    }

    if DbcsName.Length > 12 {
        return 0u8;
    }

    LastDot = 0u8;

    i = 0;
    while i < DbcsName.Length {
        if FsRtlIsLeadDbcsCharacter(*DbcsName.Buffer.offset(i as isize)) != 0 {
            if LastDot == 0 && i >= 7 {
                return 0u8;
            }

            if i == DbcsName.Length - 1 {
                return 0u8;
            }

            i += 1;
            i += 1;
            continue;
        } else if FsRtlIsAnsiCharacterLegalFat(*DbcsName.Buffer.offset(i as isize), WildCardsPermissible) == 0 {
            return 0u8;
        } else if *DbcsName.Buffer.offset(i as isize) == b'.' {
            if LastDot != 0 {
                return 0u8;
            }

            LastDot = 1u8;

            if i == DbcsName.Length - 1 || i == 0 {
                return 0u8;
            }

            if (DbcsName.Length - 1) - i > 3 {
                return 0u8;
            }

            if i > 0 && *DbcsName.Buffer.offset((i - 1) as isize) == b' ' {
                return 0u8;
            }
        } else if *DbcsName.Buffer.offset(i as isize) == b' ' && i == DbcsName.Length - 1 {
            return 0u8;
        }

        if LastDot == 0 && i >= 8 {
            return 0u8;
        }

        i += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsHpfsDbcsLegal(
    mut DbcsName: ANSI_STRING,
    WildCardsPermissible: u8,
    PathNamePermissible: u8,
    LeadingBackslashPermissible: u8,
) -> u8 {
    let mut FirstPart: ANSI_STRING = core::mem::zeroed();
    let mut RemainingPart: ANSI_STRING = core::mem::zeroed();
    let mut i: u16;

    if DbcsName.Length == 0 {
        return 0u8;
    }

    if WildCardsPermissible != 0 && (DbcsName.Length == 1 || DbcsName.Length == 2) && *DbcsName.Buffer == b'.' {
        if DbcsName.Length == 2 {
            if *DbcsName.Buffer.offset(1) == b'.' {
                return 1u8;
            }
        } else {
            return 1u8;
        }
    }

    if LeadingBackslashPermissible == 0 && *DbcsName.Buffer == b'\\' {
        return 0u8;
    } else if LeadingBackslashPermissible != 0 && *DbcsName.Buffer == b'\\' {
        DbcsName.Buffer = DbcsName.Buffer.offset(1);
        DbcsName.Length = DbcsName.Length - 1;
        DbcsName.MaximumLength = DbcsName.MaximumLength - 1;
    }

    if PathNamePermissible != 0 {
        RemainingPart.Buffer = DbcsName.Buffer;
        RemainingPart.Length = DbcsName.Length;
        RemainingPart.MaximumLength = DbcsName.MaximumLength;

        while RemainingPart.Length > 0 {
            if *RemainingPart.Buffer == b'\\' {
                return 0u8;
            }

            FsRtlDissectDbcs(RemainingPart, &mut FirstPart, &mut RemainingPart);

            if FsRtlIsHpfsDbcsLegal(FirstPart, WildCardsPermissible, 0u8, 0u8) == 0 {
                return 0u8;
            }
        }

        return 1u8;
    }

    if DbcsName.Length > 255 {
        return 0u8;
    }

    i = 0;
    while i < DbcsName.Length {
        if FsRtlIsLeadDbcsCharacter(*DbcsName.Buffer.offset(i as isize)) != 0 {
            if i == (DbcsName.Length - 1) {
                return 0u8;
            }
            i += 1;
        } else if FsRtlIsAnsiCharacterLegalHpfs(*DbcsName.Buffer.offset(i as isize), WildCardsPermissible) == 0 {
            return 0u8;
        } else if (*DbcsName.Buffer.offset(i as isize) == b' ' || *DbcsName.Buffer.offset(i as isize) == b'.') && i == (DbcsName.Length - 1) {
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
//         else if (!FsRtlIsAnsiCharacterLegalFat(DbcsName.Buffer[i], WildCardsPermissible))
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
//         else if (!FsRtlIsAnsiCharacterLegalHpfs(DbcsName.Buffer[i], WildCardsPermissible))
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
