use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::dpa::DPA_Create;
use crate::comctl32::dpa::DPA_DeletePtr;
use crate::comctl32::dpa::DPA_Destroy;
use crate::comctl32::dpa::DPA_GetPtr;
use crate::comctl32::dpa::DPA_InsertPtr;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::imagelist::ImageList_Draw;
use crate::comctl32::imagelist::ImageList_GetIconSize;
use crate::comctl32::imagelist::ImageList_Remove;
use crate::comctl32::string::Str_GetPtrW;
use crate::comctl32::comctl32undoc::Str_GetPtrWtoA;
use crate::comctl32::comctl32undoc::Str_SetPtrAtoW;
use crate::comctl32::string::Str_SetPtrW;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_tab);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

pub const WC_TABCONTROLW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"S\0y\0s\0T\0a\0b\0C\0o\0n\0t\0r\0o\0l\x003\x002\0\0\0",
    )
};

static mut themeClass: [WCHAR; 4] = [
    'T' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[repr(C)]
pub struct TAB_ITEM {
    pub dwState: DWORD,
    pub pszText: LPWSTR,
    pub iImage: INT,
    pub rect: RECT,
    pub extra: [BYTE; 1],
}
pub type PTAB_ITEM = *mut TAB_ITEM;

#[repr(C)]
pub struct TAB_INFO {
    pub hwnd: HWND,
    pub hwndNotify: HWND,
    pub uNumItem: UINT,
    pub uNumRows: UINT,
    pub tabHeight: INT,
    pub tabWidth: INT,
    pub tabMinWidth: INT,
    pub uHItemPadding: USHORT,
    pub uVItemPadding: USHORT,
    pub uHItemPadding_s: USHORT,
    pub uVItemPadding_s: USHORT,
    pub hFont: HFONT,
    pub hcurArrow: HCURSOR,
    pub himl: HIMAGELIST,
    pub hwndToolTip: HWND,
    pub leftmostVisible: INT,
    pub iSelected: INT,
    pub iHotTracked: INT,
    pub uFocus: INT,
    pub DoRedraw: BOOL,
    pub needsScrolling: BOOL,
    pub fHeightSet: BOOL,
    pub bUnicode: BOOL,
    pub hwndUpDown: HWND,
    pub cbInfo: INT,
    pub exStyle: DWORD,
    pub dwStyle: DWORD,
    pub items: HDPA,
}
pub type PTAB_INFO = *mut TAB_INFO;

pub const SELECTED_TAB_OFFSET: i32 = 2;
pub const ROUND_CORNER_SIZE: i32 = 2;
pub const DISPLAY_AREA_PADDINGX: i32 = 2;
pub const DISPLAY_AREA_PADDINGY: i32 = 2;
pub const CONTROL_BORDER_SIZEX: i32 = 2;
pub const CONTROL_BORDER_SIZEY: i32 = 2;
pub const BUTTON_SPACINGX: i32 = 3;
pub const BUTTON_SPACINGY: i32 = 3;
pub const FLAT_BTN_SPACINGX: i32 = 8;
pub const DEFAULT_MIN_TAB_WIDTH: i32 = 54;
pub const DEFAULT_PADDING_X: i32 = 6;
pub const EXTRA_ICON_PADDING: i32 = 3;
pub const TAB_HOTTRACK_TIMER: i32 = 1;
pub const TAB_HOTTRACK_TIMER_INTERVAL: i32 = 100;

const __WINE_DBCL_TRACE: __wine_debug_class = 3; // Wine enum value for TRACE class

unsafe extern "C" fn TAB_GetItem(infoPtr: *const TAB_INFO, i: INT) -> *mut TAB_ITEM {
    assert!((i >= 0 && (i as UINT) < (*infoPtr).uNumItem) as INT);
    return DPA_GetPtr((*infoPtr).items, i) as *mut TAB_ITEM;
}

unsafe extern "C" fn TAB_SendSimpleNotify(infoPtr: *const TAB_INFO, code: UINT) -> BOOL {
    let mut nmhdr: NMHDR = core::mem::zeroed();

    nmhdr.hwndFrom = (*infoPtr).hwnd;
    nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID) as UINT_PTR;
    nmhdr.code = code;

    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmhdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmhdr) as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn TAB_RelayEvent(hwndTip: HWND, hwndMsg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) {
    let mut msg: MSG = core::mem::zeroed();

    msg.hwnd = hwndMsg;
    msg.message = uMsg;
    msg.wParam = wParam;
    msg.lParam = lParam;
    msg.time = GetMessageTime() as DWORD;
    msg.pt.x = (GetMessagePos() as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ::core::ffi::c_short as LONG;
    msg.pt.y = (GetMessagePos() as DWORD_PTR >> 16 as ::core::ffi::c_int & 0xffff as DWORD_PTR) as WORD as ::core::ffi::c_short as LONG;

    SendMessageW(
        hwndTip,
        TTM_RELAYEVENT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(msg) as LPARAM,
    );
}

unsafe extern "C" fn TAB_DumpItemExternalT(pti: *const TCITEMW, iItem: UINT, isW: BOOL) {
    if (__wine_dbch_tab.flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0
        && (__wine_dbg_get_channel_flags(core::ptr::addr_of_mut!(__wine_dbch_tab)) as i32)
            & ((1 as i32) << (__WINE_DBCL_TRACE as i32))
            != 0
    {
        if (*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tab.c\0"
                    .as_ptr() as *const i8,
                b"TAB_DumpItemExternalT\0".as_ptr() as *const i8,
                207 as i32,
                b"external tab %d, mask=0x%08x, dwState=0x%08x, dwStateMask=0x%08x, cchTextMax=0x%08x\n\0"
                    .as_ptr() as *const i8,
                iItem,
                (*pti).mask,
                (*pti).dwState,
                (*pti).dwStateMask,
                (*pti).cchTextMax,
            );
        }
        if (*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/tab.c\0"
                    .as_ptr() as *const i8,
                b"TAB_DumpItemExternalT\0".as_ptr() as *const i8,
                209 as i32,
                b"external tab %d,   iImage=%d, lParam=0x%08lx, pszTextW=%s\n\0".as_ptr() as *const i8,
                iItem,
                (*pti).iImage,
                (*pti).lParam,
                if isW != 0 {
                    debugstr_w((*pti).pszText as *const WCHAR)
                } else {
                    debugstr_a((*pti).pszText as LPSTR as *const i8)
                },
            );
        }
    }
}

unsafe extern "C" fn TAB_DumpItemInternal(infoPtr: *const TAB_INFO, iItem: UINT) {
    if TRACE_ON!(tab) {
        let ti: *mut TAB_ITEM = TAB_GetItem(infoPtr, iItem as INT);
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TAB_DumpItemInternal\0" as *const u8 as *const ::core::ffi::c_char,
                220 as ::core::ffi::c_int,
                b"tab %d, dwState=0x%08x, pszText=%s, iImage=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                iItem,
                (*ti).dwState,
                debugstr_w((*ti).pszText as *const WCHAR),
                (*ti).iImage,
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TAB_DumpItemInternal\0" as *const u8 as *const ::core::ffi::c_char,
                222 as ::core::ffi::c_int,
                b"tab %d, rect.left=%d, rect.top(row)=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                iItem,
                (*ti).rect.left,
                (*ti).rect.top,
            );
        }
    }
}

unsafe extern "C" fn TAB_GetCurSel(infoPtr: *const TAB_INFO) -> LRESULT {
    // TRACE("(%p)\n", infoPtr);
    ((*infoPtr).iSelected) as _
}

unsafe extern "C" fn TAB_GetCurFocus(infoPtr: *const TAB_INFO) -> LRESULT {
    // TRACE("(%p)\n", infoPtr);
    ((*infoPtr).uFocus) as _
}

unsafe extern "C" fn TAB_GetToolTips(infoPtr: *const TAB_INFO) -> LRESULT {
    return (*infoPtr).hwndToolTip as LRESULT;
}

unsafe extern "C" fn TAB_SetCurSel(infoPtr: *mut TAB_INFO, iItem: INT) -> LRESULT {
    let mut prevItem: INT = (*infoPtr).iSelected;

    if iItem >= (*infoPtr).uNumItem as INT {
        return -1 as LRESULT;
    }

    if prevItem != iItem {
        if prevItem != -1 {
            (*TAB_GetItem(infoPtr, prevItem)).dwState &= !TCIS_BUTTONPRESSED as DWORD;
        }

        if iItem >= 0 {
            (*TAB_GetItem(infoPtr, iItem)).dwState |= TCIS_BUTTONPRESSED as DWORD;
            (*infoPtr).iSelected = iItem;
            (*infoPtr).uFocus = iItem;
        } else {
            (*infoPtr).iSelected = -1 as INT;
            (*infoPtr).uFocus = -1 as INT;
        }

        TAB_EnsureSelectionVisible(infoPtr);
        TAB_InvalidateTabArea(infoPtr);
    }

    return prevItem as LRESULT;
}

unsafe extern "C" fn TAB_SetCurFocus(infoPtr: *mut TAB_INFO, iItem: INT) -> LRESULT {
    if iItem < 0 {
        (*infoPtr).uFocus = -1;
        if (*infoPtr).iSelected != -1 {
            (*TAB_GetItem(infoPtr, (*infoPtr).iSelected)).dwState &= !(TCIS_BUTTONPRESSED as DWORD);
            (*infoPtr).iSelected = -1;
            TAB_SendSimpleNotify(infoPtr, (TCN_SELCHANGE) as UINT);
            TAB_InvalidateTabArea(infoPtr);
        }
    } else if (iItem as UINT) < (*infoPtr).uNumItem {
        if (*infoPtr).dwStyle & TCS_BUTTONS as DWORD != 0 {
            /* set focus to new item, leave selection as is */
            if (*infoPtr).uFocus != iItem {
                let mut prev_focus: INT = (*infoPtr).uFocus;
                let mut r: RECT = core::mem::zeroed();

                (*infoPtr).uFocus = iItem;

                if prev_focus != (*infoPtr).iSelected {
                    if TAB_InternalGetItemRect(
                        infoPtr,
                        prev_focus,
                        core::ptr::addr_of_mut!(r),
                        core::ptr::null_mut::<RECT>(),
                    ) != 0
                    {
                        InvalidateRect(
                            (*infoPtr).hwnd,
                            core::ptr::addr_of_mut!(r) as LPCRECT,
                            FALSE,
                        );
                    }
                }

                if TAB_InternalGetItemRect(
                    infoPtr,
                    iItem,
                    core::ptr::addr_of_mut!(r),
                    core::ptr::null_mut::<RECT>(),
                ) != 0
                {
                    InvalidateRect(
                        (*infoPtr).hwnd,
                        core::ptr::addr_of_mut!(r) as LPCRECT,
                        FALSE,
                    );
                }

                TAB_SendSimpleNotify(infoPtr, (TCN_FOCUSCHANGE) as UINT);
            }
        } else {
            let mut oldItem: INT = (*infoPtr).iSelected;
            let mut oldFocus: INT = (*infoPtr).uFocus;
            if (*infoPtr).iSelected != iItem || oldFocus == -1 {
                (*infoPtr).uFocus = iItem;
                if oldFocus != -1 {
                    if TAB_SendSimpleNotify(infoPtr, (TCN_SELCHANGING) as UINT) == 0 {
                        if oldItem != -1 {
                            (*TAB_GetItem(infoPtr, oldItem)).dwState &= !(TCIS_BUTTONPRESSED as DWORD);
                        }
                        (*infoPtr).iSelected = iItem;
                        (*TAB_GetItem(infoPtr, iItem)).dwState |= TCIS_BUTTONPRESSED as DWORD;
                        TAB_SendSimpleNotify(infoPtr, (TCN_SELCHANGE) as UINT);
                    } else {
                        (*infoPtr).iSelected = iItem;
                    }
                    TAB_EnsureSelectionVisible(infoPtr);
                    TAB_InvalidateTabArea(infoPtr);
                }
            }
        }
    }
    return 0;
}

unsafe extern "C" fn TAB_SetToolTips(mut infoPtr: *mut TAB_INFO, mut hwndToolTip: HWND) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_SetToolTips\0" as *const u8 as *const ::core::ffi::c_char,
            348 as ::core::ffi::c_int,
            b"%p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            infoPtr,
            hwndToolTip,
        );
    }
    (*infoPtr).hwndToolTip = hwndToolTip;
    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_SetPadding(infoPtr: *mut TAB_INFO, lParam: LPARAM) -> LRESULT {
    (*infoPtr).uHItemPadding_s = LOWORD!(lParam);
    (*infoPtr).uVItemPadding_s = HIWORD!(lParam);

    return 0;
}

unsafe extern "C" fn TAB_InternalGetItemRect(
    infoPtr: *const TAB_INFO,
    itemIndex: INT,
    mut itemRect: *mut RECT,
    mut selectedRect: *mut RECT,
) -> BOOL {
    let mut tmpItemRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut clientRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if (*infoPtr).uNumItem <= 0 as UINT
        || itemIndex as UINT >= (*infoPtr).uNumItem
        || (!(((*infoPtr).dwStyle & (TCS_MULTILINE as DWORD)) != 0
            || ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0)
            && itemIndex < (*infoPtr).leftmostVisible)
    {
        if (((*__wine_dbch___default).flags as ::core::ffi::c_int)
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TAB_InternalGetItemRect\0" as *const u8 as *const ::core::ffi::c_char,
                386 as ::core::ffi::c_int,
                b"Not Visible\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        SetRect(
            itemRect as LPRECT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*infoPtr).tabHeight as ::core::ffi::c_int,
        );
        SetRectEmpty(selectedRect as LPRECT);
        return FALSE as BOOL;
    }

    /*
     * Avoid special cases in this procedure by assigning the "out"
     * parameters if the caller didn't supply them
     */
    if itemRect.is_null() {
        itemRect = core::ptr::addr_of_mut!(tmpItemRect);
    }

    /* Retrieve the unmodified item rect. */
    *itemRect = (*TAB_GetItem(infoPtr, itemIndex)).rect;

    /* calculate the times bottom and top based on the row */
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(clientRect));

    if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0
        && ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0
    {
        (*itemRect).right = (clientRect.right as INT
            - SELECTED_TAB_OFFSET as INT
            - (*itemRect).left as INT * (*infoPtr).tabHeight as INT
            - (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                (*itemRect).left as INT * BUTTON_SPACINGX as INT
            } else {
                0 as INT
            })) as LONG;
        (*itemRect).left = ((*itemRect).right as INT - (*infoPtr).tabHeight as INT) as LONG;
    } else if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
        (*itemRect).left = (clientRect.left as INT
            + SELECTED_TAB_OFFSET as INT
            + (*itemRect).left as INT * (*infoPtr).tabHeight as INT
            + (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                (*itemRect).left as INT * BUTTON_SPACINGX as INT
            } else {
                0 as INT
            })) as LONG;
        (*itemRect).right = ((*itemRect).left as INT + (*infoPtr).tabHeight as INT) as LONG;
    } else if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0 {
        (*itemRect).bottom = (clientRect.bottom as INT
            - (*itemRect).top as INT * (*infoPtr).tabHeight as INT
            - (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                (*itemRect).top as INT * BUTTON_SPACINGY as INT
            } else {
                SELECTED_TAB_OFFSET as INT
            })) as LONG;
        (*itemRect).top = ((*itemRect).bottom as INT - (*infoPtr).tabHeight as INT) as LONG;
    } else {
        /* not TCS_BOTTOM and not TCS_VERTICAL */
        (*itemRect).top = (clientRect.top as INT
            + (*itemRect).top as INT * (*infoPtr).tabHeight as INT
            + (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                (*itemRect).top as INT * BUTTON_SPACINGY as INT
            } else {
                SELECTED_TAB_OFFSET as INT
            })) as LONG;
        (*itemRect).bottom = ((*itemRect).top as INT + (*infoPtr).tabHeight as INT) as LONG;
    }

    /*
     * "scroll" it to make sure the item at the very left of the
     * tab control is the leftmost visible tab.
     */
    if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
        OffsetRect(
            itemRect as LPRECT,
            0 as INT,
            -((*TAB_GetItem(infoPtr, (*infoPtr).leftmostVisible)).rect.top as INT),
        );

        /*
         * Move the rectangle so the first item is slightly offset from
         * the bottom of the tab control.
         */
        OffsetRect(
            itemRect as LPRECT,
            0 as INT,
            SELECTED_TAB_OFFSET as INT,
        );
    } else {
        OffsetRect(
            itemRect as LPRECT,
            -((*TAB_GetItem(infoPtr, (*infoPtr).leftmostVisible)).rect.left as INT),
            0 as INT,
        );

        /*
         * Move the rectangle so the first item is slightly offset from
         * the left of the tab control.
         */
        OffsetRect(
            itemRect as LPRECT,
            SELECTED_TAB_OFFSET as INT,
            0 as INT,
        );
    }

    /* Now, calculate the position of the item as if it were selected. */
    if !selectedRect.is_null() {
        *selectedRect = *itemRect;

        /* The rectangle of a selected item is a bit wider. */
        if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
            InflateRect(
                selectedRect as LPRECT,
                0 as INT,
                SELECTED_TAB_OFFSET as INT,
            );
        } else {
            InflateRect(
                selectedRect as LPRECT,
                SELECTED_TAB_OFFSET as INT,
                0 as INT,
            );
        }

        /* If it also a bit higher. */
        if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0
            && ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0
        {
            (*selectedRect).left -= 2 as LONG; /* the border is thicker on the right */
            (*selectedRect).right += SELECTED_TAB_OFFSET as LONG;
        } else if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
            (*selectedRect).left -= SELECTED_TAB_OFFSET as LONG;
            (*selectedRect).right += 1 as LONG;
        } else if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0 {
            (*selectedRect).bottom += SELECTED_TAB_OFFSET as LONG;
        } else {
            /* not TCS_BOTTOM and not TCS_VERTICAL */
            (*selectedRect).top -= SELECTED_TAB_OFFSET as LONG;
            (*selectedRect).bottom -= 1 as LONG;
        }
    }

    if (((*__wine_dbch___default).flags as ::core::ffi::c_int)
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_InternalGetItemRect\0" as *const u8 as *const ::core::ffi::c_char,
            463 as ::core::ffi::c_int,
            b"item %d tab h=%d, rect=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            itemIndex,
            (*infoPtr).tabHeight,
            wine_dbgstr_rect(itemRect),
        );
    }

    /* Check for visibility */
    if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
        return (((*itemRect).top < clientRect.bottom) && ((*itemRect).bottom > clientRect.top)) as BOOL;
    } else {
        return (((*itemRect).left < clientRect.right) && ((*itemRect).right > clientRect.left)) as BOOL;
    }
}

