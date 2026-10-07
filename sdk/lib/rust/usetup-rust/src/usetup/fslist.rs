use crate::usetup::consup::CONSOLE_SetInvertedTextXY;
use crate::usetup::consup::CONSOLE_SetTextXY;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::mui::MUIGetString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn AddProvider(mut List: PFILE_SYSTEM_LIST, mut FileSystem: PCWSTR) {
    let mut Item: PFILE_SYSTEM_ITEM = core::ptr::null_mut();
    Item = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<FILE_SYSTEM_ITEM>()) as SIZE_T,
    ) as PFILE_SYSTEM_ITEM;
    if Item.is_null() {
        return;
    }
    (*Item).FileSystem = FileSystem;
    (*Item).QuickFormat = (TRUE) as u8;
    InsertTailList(&raw mut (*List).ListHead, &raw mut (*Item).ListEntry);
    if FileSystem.is_null() {
        return;
    }
    Item = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<FILE_SYSTEM_ITEM>()) as SIZE_T,
    ) as PFILE_SYSTEM_ITEM;
    if Item.is_null() {
        return;
    }
    (*Item).FileSystem = FileSystem;
    (*Item).QuickFormat = (FALSE) as u8;
    InsertTailList(&raw mut (*List).ListHead, &raw mut (*Item).ListEntry);
}

