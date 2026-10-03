use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR, PSTR, TIMEINFO};
use core::ffi::c_void;
// ARC constants live in crate::ffi (rustconv catalog).
use crate::ffi::{ESUCCESS, OpenReadOnly};

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    fn FrLdrHeapAlloc(size: usize, tag: u32) -> PSTR;
    fn strlen(s: PCSTR) -> usize;
    fn strcpy(dest: PSTR, src: PCSTR) -> PSTR;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn ArcOpen(Path: PSTR, OpenMode: u32, FileId: *mut u32) -> u32;
    fn ArcClose(FileId: u32) -> u32;
    fn ArcGetTime() -> *mut TIMEINFO;
    fn GetBootMgrInfo() -> *mut BOOTMGRINFO;
    fn IniOpenSection(SectionName: PCSTR, SectionId: *mut usize) -> BOOLEAN;
    fn IniGetNumSectionItems(SectionId: usize) -> u32;
    fn IniReadSettingByNumber(
        SectionId: usize, SettingNumber: u32,
        SettingName: PCHAR, NameSize: ULONG,
        SettingValue: PCHAR, ValueSize: ULONG) -> BOOLEAN;
    fn IniReadSettingByName(SectionId: usize, SettingName: PCSTR, Buffer: PCHAR, BufferSize: ULONG) -> BOOLEAN;
    fn IniAddSection(SectionName: PCSTR, SectionId: *mut usize) -> BOOLEAN;
    fn IniAddSettingValueToSection(SectionId: usize, SettingName: PCSTR, SettingValue: PCSTR) -> BOOLEAN;
    fn IniModifySettingValue(SectionId: usize, SettingName: PCSTR, SettingValue: PCSTR) -> BOOLEAN;
    fn RtlStringCbCopyA(dest: PCHAR, cb_dest: usize, src: PCSTR) -> NTSTATUS;
    fn RtlStringCbCatA(dest: PCHAR, cb_dest: usize, src: PCSTR) -> NTSTATUS;
    fn RtlStringCbPrintfA(dest: PCHAR, cb_dest: usize, fmt: PCSTR, ...) -> NTSTATUS;
}

// oslist.h defines `typedef struct tagOperatingSystemItem { ULONG_PTR
// SectionId; PCSTR LoadIdentifier; }` — public, used by callers (winldr,
// options).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct OperatingSystemItem {
    pub SectionId: usize,
    pub LoadIdentifier: PCSTR,
}

// ---- file-scope preamble (translated) ----
pub const TAG_OS_ITEM: u32 = u32::from_ne_bytes(*b"tISO");

unsafe fn CopyString(Source: PCSTR) -> PCSTR {
    let Dest: PSTR;

    if Source.is_null() {
        return core::ptr::null();
    }

    Dest = FrLdrHeapAlloc(strlen(Source) + 1, TAG_STRING);
    if !Dest.is_null() {
        strcpy(Dest, Source);
    }

    return Dest as PCSTR;
}

const TAG_STRING: u32 = u32::from_be_bytes(*b" rtS"); // nolint: const-contradiction (' rtS' -- freeldr/include/oslist.h:22; win32k's TAG_STRING is ' RTS')

