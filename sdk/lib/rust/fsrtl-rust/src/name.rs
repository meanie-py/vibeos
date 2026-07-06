use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    PCUNICODE_STRING, PCWCH, PUNICODE_STRING,
    RtlFreeUnicodeString, RtlUpcaseUnicodeString,
    _UNICODE_STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[inline(always)]
unsafe fn RtlEqualMemory(a: PVOID, b: PVOID, len: usize) -> bool {
    ntkrnl_sys::ddk::RtlCompareMemory(a, b, len as _) == len as _
}

const DOS_STAR: u16 = b'<' as u16;
const DOS_QM: u16 = b'>' as u16;
const DOS_DOT: u16 = b'"' as u16;

unsafe fn FsRtlIsUnicodeCharacterWild(c: u16) -> BOOLEAN {
    if c == b'*' as u16 || c == b'?' as u16 || c == DOS_STAR || c == DOS_QM || c == DOS_DOT {
        1u8
    } else {
        0u8
    }
}

unsafe fn FsRtlIsNameInExpressionPrivate(
    Expression: PUNICODE_STRING,
    Name: PUNICODE_STRING,
    IgnoreCase: BOOLEAN,
    UpcaseTable: PCWCH,
) -> BOOLEAN {
    let mut BackTrackingBuffer: [USHORT; 16] = [0; 16];
    let mut OldBackTrackingBuffer: [USHORT; 16] = [0; 16];
    let mut BackTracking: *mut USHORT = BackTrackingBuffer.as_mut_ptr();
    let mut OldBackTracking: *mut USHORT = OldBackTrackingBuffer.as_mut_ptr();
    let mut BackTrackingBufferSize: ULONG = 16;
    let mut AllocatedBuffer: PVOID = core::ptr::null_mut();
    let mut NamePosition: USHORT = 0;
    let mut MatchingChars: USHORT = 1;
    let mut EndOfName: BOOLEAN = 0u8;
    let mut Result: BOOLEAN;
    let mut BackTrackingPosition: USHORT;
    let mut OldBackTrackingPosition: USHORT;

    if (*Name).Length == 0 || (*Expression).Length == 0 {
        if (*Name).Length == 0 && (*Expression).Length == 0 {
            return 1u8;
        } else {
            return 0u8;
        }
    }

    if (*Expression).Length == core::mem::size_of::<WCHAR>() as USHORT {
        if (*(*Expression).Buffer.offset(0)) == b'*' as u16 {
            return 1u8;
        }
    }

    if (*(*Expression).Buffer.offset(0)) == b'*' as u16 {
        let mut IntExpression = *Expression;
        IntExpression.Buffer = IntExpression.Buffer.offset(1);
        IntExpression.Length -= core::mem::size_of::<WCHAR>() as USHORT;

        if FsRtlDoesNameContainWildCards(&mut IntExpression) == 0 {
            if (*Name).Length < ((*Expression).Length - core::mem::size_of::<WCHAR>() as USHORT) {
                return 0u8;
            }

            NamePosition = ((*Name).Length - IntExpression.Length) / core::mem::size_of::<WCHAR>() as USHORT;

            if IgnoreCase == 0 {
                return RtlEqualMemory(
                    IntExpression.Buffer as PVOID,
                    (*Name).Buffer.offset(NamePosition as isize) as PVOID,
                    IntExpression.Length as usize,
                ) as BOOLEAN;
            } else {
                let mut ExpressionPosition: USHORT = 0;
                while ExpressionPosition < (IntExpression.Length / core::mem::size_of::<WCHAR>() as USHORT) {
                    let name_char = *(*Name).Buffer.offset((NamePosition + ExpressionPosition) as isize);
                    let expr_char = *IntExpression.Buffer.offset(ExpressionPosition as isize);
                    if *UpcaseTable.offset(name_char as isize) != expr_char {
                        return 0u8;
                    }
                    ExpressionPosition += 1;
                }
                return 1u8;
            }
        }
    }

    NamePosition = 0;
    while EndOfName == 0 {
        OldBackTrackingPosition = 0;
        BackTrackingPosition = 0;

        if NamePosition >= (*Name).Length / core::mem::size_of::<WCHAR>() as USHORT {
            EndOfName = 1u8;
            if MatchingChars != 0 && *OldBackTracking.offset((MatchingChars - 1) as isize) == (*Expression).Length * 2 {
                break;
            }
        }

        while MatchingChars > OldBackTrackingPosition {
            let mut ExpressionPosition: USHORT = (*OldBackTracking.offset(OldBackTrackingPosition as isize) + 1) / 2;
            OldBackTrackingPosition += 1;

            let mut Offset: USHORT = 0;
            while ExpressionPosition < (*Expression).Length {
                ExpressionPosition += Offset;

                if ExpressionPosition == (*Expression).Length {
                    *BackTracking.offset(BackTrackingPosition as isize) = (*Expression).Length * 2;
                    BackTrackingPosition += 1;
                    break;
                }

                if BackTrackingPosition > BackTrackingBufferSize as USHORT - 3 {
                    BackTrackingBufferSize = ((*Expression).Length / core::mem::size_of::<WCHAR>() as USHORT) as ULONG * 2 + 1;

                    AllocatedBuffer = ExAllocatePoolWithTag(
                        PagedPool | POOL_RAISE_IF_ALLOCATION_FAILURE,
                        (2 * BackTrackingBufferSize * core::mem::size_of::<USHORT>() as ULONG) as usize,
                        u32::from_le_bytes([b'n', b'r', b'S', b'F']),
                    );
                    if AllocatedBuffer.is_null() {
                        Result = 0u8;
                        break;
                    }

                    core::ptr::copy_nonoverlapping(
                        BackTracking as *const u8,
                        AllocatedBuffer as *mut u8,
                        16 * core::mem::size_of::<USHORT>(),
                    );

                    BackTracking = AllocatedBuffer as *mut USHORT;

                    core::ptr::copy_nonoverlapping(
                        OldBackTracking as *const u8,
                        (BackTracking as usize + BackTrackingBufferSize as usize * core::mem::size_of::<USHORT>()) as *mut u8,
                        16 * core::mem::size_of::<USHORT>(),
                    );

                    OldBackTracking = BackTracking.offset(BackTrackingBufferSize as isize);
                }

                let CompareChar: WCHAR = if NamePosition >= (*Name).Length / core::mem::size_of::<WCHAR>() as USHORT {
                    0u16
                } else if IgnoreCase != 0 {
                    *UpcaseTable.offset(*(*Name).Buffer.offset(NamePosition as isize) as isize)
                } else {
                    *(*Name).Buffer.offset(NamePosition as isize)
                };

                let expr_char = *(*Expression).Buffer.offset((ExpressionPosition / core::mem::size_of::<WCHAR>() as USHORT) as isize);

                if expr_char == CompareChar && EndOfName == 0 {
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + core::mem::size_of::<WCHAR>() as USHORT) * 2;
                    BackTrackingPosition += 1;
                } else if expr_char == b'?' as u16 && EndOfName == 0 {
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + core::mem::size_of::<WCHAR>() as USHORT) * 2;
                    BackTrackingPosition += 1;
                } else if expr_char == b'*' as u16 {
                    *BackTracking.offset(BackTrackingPosition as isize) = ExpressionPosition * 2;
                    BackTrackingPosition += 1;
                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition * 2) + 3;
                    BackTrackingPosition += 1;
                    Offset = core::mem::size_of::<WCHAR>() as USHORT;
                    continue;
                } else if expr_char == DOS_STAR {
                    let mut DontSkipDot: BOOLEAN = 1u8;
                    if EndOfName == 0 && *(*Name).Buffer.offset(NamePosition as isize) == b'.' as u16 {
                        let mut Position: USHORT = NamePosition + 1;
                        while Position < (*Name).Length / core::mem::size_of::<WCHAR>() as USHORT {
                            if *(*Name).Buffer.offset(Position as isize) == b'.' as u16 {
                                DontSkipDot = 0u8;
                                break;
                            }
                            Position += 1;
                        }
                    }

                    if EndOfName != 0 || *(*Name).Buffer.offset(NamePosition as isize) != b'.' as u16 || DontSkipDot == 0 {
                        *BackTracking.offset(BackTrackingPosition as isize) = ExpressionPosition * 2;
                        BackTrackingPosition += 1;
                    }

                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition * 2) + 3;
                    BackTrackingPosition += 1;
                    Offset = core::mem::size_of::<WCHAR>() as USHORT;
                    continue;
                } else if expr_char == DOS_DOT {
                    if EndOfName != 0 {
                        Offset = core::mem::size_of::<WCHAR>() as USHORT;
                        continue;
                    }

                    if *(*Name).Buffer.offset(NamePosition as isize) == b'.' as u16 {
                        *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + core::mem::size_of::<WCHAR>() as USHORT) * 2;
                        BackTrackingPosition += 1;
                    }
                } else if expr_char == DOS_QM {
                    if EndOfName != 0 || *(*Name).Buffer.offset(NamePosition as isize) == b'.' as u16 {
                        Offset = core::mem::size_of::<WCHAR>() as USHORT;
                        continue;
                    }

                    *BackTracking.offset(BackTrackingPosition as isize) = (ExpressionPosition + core::mem::size_of::<WCHAR>() as USHORT) * 2;
                    BackTrackingPosition += 1;
                }

                break;
            }

            let mut Position: USHORT = 0;
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
        NamePosition += 1;
    }

    Result = if MatchingChars > 0 && *OldBackTracking.offset((MatchingChars - 1) as isize) == ((*Expression).Length * 2) {
        1u8
    } else {
        0u8
    };

    if !AllocatedBuffer.is_null() {
        ExFreePoolWithTag(AllocatedBuffer, u32::from_le_bytes([b'n', b'r', b'S', b'F']));
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAreNamesEqual(
    Name1: PCUNICODE_STRING,
    Name2: PCUNICODE_STRING,
    IgnoreCase: BOOLEAN,
    UpcaseTable: PCWCH,
) -> BOOLEAN {
    let mut UpcaseName1: _UNICODE_STRING = core::mem::zeroed();
    let mut UpcaseName2: _UNICODE_STRING = core::mem::zeroed();
    let mut StringsAreEqual: BOOLEAN;
    let mut MemoryAllocated: BOOLEAN = 0u8;
    let mut Name1Ptr = Name1;
    let mut Name2Ptr = Name2;

    if (*Name1).Length != (*Name2).Length {
        return 0u8;
    }

    if IgnoreCase != 0 && UpcaseTable.is_null() {
        let mut Status = RtlUpcaseUnicodeString(
            &mut UpcaseName1 as *mut _UNICODE_STRING,
            Name1 as *const _UNICODE_STRING as *mut _UNICODE_STRING,
            1u8,
        );
        if Status < 0 {
            RtlRaiseStatus(Status);
        }

        Status = RtlUpcaseUnicodeString(
            &mut UpcaseName2 as *mut _UNICODE_STRING,
            Name2 as *const _UNICODE_STRING as *mut _UNICODE_STRING,
            1u8,
        );
        if Status < 0 {
            RtlFreeUnicodeString(&mut UpcaseName1 as *mut _UNICODE_STRING);
            RtlRaiseStatus(Status);
        }

        Name1Ptr = &UpcaseName1 as *const _UNICODE_STRING;
        Name2Ptr = &UpcaseName2 as *const _UNICODE_STRING;

        MemoryAllocated = 1u8;
    }

    if IgnoreCase == 0 || MemoryAllocated != 0 {
        StringsAreEqual = RtlEqualMemory(
            (*Name1Ptr).Buffer as PVOID,
            (*Name2Ptr).Buffer as PVOID,
            (*Name1Ptr).Length as usize,
        ) as BOOLEAN;

        if MemoryAllocated != 0 {
            RtlFreeUnicodeString(&mut UpcaseName1 as *mut _UNICODE_STRING);
            RtlFreeUnicodeString(&mut UpcaseName2 as *mut _UNICODE_STRING);
        }

        return StringsAreEqual;
    } else {
        let mut i: USHORT = 0;
        while i < (*Name1Ptr).Length / core::mem::size_of::<WCHAR>() as USHORT {
            let char1 = *(*Name1Ptr).Buffer.offset(i as isize);
            let char2 = *(*Name2Ptr).Buffer.offset(i as isize);
            if *UpcaseTable.offset(char1 as isize) != *UpcaseTable.offset(char2 as isize) {
                return 0u8;
            }
            i += 1;
        }

        return 1u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDissectName(
    Name: _UNICODE_STRING,
    FirstPart: PUNICODE_STRING,
    RemainingPart: PUNICODE_STRING,
) {
    let FirstPart = opaque_ptr(FirstPart);
    let RemainingPart = opaque_ptr(RemainingPart);
    let mut FirstPosition: USHORT;
    let mut SkipFirstSlash: USHORT = 0;

    core::ptr::write_bytes(FirstPart as *mut u8, 0, core::mem::size_of::<_UNICODE_STRING>());
    core::ptr::write_bytes(RemainingPart as *mut u8, 0, core::mem::size_of::<_UNICODE_STRING>());

    if Name.Length == 0 {
        return;
    }

    FirstPosition = Name.Length / core::mem::size_of::<WCHAR>() as USHORT;
    let mut i: USHORT = 0;
    while i < Name.Length / core::mem::size_of::<WCHAR>() as USHORT {
        if *Name.Buffer.offset(i as isize) == b'\\' as u16 {
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
    (*FirstPart).Length = (FirstPosition - SkipFirstSlash) * core::mem::size_of::<WCHAR>() as USHORT;
    (*FirstPart).MaximumLength = (*FirstPart).Length;

    if FirstPosition < (Name.Length / core::mem::size_of::<WCHAR>() as USHORT) {
        (*RemainingPart).Buffer = Name.Buffer.offset((FirstPosition + 1) as isize);
        (*RemainingPart).Length = Name.Length - (FirstPosition + 1) * core::mem::size_of::<WCHAR>() as USHORT;
        (*RemainingPart).MaximumLength = (*RemainingPart).Length;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDoesNameContainWildCards(Name: PUNICODE_STRING) -> BOOLEAN {
    let Name = opaque_ptr(Name);
    if (*Name).Length != 0 {
        let mut Ptr = (*Name).Buffer.offset(((*Name).Length / core::mem::size_of::<WCHAR>() as USHORT - 1) as isize);
        while Ptr >= (*Name).Buffer && *Ptr != b'\\' as u16 {
            if FsRtlIsUnicodeCharacterWild(*Ptr) != 0 {
                return 1u8;
            }
            Ptr = Ptr.offset(-1);
        }
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsNameInExpression(
    Expression: PUNICODE_STRING,
    Name: PUNICODE_STRING,
    IgnoreCase: BOOLEAN,
    UpcaseTable: PCWCH,
) -> BOOLEAN {
    let Expression = opaque_ptr(Expression);
    let Name = opaque_ptr(Name);
    let mut Result: BOOLEAN;
    let mut IntName: _UNICODE_STRING = core::mem::zeroed();
    let mut NamePtr = Name;

    if IgnoreCase != 0 && UpcaseTable.is_null() {
        let Status = RtlUpcaseUnicodeString(
            &mut IntName as *mut _UNICODE_STRING,
            Name,
            1u8,
        );
        if Status < 0 {
            ExRaiseStatus(Status);
        }
        NamePtr = &mut IntName as *mut _UNICODE_STRING;
    }

    Result = FsRtlIsNameInExpressionPrivate(
        Expression,
        NamePtr,
        if UpcaseTable.is_null() { 0u8 } else { IgnoreCase },
        UpcaseTable,
    );

    if !IntName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut IntName as *mut _UNICODE_STRING);
    }

    Result
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/name.c
//  * PURPOSE:         Provides name parsing and other support routines for FSDs
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
// BOOLEAN
// NTAPI
// FsRtlIsNameInExpressionPrivate(IN PUNICODE_STRING Expression,
//                                IN PUNICODE_STRING Name,
//                                IN BOOLEAN IgnoreCase,
//                                IN PWCHAR UpcaseTable OPTIONAL)
// {
//     USHORT Offset, Position, BackTrackingPosition, OldBackTrackingPosition;
//     USHORT BackTrackingBuffer[16], OldBackTrackingBuffer[16] = {0};
//     PUSHORT BackTrackingSwap, BackTracking = BackTrackingBuffer, OldBackTracking = OldBackTrackingBuffer;
//     ULONG BackTrackingBufferSize = RTL_NUMBER_OF(BackTrackingBuffer);
//     PVOID AllocatedBuffer = NULL;
//     _UNICODE_STRING IntExpression;
//     USHORT ExpressionPosition, NamePosition = 0, MatchingChars = 1;
//     BOOLEAN EndOfName = FALSE;
//     BOOLEAN Result;
//     BOOLEAN DontSkipDot;
//     WCHAR CompareChar;
//     PAGED_CODE();
//
//     /* Check if we were given strings at all */
//     if (!Name->Length || !Expression->Length)
//     {
//         /* Return TRUE if both strings are empty, otherwise FALSE */
//         if (!Name->Length && !Expression->Length)
//             return TRUE;
//         else
//             return FALSE;
//     }
//
//     /* Check for a shortcut: just one wildcard */
//     if (Expression->Length == sizeof(WCHAR))
//     {
//         if (Expression->Buffer[0] == L'*')
//             return TRUE;
//     }
//
//     ASSERT(!IgnoreCase || UpcaseTable);
//
//     /* Another shortcut, wildcard followed by some string */
//     if (Expression->Buffer[0] == L'*')
//     {
//         /* Copy Expression to our local variable */
//         IntExpression = *Expression;
//
//         /* Skip the first char */
//         IntExpression.Buffer++;
//         IntExpression.Length -= sizeof(WCHAR);
//
//         /* Continue only if the rest of the expression does NOT contain
//            any more wildcards */
//         if (!FsRtlDoesNameContainWildCards(&IntExpression))
//         {
//             /* Check for a degenerate case */
//             if (Name->Length < (Expression->Length - sizeof(WCHAR)))
//                 return FALSE;
//
//             /* Calculate position */
//             NamePosition = (Name->Length - IntExpression.Length) / sizeof(WCHAR);
//
//             /* Compare */
//             if (!IgnoreCase)
//             {
//                 /* We can just do a byte compare */
//                 return RtlEqualMemory(IntExpression.Buffer,
//                                       Name->Buffer + NamePosition,
//                                       IntExpression.Length);
//             }
//             else
//             {
//                 /* Not so easy, need to upcase and check char by char */
//                 for (ExpressionPosition = 0; ExpressionPosition < (IntExpression.Length / sizeof(WCHAR)); ExpressionPosition++)
//                 {
//                     /* Assert that expression is already upcased! */
//                     ASSERT(IntExpression.Buffer[ExpressionPosition] == UpcaseTable[IntExpression.Buffer[ExpressionPosition]]);
//
//                     /* Now compare upcased name char with expression */
//                     if (UpcaseTable[Name->Buffer[NamePosition + ExpressionPosition]] !=
//                         IntExpression.Buffer[ExpressionPosition])
//                     {
//                         return FALSE;
//                     }
//                 }
//
//                 /* It matches */
//                 return TRUE;
//             }
//         }
//     }
//
//     /* Name parsing loop */
//     for (; !EndOfName; MatchingChars = BackTrackingPosition, NamePosition++)
//     {
//         /* Reset positions */
//         OldBackTrackingPosition = BackTrackingPosition = 0;
//
//         if (NamePosition >= Name->Length / sizeof(WCHAR))
//         {
//             EndOfName = TRUE;
//             if (MatchingChars && (OldBackTracking[MatchingChars - 1] == Expression->Length * 2))
//                 break;
//         }
//
//         while (MatchingChars > OldBackTrackingPosition)
//         {
//             ExpressionPosition = (OldBackTracking[OldBackTrackingPosition++] + 1) / 2;
//
//             /* Expression parsing loop */
//             for (Offset = 0; ExpressionPosition < Expression->Length; Offset = sizeof(WCHAR))
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
//                     BackTrackingBufferSize = Expression->Length / sizeof(WCHAR) * 2 + 1;
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
//                 /* Basic check to test if chars are equal */
//                 CompareChar = (NamePosition >= Name->Length / sizeof(WCHAR)) ? UNICODE_NULL : (IgnoreCase ? UpcaseTable[Name->Buffer[NamePosition]] :
//                                            Name->Buffer[NamePosition]);
//                 if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == CompareChar && !EndOfName)
//                 {
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + sizeof(WCHAR)) * 2;
//                 }
//                 /* Check cases that eat one char */
//                 else if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == L'?' && !EndOfName)
//                 {
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + sizeof(WCHAR)) * 2;
//                 }
//                 /* Test star */
//                 else if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == L'*')
//                 {
//                     BackTracking[BackTrackingPosition++] = ExpressionPosition * 2;
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition * 2) + 3;
//                     continue;
//                 }
//                 /* Check DOS_STAR */
//                 else if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == DOS_STAR)
//                 {
//                     /* Look for last dot */
//                     DontSkipDot = TRUE;
//                     if (!EndOfName && Name->Buffer[NamePosition] == '.')
//                     {
//                         for (Position = NamePosition + 1; Position < Name->Length / sizeof(WCHAR); Position++)
//                         {
//                             if (Name->Buffer[Position] == L'.')
//                             {
//                                 DontSkipDot = FALSE;
//                                 break;
//                             }
//                         }
//                     }
//
//                     if (EndOfName || Name->Buffer[NamePosition] != L'.' || !DontSkipDot)
//                         BackTracking[BackTrackingPosition++] = ExpressionPosition * 2;
//
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition * 2) + 3;
//                     continue;
//                 }
//                 /* Check DOS_DOT */
//                 else if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == DOS_DOT)
//                 {
//                     if (EndOfName) continue;
//
//                     if (Name->Buffer[NamePosition] == L'.')
//                         BackTracking[BackTrackingPosition++] = (ExpressionPosition + sizeof(WCHAR)) * 2;
//                 }
//                 /* Check DOS_QM */
//                 else if (Expression->Buffer[ExpressionPosition / sizeof(WCHAR)] == DOS_QM)
//                 {
//                     if (EndOfName || Name->Buffer[NamePosition] == L'.') continue;
//
//                     BackTracking[BackTrackingPosition++] = (ExpressionPosition + sizeof(WCHAR)) * 2;
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
//     Result = MatchingChars > 0 && (OldBackTracking[MatchingChars - 1] == (Expression->Length * 2));
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
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlAreNamesEqual
//  * @implemented
//  *
//  * Compare two strings to check if they match
//  *
//  * @param Name1
//  *        First unicode string to compare
//  *
//  * @param Name2
//  *        Second unicode string to compare
//  *
//  * @param IgnoreCase
//  *        If TRUE, Case will be ignored when comparing strings
//  *
//  * @param UpcaseTable
//  *        Table for upcase letters. If NULL is given, system one will be used
//  *
//  * @return TRUE if the strings are equal
//  *
//  * @remarks From Bo Branten's ntifs.h v25.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlAreNamesEqual(IN PCUNICODE_STRING Name1,
//                    IN PCUNICODE_STRING Name2,
//                    IN BOOLEAN IgnoreCase,
//                    IN PCWCH UpcaseTable OPTIONAL)
// {
//     _UNICODE_STRING UpcaseName1;
//     _UNICODE_STRING UpcaseName2;
//     BOOLEAN StringsAreEqual, MemoryAllocated = FALSE;
//     USHORT i;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Well, first check their size */
//     if (Name1->Length != Name2->Length) return FALSE;
//
//     /* Check if the caller didn't give an upcase table */
//     if ((IgnoreCase) && !(UpcaseTable))
//     {
//         /* Upcase the string ourselves */
//         Status = RtlUpcaseUnicodeString(&UpcaseName1, Name1, TRUE);
//         if (!NT_SUCCESS(Status)) RtlRaiseStatus(Status);
//
//         /* Upcase the second string too */
//         Status = RtlUpcaseUnicodeString(&UpcaseName2, Name2, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlFreeUnicodeString(&UpcaseName1);
//             RtlRaiseStatus(Status);
//         }
//
//         Name1 = &UpcaseName1;
//         Name2 = &UpcaseName2;
//
//         /* Make sure we go through the path below, but free the strings */
//         IgnoreCase = FALSE;
//         MemoryAllocated = TRUE;
//     }
//
//     /* Do a case-sensitive search */
//     if (!IgnoreCase)
//     {
//         /* Use a raw memory compare */
//         StringsAreEqual = RtlEqualMemory(Name1->Buffer,
//                                          Name2->Buffer,
//                                          Name1->Length);
//
//         /* Check if we allocated strings */
//         if (MemoryAllocated)
//         {
//             /* Free them */
//             RtlFreeUnicodeString(&UpcaseName1);
//             RtlFreeUnicodeString(&UpcaseName2);
//         }
//
//         /* Return the equality */
//         return StringsAreEqual;
//     }
//     else
//     {
//         /* Case in-sensitive search */
//         for (i = 0; i < Name1->Length / sizeof(WCHAR); i++)
//         {
//             /* Check if the character matches */
//             if (UpcaseTable[Name1->Buffer[i]] != UpcaseTable[Name2->Buffer[i]])
//             {
//                 /* Non-match found! */
//                 return FALSE;
//             }
//         }
//
//         /* We finished the loop so we are equal */
//         return TRUE;
//     }
// }
//
// /*++
//  * @name FsRtlDissectName
//  * @implemented
//  *
//  * Dissects a given path name into first and remaining part.
//  *
//  * @param Name
//  *        Unicode string to dissect.
//  *
//  * @param FirstPart
//  *        Pointer to user supplied _UNICODE_STRING, that will later point
//  *        to the first part of the original name.
//  *
//  * @param RemainingPart
//  *        Pointer to user supplied _UNICODE_STRING, that will later point
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
// FsRtlDissectName(IN _UNICODE_STRING Name,
//                  OUT PUNICODE_STRING FirstPart,
//                  OUT PUNICODE_STRING RemainingPart)
// {
//     USHORT FirstPosition, i;
//     USHORT SkipFirstSlash = 0;
//     PAGED_CODE();
//
//     /* Zero the strings before continuing */
//     RtlZeroMemory(FirstPart, sizeof(_UNICODE_STRING));
//     RtlZeroMemory(RemainingPart, sizeof(_UNICODE_STRING));
//
//     /* Just quit if the string is empty */
//     if (!Name.Length) return;
//
//     /* Find first backslash */
//     FirstPosition = Name.Length / sizeof(WCHAR) ;
//     for (i = 0; i < Name.Length / sizeof(WCHAR); i++)
//     {
//         /* If we found one... */
//         if (Name.Buffer[i] == L'\\')
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
//     FirstPart->Length = (FirstPosition - SkipFirstSlash) * sizeof(WCHAR);
//     FirstPart->MaximumLength = FirstPart->Length;
//
//     /* And second one, if necessary */
//     if (FirstPosition < (Name.Length / sizeof(WCHAR)))
//     {
//         RemainingPart->Buffer = Name.Buffer + FirstPosition + 1;
//         RemainingPart->Length = Name.Length - (FirstPosition + 1) * sizeof(WCHAR);
//         RemainingPart->MaximumLength = RemainingPart->Length;
//     }
// }
//
// /*++
//  * @name FsRtlDoesNameContainWildCards
//  * @implemented
//  *
//  * Checks if the given string contains WildCards
//  *
//  * @param Name
//  *        Pointer to a _UNICODE_STRING containing Name to examine
//  *
//  * @return TRUE if Name contains wildcards, FALSE otherwise
//  *
//  * @remarks From Bo Branten's ntifs.h v12.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlDoesNameContainWildCards(IN PUNICODE_STRING Name)
// {
//     PWCHAR Ptr;
//     PAGED_CODE();
//
//     /* Loop through every character */
//     if (Name->Length)
//     {
//         Ptr = Name->Buffer + (Name->Length / sizeof(WCHAR)) - 1;
//         while ((Ptr >= Name->Buffer) && (*Ptr != L'\\'))
//         {
//             /* Check for Wildcard */
//             if (FsRtlIsUnicodeCharacterWild(*Ptr)) return TRUE;
//             Ptr--;
//         }
//     }
//
//     /* Nothing Found */
//     return FALSE;
// }
//
// /*++
//  * @name FsRtlIsNameInExpression
//  * @implemented
//  *
//  * Check if the Name string is in the Expression string.
//  *
//  * @param Expression
//  *        The string in which we've to find Name. It can contain wildcards.
//  *        If IgnoreCase is set to TRUE, this string MUST BE uppercase.
//  *
//  * @param Name
//  *        The string to find. It cannot contain wildcards
//  *
//  * @param IgnoreCase
//  *        If set to TRUE, case will be ignore with upcasing both strings
//  *
//  * @param UpcaseTable
//  *        If not NULL, and if IgnoreCase is set to TRUE, it will be used to
//  *        upcase the both strings
//  *
//  * @return TRUE if Name is in Expression, FALSE otherwise
//  *
//  * @remarks From Bo Branten's ntifs.h v12. This function should be
//  *          rewritten to avoid recursion and better wildcard handling
//  *          should be implemented (see FsRtlDoesNameContainWildCards).
//  *
//  *--*/
// BOOLEAN
// NTAPI
// FsRtlIsNameInExpression(IN PUNICODE_STRING Expression,
//                         IN PUNICODE_STRING Name,
//                         IN BOOLEAN IgnoreCase,
//                         IN PWCHAR UpcaseTable OPTIONAL)
// {
//     BOOLEAN Result;
//     NTSTATUS Status;
//     _UNICODE_STRING IntName;
//
//     if (IgnoreCase && !UpcaseTable)
//     {
//         Status = RtlUpcaseUnicodeString(&IntName, Name, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             ExRaiseStatus(Status);
//         }
//         Name = &IntName;
//         IgnoreCase = FALSE;
//     }
//     else
//     {
//         IntName.Buffer = NULL;
//     }
//
//     Result = FsRtlIsNameInExpressionPrivate(Expression, Name, IgnoreCase, UpcaseTable);
//
//     if (IntName.Buffer != NULL)
//     {
//         RtlFreeUnicodeString(&IntName);
//     }
//
//     return Result;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
