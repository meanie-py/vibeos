use crate::usetup::consup::CONSOLE_SetTextXY;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::console::WriteConsoleOutputCharacterA;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn InitGenericListUi(
    ListUi: *mut GENERIC_LIST_UI,
    List: *mut GENERIC_LIST,
    GetEntryDescriptionProc: PGET_ENTRY_DESCRIPTION,
) {
    (*ListUi).List = List;
    (*ListUi).FirstShown = core::ptr::null_mut();
    (*ListUi).LastShown = core::ptr::null_mut();
    (*ListUi).BackupEntry = core::ptr::null_mut();

    (*ListUi).GetEntryDescriptionProc = GetEntryDescriptionProc;

    (*ListUi).Left = 0;
    (*ListUi).Top = 0;
    (*ListUi).Right = 0;
    (*ListUi).Bottom = 0;
    (*ListUi).Redraw = (TRUE) as i32;

    (*ListUi).CurrentItemText[0] = ANSI_NULL;

    /* SaveGenericListUiState(ListUi); */
    (*ListUi).BackupEntry = (*(*ListUi).List).CurrentEntry;
}
#[no_mangle]
pub unsafe extern "C" fn RestoreGenericListUiState(ListUi: *mut GENERIC_LIST_UI) {
    (*(*ListUi).List).CurrentEntry = (*ListUi).BackupEntry;
}

unsafe extern "C" fn DrawListFrame(ListUi: PGENERIC_LIST_UI) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut i: SHORT = 0;

    coPos.X = (*ListUi).Left;
    coPos.Y = (*ListUi).Top;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    coPos.X = ((*ListUi).Left as i32 + 1) as SHORT;
    coPos.Y = (*ListUi).Top;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    coPos.X = (*ListUi).Right;
    coPos.Y = (*ListUi).Top;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperRightCorner,
        1,
        coPos,
        &mut Written,
    );

    i = ((*ListUi).Top as i32 + 1) as SHORT;
    while (i as i32) < (*ListUi).Bottom as i32 {
        coPos.X = (*ListUi).Left;
        coPos.Y = i;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.X = (*ListUi).Right;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
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
        &mut Written,
    );

    coPos.X = ((*ListUi).Left as i32 + 1) as SHORT;
    coPos.Y = (*ListUi).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as DWORD,
        coPos,
        &mut Written,
    );

    coPos.X = (*ListUi).Right;
    coPos.Y = (*ListUi).Bottom;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerRightCorner,
        1,
        coPos,
        &mut Written,
    );
}

unsafe extern "C" fn DrawListEntries(mut ListUi: PGENERIC_LIST_UI) {
    let mut List: PGENERIC_LIST = (*ListUi).List;
    let mut ListEntry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    let mut Width: USHORT = 0;

    coPos.X = ((*ListUi).Left as i32 + 1) as SHORT;
    coPos.Y = ((*ListUi).Top as i32 + 1) as SHORT;
    Width = ((*ListUi).Right as i32 - (*ListUi).Left as i32 - 1) as USHORT;

    Entry = (*ListUi).FirstShown;
    while Entry != &raw mut (*List).ListHead {
        ListEntry = CONTAINING_RECORD!(Entry, GENERIC_LIST_ENTRY, Entry);

        if coPos.Y as i32 == (*ListUi).Bottom as i32 {
            break;
        }
        (*ListUi).LastShown = Entry;

        (*ListUi).CurrentItemText[0 as usize] = ANSI_NULL;
        if !(*ListUi).GetEntryDescriptionProc.is_none() {
            (*ListUi).GetEntryDescriptionProc.unwrap_unchecked()(
                ListEntry,
                &raw mut (*ListUi).CurrentItemText as PSTR,
                (ARRAYSIZE!((*ListUi).CurrentItemText)) as _,
            );
        }

        FillConsoleOutputAttribute(
            StdOutput,
            (if (*List).CurrentEntry == ListEntry {
                FOREGROUND_BLUE | (BACKGROUND_WHITE) as u16
            } else {
                (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as u16
            }) as WORD,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );

        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );

        coPos.X += 1;
        WriteConsoleOutputCharacterA(
            StdOutput,
            &raw mut (*ListUi).CurrentItemText as *mut CHAR as LPCSTR,
            min(
                strlen(&raw mut (*ListUi).CurrentItemText as *mut CHAR),
                ((Width as SIZE_T).wrapping_sub(2)) as _,
            ) as DWORD,
            coPos,
            &raw mut Written,
        );
        coPos.X -= 1;

        coPos.Y += 1;
        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    while (coPos.Y as i32) < (*ListUi).Bottom as i32 {
        FillConsoleOutputAttribute(
            StdOutput,
            (FOREGROUND_WHITE | (BACKGROUND_BLUE) as u32) as WORD,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );

        FillConsoleOutputCharacterA(
            StdOutput,
            ' ' as i32 as CHAR,
            Width as DWORD,
            coPos,
            &raw mut Written,
        );
        coPos.Y += 1;
    }
}

unsafe extern "C" fn DrawScrollBarGenericList(mut ListUi: PGENERIC_LIST_UI) {
    let mut List: PGENERIC_LIST = (*ListUi).List;
    let mut coPos: COORD = COORD { X: 0, Y: 0 };
    let mut Written: DWORD = 0;
    coPos.X = ((*ListUi).Right as i32 + 1) as i16;
    coPos.Y = (*ListUi).Top;
    if (*ListUi).FirstShown != (*List).ListHead.Flink {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharUpArrow,
            1,
            coPos,
            &mut Written,
        );
    } else {
        FillConsoleOutputCharacterA(
            StdOutput,
            (' ' as i32) as CHAR,
            1,
            coPos,
            &mut Written,
        );
    }
    coPos.Y = (*ListUi).Bottom;
    if (*ListUi).LastShown != (*List).ListHead.Blink {
        FillConsoleOutputCharacterA(
            StdOutput,
            CharDownArrow,
            1,
            coPos,
            &mut Written,
        );
    } else {
        FillConsoleOutputCharacterA(
            StdOutput,
            (' ' as i32) as CHAR,
            1,
            coPos,
            &mut Written,
        );
    }
}

