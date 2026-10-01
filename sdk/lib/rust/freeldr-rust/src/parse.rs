use ntkrnl_sys::base::*;
use core::ffi::c_void;
use crate::frldr::{
    INI_SECTION, _INI_SECTION, PINI_SECTION,
    INI_SECTION_ITEM, _INI_SECTION_ITEM, PINI_SECTION_ITEM,
    TAG_INI_FILE, TAG_INI_SECTION, TAG_INI_SECTION_ITEM, TAG_INI_NAME, TAG_INI_VALUE,
    INI_FILE_COMMENT_CHAR, IniFileSectionListHead, IniFileSectionInitialized,
    IniFileSectionCount, IniFileSettingCount, FrLdrTempAlloc, FrLdrTempFree,
};

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    fn strlen(s: *const i8) -> usize;
}

#[no_mangle]
pub unsafe extern "C" fn IniParseFile(IniFileData: PCHAR, IniFileSize: ULONG) -> BOOLEAN {
    let mut CurrentOffset: ULONG;
    let mut CurrentLineNumber: ULONG;
    let mut IniFileLine: PCHAR;
    let mut IniFileLineSize: ULONG;
    let mut LineLength: ULONG;
    let mut CurrentSection: PINI_SECTION = core::ptr::null_mut();
    let mut CurrentItem: PINI_SECTION_ITEM = core::ptr::null_mut();

    TRACE(b"IniParseFile() IniFileSize: %d\n\0".as_ptr() as PCHAR, IniFileSize);

    if IniFileSectionInitialized == 0 {
        InitializeListHead(&mut IniFileSectionListHead);
        IniFileSectionInitialized = 1u8;
    }

    IniFileLineSize = 80;
    IniFileLine = FrLdrTempAlloc(IniFileLineSize as usize, TAG_INI_FILE) as *mut i8;
    if IniFileLine.is_null() {
        return 0u8;
    }

    CurrentLineNumber = 0;
    CurrentOffset = 0;
    while CurrentOffset < IniFileSize {
        if IniFileLineSize < IniGetNextLineSize(IniFileData, IniFileSize, CurrentOffset) {
            IniFileLineSize = IniGetNextLineSize(IniFileData, IniFileSize, CurrentOffset);
            FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
            IniFileLine = FrLdrTempAlloc(IniFileLineSize as usize, TAG_INI_FILE) as *mut i8;
            if IniFileLine.is_null() {
                return 0u8;
            }
        }

        CurrentOffset = IniGetNextLine(IniFileData, IniFileSize, IniFileLine, IniFileLineSize, CurrentOffset);
        LineLength = strlen(IniFileLine) as ULONG;

        if IniIsLineEmpty(IniFileLine, LineLength) != 0 || IniIsCommentLine(IniFileLine, LineLength) != 0 {
            CurrentLineNumber += 1;
            continue;
        }

        if IniIsSectionName(IniFileLine, LineLength) != 0 {
            CurrentSection = FrLdrTempAlloc(core::mem::size_of::<INI_SECTION>(), TAG_INI_SECTION) as PINI_SECTION;
            if CurrentSection.is_null() {
                FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
                return 0u8;
            }

            core::ptr::write_bytes(CurrentSection, 0, 1);

            (*CurrentSection).SectionName = FrLdrTempAlloc(IniGetSectionNameSize(IniFileLine, LineLength) as usize, TAG_INI_NAME) as *mut i8;
            if (*CurrentSection).SectionName.is_null() {
                FrLdrTempFree(CurrentSection as *mut core::ffi::c_void, TAG_INI_FILE);
                FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
                return 0u8;
            }

            IniExtractSectionName((*CurrentSection).SectionName, IniFileLine, LineLength);
            InitializeListHead(&mut (*CurrentSection).SectionItemList);

            IniFileSectionCount += 1;
            InsertTailList(&mut IniFileSectionListHead, &mut (*CurrentSection).ListEntry);

            CurrentLineNumber += 1;
            continue;
        }

        if IniIsSetting(IniFileLine, LineLength) != 0 {
            if CurrentSection.is_null() {
                ERR(b"Error: freeldr.ini:%lu: Setting '%s' found outside of a [section].\n\0".as_ptr() as PCHAR, CurrentLineNumber, IniFileLine);

                CurrentLineNumber += 1;
                continue;
            }

            CurrentItem = FrLdrTempAlloc(core::mem::size_of::<INI_SECTION_ITEM>(), TAG_INI_SECTION_ITEM) as PINI_SECTION_ITEM;
            if CurrentItem.is_null() {
                FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
                return 0u8;
            }

            core::ptr::write_bytes(CurrentItem, 0, 1);

            (*CurrentItem).ItemName = FrLdrTempAlloc(IniGetSettingNameSize(IniFileLine, LineLength) as usize, TAG_INI_NAME) as *mut i8;
            if (*CurrentItem).ItemName.is_null() {
                FrLdrTempFree(CurrentItem as *mut core::ffi::c_void, TAG_INI_SECTION_ITEM);
                FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
                return 0u8;
            }

            (*CurrentItem).ItemValue = FrLdrTempAlloc(IniGetSettingValueSize(IniFileLine, LineLength) as usize, TAG_INI_VALUE) as *mut i8;
            if (*CurrentItem).ItemValue.is_null() {
                FrLdrTempFree((*CurrentItem).ItemName as *mut core::ffi::c_void, TAG_INI_NAME);
                FrLdrTempFree(CurrentItem as *mut core::ffi::c_void, TAG_INI_SECTION_ITEM);
                FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
                return 0u8;
            }

            IniExtractSettingName((*CurrentItem).ItemName, IniFileLine, LineLength);
            IniExtractSettingValue((*CurrentItem).ItemValue, IniFileLine, LineLength);

            IniFileSettingCount += 1;
            (*CurrentSection).SectionItemCount += 1;
            InsertTailList(&mut (*CurrentSection).SectionItemList, &mut (*CurrentItem).ListEntry);

            CurrentLineNumber += 1;
            continue;
        }

        CurrentLineNumber += 1;
    }

    FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);

    TRACE(b"Parsed %d sections and %d settings.\n\0".as_ptr() as PCHAR, IniFileSectionCount, IniFileSettingCount);
    TRACE(b"IniParseFile() done.\n\0".as_ptr() as PCHAR);

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetNextLineSize(IniFileData: PCHAR, IniFileSize: ULONG, mut CurrentOffset: ULONG) -> ULONG {
    let mut LineCharCount: ULONG = 0;

    while CurrentOffset < IniFileSize {
        LineCharCount += 1;

        if *IniFileData.offset(CurrentOffset as isize) == b'\n' as i8 {
            break;
        }

        CurrentOffset += 1;
    }

    LineCharCount += 1;

    return LineCharCount;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetNextLine(IniFileData: PCHAR, IniFileSize: ULONG, Buffer: PCHAR, BufferSize: ULONG, mut CurrentOffset: ULONG) -> ULONG {
    let mut Idx: ULONG = 0;

    while CurrentOffset < IniFileSize {
        if Idx < (BufferSize - 1) {
            *Buffer.add(Idx as usize) = *IniFileData.add(CurrentOffset as usize);
            Idx += 1;
        }

        if *IniFileData.add(CurrentOffset as usize) == b'\n' as i8 {
            CurrentOffset += 1;
            break;
        }

        CurrentOffset += 1;
    }

    *Buffer.add(Idx as usize) = 0;

    while Idx > 0 {
        Idx -= 1;
        let c = *Buffer.add(Idx as usize);
        if c == b'\n' as i8 || c == b'\r' as i8 {
            *Buffer.add(Idx as usize) = 0;
        } else {
            break;
        }
    }

    CurrentOffset
}

#[no_mangle]
pub unsafe extern "C" fn IniIsLineEmpty(LineOfText: PCHAR, TextLength: ULONG) -> BOOLEAN {
    let mut Idx: ULONG;

    Idx = 0;
    while Idx < TextLength {
        if (*LineOfText.offset(Idx as isize) == b' ' as i8) ||
           (*LineOfText.offset(Idx as isize) == b'\t' as i8) ||
           (*LineOfText.offset(Idx as isize) == b'\n' as i8) ||
           (*LineOfText.offset(Idx as isize) == b'\r' as i8)
        {
            Idx += 1;
            continue;
        } else {
            return 0u8;
        }
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniIsCommentLine(LineOfText: PCHAR, TextLength: ULONG) -> BOOLEAN {
    let mut Idx: ULONG;

    Idx = 0;
    while Idx < TextLength {
        if (*LineOfText.offset(Idx as isize) == b' ' as i8) ||
           (*LineOfText.offset(Idx as isize) == b'\t' as i8) {
            Idx += 1;
            continue;
        } else if *LineOfText.offset(Idx as isize) == INI_FILE_COMMENT_CHAR as i8 {
            return 1u8;
        } else {
            break;
        }
    }

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniIsSectionName(LineOfText: PCHAR, TextLength: ULONG) -> BOOLEAN {
    let mut Idx: ULONG;

    Idx = 0;
    while Idx < TextLength {
        if (*LineOfText.offset(Idx as isize) == b' ' as i8 as CHAR) ||
           (*LineOfText.offset(Idx as isize) == b'\t' as i8 as CHAR)
        {
            Idx += 1;
            continue;
        } else if *LineOfText.offset(Idx as isize) == b'[' as i8 as CHAR {
            return 1u8;
        } else {
            break;
        }
    }

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSectionNameSize(SectionNameLine: PCHAR, LineLength: ULONG) -> ULONG {
    let mut Idx: ULONG;
    let mut NameSize: ULONG;

    Idx = 0;
    while Idx < LineLength {
        if *SectionNameLine.offset(Idx as isize) == b' ' as i8 ||
           *SectionNameLine.offset(Idx as isize) == b'\t' as i8 {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    Idx += 1;

    NameSize = 0;
    while Idx < LineLength {
        if *SectionNameLine.offset(Idx as isize) == b']' as i8 ||
           *SectionNameLine.offset(Idx as isize) == b'\0' as i8 {
            break;
        }

        NameSize += 1;
        Idx += 1;
    }

    NameSize += 1;

    return NameSize;
}

#[no_mangle]
pub unsafe extern "C" fn IniExtractSectionName(SectionName: PCHAR, SectionNameLine: PCHAR, LineLength: ULONG) {
    let mut Idx: ULONG;
    let mut DestIdx: ULONG;

    Idx = 0;
    while Idx < LineLength {
        if *SectionNameLine.offset(Idx as isize) == b' ' as i8 ||
           *SectionNameLine.offset(Idx as isize) == b'\t' as i8 {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    Idx += 1;

    DestIdx = 0;
    while Idx < LineLength {
        if *SectionNameLine.offset(Idx as isize) == b']' as i8 ||
           *SectionNameLine.offset(Idx as isize) == b'\0' as i8 {
            break;
        }

        *SectionName.offset(DestIdx as isize) = *SectionNameLine.offset(Idx as isize);
        DestIdx += 1;
        Idx += 1;
    }

    *SectionName.offset(DestIdx as isize) = b'\0' as i8;
}

#[no_mangle]
pub unsafe extern "C" fn IniIsSetting(LineOfText: PCHAR, TextLength: ULONG) -> BOOLEAN {
    let mut Idx: ULONG;

    Idx = 0;
    while Idx < TextLength {
        if *LineOfText.offset(Idx as isize) == b'=' as i8 {
            return 1u8;
        }
        Idx += 1;
    }

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSettingNameSize(SettingNameLine: PCHAR, LineLength: ULONG) -> ULONG {
    let mut Idx: ULONG;
    let mut NameSize: ULONG;

    Idx = 0;
    while Idx < LineLength {
        if (*SettingNameLine.offset(Idx as isize) == b' ' as i8) ||
           (*SettingNameLine.offset(Idx as isize) == b'\t' as i8) {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    NameSize = 0;
    while Idx < LineLength {
        if (*SettingNameLine.offset(Idx as isize) == b'=' as i8) ||
           (*SettingNameLine.offset(Idx as isize) == 0) {
            break;
        }

        NameSize += 1;
        Idx += 1;
    }

    NameSize += 1;

    return NameSize;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSettingValueSize(SettingValueLine: PCHAR, LineLength: ULONG) -> ULONG {
    let mut Idx: ULONG;
    let mut ValueSize: ULONG;

    // Skip whitespace
    Idx = 0;
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b' ' as i8 ||
           *SettingValueLine.offset(Idx as isize) == b'\t' as i8 {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    // Skip the characters up until the '=' equals sign or EOL
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b'=' as i8 {
            Idx += 1;
            break;
        }

        // If we hit EOL then obviously the value size is zero
        if *SettingValueLine.offset(Idx as isize) == b'\0' as i8 {
            return 0;
        }

        Idx += 1;
    }

    // Count the characters up until the EOL
    ValueSize = 0;
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b'\0' as i8 {
            break;
        }

        // Increment the count
        ValueSize += 1;
        Idx += 1;
    }

    // Add one for the NULL-terminator
    ValueSize += 1;

    ValueSize
}

#[no_mangle]
pub unsafe extern "C" fn IniExtractSettingName(SettingName: PCHAR, SettingNameLine: PCHAR, LineLength: ULONG) {
    let mut Idx: ULONG;
    let mut DestIdx: ULONG;

    Idx = 0;
    while Idx < LineLength {
        if (*SettingNameLine.offset(Idx as isize) == b' ' as i8) ||
           (*SettingNameLine.offset(Idx as isize) == b'\t' as i8) {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    DestIdx = 0;
    while Idx < LineLength {
        if (*SettingNameLine.offset(Idx as isize) == b'=' as i8) ||
           (*SettingNameLine.offset(Idx as isize) == b'\0' as i8) {
            // Trim trailing whitespace (upstream f77fcc15ffc): `Key = value` gave the key
            // "Key ", which no lookup ever matched.
            while DestIdx > 0 {
                let c = *SettingName.offset((DestIdx - 1) as isize);
                if c == b' ' as i8 || c == b'\t' as i8 {
                    DestIdx -= 1;
                } else {
                    break;
                }
            }
            break;
        }

        *SettingName.offset(DestIdx as isize) = *SettingNameLine.offset(Idx as isize);
        DestIdx += 1;
        Idx += 1;
    }

    *SettingName.offset(DestIdx as isize) = b'\0' as i8;
}

#[no_mangle]
pub unsafe extern "C" fn IniExtractSettingValue(SettingValue: PCHAR, SettingValueLine: PCHAR, LineLength: ULONG) {
    let mut Idx: ULONG;
    let mut DestIdx: ULONG;

    // Skip whitespace
    Idx = 0;
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b' ' as i8 ||
           *SettingValueLine.offset(Idx as isize) == b'\t' as i8 {
            Idx += 1;
            continue;
        } else {
            break;
        }
    }

    // Skip the characters up until the '=' equals sign or EOL
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b'=' as i8 {
            Idx += 1;
            break;
        }

        // If we hit EOL then obviously the value size is zero
        if *SettingValueLine.offset(Idx as isize) == b'\0' as i8 {
            *SettingValue.offset(0) = b'\0' as i8;
            return;
        }

        Idx += 1;
    }

    // Get the characters up until the EOL
    DestIdx = 0;
    while Idx < LineLength {
        if *SettingValueLine.offset(Idx as isize) == b'\0' as i8 {
            break;
        }

        // Grab a character and increment DestIdx
        *SettingValue.offset(DestIdx as isize) = *SettingValueLine.offset(Idx as isize);
        DestIdx += 1;
        Idx += 1;
    }

    // Terminate the string
    *SettingValue.offset(DestIdx as isize) = b'\0' as i8;
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
// DBG_DEFAULT_CHANNEL(INIFILE);
//
// LIST_ENTRY IniFileSectionListHead = {&IniFileSectionListHead, &IniFileSectionListHead};
// BOOLEAN IniFileSectionInitialized = FALSE;
// ULONG   IniFileSectionCount = 0;
// ULONG   IniFileSettingCount = 0;
//
//
// BOOLEAN IniParseFile(PCHAR IniFileData, ULONG IniFileSize)
// {
//     ULONG                    CurrentOffset;
//     ULONG                    CurrentLineNumber;
//     PCHAR                IniFileLine;
//     ULONG                    IniFileLineSize;
//     ULONG                    LineLength;
//     PINI_SECTION        CurrentSection = NULL;
//     PINI_SECTION_ITEM    CurrentItem = NULL;
//
//     TRACE("IniParseFile() IniFileSize: %d\n", IniFileSize);
//
//     if (!IniFileSectionInitialized)
//     {
//         InitializeListHead(&IniFileSectionListHead);
//         IniFileSectionInitialized = TRUE;
//     }
//
//     // Start with an 80-byte buffer
//     IniFileLineSize = 80;
//     IniFileLine = FrLdrTempAlloc(IniFileLineSize as usize, TAG_INI_FILE) as *mut i8;
//     if (!IniFileLine)
//     {
//         return FALSE;
//     }
//
//     // Loop through each line and parse it
//     CurrentLineNumber = 0;
//     CurrentOffset = 0;
//     while (CurrentOffset < IniFileSize)
//     {
//         // First check the line size and increase our buffer if necessary
//         if (IniFileLineSize < IniGetNextLineSize(IniFileData, IniFileSize, CurrentOffset))
//         {
//             IniFileLineSize = IniGetNextLineSize(IniFileData, IniFileSize, CurrentOffset);
//             FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//             IniFileLine = FrLdrTempAlloc(IniFileLineSize as usize, TAG_INI_FILE) as *mut i8;
//             if (!IniFileLine)
//             {
//                 return FALSE;
//             }
//         }
//
//         // Get the line of data
//         CurrentOffset = IniGetNextLine(IniFileData, IniFileSize, IniFileLine, IniFileLineSize, CurrentOffset);
//         LineLength = (ULONG)strlen(IniFileLine);
//
//         // If it is a blank line or a comment then skip it
//         if (IniIsLineEmpty(IniFileLine, LineLength) || IniIsCommentLine(IniFileLine, LineLength))
//         {
//             CurrentLineNumber++;
//             continue;
//         }
//
//         // Check if it is a new section
//         if (IniIsSectionName(IniFileLine, LineLength))
//         {
//             // Allocate a new section structure
//             CurrentSection = FrLdrTempAlloc(sizeof(INI_SECTION), TAG_INI_SECTION);
//             if (!CurrentSection)
//             {
//                 FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//                 return FALSE;
//             }
//
//             RtlZeroMemory(CurrentSection, sizeof(INI_SECTION));
//
//             // Allocate the section name buffer
//             CurrentSection->SectionName = FrLdrTempAlloc(IniGetSectionNameSize(IniFileLine, LineLength) as usize, TAG_INI_NAME);
//             if (!CurrentSection->SectionName)
//             {
//                 FrLdrTempFree(CurrentSection, TAG_INI_FILE);
//                 FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//                 return FALSE;
//             }
//
//             // Get the section name
//             IniExtractSectionName(CurrentSection->SectionName, IniFileLine, LineLength);
//             InitializeListHead(&CurrentSection->SectionItemList);
//
//             // Add it to the section list head
//             IniFileSectionCount++;
//             InsertTailList(&IniFileSectionListHead, &CurrentSection->ListEntry);
//
//             CurrentLineNumber++;
//             continue;
//         }
//
//         // Check if it is a setting
//         if (IniIsSetting(IniFileLine, LineLength))
//         {
//             // First check to make sure we're inside a [section]
//             if (CurrentSection == NULL)
//             {
//                 ERR("Error: freeldr.ini:%lu: Setting '%s' found outside of a [section].\n", CurrentLineNumber, IniFileLine);
//
//                 // Skip it
//                 CurrentLineNumber++;
//                 continue;
//             }
//
//             // Allocate a new item structure
//             CurrentItem = FrLdrTempAlloc(sizeof(INI_SECTION_ITEM), TAG_INI_SECTION_ITEM);
//             if (!CurrentItem)
//             {
//                 FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//                 return FALSE;
//             }
//
//             RtlZeroMemory(CurrentItem, sizeof(INI_SECTION_ITEM));
//
//             // Allocate the setting name buffer
//             CurrentItem->ItemName = FrLdrTempAlloc(IniGetSettingNameSize(IniFileLine, LineLength) as usize, TAG_INI_NAME);
//             if (!CurrentItem->ItemName)
//             {
//                 FrLdrTempFree(CurrentItem, TAG_INI_SECTION_ITEM);
//                 FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//                 return FALSE;
//             }
//
//             // Allocate the setting value buffer
//             CurrentItem->ItemValue = FrLdrTempAlloc(IniGetSettingValueSize(IniFileLine, LineLength) as usize, TAG_INI_VALUE);
//             if (!CurrentItem->ItemValue)
//             {
//                 FrLdrTempFree(CurrentItem->ItemName, TAG_INI_NAME);
//                 FrLdrTempFree(CurrentItem, TAG_INI_SECTION_ITEM);
//                 FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//                 return FALSE;
//             }
//
//             // Get the section name
//             IniExtractSettingName(CurrentItem->ItemName, IniFileLine, LineLength);
//             IniExtractSettingValue(CurrentItem->ItemValue, IniFileLine, LineLength);
//
//             // Add it to the current section
//             IniFileSettingCount++;
//             CurrentSection->SectionItemCount++;
//             InsertTailList(&CurrentSection->SectionItemList, &CurrentItem->ListEntry);
//
//             CurrentLineNumber++;
//             continue;
//         }
//
//         CurrentLineNumber++;
//     }
//
//     FrLdrTempFree(IniFileLine as *mut c_void, TAG_INI_FILE);
//
//     TRACE("Parsed %d sections and %d settings.\n", IniFileSectionCount, IniFileSettingCount);
//     TRACE("IniParseFile() done.\n");
//
//     return TRUE;
// }
//
// ULONG IniGetNextLineSize(PCHAR IniFileData, ULONG IniFileSize, ULONG CurrentOffset)
// {
//     ULONG        LineCharCount = 0;
//
//     // Loop through counting chars until we hit the end of the
//     // file or we encounter a new line char
//     for (; (CurrentOffset < IniFileSize); CurrentOffset++)
//     {
//         // Increment the line character count
//         LineCharCount++;
//
//         // Check for new line char
//         if (IniFileData[CurrentOffset] == '\n')
//         {
//             break;
//         }
//     }
//
//     // Add one for the NULL-terminator
//     LineCharCount++;
//
//     // Send back line character count
//     return LineCharCount;
// }
//
// ULONG IniGetNextLine(PCHAR IniFileData, ULONG IniFileSize, PCHAR Buffer, ULONG BufferSize, ULONG CurrentOffset)
// {
//     ULONG        Idx;
//
//     // Loop through grabbing chars until we hit the end of the
//     // file or we encounter a new line char
//     for (Idx=0; (CurrentOffset < IniFileSize); CurrentOffset++)
//     {
//         // If we haven't exceeded our buffer size yet
//         // then store another char
//         if (Idx < (BufferSize - 1))
//         {
//             Buffer[Idx++] = IniFileData[CurrentOffset];
//         }
//
//         // Check for new line char
//         if (IniFileData[CurrentOffset] == '\n')
//         {
//             CurrentOffset++;
//             break;
//         }
//     }
//
//     // Terminate the string
//     Buffer[Idx] = '\0';
//
//     // Get rid of newline & linefeed characters (if any)
//     while(Idx && (Buffer[--Idx] == '\n' || Buffer[Idx] == '\r'))
//         Buffer[Idx] = '\0';
//
//     // Send back new offset
//     return CurrentOffset;
// }
//
// BOOLEAN IniIsLineEmpty(PCHAR LineOfText, ULONG TextLength)
// {
//     ULONG        Idx;
//
//     // Check for text (skipping whitespace)
//     for (Idx=0; Idx<TextLength; Idx++)
//     {
//         if ((LineOfText[Idx] == ' ') ||
//             (LineOfText[Idx] == '\t') ||
//             (LineOfText[Idx] == '\n') ||
//             (LineOfText[Idx] == '\r'))
//         {
//             continue;
//         }
//         else
//         {
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// BOOLEAN IniIsCommentLine(PCHAR LineOfText, ULONG TextLength)
// {
//     ULONG        Idx;
//
//     // Check the first character (skipping whitespace)
//     // and make sure that it is an opening bracket
//     for (Idx=0; Idx<TextLength; Idx++)
//     {
//         if ((LineOfText[Idx] == ' ') ||
//             (LineOfText[Idx] == '\t'))
//         {
//             continue;
//         }
//         else if (LineOfText[Idx] == INI_FILE_COMMENT_CHAR)
//         {
//             return TRUE;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// BOOLEAN IniIsSectionName(PCHAR LineOfText, ULONG TextLength)
// {
//     ULONG        Idx;
//
//     // Check the first character (skipping whitespace)
//     // and make sure that it is an opening bracket
//     for (Idx=0; Idx<TextLength; Idx++)
//     {
//         if ((LineOfText[Idx] == ' ') ||
//             (LineOfText[Idx] == '\t'))
//         {
//             continue;
//         }
//         else if (LineOfText[Idx] == '[')
//         {
//             return TRUE;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// ULONG IniGetSectionNameSize(PCHAR SectionNameLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        NameSize;
//
//     // Find the opening bracket (skipping whitespace)
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SectionNameLine[Idx] == ' ') ||
//             (SectionNameLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else //if (SectionNameLine[Idx] == '[')
//         {
//             break;
//         }
//     }
//
//     // Skip past the opening bracket
//     Idx++;
//
//     // Count the characters up until the closing bracket or EOL
//     for (NameSize=0; Idx<LineLength; Idx++)
//     {
//         if ((SectionNameLine[Idx] == ']') ||
//             (SectionNameLine[Idx] == '\0'))
//         {
//             break;
//         }
//
//         // Increment the count
//         NameSize++;
//     }
//
//     // Add one for the NULL-terminator
//     NameSize++;
//
//     return NameSize;
// }
//
// VOID IniExtractSectionName(PCHAR SectionName, PCHAR SectionNameLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        DestIdx;
//
//     // Find the opening bracket (skipping whitespace)
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SectionNameLine[Idx] == ' ') ||
//             (SectionNameLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else //if (SectionNameLine[Idx] == '[')
//         {
//             break;
//         }
//     }
//
//     // Skip past the opening bracket
//     Idx++;
//
//     // Count the characters up until the closing bracket or EOL
//     for (DestIdx=0; Idx<LineLength; Idx++)
//     {
//         if ((SectionNameLine[Idx] == ']') ||
//             (SectionNameLine[Idx] == '\0'))
//         {
//             break;
//         }
//
//         // Grab a character and increment DestIdx
//         SectionName[DestIdx] = SectionNameLine[Idx];
//         DestIdx++;
//     }
//
//     // Terminate the string
//     SectionName[DestIdx] = '\0';
// }
//
// BOOLEAN IniIsSetting(PCHAR LineOfText, ULONG TextLength)
// {
//     ULONG        Idx;
//
//     // Basically just check for an '=' equals sign
//     for (Idx=0; Idx<TextLength; Idx++)
//     {
//         if (LineOfText[Idx] == '=')
//         {
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// ULONG IniGetSettingNameSize(PCHAR SettingNameLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        NameSize;
//
//     // Skip whitespace
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingNameLine[Idx] == ' ') ||
//             (SettingNameLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     // Count the characters up until the '=' equals sign or EOL
//     for (NameSize=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingNameLine[Idx] == '=') ||
//             (SettingNameLine[Idx] == '\0'))
//         {
//             break;
//         }
//
//         // Increment the count
//         NameSize++;
//     }
//
//     // Add one for the NULL-terminator
//     NameSize++;
//
//     return NameSize;
// }
//
// ULONG IniGetSettingValueSize(PCHAR SettingValueLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        ValueSize;
//
//     // Skip whitespace
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingValueLine[Idx] == ' ') ||
//             (SettingValueLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     // Skip the characters up until the '=' equals sign or EOL
//     for (; Idx<LineLength; Idx++)
//     {
//         if (SettingValueLine[Idx] == '=')
//         {
//             Idx++;
//             break;
//         }
//
//         // If we hit EOL then obviously the value size is zero
//         if (SettingValueLine[Idx] == '\0')
//         {
//             return 0;
//         }
//     }
//
//     // Count the characters up until the EOL
//     for (ValueSize=0; Idx<LineLength; Idx++)
//     {
//         if (SettingValueLine[Idx] == '\0')
//         {
//             break;
//         }
//
//         // Increment the count
//         ValueSize++;
//     }
//
//     // Add one for the NULL-terminator
//     ValueSize++;
//
//     return ValueSize;
// }
//
// VOID IniExtractSettingName(PCHAR SettingName, PCHAR SettingNameLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        DestIdx;
//
//     // Skip whitespace
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingNameLine[Idx] == ' ') ||
//             (SettingNameLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     // Get the characters up until the '=' equals sign or EOL
//     for (DestIdx=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingNameLine[Idx] == '=') ||
//             (SettingNameLine[Idx] == '\0'))
//         {
//             break;
//         }
//
//         // Grab a character and increment DestIdx
//         SettingName[DestIdx] = SettingNameLine[Idx];
//         DestIdx++;
//     }
//
//     // Terminate the string
//     SettingName[DestIdx] = '\0';
// }
//
// VOID IniExtractSettingValue(PCHAR SettingValue, PCHAR SettingValueLine, ULONG LineLength)
// {
//     ULONG        Idx;
//     ULONG        DestIdx;
//
//     // Skip whitespace
//     for (Idx=0; Idx<LineLength; Idx++)
//     {
//         if ((SettingValueLine[Idx] == ' ') ||
//             (SettingValueLine[Idx] == '\t'))
//         {
//             continue;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     // Skip the characters up until the '=' equals sign or EOL
//     for (; Idx<LineLength; Idx++)
//     {
//         if (SettingValueLine[Idx] == '=')
//         {
//             Idx++;
//             break;
//         }
//
//         // If we hit EOL then obviously the value size is zero
//         if (SettingValueLine[Idx] == '\0')
//         {
//             SettingValue[0] = '\0';
//             return;
//         }
//     }
//
//     // Get the characters up until the EOL
//     for (DestIdx=0; Idx<LineLength; Idx++)
//     {
//         if (SettingValueLine[Idx] == '\0')
//         {
//             break;
//         }
//
//         // Grab a character and increment DestIdx
//         SettingValue[DestIdx] = SettingValueLine[Idx];
//         DestIdx++;
//     }
//
//     // Terminate the string
//     SettingValue[DestIdx] = '\0';
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