#[no_mangle]
pub unsafe extern "C" fn InitOperatingSystemList(
    OperatingSystemCount: *mut u32,
    DefaultOperatingSystem: *mut u32,
) -> *mut OperatingSystemItem {
    let Items: *mut OperatingSystemItem;
    let mut DefaultOSName: PCSTR;
    let mut DefaultOS: u32 = 0;
    let Count: u32;
    let mut i: u32;
    let mut OsSectionId: usize = 0;
    let mut SectionId: usize;
    let mut TitleStart: *mut i8;
    let mut TitleEnd: *mut i8;
    let mut OsLoadOptions: PCSTR;
    let mut HadSection: u8;
    let mut HadNoBootType: u8;
    let mut SettingName: [i8; 260] = [0; 260];
    let mut SettingValue: [i8; 260] = [0; 260];
    let mut BootType: [i8; 80] = [0; 80];
    let mut TempBuffer: [i8; 260] = [0; 260];

    if IniOpenSection(b"Operating Systems\0".as_ptr() as PCSTR, &mut OsSectionId) == 0 {
        return core::ptr::null_mut();
    }

    Count = IniGetNumSectionItems(OsSectionId);

    Items = FrLdrHeapAlloc((Count as usize) * core::mem::size_of::<OperatingSystemItem>(), TAG_OS_ITEM) as *mut OperatingSystemItem;
    if Items.is_null() {
        return core::ptr::null_mut();
    }

    DefaultOSName = (*GetBootMgrInfo()).DefaultOs;

    i = 0;
    while i < Count {
        IniReadSettingByNumber(OsSectionId, i,
                               SettingName.as_mut_ptr(), core::mem::size_of_val(&SettingName) as ULONG,
                               SettingValue.as_mut_ptr(), core::mem::size_of_val(&SettingValue) as ULONG);
        if *SettingName.as_ptr() == 0 {
            ERR(b"Invalid OS entry %lu, skipping.\n\0".as_ptr() as PCSTR, i);
            i += 1;
            continue;
        }

        TitleStart = SettingValue.as_mut_ptr();
        while *TitleStart == b' ' as i8 || *TitleStart == b'\t' as i8 || *TitleStart == b'"' as i8 {
            TitleStart = TitleStart.offset(1);
        }
        TitleEnd = TitleStart;
        while *TitleEnd != 0 && *TitleEnd != b'"' as i8 {
            TitleEnd = TitleEnd.offset(1);
        }

        if *TitleEnd != 0 {
            *TitleEnd = 0;
            TitleEnd = TitleEnd.offset(1);
        }

        if *TitleEnd != 0 {
            while *TitleEnd == b' ' as i8 || *TitleEnd == b'\t' as i8 || *TitleEnd == b'"' as i8 {
                TitleEnd = TitleEnd.offset(1);
            }
        }
        OsLoadOptions = if *TitleEnd != 0 { TitleEnd as PCSTR } else { core::ptr::null() };

        if !DefaultOSName.is_null() && _stricmp(DefaultOSName, SettingName.as_ptr()) == 0 {
            DefaultOS = i;
            DefaultOSName = core::ptr::null();
        }

        SectionId = 0;
        HadSection = IniOpenSection(SettingName.as_ptr(), &mut SectionId);
        if HadSection != 0 {
            // Reset first (upstream 81752ca2117): a section with no BootType= line leaves the
            // buffer untouched, and the previous entry's BootType would carry over.
            *BootType.as_mut_ptr() = 0i8;
            IniReadSettingByName(SectionId, b"BootType\0".as_ptr() as PCSTR, BootType.as_mut_ptr(), core::mem::size_of_val(&BootType) as ULONG);
        } else {
            *BootType.as_mut_ptr() = 0i8;
        }

        HadNoBootType = if *BootType.as_ptr() == 0 { 1 } else { 0 };
        if HadNoBootType != 0 {
            #[cfg(target_arch = "x86")]
            {
                let mut FileId: u32 = 0;
                if ArcOpen(SettingName.as_ptr() as PSTR, OpenReadOnly, &mut FileId) == ESUCCESS {
                    ArcClose(FileId);
                    strcpy(BootType.as_mut_ptr(), b"BootSector\0".as_ptr() as *const i8);
                } else {
                    strcpy(BootType.as_mut_ptr(), b"Windows\0".as_ptr() as *const i8);
                }
            }
            #[cfg(not(target_arch = "x86"))]
            {
                strcpy(BootType.as_mut_ptr(), b"Windows\0".as_ptr() as *const i8);
            }
        }

        if HadSection == 0 {
            let TimeInfo: *mut TIMEINFO;

            RtlStringCbCopyA(TempBuffer.as_mut_ptr(), core::mem::size_of_val(&TempBuffer), SettingName.as_ptr());

            TimeInfo = ArcGetTime();
            if _stricmp(BootType.as_ptr(), b"BootSector\0".as_ptr() as *const i8) == 0 {
                RtlStringCbPrintfA(SettingName.as_mut_ptr(), core::mem::size_of_val(&SettingName),
                                   b"BootSectorFile%u%u%u%u%u%u\0".as_ptr() as PCSTR,
                                   (*TimeInfo).Year as u32, (*TimeInfo).Day as u32, (*TimeInfo).Month as u32,
                                   (*TimeInfo).Hour as u32, (*TimeInfo).Minute as u32, (*TimeInfo).Second as u32);
            } else if _stricmp(BootType.as_ptr(), b"Windows\0".as_ptr() as *const i8) == 0 {
                RtlStringCbPrintfA(SettingName.as_mut_ptr(), core::mem::size_of_val(&SettingName),
                                   b"Windows%u%u%u%u%u%u\0".as_ptr() as PCSTR,
                                   (*TimeInfo).Year as u32, (*TimeInfo).Day as u32, (*TimeInfo).Month as u32,
                                   (*TimeInfo).Hour as u32, (*TimeInfo).Minute as u32, (*TimeInfo).Second as u32);
            }

            if IniAddSection(SettingName.as_ptr(), &mut SectionId) == 0 {
                ERR(b"Could not convert legacy OS entry %lu, skipping.\n\0".as_ptr() as PCSTR, i);
                i += 1;
                continue;
            }

            if _stricmp(BootType.as_ptr(), b"BootSector\0".as_ptr() as *const i8) == 0 {
                if IniAddSettingValueToSection(SectionId, b"BootSectorFile\0".as_ptr() as PCSTR, TempBuffer.as_ptr()) == 0 {
                    ERR(b"Could not convert legacy OS entry %lu, skipping.\n\0".as_ptr() as PCSTR, i);
                    i += 1;
                    continue;
                }
            } else if _stricmp(BootType.as_ptr(), b"Windows\0".as_ptr() as *const i8) == 0 {
                if IniAddSettingValueToSection(SectionId, b"SystemPath\0".as_ptr() as PCSTR, TempBuffer.as_ptr()) == 0 {
                    ERR(b"Could not convert legacy OS entry %lu, skipping.\n\0".as_ptr() as PCSTR, i);
                    i += 1;
                    continue;
                }
            }

            if !OsLoadOptions.is_null() && IniAddSettingValueToSection(SectionId, b"Options\0".as_ptr() as PCSTR, OsLoadOptions) == 0 {
                ERR(b"Could not convert legacy OS entry %lu, skipping.\n\0".as_ptr() as PCSTR, i);
                i += 1;
                continue;
            }
        }

        if HadNoBootType != 0 && IniModifySettingValue(SectionId, b"BootType\0".as_ptr() as PCSTR, BootType.as_ptr()) == 0 {
            ERR(b"Could not fixup the BootType entry for OS '%s', ignoring.\n\0".as_ptr() as PCSTR, SettingName.as_ptr());
        }

        if HadSection != 0 && !OsLoadOptions.is_null() && *OsLoadOptions != 0 {
            *TempBuffer.as_mut_ptr() = 0i8;
            if IniReadSettingByName(SectionId, b"Options\0".as_ptr() as PCSTR, TempBuffer.as_mut_ptr(), core::mem::size_of_val(&TempBuffer) as ULONG) == 0 {
                TRACE(b"No 'Options' value found for OS '%s', ignoring.\n\0".as_ptr() as PCSTR, SettingName.as_ptr());
            }

            RtlStringCbCatA(TempBuffer.as_mut_ptr(), core::mem::size_of_val(&TempBuffer), b" \0".as_ptr() as PCSTR);
            RtlStringCbCatA(TempBuffer.as_mut_ptr(), core::mem::size_of_val(&TempBuffer), OsLoadOptions);

            if IniModifySettingValue(SectionId, b"Options\0".as_ptr() as PCSTR, TempBuffer.as_ptr()) == 0 {
                ERR(b"Could not modify the options for OS '%s', ignoring.\n\0".as_ptr() as PCSTR, SettingName.as_ptr());
            }
        }

        (*Items.offset(i as isize)).SectionId = SectionId;
        (*Items.offset(i as isize)).LoadIdentifier = CopyString(TitleStart as PCSTR);

        i += 1;
    }

    *OperatingSystemCount = Count;
    *DefaultOperatingSystem = DefaultOS;
    return Items;
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
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(INIFILE);
//
// #define TAG_OS_ITEM 'tISO'
//
// /* FUNCTIONS ******************************************************************/
//
// static PCSTR CopyString(PCSTR Source)
// {
//     PSTR Dest;
//
//     if (!Source)
//         return NULL;
//
//     Dest = FrLdrHeapAlloc(strlen(Source) + 1, TAG_STRING);
//     if (Dest)
//         strcpy(Dest, Source);
//
//     return Dest;
// }
//
// OperatingSystemItem*
// InitOperatingSystemList(
//     _Out_ PULONG OperatingSystemCount,
//     _Out_ PULONG DefaultOperatingSystem)
// {
//     OperatingSystemItem* Items;
//     PCSTR DefaultOSName;
//     ULONG DefaultOS = 0;
//     ULONG Count;
//     ULONG i;
//     ULONG_PTR OsSectionId, SectionId;
//     PCHAR TitleStart, TitleEnd;
//     PCSTR OsLoadOptions;
//     BOOLEAN HadSection;
//     BOOLEAN HadNoBootType;
//     CHAR SettingName[260];
//     CHAR SettingValue[260];
//     CHAR BootType[80];
//     CHAR TempBuffer[_countof(SettingValue)];
//
//     /* Open the [Operating Systems] section */
//     if (!IniOpenSection("Operating Systems", &OsSectionId))
//         return NULL;
//
//     /* Count the number of operating systems in the section */
//     Count = IniGetNumSectionItems(OsSectionId);
//
//     /* Allocate memory to hold operating system lists */
//     Items = FrLdrHeapAlloc(Count * sizeof(OperatingSystemItem), TAG_OS_ITEM);
//     if (!Items)
//         return NULL;
//
//     /* Retrieve the default OS */
//     DefaultOSName = GetBootMgrInfo()->DefaultOs;
//
//     /* Now loop through the operating system section and load each item */
//     for (i = 0; i < Count; ++i)
//     {
//         IniReadSettingByNumber(OsSectionId, i,
//                                SettingName, sizeof(SettingName),
//                                SettingValue, sizeof(SettingValue));
//         if (!*SettingName)
//         {
//             ERR("Invalid OS entry %lu, skipping.\n", i);
//             continue;
//         }
//
//         /* Retrieve the start and end of the title */
//         TitleStart = SettingValue;
//         /* Trim any leading whitespace and quotes */
//         while (*TitleStart == ' ' || *TitleStart == '\t' || *TitleStart == '"')
//             ++TitleStart;
//         TitleEnd = TitleStart;
//         /* Go up to the first last quote */
//         while (*TitleEnd != ANSI_NULL && *TitleEnd != '"')
//             ++TitleEnd;
//
//         /* NULL-terminate the title */
//         if (*TitleEnd)
//             *TitleEnd++ = ANSI_NULL; // Skip the quote too.
//
//         /* Retrieve the options after the quoted title */
//         if (*TitleEnd)
//         {
//             /* Trim any trailing whitespace and quotes */
//             while (*TitleEnd == ' ' || *TitleEnd == '\t' || *TitleEnd == '"')
//                 ++TitleEnd;
//         }
//         OsLoadOptions = (*TitleEnd ? TitleEnd : NULL);
//
//         // TRACE("\n"
//               // "SettingName   = '%s'\n"
//               // "TitleStart    = '%s'\n"
//               // "OsLoadOptions = '%s'\n",
//               // SettingName, TitleStart, OsLoadOptions);
//
//         /* Find the default OS item while we haven't got one */
//         if (DefaultOSName && _stricmp(DefaultOSName, SettingName) == 0)
//         {
//             DefaultOS = i;
//             DefaultOSName = NULL; // We have found the first one, don't search for others.
//         }
//
//         /*
//          * Determine whether this is a legacy operating system entry of the form:
//          *
//          * [Operating Systems]
//          * ArcOsLoadPartition="LoadIdentifier" /List /of /Options
//          *
//          * and if so, convert it into a new operating system INI entry:
//          *
//          * [Operating Systems]
//          * SectionIdentifier="LoadIdentifier"
//          *
//          * [SectionIdentifier]
//          * BootType=...
//          * SystemPath=ArcOsLoadPartition
//          * Options=/List /of /Options
//          *
//          * The "BootType" value is heuristically determined from the form of
//          * the ArcOsLoadPartition: if this is an ARC path, the "BootType" value
//          * is "Windows", otherwise if this is a DOS path the "BootType" value
//          * is "BootSector". This ensures backwards-compatibility with NTLDR.
//          */
//
//         /* Try to open the operating system section in the .ini file */
//         SectionId = 0;
//         HadSection = IniOpenSection(SettingName, &SectionId);
//         if (HadSection)
//         {
//             /* This is a new OS entry: try to read the boot type */
//             IniReadSettingByName(SectionId, "BootType", BootType, sizeof(BootType));
//         }
//         else
//         {
//             /* This is a legacy OS entry: no explicit BootType specified, we will infer one */
//             *BootType = ANSI_NULL;
//         }
//
//         /* Check whether we have got a BootType value; if not, try to infer one */
//         HadNoBootType = (*BootType == ANSI_NULL);
//         if (HadNoBootType)
//         {
// #ifdef _M_IX86
//             ULONG FileId;
//             if (ArcOpen(SettingName, OpenReadOnly, &FileId) == ESUCCESS)
//             {
//                 ArcClose(FileId);
//                 strcpy(BootType, "BootSector");
//             }
//             else
// #endif
//             {
//                 strcpy(BootType, "Windows");
//             }
//         }
//
//         /* This is a legacy OS entry: convert it into a new OS entry */
//         if (!HadSection)
//         {
//             TIMEINFO* TimeInfo;
//
//             /* Save the system path from the original SettingName (overwritten below) */
//             RtlStringCbCopyA(TempBuffer, sizeof(TempBuffer), SettingName);
//
//             /* Generate a unique section name */
//             TimeInfo = ArcGetTime();
//             if (_stricmp(BootType, "BootSector") == 0)
//             {
//                 RtlStringCbPrintfA(SettingName, sizeof(SettingName),
//                                    "BootSectorFile%u%u%u%u%u%u",
//                                    TimeInfo->Year, TimeInfo->Day, TimeInfo->Month,
//                                    TimeInfo->Hour, TimeInfo->Minute, TimeInfo->Second);
//             }
//             else if (_stricmp(BootType, "Windows") == 0)
//             {
//                 RtlStringCbPrintfA(SettingName, sizeof(SettingName),
//                                    "Windows%u%u%u%u%u%u",
//                                    TimeInfo->Year, TimeInfo->Day, TimeInfo->Month,
//                                    TimeInfo->Hour, TimeInfo->Minute, TimeInfo->Second);
//             }
//             else
//             {
//                 ASSERT(FALSE);
//             }
//
//             /* Add the section */
//             if (!IniAddSection(SettingName, &SectionId))
//             {
//                 ERR("Could not convert legacy OS entry %lu, skipping.\n", i);
//                 continue;
//             }
//
//             /* Add the system path */
//             if (_stricmp(BootType, "BootSector") == 0)
//             {
//                 if (!IniAddSettingValueToSection(SectionId, "BootSectorFile", TempBuffer))
//                 {
//                     ERR("Could not convert legacy OS entry %lu, skipping.\n", i);
//                     continue;
//                 }
//             }
//             else if (_stricmp(BootType, "Windows") == 0)
//             {
//                 if (!IniAddSettingValueToSection(SectionId, "SystemPath", TempBuffer))
//                 {
//                     ERR("Could not convert legacy OS entry %lu, skipping.\n", i);
//                     continue;
//                 }
//             }
//             else
//             {
//                 ASSERT(FALSE);
//             }
//
//             /* Add the OS options */
//             if (OsLoadOptions && !IniAddSettingValueToSection(SectionId, "Options", OsLoadOptions))
//             {
//                 ERR("Could not convert legacy OS entry %lu, skipping.\n", i);
//                 continue;
//             }
//         }
//
//         /* Add or modify the BootType if needed */
//         if (HadNoBootType && !IniModifySettingValue(SectionId, "BootType", BootType))
//         {
//             ERR("Could not fixup the BootType entry for OS '%s', ignoring.\n", SettingName);
//         }
//
//         /*
//          * If this is a new OS entry, but some options were given appended to
//          * the OS entry item, append them instead to the "Options=" value.
//          */
//         if (HadSection && OsLoadOptions && *OsLoadOptions)
//         {
//             /* Read the original "Options=" value */
//             *TempBuffer = ANSI_NULL;
//             if (!IniReadSettingByName(SectionId, "Options", TempBuffer, sizeof(TempBuffer)))
//                 TRACE("No 'Options' value found for OS '%s', ignoring.\n", SettingName);
//
//             /* Concatenate the options together */
//             RtlStringCbCatA(TempBuffer, sizeof(TempBuffer), " ");
//             RtlStringCbCatA(TempBuffer, sizeof(TempBuffer), OsLoadOptions);
//
//             /* Save them */
//             if (!IniModifySettingValue(SectionId, "Options", TempBuffer))
//                 ERR("Could not modify the options for OS '%s', ignoring.\n", SettingName);
//         }
//
//         /* Copy the OS section ID and its identifier */
//         Items[i].SectionId = SectionId;
//         Items[i].LoadIdentifier = CopyString(TitleStart);
//         // TRACE("We did Items[%lu]: SectionName = '%s' (SectionId = 0x%p), LoadIdentifier = '%s'\n",
//               // i, SettingName, Items[i].SectionId, Items[i].LoadIdentifier);
//     }
//
//     /* Return success */
//     *OperatingSystemCount = Count;
//     *DefaultOperatingSystem = DefaultOS;
//     return Items;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