unsafe extern "C" fn CenterCurrentListItem(ListUi: PGENERIC_LIST_UI) {
    let List: PGENERIC_LIST = (*ListUi).List;
    let mut Entry: PLIST_ENTRY;
    let MaxVisibleItems: ULONG;
    let ItemCount: ULONG;
    let mut i: ULONG;

    if ((*ListUi).Top == 0 && (*ListUi).Bottom == 0) ||
        IsListEmpty(&raw mut (*List).ListHead) ||
        (*List).CurrentEntry.is_null()
    {
        return;
    }

    MaxVisibleItems = ((*ListUi).Bottom - (*ListUi).Top - 1) as ULONG;

    ItemCount = (*List).NumOfEntries;

    if ItemCount > MaxVisibleItems {
        Entry = &raw mut (*(*List).CurrentEntry).Entry as PLIST_ENTRY;
        i = 0;
        while i < MaxVisibleItems / 2 {
            if (*Entry).Blink != &raw mut (*List).ListHead {
                Entry = (*Entry).Blink as PLIST_ENTRY;
            }
            i = i.wrapping_add(1);
        }

        (*ListUi).FirstShown = Entry;

        i = 0;
        while i < MaxVisibleItems {
            if (*Entry).Flink != &raw mut (*List).ListHead {
                Entry = (*Entry).Flink as PLIST_ENTRY;
            }
            i = i.wrapping_add(1);
        }

        (*ListUi).LastShown = Entry;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DrawGenericList(
    ListUi: *mut GENERIC_LIST_UI,
    Left: i16,
    Top: i16,
    Right: i16,
    Bottom: i16,
) {
    let List: *mut GENERIC_LIST = (*ListUi).List;

    (*ListUi).FirstShown = (*List).ListHead.Flink;
    (*ListUi).Left = Left;
    (*ListUi).Top = Top;
    (*ListUi).Right = Right;
    (*ListUi).Bottom = Bottom;

    DrawListFrame(ListUi);

    if IsListEmpty(&raw mut (*List).ListHead) != false {
        return;
    }

    CenterCurrentListItem(ListUi);

    DrawListEntries(ListUi);
    DrawScrollBarGenericList(ListUi);
}
#[no_mangle]
pub unsafe extern "C" fn DrawGenericListCurrentItem(
    List: PGENERIC_LIST,
    GetEntryDescriptionProc: PGET_ENTRY_DESCRIPTION,
    Left: SHORT,
    Top: SHORT,
) {
    let mut CurrentItemText: [CHAR; 256] = [0; 256];

    if !GetEntryDescriptionProc.is_none()
        && GetNumberOfListEntries(List) > 0
    {
        (GetEntryDescriptionProc.unwrap_unchecked())(
            GetCurrentListEntry(List),
            CurrentItemText.as_mut_ptr(),
            (ARRAYSIZE!(CurrentItemText)) as _,
        );
        CONSOLE_SetTextXY(Left, Top, CurrentItemText.as_ptr());
    } else {
        CONSOLE_SetTextXY(Left, Top, b"\0".as_ptr() as *const CHAR);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollDownGenericList(ListUi: PGENERIC_LIST_UI) {
    let List: PGENERIC_LIST = (*ListUi).List;
    let mut Entry: PLIST_ENTRY;

    if (*List).CurrentEntry.is_null() {
        return;
    }

    if (*(*List).CurrentEntry).Entry.Flink != &raw mut (*List).ListHead {
        Entry = (*(*List).CurrentEntry).Entry.Flink;
        if (*ListUi).LastShown == &raw mut (*(*List).CurrentEntry).Entry {
            (*ListUi).FirstShown = (*(*ListUi).FirstShown).Flink;
            (*ListUi).LastShown = (*(*ListUi).LastShown).Flink;
        }
        (*List).CurrentEntry = CONTAINING_RECORD!(Entry, GENERIC_LIST_ENTRY, Entry);

        if (*ListUi).Redraw != 0 {
            DrawListEntries(ListUi);
            DrawScrollBarGenericList(ListUi);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollUpGenericList(ListUi: PGENERIC_LIST_UI) {
    let List: PGENERIC_LIST = (*ListUi).List;
    let mut Entry: PLIST_ENTRY;

    if (*List).CurrentEntry.is_null() {
        return;
    }

    if (*(*List).CurrentEntry).Entry.Blink != &raw mut (*List).ListHead {
        Entry = (*(*List).CurrentEntry).Entry.Blink;
        if (*ListUi).FirstShown == &raw mut (*(*List).CurrentEntry).Entry {
            (*ListUi).FirstShown = (*(*ListUi).FirstShown).Blink;
            (*ListUi).LastShown = (*(*ListUi).LastShown).Blink;
        }
        (*List).CurrentEntry = CONTAINING_RECORD!(Entry, GENERIC_LIST_ENTRY, Entry);

        if (*ListUi).Redraw != 0 {
            DrawListEntries(ListUi);
            DrawScrollBarGenericList(ListUi);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScrollPageDownGenericList(ListUi: PGENERIC_LIST_UI) {
    let mut i: SHORT;

    /* Suspend auto-redraw */
    (*ListUi).Redraw = (FALSE) as i32;

    i = ((*ListUi).Top as i32 + 1) as SHORT;
    while (i as i32) < ((*ListUi).Bottom as i32 - 1) {
        ScrollDownGenericList(ListUi);
        i += 1;
    }

    /* Update user interface */
    DrawListEntries(ListUi);
    DrawScrollBarGenericList(ListUi);

    /* Re enable auto-redraw */
    (*ListUi).Redraw = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn ScrollPageUpGenericList(ListUi: PGENERIC_LIST_UI) {
    let mut i: SHORT;

    /* Suspend auto-redraw */
    (*ListUi).Redraw = (FALSE) as i32;

    i = ((*ListUi).Bottom as i32 - 1) as SHORT;
    while i as i32 > (*ListUi).Top as i32 + 1 {
        ScrollUpGenericList(ListUi);
        i -= 1;
    }

    /* Update user interface */
    DrawListEntries(ListUi);
    DrawScrollBarGenericList(ListUi);

    /* Re enable auto-redraw */
    (*ListUi).Redraw = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn ScrollToPositionGenericList(
    ListUi: PGENERIC_LIST_UI,
    uIndex: ULONG,
) {
    let List: PGENERIC_LIST = (*ListUi).List;
    let mut Entry: PLIST_ENTRY;
    let mut uCount: ULONG = 0;

    if (*List).CurrentEntry.is_null() || uIndex == 0 {
        return;
    }

    loop {
        if (*(*List).CurrentEntry).Entry.Flink != &raw mut (*List).ListHead {
            Entry = (*(*List).CurrentEntry).Entry.Flink;
            if (*ListUi).LastShown == &raw mut (*(*List).CurrentEntry).Entry {
                (*ListUi).FirstShown = (*(*ListUi).FirstShown).Flink;
                (*ListUi).LastShown = (*(*ListUi).LastShown).Flink;
            }
            (*List).CurrentEntry = CONTAINING_RECORD!(Entry, GENERIC_LIST_ENTRY, Entry);
        }
        uCount = uCount.wrapping_add(1);
        if !(uIndex != uCount) {
            break;
        }
    }

    if (*ListUi).Redraw != 0 {
        DrawListEntries(ListUi);
        DrawScrollBarGenericList(ListUi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RedrawGenericList(ListUi: *mut GENERIC_LIST_UI) {
    if (*(*ListUi).List).CurrentEntry.is_null() {
        return;
    }
    if (*ListUi).Redraw != 0 {
        DrawListEntries(ListUi);
        DrawScrollBarGenericList(ListUi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GenericListKeyPress(
    mut ListUi: PGENERIC_LIST_UI,
    mut AsciiChar: CHAR,
) {
    let mut List: PGENERIC_LIST = (*ListUi).List;
    let mut ListEntry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut OldListEntry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut Flag: BOOLEAN = (FALSE) as u8;

    ListEntry = (*List).CurrentEntry;
    OldListEntry = (*List).CurrentEntry;

    (*ListUi).Redraw = (FALSE) as i32;

    (*ListUi).CurrentItemText[0] = ANSI_NULL;
    if !(*ListUi).GetEntryDescriptionProc.is_none() {
        (*ListUi).GetEntryDescriptionProc.unwrap_unchecked()(
            ListEntry,
            (*ListUi).CurrentItemText.as_mut_ptr(),
            (ARRAYSIZE!((*ListUi).CurrentItemText)) as _,
        );
    }

    if strlen((*ListUi).CurrentItemText.as_mut_ptr()) > 0
        && tolower((*ListUi).CurrentItemText[0] as i32)
            == AsciiChar as i32
        && (*(*List).CurrentEntry).Entry.Flink != &raw mut (*List).ListHead
    {
        ScrollDownGenericList(ListUi);
        ListEntry = (*List).CurrentEntry;

        (*ListUi).CurrentItemText[0] = ANSI_NULL;
        if !(*ListUi).GetEntryDescriptionProc.is_none() {
            (*ListUi).GetEntryDescriptionProc.unwrap_unchecked()(
                ListEntry,
                (*ListUi).CurrentItemText.as_mut_ptr(),
                (ARRAYSIZE!((*ListUi).CurrentItemText)) as _,
            );
        }

        if strlen((*ListUi).CurrentItemText.as_mut_ptr()) > 0
            && tolower((*ListUi).CurrentItemText[0] as i32)
                == AsciiChar as i32
        {
            // goto End
        } else {
            while (*(*List).CurrentEntry).Entry.Blink != &raw mut (*List).ListHead {
                ScrollUpGenericList(ListUi);
            }

            ListEntry = (*List).CurrentEntry;

            loop {
                (*ListUi).CurrentItemText[0] = ANSI_NULL;
                if !(*ListUi).GetEntryDescriptionProc.is_none() {
                    (*ListUi).GetEntryDescriptionProc.unwrap_unchecked()(
                        ListEntry,
                        (*ListUi).CurrentItemText.as_mut_ptr(),
                        (ARRAYSIZE!((*ListUi).CurrentItemText)) as _,
                    );
                }

                if strlen((*ListUi).CurrentItemText.as_mut_ptr())
                    > 0
                    && tolower(
                        (*ListUi).CurrentItemText[0] as i32,
                    ) == AsciiChar as i32
                {
                    Flag = (TRUE) as u8;
                    break;
                }

                if (*(*List).CurrentEntry).Entry.Flink == &raw mut (*List).ListHead {
                    break;
                }

                ScrollDownGenericList(ListUi);
                ListEntry = (*List).CurrentEntry;
            }

            if Flag == 0 {
                while (*(*List).CurrentEntry).Entry.Blink != &raw mut (*List).ListHead {
                    if (*List).CurrentEntry != OldListEntry {
                        ScrollUpGenericList(ListUi);
                    } else {
                        break;
                    }
                }
            }
        }
    } else {
        while (*(*List).CurrentEntry).Entry.Blink != &raw mut (*List).ListHead {
            ScrollUpGenericList(ListUi);
        }

        ListEntry = (*List).CurrentEntry;

        loop {
            (*ListUi).CurrentItemText[0] = ANSI_NULL;
            if !(*ListUi).GetEntryDescriptionProc.is_none() {
                (*ListUi).GetEntryDescriptionProc.unwrap_unchecked()(
                    ListEntry,
                    (*ListUi).CurrentItemText.as_mut_ptr(),
                    (ARRAYSIZE!((*ListUi).CurrentItemText)) as _,
                );
            }

            if strlen((*ListUi).CurrentItemText.as_mut_ptr())
                > 0
                && tolower(
                    (*ListUi).CurrentItemText[0] as i32,
                ) == AsciiChar as i32
            {
                Flag = (TRUE) as u8;
                break;
            }

            if (*(*List).CurrentEntry).Entry.Flink == &raw mut (*List).ListHead {
                break;
            }

            ScrollDownGenericList(ListUi);
            ListEntry = (*List).CurrentEntry;
        }

        if Flag == 0 {
            while (*(*List).CurrentEntry).Entry.Blink != &raw mut (*List).ListHead {
                if (*List).CurrentEntry != OldListEntry {
                    ScrollUpGenericList(ListUi);
                } else {
                    break;
                }
            }
        }
    }

    DrawListEntries(ListUi);
    DrawScrollBarGenericList(ListUi);

    (*ListUi).Redraw = (TRUE) as i32;
}
