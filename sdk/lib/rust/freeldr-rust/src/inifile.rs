#[allow(unused_imports)] use crate::{TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_INIFILE; // inifile.c: DBG_DEFAULT_CHANNEL(INIFILE)
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::PCSTR;
use core::ffi::c_void;
use crate::frldr::{
    INI_SECTION, _INI_SECTION, PINI_SECTION,
    INI_SECTION_ITEM, _INI_SECTION_ITEM, PINI_SECTION_ITEM,
    TAG_INI_SECTION, TAG_INI_SECTION_ITEM, TAG_INI_NAME, TAG_INI_VALUE,
    IniFileSectionListHead, IniFileSectionCount,
    FrLdrTempAlloc, FrLdrTempFree,
};

extern "C" {
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn strncpy(dest: PCHAR, src: PCSTR, n: usize) -> PCHAR;
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn DbgDumpBuffer(level: ULONG, buffer: PCHAR, size: ULONG);
}

#[no_mangle]
pub unsafe extern "C" fn IniGetFileSectionListHead() -> PLIST_ENTRY {
    core::ptr::addr_of_mut!(IniFileSectionListHead)
}

#[no_mangle]
pub unsafe extern "C" fn IniOpenSection(SectionName: PCSTR, SectionId: *mut ULONG_PTR) -> BOOLEAN {

    let mut Entry: PLIST_ENTRY;

    TRACE!("IniOpenSection() SectionName = %s\n", SectionName);

    Entry = IniFileSectionListHead.Flink;
    while Entry != core::ptr::addr_of_mut!(IniFileSectionListHead) {
        let Section = (Entry as *mut u8).offset(-(core::mem::offset_of!(INI_SECTION, ListEntry) as isize)) as *mut INI_SECTION;

        if _stricmp(SectionName, (*Section).SectionName) == 0 {
            if !SectionId.is_null() {
                *SectionId = Section as ULONG_PTR;
            }
            TRACE!("IniOpenSection() Found it! SectionId = 0x%x\n", SectionId);
            return 1u8;
        }

        Entry = (*Entry).Flink;
    }

    TRACE!("IniOpenSection() Section not found.\n");

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniGetNumSectionItems(SectionId: ULONG_PTR) -> ULONG {
    let Section = SectionId as PINI_SECTION;

    TRACE!("IniGetNumSectionItems() SectionId = 0x%x\n", SectionId);
    TRACE!("IniGetNumSectionItems() Item count = %d\n", (*Section).SectionItemCount);

    (*Section).SectionItemCount
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSettingByNumber(
    SectionId: ULONG_PTR,
    mut SettingNumber: ULONG,
) -> PINI_SECTION_ITEM {
    let Section = SectionId as PINI_SECTION;
    let mut Entry: PLIST_ENTRY;

    Entry = (*Section).SectionItemList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Section).SectionItemList) {
        let SectionItem = (Entry as *mut u8).offset(
            -(core::mem::offset_of!(INI_SECTION_ITEM, ListEntry) as isize)
        ) as PINI_SECTION_ITEM;

        if SettingNumber == 0 {
            return SectionItem;
        }

        SettingNumber -= 1;
        Entry = (*Entry).Flink;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSectionSettingNameSize(
    SectionId: ULONG_PTR,
    SettingIndex: ULONG,
) -> ULONG {
    let SectionItem: PINI_SECTION_ITEM;

    SectionItem = IniGetSettingByNumber(SectionId, SettingIndex);
    if SectionItem.is_null() {
        return 0;
    }

    (strlen((*SectionItem).ItemName) + 1) as ULONG
}

#[no_mangle]
pub unsafe extern "C" fn IniGetSectionSettingValueSize(
    SectionId: ULONG_PTR,
    SettingIndex: ULONG,
) -> ULONG {

    let SectionItem = IniGetSettingByNumber(SectionId, SettingIndex);
    if SectionItem.is_null() {
        return 0;
    }

    (strlen((*SectionItem).ItemValue) + 1) as ULONG
}

#[no_mangle]
pub unsafe extern "C" fn IniReadSettingByNumber(
    SectionId: ULONG_PTR,
    SettingNumber: ULONG,
    SettingName: PCHAR,
    NameSize: ULONG,
    SettingValue: PCHAR,
    ValueSize: ULONG,
) -> BOOLEAN {

    const DPRINT_INIFILE: ULONG = 0;

    TRACE!(".001 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
    TRACE!("IniReadSettingByNumber() SectionId = 0x%x\n", SectionId);

    let SectionItem = IniGetSettingByNumber(SectionId, SettingNumber);
    if SectionItem.is_null() {
        TRACE!("IniReadSettingByNumber() Setting number %d not found.\n", SettingNumber);
        return 0u8;
    }

    TRACE!("IniReadSettingByNumber() Setting number %d found.\n", SettingNumber);
    TRACE!("IniReadSettingByNumber() Setting name = %s\n", (*SectionItem).ItemName);
    TRACE!("IniReadSettingByNumber() Setting value = %s\n", (*SectionItem).ItemValue);

    TRACE!("1 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
    TRACE!("2 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
    strncpy(SettingName, (*SectionItem).ItemName, (NameSize - 1) as usize);
    *SettingName.offset((NameSize - 1) as isize) = b'\0' as i8;
    TRACE!("3 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
    strncpy(SettingValue, (*SectionItem).ItemValue, (ValueSize - 1) as usize);
    *SettingValue.offset((ValueSize - 1) as isize) = b'\0' as i8;
    TRACE!("4 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
    DbgDumpBuffer(DPRINT_INIFILE, SettingName, NameSize);
    DbgDumpBuffer(DPRINT_INIFILE, SettingValue, ValueSize);

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn IniReadSettingByName(SectionId: ULONG_PTR, SettingName: PCSTR, Buffer: PCHAR, BufferSize: ULONG) -> BOOLEAN {

    let Section = SectionId as *mut INI_SECTION;
    let mut Entry: PLIST_ENTRY;
    let mut SectionItem: *mut INI_SECTION_ITEM;

    TRACE!("IniReadSettingByName() SectionId = 0x%x\n", SectionId);

    Entry = (*Section).SectionItemList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Section).SectionItemList) {
        SectionItem = (Entry as *mut u8).offset(-(core::mem::offset_of!(INI_SECTION_ITEM, ListEntry) as isize)) as *mut INI_SECTION_ITEM;

        if _stricmp(SettingName, (*SectionItem).ItemName) == 0 {
            TRACE!("IniReadSettingByName() Setting '%s' found.\n", SettingName);
            TRACE!("IniReadSettingByName() Setting value = %s\n", (*SectionItem).ItemValue);

            strncpy(Buffer, (*SectionItem).ItemValue, BufferSize as usize - 1);
            *Buffer.offset((BufferSize - 1) as isize) = 0;

            return 1u8;
        }

        Entry = (*Entry).Flink;
    }

    WARN!("IniReadSettingByName() Setting '%s' not found.\n", SettingName);

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn IniAddSection(SectionName: PCSTR, SectionId: *mut ULONG_PTR) -> BOOLEAN {

    let Section: PINI_SECTION = FrLdrTempAlloc(
        core::mem::size_of::<INI_SECTION>(),
        TAG_INI_SECTION
    ) as PINI_SECTION;
    
    if Section.is_null() {
        return 0u8;
    }

    core::ptr::write_bytes(Section, 0, 1);

    (*Section).SectionName = FrLdrTempAlloc(
        strlen(SectionName) + core::mem::size_of::<i8>(),
        TAG_INI_NAME
    ) as *mut i8;
    
    if (*Section).SectionName.is_null() {
        FrLdrTempFree(Section as *mut core::ffi::c_void, TAG_INI_SECTION);
        return 0u8;
    }

    strcpy((*Section).SectionName, SectionName);
    InitializeListHead(core::ptr::addr_of_mut!((*Section).SectionItemList));

    IniFileSectionCount += 1;
    InsertHeadList(
        core::ptr::addr_of_mut!(IniFileSectionListHead),
        core::ptr::addr_of_mut!((*Section).ListEntry)
    );

    *SectionId = Section as ULONG_PTR;

    1u8
}

pub unsafe fn IniFreeSection(Section: PINI_SECTION) {

    let mut ListEntry: PLIST_ENTRY;
    let mut SectionItem: PINI_SECTION_ITEM;

    while !IsListEmpty(core::ptr::addr_of_mut!((*Section).SectionItemList)) {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*Section).SectionItemList));
        SectionItem = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(_INI_SECTION_ITEM, ListEntry) as isize)) as PINI_SECTION_ITEM;

        FrLdrTempFree((*SectionItem).ItemName as *mut core::ffi::c_void, TAG_INI_NAME);
        FrLdrTempFree((*SectionItem).ItemValue as *mut core::ffi::c_void, TAG_INI_VALUE);
        FrLdrTempFree(SectionItem as *mut core::ffi::c_void, TAG_INI_SECTION_ITEM);
    }

    FrLdrTempFree((*Section).SectionName as *mut core::ffi::c_void, TAG_INI_NAME);
    FrLdrTempFree(Section as *mut core::ffi::c_void, TAG_INI_SECTION);
}

#[no_mangle]
pub unsafe extern "C" fn IniCleanup() {
    let mut ListEntry: PLIST_ENTRY;
    let mut Section: PINI_SECTION;

    while !IsListEmpty(core::ptr::addr_of_mut!(IniFileSectionListHead)) {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(IniFileSectionListHead));
        Section = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(_INI_SECTION, ListEntry) as isize)) as PINI_SECTION;

        IniFreeSection(Section);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IniAddSettingValueToSection(SectionId: usize, SettingName: PCSTR, SettingValue: PCSTR) -> u8 {
    let Section = SectionId as *mut _INI_SECTION;
    let SectionItem: *mut _INI_SECTION_ITEM;

    SectionItem = FrLdrTempAlloc(core::mem::size_of::<_INI_SECTION_ITEM>(), TAG_INI_SECTION_ITEM) as *mut _INI_SECTION_ITEM;
    if SectionItem.is_null() {
        return 0u8;
    }

    core::ptr::write_bytes(SectionItem, 0, 1);

    (*SectionItem).ItemName = FrLdrTempAlloc(strlen(SettingName) + 1, TAG_INI_NAME) as *mut i8;
    if (*SectionItem).ItemName.is_null() {
        FrLdrTempFree(SectionItem as *mut core::ffi::c_void, TAG_INI_SECTION_ITEM);
        return 0u8;
    }

    (*SectionItem).ItemValue = FrLdrTempAlloc(strlen(SettingValue) + 1, TAG_INI_VALUE) as *mut i8;
    if (*SectionItem).ItemValue.is_null() {
        FrLdrTempFree((*SectionItem).ItemName as *mut core::ffi::c_void, TAG_INI_NAME);
        FrLdrTempFree(SectionItem as *mut core::ffi::c_void, TAG_INI_SECTION_ITEM);
        return 0u8;
    }

    strcpy((*SectionItem).ItemName, SettingName);
    strcpy((*SectionItem).ItemValue, SettingValue);

    (*Section).SectionItemCount += 1;
    InsertTailList(core::ptr::addr_of_mut!((*Section).SectionItemList), core::ptr::addr_of_mut!((*SectionItem).ListEntry));

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn IniModifySettingValue(SectionId: usize, SettingName: PCSTR, SettingValue: PCSTR) -> u8 {

    const TAG_INI_VALUE: u32 = u32::from_be_bytes(*b"VinI"); // C literal 'VinI' -- boot/freeldr/freeldr/include/inifile.h

    let Section = SectionId as *mut _INI_SECTION;
    let mut Entry: PLIST_ENTRY;
    let mut SectionItem: *mut _INI_SECTION_ITEM = core::ptr::null_mut();
    let NewItemValue: *mut i8;

    Entry = (*Section).SectionItemList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Section).SectionItemList) {
        SectionItem = (Entry as *mut u8).offset(-(core::mem::offset_of!(_INI_SECTION_ITEM, ListEntry) as isize)) as *mut _INI_SECTION_ITEM;

        if _stricmp((*SectionItem).ItemName, SettingName) == 0 {
            break;
        }

        Entry = (*Entry).Flink;
    }

    if Entry == core::ptr::addr_of_mut!((*Section).SectionItemList) {
        return IniAddSettingValueToSection(SectionId, SettingName, SettingValue);
    }

    NewItemValue = FrLdrTempAlloc(strlen(SettingValue) + 1, TAG_INI_VALUE) as *mut i8;
    if NewItemValue.is_null() {
        return 0u8;
    }
    FrLdrTempFree((*SectionItem).ItemValue as *mut core::ffi::c_void, TAG_INI_VALUE);
    (*SectionItem).ItemValue = NewItemValue;

    strcpy((*SectionItem).ItemValue, SettingValue);

    return 1u8;
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
// PLIST_ENTRY IniGetFileSectionListHead(VOID)
// {
//     return &IniFileSectionListHead;
// }
//
// BOOLEAN IniOpenSection(PCSTR SectionName, ULONG_PTR* SectionId)
// {
//     PLIST_ENTRY Entry;
//     PINI_SECTION Section;
//
//     TRACE("IniOpenSection() SectionName = %s\n", SectionName);
//
//     // Loop through each section and find the one we want
//     for (Entry = IniFileSectionListHead.Flink;
//          Entry != &IniFileSectionListHead;
//          Entry = Entry->Flink)
//     {
//         Section = CONTAINING_RECORD(Entry, INI_SECTION, ListEntry);
//
//         // Compare against the section name
//         if (_stricmp(SectionName, Section->SectionName) == 0)
//         {
//             // We found it
//             if (SectionId)
//                 *SectionId = (ULONG_PTR)Section;
//             TRACE("IniOpenSection() Found it! SectionId = 0x%x\n", SectionId);
//             return TRUE;
//         }
//     }
//
//     TRACE("IniOpenSection() Section not found.\n");
//
//     return FALSE;
// }
//
// ULONG IniGetNumSectionItems(ULONG_PTR SectionId)
// {
//     PINI_SECTION Section = (PINI_SECTION)SectionId;
//
//     TRACE("IniGetNumSectionItems() SectionId = 0x%x\n", SectionId);
//     TRACE("IniGetNumSectionItems() Item count = %d\n", Section->SectionItemCount);
//
//     return Section->SectionItemCount;
// }
//
// PINI_SECTION_ITEM IniGetSettingByNumber(ULONG_PTR SectionId, ULONG SettingNumber)
// {
//     PINI_SECTION Section = (PINI_SECTION)SectionId;
//     PLIST_ENTRY Entry;
//     PINI_SECTION_ITEM SectionItem;
//
//     // Loop through each section item and find the one we want
//     for (Entry = Section->SectionItemList.Flink;
//          Entry != &Section->SectionItemList;
//          Entry = Entry->Flink)
//     {
//         SectionItem = CONTAINING_RECORD(Entry, INI_SECTION_ITEM, ListEntry);
//
//         // Check to see if this is the setting we want
//         if (SettingNumber == 0)
//         {
//             return SectionItem;
//         }
//
//         // Nope, keep going
//         SettingNumber--;
//     }
//     return NULL;
// }
//
// ULONG IniGetSectionSettingNameSize(ULONG_PTR SectionId, ULONG SettingIndex)
// {
//     PINI_SECTION_ITEM SectionItem;
//
//     // Retrieve requested setting
//     SectionItem = IniGetSettingByNumber(SectionId, SettingIndex);
//     if (!SectionItem)
//         return 0;
//
//     // Return the size of the string plus 1 for the null-terminator
//     return (ULONG)(strlen(SectionItem->ItemName) + 1);
// }
//
// ULONG IniGetSectionSettingValueSize(ULONG_PTR SectionId, ULONG SettingIndex)
// {
//     PINI_SECTION_ITEM SectionItem;
//
//     // Retrieve requested setting
//     SectionItem = IniGetSettingByNumber(SectionId, SettingIndex);
//     if (!SectionItem)
//         return 0;
//
//     // Return the size of the string plus 1 for the null-terminator
//     return (ULONG)(strlen(SectionItem->ItemValue) + 1);
// }
//
// BOOLEAN IniReadSettingByNumber(ULONG_PTR SectionId, ULONG SettingNumber, PCHAR SettingName, ULONG NameSize, PCHAR SettingValue, ULONG ValueSize)
// {
//     PINI_SECTION_ITEM    SectionItem;
//     TRACE(".001 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
//
//     TRACE("IniReadSettingByNumber() SectionId = 0x%x\n", SectionId);
//
//     // Retrieve requested setting
//     SectionItem = IniGetSettingByNumber(SectionId, SettingNumber);
//     if (!SectionItem)
//     {
//         TRACE("IniReadSettingByNumber() Setting number %d not found.\n", SettingNumber);
//         return FALSE;
//     }
//
//     TRACE("IniReadSettingByNumber() Setting number %d found.\n", SettingNumber);
//     TRACE("IniReadSettingByNumber() Setting name = %s\n", SectionItem->ItemName);
//     TRACE("IniReadSettingByNumber() Setting value = %s\n", SectionItem->ItemValue);
//
//     TRACE("1 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
//     TRACE("2 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
//     strncpy(SettingName, SectionItem->ItemName, NameSize - 1);
//     SettingName[NameSize - 1] = '\0';
//     TRACE("3 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
//     strncpy(SettingValue, SectionItem->ItemValue, ValueSize - 1);
//     SettingValue[ValueSize - 1] = '\0';
//     TRACE("4 NameSize = %d ValueSize = %d\n", NameSize, ValueSize);
//     DbgDumpBuffer(DPRINT_INIFILE, SettingName, NameSize);
//     DbgDumpBuffer(DPRINT_INIFILE, SettingValue, ValueSize);
//
//     return TRUE;
// }
//
// BOOLEAN IniReadSettingByName(ULONG_PTR SectionId, PCSTR SettingName, PCHAR Buffer, ULONG BufferSize)
// {
//     PINI_SECTION Section = (PINI_SECTION)SectionId;
//     PLIST_ENTRY Entry;
//     PINI_SECTION_ITEM SectionItem;
//
//     TRACE("IniReadSettingByName() SectionId = 0x%x\n", SectionId);
//
//     // Loop through each section item and find the one we want
//     for (Entry = Section->SectionItemList.Flink;
//          Entry != &Section->SectionItemList;
//          Entry = Entry->Flink)
//     {
//         SectionItem = CONTAINING_RECORD(Entry, INI_SECTION_ITEM, ListEntry);
//
//         // Check to see if this is the setting we want
//         if (_stricmp(SettingName, SectionItem->ItemName) == 0)
//         {
//             TRACE("IniReadSettingByName() Setting \'%s\' found.\n", SettingName);
//             TRACE("IniReadSettingByName() Setting value = %s\n", SectionItem->ItemValue);
//
//             strncpy(Buffer, SectionItem->ItemValue, BufferSize - 1);
//             Buffer[BufferSize - 1] = '\0';
//
//             return TRUE;
//         }
//     }
//
//     WARN("IniReadSettingByName() Setting \'%s\' not found.\n", SettingName);
//
//     return FALSE;
// }
//
// BOOLEAN IniAddSection(PCSTR SectionName, ULONG_PTR* SectionId)
// {
//     PINI_SECTION Section;
//
//     // Allocate a new section structure
//     Section = FrLdrTempAlloc(sizeof(INI_SECTION), TAG_INI_SECTION);
//     if (!Section)
//     {
//         return FALSE;
//     }
//
//     RtlZeroMemory(Section, sizeof(INI_SECTION));
//
//     // Allocate the section name buffer
//     Section->SectionName = FrLdrTempAlloc(strlen(SectionName) + sizeof(CHAR), TAG_INI_NAME);
//     if (!Section->SectionName)
//     {
//         FrLdrTempFree(Section, TAG_INI_SECTION);
//         return FALSE;
//     }
//
//     // Get the section name
//     strcpy(Section->SectionName, SectionName);
//     InitializeListHead(&Section->SectionItemList);
//
//     // Add it to the section list head
//     IniFileSectionCount++;
//     InsertHeadList(&IniFileSectionListHead, &Section->ListEntry);
//
//     *SectionId = (ULONG_PTR)Section;
//
//     return TRUE;
// }
//
// VOID IniFreeSection(PINI_SECTION Section)
// {
//     PLIST_ENTRY ListEntry;
//     PINI_SECTION_ITEM SectionItem;
//
//     // Loop while there are section items
//     while (!IsListEmpty(&Section->SectionItemList))
//     {
//         // Remove the section item
//         ListEntry = RemoveHeadList(&Section->SectionItemList);
//         SectionItem = CONTAINING_RECORD(ListEntry, INI_SECTION_ITEM, ListEntry);
//
//         // Free it
//         FrLdrTempFree(SectionItem->ItemName, TAG_INI_NAME);
//         FrLdrTempFree(SectionItem->ItemValue, TAG_INI_VALUE);
//         FrLdrTempFree(SectionItem, TAG_INI_SECTION_ITEM);
//     }
//
//     FrLdrTempFree(Section->SectionName, TAG_INI_NAME);
//     FrLdrTempFree(Section, TAG_INI_SECTION);
// }
//
// VOID IniCleanup(VOID)
// {
//     PLIST_ENTRY ListEntry;
//     PINI_SECTION Section;
//
//     // Loop while there are sections
//     while (!IsListEmpty(&IniFileSectionListHead))
//     {
//         // Remove the section
//         ListEntry = RemoveHeadList(&IniFileSectionListHead);
//         Section = CONTAINING_RECORD(ListEntry, INI_SECTION, ListEntry);
//
//         // Free it
//         IniFreeSection(Section);
//     }
// }
//
// BOOLEAN IniAddSettingValueToSection(ULONG_PTR SectionId, PCSTR SettingName, PCSTR SettingValue)
// {
//     PINI_SECTION Section = (PINI_SECTION)SectionId;
//     PINI_SECTION_ITEM SectionItem;
//
//     // Allocate a new item structure
//     SectionItem = FrLdrTempAlloc(sizeof(INI_SECTION_ITEM), TAG_INI_SECTION_ITEM);
//     if (!SectionItem)
//     {
//         return FALSE;
//     }
//
//     RtlZeroMemory(SectionItem, sizeof(INI_SECTION_ITEM));
//
//     // Allocate the setting name buffer
//     SectionItem->ItemName = FrLdrTempAlloc(strlen(SettingName) + 1, TAG_INI_NAME);
//     if (!SectionItem->ItemName)
//     {
//         FrLdrTempFree(SectionItem, TAG_INI_SECTION_ITEM);
//         return FALSE;
//     }
//
//     // Allocate the setting value buffer
//     SectionItem->ItemValue = FrLdrTempAlloc(strlen(SettingValue) + 1, TAG_INI_VALUE);
//     if (!SectionItem->ItemValue)
//     {
//         FrLdrTempFree(SectionItem->ItemName, TAG_INI_NAME);
//         FrLdrTempFree(SectionItem, TAG_INI_SECTION_ITEM);
//         return FALSE;
//     }
//
//     strcpy(SectionItem->ItemName, SettingName);
//     strcpy(SectionItem->ItemValue, SettingValue);
//
//     // Add it to the current section
//     Section->SectionItemCount++;
//     InsertTailList(&Section->SectionItemList, &SectionItem->ListEntry);
//
//     return TRUE;
// }
//
// BOOLEAN IniModifySettingValue(ULONG_PTR SectionId, PCSTR SettingName, PCSTR SettingValue)
// {
//     PINI_SECTION Section = (PINI_SECTION)SectionId;
//     PLIST_ENTRY Entry;
//     PINI_SECTION_ITEM SectionItem;
//     PCHAR NewItemValue;
//
//     // Loop through each section item and find the one we want
//     for (Entry = Section->SectionItemList.Flink;
//          Entry != &Section->SectionItemList;
//          Entry = Entry->Flink)
//     {
//         SectionItem = CONTAINING_RECORD(Entry, INI_SECTION_ITEM, ListEntry);
//
//         // Check to see if this is the setting we want
//         if (_stricmp(SectionItem->ItemName, SettingName) == 0)
//         {
//             break;
//         }
//         // Nope, keep going
//     }
//     // If the section item does not exist, create it
//     if (Entry == &Section->SectionItemList)
//     {
//         return IniAddSettingValueToSection(SectionId, SettingName, SettingValue);
//     }
//
//     // Reallocate the new setting value buffer
//     NewItemValue = FrLdrTempAlloc(strlen(SettingValue) + 1, TAG_INI_VALUE);
//     if (!NewItemValue)
//     {
//         // We failed, bail out
//         return FALSE;
//     }
//     FrLdrTempFree(SectionItem->ItemValue, TAG_INI_VALUE);
//     SectionItem->ItemValue = NewItemValue;
//
//     strcpy(SectionItem->ItemValue, SettingValue);
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
