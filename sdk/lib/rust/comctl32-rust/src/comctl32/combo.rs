use crate::comctl32::button::set_control_clipping;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_combo);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_COMBOBOXW: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"C\0o\0m\0b\0o\0B\0o\0x\0\0\0")
};

static mut hComboBmp: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

static mut CBitWidth: UINT = 0;

static mut CBitHeight: UINT = 0;

pub const COMBO_YBORDERGAP: i32 = 5;
pub const KEYDATA_ALT: u32 = 0x2000;
pub const KEYDATA_PREVSTATE: u32 = 0x4000;
pub const ID_CB_LISTBOX: i32 = 1000;
pub const ID_CB_EDIT: i32 = 1001;

static clbName: [u16; 10] = [0x0043u16, 0x006F, 0x006D, 0x0062, 0x006F, 0x004C, 0x0042, 0x006F, 0x0078, 0x0000];
static editName: [u16; 5] = [0x0045u16, 0x0064, 0x0069, 0x0074, 0x0000];

unsafe extern "C" fn COMBO_Init() -> BOOL {
    let mut hDC: HDC = core::ptr::null_mut();
    if !hComboBmp.is_null() {
        return TRUE;
    }
    hDC = CreateCompatibleDC(core::ptr::null_mut());
    if !hDC.is_null() {
        let mut bRet: BOOL = FALSE;
        hComboBmp = LoadBitmapW(
            core::ptr::null_mut(),
            MAKEINTRESOURCEW!(OBM_COMBO) as LPCWSTR,
        );
        if !hComboBmp.is_null() {
            let mut bm: BITMAP = core::mem::zeroed();
            let mut hPrevB: HBITMAP = core::ptr::null_mut();
            let mut r: RECT = core::mem::zeroed();

            GetObjectW(
                hComboBmp as HANDLE,
                core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(bm) as LPVOID,
            );
            CBitHeight = bm.bmHeight as UINT;
            CBitWidth = bm.bmWidth as UINT;

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Init\0" as *const u8 as *const ::core::ffi::c_char,
                    107 as ::core::ffi::c_int,
                    b"combo bitmap [%i,%i]\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    CBitWidth,
                    CBitHeight,
                );
            }

            hPrevB = SelectObject(hDC, hComboBmp as HGDIOBJ) as HBITMAP;
            SetRect(
                core::ptr::addr_of_mut!(r),
                0,
                0,
                CBitWidth as ::core::ffi::c_int,
                CBitHeight as ::core::ffi::c_int,
            );
            InvertRect(hDC, core::ptr::addr_of_mut!(r) as LPCRECT);
            SelectObject(hDC, hPrevB as HGDIOBJ);
            bRet = TRUE;
        }
        DeleteDC(hDC);
        return bRet;
    }
    return FALSE;
}

unsafe extern "C" fn COMBO_NCCreate(hwnd: HWND, style: LONG) -> LRESULT {
    let mut lphc: *mut HEADCOMBO = core::ptr::null_mut::<HEADCOMBO>();
    if COMBO_Init() != 0 && {
        lphc = heap_alloc_zero(core::mem::size_of::<HEADCOMBO>() as SIZE_T) as *mut HEADCOMBO;
        !lphc.is_null()
    } {
        (*lphc).self_ = hwnd;
        SetWindowLongPtrW(hwnd, 0, lphc as LONG_PTR);
        (*lphc).dwStyle = (style & (!(WS_BORDER | WS_HSCROLL | WS_VSCROLL)) as i32) as UINT;
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            style & (!(WS_BORDER | WS_HSCROLL | WS_VSCROLL)) as i32,
        );
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            GetWindowLongW(hwnd, GWL_EXSTYLE) & (!WS_EX_CLIENTEDGE) as i32,
        );
        if (style & ((CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE) as i32)) == 0 {
            (*lphc).dwStyle |= CBS_HASSTRINGS as UINT;
        }
        if (GetWindowLongW(hwnd, GWL_EXSTYLE) & (WS_EX_NOPARENTNOTIFY as i32)) == 0 {
            (*lphc).wState |= CBF_NOTIFY as UINT;
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
                b"dll/win32/comctl32/combo.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBO_NCCreate\0" as *const u8 as *const ::core::ffi::c_char,
                150 as ::core::ffi::c_int,
                b"[%p], style = %08x\n\0" as *const u8 as *const ::core::ffi::c_char,
                lphc,
                (*lphc).dwStyle,
            );
        }
        return TRUE as LRESULT;
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn COMBO_NCDestroy(lphc: *mut HEADCOMBO) -> LRESULT {
    if !lphc.is_null() {
        if (CB_GETTYPE!(lphc) != CBS_SIMPLE) && !(*lphc).hWndLBox.is_null() {
            DestroyWindow((*lphc).hWndLBox);
        }

        SetWindowLongPtrW((*lphc).self_, 0, 0);
        heap_free(lphc as *mut ::core::ffi::c_void);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn combo_get_text_height(combo: *const HEADCOMBO) -> INT {
    let mut hdc: HDC = GetDC((*combo).self_);
    let mut prev_font: HFONT = core::ptr::null_mut();
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    if !(*combo).hFont.is_null() {
        prev_font = SelectObject(hdc, (*combo).hFont as HGDIOBJ) as HFONT;
    }

    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));

    if !prev_font.is_null() {
        SelectObject(hdc, prev_font as HGDIOBJ);
    }

    ReleaseDC((*combo).self_, hdc);

    return tm.tmHeight as INT + 4;
}

unsafe extern "C" fn CBGetTextAreaHeight(
    mut lphc: *mut HEADCOMBO,
    mut clip_item_height: BOOL,
) -> INT {
    let mut item_height: INT = 0;
    let mut text_height: INT = 0;
    if clip_item_height != 0
        && ((*lphc).dwStyle & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE) as UINT) == 0
    {
        text_height = combo_get_text_height(lphc);
        if (*lphc).item_height < text_height {
            (*lphc).item_height = text_height;
        }
    }
    item_height = (*lphc).item_height;

    /*
     * Check the ownerdraw case if we haven't asked the parent the size
     * of the item yet.
     */
    if ((*lphc).dwStyle & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE) as UINT) != 0
        && ((*lphc).wState & (CBF_MEASUREITEM as UINT)) != 0
    {
        let mut measureItem: MEASUREITEMSTRUCT = tagMEASUREITEMSTRUCT {
            CtlType: 0,
            CtlID: 0,
            itemID: 0,
            itemWidth: 0,
            itemHeight: 0,
            itemData: 0,
        };
        let mut clientRect: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut originalItemHeight: INT = item_height;
        let mut id: UINT = GetWindowLongPtrW((*lphc).self_, GWLP_ID as ::core::ffi::c_int) as UINT;

        /*
         * We use the client rect for the width of the item.
         */
        GetClientRect((*lphc).self_, &raw mut clientRect);

        (*lphc).wState &= !(CBF_MEASUREITEM as UINT);

        /*
         * Send a first one to measure the size of the text area
         */
        measureItem.CtlType = ODT_COMBOBOX as UINT;
        measureItem.CtlID = id;
        measureItem.itemID = (-(1 as ::core::ffi::c_int)) as UINT;
        measureItem.itemWidth = clientRect.right as UINT;
        measureItem.itemHeight =
            (item_height as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as UINT; /* ownerdrawn cb is taller */
        measureItem.itemData = 0 as ULONG_PTR;
        SendMessageW(
            (*lphc).owner,
            WM_MEASUREITEM as UINT,
            id as WPARAM,
            &raw mut measureItem as LPARAM,
        );
        item_height = (6 as UINT).wrapping_add(measureItem.itemHeight) as INT;

        /*
         * Send a second one in the case of a fixed ownerdraw list to calculate the
         * size of the list items. (we basically do this on behalf of the listbox)
         */
        if (*lphc).dwStyle & (CBS_OWNERDRAWFIXED as UINT) != 0 {
            measureItem.CtlType = ODT_COMBOBOX as UINT;
            measureItem.CtlID = id;
            measureItem.itemID = 0 as UINT;
            measureItem.itemWidth = clientRect.right as UINT;
            measureItem.itemHeight = originalItemHeight as UINT;
            measureItem.itemData = 0 as ULONG_PTR;
            SendMessageW(
                (*lphc).owner,
                WM_MEASUREITEM as UINT,
                id as WPARAM,
                &raw mut measureItem as LPARAM,
            );
            (*lphc).fixedOwnerDrawHeight = measureItem.itemHeight as INT;
        }

        /*
         * Keep the size for the next time
         */
        (*lphc).item_height = item_height;
    }

    return item_height;
}

