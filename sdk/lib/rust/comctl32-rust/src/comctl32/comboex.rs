use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_comboex);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_COMBOBOXEXW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"C\0o\0m\0b\0o\0B\0o\0x\0E\0x\x003\x002\0\0\0",
    )
};

pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

// WINE_DEFAULT_DEBUG_CHANNEL(comboex): debug-channel macro, skipped.


pub type PCBE_ITEMDATA = *mut CBE_ITEMDATA;

#[repr(C)]
pub struct COMBOEX_INFO {
    pub himl: HIMAGELIST,
    pub hwndSelf: HWND,      // my own hwnd
    pub hwndNotify: HWND,    // my parent hwnd
    pub hwndCombo: HWND,
    pub hwndEdit: HWND,
    pub dwExtStyle: DWORD,
    pub selected: INT,       // index of selected item
    pub flags: DWORD,        // WINE internal flags
    pub defaultFont: HFONT,
    pub font: HFONT,
    pub nb_items: INT,       // Number of items
    pub unicode: BOOL,       // TRUE if this window is Unicode
    pub NtfUnicode: BOOL,    // TRUE if parent wants notify in Unicode
    pub edit: CBE_ITEMDATA,  // item data for edit item
    pub items: *mut CBE_ITEMDATA, // Array of items
}

pub type PCOMBOEX_INFO = *mut COMBOEX_INFO;

pub const WCBE_ACTEDIT: u32 = 0x00000001;
pub const WCBE_EDITCHG: u32 = 0x00000002;
pub const WCBE_EDITHASCHANGED: u32 = WCBE_ACTEDIT | WCBE_EDITCHG;
pub const WCBE_EDITFOCUSED: u32 = 0x00000004;
pub const WCBE_MOUSECAPTURED: u32 = 0x00000008;
pub const WCBE_MOUSEDRAGGED: u32 = 0x00000010;

pub const ID_CB_EDIT: i32 = 1001;

pub const ODS_COMBOEXLBOX: u32 = 0x4000;

pub const CBE_EXTRA: i32 = 3;

pub const CBE_INDENT: i32 = 10;

pub const CBE_STARTOFFSET: i32 = 6;

pub const CBE_SEP: i32 = 4;

pub const COMBO_SUBCLASSID: usize = 1;

pub const EDIT_SUBCLASSID: usize = 2;

// COMBOEX_GetInfoPtr(hwnd): function-like macro, skipped.
// Static prototypes COMBOEX_EditWndProc, COMBOEX_ComboWndProc, COMBOEX_Destroy:
// function declarations, not emitted here.

pub type cmp_func_t = unsafe extern "system" fn(LPCWSTR, LPCWSTR) -> INT;
unsafe extern "C" fn is_textW(mut str: LPCWSTR) -> BOOL {
    if !str.is_null() && str != LPSTR_TEXTCALLBACKW as LPCWSTR {
        return 1;
    } else {
        return 0;
    }
}

unsafe extern "C" fn is_textA(str: LPCSTR) -> BOOL {
    return (!str.is_null() && str != LPSTR_TEXTCALLBACKA as LPCSTR) as BOOL;
}

unsafe extern "C" fn debugstr_txt(str: LPCWSTR) -> LPCSTR {
    if str == LPSTR_TEXTCALLBACKW as LPCWSTR {
        return b"(callback)\0".as_ptr() as LPCSTR;
    }
    return debugstr_w(str) as LPCSTR;
}

unsafe extern "C" fn COMBOEX_DumpItem(item: *const CBE_ITEMDATA) {
    if ((*item).mask & CBEIF_TEXT) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBOEX_DumpItem\0" as *const u8 as *const ::core::ffi::c_char,
                146 as ::core::ffi::c_int,
                b"item %p - pszText=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                item,
                debugstr_txt((*item).pszText as LPCWSTR),
            );
        }
    }
}

unsafe extern "C" fn COMBOEX_DumpInput(input: *const COMBOBOXEXITEMW) {
    if (*input).mask & CBEIF_TEXT as UINT != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBOEX_DumpInput\0" as *const u8 as *const ::core::ffi::c_char,
                156 as ::core::ffi::c_int,
                b"input - pszText=<%s>\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_txt((*input).pszText as LPCWSTR),
            );
        }
    }
}

unsafe extern "C" fn get_item_data(infoPtr: *const COMBOEX_INFO, index: INT) -> *mut CBE_ITEMDATA {
    return SendMessageW((*infoPtr).hwndCombo, CB_GETITEMDATA, index as WPARAM, 0) as *mut CBE_ITEMDATA;
}

unsafe extern "C" fn get_cmp_func(infoPtr: *const COMBOEX_INFO) -> cmp_func_t {
    return if ((*infoPtr).dwExtStyle & (CBES_EX_CASESENSITIVE as DWORD)) != 0 {
        lstrcmpW as unsafe extern "system" fn(LPCWSTR, LPCWSTR) -> i32
    } else {
        lstrcmpiW as unsafe extern "system" fn(LPCWSTR, LPCWSTR) -> i32
    };
}

unsafe extern "C" fn COMBOEX_Notify(infoPtr: *const COMBOEX_INFO, code: INT, hdr: *mut NMHDR) -> INT {
    (*hdr).idFrom = GetDlgCtrlID((*infoPtr).hwndSelf) as UINT_PTR;
    (*hdr).hwndFrom = (*infoPtr).hwndSelf;
    (*hdr).code = code as UINT;
    if (*infoPtr).NtfUnicode != 0 {
        return SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            0 as WPARAM,
            hdr as LPARAM,
        ) as INT;
    } else {
        return SendMessageA(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            0 as WPARAM,
            hdr as LPARAM,
        ) as INT;
    }
}

unsafe extern "C" fn COMBOEX_NotifyItem(infoPtr: *const COMBOEX_INFO, code: UINT, hdr: *mut NMCOMBOBOXEXW) -> INT {
    let mut code = code;
    if (*infoPtr).NtfUnicode != 0 {
        return COMBOEX_Notify(infoPtr, (code) as INT, core::ptr::addr_of_mut!((*hdr).hdr));
    } else {
        let wstr: LPWSTR = (*hdr).ceItem.pszText;
        let mut astr: LPSTR = 0 as LPSTR;
        let mut ret: INT;
        let mut len: INT = 0 as INT;

        if ((*hdr).ceItem.mask & (CBEIF_TEXT as UINT) != 0) && is_textW(wstr as LPCWSTR) != 0 {
            len = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                wstr as LPCWSTR,
                -(1 as core::ffi::c_int),
                0 as LPSTR,
                0 as core::ffi::c_int,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as INT;
            if len > 0 as INT {
                astr = Alloc(
                    ((len + 1 as INT) as usize)
                        .wrapping_mul(core::mem::size_of::<CHAR>() as usize)
                        as DWORD,
                ) as LPSTR;
                if astr.is_null() {
                    return 0 as INT;
                }
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    wstr as LPCWSTR,
                    -(1 as core::ffi::c_int),
                    astr,
                    len as core::ffi::c_int,
                    0 as *const CHAR,
                    0 as *mut BOOL,
                );
                (*hdr).ceItem.pszText = astr as LPWSTR;
            }
        }

        if code == CBEN_ENDEDITW as UINT {
            code = CBEN_ENDEDITA as UINT;
        } else if code == CBEN_GETDISPINFOW as UINT {
            code = CBEN_GETDISPINFOA as UINT;
        } else if code == CBEN_DRAGBEGINW as UINT {
            code = CBEN_DRAGBEGINA as UINT;
        }

        ret = COMBOEX_Notify(infoPtr, code as INT, hdr as *mut NMHDR);

        if !astr.is_null() && (*hdr).ceItem.pszText == astr as LPWSTR {
            (*hdr).ceItem.pszText = wstr;
        }

        Free(astr as LPVOID);

        return ret;
    }
}

unsafe extern "C" fn COMBOEX_NotifyEndEdit(infoPtr: *const COMBOEX_INFO, neew: *mut NMCBEENDEDITW, wstr: LPCWSTR) -> INT {
    /* Change the Text item from Unicode to ANSI if necessary for NOTIFY */
    if (*infoPtr).NtfUnicode != 0 {
        lstrcpynW(core::ptr::addr_of_mut!((*neew).szText) as LPWSTR, wstr, CBEMAXSTRLEN as INT);
        return COMBOEX_Notify(infoPtr, CBEN_ENDEDITW as INT, core::ptr::addr_of_mut!((*neew).hdr));
    } else {
        let mut neea: NMCBEENDEDITA = core::mem::zeroed();

        neea.hdr = (*neew).hdr;
        neea.fChanged = (*neew).fChanged;
        neea.iNewSelection = (*neew).iNewSelection;
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            wstr,
            -(1 as ::core::ffi::c_int),
            core::ptr::addr_of_mut!(neea.szText) as LPSTR,
            CBEMAXSTRLEN as INT,
            0 as *const CHAR,
            0 as *mut BOOL,
        );
        neea.iWhy = (*neew).iWhy;

        return COMBOEX_Notify(infoPtr, CBEN_ENDEDITA as INT, core::ptr::addr_of_mut!(neea.hdr));
    }
}

unsafe extern "C" fn COMBOEX_NotifyDragBegin(mut infoPtr: *const COMBOEX_INFO, mut wstr: LPCWSTR) {
    if (*infoPtr).NtfUnicode != 0 {
        let mut ndbw: NMCBEDRAGBEGINW = NMCBEDRAGBEGINW {
            hdr: tagNMHDR {
                hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                idFrom: 0,
                code: 0,
            },
            iItemid: 0,
            szText: [0; 260],
        };
        ndbw.iItemid = -(1 as ::core::ffi::c_int);
        lstrcpynW(&raw mut ndbw.szText as LPWSTR, wstr, CBEMAXSTRLEN as ::core::ffi::c_int);
        COMBOEX_Notify(infoPtr, CBEN_DRAGBEGINW as INT, &raw mut ndbw.hdr);
    } else {
        let mut ndba: NMCBEDRAGBEGINA = NMCBEDRAGBEGINA {
            hdr: tagNMHDR {
                hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                idFrom: 0,
                code: 0,
            },
            iItemid: 0,
            szText: [0; 260],
        };
        ndba.iItemid = -(1 as ::core::ffi::c_int);
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            wstr,
            -(1 as ::core::ffi::c_int),
            &raw mut ndba.szText as LPSTR,
            CBEMAXSTRLEN as ::core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        COMBOEX_Notify(infoPtr, CBEN_DRAGBEGINA as INT, &raw mut ndba.hdr);
    };
}

unsafe extern "C" fn COMBOEX_FreeText(item: *mut CBE_ITEMDATA) {
    if is_textW((*item).pszText as LPCWSTR) != 0 {
        Free((*item).pszText as LPVOID);
    }
    (*item).pszText = core::ptr::null_mut::<WCHAR>();
    Free((*item).pszTemp as LPVOID);
    (*item).pszTemp = core::ptr::null_mut::<WCHAR>();
}

unsafe extern "C" fn COMBOEX_GetIndex(
    mut infoPtr: *const COMBOEX_INFO,
    mut item: *const CBE_ITEMDATA,
) -> INT {
    let mut moving: *const CBE_ITEMDATA = core::ptr::null::<CBE_ITEMDATA>();
    let mut index: INT = 0;
    moving = (*infoPtr).items;
    index = ((*infoPtr).nb_items as ::core::ffi::c_int - 1) as INT;
    while !moving.is_null() && moving != item {
        moving = (*moving).next;
        index -= 1;
    }
    if moving.is_null() || index < 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBOEX_GetIndex\0" as *const u8 as *const ::core::ffi::c_char,
                284 as ::core::ffi::c_int,
                b"COMBOBOXEX item structures broken. Please report!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return -1;
    }
    return index;
}