unsafe extern "C" fn TAB_GetItemRect(
    infoPtr: *const TAB_INFO,
    item: INT,
    rect: *mut RECT,
) -> BOOL {
    // TRACE("(%p, %d, %p)\n", infoPtr, item, rect);
    TAB_InternalGetItemRect(infoPtr, item, rect, core::ptr::null_mut())
}

unsafe extern "C" fn TAB_KeyDown(infoPtr: *mut TAB_INFO, keyCode: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut newItem: INT = -1 as INT;
    let mut nm: NMTCKEYDOWN = core::mem::zeroed();

    /* TCN_KEYDOWN notification sent always */
    nm.hdr.hwndFrom = (*infoPtr).hwnd;
    nm.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID) as UINT_PTR;
    nm.hdr.code = TCN_KEYDOWN as UINT;
    nm.wVKey = keyCode as WORD;
    nm.flags = lParam as UINT;
    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nm.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nm) as LPARAM,
    );

    match keyCode as UINT {
        VK_LEFT => {
            newItem = (*infoPtr).uFocus as INT - 1 as INT;
        }
        VK_RIGHT => {
            newItem = (*infoPtr).uFocus as INT + 1 as INT;
        }
        _ => {}
    }

    /* If we changed to a valid item, change focused item */
    if newItem >= 0 as INT
        && (newItem as UINT) < (*infoPtr).uNumItem
        && ((*infoPtr).uFocus) as u32 != newItem as UINT
    {
        TAB_SetCurFocus(infoPtr, newItem);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_KillFocus(infoPtr: *mut TAB_INFO) {
    /* clear current focused item back to selected for TCS_BUTTONS */
    if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 && ((*infoPtr).uFocus != (*infoPtr).iSelected) {
        let mut r: RECT = core::mem::zeroed();

        if TAB_InternalGetItemRect(infoPtr, (*infoPtr).uFocus, core::ptr::addr_of_mut!(r), core::ptr::null_mut()) != 0 {
            InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(r), FALSE);
        }

        (*infoPtr).uFocus = (*infoPtr).iSelected;
    }
}

unsafe extern "C" fn TAB_FocusChanging(infoPtr: *const TAB_INFO) {
    let mut selectedRect: RECT = core::mem::zeroed();
    let mut isVisible: BOOL;

    /*
     * Get the rectangle for the item.
     */
    isVisible = TAB_InternalGetItemRect(
        infoPtr,
        (*infoPtr).uFocus,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(selectedRect),
    );

    /*
     * If the rectangle is not completely invisible, invalidate that
     * portion of the window.
     */
    if isVisible != 0 {
        InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(selectedRect), TRUE);
    }
}

unsafe extern "C" fn TAB_InternalHitTest(infoPtr: *const TAB_INFO, pt: POINT, flags: *mut UINT) -> INT {
    let mut rect: RECT = core::mem::zeroed();
    let mut iCount: INT;
    iCount = 0 as INT;
    while (iCount as UINT) < (*infoPtr).uNumItem {
        TAB_InternalGetItemRect(
            infoPtr,
            iCount,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null_mut(),
        );

        if PtInRect(core::ptr::addr_of_mut!(rect) as LPCRECT, pt) != 0 {
            *flags = TCHT_ONITEM as UINT;
            return iCount;
        }
        iCount += 1;
    }

    *flags = TCHT_NOWHERE as UINT;
    return -(1 as INT);
}

unsafe extern "C" fn TAB_HitTest(infoPtr: *const TAB_INFO, lptest: LPTCHITTESTINFO) -> LRESULT {
    return TAB_InternalHitTest(
        infoPtr,
        (*lptest).pt,
        core::ptr::addr_of_mut!((*lptest).flags),
    ) as LRESULT;
}

unsafe extern "C" fn TAB_NCHitTest(infoPtr: *const TAB_INFO, lParam: LPARAM) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut dummyflag: UINT = 0;
    pt.x = LOWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;
    pt.y = HIWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;
    ScreenToClient((*infoPtr).hwnd, core::ptr::addr_of_mut!(pt));

    if TAB_InternalHitTest(infoPtr, pt, core::ptr::addr_of_mut!(dummyflag)) == -1 {
        return HTTRANSPARENT as LRESULT;
    } else {
        return HTCLIENT as LRESULT;
    }
}

unsafe extern "C" fn TAB_LButtonDown(mut infoPtr: *mut TAB_INFO, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let mut pt: POINT = core::mem::zeroed();
    let mut newItem: INT = 0;
    let mut dummy: UINT = 0;

    if !(*infoPtr).hwndToolTip.is_null() {
        TAB_RelayEvent((*infoPtr).hwndToolTip, (*infoPtr).hwnd, WM_LBUTTONDOWN, wParam, lParam);
    }

    if ((*infoPtr).dwStyle & TCS_FOCUSNEVER) == 0 {
        SetFocus((*infoPtr).hwnd);
    }

    if !(*infoPtr).hwndToolTip.is_null() {
        TAB_RelayEvent((*infoPtr).hwndToolTip, (*infoPtr).hwnd, WM_LBUTTONDOWN, wParam, lParam);
    }

    pt.x = (LOWORD!(lParam as DWORD) as WORD) as ::core::ffi::c_short as LONG;
    pt.y = (HIWORD!(lParam as DWORD) as WORD) as ::core::ffi::c_short as LONG;

    newItem = TAB_InternalHitTest(infoPtr, pt, core::ptr::addr_of_mut!(dummy));

    if (newItem != -1) && ((*infoPtr).iSelected != newItem) {
        if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0
            && ((*infoPtr).dwStyle & TCS_MULTISELECT) != 0
            && (wParam & (MK_CONTROL) as WPARAM) != 0
        {
            let mut r: RECT = core::mem::zeroed();

            /* toggle multiselection */
            (*TAB_GetItem(infoPtr, newItem)).dwState ^= TCIS_BUTTONPRESSED;
            if TAB_InternalGetItemRect(infoPtr, newItem, core::ptr::addr_of_mut!(r), core::ptr::null_mut::<RECT>()) != 0 {
                InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(r), TRUE);
            }
        } else {
            let mut i: INT = 0;
            let mut pressed: BOOL = FALSE;

            /* any button pressed ? */
            i = 0;
            while (i as UINT) < (*infoPtr).uNumItem {
                if ((*TAB_GetItem(infoPtr, i)).dwState & TCIS_BUTTONPRESSED) != 0
                    && ((*infoPtr).iSelected != i)
                {
                    pressed = TRUE;
                    break;
                }
                i += 1;
            }

            if TAB_SendSimpleNotify(infoPtr, (TCN_SELCHANGING) as UINT) != 0 {
                return 0;
            }

            if pressed != 0 {
                TAB_DeselectAll(infoPtr, FALSE);
            } else {
                TAB_SetCurSel(infoPtr, newItem);
            }

            TAB_SendSimpleNotify(infoPtr, (TCN_SELCHANGE) as UINT);
        }
    }

    return 0;
}

unsafe extern "C" fn TAB_LButtonUp(infoPtr: *const TAB_INFO) -> LRESULT {
    TAB_SendSimpleNotify(infoPtr, NM_CLICK);

    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_RButtonUp(infoPtr: *const TAB_INFO) {
    TAB_SendSimpleNotify(infoPtr, NM_RCLICK);
}

unsafe extern "C" fn TAB_DrawLoneItemInterior(infoPtr: *const TAB_INFO, iItem: core::ffi::c_int) {
    let hdc: HDC = GetDC((*infoPtr).hwnd);
    let mut r: RECT = core::mem::zeroed();
    let mut rC: RECT = core::mem::zeroed();

    /* Clip UpDown control to not draw over it */
    if (*infoPtr).needsScrolling != 0 {
        GetWindowRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rC));
        GetWindowRect((*infoPtr).hwndUpDown, core::ptr::addr_of_mut!(r));
        ExcludeClipRect(
            hdc,
            r.left - rC.left,
            r.top - rC.top,
            r.right - rC.left,
            r.bottom - rC.top,
        );
    }
    TAB_DrawItemInterior(infoPtr, hdc, iItem as INT, core::ptr::null_mut::<RECT>());
    ReleaseDC((*infoPtr).hwnd, hdc);
}

unsafe extern "C" fn hottrack_refresh(infoPtr: *const TAB_INFO, tabIndex: INT) {
    if tabIndex == -1 {
        return;
    }

    if !GetWindowTheme((*infoPtr).hwnd).is_null() {
        let mut rect: RECT = core::mem::zeroed();
        TAB_InternalGetItemRect(infoPtr, tabIndex, core::ptr::addr_of_mut!(rect), core::ptr::null_mut());
        InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of!(rect), FALSE);
    } else {
        TAB_DrawLoneItemInterior(infoPtr, tabIndex);
    }
}

unsafe extern "system" fn TAB_HotTrackTimerProc(
    hwnd: HWND,
    uMsg: UINT,
    idEvent: UINT_PTR,
    dwTime: DWORD,
) {
    let mut infoPtr: *mut TAB_INFO = TAB_GetInfoPtr!(hwnd);

    if !infoPtr.is_null() && (*infoPtr).iHotTracked >= 0 {
        let mut pt: POINT = POINT { x: 0, y: 0 };

        if GetCursorPos(core::ptr::addr_of_mut!(pt)) == 0
            || WindowFromPoint(pt) != hwnd
        {
            let mut iRedraw: INT = (*infoPtr).iHotTracked;
            (*infoPtr).iHotTracked = -1;
            hottrack_refresh(infoPtr, iRedraw);

            KillTimer(hwnd, (TAB_HOTTRACK_TIMER) as UINT_PTR);
        }
    }
}

unsafe extern "C" fn TAB_RecalcHotTrack(
    mut infoPtr: *mut TAB_INFO,
    mut pos: *const LPARAM,
    mut out_redrawLeave: *mut ::core::ffi::c_int,
    mut out_redrawEnter: *mut ::core::ffi::c_int,
) {
    let mut item: ::core::ffi::c_int = -1;
    if !out_redrawLeave.is_null() {
        *out_redrawLeave = -1;
    }
    if !out_redrawEnter.is_null() {
        *out_redrawEnter = -1;
    }
    if ((*infoPtr).dwStyle & (TCS_HOTTRACK as DWORD) != 0)
        || !GetWindowTheme((*infoPtr).hwnd).is_null()
    {
        let mut pt: POINT = POINT { x: 0, y: 0 };
        let mut flags: UINT = 0;
        if pos.is_null() {
            GetCursorPos(&raw mut pt);
            ScreenToClient((*infoPtr).hwnd, &raw mut pt);
        } else {
            pt.x = LOWORD!(*pos) as ::core::ffi::c_short as LONG;
            pt.y = HIWORD!(*pos) as ::core::ffi::c_short as LONG;
        }
        item = TAB_InternalHitTest(infoPtr, pt, &raw mut flags) as ::core::ffi::c_int;
    }
    if item != (*infoPtr).iHotTracked {
        if (*infoPtr).iHotTracked >= 0 {
            if !out_redrawLeave.is_null() {
                *out_redrawLeave = (*infoPtr).iHotTracked as ::core::ffi::c_int;
            }
            if item < 0 {
                KillTimer(
                    (*infoPtr).hwnd,
                    TAB_HOTTRACK_TIMER as UINT_PTR,
                );
            }
        } else {
            let mut timerID: UINT = SetTimer(
                (*infoPtr).hwnd,
                TAB_HOTTRACK_TIMER as UINT_PTR,
                TAB_HOTTRACK_TIMER_INTERVAL as UINT,
                Some(
                    TAB_HotTrackTimerProc,
                ),
            ) as UINT;
            if timerID == 0 as UINT {
                return;
            }
        }
        (*infoPtr).iHotTracked = item as INT;
        if item >= 0 {
            if !out_redrawEnter.is_null() {
                *out_redrawEnter = item;
            }
        }
    }
}