unsafe extern "C" fn InitializeFileSystemList(
    mut List: PFILE_SYSTEM_LIST,
    mut ForceFormat: BOOLEAN,
) {
    let mut FileSystemName: PCWSTR = core::ptr::null_mut();
    let mut Index: ULONG = 0;

    let mut FatPresent: BOOLEAN = (FALSE) as u8;

    Index = 0;
    loop {
        let fresh = Index;
        Index = Index.wrapping_add(1);
        if !(GetRegisteredFileSystems(fresh, &raw mut FileSystemName) != 0) {
            break;
        }
        if !(_wcsicmp(FileSystemName, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0) {
            continue;
        }
        FatPresent = (TRUE) as u8;
        break;
    }

    Index = 0;
    loop {
        let fresh = Index;
        Index = Index.wrapping_add(1);
        if !(GetRegisteredFileSystems(fresh, &raw mut FileSystemName) != 0) {
            break;
        }
        if FatPresent != 0
            && _wcsicmp(FileSystemName, RUSTCONV_WSTR_BB53EE05898F.as_ptr()) == 0
        {
            continue;
        }
        AddProvider(List, FileSystemName);
    }

    if ForceFormat == 0 {
        AddProvider(List, core::ptr::null_mut());
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateFileSystemList(
    Left: i16,
    Top: i16,
    ForceFormat: u8,
    SelectFileSystem: PCWSTR,
) -> *mut FILE_SYSTEM_LIST {
    let mut List: *mut FILE_SYSTEM_LIST;
    let mut Item: *mut FILE_SYSTEM_ITEM;
    let mut ListEntry: PLIST_ENTRY;

    List = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<FILE_SYSTEM_LIST>()) as SIZE_T,
    ) as *mut FILE_SYSTEM_LIST;
    if List.is_null() {
        return core::ptr::null_mut();
    }

    (*List).Left = Left;
    (*List).Top = Top;
    (*List).Selected = core::ptr::null_mut();
    InitializeListHead(&raw mut (*List).ListHead);

    InitializeFileSystemList(List, ForceFormat);

    ListEntry = (*List).ListHead.Flink;
    while ListEntry != &raw mut (*List).ListHead {
        Item = CONTAINING_RECORD!(ListEntry, FILE_SYSTEM_ITEM, ListEntry);
        if !(*Item).FileSystem.is_null()
            && _wcsicmp(SelectFileSystem, (*Item).FileSystem) == 0
        {
            (*List).Selected = Item;
            break;
        }
        ListEntry = (*ListEntry).Flink;
    }
    if (*List).Selected.is_null() {
        (*List).Selected = CONTAINING_RECORD!((*List).ListHead.Flink, FILE_SYSTEM_ITEM, ListEntry);
    }

    List
}
#[no_mangle]
pub unsafe extern "C" fn DestroyFileSystemList(List: *mut FILE_SYSTEM_LIST) {
    let mut ListEntry: PLIST_ENTRY;
    let mut Item: *mut FILE_SYSTEM_ITEM;

    ListEntry = (*List).ListHead.Flink;
    while !IsListEmpty(&mut (*List).ListHead) {
        ListEntry = RemoveHeadList(&mut (*List).ListHead);
        Item = CONTAINING_RECORD!(ListEntry, FILE_SYSTEM_ITEM, ListEntry);
        RtlFreeHeap(ProcessHeap, 0, (Item).cast());
    }

    RtlFreeHeap(ProcessHeap, 0, (List).cast());
}
#[no_mangle]
pub unsafe extern "C" fn DrawFileSystemList(List: *mut FILE_SYSTEM_LIST) {
    let mut ListEntry: *mut LIST_ENTRY;
    let mut Item: *mut FILE_SYSTEM_ITEM;
    let mut coPos: COORD = core::mem::zeroed();
    let mut Written: u32 = core::mem::zeroed();
    let mut Index: u32 = 0;
    let mut Buffer: [i8; 128] = [0; 128];

    ListEntry = (*List).ListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*List).ListHead) {
        Item = CONTAINING_RECORD!(ListEntry, FILE_SYSTEM_ITEM, ListEntry);

        coPos.X = (*List).Left;
        coPos.Y = ((*List).Top as i32 + Index as i16 as i32) as i16;
        FillConsoleOutputAttribute(
            StdOutput,
            (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
            core::mem::size_of::<[i8; 128]>() as u32,
            coPos,
            core::ptr::addr_of_mut!(Written),
        );
        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as i8,
            core::mem::size_of::<[i8; 128]>() as u32,
            coPos,
            core::ptr::addr_of_mut!(Written),
        );

        if !(*Item).FileSystem.is_null() {
            snprintf(
                Buffer.as_mut_ptr(),
                core::mem::size_of::<[i8; 128]>(),
                MUIGetString(
                    if (*Item).QuickFormat != 0 {
                        STRING_FORMATDISK1
                    } else {
                        STRING_FORMATDISK2
                    },
                ),
                (*Item).FileSystem,
            );
        } else {
            snprintf(
                Buffer.as_mut_ptr(),
                core::mem::size_of::<[i8; 128]>(),
                MUIGetString(STRING_KEEPFORMAT),
            );
        }

        if ListEntry == core::ptr::addr_of_mut!((*(*List).Selected).ListEntry) {
            CONSOLE_SetInvertedTextXY(
                (*List).Left,
                ((*List).Top as i32 + Index as i16 as i32) as i16,
                Buffer.as_ptr(),
            );
        } else {
            CONSOLE_SetTextXY(
                (*List).Left,
                ((*List).Top as i32 + Index as i16 as i32) as i16,
                Buffer.as_ptr(),
            );
        }
        Index = Index.wrapping_add(1);
        ListEntry = (*ListEntry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollDownFileSystemList(List: *mut FILE_SYSTEM_LIST) {
    if (*(*List).Selected).ListEntry.Flink != &raw mut (*List).ListHead {
        (*List).Selected = CONTAINING_RECORD!(
            (*(*List).Selected).ListEntry.Flink,
            FILE_SYSTEM_ITEM,
            ListEntry,
        );
        DrawFileSystemList(List);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollUpFileSystemList(List: PFILE_SYSTEM_LIST) {
    if (*(*List).Selected).ListEntry.Blink != &raw mut (*List).ListHead {
        (*List).Selected = CONTAINING_RECORD!(
            (*(*List).Selected).ListEntry.Blink,
            FILE_SYSTEM_ITEM,
            ListEntry,
        );
        DrawFileSystemList(List);
    }
}
