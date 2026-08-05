use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szCheckListWndClass: [u16; 17] = [
    67, 72, 69, 67, 75, 76, 73, 83, 84, 95, 65, 67, 76, 85, 73, 0, 0
];

pub const CI_TEXT_MARGIN_WIDTH: i32 = 8;
pub const CI_TEXT_MARGIN_HEIGHT: i32 = 3;
pub const CI_TEXT_SELECTIONMARGIN: i32 = 1;
pub const TIMER_ID_SETHITFOCUS: u32 = 1;
pub const TIMER_ID_RESETQUICKSEARCH: u32 = 2;
pub const DEFAULT_QUICKSEARCH_SETFOCUS_DELAY: u32 = 2000;
pub const DEFAULT_QUICKSEARCH_RESET_DELAY: u32 = 3000;

#[repr(C)]
pub struct _CHECKITEM {
    pub Next: *mut _CHECKITEM,
    pub AccessMask: ACCESS_MASK,
    pub State: DWORD,
    pub Name: [u16; 1],
}
pub type CHECKITEM = _CHECKITEM;
pub type PCHECKITEM = *mut _CHECKITEM;

#[repr(C)]
pub struct _CHECKLISTWND {
    pub hSelf: HWND,
    pub hNotify: HWND,
    pub hFont: HFONT,
    pub CheckItemListHead: PCHECKITEM,
    pub CheckItemCount: UINT,
    pub ItemHeight: INT,
    pub FocusedCheckItem: PCHECKITEM,
    pub FocusedCheckItemBox: UINT,
    pub TextColor: [COLORREF; 2],
    pub CheckBoxLeft: [INT; 2],
    pub QuickSearchHitItem: PCHECKITEM,
    pub QuickSearchText: [u16; 65],
    pub QuickSearchSetFocusDelay: UINT,
    pub QuickSearchResetDelay: UINT,
    pub CaretWidth: DWORD,
    pub UIState: DWORD,
    pub HoveredCheckItem: PCHECKITEM,
    pub HoveredCheckItemBox: UINT,
    pub HoverTime: UINT,
    pub ThemeHandle: HTHEME,
    // The C's four `UINT x : 1` flags, spelled as plain fields. CHECKLISTWND is entirely
    // module-private — allocated, used and freed inside this DLL and reached only through
    // the window's GWLP_USERDATA pointer — so nothing outside depends on its layout, and
    // the translation reads all four as ordinary fields at 106 sites. Keeping the packed
    // `_bitfield_1: u32` would have meant bindgen-style accessors at every one of them
    // for 12 bytes per window.
    pub HasFocus: BOOL,
    pub FocusedPushed: BOOL,
    pub QuickSearchEnabled: BOOL,
    pub ShowingCaret: BOOL,
}
pub type CHECKLISTWND = _CHECKLISTWND;
pub type PCHECKLISTWND = *mut _CHECKLISTWND;
unsafe extern "C" fn NotifyControlParent(
    infoPtr: *mut CHECKLISTWND,
    code: u32,
    data: *mut core::ffi::c_void,
) -> LRESULT {
    let mut Ret: LRESULT = 0;

    if (*infoPtr).hNotify != core::ptr::null_mut() {
        let pnmh: LPNMHDR = data as LPNMHDR;

        (*pnmh).hwndFrom = (*infoPtr).hSelf;
        (*pnmh).idFrom = GetWindowLongPtr(
            (*infoPtr).hSelf,
            GWLP_ID,
        ) as UINT_PTR;
        (*pnmh).code = code;

        Ret = SendMessage(
            (*infoPtr).hNotify,
            (WM_NOTIFY) as u32,
            (*pnmh).idFrom as WPARAM,
            pnmh as LPARAM,
        );
    }

    return Ret;
}
unsafe extern "C" fn FindCheckItemByIndex(
    infoPtr: *mut _CHECKLISTWND,
    mut Index: i32,
) -> *mut _CHECKITEM {
    let mut Item: *mut _CHECKITEM;
    let mut Found: *mut _CHECKITEM = core::ptr::null_mut();

    if Index >= 0 {
        Item = (*infoPtr).CheckItemListHead;
        while Item != core::ptr::null_mut() {
            if Index == 0 {
                Found = Item;
                break;
            }

            Index -= 1;
            Item = (*Item).Next;
        }
    }

    return Found;
}
unsafe extern "C" fn FindCheckItemIndexByAccessMask(
    infoPtr: *mut _CHECKLISTWND,
    AccessMask: ACCESS_MASK,
) -> i32 {
    let mut Item: *mut CHECKITEM;
    let mut Index: i32 = 0;
    let mut Found: i32 = -1;

    Item = (*infoPtr).CheckItemListHead;
    while !Item.is_null() {
        if (*Item).AccessMask == AccessMask {
            Found = Index;
            break;
        }

        Index += 1;
        Item = (*Item).Next;
    }

    Found
}
unsafe extern "C" fn CheckItemToIndex(
    infoPtr: *mut _CHECKLISTWND,
    Item: *mut CHECKITEM,
) -> i32 {
    let mut CurItem: *mut CHECKITEM;
    let mut Index: i32;

    CurItem = (*infoPtr).CheckItemListHead;
    Index = 0;
    while CurItem != core::ptr::null_mut() {
        if CurItem == Item {
            return Index;
        }
        CurItem = (*CurItem).Next;
        Index += 1;
    }

    return -1;
}
unsafe extern "C" fn FindCheckItem(
    infoPtr: PCHECKLISTWND,
    SearchText: LPWSTR,
) -> PCHECKITEM {
    let mut CurItem: PCHECKITEM;
    let Count: usize = wcslen(SearchText);

    CurItem = (*infoPtr).CheckItemListHead;
    while CurItem != core::ptr::null_mut() {
        if ((*CurItem).State & (CIS_DISABLED) as u32) != (CIS_DISABLED) as u32
            && _wcsnicmp((*CurItem).Name.as_ptr(), SearchText, Count) == 0
        {
            break;
        }
        CurItem = (*CurItem).Next;
    }

    CurItem
}
unsafe extern "C" fn FindFirstEnabledCheckBox(
    infoPtr: PCHECKLISTWND,
    CheckBox: *mut UINT,
) -> PCHECKITEM {
    let mut CurItem: PCHECKITEM;

    CurItem = (*infoPtr).CheckItemListHead;
    while CurItem != core::ptr::null_mut() {
        if ((*CurItem).State & (CIS_DISABLED) as u32) != (CIS_DISABLED) as u32 {
            *CheckBox = if ((*CurItem).State & (CIS_ALLOWDISABLED) as u32) == 0 {
                (CLB_ALLOW) as u32
            } else {
                (CLB_DENY) as u32
            };
            break;
        }
        CurItem = (*CurItem).Next;
    }

    CurItem
}
unsafe extern "C" fn FindLastEnabledCheckBox(
    infoPtr: *mut _CHECKLISTWND,
    CheckBox: *mut UINT,
) -> *mut _CHECKITEM {
    let mut CurItem: *mut _CHECKITEM;
    let mut LastEnabledItem: *mut _CHECKITEM = core::ptr::null_mut();

    CurItem = (*infoPtr).CheckItemListHead;
    while CurItem != core::ptr::null_mut() {
        if ((*CurItem).State & (CIS_DISABLED) as u32) != (CIS_DISABLED) as u32 {
            LastEnabledItem = CurItem;
        }
        CurItem = (*CurItem).Next;
    }

    if LastEnabledItem != core::ptr::null_mut() {
        *CheckBox = if ((*LastEnabledItem).State & (CIS_DENYDISABLED) as u32) == 0 {
            (CLB_DENY) as u32
        } else {
            (CLB_ALLOW) as u32
        };
    }

    LastEnabledItem
}
unsafe extern "C" fn FindPreviousEnabledCheckBox(
    infoPtr: *mut _CHECKLISTWND,
    CheckBox: *mut UINT,
) -> *mut _CHECKITEM {
    let mut Item: *mut _CHECKITEM;

    if (*infoPtr).FocusedCheckItem != core::ptr::null_mut() {
        Item = (*infoPtr).FocusedCheckItem;

        if (*infoPtr).FocusedCheckItemBox == (CLB_DENY) as u32
            && ((*Item).State & (CIS_ALLOWDISABLED) as u32) == 0
        {
            *CheckBox = (CLB_ALLOW) as u32;
        } else {
            let mut CurItem: *mut _CHECKITEM;

            Item = core::ptr::null_mut();

            CurItem = (*infoPtr).CheckItemListHead;
            while CurItem != (*infoPtr).FocusedCheckItem {
                if ((*CurItem).State & (CIS_DISABLED) as u32) != (CIS_DISABLED) as u32 {
                    Item = CurItem;
                }
                CurItem = (*CurItem).Next;
            }

            if Item != core::ptr::null_mut() {
                *CheckBox = if ((*Item).State & (CIS_DENYDISABLED) as u32) == 0 {
                    (CLB_DENY) as u32
                } else {
                    (CLB_ALLOW) as u32
                };
            }
        }
    } else {
        Item = FindLastEnabledCheckBox(infoPtr, CheckBox);
    }

    Item
}
unsafe extern "C" fn FindNextEnabledCheckBox(
    infoPtr: PCHECKLISTWND,
    CheckBox: *mut UINT,
) -> PCHECKITEM {
    let mut Item: PCHECKITEM;

    if (*infoPtr).FocusedCheckItem != core::ptr::null_mut() {
        Item = (*infoPtr).FocusedCheckItem;

        if (*infoPtr).FocusedCheckItemBox != (CLB_DENY) as u32 &&
            ((*Item).State & (CIS_DENYDISABLED) as u32) == 0 {
            *CheckBox = (CLB_DENY) as u32;
        } else {
            Item = (*Item).Next;

            while Item != core::ptr::null_mut() {
                if ((*Item).State & (CIS_DISABLED) as u32) != (CIS_DISABLED) as u32 {
                    *CheckBox = if ((*Item).State & (CIS_ALLOWDISABLED) as u32) == 0 {
                        (CLB_ALLOW) as u32
                    } else {
                        (CLB_DENY) as u32
                    };
                    break;
                }

                Item = (*Item).Next;
            }
        }
    } else {
        Item = FindFirstEnabledCheckBox(infoPtr, CheckBox);
    }

    Item
}
unsafe extern "C" fn FindEnabledCheckBox(
    infoPtr: PCHECKLISTWND,
    ReverseSearch: BOOL,
    CheckBox: *mut UINT,
) -> PCHECKITEM {
    let Item: PCHECKITEM;

    if ReverseSearch != 0 {
        Item = FindPreviousEnabledCheckBox(infoPtr, CheckBox);
    } else {
        Item = FindNextEnabledCheckBox(infoPtr, CheckBox);
    }

    return Item;
}
unsafe extern "C" fn PtToCheckItemBox(
    infoPtr: *mut _CHECKLISTWND,
    ppt: PPOINT,
    CheckBox: *mut u32,
    DirectlyInCheckBox: *mut i32,
) -> *mut _CHECKITEM {
    let FirstVisible: i32;
    let mut Index: i32;
    let Item: *mut _CHECKITEM;

    FirstVisible = GetScrollPos((*infoPtr).hSelf, SB_VERT);

    Index = FirstVisible + ((*ppt).y / (*infoPtr).ItemHeight);

    Item = FindCheckItemByIndex(infoPtr, Index);
    if Item != core::ptr::null_mut() {
        let cx: i32;

        cx = (*infoPtr).CheckBoxLeft[CLB_ALLOW as usize]
            + (((*infoPtr).CheckBoxLeft[CLB_DENY as usize]
                - (*infoPtr).CheckBoxLeft[CLB_ALLOW as usize])
                / 2);

        *CheckBox = if (*ppt).x <= cx {
            (CLB_ALLOW) as u32
        } else {
            (CLB_DENY) as u32
        };

        if DirectlyInCheckBox != core::ptr::null_mut() {
            let y: i32 = (*ppt).y % (*infoPtr).ItemHeight;
            let cxBox: i32 = (*infoPtr).ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT);

            if (y >= CI_TEXT_MARGIN_HEIGHT && y < (*infoPtr).ItemHeight - CI_TEXT_MARGIN_HEIGHT)
                && ((((*ppt).x
                    >= ((*infoPtr).CheckBoxLeft[CLB_ALLOW as usize]
                        - (cxBox / 2)))
                    && ((*ppt).x
                        < ((*infoPtr).CheckBoxLeft[CLB_ALLOW as usize]
                            - (cxBox / 2)
                            + cxBox)))
                    || (((*ppt).x
                        >= ((*infoPtr).CheckBoxLeft[CLB_DENY as usize]
                            - (cxBox / 2)))
                        && ((*ppt).x
                            < ((*infoPtr).CheckBoxLeft[CLB_DENY as usize]
                                - (cxBox / 2)
                                + cxBox))))
            {
                *DirectlyInCheckBox = (TRUE) as i32;
            } else {
                *DirectlyInCheckBox = (FALSE) as i32;
            }
        }
    }

    return Item;
}
unsafe extern "C" fn ClearCheckItems(infoPtr: PCHECKLISTWND) {
    let mut CurItem: PCHECKITEM = (*infoPtr).CheckItemListHead;
    while !CurItem.is_null() {
        let NextItem: PCHECKITEM = (*CurItem).Next;
        HeapFree(
            GetProcessHeap(),
            0,
            CurItem as *mut core::ffi::c_void,
        );
        CurItem = NextItem;
    }

    (*infoPtr).CheckItemListHead = core::ptr::null_mut();
    (*infoPtr).CheckItemCount = 0;
}
unsafe extern "C" fn DeleteCheckItem(
    infoPtr: *mut _CHECKLISTWND,
    Item: *mut CHECKITEM,
) -> BOOL {
    let mut CurItem: *mut CHECKITEM;
    let mut PrevPtr: *mut *mut CHECKITEM = core::ptr::addr_of_mut!((*infoPtr).CheckItemListHead);

    CurItem = (*infoPtr).CheckItemListHead;
    while CurItem != core::ptr::null_mut() {
        if CurItem == Item {
            if Item == (*infoPtr).QuickSearchHitItem && (*infoPtr).QuickSearchEnabled != 0 {
                EscapeQuickSearch(infoPtr);
            }

            if Item == (*infoPtr).HoveredCheckItem {
                ChangeCheckItemHotTrack(infoPtr,
                                        core::ptr::null_mut(),
                                        0);
            }

            if Item == (*infoPtr).FocusedCheckItem {
                ChangeCheckItemFocus(infoPtr,
                                     core::ptr::null_mut(),
                                     0);
            }

            *PrevPtr = (*CurItem).Next;
            HeapFree(GetProcessHeap(),
                     0,
                     CurItem as *mut _);
            (*infoPtr).CheckItemCount -= 1;
            return (TRUE) as i32;
        }

        PrevPtr = core::ptr::addr_of_mut!((*CurItem).Next);
        CurItem = (*CurItem).Next;
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn AddCheckItem(
    infoPtr: *mut CHECKLISTWND,
    Name: LPWSTR,
    State: DWORD,
    AccessMask: ACCESS_MASK,
    Index: *mut INT,
) -> PCHECKITEM {
    let mut CurItem: PCHECKITEM;
    let mut i: INT;
    let mut PrevPtr: *mut PCHECKITEM = core::ptr::addr_of_mut!((*infoPtr).CheckItemListHead);
    let Item: PCHECKITEM = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<CHECKITEM>() + ((wcslen(Name) * core::mem::size_of::<WCHAR>()) as SIZE_T) as usize) as SIZE_T,
    ) as PCHECKITEM;
    if Item != core::ptr::null_mut() {
        CurItem = (*infoPtr).CheckItemListHead;
        i = 0;
        while CurItem != core::ptr::null_mut() {
            PrevPtr = core::ptr::addr_of_mut!((*CurItem).Next);
            i += 1;
            CurItem = (*CurItem).Next;
        }

        (*Item).Next = core::ptr::null_mut();
        (*Item).AccessMask = AccessMask;
        (*Item).State = State & (CIS_MASK) as u32;
        wcscpy((*Item).Name.as_mut_ptr(), Name);

        *PrevPtr = Item;
        (*infoPtr).CheckItemCount += 1;

        if Index != core::ptr::null_mut() {
            *Index = i;
        }
    }

    Item
}
unsafe extern "C" fn ClearCheckBoxes(infoPtr: *mut CHECKLISTWND) -> UINT {
    let mut CurItem: PCHECKITEM;
    let mut nUpdated: UINT = 0;

    CurItem = (*infoPtr).CheckItemListHead;
    while CurItem != core::ptr::null_mut() {
        if (*CurItem).State & ((CIS_ALLOW | CIS_DENY)) as u32 != 0 {
            (*CurItem).State &= (!(CIS_ALLOW | CIS_DENY)) as u32;
            nUpdated = nUpdated.wrapping_add(1);
        }
        CurItem = (*CurItem).Next;
    }

    return nUpdated;
}
unsafe extern "C" fn UpdateControl(infoPtr: *mut CHECKLISTWND) {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();
    let mut VisibleItems: i32;

    GetClientRect((*infoPtr).hSelf, &mut rcClient);

    ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
    ScrollInfo.fMask = (SIF_PAGE | SIF_RANGE) as u32;
    ScrollInfo.nMin = 0;
    ScrollInfo.nMax = (*infoPtr).CheckItemCount as i32;
    ScrollInfo.nPage = ((rcClient.bottom - rcClient.top) + (*infoPtr).ItemHeight - 1) as u32 / (*infoPtr).ItemHeight as u32;
    ScrollInfo.nPos = 0;
    ScrollInfo.nTrackPos = 0;

    VisibleItems = (rcClient.bottom - rcClient.top) / (*infoPtr).ItemHeight;

    if ScrollInfo.nPage == VisibleItems as u32 && ScrollInfo.nMax > 0 {
        ScrollInfo.nMax -= 1;
    }

    SetScrollInfo((*infoPtr).hSelf, SB_VERT, &mut ScrollInfo, (TRUE) as i32);

    RedrawWindow((*infoPtr).hSelf, core::ptr::null_mut(), core::ptr::null_mut(), (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN) as u32);
}
unsafe extern "C" fn UpdateCheckItem(
    infoPtr: *mut _CHECKLISTWND,
    Item: *mut CHECKITEM,
) {
    let mut rcClient: RECT = core::mem::zeroed();
    let VisibleFirst: i32;
    let VisibleItems: i32;
    let Index: i32 = CheckItemToIndex(infoPtr, Item);

    if Index != -1 {
        VisibleFirst = GetScrollPos((*infoPtr).hSelf, SB_VERT);

        if Index >= VisibleFirst {
            GetClientRect((*infoPtr).hSelf, &mut rcClient);

            VisibleItems = ((rcClient.bottom - rcClient.top) + (*infoPtr).ItemHeight - 1) / (*infoPtr).ItemHeight;

            if Index <= VisibleFirst + VisibleItems {
                let mut rcUpdate: RECT = core::mem::zeroed();

                rcUpdate.left = rcClient.left;
                rcUpdate.right = rcClient.right;
                rcUpdate.top = (Index - VisibleFirst) * (*infoPtr).ItemHeight;
                rcUpdate.bottom = rcUpdate.top + (*infoPtr).ItemHeight;

                RedrawWindow(
                    (*infoPtr).hSelf,
                    &mut rcUpdate,
                    core::ptr::null_mut(),
                    (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN) as u32
                );
            }
        }
    }
}
unsafe extern "C" fn MakeCheckItemVisible(
    infoPtr: *mut _CHECKLISTWND,
    Item: *mut CHECKITEM,
) {
    let mut rcClient: RECT = core::mem::zeroed();
    let VisibleFirst: i32;
    let VisibleItems: i32;
    let mut NewPos: i32;
    let Index: i32 = CheckItemToIndex(infoPtr, Item);
    if Index != -1 {
        VisibleFirst = GetScrollPos((*infoPtr).hSelf, SB_VERT);

        if Index <= VisibleFirst {
            NewPos = Index;
        } else {
            GetClientRect((*infoPtr).hSelf, &mut rcClient);

            VisibleItems = (rcClient.bottom - rcClient.top) / (*infoPtr).ItemHeight;
            if Index - VisibleItems + 1 > VisibleFirst {
                NewPos = Index - VisibleItems + 1;
            } else {
                NewPos = VisibleFirst;
            }
        }

        if VisibleFirst != NewPos {
            let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

            ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
            ScrollInfo.fMask = (SIF_POS) as u32;
            ScrollInfo.nPos = NewPos;
            NewPos = SetScrollInfo(
                (*infoPtr).hSelf,
                SB_VERT,
                &mut ScrollInfo,
                (TRUE) as i32,
            );

            if VisibleFirst != NewPos {
                ScrollWindowEx(
                    (*infoPtr).hSelf,
                    0,
                    (NewPos - VisibleFirst) * (*infoPtr).ItemHeight,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (SW_INVALIDATE | SW_SCROLLCHILDREN) as u32,
                );

                RedrawWindow(
                    (*infoPtr).hSelf,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN) as u32,
                );
            }
        }
    }
}
unsafe extern "C" fn GetIdealItemHeight(infoPtr: PCHECKLISTWND) -> UINT {
    let hdc = GetDC((*infoPtr).hSelf);
    if hdc != core::ptr::null_mut() {
        let mut height: UINT;
        let mut tm: TEXTMETRIC = core::mem::zeroed();
        let hOldFont = SelectObject(hdc, ((*infoPtr).hFont).cast());

        if GetTextMetrics(hdc, &mut tm) != 0 {
            height = tm.tmHeight as UINT;
        } else {
            height = 2;
        }

        SelectObject(hdc, hOldFont);

        ReleaseDC((*infoPtr).hSelf, hdc);

        return height;
    }
    return 0;
}
unsafe extern "C" fn RetChangeControlFont(
    infoPtr: PCHECKLISTWND,
    hFont: HFONT,
    Redraw: BOOL,
) -> HFONT {
    let hOldFont = (*infoPtr).hFont;
    (*infoPtr).hFont = hFont;

    if hOldFont != hFont {
        (*infoPtr).ItemHeight = (2 * CI_TEXT_MARGIN_HEIGHT) + (GetIdealItemHeight(infoPtr)) as i32;
    }

    if (*infoPtr).ShowingCaret != 0 {
        DestroyCaret();
        CreateCaret(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            0,
            (*infoPtr).ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT),
        );
    }

    UpdateControl(infoPtr);

    hOldFont
}
unsafe extern "C" fn CalculateCheckBoxStyle(
    Checked: BOOL,
    Enabled: BOOL,
    HotTrack: BOOL,
    Pushed: BOOL,
) -> INT {
    let BtnState: INT;

    if Checked != 0 {
        BtnState = if Enabled != 0 {
            if Pushed != 0 {
                (CBS_CHECKEDPRESSED) as i32
            } else if HotTrack != 0 {
                (CBS_CHECKEDHOT) as i32
            } else {
                (CBS_CHECKEDNORMAL) as i32
            }
        } else {
            (CBS_CHECKEDDISABLED) as i32
        };
    } else {
        BtnState = if Enabled != 0 {
            if Pushed != 0 {
                (CBS_UNCHECKEDPRESSED) as i32
            } else if HotTrack != 0 {
                (CBS_UNCHECKEDHOT) as i32
            } else {
                (CBS_UNCHECKEDNORMAL) as i32
            }
        } else {
            (CBS_UNCHECKEDDISABLED) as i32
        };
    }

    return BtnState;
}