unsafe extern "C" fn TAB_MouseMove(mut infoPtr: *mut TAB_INFO, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let mut redrawLeave: ::core::ffi::c_int = 0;
    let mut redrawEnter: ::core::ffi::c_int = 0;
    if !(*infoPtr).hwndToolTip.is_null() {
        TAB_RelayEvent(
            (*infoPtr).hwndToolTip,
            (*infoPtr).hwnd,
            WM_LBUTTONDOWN as UINT,
            wParam,
            lParam,
        );
    }
    TAB_RecalcHotTrack(
        infoPtr,
        core::ptr::addr_of_mut!(lParam),
        core::ptr::addr_of_mut!(redrawLeave),
        core::ptr::addr_of_mut!(redrawEnter),
    );
    hottrack_refresh(infoPtr, redrawLeave);
    hottrack_refresh(infoPtr, redrawEnter);
    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_AdjustRect(infoPtr: *const TAB_INFO, fLarger: WPARAM, prc: LPRECT) -> LRESULT {
    let mut iRightBottom: *mut LONG = core::ptr::null_mut::<LONG>();
    let mut iLeftTop: *mut LONG = core::ptr::null_mut::<LONG>();

    if prc.is_null() {
        return -(1) as LRESULT;
    }

    if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
        iRightBottom = core::ptr::addr_of_mut!((*prc).right);
        iLeftTop = core::ptr::addr_of_mut!((*prc).left);
    } else {
        iRightBottom = core::ptr::addr_of_mut!((*prc).bottom);
        iLeftTop = core::ptr::addr_of_mut!((*prc).top);
    }

    if fLarger != 0 {
        /* Add the height of the tabs. */
        if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0 {
            *iRightBottom = (*iRightBottom as UINT)
                .wrapping_add(((*infoPtr).tabHeight as UINT).wrapping_mul((*infoPtr).uNumRows))
                as LONG;
        } else {
            *iLeftTop = (*iLeftTop as UINT).wrapping_sub(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(
                        (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                            (3 as UINT)
                                .wrapping_mul((*infoPtr).uNumRows.wrapping_sub(1 as UINT))
                        } else {
                            0 as UINT
                        }),
                    ),
            ) as LONG;
        }

        /* Inflate the rectangle for the padding */
        InflateRect(prc, DISPLAY_AREA_PADDINGX, DISPLAY_AREA_PADDINGY);

        /* Inflate for the border */
        InflateRect(prc, CONTROL_BORDER_SIZEX, CONTROL_BORDER_SIZEY);
    } else {
        /* Deflate the rectangle for the border */
        InflateRect(prc, -(CONTROL_BORDER_SIZEX), -(CONTROL_BORDER_SIZEY));

        /* Deflate the rectangle for the padding */
        InflateRect(prc, -(DISPLAY_AREA_PADDINGX), -(DISPLAY_AREA_PADDINGY));

        /* Remove the height of the tabs. */
        if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0 {
            *iRightBottom = (*iRightBottom as UINT)
                .wrapping_sub(((*infoPtr).tabHeight as UINT).wrapping_mul((*infoPtr).uNumRows))
                as LONG;
        } else {
            *iLeftTop = (*iLeftTop as UINT).wrapping_add(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(
                        (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
                            (3 as UINT)
                                .wrapping_mul((*infoPtr).uNumRows.wrapping_sub(1 as UINT))
                        } else {
                            0 as UINT
                        }),
                    ),
            ) as LONG;
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_OnHScroll(mut infoPtr: *mut TAB_INFO, mut nScrollCode: ::core::ffi::c_int, mut nPos: ::core::ffi::c_int) -> LRESULT {
    if (nScrollCode) as u32 == SB_THUMBPOSITION && nPos != (*infoPtr).leftmostVisible {
        if nPos < (*infoPtr).leftmostVisible {
            (*infoPtr).leftmostVisible -= 1;
        } else {
            (*infoPtr).leftmostVisible += 1;
        }
        TAB_RecalcHotTrack(
            infoPtr,
            ::core::ptr::null::<LPARAM>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        TAB_InvalidateTabArea(infoPtr);
        SendMessageW(
            (*infoPtr).hwndUpDown,
            UDM_SETPOS as UINT,
            0 as WPARAM,
            MAKELONG!((*infoPtr).leftmostVisible as WORD, 0 as WORD) as LPARAM,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_SetupScrolling(mut infoPtr: *mut TAB_INFO, mut clientRect: *const RECT) {
    static mut emptyW: [WCHAR; 1] = [0 as WCHAR];
    let mut maxRange: INT = 0 as INT;

    if (*infoPtr).needsScrolling != 0 {
        let mut controlPos: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut vsize: INT = 0;
        let mut tabwidth: INT = 0;

        controlPos.right = (*clientRect).right;
        controlPos.left = ((controlPos.right as INT)
            - (2 as INT) * GetSystemMetrics((SM_CXHSCROLL) as i32) as INT)
            as LONG;

        if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0 {
            controlPos.top = (((*clientRect).bottom as INT) - (*infoPtr).tabHeight) as LONG;
            controlPos.bottom =
                ((controlPos.top as INT) + GetSystemMetrics((SM_CYHSCROLL) as i32) as INT) as LONG;
        } else {
            controlPos.bottom = (((*clientRect).top as INT) + (*infoPtr).tabHeight) as LONG;
            controlPos.top =
                ((controlPos.bottom as INT) - GetSystemMetrics((SM_CYHSCROLL) as i32) as INT) as LONG;
        }

        if (*infoPtr).hwndUpDown == core::ptr::null_mut() {
            (*infoPtr).hwndUpDown = CreateWindowW!(
                UPDOWN_CLASSW.as_ptr(),
                core::ptr::addr_of_mut!(emptyW) as LPCWSTR,
                (WS_VISIBLE | WS_CHILD | UDS_HORZ) as DWORD,
                controlPos.left as INT,
                controlPos.top as INT,
                (controlPos.right as INT) - (controlPos.left as INT),
                (controlPos.bottom as INT) - (controlPos.top as INT),
                (*infoPtr).hwnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        } else {
            SetWindowPos(
                (*infoPtr).hwndUpDown,
                core::ptr::null_mut(),
                controlPos.left as INT,
                controlPos.top as INT,
                (controlPos.right as INT) - (controlPos.left as INT),
                (controlPos.bottom as INT) - (controlPos.top as INT),
                (SWP_SHOWWINDOW | SWP_NOZORDER) as UINT,
            );
        }

        if (*infoPtr).uNumItem != 0 {
            vsize = ((*clientRect).right as INT
                - ((controlPos.right as INT) - (controlPos.left as INT) + (1 as INT)))
                as INT;
            maxRange = (*infoPtr).uNumItem as INT;
            tabwidth = (*TAB_GetItem(
                infoPtr,
                ((*infoPtr).uNumItem.wrapping_sub(1 as UINT)) as INT,
            ))
            .rect
            .right as INT;

            while maxRange > (0 as INT) {
                if (tabwidth as LONG) - (*TAB_GetItem(infoPtr, maxRange - (1 as INT))).rect.left
                    > vsize as LONG
                {
                    break;
                }
                maxRange -= 1;
            }

            if maxRange as UINT == (*infoPtr).uNumItem {
                maxRange -= 1;
            }
        }
    } else {
        /* If we once had a scroll control... hide it */
        if (*infoPtr).hwndUpDown != core::ptr::null_mut() {
            ShowWindow((*infoPtr).hwndUpDown, (SW_HIDE) as i32);
        }
    }

    if (*infoPtr).hwndUpDown != core::ptr::null_mut() {
        SendMessageW(
            (*infoPtr).hwndUpDown,
            UDM_SETRANGE32 as UINT,
            0 as WPARAM,
            maxRange as LPARAM,
        );
    }
}

unsafe extern "C" fn TAB_SetItemBounds(infoPtr: *mut TAB_INFO) {
    let mut fontMetrics: TEXTMETRICW = core::mem::zeroed();
    let mut curItem: UINT = 0;
    let mut curItemLeftPos: INT = 0;
    let mut curItemRowCount: INT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut clientRect: RECT = core::mem::zeroed();
    let mut iTemp: INT = 0;
    let mut rcItem: *mut RECT = core::ptr::null_mut();
    let mut iIndex: INT = 0;
    let mut icon_width: INT = 0;

    /*
     * We need to get text information so we need a DC and we need to select
     * a font.
     */
    hdc = GetDC((*infoPtr).hwnd);

    hFont = if !(*infoPtr).hFont.is_null() {
        (*infoPtr).hFont
    } else {
        GetStockObject((SYSTEM_FONT as i32)) as HFONT
    };
    hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;

    /*
     * We will base the rectangle calculations on the client rectangle
     * of the control.
     */
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(clientRect));

    /* if TCS_VERTICAL then swap the height and width so this code places the
       tabs along the top of the rectangle and we can just rotate them after
       rather than duplicate all of the below code */
    if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
        iTemp = clientRect.bottom as INT;
        clientRect.bottom = clientRect.right;
        clientRect.right = iTemp as LONG;
    }

    /* Now use hPadding and vPadding */
    (*infoPtr).uHItemPadding = (*infoPtr).uHItemPadding_s;
    (*infoPtr).uVItemPadding = (*infoPtr).uVItemPadding_s;

    /* The leftmost item will be "0" aligned */
    curItemLeftPos = 0;
    curItemRowCount = if (*infoPtr).uNumItem != 0 {
        1
    } else {
        0
    };

    if (*infoPtr).fHeightSet == 0 {
        let mut item_height: INT = 0;
        let mut icon_height: INT = 0;
        let mut cx: INT = 0;

        /* Use the current font to determine the height of a tab. */
        GetTextMetricsW(hdc, core::ptr::addr_of_mut!(fontMetrics));

        /* Get the icon height */
        if !(*infoPtr).himl.is_null() {
            ImageList_GetIconSize(
                (*infoPtr).himl,
                core::ptr::addr_of_mut!(cx),
                core::ptr::addr_of_mut!(icon_height),
            );
        }

        /* Take the highest between font or icon */
        if fontMetrics.tmHeight > icon_height {
            item_height = fontMetrics.tmHeight + 2;
        } else {
            item_height = icon_height;
        }

        /*
         * Make sure there is enough space for the letters + icon + growing the
         * selected item + extra space for the selected item.
         */
        (*infoPtr).tabHeight = (item_height
            + (if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
                2
            } else {
                1
            }) * ((*infoPtr).uVItemPadding as INT)) as INT;
    }

    /* Get the icon width */
    if !(*infoPtr).himl.is_null() {
        let mut cy: INT = 0;

        ImageList_GetIconSize(
            (*infoPtr).himl,
            core::ptr::addr_of_mut!(icon_width),
            core::ptr::addr_of_mut!(cy),
        );

        if ((*infoPtr).dwStyle & TCS_FIXEDWIDTH) != 0 {
            icon_width += 4;
        } else {
            /* Add padding if icon is present */
            icon_width += (*infoPtr).uHItemPadding as INT;
        }
    }

    curItem = 0;
    while curItem < (*infoPtr).uNumItem {
        let curr: *mut TAB_ITEM = TAB_GetItem(infoPtr, curItem as INT);

        /* Set the leftmost position of the tab. */
        (*curr).rect.left = curItemLeftPos as LONG;

        if ((*infoPtr).dwStyle & TCS_FIXEDWIDTH) != 0 {
            (*curr).rect.right = (*curr).rect.left
                + max((*infoPtr).tabWidth, icon_width) as LONG;
        } else if (*curr).pszText.is_null() {
            /* If no text use minimum tab width including padding. */
            if (*infoPtr).tabMinWidth < 0 {
                (*curr).rect.right =
                    (*curr).rect.left + GET_DEFAULT_MIN_TAB_WIDTH!(infoPtr) as LONG;
            } else {
                (*curr).rect.right = (*curr).rect.left + (*infoPtr).tabMinWidth as LONG;

                /* Add extra padding if icon is present */
                if !(*infoPtr).himl.is_null()
                    && (*infoPtr).tabMinWidth > 0
                    && (*infoPtr).tabMinWidth < DEFAULT_MIN_TAB_WIDTH
                    && ((*infoPtr).uHItemPadding as INT) > 1
                {
                    (*curr).rect.right += ((EXTRA_ICON_PADDING
                        * (((*infoPtr).uHItemPadding as INT) - 1)))
                        as LONG;
                }
            }
        } else {
            let mut tabwidth: INT = 0;
            let mut size: SIZE = core::mem::zeroed();
            /* Calculate how wide the tab is depending on the text it contains */
            GetTextExtentPoint32W(
                hdc,
                (*curr).pszText,
                lstrlenW((*curr).pszText),
                core::ptr::addr_of_mut!(size),
            );

            tabwidth = size.cx + icon_width
                + (2 * ((*infoPtr).uHItemPadding as INT));

            if (*infoPtr).tabMinWidth < 0 {
                tabwidth = max(tabwidth, GET_DEFAULT_MIN_TAB_WIDTH!(infoPtr));
            } else {
                tabwidth = max(tabwidth, (*infoPtr).tabMinWidth);
            }

            (*curr).rect.right = ((*curr).rect.left + tabwidth as LONG) as LONG;
        }

        /*
         * Check if this is a multiline tab control and if so
         * check to see if we should wrap the tabs
         *
         * Wrap all these tabs. We will arrange them evenly later.
         *
         */

        if (((*infoPtr).dwStyle & TCS_MULTILINE) != 0
            || ((*infoPtr).dwStyle & TCS_VERTICAL) != 0)
            && ((*curr).rect.right
                > (clientRect.right - CONTROL_BORDER_SIZEX - DISPLAY_AREA_PADDINGX))
        {
            (*curr).rect.right -= (*curr).rect.left;

            (*curr).rect.left = (0) as LONG;
            curItemRowCount += 1;
        }

        (*curr).rect.bottom = (0) as LONG;
        (*curr).rect.top = (curItemRowCount - 1) as LONG;

        /*
         * The leftmost position of the next item is the rightmost position
         * of this one.
         */
        if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
            curItemLeftPos = (*curr).rect.right as INT + BUTTON_SPACINGX;
            if ((*infoPtr).dwStyle & TCS_FLATBUTTONS) != 0 {
                curItemLeftPos += FLAT_BTN_SPACINGX;
            }
        } else {
            curItemLeftPos = (*curr).rect.right as INT;
        }

        curItem += 1;
    }

    if !(((*infoPtr).dwStyle & TCS_MULTILINE) != 0
        || ((*infoPtr).dwStyle & TCS_VERTICAL) != 0)
    {
        /*
         * Check if we need a scrolling control.
         */
        (*infoPtr).needsScrolling = ((curItemLeftPos
            + (2 * SELECTED_TAB_OFFSET))
            > clientRect.right) as BOOL;

        /* Don't need scrolling, then update infoPtr->leftmostVisible */
        if (*infoPtr).needsScrolling == 0 {
            (*infoPtr).leftmostVisible = 0;
        }
    } else {
        /*
         * No scrolling in Multiline or Vertical styles.
         */
        (*infoPtr).needsScrolling = FALSE;
        (*infoPtr).leftmostVisible = 0;
    }
    TAB_SetupScrolling(infoPtr, core::ptr::addr_of_mut!(clientRect));

    /* Set the number of rows */
    (*infoPtr).uNumRows = curItemRowCount as UINT;

    /* Arrange all tabs evenly if style says so */
    if ((*infoPtr).dwStyle & TCS_RAGGEDRIGHT) == 0
        && (((*infoPtr).dwStyle & TCS_MULTILINE) != 0
            || ((*infoPtr).dwStyle & TCS_VERTICAL) != 0)
        && ((*infoPtr).uNumItem > 0)
        && ((*infoPtr).uNumRows > 1)
    {
        let mut tabPerRow: INT = 0;
        let mut remTab: INT = 0;
        let mut iRow: INT = 0;
        let mut iItm: UINT = 0;
        let mut iCount: INT = 0;

        /*
         * Ok windows tries to even out the rows. place the same
         * number of tabs in each row. So lets give that a shot
         */

        tabPerRow = ((*infoPtr).uNumItem / (*infoPtr).uNumRows) as INT;
        remTab = ((*infoPtr).uNumItem % (*infoPtr).uNumRows) as INT;

        iItm = 0;
        iRow = 0;
        iCount = 0;
        curItemLeftPos = 0;
        while iItm < (*infoPtr).uNumItem {
            /* normalize the current rect */
            let curr_0: *mut TAB_ITEM = TAB_GetItem(infoPtr, iItm as INT);

            /* shift the item to the left side of the clientRect */
            (*curr_0).rect.right -= (*curr_0).rect.left;
            (*curr_0).rect.left = (0) as LONG;

            /* if we have reached the maximum number of tabs on this row */
            /* move to the next row, reset our current item left position and */
            /* the count of items on this row */

            if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                /* Vert: Add the remaining tabs in the *last* remainder rows */
                if iCount
                    >= (if iRow >= ((*infoPtr).uNumRows as INT) - remTab {
                        tabPerRow + 1
                    } else {
                        tabPerRow
                    })
                {
                    iRow += 1;
                    curItemLeftPos = 0;
                    iCount = 0;
                }
            } else {
                /* Horz: Add the remaining tabs in the *first* remainder rows */
                if iCount
                    >= (if iRow < remTab {
                        tabPerRow + 1
                    } else {
                        tabPerRow
                    })
                {
                    iRow += 1;
                    curItemLeftPos = 0;
                    iCount = 0;
                }
            }

            /* shift the item to the right to place it as the next item in this row */
            (*curr_0).rect.left += curItemLeftPos as LONG;
            (*curr_0).rect.right += curItemLeftPos as LONG;
            (*curr_0).rect.top = iRow as LONG;
            if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
                curItemLeftPos = (*curr_0).rect.right as INT + 1;
                if ((*infoPtr).dwStyle & TCS_FLATBUTTONS) != 0 {
                    curItemLeftPos += FLAT_BTN_SPACINGX;
                }
            } else {
                curItemLeftPos = (*curr_0).rect.right as INT;
            }

            iItm += 1;
            iCount += 1;
        }

        /*
         * Justify the rows
         */
        {
            let mut widthDiff: INT = 0;
            let mut iIndexStart: INT = 0;
            let mut iIndexEnd: INT = 0;
            let mut remainder: INT = 0;
            let mut iCount_0: INT = 0;

            while (iIndexStart as UINT) < (*infoPtr).uNumItem {
                let start: *mut TAB_ITEM = TAB_GetItem(infoPtr, iIndexStart);

                /*
                 * find the index of the row
                 */
                /* find the first item on the next row */
                iIndexEnd = iIndexStart;
                while (iIndexEnd as UINT) < (*infoPtr).uNumItem
                    && (*TAB_GetItem(infoPtr, iIndexEnd)).rect.top
                        == (*start).rect.top
                {
                    iIndexEnd += 1;
                }
                /* intentionally blank */

                /*
                 * we need to justify these tabs so they fill the whole given
                 * client area
                 *
                 */
                /* find the amount of space remaining on this row */
                widthDiff = (clientRect.right
                    - (2 * SELECTED_TAB_OFFSET)
                    - (*TAB_GetItem(infoPtr, iIndexEnd - 1))
                        .rect
                        .right) as INT;

                /* iCount is the number of tab items on this row */
                iCount_0 = iIndexEnd - iIndexStart;

                if iCount_0 > 1 {
                    remainder = widthDiff % iCount_0;
                    widthDiff = widthDiff / iCount_0;
                    /* add widthDiff/iCount, or extra space/items on row, to each item on this row */
                    iIndex = iIndexStart;
                    iCount_0 = 0;
                    while iIndex < iIndexEnd {
                        let item: *mut TAB_ITEM = TAB_GetItem(infoPtr, iIndex);

                        (*item).rect.left += (iCount_0 * widthDiff) as LONG;
                        (*item).rect.right +=
                            ((iCount_0 + 1) * widthDiff) as LONG;

                        iIndex += 1;
                        iCount_0 += 1;
                    }
                    (*TAB_GetItem(infoPtr, iIndex - 1))
                        .rect
                        .right += remainder as LONG;
                } else {
                    /* we have only one item on this row, make it take up the entire row */
                    (*start).rect.left = clientRect.left;
                    (*start).rect.right = (clientRect.right - 4) as LONG;
                }

                iIndexStart = iIndexEnd;
            }
        }
    }

    /* if TCS_VERTICAL rotate the tabs so they are along the side of the clientRect */
    if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
        let mut rcOriginal: RECT = core::mem::zeroed();
        iIndex = 0;
        while (iIndex as UINT) < (*infoPtr).uNumItem {
            rcItem = core::ptr::addr_of_mut!(
                (*TAB_GetItem(infoPtr, iIndex)).rect
            ) as *mut RECT;

            rcOriginal = *rcItem;

            /* this is rotating the items by 90 degrees clockwise around the center of the control */
            (*rcItem).top = rcOriginal.left - clientRect.left;
            (*rcItem).bottom = (*rcItem).top + (rcOriginal.right - rcOriginal.left);
            (*rcItem).left = rcOriginal.top;
            (*rcItem).right = rcOriginal.bottom;

            iIndex += 1;
        }
    }

    TAB_EnsureSelectionVisible(infoPtr);
    TAB_RecalcHotTrack(
        infoPtr,
        core::ptr::null::<LPARAM>(),
        core::ptr::null_mut::<INT>(),
        core::ptr::null_mut::<INT>(),
    );

    /* Cleanup */
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwnd, hdc);
}