unsafe extern "C" fn COMBOEX_GetText(infoPtr: *const COMBOEX_INFO, item: *mut CBE_ITEMDATA) -> LPCWSTR {
    let mut nmce: NMCOMBOBOXEXW = core::mem::zeroed();
    let mut text: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut buf: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut len: INT = 0;

    if (*item).pszText != LPSTR_TEXTCALLBACKW {
        return (*item).pszText as LPCWSTR;
    }

    ZeroMemory!(
        &raw mut nmce as *mut ::core::ffi::c_void,
        core::mem::size_of::<NMCOMBOBOXEXW>() as size_t,
    );
    nmce.ceItem.mask = CBEIF_TEXT as UINT;
    nmce.ceItem.lParam = (*item).lParam;
    nmce.ceItem.iItem = COMBOEX_GetIndex(infoPtr, item) as INT_PTR;
    COMBOEX_NotifyItem(infoPtr, CBEN_GETDISPINFOW as UINT, &raw mut nmce);

    if is_textW(nmce.ceItem.pszText as LPCWSTR) != 0 {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            nmce.ceItem.pszText as LPSTR as LPCSTR,
            -(1 as ::core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        buf = Alloc(
            (((len + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD,
        ) as LPWSTR;
        if !buf.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                nmce.ceItem.pszText as LPSTR as LPCSTR,
                -(1 as ::core::ffi::c_int),
                buf,
                len as ::core::ffi::c_int,
            );
        }
        if (nmce.ceItem.mask & CBEIF_DI_SETITEM as UINT) != 0 {
            COMBOEX_FreeText(item);
            (*item).pszText = buf;
        } else {
            Free((*item).pszTemp as LPVOID);
            (*item).pszTemp = buf;
        }
        text = buf;
    } else {
        text = nmce.ceItem.pszText;
    }

    if (nmce.ceItem.mask & CBEIF_DI_SETITEM as UINT) != 0 {
        (*item).pszText = text;
    }
    return text as LPCWSTR;
}

unsafe extern "C" fn COMBOEX_GetComboFontSize(infoPtr: *const COMBOEX_INFO, size: *mut SIZE) {
    static strA: [u16; 2] = [0x0041u16, 0x0000];
    let mut nfont: HFONT = core::ptr::null_mut();
    let mut ofont: HFONT = core::ptr::null_mut();
    let mut mydc: HDC = core::ptr::null_mut();

    mydc = GetDC(0 as HWND); /* why the entire screen???? */
    nfont = SendMessageW(
        (*infoPtr).hwndCombo,
        WM_GETFONT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HFONT;
    ofont = SelectObject(mydc, nfont as HGDIOBJ) as HFONT;
    GetTextExtentPointW(
        mydc,
        core::ptr::addr_of!(strA) as LPCWSTR,
        1 as i32,
        size,
    );
    SelectObject(mydc, ofont as HGDIOBJ);
    ReleaseDC(0 as HWND, mydc);
}

unsafe extern "C" fn COMBOEX_CopyItem(item: *const CBE_ITEMDATA, cit: *mut COMBOBOXEXITEMW) {
    if (*cit).mask & CBEIF_TEXT as UINT != 0 {
        /*
         * when given a text buffer actually use that buffer
         */
        if !(*cit).pszText.is_null() {
            if is_textW((*item).pszText as LPCWSTR) != 0 {
                lstrcpynW(
                    (*cit).pszText,
                    (*item).pszText as LPCWSTR,
                    (*cit).cchTextMax,
                );
            } else {
                *(*cit).pszText.offset(0 as isize) = 0 as WCHAR;
            }
        } else {
            (*cit).pszText = (*item).pszText;
            (*cit).cchTextMax = (*item).cchTextMax;
        }
    }
    if (*cit).mask & CBEIF_IMAGE as UINT != 0 {
        (*cit).iImage = (*item).iImage;
    }
    if (*cit).mask & CBEIF_SELECTEDIMAGE as UINT != 0 {
        (*cit).iSelectedImage = (*item).iSelectedImage;
    }
    if (*cit).mask & CBEIF_OVERLAY as UINT != 0 {
        (*cit).iOverlay = (*item).iOverlay;
    }
    if (*cit).mask & CBEIF_INDENT as UINT != 0 {
        (*cit).iIndent = (*item).iIndent;
    }
    if (*cit).mask & CBEIF_LPARAM as UINT != 0 {
        (*cit).lParam = (*item).lParam;
    }
}

unsafe extern "C" fn COMBOEX_AdjustEditPos(infoPtr: *const COMBOEX_INFO) {
    let mut mysize: SIZE = tagSIZE { cx: 0, cy: 0 };
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut w: INT = 0;
    let mut h: INT = 0;
    let mut xioff: INT = 0;
    let mut rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if (*infoPtr).hwndEdit.is_null() {
        return;
    }

    if !(*infoPtr).himl.is_null() && ((*infoPtr).dwExtStyle & CBES_EX_NOEDITIMAGEINDENT) == 0 {
        let mut iinfo: IMAGEINFO = _IMAGEINFO {
            hbmImage: core::ptr::null_mut::<HBITMAP__>(),
            hbmMask: core::ptr::null_mut::<HBITMAP__>(),
            Unused1: 0,
            Unused2: 0,
            rcImage: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        iinfo.rcImage.right = 0 as LONG;
        iinfo.rcImage.left = iinfo.rcImage.right;
        ImageList_GetImageInfo((*infoPtr).himl, 0, &raw mut iinfo);
        xioff = (iinfo.rcImage.right - iinfo.rcImage.left + CBE_SEP) as INT;
    } else {
        xioff = 0 as INT;
    }

    GetClientRect((*infoPtr).hwndCombo, &raw mut rect);
    InflateRect(
        &raw mut rect,
        -(2 as ::core::ffi::c_int),
        -(2 as ::core::ffi::c_int),
    );
    InvalidateRect((*infoPtr).hwndCombo, &raw mut rect as LPCRECT, TRUE);

    /* reposition the Edit control based on whether icon exists */
    COMBOEX_GetComboFontSize(infoPtr, &raw mut mysize);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_AdjustEditPos\0" as *const u8 as *const ::core::ffi::c_char,
            394 as ::core::ffi::c_int,
            b"Combo font x=%d, y=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            mysize.cx,
            mysize.cy,
        );
    }

    x = xioff + CBE_STARTOFFSET + 1;
    w = rect.right - rect.left - x - GetSystemMetrics((SM_CXVSCROLL) as i32) - 1;
    h = mysize.cy + 1;
    y = rect.bottom - h - 1;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_AdjustEditPos\0" as *const u8 as *const ::core::ffi::c_char,
            401 as ::core::ffi::c_int,
            b"Combo client (%s), setting Edit to (%d,%d)-(%d,%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wine_dbgstr_rect(&raw mut rect),
            x,
            y,
            x + w,
            y + h,
        );
    }

    SetWindowPos(
        (*infoPtr).hwndEdit,
        HWND_TOP,
        x,
        y,
        w,
        h,
        SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOZORDER,
    );
}

unsafe extern "C" fn COMBOEX_ReSize(infoPtr: *const COMBOEX_INFO) {
    let mut mysize: SIZE = core::mem::zeroed();
    let mut cy: LONG = 0;
    let mut iinfo: IMAGEINFO = core::mem::zeroed();
    COMBOEX_GetComboFontSize(infoPtr, &raw mut mysize);
    cy = mysize.cy + CBE_EXTRA;
    if !(*infoPtr).himl.is_null()
        && ImageList_GetImageInfo((*infoPtr).himl, 0, &raw mut iinfo) != 0
    {
        cy = if iinfo.rcImage.bottom - iinfo.rcImage.top > cy {
            iinfo.rcImage.bottom - iinfo.rcImage.top
        } else {
            cy
        };
    }
    SendMessageW(
        (*infoPtr).hwndSelf,
        CB_SETITEMHEIGHT,
        (-1isize) as WPARAM,
        cy as LPARAM,
    );
    if !(*infoPtr).hwndCombo.is_null() {
        SendMessageW(
            (*infoPtr).hwndCombo,
            CB_SETITEMHEIGHT,
            0,
            cy as LPARAM,
        );
        if (*infoPtr).flags & CBES_EX_NOSIZELIMIT == 0 {
            let mut comboRect: RECT = core::mem::zeroed();
            let mut ourRect: RECT = core::mem::zeroed();
            GetWindowRect((*infoPtr).hwndCombo, &raw mut comboRect);
            GetWindowRect((*infoPtr).hwndSelf, &raw mut ourRect);
            if comboRect.bottom > ourRect.bottom {
                SetWindowPos(
                    (*infoPtr).hwndSelf,
                    core::ptr::null_mut(),
                    0,
                    0,
                    ourRect.right - ourRect.left,
                    comboRect.bottom - comboRect.top,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW,
                );
            }
        }
    }
}

unsafe extern "C" fn COMBOEX_SetEditText(infoPtr: *const COMBOEX_INFO, item: *mut CBE_ITEMDATA) {
    if (*infoPtr).hwndEdit.is_null() {
        return;
    }
    if (*item).mask & CBEIF_TEXT as UINT != 0 {
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_SETTEXT as UINT,
            0 as WPARAM,
            COMBOEX_GetText(infoPtr, item) as LPARAM,
        );
        SendMessageW(
            (*infoPtr).hwndEdit,
            EM_SETSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            (*infoPtr).hwndEdit,
            EM_SETSEL as UINT,
            0 as WPARAM,
            -(1 as ::core::ffi::c_int) as LPARAM,
        );
    }
}