// HAND-WRITTEN, not translated: the scanner never saw this function, so it was silently absent
// from the translation plan and from every downstream check (§9.78 class, new trigger). The C
// closes its `#if SUPPORT_UXTHEME` block BETWEEN an `if` and that `if`'s body —
//
//     if (FAILED!(hDrawResult))
//     #endif
//         { DrawFrameControl(...); }
//
// so with the flag on the braces are the `if`'s body and with it off they are a bare block.
// That token sequence does not parse in one pass, and the parse failure ate the whole function.
// aclui is built WITH -DSUPPORT_UXTHEME (see its CMakeLists), so the themed arm is the live
// one and both checkbox draws are conditional on the theme draw having failed.
unsafe extern "C" fn PaintControl(infoPtr: *mut _CHECKLISTWND, hDC: HDC, rcUpdate: PRECT) {
    let ScrollPos: INT;
    let FirstItem: PCHECKITEM;
    let mut Item: PCHECKITEM;
    let mut rcClient: RECT = core::mem::zeroed();
    let VisibleFirstIndex: UINT = ((*rcUpdate).top / (*infoPtr).ItemHeight) as UINT;
    let LastTouchedIndex: UINT = ((*rcUpdate).bottom / (*infoPtr).ItemHeight) as UINT;

    FillRect(hDC, rcUpdate, (COLOR_WINDOW + 1) as HBRUSH);

    GetClientRect((*infoPtr).hSelf, &mut rcClient);

    ScrollPos = GetScrollPos((*infoPtr).hSelf, SB_VERT);

    FirstItem = FindCheckItemByIndex(infoPtr, ScrollPos + VisibleFirstIndex as INT);
    if !FirstItem.is_null() {
        let mut TextRect: RECT = core::mem::zeroed();
        let mut ItemRect: RECT = core::mem::zeroed();
        let mut CheckBox: RECT = core::mem::zeroed();
        let hOldFont: HFONT;
        let mut CurrentIndex: DWORD;
        let OldTextColor: COLORREF;
        let Enabled: BOOL;
        let mut PrevEnabled: BOOL;
        let mut IsPushed: BOOL;
        let mut hOldBrushOrg: POINT = core::mem::zeroed();
        let mut hDrawResult: HRESULT;
        let mut ItemHovered: BOOL;

        Enabled = IsWindowEnabled((*infoPtr).hSelf);
        PrevEnabled = Enabled;

        ItemRect.left = 0;
        ItemRect.right = rcClient.right;
        ItemRect.top = (VisibleFirstIndex as INT) * (*infoPtr).ItemHeight;

        TextRect.left = ItemRect.left + CI_TEXT_MARGIN_WIDTH;
        TextRect.right = ItemRect.right - CI_TEXT_MARGIN_WIDTH;
        TextRect.top = ItemRect.top + CI_TEXT_MARGIN_HEIGHT;

        SetBrushOrgEx(hDC, ItemRect.left, ItemRect.top, &mut hOldBrushOrg);

        OldTextColor = SetTextColor(hDC, (*infoPtr).TextColor[Enabled as usize]);

        hOldFont = SelectObject(hDC, (*infoPtr).hFont as HGDIOBJ) as HFONT;

        Item = FirstItem;
        CurrentIndex = VisibleFirstIndex;
        while !Item.is_null() && CurrentIndex <= LastTouchedIndex {
            TextRect.bottom = TextRect.top + (*infoPtr).ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT);
            ItemRect.bottom = ItemRect.top + (*infoPtr).ItemHeight;

            SetBrushOrgEx(hDC, ItemRect.left, ItemRect.top, core::ptr::null_mut());

            if Enabled != 0
                && (PrevEnabled != 0)
                    != (((*Item).State & CIS_DISABLED as UINT) != CIS_DISABLED as UINT)
            {
                PrevEnabled =
                    (((*Item).State & CIS_DISABLED as UINT) != CIS_DISABLED as UINT) as BOOL;

                SetTextColor(hDC, (*infoPtr).TextColor[PrevEnabled as usize]);
            }

            ItemHovered = (Enabled != 0 && (*infoPtr).HoveredCheckItem == Item) as BOOL;

            if (*infoPtr).QuickSearchHitItem == Item {
                let OldBkColor: COLORREF;
                let OldFgColor: COLORREF;
                let mut TextSize: SIZE = core::mem::zeroed();
                let TextLen: SIZE_T;
                let HighlightLen: SIZE_T = wcslen((*infoPtr).QuickSearchText.as_ptr()) as SIZE_T;

                // highlight the quicksearch text
                if GetTextExtentPoint32(hDC, (*Item).Name.as_ptr(), HighlightLen as INT, &mut TextSize) != 0
                {
                    let HighlightTextColor: COLORREF;
                    let HighlightBackground: COLORREF;
                    let mut rcHighlight: RECT = TextRect;

                    HighlightTextColor = GetSysColor(COLOR_HIGHLIGHTTEXT);
                    HighlightBackground = GetSysColor(COLOR_HIGHLIGHT);

                    rcHighlight.right = rcHighlight.left + TextSize.cx;

                    InflateRect(&mut rcHighlight, 0, CI_TEXT_SELECTIONMARGIN);

                    OldBkColor = SetBkColor(hDC, HighlightBackground);
                    OldFgColor = SetTextColor(hDC, HighlightTextColor);

                    // draw the highlighted text
                    DrawText(
                        hDC,
                        (*Item).Name.as_ptr(),
                        HighlightLen as INT,
                        &mut rcHighlight,
                        (DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER) as UINT,
                    );

                    SetBkColor(hDC, OldBkColor);
                    SetTextColor(hDC, OldFgColor);

                    // draw the remaining part of the text
                    TextLen = wcslen((*Item).Name.as_ptr()) as SIZE_T;
                    if HighlightLen < TextLen {
                        rcHighlight.left = rcHighlight.right;
                        rcHighlight.right = TextRect.right;

                        DrawText(
                            hDC,
                            (*Item).Name.as_ptr().add((HighlightLen) as usize),
                            -1,
                            &mut rcHighlight,
                            (DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER) as UINT,
                        );
                    }
                }
            } else {
                // draw the text
                DrawText(
                    hDC,
                    (*Item).Name.as_ptr(),
                    -1,
                    &mut TextRect,
                    (DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER) as UINT,
                );
            }

            CheckBox.top = TextRect.top;
            CheckBox.bottom = TextRect.bottom;

            // draw the Allow checkbox
            IsPushed = (Enabled != 0
                && Item == (*infoPtr).FocusedCheckItem
                && (*infoPtr).HasFocus != 0
                && ((*Item).State & CIS_ALLOWDISABLED as UINT) == 0
                && (*infoPtr).FocusedCheckItemBox != CLB_DENY as UINT
                && (*infoPtr).FocusedPushed != 0) as BOOL;

            CheckBox.left = (*infoPtr).CheckBoxLeft[CLB_ALLOW as usize]
                - ((TextRect.bottom - TextRect.top) / 2);
            CheckBox.right = CheckBox.left + (TextRect.bottom - TextRect.top);
            if !(*infoPtr).ThemeHandle.is_null() {
                let BtnState: INT = CalculateCheckBoxStyle(
                    ((*Item).State & CIS_ALLOW as UINT) as BOOL,
                    (Enabled != 0 && ((*Item).State & CIS_ALLOWDISABLED as UINT) == 0) as BOOL,
                    (ItemHovered != 0
                        && (*infoPtr).HoveredCheckItemBox != CLB_DENY as UINT)
                        as BOOL,
                    IsPushed,
                );

                hDrawResult = DrawThemeBackground(
                    (*infoPtr).ThemeHandle,
                    hDC,
                    (BP_CHECKBOX) as i32,
                    BtnState,
                    &CheckBox,
                    core::ptr::null(),
                );
            } else {
                hDrawResult = E_FAIL;
            }

            // draw the standard checkbox if no themes are enabled or drawing the
            // themed control failed
            if FAILED!(hDrawResult) {
                DrawFrameControl(
                    hDC,
                    &mut CheckBox,
                    DFC_BUTTON as UINT,
                    (DFCS_BUTTONCHECK
                        | DFCS_FLAT
                        | (if ((*Item).State & CIS_ALLOWDISABLED as UINT) != 0 || Enabled == 0 {
                            DFCS_INACTIVE
                        } else {
                            0
                        })
                        | (if ((*Item).State & CIS_ALLOW as UINT) != 0 {
                            DFCS_CHECKED
                        } else {
                            0
                        })
                        | (if IsPushed != 0 { DFCS_PUSHED } else { 0 })) as UINT,
                );
            }
            if Item == (*infoPtr).FocusedCheckItem
                && ((*infoPtr).UIState & UISF_HIDEFOCUS as DWORD) == 0
                && (*infoPtr).HasFocus != 0
                && (*infoPtr).FocusedCheckItemBox != CLB_DENY as UINT
            {
                let mut rcFocus: RECT = CheckBox;

                InflateRect(&mut rcFocus, CI_TEXT_MARGIN_HEIGHT, CI_TEXT_MARGIN_HEIGHT);

                DrawFocusRect(hDC, &rcFocus);
            }

            // draw the Deny checkbox
            IsPushed = (Enabled != 0
                && Item == (*infoPtr).FocusedCheckItem
                && (*infoPtr).HasFocus != 0
                && ((*Item).State & CIS_DENYDISABLED as UINT) == 0
                && (*infoPtr).FocusedCheckItemBox == CLB_DENY as UINT
                && (*infoPtr).FocusedPushed != 0) as BOOL;

            CheckBox.left = (*infoPtr).CheckBoxLeft[CLB_DENY as usize]
                - ((TextRect.bottom - TextRect.top) / 2);
            CheckBox.right = CheckBox.left + (TextRect.bottom - TextRect.top);
            if !(*infoPtr).ThemeHandle.is_null() {
                let BtnState: INT = CalculateCheckBoxStyle(
                    ((*Item).State & CIS_DENY as UINT) as BOOL,
                    (Enabled != 0 && ((*Item).State & CIS_DENYDISABLED as UINT) == 0) as BOOL,
                    (ItemHovered != 0
                        && (*infoPtr).HoveredCheckItemBox == CLB_DENY as UINT)
                        as BOOL,
                    IsPushed,
                );

                hDrawResult = DrawThemeBackground(
                    (*infoPtr).ThemeHandle,
                    hDC,
                    (BP_CHECKBOX) as i32,
                    BtnState,
                    &CheckBox,
                    core::ptr::null(),
                );
            } else {
                hDrawResult = E_FAIL;
            }

            // draw the standard checkbox if no themes are enabled or drawing the
            // themed control failed
            if FAILED!(hDrawResult) {
                DrawFrameControl(
                    hDC,
                    &mut CheckBox,
                    DFC_BUTTON as UINT,
                    (DFCS_BUTTONCHECK
                        | DFCS_FLAT
                        | (if ((*Item).State & CIS_DENYDISABLED as UINT) != 0 || Enabled == 0 {
                            DFCS_INACTIVE
                        } else {
                            0
                        })
                        | (if ((*Item).State & CIS_DENY as UINT) != 0 {
                            DFCS_CHECKED
                        } else {
                            0
                        })
                        | (if IsPushed != 0 { DFCS_PUSHED } else { 0 })) as UINT,
                );
            }
            if (*infoPtr).HasFocus != 0
                && ((*infoPtr).UIState & UISF_HIDEFOCUS as DWORD) == 0
                && Item == (*infoPtr).FocusedCheckItem
                && (*infoPtr).FocusedCheckItemBox == CLB_DENY as UINT
            {
                let mut rcFocus: RECT = CheckBox;

                InflateRect(&mut rcFocus, CI_TEXT_MARGIN_HEIGHT, CI_TEXT_MARGIN_HEIGHT);

                DrawFocusRect(hDC, &rcFocus);
            }

            TextRect.top += (*infoPtr).ItemHeight;
            ItemRect.top += (*infoPtr).ItemHeight;

            Item = (*Item).Next;
            CurrentIndex += 1;
        }

        SelectObject(hDC, hOldFont as HGDIOBJ);

        SetTextColor(hDC, OldTextColor);

        SetBrushOrgEx(hDC, hOldBrushOrg.x, hOldBrushOrg.y, core::ptr::null_mut());
    }
}