unsafe extern "C" fn TAB_EraseTabInterior(
    mut infoPtr: *const TAB_INFO,
    mut hdc: HDC,
    mut iItem: INT,
    mut drawRect: *const RECT,
) {
    let mut hbr: HBRUSH = CreateSolidBrush(comctl32_color.clrBtnFace);
    let mut deleteBrush: BOOL = TRUE;
    let mut rTemp: RECT = *drawRect;
    if (*infoPtr).dwStyle & TCS_BUTTONS as DWORD != 0 {
        if iItem == (*infoPtr).iSelected {
            if (*infoPtr).dwStyle & TCS_OWNERDRAWFIXED as DWORD == 0 {
                DeleteObject(hbr as HGDIOBJ);
                hbr = GetSysColorBrush((COLOR_SCROLLBAR) as i32);
                SetTextColor(hdc, comctl32_color.clr3dFace);
                SetBkColor(hdc, comctl32_color.clr3dHilight);
                if comctl32_color.clr3dHilight == comctl32_color.clrWindow {
                    hbr = COMCTL32_hPattern55AABrush;
                }
                deleteBrush = FALSE as BOOL;
            }
            FillRect(hdc, &raw mut rTemp as LPCRECT, hbr);
        } else if (*infoPtr).dwStyle & TCS_FLATBUTTONS as DWORD != 0 {
            InflateRect(
                &raw mut rTemp,
                2 as ::core::ffi::c_int,
                2 as ::core::ffi::c_int,
            );
            FillRect(hdc, &raw mut rTemp as LPCRECT, hbr);
            if iItem == (*infoPtr).iHotTracked
                || iItem != (*infoPtr).iSelected && iItem == (*infoPtr).uFocus
            {
                DrawEdge(
                    hdc,
                    &raw mut rTemp,
                    BDR_RAISEDINNER as UINT,
                    BF_RECT as UINT,
                );
            }
        } else {
            FillRect(hdc, &raw mut rTemp as LPCRECT, hbr);
        }
    } else {
        InflateRect(
            &raw mut rTemp,
            -(2 as ::core::ffi::c_int),
            -(2 as ::core::ffi::c_int),
        );
        if GetWindowTheme((*infoPtr).hwnd).is_null() {
            FillRect(hdc, &raw mut rTemp as LPCRECT, hbr);
        }
    }
    if (*TAB_GetItem(infoPtr, iItem)).dwState & TCIS_HIGHLIGHTED as DWORD != 0 {
        if deleteBrush != 0 {
            DeleteObject(hbr as HGDIOBJ);
            deleteBrush = FALSE as BOOL;
        }
        hbr = GetSysColorBrush((COLOR_HIGHLIGHT) as i32);
        FillRect(hdc, &raw mut rTemp as LPCRECT, hbr);
    }
    if deleteBrush != 0 {
        DeleteObject(hbr as HGDIOBJ);
    }
}

unsafe extern "C" fn TAB_DrawItemInterior(
    mut infoPtr: *const TAB_INFO,
    mut hdc: HDC,
    mut iItem: INT,
    mut drawRect: *mut RECT,
) {
    let mut localRect: RECT = core::mem::zeroed();

    let mut htextPen: HPEN = core::ptr::null_mut();
    let mut holdPen: HPEN = core::ptr::null_mut();
    let mut oldBkMode: INT = 0;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);

    {
        let mut isVisible: BOOL = 0;
        let mut itemRect: RECT = core::mem::zeroed();
        let mut selectedRect: RECT = core::mem::zeroed();

        isVisible = TAB_InternalGetItemRect(infoPtr, iItem, &raw mut itemRect, &raw mut selectedRect);
        if isVisible == 0 {
            return;
        }

        drawRect = &raw mut localRect;

        if iItem == (*infoPtr).iSelected {
            *drawRect = selectedRect;
        } else {
            *drawRect = itemRect;
        }

        if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
            if iItem == (*infoPtr).iSelected {
                (*drawRect).left += 4;
                (*drawRect).top += 4;
                (*drawRect).right -= 4;

                if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                    if ((*infoPtr).dwStyle & TCS_BOTTOM) == 0 {
                        (*drawRect).right += 1;
                    }
                    (*drawRect).bottom -= 4;
                } else {
                    if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                        (*drawRect).top -= 2;
                        (*drawRect).bottom -= 4;
                    } else {
                        (*drawRect).bottom -= 1;
                    }
                }
            } else {
                InflateRect(drawRect, -2, -2);
            }
        } else {
            if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 && ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                if iItem != (*infoPtr).iSelected {
                    (*drawRect).left += 2;
                    InflateRect(drawRect, 0, -2);
                }
            } else if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                if iItem == (*infoPtr).iSelected {
                    (*drawRect).right += 1;
                } else {
                    (*drawRect).right -= 2;
                    InflateRect(drawRect, 0, -2);
                }
            } else if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                if iItem == (*infoPtr).iSelected {
                    (*drawRect).top -= 2;
                } else {
                    InflateRect(drawRect, -2, -2);
                    (*drawRect).bottom += 2;
                }
            } else {
                if iItem == (*infoPtr).iSelected {
                    (*drawRect).bottom += 3;
                } else {
                    (*drawRect).bottom -= 2;
                    InflateRect(drawRect, -2, 0);
                }
            }
        }
    }

    /* Clear interior */
    TAB_EraseTabInterior(infoPtr, hdc, iItem, drawRect);

    /* Draw the focus rectangle */
    if ((*infoPtr).dwStyle & TCS_FOCUSNEVER) == 0
        && (GetFocus() == (*infoPtr).hwnd)
        && (iItem == (*infoPtr).uFocus)
    {
        let mut rFocus: RECT = *drawRect;

        if ((*infoPtr).dwStyle & TCS_BUTTONS) == 0 {
            InflateRect(&raw mut rFocus, -3, -3);
        }
        if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 && ((*infoPtr).dwStyle & TCS_VERTICAL) == 0 {
            rFocus.top -= 3;
        }

        /* focus should stay on selected item for TCS_BUTTONS style */
        if !(((*infoPtr).dwStyle & TCS_BUTTONS) != 0 && (*infoPtr).iSelected != iItem) {
            DrawFocusRect(hdc, &raw mut rFocus as LPCRECT);
        }
    }

    /*
     * Text pen
     */
    htextPen = CreatePen((PS_SOLID as i32), 1, comctl32_color.clrBtnText);
    holdPen = SelectObject(hdc, htextPen as HGDIOBJ) as HPEN;
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    /*
     * Setup for text output
     */
    oldBkMode = SetBkMode(hdc, (TRANSPARENT as i32)) as INT;
    if GetWindowTheme((*infoPtr).hwnd).is_null() || ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
        if ((*infoPtr).dwStyle & TCS_HOTTRACK) != 0
            && (iItem == (*infoPtr).iHotTracked)
            && ((*infoPtr).dwStyle & TCS_FLATBUTTONS) == 0
        {
            SetTextColor(hdc, comctl32_color.clrHighlight);
        } else if ((*TAB_GetItem(infoPtr, iItem)).dwState & TCIS_HIGHLIGHTED) != 0 {
            SetTextColor(hdc, comctl32_color.clrHighlightText);
        } else {
            SetTextColor(hdc, comctl32_color.clrBtnText);
        }
    }

    /*
     * if owner draw, tell the owner to draw
     */
    if ((*infoPtr).dwStyle & TCS_OWNERDRAWFIXED) != 0 && IsWindow((*infoPtr).hwndNotify) != 0 {
        let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
        let mut id: UINT;

        (*drawRect).top += 2;
        (*drawRect).right -= 1;
        if iItem == (*infoPtr).iSelected {
            InflateRect(drawRect, -1, 0);
        }

        id = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID) as UINT;

        /* fill DRAWITEMSTRUCT */
        dis.CtlType = ODT_TAB;
        dis.CtlID = id;
        dis.itemID = iItem as UINT;
        dis.itemAction = ODA_DRAWENTIRE;
        dis.itemState = 0;
        if iItem == (*infoPtr).iSelected {
            dis.itemState |= ODS_SELECTED;
        }
        if (*infoPtr).uFocus == iItem {
            dis.itemState |= ODS_FOCUS;
        }
        dis.hwndItem = (*infoPtr).hwnd;
        dis.hDC = hdc;
        dis.rcItem = *drawRect;

        /* when extra data fits ULONG_PTR, store it directly */
        if (*infoPtr).cbInfo as usize > core::mem::size_of::<LPARAM>() as usize {
            dis.itemData = &raw mut (*TAB_GetItem(infoPtr, iItem)).extra as *mut BYTE as ULONG_PTR;
        } else {
            /* this could be considered broken on 64 bit, but that's how it works -
               only first 4 bytes are copied */
            dis.itemData = 0;
            memcpy(
                &raw mut dis.itemData as *mut ::core::ffi::c_void,
                &raw mut (*TAB_GetItem(infoPtr, iItem)).extra as *mut BYTE as *mut ULONG_PTR
                    as *const ::core::ffi::c_void,
                4 as size_t,
            );
        }

        /* draw notification */
        SendMessageW(
            (*infoPtr).hwndNotify,
            WM_DRAWITEM,
            id as WPARAM,
            &raw mut dis as LPARAM,
        );
    } else {
        let mut item: *mut TAB_ITEM = TAB_GetItem(infoPtr, iItem);
        let mut rcTemp: RECT = core::mem::zeroed();
        let mut rcImage: RECT = core::mem::zeroed();

        /* used to center the icon and text in the tab */
        let mut rcText: RECT = core::mem::zeroed();
        let mut center_offset_h: INT = 0;
        let mut center_offset_v: INT = 0;

        /* set rcImage to drawRect, we will use top & left in our ImageList_Draw call */
        rcImage = *drawRect;

        rcTemp = *drawRect;
        SetRectEmpty(&raw mut rcText);

        /* get the rectangle that the text fits in */
        if !(*item).pszText.is_null() {
            DrawTextW(
                hdc,
                (*item).pszText as LPCWSTR,
                -1,
                &raw mut rcText,
                DT_CALCRECT,
            );
        }

        /*
         * If not owner draw, then do the drawing ourselves.
         *
         * Draw the icon.
         */
        if !(*infoPtr).himl.is_null() && (*item).iImage != -1 {
            let mut cx: INT = 0;
            let mut cy: INT = 0;

            ImageList_GetIconSize((*infoPtr).himl, &raw mut cx, &raw mut cy);

            if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                center_offset_h = (((*drawRect).bottom - (*drawRect).top)
                    - (cy + (*infoPtr).uHItemPadding as INT + (rcText.right - rcText.left)))
                    / 2;
                center_offset_v = (((*drawRect).right - (*drawRect).left) - cx) / 2;
            } else {
                center_offset_h = (((*drawRect).right - (*drawRect).left)
                    - (cx + (*infoPtr).uHItemPadding as INT + (rcText.right - rcText.left)))
                    / 2;
                center_offset_v = (((*drawRect).bottom - (*drawRect).top) - cy) / 2;
            }

            /* if an item is selected, the icon is shifted up instead of down */
            if iItem == (*infoPtr).iSelected {
                center_offset_v -= (*infoPtr).uVItemPadding as INT / 2;
            } else {
                center_offset_v += (*infoPtr).uVItemPadding as INT / 2;
            }

            if ((*infoPtr).dwStyle & TCS_FIXEDWIDTH) != 0
                && ((*infoPtr).dwStyle & (TCS_FORCELABELLEFT | TCS_FORCEICONLEFT)) != 0
            {
                center_offset_h = (*infoPtr).uHItemPadding as INT;
            }

            if center_offset_h < 2 {
                center_offset_h = 2;
            }

            if center_offset_v < 0 {
                center_offset_v = 0;
            }

            if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 && ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                rcImage.top = (*drawRect).top + center_offset_h;
                /* if tab is TCS_VERTICAL and TCS_BOTTOM, the text is drawn from the */
                /* right side of the tab, but the image still uses the left as its x position */
                /* this keeps the image always drawn off of the same side of the tab */
                rcImage.left = (*drawRect).right - cx - center_offset_v;
                (*drawRect).top += cy + (*infoPtr).uHItemPadding as INT;
            } else if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                rcImage.top = (*drawRect).bottom - cy - center_offset_h;
                rcImage.left = (*drawRect).left + center_offset_v;
                (*drawRect).bottom -= cy + (*infoPtr).uHItemPadding as INT;
            } else {
                /* normal style, whether TCS_BOTTOM or not */
                rcImage.left = (*drawRect).left + center_offset_h;
                rcImage.top = (*drawRect).top + center_offset_v;
                (*drawRect).left += cx + (*infoPtr).uHItemPadding as INT;
            }

            ImageList_Draw(
                (*infoPtr).himl,
                (*item).iImage,
                hdc,
                rcImage.left,
                rcImage.top,
                ILD_NORMAL,
            );
        }

        /* Now position text */
        if ((*infoPtr).dwStyle & TCS_FIXEDWIDTH) != 0 && ((*infoPtr).dwStyle & TCS_FORCELABELLEFT) != 0 {
            center_offset_h = (*infoPtr).uHItemPadding as INT;
        } else {
            if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
                center_offset_h = (((*drawRect).bottom - (*drawRect).top) - (rcText.right - rcText.left))
                    / 2;
            } else {
                center_offset_h = (((*drawRect).right - (*drawRect).left) - (rcText.right - rcText.left))
                    / 2;
            }
        }

        if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
            if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                (*drawRect).top += center_offset_h;
            } else {
                (*drawRect).bottom -= center_offset_h;
            }

            center_offset_v = (((*drawRect).right - (*drawRect).left) - (rcText.bottom - rcText.top))
                / 2;
        } else {
            (*drawRect).left += center_offset_h;
            center_offset_v = (((*drawRect).bottom - (*drawRect).top) - (rcText.bottom - rcText.top))
                / 2;
        }

        /* if an item is selected, the text is shifted up instead of down */
        if iItem == (*infoPtr).iSelected {
            center_offset_v -= (*infoPtr).uVItemPadding as INT / 2;
        } else {
            center_offset_v += (*infoPtr).uVItemPadding as INT / 2;
        }

        if center_offset_v < 0 {
            center_offset_v = 0;
        }

        if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
            (*drawRect).left += center_offset_v;
        } else {
            (*drawRect).top += center_offset_v;
        }

        /* Draw the text */
        if ((*infoPtr).dwStyle & TCS_VERTICAL) != 0 {
            /* if we are vertical rotate the text and each character */
            let mut logfont: LOGFONTW = core::mem::zeroed();
            let mut hFont: HFONT = core::ptr::null_mut();
            let mut nEscapement: INT = 900;
            let mut nOrientation: INT = 900;

            if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                nEscapement = -900;
                nOrientation = -900;
            }

            /* to get a font with the escapement and orientation we are looking for, we need to */
            /* call CreateFontIndirect, which requires us to set the values of the logfont we pass in */
            if GetObjectW(
                (*infoPtr).hFont as HANDLE,
                core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
                &raw mut logfont as LPVOID,
            ) == 0
            {
                GetObjectW(
                    GetStockObject((DEFAULT_GUI_FONT as i32)) as HANDLE,
                    core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
                    &raw mut logfont as LPVOID,
                );
            }

            logfont.lfEscapement = nEscapement;
            logfont.lfOrientation = nOrientation;
            hFont = CreateFontIndirectW(&raw mut logfont);
            SelectObject(hdc, hFont as HGDIOBJ);

            if !(*item).pszText.is_null() {
                ExtTextOutW(
                    hdc,
                    if ((*infoPtr).dwStyle & TCS_BOTTOM) != 0 {
                        (*drawRect).right
                    } else {
                        (*drawRect).left
                    },
                    if ((*infoPtr).dwStyle & TCS_BOTTOM) == 0 {
                        (*drawRect).bottom
                    } else {
                        (*drawRect).top
                    },
                    ETO_CLIPPED,
                    drawRect,
                    (*item).pszText as LPCWSTR,
                    lstrlenW((*item).pszText as LPCWSTR) as UINT,
                    core::ptr::null(),
                );
            }

            DeleteObject(hFont as HGDIOBJ);
        } else {
            if !theme.is_null() && !(*item).pszText.is_null() {
                let mut partIndex: ::core::ffi::c_int = if iItem == (*infoPtr).iSelected {
                    (TABP_TABITEM) as i32
                } else {
                    (TABP_TOPTABITEM) as i32
                };
                let mut stateId: ::core::ffi::c_int = (TIS_NORMAL) as i32;

                if iItem == (*infoPtr).iSelected {
                    stateId = (TIS_SELECTED) as i32;
                } else if iItem == (*infoPtr).iHotTracked {
                    stateId = (TIS_HOT) as i32;
                } else if iItem == (*infoPtr).uFocus {
                    stateId = (TIS_FOCUSED) as i32;
                }

                DrawThemeText(
                    theme,
                    hdc,
                    partIndex,
                    stateId,
                    (*item).pszText as LPCWSTR,
                    lstrlenW((*item).pszText as LPCWSTR),
                    DT_LEFT | DT_SINGLELINE,
                    0,
                    drawRect,
                );
            } else if !(*item).pszText.is_null() {
                DrawTextW(
                    hdc,
                    (*item).pszText as LPCWSTR,
                    lstrlenW((*item).pszText as LPCWSTR) as ::core::ffi::c_int,
                    drawRect as LPRECT,
                    DT_LEFT | DT_SINGLELINE,
                );
            }
        }

        *drawRect = rcTemp; /* restore drawRect */
    }

    /*
     * Cleanup
     */
    SelectObject(hdc, hOldFont as HGDIOBJ);
    SetBkMode(hdc, oldBkMode);
    SelectObject(hdc, holdPen as HGDIOBJ);
    DeleteObject(htextPen as HGDIOBJ);
}