unsafe extern "C" fn CBForceDummyResize(lphc: LPHEADCOMBO) {
    let mut windowRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut newComboHeight: ::core::ffi::c_int = 0;
    newComboHeight = CBGetTextAreaHeight(lphc as *mut HEADCOMBO, FALSE) as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int * COMBO_YBORDERSIZE!() as ::core::ffi::c_int;

    GetWindowRect((*lphc).self_, &raw mut windowRect);

    /*
     * We have to be careful, resizing a combobox also has the meaning that the
     * dropped rect will be resized. In this case, we want to trigger a resize
     * to recalculate layout but we don't want to change the dropped rectangle
     * So, we pass the height of text area of control as the height.
     * this will cancel-out in the processing of the WM_WINDOWPOSCHANGING
     * message.
     */
    (*lphc).wState |= CBF_NORESIZE as UINT;
    SetWindowPos(
        (*lphc).self_,
        ::core::ptr::null_mut(),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        windowRect.right - windowRect.left,
        newComboHeight,
        (SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
    );
    (*lphc).wState &= !(CBF_NORESIZE as UINT);

    CBCalcPlacement(lphc as *mut HEADCOMBO);
    CBResetPos(lphc as *mut HEADCOMBO);
}

unsafe extern "C" fn CBCalcPlacement(combo: *mut HEADCOMBO) {
    /* Start with the client rectangle. */
    GetClientRect((*combo).self_, &raw mut (*combo).textRect);

    /* Remove the borders */
    InflateRect(
        &raw mut (*combo).textRect,
        -COMBO_XBORDERSIZE!(),
        -COMBO_YBORDERSIZE!(),
    );

    /* Chop off the bottom part to fit with the height of the text area. */
    (*combo).textRect.bottom = ((*combo).textRect.top as INT
        + CBGetTextAreaHeight(combo, FALSE)) as LONG;

    /* The button starts the same vertical position as the text area. */
    (*combo).buttonRect = (*combo).textRect;

    /* If the combobox is "simple" there is no button. */
    if CB_GETTYPE!(combo) == CBS_SIMPLE {
        (*combo).buttonRect.bottom = 0;
        (*combo).buttonRect.right = (*combo).buttonRect.bottom;
        (*combo).buttonRect.left = (*combo).buttonRect.right;
    } else {
        /*
         * Let's assume the combobox button is the same width as the
         * scrollbar button.
         * size the button horizontally and cut-off the text area.
         */
        (*combo).buttonRect.left = (*combo).buttonRect.right
            - GetSystemMetrics((SM_CXVSCROLL) as i32);
        (*combo).textRect.right = (*combo).buttonRect.left;
    }

    /* In the case of a dropdown, there is an additional spacing between the text area and the button. */
    if CB_GETTYPE!(combo) == CBS_DROPDOWN {
        (*combo).textRect.right -= COMBO_EDITBUTTONSPACE!();
    }

    /* If we have an edit control, we space it away from the borders slightly. */
    if CB_GETTYPE!(combo) != CBS_DROPDOWNLIST {
        InflateRect(
            &raw mut (*combo).textRect,
            -EDIT_CONTROL_PADDING!(),
            -EDIT_CONTROL_PADDING!(),
        );
    }

    /* Adjust the size of the listbox popup. */
    if CB_GETTYPE!(combo) == CBS_SIMPLE {
        GetClientRect((*combo).self_, &raw mut (*combo).droppedRect);
        (*combo).droppedRect.top = (*combo).textRect.bottom + COMBO_YBORDERSIZE!();
    } else {
        /* Make sure the dropped width is as large as the combobox itself. */
        if (*combo).droppedWidth < ((*combo).buttonRect.right + COMBO_XBORDERSIZE!()) {
            (*combo).droppedRect.right = (*combo).droppedRect.left
                + ((*combo).buttonRect.right + COMBO_XBORDERSIZE!());

            /* In the case of a dropdown, the popup listbox is offset to the right. We want to make sure it's flush
               with the right side of the combobox */
            if CB_GETTYPE!(combo) == CBS_DROPDOWN {
                (*combo).droppedRect.right -= COMBO_EDITBUTTONSPACE!();
            }
        } else {
            (*combo).droppedRect.right = (*combo).droppedRect.left + (*combo).droppedWidth;
        }
    }

    /* Disallow negative window width */
    if (*combo).textRect.right < (*combo).textRect.left {
        (*combo).textRect.right = (*combo).textRect.left;
    }
}

unsafe extern "C" fn CBGetDroppedControlRect(lphc: LPHEADCOMBO, lpRect: LPRECT) {
    GetWindowRect((*lphc).self_, lpRect);

    (*lpRect).right = (*lpRect).left + (*lphc).droppedRect.right - (*lphc).droppedRect.left;
    (*lpRect).bottom = (*lpRect).top + (*lphc).droppedRect.bottom - (*lphc).droppedRect.top;
}

unsafe extern "C" fn COMBO_Create(
    hwnd: HWND,
    lphc: LPHEADCOMBO,
    hwndParent: HWND,
    style: LONG,
) -> LRESULT {
    OpenThemeData(hwnd, (WC_COMBOBOXW).as_ptr());
    if CB_GETTYPE!(lphc) == 0 {
        (*lphc).dwStyle |= CBS_SIMPLE;
    }
    if CB_GETTYPE!(lphc) != CBS_DROPDOWNLIST {
        (*lphc).wState |= CBF_EDIT;
    }

    (*lphc).owner = hwndParent;

    (*lphc).droppedWidth = 0;

    (*lphc).item_height = combo_get_text_height(lphc);

    /*
     * The first time we go through, we want to measure the ownerdraw item
     */
    (*lphc).wState |= CBF_MEASUREITEM;

    /*
     * Per default the comctl32 version of combo shows up to 30 items
     */
    (*lphc).visibleItems = 30;

    /* M$ IE 3.01 actually creates (and rapidly destroys) an ownerless combobox */

    if 1 != 0 {
        let mut lbeStyle: UINT = 0 as UINT;
        let mut lbeExStyle: UINT = 0 as UINT;

        /*
         * Initialize the dropped rect to the size of the client area of the
         * control and then, force all the areas of the combobox to be
         * recalculated.
         */
        GetClientRect(hwnd, core::ptr::addr_of_mut!((*lphc).droppedRect));
        CBCalcPlacement(lphc);

        /*
         * Adjust the position of the popup listbox if it's necessary
         */
        if CB_GETTYPE!(lphc) != CBS_SIMPLE {
            (*lphc).droppedRect.top = (*lphc).textRect.bottom + COMBO_YBORDERSIZE!() as LONG;

            /*
             * If it's a dropdown, the listbox is offset
             */
            if CB_GETTYPE!(lphc) == CBS_DROPDOWN {
                (*lphc).droppedRect.left += COMBO_EDITBUTTONSPACE!() as LONG;
            }

            if (*lphc).droppedRect.bottom < (*lphc).droppedRect.top {
                (*lphc).droppedRect.bottom = (*lphc).droppedRect.top;
            }
            if (*lphc).droppedRect.right < (*lphc).droppedRect.left {
                (*lphc).droppedRect.right = (*lphc).droppedRect.left;
            }
            MapWindowPoints(
                hwnd,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*lphc).droppedRect) as LPPOINT,
                2,
            );
        }

        /* create listbox popup */

        lbeStyle = (LBS_NOTIFY | LBS_COMBOBOX | WS_BORDER | WS_CLIPSIBLINGS | WS_CHILD) as UINT
            | (style as UINT & (WS_VSCROLL | CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE) as UINT);

        if (*lphc).dwStyle & CBS_SORT != 0 {
            lbeStyle |= LBS_SORT as UINT;
        }
        if (*lphc).dwStyle & CBS_HASSTRINGS != 0 {
            lbeStyle |= LBS_HASSTRINGS as UINT;
        }
        if (*lphc).dwStyle & CBS_NOINTEGRALHEIGHT != 0 {
            lbeStyle |= LBS_NOINTEGRALHEIGHT as UINT;
        }
        if (*lphc).dwStyle & CBS_DISABLENOSCROLL != 0 {
            lbeStyle |= LBS_DISABLENOSCROLL as UINT;
        }

        if CB_GETTYPE!(lphc) == CBS_SIMPLE {
            /* child listbox */
            lbeStyle |= WS_VISIBLE as UINT;

            /*
             * In win 95 look n feel, the listbox in the simple combobox has
             * the WS_EXCLIENTEDGE style instead of the WS_BORDER style.
             */
            lbeStyle &= !(WS_BORDER as UINT);
            lbeExStyle |= WS_EX_CLIENTEDGE as UINT;
        } else {
            lbeExStyle |= (WS_EX_TOPMOST | WS_EX_TOOLWINDOW) as UINT;
        }

        (*lphc).hWndLBox = CreateWindowExW(
            lbeExStyle as DWORD,
            clbName.as_ptr(),
            core::ptr::null_mut(),
            lbeStyle as DWORD,
            (*lphc).droppedRect.left as i32,
            (*lphc).droppedRect.top as i32,
            (*lphc).droppedRect.right as i32 - (*lphc).droppedRect.left as i32,
            (*lphc).droppedRect.bottom as i32 - (*lphc).droppedRect.top as i32,
            hwnd,
            (ID_CB_LISTBOX as usize) as HMENU,
            GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
            lphc as LPVOID,
        );
        if !(*lphc).hWndLBox.is_null() {
            let mut bEdit: BOOL = TRUE;
            lbeStyle = (WS_CHILD | WS_VISIBLE | ES_NOHIDESEL | ES_LEFT | ES_COMBO) as UINT;

            if (*lphc).wState & CBF_EDIT != 0 {
                if (*lphc).dwStyle & CBS_OEMCONVERT != 0 {
                    lbeStyle |= ES_OEMCONVERT as UINT;
                }
                if (*lphc).dwStyle & CBS_AUTOHSCROLL != 0 {
                    lbeStyle |= ES_AUTOHSCROLL as UINT;
                }
                if (*lphc).dwStyle & CBS_LOWERCASE != 0 {
                    lbeStyle |= ES_LOWERCASE as UINT;
                } else if (*lphc).dwStyle & CBS_UPPERCASE != 0 {
                    lbeStyle |= ES_UPPERCASE as UINT;
                }

                if IsWindowEnabled(hwnd) == 0 {
                    lbeStyle |= WS_DISABLED as UINT;
                }

                (*lphc).hWndEdit = CreateWindowExW(
                    0 as DWORD,
                    editName.as_ptr(),
                    core::ptr::null_mut(),
                    lbeStyle as DWORD,
                    (*lphc).textRect.left as i32,
                    (*lphc).textRect.top as i32,
                    (*lphc).textRect.right as i32 - (*lphc).textRect.left as i32,
                    (*lphc).textRect.bottom as i32 - (*lphc).textRect.top as i32,
                    hwnd,
                    (ID_CB_EDIT as usize) as HMENU,
                    GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
                    core::ptr::null_mut(),
                );
                if (*lphc).hWndEdit.is_null() {
                    bEdit = FALSE;
                }
            }

            if bEdit != 0 {
                if CB_GETTYPE!(lphc) != CBS_SIMPLE {
                    /* Now do the trick with parent */
                    SetParent((*lphc).hWndLBox, HWND_DESKTOP);
                    /*
                     * If the combo is a dropdown, we must resize the control
                     * to fit only the text area and button. To do this,
                     * we send a dummy resize and the WM_WINDOWPOSCHANGING message
                     * will take care of setting the height for us.
                     */
                    CBForceDummyResize(lphc);
                }

                return 0 as LRESULT;
            }
        } else {
            if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32) != 0 {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Create\0" as *const u8 as *const ::core::ffi::c_char,
                    546 as ::core::ffi::c_int,
                    b"listbox failure.\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
    } else {
        if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_ERR as i32) != 0 {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/combo.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBO_Create\0" as *const u8 as *const ::core::ffi::c_char,
                548 as ::core::ffi::c_int,
                b"no owner for visible combo.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    /* CreateWindow() will send WM_NCDESTROY to cleanup */

    return -(1) as LRESULT;
}

unsafe extern "C" fn CBPaintButton(lphc: *mut HEADCOMBO, hdc: HDC) {
    let mut buttonState: UINT = DFCS_SCROLLCOMBOBOX as UINT;

    if IsRectEmpty(core::ptr::addr_of_mut!((*lphc).buttonRect) as LPCRECT) != 0 {
        return;
    }

    if (*lphc).wState as UINT & CBF_NOREDRAW as UINT != 0 {
        return;
    }

    if (*lphc).wState as UINT & CBF_BUTTONDOWN as UINT != 0 {
        buttonState |= DFCS_PUSHED as UINT;
    }

    if CB_DISABLED!(lphc) != 0 {
        buttonState |= DFCS_INACTIVE as UINT;
    }

    DrawFrameControl(
        hdc,
        core::ptr::addr_of_mut!((*lphc).buttonRect),
        DFC_SCROLL as UINT,
        buttonState,
    );
}

unsafe extern "C" fn COMBO_PrepareColors(lphc: LPHEADCOMBO, hDC: HDC) -> HBRUSH {
    let mut hBkgBrush: HBRUSH = core::ptr::null_mut();

    /*
     * Get the background brush for this control.
     */
    if IsWindowEnabled((*lphc).self_) == 0 {
        hBkgBrush = SendMessageW((*lphc).owner, WM_CTLCOLORSTATIC as UINT,
                hDC as WPARAM, (*lphc).self_ as LPARAM) as HBRUSH;

        /*
         * We have to change the text color since WM_CTLCOLORSTATIC will
         * set it to the "enabled" color. This is the same behavior as the
         * edit control
         */
        SetTextColor(hDC, GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF);
    } else {
        /* FIXME: In which cases WM_CTLCOLORLISTBOX should be sent? */
        hBkgBrush = SendMessageW((*lphc).owner, WM_CTLCOLOREDIT as UINT,
                hDC as WPARAM, (*lphc).self_ as LPARAM) as HBRUSH;
    }

    /*
     * Catch errors.
     */
    if hBkgBrush.is_null() {
        hBkgBrush = GetSysColorBrush((COLOR_WINDOW) as i32);
    }

    return hBkgBrush;
}

unsafe extern "C" fn CBPaintText(lphc: *mut HEADCOMBO, hdc_paint: HDC) {
    let mut rectEdit: RECT = (*lphc).textRect;
    let mut id: INT = 0;
    let mut size: INT = 0 as INT;
    let mut pText: LPWSTR = core::ptr::null_mut::<WCHAR>();

    /* follow Windows combobox that sends a bunch of text
     * inquiries to its listbox while processing WM_PAINT. */

    /* TRACE(...) entry log omitted: debug-channel macro, no runtime effect */

    id = SendMessageW(
        (*lphc).hWndLBox,
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if id != (LB_ERR as INT) {
        size = SendMessageW(
            (*lphc).hWndLBox,
            LB_GETTEXTLEN as UINT,
            id as WPARAM,
            0 as LPARAM,
        ) as INT;
        if size == (LB_ERR as INT) {
            /* FIXME("LB_ERR probably not handled yet\n") */
        }
        pText = heap_alloc(
            ((size + (1 as INT)) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !pText.is_null() {
            /* size from LB_GETTEXTLEN may be too large, from LB_GETTEXT is accurate */
            size = SendMessageW(
                (*lphc).hWndLBox,
                LB_GETTEXT as UINT,
                id as WPARAM,
                pText as LPARAM,
            ) as INT;
            *pText.offset(size as isize) = '\0' as i32 as WCHAR; /* just in case */
        } else {
            return;
        }
    }

    if ((*lphc).wState & (CBF_EDIT as UINT)) != 0 {
        static EMPTY_STRING_W: [WCHAR; 1] = [0 as WCHAR];
        if CB_HASSTRINGS!(lphc) != 0 {
            SetWindowTextW(
                (*lphc).hWndEdit,
                if !pText.is_null() {
                    pText as LPCWSTR
                } else {
                    core::ptr::addr_of!(EMPTY_STRING_W) as LPCWSTR
                },
            );
        }
        if ((*lphc).wState & (CBF_FOCUSED as UINT)) != 0 {
            SendMessageW(
                (*lphc).hWndEdit,
                EM_SETSEL as UINT,
                0 as WPARAM,
                MAXLONG as LPARAM,
            );
        }
    } else if ((*lphc).wState & (CBF_NOREDRAW as UINT)) == 0
        && IsWindowVisible((*lphc).self_) != 0
    {
        /* paint text field ourselves */
        let mut hdc: HDC = if !hdc_paint.is_null() {
            hdc_paint
        } else {
            GetDC((*lphc).self_)
        };
        let mut itemState: UINT = ODS_COMBOBOXEDIT as UINT;
        let mut hPrevFont: HFONT = if !(*lphc).hFont.is_null() {
            SelectObject(hdc, (*lphc).hFont as HGDIOBJ) as HFONT
        } else {
            0 as HFONT
        };
        let mut hPrevBrush: HBRUSH = core::ptr::null_mut::<HBRUSH__>();
        let mut hBkgBrush: HBRUSH = core::ptr::null_mut::<HBRUSH__>();

        /* Give ourselves some space. */
        InflateRect(
            core::ptr::addr_of_mut!(rectEdit),
            -(1 as INT),
            -(1 as INT),
        );

        hBkgBrush = COMBO_PrepareColors(lphc, hdc);
        hPrevBrush = SelectObject(hdc, hBkgBrush as HGDIOBJ) as HBRUSH;
        FillRect(hdc, core::ptr::addr_of_mut!(rectEdit) as LPCRECT, hBkgBrush);

        if CB_OWNERDRAWN!(lphc) != 0 {
            let mut dis: DRAWITEMSTRUCT = tagDRAWITEMSTRUCT {
                CtlType: 0,
                CtlID: 0,
                itemID: 0,
                itemAction: 0,
                itemState: 0,
                hwndItem: core::ptr::null_mut::<HWND__>(),
                hDC: core::ptr::null_mut::<HDC__>(),
                rcItem: tagRECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                itemData: 0,
            };
            let mut clipRegion: HRGN = core::ptr::null_mut::<HRGN__>();
            let ctlid: UINT = GetWindowLongPtrW((*lphc).self_, GWLP_ID as INT) as UINT;

            /* setup state for DRAWITEM message. Owner will highlight */
            if ((*lphc).wState & (CBF_FOCUSED as UINT)) != 0
                && ((*lphc).wState & (CBF_DROPPED as UINT)) == 0
            {
                itemState |= (ODS_SELECTED | ODS_FOCUS) as UINT;
            }

            if IsWindowEnabled((*lphc).self_) == 0 {
                itemState |= ODS_DISABLED as UINT;
            }

            dis.CtlType = ODT_COMBOBOX as UINT;
            dis.CtlID = ctlid;
            dis.hwndItem = (*lphc).self_;
            dis.itemAction = ODA_DRAWENTIRE as UINT;
            dis.itemID = id as UINT;
            dis.itemState = itemState;
            dis.hDC = hdc;
            dis.rcItem = rectEdit;
            dis.itemData = SendMessageW(
                (*lphc).hWndLBox,
                LB_GETITEMDATA as UINT,
                id as WPARAM,
                0 as LPARAM,
            ) as ULONG_PTR;

            /* Clip the DC and have the parent draw the item. */
            clipRegion = set_control_clipping(hdc, core::ptr::addr_of_mut!(rectEdit));

            SendMessageW(
                (*lphc).owner,
                WM_DRAWITEM as UINT,
                ctlid as WPARAM,
                core::ptr::addr_of_mut!(dis) as LPARAM,
            );

            SelectClipRgn(hdc, clipRegion);
            if !clipRegion.is_null() {
                DeleteObject(clipRegion as HGDIOBJ);
            }
        } else {
            static EMPTY_STRING_W_0: [WCHAR; 1] = [0 as WCHAR];

            if ((*lphc).wState & (CBF_FOCUSED as UINT)) != 0
                && ((*lphc).wState & (CBF_DROPPED as UINT)) == 0
            {
                /* highlight */
                FillRect(
                    hdc,
                    core::ptr::addr_of_mut!(rectEdit) as LPCRECT,
                    GetSysColorBrush(COLOR_HIGHLIGHT as INT),
                );
                SetBkColor(
                    hdc,
                    GetSysColor(COLOR_HIGHLIGHT as INT) as COLORREF,
                );
                SetTextColor(
                    hdc,
                    GetSysColor(COLOR_HIGHLIGHTTEXT as INT) as COLORREF,
                );
            }

            ExtTextOutW(
                hdc,
                rectEdit.left as INT + (1 as INT),
                rectEdit.top as INT + (1 as INT),
                (ETO_OPAQUE | ETO_CLIPPED) as UINT,
                core::ptr::addr_of_mut!(rectEdit),
                if !pText.is_null() {
                    pText as LPCWSTR
                } else {
                    core::ptr::addr_of!(EMPTY_STRING_W_0) as LPCWSTR
                },
                size as UINT,
                core::ptr::null::<INT>(),
            );

            if ((*lphc).wState & (CBF_FOCUSED as UINT)) != 0
                && ((*lphc).wState & (CBF_DROPPED as UINT)) == 0
            {
                DrawFocusRect(hdc, core::ptr::addr_of_mut!(rectEdit) as LPCRECT);
            }
        }

        if !hPrevFont.is_null() {
            SelectObject(hdc, hPrevFont as HGDIOBJ);
        }

        if !hPrevBrush.is_null() {
            SelectObject(hdc, hPrevBrush as HGDIOBJ);
        }

        if hdc_paint.is_null() {
            ReleaseDC((*lphc).self_, hdc);
        }
    }

    heap_free(pText as *mut core::ffi::c_void);
}

unsafe extern "C" fn CBPaintBorder(lphc: *const HEADCOMBO, hdc: HDC) {
    let mut clientRect: RECT = core::mem::zeroed();

    if CB_GETTYPE!(lphc) != CBS_SIMPLE {
        GetClientRect((*lphc).self_, core::ptr::addr_of_mut!(clientRect));
    } else {
        clientRect = (*lphc).textRect;

        InflateRect(core::ptr::addr_of_mut!(clientRect), EDIT_CONTROL_PADDING!(), EDIT_CONTROL_PADDING!());
        InflateRect(core::ptr::addr_of_mut!(clientRect), COMBO_XBORDERSIZE!(), COMBO_YBORDERSIZE!());
    }

    DrawEdge(hdc, core::ptr::addr_of_mut!(clientRect), EDGE_SUNKEN, BF_RECT);
}

unsafe extern "C" fn COMBO_ThemedPaint(theme: HTHEME, lphc: *mut HEADCOMBO, hdc: HDC) -> LRESULT {
    let mut button_state: ::core::ffi::c_int = 0;
    let mut frame: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    /* paint border */
    if CB_GETTYPE!(lphc) != CBS_SIMPLE {
        GetClientRect((*lphc).self_, &raw mut frame);
    } else {
        frame = (*lphc).textRect;
        InflateRect(&raw mut frame, EDIT_CONTROL_PADDING!(), EDIT_CONTROL_PADDING!());
        InflateRect(&raw mut frame, COMBO_XBORDERSIZE!(), COMBO_YBORDERSIZE!());
    }

    DrawThemeBackground(
        theme,
        hdc,
        0,
        if IsWindowEnabled((*lphc).self_) != 0 {
            (CBXS_NORMAL) as i32
        } else {
            (CBXS_DISABLED) as i32
        },
        &raw mut frame,
        core::ptr::null::<RECT>(),
    );

    /* Paint button */
    if IsRectEmpty(&raw mut (*lphc).buttonRect as LPCRECT) == 0 {
        if IsWindowEnabled((*lphc).self_) == 0 {
            button_state = (CBXS_DISABLED) as i32;
        } else if (*lphc).wState & CBF_BUTTONDOWN != 0 {
            button_state = (CBXS_PRESSED) as i32;
        } else if (*lphc).wState & CBF_HOT != 0 {
            button_state = (CBXS_HOT) as i32;
        } else {
            button_state = (CBXS_NORMAL) as i32;
        }
        DrawThemeBackground(
            theme,
            hdc,
            (CP_DROPDOWNBUTTON) as i32,
            button_state,
            &raw mut (*lphc).buttonRect,
            core::ptr::null::<RECT>(),
        );
    }

    if (*lphc).dwStyle & CBS_DROPDOWNLIST == CBS_DROPDOWNLIST {
        CBPaintText(lphc, hdc);
    }

    return 0;
}

unsafe extern "C" fn COMBO_Paint(lphc: *mut HEADCOMBO, hdc: HDC) -> LRESULT {
    let mut hPrevBrush: HBRUSH;
    let mut hBkgBrush: HBRUSH;

    /*
     * Retrieve the background brush and select it in the
     * DC.
     */
    hBkgBrush = COMBO_PrepareColors(lphc, hdc);
    hPrevBrush = SelectObject(hdc, hBkgBrush as HGDIOBJ) as HBRUSH;
    if ((*lphc).wState & CBF_EDIT) == 0 {
        FillRect(hdc, core::ptr::addr_of_mut!((*lphc).textRect) as LPCRECT, hBkgBrush);
    }

    /*
     * In non 3.1 look, there is a sunken border on the combobox
     */
    CBPaintBorder(lphc, hdc);

    CBPaintButton(lphc, hdc);

    /* paint the edit control padding area */
    if CB_GETTYPE!(lphc) != CBS_DROPDOWNLIST {
        let mut rPadEdit: RECT = (*lphc).textRect;

        InflateRect(
            core::ptr::addr_of_mut!(rPadEdit),
            EDIT_CONTROL_PADDING!(),
            EDIT_CONTROL_PADDING!(),
        );

        FrameRect(
            hdc,
            core::ptr::addr_of_mut!(rPadEdit) as LPCRECT,
            GetSysColorBrush((COLOR_WINDOW) as i32),
        );
    }

    if ((*lphc).wState & CBF_EDIT) == 0 {
        CBPaintText(lphc, hdc);
    }

    if !hPrevBrush.is_null() {
        SelectObject(hdc, hPrevBrush as HGDIOBJ);
    }

    return 0;
}

unsafe extern "C" fn CBUpdateLBox(mut lphc: LPHEADCOMBO, mut bSelect: BOOL) -> INT {
    let mut length: INT = 0;
    let mut idx: INT = 0;
    let mut pText: LPWSTR = core::ptr::null_mut();

    idx = LB_ERR;
    length = SendMessageW(
        (*lphc).hWndEdit,
        WM_GETTEXTLENGTH,
        0,
        0,
    ) as INT;

    if length > 0 {
        pText = heap_alloc(
            ((length + 1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
    }

    if !pText.is_null() {
        GetWindowTextW(
            (*lphc).hWndEdit,
            pText,
            length + 1,
        );
        idx = SendMessageW(
            (*lphc).hWndLBox,
            LB_FINDSTRING,
            (-(1i32)) as WPARAM,
            pText as LPARAM,
        ) as INT;
        heap_free(pText as *mut core::ffi::c_void);
    }

    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETCURSEL,
        (if bSelect != 0 {
            idx
        } else {
            -(1)
        }) as WPARAM,
        0,
    );

    /* probably superfluous but Windows sends this too */
    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETCARETINDEX,
        (if idx < 0 {
            0
        } else {
            idx
        }) as WPARAM,
        0,
    );
    SendMessageW(
        (*lphc).hWndLBox,
        LB_SETTOPINDEX,
        (if idx < 0 {
            0
        } else {
            idx
        }) as WPARAM,
        0,
    );

    return idx;
}

unsafe extern "C" fn CBUpdateEdit(mut lphc: LPHEADCOMBO, mut index: INT) {
    let mut length: INT = 0;
    let mut pText: LPWSTR = core::ptr::null_mut::<WCHAR>();
    static mut empty_stringW: [WCHAR; 1] = [0 as WCHAR];
    if index >= 0 as INT {
        length = SendMessageW(
            (*lphc).hWndLBox,
            LB_GETTEXTLEN as UINT,
            index as WPARAM,
            0 as LPARAM,
        ) as INT;
        if length != LB_ERR as INT {
            pText = heap_alloc(
                ((length + 1 as INT) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if !pText.is_null() {
                SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTEXT as UINT,
                    index as WPARAM,
                    pText as LPARAM,
                );
            }
        }
    }
    if CB_HASSTRINGS!(lphc) != 0 {
        (*lphc).wState |= (CBF_NOEDITNOTIFY | CBF_NOLBSELECT) as UINT;
        SendMessageW(
            (*lphc).hWndEdit,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            if !pText.is_null() {
                pText as LPARAM
            } else {
                core::ptr::addr_of!(empty_stringW) as *const WCHAR as LPARAM
            },
        );
        (*lphc).wState &= !((CBF_NOEDITNOTIFY | CBF_NOLBSELECT) as UINT);
    }
    if ((*lphc).wState & (CBF_FOCUSED as UINT)) != 0 {
        SendMessageW(
            (*lphc).hWndEdit,
            EM_SETSEL as UINT,
            0 as WPARAM,
            -(1 as LPARAM),
        );
    }
    heap_free(pText as *mut core::ffi::c_void);
}

unsafe extern "C" fn CBDropDown(lphc: LPHEADCOMBO) {
    let mut monitor: HMONITOR = core::ptr::null_mut();
    let mut mon_info: MONITORINFO = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let mut nItems: ::core::ffi::c_int = 0;
    let mut nDroppedHeight: ::core::ffi::c_int = 0;

    CB_NOTIFY!(lphc, CBN_DROPDOWN);

    /* set selection */

    (*lphc).wState |= CBF_DROPPED as UINT;
    if CB_GETTYPE!(lphc) == CBS_DROPDOWN as UINT {
        (*lphc).droppedIndex = CBUpdateLBox(lphc, TRUE as BOOL);

        /* Update edit only if item is in the list */
        if ((*lphc).wState & CBF_CAPTURE as UINT) == 0
            && (*lphc).droppedIndex >= 0 as INT
        {
            CBUpdateEdit(lphc, (*lphc).droppedIndex);
        }
    } else {
        (*lphc).droppedIndex = SendMessageW(
            (*lphc).hWndLBox,
            LB_GETCURSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;

        SendMessageW(
            (*lphc).hWndLBox,
            LB_SETTOPINDEX as UINT,
            (if (*lphc).droppedIndex == LB_ERR as INT {
                0 as INT
            } else {
                (*lphc).droppedIndex
            }) as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            (*lphc).hWndLBox,
            LB_CARETON as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }

    /* now set popup position */
    GetWindowRect((*lphc).self_, core::ptr::addr_of_mut!(rect));

    /*
     * If it's a dropdown, the listbox is offset
     */
    if CB_GETTYPE!(lphc) == CBS_DROPDOWN as UINT {
        rect.left += COMBO_EDITBUTTONSPACE!();
    }

    /* if the dropped height is greater than the total height of the dropped
       items list, then force the drop down list height to be the total height
       of the items in the dropped list */

    /* And Remove any extra space (Best Fit) */
    nDroppedHeight = ((*lphc).droppedRect.bottom - (*lphc).droppedRect.top) as ::core::ffi::c_int;
    /* if listbox length has been set directly by its handle */
    GetWindowRect((*lphc).hWndLBox, core::ptr::addr_of_mut!(r));
    if nDroppedHeight < r.bottom - r.top {
        nDroppedHeight = (r.bottom - r.top) as ::core::ffi::c_int;
    }
    nItems = SendMessageW(
        (*lphc).hWndLBox,
        LB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;

    if nItems > 0 as ::core::ffi::c_int {
        let mut nIHeight: ::core::ffi::c_int = SendMessageW(
            (*lphc).hWndLBox,
            LB_GETITEMHEIGHT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as ::core::ffi::c_int;

        if ((*lphc).dwStyle & CBS_NOINTEGRALHEIGHT as UINT) != 0 {
            nDroppedHeight = min(
                nItems * nIHeight + COMBO_YBORDERSIZE!(),
                nDroppedHeight - 1 as ::core::ffi::c_int,
            );
        } else {
            if nItems > (*lphc).visibleItems as ::core::ffi::c_int {
                nItems = (*lphc).visibleItems as ::core::ffi::c_int;
            }
            nDroppedHeight = nItems * nIHeight + COMBO_YBORDERSIZE!();
        }
    }

    r.left = rect.left;
    r.top = rect.bottom;
    r.right = r.left + (*lphc).droppedRect.right - (*lphc).droppedRect.left;
    r.bottom = r.top + nDroppedHeight as LONG;

    /*If height of dropped rectangle gets beyond a screen size it should go up, otherwise down.*/
    monitor = MonitorFromRect(
        core::ptr::addr_of_mut!(rect) as LPCRECT,
        MONITOR_DEFAULTTOPRIMARY as DWORD,
    );
    mon_info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(monitor, core::ptr::addr_of_mut!(mon_info));

    if r.bottom > mon_info.rcWork.bottom {
        r.top = max(
            rect.top - nDroppedHeight as LONG,
            mon_info.rcWork.top,
        );
        r.bottom = min(
            r.top + nDroppedHeight as LONG,
            mon_info.rcWork.bottom,
        );
    }

    SetWindowPos(
        (*lphc).hWndLBox,
        HWND_TOPMOST as HWND,
        r.left,
        r.top,
        r.right - r.left,
        r.bottom - r.top,
        (SWP_NOACTIVATE | SWP_SHOWWINDOW) as UINT,
    );

    if ((*lphc).wState & CBF_NOREDRAW as UINT) == 0 {
        RedrawWindow(
            (*lphc).self_,
            core::ptr::null::<RECT>(),
            0 as HRGN,
            (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW) as UINT,
        );
    }

    EnableWindow((*lphc).hWndLBox, TRUE as BOOL);
    if GetCapture() != (*lphc).self_ {
        SetCapture((*lphc).hWndLBox);
    }
}

unsafe extern "C" fn CBRollUp(lphc: LPHEADCOMBO, ok: BOOL, mut bButton: BOOL) {
    let hWnd: HWND = (*lphc).self_;

    CB_NOTIFY!(
        lphc,
        if ok != 0 { CBN_SELENDOK } else { CBN_SELENDCANCEL },
    );

    if IsWindow(hWnd) != 0 && CB_GETTYPE!(lphc) != CBS_SIMPLE {
        if ((*lphc).wState & CBF_DROPPED) != 0 {
            let mut rect: RECT = core::mem::zeroed::<RECT>();

            (*lphc).wState &= !(CBF_DROPPED);
            ShowWindow((*lphc).hWndLBox, (SW_HIDE) as i32);

            if GetCapture() == (*lphc).hWndLBox {
                ReleaseCapture();
            }

            if CB_GETTYPE!(lphc) == CBS_DROPDOWN {
                rect = (*lphc).buttonRect;
            } else {
                if bButton != 0 {
                    UnionRect(
                        core::ptr::addr_of_mut!(rect),
                        core::ptr::addr_of_mut!((*lphc).buttonRect),
                        core::ptr::addr_of_mut!((*lphc).textRect),
                    );
                } else {
                    rect = (*lphc).textRect;
                }

                bButton = TRUE;
            }

            if bButton != 0 && ((*lphc).wState & CBF_NOREDRAW) == 0 {
                RedrawWindow(
                    hWnd,
                    core::ptr::addr_of_mut!(rect),
                    core::ptr::null_mut(),
                    RDW_INVALIDATE
                        | RDW_ERASE
                        | RDW_UPDATENOW
                        | RDW_NOCHILDREN,
                );
            }
            CB_NOTIFY!(lphc, CBN_CLOSEUP);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn COMBO_FlipListbox(lphc: *mut HEADCOMBO, ok: BOOL, bRedrawButton: BOOL) -> BOOL {
    if (*lphc).wState & CBF_DROPPED != 0 {
        CBRollUp(lphc, ok, bRedrawButton);
        return FALSE;
    }

    CBDropDown(lphc);
    return TRUE;
}

unsafe extern "C" fn CBRepaintButton(lphc: LPHEADCOMBO) {
    InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).buttonRect) as LPCRECT, TRUE);
    UpdateWindow((*lphc).self_);
}

unsafe extern "C" fn COMBO_SetFocus(lphc: LPHEADCOMBO) {
    if !((*lphc).wState & (CBF_FOCUSED as UINT) != 0) {
        if CB_GETTYPE!(lphc) == CBS_DROPDOWNLIST {
            SendMessageW((*lphc).hWndLBox, LB_CARETON as UINT, 0 as WPARAM, 0 as LPARAM);
        }

        /* This is wrong. Message sequences seem to indicate that this
           is set *after* the notify. */
        /* (*lphc).wState |= CBF_FOCUSED;  */

        if !((*lphc).wState & (CBF_EDIT as UINT) != 0) {
            InvalidateRect((*lphc).self_, core::ptr::addr_of_mut!((*lphc).textRect) as LPCRECT, TRUE);
        }

        CB_NOTIFY!(lphc, CBN_SETFOCUS);
        (*lphc).wState |= CBF_FOCUSED as UINT;
    }
}

unsafe extern "C" fn COMBO_KillFocus(mut lphc: LPHEADCOMBO) {
    let mut hWnd: HWND = (*lphc).self_;
    if ((*lphc).wState & (CBF_FOCUSED) as UINT) != 0 {
        CBRollUp(lphc, FALSE, TRUE);
        if IsWindow(hWnd) != 0 {
            if CB_GETTYPE!(lphc) == CBS_DROPDOWNLIST {
                SendMessageW(
                    (*lphc).hWndLBox,
                    LB_CARETOFF,
                    0,
                    0,
                );
            }
            (*lphc).wState &= !((CBF_FOCUSED) as UINT);
            if ((*lphc).wState & (CBF_EDIT) as UINT) == 0 {
                InvalidateRect(
                    (*lphc).self_,
                    &raw mut (*lphc).textRect as LPCRECT,
                    TRUE,
                );
            }
            SendMessageW(
                (*lphc).owner,
                WM_COMMAND as UINT,
                ((GetWindowLongPtrW((*lphc).self_, -(12 as ::core::ffi::c_int)) as DWORD_PTR
                    & 0xffff as DWORD_PTR) as WORD as DWORD
                    | ((CBN_KILLFOCUS as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                        as DWORD)
                        << 16 as ::core::ffi::c_int) as LONG as DWORD as WPARAM,
                (*lphc).self_ as LPARAM,
            );
        }
    }
}

unsafe extern "C" fn COMBO_Command(
    mut lphc: LPHEADCOMBO,
    mut wParam: WPARAM,
    mut hWnd: HWND,
) -> LRESULT {
    if (*lphc).wState & (CBF_EDIT as UINT) != 0 && (*lphc).hWndEdit == hWnd {
        /* ">> 8" makes gcc generate jump-table instead of cmp ladder */
        let sw: u32 = (HIWORD!(wParam) as u32) >> 8;

        if sw == (EN_SETFOCUS >> 8) {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Command\0" as *const u8 as *const ::core::ffi::c_char,
                    1168 as ::core::ffi::c_int,
                    b"[%p]: edit [%p] got focus\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*lphc).self_,
                    (*lphc).hWndEdit,
                );
            }
            COMBO_SetFocus(lphc);
        } else if sw == (EN_KILLFOCUS >> 8) {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Command\0" as *const u8 as *const ::core::ffi::c_char,
                    1175 as ::core::ffi::c_int,
                    b"[%p]: edit [%p] lost focus\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*lphc).self_,
                    (*lphc).hWndEdit,
                );
            }
            /* NOTE: it seems that Windows' edit control sends an
             * undocumented message WM_USER + 0x1B instead of this
             * notification (only when it happens to be a part of
             * the combo). ?? - AK.
             */
            COMBO_KillFocus(lphc);
        } else if sw == (EN_CHANGE >> 8) {
            /*
             * In some circumstances (when the selection of the combobox
             * is changed for example) we don't want the EN_CHANGE notification
             * to be forwarded to the parent of the combobox. This code
             * checks a flag that is set in these occasions and ignores the
             * notification.
             */
            if (*lphc).wState & (CBF_NOLBSELECT as UINT) != 0 {
                (*lphc).wState &= !(CBF_NOLBSELECT as UINT);
            } else {
                CBUpdateLBox(
                    lphc,
                    ((*lphc).wState & (CBF_DROPPED as UINT)) as BOOL,
                );
            }

            if (*lphc).wState & (CBF_NOEDITNOTIFY as UINT) == 0 {
                CB_NOTIFY!(lphc, CBN_EDITCHANGE);
            }
        } else if sw == (EN_UPDATE >> 8) {
            if (*lphc).wState & (CBF_NOEDITNOTIFY as UINT) == 0 {
                CB_NOTIFY!(lphc, CBN_EDITUPDATE);
            }
        } else if sw == (EN_ERRSPACE >> 8) {
            CB_NOTIFY!(lphc, CBN_ERRSPACE);
        }
    } else if (*lphc).hWndLBox == hWnd {
        let sw2: i32 = (HIWORD!(wParam) as u16 as i16) as i32;

        if sw2 == LBN_ERRSPACE {
            CB_NOTIFY!(lphc, CBN_ERRSPACE);
        } else if (sw2) as u32 == LBN_DBLCLK {
            CB_NOTIFY!(lphc, CBN_DBLCLK);
        } else if (sw2) as u32 == LBN_SELCHANGE || (sw2) as u32 == LBN_SELCANCEL {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Command\0" as *const u8 as *const ::core::ffi::c_char,
                    1232 as ::core::ffi::c_int,
                    b"[%p]: lbox selection change [%x]\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*lphc).self_,
                    (*lphc).wState,
                );
            }

            /* do not roll up if selection is being tracked
             * by arrow keys in the dropdown listbox */
            if (*lphc).wState & (CBF_NOROLLUP as UINT) == 0 {
                CBRollUp(
                    lphc,
                    ((HIWORD!(wParam)) as u32 == LBN_SELCHANGE) as BOOL,
                    TRUE as BOOL,
                );
            } else {
                (*lphc).wState &= !(CBF_NOROLLUP as UINT);
            }

            CB_NOTIFY!(lphc, CBN_SELCHANGE);

            if (HIWORD!(wParam)) as u32 == LBN_SELCHANGE {
                if (*lphc).wState & (CBF_EDIT as UINT) != 0 {
                    (*lphc).wState |= CBF_NOLBSELECT as UINT;
                }
                CBPaintText(lphc, core::ptr::null_mut());
            }
        } else if (sw2) as u32 == LBN_SETFOCUS || (sw2) as u32 == LBN_KILLFOCUS {
            /* nothing to do here since ComboLBox always resets the focus to its
             * combo/edit counterpart */
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn COMBO_ItemOp(
    mut lphc: LPHEADCOMBO,
    mut msg: UINT,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut hWnd: HWND = (*lphc).self_;
    let mut id: UINT = GetWindowLongPtrW(hWnd, GWLP_ID) as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/combo.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBO_ItemOp\0" as *const u8 as *const ::core::ffi::c_char,
            1272 as ::core::ffi::c_int,
            b"[%p]: ownerdraw op %04x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lphc).self_,
            msg,
        );
    }
    match msg {
        WM_DELETEITEM => {
            let mut lpIS: *mut DELETEITEMSTRUCT = lParam as *mut DELETEITEMSTRUCT;
            (*lpIS).CtlType = ODT_COMBOBOX as UINT;
            (*lpIS).CtlID = id;
            (*lpIS).hwndItem = hWnd;
        }
        WM_DRAWITEM => {
            let mut lpIS_0: *mut DRAWITEMSTRUCT = lParam as *mut DRAWITEMSTRUCT;
            (*lpIS_0).CtlType = ODT_COMBOBOX as UINT;
            (*lpIS_0).CtlID = id;
            (*lpIS_0).hwndItem = hWnd;
        }
        WM_COMPAREITEM => {
            let mut lpIS_1: *mut COMPAREITEMSTRUCT = lParam as *mut COMPAREITEMSTRUCT;
            (*lpIS_1).CtlType = ODT_COMBOBOX as UINT;
            (*lpIS_1).CtlID = id;
            (*lpIS_1).hwndItem = hWnd;
        }
        WM_MEASUREITEM => {
            let mut lpIS_2: *mut MEASUREITEMSTRUCT = lParam as *mut MEASUREITEMSTRUCT;
            (*lpIS_2).CtlType = ODT_COMBOBOX as UINT;
            (*lpIS_2).CtlID = id;
        }
        _ => {}
    }
    return SendMessageW((*lphc).owner, msg, id as WPARAM, lParam);
}

unsafe extern "C" fn COMBO_GetText(lphc: *mut HEADCOMBO, count: INT, buf: LPWSTR) -> LRESULT {
    let mut length: INT;

    if (*lphc).wState & CBF_EDIT as UINT != 0 {
        return SendMessageW(
            (*lphc).hWndEdit,
            WM_GETTEXT as UINT,
            count as WPARAM,
            buf as LPARAM,
        );
    }

    /* get it from the listbox */

    if count == 0 || buf.is_null() {
        return 0 as LRESULT;
    }
    if !(*lphc).hWndLBox.is_null() {
        'error: {
            let idx: INT = SendMessageW(
                (*lphc).hWndLBox,
                LB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            if idx == LB_ERR {
                break 'error;
            }
            length = SendMessageW(
                (*lphc).hWndLBox,
                LB_GETTEXTLEN as UINT,
                idx as WPARAM,
                0 as LPARAM,
            ) as INT;
            if length == LB_ERR {
                break 'error;
            }

            /* 'length' is without the terminating character */
            if length >= count {
                let lpBuffer: *mut WCHAR = heap_alloc(
                    ((length + 1) as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                if lpBuffer.is_null() {
                    break 'error;
                }
                length = SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTEXT as UINT,
                    idx as WPARAM,
                    lpBuffer as LPARAM,
                ) as INT;

                /* truncate if buffer is too short */
                if length != LB_ERR {
                    lstrcpynW(buf, lpBuffer as LPCWSTR, count as ::core::ffi::c_int);
                    length = count;
                }
                heap_free(lpBuffer as *mut ::core::ffi::c_void);
            } else {
                length = SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTEXT as UINT,
                    idx as WPARAM,
                    buf as LPARAM,
                ) as INT;
            }

            if length == LB_ERR {
                return 0 as LRESULT;
            }
            return length as LRESULT;
        }
    }

    /* error - truncate string, return zero */
    *buf.offset(0 as isize) = 0 as WCHAR;
    return 0 as LRESULT;
}

unsafe extern "C" fn CBResetPos(combo: *mut HEADCOMBO) {
    let drop: BOOL = (CB_GETTYPE!(combo) != CBS_SIMPLE) as BOOL;

    /* NOTE: logs sometimes have WM_LBUTTONUP before a cascade of
     * sizing messages */
    if ((*combo).wState & (CBF_EDIT as UINT)) != 0 {
        SetWindowPos(
            (*combo).hWndEdit,
            core::ptr::null_mut(),
            (*combo).textRect.left as i32,
            (*combo).textRect.top as i32,
            (*combo).textRect.right as i32 - (*combo).textRect.left as i32,
            (*combo).textRect.bottom as i32 - (*combo).textRect.top as i32,
            ((SWP_NOZORDER | SWP_NOACTIVATE)
                | (if drop != 0 { SWP_NOREDRAW } else { 0 })) as UINT,
        );
    }

    SetWindowPos(
        (*combo).hWndLBox,
        core::ptr::null_mut(),
        (*combo).droppedRect.left as i32,
        (*combo).droppedRect.top as i32,
        (*combo).droppedRect.right as i32 - (*combo).droppedRect.left as i32,
        (*combo).droppedRect.bottom as i32 - (*combo).droppedRect.top as i32,
        ((SWP_NOACTIVATE | SWP_NOZORDER)
            | (if drop != 0 { SWP_NOREDRAW } else { 0 })) as UINT,
    );

    if drop != 0 {
        if ((*combo).wState & (CBF_DROPPED as UINT)) != 0 {
            (*combo).wState &= !(CBF_DROPPED as UINT);
            ShowWindow((*combo).hWndLBox, (SW_HIDE) as i32);
        }

        if ((*combo).wState & (CBF_NOREDRAW as UINT)) == 0 {
            RedrawWindow(
                (*combo).self_,
                core::ptr::null(),
                core::ptr::null_mut(),
                (RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW) as UINT,
            );
        }
    }
}

unsafe extern "C" fn COMBO_Size(lphc: *mut HEADCOMBO) {
    if (*lphc).hWndLBox.is_null()
        || (*lphc).wState & CBF_NORESIZE as UINT != 0
    {
        return;
    }
    if CB_GETTYPE!(lphc) != CBS_SIMPLE as UINT {
        let mut newComboHeight: ::core::ffi::c_int = 0;
        let mut curComboHeight: ::core::ffi::c_int = 0;
        let mut curComboWidth: ::core::ffi::c_int = 0;
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect((*lphc).self_, core::ptr::addr_of_mut!(rc));
        curComboHeight = (rc.bottom - rc.top) as ::core::ffi::c_int;
        curComboWidth = (rc.right - rc.left) as ::core::ffi::c_int;
        newComboHeight = CBGetTextAreaHeight(lphc, TRUE)
            + 2 * COMBO_YBORDERSIZE!();
        if curComboHeight > newComboHeight {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/combo.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBO_Size\0" as *const u8 as *const ::core::ffi::c_char,
                    1429 as ::core::ffi::c_int,
                    b"oldComboHeight=%d, newComboHeight=%d, oldDropBottom=%d, oldDropTop=%d\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    curComboHeight,
                    newComboHeight,
                    (*lphc).droppedRect.bottom,
                    (*lphc).droppedRect.top,
                );
            }
            (*lphc).droppedRect.bottom = ((*lphc).droppedRect.top as ::core::ffi::c_int
                + curComboHeight
                - newComboHeight) as LONG;
        }
        if curComboHeight != newComboHeight {
            (*lphc).wState |= CBF_NORESIZE as UINT;
            SetWindowPos(
                (*lphc).self_,
                core::ptr::null_mut(),
                0,
                0,
                curComboWidth,
                newComboHeight,
                (SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOREDRAW) as UINT,
            );
            (*lphc).wState &= !(CBF_NORESIZE as ::core::ffi::c_int) as UINT;
        }
    }
    CBCalcPlacement(lphc);
    CBResetPos(lphc);
}

unsafe extern "C" fn COMBO_Font(mut lphc: LPHEADCOMBO, mut hFont: HFONT, mut bRedraw: BOOL) {
    (*lphc).hFont = hFont;
    (*lphc).item_height = combo_get_text_height(lphc as *const HEADCOMBO);

    /*
     * Propagate to owned windows.
     */
    if ((*lphc).wState & CBF_EDIT) != 0 {
        SendMessageW(
            (*lphc).hWndEdit,
            WM_SETFONT,
            hFont as WPARAM,
            bRedraw as LPARAM,
        );
    }
    SendMessageW(
        (*lphc).hWndLBox,
        WM_SETFONT,
        hFont as WPARAM,
        bRedraw as LPARAM,
    );

    /*
     * Redo the layout of the control.
     */
    if CB_GETTYPE!(lphc) == CBS_SIMPLE {
        CBCalcPlacement(lphc);

        CBResetPos(lphc);
    } else {
        CBForceDummyResize(lphc);
    }
}

unsafe extern "C" fn COMBO_SetItemHeight(lphc: LPHEADCOMBO, index: INT, height: INT) -> LRESULT {
    let mut lRet: LRESULT = (CB_ERR) as _;

    if index == -1 {
        if height < 32768 {
            (*lphc).item_height = height + 2;

            if CB_GETTYPE!(lphc) == CBS_SIMPLE {
                CBCalcPlacement(lphc as *mut HEADCOMBO);

                CBResetPos(lphc as *mut HEADCOMBO);
            } else {
                CBForceDummyResize(lphc);
            }

            lRet = height as LRESULT;
        }
    } else if CB_OWNERDRAWN!(lphc) != 0 {
        lRet = SendMessageW(
            (*lphc).hWndLBox,
            LB_SETITEMHEIGHT,
            index as WPARAM,
            height as LPARAM,
        );
    }
    return lRet;
}

unsafe extern "C" fn COMBO_SelectString(
    mut lphc: LPHEADCOMBO,
    mut start: INT,
    mut pText: LPARAM,
) -> LRESULT {
    let mut index: INT = SendMessageW(
        (*lphc).hWndLBox,
        LB_SELECTSTRING as UINT,
        start as WPARAM,
        pText,
    ) as INT;
    if index >= 0 as INT {
        if (*lphc).wState & CBF_EDIT as UINT != 0 {
            CBUpdateEdit(lphc, index);
        } else {
            InvalidateRect(
                (*lphc).self_,
                core::ptr::addr_of_mut!((*lphc).textRect) as LPCRECT,
                TRUE as BOOL,
            );
        }
    }
    return index as LRESULT;
}

unsafe extern "C" fn COMBO_LButtonDown(lphc: LPHEADCOMBO, lParam: LPARAM) {
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };
    let mut bButton: BOOL = 0;
    let mut hWnd: HWND = (*lphc).self_;
    pt.x = LOWORD!(lParam) as ::core::ffi::c_short as LONG;
    pt.y = HIWORD!(lParam) as ::core::ffi::c_short as LONG;
    bButton = PtInRect(core::ptr::addr_of_mut!((*lphc).buttonRect) as LPCRECT, pt);

    if (CB_GETTYPE!(lphc) == (CBS_DROPDOWNLIST as UINT))
        || (bButton != 0 && (CB_GETTYPE!(lphc) == (CBS_DROPDOWN as UINT)))
    {
        (*lphc).wState |= (CBF_BUTTONDOWN as UINT);
        if ((*lphc).wState & (CBF_DROPPED as UINT)) != 0 {
            /* got a click to cancel selection */

            (*lphc).wState &= !(CBF_BUTTONDOWN as UINT);
            CBRollUp(lphc, TRUE as BOOL, FALSE as BOOL);
            if IsWindow(hWnd) == 0 {
                return;
            }

            if ((*lphc).wState & (CBF_CAPTURE as UINT)) != 0 {
                (*lphc).wState &= !(CBF_CAPTURE as UINT);
                ReleaseCapture();
            }
        } else {
            /* drop down the listbox and start tracking */

            (*lphc).wState |= (CBF_CAPTURE as UINT);
            SetCapture(hWnd);
            CBDropDown(lphc);
        }
        if bButton != 0 {
            CBRepaintButton(lphc);
        }
    }
}

unsafe extern "C" fn COMBO_LButtonUp(lphc: LPHEADCOMBO) {
    if (*lphc).wState & CBF_CAPTURE as ::core::ffi::c_int as UINT != 0 {
        (*lphc).wState &= !(CBF_CAPTURE as ::core::ffi::c_int) as UINT;
        if CB_GETTYPE!(lphc) == CBS_DROPDOWN as ::core::ffi::c_int as UINT {
            let mut index: INT = CBUpdateLBox(lphc, TRUE as ::core::ffi::c_int);
            if index >= 0 as ::core::ffi::c_int {
                (*lphc).wState |= CBF_NOLBSELECT as ::core::ffi::c_int as UINT;
                CBUpdateEdit(lphc, index);
                (*lphc).wState &= !(CBF_NOLBSELECT as ::core::ffi::c_int) as UINT;
            }
        }
        ReleaseCapture();
        SetCapture((*lphc).hWndLBox);
    }

    if (*lphc).wState & CBF_BUTTONDOWN as ::core::ffi::c_int as UINT != 0 {
        (*lphc).wState &= !(CBF_BUTTONDOWN as ::core::ffi::c_int) as UINT;
        CBRepaintButton(lphc);
    }
}

unsafe extern "C" fn COMBO_MouseMove(mut lphc: LPHEADCOMBO, mut wParam: WPARAM, mut lParam: LPARAM) {
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };
    let mut lbRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    pt.x = LOWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;
    pt.y = HIWORD!(lParam as DWORD) as ::core::ffi::c_short as LONG;

    if (*lphc).wState & CBF_BUTTONDOWN as UINT != 0 {
        let mut bButton: BOOL = 0;

        bButton = PtInRect(core::ptr::addr_of_mut!((*lphc).buttonRect) as LPCRECT, pt);

        if bButton == 0 {
            (*lphc).wState &= !(CBF_BUTTONDOWN as UINT) as UINT;
            CBRepaintButton(lphc);
        }
    }

    GetClientRect((*lphc).hWndLBox, core::ptr::addr_of_mut!(lbRect));
    MapWindowPoints(
        (*lphc).self_,
        (*lphc).hWndLBox,
        core::ptr::addr_of_mut!(pt),
        1 as UINT,
    );
    if PtInRect(core::ptr::addr_of_mut!(lbRect) as LPCRECT, pt) != 0 {
        (*lphc).wState &= !(CBF_CAPTURE as UINT) as UINT;
        ReleaseCapture();
        if CB_GETTYPE!(lphc) as UINT == CBS_DROPDOWN as UINT {
            CBUpdateLBox(lphc, TRUE);
        }

        /* hand over pointer tracking */
        SendMessageW(
            (*lphc).hWndLBox,
            WM_LBUTTONDOWN as UINT,
            wParam,
            lParam,
        );
    }
}

unsafe extern "C" fn COMBO_GetComboBoxInfo(
    mut lphc: *const HEADCOMBO,
    mut pcbi: *mut COMBOBOXINFO,
) -> LRESULT {
    if pcbi.is_null() || ((*pcbi).cbSize as usize) < ::core::mem::size_of::<COMBOBOXINFO>() as usize {
        return FALSE as LRESULT;
    }
    (*pcbi).rcItem = (*lphc).textRect;
    (*pcbi).rcButton = (*lphc).buttonRect;
    (*pcbi).stateButton = 0 as DWORD;
    if (*lphc).wState & CBF_BUTTONDOWN as UINT != 0 {
        (*pcbi).stateButton |= STATE_SYSTEM_PRESSED as DWORD;
    }
    if IsRectEmpty(&raw const (*lphc).buttonRect) != 0 {
        (*pcbi).stateButton |= STATE_SYSTEM_INVISIBLE as DWORD;
    }
    (*pcbi).hwndCombo = (*lphc).self_;
    (*pcbi).hwndItem = (*lphc).hWndEdit;
    (*pcbi).hwndList = (*lphc).hWndLBox;
    return TRUE as LRESULT;
}

unsafe extern "system" fn COMBO_WindowProc(
    hwnd: HWND,
    message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut lphc: *mut HEADCOMBO =
        GetWindowLongPtrW(hwnd, 0) as *mut HEADCOMBO;
    let mut theme: HTHEME = core::ptr::null_mut::<core::ffi::c_void>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/combo.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBO_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            1672 as ::core::ffi::c_int,
            b"[%p]: msg %#x wp %08lx lp %08lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            message,
            wParam,
            lParam,
        );
    }

    if IsWindow(hwnd) == 0 {
        return 0;
    }

    if !lphc.is_null() || message == WM_NCCREATE {
        let mut fallto: u8 = 0;
        match message {
            _ if message == WM_NCCREATE => {
                let style: LONG = (*(lParam as *mut CREATESTRUCTW)).style;
                return COMBO_NCCreate(hwnd, style);
            }

            _ if message == WM_NCDESTROY => {
                COMBO_NCDestroy(lphc);
            }

            _ if message == WM_CREATE => {
                let mut hwndParent: HWND = core::ptr::null_mut::<HWND__>();
                let mut style: LONG = 0;
                hwndParent = (*(lParam as *mut CREATESTRUCTW)).hwndParent;
                style = (*(lParam as *mut CREATESTRUCTW)).style;
                return COMBO_Create(hwnd, lphc, hwndParent, style);
            }

            _ if message == WM_DESTROY => {
                theme = GetWindowTheme(hwnd);
                CloseThemeData(theme);
            }

            _ if message == WM_THEMECHANGED => {
                theme = GetWindowTheme(hwnd);
                CloseThemeData(theme);
                OpenThemeData(hwnd, (WC_COMBOBOXW).as_ptr());
            }

            _ if message == WM_PRINTCLIENT || message == WM_PAINT => {
                let mut ret: LRESULT = 0;
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                let mut hdc: HDC = core::ptr::null_mut::<HDC__>();

                hdc = if wParam != 0 {
                    wParam as HDC
                } else {
                    BeginPaint(hwnd, core::ptr::addr_of_mut!(ps))
                };

                if !hdc.is_null() && ((*lphc).wState & CBF_NOREDRAW) == 0 {
                    let theme: HTHEME = GetWindowTheme(hwnd);

                    if !theme.is_null() {
                        ret = COMBO_ThemedPaint(theme, lphc, hdc);
                    } else {
                        ret = COMBO_Paint(lphc, hdc);
                    }
                }

                if wParam == 0 {
                    EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
                }

                return ret;
            }

            _ if message == WM_ERASEBKGND => {
                /* do all painting in WM_PAINT like Windows does */
                return 1;
            }

            _ if message == WM_GETDLGCODE => {
                let mut result: LRESULT =
                    (DLGC_WANTARROWS | DLGC_WANTCHARS) as LRESULT;
                if lParam != 0 && ((*(lParam as LPMSG)).message == WM_KEYDOWN) {
                    let vk: INT = (*(lParam as LPMSG)).wParam as INT;

                    if ((vk) as u32 == VK_RETURN || (vk) as u32 == VK_ESCAPE)
                        && ((*lphc).wState & CBF_DROPPED) != 0
                    {
                        result |= DLGC_WANTMESSAGE as LRESULT;
                    }
                }
                return result;
            }

            _ if message == WM_SIZE => {
                COMBO_Size(lphc);
                return (TRUE) as LRESULT;
            }

            _ if message == WM_SETFONT => {
                COMBO_Font(lphc, wParam as HFONT, lParam as BOOL);
                return (TRUE) as LRESULT;
            }

            _ if message == WM_GETFONT => {
                return (*lphc).hFont as LRESULT;
            }

            _ if message == WM_SETFOCUS => {
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    SetFocus((*lphc).hWndEdit);
                    /* The first time focus is received, select all the text */
                    if ((*lphc).wState & CBF_BEENFOCUSED) == 0 {
                        SendMessageW(
                            (*lphc).hWndEdit,
                            EM_SETSEL,
                            0,
                            -(1),
                        );
                        (*lphc).wState |= CBF_BEENFOCUSED;
                    }
                } else {
                    COMBO_SetFocus(lphc);
                }
                return (TRUE) as LRESULT;
            }

            _ if message == WM_KILLFOCUS => {
                let hwndFocus: HWND = wParam as HWND;
                if hwndFocus.is_null()
                    || (hwndFocus != (*lphc).hWndEdit && hwndFocus != (*lphc).hWndLBox)
                {
                    COMBO_KillFocus(lphc);
                }
                return (TRUE) as LRESULT;
            }

            _ if message == WM_COMMAND => {
                return COMBO_Command(lphc, wParam, lParam as HWND);
            }

            _ if message == WM_GETTEXT => {
                return COMBO_GetText(lphc, (wParam) as INT, lParam as LPWSTR);
            }

            _ if message == WM_SETTEXT
                || message == WM_GETTEXTLENGTH
                || message == WM_CLEAR =>
            {
                if (message == WM_GETTEXTLENGTH)
                    && !(LOWORD!(GetVersion()) == 0x0a03)
                    && ((*lphc).wState & CBF_EDIT) == 0
                {
                    let j: INT = SendMessageW(
                        (*lphc).hWndLBox,
                        LB_GETCURSEL,
                        0,
                        0,
                    ) as INT;
                    if j == -(1) {
                        return 0;
                    }
                    return SendMessageW(
                        (*lphc).hWndLBox,
                        LB_GETTEXTLEN,
                        j as WPARAM,
                        0,
                    );
                } else if ((*lphc).wState & CBF_EDIT) != 0 {
                    let mut ret: LRESULT = 0;
                    (*lphc).wState |= CBF_NOEDITNOTIFY;
                    ret = SendMessageW((*lphc).hWndEdit, message, wParam, lParam);
                    (*lphc).wState &= !(CBF_NOEDITNOTIFY as UINT);
                    return ret;
                } else {
                    return (CB_ERR) as LRESULT;
                }
            }

            _ if message == WM_CUT
                || message == WM_PASTE
                || message == WM_COPY =>
            {
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    return SendMessageW((*lphc).hWndEdit, message, wParam, lParam);
                } else {
                    return (CB_ERR) as LRESULT;
                }
            }

            _ if message == WM_DRAWITEM
                || message == WM_DELETEITEM
                || message == WM_COMPAREITEM
                || message == WM_MEASUREITEM =>
            {
                return COMBO_ItemOp(lphc, message, lParam);
            }

            _ if message == WM_ENABLE => {
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    EnableWindow((*lphc).hWndEdit, wParam as BOOL);
                }
                EnableWindow((*lphc).hWndLBox, wParam as BOOL);

                /* Force the control to repaint when the enabled state changes. */
                InvalidateRect(
                    (*lphc).self_,
                    core::ptr::null_mut(),
                    TRUE,
                );
                return (TRUE) as LRESULT;
            }

            _ if message == WM_SETREDRAW => {
                if wParam != 0 {
                    (*lphc).wState &= !(CBF_NOREDRAW as UINT);
                } else {
                    (*lphc).wState |= CBF_NOREDRAW;
                }

                if ((*lphc).wState & CBF_EDIT) != 0 {
                    SendMessageW((*lphc).hWndEdit, message, wParam, lParam);
                }
                SendMessageW((*lphc).hWndLBox, message, wParam, lParam);
                return 0;
            }

            _ if message == WM_SYSKEYDOWN => {
                if (KEYDATA_ALT & (HIWORD!(lParam) as u32)) != 0 {
                    if wParam == (VK_UP) as _ || wParam == (VK_DOWN) as _ {
                        COMBO_FlipListbox(
                            lphc,
                            FALSE,
                            FALSE,
                        );
                        return 0;
                    }
                }
                /* break */
            }

            _ if message == WM_KEYDOWN => {
                if (wParam == (VK_RETURN) as _ || wParam == (VK_ESCAPE) as _)
                    && ((*lphc).wState & CBF_DROPPED) != 0
                {
                    CBRollUp(
                        lphc,
                        (wParam == (VK_RETURN) as _) as BOOL,
                        FALSE,
                    );
                    return (TRUE) as LRESULT;
                } else if wParam == (VK_F4) as _
                    && ((*lphc).wState & CBF_EUI) == 0
                {
                    COMBO_FlipListbox(
                        lphc,
                        FALSE,
                        FALSE,
                    );
                    return (TRUE) as LRESULT;
                }
                /* fall through */
                fallto = 1;
            }

            _ if message == WM_CHAR || message == WM_IME_CHAR => {
                fallto = 1;
            }

            _ if message == WM_LBUTTONDOWN => {
                if ((*lphc).wState & CBF_FOCUSED) == 0 {
                    SetFocus((*lphc).self_);
                }
                if ((*lphc).wState & CBF_FOCUSED) != 0 {
                    COMBO_LButtonDown(lphc, lParam);
                }
                return TRUE as LRESULT;
            }

            _ if message == WM_LBUTTONUP => {
                COMBO_LButtonUp(lphc);
                return TRUE as LRESULT;
            }

            _ if message == WM_MOUSEMOVE => {
                if IsRectEmpty(core::ptr::addr_of_mut!((*lphc).buttonRect)) == 0 {
                    let mut pt: POINT = core::mem::zeroed();

                    pt.x = LOWORD!(lParam) as SHORT as LONG;
                    pt.y = HIWORD!(lParam) as SHORT as LONG;

                    if PtInRect(core::ptr::addr_of_mut!((*lphc).buttonRect), pt)
                        != 0
                    {
                        if ((*lphc).wState & CBF_HOT) == 0 {
                            (*lphc).wState |= CBF_HOT;
                            RedrawWindow(
                                hwnd,
                                core::ptr::addr_of_mut!((*lphc).buttonRect),
                                core::ptr::null_mut(),
                                RDW_INVALIDATE | RDW_UPDATENOW,
                            );
                        }
                    } else if ((*lphc).wState & CBF_HOT) != 0 {
                        (*lphc).wState &= !(CBF_HOT as UINT);
                        RedrawWindow(
                            hwnd,
                            core::ptr::addr_of_mut!((*lphc).buttonRect),
                            core::ptr::null_mut(),
                            RDW_INVALIDATE | RDW_UPDATENOW,
                        );
                    }
                }

                if ((*lphc).wState & CBF_CAPTURE) != 0 {
                    COMBO_MouseMove(lphc, wParam, lParam);
                }
                return TRUE as LRESULT;
            }

            _ if message == WM_MOUSEWHEEL => {
                if (wParam & ((MK_SHIFT | MK_CONTROL)) as WPARAM) != 0 {
                    return DefWindowProcW(hwnd, message, wParam, lParam);
                }

                if GET_WHEEL_DELTA_WPARAM!(wParam) > 0 {
                    return SendMessageW(
                        hwnd,
                        WM_KEYDOWN,
                        (VK_UP) as WPARAM,
                        0,
                    );
                }
                if GET_WHEEL_DELTA_WPARAM!(wParam) < 0 {
                    return SendMessageW(
                        hwnd,
                        WM_KEYDOWN,
                        (VK_DOWN) as WPARAM,
                        0,
                    );
                }
                return TRUE as LRESULT;
            }

            _ if message == WM_CTLCOLOR
                || message == WM_CTLCOLORMSGBOX
                || message == WM_CTLCOLOREDIT
                || message == WM_CTLCOLORLISTBOX
                || message == WM_CTLCOLORBTN
                || message == WM_CTLCOLORDLG
                || message == WM_CTLCOLORSCROLLBAR
                || message == WM_CTLCOLORSTATIC =>
            {
                return SendMessageW((*lphc).owner, message, wParam, lParam);
            }

            /* Combo messages */
            _ if message == CB_ADDSTRING => {
                if ((*lphc).dwStyle & CBS_LOWERCASE) != 0 {
                    CharLowerW(lParam as LPWSTR);
                } else if ((*lphc).dwStyle & CBS_UPPERCASE) != 0 {
                    CharUpperW(lParam as LPWSTR);
                }
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_ADDSTRING,
                    0,
                    lParam,
                );
            }

            _ if message == CB_INSERTSTRING => {
                if ((*lphc).dwStyle & CBS_LOWERCASE) != 0 {
                    CharLowerW(lParam as LPWSTR);
                } else if ((*lphc).dwStyle & CBS_UPPERCASE) != 0 {
                    CharUpperW(lParam as LPWSTR);
                }
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_INSERTSTRING,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_DELETESTRING => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_DELETESTRING,
                    wParam,
                    0,
                );
            }

            _ if message == CB_SELECTSTRING => {
                return COMBO_SelectString(lphc, wParam as INT, lParam);
            }

            _ if message == CB_FINDSTRING => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_FINDSTRING,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_FINDSTRINGEXACT => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_FINDSTRINGEXACT,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_SETITEMHEIGHT => {
                return COMBO_SetItemHeight(lphc, wParam as INT, lParam as INT);
            }

            _ if message == CB_GETITEMHEIGHT => {
                if (wParam as INT) >= 0 {
                    /* listbox item */
                    return SendMessageW(
                        (*lphc).hWndLBox,
                        LB_GETITEMHEIGHT,
                        wParam,
                        0,
                    );
                }
                return CBGetTextAreaHeight(lphc, FALSE) as LRESULT;
            }

            _ if message == CB_RESETCONTENT => {
                SendMessageW(
                    (*lphc).hWndLBox,
                    LB_RESETCONTENT,
                    0,
                    0,
                );

                if ((*lphc).wState & CBF_EDIT) != 0 && CB_HASSTRINGS!(lphc) != 0 {
                    static empty_stringW: [WCHAR; 1] = [0; 1];
                    SendMessageW(
                        (*lphc).hWndEdit,
                        WM_SETTEXT,
                        0,
                        core::ptr::addr_of!(empty_stringW) as *const WCHAR as LPARAM,
                    );
                } else {
                    InvalidateRect(
                        (*lphc).self_,
                        core::ptr::null_mut(),
                        TRUE,
                    );
                }
                return TRUE as LRESULT;
            }

            _ if message == CB_INITSTORAGE => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_INITSTORAGE,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_GETHORIZONTALEXTENT => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETHORIZONTALEXTENT,
                    0,
                    0,
                );
            }

            _ if message == CB_SETHORIZONTALEXTENT => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_SETHORIZONTALEXTENT,
                    wParam,
                    0,
                );
            }

            _ if message == CB_GETTOPINDEX => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTOPINDEX,
                    0,
                    0,
                );
            }

            _ if message == CB_GETLOCALE => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETLOCALE,
                    0,
                    0,
                );
            }

            _ if message == CB_SETLOCALE => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_SETLOCALE,
                    wParam,
                    0,
                );
            }

            _ if message == CB_SETDROPPEDWIDTH => {
                if CB_GETTYPE!(lphc) == CBS_SIMPLE
                    || (wParam as INT) >= 32768
                {
                    return (CB_ERR) as LRESULT;
                }

                /* new value must be higher than combobox width */
                if (wParam as INT) >= (*lphc).droppedRect.right - (*lphc).droppedRect.left {
                    (*lphc).droppedWidth = wParam as INT;
                } else if wParam != 0 {
                    (*lphc).droppedWidth = 0;
                }

                /* recalculate the combobox area */
                CBCalcPlacement(lphc);

                /* fall through */
                fallto = 2;
            }

            _ if message == CB_GETDROPPEDWIDTH => {
                fallto = 2;
            }

            _ if message == CB_GETDROPPEDCONTROLRECT => {
                if lParam != 0 {
                    CBGetDroppedControlRect(lphc, lParam as LPRECT);
                }
                return (CB_OKAY) as LRESULT;
            }

            _ if message == CB_GETDROPPEDSTATE => {
                return ((((*lphc).wState & CBF_DROPPED) != 0)
                    as ::core::ffi::c_int) as LRESULT;
            }

            _ if message == CB_DIR => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_DIR,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_SHOWDROPDOWN => {
                if CB_GETTYPE!(lphc) != CBS_SIMPLE {
                    if wParam != 0 {
                        if ((*lphc).wState & CBF_DROPPED) == 0 {
                            CBDropDown(lphc);
                        }
                    } else if ((*lphc).wState & CBF_DROPPED) != 0 {
                        CBRollUp(
                            lphc,
                            FALSE,
                            TRUE,
                        );
                    }
                }
                return TRUE as LRESULT;
            }

            _ if message == CB_GETCOUNT => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETCOUNT,
                    0,
                    0,
                );
            }

            _ if message == CB_GETCURSEL => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETCURSEL,
                    0,
                    0,
                );
            }

            _ if message == CB_SETCURSEL => {
                lParam = SendMessageW(
                    (*lphc).hWndLBox,
                    LB_SETCURSEL,
                    wParam,
                    0,
                ) as LPARAM;
                if lParam >= 0 {
                    SendMessageW(
                        (*lphc).hWndLBox,
                        LB_SETTOPINDEX,
                        wParam,
                        0,
                    );
                }

                /* no LBN_SELCHANGE in this case, update manually */
                CBPaintText(lphc, core::ptr::null_mut());
                (*lphc).wState &= !(CBF_SELCHANGE as UINT);
                return lParam;
            }

            _ if message == CB_GETLBTEXT => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTEXT,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_GETLBTEXTLEN => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETTEXTLEN,
                    wParam,
                    0,
                );
            }

            _ if message == CB_GETITEMDATA => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_GETITEMDATA,
                    wParam,
                    0,
                );
            }

            _ if message == CB_SETITEMDATA => {
                return SendMessageW(
                    (*lphc).hWndLBox,
                    LB_SETITEMDATA,
                    wParam,
                    lParam,
                );
            }

            _ if message == CB_GETEDITSEL => {
                /* Edit checks passed parameters itself */
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    return SendMessageW(
                        (*lphc).hWndEdit,
                        EM_GETSEL,
                        wParam,
                        lParam,
                    );
                }
                return (CB_ERR) as LRESULT;
            }

            _ if message == CB_SETEDITSEL => {
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    return SendMessageW(
                        (*lphc).hWndEdit,
                        EM_SETSEL,
                        ((LOWORD!(lParam) as SHORT) as INT) as WPARAM,
                        ((HIWORD!(lParam) as SHORT) as INT) as LPARAM,
                    );
                }
                return (CB_ERR) as LRESULT;
            }

            _ if message == CB_SETEXTENDEDUI => {
                if CB_GETTYPE!(lphc) == CBS_SIMPLE {
                    return (CB_ERR) as LRESULT;
                }
                if wParam != 0 {
                    (*lphc).wState |= CBF_EUI;
                } else {
                    (*lphc).wState &= !(CBF_EUI as UINT);
                }
                return (CB_OKAY) as LRESULT;
            }

            _ if message == CB_GETEXTENDEDUI => {
                return ((((*lphc).wState & CBF_EUI) != 0)
                    as ::core::ffi::c_int) as LRESULT;
            }

            _ if message == CB_GETCOMBOBOXINFO => {
                return COMBO_GetComboBoxInfo(lphc, lParam as *mut COMBOBOXINFO);
            }

            _ if message == CB_LIMITTEXT => {
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    return SendMessageW(
                        (*lphc).hWndEdit,
                        EM_LIMITTEXT,
                        wParam,
                        lParam,
                    );
                }
                return TRUE as LRESULT;
            }

            _ if message == CB_GETMINVISIBLE => {
                return (*lphc).visibleItems as LRESULT;
            }

            _ if message == CB_SETMINVISIBLE => {
                (*lphc).visibleItems = wParam as INT;
                return TRUE as LRESULT;
            }

            _ => {
                if message >= WM_USER {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"dll/win32/comctl32/combo.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"COMBO_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                            2117 as ::core::ffi::c_int,
                            b"unknown msg WM_USER+%04x wp=%04lx lp=%08lx\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            message.wrapping_sub(WM_USER),
                            wParam,
                            lParam,
                        );
                    }
                }
                /* break */
            }
        }

        match fallto {
            1 => {
                /* COMBO_KEYDOWN / CHAR / UNICHAR target */
                let mut hwndTarget: HWND = core::ptr::null_mut::<HWND__>();
                if ((*lphc).wState & CBF_DROPPED) != 0 {
                    (*lphc).wState |= CBF_NOROLLUP;
                }
                if ((*lphc).wState & CBF_EDIT) != 0 {
                    hwndTarget = (*lphc).hWndEdit;
                } else {
                    hwndTarget = (*lphc).hWndLBox;
                }
                return SendMessageW(hwndTarget, message, wParam, lParam);
            }
            2 => {
                if (*lphc).droppedWidth != 0 {
                    return (*lphc).droppedWidth as LRESULT;
                }
                return ((*lphc).droppedRect.right - (*lphc).droppedRect.left) as LRESULT;
            }
            _ => {}
        }
    }

    return DefWindowProcW(hwnd, message, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn COMBO_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(wndClass) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<WNDCLASSW>() as size_t,
    );
    wndClass.style = (CS_PARENTDC | CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW | CS_GLOBALCLASS) as UINT;
    wndClass.lpfnWndProc = Some(COMBO_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut HEADCOMBO>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = core::ptr::null_mut::<HBRUSH__>();
    wndClass.lpszClassName = WC_COMBOBOXW.as_ptr() as LPCWSTR;
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn COMBO_Unregister() {
    UnregisterClassW((WC_COMBOBOXW).as_ptr(), core::ptr::null_mut());
}