unsafe extern "C" fn COMBOEX_FindItem(mut infoPtr: *mut COMBOEX_INFO, mut index: INT_PTR) -> *mut CBE_ITEMDATA {
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut i: INT = 0;

    if index >= (*infoPtr).nb_items as INT_PTR || index < -(1 as INT_PTR) {
        return core::ptr::null_mut::<CBE_ITEMDATA>();
    }
    if index == -(1 as INT_PTR) {
        return core::ptr::addr_of_mut!((*infoPtr).edit);
    }
    item = (*infoPtr).items;
    i = ((*infoPtr).nb_items as INT - 1 as INT) as INT;

    /* find the item in the list */
    while !item.is_null() && (i as INT_PTR) > index {
        item = (*item).next as *mut CBE_ITEMDATA;
        i -= 1;
    }
    if item.is_null() || (i as INT_PTR) != index {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBOEX_FindItem\0" as *const u8 as *const ::core::ffi::c_char,
                465 as ::core::ffi::c_int,
                b"COMBOBOXEX item structures broken. Please report!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }
    return item;
}

unsafe extern "C" fn COMBOEX_GetListboxText(infoPtr: *mut COMBOEX_INFO, n: INT_PTR, buf: LPWSTR) -> UINT {
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut str: LPCWSTR = core::ptr::null::<WCHAR>();

    item = COMBOEX_FindItem(infoPtr, n);
    if item.is_null() {
        return 0 as UINT;
    }

    str = COMBOEX_GetText(infoPtr, item);
    if str.is_null() {
        if !buf.is_null() {
            if (*infoPtr).unicode != 0 {
                *buf.offset(0 as isize) = 0 as WCHAR;
            } else {
                *(buf as LPSTR) = 0 as CHAR;
            }
        }
        return 0 as UINT;
    }

    if (*infoPtr).unicode != 0 {
        if !buf.is_null() {
            lstrcpyW(buf, str);
        }
        return lstrlenW(str) as UINT;
    } else {
        let mut r: UINT = 0;
        r = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            str,
            -1,
            buf as LPSTR,
            0x40000000,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as UINT;
        if r != 0 {
            r = r.wrapping_sub(1);
        }
        return r;
    }
}

unsafe extern "C" fn COMBOEX_DeleteItem(mut infoPtr: *mut COMBOEX_INFO, mut index: INT_PTR) -> INT {
    /* if item number requested does not exist then return failure */
    if index >= (*infoPtr).nb_items as INT_PTR || index < 0 {
        return CB_ERR;
    }
    if COMBOEX_FindItem(infoPtr, index).is_null() {
        return CB_ERR;
    }

    /* doing this will result in WM_DELETEITEM being issued */
    SendMessageW(
        (*infoPtr).hwndCombo,
        CB_DELETESTRING as UINT,
        index as WPARAM,
        0 as LPARAM,
    );

    return (*infoPtr).nb_items;
}

unsafe extern "C" fn COMBOEX_GetItemW(
    mut infoPtr: *mut COMBOEX_INFO,
    mut cit: *mut COMBOBOXEXITEMW,
) -> BOOL {
    let mut index: INT_PTR = (*cit).iItem;
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut();
    if index >= (*infoPtr).nb_items as INT_PTR || index < -(1) as INT_PTR {
        return FALSE;
    }
    if index == -(1) as INT_PTR && (*infoPtr).hwndEdit.is_null() {
        return FALSE;
    }
    item = COMBOEX_FindItem(infoPtr, index);
    if item.is_null() {
        return FALSE;
    }
    COMBOEX_CopyItem(item, cit);
    return TRUE;
}

unsafe extern "C" fn COMBOEX_GetItemA(
    mut infoPtr: *mut COMBOEX_INFO,
    mut cit: *mut COMBOBOXEXITEMA,
) -> BOOL {
    let mut tmpcit: COMBOBOXEXITEMW = tagCOMBOBOXEXITEMW {
        mask: 0,
        iItem: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        iSelectedImage: 0,
        iOverlay: 0,
        iIndent: 0,
        lParam: 0,
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_GetItemA\0" as *const u8 as *const ::core::ffi::c_char,
            551 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    tmpcit.mask = (*cit).mask;
    tmpcit.iItem = (*cit).iItem;
    tmpcit.pszText = core::ptr::null_mut();
    if COMBOEX_GetItemW(infoPtr, &raw mut tmpcit) == 0 {
        return FALSE;
    }
    if (*cit).mask & CBEIF_TEXT != 0 {
        if is_textW(tmpcit.pszText as LPCWSTR) != 0 && !(*cit).pszText.is_null() {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                tmpcit.pszText as LPCWSTR,
                -1,
                (*cit).pszText,
                (*cit).cchTextMax,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
        } else if !(*cit).pszText.is_null() {
            *(*cit).pszText.offset(0 as isize) = 0 as CHAR;
        } else {
            (*cit).pszText = tmpcit.pszText as LPSTR;
        }
    }
    if (*cit).mask & CBEIF_IMAGE != 0 {
        (*cit).iImage = tmpcit.iImage;
    }
    if (*cit).mask & CBEIF_SELECTEDIMAGE != 0 {
        (*cit).iSelectedImage = tmpcit.iSelectedImage;
    }
    if (*cit).mask & CBEIF_OVERLAY != 0 {
        (*cit).iOverlay = tmpcit.iOverlay;
    }
    if (*cit).mask & CBEIF_INDENT != 0 {
        (*cit).iIndent = tmpcit.iIndent;
    }
    if (*cit).mask & CBEIF_LPARAM != 0 {
        (*cit).lParam = tmpcit.lParam;
    }
    return TRUE;
}

unsafe extern "C" fn COMBOEX_HasEditChanged(infoPtr: *const COMBOEX_INFO) -> BOOL {
    return (!(*infoPtr).hwndEdit.is_null()
        && ((*infoPtr).flags & WCBE_EDITHASCHANGED) == WCBE_EDITHASCHANGED) as BOOL;
}

unsafe extern "C" fn COMBOEX_InsertItemW(
    mut infoPtr: *mut COMBOEX_INFO,
    mut cit: *const COMBOBOXEXITEMW,
) -> INT {
    let mut index: INT_PTR = 0;
    let mut item: *mut CBE_ITEMDATA = ::core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut nmcit: NMCOMBOBOXEXW = ::core::mem::zeroed();

    if TRACE_ON!(comboex) {
        COMBOEX_DumpInput(cit);
    }

    /* get real index of item to insert */
    index = (*cit).iItem;
    if index == -1 {
        index = (*infoPtr).nb_items as INT_PTR;
    }
    if index > (*infoPtr).nb_items as INT_PTR {
        return -1;
    }

    /* get zero-filled space and chain it in */
    item = Alloc(::core::mem::size_of::<CBE_ITEMDATA>() as DWORD) as *mut CBE_ITEMDATA;
    if item.is_null() {
        return -1;
    }

    /* locate position to insert new item in */
    if index == (*infoPtr).nb_items as INT_PTR {
        /* fast path for iItem = -1 */
        (*item).next = (*infoPtr).items as *mut _CBE_ITEMDATA;
        (*infoPtr).items = item;
    } else {
        let mut i: INT = (*infoPtr).nb_items - 1;
        let mut moving: *mut CBE_ITEMDATA = (*infoPtr).items;

        while (i as INT_PTR > index) && !moving.is_null() {
            moving = (*moving).next as *mut CBE_ITEMDATA;
            i -= 1;
        }
        if moving.is_null() {
            Free(item as LPVOID);
            return -1;
        }
        (*item).next = (*moving).next;
        (*moving).next = item as *mut _CBE_ITEMDATA;
    }

    /* fill in our hidden item structure */
    (*item).mask = (*cit).mask;
    if ((*item).mask & CBEIF_TEXT) != 0 {
        let mut len: INT = 0;

        if is_textW((*cit).pszText as LPCWSTR) != 0 {
            len = lstrlenW((*cit).pszText as LPCWSTR) as INT;
        }
        if len > 0 {
            (*item).pszText = Alloc(
                ((len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ) as LPWSTR;
            if (*item).pszText.is_null() {
                Free(item as LPVOID);
                return -1;
            }
            lstrcpyW((*item).pszText, (*cit).pszText as LPCWSTR);
        } else if (*cit).pszText == LPSTR_TEXTCALLBACKW as LPWSTR {
            (*item).pszText = LPSTR_TEXTCALLBACKW as LPWSTR;
        }
        (*item).cchTextMax = (*cit).cchTextMax;
    }
    if ((*item).mask & CBEIF_IMAGE) != 0 {
        (*item).iImage = (*cit).iImage;
    }
    if ((*item).mask & CBEIF_SELECTEDIMAGE) != 0 {
        (*item).iSelectedImage = (*cit).iSelectedImage;
    }
    if ((*item).mask & CBEIF_OVERLAY) != 0 {
        (*item).iOverlay = (*cit).iOverlay;
    }
    if ((*item).mask & CBEIF_INDENT) != 0 {
        (*item).iIndent = (*cit).iIndent;
    }
    if ((*item).mask & CBEIF_LPARAM) != 0 {
        (*item).lParam = (*cit).lParam;
    }
    (*infoPtr).nb_items += 1;

    if TRACE_ON!(comboex) {
        COMBOEX_DumpItem(item);
    }

    SendMessageW(
        (*infoPtr).hwndCombo,
        CB_INSERTSTRING as UINT,
        (*cit).iItem as WPARAM,
        item as LPARAM,
    );

    memset(
        &raw mut nmcit.ceItem as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<COMBOBOXEXITEMW>() as usize,
    );
    nmcit.ceItem.mask = !0 as UINT;
    COMBOEX_CopyItem(item, &raw mut nmcit.ceItem);
    COMBOEX_NotifyItem(infoPtr, CBEN_INSERTITEM as UINT, &raw mut nmcit);

    return index as INT;
}

unsafe extern "C" fn COMBOEX_InsertItemA(infoPtr: *mut COMBOEX_INFO, cit: *const COMBOBOXEXITEMA) -> INT {
    let mut citW: COMBOBOXEXITEMW = tagCOMBOBOXEXITEMW {
        mask: 0,
        iItem: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        iSelectedImage: 0,
        iOverlay: 0,
        iIndent: 0,
        lParam: 0,
    };
    let mut wstr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ret: INT = 0;
    memcpy(
        &raw mut citW as *mut ::core::ffi::c_void,
        cit as *const ::core::ffi::c_void,
        core::mem::size_of::<COMBOBOXEXITEMA>() as usize,
    );
    if ((*cit).mask & (CBEIF_TEXT as UINT) != 0) && is_textA((*cit).pszText as LPCSTR) != 0 {
        let mut len: INT = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            (*cit).pszText as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<WCHAR>(),
            0 as ::core::ffi::c_int,
        ) as INT;
        wstr = Alloc(
            ((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ) as LPWSTR;
        if wstr.is_null() {
            return -(1 as INT);
        }
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            (*cit).pszText as LPCSTR,
            -(1 as ::core::ffi::c_int),
            wstr,
            len as ::core::ffi::c_int,
        );
        citW.pszText = wstr;
    }
    ret = COMBOEX_InsertItemW(infoPtr, &raw mut citW);

    Free(wstr as LPVOID);

    return ret;
}

unsafe extern "C" fn COMBOEX_SetExtendedStyle(
    mut infoPtr: *mut COMBOEX_INFO,
    mut mask: DWORD,
    mut style: DWORD,
) -> DWORD {
    let mut dwTemp: DWORD = 0;

    dwTemp = (*infoPtr).dwExtStyle;

    if mask != 0 {
        (*infoPtr).dwExtStyle = ((*infoPtr).dwExtStyle & !mask) | style;
    } else {
        (*infoPtr).dwExtStyle = style;
    }

    /* see if we need to change the word break proc on the edit */
    if ((*infoPtr).dwExtStyle ^ dwTemp) & (CBES_EX_PATHWORDBREAKPROC as DWORD) != 0 {
        SetPathWordBreakProc(
            (*infoPtr).hwndEdit,
            (((*infoPtr).dwExtStyle & (CBES_EX_PATHWORDBREAKPROC as DWORD))
                != (0 as DWORD)) as ::core::ffi::c_int,
        );
    }

    /* test if the control's appearance has changed */
    mask = (CBES_EX_NOEDITIMAGE | CBES_EX_NOEDITIMAGEINDENT) as DWORD;
    if ((*infoPtr).dwExtStyle & mask) != (dwTemp & mask) {
        /* if state of EX_NOEDITIMAGE changes, invalidate all */
        InvalidateRect(
            (*infoPtr).hwndSelf,
            ::core::ptr::null::<RECT>(),
            TRUE,
        );
        COMBOEX_AdjustEditPos(infoPtr);
        if !(*infoPtr).hwndEdit.is_null() {
            InvalidateRect(
                (*infoPtr).hwndEdit,
                ::core::ptr::null::<RECT>(),
                TRUE,
            );
        }
    }

    return dwTemp;
}

unsafe extern "C" fn COMBOEX_SetImageList(
    mut infoPtr: *mut COMBOEX_INFO,
    mut himl: HIMAGELIST,
) -> HIMAGELIST {
    let mut himlTemp: HIMAGELIST = (*infoPtr).himl;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
            734 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    (*infoPtr).himl = himl;
    COMBOEX_ReSize(infoPtr);
    InvalidateRect((*infoPtr).hwndCombo, core::ptr::null::<RECT>(), TRUE);
    COMBOEX_AdjustEditPos(infoPtr);
    return himlTemp;
}

unsafe extern "C" fn COMBOEX_SetItemW(mut infoPtr: *mut COMBOEX_INFO, mut cit: *const COMBOBOXEXITEMW) -> BOOL {
    let mut index: INT_PTR = (*cit).iItem;
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut::<CBE_ITEMDATA>();

    if TRACE_ON!(comboex) {
        COMBOEX_DumpInput(cit);
    }

    /* if item number requested does not exist then return failure */
    if (index >= (*infoPtr).nb_items as INT_PTR) || (index < -((1) as INT_PTR)) {
        return FALSE;
    }

    /* if the item is the edit control and there is no edit control, skip */
    if (index == -((1) as INT_PTR)) && (*infoPtr).hwndEdit.is_null() {
        return FALSE;
    }

    item = COMBOEX_FindItem(infoPtr, index);
    if item.is_null() {
        return FALSE;
    }

    /* add/change stuff to the internal item structure */
    (*item).mask |= (*cit).mask;
    if (*cit).mask & ((CBEIF_TEXT) as UINT) != 0 {
        let mut len: INT = (0) as INT;

        COMBOEX_FreeText(item);
        if is_textW((*cit).pszText as LPCWSTR) != 0 {
            len = lstrlenW((*cit).pszText as LPCWSTR) as INT;
        }
        if len > ((0) as INT) {
            (*item).pszText = Alloc(
                (((len + (1) as INT) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
            ) as LPWSTR;
            if (*item).pszText.is_null() {
                return FALSE;
            }
            lstrcpyW((*item).pszText, (*cit).pszText as LPCWSTR);
        } else if (*cit).pszText == ((LPSTR_TEXTCALLBACKW) as usize as LPWSTR) {
            (*item).pszText = (LPSTR_TEXTCALLBACKW) as usize as LPWSTR;
        }
        (*item).cchTextMax = (*cit).cchTextMax;
    }
    if (*cit).mask & ((CBEIF_IMAGE) as UINT) != 0 {
        (*item).iImage = (*cit).iImage;
    }
    if (*cit).mask & ((CBEIF_SELECTEDIMAGE) as UINT) != 0 {
        (*item).iSelectedImage = (*cit).iSelectedImage;
    }
    if (*cit).mask & ((CBEIF_OVERLAY) as UINT) != 0 {
        (*item).iOverlay = (*cit).iOverlay;
    }
    if (*cit).mask & ((CBEIF_INDENT) as UINT) != 0 {
        (*item).iIndent = (*cit).iIndent;
    }
    if (*cit).mask & ((CBEIF_LPARAM) as UINT) != 0 {
        (*item).lParam = (*cit).lParam;
    }

    if TRACE_ON!(comboex) {
        COMBOEX_DumpItem(item);
    }

    /* if original request was to update edit control, do some fast foot work */
    if (*cit).iItem == -((1) as INT_PTR)
        && (*cit).mask & ((CBEIF_TEXT) as UINT) != 0
    {
        COMBOEX_SetEditText(infoPtr, item);
        RedrawWindow(
            (*infoPtr).hwndCombo,
            ((0) as usize) as *const RECT,
            ((0) as usize) as *mut HRGN__,
            ((RDW_ERASE) | (RDW_INVALIDATE)) as UINT,
        );
    }
    return TRUE;
}

unsafe extern "C" fn COMBOEX_SetItemA(infoPtr: *mut COMBOEX_INFO, cit: *const COMBOBOXEXITEMA) -> BOOL {
    let mut citW: COMBOBOXEXITEMW = core::mem::zeroed();
    let mut wstr: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ret: BOOL = 0;

    memcpy(
        &raw mut citW as *mut core::ffi::c_void,
        cit as *const core::ffi::c_void,
        core::mem::size_of::<COMBOBOXEXITEMA>() as usize,
    );
    if ((*cit).mask & (CBEIF_TEXT as UINT)) != 0 && is_textA((*cit).pszText as LPCSTR) != 0 {
        let mut len: INT = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            (*cit).pszText as LPCSTR,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as core::ffi::c_int,
        ) as INT;
        wstr = Alloc(
            (((len + 1 as INT) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD,
        ) as LPWSTR;
        if wstr.is_null() {
            return FALSE as BOOL;
        }
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            (*cit).pszText as LPCSTR,
            -(1 as core::ffi::c_int),
            wstr,
            len as core::ffi::c_int,
        );
        citW.pszText = wstr;
    }
    ret = COMBOEX_SetItemW(infoPtr, &raw mut citW);

    Free(wstr as LPVOID);

    return ret;
}

unsafe extern "C" fn COMBOEX_SetUnicodeFormat(infoPtr: *mut COMBOEX_INFO, value: BOOL) -> BOOL {
    let mut bTemp: BOOL = (*infoPtr).unicode;

    (*infoPtr).unicode = value;

    return bTemp;
}

unsafe extern "C" fn COMBOEX_FindStringExact(infoPtr: *const COMBOEX_INFO, start: INT, str: LPCWSTR) -> INT {
    let mut i: INT = 0;
    let mut cmptext: cmp_func_t = get_cmp_func(infoPtr);
    let mut count: INT = SendMessageW(
        (*infoPtr).hwndCombo,
        CB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;

    /* now search from after starting loc and wrapping back to start */
    i = (start + 1) as INT;
    while i < count {
        let mut item: *mut CBE_ITEMDATA = get_item_data(infoPtr, i);
        if !((item as LRESULT) == (CB_ERR as LRESULT)) {
            if cmptext(COMBOEX_GetText(infoPtr, item), str) == 0 {
                return i;
            }
        }
        i += 1;
    }
    i = 0 as INT;
    while i <= start {
        let mut item_0: *mut CBE_ITEMDATA = get_item_data(infoPtr, i);
        if !((item_0 as LRESULT) == (CB_ERR as LRESULT)) {
            if cmptext(COMBOEX_GetText(infoPtr, item_0), str) == 0 {
                return i;
            }
        }
        i += 1;
    }
    return CB_ERR;
}

unsafe extern "C" fn COMBOEX_GetItemData(
    infoPtr: *mut COMBOEX_INFO,
    index: INT_PTR,
) -> DWORD_PTR {
    let mut item1: *const CBE_ITEMDATA = core::ptr::null::<CBE_ITEMDATA>();
    let mut item2: *const CBE_ITEMDATA = core::ptr::null::<CBE_ITEMDATA>();
    let mut ret: DWORD_PTR = 0 as DWORD_PTR;
    item1 = get_item_data(infoPtr, index as INT);
    if !item1.is_null() && item1 as LRESULT != CB_ERR as LRESULT {
        item2 = COMBOEX_FindItem(infoPtr, index);
        if item2 != item1 {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/comboex.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"COMBOEX_GetItemData\0" as *const u8 as *const core::ffi::c_char,
                    865 as core::ffi::c_int,
                    b"data structures damaged!\n\0" as *const u8
                        as *const core::ffi::c_char,
                );
            }
            return CB_ERR as DWORD_PTR;
        }
        if (*item1).mask & CBEIF_LPARAM as UINT != 0 {
            ret = (*item1).lParam as DWORD_PTR;
        }
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"COMBOEX_GetItemData\0" as *const u8 as *const core::ffi::c_char,
                869 as core::ffi::c_int,
                b"returning 0x%08lx\n\0" as *const u8 as *const core::ffi::c_char,
                ret,
            );
        }
    } else {
        ret = item1 as DWORD_PTR;
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"COMBOEX_GetItemData\0" as *const u8 as *const core::ffi::c_char,
                872 as core::ffi::c_int,
                b"non-valid result from combo, returning 0x%08lx\n\0" as *const u8
                    as *const core::ffi::c_char,
                ret,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn COMBOEX_SetCursel(infoPtr: *mut COMBOEX_INFO, index: INT_PTR) -> INT {
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut sel: INT = 0;
    item = COMBOEX_FindItem(infoPtr, index);
    if item.is_null() {
        return SendMessageW(
            (*infoPtr).hwndCombo,
            CB_SETCURSEL as UINT,
            index as WPARAM,
            0 as LPARAM,
        ) as INT;
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
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_SetCursel\0" as *const u8 as *const ::core::ffi::c_char,
            886 as ::core::ffi::c_int,
            b"selecting item %ld text=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            index,
            debugstr_txt((*item).pszText as LPCWSTR),
        );
    }
    (*infoPtr).selected = index as INT;
    sel = SendMessageW(
        (*infoPtr).hwndCombo,
        CB_SETCURSEL as UINT,
        index as WPARAM,
        0 as LPARAM,
    ) as INT;
    COMBOEX_SetEditText(infoPtr, item);
    return sel;
}

unsafe extern "C" fn COMBOEX_SetItemData(
    infoPtr: *mut COMBOEX_INFO,
    index: INT_PTR,
    data: DWORD_PTR,
) -> DWORD_PTR {
    let mut item1: *mut CBE_ITEMDATA = core::ptr::null_mut();
    let mut item2: *const CBE_ITEMDATA = core::ptr::null();

    item1 = get_item_data(infoPtr, index as INT);
    if !item1.is_null() && (item1 as LRESULT) != (CB_ERR as LRESULT) {
        item2 = COMBOEX_FindItem(infoPtr, index);
        if item2 != item1 as *const CBE_ITEMDATA {
            return CB_ERR as DWORD_PTR;
        }
        (*item1).mask |= CBEIF_LPARAM as UINT;
        (*item1).lParam = data as LPARAM;
        return 0 as DWORD_PTR;
    }
    return item1 as DWORD_PTR;
}

unsafe extern "C" fn COMBOEX_SetItemHeight(
    infoPtr: *const COMBOEX_INFO,
    index: INT,
    mut height: UINT,
) -> INT {
    let mut cb_wrect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut cbx_wrect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut cbx_crect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if !(*infoPtr).hwndCombo.is_null() {
        if SendMessageW(
            (*infoPtr).hwndCombo,
            CB_SETITEMHEIGHT as UINT,
            index as WPARAM,
            height as LPARAM,
        ) == CB_ERR as LRESULT
        {
            return CB_ERR;
        }
    }
    GetWindowRect((*infoPtr).hwndCombo, &raw mut cb_wrect);
    GetWindowRect((*infoPtr).hwndSelf, &raw mut cbx_wrect);
    GetClientRect((*infoPtr).hwndSelf, &raw mut cbx_crect);
    height = (cb_wrect.bottom - cb_wrect.top + cbx_wrect.bottom
        - cbx_wrect.top
        - (cbx_crect.bottom - cbx_crect.top)) as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_SetItemHeight\0" as *const u8 as *const ::core::ffi::c_char,
            935 as ::core::ffi::c_int,
            b"EX window=(%s), client=(%s)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wine_dbgstr_rect(&raw mut cbx_wrect),
            wine_dbgstr_rect(&raw mut cbx_crect),
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
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_SetItemHeight\0" as *const u8 as *const ::core::ffi::c_char,
            937 as ::core::ffi::c_int,
            b"CB window=(%s), EX setting=(0,0)-(%d,%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wine_dbgstr_rect(&raw mut cbx_wrect),
            cbx_wrect.right - cbx_wrect.left,
            height,
        );
    }
    SetWindowPos(
        (*infoPtr).hwndSelf,
        HWND_TOP,
        0,
        0,
        cbx_wrect.right as ::core::ffi::c_int - cbx_wrect.left as ::core::ffi::c_int,
        height as ::core::ffi::c_int,
        (SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOMOVE) as UINT,
    );
    return 0;
}

unsafe extern "C" fn COMBOEX_Create(hwnd: HWND, cs: *const CREATESTRUCTA) -> LRESULT {
    static mut NIL: [WCHAR; 1] = [0];
    let mut infoPtr: *mut COMBOEX_INFO = core::ptr::null_mut::<COMBOEX_INFO>();
    let mut mylogfont: LOGFONTW = core::mem::zeroed();
    let mut win_rect: RECT = core::mem::zeroed();
    let mut i: INT = 0;

    /* allocate memory for info structure */
    infoPtr = Alloc(core::mem::size_of::<COMBOEX_INFO>() as DWORD) as *mut COMBOEX_INFO;
    if infoPtr.is_null() {
        return -(1 as LRESULT);
    }

    /* initialize info structure */
    /* note that infoPtr is allocated zero-filled */

    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).selected = -(1 as INT);

    (*infoPtr).unicode = IsWindowUnicode(hwnd);
    (*infoPtr).hwndNotify = (*cs).hwndParent;

    i = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFYFORMAT as UINT,
        hwnd as WPARAM,
        NF_QUERY as LPARAM,
    ) as INT;
    if (i) as u32 != NFR_ANSI && (i) as u32 != NFR_UNICODE {
        i = (NFR_ANSI) as i32;
    }
    (*infoPtr).NtfUnicode = ((i) as u32 == NFR_UNICODE) as BOOL;

    SetWindowLongPtrW(hwnd, 0, (infoPtr as DWORD_PTR) as LONG_PTR);

    if TRACE_ON!(comboex) {
        let mut client: RECT = core::mem::zeroed();
        let mut rect: RECT = core::mem::zeroed();
        GetWindowRect(hwnd, &raw mut rect);
        GetClientRect(hwnd, &raw mut client);
    }

    /* Native version of ComboEx creates the ComboBox with DROPDOWNLIST */
    /* specified. It then creates its own version of the EDIT control   */
    /* and makes the ComboBox the parent. This is because a normal      */
    /* DROPDOWNLIST does not have an EDIT control, but we need one.     */
    /* We also need to place the edit control at the proper location    */
    /* (allow space for the icons).                                     */

    (*infoPtr).hwndCombo = CreateWindowW!(
        (WC_COMBOBOXW).as_ptr(),
        (NIL).as_ptr(),
        WS_CLIPSIBLINGS
            | WS_CLIPCHILDREN
            | WS_VSCROLL
            | CBS_NOINTEGRALHEIGHT
            | CBS_DROPDOWNLIST
            | WS_CHILD
            | WS_VISIBLE
            | CBS_OWNERDRAWFIXED
            | (GetWindowLongW(hwnd, GWL_STYLE)) as u32,
        (*cs).y,
        (*cs).x,
        (*cs).cx,
        (*cs).cy,
        hwnd,
        GetWindowLongPtrW(hwnd, GWLP_ID) as HMENU,
        GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
        core::ptr::null_mut(),
    );

    SetWindowSubclass(
        (*infoPtr).hwndCombo,
        Some(COMBOEX_ComboWndProc),
        COMBO_SUBCLASSID as UINT_PTR,
        hwnd as DWORD_PTR,
    );
    (*infoPtr).font = SendMessageW(
        (*infoPtr).hwndCombo,
        WM_GETFONT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HFONT;

    /*
     * Now create our own EDIT control so we can position it.
     * It is created only for CBS_DROPDOWN style
     */
    if (((*cs).style as u32 & CBS_DROPDOWNLIST)) == CBS_DROPDOWN {
        (*infoPtr).hwndEdit = CreateWindowExW(
            0 as DWORD,
            (WC_EDITW).as_ptr(),
            (NIL).as_ptr(),
            WS_CHILD
                | WS_VISIBLE
                | WS_CLIPSIBLINGS
                | ES_AUTOHSCROLL,
            0,
            0,
            0,
            0, /* will set later */
            (*infoPtr).hwndCombo,
            GetWindowLongPtrW(hwnd, GWLP_ID) as HMENU,
            GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
            core::ptr::null_mut(),
        );

        SetWindowSubclass(
            (*infoPtr).hwndEdit,
            Some(COMBOEX_EditWndProc),
            EDIT_SUBCLASSID as UINT_PTR,
            hwnd as DWORD_PTR,
        );

        (*infoPtr).font = SendMessageW(
            (*infoPtr).hwndCombo,
            WM_GETFONT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as HFONT;
    }

    /*
     * Locate the default font if necessary and then set it in
     * all associated controls
     */
    if (*infoPtr).font.is_null() {
        SystemParametersInfoW(
            SPI_GETICONTITLELOGFONT as UINT,
            core::mem::size_of_val(&mylogfont) as UINT,
            &raw mut mylogfont as PVOID,
            0 as UINT,
        );
        (*infoPtr).defaultFont = CreateFontIndirectW(&raw mut mylogfont);
        (*infoPtr).font = (*infoPtr).defaultFont;
    }
    SendMessageW(
        (*infoPtr).hwndCombo,
        WM_SETFONT as UINT,
        (*infoPtr).font as WPARAM,
        0 as LPARAM,
    );
    if !(*infoPtr).hwndEdit.is_null() {
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_SETFONT as UINT,
            (*infoPtr).font as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            (*infoPtr).hwndEdit,
            EM_SETMARGINS as UINT,
            EC_USEFONTINFO as WPARAM,
            0 as LPARAM,
        );
    }

    COMBOEX_ReSize(infoPtr);

    /* Above is fairly certain, below is much less certain. */

    GetWindowRect(hwnd, &raw mut win_rect);

    if TRACE_ON!(comboex) {
        let mut client: RECT = core::mem::zeroed();
        let mut rect: RECT = core::mem::zeroed();
        GetClientRect(hwnd, &raw mut client);
        GetWindowRect((*infoPtr).hwndCombo, &raw mut rect);
    }
    SetWindowPos(
        (*infoPtr).hwndCombo,
        HWND_TOP as HWND,
        0 as INT,
        0 as INT,
        (win_rect.right - win_rect.left) as INT,
        (win_rect.bottom - win_rect.top) as INT,
        (SWP_NOACTIVATE | SWP_NOREDRAW) as UINT,
    );

    GetWindowRect((*infoPtr).hwndCombo, &raw mut win_rect);

    SetWindowPos(
        hwnd,
        HWND_TOP as HWND,
        0 as INT,
        0 as INT,
        (win_rect.right - win_rect.left) as INT,
        (win_rect.bottom - win_rect.top) as INT,
        (SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOMOVE) as UINT,
    );

    COMBOEX_AdjustEditPos(infoPtr);

    return 0 as LRESULT;
}

unsafe extern "C" fn COMBOEX_Command(infoPtr: *mut COMBOEX_INFO, wParam: WPARAM) -> LRESULT {
    let mut lret: LRESULT = 0;
    let mut command: INT = HIWORD!(wParam) as INT;
    let mut item: *mut CBE_ITEMDATA = 0 as *mut CBE_ITEMDATA;
    let mut wintext: [WCHAR; 520] = [0; 520];
    let mut cursel: INT = 0;
    let mut n: INT = 0;
    let mut oldItem: INT_PTR = 0;
    let mut cbeend: NMCBEENDEDITW = core::mem::zeroed();
    let mut oldflags: DWORD = 0;
    let mut parent: HWND = (*infoPtr).hwndNotify;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_Command\0" as *const u8 as *const ::core::ffi::c_char,
            1082 as ::core::ffi::c_int,
            b"for command %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            command,
        );
    }

    match (command) as u32 {
        CBN_DROPDOWN => {
            SetFocus((*infoPtr).hwndCombo);
            ShowWindow((*infoPtr).hwndEdit, SW_HIDE as INT);
            (*infoPtr).flags |= WCBE_ACTEDIT as DWORD;
            return SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
        }
        CBN_CLOSEUP => {
            SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
            ShowWindow((*infoPtr).hwndEdit, SW_SHOW as INT);
            InvalidateRect(
                (*infoPtr).hwndCombo,
                0 as *const RECT,
                TRUE as BOOL,
            );
            if !(*infoPtr).hwndEdit.is_null() {
                InvalidateRect(
                    (*infoPtr).hwndEdit,
                    0 as *const RECT,
                    TRUE as BOOL,
                );
            }
            cursel = SendMessageW(
                (*infoPtr).hwndCombo,
                CB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            if cursel == -1 {
                let mut cmptext: cmp_func_t = get_cmp_func(infoPtr);
                /* find match from edit against those in Combobox */
                GetWindowTextW(
                    (*infoPtr).hwndEdit,
                    core::ptr::addr_of_mut!(wintext) as LPWSTR,
                    520 as INT,
                );
                n = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCOUNT as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT;
                cursel = 0 as INT;
                while cursel < n {
                    item = get_item_data(infoPtr, cursel);
                    if item as INT_PTR == CB_ERR as INT_PTR {
                        break;
                    }
                    if cmptext(
                        COMBOEX_GetText(infoPtr, item),
                        core::ptr::addr_of_mut!(wintext) as *mut WCHAR as LPCWSTR,
                    ) == 0
                    {
                        break;
                    }
                    cursel += 1;
                }
                if (cursel == n) || (item as INT_PTR == CB_ERR as INT_PTR) {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"dll/win32/comctl32/comboex.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"COMBOEX_Command\0" as *const u8 as *const ::core::ffi::c_char,
                            1109 as ::core::ffi::c_int,
                            b"failed to find match??? item=%p cursel=%d\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            item,
                            cursel,
                        );
                    }
                    if !(*infoPtr).hwndEdit.is_null() {
                        SetFocus((*infoPtr).hwndEdit);
                    }
                    return 0 as LRESULT;
                }
            } else {
                item = get_item_data(infoPtr, cursel);
                if item as INT_PTR == CB_ERR as INT_PTR {
                    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                        != 0
                    {
                        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_1,
                            __dbch_1,
                            b"dll/win32/comctl32/comboex.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"COMBOEX_Command\0" as *const u8 as *const ::core::ffi::c_char,
                            1118 as ::core::ffi::c_int,
                            b"failed to find match??? item=%p cursel=%d\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            item,
                            cursel,
                        );
                    }
                    if !(*infoPtr).hwndEdit.is_null() {
                        SetFocus((*infoPtr).hwndEdit);
                    }
                    return 0 as LRESULT;
                }
            }

            /* Save flags for testing and reset them */
            oldflags = (*infoPtr).flags;
            (*infoPtr).flags &= !(WCBE_ACTEDIT as DWORD | WCBE_EDITCHG as DWORD);

            if (oldflags & (WCBE_ACTEDIT as DWORD)) != 0 {
                cbeend.fChanged = ((oldflags & (WCBE_EDITCHG as DWORD)) as BOOL);
                cbeend.iNewSelection = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT;
                cbeend.iWhy = CBENF_DROPDOWN as INT;

                if COMBOEX_NotifyEndEdit(
                    infoPtr,
                    core::ptr::addr_of_mut!(cbeend),
                    COMBOEX_GetText(infoPtr, item),
                ) != 0
                {
                    return 0 as LRESULT;
                }
            }

            /* if selection has changed the set the new current selection */
            cursel = SendMessageW(
                (*infoPtr).hwndCombo,
                CB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            if ((oldflags & (WCBE_EDITCHG as DWORD)) != 0) || (cursel != (*infoPtr).selected) {
                (*infoPtr).selected = cursel;
                SendMessageW(
                    (*infoPtr).hwndSelf,
                    CB_SETCURSEL as UINT,
                    cursel as WPARAM,
                    0 as LPARAM,
                );
                SetFocus((*infoPtr).hwndCombo);
            }
            return 0 as LRESULT;
        }
        CBN_SELCHANGE => {
            /*
             * CB_GETCURSEL(Combo)
             * CB_GETITEMDATA(Combo)   < simulated by COMBOEX_FindItem
             * lstrlenA
             * WM_SETTEXT(Edit)
             * WM_GETTEXTLENGTH(Edit)
             * WM_GETTEXT(Edit)
             * EM_SETSEL(Edit, 0,0)
             * WM_GETTEXTLENGTH(Edit)
             * WM_GETTEXT(Edit)
             * EM_SETSEL(Edit, 0,len)
             * return WM_COMMAND to parent
             */
            oldItem = SendMessageW(
                (*infoPtr).hwndCombo,
                CB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT_PTR;
            item = COMBOEX_FindItem(infoPtr, oldItem);
            if item.is_null() {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/comctl32/comboex.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"COMBOEX_Command\0" as *const u8 as *const ::core::ffi::c_char,
                        1162 as ::core::ffi::c_int,
                        b"item %ld not found. Problem!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        oldItem,
                    );
                }
            } else {
                (*infoPtr).selected = oldItem as INT;
                COMBOEX_SetEditText(infoPtr, item);
                return SendMessageW(
                    parent,
                    WM_COMMAND as UINT,
                    wParam,
                    (*infoPtr).hwndSelf as LPARAM,
                );
            }
        }
        CBN_SELENDOK | CBN_SELENDCANCEL => {
            /*
             * We have to change the handle since we are the control
             * issuing the message. IE4 depends on this.
             */
            return SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
        }
        CBN_KILLFOCUS => {
            SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
            if ((*infoPtr).flags & (WCBE_ACTEDIT as DWORD)) != 0 {
                GetWindowTextW(
                    (*infoPtr).hwndEdit,
                    core::ptr::addr_of_mut!(wintext) as LPWSTR,
                    260 as INT,
                );
                cbeend.fChanged = (((*infoPtr).flags & (WCBE_EDITCHG as DWORD)) as BOOL);
                cbeend.iNewSelection = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT;
                cbeend.iWhy = CBENF_KILLFOCUS as INT;

                (*infoPtr).flags &= !(WCBE_ACTEDIT as DWORD | WCBE_EDITCHG as DWORD);
                if COMBOEX_NotifyEndEdit(
                    infoPtr,
                    core::ptr::addr_of_mut!(cbeend),
                    core::ptr::addr_of_mut!(wintext) as *mut WCHAR as LPCWSTR,
                ) != 0
                {
                    return 0 as LRESULT;
                }
            }
            /* possible CB_GETCURSEL */
            InvalidateRect(
                (*infoPtr).hwndCombo,
                0 as *const RECT,
                0 as BOOL,
            );
            return 0 as LRESULT;
        }
        CBN_SETFOCUS => {
            return SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
        }
        _ => {
            /*
             * We have to change the handle since we are the control
             * issuing the message. IE4 depends on this.
             * We also need to set the focus back to the Edit control
             * after passing the command to the parent of the ComboEx.
             */
            lret = SendMessageW(
                parent,
                WM_COMMAND as UINT,
                wParam,
                (*infoPtr).hwndSelf as LPARAM,
            );
            if !(*infoPtr).hwndEdit.is_null() {
                SetFocus((*infoPtr).hwndEdit);
            }
            return lret;
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn COMBOEX_WM_DeleteItem(
    mut infoPtr: *mut COMBOEX_INFO,
    mut dis: *const DELETEITEMSTRUCT,
) -> BOOL {
    let mut item: *mut CBE_ITEMDATA = ::core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut olditem: *mut CBE_ITEMDATA = ::core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut nmcit: NMCOMBOBOXEXW = NMCOMBOBOXEXW {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        ceItem: tagCOMBOBOXEXITEMW {
            mask: 0,
            iItem: 0,
            pszText: ::core::ptr::null_mut::<WCHAR>(),
            cchTextMax: 0,
            iImage: 0,
            iSelectedImage: 0,
            iOverlay: 0,
            iIndent: 0,
            lParam: 0,
        },
    };
    let mut i: UINT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_WM_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            1218 as ::core::ffi::c_int,
            b"CtlType=%08x, CtlID=%08x, itemID=%08x, hwnd=%p, data=%08lx\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            (*dis).CtlType,
            (*dis).CtlID,
            (*dis).itemID,
            (*dis).hwndItem,
            (*dis).itemData,
        );
    }
    if (*dis).itemID >= (*infoPtr).nb_items as UINT {
        return FALSE;
    }
    olditem = (*infoPtr).items;
    i = ((*infoPtr).nb_items as ::core::ffi::c_int - 1) as UINT;
    if i == (*dis).itemID {
        (*infoPtr).items = (*(*infoPtr).items).next as *mut CBE_ITEMDATA;
    } else {
        item = olditem;
        i = i.wrapping_sub(1);
        while !(*item).next.is_null() && i > (*dis).itemID {
            item = (*item).next as *mut CBE_ITEMDATA;
            i = i.wrapping_sub(1);
        }
        if (*item).next.is_null() || i != (*dis).itemID {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/comboex.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"COMBOEX_WM_DeleteItem\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1238 as ::core::ffi::c_int,
                    b"COMBOBOXEX item structures broken. Please report!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return FALSE;
        }
        olditem = (*item).next as *mut CBE_ITEMDATA;
        (*item).next = (*(*item).next).next;
    }
    (*infoPtr).nb_items -= 1;
    memset(
        &raw mut nmcit.ceItem as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<COMBOBOXEXITEMW>() as size_t,
    );
    nmcit.ceItem.mask = !(0 as ::core::ffi::c_int) as UINT;
    COMBOEX_CopyItem(olditem, &raw mut nmcit.ceItem);
    COMBOEX_NotifyItem(infoPtr, CBEN_DELETEITEM as UINT, &raw mut nmcit);
    COMBOEX_FreeText(olditem);
    Free(olditem as LPVOID);
    return TRUE;
}