unsafe extern "C" fn TAB_DrawItem(infoPtr: *const TAB_INFO, hdc: HDC, iItem: INT) {
    let mut itemRect: RECT = core::mem::zeroed();
    let mut selectedRect: RECT = core::mem::zeroed();
    let mut isVisible: BOOL;
    let mut r: RECT = core::mem::zeroed();
    let mut fillRect: RECT = core::mem::zeroed();
    let mut r1: RECT = core::mem::zeroed();
    let mut clRight: INT = 0;
    let mut clBottom: INT = 0;
    let mut bkgnd: COLORREF;
    let mut corner: COLORREF;
    let mut theme: HTHEME = core::ptr::null_mut();

    /*
     * Get the rectangle for the item.
     */
    isVisible = TAB_InternalGetItemRect(infoPtr, iItem, &raw mut itemRect, &raw mut selectedRect);

    if isVisible != 0 {
        let mut rUD: RECT = core::mem::zeroed();
        let mut rC: RECT = core::mem::zeroed();

        /* Clip UpDown control to not draw over it */
        if (*infoPtr).needsScrolling != 0 {
            GetWindowRect((*infoPtr).hwnd, &raw mut rC);
            GetWindowRect((*infoPtr).hwndUpDown, &raw mut rUD);
            ExcludeClipRect(hdc, rUD.left - rC.left, rUD.top - rC.top, rUD.right - rC.left, rUD.bottom - rC.top);
        }

        /* If you need to see what the control is doing,
         * then override these variables. They will change what
         * fill colors are used for filling the tabs, and the
         * corners when drawing the edge.
         */
        bkgnd = comctl32_color.clrBtnFace;
        corner = comctl32_color.clrBtnFace;

        if ((*infoPtr).dwStyle & TCS_BUTTONS) != 0 {
            /* Get item rectangle */
            r = itemRect;

            /* Separators between flat buttons */
            if ((*infoPtr).dwStyle & TCS_FLATBUTTONS) != 0 && ((*infoPtr).exStyle & TCS_EX_FLATSEPARATORS) != 0 {
                r1 = r;
                r1.right += FLAT_BTN_SPACINGX - 2;
                DrawEdge(hdc, &raw mut r1, EDGE_ETCHED, BF_RIGHT);
            }

            if iItem == (*infoPtr).iSelected {
                DrawEdge(hdc, &raw mut r, EDGE_SUNKEN, BF_SOFT | BF_RECT);

                OffsetRect(&raw mut r, 1, 1);
            } else {
                /* ! selected */
                let state: DWORD = (*TAB_GetItem(infoPtr, iItem)).dwState;

                if (state & TCIS_BUTTONPRESSED) != 0 || iItem == (*infoPtr).uFocus {
                    DrawEdge(hdc, &raw mut r, EDGE_SUNKEN, BF_SOFT | BF_RECT);
                } else if ((*infoPtr).dwStyle & TCS_FLATBUTTONS) == 0 {
                    DrawEdge(hdc, &raw mut r, EDGE_RAISED, BF_SOFT | BF_RECT);
                }
            }
        } else {
            /* !TCS_BUTTONS */
            /* We draw a rectangle of different sizes depending on the selection
             * state. */
            if iItem == (*infoPtr).iSelected {
                let mut rect: RECT = core::mem::zeroed();
                GetClientRect((*infoPtr).hwnd, &raw mut rect);
                clRight = rect.right;
                clBottom = rect.bottom;
                r = selectedRect;
            } else {
                r = itemRect;
            }

            /*
             * Erase the background. (Delay it but setup rectangle.)
             * This is necessary when drawing the selected item since it is larger
             * than the others, it might overlap with stuff already drawn by the
             * other tabs
             */
            fillRect = r;

            /* Draw themed tabs - but only if they are at the top.
             * Windows draws even side or bottom tabs themed, with wacky results.
             * However, since in Wine apps may get themed that did not opt in via
             * a manifest avoid theming when we know the result will be wrong */
            theme = GetWindowTheme((*infoPtr).hwnd);
            if !theme.is_null()
                && ((*infoPtr).dwStyle & (TCS_VERTICAL | TCS_BOTTOM)) == 0
            {
                static partIds: [INT; 8] = [
                    /* Normal item */
                    (TABP_TABITEM) as i32,
                    (TABP_TABITEMLEFTEDGE) as i32,
                    (TABP_TABITEMRIGHTEDGE) as i32,
                    (TABP_TABITEMBOTHEDGE) as i32,
                    /* Selected tab */
                    (TABP_TOPTABITEM) as i32,
                    (TABP_TOPTABITEMLEFTEDGE) as i32,
                    (TABP_TOPTABITEMRIGHTEDGE) as i32,
                    (TABP_TOPTABITEMBOTHEDGE) as i32,
                ];
                let mut partIndex: INT = 0;
                let mut stateId: INT = (TIS_NORMAL) as i32;

                /* selected and unselected tabs have different parts */
                if iItem == (*infoPtr).iSelected {
                    partIndex += 4;
                }
                /* The part also differs on the position of a tab on a line.
                 * "Visually" determining the position works well enough. */
                GetClientRect((*infoPtr).hwnd, &raw mut r1);
                if selectedRect.left == 0 {
                    partIndex += 1;
                }
                if selectedRect.right == r1.right {
                    partIndex += 2;
                }

                if iItem == (*infoPtr).iSelected {
                    stateId = (TIS_SELECTED) as i32;
                } else if iItem == (*infoPtr).iHotTracked {
                    stateId = (TIS_HOT) as i32;
                } else if iItem == (*infoPtr).uFocus {
                    stateId = (TIS_FOCUSED) as i32;
                }

                /* Adjust rectangle for bottommost row */
                if ((*TAB_GetItem(infoPtr, iItem)).rect.top) as u32 == (*infoPtr).uNumRows - 1 {
                    r.bottom += 3;
                }

                DrawThemeBackground(theme, hdc, partIds[partIndex as usize], stateId, &raw mut r, core::ptr::null_mut());
                GetThemeBackgroundContentRect(theme, hdc, partIds[partIndex as usize], stateId, &raw mut r, &raw mut r);
            } else if (*infoPtr).dwStyle & TCS_VERTICAL != 0 {
                /* These are for adjusting the drawing of a Selected tab      */
                /* The initial values are for the normal case of non-Selected */
                let mut ZZ: INT = 1; /* Do not stretch if selected */
                if iItem == (*infoPtr).iSelected {
                    ZZ = 0;

                    /* if leftmost draw the line longer */
                    if selectedRect.top == 0 {
                        fillRect.top += CONTROL_BORDER_SIZEY;
                    }
                    /* if rightmost draw the line longer */
                    if selectedRect.bottom == clBottom {
                        fillRect.bottom -= CONTROL_BORDER_SIZEY;
                    }
                }

                if (*infoPtr).dwStyle & TCS_BOTTOM != 0 {
                    /* Adjust both rectangles to match native */
                    r.left += 1 - ZZ;

                    /* Clear interior */
                    SetBkColor(hdc, bkgnd);
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut fillRect, core::ptr::null_mut(), 0, core::ptr::null());

                    /* Draw rectangular edge around tab */
                    DrawEdge(hdc, &raw mut r, EDGE_RAISED, BF_SOFT | BF_RIGHT | BF_TOP | BF_BOTTOM);

                    /* Now erase the top corner and draw diagonal edge */
                    SetBkColor(hdc, corner);
                    r1.left = r.right - ROUND_CORNER_SIZE - 1;
                    r1.top = r.top;
                    r1.right = r.right;
                    r1.bottom = r1.top + ROUND_CORNER_SIZE;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.right -= 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDTOPLEFT);

                    /* Now erase the bottom corner and draw diagonal edge */
                    r1.left = r.right - ROUND_CORNER_SIZE - 1;
                    r1.bottom = r.bottom;
                    r1.right = r.right;
                    r1.top = r1.bottom - ROUND_CORNER_SIZE;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.right -= 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDBOTTOMLEFT);

                    if iItem == (*infoPtr).iSelected && selectedRect.top == 0 {
                        r1 = r;
                        r1.right = r1.left;
                        r1.left -= 1;
                        DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_TOP);
                    }
                } else {
                    /* Clear interior */
                    SetBkColor(hdc, bkgnd);
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut fillRect, core::ptr::null_mut(), 0, core::ptr::null());

                    /* Draw rectangular edge around tab */
                    DrawEdge(hdc, &raw mut r, EDGE_RAISED, BF_SOFT | BF_LEFT | BF_TOP | BF_BOTTOM);

                    /* Now erase the top corner and draw diagonal edge */
                    SetBkColor(hdc, corner);
                    r1.left = r.left;
                    r1.top = r.top;
                    r1.right = r1.left + ROUND_CORNER_SIZE + 1;
                    r1.bottom = r1.top + ROUND_CORNER_SIZE;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.left += 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDTOPRIGHT);

                    /* Now erase the bottom corner and draw diagonal edge */
                    r1.left = r.left;
                    r1.bottom = r.bottom;
                    r1.right = r1.left + ROUND_CORNER_SIZE + 1;
                    r1.top = r1.bottom - ROUND_CORNER_SIZE;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.left += 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_SUNKEN, BF_DIAGONAL_ENDTOPLEFT);
                }
            } else {
                /* ! TCS_VERTICAL */
                /* These are for adjusting the drawing of a Selected tab      */
                /* The initial values are for the normal case of non-Selected */
                if iItem == (*infoPtr).iSelected {
                    /* if leftmost draw the line longer */
                    if selectedRect.left == 0 {
                        fillRect.left += CONTROL_BORDER_SIZEX;
                    }
                    /* if rightmost draw the line longer */
                    if selectedRect.right == clRight {
                        fillRect.right -= CONTROL_BORDER_SIZEX;
                    }
                }

                if (*infoPtr).dwStyle & TCS_BOTTOM != 0 {
                    /* Adjust both rectangles for topmost row */
                    if ((*TAB_GetItem(infoPtr, iItem)).rect.top) as u32 == (*infoPtr).uNumRows - 1 {
                        fillRect.top -= 2;
                        r.top -= 1;
                    }

                    /* Clear interior */
                    SetBkColor(hdc, bkgnd);
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut fillRect, core::ptr::null_mut(), 0, core::ptr::null());

                    /* Draw rectangular edge around tab */
                    DrawEdge(hdc, &raw mut r, EDGE_RAISED, BF_SOFT | BF_LEFT | BF_BOTTOM | BF_RIGHT);

                    /* Now erase the righthand corner and draw diagonal edge */
                    SetBkColor(hdc, corner);
                    r1.left = r.right - ROUND_CORNER_SIZE;
                    r1.bottom = r.bottom;
                    r1.right = r.right;
                    r1.top = r1.bottom - ROUND_CORNER_SIZE - 1;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.bottom -= 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDBOTTOMLEFT);

                    /* Now erase the lefthand corner and draw diagonal edge */
                    r1.left = r.left;
                    r1.bottom = r.bottom;
                    r1.right = r1.left + ROUND_CORNER_SIZE;
                    r1.top = r1.bottom - ROUND_CORNER_SIZE - 1;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.bottom -= 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDTOPLEFT);

                    if iItem == (*infoPtr).iSelected {
                        r.top += 2;
                        r.left += 1;
                        if selectedRect.left == 0 {
                            r1 = r;
                            r1.bottom = r1.top;
                            r1.top -= 1;
                            DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_LEFT);
                        }
                    }
                } else {
                    /* Adjust both rectangles for bottommost row */
                    if ((*TAB_GetItem(infoPtr, iItem)).rect.top) as u32 == (*infoPtr).uNumRows - 1 {
                        fillRect.bottom += 3;
                        r.bottom += 2;
                    }

                    /* Clear interior */
                    SetBkColor(hdc, bkgnd);
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut fillRect, core::ptr::null_mut(), 0, core::ptr::null());

                    /* Draw rectangular edge around tab */
                    DrawEdge(hdc, &raw mut r, EDGE_RAISED, BF_SOFT | BF_LEFT | BF_TOP | BF_RIGHT);

                    /* Now erase the righthand corner and draw diagonal edge */
                    SetBkColor(hdc, corner);
                    r1.left = r.right - ROUND_CORNER_SIZE;
                    r1.top = r.top;
                    r1.right = r.right;
                    r1.bottom = r1.top + ROUND_CORNER_SIZE + 1;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.top += 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDBOTTOMRIGHT);

                    /* Now erase the lefthand corner and draw diagonal edge */
                    r1.left = r.left;
                    r1.top = r.top;
                    r1.right = r1.left + ROUND_CORNER_SIZE;
                    r1.bottom = r1.top + ROUND_CORNER_SIZE + 1;
                    ExtTextOutW(hdc, 0, 0, 2, &raw mut r1, core::ptr::null_mut(), 0, core::ptr::null());
                    r1.top += 1;
                    DrawEdge(hdc, &raw mut r1, EDGE_RAISED, BF_SOFT | BF_DIAGONAL_ENDTOPRIGHT);
                }
            }
        }

        TAB_DumpItemInternal(infoPtr, iItem as UINT);

        /* This modifies r to be the text rectangle. */
        TAB_DrawItemInterior(infoPtr, hdc, iItem, &raw mut r);
    }
}

