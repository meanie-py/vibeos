use crate::usetup::usetup::CurrentPartition;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::mui::MUIGetString;
use crate::usetup::console::WriteConsoleOutputCharacterA;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetPartitionTypeString(
    PartEntry: *mut PARTENTRY,
    strBuffer: PSTR,
    cchBuffer: u32,
) {
    if ((*PartEntry).PartitionType) as u32 == PARTITION_ENTRY_UNUSED {
        RtlStringCchCopyA(
            strBuffer,
            (cchBuffer) as _,
            MUIGetString(STRING_FORMATUNUSED),
        );
    } else if IsContainerPartition!((*PartEntry).PartitionType) {
        RtlStringCchCopyA(
            strBuffer,
            (cchBuffer) as _,
            MUIGetString(STRING_EXTENDED_PARTITION),
        );
    } else {
        let Description: PCSTR = LookupPartitionTypeString(
            (*(*PartEntry).DiskEntry).DiskStyle,
            (&raw mut (*PartEntry).PartitionType).cast(),
        );
        if !Description.is_null() {
            RtlStringCchCopyA(strBuffer, (cchBuffer) as _, Description);
            return;
        }

        if cchBuffer > 0 {
            *strBuffer = '\0' as i8;
        }
    }

    if (cchBuffer > 0) && (*strBuffer as i32 == '\0' as i32) {
        RtlStringCchPrintfA!(
            strBuffer,
            cchBuffer,
            MUIGetString(STRING_PARTTYPE),
            (*PartEntry).PartitionType as core::ffi::c_uint,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrettifySize1(
    Size: PULONGLONG,
    Unit: *mut PCSTR,
) {
    let mut DiskSize: u64 = *Size;

    if DiskSize >= 10 * GB {
        DiskSize = RoundingDivide(DiskSize, GB);
        *Unit = MUIGetString(STRING_GB);
    } else {
        DiskSize = RoundingDivide(DiskSize, MB);
        if DiskSize == 0 {
            DiskSize = 1;
        }
        *Unit = MUIGetString(STRING_MB);
    }

    *Size = DiskSize;
}
#[no_mangle]
pub unsafe extern "C" fn PrettifySize2(
    Size: PULONGLONG,
    Unit: *mut PCSTR,
) {
    let mut PartSize: u64 = *Size;

    if PartSize >= 10 * MB {
        PartSize = RoundingDivide(PartSize, MB);
        *Unit = MUIGetString(STRING_MB);
    } else {
        PartSize = RoundingDivide(PartSize, KB);
        *Unit = MUIGetString(STRING_KB);
    }

    *Size = PartSize;
}
#[no_mangle]
pub unsafe extern "C" fn PartitionDescription(
    PartEntry: *mut PARTENTRY,
    strBuffer: PSTR,
    cchBuffer: usize,
) {
    let mut pBuffer: PSTR = strBuffer;
    let mut cchBufferSize: usize = cchBuffer;
    let mut PartSize: u64;
    let mut Unit: PCSTR = core::ptr::null();
    let mut VolInfo: *mut VOLINFO = if !(*PartEntry).Volume.is_null() {
        core::ptr::addr_of_mut!((*(*PartEntry).Volume).Info)
    } else {
        core::ptr::null_mut()
    };

    PartSize = GetPartEntrySizeInBytes!(PartEntry);
    PrettifySize2(&mut PartSize, &mut Unit);

    if ((*PartEntry).IsPartitioned) as u32 == FALSE {
        RtlStringCchPrintfExA!(
            pBuffer,
            cchBufferSize,
            &mut pBuffer,
            &mut cchBufferSize,
            0,
            b"     %s%-.30s\0".as_ptr() as *const i8,
            if (*PartEntry).LogicalPartition != 0 {
                b"  \0".as_ptr() as *const i8
            } else {
                b"\0".as_ptr() as *const i8
            },
            MUIGetString(STRING_UNPSPACE),
        );

        RtlStringCchPrintfA!(
            pBuffer,
            cchBufferSize,
            b"%*s%6I64u %s\0".as_ptr() as *const i8,
            38 - min(strlen(strBuffer), 38),
            b"\0".as_ptr() as *const i8,
            PartSize,
            Unit,
        );
        return;
    }

    if (*PartEntry).DiskEntry.is_null() == false
        && (*(*PartEntry).DiskEntry).DiskStyle == PARTITION_STYLE_MBR
        && IsContainerPartition!((*PartEntry).PartitionType)
    {
        RtlStringCchPrintfExA!(
            pBuffer,
            cchBufferSize,
            &mut pBuffer,
            &mut cchBufferSize,
            0,
            b"     %-.30s\0".as_ptr() as *const i8,
            MUIGetString(STRING_EXTENDED_PARTITION),
        );

        RtlStringCchPrintfA!(
            pBuffer,
            cchBufferSize,
            b"%*s%6I64u %s\0".as_ptr() as *const i8,
            38 - min(strlen(strBuffer), 38),
            b"\0".as_ptr() as *const i8,
            PartSize,
            Unit,
        );
        return;
    }

    RtlStringCchPrintfExA!(
        pBuffer,
        cchBufferSize,
        &mut pBuffer,
        &mut cchBufferSize,
        0,
        b"%c%c %c %s(%lu) \0".as_ptr() as *const i8,
        if !(!VolInfo.is_null() && (*VolInfo).DriveLetter != 0) {
            '-' as i32
        } else {
            (*VolInfo).DriveLetter as i32
        },
        if !(!VolInfo.is_null() && (*VolInfo).DriveLetter != 0) {
            '-' as i32
        } else {
            ':' as i32
        },
        if (*PartEntry).BootIndicator != 0 {
            '*' as i32
        } else {
            ' ' as i32
        },
        if (*PartEntry).LogicalPartition != 0 {
            b"  \0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        (*PartEntry).PartitionNumber,
    );

    if !VolInfo.is_null() && IsFormatted!(VolInfo) {
        let mut cchLabelSize: usize = 0;
        if *(*VolInfo).VolumeLabel.as_ptr() != 0 {
            RtlStringCchPrintfExA!(
                pBuffer,
                cchBufferSize,
                &mut pBuffer,
                &mut cchLabelSize,
                0,
                b"\"%-.11S\" \0".as_ptr() as *const i8,
                (*VolInfo).VolumeLabel.as_ptr(),
            );
            cchLabelSize = cchBufferSize - cchLabelSize;
            cchBufferSize -= cchLabelSize;
        }

        RtlStringCchPrintfExA!(
            pBuffer,
            cchBufferSize,
            &mut pBuffer,
            &mut cchBufferSize,
            0,
            b"[%-.*S]\0".as_ptr() as *const i8,
            25 - min(cchLabelSize, 25),
            (*VolInfo).FileSystem.as_ptr(),
        );
    } else {
        let mut PartTypeString: [i8; 32] = [0; 32];
        let mut PartType: PCSTR = PartTypeString.as_mut_ptr();

        if (*PartEntry).New != 0 {
            PartType = MUIGetString(STRING_UNFORMATTED);
        } else {
            GetPartitionTypeString(
                PartEntry,
                PartTypeString.as_mut_ptr(),
                ARRAYSIZE!(PartTypeString),
            );
            PartType = PartTypeString.as_mut_ptr();
        }
        if PartType.is_null() || *PartType == 0 {
            PartType = MUIGetString(STRING_FORMATUNKNOWN);
        }

        RtlStringCchPrintfExA!(
            pBuffer,
            cchBufferSize,
            &mut pBuffer,
            &mut cchBufferSize,
            0,
            b"[%-.*s]\0".as_ptr() as *const i8,
            25,
            PartType,
        );
    }

    RtlStringCchPrintfA!(
        pBuffer,
        cchBufferSize,
        b"%*s%6I64u %s\0".as_ptr() as *const i8,
        38 - min(strlen(strBuffer), 38),
        b"\0".as_ptr() as *const i8,
        PartSize,
        Unit,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DiskDescription(
    DiskEntry: *mut DISKENTRY,
    strBuffer: PSTR,
    cchBuffer: usize,
) {
    let mut DiskSize: u64;
    let mut Unit: PCSTR = core::ptr::null();

    DiskSize = GetDiskSizeInBytes!(DiskEntry);
    PrettifySize1(&mut DiskSize, &mut Unit);

    if (*DiskEntry).DriverName.Length > 0 {
        RtlStringCchPrintfA!(
            strBuffer,
            cchBuffer,
            MUIGetString(STRING_HDDINFO1),
            DiskSize,
            Unit,
            (*DiskEntry).DiskNumber,
            (*DiskEntry).Port as core::ffi::c_uint,
            (*DiskEntry).Bus as core::ffi::c_uint,
            (*DiskEntry).Id as core::ffi::c_uint,
            &mut (*DiskEntry).DriverName,
            if (*DiskEntry).DiskStyle == PARTITION_STYLE_MBR {
                b"MBR\0".as_ptr() as *const i8
            } else if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
                b"GPT\0".as_ptr() as *const i8
            } else {
                b"RAW\0".as_ptr() as *const i8
            },
        );
    } else {
        RtlStringCchPrintfA!(
            strBuffer,
            cchBuffer,
            MUIGetString(STRING_HDDINFO2),
            DiskSize,
            Unit,
            (*DiskEntry).DiskNumber,
            (*DiskEntry).Port as core::ffi::c_uint,
            (*DiskEntry).Bus as core::ffi::c_uint,
            (*DiskEntry).Id as core::ffi::c_uint,
            if (*DiskEntry).DiskStyle == PARTITION_STYLE_MBR {
                b"MBR\0".as_ptr() as *const i8
            } else if (*DiskEntry).DiskStyle == PARTITION_STYLE_GPT {
                b"GPT\0".as_ptr() as *const i8
            } else {
                b"RAW\0".as_ptr() as *const i8
            },
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitPartitionListUi(
    ListUi: *mut PARTLIST_UI,
    List: *mut PARTLIST,
    CurrentEntry: *mut PARTENTRY,
    Left: i16,
    Top: i16,
    Right: i16,
    Bottom: i16,
) {
    (*ListUi).List = List;
    (*ListUi).Left = Left;
    (*ListUi).Top = Top;
    (*ListUi).Right = Right;
    (*ListUi).Bottom = Bottom;
    (*ListUi).Line = 0;
    (*ListUi).Offset = 0;
    if CurrentEntry.is_null() {
        (*ListUi).CurrentDisk = core::ptr::null_mut();
        (*ListUi).CurrentPartition = core::ptr::null_mut();
        if IsListEmpty(&raw mut (*List).DiskListHead) == false {
            (*ListUi).CurrentDisk = CONTAINING_RECORD!(
                (*List).DiskListHead.Flink,
                DISKENTRY,
                ListEntry,
            );
            if IsListEmpty(&raw mut (*(*ListUi).CurrentDisk).PrimaryPartListHead) == false {
                (*ListUi).CurrentPartition = CONTAINING_RECORD!(
                    (*(*ListUi).CurrentDisk).PrimaryPartListHead.Flink,
                    PARTENTRY,
                    ListEntry,
                );
            }
        }
    } else {
        (*ListUi).CurrentPartition = CurrentEntry;
        (*ListUi).CurrentDisk = (*CurrentEntry).DiskEntry;
    }
}

unsafe extern "C" fn PrintEmptyLine(ListUi: *mut PARTLIST_UI) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: ULONG = 0;
    let mut Width: USHORT;
    let mut Height: USHORT;

    Width = ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as USHORT;
    Height = ((*ListUi).Bottom as i32 - (*ListUi).Top as i32 - 2) as USHORT;

    coPos.X = ((*ListUi).Left as i32 + 1) as SHORT;
    coPos.Y = ((*ListUi).Top as i32 + 1 + (*ListUi).Line as i32) as SHORT;

    if (*ListUi).Line as i32 >= 0 && (*ListUi).Line as i32 <= Height as i32 {
        FillConsoleOutputAttribute(
            StdOutput,
            (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
            Width as DWORD,
            coPos,
            &mut Written,
        );

        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            Width as DWORD,
            coPos,
            &mut Written,
        );
    }

    (*ListUi).Line += 1;
}

unsafe extern "C" fn PrintPartitionData(
    mut ListUi: PPARTLIST_UI,
    mut DiskEntry: PDISKENTRY,
    mut PartEntry: PPARTENTRY,
) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: ULONG = 0;
    let mut Width: USHORT = 0;
    let mut Height: USHORT = 0;
    let mut Attribute: UCHAR = 0;
    let mut LineBuffer: [CHAR; 100] = [0; 100];

    PartitionDescription(
        PartEntry,
        &raw mut LineBuffer as PSTR,
        (ARRAYSIZE!(LineBuffer) as SIZE_T) as _,
    );

    Width = ((*ListUi).Right as ::core::ffi::c_int
        - (*ListUi).Left as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as USHORT;
    Height = ((*ListUi).Bottom as ::core::ffi::c_int
        - (*ListUi).Top as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int) as USHORT;

    coPos.X = ((*ListUi).Left as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as SHORT;
    coPos.Y = ((*ListUi).Top as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + (*ListUi).Line as ::core::ffi::c_int) as SHORT;

    Attribute = (if (*ListUi).CurrentDisk == DiskEntry && (*ListUi).CurrentPartition == PartEntry {
        FOREGROUND_BLUE | (BACKGROUND_WHITE) as u16
    } else {
        (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as u16
    }) as UCHAR;

    if (*ListUi).Line as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
        && (*ListUi).Line as ::core::ffi::c_int <= Height as ::core::ffi::c_int
    {
        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );
    }

    coPos.X = (coPos.X as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as SHORT;
    Width = (Width as ::core::ffi::c_int - 8 as ::core::ffi::c_int) as USHORT;

    if (*ListUi).Line as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
        && (*ListUi).Line as ::core::ffi::c_int <= Height as ::core::ffi::c_int
    {
        FillConsoleOutputAttribute(
            StdOutput,
            Attribute as WORD,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );
    }

    coPos.X += 1;
    Width = (Width as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as USHORT;

    if (*ListUi).Line as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
        && (*ListUi).Line as ::core::ffi::c_int <= Height as ::core::ffi::c_int
    {
        WriteConsoleOutputCharacterA(
            StdOutput,
            &raw mut LineBuffer as *mut CHAR as LPCSTR,
            min(strlen(&raw mut LineBuffer as *mut CHAR), Width as size_t) as DWORD,
            coPos,
            &raw mut Written,
        );
    }

    (*ListUi).Line += 1;
}

unsafe extern "C" fn PrintDiskData(ListUi: PPARTLIST_UI, DiskEntry: PDISKENTRY) {
    let mut PrimaryPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut LogicalPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut PrimaryEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut LogicalEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: ULONG = 0;
    let mut Width: USHORT = 0;
    let mut Height: USHORT = 0;
    let mut LineBuffer: [CHAR; 100] = [0; 100];

    DiskDescription(DiskEntry, LineBuffer.as_mut_ptr() as PSTR, (ARRAYSIZE!(LineBuffer)) as _);

    Width = ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as USHORT;
    Height = ((*ListUi).Bottom as i32 - (*ListUi).Top as i32 - 2) as USHORT;

    coPos.X = ((*ListUi).Left as i32 + 1) as SHORT;
    coPos.Y = ((*ListUi).Top as i32 + 1 + (*ListUi).Line as i32) as SHORT;

    if (*ListUi).Line as i32 >= 0 && (*ListUi).Line as i32 <= Height as i32 {
        FillConsoleOutputAttribute(
            StdOutput,
            (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
            Width as DWORD,
            coPos,
            &mut Written,
        );

        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            Width as DWORD,
            coPos,
            &mut Written,
        );
    }

    coPos.X += 1;
    if (*ListUi).Line as i32 >= 0 && (*ListUi).Line as i32 <= Height as i32 {
        WriteConsoleOutputCharacterA(
            StdOutput,
            LineBuffer.as_mut_ptr() as *mut CHAR as LPCSTR,
            min(
                strlen(LineBuffer.as_mut_ptr() as *mut CHAR) as USHORT,
                Width - 2,
            ) as DWORD,
            coPos,
            &mut Written,
        );
    }

    (*ListUi).Line += 1;

    PrintEmptyLine(ListUi);

    PrimaryEntry = (*DiskEntry).PrimaryPartListHead.Flink as PLIST_ENTRY;
    while PrimaryEntry != &mut (*DiskEntry).PrimaryPartListHead {
        PrimaryPartEntry = CONTAINING_RECORD!(PrimaryEntry, PARTENTRY, ListEntry);

        PrintPartitionData(ListUi, DiskEntry, PrimaryPartEntry);

        if IsContainerPartition!((*PrimaryPartEntry).PartitionType) {
            LogicalEntry = (*DiskEntry).LogicalPartListHead.Flink as PLIST_ENTRY;
            while LogicalEntry != &mut (*DiskEntry).LogicalPartListHead {
                LogicalPartEntry = CONTAINING_RECORD!(LogicalEntry, PARTENTRY, ListEntry);

                PrintPartitionData(ListUi, DiskEntry, LogicalPartEntry);

                LogicalEntry = (*LogicalEntry).Flink as PLIST_ENTRY;
            }
        }

        PrimaryEntry = (*PrimaryEntry).Flink as PLIST_ENTRY;
    }

    PrintEmptyLine(ListUi);
}
#[no_mangle]
pub unsafe extern "C" fn DrawPartitionList(ListUi: PPARTLIST_UI) {
    let List: PPARTLIST = (*ListUi).List;
    let mut Entry: PLIST_ENTRY;
    let mut Entry2: PLIST_ENTRY;
    let mut DiskEntry: PDISKENTRY;
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: ULONG = 0;
    let mut Width: USHORT;
    let mut Height: USHORT;
    let mut i: SHORT;
    let mut CurrentDiskLine: SHORT;
    let mut CurrentPartLine: SHORT;
    let mut LastLine: SHORT;
    let mut CurrentPartLineFound: BOOLEAN = (FALSE) as u8;
    let mut CurrentDiskLineFound: BOOLEAN = (FALSE) as u8;

    Width = ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as USHORT;
    Height = ((*ListUi).Bottom as i32 - (*ListUi).Top as i32 - 2) as USHORT;

    CurrentDiskLine = 0;
    CurrentPartLine = 0;
    LastLine = 0;

    Entry = (*List).DiskListHead.Flink;
    while Entry != &raw mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        LastLine += 2;
        if (CurrentPartLineFound) as u32 == FALSE {
            CurrentPartLine += 2;
        }

        Entry2 = (*DiskEntry).PrimaryPartListHead.Flink;
        while Entry2 != &raw mut (*DiskEntry).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry2, PARTENTRY, ListEntry);
            if PartEntry == (*ListUi).CurrentPartition {
                CurrentPartLineFound = (TRUE) as u8;
            }

            if (CurrentPartLineFound) as u32 == FALSE {
                CurrentPartLine += 1;
            }

            LastLine += 1;
            Entry2 = (*Entry2).Flink;
        }

        if (CurrentPartLineFound) as u32 == FALSE {
            Entry2 = (*DiskEntry).LogicalPartListHead.Flink;
            while Entry2 != &raw mut (*DiskEntry).LogicalPartListHead {
                PartEntry = CONTAINING_RECORD!(Entry2, PARTENTRY, ListEntry);
                if PartEntry == (*ListUi).CurrentPartition {
                    CurrentPartLineFound = (TRUE) as u8;
                }

                if (CurrentPartLineFound) as u32 == FALSE {
                    CurrentPartLine += 1;
                }

                LastLine += 1;
                Entry2 = (*Entry2).Flink;
            }
        }

        if DiskEntry == (*ListUi).CurrentDisk {
            CurrentDiskLineFound = (TRUE) as u8;
        }

        if (*Entry).Flink != &raw mut (*List).DiskListHead {
            if (CurrentDiskLineFound) as u32 == FALSE {
                CurrentPartLine += 1;
                CurrentDiskLine = CurrentPartLine;
            }

            LastLine += 1;
        } else {
            LastLine -= 1;
        }

        Entry = (*Entry).Flink;
    }

    if CurrentPartLine < (*ListUi).Offset {
        (*ListUi).Offset = CurrentPartLine;
    } else if (CurrentPartLine - (*ListUi).Offset) as i32 > (Height) as i32 {
        (*ListUi).Offset = CurrentPartLine - (Height) as i16;
    }

    if CurrentDiskLine < (*ListUi).Offset && ((CurrentPartLine - CurrentDiskLine) as i32) < (Height) as i32 {
        (*ListUi).Offset = CurrentDiskLine;
    }

    coPos.X = (*ListUi).Left;
    coPos.Y = (*ListUi).Top;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperLeftCorner,
        1,
        coPos,
        &raw mut Written,
    );

    coPos.X = (*ListUi).Left + 1;
    coPos.Y = (*ListUi).Top;
    if (*ListUi).Offset == 0 {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );
    } else {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            (Width as i32 - 4) as DWORD,
            coPos,
            &raw mut Written,
        );
        {
            let mut szBuff: [CHAR; 4] = [b'(' as CHAR, b'.' as CHAR, b')' as CHAR, 0];
            szBuff[1 as usize] = CharUpArrow;
            coPos.X = (*ListUi).Right - 5;
            WriteConsoleOutputCharacterA(
                StdOutput,
                szBuff.as_ptr() as PCSTR,
                3,
                coPos,
                &raw mut Written,
            );
        }
        coPos.X = (*ListUi).Right - 2;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            2,
            coPos,
            &raw mut Written,
        );
    }

    coPos.X = (*ListUi).Right;
    coPos.Y = (*ListUi).Top;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperRightCorner,
        1,
        coPos,
        &raw mut Written,
    );

    i = (*ListUi).Top + 1;
    while i < (*ListUi).Bottom {
        coPos.X = (*ListUi).Left;
        coPos.Y = i;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &raw mut Written,
        );

        coPos.X = (*ListUi).Right;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &raw mut Written,
        );
        i += 1;
    }

    coPos.X = (*ListUi).Left;
    coPos.Y = (*ListUi).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerLeftCorner,
        1,
        coPos,
        &raw mut Written,
    );

    coPos.X = (*ListUi).Left + 1;
    coPos.Y = (*ListUi).Bottom;
    if ((LastLine - (*ListUi).Offset) as i32) <= (Height) as i32 {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );
    } else {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            (Width as i32 - 4) as DWORD,
            coPos,
            &raw mut Written,
        );
        {
            let mut szBuff: [CHAR; 4] = [b'(' as CHAR, b'.' as CHAR, b')' as CHAR, 0];
            szBuff[1 as usize] = CharDownArrow;
            coPos.X = (*ListUi).Right - 5;
            WriteConsoleOutputCharacterA(
                StdOutput,
                szBuff.as_ptr() as PCSTR,
                3,
                coPos,
                &raw mut Written,
            );
        }
        coPos.X = (*ListUi).Right - 2;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            2,
            coPos,
            &raw mut Written,
        );
    }

    coPos.X = (*ListUi).Right;
    coPos.Y = (*ListUi).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerRightCorner,
        1,
        coPos,
        &raw mut Written,
    );

    (*ListUi).Line = -(*ListUi).Offset;

    Entry = (*List).DiskListHead.Flink;
    while Entry != &raw mut (*List).DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        PrintDiskData(ListUi, DiskEntry);
        Entry = (*Entry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollUpDownPartitionList(
    ListUi: *mut PARTLIST_UI,
    Direction: BOOLEAN,
) {
    let PartEntry: *mut PARTENTRY = if Direction != 0 {
        GetNextPartition((*ListUi).List, (*ListUi).CurrentPartition)
    } else {
        GetPrevPartition((*ListUi).List, (*ListUi).CurrentPartition)
    };
    if !PartEntry.is_null() {
        (*ListUi).CurrentPartition = PartEntry;
        (*ListUi).CurrentDisk = (*PartEntry).DiskEntry;
        DrawPartitionList(ListUi);
    }
}