unsafe extern "C" fn COMBOEX_DrawItem(
    mut infoPtr: *mut COMBOEX_INFO,
    mut dis: *const DRAWITEMSTRUCT,
) -> LRESULT {
    static mut nil: [WCHAR; 1] = [0 as WCHAR];
    let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut::<CBE_ITEMDATA>();
    let mut txtsize: SIZE = tagSIZE { cx: 0, cy: 0 };
    let mut rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut str: LPCWSTR = &raw const nil as LPCWSTR;
    let mut xbase: UINT = 0;
    let mut x: UINT = 0;
    let mut y: UINT = 0;
    let mut len: INT = 0;
    let mut nbkc: COLORREF = 0;
    let mut ntxc: COLORREF = 0;
    let mut bkc: COLORREF = 0;
    let mut txc: COLORREF = 0;
    let mut drawimage: ::core::ffi::c_int = 0;
    let mut drawstate: ::core::ffi::c_int = 0;
    let mut xioff: ::core::ffi::c_int = 0;
    let mut selected: ::core::ffi::c_int = 0;

    if (*dis).itemID == (0xffffffff as UINT) {
        if (((*dis).itemAction & (ODA_FOCUS as UINT)) != 0
            && ((*dis).itemState & (ODS_SELECTED as UINT)) != 0)
            || (((*dis).itemAction & ((ODA_SELECT | ODA_DRAWENTIRE) as UINT)) != 0
                && ((*dis).itemState & (ODS_FOCUS as UINT)) != 0)
        {
            // empty
        } else if ((*dis).CtlType == (ODT_COMBOBOX as UINT))
            && ((*dis).itemAction == (ODA_DRAWENTIRE as UINT))
        {
            // draw of edit control data
            if TRACE_ON!(comboex) {
                let mut exrc: RECT = tagRECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                let mut cbrc: RECT = tagRECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                let mut edrc: RECT = tagRECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                GetWindowRect((*infoPtr).hwndSelf, &raw mut exrc);
                GetWindowRect((*infoPtr).hwndCombo, &raw mut cbrc);
                SetRect(
                    &raw mut edrc,
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                );
                if !(*infoPtr).hwndEdit.is_null() {
                    GetWindowRect((*infoPtr).hwndEdit, &raw mut edrc);
                }
            }
        } else {
            return 0 as LRESULT;
        }
    }

    // If draw item is -1 (edit control) setup the item pointer
    if (*dis).itemID == (0xffffffff as UINT) {
        item = &raw mut (*infoPtr).edit;

        if !(*infoPtr).hwndEdit.is_null() {
            // free previous text of edit item
            COMBOEX_FreeText(item);
            (*item).mask &= !(CBEIF_TEXT as UINT);
            len = GetWindowTextLengthW((*infoPtr).hwndEdit) as INT;
            if len != 0 {
                (*item).mask |= CBEIF_TEXT as UINT;
                (*item).pszText = Alloc(
                    ((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                        as DWORD,
                ) as LPWSTR;
                if !(*item).pszText.is_null() {
                    GetWindowTextW(
                        (*infoPtr).hwndEdit,
                        (*item).pszText,
                        len as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                    );
                }
            }
        }
    }

    // if the item pointer is not set, then get the data and locate it
    if item.is_null() {
        item = get_item_data(infoPtr, (*dis).itemID as INT);
        if item == (CB_ERR as *mut CBE_ITEMDATA) {
            return 0 as LRESULT;
        }
    }

    if TRACE_ON!(comboex) {
        COMBOEX_DumpItem(item);
    }

    xbase = CBE_STARTOFFSET as UINT;
    if ((*item).mask & (CBEIF_INDENT as UINT)) != 0
        && ((*dis).itemState & (ODS_COMBOEXLBOX as UINT)) != 0
    {
        let mut indent: INT = (*item).iIndent as INT;
        if indent == (I_INDENTCALLBACK as INT) {
            let mut nmce: NMCOMBOBOXEXW = NMCOMBOBOXEXW {
                hdr: tagNMHDR {
                    hwndFrom: core::ptr::null_mut::<HWND__>(),
                    idFrom: 0,
                    code: 0,
                },
                ceItem: tagCOMBOBOXEXITEMW {
                    mask: 0,
                    iItem: 0,
                    pszText: core::ptr::null_mut::<WCHAR>(),
                    cchTextMax: 0,
                    iImage: 0,
                    iSelectedImage: 0,
                    iOverlay: 0,
                    iIndent: 0,
                    lParam: 0,
                },
            };
            ZeroMemory!(
                &raw mut nmce as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<NMCOMBOBOXEXW>() as size_t,
            );
            nmce.ceItem.mask = CBEIF_INDENT as UINT;
            nmce.ceItem.lParam = (*item).lParam;
            nmce.ceItem.iItem = (*dis).itemID as INT_PTR;
            COMBOEX_NotifyItem(infoPtr, CBEN_GETDISPINFOW as UINT, &raw mut nmce);
            if (nmce.ceItem.mask & (CBEIF_DI_SETITEM as UINT)) != 0 {
                (*item).iIndent = nmce.ceItem.iIndent;
            }
            indent = nmce.ceItem.iIndent as INT;
        }
        xbase = xbase.wrapping_add(
            ((indent as ::core::ffi::c_int) * (CBE_INDENT as ::core::ffi::c_int)) as UINT,
        );
    }

    drawimage = -(2 as ::core::ffi::c_int);
    drawstate = ILD_NORMAL as ::core::ffi::c_int;
    selected = ((*infoPtr).selected as UINT == (*dis).itemID) as ::core::ffi::c_int;

    if ((*item).mask & (CBEIF_IMAGE as UINT)) != 0 {
        drawimage = (*item).iImage;
    }
    if ((*item).mask & (CBEIF_SELECTEDIMAGE as UINT)) != 0 && selected != 0 {
        drawimage = (*item).iSelectedImage;
    }
    if ((*dis).itemState & (ODS_COMBOEXLBOX as UINT)) != 0 {
        // drawing listbox entry
        if ((*dis).itemState & (ODS_SELECTED as UINT)) != 0 {
            drawstate = ILD_SELECTED as ::core::ffi::c_int;
        }
    } else {
        // drawing combo/edit entry
        if IsWindowVisible((*infoPtr).hwndEdit) != 0 {
            // if we have an edit control, the slave the
            // selection state to the Edit focus state
            if ((*infoPtr).flags & (WCBE_EDITFOCUSED as DWORD)) != 0 {
                drawstate = ILD_SELECTED as ::core::ffi::c_int;
            }
        } else {
            // if we don't have an edit control, use
            // the requested state.
            if ((*dis).itemState & (ODS_SELECTED as UINT)) != 0 {
                drawstate = ILD_SELECTED as ::core::ffi::c_int;
            }
        }
    }

    if !(*infoPtr).himl.is_null()
        && ((*infoPtr).dwExtStyle & (CBES_EX_NOEDITIMAGEINDENT as DWORD)) == 0
    {
        let mut iinfo: IMAGEINFO = _IMAGEINFO {
            hbmImage: core::ptr::null_mut::<HBITMAP__>(),
            hbmMask: core::ptr::null_mut::<HBITMAP__>(),
            Unused1: 0,
            Unused2: 0,
            rcImage: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        iinfo.rcImage.right = 0 as LONG;
        iinfo.rcImage.left = iinfo.rcImage.right;
        ImageList_GetImageInfo(
            (*infoPtr).himl,
            0 as ::core::ffi::c_int,
            &raw mut iinfo,
        );
        xioff = iinfo.rcImage.right as ::core::ffi::c_int
            - iinfo.rcImage.left as ::core::ffi::c_int
            + CBE_SEP as ::core::ffi::c_int;
    } else {
        xioff = 0 as ::core::ffi::c_int;
    }

    // setup pointer to text to be drawn
    str = COMBOEX_GetText(infoPtr, item);
    if str.is_null() {
        str = &raw const nil as *const WCHAR as LPCWSTR;
    }

    len = lstrlenW(str) as INT;
    GetTextExtentPoint32W((*dis).hDC, str, len as ::core::ffi::c_int, &raw mut txtsize);

    if ((*dis).itemAction & ((ODA_SELECT | ODA_DRAWENTIRE) as UINT)) != 0 {
        let mut overlay: ::core::ffi::c_int = (*item).iOverlay;

        if drawimage == (I_IMAGECALLBACK as ::core::ffi::c_int) {
            let mut nmce_0: NMCOMBOBOXEXW = NMCOMBOBOXEXW {
                hdr: tagNMHDR {
                    hwndFrom: core::ptr::null_mut::<HWND__>(),
                    idFrom: 0,
                    code: 0,
                },
                ceItem: tagCOMBOBOXEXITEMW {
                    mask: 0,
                    iItem: 0,
                    pszText: core::ptr::null_mut::<WCHAR>(),
                    cchTextMax: 0,
                    iImage: 0,
                    iSelectedImage: 0,
                    iOverlay: 0,
                    iIndent: 0,
                    lParam: 0,
                },
            };
            ZeroMemory!(
                &raw mut nmce_0 as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<NMCOMBOBOXEXW>() as size_t,
            );
            nmce_0.ceItem.mask = (if selected != 0 {
                CBEIF_SELECTEDIMAGE
            } else {
                CBEIF_IMAGE
            }) as UINT;
            nmce_0.ceItem.lParam = (*item).lParam;
            nmce_0.ceItem.iItem = (*dis).itemID as INT_PTR;
            COMBOEX_NotifyItem(infoPtr, CBEN_GETDISPINFOW as UINT, &raw mut nmce_0);
            if selected == 0 {
                if (nmce_0.ceItem.mask & (CBEIF_DI_SETITEM as UINT)) != 0 {
                    (*item).iImage = nmce_0.ceItem.iImage;
                }
                drawimage = nmce_0.ceItem.iImage;
            } else {
                if (nmce_0.ceItem.mask & (CBEIF_DI_SETITEM as UINT)) != 0 {
                    (*item).iSelectedImage = nmce_0.ceItem.iSelectedImage;
                }
                drawimage = nmce_0.ceItem.iSelectedImage;
            }
        }

        if overlay == (I_IMAGECALLBACK as ::core::ffi::c_int) {
            let mut nmce_1: NMCOMBOBOXEXW = NMCOMBOBOXEXW {
                hdr: tagNMHDR {
                    hwndFrom: core::ptr::null_mut::<HWND__>(),
                    idFrom: 0,
                    code: 0,
                },
                ceItem: tagCOMBOBOXEXITEMW {
                    mask: 0,
                    iItem: 0,
                    pszText: core::ptr::null_mut::<WCHAR>(),
                    cchTextMax: 0,
                    iImage: 0,
                    iSelectedImage: 0,
                    iOverlay: 0,
                    iIndent: 0,
                    lParam: 0,
                },
            };
            ZeroMemory!(
                &raw mut nmce_1 as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<NMCOMBOBOXEXW>() as size_t,
            );
            nmce_1.ceItem.mask = CBEIF_OVERLAY as UINT;
            nmce_1.ceItem.lParam = (*item).lParam;
            nmce_1.ceItem.iItem = (*dis).itemID as INT_PTR;
            COMBOEX_NotifyItem(infoPtr, CBEN_GETDISPINFOW as UINT, &raw mut nmce_1);
            if (nmce_1.ceItem.mask & (CBEIF_DI_SETITEM as UINT)) != 0 {
                (*item).iOverlay = nmce_1.ceItem.iOverlay;
            }
            overlay = nmce_1.ceItem.iOverlay;
        }

        if drawimage >= (0 as ::core::ffi::c_int)
            && ((*infoPtr).dwExtStyle
                & ((CBES_EX_NOEDITIMAGE | CBES_EX_NOEDITIMAGEINDENT) as DWORD))
                == 0
        {
            if overlay > (0 as ::core::ffi::c_int) {
                ImageList_SetOverlayImage(
                    (*infoPtr).himl,
                    overlay,
                    1 as ::core::ffi::c_int,
                );
            }
            ImageList_Draw(
                (*infoPtr).himl,
                drawimage,
                (*dis).hDC,
                xbase as ::core::ffi::c_int,
                (*dis).rcItem.top as ::core::ffi::c_int,
                (drawstate
                    | (if overlay > (0 as ::core::ffi::c_int) {
                        INDEXTOOVERLAYMASK!(1 as ::core::ffi::c_int)
                    } else {
                        0 as ::core::ffi::c_int
                    })) as UINT,
            );
        }

        // now draw the text
        if IsWindowVisible((*infoPtr).hwndEdit) == 0 {
            nbkc = if ((*dis).itemState & (ODS_SELECTED as UINT)) != 0 {
                comctl32_color.clrHighlight
            } else {
                comctl32_color.clrWindow
            };
            bkc = SetBkColor((*dis).hDC, nbkc);
            ntxc = if ((*dis).itemState & (ODS_SELECTED as UINT)) != 0 {
                comctl32_color.clrHighlightText
            } else {
                comctl32_color.clrWindowText
            };
            txc = SetTextColor((*dis).hDC, ntxc);
            x = xbase.wrapping_add(xioff as UINT);
            y = ((*dis).rcItem.top as ::core::ffi::c_int
                + ((*dis).rcItem.bottom as ::core::ffi::c_int
                    - (*dis).rcItem.top as ::core::ffi::c_int
                    - txtsize.cy as ::core::ffi::c_int)
                    / (2 as ::core::ffi::c_int)) as UINT;
            SetRect(
                &raw mut rect,
                x as ::core::ffi::c_int,
                (*dis).rcItem.top as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                x.wrapping_add(txtsize.cx as UINT) as ::core::ffi::c_int,
                (*dis).rcItem.bottom as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            );
            ExtTextOutW(
                (*dis).hDC,
                x as ::core::ffi::c_int,
                y as ::core::ffi::c_int,
                ((ETO_OPAQUE | ETO_CLIPPED) as UINT),
                &raw mut rect,
                str,
                len as UINT,
                (0 as *const INT),
            );
            SetBkColor((*dis).hDC, bkc);
            SetTextColor((*dis).hDC, txc);
        }
    }

    if ((*dis).itemAction & (ODA_FOCUS as UINT)) != 0 {
        rect.left = xbase
            .wrapping_add(xioff as UINT)
            .wrapping_sub(1 as UINT) as LONG;
        rect.right = (rect.left as ::core::ffi::c_int
            + txtsize.cx as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as LONG;
        rect.top = (*dis).rcItem.top;
        rect.bottom = (*dis).rcItem.bottom;
        DrawFocusRect((*dis).hDC, &raw mut rect);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn COMBOEX_ResetContent(infoPtr: *mut COMBOEX_INFO) {
    if !(*infoPtr).items.is_null() {
        let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut();
        let mut next: *mut CBE_ITEMDATA = core::ptr::null_mut();

        item = (*infoPtr).items;
        while !item.is_null() {
            next = (*item).next;
            COMBOEX_FreeText(item);
            Free(item as LPVOID);
            item = next;
        }
        (*infoPtr).items = core::ptr::null_mut();
    }

    (*infoPtr).selected = -1;
    (*infoPtr).nb_items = 0;
}

unsafe extern "C" fn COMBOEX_Destroy(infoPtr: *mut COMBOEX_INFO) -> LRESULT {
    if !(*infoPtr).hwndCombo.is_null() {
        SetWindowSubclass(
            (*infoPtr).hwndCombo,
            Some(
                COMBOEX_ComboWndProc
                    as unsafe extern "system" fn(
                        HWND,
                        UINT,
                        WPARAM,
                        LPARAM,
                        UINT_PTR,
                        DWORD_PTR,
                    ) -> LRESULT,
            ),
            COMBO_SUBCLASSID as UINT_PTR,
            0 as DWORD_PTR,
        );
    }

    if !(*infoPtr).hwndEdit.is_null() {
        SetWindowSubclass(
            (*infoPtr).hwndEdit,
            Some(
                COMBOEX_EditWndProc
                    as unsafe extern "system" fn(
                        HWND,
                        UINT,
                        WPARAM,
                        LPARAM,
                        UINT_PTR,
                        DWORD_PTR,
                    ) -> LRESULT,
            ),
            EDIT_SUBCLASSID as UINT_PTR,
            0 as DWORD_PTR,
        );
    }

    COMBOEX_FreeText(core::ptr::addr_of_mut!((*infoPtr).edit));
    COMBOEX_ResetContent(infoPtr);

    if !(*infoPtr).defaultFont.is_null() {
        DeleteObject((*infoPtr).defaultFont as HGDIOBJ);
    }

    SetWindowLongPtrW(
        (*infoPtr).hwndSelf,
        0 as ::core::ffi::c_int,
        0 as LONG_PTR,
    );

    /* free comboex info data */
    Free(infoPtr as LPVOID);

    return 0 as LRESULT;
}

unsafe extern "C" fn COMBOEX_Enable(mut infoPtr: *mut COMBOEX_INFO, mut enable: BOOL) -> LRESULT {
    if !(*infoPtr).hwndEdit.is_null() {
        EnableWindow((*infoPtr).hwndEdit, enable);
    }

    EnableWindow((*infoPtr).hwndCombo, enable);

    /* Force the control to repaint when the enabled state changes. */
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);

    return 1;
}

unsafe extern "C" fn COMBOEX_MeasureItem(infoPtr: *const COMBOEX_INFO, mis: *mut MEASUREITEMSTRUCT) -> LRESULT {
    static strW: [WCHAR; 2] = [0x0057u16, 0x0000];
    let mut mysize: SIZE = core::mem::zeroed();
    let mut hdc: HDC;

    hdc = GetDC(core::ptr::null_mut());
    GetTextExtentPointW(hdc, core::ptr::addr_of!(strW) as LPCWSTR, 1, core::ptr::addr_of_mut!(mysize));
    ReleaseDC(core::ptr::null_mut(), hdc);
    (*mis).itemHeight = (mysize.cy + CBE_EXTRA) as UINT;

    return 0;
}

unsafe extern "C" fn COMBOEX_NCCreate(hwnd: HWND) -> LRESULT {
    let mut oldstyle: DWORD = 0;
    let mut newstyle: DWORD = 0;
    oldstyle = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    newstyle = oldstyle & !(WS_VSCROLL | WS_HSCROLL | WS_BORDER) as DWORD;
    if newstyle != oldstyle {
        SetWindowLongW(hwnd, GWL_STYLE, newstyle as LONG);
    }
    return 1 as LRESULT;
}

unsafe extern "C" fn COMBOEX_NotifyFormat(infoPtr: *mut COMBOEX_INFO, lParam: LPARAM) -> LRESULT {
    if lParam == NF_REQUERY as LPARAM {
        let mut i: INT = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFYFORMAT as UINT,
            (*infoPtr).hwndSelf as WPARAM,
            NF_QUERY as LPARAM,
        ) as INT;
        (*infoPtr).NtfUnicode = (i == NFR_UNICODE as INT) as BOOL;
    }
    return (if (*infoPtr).NtfUnicode != 0 {
        NFR_UNICODE
    } else {
        NFR_ANSI
    }) as LRESULT;
}

unsafe extern "C" fn COMBOEX_Size(
    mut infoPtr: *mut COMBOEX_INFO,
    mut width: INT,
    mut height: INT,
) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_Size\0" as *const u8 as *const ::core::ffi::c_char,
            1587 as ::core::ffi::c_int,
            b"(width=%d, height=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            width,
            height,
        );
    }
    MoveWindow(
        (*infoPtr).hwndCombo,
        0,
        0,
        width,
        height,
        TRUE,
    );
    COMBOEX_AdjustEditPos(infoPtr);
    return 0;
}

unsafe extern "C" fn COMBOEX_SetFont(infoPtr: *mut COMBOEX_INFO, font: HFONT, redraw: BOOL) -> LRESULT {
    (*infoPtr).font = font;
    SendMessageW(
        (*infoPtr).hwndCombo,
        WM_SETFONT as UINT,
        font as WPARAM,
        0 as LPARAM,
    );
    if !(*infoPtr).hwndEdit.is_null() {
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_SETFONT as UINT,
            font as WPARAM,
            0 as LPARAM,
        );
    }
    COMBOEX_ReSize(infoPtr);
    if redraw != 0 {
        InvalidateRect(
            (*infoPtr).hwndCombo,
            ::core::ptr::null::<RECT>(),
            TRUE,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn COMBOEX_SetRedraw(infoPtr: *const COMBOEX_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut ret: LRESULT = DefWindowProcW((*infoPtr).hwndSelf, WM_SETREDRAW, wParam, lParam);
    if wParam != 0 {
        RedrawWindow((*infoPtr).hwndSelf, core::ptr::null_mut(), core::ptr::null_mut(), RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
    }
    return ret;
}

unsafe extern "C" fn COMBOEX_WindowPosChanging(
    infoPtr: *const COMBOEX_INFO,
    wp: *mut WINDOWPOS,
) -> LRESULT {
    let mut cbx_wrect: RECT = core::mem::zeroed();
    let mut cbx_crect: RECT = core::mem::zeroed();
    let mut cb_wrect: RECT = core::mem::zeroed();
    let mut width: INT = 0;
    let mut height: INT = 0;

    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(cbx_wrect));
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(cbx_crect));
    GetWindowRect((*infoPtr).hwndCombo, core::ptr::addr_of_mut!(cb_wrect));

    /* width is winpos value + border width of comboex */
    width = ((*wp).cx as LONG + (cbx_wrect.right - cbx_wrect.left)
        - (cbx_crect.right - cbx_crect.left)) as INT;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_WindowPosChanging\0" as *const u8 as *const ::core::ffi::c_char,
            1629 as ::core::ffi::c_int,
            b"winpos=(%d,%d %dx%d) flags=0x%08x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*wp).x,
            (*wp).y,
            (*wp).cx,
            (*wp).cy,
            (*wp).flags,
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
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_WindowPosChanging\0" as *const u8 as *const ::core::ffi::c_char,
            1631 as ::core::ffi::c_int,
            b"EX window=(%s), client=(%s)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(cbx_wrect)),
            wine_dbgstr_rect(core::ptr::addr_of_mut!(cbx_crect)),
        );
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_WindowPosChanging\0" as *const u8 as *const ::core::ffi::c_char,
            1633 as ::core::ffi::c_int,
            b"CB window=(%s), EX setting=(0,0)-(%d,%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(cbx_wrect)),
            width,
            cb_wrect.bottom - cb_wrect.top,
        );
    }

    if width != 0 {
        SetWindowPos(
            (*infoPtr).hwndCombo,
            HWND_TOP,
            0,
            0,
            width,
            cb_wrect.bottom - cb_wrect.top,
            SWP_NOACTIVATE,
        );
    }

    GetWindowRect((*infoPtr).hwndCombo, core::ptr::addr_of_mut!(cb_wrect));

    /* height is combo window height plus border width of comboex */
    height = ((cb_wrect.bottom - cb_wrect.top)
        + (cbx_wrect.bottom - cbx_wrect.top)
        - (cbx_crect.bottom - cbx_crect.top)) as INT;
    (*wp).cy = height;
    if !(*infoPtr).hwndEdit.is_null() {
        COMBOEX_AdjustEditPos(infoPtr);
        InvalidateRect(
            (*infoPtr).hwndCombo,
            core::ptr::null(),
            TRUE,
        );
    }

    return 0;
}
unsafe extern "system" fn COMBOEX_EditWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uId: UINT_PTR,
    ref_data: DWORD_PTR,
) -> LRESULT {
    let mut infoPtr: *mut COMBOEX_INFO = COMBOEX_GetInfoPtr!(ref_data as HWND);
    let mut cbeend: NMCBEENDEDITW = core::mem::zeroed();
    let mut edit_text: [WCHAR; 260 as usize] =
        [0 as WCHAR; 260 as usize];
    let mut obkc: COLORREF = 0;
    let mut hDC: HDC = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut lret: LRESULT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
            1668 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lParam=%lx, info_ptr=%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
            infoPtr,
        );
    }

    if uMsg == WM_NCDESTROY as UINT {
        RemoveWindowSubclass(
            hwnd,
            Some(
                COMBOEX_EditWndProc,
            ),
            EDIT_SUBCLASSID as UINT_PTR,
        );
    }

    if infoPtr.is_null() {
        return DefSubclassProc(hwnd, uMsg, wParam, lParam);
    }

    // switch (uMsg) -- each case below is an if; cases that fall through are noted
    if uMsg == WM_CHAR as UINT {
        /* handle (ignore) the return character */
        if wParam == VK_RETURN as WPARAM {
            return 0 as LRESULT;
        }
        /* all other characters pass into the real Edit */
        return DefSubclassProc(hwnd, uMsg, wParam, lParam);
    }

    if uMsg == WM_ERASEBKGND as UINT {
        hDC = wParam as HDC;
        obkc = SetBkColor(hDC, comctl32_color.clrWindow);
        GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/comboex.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
                1689 as ::core::ffi::c_int,
                b"erasing (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                wine_dbgstr_rect(core::ptr::addr_of_mut!(rect)),
            );
        }
        ExtTextOutW(
            hDC,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            ETO_OPAQUE as UINT,
            core::ptr::addr_of_mut!(rect),
            0 as LPCWSTR,
            0 as UINT,
            0 as *const INT,
        );
        SetBkColor(hDC, obkc);
        return DefSubclassProc(hwnd, uMsg, wParam, lParam);
    }

    if uMsg == WM_KEYDOWN as UINT {
        let mut oldItem: INT_PTR = 0;
        let mut selected: INT_PTR = 0;
        let mut item: *mut CBE_ITEMDATA = core::ptr::null_mut();

        'inner: {
            if (wParam as INT) == (VK_ESCAPE as INT) {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/comboex.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
                        1701 as ::core::ffi::c_int,
                        b"special code for VK_ESCAPE\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }

                GetWindowTextW(
                    (*infoPtr).hwndEdit,
                    core::ptr::addr_of_mut!(edit_text) as LPWSTR,
                    260 as ::core::ffi::c_int,
                );

                (*infoPtr).flags &= !((WCBE_ACTEDIT | WCBE_EDITCHG) as DWORD);
                cbeend.fChanged = FALSE as BOOL;
                cbeend.iNewSelection = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT;
                cbeend.iWhy = CBENF_ESCAPE as INT;

                if COMBOEX_NotifyEndEdit(
                    infoPtr,
                    core::ptr::addr_of_mut!(cbeend),
                    core::ptr::addr_of_mut!(edit_text) as LPCWSTR,
                ) != 0
                {
                    return 0 as LRESULT;
                }
                oldItem = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT_PTR;
                InvalidateRect(
                    (*infoPtr).hwndCombo,
                    core::ptr::null::<RECT>(),
                    0 as BOOL,
                );
                item = COMBOEX_FindItem(infoPtr, oldItem);
                if item.is_null() {
                    break 'inner;
                }
                (*infoPtr).selected = oldItem as INT;
                COMBOEX_SetEditText(infoPtr, item);
                RedrawWindow(
                    (*infoPtr).hwndCombo,
                    core::ptr::null::<RECT>(),
                    core::ptr::null_mut(),
                    (RDW_ERASE | RDW_INVALIDATE) as UINT,
                );
                break 'inner;
            }

            if (wParam as INT) == (VK_RETURN as INT) {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/comboex.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
                        1725 as ::core::ffi::c_int,
                        b"special code for VK_RETURN\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }

                GetWindowTextW(
                    (*infoPtr).hwndEdit,
                    core::ptr::addr_of_mut!(edit_text) as LPWSTR,
                    260 as ::core::ffi::c_int,
                );

                (*infoPtr).flags &= !((WCBE_ACTEDIT | WCBE_EDITCHG) as DWORD);
                selected = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT_PTR;

                if selected != -(1 as INT_PTR) {
                    let mut cmptext: cmp_func_t = get_cmp_func(infoPtr);
                    item = COMBOEX_FindItem(infoPtr, selected);
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/win32/comctl32/comboex.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
                            1737 as ::core::ffi::c_int,
                            b"handling VK_RETURN, selected = %ld, selected_text=%s\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            selected,
                            debugstr_txt((*item).pszText as LPCWSTR),
                        );
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
                            b"dll/win32/comctl32/comboex.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"COMBOEX_EditWndProc\0" as *const u8 as *const ::core::ffi::c_char,
                            1739 as ::core::ffi::c_int,
                            b"handling VK_RETURN, edittext=%s\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            debugstr_w(core::ptr::addr_of_mut!(edit_text) as *mut WCHAR),
                        );
                    }
                    if cmptext(
                        COMBOEX_GetText(infoPtr, item),
                        core::ptr::addr_of_mut!(edit_text) as *mut WCHAR as LPCWSTR,
                    ) != 0
                    {
                        /* strings not equal -- indicate edit has changed */
                        selected = -(1 as INT_PTR);
                    }
                }

                cbeend.iNewSelection = selected as INT;
                cbeend.fChanged = TRUE as BOOL;
                cbeend.iWhy = CBENF_RETURN as INT;
                if COMBOEX_NotifyEndEdit(
                    infoPtr,
                    core::ptr::addr_of_mut!(cbeend),
                    core::ptr::addr_of_mut!(edit_text) as LPCWSTR,
                ) != 0
                {
                    /* abort the change, restore previous */
                    COMBOEX_SetEditText(infoPtr, core::ptr::addr_of_mut!((*infoPtr).edit));
                    RedrawWindow(
                        (*infoPtr).hwndCombo,
                        core::ptr::null::<RECT>(),
                        core::ptr::null_mut(),
                        (RDW_ERASE | RDW_INVALIDATE) as UINT,
                    );
                    return 0 as LRESULT;
                }
                oldItem = SendMessageW(
                    (*infoPtr).hwndCombo,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT_PTR;
                if oldItem != -(1 as INT_PTR) {
                    /* if something is selected, then deselect it */
                    SendMessageW(
                        (*infoPtr).hwndCombo,
                        CB_SETCURSEL as UINT,
                        (-(1 as INT_PTR)) as WPARAM,
                        0 as LPARAM,
                    );
                }
                InvalidateRect(
                    (*infoPtr).hwndCombo,
                    core::ptr::null::<RECT>(),
                    0 as BOOL,
                );
                SetFocus((*infoPtr).hwndEdit);
                break 'inner;
            }

            if (wParam as INT) == (VK_UP as INT)
                || (wParam as INT) == (VK_DOWN as INT)
            {
                let mut step: INT = if wParam == VK_DOWN as WPARAM {
                    1 as INT
                } else {
                    -(1 as INT)
                };

                oldItem = SendMessageW(
                    (*infoPtr).hwndSelf,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT_PTR;
                if oldItem >= (0 as INT_PTR)
                    && (oldItem + (step as INT_PTR)) >= (0 as INT_PTR)
                {
                    SendMessageW(
                        (*infoPtr).hwndSelf,
                        CB_SETCURSEL as UINT,
                        (oldItem + (step as INT_PTR)) as WPARAM,
                        0 as LPARAM,
                    );
                }
                return 0 as LRESULT;
            }

            /* default: */
            return DefSubclassProc(hwnd, uMsg, wParam, lParam);
        }
        return 0 as LRESULT;
    }

    if uMsg == WM_SETFOCUS as UINT {
        /* remember the focus to set state of icon */
        lret = DefSubclassProc(hwnd, uMsg, wParam, lParam);
        (*infoPtr).flags |= WCBE_EDITFOCUSED as DWORD;
        return lret;
    }

    if uMsg == WM_KILLFOCUS as UINT {
        /*
         * do NOTIFY CBEN_ENDEDIT with CBENF_KILLFOCUS
         */
        (*infoPtr).flags &= !(WCBE_EDITFOCUSED as DWORD);
        if ((*infoPtr).flags & (WCBE_ACTEDIT as DWORD)) != 0 {
            (*infoPtr).flags &= !((WCBE_ACTEDIT | WCBE_EDITCHG) as DWORD);

            GetWindowTextW(
                (*infoPtr).hwndEdit,
                core::ptr::addr_of_mut!(edit_text) as LPWSTR,
                260 as ::core::ffi::c_int,
            );
            cbeend.fChanged = FALSE as BOOL;
            cbeend.iNewSelection = SendMessageW(
                (*infoPtr).hwndCombo,
                CB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            cbeend.iWhy = CBENF_KILLFOCUS as INT;

            COMBOEX_NotifyEndEdit(
                infoPtr,
                core::ptr::addr_of_mut!(cbeend),
                core::ptr::addr_of_mut!(edit_text) as LPCWSTR,
            );
        }
        /* fall through */
    }

    /* default: */
    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "system" fn COMBOEX_ComboWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uId: UINT_PTR,
    ref_data: DWORD_PTR,
) -> LRESULT {
    let mut infoPtr: *mut COMBOEX_INFO = COMBOEX_GetInfoPtr!(ref_data as HWND);
    let mut cbeend: NMCBEENDEDITW = core::mem::zeroed();
    let mut nmmse: NMMOUSE = core::mem::zeroed();
    let mut obkc: COLORREF = 0;
    let mut hDC: HDC = core::ptr::null_mut();
    let mut focusedhwnd: HWND = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut edit_text: [WCHAR; 260] = [0; 260];

    if uMsg == WM_NCDESTROY {
        RemoveWindowSubclass(
            hwnd,
            Some(COMBOEX_ComboWndProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, UINT_PTR, DWORD_PTR) -> LRESULT),
            (COMBO_SUBCLASSID) as UINT_PTR,
        );
    }

    if infoPtr.is_null() {
        return DefSubclassProc(hwnd, uMsg, wParam, lParam);
    }

    'sw: {
        match uMsg {
            WM_DRAWITEM => {
                /*
                 * The only way this message should come is from the
                 * child Listbox issuing the message. Flag this so
                 * that ComboEx knows this is listbox.
                 */
                (*(lParam as *mut DRAWITEMSTRUCT)).itemState |= ODS_COMBOEXLBOX;
                break 'sw;
            }

            WM_ERASEBKGND => {
                hDC = wParam as HDC;
                obkc = SetBkColor(hDC, comctl32_color.clrWindow);
                GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
                ExtTextOutW(
                    hDC,
                    0,
                    0,
                    ETO_OPAQUE,
                    core::ptr::addr_of_mut!(rect),
                    core::ptr::null(),
                    0,
                    core::ptr::null(),
                );
                SetBkColor(hDC, obkc);
                break 'sw;
            }

            WM_SETCURSOR => {
                /*
                 *  WM_NOTIFY to comboex parent (rebar)
                 *   with NM_SETCURSOR with extra words of 0,0,0,0,0x02010001
                 *  CallWindowProc (previous)
                 */
                nmmse.dwItemSpec = 0;
                nmmse.dwItemData = 0;
                nmmse.pt.x = 0;
                nmmse.pt.y = 0;
                nmmse.dwHitInfo = lParam;
                COMBOEX_Notify(infoPtr, (NM_SETCURSOR) as INT, core::ptr::addr_of_mut!(nmmse) as *mut NMHDR);
                break 'sw;
            }

            WM_LBUTTONDOWN => {
                GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
                rect.bottom = rect.top + (SendMessageW(
                    (*infoPtr).hwndSelf,
                    CB_GETITEMHEIGHT,
                    -1isize as WPARAM,
                    0,
                )) as i32;
                rect.left = rect.right - GetSystemMetrics((SM_CXVSCROLL) as i32);
                pt.x = (LOWORD!(lParam as DWORD) as i16) as LONG;
                pt.y = (HIWORD!(lParam as DWORD) as i16) as LONG;
                if PtInRect(core::ptr::addr_of_mut!(rect), pt) != 0 {
                    break 'sw;
                }

                (*infoPtr).flags |= WCBE_MOUSECAPTURED;
                SetCapture(hwnd);
                return 0;
            }

            WM_LBUTTONUP => {
                if ((*infoPtr).flags & WCBE_MOUSECAPTURED) == 0 {
                    break 'sw;
                }

                ReleaseCapture();
                (*infoPtr).flags &= !WCBE_MOUSECAPTURED;
                if ((*infoPtr).flags & WCBE_MOUSEDRAGGED) != 0 {
                    (*infoPtr).flags &= !WCBE_MOUSEDRAGGED;
                } else {
                    SendMessageW(hwnd, CB_SHOWDROPDOWN, (TRUE) as WPARAM, 0);
                }
                return 0;
            }

            WM_MOUSEMOVE => {
                if ((*infoPtr).flags & WCBE_MOUSECAPTURED) != 0
                    && ((*infoPtr).flags & WCBE_MOUSEDRAGGED) == 0
                {
                    GetWindowTextW((*infoPtr).hwndEdit, edit_text.as_mut_ptr(), 260);
                    COMBOEX_NotifyDragBegin(infoPtr, edit_text.as_ptr());
                    (*infoPtr).flags |= WCBE_MOUSEDRAGGED;
                }
                break 'sw;
            }

            WM_COMMAND => {
                'isw: {
                    match (HIWORD!(wParam as DWORD)) as u32 {
                        EN_UPDATE => {
                            /* traces show that COMBOEX does not issue CBN_EDITUPDATE
                             * on the EN_UPDATE
                             */
                            return 0;
                        }

                        EN_KILLFOCUS => {
                            focusedhwnd = GetFocus();
                            if ((*infoPtr).flags & WCBE_ACTEDIT) != 0 {
                                GetWindowTextW((*infoPtr).hwndEdit, edit_text.as_mut_ptr(), 260);
                                cbeend.fChanged = ((*infoPtr).flags & WCBE_EDITCHG) as BOOL;
                                cbeend.iNewSelection = SendMessageW(
                                    (*infoPtr).hwndCombo,
                                    CB_GETCURSEL,
                                    0,
                                    0,
                                ) as INT;
                                cbeend.iWhy = (CBENF_KILLFOCUS as i32);

                                (*infoPtr).flags &= !(WCBE_ACTEDIT | WCBE_EDITCHG);
                                if COMBOEX_NotifyEndEdit(infoPtr, core::ptr::addr_of_mut!(cbeend), edit_text.as_ptr()) != 0 {
                                    return 0;
                                }
                            }
                            /* possible CB_GETCURSEL */
                            InvalidateRect((*infoPtr).hwndCombo, core::ptr::null(), 0);
                            if !focusedhwnd.is_null() {
                                SendMessageW(
                                    (*infoPtr).hwndCombo,
                                    WM_KILLFOCUS,
                                    focusedhwnd as WPARAM,
                                    0,
                                );
                            }
                            return 0;
                        }

                        EN_SETFOCUS => {
                            let mut hdr: NMHDR = core::mem::zeroed();

                            SendMessageW((*infoPtr).hwndEdit, EM_SETSEL, 0, 0);
                            SendMessageW((*infoPtr).hwndEdit, EM_SETSEL, 0, -1 as LPARAM);
                            COMBOEX_Notify(infoPtr, CBEN_BEGINEDIT, core::ptr::addr_of_mut!(hdr));
                            (*infoPtr).flags |= WCBE_ACTEDIT;
                            (*infoPtr).flags &= !WCBE_EDITCHG; /* no change yet */
                            return 0;
                        }

                        EN_CHANGE => {
                            let mut lastwrk: LPCWSTR;
                            let mut cmptext: cmp_func_t = get_cmp_func(infoPtr);

                            let mut selected: INT_PTR = SendMessageW(
                                (*infoPtr).hwndCombo,
                                CB_GETCURSEL,
                                0,
                                0,
                            );

                            /* lstrlenW( lastworkingURL ) */

                            GetWindowTextW((*infoPtr).hwndEdit, edit_text.as_mut_ptr(), 260);
                            if selected == -1 {
                                lastwrk = (*infoPtr).edit.pszText as LPCWSTR;
                            } else {
                                let mut item: *mut CBE_ITEMDATA = COMBOEX_FindItem(infoPtr, selected);
                                lastwrk = COMBOEX_GetText(infoPtr, item);
                            }

                            /* cmptext is between lastworkingURL and GetWindowText */
                            if cmptext(lastwrk, edit_text.as_ptr()) != 0 {
                                /* strings not equal -- indicate edit has changed */
                                (*infoPtr).flags |= WCBE_EDITCHG;
                            }
                            SendMessageW(
                                (*infoPtr).hwndNotify,
                                WM_COMMAND,
                                MAKEWPARAM!(GetDlgCtrlID((*infoPtr).hwndSelf), CBN_EDITCHANGE) as WPARAM,
                                (*infoPtr).hwndSelf as LPARAM,
                            );
                            return 0;
                        }

                        LBN_SELCHANGE | _ => {
                            break 'isw;
                        }
                    }
                }
                /* fall through */
            }

            _ => {
                ;
            }
        }
    }

    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "system" fn COMBOEX_WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut COMBOEX_INFO = COMBOEX_GetInfoPtr!(hwnd);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comboex.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMBOEX_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            1996 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lParam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }

    if infoPtr.is_null() {
        if uMsg == WM_CREATE {
            return COMBOEX_Create(hwnd, lParam as LPCREATESTRUCTA as *const CREATESTRUCTA);
        }
        if uMsg == WM_NCCREATE {
            COMBOEX_NCCreate(hwnd);
        }
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }

    match uMsg {
        CBEM_DELETEITEM => return COMBOEX_DeleteItem(infoPtr, wParam as INT_PTR) as LRESULT,
        CBEM_GETCOMBOCONTROL => return (*infoPtr).hwndCombo as LRESULT,
        CBEM_GETEDITCONTROL => return (*infoPtr).hwndEdit as LRESULT,
        CBEM_GETEXTENDEDSTYLE => return (*infoPtr).dwExtStyle as LRESULT,
        CBEM_GETIMAGELIST => return (*infoPtr).himl as LRESULT,
        CBEM_GETITEMA => {
            return COMBOEX_GetItemA(infoPtr, lParam as *mut COMBOBOXEXITEMA) as LRESULT;
        }
        CBEM_GETITEMW => {
            return COMBOEX_GetItemW(infoPtr, lParam as *mut COMBOBOXEXITEMW) as LRESULT;
        }
        CBEM_GETUNICODEFORMAT => return (*infoPtr).unicode as LRESULT,
        CBEM_HASEDITCHANGED => return COMBOEX_HasEditChanged(infoPtr) as LRESULT,
        CBEM_INSERTITEMA => {
            return COMBOEX_InsertItemA(infoPtr, lParam as *mut COMBOBOXEXITEMA) as LRESULT;
        }
        CBEM_INSERTITEMW => {
            return COMBOEX_InsertItemW(infoPtr, lParam as *mut COMBOBOXEXITEMW) as LRESULT;
        }
        CBEM_SETEXSTYLE | CBEM_SETEXTENDEDSTYLE => {
            return COMBOEX_SetExtendedStyle(infoPtr, wParam as DWORD, lParam as DWORD) as LRESULT;
        }
        CBEM_SETIMAGELIST => {
            return COMBOEX_SetImageList(infoPtr, lParam as HIMAGELIST) as LRESULT;
        }
        CBEM_SETITEMA => {
            return COMBOEX_SetItemA(infoPtr, lParam as *mut COMBOBOXEXITEMA) as LRESULT;
        }
        CBEM_SETITEMW => {
            return COMBOEX_SetItemW(infoPtr, lParam as *mut COMBOBOXEXITEMW) as LRESULT;
        }
        CBEM_SETUNICODEFORMAT => {
            return COMBOEX_SetUnicodeFormat(infoPtr, wParam as BOOL) as LRESULT;
        }
        WM_SETTEXT | WM_GETTEXT | WM_GETTEXTLENGTH => {
            return SendMessageW((*infoPtr).hwndEdit, uMsg, wParam, lParam);
        }
        CB_GETLBTEXT => {
            return COMBOEX_GetListboxText(infoPtr, wParam as INT_PTR, lParam as LPWSTR) as LRESULT;
        }
        CB_GETLBTEXTLEN => {
            return COMBOEX_GetListboxText(infoPtr, wParam as INT_PTR, core::ptr::null_mut()) as LRESULT;
        }
        CB_RESETCONTENT => {
            COMBOEX_ResetContent(infoPtr);
            /* fall through */
        }
        /* Combo messages we are not sure if we need to process or just forward */
        CB_GETDROPPEDCONTROLRECT
        | CB_GETITEMHEIGHT
        | CB_GETEXTENDEDUI
        | CB_LIMITTEXT
        | CB_SELECTSTRING
        /* Combo messages OK to just forward to the regular COMBO */
        | CB_GETCOUNT
        | CB_GETCURSEL
        | CB_GETDROPPEDSTATE
        | CB_SETDROPPEDWIDTH
        | CB_SETEXTENDEDUI
        | CB_SHOWDROPDOWN => {}
        /* Combo messages we need to process specially */
        CB_FINDSTRINGEXACT => {
            return COMBOEX_FindStringExact(infoPtr, wParam as INT, lParam as LPCWSTR) as LRESULT;
        }
        CB_GETITEMDATA => {
            return COMBOEX_GetItemData(infoPtr, (wParam as INT) as INT_PTR) as LRESULT;
        }
        CB_SETCURSEL => {
            return COMBOEX_SetCursel(infoPtr, (wParam as INT) as INT_PTR) as LRESULT;
        }
        CB_SETITEMDATA => {
            return COMBOEX_SetItemData(infoPtr, (wParam as INT) as INT_PTR, lParam as DWORD_PTR) as LRESULT;
        }
        CB_SETITEMHEIGHT => {
            return COMBOEX_SetItemHeight(infoPtr, wParam as INT, lParam as UINT) as LRESULT;
        }
        /* Window messages passed to parent */
        WM_COMMAND => {
            return COMBOEX_Command(infoPtr, wParam);
        }
        WM_NOTIFY => {
            if (*infoPtr).NtfUnicode != 0 {
                return SendMessageW((*infoPtr).hwndNotify, uMsg, wParam, lParam);
            } else {
                return SendMessageA((*infoPtr).hwndNotify, uMsg, wParam, lParam);
            }
        }
        /* Window messages we need to process */
        WM_DELETEITEM => {
            return COMBOEX_WM_DeleteItem(infoPtr, lParam as *mut DELETEITEMSTRUCT) as LRESULT;
        }
        WM_DRAWITEM => {
            return COMBOEX_DrawItem(infoPtr, lParam as *mut DRAWITEMSTRUCT);
        }
        WM_DESTROY => {
            return COMBOEX_Destroy(infoPtr);
        }
        WM_ENABLE => {
            return COMBOEX_Enable(infoPtr, wParam as BOOL);
        }
        WM_MEASUREITEM => {
            return COMBOEX_MeasureItem(infoPtr, lParam as *mut MEASUREITEMSTRUCT);
        }
        WM_NOTIFYFORMAT => {
            return COMBOEX_NotifyFormat(infoPtr, lParam);
        }
        WM_SIZE => {
            return COMBOEX_Size(infoPtr, LOWORD!(lParam) as INT, HIWORD!(lParam) as INT);
        }
        WM_GETFONT => return (*infoPtr).font as LRESULT,
        WM_SETFONT => {
            return COMBOEX_SetFont(
                infoPtr,
                wParam as HFONT,
                (LOWORD!(lParam) as INT != 0) as BOOL,
            );
        }
        WM_SETREDRAW => {
            return COMBOEX_SetRedraw(infoPtr, wParam, lParam);
        }
        WM_WINDOWPOSCHANGING => {
            return COMBOEX_WindowPosChanging(infoPtr, lParam as *mut WINDOWPOS);
        }
        WM_SETFOCUS => {
            if !(*infoPtr).hwndEdit.is_null() {
                SetFocus((*infoPtr).hwndEdit);
            } else {
                SetFocus((*infoPtr).hwndCombo);
            }
            return 0 as LRESULT;
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && COMCTL32_IsReflectedMessage(uMsg) == 0
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/comboex.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"COMBOEX_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        2165 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }

    return SendMessageW((*infoPtr).hwndCombo, uMsg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn COMBOEX_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        &raw mut wndClass as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc = Some(COMBOEX_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut COMBOEX_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wndClass.lpszClassName = (WC_COMBOBOXEXW).as_ptr() as LPCWSTR;

    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn COMBOEX_Unregister() {
    UnregisterClassW((WC_COMBOBOXEXW).as_ptr(), core::ptr::null_mut());
}