unsafe extern "C" fn TAB_DrawBorder(infoPtr: *const TAB_INFO, hdc: HDC) {
    let mut rect: RECT = core::mem::zeroed();
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rect));

    /*
     * Adjust for the style
     */

    if (*infoPtr).uNumItem != 0 {
        if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0
            && ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) == 0
        {
            rect.bottom = (rect.bottom as UINT).wrapping_sub(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(CONTROL_BORDER_SIZEX as UINT),
            ) as LONG;
        } else if ((*infoPtr).dwStyle & (TCS_BOTTOM as DWORD)) != 0
            && ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0
        {
            rect.right = (rect.right as UINT).wrapping_sub(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(CONTROL_BORDER_SIZEX as UINT),
            ) as LONG;
        } else if ((*infoPtr).dwStyle & (TCS_VERTICAL as DWORD)) != 0 {
            rect.left = (rect.left as UINT).wrapping_add(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(CONTROL_BORDER_SIZEX as UINT),
            ) as LONG;
        } else {
            /* not TCS_VERTICAL and not TCS_BOTTOM */
            rect.top = (rect.top as UINT).wrapping_add(
                ((*infoPtr).tabHeight as UINT)
                    .wrapping_mul((*infoPtr).uNumRows)
                    .wrapping_add(CONTROL_BORDER_SIZEX as UINT),
            ) as LONG;
        }
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_DrawBorder\0" as *const u8 as *const ::core::ffi::c_char,
            2360 as ::core::ffi::c_int,
            b"border=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(rect)),
        );
    }

    if !theme.is_null() {
        DrawThemeBackground(
            theme,
            hdc,
            TABP_PANE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null::<RECT>(),
        );
    } else {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rect),
            EDGE_RAISED as UINT,
            (BF_SOFT | BF_RECT) as UINT,
        );
    }
}

unsafe extern "C" fn TAB_Refresh(infoPtr: *const TAB_INFO, hdc: HDC) {
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut i: INT = 0;

    if (*infoPtr).DoRedraw == 0 {
        return;
    }

    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
        i = 0 as INT;
        while (i as UINT) < (*infoPtr).uNumItem {
            TAB_DrawItem(infoPtr, hdc, i);
            i += 1;
        }
    } else {
        /* Draw all the non selected item first */
        i = 0 as INT;
        while (i as UINT) < (*infoPtr).uNumItem {
            if i != (*infoPtr).iSelected {
                TAB_DrawItem(infoPtr, hdc, i);
            }
            i += 1;
        }

        /* Now, draw the border, draw it before the selected item
         * since the selected item overwrites part of the border. */
        TAB_DrawBorder(infoPtr, hdc);

        /* Then, draw the selected item */
        TAB_DrawItem(infoPtr, hdc, (*infoPtr).iSelected);
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
}

unsafe extern "C" fn TAB_GetRowCount(infoPtr: *const TAB_INFO) -> DWORD {
    // TRACE("(%p)\n", infoPtr);
    (*infoPtr).uNumRows
}

unsafe extern "C" fn TAB_SetRedraw(infoPtr: *mut TAB_INFO, doRedraw: BOOL) -> LRESULT {
    (*infoPtr).DoRedraw = doRedraw;
    return 0;
}

unsafe extern "C" fn TAB_EnsureSelectionVisible(infoPtr: *mut TAB_INFO) {
    let iSelected: INT = (*infoPtr).iSelected;
    let iOrigLeftmostVisible: INT = (*infoPtr).leftmostVisible;

    if iSelected < 0 {
        return;
    }

    /* set the items row to the bottommost row or topmost row depending on
     * style */
    if ((*infoPtr).uNumRows > 1 as UINT)
        && (((*infoPtr).dwStyle & TCS_BUTTONS as DWORD) == 0)
    {
        let selected: *mut TAB_ITEM = TAB_GetItem(infoPtr, iSelected);
        let mut newselected: INT = 0;
        let mut iTargetRow: INT = 0;

        if ((*infoPtr).dwStyle & TCS_VERTICAL as DWORD) != 0 {
            newselected = (*selected).rect.left as INT;
        } else {
            newselected = (*selected).rect.top as INT;
        }

        /* the target row is always (number of rows - 1)
           as row 0 is furthest from the clientRect */
        iTargetRow = ((*infoPtr).uNumRows - 1 as UINT) as INT;

        if newselected != iTargetRow {
            let mut i: UINT = 0;
            if ((*infoPtr).dwStyle & TCS_VERTICAL as DWORD) != 0 {
                i = 0 as UINT;
                while i < (*infoPtr).uNumItem {
                    /* move everything in the row of the selected item to the iTargetRow */
                    let item: *mut TAB_ITEM = TAB_GetItem(infoPtr, i as INT);

                    if (*item).rect.left == newselected {
                        (*item).rect.left = iTargetRow as LONG;
                    } else {
                        if (*item).rect.left > newselected {
                            (*item).rect.left -= 1 as LONG;
                        }
                    }
                    i = i.wrapping_add(1);
                }
            } else {
                i = 0 as UINT;
                while i < (*infoPtr).uNumItem {
                    let item: *mut TAB_ITEM = TAB_GetItem(infoPtr, i as INT);

                    if (*item).rect.top == newselected {
                        (*item).rect.top = iTargetRow as LONG;
                    } else {
                        if (*item).rect.top > newselected {
                            (*item).rect.top -= 1 as LONG;
                        }
                    }
                    i = i.wrapping_add(1);
                }
            }
            TAB_RecalcHotTrack(
                infoPtr,
                core::ptr::null::<LPARAM>(),
                core::ptr::null_mut::<INT>(),
                core::ptr::null_mut::<INT>(),
            );
        }
    }

    /*
     * Do the trivial cases first.
     */
    if ((*infoPtr).needsScrolling == 0)
        || ((*infoPtr).hwndUpDown == 0 as HWND)
        || (((*infoPtr).dwStyle & TCS_VERTICAL as DWORD) != 0)
    {
        return;
    }

    if (*infoPtr).leftmostVisible >= iSelected {
        (*infoPtr).leftmostVisible = iSelected;
    } else {
        let selected: *mut TAB_ITEM = TAB_GetItem(infoPtr, iSelected);
        let mut r: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut width: INT = 0;
        let mut i: UINT = 0;

        /* Calculate the part of the client area that is visible */
        GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(r));
        width = r.right as INT;

        GetClientRect((*infoPtr).hwndUpDown, core::ptr::addr_of_mut!(r));
        width -= r.right as INT;

        if ((*selected).rect.right - (*selected).rect.left) >= width {
            /* Special case: width of selected item is greater than visible
             * part of control.
             */
            (*infoPtr).leftmostVisible = iSelected;
        } else {
            i = (*infoPtr).leftmostVisible as UINT;
            while i < (*infoPtr).uNumItem {
                if ((*selected).rect.right - (*TAB_GetItem(infoPtr, i as INT)).rect.left)
                    < width
                {
                    break;
                }
                i = i.wrapping_add(1);
            }
            (*infoPtr).leftmostVisible = i as INT;
        }
    }

    if (*infoPtr).leftmostVisible != iOrigLeftmostVisible {
        TAB_RecalcHotTrack(
            infoPtr,
            core::ptr::null::<LPARAM>(),
            core::ptr::null_mut::<INT>(),
            core::ptr::null_mut::<INT>(),
        );
    }

    SendMessageW(
        (*infoPtr).hwndUpDown,
        UDM_SETPOS as UINT,
        0 as WPARAM,
        MAKELONG!(
            (*infoPtr).leftmostVisible as WORD,
            0 as WORD,
        ) as LPARAM,
    );
}

unsafe extern "C" fn TAB_InvalidateTabArea(infoPtr: *const TAB_INFO) {
    let mut clientRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rInvalidate: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rAdjClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut lastRow: INT = ((*infoPtr).uNumRows as INT) - 1;
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    if lastRow < 0 {
        return;
    }

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(clientRect));
    rInvalidate = clientRect;
    rAdjClient = clientRect;

    TAB_AdjustRect(
        infoPtr,
        0 as WPARAM,
        core::ptr::addr_of_mut!(rAdjClient),
    );

    TAB_InternalGetItemRect(
        infoPtr,
        ((*infoPtr).uNumItem.wrapping_sub(1 as UINT)) as INT,
        core::ptr::addr_of_mut!(rect),
        core::ptr::null_mut::<RECT>(),
    );

    if ((*infoPtr).dwStyle & TCS_BOTTOM as DWORD) != 0
        && ((*infoPtr).dwStyle & TCS_VERTICAL as DWORD) != 0
    {
        rInvalidate.left = rAdjClient.right;
        if (*infoPtr).uNumRows == 1 as UINT {
            rInvalidate.bottom = (clientRect.top as INT
                + rect.bottom as INT
                + 2 * SELECTED_TAB_OFFSET) as LONG;
        }
    } else if ((*infoPtr).dwStyle & TCS_VERTICAL as DWORD) != 0 {
        rInvalidate.right = rAdjClient.left;
        if (*infoPtr).uNumRows == 1 as UINT {
            rInvalidate.bottom = (clientRect.top as INT
                + rect.bottom as INT
                + 2 * SELECTED_TAB_OFFSET) as LONG;
        }
    } else if ((*infoPtr).dwStyle & TCS_BOTTOM as DWORD) != 0 {
        rInvalidate.top = rAdjClient.bottom;
        if (*infoPtr).uNumRows == 1 as UINT {
            rInvalidate.right = (clientRect.left as INT
                + rect.right as INT
                + 2 * SELECTED_TAB_OFFSET) as LONG;
        }
    } else {
        rInvalidate.bottom = rAdjClient.top;
        if (*infoPtr).uNumRows == 1 as UINT {
            rInvalidate.right = (clientRect.left as INT
                + rect.right as INT
                + 2 * SELECTED_TAB_OFFSET) as LONG;
        }
    }

    /* Punch out the updown control */
    if (*infoPtr).needsScrolling != 0 && rInvalidate.right > 0 as LONG {
        let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect((*infoPtr).hwndUpDown, core::ptr::addr_of_mut!(r));
        if rInvalidate.right > clientRect.right - r.left {
            rInvalidate.right = rInvalidate.right - (r.right - r.left);
        } else {
            rInvalidate.right = clientRect.right - r.left;
        }
    }

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TAB_InvalidateTabArea\0" as *const u8 as *const core::ffi::c_char,
            2598 as core::ffi::c_int,
            b"invalidate (%s)\n\0" as *const u8 as *const core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(rInvalidate)),
        );
    }

    InvalidateRect(
        (*infoPtr).hwnd,
        core::ptr::addr_of_mut!(rInvalidate),
        TRUE,
    );
}

unsafe extern "C" fn TAB_Paint(infoPtr: *mut TAB_INFO, hdcPaint: HDC) -> LRESULT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();

    if !hdcPaint.is_null() {
        hdc = hdcPaint;
    } else {
        hdc = BeginPaint((*infoPtr).hwnd, core::ptr::addr_of_mut!(ps));
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TAB_Paint\0" as *const u8 as *const ::core::ffi::c_char,
                2613 as ::core::ffi::c_int,
                b"erase %d, rect=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                ps.fErase,
                wine_dbgstr_rect(core::ptr::addr_of_mut!(ps.rcPaint)),
            );
        }
    }

    TAB_Refresh(infoPtr, hdc);

    if hdcPaint.is_null() {
        EndPaint((*infoPtr).hwnd, core::ptr::addr_of_mut!(ps));
    }

    return 0;
}

unsafe extern "C" fn TAB_InsertItemT(
    infoPtr: *mut TAB_INFO,
    mut iItem: INT,
    pti: *const TCITEMW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut item: *mut TAB_ITEM = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();

    GetClientRect((*infoPtr).hwnd, &raw mut rect);

    if iItem < 0 {
        return -1 as LRESULT;
    }
    if iItem as UINT > (*infoPtr).uNumItem {
        iItem = (*infoPtr).uNumItem as INT;
    }

    TAB_DumpItemExternalT(pti, iItem as UINT, bUnicode);

    item = Alloc(TAB_ITEM_SIZE!(infoPtr) as u32) as *mut TAB_ITEM;
    if item.is_null() {
        return FALSE as LRESULT;
    }
    if DPA_InsertPtr((*infoPtr).items, iItem, item as *mut core::ffi::c_void)
        == -1
    {
        Free(item as LPVOID);
        return FALSE as LRESULT;
    }

    if (*infoPtr).uNumItem == 0 as UINT {
        (*infoPtr).iSelected = 0 as INT;
    } else if iItem <= (*infoPtr).iSelected {
        (*infoPtr).iSelected += 1;
    }

    (*infoPtr).uNumItem = (*infoPtr).uNumItem.wrapping_add(1);

    (*item).pszText = core::ptr::null_mut::<WCHAR>();
    if ((*pti).mask & TCIF_TEXT as UINT) != 0 {
        if bUnicode != 0 {
            Str_SetPtrW(&raw mut (*item).pszText, (*pti).pszText as LPCWSTR);
        } else {
            Str_SetPtrAtoW(&raw mut (*item).pszText, (*pti).pszText as LPSTR as LPCSTR);
        }
    }

    if ((*pti).mask & TCIF_IMAGE as UINT) != 0 {
        (*item).iImage = (*pti).iImage as INT;
    } else {
        (*item).iImage = -1 as INT;
    }

    if ((*pti).mask & TCIF_PARAM as UINT) != 0 {
        memcpy(
            &raw mut (*item).extra as *mut BYTE as *mut core::ffi::c_void,
            &raw const (*pti).lParam as *const core::ffi::c_void,
            EXTRA_ITEM_SIZE!(infoPtr) as usize,
        );
    } else {
        memset(
            &raw mut (*item).extra as *mut BYTE as *mut core::ffi::c_void,
            0,
            EXTRA_ITEM_SIZE!(infoPtr) as usize,
        );
    }

    TAB_SetItemBounds(infoPtr);
    if (*infoPtr).uNumItem > 1 as UINT {
        TAB_InvalidateTabArea(infoPtr);
    } else {
        InvalidateRect(
            (*infoPtr).hwnd,
            core::ptr::null::<RECT>(),
            TRUE,
        );
    }

    /* If we haven't set the current focus yet, set it now. */
    if (*infoPtr).uFocus == -1 {
        TAB_SetCurFocus(infoPtr, iItem);
    }

    return iItem as LRESULT;
}