unsafe extern "C" fn ChangeCheckItemFocus(
    infoPtr: *mut _CHECKLISTWND,
    NewFocus: *mut CHECKITEM,
    NewFocusBox: UINT,
) {
    if NewFocus != (*infoPtr).FocusedCheckItem {
        let OldFocus: *mut CHECKITEM = (*infoPtr).FocusedCheckItem;
        (*infoPtr).FocusedCheckItem = NewFocus;
        (*infoPtr).FocusedCheckItemBox = NewFocusBox;

        if !OldFocus.is_null() {
            UpdateCheckItem(infoPtr, OldFocus);
        }
    } else {
        (*infoPtr).FocusedCheckItemBox = NewFocusBox;
    }

    if !NewFocus.is_null() {
        MakeCheckItemVisible(infoPtr, NewFocus);
        UpdateCheckItem(infoPtr, NewFocus);
    }
}
unsafe extern "C" fn UpdateCheckItemBox(
    infoPtr: *mut _CHECKLISTWND,
    Item: *mut CHECKITEM,
    ItemBox: u32,
) {
    let mut rcClient: RECT = core::mem::zeroed();
    let VisibleFirst: i32;
    let VisibleItems: i32;
    let Index: i32 = CheckItemToIndex(infoPtr, Item);
    if Index != -1 {
        VisibleFirst = GetScrollPos((*infoPtr).hSelf, SB_VERT);

        if Index >= VisibleFirst {
            GetClientRect((*infoPtr).hSelf, &mut rcClient);

            VisibleItems = ((rcClient.bottom - rcClient.top) + (*infoPtr).ItemHeight - 1) / (*infoPtr).ItemHeight;

            if Index <= VisibleFirst + VisibleItems {
                let mut rcUpdate: RECT = core::mem::zeroed();

                rcUpdate.left = rcClient.left + (*infoPtr).CheckBoxLeft[ItemBox as usize] - ((*infoPtr).ItemHeight / 2);
                rcUpdate.right = rcUpdate.left + (*infoPtr).ItemHeight;
                rcUpdate.top = (Index - VisibleFirst) * (*infoPtr).ItemHeight;
                rcUpdate.bottom = rcUpdate.top + (*infoPtr).ItemHeight;

                RedrawWindow(
                    (*infoPtr).hSelf,
                    &mut rcUpdate,
                    core::ptr::null_mut(),
                    (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN) as u32,
                );
            }
        }
    }
}
unsafe extern "C" fn ChangeCheckItemHotTrack(
    infoPtr: *mut _CHECKLISTWND,
    NewHotTrack: *mut CHECKITEM,
    NewHotTrackBox: u32,
) {
    if NewHotTrack != (*infoPtr).HoveredCheckItem {
        let OldHotTrack = (*infoPtr).HoveredCheckItem;
        let OldHotTrackBox = (*infoPtr).HoveredCheckItemBox;

        (*infoPtr).HoveredCheckItem = NewHotTrack;
        (*infoPtr).HoveredCheckItemBox = NewHotTrackBox;

        if !OldHotTrack.is_null() {
            UpdateCheckItemBox(infoPtr, OldHotTrack, OldHotTrackBox);
        }
    } else {
        (*infoPtr).HoveredCheckItemBox = NewHotTrackBox;
    }

    if !NewHotTrack.is_null() {
        UpdateCheckItemBox(infoPtr, NewHotTrack, NewHotTrackBox);
    }
}
unsafe extern "C" fn ChangeCheckBox(
    infoPtr: *mut CHECKLISTWND,
    CheckItem: *mut CHECKITEM,
    CheckItemBox: u32,
) -> u8 {
    let mut CheckData: NMCHANGEITEMCHECKBOX = core::mem::zeroed();
    let OldState: u32 = (*CheckItem).State;
    let CheckedBit: u32 = if (*infoPtr).FocusedCheckItemBox == (CLB_DENY) as u32 {
        (CIS_DENY) as u32
    } else {
        (CIS_ALLOW) as u32
    };
    let Checked: u8 = if ((*CheckItem).State & CheckedBit) != 0 {
        1u8
    } else {
        0u8
    };

    CheckData.OldState = OldState;
    CheckData.NewState = if Checked != 0 {
        OldState & !CheckedBit
    } else {
        OldState | CheckedBit
    };
    CheckData.CheckBox = (*infoPtr).FocusedCheckItemBox;
    CheckData.Checked = if Checked != 0 { (0u8) as i32 } else { (1u8) as i32 };

    if NotifyControlParent(
        infoPtr,
        (CLN_CHANGINGITEMCHECKBOX) as u32,
        core::ptr::addr_of_mut!(CheckData) as *mut core::ffi::c_void,
    ) != (-1) as LRESULT
    {
        (*CheckItem).State = CheckData.NewState;
    }

    if (*CheckItem).State != OldState {
        1u8
    } else {
        0u8
    }
}
unsafe extern "C" fn DisplayCaret(infoPtr: *mut _CHECKLISTWND) {
    if IsWindowEnabled((*infoPtr).hSelf) != 0 && (*infoPtr).ShowingCaret == 0 {
        (*infoPtr).ShowingCaret = (TRUE) as i32;

        CreateCaret(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            ((*infoPtr).CaretWidth) as i32,
            (*infoPtr).ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT),
        );

        ShowCaret((*infoPtr).hSelf);
    }
}
unsafe extern "C" fn RemoveCaret(infoPtr: *mut CHECKLISTWND) {
    if IsWindowEnabled((*infoPtr).hSelf) != 0 && (*infoPtr).ShowingCaret != 0 {
        (*infoPtr).ShowingCaret = (FALSE) as i32;

        HideCaret((*infoPtr).hSelf);
        DestroyCaret();
    }
}
unsafe extern "C" fn KillQuickSearchTimers(infoPtr: *mut _CHECKLISTWND) {
    KillTimer((*infoPtr).hSelf, TIMER_ID_SETHITFOCUS as UINT_PTR);
    KillTimer((*infoPtr).hSelf, TIMER_ID_RESETQUICKSEARCH as UINT_PTR);
}
unsafe extern "C" fn MapItemToRect(
    infoPtr: *mut _CHECKLISTWND,
    CheckItem: *mut CHECKITEM,
    prcItem: *mut RECT,
) {
    let Index = CheckItemToIndex(infoPtr, CheckItem);
    if Index != -1 {
        let mut rcClient: RECT = core::mem::zeroed();
        let VisibleFirst: i32;

        GetClientRect((*infoPtr).hSelf, &mut rcClient);

        VisibleFirst = GetScrollPos((*infoPtr).hSelf, SB_VERT);

        (*prcItem).left = rcClient.left;
        (*prcItem).right = rcClient.right;
        (*prcItem).top = (Index - VisibleFirst) * (*infoPtr).ItemHeight;
        (*prcItem).bottom = (*prcItem).top + (*infoPtr).ItemHeight;
    } else {
        (*prcItem).left = 0;
        (*prcItem).top = 0;
        (*prcItem).right = 0;
        (*prcItem).bottom = 0;
    }
}
unsafe extern "C" fn UpdateCaretPos(infoPtr: *mut _CHECKLISTWND) {
    if (*infoPtr).ShowingCaret != 0 && (*infoPtr).QuickSearchHitItem != core::ptr::null_mut() {
        let hDC = GetDC((*infoPtr).hSelf);
        if hDC != core::ptr::null_mut() {
            let mut TextSize: SIZE = core::mem::zeroed();
            let hOldFont = SelectObject(hDC, ((*infoPtr).hFont).cast());

            TextSize.cx = 0;
            TextSize.cy = 0;

            if *(*infoPtr).QuickSearchText.as_ptr().add(0) == 0u16 ||
                GetTextExtentPoint32(
                    hDC,
                    (*(*infoPtr).QuickSearchHitItem).Name.as_ptr(),
                    (wcslen((*infoPtr).QuickSearchText.as_ptr())) as i32,
                    &mut TextSize
                ) != 0
            {
                let mut rcItem: RECT = core::mem::zeroed();

                MapItemToRect(
                    infoPtr,
                    (*infoPtr).QuickSearchHitItem,
                    &mut rcItem
                );

                SetCaretPos(
                    rcItem.left + CI_TEXT_MARGIN_WIDTH + TextSize.cx,
                    rcItem.top + CI_TEXT_MARGIN_HEIGHT
                );
            }

            SelectObject(hDC, hOldFont);

            ReleaseDC((*infoPtr).hSelf, hDC);
        }
    }
}
unsafe extern "C" fn EscapeQuickSearch(infoPtr: *mut _CHECKLISTWND) {
    if (*infoPtr).QuickSearchEnabled != 0 && (*infoPtr).QuickSearchHitItem != core::ptr::null_mut() {
        let OldHit = (*infoPtr).QuickSearchHitItem;

        (*infoPtr).QuickSearchHitItem = core::ptr::null_mut();
        (*infoPtr).QuickSearchText[0] = 0u16;

        if (*infoPtr).FocusedCheckItem != core::ptr::null_mut() {
            MakeCheckItemVisible(infoPtr, (*infoPtr).FocusedCheckItem);
        }

        UpdateCheckItem(infoPtr, OldHit);

        KillQuickSearchTimers(infoPtr);

        RemoveCaret(infoPtr);
    }
}
unsafe extern "C" fn ChangeSearchHit(
    infoPtr: *mut _CHECKLISTWND,
    NewHit: *mut CHECKITEM,
) {
    let OldHit = (*infoPtr).QuickSearchHitItem;

    (*infoPtr).QuickSearchHitItem = NewHit;

    if OldHit != NewHit
    {
        MakeCheckItemVisible(infoPtr,
                             NewHit);

        if OldHit != core::ptr::null_mut()
        {
            UpdateCheckItem(infoPtr,
                            OldHit);
        }
        else
        {
            DisplayCaret(infoPtr);
        }
    }

    UpdateCaretPos(infoPtr);

    UpdateCheckItem(infoPtr,
                    NewHit);

    KillTimer((*infoPtr).hSelf,
              TIMER_ID_RESETQUICKSEARCH as UINT_PTR);
    if (*infoPtr).QuickSearchSetFocusDelay != 0
    {
        SetTimer((*infoPtr).hSelf,
                 TIMER_ID_SETHITFOCUS as UINT_PTR,
                 (*infoPtr).QuickSearchSetFocusDelay,
                 None);
    }
}
unsafe extern "C" fn QuickSearchFindHit(
    infoPtr: PCHECKLISTWND,
    c: WCHAR,
) -> BOOL {
    if (*infoPtr).QuickSearchEnabled != 0 {
        let mut Ret: BOOL = (FALSE) as i32;
        let mut NewHit: PCHECKITEM;

        match c as INT {
            0x0D | 0x0A => {  // L'\r' | L'\n'
                Ret = ((*infoPtr).QuickSearchHitItem != core::ptr::null_mut()) as BOOL;
                if Ret != 0 {
                    ChangeCheckItemFocus(
                        infoPtr,
                        (*infoPtr).QuickSearchHitItem,
                        if ((*(*infoPtr).QuickSearchHitItem).State & (CIS_ALLOWDISABLED) as u32) == 0 {
                            (CLB_ALLOW) as u32
                        } else {
                            (CLB_DENY) as u32
                        },
                    );

                    EscapeQuickSearch(infoPtr);
                }
            }

            VK_BACK => {
                if (*infoPtr).QuickSearchHitItem != core::ptr::null_mut() {
                    let mut SearchLen: INT = (wcslen((*infoPtr).QuickSearchText.as_ptr())) as i32;
                    if SearchLen > 0 {
                        SearchLen -= 1;
                        (*infoPtr).QuickSearchText[SearchLen as usize] = 0u16;

                        if SearchLen > 0 {
                            NewHit = FindCheckItem(
                                infoPtr,
                                (*infoPtr).QuickSearchText.as_mut_ptr(),
                            );

                            if NewHit != core::ptr::null_mut() {
                                ChangeSearchHit(infoPtr, NewHit);

                                Ret = (TRUE) as i32;
                            }
                        }
                    }

                    if Ret == 0 {
                        EscapeQuickSearch(infoPtr);
                    }
                }
            }

            _ => {
                let mut SearchLen: INT = (wcslen((*infoPtr).QuickSearchText.as_ptr())) as i32;
                if SearchLen < (core::mem::size_of_val(&(*infoPtr).QuickSearchText) / core::mem::size_of::<WCHAR>()) as INT - 1 {
                    (*infoPtr).QuickSearchText[SearchLen as usize] = c;
                    SearchLen += 1;
                    (*infoPtr).QuickSearchText[SearchLen as usize] = 0u16;

                    NewHit = FindCheckItem(
                        infoPtr,
                        (*infoPtr).QuickSearchText.as_mut_ptr(),
                    );
                    if NewHit != core::ptr::null_mut() {
                        ChangeSearchHit(infoPtr, NewHit);

                        Ret = (TRUE) as i32;
                    } else {
                        SearchLen -= 1;
                        (*infoPtr).QuickSearchText[SearchLen as usize] = 0u16;
                    }
                }
            }
        }
        return Ret;
    }

    return (FALSE) as i32;
}
unsafe extern "system" fn CheckListWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: PCHECKLISTWND;
    let mut Ret: LRESULT;

    infoPtr = GetWindowLongPtr(hwnd, 0) as PCHECKLISTWND;

    if infoPtr.is_null() && uMsg != (WM_CREATE) as u32 {
        return DefWindowProc(hwnd, uMsg, wParam, lParam);
    }

    Ret = 0;

    match (uMsg) as INT {
        WM_PAINT => {
            let mut hdc: HDC;
            let mut rcUpdate: RECT = core::mem::zeroed();
            let mut ps: PAINTSTRUCT = core::mem::zeroed();

            if GetUpdateRect(hwnd, &mut rcUpdate, (FALSE) as i32) != 0 {
                hdc = if wParam != 0 {
                    wParam as HDC
                } else {
                    BeginPaint(hwnd, &mut ps)
                };

                if !hdc.is_null() {
                    PaintControl(infoPtr, hdc, &mut rcUpdate);

                    if wParam == 0 {
                        EndPaint(hwnd, &mut ps);
                    }
                }
            }
        }

        WM_MOUSEMOVE => {
            let mut pt: POINT = core::mem::zeroed();
            let mut InCheckBox: BOOL = 0;
            let hWndCapture: HWND = GetCapture();

            pt.x = LOWORD!(lParam) as i32;
            pt.y = HIWORD!(lParam) as i32;

            if hWndCapture.is_null() && (*infoPtr).ThemeHandle != core::ptr::null_mut() {
                let mut tme: TRACKMOUSEEVENT = core::mem::zeroed();
                let mut HotTrackItem: PCHECKITEM;
                let mut HotTrackItemBox: UINT = 0;

                HotTrackItem = PtToCheckItemBox(
                    infoPtr,
                    &mut pt,
                    &mut HotTrackItemBox,
                    &mut InCheckBox,
                );
                if !HotTrackItem.is_null() && InCheckBox != 0 {
                    if (*infoPtr).HoveredCheckItem != HotTrackItem
                        || (*infoPtr).HoveredCheckItemBox != HotTrackItemBox
                    {
                        ChangeCheckItemHotTrack(infoPtr, HotTrackItem, HotTrackItemBox);
                    }
                } else {
                    ChangeCheckItemHotTrack(infoPtr, core::ptr::null_mut(), 0);
                }

                tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as u32;
                tme.dwFlags = (TME_LEAVE) as u32;
                tme.hwndTrack = hwnd;
                tme.dwHoverTime = (*infoPtr).HoverTime;

                TrackMouseEvent(&mut tme);
            }

            if hWndCapture == hwnd && !(*infoPtr).FocusedCheckItem.is_null() {
                let mut PtItem: PCHECKITEM;
                let mut PtItemBox: UINT = 0;
                let OldPushed: UINT;

                PtItem = PtToCheckItemBox(infoPtr, &mut pt, &mut PtItemBox, &mut InCheckBox);

                OldPushed = ((*infoPtr).FocusedPushed) as u32;
                (*infoPtr).FocusedPushed = if InCheckBox != 0
                    && (*infoPtr).FocusedCheckItem == PtItem
                    && (*infoPtr).FocusedCheckItemBox == PtItemBox
                {
                    1
                } else {
                    0
                };

                if OldPushed != ((*infoPtr).FocusedPushed) as u32 {
                    UpdateCheckItemBox(
                        infoPtr,
                        (*infoPtr).FocusedCheckItem,
                        (*infoPtr).FocusedCheckItemBox,
                    );
                }
            }
        }

        WM_VSCROLL => {
            let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

            ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
            ScrollInfo.fMask = (SIF_RANGE | SIF_POS) as u32;

            if GetScrollInfo(hwnd, SB_VERT, &mut ScrollInfo) != 0 {
                let OldPos: INT = ScrollInfo.nPos;

                match (LOWORD!(wParam)) as INT {
                    SB_BOTTOM => {
                        ScrollInfo.nPos = ScrollInfo.nMax;
                    }

                    SB_LINEDOWN => {
                        if ScrollInfo.nPos < ScrollInfo.nMax {
                            ScrollInfo.nPos += 1;
                        }
                    }

                    SB_LINEUP => {
                        if ScrollInfo.nPos > 0 {
                            ScrollInfo.nPos -= 1;
                        }
                    }

                    SB_PAGEDOWN => {
                        let mut rcClient: RECT = core::mem::zeroed();
                        let ScrollLines: INT;

                        GetClientRect(hwnd, &mut rcClient);

                        ScrollLines = max!(
                            1,
                            (rcClient.bottom - rcClient.top) / (*infoPtr).ItemHeight,
                        );

                        if ScrollInfo.nPos + ScrollLines <= ScrollInfo.nMax {
                            ScrollInfo.nPos += ScrollLines;
                        } else {
                            ScrollInfo.nPos = ScrollInfo.nMax;
                        }
                    }

                    SB_PAGEUP => {
                        let mut rcClient: RECT = core::mem::zeroed();
                        let ScrollLines: INT;

                        GetClientRect(hwnd, &mut rcClient);

                        ScrollLines = max!(
                            1,
                            (rcClient.bottom - rcClient.top) / (*infoPtr).ItemHeight,
                        );

                        if ScrollInfo.nPos >= ScrollLines {
                            ScrollInfo.nPos -= ScrollLines;
                        } else {
                            ScrollInfo.nPos = 0;
                        }
                    }

                    SB_THUMBPOSITION | SB_THUMBTRACK => {
                        ScrollInfo.nPos = HIWORD!(wParam) as i32;
                    }

                    SB_TOP => {
                        ScrollInfo.nPos = 0;
                    }

                    _ => {}
                }

                if OldPos != ScrollInfo.nPos {
                    ScrollInfo.fMask = (SIF_POS) as u32;

                    ScrollInfo.nPos = SetScrollInfo(
                        hwnd,
                        SB_VERT,
                        &mut ScrollInfo,
                        (TRUE) as i32,
                    );

                    if OldPos != ScrollInfo.nPos {
                        ScrollWindowEx(
                            hwnd,
                            0,
                            (OldPos - ScrollInfo.nPos) * (*infoPtr).ItemHeight,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (SW_INVALIDATE | SW_SCROLLCHILDREN) as u32,
                        );

                        RedrawWindow(
                            hwnd,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN) as u32,
                        );
                    }
                }
            }
        }

        CLM_ADDITEM => {
            let mut Index: INT = -1;
            let Item: PCHECKITEM = AddCheckItem(
                infoPtr,
                lParam as LPWSTR,
                (CIS_NONE) as u32,
                wParam as ACCESS_MASK,
                &mut Index,
            );
            if !Item.is_null() {
                UpdateControl(infoPtr);
                Ret = Index as LRESULT;
            } else {
                Ret = -1 as LRESULT;
            }
        }

        CLM_DELITEM => {
            let Item: PCHECKITEM = FindCheckItemByIndex(infoPtr, wParam as i32);
            if !Item.is_null() {
                Ret = DeleteCheckItem(infoPtr, Item) as LRESULT;
                if Ret != 0 {
                    UpdateControl(infoPtr);
                }
            } else {
                Ret = FALSE as LRESULT;
            }
        }

        CLM_SETITEMSTATE => {
            let Item: PCHECKITEM = FindCheckItemByIndex(infoPtr, wParam as i32);
            if !Item.is_null() {
                let OldState: DWORD = (*Item).State;
                (*Item).State = lParam as DWORD & (CIS_MASK) as u32;

                if (*Item).State != OldState {
                    if Item == (*infoPtr).FocusedCheckItem
                        && ((*Item).State & (CIS_DISABLED) as u32) != 0
                    {
                        if (*infoPtr).FocusedCheckItemBox == (CLB_DENY) as u32 {
                            if ((*Item).State & (CIS_DENYDISABLED) as u32) != 0 {
                                (*infoPtr).FocusedCheckItem = core::ptr::null_mut();
                            }
                        } else {
                            if ((*Item).State & (CIS_ALLOWDISABLED) as u32) != 0 {
                                (*infoPtr).FocusedCheckItem = core::ptr::null_mut();
                            }
                        }
                    }

                    UpdateControl(infoPtr);
                }
                Ret = TRUE as LRESULT;
            }
        }

        CLM_GETITEMCOUNT => {
            Ret = (*infoPtr).CheckItemCount as LRESULT;
        }

        CLM_CLEAR => {
            ClearCheckItems(infoPtr);
            UpdateControl(infoPtr);
        }

        CLM_SETCHECKBOXCOLUMN => {
            (*infoPtr).CheckBoxLeft[if wParam != CLB_DENY as WPARAM {
                1
            } else {
                0
            }] = lParam as INT;
            UpdateControl(infoPtr);
            Ret = 1 as LRESULT;
        }

        CLM_GETCHECKBOXCOLUMN => {
            Ret = (*infoPtr).CheckBoxLeft[if wParam != CLB_DENY as WPARAM {
                1
            } else {
                0
            }] as LRESULT;
        }

        CLM_CLEARCHECKBOXES => {
            Ret = ClearCheckBoxes(infoPtr) as LRESULT;
            if Ret != 0 {
                UpdateControl(infoPtr);
            }
        }

        CLM_ENABLEQUICKSEARCH => {
            if wParam == 0 {
                EscapeQuickSearch(infoPtr);
            }
            (*infoPtr).QuickSearchEnabled = if wParam != 0 {
                1
            } else {
                0
            };
        }

        CLM_SETQUICKSEARCH_TIMEOUT_RESET => {
            (*infoPtr).QuickSearchResetDelay = wParam as UINT;
        }

        CLM_SETQUICKSEARCH_TIMEOUT_SETFOCUS => {
            (*infoPtr).QuickSearchSetFocusDelay = wParam as UINT;
        }

        CLM_FINDITEMBYACCESSMASK => {
            Ret = FindCheckItemIndexByAccessMask(infoPtr, wParam as ACCESS_MASK) as LRESULT;
        }

        WM_SETFONT => {
            Ret = RetChangeControlFont(
                infoPtr,
                wParam as HFONT,
                LOWORD!(lParam as u32) as BOOL,
            ) as LRESULT;
        }

        WM_GETFONT => {
            Ret = (*infoPtr).hFont as LRESULT;
        }

        WM_STYLECHANGED => {
            if wParam == GWL_STYLE as WPARAM {
                UpdateControl(infoPtr);
            }
        }

        WM_ENABLE => {
            EscapeQuickSearch(infoPtr);
            UpdateControl(infoPtr);
        }

        WM_MOUSEWHEEL => {
            let ScrollDelta: i16;
            let mut ScrollLines: UINT = 3;

            SystemParametersInfo(
                (SPI_GETWHEELSCROLLLINES) as u32,
                0,
                &mut ScrollLines as *mut UINT as *mut core::ffi::c_void,
                0,
            );
            ScrollDelta = 0 - HIWORD!(wParam as u32) as i16;

            if ScrollLines != 0
                && abs(ScrollDelta as i32) >= WHEEL_DELTA
            {
                let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();

                ScrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
                ScrollInfo.fMask = (SIF_RANGE | SIF_POS) as u32;

                if GetScrollInfo(hwnd, SB_VERT, &mut ScrollInfo) != 0 {
                    let OldPos: INT = ScrollInfo.nPos;

                    ScrollInfo.nPos += (ScrollDelta / (WHEEL_DELTA) as i16) as i32
                        * ScrollLines as i32;
                    if ScrollInfo.nPos < 0 {
                        ScrollInfo.nPos = 0;
                    } else if ScrollInfo.nPos > ScrollInfo.nMax {
                        ScrollInfo.nPos = ScrollInfo.nMax;
                    }

                    if OldPos != ScrollInfo.nPos {
                        ScrollInfo.fMask = (SIF_POS) as u32;

                        ScrollInfo.nPos = SetScrollInfo(
                            hwnd,
                            SB_VERT,
                            &mut ScrollInfo,
                            (TRUE) as i32,
                        );

                        if OldPos != ScrollInfo.nPos {
                            ScrollWindowEx(
                                hwnd,
                                0,
                                (OldPos - ScrollInfo.nPos) * (*infoPtr).ItemHeight,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                (SW_INVALIDATE | SW_SCROLLCHILDREN) as u32,
                            );

                            RedrawWindow(
                                hwnd,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                (RDW_ERASE
                                    | RDW_UPDATENOW
                                    | RDW_NOCHILDREN) as u32,
                            );
                        }
                    }
                }
            }
        }

        WM_SETFOCUS => {
            (*infoPtr).HasFocus = (TRUE) as i32;

            if (*infoPtr).FocusedCheckItem.is_null() {
                let Shift: BOOL = if GetKeyState(VK_SHIFT) as INT & 0x8000 != 0
                {
                    1
                } else {
                    0
                };
                (*infoPtr).FocusedCheckItem = FindEnabledCheckBox(
                    infoPtr,
                    Shift,
                    &mut (*infoPtr).FocusedCheckItemBox,
                );
            }
            if !(*infoPtr).FocusedCheckItem.is_null() {
                MakeCheckItemVisible(infoPtr, (*infoPtr).FocusedCheckItem);
                UpdateCheckItem(infoPtr, (*infoPtr).FocusedCheckItem);
            }
        }

        WM_KILLFOCUS => {
            EscapeQuickSearch(infoPtr);

            (*infoPtr).HasFocus = (FALSE) as i32;
            if !(*infoPtr).FocusedCheckItem.is_null() {
                (*infoPtr).FocusedPushed = (FALSE) as i32;
                UpdateCheckItem(infoPtr, (*infoPtr).FocusedCheckItem);
            }
        }

        WM_LBUTTONDBLCLK
        | WM_LBUTTONDOWN
        | WM_MBUTTONDOWN
        | WM_RBUTTONDOWN => {
            if IsWindowEnabled(hwnd) != 0 {
                let mut NewFocus: PCHECKITEM;
                let mut NewFocusBox: UINT = 0;
                let mut InCheckBox: BOOL = 0;
                let mut pt: POINT = core::mem::zeroed();
                let mut ChangeFocus: BOOL;
                let mut Capture: BOOL = (FALSE) as i32;

                pt.x = LOWORD!(lParam) as i32;
                pt.y = HIWORD!(lParam) as i32;

                NewFocus = PtToCheckItemBox(
                    infoPtr,
                    &mut pt,
                    &mut NewFocusBox,
                    &mut InCheckBox,
                );
                if !NewFocus.is_null() {
                    if ((*NewFocus).State
                        & ((if NewFocusBox != (CLB_DENY) as u32 {
                            CIS_ALLOWDISABLED
                        } else {
                            CIS_DENYDISABLED
                        })) as u32)
                        != 0
                    {
                        InCheckBox = (FALSE) as i32;

                        ChangeFocus = if ((*NewFocus).State & (CIS_DISABLED) as u32)
                            != (CIS_DISABLED) as u32
                        {
                            1
                        } else {
                            0
                        };
                        if ChangeFocus != 0 {
                            NewFocusBox = if NewFocusBox != (CLB_DENY) as u32 {
                                (CLB_DENY) as u32
                            } else {
                                (CLB_ALLOW) as u32
                            };
                        }
                    } else {
                        ChangeFocus = (TRUE) as i32;
                    }

                    if InCheckBox != 0
                        && ChangeFocus != 0
                        && GetCapture().is_null()
                        && (uMsg == (WM_LBUTTONDOWN) as u32 || uMsg == (WM_LBUTTONDBLCLK) as u32)
                    {
                        (*infoPtr).FocusedPushed = (TRUE) as i32;
                        Capture = (TRUE) as i32;
                    }
                } else {
                    ChangeFocus = (TRUE) as i32;
                }

                if ChangeFocus != 0 {
                    if (*infoPtr).QuickSearchEnabled != 0
                        && !(*infoPtr).QuickSearchHitItem.is_null()
                        && (*infoPtr).QuickSearchHitItem != NewFocus
                    {
                        EscapeQuickSearch(infoPtr);
                    }

                    ChangeCheckItemFocus(infoPtr, NewFocus, NewFocusBox);
                }

                if (*infoPtr).HasFocus == 0 {
                    SetFocus(hwnd);
                }

                if Capture != 0 {
                    SetCapture(hwnd);
                }
            }
        }

        WM_LBUTTONUP => {
            if GetCapture() == hwnd {
                if !(*infoPtr).FocusedCheckItem.is_null() && (*infoPtr).FocusedPushed != 0 {
                    let mut PtItem: PCHECKITEM;
                    let mut PtItemBox: UINT = 0;
                    let mut InCheckBox: BOOL = 0;
                    let mut pt: POINT = core::mem::zeroed();

                    pt.x = LOWORD!(lParam) as i32;
                    pt.y = HIWORD!(lParam) as i32;

                    (*infoPtr).FocusedPushed = (FALSE) as i32;

                    PtItem = PtToCheckItemBox(
                        infoPtr,
                        &mut pt,
                        &mut PtItemBox,
                        &mut InCheckBox,
                    );

                    if PtItem == (*infoPtr).FocusedCheckItem
                        && InCheckBox != 0
                        && PtItemBox == (*infoPtr).FocusedCheckItemBox
                    {
                        let OtherBox: UINT = if PtItemBox == (CLB_ALLOW) as u32 {
                            (CLB_DENY) as u32
                        } else {
                            (CLB_ALLOW) as u32
                        };
                        let OtherStateMask: DWORD = if OtherBox == (CLB_ALLOW) as u32 {
                            (CIS_ALLOW | CIS_ALLOWDISABLED) as u32
                        } else {
                            (CIS_DENY | CIS_DENYDISABLED) as u32
                        };
                        let OtherStateOld: DWORD = (*PtItem).State & OtherStateMask;
                        if ChangeCheckBox(infoPtr, PtItem, PtItemBox) != 0
                            && (((*PtItem).State & OtherStateMask) != OtherStateOld)
                        {
                            UpdateCheckItemBox(
                                infoPtr,
                                (*infoPtr).FocusedCheckItem,
                                OtherBox,
                            );
                        }
                    }

                    UpdateCheckItemBox(
                        infoPtr,
                        (*infoPtr).FocusedCheckItem,
                        (*infoPtr).FocusedCheckItemBox,
                    );
                }

                ReleaseCapture();
            }
        }

        WM_KEYDOWN => 'keydown_block: {
            match (wParam) as INT {
                VK_SPACE => {
                    if GetCapture().is_null()
                        && QuickSearchFindHit(infoPtr, ' ' as u16) == 0
                    {
                        if !(*infoPtr).FocusedCheckItem.is_null()
                            && ((*infoPtr).QuickSearchHitItem.is_null()
                                || (*infoPtr).QuickSearchHitItem == (*infoPtr).FocusedCheckItem)
                        {
                            let OldPushed: UINT = ((*infoPtr).FocusedPushed) as u32;
                            (*infoPtr).FocusedPushed = (TRUE) as i32;

                            if (*infoPtr).FocusedPushed != (OldPushed) as i32 {
                                MakeCheckItemVisible(infoPtr, (*infoPtr).FocusedCheckItem);
                                UpdateCheckItemBox(
                                    infoPtr,
                                    (*infoPtr).FocusedCheckItem,
                                    (*infoPtr).FocusedCheckItemBox,
                                );
                            }
                        }
                    }
                }

                VK_RETURN => {
                    if GetCapture().is_null()
                        && QuickSearchFindHit(infoPtr, '\n' as u16) == 0
                    {
                        if !(*infoPtr).FocusedCheckItem.is_null()
                            && (*infoPtr).QuickSearchHitItem.is_null()
                        {
                            let OtherBox: UINT;
                            let OtherStateMask: DWORD;
                            let OtherStateOld: DWORD;

                            MakeCheckItemVisible(infoPtr, (*infoPtr).FocusedCheckItem);

                            OtherBox = if (*infoPtr).FocusedCheckItemBox == (CLB_ALLOW) as u32
                            {
                                (CLB_DENY) as u32
                            } else {
                                (CLB_ALLOW) as u32
                            };
                            OtherStateMask = if OtherBox == (CLB_ALLOW) as u32 {
                                (CIS_ALLOW | CIS_ALLOWDISABLED) as u32
                            } else {
                                (CIS_DENY | CIS_DENYDISABLED) as u32
                            };
                            OtherStateOld = (*(*infoPtr).FocusedCheckItem).State & OtherStateMask;
                            if ChangeCheckBox(
                                infoPtr,
                                (*infoPtr).FocusedCheckItem,
                                (*infoPtr).FocusedCheckItemBox,
                            ) != 0
                            {
                                UpdateCheckItemBox(
                                    infoPtr,
                                    (*infoPtr).FocusedCheckItem,
                                    (*infoPtr).FocusedCheckItemBox,
                                );
                                if ((*(*infoPtr).FocusedCheckItem).State & OtherStateMask)
                                    != OtherStateOld
                                {
                                    UpdateCheckItemBox(
                                        infoPtr,
                                        (*infoPtr).FocusedCheckItem,
                                        OtherBox,
                                    );
                                }
                            }
                        }
                    }
                }

                VK_TAB => {
                    if GetCapture().is_null() {
                        let mut NewFocus: PCHECKITEM;
                        let mut NewFocusBox: UINT = 0;
                        let Shift: BOOL =
                            if GetKeyState(VK_SHIFT) as INT & 0x8000 != 0 {
                                1
                            } else {
                                0
                            };

                        EscapeQuickSearch(infoPtr);

                        NewFocus = FindEnabledCheckBox(infoPtr, Shift, &mut NewFocusBox);

                        SendMessage(
                            GetAncestor(hwnd, (GA_PARENT) as u32),
                            (WM_CHANGEUISTATE) as u32,
                            MAKEWPARAM!(UIS_INITIALIZE, 0),
                            0,
                        );

                        ChangeCheckItemFocus(infoPtr, NewFocus, NewFocusBox);
                    }
                }

                _ => {
                    break 'keydown_block;
                }
            }
        }

        WM_KEYUP => {
            if wParam as u32 == (VK_SPACE) as u32
                && IsWindowEnabled(hwnd) != 0
                && !(*infoPtr).FocusedCheckItem.is_null()
                && (*infoPtr).FocusedPushed != 0
            {
                let OtherBox: UINT = if (*infoPtr).FocusedCheckItemBox == (CLB_ALLOW) as u32 {
                    (CLB_DENY) as u32
                } else {
                    (CLB_ALLOW) as u32
                };
                let OtherStateMask: DWORD = if OtherBox == (CLB_ALLOW) as u32 {
                    (CIS_ALLOW | CIS_ALLOWDISABLED) as u32
                } else {
                    (CIS_DENY | CIS_DENYDISABLED) as u32
                };
                let OtherStateOld: DWORD = (*(*infoPtr).FocusedCheckItem).State & OtherStateMask;

                (*infoPtr).FocusedPushed = (FALSE) as i32;

                if ChangeCheckBox(
                    infoPtr,
                    (*infoPtr).FocusedCheckItem,
                    (*infoPtr).FocusedCheckItemBox,
                ) != 0
                {
                    UpdateCheckItemBox(
                        infoPtr,
                        (*infoPtr).FocusedCheckItem,
                        (*infoPtr).FocusedCheckItemBox,
                    );

                    if ((*(*infoPtr).FocusedCheckItem).State & OtherStateMask) != OtherStateOld {
                        UpdateCheckItemBox(
                            infoPtr,
                            (*infoPtr).FocusedCheckItem,
                            OtherBox,
                        );
                    }
                }
            }
        }

        WM_GETDLGCODE => {
            let virtKey: INT;

            Ret = 0;
            virtKey = if lParam != 0 {
                (*(lParam as LPMSG)).wParam as INT
            } else {
                0
            };
            match (virtKey) as INT {
                VK_RETURN => {
                    if (*infoPtr).QuickSearchEnabled != 0
                        && !(*infoPtr).QuickSearchHitItem.is_null()
                    {
                        Ret |= (DLGC_WANTCHARS | DLGC_WANTMESSAGE) as LRESULT;
                    } else {
                        Ret |= (DLGC_WANTMESSAGE) as LRESULT;
                    }
                }

                VK_TAB => {
                    let mut CheckBox: UINT = 0;
                    let EnabledBox: BOOL;
                    let Shift: BOOL =
                        if GetKeyState(VK_SHIFT) as INT & 0x8000 != 0 {
                            1
                        } else {
                            0
                        };

                    EnabledBox =
                        if !FindEnabledCheckBox(infoPtr, Shift, &mut CheckBox).is_null() {
                            1
                        } else {
                            0
                        };
                    Ret |= (if EnabledBox != 0 {
                        DLGC_WANTTAB
                    } else {
                        DLGC_WANTCHARS
                    }) as LRESULT;
                }

                _ => {
                    if (*infoPtr).QuickSearchEnabled != 0 {
                        Ret |= (DLGC_WANTCHARS) as LRESULT;
                    }
                }
            }
        }

        WM_CHAR => {
            QuickSearchFindHit(infoPtr, wParam as u16);
        }

        WM_SYSCOLORCHANGE => {
            (*infoPtr).TextColor[0 as usize] =
                GetSysColor(COLOR_GRAYTEXT);
            (*infoPtr).TextColor[1 as usize] =
                GetSysColor(COLOR_WINDOWTEXT);
        }

        WM_MOUSELEAVE => {
            if !(*infoPtr).HoveredCheckItem.is_null() {
                ChangeCheckItemHotTrack(infoPtr, core::ptr::null_mut(), 0);
            }
        }

        WM_THEMECHANGED => {
            if !(*infoPtr).ThemeHandle.is_null() {
                CloseThemeData((*infoPtr).ThemeHandle);
                (*infoPtr).ThemeHandle = core::ptr::null_mut();
            }
            if IsAppThemed() != 0 {
                (*infoPtr).ThemeHandle =
                    OpenThemeData((*infoPtr).hSelf, w!("BUTTON") as *const u16);
            }
        }

        WM_SETTINGCHANGE => {
            let OldCaretWidth: DWORD = (*infoPtr).CaretWidth;

            if SystemParametersInfo(
                (SPI_GETMOUSEHOVERTIME) as u32,
                0,
                &mut (*infoPtr).HoverTime as *mut UINT as *mut core::ffi::c_void,
                0,
            ) == 0
            {
                (*infoPtr).HoverTime = (HOVER_DEFAULT) as u32;
            }

            if SystemParametersInfo(
                (SPI_GETCARETWIDTH) as u32,
                0,
                &mut (*infoPtr).CaretWidth as *mut DWORD as *mut core::ffi::c_void,
                0,
            ) == 0
            {
                (*infoPtr).CaretWidth = 2;
            }
            if OldCaretWidth != (*infoPtr).CaretWidth && (*infoPtr).ShowingCaret != 0 {
                DestroyCaret();
                CreateCaret(
                    hwnd,
                    core::ptr::null_mut(),
                    ((*infoPtr).CaretWidth) as i32,
                    (*infoPtr).ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT),
                );
            }
        }

        WM_SIZE => {
            UpdateControl(infoPtr);
        }

        WM_UPDATEUISTATE => {
            let OldUIState: DWORD = (*infoPtr).UIState;

            match (LOWORD!(wParam as u32)) as INT {
                UIS_SET => {
                    (*infoPtr).UIState |= HIWORD!(wParam as u32) as DWORD;
                }

                UIS_CLEAR => {
                    (*infoPtr).UIState &= (!(HIWORD!(wParam as u32))) as u32;
                }

                _ => {}
            }

            if OldUIState != (*infoPtr).UIState {
                if !(*infoPtr).FocusedCheckItem.is_null() {
                    UpdateCheckItemBox(
                        infoPtr,
                        (*infoPtr).FocusedCheckItem,
                        (*infoPtr).FocusedCheckItemBox,
                    );
                }
            }
        }

        WM_TIMER => {
            match (wParam) as UINT {
                TIMER_ID_SETHITFOCUS => {
                    KillTimer(hwnd, wParam as UINT_PTR);

                    if (*infoPtr).QuickSearchEnabled != 0
                        && !(*infoPtr).QuickSearchHitItem.is_null()
                    {
                        ChangeCheckItemFocus(
                            infoPtr,
                            (*infoPtr).QuickSearchHitItem,
                            if ((*(*infoPtr).QuickSearchHitItem).State & (CIS_ALLOWDISABLED) as u32)
                                == 0
                            {
                                (CLB_ALLOW) as u32
                            } else {
                                (CLB_DENY) as u32
                            },
                        );

                        if (*infoPtr).QuickSearchResetDelay != 0 {
                            SetTimer(
                                hwnd,
                                TIMER_ID_RESETQUICKSEARCH as UINT_PTR,
                                (*infoPtr).QuickSearchResetDelay,
                                None,
                            );
                        }
                    }
                }
                TIMER_ID_RESETQUICKSEARCH => {
                    KillTimer(hwnd, wParam as UINT_PTR);
                    EscapeQuickSearch(infoPtr);
                }
                _ => {}
            }
        }

        WM_CREATE => {
            infoPtr = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<CHECKLISTWND>() as SIZE_T,
            ) as PCHECKLISTWND;
            if !infoPtr.is_null() {
                let mut rcClient: RECT = core::mem::zeroed();

                (*infoPtr).hSelf = hwnd;
                (*infoPtr).hNotify = (*(lParam as LPCREATESTRUCTW)).hwndParent;

                SetWindowLongPtr(hwnd, 0, infoPtr as LONG_PTR);

                (*infoPtr).CheckItemListHead = core::ptr::null_mut();
                (*infoPtr).CheckItemCount = 0;

                if SystemParametersInfo(
                    (SPI_GETCARETWIDTH) as u32,
                    0,
                    &mut (*infoPtr).CaretWidth as *mut DWORD as *mut core::ffi::c_void,
                    0,
                ) == 0
                {
                    (*infoPtr).CaretWidth = 2;
                }
                (*infoPtr).ItemHeight = 10;
                (*infoPtr).ShowingCaret = (FALSE) as i32;

                (*infoPtr).HasFocus = (FALSE) as i32;
                (*infoPtr).FocusedCheckItem = core::ptr::null_mut();
                (*infoPtr).FocusedCheckItemBox = 0;
                (*infoPtr).FocusedPushed = (FALSE) as i32;

                (*infoPtr).TextColor[0 as usize] =
                    GetSysColor(COLOR_GRAYTEXT);
                (*infoPtr).TextColor[1 as usize] =
                    GetSysColor(COLOR_WINDOWTEXT);

                GetClientRect(hwnd, &mut rcClient);

                (*infoPtr).CheckBoxLeft[0 as usize] =
                    rcClient.right - 30;
                (*infoPtr).CheckBoxLeft[1 as usize] =
                    rcClient.right - 15;

                (*infoPtr).QuickSearchEnabled = (FALSE) as i32;
                (*infoPtr).QuickSearchText[0 as usize] = 0;

                (*infoPtr).QuickSearchSetFocusDelay = DEFAULT_QUICKSEARCH_SETFOCUS_DELAY;
                (*infoPtr).QuickSearchResetDelay = DEFAULT_QUICKSEARCH_RESET_DELAY;

                (*infoPtr).HoveredCheckItem = core::ptr::null_mut();
                (*infoPtr).HoveredCheckItemBox = 0;
                if SystemParametersInfo(
                    (SPI_GETMOUSEHOVERTIME) as u32,
                    0,
                    &mut (*infoPtr).HoverTime as *mut UINT as *mut core::ffi::c_void,
                    0,
                ) == 0
                {
                    (*infoPtr).HoverTime = (HOVER_DEFAULT) as u32;
                }

                if IsAppThemed() != 0 {
                    (*infoPtr).ThemeHandle = OpenThemeData(
                        (*infoPtr).hSelf,
                        w!("BUTTON") as *const u16,
                    );
                } else {
                    (*infoPtr).ThemeHandle = core::ptr::null_mut();
                }

                (*infoPtr).UIState = SendMessage(
                    hwnd,
                    (WM_QUERYUISTATE) as u32,
                    0,
                    0,
                ) as DWORD;
            } else {
                Ret = -1 as LRESULT;
            }
        }

        WM_DESTROY => {
            if (*infoPtr).ShowingCaret != 0 {
                DestroyCaret();
            }

            ClearCheckItems(infoPtr);

            if !(*infoPtr).ThemeHandle.is_null() {
                CloseThemeData((*infoPtr).ThemeHandle);
            }

            HeapFree(
                GetProcessHeap(),
                0,
                infoPtr as *mut core::ffi::c_void,
            );
            SetWindowLongPtr(hwnd, 0, 0);
        }

        _ => {
            return DefWindowProc(hwnd, uMsg, wParam, lParam);
        }
    }

    Ret
}
#[no_mangle]
pub unsafe extern "C" fn RegisterCheckListControl(hInstance: HINSTANCE) -> BOOL {
    let mut wc: WNDCLASSW = core::mem::zeroed();

    wc.style = (CS_DBLCLKS) as u32;
    wc.lpfnWndProc = Some(CheckListWndProc);
    wc.cbClsExtra = 0;
    wc.cbWndExtra = core::mem::size_of::<PCHECKLISTWND>() as i32;
    wc.hInstance = hInstance;
    wc.hIcon = core::ptr::null_mut();
    wc.hCursor = LoadCursor(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR
    );
    wc.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = szCheckListWndClass.as_ptr();

    (RegisterClass(&wc) != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterCheckListControl(hInstance: HINSTANCE) {
    UnregisterClass(szCheckListWndClass.as_ptr(), hInstance);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Access Control List Editor
//  * Copyright (C) 2004-2005 ReactOS Team
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS Access Control List Editor
//  * FILE:            lib/aclui/checklist.c
//  * PURPOSE:         Access Control List Editor
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      07/01/2005  Created
//  */
//
// #include "precomp.h"
//
// #ifdef SUPPORT_UXTHEME
// #include <uxtheme.h>
// #include <tmschema.h>
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// static const WCHAR szCheckListWndClass[] = L"CHECKLIST_ACLUI";
//
// #define CI_TEXT_MARGIN_WIDTH    (8)
// #define CI_TEXT_MARGIN_HEIGHT   (3)
// #define CI_TEXT_SELECTIONMARGIN (1)
//
// #define TIMER_ID_SETHITFOCUS    (1)
// #define TIMER_ID_RESETQUICKSEARCH       (2)
//
// #define DEFAULT_QUICKSEARCH_SETFOCUS_DELAY      (2000)
// #define DEFAULT_QUICKSEARCH_RESET_DELAY (3000)
//
// typedef struct _CHECKITEM
// {
//     struct _CHECKITEM *Next;
//     ACCESS_MASK AccessMask;
//     DWORD State;
//     WCHAR Name[1];
// } CHECKITEM, *PCHECKITEM;
//
// typedef struct _CHECKLISTWND
// {
//     HWND hSelf;
//     HWND hNotify;
//     HFONT hFont;
//
//     PCHECKITEM CheckItemListHead;
//     UINT CheckItemCount;
//
//     INT ItemHeight;
//
//     PCHECKITEM FocusedCheckItem;
//     UINT FocusedCheckItemBox;
//
//     COLORREF TextColor[2];
//     INT CheckBoxLeft[2];
//
//     PCHECKITEM QuickSearchHitItem;
//     WCHAR QuickSearchText[65];
//     UINT QuickSearchSetFocusDelay;
//     UINT QuickSearchResetDelay;
//
//     DWORD CaretWidth;
//
//     DWORD UIState;
//
// #if SUPPORT_UXTHEME
//     PCHECKITEM HoveredCheckItem;
//     UINT HoveredCheckItemBox;
//     UINT HoverTime;
//
//     HTHEME ThemeHandle;
// #endif
//
//     UINT HasFocus : 1;
//     UINT FocusedPushed : 1;
//     UINT QuickSearchEnabled : 1;
//     UINT ShowingCaret : 1;
// } CHECKLISTWND, *PCHECKLISTWND;
//
// static VOID EscapeQuickSearch( PCHECKLISTWND infoPtr);
// #if SUPPORT_UXTHEME
// static VOID ChangeCheckItemHotTrack( PCHECKLISTWND infoPtr,
//                                      PCHECKITEM NewHotTrack,
//                                      UINT NewHotTrackBox);
// #endif
// static VOID ChangeCheckItemFocus( PCHECKLISTWND infoPtr,
//                                   PCHECKITEM NewFocus,
//                                   UINT NewFocusBox);
//
// /******************************************************************************/
//
// static LRESULT
// NotifyControlParent( PCHECKLISTWND infoPtr,
//                      UINT code,
//                       PVOID data)
// {
//     LRESULT Ret = 0;
//
//     if (infoPtr->hNotify != NULL)
//     {
//         LPNMHDR pnmh = (LPNMHDR)data;
//
//         pnmh->hwndFrom = infoPtr->hSelf;
//         pnmh->idFrom = GetWindowLongPtr(infoPtr->hSelf,
//                                         GWLP_ID);
//         pnmh->code = code;
//
//         Ret = SendMessage(infoPtr->hNotify,
//                           WM_NOTIFY,
//                           (WPARAM)pnmh->idFrom,
//                           (LPARAM)pnmh);
//     }
//
//     return Ret;
// }
//
// static PCHECKITEM
// FindCheckItemByIndex( PCHECKLISTWND infoPtr,
//                       INT Index)
// {
//     PCHECKITEM Item, Found = NULL;
//
//     if (Index >= 0)
//     {
//         for (Item = infoPtr->CheckItemListHead;
//              Item != NULL;
//              Item = Item->Next)
//         {
//             if (Index == 0)
//             {
//                 Found = Item;
//                 break;
//             }
//
//             Index--;
//         }
//     }
//
//     return Found;
// }
//
// static INT
// FindCheckItemIndexByAccessMask( PCHECKLISTWND infoPtr,
//                                 ACCESS_MASK AccessMask)
// {
//     PCHECKITEM Item;
//     INT Index = 0, Found = -1;
//
//     for (Item = infoPtr->CheckItemListHead;
//          Item != NULL;
//          Item = Item->Next)
//     {
//         if (Item->AccessMask == AccessMask)
//         {
//             Found = Index;
//             break;
//         }
//
//         Index++;
//     }
//
//     return Found;
// }
//
// static INT
// CheckItemToIndex( PCHECKLISTWND infoPtr,
//                   PCHECKITEM Item)
// {
//     PCHECKITEM CurItem;
//     INT Index;
//
//     for (CurItem = infoPtr->CheckItemListHead, Index = 0;
//          CurItem != NULL;
//          CurItem = CurItem->Next, Index++)
//     {
//         if (CurItem == Item)
//         {
//             return Index;
//         }
//     }
//
//     return -1;
// }
//
// static PCHECKITEM
// FindCheckItem( PCHECKLISTWND infoPtr,
//                LPWSTR SearchText)
// {
//     PCHECKITEM CurItem;
//     SIZE_T Count = wcslen(SearchText);
//
//     for (CurItem = infoPtr->CheckItemListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if ((CurItem->State & CIS_DISABLED) != CIS_DISABLED &&
//             !_wcsnicmp(CurItem->Name,
//                       SearchText, Count))
//         {
//             break;
//         }
//     }
//
//     return CurItem;
// }
//
// static PCHECKITEM
// FindFirstEnabledCheckBox( PCHECKLISTWND infoPtr,
//                           UINT *CheckBox)
// {
//     PCHECKITEM CurItem;
//
//     for (CurItem = infoPtr->CheckItemListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if ((CurItem->State & CIS_DISABLED) != CIS_DISABLED)
//         {
//             /* return the Allow checkbox in case both check boxes are enabled! */
//             *CheckBox = ((!(CurItem->State & CIS_ALLOWDISABLED)) ? CLB_ALLOW : CLB_DENY);
//             break;
//         }
//     }
//
//     return CurItem;
// }
//
// static PCHECKITEM
// FindLastEnabledCheckBox( PCHECKLISTWND infoPtr,
//                          UINT *CheckBox)
// {
//     PCHECKITEM CurItem;
//     PCHECKITEM LastEnabledItem = NULL;
//
//     for (CurItem = infoPtr->CheckItemListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if ((CurItem->State & CIS_DISABLED) != CIS_DISABLED)
//         {
//             LastEnabledItem = CurItem;
//         }
//     }
//
//     if (LastEnabledItem != NULL)
//     {
//         /* return the Deny checkbox in case both check boxes are enabled! */
//         *CheckBox = ((!(LastEnabledItem->State & CIS_DENYDISABLED)) ? CLB_DENY : CLB_ALLOW);
//     }
//
//     return LastEnabledItem;
// }
//
// static PCHECKITEM
// FindPreviousEnabledCheckBox( PCHECKLISTWND infoPtr,
//                              UINT *CheckBox)
// {
//     PCHECKITEM Item;
//
//     if (infoPtr->FocusedCheckItem != NULL)
//     {
//         Item = infoPtr->FocusedCheckItem;
//
//         if (infoPtr->FocusedCheckItemBox == CLB_DENY &&
//             !(Item->State & CIS_ALLOWDISABLED))
//         {
//             /* currently an Deny checkbox is focused. return the Allow checkbox
//                if it's enabled */
//             *CheckBox = CLB_ALLOW;
//         }
//         else
//         {
//             PCHECKITEM CurItem;
//
//             Item = NULL;
//
//             for (CurItem = infoPtr->CheckItemListHead;
//                  CurItem != infoPtr->FocusedCheckItem;
//                  CurItem = CurItem->Next)
//             {
//                 if ((CurItem->State & CIS_DISABLED) != CIS_DISABLED)
//                 {
//                     Item = CurItem;
//                 }
//             }
//
//             if (Item != NULL)
//             {
//                 /* return the Deny checkbox in case both check boxes are enabled! */
//                 *CheckBox = ((!(Item->State & CIS_DENYDISABLED)) ? CLB_DENY : CLB_ALLOW);
//             }
//         }
//     }
//     else
//     {
//         Item = FindLastEnabledCheckBox(infoPtr,
//                                        CheckBox);
//     }
//
//     return Item;
// }
//
// static PCHECKITEM
// FindNextEnabledCheckBox( PCHECKLISTWND infoPtr,
//                          UINT *CheckBox)
// {
//     PCHECKITEM Item;
//
//     if (infoPtr->FocusedCheckItem != NULL)
//     {
//         Item = infoPtr->FocusedCheckItem;
//
//         if (infoPtr->FocusedCheckItemBox != CLB_DENY &&
//             !(Item->State & CIS_DENYDISABLED))
//         {
//             /* currently an Allow checkbox is focused. return the Deny checkbox
//                if it's enabled */
//             *CheckBox = CLB_DENY;
//         }
//         else
//         {
//             Item = Item->Next;
//
//             while (Item != NULL)
//             {
//                 if ((Item->State & CIS_DISABLED) != CIS_DISABLED)
//                 {
//                     /* return the Allow checkbox in case both check boxes are enabled! */
//                     *CheckBox = ((!(Item->State & CIS_ALLOWDISABLED)) ? CLB_ALLOW : CLB_DENY);
//                     break;
//                 }
//
//                 Item = Item->Next;
//             }
//         }
//     }
//     else
//     {
//         Item = FindFirstEnabledCheckBox(infoPtr,
//                                         CheckBox);
//     }
//
//     return Item;
// }
//
// static PCHECKITEM
// FindEnabledCheckBox( PCHECKLISTWND infoPtr,
//                      BOOL ReverseSearch,
//                      UINT *CheckBox)
// {
//     PCHECKITEM Item;
//
//     if (ReverseSearch)
//     {
//         Item = FindPreviousEnabledCheckBox(infoPtr,
//                                            CheckBox);
//     }
//     else
//     {
//         Item = FindNextEnabledCheckBox(infoPtr,
//                                        CheckBox);
//     }
//
//     return Item;
// }
//
// static PCHECKITEM
// PtToCheckItemBox( PCHECKLISTWND infoPtr,
//                   PPOINT ppt,
//                   UINT *CheckBox,
//                   BOOL *DirectlyInCheckBox)
// {
//     INT FirstVisible, Index;
//     PCHECKITEM Item;
//
//     FirstVisible = GetScrollPos(infoPtr->hSelf,
//                                 SB_VERT);
//
//     Index = FirstVisible + (ppt->y / infoPtr->ItemHeight);
//
//     Item = FindCheckItemByIndex(infoPtr,
//                                 Index);
//     if (Item != NULL)
//     {
//         INT cx;
//
//         cx = infoPtr->CheckBoxLeft[CLB_ALLOW] +
//              ((infoPtr->CheckBoxLeft[CLB_DENY] - infoPtr->CheckBoxLeft[CLB_ALLOW]) / 2);
//
//         *CheckBox = ((ppt->x <= cx) ? CLB_ALLOW : CLB_DENY);
//
//         if (DirectlyInCheckBox != NULL)
//         {
//             INT y = ppt->y % infoPtr->ItemHeight;
//             INT cxBox = infoPtr->ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT);
//
//             if ((y >= CI_TEXT_MARGIN_HEIGHT &&
//                  y < infoPtr->ItemHeight - CI_TEXT_MARGIN_HEIGHT) &&
//
//                 (((ppt->x >= (infoPtr->CheckBoxLeft[CLB_ALLOW] - (cxBox / 2))) &&
//                   (ppt->x < (infoPtr->CheckBoxLeft[CLB_ALLOW] - (cxBox / 2) + cxBox)))
//                  ||
//                  ((ppt->x >= (infoPtr->CheckBoxLeft[CLB_DENY] - (cxBox / 2))) &&
//                   (ppt->x < (infoPtr->CheckBoxLeft[CLB_DENY] - (cxBox / 2) + cxBox)))))
//             {
//                 *DirectlyInCheckBox = TRUE;
//             }
//             else
//             {
//                 *DirectlyInCheckBox = FALSE;
//             }
//         }
//     }
//
//     return Item;
// }
//
// static VOID
// ClearCheckItems( PCHECKLISTWND infoPtr)
// {
//     PCHECKITEM CurItem, NextItem;
//
//     CurItem = infoPtr->CheckItemListHead;
//     while (CurItem != NULL)
//     {
//         NextItem = CurItem->Next;
//         HeapFree(GetProcessHeap(),
//                  0,
//                  CurItem);
//         CurItem = NextItem;
//     }
//
//     infoPtr->CheckItemListHead = NULL;
//     infoPtr->CheckItemCount = 0;
// }
//
// static BOOL
// DeleteCheckItem( PCHECKLISTWND infoPtr,
//                  PCHECKITEM Item)
// {
//     PCHECKITEM CurItem;
//     PCHECKITEM *PrevPtr = &infoPtr->CheckItemListHead;
//
//     for (CurItem = infoPtr->CheckItemListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if (CurItem == Item)
//         {
//             if (Item == infoPtr->QuickSearchHitItem && infoPtr->QuickSearchEnabled)
//             {
//                 EscapeQuickSearch(infoPtr);
//             }
//
// #if SUPPORT_UXTHEME
//             if (Item == infoPtr->HoveredCheckItem)
//             {
//                 ChangeCheckItemHotTrack(infoPtr,
//                                         NULL,
//                                         0);
//             }
// #endif
//
//             if (Item == infoPtr->FocusedCheckItem)
//             {
//                 ChangeCheckItemFocus(infoPtr,
//                                      NULL,
//                                      0);
//             }
//
//             *PrevPtr = CurItem->Next;
//             HeapFree(GetProcessHeap(),
//                      0,
//                      CurItem);
//             infoPtr->CheckItemCount--;
//             return TRUE;
//         }
//
//         PrevPtr = &CurItem->Next;
//     }
//
//     return FALSE;
// }
//
// static PCHECKITEM
// AddCheckItem( PCHECKLISTWND infoPtr,
//               LPWSTR Name,
//               DWORD State,
//               ACCESS_MASK AccessMask,
//               INT *Index)
// {
//     PCHECKITEM CurItem;
//     INT i;
//     PCHECKITEM *PrevPtr = &infoPtr->CheckItemListHead;
//     PCHECKITEM Item = HeapAlloc(GetProcessHeap(),
//                                 0,
//                                 sizeof(CHECKITEM) + (wcslen(Name) * sizeof(WCHAR)));
//     if (Item != NULL)
//     {
//         for (CurItem = infoPtr->CheckItemListHead, i = 0;
//              CurItem != NULL;
//              CurItem = CurItem->Next)
//         {
//             PrevPtr = &CurItem->Next;
//             i++;
//         }
//
//         Item->Next = NULL;
//         Item->AccessMask = AccessMask;
//         Item->State = State & CIS_MASK;
//         wcscpy(Item->Name,
//                Name);
//
//         *PrevPtr = Item;
//         infoPtr->CheckItemCount++;
//
//         if (Index != NULL)
//         {
//             *Index = i;
//         }
//     }
//
//     return Item;
// }
//
// static UINT
// ClearCheckBoxes( PCHECKLISTWND infoPtr)
// {
//     PCHECKITEM CurItem;
//     UINT nUpdated = 0;
//
//     for (CurItem = infoPtr->CheckItemListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if (CurItem->State & (CIS_ALLOW | CIS_DENY))
//         {
//             CurItem->State &= ~(CIS_ALLOW | CIS_DENY);
//             nUpdated++;
//         }
//     }
//
//     return nUpdated;
// }
//
// static VOID
// UpdateControl( PCHECKLISTWND infoPtr)
// {
//     RECT rcClient;
//     SCROLLINFO ScrollInfo;
//     INT VisibleItems;
//
//     GetClientRect(infoPtr->hSelf,
//                   &rcClient);
//
//     ScrollInfo.cbSize = sizeof(ScrollInfo);
//     ScrollInfo.fMask = SIF_PAGE | SIF_RANGE;
//     ScrollInfo.nMin = 0;
//     ScrollInfo.nMax = infoPtr->CheckItemCount;
//     ScrollInfo.nPage = ((rcClient.bottom - rcClient.top) + infoPtr->ItemHeight - 1) / infoPtr->ItemHeight;
//     ScrollInfo.nPos = 0;
//     ScrollInfo.nTrackPos = 0;
//
//     VisibleItems = (rcClient.bottom - rcClient.top) / infoPtr->ItemHeight;
//
//     if (ScrollInfo.nPage == (UINT)VisibleItems && ScrollInfo.nMax > 0)
//     {
//         ScrollInfo.nMax--;
//     }
//
//     SetScrollInfo(infoPtr->hSelf,
//                   SB_VERT,
//                   &ScrollInfo,
//                   TRUE);
//
//     RedrawWindow(infoPtr->hSelf,
//                  NULL,
//                  NULL,
//                  RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
// }
//
// static VOID
// UpdateCheckItem( PCHECKLISTWND infoPtr,
//                  PCHECKITEM Item)
// {
//     RECT rcClient;
//     INT VisibleFirst, VisibleItems;
//     INT Index = CheckItemToIndex(infoPtr,
//                                  Item);
//     if (Index != -1)
//     {
//         VisibleFirst = GetScrollPos(infoPtr->hSelf,
//                                     SB_VERT);
//
//         if (Index >= VisibleFirst)
//         {
//             GetClientRect(infoPtr->hSelf,
//                           &rcClient);
//
//             VisibleItems = ((rcClient.bottom - rcClient.top) + infoPtr->ItemHeight - 1) / infoPtr->ItemHeight;
//
//             if (Index <= VisibleFirst + VisibleItems)
//             {
//                 RECT rcUpdate;
//
//                 rcUpdate.left = rcClient.left;
//                 rcUpdate.right = rcClient.right;
//                 rcUpdate.top = (Index - VisibleFirst) * infoPtr->ItemHeight;
//                 rcUpdate.bottom = rcUpdate.top + infoPtr->ItemHeight;
//
//                 RedrawWindow(infoPtr->hSelf,
//                              &rcUpdate,
//                              NULL,
//                              RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
//             }
//         }
//     }
// }
//
// static VOID
// MakeCheckItemVisible( PCHECKLISTWND infoPtr,
//                       PCHECKITEM Item)
// {
//     RECT rcClient;
//     INT VisibleFirst, VisibleItems, NewPos;
//     INT Index = CheckItemToIndex(infoPtr,
//                                  Item);
//     if (Index != -1)
//     {
//         VisibleFirst = GetScrollPos(infoPtr->hSelf,
//                                     SB_VERT);
//
//         if (Index <= VisibleFirst)
//         {
//             NewPos = Index;
//         }
//         else
//         {
//             GetClientRect(infoPtr->hSelf,
//                           &rcClient);
//
//             VisibleItems = (rcClient.bottom - rcClient.top) / infoPtr->ItemHeight;
//             if (Index - VisibleItems + 1 > VisibleFirst)
//             {
//                 NewPos = Index - VisibleItems + 1;
//             }
//             else
//             {
//                 NewPos = VisibleFirst;
//             }
//         }
//
//         if (VisibleFirst != NewPos)
//         {
//             SCROLLINFO ScrollInfo;
//
//             ScrollInfo.cbSize = sizeof(ScrollInfo);
//             ScrollInfo.fMask = SIF_POS;
//             ScrollInfo.nPos = NewPos;
//             NewPos = SetScrollInfo(infoPtr->hSelf,
//                                    SB_VERT,
//                                    &ScrollInfo,
//                                    TRUE);
//
//             if (VisibleFirst != NewPos)
//             {
//                 ScrollWindowEx(infoPtr->hSelf,
//                                0,
//                                (NewPos - VisibleFirst) * infoPtr->ItemHeight,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                SW_INVALIDATE | SW_SCROLLCHILDREN);
//
//                 RedrawWindow(infoPtr->hSelf,
//                              NULL,
//                              NULL,
//                              RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
//             }
//         }
//     }
// }
//
// static UINT
// GetIdealItemHeight( PCHECKLISTWND infoPtr)
// {
//     HDC hdc = GetDC(infoPtr->hSelf);
//     if(hdc != NULL)
//     {
//         UINT height;
//         TEXTMETRIC tm;
//         HGDIOBJ hOldFont = SelectObject(hdc,
//                                         infoPtr->hFont);
//
//         if(GetTextMetrics(hdc,
//                           &tm))
//         {
//             height = tm.tmHeight;
//         }
//         else
//         {
//             height = 2;
//         }
//
//         SelectObject(hdc,
//                      hOldFont);
//
//         ReleaseDC(infoPtr->hSelf,
//                   hdc);
//
//         return height;
//     }
//     return 0;
// }
//
// static HFONT
// RetChangeControlFont( PCHECKLISTWND infoPtr,
//                       HFONT hFont,
//                       BOOL Redraw)
// {
//     HFONT hOldFont = infoPtr->hFont;
//     infoPtr->hFont = hFont;
//
//     if (hOldFont != hFont)
//     {
//         infoPtr->ItemHeight = (2 * CI_TEXT_MARGIN_HEIGHT) + GetIdealItemHeight(infoPtr);
//     }
//
//     if (infoPtr->ShowingCaret)
//     {
//         DestroyCaret();
//         CreateCaret(infoPtr->hSelf,
//                     NULL,
//                     0,
//                     infoPtr->ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT));
//     }
//
//     UpdateControl(infoPtr);
//
//     return hOldFont;
// }
//
// #if SUPPORT_UXTHEME
// static INT
// CalculateCheckBoxStyle( BOOL Checked,
//                         BOOL Enabled,
//                         BOOL HotTrack,
//                         BOOL Pushed)
// {
//     INT BtnState;
//
//     if (Checked)
//     {
//         BtnState = (Enabled ?
//                     (Pushed ? CBS_CHECKEDPRESSED : (HotTrack ? CBS_CHECKEDHOT : CBS_CHECKEDNORMAL)) :
//                     CBS_CHECKEDDISABLED);
//     }
//     else
//     {
//         BtnState = (Enabled ?
//                     (Pushed ? CBS_UNCHECKEDPRESSED : (HotTrack ? CBS_UNCHECKEDHOT : CBS_UNCHECKEDNORMAL)) :
//                     CBS_UNCHECKEDDISABLED);
//     }
//
//     return BtnState;
// }
// #endif
//
// static VOID
// PaintControl( PCHECKLISTWND infoPtr,
//               HDC hDC,
//               PRECT rcUpdate)
// {
//     INT ScrollPos;
//     PCHECKITEM FirstItem, Item;
//     RECT rcClient;
//     UINT VisibleFirstIndex = rcUpdate->top / infoPtr->ItemHeight;
//     UINT LastTouchedIndex = rcUpdate->bottom / infoPtr->ItemHeight;
//
//     FillRect(hDC,
//              rcUpdate,
//              (HBRUSH)(COLOR_WINDOW + 1));
//
//     GetClientRect(infoPtr->hSelf,
//                   &rcClient);
//
//     ScrollPos = GetScrollPos(infoPtr->hSelf,
//                              SB_VERT);
//
//     FirstItem = FindCheckItemByIndex(infoPtr,
//                                      ScrollPos + VisibleFirstIndex);
//     if (FirstItem != NULL)
//     {
//         RECT TextRect, ItemRect, CheckBox;
//         HFONT hOldFont;
//         DWORD CurrentIndex;
//         COLORREF OldTextColor;
//         BOOL Enabled, PrevEnabled, IsPushed;
//         POINT hOldBrushOrg;
// #if SUPPORT_UXTHEME
//         HRESULT hDrawResult;
//         BOOL ItemHovered;
// #endif
//
//         Enabled = IsWindowEnabled(infoPtr->hSelf);
//         PrevEnabled = Enabled;
//
//         ItemRect.left = 0;
//         ItemRect.right = rcClient.right;
//         ItemRect.top = VisibleFirstIndex * infoPtr->ItemHeight;
//
//         TextRect.left = ItemRect.left + CI_TEXT_MARGIN_WIDTH;
//         TextRect.right = ItemRect.right - CI_TEXT_MARGIN_WIDTH;
//         TextRect.top = ItemRect.top + CI_TEXT_MARGIN_HEIGHT;
//
//         SetBrushOrgEx(hDC,
//                       ItemRect.left,
//                       ItemRect.top,
//                       &hOldBrushOrg);
//
//         OldTextColor = SetTextColor(hDC,
//                                     infoPtr->TextColor[Enabled]);
//
//         hOldFont = SelectObject(hDC,
//                                 infoPtr->hFont);
//
//         for (Item = FirstItem, CurrentIndex = VisibleFirstIndex;
//              Item != NULL && CurrentIndex <= LastTouchedIndex;
//              Item = Item->Next, CurrentIndex++)
//         {
//             TextRect.bottom = TextRect.top + infoPtr->ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT);
//             ItemRect.bottom = ItemRect.top + infoPtr->ItemHeight;
//
//             SetBrushOrgEx(hDC,
//                           ItemRect.left,
//                           ItemRect.top,
//                           NULL);
//
//             if (Enabled && PrevEnabled != ((Item->State & CIS_DISABLED) != CIS_DISABLED))
//             {
//                 PrevEnabled = ((Item->State & CIS_DISABLED) != CIS_DISABLED);
//
//                 SetTextColor(hDC,
//                              infoPtr->TextColor[PrevEnabled]);
//             }
//
// #if SUPPORT_UXTHEME
//             ItemHovered = (Enabled && infoPtr->HoveredCheckItem == Item);
// #endif
//
//             if (infoPtr->QuickSearchHitItem == Item)
//             {
//                 COLORREF OldBkColor, OldFgColor;
//                 SIZE TextSize;
//                 SIZE_T TextLen, HighlightLen = wcslen(infoPtr->QuickSearchText);
//
//                 /* highlight the quicksearch text */
//                 if (GetTextExtentPoint32(hDC,
//                                          Item->Name,
//                                          HighlightLen,
//                                          &TextSize))
//                 {
//                     COLORREF HighlightTextColor, HighlightBackground;
//                     RECT rcHighlight = TextRect;
//
//                     HighlightTextColor = GetSysColor(COLOR_HIGHLIGHTTEXT);
//                     HighlightBackground = GetSysColor(COLOR_HIGHLIGHT);
//
//                     rcHighlight.right = rcHighlight.left + TextSize.cx;
//
//                     InflateRect(&rcHighlight,
//                                 0,
//                                 CI_TEXT_SELECTIONMARGIN);
//
//                     OldBkColor = SetBkColor(hDC,
//                                             HighlightBackground);
//                     OldFgColor = SetTextColor(hDC,
//                                               HighlightTextColor);
//
//                     /* draw the highlighted text */
//                     DrawText(hDC,
//                              Item->Name,
//                              HighlightLen,
//                              &rcHighlight,
//                              DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER);
//
//                     SetBkColor(hDC,
//                                OldBkColor);
//                     SetTextColor(hDC,
//                                  OldFgColor);
//
//                     /* draw the remaining part of the text */
//                     TextLen = wcslen(Item->Name);
//                     if (HighlightLen < TextLen)
//                     {
//                         rcHighlight.left = rcHighlight.right;
//                         rcHighlight.right = TextRect.right;
//
//                         DrawText(hDC,
//                                  Item->Name + HighlightLen,
//                                  -1,
//                                  &rcHighlight,
//                                  DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER);
//                     }
//                 }
//             }
//             else
//             {
//                 /* draw the text */
//                 DrawText(hDC,
//                          Item->Name,
//                          -1,
//                          &TextRect,
//                          DT_LEFT | DT_NOPREFIX | DT_SINGLELINE | DT_VCENTER);
//             }
//
//             CheckBox.top = TextRect.top;
//             CheckBox.bottom = TextRect.bottom;
//
//             /* draw the Allow checkbox */
//             IsPushed = (Enabled && Item == infoPtr->FocusedCheckItem && infoPtr->HasFocus &&
//                         !(Item->State & CIS_ALLOWDISABLED) && infoPtr->FocusedCheckItemBox != CLB_DENY &&
//                         infoPtr->FocusedPushed);
//
//             CheckBox.left = infoPtr->CheckBoxLeft[CLB_ALLOW] - ((TextRect.bottom - TextRect.top) / 2);
//             CheckBox.right = CheckBox.left + (TextRect.bottom - TextRect.top);
// #if SUPPORT_UXTHEME
//             if (infoPtr->ThemeHandle != NULL)
//             {
//                 INT BtnState = CalculateCheckBoxStyle(Item->State & CIS_ALLOW,
//                                                       Enabled && !(Item->State & CIS_ALLOWDISABLED),
//                                                       (ItemHovered && infoPtr->HoveredCheckItemBox != CLB_DENY),
//                                                       IsPushed);
//
//
//                 hDrawResult = DrawThemeBackground(infoPtr->ThemeHandle,
//                                                   hDC,
//                                                   BP_CHECKBOX,
//                                                   BtnState,
//                                                   &CheckBox,
//                                                   NULL);
//
//             }
//             else
//             {
//                 hDrawResult = E_FAIL;
//             }
//
//             /* draw the standard checkbox if no themes are enabled or drawing the
//                themed control failed */
//             if (FAILED!(hDrawResult))
// #endif
//             {
//                 DrawFrameControl(hDC,
//                                  &CheckBox,
//                                  DFC_BUTTON,
//                                  DFCS_BUTTONCHECK | DFCS_FLAT |
//                                  ((Item->State & CIS_ALLOWDISABLED) || !Enabled ? DFCS_INACTIVE : 0) |
//                                  ((Item->State & CIS_ALLOW) ? DFCS_CHECKED : 0) |
//                                  (IsPushed ? DFCS_PUSHED : 0));
//             }
//             if (Item == infoPtr->FocusedCheckItem && !(infoPtr->UIState & UISF_HIDEFOCUS) &&
//                 infoPtr->HasFocus &&
//                 infoPtr->FocusedCheckItemBox != CLB_DENY)
//             {
//                 RECT rcFocus = CheckBox;
//
//                 InflateRect (&rcFocus,
//                              CI_TEXT_MARGIN_HEIGHT,
//                              CI_TEXT_MARGIN_HEIGHT);
//
//                 DrawFocusRect(hDC,
//                               &rcFocus);
//             }
//
//             /* draw the Deny checkbox */
//             IsPushed = (Enabled && Item == infoPtr->FocusedCheckItem && infoPtr->HasFocus &&
//                         !(Item->State & CIS_DENYDISABLED) && infoPtr->FocusedCheckItemBox == CLB_DENY &&
//                         infoPtr->FocusedPushed);
//
//             CheckBox.left = infoPtr->CheckBoxLeft[CLB_DENY] - ((TextRect.bottom - TextRect.top) / 2);
//             CheckBox.right = CheckBox.left + (TextRect.bottom - TextRect.top);
// #if SUPPORT_UXTHEME
//             if (infoPtr->ThemeHandle != NULL)
//             {
//                 INT BtnState = CalculateCheckBoxStyle(Item->State & CIS_DENY,
//                                                       Enabled && !(Item->State & CIS_DENYDISABLED),
//                                                       (ItemHovered && infoPtr->HoveredCheckItemBox == CLB_DENY),
//                                                       IsPushed);
//
//                 hDrawResult = DrawThemeBackground(infoPtr->ThemeHandle,
//                                                   hDC,
//                                                   BP_CHECKBOX,
//                                                   BtnState,
//                                                   &CheckBox,
//                                                   NULL);
//
//             }
//             else
//             {
//                 hDrawResult = E_FAIL;
//             }
//
//             /* draw the standard checkbox if no themes are enabled or drawing the
//                themed control failed */
//             if (FAILED!(hDrawResult))
// #endif
//             {
//                 DrawFrameControl(hDC,
//                                  &CheckBox,
//                                  DFC_BUTTON,
//                                  DFCS_BUTTONCHECK | DFCS_FLAT |
//                                  ((Item->State & CIS_DENYDISABLED) || !Enabled ? DFCS_INACTIVE : 0) |
//                                  ((Item->State & CIS_DENY) ? DFCS_CHECKED : 0) |
//                                  (IsPushed ? DFCS_PUSHED : 0));
//             }
//             if (infoPtr->HasFocus && !(infoPtr->UIState & UISF_HIDEFOCUS) &&
//                 Item == infoPtr->FocusedCheckItem &&
//                 infoPtr->FocusedCheckItemBox == CLB_DENY)
//             {
//                 RECT rcFocus = CheckBox;
//
//                 InflateRect (&rcFocus,
//                              CI_TEXT_MARGIN_HEIGHT,
//                              CI_TEXT_MARGIN_HEIGHT);
//
//                 DrawFocusRect(hDC,
//                               &rcFocus);
//             }
//
//             TextRect.top += infoPtr->ItemHeight;
//             ItemRect.top += infoPtr->ItemHeight;
//         }
//
//         SelectObject(hDC,
//                      hOldFont);
//
//         SetTextColor(hDC,
//                      OldTextColor);
//
//         SetBrushOrgEx(hDC,
//                       hOldBrushOrg.x,
//                       hOldBrushOrg.y,
//                       NULL);
//     }
// }
//
// static VOID
// ChangeCheckItemFocus( PCHECKLISTWND infoPtr,
//                       PCHECKITEM NewFocus,
//                       UINT NewFocusBox)
// {
//     if (NewFocus != infoPtr->FocusedCheckItem)
//     {
//         PCHECKITEM OldFocus = infoPtr->FocusedCheckItem;
//         infoPtr->FocusedCheckItem = NewFocus;
//         infoPtr->FocusedCheckItemBox = NewFocusBox;
//
//         if (OldFocus != NULL)
//         {
//             UpdateCheckItem(infoPtr,
//                             OldFocus);
//         }
//     }
//     else
//     {
//         infoPtr->FocusedCheckItemBox = NewFocusBox;
//     }
//
//     if (NewFocus != NULL)
//     {
//         MakeCheckItemVisible(infoPtr,
//                              NewFocus);
//         UpdateCheckItem(infoPtr,
//                         NewFocus);
//     }
// }
//
// static VOID
// UpdateCheckItemBox( PCHECKLISTWND infoPtr,
//                     PCHECKITEM Item,
//                     UINT ItemBox)
// {
//     RECT rcClient;
//     INT VisibleFirst, VisibleItems;
//     INT Index = CheckItemToIndex(infoPtr,
//                                  Item);
//     if (Index != -1)
//     {
//         VisibleFirst = GetScrollPos(infoPtr->hSelf,
//                                     SB_VERT);
//
//         if (Index >= VisibleFirst)
//         {
//             GetClientRect(infoPtr->hSelf,
//                           &rcClient);
//
//             VisibleItems = ((rcClient.bottom - rcClient.top) + infoPtr->ItemHeight - 1) / infoPtr->ItemHeight;
//
//             if (Index <= VisibleFirst + VisibleItems)
//             {
//                 RECT rcUpdate;
//
//                 rcUpdate.left = rcClient.left + infoPtr->CheckBoxLeft[ItemBox] - (infoPtr->ItemHeight / 2);
//                 rcUpdate.right = rcUpdate.left + infoPtr->ItemHeight;
//                 rcUpdate.top = ((Index - VisibleFirst) * infoPtr->ItemHeight);
//                 rcUpdate.bottom = rcUpdate.top + infoPtr->ItemHeight;
//
//                 RedrawWindow(infoPtr->hSelf,
//                              &rcUpdate,
//                              NULL,
//                              RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
//             }
//         }
//     }
// }
//
// #if SUPPORT_UXTHEME
// static VOID
// ChangeCheckItemHotTrack( PCHECKLISTWND infoPtr,
//                          PCHECKITEM NewHotTrack,
//                          UINT NewHotTrackBox)
// {
//     if (NewHotTrack != infoPtr->HoveredCheckItem)
//     {
//         PCHECKITEM OldHotTrack = infoPtr->HoveredCheckItem;
//         UINT OldHotTrackBox = infoPtr->HoveredCheckItemBox;
//
//         infoPtr->HoveredCheckItem = NewHotTrack;
//         infoPtr->HoveredCheckItemBox = NewHotTrackBox;
//
//         if (OldHotTrack != NULL)
//         {
//             UpdateCheckItemBox(infoPtr,
//                                OldHotTrack,
//                                OldHotTrackBox);
//         }
//     }
//     else
//     {
//         infoPtr->HoveredCheckItemBox = NewHotTrackBox;
//     }
//
//     if (NewHotTrack != NULL)
//     {
//         UpdateCheckItemBox(infoPtr,
//                            NewHotTrack,
//                            NewHotTrackBox);
//     }
// }
// #endif
//
// static BOOL
// ChangeCheckBox( PCHECKLISTWND infoPtr,
//                 PCHECKITEM CheckItem,
//                 UINT CheckItemBox)
// {
//     NMCHANGEITEMCHECKBOX CheckData;
//     DWORD OldState = CheckItem->State;
//     DWORD CheckedBit = ((infoPtr->FocusedCheckItemBox == CLB_DENY) ? CIS_DENY : CIS_ALLOW);
//     BOOL Checked = (CheckItem->State & CheckedBit) != 0;
//
//     CheckData.OldState = OldState;
//     CheckData.NewState = (Checked ? OldState & ~CheckedBit : OldState | CheckedBit);
//     CheckData.CheckBox = infoPtr->FocusedCheckItemBox;
//     CheckData.Checked = !Checked;
//
//     if (NotifyControlParent(infoPtr,
//                             CLN_CHANGINGITEMCHECKBOX,
//                             &CheckData) != (LRESULT)-1)
//     {
//         CheckItem->State = CheckData.NewState;
//     }
//
//     return (CheckItem->State != OldState);
// }
//
// static VOID
// DisplayCaret( PCHECKLISTWND infoPtr)
// {
//     if (IsWindowEnabled(infoPtr->hSelf) && !infoPtr->ShowingCaret)
//     {
//         infoPtr->ShowingCaret = TRUE;
//
//         CreateCaret(infoPtr->hSelf,
//                     NULL,
//                     infoPtr->CaretWidth,
//                     infoPtr->ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT));
//
//         ShowCaret(infoPtr->hSelf);
//     }
// }
//
// static VOID
// RemoveCaret( PCHECKLISTWND infoPtr)
// {
//     if (IsWindowEnabled(infoPtr->hSelf) && infoPtr->ShowingCaret)
//     {
//         infoPtr->ShowingCaret = FALSE;
//
//         HideCaret(infoPtr->hSelf);
//         DestroyCaret();
//     }
// }
//
// static VOID
// KillQuickSearchTimers( PCHECKLISTWND infoPtr)
// {
//     KillTimer(infoPtr->hSelf,
//               TIMER_ID_SETHITFOCUS);
//     KillTimer(infoPtr->hSelf,
//               TIMER_ID_RESETQUICKSEARCH);
// }
//
// static VOID
// MapItemToRect( PCHECKLISTWND infoPtr,
//                PCHECKITEM CheckItem,
//                RECT *prcItem)
// {
//     INT Index = CheckItemToIndex(infoPtr,
//                                  CheckItem);
//     if (Index != -1)
//     {
//         RECT rcClient;
//         INT VisibleFirst;
//
//         GetClientRect(infoPtr->hSelf,
//                       &rcClient);
//
//         VisibleFirst = GetScrollPos(infoPtr->hSelf,
//                                     SB_VERT);
//
//         prcItem->left = rcClient.left;
//         prcItem->right = rcClient.right;
//         prcItem->top = (Index - VisibleFirst) * infoPtr->ItemHeight;
//         prcItem->bottom = prcItem->top + infoPtr->ItemHeight;
//     }
//     else
//     {
//         prcItem->left = 0;
//         prcItem->top = 0;
//         prcItem->right = 0;
//         prcItem->bottom = 0;
//     }
// }
//
// static VOID
// UpdateCaretPos( PCHECKLISTWND infoPtr)
// {
//     if (infoPtr->ShowingCaret && infoPtr->QuickSearchHitItem != NULL)
//     {
//         HDC hDC = GetDC(infoPtr->hSelf);
//         if (hDC != NULL)
//         {
//             SIZE TextSize;
//             HGDIOBJ hOldFont = SelectObject(hDC,
//                                             infoPtr->hFont);
//
//             TextSize.cx = 0;
//             TextSize.cy = 0;
//
//             if (infoPtr->QuickSearchText[0] == '\0' ||
//                 GetTextExtentPoint32(hDC,
//                                      infoPtr->QuickSearchHitItem->Name,
//                                      wcslen(infoPtr->QuickSearchText),
//                                      &TextSize))
//             {
//                 RECT rcItem;
//
//                 MapItemToRect(infoPtr,
//                               infoPtr->QuickSearchHitItem,
//                               &rcItem);
//
//                 /* actually change the caret position */
//                 SetCaretPos(rcItem.left + CI_TEXT_MARGIN_WIDTH + TextSize.cx,
//                             rcItem.top + CI_TEXT_MARGIN_HEIGHT);
//             }
//
//             SelectObject(hDC,
//                          hOldFont);
//
//             ReleaseDC(infoPtr->hSelf,
//                       hDC);
//         }
//     }
// }
//
// static VOID
// EscapeQuickSearch( PCHECKLISTWND infoPtr)
// {
//     if (infoPtr->QuickSearchEnabled && infoPtr->QuickSearchHitItem != NULL)
//     {
//         PCHECKITEM OldHit = infoPtr->QuickSearchHitItem;
//
//         infoPtr->QuickSearchHitItem = NULL;
//         infoPtr->QuickSearchText[0] = '\0';
//
//         /* scroll back to the focused item */
//         if (infoPtr->FocusedCheckItem != NULL)
//         {
//             MakeCheckItemVisible(infoPtr,
//                                  infoPtr->FocusedCheckItem);
//         }
//
//         /* repaint the old search hit item if it's still visible */
//         UpdateCheckItem(infoPtr,
//                         OldHit);
//
//         KillQuickSearchTimers(infoPtr);
//
//         RemoveCaret(infoPtr);
//     }
// }
//
// static VOID
// ChangeSearchHit( PCHECKLISTWND infoPtr,
//                  PCHECKITEM NewHit)
// {
//     PCHECKITEM OldHit = infoPtr->QuickSearchHitItem;
//
//     infoPtr->QuickSearchHitItem = NewHit;
//
//     if (OldHit != NewHit)
//     {
//         /* scroll to the new search hit */
//         MakeCheckItemVisible(infoPtr,
//                              NewHit);
//
//         /* repaint the old hit if present and visible */
//         if (OldHit != NULL)
//         {
//             UpdateCheckItem(infoPtr,
//                             OldHit);
//         }
//         else
//         {
//             /* show the caret the first time we find an item */
//              DisplayCaret(infoPtr);
//         }
//     }
//
//     UpdateCaretPos(infoPtr);
//
//     UpdateCheckItem(infoPtr,
//                     NewHit);
//
//     /* kill the reset timer and restart the set hit focus timer */
//     KillTimer(infoPtr->hSelf,
//               TIMER_ID_RESETQUICKSEARCH);
//     if (infoPtr->QuickSearchSetFocusDelay != 0)
//     {
//         SetTimer(infoPtr->hSelf,
//                  TIMER_ID_SETHITFOCUS,
//                  infoPtr->QuickSearchSetFocusDelay,
//                  NULL);
//     }
// }
//
// static BOOL
// QuickSearchFindHit( PCHECKLISTWND infoPtr,
//                     WCHAR c)
// {
//     if (infoPtr->QuickSearchEnabled)
//     {
//         BOOL Ret = FALSE;
//         PCHECKITEM NewHit;
//
//         switch (c)
//         {
//             case '\r':
//             case '\n':
//             {
//                 Ret = infoPtr->QuickSearchHitItem != NULL;
//                 if (Ret)
//                 {
//                     /* NOTE: QuickSearchHitItem definitely has at least one
//                              enabled check box, the user can't search for disabled
//                              check items */
//
//                     ChangeCheckItemFocus(infoPtr,
//                                          infoPtr->QuickSearchHitItem,
//                                          ((!(infoPtr->QuickSearchHitItem->State & CIS_ALLOWDISABLED)) ? CLB_ALLOW : CLB_DENY));
//
//                     EscapeQuickSearch(infoPtr);
//                 }
//                 break;
//             }
//
//             case VK_BACK:
//             {
//                 if (infoPtr->QuickSearchHitItem != NULL)
//                 {
//                     INT SearchLen = wcslen(infoPtr->QuickSearchText);
//                     if (SearchLen > 0)
//                     {
//                         /* delete the last character */
//                         infoPtr->QuickSearchText[--SearchLen] = '\0';
//
//                         if (SearchLen > 0)
//                         {
//                             /* search again */
//                             NewHit = FindCheckItem(infoPtr,
//                                                    infoPtr->QuickSearchText);
//
//                             if (NewHit != NULL)
//                             {
//                                 /* change the search hit */
//                                 ChangeSearchHit(infoPtr,
//                                                 NewHit);
//
//                                 Ret = TRUE;
//                             }
//                         }
//                     }
//
//                     if (!Ret)
//                     {
//                         EscapeQuickSearch(infoPtr);
//                     }
//                 }
//                 break;
//             }
//
//             default:
//             {
//                 INT SearchLen = wcslen(infoPtr->QuickSearchText);
//                 if (SearchLen < (INT)(sizeof(infoPtr->QuickSearchText) / sizeof(infoPtr->QuickSearchText[0])) - 1)
//                 {
//                     infoPtr->QuickSearchText[SearchLen++] = c;
//                     infoPtr->QuickSearchText[SearchLen] = '\0';
//
//                     NewHit = FindCheckItem(infoPtr,
//                                            infoPtr->QuickSearchText);
//                     if (NewHit != NULL)
//                     {
//                         /* change the search hit */
//                         ChangeSearchHit(infoPtr,
//                                         NewHit);
//
//                         Ret = TRUE;
//                     }
//                     else
//                     {
//                         /* reset the input */
//                         infoPtr->QuickSearchText[--SearchLen] = '\0';
//                     }
//                 }
//                 break;
//             }
//         }
//         return Ret;
//     }
//
//     return FALSE;
// }
//
// static LRESULT __stdcall
// CheckListWndProc( HWND hwnd,
//                   UINT uMsg,
//                   WPARAM wParam,
//                   LPARAM lParam)
// {
//     PCHECKLISTWND infoPtr;
//     LRESULT Ret;
//
//     infoPtr = (PCHECKLISTWND)GetWindowLongPtr(hwnd,
//                                               0);
//
//     if (infoPtr == NULL && uMsg != WM_CREATE)
//     {
//         goto HandleDefaultMessage;
//     }
//
//     Ret = 0;
//
//     switch (uMsg)
//     {
//         case WM_PAINT:
//         {
//             HDC hdc;
//             RECT rcUpdate;
//             PAINTSTRUCT ps;
//
//             if (GetUpdateRect(hwnd,
//                               &rcUpdate,
//                               FALSE))
//             {
//                 hdc = (wParam != 0 ? (HDC)wParam : BeginPaint(hwnd, &ps));
//
//                 if (hdc != NULL)
//                 {
//                     PaintControl(infoPtr,
//                                  hdc,
//                                  &rcUpdate);
//
//                     if (wParam == 0)
//                     {
//                         EndPaint(hwnd,
//                                  &ps);
//                     }
//                 }
//             }
//             break;
//         }
//
//         case WM_MOUSEMOVE:
//         {
//             POINT pt;
//             BOOL InCheckBox;
//             HWND hWndCapture = GetCapture();
//
//             pt.x = (LONG)LOWORD!(lParam);
//             pt.y = (LONG)HIWORD!(lParam);
//
// #if SUPPORT_UXTHEME
//             /* handle hovering checkboxes */
//             if (hWndCapture == NULL && infoPtr->ThemeHandle != NULL)
//             {
//                 TRACKMOUSEEVENT tme;
//                 PCHECKITEM HotTrackItem;
//                 UINT HotTrackItemBox;
//
//                 HotTrackItem = PtToCheckItemBox(infoPtr,
//                                                 &pt,
//                                                 &HotTrackItemBox,
//                                                 &InCheckBox);
//                 if (HotTrackItem != NULL && InCheckBox)
//                 {
//                     if (infoPtr->HoveredCheckItem != HotTrackItem ||
//                         infoPtr->HoveredCheckItemBox != HotTrackItemBox)
//                     {
//                         ChangeCheckItemHotTrack(infoPtr,
//                                                 HotTrackItem,
//                                                 HotTrackItemBox);
//                     }
//                 }
//                 else
//                 {
//                     ChangeCheckItemHotTrack(infoPtr,
//                                             NULL,
//                                             0);
//                 }
//
//                 tme.cbSize = sizeof(tme);
//                 tme.dwFlags = TME_LEAVE;
//                 tme.hwndTrack = hwnd;
//                 tme.dwHoverTime = infoPtr->HoverTime;
//
//                 TrackMouseEvent(&tme);
//             }
// #endif
//
//             if (hWndCapture == hwnd && infoPtr->FocusedCheckItem != NULL)
//             {
//                 PCHECKITEM PtItem;
//                 UINT PtItemBox;
//                 UINT OldPushed;
//
//                 PtItem = PtToCheckItemBox(infoPtr,
//                                           &pt,
//                                           &PtItemBox,
//                                           &InCheckBox);
//
//                 OldPushed = infoPtr->FocusedPushed;
//                 infoPtr->FocusedPushed = InCheckBox && infoPtr->FocusedCheckItem == PtItem &&
//                                          infoPtr->FocusedCheckItemBox == PtItemBox;
//
//                 if (OldPushed != infoPtr->FocusedPushed)
//                 {
//                     UpdateCheckItemBox(infoPtr,
//                                        infoPtr->FocusedCheckItem,
//                                        infoPtr->FocusedCheckItemBox);
//                 }
//             }
//
//             break;
//         }
//
//         case WM_VSCROLL:
//         {
//             SCROLLINFO ScrollInfo;
//
//             ScrollInfo.cbSize = sizeof(ScrollInfo);
//             ScrollInfo.fMask = SIF_RANGE | SIF_POS;
//
//             if (GetScrollInfo(hwnd,
//                               SB_VERT,
//                               &ScrollInfo))
//             {
//                 INT OldPos = ScrollInfo.nPos;
//
//                 switch (LOWORD!(wParam))
//                 {
//                     case SB_BOTTOM:
//                         ScrollInfo.nPos = ScrollInfo.nMax;
//                         break;
//
//                     case SB_LINEDOWN:
//                         if (ScrollInfo.nPos < ScrollInfo.nMax)
//                         {
//                             ScrollInfo.nPos++;
//                         }
//                         break;
//
//                     case SB_LINEUP:
//                         if (ScrollInfo.nPos > 0)
//                         {
//                             ScrollInfo.nPos--;
//                         }
//                         break;
//
//                     case SB_PAGEDOWN:
//                     {
//                         RECT rcClient;
//                         INT ScrollLines;
//
//                         /* don't use ScrollInfo.nPage because we should only scroll
//                            down by the number of completely visible list entries.
//                            nPage however also includes the partly cropped list
//                            item at the bottom of the control */
//
//                         GetClientRect(hwnd,
//                                       &rcClient);
//
//                         ScrollLines = max!(1,
//                                           (rcClient.bottom - rcClient.top) / infoPtr->ItemHeight);
//
//                         if (ScrollInfo.nPos + ScrollLines <= ScrollInfo.nMax)
//                         {
//                             ScrollInfo.nPos += ScrollLines;
//                         }
//                         else
//                         {
//                             ScrollInfo.nPos = ScrollInfo.nMax;
//                         }
//                         break;
//                     }
//
//                     case SB_PAGEUP:
//                     {
//                         RECT rcClient;
//                         INT ScrollLines;
//
//                         /* don't use ScrollInfo.nPage because we should only scroll
//                            down by the number of completely visible list entries.
//                            nPage however also includes the partly cropped list
//                            item at the bottom of the control */
//
//                         GetClientRect(hwnd,
//                                       &rcClient);
//
//                         ScrollLines = max!(1,
//                                           (rcClient.bottom - rcClient.top) / infoPtr->ItemHeight);
//
//                         if (ScrollInfo.nPos >= ScrollLines)
//                         {
//                             ScrollInfo.nPos -= ScrollLines;
//                         }
//                         else
//                         {
//                             ScrollInfo.nPos = 0;
//                         }
//                         break;
//                     }
//
//                     case SB_THUMBPOSITION:
//                     case SB_THUMBTRACK:
//                     {
//                         ScrollInfo.nPos = HIWORD!(wParam);
//                         break;
//                     }
//
//                     case SB_TOP:
//                         ScrollInfo.nPos = 0;
//                         break;
//                 }
//
//                 if (OldPos != ScrollInfo.nPos)
//                 {
//                     ScrollInfo.fMask = SIF_POS;
//
//                     ScrollInfo.nPos = SetScrollInfo(hwnd,
//                                                     SB_VERT,
//                                                     &ScrollInfo,
//                                                     TRUE);
//
//                     if (OldPos != ScrollInfo.nPos)
//                     {
//                         ScrollWindowEx(hwnd,
//                                        0,
//                                        (OldPos - ScrollInfo.nPos) * infoPtr->ItemHeight,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        SW_INVALIDATE | SW_SCROLLCHILDREN);
//
//                         RedrawWindow(hwnd,
//                                      NULL,
//                                      NULL,
//                                      RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
//                     }
//                 }
//             }
//             break;
//         }
//
//         case CLM_ADDITEM:
//         {
//             INT Index = -1;
//             PCHECKITEM Item = AddCheckItem(infoPtr,
//                                            (LPWSTR)lParam,
//                                            CIS_NONE,
//                                            (ACCESS_MASK)wParam,
//                                            &Index);
//             if (Item != NULL)
//             {
//                 UpdateControl(infoPtr);
//                 Ret = (LRESULT)Index;
//             }
//             else
//             {
//                 Ret = (LRESULT)-1;
//             }
//             break;
//         }
//
//         case CLM_DELITEM:
//         {
//             PCHECKITEM Item = FindCheckItemByIndex(infoPtr,
//                                                    wParam);
//             if (Item != NULL)
//             {
//                 Ret = DeleteCheckItem(infoPtr,
//                                       Item);
//                 if (Ret)
//                 {
//                     UpdateControl(infoPtr);
//                 }
//             }
//             else
//             {
//                 Ret = FALSE;
//             }
//             break;
//         }
//
//         case CLM_SETITEMSTATE:
//         {
//             PCHECKITEM Item = FindCheckItemByIndex(infoPtr,
//                                                    wParam);
//             if (Item != NULL)
//             {
//                 DWORD OldState = Item->State;
//                 Item->State = (DWORD)lParam & CIS_MASK;
//
//                 if (Item->State != OldState)
//                 {
//                     /* revert the focus if the currently focused item is about
//                        to be disabled */
//                     if (Item == infoPtr->FocusedCheckItem &&
//                         (Item->State & CIS_DISABLED))
//                     {
//                         if (infoPtr->FocusedCheckItemBox == CLB_DENY)
//                         {
//                             if (Item->State & CIS_DENYDISABLED)
//                             {
//                                 infoPtr->FocusedCheckItem = NULL;
//                             }
//                         }
//                         else
//                         {
//                             if (Item->State & CIS_ALLOWDISABLED)
//                             {
//                                 infoPtr->FocusedCheckItem = NULL;
//                             }
//                         }
//                     }
//
//                     UpdateControl(infoPtr);
//                 }
//                 Ret = TRUE;
//             }
//             break;
//         }
//
//         case CLM_GETITEMCOUNT:
//         {
//             Ret = infoPtr->CheckItemCount;
//             break;
//         }
//
//         case CLM_CLEAR:
//         {
//             ClearCheckItems(infoPtr);
//             UpdateControl(infoPtr);
//             break;
//         }
//
//         case CLM_SETCHECKBOXCOLUMN:
//         {
//             infoPtr->CheckBoxLeft[wParam != CLB_DENY] = (INT)lParam;
//             UpdateControl(infoPtr);
//             Ret = 1;
//             break;
//         }
//
//         case CLM_GETCHECKBOXCOLUMN:
//         {
//             Ret = (LRESULT)infoPtr->CheckBoxLeft[wParam != CLB_DENY];
//             break;
//         }
//
//         case CLM_CLEARCHECKBOXES:
//         {
//             Ret = (LRESULT)ClearCheckBoxes(infoPtr);
//             if (Ret)
//             {
//                 UpdateControl(infoPtr);
//             }
//             break;
//         }
//
//         case CLM_ENABLEQUICKSEARCH:
//         {
//             if (wParam == 0)
//             {
//                 EscapeQuickSearch(infoPtr);
//             }
//             infoPtr->QuickSearchEnabled = (wParam != 0);
//             break;
//         }
//
//         case CLM_SETQUICKSEARCH_TIMEOUT_RESET:
//         {
//             infoPtr->QuickSearchResetDelay = (UINT)wParam;
//             break;
//         }
//
//         case CLM_SETQUICKSEARCH_TIMEOUT_SETFOCUS:
//         {
//             infoPtr->QuickSearchSetFocusDelay = (UINT)wParam;
//             break;
//         }
//
//         case CLM_FINDITEMBYACCESSMASK:
//         {
//             Ret = (LRESULT)FindCheckItemIndexByAccessMask(infoPtr,
//                                                           (ACCESS_MASK)wParam);
//             break;
//         }
//
//         case WM_SETFONT:
//         {
//             Ret = (LRESULT)RetChangeControlFont(infoPtr,
//                                                 (HFONT)wParam,
//                                                 (BOOL)LOWORD!(lParam));
//             break;
//         }
//
//         case WM_GETFONT:
//         {
//             Ret = (LRESULT)infoPtr->hFont;
//             break;
//         }
//
//         case WM_STYLECHANGED:
//         {
//             if (wParam == (WPARAM)GWL_STYLE)
//             {
//                 UpdateControl(infoPtr);
//             }
//             break;
//         }
//
//         case WM_ENABLE:
//         {
//             EscapeQuickSearch(infoPtr);
//
//             UpdateControl(infoPtr);
//             break;
//         }
//
//         case WM_MOUSEWHEEL:
//         {
//             SHORT ScrollDelta;
//             UINT ScrollLines = 3;
//
//             SystemParametersInfo(SPI_GETWHEELSCROLLLINES,
//                                  0,
//                                  &ScrollLines,
//                                  0);
//             ScrollDelta = 0 - (SHORT)HIWORD!(wParam);
//
//             if (ScrollLines != 0 &&
//                 abs(ScrollDelta) >= WHEEL_DELTA)
//             {
//                 SCROLLINFO ScrollInfo;
//
//                 ScrollInfo.cbSize = sizeof(ScrollInfo);
//                 ScrollInfo.fMask = SIF_RANGE | SIF_POS;
//
//                 if (GetScrollInfo(hwnd,
//                                   SB_VERT,
//                                   &ScrollInfo))
//                 {
//                     INT OldPos = ScrollInfo.nPos;
//
//                     ScrollInfo.nPos += (ScrollDelta / WHEEL_DELTA) * ScrollLines;
//                     if (ScrollInfo.nPos < 0)
//                         ScrollInfo.nPos = 0;
//                     else if (ScrollInfo.nPos > ScrollInfo.nMax)
//                         ScrollInfo.nPos = ScrollInfo.nMax;
//
//                     if (OldPos != ScrollInfo.nPos)
//                     {
//                         ScrollInfo.fMask = SIF_POS;
//
//                         ScrollInfo.nPos = SetScrollInfo(hwnd,
//                                                         SB_VERT,
//                                                         &ScrollInfo,
//                                                         TRUE);
//
//                         if (OldPos != ScrollInfo.nPos)
//                         {
//                             ScrollWindowEx(hwnd,
//                                            0,
//                                            (OldPos - ScrollInfo.nPos) * infoPtr->ItemHeight,
//                                            NULL,
//                                            NULL,
//                                            NULL,
//                                            NULL,
//                                            SW_INVALIDATE | SW_SCROLLCHILDREN);
//
//                             RedrawWindow(hwnd,
//                                          NULL,
//                                          NULL,
//                                          RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
//                         }
//                     }
//                 }
//             }
//             break;
//         }
//
//         case WM_SETFOCUS:
//         {
//             infoPtr->HasFocus = TRUE;
//
//             if (infoPtr->FocusedCheckItem == NULL)
//             {
//                 BOOL Shift = GetKeyState(VK_SHIFT) & 0x8000;
//                 infoPtr->FocusedCheckItem = FindEnabledCheckBox(infoPtr,
//                                                                 Shift,
//                                                                 &infoPtr->FocusedCheckItemBox);
//             }
//             if (infoPtr->FocusedCheckItem != NULL)
//             {
//                 MakeCheckItemVisible(infoPtr,
//                                      infoPtr->FocusedCheckItem);
//
//                 UpdateCheckItem(infoPtr,
//                                 infoPtr->FocusedCheckItem);
//             }
//             break;
//         }
//
//         case WM_KILLFOCUS:
//         {
//             EscapeQuickSearch(infoPtr);
//
//             infoPtr->HasFocus = FALSE;
//             if (infoPtr->FocusedCheckItem != NULL)
//             {
//                 infoPtr->FocusedPushed = FALSE;
//
//                 UpdateCheckItem(infoPtr,
//                                 infoPtr->FocusedCheckItem);
//             }
//             break;
//         }
//
//         case WM_LBUTTONDBLCLK:
//         case WM_LBUTTONDOWN:
//         case WM_MBUTTONDOWN:
//         case WM_RBUTTONDOWN:
//         {
//             if (IsWindowEnabled(hwnd))
//             {
//                 PCHECKITEM NewFocus;
//                 UINT NewFocusBox = 0;
//                 BOOL InCheckBox;
//                 POINT pt;
//                 BOOL ChangeFocus, Capture = FALSE;
//
//                 pt.x = (LONG)LOWORD!(lParam);
//                 pt.y = (LONG)HIWORD!(lParam);
//
//                 NewFocus = PtToCheckItemBox(infoPtr,
//                                             &pt,
//                                             &NewFocusBox,
//                                             &InCheckBox);
//                 if (NewFocus != NULL)
//                 {
//                     if (NewFocus->State & ((NewFocusBox != CLB_DENY) ? CIS_ALLOWDISABLED : CIS_DENYDISABLED))
//                     {
//                         /* the user clicked on a disabled checkbox, try to set
//                            the focus to the other one or not change it at all */
//
//                         InCheckBox = FALSE;
//
//                         ChangeFocus = ((NewFocus->State & CIS_DISABLED) != CIS_DISABLED);
//                         if (ChangeFocus)
//                         {
//                             NewFocusBox = ((NewFocusBox != CLB_DENY) ? CLB_DENY : CLB_ALLOW);
//                         }
//                     }
//                     else
//                     {
//                         ChangeFocus = TRUE;
//                     }
//
//                     if (InCheckBox && ChangeFocus && GetCapture() == NULL &&
//                         (uMsg == WM_LBUTTONDOWN || uMsg == WM_LBUTTONDBLCLK))
//                     {
//                         infoPtr->FocusedPushed = TRUE;
//                         Capture = TRUE;
//                     }
//                 }
//                 else
//                 {
//                     ChangeFocus = TRUE;
//                 }
//
//                 if (ChangeFocus)
//                 {
//                     if (infoPtr->QuickSearchEnabled && infoPtr->QuickSearchHitItem != NULL &&
//                         infoPtr->QuickSearchHitItem != NewFocus)
//                     {
//                         EscapeQuickSearch(infoPtr);
//                     }
//
//                     ChangeCheckItemFocus(infoPtr,
//                                          NewFocus,
//                                          NewFocusBox);
//                 }
//
//                 if (!infoPtr->HasFocus)
//                 {
//                     SetFocus(hwnd);
//                 }
//
//                 if (Capture)
//                 {
//                     SetCapture(hwnd);
//                 }
//             }
//             break;
//         }
//
//         case WM_LBUTTONUP:
//         {
//             if (GetCapture() == hwnd)
//             {
//                 if (infoPtr->FocusedCheckItem != NULL && infoPtr->FocusedPushed)
//                 {
//                     PCHECKITEM PtItem;
//                     UINT PtItemBox;
//                     BOOL InCheckBox;
//                     POINT pt;
//
//                     pt.x = (LONG)LOWORD!(lParam);
//                     pt.y = (LONG)HIWORD!(lParam);
//
//                     infoPtr->FocusedPushed = FALSE;
//
//                     PtItem = PtToCheckItemBox(infoPtr,
//                                               &pt,
//                                               &PtItemBox,
//                                               &InCheckBox);
//
//                     if (PtItem == infoPtr->FocusedCheckItem && InCheckBox &&
//                         PtItemBox == infoPtr->FocusedCheckItemBox)
//                     {
//                         UINT OtherBox = ((PtItemBox == CLB_ALLOW) ? CLB_DENY : CLB_ALLOW);
//                         DWORD OtherStateMask = ((OtherBox == CLB_ALLOW) ?
//                                                 (CIS_ALLOW | CIS_ALLOWDISABLED) :
//                                                 (CIS_DENY | CIS_DENYDISABLED));
//                         DWORD OtherStateOld = PtItem->State & OtherStateMask;
//                         if (ChangeCheckBox(infoPtr,
//                                            PtItem,
//                                            PtItemBox) &&
//                             ((PtItem->State & OtherStateMask) != OtherStateOld))
//                         {
//                             UpdateCheckItemBox(infoPtr,
//                                                infoPtr->FocusedCheckItem,
//                                                OtherBox);
//                         }
//                     }
//
//                     UpdateCheckItemBox(infoPtr,
//                                        infoPtr->FocusedCheckItem,
//                                        infoPtr->FocusedCheckItemBox);
//                 }
//
//                 ReleaseCapture();
//             }
//             break;
//         }
//
//         case WM_KEYDOWN:
//         {
//             switch (wParam)
//             {
//                 case VK_SPACE:
//                 {
//                     if (GetCapture() == NULL &&
//                         !QuickSearchFindHit(infoPtr,
//                                             ' '))
//                     {
//                         if (infoPtr->FocusedCheckItem != NULL &&
//                             (infoPtr->QuickSearchHitItem == NULL ||
//                              infoPtr->QuickSearchHitItem == infoPtr->FocusedCheckItem))
//                         {
//                             UINT OldPushed = infoPtr->FocusedPushed;
//                             infoPtr->FocusedPushed = TRUE;
//
//                             if (infoPtr->FocusedPushed != OldPushed)
//                             {
//                                 MakeCheckItemVisible(infoPtr,
//                                                      infoPtr->FocusedCheckItem);
//
//                                 UpdateCheckItemBox(infoPtr,
//                                                    infoPtr->FocusedCheckItem,
//                                                    infoPtr->FocusedCheckItemBox);
//                             }
//                         }
//                     }
//                     break;
//                 }
//
//                 case VK_RETURN:
//                 {
//                     if (GetCapture() == NULL &&
//                         !QuickSearchFindHit(infoPtr,
//                                             '\n'))
//                     {
//                         if (infoPtr->FocusedCheckItem != NULL &&
//                             infoPtr->QuickSearchHitItem == NULL)
//                         {
//                             UINT OtherBox;
//                             DWORD OtherStateMask;
//                             DWORD OtherStateOld;
//
//                             MakeCheckItemVisible(infoPtr,
//                                                  infoPtr->FocusedCheckItem);
//
//                             OtherBox = ((infoPtr->FocusedCheckItemBox == CLB_ALLOW) ? CLB_DENY : CLB_ALLOW);
//                             OtherStateMask = ((OtherBox == CLB_ALLOW) ?
//                                               (CIS_ALLOW | CIS_ALLOWDISABLED) :
//                                               (CIS_DENY | CIS_DENYDISABLED));
//                             OtherStateOld = infoPtr->FocusedCheckItem->State & OtherStateMask;
//                             if (ChangeCheckBox(infoPtr,
//                                                infoPtr->FocusedCheckItem,
//                                                infoPtr->FocusedCheckItemBox))
//                             {
//                                 UpdateCheckItemBox(infoPtr,
//                                                    infoPtr->FocusedCheckItem,
//                                                    infoPtr->FocusedCheckItemBox);
//                                 if ((infoPtr->FocusedCheckItem->State & OtherStateMask) != OtherStateOld)
//                                 {
//                                     UpdateCheckItemBox(infoPtr,
//                                                        infoPtr->FocusedCheckItem,
//                                                        OtherBox);
//                                 }
//                             }
//                         }
//                     }
//                     break;
//                 }
//
//                 case VK_TAB:
//                 {
//                     if (GetCapture() == NULL)
//                     {
//                         PCHECKITEM NewFocus;
//                         UINT NewFocusBox = 0;
//                         BOOL Shift = GetKeyState(VK_SHIFT) & 0x8000;
//
//                         EscapeQuickSearch(infoPtr);
//
//                         NewFocus = FindEnabledCheckBox(infoPtr,
//                                                        Shift,
//                                                        &NewFocusBox);
//
//                         /* update the UI status */
//                         SendMessage(GetAncestor(hwnd,
//                                                 GA_PARENT),
//                                     WM_CHANGEUISTATE,
//                                     MAKEWPARAM!(UIS_INITIALIZE,
//                                                0),
//                                     0);
//
//                         ChangeCheckItemFocus(infoPtr,
//                                              NewFocus,
//                                              NewFocusBox);
//                     }
//                     break;
//                 }
//
//                 default:
//                 {
//                     goto HandleDefaultMessage;
//                 }
//             }
//             break;
//         }
//
//         case WM_KEYUP:
//         {
//             if (wParam == VK_SPACE && IsWindowEnabled(hwnd) &&
//                 infoPtr->FocusedCheckItem != NULL &&
//                 infoPtr->FocusedPushed)
//             {
//                 UINT OtherBox = ((infoPtr->FocusedCheckItemBox == CLB_ALLOW) ? CLB_DENY : CLB_ALLOW);
//                 DWORD OtherStateMask = ((OtherBox == CLB_ALLOW) ?
//                                         (CIS_ALLOW | CIS_ALLOWDISABLED) :
//                                         (CIS_DENY | CIS_DENYDISABLED));
//                 DWORD OtherStateOld = infoPtr->FocusedCheckItem->State & OtherStateMask;
//
//                 infoPtr->FocusedPushed = FALSE;
//
//                 if (ChangeCheckBox(infoPtr,
//                                    infoPtr->FocusedCheckItem,
//                                    infoPtr->FocusedCheckItemBox))
//                 {
//                     UpdateCheckItemBox(infoPtr,
//                                        infoPtr->FocusedCheckItem,
//                                        infoPtr->FocusedCheckItemBox);
//
//                     if ((infoPtr->FocusedCheckItem->State & OtherStateMask) != OtherStateOld)
//                     {
//                         UpdateCheckItemBox(infoPtr,
//                                            infoPtr->FocusedCheckItem,
//                                            OtherBox);
//                     }
//                 }
//             }
//             break;
//         }
//
//         case WM_GETDLGCODE:
//         {
//             INT virtKey;
//
//             Ret = 0;
//             virtKey = (lParam != 0 ? (INT)((LPMSG)lParam)->wParam : 0);
//             switch (virtKey)
//             {
//                 case VK_RETURN:
//                 {
//                     if (infoPtr->QuickSearchEnabled && infoPtr->QuickSearchHitItem != NULL)
//                     {
//                         Ret |= DLGC_WANTCHARS | DLGC_WANTMESSAGE;
//                     }
//                     else
//                     {
//                         Ret |= DLGC_WANTMESSAGE;
//                     }
//                     break;
//                 }
//
//                 case VK_TAB:
//                 {
//                     UINT CheckBox;
//                     BOOL EnabledBox;
//                     BOOL Shift = GetKeyState(VK_SHIFT) & 0x8000;
//
//                     EnabledBox = FindEnabledCheckBox(infoPtr,
//                                                      Shift,
//                                                      &CheckBox) != NULL;
//                     Ret |= (EnabledBox ? DLGC_WANTTAB : DLGC_WANTCHARS);
//                     break;
//                 }
//
//                 default:
//                 {
//                     if (infoPtr->QuickSearchEnabled)
//                     {
//                         Ret |= DLGC_WANTCHARS;
//                     }
//                     break;
//                 }
//             }
//             break;
//         }
//
//         case WM_CHAR:
//         {
//             QuickSearchFindHit(infoPtr,
//                                (WCHAR)wParam);
//             break;
//         }
//
//         case WM_SYSCOLORCHANGE:
//         {
//             infoPtr->TextColor[0] = GetSysColor(COLOR_GRAYTEXT);
//             infoPtr->TextColor[1] = GetSysColor(COLOR_WINDOWTEXT);
//             break;
//         }
//
// #if SUPPORT_UXTHEME
//         case WM_MOUSELEAVE:
//         {
//             if (infoPtr->HoveredCheckItem != NULL)
//             {
//                 /* reset and repaint the hovered check item box */
//                 ChangeCheckItemHotTrack(infoPtr,
//                                         NULL,
//                                         0);
//             }
//             break;
//         }
//
//         case WM_THEMECHANGED:
//         {
//             if (infoPtr->ThemeHandle != NULL)
//             {
//                 CloseThemeData(infoPtr->ThemeHandle);
//                 infoPtr->ThemeHandle = NULL;
//             }
//             if (IsAppThemed())
//             {
//                 infoPtr->ThemeHandle = OpenThemeData(infoPtr->hSelf,
//                                                      L"BUTTON");
//             }
//             break;
//         }
// #endif
//
//         case WM_SETTINGCHANGE:
//         {
//             DWORD OldCaretWidth = infoPtr->CaretWidth;
//
// #if SUPPORT_UXTHEME
//             /* update the hover time */
//             if (!SystemParametersInfo(SPI_GETMOUSEHOVERTIME,
//                                       0,
//                                       &infoPtr->HoverTime,
//                                       0))
//             {
//                 infoPtr->HoverTime = HOVER_DEFAULT;
//             }
// #endif
//
//             /* update the caret */
//             if (!SystemParametersInfo(SPI_GETCARETWIDTH,
//                                       0,
//                                       &infoPtr->CaretWidth,
//                                       0))
//             {
//                 infoPtr->CaretWidth = 2;
//             }
//             if (OldCaretWidth != infoPtr->CaretWidth && infoPtr->ShowingCaret)
//             {
//                 DestroyCaret();
//                 CreateCaret(hwnd,
//                             NULL,
//                             infoPtr->CaretWidth,
//                             infoPtr->ItemHeight - (2 * CI_TEXT_MARGIN_HEIGHT));
//             }
//             break;
//         }
//
//         case WM_SIZE:
//         {
//             UpdateControl(infoPtr);
//             break;
//         }
//
//         case WM_UPDATEUISTATE:
//         {
//             DWORD OldUIState = infoPtr->UIState;
//
//             switch (LOWORD!(wParam))
//             {
//                 case UIS_SET:
//                     infoPtr->UIState |= HIWORD!(wParam);
//                     break;
//
//                 case UIS_CLEAR:
//                     infoPtr->UIState &= ~(HIWORD!(wParam));
//                     break;
//             }
//
//             if (OldUIState != infoPtr->UIState)
//             {
//                 if (infoPtr->FocusedCheckItem != NULL)
//                 {
//                     UpdateCheckItemBox(infoPtr,
//                                        infoPtr->FocusedCheckItem,
//                                        infoPtr->FocusedCheckItemBox);
//                 }
//             }
//             break;
//         }
//
//         case WM_TIMER:
//         {
//             switch (wParam)
//             {
//                 case TIMER_ID_SETHITFOCUS:
//                 {
//                     /* kill the timer */
//                     KillTimer(hwnd,
//                               wParam);
//
//                     if (infoPtr->QuickSearchEnabled && infoPtr->QuickSearchHitItem != NULL)
//                     {
//                         /* change the focus to the hit item, this item has to have
//                            at least one enabled checkbox! */
//                         ChangeCheckItemFocus(infoPtr,
//                                              infoPtr->QuickSearchHitItem,
//                                              ((!(infoPtr->QuickSearchHitItem->State & CIS_ALLOWDISABLED)) ? CLB_ALLOW : CLB_DENY));
//
//                         /* start the timer to reset quicksearch */
//                         if (infoPtr->QuickSearchResetDelay != 0)
//                         {
//                             SetTimer(hwnd,
//                                      TIMER_ID_RESETQUICKSEARCH,
//                                      infoPtr->QuickSearchResetDelay,
//                                      NULL);
//                         }
//                     }
//                     break;
//                 }
//                 case TIMER_ID_RESETQUICKSEARCH:
//                 {
//                     /* kill the timer */
//                     KillTimer(hwnd,
//                               wParam);
//
//                     /* escape quick search */
//                     EscapeQuickSearch(infoPtr);
//                     break;
//                 }
//             }
//             break;
//         }
//
//         case WM_CREATE:
//         {
//             infoPtr = HeapAlloc(GetProcessHeap(),
//                                 0,
//                                 sizeof(CHECKLISTWND));
//             if (infoPtr != NULL)
//             {
//                 RECT rcClient;
//
//                 infoPtr->hSelf = hwnd;
//                 infoPtr->hNotify = ((LPCREATESTRUCTW)lParam)->hwndParent;
//
//                 SetWindowLongPtr(hwnd,
//                                  0,
//                                  (DWORD_PTR)infoPtr);
//
//                 infoPtr->CheckItemListHead = NULL;
//                 infoPtr->CheckItemCount = 0;
//
//                 if (!SystemParametersInfo(SPI_GETCARETWIDTH,
//                                           0,
//                                           &infoPtr->CaretWidth,
//                                           0))
//                 {
//                     infoPtr->CaretWidth = 2;
//                 }
//                 infoPtr->ItemHeight = 10;
//                 infoPtr->ShowingCaret = FALSE;
//
//                 infoPtr->HasFocus = FALSE;
//                 infoPtr->FocusedCheckItem = NULL;
//                 infoPtr->FocusedCheckItemBox = 0;
//                 infoPtr->FocusedPushed = FALSE;
//
//                 infoPtr->TextColor[0] = GetSysColor(COLOR_GRAYTEXT);
//                 infoPtr->TextColor[1] = GetSysColor(COLOR_WINDOWTEXT);
//
//                 GetClientRect(hwnd,
//                               &rcClient);
//
//                 infoPtr->CheckBoxLeft[0] = rcClient.right - 30;
//                 infoPtr->CheckBoxLeft[1] = rcClient.right - 15;
//
//                 infoPtr->QuickSearchEnabled = FALSE;
//                 infoPtr->QuickSearchText[0] = '\0';
//
//                 infoPtr->QuickSearchSetFocusDelay = DEFAULT_QUICKSEARCH_SETFOCUS_DELAY;
//                 infoPtr->QuickSearchResetDelay = DEFAULT_QUICKSEARCH_RESET_DELAY;
//
// #if SUPPORT_UXTHEME
//                 infoPtr->HoveredCheckItem = NULL;
//                 infoPtr->HoveredCheckItemBox = 0;
//                 if (!SystemParametersInfo(SPI_GETMOUSEHOVERTIME,
//                                           0,
//                                           &infoPtr->HoverTime,
//                                           0))
//                 {
//                     infoPtr->HoverTime = HOVER_DEFAULT;
//                 }
//
//                 if (IsAppThemed())
//                 {
//                     infoPtr->ThemeHandle = OpenThemeData(infoPtr->hSelf,
//                                                          L"BUTTON");
//                 }
//                 else
//                 {
//                     infoPtr->ThemeHandle = NULL;
//                 }
// #endif
//
//                 infoPtr->UIState = SendMessage(hwnd,
//                                                WM_QUERYUISTATE,
//                                                0,
//                                                0);
//             }
//             else
//             {
//                 Ret = -1;
//             }
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             if (infoPtr->ShowingCaret)
//             {
//                 DestroyCaret();
//             }
//
//             ClearCheckItems(infoPtr);
//
// #if SUPPORT_UXTHEME
//             if (infoPtr->ThemeHandle != NULL)
//             {
//                 CloseThemeData(infoPtr->ThemeHandle);
//             }
// #endif
//
//             HeapFree(GetProcessHeap(),
//                      0,
//                      infoPtr);
//             SetWindowLongPtr(hwnd,
//                              0,
//                              (DWORD_PTR)NULL);
//             break;
//         }
//
//         default:
//         {
// HandleDefaultMessage:
//             Ret = DefWindowProc(hwnd,
//                                 uMsg,
//                                 wParam,
//                                 lParam);
//             break;
//         }
//     }
//
//     return Ret;
// }
//
// BOOL
// RegisterCheckListControl( HINSTANCE hInstance)
// {
//     WNDCLASS wc;
//
//     wc.style = CS_DBLCLKS;
//     wc.lpfnWndProc = CheckListWndProc;
//     wc.cbClsExtra = 0;
//     wc.cbWndExtra = sizeof(PCHECKLISTWND);
//     wc.hInstance = hInstance;
//     wc.hIcon = NULL;
//     wc.hCursor = LoadCursor(NULL,
//                             (LPWSTR)IDC_ARROW);
//     wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
//     wc.lpszMenuName = NULL;
//     wc.lpszClassName = szCheckListWndClass;
//
//     return RegisterClass(&wc) != 0;
// }
//
// VOID
// UnregisterCheckListControl(HINSTANCE hInstance)
// {
//     UnregisterClass(szCheckListWndClass,
//                     hInstance);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
