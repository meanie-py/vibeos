use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCCH, PCSTR, PSTR};
use core::ffi::c_void;

extern "C" {
    fn strcspn(s: PCSTR, reject: PCSTR) -> usize;
    fn strspn(s: PCSTR, accept: PCSTR) -> usize;
    fn RtlStringCbCatA(pszDest: PSTR, cbDest: usize, pszSrc: PCSTR) -> i32;
    fn strlen(s: PCSTR) -> usize;
    fn strncpy(dest: PSTR, src: PCSTR, n: usize) -> PSTR;
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrGetNextOption(
    Options: *mut PCSTR,
    OptionLength: *mut u32,
) -> PCSTR {
    let mut NextOption: PCSTR;
    let mut Option: PCSTR = core::ptr::null_mut();
    let mut Length: u32 = 0;

    if !OptionLength.is_null() {
        *OptionLength = 0;
    }

    if Options.is_null() || (*Options).is_null() {
        return core::ptr::null_mut();
    }

    NextOption = *Options;
    while *NextOption != 0 {
        NextOption = NextOption.add(strspn(NextOption, b" \t\0".as_ptr() as PCSTR));

        if !Option.is_null() {
            break;
        }

        if *NextOption == ((b'/') as i8) {
            NextOption = NextOption.add(1);
        }

        Length = strcspn(NextOption, b" \t/\0".as_ptr() as PCSTR) as u32;

        if Length != 0 {
            Option = NextOption;
        }

        NextOption = NextOption.add(Length as usize);
    }

    *Options = NextOption;
    if !Option.is_null() && !OptionLength.is_null() {
        *OptionLength = Length;
    }
    Option
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrGetOptionExN(
    Options: PCSTR,
    OptionName: PCCH,
    OptNameLength: u32,
    OptionLength: *mut u32,
) -> PCSTR {

    let mut NextOptions: PCSTR;
    let mut Option: PCSTR = core::ptr::null_mut();
    let mut OptLength: u32 = 0;

    if !OptionLength.is_null() {
        *OptionLength = 0;
    }

    if Options.is_null() || *Options == 0 {
        return core::ptr::null_mut();
    }
    if OptionName.is_null() || OptNameLength == 0 || *OptionName == 0 {
        return core::ptr::null_mut();
    }

    NextOptions = Options;
    while {
        Option = NtLdrGetNextOption(&mut NextOptions, &mut OptLength);
        !Option.is_null()
    } {
        if (OptLength >= OptNameLength) &&
            (_strnicmp(Option, OptionName, OptNameLength as usize) == 0)
        {
            if (OptLength == OptNameLength) ||
                (*OptionName.offset((OptNameLength - 1) as isize) == b'=' as i8) ||
                (*OptionName.offset((OptNameLength - 1) as isize) == b':' as i8)
            {
                break;
            }
        }
    }

    if !Option.is_null() && !OptionLength.is_null() {
        *OptionLength = OptLength;
    }
    Option
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrGetOptionEx(
    Options: PCSTR,
    OptionName: PCSTR,
    OptionLength: *mut u32,
) -> PCSTR {

    NtLdrGetOptionExN(
        Options,
        OptionName,
        strlen(OptionName) as u32,
        OptionLength,
    )
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrGetOption(
    Options: PCSTR,
    OptionName: PCSTR,
) -> PCSTR {
    NtLdrGetOptionEx(Options, OptionName, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrAddOptions(
    LoadOptions: PSTR,
    BufferSize: u32,
    Append: u8,
    NewOptions: PCSTR,
) {
    let mut OptionsLength: u32;
    let mut NewOptsLength: u32;
    let mut AddSeparator: u8;

    if LoadOptions.is_null() || (BufferSize == 0) {
        return;
    }

    if NewOptions.is_null() || *NewOptions == 0 {
        return;
    }

    if Append != 0 {
        OptionsLength = strlen(LoadOptions) as u32;
        OptionsLength = core::cmp::min(OptionsLength, BufferSize - 1);

        if OptionsLength != 0 &&
            (*LoadOptions.offset(OptionsLength as isize - 1) != b' ' as i8) &&
            (*LoadOptions.offset(OptionsLength as isize - 1) != b'\t' as i8) &&
            (*NewOptions != 0) &&
            (*NewOptions != b' ' as i8) &&
            (*NewOptions != b'\t' as i8)
        {
            RtlStringCbCatA(LoadOptions, BufferSize as usize * core::mem::size_of::<i8>(), b" \0".as_ptr() as PCSTR);
        }

        RtlStringCbCatA(LoadOptions, BufferSize as usize * core::mem::size_of::<i8>(), NewOptions);
    } else {
        NewOptsLength = strlen(NewOptions) as u32;
        NewOptsLength = core::cmp::min(NewOptsLength, BufferSize - 1);

        AddSeparator = 0u8;
        if NewOptsLength != 0 &&
            (*NewOptions.offset(NewOptsLength as isize - 1) != b' ' as i8) &&
            (*NewOptions.offset(NewOptsLength as isize - 1) != b'\t' as i8) &&
            (*LoadOptions != 0) &&
            (*LoadOptions != b' ' as i8) &&
            (*LoadOptions != b'\t' as i8)
        {
            AddSeparator = 1u8;
            NewOptsLength += 1;
        }

        OptionsLength = strlen(LoadOptions) as u32 + 1;
        OptionsLength = core::cmp::min(OptionsLength, BufferSize - NewOptsLength);
        core::ptr::copy( // nolint: silent-failure (byte-array pointee — count==bytes)
            LoadOptions,
            LoadOptions.offset(NewOptsLength as isize),
            OptionsLength as usize * core::mem::size_of::<i8>(),
        );
        *LoadOptions.offset(NewOptsLength as isize).offset(OptionsLength as isize - 1) = 0;

        if AddSeparator != 0 {
            NewOptsLength -= 1;
        }
        strncpy(LoadOptions, NewOptions, NewOptsLength as usize);
        if AddSeparator != 0 {
            *LoadOptions.offset(NewOptsLength as isize) = b' ' as i8;
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     NT Kernel Load Options Support Functions
//  * COPYRIGHT:   Copyright 2020 Hermes Belusca-Maito
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
// #include "ntldropts.h"
//
// /* FUNCTIONS *****************************************************************/
//
// PCSTR
// NtLdrGetNextOption(
//     IN OUT PCSTR* Options,
//     OUT PULONG OptionLength OPTIONAL)
// {
//     PCSTR NextOption;
//     PCSTR Option = NULL;
//     ULONG Length = 0;
//
//     if (OptionLength)
//         *OptionLength = 0;
//
//     if (!Options || !*Options)
//         return NULL;
//
//     /* Loop over each option */
//     NextOption = *Options;
//     while (*NextOption)
//     {
//         /* Skip possible initial whitespace */
//         NextOption += strspn(NextOption, " \t");
//
//         /* Stop now if we have already found an option.
//          * NextOption points to the next following option. */
//         if (Option)
//             break;
//
//         /*
//          * Check whether a new option starts. Options are delimited
//          * with an option separator '/' or with whitespace.
//          */
//         if (*NextOption == '/')
//             ++NextOption;
//
//         /* Get the actual length of the option until
//          * the next whitespace or option separator. */
//         Length = (ULONG)strcspn(NextOption, " \t/");
//
//         /* Retrieve the option if present and go to the beginning of the next one */
//         if (Length != 0)
//             Option = NextOption;
//
//         /* Restart after the end of the option */
//         NextOption += Length;
//     }
//
//     *Options = NextOption;
//     if (Option && OptionLength)
//         *OptionLength = Length;
//     return Option;
// }
//
// /*
//  * OptionName specifies the option name, without any leading
//  * option separator '/', to search for within the Options.
//  * The search is made case-insensitive.
//  */
// PCSTR
// NtLdrGetOptionExN(
//     IN PCSTR Options,
//     IN PCCH OptionName,
//     IN ULONG OptNameLength,
//     OUT PULONG OptionLength OPTIONAL)
// {
//     PCSTR NextOptions;
//     PCSTR Option = NULL;
//     ULONG OptLength = 0;
//
//     if (OptionLength)
//         *OptionLength = 0;
//
//     if (!Options || !*Options)
//         return NULL;
//     if (!OptionName || (OptNameLength == 0) || !*OptionName)
//         return NULL;
//
//     NextOptions = Options;
//     while ((Option = NtLdrGetNextOption(&NextOptions, &OptLength)))
//     {
//         /*
//          * Check whether the option to find exactly matches the current
//          * load option, or is a prefix thereof if this is an option with
//          * appended data.
//          */
//         if ((OptLength >= OptNameLength) &&
//             (_strnicmp(Option, OptionName, OptNameLength) == 0))
//         {
//             if ((OptLength == OptNameLength) ||
//                 (OptionName[OptNameLength-1] == '=') ||
//                 (OptionName[OptNameLength-1] == ':'))
//             {
//                 break;
//             }
//         }
//     }
//
//     if (Option && OptionLength)
//         *OptionLength = OptLength;
//     return Option;
// }
//
// PCSTR
// NtLdrGetOptionEx(
//     IN PCSTR Options,
//     IN PCSTR OptionName,
//     OUT PULONG OptionLength OPTIONAL)
// {
//     return NtLdrGetOptionExN(Options, OptionName,
//                              (ULONG)strlen(OptionName),
//                              OptionLength);
// }
//
// PCSTR
// NtLdrGetOption(
//     IN PCSTR Options,
//     IN PCSTR OptionName)
// {
//     return NtLdrGetOptionEx(Options, OptionName, NULL);
// }
//
// /*
//  * Appends or prepends new options to the ones originally contained
//  * in the buffer pointed by LoadOptions, of maximum size BufferSize.
//  */
// VOID
// NtLdrAddOptions(
//     IN OUT PSTR LoadOptions,
//     IN ULONG BufferSize,
//     IN BOOLEAN Append,
//     IN PCSTR NewOptions OPTIONAL)
// {
//     ULONG OptionsLength;
//     ULONG NewOptsLength;
//     BOOLEAN AddSeparator;
//
//     if (!LoadOptions || (BufferSize == 0))
//         return;
//     // ASSERT(strlen(LoadOptions) + 1 <= BufferSize);
//
//     if (!NewOptions || !*NewOptions)
//         return;
//
//     if (Append)
//     {
//         OptionsLength = (ULONG)strlen(LoadOptions);
//         OptionsLength = min(OptionsLength, BufferSize-1);
//
//         /* Add a whitespace separator if needed */
//         if (OptionsLength != 0 &&
//             (LoadOptions[OptionsLength-1] != ' ') &&
//             (LoadOptions[OptionsLength-1] != '\t') &&
//             (*NewOptions != '\0') &&
//             (*NewOptions != ' ') &&
//             (*NewOptions != '\t'))
//         {
//             RtlStringCbCatA(LoadOptions, BufferSize * sizeof(CHAR), " ");
//         }
//
//         /* Append the options */
//         RtlStringCbCatA(LoadOptions, BufferSize * sizeof(CHAR), NewOptions);
//     }
//     else
//     {
//         NewOptsLength = (ULONG)strlen(NewOptions);
//         NewOptsLength = min(NewOptsLength, BufferSize-1);
//
//         /* Add a whitespace separator if needed */
//         AddSeparator = FALSE;
//         if (NewOptsLength != 0 &&
//             (NewOptions[NewOptsLength-1] != ' ') &&
//             (NewOptions[NewOptsLength-1] != '\t') &&
//             (*LoadOptions != '\0') &&
//             (*LoadOptions != ' ') &&
//             (*LoadOptions != '\t'))
//         {
//             AddSeparator = TRUE;
//             ++NewOptsLength;
//         }
//
//         /*
//          * Move the original load options forward (possibly truncating them
//          * at the end if the buffer is not large enough) to make place for
//          * the options to prepend.
//          */
//         OptionsLength = (ULONG)strlen(LoadOptions) + 1;
//         OptionsLength = min(OptionsLength, BufferSize - NewOptsLength);
//         RtlMoveMemory(LoadOptions + NewOptsLength,
//                       LoadOptions,
//                       OptionsLength * sizeof(CHAR));
//         /* NULL-terminate */
//         (LoadOptions + NewOptsLength)[OptionsLength-1] = '\0';
//
//         /* Restore the new options length back to its original value */
//         if (AddSeparator) --NewOptsLength;
//         /* Prepend the options and add the whitespace separator if needed */
//         strncpy(LoadOptions, NewOptions, NewOptsLength);
//         if (AddSeparator) LoadOptions[NewOptsLength] = ' ';
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