unsafe extern "C" fn TAB_SetItemSize(infoPtr: *mut TAB_INFO, cx: INT, cy: INT) -> LRESULT {
    let mut lResult: LONG = 0;
    let mut bNeedPaint: BOOL = FALSE;

    lResult = MAKELONG!((*infoPtr).tabWidth as WORD, (*infoPtr).tabHeight as WORD) as LONG;

    /* UNDOCUMENTED: If requested Width or Height is 0 this means that program wants to use auto size. */
    if ((*infoPtr).dwStyle & TCS_FIXEDWIDTH) != 0 && ((*infoPtr).tabWidth != cx) {
        (*infoPtr).tabWidth = cx;
        bNeedPaint = TRUE;
    }

    if (*infoPtr).tabHeight != cy {
        (*infoPtr).fHeightSet = (cy != 0) as BOOL;
        if (*infoPtr).fHeightSet != 0 {
            (*infoPtr).tabHeight = cy;
        }

        bNeedPaint = TRUE;
    }

    if bNeedPaint != 0 {
        TAB_SetItemBounds(infoPtr);
        RedrawWindow(
            (*infoPtr).hwnd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
        );
    }

    return lResult as LRESULT;
}

unsafe extern "C" fn TAB_SetMinTabWidth(infoPtr: *mut TAB_INFO, cx: INT) -> LRESULT {
    let mut oldcx: INT = 0;
    if (*infoPtr).tabMinWidth < 0 {
        oldcx = DEFAULT_MIN_TAB_WIDTH;
    } else {
        oldcx = (*infoPtr).tabMinWidth;
    }
    (*infoPtr).tabMinWidth = cx;
    TAB_SetItemBounds(infoPtr);
    return oldcx as LRESULT;
}

unsafe extern "C" fn TAB_HighlightItem(infoPtr: *mut TAB_INFO, iItem: INT, fHighlight: BOOL) -> LRESULT {
    let mut lpState: LPDWORD = core::ptr::null_mut::<DWORD>();
    let mut oldState: DWORD = 0;
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    if iItem < 0 || iItem as UINT >= (*infoPtr).uNumItem {
        return FALSE as LRESULT;
    }

    lpState = &raw mut (*TAB_GetItem(infoPtr, iItem)).dwState as LPDWORD;
    oldState = *lpState;

    if fHighlight != 0 {
        *lpState |= TCIS_HIGHLIGHTED as DWORD;
    } else {
        *lpState &= !(TCIS_HIGHLIGHTED as DWORD);
    }

    if oldState != *lpState
        && TAB_InternalGetItemRect(infoPtr, iItem, &raw mut r, core::ptr::null_mut::<RECT>()) != 0
    {
        InvalidateRect((*infoPtr).hwnd, &raw mut r as LPCRECT, TRUE);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TAB_SetItemT(
    mut infoPtr: *mut TAB_INFO,
    mut iItem: INT,
    mut tabItem: LPTCITEMW,
    mut bUnicode: BOOL,
) -> LRESULT {
    let mut wineItem: *mut TAB_ITEM = core::ptr::null_mut::<TAB_ITEM>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TAB_SetItemT\0" as *const u8 as *const core::ffi::c_char,
            2769 as core::ffi::c_int,
            b"(%p,%d,%p,%s)\n\0" as *const u8 as *const core::ffi::c_char,
            infoPtr,
            iItem,
            tabItem,
            if bUnicode != 0 {
                b"true\0" as *const u8 as *const core::ffi::c_char
            } else {
                b"false\0" as *const u8 as *const core::ffi::c_char
            },
        );
    }

    if iItem < 0 || iItem as UINT >= (*infoPtr).uNumItem {
        return FALSE as LRESULT;
    }

    TAB_DumpItemExternalT(tabItem as *const TCITEMW, iItem as UINT, bUnicode);

    wineItem = TAB_GetItem(infoPtr, iItem);

    if ((*tabItem).mask & (TCIF_IMAGE as UINT)) != 0 {
        (*wineItem).iImage = (*tabItem).iImage as INT;
    }

    if ((*tabItem).mask & (TCIF_PARAM as UINT)) != 0 {
        memcpy(
            core::ptr::addr_of_mut!((*wineItem).extra) as *mut BYTE as *mut core::ffi::c_void,
            core::ptr::addr_of!((*tabItem).lParam) as *const core::ffi::c_void,
            (*infoPtr).cbInfo as size_t,
        );
    }

    if ((*tabItem).mask & (TCIF_RTLREADING as UINT)) != 0 {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"TAB_SetItemT\0" as *const u8 as *const core::ffi::c_char,
                2785 as core::ffi::c_int,
                b"TCIF_RTLREADING\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
    }

    if ((*tabItem).mask & (TCIF_STATE as UINT)) != 0 {
        (*wineItem).dwState = ((*wineItem).dwState & !(*tabItem).dwStateMask)
            | ((*tabItem).dwState & (*tabItem).dwStateMask);
    }

    if ((*tabItem).mask & (TCIF_TEXT as UINT)) != 0 {
        Free((*wineItem).pszText as LPVOID);
        (*wineItem).pszText = core::ptr::null_mut::<WCHAR>();
        if bUnicode != 0 {
            Str_SetPtrW(
                core::ptr::addr_of_mut!((*wineItem).pszText),
                (*tabItem).pszText as LPCWSTR,
            );
        } else {
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!((*wineItem).pszText),
                (*tabItem).pszText as LPSTR as LPCSTR,
            );
        }
    }

    /* Update and repaint tabs */
    TAB_SetItemBounds(infoPtr);
    let __REACTOS__: bool = true;
    if __REACTOS__ {
        InvalidateRect(
            (*infoPtr).hwnd,
            core::ptr::null::<RECT>(),
            TRUE as BOOL,
        );
    } else {
        TAB_InvalidateTabArea(infoPtr);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TAB_GetItemCount(infoPtr: *const TAB_INFO) -> LRESULT {
    // TRACE("\n");
    ((*infoPtr).uNumItem) as _
}

unsafe extern "C" fn TAB_GetItemT(
    mut infoPtr: *mut TAB_INFO,
    mut iItem: INT,
    mut tabItem: LPTCITEMW,
    mut bUnicode: BOOL,
) -> LRESULT {
    let mut wineItem: *mut TAB_ITEM = ::core::ptr::null_mut::<TAB_ITEM>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
            2824 as ::core::ffi::c_int,
            b"(%p,%d,%p,%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            infoPtr,
            iItem,
            tabItem,
            if bUnicode != 0 {
                b"true\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"false\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
    }

    if tabItem.is_null() {
        return FALSE as LRESULT;
    }

    if iItem < 0 || iItem as UINT >= (*infoPtr).uNumItem {
        if (*tabItem).mask & (TCIF_IMAGE as UINT) != 0 {
            (*tabItem).iImage = 0;
        }
        if (*tabItem).mask & (TCIF_PARAM as UINT) != 0 {
            (*tabItem).lParam = 0;
        }
        if (*tabItem).mask & (TCIF_STATE as UINT) != 0 {
            (*tabItem).dwState = 0;
        }
        return FALSE as LRESULT;
    }

    wineItem = TAB_GetItem(infoPtr, iItem);

    if (*tabItem).mask & (TCIF_IMAGE as UINT) != 0 {
        (*tabItem).iImage = (*wineItem).iImage as ::core::ffi::c_int;
    }

    if (*tabItem).mask & (TCIF_PARAM as UINT) != 0 {
        memcpy(
            &raw mut (*tabItem).lParam as *mut ::core::ffi::c_void,
            &raw mut (*wineItem).extra as *mut BYTE as *const ::core::ffi::c_void,
            (*infoPtr).cbInfo as size_t,
        );
    }

    if (*tabItem).mask & (TCIF_RTLREADING as UINT) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/tab.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TAB_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                2846 as ::core::ffi::c_int,
                b"TCIF_RTLREADING\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    if (*tabItem).mask & (TCIF_STATE as UINT) != 0 {
        (*tabItem).dwState = (*wineItem).dwState & (*tabItem).dwStateMask;
    }

    if (*tabItem).mask & (TCIF_TEXT as UINT) != 0 {
        if bUnicode != 0 {
            Str_GetPtrW(
                (*wineItem).pszText as LPCWSTR,
                (*tabItem).pszText,
                (*tabItem).cchTextMax as INT,
            );
        } else {
            Str_GetPtrWtoA(
                (*wineItem).pszText as LPCWSTR,
                (*tabItem).pszText as LPSTR,
                (*tabItem).cchTextMax as INT,
            );
        }
    }

    TAB_DumpItemExternalT(tabItem as *const TCITEMW, iItem as UINT, bUnicode);

    return TRUE as LRESULT;
}

unsafe extern "C" fn TAB_DeleteItem(mut infoPtr: *mut TAB_INFO, mut iItem: INT) -> LRESULT {
    let mut item: *mut TAB_ITEM = core::ptr::null_mut::<TAB_ITEM>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            2869 as ::core::ffi::c_int,
            b"(%p, %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            infoPtr,
            iItem,
        );
    }

    if iItem < 0 || iItem as UINT >= (*infoPtr).uNumItem {
        return (FALSE) as LRESULT;
    }

    TAB_InvalidateTabArea(infoPtr);
    item = TAB_GetItem(infoPtr, iItem);
    Free((*item).pszText as LPVOID);
    Free(item as LPVOID);
    (*infoPtr).uNumItem = (*infoPtr).uNumItem.wrapping_sub(1);
    DPA_DeletePtr((*infoPtr).items, iItem as ::core::ffi::c_int);

    if (*infoPtr).uNumItem == 0 {
        if (*infoPtr).iHotTracked >= 0 {
            KillTimer((*infoPtr).hwnd, TAB_HOTTRACK_TIMER as UINT_PTR);
            (*infoPtr).iHotTracked = (-(1)) as INT;
        }

        (*infoPtr).iSelected = (-(1)) as INT;
    } else {
        if iItem <= (*infoPtr).iHotTracked {
            /* When tabs move left/up, the hot track item may change */
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/tab.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TAB_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
                    2895 as ::core::ffi::c_int,
                    b"Recalc hot track\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
    }

    /* adjust the selected index */
    if iItem == (*infoPtr).iSelected {
        (*infoPtr).iSelected = (-(1)) as INT;
    } else if iItem < (*infoPtr).iSelected {
        (*infoPtr).iSelected -= 1;
    }

    /* reposition and repaint tabs */
    TAB_SetItemBounds(infoPtr);

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TAB_DeleteAllItems(infoPtr: *mut TAB_INFO) -> LRESULT {
    while (*infoPtr).uNumItem != 0 {
        TAB_DeleteItem(infoPtr, 0);
    }
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TAB_GetFont(infoPtr: *const TAB_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn TAB_SetFont(infoPtr: *mut TAB_INFO, hNewFont: HFONT) -> LRESULT {
    (*infoPtr).hFont = hNewFont;

    TAB_SetItemBounds(infoPtr);

    TAB_InvalidateTabArea(infoPtr);

    return 0;
}
unsafe extern "C" fn TAB_GetImageList(infoPtr: *const TAB_INFO) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_GetImageList\0" as *const u8 as *const ::core::ffi::c_char,
            2942 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return (*infoPtr).himl as LRESULT;
}

unsafe extern "C" fn TAB_SetImageList(infoPtr: *mut TAB_INFO, himlNew: HIMAGELIST) -> LRESULT {
    let mut himlPrev: HIMAGELIST = (*infoPtr).himl;
    (*infoPtr).himl = himlNew;
    TAB_SetItemBounds(infoPtr);
    InvalidateRect((*infoPtr).hwnd, core::ptr::null_mut(), TRUE);
    return himlPrev as LRESULT;
}

unsafe extern "C" fn TAB_GetUnicodeFormat(infoPtr: *const TAB_INFO) -> LRESULT {
    // TRACE("(%p)\n", infoPtr);
    ((*infoPtr).bUnicode) as _
}

unsafe extern "C" fn TAB_SetUnicodeFormat(
    mut infoPtr: *mut TAB_INFO,
    mut bUnicode: BOOL,
) -> LRESULT {
    let mut bTemp: BOOL = (*infoPtr).bUnicode;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_SetUnicodeFormat\0" as *const u8 as *const ::core::ffi::c_char,
            2966 as ::core::ffi::c_int,
            b"(%p %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            infoPtr,
            bUnicode,
        );
    }
    (*infoPtr).bUnicode = bUnicode;
    return bTemp as LRESULT;
}

unsafe extern "C" fn TAB_Size(mut infoPtr: *mut TAB_INFO) -> LRESULT {
    TAB_SetItemBounds(infoPtr);
    InvalidateRect((*infoPtr).hwnd, core::ptr::null_mut(), TRUE);
    return 0;
}

unsafe extern "C" fn TAB_Create(hwnd: HWND, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut TAB_INFO = core::ptr::null_mut::<TAB_INFO>();
    let mut fontMetrics: TEXTMETRICW = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    let mut hOldFont: HFONT = core::ptr::null_mut::<HFONT__>();
    let mut style: DWORD = 0;

    infoPtr = Alloc(core::mem::size_of::<TAB_INFO>() as DWORD) as *mut TAB_INFO;

    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as DWORD_PTR as LONG_PTR);

    (*infoPtr).hwnd = hwnd;
    (*infoPtr).hwndNotify = (*(lParam as LPCREATESTRUCTW)).hwndParent;
    (*infoPtr).uNumItem = 0 as UINT;
    (*infoPtr).uNumRows = 0 as UINT;
    (*infoPtr).uHItemPadding = 6 as USHORT;
    (*infoPtr).uVItemPadding = 3 as USHORT;
    (*infoPtr).uHItemPadding_s = 6 as USHORT;
    (*infoPtr).uVItemPadding_s = 3 as USHORT;
    (*infoPtr).hFont = 0 as HFONT;
    (*infoPtr).items = DPA_Create(8 as ::core::ffi::c_int) as HDPA;
    (*infoPtr).hcurArrow = LoadCursorW(0 as HINSTANCE, IDC_ARROW as LPWSTR) as HCURSOR;
    (*infoPtr).iSelected = -(1 as INT);
    (*infoPtr).iHotTracked = -(1 as INT);
    (*infoPtr).uFocus = -(1 as INT);
    (*infoPtr).hwndToolTip = 0 as HWND;
    (*infoPtr).DoRedraw = TRUE as BOOL;
    (*infoPtr).needsScrolling = FALSE as BOOL;
    (*infoPtr).hwndUpDown = 0 as HWND;
    (*infoPtr).leftmostVisible = 0 as INT;
    (*infoPtr).fHeightSet = FALSE as BOOL;
    (*infoPtr).bUnicode = IsWindowUnicode(hwnd);
    (*infoPtr).cbInfo = ::core::mem::size_of::<LPARAM>() as INT;

    /* The tab control always has the WS_CLIPSIBLINGS style. Even
       if you don't specify it in CreateWindow. This is necessary in
       order for paint to work correctly. This follows windows behaviour. */
    style = GetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int) as DWORD;
    if (style & (TCS_VERTICAL as DWORD)) != 0 {
        style |= TCS_MULTILINE as DWORD;
    }
    style |= WS_CLIPSIBLINGS as DWORD;
    SetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int, style as LONG);

    (*infoPtr).dwStyle = style;
    (*infoPtr).exStyle = if (style & (TCS_FLATBUTTONS as DWORD)) != 0 {
        TCS_EX_FLATSEPARATORS as DWORD
    } else {
        0 as DWORD
    };

    if ((*infoPtr).dwStyle & (TCS_TOOLTIPS as DWORD)) != 0 {
        /* Create tooltip control */
        (*infoPtr).hwndToolTip = CreateWindowExW(
            0 as DWORD,
            (TOOLTIPS_CLASSW).as_ptr() as LPCWSTR,
            core::ptr::null::<WCHAR>(),
            WS_POPUP as DWORD,
            CW_USEDEFAULT as INT,
            CW_USEDEFAULT as INT,
            CW_USEDEFAULT as INT,
            CW_USEDEFAULT as INT,
            hwnd,
            0 as HMENU,
            0 as HINSTANCE,
            0 as LPVOID,
        );

        /* Send NM_TOOLTIPSCREATED notification */
        if !(*infoPtr).hwndToolTip.is_null() {
            let mut nmttc: NMTOOLTIPSCREATED = core::mem::zeroed();

            nmttc.hdr.hwndFrom = hwnd;
            nmttc.hdr.idFrom = GetWindowLongPtrW(hwnd, GWLP_ID as ::core::ffi::c_int) as UINT_PTR;
            nmttc.hdr.code = NM_TOOLTIPSCREATED as UINT;
            nmttc.hwndToolTips = (*infoPtr).hwndToolTip;

            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                GetWindowLongPtrW(hwnd, GWLP_ID as ::core::ffi::c_int) as WPARAM,
                core::ptr::addr_of_mut!(nmttc) as LPARAM,
            );
        }
    }

    OpenThemeData((*infoPtr).hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);

    /*
     * We need to get text information so we need a DC and we need to select
     * a font.
     */
    hdc = GetDC(hwnd);
    hOldFont = SelectObject(hdc, GetStockObject(SYSTEM_FONT as ::core::ffi::c_int)) as HFONT;

    /* Use the system font to determine the initial height of a tab. */
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(fontMetrics));

    /*
     * Make sure there is enough space for the letters + growing the
     * selected item + extra space for the selected item.
     */
    (*infoPtr).tabHeight = (fontMetrics.tmHeight as INT
        + SELECTED_TAB_OFFSET as INT
        + (if ((*infoPtr).dwStyle & (TCS_BUTTONS as DWORD)) != 0 {
            2 as INT
        } else {
            1 as INT
        }) * (*infoPtr).uVItemPadding as INT) as INT;

    /* Initialize the width of a tab. */
    if ((*infoPtr).dwStyle & (TCS_FIXEDWIDTH as DWORD)) != 0 {
        (*infoPtr).tabWidth = GetDeviceCaps(hdc, LOGPIXELSX as ::core::ffi::c_int) as INT;
    }

    (*infoPtr).tabMinWidth = -(1 as INT);

    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC(hwnd, hdc);

    return 0 as LRESULT;
}

unsafe extern "C" fn TAB_Destroy(infoPtr: *mut TAB_INFO) -> LRESULT {
    let mut iItem: INT = 0;

    SetWindowLongPtrW((*infoPtr).hwnd, 0 as ::core::ffi::c_int, 0 as LONG_PTR);

    iItem = (*infoPtr).uNumItem.wrapping_sub(1 as UINT) as INT;
    while iItem >= 0 as INT {
        let mut tab: *mut TAB_ITEM = TAB_GetItem(infoPtr, iItem);

        DPA_DeletePtr((*infoPtr).items, iItem as ::core::ffi::c_int);
        (*infoPtr).uNumItem = (*infoPtr).uNumItem.wrapping_sub(1);

        Free((*tab).pszText as LPVOID);
        Free(tab as LPVOID);
        iItem -= 1;
    }
    DPA_Destroy((*infoPtr).items);
    (*infoPtr).items = ::core::ptr::null_mut::<_DPA>();

    if !(*infoPtr).hwndToolTip.is_null() {
        DestroyWindow((*infoPtr).hwndToolTip);
    }

    if !(*infoPtr).hwndUpDown.is_null() {
        DestroyWindow((*infoPtr).hwndUpDown);
    }

    if (*infoPtr).iHotTracked >= 0 as INT {
        KillTimer((*infoPtr).hwnd, TAB_HOTTRACK_TIMER as UINT_PTR);
    }

    CloseThemeData(GetWindowTheme((*infoPtr).hwnd));

    Free(infoPtr as LPVOID);
    return 0 as LRESULT;
}

unsafe extern "C" fn theme_changed(infoPtr: *const TAB_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);
    return 0;
}

unsafe extern "C" fn TAB_NCCalcSize(wParam: WPARAM) -> LRESULT {
    if wParam == 0 {
        return 0;
    }
    return (WVR_ALIGNTOP) as LRESULT;
}

unsafe extern "C" fn TAB_SetItemExtra(infoPtr: *mut TAB_INFO, cbInfo: INT) -> LRESULT {
    if cbInfo < 0 || (*infoPtr).uNumItem != 0 {
        return (FALSE) as LRESULT;
    }

    (*infoPtr).cbInfo = cbInfo;
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TAB_RemoveImage(mut infoPtr: *mut TAB_INFO, mut image: INT) -> LRESULT {
    if ImageList_Remove((*infoPtr).himl, image) != 0 {
        let mut i: INT;
        let mut idx: *mut INT;
        let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        /* shift indices, repaint items if needed */
        i = 0;
        while (i as UINT) < (*infoPtr).uNumItem {
            idx = core::ptr::addr_of_mut!((*TAB_GetItem(infoPtr, i)).iImage);
            if *idx >= image {
                if *idx == image {
                    *idx = -1;
                } else {
                    *idx -= 1;
                }

                /* repaint item */
                if TAB_InternalGetItemRect(infoPtr, i, core::ptr::addr_of_mut!(r), core::ptr::null_mut()) != 0 {
                    InvalidateRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(r) as LPCRECT, TRUE);
                }
            }
            i += 1;
        }
    }

    return 0;
}

unsafe extern "C" fn TAB_SetExtendedStyle(infoPtr: *mut TAB_INFO, mut exMask: DWORD, mut exStyle: DWORD) -> LRESULT {
    let mut prevstyle: DWORD = (*infoPtr).exStyle;

    /* zero mask means all styles */
    if exMask == (0 as DWORD) {
        exMask = !(0 as DWORD);
    }

    if exMask & (TCS_EX_REGISTERDROP as DWORD) != 0 {
        exMask &= !(TCS_EX_REGISTERDROP as DWORD);
        exStyle &= !(TCS_EX_REGISTERDROP as DWORD);
    }

    if exMask & (TCS_EX_FLATSEPARATORS as DWORD) != 0 {
        if (prevstyle ^ exStyle) & (TCS_EX_FLATSEPARATORS as DWORD) != 0 {
            (*infoPtr).exStyle ^= TCS_EX_FLATSEPARATORS as DWORD;
            TAB_InvalidateTabArea(infoPtr);
        }
    }

    return prevstyle as LRESULT;
}

unsafe extern "C" fn TAB_GetExtendedStyle(infoPtr: *const TAB_INFO) -> LRESULT {
    ((*infoPtr).exStyle) as _
}

unsafe extern "C" fn TAB_DeselectAll(mut infoPtr: *mut TAB_INFO, mut excludesel: BOOL) -> LRESULT {
    let mut paint: BOOL = FALSE;
    let mut i: INT;
    let mut selected: INT = (*infoPtr).iSelected;

    if !((*infoPtr).dwStyle & TCS_BUTTONS != 0) {
        return 0;
    }

    i = 0;
    while (i as UINT) < (*infoPtr).uNumItem {
        if ((*TAB_GetItem(infoPtr, i)).dwState & TCIS_BUTTONPRESSED != 0)
            && (selected != i)
        {
            (*TAB_GetItem(infoPtr, i)).dwState &= !TCIS_BUTTONPRESSED;
            paint = TRUE;
        }
        i += 1;
    }

    if excludesel == 0 && (selected != -(1)) {
        (*TAB_GetItem(infoPtr, selected)).dwState &= !TCIS_BUTTONPRESSED;
        (*infoPtr).iSelected = -(1);
        paint = TRUE;
    }

    if paint != 0 {
        TAB_InvalidateTabArea(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TAB_StyleChanged(infoPtr: *mut TAB_INFO, wStyleType: WPARAM, lpss: *const STYLESTRUCT) -> INT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_StyleChanged\0" as *const u8 as *const ::core::ffi::c_char,
            3287 as ::core::ffi::c_int,
            b"(styletype=%lx, styleOld=0x%08x, styleNew=0x%08x)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wStyleType,
            (*lpss).styleOld,
            (*lpss).styleNew,
        );
    }

    if wStyleType != GWL_STYLE as WPARAM {
        return 0;
    }

    (*infoPtr).dwStyle = (*lpss).styleNew;

    TAB_SetItemBounds(infoPtr);
    InvalidateRect((*infoPtr).hwnd, core::ptr::null::<RECT>(), TRUE);

    return 0;
}

unsafe extern "system" fn TAB_WindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut TAB_INFO = TAB_GetInfoPtr!(hwnd) as *mut TAB_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/tab.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TAB_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            3304 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wParam=%lx lParam=%lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (uMsg != WM_CREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    let mut current_block_65: u64;
    match uMsg {
        TCM_GETIMAGELIST => return TAB_GetImageList(infoPtr),
        TCM_SETIMAGELIST => return TAB_SetImageList(infoPtr, lParam as HIMAGELIST),
        TCM_GETITEMCOUNT => return TAB_GetItemCount(infoPtr),
        TCM_GETITEMA | TCM_GETITEMW => {
            return TAB_GetItemT(
                infoPtr,
                wParam as INT,
                lParam as LPTCITEMW,
                (uMsg == TCM_GETITEMW) as ::core::ffi::c_int,
            );
        }
        TCM_SETITEMA | TCM_SETITEMW => {
            return TAB_SetItemT(
                infoPtr,
                wParam as INT,
                lParam as LPTCITEMW,
                (uMsg == TCM_SETITEMW) as ::core::ffi::c_int,
            );
        }
        TCM_DELETEITEM => return TAB_DeleteItem(infoPtr, wParam as INT),
        TCM_DELETEALLITEMS => return TAB_DeleteAllItems(infoPtr),
        TCM_GETITEMRECT => {
            return TAB_GetItemRect(infoPtr, wParam as INT, lParam as *mut RECT) as LRESULT;
        }
        TCM_GETCURSEL => return TAB_GetCurSel(infoPtr),
        TCM_HITTEST => return TAB_HitTest(infoPtr, lParam as LPTCHITTESTINFO),
        TCM_SETCURSEL => return TAB_SetCurSel(infoPtr, wParam as INT),
        TCM_INSERTITEMA | TCM_INSERTITEMW => {
            return TAB_InsertItemT(
                infoPtr,
                wParam as INT,
                lParam as *mut TCITEMW,
                (uMsg == TCM_INSERTITEMW) as ::core::ffi::c_int,
            );
        }
        TCM_SETITEMEXTRA => return TAB_SetItemExtra(infoPtr, wParam as INT),
        TCM_ADJUSTRECT => {
            return TAB_AdjustRect(infoPtr, wParam as BOOL as WPARAM, lParam as LPRECT);
        }
        TCM_SETITEMSIZE => {
            return TAB_SetItemSize(
                infoPtr,
                (lParam as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as INT,
                (lParam as DWORD_PTR >> 16 as ::core::ffi::c_int & 0xffff as DWORD_PTR) as WORD
                    as INT,
            );
        }
        TCM_REMOVEIMAGE => return TAB_RemoveImage(infoPtr, wParam as INT),
        TCM_SETPADDING => return TAB_SetPadding(infoPtr, lParam),
        TCM_GETROWCOUNT => return TAB_GetRowCount(infoPtr) as LRESULT,
        TCM_GETUNICODEFORMAT => return TAB_GetUnicodeFormat(infoPtr),
        TCM_SETUNICODEFORMAT => return TAB_SetUnicodeFormat(infoPtr, wParam as BOOL),
        TCM_HIGHLIGHTITEM => {
            return TAB_HighlightItem(
                infoPtr,
                wParam as INT,
                (lParam as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as BOOL,
            );
        }
        TCM_GETTOOLTIPS => return TAB_GetToolTips(infoPtr),
        TCM_SETTOOLTIPS => return TAB_SetToolTips(infoPtr, wParam as HWND),
        TCM_GETCURFOCUS => return TAB_GetCurFocus(infoPtr),
        TCM_SETCURFOCUS => return TAB_SetCurFocus(infoPtr, wParam as INT),
        TCM_SETMINTABWIDTH => return TAB_SetMinTabWidth(infoPtr, lParam as INT),
        TCM_DESELECTALL => return TAB_DeselectAll(infoPtr, wParam as BOOL),
        TCM_GETEXTENDEDSTYLE => return TAB_GetExtendedStyle(infoPtr),
        TCM_SETEXTENDEDSTYLE => {
            return TAB_SetExtendedStyle(infoPtr, wParam as DWORD, lParam as DWORD);
        }
        WM_GETFONT => return TAB_GetFont(infoPtr),
        WM_SETFONT => return TAB_SetFont(infoPtr, wParam as HFONT),
        WM_CREATE => return TAB_Create(hwnd, lParam),
        WM_NCDESTROY => return TAB_Destroy(infoPtr),
        WM_GETDLGCODE => return (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT,
        WM_LBUTTONDOWN => return TAB_LButtonDown(infoPtr, wParam, lParam),
        WM_LBUTTONUP => return TAB_LButtonUp(infoPtr),
        WM_NOTIFY => {
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY,
                wParam,
                lParam,
            );
        }
        WM_RBUTTONUP => {
            TAB_RButtonUp(infoPtr);
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
        WM_MOUSEMOVE => return TAB_MouseMove(infoPtr, wParam, lParam),
        WM_PRINTCLIENT | WM_PAINT => {
            return TAB_Paint(infoPtr, wParam as HDC);
        }
        WM_SIZE => return TAB_Size(infoPtr),
        WM_SETREDRAW => return TAB_SetRedraw(infoPtr, wParam as BOOL),
        WM_HSCROLL => {
            return TAB_OnHScroll(
                infoPtr,
                (wParam & 0xffff as DWORD_PTR) as WORD as ::core::ffi::c_int,
                (wParam >> 16 as ::core::ffi::c_int & 0xffff as DWORD_PTR) as WORD
                    as ::core::ffi::c_int,
            );
        }
        WM_STYLECHANGED => {
            return TAB_StyleChanged(
                infoPtr,
                wParam,
                lParam as LPSTYLESTRUCT as *const STYLESTRUCT,
            ) as LRESULT;
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        WM_THEMECHANGED => return theme_changed(infoPtr),
        WM_KILLFOCUS => {
            TAB_KillFocus(infoPtr);
            current_block_65 = 17324050931183573322;
        }
        WM_SETFOCUS => {
            current_block_65 = 17324050931183573322;
        }
        WM_KEYDOWN => return TAB_KeyDown(infoPtr, wParam, lParam),
        WM_NCHITTEST => return TAB_NCHitTest(infoPtr, lParam),
        WM_NCCALCSIZE => return TAB_NCCalcSize(wParam),
        _ => {
            if uMsg >= WM_USER
                && uMsg < WM_APP
                && COMCTL32_IsReflectedMessage(uMsg) == 0
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/tab.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TAB_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        3472 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }
            current_block_65 = 9859671972921157070;
        }
    }
    match current_block_65 {
        17324050931183573322 => {
            TAB_FocusChanging(infoPtr);
        }
        _ => {}
    }
    return DefWindowProcW(hwnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "C" fn TAB_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW) as UINT;
    wndClass.lpfnWndProc =
        Some(TAB_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut TAB_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    );
    wndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = (WC_TABCONTROLW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn TAB_Unregister() {
    UnregisterClassW((WC_TABCONTROLW).as_ptr(), core::ptr::null_mut());
}
